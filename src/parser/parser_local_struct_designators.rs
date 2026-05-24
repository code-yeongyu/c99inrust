use crate::front_end::lexer::Token;

use super::token_scan::{matching_top_level_bracket, top_level_comma_ranges};
use super::{
    CompileError, CompileResult, LocalStructInitializerValue, Parser,
    eval_integer_initializer_expr_with_constants, token_is_punctuator, zero_expr,
};

impl Parser<'_> {
    pub(super) fn parse_local_array_field_initializer_values(
        &self,
        tokens: &[Token],
    ) -> CompileResult<Vec<LocalStructInitializerValue>> {
        let mut values = Vec::new();
        let mut next_index = 0usize;
        for (start, end) in top_level_comma_ranges(tokens) {
            if start == end {
                continue;
            }
            let item = &tokens[start..end];
            let (index, value_tokens) =
                if let Some((index, value_tokens)) = self.array_designator_from_tokens(item)? {
                    next_index = index + 1;
                    (index, value_tokens)
                } else {
                    let index = next_index;
                    next_index += 1;
                    (index, item)
                };
            if values.len() <= index {
                values.resize_with(index + 1, || LocalStructInitializerValue::Expr(zero_expr()));
            }
            values[index] = self.parse_local_struct_initializer_value(value_tokens)?;
        }
        Ok(values)
    }

    fn array_designator_from_tokens<'a>(
        &self,
        tokens: &'a [Token],
    ) -> CompileResult<Option<(usize, &'a [Token])>> {
        if !tokens
            .first()
            .is_some_and(|token| token_is_punctuator(token, "["))
        {
            return Ok(None);
        }
        let close_bracket = matching_top_level_bracket(tokens, 0)
            .ok_or_else(|| CompileError::new("unterminated local array field designator"))?;
        if !tokens
            .get(close_bracket + 1)
            .is_some_and(|token| token_is_punctuator(token, "="))
        {
            return Err(CompileError::new(
                "expected local array field designator assignment",
            ));
        }
        let index = self.designator_index_from_tokens(&tokens[1..close_bracket])?;
        Ok(Some((index, &tokens[close_bracket + 2..])))
    }

    fn designator_index_from_tokens(&self, tokens: &[Token]) -> CompileResult<usize> {
        let mut parser = Parser {
            tokens,
            index: 0,
            known_structs: self.known_structs,
            known_constants: self.known_constants,
            known_scalar_typedefs: self.known_scalar_typedefs,
            known_pointer_typedefs: self.known_pointer_typedefs,
            known_function_pointer_typedefs: self.known_function_pointer_typedefs,
        };
        let index = eval_integer_initializer_expr_with_constants(
            &parser.expression()?,
            self.known_constants,
        )?
        .to_i64_trunc()?;
        if let Some(token) = parser.peek() {
            return Err(
                CompileError::new("unsupported local array field designator")
                    .at(token.line, token.column),
            );
        }
        usize::try_from(index)
            .map_err(|_| CompileError::new("local array field designator is negative"))
    }
}
