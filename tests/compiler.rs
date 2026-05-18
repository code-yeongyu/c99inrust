use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::{parse, parse_supported_translation_unit};

#[test]
fn compiler_emits_native_assembly_for_constant_return_program() {
    // given
    let source = "int main(void) { return 40 + 2; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains(".globl _main"));
            assert!(assembly.contains("movz w0, #40"));
            assert!(assembly.contains("add w0, w0, w1"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains(".globl _main"));
            assert!(assembly.contains("movl $40, %eax"));
            assert!(assembly.contains("addl %ecx, %eax"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains(".globl main"));
            assert!(assembly.contains("movl $40, %eax"));
            assert!(assembly.contains("addl %ecx, %eax"));
        }
    }
    assert!(assembly.contains("ret"));
}

#[test]
fn compiler_emits_stack_slots_for_local_int_assignments() {
    // given
    let source = "int main(void) { int x = 40; int y = x + 1; x = y + 1; return x; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("str w0, [sp, #0]"));
            assert!(assembly.contains("ldr w0, [sp, #0]"));
            assert!(assembly.contains("str w0, [sp, #4]"));
        }
        Target::X86_64AppleDarwin | Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("movl %eax, -4(%rbp)"));
            assert!(assembly.contains("movl -4(%rbp), %eax"));
            assert!(assembly.contains("movl %eax, -8(%rbp)"));
        }
    }
    assert!(assembly.contains("ret"));
}

#[test]
fn compiler_accepts_compound_assignment_slice() {
    // given
    let source = "int main(void) { int x = 40; int y = 8; x += y / 2; x -= 1; return x; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tidivl %ecx\n"));
    assert!(assembly.contains("\taddl %ecx, %eax\n"));
    assert!(assembly.contains("\tsubl %ecx, %eax\n"));
    assert!(assembly.contains("ret"));
}

#[test]
fn compiler_accepts_local_pointer_declaration_slice() {
    // given
    let source = "int Z_Malloc(int size, int tag, void *user) { return 0; } void Z_Free(void *p) { return; } int main(void) { short *dest; dest = (short*) Z_Malloc(8, 1, 0); Z_Free(dest); return dest == 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("Z_Malloc"));
    assert!(assembly.contains("Z_Free"));
    assert!(assembly.contains("\tcltq\n"));
    assert!(assembly.contains("\tmovq %rax, -"));
    assert!(assembly.contains("ret"));
}

#[test]
fn compiler_accepts_local_char_array_string_initializer_slice() {
    // given
    let source = r#"void use(char* name) { name = name; }
int main(void) { char name1[] = "FLOOR7_2"; char* name; name = name1; use(name); return 0; }"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovb $70, -9(%rbp)\n"));
    assert!(assembly.contains("\tmovb $0, -1(%rbp)\n"));
    assert!(assembly.contains("\tleaq -9(%rbp), %rax\n"));
}

#[test]
fn compiler_accepts_local_char_array_decay_slice() {
    // given
    let source = r#"void sprintf(char* out, char* fmt, int value);
int main(void) { char namebuf[9]; sprintf(namebuf, "AMMNUM%d", 7); return 0; }"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tleaq -9(%rbp), %rax\n"));
    assert!(assembly.contains("\tcall sprintf\n"));
}

#[test]
fn compiler_concatenates_adjacent_string_literals_slice() {
    // given
    let source = r#"void use(char* text) { text = text; } int main(void) { use("Z_CT at " "doom" ":%i"); return 0; }"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\t.byte 90,95,67,84,32,97,116,32,100,111,111,109,58,37,105,0\n"));
}

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
    assert!(assembly.contains("\tsubl %ecx, %eax\n"));
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
    assert!(assembly.contains("\taddq $1, %rax\n"));
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
    assert!(assembly.contains("\taddq %rcx, %rax\n"));
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

#[test]
fn compiler_emits_branches_for_if_else_comparisons() {
    // given
    let source =
        "int main(void) { int x = 3; if (x >= 3) { x = 9; } else { x = 1; } return x == 9; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("cmp w0, w1"));
            assert!(assembly.contains("b.lt Lmain_"));
            assert!(assembly.contains("b Lmain_"));
            assert!(assembly.contains("cset w0, eq"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("cmpl %ecx, %eax"));
            assert!(assembly.contains("setge %al"));
            assert!(assembly.contains("je Lmain_"));
            assert!(assembly.contains("sete %al"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("cmpl %ecx, %eax"));
            assert!(assembly.contains("setge %al"));
            assert!(assembly.contains("je .Lmain_"));
            assert!(assembly.contains("sete %al"));
        }
    }
}

#[test]
fn compiler_emits_back_edges_for_while_loops() {
    // given
    let source = "int main(void) { int x = 0; while (x < 5) { x = x + 1; } return x; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("cmp w0, w1"));
            assert!(assembly.contains("b.ge Lmain_"));
            assert!(assembly.contains("add w0, w0, #1"));
            assert!(assembly.contains("b Lmain_"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("setl %al"));
            assert!(assembly.contains("je Lmain_"));
            assert!(assembly.contains("jmp Lmain_"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("setl %al"));
            assert!(assembly.contains("je .Lmain_"));
            assert!(assembly.contains("jmp .Lmain_"));
        }
    }
}

#[test]
fn compiler_emits_back_edges_for_do_while_loops() {
    // given
    let source = "int main(void) { int x = 0; do { x = x + 1; } while (x < 5); return x; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("add w0, w0, #1"));
            assert!(assembly.contains("cmp w0, w1"));
            assert!(assembly.contains("b.ge Lmain_"));
            assert!(assembly.contains("b Lmain_"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("addl %ecx, %eax"));
            assert!(assembly.contains("setl %al"));
            assert!(assembly.contains("je Lmain_"));
            assert!(assembly.contains("jmp Lmain_"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("addl %ecx, %eax"));
            assert!(assembly.contains("setl %al"));
            assert!(assembly.contains("je .Lmain_"));
            assert!(assembly.contains("jmp .Lmain_"));
        }
    }
}

#[test]
fn compiler_accepts_doom_do_while_pointer_copy_slice() {
    // given
    let source = "void copy(int *buffer, int *p) { int c; do { c = *p; *(buffer++) = c; *(p++) = 0; } while (c && *p != 255); } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("copy:"));
    assert!(assembly.contains("\taddq $1, %rax\n"));
    assert!(assembly.contains("\tmovl %eax, (%rcx,%rdx,4)\n"));
}

#[test]
fn compiler_emits_short_circuit_logical_branches() {
    // given
    let source = "int main(void) { int x = 0; if (x != 0 && 10 / x > 1) { return 1; } if (x == 0 || 10 / x > 1) { return 42; } return 2; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("b.eq Lmain_"));
            assert!(assembly.contains("b.ne Lmain_"));
            assert!(assembly.contains("sdiv w0, w0, w1"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("je Lmain_"));
            assert!(assembly.contains("jne Lmain_"));
            assert!(assembly.contains("idivl %ecx"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("je .Lmain_"));
            assert!(assembly.contains("jne .Lmain_"));
            assert!(assembly.contains("idivl %ecx"));
        }
    }
}

