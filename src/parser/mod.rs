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
    Pointer,
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
    Int(i64),
    IntArray(usize),
    IntConstant(String),
    PointerNull,
    PointerArray(usize),
    UnsignedCharArray(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub return_type: ReturnType,
    pub parameters: Vec<Parameter>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Empty,
    Block(Vec<Self>),
    Declaration {
        scalar_type: ScalarType,
        name: String,
        initializer: Option<Expr>,
    },
    DeclarationList(Vec<Self>),
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
    Expression(Expr),
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
    let mut parser = Parser { tokens, index: 0 };
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
        if let Some(global) = parse_supported_global_declaration(item_tokens)? {
            globals.push(global);
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
        if self.current_identifier_starts_assignment() {
            self.assignment_statement(true)
        } else {
            self.expression_statement(true)
        }
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
        let type_includes_char = self.consume_declaration_type(base_type)?;
        let mut declarations = Vec::new();
        loop {
            let mut scalar_type = base_type;
            while self.check_punctuator("*") {
                self.advance();
                scalar_type = ScalarType::Pointer;
            }
            let name = self.expect_identifier()?;
            let initializer = if self.check_punctuator("[") {
                if !type_includes_char || scalar_type != ScalarType::Int {
                    return Err(CompileError::new(
                        "only local char arrays with string initializers are supported",
                    ));
                }
                self.advance();
                if !self.check_punctuator("]") {
                    let _size = self.expression()?;
                }
                self.expect_punctuator("]")?;
                self.expect_punctuator("=")?;
                let initializer = self.expression()?;
                if !matches!(initializer, Expr::StringLiteral(_)) {
                    return Err(CompileError::new(
                        "local char arrays require string literal initializers",
                    ));
                }
                scalar_type = ScalarType::Pointer;
                Some(initializer)
            } else if self.check_punctuator("=") {
                self.advance();
                Some(self.expression()?)
            } else {
                None
            };
            declarations.push(Statement::Declaration {
                scalar_type,
                name,
                initializer,
            });
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
            Some(Box::new(self.expression_statement(true)?))
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
            Some(Box::new(self.expression_statement(false)?))
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
        if let Some((target, next_index)) = self.cast_type_at_current() {
            self.index = next_index;
            return Ok(Expr::Cast {
                target,
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
                && let Some(size) = sizeof_type(&self.tokens[start..close])
            {
                self.index = close + 1;
                return i64::try_from(size)
                    .map(Expr::Integer)
                    .map_err(|_| CompileError::new("sizeof result does not fit i64"));
            }
        }
        let _expr = self.unary()?;
        Ok(Expr::Integer(4))
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

    fn cast_type_at_current(&self) -> Option<(ScalarType, usize)> {
        if !self.check_punctuator("(") {
            return None;
        }
        let start = self.index + 1;
        let close = self.tokens[start..]
            .iter()
            .position(|token| token_is_punctuator(token, ")"))?
            + start;
        let target = supported_cast_type(&self.tokens[start..close])?;
        Some((target, close + 1))
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
                TokenKind::StringLiteral(value) => {
                    let value = value.clone();
                    self.advance();
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
        let mut long_count = 0usize;
        while let Some(token) = self.tokens.get(index) {
            match &token.kind {
                TokenKind::Keyword(
                    Keyword::Const
                    | Keyword::Register
                    | Keyword::Restrict
                    | Keyword::Signed
                    | Keyword::Unsigned
                    | Keyword::Volatile,
                ) => {}
                TokenKind::Keyword(Keyword::Char | Keyword::Int | Keyword::Short) => {
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
                    let scalar_type = supported_typedef_scalar(name)?;
                    if scalar_type != ScalarType::Int {
                        return None;
                    }
                    saw_type = true;
                }
                _ => break,
            }
            index += 1;
        }
        if !saw_type {
            return None;
        }
        if saw_double && long_count == 0 {
            Some((ScalarType::Double, index))
        } else if long_count == 0 {
            Some((ScalarType::Int, index))
        } else {
            Some((ScalarType::LongLong, index))
        }
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
    if !token_has_keyword(tokens, Keyword::Typedef) || !token_has_keyword(tokens, Keyword::Struct) {
        return Ok(None);
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
    let Some(first_name_index) = previous_identifier_index(first, first.len()) else {
        return Ok(false);
    };
    let base_specifiers = &first[..first_name_index];
    let Some(base_type) = struct_field_type(base_specifiers, known_structs) else {
        return Ok(false);
    };
    for (range_index, (start, end)) in ranges.iter().copied().enumerate() {
        let segment = &tokens[start..end];
        let Some(name_index) = previous_identifier_index(segment, segment.len()) else {
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
            FieldType::Pointer
        } else {
            base_type.clone()
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

fn struct_field_type(tokens: &[Token], known_structs: &[StructLayout]) -> Option<FieldType> {
    if tokens.iter().any(|token| token_is_punctuator(token, "*")) {
        return Some(FieldType::Pointer);
    }
    if let Some(scalar_type) = integer_parameter_type(tokens) {
        return Some(FieldType::Scalar(scalar_type));
    }
    let name = tokens.iter().rev().find_map(token_identifier)?;
    if known_structs.iter().any(|layout| layout.name == name) {
        return Some(FieldType::Struct(name.to_owned()));
    }
    supported_typedef_scalar(name).map(FieldType::Scalar)
}

fn field_type_size(field_type: &FieldType, known_structs: &[StructLayout]) -> CompileResult<usize> {
    match field_type {
        FieldType::Scalar(scalar_type) => Ok(scalar_size_for_layout(*scalar_type)),
        FieldType::Pointer => Ok(8),
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
    field_type_size(field_type, known_structs).map(|size| size.clamp(1, 8))
}

const fn scalar_size_for_layout(scalar_type: ScalarType) -> usize {
    match scalar_type {
        ScalarType::Int => 4,
        ScalarType::LongLong | ScalarType::Double | ScalarType::Pointer => 8,
    }
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

fn parse_supported_global_declaration(tokens: &[Token]) -> CompileResult<Option<Global>> {
    if last_token_is_punctuator(tokens, "}") || !last_token_is_punctuator(tokens, ";") {
        return Ok(None);
    }
    if top_level_function_open_paren(tokens).is_some() {
        return Ok(None);
    }
    if let Some(global) = parse_global_unsigned_char_array(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer_array(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_int_array(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_extern_scalar(tokens)? {
        return Ok(Some(global));
    }
    if let Some(global) = parse_global_pointer(tokens)? {
        return Ok(Some(global));
    }
    parse_global_int(tokens)
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
    Ok(Some(Global {
        name,
        initializer: GlobalInitializer::PointerArray(length),
    }))
}

fn parse_global_int_array(tokens: &[Token]) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(open_bracket) = top_level_punctuator_index(declaration, "[") else {
        return Ok(None);
    };
    let Some(name_index) = previous_identifier_index(declaration, open_bracket) else {
        return Ok(None);
    };
    if !global_specifiers_are_int(&declaration[..name_index]) {
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
    if top_level_punctuator_index(&declaration[close_bracket + 1..], "=").is_some() {
        return Ok(None);
    }
    let length = parse_unsigned_char_array_length(&declaration[open_bracket + 1..close_bracket])?;
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global int-array name"))?
        .to_owned();
    Ok(Some(Global {
        name,
        initializer: GlobalInitializer::IntArray(length),
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
    let scalar_type = if global_specifiers_are_extern_pointer(&declaration[..name_index]) {
        ScalarType::Pointer
    } else if global_specifiers_are_extern_int(&declaration[..name_index]) {
        ScalarType::Int
    } else {
        return Ok(None);
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected extern global name"))?
        .to_owned();
    Ok(Some(Global {
        name,
        initializer: GlobalInitializer::Extern(scalar_type),
    }))
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
    if end_index != declaration.len() {
        let Ok(value) = parse_integer_initializer(&declaration[end_index + 1..]) else {
            return Ok(None);
        };
        if value != 0 {
            return Ok(None);
        }
    }
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global pointer name"))?
        .to_owned();
    Ok(Some(Global {
        name,
        initializer: GlobalInitializer::PointerNull,
    }))
}

fn parse_global_int(tokens: &[Token]) -> CompileResult<Option<Global>> {
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
    if !global_specifiers_are_int(&declaration[..name_index]) {
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

fn global_specifiers_are_unsigned_char(tokens: &[Token]) -> bool {
    let mut saw_unsigned = false;
    let mut saw_char = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Static | Keyword::Const | Keyword::Volatile) => {}
            TokenKind::Keyword(Keyword::Unsigned) => saw_unsigned = true,
            TokenKind::Keyword(Keyword::Char) => saw_char = true,
            _ => return false,
        }
    }
    saw_unsigned && saw_char
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
            ) => saw_type = true,
            TokenKind::Identifier(name) => {
                if supported_typedef_scalar(name).is_none() {
                    return false;
                }
                saw_type = true;
            }
            TokenKind::Punctuator(value) if value == "*" => saw_pointer = true,
            _ => return false,
        }
    }
    saw_type && saw_pointer
}

fn global_specifiers_are_int(tokens: &[Token]) -> bool {
    !token_has_keyword(tokens, Keyword::Extern) && global_specifiers_are_int_like(tokens, false)
}

fn global_specifiers_are_extern_int(tokens: &[Token]) -> bool {
    token_has_keyword(tokens, Keyword::Extern) && global_specifiers_are_int_like(tokens, true)
}

fn global_specifiers_are_int_like(tokens: &[Token], allow_extern: bool) -> bool {
    let mut saw_int = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Keyword(Keyword::Extern) if allow_extern => {}
            TokenKind::Keyword(
                Keyword::Static | Keyword::Const | Keyword::Volatile | Keyword::Signed,
            ) => {}
            TokenKind::Keyword(Keyword::Int) | TokenKind::Identifier(_) => saw_int = true,
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
    let mut parser = Parser { tokens, index: 0 };
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
        Expr::Cast { target, expr } => {
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
        "boolean" | "byte" | "fixed_t" | "lighttable_t" => Some(ScalarType::Int),
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
