use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub structs: Vec<StructLayout>,
    pub constants: Vec<Constant>,
    pub globals: Vec<Global>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLayout {
    pub name: String,
    pub fields: Vec<StructField>,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: String,
    pub field_type: FieldType,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    Scalar(ScalarType),
    Struct(String),
    Pointer {
        referent: Option<String>,
    },
    Array {
        element_type: ScalarType,
        length: usize,
    },
    StructArray {
        struct_name: String,
        length: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constant {
    pub name: String,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Global {
    pub name: String,
    pub initializer: GlobalInitializer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalInitializer {
    Extern(ScalarType),
    ExternPointer {
        referent: Option<String>,
    },
    ExternIntArray,
    ExternPointerArray {
        referent: Option<String>,
    },
    ExternStructArray {
        struct_name: String,
    },
    Int(i64),
    IntArray(Vec<i32>),
    IntConstant(String),
    PointerNull {
        referent: Option<String>,
    },
    PointerString {
        referent: Option<String>,
        value: String,
    },
    PointerArray {
        referent: Option<String>,
        length: usize,
    },
    PointerStringArray {
        referent: Option<String>,
        values: Vec<String>,
    },
    StructObject {
        struct_name: String,
    },
    StructArray {
        struct_name: String,
        length: usize,
    },
    UnsignedCharArray(Vec<u8>),
}

impl GlobalInitializer {
    const fn is_extern(&self) -> bool {
        matches!(
            self,
            Self::Extern(_)
                | Self::ExternPointer { .. }
                | Self::ExternIntArray
                | Self::ExternPointerArray { .. }
                | Self::ExternStructArray { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub return_type: ReturnType,
    pub parameters: Vec<Parameter>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchCase {
    pub value: Expr,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub scalar_type: ScalarType,
    pub referent: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnType {
    Int,
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    Int,
    LongLong,
    Double,
    Pointer,
}

const POINTER_REFERENT: &str = "*";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Empty,
    Block(Vec<Self>),
    Declaration {
        scalar_type: ScalarType,
        name: String,
        referent: Option<String>,
        initializer: Option<Expr>,
    },
    LocalCharArray {
        name: String,
        length: usize,
        initializer: Option<String>,
    },
    LocalCharMatrix {
        name: String,
        rows: usize,
        columns: usize,
        initializer: Option<Vec<String>>,
    },
    LocalIntArray {
        name: String,
        length: usize,
        initializer: Option<Vec<i32>>,
    },
    LocalPointerArray {
        name: String,
        length: usize,
    },
    LocalStruct {
        name: String,
        struct_name: String,
    },
    LocalConstants(Vec<Constant>),
    DeclarationList(Vec<Self>),
    ExpressionList(Vec<Self>),
    ExternGlobal(Global),
    Label(String),
    Goto(String),
    Assignment {
        target: LValue,
        value: Expr,
    },
    If {
        condition: Expr,
        then_branch: Box<Self>,
        else_branch: Option<Box<Self>>,
    },
    While {
        condition: Expr,
        body: Box<Self>,
    },
    DoWhile {
        body: Box<Self>,
        condition: Expr,
    },
    For {
        initializer: Option<Box<Self>>,
        condition: Option<Expr>,
        post: Option<Box<Self>>,
        body: Box<Self>,
    },
    Switch {
        condition: Expr,
        cases: Vec<SwitchCase>,
        default: Vec<Self>,
    },
    Expression(Expr),
    Break,
    Continue,
    Return(Option<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LValue {
    Identifier(String),
    Subscript {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Member {
        base: Box<Expr>,
        field: String,
        dereference: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Call {
        callee: String,
        args: Vec<Self>,
    },
    Identifier(String),
    Integer(i64),
    DoubleLiteral(String),
    StringLiteral(String),
    SizeOfExpr {
        expr: Box<Self>,
    },
    Subscript {
        array: Box<Self>,
        index: Box<Self>,
    },
    Dereference {
        pointer: Box<Self>,
    },
    AddressOf {
        target: LValue,
    },
    Member {
        base: Box<Self>,
        field: String,
        dereference: bool,
    },
    Assignment {
        target: LValue,
        value: Box<Self>,
    },
    PostIncrement {
        target: LValue,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Self>,
    },
    Cast {
        target: ScalarType,
        referent: Option<String>,
        expr: Box<Self>,
    },
    Conditional {
        condition: Box<Self>,
        then_expr: Box<Self>,
        else_expr: Box<Self>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Self>,
        right: Box<Self>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    BitNot,
    LogicalNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    ShiftLeft,
    ShiftRight,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    LogicalAnd,
    LogicalOr,
    BitAnd,
    BitXor,
    BitOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentOperator {
    Simple,
    Compound(BinaryOp),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceTranslationUnit {
    pub items: Vec<ExternalItem>,
}

impl SurfaceTranslationUnit {
    #[must_use]
    pub fn typedef_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, ExternalItem::Typedef { .. }))
            .count()
    }

    #[must_use]
    pub fn prototype_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, ExternalItem::Prototype { .. }))
            .count()
    }

    #[must_use]
    pub fn declaration_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, ExternalItem::Declaration { .. }))
            .count()
    }

    #[must_use]
    pub fn function_definition_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, ExternalItem::FunctionDefinition { .. }))
            .count()
    }

    #[must_use]
    pub fn struct_forward_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, ExternalItem::StructForward { .. }))
            .count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalItem {
    Typedef { name: String },
    Declaration { name: String },
    Prototype { name: String },
    FunctionDefinition { name: String },
    StructForward { name: String },
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
    let mut parser = SurfaceParser { tokens, index: 0 };
    parser.translation_unit()
}

/// Parses supported executable functions from a full translation unit.
///
/// # Errors
///
/// Returns an error when the translation unit contains a function definition
/// outside the supported executable subset.
pub fn parse_supported_translation_unit(tokens: &[Token]) -> CompileResult<Program> {
    let mut parser = SurfaceParser { tokens, index: 0 };
    let external_items = parser.external_token_groups()?;
    let mut structs = Vec::new();
    let mut constants = Vec::new();
    let mut globals = Vec::new();
    let mut functions = Vec::new();
    let mut unsupported_data_declaration = false;
    for item_tokens in &external_items {
        if let Some(layout) = parse_struct_typedef(item_tokens, &structs)? {
            structs.push(layout);
            continue;
        }
        let enum_constants = parse_enum_constants(item_tokens)?;
        if !enum_constants.is_empty() {
            constants.extend(enum_constants);
            continue;
        }
        let parsed_globals = parse_supported_global_declarations(item_tokens, &structs)?;
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
        if !function_definition_has_supported_signature(item_tokens) {
            let Some(token) = item_tokens.first() else {
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
            tokens: item_tokens,
            index: 0,
            known_structs: &structs,
        };
        functions.push(function_parser.function()?);
        if !function_parser.check_end() {
            return Err(CompileError::new(format!(
                "trailing tokens after function definition: {name}"
            )));
        }
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
        functions,
    })
}

struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
    known_structs: &'a [StructLayout],
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
            functions,
        })
    }

    fn function(&mut self) -> CompileResult<Function> {
        let (return_type, name) = self.function_signature()?;
        let parameters = self.parameter_list()?;
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

    fn parameter_list(&mut self) -> CompileResult<Vec<Parameter>> {
        let mut parameters = Vec::new();
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
                    self.push_parameter(&mut parameters, parameter_start, self.index)?;
                    self.advance();
                    return Ok(parameters);
                }
                depth -= 1;
                self.advance();
                continue;
            }
            if depth == 0 && self.check_punctuator(",") {
                self.push_parameter(&mut parameters, parameter_start, self.index)?;
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
    ) -> CompileResult<()> {
        let tokens = &self.tokens[start..end];
        if parameter_is_void(tokens) {
            return Ok(());
        }
        let Some(name) = tokens.iter().rev().find_map(token_identifier) else {
            return Err(CompileError::new("unsupported function parameter"));
        };
        let scalar_type = parameter_scalar_type(tokens)
            .ok_or_else(|| CompileError::new("unsupported function parameter"))?;
        parameters.push(Parameter {
            name: name.to_owned(),
            scalar_type,
            referent: pointer_referent_type(tokens),
        });
        Ok(())
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
        if let Some(statement) = self.static_aggregate_declaration()? {
            return Ok(statement);
        }
        if let Some(statement) = self.local_enum_declaration()? {
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
        let base_referent = declaration_base_referent_type(&self.tokens[self.index..type_end]);
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
                self.local_array_declaration(type_includes_char, scalar_type, name)?
            } else if self.check_punctuator("=") {
                self.advance();
                Statement::Declaration {
                    scalar_type,
                    name,
                    referent,
                    initializer: Some(self.expression()?),
                }
            } else {
                Statement::Declaration {
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
        scalar_type: ScalarType,
        name: String,
    ) -> CompileResult<Statement> {
        self.advance();
        let explicit_length = if self.check_punctuator("]") {
            None
        } else {
            Some(local_array_length(&self.expression()?)?)
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
        self.local_int_array_declaration(name, explicit_length)
    }

    fn local_char_array_declaration(
        &mut self,
        name: String,
        explicit_length: Option<usize>,
    ) -> CompileResult<Statement> {
        let initializer = if self.check_punctuator("=") {
            self.advance();
            let initializer = self.expression()?;
            let Expr::StringLiteral(value) = initializer else {
                return Err(CompileError::new(
                    "local char arrays require string literal initializers",
                ));
            };
            Some(value)
        } else {
            None
        };
        let length = match (explicit_length, &initializer) {
            (Some(length), _) => length,
            (None, Some(value)) => inferred_local_char_array_length(value)?,
            (None, None) => {
                return Err(CompileError::new(
                    "local char arrays require a size or string literal initializer",
                ));
            }
        };
        if let Some(value) = &initializer {
            validate_local_char_array_initializer(value, length)?;
        }
        Ok(Statement::LocalCharArray {
            name,
            length,
            initializer,
        })
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
        let columns = local_array_length(&self.expression()?)?;
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
        Ok(Statement::LocalPointerArray { name, length })
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
        let Some(global) = parse_supported_global_declaration(declaration, self.known_structs)?
        else {
            return Ok(None);
        };
        if !global.initializer.is_extern() {
            return Ok(None);
        }
        self.index += semicolon_index + 1;
        Ok(Some(Statement::ExternGlobal(global)))
    }

    fn local_struct_declaration(&mut self) -> CompileResult<Option<Statement>> {
        let type_index = if self.check_keyword(Keyword::Static) {
            self.index + 1
        } else {
            self.index
        };
        let Some(struct_name) = self.local_struct_name_at(type_index) else {
            return Ok(None);
        };
        self.index = type_index + 1;
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

    fn local_struct_name_at(&self, index: usize) -> Option<String> {
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
        Some(name.clone())
    }

    fn local_int_array_initializer(&mut self) -> CompileResult<Vec<i32>> {
        self.expect_punctuator("{")?;
        let mut values = Vec::new();
        if self.check_punctuator("}") {
            self.advance();
            return Ok(values);
        }
        loop {
            let value = eval_integer_initializer_expr(&self.expression()?)?.to_i64_trunc()?;
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
        let mut next_value = 0i64;
        while !self.check_punctuator("}") {
            let name = self.expect_identifier()?;
            let value = if self.check_punctuator("=") {
                self.advance();
                eval_integer_initializer_expr(&self.expression()?)?.to_i64_trunc()?
            } else {
                next_value
            };
            next_value = value
                .checked_add(1)
                .ok_or_else(|| CompileError::new("enum constant overflow"))?;
            constants.push(Constant { name, value });
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
            if self.check_punctuator("++") {
                self.advance();
                expr = Expr::PostIncrement {
                    target: lvalue_from_expr(expr)?,
                };
                continue;
            }
            if self.check_punctuator("--") {
                self.advance();
                let target = lvalue_from_expr(expr.clone())?;
                expr = Expr::Assignment {
                    target,
                    value: Box::new(Expr::Binary {
                        op: BinaryOp::Sub,
                        left: Box::new(expr),
                        right: Box::new(Expr::Integer(1)),
                    }),
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
        let close = self.tokens[start..]
            .iter()
            .position(|token| token_is_punctuator(token, ")"))?
            + start;
        let cast_tokens = &self.tokens[start..close];
        let target = supported_cast_type(cast_tokens)?;
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
                TokenKind::Keyword(Keyword::Double) => {
                    saw_type = true;
                    saw_double = true;
                }
                TokenKind::Keyword(Keyword::Long) => {
                    saw_type = true;
                    long_count += 1;
                }
                TokenKind::Identifier(name) => {
                    if saw_type {
                        break;
                    }
                    if let Some(scalar_type) = supported_typedef_scalar(name) {
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
        if saw_struct_pointer {
            Some((ScalarType::Pointer, index))
        } else if saw_double && long_count == 0 {
            Some((ScalarType::Double, index))
        } else if long_count == 0 {
            Some((ScalarType::Int, index))
        } else {
            Some((ScalarType::LongLong, index))
        }
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

fn parameter_scalar_type(tokens: &[Token]) -> Option<ScalarType> {
    if parameter_has_pointer(tokens) {
        return Some(ScalarType::Pointer);
    }
    let name_index = tokens
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, token)| token_identifier(token).map(|_name| index))?;
    integer_parameter_type(&tokens[..name_index])
}

fn parameter_has_pointer(tokens: &[Token]) -> bool {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for token in tokens {
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && token_is_punctuator(token, "*")
        {
            return true;
        }
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    false
}

fn pointer_referent_type(tokens: &[Token]) -> Option<String> {
    if !parameter_has_pointer(tokens) {
        return None;
    }
    let name_index = tokens
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, token)| token_identifier(token).map(|_name| index))?;
    tokens[..name_index]
        .iter()
        .rev()
        .find_map(token_identifier)
        .filter(|name| supported_typedef_scalar(name).is_none())
        .map(ToOwned::to_owned)
}

fn declaration_base_referent_type(tokens: &[Token]) -> Option<String> {
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Char)))
    {
        return Some("char".to_owned());
    }
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Int)))
    {
        return Some("int".to_owned());
    }
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Void)))
    {
        return Some("void".to_owned());
    }
    tokens
        .iter()
        .rev()
        .find_map(token_identifier)
        .filter(|name| supported_typedef_scalar(name).is_none())
        .map(ToOwned::to_owned)
}

