use crate::{
    analysis_ctx::AnalysisContext,
    ast::TokenNode as Node,
    data_model::{LineNumber, Reference, ReferenceType, Usage, VarData},
};
use std::ops::Range;

// TODO: Derermine if overlapping value uses mutate or don't mutate
// If it doesn't mutate, clone the underlying value instead
#[derive(Debug, Clone)]
pub enum BorrowError {
    MutMutOverlap {
        first_ptr_id: String,
        second_ptr_id: String,
        value_id: String,
    },
    MutImutOverlap {
        mut_ptr_id: String,
        imut_ptr_id: String,
        value_id: String,
    },
    MutMutSameLine {
        first_ptr_id: String,
        second_ptr_id: String,
        value_id: String,
    },
    MutImutSameLine {
        mut_ptr_id: String,
        imut_ptr_id: String,
        value_id: String,
    },

    ValueMutOverlap {
        ptr_id: String,
        value_id: String,
    },
    ValueMutSameLine {
        ptr_id: String,
        value_id: String,
        // value_instance_nodes: Vec<(Rc<RefCell<Box<[Node]>>>, usize)>,
    },
}

fn set_ptr_rc(value_id: &str, ctx: &mut AnalysisContext) {
    let var_data = ctx.get_var_mut(value_id);
    var_data.rc = true;

    let ptrs = var_data.pointed_to.clone();

    ptrs.iter().for_each(|reference_block| {
        reference_block.borrow_mut().set_rc();
        // TODO Check if we have to cascade RCs
        // set_ptr_rc(reference_block.borrow().get_borrower(), ctx);
    });
}

/// This function is problematic because it requires the ast to be changed :(
/// Either that, or we could use some othe protocole for conveying a new variable
/// Or, we could not add a new variable because we're weak and don't want to change business logic
/// If we insert, we need to be able to modify the ast here
// fn create_clone(
//     value_id: &str,
//     _ptr_id: &str,
//     ctx: &mut AnalysisContext,
//     root: &mut Node,
//     value_instance_nodes: Vec<(Rc<RefCell<Box<[Node]>>>, usize)>,
// ) {
//     let var_data = ctx.get_var(value_id);
//     // TODO The make this to be cloned in annotation
//     let clone_expr = Node::new(NodeType::Id(value_id.to_string()), None, 0);
//     // TODO Get CType
//     let clone_id = format!("{}_clone", value_id);
//     let clone_declaration = Node::new(
//         NodeType::Declaration(clone_id.clone(), CType::Int, var_data.addresses.len()),
//         Some(Rc::new(RefCell::new(Box::new([clone_expr])))),
//         0,
//     );
//     // This symbol goes after the new node
//     let place_before_symbol = &var_data.pointed_to_by[0];
//
//     fn search(root: &Node, place_before_symbol: &str) -> Option<(Node, usize)> {
//         match root.children.as_ref() {
//             Some(children) => {
//                 for (i, child) in children.borrow().iter().enumerate() {
//                     println!("child token: {:?}", child.token);
//                     match &child.token {
//                         NodeType::Declaration(var_id, _, _) if *var_id == place_before_symbol => {
//                             return Some((root.clone(), i));
//                         }
//                         NodeType::PtrDeclaration(var_id, _, _)
//                             if *var_id == place_before_symbol =>
//                         {
//                             return Some((root.clone(), i));
//                         }
//                         _ => {}
//                     }
//                     if let Some(parent) = search(child, place_before_symbol) {
//                         return Some(parent);
//                     };
//                 }
//             }
//             None => return None,
//         }
//         None
//     }
//
//     // Nodes that are on the same line as other nodes that reference them
//     let same_line_nodes = value_instance_nodes.iter().filter(|nodes| {
//         let node = &nodes.0.borrow()[nodes.1];
//         value_instance_nodes
//             .iter()
//             .any(|other_nodes| other_nodes.0.borrow()[other_nodes.1].line == node.line)
//     });
//
//     // TODO Figure out what to do with this
//
//     let ret = search(root, place_before_symbol);
//     if let Some((mut parent, i)) = ret {
//         let children = parent
//             .children
//             .as_mut()
//             .expect("Parent doesn't have children");
//         // TODO This doesn't work, find way to modify ast
//         let mut new = children.borrow().clone().to_vec();
//         new.insert(i, clone_declaration);
//         *children.borrow_mut() = new.into_boxed_slice();
//
//         println!("HERERERERERE {:?}", children.borrow());
//         // TODO Consider when to take a value_instance_node
//         println!("{:?}", value_instance_nodes);
//
//         for (sibiling_nodes, i) in value_instance_nodes.iter() {
//             match &sibiling_nodes.borrow()[*i].token {
//                 NodeType::Id(_id) => {}
//                 _ => panic!("value instance node must be id"),
//             }
//
//             sibiling_nodes.borrow_mut()[*i].token = NodeType::Id(clone_id.clone());
//
//             // NOTE Run the analyzer and checker again with the new variable
//             *ctx = AnalysisContext::new();
//             println!("NEW ITERATION\n\n");
//             analyzer::determine_var_mutability(root, ctx, Rc::new(RefCell::new(Box::new([]))), 0);
//             let new_errors = checker::borrow_check(ctx);
//             adjust_ptr_type(new_errors, ctx, root);
//         }
//     }
// }

