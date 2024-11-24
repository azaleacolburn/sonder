use std::{collections::HashMap, fmt::Display};

use crate::{
    lexer::CType,
    parser::{AssignmentOpType, NodeType, TokenNode as Node},
};

#[derive(Debug, Clone)]
enum PtrType {
    ConstPtrConst,
    ConstPtrMut,
    MutPtrConst,
    MutPtrMut,

    ConstRefConst,
    ConstRefMut,
    MutRefConst,
    MutRefMut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtrData {
    pub points_to: String,
    pub mutates: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarData<'a> {
    pub ptr_data: Option<PtrData>,
    pub pointed_to_by: Vec<&'a str>,
    pub is_mut_by_ptr: bool,
    pub is_mut_direct: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotatedNode {
    pub token: AnnotatedNodeT,
    pub children: Option<Vec<AnnotatedNode>>,
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
        ptr_data: PtrData,
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
            let ptr_data = ptr_info
                .ptr_data
                .clone()
                .expect("Declared Ptr not in info map");
            // Decide if we want an enum or two bools
            let _ptr_type = match (is_mut, ptr_data.mutates) {
                (true, true) => PtrType::MutPtrMut,
                (true, false) => PtrType::MutPtrConst,
                (false, true) => PtrType::ConstPtrMut,
                (false, false) => PtrType::ConstPtrMut,
            };
            AnnotatedNodeT::PtrDeclaration {
                id: id.to_string(),
                is_mut,
                ptr_data,
                t: t.clone(),
                adr: annotated_adr,
            }
        }
        NodeType::Adr(id) => {
            let adr_info = var_info.get(id).expect("Adr to undeclared variable");
            let is_mut = ;
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
            // TODO: Figure out how to annotate specific address call as mutable or immutable
            if expr_ids.len() == 1 {
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
            println!("deref_ids: {:?}", deref_ids);
            // This breakes because `*(t + s) = bar` is not allowed
            // However, **m is fine
            if deref_ids.len() > 1 {
                panic!("Unsupported: Multiple items dereferenced");
            } else if deref_ids.len() != 1 {
                panic!("Unsupported: no_ids being dereffed")
            }
            let num_of_vars = count_derefs(&l_side) + 1;
            let mut ptr_chain = traverse_pointer_chain(&deref_ids[0], &vars, 0, num_of_vars)
                .into_iter()
                .rev();
            println!("ptr_chain: {:?}", ptr_chain);
            // [m, p, n]
            let first_ptr = ptr_chain.next().expect("No pointers in chain");
            vars.entry(first_ptr).and_modify(|var_data| {
                var_data.is_mut_direct = true;
                var_data
                    .ptr_data
                    .as_mut()
                    .expect("First ptr in deref not ptr")
                    .mutates = true;
            });

            // TODO: Figure out if we can move ptr_chain here
            ptr_chain.clone().enumerate().for_each(|(i, var)| {
                if i != ptr_chain.len() - 1 {
                    vars.entry(var.clone()).and_modify(|var_data| {
                        var_data
                            .ptr_data
                            .as_mut()
                            .expect("Non-last in chain not ptr")
                            .mutates = true;
                    });
                }
                vars.entry(var).and_modify(|var_data| {
                    var_data.is_mut_by_ptr = true;
                });
            })
        }
        _ => {}
    };
    vars
}

fn count_derefs(root: &Node) -> u8 {
    let mut count = 0;
    let children = root.children.as_ref();
    if let Some(children) = children {
        count += children.iter().map(count_derefs).sum::<u8>();
    }
    match &root.token {
        NodeType::DeRef(expr) => {
            count += count_derefs(&expr) + 1;
        }
        _ => {}
    };
    count
}

fn traverse_pointer_chain<'a>(
    root: &'a str,
    var_info: &HashMap<String, VarData<'a>>,
    total_depth: u8,
    max_depth: u8,
) -> Vec<String> {
    if total_depth == max_depth {
        return vec![];
    }
    let is_ptr = &var_info
        .get(root)
        .as_ref()
        .expect("Root not found in map")
        .ptr_data;
    match is_ptr {
        Some(ref ptr_data) => {
            let mut vec =
                traverse_pointer_chain(&ptr_data.points_to, var_info, total_depth + 1, max_depth);
            vec.push(root.to_string());
            vec
        }
        None => {
            println!("should be n {root}");
            vec![root.to_string()]
        }
    }
}

fn find_ids<'a>(root: &'a Node) -> Vec<String> {
    println!("find_ids root: {root}");
    let mut ids: Vec<String> = root
        .children
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .flat_map(find_ids)
        .collect();
    match &root.token {
        NodeType::Id(id) => ids.push(id.to_string()),
        NodeType::Adr(id) => ids.push(id.to_string()),
        NodeType::DeRef(node) => {
            ids.append(&mut find_ids(&*node));
        }
        _ => {}
    }

    ids
}
