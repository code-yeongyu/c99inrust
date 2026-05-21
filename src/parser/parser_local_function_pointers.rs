use super::{
    CompileResult, Parser, ScalarType, Statement, function_pointer_variable,
    pointer_referent_for_depth,
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
        let Some(declarator) = function_pointer_variable(&self.tokens[type_end..]) else {
            return Ok(None);
        };
        self.index = type_end + declarator.consumed;
        let initializer = if self.check_punctuator("=") {
            self.advance();
            Some(self.assignment()?)
        } else {
            None
        };
        self.expect_punctuator(";")?;
        Ok(Some(Statement::Declaration {
            is_static: false,
            scalar_type: ScalarType::Pointer,
            name: declarator.name,
            referent: pointer_referent_for_depth(declarator.pointer_depth, None),
            initializer,
        }))
    }
}
