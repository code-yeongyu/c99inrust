use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn stress_nested_struct_union_padding_offsets_match_host_stdout_and_exit_code() {
    // given
    let name = "stress_nested_struct_union_padding_offsets";
    let source = "int puts(char*); typedef struct { char c; int i; short s; } leaf_t; typedef union { leaf_t leaf; double d; char raw[24]; } payload_t; typedef struct { short tag; payload_t payload[2]; char tail; } node_t; int main(void) { node_t node; int a = (int)((char *)&node.payload[0].leaf.i - (char *)&node); int b = (int)((char *)&node.payload[1].leaf.s - (char *)&node); puts(\"stress-layout\"); return sizeof(leaf_t) + sizeof(payload_t) + sizeof(node_t) + a + b; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_function_pointer_array_reassignment_matches_host_stdout_and_exit_code() {
    // given
    let name = "stress_function_pointer_array_reassignment";
    let source = "int puts(char*); int add(int a, int b) { return a + b; } int sub(int a, int b) { return a - b; } int mul(int a, int b) { return a * b; } int mix(int a, int b) { return a * 10 + b; } int main(void) { int (*ops[4])(int, int); int index = 0; ops[0] = add; ops[1] = sub; ops[2] = mul; ops[3] = mix; index++; puts(\"stress-fnptr\"); return ops[index](9, 4) + ops[index + 1](3, 7) + ops[3](2, 5); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_kr_style_mixed_pointer_and_unsigned_params_match_host_stdout_and_exit_code() {
    // given
    let name = "stress_kr_style_mixed_pointer_and_unsigned_params";
    let source = "int puts(char*); int legacy(n, values, bias) unsigned n; int *values; short bias; { unsigned i; int total = bias; for (i = 0; i < n; i++) total = total + values[i] * (int)(i + 1); return total; } int main(void) { int values[3] = { 4, 5, 6 }; puts(\"stress-kr\"); return legacy(3, values, -2); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_anonymous_struct_union_nested_overlay_matches_host_stdout_and_exit_code() {
    // given
    let name = "stress_anonymous_struct_union_nested_overlay";
    let source = "int puts(char*); typedef struct { int kind; union { struct { short lo; short hi; }; int word; }; struct { int x; int y; }; } packet_t; int main(void) { packet_t packet; packet.word = 0; packet.lo = 11; packet.hi = 13; packet.x = 17; packet.y = 19; puts(\"stress-anon\"); return packet.lo + packet.hi + packet.x + packet.y + sizeof(packet_t); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_typeof_attribute_pointer_expression_matches_host_stdout_and_exit_code() {
    // given
    let name = "stress_typeof_attribute_pointer_expression";
    let source = "int puts(char*); typedef struct { int value; } item_t; int main(void) { item_t item; item_t *cursor = &item; item.value = 23; __typeof__(cursor->value + 5) total __attribute__((unused, aligned(16))) = cursor->value + 5; puts(\"stress-typeof\"); return total + sizeof(total); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_multifile_extern_function_pointer_state_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "stress_multifile_extern_function_pointer_state",
        files: &[
            OracleSourceFile {
                path: "api.h",
                source: "extern int state; int bump(int); int call_with_state(int (*fn)(int));\n",
            },
            OracleSourceFile {
                path: "state.c",
                source: "#include \"api.h\"\nint state = 5; int bump(int x) { state = state + x; return state; }\n",
            },
            OracleSourceFile {
                path: "call.c",
                source: "#include \"api.h\"\nint call_with_state(int (*fn)(int)) { return fn(state); }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "#include \"api.h\"\nint puts(char*); int main(void) { puts(\"stress-mf\"); return call_with_state(bump) + state; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn stress_static_inline_pointer_helper_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "stress_static_inline_pointer_helper_chain";
    let source = "int puts(char*); static inline int load_at(int *base, int index) { return *(base + index); } static inline int weighted(int *base, int index) { return load_at(base, index) * (index + 2); } int main(void) { int values[3] = { 3, 5, 7 }; puts(\"stress-inline\"); return weighted(values, 0) + weighted(values, 2); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_restrict_const_pointer_indexed_update_matches_host_stdout_and_exit_code() {
    // given
    let name = "stress_restrict_const_pointer_indexed_update";
    let source = "int puts(char*); void update(int n, int * restrict out, const int * restrict left, const int * restrict right) { int i; for (i = 0; i < n; i++) out[i] = left[n - i - 1] + right[i] * 2; } int main(void) { int left[4] = { 1, 2, 3, 4 }; int right[4] = { 5, 6, 7, 8 }; int out[4]; update(4, out, left, right); puts(\"stress-restrict\"); return out[0] + out[1] + out[2] + out[3]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_constant_expression_array_bound_mix_matches_host_stdout_and_exit_code() {
    // given
    let name = "stress_constant_expression_array_bound_mix";
    let source = "int puts(char*); enum { A = sizeof(int) * 3, B = (A << 1) - sizeof(short), C = B % 7, COUNT = C ? B / C : 1 }; int main(void) { int values[COUNT + 2]; puts(\"stress-constexpr\"); return sizeof(values) + COUNT; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_function_like_recursive_macro_ladder_matches_host_stdout_and_exit_code() {
    // given
    let name = "stress_function_like_recursive_macro_ladder";
    let source = "#define S0(x) (x)\n#define S1(x) S0((x) + 1)\n#define S2(x) S1((x) * 2)\n#define S3(x) S2((x) + 3)\n#define S4(x) S3((x) * 2)\n#define S5(x) S4((x) + 5)\n#define S6(x) S5((x) * 2)\n#define S7(x) S6((x) + 7)\n#define S8(x) S7((x) * 2)\nint puts(char*); int main(void) { puts(\"stress-macro\"); return S8(1); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_variadic_macro_repeated_va_args_match_host_stdout_and_exit_code() {
    // given
    let name = "stress_variadic_macro_repeated_va_args";
    let source = "#define SUM4(a, b, c, d) ((a) + (b) + (c) + (d))\n#define CALL4(fn, ...) fn(__VA_ARGS__)\n#define DOUBLE_CALL(fn, ...) (CALL4(fn, __VA_ARGS__) + CALL4(fn, __VA_ARGS__))\nint puts(char*); int main(void) { puts(\"stress-vaargs\"); return DOUBLE_CALL(SUM4, 1, 3, 5, 7); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_predefined_macros_relative_line_and_func_match_host_stdout_and_exit_code() {
    // given
    let name = "stress_predefined_macros_relative_line_and_func";
    let source = "#define LINE_VALUE() __LINE__\nint puts(char*);\nint helper(void) { return LINE_VALUE(); }\nint main(void) {\n    int first = LINE_VALUE();\n    int second = LINE_VALUE();\n    char *file = __FILE__;\n    puts(__func__);\n    return second == first + 1 && helper() < first && file[0] != 0 && __func__[0] == 'm' ? 0 : 1;\n}\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_nested_initializer_list_struct_arrays_match_host_stdout_and_exit_code() {
    // given
    let name = "stress_nested_initializer_list_struct_arrays";
    let source = "int puts(char*); typedef struct { int xy[2]; } point_t; typedef struct { point_t points[2]; int tail[2]; } shape_t; int main(void) { shape_t shape = { { { { 1, 2 } }, { { 3 } } }, { 5 } }; puts(\"stress-init\"); return shape.points[0].xy[1] + shape.points[1].xy[0] + shape.points[1].xy[1] + shape.tail[0] + shape.tail[1]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_ternary_side_effects_nested_selected_only_match_host_stdout_and_exit_code() {
    // given
    let name = "stress_ternary_side_effects_nested_selected_only";
    let source = "int puts(char*); int main(void) { int x = 0; int y = 10; int z = 20; int a = x++ ? (y += 100, y) : (z += x, z); int b = x ? (y += a, y) : (z += 1000, z); puts(\"stress-ternary\"); return x + y + z + a + b; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_sizeof_type_expression_pointer_and_array_match_host_stdout_and_exit_code() {
    // given
    let name = "stress_sizeof_type_expression_pointer_and_array";
    let source = "int puts(char*); int bump(int *p) { *p = *p + 77; return *p; } int main(void) { int x = 4; int values[6]; int *ptr = values; int total = sizeof(int[2]) + sizeof values + sizeof ptr + sizeof(bump(&x)); puts(\"stress-sizeof\"); return total + x; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_incompatible_pointer_cast_array_to_struct_matches_host_stdout_and_exit_code() {
    // given
    let name = "stress_incompatible_pointer_cast_array_to_struct";
    let source = "int puts(char*); typedef struct { short a; short b; } pair_t; int main(void) { int word = 0; pair_t *pair = (pair_t *)(void *)&word; pair->a = 21; pair->b = 22; puts(\"stress-ptrcast\"); return pair->a + pair->b + (word != 0); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_char_signedness_arithmetic_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "stress_char_signedness_arithmetic_chain";
    let source = "int puts(char*); int main(void) { char plain = (char)255; unsigned char wide = plain; signed char narrow = wide; int score = (plain < 0 ? 10 : 20) + (wide == 255 ? 3 : 5) + (narrow == -1 ? 7 : 9); puts(\"stress-char\"); return score; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_long_long_bitwise_arithmetic_mix_matches_host_stdout_and_exit_code() {
    // given
    let name = "stress_long_long_bitwise_arithmetic_mix";
    let source = "int puts(char*); int main(void) { long long value = (1LL << 48) + (7LL << 16) + 123LL; long long masked = value & 65535LL; long long high = value >> 40; puts(\"stress-ll\"); return masked == 123LL && high == 256LL ? (int)((value >> 16) & 255LL) : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_decimal_float_literal_suffix_mix_matches_host_stdout_and_exit_code() {
    // given
    let name = "stress_decimal_float_literal_suffix_mix";
    let source = "int puts(char*); int main(void) { double a = 1.25e2; double b = .5e1f; double c = 7.L; double d = 9e-1; puts(\"stress-float\"); return (int)(a + b + c + d); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn stress_hex_float_fraction_suffix_mix_matches_host_stdout_and_exit_code() {
    // given
    let name = "stress_hex_float_fraction_suffix_mix";
    let source = "int puts(char*); int main(void) { double a = 0x1.cp+4; double b = 0X1.8P+1f; double c = 0x1p-2; puts(\"stress-hexfloat\"); return (int)(a + b + c); }\n";

    // when/then
    assert_case(name, source);
}
