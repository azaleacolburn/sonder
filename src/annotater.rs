use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

use crate::{
    analyzer::{count_derefs, find_ids, AdrData, AnalysisContext, PtrType, VarData},
    lexer::CType,
    parser::{AssignmentOpType, NodeType, TokenNode as Node},
};

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotatedNode<'a> {
    pub token: AnnotatedNodeT<'a>,
    pub children: Option<Vec<AnnotatedNode<'a>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotatedNodeT<'a> {
    Program {
        imports: Vec<String>,
    },
    Sub,
    Div,
    Eq,
    Id {
        id: String,
        rc: bool,
    }, // figure out if we want this here
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
    _MNeg,
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
        rc: bool,
    },
    DerefAssignment {
        op: AssignmentOpType,
        id: String,
        rc: bool,
        count: u8,
    },
    Declaration {
        id: String,
        is_mut: bool,
        t: CType,
        rc: bool,
    },
    PtrDeclaration {
        id: String,
        is_mut: bool,
        ptr_data: AdrData<'a>,
        t: CType,
        adr: Box<AnnotatedNode<'a>>,
        // Refers to it being an rc_ptr itself, not a
        rc: bool,
    },
    Asm(String),
    // This is handled by the ptr declaration for now
    Adr {
        id: String,
        rc: bool,
    },
    DeRef {
        id: String,
        rc: bool,
        count: u8,
    },
    ArrayDeclaration {
        id: String,
        t: CType,
        size: usize,
    },
    FunctionDecaration {
        id: String,
        t: CType,
    },
    Assert,
    Return,
    PutChar,
    StructDeclaration(String),
}
impl<'a> Display for AnnotatedNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.token) // doesn't print values
    }
}

impl<'a> AnnotatedNode<'a> {
    pub fn print(&self, n: &mut i32) {
        (0..*n).into_iter().for_each(|_| print!("\t"));
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
pub fn annotate_ast<'a>(root: &'a Node, ctx: AnalysisContext) -> AnnotatedNode<'a> {
    let children = root.children.as_ref().unwrap_or(&vec![]).to_vec();
    let annotated_node_children = Some(
        children
            .iter()
            .map(|node| annotate_ast(node, ctx))
            .collect(),
    );
    let token = match &root.token {
        NodeType::Declaration(id, t, _) => {
            let declaration_info = ctx.get_var(id).expect("Declared id not in map");
            let is_mut = declaration_info.is_mut_by_ptr || declaration_info.is_mut_direct;
            let rc = declaration_info.rc;
            AnnotatedNodeT::Declaration {
                id: id.to_string(),
                is_mut,
                t: t.clone(),
                rc,
            }
        }
        NodeType::PtrDeclaration(id, t, adr) => {
            let ptr_info = ctx.get_var(id).expect("Ptr not found in info map");
            let is_mut = ptr_info.is_mut_by_ptr || ptr_info.is_mut_direct;
            let rc = ptr_info.rc;
            let annotated_adr = Box::new(annotate_ast(adr, ctx));
            let ptr_data = qtr_info
                .ptr_data
                .clone()
                .expect("Declared Ptr not in info map");
            AnnotatedNodeT::PtrDeclaration {
                id: id.to_string(),
                is_mut,
                ptr_data,
                t: t.clone(),
                adr: annotated_adr,
                rc,
            }
        }
        NodeType::Adr(id) => {
            // `(&mut t + (&b))` illegal
            // `&mut &mut &t` illegal
            // Unsafe assumption: Adresses are always immutable unless explicitely annotated otherwise by the ptr declaration
            // `list.append(&mut other_list)` isn't something we're going to worry about for now
            let rc = ctx
                .get_var(id)
                .as_ref()
                .expect("Id of adr not found in map")
                .rc;
            AnnotatedNodeT::Adr {
                id: id.to_string(),
                rc,
            }
        }
        // It seems like assignments and deref assignments need to handle referencing themselves
        // Unless we want Adr nodes to know what kind of reference they are (which actually is
        // sounding like the right decision now)
        NodeType::DerefAssignment(op, adr) => {
            let ids = find_ids(&adr);
            let derefed_id = ids[0].clone();
            let count = count_derefs(adr);
            let rc = *ctx
                .get_var(&derefed_id)
                .as_ref()
                .expect("dereffed_id not in map")
                .ptr_data
                .as_ref()
                .expect("ptr not ptr")
                .ptr_type
                .last()
                .unwrap()
                == PtrType::RcClone;
            AnnotatedNodeT::DerefAssignment {
                op: op.clone(),
                id: derefed_id.clone(),
                rc,
                count,
            }
        }
        NodeType::DeRef(expr) => {
            let count = count_derefs(expr) + 1;
            let ids = find_ids(&expr);
            let derefed_id = ids[0].clone();
            let rc = *ctx
                .get_var(&derefed_id)
                .as_ref()
                .expect("dereffed_id not in map")
                .ptr_data
                .as_ref()
                .expect("ptr not ptr")
                .ptr_type
                .last()
                .unwrap()
                == PtrType::RcClone;

            AnnotatedNodeT::DeRef {
                id: derefed_id.clone(),
                rc,
                count,
            }
        }
        NodeType::Id(id) => {
            let rc = ctx.get_var(id).as_ref().expect("Id not in map").rc;
            AnnotatedNodeT::Id {
                id: id.to_string(),
                rc,
            }
        }
        NodeType::Program => {
            let imports: Vec<String> = ctx
                .iter()
                .flat_map(|(_, data)| match &data.ptr_data {
                    Some(ptr_data) => ptr_data
                        .ptr_type
                        .iter()
                        .map(|ptr_type| match ptr_type {
                            PtrType::Rc => String::from("use std::rc::Rc;"),
                            PtrType::RefCell => String::from("use std::cell::RefCell;"),
                            PtrType::RcClone => String::from("use std::{rc::Rc, cell::RefCell};"),
                            _ => String::from(""),
                        })
                        .collect(),
                    None => vec![String::from("")],
                })
                .filter(|s| *s != String::new())
                .unique()
                .collect();
            AnnotatedNodeT::Program { imports }
        }
        NodeType::Assignment(op, id) => {
            let rc = ctx
                .get_var(id)
                .as_ref()
                .expect("Id being assigned to not in map")
                .rc;
            AnnotatedNodeT::Assignment {
                id: id.clone(),
                op: op.clone(),
                rc,
            }
        }
        node => node.to_annotated_node(),
    };
    AnnotatedNode {
        token,
        children: annotated_node_children,
    }
}
