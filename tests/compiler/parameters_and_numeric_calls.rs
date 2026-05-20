use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

#[test]
fn compiler_accepts_global_char_array_braced_initializer_slice() {
    // given
    let source = r"char frenchKeyMap[4] = { 'P', '\\', 127, 0 };
char ForeignTranslation(unsigned char ch) {
    return ch < 4 ? frenchKeyMap[ch] : ch;
}
int main(void) {
    return ForeignTranslation(1);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("frenchKeyMap:"));
    assert!(assembly.contains("\t.byte 80,92,127,0\n"));
    assert!(assembly.contains("ForeignTranslation"));
}

#[test]
fn compiler_accepts_global_pointer_array_slice() {
    // given
    let source = r"int* ylookup[4];
int main(void) {
    int* p;
    p = 0;
    ylookup[2] = p;
    return ylookup[2] ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("ylookup:"));
    assert!(assembly.contains("\t.zero 32\n"));
    assert!(assembly.contains("\tmovq %rax, (%rcx,%rdx,8)\n"));
    assert!(assembly.contains("\tmovq (%rcx,%rax,8), %rax\n"));
}

#[test]
fn compiler_accepts_typed_global_struct_pointer_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
} point_t;
static point_t *cursor;
int main(void) {
    return cursor->x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("cursor:"));
    assert!(assembly.contains("\tmovq cursor(%rip), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_struct_fields_with_typedef_and_array_slice() {
    // given
    let source = r"typedef int state_t;
typedef struct {
    state_t state;
    int powers[4];
    int x;
} player_t;
static player_t *plr;
int main(void) {
    return plr->x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("plr:"));
    assert!(assembly.contains("\tmovq plr(%rip), %rax\n"));
    assert!(assembly.contains("\tmovl 20(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_nested_typed_pointer_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
} mobj_t;
typedef struct {
    mobj_t* mo;
} player_t;
static player_t *plr;
int main(void) {
    return plr->mo->x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("plr:"));
    assert!(assembly.contains("\tmovq plr(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq 0(%rax), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_pointer_subscript_struct_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
} vertex_t;
typedef struct {
    vertex_t* v1;
} line_t;
static line_t *lines;
int main(void) {
    int i = 0;
    return lines[i].v1->x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("lines:"));
    assert!(assembly.contains("\tmovq lines(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq 0(%rax), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_extern_pointer_subscript_struct_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
} vertex_t;
typedef struct {
    vertex_t* v1;
} line_t;
extern line_t *lines;
int main(void) {
    int i = 0;
    return lines[i].v1->x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("lines:\n"));
    assert!(assembly.contains("\tmovq lines(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq 0(%rax), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_global_struct_array_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
    int y;
} mpoint_t;
static mpoint_t markpoints[4];
int main(void) {
    int i = 0;
    markpoints[i].x = 1;
    return markpoints[i].y;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("markpoints:"));
    assert!(assembly.contains("\t.zero 32\n"));
    assert!(assembly.contains("\tleaq markpoints(%rip), %rax\n"));
    assert!(assembly.contains("\tmovl %eax, 0(%rcx)\n"));
    assert!(assembly.contains("\tmovl 4(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_global_struct_object_member_slice() {
    // given
    let source = r"typedef struct {
    int x;
    int y;
} mpoint_t;
static mpoint_t m_paninc;
int main(void) {
    m_paninc.x = 1;
    return m_paninc.y;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("m_paninc:"));
    assert!(assembly.contains("\t.zero 8\n"));
    assert!(assembly.contains("\tleaq m_paninc(%rip), %rax\n"));
    assert!(assembly.contains("\tmovl %eax, 0(%rcx)\n"));
    assert!(assembly.contains("\tmovl 4(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_global_struct_object_with_struct_array_field_slice() {
    // given
    let source = r"typedef struct {
    int in;
    int frags[4];
} wbplayerstruct_t;
typedef struct {
    int epsd;
    wbplayerstruct_t plyr[4];
} wbstartstruct_t;
wbstartstruct_t wminfo;
int main(void) {
    int i;
    i = 0;
    wminfo.epsd = 1;
    wminfo.plyr[i].in = 2;
    return sizeof(wminfo);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("wminfo:"));
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovl $84, %eax\n"));
}
