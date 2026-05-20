use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::{parse, parse_supported_translation_unit};

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
