use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::{parse, parse_supported_translation_unit};

#[test]
fn compiler_emits_native_assembly_for_constant_return_program() {
    // given
    let source = "int main(void) { return 40 + 2; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains(".globl _main"));
            assert!(assembly.contains("movz w0, #40"));
            assert!(assembly.contains("add w0, w0, w1"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains(".globl _main"));
            assert!(assembly.contains("movl $40, %eax"));
            assert!(assembly.contains("addl %ecx, %eax"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains(".globl main"));
            assert!(assembly.contains("movl $40, %eax"));
            assert!(assembly.contains("addl %ecx, %eax"));
        }
    }
    assert!(assembly.contains("ret"));
}

#[test]
fn compiler_emits_stack_slots_for_local_int_assignments() {
    // given
    let source = "int main(void) { int x = 40; int y = x + 1; x = y + 1; return x; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("str w0, [sp, #0]"));
            assert!(assembly.contains("ldr w0, [sp, #0]"));
            assert!(assembly.contains("str w0, [sp, #4]"));
        }
        Target::X86_64AppleDarwin | Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("movl %eax, -4(%rbp)"));
            assert!(assembly.contains("movl -4(%rbp), %eax"));
            assert!(assembly.contains("movl %eax, -8(%rbp)"));
        }
    }
    assert!(assembly.contains("ret"));
}

#[test]
fn compiler_accepts_compound_assignment_slice() {
    // given
    let source = "int main(void) { int x = 40; int y = 8; x += y / 2; x -= 1; return x; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tidivl %ecx\n"));
    assert!(assembly.contains("\taddl %ecx, %eax\n"));
    assert!(assembly.contains("\tsubl %ecx, %eax\n"));
    assert!(assembly.contains("ret"));
}

#[test]
fn compiler_accepts_local_pointer_declaration_slice() {
    // given
    let source = "int Z_Malloc(int size, int tag, void *user) { return 0; } void Z_Free(void *p) { return; } int main(void) { short *dest; dest = (short*) Z_Malloc(8, 1, 0); Z_Free(dest); return dest == 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("Z_Malloc"));
    assert!(assembly.contains("Z_Free"));
    assert!(assembly.contains("\tcltq\n"));
    assert!(assembly.contains("\tmovq %rax, -"));
    assert!(assembly.contains("ret"));
}

#[test]
fn compiler_accepts_pointer_dereference_slice() {
    // given
    let source = "int read_and_bump(int *p) { int value; value = *p; p++; return value; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("read_and_bump"));
    assert!(assembly.contains("\tmovl (%rcx,%rax,4), %eax\n"));
    assert!(assembly.contains("\taddq %rcx, %rax\n"));
}

#[test]
fn compiler_accepts_sizeof_type_slice() {
    // given
    let source = "int Z_Malloc(int size, int tag, void *user) { return 0; } int main(void) { int *y; y = (int*) Z_Malloc(4 * sizeof(int), 1, 0); return sizeof(int) == 4 ? 0 : 1; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("Z_Malloc"));
    assert!(assembly.contains("\tmovl $4, %eax\n"));
    assert!(assembly.contains("\timull %ecx, %eax\n"));
}

#[test]
fn compiler_accepts_post_decrement_condition_slice() {
    // given
    let source =
        "void run(int ticks) { while (ticks--) { ticks = ticks; } } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("run"));
    assert!(assembly.contains("\tsubl %ecx, %eax\n"));
    assert!(assembly.contains("\tje .Lrun_"));
}

#[test]
fn compiler_accepts_address_of_subscript_slice() {
    // given
    let source = "int address_of_subscript(int *p, int i) { int *q; q = &p[i]; return 0; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("address_of_subscript"));
    assert!(assembly.contains("\taddq %rcx, %rax\n"));
    assert!(assembly.contains("\tmovq %rax, -"));
}

#[test]
fn compiler_accepts_unsigned_cast_slice() {
    // given
    let source =
        "int main(void) { int x = 7; return ((unsigned)x >= 0 && (unsigned char)x == x) ? 0 : 1; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("\tcmpl %ecx, %eax\n"));
    assert!(assembly.contains("\tsetge %al\n"));
    assert!(assembly.contains("\tsete %al\n"));
}

