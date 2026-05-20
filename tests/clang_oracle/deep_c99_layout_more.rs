use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn nested_struct_array_padding_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_struct_array_padding",
        source: "typedef struct { char tag; int values[3]; } inner_t; typedef struct { char head; inner_t inner; char tail; } outer_t; int main(void) { return sizeof(inner_t) == 16 && sizeof(outer_t) == 24 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn nested_union_alignment_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_union_alignment",
        source: "typedef union { char bytes[8]; long long wide; } cell_t; typedef struct { char head; cell_t cell; int tail; } box_t; int main(void) { return sizeof(cell_t) == 8 && sizeof(box_t) == 24 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn struct_array_member_layout_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "struct_array_member_layout",
        source: "typedef struct { short a; char b; } pair_t; typedef struct { pair_t pairs[3]; int tail; } bag_t; int main(void) { return sizeof(pair_t) == 4 && sizeof(bag_t) == 16 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn zero_width_bitfield_layout_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "zero_width_bitfield_layout",
        source: "typedef struct { unsigned a:3; unsigned :0; unsigned b:5; } bits_t; int main(void) { return sizeof(bits_t) == 8 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn union_bitfield_storage_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "union_bitfield_storage",
        source: "typedef union { unsigned a:1; unsigned b:31; } bits_t; int main(void) { return sizeof(bits_t) == 4 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn nested_named_struct_member_access_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_named_struct_member_access",
        source: "typedef struct { int value; } inner_t; typedef struct { inner_t inner; int tail; } outer_t; int main(void) { outer_t item; item.inner.value = 3; item.tail = 4; return item.inner.value + item.tail == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn nested_named_union_member_access_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_named_union_member_access",
        source: "typedef struct { union { int value; char bytes[4]; } data; int tail; } packet_t; int main(void) { packet_t packet; packet.data.value = 0x12345678; packet.tail = 5; return packet.data.bytes[0] != 0 && packet.tail == 5 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn anonymous_union_member_access_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "anonymous_union_member_access",
        source: "typedef struct { int tag; union { int value; char bytes[4]; }; } packet_t; int main(void) { packet_t packet; packet.value = 0x12345678; return packet.bytes[0] != 0 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn anonymous_struct_member_access_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "anonymous_struct_member_access",
        source: "typedef struct { int prefix; struct { int x; int y; }; int tail; } point_t; int main(void) { point_t point; point.x = 2; point.y = 3; point.tail = 4; return point.x + point.y + point.tail == 9 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn local_anonymous_union_doom_shape_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "local_anonymous_union_doom_shape",
        source: "int main(void) { union { double d; unsigned u[2]; } pixel; pixel.u[0] = 7; pixel.u[1] = 9; return sizeof(pixel) == 8 && pixel.u[0] == 7 && pixel.u[1] == 9 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}
