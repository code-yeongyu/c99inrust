use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::token_scan::{token_identifier, token_is_punctuator};
use super::type_recognition::supported_typedef_scalar;
use super::typedef_referent;

const POINTER_REFERENT: &str = "*";

pub(super) fn declaration_base_referent_type(tokens: &[Token]) -> Option<String> {
    if specifiers_are_unsigned_char(tokens) {
        return Some("byte".to_owned());
    }
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Char)))
    {
        return Some("char".to_owned());
    }
    if specifiers_are_unsigned_short(tokens) {
        return Some("unsigned short".to_owned());
    }
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Short)))
    {
        return Some("short".to_owned());
    }
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Bool)))
    {
        return Some("_Bool".to_owned());
    }
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Int)))
    {
        return Some("int".to_owned());
    }
    complex_referent(tokens)
        .or_else(|| double_referent(tokens))
        .or_else(|| float_referent(tokens))
        .or_else(|| long_referent(tokens))
        .or_else(|| void_referent(tokens))
        .or_else(|| typedef_or_named_referent(tokens))
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

fn complex_referent(tokens: &[Token]) -> Option<String> {
    if !tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Complex)))
    {
        return None;
    }
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Float)))
    {
        Some("float _Complex".to_owned())
    } else if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Long)))
    {
        Some("long double _Complex".to_owned())
    } else {
        Some("double _Complex".to_owned())
    }
}

fn long_referent(tokens: &[Token]) -> Option<String> {
    tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Long)))
        .then(|| "long long".to_owned())
}

fn double_referent(tokens: &[Token]) -> Option<String> {
    if !tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Double)))
    {
        return None;
    }
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Long)))
    {
        Some("long double".to_owned())
    } else {
        Some("double".to_owned())
    }
}

fn float_referent(tokens: &[Token]) -> Option<String> {
    tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Float)))
        .then(|| "float".to_owned())
}

fn void_referent(tokens: &[Token]) -> Option<String> {
    tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Void)))
        .then(|| "void".to_owned())
}

fn typedef_or_named_referent(tokens: &[Token]) -> Option<String> {
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

fn specifiers_are_unsigned_char(tokens: &[Token]) -> bool {
    let mut saw_unsigned = false;
    let mut saw_char = false;
    for token in tokens {
        match token.kind {
            TokenKind::Keyword(Keyword::Unsigned) => saw_unsigned = true,
            TokenKind::Keyword(Keyword::Char) => saw_char = true,
            _ => {}
        }
    }
    saw_unsigned && saw_char
}

fn specifiers_are_unsigned_short(tokens: &[Token]) -> bool {
    let mut saw_unsigned = false;
    let mut saw_short = false;
    for token in tokens {
        match token.kind {
            TokenKind::Keyword(Keyword::Unsigned) => saw_unsigned = true,
            TokenKind::Keyword(Keyword::Short) => saw_short = true,
            _ => {}
        }
    }
    saw_unsigned && saw_short
}
