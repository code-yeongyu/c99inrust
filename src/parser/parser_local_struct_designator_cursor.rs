use crate::front_end::lexer::Token;

use super::parser_local_struct_array_element_array_designators::{
    LocalStructArrayElementArrayTarget, LocalStructArrayElementArrayWrite,
};
use super::parser_local_struct_designator_cursor_steps::{
    next_local_array_field_cursor, next_local_struct_array_path_cursor,
    next_local_struct_field_cursor,
};
use super::{
    CompileResult, LocalStructInitializerValue, Parser, StructLayout, resize_values_for_index,
    struct_field_index, struct_field_path_designator,
};

pub(super) struct LocalStructDesignatorWrite {
    pub(super) next_index: usize,
    pub(super) cursor: Option<LocalStructDesignatorCursor>,
}

pub(super) enum LocalStructDesignatorCursor {
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

impl Parser<'_> {
    pub(super) fn write_local_struct_designator_item(
        &self,
        layout: &StructLayout,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        item: &[Token],
    ) -> CompileResult<Option<LocalStructDesignatorWrite>> {
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
            return next_local_array_field_cursor(layout, index, element_index).map(Some);
        }
        if let Some(write) = self.write_local_struct_array_element_designator_item(
            layout,
            struct_name,
            values,
            item,
        )? {
            return Ok(Some(write));
        }
        if let Some((field_path, element_index, value_tokens)) =
            self.struct_array_field_path_designator(item)?
        {
            let index = struct_field_index(self.known_structs, struct_name, field_path[0])?;
            let index_path = self.local_struct_field_index_path(layout, index, &field_path[1..])?;
            self.write_local_struct_array_index_path_value(
                struct_name,
                values,
                &index_path,
                element_index,
                value_tokens,
            )?;
            return next_local_struct_array_path_cursor(
                self,
                struct_name,
                &index_path,
                element_index,
            )
            .map(Some);
        }
        if let Some((field_path, value_tokens)) = struct_field_path_designator(item)? {
            let index = struct_field_index(self.known_structs, struct_name, field_path[0])?;
            let index_path = self.local_struct_field_index_path(layout, index, &field_path[1..])?;
            self.write_local_struct_index_path_value(
                struct_name,
                values,
                &index_path,
                value_tokens,
            )?;
            return next_local_struct_field_cursor(self, struct_name, &index_path).map(Some);
        }
        Ok(None)
    }

    pub(super) fn write_local_struct_cursor_value(
        &self,
        layout: &StructLayout,
        struct_name: &str,
        values: &mut Vec<LocalStructInitializerValue>,
        cursor: LocalStructDesignatorCursor,
        value_tokens: &[Token],
    ) -> CompileResult<LocalStructDesignatorWrite> {
        match cursor {
            LocalStructDesignatorCursor::ArrayField {
                field_index,
                element_index,
            } => {
                resize_values_for_index(values, layout, field_index);
                self.write_local_array_field_designator(
                    layout,
                    values,
                    field_index,
                    element_index,
                    value_tokens,
                )?;
                next_local_array_field_cursor(layout, field_index, element_index)
            }
            LocalStructDesignatorCursor::ArrayPath {
                index_path,
                element_index,
            } => {
                self.write_local_struct_array_index_path_value(
                    struct_name,
                    values,
                    &index_path,
                    element_index,
                    value_tokens,
                )?;
                next_local_struct_array_path_cursor(self, struct_name, &index_path, element_index)
            }
            LocalStructDesignatorCursor::StructArrayFieldPath {
                array_path,
                element_index,
                field_path,
            } => self.write_local_struct_array_element_cursor_value(
                struct_name,
                values,
                &array_path,
                element_index,
                &field_path,
                value_tokens,
            ),
            LocalStructDesignatorCursor::StructArrayArrayFieldPath {
                array_path,
                element_index,
                field_path,
                field_element_index,
            } => {
                let target = LocalStructArrayElementArrayTarget {
                    array_path: &array_path,
                    element_index,
                    field_path: &field_path,
                    field_element_index,
                };
                self.write_local_struct_array_element_array_cursor_value(
                    struct_name,
                    values,
                    LocalStructArrayElementArrayWrite {
                        target,
                        value_tokens,
                    },
                )
            }
            LocalStructDesignatorCursor::FieldPath(index_path) => {
                self.write_local_struct_index_path_value(
                    struct_name,
                    values,
                    &index_path,
                    value_tokens,
                )?;
                next_local_struct_field_cursor(self, struct_name, &index_path)
            }
        }
    }
}
