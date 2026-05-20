use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::{parse, parse_supported_translation_unit};

#[test]
fn compiler_accepts_pointer_dereference_slice() {
    // given
    let source = "int read_and_bump(int *p) { int value; value = *p; p++; return value; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("read_and_bump"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
    assert!(assembly.contains("\taddq %rcx, %rax\n"));
}

#[test]
fn compiler_accepts_sizeof_type_slice() {
    // given
    let source = "int Z_Malloc(int size, int tag, void *user) { return 0; } int main(void) { int *y; y = (int*) Z_Malloc(4 * sizeof(int), 1, 0); return sizeof(int) == 4 ? 0 : 1; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("Z_Malloc"));
    assert!(assembly.contains("\tmovl $4, %eax\n"));
    assert!(assembly.contains("\timull %ecx, %eax\n"));
}

#[test]
fn compiler_accepts_post_decrement_condition_slice() {
    // given
    let source =
        "void run(int ticks) { while (ticks--) { ticks = ticks; } } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("run"));
    assert!(assembly.contains("\taddl $-1, %eax\n"));
    assert!(assembly.contains("\tje .Lrun_"));
}

#[test]
fn compiler_emits_post_increment_value_slice() {
    // given
    let source = "int main(void) { int x = 4; return (x++ == 4 && x == 5) ? 0 : 1; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\taddl $1, %eax\n"));
    assert!(assembly.contains("\tmovl $4, %eax\n"));
    assert!(assembly.contains("\tmovl $5, %eax\n"));
}

#[test]
fn compiler_accepts_prefix_increment_condition_slice() {
    // given
    let source =
        "int fuzzpos; int main(void) { fuzzpos = 49; if (++fuzzpos == 50) return 0; return 1; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("fuzzpos"));
    assert!(assembly.contains("\taddl %ecx, %eax\n"));
}

#[test]
fn compiler_accepts_pointer_post_increment_dereference_slice() {
    // given
    let source = "void skip(int *p) { while (*(p++) != 1); } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("skip:"));
    assert!(assembly.contains("\taddq $4, %rax\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_address_of_subscript_slice() {
    // given
    let source = "int address_of_subscript(int *p, int i) { int *q; q = &p[i]; return 0; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("address_of_subscript"));
    assert!(assembly.contains("\tleaq (%rcx,%rax,4), %rax\n"));
    assert!(assembly.contains("\tmovq %rax, -"));
}

#[test]
fn compiler_accepts_unsigned_cast_slice() {
    // given
    let source =
        "int main(void) { int x = 7; return ((unsigned)x >= 0 && (unsigned char)x == x) ? 0 : 1; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tcmpl %ecx, %eax\n"));
    assert!(assembly.contains("\tsetge %al\n"));
    assert!(assembly.contains("\tsete %al\n"));
}

#[test]
fn compiler_accepts_unsigned_32_bit_mask_literals_slice() {
    // given
    let source = "int main(void) { return (0x80000000 & 0x0fffffff) == 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovl $-2147483648, %eax\n"));
    assert!(assembly.contains("\tmovl $268435455, %eax\n"));
    assert!(assembly.contains("\tandl %ecx, %eax\n"));
}

#[test]
fn compiler_marks_linux_assembly_stack_non_executable() {
    // given
    let source = "int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let linux_assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("linux assembly should emit");
    let apple_assembly =
        emit_assembly(&lowered, Target::X86_64AppleDarwin).expect("apple assembly should emit");

    // then
    assert!(linux_assembly.contains(".section .note.GNU-stack,\"\",@progbits"));
    assert!(!apple_assembly.contains(".note.GNU-stack"));
}
