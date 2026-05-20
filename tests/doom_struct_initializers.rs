use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

fn compile_to_linux_assembly(source: &str) -> String {
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit")
}

#[test]
fn doom_struct_object_initializers_emit_pointer_fields() {
    // given
    let source = r"typedef struct {
    unsigned char *sequence;
    unsigned char *p;
} cheatseq_t;
unsigned char cheat_amap_seq[] = { 0xb2, 0x26, 0xff };
cheatseq_t cheat_amap = { cheat_amap_seq, 0 };
int main(void) { return 42; }";

    // when
    let assembly = compile_to_linux_assembly(source);

    // then
    assert!(assembly.contains("cheat_amap:"));
    assert!(assembly.contains("\t.quad cheat_amap_seq\n"));
    assert!(assembly.contains("\t.quad 0\n"));
    assert!(!assembly.contains("cheat_amap:\n\t.zero 16\n"));
}

#[test]
fn doom_struct_matrix_row_initializers_use_byte_offsets() {
    // given
    let source = r"typedef struct {
    unsigned char *sequence;
    unsigned char *p;
} cheatseq_t;
unsigned char cheat_powerup_seq[2][3] = {
    { 1, 2, 0xff },
    { 3, 4, 0xff }
};
cheatseq_t cheat_powerup[2] = {
    { cheat_powerup_seq[0], 0 },
    { cheat_powerup_seq[1], 0 }
};
int main(void) { return 0; }";

    // when
    let assembly = compile_to_linux_assembly(source);

    // then
    assert!(assembly.contains("cheat_powerup:"));
    assert!(assembly.contains("\t.quad cheat_powerup_seq\n"));
    assert!(assembly.contains("\t.quad cheat_powerup_seq+3\n"));
}
