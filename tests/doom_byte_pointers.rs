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
fn compiler_uses_byte_stride_for_doom_byte_pointer_slice() {
    // given
    let source = r"typedef unsigned char byte;
int main(void) {
    byte *patchcount;
    int x;
    patchcount = (byte*)0;
    x = 4;
    patchcount[x]++;
    return patchcount[x];
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovzbl (%rcx,%rax,1), %eax\n"));
    assert!(assembly.contains("\tmovb %al, (%rcx,%rdx,1)\n"));
    assert!(!assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
    assert!(!assembly.contains("\tmovl %eax, (%rcx,%rdx,4)\n"));
}