#[test]
fn compiler_marks_linux_assembly_stack_non_executable() {
    // given
    let source = "int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let linux_assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("linux assembly should emit");
    let apple_assembly =
        emit_assembly(&lowered, Target::X86_64AppleDarwin).expect("apple assembly should emit");

    // then
    assert!(linux_assembly.contains(".section .note.GNU-stack,\"\",@progbits"));
    assert!(!apple_assembly.contains(".note.GNU-stack"));
}

#[test]
fn compiler_emits_branches_for_if_else_comparisons() {
    // given
    let source =
        "int main(void) { int x = 3; if (x >= 3) { x = 9; } else { x = 1; } return x == 9; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("cmp w0, w1"));
            assert!(assembly.contains("b.lt Lmain_"));
            assert!(assembly.contains("b Lmain_"));
            assert!(assembly.contains("cset w0, eq"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("cmpl %ecx, %eax"));
            assert!(assembly.contains("setge %al"));
            assert!(assembly.contains("je Lmain_"));
            assert!(assembly.contains("sete %al"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("cmpl %ecx, %eax"));
            assert!(assembly.contains("setge %al"));
            assert!(assembly.contains("je .Lmain_"));
            assert!(assembly.contains("sete %al"));
        }
    }
}

#[test]
fn compiler_emits_back_edges_for_while_loops() {
    // given
    let source = "int main(void) { int x = 0; while (x < 5) { x = x + 1; } return x; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("cmp w0, w1"));
            assert!(assembly.contains("b.ge Lmain_"));
            assert!(assembly.contains("add w0, w0, #1"));
            assert!(assembly.contains("b Lmain_"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("setl %al"));
            assert!(assembly.contains("je Lmain_"));
            assert!(assembly.contains("jmp Lmain_"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("setl %al"));
            assert!(assembly.contains("je .Lmain_"));
            assert!(assembly.contains("jmp .Lmain_"));
        }
    }
}

#[test]
fn compiler_emits_short_circuit_logical_branches() {
    // given
    let source = "int main(void) { int x = 0; if (x != 0 && 10 / x > 1) { return 1; } if (x == 0 || 10 / x > 1) { return 42; } return 2; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("b.eq Lmain_"));
            assert!(assembly.contains("b.ne Lmain_"));
            assert!(assembly.contains("sdiv w0, w0, w1"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("je Lmain_"));
            assert!(assembly.contains("jne Lmain_"));
            assert!(assembly.contains("idivl %ecx"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("je .Lmain_"));
            assert!(assembly.contains("jne .Lmain_"));
            assert!(assembly.contains("idivl %ecx"));
        }
    }
}

#[test]
fn compiler_emits_for_loop_back_edges() {
    // given
    let source = "int main(void) { int total = 0; for (int i = 0; i < 5; i = i + 1) { total = total + i; } return total; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("cmp w0, w1"));
            assert!(assembly.contains("b.ge Lmain_"));
            assert!(assembly.contains("add w0, w0, #1"));
            assert!(assembly.contains("b Lmain_"));
            assert!(assembly.contains("str w0, [sp, #4]"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("setl %al"));
            assert!(assembly.contains("je Lmain_"));
            assert!(assembly.contains("jmp Lmain_"));
            assert!(assembly.contains("movl %eax, -8(%rbp)"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("setl %al"));
            assert!(assembly.contains("je .Lmain_"));
            assert!(assembly.contains("jmp .Lmain_"));
            assert!(assembly.contains("movl %eax, -8(%rbp)"));
        }
    }
}

#[test]
fn compiler_emits_zero_arg_function_calls() {
    // given
    let source = "int answer(void) { int value = 40; return value; } int main(void) { return 2 + answer(); }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains(".globl _answer"));
            assert!(assembly.contains("str x30, [sp, #"));
            assert!(assembly.contains("bl _answer"));
            assert!(assembly.contains("ldr x30, [sp, #"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains(".globl _answer"));
            assert!(assembly.contains("call _answer"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains(".globl answer"));
            assert!(assembly.contains("call answer"));
        }
    }
    assert!(assembly.contains("ret"));
}

