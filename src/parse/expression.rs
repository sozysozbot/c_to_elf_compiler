use crate::apperror::*;
use crate::ast::*;
use crate::token::*;
use std::{iter::Peekable, slice::Iter};

use super::combinator::recover;
use super::combinator::satisfy;
use super::toplevel::Context;
use super::toplevel::SymbolDeclaration;
use super::toplevel::Type;
use super::typ::parse_type;
fn parse_primary(
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
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
            tok: Tok::StringLiteral(val),
            pos,
        } => {
            let expr = Expr::StringLiteral {
                val: val.clone(),
                pos: *pos,
                typ: Type::Arr(Box::new(Type::Char), (val.len() + 1).try_into().unwrap()),
            };
            Ok(expr)
        }
        Token {
            tok: Tok::Identifier(ident),
            pos: ident_pos,
        } => {
            /*
            関数呼び出しはsuffix opとして処理したいが、`識別子(...)` の形に限られるかつ、関数呼び出しか変数参照かで `識別子` の意味が大きく異なる為ここで処理する
            構文解析と意味解析が分かれていれば問題ないが、ここで `識別子` の型を決めなければいけないため
            */
            let open_pos = tokens.peek().unwrap().pos;
            if (recover(tokens, |tokens| {
                satisfy(
                    tokens,
                    filename,
                    input,
                    |tok| tok == &Tok::開き丸括弧,
                    "開き丸括弧ではありません",
                )
            })?)
            .is_some()
            {
                let mut args = Vec::new();

                if (recover(tokens, |tokens| {
                    satisfy(
                        tokens,
                        filename,
                        input,
                        |tok| tok == &Tok::閉じ丸括弧,
                        "閉じ丸括弧ではありません",
                    )
                })?)
                .is_some()
                {
                    let func_decl = match context.global_declarations.symbols.get(ident) {
                        Some(SymbolDeclaration::Func(f)) => f.clone(),
                        Some(SymbolDeclaration::GVar(_)) => {
                            return Err(AppError {
                                message: format!(
                                    "{ident} は関数ではなくグローバル変数であり、呼び出せません",
                                ),
                                input: input.to_string(),
                                filename: filename.to_string(),
                                pos: *ident_pos,
                            })
                        }
                        None => {
                            return Err(AppError {
                                message: format!(
                                    "関数 {ident} は宣言されておらず、戻り値の型が分かりません",
                                ),
                                input: input.to_string(),
                                filename: filename.to_string(),
                                pos: *ident_pos,
                            })
                        }
                    };
                    let expr = Expr::Call {
                        ident: ident.clone(),
                        args,
                        pos: *ident_pos,
                        typ: func_decl.return_type,
                    };
                    return Ok(expr);
                } else {
                    let expr = parse_expr(context, tokens, filename, input)?;
                    args.push(*decay_if_arr(expr));
                }

                loop {
                    if (recover(tokens, |tokens| {
                        satisfy(
                            tokens,
                            filename,
                            input,
                            |tok| tok == &Tok::閉じ丸括弧,
                            "閉じ丸括弧ではありません",
                        )
                    })?)
                    .is_some()
                    {
                        let func_decl = match context.global_declarations.symbols.get(ident) {
                            Some(SymbolDeclaration::Func(f)) => f.clone(),
                            Some(SymbolDeclaration::GVar(_)) => {
                                return Err(AppError {
                                    message: format!(
                                    "{ident} は関数ではなくグローバル変数であり、呼び出せません",
                                ),
                                    input: input.to_string(),
                                    filename: filename.to_string(),
                                    pos: *ident_pos,
                                })
                            }
                            None => {
                                return Err(AppError {
                                    message: format!(
                                        "関数 {ident} は宣言されておらず、戻り値の型が分かりません",
                                    ),
                                    input: input.to_string(),
                                    filename: filename.to_string(),
                                    pos: *ident_pos,
                                })
                            }
                        };
                        let expr = Expr::Call {
                            ident: ident.clone(),
                            args,
                            pos: *ident_pos,
                            typ: func_decl.return_type,
                        };
                        break Ok(expr);
                    } else if (recover(tokens, |tokens| {
                        satisfy(
                            tokens,
                            filename,
                            input,
                            |tok| tok == &Tok::Comma,
                            "カンマではありません",
                        )
                    })?)
                    .is_some()
                    {
                        let expr = parse_expr(context, tokens, filename, input)?;
                        args.push(*decay_if_arr(expr));
                    } else {
                        break Err(AppError {
                            message: "閉じ丸括弧かカンマが期待されていました".to_string(),
                            input: input.to_string(),
                            filename: filename.to_string(),
                            pos: open_pos + 1,
                        });
                    }
                }
            } else {
                let typ = match context.local_var_and_param_declarations.get(ident) {
                    Some(t) => t.clone(),
                    None => match context.global_declarations.symbols.get(ident) {
                        Some(SymbolDeclaration::GVar(t)) => t.clone(),
                        Some(SymbolDeclaration::Func(_u)) => Err(AppError {
                            message: format!(
                                "識別子 {ident} は関数であり、現在関数ポインタは実装されていません",
                            ),
                            input: input.to_string(),
                            filename: filename.to_string(),
                            pos: *ident_pos,
                        })?,
                        None => Err(AppError {
                            message: format!("識別子 {ident} は定義されておらず、型が分かりません",),
                            input: input.to_string(),
                            filename: filename.to_string(),
                            pos: *ident_pos,
                        })?,
                    },
                };
                let expr = Expr::Identifier {
                    ident: ident.clone(),
                    pos: *ident_pos,
                    typ,
                };
                Ok(expr)
            }
        }
        Token {
            tok: Tok::開き丸括弧,
            ..
        } => {
            let expr = parse_expr(context, tokens, filename, input)?;
            satisfy(
                tokens,
                filename,
                input,
                |tok| tok == &Tok::閉じ丸括弧,
                "この開き丸括弧に対応する閉じ丸括弧がありません",
            )?;
            Ok(expr)
        }
        tok => Err(AppError {
            message: "数値リテラルでも開き丸括弧でもないものが来ました".to_string(),
            input: input.to_string(),
            filename: filename.to_string(),
            pos: tok.pos,
        }),
    }
}

