use super::{
    CompileError, CompileResult, Expr, Parser, ScalarType, Statement, local_array_length,
    token_is_punctuator,
};

impl Parser<'_> {
    pub(super) fn local_function_pointer_array_declaration(
        &mut self,
    ) -> CompileResult<Option<Statement>> {
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

    pub(super) fn skip_balanced_parentheses(&mut self) -> CompileResult<()> {
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

    pub(super) fn local_pointer_array_initializer(&mut self) -> CompileResult<Vec<Expr>> {
        self.expect_punctuator("{")?;
        let mut values = Vec::new();
        if self.check_punctuator("}") {
            self.advance();
            return Ok(values);
        }
        loop {
            values.push(self.assignment()?);
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

    pub(super) fn local_short_array_declaration(
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

    pub(super) fn local_pointer_array_declaration(
        &mut self,
        name: String,
        explicit_length: Option<usize>,
    ) -> CompileResult<Statement> {
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
                    "local pointer arrays require a size or initializer",
                ));
            }
        };
        Ok(Statement::LocalPointerArray {
            name,
            length,
            initializer,
        })
    }
}