fn integer_parameter_type(tokens: &[Token]) -> Option<ScalarType> {
    let mut saw_type = false;
    let mut long_count = 0usize;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(
                Keyword::Const | Keyword::Register | Keyword::Restrict | Keyword::Volatile,
            ) => {}
            TokenKind::Keyword(
                Keyword::Char | Keyword::Int | Keyword::Short | Keyword::Signed | Keyword::Unsigned,
            ) => saw_type = true,
            TokenKind::Keyword(Keyword::Long) => {
                saw_type = true;
                long_count += 1;
            }
            TokenKind::Identifier(name) => {
                let scalar_type = supported_typedef_scalar(name)?;
                if scalar_type != ScalarType::Int {
                    return None;
                }
                saw_type = true;
            }
            _ => return None,
        }
    }
    if !saw_type {
        return None;
    }
    if long_count == 0 {
        Some(ScalarType::Int)
    } else {
        Some(ScalarType::LongLong)
    }
}

fn parse_enum_constants(tokens: &[Token]) -> CompileResult<Vec<Constant>> {
    if !tokens_start_enum_declaration(tokens) {
        return Ok(Vec::new());
    }
    let Some(open_brace) = top_level_punctuator_index(tokens, "{") else {
        return Ok(Vec::new());
    };
    let Some(close_brace) = matching_top_level_brace(tokens, open_brace) else {
        return Err(CompileError::new("unterminated enum declaration")
            .at(tokens[open_brace].line, tokens[open_brace].column));
    };
    parse_enum_body(&tokens[open_brace + 1..close_brace])
}

