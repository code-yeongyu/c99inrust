use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::ScalarType;
use super::token_scan::{token_identifier, token_is_punctuator, update_depths};
use super::type_recognition::supported_typedef_scalar;
use super::typedef_referent;

const POINTER_REFERENT: &str = "*";

pub(super) fn parameter_scalar_type(
    tokens: &[Token],
    known_scalar_typedefs: &[String],
    known_pointer_typedefs: &[String],
) -> Option<ScalarType> {
    if parameter_has_pointer(tokens) {
        return Some(ScalarType::Pointer);
    }
    let name_index = tokens
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, token)| token_identifier(token).map(|_name| index))?;
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
    let name_index = tokens
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, token)| token_identifier(token).map(|_name| index))?;
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

fn parameter_array_depth(tokens: &[Token]) -> usize {
    tokens
        .iter()
        .filter(|token| token_is_punctuator(token, "["))
        .count()
}

pub(super) fn declaration_base_referent_type(tokens: &[Token]) -> Option<String> {
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Char)))
    {
        return Some("char".to_owned());
    }
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Int)))
    {
        return Some("int".to_owned());
    }
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Short)))
    {
        return Some("short".to_owned());
    }
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Void)))
    {
        return Some("void".to_owned());
    }
    tokens
        .iter()
        .rev()
        .find_map(token_identifier)
        .and_then(|name| {
            typedef_referent::byte_sized(name)
                .map(ToOwned::to_owned)
                .or_else(|| {
                    supported_typedef_scalar(name)
                        .is_none()
                        .then(|| name.to_owned())
                })
        })
}

pub(super) fn integer_parameter_type(tokens: &[Token]) -> Option<ScalarType> {
    integer_parameter_type_with_typedefs(tokens, &[])
}

fn integer_parameter_type_with_typedefs(
    tokens: &[Token],
    known_scalar_typedefs: &[String],
) -> Option<ScalarType> {
    let mut saw_type = false;
    let mut long_count = 0usize;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(
                Keyword::Const | Keyword::Register | Keyword::Restrict | Keyword::Volatile,
            ) => {}
            TokenKind::Keyword(
                Keyword::Char | Keyword::Int | Keyword::Short | Keyword::Signed | Keyword::Unsigned,
            ) => saw_type = true,
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
    if long_count == 0 {
        Some(ScalarType::Int)
    } else {
        Some(ScalarType::LongLong)
    }
}

pub(super) fn pointer_referent_from_specifiers(tokens: &[Token]) -> Option<String> {
    let pointer_depth = tokens
        .iter()
        .filter(|token| token_is_punctuator(token, "*"))
        .count();
    if pointer_depth == 0 {
        return None;
    }
    pointer_referent_for_depth(
        pointer_depth,
        declaration_base_referent_type(tokens).as_deref(),
    )
}

pub(super) fn pointer_referent_for_depth(
    pointer_depth: usize,
    base_referent: Option<&str>,
) -> Option<String> {
    match pointer_depth {
        0 => None,
        1 => base_referent.map(ToOwned::to_owned),
        depth => {
            let mut referent = POINTER_REFERENT.repeat(depth - 1);
            if let Some(base) = base_referent {
                referent.push_str(base);
            }
            Some(referent)
        }
    }
}
