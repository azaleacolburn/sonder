use std::collections::HashMap;

use crate::parser::{NodeType, TokenNode as Node};

#[derive(Debug, Clone)]
enum PtrType {
    TrueRaw,

    ConstPtrConst,
    ConstPtrMut,
    MutPtrConst,
    MutPtrMut,

    ConstRef,
    MutRef,
}

#[derive(Debug, Clone)]
enum PtrUsage {
    DerefL,
    DerefR,
    AssignL,
    AssignR,
}

#[derive(Debug, Clone)]
pub struct Ptr<'a> {
    name: String,
    points_to: &'a Node,
    t: PtrType,
    is_mut: bool,
    deref_instances: Vec<Usage<'a>>,
}

#[derive(Debug, Clone)]
pub struct Usage<'a> {
    usage_type: PtrUsage,
    lvalue: &'a Node,
    rvalue: &'a Node,
}

#[derive(Debug, Clone)]
pub struct Deref<'a> {
    ptr_name: String,
    deref_type: DerefType,
    lvalue: &'a Node,
    rvalue: &'a Node,
}

#[derive(Debug, Clone)]
pub enum DerefType {
    Left,
    Right,
}

/// We need this to check if pointer values are being reassigned
#[derive(Clone, Copy, Debug)]
pub enum AssignmentBool {
    IsAssignment(bool), // id being assigned to, is_mut
    NotAssignment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarData<'a> {
    is_ptr: bool,
    pointed_to_by: Vec<&'a str>,
    is_mut_by_ptr: bool,
    is_mut_direct: bool,
}

pub fn determine_var_mutability<'a>(root: &'a Node) -> HashMap<String, VarData> {
    let mut vars: HashMap<String, VarData> = HashMap::new();
    match &root.token {
        NodeType::Declaration(id, _, _) => {
            vars.insert(
                id.to_string(),
                VarData {
                    is_ptr: false,
                    pointed_to_by: vec![],
                    is_mut_by_ptr: false,
                    is_mut_direct: false,
                },
            );
        }
        NodeType::Assignment(_, id) => {
            vars.get_mut(id).expect("Undeclared Id").is_mut_direct = true;
        }
        NodeType::PtrDeclaration(id, _, expr) => {
            let var = VarData {
                is_ptr: true,
                pointed_to_by: vec![],
                is_mut_by_ptr: false,
                is_mut_direct: false,
            };
            let expr_id = find_ids(expr);
            vars.insert(id.to_string(), var);
            // Doesn't support &that + &this
            // This immediantly breakes borrow checking rules
            vars.get_mut(&expr_id[0])
                .expect("Undeclared Id")
                .pointed_to_by
                .push(id);
        }
        NodeType::DerefAssignment(_, l_side) => {
            let deref_ids = find_ids(&l_side);
            // This breakes because `*(t + s) = bar` is not allowed
            if deref_ids.len() > 1 {
                panic!("Unsupported: Multiple items dereferenced");
            }
            let ptr_id: &str = &deref_ids[0];
            let mutated_var_name: Vec<String> = vars
                .iter()
                .filter(|(_name, data)| data.pointed_to_by.contains(&ptr_id))
                .map(|(name, _data)| name.clone())
                .collect();
            if mutated_var_name.len() > 1 {
                panic!("Unsupported: Pointer points to more than one thing");
            }
            vars.get_mut(&mutated_var_name[0])
                .expect("Undeclared Id Being Dereferenced")
                .is_mut_by_ptr = true;
        }
        _ => {}
    };
    vars
}

fn find_ids<'a>(root: &'a Node) -> Vec<String> {
    let mut ids: Vec<String> = root
        .children
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .flat_map(|child| find_ids(child))
        .collect();
    if let NodeType::Id(id) = &root.token {
        ids.push(id.to_string());
    }
    ids
}

// /// Returns a vector of all pointers and derefts
// /// Note: It only returns explicit pointers
// /// TODO: Make pointer grabbing and mut checking for every variable a single sweep over the tree
// ///
// pub fn get_all_pointers_and_derefs<'a>(root: &'a Node, ptrs: &mut Vec<Ptr<'a>>) -> Vec<Ptr<'a>> {
//     let mut sub_ptrs: Vec<Ptr> = match &root.children {
//         Some(children) => children
//             .iter()
//             .map(|child| get_all_pointers_and_derefs(child, ptrs))
//             .collect(),
//         None => vec![],
//     };
//
//     match &root.token {
//         NodeType::Assignment(_, id) => {
//             if sub_ptrs.len() > 1 {
//                 println!("More than one dereferenced pointer in R-value no non-deref assignment on id: {id}");
//             }
//         }
//         NodeType::DeRef(node) => {
//             let deref_id_node = node
//                 .children
//                 .as_ref()
//                 .unwrap()
//                 .iter()
//                 .find(|node| {
//                     std::mem::discriminant(&node.token)
//                         == std::mem::discriminant(&NodeType::Id(String::new()))
//                 })
//                 .expect("No id in deref");
//             if let NodeType::Id(id) = &deref_id_node.token {
//                 // FIXME: This doesn't support: *(t + f)
//                 todo!();
//                 let usage = Usage {
//                     usage_type: PtrUsage::DerefR,
//                 };
//                 ptrs.iter_mut()
//                     .find(|ptr| ptr.name == *id)
//                     .expect("Non-ptr")
//                     .deref_instances
//                     .push(usage);
//             } else {
//                 panic!("Non-Id made it through filtering out non-ids");
//             }
//         }
//         NodeType::DerefAssignment(_, deref_node) => {
//             // FIXME: This doesn't support things like
//             // int g = 0;
//             // int y = 9;
//             // int* j = &g;
//             // *(j + y) = 7;
//             //
//             // Where there are two variables being dereferenced together
//
//             // L-side deref ids
//             let deref_ids: Vec<Node> = deref_node
//                 .children
//                 .as_ref()
//                 .unwrap()
//                 .iter()
//                 .filter(|node| {
//                     std::mem::discriminant(&node.token)
//                         == std::mem::discriminant(&NodeType::Id(String::new()))
//                 })
//                 .map(|node| node.clone())
//                 .collect();
//
//             println!("l-side deref ids: {:?}", deref_ids);
//
//             if deref_ids.len() > 1 {
//                 println!("Multiple ptrs deref assigned at once, all things here are raw pointers");
//             } else {
//                 if let NodeType::Id(id) = &deref_ids[0].token {
//                     // Write handler later
//                     println!("ptrs: {:?}", ptrs);
//                     let dereffed_ptr = ptrs
//                         .iter_mut()
//                         .find(|ptr| ptr.name == *id)
//                         .expect("None of the ids in deref assignment are ptrs");
//                     dereffed_ptr.is_mut = true;
//                 } else {
//                     panic!("Non-Id made it through filtering out non-ids")
//                 }
//             }
//             let dereffed_ptr = ptrs
//                 .iter_mut()
//                 .find(|ptr| ptr.name == deref_id)
//                 .expect("None of the ids in deref assignment are ptrs");
//             dereffed_ptr.is_mut = true;
//         }
//         NodeType::PtrDeclaration(name, _ptr_type, points_to) => {
//             let ptr = Ptr {
//                 name: name.clone(),
//                 points_to: &*points_to,
//                 deref_instances: vec![],
//                 t: PtrType::TrueRaw,
//                 is_mut: false,
//             };
//             sub_ptrs.push(ptr);
//         }
//         _ => {
//             println!("Not semantically important token for grabbing ptrs");
//         }
//     }
//
//     ptrs.append(&mut (sub_ptrs.clone()));
//
//     sub_ptrs
// }
