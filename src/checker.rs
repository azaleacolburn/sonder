use crate::analyzer::{AnalysisContext, PtrType, VarData};
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
    }
}

fn set_ptr_rc(value_id: &str, ctx: &mut AnalysisContext) {
    let sub_var_data = ctx.get_var(&value_id);
    let ptrs = sub_var_data.pointed_to_by.clone();

    ptrs.iter().for_each(|ptr_id| {
        ctx.mut_var(ptr_id.clone(), |ptr_data| {
            // TODO: Grab what the type was of the dpecific ref that pointed to the ref that we
            // care about
            let mut ptr_to_value = ptr_data
                .addresses
                .iter()
                .find(|other_ptr_data| other_ptr_data.borrow().adr_of == *value_id)
                .expect("value and ptr ctx disagree")
                .borrow_mut();
            let other_ref_level_len = ptr_to_value.ptr_type.len() - 1;

            // TODO: We want the specific reference that applies at the same level of the
            // problematic ptr
            // This is a really hard problem
            // For now we'll just make the top ptr_type be an RcClone
            ptr_to_value.ptr_type[other_ref_level_len] = PtrType::RcRefClone;
        });
        set_ptr_rc(&ptr_id, ctx);
    });
}

fn set_rc(value_id: &str, ctx: &mut AnalysisContext) {
    ctx.mut_var(value_id.to_string(), |var_data| var_data.rc = true);
    set_ptr_rc(value_id, ctx);
}

fn set_raw(ptr_id: &str, ctx: &mut AnalysisContext) {}

// fn clone_solution<'a>(
//     mut_ptr_id: &str,
//     value_id: &str,
//     ctx: &mut AnalysisContext<'a>,
//     root: &mut Node,
// ) {
//     let clone_id = format!("{value_id}_clone");
//     let clone_declaration_node = Node::new(NodeType::Declaration(clone_id, CType::Int, 0), vec![]);
//     // TODO: Figure out how to annotated cloning
//     // let value_data = ctx.get_var(value_id).expect("value id not in map");
//     // let cloned_value_id = format!("{}_clone", value_id);
//     // let cloned_value_data = VarData {
//     //     addresses: value_data.addresses.clone(),
//     //     pointed_to_by: value_data.pointed_to_by.clone(),
//     //     is_mut_direct: false,
//     //     is_mut_by_ptr: false,
//     //     non_borrowed_lines: vec![],
//     //     rc: false,
//     //     set_start_borrow: false,
//     // };
//     // ctx.new_var(cloned_value_id, cloned_value_data);
// }

/// This function is problematic because it requires the ast to be changed :(
/// Either that, or we could use some othe protocole for conveying a new variable
/// Or, we could not add a new variable because we're weak and don't want to change business logic
fn create_clone(_value_id: &str, _ptr_id: &str, _ctx: &mut AnalysisContext) {}

pub fn adjust_ptr_type(errors: Vec<BorrowError>, ctx: &mut AnalysisContext) {
    errors.iter().for_each(|error| {
        // A lot of work for nothing
        match &error {
            BorrowError::MutMutOverlap {
                first_ptr_id: _,
                second_ptr_id: _,
                value_id,
            } => set_rc(value_id, ctx),
            BorrowError::MutImutOverlap {
                mut_ptr_id: _,
                imut_ptr_id: _,
                value_id,
            } => set_rc(value_id, ctx),
            BorrowError::MutMutSameLine { first_ptr_id, second_ptr_id, value_id: _ } => 
            {set_raw(first_ptr_id, ctx); set_raw(second_ptr_id, ctx)},
            
            BorrowError::MutImutSameLine { mut_ptr_id, imut_ptr_id, value_id: _ } => {
                set_raw(mut_ptr_id, ctx);
                set_raw(imut_ptr_id, ctx);
            }
            // TODO: if the id is the value, we can clone
            BorrowError::ValueMutOverlap {
                ptr_id: _,
                value_id,
            } => {
                set_rc(value_id, ctx)
                // clone_solution(ptr_id, value_id, ctx, root)
            }
            BorrowError::ValueMutSameLine { ptr_id, value_id } => {
                create_clone(value_id, ptr_id, ctx);
                // set_raw(value_id, ptr_id);
            }
        };

        // TODO: This should actually traverse the pointer chain downwards
    });
}

#[derive(Debug, Clone)]
struct PtrInfo<'a> {
    ptr_id: String,
    ptr_var_data: &'a VarData,
    ptr_type: PtrType,
}

