use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

#[test]
fn compiler_accepts_doom_function_designator_callback_argument_slice() {
    // given
    let source = r"typedef int boolean;
typedef boolean (*thing_checker_t)(int);
int main(void) {
    return P_BlockThingsIterator(1, 2, PIT_StompThing);
}
boolean PIT_StompThing(int thing) {
    return thing;
}
boolean P_BlockThingsIterator(int x, int y, thing_checker_t checker) {
    return 1;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tleaq PIT_StompThing(%rip), %rax\n"));
    assert!(assembly.contains("\tcall P_BlockThingsIterator\n"));
}

#[test]
fn compiler_accepts_prototype_function_designator_assignment_slice() {
    // given
    let source = r"void R_DrawColumn(void);
void (*colfunc)(void);
void (*basecolfunc)(void);
void setup(void) {
    colfunc = basecolfunc = R_DrawColumn;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tleaq R_DrawColumn(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq %rax, basecolfunc(%rip)\n"));
    assert!(assembly.contains("\tmovq %rax, colfunc(%rip)\n"));
}

#[test]
fn compiler_accepts_member_access_on_pointer_return_call_slice() {
    // given
    let source = r"typedef struct {
    int sector;
} side_t;
side_t* getSide(int currentSector, int line, int side);
int main(void) {
    return getSide(0, 0, 0)->sector;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tcall getSide\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_forward_struct_typedef_then_tag_definition_slice() {
    // given
    let source = r"typedef struct sfxinfo_struct sfxinfo_t;
struct sfxinfo_struct {
    char* name;
    void* data;
};
extern sfxinfo_t S_sfx[];
int from_global(int index) {
    return S_sfx[index].name != 0;
}
int from_pointer(sfxinfo_t* sfx) {
    return sfx->data != 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("from_global:"));
    assert!(assembly.contains("from_pointer:"));
    assert!(assembly.contains("S_sfx"));
}

#[test]
fn compiler_accepts_global_struct_array_arrow_decay_slice() {
    // given
    let source = r"typedef struct {
    int* soundorg;
} button_t;
button_t buttonlist[16];
int main(void) {
    return buttonlist->soundorg != 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("buttonlist:"));
    assert!(assembly.contains("main:"));
}

#[test]
fn compiler_accepts_global_struct_array_subscript_pointer_member_chain_slice() {
    // given
    let source = r"typedef struct {
    int angle;
} mobj_t;
typedef struct {
    mobj_t* mo;
} player_t;
extern player_t players[4];
int consoleplayer;
int main(void) {
    return players[consoleplayer].mo->angle;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(!assembly.contains("players:\n"));
    assert!(assembly.contains("consoleplayer:"));
    assert!(assembly.contains("\tleaq players(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq 0(%rax), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_global_pointer_matrix_assignment_slice() {
    // given
    let source = r"typedef struct {
    int width;
} patch_t;
void* W_CacheLumpName(char* name, int tag);
patch_t* arms[6][2];
int main(void) {
    int i;
    char* namebuf;
    i = 3;
    arms[i][0] = (patch_t*) W_CacheLumpName(namebuf, 1);
    return arms[i][0]->width;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("arms:"));
    assert!(assembly.contains("\t.zero 96\n"));
    assert!(assembly.contains("\tcall W_CacheLumpName\n"));
    assert!(assembly.contains("\tmovq %rax, (%rcx,%rdx,8)\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_global_sizeof_struct_array_initializer_slice() {
    // given
    let source = r"typedef struct {
    int type;
    int period;
} anim_t;
static anim_t epsd0animinfo[] = { { 0, 1 }, { 0, 2 } };
static anim_t epsd1animinfo[] = { { 1, 3 } };
static int NUMANIMS[2] = {
    sizeof(epsd0animinfo)/sizeof(anim_t),
    sizeof(epsd1animinfo)/sizeof(anim_t)
};
int main(void) {
    return NUMANIMS[0] + NUMANIMS[1];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("NUMANIMS:"));
    assert!(assembly.contains("\t.long 2,1\n"));
    assert!(assembly.contains("main:"));
}

#[test]
fn compiler_accepts_global_pointer_name_array_initializer_slice() {
    // given
    let source = r"typedef struct {
    int type;
} anim_t;
static anim_t epsd0animinfo[] = { { 0 } };
static anim_t epsd1animinfo[] = { { 1 } };
static anim_t *anims[3] = {
    epsd0animinfo,
    epsd1animinfo
};
int main(void) {
    return anims[1][0].type;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("anims:"));
    assert!(assembly.contains("\t.quad epsd0animinfo\n"));
    assert!(assembly.contains("\t.quad epsd1animinfo\n"));
    assert!(assembly.contains("\t.quad 0\n"));
}

#[test]
fn compiler_merges_extern_and_defined_global_matrices_slice() {
    // given
    let source = r"typedef struct {
    int forwardmove;
} ticcmd_t;
typedef unsigned char lighttable_t;
extern ticcmd_t netcmds[4][12];
ticcmd_t netcmds[4][12];
extern lighttable_t* scalelight[16][48];
lighttable_t* scalelight[16][48];
int main(void) {
    return netcmds[1][2].forwardmove + (scalelight[1][2] != 0);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("netcmds:"));
    assert!(assembly.contains("scalelight:"));
    assert!(assembly.contains("main:"));
}
