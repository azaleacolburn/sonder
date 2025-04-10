use crate::{
    annotater::{AnnotatedNode, AnnotatedNodeT},
    data_model::{FieldDefinition, ReferenceType},
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
            is_used,
        } => {
            let unused = match is_used {
                true => "",
                false => "_",
            };
            let rust_t = t.to_rust_type();
            let rust_adr = convert_annotated_ast(&adr);
            let mut_binding = if *is_mut { "mut " } else { "" };

            let ref_type_iter = &mut ref_type.into_iter().cloned();
            let rust_ref_type = construct_ptr_type(ref_type_iter, &rust_t);

            let rust_reference = match points_to[0].borrow().get_reference_type() {
                ReferenceType::MutBorrowed => format!("&mut {rust_adr}"),
                ReferenceType::ConstBorrowed => format!("&{rust_adr}"),
                ReferenceType::RcRefClone => format!("{rust_adr}.clone()"),
                ReferenceType::MutPtr => format!("&mut {rust_adr} as {rust_ref_type}"),
                ReferenceType::ConstPtr => {
                    format!("&{rust_adr} as {rust_ref_type}")
                }
            };

            format!("let {mut_binding}{unused}{id}: {rust_ref_type} = {rust_reference};")
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
            let mut expr_child = root
                .children
                .iter()
                .map(convert_annotated_ast)
                .collect::<Vec<String>>()[0]
                .clone();
            let mut l_side = id.clone();
            let is_rc_clone = ref_types.contains(&ReferenceType::RcRefClone);

            ref_types
                .into_iter()
                .inspect(|t| println!("{:?}", t))
                .for_each(|deref_type| match deref_type {
                    ReferenceType::RcRefClone => l_side = format!("{l_side}.borrow_mut()"),
                    ReferenceType::MutBorrowed if !is_rc_clone => l_side = format!("*{l_side}"),
                    ReferenceType::MutBorrowed => {
                        println!("DEREFFED PTR BOTH MutBorrowed and is_rc_clone");
                    }
                    ReferenceType::MutPtr => {
                        l_side = format!("unsafe {{ *{l_side}");
                        expr_child.push_str(" }");
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
        AnnotatedNodeT::Declaration {
            id,
            is_mut,
            t,
            rc,
            is_used,
        } => {
            let unused = match is_used {
                true => "",
                false => "_",
            };
            let rust_t = t.to_rust_type();
            let expr_children = root
                .children
                .iter()
                .map(convert_annotated_ast)
                .collect::<Vec<String>>();
            if expr_children.len() > 0 {
                let expr_child = expr_children[0].clone();

                if *rc {
                    format!(
                        "let {unused}{id}: Rc<RefCell<{rust_t}>> = Rc::new(RefCell::new({expr_child}));"
                    )
                } else {
                    let binding = if *is_mut { "mut " } else { "" };
                    format!("let {binding}{unused}{id}: {rust_t} = {expr_child};")
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
        AnnotatedNodeT::Adr { id } => {
            format!("{id}") // NOTE This isnt' a bug, just cursed
        }
        AnnotatedNodeT::ArrayDeclaration {
            id,
            t,
            size,
            is_used,
            is_mut,
            items,
        } => {
            let rust_t = t.to_rust_type();
            let used = if *is_used { "" } else { "_" };
            let mut_str = if *is_mut { "mut " } else { "" };
            let items_str = items
                .iter()
                .map(convert_annotated_ast)
                .collect::<Vec<String>>()
                .join(", ");

            format!("let {mut_str}{used}{id}: {rust_t} = &[{size}; {items_str}]")
        }
        AnnotatedNodeT::StructDeclaration {
            var_id,
            struct_id,
            is_mut,
            fields,
            is_used,
        } => {
            let unused = match is_used {
                true => "",
                false => "_",
            };
            let mut_binding = match is_mut {
                true => "mut ",
                false => "",
            };
            let mut ret = format!("let {mut_binding}{unused}{var_id} = {struct_id} {{ ");
            fields.iter().for_each(|(field, expr)| {
                ret.push_str(&convert_field_literal(expr, field.clone()))
            });
            ret.push_str("};");
            ret
        }
        _ => non_ptr_conversion(root),
    }
}

fn convert_field_literal(expr: &AnnotatedNode, field: FieldDefinition) -> String {
    // NOTE If expr is a ptr, it must be just a ptr
    // i don't have the mental sauce right now for transpiling stuff like this
    // ```c
    // int t = 0;
    // int g = &t (intptr_t) + 1;
    // ```
    // to this
    // ```c
    // let t: i32 = 0;
    // let g: *const i32 = &t as *const i32 + 1;
    // TODO Write ptr transpilation
    //
    // # Ptr Transpilation
    // Because pointer arithmatic in rust is done by method, not arithmetic symbols, we need a
    // totally separate system for converting expression that involve raw ptrs, meaning we
    // shouldn't worry about them for now
    let mut converted_expr: String = convert_annotated_ast(expr);
    // let rust_type = field.c_type.to_rust_type();
    if field.ptr_type.len() > 0 {
        // NOTE If it's a ptr, only one factor, an adr
        // The reference taking is handled by the statement node
        // let reference_type = construct_ptr_type(&mut field.ptr_type.into_iter(), &rust_type);
        converted_expr = match field.ptr_type[0] {
            ReferenceType::MutBorrowed => format!("&mut {converted_expr}"),
            ReferenceType::ConstBorrowed => format!("&{converted_expr}"),
            ReferenceType::RcRefClone => format!("{converted_expr}.clone()"),
            // ReferenceType::MutPtr => format!("&mut {converted_expr} as {reference_type}"),
            // ReferenceType::ConstPtr => {
            //     format!("&{converted_expr} as {reference_type}")
            // }
            _ => panic!("Not supporting raw ptrs yet"),
        };
    }

    format!("{}: {},", field.id, converted_expr)
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
            let rust_t = match id == "main" {
                true => "()".into(),
                false => t.to_rust_type(),
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
            has_ref,
        } => {
            let lifetime = match has_ref {
                true => "<'a>",
                false => "",
            };
            let mut ret = format!("struct {struct_id}{lifetime} {{\n");
            field_definitions.iter().for_each(|field| {
                let mut field_type = field.c_type.to_rust_type();

                field
                    .ptr_type
                    .iter()
                    .rev()
                    .for_each(|p| match (p, has_ref) {
                        (ReferenceType::MutBorrowed, true) => {
                            field_type = format!("&'a mut {field_type}")
                        }
                        (ReferenceType::ConstBorrowed, true) => {
                            field_type = format!("&'a {field_type}")
                        }

                        (ReferenceType::MutBorrowed, false) => {
                            field_type = format!("&mut {field_type}")
                        }
                        (ReferenceType::ConstBorrowed, false) => {
                            field_type = format!("&{field_type}")
                        }
                        (ReferenceType::RcRefClone, _) => {
                            field_type = format!("Rc<RefCell<{field_type}>>")
                        }
                        // TODO Check if rc is used for original rc ptrs or if RcRefClone is used
                        // for all
                        (ReferenceType::MutPtr, _) => field_type = format!("*mut {field_type}"),
                        (ReferenceType::ConstPtr, _) => field_type = format!("*const {field_type}"),
                    });
                let field_ret = format!("{}: {}", field.id, field_type);
                ret.push_str(format!("\t{},\n", field_ret).as_str());
            });
            ret.push('}');
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
