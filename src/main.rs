#![warn(clippy::pedantic)]
use std::collections::HashMap;
use std::io::Write;

use c_to_elf_compiler::apperror::AppError;
use c_to_elf_compiler::ast::FunctionContent;
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
            writer.write_all(&buf)?;
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn parse_and_codegen(tokens: &[Token], input: &str) -> Result<Vec<u8>, AppError> {
    let mut tokens = tokens.iter().peekable();
    let content_main = parser::parse(&mut tokens, input)?;

    let tiny = include_bytes!("../experiment/tiny");
    let buf = Buf::from(&tiny[0..0x78]);

    let mut functions = HashMap::new();

    let builtin_three_pos = u32::try_from(buf.len()).expect("バッファの長さが u32 に収まりません");
    let buf = buf.join(codegen::builtin_three関数を生成());
    functions.insert("__builtin_three".to_string(), builtin_three_pos);

    let builtin_putchar_pos =
        u32::try_from(buf.len()).expect("バッファの長さが u32 に収まりません");
    let buf = buf.join(codegen::builtin_putchar関数を生成());
    functions.insert("__builtin_putchar".to_string(), builtin_putchar_pos);

    let (buf, main_pos) = generate_function(buf, &content_main, &functions);
    functions.insert("main".to_string(), u32::from(main_pos));

    let content_entry = {
        // スタートアップ処理はここに C のソースコードとして実装
        let tokens = tokenize::tokenize("__throw main();").unwrap();
        let mut tokens = tokens.iter().peekable();
        parser::parse(&mut tokens, input)?
    };
    let (buf, entry_pos) = generate_function(buf, &content_entry, &functions);

    let mut buf = buf.to_vec();
    // エントリポイント書き換え
    let entry_pos_buf = entry_pos.to_le_bytes();
    buf[0x18] = entry_pos_buf[0];
    buf[0x19] = entry_pos_buf[1];

    Ok(buf)
}

fn generate_function(buf: Buf, content: &FunctionContent, functions: &HashMap<String, u32>) -> (Buf, u16) {
    let function_pos = u16::try_from(buf.len()).expect("バッファの長さが u16 に収まりません");
    let buf = buf.join(codegen::rbpをプッシュ());
    let buf = buf.join(codegen::rspをrbpにコピー());
    let mut idents = HashMap::new();
    let mut stack_size = 8;
    let content_buf = codegen::関数の中身を評価(content, &mut idents, functions, &mut stack_size);

    let buf = buf.join(codegen::rspから即値を引く(
        u8::try_from(idents.len()).expect("識別子の個数が u8 に収まりません") * 4,
    ));
    let buf = buf.join(content_buf);

    (buf, function_pos)
}
