use crate::{
    analyzer::{AnnotatedNode, AnnotatedNodeT, RefType},
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
            let ptr_type = ptr_data
                .ptr_type
                .iter()
                .map(|t| match t {
                    RefType::Mut => "&mut ",
                    RefType::Imut => "&",
                })
                .collect::<Vec<&str>>()
                .join("");
            println!("ptr_type: {ptr_type}");
            let rust_adr = convert_annotated_ast(&adr);
            let ref_type = if ptr_data.mutates { " &mut " } else { " &" };

            let mut_binding = if *is_mut { "mut " } else { "" };
            // Only supports one reference at a time
            format!("let {mut_binding}{id}: {ptr_type}{rust_t} = {ref_type}{rust_adr};")
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
            let expr_child = root
                .children
                .as_ref()
                .expect("deref assignment must have r_side")
                .iter()
                .map(convert_annotated_ast)
                .collect::<Vec<String>>()[0]
                .clone();
            format!("{rust_adr} {rust_op} {expr_child};")
        }
        AnnotatedNodeT::Declaration { id, is_mut, t } => {
            let rust_t = match t {
                CType::Void => "()",
                CType::Int => "i32",
                CType::Char => "u8",
            };
            let binding = if *is_mut { "mut " } else { "" };
            let expr_child = root
                .children
                .as_ref()
                .expect("deref assignment must have r_side")
                .iter()
                .map(convert_annotated_ast)
                .collect::<Vec<String>>()[0]
                .clone();
            format!("let {binding}{id}: {rust_t} = {expr_child};")
        }
        AnnotatedNodeT::DeRef(id) => {
            format!("*{}", convert_annotated_ast(id))
        }
        AnnotatedNodeT::Adr { id } => {
            format!("{id}")
        }
        _ => non_ptr_conversion(root),
    }
}
fn non_ptr_conversion(root: &AnnotatedNode) -> String {
    match &root.token {
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
        AnnotatedNodeT::Id(id) => {
            format!("{id}")
        }
        AnnotatedNodeT::NumLiteral(n) => {
            format!("{n}")
        }
        AnnotatedNodeT::Assignment { op, id } => {
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

            let children = root
                .children
                .as_ref()
                .expect("Add Node should have children");
            let rust_expr = convert_annotated_ast(&children[0]);

            format!("{id} {rust_op} {rust_expr}")
        }
        AnnotatedNodeT::FunctionDecaration { id, t } => {
            let rust_t = match (id.as_str(), t) {
                ("main", _) => "()",
                (_, CType::Void) => "()",
                (_, CType::Int) => "i32",
                (_, CType::Char) => "u8",
            };

            let mut ret = format!("fn {id}() -> {rust_t} {{\n\t");
            root.children
                .as_ref()
                .expect("Function should have children")
                .iter()
                .for_each(|child| {
                    ret.push_str(&convert_annotated_ast(child));
                });
            ret.push_str("\n}");
            ret
        }
        AnnotatedNodeT::Program => root
            .children
            .as_ref()
            .expect("Program should have children")
            .iter()
            .map(convert_annotated_ast)
            .collect::<Vec<String>>()
            .join("\n"),
        AnnotatedNodeT::Scope(_) => root
            .children
            .as_ref()
            .expect("Program should have children")
            .iter()
            .map(convert_annotated_ast)
            .collect::<Vec<String>>()
            .join("\n\t"),
        _ => panic!("Unsupported AnnotatedNode"),
    }
}
