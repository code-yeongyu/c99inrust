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
fn compiler_decays_doom_bbox_row_to_pointer_argument_slice() {
    // given
    let source = r"typedef int fixed_t;
typedef struct {
    fixed_t bbox[2][4];
} node_t;
int side;
node_t *bsp;
int R_CheckBBox(fixed_t *box) {
    return box[0];
}
int main(void) {
    return R_CheckBBox(bsp->bbox[side ^ 1]);
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\timulq $16, %rax\n"));
    assert!(assembly.contains("\taddq %rcx, %rax\n"));
    assert!(assembly.contains("\tmovq -8(%rbp), %rdi\n\tcall R_CheckBBox\n"));
}

#[test]
fn compiler_decays_doom_scalelight_row_to_pointer_assignment_slice() {
    // given
    let source = r"typedef unsigned char byte;
typedef byte lighttable_t;
extern lighttable_t *scalelight[16][48];
lighttable_t **spritelights;
int lightnum;
int main(void) {
    spritelights = scalelight[lightnum];
    return spritelights != 0;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\timulq $384, %rax\n"));
    assert!(assembly.contains("\taddq %rcx, %rax\n"));
    assert!(assembly.contains("\tmovq %rax, spritelights(%rip)\n"));
    assert!(!assembly.contains("\tmovq (%rcx,%rax,8), %rax\n"));
}
