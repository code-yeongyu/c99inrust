use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn mega_d_nested_struct_union_alignment_stride_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_nested_struct_union_alignment_stride";
    let source = "int puts(char*); typedef struct { char code; double amount; int tail; } node_t; typedef struct { short tag; union { node_t node; char bytes[32]; } u; char end; } wrap_t; typedef struct { char head; wrap_t wraps[3]; int done; } root_t; int main(void) { root_t root; int a = (int)((char *)&root.wraps[1].u.node.amount - (char *)&root); int b = (int)((char *)&root.done - (char *)&root); puts(\"mega-d-layout\"); return sizeof(node_t) + sizeof(wrap_t) + sizeof(root_t) + a + b; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_function_pointer_array_indirect_calls_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_function_pointer_array_indirect_calls";
    let source = "int puts(char*); int add(int a, int b) { return a + b; } int sub(int a, int b) { return a - b; } int mul(int a, int b) { return a * b; } int mix(int a, int b) { return a * 10 + b; } int main(void) { int (*table[4])(int, int) = { add, sub, mul, mix }; int index = 2; puts(\"mega-d-fnptr\"); return table[index](3, 4) + table[index - 1](9, 2) + table[3](5, 6); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_kr_function_pointer_and_char_parameters_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_kr_function_pointer_and_char_parameters";
    let source = "int puts(char*); int legacy(count, values, bump) unsigned count; int *values; char bump; { unsigned i; int total = 0; for (i = 0; i < count; i++) { values[i] = values[i] + bump; total = total + values[i]; } return total; } int main(void) { int values[3] = { 4, 5, 6 }; puts(\"mega-d-kr\"); return legacy(3, values, 2); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_anonymous_struct_inside_union_fields_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_anonymous_struct_inside_union_fields";
    let source = "int puts(char*); typedef struct { int kind; union { struct { short lo; short hi; }; int word; }; int tail; } pair_t; int main(void) { pair_t pair; pair.word = 0; pair.lo = 7; pair.hi = 9; pair.tail = 5; puts(\"mega-d-anon\"); return pair.lo + pair.hi + pair.tail + sizeof(pair_t); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_typeof_conditional_with_aligned_attribute_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_typeof_conditional_with_aligned_attribute";
    let source = "int puts(char*); int main(void) { int left = 8; int right = 13; __typeof__(left > right ? left : right) chosen __attribute__((unused, aligned(16))) = left > right ? left : right; puts(\"mega-d-typeof\"); return chosen + sizeof(chosen); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_extern_array_shared_across_units_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "mega_d_extern_array_shared_across_units",
        files: &[
            OracleSourceFile {
                path: "state.c",
                source: "int shared_values[3] = { 2, 4, 6 };\n",
            },
            OracleSourceFile {
                path: "bump.c",
                source: "extern int shared_values[3]; int bump_shared(int index) { shared_values[index] = shared_values[index] + index + 1; return shared_values[index]; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "int puts(char*); extern int shared_values[3]; int bump_shared(int); int main(void) { puts(\"mega-d-extern\"); return bump_shared(0) + bump_shared(2) + shared_values[1]; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn mega_d_static_inline_chained_helpers_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_static_inline_chained_helpers";
    let source = "int puts(char*); static inline int square(int x) { return x * x; } static inline int fold(int a, int b) { return square(a) + square(b) + a; } int main(void) { int i; int total = 0; for (i = 1; i <= 3; i++) total = total + fold(i, i + 1); puts(\"mega-d-inline\"); return total; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_restrict_const_pointer_accumulator_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_restrict_const_pointer_accumulator";
    let source = "int puts(char*); void add_scaled(int n, int * restrict out, const int * restrict left, const int * restrict right) { int i; for (i = 0; i < n; i++) out[i] = left[i] * 2 + right[i] * 3; } int main(void) { int left[3] = { 1, 2, 3 }; int right[3] = { 4, 5, 6 }; int out[3]; add_scaled(3, out, left, right); puts(\"mega-d-restrict\"); return out[0] + out[1] + out[2]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_constant_expression_array_bound_mix_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_constant_expression_array_bound_mix";
    let source = "int puts(char*); enum { BASE = sizeof(short) << 2, EXTRA = (BASE + 9) / 3, COUNT = (EXTRA & 1) ? EXTRA + 5 : EXTRA + 7 }; int main(void) { int values[COUNT]; puts(\"mega-d-constexpr\"); return sizeof(values) + COUNT; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_recursive_macro_expansion_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_recursive_macro_expansion_chain";
    let source = "#define R0(x) (x)\n#define R1(x) R0((x) * 2)\n#define R2(x) R1((x) + 1)\n#define R3(x) R2((x) + 1)\n#define R4(x) R3((x) + 1)\n#define R5(x) R4((x) + 1)\n#define R6(x) R5((x) + 1)\n#define R7(x) R6((x) + 1)\nint puts(char*); int main(void) { puts(\"mega-d-macro\"); return R7(3); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_variadic_macro_forwarding_twice_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_variadic_macro_forwarding_twice";
    let source = "#define JOIN3(a, b, c) ((a) * 100 + (b) * 10 + (c))\n#define CALL3(...) JOIN3(__VA_ARGS__)\n#define BOTH(x, ...) (CALL3(x, __VA_ARGS__) + CALL3(__VA_ARGS__, x))\nint puts(char*); int main(void) { puts(\"mega-d-vaargs\"); return BOTH(1, 2, 3); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_predefined_macros_relative_lines_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_predefined_macros_relative_lines";
    let source = "#define LINE_MARK() __LINE__\nint puts(char*); int earlier(void) { return LINE_MARK(); }\nint main(void) { int here = LINE_MARK(); char *file = __FILE__; puts(__func__); puts(file); return file[0] != 0 && earlier() < here && __func__[0] == 'm' ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_nested_initializer_list_holes_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_nested_initializer_list_holes";
    let source = "int puts(char*); typedef struct { int b[2]; } row_t; typedef struct { int a[2]; row_t inner[2]; int tail; } bag_t; int main(void) { bag_t bag = { { 1 }, { { { 2, 3 } }, { { 4 } } }, 5 }; puts(\"mega-d-init\"); return bag.a[0] + bag.a[1] + bag.inner[0].b[1] + bag.inner[1].b[0] + bag.inner[1].b[1] + bag.tail; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_ternary_side_effect_selected_branch_only_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_ternary_side_effect_selected_branch_only";
    let source = "int puts(char*); int main(void) { int x = 1; int y = 10; int z = 20; int a = x++ ? (y += x, y) : (z += 100, z); int b = (x - 2) ? (z += a, z) : (y += 3, y); puts(\"mega-d-ternary\"); return x + y + z + a + b; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_sizeof_suppresses_expression_side_effects_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_sizeof_suppresses_expression_side_effects";
    let source = "int puts(char*); int bump(int *p) { *p = *p + 40; return *p; } int main(void) { int x = 5; int values[4]; int total = sizeof x + sizeof(int *) + sizeof(values) + sizeof(bump(&x)); puts(\"mega-d-sizeof\"); return total + x; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_incompatible_pointer_cast_halfword_overlay_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_incompatible_pointer_cast_halfword_overlay";
    let source = "int puts(char*); typedef struct { short lo; short hi; } halves_t; int main(void) { int word = 0; halves_t *halves = (halves_t *)(void *)&word; halves->lo = 3; halves->hi = 4; puts(\"mega-d-ptrcast\"); return halves->lo + halves->hi + (word != 0); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_char_signedness_cast_roundtrip_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_char_signedness_cast_roundtrip";
    let source = "int puts(char*); int main(void) { char c = (char)0x80; unsigned char u = c; signed char s = c; puts(\"mega-d-char\"); return (c < 0 ? 10 : 20) + (u == 128 ? 3 : 5) + (s < 0 ? 7 : 9); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_long_long_shift_divide_mod_mix_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_long_long_shift_divide_mod_mix";
    let source = "int puts(char*); int main(void) { long long value = (1LL << 50) + (3LL << 20) + 77LL; long long scaled = value / 7LL; long long round = scaled * 7LL + value % 7LL; puts(\"mega-d-longlong\"); return round == value ? (int)((value >> 20) & 255LL) : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_decimal_float_literals_with_suffixes_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_decimal_float_literals_with_suffixes";
    let source = "int puts(char*); int main(void) { double a = 6.0e1; double b = .125f; double c = 10e-1; double d = 2.; puts(\"mega-d-float\"); return (int)(a + b + c + d); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_d_hex_float_fraction_and_suffix_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_d_hex_float_fraction_and_suffix";
    let source = "int puts(char*); int main(void) { double a = 0x1.4p+2; double b = 0XAP-1f; puts(\"mega-d-hexfloat\"); return (int)(a + b); }\n";

    // when/then
    assert_case(name, source);
}
