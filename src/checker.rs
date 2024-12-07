use crate::analyzer::{AnalysisContext, PtrType, VarData};
use std::ops::Range;

// TODO: Derermine if overlapping value uses mutate or don't mutate
// If it doesn't mutate, clone the underlying value instead
#[derive(Debug, Clone)]
pub enum BorrowError {
    MutMutOverlap {
        first_ptr_id: String,
        second_ptr_id: String,
        sub_id: String,

    },
    MutImutOverlap {
        mut_ptr_id: String,
        imut_ptr_id: String,
        sub_id: String,
        
    },
    MutValueOverlap {
        ptr_id: String,
        value_id: String
    },
}


pub fn adjust_ptr_type<'a>(errors: Vec<BorrowError>, ctx: &mut AnalysisContext<'a>) {
    errors.iter().for_each(|error| {
        // A lot of work for nothing
        let ptr_data = ctx.get_var(&error.id).expect("Ptr in error not in map");
        let ref_during_error = ctx.find_which_ref_at_id(&error.id, error.line);
        let ref_mutates = ptr_data
            .addresses
            .iter()
            .find(|ref_data| ref_data.borrow().adr_of == ref_during_error)
            .expect("Ref not assigned to ptr")
            .borrow()
            .mutates;
        let new_ptr_type = match (&error.err, ref_mutates) {
            (BorrowError::MutMutOverlap, _) => PtrType::RcClone,
            (BorrowError::MutImutOverlap, _) => PtrType::RcClone,
            // TODO: if the id is the value, we can clone
            (BorrowError::MutValueOverlap, _) => PtrType::RcClone,
        };
        println!("ERROR ID: {}", error.id);

        // TODO: This should actually traverse the pointer chain downwards
        let sub_id = ctx
            .get_adr(&ref_during_error)
            .expect("Adr not in map")
            .borrow()
            .adr_of
            .clone();
        ctx.mut_var(sub_id.to_string(), |var_data| var_data.rc = true);

        let sub_var = ctx.get_var(&sub_id).expect("Sub id not in map");
        let other_same_level_ptrs = sub_var.pointed_to_by.clone();

        other_same_level_ptrs.iter().for_each(|ptr| {
            ctx.mut_var(ptr.to_string(), |other_ptr_data| {
                // TODO: Grab what the type was of the dpecific ref that pointed to the ref that we
                // care about
                let mut other_ptr_data_for_same_sub_var = other_ptr_data
                    .addresses
                    .iter()
                    .find(|other_ptr_data| other_ptr_data.borrow().adr_of == sub_id)
                    .expect("Should both have to the same object")
                    .borrow_mut();
                let other_ref_level_len = other_ptr_data_for_same_sub_var.ptr_type.len() - 1;

                // TODO: We want the specific reference that applies at the same level of the
                // problematic ptr
                // This is a really hard problem
                // For now we'll just make the top ptr_type be an RcClone
                other_ptr_data_for_same_sub_var.ptr_type[other_ref_level_len] =
                    new_ptr_type.clone();
            });
        });

        ctx.mut_var(error.id.to_string(), |var_data| {
            let mut ptr_data = var_data
                .addresses
                .iter()
                .find(|r| r.borrow().adr_of == ref_during_error)
                .unwrap()
                .borrow_mut();
            let ref_level_len = ptr_data.ptr_type.len() - 1;
            ptr_data
                // TODO: Determine correct solution to multiple reference issue
                // This *probably* works
                .ptr_type[ref_level_len] = new_ptr_type;
        });
    });
}

