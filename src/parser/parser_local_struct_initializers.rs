use crate::front_end::lexer::Token;

use super::token_scan::top_level_comma_ranges;
use super::{
    CompileError, CompileResult, FieldType, LocalStructInitializer, LocalStructInitializerValue,
    Parser, StructLayout, matching_top_level_brace, struct_field_designator, struct_field_index,
    token_is_punctuator, zero_expr,
};

impl Parser<'_> {
    pub(super) fn local_struct_initializer(
        &mut self,
        struct_name: &str,
    ) -> CompileResult<LocalStructInitializer> {
        if self.check_punctuator("{") {
            return self
                .local_struct_initializer_values_for_struct(struct_name)
                .map(LocalStructInitializer::Values);
        }
        self.expression().map(LocalStructInitializer::Copy)
    }

    pub(super) fn local_struct_initializer_values(
        &mut self,
    ) -> CompileResult<Vec<LocalStructInitializerValue>> {
        let open_brace = self.index;
        let close_brace = matching_top_level_brace(self.tokens, open_brace)
            .ok_or_else(|| CompileError::new("unterminated local struct initializer"))?;
        let values =
            self.parse_local_struct_initializer_values(&self.tokens[open_brace + 1..close_brace])?;
        self.index = close_brace + 1;
        Ok(values)
    }

    pub(super) fn local_struct_initializer_values_for_struct(
        &mut self,
        struct_name: &str,
    ) -> CompileResult<Vec<LocalStructInitializerValue>> {
        let open_brace = self.index;
        let close_brace = matching_top_level_brace(self.tokens, open_brace)
            .ok_or_else(|| CompileError::new("unterminated local struct initializer"))?;
        let values = self.parse_local_struct_initializer_values_for_struct(
            struct_name,
            &self.tokens[open_brace + 1..close_brace],
        )?;
        self.index = close_brace + 1;
        Ok(values)
    }

    fn parse_local_struct_initializer_values(
        &self,
        tokens: &[Token],
    ) -> CompileResult<Vec<LocalStructInitializerValue>> {
        let mut values = Vec::new();
        for (start, end) in top_level_comma_ranges(tokens) {
            if start == end {
                continue;
            }
            values.push(self.parse_local_struct_initializer_value(&tokens[start..end])?);
        }
        Ok(values)
    }

    fn parse_local_struct_initializer_values_for_struct(
        &self,
        struct_name: &str,
        tokens: &[Token],
    ) -> CompileResult<Vec<LocalStructInitializerValue>> {
        let layout = self.local_struct_layout(struct_name)?;
        let mut values = Vec::new();
        let mut next_index = 0usize;
        for (start, end) in top_level_comma_ranges(tokens) {
            if start == end {
                continue;
            }
            let item = &tokens[start..end];
            let (index, value_tokens) =
                if let Some((field_name, value_tokens)) = struct_field_designator(item)? {
                    let index = struct_field_index(self.known_structs, struct_name, field_name)?;
                    next_index = index + 1;
                    (index, value_tokens)
                } else {
                    let index = next_index;
                    next_index += 1;
                    (index, item)
                };
            let value = if let Some(field_type) = field_type_at(layout, index) {
                self.parse_local_struct_field_value(field_type, value_tokens)?
            } else {
                self.parse_local_struct_initializer_value(value_tokens)?
            };
            resize_values_for_index(&mut values, layout, index);
            values[index] = value;
        }
        Ok(values)
    }

    fn parse_local_struct_field_value(
        &self,
        field_type: &FieldType,
        tokens: &[Token],
    ) -> CompileResult<LocalStructInitializerValue> {
        if let Some(tokens) = braced_value_tokens(tokens)? {
            match field_type {
                FieldType::Struct(struct_name) => {
                    return self
                        .parse_local_struct_initializer_values_for_struct(struct_name, tokens)
                        .map(LocalStructInitializerValue::Nested);
                }
                FieldType::Array { .. } => {
                    return self
                        .parse_local_array_field_initializer_values(tokens)
                        .map(LocalStructInitializerValue::Nested);
                }
                FieldType::Scalar(_)
                | FieldType::Pointer { .. }
                | FieldType::StructArray { .. } => {}
            }
        }
        self.parse_local_struct_initializer_value(tokens)
    }

    pub(super) fn parse_local_struct_initializer_value(
        &self,
        tokens: &[Token],
    ) -> CompileResult<LocalStructInitializerValue> {
        if let Some(tokens) = braced_value_tokens(tokens)? {
            return self
                .parse_local_struct_initializer_values(tokens)
                .map(LocalStructInitializerValue::Nested);
        }
        let mut parser = Parser {
            tokens,
            index: 0,
            known_structs: self.known_structs,
            known_constants: self.known_constants,
            known_scalar_typedefs: self.known_scalar_typedefs,
            known_pointer_typedefs: self.known_pointer_typedefs,
            known_function_pointer_typedefs: self.known_function_pointer_typedefs,
        };
        let expr = parser.expression()?;
        if let Some(token) = parser.peek() {
            return Err(
                CompileError::new("unsupported local struct initializer value")
                    .at(token.line, token.column),
            );
        }
        Ok(LocalStructInitializerValue::Expr(expr))
    }

    fn local_struct_layout(&self, struct_name: &str) -> CompileResult<&StructLayout> {
        self.known_structs
            .iter()
            .find(|layout| layout.name == struct_name)
            .ok_or_else(|| CompileError::new(format!("unknown struct type: {struct_name}")))
    }
}

fn braced_value_tokens(tokens: &[Token]) -> CompileResult<Option<&[Token]>> {
    if !tokens
        .first()
        .is_some_and(|token| token_is_punctuator(token, "{"))
    {
        return Ok(None);
    }
    let close_brace = matching_top_level_brace(tokens, 0)
        .ok_or_else(|| CompileError::new("unterminated local struct initializer value"))?;
    if close_brace + 1 == tokens.len() {
        return Ok(Some(&tokens[1..close_brace]));
    }
    Ok(None)
}

fn field_type_at(layout: &StructLayout, index: usize) -> Option<&FieldType> {
    layout.fields.get(index).map(|field| &field.field_type)
}

fn resize_values_for_index(
    values: &mut Vec<LocalStructInitializerValue>,
    layout: &StructLayout,
    index: usize,
) {
    while values.len() <= index {
        values.push(layout.fields.get(values.len()).map_or_else(
            || LocalStructInitializerValue::Expr(zero_expr()),
            |field| zero_value_for_field(&field.field_type),
        ));
    }
}

const fn zero_value_for_field(field_type: &FieldType) -> LocalStructInitializerValue {
    match field_type {
        FieldType::Scalar(_) | FieldType::Pointer { .. } => {
            LocalStructInitializerValue::Expr(zero_expr())
        }
        FieldType::Struct(_) | FieldType::Array { .. } | FieldType::StructArray { .. } => {
            LocalStructInitializerValue::Nested(Vec::new())
        }
    }
}
