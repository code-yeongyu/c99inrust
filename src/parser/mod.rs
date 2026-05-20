use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token, TokenKind};

mod aggregate_declarations;
mod anonymous_union;
mod builtin_field;
mod builtin_layout;
mod declarator_types;
mod doom_layout;
mod enum_declarations;
mod external_declarations;
mod global_sizeof;
mod global_struct_initializer;
mod global_struct_object;
mod local_arrays;
mod model;
mod program;
mod scalar_layout;
mod struct_fields;
mod struct_layout_helpers;
mod surface_parser;
mod surface_types;
mod syntax;
mod token_scan;
mod type_recognition;
mod typedef_referent;

use aggregate_declarations::{
    aggregate_tag_name, parse_struct_definition, parse_struct_typedef, struct_alias_layouts,
    struct_forward_typedef_alias,
};
use anonymous_union::anonymous_union_struct_name;
use builtin_layout::builtin_struct_layouts;
use declarator_types::{
    declaration_base_referent_type, parameter_scalar_type, pointer_referent_for_depth,
    pointer_referent_from_specifiers, pointer_referent_type,
};
use enum_declarations::{enum_typedef_name, parse_enum_constants};
use external_declarations::{
    classify_external_item, function_definition_has_supported_signature, function_definition_name,
    function_pointer_cast_type, function_pointer_name, function_pointer_typedef_name,
    function_prototype_name, pointer_return_function, top_level_function_open_paren,
};
use global_sizeof::global_sizeof_symbols;
use local_arrays::{
    inferred_local_char_array_length, local_array_length, validate_local_char_array_initializer,
    validate_local_char_array_initializer_size,
};
pub use model::{
    Constant, FieldType, Global, GlobalInitializer, GlobalStructInitializerAddress,
    GlobalStructInitializerValue, PointerReturnFunction, ReturnType, ScalarFieldType, ScalarType,
    StructField, StructLayout,
};
pub use program::{Function, Parameter, Program};
use surface_parser::SurfaceParser;
pub use surface_types::{ExternalItem, SurfaceTranslationUnit};
pub use syntax::{
    BinaryOp, Expr, LValue, LocalCharArrayInitializer, Statement, SwitchCase, UnaryOp,
};
use token_scan::{
    last_token_is_punctuator, matching_top_level_brace, matching_top_level_bracket,
    matching_top_level_paren, parameter_is_variadic, parameter_is_void, previous_identifier_index,
    token_has_keyword, token_identifier, token_is_assignment_operator, token_is_keyword,
    token_is_punctuator, top_level_comma_ranges, top_level_punctuator_index,
};
use type_recognition::{
    sizeof_type, supported_cast_type_with_typedefs, supported_return_type, supported_typedef_scalar,
};

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
        if let Some(name) = function_prototype_name(item_tokens) {
            function_prototypes.push(name);
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

struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
    known_structs: &'a [StructLayout],
    known_constants: &'a [Constant],
    known_scalar_typedefs: &'a [String],
    known_pointer_typedefs: &'a [String],
}

