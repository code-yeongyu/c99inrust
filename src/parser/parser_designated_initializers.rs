use crate::front_end::lexer::Token;

use super::{
    CompileError, CompileResult, Expr, Parser, StructLayout,
    eval_integer_initializer_expr_with_constants, token_identifier, token_is_punctuator,
};

impl Parser<'_> {
    pub(super) fn local_array_designator_index(&mut self) -> CompileResult<Option<usize>> {
        if !self.check_punctuator("[") {
            return Ok(None);
        }
        self.advance();
        let index = eval_integer_initializer_expr_with_constants(
            &self.expression()?,
            self.known_constants,
        )?
        .to_i64_trunc()?;
        self.expect_punctuator("]")?;
        self.expect_punctuator("=")?;
        usize::try_from(index)
            .map(Some)
            .map_err(|_| CompileError::new("local array designator is negative"))
    }
}

pub(super) fn struct_field_designator(tokens: &[Token]) -> CompileResult<Option<(&str, &[Token])>> {
    if !tokens
        .first()
        .is_some_and(|token| token_is_punctuator(token, "."))
    {
        return Ok(None);
    }
    let Some(field) = tokens.get(1).and_then(token_identifier) else {
        return Err(CompileError::new("expected struct initializer field name"));
    };
    if !tokens
        .get(2)
        .is_some_and(|token| token_is_punctuator(token, "="))
    {
        return Err(CompileError::new(
            "expected struct initializer designator assignment",
        ));
    }
    Ok(Some((field, &tokens[3..])))
}

pub(super) fn struct_field_index(
    layouts: &[StructLayout],
    struct_name: &str,
    field_name: &str,
) -> CompileResult<usize> {
    let layout = layouts
        .iter()
        .find(|layout| layout.name == struct_name)
        .ok_or_else(|| CompileError::new(format!("unknown struct type: {struct_name}")))?;
    layout
        .fields
        .iter()
        .position(|field| field.name == field_name)
        .ok_or_else(|| CompileError::new(format!("unknown struct field: {field_name}")))
}

pub(super) const fn zero_expr() -> Expr {
    Expr::Integer(0)
}