#[test]
fn compiler_emits_for_loop_back_edges() {
    // given
    let source = "int main(void) { int total = 0; for (int i = 0; i < 5; i = i + 1) { total = total + i; } return total; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("cmp w0, w1"));
            assert!(assembly.contains("b.ge Lmain_"));
            assert!(assembly.contains("add w0, w0, #1"));
            assert!(assembly.contains("b Lmain_"));
            assert!(assembly.contains("str w0, [sp, #4]"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("setl %al"));
            assert!(assembly.contains("je Lmain_"));
            assert!(assembly.contains("jmp Lmain_"));
            assert!(assembly.contains("movl %eax, -8(%rbp)"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("setl %al"));
            assert!(assembly.contains("je .Lmain_"));
            assert!(assembly.contains("jmp .Lmain_"));
            assert!(assembly.contains("movl %eax, -8(%rbp)"));
        }
    }
}

#[test]
fn compiler_accepts_for_comma_expression_slice() {
    // given
    let source = r"int main(void) {
    int index;
    int k;
    int total;
    total = 0;
    for (index = 0, k = 1; k < 4; index++, k++) {
        total += index;
    }
    return total;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\taddl"));
    assert!(assembly.contains("\tjmp .Lmain_"));
}

#[test]
fn compiler_accepts_break_statement_slice() {
    // given
    let source = r"int main(void) {
    int x;
    x = 0;
    for (;;) {
        x = 1;
        break;
        x = 2;
    }
    return x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tjmp .Lmain_"));
}

#[test]
fn compiler_accepts_local_static_scalar_declaration_slice() {
    // given
    let source = r"int gamemap;
int main(void) {
    static nexttic = 0;
    static int lastlevel = -1, lastepisode = -1;
    if (lastlevel != gamemap) {
        lastlevel = gamemap + nexttic;
    }
    return lastepisode;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovl $0, %eax\n\tmovl %eax, -4(%rbp)\n"));
    assert!(assembly.contains("\tnegl %eax\n\tmovl %eax, -8(%rbp)\n"));
    assert!(assembly.contains("\tnegl %eax\n\tmovl %eax, -12(%rbp)\n"));
}

#[test]
fn compiler_accepts_doom_enum_typedef_scalar_slice() {
    // given
    let source = r"typedef enum {
    GS_LEVEL,
    GS_DEMOSCREEN
} gamestate_t;
typedef enum {
    sk_baby,
    sk_nightmare
} skill_t;
int display(skill_t skill) {
    static gamestate_t oldgamestate = -1;
    return oldgamestate + skill;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("display:"));
    assert!(assembly.contains("\tnegl %eax\n\tmovl %eax, -"));
    assert!(assembly.contains("\taddl"));
}

#[test]
fn compiler_accepts_local_int_array_sizeof_slice() {
    // given
    let source = r"int main(void) {
    static int values[] = { 0, 4, 7 };
    static int index = 0;
    int out;
    out = values[index++];
    if (index == sizeof(values)/sizeof(int)) index = 0;
    return out;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovl $0, -12(%rbp)\n"));
    assert!(assembly.contains("\tmovl $4, -8(%rbp)\n"));
    assert!(assembly.contains("\tmovl $7, -4(%rbp)\n"));
    assert!(assembly.contains("\tmovl $12, %eax\n"));
}

#[test]
fn compiler_accepts_local_int_array_global_enum_initializers_slice() {
    // given
    let source = r"typedef enum {
    mus_None,
    mus_e3m2,
    mus_e3m4
} musicenum_t;
int main(void) {
    int spmus[] = { mus_e3m4, mus_e3m2 };
    return spmus[0];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovl $2, -8(%rbp)\n"));
    assert!(assembly.contains("\tmovl $1, -4(%rbp)\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_local_array_of_enum_typedef_slice() {
    // given
    let source = r"typedef enum {
    DI_EAST,
    DI_WEST
} dirtype_t;
int main(void) {
    dirtype_t d[3];
    d[0] = DI_WEST;
    return d[0];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovl %eax, (%rcx,%rdx,4)\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_enum_typedef_parameter_slice() {
    // given
    let source = r"typedef enum {
    lowerToFloor,
    raiseToHighest
} ceiling_e;
int apply(ceiling_e type) {
    return type;
}
int main(void) {
    return apply(raiseToHighest);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("apply:"));
    assert!(assembly.contains("\tmovl %edi, -4(%rbp)\n"));
    assert!(assembly.contains("\tcall apply\n"));
}

#[test]
fn compiler_accepts_local_pointer_array_slice() {
    // given
    let source = r#"void use(char* value);
int main(void) {
    char *moreargs[20];
    moreargs[0] = "abc";
    use(moreargs[0]);
    return sizeof(moreargs);
}"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\t.byte 97,98,99,0\n"));
    assert!(assembly.contains("\tcall use\n"));
    assert!(assembly.contains("\tmovl $160, %eax\n"));
}

#[test]
fn compiler_accepts_local_char_matrix_row_decay_slice() {
    // given
    let source = r#"void use(char* value);
int main(void) {
    char name[3][8] = {
        "e2m1",
        "dphoof",
        "spida1"
    };
    int i = 1;
    use(name[i]);
    return sizeof(name);
}"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovb $101,"));
    assert!(assembly.contains("\tcall use\n"));
    assert!(assembly.contains("\tmovl $24, %eax\n"));
}

#[test]
fn compiler_accepts_switch_case_break_slice() {
    // given
    let source = r"int main(void) {
    int key;
    int result;
    key = 2;
    result = 0;
    switch (key) {
      case 1:
        result = 10;
        break;
      case '-':
        result = 20;
        break;
      default:
        result = 30;
    }
    return result;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovl $45, %eax\n"));
    assert!(assembly.contains("\tsete %al\n"));
    assert!(assembly.contains("\tjmp .Lmain_"));
}

#[test]
fn compiler_accepts_continue_statement_slice() {
    // given
    let source = r"int main(void) {
    int i = 0;
    int out = 0;
    for (i = 0; i < 4; i++) {
        if (i == 2) continue;
        out += i;
    }
    return out;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\taddl %ecx, %eax\n"));
    assert!(assembly.contains("\tjmp .Lmain_"));
}

#[test]
fn compiler_accepts_local_enum_and_register_implicit_int_slice() {
    // given
    let source = r"int main(void) {
    enum {
        LEFT = 1,
        RIGHT = 2
    };
    register outcode = 0;
    outcode = LEFT | RIGHT;
    return outcode;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovl $1, %eax\n"));
    assert!(assembly.contains("\tmovl $2, %eax\n"));
    assert!(assembly.contains("\torl %ecx, %eax\n"));
}

