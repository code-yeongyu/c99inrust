use super::{
    CompileError, CompileResult, Constant, Keyword, Parser, ScalarType, Statement, TokenKind,
    anonymous_union_struct_name, eval_integer_initializer_expr_with_constants,
    matching_top_level_brace, parse_supported_global_declaration, previous_identifier_index,
    supported_return_type, token_identifier, token_is_keyword, token_is_punctuator,
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

    pub(super) fn local_struct_declaration(&mut self) -> CompileResult<Option<Statement>> {
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

    pub(super) fn local_struct_name_at(&self, index: usize) -> Option<(String, usize)> {
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

    pub(super) fn local_anonymous_union_declaration(&mut self) -> CompileResult<Option<Statement>> {
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

    pub(super) fn local_int_array_initializer(&mut self) -> CompileResult<Vec<i32>> {
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
