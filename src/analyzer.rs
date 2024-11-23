use std::{collections::HashMap, fmt::Display};

use crate::{
    lexer::CType,
    parser::{AssignmentOpType, NodeType, TokenNode as Node},
};

#[derive(Debug, Clone)]
enum PtrType {
    TrueRaw,

    ConstPtrConst,
    ConstPtrMut,
    MutPtrConst,
    MutPtrMut,

    ConstRef,
    MutRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtrData {
    points_to: String,
    mutates: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarData<'a> {
    ptr_data: Option<PtrData>,
    pointed_to_by: Vec<&'a str>,
    is_mut_by_ptr: bool,
    is_mut_direct: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotatedNode {
    token: AnnotatedNodeT,
    children: Option<Vec<AnnotatedNode>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotatedNodeT {
    Program,
    Sub,
    Div,
    Eq,
    Id(String), // figure out if we want this here
    EqCmp,
    NeqCmp,
    BOr,
    BAnd,
    BXor,
    BOrEq,
    BAndEq,
    BXorEq,
    SubEq,
    AddEq,
    DivEq,
    MulEq,
    Mul,
    MNeg,
    AndCmp,
    OrCmp,
    NumLiteral(usize),
    Add,
    If,
    For,
    While,
    _Loop,
    Break,
    FunctionCall(String),
    Scope(Option<CType>), // <-- anything that has {} is a scope, scope is how we're handling multiple statements, scopes return the last statement's result or void
    Assignment {
        op: AssignmentOpType,
        id: String,
    },
    DerefAssignment {
        op: AssignmentOpType,
        adr: Box<AnnotatedNode>,
    },
    Declaration {
        id: String,
        is_mut: bool,
        t: CType,
    },
    PtrDeclaration {
        id: String,
        is_mut: bool,
        mutates: bool,
        t: CType,
        adr: Box<AnnotatedNode>,
    },
    RefDeclaration {
        id: String,
        is_mut: bool,
        t: CType,
        adr: Box<AnnotatedNode>,
    },
    Asm(String),
    Adr {
        id: String,
        is_mut: bool,
    },
    Ref {
        id: String,
        is_mut: bool,
    },
    DeRef(Box<AnnotatedNode>),
    ArrayDeclaration {
        id: String,
        t: CType,
        size: usize,
    },
    FunctionDecaration {
        id: String,
        t: CType,
    },
    Type(CType),
    Assert,
    Return,
    PutChar,
    StructDeclaration(String),
}
impl Display for AnnotatedNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.token) // doesn't print values
    }
}

impl AnnotatedNode {
    pub fn print(&self, n: &mut i32) {
        (0..*n).into_iter().for_each(|_| print!("  "));
        println!("{}", self);
        *n += 1;
        if let Some(children) = &self.children {
            children.iter().for_each(|node| {
                node.print(n);
            })
        }
        *n -= 1;
        // println!("End Children");
    }
}
pub fn annotate_ast<'a>(root: &'a Node, var_info: &HashMap<String, VarData<'a>>) -> AnnotatedNode {
    let children = root.children.as_ref().unwrap_or(&vec![]).to_vec();
    let annotated_node_children = Some(
        children
            .iter()
            .map(|node| annotate_ast(node, var_info))
            .collect(),
    );
    let token = match &root.token {
        NodeType::Declaration(id, t, _) => {
            let declaration_info = var_info.get(id).expect("Declared id not in map");
            let is_mut = declaration_info.is_mut_by_ptr || declaration_info.is_mut_direct;
            AnnotatedNodeT::Declaration {
                id: id.to_string(),
                is_mut,
                t: t.clone(),
            }
        }
        NodeType::PtrDeclaration(id, t, adr) => {
            let ptr_info = var_info.get(id).expect("Ptr not found in info map");
            let is_mut = ptr_info.is_mut_by_ptr || ptr_info.is_mut_direct;
            let annotated_adr = Box::new(annotate_ast(adr, var_info));
            AnnotatedNodeT::PtrDeclaration {
                id: id.to_string(),
                is_mut,
                t: t.clone(),
                adr: annotated_adr,
            }
        }
        NodeType::Adr(id) => {
            let adr_info = var_info.get(id).expect("Adr to undeclared variable");
            // TODO: This doesn't mean that it's modified by *this* node
            let is_mut = adr_info.is_mut_by_ptr;
            AnnotatedNodeT::Adr {
                id: id.to_string(),
                is_mut,
            }
        }
        NodeType::DerefAssignment(op, adr) => {
            let annotated_adr = Box::new(annotate_ast(adr, var_info));
            AnnotatedNodeT::DerefAssignment {
                op: op.clone(),
                adr: annotated_adr,
            }
        }
        NodeType::DeRef(expr) => {
            let annotated_expr = Box::new(annotate_ast(expr, var_info));
            AnnotatedNodeT::DeRef(annotated_expr)
        }
        node => node.to_annotated_node(),
    };
    AnnotatedNode {
        token,
        children: annotated_node_children,
    }
}

