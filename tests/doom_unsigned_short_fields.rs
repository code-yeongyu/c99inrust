use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

fn compile_x86_64(source: &str) -> String {
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit")
}

#[test]
fn compiler_zero_extends_doom_unsigned_short_array_field_slice() {
    // given
    let source = r"typedef struct {
    int x;
    unsigned short children[2];
} node_t;
node_t *node;
int side;
int main(void) {
    return node->children[side];
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovzwl (%rcx,%rax,2), %eax\n"));
    assert!(!assembly.contains("\tmovswl (%rcx,%rax,2), %eax\n"));
}

#[test]
fn compiler_keeps_signed_short_field_load_sign_extended_slice() {
    // given
    let source = r"typedef struct {
    short y;
} thing_t;
thing_t *thing;
int main(void) {
    return thing->y;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovswl 0(%rax), %eax\n"));
    assert!(!assembly.contains("\tmovzwl 0(%rax), %eax\n"));
}
