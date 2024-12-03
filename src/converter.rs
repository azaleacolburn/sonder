use crate::{
    analyzer::PtrType,
    annotater::{AnnotatedNode, AnnotatedNodeT},
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
            rc,
        } => {
            let rust_t = match t {
                CType::Void => "()",
                CType::Int => "i32",
                CType::Char => "u8",
            };
            let rust_adr = convert_annotated_ast(&adr);
            let mut_binding =
                if *is_mut && !(*rc || *ptr_data.ptr_type.last().unwrap() == PtrType::RcClone) {
                    "mut "
                } else {
                    ""
                };
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
                    Some(PtrType::RcClone) => {
                        format!("Rc<RefCell<{}>>", get_ref_type(ptr_types, rust_t))
                    }
                    Some(PtrType::RefCell) => {
                        format!("RefCell<{}>", get_ref_type(ptr_types, rust_t))
                    }
                    Some(PtrType::Rc) => format!("Rc<{}>", get_ref_type(ptr_types, rust_t)),
                    Some(PtrType::RawPtrMut) => format!("*mut {}", get_ref_type(ptr_types, rust_t)),
                    Some(PtrType::RawPtrImut) => {
                        format!("*const {}", get_ref_type(ptr_types, rust_t))
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
                // .clone() should be handled by the adr call iself
                PtrType::RcClone => format!("{rust_adr}"),
                PtrType::RawPtrMut => format!("&mut {rust_adr} as {rust_ref_type}"),
                PtrType::RawPtrImut => {
                    format!("&{rust_adr} as {rust_ref_type}")
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
        AnnotatedNodeT::DerefAssignment { op, id, rc, count } => {
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
            let expr_child = root
                .children
                .as_ref()
                .expect("deref assignment must have r_side")
                .iter()
                .map(convert_annotated_ast)
                .collect::<Vec<String>>()[0]
                .clone();
            let derefs: String =
                (0..count.clone())
                    .into_iter()
                    .fold(String::new(), |mut acc, _| {
                        acc.push_str("*");
                        acc
                    });

            if *rc {
                format!("{derefs}{id}.borrow_mut() {rust_op} {expr_child};")
            } else {
                format!("{derefs}{id} {rust_op} {expr_child};")
            }
        }
        AnnotatedNodeT::Declaration { id, is_mut, t, rc } => {
            let rust_t = match t {
                CType::Void => "()",
                CType::Int => "i32",
                CType::Char => "u8",
            };
            let expr_child = root
                .children
                .as_ref()
                .expect("deref assignment must have r_side")
                .iter()
                .map(convert_annotated_ast)
                .collect::<Vec<String>>()[0]
                .clone();

            if *rc {
                format!("let {id}: Rc<RefCell<{rust_t}>> = Rc::new(RefCell::new({expr_child}));")
            } else {
                let binding = if *is_mut { "mut " } else { "" };
                format!("let {binding}{id}: {rust_t} = {expr_child};")
            }
        }
        AnnotatedNodeT::DeRef { id, rc, count } => {
            let derefs: String =
                (0..count.clone())
                    .into_iter()
                    .fold(String::new(), |mut acc, _| {
                        acc.push_str("*");
                        acc
                    });
            if *rc {
                format!("{derefs}{id}.borrow()")
            } else {
                format!("{derefs}{id}")
            }
        }
        AnnotatedNodeT::Adr { id, rc } => {
            if *rc {
                format!("{id}.clone()")
            } else {
                format!("{id}")
            }
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
        AnnotatedNodeT::Id { id, rc } => {
            if *rc {
                format!("*{id}.borrow()")
            } else {
                format!("{id}")
            }
        }
        AnnotatedNodeT::NumLiteral(n) => {
            format!("{n}")
        }
        AnnotatedNodeT::Assignment { op, id, rc } => {
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

            if *rc {
                format!("*{id}.borrow_mut() {rust_op} {rust_expr};")
            } else {
                format!("{id} {rust_op} {rust_expr};")
            }
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
        AnnotatedNodeT::Program { imports } => {
            let mut t = imports.clone();
            t.push(
                root.children
                    .as_ref()
                    .expect("Program should have children")
                    .iter()
                    .map(convert_annotated_ast)
                    .collect::<Vec<String>>()
                    .join("\n"),
            );
            t.join("\n")
        }
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