fn parse_suffix_op(
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_primary(context, tokens, filename, input)?;

    loop {
        match tokens.peek().unwrap() {
            Token {
                tok: Tok::Increment,
                ..
            } => {
                tokens.next();
                let op_pos = tokens.peek().unwrap().pos;

                let message = format!("型が {:?} なので、インクリメントできません", expr.typ(),);

                // a++ can be compiled to ((++a) - 1)

                let incremented_expr = Expr::UnaryExpr {
                    op: UnaryOp::Increment,
                    op_pos,
                    typ: expr.typ(),
                    expr: Box::new(expr),
                };

                let one = Expr::Numeric {
                    val: 1,
                    pos: op_pos,
                    typ: Type::Int,
                };

                expr = subtract(context, Box::new(incremented_expr), Box::new(one), op_pos).ok_or(
                    AppError {
                        message,
                        input: input.to_string(),
                        filename: filename.to_string(),
                        pos: op_pos,
                    },
                )?;
            }

            Token {
                tok: Tok::Decrement,
                ..
            } => {
                tokens.next();
                let op_pos = tokens.peek().unwrap().pos;

                let message = format!("型が {:?} なので、デクリメントできません", expr.typ(),);

                // a-- can be compiled to ((--a) + 1)

                let decremented_expr = Expr::UnaryExpr {
                    op: UnaryOp::Decrement,
                    op_pos,
                    typ: expr.typ(),
                    expr: Box::new(expr),
                };

                let one = Expr::Numeric {
                    val: 1,
                    pos: op_pos,
                    typ: Type::Int,
                };

                expr = add(context, Box::new(decremented_expr), Box::new(one), op_pos).ok_or(
                    AppError {
                        message,
                        input: input.to_string(),
                        filename: filename.to_string(),
                        pos: op_pos,
                    },
                )?;
            }

            Token {
                tok: Tok::開き角括弧,
                ..
            } => {
                tokens.next();
                let index = parse_expr(context, tokens, filename, input)?;
                let op_pos = tokens.peek().unwrap().pos;
                satisfy(
                    tokens,
                    filename,
                    input,
                    |tok| tok == &Tok::閉じ角括弧,
                    "この開き角括弧に対応する閉じ角括弧がありません",
                )?;
                let arr = decay_if_arr(expr);
                let typ = match arr.typ() {
                    Type::Ptr(element_typ) => *element_typ,
                    _ => {
                        return Err(AppError {
                            message: "ポインタではありません".to_string(),
                            input: input.to_string(),
                            filename: filename.to_string(),
                            pos: op_pos,
                        })
                    }
                };
                expr = Expr::UnaryExpr {
                    op_pos,
                    op: UnaryOp::Deref,
                    expr: Box::new(Expr::BinaryExpr {
                        op_pos,
                        op: BinaryOp::Add,
                        左辺: arr,
                        右辺: decay_if_arr(index),
                        typ: typ.clone(),
                    }),
                    typ,
                };
            }
            _ => {
                return Ok(expr);
            }
        }
    }
}

