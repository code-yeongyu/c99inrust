use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::global_struct_initializer_designator_cursor::{
    GlobalStructDesignatorCursor, GlobalStructDesignatorWrite,
};
use super::global_struct_initializer_designator_cursor_steps::{
    next_struct_field_cursor, next_struct_field_path,
};
use super::global_struct_initializer_designator_paths::parent_struct_name;
use super::global_struct_initializer_designators::{
    global_struct_layout, write_array_index_path_value,
};
use super::global_struct_initializer_struct_array_designators::struct_array_field_info;
use super::{Constant, FieldType, GlobalStructInitializerValue, StructLayout};

#[derive(Clone, Copy)]
pub(super) struct GlobalStructArrayElementArrayTarget<'a> {
    pub(super) array_path: &'a [usize],
    pub(super) element_index: usize,
    pub(super) field_path: &'a [usize],
    pub(super) field_element_index: usize,
}

#[derive(Clone, Copy)]
pub(super) struct GlobalStructArrayElementArrayWrite<'a> {
    pub(super) target: GlobalStructArrayElementArrayTarget<'a>,
    pub(super) value_tokens: &'a [Token],
}

pub(super) struct StructArrayElementArrayWriter<'a> {
    known_structs: &'a [StructLayout],
    constants: &'a [Constant],
}

impl<'a> StructArrayElementArrayWriter<'a> {
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
        write: GlobalStructArrayElementArrayWrite<'_>,
    ) -> CompileResult<()> {
        let Some((field_index, nested_path)) = write.target.array_path.split_first() else {
            return Err(CompileError::new(
                "expected global struct-array element array designator",
            ));
        };
        let layout = global_struct_layout(self.known_structs, struct_name)?;
        if values.len() <= *field_index {
            values.resize(*field_index + 1, GlobalStructInitializerValue::Integer(0));
        }
        if nested_path.is_empty() {
            return self.write_element_array_value(values, layout, *field_index, write);
        }
        let Some(FieldType::Struct(nested_struct_name)) = layout
            .fields
            .get(*field_index)
            .map(|field| &field.field_type)
        else {
            return Err(CompileError::new(
                "global struct-array element array designator requires struct path",
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
                "global struct-array element array designator requires nested field value",
            ));
        };
        self.write_field_path_value(
            nested_values,
            nested_struct_name,
            GlobalStructArrayElementArrayWrite {
                target: GlobalStructArrayElementArrayTarget {
                    array_path: nested_path,
                    ..write.target
                },
                value_tokens: write.value_tokens,
            },
        )
    }

    fn write_element_array_value(
        &self,
        values: &mut [GlobalStructInitializerValue],
        layout: &StructLayout,
        field_index: usize,
        write: GlobalStructArrayElementArrayWrite<'_>,
    ) -> CompileResult<()> {
        let Some(FieldType::StructArray { struct_name, .. }) = layout
            .fields
            .get(field_index)
            .map(|field| &field.field_type)
        else {
            return Err(CompileError::new(
                "global struct-array element array designator requires struct-array field",
            ));
        };
        if !matches!(values[field_index], GlobalStructInitializerValue::Nested(_)) {
            values[field_index] = GlobalStructInitializerValue::Nested(Vec::new());
        }
        let GlobalStructInitializerValue::Nested(elements) = &mut values[field_index] else {
            return Err(CompileError::new(
                "global struct-array element array designator requires nested field value",
            ));
        };
        if elements.len() <= write.target.element_index {
            elements.resize_with(write.target.element_index + 1, || {
                GlobalStructInitializerValue::Nested(Vec::new())
            });
        }
        if !matches!(
            elements[write.target.element_index],
            GlobalStructInitializerValue::Nested(_)
        ) {
            elements[write.target.element_index] = GlobalStructInitializerValue::Nested(Vec::new());
        }
        let GlobalStructInitializerValue::Nested(element_values) =
            &mut elements[write.target.element_index]
        else {
            return Err(CompileError::new(
                "global struct-array element array designator requires nested element value",
            ));
        };
        write_array_index_path_value(
            element_values,
            self.known_structs,
            struct_name,
            write.target.field_path,
            write.target.field_element_index,
            write.value_tokens,
            self.constants,
        )
    }
}

pub(super) fn next_struct_array_element_array_field_cursor(
    known_structs: &[StructLayout],
    struct_name: &str,
    target: GlobalStructArrayElementArrayTarget<'_>,
) -> CompileResult<GlobalStructDesignatorWrite> {
    let (element_struct_name, length) =
        struct_array_field_info(known_structs, struct_name, target.array_path)?;
    let field_length = struct_array_element_array_field_len(
        known_structs,
        &element_struct_name,
        target.field_path,
    )?;
    let next_field_element = target
        .field_element_index
        .checked_add(1)
        .ok_or_else(|| CompileError::new("global struct-array nested array index overflow"))?;
    if next_field_element < field_length {
        return Ok(GlobalStructDesignatorWrite {
            next_index: target.array_path[0] + 1,
            cursor: Some(GlobalStructDesignatorCursor::StructArrayArrayFieldPath {
                array_path: target.array_path.to_vec(),
                element_index: target.element_index,
                field_path: target.field_path.to_vec(),
                field_element_index: next_field_element,
            }),
        });
    }
    if let Some(next_path) =
        next_struct_field_path(known_structs, &element_struct_name, target.field_path)?
    {
        return Ok(GlobalStructDesignatorWrite {
            next_index: target.array_path[0] + 1,
            cursor: Some(GlobalStructDesignatorCursor::StructArrayFieldPath {
                array_path: target.array_path.to_vec(),
                element_index: target.element_index,
                field_path: next_path,
            }),
        });
    }
    let next_element = target
        .element_index
        .checked_add(1)
        .ok_or_else(|| CompileError::new("global struct-array designator index overflow"))?;
    if next_element < length {
        return Ok(GlobalStructDesignatorWrite {
            next_index: target.array_path[0] + 1,
            cursor: Some(GlobalStructDesignatorCursor::StructArrayFieldPath {
                array_path: target.array_path.to_vec(),
                element_index: next_element,
                field_path: vec![0],
            }),
        });
    }
    next_struct_field_cursor(known_structs, struct_name, target.array_path)
}

fn struct_array_element_array_field_len(
    known_structs: &[StructLayout],
    struct_name: &str,
    field_path: &[usize],
) -> CompileResult<usize> {
    let Some(field_index) = field_path.last().copied() else {
        return Err(CompileError::new(
            "expected global struct-array element array designator field path",
        ));
    };
    let parent_struct_name = parent_struct_name(known_structs, struct_name, field_path)?;
    let parent_layout = global_struct_layout(known_structs, &parent_struct_name)?;
    let Some(FieldType::Array { length, .. }) = parent_layout
        .fields
        .get(field_index)
        .map(|field| &field.field_type)
    else {
        return Err(CompileError::new(
            "global struct-array element array designator requires array field",
        ));
    };
    Ok(*length)
}