fn parse_struct_typedef(
    tokens: &[Token],
    known_structs: &[StructLayout],
) -> CompileResult<Option<StructLayout>> {
    if !token_has_keyword(tokens, Keyword::Typedef) {
        return Ok(None);
    }
    if !token_has_keyword(tokens, Keyword::Struct) {
        return Ok(parse_struct_alias_typedef(tokens, known_structs));
    }
    let Some(open_brace) = top_level_punctuator_index(tokens, "{") else {
        return Ok(None);
    };
    let Some(close_brace) = matching_top_level_brace(tokens, open_brace) else {
        return Ok(None);
    };
    let Some(name) = last_top_level_identifier(tokens) else {
        return Ok(None);
    };
    let Some((fields, size)) =
        parse_struct_fields(&tokens[open_brace + 1..close_brace], known_structs)?
    else {
        return Ok(None);
    };
    Ok(Some(StructLayout { name, fields, size }))
}

fn parse_struct_alias_typedef(
    tokens: &[Token],
    known_structs: &[StructLayout],
) -> Option<StructLayout> {
    let names = tokens
        .iter()
        .filter_map(token_identifier)
        .collect::<Vec<_>>();
    let [source, alias] = names.as_slice() else {
        return None;
    };
    known_structs
        .iter()
        .find(|layout| layout.name == *source)
        .map(|layout| StructLayout {
            name: (*alias).to_owned(),
            fields: layout.fields.clone(),
            size: layout.size,
        })
}

fn parse_struct_fields(
    tokens: &[Token],
    known_structs: &[StructLayout],
) -> CompileResult<Option<(Vec<StructField>, usize)>> {
    let mut fields = Vec::new();
    let mut offset = 0usize;
    let mut max_alignment = 1usize;
    let mut start = 0usize;
    for index in 0..tokens.len() {
        if !token_is_punctuator(&tokens[index], ";") {
            continue;
        }
        let declaration = &tokens[start..index];
        if !declaration.is_empty()
            && !parse_struct_field_declaration(
                declaration,
                known_structs,
                &mut fields,
                &mut offset,
                &mut max_alignment,
            )?
        {
            return Ok(None);
        }
        start = index + 1;
    }
    if start < tokens.len() {
        return Ok(None);
    }
    let size = align_struct_offset(offset, max_alignment)?;
    Ok(Some((fields, size)))
}

fn parse_struct_field_declaration(
    tokens: &[Token],
    known_structs: &[StructLayout],
    fields: &mut Vec<StructField>,
    offset: &mut usize,
    max_alignment: &mut usize,
) -> CompileResult<bool> {
    let ranges = top_level_comma_ranges(tokens);
    let Some((first_start, first_end)) = ranges.first().copied() else {
        return Ok(false);
    };
    let first = &tokens[first_start..first_end];
    let Some(first_name_index) = declarator_name_index(first) else {
        return Ok(false);
    };
    let base_specifiers = &first[..first_name_index];
    let Some(base_type) = struct_field_type(base_specifiers, known_structs) else {
        return Ok(false);
    };
    for (range_index, (start, end)) in ranges.iter().copied().enumerate() {
        let segment = &tokens[start..end];
        let Some(name_index) = declarator_name_index(segment) else {
            return Ok(false);
        };
        let Some(name) = token_identifier(&segment[name_index]) else {
            return Ok(false);
        };
        let field_type = if range_index == 0 {
            base_type.clone()
        } else if segment[..name_index]
            .iter()
            .any(|token| token_is_punctuator(token, "*"))
        {
            FieldType::Pointer {
                referent: pointer_referent_from_specifiers(&segment[..name_index]),
            }
        } else {
            base_type.clone()
        };
        let field_type = if let Some(length) = struct_field_array_length(segment) {
            match field_type {
                FieldType::Scalar(element_type) => FieldType::Array {
                    element_type,
                    length,
                },
                FieldType::Pointer { .. } => FieldType::Array {
                    element_type: ScalarType::Pointer,
                    length,
                },
                FieldType::Struct(struct_name) => FieldType::StructArray {
                    struct_name,
                    length,
                },
                FieldType::Array { .. } | FieldType::StructArray { .. } => field_type,
            }
        } else {
            field_type
        };
        let size = field_type_size(&field_type, known_structs)?;
        let alignment = field_type_alignment(&field_type, known_structs)?;
        *max_alignment = (*max_alignment).max(alignment);
        *offset = align_struct_offset(*offset, alignment)?;
        fields.push(StructField {
            name: name.to_owned(),
            field_type,
            offset: *offset,
        });
        *offset = offset
            .checked_add(size)
            .ok_or_else(|| CompileError::new("struct size overflow"))?;
    }
    Ok(true)
}

