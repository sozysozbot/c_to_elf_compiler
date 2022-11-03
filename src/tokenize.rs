use crate::apperror::AppError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenPayload {
    Num(u8),
    Add,
    Sub,
    Mul,
    Div,
    開き丸括弧,
    閉じ丸括弧,
    Eof,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    pub payload: TokenPayload,
    pub pos: usize,
}

#[test]
fn tokenize_test() {
    assert_eq!(
        tokenize("5 - 3").unwrap(),
        vec![
            Token {
                payload: TokenPayload::Num(5),
                pos: 0
            },
            Token {
                payload: TokenPayload::Sub,
                pos: 2
            },
            Token {
                payload: TokenPayload::Num(3),
                pos: 4
            },
            Token {
                payload: TokenPayload::Eof,
                pos: 5
            }
        ]
    );
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, AppError> {
    let mut ans = vec![];
    let mut iter = input.chars().enumerate().peekable();
    while let Some(&(pos, c)) = iter.peek() {
        match c {
            ' ' => {
                iter.next();
                continue;
            }
            '+' => {
                iter.next();
                ans.push(Token {
                    payload: TokenPayload::Add,
                    pos,
                });
            }
            '-' => {
                iter.next();
                ans.push(Token {
                    payload: TokenPayload::Sub,
                    pos,
                });
            }
            '*' => {
                iter.next();
                ans.push(Token {
                    payload: TokenPayload::Mul,
                    pos,
                });
            }
            '/' => {
                iter.next();
                ans.push(Token {
                    payload: TokenPayload::Div,
                    pos,
                });
            }
            '(' => {
                iter.next();
                ans.push(Token {
                    payload: TokenPayload::開き丸括弧,
                    pos,
                });
            }
            ')' => {
                iter.next();
                ans.push(Token {
                    payload: TokenPayload::閉じ丸括弧,
                    pos,
                });
            }
            '0'..='9' => ans.push(Token {
                payload: TokenPayload::Num(parse_num(&mut iter).map_err(|message| AppError {
                    message,
                    input: input.to_string(),
                    pos,
                })?),
                pos,
            }),
            _ => {
                return Err(AppError {
                    message: "トークナイズできない不正な文字です".to_string(),
                    input: input.to_string(),
                    pos,
                })
            }
        }
    }
    ans.push(Token {
        payload: TokenPayload::Eof,
        pos: input.len(),
    });
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