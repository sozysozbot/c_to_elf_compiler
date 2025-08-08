use std::{collections::HashMap, iter::Peekable, slice::Iter};

use crate::{
    apperror::AppError,
    parse::toplevel::StructDefinition,
    token::{Tok, Token},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Type {
    Int,
    Char,
    Ptr(Box<Type>),
    Arr(Box<Type>, i32),
    Struct { struct_name: String },
    Void,
}

impl Type {
    pub fn deref(&self) -> Option<Self> {
        match self {
            Type::Int | Type::Char | Type::Struct { .. } | Type::Void => None,
            Type::Ptr(x) | Type::Arr(x, _) => Some((**x).clone()),
        }
    }

    pub fn sizeof_primitive(&self, msg: &str) -> i32 {
        match self {
            Type::Int => 4,
            Type::Char => 1,
            Type::Ptr(_) => 8,
            Type::Arr(t, len) => t
                .sizeof_primitive(msg)
                .checked_mul(*len)
                .expect("型のサイズが u8 に収まりません"),
            _ => panic!("sizeof_primitive() は構造体に対しては定義されていません。 msg: {msg}"),
        }
    }

    pub fn sizeof(&self, struct_def_table: &HashMap<String, StructDefinition>) -> i32 {
        match self {
            Type::Int => 4,
            Type::Char => 1,
            Type::Void => 1, // GNU extension
            Type::Ptr(_) => 8,
            Type::Arr(t, len) => t
                .sizeof(struct_def_table)
                .checked_mul(*len)
                .expect("型のサイズが i32 に収まりません"),
            Type::Struct { struct_name } => struct_def_table.get(struct_name).map_or_else(
                || {
                    panic!("構造体 {struct_name} の定義が見つかりません");
                },
                |s| s.size,
            ),
        }
    }

    pub fn alignof(&self, struct_def_table: &HashMap<String, StructDefinition>) -> i32 {
        match self {
            Type::Int => 4,
            Type::Char => 1,
            Type::Void => 1, // GNU extension
            Type::Ptr(_) => 8,
            Type::Arr(t, _) => t.alignof(struct_def_table),
            Type::Struct { struct_name } => struct_def_table.get(struct_name).map_or_else(
                || {
                    panic!("構造体 {struct_name} の定義が見つかりません");
                },
                |s| s.align,
            ),
        }
    }
}

pub fn parse_type(
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Type, AppError> {
    let mut typ = match tokens.peek().unwrap() {
        Token { tok: Tok::Int, .. } => {
            tokens.next().unwrap();
            Type::Int
        }
        Token { tok: Tok::Char, .. } => {
            tokens.next().unwrap();
            Type::Char
        }
        Token { tok: Tok::Void, .. } => {
            tokens.next().unwrap();
            Type::Void
        }
        Token {
            tok: Tok::Struct, ..
        } => {
            tokens.next().unwrap();
            match tokens.next() {
                Some(Token {
                    tok: Tok::Identifier(struct_name),
                    ..
                }) => Type::Struct {
                    struct_name: struct_name.clone(),
                },
                Some(Token { pos, .. }) => {
                    return Err(AppError {
                        message: "構造体名がありません".to_string(),
                        input: input.to_string(),
                        filename: filename.to_string(),
                        pos: *pos,
                    });
                }
                None => {
                    return Err(AppError {
                        message: "構造体名がありません".to_string(),
                        input: input.to_string(),
                        filename: filename.to_string(),
                        pos: 0,
                    });
                }
            }
        }
        Token { pos, .. } => {
            return Err(AppError {
                message: "型名でありません".to_string(),
                input: input.to_string(),
                filename: filename.to_string(),
                pos: *pos,
            })
        }
    };

    loop {
        match tokens.peek().unwrap() {
            Token {
                tok: Tok::Asterisk, ..
            } => {
                typ = Type::Ptr(Box::new(typ));
                tokens.next().unwrap();
            }
            _ => return Ok(typ),
        }
    }
}