fn top_level_comma_ranges(tokens: &[Token]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if token_is_punctuator(token, ",") {
            ranges.push((start, index));
            start = index + 1;
        }
    }
    ranges.push((start, tokens.len()));
    ranges
}

fn declarator_name_index(tokens: &[Token]) -> Option<usize> {
    let before = top_level_punctuator_index(tokens, "[").unwrap_or(tokens.len());
    previous_identifier_index(tokens, before)
}

fn struct_field_array_length(tokens: &[Token]) -> Option<usize> {
    let open_bracket = top_level_punctuator_index(tokens, "[")?;
    let close_bracket = matching_top_level_bracket(tokens, open_bracket)?;
    match &tokens.get(open_bracket + 1)?.kind {
        TokenKind::Integer(value) => usize::try_from(*value).ok().filter(|length| *length > 0),
        _ => Some(1),
    }
    .filter(|_length| close_bracket > open_bracket)
}

fn struct_field_type(tokens: &[Token], known_structs: &[StructLayout]) -> Option<FieldType> {
    if tokens.iter().any(|token| token_is_punctuator(token, "*")) {
        return Some(FieldType::Pointer {
            referent: pointer_referent_from_specifiers(tokens),
        });
    }
    if let Some(scalar_type) = integer_parameter_type(tokens) {
        return Some(FieldType::Scalar(scalar_type));
    }
    let name = tokens.iter().rev().find_map(token_identifier)?;
    if known_structs.iter().any(|layout| layout.name == name) {
        return Some(FieldType::Struct(name.to_owned()));
    }
    Some(FieldType::Scalar(
        supported_typedef_scalar(name).unwrap_or(ScalarType::Int),
    ))
}

fn field_type_size(field_type: &FieldType, known_structs: &[StructLayout]) -> CompileResult<usize> {
    match field_type {
        FieldType::Scalar(scalar_type) => Ok(scalar_size_for_layout(*scalar_type)),
        FieldType::Pointer { .. } => Ok(8),
        FieldType::Array {
            element_type,
            length,
        } => scalar_size_for_layout(*element_type)
            .checked_mul(*length)
            .ok_or_else(|| CompileError::new("struct array field size overflow")),
        FieldType::StructArray {
            struct_name,
            length,
        } => known_structs
            .iter()
            .find(|layout| layout.name == *struct_name)
            .map(|layout| layout.size)
            .ok_or_else(|| CompileError::new(format!("unknown struct field type: {struct_name}")))?
            .checked_mul(*length)
            .ok_or_else(|| CompileError::new("struct array field size overflow")),
        FieldType::Struct(name) => known_structs
            .iter()
            .find(|layout| layout.name == *name)
            .map(|layout| layout.size)
            .ok_or_else(|| CompileError::new(format!("unknown struct field type: {name}"))),
    }
}

fn field_type_alignment(
    field_type: &FieldType,
    known_structs: &[StructLayout],
) -> CompileResult<usize> {
    match field_type {
        FieldType::Array { element_type, .. } => Ok(scalar_size_for_layout(*element_type)),
        FieldType::StructArray { struct_name, .. } => known_structs
            .iter()
            .find(|layout| layout.name == *struct_name)
            .map(|layout| layout.size.clamp(1, 8))
            .ok_or_else(|| CompileError::new(format!("unknown struct field type: {struct_name}"))),
        _ => field_type_size(field_type, known_structs).map(|size| size.clamp(1, 8)),
    }
}

const fn scalar_size_for_layout(scalar_type: ScalarType) -> usize {
    match scalar_type {
        ScalarType::Int => 4,
        ScalarType::LongLong | ScalarType::Double | ScalarType::Pointer => 8,
    }
}

fn local_array_length(expr: &Expr) -> CompileResult<usize> {
    let value = eval_integer_initializer_expr(expr)?.to_i64_trunc()?;
    if value <= 0 {
        return Err(CompileError::new("local char array size must be positive"));
    }
    usize::try_from(value).map_err(|_| CompileError::new("local char array size is too large"))
}

fn inferred_local_char_array_length(value: &str) -> CompileResult<usize> {
    value
        .len()
        .checked_add(1)
        .ok_or_else(|| CompileError::new("local char array size overflow"))
}

fn validate_local_char_array_initializer(value: &str, length: usize) -> CompileResult<()> {
    if value.len() > length {
        return Err(CompileError::new(
            "local char array initializer is too large",
        ));
    }
    Ok(())
}

fn align_struct_offset(offset: usize, alignment: usize) -> CompileResult<usize> {
    let remainder = offset % alignment;
    if remainder == 0 {
        return Ok(offset);
    }
    offset
        .checked_add(alignment - remainder)
        .ok_or_else(|| CompileError::new("struct offset overflow"))
}

fn parse_enum_body(tokens: &[Token]) -> CompileResult<Vec<Constant>> {
    let mut constants = Vec::new();
    let mut value = 0i64;
    let mut index = 0usize;
    while index < tokens.len() {
        if token_is_punctuator(&tokens[index], ",") {
            index += 1;
            continue;
        }
        let Some(name) = token_identifier(&tokens[index]) else {
            return Ok(Vec::new());
        };
        index += 1;
        if tokens
            .get(index)
            .is_some_and(|token| token_is_punctuator(token, "="))
        {
            let initializer_start = index + 1;
            let initializer_end = next_enum_separator(tokens, initializer_start);
            value = parse_integer_initializer(&tokens[initializer_start..initializer_end])?;
            index = initializer_end;
        }
        constants.push(Constant {
            name: name.to_owned(),
            value,
        });
        value = value
            .checked_add(1)
            .ok_or_else(|| CompileError::new("enum constant overflow"))?;
    }
    Ok(constants)
}

fn tokens_start_enum_declaration(tokens: &[Token]) -> bool {
    matches!(
        tokens.first().map(|token| &token.kind),
        Some(TokenKind::Keyword(Keyword::Enum))
    ) || matches!(
        (
            tokens.first().map(|token| &token.kind),
            tokens.get(1).map(|token| &token.kind)
        ),
        (
            Some(TokenKind::Keyword(Keyword::Typedef)),
            Some(TokenKind::Keyword(Keyword::Enum))
        )
    )
}

fn next_enum_separator(tokens: &[Token], start: usize) -> usize {
    tokens[start..]
        .iter()
        .position(|token| token_is_punctuator(token, ","))
        .map_or(tokens.len(), |offset| start + offset)
}