pub fn adjust_ptr_type(errors: Vec<BorrowError>, ctx: &mut AnalysisContext, root: &mut Node) {
    errors.iter().for_each(|error| {
        // A lot of work for nothing
        match &error {
            BorrowError::MutMutOverlap {
                first_ptr_id: _,
                second_ptr_id: _,
                value_id,
            } => set_ptr_rc(value_id, ctx),
            BorrowError::MutImutOverlap {
                mut_ptr_id: _,
                imut_ptr_id: _,
                value_id,
            } => set_ptr_rc(value_id, ctx),
            BorrowError::MutMutSameLine {
                first_ptr_id,
                second_ptr_id,
                value_id: _,
            } => {
                set_ptr_rc(first_ptr_id, ctx);
                // set_raw(second_ptr_id, ctx)
            }

            BorrowError::MutImutSameLine {
                mut_ptr_id,
                imut_ptr_id,
                value_id: _,
            } => {
                // set_raw(mut_ptr_id, ctx);
                // set_raw(imut_ptr_id, ctx);
            }
            // TODO: if the id is the value, we can clone
            BorrowError::ValueMutOverlap {
                ptr_id: _,
                value_id,
            } => {
                set_ptr_rc(value_id, ctx)
                // clone_solution(ptr_id, value_id, ctx, root)
            }
            BorrowError::ValueMutSameLine {
                ptr_id,
                value_id,
                // value_instance_nodes,
            } => {
                // create_clone(value_id, ptr_id, ctx, root, value_instance_nodes.clone());
            }
        };

        // TODO: This should actually traverse the pointer chain downwards
    });
}

#[derive(Debug, Clone)]
struct PtrData<'a> {
    ptr_id: String,
    ptr_var_data: &'a VarData,
    ptr_type: ReferenceType,
}

