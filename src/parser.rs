use crate::apperror::*;
use crate::ast::*;
use crate::token::*;
use std::{iter::Peekable, slice::Iter};

#[test]
fn parse_test() {
    use crate::tokenize::tokenize;
    let input = "5 - 3";
    let tokens = tokenize(input).unwrap();
    let mut tokens = tokens.iter().peekable();
    assert_eq!(
        parse(&mut tokens, input).unwrap(),
        Expr::BinaryExpr {
            op: BinaryOp::Sub,
            op_pos: 2,
            左辺: Box::new(Expr::Numeric { val: 5, pos: 0 }),
            右辺: Box::new(Expr::Numeric { val: 3, pos: 4 })
        }
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
        } => {
            let expr = Expr::Identifier {
                ident: *ident,
                pos: *pos,
            };
            Ok(expr)
        }
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

fn parse_program(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let mut expr = parse_expr(tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                payload: TokenPayload::Semicolon,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_expr(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::AndThen,
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

pub fn parse(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let expr = parse_program(tokens, input)?;
    let tok = tokens.peek().unwrap();
    if tok.payload == TokenPayload::Eof {
        Ok(expr)
    } else {
        Err(AppError {
            message: "期待されたeofが来ませんでした".to_string(),
            input: input.to_string(),
            pos: tok.pos,
        })
    }
}
