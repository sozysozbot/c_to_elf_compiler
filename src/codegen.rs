use crate::{ast::*, parser::FunctionDefinition, Buf};
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

const WORD_SIZE: u8 = 8;
const WORD_SIZE_AS_I8: i8 = WORD_SIZE as i8;
const WORD_SIZE_AS_U32: u32 = WORD_SIZE as u32;

fn rdiから即値を引く(n: u8) -> [u8; 4] {
    [0x48, 0x83, 0xef, n]
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

fn raxが指す位置にrdiを代入() -> [u8; 3] {
    [0x48, 0x89, 0x38]
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

pub struct FunctionGen<'a> {
    local_var_table: HashMap<String, u8>,
    stack_size: u32,
    global_function_table: &'a HashMap<String, u32>,
    function_name: &'a str,
}

impl<'a> FunctionGen<'a> {
    pub fn exprを左辺値として評価してアドレスをrdiレジスタへ(
        &mut self,
        writer: &mut impl Write,
        expr: &Expr,
    ) {
        match expr {
            Expr::Identifier { ident, pos: _ } => {
                if !self.local_var_table.contains_key(ident) {
                    panic!(
                        "変数 {ident} は関数 {} 内で宣言されていません",
                        self.function_name
                    )
                }
                let len = self.local_var_table.len();
                let idx = self
                    .local_var_table
                    .entry(ident.clone())
                    .or_insert(len as u8);
                let offset = *idx * WORD_SIZE + WORD_SIZE;
                writer.write_all(&rbpをプッシュ()).unwrap();
                writer.write_all(&rdiへとポップ()).unwrap();
                writer.write_all(&rdiから即値を引く(offset)).unwrap();
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
                let mut writer = Vec::new();
                self.exprを評価してediレジスタへ(&mut writer, expr);
                Buf::from(writer)
            }
            Statement::Throw {
                expr,
                semicolon_pos: _,
            } => {
                let mut writer = Vec::new();
                self.exprを評価してediレジスタへ(&mut writer, expr);
                writer.write_all(&[0xb8, 0x3c, 0x00, 0x00, 0x00]).unwrap();
                writer.write_all(&[0x0f, 0x05]).unwrap();
                Buf::from(writer)
            }
            Statement::Return {
                expr,
                semicolon_pos: _,
            } => {
                let mut writer = Vec::new();
                self.exprを評価してediレジスタへ(&mut writer, expr);
                writer.write_all(&ediをeaxにmov()).unwrap();
                writer.write_all(&leave_ret()).unwrap();
                Buf::from(writer)
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
                                        else_buf.len(), else_buf.to_vec().iter().map(|a| format!("{:02x}", a)).collect::<Vec<_>>().join(" ")
                                    )
                                )
                            )
                        )
                    })
                    .unwrap_or_else(Buf::new),
            );

                let cond_buf = {
                    let mut v = Vec::new();
                    self.exprを評価してediレジスタへ(&mut v, cond);
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
                let body_buf = self.statementを評価(body.as_ref());
                let cond_buf = {
                    let mut v = Vec::new();
                    self.exprを評価してediレジスタへ(&mut v, cond);
                    v.write_all(&ediが0かを確認()).unwrap();
                    v.write_all(&je(i8::try_from(body_buf.len() + 2).unwrap()))
                        .unwrap();
                    Buf::from(v)
                };
                let buf = cond_buf.join(body_buf);
                let buf_len = i8::try_from(-(buf.len() as i64) - 2).unwrap_or_else(
                |_| panic!("while 文の中でジャンプするためのバッファの長さが i8 に収まりません。バッファの長さは {}、中身は 0x[{}] です", buf.len(), buf.to_vec().iter().map(|a| format!("{:02x}", a)).collect::<Vec<_>>().join(" "))
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
            }),
            Statement::Block { statements, .. } => statements
                .iter()
                .fold(Buf::new(), |acc, stmt| acc.join(self.statementを評価(stmt))),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn exprを評価してediレジスタへ(&mut self, writer: &mut impl Write, expr: &Expr) {
        match expr {
            Expr::BinaryExpr {
                op: BinaryOp::Assign,
                op_pos: _,
                左辺,
                右辺,
            } => {
                self.exprを左辺値として評価してアドレスをrdiレジスタへ(
                    writer, 左辺,
                );
                writer.write_all(&rdiをプッシュ()).unwrap();
                self.stack_size += WORD_SIZE_AS_U32;
                self.exprを評価してediレジスタへ(writer, 右辺);

                writer.write_all(&raxへとポップ()).unwrap(); // 左辺のアドレス
                self.stack_size -= WORD_SIZE_AS_U32;
                writer.write_all(&raxが指す位置にrdiを代入()).unwrap();
            }
            Expr::Identifier { .. } => {
                self.exprを左辺値として評価してアドレスをrdiレジスタへ(
                    writer, expr,
                );
                writer.write_all(&rdiを間接参照()).unwrap();
            }
            Expr::BinaryExpr {
                op: BinaryOp::AndThen,
                op_pos: _,
                左辺,
                右辺,
            } => {
                self.exprを評価してediレジスタへ(writer, 左辺); // 左辺は push せずに捨てる
                self.exprを評価してediレジスタへ(writer, 右辺);
            }
            Expr::BinaryExpr {
                op: BinaryOp::Add,
                op_pos: _,
                左辺,
                右辺,
            } => {
                self.exprを評価してediレジスタへ(writer, 左辺);
                writer.write_all(&rdiをプッシュ()).unwrap();
                self.stack_size += WORD_SIZE_AS_U32;
                self.exprを評価してediレジスタへ(writer, 右辺);
                writer.write_all(&rdiをプッシュ()).unwrap();
                self.stack_size += WORD_SIZE_AS_U32;
                writer.write_all(&raxへとポップ()).unwrap();
                self.stack_size -= WORD_SIZE_AS_U32;
                writer.write_all(&rdiへとポップ()).unwrap();
                self.stack_size -= WORD_SIZE_AS_U32;
                writer.write_all(&rdiにraxを足し合わせる()).unwrap();
            }
            Expr::BinaryExpr {
                op: BinaryOp::Sub,
                op_pos: _,
                左辺,
                右辺,
            } => {
                self.exprを評価してediレジスタへ(writer, 左辺);
                writer.write_all(&rdiをプッシュ()).unwrap();
                self.stack_size += WORD_SIZE_AS_U32;
                self.exprを評価してediレジスタへ(writer, 右辺);
                writer.write_all(&rdiをプッシュ()).unwrap();
                self.stack_size += WORD_SIZE_AS_U32;
                writer.write_all(&raxへとポップ()).unwrap();
                self.stack_size -= WORD_SIZE_AS_U32;
                writer.write_all(&rdiへとポップ()).unwrap();
                self.stack_size -= WORD_SIZE_AS_U32;
                writer.write_all(&rdiからraxを減じる()).unwrap();
            }
            Expr::BinaryExpr {
                op: BinaryOp::Mul,
                op_pos: _,
                左辺,
                右辺,
            } => {
                self.exprを評価してediレジスタへ(writer, 左辺);
                writer.write_all(&rdiをプッシュ()).unwrap();
                self.stack_size += WORD_SIZE_AS_U32;
                self.exprを評価してediレジスタへ(writer, 右辺);
                writer.write_all(&rdiをプッシュ()).unwrap();
                self.stack_size += WORD_SIZE_AS_U32;
                writer.write_all(&raxへとポップ()).unwrap();
                self.stack_size -= WORD_SIZE_AS_U32;
                writer.write_all(&rdiへとポップ()).unwrap();
                self.stack_size -= WORD_SIZE_AS_U32;
                writer.write_all(&rdiをrax倍にする()).unwrap();
            }

            Expr::BinaryExpr {
                op: BinaryOp::Div,
                op_pos: _,
                左辺,
                右辺,
            } => {
                self.exprを評価してediレジスタへ(writer, 左辺);
                writer.write_all(&rdiをプッシュ()).unwrap();
                self.stack_size += WORD_SIZE_AS_U32;
                self.exprを評価してediレジスタへ(writer, 右辺);
                writer.write_all(&rdiをプッシュ()).unwrap();
                self.stack_size += WORD_SIZE_AS_U32;

                // 右辺を edi に、左辺を eax に入れる必要がある
                writer.write_all(&rdiへとポップ()).unwrap();
                self.stack_size -= WORD_SIZE_AS_U32;
                writer.write_all(&raxへとポップ()).unwrap();
                self.stack_size -= WORD_SIZE_AS_U32;

                writer.write_all(&eaxの符号ビットをedxへ拡張()).unwrap();
                writer
                    .write_all(&edx_eaxをediで割る_商はeaxに_余りはedxに())
                    .unwrap();

                // 結果は eax レジスタに入るので、ediに移し替える
                writer.write_all(&raxをプッシュ()).unwrap();
                writer.write_all(&rdiへとポップ()).unwrap();
            }
            Expr::BinaryExpr {
                op: BinaryOp::Equal,
                op_pos: _,
                左辺,
                右辺,
            } => {
                self.比較演算を評価してediレジスタへ(
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
                self.比較演算を評価してediレジスタへ(
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
                self.比較演算を評価してediレジスタへ(
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
                self.比較演算を評価してediレジスタへ(
                    writer,
                    左辺,
                    右辺,
                    &フラグを読んで以下であるかどうかをalにセット(),
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
                let function = *self
                    .global_function_table
                    .get(ident)
                    .unwrap_or_else(|| panic!("関数 {} が見つかりません", ident));

                let stack_args_len = if args.len() > 6 { args.len() - 6 } else { 0 };

                let addrsp = self.stack_size + WORD_SIZE_AS_U32 * stack_args_len as u32 % 16;
                writer
                    .write_all(&rspから即値を引く(addrsp as u8).to_vec())
                    .unwrap();
                self.stack_size += addrsp;

                // 引数の評価順序変わるけど未規定のはずなのでよし
                for arg in args.iter().rev() {
                    self.exprを評価してediレジスタへ(writer, arg);
                    writer.write_all(&rdiをプッシュ()).unwrap();
                    self.stack_size += WORD_SIZE_AS_U32;
                }

                #[allow(clippy::len_zero)]
                if args.len() >= 1 {
                    writer.write_all(&rdiへとポップ()).unwrap();
                    self.stack_size -= WORD_SIZE_AS_U32;
                }

                if args.len() >= 2 {
                    writer.write_all(&rsiへとポップ()).unwrap();
                    self.stack_size -= WORD_SIZE_AS_U32;
                }

                if args.len() >= 3 {
                    writer.write_all(&rdxへとポップ()).unwrap();
                    self.stack_size -= WORD_SIZE_AS_U32;
                }

                if args.len() >= 4 {
                    writer.write_all(&rcxへとポップ()).unwrap();
                    self.stack_size -= WORD_SIZE_AS_U32;
                }

                if args.len() >= 5 {
                    writer.write_all(&r8へとポップ()).unwrap();
                    self.stack_size -= WORD_SIZE_AS_U32;
                }

                if args.len() >= 6 {
                    writer.write_all(&r9へとポップ()).unwrap();
                    self.stack_size -= WORD_SIZE_AS_U32;
                }

                writer
                    .write_all(&eaxに即値をセット(function + 0x00400000))
                    .unwrap();
                writer.write_all(&call_rax()).unwrap();
                writer.write_all(&eaxをediにmov()).unwrap();
                writer
                    .write_all(&rspに即値を足す(addrsp as u8).to_vec())
                    .unwrap();
                self.stack_size -= addrsp;
            }
            Expr::UnaryExpr {
                op: UnaryOp::Addr,
                op_pos: _,
                expr,
            } => {
                self.exprを左辺値として評価してアドレスをrdiレジスタへ(
                    writer, expr,
                );
            }
            Expr::UnaryExpr {
                op: UnaryOp::Deref,
                op_pos: _,
                expr,
            } => {
                self.exprを評価してediレジスタへ(writer, expr);
                writer.write_all(&rdiを間接参照()).unwrap();
            }
        }
    }

    fn 比較演算を評価してediレジスタへ(
        &mut self,
        writer: &mut impl Write,
        左辺: &Expr,
        右辺: &Expr,
        フラグをalに移す: &[u8],
    ) {
        self.exprを評価してediレジスタへ(writer, 左辺);
        writer.write_all(&rdiをプッシュ()).unwrap();
        self.stack_size += WORD_SIZE_AS_U32;
        self.exprを評価してediレジスタへ(writer, 右辺);
        writer.write_all(&rdiをプッシュ()).unwrap();
        self.stack_size += WORD_SIZE_AS_U32;

        writer.write_all(&rdiへとポップ()).unwrap();
        self.stack_size -= WORD_SIZE_AS_U32;
        writer.write_all(&raxへとポップ()).unwrap();
        self.stack_size -= WORD_SIZE_AS_U32;

        writer
            .write_all(&eaxとediを比較してフラグをセット())
            .unwrap();

        writer.write_all(フラグをalに移す).unwrap();

        writer.write_all(&alをゼロ拡張してediにセット()).unwrap();
    }
}

pub fn 関数をコード生成しメインバッファとグローバル関数テーブルに挿入(
    global_function_table: &mut HashMap<String, u32>,
    main_buf: &mut Buf,
    definition: &FunctionDefinition,
) -> u16 {
    let buf = std::mem::take(main_buf);
    let func_pos = u16::try_from(buf.len()).expect("バッファの長さが u16 に収まりません");
    global_function_table.insert(definition.func_name.clone(), u32::from(func_pos));

    let mut function_gen = FunctionGen {
        local_var_table: HashMap::new(),
        stack_size: 0,
        global_function_table,
        function_name: &definition.func_name,
    };
    let buf = buf.join(rbpをプッシュ());
    function_gen.stack_size += 8;
    let buf = buf.join(rspをrbpにコピー());

    let content_buf = match &definition.content {
        FunctionContent::Statements(statements) => {
            let mut parameter_buf = Buf::new();
            let _return_type = &definition.return_type;
            for (i, (_param_type, param)) in definition.params.iter().enumerate() {
                let tmp_buf = std::mem::take(&mut parameter_buf);

                let len = function_gen.local_var_table.len();
                if function_gen.local_var_table.contains_key(&param.ident) {
                    panic!(
                        "関数 `{}` の仮引数 {} が重複しています",
                        definition.func_name, param.ident
                    )
                }
                let idx = function_gen
                    .local_var_table
                    .entry(param.ident.clone())
                    .or_insert(len as u8);
                let offset = *idx * WORD_SIZE + WORD_SIZE;
                // rbp から offset を引いた値のアドレスに、レジスタから読んできた値を入れる必要がある
                // （関数 `exprを左辺値として評価してアドレスをrdiレジスタへ` も参照）
                let negative_offset: i8 = -(offset as i8);
                let tmp_buf = match i {
                    0 => tmp_buf.join(rbpにoffsetを足した位置にediを代入(
                        negative_offset,
                    )),
                    1 => tmp_buf.join(rbpにoffsetを足した位置にesiを代入(
                        negative_offset,
                    )),
                    2 => tmp_buf.join(rbpにoffsetを足した位置にedxを代入(
                        negative_offset,
                    )),
                    3 => tmp_buf.join(rbpにoffsetを足した位置にecxを代入(
                        negative_offset,
                    )),
                    4 => tmp_buf.join(rbpにoffsetを足した位置にr8dを代入(
                        negative_offset,
                    )),
                    5 => tmp_buf.join(rbpにoffsetを足した位置にr9dを代入(
                        negative_offset,
                    )),
                    _ => panic!(
                        "関数 `{}` に 7 つ以上の仮引数があります",
                        definition.func_name
                    ),
                };
                parameter_buf = tmp_buf;
            }

            for (local_var_name, _local_var_type) in definition.local_var_declarations.iter() {
                let len = function_gen.local_var_table.len();
                if function_gen.local_var_table.contains_key(local_var_name) {
                    panic!(
                        "関数 `{}` 先頭で定義されているローカル変数 {} が仮引数またはローカル変数と重複しています",
                        definition.func_name, local_var_name
                    )
                }
                function_gen
                    .local_var_table
                    .entry(local_var_name.clone())
                    .or_insert(len as u8);
            }

            statements
                .iter()
                .map(|stmt| function_gen.statementを評価(stmt))
                .fold(parameter_buf, Buf::join)
        }
    };

    let buf = buf.join(rspから即値を引く(
        u8::try_from(function_gen.local_var_table.len() * WORD_SIZE as usize)
            .expect("識別子の個数が u8 に収まりません"),
    ));
    let buf = buf.join(content_buf);

    *main_buf = buf;
    func_pos
}
