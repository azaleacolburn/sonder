use crate::{
    annotater::{AnnotatedNode, AnnotatedNodeT},
    data_model::{FieldDefinition, ReferenceType},
};

impl AnnotatedNode {
    pub fn convert(&self) -> String {
        let root = self;
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
                init_value_unused,
            } => {
                let unused = match is_used {
                    true => "",
                    false => "_",
                };
                let rust_t = t.to_rust_type();
                let rust_adr = adr.convert();
                let mut_binding = if *is_mut { "mut " } else { "" };

                let ref_type_iter = &mut ref_type.iter().cloned();
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

                let l_side = if *init_value_unused {
                    "".into()
                } else {
                    format!(" = {rust_reference}")
                };

                format!("let {mut_binding}{unused}{id}: {rust_ref_type}{l_side};")
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
                    .map(Self::convert)
                    .collect::<Vec<String>>()[0]
                    .clone();
                let mut l_side = id.clone();
                let is_rc_clone = ref_types.contains(&ReferenceType::RcRefClone);

                ref_types.iter().for_each(|deref_type| match deref_type {
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
                init_value_unused,
            } => {
                let unused = match is_used {
                    true => "",
                    false => "_",
                };
                let rust_t = t.to_rust_type();
                let expr_children = root
                    .children
                    .iter()
                    .map(Self::convert)
                    .collect::<Vec<String>>();
                if !expr_children.is_empty() {
                    let expr_child = expr_children[0].clone();
                    let l_side = if *init_value_unused {
                        "".into()
                    } else {
                        format!(" = {expr_child}")
                    };

                    if *rc {
                        format!(
                        "let {unused}{id}: Rc<RefCell<{rust_t}>> = Rc::new(RefCell::new({expr_child}));"
                    )
                    } else {
                        let binding = if *is_mut { "mut " } else { "" };
                        format!("let {binding}{unused}{id}: {rust_t}{l_side};")
                    }
                } else {
                    // NOTE:
                    // It should never be a struct field definitions because those are handled
                    // internally by the StructDefinition node
                    format!("let {id}: {rust_t};")
                }
            }
            AnnotatedNodeT::DeRef { id, rc, count } => {
                let derefs: String = (0..*count).fold(String::new(), |mut acc, _| {
                    acc.push('*');
                    acc
                });
                if *rc {
                    format!("{derefs}{id}.borrow()")
                } else {
                    format!("{derefs}{id}")
                }
            }
            AnnotatedNodeT::Adr { id } => {
                id.to_string() // NOTE This isnt' a bug, just cursed
            }
            AnnotatedNodeT::ArrayDeclaration {
                id,
                t,
                size,
                is_used,
                is_mut,
                items,
                init_value_unused,
            } => {
                let rust_t = t.to_rust_type();
                let used = if *is_used { "" } else { "_" };
                let mut_str = if *is_mut { "mut " } else { "" };
                let items_str = items
                    .iter()
                    .map(Self::convert)
                    .collect::<Vec<String>>()
                    .join(", ");

                let l_side = if *init_value_unused {
                    "".into()
                } else {
                    format!(" = &[{size}; {items_str}]")
                };

                format!("let {mut_str}{used}{id}: {rust_t}{l_side};")
            }
            AnnotatedNodeT::NumLiteral(n) => {
                format!("{n}")
            }
            AnnotatedNodeT::Assignment { op, id, rc } => {
                let rust_expr = &root.children[0].convert();

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
                let args = root
                    .children
                    .iter()
                    .take(i32::max(root.children.len() as i32 - 1, 0) as usize)
                    .map(convert_argument)
                    .collect::<Vec<String>>()
                    .join(", ");
                let scope = root
                    .children
                    .last()
                    .unwrap_or(&AnnotatedNode {
                        token: AnnotatedNodeT::Scope(None),
                        children: vec![],
                    })
                    .convert();

                format!("fn {id}({args}) -> {rust_t} {{\n{scope}\n}}")
            }
            AnnotatedNodeT::FunctionCall(id) => {
                let args = root
                    .children
                    .iter()
                    .map(Self::convert)
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("{id}({args})")
            }
            AnnotatedNodeT::If => {
                let condition = root.children[0].convert();
                let scope = root.children[1].convert();

                format!("if {condition} {{\n{scope}\n}}")
            }
            AnnotatedNodeT::Program { imports } => {
                let mut t = imports.clone();
                t.push(
                    root.children
                        .iter()
                        .map(AnnotatedNode::convert)
                        .collect::<Vec<String>>()
                        .join("\n"),
                );

                t.join("\n")
            }
            AnnotatedNodeT::StructDeclaration {
                var_id,
                struct_id,
                is_mut,
                fields,
                is_used,
                init_value_unused,
            } => {
                let unused = match is_used {
                    true => "",
                    false => "_",
                };
                let mut_binding = match is_mut {
                    true => "mut ",
                    false => "",
                };
                let l_side = if *init_value_unused {
                    "".into()
                } else {
                    let mut l_side = format!(" = {struct_id} {{");
                    fields.iter().for_each(|(field, expr)| {
                        l_side.push_str(expr.convert_field_literal(field.clone()).as_str())
                    });
                    l_side.push('}');

                    l_side
                };

                format!("let {mut_binding}{unused}{var_id}{l_side};")
            }
            _ => root.non_ptr_conversion(),
        }
    }

