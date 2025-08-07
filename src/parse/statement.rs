use crate::apperror::*;
use crate::ast::*;
use crate::parse::combinator::recover;
use crate::parse::toplevel::Context;
use crate::parse::toplevel::TypeAndSize;
use crate::parse::typ::Type;
use crate::token::*;
use std::{iter::Peekable, slice::Iter};

use super::combinator::satisfy;
use super::expression::parse_expr;
use super::typ::parse_type;

#[test]
fn parse_test() {
    use crate::parse::toplevel::GlobalDeclarations;
    use crate::parse::typ::Type;
    use crate::tokenize::tokenize;
    use std::collections::HashMap;
    let input = "5 - 3;";
    let tokens = tokenize(input, "test.c").unwrap();
    let mut tokens = tokens.iter().peekable();
    assert_eq!(
        parse_statement(
            &mut Context::new(
                HashMap::new(),
                GlobalDeclarations {
                    symbols: HashMap::new(),
                    struct_names: HashMap::new()
                },
            ),
            &mut tokens,
            "test.c",
            input
        )
        .unwrap(),
        Statement::Expr {
            expr: Box::new(Expr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos: 2,
                typ: Type::Int,
                左辺: decay_if_arr(Expr::Numeric {
                    val: 5,
                    pos: 0,
                    typ: Type::Int
                }),
                右辺: decay_if_arr(Expr::Numeric {
                    val: 3,
                    pos: 4,
                    typ: Type::Int
                })
            }),
            semicolon_pos: 5
        }
    );
}

pub fn parse_statement_or_declaration(
    context: &mut Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<StatementOrDeclaration, AppError> {
    if let Some((local_var_type, local_var_name)) = recover(tokens, |tokens| {
        parse_type_and_identifier(tokens, filename, input)
    })? {
        match tokens.peek().unwrap() {
            Token {
                tok: Tok::Semicolon,
                ..
            } => {
                tokens.next();

                let typ_and_size = TypeAndSize {
                    typ: local_var_type.clone(),
                    size: local_var_type.sizeof(&context.global_declarations.struct_names),
                };

                context.insert_local_var(local_var_name.clone(), typ_and_size.clone());
                Ok(StatementOrDeclaration::Declaration {
                    name: local_var_name,
                    typ_and_size,
                })
            }
            Token { pos, .. } => {
                Err(AppError {
                    message: "関数内の変数宣言で、型名と識別子の後にセミコロン以外が来ました"
                        .to_string(),
                    input: input.to_string(),
                    filename: filename.to_string(),
                    pos: *pos,
                })
            }
        }
    } else {
        parse_statement(context, tokens, filename, input).map(StatementOrDeclaration::Statement)
    }
}

fn parse_statement(
    context: &mut Context,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Statement, AppError> {
    let tok = tokens.peek().unwrap();
    match tok {
        Token {
            tok: Tok::Throw, ..
        } => {
            tokens.next();
            let expr = Box::new(parse_expr(context, tokens, filename, input)?);
            let tok = tokens.peek().unwrap();
            let semicolon_pos = match tok {
                Token {
                    tok: Tok::Semicolon,
                    pos,
                } => {
                    tokens.next();
                    *pos
                }
                _ => {
                    return Err(AppError {
                        message: "期待されたセミコロンが来ませんでした".to_string(),
                        input: input.to_string(),
                        filename: filename.to_string(),
                        pos: tok.pos,
                    })
                }
            };
            Ok(Statement::Throw {
                semicolon_pos,
                expr,
            })
        }
        Token {
            tok: Tok::Return, ..
        } => {
            tokens.next();
            let expr = Box::new(parse_expr(context, tokens, filename, input)?);
            let tok = tokens.peek().unwrap();
            let semicolon_pos = match tok {
                Token {
                    tok: Tok::Semicolon,
                    pos,
                } => {
                    tokens.next();
                    *pos
                }
                _ => {
                    return Err(AppError {
                        message: "期待されたセミコロンが来ませんでした".to_string(),
                        input: input.to_string(),
                        filename: filename.to_string(),
                        pos: tok.pos,
                    })
                }
            };
            Ok(Statement::Return {
                semicolon_pos,
                expr,
            })
        }
        Token { tok: Tok::If, pos } => {
            tokens.next();
            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    tok: Tok::開き丸括弧,
                    ..
                } => {
                    tokens.next();
                }
                _ => {
                    return Err(AppError {
                        message: "期待された開き括弧が来ませんでした".to_string(),
                        input: input.to_string(),
                        filename: filename.to_string(),
                        pos: tok.pos,
                    })
                }
            }
            let cond = Box::new(parse_expr(context, tokens, filename, input)?);

            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    tok: Tok::閉じ丸括弧,
                    ..
                } => {
                    tokens.next();
                }
                _ => {
                    return Err(AppError {
                        message: "期待された閉じ括弧が来ませんでした".to_string(),
                        input: input.to_string(),
                        filename: filename.to_string(),
                        pos: tok.pos,
                    })
                }
            }
            let then = Box::new(parse_statement_or_declaration(
                context, tokens, filename, input,
            )?);
            let tok = tokens.peek().unwrap();
            let else_ = match tok {
                Token { tok: Tok::Else, .. } => {
                    tokens.next();
                    Some(Box::new(parse_statement_or_declaration(
                        context, tokens, filename, input,
                    )?))
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
            tok: Tok::While,
            pos,
        } => {
            tokens.next();
            satisfy(
                tokens,
                filename,
                input,
                |tok| tok == &Tok::開き丸括弧,
                "期待された開き括弧が来ませんでした",
            )?;
            let cond = Box::new(parse_expr(context, tokens, filename, input)?);

            satisfy(
                tokens,
                filename,
                input,
                |tok| tok == &Tok::閉じ丸括弧,
                "期待された閉じ括弧が来ませんでした",
            )?;
            let body = Box::new(parse_statement_or_declaration(
                context, tokens, filename, input,
            )?);
            Ok(Statement::While {
                cond,
                body,
                pos: *pos,
            })
        }
        Token { tok: Tok::For, pos } => {
            tokens.next();
            satisfy(
                tokens,
                filename,
                input,
                |tok| tok == &Tok::開き丸括弧,
                "期待された開き括弧が来ませんでした",
            )?;
            let tok = tokens.peek().unwrap();
            let init = match tok {
                Token {
                    tok: Tok::Semicolon,
                    ..
                } => None,
                _ => Some(Box::new(parse_expr(context, tokens, filename, input)?)),
            };
            satisfy(
                tokens,
                filename,
                input,
                |tok| tok == &Tok::Semicolon,
                "期待されたセミコロンが来ませんでした",
            )?;
            let tok = tokens.peek().unwrap();
            let cond = match tok {
                Token {
                    tok: Tok::Semicolon,
                    ..
                } => None,
                _ => Some(Box::new(parse_expr(context, tokens, filename, input)?)),
            };
            satisfy(
                tokens,
                filename,
                input,
                |tok| tok == &Tok::Semicolon,
                "期待されたセミコロンが来ませんでした",
            )?;
            let tok = tokens.peek().unwrap();
            let update = match tok {
                Token {
                    tok: Tok::閉じ丸括弧,
                    ..
                } => None,
                _ => Some(Box::new(parse_expr(context, tokens, filename, input)?)),
            };
            satisfy(
                tokens,
                filename,
                input,
                |tok| tok == &Tok::閉じ丸括弧,
                "期待された閉じ括弧が来ませんでした",
            )?;
            let body = Box::new(parse_statement_or_declaration(
                context, tokens, filename, input,
            )?);
            Ok(Statement::For {
                init,
                cond,
                update,
                body,
                pos: *pos,
            })
        }
        Token {
            tok: Tok::開き波括弧,
            pos,
        } => {
            tokens.next();
            let mut statements = vec![];
            loop {
                match tokens.peek() {
                    None => {
                        return Err(AppError {
                            message: "期待された閉じ波括弧が来ませんでした".to_string(),
                            input: input.to_string(),
                            filename: filename.to_string(),
                            pos: input.len(),
                        })
                    }
                    Some(Token {
                        tok: Tok::閉じ波括弧,
                        ..
                    }) => {
                        tokens.next();

                        break;
                    }
                    _ => statements.push(parse_statement_or_declaration(
                        context,
                        tokens,
                        filename,
                        input,
                    )?),
                }
            }
            Ok(Statement::Block {
                statements,
                pos: *pos,
            })
        }
        _ => {
            let expr = Box::new(parse_expr(context, tokens, filename, input)?);
            let semicolon_pos = tokens.peek().unwrap().pos;
            satisfy(
                tokens,
                filename,
                input,
                |tok| tok == &Tok::Semicolon,
                "期待されたセミコロンが来ませんでした",
            )?;
            Ok(Statement::Expr {
                expr,
                semicolon_pos,
            })
        }
    }
}

