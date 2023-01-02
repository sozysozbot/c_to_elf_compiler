use crate::apperror::*;
use crate::ast::*;
use crate::token::*;
use std::{iter::Peekable, slice::Iter};

#[test]
fn parse_test() {
    use crate::tokenize::tokenize;
    let input = "5 - 3;";
    let tokens = tokenize(input).unwrap();
    let mut tokens = tokens.iter().peekable();
    assert_eq!(
        parse(&mut tokens, input).unwrap(),
        Program::Statements(vec![Statement::Expr {
            expr: Box::new(Expr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos: 2,
                左辺: Box::new(Expr::Numeric { val: 5, pos: 0 }),
                右辺: Box::new(Expr::Numeric { val: 3, pos: 4 })
            }),
            semicolon_pos: 5
        }])
    );
}

fn parse_primary(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    match tokens.next().unwrap() {
        Token {
            payload: TokenPayload::Num(val),
            pos,
        } => {
            let expr = Expr::Numeric {
                val: *val,
                pos: *pos,
            };
            Ok(expr)
        }
        Token {
            payload: TokenPayload::Identifier(ident),
            pos,
        } => match tokens.peek().unwrap() {
            Token {
                payload: TokenPayload::開き丸括弧,
                pos: open_pos,
            } => {
                tokens.next();

                let mut args = Vec::new();

                match tokens.peek().unwrap() {
                    Token {
                        payload: TokenPayload::閉じ丸括弧,
                        ..
                    } => {
                        tokens.next();
                        let expr = Expr::Call {
                            ident: ident.clone(),
                            args,
                            pos: *pos,
                        };
                        return Ok(expr);
                    }
                    _ => {
                        let expr = parse_expr(tokens, input)?;
                        args.push(expr);
                    }
                }

                loop {
                    match tokens.peek().unwrap() {
                        Token {
                            payload: TokenPayload::閉じ丸括弧,
                            ..
                        } => {
                            tokens.next();
                            let expr = Expr::Call {
                                ident: ident.clone(),
                                args,
                                pos: *pos,
                            };
                            break Ok(expr);
                        }
                        Token {
                            payload: TokenPayload::Comma,
                            ..
                        } => {
                            tokens.next();
                            let expr = parse_expr(tokens, input)?;
                            args.push(expr);
                        }
                        _ => {
                            break Err(AppError {
                                message: "閉じ丸括弧かカンマが期待されていました".to_string(),
                                input: input.to_string(),
                                pos: *open_pos + 1,
                            })
                        }
                    }
                }
            }
            _ => {
                let expr = Expr::Identifier {
                    ident: ident.clone(),
                    pos: *pos,
                };
                Ok(expr)
            }
        },
        Token {
            payload: TokenPayload::開き丸括弧,
            pos,
        } => {
            let expr = parse_expr(tokens, input)?;
            match tokens.next().unwrap() {
                Token {
                    payload: TokenPayload::閉じ丸括弧,
                    ..
                } => Ok(expr),
                _ => Err(AppError {
                    message: "この開き丸括弧に対応する閉じ丸括弧がありません".to_string(),
                    input: input.to_string(),
                    pos: *pos,
                }),
            }
        }
        tok => Err(AppError {
            message: "数値リテラルでも開き丸括弧でもないものが来ました".to_string(),
            input: input.to_string(),
            pos: tok.pos,
        }),
    }
}

fn parse_unary(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    match tokens.peek() {
        Some(Token {
            payload: TokenPayload::Add,
            ..
        }) => {
            tokens.next();
            parse_primary(tokens, input)
        }
        Some(Token {
            payload: TokenPayload::Sub,
            pos,
        }) => {
            tokens.next();
            let expr = parse_primary(tokens, input)?;
            Ok(Expr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos: *pos,
                左辺: Box::new(Expr::Numeric { val: 0, pos: *pos }),
                右辺: Box::new(expr),
            })
        }
        _ => parse_primary(tokens, input),
    }
}

fn parse_multiplicative(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let mut expr = parse_unary(tokens, input)?;
    loop {
        match tokens.peek() {
            Some(Token {
                payload: TokenPayload::Mul,
                pos: op_pos,
            }) => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_unary(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Mul,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                };
            }
            Some(Token {
                payload: TokenPayload::Div,
                pos: op_pos,
            }) => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_unary(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Div,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                };
            }

            _ => {
                return Ok(expr);
            }
        }
    }
}

