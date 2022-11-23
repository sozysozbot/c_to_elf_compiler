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

fn main() -> std::io::Result<()> {
    let input = std::env::args().nth(1).expect("入力が与えられていません");

    let tokens = tokenize::tokenize(&input).unwrap();

    let file = std::fs::File::create("a.out")?;
    let mut writer = std::io::BufWriter::new(file);
    if let Err(e) = parse_and_codegen(&mut writer, &tokens, &input) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}

fn parse_and_codegen(
    mut writer: &mut (impl Write + Seek),
    tokens: &[Token],
    input: &str,
) -> Result<(), AppError> {
    let mut tokens = tokens.iter().peekable();
    let expr = parser::parse(&mut tokens, input)?;

    let tiny = include_bytes!("../experiment/tiny");
    writer.write_all(&tiny[0..0x78]).unwrap();

    writer.write_all(&codegen::rbpをプッシュ()).unwrap();
    writer.write_all(&codegen::rspをrbpにコピー()).unwrap();
    let lvars_count_pos = codegen::rspから即値を引く(writer);
    let mut idents = HashMap::new();
    codegen::exprを評価してediレジスタへ(&mut writer, &expr, &mut idents);

    writer.write_all(&[0xb8, 0x3c, 0x00, 0x00, 0x00]).unwrap();
    writer.write_all(&[0x0f, 0x05]).unwrap();

    writer.seek(SeekFrom::Start(lvars_count_pos)).unwrap();
    writer.write_all(&[idents.len() as u8 * 4]).unwrap();
    Ok(())
}
