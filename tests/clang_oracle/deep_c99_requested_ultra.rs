use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn nested_struct_padding_with_double_alignment_matches_host_stdout_and_exit_code() {
    // given
    let name = "nested_struct_padding_with_double_alignment";
    let source = "int puts(char*); typedef struct { char tag; short count; int value; } leaf_t; typedef struct { char head; leaf_t leaf; double wide; char tail; } packet_t; int main(void) { puts(\"nested-pad2\"); return sizeof(leaf_t) == 8 && sizeof(packet_t) == 32 && sizeof(packet_t[2]) == 64 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn indexed_function_pointer_table_matches_host_stdout_and_exit_code() {
    // given
    let name = "indexed_function_pointer_table";
    let source = "int puts(char*); int add(int a, int b) { return a + b; } int sub(int a, int b) { return a - b; } int mul(int a, int b) { return a * b; } int max2(int a, int b) { return a > b ? a : b; } int main(void) { int (*ops[4])(int, int) = { add, sub, mul, max2 }; int index = 2; puts(\"fn-table2\"); return ops[index](3, 4) == 12 && ops[index - 1](9, 2) == 7 && ops[3](5, 8) == 8 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn kr_style_pointer_parameter_loop_matches_host_stdout_and_exit_code() {
    // given
    let name = "kr_style_pointer_parameter_loop";
    let source = "int puts(char*); int fold(n, values, bias) int n; int *values; int bias; { int i; int sum; sum = bias; for (i = 0; i < n; i++) sum = sum + values[i]; return sum; } int main(void) { int values[3] = { 2, 4, 6 }; puts(\"kr-pointer\"); return fold(3, values, 5) == 17 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn anonymous_struct_union_fields_share_storage_match_host_stdout_and_exit_code() {
    // given
    let name = "anonymous_struct_union_fields_share_storage";
    let source = "int puts(char*); typedef struct { int tag; union { struct { int x; int y; }; int pair[2]; }; } cell_t; int main(void) { cell_t cell; cell.x = 3; cell.y = 9; puts(\"anon-array\"); return cell.pair[0] == 3 && cell.pair[1] == 9 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn typeof_member_expression_with_attribute_matches_host_stdout_and_exit_code() {
    // given
    let name = "typeof_member_expression_with_attribute";
    let source = "int puts(char*); typedef struct { int value; } item_t; int main(void) { item_t item; item.value = 5; __typeof__(item.value) copy __attribute__((unused)) = item.value + 2; puts(\"typeof-member\"); return copy == 7 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn three_file_extern_linkage_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "three_file_extern_linkage",
        files: &[
            OracleSourceFile {
                path: "state.c",
                source: "int shared = 5;\n",
            },
            OracleSourceFile {
                path: "scale.c",
                source: "int puts(char*); extern int shared; int scale_shared(int value) { puts(\"extern3\"); return value * shared; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "extern int shared; int scale_shared(int value); int main(void) { return scale_shared(3) == 15 && shared == 5 ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn static_inline_call_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "static_inline_call_chain";
    let source = "int puts(char*); static inline int twice(int value) { return value * 2; } static inline int add_twice(int value, int bias) { return twice(value) + bias; } int main(void) { puts(\"static-inline2\"); return add_twice(6, 3) == 15 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn restrict_qualified_const_source_updates_match_host_stdout_and_exit_code() {
    // given
    let name = "restrict_qualified_const_source_updates";
    let source = "int puts(char*); void add_scaled(int n, int * restrict dst, const int * restrict src, int scale) { int i; for (i = 0; i < n; i++) dst[i] = dst[i] + src[i] * scale; } int main(void) { int dst[3] = { 1, 2, 3 }; int src[3] = { 2, 3, 4 }; add_scaled(3, dst, src, 5); puts(\"restrict-scale\"); return dst[0] == 11 && dst[1] == 17 && dst[2] == 23 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn sizeof_based_constant_expression_array_matches_host_stdout_and_exit_code() {
    // given
    let name = "sizeof_based_constant_expression_array";
    let source = "int puts(char*); enum { BASE = sizeof(long long), WIDE = BASE << 1, COUNT = WIDE > 8 ? WIDE - 3 : 1 }; int main(void) { int values[COUNT + 6]; puts(\"constexpr-sizeof\"); return sizeof(values) == 19 * sizeof(int) ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn eighty_step_macro_expansion_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "eighty_step_macro_expansion_chain";
    let source = concat!(
        "#define R0 13\n",
        "#define R1 R0\n",
        "#define R2 R1\n",
        "#define R3 R2\n",
        "#define R4 R3\n",
        "#define R5 R4\n",
        "#define R6 R5\n",
        "#define R7 R6\n",
        "#define R8 R7\n",
        "#define R9 R8\n",
        "#define R10 R9\n",
        "#define R11 R10\n",
        "#define R12 R11\n",
        "#define R13 R12\n",
        "#define R14 R13\n",
        "#define R15 R14\n",
        "#define R16 R15\n",
        "#define R17 R16\n",
        "#define R18 R17\n",
        "#define R19 R18\n",
        "#define R20 R19\n",
        "#define R21 R20\n",
        "#define R22 R21\n",
        "#define R23 R22\n",
        "#define R24 R23\n",
        "#define R25 R24\n",
        "#define R26 R25\n",
        "#define R27 R26\n",
        "#define R28 R27\n",
        "#define R29 R28\n",
        "#define R30 R29\n",
        "#define R31 R30\n",
        "#define R32 R31\n",
        "#define R33 R32\n",
        "#define R34 R33\n",
        "#define R35 R34\n",
        "#define R36 R35\n",
        "#define R37 R36\n",
        "#define R38 R37\n",
        "#define R39 R38\n",
        "#define R40 R39\n",
        "#define R41 R40\n",
        "#define R42 R41\n",
        "#define R43 R42\n",
        "#define R44 R43\n",
        "#define R45 R44\n",
        "#define R46 R45\n",
        "#define R47 R46\n",
        "#define R48 R47\n",
        "#define R49 R48\n",
        "#define R50 R49\n",
        "#define R51 R50\n",
        "#define R52 R51\n",
        "#define R53 R52\n",
        "#define R54 R53\n",
        "#define R55 R54\n",
        "#define R56 R55\n",
        "#define R57 R56\n",
        "#define R58 R57\n",
        "#define R59 R58\n",
        "#define R60 R59\n",
        "#define R61 R60\n",
        "#define R62 R61\n",
        "#define R63 R62\n",
        "#define R64 R63\n",
        "#define R65 R64\n",
        "#define R66 R65\n",
        "#define R67 R66\n",
        "#define R68 R67\n",
        "#define R69 R68\n",
        "#define R70 R69\n",
        "#define R71 R70\n",
        "#define R72 R71\n",
        "#define R73 R72\n",
        "#define R74 R73\n",
        "#define R75 R74\n",
        "#define R76 R75\n",
        "#define R77 R76\n",
        "#define R78 R77\n",
        "#define R79 R78\n",
        "#define R80 R79\n",
        "int puts(char*); int main(void) { puts(\"macro80\"); return R80 == 13 ? 0 : 1; }\n",
    );

    // when/then
    assert_case(name, source);
}

#[test]
fn variadic_macro_nested_dispatch_matches_host_stdout_and_exit_code() {
    // given
    let name = "variadic_macro_nested_dispatch";
    let source = "#define CALL(fn, ...) fn(__VA_ARGS__)\n#define TWICE(fn, ...) (CALL(fn, __VA_ARGS__) + CALL(fn, __VA_ARGS__))\nint puts(char*); int mix(int a, int b, int c) { return a * 100 + b * 10 + c; } int main(void) { puts(\"varargs2\"); return TWICE(mix, 1, 2, 3) == 246 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn predefined_macros_in_macro_and_function_match_host_stdout_and_exit_code() {
    // given
    let name = "predefined_macros_in_macro_and_function";
    let source = "#define CURRENT_LINE() __LINE__\nint puts(char*); int helper(void) { return CURRENT_LINE(); } int main(void) { char *file = __FILE__; puts(__func__); return helper() > 0 && file[0] != 0 && __func__[0] == 'm' ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn nested_struct_matrix_initializer_matches_host_stdout_and_exit_code() {
    // given
    let name = "nested_struct_matrix_initializer";
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; pair_t pairs[3] = { { 1, 2 }, { 3, 4 }, { 5, 6 } }; int main(void) { puts(\"init-matrix\"); return pairs[0].y == 2 && pairs[2].x == 5 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn ternary_comma_side_effect_branch_matches_host_stdout_and_exit_code() {
    // given
    let name = "nested_ternary_side_effect_branch";
    let source = "int puts(char*); int main(void) { int x = 0; int y = 1; int result = x++ == 0 ? ++y : y++; int second = x == 1 ? x++ : ++x; puts(\"ternary-comma\"); return x == 2 && y == 2 && result == 2 && second == 1 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn sizeof_unparenthesized_expression_and_array_type_match_host_stdout_and_exit_code() {
    // given
    let name = "sizeof_unparenthesized_expression_and_array_type";
    let source = "int puts(char*); int bump(int *value) { *value = 99; return *value; } int main(void) { int x = 1; int values[3]; int ok = sizeof bump(&x) == sizeof(int) && x == 1 && sizeof values == sizeof(int[3]); puts(\"sizeof-unparen\"); return ok ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn incompatible_pointer_cast_through_void_matches_host_stdout_and_exit_code() {
    // given
    let name = "incompatible_pointer_cast_through_void";
    let source = "int puts(char*); typedef struct { int a; int b; } pair_t; typedef struct { int x; int y; } point_t; int main(void) { pair_t pair; point_t *point = (point_t *)(void *)&pair; point->x = 6; point->y = 7; puts(\"ptr-cast2\"); return pair.a + pair.b == 13 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn signed_and_unsigned_char_conversion_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "signed_and_unsigned_char_conversion_chain";
    let source = "int puts(char*); int main(void) { char plain[1]; unsigned char wide[1]; signed char narrow[1]; plain[0] = -1; wide[0] = plain[0]; narrow[0] = wide[0]; puts(\"char-chain2\"); return plain[0] < 0 && wide[0] == 255 && narrow[0] == -1 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn long_long_shift_roundtrip_matches_host_stdout_and_exit_code() {
    // given
    let name = "long_long_shift_roundtrip";
    let source = "int puts(char*); int main(void) { long long one = 1LL; long long value = (one << 40) + 12345LL; long long high = value >> 32; long long low = value - (high << 32); puts(\"ll-roundtrip\"); return high == 256LL && low == 12345LL ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn decimal_float_literal_forms_match_host_stdout_and_exit_code() {
    // given
    let name = "decimal_float_literal_forms";
    let source = "int puts(char*); int main(void) { double a = 6.5e1; double b = .125e2; double c = 2.; puts(\"float-forms\"); return (int)(a + b + c) == 79 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn hex_float_mixed_exponents_match_host_stdout_and_exit_code() {
    // given
    let name = "hex_float_mixed_exponents";
    let source = "int puts(char*); int main(void) { double a = 0x1.8p+3; double b = 0x1p-1; puts(\"hexfloat2\"); return (int)(a + b) == 12 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
