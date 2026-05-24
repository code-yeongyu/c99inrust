use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::global_byte_declarations::parse_unsigned_char_array_length;
use super::global_int_initializers::parse_int_array_initializer;
use super::token_scan::{
    matching_top_level_bracket, previous_identifier_index, token_has_keyword, token_identifier,
    top_level_punctuator_index,
};
use super::{
    Constant, Global, GlobalInitializer, ScalarType, parse_integer_initializer_with_context,
};

pub(super) fn parse_global_bool(
    tokens: &[Token],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let end_index = top_level_punctuator_index(declaration, "=").unwrap_or(declaration.len());
    if let Some(open_bracket) = top_level_punctuator_index(&declaration[..end_index], "[") {
        return parse_bool_array(declaration, open_bracket, constants, sizeof_symbols);
    }
    parse_bool_scalar(declaration, end_index, constants, sizeof_symbols)
}

fn parse_bool_scalar(
    declaration: &[Token],
    end_index: usize,
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    let Some(name_index) = previous_identifier_index(declaration, end_index) else {
        return Ok(None);
    };
    let specifiers = &declaration[..name_index];
    if !global_specifiers_are_bool(specifiers) {
        return Ok(None);
    }
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global bool name"))?
        .to_owned();
    let initializer =
        if token_has_keyword(specifiers, Keyword::Extern) && end_index == declaration.len() {
            GlobalInitializer::Extern(ScalarType::Bool)
        } else if end_index == declaration.len() {
            GlobalInitializer::Bool(0)
        } else {
            GlobalInitializer::Bool(parse_integer_initializer_with_context(
                &declaration[end_index + 1..],
                constants,
                sizeof_symbols,
            )?)
        };
    Ok(Some(Global::new(name, initializer)))
}

fn parse_bool_array(
    declaration: &[Token],
    open_bracket: usize,
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    if !global_specifiers_are_bool(&declaration[..name_index]) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(CompileError::new(
            "unterminated global bool-array declarator",
        ));
    };
    let length_tokens = &declaration[open_bracket + 1..close_bracket];
    let explicit_length = if length_tokens.is_empty() {
        None
    } else {
        Some(parse_unsigned_char_array_length(length_tokens, constants)?)
    };
    let assign_index = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
        .map(|offset| close_bracket + 1 + offset);
    let values = if let Some(assign_index) = assign_index {
        bool_values(parse_int_array_initializer(
            &declaration[assign_index + 1..],
            explicit_length,
            constants,
            sizeof_symbols,
        )?)
    } else {
        let Some(length) = explicit_length else {
            return Err(CompileError::new("expected global bool array length"));
        };
        vec![0; length]
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global bool-array name"))?
        .to_owned();
    Ok(Some(Global::new(
        name,
        GlobalInitializer::BoolArray(values),
    )))
}

fn bool_values(values: Vec<i32>) -> Vec<u8> {
    values
        .into_iter()
        .map(|value| u8::from(value != 0))
        .collect()
}

fn global_specifiers_are_bool(tokens: &[Token]) -> bool {
    let mut saw_bool = false;
    for token in tokens {
        match token.kind {
            TokenKind::Keyword(
                Keyword::Const | Keyword::Extern | Keyword::Static | Keyword::Volatile,
            ) => {}
            TokenKind::Keyword(Keyword::Bool) => saw_bool = true,
            _ => return false,
        }
    }
    saw_bool
}
