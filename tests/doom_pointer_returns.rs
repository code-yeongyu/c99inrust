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
fn compiler_preserves_doom_shmat_pointer_return_width_slice() {
    // given
    let source = r"char *probe(int id) {
    return (char *) shmat(id, 0, 0);
}
int main(void) { return 0; }";

    // when
    let assembly = compile_x86_64(source);

    // then
    let call_index = assembly
        .find("\tcall shmat\n")
        .expect("assembly should call shmat");
    let call_tail = &assembly[call_index..];
    let return_index = call_tail.find("\tret\n").expect("probe should return");
    assert!(!call_tail[..return_index].contains("\tcltq\n"));
}
