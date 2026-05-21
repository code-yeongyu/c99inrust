# Official Doom QA Target

The target source is the public id Software release:

- GitHub: https://github.com/id-Software/DOOM
- Historic archive: https://www.gamers.org/pub/idgames/idstuff/source/

The upstream README states the source release is Linux-only and still needs real
Doom game data. The Linux target links X11/Xext and a small set of platform
libraries.

## Audit

```bash
git clone https://github.com/id-Software/DOOM /tmp/DOOM
cargo run -- doom-audit /tmp/DOOM
```

Expected today:

- counts C and header files under `linuxdoom-1.10`
- confirms the Makefile exists
- reports whether the input has the expected 62-unit official source shape
- prints recorded compile/link/input/movement QA evidence with `recorded-*`
  prefixes, so it is not mistaken for a live smoke run

## Frontend Surface Gate

The current compiler can preprocess, lex, and surface-parse all official
`linuxdoom-1.10` C/header files with the upstream Linux build defines:

```bash
doom=/path/to/DOOM/linuxdoom-1.10
for file in "$doom"/*.[ch]; do
  cargo run --quiet -- parse-check -D NORMALUNIX -D LINUX -I "$doom" "$file"
done
```

This remains a useful frontend check. The stronger current gate is the full
compile/link/run smoke below.

## Compile Progress

