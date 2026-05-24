use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::function_pointer_typedefs::function_pointer_typedef_referent;
use super::global_byte_declarations::parse_unsigned_char_array_length;
use super::global_specifiers::{
    global_specifiers_are_extern_pointer, global_specifiers_are_extern_pointer_typedef,
    global_specifiers_are_pointer, global_specifiers_are_pointer_typedef,
};
use super::pointer_referent_from_specifiers;
use super::token_scan::{
    matching_top_level_bracket, previous_identifier_index, token_identifier, token_is_punctuator,
    top_level_punctuator_index,
};
use super::{Constant, Global, GlobalInitializer};

pub(super) fn parse_global_pointer_array(
    tokens: &[Token],
    constants: &[Constant],
    known_pointer_typedefs: &[String],
    function_pointer_typedefs: &[(String, String)],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = declarator_open_bracket(declaration) else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    let specifiers = &declaration[..name_index];
    if !global_specifiers_are_pointer(specifiers)
        && !global_specifiers_are_pointer_typedef(specifiers, known_pointer_typedefs)
    {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global pointer-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    if top_level_punctuator_index(&declaration[close_bracket + 1..], "=").is_some() {
        return Ok(None);
    }
    let rows =
        parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket], constants)?;
    let (length, columns, last_dimension_close) = if declaration
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        let second_open = close_bracket + 1;
        let Some(second_close) = matching_top_level_bracket(declaration, second_open) else {
            return Err(
                CompileError::new("unterminated global pointer-matrix declarator").at(
                    declaration[second_open].line,
                    declaration[second_open].column,
                ),
            );
        };
        let columns = parse_unsigned_char_array_length(
            &declaration[second_open + 1..second_close],
            constants,
        )?;
        let length = rows
            .checked_mul(columns)
            .ok_or_else(|| CompileError::new("global pointer-matrix size overflow"))?;
        (length, Some(columns), second_close)
    } else {
        (rows, None, close_bracket)
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global pointer-array name"))?
        .to_owned();
    if top_level_punctuator_index(&declaration[last_dimension_close + 1..], "=").is_some() {
        return Ok(None);
    }
    let referent = pointer_referent_from_specifiers(specifiers)
        .or_else(|| function_pointer_typedef_array_referent(specifiers, function_pointer_typedefs));
    Ok(Some(Global::new(
        name,
        GlobalInitializer::PointerArray {
            referent,
            length,
            columns,
        },
    )))
}

pub(super) fn parse_global_extern_pointer_array(
    tokens: &[Token],
    constants: &[Constant],
    known_pointer_typedefs: &[String],
    function_pointer_typedefs: &[(String, String)],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = declarator_open_bracket(declaration) else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    let specifiers = &declaration[..name_index];
    if !global_specifiers_are_extern_pointer(specifiers)
        && !global_specifiers_are_extern_pointer_typedef(specifiers, known_pointer_typedefs)
    {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated extern global pointer-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    let (columns, last_dimension_close) = if declaration
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        let second_open = close_bracket + 1;
        let Some(second_close) = matching_top_level_bracket(declaration, second_open) else {
            return Err(
                CompileError::new("unterminated extern global pointer-matrix declarator").at(
                    declaration[second_open].line,
                    declaration[second_open].column,
                ),
            );
        };
        let columns = parse_unsigned_char_array_length(
            &declaration[second_open + 1..second_close],
            constants,
        )?;
        (Some(columns), second_close)
    } else {
        (None, close_bracket)
    };
    if top_level_punctuator_index(&declaration[last_dimension_close + 1..], "=").is_some() {
        return Ok(None);
    }
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected extern global pointer-array name"))?
        .to_owned();
    let referent = pointer_referent_from_specifiers(specifiers)
        .or_else(|| function_pointer_typedef_array_referent(specifiers, function_pointer_typedefs));
    Ok(Some(Global::new(
        name,
        GlobalInitializer::ExternPointerArray { referent, columns },
    )))
}

fn declarator_open_bracket(declaration: &[Token]) -> Option<usize> {
    let end = top_level_punctuator_index(declaration, "=").unwrap_or(declaration.len());
    top_level_punctuator_index(&declaration[..end], "[")
}

fn function_pointer_typedef_array_referent(
    specifiers: &[Token],
    function_pointer_typedefs: &[(String, String)],
) -> Option<String> {
    let name = specifiers.iter().rev().find_map(token_identifier)?;
    function_pointer_typedef_referent(function_pointer_typedefs, name)
}
