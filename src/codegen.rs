use crate::{ast::*, Buf};
use std::{collections::HashMap, io::Write};

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

fn rdiから即値を引く(n: u8) -> [u8; 4] {
    [0x48, 0x83, 0xef, n]
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

pub fn rspから即値を引く(x: u8) -> Buf {
    Buf::from([0x48, 0x83, 0xec, x])
}

fn rspに即値を足す(x: u8) -> Buf {
    Buf::from([0x48, 0x83, 0xc4, x])
}

pub fn プロローグ(x: u8) -> Buf {
    Buf::from(rbpをプッシュ())
        .join(rspをrbpにコピー())
        .join(rspから即値を引く(x))
}

// leave = mov rsp, rbp + pop rbp
pub fn leave() -> Buf {
    Buf::from([0xc9])
}

pub fn エピローグ() -> Buf {
    leave().join(ret())
}

fn ediへとポップ() -> [u8; 1] {
    [0x5f]
}

fn esiへとポップ() -> [u8; 1] {
    [0x5e]
}

fn eaxへとポップ() -> [u8; 1] {
    [0x58]
}

fn edxへとポップ() -> [u8; 1] {
    [0x5a]
}

fn ecxへとポップ() -> [u8; 1] {
    [0x59]
}

fn r8dへとポップ() -> [u8; 2] {
    [0x41, 0x58]
}

fn r9dへとポップ() -> [u8; 2] {
    [0x41, 0x59]
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

fn ediが0かを確認() -> [u8; 3] {
    [0x83, 0xff, 0x00]
}

fn jmp(n: i8) -> [u8; 2] {
    [0xeb, n.to_le_bytes()[0]]
}

fn je(n: i8) -> [u8; 2] {
    [0x74, n.to_le_bytes()[0]]
}

fn call_rax() -> [u8; 2] {
    [0xff, 0xd0]
}

fn eaxに即値をセット(n: u32) -> [u8; 5] {
    let buf = n.to_le_bytes();
    [0xb8, buf[0], buf[1], buf[2], buf[3]]
}

fn edxに即値をセット(n: u32) -> [u8; 5] {
    let buf = n.to_le_bytes();
    [0xba, buf[0], buf[1], buf[2], buf[3]]
}

fn syscall() -> [u8; 2] {
    [0x0f, 0x05]
}

fn ret() -> [u8; 1] {
    [0xc3]
}

fn eaxをediにmov() -> [u8; 2] {
    [0x89, 0xc7]
}
fn esiにespをセット() -> [u8; 3] {
    [0x48, 0x89, 0xe6]
}

pub fn builtin_three関数を生成() -> Buf {
    プロローグ(0).join(eaxに即値をセット(3)).join(エピローグ())
}

fn ediをrbpにoffsetを足した位置に代入(offset: i8) -> [u8; 3] {
    [0x89, 0x7d, offset.to_le_bytes()[0]]
}

fn rsiにrbpにoffsetを足したアドレスを代入(offset: i8) -> [u8; 4] {
    [0x48, 0x8d, 0x75, offset.to_le_bytes()[0]]
}

pub fn builtin_putchar関数を生成() -> Buf {
    プロローグ(4)
        .join(ediをrbpにoffsetを足した位置に代入(-4))
        .join(eaxに即値をセット(1)) // write
        .join(ediに代入(1)) // fd
        .join(rsiにrbpにoffsetを足したアドレスを代入(-4)) // buf
        .join(edxに即値をセット(1)) // count
        .join(syscall())
        .join(エピローグ())
}

pub fn exprを左辺値として評価してアドレスをrdiレジスタへ(
    writer: &mut impl Write,
    expr: &Expr,
    idents: &mut HashMap<String, u8>,
    _stack_size: &mut u32,
) {
    match expr {
        Expr::Identifier { ident, pos: _ } => {
            let len = idents.len();
            let idx = idents.entry(ident.clone()).or_insert(len as u8);
            let offset = *idx * 4;
            writer.write_all(&rbpをプッシュ()).unwrap();
            writer.write_all(&ediへとポップ()).unwrap();
            writer.write_all(&rdiから即値を引く(offset)).unwrap();
        }
        _ => panic!("代入式の左辺に左辺値以外が来ています"),
    }
}

pub fn statementを評価(
    stmt: &Statement,
    idents: &mut HashMap<String, u8>,
    functions: &HashMap<String, u32>,
    stack_size: &mut u32,
) -> Buf {
    match stmt {
        Statement::Expr {
            expr,
            semicolon_pos: _,
        } => {
            let mut writer = Vec::new();
            exprを評価してediレジスタへ(&mut writer, expr, idents, functions, stack_size);
            Buf::from(writer)
        }
        Statement::Return {
            expr,
            semicolon_pos: _,
        } => {
            let mut writer = Vec::new();
            exprを評価してediレジスタへ(&mut writer, expr, idents, functions, stack_size);
            writer.write_all(&[0xb8, 0x3c, 0x00, 0x00, 0x00]).unwrap();
            writer.write_all(&[0x0f, 0x05]).unwrap();
            Buf::from(writer)
        }
        Statement::If {
            cond, then, else_, ..
        } => {
            let else_buf = else_
                .as_ref()
                .map(|else_| statementを評価(else_.as_ref(), idents, functions, stack_size));

            let then_buf = statementを評価(then.as_ref(), idents, functions, stack_size).join(
                else_buf
                    .as_ref()
                    .map(|else_buf| Buf::from(jmp(i8::try_from(else_buf.len()).unwrap())))
                    .unwrap_or_else(Buf::new),
            );

            let cond_buf = {
                let mut v = Vec::new();
                exprを評価してediレジスタへ(&mut v, cond, idents, functions, stack_size);
                v.write_all(&ediが0かを確認()).unwrap();
                v.write_all(&je(i8::try_from(then_buf.len()).unwrap()))
                    .unwrap();
                Buf::from(v)
            };

            cond_buf
                .join(then_buf)
                .join(else_buf.unwrap_or_else(Buf::new))
        }
        Statement::While { cond, body, .. } => {
            let body_buf = statementを評価(body.as_ref(), idents, functions, stack_size);
            let cond_buf = {
                let mut v = Vec::new();
                exprを評価してediレジスタへ(&mut v, cond, idents, functions, stack_size);
                v.write_all(&ediが0かを確認()).unwrap();
                v.write_all(&je(i8::try_from(body_buf.len() + 2).unwrap()))
                    .unwrap();
                Buf::from(v)
            };
            let buf = cond_buf.join(body_buf);
            let buf_len = i8::try_from(-(buf.len() as i64) - 2).unwrap();
            buf.join(Buf::from(jmp(buf_len)))
        }
        Statement::For {
            init,
            cond,
            update,
            body,
            pos,
        } => statementを評価(
            &Statement::Block {
                statements: vec![
                    init.clone().map(|init| Statement::Expr {
                        expr: init,
                        semicolon_pos: *pos,
                    }),
                    Some(Statement::While {
                        cond: cond
                            .clone()
                            .unwrap_or_else(|| Box::new(Expr::Numeric { val: 1, pos: *pos })),
                        body: Box::new(Statement::Block {
                            statements: vec![
                                Some(body.as_ref().clone()),
                                update.clone().map(|update| Statement::Expr {
                                    expr: update,
                                    semicolon_pos: *pos,
                                }),
                            ]
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>(),
                            pos: *pos,
                        }),
                        pos: *pos,
                    }),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
                pos: *pos,
            },
            idents,
            functions,
            stack_size,
        ),
        Statement::Block { statements, .. } => statements.iter().fold(Buf::new(), |acc, stmt| {
            acc.join(statementを評価(stmt, idents, functions, stack_size))
        }),
    }
}

pub fn programを評価(
    program: &Program,
    idents: &mut HashMap<String, u8>,
    functions: &mut HashMap<String, u32>,
    stack_size: &mut u32,
) -> Buf {
    match program {
        Program::Statements(statements) => statements
            .iter()
            .map(|stmt| statementを評価(stmt, idents, &*functions, stack_size))
            .fold(Buf::new(), Buf::join),
    }
}

#[allow(clippy::too_many_lines)]
pub fn exprを評価してediレジスタへ(
    writer: &mut impl Write,
    expr: &Expr,
    idents: &mut HashMap<String, u8>,
    functions: &HashMap<String, u32>,
    stack_size: &mut u32,
) {
    match expr {
        Expr::BinaryExpr {
            op: BinaryOp::Assign,
            op_pos: _,
            左辺,
            右辺,
        } => {
            exprを左辺値として評価してアドレスをrdiレジスタへ(
                writer, 左辺, idents, stack_size,
            );
            writer.write_all(&ediをプッシュ()).unwrap();
            *stack_size += 4;
            exprを評価してediレジスタへ(writer, 右辺, idents, functions, stack_size);

            writer.write_all(&eaxへとポップ()).unwrap(); // 左辺のアドレス
            *stack_size -= 4;
            writer.write_all(&raxが指す位置にediを代入()).unwrap();
        }
        Expr::Identifier { .. } => {
            exprを左辺値として評価してアドレスをrdiレジスタへ(
                writer, expr, idents, stack_size,
            );
            writer.write_all(&rdiを間接参照()).unwrap();
        }
        Expr::BinaryExpr {
            op: BinaryOp::AndThen,
            op_pos: _,
            左辺,
            右辺,
        } => {
            exprを評価してediレジスタへ(writer, 左辺, idents, functions, stack_size); // 左辺は push せずに捨てる
            exprを評価してediレジスタへ(writer, 右辺, idents, functions, stack_size);
        }
        Expr::BinaryExpr {
            op: BinaryOp::Add,
            op_pos: _,
            左辺,
            右辺,
        } => {
            exprを評価してediレジスタへ(writer, 左辺, idents, functions, stack_size);
            writer.write_all(&ediをプッシュ()).unwrap();
            *stack_size += 4;
            exprを評価してediレジスタへ(writer, 右辺, idents, functions, stack_size);
            writer.write_all(&ediをプッシュ()).unwrap();
            *stack_size += 4;
            writer.write_all(&eaxへとポップ()).unwrap();
            *stack_size -= 4;
            writer.write_all(&ediへとポップ()).unwrap();
            *stack_size -= 4;
            writer.write_all(&ediにeaxを足し合わせる()).unwrap();
        }
        Expr::BinaryExpr {
            op: BinaryOp::Sub,
            op_pos: _,
            左辺,
            右辺,
        } => {
            exprを評価してediレジスタへ(writer, 左辺, idents, functions, stack_size);
            writer.write_all(&ediをプッシュ()).unwrap();
            *stack_size += 4;
            exprを評価してediレジスタへ(writer, 右辺, idents, functions, stack_size);
            writer.write_all(&ediをプッシュ()).unwrap();
            *stack_size += 4;
            writer.write_all(&eaxへとポップ()).unwrap();
            *stack_size -= 4;
            writer.write_all(&ediへとポップ()).unwrap();
            *stack_size -= 4;
            writer.write_all(&ediからeaxを減じる()).unwrap();
        }
        Expr::BinaryExpr {
            op: BinaryOp::Mul,
            op_pos: _,
            左辺,
            右辺,
        } => {
            exprを評価してediレジスタへ(writer, 左辺, idents, functions, stack_size);
            writer.write_all(&ediをプッシュ()).unwrap();
            *stack_size += 4;
            exprを評価してediレジスタへ(writer, 右辺, idents, functions, stack_size);
            writer.write_all(&ediをプッシュ()).unwrap();
            *stack_size += 4;
            writer.write_all(&eaxへとポップ()).unwrap();
            *stack_size -= 4;
            writer.write_all(&ediへとポップ()).unwrap();
            *stack_size -= 4;
            writer.write_all(&ediをeax倍にする()).unwrap();
        }

        Expr::BinaryExpr {
            op: BinaryOp::Div,
            op_pos: _,
            左辺,
            右辺,
        } => {
            exprを評価してediレジスタへ(writer, 左辺, idents, functions, stack_size);
            writer.write_all(&ediをプッシュ()).unwrap();
            *stack_size += 4;
            exprを評価してediレジスタへ(writer, 右辺, idents, functions, stack_size);
            writer.write_all(&ediをプッシュ()).unwrap();
            *stack_size += 4;

            // 右辺を edi に、左辺を eax に入れる必要がある
            writer.write_all(&ediへとポップ()).unwrap();
            *stack_size -= 4;
            writer.write_all(&eaxへとポップ()).unwrap();
            *stack_size -= 4;

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
                idents,
                functions,
                stack_size,
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
                idents,
                functions,
                stack_size,
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
                idents,
                functions,
                stack_size,
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
                idents,
                functions,
                stack_size,
            );
        }
        Expr::Numeric { val, pos: _ } => {
            writer.write_all(&ediに代入(*val)).unwrap();
        }
        Expr::Call {
            ident,
            args,
            pos: _,
        } => {
            let function = functions
                .get(ident)
                .unwrap_or_else(|| panic!("関数 {} が見つかりません", ident));

            let stack_args_len = if args.len() > 6 { args.len() - 6 } else { 0 };

            let addrsp = *stack_size + 4 * stack_args_len as u32 % 16;
            writer
                .write_all(&rspから即値を引く(addrsp as u8).to_vec())
                .unwrap();
            *stack_size += addrsp;

            // 引数の評価順序変わるけど未規定のはずなのでよし
            for arg in args.iter().rev() {
                exprを評価してediレジスタへ(writer, arg, idents, functions, stack_size);
                writer.write_all(&ediをプッシュ()).unwrap();
                *stack_size += 4;
            }

            if args.len() >= 1 {
                writer.write_all(&ediへとポップ()).unwrap();
                *stack_size -= 4;
            }

            if args.len() >= 2 {
                writer.write_all(&esiへとポップ()).unwrap();
                *stack_size -= 4;
            }

            if args.len() >= 3 {
                writer.write_all(&edxへとポップ()).unwrap();
                *stack_size -= 4;
            }

            if args.len() >= 4 {
                writer.write_all(&ecxへとポップ()).unwrap();
                *stack_size -= 4;
            }

            if args.len() >= 5 {
                writer.write_all(&r8dへとポップ()).unwrap();
                *stack_size -= 4;
            }

            if args.len() >= 6 {
                writer.write_all(&r9dへとポップ()).unwrap();
                *stack_size -= 4;
            }

            writer
                .write_all(&eaxに即値をセット(*function + 0x00400000))
                .unwrap();
            writer.write_all(&call_rax()).unwrap();
            writer.write_all(&eaxをediにmov()).unwrap();
            writer
                .write_all(&rspに即値を足す(addrsp as u8).to_vec())
                .unwrap();
            *stack_size -= addrsp;
        }
    }
}

fn 比較演算を評価してediレジスタへ(
    writer: &mut impl Write,
    左辺: &Expr,
    右辺: &Expr,
    フラグをalに移す: &[u8],
    idents: &mut HashMap<String, u8>,
    functions: &HashMap<String, u32>,
    stack_size: &mut u32,
) {
    exprを評価してediレジスタへ(writer, 左辺, idents, functions, stack_size);
    writer.write_all(&ediをプッシュ()).unwrap();
    *stack_size += 4;
    exprを評価してediレジスタへ(writer, 右辺, idents, functions, stack_size);
    writer.write_all(&ediをプッシュ()).unwrap();
    *stack_size += 4;

    writer.write_all(&ediへとポップ()).unwrap();
    *stack_size -= 4;
    writer.write_all(&eaxへとポップ()).unwrap();
    *stack_size -= 4;

    writer
        .write_all(&eaxとediを比較してフラグをセット())
        .unwrap();

    writer.write_all(フラグをalに移す).unwrap();

    writer.write_all(&alをゼロ拡張してediにセット()).unwrap();
}