impl Parser<'_> {
    fn program(&mut self) -> CompileResult<Program> {
        let mut functions = Vec::new();
        while !self.check_end() {
            functions.push(self.function()?);
        }
        Ok(Program {
            structs: Vec::new(),
            constants: Vec::new(),
            globals: Vec::new(),
            pointer_return_functions: Vec::new(),
            function_prototypes: Vec::new(),
            functions,
        })
    }

    fn function(&mut self) -> CompileResult<Function> {
        let (return_type, name) = self.function_signature()?;
        let (parameters, is_variadic) = self.parameter_list()?;
        self.expect_punctuator("{")?;
        let mut statements = Vec::new();
        while !self.check_punctuator("}") {
            statements.push(self.statement()?);
        }
        self.expect_punctuator("}")?;
        Ok(Function {
            name,
            return_type,
            parameters,
            is_variadic,
            statements,
        })
    }

    fn function_signature(&mut self) -> CompileResult<(ReturnType, String)> {
        let tokens = &self.tokens[self.index..];
        let Some(open_index) = top_level_function_open_paren(tokens) else {
            return self.expected("function parameter list");
        };
        let Some(name_index) = previous_identifier_index(tokens, open_index) else {
            return self.expected("function name");
        };
        let return_type = supported_return_type(&tokens[..name_index])
            .ok_or_else(|| CompileError::new("unsupported function return type"))?;
        let name = token_identifier(&tokens[name_index])
            .ok_or_else(|| CompileError::new("expected function name"))?
            .to_owned();
        self.index += open_index + 1;
        Ok((return_type, name))
    }

    fn parameter_list(&mut self) -> CompileResult<(Vec<Parameter>, bool)> {
        let mut parameters = Vec::new();
        let mut is_variadic = false;
        let mut parameter_start = self.index;
        let mut depth = 0usize;
        while !self.check_end() {
            if self.check_punctuator("(") {
                depth += 1;
                self.advance();
                continue;
            }
            if self.check_punctuator(")") {
                if depth == 0 {
                    if parameter_start == self.index {
                        self.advance();
                        return Ok((parameters, is_variadic));
                    }
                    is_variadic |=
                        self.push_parameter(&mut parameters, parameter_start, self.index)?;
                    self.advance();
                    return Ok((parameters, is_variadic));
                }
                depth -= 1;
                self.advance();
                continue;
            }
            if depth == 0 && self.check_punctuator(",") {
                is_variadic |= self.push_parameter(&mut parameters, parameter_start, self.index)?;
                self.advance();
                parameter_start = self.index;
                continue;
            }
            self.advance();
        }
        Err(CompileError::new("unterminated function parameter list"))
    }

    fn push_parameter(
        &self,
        parameters: &mut Vec<Parameter>,
        start: usize,
        end: usize,
    ) -> CompileResult<bool> {
        let tokens = &self.tokens[start..end];
        if parameter_is_void(tokens) {
            return Ok(false);
        }
        if parameter_is_variadic(tokens) {
            return Ok(true);
        }
        if let Some(name) = function_pointer_name(tokens) {
            parameters.push(Parameter {
                name,
                scalar_type: ScalarType::Pointer,
                referent: None,
            });
            return Ok(false);
        }
        let Some(name) = tokens.iter().rev().find_map(token_identifier) else {
            return Err(CompileError::new("unsupported function parameter"));
        };
        let scalar_type = parameter_scalar_type(
            tokens,
            self.known_scalar_typedefs,
            self.known_pointer_typedefs,
        )
        .ok_or_else(|| CompileError::new("unsupported function parameter"))?;
        parameters.push(Parameter {
            name: name.to_owned(),
            scalar_type,
            referent: pointer_referent_type(tokens),
        });
        Ok(false)
    }

    fn statement(&mut self) -> CompileResult<Statement> {
        if self.check_punctuator(";") {
            self.advance();
            return Ok(Statement::Empty);
        }
        if self.check_punctuator("{") {
            return Ok(Statement::Block(self.block_items()?));
        }
        if let Some(statement) = self.block_extern_declaration()? {
            return Ok(statement);
        }
        if let Some(statement) = self.local_function_pointer_array_declaration()? {
            return Ok(statement);
        }
        if let Some(statement) = self.block_function_prototype_declaration() {
            return Ok(statement);
        }
        if let Some(statement) = self.static_aggregate_declaration()? {
            return Ok(statement);
        }
        if let Some(statement) = self.local_enum_declaration()? {
            return Ok(statement);
        }
        if let Some(statement) = self.local_anonymous_union_declaration()? {
            return Ok(statement);
        }
        if let Some(statement) = self.local_struct_declaration()? {
            return Ok(statement);
        }
        if let Some(scalar_type) = self.declaration_type_at_current() {
            return self.declaration_statement(scalar_type);
        }
        if self.check_keyword(Keyword::If) {
            return self.if_statement();
        }
        if self.check_keyword(Keyword::While) {
            return self.while_statement();
        }
        if self.check_keyword(Keyword::Do) {
            return self.do_while_statement();
        }
        if self.check_keyword(Keyword::For) {
            return self.for_statement();
        }
        if self.check_keyword(Keyword::Switch) {
            return self.switch_statement();
        }
        if self.check_keyword(Keyword::Break) {
            self.advance();
            self.expect_punctuator(";")?;
            return Ok(Statement::Break);
        }
        if self.check_keyword(Keyword::Continue) {
            self.advance();
            self.expect_punctuator(";")?;
            return Ok(Statement::Continue);
        }
        if self.check_keyword(Keyword::Return) {
            self.advance();
            let expr = if self.check_punctuator(";") {
                None
            } else {
                Some(self.expression()?)
            };
            self.expect_punctuator(";")?;
            return Ok(Statement::Return(expr));
        }
        if self.check_keyword(Keyword::Goto) {
            self.advance();
            let label = self.expect_identifier()?;
            self.expect_punctuator(";")?;
            return Ok(Statement::Goto(label));
        }
        if let Some(label) = self.label_statement() {
            return Ok(label);
        }
        if self.current_identifier_starts_assignment() {
            self.assignment_statement(true)
        } else {
            self.expression_statement(true)
        }
    }

    fn label_statement(&mut self) -> Option<Statement> {
        let Some(Token {
            kind: TokenKind::Identifier(name),
            ..
        }) = self.tokens.get(self.index)
        else {
            return None;
        };
        if !self
            .tokens
            .get(self.index + 1)
            .is_some_and(|token| token_is_punctuator(token, ":"))
        {
            return None;
        }
        let label = name.clone();
        self.index += 2;
        Some(Statement::Label(label))
    }

    fn assignment_statement(&mut self, expect_semicolon: bool) -> CompileResult<Statement> {
        let name = self.expect_identifier()?;
        let op = self
            .assignment_operator_at_current()
            .ok_or_else(|| CompileError::new("expected assignment operator"))?;
        self.advance();
        let value = self.assignment()?;
        let value = match op {
            AssignmentOperator::Simple => value,
            AssignmentOperator::Compound(op) => Expr::Binary {
                op,
                left: Box::new(Expr::Identifier(name.clone())),
                right: Box::new(value),
            },
        };
        if expect_semicolon {
            self.expect_punctuator(";")?;
        }
        Ok(Statement::Assignment {
            target: LValue::Identifier(name),
            value,
        })
    }

    fn declaration_statement(&mut self, base_type: ScalarType) -> CompileResult<Statement> {
        let Some((_actual, type_end)) = self.declaration_type_span_at_current() else {
            return self.expected("declaration type");
        };
        let type_tokens = &self.tokens[self.index..type_end];
        let base_referent = declaration_base_referent_type(type_tokens);
        let type_includes_short = type_tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Short)));
        let type_is_unsigned = type_tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Unsigned)));
        let is_static = type_tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Static)));
        let type_includes_char = self.consume_declaration_type(base_type)?;
        let mut declarations = Vec::new();
        loop {
            let mut scalar_type = base_type;
            let mut pointer_depth = 0usize;
            while self.check_punctuator("*") {
                self.advance();
                pointer_depth += 1;
                scalar_type = ScalarType::Pointer;
            }
            let name = self.expect_identifier()?;
            let referent = if scalar_type == ScalarType::Pointer {
                pointer_referent_for_depth(pointer_depth, base_referent.as_deref())
            } else {
                None
            };
            let statement = if self.check_punctuator("[") {
                self.local_array_declaration(
                    type_includes_char,
                    type_includes_short,
                    type_is_unsigned,
                    scalar_type,
                    name,
                )?
            } else if self.check_punctuator("=") {
                self.advance();
                Statement::Declaration {
                    is_static,
                    scalar_type,
                    name,
                    referent,
                    initializer: Some(self.expression()?),
                }
            } else {
                Statement::Declaration {
                    is_static,
                    scalar_type,
                    name,
                    referent,
                    initializer: None,
                }
            };
            declarations.push(statement);
            if self.check_punctuator(",") {
                self.advance();
                continue;
            }
            self.expect_punctuator(";")?;
            break;
        }
        if declarations.len() == 1 {
            Ok(declarations.remove(0))
        } else {
            Ok(Statement::DeclarationList(declarations))
        }
    }

    fn local_array_declaration(
        &mut self,
        type_includes_char: bool,
        type_includes_short: bool,
        type_is_unsigned: bool,
        scalar_type: ScalarType,
        name: String,
    ) -> CompileResult<Statement> {
        self.advance();
        let explicit_length = if self.check_punctuator("]") {
            None
        } else {
            Some(local_array_length(
                &self.expression()?,
                self.known_constants,
            )?)
        };
        self.expect_punctuator("]")?;
        if scalar_type == ScalarType::Pointer {
            return self.local_pointer_array_declaration(name, explicit_length);
        }
        if scalar_type != ScalarType::Int {
            return Err(CompileError::new(
                "only local int, char, and pointer arrays are supported",
            ));
        }
        if type_includes_char && self.check_punctuator("[") {
            return self.local_char_matrix_declaration(name, explicit_length);
        }
        if type_includes_char {
            return self.local_char_array_declaration(name, explicit_length);
        }
        if type_includes_short {
            return self.local_short_array_declaration(name, explicit_length, type_is_unsigned);
        }
        self.local_int_array_declaration(name, explicit_length)
    }

    fn local_char_array_declaration(
        &mut self,
        name: String,
        explicit_length: Option<usize>,
    ) -> CompileResult<Statement> {
        let initializer = if self.check_punctuator("=") {
            self.advance();
            Some(self.local_char_array_initializer()?)
        } else {
            None
        };
        let length = match (explicit_length, &initializer) {
            (Some(length), _) => length,
            (None, Some(LocalCharArrayInitializer::StringLiteral(value))) => {
                inferred_local_char_array_length(value)?
            }
            (None, Some(LocalCharArrayInitializer::Bytes(values))) if !values.is_empty() => {
                values.len()
            }
            (None, None) => {
                return Err(CompileError::new(
                    "local char arrays require a size or string literal initializer",
                ));
            }
            (None, Some(LocalCharArrayInitializer::Bytes(_))) => {
                return Err(CompileError::new(
                    "local char arrays require a size or nonempty initializer",
                ));
            }
        };
        if let Some(initializer) = &initializer {
            validate_local_char_array_initializer_size(initializer, length)?;
        }
        Ok(Statement::LocalCharArray {
            name,
            length,
            initializer,
        })
    }

    fn local_char_array_initializer(&mut self) -> CompileResult<LocalCharArrayInitializer> {
        if self.check_punctuator("{") {
            return Ok(LocalCharArrayInitializer::Bytes(
                self.local_char_array_braced_initializer()?,
            ));
        }
        let initializer = self.expression()?;
        let Expr::StringLiteral(value) = initializer else {
            return Err(CompileError::new(
                "local char arrays require string literal or braced byte initializers",
            ));
        };
        Ok(LocalCharArrayInitializer::StringLiteral(value))
    }

    fn local_char_array_braced_initializer(&mut self) -> CompileResult<Vec<u8>> {
        self.expect_punctuator("{")?;
        let mut values = Vec::new();
        if self.check_punctuator("}") {
            self.advance();
            return Ok(values);
        }
        loop {
            let value = eval_integer_initializer_expr_with_constants(
                &self.expression()?,
                self.known_constants,
            )?
            .to_i64_trunc()?;
            values.push(
                u8::try_from(value)
                    .map_err(|_| CompileError::new("local char array initializer too large"))?,
            );
            if self.check_punctuator("}") {
                self.advance();
                return Ok(values);
            }
            self.expect_punctuator(",")?;
            if self.check_punctuator("}") {
                self.advance();
                return Ok(values);
            }
        }
    }

    fn local_char_matrix_declaration(
        &mut self,
        name: String,
        explicit_rows: Option<usize>,
    ) -> CompileResult<Statement> {
        let Some(rows) = explicit_rows else {
            return Err(CompileError::new("local char matrix rows require a size"));
        };
        self.expect_punctuator("[")?;
        let columns = local_array_length(&self.expression()?, self.known_constants)?;
        self.expect_punctuator("]")?;
        let initializer = if self.check_punctuator("=") {
            self.advance();
            Some(self.local_string_list_initializer(columns)?)
        } else {
            None
        };
        Ok(Statement::LocalCharMatrix {
            name,
            rows,
            columns,
            initializer,
        })
    }

    fn local_string_list_initializer(&mut self, columns: usize) -> CompileResult<Vec<String>> {
        self.expect_punctuator("{")?;
        let mut values = Vec::new();
        if self.check_punctuator("}") {
            self.advance();
            return Ok(values);
        }
        loop {
            let Expr::StringLiteral(value) = self.expression()? else {
                return Err(CompileError::new(
                    "local char matrix initializers require string literals",
                ));
            };
            validate_local_char_array_initializer(&value, columns)?;
            values.push(value);
            if self.check_punctuator("}") {
                self.advance();
                return Ok(values);
            }
            self.expect_punctuator(",")?;
            if self.check_punctuator("}") {
                self.advance();
                return Ok(values);
            }
        }
    }

    fn local_int_array_declaration(
        &mut self,
        name: String,
        explicit_length: Option<usize>,
    ) -> CompileResult<Statement> {
        let initializer = if self.check_punctuator("=") {
            self.advance();
            Some(self.local_int_array_initializer()?)
        } else {
            None
        };
        let length = match (explicit_length, &initializer) {
            (Some(length), _) => length,
            (None, Some(values)) if !values.is_empty() => values.len(),
            (None, _) => {
                return Err(CompileError::new(
                    "local int arrays require a size or initializer",
                ));
            }
        };
        let initializer = initializer
            .map(|mut values| {
                if values.len() > length {
                    return Err(CompileError::new(
                        "local int array initializer is too large",
                    ));
                }
                values.resize(length, 0);
                Ok(values)
            })
            .transpose()?;
        Ok(Statement::LocalIntArray {
            name,
            length,
            initializer,
        })
    }

    fn local_short_array_declaration(
        &self,
        name: String,
        explicit_length: Option<usize>,
        is_unsigned: bool,
    ) -> CompileResult<Statement> {
        if self.check_punctuator("=") {
            return Err(CompileError::new(
                "local short array initializers are not supported",
            ));
        }
        let Some(length) = explicit_length else {
            return Err(CompileError::new("local short arrays require a size"));
        };
        Ok(Statement::LocalShortArray {
            name,
            length,
            is_unsigned,
        })
    }

    fn local_pointer_array_declaration(
        &self,
        name: String,
        explicit_length: Option<usize>,
    ) -> CompileResult<Statement> {
        if self.check_punctuator("=") {
            return Err(CompileError::new(
                "local pointer array initializers are not supported",
            ));
        }
        let Some(length) = explicit_length else {
            return Err(CompileError::new("local pointer arrays require a size"));
        };
        Ok(Statement::LocalPointerArray {
            name,
            length,
            initializer: None,
        })
    }

    fn local_function_pointer_array_declaration(&mut self) -> CompileResult<Option<Statement>> {
        let Some((scalar_type, type_end)) = self.declaration_type_span_at_current() else {
            return Ok(None);
        };
        if scalar_type != ScalarType::Int
            || !self
                .tokens
                .get(type_end)
                .is_some_and(|token| token_is_punctuator(token, "("))
            || !self
                .tokens
                .get(type_end + 1)
                .is_some_and(|token| token_is_punctuator(token, "*"))
            || !self
                .tokens
                .get(type_end + 3)
                .is_some_and(|token| token_is_punctuator(token, "["))
        {
            return Ok(None);
        }

        self.index = type_end;
        self.expect_punctuator("(")?;
        self.expect_punctuator("*")?;
        let name = self.expect_identifier()?;
        self.expect_punctuator("[")?;
        let explicit_length = if self.check_punctuator("]") {
            None
        } else {
            Some(local_array_length(
                &self.expression()?,
                self.known_constants,
            )?)
        };
        self.expect_punctuator("]")?;
        self.expect_punctuator(")")?;
        self.skip_balanced_parentheses()?;
        let initializer = if self.check_punctuator("=") {
            self.advance();
            Some(self.local_pointer_array_initializer()?)
        } else {
            None
        };
        let length = match (explicit_length, &initializer) {
            (Some(length), _) => length,
            (None, Some(values)) if !values.is_empty() => values.len(),
            (None, _) => {
                return Err(CompileError::new(
                    "local function pointer arrays require a size or initializer",
                ));
            }
        };
        self.expect_punctuator(";")?;
        Ok(Some(Statement::LocalPointerArray {
            name,
            length,
            initializer,
        }))
    }

    fn skip_balanced_parentheses(&mut self) -> CompileResult<()> {
        self.expect_punctuator("(")?;
        let mut depth = 1usize;
        while !self.check_end() {
            if self.check_punctuator("(") {
                depth += 1;
                self.advance();
                continue;
            }
            if self.check_punctuator(")") {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| CompileError::new("unbalanced parentheses"))?;
                self.advance();
                if depth == 0 {
                    return Ok(());
                }
                continue;
            }
            self.advance();
        }
        Err(CompileError::new("unterminated parenthesized declarator"))
    }

    fn local_pointer_array_initializer(&mut self) -> CompileResult<Vec<Expr>> {
        self.expect_punctuator("{")?;
        let mut values = Vec::new();
        if self.check_punctuator("}") {
            self.advance();
            return Ok(values);
        }
        loop {
            values.push(self.expression()?);
            if self.check_punctuator("}") {
                self.advance();
                return Ok(values);
            }
            self.expect_punctuator(",")?;
            if self.check_punctuator("}") {
                self.advance();
                return Ok(values);
            }
        }
    }

    fn block_extern_declaration(&mut self) -> CompileResult<Option<Statement>> {
        if !self.check_keyword(Keyword::Extern) {
            return Ok(None);
        }
        let tokens = &self.tokens[self.index..];
        let Some(semicolon_index) = top_level_punctuator_index(tokens, ";") else {
            return Err(CompileError::new("unterminated extern declaration"));
        };
        let declaration = &tokens[..=semicolon_index];
        let Some(global) = parse_supported_global_declaration(
            declaration,
            self.known_structs,
            self.known_constants,
            &[],
        )?
        else {
            return Ok(None);
        };
        if !global.initializer.is_extern() {
            return Ok(None);
        }
        self.index += semicolon_index + 1;
        Ok(Some(Statement::ExternGlobal(global)))
    }

    fn block_function_prototype_declaration(&mut self) -> Option<Statement> {
        let tokens = &self.tokens[self.index..];
        let semicolon_index = top_level_punctuator_index(tokens, ";")?;
        let declaration = &tokens[..semicolon_index];
        let open_index = top_level_function_open_paren(declaration)?;
        let name_index = previous_identifier_index(declaration, open_index)?;
        supported_return_type(&declaration[..name_index])?;
        self.index += semicolon_index + 1;
        Some(Statement::Empty)
    }

    fn local_struct_declaration(&mut self) -> CompileResult<Option<Statement>> {
        let type_index = if self.check_keyword(Keyword::Static) {
            self.index + 1
        } else {
            self.index
        };
        let Some((struct_name, declarator_index)) = self.local_struct_name_at(type_index) else {
            return Ok(None);
        };
        self.index = declarator_index;
        let mut declarations = Vec::new();
        loop {
            let name = self.expect_identifier()?;
            if self.check_punctuator("=") {
                return Err(CompileError::new(
                    "local struct initializers are not supported",
                ));
            }
            declarations.push(Statement::LocalStruct {
                name,
                struct_name: struct_name.clone(),
            });
            if self.check_punctuator(",") {
                self.advance();
                continue;
            }
            self.expect_punctuator(";")?;
            break;
        }
        if declarations.len() == 1 {
            Ok(Some(declarations.remove(0)))
        } else {
            Ok(Some(Statement::DeclarationList(declarations)))
        }
    }

    fn local_struct_name_at(&self, index: usize) -> Option<(String, usize)> {
        if token_is_keyword(self.tokens.get(index)?, Keyword::Struct) {
            let name = token_identifier(self.tokens.get(index + 1)?)?;
            if !self.known_structs.iter().any(|layout| layout.name == name) {
                return None;
            }
            if !matches!(
                self.tokens.get(index + 2).map(|token| &token.kind),
                Some(TokenKind::Identifier(_))
            ) {
                return None;
            }
            return Some((name.to_owned(), index + 2));
        }
        let TokenKind::Identifier(name) = &self.tokens.get(index)?.kind else {
            return None;
        };
        if !self.known_structs.iter().any(|layout| layout.name == *name) {
            return None;
        }
        if !matches!(
            self.tokens.get(index + 1).map(|token| &token.kind),
            Some(TokenKind::Identifier(_))
        ) {
            return None;
        }
        Some((name.clone(), index + 1))
    }

    fn local_anonymous_union_declaration(&mut self) -> CompileResult<Option<Statement>> {
        if !self.check_keyword(Keyword::Union) {
            return Ok(None);
        }
        let open_brace = self.index + 1;
        if !self
            .tokens
            .get(open_brace)
            .is_some_and(|token| token_is_punctuator(token, "{"))
        {
            return Ok(None);
        }
        let close_brace = matching_top_level_brace(self.tokens, open_brace)
            .ok_or_else(|| CompileError::new("unterminated anonymous union declaration"))?;
        let Some(struct_name) =
            anonymous_union_struct_name(&self.tokens[open_brace + 1..close_brace])
        else {
            return Ok(None);
        };
        self.index = close_brace + 1;
        let name = self.expect_identifier()?;
        self.expect_punctuator(";")?;
        Ok(Some(Statement::LocalStruct {
            name,
            struct_name: struct_name.to_owned(),
        }))
    }

    fn local_int_array_initializer(&mut self) -> CompileResult<Vec<i32>> {
        self.expect_punctuator("{")?;
        let mut values = Vec::new();
        if self.check_punctuator("}") {
            self.advance();
            return Ok(values);
        }
        loop {
            let value = eval_integer_initializer_expr_with_constants(
                &self.expression()?,
                self.known_constants,
            )?
            .to_i64_trunc()?;
            values.push(
                i32::try_from(value)
                    .map_err(|_| CompileError::new("local int array initializer too large"))?,
            );
            if self.check_punctuator("}") {
                self.advance();
                return Ok(values);
            }
            self.expect_punctuator(",")?;
            if self.check_punctuator("}") {
                self.advance();
                return Ok(values);
            }
        }
    }

    fn local_enum_declaration(&mut self) -> CompileResult<Option<Statement>> {
        if !self.check_keyword(Keyword::Enum) {
            return Ok(None);
        }
        self.advance();
        if matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Identifier(_))
        ) && !self
            .tokens
            .get(self.index + 1)
            .is_some_and(|token| token_is_punctuator(token, "="))
        {
            self.advance();
        }
        self.expect_punctuator("{")?;
        let constants = self.local_enum_constants()?;
        self.expect_punctuator("}")?;
        self.expect_punctuator(";")?;
        Ok(Some(Statement::LocalConstants(constants)))
    }

    fn local_enum_constants(&mut self) -> CompileResult<Vec<Constant>> {
        let mut constants = Vec::new();
        let mut available_constants = self.known_constants.to_vec();
        let mut next_value = 0i64;
        while !self.check_punctuator("}") {
            let name = self.expect_identifier()?;
            let value = if self.check_punctuator("=") {
                self.advance();
                eval_integer_initializer_expr_with_constants(
                    &self.expression()?,
                    &available_constants,
                )?
                .to_i64_trunc()?
            } else {
                next_value
            };
            next_value = value
                .checked_add(1)
                .ok_or_else(|| CompileError::new("enum constant overflow"))?;
            let constant = Constant { name, value };
            available_constants.push(constant.clone());
            constants.push(constant);
            if self.check_punctuator(",") {
                self.advance();
                continue;
            }
            break;
        }
        Ok(constants)
    }

    fn static_aggregate_declaration(&mut self) -> CompileResult<Option<Statement>> {
        if !self.check_keyword(Keyword::Static) {
            return Ok(None);
        }
        let tokens = &self.tokens[self.index..];
        let Some(assign_index) = top_level_punctuator_index(tokens, "=") else {
            return Ok(None);
        };
        if top_level_punctuator_index(&tokens[..assign_index], "[").is_some() {
            return Ok(None);
        }
        if !tokens
            .get(assign_index + 1)
            .is_some_and(|token| token_is_punctuator(token, "{"))
        {
            return Ok(None);
        }
        let Some(name_index) = previous_identifier_index(tokens, assign_index) else {
            return Err(CompileError::new("expected static aggregate name"));
        };
        let name = token_identifier(&tokens[name_index])
            .ok_or_else(|| CompileError::new("expected static aggregate name"))?
            .to_owned();
        let Some(semicolon_index) = top_level_punctuator_index(tokens, ";") else {
            return Err(CompileError::new(
                "unterminated static aggregate declaration",
            ));
        };
        self.index += semicolon_index + 1;
        Ok(Some(Statement::Declaration {
            is_static: false,
            scalar_type: ScalarType::Int,
            name,
            referent: None,
            initializer: None,
        }))
    }

    fn if_statement(&mut self) -> CompileResult<Statement> {
        self.expect_keyword(Keyword::If)?;
        self.expect_punctuator("(")?;
        let condition = self.expression()?;
        self.expect_punctuator(")")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.check_keyword(Keyword::Else) {
            self.advance();
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> CompileResult<Statement> {
        self.expect_keyword(Keyword::While)?;
        self.expect_punctuator("(")?;
        let condition = self.expression()?;
        self.expect_punctuator(")")?;
        let body = Box::new(self.statement()?);
        Ok(Statement::While { condition, body })
    }

    fn do_while_statement(&mut self) -> CompileResult<Statement> {
        self.expect_keyword(Keyword::Do)?;
        let body = Box::new(self.statement()?);
        self.expect_keyword(Keyword::While)?;
        self.expect_punctuator("(")?;
        let condition = self.expression()?;
        self.expect_punctuator(")")?;
        self.expect_punctuator(";")?;
        Ok(Statement::DoWhile { body, condition })
    }

    fn for_statement(&mut self) -> CompileResult<Statement> {
        self.expect_keyword(Keyword::For)?;
        self.expect_punctuator("(")?;
        let initializer = if self.check_punctuator(";") {
            self.advance();
            None
        } else if let Some(scalar_type) = self.declaration_type_at_current() {
            Some(Box::new(self.declaration_statement(scalar_type)?))
        } else {
            let initializer = self.comma_expression_statement()?;
            self.expect_punctuator(";")?;
            Some(Box::new(initializer))
        };
        let condition = if self.check_punctuator(";") {
            None
        } else {
            Some(self.expression()?)
        };
        self.expect_punctuator(";")?;
        let post = if self.check_punctuator(")") {
            None
        } else {
            Some(Box::new(self.comma_expression_statement()?))
        };
        self.expect_punctuator(")")?;
        let body = Box::new(self.statement()?);
        Ok(Statement::For {
            initializer,
            condition,
            post,
            body,
        })
    }

    fn switch_statement(&mut self) -> CompileResult<Statement> {
        self.expect_keyword(Keyword::Switch)?;
        self.expect_punctuator("(")?;
        let condition = self.expression()?;
        self.expect_punctuator(")")?;
        self.expect_punctuator("{")?;
        let mut cases = Vec::new();
        let mut default = Vec::new();
        let mut saw_default = false;
        while !self.check_punctuator("}") {
            if self.check_keyword(Keyword::Case) {
                self.advance();
                let value = self.expression()?;
                self.expect_punctuator(":")?;
                let statements = self.switch_label_statements()?;
                cases.push(SwitchCase { value, statements });
                continue;
            }
            if self.check_keyword(Keyword::Default) {
                if saw_default {
                    return Err(CompileError::new("duplicate default label"));
                }
                saw_default = true;
                self.advance();
                self.expect_punctuator(":")?;
                default = self.switch_label_statements()?;
                continue;
            }
            return self.expected("switch case label");
        }
        self.expect_punctuator("}")?;
        Ok(Statement::Switch {
            condition,
            cases,
            default,
        })
    }

    fn switch_label_statements(&mut self) -> CompileResult<Vec<Statement>> {
        let mut statements = Vec::new();
        while !self.check_punctuator("}")
            && !self.check_keyword(Keyword::Case)
            && !self.check_keyword(Keyword::Default)
        {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn block_items(&mut self) -> CompileResult<Vec<Statement>> {
        self.expect_punctuator("{")?;
        let mut statements = Vec::new();
        while !self.check_punctuator("}") {
            statements.push(self.statement()?);
        }
        self.expect_punctuator("}")?;
        Ok(statements)
    }

    fn expression_statement(&mut self, expect_semicolon: bool) -> CompileResult<Statement> {
        let expr = self.expression()?;
        if expect_semicolon {
            self.expect_punctuator(";")?;
        }
        Ok(statement_from_expression(expr))
    }

    fn comma_expression_statement(&mut self) -> CompileResult<Statement> {
        let mut statements = vec![self.expression_statement(false)?];
        while self.check_punctuator(",") {
            self.advance();
            statements.push(self.expression_statement(false)?);
        }
        if statements.len() == 1 {
            Ok(statements.remove(0))
        } else {
            Ok(Statement::ExpressionList(statements))
        }
    }

    fn expression(&mut self) -> CompileResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> CompileResult<Expr> {
        let target = self.conditional()?;
        let Some(op) = self.assignment_operator_at_current() else {
            return Ok(target);
        };
        self.advance();
        let lvalue = lvalue_from_expr(target.clone())?;
        let value = self.assignment()?;
        let value = match op {
            AssignmentOperator::Simple => value,
            AssignmentOperator::Compound(op) => Expr::Binary {
                op,
                left: Box::new(target),
                right: Box::new(value),
            },
        };
        Ok(Expr::Assignment {
            target: lvalue,
            value: Box::new(value),
        })
    }

    fn conditional(&mut self) -> CompileResult<Expr> {
        let condition = self.logical_or()?;
        if !self.check_punctuator("?") {
            return Ok(condition);
        }
        self.advance();
        let then_expr = self.expression()?;
        self.expect_punctuator(":")?;
        let else_expr = self.conditional()?;
        Ok(Expr::Conditional {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
        })
    }

    fn logical_or(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::logical_and, &[("||", BinaryOp::LogicalOr)])
    }

    fn logical_and(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::bit_or, &[("&&", BinaryOp::LogicalAnd)])
    }

    fn bit_or(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::bit_xor, &[("|", BinaryOp::BitOr)])
    }

    fn bit_xor(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::bit_and, &[("^", BinaryOp::BitXor)])
    }

    fn bit_and(&mut self) -> CompileResult<Expr> {
        self.left_associative(Self::equality, &[("&", BinaryOp::BitAnd)])
    }

    fn equality(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::relational,
            &[("==", BinaryOp::Equal), ("!=", BinaryOp::NotEqual)],
        )
    }

    fn relational(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::shift,
            &[
                ("<", BinaryOp::Less),
                ("<=", BinaryOp::LessEqual),
                (">", BinaryOp::Greater),
                (">=", BinaryOp::GreaterEqual),
            ],
        )
    }

    fn shift(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::additive,
            &[("<<", BinaryOp::ShiftLeft), (">>", BinaryOp::ShiftRight)],
        )
    }

    fn additive(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::multiplicative,
            &[("+", BinaryOp::Add), ("-", BinaryOp::Sub)],
        )
    }

    fn multiplicative(&mut self) -> CompileResult<Expr> {
        self.left_associative(
            Self::unary,
            &[
                ("*", BinaryOp::Mul),
                ("/", BinaryOp::Div),
                ("%", BinaryOp::Mod),
            ],
        )
    }

    fn left_associative(
        &mut self,
        next: fn(&mut Self) -> CompileResult<Expr>,
        ops: &[(&str, BinaryOp)],
    ) -> CompileResult<Expr> {
        let mut expr = next(self)?;
        loop {
            let Some((punctuator, op)) = ops
                .iter()
                .find(|(punctuator, _op)| self.check_punctuator(punctuator))
                .copied()
            else {
                return Ok(expr);
            };
            self.expect_punctuator(punctuator)?;
            let right = next(self)?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
    }

    fn unary(&mut self) -> CompileResult<Expr> {
        if let Some((target, referent, next_index)) = self.cast_type_at_current() {
            self.index = next_index;
            return Ok(Expr::Cast {
                target,
                referent,
                expr: Box::new(self.unary()?),
            });
        }
        if self.check_keyword(Keyword::Sizeof) {
            return self.sizeof_expr();
        }
        if self.check_punctuator("++") {
            self.advance();
            return prefix_update_expr(self.unary()?, BinaryOp::Add);
        }
        if self.check_punctuator("--") {
            self.advance();
            return prefix_update_expr(self.unary()?, BinaryOp::Sub);
        }
        if self.check_punctuator("&") {
            self.advance();
            let target = lvalue_from_expr(self.unary()?)?;
            return Ok(Expr::AddressOf { target });
        }
        if self.check_punctuator("*") {
            self.advance();
            return Ok(Expr::Dereference {
                pointer: Box::new(self.unary()?),
            });
        }
        let op = if self.check_punctuator("+") {
            Some(UnaryOp::Plus)
        } else if self.check_punctuator("-") {
            Some(UnaryOp::Minus)
        } else if self.check_punctuator("~") {
            Some(UnaryOp::BitNot)
        } else if self.check_punctuator("!") {
            Some(UnaryOp::LogicalNot)
        } else {
            None
        };
        if let Some(op) = op {
            self.advance();
            return Ok(Expr::Unary {
                op,
                expr: Box::new(self.unary()?),
            });
        }
        self.postfix()
    }

    fn sizeof_expr(&mut self) -> CompileResult<Expr> {
        self.expect_keyword(Keyword::Sizeof)?;
        if self.check_punctuator("(") {
            let start = self.index + 1;
            let close = self.tokens[start..]
                .iter()
                .position(|token| token_is_punctuator(token, ")"))
                .map(|offset| start + offset);
            if let Some(close) = close
                && let Some(size) = self.sizeof_type(&self.tokens[start..close])
            {
                self.index = close + 1;
                return i64::try_from(size)
                    .map(Expr::Integer)
                    .map_err(|_| CompileError::new("sizeof result does not fit i64"));
            }
        }
        Ok(Expr::SizeOfExpr {
            expr: Box::new(self.unary()?),
        })
    }

    fn sizeof_type(&self, tokens: &[Token]) -> Option<usize> {
        if let Some(size) = sizeof_type(tokens) {
            return Some(size);
        }
        let name = tokens.iter().rev().find_map(token_identifier)?;
        self.known_structs
            .iter()
            .find(|layout| layout.name == name)
            .map(|layout| layout.size)
    }

    fn postfix(&mut self) -> CompileResult<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.check_punctuator("[") {
                self.advance();
                let index = self.expression()?;
                self.expect_punctuator("]")?;
                expr = Expr::Subscript {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
                continue;
            }
            if self.check_punctuator(".") || self.check_punctuator("->") {
                let dereference = self.check_punctuator("->");
                self.advance();
                let field = self.expect_identifier()?;
                expr = Expr::Member {
                    base: Box::new(expr),
                    field,
                    dereference,
                };
                continue;
            }
            if self.check_punctuator("(") {
                expr = Expr::IndirectCall {
                    callee: Box::new(expr),
                    args: self.call_arguments()?,
                };
                continue;
            }
            if self.check_punctuator("++") {
                self.advance();
                expr = Expr::PostIncrement {
                    target: lvalue_from_expr(expr)?,
                    decrement: false,
                };
                continue;
            }
            if self.check_punctuator("--") {
                self.advance();
                expr = Expr::PostIncrement {
                    target: lvalue_from_expr(expr)?,
                    decrement: true,
                };
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn cast_type_at_current(&self) -> Option<(ScalarType, Option<String>, usize)> {
        if !self.check_punctuator("(") {
            return None;
        }
        let start = self.index + 1;
        let close = matching_top_level_paren(self.tokens, self.index)?;
        let cast_tokens = &self.tokens[start..close];
        if function_pointer_cast_type(cast_tokens) {
            return Some((ScalarType::Pointer, None, close + 1));
        }
        let target = supported_cast_type_with_typedefs(
            cast_tokens,
            self.known_scalar_typedefs,
            self.known_pointer_typedefs,
        )?;
        let referent = if target == ScalarType::Pointer {
            pointer_referent_from_specifiers(cast_tokens)
        } else {
            None
        };
        Some((target, referent, close + 1))
    }

    fn primary(&mut self) -> CompileResult<Expr> {
        if let Some(token) = self.peek() {
            match &token.kind {
                TokenKind::Integer(value) => {
                    let value = *value;
                    self.advance();
                    if self.check_punctuator(".") {
                        self.advance();
                        let fractional = self.expect_integer()?;
                        return Ok(Expr::DoubleLiteral(format!("{value}.{fractional}")));
                    }
                    Ok(Expr::Integer(value))
                }
                TokenKind::CharLiteral(value) => {
                    let value = i64::from(u32::from(*value));
                    self.advance();
                    Ok(Expr::Integer(value))
                }
                TokenKind::StringLiteral(value) => {
                    let mut value = value.clone();
                    self.advance();
                    while let Some(Token {
                        kind: TokenKind::StringLiteral(next),
                        ..
                    }) = self.peek()
                    {
                        value.push_str(next);
                        self.advance();
                    }
                    Ok(Expr::StringLiteral(value))
                }
                TokenKind::Identifier(value) => {
                    let value = value.clone();
                    self.advance();
                    if self.check_punctuator("(") {
                        return Ok(Expr::Call {
                            callee: value,
                            args: self.call_arguments()?,
                        });
                    }
                    Ok(Expr::Identifier(value))
                }
                TokenKind::Punctuator(value) if value == "." => {
                    self.advance();
                    let fractional = self.expect_integer()?;
                    Ok(Expr::DoubleLiteral(format!("0.{fractional}")))
                }
                TokenKind::Punctuator(value) if value == "(" => {
                    self.advance();
                    let expr = self.expression()?;
                    self.expect_punctuator(")")?;
                    Ok(expr)
                }
                _ => Err(CompileError::new("expected expression").at(token.line, token.column)),
            }
        } else {
            Err(CompileError::new("unexpected end of token stream"))
        }
    }

    fn call_arguments(&mut self) -> CompileResult<Vec<Expr>> {
        self.expect_punctuator("(")?;
        let mut args = Vec::new();
        if self.check_punctuator(")") {
            self.advance();
            return Ok(args);
        }
        loop {
            args.push(self.expression()?);
            if self.check_punctuator(")") {
                self.advance();
                return Ok(args);
            }
            self.expect_punctuator(",")?;
        }
    }

    fn declaration_type_at_current(&self) -> Option<ScalarType> {
        self.declaration_type_span_at_current()
            .map(|(scalar_type, _end)| scalar_type)
    }

    fn consume_declaration_type(&mut self, expected: ScalarType) -> CompileResult<bool> {
        let Some((actual, end)) = self.declaration_type_span_at_current() else {
            return self.expected("declaration type");
        };
        if actual != expected {
            return Err(CompileError::new("unexpected declaration type"));
        }
        let type_includes_char = self.tokens[self.index..end]
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Char)));
        self.index = end;
        Ok(type_includes_char)
    }

    fn declaration_type_span_at_current(&self) -> Option<(ScalarType, usize)> {
        let mut index = self.index;
        let mut saw_type = false;
        let mut saw_double = false;
        let mut saw_storage_class = false;
        let mut saw_struct_pointer = false;
        let mut saw_pointer_typedef = false;
        let mut long_count = 0usize;
        while let Some(token) = self.tokens.get(index) {
            match &token.kind {
                TokenKind::Keyword(
                    Keyword::Const | Keyword::Restrict | Keyword::Signed | Keyword::Volatile,
                ) => {}
                TokenKind::Keyword(Keyword::Register | Keyword::Static) => {
                    saw_storage_class = true;
                }
                TokenKind::Keyword(
                    Keyword::Char | Keyword::Int | Keyword::Short | Keyword::Unsigned,
                ) => {
                    saw_type = true;
                }
                TokenKind::Keyword(Keyword::Void) => {
                    if !self.struct_pointer_declarator_follows(index + 1) {
                        return None;
                    }
                    saw_type = true;
                    saw_pointer_typedef = true;
                }
                TokenKind::Keyword(Keyword::Double) => {
                    saw_type = true;
                    saw_double = true;
                }
                TokenKind::Keyword(Keyword::Struct) => {
                    if saw_type {
                        break;
                    }
                    let name = self.tokens.get(index + 1).and_then(token_identifier)?;
                    if !self.known_structs.iter().any(|layout| layout.name == name)
                        || !self.struct_pointer_declarator_follows(index + 2)
                    {
                        return None;
                    }
                    saw_type = true;
                    saw_struct_pointer = true;
                    index += 2;
                    continue;
                }
                TokenKind::Keyword(Keyword::Long) => {
                    saw_type = true;
                    long_count += 1;
                }
                TokenKind::Identifier(name) => {
                    if saw_type {
                        break;
                    }
                    if self
                        .known_pointer_typedefs
                        .iter()
                        .any(|known| known == name)
                    {
                        saw_pointer_typedef = true;
                    } else if let Some(scalar_type) =
                        self.supported_declaration_typedef_scalar(name)
                    {
                        if scalar_type == ScalarType::VaList {
                            return Some((ScalarType::VaList, index + 1));
                        }
                        if scalar_type != ScalarType::Int {
                            return None;
                        }
                    } else if self.known_structs.iter().any(|layout| layout.name == *name)
                        && self.struct_pointer_declarator_follows(index + 1)
                    {
                        saw_struct_pointer = true;
                    } else if saw_storage_class {
                        break;
                    } else {
                        return None;
                    }
                    saw_type = true;
                }
                _ => break,
            }
            index += 1;
        }
        if !saw_type {
            if saw_storage_class {
                return Some((ScalarType::Int, index));
            }
            return None;
        }
        if saw_struct_pointer || saw_pointer_typedef {
            Some((ScalarType::Pointer, index))
        } else if saw_double && long_count == 0 {
            Some((ScalarType::Double, index))
        } else if long_count == 0 {
            Some((ScalarType::Int, index))
        } else {
            Some((ScalarType::LongLong, index))
        }
    }

    fn supported_declaration_typedef_scalar(&self, name: &str) -> Option<ScalarType> {
        supported_typedef_scalar(name).or_else(|| {
            self.known_scalar_typedefs
                .iter()
                .any(|known_name| known_name == name)
                .then_some(ScalarType::Int)
        })
    }

    fn struct_pointer_declarator_follows(&self, mut index: usize) -> bool {
        while let Some(token) = self.tokens.get(index) {
            match &token.kind {
                TokenKind::Keyword(Keyword::Const | Keyword::Restrict | Keyword::Volatile) => {
                    index += 1;
                }
                TokenKind::Punctuator(value) if value == "*" => return true,
                _ => return false,
            }
        }
        false
    }

    fn current_identifier_starts_assignment(&self) -> bool {
        matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Identifier(_))
        ) && self
            .tokens
            .get(self.index + 1)
            .is_some_and(token_is_assignment_operator)
    }

    fn assignment_operator_at_current(&self) -> Option<AssignmentOperator> {
        let token = self.peek()?;
        if token_is_punctuator(token, "=") {
            Some(AssignmentOperator::Simple)
        } else if token_is_punctuator(token, "+=") {
            Some(AssignmentOperator::Compound(BinaryOp::Add))
        } else if token_is_punctuator(token, "-=") {
            Some(AssignmentOperator::Compound(BinaryOp::Sub))
        } else if token_is_punctuator(token, "*=") {
            Some(AssignmentOperator::Compound(BinaryOp::Mul))
        } else if token_is_punctuator(token, "/=") {
            Some(AssignmentOperator::Compound(BinaryOp::Div))
        } else if token_is_punctuator(token, "%=") {
            Some(AssignmentOperator::Compound(BinaryOp::Mod))
        } else if token_is_punctuator(token, "<<=") {
            Some(AssignmentOperator::Compound(BinaryOp::ShiftLeft))
        } else if token_is_punctuator(token, ">>=") {
            Some(AssignmentOperator::Compound(BinaryOp::ShiftRight))
        } else if token_is_punctuator(token, "&=") {
            Some(AssignmentOperator::Compound(BinaryOp::BitAnd))
        } else if token_is_punctuator(token, "^=") {
            Some(AssignmentOperator::Compound(BinaryOp::BitXor))
        } else if token_is_punctuator(token, "|=") {
            Some(AssignmentOperator::Compound(BinaryOp::BitOr))
        } else {
            None
        }
    }

    fn expect_keyword(&mut self, expected: Keyword) -> CompileResult<()> {
        if self.check_keyword(expected) {
            self.advance();
            return Ok(());
        }
        self.expected(&format!("keyword {expected:?}"))
    }

    fn expect_identifier(&mut self) -> CompileResult<String> {
        if let Some(Token {
            kind: TokenKind::Identifier(value),
            ..
        }) = self.peek()
        {
            let value = value.clone();
            self.advance();
            return Ok(value);
        }
        self.expected("identifier")
    }

    fn expect_integer(&mut self) -> CompileResult<i64> {
        if let Some(Token {
            kind: TokenKind::Integer(value),
            ..
        }) = self.peek()
        {
            let value = *value;
            self.advance();
            return Ok(value);
        }
        self.expected("integer")
    }

    fn expect_punctuator(&mut self, expected: &str) -> CompileResult<()> {
        if self.check_punctuator(expected) {
            self.advance();
            return Ok(());
        }
        self.expected(&format!("punctuator {expected}"))
    }

    fn expected<T>(&self, expected: &str) -> CompileResult<T> {
        if let Some(token) = self.peek() {
            return Err(
                CompileError::new(format!("expected {expected}")).at(token.line, token.column)
            );
        }
        Err(CompileError::new(format!("expected {expected}")))
    }

    fn check_keyword(&self, expected: Keyword) -> bool {
        matches!(self.peek(), Some(Token { kind: TokenKind::Keyword(value), .. }) if *value == expected)
    }

    fn check_punctuator(&self, expected: &str) -> bool {
        matches!(self.peek(), Some(Token { kind: TokenKind::Punctuator(value), .. }) if value == expected)
    }

    fn check_end(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token {
                kind: TokenKind::End,
                ..
            }) | None
        )
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    const fn advance(&mut self) {
        self.index += 1;
    }
}

