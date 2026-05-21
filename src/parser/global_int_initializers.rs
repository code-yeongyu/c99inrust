use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::token_scan::{
    matching_top_level_brace, matching_top_level_bracket, token_identifier, token_is_punctuator,
    top_level_punctuator_index,
};
use super::{Constant, GlobalInitializer, parse_integer_initializer_with_context};

pub(super) fn parse_global_int_initializer(
    tokens: &[Token],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<GlobalInitializer> {
    if let Ok(value) = parse_integer_initializer_with_context(tokens, constants, sizeof_symbols) {
        return Ok(GlobalInitializer::Int(value));
    }
    match tokens {
        [token] => {
            let Some(name) = token_identifier(token) else {
                return Err(CompileError::new("unsupported global integer initializer")
                    .at(token.line, token.column));
            };
            Ok(GlobalInitializer::IntConstant(name.to_owned()))
        }
        [first, ..] => Err(CompileError::new("unsupported global integer initializer")
            .at(first.line, first.column)),
        [] => Err(CompileError::new("expected global integer initializer")),
    }
}

pub(super) fn parse_int_array_initializer(
    tokens: &[Token],
    explicit_length: Option<usize>,
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Vec<i32>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new("expected global int array initializer"));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global int array initializer").at(first.line, first.column)
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global int array initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global int array initializer")
                .at(token.line, token.column),
        );
    }

    let mut values = Vec::new();
    let mut start = 1usize;
    let mut next_index = 0usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global int array initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        let item = &item[..item_len];
        let (index, value_tokens) = if token_is_punctuator(&item[0], "[") {
            let close = matching_top_level_bracket(item, 0)
                .ok_or_else(|| CompileError::new("unterminated global int array designator"))?;
            let Some(_assign) = item
                .get(close + 1)
                .filter(|token| token_is_punctuator(token, "="))
            else {
                return Err(CompileError::new(
                    "expected global int array designator assignment",
                ));
            };
            let index =
                parse_integer_initializer_with_context(&item[1..close], constants, sizeof_symbols)?;
            let index = usize::try_from(index)
                .map_err(|_| CompileError::new("global int array designator is negative"))?;
            next_index = index + 1;
            (index, &item[(close + 2)..])
        } else {
            let index = next_index;
            next_index += 1;
            (index, item)
        };
        let value =
            parse_integer_initializer_with_context(value_tokens, constants, sizeof_symbols)?;
        if values.len() <= index {
            values.resize(index + 1, 0);
        }
        values[index] = i32::try_from(value).map_err(|_| {
            CompileError::new("global int array initializer does not fit i32")
                .at(tokens[start].line, tokens[start].column)
        })?;
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    let length = explicit_length.unwrap_or(values.len());
    if values.len() > length {
        return Err(CompileError::new("too many global int array initializers")
            .at(first.line, first.column));
    }
    values.resize(length, 0);
    Ok(values)
}

pub(super) fn parse_short_initializer_values(
    values: Vec<i32>,
    is_unsigned: bool,
) -> CompileResult<Vec<i32>> {
    for value in &values {
        if is_unsigned {
            u16::try_from(*value)
                .map_err(|_| CompileError::new("global unsigned short initializer too large"))?;
        } else {
            i16::try_from(*value)
                .map_err(|_| CompileError::new("global short initializer too large"))?;
        }
    }
    Ok(values)
}

pub(super) fn parse_int_matrix_initializer(
    tokens: &[Token],
    rows: usize,
    columns: usize,
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Vec<i32>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new("expected global int matrix initializer"));
    };
    if !token_is_punctuator(first, "{") {
        return Err(CompileError::new("expected global int matrix initializer")
            .at(first.line, first.column));
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global int matrix initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global int matrix initializer")
                .at(token.line, token.column),
        );
    }

    let length = rows
        .checked_mul(columns)
        .ok_or_else(|| CompileError::new("global int matrix size overflow"))?;
    let mut values = Vec::with_capacity(length);
    let mut start = 1usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global int matrix initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        let item = &item[..item_len];
        if item
            .first()
            .is_some_and(|token| token_is_punctuator(token, "{"))
        {
            values.extend(parse_int_array_initializer(
                item,
                Some(columns),
                constants,
                sizeof_symbols,
            )?);
        } else {
            let value = parse_integer_initializer_with_context(item, constants, sizeof_symbols)?;
            values.push(i32::try_from(value).map_err(|_| {
                CompileError::new("global int matrix initializer does not fit i32")
                    .at(tokens[start].line, tokens[start].column)
            })?);
        }
        if values.len() > length {
            return Err(CompileError::new("too many global int matrix initializers")
                .at(tokens[start].line, tokens[start].column));
        }
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    values.resize(length, 0);
    Ok(values)
}
