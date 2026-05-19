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
fn compiler_sizes_doom_wad_filelump_char_array_field_slice() {
    // given
    let source = r#"typedef struct {
    int filepos;
    int size;
    char name[8];
} filelump_t;
filelump_t directory[2] = {
    { 1, 2, "A" },
    { 3, 4, "B" }
};
int main(void) { return sizeof(filelump_t); }"#;

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovl $16, %eax\n"));
    assert!(assembly.contains("\t.byte 65,0,0,0,0,0,0,0\n"));
    assert!(assembly.contains("\t.byte 66,0,0,0,0,0,0,0\n"));
    assert!(!assembly.contains("\tmovl $40, %eax\n"));
    assert!(!assembly.contains("\t.zero 80\n"));
}

#[test]
fn compiler_increments_doom_wad_filelump_pointer_by_struct_size_slice() {
    // given
    let source = r"typedef struct {
    int filepos;
    int size;
    char name[8];
} filelump_t;
filelump_t directory[2];
int main(void) {
    filelump_t *fileinfo;
    fileinfo = directory;
    fileinfo++;
    return (int) fileinfo++;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovl $16, %eax\n"));
    assert!(assembly.contains("\taddq %rcx, %rax\n"));
    assert!(assembly.contains("\taddq $16, %rax\n"));
    assert!(!assembly.contains("\taddq $1, %rax\n"));
    assert!(!assembly.contains("\tmovl $1, %eax\n"));
}
