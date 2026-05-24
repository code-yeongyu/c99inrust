use super::{CompileError, CompileResult, Expr, Parser, ScalarType, Statement, zero_expr};

impl Parser<'_> {
    pub(super) fn local_scalar_array_declaration(
        &mut self,
        name: String,
        scalar_type: ScalarType,
        explicit_length: Option<usize>,
    ) -> CompileResult<Statement> {
        let initializer = if self.check_punctuator("=") {
            self.advance();
            Some(self.local_scalar_array_initializer()?)
        } else {
            None
        };
        let length = match (explicit_length, &initializer) {
            (Some(length), _) => length,
            (None, Some(values)) if !values.is_empty() => values.len(),
            (None, _) => {
                return Err(CompileError::new(
                    "local scalar arrays require a size or initializer",
                ));
            }
        };
        validate_local_scalar_array_initializer(length, initializer.as_deref())?;
        Ok(Statement::LocalScalarArray {
            name,
            scalar_type,
            length,
            initializer,
        })
    }

    fn local_scalar_array_initializer(&mut self) -> CompileResult<Vec<Expr>> {
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
            let value = self.assignment()?;
            if values.len() <= index {
                values.resize_with(index + 1, zero_expr);
            }
            values[index] = value;
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
}

fn validate_local_scalar_array_initializer(
    length: usize,
    initializer: Option<&[Expr]>,
) -> CompileResult<()> {
    if initializer.is_some_and(|values| values.len() > length) {
        return Err(CompileError::new(
            "local scalar array initializer is too large",
        ));
    }
    Ok(())
}
