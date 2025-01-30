use std::{cell::RefCell, rc::Rc};

use crate::{
    annotater::AnnotatedNodeT,
    lexer::{CType, Token},
};
#[derive(Debug, PartialEq, Clone)]
pub enum ScopeType {
    Function(CType),
    While,
    Program,
    If,
    Loop,
    _For,
}

// Valid Node Types
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
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
    Assignment(AssignmentOpType, String), // id
    DerefAssignment(AssignmentOpType, Box<TokenNode>), // deref_node, deref count
    Declaration(String, CType, usize), // id, type, additional_reserved_size (for arrays)
    PtrDeclaration(String, CType, Box<TokenNode>),
    Asm(String),
    Adr(String),
    DeRef(Box<TokenNode>),
    ArrayDeclaration(String, CType, usize), // id, type, count
    FunctionDeclaration(String, CType),
    Assert,
    Return,
    PutChar,
    StructDefinition {
        struct_id: String,
        field_definitions: Vec<(String, usize, CType)>, // id, ptr_count, underlying type
    },
    StructDeclaration {
        var_id: String,
        struct_id: String,
        exprs: Vec<TokenNode>,
    }, // expr nodes
    StructFieldAssignment {
        var_id: String,
        field_id: String,
        assignment_op: AssignmentOpType,
        expr: Box<TokenNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentOpType {
    Eq,
    SubEq,
    AddEq,
    DivEq,
    MulEq,
    BOrEq,
    BAndEq,
    BXorEq,
}

impl AssignmentOpType {
    pub fn from_token(tok: &Token) -> Result<AssignmentOpType, ()> {
        match tok {
            Token::Eq => Ok(AssignmentOpType::Eq),
            Token::SubEq => Ok(AssignmentOpType::SubEq),
            Token::AddEq => Ok(AssignmentOpType::AddEq),
            Token::DivEq => Ok(AssignmentOpType::DivEq),
            Token::MulEq => Ok(AssignmentOpType::MulEq),
            Token::BOrEq => Ok(AssignmentOpType::BOrEq),
            Token::BAndEq => Ok(AssignmentOpType::BAndEq),
            Token::BXorEq => Ok(AssignmentOpType::BXorEq),
            _ => {
                println!("Oh God No, Not A Valid OpEq Token");
                return Err(());
            }
        }
    }
}

impl std::fmt::Display for AssignmentOpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match self {
            AssignmentOpType::Eq => "=",
            AssignmentOpType::SubEq => "-=",
            AssignmentOpType::DivEq => "/=",
            AssignmentOpType::AddEq => "+=",
            AssignmentOpType::MulEq => "*=",
            AssignmentOpType::BOrEq => "|=",
            AssignmentOpType::BXorEq => "^=",
            AssignmentOpType::BAndEq => "&=",
        };

        write!(f, "{}", op)
    }
}

impl NodeType {
    pub fn from_token(tok: &Token) -> Result<NodeType, ()> {
        println!("tok: {:?}", tok);
        match tok {
            Token::Sub => Ok(NodeType::Sub),
            Token::Div => Ok(NodeType::Div),
            Token::Eq => Ok(NodeType::Eq),
            Token::Id(str) => Ok(NodeType::Id(str.to_string())),
            Token::EqCmp => Ok(NodeType::EqCmp),
            Token::NeqCmp => Ok(NodeType::NeqCmp),
            Token::OrCmp => Ok(NodeType::OrCmp),
            Token::AndCmp => Ok(NodeType::AndCmp),
            Token::BOr => Ok(NodeType::BOr),
            Token::BAnd => Ok(NodeType::BAnd),
            Token::BXor => Ok(NodeType::BXor),
            Token::BOrEq => Ok(NodeType::BOrEq),
            Token::BAndEq => Ok(NodeType::BAndEq),
            Token::BXorEq => Ok(NodeType::BXorEq),
            Token::SubEq => Ok(NodeType::SubEq),
            Token::AddEq => Ok(NodeType::AddEq),
            Token::DivEq => Ok(NodeType::DivEq),
            Token::MulEq => Ok(NodeType::MulEq),
            Token::Star => Ok(NodeType::Mul), // exception for pointer
            Token::NumLiteral(i) => Ok(NodeType::NumLiteral(*i)),
            Token::Add => Ok(NodeType::Add),
            Token::For => Ok(NodeType::For),
            Token::While => Ok(NodeType::While),
            Token::If => Ok(NodeType::If),
            Token::Break => Ok(NodeType::Break),
            _ => {
                println!("Oh God No, Not A Valid Token");
                return Err(());
            }
        }
    }

