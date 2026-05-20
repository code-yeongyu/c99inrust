use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn local_short_initializer_conversion_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_short_initializer_conversion_chain";
    let source = "int puts(char*); int main(void) { short neg = -1; unsigned short wide = neg; short wrapped = 65535; puts(\"short-init\"); return neg == -1 && wide == 65535 && wrapped == -1 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn mixed_char_short_initializer_wraps_match_host_stdout_and_exit_code() {
    // given
    let name = "mixed_char_short_initializer_wraps";
    let source = "int puts(char*); int main(void) { char c = 255; unsigned char uc = -2; short s = 32768; unsigned short us = -2; puts(\"narrow-init\"); return c == -1 && uc == 254 && s == -32768 && us == 65534 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn static_local_scalar_persists_across_calls_matches_host_stdout_and_exit_code() {
    // given
    let name = "static_local_scalar_persists_across_calls";
    let source = "int puts(char*); int next(void) { static int value = 3; value = value + 2; return value; } int main(void) { puts(\"static-counter\"); return next() == 5 && next() == 7 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn static_local_char_array_reads_match_host_stdout_and_exit_code() {
    // given
    let name = "static_local_char_array_reads";
    let source = "int puts(char*); int probe(int index) { static char keys[4] = { 'd', 'o', 'o', 'm' }; return keys[index]; } int main(void) { puts(\"static-keys\"); return probe(2) == 'o' && probe(3) == 'm' ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_include_typedef_macro_multifile_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "local_include_typedef_macro_multifile",
        files: &[
            OracleSourceFile {
                path: "defs.h",
                source: "typedef struct { int x; int y; } point_t;\n#define SCALE(value) ((value) * 3)\n",
            },
            OracleSourceFile {
                path: "state.c",
                source: "#include \"defs.h\"\npoint_t origin = { 2, 4 }; int scaled_y(void) { return SCALE(origin.y); }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "#include \"defs.h\"\nextern point_t origin; int puts(char*); int scaled_y(void); int main(void) { puts(\"include-mf\"); return origin.x == 2 && scaled_y() == 12 ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn extern_function_pointer_installed_across_units_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "extern_function_pointer_installed_across_units",
        files: &[
            OracleSourceFile {
                path: "hooks.c",
                source: "int add5(int value) { return value + 5; } int (*hook)(int); void install(void) { hook = add5; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "extern int (*hook)(int); void install(void); int puts(char*); int main(void) { install(); puts(\"hook-mf\"); return hook(7) == 12 ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn pointer_array_iteration_over_structs_matches_host_stdout_and_exit_code() {
    // given
    let name = "pointer_array_iteration_over_structs";
    let source = "int puts(char*); typedef struct { int x; int y; } node_t; int sum(node_t **items) { return items[0]->x + items[1]->y; } int main(void) { node_t a; node_t b; node_t *items[2]; a.x = 3; b.y = 9; items[0] = &a; items[1] = &b; puts(\"ptr-array\"); return sum(items) == 12 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_partial_nested_initializer_zero_fills_match_host_stdout_and_exit_code() {
    // given
    let name = "global_partial_nested_initializer_zero_fills";
    let source = "int puts(char*); typedef struct { int values[3]; } row_t; row_t rows[2] = { { { 1 } }, { { 2, 3 } } }; int main(void) { puts(\"partial-global\"); return rows[0].values[1] == 0 && rows[1].values[2] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn macro_defined_elif_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "macro_defined_elif_chain";
    let source = "#define FEATURE 1\n#if defined(FEATURE) && FEATURE\n#define PICK 17\n#elif defined(OTHER)\n#define PICK 23\n#else\n#define PICK 31\n#endif\nint puts(char*); int main(void) { puts(\"defined-elif\"); return PICK == 17 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn sizeof_parameter_array_decay_matches_host_stdout_and_exit_code() {
    // given
    let name = "sizeof_parameter_array_decay";
    let source = "int puts(char*); int probe(int values[4]) { return sizeof(values) == sizeof(int*) ? values[2] : 99; } int main(void) { int values[4] = { 1, 2, 7, 8 }; puts(\"sizeof-param\"); return probe(values) == 7 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn nested_struct_copy_preserves_array_field_matches_host_stdout_and_exit_code() {
    // given
    let name = "nested_struct_copy_preserves_array_field";
    let source = "int puts(char*); typedef struct { int cells[2]; } row_t; typedef struct { row_t rows[2]; int tag; } board_t; int main(void) { board_t a = { { { { 1, 2 } }, { { 3, 4 } } }, 5 }; board_t b; b = a; puts(\"copy-array\"); return b.rows[1].cells[0] == 3 && b.tag == 5 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn comma_expression_initializer_order_matches_host_stdout_and_exit_code() {
    // given
    let name = "comma_expression_initializer_order";
    let source = "int puts(char*); int main(void) { int x = 1; int y = (x = x + 2, x * 4); puts(\"comma-init\"); return x == 3 && y == 12 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
