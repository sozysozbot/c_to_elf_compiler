#![warn(clippy::pedantic)]
use c_to_elf_compiler::apperror::AppError;
use c_to_elf_compiler::codegen;
use c_to_elf_compiler::parse::toplevel;
use c_to_elf_compiler::parse::toplevel::parse_toplevel_definition;
use c_to_elf_compiler::parse::toplevel::FunctionDefinition;
use c_to_elf_compiler::parse::toplevel::FunctionSignature;
use c_to_elf_compiler::parse::toplevel::GlobalDeclarations;
use c_to_elf_compiler::parse::toplevel::SymbolDeclaration;
use c_to_elf_compiler::parse::toplevel::ToplevelDefOrDecl;
use c_to_elf_compiler::parse::typ::Type;
use c_to_elf_compiler::strlit_collector::StrLitCollector;
use c_to_elf_compiler::token::Token;
use c_to_elf_compiler::tokenize;
use c_to_elf_compiler::Buf;
use std::collections::HashMap;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let filename = std::env::args()
        .nth(1)
        .expect("ファイル名が与えられていません");
    let mut input = std::fs::read_to_string(&filename)?;
    if !input.ends_with('\n') {
        input.push('\n');
    }

    input = format!(
        "{}{input}",
        r"
int atoi(const char *s) {
    int n = 0;
    while (*s >= '0' && *s <= '9') {
        n = n * 10 + (*s - '0');
        s++;
    }
    return n;
}
    ",
    );

    let tokens = tokenize::tokenize(&input, &filename).unwrap();

    let file = std::fs::File::create("a.out")?;
    let mut writer = std::io::BufWriter::new(file);
    match parse_and_codegen(&tokens, &input, &filename) {
        Ok(buf) => {
            writer.write_all(&buf)?;
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn parse_and_codegen(tokens: &[Token], input: &str, filename: &str) -> Result<Vec<u8>, AppError> {
    let mut tokens = tokens.iter().peekable();
    let signatures_of_builtin_functions: HashMap<String, FunctionSignature> = [
        (
            "__builtin_three".to_string(),
            FunctionSignature {
                params: Some(Vec::new()),
                pos: 0,
                return_type: Type::Int,
            },
        ),
        (
            "__builtin_putchar".to_string(),
            FunctionSignature {
                params: Some(vec![Type::Int]),
                pos: 0,
                return_type: Type::Int,
            },
        ),
        (
            "__builtin_alloc4".to_string(),
            FunctionSignature {
                params: Some(vec![Type::Int, Type::Int, Type::Int, Type::Int]),
                pos: 0,
                return_type: Type::Ptr(Box::new(Type::Int)),
            },
        ),
    ]
    .into_iter()
    .collect();

    let mut global_declarations = GlobalDeclarations {
        symbols: HashMap::new(),
        struct_names: HashMap::new(),
    };
    global_declarations.symbols.extend(
        signatures_of_builtin_functions
            .into_iter()
            .map(|(name, signature)| (name, SymbolDeclaration::Func(signature))),
    );

    let mut strlit_collector: StrLitCollector = StrLitCollector::new();

    let function_definitions = toplevel::parse_all(
        &mut strlit_collector,
        &mut global_declarations,
        &mut tokens,
        filename,
        input,
    )?;

    // strlit_collector is fully populated here

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

    let builtin_alloc4_pos = u32::try_from(buf.len()).expect("バッファの長さが u32 に収まりません");
    buf.append(codegen::builtin_alloc4関数を生成());
    global_function_table.insert("__builtin_alloc4".to_string(), builtin_alloc4_pos);

    let strlit_constant_pool = strlit_collector.to_pool();
    for (id, string) in strlit_constant_pool.iter().enumerate() {
        let pos = u32::try_from(buf.len()).expect("バッファの長さが u32 に収まりません");
        buf.append(codegen::builtin_strlit_n関数を生成(string.as_bytes()));
        global_function_table.insert(format!("__builtin_strlit_{id}"), pos);
    }

    for definition in function_definitions {
        codegen::関数をコード生成しメインバッファとグローバル関数テーブルに挿入(
            &mut global_function_table,
            &mut buf,
            &definition,
        );
    }

    let entry: FunctionDefinition = {
        // スタートアップ処理はここに C のソースコードとして実装
        let tokens = tokenize::tokenize(
            "int __start() { __builtin_populate_argc_argv; __throw main(); }",
            filename,
        )
        .unwrap();
        let mut tokens = tokens.iter().peekable();
        let previous_symbol_declarations: HashMap<String, SymbolDeclaration> = [(
            "main".to_string(),
            SymbolDeclaration::Func(FunctionSignature {
                params: Some(vec![]), // todo: possible argc and argv
                pos: 0,
                return_type: Type::Int,
            }),
        )]
        .into_iter()
        .collect();
        if let ToplevelDefOrDecl::FuncDef(entry) = parse_toplevel_definition(
            &mut StrLitCollector::new(), // do not collect any string literals from the startup code
            &GlobalDeclarations {
                symbols: previous_symbol_declarations,
                struct_names: HashMap::new(),
            },
            &mut tokens,
            filename,
            input,
        )? {
            entry
        } else {
            panic!("スタートアップ処理が関数定義の形で書かれていません")
        }
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
