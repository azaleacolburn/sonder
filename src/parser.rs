use crate::ast::{
    ArithExprNode, ArithExprOpNode, ArithFactorNode, ArithTermNode, ArithTermOpNode,
    AssignmentOpType, CType, CondExprNode, CondExprOpNode, CondTermNode, CondTermOpNode, ScopeType,
    StatementNode,
};
use crate::error::{ErrType as ET, RhErr};
use crate::lexer::{LineNumHandler, Token};

pub struct TokenHandler {
    tokens: Vec<Token>,
    curr_token: usize,
    token_lines: Vec<i32>,
}

#[allow(dead_code)]
impl TokenHandler {
    pub fn new(tokens: Vec<Token>, line_tracker: LineNumHandler) -> Self {
        TokenHandler {
            tokens,
            curr_token: 0,
            token_lines: line_tracker.token_lines,
        }
    }

    pub fn next_token(&mut self) {
        self.curr_token += 1;
    }

    pub fn peek(&self, i: usize) -> &Token {
        &self.tokens[self.curr_token + i]
    }

    pub fn prev_token(&mut self) {
        self.curr_token -= 1;
    }

    pub fn get_token(&self) -> &Token {
        &self.tokens[self.curr_token]
    }

    pub fn get_prev_token(&self) -> &Token {
        &self.tokens[self.curr_token - 1]
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn new_err(&self, err: ET) -> RhErr {
        RhErr {
            err,
            line: self.token_lines[self.curr_token],
        }
    }
}

pub fn program(
    tokens: Vec<Token>,
    line_tracker: LineNumHandler,
    _debug: bool,
) -> Result<StatementNode, RhErr> {
    let mut token_handler = TokenHandler::new(tokens, line_tracker);

    let top_scope = scope(&mut token_handler, ScopeType::Program)?;
    let program_node = StatementNode::Program(top_scope);

    // TODO: Write print function for new ast structure
    // program_node.print(&mut 0);
    Ok(program_node)
}

pub fn scope(
    token_handler: &mut TokenHandler,
    scope_type: ScopeType,
) -> Result<Vec<StatementNode>, RhErr> {
    let mut children = vec![];
    while *token_handler.get_token() != Token::CCurl
        && token_handler.curr_token + 1 == token_handler.len() - 1
    {
        if token_handler.curr_token > token_handler.len() {
            return Err(token_handler.new_err(ET::ExpectedCParen));
        }

        children.push(statement(token_handler, scope_type.clone())?);
        token_handler.next_token();
    }
    Ok(children)
}
pub fn statement(
    token_handler: &mut TokenHandler,
    scope_type: ScopeType,
) -> Result<StatementNode, RhErr> {
    let statement_token = token_handler.get_token();
    println!("Statement Token: {:?}", statement_token);
    match statement_token {
        Token::Type(t) => type_statement(token_handler, t.clone()),
        Token::Id(name) => id_statement(token_handler, name.to_string()),
        // TODO: Maybe split deref_assignment into two null-terminals
        Token::Star => {
            token_handler.next_token();
            deref_assignment(token_handler, None)
        }
        Token::If => if_statement(token_handler),
        Token::While => while_statement(token_handler),
        Token::For => for_statement(token_handler),
        Token::Break => {
            if scope_type == ScopeType::While || scope_type == ScopeType::Loop {
                Ok(StatementNode::Break)
            } else {
                Err(token_handler.new_err(ET::ExpectedStatement))
            }
        }
        Token::Asm => asm_statement(token_handler),
        Token::Assert => assert_statement(token_handler),
        Token::Return => return_statement(token_handler),
        Token::PutChar => putchar_statement(token_handler),
        Token::Struct => struct_declaration_handler(token_handler),
        _ => Err(token_handler.new_err(ET::ExpectedStatement)),
    }
}

fn scalar_declaration_statement(
    token_handler: &mut TokenHandler,
    t: CType,
    id: String,
    ptr_cnt: u8,
) -> Result<StatementNode, RhErr> {
    if *token_handler.get_token() != Token::Eq {
        return Err(token_handler.new_err(ET::ExpectedEq));
    }
    token_handler.next_token();
    let expr = if ptr_cnt > 0 {
        CondExprNode::Term(CondTermNode::Factor(arithmetic_expression(token_handler)?))
    } else {
        condition_expr(token_handler)?
    };
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }
    Ok(if ptr_cnt > 0 {
        StatementNode::PtrDeclaration(id, t, Box::new(expr))
    } else {
        StatementNode::Declaration(id, t, Some(Box::new(expr)))
    })
}

