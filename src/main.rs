#![warn(clippy::pedantic)]
use std::io::Write;

use c_to_elf_compiler::apperror::AppError;
use c_to_elf_compiler::codegen;
use c_to_elf_compiler::parser;
use c_to_elf_compiler::parser::FunctionDefinition;
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
    let function_definitions = parser::parse(&mut tokens, input)?;

    let tiny = include_bytes!("../experiment/tiny");
    let buf = Buf::from(&tiny[0..0x78]);

    let mut codegen = codegen::Codegen::new();

    let builtin_three_pos = u32::try_from(buf.len()).expect("バッファの長さが u32 に収まりません");
    let buf = buf.join(codegen::builtin_three関数を生成());
    codegen
        .functions
        .insert("__builtin_three".to_string(), builtin_three_pos);

    let builtin_putchar_pos =
        u32::try_from(buf.len()).expect("バッファの長さが u32 に収まりません");
    let buf = buf.join(codegen::builtin_putchar関数を生成());
    codegen
        .functions
        .insert("__builtin_putchar".to_string(), builtin_putchar_pos);

    let mut buf = buf;

    for definition in function_definitions {
        codegen.関数をコード生成しメインバッファに挿入(&mut buf, &definition);
    }

    let entry: FunctionDefinition = {
        // スタートアップ処理はここに C のソースコードとして実装
        let tokens = tokenize::tokenize("__start() { __throw main(); }").unwrap();
        let mut tokens = tokens.iter().peekable();
        parser::parse_toplevel_function_definition(&mut tokens, input)?
    };
    let entry_pos =
        codegen.関数をコード生成しメインバッファに挿入(&mut buf, &entry);

    let mut buf = buf.to_vec();
    // エントリポイント書き換え
    let entry_pos_buf = entry_pos.to_le_bytes();
    buf[0x18] = entry_pos_buf[0];
    buf[0x19] = entry_pos_buf[1];

    Ok(buf)
}
