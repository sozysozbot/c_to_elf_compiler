#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Tok {
    Num(u8),
    Add,
    Sub,
    Asterisk,
    Div,
    開き丸括弧,
    閉じ丸括弧,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
    Assign,
    Semicolon,
    Identifier(String),
    Throw,
    Return,
    If,
    Else,
    While,
    For,
    開き波括弧,
    閉じ波括弧,
    Comma,
    Ampersand,
    Int,
    Sizeof,
    開き角括弧,
    閉じ角括弧,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub pos: usize,
}
