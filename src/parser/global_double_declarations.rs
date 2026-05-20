use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_byte_declarations::parse_unsigned_char_array_length;
use super::global_specifiers::global_specifiers_are_double;
use super::token_scan::{
    matching_top_level_bracket, previous_identifier_index, token_identifier,
    top_level_punctuator_index,
};
use super::{Constant, Global, GlobalInitializer};

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
    if !global_specifiers_are_double(&declaration[..name_index]) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global double-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    if top_level_punctuator_index(&declaration[close_bracket + 1..], "=").is_some() {
        return Ok(None);
    }
    let length =
        parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket], constants)?;
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global double-array name"))?
        .to_owned();
    Ok(Some(Global::new(
        name,
        GlobalInitializer::DoubleArray { length },
    )))
}