fn assignment(token_handler: &mut TokenHandler, name: String) -> Result<StatementNode, RhErr> {
    println!("Assignment token: {:?}", token_handler.get_token());
    if *token_handler.peek(1) == Token::OSquare {
        token_handler.next_token();
        return deref_assignment(token_handler, Some(name.clone()));
    }

    token_handler.next_token();
    let assignment_tok = AssignmentOpType::from_token(token_handler)?;

    token_handler.next_token();
    let token = StatementNode::Assignment(
        assignment_tok,
        name.clone(),
        Box::new(condition_expr(token_handler)?),
    );
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    Ok(token)
}

fn deref_assignment(
    token_handler: &mut TokenHandler,
    name: Option<String>,
) -> Result<StatementNode, RhErr> {
    let first = token_handler.get_token().clone();
    println!("DeRef Assignment First: {:?}", first);

    let deref_token = match first {
        Token::OSquare => {
            token_handler.next_token();
            let post_mul = ArithTermOpNode::Mul(Box::new((
                ArithFactorNode::ArithExpr(Box::new(arithmetic_expression(token_handler)?)),
                ArithFactorNode::NumLiteral(8),
            )));
            let post_sub = ArithExprOpNode::Sub(Box::new((
                ArithTermNode::Factor(ArithFactorNode::Id(
                    name.expect("Array assignments must have ids with names")
                        .clone(),
                )),
                ArithTermNode::Op(post_mul),
            )));
            if *token_handler.get_token() != Token::CSquare {
                return Err(token_handler.new_err(ET::ExpectedCSquare));
            }
            token_handler.next_token();

            ArithExprNode::Op(post_sub)
        }
        _ => arithmetic_expression(token_handler)?,
    };

    let assignment_tok = AssignmentOpType::from_token(token_handler)?;
    token_handler.next_token();
    let assignment_token = StatementNode::DerefAssignment(
        assignment_tok,
        Box::new(deref_token),
        Box::new(CondExprNode::Term(CondTermNode::Factor(
            arithmetic_expression(token_handler)?,
        ))),
    );
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    Ok(assignment_token)
}

fn while_statement(token_handler: &mut TokenHandler) -> Result<StatementNode, RhErr> {
    token_handler.next_token();
    let condition_node = condition(token_handler)?;

    token_handler.next_token();
    token_handler.next_token();

    let scope_node = scope(token_handler, ScopeType::While)?;

    Ok(StatementNode::While(Box::new(condition_node), scope_node))
}

fn if_statement(token_handler: &mut TokenHandler) -> Result<StatementNode, RhErr> {
    token_handler.next_token(); // might make semi handled by the called functions instead
    let condition_node = condition(token_handler)?;

    token_handler.next_token();
    token_handler.next_token();

    let scope_node = scope(token_handler, ScopeType::If)?;

    Ok(StatementNode::If(Box::new(condition_node), scope_node))
}

fn function_declare_statement(
    token_handler: &mut TokenHandler,
    t: CType,
    id: String,
) -> Result<StatementNode, RhErr> {
    println!(
        "Function Declaration\nFunction Return Type: {:?}\nToken: {:?}",
        t,
        token_handler.get_token()
    );
    token_handler.next_token();
    let mut args = vec![];
    loop {
        let t = match token_handler.get_token() {
            Token::Type(t) => t.clone(),
            _ => break,
        };
        token_handler.next_token();
        let id = match token_handler.get_token() {
            Token::Id(id) => id.clone(),
            _ => return Err(token_handler.new_err(ET::ExpectedId)),
        };
        let arg_node = StatementNode::Declaration(id, t, None);
        args.push(arg_node);
        println!("Comma or CParen: {:?}", token_handler.get_token());
        if *token_handler.get_token() != Token::Comma {
            break;
        }
        token_handler.next_token();
    }
    println!("CParen: {:?}", token_handler.get_token());
    if *token_handler.get_token() != Token::CParen {
        return Err(token_handler.new_err(ET::ExpectedCParen));
    }
    token_handler.next_token();
    println!("OCurl: {:?}", token_handler.get_token());
    token_handler.next_token();
    let scope = scope(token_handler, ScopeType::Function(t.clone()))?;

    Ok(StatementNode::FunctionDecaration(
        id.clone(),
        t.clone(),
        args,
        scope,
    ))
}

