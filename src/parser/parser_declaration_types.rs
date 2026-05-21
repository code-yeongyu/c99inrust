use super::{
    CompileError, CompileResult, Keyword, Parser, ScalarType, TokenKind, matching_top_level_paren,
    supported_typedef_scalar, token_identifier, token_is_punctuator,
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
        if let Some(end) = self.typeof_span_at(index) {
            return Some((ScalarType::Int, end));
        }
        let mut saw_type = false;
        let mut saw_bool = false;
        let mut saw_complex = false;
        let mut saw_double = false;
        let mut saw_float = false;
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
                TokenKind::Keyword(Keyword::Bool) => {
                    saw_type = true;
                    saw_bool = true;
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
                TokenKind::Keyword(Keyword::Float) => {
                    saw_type = true;
                    saw_float = true;
                }
                TokenKind::Keyword(Keyword::Complex) => {
                    saw_type = true;
                    saw_complex = true;
                }
                TokenKind::Keyword(Keyword::Struct) => {
                    if saw_type {
                        break;
                    }
                    if !self.known_struct_pointer_at(index)? {
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
                    match self.declaration_identifier_kind(name, index, saw_type, saw_storage_class)
                    {
                        DeclarationIdentifier::Break => break,
                        DeclarationIdentifier::Unsupported => return None,
                        DeclarationIdentifier::VaList => {
                            return Some((ScalarType::VaList, index + 1));
                        }
                        DeclarationIdentifier::PointerTypedef => saw_pointer_typedef = true,
                        DeclarationIdentifier::StructPointer => saw_struct_pointer = true,
                        DeclarationIdentifier::Scalar => {}
                    }
                    saw_type = true;
                }
                _ => break,
            }
            index += 1;
        }
        let flags = bool_flag(saw_type, SAW_TYPE)
            | bool_flag(saw_storage_class, SAW_STORAGE_CLASS)
            | bool_flag(saw_struct_pointer || saw_pointer_typedef, SAW_POINTER)
            | bool_flag(saw_bool, SAW_BOOL)
            | bool_flag(saw_double, SAW_DOUBLE)
            | bool_flag(saw_complex, SAW_COMPLEX)
            | bool_flag(saw_float, SAW_FLOAT);
        declaration_type_from_flags(flags, long_count, index)
    }

    fn typeof_span_at(&self, index: usize) -> Option<usize> {
        let TokenKind::Identifier(name) = &self.tokens.get(index)?.kind else {
            return None;
        };
        if !matches!(name.as_str(), "typeof" | "__typeof__")
            || !token_is_punctuator(self.tokens.get(index + 1)?, "(")
        {
            return None;
        }
        matching_top_level_paren(self.tokens, index + 1).map(|close| close + 1)
    }

    fn known_struct_pointer_at(&self, index: usize) -> Option<bool> {
        let name = self.tokens.get(index + 1).and_then(token_identifier)?;
        Some(
            self.known_structs.iter().any(|layout| layout.name == name)
                && self.struct_pointer_declarator_follows(index + 2),
        )
    }

    fn declaration_identifier_kind(
        &self,
        name: &str,
        index: usize,
        saw_type: bool,
        saw_storage_class: bool,
    ) -> DeclarationIdentifier {
        if saw_type {
            return DeclarationIdentifier::Break;
        }
        if self
            .known_pointer_typedefs
            .iter()
            .any(|known| known == name)
        {
            return DeclarationIdentifier::PointerTypedef;
        }
        if self
            .supported_declaration_typedef_scalar(name)
            .is_some_and(|scalar_type| scalar_type == ScalarType::VaList)
        {
            return DeclarationIdentifier::VaList;
        }
        if self.supported_declaration_typedef_scalar(name) == Some(ScalarType::Int) {
            return DeclarationIdentifier::Scalar;
        }
        if self.known_structs.iter().any(|layout| layout.name == name)
            && self.struct_pointer_declarator_follows(index + 1)
        {
            return DeclarationIdentifier::StructPointer;
        }
        if saw_storage_class {
            DeclarationIdentifier::Break
        } else {
            DeclarationIdentifier::Unsupported
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

enum DeclarationIdentifier {
    Break,
    PointerTypedef,
    Scalar,
    StructPointer,
    Unsupported,
    VaList,
}

const SAW_TYPE: u8 = 1;
const SAW_STORAGE_CLASS: u8 = 2;
const SAW_POINTER: u8 = 4;
const SAW_BOOL: u8 = 8;
const SAW_DOUBLE: u8 = 16;
const SAW_COMPLEX: u8 = 32;
const SAW_FLOAT: u8 = 64;

const fn declaration_type_from_flags(
    flags: u8,
    long_count: usize,
    index: usize,
) -> Option<(ScalarType, usize)> {
    if !has_flag(flags, SAW_TYPE) {
        if has_flag(flags, SAW_STORAGE_CLASS) {
            return Some((ScalarType::Int, index));
        }
        return None;
    }
    if has_flag(flags, SAW_POINTER) {
        Some((ScalarType::Pointer, index))
    } else if has_flag(flags, SAW_COMPLEX) {
        Some((complex_declaration_type(flags, long_count), index))
    } else if has_flag(flags, SAW_FLOAT) {
        None
    } else if has_flag(flags, SAW_BOOL) {
        Some((ScalarType::Bool, index))
    } else if has_flag(flags, SAW_DOUBLE) && long_count == 0 {
        Some((ScalarType::Double, index))
    } else if has_flag(flags, SAW_DOUBLE) {
        Some((ScalarType::LongDouble, index))
    } else if long_count == 0 {
        Some((ScalarType::Int, index))
    } else {
        Some((ScalarType::LongLong, index))
    }
}

const fn complex_declaration_type(flags: u8, long_count: usize) -> ScalarType {
    if has_flag(flags, SAW_FLOAT) {
        ScalarType::ComplexFloat
    } else if long_count == 0 {
        ScalarType::ComplexDouble
    } else {
        ScalarType::ComplexLongDouble
    }
}

const fn bool_flag(value: bool, flag: u8) -> u8 {
    if value { flag } else { 0 }
}

const fn has_flag(flags: u8, flag: u8) -> bool {
    flags & flag != 0
}
