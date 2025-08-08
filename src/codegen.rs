use crate::{
    ast::*, parse::{
        toplevel::{FunctionDefinition, TypeAndSize},
        typ::Type,
    }, x86_64_no_arg::*, x86_64_with_arg::*, Buf
};
use core::panic;
use std::collections::HashMap;


const WORD_SIZE: u8 = 8;
const WORD_SIZE_AS_I8: i8 = WORD_SIZE as i8;
const WORD_SIZE_AS_U32: u32 = WORD_SIZE as u32;
const WORD_SIZE_AS_I32: i32 = WORD_SIZE as i32;


pub fn builtin_three関数を生成() -> Buf {
    プロローグ(0).join(eaxに即値をセット(3)).join(エピローグ())
}


pub fn builtin_putchar関数を生成() -> Buf {
    プロローグ(WORD_SIZE_AS_I32)
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
    プロローグ(WORD_SIZE_AS_I32 * 4)
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
    pub offsets: Vec<(String, u64, i32)>,
    pub max_offset: i32,
}

impl LocalVarTable {
    pub fn allocate(&mut self, ident: &str, id: u64, size: i32) -> i32 {
        let size = ((size as u32).div_ceil(WORD_SIZE_AS_U32) * WORD_SIZE_AS_U32) as i32;
        let offset = self
            .max_offset
            .checked_add(size)
            .expect("オフセットが i32 に収まりません");
        self.max_offset = offset;
        self.offsets.push((ident.to_owned(), id, offset));
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
                local_var_id: Some(local_var_id),
                ..
            } => {
                // If the name exists but the id does not match, report
                let candidates = self
                    .local_var_table
                    .offsets
                    .iter()
                    .filter(|(i, _, _)| i == &ident.to_owned())
                    .collect::<Vec<_>>();

                let offset: i32 = self
                    .local_var_table
                    .offsets
                    .iter()
                    .find(|(i, l, _)| i == &ident.to_owned() && l == &local_var_id.to_owned())
                    .unwrap_or_else(|| {
                        panic!(
                            "関数 {} 内で、変数 {ident} は id {local_var_id} で参照されているが、id の候補は {candidates:?} です",
                            self.function_name
                        )
                    })
                    .2;
                buf.append(rbpをプッシュ());
                buf.append(rdiへとポップ());
                buf.append(rdiから即値を引く(offset));
            }
            Expr::Identifier {
                ident,
                pos: _,
                local_var_id: None,
                ..
            } => {
                panic!("グローバル変数 {ident} は今のところ左辺値として使用できません");
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

    pub fn statement_or_declarationを評価(
        &mut self,
        stmt_or_decl: &StatementOrDeclaration,
    ) -> Buf {
        let stmt = match stmt_or_decl {
            StatementOrDeclaration::Statement(stmt) => stmt.to_owned(),
            StatementOrDeclaration::Declaration { .. } => {
                return Buf::new(); // declaration disappears in codegen
            }
            StatementOrDeclaration::DeclarationWithInitializer {
                name,
                id,
                typ_and_size,
                initializer,
            } => {
                // compile to code that assigns the initializer to the variable
                Statement::Expr {
                    expr: Box::new(Expr::BinaryExpr {
                        op: BinaryOp::Assign,
                        左辺: Box::new(Expr::Identifier {
                            ident: name.clone(),
                            pos: 0, // pos is not used in codegen
                            typ: typ_and_size.typ.clone(),
                            local_var_id: Some(*id),
                        }),
                        右辺: initializer.clone(),
                        op_pos: 0, // op_pos is not used in codegen
                        typ: typ_and_size.typ.clone(),
                    }),
                    semicolon_pos: 0, // semicolon_pos is not used in codegen
                }
            }
        };
        self.statementを評価(&stmt)
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
                    .map(|else_| self.statement_or_declarationを評価(else_.as_ref()));

                let then_buf =  self.statement_or_declarationを評価(then.as_ref()).join(
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

                match cond.typ().sizeof_primitive("if") {
                    8 => cond_buf.append(rdiが0かを確認()),
                    4 => cond_buf.append(ediが0かを確認()),
                    1 => cond_buf.append(dilが0かを確認()),
                    _ => panic!("条件式の型のサイズがよろしくない"),
                }

                cond_buf.append(je(i8::try_from(then_buf.len()).unwrap()));

                cond_buf
                    .join(then_buf)
                    .join(else_buf.unwrap_or_else(Buf::new))
            }
            Statement::While { cond, body, .. } => {
                let body_buf = self.statement_or_declarationを評価(body.as_ref());

                let mut cond_buf = Buf::new();
                self.exprを評価してediレジスタへ(&mut cond_buf, cond);
                match cond.typ().sizeof_primitive("if") {
                    8 => cond_buf.append(rdiが0かを確認()),
                    4 => cond_buf.append(ediが0かを確認()),
                    1 => cond_buf.append(dilが0かを確認()),
                    _ => panic!("条件式の型のサイズがよろしくない"),
                }
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
            } => {
                let body: Box<StatementOrDeclaration> =
                    Box::new(StatementOrDeclaration::Statement(Statement::Block {
                        statements: vec![
                            Some(body.as_ref().clone()),
                            update.clone().map(|update| {
                                StatementOrDeclaration::Statement(Statement::Expr {
                                    expr: update,
                                    semicolon_pos: *pos,
                                })
                            }),
                        ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>(),
                        pos: *pos,
                    }));
                let statements: Vec<StatementOrDeclaration> = vec![
                    *init.clone(),
                    StatementOrDeclaration::Statement(Statement::While {
                        cond: cond.clone().unwrap_or_else(|| {
                            Box::new(Expr::Numeric {
                                val: 1,
                                pos: *pos,
                                typ: Type::Int,
                            })
                        }),
                        body,
                        pos: *pos,
                    }),
                ];
                self.statementを評価(&Statement::Block {
                    statements,
                    pos: *pos,
                })
            }
            Statement::Block { statements, .. } => {
                statements.iter().fold(Buf::new(), |acc, stmt| {
                    acc.join(self.statement_or_declarationを評価(stmt))
                })
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn exprを評価してediレジスタへ(&mut self, buf: &mut Buf, expr: &Expr) {
        if matches!(expr.typ(), Type::Arr(_, _)) {
            self.exprを左辺値として評価してアドレスをrdiレジスタへ(buf, expr);
            return;
        }

        match expr {
            Expr::NullPtr { .. } => {
                // x64 なので、edi に 0 をセットすると rdi に 0 がセットされる
                buf.append(ediに代入(0));
            }

            Expr::BinaryExpr {
                op: BinaryOp::LogicalOr,
                左辺,
                右辺,
                op_pos: _,
                typ: _,
            } => {
                // `a || b` is equivalent to `a!=0 ? 1 : b!=0`

                let mut else_buf = Buf::new();
                self.exprを評価してediレジスタへ(&mut else_buf, 右辺);
                match 右辺.typ().sizeof_primitive("if") {
                    8 => else_buf.append(rdiが0かを確認()),
                    4 => else_buf.append(ediが0かを確認()),
                    1 => else_buf.append(dilが0かを確認()),
                    _ => panic!("条件式の型のサイズがよろしくない"),
                }
                else_buf.append(フラグを読んで異なっているかどうかをalにセット());
                else_buf.append(alをゼロ拡張してediにセット());

                let mut then_buf = Buf::new();
                then_buf.append(ediに代入(1));
                then_buf.append(jmp(i8::try_from(else_buf.len())
                    .expect("|| の右辺をコンパイルした長さが i8 に収まりません")));

                let mut cond_buf = Buf::new();
                self.exprを評価してediレジスタへ(&mut cond_buf, 左辺);
                match 左辺.typ().sizeof_primitive("if") {
                    8 => cond_buf.append(rdiが0かを確認()),
                    4 => cond_buf.append(ediが0かを確認()),
                    1 => cond_buf.append(dilが0かを確認()),
                    _ => panic!("条件式の型のサイズがよろしくない"),
                }
                cond_buf.append(je(i8::try_from(then_buf.len()).expect(
                    "|| の右辺をコンパイルした長さが長すぎてジャンプを構築できません",
                )));

                buf.append(cond_buf.join(then_buf).join(else_buf));
            }

            Expr::BinaryExpr {
                op: BinaryOp::LogicalAnd,
                左辺,
                右辺,
                op_pos: _,
                typ: _,
            } => {
                // `a && b` is equivalent to `a!=0 ? b!=0 : 0`

                let else_buf = ediに代入(0);

                let mut then_buf = Buf::new();
                self.exprを評価してediレジスタへ(&mut then_buf, 右辺);
                match 右辺.typ().sizeof_primitive("if") {
                    8 => then_buf.append(rdiが0かを確認()),
                    4 => then_buf.append(ediが0かを確認()),
                    1 => then_buf.append(dilが0かを確認()),
                    _ => panic!("条件式の型のサイズがよろしくない"),
                }
                then_buf.append(フラグを読んで異なっているかどうかをalにセット());
                then_buf.append(alをゼロ拡張してediにセット());
                then_buf.append(jmp(i8::try_from(else_buf.len()).unwrap()));

                let mut cond_buf = Buf::new();
                self.exprを評価してediレジスタへ(&mut cond_buf, 左辺);
                match 左辺.typ().sizeof_primitive("if") {
                    8 => cond_buf.append(rdiが0かを確認()),
                    4 => cond_buf.append(ediが0かを確認()),
                    1 => cond_buf.append(dilが0かを確認()),
                    _ => panic!("条件式の型のサイズがよろしくない"),
                }
                cond_buf.append(je(i8::try_from(then_buf.len())
                    .expect("&& の右辺をコンパイルした長さが i8 に収まりません")));

                buf.append(cond_buf.join(then_buf).join(else_buf))
            }

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
                match typ.sizeof_primitive("a") {
                    8 => buf.append(raxが指す位置にrdiを代入()),
                    4 => buf.append(raxが指す位置にediを代入()),
                    1 => buf.append(raxが指す位置にdilを代入()),
                    _ => panic!(
                        "size が {} な型への代入はできません",
                        typ.sizeof_primitive("b")
                    ),
                };
            }

            Expr::BinaryExpr {
                op: BinaryOp::AddAssign,
                op_pos: _,
                左辺,
                右辺,
                typ,
            } => {
                self.exprを評価してediレジスタへ(buf, 右辺);
                buf.append(rdiをプッシュ());
                self.stack_size += WORD_SIZE_AS_U32;

                // スタックトップ：右辺

                self.exprを左辺値として評価してアドレスをrdiレジスタへ(
                    buf, 左辺,
                );
                buf.append(rdiをプッシュ()); // 左辺のアドレス：rdi
                self.stack_size += WORD_SIZE_AS_U32;

                buf.append(rdiを間接参照()); // 左辺の値：rdi

                buf.append(rsiへとポップ()); // 左辺のアドレス：rsi
                self.stack_size -= WORD_SIZE_AS_U32;

                buf.append(raxへとポップ()); // 右辺の値：rax
                self.stack_size -= WORD_SIZE_AS_U32;

                buf.append(rdiにraxを足し合わせる());
                buf.append(rsiをraxにコピー());

                match typ.sizeof_primitive("c") {
                    8 => buf.append(raxが指す位置にrdiを代入()),
                    4 => buf.append(raxが指す位置にediを代入()),
                    1 => buf.append(raxが指す位置にdilを代入()),
                    _ => panic!(
                        "size が {} な型への代入はできません",
                        typ.sizeof_primitive("d")
                    ),
                };
            }

            Expr::BinaryExpr {
                op: BinaryOp::SubAssign,
                op_pos: _,
                左辺,
                右辺,
                typ,
            } => {
                self.exprを評価してediレジスタへ(buf, 右辺);
                buf.append(rdiをプッシュ());
                self.stack_size += WORD_SIZE_AS_U32;

                // スタックトップ：右辺

                self.exprを左辺値として評価してアドレスをrdiレジスタへ(
                    buf, 左辺,
                );
                buf.append(rdiをプッシュ()); // 左辺のアドレス：rdi
                self.stack_size += WORD_SIZE_AS_U32;

                buf.append(rdiを間接参照()); // 左辺の値：rdi

                buf.append(rsiへとポップ()); // 左辺のアドレス：rsi
                self.stack_size -= WORD_SIZE_AS_U32;

                buf.append(raxへとポップ()); // 右辺の値：rax
                self.stack_size -= WORD_SIZE_AS_U32;

                buf.append(rdiからraxを減じる());
                buf.append(rsiをraxにコピー());

                match typ.sizeof_primitive("e") {
                    8 => buf.append(raxが指す位置にrdiを代入()),
                    4 => buf.append(raxが指す位置にediを代入()),
                    1 => buf.append(raxが指す位置にdilを代入()),
                    _ => panic!(
                        "size が {} な型への代入はできません",
                        typ.sizeof_primitive("f")
                    ),
                };
            }

            Expr::Identifier { .. } => {
                self.exprを左辺値として評価してアドレスをrdiレジスタへ(
                    buf, expr,
                );
                match expr.typ().sizeof_primitive("k") {
                    8 => buf.append(rdiを間接参照()),
                    4 => buf.append(rdiを間接参照()),
                    1 => buf.append(rdiをmovzxで間接参照()),
                    _ => panic!(
                        "size が {} な型の参照はできません",
                        expr.typ().sizeof_primitive("l")
                    ),
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
                buf.append(ediに代入(*val as u32));
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
                buf.append(rspから即値を引く(stack_size_adjustment as i32).to_vec());
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
                buf.append(rspに即値を足す(stack_size_adjustment as i32).to_vec());
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
            Expr::UnaryExpr {
                op: UnaryOp::Deref,
                typ,
                ..
            } => {
                self.exprを左辺値として評価してアドレスをrdiレジスタへ(
                    buf, expr,
                );

                match typ.sizeof_primitive("k") {
                    8 => buf.append(rdiを間接参照()),
                    4 => buf.append(rdiを間接参照()),
                    1 => buf.append(rdiをmovzxで間接参照()),
                    _ => panic!(
                        "size が {} な型の参照はできません",
                        expr.typ().sizeof_primitive("l")
                    ),
                };
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
            offsets: Vec::new(),
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

    // context.rs の実装詳細「param には 0 番から順番に ID が振られている」に依存
    for (i, (param_type, param)) in definition.params.iter().enumerate() {
        let offset = function_gen.local_var_table.allocate(
            param,
            i as u64,
            param_type.sizeof_primitive("m"),
        );
        // rbp から offset を引いた値のアドレスに、レジスタから読んできた値を入れる必要がある
        // （関数 `exprを左辺値として評価してアドレスをrdiレジスタへ` も参照）
        let negative_offset: i8 = -(offset as i8);
        match (i, param_type.sizeof_primitive("n")) {
            (0, 8) => parameter_buf.append(rbpにoffsetを足した位置にrdiを代入(
                negative_offset,
            )),
            (1, 8) => parameter_buf.append(rbpにoffsetを足した位置にrsiを代入(
                negative_offset,
            )),
            (2, 8) => parameter_buf.append(rbpにoffsetを足した位置にrdxを代入(
                negative_offset,
            )),
            (3, 8) => parameter_buf.append(rbpにoffsetを足した位置にrcxを代入(
                negative_offset,
            )),
            (4, 8) => parameter_buf.append(rbpにoffsetを足した位置にr8を代入(
                negative_offset,
            )),
            (5, 8) => parameter_buf.append(rbpにoffsetを足した位置にr9を代入(
                negative_offset,
            )),
            (0, 4) => parameter_buf.append(rbpにoffsetを足した位置にediを代入(
                negative_offset,
            )),
            (1, 4) => parameter_buf.append(rbpにoffsetを足した位置にesiを代入(
                negative_offset,
            )),
            (2, 4) => parameter_buf.append(rbpにoffsetを足した位置にedxを代入(
                negative_offset,
            )),
            (3, 4) => parameter_buf.append(rbpにoffsetを足した位置にecxを代入(
                negative_offset,
            )),
            (4, 4) => parameter_buf.append(rbpにoffsetを足した位置にr8dを代入(
                negative_offset,
            )),
            (5, 4) => parameter_buf.append(rbpにoffsetを足した位置にr9dを代入(
                negative_offset,
            )),
            (0, 1) => parameter_buf.append(rbpにoffsetを足した位置にdilを代入(
                negative_offset,
            )),
            (1, 1) => parameter_buf.append(rbpにoffsetを足した位置にsilを代入(
                negative_offset,
            )),
            (2, 1) => parameter_buf.append(rbpにoffsetを足した位置にdlを代入(
                negative_offset,
            )),
            (3, 1) => parameter_buf.append(rbpにoffsetを足した位置にclを代入(
                negative_offset,
            )),
            (4, 1) => parameter_buf.append(rbpにoffsetを足した位置にr8bを代入(
                negative_offset,
            )),
            (5, 1) => parameter_buf.append(rbpにoffsetを足した位置にr9bを代入(
                negative_offset,
            )),
            (0..=5, _) => {
                todo!(
                    "関数 `{}` の仮引数 {} の型の size {} は未対応です",
                    definition.func_name,
                    param,
                    param_type.sizeof_primitive("o")
                )
            }
            (_, _) => panic!(
                "関数 `{}` に 7 つ以上の仮引数があります",
                definition.func_name
            ),
        };
    }

    for (
        local_var_name,
        id,
        TypeAndSize {
            typ: _,
            size: local_var_size,
        },
    ) in definition.all_local_var_declarations.iter()
    {
        function_gen
            .local_var_table
            .allocate(local_var_name, *id, *local_var_size);
    }

    let content_buf = definition
        .statements
        .iter()
        .map(|stmt| function_gen.statement_or_declarationを評価(stmt))
        .fold(parameter_buf, Buf::join);

    main_buf.append(rspから即値を引く(
        i32::try_from(function_gen.local_var_table.max_offset as usize)
            .expect("ローカル変数のオフセットが i32 に収まりません"),
    ));
    main_buf.append(content_buf);

    func_pos
}