// TODO: Figure out how to include line numbers in error reports
pub fn borrow_check<'a>(ctx: &'a AnalysisContext) -> Vec<BorrowError> {
    ctx.print_refs();
    ctx.variables
        .iter()
        .flat_map(|(var_id, var_data)| -> Vec<BorrowError> {
            let pointed_to_by: Vec<PtrInfo> = var_data
                .pointed_to_by
                .iter()
                .map(|ptr_id| {
                    let ptr_var_data = ctx.get_var(ptr_id);
                    let adr_data = ptr_var_data
                        .addresses
                        .iter()
                        .find(|ptr_data| ptr_data.borrow().adr_of == *var_id)
                        .expect("Ptr does not record reference to var in map");
                    PtrInfo {
                        ptr_id: ptr_id.to_string(),
                        ptr_var_data,
                        // TODO: Check if this is a fine solution, if the top is mutable than all
                        // the rest should be too
                        // But is this ever what we want
                        ptr_type: adr_data.borrow().ptr_type[0].clone(),
                    }
                })
                .collect();
            println!("{var_id} is pointed to by: {:?}", pointed_to_by);
            let pointed_to_by_mutably = pointed_to_by
                .iter()
                .filter(|ptr_info| ptr_info.ptr_type == PtrType::MutRef);
            let mut value_overlaps_with_mut_ptr: Vec<BorrowError> = pointed_to_by_mutably
                .clone()
                .filter_map(|mut_ptr_info| {
                    // NOTE This fails because the value and the pointer are going to overlap on the line
                    // the ref to the pointer is taken
                    // eg. `let m = &mut n`
                    let overlap_state = var_active_range_overlap(
                        mut_ptr_info.ptr_var_data.non_borrowed_lines.clone(),
                        var_data.non_borrowed_lines.clone(),
                    );

                    match overlap_state {
                        OverlapState::Overlap => 
                            Some(BorrowError::ValueMutOverlap {
                            ptr_id: mut_ptr_info.ptr_id.clone(),
                            value_id: var_id.clone(),
                        }
                        ),
                        OverlapState::SameLine => Some(BorrowError::ValueMutSameLine { ptr_id: mut_ptr_info.ptr_id.clone(), value_id: var_id.clone() }),
                    _ => None,
                    }
                })
                .collect();
                let mut mutable_ref_overlaps_with_ptr: Vec<BorrowError> = pointed_to_by_mutably.flat_map(|mut_ptr_data| {
                pointed_to_by
                    .iter()
                    .filter(|other_ptr_data| mut_ptr_data.ptr_id != other_ptr_data.ptr_id)
                    .filter_map(|other_ptr_data| {
                        let overlap_state = both_ptr_active_range_overlap(
                            mut_ptr_data.ptr_var_data.non_borrowed_lines.clone(),
                            other_ptr_data.ptr_var_data.non_borrowed_lines.clone(),
                        );
                        // NOTE All of these represent the ptr type and the overlap state
                        // between some ptr (other_ptr) and mutable reference (mut_ptr)
                        // to the same symbol
                        match (other_ptr_data.ptr_type.clone(), overlap_state) {
                            // NOTE In these cases, an Rc<RefCell> solution works, since they overlap and borrows can be
                            // made on different lines and both dropped after one line
                            (PtrType::MutRef, OverlapState::Overlap) => {
                                Some(BorrowError::MutMutOverlap {
                                    first_ptr_id: mut_ptr_data.ptr_id.clone(),
                                    second_ptr_id: other_ptr_data.ptr_id.clone(),
                                    value_id: var_id.clone(),
                                })
                            }
                            (PtrType::ImutRef, OverlapState::Overlap) => {
                                Some(BorrowError::MutImutOverlap {
                                    mut_ptr_id: mut_ptr_data.ptr_id.clone(),
                                    imut_ptr_id: other_ptr_data.ptr_id.clone(),
                                    value_id: var_id.clone(),
                                })
                            }
                            // NOTE The solution won't work in these case, since the borrow 
                            // will be made on the same line,violating borrow checking
                            // rules at runtime. Doing so causes the Rc to panic
                            (PtrType::MutRef, OverlapState::SameLine) => {
                                Some(BorrowError::MutMutSameLine {
                                    first_ptr_id: mut_ptr_data.ptr_id.clone(),
                                    second_ptr_id: other_ptr_data.ptr_id.clone(),
                                    value_id: var_id.clone(),
                                })
                            }
                            (PtrType::ImutRef, OverlapState::SameLine) => {
                                panic!("ImutRef on same line, this is fine\n This actually might be a problem if we have a mutable and immutable reference overlapping on the same line");
                            }
                            (_, OverlapState::NoOverlap) => {
                                None
                            }
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
pub fn both_ptr_active_range_overlap(
    l_1: Vec<Range<usize>>,
    l_2: Vec<Range<usize>>,
) -> OverlapState {
    let ranges_overlap = |l_1: &Range<usize>, l_2: &Range<usize>| -> OverlapState {
        if l_1.start < l_2.end && l_2.start < l_1.end {
            OverlapState::Overlap
        } else if l_1.start == l_2.end || l_2.start == l_1.end {
            OverlapState::SameLine
        } else {
            OverlapState::NoOverlap
        }
    };

    let overlaps_list: Vec<OverlapState> = l_1
        .iter()
        .flat_map(|l_1| l_2.iter().map(|l_2| ranges_overlap(l_1, l_2)))
        .collect();
    if overlaps_list.contains(&OverlapState::SameLine) {
        OverlapState::SameLine
    } else if overlaps_list.contains(&OverlapState::Overlap) {
        OverlapState::Overlap
    } else {
        OverlapState::NoOverlap
    }
}

pub fn var_active_range_overlap(value: Vec<Range<usize>>, ptr: Vec<Range<usize>>) -> OverlapState {
    let ranges_overlap = |value: &Range<usize>, ptr: &Range<usize>| -> OverlapState {
        if value.start < ptr.end && ptr.start < value.end {
            OverlapState::Overlap
            // `ptr.start == value.end` is fine because that's what happends when we init a reference
        } else if value.start == ptr.end {
            OverlapState::SameLine
        } else {
            OverlapState::NoOverlap
        }
    };

    let overlaps_list: Vec<OverlapState> = value
        .iter()
        .flat_map(|value| ptr.iter().map(|ptr| ranges_overlap(value, ptr)))
        .collect();
    if overlaps_list.contains(&OverlapState::SameLine) {
        OverlapState::SameLine
    } else if overlaps_list.contains(&OverlapState::Overlap) {
        OverlapState::Overlap
    } else {
        OverlapState::NoOverlap
    }
}
