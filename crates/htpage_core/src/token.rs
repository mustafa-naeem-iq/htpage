use crate::span::Span;

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
    pub span: Span,
}

impl Token {
    pub fn new(token_type: TokenType, span: Span) -> Self {
        Self { token_type, span }
    }

    pub fn pos(&self) -> usize {
        self.span.start
    }
}
