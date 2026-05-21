use super::{
    CompileError, CompileResult, Keyword, Parser, Statement, TokenKind,
    anonymous_union_struct_name, local_array_length, matching_top_level_brace, token_identifier,
    token_is_keyword, token_is_punctuator,
};

impl Parser<'_> {
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
            let statement = if self.check_punctuator("[") {
                self.advance();
                let length = local_array_length(&self.expression()?, self.known_constants)?;
                self.expect_punctuator("]")?;
                let initializer = if self.check_punctuator("=") {
                    self.advance();
                    if !self.check_punctuator("{") {
                        return Err(CompileError::new(
                            "local struct array initializers require braces",
                        ));
                    }
                    Some(self.local_struct_initializer_values()?)
                } else {
                    None
                };
                Statement::LocalStructArray {
                    name,
                    struct_name: struct_name.clone(),
                    length,
                    initializer,
                }
            } else if self.check_punctuator("=") {
                self.advance();
                Statement::LocalStruct {
                    name,
                    struct_name: struct_name.clone(),
                    initializer: Some(self.local_struct_initializer(&struct_name)?),
                }
            } else {
                Statement::LocalStruct {
                    name,
                    struct_name: struct_name.clone(),
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
            initializer: None,
        }))
    }
}
