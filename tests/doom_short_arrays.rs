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
fn compiler_scales_doom_extern_short_clip_array_pointer_arithmetic_slice() {
    // given
    let source = r"extern short ceilingclip[320];
short *lastopening;
int start;
void *memcpy(void *dest, void *src, int count);
int main(void) {
    memcpy(lastopening, ceilingclip + start, 2);
    return ceilingclip[start];
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tleaq (%rcx,%rax,2), %rax\n"));
    assert!(assembly.contains("\tmovswl (%rcx,%rax,2), %eax\n"));
    assert!(!assembly.contains("\taddq %rcx, %rax\n"));
    assert!(!assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_stores_doom_global_short_clip_arrays_as_halfwords_slice() {
    // given
    let source = r"short floorclip[320];
int i;
int main(void) {
    floorclip[i] = 200;
    return floorclip[i];
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("floorclip:\n\t.zero 640\n"));
    assert!(assembly.contains("\tmovw %ax, (%rcx,%rdx,2)\n"));
    assert!(assembly.contains("\tmovswl (%rcx,%rax,2), %eax\n"));
    assert!(!assembly.contains("floorclip:\n\t.zero 1280\n"));
}

#[test]
fn compiler_uses_halfword_stride_for_doom_local_short_clip_arrays_slice() {
    // given
    let source = r"short *mfloorclip;
int x;
int main(void) {
    short clipbot[320];
    x = 100;
    clipbot[x] = -2;
    mfloorclip = clipbot;
    return mfloorclip[x];
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovw %ax, (%rcx,%rdx,2)\n"));
    assert!(assembly.contains("\tmovswl (%rcx,%rax,2), %eax\n"));
    assert!(!assembly.contains("\tmovl %eax, (%rcx,%rdx,4)\n"));
}

#[test]
fn compiler_uses_halfword_stride_for_doom_global_short_matrix_slice() {
    // given
    let source = r"short consistancy[4][12];
int player;
int tic;
int main(void) {
    consistancy[player][tic] = 7;
    return consistancy[player][tic];
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("consistancy:\n\t.zero 96\n"));
    assert!(assembly.contains("\timulq $24, %rax\n"));
    assert!(assembly.contains("\tmovw %ax, (%rcx,%rdx,2)\n"));
    assert!(assembly.contains("\tmovswl (%rcx,%rax,2), %eax\n"));
}

#[test]
fn compiler_respects_doom_wipe_local_y_shadowing_static_pointer_slice() {
    // given
    let source = r"static int *y;
int main(void) {
    short buffer[16];
    short *dest;
    int x;
    int height;
    int y;
    dest = buffer;
    x = 1;
    height = 4;
    y = 2;
    dest[x * height + y] = 7;
    return dest[6];
}";

    // when
    let assembly = compile_x86_64(source);

    // then
    assert!(assembly.contains("\tmovw %ax, (%rcx,%rdx,2)\n"));
    assert!(assembly.contains("\tmovswl (%rcx,%rax,2), %eax\n"));
    assert!(!assembly.contains("\tleaq (%rcx,%rax,4), %rax\n"));
}