fn function_call_statement(
    token_handler: &mut TokenHandler,
    name: String,
) -> Result<StatementNode, RhErr> {
    println!(
        "Function call statement node: {:?}",
        token_handler.get_token()
    );
    let call_node = function_call(token_handler, name)?;
    token_handler.next_token();
    println!("post call statement {:?}", token_handler.get_token());
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }
    Ok(call_node)
}

fn function_call(token_handler: &mut TokenHandler, name: String) -> Result<StatementNode, RhErr> {
    println!("Function call node: {:?}", token_handler.get_token());
    token_handler.next_token();
    if *token_handler.get_token() != Token::OParen {
        return Err(token_handler.new_err(ET::ExpectedOParen));
    }
    let mut args = vec![];
    token_handler.next_token();
    loop {
        println!("Call arg: {:?}", token_handler.get_token());
        if *token_handler.get_token() == Token::CParen {
            break;
        }
        let arg_node = condition_expr(token_handler)?;
        args.push(arg_node);
        if *token_handler.get_token() != Token::Comma {
            break;
        }
        token_handler.next_token();
    }
    println!("post args token: {:?}", token_handler.get_token());
    if *token_handler.get_token() != Token::CParen {
        return Err(token_handler.new_err(ET::ExpectedCParen));
    }

    Ok(StatementNode::FunctionCall(name, args))
}

fn id_statement(token_handler: &mut TokenHandler, id: String) -> Result<StatementNode, RhErr> {
    println!("id statement token: {:?}", token_handler.get_token());
    match token_handler.peek(1) {
        Token::OParen => function_call_statement(token_handler, id),
        _ => assignment(token_handler, id),
    }
}

fn type_statement(token_handler: &mut TokenHandler, t: CType) -> Result<StatementNode, RhErr> {
    // let id = if let Token::Id(id) = token_handler.get_token() {
    //     id
    // } else {
    //     return Err(token_handler.new_err(ET::ExpectedId));
    // };
    token_handler.next_token();
    let mut ptr_cnt = 0;
    let mut ptr_tok = token_handler.get_token();
    println!("tok: {:?}", ptr_tok);
    while *ptr_tok == Token::Star {
        ptr_cnt += 1;
        token_handler.next_token();
        ptr_tok = token_handler.get_token();
    }

    let id = if let Token::Id(id) = token_handler.get_token() {
        id.clone()
    } else {
        return Err(token_handler.new_err(ET::ExpectedId));
    };

    token_handler.next_token();
    match token_handler.get_token() {
        Token::OParen => function_declare_statement(token_handler, t, id.clone()),
        Token::OSquare => array_declare_statement(token_handler, t, id.clone()),
        _ => scalar_declaration_statement(token_handler, t, id.clone(), ptr_cnt),
    }
}

fn array_declare_statement(
    token_handler: &mut TokenHandler,
    t: CType,
    id: String,
) -> Result<StatementNode, RhErr> {
    token_handler.next_token(); // Already checked open square bracket
    let alloc_count = match token_handler.get_token() {
        Token::NumLiteral(n) => *n as usize,
        _ => return Err(token_handler.new_err(ET::ExpectedNumLiteral)),
    };

    token_handler.next_token();
    if *token_handler.get_token() != Token::CSquare {
        return Err(token_handler.new_err(ET::ExpectedCSquare));
    }

    if *token_handler.get_token() != Token::Eq {
        if *token_handler.get_token() != Token::Semi {
            return Err(token_handler.new_err(ET::ExpectedSemi));
        }
        return Ok(StatementNode::ArrayDeclaration(
            id.clone(),
            t,
            alloc_count,
            vec![],
        ));
    }

    token_handler.next_token();
    if *token_handler.get_token() != Token::OCurl {
        return Err(token_handler.new_err(ET::ExpectedOCurl));
    }

    token_handler.next_token();
    let mut items: Vec<CondExprNode> = vec![];

    loop {
        if *token_handler.get_token() == Token::CCurl {
            break;
        }
        token_handler.next_token();
        let expr = condition_expr(token_handler)?;
        items.push(expr);

        if *token_handler.get_token() != Token::Comma {
            return Err(token_handler.new_err(ET::ExpectedComma));
        }
        token_handler.next_token();
    }

    // TODO: Check if this is needed
    if *token_handler.get_token() != Token::CCurl {
        return Err(token_handler.new_err(ET::ExpectedCCurl));
    }

    token_handler.next_token();
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    Ok(StatementNode::ArrayDeclaration(id, t, alloc_count, items))
}

