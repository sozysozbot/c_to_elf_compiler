use std::collections::HashMap;

use crate::parse::toplevel::{GlobalDeclarations, SymbolDeclaration, TypeAndSize};


pub struct Context {
    currently_active_local_var_and_param_declarations: Vec<HashMap<String, TypeAndSize>>,
    pub global_declarations: GlobalDeclarations,
}

impl Context {
    pub fn new(
        param_declarations: HashMap<String, TypeAndSize>,
        global_declarations: GlobalDeclarations,
    ) -> Self {
        Self {
            currently_active_local_var_and_param_declarations: vec![param_declarations],
            global_declarations,
        }
    }

    pub fn push_new_scope(&mut self) {
        // push a new scope for local variable declarations
        self.currently_active_local_var_and_param_declarations
            .push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        // pop the most recent scope for local variable declarations
        if self.currently_active_local_var_and_param_declarations.len() > 1 {
            self.currently_active_local_var_and_param_declarations.pop();
        } else {
            panic!("スコープをこれ以上ポップできません。");
        }
    }

    pub fn insert_local_var(&mut self, ident: String, typ_and_size: TypeAndSize) {
        // We insert the local variable into the most recent scope
        let current_scope = self
            .currently_active_local_var_and_param_declarations
            .last_mut()
            .expect("現在のスコープが存在しません");

        // when there is conflict in the same scope, we throw an error
        if current_scope.contains_key(&ident) {
            panic!("ローカル変数の再定義: {ident}");
        }

        current_scope.insert(ident, typ_and_size);
    }

    pub fn resolve_type_and_size_as_var(&self, ident: &str) -> Result<TypeAndSize, String> {
        // loop from the most recent scope to the oldest scope
        // thus, an inverse iteration of Vec
        for scope in self
            .currently_active_local_var_and_param_declarations
            .iter()
            .rev()
        {
            if let Some(typ_and_size) = scope.get(ident) {
                return Ok(typ_and_size.clone());
            }
        }

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