    fn convert_field_literal(&self, field: FieldDefinition) -> String {
        // NOTE If self is a ptr, it must be just a ptr
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
        let mut converted_expr: String = self.convert();
        // let rust_type = field.c_type.to_rust_type();
        if !field.ptr_type.is_empty() {
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

    fn non_ptr_conversion(&self) -> String {
        let root = self;

        let mut left: Option<String> = None;
        let mut right: Option<String> = None;
        if root.children.len() > 1 {
            left = Some(root.children[0].convert());
            right = Some(root.children[1].convert());
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
            AnnotatedNodeT::Eq => "=".to_string(),
            AnnotatedNodeT::EqCmp => {
                format!("{} == {}", left.unwrap(), right.unwrap())
            }
            AnnotatedNodeT::Id { id, rc } => {
                if *rc {
                    format!("*{id}.borrow()")
                } else {
                    id.to_string()
                }
            }
            AnnotatedNodeT::NumLiteral(n) => {
                format!("{n}")
            }
            AnnotatedNodeT::Assignment { op, id, rc } => {
                let rust_expr = root.children[0].convert();

                if *rc {
                    format!("*{id}.borrow_mut() {op} {rust_expr};")
                } else {
                    format!("{id} {op} {rust_expr};")
                }
            }

            AnnotatedNodeT::Program { imports } => {
                let mut t = imports.clone();
                t.push(
                    root.children
                        .iter()
                        .map(Self::convert)
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
                            (ReferenceType::ConstPtr, _) => {
                                field_type = format!("*const {field_type}")
                            }
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
                let rust_expr = expr.convert();
                format!("{var_id}.{field_id} {op} {rust_expr};")
            }
            AnnotatedNodeT::While => {
                let condition = left.unwrap();
                let scope = right.unwrap();

                format!("while {condition} {{\n\t\t{scope}\n\t}}")
            }

            AnnotatedNodeT::Scope(_) => root
                .children
                .iter()
                .map(Self::convert)
                .collect::<Vec<String>>()
                .join("\n\t"),
            AnnotatedNodeT::Return { expr } => {
                let expr = expr.convert();

                format!("return({expr});")
            }
            node => panic!("Unsupported AnnotatedNode: {node:?}"),
        }
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

fn convert_argument(expr: &AnnotatedNode) -> String {
    match &expr.token {
        AnnotatedNodeT::Declaration {
            id,
            is_mut,
            t,
            rc: _,
            is_used,
            init_value_unused: _,
        } => {
            let mut_str = if *is_mut { "mut " } else { "" };
            let _used_str = if *is_used { "_" } else { "" };
            format!("{mut_str}{id}: {}", t.to_rust_type())
        }
        AnnotatedNodeT::PtrDeclaration {
            id,
            is_mut,
            t,
            rc: _,
            is_used,
            points_to: _,
            adr: _,
            ref_type,
            init_value_unused: _,
        } => {
            let mut_str = if *is_mut { "mut " } else { "" };
            let _used_str = if *is_used { "_" } else { "" };
            let ref_type_iter = &mut ref_type.iter().cloned();
            let ptr_type = construct_ptr_type(ref_type_iter, &t.to_rust_type());

            format!("{mut_str}{id}: {ptr_type}")
        }
        node_t => panic!("Unexpected Argument Node Type: {:?}", node_t),
    }
}
