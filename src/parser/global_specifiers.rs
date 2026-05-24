use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::StructLayout;
use super::token_scan::{token_has_keyword, token_identifier, token_is_punctuator};

pub(super) fn global_specifiers_are_unsigned_char(tokens: &[Token]) -> bool {
    let mut saw_char = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(
                Keyword::Extern
                | Keyword::Static
                | Keyword::Const
                | Keyword::Volatile
                | Keyword::Unsigned,
            ) => {}
            TokenKind::Keyword(Keyword::Char) => saw_char = true,
            TokenKind::Identifier(name) if name == "byte" => saw_char = true,
            _ => return false,
        }
    }
    saw_char
}

pub(super) fn global_specifiers_are_static_const_char(tokens: &[Token]) -> bool {
    let mut saw_static = false;
    let mut saw_const = false;
    let mut saw_char = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Static) => saw_static = true,
            TokenKind::Keyword(Keyword::Const) => saw_const = true,
            TokenKind::Keyword(Keyword::Char) => saw_char = true,
            _ => return false,
        }
    }
    saw_static && saw_const && saw_char
}

pub(super) fn global_specifiers_are_pointer(tokens: &[Token]) -> bool {
    !token_has_keyword(tokens, Keyword::Extern) && global_specifiers_are_pointer_like(tokens, false)
}

pub(super) fn global_specifiers_are_pointer_typedef(
    tokens: &[Token],
    known_pointer_typedefs: &[String],
) -> bool {
    !token_has_keyword(tokens, Keyword::Extern)
        && global_specifiers_are_pointer_typedef_like(tokens, known_pointer_typedefs, false)
}

pub(super) fn global_specifiers_are_extern_pointer(tokens: &[Token]) -> bool {
    token_has_keyword(tokens, Keyword::Extern) && global_specifiers_are_pointer_like(tokens, true)
}

pub(super) fn global_specifiers_are_extern_pointer_typedef(
    tokens: &[Token],
    known_pointer_typedefs: &[String],
) -> bool {
    token_has_keyword(tokens, Keyword::Extern)
        && global_specifiers_are_pointer_typedef_like(tokens, known_pointer_typedefs, true)
}

fn global_specifiers_are_pointer_like(tokens: &[Token], allow_extern: bool) -> bool {
    let mut saw_type = false;
    let mut saw_pointer = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Extern) if allow_extern => {}
            TokenKind::Keyword(
                Keyword::Const
                | Keyword::Restrict
                | Keyword::Static
                | Keyword::Volatile
                | Keyword::Signed
                | Keyword::Unsigned,
            ) => {}
            TokenKind::Keyword(
                Keyword::Char | Keyword::Int | Keyword::Long | Keyword::Short | Keyword::Void,
            )
            | TokenKind::Identifier(_) => saw_type = true,
            TokenKind::Punctuator(value) if value == "*" => saw_pointer = true,
            _ => return false,
        }
    }
    saw_type && saw_pointer
}

fn global_specifiers_are_pointer_typedef_like(
    tokens: &[Token],
    known_pointer_typedefs: &[String],
    allow_extern: bool,
) -> bool {
    let mut saw_typedef = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Extern) if allow_extern => {}
            TokenKind::Keyword(
                Keyword::Const | Keyword::Restrict | Keyword::Static | Keyword::Volatile,
            ) => {}
            TokenKind::Identifier(name)
                if known_pointer_typedefs.iter().any(|known| known == name) =>
            {
                saw_typedef = true;
            }
            _ => return false,
        }
    }
    saw_typedef
}

pub(super) fn global_struct_specifier_name(
    tokens: &[Token],
    known_structs: &[StructLayout],
) -> Option<String> {
    if tokens.iter().any(|token| token_is_punctuator(token, "*")) {
        return None;
    }
    let name = tokens.iter().rev().find_map(token_identifier)?;
    known_structs
        .iter()
        .any(|layout| layout.name == name)
        .then(|| name.to_owned())
}

pub(super) fn global_specifiers_are_int(tokens: &[Token], known_structs: &[StructLayout]) -> bool {
    !token_has_keyword(tokens, Keyword::Extern)
        && global_specifiers_are_int_like(tokens, false, known_structs)
}

pub(super) fn global_specifiers_are_extern_int(tokens: &[Token]) -> bool {
    token_has_keyword(tokens, Keyword::Extern) && global_specifiers_are_int_like(tokens, true, &[])
}

pub(super) fn global_specifiers_are_long_long(tokens: &[Token]) -> bool {
    !token_has_keyword(tokens, Keyword::Extern)
        && global_specifiers_are_long_long_like(tokens, false)
}

pub(super) fn global_specifiers_are_extern_long_long(tokens: &[Token]) -> bool {
    token_has_keyword(tokens, Keyword::Extern) && global_specifiers_are_long_long_like(tokens, true)
}

pub(super) fn global_specifiers_are_short(tokens: &[Token]) -> bool {
    global_specifiers_are_int_like(tokens, true, &[])
        && tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Short)))
}

fn global_specifiers_are_int_like(
    tokens: &[Token],
    allow_extern: bool,
    known_structs: &[StructLayout],
) -> bool {
    let mut saw_int = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Extern) if allow_extern => {}
            TokenKind::Keyword(
                Keyword::Static | Keyword::Const | Keyword::Volatile | Keyword::Signed,
            ) => {}
            TokenKind::Keyword(Keyword::Unsigned | Keyword::Int | Keyword::Short) => {
                saw_int = true;
            }
            TokenKind::Identifier(name) => {
                if known_structs.iter().any(|layout| layout.name == *name) {
                    return false;
                }
                saw_int = true;
            }
            _ => return false,
        }
    }
    saw_int
}

fn global_specifiers_are_long_long_like(tokens: &[Token], allow_extern: bool) -> bool {
    let mut long_count = 0usize;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Extern) if allow_extern => {}
            TokenKind::Keyword(
                Keyword::Static
                | Keyword::Const
                | Keyword::Volatile
                | Keyword::Signed
                | Keyword::Int,
            ) => {}
            TokenKind::Keyword(Keyword::Long) => long_count += 1,
            _ => return false,
        }
    }
    long_count > 0
}
