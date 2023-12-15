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