#[test]
fn compiler_emits_zero_arg_function_calls() {
    // given
    let source = "int answer(void) { int value = 40; return value; } int main(void) { return 2 + answer(); }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains(".globl _answer"));
            assert!(assembly.contains("str x30, [sp, #"));
            assert!(assembly.contains("bl _answer"));
            assert!(assembly.contains("ldr x30, [sp, #"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains(".globl _answer"));
            assert!(assembly.contains("call _answer"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains(".globl answer"));
            assert!(assembly.contains("call answer"));
        }
    }
    assert!(assembly.contains("ret"));
}

#[test]
fn compiler_emits_integer_function_call_arguments() {
    // given
    let source = "int add(int left, int right) { return left + right; } int main(int argc, char **argv) { return add(argc, 41); }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("\tldr w0, [sp, #0]\n"));
            assert!(assembly.contains("\tstr x1, [sp, #8]\n"));
            assert!(assembly.contains("\tmovz w0, #41\n"));
            assert!(assembly.contains("\tldr w0, [sp, #16]\n"));
            assert!(assembly.contains("\tldr w1, [sp, #24]\n"));
            assert!(assembly.contains("\tbl _add\n"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("\tmovl -4(%rbp), %eax\n"));
            assert!(assembly.contains("\tmovq %rsi, -16(%rbp)\n"));
            assert!(assembly.contains("\tmovl $41, %eax\n"));
            assert!(assembly.contains("\tmovl -20(%rbp), %edi\n"));
            assert!(assembly.contains("\tmovl -28(%rbp), %esi\n"));
            assert!(assembly.contains("\tcall _add\n"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("\tmovl -4(%rbp), %eax\n"));
            assert!(assembly.contains("\tmovq %rsi, -16(%rbp)\n"));
            assert!(assembly.contains("\tmovl $41, %eax\n"));
            assert!(assembly.contains("\tmovl -20(%rbp), %edi\n"));
            assert!(assembly.contains("\tmovl -28(%rbp), %esi\n"));
            assert!(assembly.contains("\tcall add\n"));
        }
    }
}

#[test]
fn compiler_emits_conditional_expression_branches() {
    // given
    let source = "int main(int argc, char **argv) { return argc < 0 ? 2 : 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("\tb.eq Lmain_"));
            assert!(assembly.contains("\tmovz w0, #2\n"));
            assert!(assembly.contains("\tmovz w0, #42\n"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("\tje Lmain_"));
            assert!(assembly.contains("\tmovl $2, %eax\n"));
            assert!(assembly.contains("\tmovl $42, %eax\n"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("\tje .Lmain_"));
            assert!(assembly.contains("\tmovl $2, %eax\n"));
            assert!(assembly.contains("\tmovl $42, %eax\n"));
        }
    }
}

#[test]
fn aarch64_keeps_binary_left_operand_in_preserved_register_across_direct_call() {
    // given
    let source = "int answer(void) { int value = 40; return value; } int main(void) { return 2 + answer(); }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains("str x19, [sp, #"));
    assert!(assembly.contains("mov w19, w0"));
    assert!(assembly.contains("bl _answer"));
    assert!(assembly.contains("mov w0, w19"));
    assert!(assembly.contains("ldr x19, [sp, #"));
}

#[test]
fn compiler_folds_calls_to_integer_constant_functions() {
    // given
    let source = "int tick(void) { return 1; } int main(void) { return tick(); }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _tick"));
    assert!(assembly.contains(".globl _main"));
    assert!(assembly.contains("movz w0, #1"));
    assert!(!assembly.contains("\tbl _tick"));
}

#[test]
fn compiler_skips_top_level_declarations_before_supported_functions() {
    // given
    let source = "static const char rcsid[] = \"doom\"; int main(void) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _main"));
    assert!(assembly.contains("movz w0, #42"));
}

#[test]
fn compiler_accepts_ignorable_static_metadata_translation_unit() {
    // given
    let source = "static const char rcsid[] = \"doom\";";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(program.functions.is_empty());
    assert!(program.globals.is_empty());
    assert!(assembly.is_empty());
}

#[test]
fn compiler_rejects_unsupported_data_only_translation_unit() {
    // given
    let source = "typedef int weaponinfo_t; weaponinfo_t* weaponinfo[2] = { 0, 0 };";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let error = parse_supported_translation_unit(&tokens)
        .expect_err("translation unit should reject unsupported data-only globals");

    // then
    assert!(
        error
            .to_string()
            .contains("translation unit has no supported function definitions")
    );
}

#[test]
fn compiler_accepts_doomstat_enum_globals_slice() {
    // given
    let source = r"typedef enum { shareware, registered, commercial, retail, indetermined } GameMode_t;
typedef enum { doom, doom2, pack_tnt, pack_plut, none } GameMission_t;
typedef enum { english, french, german, unknown } Language_t;
typedef int boolean;
GameMode_t gamemode = indetermined;
GameMission_t gamemission = doom;
Language_t language = english;
boolean modifiedgame;";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("gamemode:"));
    assert!(assembly.contains("gamemission:"));
    assert!(assembly.contains("language:"));
    assert!(assembly.contains("modifiedgame:"));
    assert!(assembly.contains("\t.long 4\n"));
    assert!(assembly.contains("\t.long 0\n"));
}

#[test]
fn compiler_accepts_doomstat_globals_after_header_extern_arrays_slice() {
    // given
    let source = r#"static const char rcsid[] = "doom";
typedef enum { shareware, registered, commercial, retail, indetermined } GameMode_t;
typedef int boolean;
extern int finesine[5*8192/4];
extern char *sprnames[138];
extern boolean nomonsters;
extern GameMode_t gamemode;
GameMode_t gamemode = indetermined;
boolean modifiedgame;"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("gamemode:"));
    assert!(assembly.contains("modifiedgame:"));
    assert!(assembly.contains("\t.long 4\n"));
    assert!(assembly.contains("\t.long 0\n"));
}

#[test]
fn compiler_accepts_unparenthesized_global_integer_initializer_slice() {
    // given
    let source = "static int finit_height = 200 - 32;";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("finit_height:"));
    assert!(assembly.contains("\t.long 168\n"));
}

#[test]
fn compiler_accepts_fixed_point_global_initializer_slice() {
    // given
    let source = "typedef int fixed_t; static fixed_t scale_mtof = (.2*(1<<16));";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("scale_mtof:"));
    assert!(assembly.contains("\t.long 13107\n"));
}

#[test]
fn compiler_accepts_aggregate_global_initializer_before_supported_function() {
    // given
    let source = r"typedef struct {
    unsigned char *sequence;
    unsigned char *p;
} cheatseq_t;
static unsigned char cheat_amap_seq[] = { 0xb2, 0x26, 0xff };
static cheatseq_t cheat_amap = { cheat_amap_seq, 0 };
int main(void) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("cheat_amap_seq:"));
    assert!(assembly.contains("cheat_amap:"));
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("movl $42, %eax"));
}

