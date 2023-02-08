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

pub fn recover<A>(
    tokens: &mut Peekable<Iter<Token>>,
    f: impl FnOnce(&mut Peekable<Iter<Token>>) -> Result<A, AppError>,
) -> Result<Option<A>, AppError> {
    let prev_pos = tokens.peek().unwrap().pos;
    match f(tokens) {
        Ok(a) => Ok(Some(a)),
        Err(e) => {
            let pos = tokens.peek().unwrap().pos;
            if pos == prev_pos {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}
