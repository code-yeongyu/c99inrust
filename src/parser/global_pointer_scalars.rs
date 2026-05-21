use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_specifiers::global_specifiers_are_pointer;
use super::global_string_initializers::parse_string_initializer;
use super::integer_initializer::{
    parse_integer_initializer, parse_integer_initializer_with_constants,
};
use super::pointer_referent_from_specifiers;
use super::token_scan::{
    matching_top_level_bracket, previous_identifier_index, token_identifier, token_is_punctuator,
    top_level_punctuator_index,
};
use super::{Constant, Global, GlobalInitializer};

pub(super) fn parse_global_pointer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let end_index = top_level_punctuator_index(declaration, "=").unwrap_or(declaration.len());
    if top_level_punctuator_index(&declaration[..end_index], "[").is_some() {
        return Ok(None);
    }
    let Some(name_index) = previous_identifier_index(declaration, end_index) else {
        return Ok(None);
    };
    if !global_specifiers_are_pointer(&declaration[..name_index]) {
        return Ok(None);
    }
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global pointer name"))?
        .to_owned();
    let referent = pointer_referent_from_specifiers(&declaration[..name_index]);
    if end_index != declaration.len() {
        let initializer = &declaration[end_index + 1..];
        if let Ok(value) = parse_string_initializer(initializer) {
            return Ok(Some(Global::new(
                name,
                GlobalInitializer::PointerString { referent, value },
            )));
        }
        if let Some((base, index)) =
            parse_global_pointer_subscript_address_initializer(initializer, constants)?
        {
            return Ok(Some(Global::new(
                name,
                GlobalInitializer::PointerSubscriptAddress {
                    referent,
                    base,
                    index,
                },
            )));
        }
        if let Some((base, index)) = parse_global_pointer_decay_initializer(initializer, constants)?
        {
            return Ok(Some(Global::new(
                name,
                GlobalInitializer::PointerSubscriptAddress {
                    referent,
                    base,
                    index,
                },
            )));
        }
        let Ok(value) = parse_integer_initializer(initializer) else {
            return Ok(None);
        };
        if value != 0 {
            return Ok(None);
        }
    }
    Ok(Some(Global::new(
        name,
        GlobalInitializer::PointerNull { referent },
    )))
}

fn parse_global_pointer_decay_initializer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<(String, usize)>> {
    let Some(base) = tokens.first().and_then(token_identifier) else {
        return Ok(None);
    };
    if tokens.len() == 1 {
        return Ok(Some((base.to_owned(), 0)));
    }
    if !tokens
        .get(1)
        .is_some_and(|token| token_is_punctuator(token, "+"))
    {
        return Ok(None);
    }
    let index = parse_integer_initializer_with_constants(&tokens[2..], constants)?;
    if index < 0 {
        return Err(CompileError::new(
            "global pointer initializer offset must be nonnegative",
        ));
    }
    usize::try_from(index)
        .map(|index| Some((base.to_owned(), index)))
        .map_err(|_| CompileError::new("global pointer initializer offset is too large"))
}

fn parse_global_pointer_subscript_address_initializer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<(String, usize)>> {
    if !tokens
        .first()
        .is_some_and(|token| token_is_punctuator(token, "&"))
    {
        return Ok(None);
    }
    let Some(base) = tokens.get(1).and_then(token_identifier) else {
        return Ok(None);
    };
    if !tokens
        .get(2)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(tokens, 2) else {
        return Err(
            CompileError::new("unterminated global pointer initializer subscript")
                .at(tokens[2].line, tokens[2].column),
        );
    };
    if close_bracket + 1 != tokens.len() {
        return Ok(None);
    }
    let index = parse_integer_initializer_with_constants(&tokens[3..close_bracket], constants)?;
    if index < 0 {
        return Err(
            CompileError::new("global pointer initializer subscript must be nonnegative")
                .at(tokens[2].line, tokens[2].column),
        );
    }
    usize::try_from(index)
        .map(|index| Some((base.to_owned(), index)))
        .map_err(|_| CompileError::new("global pointer initializer subscript is too large"))
}
