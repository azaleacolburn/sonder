/// Each variant wraps the line numberi the error was found on
#[derive(Debug, Clone, strum::Display)]
pub enum ErrType {
    ExpectedCParen,
    ExpectedCSquare,
    ExpectedExpression,
    ExpectedId,
    UndeclaredId,
    ExpectedAssignment,
    ExpectedStatement,
    ExpectedCondition,
    ExpectedOSquare,
    ExpectedOParen,
    ExpectedCCurl,
    ExpectedOCurl,
    ExpectedStrLiteral,
    ExpectedType,
    ExpectedSemi,
    ExpectedEq,
    ExpectedNumLiteral,
    ExpectedCondExprOp,
    ExpectedCondTermOp,
    ExpectedArithExprOp,
    ExpectedArithTermOp,
}

#[derive(Debug, Clone)]
pub struct RhErr {
    pub err: ErrType,
    pub line: i32,
}