#[test]
fn compiler_emits_integer_function_call_arguments() {
    // given
    let source = "int add(int left, int right) { return left + right; } int main(int argc, char **argv) { return add(argc, 41); }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("\tldr w0, [sp, #0]\n"));
            assert!(assembly.contains("\tstr x1, [sp, #8]\n"));
            assert!(assembly.contains("\tmovz w0, #41\n"));
            assert!(assembly.contains("\tldr w0, [sp, #16]\n"));
            assert!(assembly.contains("\tldr w1, [sp, #24]\n"));
            assert!(assembly.contains("\tbl _add\n"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("\tmovl -4(%rbp), %eax\n"));
            assert!(assembly.contains("\tmovq %rsi, -16(%rbp)\n"));
            assert!(assembly.contains("\tmovl $41, %eax\n"));
            assert!(assembly.contains("\tmovl -20(%rbp), %edi\n"));
            assert!(assembly.contains("\tmovl -28(%rbp), %esi\n"));
            assert!(assembly.contains("\tcall _add\n"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("\tmovl -4(%rbp), %eax\n"));
            assert!(assembly.contains("\tmovq %rsi, -16(%rbp)\n"));
            assert!(assembly.contains("\tmovl $41, %eax\n"));
            assert!(assembly.contains("\tmovl -20(%rbp), %edi\n"));
            assert!(assembly.contains("\tmovl -28(%rbp), %esi\n"));
            assert!(assembly.contains("\tcall add\n"));
        }
    }
}

#[test]
fn compiler_emits_conditional_expression_branches() {
    // given
    let source = "int main(int argc, char **argv) { return argc < 0 ? 2 : 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("\tb.eq Lmain_"));
            assert!(assembly.contains("\tmovz w0, #2\n"));
            assert!(assembly.contains("\tmovz w0, #42\n"));
        }
        Target::X86_64AppleDarwin => {
            assert!(assembly.contains("\tje Lmain_"));
            assert!(assembly.contains("\tmovl $2, %eax\n"));
            assert!(assembly.contains("\tmovl $42, %eax\n"));
        }
        Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("\tje .Lmain_"));
            assert!(assembly.contains("\tmovl $2, %eax\n"));
            assert!(assembly.contains("\tmovl $42, %eax\n"));
        }
    }
}

#[test]
fn aarch64_keeps_binary_left_operand_in_preserved_register_across_direct_call() {
    // given
    let source = "int answer(void) { int value = 40; return value; } int main(void) { return 2 + answer(); }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains("str x19, [sp, #"));
    assert!(assembly.contains("mov w19, w0"));
    assert!(assembly.contains("bl _answer"));
    assert!(assembly.contains("mov w0, w19"));
    assert!(assembly.contains("ldr x19, [sp, #"));
}

#[test]
fn compiler_folds_calls_to_integer_constant_functions() {
    // given
    let source = "int tick(void) { return 1; } int main(void) { return tick(); }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _tick"));
    assert!(assembly.contains(".globl _main"));
    assert!(assembly.contains("movz w0, #1"));
    assert!(!assembly.contains("\tbl _tick"));
}

#[test]
fn compiler_skips_top_level_declarations_before_supported_functions() {
    // given
    let source = "static const char rcsid[] = \"doom\"; int main(void) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _main"));
    assert!(assembly.contains("movz w0, #42"));
}

#[test]
fn compiler_accepts_ignorable_static_metadata_translation_unit() {
    // given
    let source = "static const char rcsid[] = \"doom\";";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(program.functions.is_empty());
    assert!(program.globals.is_empty());
    assert!(assembly.is_empty());
}

#[test]
fn compiler_rejects_unsupported_data_only_translation_unit() {
    // given
    let source = "typedef int weaponinfo_t; weaponinfo_t weaponinfo[2] = { 1, 2 };";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let error = parse_supported_translation_unit(&tokens)
        .expect_err("translation unit should reject unsupported data-only globals");

    // then
    assert!(
        error
            .to_string()
            .contains("translation unit has no supported function definitions")
    );
}

