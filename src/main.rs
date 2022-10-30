#![warn(clippy::pedantic)]
use std::{io::Write, iter::Peekable, slice::Iter};

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
    Mul,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Expr {
    BinaryExpr {
        op: BinaryOp,
        op_pos: usize,
        左辺: Box<Expr>,
        右辺: Box<Expr>,
    },
    Primary {
        val: u8,
        pos: usize,
    },
}

#[test]
fn parse_test() {
    use crate::tokenize::tokenize;
    let input = "5 - 3";
    let tokens = tokenize(input).unwrap();
    let mut tokens = tokens.iter().peekable();
    assert_eq!(
        parse(&mut tokens, input).unwrap(),
        Expr::BinaryExpr {
            op: BinaryOp::Sub,
            op_pos: 2,
            左辺: Box::new(Expr::Primary { val: 5, pos: 0 }),
            右辺: Box::new(Expr::Primary { val: 3, pos: 4 })
        }
    );
}

fn parse_primary(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    match tokens.next().unwrap() {
        Token {
            payload: TokenPayload::Num(first),
            pos,
        } => {
            let expr = Expr::Primary {
                val: *first,
                pos: *pos,
            };
            Ok(expr)
        }
        tok => Err(AppError {
            message: "数値リテラル以外が来ました".to_string(),
            input: input.to_string(),
            pos: tok.pos,
        }),
    }
}

fn parse_multiplicative(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let mut expr = parse_primary(tokens, input)?;
    loop {
        match tokens.peek() {
            Some(Token {
                payload: TokenPayload::Mul,
                pos: op_pos,
            }) => {
                tokens.next();
                match tokens.next().unwrap() {
                    Token {
                        payload: TokenPayload::Num(n),
                        pos,
                    } => {
                        let 左辺 = Box::new(expr);
                        let 右辺 = Box::new(Expr::Primary { val: *n, pos: *pos });
                        expr = Expr::BinaryExpr {
                            op: BinaryOp::Mul,
                            op_pos: *op_pos,
                            左辺,
                            右辺,
                        }
                    }
                    tok => {
                        return Err(AppError {
                            message: "演算子の次に来ているものが数値ではありません".to_string(),
                            input: input.to_string(),
                            pos: tok.pos,
                        });
                    }
                }
            }

            _ => {
                return Ok(expr);
            }
        }
    }
}

fn parse_additive(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let mut expr = parse_multiplicative(tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                payload: TokenPayload::Add,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_multiplicative(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Add,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                }
            }
            Token {
                payload: TokenPayload::Sub,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_multiplicative(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Sub,
                    op_pos: *op_pos,
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

fn parse(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let expr = parse_additive(tokens, input)?;
    let tok = tokens.peek().unwrap();
    if tok.payload == TokenPayload::Eof {
        Ok(expr)
    } else {
        Err(AppError {
            message: "期待されたeofが来ませんでした".to_string(),
            input: input.to_string(),
            pos: tok.pos,
        })
    }
}

/*
fn edi増加(n: u8) -> [u8; 3] {
    [0x83, 0xc7, n]
}

fn edi減少(n: u8) -> [u8; 3] {
    [0x83, 0xef, n]
}

fn 即値をプッシュ(n: u8) -> [u8; 2] {
    [0x6a, n]
}
*/

fn ediに代入(n: u8) -> [u8; 5] {
    [0xbf, n, 0x00, 0x00, 0x00]
}

fn ediをプッシュ() -> [u8; 1] {
    [0x57]
}

fn ediへとポップ() -> [u8; 1] {
    [0x5f]
}

fn eaxへとポップ() -> [u8; 1] {
    [0x58]
}

fn ediにeaxを足し合わせる() -> [u8; 2] {
    [0x01, 0xc7]
}

fn ediからeaxを減じる() -> [u8; 2] {
    [0x29, 0xc7]
}

fn ediをeax倍にする() -> [u8; 3] {
    [0x0f, 0xaf, 0xf8]
}

fn exprを評価してediレジスタへ(writer: &mut impl Write, expr: &Expr) {
    match expr {
        Expr::BinaryExpr {
            op: BinaryOp::Add,
            op_pos: _,
            左辺,
            右辺,
        } => {
            exprを評価してediレジスタへ(writer, 左辺);
            writer.write_all(&ediをプッシュ()).unwrap();
            exprを評価してediレジスタへ(writer, 右辺);
            writer.write_all(&ediをプッシュ()).unwrap();
            writer.write_all(&eaxへとポップ()).unwrap();
            writer.write_all(&ediへとポップ()).unwrap();
            writer.write_all(&ediにeaxを足し合わせる()).unwrap();
        }
        Expr::BinaryExpr {
            op: BinaryOp::Sub,
            op_pos: _,
            左辺,
            右辺,
        } => {
            exprを評価してediレジスタへ(writer, 左辺);
            writer.write_all(&ediをプッシュ()).unwrap();
            exprを評価してediレジスタへ(writer, 右辺);
            writer.write_all(&ediをプッシュ()).unwrap();
            writer.write_all(&eaxへとポップ()).unwrap();
            writer.write_all(&ediへとポップ()).unwrap();
            writer.write_all(&ediからeaxを減じる()).unwrap();
        }
        Expr::BinaryExpr {
            op: BinaryOp::Mul,
            op_pos: _,
            左辺,
            右辺,
        } => {
            exprを評価してediレジスタへ(writer, 左辺);
            writer.write_all(&ediをプッシュ()).unwrap();
            exprを評価してediレジスタへ(writer, 右辺);
            writer.write_all(&ediをプッシュ()).unwrap();
            writer.write_all(&eaxへとポップ()).unwrap();
            writer.write_all(&ediへとポップ()).unwrap();
            writer.write_all(&ediをeax倍にする()).unwrap();
        }
        Expr::Primary { val, pos: _ } => {
            writer.write_all(&ediに代入(*val)).unwrap();
        }
    }
}

fn parse_and_codegen(
    mut writer: &mut impl Write,
    tokens: &[Token],
    input: &str,
) -> Result<(), AppError> {
    let mut tokens = tokens.iter().peekable();
    let expr = parse(&mut tokens, input)?;

    let tiny = include_bytes!("../experiment/tiny");
    writer.write_all(&tiny[0..0x78]).unwrap();
    exprを評価してediレジスタへ(&mut writer, &expr);
    writer.write_all(&[0xb8, 0x3c, 0x00, 0x00, 0x00]).unwrap();
    writer.write_all(&[0x0f, 0x05]).unwrap();
    Ok(())
}
mod apperror;

mod tokenize;