fn parse_additive(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let mut expr = parse_multiplicative(tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                payload: TokenPayload::Add,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_multiplicative(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Add,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                }
            }
            Token {
                payload: TokenPayload::Sub,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_multiplicative(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Sub,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                }
            }
            _ => {
                return Ok(expr);
            }
        }
    }
}

fn parse_relational(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let mut expr = parse_additive(tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                payload: TokenPayload::LessThan,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LessThan,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                }
            }
            Token {
                payload: TokenPayload::LessThanOrEqual,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LessThanOrEqual,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                }
            }
            Token {
                payload: TokenPayload::GreaterThan,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LessThan, // ここを逆転させ、
                    op_pos: *op_pos,
                    左辺: 右辺, // ここを逆転させればよい
                    右辺: 左辺,
                }
            }
            Token {
                payload: TokenPayload::GreaterThanOrEqual,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LessThanOrEqual, // ここを逆転させ、
                    op_pos: *op_pos,
                    左辺: 右辺, // ここを逆転させればよい
                    右辺: 左辺,
                }
            }
            _ => {
                return Ok(expr);
            }
        }
    }
}

fn parse_equality(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let mut expr = parse_relational(tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                payload: TokenPayload::Equal,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_relational(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Equal,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                }
            }
            Token {
                payload: TokenPayload::NotEqual,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_relational(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::NotEqual,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                }
            }
            _ => {
                return Ok(expr);
            }
        }
    }
}

fn parse_expr(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let expr = parse_equality(tokens, input)?;
    let tok = tokens.peek().unwrap();
    match tok {
        Token {
            payload: TokenPayload::Assign,
            pos: op_pos,
        } => {
            tokens.next();
            let 左辺 = Box::new(expr);
            let 右辺 = Box::new(parse_expr(tokens, input)?);
            Ok(Expr::BinaryExpr {
                op: BinaryOp::Assign,
                op_pos: *op_pos,
                左辺,
                右辺,
            })
        }
        _ => Ok(expr),
    }
}

