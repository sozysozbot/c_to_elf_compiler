use crate::apperror::*;
use crate::ast::*;
use crate::token::*;
use std::{iter::Peekable, slice::Iter};

use super::toplevel::Context;
use super::toplevel::Type;
use super::typ::parse_type;
fn parse_primary(
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<Expr, AppError> {
    match tokens.next().unwrap() {
        Token {
            tok: Tok::Num(val),
            pos,
        } => {
            let expr = Expr::Numeric {
                val: *val,
                pos: *pos,
                typ: Type::Int,
            };
            Ok(expr)
        }
        Token {
            tok: Tok::Identifier(ident),
            pos: ident_pos,
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
                        let func_decl =
                            context.function_declarations.get(ident).ok_or(AppError {
                                message: format!(
                                    "関数 {} は宣言されておらず、戻り値の型が分かりません",
                                    ident
                                ),
                                input: input.to_string(),
                                pos: *ident_pos,
                            })?;
                        let expr = Expr::Call {
                            ident: ident.clone(),
                            args,
                            pos: *ident_pos,
                            typ: func_decl.return_type.clone(),
                        };
                        return Ok(expr);
                    }
                    _ => {
                        let expr = parse_expr(context, tokens, input)?;
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
                            let func_decl =
                                context.function_declarations.get(ident).ok_or(AppError {
                                    message: format!(
                                        "関数 {} は宣言されておらず、戻り値の型が分かりません",
                                        ident
                                    ),
                                    input: input.to_string(),
                                    pos: *ident_pos,
                                })?;
                            let expr = Expr::Call {
                                ident: ident.clone(),
                                args,
                                pos: *ident_pos,
                                typ: func_decl.return_type.clone(),
                            };
                            break Ok(expr);
                        }
                        Token {
                            tok: Tok::Comma, ..
                        } => {
                            tokens.next();
                            let expr = parse_expr(context, tokens, input)?;
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
                    pos: *ident_pos,
                    typ: context
                        .local_var_and_param_declarations
                        .get(ident)
                        .ok_or(AppError {
                            message: format!(
                                "識別子 {} は定義されておらず、型が分かりません",
                                ident
                            ),
                            input: input.to_string(),
                            pos: *ident_pos,
                        })?
                        .clone(),
                };
                Ok(expr)
            }
        },
        Token {
            tok: Tok::開き丸括弧,
            pos,
        } => {
            let expr = parse_expr(context, tokens, input)?;
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

fn parse_unary(
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<Expr, AppError> {
    match tokens.peek() {
        Some(Token { tok: Tok::Add, .. }) => {
            tokens.next();
            parse_primary(context, tokens, input)
        }
        Some(Token { tok: Tok::Sub, pos }) => {
            tokens.next();
            let expr = parse_primary(context, tokens, input)?;
            Ok(Expr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos: *pos,
                typ: Type::Int,
                左辺: Box::new(Expr::Numeric {
                    val: 0,
                    pos: *pos,
                    typ: Type::Int,
                }),
                右辺: Box::new(expr),
            })
        }
        Some(Token {
            tok: Tok::Asterisk,
            pos,
        }) => {
            tokens.next();
            let expr = parse_unary(context, tokens, input)?;
            Ok(Expr::UnaryExpr {
                op: UnaryOp::Deref,
                op_pos: *pos,
                typ: expr.typ().deref().ok_or(AppError {
                    message: "deref できない型を deref しようとしました".to_string(),
                    input: input.to_string(),
                    pos: *pos,
                })?,
                expr: Box::new(expr),
            })
        }
        Some(Token {
            tok: Tok::Ampersand,
            pos,
        }) => {
            tokens.next();
            let expr = parse_unary(context, tokens, input)?;
            Ok(Expr::UnaryExpr {
                op: UnaryOp::Addr,
                op_pos: *pos,
                typ: Type::Ptr(Box::new(expr.typ())),
                expr: Box::new(expr),
            })
        }
        Some(Token {
            tok: Tok::Sizeof,
            pos,
        }) => {
            tokens.next();

            let typ = match tokens.peek() {
                Some(Token {
                    tok: Tok::開き丸括弧,
                    ..
                }) => {
                    tokens.next();
                    let typ = match tokens.peek() {
                        Some(Token { tok: Tok::Int, .. }) => parse_type(tokens, input)?,
                        _ => parse_unary(context, tokens, input)?.typ(),
                    };
                    match tokens.next() {
                        Some(Token {
                            tok: Tok::閉じ丸括弧,
                            ..
                        }) => typ,
                        _ => {
                            return Err(AppError {
                                message: "開き丸括弧に対応する閉じ丸括弧がありません".to_string(),
                                input: input.to_string(),
                                pos: *pos,
                            })
                        }
                    }
                }
                _ => parse_unary(context, tokens, input)?.typ(),
            };

            Ok(Expr::Numeric {
                val: typ.sizeof(),
                pos: *pos,
                typ: Type::Int,
            })
        }
        _ => parse_primary(context, tokens, input),
    }
}

fn parse_multiplicative(
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_unary(context, tokens, input)?;
    loop {
        match tokens.peek() {
            Some(Token {
                tok: Tok::Asterisk,
                pos: op_pos,
            }) => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_unary(context, tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Mul,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Type::Int,
                };
            }
            Some(Token {
                tok: Tok::Div,
                pos: op_pos,
            }) => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_unary(context, tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Div,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Type::Int,
                };
            }

            _ => {
                return Ok(expr);
            }
        }
    }
}

