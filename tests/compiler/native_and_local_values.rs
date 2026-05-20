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
fn compiler_accepts_local_char_array_braced_initializer_slice() {
    // given
    let source = "int main(void) { static char destination_keys[4] = { 'g', 'i', 'b', 'r' }; return destination_keys[2]; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovb $103, -4(%rbp)\n"));
    assert!(assembly.contains("\tmovb $114, -1(%rbp)\n"));
    assert!(assembly.contains("ret"));
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
