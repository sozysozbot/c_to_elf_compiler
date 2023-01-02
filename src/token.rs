#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenPayload {
    Num(u8),
    Add,
    Sub,
    Mul,
    Div,
    開き丸括弧,
    閉じ丸括弧,
    Eof,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
    Assign,
    Semicolon,
    Identifier(String),
    Return,
    If,
    Else,
    While,
    For,
    開き波括弧,
    閉じ波括弧,
    Comma,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    pub payload: TokenPayload,
    pub pos: usize,
}