pub fn determine_var_mutability<'a>(
    root: &'a Node,
    prev_vars: &HashMap<String, VarData<'a>>,
) -> HashMap<String, VarData<'a>> {
    let mut vars: HashMap<String, VarData> = prev_vars.clone();
    if root.children.is_some() {
        root.children.as_ref().unwrap().iter().for_each(|node| {
            determine_var_mutability(node, &vars)
                .iter()
                .for_each(|(id, var_data)| {
                    vars.insert(id.to_string(), var_data.clone());
                });
        })
    }

    match &root.token {
        NodeType::Declaration(id, _, _) => {
            println!("Declaration: {id}");
            vars.insert(
                id.to_string(),
                VarData {
                    ptr_data: None,
                    pointed_to_by: vec![],
                    is_mut_by_ptr: false,
                    is_mut_direct: false,
                },
            );
        }
        NodeType::Assignment(_, id) => {
            vars.get_mut(id).expect("Undeclared Id").is_mut_direct = true;
        }
        NodeType::PtrDeclaration(id, _, expr) => {
            let expr_ids = find_ids(expr);
            if expr_ids.len() > 1 {
                panic!("More than one id in expr: `&(a + b)` not legal");
            } else if expr_ids.len() != 1 {
                panic!("ptr to no id");
            }
            let ptr_data = Some(PtrData {
                points_to: expr_ids[0].clone(),
                mutates: false,
            });
            let var = VarData {
                ptr_data,
                pointed_to_by: vec![],
                is_mut_by_ptr: false,
                is_mut_direct: false,
            };
            let expr_ids = find_ids(expr);
            if expr_ids.len() > 0 {
                vars.insert(id.to_string(), var);
                // Doesn't support &that + &this
                // This immediantly breakes borrow checking rules
                println!("{:?}", vars);
                println!("expr_id: {}", expr_ids[0]);
                vars.get_mut(&expr_ids[0])
                    .expect("Undeclared Id")
                    .pointed_to_by
                    .push(id);
            }
        }
        NodeType::DerefAssignment(_, l_side) => {
            let deref_ids = find_ids(&l_side);
            println!("{:?}", deref_ids);
            // This breakes because `*(t + s) = bar` is not allowed
            // Actually, it shouldn't, because **m is fine
            if deref_ids.len() > 1 {
                panic!("Unsupported: Multiple items dereferenced");
            } else if deref_ids.len() != 1 {
                panic!("Unsupported: no_ids being dereffed")
            }

            for ptr_id in deref_ids {
                vars.get_mut(ptr_id.as_str())
                    .as_mut()
                    .expect("Deref Assigning Undeclared Pointer")
                    .ptr_data
                    .as_mut()
                    // This panics if `*(t + non_ptr)`
                    .expect("Deref assigning non-ptr")
                    .mutates = true;

                let mutated_var_name: Vec<String> = vars
                    .iter()
                    .filter(|(_name, data)| data.pointed_to_by.contains(&ptr_id.as_str()))
                    .map(|(name, _data)| name.clone())
                    .collect();

                vars.get_mut(&mutated_var_name[0])
                    .expect("Undeclared Id Being Dereferenced")
                    .is_mut_by_ptr = true;
            }
        }
        _ => {}
    };
    vars
}

fn find_ids<'a>(root: &'a Node) -> Vec<String> {
    println!("{root}");
    let mut ids: Vec<String> = root
        .children
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .flat_map(|child| find_ids(child))
        .collect();
    match &root.token {
        NodeType::Id(id) => ids.push(id.to_string()),
        NodeType::Adr(id) => ids.push(id.to_string()),
        NodeType::DeRef(node) => ids.append(&mut find_ids(&*node)),
        _ => {}
    }

    ids
}
