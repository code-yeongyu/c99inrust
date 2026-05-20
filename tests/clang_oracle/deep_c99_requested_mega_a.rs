use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn nested_struct_padding_exact_sizes_match_host_stdout_and_exit_code() {
    // given
    let name = "nested_struct_padding_exact_sizes";
    let source = "int puts(char*); typedef struct { char c; int i; short s; } leaf_t; typedef struct { char head; leaf_t leaves[2]; double d; char tail; } root_t; int main(void) { puts(\"mega-layout\"); return sizeof(leaf_t) + sizeof(root_t) + sizeof(root_t[2]); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn function_pointer_array_exact_call_sum_matches_host_stdout_and_exit_code() {
    // given
    let name = "function_pointer_array_exact_call_sum";
    let source = "int puts(char*); int add(int a, int b) { return a + b; } int sub(int a, int b) { return a - b; } int mul(int a, int b) { return a * b; } int main(void) { int (*ops[3])(int, int) = { add, sub, mul }; int pick = 2; puts(\"mega-fnptr\"); return ops[0](8, 5) + ops[1](8, 5) + ops[pick](8, 5); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn kr_style_unsigned_and_pointer_mix_matches_host_stdout_and_exit_code() {
    // given
    let name = "kr_style_unsigned_and_pointer_mix";
    let source = "int puts(char*); int fold(n, values, bias) unsigned n; int *values; unsigned char bias; { unsigned i; int total; total = bias; for (i = 0; i < n; i++) total = total + values[i]; return total; } int main(void) { int values[4] = { 1, 3, 5, 7 }; puts(\"mega-kr\"); return fold(4, values, 9); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn anonymous_struct_union_nested_exact_value_matches_host_stdout_and_exit_code() {
    // given
    let name = "anonymous_struct_union_nested_exact_value";
    let source = "int puts(char*); typedef struct { union { struct { short lo; short hi; }; int word; }; int tail; } pair_t; int main(void) { pair_t item; item.lo = 7; item.hi = 11; item.tail = 13; puts(\"mega-anon\"); return item.lo + item.hi + item.tail + sizeof(pair_t); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn typeof_attribute_nested_expression_matches_host_stdout_and_exit_code() {
    // given
    let name = "typeof_attribute_nested_expression";
    let source = "int puts(char*); int main(void) { int values[3] = { 4, 6, 8 }; __typeof__(values[1] + values[2]) total __attribute__((unused, aligned(4))) = values[1] + values[2]; puts(\"mega-typeof\"); return total + sizeof(total); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn three_file_extern_array_and_function_match_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "three_file_extern_array_and_function",
        files: &[
            OracleSourceFile {
                path: "state.c",
                source: "int values[3] = { 2, 4, 8 };\n",
            },
            OracleSourceFile {
                path: "sum.c",
                source: "int puts(char*); extern int values[3]; int sum_values(void) { puts(\"mega-extern\"); return values[0] + values[1] + values[2]; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "extern int values[3]; int sum_values(void); int main(void) { values[1] = values[1] + 5; return sum_values(); }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn static_inline_nested_call_exact_value_matches_host_stdout_and_exit_code() {
    // given
    let name = "static_inline_nested_call_exact_value";
    let source = "int puts(char*); static inline int scale(int value) { return value * 3; } static inline int mix(int left, int right) { return scale(left) + scale(right); } int main(void) { puts(\"mega-inline\"); return mix(5, 7); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn restrict_pointer_two_output_arrays_match_host_stdout_and_exit_code() {
    // given
    let name = "restrict_pointer_two_output_arrays";
    let source = "int puts(char*); void split(int n, int * restrict even, int * restrict odd, const int * restrict src) { int i; for (i = 0; i < n; i++) { even[i] = src[i] * 2; odd[i] = src[i] * 2 + 1; } } int main(void) { int src[3] = { 2, 3, 4 }; int even[3]; int odd[3]; split(3, even, odd, src); puts(\"mega-restrict\"); return even[2] + odd[1]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn constant_expression_array_size_exact_bytes_match_host_stdout_and_exit_code() {
    // given
    let name = "constant_expression_array_size_exact_bytes";
    let source = "int puts(char*); enum { BASE = sizeof(short) + 6, COUNT = (BASE << 1) - (3 ? 5 : 1) }; int main(void) { int values[COUNT]; puts(\"mega-constexpr\"); return sizeof(values); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn token_paste_macro_expansion_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "token_paste_macro_expansion_chain";
    let source = "#define JOIN(a, b) a ## b\n#define EXPAND_JOIN(a, b) JOIN(a, b)\n#define STEP0 value\n#define STEP1 STEP0\n#define STEP2 STEP1\n#define STEP3 STEP2\n#define STEP4 STEP3\nint puts(char*); int value42(void) { return 42; } int main(void) { puts(\"mega-macro-chain\"); return EXPAND_JOIN(STEP4, 42)(); }\n";

    // when/then
    assert_case(name, source);
}
