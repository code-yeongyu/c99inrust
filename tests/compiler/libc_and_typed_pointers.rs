use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

#[test]
fn compiler_accepts_global_enum_sized_int_array_slice() {
    // given
    let source = r"enum { NUMAMMO = 4 };
int maxammo[NUMAMMO] = { 200, 50, NUMAMMO, NUMAMMO + 1 };
int main(void) {
    return maxammo[2];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("maxammo:"));
    assert!(assembly.contains("\t.long 200,50,4,5\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_unsized_global_enum_int_array_slice() {
    // given
    let source = r"typedef enum {
    DI_EAST,
    DI_WEST
} dirtype_t;
dirtype_t opposite[] = { DI_WEST, DI_EAST };
int main(void) {
    return opposite[0];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("opposite:"));
    assert!(assembly.contains("\t.long 1,0\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_global_enum_sized_pointer_array_slice() {
    // given
    let source = r"enum { NUMCARDS = 3 };
char *keys[NUMCARDS];
int main(void) {
    return keys[2] ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("keys:"));
    assert!(assembly.contains("\t.zero 24\n"));
    assert!(assembly.contains("\tmovq (%rcx,%rax,8), %rax\n"));
}

#[test]
fn compiler_accepts_global_pointer_string_initializer_slice() {
    // given
    let source = r#"char* e1text = "E1";
char* finaletext;
int main(void) {
    finaletext = e1text;
    return finaletext ? 0 : 1;
}"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("e1text:\n\t.quad .Le1text_str0\n"));
    assert!(assembly.contains(".Le1text_str0:\n\t.byte 69,49,0\n"));
    assert!(assembly.contains("\tmovq e1text(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq %rax, finaletext(%rip)\n"));
}

#[test]
fn compiler_accepts_global_pointer_string_array_initializer_slice() {
    // given
    let source = r#"enum { NUM_QUITMESSAGES = 2 };
char* endmsg[NUM_QUITMESSAGES+1] = { "A", "B" "C", "D", };"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("endmsg:\n\t.quad .Lendmsg_str0\n"));
    assert!(assembly.contains("\t.quad .Lendmsg_str1\n"));
    assert!(assembly.contains("\t.quad .Lendmsg_str2\n"));
    assert!(assembly.contains(".Lendmsg_str1:\n\t.byte 66,67,0\n"));
}

#[test]
fn compiler_accepts_extern_typed_pointer_array_member_slice() {
    // given
    let source = r"typedef struct {
    int width;
} patch_t;
extern patch_t *hu_font[2];
int main(void) {
    return hu_font[1]->width;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("hu_font:\n"));
    assert!(assembly.contains("\tleaq hu_font(%rip), %rcx\n"));
    assert!(assembly.contains("\tmovq (%rcx,%rax,8), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_struct_typedef_alias_pointer_member_slice() {
    // given
    let source = r"typedef struct {
    int length;
} post_t;
typedef post_t column_t;
int main(void) {
    column_t* column;
    column = 0;
    return column->length;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_global_int_array_slice() {
    // given
    let source = r"int columnofs[4];
int main(void) {
    columnofs[2] = 7;
    return columnofs[2];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("columnofs:"));
    assert!(assembly.contains("\t.zero 16\n"));
    assert!(assembly.contains("\tmovl %eax, (%rcx,%rdx,4)\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_emits_static_globals_with_internal_linkage_slice() {
    // given
    let source = "static int plr; int main(void) { return plr; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("plr:\n"));
    assert!(!assembly.contains(".globl plr\n"));
    assert!(assembly.contains("main:"));
}

#[test]
fn compiler_emits_x86_64_linux_errno_via_libc_location_slice() {
    // given
    let source = "extern int errno; int main(void) { return errno; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tcall __errno_location\n"));
    assert!(assembly.contains("\tmovl (%rax), %eax\n"));
    assert!(!assembly.contains("errno(%rip)"));
}
