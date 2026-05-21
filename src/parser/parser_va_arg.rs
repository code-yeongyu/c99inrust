use super::{
    CompileError, CompileResult, Expr, Parser, ScalarType, Token, pointer_referent_from_specifiers,
    supported_cast_type_with_typedefs, top_level_punctuator_index,
};

impl Parser<'_> {
    pub(super) fn va_arg_after_identifier(&mut self, name: &str) -> CompileResult<Option<Expr>> {
        if name != "va_arg" || !self.check_punctuator("(") {
            return Ok(None);
        }
        self.expect_punctuator("(")?;
        let list = self.assignment()?;
        self.expect_punctuator(",")?;
        let type_start = self.index;
        let type_len = top_level_punctuator_index(&self.tokens[type_start..], ")")
            .ok_or_else(|| CompileError::new("unterminated va_arg type"))?;
        let type_tokens = &self.tokens[type_start..type_start + type_len];
        let scalar_type = self.va_arg_type(type_tokens)?;
        let referent = if scalar_type == ScalarType::Pointer {
            pointer_referent_from_specifiers(type_tokens)
        } else {
            None
        };
        self.index = type_start + type_len;
        self.expect_punctuator(")")?;
        Ok(Some(Expr::VaArg {
            list: Box::new(list),
            scalar_type,
            referent,
        }))
    }

    fn va_arg_type(&self, tokens: &[Token]) -> CompileResult<ScalarType> {
        supported_cast_type_with_typedefs(
            tokens,
            self.known_scalar_typedefs,
            self.known_pointer_typedefs,
        )
        .ok_or_else(|| CompileError::new("unsupported va_arg type"))
    }
}
