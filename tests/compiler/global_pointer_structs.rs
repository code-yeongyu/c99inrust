use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

#[test]
fn compiler_accepts_extern_struct_array_address_slice() {
    // given
    let source = r"typedef struct {
    int x;
} player_t;
extern player_t players[4];
int main(void) {
    player_t* p;
    int i = 0;
    p = &players[i];
    return p->x;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("players:\n"));
    assert!(assembly.contains("\tleaq players(%rip), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_nested_extern_struct_array_address_slice() {
    // given
    let source = r"typedef struct {
    int forwardmove;
} ticcmd_t;
extern ticcmd_t netcmds[MAXPLAYERS][BACKUPTICS];
void build(ticcmd_t* cmd);
int main(void) {
    int player = 0;
    int tic = 1;
    build(&netcmds[player][tic]);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("netcmds:\n"));
    assert!(assembly.contains("\tleaq netcmds(%rip), %rax\n"));
    assert!(assembly.contains("\tcall build\n"));
}

#[test]
fn compiler_accepts_extern_int_array_slice() {
    // given
    let source = r"extern int playeringame[MAXPLAYERS];
int main(void) {
    int i = 0;
    return playeringame[i];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("playeringame:\n"));
    assert!(assembly.contains("\tleaq playeringame(%rip), %rcx\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_block_extern_int_array_slice() {
    // given
    let source = r"int main(void) {
    extern int forwardmove[2];
    int scale = 2;
    forwardmove[0] = forwardmove[0] * scale;
    return forwardmove[0];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("forwardmove:\n"));
    assert!(assembly.contains("\tleaq forwardmove(%rip), %rcx\n"));
    assert!(assembly.contains("\tmovl %eax, (%rcx,%rdx,4)\n"));
}

#[test]
fn compiler_accepts_pointer_to_pointer_subscript_address_slice() {
    // given
    let source = r"extern char** myargv;
void use(char* value);
int main(void) {
    int i = 0;
    use(&myargv[i][1]);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovq myargv(%rip), %rax\n"));
    assert!(assembly.contains("\tcall use\n"));
}

#[test]
fn compiler_emits_byte_access_for_char_pointer_dereference_slice() {
    // given
    let source = r#"int main(void) {
    char* infile;
    int k;
    infile = "az";
    k = 1;
    *(infile+k) = 0;
    return *(infile+k);
}"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovb %al, (%rcx,%rdx,1)\n"));
    assert!(assembly.contains("\tmovzbl (%rcx,%rax,1), %eax\n"));
}

#[test]
fn compiler_emits_byte_access_for_char_pointer_nested_subscript_slice() {
    // given
    let source = r"extern char** myargv;
int main(void) {
    return myargv[1][0];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovq myargv(%rip), %rax\n"));
    assert!(assembly.contains("\tmovzbl (%rcx,%rax,1), %eax\n"));
}

#[test]
fn compiler_accepts_opaque_file_pointer_local_slice() {
    // given
    let source = r"int main(void) {
    FILE* handle;
    handle = 0;
    return handle == 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tmovq %rax, -8(%rbp)\n"));
}

#[test]
fn compiler_accepts_sizeof_struct_typedef_slice() {
    // given
    let source = r"typedef struct {
    int id;
    int tag;
} memblock_t;
int main(void) {
    return sizeof(memblock_t);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovl $8, %eax\n"));
}

#[test]
fn compiler_accepts_typed_pointer_cast_member_slice() {
    // given
    let source = r"typedef struct {
    int id;
} memblock_t;
int main(void) {
    int* raw;
    raw = 0;
    return ((memblock_t*)raw)->id;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}
