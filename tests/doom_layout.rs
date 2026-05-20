use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

fn compile_x86_64(source: &str) -> String {
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit")
}

#[test]
fn compiler_sizes_doom_wad_filelump_char_array_field_slice() {
    // given
    let source = r#"typedef struct {
    int filepos;
    int size;
    char name[8];
} filelump_t;
filelump_t directory[2] = {
    { 1, 2, "A" },
    { 3, 4, "B" }
};
int main(void) { return sizeof(filelump_t); }"#;

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovl $16, %eax\n"));
    assert!(assembly.contains("\t.byte 65,0,0,0,0,0,0,0\n"));
    assert!(assembly.contains("\t.byte 66,0,0,0,0,0,0,0\n"));
    assert!(!assembly.contains("\tmovl $40, %eax\n"));
    assert!(!assembly.contains("\t.zero 80\n"));
}

#[test]
fn compiler_increments_doom_wad_filelump_pointer_by_struct_size_slice() {
    // given
    let source = r"typedef struct {
    int filepos;
    int size;
    char name[8];
} filelump_t;
filelump_t directory[2];
int main(void) {
    filelump_t *fileinfo;
    fileinfo = directory;
    fileinfo++;
    return (int) fileinfo++;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovabsq $16, %rax\n"));
    assert!(assembly.contains("\taddq %rcx, %rax\n"));
    assert!(assembly.contains("\taddq $16, %rax\n"));
    assert!(!assembly.contains("\taddq $1, %rax\n"));
    assert!(!assembly.contains("\tmovl $1, %eax\n"));
}

#[test]
fn compiler_sizes_doom_texture_short_fields_slice() {
    // given
    let source = r"typedef int boolean;
typedef struct {
    short originx;
    short originy;
    short patch;
    short stepdir;
    short colormap;
} mappatch_t;
typedef struct {
    char name[8];
    boolean masked;
    short width;
    short height;
    void **columndirectory;
    short patchcount;
    mappatch_t patches[1];
} maptexture_t;
int main(void) { return sizeof(mappatch_t) + sizeof(maptexture_t); }";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovl $10, %eax\n"));
    assert!(assembly.contains("\tmovl $32, %eax\n"));
    assert!(!assembly.contains("\tmovl $20, %eax\n"));
    assert!(!assembly.contains("\tmovl $40, %eax\n"));
    assert!(!assembly.contains("\tmovl $64, %eax\n"));
}

#[test]
fn compiler_loads_doom_texture_short_field_as_halfword_slice() {
    // given
    let source = r"typedef struct {
    short originx;
    short originy;
    short patch;
} mappatch_t;
int main(void) {
    mappatch_t *mpatch;
    return mpatch->patch;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovswl 4(%rax), %eax\n"));
    assert!(!assembly.contains("\tmovl 8(%rax), %eax\n"));
}

#[test]
fn compiler_loads_doom_maptexture_patchcount_from_disk_offset_slice() {
    // given
    let source = r"typedef int boolean;
typedef struct {
    short originx;
    short originy;
    short patch;
    short stepdir;
    short colormap;
} mappatch_t;
typedef struct {
    char name[8];
    boolean masked;
    short width;
    short height;
    void **columndirectory;
    short patchcount;
    mappatch_t patches[1];
} maptexture_t;
int main(void) {
    maptexture_t *mtexture;
    return mtexture->patchcount;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovswl 20(%rax), %eax\n"));
    assert!(!assembly.contains("\tmovswl 24(%rax), %eax\n"));
}

#[test]
fn compiler_stores_doom_name8_union_char_member_as_byte_slice() {
    // given
    let source = r"int main(void) {
    union {
        char s[9];
        int x[2];
    } name8;
    name8.s[8] = 0;
    return name8.x[1];
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovb %al, (%rcx,%rdx,1)\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
    assert!(!assembly.contains("\tmovl %eax, (%rcx,%rdx,4)\n"));
}

#[test]
fn compiler_divides_struct_pointer_difference_by_element_size_slice() {
    // given
    let source = r"typedef struct {
    int handle;
    int position;
    int size;
    char name[8];
} lumpinfo_t;
lumpinfo_t lumpinfo[4];
int main(void) {
    lumpinfo_t *lump_p;
    lump_p = &lumpinfo[2];
    return lump_p - lumpinfo;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovabsq $20, %rax\n"));
    assert!(assembly.contains("\tidivq %rcx\n"));
    assert!(assembly.contains("\tsubq %rcx, %rax\n"));
}

#[test]
fn compiler_divides_struct_tag_alias_pointer_difference_by_element_size_slice() {
    // given
    let source = r"typedef struct player_s {
    int health;
    int armor;
    int ammo;
} player_t;
typedef struct {
    struct player_s *player;
} mobj_t;
player_t players[4];
mobj_t target;
int main(void) {
    return target.player - players;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovabsq $12, %rax\n"));
    assert!(assembly.contains("\tidivq %rcx\n"));
    assert!(assembly.contains("\tsubq %rcx, %rax\n"));
}

#[test]
fn compiler_divides_tag_alias_pointer_pointer_difference_by_pointer_size_slice() {
    // given
    let source = r"typedef struct line_s {
    int flags;
} line_t;
typedef struct {
    struct line_s **lines;
} sector_t;
int main(void) {
    line_t **linebuffer;
    sector_t sector;
    return linebuffer - sector.lines;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovabsq $8, %rax\n"));
    assert!(assembly.contains("\tidivq %rcx\n"));
    assert!(assembly.contains("\tsubq %rcx, %rax\n"));
}

#[test]
fn compiler_scales_address_of_pointer_subscript_by_pointer_size_slice() {
    // given
    let source = r"void **lumpcache;
int main(void) {
    int lump;
    return (int)&lumpcache[lump];
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tleaq (%rcx,%rax,8), %rax\n"));
    assert!(!assembly.contains("\taddq %rcx, %rax\n"));
}

#[test]
fn compiler_scales_address_of_struct_char_array_field_by_byte_slice() {
    // given
    let source = r"typedef struct {
    char name[8];
    int size;
} lumpinfo_t;
lumpinfo_t *lump_p;
int main(void) {
    return *(int *)&lump_p->name[4];
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tleaq (%rcx,%rax,1), %rax\n"));
    assert!(!assembly.contains("\tleaq (%rcx,%rax,4), %rax\n"));
}

#[test]
fn compiler_widens_doom_z_malloc_pointer_array_scale_slice() {
    // given
    let source = r"void *Z_Malloc(int size, int tag, void *user);
void **textures;
int numtextures;
int main(void) {
    textures = Z_Malloc(numtextures*4, 1, 0);
    return 0;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\timull %ecx, %eax\n"));
    assert!(assembly.contains("\tmovl $8, %eax\n"));
    assert!(!assembly.contains("\tmovl $4, %eax\n"));
}

#[test]
fn compiler_keeps_alloca_result_as_pointer_width_slice() {
    // given
    let source = r"void *alloca(int size);
int main(void) {
    char *patchcount;
    patchcount = (char *)alloca(24);
    return patchcount != 0;
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovq %rsp, %rax\n"));
    assert!(!assembly.contains("\tmovq %rsp, %rax\n\tcltq\n"));
}
