use crate::front_end::lexer::Token;

use super::parser_local_struct_array_element_array_designators::{
    LocalStructArrayElementArrayTarget, LocalStructArrayElementArrayWrite,
};
use super::parser_local_struct_designator_cursor::LocalStructDesignatorWrite;
use super::parser_struct_array_element_designators::StructArrayElementDesignatorTarget;
use super::{CompileResult, LocalStructInitializerValue, Parser, StructLayout, struct_field_index};

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
        match designator.target {
            StructArrayElementDesignatorTarget::FieldPath(field_names) => {
                let field_path =
                    self.local_struct_field_path_from_names(&element_struct_name, &field_names)?;
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
            StructArrayElementDesignatorTarget::ArrayField {
                field_path: field_names,
                element_index,
            } => {
                let field_path =
                    self.local_struct_field_path_from_names(&element_struct_name, &field_names)?;
                let target = LocalStructArrayElementArrayTarget {
                    array_path: &array_path,
                    element_index: designator.element_index,
                    field_path: &field_path,
                    field_element_index: element_index,
                };
                self.write_local_struct_array_element_array_field_path_value(
                    struct_name,
                    values,
                    LocalStructArrayElementArrayWrite {
                        target,
                        value_tokens: designator.value_tokens,
                    },
                )?;
                self.next_local_struct_array_element_array_field_cursor(struct_name, target)
                    .map(Some)
            }
        }
    }
}
