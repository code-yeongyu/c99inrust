use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn struct_compound_literal_positional_member_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; int main(void) { puts(\"compound-struct-pos\"); return ((pair_t){ 3, 4 }).y; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "struct_compound_literal_positional_member",
        source,
    });
}

#[test]
fn struct_compound_literal_designated_member_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; int main(void) { puts(\"compound-struct-designated\"); return ((pair_t){ .y = 9, .x = 2 }).x + ((pair_t){ .y = 9, .x = 2 }).y; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "struct_compound_literal_designated_member",
        source,
    });
}

#[test]
fn struct_compound_literal_partial_zero_fill_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; int z; } triple_t; int main(void) { puts(\"compound-struct-zero\"); return ((triple_t){ .y = 7 }).x + ((triple_t){ .y = 7 }).y + ((triple_t){ .y = 7 }).z; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "struct_compound_literal_partial_zero_fill",
        source,
    });
}

#[test]
fn struct_compound_literal_sizeof_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { char tag; int value; } item_t; int main(void) { puts(\"compound-struct-sizeof\"); return sizeof((item_t){ 'a', 23 }); }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "struct_compound_literal_sizeof",
        source,
    });
}

#[test]
fn struct_compound_literal_member_in_call_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int add(int x, int y) { return x + y; } typedef struct { int x; int y; } pair_t; int main(void) { puts(\"compound-call-member\"); return add(((pair_t){ 5, 6 }).x, ((pair_t){ 5, 6 }).y); }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "struct_compound_literal_member_in_call",
        source,
    });
}

#[test]
fn struct_compound_literal_ternary_members_match_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; int main(void) { int choose = 0; puts(\"compound-ternary-member\"); return choose ? ((pair_t){ 1, 2 }).x : ((pair_t){ 3, 4 }).y; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "struct_compound_literal_ternary_members",
        source,
    });
}

#[test]
fn struct_compound_literal_signed_char_conversion_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { char c; } tiny_t; int main(void) { puts(\"compound-schar\"); return ((tiny_t){ 255 }).c < 0 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "struct_compound_literal_signed_char_conversion",
        source,
    });
}

#[test]
fn struct_compound_literal_unsigned_char_conversion_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { unsigned char c; } tiny_t; int main(void) { puts(\"compound-uchar\"); return ((tiny_t){ 300 }).c == 44 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "struct_compound_literal_unsigned_char_conversion",
        source,
    });
}

#[test]
fn struct_compound_literal_short_conversions_match_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { short s; unsigned short u; } tiny_t; int main(void) { puts(\"compound-short\"); return ((tiny_t){ 65535, -2 }).s == -1 && ((tiny_t){ 65535, -2 }).u == 65534 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "struct_compound_literal_short_conversions",
        source,
    });
}

#[test]
fn int_array_compound_literal_subscript_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { puts(\"compound-int-array\"); return (int[]){ 5, 6, 7 }[2]; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "int_array_compound_literal_subscript",
        source,
    });
}

#[test]
fn sized_int_array_compound_literal_zero_fill_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { puts(\"compound-int-array-zero\"); return (int[4]){ 1, 2 }[3]; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "sized_int_array_compound_literal_zero_fill",
        source,
    });
}

#[test]
fn char_array_compound_literal_subscript_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { puts(\"compound-char-array\"); return (char[]){ 'a', 'b', 0 }[1]; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "char_array_compound_literal_subscript",
        source,
    });
}

#[test]
fn array_compound_literal_sizeof_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { puts(\"compound-array-sizeof\"); return sizeof((int[3]){ 1, 2, 3 }); }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "array_compound_literal_sizeof",
        source,
    });
}

#[test]
fn array_compound_literal_in_call_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int pick(int value) { return value; } int main(void) { puts(\"compound-array-call\"); return pick((int[]){ 8, 9 }[0]); }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "array_compound_literal_in_call",
        source,
    });
}

#[test]
fn array_compound_literal_ternary_subscript_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { int choose = 1; puts(\"compound-array-ternary\"); return choose ? (int[]){ 10, 11 }[1] : (int[]){ 12, 13 }[0]; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "array_compound_literal_ternary_subscript",
        source,
    });
}

#[test]
fn flexible_array_member_int_heap_access_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); void *malloc(int); typedef struct { int length; int data[]; } packet_t; int main(void) { packet_t *p = (packet_t*)malloc(sizeof(packet_t) + 3 * sizeof(int)); p->length = 3; p->data[0] = 4; p->data[1] = 5; p->data[2] = 6; puts(\"flex-int-heap\"); return p->data[0] + p->data[2]; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "flexible_array_member_int_heap_access",
        source,
    });
}

#[test]
fn flexible_array_member_char_heap_access_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); void *malloc(int); typedef struct { int length; char bytes[]; } blob_t; int main(void) { blob_t *p = (blob_t*)malloc(sizeof(blob_t) + 4); p->bytes[0] = 'a'; p->bytes[1] = 'b'; p->bytes[2] = 'c'; p->bytes[3] = 0; puts(\"flex-char-heap\"); return p->bytes[1]; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "flexible_array_member_char_heap_access",
        source,
    });
}

#[test]
fn flexible_array_member_helper_function_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); void *malloc(int); typedef struct { int length; int data[]; } packet_t; int sum(packet_t *p) { int i; int total = 0; for (i = 0; i < p->length; i++) total = total + p->data[i]; return total; } int main(void) { packet_t *p = (packet_t*)malloc(sizeof(packet_t) + 2 * sizeof(int)); p->length = 2; p->data[0] = 8; p->data[1] = 9; puts(\"flex-helper\"); return sum(p); }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "flexible_array_member_helper_function",
        source,
    });
}

#[test]
fn flexible_array_member_sizeof_allocation_expression_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); void *malloc(int); typedef struct { char tag; int data[]; } packet_t; int main(void) { packet_t *p = (packet_t*)malloc(sizeof(*p) + 2 * sizeof(p->data[0])); p->data[0] = 12; p->data[1] = 13; puts(\"flex-sizeof-alloc\"); return p->data[0] + p->data[1]; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "flexible_array_member_sizeof_allocation_expression",
        source,
    });
}

#[test]
fn flexible_array_member_pointer_cursor_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); void *malloc(int); typedef struct { int length; int data[]; } packet_t; int main(void) { packet_t *p = (packet_t*)malloc(sizeof(packet_t) + 3 * sizeof(int)); int *cursor; p->length = 3; p->data[0] = 2; p->data[1] = 4; p->data[2] = 6; cursor = p->data + 1; puts(\"flex-cursor\"); return cursor[0] + cursor[1]; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "flexible_array_member_pointer_cursor",
        source,
    });
}

#[test]
fn flexible_array_member_unsigned_char_promotion_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); void *malloc(int); typedef struct { int length; unsigned char bytes[]; } blob_t; int main(void) { blob_t *p = (blob_t*)malloc(sizeof(blob_t) + 2); p->bytes[0] = 250; p->bytes[1] = 6; puts(\"flex-uchar\"); return p->bytes[0] + p->bytes[1]; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "flexible_array_member_unsigned_char_promotion",
        source,
    });
}
