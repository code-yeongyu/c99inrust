use crate::front_end::lexer::Token;

use super::{
    CompileError, CompileResult, FieldType, LocalStructInitializerValue, Parser, StructLayout,
    field_type_at, resize_values_for_index, struct_field_index,
};

impl Parser<'_> {
    pub(super) fn local_parent_struct_name(
        &self,
        struct_name: &str,
        index_path: &[usize],
    ) -> CompileResult<String> {
        let mut current = struct_name.to_owned();
        for index in &index_path[..index_path.len().saturating_sub(1)] {
            let layout = self.local_struct_layout(&current)?;
            let Some(FieldType::Struct(next)) = field_type_at(layout, *index) else {
                return Err(CompileError::new(
                    "nested struct field designator requires struct field",
                ));
            };
            current = next.clone();
        }
        Ok(current)
    }

    pub(super) fn local_struct_field_index_path(
        &self,
        layout: &StructLayout,
        field_index: usize,
        field_path: &[&str],
    ) -> CompileResult<Vec<usize>> {
        let mut index_path = vec![field_index];
        let mut current_struct = match field_type_at(layout, field_index) {
            Some(FieldType::Struct(struct_name)) => struct_name.clone(),
            Some(_) if field_path.is_empty() => return Ok(index_path),
            _ => {
                return Err(CompileError::new(
                    "nested struct field designator requires struct field",
                ));
            }
        };
        for (position, field_name) in field_path.iter().enumerate() {
            let index = struct_field_index(self.known_structs, &current_struct, field_name)?;
            index_path.push(index);
            if position + 1 < field_path.len() {
                let layout = self.local_struct_layout(&current_struct)?;
                let Some(FieldType::Struct(next_struct)) = field_type_at(layout, index) else {
                    return Err(CompileError::new(
                        "nested struct field designator requires struct field",
                    ));
                };
                current_struct = next_struct.clone();
            }
        }
        Ok(index_path)
    }

    pub(super) fn write_local_struct_index_path_value(
        &self,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        index_path: &[usize],
        value_tokens: &[Token],
    ) -> CompileResult<()> {
        let Some((field_index, nested_path)) = index_path.split_first() else {
            return Err(CompileError::new("expected nested struct field designator"));
        };
        let layout = self.local_struct_layout(struct_name)?;
        resize_values_for_index(values, layout, *field_index);
        let Some(field_type) = field_type_at(layout, *field_index) else {
            return Err(CompileError::new("unknown nested struct field designator"));
        };
        if nested_path.is_empty() {
            values[*field_index] = self.parse_local_struct_field_value(field_type, value_tokens)?;
            return Ok(());
        }
        let FieldType::Struct(nested_struct_name) = field_type else {
            return Err(CompileError::new(
                "nested struct field designator requires struct field",
            ));
        };
        let LocalStructInitializerValue::Nested(nested_values) = &mut values[*field_index] else {
            return Err(CompileError::new(
                "nested struct field designator requires nested field value",
            ));
        };
        self.write_local_struct_index_path_value(
            nested_struct_name,
            nested_values,
            nested_path,
            value_tokens,
        )
    }

    pub(super) fn write_local_struct_array_index_path_value(
        &self,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        index_path: &[usize],
        element_index: usize,
        value_tokens: &[Token],
    ) -> CompileResult<()> {
        let Some((field_index, nested_path)) = index_path.split_first() else {
            return Err(CompileError::new(
                "expected nested struct array field designator",
            ));
        };
        let layout = self.local_struct_layout(struct_name)?;
        resize_values_for_index(values, layout, *field_index);
        if nested_path.is_empty() {
            return self.write_local_array_field_designator(
                layout,
                values,
                *field_index,
                element_index,
                value_tokens,
            );
        }
        let Some(FieldType::Struct(nested_struct_name)) = field_type_at(layout, *field_index)
        else {
            return Err(CompileError::new(
                "nested struct array field designator requires struct field",
            ));
        };
        let LocalStructInitializerValue::Nested(nested_values) = &mut values[*field_index] else {
            return Err(CompileError::new(
                "nested struct array field designator requires nested field value",
            ));
        };
        self.write_local_struct_array_index_path_value(
            nested_struct_name,
            nested_values,
            nested_path,
            element_index,
            value_tokens,
        )
    }
}
