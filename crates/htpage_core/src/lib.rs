pub mod ast;
pub mod compiler;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod token;
pub mod validator;

pub use compiler::compile;
pub use diagnostic::{Diagnostic, DiagnosticSeverity};
pub use span::{LineIndex, Span};
pub use validator::validate;

#[derive(Debug, Clone)]
pub struct ParseOutput {
    pub ast: Vec<ast::Node>,
    pub diagnostics: Vec<Diagnostic>,
    pub line_index: LineIndex,
}

pub fn parse_with_diagnostics(source: &str) -> ParseOutput {
    let line_index = LineIndex::new(source);
    let lexer = lexer::Lexer::new(source);
    let mut parser = parser::Parser::new(lexer);
    let (ast, mut diagnostics) = parser.parse_program_tolerant();
    let mut semantic_diags = validator::validate(&ast);
    diagnostics.append(&mut semantic_diags);
    ParseOutput {
        ast,
        diagnostics,
        line_index,
    }
}
