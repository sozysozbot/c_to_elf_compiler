use core::panic;

fn main() -> std::io::Result<()> {
    let input = std::env::args().nth(1).expect("入力が与えられていません");
    let tiny = include_bytes!("../experiment/tiny");

    let file = std::fs::File::create("a.out")?;

    use std::io::Write;
    let mut writer = std::io::BufWriter::new(file);
    writer.write_all(&tiny[0..0x78])?;

    let tokens = tokenize(&mut input.chars().peekable());
    let mut tokens = tokens.iter();

    match tokens.next() {
        None => panic!("入力が空です"),
        Some(Token::Add | Token::Sub) => panic!("入力が演算子で始まっています"),
        Some(Token::Num(first)) => {
            writer.write_all(&[0xb8, 0x3c, 0x00, 0x00, 0x00])?;
            writer.write_all(&[0xbf, *first as u8, 0x00, 0x00, 0x00])?;

            while let Some(tok) = tokens.next() {
                match tok {
                    Token::Add => match tokens.next() {
                        Some(Token::Num(n)) => writer.write_all(&[0x83, 0xc7, *n])?,
                        Some(Token::Add | Token::Sub) => panic!("演算子が二つ続いています"),
                        None => panic!("入力が演算子で終わりました"),
                    },
                    Token::Sub => match tokens.next() {
                        Some(Token::Num(n)) => writer.write_all(&[0x83, 0xef, *n])?,
                        Some(Token::Add | Token::Sub) => panic!("演算子が二つ続いています"),
                        None => panic!("入力が演算子で終わりました"),
                    },
                    _ => {
                        panic!("入力が不正です");
                    }
                }
            }

            writer.write_all(&[0x0f, 0x05])?;
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Token {
    Num(u8),
    Add,
    Sub,
}

#[test]
fn tokenize_test() {
    assert_eq!(
        tokenize(&mut "5 - 3".chars().peekable()),
        vec![Token::Num(5), Token::Sub, Token::Num(3)]
    );
}

fn tokenize(iter: &mut std::iter::Peekable<impl Iterator<Item = char>>) -> Vec<Token> {
    let mut ans = vec![];
    while let Some(c) = iter.peek() {
        match c {
            ' ' => {
                iter.next();
                continue;
            }
            '+' => {
                iter.next();
                ans.push(Token::Add)
            }
            '-' => {
                iter.next();
                ans.push(Token::Sub)
            }
            '0'..='9' => ans.push(Token::Num(parse_num(iter))),
            c => {
                panic!("トークナイザ内で不正な文字 `{}` を検出", c);
            }
        }
    }
    ans
}

fn parse_num(iter: &mut std::iter::Peekable<impl Iterator<Item = char>>) -> u8 {
    let mut s = String::new();

    while let Some(&c) = iter.peek() {
        if c.is_ascii_digit() {
            s.push(c);
            iter.next();
        } else {
            break;
        }
    }

    s.parse::<u8>().expect("入力が数字ではありません")
}
