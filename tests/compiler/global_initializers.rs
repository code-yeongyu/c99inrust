use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

#[test]
fn compiler_accepts_fixed_point_global_initializer_slice() {
    // given
    let source = "typedef int fixed_t; static fixed_t scale_mtof = (.2*(1<<16));";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("scale_mtof:"));
    assert!(assembly.contains("\t.long 13107\n"));
}

#[test]
fn compiler_accepts_aggregate_global_initializer_before_supported_function() {
    // given
    let source = r"typedef struct {
    unsigned char *sequence;
    unsigned char *p;
} cheatseq_t;
static unsigned char cheat_amap_seq[] = { 0xb2, 0x26, 0xff };
static cheatseq_t cheat_amap = { cheat_amap_seq, 0 };
int main(void) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("cheat_amap_seq:"));
    assert!(assembly.contains("cheat_amap:"));
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("movl $42, %eax"));
}

#[test]
fn compiler_accepts_aggregate_global_address_slice() {
    // given
    let source = r"typedef struct {
    unsigned char *sequence;
    int offset;
} cheatseq_t;
static unsigned char cheat_amap_seq[] = { 0xb2, 0x26, 0xff };
static cheatseq_t cheat_amap = { cheat_amap_seq, 0 };
void use(cheatseq_t* value);
int main(void) {
    use(&cheat_amap);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("cheat_amap:"));
    assert!(assembly.contains("\tleaq cheat_amap(%rip), %rax\n"));
    assert!(assembly.contains("\tcall use\n"));
}

#[test]
fn compiler_accepts_struct_array_initializer_before_supported_function() {
    // given
    let source = r"typedef struct {
    int x;
    int y;
} mpoint_t;
typedef struct {
    mpoint_t a;
    mpoint_t b;
} mline_t;
mline_t player_arrow[] = {
    { { -8, 0 }, { 8, 0 } },
    { { 8, 0 }, { 0, 8 } }
};
int main(void) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("player_arrow:"));
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("movl $42, %eax"));
}

#[test]
fn compiler_accepts_global_struct_array_decay_slice() {
    // given
    let source = r"typedef struct {
    int x;
} mline_t;
mline_t player_arrow[] = {
    { 1 },
    { 2 }
};
void draw(mline_t* lines);
int main(void) {
    draw(player_arrow);
    return sizeof(player_arrow);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("player_arrow:"));
    assert!(assembly.contains("\tleaq player_arrow(%rip), %rax\n"));
    assert!(assembly.contains("\tcall draw\n"));
    assert!(assembly.contains("\tmovl $8, %eax\n"));
}

#[test]
fn compiler_emits_global_struct_array_initializers_slice() {
    // given
    let source = r#"typedef struct {
    char* name;
    int* location;
    int defaultvalue;
} default_t;
int mouseSensitivity;
default_t defaults[] = {
    { "mouse_sensitivity", &mouseSensitivity, 5 }
};
int main(void) { return defaults[0].defaultvalue; }"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("defaults:\n"));
    assert!(assembly.contains("\t.quad .Ldefaults_str0\n"));
    assert!(assembly.contains("\t.quad mouseSensitivity\n"));
    assert!(assembly.contains("\t.long 5\n"));
    assert!(!assembly.contains("defaults:\n\t.zero 24\n"));
}

#[test]
fn compiler_sizes_global_struct_array_elements_slice() {
    // given
    let source = r#"typedef struct {
    char* name;
    int* location;
    int defaultvalue;
} default_t;
int mouseSensitivity;
default_t defaults[] = {
    { "mouse_sensitivity", &mouseSensitivity, 5 }
};
int main(void) { return sizeof(defaults) / sizeof(defaults[0]); }"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovl $24, %eax\n"));
    assert!(!assembly.contains("\tmovl $8, %eax\n"));
}

#[test]
fn compiler_accepts_extern_struct_array_before_definition_slice() {
    // given
    let source = r"typedef struct {
    int ammo;
    int upstate;
} weaponinfo_t;
extern weaponinfo_t weaponinfo[NUMWEAPONS];
weaponinfo_t weaponinfo[NUMWEAPONS] = {
    { 1, 2 },
    { 3, 4 },
    { 5, 6 }
};
int main(void) {
    return sizeof(weaponinfo);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert_eq!(assembly.matches("weaponinfo:\n").count(), 1);
    assert!(assembly.contains("\t.long 1\n"));
    assert!(assembly.contains("\t.long 6\n"));
    assert!(assembly.contains("\tmovl $24, %eax\n"));
}

#[test]
fn compiler_accepts_multi_declarator_local_int_slice() {
    // given
    let source = "int main(void) { int dx, dy; dx = 40; dy = 2; return dx + dy; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("movl %eax, -4(%rbp)"));
    assert!(assembly.contains("movl %eax, -8(%rbp)"));
    assert!(assembly.contains("addl %ecx, %eax"));
}

#[test]
fn compiler_accepts_global_int_declarator_list_slice() {
    // given
    let source = r"typedef int fixed_t;
static fixed_t m_x, m_y;
int main(void) {
    m_x = 4;
    m_y = m_x + 2;
    return m_y;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("m_x:"));
    assert!(assembly.contains("m_y:"));
    assert!(assembly.contains("\tmovl %eax, m_x(%rip)\n"));
    assert!(assembly.contains("\tmovl %eax, m_y(%rip)\n"));
}
