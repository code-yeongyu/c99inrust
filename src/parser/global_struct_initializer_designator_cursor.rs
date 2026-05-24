use crate::diagnostics::CompileResult;
use crate::front_end::lexer::Token;

use super::global_struct_initializer_designator_cursor_steps::{
    next_array_field_cursor, next_struct_array_path_cursor, next_struct_field_cursor,
};
use super::global_struct_initializer_designator_paths::global_struct_field_index_path;
use super::global_struct_initializer_designators::{
    write_array_field_designator, write_array_index_path_value, write_index_path_value,
};
use super::global_struct_initializer_struct_array_array_designators::{
    GlobalStructArrayElementArrayTarget, GlobalStructArrayElementArrayWrite,
    StructArrayElementArrayWriter, next_struct_array_element_array_field_cursor,
};
use super::global_struct_initializer_struct_array_designators::{
    StructArrayElementWriter, next_struct_array_element_field_cursor,
};
use super::global_struct_initializer_struct_array_dispatch::write_struct_array_element_designator;
use super::{
    Constant, GlobalStructInitializerValue, Parser, StructLayout, struct_field_index,
    struct_field_path_designator,
};

pub(super) struct GlobalStructDesignatorWrite {
    pub(super) next_index: usize,
    pub(super) cursor: Option<GlobalStructDesignatorCursor>,
}

pub(super) enum GlobalStructDesignatorCursor {
    ArrayField {
        field_index: usize,
        element_index: usize,
    },
    ArrayPath {
        index_path: Vec<usize>,
        element_index: usize,
    },
    StructArrayFieldPath {
        array_path: Vec<usize>,
        element_index: usize,
        field_path: Vec<usize>,
    },
    StructArrayArrayFieldPath {
        array_path: Vec<usize>,
        element_index: usize,
        field_path: Vec<usize>,
        field_element_index: usize,
    },
    FieldPath(Vec<usize>),
}

pub(super) fn write_designator(
    values: &mut Vec<GlobalStructInitializerValue>,
    designator_parser: &Parser<'_>,
    known_structs: &[StructLayout],
    struct_name: &str,
    item: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<GlobalStructDesignatorWrite>> {
    if let Some((field_name, element_index, value_tokens)) =
        designator_parser.struct_array_field_designator(item)?
    {
        let index = struct_field_index(known_structs, struct_name, field_name)?;
        write_array_field_designator(
            values,
            known_structs,
            struct_name,
            index,
            element_index,
            value_tokens,
            constants,
        )?;
        return next_array_field_cursor(known_structs, struct_name, index, element_index).map(Some);
    }
    if let Some(write) = write_struct_array_element_designator(
        values,
        designator_parser,
        known_structs,
        struct_name,
        item,
        constants,
    )? {
        return Ok(Some(write));
    }
    if let Some((field_path, element_index, value_tokens)) =
        designator_parser.struct_array_field_path_designator(item)?
    {
        let index_path = global_struct_field_index_path(
            known_structs,
            struct_name,
            field_path[0],
            &field_path[1..],
        )?;
        write_array_index_path_value(
            values,
            known_structs,
            struct_name,
            &index_path,
            element_index,
            value_tokens,
            constants,
        )?;
        return next_struct_array_path_cursor(
            known_structs,
            struct_name,
            &index_path,
            element_index,
        )
        .map(Some);
    }
    if let Some((field_path, value_tokens)) = struct_field_path_designator(item)? {
        let index_path = global_struct_field_index_path(
            known_structs,
            struct_name,
            field_path[0],
            &field_path[1..],
        )?;
        write_index_path_value(
            values,
            known_structs,
            struct_name,
            &index_path,
            value_tokens,
            constants,
        )?;
        return next_struct_field_cursor(known_structs, struct_name, &index_path).map(Some);
    }
    Ok(None)
}

pub(super) fn write_cursor_value(
    values: &mut Vec<GlobalStructInitializerValue>,
    known_structs: &[StructLayout],
    struct_name: &str,
    cursor: GlobalStructDesignatorCursor,
    value_tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<GlobalStructDesignatorWrite> {
    match cursor {
        GlobalStructDesignatorCursor::ArrayField {
            field_index,
            element_index,
        } => {
            write_array_field_designator(
                values,
                known_structs,
                struct_name,
                field_index,
                element_index,
                value_tokens,
                constants,
            )?;
            next_array_field_cursor(known_structs, struct_name, field_index, element_index)
        }
        GlobalStructDesignatorCursor::ArrayPath {
            index_path,
            element_index,
        } => {
            write_array_index_path_value(
                values,
                known_structs,
                struct_name,
                &index_path,
                element_index,
                value_tokens,
                constants,
            )?;
            next_struct_array_path_cursor(known_structs, struct_name, &index_path, element_index)
        }
        GlobalStructDesignatorCursor::StructArrayFieldPath {
            array_path,
            element_index,
            field_path,
        } => {
            let writer = StructArrayElementWriter::new(known_structs, constants);
            writer.write_field_path_value(
                values,
                struct_name,
                &array_path,
                element_index,
                &field_path,
                value_tokens,
            )?;
            next_struct_array_element_field_cursor(
                known_structs,
                struct_name,
                &array_path,
                element_index,
                &field_path,
            )
        }
        GlobalStructDesignatorCursor::StructArrayArrayFieldPath {
            array_path,
            element_index,
            field_path,
            field_element_index,
        } => {
            let target = GlobalStructArrayElementArrayTarget {
                array_path: &array_path,
                element_index,
                field_path: &field_path,
                field_element_index,
            };
            let writer = StructArrayElementArrayWriter::new(known_structs, constants);
            writer.write_field_path_value(
                values,
                struct_name,
                GlobalStructArrayElementArrayWrite {
                    target,
                    value_tokens,
                },
            )?;
            next_struct_array_element_array_field_cursor(known_structs, struct_name, target)
        }
        GlobalStructDesignatorCursor::FieldPath(index_path) => {
            write_index_path_value(
                values,
                known_structs,
                struct_name,
                &index_path,
                value_tokens,
                constants,
            )?;
            next_struct_field_cursor(known_structs, struct_name, &index_path)
        }
    }
}
