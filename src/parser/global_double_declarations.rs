use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_byte_declarations::parse_unsigned_char_array_length;
use super::global_floatlike_declarations::{
    global_floatlike_scalar_type, parse_global_real_initializer,
};
use super::token_scan::{
    matching_top_level_brace, matching_top_level_bracket, previous_identifier_index,
    token_identifier, token_is_punctuator, top_level_punctuator_index,
};
use super::{Constant, Global, GlobalInitializer, ScalarType};

pub(super) fn parse_global_double_array(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    let Some(scalar_type) = global_floatlike_scalar_type(&declaration[..name_index], false) else {
        return Ok(None);
    };
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global double-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    let length =
        parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket], constants)?;
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global double-array name"))?
        .to_owned();
    let initializer = if let Some(assign_index) =
        top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
    {
        let values = parse_global_real_array_initializer(
            &declaration[close_bracket + assign_index + 2..],
            constants,
        )?;
        if values.len() > length {
            return Err(CompileError::new(
                "too many global scalar-array initializers",
            ));
        }
        GlobalInitializer::ScalarArrayValues {
            scalar_type,
            length,
            values,
        }
    } else if scalar_type == ScalarType::Double {
        GlobalInitializer::DoubleArray { length }
    } else {
        GlobalInitializer::ScalarArray {
            scalar_type,
            length,
        }
    };
    Ok(Some(Global::new(name, initializer)))
}

fn parse_global_real_array_initializer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Vec<String>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new(
            "expected global scalar-array initializer",
        ));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global scalar-array initializer")
                .at(first.line, first.column),
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global scalar-array initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global scalar-array initializer")
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
                CompileError::new("expected global scalar-array initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        values.push(parse_global_real_initializer(
            &item[..item_len],
            constants,
            &[],
        )?);
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    Ok(values)
}
