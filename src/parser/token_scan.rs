mod declarators;
mod depth;
mod matchers;

pub(super) use declarators::{
    array_declarator_name, last_top_level_identifier, parameter_is_variadic, parameter_is_void,
    previous_identifier, previous_identifier_index,
};
pub(super) use depth::{
    decrease_depth, matching_top_level_brace, matching_top_level_bracket, matching_top_level_paren,
    top_level_comma_ranges, top_level_punctuator_index, update_depths,
};
pub(super) use matchers::{
    last_token_is_punctuator, token_has_keyword, token_identifier, token_is_assignment_operator,
    token_is_keyword, token_is_punctuator,
};
