use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token};

use super::token_scan::{
    previous_identifier_index, token_has_keyword, token_identifier, token_is_punctuator,
    top_level_punctuator_index,
};
use super::{
    Constant, Global, GlobalInitializer, StructLayout, global_struct_initializer,
    global_struct_specifier_name,
};

pub(super) fn parse(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let end_index = top_level_punctuator_index(declaration, "=").unwrap_or(declaration.len());
    if top_level_punctuator_index(&declaration[..end_index], "[").is_some() {
        return Ok(None);
    }
    if token_has_keyword(declaration, Keyword::Extern) {
        return Ok(None);
    }
    if end_index != declaration.len()
        && !declaration
            .get(end_index + 1)
            .is_some_and(|token| token_is_punctuator(token, "{"))
    {
        return Ok(None);
    }
    let Some(name_index) = previous_identifier_index(declaration, end_index) else {
        return Ok(None);
    };
    let Some(struct_name) = global_struct_specifier_name(&declaration[..name_index], known_structs)
    else {
        return Ok(None);
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global struct object name"))?
        .to_owned();
    let values = if end_index == declaration.len() {
        Vec::new()
    } else {
        global_struct_initializer::parse_object(
            &declaration[end_index + 1..],
            known_structs,
            constants,
        )?
    };
    Ok(Some(Global::new(
        name,
        GlobalInitializer::StructObject {
            struct_name,
            values,
        },
    )))
}
