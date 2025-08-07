use std::{iter::Peekable, slice::Iter};

use crate::{
    apperror::AppError,
    token::{Tok, Token},
};

use super::toplevel::Type;

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
        Token {
            tok: Tok::Struct, ..
        } => {
            tokens.next().unwrap();
            match tokens.next() {
                Some(Token {
                    tok: Tok::Identifier(struct_name),
                    ..
                }) => {

                    Type::Struct {
                        struct_name: struct_name.clone(),
                    }
                }
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
