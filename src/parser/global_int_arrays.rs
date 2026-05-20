use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_byte_declarations::parse_unsigned_char_array_length;
use super::global_int_initializers::{parse_int_array_initializer, parse_int_matrix_initializer};
use super::global_specifiers::global_specifiers_are_int;
use super::token_scan::{
    matching_top_level_bracket, previous_identifier_index, token_identifier, token_is_punctuator,
    top_level_punctuator_index,
};
use super::{Constant, Global, GlobalInitializer, StructLayout};

pub(super) fn parse_global_int_array(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
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
    if !global_specifiers_are_int(&declaration[..name_index], known_structs) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global int-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    if declaration
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        let second_open = close_bracket + 1;
        let Some(second_close) = matching_top_level_bracket(declaration, second_open) else {
            return Err(
                CompileError::new("unterminated global int-matrix declarator").at(
                    declaration[second_open].line,
                    declaration[second_open].column,
                ),
            );
        };
        let rows = parse_unsigned_char_array_length(
            &declaration[open_bracket + 1..close_bracket],
            constants,
        )?;
        let columns = parse_unsigned_char_array_length(
            &declaration[second_open + 1..second_close],
            constants,
        )?;
        let length = rows
            .checked_mul(columns)
            .ok_or_else(|| CompileError::new("global int matrix size overflow"))?;
        let values = if let Some(assign_index) =
            top_level_punctuator_index(&declaration[second_close + 1..], "=")
        {
            let assign_index = second_close + 1 + assign_index;
            parse_int_matrix_initializer(
                &declaration[assign_index + 1..],
                rows,
                columns,
                constants,
                sizeof_symbols,
            )?
        } else {
            vec![0; length]
        };
        let name = token_identifier(&declaration[name_index])
            .ok_or_else(|| CompileError::new("expected global int-matrix name"))?
            .to_owned();
        return Ok(Some(Global::new(
            name,
            GlobalInitializer::IntMatrix { values, columns },
        )));
    }
    let explicit_length = if open_bracket + 1 == close_bracket {
        None
    } else {
        Some(parse_unsigned_char_array_length(
            &declaration[open_bracket + 1..close_bracket],
            constants,
        )?)
    };
    let assign_index = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
        .map(|offset| close_bracket + 1 + offset);
    let values = if let Some(assign_index) = assign_index {
        parse_int_array_initializer(
            &declaration[assign_index + 1..],
            explicit_length,
            constants,
            sizeof_symbols,
        )?
    } else {
        let Some(length) = explicit_length else {
            return Err(CompileError::new("expected unsigned char array length"));
        };
        vec![0; length]
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global int-array name"))?
        .to_owned();
    Ok(Some(Global::new(name, GlobalInitializer::IntArray(values))))
}
