use crate::parser::{NodeType, TokenNode as Node};

fn program(ast: Node) {
    let children = ast.children.expect("node must hav children");
    children.iter().for_each(|node| match &node.token {
        NodeType::FunctionDecaration((name, return_type)) => {
            let args: String = node
                .children.clone()
                .expect("Node must have children")
                .iter()
                .map(|child| child.token.clone())
                .filter_map(|token| {
                    if let NodeType::Declaration((name, size, allocated_size)) = token {
                        Some((name, size, allocated_size))
                    } else {
                        panic!("Argument node must be declaration");
                    }
                })
                .map(|(name, size, allocated_size)| {
                    fn f(size: usize, allocated_size: usize) -> String {
                        match size {
                            1 => "u8".to_string(),
                            4 => "u16".to_string(),
                            8 => {
                                let al = f(allocated_size, 0);
                                format!("&{al}")
                            }
                            0 => panic!("Multiple references not yet supported"),
                            _ => panic!("Unsupport variable size; we need to switch to unsupported types anyways")
                        }
                    }

                    let size = f(size, allocated_size);
                    format!("{name}:")
                }).collect::<Vec<String>>().join(", ");
            let scope_code = scope(node.children.unwrap());
            let code = format!("fn {name}({args}) -> {return_type} {{{scope_code}}}");
        }
        _ => {
            panic!("Unrecognized NodeType");
        }
    })
}

fn scope(scope_children: Vec<Node>) -> String {
    scope_children.iter().fold(String::new(), |mut code, node| {
        let children = node.children.expect("Must have children");
        code.push_str(
            match &node.token {
                NodeType::Declaration((name, size, allocated_size)) => {
                    declaration(children[0], *name, *size, *allocated_size)
                }
                _ => panic!("Unrecognized statement node"),
            }
            .as_str(),
        );
        code
    })
}

fn declaration(assignment_node: Node, name: String, size: usize, allocated_size: usize) -> String {
    todo!()
}
