use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{AssignmentOpType, NodeType, ScopeType, TokenNode};
use crate::error::{ErrType as ET, RhErr};
use crate::lexer::{CType, LineNumHandler, Token};
use crate::token_handler::TokenHandler;

pub fn program(
    tokens: Vec<Token>,
    line_tracker: LineNumHandler,
    _debug: bool,
) -> Result<TokenNode, RhErr> {
    let mut token_handler = TokenHandler::new(tokens, line_tracker);
    let mut top_scope = scope(&mut token_handler, ScopeType::Program)?;

    if !top_scope.iter().any(|node| {
        node.token == NodeType::FunctionDeclaration("main".into(), CType::Int)
            || node.token == NodeType::FunctionDeclaration("main".into(), CType::Void)
    }) {
        top_scope.push(TokenNode::new(
            // TODO Figure out if we need a vector
            NodeType::FunctionDeclaration("main".into(), CType::Int),
            None,
            token_handler.line(),
        ))
    }
    let program_children = Rc::new(RefCell::new(top_scope.into_boxed_slice()));
    let program_node = TokenNode::new(NodeType::Program, Some(program_children), 0);

    program_node.print(&mut 0);
    Ok(program_node)
}

pub fn scope(
    token_handler: &mut TokenHandler,
    scope_type: ScopeType,
) -> Result<Vec<TokenNode>, RhErr> {
    // Result<TokenNode, RhErr> {
    let mut scope_children: Vec<TokenNode> = vec![];
    while *token_handler.get_token() != Token::CCurl {
        if token_handler.curr_token > token_handler.len() {
            return Err(token_handler.new_err(ET::ExpectedCParen));
        }

        scope_children.push(statement(token_handler, scope_type.clone())?);

        if token_handler.curr_token == token_handler.len() - 1 {
            return Ok(scope_children);
        }
        token_handler.next_token();
    }
    Ok(scope_children)
    // let mut scope_node = TokenNode::new(NodeType::Scope(None), None, token_handler.line());
    // if *token_handler.get_prev_token() == Token::Semi {
    //     scope_node.token = NodeType::Scope(Some(CType::Int)) // TODO: Change this to evaluate the  type of the last statement
    // }
    //
    // Ok(scope_node)
}

pub fn statement(
    token_handler: &mut TokenHandler,
    scope_type: ScopeType,
) -> Result<TokenNode, RhErr> {
    let statement_token = token_handler.get_token();
    println!("Statement Token: {:?}", statement_token);
    match statement_token {
        Token::Type(t) => type_statement(token_handler, t.clone()),
        Token::Id(name) => id_statement(token_handler, name.to_string()),
        // TODO: Maybe split deref_assignment into two null-terminals
        Token::Star => deref_assignment(token_handler, None),
        Token::If => if_statement(token_handler),
        Token::While => while_statement(token_handler),
        Token::For => for_statement(token_handler),
        Token::Break => {
            if scope_type == ScopeType::While || scope_type == ScopeType::Loop {
                Ok(TokenNode::new(NodeType::Break, None, token_handler.line()))
            } else {
                Err(token_handler.new_err(ET::ExpectedStatement))
            }
        }
        Token::Asm => asm_statement(token_handler),
        Token::Assert => assert_statement(token_handler),
        Token::Return => return_statement(token_handler),
        Token::PutChar => putchar_statement(token_handler),
        Token::Struct => struct_statement(token_handler),
        _ => Err(token_handler.new_err(ET::ExpectedStatement)),
    }
}

fn scalar_declaration_statement(
    token_handler: &mut TokenHandler,
    t: CType,
    id: String,
    ptr_cnt: u8,
) -> Result<TokenNode, RhErr> {
    if *token_handler.get_token() != Token::Eq {
        return Err(token_handler.new_err(ET::ExpectedEq));
    }
    token_handler.next_token();
    let expr = if ptr_cnt > 0 {
        arithmetic_expression(token_handler)?
    } else {
        condition_expr(token_handler)?
    };
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }
    Ok(if ptr_cnt > 0 {
        TokenNode::new(
            NodeType::PtrDeclaration(id, t, Box::new(expr.clone())),
            None,
            token_handler.line(),
        )
    } else {
        TokenNode::new(
            NodeType::Declaration(id, t, 0),
            Some(Rc::new(RefCell::new(Box::new([expr])))),
            token_handler.line(),
        )
    })
}

