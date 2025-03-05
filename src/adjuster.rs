use crate::{
    analysis_ctx::AnalysisContext,
    ast::TokenNode as Node,
    checker::BorrowError,
    data_model::{LineNumber, ReferenceType, Usage, UsageType},
};

pub fn adjust_ptr_type(errors: Vec<BorrowError>, ctx: &mut AnalysisContext, root: &mut Node) {
    errors.iter().for_each(|error| {
        match &error {
            BorrowError::MutMutOverlap {
                first_ptr_id: _,
                second_ptr_id: _,
                value_id,
            } => set_ptr_rc(value_id, ctx),
            BorrowError::MutConstOverlap {
                mut_ptr_id,
                imut_ptr_id,
                value_id,
            } => {
                if !line_rearrangement_mut_const_overlap(
                    mut_ptr_id,
                    imut_ptr_id,
                    value_id,
                    root,
                    ctx,
                ) {
                    set_ptr_rc(value_id, ctx);
                }
            }
            BorrowError::MutMutSameLine {
                first_ptr_id,
                second_ptr_id,
                value_id: _,
            } => {
                set_ptr_rc(first_ptr_id, ctx);
                // set_raw(second_ptr_id, ctx)
            }

            BorrowError::MutConstSameLine {
                mut_ptr_id,
                imut_ptr_id,
                value_id: _,
            } => {
                // set_raw(mut_ptr_id, ctx);
                // set_raw(imut_ptr_id, ctx);
            }
            // TODO: if the id is the value, we can clone
            BorrowError::ValueMutOverlap { ptr_id, value_id } => {
                if !line_rearrangement_value_ptr_overlap(value_id, ptr_id, root, ctx, false) {
                    set_ptr_rc(value_id, ctx);
                }
                // clone_solution(ptr_id, value_id, ctx, root)
            }
            BorrowError::ValueMutSameLine {
                ptr_id,
                value_id,
                // value_instance_nodes,
            } => {
                // create_clone(value_id, ptr_id, ctx, root, value_instance_nodes.clone());
            }
            BorrowError::ValueConstOverlap { ptr_id, value_id } => {
                if !line_rearrangement_value_ptr_overlap(value_id, ptr_id, root, ctx, true) {
                    set_ptr_rc(value_id, ctx);
                }
                // clone_solution(ptr_id, value_id, ctx, root)
            }
            BorrowError::ValueConstSameLine {
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

fn line_rearrangement_mut_const_overlap(
    mut_ptr_id: &str,
    const_ptr_id: &str,
    value_id: &str,
    root: &mut Node,
    ctx: &mut AnalysisContext,
) -> bool {
    let mut_ptr = ctx.get_var(mut_ptr_id);
    let const_ptr = ctx.get_var(const_ptr_id);

    let mut_reference = mut_ptr.reference_to_var(value_id).unwrap().clone();
    let const_reference = const_ptr.reference_to_var(value_id).unwrap().clone();

    let mut mut_ptr_usages = mut_ptr.usages.iter();
    let mut const_ptr_usages = const_ptr.usages.iter();

    let const_range = const_reference.borrow().get_range();
    let mut_range = mut_reference.borrow().get_range();

    match const_range.start > mut_range.start {
        true => {
            let first_mut_usage_in_reference = mut_ptr_usages
                .find(|mut_usage| {
                    const_reference
                        .borrow()
                        .contained_within_current_range(mut_usage.get_line_number())
                })
                .unwrap();

            let last_const_usage_in_reference = const_ptr_usages
                .filter(|const_usage| {
                    mut_reference
                        .borrow()
                        .contained_within_current_range(const_usage.get_line_number())
                })
                .last()
                .unwrap();

            // if first_mut_usage_in_reference.get_line_number()
            //     > last_const_usage_in_reference.get_line_number()
            // {
            //     // TODO Iteratively move all overlapped lines
            //     rearrange_lines(mut_range.start, const_range.end, root);
            //     true
            // } else {
            //     false
            // }
            false
        }
        false => {
            // TODO Maybe make it a union reference instead
            let last_const_usage_in_reference = const_ptr_usages
                .filter(|const_usage| {
                    mut_reference
                        .borrow()
                        .contained_within_current_range(const_usage.get_line_number())
                })
                .last()
                .unwrap();

            let first_mut_usage_in_reference = mut_ptr_usages
                .find(|mut_usage| {
                    mut_reference
                        .borrow()
                        .contained_within_current_range(mut_usage.get_line_number())
                })
                .unwrap();

            if last_const_usage_in_reference.get_line_number()
                < first_mut_usage_in_reference.get_line_number()
            {
                // TODO Iteratively move all overlapped lines
                rearrange_lines(mut_range.start, const_range.end, root);
                true
            } else {
                false
            }
        }
    }
}

/// Checks if a simple rearrangement of lines could fix = the borrow error
///
///
/// # Important
/// A value can be used behind a immutable reference if it's an rvalue that implements copy, which
/// every non-struct, non-ptr rust analog to a c variable does
fn line_rearrangement_value_ptr_overlap(
    value_id: &str,
    ptr_id: &str,
    root: &mut Node,
    ctx: &mut AnalysisContext,
    const_ptr: bool,
) -> bool {
    let var_data = ctx.get_var(value_id);
    let ptr_data = ctx.get_var(ptr_id);
    let reference = ptr_data.reference_to_var(value_id).unwrap().clone();

    // This means it's modified
    let var_mut_usages = var_data
        .usages
        .iter()
        .filter(|usage| *usage.get_usage_type() == UsageType::LValue || !const_ptr);

    let last_var_usage = var_mut_usages.last().unwrap();
    let first_ptr_usage = ptr_data
        .usages
        .iter()
        .find(|usage| {
            reference
                .borrow()
                .contained_within_current_range(usage.get_line_number())
        })
        .expect("Ptr never used within reference (meaning it lasts a single)")
        .clone();

    match last_var_usage.get_line_number() < first_ptr_usage.get_line_number() {
        true => {
            rearrange_lines(
                reference.borrow().get_range().start,
                last_var_usage.get_line_number(),
                root,
            );
            true
        }
        false => false,
    }
}

// TODO Call analyzer again or manually go through and change ctx line numbers for both
// usages and nodes
//
// Wait, do we even need to do that? Will we ever used LineNumbers again?
fn rearrange_lines(first_line: LineNumber, second_line: LineNumber, root: &mut Node) {
    let mut first = false;
    let mut second = false;
    if let Some(children) = root.children.as_mut() {
        for child in children.iter_mut() {
            if child.line == first_line {
                first = true;
            } else if child.line == second_line {
                second = true;
            }

            if first && second {
                let first_node_index = children
                    .iter()
                    .position(|child| child.line == first_line)
                    .unwrap();
                let second_node_index = children
                    .iter()
                    .position(|child| child.line == second_line)
                    .unwrap();

                move_node_before(children, second_node_index, first_node_index);
                return;
            }
        }

        // NOTE This makes it a breadth first search sorta
        for child in children.iter_mut() {
            rearrange_lines(first_line, second_line, child);
        }
    }
}

fn move_node_before(children: &mut Box<[Node]>, from_index: usize, to_index: usize) {
    if from_index == to_index || from_index >= children.len() || to_index >= children.len() {
        return; // No need to move if indices are the same or out of bounds.
    }

    let mut vec = children.to_vec();
    let node = vec.remove(from_index);

    let insert_index = if from_index < to_index {
        to_index - 1
    } else {
        to_index
    };

    vec.insert(insert_index, node);

    *children = vec.into_boxed_slice();
}

fn set_ptr_rc(value_id: &str, ctx: &mut AnalysisContext) {
    let var_data = ctx.get_var_mut(value_id);
    var_data.rc = true;

    // TODO distinguish between `ptr = &m` and `let another = &mut ptr`
    // Essentially bring back `is_mut_ptr` and `is_mut_direct`
    var_data.is_mut = false;

    let ptrs = var_data.pointed_to.clone();

    ptrs.iter().for_each(|reference_block| {
        reference_block.borrow_mut().set_rc();

        let b = reference_block.borrow();
        let ptr_id = b.get_borrower();
        ctx.mut_var(ptr_id.to_string(), |ptr_data| {
            let has_higher_mut_borrower = ptr_data
                .pointed_to
                .iter()
                .any(|r| r.borrow().get_reference_type() == ReferenceType::MutBorrowed);

            if !has_higher_mut_borrower {
                ptr_data.is_mut = false;
            }
        })
    });
}
