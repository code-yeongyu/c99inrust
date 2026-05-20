use super::{
    CompileError, CompileResult, Keyword, Parser, ScalarType, TokenKind, supported_typedef_scalar,
    token_identifier,
};

impl Parser<'_> {
    pub(super) fn declaration_type_at_current(&self) -> Option<ScalarType> {
        self.declaration_type_span_at_current()
            .map(|(scalar_type, _end)| scalar_type)
    }

    pub(super) fn consume_declaration_type(&mut self, expected: ScalarType) -> CompileResult<bool> {
        let Some((actual, end)) = self.declaration_type_span_at_current() else {
            return self.expected("declaration type");
        };
        if actual != expected {
            return Err(CompileError::new("unexpected declaration type"));
        }
        let type_includes_char = self.tokens[self.index..end]
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Keyword(Keyword::Char)));
        self.index = end;
        Ok(type_includes_char)
    }

    pub(super) fn declaration_type_span_at_current(&self) -> Option<(ScalarType, usize)> {
        let mut index = self.index;
        let mut saw_type = false;
        let mut saw_double = false;
        let mut saw_storage_class = false;
        let mut saw_struct_pointer = false;
        let mut saw_pointer_typedef = false;
        let mut long_count = 0usize;
        while let Some(token) = self.tokens.get(index) {
            match &token.kind {
                TokenKind::Keyword(
                    Keyword::Const | Keyword::Restrict | Keyword::Signed | Keyword::Volatile,
                ) => {}
                TokenKind::Keyword(Keyword::Register | Keyword::Static) => {
                    saw_storage_class = true;
                }
                TokenKind::Keyword(
                    Keyword::Char | Keyword::Int | Keyword::Short | Keyword::Unsigned,
                ) => {
                    saw_type = true;
                }
                TokenKind::Keyword(Keyword::Void) => {
                    if !self.struct_pointer_declarator_follows(index + 1) {
                        return None;
                    }
                    saw_type = true;
                    saw_pointer_typedef = true;
                }
                TokenKind::Keyword(Keyword::Double) => {
                    saw_type = true;
                    saw_double = true;
                }
                TokenKind::Keyword(Keyword::Struct) => {
                    if saw_type {
                        break;
                    }
                    let name = self.tokens.get(index + 1).and_then(token_identifier)?;
                    if !self.known_structs.iter().any(|layout| layout.name == name)
                        || !self.struct_pointer_declarator_follows(index + 2)
                    {
                        return None;
                    }
                    saw_type = true;
                    saw_struct_pointer = true;
                    index += 2;
                    continue;
                }
                TokenKind::Keyword(Keyword::Long) => {
                    saw_type = true;
                    long_count += 1;
                }
                TokenKind::Identifier(name) => {
                    if saw_type {
                        break;
                    }
                    if self
                        .known_pointer_typedefs
                        .iter()
                        .any(|known| known == name)
                    {
                        saw_pointer_typedef = true;
                    } else if let Some(scalar_type) =
                        self.supported_declaration_typedef_scalar(name)
                    {
                        if scalar_type == ScalarType::VaList {
                            return Some((ScalarType::VaList, index + 1));
                        }
                        if scalar_type != ScalarType::Int {
                            return None;
                        }
                    } else if self.known_structs.iter().any(|layout| layout.name == *name)
                        && self.struct_pointer_declarator_follows(index + 1)
                    {
                        saw_struct_pointer = true;
                    } else if saw_storage_class {
                        break;
                    } else {
                        return None;
                    }
                    saw_type = true;
                }
                _ => break,
            }
            index += 1;
        }
        if !saw_type {
            if saw_storage_class {
                return Some((ScalarType::Int, index));
            }
            return None;
        }
        if saw_struct_pointer || saw_pointer_typedef {
            Some((ScalarType::Pointer, index))
        } else if saw_double && long_count == 0 {
            Some((ScalarType::Double, index))
        } else if long_count == 0 {
            Some((ScalarType::Int, index))
        } else {
            Some((ScalarType::LongLong, index))
        }
    }

    pub(super) fn supported_declaration_typedef_scalar(&self, name: &str) -> Option<ScalarType> {
        supported_typedef_scalar(name).or_else(|| {
            self.known_scalar_typedefs
                .iter()
                .any(|known_name| known_name == name)
                .then_some(ScalarType::Int)
        })
    }

    pub(super) fn struct_pointer_declarator_follows(&self, mut index: usize) -> bool {
        while let Some(token) = self.tokens.get(index) {
            match &token.kind {
                TokenKind::Keyword(Keyword::Const | Keyword::Restrict | Keyword::Volatile) => {
                    index += 1;
                }
                TokenKind::Punctuator(value) if value == "*" => return true,
                _ => return false,
            }
        }
        false
    }
}
