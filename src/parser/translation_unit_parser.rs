use super::{
    CompileError, CompileResult, Constant, Function, Parser, Program, StructLayout, SurfaceParser,
    Token, aggregate_tag_name, builtin_struct_layouts, enum_typedef_name,
    function_definition_has_supported_signature, function_definition_name,
    function_pointer_typedef_name, function_prototype, global_sizeof_symbols, parse_enum_constants,
    parse_struct_definition, parse_struct_typedef, parse_supported_global_declarations,
    pointer_return_function, struct_alias_layouts, struct_forward_typedef_alias,
    unsupported_data_declaration_blocks_empty_unit,
};

/// Parses supported executable functions from a full translation unit.
///
/// # Errors
///
/// Returns an error when the translation unit contains a function definition
/// outside the supported executable subset.
pub fn parse_supported_translation_unit(tokens: &[Token]) -> CompileResult<Program> {
    let mut parser = SurfaceParser::new(tokens);
    let external_items = parser.external_token_groups()?;
    let mut structs = builtin_struct_layouts();
    let mut struct_aliases = Vec::new();
    let mut constants = Vec::new();
    let mut scalar_typedefs = Vec::new();
    let mut pointer_typedefs = Vec::new();
    let mut globals = Vec::new();
    let mut pointer_return_functions = Vec::new();
    let mut function_prototypes = Vec::new();
    let mut functions = Vec::new();
    let mut unsupported_data_declaration = false;
    for item_tokens in &external_items {
        if let Some(name) = function_pointer_typedef_name(item_tokens) {
            pointer_typedefs.push(name);
            continue;
        }
        if let Some(alias) = struct_forward_typedef_alias(item_tokens) {
            struct_aliases.push(alias);
            continue;
        }
        if let Some(function) = pointer_return_function(item_tokens) {
            pointer_return_functions.push(function);
        }
        if let Some(prototype) = function_prototype(item_tokens) {
            function_prototypes.push(prototype);
            continue;
        }
        if let Some(layouts) =
            parse_struct_typedef(item_tokens, &structs, &constants, &pointer_typedefs)?
        {
            register_struct_typedef_layouts(item_tokens, layouts, &mut structs, &struct_aliases);
            continue;
        }
        if let Some(layouts) =
            parse_struct_definition(item_tokens, &structs, &constants, &pointer_typedefs)?
        {
            register_struct_layouts(layouts, &mut structs, &struct_aliases);
            continue;
        }
        let enum_constants = parse_enum_constants(item_tokens, &constants)?;
        if !enum_constants.is_empty() {
            if let Some(name) = enum_typedef_name(item_tokens) {
                scalar_typedefs.push(name);
            }
            constants.extend(enum_constants);
            continue;
        }
        let sizeof_symbols = global_sizeof_symbols(&globals, &structs)?;
        let parsed_globals = parse_supported_global_declarations(
            item_tokens,
            &structs,
            &constants,
            &sizeof_symbols,
        )?;
        if !parsed_globals.is_empty() {
            globals.extend(parsed_globals);
            continue;
        }
        let Some(name) = function_definition_name(item_tokens) else {
            if unsupported_data_declaration_blocks_empty_unit(item_tokens) {
                unsupported_data_declaration = true;
            }
            continue;
        };
        functions.push(parse_supported_function_definition(
            item_tokens,
            &name,
            &structs,
            &constants,
            &scalar_typedefs,
            &pointer_typedefs,
        )?);
    }
    if functions.is_empty() && unsupported_data_declaration {
        return Err(CompileError::new(
            "translation unit has no supported function definitions",
        ));
    }
    Ok(Program {
        structs,
        constants,
        globals,
        pointer_return_functions,
        function_prototypes,
        functions,
    })
}

fn register_struct_typedef_layouts(
    item_tokens: &[Token],
    layouts: Vec<StructLayout>,
    structs: &mut Vec<StructLayout>,
    struct_aliases: &[(String, String)],
) {
    let Some(layout) = layouts.last() else {
        return;
    };
    if let Some(tag_name) = aggregate_tag_name(item_tokens)
        && tag_name != layout.name
    {
        let tag_layout = StructLayout {
            name: tag_name,
            fields: layout.fields.clone(),
            size: layout.size,
        };
        structs.extend(struct_alias_layouts(&tag_layout, struct_aliases));
        structs.push(tag_layout);
    }
    register_struct_layouts(layouts, structs, struct_aliases);
}

fn register_struct_layouts(
    layouts: Vec<StructLayout>,
    structs: &mut Vec<StructLayout>,
    struct_aliases: &[(String, String)],
) {
    for layout in &layouts {
        structs.extend(struct_alias_layouts(layout, struct_aliases));
    }
    structs.extend(layouts);
}

fn parse_supported_function_definition(
    tokens: &[Token],
    name: &str,
    structs: &[StructLayout],
    constants: &[Constant],
    scalar_typedefs: &[String],
    pointer_typedefs: &[String],
) -> CompileResult<Function> {
    if !function_definition_has_supported_signature(tokens) {
        let Some(token) = tokens.first() else {
            return Err(CompileError::new(format!(
                "unsupported function definition: {name}"
            )));
        };
        return Err(
            CompileError::new(format!("unsupported function definition: {name}"))
                .at(token.line, token.column),
        );
    }
    let mut function_parser = Parser {
        tokens,
        index: 0,
        known_structs: structs,
        known_constants: constants,
        known_scalar_typedefs: scalar_typedefs,
        known_pointer_typedefs: pointer_typedefs,
    };
    let function = function_parser.function()?;
    if !function_parser.check_end() {
        return Err(CompileError::new(format!(
            "trailing tokens after function definition: {name}"
        )));
    }
    Ok(function)
}
