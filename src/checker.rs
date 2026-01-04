use crate::{
    analysis_ctx::AnalysisContext,
    data_model::{LineNumber, Reference, ReferenceType, Usage, UsageType, VarData},
};
use std::ops::Range;

// TODO: Figure out how to include line numbers in error reports
pub fn borrow_check(ctx: &mut AnalysisContext) -> Vec<BorrowError> {
    // ctx.print_refs();
    ctx.current_scope_mut().variables
        .iter_mut()
        .flat_map(|(var_id, var_data)| -> Vec<BorrowError> {
            let pointed_to_by: Vec<Reference> = var_data
                .pointed_to
                .iter()
                .map(|reference_block| {
                    reference_block.borrow().clone()
                })
                .collect();

            let pointed_to_by_mutably  = pointed_to_by
                .iter()
                .filter(|ptr_info| ptr_info.get_reference_type() == ReferenceType::MutBorrowed);

            let rvalue_usages: Vec<Usage> = var_data.usages.clone().into_iter().filter(|usage| *usage.get_usage_type() == UsageType::RValue).collect();
            let lvalue_usages: Vec<Usage> = var_data.usages.clone().into_iter().filter(|usage| *usage.get_usage_type() == UsageType::LValue).collect();
            check_unused_init_value(var_data, &rvalue_usages, &lvalue_usages);


            let mut value_overlaps_with_mut_ptr: Vec<BorrowError> = check_value_overlaps_with_mut_ptr(var_id, var_data, pointed_to_by_mutably.clone());
            let mut value_overlaps_with_const_ptr: Vec<BorrowError> = check_value_overlaps_with_const_ptr(var_id, lvalue_usages, pointed_to_by.iter());
            let mut mutable_ref_overlaps_with_ptr: Vec<BorrowError> = check_mutable_ref_overlaps_with_ptr(var_id, pointed_to_by_mutably, pointed_to_by.iter());

            println!(
                "value_overlaps_with_mut_ptr {var_id}: {:?}\nvalue_overlaps_with_const_ptr: {:?}\nmutable_ref_overlaps {var_id}: {:?}",
                value_overlaps_with_mut_ptr, value_overlaps_with_const_ptr, mutable_ref_overlaps_with_ptr
            );
            value_overlaps_with_mut_ptr.append(&mut value_overlaps_with_const_ptr);
            value_overlaps_with_mut_ptr.append(&mut mutable_ref_overlaps_with_ptr);
            value_overlaps_with_mut_ptr
        })
        .collect()
}

fn check_value_overlaps_with_mut_ptr<'a, T>(
    var_id: &str,
    var_data: &VarData,
    pointed_to_by_mutably: T,
) -> Vec<BorrowError>
where
    T: Iterator<Item = &'a Reference>,
{
    pointed_to_by_mutably
        .filter_map(|reference_block| {
            let overlap_state =
                var_ptr_range_overlap(var_data.usages.clone(), reference_block.get_range());
            let borrower_id = reference_block.get_borrower();

            match overlap_state {
                OverlapState::Overlap => Some(BorrowError::ValueMutOverlap {
                    ptr_id: borrower_id.to_string(),
                    value_id: var_id.to_string(),
                }),
                OverlapState::SameLine => Some(BorrowError::ValueMutSameLine {
                    ptr_id: borrower_id.to_string(),
                    value_id: var_id.to_string(),
                }),
                _ => None,
            }
        })
        .collect()
}

fn check_value_overlaps_with_const_ptr<'a, T>(
    var_id: &str,
    lvalue_usages: Vec<Usage>,
    pointed_to_by: T,
) -> Vec<BorrowError>
where
    T: Iterator<Item = &'a Reference>,
{
    pointed_to_by
        .filter_map(|reference_block| {
            let overlap_state =
                var_ptr_range_overlap(lvalue_usages.clone(), reference_block.get_range());

            let borrower_id = reference_block.get_borrower();

            match overlap_state {
                OverlapState::Overlap => Some(BorrowError::ValueConstOverlap {
                    ptr_id: borrower_id.to_string(),
                    value_id: var_id.to_string(),
                }),
                OverlapState::SameLine => Some(BorrowError::ValueConstSameLine {
                    ptr_id: borrower_id.to_string(),
                    value_id: var_id.to_string(),
                }),
                _ => None,
            }
        })
        .collect()
}

