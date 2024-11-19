use crate::{
    error::{ErrType, RhErr},
    lexer::Token,
    parser::TokenHandler,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeType {
    Function(CType),
    While,
    Program,
    If,
    Loop,
    For,
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
    StructDeclaration(String, Vec<(String, CType)>),

    // Misc
    Asm(String),
    Assert(Box<CondExprNode>),
    PutChar(Box<CondExprNode>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CondExprNode {
    Op(CondExprOpNode),
    Term(CondTermNode),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CondExprOpNode {
    And(Box<(CondExprNode, CondTermNode)>),
    Or(Box<(CondExprNode, CondTermNode)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CondTermNode {
    Factor(ArithExprNode),
    Op(CondTermOpNode),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CondTermOpNode {
    NEq(Box<(CondTermNode, ArithExprNode)>),
    Eq(Box<(CondTermNode, ArithExprNode)>),
    LT(Box<(CondTermNode, ArithExprNode)>),
    GT(Box<(CondTermNode, ArithExprNode)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithExprNode {
    Term(ArithTermNode),
    Op(ArithExprOpNode),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithTermNode {
    Factor(ArithFactorNode),
    Op(ArithTermOpNode),
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
    Mul(Box<(ArithTermNode, ArithFactorNode)>),
    Div(Box<(ArithTermNode, ArithFactorNode)>),
    Mod(Box<(ArithTermNode, ArithFactorNode)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithExprOpNode {
    Add(Box<(ArithExprNode, ArithTermNode)>),
    Sub(Box<(ArithExprNode, ArithTermNode)>),
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