fn arithmetic_expression(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    let mut left = arithmetic_term(token_handler)?;
    let mut curr = token_handler.get_token().clone();
    println!("Expression curr: {:?}", curr);
    while curr == Token::Add || curr == Token::Sub {
        token_handler.next_token();
        let right = arithmetic_term(token_handler)?;
        left = TokenNode::new(
            NodeType::from_token(&curr).unwrap(),
            Some(Rc::new(RefCell::new(Box::new([left, right])))),
            token_handler.line(),
        );
        curr = token_handler.get_token().clone();
    }
    Ok(left)
}

fn arithmetic_term(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    let mut left: TokenNode = arithmetic_factor(token_handler)?;
    let mut curr = token_handler.get_token().clone();
    println!("Term curr: {:?}", curr);
    while curr == Token::Star || curr == Token::Div {
        token_handler.next_token();
        let right = arithmetic_factor(token_handler)?;
        left = TokenNode::new(
            NodeType::from_token(&curr).unwrap(),
            Some(Rc::new(RefCell::new(Box::new([left, right])))),
            token_handler.line(),
        );
        curr = token_handler.get_token().clone();
    }
    Ok(left)
}

fn arithmetic_factor(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    let token = token_handler.get_token().clone();
    let ret = match token {
        Token::NumLiteral(num) => Ok(TokenNode::new(
            NodeType::NumLiteral(num),
            None,
            token_handler.line(),
        )),
        Token::Id(id) => {
            if *token_handler.peek(1) == Token::OParen {
                Ok(function_call(token_handler, id.to_string())?)
            } else if *token_handler.peek(1) == Token::OSquare {
                token_handler.next_token();
                token_handler.next_token();
                let post_mul = TokenNode::new(
                    NodeType::Mul,
                    Some(Rc::new(RefCell::new(Box::new([
                        arithmetic_expression(token_handler)?,
                        TokenNode::new(NodeType::NumLiteral(8), None, token_handler.line()),
                    ])))),
                    token_handler.line(),
                );
                let post_add = TokenNode::new(
                    NodeType::Sub,
                    Some(Rc::new(RefCell::new(Box::new([
                        TokenNode::new(NodeType::Id(id.to_string()), None, token_handler.line()),
                        post_mul,
                    ])))),
                    token_handler.line(),
                );
                if *token_handler.get_token() != Token::CSquare {
                    return Err(token_handler.new_err(ET::ExpectedCSquare));
                }
                Ok(TokenNode::new(
                    NodeType::DeRef(Box::new(post_add)),
                    None,
                    token_handler.line(),
                ))
            } else {
                Ok(TokenNode::new(
                    NodeType::Id(id.to_string()),
                    None,
                    token_handler.line(),
                ))
            }
        }

        // Address of a variable
        Token::BAnd => {
            token_handler.next_token();
            if let Token::Id(id) = token_handler.get_token() {
                Ok(TokenNode::new(
                    NodeType::Adr(id.to_string()),
                    None,
                    token_handler.line(),
                ))
            } else {
                Err(token_handler.new_err(ET::ExpectedId))
            }
        }

        Token::Star => {
            token_handler.next_token();
            let factor = arithmetic_factor(token_handler)?;
            token_handler.prev_token();
            Ok(TokenNode::new(
                NodeType::DeRef(Box::new(factor)),
                None,
                token_handler.line(),
            ))
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

fn assignment(token_handler: &mut TokenHandler, name: String) -> Result<TokenNode, RhErr> {
    println!("Assignment token: {:?}", token_handler.get_token());
    if *token_handler.peek(1) == Token::OSquare {
        token_handler.next_token();
        return deref_assignment(token_handler, Some(name.clone()));
    }

    token_handler.next_token();
    let assignment_tok = AssignmentOpType::from_token(token_handler.get_token()).unwrap();

    token_handler.next_token();
    let token = TokenNode::new(
        NodeType::Assignment(assignment_tok, name.clone()),
        Some(Rc::new(RefCell::new(Box::new([arithmetic_expression(
            token_handler,
        )?])))),
        token_handler.line(),
    );
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    Ok(token)
}

// Token coming in should be (, id or [
// if [] => Some(name)
// else => None
fn deref_assignment(
    token_handler: &mut TokenHandler,
    name: Option<String>,
) -> Result<TokenNode, RhErr> {
    let first = token_handler.get_token().clone();
    println!("DeRef Assignment First: {:?}", first);

    let token = match first {
        Token::OSquare => {
            token_handler.next_token();
            let post_mul = TokenNode::new(
                NodeType::Mul,
                Some(Rc::new(RefCell::new(Box::new([
                    arithmetic_expression(token_handler)?,
                    TokenNode::new(NodeType::NumLiteral(8), None, token_handler.line()),
                ])))),
                token_handler.line(),
            );
            let post_mul = TokenNode::new(
                NodeType::Sub,
                Some(Rc::new(RefCell::new(Box::new([
                    TokenNode::new(
                        NodeType::Id(
                            name.expect("Array assignments must have ids with names")
                                .clone(),
                        ),
                        None,
                        token_handler.line(),
                    ),
                    post_mul,
                ])))),
                token_handler.line(),
            );
            if *token_handler.get_token() != Token::CSquare {
                return Err(token_handler.new_err(ET::ExpectedCSquare));
            }
            token_handler.next_token();

            post_mul
        }
        _ => arithmetic_expression(token_handler)?,
    };

    let deref_token = TokenNode::new(NodeType::DeRef(Box::new(token)), None, token_handler.line());
    let assignment_tok = AssignmentOpType::from_token(token_handler.get_token()).unwrap();
    token_handler.next_token();
    let token = TokenNode::new(
        NodeType::DerefAssignment(assignment_tok, Box::new(deref_token)),
        Some(Rc::new(RefCell::new(Box::new([arithmetic_expression(
            token_handler,
        )?])))),
        token_handler.line(),
    );
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    Ok(token)
}

fn while_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    token_handler.next_token();
    let condition_node = condition(token_handler)?;

    token_handler.next_token();
    token_handler.next_token();
    let scope_node = TokenNode::new(
        NodeType::Scope(None),
        Some(Rc::new(RefCell::new(
            scope(token_handler, ScopeType::While)?.into_boxed_slice(),
        ))),
        token_handler.line(),
    );

    let while_children = Box::new([condition_node, scope_node]);
    Ok(TokenNode::new(
        NodeType::While,
        Some(Rc::new(RefCell::new(while_children))),
        token_handler.line(),
    ))
}

fn if_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    token_handler.next_token(); // might make semi handled by the called functions instead
    let condition_node = condition(token_handler)?;

    token_handler.next_token();
    token_handler.next_token();
    let scope_node = TokenNode::new(
        NodeType::Scope(None),
        Some(Rc::new(RefCell::new(
            scope(token_handler, ScopeType::If)?.into_boxed_slice(),
        ))),
        token_handler.line(),
    );

    let if_children = Box::new([condition_node, scope_node]);
    Ok(TokenNode::new(
        NodeType::If,
        Some(Rc::new(RefCell::new(if_children))),
        token_handler.line(),
    ))
}

fn function_declare_statement(
    token_handler: &mut TokenHandler,
    t: CType,
    id: String,
) -> Result<TokenNode, RhErr> {
    token_handler.next_token();
    let mut args_scope = Vec::with_capacity(4);
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
        let arg_node = TokenNode::new(NodeType::Declaration(id, t, 0), None, token_handler.line());
        args_scope.push(arg_node);
        if *token_handler.get_token() != Token::Comma {
            break;
        }
        token_handler.next_token();
    }

    if *token_handler.get_token() != Token::CParen {
        return Err(token_handler.new_err(ET::ExpectedCParen));
    }
    token_handler.next_token();
    token_handler.next_token();

    let scope_node = TokenNode::new(
        NodeType::Scope(None),
        Some(Rc::new(RefCell::new(
            scope(token_handler, ScopeType::Function(t.clone()))?.into_boxed_slice(),
        ))),
        token_handler.line(),
    );
    args_scope.push(scope_node);

    let function_node = TokenNode::new(
        NodeType::FunctionDeclaration(id.clone(), t.clone()),
        Some(Rc::new(RefCell::new(args_scope.into_boxed_slice()))),
        token_handler.line(),
    );

    Ok(function_node)
}

