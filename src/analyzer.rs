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

/// Returns a vector of all pointers and derefts
/// Note: It only returns explicit pointers
/// TODO: Make pointer grabbing and mut checking for every variable a single sweep over the tree
///
pub fn get_all_pointers_and_derefs<'a>(
    root: &'a Node,
    ptrs: &mut Vec<Ptr<'a>>,
    derefs: &Vec<Deref<'a>>,
) -> (Vec<Ptr<'a>>, Vec<Deref<'a>>) {
    let sub_ptrs_and_derefs: Vec<(Vec<Ptr>, Vec<Deref>)> = match &root.children {
        Some(children) => children
            .iter()
            .map(|child| get_all_pointers_and_derefs(child, ptrs, derefs))
            .collect(),
        None => vec![],
    };

    let mut sub_ptrs: Vec<Ptr> = sub_ptrs_and_derefs
        .clone()
        .into_iter()
        .flat_map(|pair| pair.0)
        .collect();
    let sub_derefs: Vec<Deref> = sub_ptrs_and_derefs
        .into_iter()
        .flat_map(|pair| pair.1)
        .collect();

    match &root.token {
        NodeType::Assignment(_, id) => {
            if sub_ptrs.len() > 1 {
                println!("More than one dereferenced pointer in R-value no non-deref assignment on id: {id}");
            }
        }
        NodeType::DeRef(node) => {
            let deref_id_node = node
                .children
                .as_ref()
                .unwrap()
                .iter()
                .find(|node| {
                    std::mem::discriminant(&node.token)
                        == std::mem::discriminant(&NodeType::Id(String::new()))
                })
                .expect("No id in deref");
            if let NodeType::Id(id) = &deref_id_node.token {
                // FIXME: This doesn't support: *(t + f)
                let usage = Usage {
                    usage_type: PtrUsage::DerefR,
                    lvalue: &'a 

                };
                ptrs.iter_mut()
                    .find(|ptr| ptr.name == *id)
                    .expect("Non-ptr")
                    .deref_instances
                    .push();
            } else {
                panic!("Non-Id made it through filtering out non-ids");
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

            // L-side deref ids
            let deref_ids: Vec<Node> = deref_node
                .children
                .as_ref()
                .unwrap()
                .iter()
                .filter(|node| {
                    std::mem::discriminant(&node.token)
                        == std::mem::discriminant(&NodeType::Id(String::new()))
                })
                .map(|node| node.clone())
                .collect();

            println!("l-side deref ids: {:?}", deref_ids);

            if deref_ids.len() > 1 {
                println!("Multiple ptrs deref assigned at once, all things here are raw pointers");
            } else {
                if let NodeType::Id(id) = &deref_ids[0].token {
                    // Write handler later
                    println!("ptrs: {:?}", ptrs);
                    let dereffed_ptr = ptrs
                        .iter_mut()
                        .find(|ptr| ptr.name == *id)
                        .expect("None of the ids in deref assignment are ptrs");
                    dereffed_ptr.is_mut = true;
                } else {
                    panic!("Non-Id made it through filtering out non-ids")
                }
            }
        }
        NodeType::PtrDeclaration(name, _ptr_type, points_to) => {
            let ptr = Ptr {
                name: name.clone(),
                points_to: &*points_to,
                deref_instances: vec![],
                t: PtrType::TrueRaw,
                is_mut: false,
            };
            sub_ptrs.push(ptr);
        }
        _ => {
            println!("Not semantically important token for grabbing ptrs");
        }
    }

    ptrs.append(&mut (sub_ptrs.clone()));

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
