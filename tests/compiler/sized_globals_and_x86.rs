use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

#[test]
fn compiler_emits_x86_64_alloca_without_external_call_slice() {
    // given
    let source = "void* alloca(int size); int main(void) { char* p; p = (char*) alloca(12); return p != 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tandq $-16, %rax\n"));
    assert!(assembly.contains("\tsubq %rax, %rsp\n"));
    assert!(!assembly.contains("\tcall alloca\n"));
}

#[test]
fn compiler_preserves_x86_64_builtin_pointer_call_returns_slice() {
    // given
    let source =
        "char* probe(int size) { return (char*) malloc(size); } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    let call_index = assembly
        .find("\tcall malloc\n")
        .expect("assembly should call malloc");
    let call_tail = &assembly[call_index..];
    let return_index = call_tail.find("\tret\n").expect("probe should return");
    assert!(!call_tail[..return_index].contains("\tcltq\n"));
}

#[test]
fn compiler_emits_x86_64_va_start_end_without_external_calls_slice() {
    // given
    let source = "void probe(char* fmt, ...) { va_list ap; va_start(ap, fmt); va_end(ap); } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("probe:"));
    assert!(assembly.contains("\tmovl $8, 0(%r10)\n"));
    assert!(assembly.contains("\tmovl $48, 4(%r10)\n"));
    assert!(assembly.contains("\tmovq %rax, 16(%r10)\n"));
    assert!(!assembly.contains("\tcall va_start\n"));
    assert!(!assembly.contains("\tcall va_end\n"));
}

#[test]
fn compiler_accepts_initialized_global_int_array_slice() {
    // given
    let source = r"int fuzzoffset[4] = { 320, -320, (320), -(320) };
int main(void) {
    return fuzzoffset[0] + fuzzoffset[1] + fuzzoffset[2] + fuzzoffset[3];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("fuzzoffset:"));
    assert!(assembly.contains("\t.long 320,-320,320,-320\n"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
}

#[test]
fn compiler_accepts_lighttable_pointer_global_slice() {
    // given
    let source = r"typedef unsigned char byte;
typedef byte lighttable_t;
lighttable_t* dc_colormap;
int main(void) {
    dc_colormap = 0;
    return dc_colormap ? 1 : 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("dc_colormap:"));
    assert!(assembly.contains("\t.quad 0\n"));
    assert!(assembly.contains("\tmovq %rax, dc_colormap(%rip)\n"));
    assert!(assembly.contains("\tmovq dc_colormap(%rip), %rax\n"));
}

#[test]
fn compiler_accepts_angle_t_parameter_slice() {
    // given
    let source = r"typedef unsigned int angle_t;
int rotate(angle_t angle) {
    return angle >> 19;
}
int main(void) { return rotate(0); }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("rotate:"));
    assert!(assembly.contains("\tsarl %cl, %eax\n"));
}

#[test]
fn compiler_accepts_parenthesized_product_before_shift_slice() {
    // given
    let source = r"int main(void) {
    int volume;
    int seperation;
    return volume - ((volume * seperation * seperation) >> 16);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("\tsarl %cl, %eax\n"));
    assert!(assembly.contains("ret"));
}

#[test]
fn compiler_accepts_empty_parameter_function_definition_slice() {
    // given
    let source = "void I_InitSound() { return; } int main(void) { I_InitSound(); return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("I_InitSound:"));
    assert!(assembly.contains("\tcall I_InitSound\n"));
}

#[test]
fn compiler_accepts_m_cheat_xlate_table_slice() {
    // given
    let source = r"static unsigned char cheat_xlate_table[256];
int main(void) {
    int i;
    for (i = 0; i < 4; i++) cheat_xlate_table[i] = i + 1;
    return cheat_xlate_table[(unsigned char)2];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly = emit_assembly(&lowered, Target::native()).expect("assembly should emit");

    // then
    assert!(assembly.contains("cheat_xlate_table"));
    assert!(assembly.contains(".byte 0,0,0,0"));
    assert!(assembly.contains("main"));
}

#[test]
fn compiler_accepts_m_bbox_pointer_subscript_slice() {
    // given
    let source = r"enum { BOXTOP, BOXBOTTOM, BOXLEFT, BOXRIGHT };
void M_ClearBox(int *box) {
    box[BOXTOP] = box[BOXRIGHT] = -1;
    box[BOXBOTTOM] = box[BOXLEFT] = 10;
}
void M_AddToBox(int *box, int x, int y) {
    if (x < box[BOXLEFT])
        box[BOXLEFT] = x;
    else if (x > box[BOXRIGHT])
        box[BOXRIGHT] = x;
    if (y < box[BOXBOTTOM])
        box[BOXBOTTOM] = y;
    else if (y > box[BOXTOP])
        box[BOXTOP] = y;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly = emit_assembly(&lowered, Target::native()).expect("assembly should emit");
    let linux_x86_64_assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("linux assembly should emit");

    // then
    assert!(assembly.contains("M_ClearBox"));
    assert!(assembly.contains("M_AddToBox"));
    match Target::native() {
        Target::Aarch64AppleDarwin => assert!(assembly.contains("sxtw #2")),
        Target::X86_64AppleDarwin | Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains(",%rax,4)"));
        }
    }
    assert!(linux_x86_64_assembly.contains("(%rcx,%rax,4)"));
    assert!(linux_x86_64_assembly.contains("(%rcx,%rdx,4)"));
}
