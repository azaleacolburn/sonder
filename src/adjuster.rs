use crate::{
    analysis_ctx::AnalysisContext,
    ast::TokenNode as Node,
    checker::BorrowError,
    data_model::{LineNumber, ReferenceType, UsageType},
};

impl AnalysisContext {
    fn set_ptr_rc(&mut self, value_id: &str) {
        let var_data = self.get_var_mut(value_id);
        var_data.rc = true;

        // TODO distinguish between `ptr = &m` and `let another = &mut ptr`
        // Essentially make `is_mut_ptr` and `is_mut_direct` distinct
        var_data.is_mut = false;
        let ptrs = var_data.pointed_to.clone();

        // TODO Figure out what to do about this ugly code
        // - Putting everything in set_rc doesn't make sense, despite being cleaner
        ptrs.iter().for_each(|reference_block| {
            reference_block.borrow_mut().set_rc();

            let b = reference_block.borrow();
            let ptr_id = b.get_borrower();
            self.mut_var(ptr_id.to_string(), |ptr_data| {
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

    fn set_ptr_raw(&mut self, ptr_id: &str) {
        // NOTE If we're here, we need raw ptrs because of overlapping rather
        // than arithmatic, so we just need to use the unsafe system, not create
        // a new system for arithmetic translation
        //
        // TODO Cascade raw pointers to variables that the original ptr
        // is an rside value of
        let ptr_data = self.get_var_mut(ptr_id);
        ptr_data.set_raw();

        ptr_data
            .usages
            .iter()
            .filter(|usage| *usage.get_usage_type() == UsageType::RValue)
            .for_each(|_usage| {
                // TODO Adjust to use special indexing rules (and other stuff) for raw pointers
                // This is actually a fairly big task, as requires annoating specific id notes with
                // usage data. Which is definitely something we need to do, it just requires some
                // refactoring
            });
    }

    pub fn adjust_ptr_type(&mut self, errors: Vec<BorrowError>, root: &mut Node) {
        // TODO Figure out correct error hierarchy
        // errors.sort();
        // TODO Implement temporary cloned variable solution for empty cases
        errors.iter().for_each(|error| {
            match &error {
                BorrowError::MutMutOverlap {
                    first_ptr_id: _,
                    second_ptr_id: _,
                    value_id,
                } => self.set_ptr_rc(value_id),
                BorrowError::MutConstOverlap {
                    mut_ptr_id,
                    const_ptr_id,
                    value_id,
                } if !line_rearrangement_mut_const_overlap(
                    mut_ptr_id,
                    const_ptr_id,
                    value_id,
                    root,
                    self,
                ) =>
                {
                    self.set_ptr_rc(value_id);
                }
                BorrowError::MutConstOverlap {
                    mut_ptr_id: _,
                    const_ptr_id: _,
                    value_id: _,
                } => {}
                BorrowError::MutMutSameLine {
                    first_ptr_id,
                    second_ptr_id,
                    value_id: _,
                } => {
                    self.set_ptr_raw(second_ptr_id);
                    self.set_ptr_raw(first_ptr_id);
                }

                BorrowError::MutConstSameLine {
                    mut_ptr_id,
                    const_ptr_id,
                    value_id: _,
                } => {
                    self.set_ptr_raw(mut_ptr_id);
                    self.set_ptr_raw(const_ptr_id);
                }
                // TODO: if the id is the value, we can clone
                BorrowError::ValueMutOverlap { ptr_id, value_id }
                    if !line_rearrangement_value_ptr_overlap(
                        value_id, ptr_id, root, self, false,
                    ) =>
                {
                    self.set_ptr_rc(value_id);
                }
                BorrowError::ValueMutOverlap {
                    ptr_id: _,
                    value_id: _,
                } => {}
                BorrowError::ValueMutSameLine {
                    ptr_id,
                    value_id: _,
                } => {
                    self.set_ptr_raw(ptr_id);
                }
                BorrowError::ValueConstOverlap { ptr_id, value_id }
                    if !line_rearrangement_value_ptr_overlap(
                        value_id, ptr_id, root, self, true,
                    ) =>
                {
                    self.set_ptr_rc(value_id)
                }
                BorrowError::ValueConstOverlap {
                    ptr_id: _,
                    value_id: _,
                } => {}
                BorrowError::ValueConstSameLine {
                    ptr_id: _,
                    value_id: _,
                } => {
                    // NOTE This must be rside, so it's fine i think (not actually an error)
                    // TODO Remove this error type and make permanent note
                    // self.set_ptr_raw(ptr_id);
                }
            };

            // TODO: This should actually traverse the pointer chain downwards
        });
    }
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

    if const_range.start == const_range.end {
        rearrange_lines_tree(mut_range.start, const_range.start, root);
        return true;
    } else if mut_range.start == mut_range.end {
        rearrange_lines_tree(const_range.start, mut_range.start, root);
        return true;
    }

    match const_range.start > mut_range.start {
        true => {
            let first_const_usage_in_reference = const_ptr_usages
                .find(|const_usage| {
                    const_reference
                        .borrow()
                        .contained_within_current_range(const_usage.get_line_number())
                })
                .unwrap();

            let last_mut_usage_in_reference = mut_ptr_usages
                .filter(|mut_usage| {
                    const_reference
                        .borrow()
                        .contained_within_current_range(mut_usage.get_line_number())
                })
                .last()
                .unwrap();

            if first_const_usage_in_reference.get_line_number()
                > last_mut_usage_in_reference.get_line_number()
            {
                // TODO Iteratively move all overlapped lines
                rearrange_lines_tree(mut_range.start, const_range.end, root);
                true
            } else {
                false
            }
        }
        false => {
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

            if first_mut_usage_in_reference.get_line_number()
                > last_const_usage_in_reference.get_line_number()
            {
                // TODO Iteratively move all overlapped lines
                rearrange_lines_tree(mut_range.start, const_range.end, root);
                true
            } else {
                false
            }
        }
    }
}

/// Checks if a simple rearrangement of lines could fix = the borrow error
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

    // NOTE Each of these are mutations of var or a usage by a mutable reference
    let var_mut_usages = var_data
        .usages
        .iter()
        .filter(|usage| *usage.get_usage_type() == UsageType::LValue || !const_ptr);

    let last_var_usage = var_mut_usages.clone().last().unwrap();
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

    if last_var_usage.get_line_number() < first_ptr_usage.get_line_number() {
        var_mut_usages.for_each(|usage| {
            rearrange_lines_tree(
                reference.borrow().get_range().start,
                usage.get_line_number(),
                root,
            )
        });

        return true;
    }

    false
}

/// This function assumes that `rearrange_lines_tree` has already been called
fn rearrange_lines_ctx(pivot: LineNumber, swing: LineNumber, ctx: &mut AnalysisContext) {
    ctx.current_scope_mut()
        .variables
        .iter_mut()
        .for_each(|(_var_id, var_data)| {
            var_data
                .usages
                .iter_mut()
                .filter(|usage| usage.get_line_number() >= pivot && usage.get_line_number() < swing)
                .for_each(|usage| usage.set_line_number(usage.get_line_number().clone() + 1));

            var_data
                .usages
                .iter_mut()
                .filter(|usage| usage.get_line_number() == swing)
                .for_each(|usage| usage.set_line_number(pivot));
        })
}

// TODO Call analyzer again or manually go through and change ctx line numbers for both
// usages and nodes
// Wait, do we even need to do that? Will we ever used LineNumbers again?
fn rearrange_lines_tree(first_line: LineNumber, second_line: LineNumber, root: &mut Node) {
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

                // NOTE Returns from function
                return;
            }
        }

        // NOTE This makes it a breadth first search sorta
        for child in children.iter_mut() {
            rearrange_lines_tree(first_line, second_line, child);
        }
    }
}

fn move_node_before(children: &mut Box<[Node]>, from_index: usize, to_index: usize) {
    if from_index == to_index || from_index >= children.len() || to_index >= children.len() {
        return; // NOTE No need to move if indices are the same or out of bounds.
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
