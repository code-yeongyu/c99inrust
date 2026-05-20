use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

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
