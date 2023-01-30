#![warn(clippy::pedantic)]
use std::collections::HashMap;
use std::io::Write;

use c_to_elf_compiler::apperror::AppError;
use c_to_elf_compiler::codegen;
use c_to_elf_compiler::parse::toplevel;
use c_to_elf_compiler::parse::toplevel::FunctionDefinition;
use c_to_elf_compiler::parse::toplevel::FunctionSignature;
use c_to_elf_compiler::parse::toplevel::Type;
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
    let mut function_declarations: HashMap<String, FunctionSignature> = [
        (
            "__builtin_three".to_string(),
            FunctionSignature {
                params: Vec::new(),
                pos: 0,
                return_type: Type::Int,
            },
        ),
        (
            "__builtin_putchar".to_string(),
            FunctionSignature {
                params: vec![Type::Int],
                pos: 0,
                return_type: Type::Int,
            },
        ),
    ]
    .into_iter()
    .collect();

    let function_definitions = toplevel::parse(&mut function_declarations, &mut tokens, input)?;

    let tiny = include_bytes!("../experiment/tiny");
    let mut buf = Buf::from(&tiny[0..0x78]);

    let mut global_function_table: HashMap<String, u32> = HashMap::new();

    let builtin_three_pos = u32::try_from(buf.len()).expect("バッファの長さが u32 に収まりません");
    buf.append(codegen::builtin_three関数を生成());
    global_function_table.insert("__builtin_three".to_string(), builtin_three_pos);

    let builtin_putchar_pos =
        u32::try_from(buf.len()).expect("バッファの長さが u32 に収まりません");
    buf.append(codegen::builtin_putchar関数を生成());
    global_function_table.insert("__builtin_putchar".to_string(), builtin_putchar_pos);

    let mut buf = buf;

    for definition in function_definitions {
        codegen::関数をコード生成しメインバッファとグローバル関数テーブルに挿入(
            &mut global_function_table,
            &mut buf,
            &definition,
        );
    }

    let entry: FunctionDefinition = {
        // スタートアップ処理はここに C のソースコードとして実装
        let tokens = tokenize::tokenize("int __start() { __throw main(); }").unwrap();
        let mut tokens = tokens.iter().peekable();
        toplevel::parse_toplevel_function_definition(
            &[(
                "main".to_string(),
                FunctionSignature {
                    params: vec![],
                    pos: 0,
                    return_type: Type::Int,
                },
            )]
            .into_iter()
            .collect(),
            &mut tokens,
            input,
        )?
    };
    let entry_pos = codegen::関数をコード生成しメインバッファとグローバル関数テーブルに挿入(
        &mut global_function_table,
        &mut buf,
        &entry,
    );

    let mut buf = buf.to_vec();
    // エントリポイント書き換え
    let entry_pos_buf = entry_pos.to_le_bytes();
    buf[0x18] = entry_pos_buf[0];
    buf[0x19] = entry_pos_buf[1];

    Ok(buf)
}