    pub fn to_annotated_node(&self) -> AnnotatedNodeT {
        match self {
            NodeType::Sub => AnnotatedNodeT::Sub,
            NodeType::Div => AnnotatedNodeT::Div,
            NodeType::Eq => AnnotatedNodeT::Eq,
            NodeType::EqCmp => AnnotatedNodeT::EqCmp,
            NodeType::NeqCmp => AnnotatedNodeT::NeqCmp,
            NodeType::BOr => AnnotatedNodeT::BOr,
            NodeType::BAnd => AnnotatedNodeT::BAnd,
            NodeType::BXor => AnnotatedNodeT::BXor,
            NodeType::BOrEq => AnnotatedNodeT::BOrEq,
            NodeType::BAndEq => AnnotatedNodeT::BAndEq,
            NodeType::BXorEq => AnnotatedNodeT::BXorEq,
            NodeType::SubEq => AnnotatedNodeT::SubEq,
            NodeType::AddEq => AnnotatedNodeT::AddEq,
            NodeType::DivEq => AnnotatedNodeT::DivEq,
            NodeType::MulEq => AnnotatedNodeT::MulEq,
            NodeType::Mul => AnnotatedNodeT::Mul,
            NodeType::AndCmp => AnnotatedNodeT::AndCmp,
            NodeType::OrCmp => AnnotatedNodeT::OrCmp,
            NodeType::NumLiteral(size) => AnnotatedNodeT::NumLiteral(*size),
            NodeType::Add => AnnotatedNodeT::Add,
            NodeType::If => AnnotatedNodeT::If,
            NodeType::For => AnnotatedNodeT::For,
            NodeType::While => AnnotatedNodeT::While,
            NodeType::_Loop => AnnotatedNodeT::_Loop,
            NodeType::Break => AnnotatedNodeT::Break,
            NodeType::FunctionCall(s) => AnnotatedNodeT::FunctionCall(s.to_string()),
            NodeType::Scope(s) => AnnotatedNodeT::Scope(s.clone()),
            NodeType::Asm(asm) => AnnotatedNodeT::Asm(asm.to_string()),
            NodeType::ArrayDeclaration(id, t, size) => AnnotatedNodeT::ArrayDeclaration {
                id: id.to_string(),
                t: t.clone(),
                size: *size,
            },
            NodeType::FunctionDeclaration(id, t) => AnnotatedNodeT::FunctionDeclaration {
                id: id.to_string(),
                t: t.clone(),
            },
            NodeType::Assert => AnnotatedNodeT::Assert,
            NodeType::Return => AnnotatedNodeT::Return,
            NodeType::PutChar => AnnotatedNodeT::PutChar,
            node => {
                panic!("Should have been caught by parent match: {:?}", node)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenNode {
    pub token: NodeType,
    pub line: usize,
    pub children: Option<Rc<RefCell<[TokenNode]>>>,
}

impl std::fmt::Display for TokenNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.token) // doesn't print values
    }
}

impl TokenNode {
    pub fn new(
        token: NodeType,
        children: Option<Rc<RefCell<[TokenNode]>>>,
        line: usize,
    ) -> TokenNode {
        TokenNode {
            token,
            line,
            children,
        }
    }

    pub fn print(&self, n: &mut i32) {
        (0..*n).into_iter().for_each(|_| print!("  "));
        println!("{}", self);
        *n += 1;
        self.children.borrow().iter().for_each(|node| {
            node.print(n);
        });
        *n -= 1;
        // println!("End Children");
    }
}