fn lvalue_from_expr(expr: Expr) -> CompileResult<LValue> {
    match expr {
        Expr::Identifier(name) => Ok(LValue::Identifier(name)),
        Expr::Subscript { array, index } => Ok(LValue::Subscript { array, index }),
        Expr::Dereference { pointer } => Ok(LValue::Subscript {
            array: pointer,
            index: Box::new(Expr::Integer(0)),
        }),
        Expr::Member {
            base,
            field,
            dereference,
        } => Ok(LValue::Member {
            base,
            field,
            dereference,
        }),
        _ => Err(CompileError::new("unsupported assignment target")),
    }
}

fn prefix_update_expr(expr: Expr, op: BinaryOp) -> CompileResult<Expr> {
    let target = lvalue_from_expr(expr.clone())?;
    Ok(Expr::Assignment {
        target,
        value: Box::new(Expr::Binary {
            op,
            left: Box::new(expr),
            right: Box::new(Expr::Integer(1)),
        }),
    })
}

fn statement_from_expression(expr: Expr) -> Statement {
    match expr {
        Expr::Assignment { target, value } => Statement::Assignment {
            target,
            value: *value,
        },
        _ => Statement::Expression(expr),
    }
}

fn parse_supported_global_declaration(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    if last_token_is_punctuator(tokens, "}") || !last_token_is_punctuator(tokens, ";") {
        return Ok(None);
    }
    if let Some(global) = parse_global_function_pointer(tokens) {
        return Ok(Some(global));
    }
    if top_level_function_open_paren(tokens).is_some() {
        return Ok(None);
    }
    if let Some(global) = parse_global_unsigned_char_array(tokens, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer_string_array(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer_name_array(tokens, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer_array(tokens, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_extern_pointer_array(tokens, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_struct_array(tokens, known_structs, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = global_struct_object::parse(tokens, known_structs, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_short_array(tokens, constants, sizeof_symbols)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_extern_int_array(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_double_array(tokens, constants)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_int_array(tokens, known_structs, constants, sizeof_symbols)?
    {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_extern_scalar(tokens, known_structs)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer(tokens, constants)? {
        return Ok(Some(global));
    }
    parse_global_int(tokens, known_structs, constants, sizeof_symbols)
}

fn parse_supported_global_declarations(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Vec<Global>> {
    if let Some(globals) = parse_global_int_declarator_list(tokens, known_structs, constants)? {
        return Ok(with_global_linkage(tokens, globals));
    }
    parse_supported_global_declaration(tokens, known_structs, constants, sizeof_symbols).map(
        |global| {
            let globals = global.map_or_else(Vec::new, |global| vec![global]);
            with_global_linkage(tokens, globals)
        },
    )
}

fn with_global_linkage(tokens: &[Token], mut globals: Vec<Global>) -> Vec<Global> {
    let is_static = token_has_keyword(tokens, Keyword::Static);
    for global in &mut globals {
        global.is_static = is_static;
    }
    globals
}

fn parse_global_function_pointer(tokens: &[Token]) -> Option<Global> {
    let declaration = tokens.get(..tokens.len().checked_sub(1)?)?;
    let name = function_pointer_name(declaration)?;
    let initializer = if token_has_keyword(declaration, Keyword::Extern) {
        GlobalInitializer::ExternPointer { referent: None }
    } else {
        GlobalInitializer::PointerNull { referent: None }
    };
    Some(Global::new(name, initializer))
}

fn unsupported_data_declaration_blocks_empty_unit(tokens: &[Token]) -> bool {
    if ignorable_static_const_char_array(tokens) {
        return false;
    }
    if declaration_only_extern(tokens) {
        return false;
    }
    matches!(
        classify_external_item(tokens),
        Some(ExternalItem::Declaration { .. })
    )
}

fn declaration_only_extern(tokens: &[Token]) -> bool {
    token_has_keyword(tokens, Keyword::Extern) && top_level_punctuator_index(tokens, "=").is_none()
}

fn ignorable_static_const_char_array(tokens: &[Token]) -> bool {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return false;
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return false;
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return false;
    };
    if !global_specifiers_are_static_const_char(&declaration[..name_index]) {
        return false;
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return false;
    };
    let Some(assign_index) = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
    else {
        return false;
    };
    let initializer = &declaration[close_bracket + 2 + assign_index..];
    matches!(
        initializer,
        [Token {
            kind: TokenKind::StringLiteral(_),
            ..
        }]
    )
}

fn parse_global_unsigned_char_array(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    if !global_specifiers_are_unsigned_char(&declaration[..name_index]) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    if let Some(global) = parse_global_unsigned_char_matrix(
        declaration,
        open_bracket,
        close_bracket,
        name_index,
        constants,
    )? {
        return Ok(Some(global));
    }
    let values = if let Some(assign_index) =
        top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
    {
        let assign_index = close_bracket + 1 + assign_index;
        let Ok(values) =
            parse_unsigned_char_initializer(&declaration[assign_index + 1..], constants)
        else {
            return Ok(None);
        };
        values
    } else {
        let length = parse_unsigned_char_array_length(
            &declaration[open_bracket + 1..close_bracket],
            constants,
        )?;
        vec![0; length]
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global array name"))?
        .to_owned();
    if token_has_keyword(&declaration[..name_index], Keyword::Extern) {
        return Ok(Some(Global::new(
            name,
            GlobalInitializer::ExternUnsignedCharArray,
        )));
    }
    Ok(Some(Global::new(
        name,
        GlobalInitializer::UnsignedCharArray(values),
    )))
}

fn parse_global_unsigned_char_matrix(
    declaration: &[Token],
    open_bracket: usize,
    close_bracket: usize,
    name_index: usize,
    constants: &[Constant],
) -> CompileResult<Option<Global>> {
    if !declaration
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        return Ok(None);
    }
    let second_open = close_bracket + 1;
    let Some(second_close) = matching_top_level_bracket(declaration, second_open) else {
        return Err(
            CompileError::new("unterminated global matrix declarator").at(
                declaration[second_open].line,
                declaration[second_open].column,
            ),
        );
    };
    let rows =
        parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket], constants)?;
    let columns =
        parse_unsigned_char_array_length(&declaration[second_open + 1..second_close], constants)?;
    let length = rows
        .checked_mul(columns)
        .ok_or_else(|| CompileError::new("global byte matrix size overflow"))?;
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global matrix name"))?
        .to_owned();
    if token_has_keyword(&declaration[..name_index], Keyword::Extern) {
        return Ok(Some(Global::new(
            name,
            GlobalInitializer::ExternUnsignedCharMatrix { columns },
        )));
    }
    let values = parse_global_unsigned_char_matrix_values(
        declaration,
        second_close,
        rows,
        columns,
        length,
        constants,
    )?;
    Ok(Some(Global::new(
        name,
        GlobalInitializer::UnsignedCharMatrix { values, columns },
    )))
}

fn parse_global_unsigned_char_matrix_values(
    declaration: &[Token],
    second_close: usize,
    rows: usize,
    columns: usize,
    length: usize,
    constants: &[Constant],
) -> CompileResult<Vec<u8>> {
    let Some(assign_index) = top_level_punctuator_index(&declaration[second_close + 1..], "=")
    else {
        return Ok(vec![0; length]);
    };
    parse_char_matrix_initializer(
        &declaration[second_close + 2 + assign_index..],
        rows,
        columns,
        constants,
    )
}

fn parse_unsigned_char_array_length(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<usize> {
    if tokens.is_empty() {
        return Err(CompileError::new("expected unsigned char array length"));
    }
    let value = parse_integer_initializer_with_constants(tokens, constants)?;
    if value <= 0 {
        return Err(CompileError::new("global array length must be positive"));
    }
    usize::try_from(value).map_err(|_| CompileError::new("global array length is too large"))
}

fn parse_global_pointer_array(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    if !global_specifiers_are_pointer(&declaration[..name_index]) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global pointer-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    if top_level_punctuator_index(&declaration[close_bracket + 1..], "=").is_some() {
        return Ok(None);
    }
    let rows =
        parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket], constants)?;
    let (length, columns, last_dimension_close) = if declaration
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        let second_open = close_bracket + 1;
        let Some(second_close) = matching_top_level_bracket(declaration, second_open) else {
            return Err(
                CompileError::new("unterminated global pointer-matrix declarator").at(
                    declaration[second_open].line,
                    declaration[second_open].column,
                ),
            );
        };
        let columns = parse_unsigned_char_array_length(
            &declaration[second_open + 1..second_close],
            constants,
        )?;
        let length = rows
            .checked_mul(columns)
            .ok_or_else(|| CompileError::new("global pointer-matrix size overflow"))?;
        (length, Some(columns), second_close)
    } else {
        (rows, None, close_bracket)
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global pointer-array name"))?
        .to_owned();
    if top_level_punctuator_index(&declaration[last_dimension_close + 1..], "=").is_some() {
        return Ok(None);
    }
    let referent = pointer_referent_from_specifiers(&declaration[..name_index]);
    Ok(Some(Global::new(
        name,
        GlobalInitializer::PointerArray {
            referent,
            length,
            columns,
        },
    )))
}

fn parse_global_pointer_string_array(tokens: &[Token]) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    if !global_specifiers_are_pointer(&declaration[..name_index]) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global pointer-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    let Some(assign_index) = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
    else {
        return Ok(None);
    };
    let assign_index = close_bracket + 1 + assign_index;
    let Ok(values) = parse_string_array_initializer(&declaration[assign_index + 1..]) else {
        return Ok(None);
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global pointer-array name"))?
        .to_owned();
    let referent = pointer_referent_from_specifiers(&declaration[..name_index]);
    Ok(Some(Global::new(
        name,
        GlobalInitializer::PointerStringArray { referent, values },
    )))
}

fn parse_global_pointer_name_array(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    if !global_specifiers_are_pointer(&declaration[..name_index]) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global pointer-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    let Some(assign_index) = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
    else {
        return Ok(None);
    };
    let assign_index = close_bracket + 1 + assign_index;
    let Ok(values) = parse_identifier_array_initializer(&declaration[assign_index + 1..]) else {
        return Ok(None);
    };
    let explicit_length = if open_bracket + 1 == close_bracket {
        None
    } else {
        Some(parse_unsigned_char_array_length(
            &declaration[open_bracket + 1..close_bracket],
            constants,
        )?)
    };
    let length = explicit_length.unwrap_or(values.len());
    if values.len() > length {
        return Err(CompileError::new(
            "too many global pointer-array name initializers",
        ));
    }
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global pointer-array name"))?
        .to_owned();
    let referent = pointer_referent_from_specifiers(&declaration[..name_index]);
    Ok(Some(Global::new(
        name,
        GlobalInitializer::PointerNameArray {
            referent,
            values,
            length,
        },
    )))
}

fn parse_global_extern_pointer_array(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    if !global_specifiers_are_extern_pointer(&declaration[..name_index]) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated extern global pointer-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    let (columns, last_dimension_close) = if declaration
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        let second_open = close_bracket + 1;
        let Some(second_close) = matching_top_level_bracket(declaration, second_open) else {
            return Err(
                CompileError::new("unterminated extern global pointer-matrix declarator").at(
                    declaration[second_open].line,
                    declaration[second_open].column,
                ),
            );
        };
        let columns = parse_unsigned_char_array_length(
            &declaration[second_open + 1..second_close],
            constants,
        )?;
        (Some(columns), second_close)
    } else {
        (None, close_bracket)
    };
    if top_level_punctuator_index(&declaration[last_dimension_close + 1..], "=").is_some() {
        return Ok(None);
    }
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected extern global pointer-array name"))?
        .to_owned();
    let referent = pointer_referent_from_specifiers(&declaration[..name_index]);
    Ok(Some(Global::new(
        name,
        GlobalInitializer::ExternPointerArray { referent, columns },
    )))
}

fn parse_global_struct_array(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    let Some(struct_name) = global_struct_specifier_name(&declaration[..name_index], known_structs)
    else {
        return Ok(None);
    };
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global struct-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global struct-array name"))?
        .to_owned();
    let is_extern = token_has_keyword(&declaration[..name_index], Keyword::Extern);
    let columns =
        parse_global_struct_array_columns(declaration, close_bracket, is_extern, constants)?;
    let assign_index = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
        .map(|offset| close_bracket + 1 + offset);
    let initializer = if is_extern {
        GlobalInitializer::ExternStructArray { struct_name }
    } else {
        let length = parse_global_struct_array_length(
            declaration,
            open_bracket,
            close_bracket,
            columns,
            assign_index,
            constants,
        )?;
        let values = parse_global_struct_array_values(
            declaration,
            columns,
            assign_index,
            known_structs,
            constants,
        );
        if values.len() > length {
            return Err(CompileError::new(
                "too many global struct-array initializers",
            ));
        }
        GlobalInitializer::StructArray {
            struct_name,
            length,
            columns,
            values,
        }
    };
    Ok(Some(Global::new(name, initializer)))
}

fn parse_global_struct_array_columns(
    declaration: &[Token],
    close_bracket: usize,
    is_extern: bool,
    constants: &[Constant],
) -> CompileResult<Option<usize>> {
    if !declaration
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        return Ok(None);
    }
    let second_open = close_bracket + 1;
    let Some(second_close) = matching_top_level_bracket(declaration, second_open) else {
        return Err(
            CompileError::new("unterminated global struct-matrix declarator").at(
                declaration[second_open].line,
                declaration[second_open].column,
            ),
        );
    };
    let tokens = &declaration[second_open + 1..second_close];
    if is_extern {
        parse_optional_symbolic_array_length(tokens, constants)
    } else {
        parse_unsigned_char_array_length(tokens, constants).map(Some)
    }
}

fn parse_global_struct_array_length(
    declaration: &[Token],
    open_bracket: usize,
    close_bracket: usize,
    columns: Option<usize>,
    assign_index: Option<usize>,
    constants: &[Constant],
) -> CompileResult<usize> {
    if let Some(columns) = columns {
        if open_bracket + 1 == close_bracket {
            return Err(CompileError::new("expected global struct-matrix row count"));
        }
        let rows = parse_unsigned_char_array_length(
            &declaration[open_bracket + 1..close_bracket],
            constants,
        )?;
        return rows
            .checked_mul(columns)
            .ok_or_else(|| CompileError::new("global struct-matrix size overflow"));
    }
    if let Some(assign_index) = assign_index {
        let initializer = &declaration[assign_index + 1..];
        return aggregate_initializer_length(initializer).ok_or_else(|| {
            CompileError::new("expected global struct-array initializer").at(
                declaration[assign_index].line,
                declaration[assign_index].column,
            )
        });
    }
    if open_bracket + 1 == close_bracket {
        return Err(CompileError::new("expected global struct-array length"));
    }
    parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket], constants)
}

fn parse_global_struct_array_values(
    declaration: &[Token],
    columns: Option<usize>,
    assign_index: Option<usize>,
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> Vec<Vec<GlobalStructInitializerValue>> {
    if columns.is_some() {
        return Vec::new();
    }
    let Some(assign_index) = assign_index else {
        return Vec::new();
    };
    global_struct_initializer::parse(&declaration[assign_index + 1..], known_structs, constants)
        .unwrap_or_default()
}

fn parse_optional_symbolic_array_length(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<usize>> {
    if let [
        Token {
            kind: TokenKind::Identifier(name),
            ..
        },
    ] = tokens
        && !constants.iter().any(|constant| constant.name == *name)
    {
        return Ok(None);
    }
    parse_unsigned_char_array_length(tokens, constants).map(Some)
}

fn aggregate_initializer_length(tokens: &[Token]) -> Option<usize> {
    if !tokens
        .first()
        .is_some_and(|token| token_is_punctuator(token, "{"))
    {
        return None;
    }
    let close = matching_top_level_brace(tokens, 0)?;
    let values = &tokens[1..close];
    if values.is_empty() {
        return Some(0);
    }
    let mut depth = 0usize;
    let mut count = 1usize;
    for token in values {
        if token_is_punctuator(token, "{") {
            depth += 1;
        } else if token_is_punctuator(token, "}") {
            depth = depth.checked_sub(1)?;
        } else if depth == 0 && token_is_punctuator(token, ",") {
            count += 1;
        }
    }
    Some(count)
}

fn parse_global_extern_int_array(tokens: &[Token]) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    if !global_specifiers_are_extern_int(&declaration[..name_index]) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated extern global int-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    if top_level_punctuator_index(&declaration[close_bracket + 1..], "=").is_some() {
        return Ok(None);
    }
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected extern global int-array name"))?
        .to_owned();
    Ok(Some(Global::new(name, GlobalInitializer::ExternIntArray)))
}

fn parse_global_short_array(
    tokens: &[Token],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    let specifiers = &declaration[..name_index];
    if !global_specifiers_are_short(specifiers) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global short-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global short-array name"))?
        .to_owned();
    let is_extern = token_has_keyword(specifiers, Keyword::Extern);
    let is_unsigned = token_has_keyword(specifiers, Keyword::Unsigned);
    if declaration
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        return parse_global_short_matrix(
            ShortArrayDeclarator {
                declaration,
                open_bracket,
                close_bracket,
                name: &name,
                is_extern,
                is_unsigned,
            },
            constants,
            sizeof_symbols,
        );
    }
    if is_extern {
        return Ok(Some(Global::new(
            name,
            GlobalInitializer::ExternShortArray {
                is_unsigned,
                columns: None,
            },
        )));
    }
    let explicit_length = if open_bracket + 1 == close_bracket {
        None
    } else {
        Some(parse_unsigned_char_array_length(
            &declaration[open_bracket + 1..close_bracket],
            constants,
        )?)
    };
    let assign_index = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
        .map(|offset| close_bracket + 1 + offset);
    let values = if let Some(assign_index) = assign_index {
        parse_short_initializer_values(
            parse_int_array_initializer(
                &declaration[assign_index + 1..],
                explicit_length,
                constants,
                sizeof_symbols,
            )?,
            is_unsigned,
        )?
    } else {
        let Some(length) = explicit_length else {
            return Err(CompileError::new("expected short array length"));
        };
        vec![0; length]
    };
    Ok(Some(Global::new(
        name,
        GlobalInitializer::ShortArray {
            values,
            is_unsigned,
            columns: None,
        },
    )))
}

fn parse_global_short_matrix(
    spec: ShortArrayDeclarator<'_>,
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    let second_open = spec.close_bracket + 1;
    let Some(second_close) = matching_top_level_bracket(spec.declaration, second_open) else {
        return Err(
            CompileError::new("unterminated global short-matrix declarator").at(
                spec.declaration[second_open].line,
                spec.declaration[second_open].column,
            ),
        );
    };
    let columns = parse_unsigned_char_array_length(
        &spec.declaration[second_open + 1..second_close],
        constants,
    )?;
    if spec.is_extern {
        return Ok(Some(Global::new(
            spec.name.to_owned(),
            GlobalInitializer::ExternShortArray {
                is_unsigned: spec.is_unsigned,
                columns: Some(columns),
            },
        )));
    }
    let rows = parse_unsigned_char_array_length(
        &spec.declaration[spec.open_bracket + 1..spec.close_bracket],
        constants,
    )?;
    let length = rows
        .checked_mul(columns)
        .ok_or_else(|| CompileError::new("global short matrix size overflow"))?;
    let values = if let Some(assign_index) =
        top_level_punctuator_index(&spec.declaration[second_close + 1..], "=")
    {
        let assign_index = second_close + 1 + assign_index;
        parse_short_initializer_values(
            parse_int_matrix_initializer(
                &spec.declaration[assign_index + 1..],
                rows,
                columns,
                constants,
                sizeof_symbols,
            )?,
            spec.is_unsigned,
        )?
    } else {
        vec![0; length]
    };
    Ok(Some(Global::new(
        spec.name.to_owned(),
        GlobalInitializer::ShortArray {
            values,
            is_unsigned: spec.is_unsigned,
            columns: Some(columns),
        },
    )))
}

#[derive(Clone, Copy)]
struct ShortArrayDeclarator<'a> {
    declaration: &'a [Token],
    open_bracket: usize,
    close_bracket: usize,
    name: &'a str,
    is_extern: bool,
    is_unsigned: bool,
}

fn parse_global_int_array(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    if !global_specifiers_are_int(&declaration[..name_index], known_structs) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global int-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    if declaration
        .get(close_bracket + 1)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        let second_open = close_bracket + 1;
        let Some(second_close) = matching_top_level_bracket(declaration, second_open) else {
            return Err(
                CompileError::new("unterminated global int-matrix declarator").at(
                    declaration[second_open].line,
                    declaration[second_open].column,
                ),
            );
        };
        let rows = parse_unsigned_char_array_length(
            &declaration[open_bracket + 1..close_bracket],
            constants,
        )?;
        let columns = parse_unsigned_char_array_length(
            &declaration[second_open + 1..second_close],
            constants,
        )?;
        let length = rows
            .checked_mul(columns)
            .ok_or_else(|| CompileError::new("global int matrix size overflow"))?;
        let values = if let Some(assign_index) =
            top_level_punctuator_index(&declaration[second_close + 1..], "=")
        {
            let assign_index = second_close + 1 + assign_index;
            parse_int_matrix_initializer(
                &declaration[assign_index + 1..],
                rows,
                columns,
                constants,
                sizeof_symbols,
            )?
        } else {
            vec![0; length]
        };
        let name = token_identifier(&declaration[name_index])
            .ok_or_else(|| CompileError::new("expected global int-matrix name"))?
            .to_owned();
        return Ok(Some(Global::new(
            name,
            GlobalInitializer::IntMatrix { values, columns },
        )));
    }
    let explicit_length = if open_bracket + 1 == close_bracket {
        None
    } else {
        Some(parse_unsigned_char_array_length(
            &declaration[open_bracket + 1..close_bracket],
            constants,
        )?)
    };
    let assign_index = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
        .map(|offset| close_bracket + 1 + offset);
    let values = if let Some(assign_index) = assign_index {
        parse_int_array_initializer(
            &declaration[assign_index + 1..],
            explicit_length,
            constants,
            sizeof_symbols,
        )?
    } else {
        let Some(length) = explicit_length else {
            return Err(CompileError::new("expected unsigned char array length"));
        };
        vec![0; length]
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global int-array name"))?
        .to_owned();
    Ok(Some(Global::new(name, GlobalInitializer::IntArray(values))))
}

fn parse_global_double_array(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    if !global_specifiers_are_double(&declaration[..name_index]) {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(declaration, open_bracket) else {
        return Err(
            CompileError::new("unterminated global double-array declarator").at(
                declaration[open_bracket].line,
                declaration[open_bracket].column,
            ),
        );
    };
    if top_level_punctuator_index(&declaration[close_bracket + 1..], "=").is_some() {
        return Ok(None);
    }
    let length =
        parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket], constants)?;
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global double-array name"))?
        .to_owned();
    Ok(Some(Global::new(
        name,
        GlobalInitializer::DoubleArray { length },
    )))
}