fn condition(token_handler: &mut TokenHandler) -> Result<CondExprNode, RhErr> {
    // let condition_node = TokenNode::new(NodeType::Condition());
    // token_handler.next_token();
    println!("\nOpening condition token: {:?}", token_handler.get_token());
    match token_handler.get_token() {
        Token::OParen => {
            // evaluate condition
            token_handler.next_token();
            let condition = condition_expr(token_handler);
            println!("Post condition token: {:?}", token_handler.get_token());
            //token_handler.next_token();
            match token_handler.get_token() {
                Token::CParen => condition,
                _ => {
                    println!("post condition {:?}\n", token_handler.get_token());
                    Err(token_handler.new_err(ET::ExpectedCParen))
                }
            }
        }
        _ => Err(token_handler.new_err(ET::ExpectedOParen)),
    }
}

fn condition_expr(token_handler: &mut TokenHandler) -> Result<CondExprNode, RhErr> {
    // TODO: Each side of the equation needs to hold two CondTermNodes, but they're likely actually
    // cond-expr nodes
    // The grammar is very round-about and allows multiple expression ops through a variant of
    // Factor, but this is really ugly to put in the AST
    let mut left = CondExprNode::Term(condition_term(token_handler)?);
    println!("Condition Expr Left: {:?}", left);
    let mut curr = token_handler.get_token().clone();
    println!("cond expr curr: {:?}", curr);
    while curr == Token::AndCmp || curr == Token::OrCmp {
        token_handler.next_token();
        let right: CondTermNode = if *token_handler.get_token() == Token::OParen {
            token_handler.next_token();
            let expr = condition_term(token_handler)?;
            if *token_handler.get_token() != Token::CParen {
                return Err(token_handler.new_err(ET::ExpectedCParen));
            }

            token_handler.next_token();
            expr
        } else {
            condition_term(token_handler)?
        };
        left = CondExprNode::Op(match token_handler.get_token() {
            Token::AndCmp => CondExprOpNode::And(Box::new((left, right))),
            Token::OrCmp => CondExprOpNode::Or(Box::new((left, right))),
            _ => return Err(token_handler.new_err(ET::ExpectedCondExprOp)),
        });
        curr = token_handler.get_token().clone();
        println!("\nCondition expr curr: {:?}", curr);
    }
    Ok(left)
}

fn condition_term(token_handler: &mut TokenHandler) -> Result<CondTermNode, RhErr> {
    let mut left = CondTermNode::Factor(arithmetic_expression(token_handler)?);
    println!("Left factor: {:?}", left);
    let mut curr = token_handler.get_token().clone();
    while curr == Token::NeqCmp || curr == Token::EqCmp {
        token_handler.next_token();
        let right = condition_factor(token_handler)?;
        println!("Right factor: {:?}", right);
        left = CondTermNode::Op(match token_handler.get_token() {
            Token::NeqCmp => CondTermOpNode::NEq(Box::new((left, right))),
            Token::EqCmp => CondTermOpNode::Eq(Box::new((left, right))),
            _ => return Err(token_handler.new_err(ET::ExpectedCondTermOp)),
        });

        curr = token_handler.get_token().clone();
        println!("curr: {:?}", curr);
    }
    Ok(left)
}

fn condition_factor(token_handler: &mut TokenHandler) -> Result<ArithExprNode, RhErr> {
    println!("Condition factor token: {:?}", token_handler.get_token());
    match token_handler.get_token() {
        Token::OParen => {
            token_handler.next_token();
            let expr = condition_expr(token_handler)?;
            println!("Post arith token: {:?}", token_handler.get_token());
            if *token_handler.get_token() != Token::CParen {
                return Err(token_handler.new_err(ET::ExpectedCParen));
            }
            Ok(expr)
        }
        _ => arithmetic_expression(token_handler),
    }
}

fn arithmetic_expression(token_handler: &mut TokenHandler) -> Result<ArithExprNode, RhErr> {
    let mut left = arithmetic_term(token_handler)?;
    let mut curr = token_handler.get_token().clone();
    println!("Expression curr: {:?}", curr);
    while curr == Token::Add || curr == Token::Sub {
        token_handler.next_token();
        let right = arithmetic_term(token_handler)?;
        left = match token_handler.get_token() {
            Token::Add => ArithExprOpNode::Add(Box::new((left, right))),
            Token::Sub => ArithExprOpNode::Sub(Box::new((left, right))),
            _ => return token_handler.new_err(ET::ExpectedArithExprOp),
        };

        curr = token_handler.get_token().clone();
    }
    Ok(left)
}

