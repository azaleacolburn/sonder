use crate::parser::{AssignmentOpType, NodeType, TokenNode as Node};

struct Handler {
    node: Node,
    code: String,
}

impl Handler {
    pub fn new(node: Node) -> Handler {
        Handler {
            node,
            code: String::new(),
        }
    }

    pub fn push(&mut self, string: &str) {
        self.code.push_str(string);
    }
}

fn program(node: Node) {
    let mut code = String::new();
    let children = node.children.expect("node must hav children");
    children.into_iter().for_each(|child_node| match child_node.token {
        NodeType::FunctionDecaration((ref name, return_type)) => {
            let last = child_node.clone().children.unwrap().last_mut();
            let args: String = child_node.clone()
                .children
                .expect("Node must have children")
                .iter()
                .map(|child| child.token.clone())
                .filter_map(|token| {
                    if let NodeType::Declaration((name, size, allocated_size)) = token {
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

                    let type_size = f(size, allocated_size);
                    Some(format!("{name}: u{type_size}"))
                    } else {
                        panic!("Argument node must be declaration");
                    }
                }).collect::<Vec<String>>().join(", ");
            let scope_code = scope(&child_node.children.as_ref().unwrap().last().unwrap());
            let function_code = format!("fn {name}({args}) -> {return_type} {{{scope_code}}}");
            code.push_str(function_code.as_str());
        }
        _ => {
            panic!("Unrecognized NodeType");
        }
    })
}

fn scope(scope_node: &Node) -> String {
    let scope_children = scope_node.children.as_ref().unwrap();
    scope_children.iter().fold(String::new(), |mut code, node| {
        code.push_str(statement(&node).as_str());
        code
    })
}

fn statement(node: &Node) -> String {
    match &node.token {
        NodeType::Declaration((name, size, allocated_size)) => declaration(&node, name.to_string(), *size, *allocated_size)
        NodeType::Assignment(op_type) => assignment(&node, op_type),

        _ => panic!("Unrecognized statement node"),
    }
}

fn declaration(node: &Node, name: String, size: usize, allocated_size: usize) -> String {

    
    todo!()
}

fn assignment(node: &Node, op_type: AssignmentOpType) -> String {
    let op_type = if let NodeType::Assignment(op_type) = node.token {
    op_type
} ;
    let children = node.children.as_ref();
    let id_token = children[0];
    let expression = children[1];

    let code = format!("{id_token} ")

    todo!()
}
