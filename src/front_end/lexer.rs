use crate::diagnostics::CompileResult;

use super::scanner::Scanner;
pub use super::token::{Keyword, Token, TokenKind};

/// Lexes C source into frontend tokens.
///
/// # Errors
///
/// Returns an error when a literal, punctuator, or block comment is malformed
/// for the supported C surface.
pub fn lex(source: &str) -> CompileResult<Vec<Token>> {
    let mut scanner = Scanner::new(source);
    let mut tokens = Vec::new();
    loop {
        let token = scanner.next_token()?;
        let is_end = token.kind == TokenKind::End;
        tokens.push(token);
        if is_end {
            return Ok(tokens);
        }
    }
}
