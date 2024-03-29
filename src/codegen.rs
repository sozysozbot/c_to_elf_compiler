use crate::{
    ast::*,
    parse::toplevel::{FunctionDefinition, Type},
    Buf,
};
use std::collections::HashMap;

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

const WORD_SIZE: u8 = 8;
const WORD_SIZE_AS_I8: i8 = WORD_SIZE as i8;
const WORD_SIZE_AS_U32: u32 = WORD_SIZE as u32;

fn rdiから即値を引く(n: u8) -> [u8; 4] {
    [0x48, 0x83, 0xef, n]
}

fn raxから即値を引く(n: u8) -> [u8; 4] {
    [0x48, 0x83, 0xe8, n]
}

fn ediに代入(n: u8) -> [u8; 5] {
    [0xbf, n, 0x00, 0x00, 0x00]
}

fn rdiをプッシュ() -> [u8; 1] {
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

fn rdiへとポップ() -> [u8; 1] {
    [0x5f]
}

fn rsiへとポップ() -> [u8; 1] {
    [0x5e]
}

fn raxへとポップ() -> [u8; 1] {
    [0x58]
}

fn rdxへとポップ() -> [u8; 1] {
    [0x5a]
}

fn rcxへとポップ() -> [u8; 1] {
    [0x59]
}

fn r8へとポップ() -> [u8; 2] {
    [0x41, 0x58]
}

fn r9へとポップ() -> [u8; 2] {
    [0x41, 0x59]
}

fn rdiにraxを足し合わせる() -> [u8; 3] {
    [0x48, 0x01, 0xc7]
}

fn rdiからraxを減じる() -> [u8; 3] {
    [0x48, 0x29, 0xc7]
}

fn rdiをrax倍にする() -> [u8; 4] {
    [0x48, 0x0f, 0xaf, 0xf8]
}

fn eaxの符号ビットをedxへ拡張() -> [u8; 1] {
    [0x99]
}

fn edx_eaxをediで割る_商はeaxに_余りはedxに() -> [u8; 2] {
    [0xf7, 0xff]
}

fn raxをプッシュ() -> [u8; 1] {
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

fn rdiをmovzxで間接参照() -> [u8; 4] {
    [0x48, 0x0f, 0xb6, 0x3f]
}

fn raxが指す位置にrdiを代入() -> [u8; 3] {
    [0x48, 0x89, 0x38]
}

fn raxが指す位置にediを代入() -> [u8; 2] {
    [0x89, 0x38]
}

fn raxが指す位置にdilを代入() -> [u8; 3] {
    [0x40, 0x88, 0x38]
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

fn ediをeaxにmov() -> [u8; 2] {
    [0x89, 0xf8]
}

fn leave_ret() -> [u8; 2] {
    [0xc9, 0xc3]
}

fn rbpにoffsetを足した位置にediを代入(offset: i8) -> [u8; 3] {
    [0x89, 0x7d, offset.to_le_bytes()[0]]
}

fn rbpにoffsetを足した位置にesiを代入(offset: i8) -> [u8; 3] {
    [0x89, 0x75, offset.to_le_bytes()[0]]
}

fn rbpにoffsetを足した位置にedxを代入(offset: i8) -> [u8; 3] {
    [0x89, 0x55, offset.to_le_bytes()[0]]
}

fn rbpにoffsetを足した位置にecxを代入(offset: i8) -> [u8; 3] {
    [0x89, 0x4d, offset.to_le_bytes()[0]]
}

fn rbpにoffsetを足した位置にr8dを代入(offset: i8) -> [u8; 4] {
    [0x44, 0x89, 0x45, offset.to_le_bytes()[0]]
}

fn rbpにoffsetを足した位置にr9dを代入(offset: i8) -> [u8; 4] {
    [0x44, 0x89, 0x4d, offset.to_le_bytes()[0]]
}

pub fn builtin_three関数を生成() -> Buf {
    プロローグ(0).join(eaxに即値をセット(3)).join(エピローグ())
}

fn rbpにoffsetを足したアドレスをrsiに代入(offset: i8) -> [u8; 4] {
    [0x48, 0x8d, 0x75, offset.to_le_bytes()[0]]
}

fn rbpにoffsetを足したアドレスをrdiに代入(offset: i8) -> [u8; 4] {
    [0x48, 0x8d, 0x7d, offset.to_le_bytes()[0]]
}

pub fn builtin_putchar関数を生成() -> Buf {
    プロローグ(WORD_SIZE)
        .join(rbpにoffsetを足した位置にediを代入(
            -WORD_SIZE_AS_I8,
        ))
        .join(eaxに即値をセット(1)) // write
        .join(ediに代入(1)) // fd
        .join(rbpにoffsetを足したアドレスをrsiに代入(
            -WORD_SIZE_AS_I8,
        )) // buf
        .join(edxに即値をセット(1)) // count
        .join(syscall())
        .join(エピローグ())
}

/**
 * int *alloc4(int a, int b, int c, int d) {
 *     int *br = syscall(12, NULL); // On failure, the system call returns the current break
 *     int *new_br = syscall(12, br + 4); // the actual Linux system call returns the new program break on success
 *     new_br--; // new_br = br + 3
 *     *new_br = d;
 *     new_br--; // new_br = br + 2
 *     *new_br = c;
 *     new_br--; // new_br = br + 1
 *     *new_br = b;
 *     new_br--; // new_br = br
 *     *new_br = a;
 *     return new_br; // new_br == br
 * }
 */
pub fn builtin_alloc4関数を生成() -> Buf {
    プロローグ(WORD_SIZE * 4)
        .join(rbpにoffsetを足した位置にediを代入(
            -WORD_SIZE_AS_I8, // a
        ))
        .join(rbpにoffsetを足した位置にesiを代入(
            -WORD_SIZE_AS_I8 * 2, // b
        ))
        .join(rbpにoffsetを足した位置にedxを代入(
            -WORD_SIZE_AS_I8 * 3, // c
        ))
        .join(rbpにoffsetを足した位置にecxを代入(
            -WORD_SIZE_AS_I8 * 4, // d
        ))
        .join(eaxに即値をセット(12)) // sys_brk
        .join(ediに代入(0)) // NULL
        .join(syscall()) // rax: br
        .join(raxをプッシュ())
        .join(rdiへとポップ()) // rdi: br,
        .join(eaxに即値をセット(16)) // rax: 16, rdi: br
        .join(rdiにraxを足し合わせる()) // rdi: br + 16
        .join(eaxに即値をセット(12)) // rax: 12(sys_brk), rdi: br + 16
        .join(syscall()) // rax: br + 16
        //
        // br[3] = d
        .join(raxから即値を引く(4)) // rax: br + 12
        .join(rbpにoffsetを足したアドレスをrdiに代入(
            -WORD_SIZE_AS_I8 * 4,
        )) // rax: br + 12, rdi: &d
        .join(rdiを間接参照()) // rax: br + 12, rdi: d
        .join(raxが指す位置にediを代入()) // *(br + 12) = d;
        //
        // br[2] = c
        .join(raxから即値を引く(4)) // rax: br + 8
        .join(rbpにoffsetを足したアドレスをrdiに代入(
            -WORD_SIZE_AS_I8 * 3,
        )) // rax: br + 12, rdi: &c
        .join(rdiを間接参照()) // rax: br + 8, rdi: c
        .join(raxが指す位置にediを代入()) // *(br + 8) = c;
        //
        // br[1] = b
        .join(raxから即値を引く(4)) // rax: br + 4
        .join(rbpにoffsetを足したアドレスをrdiに代入(
            -WORD_SIZE_AS_I8 * 2,
        )) // rax: br + 12, rdi: &b
        .join(rdiを間接参照()) // rax: br + 4, rdi: b
        .join(raxが指す位置にediを代入()) // *(br + 4) = b;
        //
        // br[0] = a
        .join(raxから即値を引く(4)) // rax: br
        .join(rbpにoffsetを足したアドレスをrdiに代入(
            -WORD_SIZE_AS_I8,
        )) // rax: br, rdi: &a
        .join(rdiを間接参照()) // rax: br, rdi: a
        .join(raxが指す位置にediを代入()) // *br = a;
        .join(エピローグ())
}

pub struct LocalVarTable {
    pub offsets: HashMap<String, u8>,
    pub max_offset: u8,
}

impl LocalVarTable {
    pub fn allocate(&mut self, ident: &str, size: u8) -> u8 {
        let size = (size + WORD_SIZE - 1) / WORD_SIZE * WORD_SIZE;
        let offset = self
            .max_offset
            .checked_add(size)
            .expect("オフセットが u8 に収まりません");
        self.max_offset = offset;
        self.offsets.insert(ident.to_owned(), offset);
        offset
    }
}

pub struct FunctionGen<'a> {
    local_var_table: LocalVarTable,
    stack_size: u32,
    global_function_table: &'a HashMap<String, u32>,
    function_name: &'a str,
}

impl<'a> FunctionGen<'a> {
    pub fn exprを左辺値として評価してアドレスをrdiレジスタへ(
        &mut self,
        buf: &mut Buf,
        expr: &Expr,
    ) {
        match expr {
            Expr::Identifier {
                ident,
                pos: _,
                typ: _,
            } => {
                let offset = self.local_var_table.offsets.get(ident).unwrap_or_else(|| {
                    panic!(
                        "変数 {ident} は関数 {} 内で宣言されていません",
                        self.function_name
                    )
                });
                buf.append(rbpをプッシュ());
                buf.append(rdiへとポップ());
                buf.append(rdiから即値を引く(*offset));
            }
            Expr::UnaryExpr {
                op: UnaryOp::Deref,
                expr,
                ..
            } => {
                self.exprを評価してediレジスタへ(buf, expr);
            }
            _ => panic!("代入式の左辺に左辺値以外が来ています"),
        }
    }

    pub fn statementを評価(&mut self, stmt: &Statement) -> Buf {
        match stmt {
            Statement::Expr {
                expr,
                semicolon_pos: _,
            } => {
                let mut buf = Buf::new();
                self.exprを評価してediレジスタへ(&mut buf, expr);
                buf
            }
            Statement::Throw {
                expr,
                semicolon_pos: _,
            } => {
                let mut buf = Buf::new();
                self.exprを評価してediレジスタへ(&mut buf, expr);
                buf.append([0xb8, 0x3c, 0x00, 0x00, 0x00]);
                buf.append([0x0f, 0x05]);
                buf
            }
            Statement::Return {
                expr,
                semicolon_pos: _,
            } => {
                let mut buf = Buf::new();
                self.exprを評価してediレジスタへ(&mut buf, expr);
                buf.append(ediをeaxにmov());
                buf.append(leave_ret());
                buf
            }
            Statement::If {
                cond, then, else_, ..
            } => {
                let else_buf = else_
                    .as_ref()
                    .map(|else_| self.statementを評価(else_.as_ref()));

                let then_buf =  self.statementを評価(then.as_ref()).join(
                else_buf
                    .as_ref()
                    .map(|else_buf| {
                        Buf::from(
                            jmp(
                                i8::try_from(else_buf.len())
                                .unwrap_or_else(
                                    |_| panic!(
                                        "else でジャンプするためのバッファの長さが i8 に収まりません。バッファの長さは {}、中身は 0x[{}] です",
                                        else_buf.len(), else_buf.to_vec().iter().map(|a| format!("{a:02x}")).collect::<Vec<_>>().join(" ")
                                    )
                                )
                            )
                        )
                    })
                    .unwrap_or_else(Buf::new),
            );

                let mut cond_buf = Buf::new();
                self.exprを評価してediレジスタへ(&mut cond_buf, cond);
                cond_buf.append(ediが0かを確認());
                cond_buf.append(je(i8::try_from(then_buf.len()).unwrap()));

                cond_buf
                    .join(then_buf)
                    .join(else_buf.unwrap_or_else(Buf::new))
            }
            Statement::While { cond, body, .. } => {
                let body_buf = self.statementを評価(body.as_ref());

                let mut cond_buf = Buf::new();
                self.exprを評価してediレジスタへ(&mut cond_buf, cond);
                cond_buf.append(ediが0かを確認());
                cond_buf.append(je(i8::try_from(body_buf.len() + 2).unwrap()));

                let buf = cond_buf.join(body_buf);
                let buf_len = i8::try_from(-(buf.len() as i64) - 2).unwrap_or_else(
                |_| panic!("while 文の中でジャンプするためのバッファの長さが i8 に収まりません。バッファの長さは {}、中身は 0x[{}] です", buf.len(), buf.to_vec().iter().map(|a| format!("{a:02x}")).collect::<Vec<_>>().join(" "))
            );
                buf.join(Buf::from(jmp(buf_len)))
            }
            Statement::For {
                init,
                cond,
                update,
                body,
                pos,
            } => self.statementを評価(&Statement::Block {
                statements: vec![
                    init.clone().map(|init| Statement::Expr {
                        expr: init,
                        semicolon_pos: *pos,
                    }),
                    Some(Statement::While {
                        cond: cond.clone().unwrap_or_else(|| {
                            Box::new(Expr::Numeric {
                                val: 1,
                                pos: *pos,
                                typ: Type::Int,
                            })
                        }),
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
            }),
            Statement::Block { statements, .. } => statements
                .iter()
                .fold(Buf::new(), |acc, stmt| acc.join(self.statementを評価(stmt))),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn exprを評価してediレジスタへ(&mut self, buf: &mut Buf, expr: &Expr) {
        if matches!(expr.typ(), Type::Arr(_, _)) {
            self.exprを左辺値として評価してアドレスをrdiレジスタへ(buf, expr);
            return;
        }

        match expr {
            Expr::DecayedArr { expr, .. } => {
                self.exprを評価してediレジスタへ(buf, expr);
            }
            Expr::BinaryExpr {
                op: BinaryOp::Assign,
                op_pos: _,
                左辺,
                右辺,
                typ,
            } => {
                self.exprを左辺値として評価してアドレスをrdiレジスタへ(
                    buf, 左辺,
                );
                buf.append(rdiをプッシュ());
                self.stack_size += WORD_SIZE_AS_U32;
                self.exprを評価してediレジスタへ(buf, 右辺);

                buf.append(raxへとポップ()); // 左辺のアドレス
                self.stack_size -= WORD_SIZE_AS_U32;
                match typ.sizeof() {
                    8 => buf.append(raxが指す位置にrdiを代入()),
                    4 => buf.append(raxが指す位置にediを代入()),
                    1 => buf.append(raxが指す位置にdilを代入()),
                    _ => panic!("size が {} な型への代入はできません", typ.sizeof()),
                };
            }
            Expr::Identifier { .. } => {
                self.exprを左辺値として評価してアドレスをrdiレジスタへ(
                    buf, expr,
                );
                match expr.typ().sizeof() {
                    8 => buf.append(rdiを間接参照()),
                    4 => buf.append(rdiを間接参照()),
                    1 => buf.append(rdiをmovzxで間接参照()),
                    _ => panic!("size が {} な型の参照はできません", expr.typ().sizeof()),
                };
            }
            Expr::BinaryExpr {
                op: BinaryOp::AndThen,
                op_pos: _,
                左辺,
                右辺,
                typ: _,
            } => {
                self.exprを評価してediレジスタへ(buf, 左辺); // 左辺は push せずに捨てる
                self.exprを評価してediレジスタへ(buf, 右辺);
            }
            Expr::BinaryExpr {
                op: BinaryOp::Add,
                op_pos: _,
                左辺,
                右辺,
                typ: _,
            } => {
                self.exprを評価してediレジスタへ(buf, 左辺);
                buf.append(rdiをプッシュ());
                self.stack_size += WORD_SIZE_AS_U32;
                self.exprを評価してediレジスタへ(buf, 右辺);
                buf.append(rdiをプッシュ());
                self.stack_size += WORD_SIZE_AS_U32;
                buf.append(raxへとポップ());
                self.stack_size -= WORD_SIZE_AS_U32;
                buf.append(rdiへとポップ());
                self.stack_size -= WORD_SIZE_AS_U32;
                buf.append(rdiにraxを足し合わせる());
            }
            Expr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos: _,
                左辺,
                右辺,
                typ: _,
            } => {
                self.exprを評価してediレジスタへ(buf, 左辺);
                buf.append(rdiをプッシュ());
                self.stack_size += WORD_SIZE_AS_U32;
                self.exprを評価してediレジスタへ(buf, 右辺);
                buf.append(rdiをプッシュ());
                self.stack_size += WORD_SIZE_AS_U32;
                buf.append(raxへとポップ());
                self.stack_size -= WORD_SIZE_AS_U32;
                buf.append(rdiへとポップ());
                self.stack_size -= WORD_SIZE_AS_U32;
                buf.append(rdiからraxを減じる());
            }
            Expr::BinaryExpr {
                op: BinaryOp::Mul,
                op_pos: _,
                左辺,
                右辺,
                typ: _,
            } => {
                self.exprを評価してediレジスタへ(buf, 左辺);
                buf.append(rdiをプッシュ());
                self.stack_size += WORD_SIZE_AS_U32;
                self.exprを評価してediレジスタへ(buf, 右辺);
                buf.append(rdiをプッシュ());
                self.stack_size += WORD_SIZE_AS_U32;
                buf.append(raxへとポップ());
                self.stack_size -= WORD_SIZE_AS_U32;
                buf.append(rdiへとポップ());
                self.stack_size -= WORD_SIZE_AS_U32;
                buf.append(rdiをrax倍にする())
            }

            Expr::BinaryExpr {
                op: BinaryOp::Div,
                op_pos: _,
                左辺,
                右辺,
                typ: _,
            } => {
                self.exprを評価してediレジスタへ(buf, 左辺);
                buf.append(rdiをプッシュ());
                self.stack_size += WORD_SIZE_AS_U32;
                self.exprを評価してediレジスタへ(buf, 右辺);
                buf.append(rdiをプッシュ());
                self.stack_size += WORD_SIZE_AS_U32;

                // 右辺を edi に、左辺を eax に入れる必要がある
                buf.append(rdiへとポップ());
                self.stack_size -= WORD_SIZE_AS_U32;
                buf.append(raxへとポップ());
                self.stack_size -= WORD_SIZE_AS_U32;

                buf.append(eaxの符号ビットをedxへ拡張());
                buf.append(edx_eaxをediで割る_商はeaxに_余りはedxに());

                // 結果は eax レジスタに入るので、ediに移し替える
                buf.append(raxをプッシュ());
                buf.append(rdiへとポップ());
            }
            Expr::BinaryExpr {
                op: BinaryOp::Equal,
                op_pos: _,
                左辺,
                右辺,
                typ: _,
            } => {
                self.比較演算を評価してediレジスタへ(
                    buf,
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
                typ: _,
            } => {
                self.比較演算を評価してediレジスタへ(
                    buf,
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
                typ: _,
            } => {
                self.比較演算を評価してediレジスタへ(
                    buf,
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
                typ: _,
            } => {
                self.比較演算を評価してediレジスタへ(
                    buf,
                    左辺,
                    右辺,
                    &フラグを読んで以下であるかどうかをalにセット(),
                );
            }
            Expr::Numeric {
                val,
                pos: _,
                typ: _,
            } => {
                buf.append(ediに代入(*val));
            }
            Expr::StringLiteral {
                val: _,
                pos: _,
                typ: _,
            } => {
                unimplemented!("文字列リテラルを値として扱うのは未実装です（必ず sizeof のオペランドにしてください）");
            }
            Expr::Call {
                ident,
                args,
                pos: _,
                typ: _,
            } => {
                let function = *self
                    .global_function_table
                    .get(ident)
                    .unwrap_or_else(|| panic!("関数 {ident} が見つかりません"));

                let stack_args_len = if args.len() > 6 { args.len() - 6 } else { 0 };

                let stack_size_adjustment =
                    self.stack_size + WORD_SIZE_AS_U32 * stack_args_len as u32 % 16;
                buf.append(rspから即値を引く(stack_size_adjustment as u8).to_vec());
                self.stack_size += stack_size_adjustment;

                // 引数の評価順序変わるけど未規定のはずなのでよし
                for arg in args.iter().rev() {
                    self.exprを評価してediレジスタへ(buf, arg);
                    buf.append(rdiをプッシュ());
                    self.stack_size += WORD_SIZE_AS_U32;
                }

                #[allow(clippy::len_zero)]
                if args.len() >= 1 {
                    buf.append(rdiへとポップ());
                    self.stack_size -= WORD_SIZE_AS_U32;
                }

                if args.len() >= 2 {
                    buf.append(rsiへとポップ());
                    self.stack_size -= WORD_SIZE_AS_U32;
                }

                if args.len() >= 3 {
                    buf.append(rdxへとポップ());
                    self.stack_size -= WORD_SIZE_AS_U32;
                }

                if args.len() >= 4 {
                    buf.append(rcxへとポップ());
                    self.stack_size -= WORD_SIZE_AS_U32;
                }

                if args.len() >= 5 {
                    buf.append(r8へとポップ());
                    self.stack_size -= WORD_SIZE_AS_U32;
                }

                if args.len() >= 6 {
                    buf.append(r9へとポップ());
                    self.stack_size -= WORD_SIZE_AS_U32;
                }

                buf.append(eaxに即値をセット(function + 0x00400000));
                buf.append(call_rax());
                buf.append(eaxをediにmov());
                buf.append(rspに即値を足す(stack_size_adjustment as u8).to_vec());
                self.stack_size -= stack_size_adjustment;
            }
            Expr::UnaryExpr {
                op: UnaryOp::Addr,
                op_pos: _,
                expr,
                typ: _,
            } => {
                self.exprを左辺値として評価してアドレスをrdiレジスタへ(
                    buf, expr,
                );
            }
            Expr::UnaryExpr { .. } => {
                self.exprを左辺値として評価してアドレスをrdiレジスタへ(
                    buf, expr,
                );
                buf.append(rdiを間接参照());
            }
        }
    }

    fn 比較演算を評価してediレジスタへ(
        &mut self,
        buf: &mut Buf,
        左辺: &Expr,
        右辺: &Expr,
        フラグをalに移す: &[u8],
    ) {
        self.exprを評価してediレジスタへ(buf, 左辺);
        buf.append(rdiをプッシュ());
        self.stack_size += WORD_SIZE_AS_U32;
        self.exprを評価してediレジスタへ(buf, 右辺);
        buf.append(rdiをプッシュ());
        self.stack_size += WORD_SIZE_AS_U32;

        buf.append(rdiへとポップ());
        self.stack_size -= WORD_SIZE_AS_U32;
        buf.append(raxへとポップ());
        self.stack_size -= WORD_SIZE_AS_U32;

        buf.append(eaxとediを比較してフラグをセット());
        buf.append(フラグをalに移す);
        buf.append(alをゼロ拡張してediにセット());
    }
}

pub fn 関数をコード生成しメインバッファとグローバル関数テーブルに挿入(
    global_function_table: &mut HashMap<String, u32>,
    main_buf: &mut Buf,
    definition: &FunctionDefinition,
) -> u16 {
    let func_pos = u16::try_from(main_buf.len()).expect("バッファの長さが u16 に収まりません");
    global_function_table.insert(definition.func_name.clone(), u32::from(func_pos));

    let mut function_gen = FunctionGen {
        local_var_table: LocalVarTable {
            offsets: HashMap::new(),
            max_offset: 0,
        },
        stack_size: 0,
        global_function_table,
        function_name: &definition.func_name,
    };
    main_buf.append(rbpをプッシュ());
    function_gen.stack_size += 8;
    main_buf.append(rspをrbpにコピー());

    let mut parameter_buf = Buf::new();
    let _return_type = &definition.return_type;

    for (i, (param_type, param)) in definition.params.iter().enumerate() {
        if function_gen.local_var_table.offsets.contains_key(param) {
            panic!(
                "関数 `{}` の仮引数 {} が重複しています",
                definition.func_name, param
            )
        }
        let offset = function_gen
            .local_var_table
            .allocate(param, param_type.sizeof());
        // rbp から offset を引いた値のアドレスに、レジスタから読んできた値を入れる必要がある
        // （関数 `exprを左辺値として評価してアドレスをrdiレジスタへ` も参照）
        let negative_offset: i8 = -(offset as i8);
        match i {
            0 => parameter_buf.append(rbpにoffsetを足した位置にediを代入(
                negative_offset,
            )),
            1 => parameter_buf.append(rbpにoffsetを足した位置にesiを代入(
                negative_offset,
            )),
            2 => parameter_buf.append(rbpにoffsetを足した位置にedxを代入(
                negative_offset,
            )),
            3 => parameter_buf.append(rbpにoffsetを足した位置にecxを代入(
                negative_offset,
            )),
            4 => parameter_buf.append(rbpにoffsetを足した位置にr8dを代入(
                negative_offset,
            )),
            5 => parameter_buf.append(rbpにoffsetを足した位置にr9dを代入(
                negative_offset,
            )),
            _ => panic!(
                "関数 `{}` に 7 つ以上の仮引数があります",
                definition.func_name
            ),
        };
    }

    for (local_var_name, local_var_type) in definition.local_var_declarations.iter() {
        if function_gen
            .local_var_table
            .offsets
            .contains_key(local_var_name)
        {
            panic!(
                "関数 `{}` 先頭で定義されているローカル変数 {} が仮引数またはローカル変数と重複しています",
                definition.func_name, local_var_name
            )
        }
        function_gen
            .local_var_table
            .allocate(local_var_name, local_var_type.sizeof());
    }

    let content_buf = definition
        .statements
        .iter()
        .map(|stmt| function_gen.statementを評価(stmt))
        .fold(parameter_buf, Buf::join);

    main_buf.append(rspから即値を引く(
        u8::try_from(function_gen.local_var_table.max_offset as usize)
            .expect("ローカル変数のオフセットが u8 に収まりません"),
    ));
    main_buf.append(content_buf);

    func_pos
}
