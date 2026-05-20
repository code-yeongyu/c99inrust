use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::{ScalarFieldType, ScalarType};

pub(super) fn scalar_field_type(specifiers: &[Token], scalar_type: ScalarType) -> ScalarFieldType {
    ScalarFieldType {
        scalar_type,
        byte_size: scalar_byte_size(specifiers, scalar_type),
        is_unsigned: specifiers.iter().any(is_unsigned_int),
    }
}

pub(super) fn sizeof_scalar_type(specifiers: &[Token], scalar_type: ScalarType) -> usize {
    scalar_byte_size(specifiers, scalar_type)
}

pub(super) const fn scalar_size_for_layout(scalar_type: ScalarType) -> usize {
    match scalar_type {
        ScalarType::Int => 4,
        ScalarType::LongLong | ScalarType::Double | ScalarType::Pointer => 8,
        ScalarType::VaList => 24,
    }
}

fn scalar_byte_size(specifiers: &[Token], scalar_type: ScalarType) -> usize {
    if scalar_type == ScalarType::Int {
        if specifiers.iter().any(is_char_sized_int) {
            return 1;
        }
        if specifiers.iter().any(is_short_sized_int) {
            return 2;
        }
    }
    scalar_size_for_layout(scalar_type)
}

fn is_char_sized_int(token: &Token) -> bool {
    matches!(token.kind, TokenKind::Keyword(Keyword::Char))
        || matches!(&token.kind, TokenKind::Identifier(name) if name == "byte")
}

const fn is_short_sized_int(token: &Token) -> bool {
    matches!(token.kind, TokenKind::Keyword(Keyword::Short))
}

const fn is_unsigned_int(token: &Token) -> bool {
    matches!(token.kind, TokenKind::Keyword(Keyword::Unsigned))
}