fn consume_num(
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
    msg: &str,
) -> Result<u8, AppError> {
    match tokens.peek().unwrap() {
        Token {
            tok: Tok::Num(n), ..
        } => {
            tokens.next();
            Ok(*n)
        }
        Token { pos, .. } => Err(AppError {
            message: msg.to_string(),
            input: input.to_string(),
            filename: filename.to_string(),
            pos: *pos,
        }),
    }
}

pub fn parse_角括弧に包まれた数の列(
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
    typ: &mut Type,
) -> Result<(), AppError> {
    let mut sizes = vec![];
    while let Token {
        tok: Tok::開き角括弧,
        ..
    } = tokens.peek().unwrap()
    {
        tokens.next();
        let s = consume_num(tokens, filename, input, "開き角括弧の後に数がない")?;
        satisfy(
            tokens,
            filename,
            input,
            |tok| tok == &Tok::閉じ角括弧,
            "数の後に閉じ角括弧がない",
        )?;
        sizes.push(s);
    }

    for s in sizes.into_iter().rev() {
        let t = std::mem::replace(typ, Type::Int);
        *typ = Type::Arr(Box::new(t), s);
    }

    Ok(())
}

pub fn parse_type_and_identifier(
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<(Type, String), AppError> {
    let mut typ = parse_type(tokens, filename, input)?;
    match tokens.next().unwrap() {
        Token {
            tok: Tok::Identifier(ident),
            ..
        } => {
            parse_角括弧に包まれた数の列(tokens, filename, input, &mut typ)?;
            Ok((typ, ident.clone()))
        }
        Token { pos, .. } => Err(AppError {
            message: "「型と識別子」をパースできません".to_string(),
            input: input.to_string(),
            filename: filename.to_string(),
            pos: *pos,
        }),
    }
}
