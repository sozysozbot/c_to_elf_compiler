use crate::apperror::*;
use crate::token::*;
use std::{iter::Peekable, slice::Iter};

pub fn satisfy(
    tokens: &mut Peekable<Iter<Token>>,
    input: &str,
    cond: impl FnOnce(&Tok) -> bool,
    msg: &str,
) -> Result<(), AppError> {
    match tokens.peek().unwrap() {
        Token { tok, .. } if cond(tok) => {
            tokens.next();
            Ok(())
        }
        Token { pos, .. } => Err(AppError {
            message: msg.to_string(),
            input: input.to_string(),
            pos: *pos,
        }),
    }
}
