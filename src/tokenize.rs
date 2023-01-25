use crate::apperror::AppError;
use crate::token::*;

#[test]
fn tokenize_test() {
    assert_eq!(
        tokenize("5 - 3").unwrap(),
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
pub fn tokenize(input: &str) -> Result<Vec<Token>, AppError> {
    let mut ans = vec![];
    let mut iter = input.chars().enumerate().peekable();
    while let Some(&(pos, c)) = iter.peek() {
        match c {
            ' ' => {
                iter.next();
                continue;
            }
            'a'..='z' | '_' => {
                iter.next();
                let mut ident = String::new();
                ident.push(c);
                while let Some(&(_, c)) = iter.peek() {
                    match c {
                        'a'..='z' | '0'..='9' | '_' => {
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
            '+' => {
                iter.next();
                ans.push(Token { tok: Tok::Add, pos });
            }
            '-' => {
                iter.next();
                ans.push(Token { tok: Tok::Sub, pos });
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
                ans.push(Token { tok: Tok::Div, pos });
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
                        return Err(AppError {
                            message: "`!`演算子はありません".to_string(),
                            input: input.to_string(),
                            pos,
                        })
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
            '0'..='9' => ans.push(Token {
                tok: Tok::Num(parse_num(&mut iter).map_err(|message| AppError {
                    message,
                    input: input.to_string(),
                    pos,
                })?),
                pos,
            }),
            ',' => {
                iter.next();
                ans.push(Token {
                    tok: Tok::Comma,
                    pos,
                });
            }
            '&' => {
                iter.next();
                ans.push(Token {
                    tok: Tok::Ampersand,
                    pos,
                });
            }
            c => {
                return Err(AppError {
                    message: format!(
                        "{c} (U+{:04X}) はトークナイズできない不正な文字です",
                        u32::from(c)
                    ),
                    input: input.to_string(),
                    pos,
                })
            }
        }
    }
    Ok(ans)
}

fn parse_num(
    iter: &mut std::iter::Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<u8, String> {
    let mut s = String::new();

    while let Some(&(_, c)) = iter.peek() {
        if c.is_ascii_digit() {
            s.push(c);
            iter.next();
        } else {
            break;
        }
    }

    s.parse::<u8>()
        .map_err(|_| "入力が符号なし8bit整数ではありません".to_string())
}
