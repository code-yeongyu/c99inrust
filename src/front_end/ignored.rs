use crate::diagnostics::{CompileError, CompileResult};

use super::scanner::Scanner;

pub(super) fn skip(scanner: &mut Scanner) -> CompileResult<()> {
    loop {
        while scanner.current().is_some_and(char::is_whitespace) {
            scanner.advance();
        }
        if scanner.current() == Some('/') && scanner.peek() == Some('/') {
            while scanner.current().is_some_and(|value| value != '\n') {
                scanner.advance();
            }
            continue;
        }
        if scanner.current() == Some('/') && scanner.peek() == Some('*') {
            skip_block_comment(scanner)?;
            continue;
        }
        return Ok(());
    }
}

fn skip_block_comment(scanner: &mut Scanner) -> CompileResult<()> {
    let line = scanner.line;
    let column = scanner.column;
    scanner.advance();
    scanner.advance();
    while scanner.current().is_some() {
        if scanner.current() == Some('*') && scanner.peek() == Some('/') {
            scanner.advance();
            scanner.advance();
            return Ok(());
        }
        scanner.advance();
    }
    Err(CompileError::new("unterminated block comment").at(line, column))
}
