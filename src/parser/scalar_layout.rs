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
        ScalarType::Bool => 1,
        ScalarType::Int => 4,
        ScalarType::LongLong
        | ScalarType::ComplexFloat
        | ScalarType::Double
        | ScalarType::Pointer => 8,
        ScalarType::ComplexDouble => 16,
        ScalarType::LongDouble => long_double_size_for_layout(),
        ScalarType::ComplexLongDouble => 2 * long_double_size_for_layout(),
        ScalarType::VaList => 24,
    }
}

const fn long_double_size_for_layout() -> usize {
    if cfg!(all(target_arch = "x86_64", not(target_os = "macos"))) {
        16
    } else {
        8
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