`compile -S` now accepts full translation units enough to skip top-level
declarations and prototypes before attempting supported function definitions.
It also accepts `int` and `void` function return types, binds supported integer
parameters into ABI-backed local slots, emits terminal returns for `void`
functions that can fall through, treats supported scalar/typedef return
specifiers such as `fixed_t`, `boolean`, `char`, and `unsigned short` as the
current integer return ABI, emits signed integer expression slices for
`long long` casts, function call arguments, and `?:` conditionals, and now
covers the Doom `FixedDiv2` double-expression slice plus the Linux
`<values.h>` integer limit macros used by `doomtype.h`. It also emits the
global int and `unsigned char` table slice needed by `m_random.c`, including
right-associative chained assignment and global byte-array subscripts. It now
also lowers the Doom `m_bbox.c` pointer-parameter subscript slice, including
anonymous enum constants, pointer-width parameter spills, and int element
load/store for `box[index]`. It also records `extern int` and `extern`
pointer declarations as scalar global bindings without emitting definitions,
which lets the Doom `i_main.c` entry translation unit assign `myargc` and
`myargv`. Postfix `++` is supported as an `int` side-effect statement, covering
the `m_argv.c` `for (...; i++)` scan loop. The compiler also accepts
translation units that contain only ignorable internal static metadata, covering
`doomdef.c`, while still rejecting unsupported data-only globals. It now accepts
the `doomstat.c` enum-backed global state definitions after declaration-only
header `extern` arrays, including simple checked enum arithmetic such as
`(8+16+32)` with C operator precedence for the supported arithmetic operators.
Unparenthesized numeric scalar global initializers such as `200 - 32` are also
accepted, and the preprocessor supplies the Doom-era `IPPORT_USERRESERVED`
constant for `<netinet/in.h>`. Decimal fixed-point global initializers such as
`(.2*(1<<16))` are folded into integer storage for the `am_map.c` scale slice.
Unsupported brace-initialized aggregate globals are skipped before supported
function bodies, so Doom typedef structs such as `cheatseq_t` no longer get
misclassified as scalar integer globals. Local comma-separated integer
declarations such as `int dx, dy;` are accepted, as are local declaration
specifier sequences such as `unsigned char *p, c;` and `unsigned short *p;`.
Typed pointer parameters can
now drive scalar member loads and stores through nested Doom typedef structs,
covering the `AM_getIslope` shape `ml->a.y`, `ml->b.x`, and `is->islp`.
Compound scalar assignments such as `m_x += m_w/2` and `m_x -= m_w/2` are also
parsed and lowered through the existing assignment path. Local pointer
declarators, pointer casts, unary pointer dereference, pointer post-decrement
conditions, `sizeof(type)`, and address-of-subscript expressions are accepted
for the `f_wipe.c` pointer-walk slice. Unsigned integer casts such as
`(unsigned)x` and `(unsigned char)x` are now accepted through the current
integer ABI for Doom-shaped positive-value expression slices; full unsigned
conversion and wraparound semantics remain future work. Post-increment
expressions now produce their old value while updating direct `int` and pointer
lvalues, and empty statements are accepted for Doom loops such as
`while (*(p++) != 1);`. `do { ... } while (...)` statements now lower to a
body-first loop with a conditional back edge. Static unsigned-byte globals can
now be zero-filled and assigned through byte subscripts, covering tables such as
`cheat_xlate_table[256]`. Pointer-valued struct members can now be read and
assigned, covering `cheatseq_t` fields such as `cht->p` and `cht->sequence`.
Post-increment expressions on pointer struct members now produce the old field
value while updating the field.
Prefix increment and decrement expressions are parsed through the existing
assignment-expression path, covering scalar conditions such as `++fuzzpos`.
Local `char name[] = "literal"` and fixed-size `char namebuf[9]` declarations
are accepted as stack byte storage, and local array identifiers decay to stack
addresses for calls such as `sprintf(namebuf, ...)`.
Adjacent string literals are concatenated, and the preprocessor expands the
Doom macro builtins `__FILE__` and `__LINE__`.
`break` statements and minimal `switch`/`case`/`default` control flow now lower
through labels, including character-literal case values such as `case '-'`.
Local `static` scalar declarations, including old-style implicit-int forms such
as `static nexttic = 0`, are parsed and lowered to internal data symbols such
as `I_GetTime__static__basetime`, so their values persist across calls.
Local integer arrays with initializer lists, such as
`static int litelevels[] = { ... }`, are still accepted as stack integer
storage, and `sizeof(local_array)` reports the local array byte size for
supported local array bindings. Local anonymous enum declarations add their
constants to the current lowering context, and `register name = ...` is
accepted as another old-style implicit-int local declaration.
Known struct typedef locals such as `fpoint_t tmp;` and local `static` struct
objects such as `static fline_t fl;` are accepted as stack-backed objects, with
scalar member reads/writes and same-type struct assignment lowered as member
copies. Aggregate local static persistence is still future work.
`continue` statements are lowered for loops. Doom scalar typedef `angle_t` is
accepted as an integer type, and top-level integer declarator lists such as
`static fixed_t m_x, m_y;` emit one integer global per declarator.
File-scope pointers to struct typedefs, such as `static player_t *plr`, retain
their referent for member access. Pointer-valued struct fields also carry their
referent, covering nested forms such as `plr->mo->x` when the pointee layouts
are known. Struct field parsing accepts untracked typedef scalars and simple
array declarators, including scalar member-array subscripts such as
`plr->powers[pw_allmap]`.
Plain `unsigned` parameters are accepted as integer parameters.
The Doom typedef scalar set includes `lighttable_t`, enabling globals such as
`lighttable_t* dc_colormap`.
Global pointer arrays such as `byte* ylookup[MAXHEIGHT]` are emitted as
zero-filled pointer storage and support pointer element loads/stores.
Global int arrays such as `int columnofs[MAXWIDTH]` are emitted as zero-filled
integer storage and support int element loads/stores.
Global int arrays with integer initializer lists are emitted as `.long` data,
covering Doom tables such as `fuzzoffset`.
Extern global pointer arrays such as `extern byte* screens[5]` and
`extern char *sprnames[NUMSPRITES]` are accepted as bindings without emitting
storage.
Local pointers to typedef'd structs such as `patch_t* patch` are accepted in
function bodies that have already seen the struct typedef.
Top-level arrays of known struct typedefs with aggregate initializers are
accepted as zero-filled storage and decay to their global address for calls.
Global struct objects, including brace-initialized objects such as
`cheatseq_t cheat_amap = { cheat_amap_seq, 0 }`, now emit their scalar and
pointer field initializers, including pointer fields that decay from global
byte arrays.
Extern pointers to struct typedefs retain their referent, and struct objects
reached through pointer subscripts such as `lines[i].v1->x` are lowered through
dynamic `base + index * sizeof(struct)` addressing. Local typed struct pointers
retain their referent, typed pointer casts such as `(memblock_t*)p` carry their
pointee for `->` access, `sizeof(struct_typedef)` reports known layout size,
standard streams `stdin`/`stdout`/`stderr` are available as extern pointer
bindings, and `&nested.struct.member` lowers to a field address.
Extern declarations of known struct arrays can be merged with a later
definition that discovers the initializer length, covering
`extern weaponinfo_t weaponinfo[NUMWEAPONS]` followed by the `d_items.c`
definition.
The compiler now also accepts the declaration and expression shapes needed by
`d_main.c`: Doom enum typedef scalars such as `gamestate_t` and `skill_t`,
opaque `FILE*` locals, local pointer arrays, comma-expression `for`
initializers/posts, block-scope extern arrays, local `char name[23][8]`
matrices with row decay, global plain `char` arrays, and byte loads/stores
through `char*` pointer arithmetic and nested `char**` subscripts. The
preprocessor supplies the Doom-era libc constants `R_OK`, `SEEK_SET`,
`SEEK_CUR`, `SEEK_END`, and `NULL`.
The compiler now also accepts the declaration, control-flow, and struct-copy
shapes reached by `d_net.c`: plain `unsigned` locals, `goto` labels, integer
literal suffixes such as `0x12345678l`, struct arrays inside struct fields,
global struct object assignment from pointer dereference, and unsigned
32-bit mask immediates. This also moves `tables.c` to assembly generation.
File-scope pointer string initializers and pointer string-array initializers
are emitted with backing string data, covering `f_finale.c` text globals and
the data-only `dstrings.c` quit-message table. Extern pointer arrays now retain
their struct referent, and simple struct typedef aliases such as
`typedef post_t column_t;` keep the aliased layout for pointer casts and member
access.
The Doom-era `<values.h>` builtin now also supplies `MAXSHORT` and related
short/char limits, and global `short` arrays are accepted through the current
integer-array compile path, moving `r_segs.c` to assembly generation.
Global array bounds now accept integer initializer expressions such as
`320*64`, and pointer-returning function signatures lower return values through
the current scalar backend, moving `r_plane.c` to assembly generation.
Enum constants now feed global and local array lengths, global integer array
initializer values, and unsized enum-typed global arrays with aggregate
initializers. This moves `NUMSFX`, `NUMAMMO`, `NUMCARDS`, `sfx_pldeth`,
`mus_e3m4`, and `dirtype_t opposite[]` blockers to later parser/lowering
gaps, although the total scan count remains unchanged.
Enum typedef names introduced by `typedef enum { ... } name;` are also tracked
as scalar local declaration types, moving local arrays such as
`dirtype_t d[3]` and several `p_*` enum-parameter blockers to later failures.
Those enum typedef names are now accepted in function parameter lists as well,
moving `ceiling_e`, `vldoor_e`, `floor_e`, and related parameters to later
function-pointer cast assignment blockers.
Doom action-function pointer typedefs such as `actionf_p1` are now tracked as
pointer types, `typedef union { ... } actionf_t` is represented as a
pointer-sized aggregate layout, and tagged structs such as
`typedef struct line_s { ... } line_t` also register their tag name for
`struct line_s**` pointer referents. Pointer casts of function symbols lower to
symbol addresses, indirect calls through pointer-valued members emit real
indirect calls, prototype-only pointer-returning functions can drive `->`
member access, and extern struct objects can merge with their later
definitions. This moves the Doom thinker/action shapes in `p_ceilng.c`,
`p_doors.c`, `p_floor.c`, `p_plats.c`, `p_saveg.c`, `p_telept.c`, and
`p_tick.c` to assembly generation.
Bare function designators that name known functions now lower to symbol
addresses as callback arguments, moving the `P_BlockThingsIterator(...,
PIT_VileCheck)` shape in `p_enemy.c` to assembly generation and moving
`p_map.c` to its next `intercept_t` struct-typing blocker.
Named inner aggregate fields such as `union { ... } d;` now get nested struct
layouts, and top-level struct/union detection ignores nested aggregate keywords.
This moves `intercept_t` member chains such as `in->d.line` and `in->d.thing`
in `p_map.c` to assembly generation.
Parenthesized pointer member targets such as `(*link)->bprev`, raw
function-pointer parameters such as `boolean (*func)(line_t*)`, and direct
calls through those parameter bindings are accepted. Pointer referents now also
track `short` as a 2-byte element for pointer arithmetic, with signed halfword
loads and stores in the native backends. This moves `p_maputl.c` and `r_data.c`
to assembly generation.
Local function-pointer arrays with function-designator initializers, such as
`static int (*wipes[])(int, int, int) = { ... }`, are accepted as stack pointer
arrays. Block-scope function prototypes are skipped, and `(*funcptr)(...)`
calls lower through the existing indirect-call path. This moves `f_wipe.c` to
assembly generation.
The Doom-era `<fcntl.h>` builtin now supplies open flags such as `O_RDONLY`,
`O_WRONLY`, `O_CREAT`, `O_TRUNC`, and `O_BINARY`. Global `char` matrices such
as `char savegamestrings[10][24]` now decay to row pointers and support nested
byte subscripts, including string-list initializers. File-scope function
pointers and function-pointer struct fields are represented as pointer storage.
This moves `m_menu.c` to assembly generation.
Prototype-only function declarations now count as known function designators,
so assignments such as `colfunc = basecolfunc = R_DrawColumn` and
`colfunc = R_DrawTranslatedColumn` lower to symbol addresses even when the
function body lives in another translation unit. Typed `ptr +/- integer`
expressions now lower through scaled pointer-offset IR, including struct
pointer arithmetic used by `(vissprite_p-1)->next`. This moves `r_main.c` and
`r_things.c` to assembly generation.
Function parameter referents now preserve pointer depth for double-pointer
parameters such as `patch_t** font`, allowing `font[0]->height` and matching
`hu_textline_t.f` access through indexed pointer members. This moves
`hu_lib.c` to assembly generation.

