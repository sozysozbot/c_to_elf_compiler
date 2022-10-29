use std::fmt;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let input = std::env::args().nth(1).expect("入力が与えられていません");

    let tokens = tokenize(&input).unwrap();

    let file = std::fs::File::create("a.out")?;
    let mut writer = std::io::BufWriter::new(file);
    if let Err(e) = parse_and_codegen(&mut writer, &tokens, &input) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}

fn parse_and_codegen(
    writer: &mut impl Write,
    tokens: &Vec<Token>,
    input: &str,
) -> Result<(), AppError> {
    let mut tokens = tokens.iter();

    let tiny = include_bytes!("../experiment/tiny");
    writer.write_all(&tiny[0..0x78]).unwrap();

    match tokens.next().unwrap() {
        Token {
            payload: TokenPayload::Num(first),
            ..
        } => {
            writer.write_all(&[0xb8, 0x3c, 0x00, 0x00, 0x00]).unwrap();
            writer
                .write_all(&[0xbf, *first as u8, 0x00, 0x00, 0x00])
                .unwrap();

            loop {
                let tok = tokens.next().unwrap();
                match tok.payload {
                    TokenPayload::Add => match tokens.next().unwrap() {
                        Token {
                            payload: TokenPayload::Num(n),
                            ..
                        } => writer.write_all(&[0x83, 0xc7, *n]).unwrap(),
                        tok => {
                            return Err(AppError {
                                message: "数値ではありません".to_string(),
                                input: input.to_string(),
                                pos: tok.pos,
                            });
                        }
                    },
                    TokenPayload::Sub => match tokens.next() {
                        Some(Token {
                            payload: TokenPayload::Num(n),
                            ..
                        }) => writer.write_all(&[0x83, 0xef, *n]).unwrap(),
                        Some(tok) => {
                            return Err(AppError {
                                message: "数値ではありません".to_string(),
                                input: input.to_string(),
                                pos: tok.pos,
                            });
                        }
                        None => panic!("入力が演算子で終わりました"),
                    },
                    TokenPayload::Eof => {
                        writer.write_all(&[0x0f, 0x05]).unwrap();
                        return Ok(());
                    }
                    _ => {
                        return Err(AppError {
                            message: "演算子かeofが期待されています".to_string(),
                            input: input.to_string(),
                            pos: tok.pos,
                        });
                    }
                }
            }
        }
        tok => {
            return Err(AppError {
                message: "入力が数字以外で始まっています".to_string(),
                input: input.to_string(),
                pos: tok.pos,
            });
        }
    }
}

#[derive(Debug, Clone)]
struct AppError {
    message: String,
    input: String,
    pos: usize,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", self.input)?;
        write!(f, "{}^ {}", " ".repeat(self.pos), self.message)?;
        Ok(())
    }
}

impl std::error::Error for AppError {}

#[derive(Debug, Clone, Eq, PartialEq)]
enum TokenPayload {
    Num(u8),
    Add,
    Sub,
    Eof,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Token {
    payload: TokenPayload,
    pos: usize,
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

fn tokenize(input: &str) -> Result<Vec<Token>, AppError> {
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
                })
            }
            '-' => {
                iter.next();
                ans.push(Token {
                    payload: TokenPayload::Sub,
                    pos,
                })
            }
            '0'..='9' => ans.push(Token {
                payload: TokenPayload::Num(parse_num(&mut iter).map_err(|message| AppError {
                    message,
                    input: input.to_string(),
                    pos,
                })?),
                pos,
            }),
            c => {
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
