use std::{
    cell::{RefCell, RefMut},
    cmp,
    collections::HashMap,
    ops::Range,
    rc::Rc,
};

use crate::{
    analysis_ctx::AnalysisContext,
    annotater::FieldDefinition,
    ast::{NodeType, TokenNode as Node},
    data_model::{FieldInfo, PtrType, StructData, VarData},
    lexer::CType,
};

pub fn determine_var_mutability<'a>(
    root: &'a Node,
    ctx: &mut AnalysisContext,
    parent_children: Rc<RefCell<Box<[Node]>>>,
    root_index: usize,
) {
    // NOTE Let nodes handle their own children
    // This is going to introduce a few bugs were I forget to recurse, but is necessary for
    // ancestors to have access to child reference information and to avoid traversing pointer
    // chains and pointer counting along nodes
    //
    //
    if let Some(children) = root.children.as_ref() {
        children.borrow().iter().enumerate().for_each(|(i, node)| {
            // WARNING This assumes that
            // an rc can be cloned while the refcell is borrowed

            determine_var_mutability(node, ctx, children.clone(), i);
        })
    }

    match &root.token {
        NodeType::Declaration(id, c_type, _) => {
            let instanceof_struct = match c_type {
                CType::Struct(struct_id) => Some(struct_id.clone()),
                _ => None,
            };

            let variable = VarData::new(c_type.clone(), false, instanceof_struct, None);

            ctx.declaration(id, variable);
        }
        NodeType::Assignment(_, id) => handle_assignment_analysis(ctx, id, root),
        NodeType::PtrDeclaration(id, c_type, expr) => {
            // TODO Determine if this is needed (I think not)
            // determine_var_mutability(expr, ctx, parent_children, root_index);
            let rm = 5;

            // Check if struct ptr
            let instanceof_struct = if let CType::Struct(struct_id) = c_type {
                Some(struct_id.clone())
            } else {
                None
            };

            let borrowed = ptr_from_expression(root, ctx);
            let v = VarData::new(c_type.clone(), false, instanceof_struct, None);

            ctx.declaration(id, v);
            ctx.ptr_assignment(&borrowed, id, root.line);
        }
        NodeType::DerefAssignment(_, l_side) => {
            // determine_var_mutability(&l_side, ctx, parent_children, root_index);
            let deref_ids = find_ids(&l_side);
            // NOTE `*(t + s) = bar` is not allowed
            // However, ``**m` is fine
            if deref_ids.len() > 1 {
                panic!("Unsupported: Multiple items dereferenced");
            } else if deref_ids.len() != 1 {
                panic!("Unsupported: no_ids being dereffed")
            }
            let number_of_derefs = count_derefs(&l_side) + 1;
            let mut ptr_chain = ctx
                .construct_ptr_chain(deref_ids[0].clone(), 0, number_of_derefs)
                .into_iter()
                .rev();
            let first_ptr = ptr_chain.next().expect("No pointers in chain");

            // NOTE Sets every ptr + reference in chain to also be mut
            let set_ptr_mut = |ptr_id: String| {
                ctx.mut_var(ptr_id, |ptr_var| {
                    ptr_var.is_mut = true;
                    ptr_var
                        .current_reference_held()
                        .expect("No reference held by ptr in chain")
                        .borrow_mut()
                        .set_mut();
                })
            };
            set_ptr_mut(first_ptr.clone());
            ptr_chain.for_each(set_ptr_mut);

            ctx.deref_assignment(&first_ptr, root.line);
        }
        NodeType::Id(id) => {
            ctx.mut_var(id.to_string(), |var_data| {
                var_data.new_usage(root.line);
            });
        }
        NodeType::Adr(id) => {
            let ptr_type_chain = ctx
                .construct_ptr_chain(id.clone(), 0, u8::MAX)
                .iter()
                .map(|_| PtrType::ImutRef)
                .collect();
            let adr_data = Rc::new(RefCell::new(AdrData {
                adr_of: id.to_string(),
                mutates: false,
                held_by: None,
                ptr_type: ptr_type_chain,
                line_taken: root.line,
            }));
            // We don't know if a variable owns this ref yet
            // that's for the ptr_declaration to figure out
            ctx.new_adr(adr_data, None);
            println!("NEW BORROW: {}", id);
            ctx.mut_var(id.to_string(), |var_data| var_data.new_borrow(root.line));
        }
        NodeType::DeRef(adr) => {
            let ids = find_ids(&adr);
            // Panics if more than one id derefed
            if ids.len() != 1 {
                panic!("more than one or 0 ids derefed");
            }
            let id = ids[0].clone();
            ctx.mut_var(id, |var_data| var_data.add_non_borrowed_line(root.line));
        }
        NodeType::StructDefinition {
            struct_id,
            field_definitions,
        } => {
            let field_definitions: Vec<FieldDefinition> = field_definitions
                .into_iter()
                .map(|(id, ptr_count, c_type)| {
                    // TODO: Update according to corresponding variables as we analyze
                    let ptr_type = (0..*ptr_count).map(|_| PtrType::ImutRef).collect();
                    FieldDefinition {
                        id: id.clone(),
                        c_type: c_type.clone(),
                        ptr_type,
                    }
                })
                .collect();
            ctx.new_struct(struct_id.to_string(), StructData { field_definitions });
        }
        NodeType::StructDeclaration {
            var_id,
            struct_id,
            exprs,
        } => {
            let struct_data = ctx.get_struct(struct_id).clone(); // ugh

            assert_eq!(exprs.len(), struct_data.field_definitions.len());
            struct_data
                .field_definitions
                .iter()
                .enumerate()
                .for_each(|(_i, field)| {
                    ctx.new_var(
                        format!("{}.{}", var_id, field.id),
                        VarData {
                            pointed_to_by: vec![],
                            is_mut_by_ptr: false,
                            is_mut_direct: false,
                            rc: false,
                            clone: false,
                            set_start_borrow: false,
                            non_borrowed_lines: vec![Range {
                                start: root.line,
                                end: root.line,
                            }],
                            // NOTE Right now we only support primitives as struct field types
                            // Use this instead
                            // Some(struct_data.field_definitions[i].c_type)
                            instanceof_struct: None,
                            fieldof_struct: Some(FieldInfo {
                                struct_id: struct_id.clone(),
                                field_id: field.id.clone(),
                            }),
                            same_line_usage_array_and_index: Vec::new(),
                        },
                    )
                });

            let var_data = VarData {
                addresses: vec![],
                pointed_to_by: vec![],
                is_mut_by_ptr: false,
                is_mut_direct: false,
                rc: false,
                clone: false,
                set_start_borrow: false,
                non_borrowed_lines: vec![Range {
                    start: root.line,
                    end: root.line,
                }],
                instanceof_struct: Some(struct_id.clone()),
                fieldof_struct: None,
                same_line_usage_array_and_index: Vec::new(),
            };
            ctx.new_var(var_id.clone(), var_data);
        }
        NodeType::StructFieldAssignment {
            var_id,
            field_id,
            assignment_op: _,
            expr: _,
        } => {
            let field_var_id = format!("{var_id}.{field_id}");
            // Handle the field as a variable itself
            handle_assignment_analysis(ctx, field_var_id.as_str(), &root);

            // Apply mutability checking to the struct instance itself as well

            let var_data = ctx.get_var(format!("{var_id}.{field_id}").as_str());
            let (direct, ptr) = (var_data.is_mut_direct, var_data.is_mut_by_ptr);

            ctx.mut_var(var_id.clone(), |var_data| {
                var_data.is_mut_direct |= direct;
                var_data.is_mut_by_ptr |= ptr;
            });

            // NOTE We don't need to apply mutability checking to the struct fields themselves
        }
        _ => {}
    };
}

