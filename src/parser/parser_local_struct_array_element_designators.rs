use crate::front_end::lexer::Token;

use super::parser_local_struct_designator_cursor::{
    LocalStructDesignatorCursor, LocalStructDesignatorWrite,
};
use super::{
    CompileError, CompileResult, FieldType, LocalStructInitializerValue, Parser, StructLayout,
    field_type_at, resize_values_for_index, struct_field_index,
};

impl Parser<'_> {
    pub(super) fn write_local_struct_array_element_designator_item(
        &self,
        layout: &StructLayout,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        item: &[Token],
    ) -> CompileResult<Option<LocalStructDesignatorWrite>> {
        let Some(designator) = self.struct_array_element_field_designator(item)? else {
            return Ok(None);
        };
        let index = struct_field_index(self.known_structs, struct_name, designator.array_path[0])?;
        let array_path =
            self.local_struct_field_index_path(layout, index, &designator.array_path[1..])?;
        let (element_struct_name, _) =
            self.local_struct_array_field_info(struct_name, &array_path)?;
        let field_path =
            self.local_struct_field_path_from_names(&element_struct_name, &designator.field_path)?;
        self.write_local_struct_array_element_field_path_value(
            struct_name,
            values,
            &array_path,
            designator.element_index,
            &field_path,
            designator.value_tokens,
        )?;
        self.next_local_struct_array_element_field_cursor(
            struct_name,
            &array_path,
            designator.element_index,
            &field_path,
        )
        .map(Some)
    }

    pub(super) fn write_local_struct_array_element_cursor_value(
        &self,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        array_path: &[usize],
        element_index: usize,
        field_path: &[usize],
        value_tokens: &[Token],
    ) -> CompileResult<LocalStructDesignatorWrite> {
        self.write_local_struct_array_element_field_path_value(
            struct_name,
            values,
            array_path,
            element_index,
            field_path,
            value_tokens,
        )?;
        self.next_local_struct_array_element_field_cursor(
            struct_name,
            array_path,
            element_index,
            field_path,
        )
    }

    pub(super) fn write_local_struct_array_element_field_path_value(
        &self,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        array_path: &[usize],
        element_index: usize,
        field_path: &[usize],
        value_tokens: &[Token],
    ) -> CompileResult<()> {
        let Some((field_index, nested_path)) = array_path.split_first() else {
            return Err(CompileError::new(
                "expected struct-array element field designator",
            ));
        };
        let layout = self.local_struct_layout(struct_name)?;
        resize_values_for_index(values, layout, *field_index);
        if nested_path.is_empty() {
            return self.write_local_struct_array_element_value(
                layout,
                values,
                *field_index,
                element_index,
                field_path,
                value_tokens,
            );
        }
        let Some(FieldType::Struct(nested_struct_name)) = field_type_at(layout, *field_index)
        else {
            return Err(CompileError::new(
                "struct-array element designator requires struct path",
            ));
        };
        let LocalStructInitializerValue::Nested(nested_values) = &mut values[*field_index] else {
            return Err(CompileError::new(
                "struct-array element designator requires nested field value",
            ));
        };
        self.write_local_struct_array_element_field_path_value(
            nested_struct_name,
            nested_values,
            nested_path,
            element_index,
            field_path,
            value_tokens,
        )
    }

    pub(super) fn next_local_struct_array_element_field_cursor(
        &self,
        struct_name: &str,
        array_path: &[usize],
        element_index: usize,
        field_path: &[usize],
    ) -> CompileResult<LocalStructDesignatorWrite> {
        let (element_struct_name, length) =
            self.local_struct_array_field_info(struct_name, array_path)?;
        if let Some(next_field_path) =
            self.next_local_struct_field_path(&element_struct_name, field_path)?
        {
            return Ok(LocalStructDesignatorWrite {
                next_index: array_path[0] + 1,
                cursor: Some(LocalStructDesignatorCursor::StructArrayFieldPath {
                    array_path: array_path.to_vec(),
                    element_index,
                    field_path: next_field_path,
                }),
            });
        }
        let next_element = element_index
            .checked_add(1)
            .ok_or_else(|| CompileError::new("struct-array designator index overflow"))?;
        if next_element < length {
            return Ok(LocalStructDesignatorWrite {
                next_index: array_path[0] + 1,
                cursor: Some(LocalStructDesignatorCursor::StructArrayFieldPath {
                    array_path: array_path.to_vec(),
                    element_index: next_element,
                    field_path: vec![0],
                }),
            });
        }
        self.next_local_struct_field_cursor(struct_name, array_path)
    }

    pub(super) fn local_struct_array_field_info(
        &self,
        struct_name: &str,
        array_path: &[usize],
    ) -> CompileResult<(String, usize)> {
        let Some(field_index) = array_path.last().copied() else {
            return Err(CompileError::new(
                "expected struct-array element field designator",
            ));
        };
        let parent_struct_name = self.local_parent_struct_name(struct_name, array_path)?;
        let parent_layout = self.local_struct_layout(&parent_struct_name)?;
        let Some(FieldType::StructArray {
            struct_name,
            length,
        }) = field_type_at(parent_layout, field_index)
        else {
            return Err(CompileError::new(
                "struct-array element designator requires struct-array field",
            ));
        };
        Ok((struct_name.clone(), *length))
    }

    pub(super) fn local_struct_field_path_from_names(
        &self,
        struct_name: &str,
        field_path: &[&str],
    ) -> CompileResult<Vec<usize>> {
        let Some((field_name, nested_path)) = field_path.split_first() else {
            return Err(CompileError::new(
                "expected struct-array element field designator field path",
            ));
        };
        let layout = self.local_struct_layout(struct_name)?;
        let index = struct_field_index(self.known_structs, struct_name, field_name)?;
        self.local_struct_field_index_path(layout, index, nested_path)
    }

    fn write_local_struct_array_element_value(
        &self,
        layout: &StructLayout,
        values: &mut [LocalStructInitializerValue],
        field_index: usize,
        element_index: usize,
        field_path: &[usize],
        value_tokens: &[Token],
    ) -> CompileResult<()> {
        let Some(FieldType::StructArray { struct_name, .. }) = field_type_at(layout, field_index)
        else {
            return Err(CompileError::new(
                "struct-array element designator requires struct-array field",
            ));
        };
        let LocalStructInitializerValue::Nested(elements) = &mut values[field_index] else {
            return Err(CompileError::new(
                "struct-array element designator requires nested field value",
            ));
        };
        if elements.len() <= element_index {
            elements.resize_with(element_index + 1, || {
                LocalStructInitializerValue::Nested(Vec::new())
            });
        }
        if !matches!(
            elements[element_index],
            LocalStructInitializerValue::Nested(_)
        ) {
            elements[element_index] = LocalStructInitializerValue::Nested(Vec::new());
        }
        let LocalStructInitializerValue::Nested(element_values) = &mut elements[element_index]
        else {
            return Err(CompileError::new(
                "struct-array element designator requires nested element value",
            ));
        };
        self.write_local_struct_index_path_value(
            struct_name,
            element_values,
            field_path,
            value_tokens,
        )
    }
}
