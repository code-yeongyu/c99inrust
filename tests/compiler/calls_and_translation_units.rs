use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::{parse, parse_supported_translation_unit};

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
    assert_eq!(assembly, ".section .note.GNU-stack,\"\",@progbits\n");
}

#[test]
fn compiler_rejects_unsupported_data_only_translation_unit() {
    // given
    let source = "typedef int weaponinfo_t; weaponinfo_t* weaponinfo[2] = { 0, 0 };";

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
