use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::{parse, parse_supported_translation_unit};

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
fn compiler_loads_x86_64_stack_parameters_into_local_slots() {
    // given
    let source = "int seventh(int a, int b, int c, int d, int e, int f, int g) { return g; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("seventh:\n\tpushq %rbp"));
    assert!(assembly.contains("\tmovl 16(%rbp), %r10d\n"));
    assert!(assembly.contains("\tmovl %r10d, -28(%rbp)\n"));
    assert!(assembly.contains("\tmovl -28(%rbp), %eax\n"));
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
