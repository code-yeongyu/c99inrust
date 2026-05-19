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
fn compiler_preserves_doom_pointer_alignment_cast_width_slice() {
    // given
    let source = r"typedef unsigned char byte;
byte *colormaps;
int main(void) {
    colormaps = (byte *)(((int)colormaps + 255) & ~0xff);
    return colormaps != (byte *)0;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\taddq %rcx, %rax\n"));
    assert!(assembly.contains("\tandq %rcx, %rax\n"));
    assert!(!assembly.contains("\taddl %ecx, %eax\n"));
    assert!(!assembly.contains("\tandl %ecx, %eax\n"));
}
