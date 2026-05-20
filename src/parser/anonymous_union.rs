use crate::front_end::lexer::{Keyword, Token};

use super::token_scan::{token_identifier, token_is_keyword, token_is_punctuator};
use super::{DOOM_EXPAND_PIXEL_UNION, DOOM_NAME8_UNION};

pub(super) fn anonymous_union_struct_name(tokens: &[Token]) -> Option<&'static str> {
    if anonymous_doom_expand_pixel_union(tokens) {
        Some(DOOM_EXPAND_PIXEL_UNION)
    } else if anonymous_doom_name8_union(tokens) {
        Some(DOOM_NAME8_UNION)
    } else {
        None
    }
}

fn anonymous_doom_expand_pixel_union(tokens: &[Token]) -> bool {
    let has_double_d = tokens.windows(2).any(|window| {
        token_is_keyword(&window[0], Keyword::Double)
            && token_identifier(&window[1]).is_some_and(|name| name == "d")
    });
    let has_unsigned_u = tokens.windows(5).any(|window| {
        token_is_keyword(&window[0], Keyword::Unsigned)
            && token_identifier(&window[1]).is_some_and(|name| name == "u")
            && token_is_punctuator(&window[2], "[")
            && window[3].kind.integer_value() == Some(2)
            && token_is_punctuator(&window[4], "]")
    });
    has_double_d && has_unsigned_u
}

fn anonymous_doom_name8_union(tokens: &[Token]) -> bool {
    let has_char_s = tokens.windows(5).any(|window| {
        token_is_keyword(&window[0], Keyword::Char)
            && token_identifier(&window[1]).is_some_and(|name| name == "s")
            && token_is_punctuator(&window[2], "[")
            && window[3].kind.integer_value() == Some(9)
            && token_is_punctuator(&window[4], "]")
    });
    let has_int_x = tokens.windows(5).any(|window| {
        token_is_keyword(&window[0], Keyword::Int)
            && token_identifier(&window[1]).is_some_and(|name| name == "x")
            && token_is_punctuator(&window[2], "[")
            && window[3].kind.integer_value() == Some(2)
            && token_is_punctuator(&window[4], "]")
    });
    has_char_s && has_int_x
}
