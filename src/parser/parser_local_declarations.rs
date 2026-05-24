use super::{
    CompileError, CompileResult, Constant, Keyword, Parser, ScalarType, Statement, TokenKind,
    eval_integer_initializer_expr_with_constants, parse_supported_global_declaration,
    previous_identifier_index, supported_return_type, token_identifier, token_is_punctuator,
    top_level_function_open_paren, top_level_punctuator_index,
};

impl Parser<'_> {
    pub(super) fn block_extern_declaration(&mut self) -> CompileResult<Option<Statement>> {
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
            self.known_pointer_typedefs,
            self.known_function_pointer_typedefs,
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

    pub(super) fn block_function_prototype_declaration(&mut self) -> Option<Statement> {
        let tokens = &self.tokens[self.index..];
        let semicolon_index = top_level_punctuator_index(tokens, ";")?;
        let declaration = &tokens[..semicolon_index];
        let open_index = top_level_function_open_paren(declaration)?;
        let name_index = previous_identifier_index(declaration, open_index)?;
        supported_return_type(&declaration[..name_index])?;
        self.index += semicolon_index + 1;
        Some(Statement::Empty)
    }

    pub(super) fn local_int_array_initializer(&mut self) -> CompileResult<Vec<i32>> {
        self.expect_punctuator("{")?;
        let mut values = Vec::new();
        if self.check_punctuator("}") {
            self.advance();
            return Ok(values);
        }
        let mut next_index = 0usize;
        loop {
            let index = if let Some(index) = self.local_array_designator_index()? {
                next_index = index + 1;
                index
            } else {
                let index = next_index;
                next_index += 1;
                index
            };
            let value = self.local_int_initializer_value()?;
            if values.len() <= index {
                values.resize(index + 1, 0);
            }
            values[index] = i32::try_from(value)
                .map_err(|_| CompileError::new("local int array initializer too large"))?;
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

    pub(super) fn local_int_initializer_value(&mut self) -> CompileResult<i64> {
        if self.check_punctuator("{") {
            self.advance();
            let value = self.local_int_initializer_value()?;
            self.expect_punctuator("}")?;
            return Ok(value);
        }
        eval_integer_initializer_expr_with_constants(&self.assignment()?, self.known_constants)?
            .to_i64_trunc()
    }

    pub(super) fn local_enum_declaration(&mut self) -> CompileResult<Option<Statement>> {
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

    pub(super) fn local_enum_constants(&mut self) -> CompileResult<Vec<Constant>> {
        let mut constants = Vec::new();
        let mut available_constants = self.known_constants.to_vec();
        let mut next_value = 0i64;
        while !self.check_punctuator("}") {
            let name = self.expect_identifier()?;
            let value = if self.check_punctuator("=") {
                self.advance();
                eval_integer_initializer_expr_with_constants(
                    &self.assignment()?,
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

    pub(super) fn static_aggregate_declaration(&mut self) -> CompileResult<Option<Statement>> {
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
}