An earlier Doom compile scan reached actual supported function bodies, but 11
of the 62 C files still fail before object generation. `am_map.c`,
`d_items.c`, `d_main.c`, `d_net.c`, `doomdef.c`, `doomstat.c`, `dstrings.c`,
`f_finale.c`, `f_wipe.c`, `g_game.c`, `hu_lib.c`, `i_main.c`, `i_net.c`,
`i_system.c`, `i_video.c`, `m_argv.c`, `m_bbox.c`, `m_cheat.c`, `m_fixed.c`,
`m_menu.c`, `m_misc.c`, `m_random.c`, `m_swap.c`, `p_ceilng.c`, `p_doors.c`,
`p_enemy.c`, `p_floor.c`, `p_inter.c`, `p_lights.c`, `p_map.c`,
`p_maputl.c`, `p_plats.c`, `p_pspr.c`, `p_saveg.c`, `p_sight.c`, `p_spec.c`,
`p_telept.c`, `p_tick.c`, `p_user.c`, `r_bsp.c`, `r_data.c`, `r_draw.c`,
`r_main.c`, `r_plane.c`, `r_segs.c`, `r_sky.c`, `r_things.c`, `st_lib.c`,
`tables.c`, `w_wad.c`, and `z_zone.c` currently reach assembly generation.
The former `am_map.c` blockers have moved past `AM_getIslope` member
expressions, `st_notify` local static aggregate, `namebuf` stack array, switch
statement, `case '-'` label, `litelevels` local integer array, local enum,
`register` implicit-int locals, local `fpoint_t tmp`, static local
`fline_t fl`, `continue`, `angle_t`, `static fixed_t m_x, m_y`, `static
player_t *plr`, typed pointer-subscript member access such as `lines[i].v1->x`,
`markpoints[i].x`, `m_paninc.x`, `sizeof(memblock_t)`, `(memblock_t*)` member
access, `stderr`, `plr->powers[pw_allmap]`, `&l.a.x`, and initialized vector
tables such as `cheat_player_arrow`.
The former `d_items.c` blocker moved past the conflicting `weaponinfo`
extern/definition pair. The former `d_main.c` blockers moved past enum typedef
locals, `FILE*`, `R_OK`, `SEEK_*`, response-file `char*` pointer arithmetic,
and `NULL`. The former `d_net.c` blockers moved past local plain `unsigned`,
`goto`, suffixed integer literals, struct-array field subscripts, and the
`reboundstore = *netbuffer` / `*netbuffer = reboundstore` struct copy shape.
The former `dstrings.c` and `f_finale.c` blockers moved past file-scope string
pointer data, `hu_font[c]->width`, `column_t*` casts, and `column->topdelta` /
`column->length` member access.
The former `r_segs.c` blockers moved past `MAXSHORT` and extern/global short
array declarations such as `ceilingclip[SCREENWIDTH]`.
The former `r_plane.c` blockers moved past expression-sized global arrays such
as `short openings[320*64]` and pointer-returning signatures such as
`visplane_t* R_FindPlane(...)`.
The former enum-constant blockers in `i_sound.c`, `p_inter.c`, `m_menu.c`,
`p_enemy.c`, `s_sound.c`, `sounds.c`, and `st_stuff.c` now reach later
unsupported expression, struct, pointer, or old-style declaration forms.
The former local enum typedef and callback designator blockers in `p_enemy.c`
moved past `dirtype_t d[3]` and `PIT_VileCheck`. The former enum typedef
parameter and action-function pointer blockers in `p_ceilng.c`, `p_doors.c`,
`p_floor.c`, and `p_plats.c` now reach assembly generation.
Many remaining files are blocked by old-style function definitions,
function-pointer declarations, typed-pointer gaps, unsupported expression
forms, and global/declaration gaps.
The former `p_maputl.c` blockers moved past parenthesized pointer member
assignment, raw function-pointer parameters, direct calls through function
pointer parameters, and `short*` pointer arithmetic over `blockmap`.
The former `r_data.c` blocker moved past `short*` and `unsigned short*`
pointer-declaration/referent tracking enough for this scan to reach assembly
generation.
The former `f_wipe.c` blocker moved past the local static function-pointer
array `static int (*wipes[])(int, int, int) = { ... }`, block-scope prototype
`void V_MarkRect(int, int, int, int);`, and `(*wipes[index])(...)` calls.
The former `m_menu.c` blockers moved past `<fcntl.h>` open flags,
`savegamestrings[slot][index]` global char-matrix access, file-scope callback
pointer `messageRoutine`, and `menuitem_t.routine` / `menu_t.routine`
function-pointer fields.
The former `r_main.c` and `r_things.c` blockers moved past prototype-only draw
function designators such as `R_DrawColumn` / `R_DrawTranslatedColumn`, and
the `r_things.c` sprite sort path now accepts scaled struct-pointer arithmetic
before `->` member access.
The former `hu_lib.c` blocker moved past double-pointer parameter referents for
`patch_t** font` and indexed member chains such as `font[0]->height`.
The former `i_video.c` blocker moved past Doom's X11 and SysV surface shapes:
X11 opaque scalar parameters, the XEvent/XImage/XColor/XVisualInfo member
chains used by the Linux video backend, local `struct shmid_ds` declarations,
function-pointer casts such as `(void (*)(int)) I_Quit`, extern byte matrices
such as `gammatable[usegamma][...]`, global unsigned and double stretch tables,
local anonymous union `pixel`, many-argument Xlib calls, and post-increment of
pointer elements such as `olineptrs[0]++`.
The former `g_game.c` and `r_bsp.c` blockers moved past nested global int
matrix initializers such as `pars[4][10]`, global pointer initializers such as
`boolean* mousebuttons = &mousearray[1]`, and `sizeof` on global struct
objects such as `wminfo`.
The former `r_draw.c` blockers have moved past `(unsigned)dc_x`, the first
`do { ... } while (...)` loops, prefix increment in `R_DrawFuzzColumn`, local
char-array string initializers in `R_FillBackScreen`, the `unsigned ofs`
parameter in `R_VideoErase`, global arrays `ylookup` and `columnofs`, the
`lighttable_t* dc_colormap` global, `fuzzoffset`, `screens`, `sprnames`, and
the local `patch_t* patch` declaration. `r_draw.c` now reaches assembly
generation.
The former `i_net.c`, `i_system.c`, `m_misc.c`, and `w_wad.c` blockers moved
past Doom's libc/network surface: `struct sockaddr_in`, `struct timeval`,
`struct timezone`, `struct stat`, and `struct hostent` locals, socket and ioctl
constants, the external `errno` scalar, variadic function definitions with
`va_list`, the `W_CheckNumForName` anonymous `name8` union, and local
`void*` declarations.

