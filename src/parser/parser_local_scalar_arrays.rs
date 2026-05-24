use super::{CompileError, CompileResult, Expr, Parser, ScalarType, Statement};

impl Parser<'_> {
    pub(super) fn local_scalar_array_declaration(
        &mut self,
        name: String,
        scalar_type: ScalarType,
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
