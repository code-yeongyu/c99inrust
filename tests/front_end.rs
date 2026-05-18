use c99inrust::front_end::lexer::{Keyword, TokenKind, lex};
use c99inrust::front_end::preprocessor::Preprocessor;

#[test]
fn lexer_handles_comments_keywords_and_integer_tokens() {
    // given
    let source = "int main(void) { /* Doom-era comment */ return 42; }\n";

    // when
    let tokens = lex(source).expect("lexer should tokenize C source");

    // then
    let kinds = tokens
        .into_iter()
        .map(|token| token.kind)
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Keyword(Keyword::Int),
            TokenKind::Identifier("main".to_string()),
            TokenKind::Punctuator("(".to_string()),
            TokenKind::Keyword(Keyword::Void),
            TokenKind::Punctuator(")".to_string()),
            TokenKind::Punctuator("{".to_string()),
            TokenKind::Keyword(Keyword::Return),
            TokenKind::Integer(42),
            TokenKind::Punctuator(";".to_string()),
            TokenKind::Punctuator("}".to_string()),
            TokenKind::End,
        ]
    );
}

#[test]
fn preprocessor_expands_object_macros_without_touching_strings() {
    // given
    let source = "#define ANSWER 42\nint main(void) { return ANSWER; }\nchar *s = \"ANSWER\";\n";

    // when
    let unit = Preprocessor::new()
        .preprocess_text("macro.c", source)
        .expect("preprocessor should expand object macros");

    // then
    assert!(unit.source.contains("return 42;"));
    assert!(unit.source.contains("\"ANSWER\""));
    assert!(unit.included_files.is_empty());
}
