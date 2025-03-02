use crate::{
    analysis_ctx::AnalysisContext, ast::TokenNode as Node, checker::BorrowError,
    data_model::Reference,
};

pub fn adjust_ptr_type(errors: Vec<BorrowError>, ctx: &mut AnalysisContext, root: &mut Node) {
    errors.iter().for_each(|error| {
        // A lot of work for nothing
        match &error {
            BorrowError::MutMutOverlap {
                first_ptr_id: _,
                second_ptr_id: _,
                value_id,
            } => set_ptr_rc(value_id, ctx),
            BorrowError::MutConstOverlap {
                mut_ptr_id: _,
                imut_ptr_id: _,
                value_id,
            } => set_ptr_rc(value_id, ctx),
            BorrowError::MutMutSameLine {
                first_ptr_id,
                second_ptr_id,
                value_id: _,
            } => {
                set_ptr_rc(first_ptr_id, ctx);
                // set_raw(second_ptr_id, ctx)
            }

            BorrowError::MutConstSameLine {
                mut_ptr_id,
                imut_ptr_id,
                value_id: _,
            } => {
                // set_raw(mut_ptr_id, ctx);
                // set_raw(imut_ptr_id, ctx);
            }
            // TODO: if the id is the value, we can clone
            BorrowError::ValueMutOverlap {
                ptr_id: _,
                value_id,
            } => {
                set_ptr_rc(value_id, ctx)
                // clone_solution(ptr_id, value_id, ctx, root)
            }
            BorrowError::ValueMutSameLine {
                ptr_id,
                value_id,
                // value_instance_nodes,
            } => {
                // create_clone(value_id, ptr_id, ctx, root, value_instance_nodes.clone());
            }
            BorrowError::ValueConstOverlap {
                ptr_id: _,
                value_id,
            } => {
                set_ptr_rc(value_id, ctx)
                // clone_solution(ptr_id, value_id, ctx, root)
            }
            BorrowError::ValueConstSameLine {
                ptr_id,
                value_id,
                // value_instance_nodes,
            } => {
                // create_clone(value_id, ptr_id, ctx, root, value_instance_nodes.clone());
            }
        };

        // TODO: This should actually traverse the pointer chain downwards
    });
}

/// Checks if a simple rearrangement of lines could fix = the borrow error
///
///
/// # Important
/// A value can be used behind a immutable reference if it's an rvalue that implements copy, which
/// every non-struct, non-ptr c variable does
fn check_line_rearrangement_value_constref_overlap(
    value_id: &str,
    ptr_id: &str,
    root: &mut Node,
    ctx: &mut AnalysisContext,
) {
    let var_data = ctx.get_var(value_id);
    let ptr_data = ctx.get_var(ptr_id);
    let reference = ptr_data.reference_to_var(value_id).unwrap().clone();
}

fn set_ptr_rc(value_id: &str, ctx: &mut AnalysisContext) {
    let var_data = ctx.get_var_mut(value_id);
    var_data.rc = true;

    // TODO distinguish between `ptr = &m` and `let another = &mut ptr`
    // Essentially bring back `is_mut_ptr` and `is_mut_direct`
    var_data.is_mut = false;

    let ptrs = var_data.pointed_to.clone();

    ptrs.iter().for_each(|reference_block| {
        reference_block.borrow_mut().set_rc();
        // TODO Check if we have to cascade RCs
        // set_ptr_rc(reference_block.borrow().get_borrower(), ctx);
    });
}

// This function is problematic because it requires the ast to be changed :(
// Either that, or we could use some othe protocole for conveying a new variable
// Or, we could not add a new variable because we're weak and don't want to change business logic
// If we insert, we need to be able to modify the ast here
// fn create_clone(
//     value_id: &str,
//     _ptr_id: &str,
//     ctx: &mut AnalysisContext,
//     root: &mut Node,
//     value_instance_nodes: Vec<(Box<[Node]>>>, usize)>,
// ) {
//     let var_data = ctx.get_var(value_id);
//     // TODO The make this to be cloned in annotation
//     let clone_expr = Node::new(NodeType::Id(value_id.to_string()), None, 0);
//     // TODO Get CType
//     let clone_id = format!("{}_clone", value_id);
//     let clone_declaration = Node::new(
//         NodeType::Declaration(clone_id.clone(), CType::Int, var_data.addresses.len()),
//         Some(Rc::new(RefCell::new(Box::new([clone_expr])))),
//         0,
//     );
//     // This symbol goes after the new node
//     let place_before_symbol = &var_data.pointed_to_by[0];
//
//     fn search(root: &Node, place_before_symbol: &str) -> Option<(Node, usize)> {
//         match root.children.as_ref() {
//             Some(children) => {
//                 for (i, child) in children.borrow().iter().enumerate() {
//                     println!("child token: {:?}", child.token);
//                     match &child.token {
//                         NodeType::Declaration(var_id, _, _) if *var_id == place_before_symbol => {
//                             return Some((root.clone(), i));
//                         }
//                         NodeType::PtrDeclaration(var_id, _, _)
//                             if *var_id == place_before_symbol =>
//                         {
//                             return Some((root.clone(), i));
//                         }
//                         _ => {}
//                     }
//                     if let Some(parent) = search(child, place_before_symbol) {
//                         return Some(parent);
//                     };
//                 }
//             }
//             None => return None,
//         }
//         None
//     }
//
//     // Nodes that are on the same line as other nodes that reference them
//     let same_line_nodes = value_instance_nodes.iter().filter(|nodes| {
//         let node = &nodes.0.borrow()[nodes.1];
//         value_instance_nodes
//             .iter()
//             .any(|other_nodes| other_nodes.0.borrow()[other_nodes.1].line == node.line)
//     });
//
//     // TODO Figure out what to do with this
//
//     let ret = search(root, place_before_symbol);
//     if let Some((mut parent, i)) = ret {
//         let children = parent
//             .children
//             .as_mut()
//             .expect("Parent doesn't have children");
//         // TODO This doesn't work, find way to modify ast
//         let mut new = children.borrow().clone().to_vec();
//         new.insert(i, clone_declaration);
//         *children.borrow_mut() = new.into_boxed_slice();
//
//         println!("HERERERERERE {:?}", children.borrow());
//         // TODO Consider when to take a value_instance_node
//         println!("{:?}", value_instance_nodes);
//
//         for (sibiling_nodes, i) in value_instance_nodes.iter() {
//             match &sibiling_nodes.borrow()[*i].token {
//                 NodeType::Id(_id) => {}
//                 _ => panic!("value instance node must be id"),
//             }
//
//             sibiling_nodes.borrow_mut()[*i].token = NodeType::Id(clone_id.clone());
//
//             // NOTE Run the analyzer and checker again with the new variable
//             *ctx = AnalysisContext::new();
//             println!("NEW ITERATION\n\n");
//             analyzer::determine_var_mutability(root, ctx, Rc::new(RefCell::new(Box::new([]))), 0);
//             let new_errors = checker::borrow_check(ctx);
//             adjust_ptr_type(new_errors, ctx, root);
//         }
//     }
// }
