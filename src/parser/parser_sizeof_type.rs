use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::{
    Parser, matching_top_level_bracket, sizeof_type, token_identifier, token_is_keyword,
    token_is_punctuator, top_level_punctuator_index,
};

impl Parser<'_> {
    pub(super) fn sizeof_type(&self, tokens: &[Token]) -> Option<usize> {
        let (base_tokens, length) = type_array_suffix(tokens).unwrap_or((tokens, 1));
        let base_size = self
            .known_struct_type_size(base_tokens)
            .or_else(|| sizeof_type(base_tokens))
            .or_else(|| self.named_struct_type_size(base_tokens))?;
        base_size.checked_mul(length)
    }

    fn known_struct_type_size(&self, tokens: &[Token]) -> Option<usize> {
        if !tokens
            .iter()
            .any(|token| token_is_keyword(token, Keyword::Struct))
            || tokens.iter().any(|token| token_is_punctuator(token, "*"))
        {
            return None;
        }
        self.named_struct_type_size(tokens)
    }

    fn named_struct_type_size(&self, tokens: &[Token]) -> Option<usize> {
        let name = tokens.iter().rev().find_map(token_identifier)?;
        self.known_structs
            .iter()
            .find(|layout| layout.name == name)
            .map(|layout| layout.size)
    }
}

fn type_array_suffix(tokens: &[Token]) -> Option<(&[Token], usize)> {
    let open = top_level_punctuator_index(tokens, "[")?;
    let close = matching_top_level_bracket(tokens, open)?;
    if close + 1 != tokens.len() {
        return None;
    }
    let [
        Token {
            kind: TokenKind::Integer(length),
            ..
        },
    ] = &tokens[open + 1..close]
    else {
        return None;
    };
    usize::try_from(*length)
        .ok()
        .filter(|length| *length > 0)
        .map(|length| (&tokens[..open], length))
}
