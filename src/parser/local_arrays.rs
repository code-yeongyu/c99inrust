use crate::diagnostics::{CompileError, CompileResult};

use super::{
    Constant, Expr, LocalCharArrayInitializer, eval_integer_initializer_expr_with_constants,
};

pub(super) fn local_array_length(expr: &Expr, constants: &[Constant]) -> CompileResult<usize> {
    let value = match eval_integer_initializer_expr_with_constants(expr, constants) {
        Ok(value) => value.to_i64_trunc()?,
        Err(_) if matches!(expr, Expr::Identifier(_)) => return Ok(1),
        Err(error) => return Err(error),
    };
    if value <= 0 {
        return Err(CompileError::new("local char array size must be positive"));
    }
    usize::try_from(value).map_err(|_| CompileError::new("local char array size is too large"))
}

pub(super) fn inferred_local_char_array_length(value: &str) -> CompileResult<usize> {
    value
        .len()
        .checked_add(1)
        .ok_or_else(|| CompileError::new("local char array size overflow"))
}

pub(super) fn validate_local_char_array_initializer(
    value: &str,
    length: usize,
) -> CompileResult<()> {
    if value.len() > length {
        return Err(CompileError::new(
            "local char array initializer is too large",
        ));
    }
    Ok(())
}

pub(super) fn validate_local_char_array_initializer_size(
    initializer: &LocalCharArrayInitializer,
    length: usize,
) -> CompileResult<()> {
    match initializer {
        LocalCharArrayInitializer::StringLiteral(value) => {
            validate_local_char_array_initializer(value, length)
        }
        LocalCharArrayInitializer::Bytes(values) => {
            if values.len() > length {
                return Err(CompileError::new(
                    "local char array initializer is too large",
                ));
            }
            Ok(())
        }
    }
}
