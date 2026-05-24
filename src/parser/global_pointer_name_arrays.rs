use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::function_pointer_typedefs::function_pointer_typedef_referent;
use super::global_byte_declarations::parse_unsigned_char_array_length;
use super::global_name_pointer_array_initializers::parse_name_pointer_array_initializer;
use super::global_specifiers::{
    global_specifiers_are_pointer, global_specifiers_are_pointer_typedef,
};
use super::pointer_referent_from_specifiers;
use super::token_scan::{
    matching_top_level_bracket, previous_identifier_index, token_identifier,
    top_level_punctuator_index,
};
use super::{Constant, Global, GlobalInitializer, StructLayout};

pub(super) fn parse_global_pointer_name_array(
    tokens: &[Token],
    known_structs: &[StructLayout],
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
    let Some(assign_index) = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
    else {
        return Ok(None);
    };
    let assign_index = close_bracket + 1 + assign_index;
    let Ok(values) = parse_name_pointer_array_initializer(
        &declaration[assign_index + 1..],
        known_structs,
        constants,
    ) else {
        return Ok(None);
    };
    let length = pointer_array_length(
        declaration,
        open_bracket,
        close_bracket,
        values.len(),
        constants,
    )?;
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global pointer-array name"))?
        .to_owned();
    let referent = pointer_referent_from_specifiers(specifiers)
        .or_else(|| function_pointer_typedef_array_referent(specifiers, function_pointer_typedefs));
    Ok(Some(Global::new(
        name,
        GlobalInitializer::PointerNameArray {
            referent,
            values,
            length,
        },
    )))
}

fn function_pointer_typedef_array_referent(
    specifiers: &[Token],
    function_pointer_typedefs: &[(String, String)],
) -> Option<String> {
    let name = specifiers.iter().rev().find_map(token_identifier)?;
    function_pointer_typedef_referent(function_pointer_typedefs, name)
}

fn pointer_array_length(
    declaration: &[Token],
    open_bracket: usize,
    close_bracket: usize,
    initializer_count: usize,
    constants: &[Constant],
) -> CompileResult<usize> {
    let explicit_length = if open_bracket + 1 == close_bracket {
        None
    } else {
        Some(parse_unsigned_char_array_length(
            &declaration[open_bracket + 1..close_bracket],
            constants,
        )?)
    };
    let length = explicit_length.unwrap_or(initializer_count);
    if initializer_count > length {
        return Err(CompileError::new(
            "too many global pointer-array name initializers",
        ));
    }
    Ok(length)
}

fn declarator_open_bracket(declaration: &[Token]) -> Option<usize> {
    let end = top_level_punctuator_index(declaration, "=").unwrap_or(declaration.len());
    top_level_punctuator_index(&declaration[..end], "[")
}
