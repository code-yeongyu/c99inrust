use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token, TokenKind};

mod aggregate_declarations;
mod anonymous_union;
mod builtin_field;
mod builtin_layout;
mod declaration_referents;
mod declaration_type_flags;
mod declarator_types;
mod doom_layout;
mod enum_declarations;
mod external_declarations;
mod function_pointer_declarators;
mod function_pointer_typedefs;
mod global_array_compound_literals;
mod global_bool_declarations;
mod global_byte_declarations;
mod global_byte_initializers;
mod global_declarations;
mod global_double_declarations;
mod global_floatlike_declarations;
mod global_int_arrays;
mod global_int_initializers;
mod global_member_addresses;
mod global_name_pointer_array_initializers;
mod global_pointer_arrays;
mod global_pointer_compound_array_literals;
mod global_pointer_compound_literals;
mod global_pointer_name_arrays;
mod global_pointer_scalars;
mod global_pointer_string_arrays;
mod global_scalar_compound_literals;
mod global_scalar_declarations;
mod global_short_declarations;
mod global_sizeof;
mod global_specifiers;
mod global_string_initializers;
mod global_string_pointer_arrays;
mod global_struct_arrays;
mod global_struct_initializer;
mod global_struct_initializer_addresses;
mod global_struct_initializer_designator_cursor;
mod global_struct_initializer_designator_cursor_steps;
mod global_struct_initializer_designator_paths;
mod global_struct_initializer_designators;
mod global_struct_initializer_struct_array_array_designators;
mod global_struct_initializer_struct_array_designators;
mod global_struct_initializer_struct_array_dispatch;
mod global_struct_initializer_struct_array_paths;
mod global_struct_object;
mod initializer_number;
mod integer_initializer;
mod local_arrays;
mod local_scalar_initializer;
mod model;
mod parser_assignment_ops;
mod parser_compound_literals;
mod parser_control_flow;
mod parser_declaration_statement;
mod parser_declaration_types;
mod parser_designated_initializers;
mod parser_expression_core;
mod parser_expression_helpers;
mod parser_expression_postfix;
mod parser_float_literals;
mod parser_functions;
mod parser_local_arrays;
mod parser_local_char_matrices;
mod parser_local_declarations;
mod parser_local_function_pointers;
mod parser_local_int_matrices;
mod parser_local_pointer_arrays;
mod parser_local_scalar_arrays;
mod parser_local_struct_array_element_array_designators;
mod parser_local_struct_array_element_designators;
mod parser_local_struct_array_element_dispatch;
mod parser_local_struct_designator_cursor;
mod parser_local_struct_designator_cursor_steps;
mod parser_local_struct_designator_paths;
mod parser_local_struct_designators;
mod parser_local_struct_initializers;
mod parser_local_structs;
mod parser_local_vlas;
mod parser_sizeof_type;
mod parser_statement_dispatch;
mod parser_struct_array_element_designators;
mod parser_token_stream;
mod parser_va_arg;
mod program;
mod scalar_layout;
mod struct_anonymous_fields;
mod struct_bitfields;
mod struct_fields;
mod struct_layout_helpers;
mod supported_typedefs;
mod surface_parser;
mod surface_types;
mod syntax;
mod token_scan;
mod translation_unit_parser;
mod type_recognition;
mod typedef_referent;

use aggregate_declarations::{
    aggregate_tag_name, parse_struct_definition, parse_struct_typedef, struct_alias_layouts,
    struct_forward_typedef_alias,
};
use anonymous_union::anonymous_union_struct_name;
use builtin_layout::builtin_struct_layouts;
use declaration_referents::{
    declaration_base_referent_type, pointer_referent_for_depth, pointer_referent_from_specifiers,
};
use declarator_types::{parameter_scalar_type, pointer_referent_type};
use enum_declarations::{enum_typedef_name, parse_enum_constants};
use external_declarations::{
    function_definition_has_supported_signature, function_definition_name,
    function_pointer_cast_type, function_pointer_name, function_pointer_typedef,
    function_prototype, pointer_return_function, top_level_function_open_paren,
};
use function_pointer_declarators::{
    function_pointer_variable, function_referent_for_return, function_referent_for_scalar,
    pointer_return_declarator,
};
use global_declarations::{
    parse_supported_global_declaration, parse_supported_global_declarations,
    unsupported_data_declaration_blocks_empty_unit,
};
use global_sizeof::global_sizeof_symbols;
use initializer_number::InitializerNumber;
use integer_initializer::{
    eval_integer_initializer_expr_with_constants, parse_integer_initializer_with_context,
};
use local_arrays::{
    inferred_local_char_array_length, local_array_length, validate_local_char_array_initializer,
    validate_local_char_array_initializer_size,
};
use local_scalar_initializer::local_scalar_initializer;
pub use model::{
    Constant, FieldType, Global, GlobalInitializer, GlobalPointerAddress,
    GlobalStructInitializerAddress, GlobalStructInitializerValue, PointerReturnFunction,
    ReturnType, ScalarFieldType, ScalarType, StructField, StructLayout,
};
use parser_designated_initializers::{
    struct_field_designator, struct_field_index, struct_field_path_designator, zero_expr,
};
use parser_expression_helpers::{lvalue_from_expr, prefix_update_expr, statement_from_expression};
use parser_local_struct_initializers::{field_type_at, resize_values_for_index};
use parser_local_vlas::LocalVlaElement;
pub use program::{Function, FunctionPrototype, Parameter, Program};
use supported_typedefs::supported_typedef_scalar;
use surface_parser::SurfaceParser;
pub use surface_types::{ExternalItem, SurfaceTranslationUnit};
pub use syntax::{
    BinaryOp, Expr, LValue, LocalCharArrayInitializer, LocalStructInitializer,
    LocalStructInitializerValue, Statement, SwitchCase, UnaryOp,
};
use token_scan::{
    matching_top_level_brace, matching_top_level_bracket, matching_top_level_paren,
    parameter_is_variadic, parameter_is_void, previous_identifier_index, token_identifier,
    token_is_assignment_operator, token_is_keyword, token_is_punctuator,
    top_level_punctuator_index,
};
pub use translation_unit_parser::parse_supported_translation_unit;
use type_recognition::{sizeof_type, supported_cast_type_with_typedefs, supported_return_type};

const DOOM_EXPAND_PIXEL_UNION: &str = "__doom_expand_pixel_union";
const DOOM_NAME8_UNION: &str = "__doom_name8_union";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentOperator {
    Simple,
    Compound(BinaryOp),
}

/// Parses the supported executable function-body subset.
///
/// # Errors
///
/// Returns an error when the token stream does not match the currently
/// supported C subset.
pub fn parse(tokens: &[Token]) -> CompileResult<Program> {
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs: &[],
        known_constants: &[],
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
        known_function_pointer_typedefs: &[],
    };
    parser.program()
}

/// Surface-parses a translation unit for Doom-facing frontend audits.
///
/// # Errors
///
/// Returns an error when external declarations or function-definition
/// boundaries are structurally unbalanced.
pub fn parse_translation_unit(tokens: &[Token]) -> CompileResult<SurfaceTranslationUnit> {
    let mut parser = SurfaceParser::new(tokens);
    parser.translation_unit()
}

struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
    known_structs: &'a [StructLayout],
    known_constants: &'a [Constant],
    known_scalar_typedefs: &'a [String],
    known_pointer_typedefs: &'a [String],
    known_function_pointer_typedefs: &'a [(String, String)],
}
