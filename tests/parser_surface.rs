use c99inrust::front_end::lexer::lex;
use c99inrust::parser::{ExternalItem, parse_translation_unit};

fn names_matching<F>(items: &[ExternalItem], mut predicate: F) -> Vec<String>
where
    F: FnMut(&ExternalItem) -> Option<&String>,
{
    items
        .iter()
        .filter_map(|item| predicate(item).cloned())
        .collect()
}

#[test]
fn surface_parser_recognizes_doom_shaped_header_declarations() {
    // given
    let source = r"
        typedef enum { false, true } boolean;
        typedef unsigned char byte;
        typedef struct { char forwardmove; short angleturn; byte buttons; } ticcmd_t;
        struct line_s;
        typedef void (*planefunction_t) (int top, int bottom);
        fixed_t FixedMul(fixed_t a, fixed_t b);
        extern fixed_t finesine[5*FINEANGLES/4];
        extern void (*colfunc) (void);
    ";

    // when
    let tokens = lex(source).expect("lexer should tokenize Doom-shaped declarations");
    let unit = parse_translation_unit(&tokens).expect("surface parser should accept declarations");

    // then
    let typedefs = names_matching(&unit.items, |item| match item {
        ExternalItem::Typedef { name } => Some(name),
        _ => None,
    });
    let prototypes = names_matching(&unit.items, |item| match item {
        ExternalItem::Prototype { name } => Some(name),
        _ => None,
    });
    let declarations = names_matching(&unit.items, |item| match item {
        ExternalItem::Declaration { name } => Some(name),
        _ => None,
    });
    let forwards = names_matching(&unit.items, |item| match item {
        ExternalItem::StructForward { name } => Some(name),
        _ => None,
    });

    assert_eq!(unit.typedef_count(), 4);
    assert_eq!(unit.prototype_count(), 1);
    assert_eq!(unit.declaration_count(), 2);
    assert!(typedefs.contains(&"boolean".to_string()));
    assert!(typedefs.contains(&"byte".to_string()));
    assert!(typedefs.contains(&"ticcmd_t".to_string()));
    assert!(typedefs.contains(&"planefunction_t".to_string()));
    assert!(prototypes.contains(&"FixedMul".to_string()));
    assert!(declarations.contains(&"finesine".to_string()));
    assert!(declarations.contains(&"colfunc".to_string()));
    assert!(forwards.contains(&"line_s".to_string()));
}
