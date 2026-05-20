use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::parse_supported_translation_unit;

#[test]
fn compiler_accepts_many_argument_calls() {
    // given
    let source = r"void sink(int a, int b, int c, int d, int e, int f, int g, int h, int i, int j);
int main(void) {
    sink(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
    return 0;
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let aarch64_assembly =
        emit_assembly(&lowered, Target::Aarch64AppleDarwin).expect("aarch64 assembly should emit");
    let x86_64_assembly = emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu)
        .expect("x86_64 assembly should emit");

    // then
    assert!(aarch64_assembly.contains("\tbl _sink\n"));
    assert!(x86_64_assembly.contains("\tcall sink\n"));
}

#[test]
fn compiler_accepts_global_int_matrix_slice() {
    // given
    let source = r"int pars[4][10] =
{
    {0},
    {0,30,75,120,90,165,180,180,30,165},
    {0,90,90,90,120,90,360,240,30,170},
    {0,90,45,90,150,90,90,165,30,135}
};
int value(int episode, int map) {
    return pars[episode][map];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("pars:"));
    assert!(assembly.contains("value:"));
}

#[test]
fn compiler_accepts_global_pointer_subscript_initializer_slice() {
    // given
    let source = r"typedef int boolean;
boolean mousearray[4];
boolean* mousebuttons = &mousearray[1];
int main(void) {
    return mousebuttons[0];
}";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("mousebuttons:"));
    assert!(assembly.contains("\t.quad mousearray+4\n"));
}

#[test]
fn compiler_accepts_doom_libc_struct_local_slice() {
    // given
    let source = r"int sigaction(int sig, struct sigaction* act, struct sigaction* oact);
void handler(int ignore) { ignore = 0; }
int probe(void) {
    struct stat fileinfo;
    struct timeval tp;
    struct timezone tzp;
    struct sockaddr_in address;
    struct hostent* hostentry;
    struct itimerval value;
    struct itimerval ovalue;
    struct sigaction act;
    struct sigaction oact;
    int out;
    fileinfo.st_size = 4;
    tp.tv_sec = fileinfo.st_size;
    tp.tv_usec = 5;
    address.sin_family = 2;
    address.sin_addr.s_addr = 0;
    address.sin_port = 5029;
    value.it_interval.tv_sec = 0;
    value.it_interval.tv_usec = 1000;
    value.it_value.tv_sec = value.it_interval.tv_sec;
    value.it_value.tv_usec = value.it_interval.tv_usec;
    act.sa_handler = handler;
    act.sa_flags = 1;
    sigaction(14, &act, &oact);
    out = *(int *)hostentry->h_addr_list[0];
    return out + tp.tv_sec + tp.tv_usec + tzp.tz_minuteswest + ovalue.it_value.tv_sec;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("probe:"));
}

#[test]
fn compiler_accepts_doom_name8_union_slice() {
    // given
    let source = r"void strupr(char* s);
int check(char* name) {
    union {
        char s[9];
        int x[2];
    } name8;
    int v1;
    name8.s[8] = 0;
    strupr(name8.s);
    v1 = name8.x[0];
    return v1;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("check:"));
}

#[test]
fn compiler_accepts_x11_opaque_doom_video_slice() {
    // given
    let source = r"typedef unsigned char byte;
XEvent X_event;
XVisualInfo X_visualinfo;
XShmSegmentInfo X_shminfo;
XImage* image;
Visual* X_visual;
extern byte gammatable[5][256];
extern int usegamma;
unsigned exptable[256];
double exptable2[256*256];
Cursor createnullcursor(Display* display, Window root) {
    Pixmap cursormask;
    XGCValues xgc;
    GC gc;
    XColor dummycolour;
    Cursor cursor;
    xgc.function = 0;
    dummycolour.pixel = 0;
    dummycolour.red = 0;
    dummycolour.green = 0;
    dummycolour.blue = 0;
    dummycolour.flags = 4;
    return cursor;
}
void probe(Colormap cmap, byte *palette) {
    int rc;
    rc = X_event.xkey.keycode;
    rc = gammatable[usegamma][*palette++];
    rc = X_event.xbutton.state | X_event.xmotion.x | X_event.xexpose.count;
    if (X_visualinfo.class == 3 && X_visualinfo.depth == 8) rc = 1;
    X_visual = X_visualinfo.visual;
    X_shminfo.shmid = rc;
    X_shminfo.shmaddr = image->data;
    image->data = X_shminfo.shmaddr;
    rc = image->bytes_per_line * image->height;
}
void window(void) {
    XSetWindowAttributes attribs;
    XGCValues xgcvalues;
    attribs.event_mask = 1;
    attribs.colormap = 0;
    attribs.border_pixel = 0;
    xgcvalues.graphics_exposures = 0;
}
void shared(void) {
    struct shmid_ds shminfo;
    int key;
    int rc;
    rc = shmget((key_t) key, 64000, 0777);
    rc = shminfo.shm_nattch;
    rc = shminfo.shm_cpid;
    if (rc == shminfo.shm_perm.cuid) rc = shminfo.shm_segsz;
}
void I_Quit(int code);
void signals(void) {
    signal(2, (void (*)(int)) I_Quit);
}
void expand(void) {
    union {
        double d;
        unsigned u[2];
    } pixel;
    unsigned values[3];
    unsigned *ptrs[2];
    double* exp;
    double out;
    exp = exptable2;
    ptrs[0] = values;
    exptable[0] = 1;
    values[0] = 1;
    values[1] = values[0];
    pixel.u[0] = 1;
    pixel.u[1] = 2;
    *ptrs[0]++ = values[1];
    *exp++ = pixel.d;
    out = pixel.d;
}
int main(void) { return 0; }";

    // when
    let tokens = lex(source).expect("lexer should succeed");
    let program = parse_supported_translation_unit(&tokens).expect("translation unit should parse");
    let lowered = lower(&program).expect("ir lowering should succeed");
    let assembly =
        emit_assembly(&lowered, Target::X86_64UnknownLinuxGnu).expect("assembly should emit");

    // then
    assert!(assembly.contains("createnullcursor:"));
    assert!(assembly.contains("probe:"));
    assert!(assembly.contains("window:"));
    assert!(assembly.contains("shared:"));
    assert!(assembly.contains("signals:"));
    assert!(assembly.contains("expand:"));
}