fn add(左辺: Box<Expr>, 右辺: Box<Expr>, op_pos: usize) -> Option<Expr> {
    match (左辺.typ(), 右辺.typ()) {
        (Type::Int, Type::Int) => Some(Expr::BinaryExpr {
            op: BinaryOp::Add,
            op_pos,
            typ: Type::Int,
            左辺,
            右辺,
        }),
        (Type::Ptr(t), Type::Int) => Some(Expr::BinaryExpr {
            op: BinaryOp::Add,
            op_pos,
            左辺,
            右辺: Box::new(Expr::BinaryExpr {
                op: BinaryOp::Mul,
                op_pos,
                左辺: Box::new(Expr::Numeric {
                    val: t.sizeof(),
                    pos: op_pos,
                    typ: Type::Int,
                }),
                右辺,
                typ: Type::Int,
            }),
            typ: Type::Ptr(t),
        }),
        (Type::Int, _) => add(右辺, 左辺, op_pos),
        _ => None,
    }
}

fn subtract(左辺: Box<Expr>, 右辺: Box<Expr>, op_pos: usize) -> Option<Expr> {
    match (左辺.typ(), 右辺.typ()) {
        (Type::Int, Type::Int) => Some(Expr::BinaryExpr {
            op: BinaryOp::Sub,
            op_pos,
            typ: Type::Int,
            左辺,
            右辺,
        }),
        (Type::Ptr(t), Type::Int) => Some(Expr::BinaryExpr {
            op: BinaryOp::Sub,
            op_pos,
            左辺,
            右辺: Box::new(Expr::BinaryExpr {
                op: BinaryOp::Mul,
                op_pos,
                左辺: Box::new(Expr::Numeric {
                    val: t.sizeof(),
                    pos: op_pos,
                    typ: Type::Int,
                }),
                右辺,
                typ: Type::Int,
            }),
            typ: Type::Ptr(t),
        }),
        (Type::Ptr(t1), Type::Ptr(t2)) if t1 == t2 => Some(Expr::BinaryExpr {
            op: BinaryOp::Div,
            op_pos,
            左辺: Box::new(Expr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos,
                左辺,
                右辺,
                typ: Type::Int,
            }),
            右辺: Box::new(Expr::Numeric {
                val: t1.sizeof(),
                pos: op_pos,
                typ: Type::Int,
            }),
            typ: Type::Int,
        }),
        _ => None,
    }
}

fn parse_additive(
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_multiplicative(context, tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::Add,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_multiplicative(context, tokens, input)?);
                let message = format!(
                    "左辺の型が {:?}、右辺の型が {:?} なので、足し合わせることができません",
                    左辺.typ(),
                    右辺.typ()
                );
                expr = add(左辺, 右辺, *op_pos).ok_or(AppError {
                    message,
                    input: input.to_string(),
                    pos: *op_pos,
                })?;
            }
            Token {
                tok: Tok::Sub,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_multiplicative(context, tokens, input)?);
                let message = format!(
                    "左辺の型が {:?}、右辺の型が {:?} なので、引き算できません",
                    左辺.typ(),
                    右辺.typ()
                );

                expr = subtract(左辺, 右辺, *op_pos).ok_or(AppError {
                    message,
                    input: input.to_string(),
                    pos: *op_pos,
                })?;
            }
            _ => {
                return Ok(expr);
            }
        }
    }
}

fn parse_relational(
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_additive(context, tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::LessThan,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(context, tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LessThan,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Type::Int,
                }
            }
            Token {
                tok: Tok::LessThanOrEqual,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(context, tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LessThanOrEqual,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Type::Int,
                }
            }
            Token {
                tok: Tok::GreaterThan,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(context, tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LessThan, // ここを逆転させ、
                    op_pos: *op_pos,
                    左辺: 右辺, // ここを逆転させればよい
                    右辺: 左辺,
                    typ: Type::Int,
                }
            }
            Token {
                tok: Tok::GreaterThanOrEqual,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(context, tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LessThanOrEqual, // ここを逆転させ、
                    op_pos: *op_pos,
                    左辺: 右辺, // ここを逆転させればよい
                    右辺: 左辺,
                    typ: Type::Int,
                }
            }
            _ => {
                return Ok(expr);
            }
        }
    }
}

fn parse_equality(
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_relational(context, tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::Equal,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_relational(context, tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Equal,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Type::Int,
                }
            }
            Token {
                tok: Tok::NotEqual,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_relational(context, tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::NotEqual,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Type::Int,
                }
            }
            _ => {
                return Ok(expr);
            }
        }
    }
}

pub fn parse_expr(
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<Expr, AppError> {
    let expr = parse_equality(context, tokens, input)?;
    let tok = tokens.peek().unwrap();
    match tok {
        Token {
            tok: Tok::Assign,
            pos: op_pos,
        } => {
            tokens.next();
            let 左辺 = Box::new(expr);
            let 右辺 = Box::new(parse_expr(context, tokens, input)?);
            Ok(Expr::BinaryExpr {
                op: BinaryOp::Assign,
                op_pos: *op_pos,
                typ: 左辺.typ(),
                左辺,
                右辺,
            })
        }
        _ => Ok(expr),
    }
}
