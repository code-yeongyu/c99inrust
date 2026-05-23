use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Token, TokenKind};

use super::Constant;
use super::global_string_initializers::parse_string_pointer_initializer;
use super::token_scan::{
    matching_top_level_brace, matching_top_level_bracket, token_is_punctuator,
    top_level_punctuator_index,
};

pub(super) fn parse_string_pointer_array_initializer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Vec<Option<(String, usize)>>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new(
            "expected global pointer-array initializer",
        ));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global pointer-array initializer")
                .at(first.line, first.column),
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global pointer-array initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global pointer-array initializer")
                .at(token.line, token.column),
        );
    }

    let mut values = Vec::new();
    let mut next_index = 0usize;
    let mut start = 1usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global pointer-array initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        let (index, value_tokens) = if let Some((index, value_tokens)) =
            string_pointer_array_designator(&item[..item_len], constants)?
        {
            next_index = index + 1;
            (index, value_tokens)
        } else {
            let index = next_index;
            next_index += 1;
            (index, &item[..item_len])
        };
        if values.len() <= index {
            values.resize(index + 1, None);
        }
        values[index] = parse_string_pointer_array_value(value_tokens, constants)?;
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    Ok(values)
}

fn parse_string_pointer_array_value(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<(String, usize)>> {
    if is_null_pointer_initializer(tokens) {
        return Ok(None);
    }
    parse_string_pointer_initializer(tokens, constants)?
        .map(Some)
        .ok_or_else(|| CompileError::new("expected global string pointer-array initializer value"))
}

fn string_pointer_array_designator<'a>(
    tokens: &'a [Token],
    constants: &[Constant],
) -> CompileResult<Option<(usize, &'a [Token])>> {
    if !tokens
        .first()
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        return Ok(None);
    }
    let close_bracket = matching_top_level_bracket(tokens, 0)
        .ok_or_else(|| CompileError::new("unterminated global pointer-array designator"))?;
    if !tokens
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "="))
    {
        return Err(CompileError::new(
            "expected global pointer-array designator assignment",
        ));
    }
    let index = super::integer_initializer::parse_integer_initializer_with_constants(
        &tokens[1..close_bracket],
        constants,
    )?;
    let index = usize::try_from(index)
        .map_err(|_| CompileError::new("global pointer-array designator is negative"))?;
    Ok(Some((index, &tokens[close_bracket + 2..])))
}

fn is_null_pointer_initializer(tokens: &[Token]) -> bool {
    matches!(
        tokens,
        [Token {
            kind: TokenKind::Integer(0),
            ..
        }]
    )
}
