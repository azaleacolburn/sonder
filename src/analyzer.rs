use crate::parser::{AssignmentOpType, NodeType, TokenNode as Node};

#[derive(Debug)]
enum PtrType {
    TrueRaw,

    ConstPtrConst,
    ConstPtrMut,
    MutPtrConst,
    MutPtrMut,

    ConstRef,
    MutRef,
}

#[derive(Debug)]
pub struct Ptr {
    points_to: String,
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

#[derive(Clone, Copy, Debug)]
pub enum AssignmentBool {
    IsAssignment(bool), // id being assigned to, is_mut
    NotAssignment,
}

/// Returns a vector of all pointers pointing to this id
/// Note: It only returns explicit pointers
/// TODO: Make pointer grabbing and mut checking for every variable a single sweep over the tree
pub fn get_all_pointers(var_name: &String, root: &Node, is_assignment: AssignmentBool) -> Vec<Ptr> {
    root.children
        .as_ref()
        .unwrap()
        .iter()
        .flat_map(|child| {
            let this_is_assignment = if let NodeType::Assignment(_) = &child.token {
                let is_mut = match &child.children.as_ref().unwrap()[0].token {
                    NodeType::Id(id) => is_mut(&id, root),
                    _ => panic!("Expeceter first assignment child to be id"),
                };
                AssignmentBool::IsAssignment(is_mut)
            } else {
                is_assignment
            };
            let mut sub_ptrs = get_all_pointers(var_name, child, this_is_assignment);
            if child.token == NodeType::Adr(var_name.clone()) {
                let is_mut = match &is_assignment {
                    AssignmentBool::IsAssignment(is_mut) => *is_mut,
                    AssignmentBool::NotAssignment => false,
                };
                let ptr = Ptr {
                    points_to: var_name.clone(),
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
pub fn is_mut(ptr_name: &String, root: &Node) -> bool {
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