fn parse_statement(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Statement, AppError> {
    let tok = tokens.peek().unwrap();
    match tok {
        Token {
            payload: TokenPayload::Return,
            ..
        } => {
            tokens.next();
            let expr = Box::new(parse_expr(tokens, input)?);
            let tok = tokens.peek().unwrap();
            let semicolon_pos = match tok {
                Token {
                    payload: TokenPayload::Semicolon,
                    pos,
                } => {
                    tokens.next();
                    *pos
                }
                _ => {
                    return Err(AppError {
                        message: "期待されたセミコロンが来ませんでした".to_string(),
                        input: input.to_string(),
                        pos: tok.pos,
                    })
                }
            };
            Ok(Statement::Return {
                semicolon_pos,
                expr,
            })
        }
        Token {
            payload: TokenPayload::If,
            pos,
        } => {
            tokens.next();
            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    payload: TokenPayload::開き丸括弧,
                    ..
                } => {
                    tokens.next();
                }
                _ => {
                    return Err(AppError {
                        message: "期待された開き括弧が来ませんでした".to_string(),
                        input: input.to_string(),
                        pos: tok.pos,
                    })
                }
            }
            let cond = Box::new(parse_expr(tokens, input)?);

            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    payload: TokenPayload::閉じ丸括弧,
                    ..
                } => {
                    tokens.next();
                }
                _ => {
                    return Err(AppError {
                        message: "期待された閉じ括弧が来ませんでした".to_string(),
                        input: input.to_string(),
                        pos: tok.pos,
                    })
                }
            }
            let then = Box::new(parse_statement(tokens, input)?);
            let tok = tokens.peek().unwrap();
            let else_ = match tok {
                Token {
                    payload: TokenPayload::Else,
                    ..
                } => {
                    tokens.next();
                    Some(Box::new(parse_statement(tokens, input)?))
                }
                _ => None,
            };
            Ok(Statement::If {
                cond,
                then,
                else_,
                pos: *pos,
            })
        }
        Token {
            payload: TokenPayload::While,
            pos,
        } => {
            tokens.next();
            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    payload: TokenPayload::開き丸括弧,
                    ..
                } => {
                    tokens.next();
                }
                _ => {
                    return Err(AppError {
                        message: "期待された開き括弧が来ませんでした".to_string(),
                        input: input.to_string(),
                        pos: tok.pos,
                    })
                }
            }
            let cond = Box::new(parse_expr(tokens, input)?);

            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    payload: TokenPayload::閉じ丸括弧,
                    ..
                } => {
                    tokens.next();
                }
                _ => {
                    return Err(AppError {
                        message: "期待された閉じ括弧が来ませんでした".to_string(),
                        input: input.to_string(),
                        pos: tok.pos,
                    })
                }
            }
            let body = Box::new(parse_statement(tokens, input)?);
            Ok(Statement::While {
                cond,
                body,
                pos: *pos,
            })
        }
        Token {
            payload: TokenPayload::For,
            pos,
        } => {
            tokens.next();
            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    payload: TokenPayload::開き丸括弧,
                    ..
                } => {
                    tokens.next();
                }
                _ => {
                    return Err(AppError {
                        message: "期待された開き括弧が来ませんでした".to_string(),
                        input: input.to_string(),
                        pos: tok.pos,
                    })
                }
            }
            let tok = tokens.peek().unwrap();
            let init = match tok {
                Token {
                    payload: TokenPayload::Semicolon,
                    ..
                } => None,
                _ => Some(Box::new(parse_expr(tokens, input)?)),
            };
            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    payload: TokenPayload::Semicolon,
                    ..
                } => {
                    tokens.next();
                }
                _ => {
                    return Err(AppError {
                        message: "期待されたセミコロンが来ませんでした".to_string(),
                        input: input.to_string(),
                        pos: tok.pos,
                    })
                }
            }
            let tok = tokens.peek().unwrap();
            let cond = match tok {
                Token {
                    payload: TokenPayload::Semicolon,
                    ..
                } => None,
                _ => Some(Box::new(parse_expr(tokens, input)?)),
            };
            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    payload: TokenPayload::Semicolon,
                    ..
                } => {
                    tokens.next();
                }
                _ => {
                    return Err(AppError {
                        message: "期待されたセミコロンが来ませんでした".to_string(),
                        input: input.to_string(),
                        pos: tok.pos,
                    })
                }
            }
            let tok = tokens.peek().unwrap();
            let update = match tok {
                Token {
                    payload: TokenPayload::閉じ丸括弧,
                    ..
                } => None,
                _ => Some(Box::new(parse_expr(tokens, input)?)),
            };
            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    payload: TokenPayload::閉じ丸括弧,
                    ..
                } => {
                    tokens.next();
                }
                _ => {
                    return Err(AppError {
                        message: "期待された閉じ括弧が来ませんでした".to_string(),
                        input: input.to_string(),
                        pos: tok.pos,
                    })
                }
            }
            let body = Box::new(parse_statement(tokens, input)?);
            Ok(Statement::For {
                init,
                cond,
                update,
                body,
                pos: *pos,
            })
        }
        Token {
            payload: TokenPayload::開き波括弧,
            pos,
        } => {
            tokens.next();
            let mut statements = vec![];
            loop {
                match tokens.peek().unwrap() {
                    Token {
                        payload: TokenPayload::Eof,
                        pos,
                    } => {
                        return Err(AppError {
                            message: "期待された閉じ波括弧が来ませんでした".to_string(),
                            input: input.to_string(),
                            pos: *pos,
                        })
                    }
                    Token {
                        payload: TokenPayload::閉じ波括弧,
                        ..
                    } => {
                        tokens.next();

                        break;
                    }
                    _ => statements.push(parse_statement(tokens, input)?),
                }
            }
            Ok(Statement::Block {
                statements,
                pos: *pos,
            })
        }
        _ => {
            let expr = Box::new(parse_expr(tokens, input)?);
            let tok = tokens.peek().unwrap();
            let semicolon_pos = match tok {
                Token {
                    payload: TokenPayload::Semicolon,
                    pos,
                } => {
                    tokens.next();
                    *pos
                }
                _ => {
                    return Err(AppError {
                        message: "期待されたセミコロンが来ませんでした".to_string(),
                        input: input.to_string(),
                        pos: tok.pos,
                    })
                }
            };
            Ok(Statement::Expr {
                expr,
                semicolon_pos,
            })
        }
    }
}

fn parse_program(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Program, AppError> {
    let mut statements = vec![];
    while !matches!(
        tokens.peek(),
        Some(Token {
            payload: TokenPayload::Eof,
            pos: _,
        }),
    ) {
        statements.push(parse_statement(tokens, input)?);
    }
    Ok(Program::Statements(statements))
}

pub fn parse(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Program, AppError> {
    let program = parse_program(tokens, input)?;
    let tok = tokens.peek().unwrap();
    if tok.payload == TokenPayload::Eof {
        Ok(program)
    } else {
        Err(AppError {
            message: "期待されたeofが来ませんでした".to_string(),
            input: input.to_string(),
            pos: tok.pos,
        })
    }
}
