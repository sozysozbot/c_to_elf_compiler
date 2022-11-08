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
    Div,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Expr {
    BinaryExpr {
        op: BinaryOp,
        op_pos: usize,
        左辺: Box<Expr>,
        右辺: Box<Expr>,
    },
    Numeric {
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
            左辺: Box::new(Expr::Numeric { val: 5, pos: 0 }),
            右辺: Box::new(Expr::Numeric { val: 3, pos: 4 })
        }
    );
}

fn parse_primary(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    match tokens.next().unwrap() {
        Token {
            payload: TokenPayload::Num(first),
            pos,
        } => {
            let expr = Expr::Numeric {
                val: *first,
                pos: *pos,
            };
            Ok(expr)
        }
        Token {
            payload: TokenPayload::開き丸括弧,
            pos,
        } => {
            let expr = parse_expr(tokens, input)?;
            match tokens.next().unwrap() {
                Token {
                    payload: TokenPayload::閉じ丸括弧,
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

fn parse_unary(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    match tokens.peek() {
        Some(Token {
            payload: TokenPayload::Add,
            ..
        }) => {
            tokens.next();
            parse_primary(tokens, input)
        }
        Some(Token {
            payload: TokenPayload::Sub,
            pos,
        }) => {
            tokens.next();
            let expr = parse_primary(tokens, input)?;
            Ok(Expr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos: *pos,
                左辺: Box::new(Expr::Numeric { val: 0, pos: *pos }),
                右辺: Box::new(expr),
            })
        }
        _ => parse_primary(tokens, input),
    }
}

fn parse_multiplicative(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let mut expr = parse_unary(tokens, input)?;
    loop {
        match tokens.peek() {
            Some(Token {
                payload: TokenPayload::Mul,
                pos: op_pos,
            }) => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_unary(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Mul,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                };
            }
            Some(Token {
                payload: TokenPayload::Div,
                pos: op_pos,
            }) => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_unary(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Div,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                };
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

fn parse_relational(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let mut expr = parse_additive(tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                payload: TokenPayload::LessThan,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LessThan,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                }
            }
            Token {
                payload: TokenPayload::LessThanOrEqual,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LessThanOrEqual,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                }
            }
            Token {
                payload: TokenPayload::GreaterThan,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LessThan, // ここを逆転させ、
                    op_pos: *op_pos,
                    左辺: 右辺, // ここを逆転させればよい
                    右辺: 左辺,
                }
            }
            Token {
                payload: TokenPayload::GreaterThanOrEqual,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_additive(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::LessThanOrEqual, // ここを逆転させ、
                    op_pos: *op_pos,
                    左辺: 右辺, // ここを逆転させればよい
                    右辺: 左辺,
                }
            }
            _ => {
                return Ok(expr);
            }
        }
    }
}

fn parse_equality(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let mut expr = parse_relational(tokens, input)?;
    loop {
        let tok = tokens.peek().unwrap();
        match tok {
            Token {
                payload: TokenPayload::Equal,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_relational(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::Equal,
                    op_pos: *op_pos,
                    左辺,
                    右辺,
                }
            }
            Token {
                payload: TokenPayload::NotEqual,
                pos: op_pos,
            } => {
                tokens.next();
                let 左辺 = Box::new(expr);
                let 右辺 = Box::new(parse_relational(tokens, input)?);
                expr = Expr::BinaryExpr {
                    op: BinaryOp::NotEqual,
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

fn parse_expr(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    parse_equality(tokens, input)
}

fn parse(tokens: &mut Peekable<Iter<Token>>, input: &str) -> Result<Expr, AppError> {
    let expr = parse_expr(tokens, input)?;
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
fn ediに即値を足す(n: u8) -> [u8; 3] {
    [0x83, 0xc7, n]
}

fn ediから即値を引く(n: u8) -> [u8; 3] {
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

fn eaxの符号ビットをedxへ拡張() -> [u8; 1] {
    [0x99]
}

fn edx_eaxをediで割る_商はeaxに_余りはedxに() -> [u8; 2] {
    [0xf7, 0xff]
}

fn eaxをプッシュ() -> [u8; 1] {
    [0x50]
}

fn eaxとediを比較してフラグをセット() -> [u8; 2] {
    [0x39, 0xf8]
}

fn フラグを読んで等しいかどうかをalにセット() -> [u8; 3] {
    [0x0f, 0x94, 0xc0]
}

fn フラグを読んで異なっているかどうかをalにセット() -> [u8; 3] {
    [0x0f, 0x95, 0xc0]
}

fn フラグを読んで未満であるかどうかをalにセット() -> [u8; 3] {
    [0x0f, 0x9c, 0xc0]
}

fn フラグを読んで以下であるかどうかをalにセット() -> [u8; 3] {
    [0x0f, 0x9e, 0xc0]
}

fn alをゼロ拡張してediにセット() -> [u8; 3] {
    [0x0f, 0xb6, 0xf8]
}

#[allow(clippy::too_many_lines)]
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

        Expr::BinaryExpr {
            op: BinaryOp::Div,
            op_pos: _,
            左辺,
            右辺,
        } => {
            exprを評価してediレジスタへ(writer, 左辺);
            writer.write_all(&ediをプッシュ()).unwrap();
            exprを評価してediレジスタへ(writer, 右辺);
            writer.write_all(&ediをプッシュ()).unwrap();

            // 右辺を edi に、左辺を eax に入れる必要がある
            writer.write_all(&ediへとポップ()).unwrap();
            writer.write_all(&eaxへとポップ()).unwrap();

            writer.write_all(&eaxの符号ビットをedxへ拡張()).unwrap();
            writer
                .write_all(&edx_eaxをediで割る_商はeaxに_余りはedxに())
                .unwrap();

            // 結果は eax レジスタに入るので、ediに移し替える
            writer.write_all(&eaxをプッシュ()).unwrap();
            writer.write_all(&ediへとポップ()).unwrap();
        }
        Expr::BinaryExpr {
            op: BinaryOp::Equal,
            op_pos: _,
            左辺,
            右辺,
        } => {
            比較演算を評価してediレジスタへ(
                writer,
                左辺,
                右辺,
                &フラグを読んで等しいかどうかをalにセット(),
            );
        }
        Expr::BinaryExpr {
            op: BinaryOp::NotEqual,
            op_pos: _,
            左辺,
            右辺,
        } => {
            比較演算を評価してediレジスタへ(
                writer,
                左辺,
                右辺,
                &フラグを読んで異なっているかどうかをalにセット(),
            );
        }
        Expr::BinaryExpr {
            op: BinaryOp::LessThan,
            op_pos: _,
            左辺,
            右辺,
        } => {
            比較演算を評価してediレジスタへ(
                writer,
                左辺,
                右辺,
                &フラグを読んで未満であるかどうかをalにセット(),
            );
        }
        Expr::BinaryExpr {
            op: BinaryOp::LessThanOrEqual,
            op_pos: _,
            左辺,
            右辺,
        } => {
            比較演算を評価してediレジスタへ(
                writer,
                左辺,
                右辺,
                &フラグを読んで以下であるかどうかをalにセット(),
            );
        }
        Expr::Numeric { val, pos: _ } => {
            writer.write_all(&ediに代入(*val)).unwrap();
        }
    }
}

fn 比較演算を評価してediレジスタへ(
    writer: &mut impl Write,
    左辺: &Expr,
    右辺: &Expr,
    フラグをalに移す: &[u8],
) {
    exprを評価してediレジスタへ(writer, 左辺);
    writer.write_all(&ediをプッシュ()).unwrap();
    exprを評価してediレジスタへ(writer, 右辺);
    writer.write_all(&ediをプッシュ()).unwrap();

    writer.write_all(&ediへとポップ()).unwrap();
    writer.write_all(&eaxへとポップ()).unwrap();

    writer
        .write_all(&eaxとediを比較してフラグをセット())
        .unwrap();

    writer.write_all(フラグをalに移す).unwrap();

    writer.write_all(&alをゼロ拡張してediにセット()).unwrap();
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