fn function_call_statement(
    token_handler: &mut TokenHandler,
    name: String,
) -> Result<TokenNode, RhErr> {
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

fn function_call(token_handler: &mut TokenHandler, name: String) -> Result<TokenNode, RhErr> {
    token_handler.next_token();
    if *token_handler.get_token() != Token::OParen {
        return Err(token_handler.new_err(ET::ExpectedOParen));
    }
    token_handler.next_token();
    let mut args = Vec::with_capacity(3);
    loop {
        if *token_handler.get_token() == Token::CParen {
            break;
        }
        let arg_node = arithmetic_expression(token_handler)?;
        args.push(arg_node);
        if *token_handler.get_token() != Token::Comma {
            break;
        }
        token_handler.next_token();
    }
    if *token_handler.get_token() != Token::CParen {
        return Err(token_handler.new_err(ET::ExpectedCParen));
    }
    let function_call_node = TokenNode::new(
        NodeType::FunctionCall(name),
        Some(Rc::new(RefCell::new(args.into_boxed_slice()))),
        token_handler.line(),
    );

    Ok(function_call_node)
}

fn id_statement(token_handler: &mut TokenHandler, id: String) -> Result<TokenNode, RhErr> {
    println!("id statement token: {:?}", token_handler.get_token());
    match token_handler.peek(1) {
        Token::OParen => function_call_statement(token_handler, id),
        Token::Dot => struct_field_assignment(token_handler, id),
        _ => assignment(token_handler, id),
    }
}

fn type_statement(token_handler: &mut TokenHandler, t: CType) -> Result<TokenNode, RhErr> {
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
) -> Result<TokenNode, RhErr> {
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
        return Ok(TokenNode::new(
            NodeType::ArrayDeclaration(id.clone(), t, alloc_count),
            None,
            token_handler.line(),
        ));
    }

    token_handler.next_token();
    if *token_handler.get_token() != Token::OCurl {
        return Err(token_handler.new_err(ET::ExpectedOCurl));
    }

    token_handler.next_token();
    let mut items: Vec<TokenNode> = Vec::with_capacity(4);
    while let Token::NumLiteral(n) = *token_handler.get_token() {
        let item_node = TokenNode::new(NodeType::NumLiteral(n), None, token_handler.line());
        items.push(item_node);
        token_handler.next_token();
        let tok = token_handler.get_token();
        if *tok != Token::Comma && *tok != Token::CCurl {
            return Err(token_handler.new_err(ET::ExpectedCCurl));
        }

        token_handler.next_token();
    }

    if *token_handler.get_token() != Token::CCurl {
        return Err(token_handler.new_err(ET::ExpectedCCurl));
    }

    token_handler.next_token();
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    Ok(TokenNode::new(
        NodeType::ArrayDeclaration(id, t, alloc_count),
        Some(Rc::new(RefCell::new(items.into_boxed_slice()))),
        token_handler.line(),
    ))
}

