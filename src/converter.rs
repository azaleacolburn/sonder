use crate::{
    annotater::{AnnotatedNode, AnnotatedNodeT},
    data_model::ReferenceType,
    lexer::CType,
};
pub fn convert_annotated_ast(root: &AnnotatedNode) -> String {
    match &root.token {
        AnnotatedNodeT::PtrDeclaration {
            id,
            is_mut,
            points_to,
            ref_type,
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

            let ref_type_iter = &mut ref_type.into_iter().cloned();
            let rust_ref_type = construct_ptr_type(ref_type_iter, rust_t);

            let rust_reference = match points_to[0].borrow().get_reference_type() {
                ReferenceType::MutBorrowed => format!("&mut {rust_adr}"),
                ReferenceType::ConstBorrowed => format!("&{rust_adr}"),
                ReferenceType::RcRefClone => format!("{rust_adr}"),
                ReferenceType::MutPtr => format!("&mut {rust_adr} as {rust_ref_type}"),
                ReferenceType::ConstPtr => {
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
            rc: _, // TODO Create a clearer distinction between rc variables and rc pointers
            ref_types,
        } => {
            let expr_child = root
                .children
                .iter()
                .map(convert_annotated_ast)
                .collect::<Vec<String>>()[0]
                .clone();
            let mut l_side = id.clone();
            let is_rc_clone = ref_types.contains(&ReferenceType::RcRefClone);

            println!("HI");

            ref_types
                .into_iter()
                .inspect(|t| println!("{:?}", t))
                .for_each(|deref_type| match deref_type {
                    ReferenceType::RcRefClone => l_side = format!("{l_side}.borrow_mut()"),
                    ReferenceType::MutBorrowed if !is_rc_clone => l_side = format!("*{l_side}"),
                    ReferenceType::MutBorrowed => {
                        println!("DEREFFED PTR BOTH MutBorrowed and is_rc_clone");
                    }
                    t => panic!(
                        "Invalid Ptr Type being Derefferenced on lside of deref assignment: {:?}",
                        t
                    ),
                });
            if is_rc_clone {
                l_side = format!("*{l_side}");
            }

            format!("{l_side} {op} {expr_child};")
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
                // It should never be a struct field definitions because those are handled
                // internally by the StructDefinition node
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
            let rust_expr = convert_annotated_ast(&root.children[0]);

            if *rc {
                format!("*{id}.borrow_mut() {op} {rust_expr};")
            } else {
                format!("{id} {op} {rust_expr};")
            }
        }
        AnnotatedNodeT::FunctionDeclaration { id, t } => {
            let rust_t = match (id.as_str(), t) {
                ("main", _) => "()",
                (_, CType::Void) => "()",
                (_, CType::Int) => "i32",
                (_, CType::Char) => "u8",
                (_, CType::Struct(id)) => id.as_str(),
            };

            let mut ret = format!("fn {id}() -> {rust_t} {{\n");
            root.children.iter().for_each(|child| {
                ret.push_str(format!("\t{}", &convert_annotated_ast(child)).as_str());
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
        AnnotatedNodeT::StructDefinition {
            struct_id,
            field_definitions,
        } => {
            let mut ret = format!("struct {struct_id} {{\n");
            field_definitions.iter().for_each(|field| {
                let mut field_type = match &field.c_type {
                    CType::Void => "()",
                    CType::Char => "u8",
                    CType::Int => "u16",
                    CType::Struct(id) => id.as_str(),
                }
                .to_string();
                field.ptr_type.iter().rev().for_each(|p| match p {
                    ReferenceType::MutBorrowed => field_type = format!("&mut {field_type}"),
                    ReferenceType::ConstBorrowed => field_type = format!("&{field_type}"),
                    ReferenceType::RcRefClone => field_type = format!("Rc<RefCell<{field_type}>>"),
                    // TODO Check if rc is used for original rc ptrs or if RcRefClone is used
                    // for all
                    ReferenceType::MutPtr => field_type = format!("*mut {field_type}"),
                    ReferenceType::ConstPtr => field_type = format!("*const {field_type}"),
                });
                let field_ret = format!("{}: {}", field.id, field_type);
                ret.push_str(format!("\t{},\n", field_ret).as_str());
            });
            ret.push('}');
            ret
        }
        AnnotatedNodeT::StructDeclaration {
            var_id,
            struct_id,
            is_mut,
            fields,
        } => {
            let mut_binding = match is_mut {
                true => "mut",
                false => "",
            };
            let mut ret = format!("let {mut_binding} {var_id} = {struct_id} {{ ");
            fields.iter().for_each(|(field, expr)| {
                // NOTE: All the other fancy field stuff should be handled by expr
                let expr = convert_annotated_ast(expr);
                ret.push_str(format!("{}: {}, ", field.id, expr).as_str());
            });
            ret.push_str("};");
            ret
        }
        AnnotatedNodeT::StructFieldAssignment {
            var_id,
            field_id,
            op,
            expr,
        } => {
            let rust_expr = convert_annotated_ast(expr);
            format!("{var_id}.{field_id} {op} {rust_expr};")
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

fn construct_ptr_type<T>(points_to: &mut T, rust_t: &str) -> String
where
    T: Iterator<Item = ReferenceType>,
{
    match points_to.next() {
        Some(ReferenceType::MutBorrowed) => {
            format!("&mut {} ", construct_ptr_type(points_to, rust_t))
        }
        Some(ReferenceType::ConstBorrowed) => {
            format!("&{}", construct_ptr_type(points_to, rust_t))
        }
        Some(ReferenceType::RcRefClone) => {
            format!("Rc<RefCell<{}>>", construct_ptr_type(points_to, rust_t))
        }
        Some(ReferenceType::MutPtr) => format!("*mut {}", construct_ptr_type(points_to, rust_t)),
        Some(ReferenceType::ConstPtr) => {
            format!("*const {}", construct_ptr_type(points_to, rust_t))
        }
        None => rust_t.to_string(),
    }
}
