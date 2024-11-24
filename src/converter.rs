use crate::{
    analyzer::{AnnotatedNode, AnnotatedNodeT},
    lexer::CType,
    parser::AssignmentOpType,
};
pub fn convert_annotated_ast(root: &AnnotatedNode) -> String {
    match &root.token {
        AnnotatedNodeT::PtrDeclaration {
            id,
            is_mut,
            ptr_data,
            t,
            adr,
        } => {
            let rust_t = match t {
                CType::Void => "()",
                CType::Int => "i32",
                CType::Char => "u8",
            };
            //let rust_adr = convert_annotated_ast(&adr);
            let ref_type = if ptr_data.mutates { " &mut " } else { " &" };

            let mut_binding = if *is_mut { " mut " } else { "" };
            // Only supports one reference at a time
            format!("let{mut_binding}{id}: {rust_t} ={ref_type}{adr};")
        }
        AnnotatedNodeT::DerefAssignment { op, adr } => {
            let rust_op = match op {
                AssignmentOpType::Eq => "=",
                AssignmentOpType::SubEq => "-=",
                AssignmentOpType::DivEq => "/=",
                AssignmentOpType::AddEq => "+=",
                AssignmentOpType::MulEq => "*=",
                AssignmentOpType::BOrEq => "|=",
                AssignmentOpType::BXorEq => "^=",
                AssignmentOpType::BAndEq => "&=",
            };
            let rust_adr = convert_annotated_ast(&*adr);
            format!("{rust_adr} {rust_op} = ")
        }
        AnnotatedNodeT::Declaration { id, is_mut, t } => {
            let rust_t = match t {
                CType::Void => "()",
                CType::Int => "i32",
                CType::Char => "u8",
            };
            let binding = if *is_mut { " mut " } else { "" };
            format!("let{binding}{id}: {rust_t} = ")
        }
        _ => non_ptr_conversion(root),
    }
}
fn non_ptr_conversion(root: &AnnotatedNode) -> String {
    match root.token {
        AnnotatedNodeT::Add => {
            let children = root
                .children
                .as_ref()
                .expect("Add Node should have children");
            let left = convert_annotated_ast(&children[0]);
            let right = convert_annotated_ast(&children[1]);
            format!("{left} + {right}")
        }
        AnnotatedNodeT::Sub => {
            let children = root
                .children
                .as_ref()
                .expect("Add Node should have children");
            let left = convert_annotated_ast(&children[0]);
            let right = convert_annotated_ast(&children[1]);
            format!("{left} - {right}")
        }
        AnnotatedNodeT::Mul => {
            let children = root
                .children
                .as_ref()
                .expect("Add Node should have children");
            let left = convert_annotated_ast(&children[0]);
            let right = convert_annotated_ast(&children[1]);
            format!("{left} * {right}")
        }
        AnnotatedNodeT::Div => {
            let children = root
                .children
                .as_ref()
                .expect("Add Node should have children");
            let left = convert_annotated_ast(&children[0]);
            let right = convert_annotated_ast(&children[1]);
            format!("{left} / {right}")
        }
        AnnotatedNodeT::Eq => {
            format!("=")
        }
        _ => todo!(),
    }
}
