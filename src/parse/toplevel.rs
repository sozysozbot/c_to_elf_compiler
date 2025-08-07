use super::combinator::satisfy;
use super::statement::parse_statement_or_declaration;
use super::statement::parse_type_and_identifier;
use super::statement::parse_角括弧に包まれた数の列;
use super::typ::parse_type;
use super::typ::Type;
use crate::apperror::*;
use crate::ast::*;
use crate::token::*;
use std::collections::HashMap;
use std::{iter::Peekable, slice::Iter};

#[derive(Debug, Clone)]
pub enum ToplevelDefinition {
    Func(FunctionDefinition),
    GVar(GlobalVariableDefinition),
}

#[derive(Debug, Clone)]
pub struct GlobalVariableDefinition {
    pub name: String,
    pub typ: Type,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructDefinition {
    pub struct_name: String,
    pub size: u8,
    pub align: u8,
    pub members: HashMap<String, StructMember>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructMember {
    pub member_type: Type,
    pub offset: u8,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub func_name: String,
    pub params: Vec<(Type, String)>,
    pub pos: usize,
    pub statements: Vec<StatementOrDeclaration>,
    pub return_type: Type,
    pub local_var_declarations: HashMap<String, TypeAndSize>,
}

impl From<FunctionDefinition> for (String, FunctionSignature) {
    fn from(s: FunctionDefinition) -> (String, FunctionSignature) {
        (
            s.func_name,
            FunctionSignature {
                pos: s.pos,
                return_type: s.return_type,
                params: s.params.into_iter().map(|(typ, _)| typ).collect(),
            },
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionSignature {
    pub params: Vec<Type>,
    pub pos: usize,
    pub return_type: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolDeclaration {
    Func(FunctionSignature),
    GVar(Type),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalDeclarations {
    pub symbols: HashMap<String, SymbolDeclaration>,
    pub struct_names: HashMap<String, StructDefinition>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeAndSize {
    pub typ: Type,
    pub size: u8,
}

pub struct Context {
    local_var_and_param_declarations: HashMap<String, TypeAndSize>,
    pub global_declarations: GlobalDeclarations,
}

impl Context {
    pub fn new(
        local_var_and_param_declarations: HashMap<String, TypeAndSize>,
        global_declarations: GlobalDeclarations,
    ) -> Self {
        Self {
            local_var_and_param_declarations,
            global_declarations,
        }
    }

    pub fn insert_local_var(&mut self, ident: String, typ_and_size: TypeAndSize) {
        // when there is conflict in the same scope, we throw an error
        if self.local_var_and_param_declarations.contains_key(&ident) {
            panic!("ローカル変数の再定義: {ident}");
        }

        self.local_var_and_param_declarations
            .insert(ident, typ_and_size);
    }

    pub fn resolve_type_and_size_as_var(&self, ident: &str) -> Result<TypeAndSize, String> {
        if let Some(typ_and_size) = self.local_var_and_param_declarations.get(ident) {
            Ok(typ_and_size.clone())
        } else {
            match self.global_declarations.symbols.get(ident) {
                Some(SymbolDeclaration::GVar(t)) => Ok(TypeAndSize {
                    typ: t.clone(),
                    size: t.sizeof(&self.global_declarations.struct_names),
                }),
                Some(SymbolDeclaration::Func(_u)) => Err(format!(
                    "識別子 {ident} は関数であり、現在関数ポインタは実装されていません",
                )),
                None => Err(format!(
                    "識別子 {ident} は定義されておらず、型が分かりません",
                )),
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn after_param_list(
    previous_global_declarations: &GlobalDeclarations,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
    params: Vec<(Type, String)>,
    pos: usize,
    return_type: Type,
    func_name: &str,
) -> Result<FunctionDefinition, AppError> {
    match tokens.peek().unwrap() {
        Token {
            tok: Tok::開き波括弧,
            ..
        } => {
            tokens.next();

            let mut statements_or_declarations: Vec<StatementOrDeclaration> = vec![];

            let mut local_var_and_param_declarations: HashMap<String, TypeAndSize> = HashMap::new();
            local_var_and_param_declarations.extend(params.iter().map(|(typ, ident)| {
                (
                    ident.clone(),
                    TypeAndSize {
                        typ: (*typ).clone(),
                        size: typ.sizeof(&previous_global_declarations.struct_names),
                    },
                )
            }));

            let mut global_declarations = (*previous_global_declarations).clone();

            let signature = FunctionSignature {
                params: params.iter().map(|(typ, _)| (*typ).clone()).collect(),
                pos,
                return_type: return_type.clone(),
            };
            // 今読んでる関数の定義も足さないと再帰呼び出しができない
            global_declarations.symbols.insert(
                func_name.to_string(),
                SymbolDeclaration::Func(signature.clone()),
            );

            let mut context = Context::new(local_var_and_param_declarations, global_declarations);

            loop {
                match tokens.peek() {
                    None => {
                        return Err(AppError {
                            message: "期待された閉じ波括弧が来ませんでした".to_string(),
                            input: input.to_string(),
                            filename: filename.to_string(),
                            pos: input.len(),
                        })
                    }
                    Some(Token {
                        tok: Tok::閉じ波括弧,
                        ..
                    }) => {
                        tokens.next();
                        break;
                    }
                    _ => {
                        let parsed = parse_statement_or_declaration(
                            &mut context,
                            tokens,
                            filename,
                            input,
                        )?;
                        statements_or_declarations.push(parsed);
                    }
                }
            }

            // collect the local variable declarations (for now, we only need to handle those at the immediate function scope)
            let mut local_var_declarations: HashMap<String, TypeAndSize> = HashMap::new();
            for s_or_d in statements_or_declarations.iter() {
                match s_or_d {
                    StatementOrDeclaration::Statement(_) => {}
                    // If it's a declaration, we insert it into the local variable declarations
                    StatementOrDeclaration::Declaration { name, typ_and_size } => {
                        local_var_declarations.insert(name.clone(), typ_and_size.clone());
                    }
                    StatementOrDeclaration::DeclarationWithInitializer {
                        name,
                        typ_and_size,
                        initializer: _,
                    } => {
                        local_var_declarations.insert(name.clone(), typ_and_size.clone());
                    }
                }
            }

            Ok(FunctionDefinition {
                func_name: func_name.to_string(),
                params,
                pos,
                statements: statements_or_declarations,
                return_type,
                local_var_declarations,
            })
        }
        Token { pos, .. } => Err(AppError {
            message: "仮引数リストの後に、開き波括弧以外のトークンが来ました".to_string(),
            input: input.to_string(),
            filename: filename.to_string(),
            pos: *pos,
        }),
    }
}

pub fn parse_toplevel_definition(
    previous_declarations: &GlobalDeclarations,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<ToplevelDefinition, AppError> {
    let mut return_type = parse_type(tokens, filename, input)?;
    match tokens.next().unwrap() {
        Token {
            tok: Tok::Identifier(ident),
            pos,
        } => match tokens.peek().unwrap() {
            Token {
                tok: Tok::開き丸括弧,
                pos: open_pos,
            } => {
                tokens.next();

                let mut params = Vec::new();

                match tokens.peek().unwrap() {
                    Token {
                        tok: Tok::閉じ丸括弧,
                        ..
                    } => {
                        tokens.next();
                        return Ok(ToplevelDefinition::Func(after_param_list(
                            previous_declarations,
                            tokens,
                            filename,
                            input,
                            params,
                            *pos,
                            return_type,
                            ident,
                        )?));
                    }
                    _ => {
                        let param = parse_type_and_identifier(tokens,filename, input)?;
                        params.push(param);
                    }
                }

                loop {
                    match tokens.peek().unwrap() {
                        Token {
                            tok: Tok::閉じ丸括弧,
                            ..
                        } => {
                            tokens.next();
                            return Ok(ToplevelDefinition::Func(after_param_list(
                                previous_declarations,
                                tokens,
                                filename,
                                input,
                                params,
                                *pos,
                                return_type,
                                ident,
                            )?));
                        }
                        Token {
                            tok: Tok::Comma, ..
                        } => {
                            tokens.next();
                            let param = parse_type_and_identifier(tokens, filename, input)?;
                            params.push(param);
                        }
                        _ => {
                            break Err(AppError {
                                message: "閉じ丸括弧かカンマが期待されていました".to_string(),
                                input: input.to_string(),
                                filename: filename.to_string(),
                                pos: *open_pos + 1,
                            })
                        }
                    }
                }
            }
            Token {
                tok: Tok::Semicolon,
                ..
            } => {
                tokens.next();
                Ok(ToplevelDefinition::GVar(GlobalVariableDefinition { name: ident.to_string(), typ: return_type }))
            }
            Token {
                tok: Tok::開き角括弧,
                ..
            } => {
                parse_角括弧に包まれた数の列(tokens, filename,input, &mut return_type)?;
                satisfy(tokens,filename, input, |t| *t == Tok::Semicolon, "グローバルな配列宣言の後のセミコロンが期待されていました")?;
                Ok(ToplevelDefinition::GVar(GlobalVariableDefinition { name: ident.to_string(), typ: return_type }))
            }
            _ => Err(AppError {
                message: "トップレベルに識別子がありますが、その後に来たものが「関数引数の丸括弧」でも「グローバル変数定義を終わらせるセミコロン」でも「グローバル変数として配列を定義するための開き角括弧」でもありません"
                    .to_string(),
                input: input.to_string(),
                filename: filename.to_string(),
                pos: *pos + 1,
            }),
        },
        Token { pos, .. } => Err(AppError {
            message: "トップレベルが識別子でないもので始まっています".to_string(),
            input: input.to_string(),
            filename: filename.to_string(),
            pos: *pos + 1,
        }),
    }
}

pub fn parse(
    global_declarations: &mut GlobalDeclarations,
    tokens: &mut Peekable<Iter<Token>>,
    filename: &str,
    input: &str,
) -> Result<Vec<FunctionDefinition>, AppError> {
    let mut function_definitions: Vec<FunctionDefinition> = vec![];
    while tokens.peek().is_some() {
        // If it starts with the keyword `struct`, we might be seeing a toplevel struct definition:
        // `struct Foo { int x; }`
        // To check for that, we peek three tokens:

        if let Some(Token {
            tok: Tok::Struct, ..
        }) = tokens.peek()
        {
            let mut duplicated_iter = tokens.clone();
            duplicated_iter.next();
            if let Some(Token {
                tok: Tok::Identifier(struct_name),
                ..
            }) = duplicated_iter.next()
            {
                if let Some(Token {
                    tok: Tok::開き波括弧,
                    ..
                }) = duplicated_iter.next()
                {
                    // We have a struct definition
                    tokens.next(); // consume `struct`
                    tokens.next(); // consume `struct_name`
                    tokens.next(); // consume `{`

                    let mut members = HashMap::new();
                    let mut overall_alignment = 1;
                    let mut next_member_offset = 0;

                    loop {
                        match tokens.peek() {
                            None => {
                                return Err(AppError {
                                    message: "期待された閉じ波括弧が来ませんでした".to_string(),
                                    input: input.to_string(),
                                    filename: filename.to_string(),
                                    pos: input.len(),
                                })
                            }
                            Some(Token {
                                tok: Tok::閉じ波括弧,
                                ..
                            }) => {
                                tokens.next();

                                break;
                            }
                            _ => {
                                let member_type = parse_type(tokens, filename, input)?;
                                match tokens.next().unwrap() {
                                    Token {
                                        tok: Tok::Identifier(member_name),
                                        ..
                                    } => {
                                        satisfy(
                                            tokens,
                                            filename,
                                            input,
                                            |tok| tok == &Tok::Semicolon,
                                            "メンバーの後にセミコロンがありません",
                                        )?;
                                        let member_size =
                                            member_type.sizeof(&global_declarations.struct_names);
                                        if next_member_offset
                                            % member_type.alignof(&global_declarations.struct_names)
                                            != 0
                                        {
                                            next_member_offset += member_type
                                                .alignof(&global_declarations.struct_names)
                                                - (next_member_offset
                                                    % member_type.alignof(
                                                        &global_declarations.struct_names,
                                                    ));
                                        }
                                        overall_alignment = overall_alignment.max(
                                            member_type.alignof(&global_declarations.struct_names),
                                        );
                                        members.insert(
                                            member_name.to_owned(),
                                            StructMember {
                                                member_type,
                                                offset: next_member_offset,
                                            },
                                        );
                                        next_member_offset += member_size;
                                    }
                                    Token { pos, .. } => {
                                        return Err(AppError {
                                            message: "構造体のメンバー名がありません".to_string(),
                                            input: input.to_string(),
                                            filename: filename.to_string(),
                                            pos: *pos,
                                        })
                                    }
                                }
                            }
                        }
                    }

                    satisfy(
                        tokens,
                        filename,
                        input,
                        |tok| tok == &Tok::Semicolon,
                        "構造体定義の終わりの直後にセミコロンがありません",
                    )?;

                    global_declarations.struct_names.insert(
                        struct_name.clone(),
                        StructDefinition {
                            struct_name: struct_name.clone(),
                            size: next_member_offset.div_ceil(overall_alignment)
                                * overall_alignment,
                            align: overall_alignment,
                            members,
                        },
                    );
                    continue; // skip to the next iteration
                }
            }
        }

        let new_def = parse_toplevel_definition(global_declarations, tokens, filename, input)?;
        match new_def {
            ToplevelDefinition::Func(new_def) => {
                let (name, signature) = new_def.clone().into();
                global_declarations
                    .symbols
                    .insert(name, SymbolDeclaration::Func(signature));
                function_definitions.push(new_def);
            }
            ToplevelDefinition::GVar(gvar) => {
                global_declarations
                    .symbols
                    .insert(gvar.name, SymbolDeclaration::GVar(gvar.typ));
            }
        }
    }
    Ok(function_definitions)
}
