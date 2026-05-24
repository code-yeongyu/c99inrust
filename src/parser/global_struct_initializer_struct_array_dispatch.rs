use crate::diagnostics::CompileResult;
use crate::front_end::lexer::Token;

use super::global_struct_initializer_designator_cursor::GlobalStructDesignatorWrite;
use super::global_struct_initializer_designator_paths::global_struct_field_index_path;
use super::global_struct_initializer_struct_array_array_designators::{
    GlobalStructArrayElementArrayTarget, GlobalStructArrayElementArrayWrite,
    StructArrayElementArrayWriter, next_struct_array_element_array_field_cursor,
};
use super::global_struct_initializer_struct_array_designators::{
    StructArrayElementWriter, next_struct_array_element_field_cursor, struct_array_field_info,
    struct_field_path_from_names,
};
use super::parser_struct_array_element_designators::StructArrayElementDesignatorTarget;
use super::{Constant, GlobalStructInitializerValue, Parser, StructLayout};

pub(super) fn write_struct_array_element_designator(
    values: &mut Vec<GlobalStructInitializerValue>,
    designator_parser: &Parser<'_>,
    known_structs: &[StructLayout],
    struct_name: &str,
    item: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<GlobalStructDesignatorWrite>> {
    let Some(designator) = designator_parser.struct_array_element_field_designator(item)? else {
        return Ok(None);
    };
    let array_path = global_struct_field_index_path(
        known_structs,
        struct_name,
        designator.array_path[0],
        &designator.array_path[1..],
    )?;
    let (element_struct_name, _) =
        struct_array_field_info(known_structs, struct_name, &array_path)?;
    match designator.target {
        StructArrayElementDesignatorTarget::FieldPath(field_names) => {
            let field_path =
                struct_field_path_from_names(known_structs, &element_struct_name, &field_names)?;
            let writer = StructArrayElementWriter::new(known_structs, constants);
            writer.write_field_path_value(
                values,
                struct_name,
                &array_path,
                designator.element_index,
                &field_path,
                designator.value_tokens,
            )?;
            next_struct_array_element_field_cursor(
                known_structs,
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
                struct_field_path_from_names(known_structs, &element_struct_name, &field_names)?;
            let target = GlobalStructArrayElementArrayTarget {
                array_path: &array_path,
                element_index: designator.element_index,
                field_path: &field_path,
                field_element_index: element_index,
            };
            let writer = StructArrayElementArrayWriter::new(known_structs, constants);
            writer.write_field_path_value(
                values,
                struct_name,
                GlobalStructArrayElementArrayWrite {
                    target,
                    value_tokens: designator.value_tokens,
                },
            )?;
            next_struct_array_element_array_field_cursor(known_structs, struct_name, target)
                .map(Some)
        }
    }
}