fn parse_global_extern_scalar(
    tokens: &[Token],
    known_structs: &[StructLayout],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    if top_level_punctuator_index(declaration, "=").is_some()
        || top_level_punctuator_index(declaration, "[").is_some()
    {
        return Ok(None);
    }
    let Some(name_index) = previous_identifier_index(declaration, declaration.len()) else {
        return Ok(None);
    };
    let specifiers = &declaration[..name_index];
    let initializer = if global_specifiers_are_extern_pointer(specifiers) {
        GlobalInitializer::ExternPointer {
            referent: pointer_referent_from_specifiers(specifiers),
        }
    } else if let Some(struct_name) = global_struct_specifier_name(specifiers, known_structs) {
        GlobalInitializer::ExternStructObject { struct_name }
    } else if global_specifiers_are_extern_int(specifiers) {
        GlobalInitializer::Extern(ScalarType::Int)
    } else {
        return Ok(None);
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected extern global name"))?
        .to_owned();
    Ok(Some(Global::new(name, initializer)))
}

fn parse_global_pointer(tokens: &[Token], constants: &[Constant]) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let end_index = top_level_punctuator_index(declaration, "=").unwrap_or(declaration.len());
    if top_level_punctuator_index(&declaration[..end_index], "[").is_some() {
        return Ok(None);
    }
    let Some(name_index) = previous_identifier_index(declaration, end_index) else {
        return Ok(None);
    };
    if !global_specifiers_are_pointer(&declaration[..name_index]) {
        return Ok(None);
    }
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global pointer name"))?
        .to_owned();
    let referent = pointer_referent_from_specifiers(&declaration[..name_index]);
    if end_index != declaration.len() {
        let initializer = &declaration[end_index + 1..];
        if let Ok(value) = parse_string_initializer(initializer) {
            return Ok(Some(Global::new(
                name,
                GlobalInitializer::PointerString { referent, value },
            )));
        }
        if let Some((base, index)) =
            parse_global_pointer_subscript_address_initializer(initializer, constants)?
        {
            return Ok(Some(Global::new(
                name,
                GlobalInitializer::PointerSubscriptAddress {
                    referent,
                    base,
                    index,
                },
            )));
        }
        let Ok(value) = parse_integer_initializer(initializer) else {
            return Ok(None);
        };
        if value != 0 {
            return Ok(None);
        }
    }
    Ok(Some(Global::new(
        name,
        GlobalInitializer::PointerNull { referent },
    )))
}

