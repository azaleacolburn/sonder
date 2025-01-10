use std::{
    cell::{RefCell, RefMut},
    cmp,
    collections::HashMap,
    ops::Range,
    rc::Rc,
};

use crate::{
    annotater::FieldDefinition,
    ast::{NodeType, TokenNode as Node},
    lexer::CType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtrType {
    Rc,
    RcRefClone,
    RefCell,
    RawPtrMut,
    RawPtrImut,
    MutRef,
    ImutRef,
}

/// The top-level datastructure that stores data about all the variables and referencing
/// Stores a vector of the instances of addresses being taken, in order
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisContext {
    pub variables: HashMap<String, VarData>,
    pub addresses: Vec<Rc<RefCell<AdrData>>>,
    pub structs: HashMap<String, StructData>,
}

impl AnalysisContext {
    pub fn new() -> AnalysisContext {
        AnalysisContext {
            variables: HashMap::new(),
            addresses: vec![],
            structs: HashMap::new(),
        }
    }
    pub fn new_var(&mut self, id: String, data: VarData) {
        self.variables.insert(id, data);
    }
    pub fn new_adr(&mut self, adr: Rc<RefCell<AdrData>>, var: Option<String>) {
        self.addresses.push(adr.clone());

        if let Some(var) = var {
            self.variables.entry(var).and_modify(|var_data| {
                var_data.addresses.push(adr.clone());
            });
        }
    }

    pub fn new_struct(&mut self, id: String, struct_data: StructData) {
        self.structs.insert(id, struct_data);
    }

    pub fn get_struct(&self, id: &str) -> &StructData {
        self.structs.get(id).expect("Struct not in map")
    }

    pub fn get_var(&self, id: &str) -> &VarData {
        self.variables.get(id).expect("Var not in map")
    }

    /// Gets an address, given the id the address points to
    /// If more than one exists, the first one is returned
    pub fn get_adr(&self, var_id: &str) -> &Rc<RefCell<AdrData>> {
        self.addresses
            .iter()
            .find(|adr_data| adr_data.borrow().adr_of == var_id)
            .expect("Address not in map")
    }

    pub fn mut_var<F>(&mut self, id: String, f: F)
    where
        F: FnOnce(&mut VarData),
    {
        self.variables.entry(id).and_modify(f);
    }

    /// Applies a function to an adr_data given the underlying id the adr points to
    pub fn mut_adr<F>(&mut self, id: String, f: F)
    where
        F: FnOnce(RefMut<AdrData>),
    {
        let adr_data = self
            .addresses
            .iter_mut()
            .map(|adr_cell| adr_cell.clone())
            .find(|adr_data| adr_data.borrow().adr_of == id)
            .expect(format!("No adr that points to given id: {id}").as_str());

        f(adr_data.borrow_mut())
    }

    pub fn is_ptr(&self, id: &String) -> bool {
        self.variables
            .get(id)
            .as_ref()
            .expect("Checked ptr not in ctx")
            .addresses
            .len()
            > 0
    }
    pub fn traverse_pointer_chain<T>(&self, root: String, total_depth: u8, max_depth: u8) -> T
    where
        T: Iterator<Item = String> + Extend<String>,
    {
        if total_depth == max_depth {
            return [].into_iter() as T;
        }
        let ptr_data = &self
            .variables
            .get(&root)
            .as_ref()
            .expect("Root in traversing ptr chain not found in map")
            .addresses;

        match ptr_data.is_empty() {
            false => {
                let t: T = self.traverse_pointer_chain::<T>(
                    ptr_data.last().unwrap().borrow().adr_of.clone(),
                    total_depth + 1,
                    max_depth,
                );
                t.extend(std::iter::once(root.to_string()));
                t
            }
            true => [root.to_string()].into_iter(),
        }
    }
    /// Finds which reference a specific variable held at the given line number
    /// Panics if:
    /// - Variable was declared on line given <- up for question
    /// - Variable doesn't exist in variables
    /// - Variable doesn't exist on given line
    /// - Variable not a ptr or never initialized
    pub fn find_which_ref_at_id(&self, var_id: &str, line: usize) -> String {
        let init_at = self
            .variables
            .get(var_id)
            .expect("Variable given doesn't exist")
            .non_borrowed_lines[0]
            .start;
        // TODO: Check if this should be > or >=
        println!("var_id: {var_id} init_at: {init_at} line: {line}");
        assert!(init_at < line);

        self.variables
            .get(var_id)
            .expect("Variable given doesn't exist")
            .addresses
            .iter()
            .map(|adr_data| adr_data.borrow())
            .filter(|adr_data_ref| adr_data_ref.line_taken < 1)
            .fold(String::new(), |_, adr_data_ref| adr_data_ref.adr_of.clone())
    }

    pub fn print_refs(&self) {
        self.variables.iter().for_each(|(id, var_data)| {
            println!("{id}:");
            var_data
                .non_borrowed_lines
                .iter()
                .for_each(|non_borrowed_range| println!("\t{:?}", non_borrowed_range))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructData {
    pub field_definitions: Vec<FieldDefinition>,
}

/// Data of a specific instance of the address of a variable being taken
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrData {
    pub adr_of: String,
    pub mutates: bool,
    pub held_by: Option<String>,
    // Determine if ptrtype makes sense in a variable-independent context
    pub ptr_type: Vec<PtrType>,
    pub line_taken: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarData {
    // An emptry vector indicates a non-ptr variable
    pub addresses: Vec<Rc<RefCell<AdrData>>>,
    // The type of ptr here is relevent for annotating adr
    // Reference order matters here, so we have to be careful
    pub pointed_to_by: Vec<String>,
    pub is_mut_by_ptr: bool,
    pub is_mut_direct: bool,
    pub rc: bool,
    pub clone: bool,
    pub set_start_borrow: bool, // do we need to set the start of the new borrow
    // The pattern of initializing and instantiating seperately is harder to analyze and requires a PtrAssignment node
    // Line ranges when the var isn't borrowed
    pub non_borrowed_lines: Vec<Range<usize>>,

    // Struct handling
    pub instanceof_struct: Option<String>,
    pub fieldof_struct: Option<FieldInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldInfo {
    struct_id: String,
    field_id: String,
}

impl VarData {
    pub fn add_non_borrowed_line(&mut self, line: usize) {
        let len = self.non_borrowed_lines.len() - 1;
        let end = &mut self.non_borrowed_lines[len].end;
        *end = cmp::max(line, *end);
        if self.set_start_borrow {
            let start = &mut self.non_borrowed_lines[len].start;
            *start = cmp::max(line, *start);
            self.set_start_borrow = false;
        }
    }
    pub fn new_borrow(&mut self, line: usize) {
        self.add_non_borrowed_line(line);
        self.non_borrowed_lines.push(Range {
            start: line,
            end: line,
        });
        self.set_start_borrow = true;
    }
}

pub fn determine_var_mutability<'a>(root: &'a Node, ctx: &mut AnalysisContext) {
    if root.children.is_some() {
        root.children.as_ref().unwrap().iter().for_each(|node| {
            determine_var_mutability(node, ctx);
        })
    }

    match &root.token {
        NodeType::Declaration(id, c_type, _) => {
            let instanceof_struct = if let CType::Struct(struct_id) = c_type {
                Some(struct_id.clone())
            } else {
                None
            };
            println!("Declaration: {id}");
            ctx.new_var(
                id.to_string(),
                VarData {
                    addresses: vec![],
                    pointed_to_by: vec![],
                    is_mut_by_ptr: false,
                    is_mut_direct: false,
                    rc: false,
                    non_borrowed_lines: vec![Range {
                        start: root.line,
                        end: root.line,
                    }],
                    set_start_borrow: false,
                    clone: false,
                    instanceof_struct,
                    // A different node (StructFieldAssignment) will handle `my_struct.id = true`
                    fieldof_struct: None,
                },
            );
        }
        NodeType::Assignment(_, id) => {
            // The first that we have to do is determine if what we're assigning to is a pointer
            let var_data = ctx.get_var(id.as_str());
            let is_ptr = var_data.addresses.len() > 0;
            if is_ptr {
                let ids =
                    find_ids(&root.children.as_ref().expect("Assignment missing children")[0]);
                let expr_ptrs: Vec<&String> = ids.iter().filter(|id| ctx.is_ptr(id)).collect();
                let mut ptr_type_chain: Vec<PtrType> = if expr_ptrs.len() == 1 {
                    // As we go, we replace certain elements in this vector with `PtrType::MutRef`
                    ctx.traverse_pointer_chain(expr_ptrs[0].clone(), 0, u8::MAX)
                        .iter()
                        .map(|_| PtrType::ImutRef)
                        .collect()
                } else if expr_ptrs.len() > 1 {
                    // Ptr arithmatic outside the context of arrays is automatically a raw ptr
                    vec![PtrType::RawPtrImut]
                } else {
                    vec![PtrType::ImutRef]
                };

                let addresses =
                    find_addresses(&root.children.as_ref().expect("Assignment missing child")[0]);
                // TODO: Add ptr chain len for non-overlapping ptrs
                let points_to = match addresses.len() {
                    1 => addresses.last(),
                    0 if expr_ptrs.len() != 0 => expr_ptrs.last().map(|n| &**n),
                    n if n > 1 => {
                        if let Some(last) = ptr_type_chain.last_mut() {
                            *last = PtrType::RawPtrImut;
                        }
                        addresses.last()
                    }
                    _ => panic!(".len() is a usize; expr_ptr.len: {}", expr_ptrs.len()),
                }
                .unwrap();

                ctx.mut_adr(points_to.to_string(), |mut adr_data| {
                    adr_data.held_by = Some(id.clone());
                });
                ctx.mut_var(points_to.to_string(), |var_data| {
                    var_data.pointed_to_by.push(id.clone());
                });
            }

            ctx.mut_var(id.to_string(), |var_data| {
                var_data.is_mut_direct = true;
                var_data.add_non_borrowed_line(root.line);
            });
        }
        NodeType::PtrDeclaration(id, c_type, expr) => {
            determine_var_mutability(expr, ctx);

            let ids = find_ids(&expr);
            let expr_ptrs: Vec<&String> = ids.iter().filter(|id| ctx.is_ptr(id)).collect();

            let mut ptr_type_chain: Vec<PtrType> = if expr_ptrs.len() == 1 {
                // As we go, we replace certain elements in this vector with `PtrType::MutRef`
                ctx.traverse_pointer_chain(expr_ptrs[0].clone(), 0, u8::MAX)
                    .iter()
                    .map(|_| PtrType::ImutRef)
                    .collect()
            } else if expr_ptrs.len() > 1 {
                // Ptr arithmatic outside the context of arrays is automatically a raw ptr
                vec![PtrType::RawPtrImut]
            } else {
                vec![PtrType::ImutRef]
            };

            let addresses = find_addresses(&expr);
            // TODO: Add ptr chain len for non-overlapping ptrs
            let points_to = match addresses.len() {
                1 => addresses.last(),
                0 if expr_ptrs.len() != 0 => expr_ptrs.last().map(|n| &**n),
                n if n > 1 => {
                    if let Some(last) = ptr_type_chain.last_mut() {
                        *last = PtrType::RawPtrImut;
                    }
                    addresses.last()
                }
                _ => {
                    panic!(".len() is a usize; expr_ptr.len: {}", expr_ptrs.len());
                }
            }
            .unwrap();

            let adr_data = ctx.get_adr(&points_to).clone();
            adr_data.borrow_mut().held_by = Some(id.clone());

            // Check if struct ptr
            let instanceof_struct = if let CType::Struct(struct_id) = c_type {
                Some(struct_id.clone())
            } else {
                None
            };

            let var = VarData {
                addresses: vec![adr_data],
                pointed_to_by: vec![],
                is_mut_by_ptr: false,
                is_mut_direct: false,
                rc: false,
                non_borrowed_lines: vec![Range {
                    start: root.line,
                    end: root.line,
                }],
                set_start_borrow: false,
                clone: false,
                instanceof_struct,
                // This will be handled by other node (StructFieldPtrDeclaration)
                fieldof_struct: None,
            };
            // TODO: Figure out how to annotate specific address call as mutable or immutable
            ctx.new_var(id.to_string(), var);
            // Doesn't support &that + &this
            // This immediantly breakes borrow checking rules
            ctx.mut_var(points_to.to_string(), |var_data| {
                var_data.pointed_to_by.push(id.clone());
            });
        }
        NodeType::DerefAssignment(_, l_side) => {
            determine_var_mutability(&l_side, ctx);
            let deref_ids = find_ids(&l_side);
            // This breakes because `*(t + s) = bar` is not allowed
            // However, **m is fine
            if deref_ids.len() > 1 {
                panic!("Unsupported: Multiple items dereferenced");
            } else if deref_ids.len() != 1 {
                panic!("Unsupported: no_ids being dereffed")
            }
            let num_of_vars = count_derefs(&l_side) + 1;
            let mut ptr_chain = ctx
                .traverse_pointer_chain(deref_ids[0].clone(), 0, num_of_vars)
                .into_iter()
                .rev();
            // eg. [m, p, n]
            let first_ptr = ptr_chain.next().expect("No pointers in chain");
            ctx.mut_var(first_ptr.clone(), |var_data| {
                var_data.add_non_borrowed_line(root.line);
                let adr = var_data.addresses.last().expect("Variable not ptr");
                adr.borrow_mut().mutates = true;
                adr.borrow_mut().ptr_type.fill(PtrType::MutRef);
            });

            ptr_chain.clone().enumerate().for_each(|(i, var)| {
                if i != ptr_chain.len() - 1 {
                    ctx.mut_var(var.clone(), |var_data| {
                        let mut adr = var_data
                            .addresses
                            .last()
                            .expect("Variable not ptr")
                            .borrow_mut();

                        adr.mutates = true;

                        (i..adr.ptr_type.len()).for_each(|type_chain_i| {
                            adr.ptr_type[type_chain_i] = PtrType::MutRef;
                        });
                    });
                }
                ctx.mut_var(var, |var_data| var_data.is_mut_by_ptr = true);
            });
        }
        NodeType::Id(id) => {
            ctx.mut_var(id.to_string(), |var_data| {
                var_data.add_non_borrowed_line(root.line)
            });
        }
        NodeType::Adr(id) => {
            let ptr_type_chain = ctx
                .traverse_pointer_chain(id.clone(), 0, u8::MAX)
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
        NodeType::StructDefinition(struct_id, declarations) => {
            let field_definitions: Vec<FieldDefinition> = declarations
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
        NodeType::StructFieldAssignment {
            var_id,
            field_id,
            assignment_op,
            expr,
        } => {
            let var_data = ctx.get_var(var_id);
            let struct_data = var_data
                .instanceof_struct
                .expect("Struct defintion parent not instance of struct in ctx");
        }
        _ => {}
    };
}

pub fn find_addresses(root: &Node) -> Vec<String> {
    let mut vec = match root.children.as_ref() {
        Some(children) => children
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
        count += children.iter().map(count_derefs).sum::<u8>();
    }
    match &root.token {
        NodeType::DeRef(expr) => count += count_derefs(&expr) + 1,
        _ => {}
    };
    count
}

pub fn find_type_ids<'a>(root: &'a Node) -> Vec<(String, CType)> {
    let mut type_ids: Vec<(String, CType)> = root
        .children
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .flat_map(find_type_ids)
        .collect();
    match &root.token {
        NodeType::Declaration(id, c_type, _size) => type_ids.push((id.clone(), c_type.clone())),
        _ => {}
    };
    type_ids
}

pub fn find_ids<'a>(root: &'a Node) -> Vec<String> {
    let mut ids: Vec<String> = root
        .children
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .flat_map(find_ids)
        .collect();
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
    let mut ptr_types: Vec<PtrType> = root
        .children
        .as_ref()
        .unwrap_or(&Vec::new())
        .iter()
        .flat_map(count_declaration_ref)
        .collect();
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
