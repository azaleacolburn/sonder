use crate::error::ErrType as ET;
use crate::parser::{AssignmentOpType, NodeType, TokenNode as Node};

enum PtrType {
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

/// Checks if a pointer is ever defererenced and modified
fn is_mut(ptr_name: &String, root: &Node) -> bool {
    for child in root.children.as_ref().unwrap().iter().filter(|child| {
        std::mem::discriminant(&child.token)
            == std::mem::discriminant(&NodeType::Assignment(AssignmentOpType::Eq))
    }) {
        let deref = &child.children.as_ref().unwrap()[0];
        let expression = &deref.children.as_ref().unwrap();
        if deref.token == NodeType::DeRef && expression.len() == 1 {
            if let NodeType::Id(ptr) = &expression[0].token {
                if *ptr == *ptr_name {
                    return true;
                }
            }
        }
        if is_mut(ptr_name, &child) {
            return true;
        }
    }

    return false;
}
