use crate::apperror::*;
use crate::ast::*;
use crate::token::*;
use std::{iter::Peekable, slice::Iter};

use super::toplevel::Context;
use super::toplevel::Type;
impl Context {
    fn parse_primary(
        &self,
        tokens: &mut Peekable<Iter<Token>>,
        input: &str,
    ) -> Result<Expr<Type>, AppError> {
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
            } => {
                match tokens.peek().unwrap() {
                    Token {
                        tok: Tok::開き丸括弧,
                        pos: open_pos,
                    } => {
                        tokens.next();

                        let mut args: Vec<Expr<Type>> = Vec::new();

                        match tokens.peek().unwrap() {
                            Token {
                                tok: Tok::閉じ丸括弧,
                                ..
                            } => {
                                tokens.next();
                                let func_decl = self
                                    .previous_function_declarations
                                    .get(ident)
                                    .ok_or(AppError {
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
                                let expr = self.parse_expr(tokens, input)?;
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
                                self.previous_function_declarations
                                    .get(ident)
                                    .ok_or(AppError {
                                        message: format!("関数 {} は宣言されておらず、戻り値の型が分かりません", ident),
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
                                    let expr = self.parse_expr(tokens, input)?;
                                    args.push(expr);
                                }
                                _ => {
                                    break Err(AppError {
                                        message: "閉じ丸括弧かカンマが期待されていました"
                                            .to_string(),
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
                            typ: self
                                .local_var_declarations
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
                }
            }
            Token {
                tok: Tok::開き丸括弧,
                pos,
            } => {
                let expr = self.parse_expr(tokens, input)?;
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
        &self,
        tokens: &mut Peekable<Iter<Token>>,
        input: &str,
    ) -> Result<Expr<Type>, AppError> {
        match tokens.peek() {
            Some(Token { tok: Tok::Add, .. }) => {
                tokens.next();
                self.parse_primary(tokens, input)
            }
            Some(Token { tok: Tok::Sub, pos }) => {
                tokens.next();
                let expr: Expr<Type> = self.parse_primary(tokens, input)?;
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
                let expr = self.parse_unary(tokens, input)?;
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
                let expr = self.parse_unary(tokens, input)?;
                Ok(Expr::UnaryExpr {
                    op: UnaryOp::Addr,
                    op_pos: *pos,
                    typ: Type::Ptr(Box::new(expr.typ())),
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(tokens, input),
        }
    }

    fn parse_multiplicative(
        &self,
        tokens: &mut Peekable<Iter<Token>>,
        input: &str,
    ) -> Result<Expr<Type>, AppError> {
        let mut expr: Expr<Type> = self.parse_unary(tokens, input)?;
        loop {
            match tokens.peek() {
                Some(Token {
                    tok: Tok::Asterisk,
                    pos: op_pos,
                }) => {
                    tokens.next();
                    let 左辺 = Box::new(expr);
                    let 右辺 = Box::new(self.parse_unary(tokens, input)?);
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
                    let 右辺 = Box::new(self.parse_unary(tokens, input)?);
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

    fn parse_additive(
        &self,
        tokens: &mut Peekable<Iter<Token>>,
        input: &str,
    ) -> Result<Expr<Type>, AppError> {
        let mut expr: Expr<Type> = self.parse_multiplicative(tokens, input)?;
        loop {
            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    tok: Tok::Add,
                    pos: op_pos,
                } => {
                    tokens.next();
                    let 左辺 = Box::new(expr);
                    let 右辺 = Box::new(self.parse_multiplicative(tokens, input)?);
                    expr = Expr::BinaryExpr {
                        op: BinaryOp::Add,
                        op_pos: *op_pos,
                        typ: 左辺.typ().add(&右辺.typ()).ok_or(AppError {
                            message: "左辺の型と右辺の型を足し合わせることができません".to_string(),
                            input: input.to_string(),
                            pos: *op_pos,
                        })?,
                        左辺,
                        右辺,
                    }
                }
                Token {
                    tok: Tok::Sub,
                    pos: op_pos,
                } => {
                    tokens.next();
                    let 左辺 = Box::new(expr);
                    let 右辺 = Box::new(self.parse_multiplicative(tokens, input)?);
                    expr = Expr::BinaryExpr {
                        op: BinaryOp::Sub,
                        op_pos: *op_pos,
                        typ: 左辺.typ().sub(&右辺.typ()).ok_or(AppError {
                            message: "左辺の型と右辺の型を足し合わせることができません".to_string(),
                            input: input.to_string(),
                            pos: *op_pos,
                        })?,
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

    fn parse_relational(
        &self,
        tokens: &mut Peekable<Iter<Token>>,
        input: &str,
    ) -> Result<Expr<Type>, AppError> {
        let mut expr: Expr<Type> = self.parse_additive(tokens, input)?;
        loop {
            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    tok: Tok::LessThan,
                    pos: op_pos,
                } => {
                    tokens.next();
                    let 左辺 = Box::new(expr);
                    let 右辺 = Box::new(self.parse_additive(tokens, input)?);
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
                    let 右辺 = Box::new(self.parse_additive(tokens, input)?);
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
                    let 右辺 = Box::new(self.parse_additive(tokens, input)?);
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
                    let 右辺 = Box::new(self.parse_additive(tokens, input)?);
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
        &self,
        tokens: &mut Peekable<Iter<Token>>,
        input: &str,
    ) -> Result<Expr<Type>, AppError> {
        let mut expr: Expr<Type> = self.parse_relational(tokens, input)?;
        loop {
            let tok = tokens.peek().unwrap();
            match tok {
                Token {
                    tok: Tok::Equal,
                    pos: op_pos,
                } => {
                    tokens.next();
                    let 左辺 = Box::new(expr);
                    let 右辺 = Box::new(self.parse_relational(tokens, input)?);
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
                    let 右辺 = Box::new(self.parse_relational(tokens, input)?);
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
        &self,
        tokens: &mut Peekable<Iter<Token>>,
        input: &str,
    ) -> Result<Expr<Type>, AppError> {
        let expr = self.parse_equality(tokens, input)?;
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                tok: Tok::Assign,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(self.parse_expr(tokens, input)?);
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
}
