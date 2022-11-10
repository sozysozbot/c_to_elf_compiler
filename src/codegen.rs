use crate::ast::*;
use std::io::Write;

/*
fn ediに即値を足す(n: u8) -> [u8; 3] {
    [0x83, 0xc7, n]
}

fn 即値をプッシュ(n: u8) -> [u8; 2] {
    [0x6a, n]
}
*/

fn ediから即値を引く(n: u8) -> [u8; 3] {
    [0x83, 0xef, n]
}

fn ediに代入(n: u8) -> [u8; 5] {
    [0xbf, n, 0x00, 0x00, 0x00]
}

fn ediをプッシュ() -> [u8; 1] {
    [0x57]
}

pub fn rbpをプッシュ() -> [u8; 1] {
    [0x55]
}

pub fn rspをrbpにコピー() -> [u8; 3] {
    [0x48, 0x89, 0xe5]
}

pub fn rspから即値を引く(n: u8) -> [u8; 4] {
    [0x48, 0x83, 0xec, n]
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

fn rdiを間接参照() -> [u8; 3] {
    [0x48, 0x8b, 0x3f]
}

fn raxが指す位置にediを代入() -> [u8; 2] {
    [0x89, 0x38]
}

pub fn exprを左辺値として評価してアドレスをrdiレジスタへ(
    writer: &mut impl Write,
    expr: &Expr,
) {
    match expr {
        Expr::Identifier { ident, pos: _ } => {
            let offset = (((*ident as usize) - ('a' as usize)) * 4) as u8;
            writer.write_all(&rbpをプッシュ()).unwrap();
            writer.write_all(&ediへとポップ()).unwrap();
            writer.write_all(&ediから即値を引く(offset)).unwrap();
        }
        _ => panic!("代入式の左辺に左辺値以外が来ています"),
    }
}

#[allow(clippy::too_many_lines)]
pub fn exprを評価してediレジスタへ(writer: &mut impl Write, expr: &Expr) {
    match expr {
        Expr::BinaryExpr {
            op: BinaryOp::Assign,
            op_pos: _,
            左辺,
            右辺,
        } => {
            exprを左辺値として評価してアドレスをrdiレジスタへ(writer, 左辺);
            writer.write_all(&ediをプッシュ()).unwrap();
            exprを評価してediレジスタへ(writer, 右辺);

            writer.write_all(&eaxへとポップ()).unwrap(); // 左辺のアドレス
            writer.write_all(&raxが指す位置にediを代入()).unwrap();
        }
        Expr::Identifier { .. } => {
            exprを左辺値として評価してアドレスをrdiレジスタへ(writer, expr);
            writer.write_all(&rdiを間接参照()).unwrap();
        }
        Expr::BinaryExpr {
            op: BinaryOp::AndThen,
            op_pos: _,
            左辺,
            右辺,
        } => {
            exprを評価してediレジスタへ(writer, 左辺); // 左辺は push せずに捨てる
            exprを評価してediレジスタへ(writer, 右辺);
        }
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
