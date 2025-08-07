use crate::parse::statement::Type;

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
    AddAssign,
    SubAssign,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UnaryOp {
    Addr,
    Deref,
    Increment,
    Decrement,
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
    NullPtr {
        pos: usize,
        typ: Type,
    },
    StringLiteral {
        val: String,
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
    DecayedArr {
        expr: Box<Expr>,
        typ: Type,
    },
}

pub fn decay_if_arr(expr: Expr) -> Box<Expr> {
    match expr.typ() {
        Type::Arr(t, _) => Box::new(Expr::DecayedArr {
            expr: Box::new(expr),
            typ: Type::Ptr(t),
        }),
        _ => Box::new(expr),
    }
}

pub fn throw_if_arr(expr: Expr) -> Box<Expr> {
    if let Type::Arr(_, _) = expr.typ() {
        panic!("配列型に対して適用できない操作があります: {expr:?}");
    }
    Box::new(expr)
}

pub fn no_decay_even_if_arr(expr: Expr) -> Box<Expr> {
    Box::new(expr)
}

impl Expr {
    pub fn typ(&self) -> Type {
        match self {
            Expr::BinaryExpr { typ, .. }
            | Expr::Numeric { typ, .. }
            | Expr::StringLiteral { typ, .. }
            | Expr::Identifier { typ, .. }
            | Expr::Call { typ, .. }
            | Expr::DecayedArr { typ, .. }
            | Expr::UnaryExpr { typ, .. }
            | Expr::NullPtr { typ, .. } => (*typ).clone(),
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
