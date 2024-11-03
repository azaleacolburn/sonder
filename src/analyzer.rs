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
enum PtrUsage<'a> {
    DerefL { lvalue: &'a Node, rvalue: &'a Node }, // deref is on the left side of the statement
    DerefR { rvalue: &'a Node },                   // deref is on the right side of statement
    AssignL { lvalue: &'a Node, rvalue: &'a Node }, // ptr is on the left side of assignment
    AssignR { rvalue: &'a Node },                  // ptr is on the right side of assignment
}

#[derive(Debug, Clone)]
pub struct Ptr<'a> {
    name: String,
    points_to: &'a Node,
    t: PtrType,
    is_mut: bool,
    deref_instances: Vec<PtrUsage<'a>>,
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
pub fn get_all_pointers_and_derefs<'a>(root: &'a Node, ptrs: &mut Vec<Ptr<'a>>) -> Vec<Ptr<'a>> {
    let mut sub_ptrs: Vec<Ptr> = match &root.children {
        Some(children) => children
            .iter()
            .flat_map(|child| get_all_pointers_and_derefs(child, ptrs))
            .collect(),
        None => vec![],
    };

    match &root.token {
        NodeType::Assignment(_, id) => {
            if sub_ptrs.len() > 1 {
                println!("More than one dereferenced pointer in R-value no non-deref assignment on id: {id}");
            }
        }
        NodeType::DeRef(node) => {
            println!(
                "deref_node_children: {:?}",
                node.children.as_ref().expect("Deref node children empty")
            );
            let deref_id_node = node
                .children
                .as_ref()
                .unwrap()
                .iter()
                .find(|node| {
                    std::mem::discriminant(&node.token)
                        == std::mem::discriminant(&NodeType::Id(String::new()))
                })
                .unwrap_or(node);
            if let NodeType::Id(id) = &deref_id_node.token {
                // FIXME: This doesn't support: *(t + f)
                let usage = PtrUsage::DerefR { rvalue: node };

                ptrs.iter_mut()
                    .find(|ptr| ptr.name == *id)
                    .expect("Non-ptr")
                    .deref_instances
                    .push(usage);
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
            println!("node: {:?}", deref_node);

            let deref_id: String = if let NodeType::Id(id) = &deref_node.token {
                println!("here");
                id.clone()
            } else {
                println!(
                    "deref_node_children: {:?}",
                    deref_node
                        .children
                        .as_ref()
                        .expect("Deref node children empty")
                );

                let deref_ids: Vec<Node> = deref_node
                    .children
                    .as_ref()
                    .expect("No id being assigned")
                    .iter()
                    .filter(|node| {
                        std::mem::discriminant(&node.token)
                            == std::mem::discriminant(&NodeType::Id(String::new()))
                    })
                    .map(|node| node.clone())
                    .collect();
                if deref_ids.len() > 1 {
                    panic!(
                        "Multiple ptrs deref assigned at once, all things here are raw pointers"
                    );
                } else {
                    if let NodeType::Id(id) = &deref_ids[0].token {
                        // Write handler later
                        println!("ptrs: {:?}", ptrs);
                        id.clone()
                    } else {
                        panic!("Non-Id made it through filtering out non-ids")
                    }
                }
            };
            let dereffed_ptr = ptrs
                .iter_mut()
                .find(|ptr| ptr.name == deref_id)
                .expect("None of the ids in deref assignment are ptrs");
            dereffed_ptr.is_mut = true;
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

    sub_ptrs
}