fn check_mutable_ref_overlaps_with_ptr<'a, T, I>(
    var_id: &str,
    pointed_to_by_mutably: T,
    pointed_to_by: I,
) -> Vec<BorrowError>
where
    T: Iterator<Item = &'a Reference>,
    I: Iterator<Item = &'a Reference> + Clone,
{
    pointed_to_by_mutably.flat_map(|mut_ref| {
        pointed_to_by.clone()
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
                        value_id: var_id.to_string(),
                    })
                }
                (ReferenceType::ConstBorrowed, OverlapState::Overlap) => {
                    Some(BorrowError::MutConstOverlap {
                        mut_ptr_id: mut_id.to_string(),
                        const_ptr_id: other_id.to_string(),
                        value_id: var_id.to_string(),
                    })
                }
                // NOTE The solution won't work in these case, since the borrow
                // will be made on the same line,violating borrow checking
                // rules at runtime. Doing so causes the Rc to panic
                (ReferenceType::MutBorrowed, OverlapState::SameLine) => {
                    Some(BorrowError::MutMutSameLine {
                        first_ptr_id: mut_id.to_string(),
                        second_ptr_id: other_id.to_string(),
                        value_id: var_id.to_string(),
                    })
                }
                (ReferenceType::ConstBorrowed, OverlapState::SameLine) => panic!("ConstRef on same line, this is fine\n This actually might be a problem if we have a mutable and immutable reference overlapping on the same line"),
                (_, OverlapState::NoOverlap) => None,
                (_, _) => panic!("Basic ref should not have smart ptr type"),
            }
        }).collect::<Vec<BorrowError>>()
    }).collect()
}

fn check_unused_init_value(
    var_data: &mut VarData,
    rvalue_usages: &[Usage],
    lvalue_usages: &[Usage],
) {
    if !rvalue_usages.is_empty() {
        if !lvalue_usages.is_empty() {
            if rvalue_usages[0].get_line_number() > lvalue_usages[0].get_line_number() {
                var_data.set_init_value_unused();
            }
        } else {
            var_data.set_init_value_unused();
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OverlapState {
    Overlap,
    SameLine,
    NoOverlap,
}

// TODO: Create more elegant solution than seperate functions for simply changing the exclusively
// of an inequality
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

// TODO: Derermine if overlapping value uses mutate or don't mutate
// If it doesn't mutate, clone the underlying value instead
#[derive(Debug, Clone)]
pub enum BorrowError {
    MutMutOverlap {
        first_ptr_id: String,
        second_ptr_id: String,
        value_id: String,
    },
    MutConstOverlap {
        mut_ptr_id: String,
        const_ptr_id: String,
        value_id: String,
    },
    MutMutSameLine {
        first_ptr_id: String,
        second_ptr_id: String,
        value_id: String,
    },
    MutConstSameLine {
        mut_ptr_id: String,
        const_ptr_id: String,
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
    ValueConstOverlap {
        ptr_id: String,
        value_id: String,
    },
    ValueConstSameLine {
        ptr_id: String,
        value_id: String,
    },
}
impl PartialEq for BorrowError {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
impl Eq for BorrowError {}

impl PartialOrd for BorrowError {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BorrowError {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (
                BorrowError::ValueMutOverlap {
                    ptr_id: _,
                    value_id: _,
                },
                BorrowError::MutConstOverlap {
                    mut_ptr_id: _,
                    const_ptr_id: _,
                    value_id: _,
                },
            ) => std::cmp::Ordering::Greater,
            (
                BorrowError::MutConstOverlap {
                    mut_ptr_id: _,
                    const_ptr_id: _,
                    value_id: _,
                },
                BorrowError::ValueMutOverlap {
                    ptr_id: _,
                    value_id: _,
                },
            ) => std::cmp::Ordering::Less,
            (_, _) => std::cmp::Ordering::Equal,
        }
    }
}
