use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse;

#[test]
fn compiler_emits_native_assembly_for_constant_return_program() {
    // given
    let source = "int main(void) { return 40 + 2; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly = emit_assembly(&lowered, Target::native()).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _main"));
    assert!(assembly.contains("mov w0, #42"));
    assert!(assembly.contains("ret"));
}