fn parse_supported_global_declaration(
    tokens: &[Token],
    known_structs: &[StructLayout],
) -> CompileResult<Option<Global>> {
    if last_token_is_punctuator(tokens, "}") || !last_token_is_punctuator(tokens, ";") {
        return Ok(None);
    }
    if top_level_function_open_paren(tokens).is_some() {
        return Ok(None);
    }
    if let Some(global) = parse_global_unsigned_char_array(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer_string_array(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer_array(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_extern_pointer_array(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_struct_array(tokens, known_structs)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_struct_object(tokens, known_structs)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_extern_int_array(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_int_array(tokens, known_structs)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_extern_scalar(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer(tokens)? {
        return Ok(Some(global));
    }
    parse_global_int(tokens, known_structs)
}

fn parse_supported_global_declarations(
    tokens: &[Token],
    known_structs: &[StructLayout],
) -> CompileResult<Vec<Global>> {
    if let Some(globals) = parse_global_int_declarator_list(tokens, known_structs)? {
        return Ok(globals);
    }
    parse_supported_global_declaration(tokens, known_structs)
        .map(|global| global.map_or_else(Vec::new, |global| vec![global]))
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

fn parse_global_unsigned_char_array(tokens: &[Token]) -> CompileResult<Option<Global>> {
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
    let values = if let Some(assign_index) =
        top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
    {
        let assign_index = close_bracket + 1 + assign_index;
        let Ok(values) = parse_unsigned_char_initializer(&declaration[assign_index + 1..]) else {
            return Ok(None);
        };
        values
    } else {
        let length =
            parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket])?;
        vec![0; length]
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global array name"))?
        .to_owned();
    Ok(Some(Global {
        name,
        initializer: GlobalInitializer::UnsignedCharArray(values),
    }))
}

fn parse_unsigned_char_array_length(tokens: &[Token]) -> CompileResult<usize> {
    match tokens {
        [
            Token {
                kind: TokenKind::Integer(value),
                line,
                column,
            },
        ] => usize::try_from(*value).map_err(|_| {
            CompileError::new("unsigned char array length does not fit usize").at(*line, *column)
        }),
        [first, ..] => {
            Err(CompileError::new("expected unsigned char array length")
                .at(first.line, first.column))
        }
        [] => Err(CompileError::new("expected unsigned char array length")),
    }
}

fn parse_global_pointer_array(tokens: &[Token]) -> CompileResult<Option<Global>> {
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
    let length = parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket])?;
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global pointer-array name"))?
        .to_owned();
    let referent = pointer_referent_from_specifiers(&declaration[..name_index]);
    Ok(Some(Global {
        name,
        initializer: GlobalInitializer::PointerArray { referent, length },
    }))
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
    Ok(Some(Global {
        name,
        initializer: GlobalInitializer::PointerStringArray { referent, values },
    }))
}

fn parse_global_extern_pointer_array(tokens: &[Token]) -> CompileResult<Option<Global>> {
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
    if top_level_punctuator_index(&declaration[close_bracket + 1..], "=").is_some() {
        return Ok(None);
    }
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected extern global pointer-array name"))?
        .to_owned();
    let referent = pointer_referent_from_specifiers(&declaration[..name_index]);
    Ok(Some(Global {
        name,
        initializer: GlobalInitializer::ExternPointerArray { referent },
    }))
}

fn parse_global_struct_array(
    tokens: &[Token],
    known_structs: &[StructLayout],
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
    let assign_index = top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
        .map(|offset| close_bracket + 1 + offset);
    let initializer = if token_has_keyword(&declaration[..name_index], Keyword::Extern) {
        GlobalInitializer::ExternStructArray { struct_name }
    } else {
        let length = if let Some(assign_index) = assign_index {
            let initializer = &declaration[assign_index + 1..];
            aggregate_initializer_length(initializer).ok_or_else(|| {
                CompileError::new("expected global struct-array initializer").at(
                    declaration[assign_index].line,
                    declaration[assign_index].column,
                )
            })?
        } else {
            parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket])?
        };
        GlobalInitializer::StructArray {
            struct_name,
            length,
        }
    };
    Ok(Some(Global { name, initializer }))
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

fn parse_global_struct_object(
    tokens: &[Token],
    known_structs: &[StructLayout],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let end_index = top_level_punctuator_index(declaration, "=").unwrap_or(declaration.len());
    if top_level_punctuator_index(&declaration[..end_index], "[").is_some() {
        return Ok(None);
    }
    if token_has_keyword(declaration, Keyword::Extern) {
        return Ok(None);
    }
    if end_index != declaration.len()
        && !declaration
            .get(end_index + 1)
            .is_some_and(|token| token_is_punctuator(token, "{"))
    {
        return Ok(None);
    }
    let Some(name_index) = previous_identifier_index(declaration, end_index) else {
        return Ok(None);
    };
    let Some(struct_name) = global_struct_specifier_name(&declaration[..name_index], known_structs)
    else {
        return Ok(None);
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global struct object name"))?
        .to_owned();
    Ok(Some(Global {
        name,
        initializer: GlobalInitializer::StructObject { struct_name },
    }))
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
    Ok(Some(Global {
        name,
        initializer: GlobalInitializer::ExternIntArray,
    }))
}

fn parse_global_int_array(
    tokens: &[Token],
    known_structs: &[StructLayout],
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
    let length = parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket])?;
    let values = if let Some(assign_index) =
        top_level_punctuator_index(&declaration[close_bracket + 1..], "=")
    {
        let assign_index = close_bracket + 1 + assign_index;
        parse_int_array_initializer(&declaration[assign_index + 1..], length)?
    } else {
        vec![0; length]
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global int-array name"))?
        .to_owned();
    Ok(Some(Global {
        name,
        initializer: GlobalInitializer::IntArray(values),
    }))
}

fn parse_global_extern_scalar(tokens: &[Token]) -> CompileResult<Option<Global>> {
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
    } else if global_specifiers_are_extern_int(specifiers) {
        GlobalInitializer::Extern(ScalarType::Int)
    } else {
        return Ok(None);
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected extern global name"))?
        .to_owned();
    Ok(Some(Global { name, initializer }))
}

fn parse_global_pointer(tokens: &[Token]) -> CompileResult<Option<Global>> {
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
            return Ok(Some(Global {
                name,
                initializer: GlobalInitializer::PointerString { referent, value },
            }));
        }
        let Ok(value) = parse_integer_initializer(initializer) else {
            return Ok(None);
        };
        if value != 0 {
            return Ok(None);
        }
    }
    Ok(Some(Global {
        name,
        initializer: GlobalInitializer::PointerNull { referent },
    }))
}

fn parse_global_int(
    tokens: &[Token],
    known_structs: &[StructLayout],
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
        parse_global_int_initializer(&declaration[end_index + 1..])?
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global int name"))?
        .to_owned();
    Ok(Some(Global { name, initializer }))
}

fn parse_global_int_declarator_list(
    tokens: &[Token],
    known_structs: &[StructLayout],
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
            parse_global_int_initializer(&segment[end_index + 1..])?
        };
        let name = token_identifier(&segment[name_index])
            .ok_or_else(|| CompileError::new("expected global int name"))?
            .to_owned();
        globals.push(Global { name, initializer });
    }
    Ok(Some(globals))
}

