use crate::apperror::*;
use crate::ast::*;
use crate::parse::toplevel::FunctionSignature;
use crate::parse::toplevel::StructMember;
use crate::parse::toplevel::TypeAndSize;
use crate::strlit_collector::StrLitCollector;
use crate::token::*;
use std::{iter::Peekable, slice::Iter};

use super::combinator::recover;
use super::combinator::satisfy;
use super::context::Context;
use super::toplevel::SymbolDeclaration;
use super::typ::parse_type;
use super::typ::Type;
fn parse_primary(
    strlit_collector: &mut StrLitCollector,
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
            let id = strlit_collector.insert_and_get_id(val.clone());

            // ビルトイン関数 __builtin_strlit_{id} を呼び出す
            Ok(Expr::Call {
                ident: format!("__builtin_strlit_{id}"),
                args: Vec::new(),
                pos: *pos,
                typ: Type::Arr(Box::new(Type::Char), (val.len() + 1) as i32),
            })

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
                    let func_decl = {
                        if ident.starts_with("__builtin_strlit_") {
                            // 文字列リテラルを召喚するビルトイン関数

                            let id: usize = ident
                                .strip_prefix("__builtin_strlit_")
                                .unwrap()
                                .parse()
                                .unwrap();
                            let string =
                                strlit_collector.search_string_from_id(id).ok_or(AppError {
                                    message: format!("文字列リテラル ID {id} が見つかりません"),
                                    input: input.to_string(),
                                    filename: filename.to_string(),
                                    pos: *ident_pos,
                                })?;

                            FunctionSignature {
                                params: Some(Vec::new()),
                                pos: *ident_pos,
                                return_type: Type::Arr(
                                    Box::new(Type::Char),
                                    (string.len() /* length in bytes */ + 1) as i32,
                                ),
                            }
                        } else {
                            match context.global_declarations.symbols.get(ident) {
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
                            }
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
                    let expr = parse_expr(strlit_collector, context, tokens, filename, input)?;
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
                        let expr = parse_expr(strlit_collector, context, tokens, filename, input)?;
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
                let (local_var_id, TypeAndSize { typ, .. }) =
                    match context.resolve_type_and_size_as_var(ident) {
                        Ok(t) => t.clone(),
                        Err(message) => {
                            return Err(AppError {
                                message,
                                input: input.to_string(),
                                filename: filename.to_string(),
                                pos: *ident_pos,
                            })
                        }
                    };
                let expr = Expr::Identifier {
                    ident: ident.clone(),
                    pos: *ident_pos,
                    local_var_id,
                    typ,
                };
                Ok(expr)
            }
        }
        Token {
            tok: Tok::開き丸括弧,
            ..
        } => {
            let expr = parse_expr(strlit_collector, context, tokens, filename, input)?;
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

fn search_struct_member<'a>(
    context: &'a Context,
    struct_name: &str,
    ident: &str,
    input: &str,
    filename: &str,
    op_pos: usize,
) -> Result<&'a StructMember, AppError> {
    let member = context
        .global_declarations
        .struct_names
        .get(struct_name)
        .and_then(|s| s.members.get(ident))
        .map_or_else(
            || {
                Err(AppError {
                    message: format!("構造体 {struct_name} にフィールド {ident} がありません",),
                    input: input.to_string(),
                    filename: filename.to_string(),
                    pos: op_pos,
                })
            },
            Ok,
        )?;
    Ok(member)
}

fn arrow_expr(op_pos: usize, expr: Expr, offset: i32, typ_of_member: Type) -> Expr {
    Expr::UnaryExpr {
        op: UnaryOp::Deref,
        op_pos,
        expr: Box::new(Expr::BinaryExpr {
            op_pos,
            op: BinaryOp::Add,
            左辺: decay_if_arr(expr),
            右辺: Box::new(Expr::Numeric {
                val: offset,
                pos: op_pos,
                typ: Type::Int,
            }),
            typ: Type::Ptr(Box::new(typ_of_member.clone())),
        }),
        typ: typ_of_member,
    }
}