fn parse_unary(
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    match tokens.peek() {
        Some(Token { tok: Tok::Add, pos }) => {
            tokens.next();
            let expr = parse_suffix_op(context, tokens, filename, input)?;
            Ok(Expr::BinaryExpr {
                op: BinaryOp::Add,
                op_pos: *pos,
                typ: Type::Int,
                左辺: decay_if_arr(Expr::Numeric {
                    val: 0,
                    pos: *pos,
                    typ: Type::Int,
                }),
                右辺: decay_if_arr(expr),
            })
        }
        Some(Token { tok: Tok::Sub, pos }) => {
            tokens.next();
            let expr = parse_suffix_op(context, tokens, filename, input)?;
            Ok(Expr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos: *pos,
                typ: Type::Int,
                左辺: decay_if_arr(Expr::Numeric {
                    val: 0,
                    pos: *pos,
                    typ: Type::Int,
                }),
                右辺: decay_if_arr(expr),
            })
        }
        Some(Token {
            tok: Tok::LogicalNot,
            pos,
        }) => {
            tokens.next();
            let expr = parse_suffix_op(context, tokens, filename, input)?;

            // The expression !E is equivalent to (0==E)
            // オペランドがポインタなら比較対象はヌルポインタ定数

            let zero = if let Type::Ptr(_) = expr.typ() {
                Expr::NullPtr {
                    pos: *pos,
                    typ: expr.typ(),
                }
            } else {
                Expr::Numeric {
                    val: 0,
                    pos: *pos,
                    typ: Type::Int,
                }
            };

            Ok(Expr::BinaryExpr {
                op: BinaryOp::Equal,
                op_pos: *pos,
                左辺: Box::new(zero),
                右辺: decay_if_arr(expr),
                typ: Type::Int,
            })
        }
        Some(Token {
            tok: Tok::Asterisk,
            pos,
        }) => {
            tokens.next();
            let expr = parse_unary(context, tokens, filename, input)?;
            Ok(Expr::UnaryExpr {
                op: UnaryOp::Deref,
                op_pos: *pos,
                typ: expr.typ().deref().ok_or(AppError {
                    message: "deref できない型を deref しようとしました".to_string(),
                    input: input.to_string(),
                    filename: filename.to_string(),
                    pos: *pos,
                })?,
                expr: decay_if_arr(expr),
            })
        }
        Some(Token {
            tok: Tok::Ampersand,
            pos,
        }) => {
            tokens.next();
            let expr = parse_unary(context, tokens, filename, input)?;
            Ok(Expr::UnaryExpr {
                op: UnaryOp::Addr,
                op_pos: *pos,
                typ: Type::Ptr(Box::new(expr.typ())),
                expr: no_decay_even_if_arr(expr),
            })
        }
        Some(Token {
            tok: Tok::Increment,
            pos,
        }) => {
            tokens.next();
            let expr = parse_unary(context, tokens, filename, input)?;
            Ok(Expr::UnaryExpr {
                op: UnaryOp::Increment,
                op_pos: *pos,
                typ: expr.typ(),
                expr: throw_if_arr(expr), // 配列型に ++ されることはあり得ない
            })
        }
        Some(Token {
            tok: Tok::Decrement,
            pos,
        }) => {
            tokens.next();
            let expr = parse_unary(context, tokens, filename, input)?;
            Ok(Expr::UnaryExpr {
                op: UnaryOp::Decrement,
                op_pos: *pos,
                typ: expr.typ(),
                expr: throw_if_arr(expr), // 配列型に -- されることはあり得ない
            })
        }
        Some(Token {
            tok: Tok::Sizeof,
            pos,
        }) => {
            tokens.next();

            let typ = if (recover(tokens, |tokens| {
                satisfy(
                    tokens,
                    filename,
                    input,
                    |tok| tok == &Tok::開き丸括弧,
                    "開き丸括弧ではありません",
                )
            })?)
            .is_some()
            {
                let typ = if let Some(typ) =
                    recover(tokens, |tokens| parse_type(tokens, filename, input))?
                {
                    typ
                } else {
                    parse_expr(context, tokens, filename, input)?.typ()
                };
                satisfy(
                    tokens,
                    filename,
                    input,
                    |tok| tok == &Tok::閉じ丸括弧,
                    "開き丸括弧に対応する閉じ丸括弧がありません",
                )?;
                typ
            } else {
                parse_unary(context, tokens, filename, input)?.typ()
            };

            Ok(Expr::Numeric {
                val: typ.sizeof(&context.global_declarations.struct_names),
                pos: *pos,
                typ: Type::Int,
            })
        }
        Some(Token {
            tok: Tok::Alignof,
            pos,
        }) => {
            tokens.next();

            let typ = if (recover(tokens, |tokens| {
                satisfy(
                    tokens,
                    filename,
                    input,
                    |tok| tok == &Tok::開き丸括弧,
                    "開き丸括弧ではありません",
                )
            })?)
            .is_some()
            {
                let typ = if let Some(typ) =
                    recover(tokens, |tokens| parse_type(tokens, filename, input))?
                {
                    typ
                } else {
                    // The use of _Alignof with expressions is allowed by some C compilers as a non-standard extension.
                    parse_expr(context, tokens, filename, input)?.typ()
                };
                satisfy(
                    tokens,
                    filename,
                    input,
                    |tok| tok == &Tok::閉じ丸括弧,
                    "開き丸括弧に対応する閉じ丸括弧がありません",
                )?;
                typ
            } else {
                parse_unary(context, tokens, filename, input)?.typ()
            };

            Ok(Expr::Numeric {
                val: typ.alignof(&context.global_declarations.struct_names),
                pos: *pos,
                typ: Type::Int,
            })
        }
        _ => parse_suffix_op(context, tokens, filename, input),
    }
}