fn condition(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
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

/// Expression parsing always ends on the token after the expression, usually a semicolon
fn condition_expr(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    let mut left = condition_term(token_handler)?;
    println!("Condition Expr Left: {:?}", left);
    let mut curr = token_handler.get_token().clone();
    println!("cond expr curr: {:?}", curr);
    while curr == Token::AndCmp || curr == Token::OrCmp {
        token_handler.next_token();
        let right = if *token_handler.get_token() == Token::OParen {
            token_handler.next_token();
            let expr = condition_expr(token_handler)?;
            if *token_handler.get_token() != Token::CParen {
                return Err(token_handler.new_err(ET::ExpectedCParen));
            }

            token_handler.next_token();
            expr
        } else {
            condition_term(token_handler)?
        };
        left = TokenNode::new(
            NodeType::from_token(&curr).unwrap(),
            Some(Rc::new(RefCell::new(Box::new([left, right])))),
            token_handler.line(),
        );
        curr = token_handler.get_token().clone();
        println!("\nCondition expr curr: {:?}", curr);
    }
    Ok(left)
}

fn condition_term(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    let mut left = arithmetic_expression(token_handler)?;
    println!("Left factor: {:?}", left);
    let mut curr = token_handler.get_token().clone();
    while curr == Token::NeqCmp || curr == Token::EqCmp {
        token_handler.next_token();
        let right = condition_factor(token_handler)?;
        println!("Right factor: {:?}", right);
        left = TokenNode::new(
            NodeType::from_token(&curr).unwrap(),
            Some(Rc::new(RefCell::new(Box::new([left, right])))),
            token_handler.line(),
        );
        curr = token_handler.get_token().clone();
        println!("curr: {:?}", curr);
    }
    Ok(left)
}

fn condition_factor(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    println!("Condition factor token: {:?}", token_handler.get_token());
    match token_handler.get_token() {
        Token::OParen => {
            token_handler.next_token();
            let expr = condition_expr(token_handler);
            println!("Post arith token: {:?}", token_handler.get_token());
            if *token_handler.get_token() != Token::CParen {
                return Err(token_handler.new_err(ET::ExpectedCParen));
            }
            expr
        }
        _ => arithmetic_expression(token_handler),
    }
}

fn asm_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
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
            return Ok(TokenNode::new(
                NodeType::Asm(str.to_string()),
                None,
                token_handler.line(),
            ));
        }
        _ => return Err(token_handler.new_err(ET::ExpectedStrLiteral)),
    }
}

