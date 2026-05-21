use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn doom_like_global_string_table_indexes_match_host_stdout_and_exit_code() {
    // given
    let name = "doom_like_global_string_table_indexes";
    let source = "int puts(char*); char *names[] = { \"M_EPI1\", \"M_EPI2\", \"M_EPI3\" }; int main(void) { int index = 1; puts(names[index]); return names[0][2] == 'E' && names[2][5] == '3' && sizeof(names) == 3 * sizeof(char*) ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn doom_like_fixed_point_mul_shift_matches_host_stdout_and_exit_code() {
    // given
    let name = "doom_like_fixed_point_mul_shift";
    let source = "int puts(char*); typedef int fixed_t; fixed_t FixedMul(fixed_t a, fixed_t b) { return (fixed_t)(((long long)a * (long long)b) >> 16); } int main(void) { fixed_t one = 1 << 16; fixed_t half = one >> 1; puts(\"fixed-mul\"); return FixedMul(one + half, one + half) == (2 * one + (one >> 2)) ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn doom_like_ticcmd_signed_char_fields_match_host_stdout_and_exit_code() {
    // given
    let name = "doom_like_ticcmd_signed_char_fields";
    let source = "int puts(char*); typedef struct { signed char forwardmove; signed char sidemove; unsigned char buttons; } ticcmd_t; int main(void) { ticcmd_t cmd; ticcmd_t copy; cmd.forwardmove = -50; cmd.sidemove = 24; cmd.buttons = 255; copy = cmd; puts(\"ticcmd\"); return copy.forwardmove == -50 && copy.sidemove == 24 && copy.buttons == 255 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn doom_like_event_ring_buffer_struct_copy_matches_host_stdout_and_exit_code() {
    // given
    let name = "doom_like_event_ring_buffer_struct_copy";
    let source = "int puts(char*); typedef struct { int type; int data1; int data2; int data3; } event_t; event_t events[4]; int head; void post(event_t *ev) { events[head] = *ev; head = (head + 1) & 3; } int main(void) { event_t ev; ev.type = 2; ev.data1 = 17; ev.data2 = 23; ev.data3 = 29; post(&ev); ev.data1 = 31; post(&ev); puts(\"event-ring\"); return events[0].data1 + events[1].data1 + head; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn doom_like_recursive_mobj_links_match_host_stdout_and_exit_code() {
    // given
    let name = "doom_like_recursive_mobj_links";
    let source = "int puts(char*); typedef struct mobj_s { int x; struct mobj_s *next; struct mobj_s *prev; } mobj_t; int main(void) { mobj_t a; mobj_t b; mobj_t c; a.x = 3; b.x = 5; c.x = 7; a.next = &b; b.next = &c; c.prev = &b; puts(\"mobj-links\"); return a.next->next->x + c.prev->x; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn doom_like_switch_state_table_via_local_function_pointers_match_host_stdout_and_exit_code() {
    // given
    let name = "doom_like_switch_state_table_via_local_function_pointers";
    let source = "int puts(char*); int idle(int tic) { return tic + 1; } int chase(int tic) { return tic + 3; } int attack(int tic) { return tic + 7; } int main(void) { int (*states[])(int) = { idle, chase, attack }; int state = 2; puts(\"state-table\"); return states[state](10) + states[state - 1](10); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn doom_like_drawseg_pointer_cursor_matches_host_stdout_and_exit_code() {
    // given
    let name = "doom_like_drawseg_pointer_cursor";
    let source = "int puts(char*); typedef struct { int x1; int x2; int scale1; } drawseg_t; drawseg_t drawsegs[3]; int main(void) { drawseg_t *ds; drawsegs[0].x1 = 10; drawsegs[1].x2 = 25; drawsegs[2].scale1 = 7; ds = drawsegs; ds++; puts(\"drawseg\"); return ds->x2 + (ds + 1)->scale1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn doom_like_multifile_extern_string_table_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "doom_like_multifile_extern_string_table",
        files: &[
            OracleSourceFile {
                path: "names.h",
                source: "extern char *sfx_names[3]; int pick_name(int index);\n",
            },
            OracleSourceFile {
                path: "names.c",
                source: "#include \"names.h\"\nchar *sfx_names[3] = { \"pistol\", \"switch\", \"telept\" }; int pick_name(int index) { return sfx_names[index][0] + sfx_names[index][5]; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "#include \"names.h\"\nint puts(char*); int main(void) { puts(sfx_names[1]); return pick_name(2) == 't' + 't' ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}
