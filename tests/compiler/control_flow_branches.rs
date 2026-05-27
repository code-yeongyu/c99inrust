use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse;

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
    assert!(assembly.contains("\taddq $4, %rax\n"));
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
fn compiler_branches_directly_for_x86_64_relational_conditions() {
    // given
    let source = "int main(void) { int x = 3; if (x < 5) { return 7; } return 2; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(
        assembly.contains("\tcmpl %ecx, %eax\n\tjge .Lmain_"),
        "relational if should branch directly without materializing a boolean:\n{assembly}"
    );
    assert!(
        !assembly.contains("\tsetl %al\n\tmovzbl %al, %eax\n\tcmpl $0, %eax\n"),
        "relational if still materializes setcc before branching:\n{assembly}"
    );
}

#[test]
fn compiler_branches_directly_for_x86_64_logical_not_conditions() {
    // given
    let source = "int main(void) { int x = 0; if (!x) { return 7; } return 2; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(
        assembly.contains("\tjne .Lmain_"),
        "logical-not if should invert the zero branch directly:\n{assembly}"
    );
    assert!(
        !assembly.contains("\tsete %al\n\tmovzbl %al, %eax\n\tcmpl $0, %eax\n"),
        "logical-not if still materializes setcc before branching:\n{assembly}"
    );
}

#[test]
fn compiler_still_materializes_x86_64_scalar_boolean_values() {
    // given
    let source = "int main(void) { int x = 3; int y = x < 5; return y == 1 ? 0 : 1; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(
        assembly.contains("\tsetl %al\n\tmovzbl %al, %eax\n"),
        "scalar boolean value should still be materialized for assignment:\n{assembly}"
    );
}

#[test]
fn compiler_branches_directly_for_x86_64_double_relational_conditions() {
    // given
    let source = "int main(void) { double x = 1.5; if (x < 2.0) { return 7; } return 2; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(
        assembly.contains("\tucomisd %xmm1, %xmm0\n\tjae .Lmain_"),
        "double relational if should use floating false-branch condition:\n{assembly}"
    );
    assert!(
        !assembly.contains("\tsetb %al\n\tmovzbl %al, %eax\n\tcmpl $0, %eax\n"),
        "double relational if still materializes setcc before branching:\n{assembly}"
    );
}
