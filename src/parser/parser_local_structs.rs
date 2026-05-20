use crate::front_end::lexer::Token;

use super::token_scan::top_level_comma_ranges;
use super::{
    CompileError, CompileResult, Keyword, LocalStructInitializer, LocalStructInitializerValue,
    Parser, Statement, TokenKind, anonymous_union_struct_name, matching_top_level_brace,
    token_identifier, token_is_keyword, token_is_punctuator,
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
            let initializer = if self.check_punctuator("=") {
                self.advance();
                Some(self.local_struct_initializer()?)
            } else {
                None
            };
            declarations.push(Statement::LocalStruct {
                name,
                struct_name: struct_name.clone(),
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

    fn local_struct_initializer(&mut self) -> CompileResult<LocalStructInitializer> {
        if self.check_punctuator("{") {
            return self
                .local_struct_initializer_values()
                .map(LocalStructInitializer::Values);
        }
        self.expression().map(LocalStructInitializer::Copy)
    }

    fn local_struct_initializer_values(
        &mut self,
    ) -> CompileResult<Vec<LocalStructInitializerValue>> {
        let open_brace = self.index;
        let close_brace = matching_top_level_brace(self.tokens, open_brace)
            .ok_or_else(|| CompileError::new("unterminated local struct initializer"))?;
        let values =
            self.parse_local_struct_initializer_values(&self.tokens[open_brace + 1..close_brace])?;
        self.index = close_brace + 1;
        Ok(values)
    }

    fn parse_local_struct_initializer_values(
        &self,
        tokens: &[Token],
    ) -> CompileResult<Vec<LocalStructInitializerValue>> {
        let mut values = Vec::new();
        for (start, end) in top_level_comma_ranges(tokens) {
            if start == end {
                continue;
            }
            values.push(self.parse_local_struct_initializer_value(&tokens[start..end])?);
        }
        Ok(values)
    }

    fn parse_local_struct_initializer_value(
        &self,
        tokens: &[Token],
    ) -> CompileResult<LocalStructInitializerValue> {
        if token_is_punctuator(&tokens[0], "{") {
            let close_brace = matching_top_level_brace(tokens, 0)
                .ok_or_else(|| CompileError::new("unterminated local struct initializer value"))?;
            if close_brace + 1 == tokens.len() {
                return self
                    .parse_local_struct_initializer_values(&tokens[1..close_brace])
                    .map(LocalStructInitializerValue::Nested);
            }
        }
        let mut parser = Parser {
            tokens,
            index: 0,
            known_structs: self.known_structs,
            known_constants: self.known_constants,
            known_scalar_typedefs: self.known_scalar_typedefs,
            known_pointer_typedefs: self.known_pointer_typedefs,
        };
        let expr = parser.expression()?;
        if let Some(token) = parser.peek() {
            return Err(
                CompileError::new("unsupported local struct initializer value")
                    .at(token.line, token.column),
            );
        }
        Ok(LocalStructInitializerValue::Expr(expr))
    }
}
