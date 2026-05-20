use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token};

use super::global_byte_initializers::{
    parse_char_matrix_initializer, parse_unsigned_char_initializer,
};
use super::global_specifiers::global_specifiers_are_unsigned_char;
use super::integer_initializer::parse_integer_initializer_with_constants;
use super::token_scan::{
    matching_top_level_bracket, previous_identifier_index, token_has_keyword, token_identifier,
    token_is_punctuator, top_level_punctuator_index,
};
use super::{Constant, Global, GlobalInitializer};

pub(super) fn parse_global_unsigned_char_array(
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
    if !global_specifiers_are_unsigned_char(&declaration[..name_index]) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    if let Some(global) = parse_global_unsigned_char_matrix(
        declaration,
        open_bracket,
        close_bracket,
        name_index,
        constants,
    )? {
        return Ok(Some(global));
    }
    let values = if let Some(assign_index) =
        top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
    {
        let assign_index = close_bracket + 1 + assign_index;
        let Ok(values) =
            parse_unsigned_char_initializer(&declaration[assign_index + 1..], constants)
        else {
            return Ok(None);
        };
        values
    } else {
        let length = parse_unsigned_char_array_length(
            &declaration[open_bracket + 1..close_bracket],
            constants,
        )?;
        vec![0; length]
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global array name"))?
        .to_owned();
    if token_has_keyword(&declaration[..name_index], Keyword::Extern) {
        return Ok(Some(Global::new(
            name,
            GlobalInitializer::ExternUnsignedCharArray,
        )));
    }
    Ok(Some(Global::new(
        name,
        GlobalInitializer::UnsignedCharArray(values),
    )))
}

fn parse_global_unsigned_char_matrix(
    declaration: &[Token],
    open_bracket: usize,
    close_bracket: usize,
    name_index: usize,
    constants: &[Constant],
) -> CompileResult<Option<Global>> {
    if !declaration
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        return Ok(None);
    }
    let second_open = close_bracket + 1;
    let Some(second_close) = matching_top_level_bracket(declaration, second_open) else {
        return Err(
            CompileError::new("unterminated global matrix declarator").at(
                declaration[second_open].line,
                declaration[second_open].column,
            ),
        );
    };
    let rows =
        parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket], constants)?;
    let columns =
        parse_unsigned_char_array_length(&declaration[second_open + 1..second_close], constants)?;
    let length = rows
        .checked_mul(columns)
        .ok_or_else(|| CompileError::new("global byte matrix size overflow"))?;
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global matrix name"))?
        .to_owned();
    if token_has_keyword(&declaration[..name_index], Keyword::Extern) {
        return Ok(Some(Global::new(
            name,
            GlobalInitializer::ExternUnsignedCharMatrix { columns },
        )));
    }
    let values = parse_global_unsigned_char_matrix_values(
        declaration,
        second_close,
        rows,
        columns,
        length,
        constants,
    )?;
    Ok(Some(Global::new(
        name,
        GlobalInitializer::UnsignedCharMatrix { values, columns },
    )))
}

fn parse_global_unsigned_char_matrix_values(
    declaration: &[Token],
    second_close: usize,
    rows: usize,
    columns: usize,
    length: usize,
    constants: &[Constant],
) -> CompileResult<Vec<u8>> {
    let Some(assign_index) = top_level_punctuator_index(&declaration[second_close + 1..], "=")
    else {
        return Ok(vec![0; length]);
    };
    parse_char_matrix_initializer(
        &declaration[second_close + 2 + assign_index..],
        rows,
        columns,
        constants,
    )
}

pub(super) fn parse_unsigned_char_array_length(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<usize> {
    if tokens.is_empty() {
        return Err(CompileError::new("expected unsigned char array length"));
    }
    let value = parse_integer_initializer_with_constants(tokens, constants)?;
    if value <= 0 {
        return Err(CompileError::new("global array length must be positive"));
    }
    usize::try_from(value).map_err(|_| CompileError::new("global array length is too large"))
}
