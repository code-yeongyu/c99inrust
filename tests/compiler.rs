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
    let program = parse(&tokens).expect("parser should succeed");
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
fn compiler_emits_zero_arg_function_calls() {
    // given
    let source = "int answer(void) { return 40; } int main(void) { return 2 + answer(); }";

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
