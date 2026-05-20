use super::{
    CompileError, CompileResult, Expr, LocalCharArrayInitializer, Parser, ScalarType, Statement,
    eval_integer_initializer_expr_with_constants, inferred_local_char_array_length,
    local_array_length, validate_local_char_array_initializer,
    validate_local_char_array_initializer_size,
};

impl Parser<'_> {
    pub(super) fn local_array_declaration(
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

    pub(super) fn local_char_array_declaration(
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

    pub(super) fn local_char_array_initializer(
        &mut self,
    ) -> CompileResult<LocalCharArrayInitializer> {
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

    pub(super) fn local_char_array_braced_initializer(&mut self) -> CompileResult<Vec<u8>> {
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

    pub(super) fn local_char_matrix_declaration(
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

    pub(super) fn local_string_list_initializer(
        &mut self,
        columns: usize,
    ) -> CompileResult<Vec<String>> {
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

    pub(super) fn local_int_array_declaration(
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
}
