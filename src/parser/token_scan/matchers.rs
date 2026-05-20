use crate::front_end::lexer::{Keyword, Token, TokenKind};

pub(in crate::parser) fn token_has_keyword(tokens: &[Token], keyword: Keyword) -> bool {
    tokens.iter().any(|token| token_is_keyword(token, keyword))
}

pub(in crate::parser) fn token_identifier(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Identifier(value) => Some(value),
        _ => None,
    }
}

pub(in crate::parser) fn token_is_keyword(token: &Token, keyword: Keyword) -> bool {
    matches!(&token.kind, TokenKind::Keyword(value) if *value == keyword)
}

pub(in crate::parser) fn token_is_punctuator(token: &Token, expected: &str) -> bool {
    matches!(&token.kind, TokenKind::Punctuator(value) if value == expected)
}

pub(in crate::parser) fn token_is_assignment_operator(token: &Token) -> bool {
    matches!(
        &token.kind,
        TokenKind::Punctuator(value)
            if matches!(
                value.as_str(),
                "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "<<=" | ">>=" | "&=" | "^=" | "|="
            )
    )
}

pub(in crate::parser) fn last_token_is_punctuator(tokens: &[Token], expected: &str) -> bool {
    tokens
        .iter()
        .rev()
        .find(|token| !matches!(token.kind, TokenKind::End))
        .is_some_and(|token| token_is_punctuator(token, expected))
}