fn global_specifiers_are_unsigned_char(tokens: &[Token]) -> bool {
    let mut saw_char = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(
                Keyword::Static | Keyword::Const | Keyword::Volatile | Keyword::Unsigned,
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

fn pointer_referent_from_specifiers(tokens: &[Token]) -> Option<String> {
    let pointer_depth = tokens
        .iter()
        .filter(|token| token_is_punctuator(token, "*"))
        .count();
    if pointer_depth == 0 {
        return None;
    }
    pointer_referent_for_depth(
        pointer_depth,
        declaration_base_referent_type(tokens).as_deref(),
    )
}

fn pointer_referent_for_depth(pointer_depth: usize, base_referent: Option<&str>) -> Option<String> {
    match pointer_depth {
        0 => None,
        1 => base_referent.map(ToOwned::to_owned),
        depth => {
            let mut referent = POINTER_REFERENT.repeat(depth - 1);
            if let Some(base) = base_referent {
                referent.push_str(base);
            }
            Some(referent)
        }
    }
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
            TokenKind::Keyword(Keyword::Int) => saw_int = true,
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

fn parse_global_int_initializer(tokens: &[Token]) -> CompileResult<GlobalInitializer> {
    if let Ok(value) = parse_integer_initializer(tokens) {
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

fn parse_int_array_initializer(tokens: &[Token], length: usize) -> CompileResult<Vec<i32>> {
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
        let value = parse_integer_initializer(&item[..item_len])?;
        values.push(i32::try_from(value).map_err(|_| {
            CompileError::new("global int array initializer does not fit i32")
                .at(tokens[start].line, tokens[start].column)
        })?);
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    if values.len() > length {
        return Err(CompileError::new("too many global int array initializers")
            .at(first.line, first.column));
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

fn parse_string_initializer(tokens: &[Token]) -> CompileResult<String> {
    if tokens.is_empty() {
        return Err(CompileError::new("expected global string initializer"));
    }
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs: &[],
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

fn parse_unsigned_char_initializer(tokens: &[Token]) -> CompileResult<Vec<u8>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new("expected global array initializer"));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global array initializer").at(first.line, first.column)
        );
    }
    let mut values = Vec::new();
    let mut index = 1usize;
    loop {
        let Some(token) = tokens.get(index) else {
            return Err(CompileError::new("unterminated global array initializer")
                .at(first.line, first.column));
        };
        if token_is_punctuator(token, "}") {
            return Ok(values);
        }
        let value = integer_token_value(token)?;
        let byte = u8::try_from(value).map_err(|_| {
            CompileError::new("unsigned char initializer does not fit u8")
                .at(token.line, token.column)
        })?;
        values.push(byte);
        index += 1;
        let Some(separator) = tokens.get(index) else {
            return Err(CompileError::new("unterminated global array initializer")
                .at(first.line, first.column));
        };
        if token_is_punctuator(separator, ",") {
            index += 1;
            continue;
        }
        if token_is_punctuator(separator, "}") {
            continue;
        }
        return Err(
            CompileError::new("expected global array initializer separator")
                .at(separator.line, separator.column),
        );
    }
}

fn parse_integer_initializer(tokens: &[Token]) -> CompileResult<i64> {
    if tokens.is_empty() {
        return Err(CompileError::new("expected global integer initializer"));
    }
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs: &[],
    };
    let expr = parser.expression()?;
    if let Some(token) = parser.peek() {
        return Err(CompileError::new("unsupported global integer initializer")
            .at(token.line, token.column));
    }
    eval_integer_initializer_expr(&expr)?.to_i64_trunc()
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

fn eval_integer_initializer_expr(expr: &Expr) -> CompileResult<InitializerNumber> {
    match expr {
        Expr::Integer(value) => Ok(InitializerNumber::integer(*value)),
        Expr::DoubleLiteral(value) => InitializerNumber::decimal(value),
        Expr::Unary { op, expr } => {
            let value = eval_integer_initializer_expr(expr)?;
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
            let value = eval_integer_initializer_expr(expr)?;
            match target {
                ScalarType::Int | ScalarType::LongLong | ScalarType::Pointer => {
                    Ok(InitializerNumber::integer(value.to_i64_trunc()?))
                }
                ScalarType::Double => Ok(value),
            }
        }
        Expr::Binary { op, left, right } => {
            let left = eval_integer_initializer_expr(left)?;
            let right = eval_integer_initializer_expr(right)?;
            eval_integer_binary_initializer_expr(*op, left, right)
        }
        Expr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            if eval_integer_initializer_expr(condition)?.to_i128_integer()? == 0 {
                eval_integer_initializer_expr(else_expr)
            } else {
                eval_integer_initializer_expr(then_expr)
            }
        }
        Expr::Identifier(name) => Err(CompileError::new(format!(
            "identifier {name} is not an integer initializer"
        ))),
        Expr::Call { callee, .. } => Err(CompileError::new(format!(
            "call to {callee} is not an integer initializer"
        ))),
        Expr::StringLiteral(_)
        | Expr::SizeOfExpr { .. }
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

fn integer_token_value(token: &Token) -> CompileResult<i64> {
    if let TokenKind::Integer(value) = &token.kind {
        return Ok(*value);
    }
    Err(CompileError::new("expected integer initializer").at(token.line, token.column))
}

fn top_level_punctuator_index(tokens: &[Token], expected: &str) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && token_is_punctuator(token, expected)
        {
            return Some(index);
        }
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    None
}

fn matching_top_level_bracket(tokens: &[Token], open_bracket: usize) -> Option<usize> {
    let mut bracket_depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_bracket) {
        if token_is_punctuator(token, "[") {
            bracket_depth += 1;
            continue;
        }
        if token_is_punctuator(token, "]") {
            bracket_depth = bracket_depth.checked_sub(1)?;
            if bracket_depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn matching_top_level_brace(tokens: &[Token], open_brace: usize) -> Option<usize> {
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_brace) {
        if token_is_punctuator(token, "{") {
            brace_depth += 1;
            continue;
        }
        if token_is_punctuator(token, "}") {
            brace_depth = brace_depth.checked_sub(1)?;
            if brace_depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

struct SurfaceParser<'a> {
    tokens: &'a [Token],
    index: usize,
}

impl SurfaceParser<'_> {
    fn translation_unit(&mut self) -> CompileResult<SurfaceTranslationUnit> {
        let external_items = self.external_token_groups()?;
        let mut items = Vec::new();
        for external_tokens in external_items {
            if let Some(item) = classify_external_item(&external_tokens) {
                items.push(item);
            }
        }
        Ok(SurfaceTranslationUnit { items })
    }

    fn external_token_groups(&mut self) -> CompileResult<Vec<Vec<Token>>> {
        let mut items = Vec::new();
        while !self.check_end() {
            if self.check_punctuator("#") {
                self.skip_directive();
                continue;
            }
            if self.check_punctuator(";") {
                self.advance();
                continue;
            }
            let external_tokens = self.collect_external_item()?;
            items.push(external_tokens);
        }
        Ok(items)
    }

    fn collect_external_item(&mut self) -> CompileResult<Vec<Token>> {
        let start = self.index;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut function_body = false;
        while !self.check_end() {
            let Some(token) = self.peek() else {
                break;
            };
            let is_top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if is_top_level && token_is_punctuator(token, ";") {
                self.advance();
                return Ok(self.tokens[start..self.index].to_vec());
            }
            match &token.kind {
                TokenKind::Punctuator(value) if value == "(" => {
                    paren_depth += 1;
                }
                TokenKind::Punctuator(value) if value == ")" => {
                    decrease_depth(&mut paren_depth, token, "parenthesis")?;
                }
                TokenKind::Punctuator(value) if value == "[" => {
                    bracket_depth += 1;
                }
                TokenKind::Punctuator(value) if value == "]" => {
                    decrease_depth(&mut bracket_depth, token, "bracket")?;
                }
                TokenKind::Punctuator(value) if value == "{" => {
                    if is_top_level
                        && last_token_is_punctuator(&self.tokens[start..self.index], ")")
                    {
                        function_body = true;
                    }
                    brace_depth += 1;
                }
                TokenKind::Punctuator(value) if value == "}" => {
                    decrease_depth(&mut brace_depth, token, "brace")?;
                    self.advance();
                    if function_body && brace_depth == 0 {
                        return Ok(self.tokens[start..self.index].to_vec());
                    }
                    continue;
                }
                _ => {}
            }
            self.advance();
        }
        if paren_depth != 0 || bracket_depth != 0 || brace_depth != 0 {
            let error = CompileError::new("unterminated external declaration");
            let Some(token) = self.tokens.get(start) else {
                return Err(error);
            };
            return Err(error.at(token.line, token.column));
        }
        Ok(self.tokens[start..self.index].to_vec())
    }

    fn skip_directive(&mut self) {
        let Some(start_line) = self.peek().map(|token| token.line) else {
            return;
        };
        while let Some(token) = self.peek() {
            if matches!(token.kind, TokenKind::End) || token.line != start_line {
                return;
            }
            self.advance();
        }
    }

    fn check_punctuator(&self, expected: &str) -> bool {
        matches!(self.peek(), Some(token) if token_is_punctuator(token, expected))
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

fn classify_external_item(tokens: &[Token]) -> Option<ExternalItem> {
    if token_has_keyword(tokens, Keyword::Typedef) {
        return typedef_name(tokens).map(|name| ExternalItem::Typedef { name });
    }
    if let Some(name) = struct_forward_name(tokens) {
        return Some(ExternalItem::StructForward { name });
    }
    if let Some(name) = function_pointer_name(tokens) {
        return Some(ExternalItem::Declaration { name });
    }
    if let Some(name) = normal_function_name(tokens) {
        if last_token_is_punctuator(tokens, "}") {
            return Some(ExternalItem::FunctionDefinition { name });
        }
        return Some(ExternalItem::Prototype { name });
    }
    declaration_name(tokens).map(|name| ExternalItem::Declaration { name })
}

fn function_definition_name(tokens: &[Token]) -> Option<String> {
    if last_token_is_punctuator(tokens, "}") {
        return normal_function_name(tokens);
    }
    None
}

fn function_definition_has_supported_signature(tokens: &[Token]) -> bool {
    let Some(open_index) = top_level_function_open_paren(tokens) else {
        return false;
    };
    let Some(name_index) = previous_identifier_index(tokens, open_index) else {
        return false;
    };
    if supported_return_type(&tokens[..name_index]).is_none() {
        return false;
    }
    let mut paren_depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_index) {
        if token_is_punctuator(token, "(") {
            paren_depth += 1;
            continue;
        }
        if token_is_punctuator(token, ")") {
            if paren_depth == 0 {
                return false;
            }
            paren_depth -= 1;
            if paren_depth == 0 {
                return tokens
                    .get(index + 1)
                    .is_some_and(|next| token_is_punctuator(next, "{"));
            }
        }
    }
    false
}

fn typedef_name(tokens: &[Token]) -> Option<String> {
    function_pointer_name(tokens).or_else(|| last_top_level_identifier(tokens))
}

fn struct_forward_name(tokens: &[Token]) -> Option<String> {
    let meaningful = tokens
        .iter()
        .filter(|token| !token_is_punctuator(token, ";"))
        .collect::<Vec<_>>();
    if meaningful.len() != 2 {
        return None;
    }
    if !token_is_keyword(meaningful[0], Keyword::Struct) {
        return None;
    }
    token_identifier(meaningful[1]).map(ToOwned::to_owned)
}

fn declaration_name(tokens: &[Token]) -> Option<String> {
    function_pointer_name(tokens)
        .or_else(|| array_declarator_name(tokens))
        .or_else(|| last_top_level_identifier(tokens))
}

fn function_pointer_name(tokens: &[Token]) -> Option<String> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && token_is_punctuator(token, "(")
            && tokens
                .get(index + 1)
                .is_some_and(|next| token_is_punctuator(next, "*"))
            && tokens
                .get(index + 3)
                .is_some_and(|next| token_is_punctuator(next, ")"))
        {
            return tokens
                .get(index + 2)
                .and_then(token_identifier)
                .map(ToOwned::to_owned);
        }
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    None
}

fn normal_function_name(tokens: &[Token]) -> Option<String> {
    let open_index = top_level_function_open_paren(tokens)?;
    previous_identifier(tokens, open_index).map(ToOwned::to_owned)
}

fn top_level_function_open_paren(tokens: &[Token]) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut saw_assignment = false;
    for (index, token) in tokens.iter().enumerate() {
        if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 {
            if token_is_punctuator(token, "=") {
                saw_assignment = true;
            }
            if !saw_assignment && token_is_punctuator(token, "(") {
                return Some(index);
            }
        }
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    None
}

fn supported_return_type(tokens: &[Token]) -> Option<ReturnType> {
    let mut saw_void = false;
    let mut saw_non_void_type = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Identifier(_) => saw_non_void_type = true,
            TokenKind::Keyword(keyword) => match keyword {
                Keyword::Auto
                | Keyword::Const
                | Keyword::Extern
                | Keyword::Inline
                | Keyword::Register
                | Keyword::Static
                | Keyword::Volatile => {}
                Keyword::Void => saw_void = true,
                Keyword::Bool
                | Keyword::Char
                | Keyword::Enum
                | Keyword::Int
                | Keyword::Long
                | Keyword::Short
                | Keyword::Signed
                | Keyword::Unsigned => saw_non_void_type = true,
                _ => return None,
            },
            TokenKind::Punctuator(value) if value == "*" => return None,
            TokenKind::Punctuator(_) | TokenKind::Integer(_) | TokenKind::CharLiteral(_) => {
                return None;
            }
            TokenKind::StringLiteral(_) | TokenKind::End => return None,
        }
    }
    match (saw_void, saw_non_void_type) {
        (true, false) => Some(ReturnType::Void),
        (false, true) => Some(ReturnType::Int),
        (true, true) | (false, false) => None,
    }
}

fn supported_cast_type(tokens: &[Token]) -> Option<ScalarType> {
    if tokens.is_empty() {
        return None;
    }
    let mut saw_type = false;
    let mut saw_double = false;
    let mut saw_named_type = false;
    let mut saw_pointer = false;
    let mut long_count = 0usize;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(
                Keyword::Const | Keyword::Restrict | Keyword::Signed | Keyword::Volatile,
            ) => {}
            TokenKind::Keyword(Keyword::Double) => {
                saw_type = true;
                saw_double = true;
            }
            TokenKind::Keyword(
                Keyword::Char | Keyword::Int | Keyword::Short | Keyword::Unsigned | Keyword::Void,
            ) => {
                saw_type = true;
            }
            TokenKind::Keyword(Keyword::Long) => {
                saw_type = true;
                long_count += 1;
            }
            TokenKind::Identifier(name) => {
                if let Some(scalar_type) = supported_typedef_scalar(name) {
                    if scalar_type != ScalarType::Int {
                        return None;
                    }
                } else {
                    saw_named_type = true;
                }
                saw_type = true;
            }
            TokenKind::Punctuator(value) if value == "*" => saw_pointer = true,
            TokenKind::Integer(_)
            | TokenKind::StringLiteral(_)
            | TokenKind::CharLiteral(_)
            | TokenKind::Punctuator(_)
            | TokenKind::End
            | TokenKind::Keyword(_) => return None,
        }
    }
    if !saw_type {
        return None;
    }
    if saw_pointer {
        return Some(ScalarType::Pointer);
    }
    if saw_named_type {
        return None;
    }
    if saw_double && long_count == 0 {
        Some(ScalarType::Double)
    } else if long_count == 0 {
        Some(ScalarType::Int)
    } else {
        Some(ScalarType::LongLong)
    }
}

fn sizeof_type(tokens: &[Token]) -> Option<usize> {
    supported_cast_type(tokens).map(scalar_size_for_layout)
}

fn supported_typedef_scalar(name: &str) -> Option<ScalarType> {
    match name {
        "FILE" | "GameMission_t" | "GameMode_t" | "Language_t" | "ammotype_t" | "angle_t"
        | "boolean" | "buttoncode_t" | "byte" | "card_t" | "cheat_t" | "command_t" | "evtype_t"
        | "fixed_t" | "gameaction_t" | "gamestate_t" | "lighttable_t" | "mobjflag_t"
        | "mobjtype_t" | "playerstate_t" | "powerduration_t" | "powertype_t" | "psprnum_t"
        | "skill_t" | "slopetype_t" | "spritenum_t" | "statenum_t" | "weapontype_t" => {
            Some(ScalarType::Int)
        }
        _ => None,
    }
}

fn parameter_is_void(tokens: &[Token]) -> bool {
    let mut saw_void = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Void) => saw_void = true,
            TokenKind::Keyword(
                Keyword::Const | Keyword::Register | Keyword::Restrict | Keyword::Volatile,
            ) => {}
            _ => return false,
        }
    }
    saw_void
}