fn arithmetic_term(token_handler: &mut TokenHandler) -> Result<ArithTermNode, RhErr> {
    let mut left = arithmetic_factor(token_handler)?;
    let mut curr = token_handler.get_token().clone();
    println!("Term curr: {:?}", curr);
    while curr == Token::Star || curr == Token::Div {
        token_handler.next_token();
        let right = arithmetic_factor(token_handler)?;
        left = match token_handler.get_token() {
            Token::Star => ArithTermOpNode::Mul(Box::new((left, right))),
            Token::Div => ArithTermOpNode::Div(Box::new((left, right))),
            _ => return token_handler.new_err(ET::ExpectedArithTermOp),
        };
        curr = token_handler.get_token().clone();
    }
    Ok(left)
}

fn arithmetic_factor(token_handler: &mut TokenHandler) -> Result<ArithFactorNode, RhErr> {
    let token = token_handler.get_token().clone();
    let ret = match token {
        Token::NumLiteral(num) => Ok(ArithFactorNode::NumLiteral(num)),
        Token::Id(id) => {
            if *token_handler.peek(1) == Token::OParen {
                Ok(function_call(token_handler, id.to_string())?)
            } else if *token_handler.peek(1) == Token::OSquare {
                token_handler.next_token();
                token_handler.next_token();
                let post_mul = ArithTermOpNode::Mul(Box::new((
                    arithmetic_expression(token_handler),
                    ArithFactorNode::NumLiteral(8),
                )));
                let post_add =
                    ArithExprOpNode::Sub(Box::new((ArithFactorNode::Id(id.to_string()), post_mul)));
                if *token_handler.get_token() != Token::CSquare {
                    return Err(token_handler.new_err(ET::ExpectedCSquare));
                }
                Ok(ArithFactorNode::DeRef(Box::new(post_add)))
            } else {
                Ok(ArithFactorNode::Id(id.to_string()))
            }
        }

        // Address of a variable
        Token::BAnd => {
            token_handler.next_token();
            if let Token::Id(id) = token_handler.get_token() {
                Ok(ArithFactorNode::Adr(id.to_string()))
            } else {
                Err(token_handler.new_err(ET::ExpectedId))
            }
        }

        Token::Star => {
            token_handler.next_token();
            let factor = arithmetic_factor(token_handler)?;
            token_handler.prev_token();
            Ok(ArithFactorNode::DeRef(Box::new(factor)))
        }

        Token::OParen => {
            token_handler.next_token();
            match arithmetic_expression(token_handler) {
                Ok(node) => {
                    if *token_handler.get_token() == Token::CParen {
                        Ok(node)
                    } else {
                        Err(token_handler.new_err(ET::ExpectedCParen))
                    }
                }
                Err(err) => Err(err),
            }
        }
        _ => Err(token_handler.new_err(ET::ExpectedExpression)),
    };
    token_handler.next_token();
    return ret;
}

fn asm_statement(token_handler: &mut TokenHandler) -> Result<StatementNode, RhErr> {
    token_handler.next_token();
    if *token_handler.get_token() != Token::OParen {
        return Err(token_handler.new_err(ET::ExpectedOParen));
    }
    token_handler.next_token();
    match token_handler.get_token().clone() {
        Token::StrLiteral(str) => {
            println!("Asm string: {}", str);
            token_handler.next_token();
            if *token_handler.get_token() != Token::CParen {
                return Err(token_handler.new_err(ET::ExpectedCParen));
            }
            token_handler.next_token();
            if *token_handler.get_token() != Token::Semi {
                println!("TOKEN: {:?}", token_handler.get_token());
                return Err(token_handler.new_err(ET::ExpectedSemi));
            }
            return Ok(StatementNode::Asm(str.to_string()));
        }
        _ => return Err(token_handler.new_err(ET::ExpectedStrLiteral)),
    }
}

