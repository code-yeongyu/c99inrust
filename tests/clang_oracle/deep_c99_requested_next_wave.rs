use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn next_wave_nested_struct_padding_alignment_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_nested_struct_padding_alignment";
    let source = "int puts(char*); typedef struct { char tag; double wide; short tail; } leaf_t; typedef struct { char head; leaf_t leaves[2]; int end; } root_t; int main(void) { puts(\"next-pad\"); return sizeof(leaf_t) == 24 && sizeof(root_t) == 56 && sizeof(root_t[2]) == 112 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_function_pointer_array_indirect_calls_match_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_function_pointer_array_indirect_calls";
    let source = "int puts(char*); int add(int a, int b) { return a + b; } int mul(int a, int b) { return a * b; } int main(void) { int (*ops[2])(int, int) = { add, mul }; int pick = 1; puts(\"next-fnptr\"); return ops[0](2, 3) == 5 && ops[pick](4, 5) == 20 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_kr_style_definition_with_promotions_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_kr_style_definition_with_promotions";
    let source = "int puts(char*); int blend(a, b, c) unsigned char a; short b; int c; { return a + b + c; } int main(void) { puts(\"next-kr\"); return blend(250, -5, 9) == 254 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_anonymous_struct_union_member_array_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_anonymous_struct_union_member_array";
    let source = "int puts(char*); typedef struct { union { struct { int x; int y; }; int raw[2]; }; int z; } cell_t; int main(void) { cell_t cell; cell.x = 5; cell.y = 7; cell.z = 11; puts(\"next-anon\"); return cell.raw[0] == 5 && cell.raw[1] == 7 && cell.z == 11 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_typeof_attribute_expression_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_typeof_attribute_expression";
    let source = "int puts(char*); int main(void) { int base = 4; __typeof__(base + 7) total __attribute__((unused, aligned(8))) = base * 3; puts(\"next-typeof\"); return total == 12 && sizeof(total) == sizeof(int) ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_multifile_extern_array_and_function_match_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "next_wave_multifile_extern_array_and_function",
        files: &[
            OracleSourceFile {
                path: "defs.h",
                source: "extern int shared[3]; int total_shared(void);\n",
            },
            OracleSourceFile {
                path: "state.c",
                source: "#include \"defs.h\"\nint shared[3] = { 2, 4, 6 }; int total_shared(void) { return shared[0] + shared[1] + shared[2]; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "#include \"defs.h\"\nint puts(char*); int main(void) { shared[1] = 9; puts(\"next-mf\"); return total_shared() == 17 ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn next_wave_static_inline_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_static_inline_chain";
    let source = "int puts(char*); static inline int inc(int x) { return x + 1; } static inline int twice(int x) { return inc(x) + inc(x); } int main(void) { puts(\"next-inline\"); return twice(5) == 12 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_restrict_pointer_accumulation_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_restrict_pointer_accumulation";
    let source = "int puts(char*); void mix(int * restrict out, int * restrict left, int * restrict right) { int i; for (i = 0; i < 3; i++) out[i] = left[i] + right[i]; } int main(void) { int a[3] = { 1, 2, 3 }; int b[3] = { 5, 7, 9 }; int out[3]; mix(out, a, b); puts(\"next-restrict\"); return out[0] == 6 && out[2] == 12 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_constant_expression_array_size_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_constant_expression_array_size";
    let source = "int puts(char*); enum { BASE = sizeof(short) + 6, COUNT = ((BASE << 1) > 12 ? BASE + 5 : 1) }; int main(void) { int values[COUNT - 2]; puts(\"next-constexpr\"); return sizeof(values) == 11 * sizeof(int) ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_recursive_macro_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_recursive_macro_chain";
    let source = "#define M0 1\n#define M1 M0 + 1\n#define M2 M1 + 1\n#define M3 M2 + 1\n#define M4 M3 + 1\n#define M5 M4 + 1\n#define M6 M5 + 1\n#define M7 M6 + 1\n#define M8 M7 + 1\n#define M9 M8 + 1\n#define M10 M9 + 1\n#define M11 M10 + 1\n#define M12 M11 + 1\n#define M13 M12 + 1\n#define M14 M13 + 1\n#define M15 M14 + 1\n#define M16 M15 + 1\n#define M17 M16 + 1\n#define M18 M17 + 1\n#define M19 M18 + 1\n#define M20 M19 + 1\nint puts(char*); int main(void) { puts(\"next-macro\"); return M20 == 21 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_variadic_macro_va_args_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_variadic_macro_va_args";
    let source = "#define CALL(fn, ...) fn(__VA_ARGS__)\n#define SUM3(a, b, c) ((a) + (b) + (c))\nint puts(char*); int main(void) { puts(\"next-va\"); return CALL(SUM3, 3, 5, 7) == 15 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_predefined_macros_match_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_predefined_macros";
    let source = "int puts(char*); int helper(void) { return __LINE__ > 0 && __func__[0] == 'h'; } int main(void) { char *file = __FILE__; puts(__func__); return helper() && file[0] != 0 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_nested_brace_initializer_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_nested_brace_initializer";
    let source = "int puts(char*); typedef struct { int cells[2][2]; int tag; } grid_t; int main(void) { grid_t grid = { { { 1, 2 }, { 3, 4 } }, 9 }; puts(\"next-init\"); return grid.cells[1][0] == 3 && grid.tag == 9 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_ternary_side_effects_match_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_ternary_side_effects";
    let source = "int puts(char*); int main(void) { int x = 0; int y = x++ ? (x = x + 10) : (x = x + 3); puts(\"next-ternary\"); return x == 4 && y == 4 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_sizeof_expression_and_type_match_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_sizeof_expression_and_type";
    let source = "int puts(char*); int bump(int *p) { *p = 99; return *p; } int main(void) { int x = 1; int values[5]; puts(\"next-sizeof\"); return sizeof(bump(&x)) == sizeof(int) && x == 1 && sizeof(values) == sizeof(int[5]) ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_incompatible_pointer_cast_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_incompatible_pointer_cast";
    let source = "int puts(char*); int main(void) { int value = 0; short *parts = (short*)&value; parts[0] = 123; puts(\"next-cast\"); return parts[0] == 123 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_char_signedness_conversion_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_char_signedness_conversion";
    let source = "int puts(char*); int main(void) { char c = 255; int promoted = c; puts(\"next-char\"); return promoted == -1 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_long_long_arithmetic_matches_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_long_long_arithmetic";
    let source = "int puts(char*); int main(void) { long long wide = (1LL << 40) + 255LL; long long mixed = wide - (1LL << 32); puts(\"next-ll\"); return (int)(wide >> 32) == 256 && (int)(mixed >> 32) == 255 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_decimal_float_literals_match_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_decimal_float_literals";
    let source = "int puts(char*); int main(void) { double a = 3.25e1L; double b = .75e1f; puts(\"next-float\"); return (int)(a + b) == 40 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn next_wave_hex_float_constants_match_host_stdout_and_exit_code() {
    // given
    let name = "next_wave_hex_float_constants";
    let source = "int puts(char*); int main(void) { double a = 0x1.8p+4; double b = 0x1p-1; puts(\"next-hex\"); return (int)(a + b) == 24 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
