use super::{
    CompileResult, Parser, ScalarType, Statement, function_pointer_variable,
    local_scalar_initializer, pointer_referent_for_depth,
};

impl Parser<'_> {
    pub(super) fn local_function_pointer_declaration(
        &mut self,
    ) -> CompileResult<Option<Statement>> {
        let Some((scalar_type, type_end)) = self.declaration_type_span_at_current() else {
            return Ok(None);
        };
        if scalar_type != ScalarType::Int {
            return Ok(None);
        }
        if function_pointer_variable(&self.tokens[type_end..]).is_none() {
            return Ok(None);
        }
        self.index = type_end;
        let mut declarations = Vec::new();
        loop {
            let statement =
                if let Some(declarator) = function_pointer_variable(&self.tokens[self.index..]) {
                    self.index += declarator.consumed;
                    let initializer = if self.check_punctuator("=") {
                        self.advance();
                        Some(self.assignment()?)
                    } else {
                        None
                    };
                    Statement::Declaration {
                        is_static: false,
                        scalar_type: ScalarType::Pointer,
                        name: declarator.name,
                        referent: pointer_referent_for_depth(declarator.pointer_depth, None),
                        initializer,
                    }
                } else {
                    self.local_int_declarator_statement()?
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

    fn local_int_declarator_statement(&mut self) -> CompileResult<Statement> {
        let mut scalar_type = ScalarType::Int;
        let mut pointer_depth = 0usize;
        while self.check_punctuator("*") {
            self.advance();
            pointer_depth += 1;
            scalar_type = ScalarType::Pointer;
        }
        let name = self.expect_identifier()?;
        if self.check_punctuator("[") {
            return self.local_array_declaration(false, false, false, scalar_type, name);
        }
        let referent = (scalar_type == ScalarType::Pointer)
            .then(|| pointer_referent_for_depth(pointer_depth, None))
            .flatten();
        let initializer = if self.check_punctuator("=") {
            self.advance();
            Some(local_scalar_initializer(
                scalar_type,
                false,
                false,
                false,
                self.assignment()?,
            ))
        } else {
            None
        };
        Ok(Statement::Declaration {
            is_static: false,
            scalar_type,
            name,
            referent,
            initializer,
        })
    }
}
