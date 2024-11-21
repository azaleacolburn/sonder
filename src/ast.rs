use crate::{
    error::{ErrType, RhErr},
    lexer::Token,
    parser::TokenHandler,
};
use std::fmt::Debug;

pub trait Node: Debug {
    fn children(&self) -> Vec<&'static dyn Node> {
        todo!()
    }
}

// Valid Node Types
#[derive(Debug, Clone, PartialEq)]
pub enum StatementNode {
    Program(Vec<StatementNode>),
    Scope(Vec<StatementNode>),
    // Control Flow
    If(Box<CondExprNode>, Vec<StatementNode>),
    For(
        // Declaration, Condition, Anything
        Box<(
            Option<StatementNode>,
            Option<CondExprNode>,
            Option<StatementNode>,
        )>,
        Vec<StatementNode>,
    ),
    While(Box<CondExprNode>, Vec<StatementNode>),
    Break,

    // Functions
    FunctionCall(String, Vec<CondExprNode>),
    FunctionDecaration(String, CType, Vec<StatementNode>, Vec<StatementNode>),
    Return(Box<CondExprNode>),

    // Variables
    Assignment(AssignmentOpType, String, Box<CondExprNode>),
    Declaration(String, CType, Option<Box<CondExprNode>>),
    // Ptrs
    DerefAssignment(AssignmentOpType, Box<ArithExprNode>, Box<CondExprNode>),
    PtrDeclaration(String, CType, Box<CondExprNode>),
    ArrayDeclaration(String, CType, usize, Vec<CondExprNode>), // id, type, count

    // Types
    StructDeclaration(String, Vec<StatementNode>),

    // Misc
    Asm(String),
    Assert(Box<CondExprNode>),
    PutChar(Box<CondExprNode>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CondExprNode {
    Op(CondExprOp, Box<(CondExprNode, CondTermNode)>),
    Term(CondTermNode),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CondExprOp {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CondTermNode {
    Factor(ArithExprNode),
    Op(CondTermOpNode, Box<(CondTermNode, ArithExprNode)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CondTermOpNode {
    NEq,
    Eq,
    LT,
    GT,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithExprNode {
    Term(ArithTermNode),
    Op(ArithExprOpNode, Box<(ArithExprNode, ArithTermNode)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithTermNode {
    Factor(ArithFactorNode),
    Op(ArithTermOpNode, Box<(ArithTermNode, ArithFactorNode)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithFactorNode {
    Unary(UnaryOpNode, Box<ArithFactorNode>),
    Id(String),
    NumLiteral(usize),
    Adr(String),
    DeRef(Box<ArithExprNode>),
    FunctionCall(StatementNode),
    CondExpr(Box<CondExprNode>), // Used normally
    ArithExpr(Box<ArithExprNode>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpNode {
    Unary(UnaryOpNode, Box<ArithFactorNode>),
    Term(ArithTermOpNode),
    Expr(ArithExprOpNode),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithTermOpNode {
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithExprOpNode {
    Add,
    Sub,
}

/// Unary Operators Come before value
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOpNode {
    Negative(Box<ArithFactorNode>),
    Not(Box<ArithFactorNode>),
    BNot(Box<ArithFactorNode>),
    PlusPlus(Box<ArithFactorNode>),
    MinusMinus(Box<ArithFactorNode>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BitwiseOpNode {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentOpType {
    Eq,
    Sub,
    Add,
    Div,
    Mul,
    BOr,
    BAnd,
    BXor,
}

impl Node for StatementNode {
    fn children(&self) -> Vec<&'static dyn Node> {
        match self {
            StatementNode::If(_, children) => children.to_vec(),
            StatementNode::For(_, children) => children.to_vec(),
            StatementNode::Asm(_) => vec![],
            StatementNode::Scope(children) => children.to_vec(),
            StatementNode::While(_, children) => children.to_vec(),
            StatementNode::Break => vec![],
            StatementNode::Return(expr) => vec![*expr],
            StatementNode::Assert(expr) => vec![*expr],
            StatementNode::Program(children) => children.to_vec(),
            StatementNode::PutChar(expr) => vec![*expr],
            StatementNode::Assignment(_, _, expr) => vec![*expr],
            StatementNode::Declaration(_, _, expr) => vec![expr.unwrap()],
            StatementNode::FunctionCall(_, children) => *children,
            StatementNode::PtrDeclaration(_, _, expr) => vec![expr],
            StatementNode::DerefAssignment(_, _, expr) => vec![expr],
            StatementNode::ArrayDeclaration(_, _, _, items) => items,
            StatementNode::StructDeclaration(_, fields) => fields,
            StatementNode::FunctionDecaration(_, _, args, statements) => {
                args.iter().chain(statements.iter())
            }
        }
    }
}

impl Node for CondExprNode {
    fn children(&self) -> Vec<&'static dyn Node> {
        match self {
            CondExprNode::Op(_, exprs) => exprs,
            CondExprNode::Term(term) => term.children(),
        }
    }
}

impl Node for CondTermNode {
    fn children(&self) -> Vec<&'static dyn Node> {
        match self {
            CondTermNode::Op(_, exprs) => exprs,
            CondTermNode::Factor(factor) => factor.children(),
        }
    }
}

impl Node for ArithExprNode {
    fn children(&self) -> Vec<&'static dyn Node> {
        match self {
            ArithExprNode::Op(_, expr) => expr,
            ArithExprNode::Term(term) => term.children(),
        }
    }
}

impl Node for ArithTermNode {
    fn children(&self) -> Vec<&'static dyn Node> {
        match self {
            ArithTermNode::Op(_, term) => term,
            ArithTermNode::Factor(factor) => factor.children(),
        }
    }
}

impl Node for ArithFactorNode {
    fn children(&self) -> Vec<&'static dyn Node> {
        match self {
            ArithFactorNode::ArithExpr(expr) => expr.children(),
            ArithFactorNode::CondExpr(expr) => expr.children(),
            ArithFactorNode::FunctionCall(call) => call.children(),
            _ => vec![],
        }
    }
}

impl AssignmentOpType {
    pub fn from_token(token_handler: &mut TokenHandler) -> Result<AssignmentOpType, RhErr> {
        match token_handler.get_token() {
            Token::Eq => Ok(AssignmentOpType::Eq),
            Token::SubEq => Ok(AssignmentOpType::Sub),
            Token::AddEq => Ok(AssignmentOpType::Add),
            Token::DivEq => Ok(AssignmentOpType::Div),
            Token::MulEq => Ok(AssignmentOpType::Mul),
            Token::BOrEq => Ok(AssignmentOpType::BOr),
            Token::BAndEq => Ok(AssignmentOpType::BAnd),
            Token::BXor => Ok(AssignmentOpType::BXor),
            _ => Err(token_handler.new_err(ErrType::ExpectedAssignment)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CType {
    Void,
    Int,
    Char,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeType {
    Function(CType),
    While,
    Program,
    If,
    Loop,
    For,
}
