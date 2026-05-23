use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Token, TokenKind};

use super::Constant;
use super::integer_initializer::parse_integer_initializer_with_constants;
use super::token_scan::{
    matching_top_level_brace, matching_top_level_bracket, token_identifier, token_is_punctuator,
    top_level_punctuator_index,
};

pub(super) fn parse_name_pointer_array_initializer(
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
        values.push(parse_name_pointer_array_value(
            &item[..item_len],
            constants,
        )?);
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    Ok(values)
}

fn parse_name_pointer_array_value(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<(String, usize)>> {
    if is_null_pointer_initializer(tokens) {
        return Ok(None);
    }
    let Some((base, index)) = parse_name_pointer_initializer(tokens, constants)? else {
        return Err(CompileError::new(
            "expected global pointer-array name initializer",
        ));
    };
    usize::try_from(index)
        .map(|index| Some((base, index)))
        .map_err(|_| CompileError::new("global pointer-array offset must be nonnegative"))
}

fn parse_name_pointer_initializer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<(String, i64)>> {
    if let Some(value) = parse_subscript_address_initializer(tokens, constants)? {
        return Ok(Some(value));
    }
    parse_decay_initializer(tokens, constants)
}

fn parse_decay_initializer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<(String, i64)>> {
    if let Some(name) = single_identifier(tokens) {
        return Ok(Some((name.to_owned(), 0)));
    }
    if let Some(index) = top_level_punctuator_index(tokens, "+") {
        if let Some(base) = single_identifier(&tokens[..index]) {
            let offset = parse_integer_initializer_with_constants(&tokens[index + 1..], constants)?;
            return Ok(Some((base.to_owned(), offset)));
        }
        if let Some(base) = single_identifier(&tokens[index + 1..]) {
            let offset = parse_integer_initializer_with_constants(&tokens[..index], constants)?;
            return Ok(Some((base.to_owned(), offset)));
        }
    }
    if let Some(index) = top_level_punctuator_index(tokens, "-")
        && let Some(base) = single_identifier(&tokens[..index])
    {
        let offset = parse_integer_initializer_with_constants(&tokens[index + 1..], constants)?;
        return Ok(Some((base.to_owned(), -offset)));
    }
    Ok(None)
}

fn parse_subscript_address_initializer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<(String, i64)>> {
    if !tokens
        .first()
        .is_some_and(|token| token_is_punctuator(token, "&"))
    {
        return Ok(None);
    }
    let Some(base) = tokens.get(1).and_then(token_identifier) else {
        return Ok(None);
    };
    if !tokens
        .get(2)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(tokens, 2) else {
        return Err(
            CompileError::new("unterminated global pointer initializer subscript")
                .at(tokens[2].line, tokens[2].column),
        );
    };
    let index = parse_integer_initializer_with_constants(&tokens[3..close_bracket], constants)?;
    let index = offset_subscript_index(index, &tokens[close_bracket + 1..], constants)?;
    Ok(Some((base.to_owned(), index)))
}

fn offset_subscript_index(
    index: i64,
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<i64> {
    let Some(operator) = tokens.first() else {
        return Ok(index);
    };
    if tokens.len() == 1 {
        return Err(CompileError::new("expected global pointer-array offset"));
    }
    let offset = parse_integer_initializer_with_constants(&tokens[1..], constants)?;
    if token_is_punctuator(operator, "+") {
        Ok(index + offset)
    } else if token_is_punctuator(operator, "-") {
        Ok(index - offset)
    } else {
        Err(CompileError::new(
            "unsupported global pointer-array subscript offset",
        ))
    }
}

fn single_identifier(tokens: &[Token]) -> Option<&str> {
    let [token] = tokens else {
        return None;
    };
    token_identifier(token)
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