/// Finds Adrs taken in an expression
pub fn find_addresses(root: &Node) -> Vec<String> {
    let mut vec = match root.children.as_ref() {
        Some(children) => children
            .borrow()
            .iter()
            .flat_map(find_addresses)
            .collect::<Vec<String>>(),
        None => vec![],
    };
    match &root.token {
        NodeType::Adr(id) => vec.push(id.to_string()),
        _ => {}
    }
    vec
}

pub fn count_derefs(root: &Node) -> u8 {
    let mut count = 0;
    let children = root.children.as_ref();
    if let Some(children) = children {
        count += children.borrow().iter().map(count_derefs).sum::<u8>();
    }
    match &root.token {
        NodeType::DeRef(expr) => count += count_derefs(&expr) + 1,
        _ => {}
    };
    count
}

pub fn find_type_ids<'a>(root: &'a Node) -> Vec<(String, CType)> {
    let mut type_ids: Vec<(String, CType)> = if let Some(children) = root.children.as_ref() {
        children.borrow().iter().flat_map(find_type_ids).collect()
    } else {
        vec![]
    };
    match &root.token {
        NodeType::Declaration(id, c_type, _size) => type_ids.push((id.clone(), c_type.clone())),
        _ => {}
    };
    type_ids
}

pub fn find_ids<'a>(root: &'a Node) -> Vec<String> {
    let mut ids = match root.children.as_ref() {
        Some(children) => children.borrow().iter().flat_map(find_ids).collect(),
        None => vec![],
    };

    match &root.token {
        NodeType::Id(id) => ids.push(id.to_string()),
        NodeType::Adr(id) => ids.push(id.to_string()),
        NodeType::DeRef(node) => ids.append(&mut find_ids(&*node)),
        _ => {}
    }

    ids
}

