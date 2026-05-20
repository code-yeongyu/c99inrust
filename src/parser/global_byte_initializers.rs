use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::token_scan::{
    matching_top_level_brace, token_is_punctuator, top_level_comma_ranges,
    top_level_punctuator_index,
};
use super::{Constant, parse_integer_initializer_with_constants, parse_string_array_initializer};

pub(super) fn parse_char_matrix_initializer(
    tokens: &[Token],
    rows: usize,
    columns: usize,
    constants: &[Constant],
) -> CompileResult<Vec<u8>> {
    let Ok(values) = parse_string_array_initializer(tokens) else {
        return parse_unsigned_char_matrix_initializer(tokens, rows, columns, constants);
    };
    if values.len() > rows {
        return Err(CompileError::new(
            "global char matrix initializer has too many rows",
        ));
    }
    let length = rows
        .checked_mul(columns)
        .ok_or_else(|| CompileError::new("global char matrix size overflow"))?;
    let mut bytes = Vec::with_capacity(length);
    for value in values {
        if value.len() > columns {
            return Err(CompileError::new(
                "global char matrix initializer row is too large",
            ));
        }
        let row_end = bytes
            .len()
            .checked_add(columns)
            .ok_or_else(|| CompileError::new("global char matrix row size overflow"))?;
        bytes.extend_from_slice(value.as_bytes());
        if value.len() < columns {
            bytes.push(0);
        }
        bytes.resize(row_end, 0);
    }
    bytes.resize(length, 0);
    Ok(bytes)
}

fn parse_unsigned_char_matrix_initializer(
    tokens: &[Token],
    rows: usize,
    columns: usize,
    constants: &[Constant],
) -> CompileResult<Vec<u8>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new("expected global byte matrix initializer"));
    };
    if !token_is_punctuator(first, "{") {
        return Err(CompileError::new("expected global byte matrix initializer")
            .at(first.line, first.column));
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global byte matrix initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global byte matrix initializer")
                .at(token.line, token.column),
        );
    }

    let length = rows
        .checked_mul(columns)
        .ok_or_else(|| CompileError::new("global byte matrix size overflow"))?;
    let mut values = Vec::with_capacity(length);
    let mut start = 1usize;
    let mut row_count = 0usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global byte matrix initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        let item = &item[..item_len];
        if item
            .first()
            .is_some_and(|token| token_is_punctuator(token, "{"))
        {
            let row = parse_unsigned_char_initializer(item, constants)?;
            if row.len() > columns {
                return Err(CompileError::new("global byte matrix row is too large")
                    .at(tokens[start].line, tokens[start].column));
            }
            values.extend(row.iter());
            values.resize(
                values
                    .len()
                    .checked_add(columns - row.len())
                    .ok_or_else(|| CompileError::new("global byte matrix row size overflow"))?,
                0,
            );
            row_count += 1;
        } else {
            let value = parse_integer_initializer_with_constants(item, constants)?;
            values.push(u8::try_from(value).map_err(|_| {
                CompileError::new("global byte matrix initializer does not fit u8")
                    .at(tokens[start].line, tokens[start].column)
            })?);
        }
        if row_count > rows || values.len() > length {
            return Err(
                CompileError::new("too many global byte matrix initializers")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    values.resize(length, 0);
    Ok(values)
}

pub(super) fn parse_unsigned_char_initializer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Vec<u8>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new("expected global array initializer"));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global array initializer").at(first.line, first.column)
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global array initializer").at(first.line, first.column)
        );
    };
    if close_brace + 1 != tokens.len() {
        let token = &tokens[close_brace + 1];
        return Err(
            CompileError::new("unsupported global array initializer").at(token.line, token.column)
        );
    }
    let initializer = &tokens[1..close_brace];
    if initializer.is_empty() {
        return Ok(Vec::new());
    }
    let mut values = Vec::new();
    for (start, end) in top_level_comma_ranges(initializer) {
        if start == end && end == initializer.len() {
            continue;
        }
        if start == end {
            let token = &initializer[start];
            return Err(CompileError::new("expected global array initializer value")
                .at(token.line, token.column));
        }
        let value = parse_integer_initializer_with_constants(&initializer[start..end], constants)?;
        let byte = u8::try_from(value).map_err(|_| {
            CompileError::new("unsigned char initializer does not fit u8")
                .at(initializer[start].line, initializer[start].column)
        })?;
        values.push(byte);
    }
    Ok(values)
}
