use crate::front_end::lexer::Token;

use super::{
    CompileError, CompileResult, Expr, Parser, StructLayout,
    eval_integer_initializer_expr_with_constants, matching_top_level_bracket, token_identifier,
    token_is_punctuator,
};

type StructArrayFieldPathDesignator<'a> = (Vec<&'a str>, usize, &'a [Token]);

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

    pub(super) fn struct_array_field_designator<'a>(
        &self,
        tokens: &'a [Token],
    ) -> CompileResult<Option<(&'a str, usize, &'a [Token])>> {
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
            .is_some_and(|token| token_is_punctuator(token, "["))
        {
            return Ok(None);
        }
        let close_bracket = matching_top_level_bracket(tokens, 2)
            .ok_or_else(|| CompileError::new("unterminated struct array field designator"))?;
        if !tokens
            .get(close_bracket + 1)
            .is_some_and(|token| token_is_punctuator(token, "="))
        {
            return Err(CompileError::new(
                "expected struct array field designator assignment",
            ));
        }
        let index = self.designator_index_from_tokens(&tokens[3..close_bracket])?;
        Ok(Some((field, index, &tokens[close_bracket + 2..])))
    }

    pub(super) fn struct_array_field_path_designator<'a>(
        &self,
        tokens: &'a [Token],
    ) -> CompileResult<Option<StructArrayFieldPathDesignator<'a>>> {
        if !tokens
            .first()
            .is_some_and(|token| token_is_punctuator(token, "."))
        {
            return Ok(None);
        }
        let mut fields = Vec::new();
        let mut index = 0usize;
        loop {
            let Some(field) = tokens.get(index + 1).and_then(token_identifier) else {
                return Err(CompileError::new("expected struct initializer field name"));
            };
            fields.push(field);
            index += 2;
            let Some(token) = tokens.get(index) else {
                return Err(CompileError::new(
                    "expected nested struct array designator assignment",
                ));
            };
            if token_is_punctuator(token, ".") {
                continue;
            }
            if token_is_punctuator(token, "=") {
                return Ok(None);
            }
            if token_is_punctuator(token, "[") {
                if fields.len() == 1 {
                    return Ok(None);
                }
                let close_bracket = matching_top_level_bracket(tokens, index).ok_or_else(|| {
                    CompileError::new("unterminated nested struct array field designator")
                })?;
                if !tokens
                    .get(close_bracket + 1)
                    .is_some_and(|token| token_is_punctuator(token, "="))
                {
                    return Err(CompileError::new(
                        "expected nested struct array field designator assignment",
                    ));
                }
                let element_index =
                    self.designator_index_from_tokens(&tokens[index + 1..close_bracket])?;
                return Ok(Some((fields, element_index, &tokens[close_bracket + 2..])));
            }
            return Err(CompileError::new(
                "expected nested struct array designator assignment",
            ));
        }
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

pub(super) fn struct_field_path_designator(
    tokens: &[Token],
) -> CompileResult<Option<(Vec<&str>, &[Token])>> {
    if !tokens
        .first()
        .is_some_and(|token| token_is_punctuator(token, "."))
    {
        return Ok(None);
    }
    let mut fields = Vec::new();
    let mut index = 0usize;
    loop {
        let Some(field) = tokens.get(index + 1).and_then(token_identifier) else {
            return Err(CompileError::new("expected struct initializer field name"));
        };
        fields.push(field);
        index += 2;
        let Some(token) = tokens.get(index) else {
            return Err(CompileError::new(
                "expected nested struct initializer designator assignment",
            ));
        };
        if token_is_punctuator(token, ".") {
            continue;
        }
        if token_is_punctuator(token, "=") {
            return if fields.len() > 1 {
                Ok(Some((fields, &tokens[index + 1..])))
            } else {
                Ok(None)
            };
        }
        if fields.len() == 1 && token_is_punctuator(token, "[") {
            return Ok(None);
        }
        return Err(CompileError::new(
            "expected nested struct initializer designator assignment",
        ));
    }
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