fn for_statement(token_handler: &mut TokenHandler) -> Result<StatementNode, RhErr> {
    token_handler.next_token();
    if *token_handler.get_token() != Token::OParen {
        return Err(token_handler.new_err(ET::ExpectedOParen));
    }

    token_handler.next_token();
    let iterator_init = match token_handler.get_token().clone() {
        Token::Type(t) => {
            token_handler.next_token();
            let id = match token_handler.get_token() {
                Token::Id(id) => id.clone(),
                _ => return Err(token_handler.new_err(ET::ExpectedId)),
            };
            token_handler.next_token();
            let expr = match token_handler.get_token() {
                Token::Semi => Some(StatementNode::Declaration(id, t, None)),
                Token::Eq => Some(condition_expr(token_handler)),
                _ => return Err(token_handler.new_err(ET::ExpectedEq)),
            };

            Some(StatementNode::Declaration(id, t, Some(Box::new(expr))))
        }
        Token::Semi => None,
        _ => return Err(token_handler.new_err(ET::ExpectedSemi)),
    };
    token_handler.next_token();
    let condition_expr = match token_handler.get_token().clone() {
        Token::Semi => None,
        _ => Some(condition_expr(token_handler)?),
    };
    let assignment_token = match token_handler.get_token().clone() {
        Token::Semi => None,
        // TODO: Check what exactly is allowed in the third slot
        Token::Id(id) => Some(assignment(token_handler, id)?),
        _ => return Err(token_handler.new_err(ET::ExpectedSemi)),
    };

    let scope = scope(token_handler, ScopeType::For)?;

    Ok(StatementNode::For(
        Box::new((iterator_init, condition_expr, assignment_token)),
        scope,
    ))
}

pub fn assert_statement(token_handler: &mut TokenHandler) -> Result<StatementNode, RhErr> {
    token_handler.next_token();
    if *token_handler.get_token() != Token::OParen {
        return Err(token_handler.new_err(ET::ExpectedOParen));
    }
    token_handler.next_token();

    let condition_node = condition_expr(token_handler)?;
    let node = StatementNode::Assert(Box::new(condition_node));

    if *token_handler.get_token() == Token::CParen {
        return Err(token_handler.new_err(ET::ExpectedCParen));
    }
    token_handler.next_token();
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    return Ok(node);
}

pub fn putchar_statement(token_handler: &mut TokenHandler) -> Result<StatementNode, RhErr> {
    token_handler.next_token();
    if *token_handler.get_token() != Token::OParen {
        return Err(token_handler.new_err(ET::ExpectedOParen));
    }

    token_handler.next_token();
    let expr_node = arithmetic_expression(token_handler)?;

    if *token_handler.get_token() != Token::CParen {
        return Err(token_handler.new_err(ET::ExpectedCParen));
    }
    token_handler.next_token();
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    Ok(StatementNode::PutChar(Box::new(expr_node)))
}

pub fn return_statement(token_handler: &mut TokenHandler) -> Result<StatementNode, RhErr> {
    token_handler.next_token();
    if *token_handler.get_token() != Token::OParen {
        return Err(token_handler.new_err(ET::ExpectedOParen));
    }
    token_handler.next_token();
    let expr_node = condition_expr(token_handler)?;
    println!("post return {:?}", token_handler.get_token());
    if *token_handler.get_token() != Token::CParen {
        return Err(token_handler.new_err(ET::ExpectedCParen));
    }
    token_handler.next_token();
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    Ok(StatementNode::Return(Box::new(expr_node)))
}

pub fn struct_declaration_handler(
    token_handler: &mut TokenHandler,
) -> Result<StatementNode, RhErr> {
    token_handler.next_token();
    let id = if let Token::Id(id) = token_handler.get_token() {
        id.to_string()
    } else {
        return Err(token_handler.new_err(ET::ExpectedId));
    };

    token_handler.next_token();
    if *token_handler.get_token() != Token::OCurl {
        return Err(token_handler.new_err(ET::ExpectedOCurl));
    }

    let mut field_definitions = vec![];
    token_handler.next_token();
    while let Token::Type(t) = token_handler.get_token().clone() {
        token_handler.next_token();
        let id = match token_handler.get_token() {
            Token::Id(id) => id,
            _ => return Err(token_handler.new_err(ET::ExpectedId)),
        };
        let declaration = StatementNode::Declaration(id.clone(), t, None);
        field_definitions.push(declaration);
        token_handler.next_token();
        if *token_handler.get_token() != Token::Comma && *token_handler.get_token() != Token::CParen
        {
            return Err(token_handler.new_err(ET::ExpectedSemi));
        }
    }

    if *token_handler.get_token() != Token::CCurl {
        return Err(token_handler.new_err(ET::ExpectedCCurl));
    }

    Ok(StatementNode::StructDeclaration(id, field_definitions))
}
