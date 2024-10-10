use crate::error::ErrType as ET;
use crate::parser::{AssignmentOpType, NodeType, TokenNode as Node};

enum PtrType {
    TrueRaw,

    ConstPtrConst,
    ConstPtrMut,
    MutPtrConst,
    MutPtrMut,

    ConstRef,
    MutRef,
}

struct Ptr {
    name: String,
    t: PtrType,
    is_mut: bool,
}

struct StackData {
    occurences: Vec<Node>,
    refs: Vec<Ptr>, // type_t: we don't care about how large data is
}

struct Function<'a> {
    ptr_params: Vec<&'a StackData>,
    owned_params: Vec<&'a StackData>,
}

struct Arena {
    data: Vec<StackData>,
}

/// Returns a vector of all pointers pointing to this id
/// Note: It only returns explicit pointers
fn get_all_pointers(var_name: &String, root: &Node, is_assignment: bool) -> Vec<Ptr> {
    root.children
        .as_ref()
        .unwrap()
        .iter()
        .flat_map(|child| {
            // TODO: Figure out how to check what pointers point to what, since some have names and
            // some don't
            // There's a difference between dereferencing and referencing that needs to be resolved
            let this_is_assignment = if let NodeType::Assignment(_) = &child.token {
                true
            } else {
                is_assignment
            };
            let mut sub_ptrs = get_all_pointers(var_name, child, this_is_assignment);
            if child.token == NodeType::Adr(var_name.clone()) {
                // FIXME: We actually want to traverse the tree upwards here for an assignment node
                let is_mut = if std::mem::discriminant(&root.token)
                    == std::mem::discriminant(&NodeType::Assignment(AssignmentOpType::Eq))
                {
                    match &root.children.as_ref().unwrap()[0].token {
                        // FIXME: Should look start with the parent node of root instead
                        NodeType::Id(id) => is_mut(&id, root),
                        _ => panic!("Expected first assignment child to be id"),
                    }
                } else {
                    false
                };
                // let is_mut = is_mut();
                let ptr = Ptr {
                    name: var_name.clone(),
                    t: PtrType::TrueRaw,
                    is_mut,
                };
                sub_ptrs.push(ptr);
            }
            sub_ptrs
        })
        .collect()
}

/// Checks if a pointer is ever defererenced and modified
fn is_mut(ptr_name: &String, root: &Node) -> bool {
    for child in root.children.as_ref().unwrap().iter() {
        if is_mut(ptr_name, &child) {
            return true;
        }
        if std::mem::discriminant(&child.token)
            != std::mem::discriminant(&NodeType::Assignment(AssignmentOpType::Eq))
        {
            continue;
        }

        let deref = &child.children.as_ref().unwrap()[0];
        let expression = &deref.children.as_ref().unwrap();
        if !(deref.token == NodeType::DeRef) || !(expression.len() == 1) {
            continue;
        }

        if let NodeType::Id(ptr) = &expression[0].token {
            if *ptr == *ptr_name {
                return true;
            }
        }
    }

    return false;
}
