use super::function_pointer_typedefs::function_pointer_typedef_declaration_referent;
use super::{
    CompileResult, Keyword, Parser, ScalarType, Statement, TokenKind,
    declaration_base_referent_type, local_scalar_initializer, pointer_referent_for_depth,
};

impl Parser<'_> {
    pub(super) fn declaration_statement(
        &mut self,
        base_type: ScalarType,
    ) -> CompileResult<Statement> {
        let Some((_actual, type_end)) = self.declaration_type_span_at_current() else {
            return self.expected("declaration type");
        };
        let type_tokens = &self.tokens[self.index..type_end];
        let base_referent = declaration_base_referent_type(type_tokens);
        let type_includes_short = type_tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Short)));
        let type_is_unsigned = type_tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Unsigned)));
        let is_static = type_tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Static)));
        let type_includes_char = self.consume_declaration_type(base_type)?;
        let mut declarations = Vec::new();
        loop {
            let mut scalar_type = base_type;
            let mut pointer_depth = 0usize;
            while self.check_punctuator("*") {
                self.advance();
                pointer_depth += 1;
                scalar_type = ScalarType::Pointer;
            }
            let name = self.expect_identifier()?;
            let referent = if scalar_type == ScalarType::Pointer {
                function_pointer_typedef_declaration_referent(
                    self.known_function_pointer_typedefs,
                    base_referent.as_deref(),
                    pointer_depth,
                )
                .or_else(|| pointer_referent_for_depth(pointer_depth, base_referent.as_deref()))
            } else {
                scalar_declaration_referent(
                    type_includes_char,
                    type_includes_short,
                    type_is_unsigned,
                )
            };
            let initializer = if self.check_punctuator("=") {
                self.advance();
                Some(local_scalar_initializer(
                    scalar_type,
                    type_includes_char,
                    type_includes_short,
                    type_is_unsigned,
                    self.assignment()?,
                ))
            } else {
                None
            };
            let statement = if self.check_punctuator("[") {
                self.local_array_declaration(
                    type_includes_char,
                    type_includes_short,
                    type_is_unsigned,
                    scalar_type,
                    referent,
                    name,
                )?
            } else {
                Statement::Declaration {
                    is_static,
                    scalar_type,
                    name,
                    referent,
                    initializer,
                }
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
            Ok(declarations.remove(0))
        } else {
            Ok(Statement::DeclarationList(declarations))
        }
    }
}

fn scalar_declaration_referent(
    type_includes_char: bool,
    type_includes_short: bool,
    type_is_unsigned: bool,
) -> Option<String> {
    if type_includes_char {
        return Some(if type_is_unsigned { "byte" } else { "char" }.to_owned());
    }
    if type_includes_short {
        return Some(
            if type_is_unsigned {
                "unsigned short"
            } else {
                "short"
            }
            .to_owned(),
        );
    }
    None
}