// TODO: Figure out how to include line numbers in error reports
pub fn borrow_check<'a>(ctx: &AnalysisContext<'a>) -> Vec<BorrowErrorReport> {
    ctx.variables
        .iter()
        .flat_map(|(id, var_data)| -> Vec<(String, BorrowError)> {
            let pointed_to_by: Vec<(String, &VarData<'a>, PtrType)> = var_data
                .pointed_to_by
                .iter()
                .map(|ptr| {
                    let ptr_var_data = ctx.get_var(ptr).expect("ptr to var not in scope");
                    let ptr_data_to_var = ptr_var_data
                        .addresses
                        .iter()
                        .find(|ptr_data| ptr_data.borrow().adr_of == *id)
                        .expect("Ptr does not record reference to var in map");
                    (
                        ptr.to_string(),
                        ptr_var_data,
                        // TODO: Check if this is a fine solution, if the top is mutable than all
                        // the rest should be too
                        // But is this ever what we want
                        ptr_data_to_var.borrow().ptr_type[0].clone(),
                    )
                })
                .collect();
            println!("{id} is pointed to by: {:?}", pointed_to_by);
            let pointed_to_by_mutably = pointed_to_by
                .iter()
                .filter(|(_, _, ref_type)| *ref_type == PtrType::MutRef);
            let mut value_overlaps_with_mut_ptr: Vec<(String, BorrowError, usize)> =
                pointed_to_by_mutably
                    .clone()
                    .filter(|(_, mut_ptr_data, _)| {
                        // This fails because the value and the pointer are going to overlap on the line
                        // the ref to the pointer is taken
                        // eg. `let m = &mut n`
                        var_active_range_overlap(
                            mut_ptr_data.non_borrowed_lines.clone(),
                            var_data.non_borrowed_lines.clone(),
                        )
                    })
                    .map(|(ptr_id, _, _)| (ptr_id.clone(), BorrowError::MutValueOverlap, ))
                    .collect();
            let mut mutable_ref_overlaps: Vec<(String, BorrowError, usize)> = pointed_to_by_mutably
                .flat_map(|(_, mut_ptr_data, _)| {
                    pointed_to_by
                        .iter()
                        .filter(|(_, other_ptr_data, _)| mut_ptr_data != other_ptr_data)
                        .filter(|(_, other_ptr_data, _)| {
                            both_ptr_active_range_overlap(
                                mut_ptr_data.non_borrowed_lines.clone(),
                                other_ptr_data.non_borrowed_lines.clone(),
                            )
                        })
                        .map(|(id, _, ref_type)| match ref_type {
                            PtrType::MutRef => (id.clone(), BorrowError::MutMutOverlap),
                            PtrType::ImutRef => (id.clone(), BorrowError::MutImutOverlap),
                            _ => panic!("Basic ref should not have smart ptr type"),
                        })
                        .collect::<Vec<(String, BorrowError)>>()
                })
                .collect();
            println!(
                "value_overlaps_with_mut_ptr {id}: {:?}\nmutable_ref_overlaps {id}: {:?}",
                value_overlaps_with_mut_ptr, mutable_ref_overlaps
            );
            value_overlaps_with_mut_ptr.append(&mut mutable_ref_overlaps);
            value_overlaps_with_mut_ptr
        })
        .collect()
}

// TODO: Create more elegant solution than seperate functions for simply changing the exclusively
// of an inequality
//
// Returns the function
pub fn both_ptr_active_range_overlap(l_1: Vec<Range<usize>>, l_2: Vec<Range<usize>>) -> Option<{
    let ranges_overlap =
        |l_1: &Range<usize>, l_2: &Range<usize>| l_1.start <= l_2.end && l_2.start <= l_1.end;
    l_1.iter()
        .flat_map(|l_1| l_2.iter().map(|l_2| ranges_overlap(l_1, l_2)))
        .any(|overlaps| overlaps)
}
pub fn var_active_range_overlap(l_1: Vec<Range<usize>>, l_2: Vec<Range<usize>>) -> bool {
    let ranges_overlap =
        |l_1: &Range<usize>, l_2: &Range<usize>| l_1.start < l_2.end && l_2.start < l_1.end;
    l_1.iter()
        .flat_map(|l_1| l_2.iter().map(|l_2| ranges_overlap(l_1, l_2)))
        .any(|overlaps| overlaps)
}
