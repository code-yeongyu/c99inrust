use c99inrust::front_end::lexer::{Keyword, TokenKind, lex};
use c99inrust::front_end::preprocessor::Preprocessor;
use std::fs;

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
fn lexer_accepts_integer_literal_suffixes() {
    // given
    let source = "int magic = 0x12345678l; int count = 42UL;\n";

    // when
    let tokens = lex(source).expect("lexer should tokenize suffixed integer literals");

    // then
    let integers = tokens
        .into_iter()
        .filter_map(|token| match token.kind {
            TokenKind::Integer(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(integers, vec![305_419_896, 42]);
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

#[test]
fn preprocessor_handles_doom_shaped_conditionals_and_undef() {
    // given
    let source = "#define LINUX 1\n#if defined(LINUX) && !defined(SNDSERV)\nint sound = 1;\n#elif defined(SNDSERV)\nint sound = 2;\n#else\nint sound = 3;\n#endif\n#undef LINUX\n#ifdef LINUX\nint after = 4;\n#else\nint after = 5;\n#endif\n";

    // when
    let unit = Preprocessor::new()
        .preprocess_text("doom-conditionals.c", source)
        .expect("preprocessor should evaluate Doom-shaped conditionals");

    // then
    assert!(unit.source.contains("int sound = 1;"));
    assert!(unit.source.contains("int after = 5;"));
    assert!(!unit.source.contains("int sound = 2;"));
    assert!(!unit.source.contains("int sound = 3;"));
    assert!(!unit.source.contains("int after = 4;"));
}

#[test]
fn preprocessor_expands_function_macros_and_spliced_lines() {
    // given
    let source = "#define MTOF(x) (FixedMul((x),scale_mtof)>>16)\n#define LONG_TEXT \"HELLO\" \\\n\" DOOM\"\nint a = MTOF(thing->x + 1);\nchar *text = LONG_TEXT;\n";

    // when
    let unit = Preprocessor::new()
        .preprocess_text("doom-macros.c", source)
        .expect("preprocessor should expand function-like macros and spliced definitions");

    // then
    assert!(
        unit.source
            .contains("int a = (FixedMul((thing->x + 1),scale_mtof)>>16);")
    );
    assert!(unit.source.contains("char *text = \"HELLO\" \" DOOM\";"));
}

#[test]
fn preprocessor_expands_file_and_line_builtins_after_macros() {
    // given
    let source = "#define LOC __FILE__, __LINE__\nint here[] = { LOC };\n";

    // when
    let unit = Preprocessor::new()
        .preprocess_text("doom/map.c", source)
        .expect("preprocessor should expand file and line builtins");

    // then
    assert!(unit.source.contains("int here[] = { \"doom/map.c\", 2 };"));
}

#[test]
fn preprocessor_provides_doom_values_h_integer_limits() {
    // given
    let source = "#include <values.h>\nint lo = MININT;\nint hi = MAXINT;\n";

    // when
    let unit = Preprocessor::new()
        .preprocess_text("doom-values.c", source)
        .expect("preprocessor should provide Doom-era values.h integer limits");

    // then
    assert!(unit.source.contains("#include <values.h>"));
    assert!(unit.source.contains("int lo = (-2147483647 - 1);"));
    assert!(unit.source.contains("int hi = 2147483647;"));
}

#[test]
fn preprocessor_provides_doom_netinet_port_base() {
    // given
    let source = "#include <netinet/in.h>\nint port = IPPORT_USERRESERVED + 0x1d;\n";

    // when
    let unit = Preprocessor::new()
        .preprocess_text("doom-netinet.c", source)
        .expect("preprocessor should provide Doom-era netinet port base");

    // then
    assert!(unit.source.contains("#include <netinet/in.h>"));
    assert!(unit.source.contains("int port = 5000 + 0x1d;"));
}

#[test]
fn preprocessor_provides_doom_stdio_seek_constants() {
    // given
    let source = "#include <stdio.h>\nint end = SEEK_END;\nint set = SEEK_SET;\nint nil = NULL;\n";

    // when
    let unit = Preprocessor::new()
        .preprocess_text("doom-stdio.c", source)
        .expect("preprocessor should provide Doom-era stdio seek constants");

    // then
    assert!(unit.source.contains("#include <stdio.h>"));
    assert!(unit.source.contains("int end = 2;"));
    assert!(unit.source.contains("int set = 0;"));
    assert!(unit.source.contains("int nil = 0;"));
}

#[test]
fn preprocessor_provides_doom_unistd_access_constant() {
    // given
    let source = "#include <unistd.h>\nint readable = R_OK;\n";

    // when
    let unit = Preprocessor::new()
        .preprocess_text("doom-unistd.c", source)
        .expect("preprocessor should provide Doom-era unistd access constants");

    // then
    assert!(unit.source.contains("#include <unistd.h>"));
    assert!(unit.source.contains("int readable = 4;"));
}

#[test]
fn preprocessor_removes_comments_before_macro_expansion() {
    // given
    let source = "#define HU_FONTSTART '!'\t// the first font character\n#define HU_FONTSIZE ('_' - HU_FONTSTART + 1)\nextern int hu_font[HU_FONTSIZE];\n";

    // when
    let unit = Preprocessor::new()
        .preprocess_text("commented-macro.c", source)
        .expect("preprocessor should remove comments before expanding macros");

    // then
    assert!(unit.source.contains("extern int hu_font[('_' - '!' + 1)];"));
    assert!(!unit.source.contains("// the first font character"));
}

#[test]
fn preprocessor_resolves_local_includes_and_preserves_system_includes() {
    // given
    let root = std::env::temp_dir().join(format!("c99inrust-front-end-{}", std::process::id()));
    fs::create_dir_all(&root).expect("fixture dir should be created");
    let header = root.join("doomdef.h");
    let source = root.join("d_main.c");
    fs::write(&header, "#define TICRATE 35\n").expect("header should be written");
    fs::write(
        &source,
        "#include <unistd.h>\n#include \"stdlib.h\"\n#include \"doomdef.h\"\nint tic = TICRATE;\n",
    )
    .expect("source should be written");

    // when
    let unit = Preprocessor::new()
        .preprocess_file(&source)
        .expect("preprocessor should resolve local includes and preserve system includes");

    // then
    assert!(unit.source.contains("#include <unistd.h>"));
    assert!(unit.source.contains("#include \"stdlib.h\""));
    assert!(unit.source.contains("int tic = 35;"));
    assert_eq!(unit.included_files, vec![header]);
}
