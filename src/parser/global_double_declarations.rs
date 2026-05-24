use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_byte_declarations::parse_unsigned_char_array_length;
use super::global_floatlike_declarations::global_floatlike_scalar_type;
use super::token_scan::{
    matching_top_level_bracket, previous_identifier_index, token_identifier,
    top_level_punctuator_index,
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
    if top_level_punctuator_index(&declaration[close_bracket + 1..], "=").is_some() {
        return Ok(None);
    }
    let length =
        parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket], constants)?;
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global double-array name"))?
        .to_owned();
    let initializer = if scalar_type == ScalarType::Double {
        GlobalInitializer::DoubleArray { length }
    } else {
        GlobalInitializer::ScalarArray {
            scalar_type,
            length,
        }
    };
    Ok(Some(Global::new(name, initializer)))
}
