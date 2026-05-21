use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::scalar_layout::sizeof_scalar_type;
use super::token_scan::token_is_punctuator;
use super::{ReturnType, ScalarType};

pub(super) fn supported_return_type(tokens: &[Token]) -> Option<ReturnType> {
    let mut saw_void = false;
    let mut saw_non_void_type = false;
    let mut saw_pointer = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Identifier(_) => saw_non_void_type = true,
            TokenKind::Keyword(keyword) => match keyword {
                Keyword::Auto
                | Keyword::Const
                | Keyword::Extern
                | Keyword::Inline
                | Keyword::Register
                | Keyword::Static
                | Keyword::Volatile => {}
                Keyword::Void => saw_void = true,
                Keyword::Bool
                | Keyword::Char
                | Keyword::Enum
                | Keyword::Int
                | Keyword::Long
                | Keyword::Short
                | Keyword::Signed
                | Keyword::Unsigned => saw_non_void_type = true,
                _ => return None,
            },
            TokenKind::Punctuator(value) if value == "*" => saw_pointer = true,
            TokenKind::Punctuator(_)
            | TokenKind::Integer(_)
            | TokenKind::LongInteger(_)
            | TokenKind::CharLiteral(_)
            | TokenKind::StringLiteral(_)
            | TokenKind::End => return None,
        }
    }
    match (saw_void, saw_non_void_type, saw_pointer) {
        (true, false, false) => Some(ReturnType::Void),
        (_, _, true) if saw_void || saw_non_void_type => Some(ReturnType::Pointer),
        (false, true, false) => Some(ReturnType::Int),
        _ => None,
    }
}

pub(super) fn supported_cast_type(tokens: &[Token]) -> Option<ScalarType> {
    supported_cast_type_with_typedefs(tokens, &[], &[])
}

pub(super) fn supported_cast_type_with_typedefs(
    tokens: &[Token],
    known_scalar_typedefs: &[String],
    known_pointer_typedefs: &[String],
) -> Option<ScalarType> {
    if tokens.is_empty() {
        return None;
    }
    if cast_type_starts_with_pointer_declarator(tokens) {
        return None;
    }
    let mut saw_type = false;
    let mut saw_bool = false;
    let mut saw_double = false;
    let mut saw_named_type = false;
    let mut saw_pointer = false;
    let mut expecting_struct_tag = false;
    let mut long_count = 0usize;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(
                Keyword::Const | Keyword::Restrict | Keyword::Signed | Keyword::Volatile,
            ) => expecting_struct_tag = false,
            TokenKind::Keyword(Keyword::Double) => {
                saw_type = true;
                saw_double = true;
                expecting_struct_tag = false;
            }
            TokenKind::Keyword(Keyword::Bool) => {
                saw_type = true;
                saw_bool = true;
                expecting_struct_tag = false;
            }
            TokenKind::Keyword(
                Keyword::Char | Keyword::Int | Keyword::Short | Keyword::Unsigned | Keyword::Void,
            ) => {
                saw_type = true;
                expecting_struct_tag = false;
            }
            TokenKind::Keyword(Keyword::Struct) => {
                saw_type = true;
                expecting_struct_tag = true;
            }
            TokenKind::Keyword(Keyword::Long) => {
                saw_type = true;
                long_count += 1;
                expecting_struct_tag = false;
            }
            TokenKind::Identifier(name) => {
                if expecting_struct_tag {
                    expecting_struct_tag = false;
                } else if known_pointer_typedefs.iter().any(|known| known == name) {
                    saw_pointer = true;
                } else if let Some(scalar_type) = supported_typedef_scalar(name).or_else(|| {
                    known_scalar_typedefs
                        .iter()
                        .any(|known_name| known_name == name)
                        .then_some(ScalarType::Int)
                }) {
                    if scalar_type != ScalarType::Int {
                        return None;
                    }
                } else if saw_pointer {
                    return None;
                } else {
                    saw_named_type = true;
                }
                saw_type = true;
            }
            TokenKind::Punctuator(value) if value == "*" => {
                saw_pointer = true;
                expecting_struct_tag = false;
            }
            TokenKind::Integer(_)
            | TokenKind::LongInteger(_)
            | TokenKind::StringLiteral(_)
            | TokenKind::CharLiteral(_)
            | TokenKind::Punctuator(_)
            | TokenKind::End
            | TokenKind::Keyword(_) => return None,
        }
    }
    if !saw_type {
        return None;
    }
    if saw_pointer {
        return Some(ScalarType::Pointer);
    }
    if saw_named_type {
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

fn cast_type_starts_with_pointer_declarator(tokens: &[Token]) -> bool {
    tokens
        .iter()
        .find(|token| {
            !matches!(
                token.kind,
                TokenKind::Keyword(Keyword::Const | Keyword::Restrict | Keyword::Volatile)
            )
        })
        .is_some_and(|token| token_is_punctuator(token, "*"))
}

pub(super) fn sizeof_type(tokens: &[Token]) -> Option<usize> {
    supported_cast_type(tokens).map(|scalar_type| sizeof_scalar_type(tokens, scalar_type))
}

pub(super) fn supported_typedef_scalar(name: &str) -> Option<ScalarType> {
    match name {
        "Atom" | "Bool" | "Colormap" | "Cursor" | "Drawable" | "FILE" | "Font" | "GC"
        | "GameMission_t" | "GameMode_t" | "KeyCode" | "KeySym" | "Language_t" | "Pixmap"
        | "ShmSeg" | "Status" | "Time" | "VisualID" | "Window" | "XID" | "ammotype_t"
        | "angle_t" | "boolean" | "buttoncode_t" | "byte" | "card_t" | "cheat_t" | "command_t"
        | "evtype_t" | "fixed_t" | "gameaction_t" | "gamestate_t" | "key_t" | "lighttable_t"
        | "mobjflag_t" | "mobjtype_t" | "playerstate_t" | "powerduration_t" | "powertype_t"
        | "psprnum_t" | "skill_t" | "slopetype_t" | "spritenum_t" | "statenum_t"
        | "weapontype_t" => Some(ScalarType::Int),
        "va_list" => Some(ScalarType::VaList),
        _ => None,
    }
}
