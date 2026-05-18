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
- reports that full Doom compilation is still a future milestone

## Frontend Surface Gate

The current compiler can preprocess, lex, and surface-parse all official
`linuxdoom-1.10` C/header files with the upstream Linux build defines:

```bash
doom=/path/to/DOOM/linuxdoom-1.10
for file in "$doom"/*.[ch]; do
  cargo run --quiet -- parse-check -D NORMALUNIX -D LINUX -I "$doom" "$file"
done
```

This is a frontend milestone only. `parse-check` recognizes Doom-shaped
top-level declarations and function-definition boundaries, but it does not type
check, lower, compile, link, or run Doom yet.

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
as `static nexttic = 0`, are parsed and lowered as stack-backed locals. This is
only a compile-progress approximation; true persistent local static storage is
still future work.
Local integer arrays with initializer lists, such as
`static int litelevels[] = { ... }`, are accepted as stack integer storage, and
`sizeof(local_array)` reports the local array byte size for supported local
array bindings. Local anonymous enum declarations add their constants to the
current lowering context, and `register name = ...` is accepted as another
old-style implicit-int local declaration.
Known struct typedef locals such as `fpoint_t tmp;` and local `static` struct
objects such as `static fline_t fl;` are accepted as stack-backed objects, with
scalar member reads/writes and same-type struct assignment lowered as member
copies. This is a compile-progress approximation; true persistent local static
storage is still future work.
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
`cheatseq_t cheat_amap = { ... }`, are also emitted as zero-filled storage for
address-taking.
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

The current Doom compile scan reaches actual supported function bodies, but 41
of the 62 C files still fail before object generation. `am_map.c`,
`d_items.c`, `d_main.c`, `d_net.c`, `doomdef.c`, `doomstat.c`, `dstrings.c`,
`f_finale.c`, `i_main.c`, `m_argv.c`, `m_bbox.c`, `m_cheat.c`, `m_fixed.c`,
`m_random.c`, `m_swap.c`, `r_draw.c`, `r_plane.c`, `r_segs.c`, `r_sky.c`,
`st_lib.c`, and `tables.c` currently reach assembly generation.
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
The former local enum typedef blocker in `p_enemy.c` moved past
`dirtype_t d[3]`; `p_ceilng.c`, `p_doors.c`, `p_floor.c`, and `p_plats.c`
now report unsupported function parameters rather than enum typedef parse
punctuation errors.
The former enum typedef parameter blockers in those `p_*` files now move to
later function-pointer cast assignments such as
`ceiling->thinker.function.acp1 = (actionf_p1)T_MoveCeiling`.
Many remaining files are blocked by old-style function definitions,
function-pointer declarations, typed-pointer gaps, and unsupported expression
forms.
The current `f_wipe.c` blocker is the local static function-pointer array
`static int (*wipes[])(int, int, int) = { ... }`.
The former `r_draw.c` blockers have moved past `(unsigned)dc_x`, the first
`do { ... } while (...)` loops, prefix increment in `R_DrawFuzzColumn`, local
char-array string initializers in `R_FillBackScreen`, the `unsigned ofs`
parameter in `R_VideoErase`, global arrays `ylookup` and `columnofs`, the
`lighttable_t* dc_colormap` global, `fuzzoffset`, `screens`, `sprnames`, and
the local `patch_t* patch` declaration. `r_draw.c` now reaches assembly
generation.
Evidence is recorded in
`docs/qa/2026-05-18-doom-translation-unit.md`.

Repeat the compile-progress scan with:

```bash
cargo build
tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-compile-scan.txt
```

## Playability Gate

Future acceptance requires:

1. Compile `linuxdoom-1.10` with `c99inrust`.
2. Link the executable for a Linux/X11 environment.
3. Provide a legal IWAD path through `DOOMWADDIR`.
4. Run inside tmux without `tmux kill-server`.
5. Verify a window/title loop appears.
6. Start a map and verify keyboard input moves the player.

Until all six pass, this repository must not claim playable Doom support.
