use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn nested_struct_union_bit_field_layout_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_struct_union_bit_field_layout",
        source: "int puts(char*); typedef struct { unsigned a:3; union { unsigned b:5; unsigned c:7; } u; unsigned d:9; } outer_t; int main(void) { puts(\"bitfield-layout\"); return sizeof(outer_t) == 12 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn function_pointer_pointer_cast_chain_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "function_pointer_pointer_cast_chain",
        source: "int puts(char*); typedef int (*op_t)(int); typedef op_t (*factory_t)(int); int add1(int x) { return x + 1; } op_t pick(int flag) { if (flag) return add1; return 0; } int main(void) { factory_t factory; op_t op; factory = (factory_t)(void*)pick; op = ((op_t (*)(int))(void*)factory)(1); puts(\"fncast\"); return op(40) == 41 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn recursive_struct_self_referencing_pointers_match_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "recursive_struct_self_referencing_pointers",
        source: "int puts(char*); typedef struct node_s node_t; struct node_s { int value; node_t *next; }; int main(void) { node_t a; node_t b; a.value = 7; b.value = 9; a.next = &b; b.next = &a; puts(\"recursive\"); return a.next->next->value == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn multi_dimensional_vla_parameter_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "multi_dimensional_vla_parameter",
        source: "int puts(char*); int shape(int rows, int cols, int matrix[rows][cols]) { puts(\"vla-matrix\"); return rows == 2 && cols == 3 ? 0 : 1; } int main(void) { return shape(2, 3, 0); }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn bool_conversion_edges_match_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "bool_conversion_edges",
        source: "int puts(char*); int main(void) { _Bool zero = 0; _Bool neg = -5; _Bool nullp = (void*)0; puts(\"bool\"); return zero == 0 && neg == 1 && nullp == 0 && (_Bool)256 == 1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn enum_negative_and_large_values_match_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "enum_negative_and_large_values",
        source: "int puts(char*); int main(void) { enum weird { NEG = -3, BIG = 0x7fffffff, AFTER = NEG + 5 }; puts(\"enum\"); return NEG == -3 && BIG > 0 && AFTER == 2 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn duff_device_style_switch_fallthrough_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "duff_device_style_switch_fallthrough",
        source: "int puts(char*); int main(void) { int n = 0; switch (2) { case 1: n = n + 1; case 2: n = n + 2; default: n = n + 4; } puts(\"duff\"); return n == 6 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn goto_out_of_vla_scope_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "goto_out_of_vla_scope",
        source: "int puts(char*); int main(void) { int n = 1; { int a[n]; a[0] = 9; if (a[0] == 9) goto done; return 1; } done: puts(\"goto-vla\"); return 0; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn const_qualified_pointer_aliasing_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "const_qualified_pointer_aliasing",
        source: "int puts(char*); int main(void) { int x = 3; const int *cp = &x; int *p = (int*)cp; *p = *cp + 4; puts(\"constalias\"); return x == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn volatile_signal_like_global_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "volatile_signal_like_global",
        source: "int puts(char*); volatile int flag; void tick(void) { flag = flag + 1; } int main(void) { flag = 0; tick(); puts(\"volatile\"); return flag == 1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn macro_stringification_and_token_pasting_match_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "macro_stringification_and_token_pasting",
        source: "#define JOIN(a, b) a ## b\n#define STR(value) #value\nint puts(char*); int foobar(void) { return 7; } int main(void) { char text[] = STR(hello world); puts(text); return JOIN(foo, bar)() == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn trigraph_and_digraph_tokens_match_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "trigraph_and_digraph_tokens",
        source: "??=define ITEM(name) name\nint puts(char*); int main(void) <% int values<:2:> = <% 3, 4 %>; puts(\"tri-digraph\"); return ITEM(values)<:1:> == 4 ? 0 : 1; %>\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn wide_char_and_multibyte_literals_match_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "wide_char_and_multibyte_literals",
        source: "int puts(char*); int main(void) { int wc = L'A'; char text[] = \"multi\"; puts(text); return wc == 65 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn signed_integer_bitwise_operations_match_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "signed_integer_bitwise_operations",
        source: "int puts(char*); int main(void) { int x = -8; int y = (x >> 1) ^ -1; puts(\"bitwise\"); return y == 3 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}