fn for_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
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
            Some(TokenNode::new(
                NodeType::Declaration(id, t, 0),
                None,
                token_handler.line(),
            ))
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

    let children = Box::new([iterator_init, condition_expr, assignment_token])

    Ok(TokenNode::new(
        NodeType::For,
        Some(Rc::new(RefCell::new(children))),
        token_handler.line(),
    ))
}

pub fn assert_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    token_handler.next_token();
    if *token_handler.get_token() != Token::OParen {
        return Err(token_handler.new_err(ET::ExpectedOParen));
    }
    token_handler.next_token();

    let condition_node = condition_expr(token_handler)?;

    let node = TokenNode::new(
        NodeType::Assert,
        vec![condition_node].into(),
        token_handler.line(),
    );

    if *token_handler.get_token() == Token::CParen {
        return Err(token_handler.new_err(ET::ExpectedCParen));
    }
    token_handler.next_token();
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    return Ok(node);
}

pub fn putchar_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    token_handler.next_token();
    if *token_handler.get_token() != Token::OParen {
        return Err(token_handler.new_err(ET::ExpectedOParen));
    }
    token_handler.next_token();
    let expr_node = arithmetic_expression(token_handler)?;
    let putchar_node = TokenNode::new(
        NodeType::PutChar,
        Some(vec![expr_node]),
        token_handler.line(),
    );
    println!("putchar token after: {:?}", token_handler.get_token());
    if *token_handler.get_token() != Token::CParen {
        return Err(token_handler.new_err(ET::ExpectedCParen));
    }
    token_handler.next_token();
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }
    return Ok(putchar_node);
}

// pub fn print_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {}

pub fn return_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
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
    let return_token = TokenNode::new(
        NodeType::Return,
        Some(vec![expr_node]),
        token_handler.line(),
    );
    return Ok(return_token);
}

pub fn compound_literal(token_handler: &mut TokenHandler) -> Result<Vec<TokenNode>, RhErr> {
    token_handler.next_token();
    println!("compound_token: {:?}", token_handler.get_token());
    if *token_handler.get_token() != Token::OCurl {
        return Err(token_handler.new_err(ET::ExpectedOCurl));
    }

    token_handler.next_token();
    let mut fields = vec![];
    loop {
        let expr = condition_expr(token_handler)?;
        println!("expr_tok: {:?}", token_handler.get_token());
        fields.push(expr);
        if *token_handler.get_token() != Token::Comma {
            break;
        }
        token_handler.next_token()
    }

    if *token_handler.get_token() != Token::CCurl {
        return Err(token_handler.new_err(ET::ExpectedCCurl));
    }

    Ok(fields)
}

