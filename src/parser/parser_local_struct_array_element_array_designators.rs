use crate::front_end::lexer::Token;

use super::parser_local_struct_designator_cursor::{
    LocalStructDesignatorCursor, LocalStructDesignatorWrite,
};
use super::parser_local_struct_designator_cursor_steps::next_local_struct_field_path;
use super::{
    CompileError, CompileResult, FieldType, LocalStructInitializerValue, Parser, StructLayout,
    field_type_at, resize_values_for_index,
};

#[derive(Clone, Copy)]
pub(super) struct LocalStructArrayElementArrayTarget<'a> {
    pub(super) array_path: &'a [usize],
    pub(super) element_index: usize,
    pub(super) field_path: &'a [usize],
    pub(super) field_element_index: usize,
}

#[derive(Clone, Copy)]
pub(super) struct LocalStructArrayElementArrayWrite<'a> {
    pub(super) target: LocalStructArrayElementArrayTarget<'a>,
    pub(super) value_tokens: &'a [Token],
}

impl Parser<'_> {
    pub(super) fn write_local_struct_array_element_array_cursor_value(
        &self,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        write: LocalStructArrayElementArrayWrite<'_>,
    ) -> CompileResult<LocalStructDesignatorWrite> {
        self.write_local_struct_array_element_array_field_path_value(struct_name, values, write)?;
        self.next_local_struct_array_element_array_field_cursor(struct_name, write.target)
    }

    pub(super) fn write_local_struct_array_element_array_field_path_value(
        &self,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        write: LocalStructArrayElementArrayWrite<'_>,
    ) -> CompileResult<()> {
        let Some((field_index, nested_path)) = write.target.array_path.split_first() else {
            return Err(CompileError::new(
                "expected struct-array element array designator",
            ));
        };
        let layout = self.local_struct_layout(struct_name)?;
        resize_values_for_index(values, layout, *field_index);
        if nested_path.is_empty() {
            return self.write_local_struct_array_element_array_value(
                layout,
                values,
                *field_index,
                write,
            );
        }
        let Some(FieldType::Struct(nested_struct_name)) = field_type_at(layout, *field_index)
        else {
            return Err(CompileError::new(
                "struct-array element array designator requires struct path",
            ));
        };
        let LocalStructInitializerValue::Nested(nested_values) = &mut values[*field_index] else {
            return Err(CompileError::new(
                "struct-array element array designator requires nested field value",
            ));
        };
        self.write_local_struct_array_element_array_field_path_value(
            nested_struct_name,
            nested_values,
            LocalStructArrayElementArrayWrite {
                target: LocalStructArrayElementArrayTarget {
                    array_path: nested_path,
                    ..write.target
                },
                value_tokens: write.value_tokens,
            },
        )
    }

    pub(super) fn next_local_struct_array_element_array_field_cursor(
        &self,
        struct_name: &str,
        target: LocalStructArrayElementArrayTarget<'_>,
    ) -> CompileResult<LocalStructDesignatorWrite> {
        let (element_struct_name, length) =
            self.local_struct_array_field_info(struct_name, target.array_path)?;
        let field_length =
            self.local_struct_array_field_path_len(&element_struct_name, target.field_path)?;
        let next_field_element = target
            .field_element_index
            .checked_add(1)
            .ok_or_else(|| CompileError::new("struct-array nested array index overflow"))?;
        if next_field_element < field_length {
            return Ok(LocalStructDesignatorWrite {
                next_index: target.array_path[0] + 1,
                cursor: Some(LocalStructDesignatorCursor::StructArrayArrayFieldPath {
                    array_path: target.array_path.to_vec(),
                    element_index: target.element_index,
                    field_path: target.field_path.to_vec(),
                    field_element_index: next_field_element,
                }),
            });
        }
        if let Some(next_path) =
            next_local_struct_field_path(self, &element_struct_name, target.field_path)?
        {
            return Ok(LocalStructDesignatorWrite {
                next_index: target.array_path[0] + 1,
                cursor: Some(LocalStructDesignatorCursor::StructArrayFieldPath {
                    array_path: target.array_path.to_vec(),
                    element_index: target.element_index,
                    field_path: next_path,
                }),
            });
        }
        let next_element = target
            .element_index
            .checked_add(1)
            .ok_or_else(|| CompileError::new("struct-array designator index overflow"))?;
        if next_element < length {
            return Ok(LocalStructDesignatorWrite {
                next_index: target.array_path[0] + 1,
                cursor: Some(LocalStructDesignatorCursor::StructArrayFieldPath {
                    array_path: target.array_path.to_vec(),
                    element_index: next_element,
                    field_path: vec![0],
                }),
            });
        }
        self.next_local_struct_field_cursor_after_array_path(struct_name, target.array_path)
    }

    fn write_local_struct_array_element_array_value(
        &self,
        layout: &StructLayout,
        values: &mut [LocalStructInitializerValue],
        field_index: usize,
        write: LocalStructArrayElementArrayWrite<'_>,
    ) -> CompileResult<()> {
        let Some(FieldType::StructArray { struct_name, .. }) = field_type_at(layout, field_index)
        else {
            return Err(CompileError::new(
                "struct-array element array designator requires struct-array field",
            ));
        };
        let LocalStructInitializerValue::Nested(elements) = &mut values[field_index] else {
            return Err(CompileError::new(
                "struct-array element array designator requires nested field value",
            ));
        };
        if elements.len() <= write.target.element_index {
            elements.resize_with(write.target.element_index + 1, || {
                LocalStructInitializerValue::Nested(Vec::new())
            });
        }
        if !matches!(
            elements[write.target.element_index],
            LocalStructInitializerValue::Nested(_)
        ) {
            elements[write.target.element_index] = LocalStructInitializerValue::Nested(Vec::new());
        }
        let LocalStructInitializerValue::Nested(element_values) =
            &mut elements[write.target.element_index]
        else {
            return Err(CompileError::new(
                "struct-array element array designator requires nested element value",
            ));
        };
        self.write_local_struct_array_index_path_value(
            struct_name,
            element_values,
            write.target.field_path,
            write.target.field_element_index,
            write.value_tokens,
        )
    }

    fn local_struct_array_field_path_len(
        &self,
        struct_name: &str,
        field_path: &[usize],
    ) -> CompileResult<usize> {
        let Some(field_index) = field_path.last().copied() else {
            return Err(CompileError::new(
                "expected struct-array element array designator field path",
            ));
        };
        let parent_struct_name = self.local_parent_struct_name(struct_name, field_path)?;
        let parent_layout = self.local_struct_layout(&parent_struct_name)?;
        let Some(FieldType::Array { length, .. }) = field_type_at(parent_layout, field_index)
        else {
            return Err(CompileError::new(
                "struct-array element array designator requires array field",
            ));
        };
        Ok(*length)
    }

    fn next_local_struct_field_cursor_after_array_path(
        &self,
        struct_name: &str,
        array_path: &[usize],
    ) -> CompileResult<LocalStructDesignatorWrite> {
        super::parser_local_struct_designator_cursor_steps::next_local_struct_field_cursor(
            self,
            struct_name,
            array_path,
        )
    }
}
