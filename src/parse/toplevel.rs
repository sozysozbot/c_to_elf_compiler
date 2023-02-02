use crate::apperror::*;
use crate::ast::*;
use crate::token::*;
use std::collections::HashMap;
use std::{iter::Peekable, slice::Iter};

use super::expression::parse_expr;
use super::typ::parse_type;

#[test]
fn parse_test() {
    use crate::tokenize::tokenize;
    let input = "5 - 3;";
    let tokens = tokenize(input).unwrap();
    let mut tokens = tokens.iter().peekable();
    assert_eq!(
        parse_statement(
            &Context {
                local_var_and_param_declarations: HashMap::new(),
                function_declarations: HashMap::new()
            },
            &mut tokens,
            input
        )
        .unwrap(),
        Statement::Expr {
            expr: Box::new(Expr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos: 2,
                typ: Type::Int,
                左辺: Box::new(Expr::Numeric {
                    val: 5,
                    pos: 0,
                    typ: Type::Int
                }),
                右辺: Box::new(Expr::Numeric {
                    val: 3,
                    pos: 4,
                    typ: Type::Int
                })
            }),
            semicolon_pos: 5
        }
    );
}
fn parse_statement(
    context: &Context,
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<Statement, AppError> {
    let tok = tokens.peek().unwrap();
    match tok {
        Token {
            tok: Tok::Throw, ..
        } => {
            tokens.next();
            let expr = Box::new(parse_expr(context, tokens, input)?);
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
            let expr = Box::new(parse_expr(context, tokens, input)?);
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
                        pos: tok.pos,
                    })
                }
            }
            let cond = Box::new(parse_expr(context, tokens, input)?);

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
                        pos: tok.pos,
                    })
                }
            }
            let then = Box::new(parse_statement(context, tokens, input)?);
            let tok = tokens.peek().unwrap();
            let else_ = match tok {
                Token { tok: Tok::Else, .. } => {
                    tokens.next();
                    Some(Box::new(parse_statement(context, tokens, input)?))
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
                        pos: tok.pos,
                    })
                }
            }
            let cond = Box::new(parse_expr(context, tokens, input)?);

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
                        pos: tok.pos,
                    })
                }
            }
            let body = Box::new(parse_statement(context, tokens, input)?);
            Ok(Statement::While {
                cond,
                body,
                pos: *pos,
            })
        }
        Token { tok: Tok::For, pos } => {
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
                        pos: tok.pos,
                    })
                }
            }
            let tok = tokens.peek().unwrap();
            let init = match tok {
                Token {
                    tok: Tok::Semicolon,
                    ..
                } => None,
                _ => Some(Box::new(parse_expr(context, tokens, input)?)),
            };
            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    tok: Tok::Semicolon,
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
                    tok: Tok::Semicolon,
                    ..
                } => None,
                _ => Some(Box::new(parse_expr(context, tokens, input)?)),
            };
            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    tok: Tok::Semicolon,
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
                    tok: Tok::閉じ丸括弧,
                    ..
                } => None,
                _ => Some(Box::new(parse_expr(context, tokens, input)?)),
            };
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
                        pos: tok.pos,
                    })
                }
            }
            let body = Box::new(parse_statement(context, tokens, input)?);
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
                    _ => statements.push(parse_statement(context, tokens, input)?),
                }
            }
            Ok(Statement::Block {
                statements,
                pos: *pos,
            })
        }
        _ => {
            let expr = Box::new(parse_expr(context, tokens, input)?);
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

fn parse_type_and_identifier(
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<(Type, String), AppError> {
    let typ = parse_type(tokens, input)?;
    match tokens.next().unwrap() {
        Token {
            tok: Tok::Identifier(ident),
            ..
        } => Ok((typ, ident.clone())),
        Token { pos, .. } => Err(AppError {
            message: "「型と識別子」をパースできません".to_string(),
            input: input.to_string(),
            pos: *pos,
        }),
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Type {
    Int,
    Ptr(Box<Type>),
    Arr(Box<Type>, u8),
}

impl Type {
    pub fn deref(&self) -> Option<Self> {
        match self {
            Type::Int => None,
            Type::Ptr(x) | Type::Arr(x, _) => Some((**x).clone()),
        }
    }

    pub fn sizeof(&self) -> u8 {
        match self {
            Type::Int => 4,
            Type::Ptr(_) => 8,
            Type::Arr(t, len) => t
                .sizeof()
                .checked_mul(*len)
                .expect("型のサイズが u8 に収まりません"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub func_name: String,
    pub params: Vec<(Type, String)>,
    pub pos: usize,
    pub statements: Vec<Statement>,
    pub return_type: Type,
    pub local_var_declarations: HashMap<String, Type>,
}

impl From<FunctionDefinition> for FunctionDeclaration {
    fn from(s: FunctionDefinition) -> FunctionDeclaration {
        (
            s.func_name,
            FunctionSignature {
                pos: s.pos,
                return_type: s.return_type,
                params: s.params.into_iter().map(|(typ, _)| typ).collect(),
            },
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionSignature {
    pub params: Vec<Type>,
    pub pos: usize,
    pub return_type: Type,
}

pub type FunctionDeclaration = (String, FunctionSignature);

pub struct Context {
    pub local_var_and_param_declarations: HashMap<String, Type>,
    pub function_declarations: HashMap<String, FunctionSignature>,
}

fn after_param_list(
    previous_function_declarations: &HashMap<String, FunctionSignature>,
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
    params: Vec<(Type, String)>,
    pos: usize,
    return_type: Type,
    ident: &str,
) -> Result<FunctionDefinition, AppError> {
    let signature = FunctionSignature {
        params: params.iter().map(|(typ, _)| (*typ).clone()).collect(),
        pos,
        return_type: return_type.clone(),
    };
    match tokens.peek().unwrap() {
        Token {
            tok: Tok::開き波括弧,
            ..
        } => {
            tokens.next();
            let mut local_var_declarations = HashMap::new();
            #[allow(clippy::while_let_loop)]
            loop {
                let (local_var_type, local_var_name) = match tokens.peek().unwrap() {
                    Token { tok: Tok::Int, .. } => parse_type_and_identifier(tokens, input)?,
                    _ => break,
                };

                match tokens.peek().unwrap() {
                    Token {
                        tok: Tok::Semicolon,
                        ..
                    } => {
                        tokens.next();
                    }
                    Token { pos, .. } => {
                        return Err(AppError {
                            message:
                                "関数内の変数宣言で、型名と識別子の後にセミコロン以外が来ました"
                                    .to_string(),
                            input: input.to_string(),
                            pos: *pos,
                        })
                    }
                }
                local_var_declarations.insert(local_var_name, local_var_type);
            }

            let mut statements = vec![];
            loop {
                match tokens.peek() {
                    None => {
                        return Err(AppError {
                            message: "期待された閉じ波括弧が来ませんでした".to_string(),
                            input: input.to_string(),
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
                    _ => {
                        let mut local_var_and_param_declarations = local_var_declarations.clone();
                        local_var_and_param_declarations.extend(
                            params
                                .iter()
                                .map(|(typ, ident)| (ident.clone(), (*typ).clone())),
                        );

                        let mut function_declarations = previous_function_declarations.clone();

                        // 今読んでる関数の定義も足さないと再帰呼び出しができない
                        function_declarations.insert(ident.to_string(), signature.clone());

                        statements.push(parse_statement(
                            &Context {
                                local_var_and_param_declarations,
                                function_declarations,
                            },
                            tokens,
                            input,
                        )?)
                    }
                }
            }

            let func_def = FunctionDefinition {
                func_name: ident.to_string(),
                params,
                pos,
                statements,
                return_type,
                local_var_declarations,
            };
            Ok(func_def)
        }
        Token { pos, .. } => Err(AppError {
            message: "仮引数リストの後に、開き波括弧以外のトークンが来ました".to_string(),
            input: input.to_string(),
            pos: *pos,
        }),
    }
}

pub fn parse_toplevel_function_definition(
    previous_declarations: &HashMap<String, FunctionSignature>,
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<FunctionDefinition, AppError> {
    let return_type = parse_type(tokens, input)?;
    match tokens.next().unwrap() {
        Token {
            tok: Tok::Identifier(ident),
            pos,
        } => match tokens.peek().unwrap() {
            Token {
                tok: Tok::開き丸括弧,
                pos: open_pos,
            } => {
                tokens.next();

                let mut params = Vec::new();

                match tokens.peek().unwrap() {
                    Token {
                        tok: Tok::閉じ丸括弧,
                        ..
                    } => {
                        tokens.next();
                        return after_param_list(
                            previous_declarations,
                            tokens,
                            input,
                            params,
                            *pos,
                            return_type,
                            ident,
                        );
                    }
                    _ => {
                        let param = parse_type_and_identifier(tokens, input)?;
                        params.push(param);
                    }
                }

                loop {
                    match tokens.peek().unwrap() {
                        Token {
                            tok: Tok::閉じ丸括弧,
                            ..
                        } => {
                            tokens.next();
                            return after_param_list(
                                previous_declarations,
                                tokens,
                                input,
                                params,
                                *pos,
                                return_type,
                                ident,
                            );
                        }
                        Token {
                            tok: Tok::Comma, ..
                        } => {
                            tokens.next();
                            let param = parse_type_and_identifier(tokens, input)?;
                            params.push(param);
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
            _ => Err(AppError {
                message: "トップレベルに識別子がありますが、その後に関数引数の丸括弧がありません"
                    .to_string(),
                input: input.to_string(),
                pos: *pos + 1,
            }),
        },
        Token { pos, .. } => Err(AppError {
            message: "トップレベルが識別子でないもので始まっています".to_string(),
            input: input.to_string(),
            pos: *pos + 1,
        }),
    }
}

pub fn parse(
    function_declarations: &mut HashMap<String, FunctionSignature>,
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<Vec<FunctionDefinition>, AppError> {
    let mut function_definitions: Vec<FunctionDefinition> = vec![];
    while tokens.peek().is_some() {
        let new_def = parse_toplevel_function_definition(function_declarations, tokens, input)?;
        let (name, signature) = new_def.clone().into();
        function_declarations.insert(name, signature);
        function_definitions.push(new_def);
    }
    Ok(function_definitions)
}
