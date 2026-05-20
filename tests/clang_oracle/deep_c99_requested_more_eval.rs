use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn sixty_four_step_macro_chain_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "sixty_four_step_macro_chain",
        source: "#define M00 13\n#define M01 M00\n#define M02 M01\n#define M03 M02\n#define M04 M03\n#define M05 M04\n#define M06 M05\n#define M07 M06\n#define M08 M07\n#define M09 M08\n#define M10 M09\n#define M11 M10\n#define M12 M11\n#define M13 M12\n#define M14 M13\n#define M15 M14\n#define M16 M15\n#define M17 M16\n#define M18 M17\n#define M19 M18\n#define M20 M19\n#define M21 M20\n#define M22 M21\n#define M23 M22\n#define M24 M23\n#define M25 M24\n#define M26 M25\n#define M27 M26\n#define M28 M27\n#define M29 M28\n#define M30 M29\n#define M31 M30\n#define M32 M31\n#define M33 M32\n#define M34 M33\n#define M35 M34\n#define M36 M35\n#define M37 M36\n#define M38 M37\n#define M39 M38\n#define M40 M39\n#define M41 M40\n#define M42 M41\n#define M43 M42\n#define M44 M43\n#define M45 M44\n#define M46 M45\n#define M47 M46\n#define M48 M47\n#define M49 M48\n#define M50 M49\n#define M51 M50\n#define M52 M51\n#define M53 M52\n#define M54 M53\n#define M55 M54\n#define M56 M55\n#define M57 M56\n#define M58 M57\n#define M59 M58\n#define M60 M59\n#define M61 M60\n#define M62 M61\n#define M63 M62\n#define M64 M63\nint main(void) { return M64 == 13 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn nested_variadic_macro_va_args_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_variadic_macro_va_args",
        source: "#define SUM3(a, b, c) ((a) + (b) + (c))\n#define WRAP(first, ...) SUM3(first, __VA_ARGS__)\nint main(void) { return WRAP(2, 3, 4) == 9 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn predefined_line_inside_macro_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "predefined_line_inside_macro",
        source: "#define HERE __LINE__\nint main(void) {\nint first = HERE;\nint second = HERE;\nreturn first == 3 && second == 4 ? 0 : 1;\n}\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn ternary_nested_side_effects_sequence_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "ternary_nested_side_effects_sequence",
        source: "int main(void) { int x = 0; int y = (++x ? x++ : ++x); int z = (x == 2 ? ++x : x++); return x == 3 && y == 1 && z == 3 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn sizeof_dereferenced_null_expression_is_not_evaluated_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "sizeof_dereferenced_null_expression_is_not_evaluated",
        source: "int main(void) { int x = 1; return sizeof(*(long long*)0) == sizeof(long long) && sizeof(x++) == sizeof(int) && x == 1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn incompatible_char_pointer_cast_updates_int_object_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "incompatible_char_pointer_cast_updates_int_object",
        source: "int main(void) { int value = 0; unsigned char *bytes = (unsigned char*)&value; bytes[0] = 0x7f; return value != 0 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn char_signedness_conversion_chain_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "char_signedness_conversion_chain",
        source: "int main(void) { signed char s = -1; char c = s; unsigned char u = c; return (c < 0) == ((char)-1 < 0) && u == 255 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn long_long_struct_field_compound_assignment_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "long_long_struct_field_compound_assignment",
        source: "typedef struct { long long value; } cell_t; int main(void) { cell_t cell; cell.value = 0x100000000LL; cell.value += 7LL; return (int)(cell.value >> 32) == 1 && (int)cell.value == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn decimal_float_exponent_literals_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "decimal_float_exponent_literals",
        source: "int main(void) { double a = 1.25e2; double b = 7.5e-1; return (int)(a + b) == 125 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn hex_float_negative_exponent_literals_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "hex_float_negative_exponent_literals",
        source: "int main(void) { double a = 0x1.fp+2; double b = 0x1p-1; return (int)(a + b) == 8 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}
