use crate::front_end::lexer::Token;

use super::token_scan::{matching_top_level_bracket, top_level_comma_ranges};
use super::{
    CompileError, CompileResult, FieldType, LocalStructInitializerValue, Parser, StructLayout,
    eval_integer_initializer_expr_with_constants, field_type_at, resize_values_for_index,
    struct_field_index, struct_field_path_designator, token_is_punctuator, zero_expr,
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

    pub(super) fn designator_index_from_tokens(&self, tokens: &[Token]) -> CompileResult<usize> {
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

    pub(super) fn write_local_struct_designator_item(
        &self,
        layout: &StructLayout,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        item: &[Token],
    ) -> CompileResult<Option<usize>> {
        if let Some((field_name, element_index, value_tokens)) =
            self.struct_array_field_designator(item)?
        {
            let index = struct_field_index(self.known_structs, struct_name, field_name)?;
            resize_values_for_index(values, layout, index);
            self.write_local_array_field_designator(
                layout,
                values,
                index,
                element_index,
                value_tokens,
            )?;
            return Ok(Some(index + 1));
        }
        if let Some((field_path, value_tokens)) = struct_field_path_designator(item)? {
            let index = struct_field_index(self.known_structs, struct_name, field_path[0])?;
            resize_values_for_index(values, layout, index);
            self.write_local_struct_field_path_designator(
                layout,
                values,
                index,
                &field_path[1..],
                value_tokens,
            )?;
            return Ok(Some(index + 1));
        }
        Ok(None)
    }

    pub(super) fn write_local_struct_field_path_designator(
        &self,
        layout: &StructLayout,
        values: &mut [LocalStructInitializerValue],
        field_index: usize,
        field_path: &[&str],
        value_tokens: &[Token],
    ) -> CompileResult<()> {
        let Some(FieldType::Struct(struct_name)) = field_type_at(layout, field_index) else {
            return Err(CompileError::new(
                "nested struct field designator requires struct field",
            ));
        };
        let LocalStructInitializerValue::Nested(nested_values) = &mut values[field_index] else {
            return Err(CompileError::new(
                "nested struct field designator requires nested field value",
            ));
        };
        self.write_local_nested_field_value(
            struct_name.as_str(),
            nested_values,
            field_path,
            value_tokens,
        )
    }

    fn write_local_nested_field_value(
        &self,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        field_path: &[&str],
        value_tokens: &[Token],
    ) -> CompileResult<()> {
        let Some(field_name) = field_path.first() else {
            return Err(CompileError::new("expected nested struct field designator"));
        };
        let layout = self.local_struct_layout(struct_name)?;
        let index = struct_field_index(self.known_structs, struct_name, field_name)?;
        resize_values_for_index(values, layout, index);
        let Some(field_type) = field_type_at(layout, index) else {
            return Err(CompileError::new("unknown nested struct field designator"));
        };
        if field_path.len() == 1 {
            values[index] = self.parse_local_struct_field_value(field_type, value_tokens)?;
            return Ok(());
        }
        let FieldType::Struct(nested_struct_name) = field_type else {
            return Err(CompileError::new(
                "nested struct field designator requires struct field",
            ));
        };
        let LocalStructInitializerValue::Nested(nested_values) = &mut values[index] else {
            return Err(CompileError::new(
                "nested struct field designator requires nested field value",
            ));
        };
        self.write_local_nested_field_value(
            nested_struct_name,
            nested_values,
            &field_path[1..],
            value_tokens,
        )
    }
}