// TODO: Figure out how to include line numbers in error reports
pub fn borrow_check<'a>(ctx: &'a AnalysisContext) -> Vec<BorrowError> {
    // ctx.print_refs();
    ctx.variables
        .iter()
        .flat_map(|(var_id, var_data)| -> Vec<BorrowError> {
            let pointed_to_by: Vec<Reference> = var_data
                .pointed_to
                .iter()
                .map(|reference_block| {
                    reference_block.borrow().clone()
                })
                .collect();
            println!("\n{var_id} pointed to by: {:?}\n", pointed_to_by);

            let pointed_to_by_mutably = pointed_to_by
                .iter()
                .filter(|ptr_info| ptr_info.get_reference_type() == ReferenceType::MutBorrowed);

            let mut value_overlaps_with_mut_ptr: Vec<BorrowError> = pointed_to_by_mutably
                .clone()
                .filter_map(|reference_block| {
                    println!("{var_id} usages: {:?}\n", var_data.usages.clone());
                    let overlap_state = var_ptr_range_overlap(
                        var_data.usages.clone(),
                        reference_block.get_range()
                    );

                    let borrower_id = reference_block.get_borrower();

                    match overlap_state {
                        OverlapState::Overlap => Some(BorrowError::ValueMutOverlap {
                            ptr_id: borrower_id.to_string(),
                            value_id: var_id.clone(),
                        }),
                        OverlapState::SameLine => Some(BorrowError::ValueMutSameLine { ptr_id: borrower_id.to_string(), value_id: var_id.clone()}),
                        _ => None,
                    }
                })
                .collect();

                let mut mutable_ref_overlaps_with_ptr: Vec<BorrowError> = pointed_to_by_mutably.flat_map(|mut_ref| {
                pointed_to_by
                    .iter()
                    .filter(|other_ref| mut_ref.get_borrower() != other_ref.get_borrower())
                    .filter_map(|other_ref| {
                        let overlap_state = ptr_range_overlap(
                            mut_ref.get_range(),
                            other_ref.get_range()
                        );

                        let other_id = other_ref.get_borrower();
                        let mut_id = mut_ref.get_borrower();

                        match (other_ref.get_reference_type().clone(), overlap_state) {
                            // NOTE In these cases, an Rc<RefCell> solution works, since they overlap and borrows can be
                            // made on different lines and both dropped after one line
                            (ReferenceType::MutBorrowed, OverlapState::Overlap) => {
                                Some(BorrowError::MutMutOverlap {
                                    first_ptr_id: mut_id.to_string(),
                                    second_ptr_id: other_id.to_string(),
                                    value_id: var_id.clone(),
                                })
                            }
                            (ReferenceType::ConstBorrowed, OverlapState::Overlap) => {
                                Some(BorrowError::MutImutOverlap {
                                    mut_ptr_id: mut_id.to_string(),
                                    imut_ptr_id: other_id.to_string(),
                                    value_id: var_id.clone(),
                                })
                            }
                            // NOTE The solution won't work in these case, since the borrow 
                            // will be made on the same line,violating borrow checking
                            // rules at runtime. Doing so causes the Rc to panic
                            (ReferenceType::MutBorrowed, OverlapState::SameLine) => {
                                Some(BorrowError::MutMutSameLine {
                                    first_ptr_id: mut_id.to_string(),
                                    second_ptr_id: other_id.to_string(),
                                    value_id: var_id.clone(),
                                })
                            }
                            (ReferenceType::ConstBorrowed, OverlapState::SameLine) => panic!("ImutRef on same line, this is fine\n This actually might be a problem if we have a mutable and immutable reference overlapping on the same line"),
                            (_, OverlapState::NoOverlap) => None,
                            (_, _) => panic!("Basic ref should not have smart ptr type"),
                        }
                    }).collect::<Vec<BorrowError>>()
            }).collect();

            println!(
                "value_overlaps_with_mut_ptr {var_id}: {:?}\nmutable_ref_overlaps {var_id}: {:?}",
                value_overlaps_with_mut_ptr, mutable_ref_overlaps_with_ptr
            );
            value_overlaps_with_mut_ptr.append(&mut mutable_ref_overlaps_with_ptr);
            value_overlaps_with_mut_ptr
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub enum OverlapState {
    Overlap,
    SameLine,
    NoOverlap,
}

// TODO: Create more elegant solution than seperate functions for simply changing the exclusively
// of an inequality
//
// Returns the function
pub fn ptr_range_overlap(l_1: Range<LineNumber>, l_2: Range<LineNumber>) -> OverlapState {
    if l_1.start < l_2.end && l_1.end > l_2.start {
        OverlapState::Overlap
    } else if l_1.start == l_2.end
        || l_1.end == l_2.start
        || l_1.end == l_2.end
        || l_1.start == l_2.start
    {
        OverlapState::SameLine
    } else {
        OverlapState::NoOverlap
    }
}

pub fn var_ptr_range_overlap(
    value_usages: Vec<Usage>,
    ptr_range: Range<LineNumber>,
) -> OverlapState {
    let value_lines = value_usages.iter().map(|usage| usage.get_line_number());
    // NOTE `ptr.start == value` is fine because that's what happends when we init a reference
    let usage_in_block = |usage: LineNumber, ptr: &Range<LineNumber>| -> OverlapState {
        if usage < ptr.end && ptr.start < usage {
            OverlapState::Overlap
        } else if usage == ptr.end {
            OverlapState::SameLine
        } else {
            OverlapState::NoOverlap
        }
    };

    let overlaps: Vec<OverlapState> = value_lines
        .map(|value_line| usage_in_block(value_line, &ptr_range))
        .collect();

    if overlaps.contains(&OverlapState::SameLine) {
        OverlapState::SameLine
    } else if overlaps.contains(&OverlapState::Overlap) {
        OverlapState::Overlap
    } else {
        OverlapState::NoOverlap
    }
}