fn parse_suffix_op(
    strlit_collector: &mut StrLitCollector,
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_primary(strlit_collector, context, tokens, filename, input)?;

    loop {
        match tokens.peek().unwrap() {
            Token {
                tok: Tok::Increment,
                ..
            } => {
                tokens.next();
                let op_pos = tokens.peek().unwrap().pos;

                let message = format!("型が {:?} なので、インクリメントできません", expr.typ(),);

                // a++ can be compiled to ((a += 1) - 1)
                let one = Expr::Numeric {
                    val: 1,
                    pos: op_pos,
                    typ: Type::Int,
                };

                let incremented_expr = add_assign_with_potential_scaling(
                    context,
                    op_pos,
                    Box::new(expr),
                    Box::new(one.clone()),
                );

                expr = subtract_with_potential_scaling_by_sizeof(
                    context,
                    Box::new(incremented_expr),
                    Box::new(one),
                    op_pos,
                )
                .ok_or(AppError {
                    message,
                    input: input.to_string(),
                    filename: filename.to_string(),
                    pos: op_pos,
                })?;
            }

            Token {
                tok: Tok::Decrement,
                ..
            } => {
                tokens.next();
                let op_pos = tokens.peek().unwrap().pos;

                let message = format!("型が {:?} なので、デクリメントできません", expr.typ(),);

                // a-- can be compiled to ((a -= 1) + 1)
                let one = Expr::Numeric {
                    val: 1,
                    pos: op_pos,
                    typ: Type::Int,
                };

                let decremented_expr = sub_assign_with_potential_scaling(
                    context,
                    op_pos,
                    Box::new(expr),
                    Box::new(one.clone()),
                );

                expr = add_with_potential_scaling_by_sizeof(
                    context,
                    Box::new(decremented_expr),
                    Box::new(one),
                    op_pos,
                )
                .ok_or(AppError {
                    message,
                    input: input.to_string(),
                    filename: filename.to_string(),
                    pos: op_pos,
                })?;
            }

            Token {
                tok: Tok::Arrow, ..
            } => {
                tokens.next();
                match tokens.next() {
                    Some(Token {
                        tok: Tok::Identifier(ident),
                        pos,
                    }) => {
                        let op_pos = *pos;
                        let typ_lhs_points_to = match expr.typ() {
                            Type::Ptr(t) => t.clone(),
                            _ => {
                                return Err(AppError {
                                    message: "-> のオペランドがポインタではありません".to_string(),
                                    input: input.to_string(),
                                    filename: filename.to_string(),
                                    pos: op_pos,
                                })
                            }
                        };

                        let Type::Struct { struct_name } = (*typ_lhs_points_to).clone() else {
                            return Err(AppError {
                                message: "-> のオペランドが構造体へのポインタではありません"
                                    .to_string(),
                                input: input.to_string(),
                                filename: filename.to_string(),
                                pos: op_pos,
                            });
                        };

                        // ptr->ident is *(((char *)ptr + offsetof(struct, ident)))
                        // and its type is the type of struct_name.ident
                        let member = search_struct_member(
                            context,
                            &struct_name,
                            ident,
                            input,
                            filename,
                            op_pos,
                        )?;

                        let typ_of_member = member.member_type.clone();
                        let offset = member.offset;

                        expr = arrow_expr(op_pos, expr, offset, typ_of_member);
                    }
                    _ => {
                        return Err(AppError {
                            message: "-> の右側には識別子が必要です".to_string(),
                            input: input.to_string(),
                            filename: filename.to_string(),
                            pos: tokens.peek().unwrap().pos,
                        });
                    }
                }
            }

            Token { tok: Tok::Dot, .. } => {
                tokens.next();
                match tokens.next() {
                    Some(Token {
                        tok: Tok::Identifier(ident),
                        pos,
                    }) => {
                        let op_pos = *pos;

                        let Type::Struct { struct_name } = expr.typ().clone() else {
                            return Err(AppError {
                                message: ". のオペランドが構造体ではありません".to_string(),
                                input: input.to_string(),
                                filename: filename.to_string(),
                                pos: op_pos,
                            });
                        };

                        // ptr->ident is *(((char *)ptr + offsetof(struct, ident)))
                        // and its type is the type of struct_name.ident
                        let member = search_struct_member(
                            context,
                            &struct_name,
                            ident,
                            input,
                            filename,
                            op_pos,
                        )?;

                        let typ_of_member = member.member_type.clone();
                        let offset = member.offset;

                        let ptr = Expr::UnaryExpr {
                            op: UnaryOp::Addr,
                            op_pos,
                            typ: Type::Ptr(Box::new(typ_of_member.clone())),
                            expr: Box::new(expr),
                        };

                        expr = arrow_expr(op_pos, ptr, offset, typ_of_member);
                    }
                    _ => {
                        return Err(AppError {
                            message: ". の右側には識別子が必要です".to_string(),
                            input: input.to_string(),
                            filename: filename.to_string(),
                            pos: tokens.peek().unwrap().pos,
                        });
                    }
                }
            }

            Token {
                tok: Tok::開き角括弧,
                ..
            } => {
                tokens.next();
                let 右辺 = parse_expr(strlit_collector, context, tokens, filename, input)?;
                let op_pos = tokens.peek().unwrap().pos;
                satisfy(
                    tokens,
                    filename,
                    input,
                    |tok| tok == &Tok::閉じ角括弧,
                    "この開き角括弧に対応する閉じ角括弧がありません",
                )?;
                let 左辺 = decay_if_arr(expr);
                let typ = match 左辺.typ() {
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
                    expr: Box::new(
                        add_with_potential_scaling_by_sizeof(context, 左辺, Box::new(右辺), op_pos)
                            .unwrap(),
                    ),
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
    strlit_collector: &mut StrLitCollector,
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    match tokens.peek() {
        Some(Token { tok: Tok::Add, pos }) => {
            tokens.next();
            let expr = parse_suffix_op(strlit_collector, context, tokens, filename, input)?;
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
            let expr = parse_suffix_op(strlit_collector, context, tokens, filename, input)?;
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
            let expr = parse_suffix_op(strlit_collector, context, tokens, filename, input)?;

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
            let expr = parse_unary(strlit_collector, context, tokens, filename, input)?;
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
            let expr = parse_unary(strlit_collector, context, tokens, filename, input)?;
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
            let expr = parse_unary(strlit_collector, context, tokens, filename, input)?;
            let one = Expr::Numeric {
                val: 1,
                pos: *pos,
                typ: Type::Int,
            };
            Ok(add_assign_with_potential_scaling(
                context,
                *pos,
                Box::new(expr),
                Box::new(one.clone()),
            ))
        }
        Some(Token {
            tok: Tok::Decrement,
            pos,
        }) => {
            tokens.next();
            let expr = parse_unary(strlit_collector, context, tokens, filename, input)?;
            let one = Expr::Numeric {
                val: 1,
                pos: *pos,
                typ: Type::Int,
            };

            Ok(sub_assign_with_potential_scaling(
                context,
                *pos,
                Box::new(expr),
                Box::new(one.clone()),
            ))
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
                    parse_expr(strlit_collector, context, tokens, filename, input)?.typ()
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
                parse_unary(strlit_collector, context, tokens, filename, input)?.typ()
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
                    parse_expr(strlit_collector, context, tokens, filename, input)?.typ()
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
                parse_unary(strlit_collector, context, tokens, filename, input)?.typ()
            };

            Ok(Expr::Numeric {
                val: typ.alignof(&context.global_declarations.struct_names),
                pos: *pos,
                typ: Type::Int,
            })
        }
        _ => parse_suffix_op(strlit_collector, context, tokens, filename, input),
    }
}

fn parse_multiplicative(
    strlit_collector: &mut StrLitCollector,
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_unary(strlit_collector, context, tokens, filename, input)?;
    loop {
        match tokens.peek() {
            Some(Token {
                tok: Tok::Asterisk,
                pos: op_pos,
            }) => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_unary(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
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
                let 右辺 = decay_if_arr(parse_unary(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Div,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                    typ: Type::Int,
                };
            }
            Some(Token {
                tok: Tok::Percent,
                pos: op_pos,
            }) => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_unary(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Remainder,
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

fn add_with_potential_scaling_by_sizeof(
    context: &Context,
    左辺: Box<Expr>,
    右辺: Box<Expr>,
    op_pos: usize,
) -> Option<Expr> {
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
        (Type::Int, _) => add_with_potential_scaling_by_sizeof(context, 右辺, 左辺, op_pos),
        _ => None,
    }
}

fn subtract_with_potential_scaling_by_sizeof(
    context: &Context,
    左辺: Box<Expr>,
    右辺: Box<Expr>,
    op_pos: usize,
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
    strlit_collector: &mut StrLitCollector,
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_multiplicative(strlit_collector, context, tokens, filename, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::Add,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_multiplicative(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
                let message = format!(
                    "左辺の型が {:?}、右辺の型が {:?} なので、足し合わせることができません",
                    左辺.typ(),
                    右辺.typ()
                );
                expr = add_with_potential_scaling_by_sizeof(context, 左辺, 右辺, *op_pos).ok_or(
                    AppError {
                        message,
                        input: input.to_string(),
                        filename: filename.to_string(),
                        pos: *op_pos,
                    },
                )?;
            }
            Token {
                tok: Tok::Sub,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_multiplicative(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
                let message = format!(
                    "左辺の型が {:?}、右辺の型が {:?} なので、引き算できません",
                    左辺.typ(),
                    右辺.typ()
                );

                expr = subtract_with_potential_scaling_by_sizeof(context, 左辺, 右辺, *op_pos)
                    .ok_or(AppError {
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
    strlit_collector: &mut StrLitCollector,
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_additive(strlit_collector, context, tokens, filename, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::LessThan,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_additive(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
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
                let 右辺 = decay_if_arr(parse_additive(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
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
                let 右辺 = decay_if_arr(parse_additive(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
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
                let 右辺 = decay_if_arr(parse_additive(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
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
    strlit_collector: &mut StrLitCollector,
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_relational(strlit_collector, context, tokens, filename, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::Equal,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_relational(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
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
                let 右辺 = decay_if_arr(parse_relational(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
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

fn parse_logical_and(
    strlit_collector: &mut StrLitCollector,
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_equality(strlit_collector, context, tokens, filename, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::LogicalAnd,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_equality(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LogicalAnd,
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

fn parse_logical_or(
    strlit_collector: &mut StrLitCollector,
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let mut expr = parse_logical_and(strlit_collector, context, tokens, filename, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::LogicalOr,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = decay_if_arr(expr);
                let 右辺 = decay_if_arr(parse_logical_and(
                    strlit_collector,
                    context,
                    tokens,
                    filename,
                    input,
                )?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LogicalOr,
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
    strlit_collector: &mut StrLitCollector,
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Expr, AppError> {
    let expr = parse_logical_or(strlit_collector, context, tokens, filename, input)?;
    let tok = tokens.peek().unwrap();
    match tok {
        Token {
            tok: Tok::Assign,
            pos: op_pos,
        } => {
            tokens.next();
            let 左辺 = decay_if_arr(expr);
            let 右辺 = decay_if_arr(parse_expr(
                strlit_collector,
                context,
                tokens,
                filename,
                input,
            )?);

            // special case for assigning 0 to a pointer
            if let Type::Ptr(_) = 左辺.typ() {
                if let Expr::Numeric { val, pos, .. } = &*右辺 {
                    if *val == 0 {
                        return Ok(Expr::NullPtr {
                            pos: *pos,
                            typ: 左辺.typ(),
                        });
                    }
                }
            }

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
            let 右辺 = decay_if_arr(parse_expr(
                strlit_collector,
                context,
                tokens,
                filename,
                input,
            )?);

            Ok(add_assign_with_potential_scaling(
                context, *op_pos, 左辺, 右辺,
            ))
        }
        Token {
            tok: Tok::SubAssign,
            pos: op_pos,
        } => {
            tokens.next();
            let 左辺 = decay_if_arr(expr);
            let 右辺 = decay_if_arr(parse_expr(
                strlit_collector,
                context,
                tokens,
                filename,
                input,
            )?);

            Ok(sub_assign_with_potential_scaling(
                context, *op_pos, 左辺, 右辺,
            ))
        }
        _ => Ok(expr),
    }
}

fn add_assign_with_potential_scaling(
    context: &Context,
    op_pos: usize,
    左辺: Box<Expr>,
    右辺: Box<Expr>,
) -> Expr {
    let scaled_右辺 = match 左辺.typ() {
        Type::Ptr(t) => Box::new(Expr::BinaryExpr {
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
        _ => 右辺,
    };

    Expr::BinaryExpr {
        op: BinaryOp::AddAssign,
        op_pos,
        typ: 左辺.typ(),
        左辺,
        右辺: scaled_右辺,
    }
}

fn sub_assign_with_potential_scaling(
    context: &Context,
    op_pos: usize,
    左辺: Box<Expr>,
    右辺: Box<Expr>,
) -> Expr {
    let scaled_右辺 = match 左辺.typ() {
        Type::Ptr(t) => Box::new(Expr::BinaryExpr {
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
        _ => 右辺,
    };

    Expr::BinaryExpr {
        op: BinaryOp::SubAssign,
        op_pos,
        typ: 左辺.typ(),
        左辺,
        右辺: scaled_右辺,
    }
}
