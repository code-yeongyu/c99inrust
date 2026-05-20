use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

#[test]
fn compiler_accepts_short_pointer_arithmetic_dereference_slice() {
    // given
    let source = r"short* blockmap;
int main(void) {
    int offset;
    offset = 0;
    blockmap[offset] = 5;
    return *(blockmap + offset);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("blockmap:"));
    assert!(assembly.contains("\tmovw %ax, (%rcx,%rdx,2)\n"));
    assert!(assembly.contains("\tmovswl (%rcx,%rax,2), %eax\n"));
}

#[test]
fn compiler_accepts_struct_pointer_arithmetic_member_slice() {
    // given
    let source = r"typedef struct vissprite_s {
    struct vissprite_s* next;
} vissprite_t;
vissprite_t items[4];
vissprite_t* ptr;
void link(void) {
    ptr = items;
    (ptr - 1)->next = &items[0];
    ptr = ptr + 1;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tleaq (%rcx,%rax,8), %rax\n"));
    assert!(assembly.contains("\tmovq %rax, 0(%rcx)\n"));
    assert!(assembly.contains("\tmovq %rax, ptr(%rip)\n"));
}

#[test]
fn compiler_accepts_tagged_struct_pointer_referent_slice() {
    // given
    let source = r"typedef struct line_s {
    int flags;
} line_t;
typedef struct {
    struct line_s** lines;
} sector_t;
int main(void) {
    sector_t* sec;
    sec = 0;
    return sec->lines[0]->flags;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovq 0(%rax), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_merges_extern_struct_object_with_definition_slice() {
    // given
    let source = r"typedef struct thinker_s {
    struct thinker_s* next;
} thinker_t;
extern thinker_t thinkercap;
thinker_t thinkercap;
int main(void) {
    return thinkercap.next ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("thinkercap:"));
    assert!(assembly.contains("\tleaq thinkercap(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq 0(%rax), %rax\n"));
}

#[test]
fn compiler_accepts_local_struct_pointer_declaration_slice() {
    // given
    let source = r"typedef struct {
    int width;
} patch_t;
int main(void) {
    patch_t* patch;
    patch = 0;
    return patch ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tcmpq $0, %rax\n"));
}

#[test]
fn compiler_accepts_local_struct_object_member_slice() {
    // given
    let source = r"typedef struct {
    int x, y;
} fpoint_t;
typedef struct {
    fpoint_t a, b;
} fline_t;
void clip(fline_t* fl) {
    fpoint_t tmp;
    tmp.x = fl->a.x + 1;
    tmp.y = 0;
    fl->b = tmp;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("clip:"));
    assert!(assembly.contains("\tleaq -16(%rbp), %rax\n"));
    assert!(assembly.contains("\tmovl %eax, 0(%rcx)\n"));
    assert!(assembly.contains("\tmovl %eax, 4(%rcx)\n"));
    assert!(assembly.contains("\tmovl %eax, 8(%rcx)\n"));
    assert!(assembly.contains("\tmovl %eax, 12(%rcx)\n"));
}

#[test]
fn compiler_accepts_static_local_struct_object_slice() {
    // given
    let source = r"typedef struct {
    int x, y;
} fpoint_t;
typedef struct {
    fpoint_t a, b;
} fline_t;
int draw(void) {
    static fline_t fl;
    fl.a.x = 1;
    fl.b.y = 2;
    return fl.a.x;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("draw:"));
    assert!(assembly.contains("\tmovl %eax, 0(%rcx)\n"));
    assert!(assembly.contains("\tmovl %eax, 12(%rcx)\n"));
}

#[test]
fn compiler_accepts_local_static_aggregate_address_slice() {
    // given
    let source = r"typedef enum { ev_keyup } evtype_t;
typedef struct {
    evtype_t type;
    int data1;
} event_t;
void ST_Responder(event_t* ev);
int main(void) {
    static event_t st_notify = { ev_keyup, 1 };
    ST_Responder(&st_notify);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tleaq -4(%rbp), %rax\n"));
    assert!(assembly.contains("\tcall ST_Responder\n"));
}

#[test]
fn compiler_accepts_pointer_member_post_increment_value_slice() {
    // given
    let source = r"typedef struct {
    unsigned char* sequence;
    unsigned char* p;
} cheatseq_t;
void seed(cheatseq_t* cht, int key) {
    if (*cht->p == 0) *(cht->p++) = key;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("seed:"));
    assert!(assembly.contains("\taddq $1, %rax\n"));
    assert!(assembly.contains("\tmovq %rax, 8(%rcx)\n"));
}

#[test]
fn compiler_accepts_pointer_return_signatures() {
    // given
    let source = "char *name(void) { return 0; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("name:"));
    assert!(assembly.contains("\tmovl $0, %eax\n"));
}
