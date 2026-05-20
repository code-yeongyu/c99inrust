use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn mega_c_nested_struct_padding_offsets_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_nested_struct_padding_offsets";
    let source = "int puts(char*); typedef struct { char tag; int count; short lane; } leaf_t; typedef union { leaf_t leaf; long long wide; char bytes[16]; } payload_t; typedef struct { char head; payload_t payload; short tail; } middle_t; typedef struct { char prefix; middle_t mids[2]; double marker; } root_t; int main(void) { root_t root; int p0 = (int)((char *)&root.mids[0].payload - (char *)&root); int p1 = (int)((char *)&root.mids[1].tail - (char *)&root); puts(\"mega-c-layout\"); return sizeof(leaf_t) + sizeof(payload_t) + sizeof(middle_t) + sizeof(root_t) + p0 + p1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_function_pointer_array_reindexed_calls_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_function_pointer_array_reindexed_calls";
    let source = "int puts(char*); int inc(int x) { return x + 1; } int twice(int x) { return x * 2; } int dec(int x) { return x - 1; } int triple(int x) { return x * 3; } int main(void) { int (*ops[4])(int) = { inc, twice, dec, triple }; int left = 1; int right = 3; puts(\"mega-c-fnptr\"); return ops[left](5) + ops[right](4) + ops[left - 1](9); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_kr_style_array_and_long_long_parameter_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_kr_style_array_and_long_long_parameter";
    let source = "int puts(char*); int fold(n, values, seed) unsigned short n; int values[]; long long seed; { unsigned short i; long long total = seed; for (i = 0; i < n; i++) total = total + values[i]; return (int)(total - seed); } int main(void) { int values[4] = { 2, 4, 6, 8 }; puts(\"mega-c-kr\"); return fold(4, values, 10000000000LL); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_anonymous_struct_union_byte_overlay_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_anonymous_struct_union_byte_overlay";
    let source = "int puts(char*); typedef struct { int tag; struct { union { int word; unsigned char bytes[4]; }; int extra; }; } packet_t; int main(void) { packet_t packet; packet.word = 0; packet.bytes[0] = 9; packet.bytes[1] = 7; packet.extra = 5; puts(\"mega-c-anon\"); return packet.bytes[0] + packet.bytes[1] + packet.extra + sizeof(packet_t); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_typeof_pointer_expression_with_attribute_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_typeof_pointer_expression_with_attribute";
    let source = "int puts(char*); int main(void) { int values[3] = { 4, 9, 16 }; int *cursor = values + 1; __typeof__(*cursor + values[2]) total __attribute__((unused, aligned(8))) = *cursor + values[2]; puts(\"mega-c-typeof\"); return total + sizeof(total); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_four_file_extern_state_pipeline_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "mega_c_four_file_extern_state_pipeline",
        files: &[
            OracleSourceFile {
                path: "state.c",
                source: "int shared_counter = 3;\n",
            },
            OracleSourceFile {
                path: "bump.c",
                source: "extern int shared_counter; int bump_shared(int step) { shared_counter = shared_counter + step; return shared_counter; }\n",
            },
            OracleSourceFile {
                path: "read.c",
                source: "extern int shared_counter; int read_shared(void) { return shared_counter * 2; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "int puts(char*); int bump_shared(int); int read_shared(void); int main(void) { puts(\"mega-c-extern\"); return bump_shared(4) + read_shared(); }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn mega_c_static_inline_nested_loop_calls_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_static_inline_nested_loop_calls";
    let source = "int puts(char*); static inline int scale(int value) { return value * 2 + 1; } static inline int mix(int left, int right) { return scale(left) + scale(right); } int main(void) { int i; int total = 0; for (i = 0; i < 3; i++) total = total + mix(i, i + 1); puts(\"mega-c-inline\"); return total; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_restrict_pointer_pairwise_rotation_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_restrict_pointer_pairwise_rotation";
    let source = "int puts(char*); void rotate(int n, int * restrict left, int * restrict right) { int i; for (i = 0; i < n; i++) { int old = left[i]; left[i] = right[i] + 1; right[i] = old + 2; } } int main(void) { int a[3] = { 1, 2, 3 }; int b[3] = { 5, 7, 9 }; rotate(3, a, b); puts(\"mega-c-restrict\"); return a[0] + a[2] + b[1]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_constant_expression_array_size_mix_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_constant_expression_array_size_mix";
    let source = "int puts(char*); enum { A = sizeof(long long), B = (A * 3 + 5) / 2, C = (B & 7) ? B : 1 }; int main(void) { int values[C - 1]; puts(\"mega-c-constexpr\"); return sizeof(values) + C; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_recursive_macro_expansion_ladder_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_recursive_macro_expansion_ladder";
    let source = "#define M0(x) (x)\n#define M1(x) M0((x) + 1)\n#define M2(x) M1((x) + 1)\n#define M3(x) M2((x) + 1)\n#define M4(x) M3((x) + 1)\n#define M5(x) M4((x) + 1)\n#define M6(x) M5((x) + 1)\nint puts(char*); int main(void) { puts(\"mega-c-macro\"); return M6(10); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_variadic_macro_nested_va_args_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_variadic_macro_nested_va_args";
    let source = "#define SUM3(a, b, c) ((a) + (b) + (c))\n#define FORWARD(first, ...) SUM3(first, __VA_ARGS__)\n#define TWICE(...) (FORWARD(__VA_ARGS__) + FORWARD(__VA_ARGS__))\nint puts(char*); int main(void) { puts(\"mega-c-vaargs\"); return TWICE(2, 3, 4); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_predefined_macro_line_file_func_flow_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_predefined_macro_line_file_func_flow";
    let source = "#define LINE_AT_USE() __LINE__\nint puts(char*); int helper(void) { puts(__func__); return LINE_AT_USE(); }\nint main(void) { char *file = __FILE__; int here = LINE_AT_USE(); puts(file); return file[0] != 0 && helper() > here && __func__[0] == 'm' ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_nested_initializer_braces_partial_zero_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_nested_initializer_braces_partial_zero";
    let source = "int puts(char*); typedef struct { int cells[2][3]; int tail; } grid_t; int main(void) { grid_t grid = { { { 1 }, { 2, 3 } }, 4 }; puts(\"mega-c-init\"); return grid.cells[0][0] + grid.cells[0][2] + grid.cells[1][1] + grid.cells[1][2] + grid.tail; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_ternary_side_effect_comma_branches_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_ternary_side_effect_comma_branches";
    let source = "int puts(char*); int main(void) { int flag = 0; int x = 2; int y = 5; int a = flag ? (x += 10, x) : (y += 3, y); flag = 1; int b = flag ? (x += a, x) : (y += 9, y); puts(\"mega-c-ternary\"); return x * 10 + y + a + b; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_sizeof_type_expression_and_array_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_sizeof_type_expression_and_array";
    let source = "int puts(char*); int bump(int *p) { *p = *p + 50; return *p; } int main(void) { int x = 4; int values[5]; int total = sizeof(int[3]) + sizeof values + sizeof(values[0] + bump(&x)); puts(\"mega-c-sizeof\"); return total + x; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_incompatible_pointer_struct_overlay_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_incompatible_pointer_struct_overlay";
    let source = "int puts(char*); typedef struct { int x; int y; } left_t; typedef struct { int a; int b; } right_t; int main(void) { left_t left; right_t *right = (right_t *)(void *)&left; right->a = 11; right->b = 13; puts(\"mega-c-ptrcast\"); return left.x + left.y; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_char_signedness_array_roundtrip_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_char_signedness_array_roundtrip";
    let source = "int puts(char*); int main(void) { char text[2] = { -1, 2 }; unsigned char first = text[0]; signed char back = first; puts(\"mega-c-char\"); return (text[0] < 0 ? 10 : 20) + (first == 255 ? 3 : 5) + (back == -1 ? 7 : 9) + text[1]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_long_long_multiply_divide_remainder_matches_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_long_long_multiply_divide_remainder";
    let source = "int puts(char*); int main(void) { long long base = 1234567LL; long long product = base * 97LL; long long quotient = product / 97LL; long long rem = product % 97LL; puts(\"mega-c-longlong\"); return quotient == base && rem == 0LL ? (int)(product & 127LL) : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_floating_literal_forms_truncate_like_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_floating_literal_forms_truncate_like_host";
    let source = "int puts(char*); int main(void) { double a = 12.; double b = .25e2; double c = 3e+1f; double d = 4e-1; puts(\"mega-c-float\"); return (int)(a + b + c + d); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mega_c_hex_float_uppercase_suffix_forms_match_host_stdout_and_exit_code() {
    // given
    let name = "mega_c_hex_float_uppercase_suffix_forms";
    let source = "int puts(char*); int main(void) { double a = 0X1.FP+3; double b = 0x1.0p-1f; puts(\"mega-c-hexfloat\"); return (int)(a + b); }\n";

    // when/then
    assert_case(name, source);
}
