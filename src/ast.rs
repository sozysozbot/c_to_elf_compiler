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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Any;

pub type UntypedExpr = Expr<Any>;
pub type TypedExpr = Expr<Type>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr<T> {
    BinaryExpr {
        op: BinaryOp,
        op_pos: usize,
        左辺: Box<Expr<T>>,
        右辺: Box<Expr<T>>,
        typ: T,
    },
    Numeric {
        val: u8,
        pos: usize,
        typ: T,
    },
    Identifier {
        ident: String,
        pos: usize,
        typ: T,
    },
    Call {
        ident: String,
        pos: usize,
        args: Vec<Expr<T>>,
        typ: T,
    },
    UnaryExpr {
        op: UnaryOp,
        op_pos: usize,
        expr: Box<Expr<T>>,
        typ: T,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Statement<T> {
    Expr {
        expr: Box<Expr<T>>,
        semicolon_pos: usize,
    },
    Throw {
        expr: Box<Expr<T>>,
        semicolon_pos: usize,
    },
    Return {
        expr: Box<Expr<T>>,
        semicolon_pos: usize,
    },
    If {
        cond: Box<Expr<T>>,
        then: Box<Statement<T>>,
        else_: Option<Box<Statement<T>>>,
        pos: usize,
    },
    While {
        cond: Box<Expr<T>>,
        body: Box<Statement<T>>,
        pos: usize,
    },
    For {
        init: Option<Box<Expr<T>>>,
        cond: Option<Box<Expr<T>>>,
        update: Option<Box<Expr<T>>>,
        body: Box<Statement<T>>,
        pos: usize,
    },
    Block {
        statements: Vec<Statement<T>>,
        pos: usize,
    },
}