## Latest Compile Scan

`compile -S` now reaches assembly generation for all 62 official
`linuxdoom-1.10` C translation units with the upstream Linux build defines.
The latest slice covers the remaining Doom surface blockers, including
`struct sigaction` and the paired `struct itimerval` timer locals in
`i_sound.c`, pointer-name global initializers in `wi_stuff.c`, global pointer
and struct matrices, 2D struct field arrays, forward struct typedef/tag
linking, empty old-style parameter definitions, and the Doom-era signal,
timer, and `access(2)` constants.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779162206`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779162206.txt
ok=62
fail=0
```

The same compile surface was rechecked after the deeper C99 oracle expansion on
commit `deea66e`:

```text
scan=/tmp/c99inrust-doom-compile-scan-deea66e.txt
ok=62
fail=0
```

Evidence is recorded in `docs/qa/2026-05-18-doom-translation-unit.md` and
`docs/qa/2026-05-21-doom-compile-scan.md`.

Repeat the compile-progress scan with:

```bash
cargo build
tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-compile-scan.txt
```

CI also runs a no-IWAD link scan that compiles all 62 official units to
x86_64 Linux assembly and links them into a Linux/X11 ELF:

```bash
cargo build
tools/doom-link-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-link-scan
```

Latest local result:

```text
commit=d9836c4
scan=/tmp/c99inrust-doom-link-scan-d9836c4-mega-c
compile_ok=62 compile_fail=0
link_status=0
/out/linuxdoom-c99inrust: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, BuildID[sha1]=fb94a33afbca55422922016083b19176afb27bbf, for GNU/Linux 3.2.0, with debug_info, not stripped
```

