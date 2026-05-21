use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::scalar_layout::sizeof_scalar_type;
use super::token_scan::top_level_comma_ranges;
use super::{
    CompileError, CompileResult, Expr, Parser, ScalarType,
    eval_integer_initializer_expr_with_constants, matching_top_level_brace,
    matching_top_level_bracket, matching_top_level_paren, supported_cast_type_with_typedefs,
    token_identifier, token_is_keyword, token_is_punctuator,
};

impl Parser<'_> {
    pub(super) fn compound_literal_at_current(&mut self) -> CompileResult<Option<Expr>> {
        if !self.check_punctuator("(") {
            return Ok(None);
        }
        let Some(close_paren) = matching_top_level_paren(self.tokens, self.index) else {
            return Ok(None);
        };
        if !self
            .tokens
            .get(close_paren + 1)
            .is_some_and(|token| token_is_punctuator(token, "{"))
        {
            return Ok(None);
        }
        let type_tokens = &self.tokens[self.index + 1..close_paren];
        if let Some(struct_name) = self.compound_struct_name(type_tokens) {
            self.index = close_paren + 1;
            let values = self.local_struct_initializer_values_for_struct(&struct_name)?;
            return Ok(Some(Expr::StructCompoundLiteral {
                struct_name,
                values,
            }));
        }
        let Some(array_type) = self.compound_array_type(type_tokens)? else {
            return Ok(None);
        };
        self.index = close_paren + 1;
        self.compound_array_literal(array_type).map(Some)
    }

    fn compound_struct_name(&self, tokens: &[Token]) -> Option<String> {
        if tokens.len() == 1 {
            let name = token_identifier(&tokens[0])?;
            return self
                .known_structs
                .iter()
                .any(|layout| layout.name == name)
                .then(|| name.to_owned());
        }
        if tokens.len() == 2 && token_is_keyword(&tokens[0], Keyword::Struct) {
            let name = token_identifier(&tokens[1])?;
            return self
                .known_structs
                .iter()
                .any(|layout| layout.name == name)
                .then(|| name.to_owned());
        }
        None
    }

    fn compound_array_type(&self, tokens: &[Token]) -> CompileResult<Option<CompoundArrayType>> {
        let Some(open_bracket) = tokens
            .iter()
            .position(|token| token_is_punctuator(token, "["))
        else {
            return Ok(None);
        };
        let Some(close_bracket) = matching_top_level_bracket(tokens, open_bracket) else {
            return Ok(None);
        };
        if close_bracket + 1 != tokens.len() {
            return Ok(None);
        }
        let specifiers = &tokens[..open_bracket];
        let Some(element_type) = supported_cast_type_with_typedefs(
            specifiers,
            self.known_scalar_typedefs,
            self.known_pointer_typedefs,
        ) else {
            return Ok(None);
        };
        let length = self.compound_array_length(&tokens[open_bracket + 1..close_bracket])?;
        Ok(Some(CompoundArrayType {
            element_type,
            element_byte_size: compound_array_element_size(specifiers, element_type),
            element_unsigned: specifiers_are_unsigned_char(specifiers),
            length,
        }))
    }

    fn compound_array_length(&self, tokens: &[Token]) -> CompileResult<Option<usize>> {
        if tokens.is_empty() {
            return Ok(None);
        }
        let expr = self.parse_compound_literal_expr(tokens)?;
        let value = eval_integer_initializer_expr_with_constants(&expr, self.known_constants)?
            .to_i128_integer()?;
        if value < 0 {
            return Err(CompileError::new("compound literal array size is negative"));
        }
        usize::try_from(value)
            .map(Some)
            .map_err(|_| CompileError::new("compound literal array size is too large"))
    }

    fn compound_array_literal(&mut self, array_type: CompoundArrayType) -> CompileResult<Expr> {
        let open_brace = self.index;
        let close_brace = matching_top_level_brace(self.tokens, open_brace)
            .ok_or_else(|| CompileError::new("unterminated compound literal"))?;
        let values =
            self.parse_compound_array_literal_exprs(&self.tokens[open_brace + 1..close_brace])?;
        self.index = close_brace + 1;
        let length = array_type.length.unwrap_or(values.len());
        Ok(Expr::ArrayCompoundLiteral {
            element_type: array_type.element_type,
            element_byte_size: array_type.element_byte_size,
            element_unsigned: array_type.element_unsigned,
            length,
            values,
        })
    }

    fn parse_compound_array_literal_exprs(&self, tokens: &[Token]) -> CompileResult<Vec<Expr>> {
        let mut values = Vec::new();
        let mut next_index = 0usize;
        for (start, end) in top_level_comma_ranges(tokens) {
            if start == end {
                continue;
            }
            let item = &tokens[start..end];
            let (index, value_tokens) =
                if let Some((index, value_tokens)) = self.compound_array_designator(item)? {
                    next_index = index + 1;
                    (index, value_tokens)
                } else {
                    let index = next_index;
                    next_index += 1;
                    (index, item)
                };
            if values.len() <= index {
                values.resize(index + 1, Expr::Integer(0));
            }
            values[index] = self.parse_compound_literal_expr(value_tokens)?;
        }
        Ok(values)
    }

    fn compound_array_designator<'a>(
        &self,
        tokens: &'a [Token],
    ) -> CompileResult<Option<(usize, &'a [Token])>> {
        if !tokens
            .first()
            .is_some_and(|token| token_is_punctuator(token, "["))
        {
            return Ok(None);
        }
        let close_bracket = matching_top_level_bracket(tokens, 0)
            .ok_or_else(|| CompileError::new("unterminated compound literal designator"))?;
        if !tokens
            .get(close_bracket + 1)
            .is_some_and(|token| token_is_punctuator(token, "="))
        {
            return Err(CompileError::new(
                "expected compound literal designator assignment",
            ));
        }
        let expr = self.parse_compound_literal_expr(&tokens[1..close_bracket])?;
        let index = eval_integer_initializer_expr_with_constants(&expr, self.known_constants)?
            .to_i128_integer()?;
        let index = usize::try_from(index)
            .map_err(|_| CompileError::new("compound literal designator is negative"))?;
        Ok(Some((index, &tokens[close_bracket + 2..])))
    }

    fn parse_compound_literal_expr(&self, tokens: &[Token]) -> CompileResult<Expr> {
        let mut parser = Parser {
            tokens,
            index: 0,
            known_structs: self.known_structs,
            known_constants: self.known_constants,
            known_scalar_typedefs: self.known_scalar_typedefs,
            known_pointer_typedefs: self.known_pointer_typedefs,
        };
        let expr = parser.expression()?;
        if let Some(token) = parser.peek() {
            return Err(CompileError::new("unsupported compound literal value")
                .at(token.line, token.column));
        }
        Ok(expr)
    }
}

#[derive(Clone, Copy)]
struct CompoundArrayType {
    element_type: ScalarType,
    element_byte_size: usize,
    element_unsigned: bool,
    length: Option<usize>,
}

fn compound_array_element_size(tokens: &[Token], element_type: ScalarType) -> usize {
    if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Char)))
    {
        1
    } else if tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Short)))
    {
        2
    } else {
        sizeof_scalar_type(tokens, element_type)
    }
}

fn specifiers_are_unsigned_char(tokens: &[Token]) -> bool {
    let mut saw_unsigned = false;
    let mut saw_char = false;
    for token in tokens {
        match token.kind {
            TokenKind::Keyword(Keyword::Unsigned) => saw_unsigned = true,
            TokenKind::Keyword(Keyword::Char) => saw_char = true,
            _ => {}
        }
    }
    saw_unsigned && saw_char
}
