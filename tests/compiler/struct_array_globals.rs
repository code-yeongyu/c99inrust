use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

#[test]
fn compiler_accepts_global_char_array_decay_slice() {
    // given
    let source = r"char basedefault[1024];
void use(char* value);
int main(void) {
    use(basedefault);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("basedefault:"));
    assert!(assembly.contains("\t.byte 0"));
    assert!(assembly.contains("\tleaq basedefault(%rip), %rax\n"));
}

#[test]
fn compiler_accepts_global_char_matrix_row_decay_slice() {
    // given
    let source = r#"void use(char* value);
char savegamestrings[10][24];
char detailNames[2][9] = {"M_GDHIGH", "M_GDLOW"};
int main(void) {
    use(savegamestrings[1]);
    use(&savegamestrings[1][0]);
    use(detailNames[1]);
    savegamestrings[1][0] = 7;
    return savegamestrings[1][0] + detailNames[0][0];
}"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("savegamestrings:"));
    assert!(assembly.contains("detailNames:"));
    assert!(assembly.contains("\tcall use\n"));
    assert!(assembly.contains("\tmovb %al, (%rcx,%rdx,1)\n"));
    assert!(assembly.contains("\tmovzbl (%rcx,%rax,1), %eax\n"));
}

#[test]
fn compiler_accepts_global_unsigned_char_numeric_matrix_initializer_slice() {
    // given
    let source = r"typedef unsigned char byte;
byte gammatable[2][4] = {
    {1, 2, 3, 4},
    {5, 6, 7}
};
int main(void) {
    return gammatable[1][3];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("gammatable:"));
    assert!(assembly.contains("\t.byte 1,2,3,4,5,6,7,0\n"));
    assert!(assembly.contains("main:"));
}

#[test]
fn compiler_accepts_mixed_pointer_scalar_local_declaration_slice() {
    // given
    let source = "int main(void) { unsigned char *p, c; p = 0; c = 7; return c; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovq %rax, -"));
    assert!(assembly.contains("\tmovl %eax, -"));
    assert!(assembly.contains("\tmovl $7, %eax\n"));
}

#[test]
fn compiler_accepts_plain_unsigned_local_declaration_slice() {
    // given
    let source = "unsigned NetbufferChecksum(void) { unsigned c; c = 0x1234567; return c; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("NetbufferChecksum:"));
    assert!(assembly.contains("\tmovl $19088743, %eax\n"));
}

#[test]
fn compiler_accepts_goto_label_slice() {
    // given
    let source = r"int main(void) {
    int value;
    value = 1;
    goto done;
    value = 2;
done:
    return value;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tjmp .Lmain_"));
    assert!(assembly.contains("\n.Lmain_"));
}

#[test]
fn compiler_accepts_struct_array_field_subscript_address_slice() {
    // given
    let source = r"typedef struct {
    int forwardmove;
} ticcmd_t;
typedef struct {
    ticcmd_t cmds[4];
} doomdata_t;
int main(void) {
    doomdata_t packet;
    ticcmd_t source;
    ticcmd_t* dest;
    int index;
    index = 2;
    source.forwardmove = 7;
    packet.cmds[index] = source;
    dest = &packet.cmds[index];
    return dest->forwardmove;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovl $7, %eax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_global_struct_object_assignment_from_pointer_slice() {
    // given
    let source = r"typedef struct {
    int forwardmove;
} ticcmd_t;
typedef struct {
    int checksum;
    ticcmd_t cmds[4];
} doomdata_t;
doomdata_t* netbuffer;
doomdata_t reboundstore;
int main(void) {
    reboundstore = *netbuffer;
    *netbuffer = reboundstore;
    return reboundstore.checksum;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("reboundstore:"));
    assert!(assembly.contains("\tmovq netbuffer(%rip), %rax\n"));
    assert!(assembly.contains("\tleaq reboundstore(%rip), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_emits_void_functions_with_value_less_return() {
    // given
    let source = "void tick(void) { return; } int main(void) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _tick"));
    assert!(assembly.contains("_tick:\n\tret"));
    assert!(assembly.contains(".globl _main"));
}

#[test]
fn compiler_adds_terminal_return_to_void_functions_that_can_fall_through() {
    // given
    let source = "void tick(void) { if (0) { return; } } int main(void) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains("_tick:\n\tmovz w0, #0"));
    assert!(assembly.contains("Ltick_0:\n\tret"));
}