#[test]
fn compiler_accepts_doomstat_enum_globals_slice() {
    // given
    let source = r"typedef enum { shareware, registered, commercial, retail, indetermined } GameMode_t;
typedef enum { doom, doom2, pack_tnt, pack_plut, none } GameMission_t;
typedef enum { english, french, german, unknown } Language_t;
typedef int boolean;
GameMode_t gamemode = indetermined;
GameMission_t gamemission = doom;
Language_t language = english;
boolean modifiedgame;";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("gamemode:"));
    assert!(assembly.contains("gamemission:"));
    assert!(assembly.contains("language:"));
    assert!(assembly.contains("modifiedgame:"));
    assert!(assembly.contains("\t.long 4\n"));
    assert!(assembly.contains("\t.long 0\n"));
}

#[test]
fn compiler_accepts_doomstat_globals_after_header_extern_arrays_slice() {
    // given
    let source = r#"static const char rcsid[] = "doom";
typedef enum { shareware, registered, commercial, retail, indetermined } GameMode_t;
typedef int boolean;
extern int finesine[5*8192/4];
extern char *sprnames[138];
extern boolean nomonsters;
extern GameMode_t gamemode;
GameMode_t gamemode = indetermined;
boolean modifiedgame;"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("gamemode:"));
    assert!(assembly.contains("modifiedgame:"));
    assert!(assembly.contains("\t.long 4\n"));
    assert!(assembly.contains("\t.long 0\n"));
}

#[test]
fn compiler_accepts_unparenthesized_global_integer_initializer_slice() {
    // given
    let source = "static int finit_height = 200 - 32;";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("finit_height:"));
    assert!(assembly.contains("\t.long 168\n"));
}

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
fn compiler_skips_aggregate_global_initializer_before_supported_function() {
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
    assert!(!assembly.contains("cheat_amap:"));
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("movl $42, %eax"));
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
fn compiler_emits_void_functions_with_value_less_return() {
    // given
    let source = "void tick(void) { return; } int main(void) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _tick"));
    assert!(assembly.contains("_tick:\n\tret"));
    assert!(assembly.contains(".globl _main"));
}

#[test]
fn compiler_adds_terminal_return_to_void_functions_that_can_fall_through() {
    // given
    let source = "void tick(void) { if (0) { return; } } int main(void) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains("_tick:\n\tmovz w0, #0"));
    assert!(assembly.contains("Ltick_0:\n\tret"));
}

#[test]
fn compiler_rejects_value_less_return_from_int_functions() {
    // given
    let source = "int main(void) { return; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let error = lower(&program).expect_err("lowering should reject a value-less int return");

    // then
    assert!(
        error
            .to_string()
            .contains("int function must return a value")
    );
}

#[test]
fn compiler_rejects_value_return_from_void_functions() {
    // given
    let source = "void tick(void) { return 1; } int main(void) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse(&tokens).expect("parser should succeed");
    let error = lower(&program).expect_err("lowering should reject a valued void return");

    // then
    assert!(
        error
            .to_string()
            .contains("void function cannot return a value")
    );
}

#[test]
fn compiler_accepts_parameter_list_signatures_when_body_does_not_use_parameters() {
    // given
    let source = "int main(int argc, char **argv) { return 42; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _main"));
    assert!(assembly.contains("movz w0, #42"));
}

#[test]
fn compiler_binds_parameters_as_local_slots_on_aarch64() {
    // given
    let source = "int identity(int value) { return value; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains("_identity:\n\tsub sp, sp, #16\n\tstr w0, [sp, #0]"));
    assert!(assembly.contains("\tldr w0, [sp, #0]"));
}

#[test]
fn compiler_binds_parameters_as_local_slots_on_x86_64() {
    // given
    let source = "int identity(int value) { return value; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("identity:\n\tpushq %rbp"));
    assert!(assembly.contains("\tmovl %edi, -4(%rbp)"));
    assert!(assembly.contains("\tmovl -4(%rbp), %eax"));
}