fn parse_global_pointer_subscript_address_initializer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<(String, usize)>> {
    if !tokens
        .first()
        .is_some_and(|token| token_is_punctuator(token, "&"))
    {
        return Ok(None);
    }
    let Some(base) = tokens.get(1).and_then(token_identifier) else {
        return Ok(None);
    };
    if !tokens
        .get(2)
        .is_some_and(|token| token_is_punctuator(token, "["))
    {
        return Ok(None);
    }
    let Some(close_bracket) = matching_top_level_bracket(tokens, 2) else {
        return Err(
            CompileError::new("unterminated global pointer initializer subscript")
                .at(tokens[2].line, tokens[2].column),
        );
    };
    if close_bracket + 1 != tokens.len() {
        return Ok(None);
    }
    let index = parse_integer_initializer_with_constants(&tokens[3..close_bracket], constants)?;
    if index < 0 {
        return Err(
            CompileError::new("global pointer initializer subscript must be nonnegative")
                .at(tokens[2].line, tokens[2].column),
        );
    }
    usize::try_from(index)
        .map(|index| Some((base.to_owned(), index)))
        .map_err(|_| CompileError::new("global pointer initializer subscript is too large"))
}

fn parse_global_int(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let end_index = top_level_punctuator_index(declaration, "=").unwrap_or(declaration.len());
    if top_level_punctuator_index(&declaration[..end_index], "[").is_some() {
        return Ok(None);
    }
    let Some(name_index) = previous_identifier_index(declaration, end_index) else {
        return Ok(None);
    };
    if !global_specifiers_are_int(&declaration[..name_index], known_structs) {
        return Ok(None);
    }
    if declaration
        .get(end_index + 1)
        .is_some_and(|token| token_is_punctuator(token, "{"))
    {
        return Ok(None);
    }
    let initializer = if end_index == declaration.len() {
        GlobalInitializer::Int(0)
    } else {
        parse_global_int_initializer(&declaration[end_index + 1..], constants, sizeof_symbols)?
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global int name"))?
        .to_owned();
    Ok(Some(Global::new(name, initializer)))
}

fn parse_global_int_declarator_list(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Option<Vec<Global>>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let ranges = top_level_comma_ranges(declaration);
    if ranges.len() <= 1 {
        return Ok(None);
    }
    let Some((first_start, first_end)) = ranges.first().copied() else {
        return Ok(None);
    };
    let first = &declaration[first_start..first_end];
    let first_end_index = top_level_punctuator_index(first, "=").unwrap_or(first.len());
    let Some(first_name_index) = previous_identifier_index(first, first_end_index) else {
        return Ok(None);
    };
    let base_specifiers = &first[..first_name_index];
    if !global_specifiers_are_int(base_specifiers, known_structs) {
        return Ok(None);
    }
    let mut globals = Vec::with_capacity(ranges.len());
    for (range_index, (start, end)) in ranges.iter().copied().enumerate() {
        let segment = &declaration[start..end];
        let end_index = top_level_punctuator_index(segment, "=").unwrap_or(segment.len());
        if top_level_punctuator_index(&segment[..end_index], "[").is_some() {
            return Ok(None);
        }
        let Some(name_index) = previous_identifier_index(segment, end_index) else {
            return Ok(None);
        };
        if range_index > 0 && !segment[..name_index].is_empty() {
            return Ok(None);
        }
        let initializer = if end_index == segment.len() {
            GlobalInitializer::Int(0)
        } else {
            parse_global_int_initializer(&segment[end_index + 1..], constants, &[])?
        };
        let name = token_identifier(&segment[name_index])
            .ok_or_else(|| CompileError::new("expected global int name"))?
            .to_owned();
        globals.push(Global::new(name, initializer));
    }
    Ok(Some(globals))
}

fn global_specifiers_are_unsigned_char(tokens: &[Token]) -> bool {
    let mut saw_char = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(
                Keyword::Extern
                | Keyword::Static
                | Keyword::Const
                | Keyword::Volatile
                | Keyword::Unsigned,
            ) => {}
            TokenKind::Keyword(Keyword::Char) => saw_char = true,
            TokenKind::Identifier(name) if name == "byte" => saw_char = true,
            _ => return false,
        }
    }
    saw_char
}

