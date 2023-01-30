use crate::apperror::*;
use crate::ast::*;
use crate::token::*;
use std::{iter::Peekable, slice::Iter};

fn parse_primary(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<UntypedExpr, AppError> {
    match tokens.next().unwrap() {
        Token {
            tok: Tok::Num(val),
            pos,
        } => {
            let expr = UntypedExpr::Numeric {
                val: *val,
                pos: *pos,
                typ: Any,
            };
            Ok(expr)
        }
        Token {
            tok: Tok::Identifier(ident),
            pos,
        } => match tokens.peek().unwrap() {
            Token {
                tok: Tok::開き丸括弧,
                pos: open_pos,
            } => {
                tokens.next();

                let mut args = Vec::new();

                match tokens.peek().unwrap() {
                    Token {
                        tok: Tok::閉じ丸括弧,
                        ..
                    } => {
                        tokens.next();
                        let expr = UntypedExpr::Call {
                            ident: ident.clone(),
                            args,
                            pos: *pos,
                            typ: Any,
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
                            tok: Tok::閉じ丸括弧,
                            ..
                        } => {
                            tokens.next();
                            let expr = UntypedExpr::Call {
                                ident: ident.clone(),
                                args,
                                pos: *pos,
                                typ: Any,
                            };
                            break Ok(expr);
                        }
                        Token {
                            tok: Tok::Comma, ..
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
                let expr = UntypedExpr::Identifier {
                    ident: ident.clone(),
                    pos: *pos,
                    typ: Any,
                };
                Ok(expr)
            }
        },
        Token {
            tok: Tok::開き丸括弧,
            pos,
        } => {
            let expr = parse_expr(tokens, input)?;
            match tokens.next().unwrap() {
                Token {
                    tok: Tok::閉じ丸括弧,
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

fn parse_unary(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<UntypedExpr, AppError> {
    match tokens.peek() {
        Some(Token { tok: Tok::Add, .. }) => {
            tokens.next();
            parse_primary(tokens, input)
        }
        Some(Token { tok: Tok::Sub, pos }) => {
            tokens.next();
            let expr = parse_primary(tokens, input)?;
            Ok(UntypedExpr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos: *pos,
                typ: Any,
                左辺: Box::new(UntypedExpr::Numeric { val: 0, pos: *pos, typ: Any, }),
                右辺: Box::new(expr),
            })
        }
        Some(Token {
            tok: Tok::Asterisk,
            pos,
        }) => {
            tokens.next();
            let expr = parse_unary(tokens, input)?;
            Ok(UntypedExpr::UnaryExpr {
                op: UnaryOp::Deref,
                op_pos: *pos,
                expr: Box::new(expr),
                typ: Any,
            })
        }
        Some(Token {
            tok: Tok::Ampersand,
            pos,
        }) => {
            tokens.next();
            let expr = parse_unary(tokens, input)?;
            Ok(UntypedExpr::UnaryExpr {
                op: UnaryOp::Addr,
                op_pos: *pos,
                expr: Box::new(expr),
                typ: Any,
            })
        }
        _ => parse_primary(tokens, input),
    }
}

fn parse_multiplicative(
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<UntypedExpr, AppError> {
    let mut expr = parse_unary(tokens, input)?;
    loop {
        match tokens.peek() {
            Some(Token {
                tok: Tok::Asterisk,
                pos: op_pos,
            }) => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_unary(tokens, input)?);
                expr = UntypedExpr::BinaryExpr {
                    op: BinaryOp::Mul,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Any,
                };
            }
            Some(Token {
                tok: Tok::Div,
                pos: op_pos,
            }) => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_unary(tokens, input)?);
                expr = UntypedExpr::BinaryExpr {
                    op: BinaryOp::Div,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Any,
                };
            }

            _ => {
                return Ok(expr);
            }
        }
    }
}

fn parse_additive(
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<UntypedExpr, AppError> {
    let mut expr = parse_multiplicative(tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::Add,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_multiplicative(tokens, input)?);
                expr = UntypedExpr::BinaryExpr {
                    op: BinaryOp::Add,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Any,
                }
            }
            Token {
                tok: Tok::Sub,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_multiplicative(tokens, input)?);
                expr = UntypedExpr::BinaryExpr {
                    op: BinaryOp::Sub,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Any,
                }
            }
            _ => {
                return Ok(expr);
            }
        }
    }
}

fn parse_relational(
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<UntypedExpr, AppError> {
    let mut expr = parse_additive(tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::LessThan,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(tokens, input)?);
                expr = UntypedExpr::BinaryExpr {
                    op: BinaryOp::LessThan,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Any,
                }
            }
            Token {
                tok: Tok::LessThanOrEqual,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(tokens, input)?);
                expr = UntypedExpr::BinaryExpr {
                    op: BinaryOp::LessThanOrEqual,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Any,
                }
            }
            Token {
                tok: Tok::GreaterThan,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(tokens, input)?);
                expr = UntypedExpr::BinaryExpr {
                    op: BinaryOp::LessThan, // ここを逆転させ、
                    op_pos: *op_pos,
                    左辺: 右辺, // ここを逆転させればよい
                    右辺: 左辺,
                    typ: Any,
                }
            }
            Token {
                tok: Tok::GreaterThanOrEqual,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(tokens, input)?);
                expr = UntypedExpr::BinaryExpr {
                    op: BinaryOp::LessThanOrEqual, // ここを逆転させ、
                    op_pos: *op_pos,
                    左辺: 右辺, // ここを逆転させればよい
                    右辺: 左辺,
                    typ: Any,
                }
            }
            _ => {
                return Ok(expr);
            }
        }
    }
}

fn parse_equality(
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<UntypedExpr, AppError> {
    let mut expr = parse_relational(tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::Equal,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_relational(tokens, input)?);
                expr = UntypedExpr::BinaryExpr {
                    op: BinaryOp::Equal,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Any,
                }
            }
            Token {
                tok: Tok::NotEqual,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_relational(tokens, input)?);
                expr = UntypedExpr::BinaryExpr {
                    op: BinaryOp::NotEqual,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Any,
                }
            }
            _ => {
                return Ok(expr);
            }
        }
    }
}

pub fn parse_expr(
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<UntypedExpr, AppError> {
    let expr = parse_equality(tokens, input)?;
    let tok = tokens.peek().unwrap();
    match tok {
        Token {
            tok: Tok::Assign,
            pos: op_pos,
        } => {
            tokens.next();
            let 左辺 = Box::new(expr);
            let 右辺 = Box::new(parse_expr(tokens, input)?);
            Ok(UntypedExpr::BinaryExpr {
                op: BinaryOp::Assign,
                op_pos: *op_pos,
                左辺,
                右辺,
                typ: Any,
            })
        }
        _ => Ok(expr),
    }
}
