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
pub enum Expr {
    BinaryExpr {
        op: BinaryOp,
        op_pos: usize,
        左辺: Box<Expr>,
        右辺: Box<Expr>,
    },
    Numeric {
        val: u8,
        pos: usize,
    },
    Identifier {
        ident: String,
        pos: usize,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Statement {
    Expr {
        expr: Box<Expr>,
        semicolon_pos: usize,
    },
    Return {
        expr: Box<Expr>,
        semicolon_pos: usize,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Program {
    Statements(Vec<Statement>),
}
