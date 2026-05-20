use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

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
    assert!(assembly.contains("display__static__oldgamestate:\n\t.long -1\n"));
    assert!(assembly.contains("\tmovl display__static__oldgamestate(%rip), %eax\n"));
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
