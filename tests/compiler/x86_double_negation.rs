use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse;

#[test]
fn compiler_emits_scalar_safe_x86_64_double_negation_mask_load() {
    // given
    let source = "int main(void) { double value = 1.25; value = -value; return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("(%rip), %xmm1\n\txorpd %xmm1, %xmm0\n"));
    assert!(!assembly.contains("\txorpd .L"));
}
