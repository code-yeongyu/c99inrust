use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

#[test]
fn compiler_accepts_double_pointer_parameter_member_slice() {
    // given
    let source = r"typedef struct {
    int height;
} patch_t;
int first_height(patch_t** font) {
    return font[0]->height;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("first_height:"));
    assert!(assembly.contains("\tmovq (%rcx,%rax,8), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_pointer_element_array_parameter_member_slice() {
    // given
    let source = r"typedef struct {
    int leftoffset;
} patch_t;
int first_left(patch_t* patches[]) {
    return patches[0]->leftoffset;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("first_left:"));
    assert!(assembly.contains("\tmovq (%rcx,%rax,8), %rax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
}

#[test]
fn compiler_accepts_global_function_pointer_assignment_call_slice() {
    // given
    let source = r"void (*messageRoutine)(int response);
void set(void (*routine)(int response), int ch) {
    messageRoutine = routine;
    if (messageRoutine)
        messageRoutine(ch);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("messageRoutine:"));
    assert!(assembly.contains("\tmovq %rax, messageRoutine(%rip)\n"));
    assert!(assembly.contains("\tcall *%rax\n"));
}

#[test]
fn compiler_accepts_struct_function_pointer_field_call_slice() {
    // given
    let source = r"typedef struct {
    void (*routine)(int choice);
} menuitem_t;
void choose(int choice) {
}
int main(void) {
    menuitem_t item;
    item.routine = choose;
    if (item.routine)
        item.routine(1);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tleaq choose(%rip), %rax\n"));
    assert!(assembly.contains("\tmovq %rax, 0(%rcx)\n"));
    assert!(assembly.contains("\tcall *%rax\n"));
}

#[test]
fn compiler_accepts_local_function_pointer_array_call_slice() {
    // given
    let source = r"int init(int width, int height, int ticks) {
    return 1;
}
int step(int width, int height, int ticks) {
    return 2;
}
int done(int width, int height, int ticks) {
    return 3;
}
int run(int wipeno, int width, int height, int ticks) {
    static int (*wipes[])(int, int, int) = {
        init, step, done
    };
    void mark(int, int, int, int);
    mark(0, 0, width, height);
    return (*wipes[wipeno])(width, height, ticks);
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tleaq init(%rip), %rax\n"));
    assert!(assembly.contains("\tleaq step(%rip), %rax\n"));
    assert!(assembly.contains("\tleaq done(%rip), %rax\n"));
    assert!(assembly.contains("\tcall mark\n"));
    assert!(assembly.contains("\tcall *%rax\n"));
}
