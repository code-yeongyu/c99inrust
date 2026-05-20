use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::{parse, parse_supported_translation_unit};

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
    assert!(assembly.contains("main__static__nexttic:\n\t.long 0\n"));
    assert!(assembly.contains("main__static__lastlevel:\n\t.long -1\n"));
    assert!(assembly.contains("main__static__lastepisode:\n\t.long -1\n"));
    assert!(assembly.contains("\tmovl main__static__lastlevel(%rip), %eax\n"));
    assert!(assembly.contains("\tmovl %eax, main__static__lastlevel(%rip)\n"));
    assert!(!assembly.contains("\tmovl %eax, -8(%rbp)\n"));
}

#[test]
fn compiler_persists_doom_gettime_static_basetime_slice() {
    // given
    let source = r"int get(void);
int I_GetTime(void) {
    int now;
    int out;
    static int basetime = 0;
    now = get();
    if (!basetime) basetime = now;
    out = now - basetime;
    return out;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let function = lowered
        .functions
        .iter()
        .find(|function| function.name == "I_GetTime")
        .expect("I_GetTime should lower");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("I_GetTime__static__basetime:\n\t.long 0\n"));
    assert!(assembly.contains("\tmovl I_GetTime__static__basetime(%rip), %eax\n"));
    assert!(assembly.contains("\tmovl %eax, I_GetTime__static__basetime(%rip)\n"));
    assert_eq!(function.local_slots.len(), 2);
}
