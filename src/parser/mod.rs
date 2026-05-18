use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Block(Vec<Self>),
    Declaration {
        name: String,
        initializer: Option<Expr>,
    },
    Assignment {
        name: String,
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
    For {
        initializer: Option<Box<Self>>,
        condition: Option<Expr>,
        post: Option<Box<Self>>,
        body: Box<Self>,
    },
    Return(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Call {
        callee: String,
    },
    Identifier(String),
    Integer(i64),
    Unary {
        op: UnaryOp,
        expr: Box<Self>,
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
    let mut functions = Vec::new();
    for item_tokens in &external_items {
        let Some(name) = function_definition_name(item_tokens) else {
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
    if functions.is_empty() {
        return Err(CompileError::new(
            "translation unit has no supported function definitions",
        ));
    }
    Ok(Program { functions })
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
        Ok(Program { functions })
    }

    fn function(&mut self) -> CompileResult<Function> {
        self.expect_keyword(Keyword::Int)?;
        let name = self.expect_identifier()?;
        self.expect_punctuator("(")?;
        if self.check_keyword(Keyword::Void) {
            self.advance();
        }
        self.expect_punctuator(")")?;
        self.expect_punctuator("{")?;
        let mut statements = Vec::new();
        while !self.check_punctuator("}") {
            statements.push(self.statement()?);
        }
        self.expect_punctuator("}")?;
        Ok(Function { name, statements })
    }

    fn statement(&mut self) -> CompileResult<Statement> {
        if self.check_punctuator("{") {
            return Ok(Statement::Block(self.block_items()?));
        }
        if self.check_keyword(Keyword::Int) {
            return self.declaration_statement();
        }
        if self.check_keyword(Keyword::If) {
            return self.if_statement();
        }
        if self.check_keyword(Keyword::While) {
            return self.while_statement();
        }
        if self.check_keyword(Keyword::For) {
            return self.for_statement();
        }
        if self.check_keyword(Keyword::Return) {
            self.advance();
            let expr = self.expression()?;
            self.expect_punctuator(";")?;
            return Ok(Statement::Return(expr));
        }
        self.assignment_statement(true)
    }

    fn assignment_statement(&mut self, expect_semicolon: bool) -> CompileResult<Statement> {
        let name = self.expect_identifier()?;
        self.expect_punctuator("=")?;
        let value = self.expression()?;
        if expect_semicolon {
            self.expect_punctuator(";")?;
        }
        Ok(Statement::Assignment { name, value })
    }

    fn declaration_statement(&mut self) -> CompileResult<Statement> {
        self.expect_keyword(Keyword::Int)?;
        let name = self.expect_identifier()?;
        let initializer = if self.check_punctuator("=") {
            self.advance();
            Some(self.expression()?)
        } else {
            None
        };
        self.expect_punctuator(";")?;
        Ok(Statement::Declaration { name, initializer })
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

    fn for_statement(&mut self) -> CompileResult<Statement> {
        self.expect_keyword(Keyword::For)?;
        self.expect_punctuator("(")?;
        let initializer = if self.check_punctuator(";") {
            self.advance();
            None
        } else if self.check_keyword(Keyword::Int) {
            Some(Box::new(self.declaration_statement()?))
        } else {
            Some(Box::new(self.assignment_statement(true)?))
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
            Some(Box::new(self.assignment_statement(false)?))
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

    fn expression(&mut self) -> CompileResult<Expr> {
        self.logical_or()
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
        self.primary()
    }

    fn primary(&mut self) -> CompileResult<Expr> {
        if let Some(token) = self.peek() {
            match &token.kind {
                TokenKind::Integer(value) => {
                    let value = *value;
                    self.advance();
                    Ok(Expr::Integer(value))
                }
                TokenKind::Identifier(value) => {
                    let value = value.clone();
                    self.advance();
                    if self.check_punctuator("(") {
                        self.advance();
                        self.expect_punctuator(")")?;
                        return Ok(Expr::Call { callee: value });
                    }
                    Ok(Expr::Identifier(value))
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
    matches!(
        tokens,
        [
            Token {
                kind: TokenKind::Keyword(Keyword::Int),
                ..
            },
            Token {
                kind: TokenKind::Identifier(_),
                ..
            },
            Token {
                kind: TokenKind::Punctuator(open),
                ..
            },
            Token {
                kind: TokenKind::Keyword(Keyword::Void),
                ..
            },
            Token {
                kind: TokenKind::Punctuator(close),
                ..
            },
            Token {
                kind: TokenKind::Punctuator(body),
                ..
            },
            ..
        ] if open == "(" && close == ")" && body == "{"
    )
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
                return previous_identifier(tokens, index).map(ToOwned::to_owned);
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
