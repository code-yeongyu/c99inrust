use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

#[test]
fn compiler_accepts_global_struct_matrix_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
    int y;
} point_t;
typedef struct {
    int epsd;
} wbstartstruct_t;
static point_t lnodes[4][9] = {
    { { 185, 164 } },
    { { 254, 25 } }
};
static wbstartstruct_t* wbs;
int main(void) {
    int n;
    n = 0;
    return lnodes[wbs->epsd][n].x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("lnodes:"));
    assert!(assembly.contains("wbs:"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_struct_two_dimensional_array_field_assignment_slice() {
    // given
    let source = r"typedef struct {
    int bbox[2][4];
} node_t;
int main(void) {
    node_t* no;
    int j;
    int k;
    no = 0;
    j = 1;
    k = 2;
    no->bbox[j][k] = 7;
    return no->bbox[j][k];
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
}

#[test]
fn compiler_accepts_doom_named_inner_union_member_slice() {
    // given
    let source = r"typedef struct {
    int health;
} mobj_t;
typedef struct {
    int flags;
} line_t;
typedef struct {
    int frac;
    int isaline;
    union {
        mobj_t* thing;
        line_t* line;
    } d;
} intercept_t;
int main(void) {
    intercept_t* in;
    in = 0;
    return in->d.line->flags;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovq 8("));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_parenthesized_pointer_member_assignment_slice() {
    // given
    let source = r"typedef struct mobj_s {
    struct mobj_s* bprev;
} mobj_t;
int main(void) {
    mobj_t* thing;
    mobj_t** link;
    thing = 0;
    link = 0;
    (*link)->bprev = thing;
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovq"));
    assert!(assembly.contains("\tmovq %rax, 0(%rcx)\n"));
}

#[test]
fn compiler_accepts_function_pointer_parameter_call_slice() {
    // given
    let source = r"typedef int boolean;
typedef struct {
    int flags;
} line_t;
boolean run(boolean (*func)(line_t*)) {
    return func(0);
}
boolean check(line_t* line) {
    return 1;
}
int main(void) {
    return run(check);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tcall *%rax\n"));
    assert!(assembly.contains("\tleaq check(%rip), %rax\n"));
}