Evidence is recorded in `docs/qa/2026-05-21-doom-link-scan.md`.

Latest CI recheck:

```text
commit=5a2f0bb
github_actions_run=26197824812
job=doom compile/link scan
compile_ok=62 compile_fail=0
link_status=0
conclusion=success
```

## Latest Link And Run Smoke

The 62 generated x86_64 Linux assembly files now link into a Linux/X11 ELF in
an amd64 Ubuntu container with system `gcc`, libc headers, X11, and Xext:

```text
tmux_session=c99inrust-doom-link-1779165229
scan=/tmp/c99inrust-doom-link-1779165229.txt
asm_dir=/tmp/c99inrust-doom-link-1779165229-asm
binary=/tmp/c99inrust-doom-link-1779165229-out/linuxdoom-c99inrust
```

The newer full smoke also runs the linked binary with a legal shareware IWAD
under an 8-bit Xvfb screen. The latest tmux run exited naturally without
`tmux kill-server`:

```text
tmux_session=c99inrust-doom-qa-1779255203
script=/tmp/c99inrust-doom-qa-1779255203.sh
out=/tmp/c99inrust-doom-qa-1779255203-out
compile_ok=62 compile_fail=0
link_status=0
run_status=124
```

`run_status=124` is the scripted 25 second timeout. The run log reached Doom
startup, WAD loading, renderer initialization, play-loop setup, status bar
initialization, MIT-SHM setup, and shared-memory allocation, with no
`Segmentation` or Doom `Error:` line.

