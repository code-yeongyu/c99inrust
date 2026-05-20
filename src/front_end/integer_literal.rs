use crate::diagnostics::{CompileError, CompileResult};

pub(super) fn parse_hexadecimal(digits: &str, line: usize, column: usize) -> CompileResult<i64> {
    parse_radix(digits, 16, line, column)
}

pub(super) fn parse_decimal_or_octal(
    digits: &str,
    line: usize,
    column: usize,
) -> CompileResult<i64> {
    let Some(octal_digits) = digits.strip_prefix('0') else {
        return parse_radix(digits, 10, line, column);
    };
    if octal_digits.is_empty() {
        return Ok(0);
    }
    if !octal_digits.chars().all(|digit| matches!(digit, '0'..='7')) {
        return Err(CompileError::new("invalid octal integer literal").at(line, column));
    }
    parse_radix(octal_digits, 8, line, column)
}

fn parse_radix(digits: &str, radix: u32, line: usize, column: usize) -> CompileResult<i64> {
    i64::from_str_radix(digits, radix)
        .map_err(|_| CompileError::new("integer literal is too large").at(line, column))
}
