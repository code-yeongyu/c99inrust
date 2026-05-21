use super::{
    CompileError, CompileResult, Expr, Parser, Statement, local_array_length,
    validate_local_char_array_initializer,
};

impl Parser<'_> {
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
            let Expr::StringLiteral(value) = self.assignment()? else {
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
}
