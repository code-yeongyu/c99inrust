#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Auto,
    Bool,
    Break,
    Case,
    Char,
    Complex,
    Const,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Enum,
    Extern,
    Float,
    For,
    Goto,
    If,
    Inline,
    Int,
    Long,
    Register,
    Restrict,
    Return,
    Short,
    Signed,
    Sizeof,
    Static,
    Struct,
    Switch,
    Typedef,
    Union,
    Unsigned,
    Void,
    Volatile,
    While,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier(String),
    Integer(i64),
    LongInteger(i64),
    StringLiteral(String),
    CharLiteral(char),
    Keyword(Keyword),
    Punctuator(String),
    End,
}

impl TokenKind {
    #[must_use]
    pub const fn integer_value(&self) -> Option<i64> {
        match self {
            Self::Integer(value) | Self::LongInteger(value) => Some(*value),
            Self::Identifier(_)
            | Self::StringLiteral(_)
            | Self::CharLiteral(_)
            | Self::Keyword(_)
            | Self::Punctuator(_)
            | Self::End => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

pub(super) fn identifier_or_keyword_kind(value: String) -> TokenKind {
    match value.as_str() {
        "_Bool" => TokenKind::Keyword(Keyword::Bool),
        "auto" => TokenKind::Keyword(Keyword::Auto),
        "break" => TokenKind::Keyword(Keyword::Break),
        "case" => TokenKind::Keyword(Keyword::Case),
        "char" => TokenKind::Keyword(Keyword::Char),
        "_Complex" => TokenKind::Keyword(Keyword::Complex),
        "const" => TokenKind::Keyword(Keyword::Const),
        "continue" => TokenKind::Keyword(Keyword::Continue),
        "default" => TokenKind::Keyword(Keyword::Default),
        "do" => TokenKind::Keyword(Keyword::Do),
        "double" => TokenKind::Keyword(Keyword::Double),
        "else" => TokenKind::Keyword(Keyword::Else),
        "enum" => TokenKind::Keyword(Keyword::Enum),
        "extern" => TokenKind::Keyword(Keyword::Extern),
        "float" => TokenKind::Keyword(Keyword::Float),
        "for" => TokenKind::Keyword(Keyword::For),
        "goto" => TokenKind::Keyword(Keyword::Goto),
        "if" => TokenKind::Keyword(Keyword::If),
        "inline" => TokenKind::Keyword(Keyword::Inline),
        "int" => TokenKind::Keyword(Keyword::Int),
        "long" => TokenKind::Keyword(Keyword::Long),
        "register" => TokenKind::Keyword(Keyword::Register),
        "restrict" => TokenKind::Keyword(Keyword::Restrict),
        "return" => TokenKind::Keyword(Keyword::Return),
        "short" => TokenKind::Keyword(Keyword::Short),
        "signed" => TokenKind::Keyword(Keyword::Signed),
        "sizeof" => TokenKind::Keyword(Keyword::Sizeof),
        "static" => TokenKind::Keyword(Keyword::Static),
        "struct" => TokenKind::Keyword(Keyword::Struct),
        "switch" => TokenKind::Keyword(Keyword::Switch),
        "typedef" => TokenKind::Keyword(Keyword::Typedef),
        "union" => TokenKind::Keyword(Keyword::Union),
        "unsigned" => TokenKind::Keyword(Keyword::Unsigned),
        "void" => TokenKind::Keyword(Keyword::Void),
        "volatile" => TokenKind::Keyword(Keyword::Volatile),
        "while" => TokenKind::Keyword(Keyword::While),
        _ => TokenKind::Identifier(value),
    }
}