fn global_specifiers_are_static_const_char(tokens: &[Token]) -> bool {
    let mut saw_static = false;
    let mut saw_const = false;
    let mut saw_char = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Static) => saw_static = true,
            TokenKind::Keyword(Keyword::Const) => saw_const = true,
            TokenKind::Keyword(Keyword::Char) => saw_char = true,
            _ => return false,
        }
    }
    saw_static && saw_const && saw_char
}

fn global_specifiers_are_pointer(tokens: &[Token]) -> bool {
    !token_has_keyword(tokens, Keyword::Extern) && global_specifiers_are_pointer_like(tokens, false)
}

fn global_specifiers_are_extern_pointer(tokens: &[Token]) -> bool {
    token_has_keyword(tokens, Keyword::Extern) && global_specifiers_are_pointer_like(tokens, true)
}

fn global_specifiers_are_pointer_like(tokens: &[Token], allow_extern: bool) -> bool {
    let mut saw_type = false;
    let mut saw_pointer = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Extern) if allow_extern => {}
            TokenKind::Keyword(
                Keyword::Const
                | Keyword::Restrict
                | Keyword::Static
                | Keyword::Volatile
                | Keyword::Signed
                | Keyword::Unsigned,
            ) => {}
            TokenKind::Keyword(
                Keyword::Char | Keyword::Int | Keyword::Long | Keyword::Short | Keyword::Void,
            )
            | TokenKind::Identifier(_) => saw_type = true,
            TokenKind::Punctuator(value) if value == "*" => saw_pointer = true,
            _ => return false,
        }
    }
    saw_type && saw_pointer
}

