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
    name: String,
    points_to: String,
    t: PtrType,
    is_mut: bool,
}

#[derive(Debug)]
struct StackData {
    occurences: Vec<Node>,
    refs: Vec<Ptr>, // type_t: we don't care about how large data is
}

#[derive(Debug)]
struct Function<'a> {
    ptr_params: Vec<&'a StackData>,
    owned_params: Vec<&'a StackData>,
}

#[derive(Debug)]
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
pub fn get_all_pointers(var_name: &String, root: &Node, ptrs: Vec<Ptr>) -> Vec<Ptr> {
    println!("root: {:?}\n", root);
    let mut new_ptrs: Vec<Ptr> = vec![];

    let first_child = &root.children.as_ref().unwrap()[0];
    let this_is_assignment = match (&root.token, &first_child.token) {
        (_, NodeType::Id(_)) => AssignmentBool::IsAssignment(false),
        (NodeType::Assignment(_), NodeType::DeRef) => {
            match ptrs.iter().map(|ptr| ptr.name).position(ptr_name) {
                Some(i) => ptrs[i].is_mut = true,
                None => 
            }
        }
        (NodeType::Adr(adr_name), _) => {
            if adr_name == var_name {
                let is_mut = match &is_assignment {
                    AssignmentBool::IsAssignment(is_mut) => *is_mut,
                    AssignmentBool::NotAssignment => false,
                };
                let ptr = Ptr {
                    points_to: var_name.clone(),
                    t: PtrType::TrueRaw,
                    is_mut,
                };
                new_ptrs.push(ptr);
            }

            is_assignment
        }
        _ => is_assignment,
    };

    if let Some(children) = &root.children {
        let mut sub_pointers = children
            .iter()
            .flat_map(|child| get_all_pointers(var_name, child, this_is_assignment))
            .collect::<Vec<Ptr>>();
        new_ptrs.append(&mut sub_pointers);
    }

    new_ptrs
}

/// Checks if a pointer is ever defererenced and modified
pub fn is_mut(ptr_name: &String, root: &Node) -> bool {
    // TODO: Fix this being scuffed
    let blank = vec![];
    for child in root.children.as_ref().unwrap_or_else(|| &blank).iter() {
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
