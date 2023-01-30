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
        Context {
            local_var_declarations: HashMap::new()
        }
        .parse_statement(&mut tokens, input)
        .unwrap(),
        Statement::Expr {
            expr: Box::new(UntypedExpr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos: 2,
                typ: Any,
                左辺: Box::new(UntypedExpr::Numeric {
                    val: 5,
                    pos: 0,
                    typ: Any
                }),
                右辺: Box::new(UntypedExpr::Numeric {
                    val: 3,
                    pos: 4,
                    typ: Any
                })
            }),
            semicolon_pos: 5
        }
    );
}
impl Context {
    fn parse_statement(
        &self,
        tokens: &mut Peekable<Iter<Token>>,
        input: &str,
    ) -> Result<Statement<Any>, AppError> {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::Throw, ..
            } => {
                tokens.next();
                let expr = Box::new(parse_expr(tokens, input)?);
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
                let expr = Box::new(parse_expr(tokens, input)?);
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
                let cond = Box::new(parse_expr(tokens, input)?);

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
                let then = Box::new(self.parse_statement(tokens, input)?);
                let tok = tokens.peek().unwrap();
                let else_ = match tok {
                    Token { tok: Tok::Else, .. } => {
                        tokens.next();
                        Some(Box::new(self.parse_statement(tokens, input)?))
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
                let cond = Box::new(parse_expr(tokens, input)?);

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
                let body = Box::new(self.parse_statement(tokens, input)?);
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
                    _ => Some(Box::new(parse_expr(tokens, input)?)),
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
                    _ => Some(Box::new(parse_expr(tokens, input)?)),
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
                    _ => Some(Box::new(parse_expr(tokens, input)?)),
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
                let body = Box::new(self.parse_statement(tokens, input)?);
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
                        _ => statements.push(self.parse_statement(tokens, input)?),
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
}
#[derive(Debug, Clone)]
pub struct ParameterIdentifier {
    pub ident: String,
    pub pos: usize,
}

fn parse_parameter_type_and_identifier(
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<(Type, ParameterIdentifier), AppError> {
    let parameter_type = parse_type(tokens, input)?;

    match tokens.next().unwrap() {
        Token {
            tok: Tok::Identifier(ident),
            pos,
        } => Ok((
            parameter_type,
            ParameterIdentifier {
                ident: ident.clone(),
                pos: *pos,
            },
        )),
        Token { pos, .. } => Err(AppError {
            message: "仮引数をパースできません".to_string(),
            input: input.to_string(),
            pos: *pos,
        }),
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Ptr(Box<Type>),
}

pub struct FunctionDefinition {
    pub func_name: String,
    pub params: Vec<(Type, ParameterIdentifier)>,
    pub pos: usize,
    pub statements: Vec<Statement<Any>>,
    pub return_type: Type,
    pub local_var_declarations: HashMap<String, Type>,
}

pub struct Context {
    pub local_var_declarations: HashMap<String, Type>,
}

fn after_param_list(
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
    params: Vec<(Type, ParameterIdentifier)>,
    pos: usize,
    return_type: Type,
    ident: &str,
) -> Result<FunctionDefinition, AppError> {
    match tokens.peek().unwrap() {
        Token {
            tok: Tok::開き波括弧,
            ..
        } => {
            tokens.next();
            let mut local_var_declarations = HashMap::new();
            #[allow(clippy::while_let_loop)]
            loop {
                let local_var_type = match tokens.peek().unwrap() {
                    Token { tok: Tok::Int, .. } => parse_type(tokens, input)?,
                    Token { .. } => {
                        // 変数定義は終わり
                        break;
                    }
                };
                let local_var_name = match tokens.peek().unwrap() {
                    Token {
                        tok: Tok::Identifier(ident),
                        ..
                    } => {
                        tokens.next();
                        ident.clone()
                    }
                    Token { pos, .. } => {
                        return Err(AppError {
                            message: "関数内の変数宣言で、型名の後に識別子以外が来ました"
                                .to_string(),
                            input: input.to_string(),
                            pos: *pos,
                        })
                    }
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
                    _ => statements.push(
                        Context {
                            local_var_declarations: local_var_declarations.clone(),
                        }
                        .parse_statement(tokens, input)?,
                    ),
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
                        return after_param_list(tokens, input, params, *pos, return_type, ident);
                    }
                    _ => {
                        let param = parse_parameter_type_and_identifier(tokens, input)?;
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
                            let param = parse_parameter_type_and_identifier(tokens, input)?;
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
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
) -> Result<Vec<FunctionDefinition>, AppError> {
    let mut function_definitions = vec![];
    while tokens.peek().is_some() {
        function_definitions.push(parse_toplevel_function_definition(tokens, input)?);
    }
    Ok(function_definitions)
}
