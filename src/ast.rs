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
        左辺: Box<Expr>,
        右辺: Box<Expr>,
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
        args: Vec<Expr>,
        typ: Type,
    },
    UnaryExpr {
        op: UnaryOp,
        op_pos: usize,
        expr: Box<Expr>,
        typ: Type,
    },
}

impl Expr {
    pub fn typ(&self) -> Type
    where
        Type: Clone,
    {
        match self {
            Expr::BinaryExpr { typ, .. }
            | Expr::Numeric { typ, .. }
            | Expr::Identifier { typ, .. }
            | Expr::Call { typ, .. }
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
