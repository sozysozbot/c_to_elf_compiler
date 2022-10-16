fn main() -> std::io::Result<()> {
    let input = std::env::args().nth(1).expect("入力が与えられていません");
    let tiny = include_bytes!("../experiment/tiny");

    let file = std::fs::File::create("a.out")?;

    use std::io::Write;
    let mut writer = std::io::BufWriter::new(file);
    writer.write_all(&tiny[0..0x78])?;

    let mut input = input.chars().peekable();
    let first = parse_num(&mut input);

    writer.write_all(&[0xb8, 0x3c, 0x00, 0x00, 0x00])?;
    writer.write_all(&[0xbf, first as u8, 0x00, 0x00, 0x00])?;

    while let Some(c) = input.next() {
        match c {
            '+' => {
                let n = parse_num(&mut input);
                writer.write_all(&[0x83, 0xc7, n])?;
            }
            '-' => {
                let n = parse_num(&mut input);
                writer.write_all(&[0x83, 0xef, n])?;
            }
            _ => {
                panic!("入力が不正です");
            }
        }
    }

    writer.write_all(&[0x0f, 0x05])?;
    Ok(())
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
