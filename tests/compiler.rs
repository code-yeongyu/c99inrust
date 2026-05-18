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
