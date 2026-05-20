use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

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
fn compiler_accepts_errno_global_slice() {
    // given
    let source = r"int main(void) {
    return errno;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tcall __errno_location\n"));
    assert!(assembly.contains("\tmovl (%rax), %eax\n"));
}

#[test]
fn compiler_accepts_variadic_function_definition_slice() {
    // given
    let source = r#"void va_start(int ap, char* last);
void va_end(int ap);
void I_Error(char *error, ...) {
    va_list argptr;
    va_start(argptr, error);
    va_end(argptr);
    error = error;
}
int main(void) {
    I_Error("doom", 1);
    return 0;
}"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("I_Error:"));
    assert!(assembly.contains("\tcall I_Error\n"));
}

#[test]
fn compiler_accepts_local_void_pointer_declaration_slice() {
    // given
    let source = r"int main(void) {
    void* ptr;
    ptr = 0;
    return ptr ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
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
    assert!(assembly.contains("\tleaq ceilingclip(%rip), %rax\n"));
    assert!(assembly.contains("\tmovswl (%rcx,%rax,2), %eax\n"));
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
    assert!(assembly.contains("\t.zero 12\n"));
    assert!(assembly.contains("\tmovw %ax, (%rcx,%rdx,2)\n"));
}
