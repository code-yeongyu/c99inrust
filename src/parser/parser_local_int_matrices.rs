use super::{CompileError, CompileResult, Parser, Statement, local_array_length};

impl Parser<'_> {
    pub(super) fn local_int_matrix_declaration(
        &mut self,
        name: String,
        explicit_rows: Option<usize>,
    ) -> CompileResult<Statement> {
        let Some(rows) = explicit_rows else {
            return Err(CompileError::new("local int matrix rows require a size"));
        };
        self.expect_punctuator("[")?;
        let columns = local_array_length(&self.expression()?, self.known_constants)?;
        self.expect_punctuator("]")?;
        let initializer = if self.check_punctuator("=") {
            self.advance();
            Some(self.local_int_matrix_initializer(columns)?)
        } else {
            None
        };
        let length = rows
            .checked_mul(columns)
            .ok_or_else(|| CompileError::new("local int matrix size overflow"))?;
        let initializer = initializer
            .map(|mut values| {
                if values.len() > length {
                    return Err(CompileError::new(
                        "local int matrix initializer is too large",
                    ));
                }
                values.resize(length, 0);
                Ok(values)
            })
            .transpose()?;
        Ok(Statement::LocalIntMatrix {
            name,
            rows,
            columns,
            initializer,
        })
    }

    fn local_int_matrix_initializer(&mut self, columns: usize) -> CompileResult<Vec<i32>> {
        self.expect_punctuator("{")?;
        let mut values = Vec::new();
        if self.check_punctuator("}") {
            self.advance();
            return Ok(values);
        }
        loop {
            if self.check_punctuator("{") {
                self.local_int_matrix_row_initializer(columns, &mut values)?;
            } else {
                values.push(self.local_int_initializer_i32()?);
            }
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

    fn local_int_matrix_row_initializer(
        &mut self,
        columns: usize,
        values: &mut Vec<i32>,
    ) -> CompileResult<()> {
        self.expect_punctuator("{")?;
        let row_start = values.len();
        if !self.check_punctuator("}") {
            loop {
                values.push(self.local_int_initializer_i32()?);
                if values.len() - row_start > columns {
                    return Err(CompileError::new("local int matrix row is too large"));
                }
                if self.check_punctuator("}") {
                    break;
                }
                self.expect_punctuator(",")?;
                if self.check_punctuator("}") {
                    break;
                }
            }
        }
        self.expect_punctuator("}")?;
        values.resize(row_start + columns, 0);
        Ok(())
    }

    fn local_int_initializer_i32(&mut self) -> CompileResult<i32> {
        let value = self.local_int_initializer_value()?;
        i32::try_from(value)
            .map_err(|_| CompileError::new("local int matrix initializer too large"))
    }
}
