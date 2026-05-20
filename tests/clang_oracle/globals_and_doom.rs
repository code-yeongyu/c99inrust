use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn enum_global_initializer_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "enum_global_initializer",
        source: "typedef enum { shareware, registered, indetermined } GameMode_t; GameMode_t gamemode = indetermined; int main(void) { return gamemode == 2 ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn enum_arithmetic_initializer_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "enum_arithmetic_initializer",
        source: "typedef enum { INVULNTICS = (30*35) } powerduration_t; int main(void) { return INVULNTICS == 1050 ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn enum_additive_chain_initializer_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "enum_additive_chain_initializer",
        source: "typedef enum { BT_WEAPONMASK = (8+16+32) } buttoncode_t; int main(void) { return BT_WEAPONMASK == 56 ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn enum_mixed_precedence_initializer_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "enum_mixed_precedence_initializer",
        source: "typedef enum { MIXED = (8+16*32-4/2) } buttoncode_t; int main(void) { return MIXED == 518 ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn aggregate_global_initializer_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "aggregate_global_initializer_slice",
        source: r"typedef struct {
    unsigned char *sequence;
    unsigned char *p;
} cheatseq_t;
static unsigned char cheat_amap_seq[] = { 0xb2, 0x26, 0xff };
static cheatseq_t cheat_amap = { cheat_amap_seq, 0 };
int main(void) { return 42; }
",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn multi_declarator_local_int_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "multi_declarator_local_int_slice",
        source: "int main(void) { int dx, dy; dx = 40; dy = 2; return dx + dy; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn mixed_pointer_scalar_local_declaration_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "mixed_pointer_scalar_local_declaration_slice",
        source: "int main(void) { unsigned char *p, c; p = 0; c = 7; return c; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn m_random_global_array_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "m_random_global_array_slice",
        source: r"unsigned char rndtable[4] = { 3, 5, 7, 11 };
int rndindex = 0;
int prndindex = 0;
int P_Random(void) {
    prndindex = (prndindex + 1) & 0x3;
    return rndtable[prndindex];
}
int M_Random(void) {
    rndindex = (rndindex + 1) & 0x3;
    return rndtable[rndindex];
}
void M_ClearRandom(void) {
    rndindex = prndindex = 0;
}
int main(void) {
    int a = P_Random();
    int b = M_Random();
    M_ClearRandom();
    return a == 5 && b == 5 ? 0 : 1;
}
",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn global_int_array_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "global_int_array_slice",
        source: r"int columnofs[4];
int main(void) {
    columnofs[1] = 33;
    columnofs[2] = 9;
    return columnofs[1] - columnofs[2] - 24;
}
",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn initialized_global_int_array_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "initialized_global_int_array_slice",
        source: r"int fuzzoffset[4] = { 320, -320, (320), -(320) };
int main(void) {
    return fuzzoffset[0] + fuzzoffset[1] + fuzzoffset[2] + fuzzoffset[3];
}
",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn m_cheat_xlate_table_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "m_cheat_xlate_table_slice",
        source: r"static unsigned char cheat_xlate_table[256];
int main(void) {
    int i;
    for (i = 0; i < 4; i++) cheat_xlate_table[i] = i + 1;
    return cheat_xlate_table[(unsigned char)2];
}
",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn m_bbox_pointer_subscript_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "m_bbox_pointer_subscript_slice",
        source: r"enum { BOXTOP, BOXBOTTOM, BOXLEFT, BOXRIGHT };
void M_ClearBox(int *box) {
    box[BOXTOP] = box[BOXRIGHT] = -1;
    box[BOXBOTTOM] = box[BOXLEFT] = 10;
}
void M_AddToBox(int *box, int x, int y) {
    if (x < box[BOXLEFT])
        box[BOXLEFT] = x;
    else if (x > box[BOXRIGHT])
        box[BOXRIGHT] = x;
    if (y < box[BOXBOTTOM])
        box[BOXBOTTOM] = y;
    else if (y > box[BOXTOP])
        box[BOXTOP] = y;
}
int main(void) { return 0; }
",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn doom_member_access_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "doom_member_access_slice",
        source: r"typedef int fixed_t;
typedef struct { fixed_t x,y; } mpoint_t;
typedef struct { mpoint_t a,b; } mline_t;
typedef struct { fixed_t slp, islp; } islope_t;
void AM_getIslope(mline_t* ml, islope_t* is) {
    int dx, dy;
    dy = ml->a.y - ml->b.y;
    dx = ml->b.x - ml->a.x;
    is->islp = dx + dy;
}
int main(void) { return 0; }
",
    };
    assert_compile_run_matches_host(case);
}
