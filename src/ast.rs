#[derive(Debug, PartialEq, Clone)]
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
    // Control Flow
    If(Box<(StatementNode, StatementNode, StatementNode, StatementNode)>),
    For(
        Box<(
            (String, CType, Box<ExprNode>), // Declaration
            StatementNode,                  // Can be anything
            StatementNode,                  // Can be anything
        )>,
        Vec<StatementNode>,
    ),
    While(Box<ExprNode>, Vec<StatementNode>),
    Break,

    // Functions
    FunctionCall(String, Vec<ExprNode>),
    FunctionDecaration(String, CType, Vec<(String, CType)>),
    Return,

    // Variables
    Assignment(OpNode, String, Box<CondExprNode>),
    Declaration(String, CType, Box<CondExprNode>),
    // Ptrs
    DerefAssignment(OpNode, Box<CondExprNode>),
    PtrDeclaration(String, CType, Box<CondExprNode>),
    ArrayDeclaration(String, CType, usize, Vec<CondExprNode>), // id, type, count

    // Types
    StructDeclaration(String, Vec<(String, CType)>),

    // Misc
    Asm(String),
    Assert(Box<(CondExprNode, CondExprNode)>),
    PutChar(Box<StatementNode>),
}

#[derive(Debug, PartialEq)]
pub enum CondExprNode {
    Term(CondTermNode),
    Op(CondExprOpNode),
}

#[derive(Debug, PartialEq)]
pub enum CondTermNode {
    Factor(CondFactorNode),
    Op(CondTermOpNode),
}

#[derive(Debug, PartialEq)]
pub enum CondFactorNode {
    ComparisonLiteral,
}

#[derive(Debug, PartialEq)]
pub enum ArithExprNode {
    Term(ArithTermNode),
    Op(ArithExprOpNode),
}

#[derive(Debug, PartialEq)]
pub enum ArithTermNode {
    Factor(ArithFactorNode),
    Op(ArithTermOpNode),
}

#[derive(Debug, PartialEq)]
pub enum ArithFactorNode {
    Id(String),
    NumLiteral(usize),
    Adr(String),
    DeRef(Box<CondExprNode>),
    Expr(Box<CondExprNode>),
}

pub enum OpNode {
    Unary(UnaryOpNode, Box<ArithFactorNode>),
    Term(ArithTermOpNode),
    Expr(ArithExprOpNode),
}

#[derive(Debug, PartialEq)]
pub enum ArithTermOpNode {
    Mul,
    Div,
    Mod,
}

#[derive(Debug, PartialEq)]
pub enum ArithExprOpNode {
    Add,
    Sub,
}

/// Unary Operators Come before value
#[derive(Debug, PartialEq)]
pub enum UnaryOpNode {
    Positive,
    Negative,
    Not,
    BNot,
    PlusPlus,
    MinusMinus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CondExprOpNode {
    Eq,
    NEq,
    Greater,
    Less,
    GreaterEq,
    LessEq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CondTermOpNode {
    Or,
    And,
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
    fn from_token(tok: &Token) -> Result<AssignmentOpType, ()> {
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

impl NodeType {
    fn from_token(tok: &Token) -> Result<NodeType, ()> {
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
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CType {
    Char,
    Int,
    Void,
}

impl fmt::Display for CType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        let p = match self {
            RhType::Char => "u8",
            RhType::Int => "u16",
            RhType::Void => "()",
        };
        write!(f, "{}", p)
    }
}
