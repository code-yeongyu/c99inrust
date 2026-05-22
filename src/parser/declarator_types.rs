use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::ScalarType;
use super::declaration_base_referent_type;
use super::pointer_referent_for_depth;
use super::token_scan::{token_identifier, token_is_punctuator, update_depths};
use super::type_recognition::supported_typedef_scalar;

pub(super) fn parameter_scalar_type(
    tokens: &[Token],
    known_scalar_typedefs: &[String],
    known_pointer_typedefs: &[String],
) -> Option<ScalarType> {
    if parameter_has_pointer(tokens) {
        return Some(ScalarType::Pointer);
    }
    let name_index = parameter_name_index(tokens)?;
    if tokens[..name_index]
        .iter()
        .rev()
        .find_map(token_identifier)
        .is_some_and(|name| known_pointer_typedefs.iter().any(|known| known == name))
    {
        return Some(ScalarType::Pointer);
    }
    integer_parameter_type_with_typedefs(&tokens[..name_index], known_scalar_typedefs)
}

fn parameter_has_pointer(tokens: &[Token]) -> bool {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for token in tokens {
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && token_is_punctuator(token, "*")
        {
            return true;
        }
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && token_is_punctuator(token, "[")
        {
            return true;
        }
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    false
}

pub(super) fn pointer_referent_type(tokens: &[Token]) -> Option<String> {
    if !parameter_has_pointer(tokens) {
        return None;
    }
    let name_index = parameter_name_index(tokens)?;
    let specifiers = &tokens[..name_index];
    let pointer_depth = specifiers
        .iter()
        .filter(|token| token_is_punctuator(token, "*"))
        .count()
        .saturating_add(parameter_array_depth(&tokens[name_index + 1..]));
    let base_referent = specifiers
        .iter()
        .rev()
        .find_map(token_identifier)
        .filter(|name| supported_typedef_scalar(name).is_none())
        .map(ToOwned::to_owned)
        .or_else(|| declaration_base_referent_type(specifiers));
    pointer_referent_for_depth(pointer_depth, base_referent.as_deref())
}

fn parameter_name_index(tokens: &[Token]) -> Option<usize> {
    let before = parameter_array_start(tokens).unwrap_or(tokens.len());
    tokens[..before]
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, token)| token_identifier(token).map(|_name| index))
}

fn parameter_array_start(tokens: &[Token]) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && token_is_punctuator(token, "[")
        {
            return Some(index);
        }
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    None
}

fn parameter_array_depth(tokens: &[Token]) -> usize {
    tokens
        .iter()
        .filter(|token| token_is_punctuator(token, "["))
        .count()
}

pub(super) fn integer_parameter_type(tokens: &[Token]) -> Option<ScalarType> {
    integer_parameter_type_with_typedefs(tokens, &[])
}

fn integer_parameter_type_with_typedefs(
    tokens: &[Token],
    known_scalar_typedefs: &[String],
) -> Option<ScalarType> {
    let mut saw_type = false;
    let mut saw_bool = false;
    let mut saw_double = false;
    let mut long_count = 0usize;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(
                Keyword::Const | Keyword::Register | Keyword::Restrict | Keyword::Volatile,
            ) => {}
            TokenKind::Keyword(Keyword::Bool) => {
                saw_type = true;
                saw_bool = true;
            }
            TokenKind::Keyword(
                Keyword::Char | Keyword::Int | Keyword::Short | Keyword::Signed | Keyword::Unsigned,
            ) => saw_type = true,
            TokenKind::Keyword(Keyword::Double) => {
                saw_type = true;
                saw_double = true;
            }
            TokenKind::Keyword(Keyword::Long) => {
                saw_type = true;
                long_count += 1;
            }
            TokenKind::Identifier(name) => {
                let scalar_type = supported_typedef_scalar(name).or_else(|| {
                    known_scalar_typedefs
                        .iter()
                        .any(|known_name| known_name == name)
                        .then_some(ScalarType::Int)
                })?;
                if scalar_type != ScalarType::Int {
                    return None;
                }
                saw_type = true;
            }
            _ => return None,
        }
    }
    if !saw_type {
        return None;
    }
    if saw_bool {
        Some(ScalarType::Bool)
    } else if saw_double && long_count == 0 {
        Some(ScalarType::Double)
    } else if saw_double {
        Some(ScalarType::LongDouble)
    } else if long_count == 0 {
        Some(ScalarType::Int)
    } else {
        Some(ScalarType::LongLong)
    }
}