fn parse_multiplicative(
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_unary(context, tokens, filename, input)?;
    loop {
        match tokens.peek() {
            Some(Token {
                tok: Tok::Asterisk,
                pos: op_pos,
            }) => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_unary(context, tokens, filename, input)?);
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
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_unary(context, tokens, filename, input)?);
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

fn add(context: &Context, 左辺: Box<Expr>, 右辺: Box<Expr>, op_pos: usize) -> Option<Expr> {
    match (左辺.typ(), 右辺.typ()) {
        (Type::Int | Type::Char, Type::Int | Type::Char) => Some(Expr::BinaryExpr {
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
            右辺: decay_if_arr(Expr::BinaryExpr {
                op: BinaryOp::Mul,
                op_pos,
                左辺: decay_if_arr(Expr::Numeric {
                    val: t.sizeof(&context.global_declarations.struct_names),
                    pos: op_pos,
                    typ: Type::Int,
                }),
                右辺,
                typ: Type::Int,
            }),
            typ: Type::Ptr(t),
        }),
        (Type::Int, _) => add(context, 右辺, 左辺, op_pos),
        _ => None,
    }
}

fn subtract(
    context: &Context, 左辺: Box<Expr>, 右辺: Box<Expr>, op_pos: usize
) -> Option<Expr> {
    match (左辺.typ(), 右辺.typ()) {
        (Type::Int | Type::Char, Type::Int | Type::Char) => Some(Expr::BinaryExpr {
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
            右辺: decay_if_arr(Expr::BinaryExpr {
                op: BinaryOp::Mul,
                op_pos,
                左辺: decay_if_arr(Expr::Numeric {
                    val: t.sizeof(&context.global_declarations.struct_names),
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
            左辺: decay_if_arr(Expr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos,
                左辺,
                右辺,
                typ: Type::Int,
            }),
            右辺: decay_if_arr(Expr::Numeric {
                val: t1.sizeof(&context.global_declarations.struct_names),
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
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_multiplicative(context, tokens, filename, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::Add,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_multiplicative(context, tokens, filename, input)?);
                let message = format!(
                    "左辺の型が {:?}、右辺の型が {:?} なので、足し合わせることができません",
                    左辺.typ(),
                    右辺.typ()
                );
                expr = add(context, 左辺, 右辺, *op_pos).ok_or(AppError {
                    message,
                    input: input.to_string(),
                    filename: filename.to_string(),
                    pos: *op_pos,
                })?;
            }
            Token {
                tok: Tok::Sub,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_multiplicative(context, tokens, filename, input)?);
                let message = format!(
                    "左辺の型が {:?}、右辺の型が {:?} なので、引き算できません",
                    左辺.typ(),
                    右辺.typ()
                );

                expr = subtract(context, 左辺, 右辺, *op_pos).ok_or(AppError {
                    message,
                    input: input.to_string(),
                    filename: filename.to_string(),
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
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_additive(context, tokens, filename, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::LessThan,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_additive(context, tokens, filename, input)?);
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
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_additive(context, tokens, filename, input)?);
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
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_additive(context, tokens, filename, input)?);
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
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_additive(context, tokens, filename, input)?);
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
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_relational(context, tokens, filename, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::Equal,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_relational(context, tokens, filename, input)?);
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
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_relational(context, tokens, filename, input)?);
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
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let expr = parse_equality(context, tokens, filename, input)?;
    let tok = tokens.peek().unwrap();
    match tok {
        Token {
            tok: Tok::Assign,
            pos: op_pos,
        } => {
            tokens.next();
            let 左辺 = decay_if_arr(expr);
            let 右辺 = decay_if_arr(parse_expr(context, tokens, filename, input)?);
            Ok(Expr::BinaryExpr {
                op: BinaryOp::Assign,
                op_pos: *op_pos,
                typ: 左辺.typ(),
                左辺,
                右辺,
            })
        }
        Token {
            tok: Tok::AddAssign,
            pos: op_pos,
        } => {
            tokens.next();
            let 左辺 = decay_if_arr(expr);
            let 右辺 = decay_if_arr(parse_expr(context, tokens, filename, input)?);
            Ok(Expr::BinaryExpr {
                op: BinaryOp::AddAssign,
                op_pos: *op_pos,
                typ: 左辺.typ(),
                左辺,
                右辺,
            })
        }
        Token {
            tok: Tok::SubAssign,
            pos: op_pos,
        } => {
            tokens.next();
            let 左辺 = decay_if_arr(expr);
            let 右辺 = decay_if_arr(parse_expr(context, tokens, filename, input)?);
            Ok(Expr::BinaryExpr {
                op: BinaryOp::SubAssign,
                op_pos: *op_pos,
                typ: 左辺.typ(),
                左辺,
                右辺,
            })
        }
        _ => Ok(expr),
    }
}