#[test]
fn compiler_accepts_aggregate_global_address_slice() {
    // given
    let source = r"typedef struct {
    unsigned char *sequence;
    int offset;
} cheatseq_t;
static unsigned char cheat_amap_seq[] = { 0xb2, 0x26, 0xff };
static cheatseq_t cheat_amap = { cheat_amap_seq, 0 };
void use(cheatseq_t* value);
int main(void) {
    use(&cheat_amap);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("cheat_amap:"));
    assert!(assembly.contains("\tleaq cheat_amap(%rip), %rax\n"));
    assert!(assembly.contains("\tcall use\n"));
}

#[test]
fn compiler_accepts_struct_array_initializer_before_supported_function() {
    // given
    let source = r"typedef struct {
    int x;
    int y;
} mpoint_t;
typedef struct {
    mpoint_t a;
    mpoint_t b;
} mline_t;
mline_t player_arrow[] = {
    { { -8, 0 }, { 8, 0 } },
    { { 8, 0 }, { 0, 8 } }
};
int main(void) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("player_arrow:"));
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("movl $42, %eax"));
}

#[test]
fn compiler_accepts_global_struct_array_decay_slice() {
    // given
    let source = r"typedef struct {
    int x;
} mline_t;
mline_t player_arrow[] = {
    { 1 },
    { 2 }
};
void draw(mline_t* lines);
int main(void) {
    draw(player_arrow);
    return sizeof(player_arrow);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("player_arrow:"));
    assert!(assembly.contains("\tleaq player_arrow(%rip), %rax\n"));
    assert!(assembly.contains("\tcall draw\n"));
    assert!(assembly.contains("\tmovl $8, %eax\n"));
}

#[test]
fn compiler_accepts_extern_struct_array_before_definition_slice() {
    // given
    let source = r"typedef struct {
    int ammo;
    int upstate;
} weaponinfo_t;
extern weaponinfo_t weaponinfo[NUMWEAPONS];
weaponinfo_t weaponinfo[NUMWEAPONS] = {
    { 1, 2 },
    { 3, 4 },
    { 5, 6 }
};
int main(void) {
    return sizeof(weaponinfo);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert_eq!(assembly.matches("weaponinfo:\n").count(), 1);
    assert!(assembly.contains("\t.zero 24\n"));
    assert!(assembly.contains("\tmovl $24, %eax\n"));
}

#[test]
fn compiler_accepts_multi_declarator_local_int_slice() {
    // given
    let source = "int main(void) { int dx, dy; dx = 40; dy = 2; return dx + dy; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("movl %eax, -4(%rbp)"));
    assert!(assembly.contains("movl %eax, -8(%rbp)"));
    assert!(assembly.contains("addl %ecx, %eax"));
}

#[test]
fn compiler_accepts_global_int_declarator_list_slice() {
    // given
    let source = r"typedef int fixed_t;
static fixed_t m_x, m_y;
int main(void) {
    m_x = 4;
    m_y = m_x + 2;
    return m_y;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("m_x:"));
    assert!(assembly.contains("m_y:"));
    assert!(assembly.contains("\tmovl %eax, m_x(%rip)\n"));
    assert!(assembly.contains("\tmovl %eax, m_y(%rip)\n"));
}

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

#[test]
fn compiler_rejects_value_less_return_from_int_functions() {
    // given
    let source = "int main(void) { return; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let error = lower(&program).expect_err("lowering should reject a value-less int return");

    // then
    assert!(
        error
            .to_string()
            .contains("int function must return a value")
    );
}

#[test]
fn compiler_rejects_value_return_from_void_functions() {
    // given
    let source = "void tick(void) { return 1; } int main(void) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let error = lower(&program).expect_err("lowering should reject a valued void return");

    // then
    assert!(
        error
            .to_string()
            .contains("void function cannot return a value")
    );
}

#[test]
fn compiler_accepts_parameter_list_signatures_when_body_does_not_use_parameters() {
    // given
    let source = "int main(int argc, char **argv) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _main"));
    assert!(assembly.contains("movz w0, #42"));
}

#[test]
fn compiler_accepts_unsigned_parameter_slice() {
    // given
    let source = "void R_VideoErase(unsigned ofs, int count) { ofs = ofs + count; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("R_VideoErase:"));
    assert!(assembly.contains("\taddl %ecx, %eax\n"));
}

#[test]
fn compiler_binds_parameters_as_local_slots_on_aarch64() {
    // given
    let source = "int identity(int value) { return value; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains("_identity:\n\tsub sp, sp, #16\n\tstr w0, [sp, #0]"));
    assert!(assembly.contains("\tldr w0, [sp, #0]"));
}

#[test]
fn compiler_binds_parameters_as_local_slots_on_x86_64() {
    // given
    let source = "int identity(int value) { return value; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("identity:\n\tpushq %rbp"));
    assert!(assembly.contains("\tmovl %edi, -4(%rbp)"));
    assert!(assembly.contains("\tmovl -4(%rbp), %eax"));
}

#[test]
fn compiler_emits_signed_long_long_cast_intermediates() {
    // given
    let source = "typedef int fixed_t; fixed_t FixedMul(fixed_t a, fixed_t b) { return ((long long) a * (long long) b) >> 16; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("_FixedMul:\n"));
            assert!(assembly.contains("\tsxtw x0, w0\n"));
            assert!(assembly.contains("\tmul x0, x0, x1\n"));
            assert!(assembly.contains("\tasr x0, x0, x1\n"));
        }
        Target::X86_64AppleDarwin | Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("FixedMul:\n") || assembly.contains("_FixedMul:\n"));
            assert!(assembly.contains("\tmovl -4(%rbp), %eax\n\tcltq\n"));
            assert!(assembly.contains("\timulq %rcx, %rax\n"));
            assert!(assembly.contains("\tsarq %cl, %rax\n"));
        }
    }
}

#[test]
fn compiler_accepts_fixeddiv2_double_slice() {
    // given
    let source = r#"typedef int fixed_t;
void I_Error(char *message) { return; }
fixed_t FixedDiv2(fixed_t a, fixed_t b) {
    double c;
    c = ((double)a) / ((double)b) * (1<<16);
    if (c >= 2147483648.0 || c < -2147483648.0)
        I_Error("FixedDiv: divide by zero");
    return (fixed_t)c;
}
int main(void) { return FixedDiv2(3, 2) == 98304 ? 0 : 1; }"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    assert!(assembly.contains("FixedDiv2"));
    assert!(assembly.contains("I_Error"));
    assert!(assembly.contains(
        ".byte 70,105,120,101,100,68,105,118,58,32,100,105,118,105,100,101,32,98,121,32,122,101,114,111,0"
    ));
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("\tfdiv d0, d0, d1\n"));
            assert!(assembly.contains("\tfmul d0, d0, d1\n"));
            assert!(assembly.contains("\tfcmp d0, d1\n"));
        }
        Target::X86_64AppleDarwin | Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("\tdivsd %xmm1, %xmm0\n"));
            assert!(assembly.contains("\tmulsd %xmm1, %xmm0\n"));
            assert!(assembly.contains("\tucomisd %xmm1, %xmm0\n"));
        }
    }
}