fn global_struct_specifier_name(
    tokens: &[Token],
    known_structs: &[StructLayout],
) -> Option<String> {
    if tokens.iter().any(|token| token_is_punctuator(token, "*")) {
        return None;
    }
    let name = tokens.iter().rev().find_map(token_identifier)?;
    known_structs
        .iter()
        .any(|layout| layout.name == name)
        .then(|| name.to_owned())
}

fn global_specifiers_are_int(tokens: &[Token], known_structs: &[StructLayout]) -> bool {
    !token_has_keyword(tokens, Keyword::Extern)
        && global_specifiers_are_int_like(tokens, false, known_structs)
}

fn global_specifiers_are_extern_int(tokens: &[Token]) -> bool {
    token_has_keyword(tokens, Keyword::Extern) && global_specifiers_are_int_like(tokens, true, &[])
}

fn global_specifiers_are_short(tokens: &[Token]) -> bool {
    global_specifiers_are_int_like(tokens, true, &[])
        && tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Short)))
}

fn global_specifiers_are_int_like(
    tokens: &[Token],
    allow_extern: bool,
    known_structs: &[StructLayout],
) -> bool {
    let mut saw_int = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Extern) if allow_extern => {}
            TokenKind::Keyword(
                Keyword::Static | Keyword::Const | Keyword::Volatile | Keyword::Signed,
            ) => {}
            TokenKind::Keyword(Keyword::Unsigned | Keyword::Int | Keyword::Short) => {
                saw_int = true;
            }
            TokenKind::Identifier(name) => {
                if known_structs.iter().any(|layout| layout.name == *name) {
                    return false;
                }
                saw_int = true;
            }
            _ => return false,
        }
    }
    saw_int
}

fn global_specifiers_are_double(tokens: &[Token]) -> bool {
    let mut saw_double = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Static | Keyword::Const | Keyword::Volatile) => {}
            TokenKind::Keyword(Keyword::Double) => saw_double = true,
            _ => return false,
        }
    }
    saw_double
}

fn parse_global_int_initializer(
    tokens: &[Token],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<GlobalInitializer> {
    if let Ok(value) = parse_integer_initializer_with_context(tokens, constants, sizeof_symbols) {
        return Ok(GlobalInitializer::Int(value));
    }
    match tokens {
        [token] => {
            let Some(name) = token_identifier(token) else {
                return Err(CompileError::new("unsupported global integer initializer")
                    .at(token.line, token.column));
            };
            Ok(GlobalInitializer::IntConstant(name.to_owned()))
        }
        [first, ..] => Err(CompileError::new("unsupported global integer initializer")
            .at(first.line, first.column)),
        [] => Err(CompileError::new("expected global integer initializer")),
    }
}

fn parse_int_array_initializer(
    tokens: &[Token],
    explicit_length: Option<usize>,
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Vec<i32>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new("expected global int array initializer"));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global int array initializer").at(first.line, first.column)
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global int array initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global int array initializer")
                .at(token.line, token.column),
        );
    }

    let mut values = Vec::new();
    let mut start = 1usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global int array initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        let value =
            parse_integer_initializer_with_context(&item[..item_len], constants, sizeof_symbols)?;
        values.push(i32::try_from(value).map_err(|_| {
            CompileError::new("global int array initializer does not fit i32")
                .at(tokens[start].line, tokens[start].column)
        })?);
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    let length = explicit_length.unwrap_or(values.len());
    if values.len() > length {
        return Err(CompileError::new("too many global int array initializers")
            .at(first.line, first.column));
    }
    values.resize(length, 0);
    Ok(values)
}

fn parse_short_initializer_values(values: Vec<i32>, is_unsigned: bool) -> CompileResult<Vec<i32>> {
    for value in &values {
        if is_unsigned {
            u16::try_from(*value)
                .map_err(|_| CompileError::new("global unsigned short initializer too large"))?;
        } else {
            i16::try_from(*value)
                .map_err(|_| CompileError::new("global short initializer too large"))?;
        }
    }
    Ok(values)
}

fn parse_int_matrix_initializer(
    tokens: &[Token],
    rows: usize,
    columns: usize,
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Vec<i32>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new("expected global int matrix initializer"));
    };
    if !token_is_punctuator(first, "{") {
        return Err(CompileError::new("expected global int matrix initializer")
            .at(first.line, first.column));
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global int matrix initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global int matrix initializer")
                .at(token.line, token.column),
        );
    }

    let length = rows
        .checked_mul(columns)
        .ok_or_else(|| CompileError::new("global int matrix size overflow"))?;
    let mut values = Vec::with_capacity(length);
    let mut start = 1usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global int matrix initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        let item = &item[..item_len];
        if item
            .first()
            .is_some_and(|token| token_is_punctuator(token, "{"))
        {
            values.extend(parse_int_array_initializer(
                item,
                Some(columns),
                constants,
                sizeof_symbols,
            )?);
        } else {
            let value = parse_integer_initializer_with_context(item, constants, sizeof_symbols)?;
            values.push(i32::try_from(value).map_err(|_| {
                CompileError::new("global int matrix initializer does not fit i32")
                    .at(tokens[start].line, tokens[start].column)
            })?);
        }
        if values.len() > length {
            return Err(CompileError::new("too many global int matrix initializers")
                .at(tokens[start].line, tokens[start].column));
        }
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    values.resize(length, 0);
    Ok(values)
}

fn parse_string_array_initializer(tokens: &[Token]) -> CompileResult<Vec<String>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new(
            "expected global pointer-array initializer",
        ));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global pointer-array initializer")
                .at(first.line, first.column),
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global pointer-array initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global pointer-array initializer")
                .at(token.line, token.column),
        );
    }

    let mut values = Vec::new();
    let mut start = 1usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global pointer-array initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        values.push(parse_string_initializer(&item[..item_len])?);
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    Ok(values)
}

fn parse_identifier_array_initializer(tokens: &[Token]) -> CompileResult<Vec<String>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new(
            "expected global pointer-array initializer",
        ));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global pointer-array initializer")
                .at(first.line, first.column),
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global pointer-array initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global pointer-array initializer")
                .at(token.line, token.column),
        );
    }

    let mut values = Vec::new();
    let mut start = 1usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global pointer-array initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        let [token] = &item[..item_len] else {
            return Err(
                CompileError::new("unsupported global pointer-array initializer")
                    .at(tokens[start].line, tokens[start].column),
            );
        };
        let Some(name) = token_identifier(token) else {
            return Err(
                CompileError::new("expected global pointer-array initializer name")
                    .at(token.line, token.column),
            );
        };
        values.push(name.to_owned());
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    Ok(values)
}

fn parse_char_matrix_initializer(
    tokens: &[Token],
    rows: usize,
    columns: usize,
    constants: &[Constant],
) -> CompileResult<Vec<u8>> {
    let Ok(values) = parse_string_array_initializer(tokens) else {
        return parse_unsigned_char_matrix_initializer(tokens, rows, columns, constants);
    };
    if values.len() > rows {
        return Err(CompileError::new(
            "global char matrix initializer has too many rows",
        ));
    }
    let length = rows
        .checked_mul(columns)
        .ok_or_else(|| CompileError::new("global char matrix size overflow"))?;
    let mut bytes = Vec::with_capacity(length);
    for value in values {
        if value.len() > columns {
            return Err(CompileError::new(
                "global char matrix initializer row is too large",
            ));
        }
        let row_end = bytes
            .len()
            .checked_add(columns)
            .ok_or_else(|| CompileError::new("global char matrix row size overflow"))?;
        bytes.extend_from_slice(value.as_bytes());
        if value.len() < columns {
            bytes.push(0);
        }
        bytes.resize(row_end, 0);
    }
    bytes.resize(length, 0);
    Ok(bytes)
}

fn parse_unsigned_char_matrix_initializer(
    tokens: &[Token],
    rows: usize,
    columns: usize,
    constants: &[Constant],
) -> CompileResult<Vec<u8>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new("expected global byte matrix initializer"));
    };
    if !token_is_punctuator(first, "{") {
        return Err(CompileError::new("expected global byte matrix initializer")
            .at(first.line, first.column));
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global byte matrix initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global byte matrix initializer")
                .at(token.line, token.column),
        );
    }

    let length = rows
        .checked_mul(columns)
        .ok_or_else(|| CompileError::new("global byte matrix size overflow"))?;
    let mut values = Vec::with_capacity(length);
    let mut start = 1usize;
    let mut row_count = 0usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global byte matrix initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        let item = &item[..item_len];
        if item
            .first()
            .is_some_and(|token| token_is_punctuator(token, "{"))
        {
            let row = parse_unsigned_char_initializer(item, constants)?;
            if row.len() > columns {
                return Err(CompileError::new("global byte matrix row is too large")
                    .at(tokens[start].line, tokens[start].column));
            }
            values.extend(row.iter());
            values.resize(
                values
                    .len()
                    .checked_add(columns - row.len())
                    .ok_or_else(|| CompileError::new("global byte matrix row size overflow"))?,
                0,
            );
            row_count += 1;
        } else {
            let value = parse_integer_initializer_with_constants(item, constants)?;
            values.push(u8::try_from(value).map_err(|_| {
                CompileError::new("global byte matrix initializer does not fit u8")
                    .at(tokens[start].line, tokens[start].column)
            })?);
        }
        if row_count > rows || values.len() > length {
            return Err(
                CompileError::new("too many global byte matrix initializers")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    values.resize(length, 0);
    Ok(values)
}

fn parse_string_initializer(tokens: &[Token]) -> CompileResult<String> {
    if tokens.is_empty() {
        return Err(CompileError::new("expected global string initializer"));
    }
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs: &[],
        known_constants: &[],
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
    };
    let expr = parser.expression()?;
    if let Some(token) = parser.peek() {
        return Err(
            CompileError::new("unsupported global string initializer").at(token.line, token.column)
        );
    }
    let Expr::StringLiteral(value) = expr else {
        return Err(CompileError::new("expected global string initializer"));
    };
    Ok(value)
}

