#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Eof,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Assign,
    Colon,
    Comma,
    Sep(String),
    Word(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub pos: usize,
}

impl Token {
    pub fn new(token_type: TokenType, pos: usize) -> Self {
        Self { token_type, pos }
    }
}
