use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_struct_initializer_designator_cursor::{
    GlobalStructDesignatorCursor, GlobalStructDesignatorWrite,
};
use super::global_struct_initializer_designator_cursor_steps::{
    next_struct_field_cursor, next_struct_field_path,
};
use super::global_struct_initializer_designator_paths::{
    global_struct_field_index_path, parent_struct_name,
};
use super::global_struct_initializer_designators::global_struct_layout;
use super::global_struct_initializer_struct_array_paths::write_index_path_value;
use super::{Constant, FieldType, GlobalStructInitializerValue, StructLayout};

pub(super) struct StructArrayElementWriter<'a> {
    known_structs: &'a [StructLayout],
    constants: &'a [Constant],
}

impl<'a> StructArrayElementWriter<'a> {
    pub(super) const fn new(known_structs: &'a [StructLayout], constants: &'a [Constant]) -> Self {
        Self {
            known_structs,
            constants,
        }
    }

    pub(super) fn write_field_path_value(
        &self,
        values: &mut Vec<GlobalStructInitializerValue>,
        struct_name: &str,
        array_path: &[usize],
        element_index: usize,
        field_path: &[usize],
        value_tokens: &[Token],
    ) -> CompileResult<()> {
        let Some((field_index, nested_path)) = array_path.split_first() else {
            return Err(CompileError::new(
                "expected global struct-array element field designator",
            ));
        };
        let layout = global_struct_layout(self.known_structs, struct_name)?;
        if values.len() <= *field_index {
            values.resize(*field_index + 1, GlobalStructInitializerValue::Integer(0));
        }
        if nested_path.is_empty() {
            return self.write_element_value(
                values,
                layout,
                *field_index,
                element_index,
                field_path,
                value_tokens,
            );
        }
        let Some(FieldType::Struct(nested_struct_name)) = layout
            .fields
            .get(*field_index)
            .map(|field| &field.field_type)
        else {
            return Err(CompileError::new(
                "global struct-array element designator requires struct path",
            ));
        };
        if !matches!(
            values[*field_index],
            GlobalStructInitializerValue::Nested(_)
        ) {
            values[*field_index] = GlobalStructInitializerValue::Nested(Vec::new());
        }
        let GlobalStructInitializerValue::Nested(nested_values) = &mut values[*field_index] else {
            return Err(CompileError::new(
                "global struct-array element designator requires nested field value",
            ));
        };
        self.write_field_path_value(
            nested_values,
            nested_struct_name,
            nested_path,
            element_index,
            field_path,
            value_tokens,
        )
    }

    fn write_element_value(
        &self,
        values: &mut [GlobalStructInitializerValue],
        layout: &StructLayout,
        field_index: usize,
        element_index: usize,
        field_path: &[usize],
        value_tokens: &[Token],
    ) -> CompileResult<()> {
        let Some(FieldType::StructArray { struct_name, .. }) = layout
            .fields
            .get(field_index)
            .map(|field| &field.field_type)
        else {
            return Err(CompileError::new(
                "global struct-array element designator requires struct-array field",
            ));
        };
        if !matches!(values[field_index], GlobalStructInitializerValue::Nested(_)) {
            values[field_index] = GlobalStructInitializerValue::Nested(Vec::new());
        }
        let GlobalStructInitializerValue::Nested(elements) = &mut values[field_index] else {
            return Err(CompileError::new(
                "global struct-array element designator requires nested field value",
            ));
        };
        if elements.len() <= element_index {
            elements.resize_with(element_index + 1, || {
                GlobalStructInitializerValue::Nested(Vec::new())
            });
        }
        if !matches!(
            elements[element_index],
            GlobalStructInitializerValue::Nested(_)
        ) {
            elements[element_index] = GlobalStructInitializerValue::Nested(Vec::new());
        }
        let GlobalStructInitializerValue::Nested(element_values) = &mut elements[element_index]
        else {
            return Err(CompileError::new(
                "global struct-array element designator requires nested element value",
            ));
        };
        write_index_path_value(
            element_values,
            self.known_structs,
            struct_name,
            field_path,
            value_tokens,
            self.constants,
        )
    }
}

pub(super) fn next_struct_array_element_field_cursor(
    known_structs: &[StructLayout],
    struct_name: &str,
    array_path: &[usize],
    element_index: usize,
    field_path: &[usize],
) -> CompileResult<GlobalStructDesignatorWrite> {
    let (element_struct_name, length) =
        struct_array_field_info(known_structs, struct_name, array_path)?;
    if let Some(next_field_path) =
        next_struct_field_path(known_structs, &element_struct_name, field_path)?
    {
        return Ok(GlobalStructDesignatorWrite {
            next_index: array_path[0] + 1,
            cursor: Some(GlobalStructDesignatorCursor::StructArrayFieldPath {
                array_path: array_path.to_vec(),
                element_index,
                field_path: next_field_path,
            }),
        });
    }
    let next_element = element_index
        .checked_add(1)
        .ok_or_else(|| CompileError::new("global struct-array designator index overflow"))?;
    if next_element < length {
        return Ok(GlobalStructDesignatorWrite {
            next_index: array_path[0] + 1,
            cursor: Some(GlobalStructDesignatorCursor::StructArrayFieldPath {
                array_path: array_path.to_vec(),
                element_index: next_element,
                field_path: vec![0],
            }),
        });
    }
    next_struct_field_cursor(known_structs, struct_name, array_path)
}

pub(super) fn struct_array_field_info(
    known_structs: &[StructLayout],
    struct_name: &str,
    array_path: &[usize],
) -> CompileResult<(String, usize)> {
    let Some(field_index) = array_path.last().copied() else {
        return Err(CompileError::new(
            "expected global struct-array element field designator",
        ));
    };
    let parent_struct_name = parent_struct_name(known_structs, struct_name, array_path)?;
    let parent_layout = global_struct_layout(known_structs, &parent_struct_name)?;
    let Some(FieldType::StructArray {
        struct_name,
        length,
    }) = parent_layout
        .fields
        .get(field_index)
        .map(|field| &field.field_type)
    else {
        return Err(CompileError::new(
            "global struct-array element designator requires struct-array field",
        ));
    };
    Ok((struct_name.clone(), *length))
}

pub(super) fn struct_field_path_from_names(
    known_structs: &[StructLayout],
    struct_name: &str,
    field_path: &[&str],
) -> CompileResult<Vec<usize>> {
    let Some((field_name, nested_path)) = field_path.split_first() else {
        return Err(CompileError::new(
            "expected global struct-array element designator field path",
        ));
    };
    global_struct_field_index_path(known_structs, struct_name, field_name, nested_path)
}
