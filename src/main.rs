#![warn(clippy::pedantic)]
use std::collections::HashMap;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

use c_to_elf_compiler::apperror::AppError;
use c_to_elf_compiler::codegen;
use c_to_elf_compiler::parser;
use c_to_elf_compiler::token::Token;
use c_to_elf_compiler::tokenize;
use c_to_elf_compiler::Buf;

fn main() -> std::io::Result<()> {
    let input = std::env::args().nth(1).expect("入力が与えられていません");

    let tokens = tokenize::tokenize(&input).unwrap();

    let file = std::fs::File::create("a.out")?;
    let mut writer = std::io::BufWriter::new(file);
    match parse_and_codegen(&tokens, &input) {
        Ok(buf) => {
            writer.write_all(&buf.to_vec())?;
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn parse_and_codegen(tokens: &[Token], input: &str) -> Result<Buf, AppError> {
    let mut tokens = tokens.iter().peekable();
    let program = parser::parse(&mut tokens, input)?;

    let tiny = include_bytes!("../experiment/tiny");
    let buf = Buf::from(&tiny[0..0x78]);

    let buf = buf.join(Buf::from(codegen::rbpをプッシュ()));
    let buf = buf.join(Buf::from(codegen::rspをrbpにコピー()));
    let mut idents = HashMap::new();
    let program_buf = codegen::programを評価(&program, &mut idents);

    let buf = buf.join(codegen::rspから即値を引く(idents.len() as u8 * 4));
    let buf = buf.join(program_buf);

    Ok(buf)
}
