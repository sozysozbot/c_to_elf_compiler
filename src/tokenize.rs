use crate::apperror::AppError;
use crate::token::*;

#[test]
fn tokenize_test() {
    assert_eq!(
        tokenize("5 - 3", "test.c").unwrap(),
        vec![
            Token {
                tok: Tok::Num(5),
                pos: 0
            },
            Token {
                tok: Tok::Sub,
                pos: 2
            },
            Token {
                tok: Tok::Num(3),
                pos: 4
            }
        ]
    );
}

#[allow(clippy::too_many_lines)]
pub fn tokenize(input: &str, filename: &str) -> Result<Vec<Token>, AppError> {
    let mut ans = vec![];
    let mut iter: std::iter::Peekable<std::iter::Enumerate<std::str::Chars<'_>>> =
        input.chars().enumerate().peekable();
    while let Some(&(pos, c)) = iter.peek() {
        match c {
            '"' => {
                iter.next();
                let mut string_content = String::new();
                loop {
                    let (_, c) = iter
                        .next()
                        .unwrap_or_else(|| panic!("文字列リテラルが終了する前にEOFが来ました"));
                    if c == '"' {
                        break;
                    } else if c == '\\' {
                        panic!("エスケープシーケンスは未対応です");
                    } else {
                        string_content.push(c);
                    }
                }
                ans.push(Token {
                    tok: Tok::StringLiteral(string_content),
                    pos,
                });
            }
            ' ' | '\n' | '\r' | '\t' => {
                iter.next();
                continue;
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                iter.next();
                let mut ident = String::new();
                ident.push(c);
                while let Some(&(_, c)) = iter.peek() {
                    match c {
                        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                            iter.next();
                            ident.push(c);
                        }
                        _ => break,
                    }
                }

                let payload = match ident.as_str() {
                    "__throw" => Tok::Throw,
                    "return" => Tok::Return,
                    "if" => Tok::If,
                    "else" => Tok::Else,
                    "while" => Tok::While,
                    "for" => Tok::For,
                    "int" => Tok::Int,
                    "char" => Tok::Char,
                    "sizeof" => Tok::Sizeof,
                    "_Alignof" => Tok::Alignof,
                    "struct" => Tok::Struct,
                    "void" => Tok::Void,
                    _ => Tok::Identifier(ident),
                };

                ans.push(Token { tok: payload, pos });
            }
            ';' => {
                iter.next();
                ans.push(Token {
                    tok: Tok::Semicolon,
                    pos,
                });
            }
            '{' => {
                iter.next();
                ans.push(Token {
                    tok: Tok::開き波括弧,
                    pos,
                });
            }
            '}' => {
                iter.next();
                ans.push(Token {
                    tok: Tok::閉じ波括弧,
                    pos,
                });
            }
            '[' => {
                iter.next();
                ans.push(Token {
                    tok: Tok::開き角括弧,
                    pos,
                });
            }
            ']' => {
                iter.next();
                ans.push(Token {
                    tok: Tok::閉じ角括弧,
                    pos,
                });
            }
            '.' => {
                iter.next();
                ans.push(Token { tok: Tok::Dot, pos });
            }
            '+' => {
                iter.next();
                match iter.peek() {
                    Some(&(pos, '+')) => {
                        iter.next();
                        ans.push(Token {
                            tok: Tok::Increment,
                            pos,
                        });
                    }
                    Some(&(pos, '=')) => {
                        iter.next();
                        ans.push(Token {
                            tok: Tok::AddAssign,
                            pos,
                        });
                    }
                    _ => {
                        ans.push(Token { tok: Tok::Add, pos });
                    }
                }
            }
            '-' => {
                iter.next();
                match iter.peek() {
                    Some(&(pos, '-')) => {
                        iter.next();
                        ans.push(Token {
                            tok: Tok::Decrement,
                            pos,
                        });
                    }
                    Some(&(pos, '>')) => {
                        iter.next();
                        ans.push(Token {
                            tok: Tok::Arrow,
                            pos,
                        });
                    }
                    Some(&(pos, '=')) => {
                        iter.next();
                        ans.push(Token {
                            tok: Tok::SubAssign,
                            pos,
                        });
                    }
                    _ => {
                        ans.push(Token { tok: Tok::Sub, pos });
                    }
                }
            }
            '*' => {
                iter.next();
                ans.push(Token {
                    tok: Tok::Asterisk,
                    pos,
                });
            }
            '/' => {
                iter.next();
                match iter.peek() {
                    Some((_, '*')) => {
                        iter.next();
                        loop {
                            match iter.peek() {
                                Some((_, '*')) => {
                                    iter.next();
                                    if let Some((_, '/')) = iter.peek() {
                                        iter.next();
                                        break;
                                    }
                                }
                                Some(_) => {
                                    iter.next();
                                }
                                None => {
                                    return Err(AppError {
                                        message: "コメントが終了する前にEOFが来ました".to_string(),
                                        input: input.to_string(),
                                        filename: filename.to_string(),
                                        pos,
                                    })
                                }
                            }
                        }
                    }
                    Some((_, '/')) => {
                        iter.next();
                        while let Some(&(_, c)) = iter.peek() {
                            match c {
                                '\n' => break,
                                _ => {
                                    iter.next();
                                }
                            }
                        }
                    }
                    _ => {
                        ans.push(Token { tok: Tok::Div, pos });
                    }
                }
            }
            '(' => {
                iter.next();
                ans.push(Token {
                    tok: Tok::開き丸括弧,
                    pos,
                });
            }
            ')' => {
                iter.next();
                ans.push(Token {
                    tok: Tok::閉じ丸括弧,
                    pos,
                });
            }
            '=' => {
                iter.next();
                match iter.peek() {
                    Some(&(pos, '=')) => {
                        iter.next();
                        ans.push(Token {
                            tok: Tok::Equal,
                            pos,
                        });
                    }
                    _ => {
                        ans.push(Token {
                            tok: Tok::Assign,
                            pos,
                        });
                    }
                }
            }
            '!' => {
                iter.next();
                match iter.peek() {
                    Some(&(pos, '=')) => {
                        iter.next();
                        ans.push(Token {
                            tok: Tok::NotEqual,
                            pos,
                        });
                    }
                    _ => {
                        ans.push(Token {
                            tok: Tok::LogicalNot,
                            pos,
                        });
                    }
                }
            }
            '<' => {
                iter.next();
                match iter.peek() {
                    Some(&(pos, '=')) => {
                        iter.next();
                        ans.push(Token {
                            tok: Tok::LessThanOrEqual,
                            pos,
                        });
                    }
                    _ => ans.push(Token {
                        tok: Tok::LessThan,
                        pos,
                    }),
                }
            }
            '>' => {
                iter.next();
                match iter.peek() {
                    Some(&(pos, '=')) => {
                        iter.next();
                        ans.push(Token {
                            tok: Tok::GreaterThanOrEqual,
                            pos,
                        });
                    }
                    _ => ans.push(Token {
                        tok: Tok::GreaterThan,
                        pos,
                    }),
                }
            }
            '\'' => {
                iter.next();
                match iter.next() {
                    Some((_, '\\')) => {
                        // escape sequence
                        match iter.next() {
                            Some((_, 'n')) => ans.push(Token {
                                tok: Tok::Num(10),
                                pos,
                            }),
                            Some((_, '\'')) => ans.push(Token {
                                tok: Tok::Num(39),
                                pos,
                            }),
                            Some((_, '\\')) => ans.push(Token {
                                tok: Tok::Num(92),
                                pos,
                            }),
                            Some((_, c)) => {
                                return Err(AppError {
                                    message: format!(
                                        "文字リテラルのエスケープシーケンス '\\{c}' は未対応です",
                                    ),
                                    input: input.to_string(),
                                    filename: filename.to_string(),
                                    pos,
                                });
                            }
                            None => {
                                return Err(AppError {
                                    message: "文字リテラルが終了する前にEOFが来ました".to_string(),
                                    input: input.to_string(),
                                    filename: filename.to_string(),
                                    pos,
                                });
                            }
                        }

                        expect_end_of_char_lit(input, filename, pos, &mut iter)?;
                    }
                    Some((_, c)) => {
                        let charcode = u32::from(c);
                        if charcode > 255 {
                            return Err(AppError {
                                message: format!(
                                    "文字リテラルの値が符号なし8ビット整数に収まりません：'{c}'"
                                ),
                                input: input.to_string(),
                                filename: filename.to_string(),
                                pos,
                            });
                        }

                        ans.push(Token {
                            tok: Tok::Num(charcode as i32),
                            pos,
                        });
                        expect_end_of_char_lit(input, filename, pos, &mut iter)?;
                    }
                    None => {
                        return Err(AppError {
                            message: "文字リテラルが終了する前にEOFが来ました".to_string(),
                            input: input.to_string(),
                            filename: filename.to_string(),
                            pos,
                        });
                    }
                }
            }
            '0'..='9' => {
                let num = parse_num(&mut iter);

                let new_tok = Token {
                    tok: Tok::Num(num.map_err(|message| AppError {
                        message,
                        input: input.to_string(),
                        filename: filename.to_string(),
                        pos,
                    })?),
                    pos,
                };
                ans.push(new_tok);
            }
            ',' => {
                iter.next();
                ans.push(Token {
                    tok: Tok::Comma,
                    pos,
                });
            }
            '&' => {
                iter.next();
                match iter.peek() {
                    Some(&(pos, '&')) => {
                        iter.next();
                        ans.push(Token {
                            tok: Tok::LogicalAnd,
                            pos,
                        });
                    }
                    _ => ans.push(Token {
                        tok: Tok::Ampersand,
                        pos,
                    }),
                }
            }
            '|' => {
                iter.next();
                match iter.peek() {
                    Some(&(pos, '|')) => {
                        iter.next();
                        ans.push(Token {
                            tok: Tok::LogicalOr,
                            pos,
                        });
                    }
                    _ => {
                        return Err(AppError {
                            message: "bit OR is not yet implemented".to_string(),
                            input: input.to_string(),
                            filename: filename.to_string(),
                            pos,
                        })
                    }
                }
            }
            c => {
                return Err(AppError {
                    message: format!(
                        "{c} (U+{:04X}) はトークナイズできない不正な文字です",
                        u32::from(c)
                    ),
                    input: input.to_string(),
                    filename: filename.to_string(),
                    pos,
                })
            }
        }
    }
    Ok(ans)
}

fn expect_end_of_char_lit(
    input: &str,
    filename: &str,
    pos: usize,
    iter: &mut std::iter::Peekable<std::iter::Enumerate<std::str::Chars<'_>>>,
) -> Result<(), AppError> {
    match iter.next() {
        Some((_, '\'')) => Ok(()),
        None => Err(AppError {
            message: "文字リテラルが終了する前にEOFが来ました".to_string(),
            input: input.to_string(),
            filename: filename.to_string(),
            pos,
        }),
        Some((_, c)) => Err(AppError {
            message: format!(
                "文字リテラルの終端が不正です。予期される終端は ' ですが、実際には '{c}' でした"
            ),
            input: input.to_string(),
            filename: filename.to_string(),
            pos,
        }),
    }
}

fn parse_num(
    iter: &mut std::iter::Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<i32, String> {
    let mut s = String::new();

    while let Some(&(_, c)) = iter.peek() {
        if c.is_ascii_digit() {
            s.push(c);
            iter.next();
        } else {
            break;
        }
    }

    s.parse::<i32>()
        .map_err(|_| "入力が i32 に収まりません".to_string())
}
