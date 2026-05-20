use super::{
    AssignmentOperator, BinaryOp, CompileError, CompileResult, Expr, Keyword, LValue, Parser,
    ScalarType, Statement, Token, TokenKind, declaration_base_referent_type,
    pointer_referent_for_depth, token_is_punctuator,
};

impl Parser<'_> {
    pub(super) fn statement(&mut self) -> CompileResult<Statement> {
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

    pub(super) fn label_statement(&mut self) -> Option<Statement> {
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

    pub(super) fn assignment_statement(
        &mut self,
        expect_semicolon: bool,
    ) -> CompileResult<Statement> {
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

    pub(super) fn declaration_statement(
        &mut self,
        base_type: ScalarType,
    ) -> CompileResult<Statement> {
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
                let initializer = self.declaration_initializer(
                    scalar_type,
                    type_includes_char,
                    type_is_unsigned,
                )?;
                Statement::Declaration {
                    is_static,
                    scalar_type,
                    name,
                    referent,
                    initializer: Some(initializer),
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

    fn declaration_initializer(
        &mut self,
        scalar_type: ScalarType,
        type_includes_char: bool,
        type_is_unsigned: bool,
    ) -> CompileResult<Expr> {
        let expr = self.expression()?;
        if scalar_type == ScalarType::Int && type_includes_char && type_is_unsigned {
            Ok(Expr::Binary {
                op: BinaryOp::BitAnd,
                left: Box::new(expr),
                right: Box::new(Expr::Integer(255)),
            })
        } else {
            Ok(expr)
        }
    }
}
