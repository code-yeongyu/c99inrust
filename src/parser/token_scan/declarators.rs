use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::depth::update_depths;
use super::matchers::{token_identifier, token_is_punctuator};

pub(in crate::parser) fn parameter_is_void(tokens: &[Token]) -> bool {
    let mut saw_void = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Void) => saw_void = true,
            TokenKind::Keyword(
                Keyword::Const | Keyword::Register | Keyword::Restrict | Keyword::Volatile,
            ) => {}
            _ => return false,
        }
    }
    saw_void
}

pub(in crate::parser) fn parameter_is_variadic(tokens: &[Token]) -> bool {
    matches!(
        tokens,
        [Token {
            kind: TokenKind::Punctuator(value),
            ..
        }] if value == "..."
    )
}

pub(in crate::parser) fn array_declarator_name(tokens: &[Token]) -> Option<String> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && token_is_punctuator(token, "[")
        {
            return previous_identifier(tokens, index).map(ToOwned::to_owned);
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

pub(in crate::parser) fn last_top_level_identifier(tokens: &[Token]) -> Option<String> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut candidate = None;
    for token in tokens {
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && let Some(identifier) = token_identifier(token)
        {
            candidate = Some(identifier.to_owned());
        }
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    candidate
}

pub(in crate::parser) fn previous_identifier(tokens: &[Token], before: usize) -> Option<&str> {
    tokens
        .get(..before)?
        .iter()
        .rev()
        .find_map(token_identifier)
}

pub(in crate::parser) fn previous_identifier_index(
    tokens: &[Token],
    before: usize,
) -> Option<usize> {
    tokens
        .get(..before)?
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, token)| token_identifier(token).map(|_name| index))
}
