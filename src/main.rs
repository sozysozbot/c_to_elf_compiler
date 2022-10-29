#![warn(clippy::pedantic)]
use std::io::Write;

use apperror::AppError;
use tokenize::{Token, TokenPayload};

fn main() -> std::io::Result<()> {
    let input = std::env::args().nth(1).expect("入力が与えられていません");

    let tokens = tokenize::tokenize(&input).unwrap();

    let file = std::fs::File::create("a.out")?;
    let mut writer = std::io::BufWriter::new(file);
    if let Err(e) = parse_and_codegen(&mut writer, &tokens, &input) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum BinaryOp {
    Add,
    Sub,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Expr {
    BinaryExpr {
        op: BinaryOp,
        op_pos: usize,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Primary {
        val: u8,
        pos: usize,
    },
}

#[test]
fn parse_test() {
    use crate::tokenize::tokenize;
    let tokens = tokenize("5 - 3").unwrap();
    assert_eq!(
        parse(&tokens).unwrap(),
        Expr::BinaryExpr {
            op: BinaryOp::Sub,
            op_pos: 2,
            lhs: Box::new(Expr::Primary { val: 5, pos: 0 }),
            rhs: Box::new(Expr::Primary { val: 3, pos: 4 })
        }
    );
}

fn parse(tokens: &[Token]) -> Result<Expr, AppError> {
    unimplemented!()
}

fn parse_and_codegen(
    writer: &mut impl Write,
    tokens: &[Token],
    input: &str,
) -> Result<(), AppError> {
    let mut tokens = tokens.iter();

    let tiny = include_bytes!("../experiment/tiny");
    writer.write_all(&tiny[0..0x78]).unwrap();

    match tokens.next().unwrap() {
        Token {
            payload: TokenPayload::Num(first),
            ..
        } => {
            writer.write_all(&[0xb8, 0x3c, 0x00, 0x00, 0x00]).unwrap();
            writer
                .write_all(&[0xbf, *first as u8, 0x00, 0x00, 0x00])
                .unwrap();

            loop {
                let tok = tokens.next().unwrap();
                match tok.payload {
                    TokenPayload::Add => match tokens.next().unwrap() {
                        Token {
                            payload: TokenPayload::Num(n),
                            ..
                        } => writer.write_all(&[0x83, 0xc7, *n]).unwrap(),
                        tok => {
                            return Err(AppError {
                                message: "数値ではありません".to_string(),
                                input: input.to_string(),
                                pos: tok.pos,
                            });
                        }
                    },
                    TokenPayload::Sub => match tokens.next() {
                        Some(Token {
                            payload: TokenPayload::Num(n),
                            ..
                        }) => writer.write_all(&[0x83, 0xef, *n]).unwrap(),
                        Some(tok) => {
                            return Err(AppError {
                                message: "数値ではありません".to_string(),
                                input: input.to_string(),
                                pos: tok.pos,
                            });
                        }
                        None => panic!("入力が演算子で終わりました"),
                    },
                    TokenPayload::Eof => {
                        writer.write_all(&[0x0f, 0x05]).unwrap();
                        return Ok(());
                    }
                    TokenPayload::Num(_) => {
                        return Err(AppError {
                            message: "演算子かeofが期待されていますが、数が来ました".to_string(),
                            input: input.to_string(),
                            pos: tok.pos,
                        });
                    }
                }
            }
        }
        tok => Err(AppError {
            message: "入力が数字以外で始まっています".to_string(),
            input: input.to_string(),
            pos: tok.pos,
        }),
    }
}
mod apperror;

mod tokenize;
