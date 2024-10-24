use crate::parser::{AssignmentOpType, NodeType, TokenNode as Node};

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
    points_to: String,
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

/// Returns a vector of all pointers and derefts
/// Note: It only returns explicit pointers
/// TODO: Make pointer grabbing and mut checking for every variable a single sweep over the tree
///
pub fn get_all_pointers_and_derefs<'a>(
    root: &Node,
    ptrs: Vec<Ptr<'a>>,
    derefs: Vec<Deref<'a>>,
) -> (Vec<Ptr<'a>>, Vec<Deref<'a>>) {
    println!("root: {:?}\n", root);

    let (sub_ptrs, sub_derefs): (Vec<Ptr>, Vec<Deref>) = match &root.children {
        Some(children) => children
            .iter()
            .map(|child| get_all_pointers_and_derefs(child)),
        None => (vec![], vec![]),
    };

    match &root.token {
        NodeType::Assignment(_, id) => {
            if sub_ptrs.len() > 1 {
                println!("More than one dereferenced pointer in R-value no non-deref assignment");
            }
        }
        NodeType::DerefAssignment(_, deref_node) => {
            // FIXME: This doesn't support things like
            // int g = 0;
            // int y = 9;
            // int* j = &g;
            // *(j + y) = 7;
            //
            // Where there are two variables being dereferenced together

            let deref_ids: Vec<NodeType> = deref_node
                .children
                .as_ref()
                .unwrap()
                .iter()
                .filter(|node| {
                    std::mem::discriminant(&node.token) == std::mem::discriminant(&NodeType::Id)
                })
                .collect();

            println!("deref_ids: {:?}", deref_ids);

            if deref_ids.len() > 1 {
                println!("Multiple ptrs deref assigned at once, all things here are raw pointers");
            } else {
                if let NodeType::Id(id) = deref_ids[0] {
                    let dereffed_ptr = sub_ptrs
                        .iter()
                        // () => {}
                        .find(|ptr| ptr.name == id)
                        .expect("only id in deref assignment wasn't a pointer");
                    dereffed_ptr.is_mut = true;
                }
                panic!("Id made it through filtering out non-ids")
            }
        }
        _ => {}
    }

    (sub_ptrs, sub_derefs)
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
            != std::mem::discriminant(&NodeType::Assignment(
                AssignmentOpType::Eq,
                ptr_name.to_string(),
            ))
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
