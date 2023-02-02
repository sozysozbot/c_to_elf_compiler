use std::ops::Deref;

use crate::parse::toplevel::Type;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
    AndThen,
    Assign,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UnaryOp {
    Addr,
    Deref,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr {
    BinaryExpr {
        op: BinaryOp,
        op_pos: usize,
        左辺: Box_<Expr>,
        右辺: Box_<Expr>,
        typ: Type,
    },
    Numeric {
        val: u8,
        pos: usize,
        typ: Type,
    },
    Identifier {
        ident: String,
        pos: usize,
        typ: Type,
    },
    Call {
        ident: String,
        pos: usize,
        args: Vec<Box_<Expr>>,
        typ: Type,
    },
    UnaryExpr {
        op: UnaryOp,
        op_pos: usize,
        expr: Box_<Expr>,
        typ: Type,
    },
    DecayedArr {
        expr: Box_<Expr>,
        typ: Type,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Box_<T>(pub Box<T>);

impl Box_<Expr> {
    pub fn typ(&self) -> Type {
        self.0.typ()
    }
}

impl<T> Deref for Box_<T>{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn decay_if_arr(expr: Expr) -> Box_<Expr> {
    match expr.typ() {
        Type::Arr(t, _) => Box_(Box::new(Expr::DecayedArr {
            expr: Box_(Box::new(expr)),
            typ: Type::Ptr(t),
        })),
        _ => Box_(Box::new(expr)),
    }
}

pub fn no_decay_even_if_arr(expr: Expr) -> Box_<Expr> {
    Box_(Box::new(expr))
}

impl Expr {
    pub fn typ(&self) -> Type {
        match self {
            Expr::BinaryExpr { typ, .. }
            | Expr::Numeric { typ, .. }
            | Expr::Identifier { typ, .. }
            | Expr::Call { typ, .. }
            | Expr::DecayedArr { typ, .. }
            | Expr::UnaryExpr { typ, .. } => (*typ).clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Statement {
    Expr {
        expr: Box<Expr>,
        semicolon_pos: usize,
    },
    Throw {
        expr: Box<Expr>,
        semicolon_pos: usize,
    },
    Return {
        expr: Box<Expr>,
        semicolon_pos: usize,
    },
    If {
        cond: Box<Expr>,
        then: Box<Statement>,
        else_: Option<Box<Statement>>,
        pos: usize,
    },
    While {
        cond: Box<Expr>,
        body: Box<Statement>,
        pos: usize,
    },
    For {
        init: Option<Box<Expr>>,
        cond: Option<Box<Expr>>,
        update: Option<Box<Expr>>,
        body: Box<Statement>,
        pos: usize,
    },
    Block {
        statements: Vec<Statement>,
        pos: usize,
    },
}