#[test]
fn compiler_accepts_m_random_global_array_slice() {
    // given
    let source = r"unsigned char rndtable[4] = { 3, 5, 7, 11 };
int rndindex = 0;
int prndindex = 0;
int P_Random(void) {
    prndindex = (prndindex + 1) & 0x3;
    return rndtable[prndindex];
}
int M_Random(void) {
    rndindex = (rndindex + 1) & 0x3;
    return rndtable[rndindex];
}
void M_ClearRandom(void) {
    rndindex = prndindex = 0;
}
int main(void) {
    int a = P_Random();
    int b = M_Random();
    M_ClearRandom();
    return a == 5 && b == 5 ? 0 : 1;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly = emit_assembly(&lowered, Target::native()).expect("assembly should emit");

    // then
    assert!(assembly.contains("rndtable"));
    assert!(assembly.contains("rndindex"));
    assert!(assembly.contains("prndindex"));
    assert!(assembly.contains("M_ClearRandom"));
}

#[test]
fn compiler_accepts_global_pointer_array_slice() {
    // given
    let source = r"int* ylookup[4];
int main(void) {
    int* p;
    p = 0;
    ylookup[2] = p;
    return ylookup[2] ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("ylookup:"));
    assert!(assembly.contains("\t.zero 32\n"));
    assert!(assembly.contains("\tmovq %rax, (%rcx,%rdx,8)\n"));
    assert!(assembly.contains("\tmovq (%rcx,%rax,8), %rax\n"));
}

#[test]
fn compiler_accepts_typed_global_struct_pointer_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
} point_t;
static point_t *cursor;
int main(void) {
    return cursor->x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("cursor:"));
    assert!(assembly.contains("\tmovq cursor(%rip), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_struct_fields_with_typedef_and_array_slice() {
    // given
    let source = r"typedef int state_t;
typedef struct {
    state_t state;
    int powers[4];
    int x;
} player_t;
static player_t *plr;
int main(void) {
    return plr->x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("plr:"));
    assert!(assembly.contains("\tmovq plr(%rip), %rax\n"));
    assert!(assembly.contains("\tmovl 20(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_nested_typed_pointer_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
} mobj_t;
typedef struct {
    mobj_t* mo;
} player_t;
static player_t *plr;
int main(void) {
    return plr->mo->x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("plr:"));
    assert!(assembly.contains("\tmovq plr(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq 0(%rax), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_pointer_subscript_struct_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
} vertex_t;
typedef struct {
    vertex_t* v1;
} line_t;
static line_t *lines;
int main(void) {
    int i = 0;
    return lines[i].v1->x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("lines:"));
    assert!(assembly.contains("\tmovq lines(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq 0(%rax), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_extern_pointer_subscript_struct_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
} vertex_t;
typedef struct {
    vertex_t* v1;
} line_t;
extern line_t *lines;
int main(void) {
    int i = 0;
    return lines[i].v1->x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("lines:\n"));
    assert!(assembly.contains("\tmovq lines(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq 0(%rax), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_global_struct_array_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
    int y;
} mpoint_t;
static mpoint_t markpoints[4];
int main(void) {
    int i = 0;
    markpoints[i].x = 1;
    return markpoints[i].y;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("markpoints:"));
    assert!(assembly.contains("\t.zero 32\n"));
    assert!(assembly.contains("\tleaq markpoints(%rip), %rax\n"));
    assert!(assembly.contains("\tmovl %eax, 0(%rcx)\n"));
    assert!(assembly.contains("\tmovl 4(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_global_struct_object_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
    int y;
} mpoint_t;
static mpoint_t m_paninc;
int main(void) {
    m_paninc.x = 1;
    return m_paninc.y;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("m_paninc:"));
    assert!(assembly.contains("\t.zero 8\n"));
    assert!(assembly.contains("\tleaq m_paninc(%rip), %rax\n"));
    assert!(assembly.contains("\tmovl %eax, 0(%rcx)\n"));
    assert!(assembly.contains("\tmovl 4(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_extern_struct_array_address_slice() {
    // given
    let source = r"typedef struct {
    int x;
} player_t;
extern player_t players[4];
int main(void) {
    player_t* p;
    int i = 0;
    p = &players[i];
    return p->x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("players:\n"));
    assert!(assembly.contains("\tleaq players(%rip), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_nested_extern_struct_array_address_slice() {
    // given
    let source = r"typedef struct {
    int forwardmove;
} ticcmd_t;
extern ticcmd_t netcmds[MAXPLAYERS][BACKUPTICS];
void build(ticcmd_t* cmd);
int main(void) {
    int player = 0;
    int tic = 1;
    build(&netcmds[player][tic]);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("netcmds:\n"));
    assert!(assembly.contains("\tleaq netcmds(%rip), %rax\n"));
    assert!(assembly.contains("\tcall build\n"));
}

#[test]
fn compiler_accepts_extern_int_array_slice() {
    // given
    let source = r"extern int playeringame[MAXPLAYERS];
int main(void) {
    int i = 0;
    return playeringame[i];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("playeringame:\n"));
    assert!(assembly.contains("\tleaq playeringame(%rip), %rcx\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_block_extern_int_array_slice() {
    // given
    let source = r"int main(void) {
    extern int forwardmove[2];
    int scale = 2;
    forwardmove[0] = forwardmove[0] * scale;
    return forwardmove[0];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("forwardmove:\n"));
    assert!(assembly.contains("\tleaq forwardmove(%rip), %rcx\n"));
    assert!(assembly.contains("\tmovl %eax, (%rcx,%rdx,4)\n"));
}

#[test]
fn compiler_accepts_pointer_to_pointer_subscript_address_slice() {
    // given
    let source = r"extern char** myargv;
void use(char* value);
int main(void) {
    int i = 0;
    use(&myargv[i][1]);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovq myargv(%rip), %rax\n"));
    assert!(assembly.contains("\tcall use\n"));
}

#[test]
fn compiler_emits_byte_access_for_char_pointer_dereference_slice() {
    // given
    let source = r#"int main(void) {
    char* infile;
    int k;
    infile = "az";
    k = 1;
    *(infile+k) = 0;
    return *(infile+k);
}"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovb %al, (%rcx,%rdx,1)\n"));
    assert!(assembly.contains("\tmovzbl (%rcx,%rax,1), %eax\n"));
}

#[test]
fn compiler_emits_byte_access_for_char_pointer_nested_subscript_slice() {
    // given
    let source = r"extern char** myargv;
int main(void) {
    return myargv[1][0];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovq myargv(%rip), %rax\n"));
    assert!(assembly.contains("\tmovzbl (%rcx,%rax,1), %eax\n"));
}

#[test]
fn compiler_accepts_opaque_file_pointer_local_slice() {
    // given
    let source = r"int main(void) {
    FILE* handle;
    handle = 0;
    return handle == 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovq %rax, -8(%rbp)\n"));
}

#[test]
fn compiler_accepts_sizeof_struct_typedef_slice() {
    // given
    let source = r"typedef struct {
    int id;
    int tag;
} memblock_t;
int main(void) {
    return sizeof(memblock_t);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovl $8, %eax\n"));
}

#[test]
fn compiler_accepts_typed_pointer_cast_member_slice() {
    // given
    let source = r"typedef struct {
    int id;
} memblock_t;
int main(void) {
    int* raw;
    raw = 0;
    return ((memblock_t*)raw)->id;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_struct_array_field_subscript_slice() {
    // given
    let source = r"typedef struct {
    int powers[4];
} player_t;
static player_t *plr;
int main(void) {
    return plr->powers[2];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovq plr(%rip), %rax\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_struct_member_address_slice() {
    // given
    let source = r"typedef struct {
    int x;
    int y;
} mpoint_t;
typedef struct {
    mpoint_t a;
} mline_t;
void rotate(int* x);
int main(void) {
    mline_t l;
    rotate(&l.a.x);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tleaq -8(%rbp), %rax\n"));
    assert!(assembly.contains("\tcall rotate\n"));
}

#[test]
fn compiler_accepts_standard_stream_global_slice() {
    // given
    let source = r"void use(int* stream);
int main(void) {
    use(stderr);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovq stderr(%rip), %rax\n"));
    assert!(assembly.contains("\tcall use\n"));
}

#[test]
fn compiler_accepts_extern_global_pointer_array_slice() {
    // given
    let source = r"typedef unsigned char byte;
extern byte* screens[5];
int main(void) {
    return screens[0] ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("screens:\n"));
    assert!(assembly.contains("\tleaq screens(%rip), %rcx\n"));
    assert!(assembly.contains("\tmovq (%rcx,%rax,8), %rax\n"));
}

#[test]
fn compiler_accepts_extern_global_pointer_array_symbolic_length() {
    // given
    let source = r"enum { NUMSPRITES = 2 };
extern char *sprnames[NUMSPRITES];
int main(void) {
    return sprnames[1] ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("sprnames:\n"));
    assert!(assembly.contains("\tleaq sprnames(%rip), %rcx\n"));
    assert!(assembly.contains("\tmovq (%rcx,%rax,8), %rax\n"));
}

#[test]
fn compiler_accepts_extern_short_array_slice() {
    // given
    let source = r"extern short ceilingclip[2];
int main(void) {
    return ceilingclip[1];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("ceilingclip:\n"));
    assert!(assembly.contains("\tleaq ceilingclip(%rip), %rcx\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_global_short_array_expression_length_slice() {
    // given
    let source = r"short openings[2*3];
int main(void) {
    openings[5] = 7;
    return openings[5];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("openings:"));
    assert!(assembly.contains("\t.zero 24\n"));
    assert!(assembly.contains("\tmovl %eax, (%rcx,%rdx,4)\n"));
}

#[test]
fn compiler_accepts_global_enum_sized_int_array_slice() {
    // given
    let source = r"enum { NUMAMMO = 4 };
int maxammo[NUMAMMO] = { 200, 50, NUMAMMO, NUMAMMO + 1 };
int main(void) {
    return maxammo[2];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("maxammo:"));
    assert!(assembly.contains("\t.long 200,50,4,5\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_unsized_global_enum_int_array_slice() {
    // given
    let source = r"typedef enum {
    DI_EAST,
    DI_WEST
} dirtype_t;
dirtype_t opposite[] = { DI_WEST, DI_EAST };
int main(void) {
    return opposite[0];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("opposite:"));
    assert!(assembly.contains("\t.long 1,0\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_global_enum_sized_pointer_array_slice() {
    // given
    let source = r"enum { NUMCARDS = 3 };
char *keys[NUMCARDS];
int main(void) {
    return keys[2] ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("keys:"));
    assert!(assembly.contains("\t.zero 24\n"));
    assert!(assembly.contains("\tmovq (%rcx,%rax,8), %rax\n"));
}

#[test]
fn compiler_accepts_global_pointer_string_initializer_slice() {
    // given
    let source = r#"char* e1text = "E1";
char* finaletext;
int main(void) {
    finaletext = e1text;
    return finaletext ? 0 : 1;
}"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("e1text:\n\t.quad .Le1text_str0\n"));
    assert!(assembly.contains(".Le1text_str0:\n\t.byte 69,49,0\n"));
    assert!(assembly.contains("\tmovq e1text(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq %rax, finaletext(%rip)\n"));
}

#[test]
fn compiler_accepts_global_pointer_string_array_initializer_slice() {
    // given
    let source = r#"enum { NUM_QUITMESSAGES = 2 };
char* endmsg[NUM_QUITMESSAGES+1] = { "A", "B" "C", "D", };"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("endmsg:\n\t.quad .Lendmsg_str0\n"));
    assert!(assembly.contains("\t.quad .Lendmsg_str1\n"));
    assert!(assembly.contains("\t.quad .Lendmsg_str2\n"));
    assert!(assembly.contains(".Lendmsg_str1:\n\t.byte 66,67,0\n"));
}

#[test]
fn compiler_accepts_extern_typed_pointer_array_member_slice() {
    // given
    let source = r"typedef struct {
    int width;
} patch_t;
extern patch_t *hu_font[2];
int main(void) {
    return hu_font[1]->width;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("hu_font:\n"));
    assert!(assembly.contains("\tleaq hu_font(%rip), %rcx\n"));
    assert!(assembly.contains("\tmovq (%rcx,%rax,8), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_struct_typedef_alias_pointer_member_slice() {
    // given
    let source = r"typedef struct {
    int length;
} post_t;
typedef post_t column_t;
int main(void) {
    column_t* column;
    column = 0;
    return column->length;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_global_int_array_slice() {
    // given
    let source = r"int columnofs[4];
int main(void) {
    columnofs[2] = 7;
    return columnofs[2];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("columnofs:"));
    assert!(assembly.contains("\t.zero 16\n"));
    assert!(assembly.contains("\tmovl %eax, (%rcx,%rdx,4)\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_initialized_global_int_array_slice() {
    // given
    let source = r"int fuzzoffset[4] = { 320, -320, (320), -(320) };
int main(void) {
    return fuzzoffset[0] + fuzzoffset[1] + fuzzoffset[2] + fuzzoffset[3];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("fuzzoffset:"));
    assert!(assembly.contains("\t.long 320,-320,320,-320\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_lighttable_pointer_global_slice() {
    // given
    let source = r"typedef unsigned char byte;
typedef byte lighttable_t;
lighttable_t* dc_colormap;
int main(void) {
    dc_colormap = 0;
    return dc_colormap ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("dc_colormap:"));
    assert!(assembly.contains("\t.quad 0\n"));
    assert!(assembly.contains("\tmovq %rax, dc_colormap(%rip)\n"));
    assert!(assembly.contains("\tmovq dc_colormap(%rip), %rax\n"));
}

#[test]
fn compiler_accepts_angle_t_parameter_slice() {
    // given
    let source = r"typedef unsigned int angle_t;
int rotate(angle_t angle) {
    return angle >> 19;
}
int main(void) { return rotate(0); }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("rotate:"));
    assert!(assembly.contains("\tsarl %cl, %eax\n"));
}

#[test]
fn compiler_accepts_m_cheat_xlate_table_slice() {
    // given
    let source = r"static unsigned char cheat_xlate_table[256];
int main(void) {
    int i;
    for (i = 0; i < 4; i++) cheat_xlate_table[i] = i + 1;
    return cheat_xlate_table[(unsigned char)2];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly = emit_assembly(&lowered, Target::native()).expect("assembly should emit");

    // then
    assert!(assembly.contains("cheat_xlate_table"));
    assert!(assembly.contains(".byte 0,0,0,0"));
    assert!(assembly.contains("main"));
}

#[test]
fn compiler_accepts_m_bbox_pointer_subscript_slice() {
    // given
    let source = r"enum { BOXTOP, BOXBOTTOM, BOXLEFT, BOXRIGHT };
void M_ClearBox(int *box) {
    box[BOXTOP] = box[BOXRIGHT] = -1;
    box[BOXBOTTOM] = box[BOXLEFT] = 10;
}
void M_AddToBox(int *box, int x, int y) {
    if (x < box[BOXLEFT])
        box[BOXLEFT] = x;
    else if (x > box[BOXRIGHT])
        box[BOXRIGHT] = x;
    if (y < box[BOXBOTTOM])
        box[BOXBOTTOM] = y;
    else if (y > box[BOXTOP])
        box[BOXTOP] = y;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly = emit_assembly(&lowered, Target::native()).expect("assembly should emit");
    let linux_x86_64_assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("linux assembly should emit");

    // then
    assert!(assembly.contains("M_ClearBox"));
    assert!(assembly.contains("M_AddToBox"));
    match Target::native() {
        Target::Aarch64AppleDarwin => assert!(assembly.contains("sxtw #2")),
        Target::X86_64AppleDarwin | Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains(",%rax,4)"));
        }
    }
    assert!(linux_x86_64_assembly.contains("(%rcx,%rax,4)"));
    assert!(linux_x86_64_assembly.contains("(%rcx,%rdx,4)"));
}

#[test]
fn compiler_accepts_i_main_global_pointer_slice() {
    // given
    let source = r"int myargc;
char **myargv;
void D_DoomMain(void) { return; }
int main(int argc, char **argv) {
    myargc = argc;
    myargv = argv;
    D_DoomMain();
    if (myargv != argv)
        return 2;
    return myargc == argc ? 0 : 1;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly = emit_assembly(&lowered, Target::native()).expect("assembly should emit");
    let linux_x86_64_assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("linux assembly should emit");

    // then
    assert!(assembly.contains("myargc"));
    assert!(assembly.contains("myargv"));
    assert!(assembly.contains("D_DoomMain"));
    assert!(linux_x86_64_assembly.contains("\t.quad 0\n"));
    assert!(linux_x86_64_assembly.contains("\tmovq %rax, myargv(%rip)\n"));
    assert!(linux_x86_64_assembly.contains("\tmovq myargv(%rip), %rax\n"));
}

#[test]
fn compiler_accepts_i_main_extern_global_slice() {
    // given
    let source = r"extern int myargc;
extern char **myargv;
void D_DoomMain(void);
int main(int argc, char **argv) {
    myargc = argc;
    myargv = argv;
    D_DoomMain();
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let linux_x86_64_assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("linux assembly should emit");

    // then
    assert!(!linux_x86_64_assembly.contains("\t.long 0\n"));
    assert!(!linux_x86_64_assembly.contains("\t.quad 0\n"));
    assert!(linux_x86_64_assembly.contains("\tmovl %eax, myargc(%rip)\n"));
    assert!(linux_x86_64_assembly.contains("\tmovq %rax, myargv(%rip)\n"));
    assert!(linux_x86_64_assembly.contains("\tcall D_DoomMain\n"));
}

#[test]
fn compiler_accepts_m_argv_post_increment_slice() {
    // given
    let source = r"int myargc;
char **myargv;
int M_CheckParm(char *check) {
    int i;
    for (i = 1; i < myargc; i++) {
        if (!strcasecmp(check, myargv[i]))
            return i;
    }
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let linux_x86_64_assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("linux assembly should emit");

    // then
    assert!(linux_x86_64_assembly.contains("M_CheckParm"));
    assert!(linux_x86_64_assembly.contains("myargc"));
    assert!(linux_x86_64_assembly.contains("myargv"));
    assert!(linux_x86_64_assembly.contains("\taddl %ecx, %eax\n"));
}

#[test]
fn compiler_accepts_typedef_return_signatures() {
    // given
    let source = "typedef int fixed_t; fixed_t FixedMul(fixed_t a, fixed_t b) { return 42; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _FixedMul"));
    assert!(assembly.contains("movz w0, #42"));
}

#[test]
fn compiler_accepts_split_line_typedef_return_signatures() {
    // given
    let source = "typedef int fixed_t; fixed_t\nFixedMul\n(fixed_t a,\n fixed_t b)\n{ return 42; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _FixedMul"));
    assert!(assembly.contains("movz w0, #42"));
}

#[test]
fn compiler_accepts_unsigned_scalar_return_signatures() {
    // given
    let source =
        "unsigned short SwapSHORT(unsigned short x) { return 42; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _SwapSHORT"));
    assert!(assembly.contains("movz w0, #42"));
}

#[test]
fn compiler_accepts_doom_member_access_slice() {
    // given
    let source = r"typedef int fixed_t;
typedef struct { fixed_t x,y; } mpoint_t;
typedef struct { mpoint_t a,b; } mline_t;
typedef struct { fixed_t slp, islp; } islope_t;
void AM_getIslope(mline_t* ml, islope_t* is) {
    int dx, dy;
    dy = ml->a.y - ml->b.y;
    dx = ml->b.x - ml->a.x;
    is->islp = dx + dy;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl AM_getIslope"));
    assert!(assembly.contains("\tmovl 4(%rax), %eax\n"));
    assert!(assembly.contains("\tmovl 12(%rax), %eax\n"));
    assert!(assembly.contains("\tmovl 8(%rax), %eax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
    assert!(assembly.contains("\tmovl %eax, 4(%rcx)\n"));
}

#[test]
fn compiler_accepts_pointer_struct_member_values_slice() {
    // given
    let source = r"typedef struct {
    unsigned char* sequence;
    unsigned char* p;
} cheatseq_t;
int check(cheatseq_t* cht) {
    if (!cht->p) cht->p = cht->sequence;
    return cht->p == cht->sequence ? 0 : 1;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly = emit_assembly(&lowered, Target::native()).expect("assembly should emit");

    // then
    assert!(assembly.contains("check"));
    assert!(assembly.contains("main"));
}

#[test]
fn compiler_accepts_doom_action_function_pointer_slice() {
    // given
    let source = r"typedef void (*actionf_p1)(void*);
typedef union {
    actionf_p1 acp1;
} actionf_t;
typedef actionf_t think_t;
typedef struct thinker_s {
    think_t function;
} thinker_t;
void T_MoveCeiling(void* value) {
}
int main(void) {
    thinker_t thinker;
    thinker.function.acp1 = (actionf_p1)T_MoveCeiling;
    return thinker.function.acp1 ? 0 : 1;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("T_MoveCeiling:"));
    assert!(assembly.contains("\tleaq T_MoveCeiling(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq %rax, 0(%rcx)\n"));
    assert!(assembly.contains("\tcmpq $0, %rax\n"));
}

#[test]
fn compiler_accepts_doom_action_function_pointer_call_slice() {
    // given
    let source = r"typedef void (*actionf_p1)(void*);
typedef union {
    actionf_p1 acp1;
} actionf_t;
typedef struct {
    actionf_t action;
} state_t;
void run(state_t* state, void* mobj) {
    if (state->action.acp1) state->action.acp1(mobj);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("run:"));
    assert!(assembly.contains("\tcall *%rax\n"));
}

#[test]
fn compiler_accepts_doom_function_designator_callback_argument_slice() {
    // given
    let source = r"typedef int boolean;
typedef boolean (*thing_checker_t)(int);
int main(void) {
    return P_BlockThingsIterator(1, 2, PIT_StompThing);
}
boolean PIT_StompThing(int thing) {
    return thing;
}
boolean P_BlockThingsIterator(int x, int y, thing_checker_t checker) {
    return 1;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tleaq PIT_StompThing(%rip), %rax\n"));
    assert!(assembly.contains("\tcall P_BlockThingsIterator\n"));
}

#[test]
fn compiler_accepts_member_access_on_pointer_return_call_slice() {
    // given
    let source = r"typedef struct {
    int sector;
} side_t;
side_t* getSide(int currentSector, int line, int side);
int main(void) {
    return getSide(0, 0, 0)->sector;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tcall getSide\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_doom_named_inner_union_member_slice() {
    // given
    let source = r"typedef struct {
    int health;
} mobj_t;
typedef struct {
    int flags;
} line_t;
typedef struct {
    int frac;
    int isaline;
    union {
        mobj_t* thing;
        line_t* line;
    } d;
} intercept_t;
int main(void) {
    intercept_t* in;
    in = 0;
    return in->d.line->flags;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovq 8("));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_tagged_struct_pointer_referent_slice() {
    // given
    let source = r"typedef struct line_s {
    int flags;
} line_t;
typedef struct {
    struct line_s** lines;
} sector_t;
int main(void) {
    sector_t* sec;
    sec = 0;
    return sec->lines[0]->flags;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovq 0(%rax), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_merges_extern_struct_object_with_definition_slice() {
    // given
    let source = r"typedef struct thinker_s {
    struct thinker_s* next;
} thinker_t;
extern thinker_t thinkercap;
thinker_t thinkercap;
int main(void) {
    return thinkercap.next ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("thinkercap:"));
    assert!(assembly.contains("\tleaq thinkercap(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq 0(%rax), %rax\n"));
}

#[test]
fn compiler_accepts_local_struct_pointer_declaration_slice() {
    // given
    let source = r"typedef struct {
    int width;
} patch_t;
int main(void) {
    patch_t* patch;
    patch = 0;
    return patch ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tcmpq $0, %rax\n"));
}

#[test]
fn compiler_accepts_local_struct_object_member_slice() {
    // given
    let source = r"typedef struct {
    int x, y;
} fpoint_t;
typedef struct {
    fpoint_t a, b;
} fline_t;
void clip(fline_t* fl) {
    fpoint_t tmp;
    tmp.x = fl->a.x + 1;
    tmp.y = 0;
    fl->b = tmp;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("clip:"));
    assert!(assembly.contains("\tleaq -16(%rbp), %rax\n"));
    assert!(assembly.contains("\tmovl %eax, 0(%rcx)\n"));
    assert!(assembly.contains("\tmovl %eax, 4(%rcx)\n"));
    assert!(assembly.contains("\tmovl %eax, 8(%rcx)\n"));
    assert!(assembly.contains("\tmovl %eax, 12(%rcx)\n"));
}

#[test]
fn compiler_accepts_static_local_struct_object_slice() {
    // given
    let source = r"typedef struct {
    int x, y;
} fpoint_t;
typedef struct {
    fpoint_t a, b;
} fline_t;
int draw(void) {
    static fline_t fl;
    fl.a.x = 1;
    fl.b.y = 2;
    return fl.a.x;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("draw:"));
    assert!(assembly.contains("\tmovl %eax, 0(%rcx)\n"));
    assert!(assembly.contains("\tmovl %eax, 12(%rcx)\n"));
}

#[test]
fn compiler_accepts_local_static_aggregate_address_slice() {
    // given
    let source = r"typedef enum { ev_keyup } evtype_t;
typedef struct {
    evtype_t type;
    int data1;
} event_t;
void ST_Responder(event_t* ev);
int main(void) {
    static event_t st_notify = { ev_keyup, 1 };
    ST_Responder(&st_notify);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tleaq -4(%rbp), %rax\n"));
    assert!(assembly.contains("\tcall ST_Responder\n"));
}

#[test]
fn compiler_accepts_pointer_member_post_increment_value_slice() {
    // given
    let source = r"typedef struct {
    unsigned char* sequence;
    unsigned char* p;
} cheatseq_t;
void seed(cheatseq_t* cht, int key) {
    if (*cht->p == 0) *(cht->p++) = key;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("seed:"));
    assert!(assembly.contains("\taddq $1, %rax\n"));
    assert!(assembly.contains("\tmovq %rax, 8(%rcx)\n"));
}

#[test]
fn compiler_accepts_pointer_return_signatures() {
    // given
    let source = "char *name(void) { return 0; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("name:"));
    assert!(assembly.contains("\tmovl $0, %eax\n"));
}