#[test]
fn compiler_emits_signed_long_long_cast_intermediates() {
    // given
    let source = "typedef int fixed_t; fixed_t FixedMul(fixed_t a, fixed_t b) { return ((long long) a * (long long) b) >> 16; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("_FixedMul:\n"));
            assert!(assembly.contains("\tsxtw x0, w0\n"));
            assert!(assembly.contains("\tmul x0, x0, x1\n"));
            assert!(assembly.contains("\tasr x0, x0, x1\n"));
        }
        Target::X86_64AppleDarwin | Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("FixedMul:\n") || assembly.contains("_FixedMul:\n"));
            assert!(assembly.contains("\tmovl -4(%rbp), %eax\n\tcltq\n"));
            assert!(assembly.contains("\timulq %rcx, %rax\n"));
            assert!(assembly.contains("\tsarq %cl, %rax\n"));
        }
    }
}

#[test]
fn compiler_accepts_fixeddiv2_double_slice() {
    // given
    let source = r#"typedef int fixed_t;
void I_Error(char *message) { return; }
fixed_t FixedDiv2(fixed_t a, fixed_t b) {
    double c;
    c = ((double)a) / ((double)b) * (1<<16);
    if (c >= 2147483648.0 || c < -2147483648.0)
        I_Error("FixedDiv: divide by zero");
    return (fixed_t)c;
}
int main(void) { return FixedDiv2(3, 2) == 98304 ? 0 : 1; }"#;

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let target = Target::native();
    let assembly = emit_assembly(&lowered, target).expect("assembly should emit");

    // then
    assert!(assembly.contains("FixedDiv2"));
    assert!(assembly.contains("I_Error"));
    assert!(assembly.contains(
        ".byte 70,105,120,101,100,68,105,118,58,32,100,105,118,105,100,101,32,98,121,32,122,101,114,111,0"
    ));
    match target {
        Target::Aarch64AppleDarwin => {
            assert!(assembly.contains("\tfdiv d0, d0, d1\n"));
            assert!(assembly.contains("\tfmul d0, d0, d1\n"));
            assert!(assembly.contains("\tfcmp d0, d1\n"));
        }
        Target::X86_64AppleDarwin | Target::X86_64UnknownLinuxGnu => {
            assert!(assembly.contains("\tdivsd %xmm1, %xmm0\n"));
            assert!(assembly.contains("\tmulsd %xmm1, %xmm0\n"));
            assert!(assembly.contains("\tucomisd %xmm1, %xmm0\n"));
        }
    }
}

