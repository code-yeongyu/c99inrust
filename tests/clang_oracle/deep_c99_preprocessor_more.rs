use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn token_paste_identifier_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "token_paste_identifier",
        source: "#define CAT(a, b) a ## b\nint main(void) { int xy = 7; return CAT(x, y) == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn stringification_preserves_argument_text_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "stringification_preserves_argument_text",
        source: "#define STR(x) #x\nint main(void) { char *s = STR(hello world); return s[0] == 'h' && s[5] == ' ' && s[6] == 'w' ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn variadic_macro_operator_tail_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "variadic_macro_operator_tail",
        source: "#define APPLY(base, ...) ((base) __VA_ARGS__)\nint main(void) { return APPLY(4, + 3) == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn variadic_macro_forwards_call_arguments_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "variadic_macro_forwards_call_arguments",
        source: "#define CALL(fn, ...) fn(__VA_ARGS__)\nint add(int a, int b) { return a + b; } int main(void) { return CALL(add, 3, 4) == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn deep_recursive_macro_chain_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "deep_recursive_macro_chain",
        source: "#define M00 5\n#define M01 M00\n#define M02 M01\n#define M03 M02\n#define M04 M03\n#define M05 M04\n#define M06 M05\n#define M07 M06\n#define M08 M07\n#define M09 M08\n#define M10 M09\n#define M11 M10\n#define M12 M11\n#define M13 M12\n#define M14 M13\n#define M15 M14\n#define M16 M15\n#define M17 M16\n#define M18 M17\n#define M19 M18\n#define M20 M19\n#define M21 M20\n#define M22 M21\n#define M23 M22\n#define M24 M23\n#define M25 M24\n#define M26 M25\n#define M27 M26\n#define M28 M27\n#define M29 M28\n#define M30 M29\n#define M31 M30\n#define M32 M31\n#define M33 M32\n#define M34 M33\n#define M35 M34\n#define M36 M35\n#define M37 M36\n#define M38 M37\n#define M39 M38\n#define M40 M39\nint main(void) { return M40 == 5 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn predefined_line_macro_exact_values_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "predefined_line_macro_exact_values",
        source: "int main(void) {\nint first = __LINE__;\nint second = __LINE__;\nreturn first == 2 && second == 3 ? 0 : 1;\n}\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn predefined_file_macro_stdout_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "predefined_file_macro_stdout",
        source: "int puts(char*); int main(void) { puts(__FILE__); return __FILE__[0] != 0 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn predefined_func_macro_in_helper_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "predefined_func_macro_in_helper",
        source: "int puts(char*); int helper(void) { puts(__func__); return __func__[0] == 'h'; } int main(void) { return helper() ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn digraph_block_and_subscript_tokens_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "digraph_block_and_subscript_tokens",
        source: "int main(void) <% int values<:2:> = <% 3, 4 %>; return values<:1:> == 4 ? 0 : 1; %>\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn typeof_keyword_and_nested_attribute_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "typeof_keyword_and_nested_attribute",
        source: "int marker __attribute__((unused, aligned(8))); int main(void) { int x = 3; __typeof__(x) y = x + 4; return y == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}
