use crate::{
    analyzer::PtrType,
    annotater::{AnnotatedNode, AnnotatedNodeT},
    ast::AssignmentOpType,
    lexer::CType,
};
pub fn convert_annotated_ast(root: &AnnotatedNode) -> String {
    match &root.token {
        AnnotatedNodeT::PtrDeclaration {
            id,
            is_mut,
            adr_data,
            t,
            adr,
            rc: _,
        } => {
            let rust_t = match &t {
                CType::Void => "()",
                CType::Int => "i32",
                CType::Char => "u8",
                CType::Struct(id) => id.as_str(),
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
                    Some(PtrType::RcRefClone) => {
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

            let rust_ref_type =
                get_ref_type(&mut adr_data.borrow().ptr_type.clone().into_iter(), rust_t);
            let rust_reference = match adr_data.borrow().ptr_type[0] {
                PtrType::MutRef => format!("&mut {rust_adr}"),
                PtrType::ImutRef => format!("&{rust_adr}"),
                PtrType::RefCell => format!("RefCell::new({rust_adr}) "),
                PtrType::Rc => format!("Rc::new({rust_adr}) "),
                // .clone() should be handled by the adr call iself
                PtrType::RcRefClone => format!("{rust_adr}"),
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
        AnnotatedNodeT::DerefAssignment {
            op,
            id,
            rc,
            ref_types,
        } => {
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
                .iter()
                .map(convert_annotated_ast)
                .collect::<Vec<String>>()[0]
                .clone();
            let mut l_side = id.clone();
            let is_rc_clone = ref_types.contains(&PtrType::RcRefClone);

            ref_types
                .into_iter()
                .for_each(|deref_type| match deref_type {
                    PtrType::RcRefClone => l_side = format!("{l_side}.borrow_mut()"),
                    PtrType::MutRef if !is_rc_clone => l_side = format!("*{l_side}"),
                    PtrType::MutRef => {}
                    t => panic!(
                        "Invalid Ptr Type being Derefferenced on lside of deref assignment: {:?}",
                        t
                    ),
                });
            if is_rc_clone {
                l_side = format!("*{l_side}");
            }

            format!("{l_side} {rust_op} {expr_child};")
        }
        AnnotatedNodeT::Declaration { id, is_mut, t, rc } => {
            let rust_t = match &t {
                CType::Void => "()",
                CType::Int => "i32",
                CType::Char => "u8",
                CType::Struct(id) => id.as_str(),
            };
            let expr_children = root
                .children
                .iter()
                .map(convert_annotated_ast)
                .collect::<Vec<String>>();
            if expr_children.len() > 0 {
                let expr_child = expr_children[0].clone();

                if *rc {
                    format!(
                        "let {id}: Rc<RefCell<{rust_t}>> = Rc::new(RefCell::new({expr_child}));"
                    )
                } else {
                    let binding = if *is_mut { "mut " } else { "" };
                    format!("let {binding}{id}: {rust_t} = {expr_child};")
                }
            } else {
                // NOTE:
                // It should never be a struct field declaration because those are handled
                // internally by the StructDeclaration node
                format!("let {id}: {rust_t};")
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
    let mut left: Option<String> = None;
    let mut right: Option<String> = None;
    if root.children.len() > 1 {
        left = Some(convert_annotated_ast(&root.children[0]));
        right = Some(convert_annotated_ast(&root.children[1]));
    }
    match &root.token {
        AnnotatedNodeT::Add => {
            format!("{} + {}", left.unwrap(), right.unwrap())
        }
        AnnotatedNodeT::Sub => {
            format!("{} - {}", left.unwrap(), right.unwrap())
        }
        AnnotatedNodeT::Mul => {
            format!("{} * {}", left.unwrap(), right.unwrap())
        }
        AnnotatedNodeT::Div => {
            format!("{} / {}", left.unwrap(), right.unwrap())
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

            let rust_expr = convert_annotated_ast(&root.children[0]);

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
                (_, CType::Struct(id)) => id.as_str(),
            };

            let mut ret = format!("fn {id}() -> {rust_t} {{\n\t");
            root.children.iter().for_each(|child| {
                ret.push_str(&convert_annotated_ast(child));
            });
            ret.push_str("\n}");
            ret
        }
        AnnotatedNodeT::Program { imports } => {
            let mut t = imports.clone();
            println!("imports: {:?}", imports);
            t.push(
                root.children
                    .iter()
                    .map(convert_annotated_ast)
                    .collect::<Vec<String>>()
                    .join("\n"),
            );
            t.join("\n")
        }
        AnnotatedNodeT::StructDeclaration(struct_name, field_definitions) => {
            let mut ret = format!("struct {struct_name} {{\n");
            field_definitions.iter().for_each(|field| {
                let mut field_type = match &field.c_type {
                    CType::Void => "()",
                    CType::Char => "u8",
                    CType::Int => "u16",
                    CType::Struct(id) => id.as_str(),
                }
                .to_string();
                field.ptr_type.iter().rev().for_each(|p| match p {
                    PtrType::MutRef => field_type = format!("&mut {field_type}"),
                    PtrType::ImutRef => field_type = format!("&{field_type}"),
                    PtrType::RcRefClone => field_type = format!("Rc<RefCell<{field_type}>>"),
                    // TODO Check if rc is used for original rc ptrs or if RcRefClone is used
                    // for all
                    PtrType::Rc => field_type = format!("Rc<{field_type}>"),
                    PtrType::RefCell => field_type = format!("RefCell<{field_type}>"),
                    PtrType::RawPtrMut => field_type = format!("*mut {field_type}"),
                    PtrType::RawPtrImut => field_type = format!("*const {field_type}"),
                });
                let field_ret = format!("{}: {}", field.id, field_type);
                ret.push_str(format!("\t{},\n", field_ret).as_str());
            });
            ret.push('}');
            ret
        }
        AnnotatedNodeT::Scope(_) => root
            .children
            .iter()
            .map(convert_annotated_ast)
            .collect::<Vec<String>>()
            .join("\n\t"),
        _ => panic!("Unsupported AnnotatedNode"),
    }
}
