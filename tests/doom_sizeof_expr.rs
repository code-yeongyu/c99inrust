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
fn compiler_sizes_doom_dereferenced_sprite_pointer_slice() {
    // given
    let source = r"void *Z_Malloc(int size, int tag, void *user);
typedef struct {
    int rotate;
} spriteframe_t;
typedef struct {
    int numframes;
    spriteframe_t *spriteframes;
} spritedef_t;
spritedef_t *sprites;
int numsprites;
int main(void) {
    sprites = Z_Malloc(numsprites * sizeof(*sprites), 1, 0);
    return 0;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovl $16, %eax\n"));
    assert!(assembly.contains("\timull %ecx, %eax\n"));
    assert!(!assembly.contains("\tmovl $4, %eax\n"));
}