pub fn struct_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    token_handler.next_token();
    let struct_id = match token_handler.get_token() {
        Token::Id(struct_id) => struct_id.clone(),
        _ => return Err(token_handler.new_err(ET::ExpectedId)),
    };

    token_handler.next_token();
    match token_handler.get_token() {
        Token::OCurl => struct_definition(struct_id, token_handler),
        Token::Id(var_id) => struct_variable_declaration(struct_id, var_id.clone(), token_handler),
        _ => return Err(token_handler.new_err(ET::ExpectedId)),
    }
}

pub fn struct_variable_declaration(
    struct_id: String,
    var_id: String,
    token_handler: &mut TokenHandler,
) -> Result<TokenNode, RhErr> {
    token_handler.next_token();
    let exprs = match token_handler.get_token() {
        Token::Eq => compound_literal(token_handler)?,
        Token::Semi => Vec::new(),
        _ => return Err(token_handler.new_err(ET::ExpectedSemi)),
    };

    token_handler.next_token();
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    Ok(TokenNode::new(
        NodeType::StructDeclaration {
            var_id,
            struct_id,
            exprs,
        },
        None,
        token_handler.line(),
    ))
}

pub fn struct_definition(
    struct_id: String,
    token_handler: &mut TokenHandler,
) -> Result<TokenNode, RhErr> {
    let mut field_definitions: Vec<(String, usize, CType)> = vec![];
    token_handler.next_token();
    while let Ok(t) = get_type_name(token_handler) {
        token_handler.next_token();
        let mut ptr_count: usize = 0;
        while *token_handler.get_token() == Token::Star {
            ptr_count += 1;
            token_handler.next_token();
        }
        let id = match token_handler.get_token() {
            Token::Id(id) => id,
            _ => return Err(token_handler.new_err(ET::ExpectedId)),
        };
        field_definitions.push((id.clone(), ptr_count, t.clone()));
        token_handler.next_token();
        if *token_handler.get_token() != Token::Semi {
            return Err(token_handler.new_err(ET::ExpectedSemi));
        }
        token_handler.next_token();
    }

    println!("token: {:?}", token_handler.get_token());

    if *token_handler.get_token() != Token::CCurl {
        return Err(token_handler.new_err(ET::ExpectedCCurl));
    }

    token_handler.next_token();
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    Ok(TokenNode::new(
        NodeType::StructDefinition {
            struct_id,
            field_definitions,
        },
        None,
        token_handler.line(),
    ))
}

pub fn struct_field_assignment(
    token_handler: &mut TokenHandler,
    var_id: String,
) -> Result<TokenNode, RhErr> {
    // NOTE The dot is found with a peek(), so we have to the Dot token, then past it
    token_handler.next_token();
    token_handler.next_token();
    println!("token: {:?}", token_handler.get_token());
    let field_id = match token_handler.get_token() {
        Token::Id(id) => id,
        _ => return Err(token_handler.new_err(ET::ExpectedId)),
    }
    .clone();

    token_handler.next_token();
    let assignment_op = match AssignmentOpType::from_token(token_handler.get_token()) {
        Ok(op) if op != AssignmentOpType::Eq => return Err(token_handler.new_err(ET::ExpectedEq)),
        Ok(op) => op,
        Err(_) => return Err(token_handler.new_err(ET::ExpectedAssignment)),
    };

    token_handler.next_token();
    let expr = Box::new(condition_expr(token_handler)?);

    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    Ok(TokenNode::new(
        NodeType::StructFieldAssignment {
            var_id,
            field_id,
            assignment_op,
            expr,
        },
        None,
        token_handler.line(),
    ))
}

pub fn get_type_name(token_handler: &mut TokenHandler) -> Result<CType, RhErr> {
    match token_handler.get_token() {
        Token::Struct => {
            token_handler.next_token();
            match token_handler.get_token() {
                Token::Id(id) => Ok(CType::Struct(id.clone())),
                _ => return Err(token_handler.new_err(ET::ExpectedType)),
            }
        }
        Token::Type(t) => Ok(t.clone()),
        _ => return Err(token_handler.new_err(ET::ExpectedType)),
    }
}