fn array_declarator_name(tokens: &[Token]) -> Option<String> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && token_is_punctuator(token, "[")
        {
            return previous_identifier(tokens, index).map(ToOwned::to_owned);
        }
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    None
}

fn last_top_level_identifier(tokens: &[Token]) -> Option<String> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut candidate = None;
    for token in tokens {
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && let Some(identifier) = token_identifier(token)
        {
            candidate = Some(identifier.to_owned());
        }
        update_depths(
            token,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
    }
    candidate
}

fn previous_identifier(tokens: &[Token], before: usize) -> Option<&str> {
    tokens
        .get(..before)?
        .iter()
        .rev()
        .find_map(token_identifier)
}

fn previous_identifier_index(tokens: &[Token], before: usize) -> Option<usize> {
    tokens
        .get(..before)?
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, token)| token_identifier(token).map(|_name| index))
}

fn token_has_keyword(tokens: &[Token], keyword: Keyword) -> bool {
    tokens.iter().any(|token| token_is_keyword(token, keyword))
}

fn token_identifier(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Identifier(value) => Some(value),
        _ => None,
    }
}

fn token_is_keyword(token: &Token, keyword: Keyword) -> bool {
    matches!(&token.kind, TokenKind::Keyword(value) if *value == keyword)
}