#[test]
fn compiler_accepts_m_random_global_array_slice() {
    // given
    let source = r"unsigned char rndtable[4] = { 3, 5, 7, 11 };
int rndindex = 0;
int prndindex = 0;
int P_Random(void) {
    prndindex = (prndindex + 1) & 0x3;
    return rndtable[prndindex];
}
int M_Random(void) {
    rndindex = (rndindex + 1) & 0x3;
    return rndtable[rndindex];
}
void M_ClearRandom(void) {
    rndindex = prndindex = 0;
}
int main(void) {
    int a = P_Random();
    int b = M_Random();
    M_ClearRandom();
    return a == 5 && b == 5 ? 0 : 1;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly = emit_assembly(&lowered, Target::native()).expect("assembly should emit");

    // then
    assert!(assembly.contains("rndtable"));
    assert!(assembly.contains("rndindex"));
    assert!(assembly.contains("prndindex"));
    assert!(assembly.contains("M_ClearRandom"));
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

#[test]
fn compiler_accepts_i_main_global_pointer_slice() {
    // given
    let source = r"int myargc;
char **myargv;
void D_DoomMain(void) { return; }
int main(int argc, char **argv) {
    myargc = argc;
    myargv = argv;
    D_DoomMain();
    if (myargv != argv)
        return 2;
    return myargc == argc ? 0 : 1;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly = emit_assembly(&lowered, Target::native()).expect("assembly should emit");
    let linux_x86_64_assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("linux assembly should emit");

    // then
    assert!(assembly.contains("myargc"));
    assert!(assembly.contains("myargv"));
    assert!(assembly.contains("D_DoomMain"));
    assert!(linux_x86_64_assembly.contains("\t.quad 0\n"));
    assert!(linux_x86_64_assembly.contains("\tmovq %rax, myargv(%rip)\n"));
    assert!(linux_x86_64_assembly.contains("\tmovq myargv(%rip), %rax\n"));
}

#[test]
fn compiler_accepts_i_main_extern_global_slice() {
    // given
    let source = r"extern int myargc;
extern char **myargv;
void D_DoomMain(void);
int main(int argc, char **argv) {
    myargc = argc;
    myargv = argv;
    D_DoomMain();
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let linux_x86_64_assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("linux assembly should emit");

    // then
    assert!(!linux_x86_64_assembly.contains("\t.long 0\n"));
    assert!(!linux_x86_64_assembly.contains("\t.quad 0\n"));
    assert!(linux_x86_64_assembly.contains("\tmovl %eax, myargc(%rip)\n"));
    assert!(linux_x86_64_assembly.contains("\tmovq %rax, myargv(%rip)\n"));
    assert!(linux_x86_64_assembly.contains("\tcall D_DoomMain\n"));
}

#[test]
fn compiler_accepts_m_argv_post_increment_slice() {
    // given
    let source = r"int myargc;
char **myargv;
int M_CheckParm(char *check) {
    int i;
    for (i = 1; i < myargc; i++) {
        if (!strcasecmp(check, myargv[i]))
            return i;
    }
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let linux_x86_64_assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("linux assembly should emit");

    // then
    assert!(linux_x86_64_assembly.contains("M_CheckParm"));
    assert!(linux_x86_64_assembly.contains("myargc"));
    assert!(linux_x86_64_assembly.contains("myargv"));
    assert!(linux_x86_64_assembly.contains("\taddl %ecx, %eax\n"));
}

#[test]
fn compiler_accepts_typedef_return_signatures() {
    // given
    let source = "typedef int fixed_t; fixed_t FixedMul(fixed_t a, fixed_t b) { return 42; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _FixedMul"));
    assert!(assembly.contains("movz w0, #42"));
}

#[test]
fn compiler_accepts_split_line_typedef_return_signatures() {
    // given
    let source = "typedef int fixed_t; fixed_t\nFixedMul\n(fixed_t a,\n fixed_t b)\n{ return 42; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _FixedMul"));
    assert!(assembly.contains("movz w0, #42"));
}

#[test]
fn compiler_accepts_unsigned_scalar_return_signatures() {
    // given
    let source =
        "unsigned short SwapSHORT(unsigned short x) { return 42; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl _SwapSHORT"));
    assert!(assembly.contains("movz w0, #42"));
}

#[test]
fn compiler_accepts_doom_member_access_slice() {
    // given
    let source = r"typedef int fixed_t;
typedef struct { fixed_t x,y; } mpoint_t;
typedef struct { mpoint_t a,b; } mline_t;
typedef struct { fixed_t slp, islp; } islope_t;
void AM_getIslope(mline_t* ml, islope_t* is) {
    int dx, dy;
    dy = ml->a.y - ml->b.y;
    dx = ml->b.x - ml->a.x;
    is->islp = dx + dy;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains(".globl AM_getIslope"));
    assert!(assembly.contains("\tmovl 4(%rax), %eax\n"));
    assert!(assembly.contains("\tmovl 12(%rax), %eax\n"));
    assert!(assembly.contains("\tmovl 8(%rax), %eax\n"));
    assert!(assembly.contains("\tmovl 0(%rax), %eax\n"));
    assert!(assembly.contains("\tmovl %eax, 4(%rcx)\n"));
}

#[test]
fn compiler_rejects_pointer_return_signatures() {
    // given
    let source = "char *name(void) { return 0; } int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let error = parse_supported_translation_unit(&tokens)
        .expect_err("translation unit should reject pointer returns");

    // then
    assert!(
        error
            .to_string()
            .contains("unsupported function definition: name")
    );
}
