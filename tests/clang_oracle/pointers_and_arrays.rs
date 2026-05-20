use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn local_pointer_declaration_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "local_pointer_declaration",
        source: "int Z_Malloc(int size, int tag, void *user) { return 0; } void Z_Free(void *p) { return; } int main(void) { short *dest; dest = (short*) Z_Malloc(8, 1, 0); Z_Free(dest); return dest == 0; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn local_char_array_string_initializer_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "local_char_array_string_initializer",
        source: "int main(void) { char name1[] = \"FLOOR7_2\"; char *name; name = name1; return name ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn pointer_dereference_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "pointer_dereference_slice",
        source: "int read_and_bump(int *p) { int value; value = *p; p++; return value; } int main(void) { return 0; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn sizeof_type_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "sizeof_type_slice",
        source: "int Z_Malloc(int size, int tag, void *user) { return 0; } int main(void) { int *y; y = (int*) Z_Malloc(4 * sizeof(int), 1, 0); return sizeof(int) == 4 ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn address_of_subscript_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "address_of_subscript_slice",
        source: "int address_of_subscript(int *p, int i) { int *q; q = &p[i]; return 0; } int main(void) { return 0; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn global_pointer_array_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "global_pointer_array_slice",
        source: r"int* ylookup[4];
int main(void) {
    int* p;
    p = 0;
    ylookup[2] = p;
    return ylookup[2] ? 1 : 0;
}
",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn extern_global_pointer_array_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "extern_global_pointer_array_slice",
        source: r"typedef unsigned char byte;
extern byte* screens[2];
byte* screens[2];
int main(void) {
    screens[0] = 0;
    return screens[0] ? 1 : 0;
}
",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn lighttable_pointer_global_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "lighttable_pointer_global_slice",
        source: r"typedef unsigned char byte;
typedef byte lighttable_t;
lighttable_t* dc_colormap;
int main(void) {
    dc_colormap = 0;
    return dc_colormap ? 1 : 0;
}
",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn i_main_global_pointer_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "i_main_global_pointer_slice",
        source: r"extern int myargc;
extern char **myargv;
int myargc;
char **myargv;
void D_DoomMain(void) { return; }
int main(int argc, char **argv) {
    myargc = argc;
    myargv = argv;
    D_DoomMain();
    if (myargv != argv)
        return 2;
    return myargc == argc ? 0 : 1;
}
",
    };
    assert_compile_run_matches_host(case);
}
