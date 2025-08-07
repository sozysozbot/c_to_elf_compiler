use std::{collections::HashMap, vec};

use crate::parse::toplevel::{GlobalDeclarations, SymbolDeclaration, TypeAndSize};

type ID = u64;

pub struct Context {
    currently_active_local_var_and_param_declarations: Vec<HashMap<String, (ID, TypeAndSize)>>,
    pub global_declarations: GlobalDeclarations,

    // The list of all local variable declarations, including those that went out of scope.
    // This is used for codegen.
    all_local_var_declarations: Vec<(String, ID, TypeAndSize)>,
    next_local_var_id: ID,
}

impl Context {
    pub fn all_local_var_declarations_cloned(&self) -> Vec<(String, ID, TypeAndSize)> {
        self.all_local_var_declarations.clone()
    }
    pub fn new(
        param_declarations: HashMap<String, TypeAndSize>,
        global_declarations: GlobalDeclarations,
    ) -> Self {
        let mut next_local_var_id = 0;
        let mut param_declarations_with_ids = HashMap::new();
        for (ident, typ_and_size) in param_declarations.iter() {
            param_declarations_with_ids
            .insert(ident.clone(), (next_local_var_id, typ_and_size.clone()));
            next_local_var_id += 1;
        }

        Self {
            currently_active_local_var_and_param_declarations: vec![param_declarations_with_ids],
            global_declarations,
            all_local_var_declarations: vec![],
            next_local_var_id,
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

    #[must_use]
    pub fn insert_local_var(&mut self, ident: String, typ_and_size: TypeAndSize) -> u64 {
        // We insert the local variable into the most recent scope
        let current_scope = self
            .currently_active_local_var_and_param_declarations
            .last_mut()
            .expect("現在のスコープが存在しません");

        // when there is conflict in the same scope, we throw an error
        if current_scope.contains_key(&ident) {
            panic!("ローカル変数の再定義: {ident}");
        }

        let id = self.next_local_var_id;
        self.next_local_var_id += 1;
        
        self.all_local_var_declarations.push((
            ident.clone(),
            id,
            typ_and_size.clone(),
        ));

        current_scope.insert(ident, (id, typ_and_size));
        id
    }

    pub fn resolve_type_and_size_as_var(
        &self,
        ident: &str,
    ) -> Result<(Option<ID>, TypeAndSize), String> {
        // loop from the most recent scope to the oldest scope
        // thus, an inverse iteration of Vec
        for scope in self
            .currently_active_local_var_and_param_declarations
            .iter()
            .rev()
        {
            if let Some((id, typ_and_size)) = scope.get(ident) {
                return Ok((Some(*id), typ_and_size.clone()));
            }
        }

        match self.global_declarations.symbols.get(ident) {
            Some(SymbolDeclaration::GVar(t)) => Ok((
                None,
                TypeAndSize {
                    typ: t.clone(),
                    size: t.sizeof(&self.global_declarations.struct_names),
                },
            )),
            Some(SymbolDeclaration::Func(_u)) => Err(format!(
                "識別子 {ident} は関数であり、現在関数ポインタは実装されていません",
            )),
            None => Err(format!(
                "識別子 {ident} は定義されておらず、型が分かりません",
            )),
        }
    }
}