// An empty vector represents a non ptr
pub fn count_declaration_ref(root: &Node) -> Vec<PtrType> {
    let mut ptr_types = match root.children.as_ref() {
        Some(children) => children
            .borrow()
            .iter()
            .flat_map(count_declaration_ref)
            .collect(),
        None => vec![],
    };

    match &root.token {
        NodeType::PtrDeclaration(_id, _c_type, _expr) => {
            // TODO
            // This will be edited as we go by the analyzer
            // Ideally, struct declarations will be handled first
            // Meaning they'll be placed first in the ast by the parser
            ptr_types.push(PtrType::ImutRef);
        }
        _ => {}
    };
    ptr_types
}

pub fn handle_assignment_analysis(ctx: &mut AnalysisContext, id: &str, root: &Node) {
    let lvalue = ctx.get_var(id);
    if lvalue.is_ptr() {
        let points_to = &ptr_from_expression(root, ctx);
        ctx.ptr_assignment(points_to, id, root.line);
    } else {
        ctx.assignment(id);
    }
}

// All Refs are Adr
fn ptr_type_chain(rvalue_ptrs: &Vec<String>, ctx: &mut AnalysisContext) -> Vec<PtrType> {
    if rvalue_ptrs.len() == 1 {
        // NOTE  As we go, we replace certain elements in this vector with `PtrType::MutRef`
        ctx.construct_ptr_chain(rvalue_ptrs[0].clone(), 0, u8::MAX)
            .iter()
            .map(|_| PtrType::ImutRef)
            .collect()
    } else if rvalue_ptrs.len() > 1 {
        // Ptr arithmatic outside the context of arrays is automatically a raw ptr
        vec![PtrType::RawPtrImut]
    } else {
        vec![PtrType::ImutRef]
    }
}

/// Assumes that there are only ever either derefereces or refs in rvalue
/// Returns the name of the variable this expression evaluates to
fn ptr_from_expression(root: &Node, ctx: &mut AnalysisContext) -> String {
    let rvalue_ids = find_ids(
        &root
            .children
            .as_ref()
            .expect("Assignment missing children")
            .borrow()[0],
    );

    let rvalue_ptrs: Vec<String> = rvalue_ids
        .into_iter()
        .filter(|id| ctx.get_var(id).is_ptr())
        .collect();

    let mut ptr_type_chain = ptr_type_chain(&rvalue_ptrs, ctx);
    let addresses: Vec<String> = find_addresses(
        &root
            .children
            .as_ref()
            .expect("Assignment missing child")
            .borrow()[0],
    );

    match addresses.len() {
        1 => addresses[0].clone(),
        // 0 if rvalue_ptrs.len() != 0 => rvalue_ptrs.last(),
        _ => {
            if let Some(last) = ptr_type_chain.last_mut() {
                *last = PtrType::RawPtrImut;
            }
            addresses.last().unwrap().clone()
        }
    }
}
