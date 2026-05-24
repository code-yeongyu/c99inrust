use crate::front_end::lexer::Token;

use super::{
    CompileError, CompileResult, Parser, matching_top_level_bracket, token_identifier,
    token_is_punctuator,
};

pub(super) struct StructArrayElementFieldDesignator<'a> {
    pub(super) array_path: Vec<&'a str>,
    pub(super) element_index: usize,
    pub(super) field_path: Vec<&'a str>,
    pub(super) value_tokens: &'a [Token],
}

impl Parser<'_> {
    pub(super) fn struct_array_element_field_designator<'a>(
        &self,
        tokens: &'a [Token],
    ) -> CompileResult<Option<StructArrayElementFieldDesignator<'a>>> {
        if !tokens
            .first()
            .is_some_and(|token| token_is_punctuator(token, "."))
        {
            return Ok(None);
        }
        let mut array_path = Vec::new();
        let mut index = 0usize;
        loop {
            if !tokens
                .get(index)
                .is_some_and(|token| token_is_punctuator(token, "."))
            {
                return Ok(None);
            }
            let Some(field) = tokens.get(index + 1).and_then(token_identifier) else {
                return Err(CompileError::new("expected struct initializer field name"));
            };
            array_path.push(field);
            index += 2;
            let Some(token) = tokens.get(index) else {
                return Err(CompileError::new(
                    "expected struct-array element field designator assignment",
                ));
            };
            if token_is_punctuator(token, ".") {
                continue;
            }
            if token_is_punctuator(token, "=") {
                return Ok(None);
            }
            if token_is_punctuator(token, "[") {
                return self
                    .struct_array_element_field_designator_after_index(tokens, array_path, index);
            }
            return Err(CompileError::new(
                "expected struct-array element field designator assignment",
            ));
        }
    }

    fn struct_array_element_field_designator_after_index<'a>(
        &self,
        tokens: &'a [Token],
        array_path: Vec<&'a str>,
        open_bracket: usize,
    ) -> CompileResult<Option<StructArrayElementFieldDesignator<'a>>> {
        let close_bracket = matching_top_level_bracket(tokens, open_bracket).ok_or_else(|| {
            CompileError::new("unterminated struct-array element field designator")
        })?;
        if tokens
            .get(close_bracket + 1)
            .is_some_and(|token| token_is_punctuator(token, "="))
        {
            return Ok(None);
        }
        if !tokens
            .get(close_bracket + 1)
            .is_some_and(|token| token_is_punctuator(token, "."))
        {
            return Err(CompileError::new(
                "expected struct-array element field designator assignment",
            ));
        }
        let element_index =
            self.designator_index_from_tokens(&tokens[open_bracket + 1..close_bracket])?;
        let (field_path, value_tokens) = field_path_after_array_index(tokens, close_bracket + 1)?;
        Ok(Some(StructArrayElementFieldDesignator {
            array_path,
            element_index,
            field_path,
            value_tokens,
        }))
    }
}

fn field_path_after_array_index(
    tokens: &[Token],
    mut index: usize,
) -> CompileResult<(Vec<&str>, &[Token])> {
    let mut field_path = Vec::new();
    loop {
        if !tokens
            .get(index)
            .is_some_and(|token| token_is_punctuator(token, "."))
        {
            return Err(CompileError::new(
                "expected struct-array element field designator assignment",
            ));
        }
        let Some(field) = tokens.get(index + 1).and_then(token_identifier) else {
            return Err(CompileError::new("expected struct initializer field name"));
        };
        field_path.push(field);
        index += 2;
        let Some(token) = tokens.get(index) else {
            return Err(CompileError::new(
                "expected struct-array element field designator assignment",
            ));
        };
        if token_is_punctuator(token, ".") {
            continue;
        }
        if token_is_punctuator(token, "=") {
            return Ok((field_path, &tokens[index + 1..]));
        }
        return Err(CompileError::new(
            "expected struct-array element field designator assignment",
        ));
    }
}