fn token_is_punctuator(token: &Token, expected: &str) -> bool {
    matches!(&token.kind, TokenKind::Punctuator(value) if value == expected)
}

fn token_is_assignment_operator(token: &Token) -> bool {
    matches!(
        &token.kind,
        TokenKind::Punctuator(value)
            if matches!(
                value.as_str(),
                "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "<<=" | ">>=" | "&=" | "^=" | "|="
            )
    )
}

fn last_token_is_punctuator(tokens: &[Token], expected: &str) -> bool {
    tokens
        .iter()
        .rev()
        .find(|token| !matches!(token.kind, TokenKind::End))
        .is_some_and(|token| token_is_punctuator(token, expected))
}

fn update_depths(
    token: &Token,
    paren_depth: &mut usize,
    bracket_depth: &mut usize,
    brace_depth: &mut usize,
) {
    match &token.kind {
        TokenKind::Punctuator(value) if value == "(" => *paren_depth += 1,
        TokenKind::Punctuator(value) if value == ")" && *paren_depth > 0 => *paren_depth -= 1,
        TokenKind::Punctuator(value) if value == "[" => *bracket_depth += 1,
        TokenKind::Punctuator(value) if value == "]" && *bracket_depth > 0 => *bracket_depth -= 1,
        TokenKind::Punctuator(value) if value == "{" => *brace_depth += 1,
        TokenKind::Punctuator(value) if value == "}" && *brace_depth > 0 => *brace_depth -= 1,
        _ => {}
    }
}

fn decrease_depth(depth: &mut usize, token: &Token, delimiter: &str) -> CompileResult<()> {
    let Some(next_depth) = depth.checked_sub(1) else {
        return Err(CompileError::new(format!("unmatched closing {delimiter}"))
            .at(token.line, token.column));
    };
    *depth = next_depth;
    Ok(())
}