fn parse_unsigned_char_initializer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Vec<u8>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new("expected global array initializer"));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global array initializer").at(first.line, first.column)
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global array initializer").at(first.line, first.column)
        );
    };
    if close_brace + 1 != tokens.len() {
        let token = &tokens[close_brace + 1];
        return Err(
            CompileError::new("unsupported global array initializer").at(token.line, token.column)
        );
    }
    let initializer = &tokens[1..close_brace];
    if initializer.is_empty() {
        return Ok(Vec::new());
    }
    let mut values = Vec::new();
    for (start, end) in top_level_comma_ranges(initializer) {
        if start == end && end == initializer.len() {
            continue;
        }
        if start == end {
            let token = &initializer[start];
            return Err(CompileError::new("expected global array initializer value")
                .at(token.line, token.column));
        }
        let value = parse_integer_initializer_with_constants(&initializer[start..end], constants)?;
        let byte = u8::try_from(value).map_err(|_| {
            CompileError::new("unsigned char initializer does not fit u8")
                .at(initializer[start].line, initializer[start].column)
        })?;
        values.push(byte);
    }
    Ok(values)
}

fn parse_integer_initializer(tokens: &[Token]) -> CompileResult<i64> {
    parse_integer_initializer_with_constants(tokens, &[])
}

fn parse_integer_initializer_with_constants(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<i64> {
    parse_integer_initializer_with_context(tokens, constants, &[])
}

fn parse_integer_initializer_with_context(
    tokens: &[Token],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<i64> {
    if tokens.is_empty() {
        return Err(CompileError::new("expected global integer initializer"));
    }
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs: &[],
        known_constants: constants,
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
    };
    let expr = parser.expression()?;
    if let Some(token) = parser.peek() {
        return Err(CompileError::new("unsupported global integer initializer")
            .at(token.line, token.column));
    }
    eval_integer_initializer_expr_with_context(&expr, constants, sizeof_symbols)?.to_i64_trunc()
}

#[derive(Clone, Copy)]
struct InitializerNumber {
    numerator: i128,
    denominator: i128,
}

impl InitializerNumber {
    fn integer(value: i64) -> Self {
        Self {
            numerator: i128::from(value),
            denominator: 1,
        }
    }

    fn new(numerator: i128, denominator: i128) -> CompileResult<Self> {
        if denominator == 0 {
            return Err(CompileError::new("integer initializer division by zero"));
        }
        if denominator < 0 {
            return Ok(Self {
                numerator: numerator
                    .checked_neg()
                    .ok_or_else(|| CompileError::new("integer initializer overflow"))?,
                denominator: denominator
                    .checked_neg()
                    .ok_or_else(|| CompileError::new("integer initializer overflow"))?,
            });
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    fn decimal(value: &str) -> CompileResult<Self> {
        let Some((whole, fractional)) = value.split_once('.') else {
            return Err(CompileError::new("unsupported decimal initializer"));
        };
        let whole = if whole.is_empty() {
            0
        } else {
            whole
                .parse::<i128>()
                .map_err(|_| CompileError::new("decimal initializer is too large"))?
        };
        let fractional = if fractional.is_empty() {
            0
        } else {
            fractional
                .parse::<i128>()
                .map_err(|_| CompileError::new("decimal initializer is too large"))?
        };
        let mut denominator = 1i128;
        for _digit in value
            .split_once('.')
            .map_or("", |(_whole, fractional)| fractional)
            .chars()
        {
            denominator = denominator
                .checked_mul(10)
                .ok_or_else(|| CompileError::new("decimal initializer is too large"))?;
        }
        let numerator = whole
            .checked_mul(denominator)
            .and_then(|whole| whole.checked_add(fractional))
            .ok_or_else(|| CompileError::new("decimal initializer is too large"))?;
        Self::new(numerator, denominator)
    }

    fn to_i128_integer(self) -> CompileResult<i128> {
        if self.denominator != 1 {
            return Err(CompileError::new(
                "non-integer operand in integer initializer",
            ));
        }
        Ok(self.numerator)
    }

    fn to_i64_trunc(self) -> CompileResult<i64> {
        i64::try_from(self.numerator / self.denominator)
            .map_err(|_| CompileError::new("integer initializer does not fit i64"))
    }

    fn checked_neg(self) -> CompileResult<Self> {
        Self::new(
            self.numerator
                .checked_neg()
                .ok_or_else(|| CompileError::new("integer initializer overflow"))?,
            self.denominator,
        )
    }

    fn checked_add(self, right: Self) -> CompileResult<Self> {
        let numerator = self
            .numerator
            .checked_mul(right.denominator)
            .and_then(|left| {
                right
                    .numerator
                    .checked_mul(self.denominator)
                    .and_then(|right| left.checked_add(right))
            })
            .ok_or_else(|| CompileError::new("integer initializer overflow"))?;
        let denominator = self
            .denominator
            .checked_mul(right.denominator)
            .ok_or_else(|| CompileError::new("integer initializer overflow"))?;
        Self::new(numerator, denominator)
    }

    fn checked_sub(self, right: Self) -> CompileResult<Self> {
        self.checked_add(right.checked_neg()?)
    }

    fn checked_mul(self, right: Self) -> CompileResult<Self> {
        let numerator = self
            .numerator
            .checked_mul(right.numerator)
            .ok_or_else(|| CompileError::new("integer initializer overflow"))?;
        let denominator = self
            .denominator
            .checked_mul(right.denominator)
            .ok_or_else(|| CompileError::new("integer initializer overflow"))?;
        Self::new(numerator, denominator)
    }

    fn checked_div(self, right: Self) -> CompileResult<Self> {
        let numerator = self
            .numerator
            .checked_mul(right.denominator)
            .ok_or_else(|| CompileError::new("integer initializer overflow"))?;
        let denominator = self
            .denominator
            .checked_mul(right.numerator)
            .ok_or_else(|| CompileError::new("integer initializer overflow"))?;
        Self::new(numerator, denominator)
    }

    fn checked_rem(self, right: Self) -> CompileResult<Self> {
        let left = self.to_i128_integer()?;
        let right = right.to_i128_integer()?;
        if right == 0 {
            return Err(CompileError::new("integer initializer modulo by zero"));
        }
        Self::new(
            left.checked_rem(right)
                .ok_or_else(|| CompileError::new("integer initializer overflow"))?,
            1,
        )
    }

    fn checked_shl(self, right: Self) -> CompileResult<Self> {
        let left = self.to_i128_integer()?;
        let right = initializer_shift_count(right)?;
        Self::new(
            left.checked_shl(right)
                .ok_or_else(|| CompileError::new("integer initializer shift overflow"))?,
            1,
        )
    }

    fn checked_shr(self, right: Self) -> CompileResult<Self> {
        let left = self.to_i128_integer()?;
        let right = initializer_shift_count(right)?;
        Self::new(
            left.checked_shr(right)
                .ok_or_else(|| CompileError::new("integer initializer shift overflow"))?,
            1,
        )
    }
}

fn initializer_shift_count(value: InitializerNumber) -> CompileResult<u32> {
    let value = value.to_i128_integer()?;
    if value < 0 {
        return Err(CompileError::new(
            "negative integer initializer shift count",
        ));
    }
    u32::try_from(value).map_err(|_| CompileError::new("integer initializer shift count too large"))
}

fn eval_integer_initializer_expr_with_constants(
    expr: &Expr,
    constants: &[Constant],
) -> CompileResult<InitializerNumber> {
    eval_integer_initializer_expr_with_context(expr, constants, &[])
}

fn eval_integer_initializer_expr_with_context(
    expr: &Expr,
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<InitializerNumber> {
    match expr {
        Expr::Integer(value) => Ok(InitializerNumber::integer(*value)),
        Expr::DoubleLiteral(value) => InitializerNumber::decimal(value),
        Expr::Unary { op, expr } => {
            let value =
                eval_integer_initializer_expr_with_context(expr, constants, sizeof_symbols)?;
            match op {
                UnaryOp::Plus => Ok(value),
                UnaryOp::Minus => value.checked_neg(),
                UnaryOp::BitNot => {
                    let value = value.to_i128_integer()?;
                    InitializerNumber::new(!value, 1)
                }
                UnaryOp::LogicalNot => {
                    let value = value.to_i128_integer()?;
                    Ok(InitializerNumber::integer(i64::from(value == 0)))
                }
            }
        }
        Expr::Cast { target, expr, .. } => {
            let value =
                eval_integer_initializer_expr_with_context(expr, constants, sizeof_symbols)?;
            match target {
                ScalarType::Int
                | ScalarType::LongLong
                | ScalarType::Pointer
                | ScalarType::VaList => Ok(InitializerNumber::integer(value.to_i64_trunc()?)),
                ScalarType::Double => Ok(value),
            }
        }
        Expr::Binary { op, left, right } => {
            let left = eval_integer_initializer_expr_with_context(left, constants, sizeof_symbols)?;
            let right =
                eval_integer_initializer_expr_with_context(right, constants, sizeof_symbols)?;
            eval_integer_binary_initializer_expr(*op, left, right)
        }
        Expr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            if eval_integer_initializer_expr_with_context(condition, constants, sizeof_symbols)?
                .to_i128_integer()?
                == 0
            {
                eval_integer_initializer_expr_with_context(else_expr, constants, sizeof_symbols)
            } else {
                eval_integer_initializer_expr_with_context(then_expr, constants, sizeof_symbols)
            }
        }
        Expr::Identifier(name) => constants
            .iter()
            .rev()
            .find(|constant| constant.name == *name)
            .map(|constant| InitializerNumber::integer(constant.value))
            .ok_or_else(|| {
                CompileError::new(format!("identifier {name} is not an integer initializer"))
            }),
        Expr::Call { callee, .. } => Err(CompileError::new(format!(
            "call to {callee} is not an integer initializer"
        ))),
        Expr::IndirectCall { .. } => Err(CompileError::new(
            "indirect call is not an integer initializer",
        )),
        Expr::SizeOfExpr { expr } => eval_sizeof_initializer_expr(expr, sizeof_symbols),
        Expr::StringLiteral(_)
        | Expr::AddressOf { .. }
        | Expr::Dereference { .. }
        | Expr::Member { .. }
        | Expr::Subscript { .. }
        | Expr::Assignment { .. }
        | Expr::PostIncrement { .. } => {
            Err(CompileError::new("unsupported global integer initializer"))
        }
    }
}

fn eval_sizeof_initializer_expr(
    expr: &Expr,
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<InitializerNumber> {
    let Expr::Identifier(name) = expr else {
        return Err(CompileError::new("unsupported global sizeof initializer"));
    };
    let Some((_name, size)) = sizeof_symbols
        .iter()
        .rev()
        .find(|(candidate, _size)| candidate == name)
    else {
        return Err(CompileError::new(format!(
            "unknown global sizeof initializer: {name}"
        )));
    };
    i64::try_from(*size)
        .map(InitializerNumber::integer)
        .map_err(|_| CompileError::new("global sizeof initializer is too large"))
}

fn eval_integer_binary_initializer_expr(
    op: BinaryOp,
    left: InitializerNumber,
    right: InitializerNumber,
) -> CompileResult<InitializerNumber> {
    match op {
        BinaryOp::Mul => left.checked_mul(right),
        BinaryOp::Div => left.checked_div(right),
        BinaryOp::Mod => left.checked_rem(right),
        BinaryOp::Add => left.checked_add(right),
        BinaryOp::Sub => left.checked_sub(right),
        BinaryOp::ShiftLeft => left.checked_shl(right),
        BinaryOp::ShiftRight => left.checked_shr(right),
        BinaryOp::BitAnd => {
            InitializerNumber::new(left.to_i128_integer()? & right.to_i128_integer()?, 1)
        }
        BinaryOp::BitXor => {
            InitializerNumber::new(left.to_i128_integer()? ^ right.to_i128_integer()?, 1)
        }
        BinaryOp::BitOr => {
            InitializerNumber::new(left.to_i128_integer()? | right.to_i128_integer()?, 1)
        }
        BinaryOp::Less => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? < right.to_i128_integer()?,
        ))),
        BinaryOp::LessEqual => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? <= right.to_i128_integer()?,
        ))),
        BinaryOp::Greater => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? > right.to_i128_integer()?,
        ))),
        BinaryOp::GreaterEqual => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? >= right.to_i128_integer()?,
        ))),
        BinaryOp::Equal => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? == right.to_i128_integer()?,
        ))),
        BinaryOp::NotEqual => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? != right.to_i128_integer()?,
        ))),
        BinaryOp::LogicalAnd => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? != 0 && right.to_i128_integer()? != 0,
        ))),
        BinaryOp::LogicalOr => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? != 0 || right.to_i128_integer()? != 0,
        ))),
    }
}
