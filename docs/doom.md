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
Local `char name[] = "literal"` declarations are accepted by lowering them as
pointer locals initialized from string-literal storage.
Plain `unsigned` parameters are accepted as integer parameters.
Global pointer arrays such as `byte* ylookup[MAXHEIGHT]` are emitted as
zero-filled pointer storage and support pointer element loads/stores.
Pointer returns remain unsupported.

The current Doom compile scan reaches actual supported function bodies, but all
but ten of the 62 C files still fail before object generation. `doomdef.c`,
`doomstat.c`, `i_main.c`, `m_argv.c`, `m_bbox.c`, `m_cheat.c`, `m_fixed.c`,
`m_random.c`, `m_swap.c`, and `r_sky.c` currently reach assembly generation.
The current `am_map.c` blocker is the local static aggregate declaration
`static event_t st_notify = { ... }` in `AM_initVariables`, not the earlier
`AM_getIslope` member expressions.
The current `f_wipe.c` blocker is the local static function-pointer array
`static int (*wipes[])(int, int, int) = { ... }`.
The current `r_draw.c` blocker has moved past `(unsigned)dc_x`, the first
`do { ... } while (...)` loops, prefix increment in `R_DrawFuzzColumn`, and the
local char-array string initializers in `R_FillBackScreen`, and the
`unsigned ofs` parameter in `R_VideoErase`. It is now the global int-array
declaration/use of `columnofs`.
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