Evidence is recorded in `docs/qa/2026-05-19-doom-link.md`.

Repeat the current smoke with:

```bash
cargo build
tools/doom-smoke.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-smoke
```

## Latest Visible Window And Input Smoke

The input smoke compiles and links the same 62 translation units, starts Xvfb
manually, verifies Doom has a viewable `320x200` X11 child window, dispatches
`Return Up Up Left Right` through `xdotool`, and waits for the binary to survive
until the scripted timeout:

```text
tmux_session=c99inrust-doom-input-1779266316
out=/tmp/c99inrust-doom-input-1779266316-out
compile_ok=62 compile_fail=0
link_status=0
display_status=0
window_status=0
window_id=0x400002
input_status=0
tail_status=124
run_status=124
tmux_command_status=0
```

This run also validates the fix for global `cheatseq_t` struct object
initializers. Before the fix, `cheat_god` and related globals were emitted as
zeroed storage, and Up-arrow input crashed in `cht_CheckCheat` while reading a
null `sequence` pointer. The repaired assembly now emits data such as
`.quad cheat_god_seq`, and 2D byte-array row initializers such as
`cheat_powerup_seq[6]` use row-sized offsets like `+60`.

Evidence is recorded in `docs/qa/2026-05-20-doom-input-smoke.md`.

