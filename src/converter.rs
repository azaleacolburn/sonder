use crate::{
    analyzer::{AnnotatedNode, AnnotatedNodeT, PtrType},
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
            let rust_adr = convert_annotated_ast(&adr);
            let mut_binding = if *is_mut { "mut " } else { "" };
            // Only supports one reference at a time
            fn get_ref_type<T>(ptr_types: &mut T, rust_t: &str) -> String
            where
                T: Iterator<Item = PtrType>,
            {
                match ptr_types.next() {
                    Some(PtrType::MutRef) => {
                        format!("&mut {} ", get_ref_type(ptr_types, rust_t))
                    }
                    Some(PtrType::ImutRef) => {
                        format!("&{}", get_ref_type(ptr_types, rust_t))
                    }
                    Some(PtrType::RefCell) => {
                        format!("RefCell<{}>", get_ref_type(ptr_types, rust_t))
                    }
                    Some(PtrType::Rc) => format!("Rc<{}>", get_ref_type(ptr_types, rust_t)),
                    Some(PtrType::RawPtrMut) => format!("*mut {rust_t} "),
                    Some(PtrType::RawPtrImut) => {
                        format!("*const {rust_t} ")
                    }
                    None => rust_t.to_string(),
                }
            }

            let rust_ref_type = get_ref_type(&mut ptr_data.ptr_type.clone().into_iter(), rust_t);
            let rust_reference = match ptr_data.ptr_type[0] {
                PtrType::MutRef => format!("&mut {rust_adr}"),
                PtrType::ImutRef => format!("&{rust_adr}"),
                PtrType::RefCell => format!("RefCell::new({rust_adr}) "),
                PtrType::Rc => format!("Rc::new({rust_adr}) "),
                PtrType::RawPtrMut => format!("&mut {rust_adr} as *mut {rust_ref_type}"),
                PtrType::RawPtrImut => {
                    format!("&{rust_adr} as *const {rust_ref_type}")
                }
            };

            format!("let {mut_binding}{id}: {rust_ref_type} = {rust_reference};")
        }
        // = &mut {rust_adr}
        // = &{rust_adr}
        // = RefCell::new({rust_adr})
        // = Rc::new({rust_adr})
        // = &{rust_adr} as *mut {rust_t}
        // = &{rust_adr} as *const {rust_t}
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
