use super::{
    CompileError, CompileResult, Expr, LocalCharArrayInitializer, LocalVlaElement, Parser,
    ScalarType, Statement, eval_integer_initializer_expr_with_constants,
    inferred_local_char_array_length, local_array_length,
    validate_local_char_array_initializer_size,
};

fn local_char_initializer_byte(value: i64) -> CompileResult<u8> {
    u8::try_from(value & 0xff)
        .map_err(|_| CompileError::new("local char array initializer too large"))
}

impl Parser<'_> {
    pub(super) fn local_array_declaration(
        &mut self,
        type_includes_char: bool,
        type_includes_short: bool,
        type_is_unsigned: bool,
        scalar_type: ScalarType,
        referent: Option<String>,
        name: String,
    ) -> CompileResult<Statement> {
        self.advance();
        let explicit_length_expr = if self.check_punctuator("]") {
            None
        } else {
            Some(self.expression()?)
        };
        self.expect_punctuator("]")?;
        if scalar_type == ScalarType::Pointer {
            let explicit_length = explicit_length_expr
                .as_ref()
                .map(|expr| local_array_length(expr, self.known_constants))
                .transpose()?;
            return self.local_pointer_array_declaration(name, explicit_length, referent);
        }
        if scalar_type == ScalarType::Double {
            let explicit_length = explicit_length_expr
                .as_ref()
                .map(|expr| local_array_length(expr, self.known_constants))
                .transpose()?;
            return self.local_scalar_array_declaration(name, scalar_type, explicit_length);
        }
        if scalar_type != ScalarType::Int {
            return Err(CompileError::new(
                "only local int, char, double, and pointer arrays are supported",
            ));
        }
        let has_second_dimension = self.check_punctuator("[");
        if let Some(length) = &explicit_length_expr {
            let element = if type_includes_char {
                Some(LocalVlaElement::Char {
                    is_unsigned: type_is_unsigned,
                })
            } else if type_includes_short {
                None
            } else {
                Some(LocalVlaElement::Int)
            };
            if let Some(element) = element
                && let Some(statement) =
                    self.local_vla_array_declaration(element, &name, length, has_second_dimension)?
            {
                return Ok(statement);
            }
        }
        let explicit_length = explicit_length_expr
            .as_ref()
            .map(|expr| local_array_length(expr, self.known_constants))
            .transpose()?;
        if type_includes_char && has_second_dimension {
            return self.local_char_matrix_declaration(name, explicit_length);
        }
        if type_includes_char {
            return self.local_char_array_declaration(name, explicit_length, type_is_unsigned);
        }
        if !type_includes_short && self.check_punctuator("[") {
            return self.local_int_matrix_declaration(name, explicit_length);
        }
        if type_includes_short {
            return self.local_short_array_declaration(name, explicit_length, type_is_unsigned);
        }
        self.local_int_array_declaration(name, explicit_length)
    }

    pub(super) fn local_scalar_array_declaration(
        &self,
        name: String,
        scalar_type: ScalarType,
        explicit_length: Option<usize>,
    ) -> CompileResult<Statement> {
        if self.check_punctuator("=") {
            return Err(CompileError::new(
                "local scalar array initializers are not supported",
            ));
        }
        let Some(length) = explicit_length else {
            return Err(CompileError::new("local scalar arrays require a size"));
        };
        Ok(Statement::LocalScalarArray {
            name,
            scalar_type,
            length,
        })
    }

    pub(super) fn local_char_array_declaration(
        &mut self,
        name: String,
        explicit_length: Option<usize>,
        is_unsigned: bool,
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
            is_unsigned,
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
            let value = eval_integer_initializer_expr_with_constants(
                &self.assignment()?,
                self.known_constants,
            )?
            .to_i64_trunc()?;
            if values.len() <= index {
                values.resize(index + 1, 0);
            }
            values[index] = local_char_initializer_byte(value)?;
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