Repeat the input smoke with:

```bash
cargo build
tools/doom-input-smoke.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-input-smoke
```

## Latest Movement Smoke

The movement smoke compiles and links the same 62 translation units, starts
Xvfb manually, verifies the viewable Doom window, dispatches `Up`, and reads
live player state from `/proc/<pid>/mem`. It fails unless the player mobj stays
in the thinker list with `P_MobjThinker` and the player coordinates change:

```text
commit=2eb3f12
out=/tmp/c99inrust-doom-movement-d9836c4-mega-c
compile_ok=62 compile_fail=0
link_status=0
display_status=0
window_status=0
window_id=0x400002
input_status=0
movement_status=0
tail_status=124
run_status=124
before 5 5 1 1 1 69206016 -236978176 0 0 0 -1
up_11 117 117 1 1 1 69194790 -209582386 -107 261602 25 1
```

An older movement smoke was rerun inside tmux session
`c99inrust-doom-movement-1779270567`, with `tmux_command_status=0`,
`movement_status=0`, and no `tmux kill-server`.

This run validates the fix for Doom `state_t states[]` initializers. Before the
fix, `states` was emitted as `.zero 54152`; movement switched the player to a
zeroed run state and removed the player mobj from the thinker list. The repaired
assembly emits function-pointer fields such as `.quad A_Pain`.

Evidence is recorded in `docs/qa/2026-05-20-doom-movement-smoke.md`.

Repeat the movement smoke with:

```bash
cargo build
tools/doom-movement-smoke.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-movement-smoke
```

## Manual Play Harness

`tools/doom-manual-play.sh` builds the same 62-unit Linux/X11 Doom executable
and can either stop after linking or launch the binary against a host X11
display for direct keyboard control.

Headless build-only proof:

```bash
cargo build
DOOM_MANUAL_RUN=0 tools/doom-manual-play.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-manual-play
```

The latest manual harness build-only recheck on commit `20120f6` produced:

```text
out=/tmp/c99inrust-doom-manual-20120f6-transcript
compile_ok=62 compile_fail=0
link_status=0
binary=/tmp/c99inrust-doom-manual-20120f6-transcript/linuxdoom-c99inrust
manual_run=skipped
reason=DOOM_MANUAL_RUN=0
transcript=/tmp/c99inrust-doom-manual-20120f6-transcript/manual-transcript.txt
```

This confirms the current manual harness still builds and links the official
Doom binary and writes a transcript scaffold. A host X11 display is still
required for a human-visible interactive manual playthrough.

Interactive visible-session run:

```bash
cargo build
tools/doom-manual-play.sh /tmp/c99inrust-doom-src /path/to/doom1.wad /tmp/c99inrust-doom-manual-play -- -warp 1 1 -nosound
```

Evidence and tmux instructions are recorded in
`docs/qa/2026-05-20-doom-manual-play.md`.

## Playability Gate

The current smokes prove visible-window startup, scripted keyboard delivery,
keyboard-driven player movement, and survival under Xvfb. Full
human-verified playability still requires:

1. Compile `linuxdoom-1.10` with `c99inrust`.
2. Link the executable for a Linux/X11 environment.
3. Provide a legal IWAD path.
4. Run inside tmux without `tmux kill-server`.
5. Verify the process survives startup and map load without segfault.
6. Verify a visible window/title loop appears.
7. Dispatch keyboard input to the window without crashing.
8. Start a map and verify keyboard input moves the player.

Items 1 through 8 are covered by the latest Xvfb smokes. A human visible-session
playthrough harness is now available, but a completed transcript remains outside
the automated smoke evidence.
