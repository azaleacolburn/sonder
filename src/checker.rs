use crate::analyzer::{PtrType, VarData};
use std::collections::HashMap;
use std::ops::Range;

pub enum BorrowError {
    MutMutOverlap,
    MutImutOverlap,
    ValueMutOverlap,
}

pub fn adjust_ptr_type<'a>(
    errors: Vec<(String, BorrowError)>,
    vars: &mut HashMap<String, VarData<'a>>,
) {
    errors.iter().for_each(|(id, err)| {
        let new_ptr_type = match err {
            BorrowError::MutMutOverlap => PtrType::Rc,
            BorrowError::MutImutOverlap => PtrType::RefCell,
            BorrowError::ValueMutOverlap => PtrType::RawPtr,
        };
        vars.entry(id.to_string()).and_modify(|var_data| {
            let ptr_data = var_data
                .ptr_data
                .as_mut()
                .expect("Ptr not a ptr in map lol");
            let ptr_type_len = ptr_data.ptr_type.len() - 1;
            ptr_data
                // TODO: Determine correct solution to multiple reference issue
                // This *probably* works
                .ptr_type[ptr_type_len] = new_ptr_type;
        });
    });
}

pub fn borrow_check<'a>(vars: &HashMap<String, VarData<'a>>) -> Vec<(String, BorrowError)> {
    vars.iter()
        .flat_map(|(id, var_data)| -> Vec<(String, BorrowError)> {
            let pointed_to_by: Vec<(String, &VarData<'a>, PtrType)> = var_data
                .pointed_to_by
                .iter()
                .map(|ptr| {
                    let var_data = vars.get(*ptr);
                    (
                        ptr.to_string(),
                        *var_data.as_ref().expect("Ptr not listed in vars"),
                        match var_data
                            .as_ref()
                            .expect("Ptr not listed in vars")
                            .ptr_data
                            .as_ref()
                            .expect(format!("Ptr {ptr} to {id} not a ptr in var map").as_str())
                            .mutates
                        {
                            true => PtrType::MutRef,
                            false => PtrType::ImutRef,
                        },
                    )
                })
                .collect();
            println!("{id} is pointed to by: {:?}", pointed_to_by);
            let pointed_to_by_mutably = pointed_to_by
                .iter()
                .filter(|(_, _, ref_type)| *ref_type == PtrType::MutRef);
            let mut value_overlaps_with_mut_ptr: Vec<(String, BorrowError)> = pointed_to_by_mutably
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
                .map(|(id, _, _)| (id.clone(), BorrowError::ValueMutOverlap))
                .collect();
            let mut mutable_ref_overlaps: Vec<(String, BorrowError)> = pointed_to_by_mutably
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
            value_overlaps_with_mut_ptr.append(&mut mutable_ref_overlaps);
            value_overlaps_with_mut_ptr
        })
        .collect()
}

// TODO: Create more elegant solution than seperate functions for simply changing the exclusively
// of an inequality
pub fn both_ptr_active_range_overlap(l_1: Vec<Range<usize>>, l_2: Vec<Range<usize>>) -> bool {
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
