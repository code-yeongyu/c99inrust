# Doom Translation Unit Compile Progress

Date: 2026-05-18
Public Doom checkout: `id-Software/DOOM` at `a77dfb9`

## Baseline

The public Doom checkout contains 62 C files and 62 header files under
`linuxdoom-1.10`.

```text
cargo run --quiet -- doom-audit /tmp/DOOM
official-doom-root=/tmp/DOOM
linuxdoom-c-files=62
linuxdoom-h-files=62
linuxdoom-makefile=true
status=audited language surface only; full Doom compilation is a future milestone
```

The frontend surface gate still passes across all 124 C/header files:

```text
parse_check_files=124
```

## Compile Scan Before This Slice

Before the translation-unit compile slice, all 62 C files failed before reaching
any function body because `compile -S` expected every token stream to begin with
an `int` function definition.

Representative failure:

```text
FAIL am_map.c
  error: 24:1: expected keyword Int
```

## Compile Scan After This Slice

`compile -S` now skips top-level declarations and prototypes, then compiles only
supported executable function definitions. This moves Doom compilation past
file-level declarations and into actual function signatures or bodies.

Current scan:

```text
scan=/tmp/c99inrust-doom-compile-scan-after-tu.txt
fail=62
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7137:1: unsupported function definition: AM_getIslope
FAIL i_main.c
  error: 579:1: unsupported function definition: main
FAIL d_net.c
  error: 4126:13: expected expression
FAIL m_random.c
  error: 60:20: expected punctuator ;
```

## Translation-Unit Slice Status

This was forward progress toward the Doom compile goal, not a playable Doom
claim. At this point, the next required compiler work was Doom function
signatures and bodies:

```text
void and non-int return types
parameter lists
pointers and arrays
static function linkage
function call arguments
broader expression and statement coverage
```

## Compile Scan After Signature Slice

`compile -S` now accepts supported Doom function signatures with `int` and
`void` returns, skips parameter-list tokens, accepts value-less `return;` in
`void` functions, rejects value-less `return;` in `int` functions, rejects
valued returns in `void` functions, and emits a terminal `return;` for `void`
functions whose body can fall through.

Regression coverage added:

```text
compiler_emits_void_functions_with_value_less_return
compiler_adds_terminal_return_to_void_functions_that_can_fall_through
compiler_rejects_value_less_return_from_int_functions
compiler_rejects_value_return_from_void_functions
compiler_accepts_parameter_list_signatures_when_body_does_not_use_parameters
void_function_slice_matches_host_c_compiler_exit_code
parameter_list_signature_slice_matches_host_c_compiler_exit_code
```

Frontend surface gate after this slice:

```text
scan=/tmp/c99inrust-doom-parse-check-after-signature.txt
parse_check_files=124
fail=0
```

Current compile scan:

```text
scan=/tmp/c99inrust-doom-compile-scan-after-signature.txt
ok=0
fail=62
```

Manual CLI QA was run inside tmux session `c99sigqa` with
`target/debug/c99inrust`:

```text
tmux_session=c99sigqa
void_explicit_exit=42
params_exit=42
void_fallthrough_exit=42
void_fallthrough_terminal_ret=PASS
error: int function must return a value
error: void function cannot return a value
manual_qa=PASS
```

Local Rust/slop gate for the signature slice:

```text
fmt: PASS
strict clippy: PASS, no warnings
rust-programmer no-excuse: PASS for 5 changed files
LSP diagnostics: PASS, 0 diagnostics
cargo test --all-targets --all-features: PASS, 36 tests
cargo nextest run --all-targets --all-features: PASS, 36 tests
cargo machete: PASS
cargo deny check: PASS
cargo audit: PASS
unsafe/miri: N/A; unsafe code is forbidden in Cargo.toml and src/lib.rs
remove-ai-slops: PASS for this slice; no debug leftovers, warning suppressions,
dead code, or needless behavior-changing cleanup found
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7142:11: expected punctuator ;
FAIL i_main.c
  error: 587:16: expected punctuator =
FAIL i_net.c
  error: 3889:17: expected punctuator )
FAIL m_fixed.c
  error: 429:1: unsupported function definition: FixedMul
FAIL r_sky.c
  error: assignment to undeclared local: skytexturemid
```

The remaining compile blockers have moved beyond the initial `void main(...)`
and parameter-list signature failures into C type/declarator and body coverage:

```text
typedef-backed return types such as fixed_t and boolean
local declarations with non-int Doom typedefs and pointer declarators
multiple declarators in one statement
function call arguments
pointer/member/subscript expressions
global variable storage and lookup
```

This is still not a playable Doom claim. Full success requires compiling all
translation units, linking the Doom executable, and manually running a playable
public Doom target.

## Compile Scan After Scalar Return Slice

`compile -S` now accepts scalar and typedef-backed return specifiers before the
function name. This covers Doom-style split-line signatures such as
`fixed_t\nFixedMul\n(...)`, boolean-like typedef returns, `char`, and
`unsigned short`. Pointer returns such as `char *name(void)` are still rejected
instead of being lowered through the integer ABI.

Regression coverage added:

```text
compiler_accepts_typedef_return_signatures
compiler_accepts_split_line_typedef_return_signatures
compiler_accepts_unsigned_scalar_return_signatures
compiler_rejects_pointer_return_signatures
typedef_return_signature_slice_matches_host_c_compiler_exit_code
unsigned_return_signature_slice_matches_host_c_compiler_exit_code
```

Current compile scan:

```text
scan=/tmp/c99inrust-doom-compile-scan-after-scalar-signature.txt
ok=0
fail=62
```

Manual CLI QA was run inside tmux session `c99scalarqa` with
`target/debug/c99inrust`:

```text
tmux_session=c99scalarqa
fixed_exit=42
unsigned_exit=42
error: 1:1: unsupported function definition: name
manual_qa=PASS
```

Local Rust/slop gate for the scalar return slice:

```text
fmt: PASS
strict clippy: PASS, no warnings
rust-programmer no-excuse: PASS for 3 changed files
LSP diagnostics: PASS, 0 diagnostics
cargo test --all-targets --all-features: PASS, 42 tests
cargo nextest run --all-targets --all-features: PASS, 42 tests
cargo machete: PASS
cargo deny check: PASS
cargo audit: PASS
unsafe/miri: N/A; unsafe code is forbidden in Cargo.toml and src/lib.rs
remove-ai-slops: PASS for this slice; no debug leftovers, warning suppressions,
dead code, or needless behavior-changing cleanup found
```

Representative moved failures:

```text
FAIL hu_stuff.c
  before: unsupported function definition: ForeignTranslation
  after:  5601:21: expected punctuator ;
FAIL m_fixed.c
  before: unsupported function definition: FixedMul
  after:  434:14: expected expression
FAIL m_swap.c
  before: unsupported function definition: SwapSHORT
  after:  unknown local: x
FAIL p_maputl.c
  before: unsupported function definition: P_AproxDistance
  after:  5456:14: expected punctuator )
```

The next high-value blocker is parameter binding plus ABI prologue stores.
`m_swap.c` now reaches `SwapSHORT`'s body and fails because parameter `x` is not
registered as a local yet.

## Compile Scan After Parameter Binding Slice

Function parameter names are now captured from supported signatures, registered
as the first local slots, and initialized from the current integer ABI registers
in each function prologue.

Regression coverage added:

```text
compiler_binds_parameters_as_local_slots_on_aarch64
compiler_binds_parameters_as_local_slots_on_x86_64
parameter_binding_slice_matches_host_c_compiler_exit_code
```

Current compile scan:

```text
scan=/tmp/c99inrust-doom-compile-scan-after-parameter-binding.txt
ok=1
fail=61
OK m_swap.c
```

Manual CLI QA was run inside tmux session `c99paramqa` with
`target/debug/c99inrust`:

```text
tmux_session=c99paramqa
argc_exit=1
	str w0, [sp, #0]
parameter_store=PASS
manual_qa=PASS
```

Local Rust/slop gate for the parameter binding slice:

```text
fmt: PASS
strict clippy: PASS, no warnings
rust-programmer no-excuse: PASS for 5 changed files
LSP diagnostics: PASS, 0 diagnostics
cargo test --all-targets --all-features: PASS, 45 tests
cargo nextest run --all-targets --all-features: PASS, 45 tests
cargo machete: PASS
cargo deny check: PASS
cargo audit: PASS
unsafe/miri: N/A; unsafe code is forbidden in Cargo.toml and src/lib.rs
remove-ai-slops: PASS for this slice; no debug leftovers, warning suppressions,
dead code, or needless behavior-changing cleanup found
```

Representative next blockers:

```text
FAIL m_fixed.c
  error: 434:14: expected expression
FAIL p_maputl.c
  error: 5456:14: expected punctuator )
FAIL r_sky.c
  error: assignment to undeclared local: skytexturemid
```

The next high-value blockers are cast expressions, call arguments, and broader
declarator parsing. `m_swap.c` is the first public Doom C translation unit to
reach assembly generation in this workspace baseline.

## Compile Scan After Expression Slice

The compiler now parses/lower/emits three C expression forms reached by public
Doom bodies:

```text
signed long long casts with 64-bit intermediates
integer function call arguments in ABI registers
conditional expressions with branch-shaped codegen
```

Regression coverage added:

```text
compiler_emits_signed_long_long_cast_intermediates
signed_long_long_cast_slice_matches_host_c_compiler_exit_code
compiler_emits_integer_function_call_arguments
function_call_argument_slice_matches_host_c_compiler_exit_code
compiler_emits_conditional_expression_branches
conditional_expression_slice_matches_host_c_compiler_exit_code
```

Current compile scan:

```text
scan=/tmp/c99inrust-doom-compile-scan-after-conditional.txt
ok=1
fail=61
OK m_swap.c
```

Manual CLI QA was run inside tmux session `c99exprqa` with
`target/debug/c99inrust`. The session closed by running `exit`.

```text
tmux_session=c99exprqa
long_long_cast_exit=4
call_args_exit=42
conditional_exit=42
m_fixed_exit=2
error: 461:5: expected identifier
manual_qa=PASS
```

Local Rust/slop gate for the expression slice:

```text
fmt: PASS
strict clippy: PASS, no warnings
rust-programmer no-excuse: PASS for 5 changed files
LSP diagnostics: PASS, 0 diagnostics
cargo test --all-targets --all-features: PASS, 51 tests
cargo nextest run --all-targets --all-features: PASS, 51 tests
cargo machete: PASS
cargo deny check: PASS
cargo audit: PASS
unsafe/miri: N/A; unsafe code is forbidden in Cargo.toml and src/lib.rs
remove-ai-slops: PASS for this slice; no debug leftovers, warning suppressions,
dead code, unsafe blocks, or needless behavior-changing cleanup found
```

Representative moved failures:

```text
FAIL m_fixed.c
  before: 434:14: expected expression
  after:  461:5: expected identifier
FAIL p_maputl.c
  before: 5456:14: expected punctuator )
  after:  5474:13: expected punctuator =
```

The next high-value blocker for `m_fixed.c` is the active `FixedDiv2` floating
path: `double` local declarations, double casts/arithmetic/comparisons, string
arguments, and call statements. Full Doom compile/link/run remains incomplete.

## Compile Scan After FixedDiv2 Double Slice

The compiler now parses, lowers, emits, assembles, and runs the isolated Doom
`FixedDiv2` double-expression slice against the host C compiler oracle. Coverage
added in this slice:

```text
double local declarations
double literals and `(double)` casts
double arithmetic and comparisons
string literals as pointer call arguments
expression statements for calls such as `I_Error(...)`
```

The first full Doom scan after double support moved `m_fixed.c` from the
`double c;` parse blocker to the Linux `<values.h>` constant blocker:

```text
tmux_session=c99-doom-double-qa
scan=/tmp/c99inrust-doom-compile-scan-after-double.txt
ok=1
fail=61
FAIL m_fixed.c
  error: unknown local: MININT
OK m_swap.c
```

Regression coverage added:

```text
compiler_accepts_fixeddiv2_double_slice
fixeddiv2_double_slice_matches_host_c_compiler_exit_code
```

## Compile Scan After values.h Integer Limits

For the public Linux Doom target, `doomtype.h` includes `<values.h>` under
`LINUX` and expects `MININT`/`MAXINT` to be available. The preprocessor now
preserves the system include line while injecting the Doom-era integer limit
macros needed by the current target slice.

Regression coverage added:

```text
preprocessor_provides_doom_values_h_integer_limits
```

Current compile scan:

```text
tmux_session=c99-doom-values-qa
scan=/tmp/c99inrust-doom-compile-scan-after-values.txt
ok=2
fail=60
OK m_fixed.c
OK m_swap.c
```

Representative next blockers:

```text
FAIL m_random.c
  error: 60:20: expected punctuator ;
FAIL m_bbox.c
  error: 131:8: expected punctuator ;
FAIL i_main.c
  error: assignment to undeclared local: myargc
FAIL r_sky.c
  error: assignment to undeclared local: skytexturemid
```

This remains a compile-progress milestone only. Full Doom compile/link/run/play
evidence is still missing.

## Manual CLI QA After Repeatable Scan Script

The repeatable scan script was run in tmux against the pinned official Doom
checkout without `tmux kill-server`.

```text
tmux_session=c99-doom-m-bbox-qa-20260518194530
scan=/tmp/c99-doom-m-bbox-qa-20260518194530.txt
pane=/tmp/c99-doom-m-bbox-qa-20260518194530-pane.txt
command=tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99-doom-m-bbox-qa-20260518194530.txt
ok=5
fail=57
OK m_bbox.c
OK m_fixed.c
OK m_random.c
OK m_swap.c
OK r_sky.c
```

The real CLI `build` path was also run in a bash-backed tmux session because
the default tmux shell is fish:

```text
tmux_session=c99-cli-bash-qa-20260518195007
source=/var/folders/nj/hqfr8ndn5q56cqw7jqgbrck40000gn/T/c99inrust-cli-qa.XXXXXX.h7E4NIjTnu/answer.c
log=/var/folders/nj/hqfr8ndn5q56cqw7jqgbrck40000gn/T/c99inrust-cli-qa.XXXXXX.h7E4NIjTnu/manual-qa.log
build=0
exe=42
```

## Compile Scan After Bounding Box Pointer Slice

`compile -S` now accepts the Doom `m_bbox.c` slice. This adds anonymous enum
constants as integer identifiers, preserves pointer parameters as pointer-width
local slots, and lowers `int` element pointer subscripts for both expression
loads and assignment targets. The immediate Doom blocker
`assignment to subscript targets is not supported yet` is cleared for
`m_bbox.c`.

Regression coverage added:

```text
compiler_accepts_m_bbox_pointer_subscript_slice
m_bbox_pointer_subscript_slice_matches_host_c_compiler_exit_code
```

Manual single-file QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX -I /tmp/DOOM/linuxdoom-1.10 /tmp/DOOM/linuxdoom-1.10/m_bbox.c -o /tmp/doom-m_bbox-next.s
lines=147
observed=_M_ClearBox,_M_AddToBox
observed=str w0, [x16, w17, sxtw #2]
observed=ldr w0, [x16, w0, sxtw #2]
```

Cross-target assembly QA:

```text
target/debug/c99inrust compile -S --target x86_64-unknown-linux-gnu -D NORMALUNIX -D LINUX -I /tmp/DOOM/linuxdoom-1.10 /tmp/DOOM/linuxdoom-1.10/m_bbox.c -o /tmp/doom-m_bbox-linux-x86_64.s
observed=movl (%rcx,%rax,4), %eax
observed=movl %eax, (%rcx,%rdx,4)
observed=.section .note.GNU-stack,"",@progbits

target/debug/c99inrust compile -S --target x86_64-apple-darwin -D NORMALUNIX -D LINUX -I /tmp/DOOM/linuxdoom-1.10 /tmp/DOOM/linuxdoom-1.10/m_bbox.c -o /tmp/doom-m_bbox-darwin-x86_64.s
observed=_M_ClearBox,_M_AddToBox
observed=movl (%rcx,%rax,4), %eax
observed=movl %eax, (%rcx,%rdx,4)
```

Current tmux compile scan:

```text
tmux_session=c99-doom-m-bbox-qa-20260518194530
scan=/tmp/c99-doom-m-bbox-qa-20260518194530.txt
pane=/tmp/c99-doom-m-bbox-qa-20260518194530-pane.txt
command=tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99-doom-m-bbox-qa-20260518194530.txt
ok=5
fail=57
OK m_bbox.c
OK m_fixed.c
OK m_random.c
OK m_swap.c
OK r_sky.c
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7142:11: expected punctuator ;
FAIL i_main.c
  error: assignment to undeclared local or global: myargc
FAIL p_inter.c
  error: unsupported function parameter
FAIL z_zone.c
  error: 753:9: expected punctuator ;
```

This remains a compile-progress milestone only. Full Doom compile/link/run/play
evidence is still missing.

## Compile Scan After doomdef Static Metadata

`compile -S` now accepts `doomdef.c`, which carries only an internal static
metadata string after preprocessing:

```text
static const char rcsid[] = "...";
```

The compiler allows this narrow empty-object case only when unsupported
data-only declarations are absent. Required but unsupported data globals, such
as the `weaponinfo` table in `d_items.c`, still fail instead of being silently
dropped.

Regression coverage added:

```text
compiler_accepts_ignorable_static_metadata_translation_unit
compiler_rejects_unsupported_data_only_translation_unit
```

Manual single-file QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX -I /tmp/c99inrust-doom-src/linuxdoom-1.10 /tmp/c99inrust-doom-src/linuxdoom-1.10/doomdef.c -o /tmp/c99inrust-doomdef.s
bytes=0

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX -I /tmp/c99inrust-doom-src/linuxdoom-1.10 /tmp/c99inrust-doom-src/linuxdoom-1.10/d_items.c -o /tmp/c99inrust-d-items.s
error=translation unit has no supported function definitions
```

The repeatable scan script was run in tmux against the pinned official Doom
checkout without `tmux kill-server`.

```text
tmux_session=c99inrust-doom-scan-1779104614
scan=/tmp/c99inrust-doom-scan-1779104614.txt
command=tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-scan-1779104614.txt
ok=8
fail=54
OK doomdef.c
OK i_main.c
OK m_argv.c
OK m_bbox.c
OK m_fixed.c
OK m_random.c
OK m_swap.c
OK r_sky.c
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7142:11: expected punctuator ;
FAIL d_items.c
  error: translation unit has no supported function definitions
FAIL m_cheat.c
  error: 107:13: expected punctuator )
FAIL z_zone.c
  error: 753:9: expected punctuator ;
```

This remains a compile-progress milestone only. Full Doom compile/link/run/play
evidence is still missing.

## Compile Scan After m_argv Post-Increment

`compile -S` now accepts the Doom `m_argv.c` command-line parameter scan. The
new surface is postfix `++` as an `int` side-effect expression in `for` post
clauses:

```text
for (i = 1;i<myargc;i++)
```

The implementation lowers this only where the increment value is unused. Using
the value of `i++` as an expression remains outside the supported executable
subset.

Regression coverage added:

```text
compiler_accepts_m_argv_post_increment_slice
post_increment_for_loop_matches_host_c_compiler_exit_code
```

Manual single-file QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX -I /tmp/c99inrust-doom-src/linuxdoom-1.10 /tmp/c99inrust-doom-src/linuxdoom-1.10/m_argv.c -o /tmp/c99inrust-m-argv.s
observed=_M_CheckParm
observed=_myargc
observed=_myargv
observed=call _strcasecmp
```

The repeatable scan script was run in tmux against the pinned official Doom
checkout without `tmux kill-server`.

```text
tmux_session=c99inrust-doom-scan-1779103487
scan=/tmp/c99inrust-doom-scan-1779103487.txt
command=tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-scan-1779103487.txt
ok=7
fail=55
OK i_main.c
OK m_argv.c
OK m_bbox.c
OK m_fixed.c
OK m_random.c
OK m_swap.c
OK r_sky.c
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7142:11: expected punctuator ;
FAIL m_cheat.c
  error: 107:13: expected punctuator )
FAIL p_inter.c
  error: unsupported function parameter
FAIL z_zone.c
  error: 753:9: expected punctuator ;
```

This remains a compile-progress milestone only. Full Doom compile/link/run/play
evidence is still missing.

## Compile Scan After i_main Extern Globals

`compile -S` now accepts the Doom `i_main.c` entry translation unit. The
preprocessed file gets `myargc` and `myargv` from `m_argv.h` as external
declarations:

```text
extern  int     myargc;
extern  char**  myargv;
```

The compiler records those declarations as scalar global bindings for lowering
and codegen, but does not emit storage for them in `i_main.c`; the definitions
remain the job of `m_argv.c`.

Regression coverage added:

```text
compiler_accepts_i_main_global_pointer_slice
compiler_accepts_i_main_extern_global_slice
i_main_global_pointer_slice_matches_host_c_compiler_exit_code
```

Manual single-file QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX -I /tmp/c99inrust-doom-src/linuxdoom-1.10 /tmp/c99inrust-doom-src/linuxdoom-1.10/i_main.c -o /tmp/c99inrust-i-main.s
observed=_main
observed=str w0, [x16, _myargc@PAGEOFF]
observed=str x0, [x16, _myargv@PAGEOFF]
observed=bl _D_DoomMain
```

The repeatable scan script was run in tmux against the pinned official Doom
checkout without `tmux kill-server`.

```text
tmux_session=c99inrust-doom-scan-1779102851
scan=/tmp/c99inrust-doom-scan-1779102851.txt
command=tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-scan-1779102851.txt
ok=6
fail=56
OK i_main.c
OK m_bbox.c
OK m_fixed.c
OK m_random.c
OK m_swap.c
OK r_sky.c
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7142:11: expected punctuator ;
FAIL m_argv.c
  error: 45:26: expected punctuator =
FAIL p_inter.c
  error: unsupported function parameter
FAIL z_zone.c
  error: 753:9: expected punctuator ;
```

This remains a compile-progress milestone only. Full Doom compile/link/run/play
evidence is still missing.

## Compile Scan After m_random Globals and Subscripts

The compiler now captures the Doom `m_random.c` global random table and mutable
indices:

```text
unsigned char rndtable[256] = { ... };
int rndindex = 0;
int prndindex = 0;
```

The executable subset also accepts global scalar assignments, global byte-array
subscripts, and the right-associative chained assignment in
`M_ClearRandom`:

```text
rndindex = prndindex = 0;
return rndtable[prndindex];
```

Regression coverage added:

```text
compiler_accepts_m_random_global_array_slice
m_random_global_array_slice_matches_host_c_compiler_exit_code
```

Manual compile QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX -I /tmp/DOOM/linuxdoom-1.10 /tmp/DOOM/linuxdoom-1.10/m_random.c -o /tmp/doom-m_random-next.s
lines=72
observed=_rndtable,_rndindex,_prndindex,_P_Random,_M_Random,_M_ClearRandom
observed=ldrb w0, [x16, w0, sxtw]
```

Cross-target assembly QA:

```text
target/debug/c99inrust compile -S --target x86_64-unknown-linux-gnu -D NORMALUNIX -D LINUX -I /tmp/DOOM/linuxdoom-1.10 /tmp/DOOM/linuxdoom-1.10/m_random.c -o /tmp/doom-m_random-linux-x86_64.s
observed=movzbl (%rcx,%rax), %eax
observed=.section .note.GNU-stack,"",@progbits

target/debug/c99inrust compile -S --target x86_64-apple-darwin -D NORMALUNIX -D LINUX -I /tmp/DOOM/linuxdoom-1.10 /tmp/DOOM/linuxdoom-1.10/m_random.c -o /tmp/doom-m_random-darwin-x86_64.s
observed=_rndtable(%rip)
observed=movzbl (%rcx,%rax), %eax
```

Current tmux compile scan:

```text
tmux_session=c99-doom-m-random-qa
scan=/tmp/c99inrust-doom-compile-scan-after-m-random.txt
ok=4
fail=58
OK m_fixed.c
OK m_random.c
OK m_swap.c
OK r_sky.c
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7142:11: expected punctuator ;
FAIL m_bbox.c
  error: assignment to subscript targets is not supported yet
FAIL i_main.c
  error: assignment to undeclared local or global: myargc
FAIL p_ceilng.c
  error: 6350:14: expected punctuator ;
```

This remains a compile-progress milestone only. Full Doom compile/link/run/play
evidence is still missing.

## Compile Scan After doomstat Enum Globals

`compile -S` now accepts the Doom `doomstat.c` global-state translation unit.
The source defines enum-backed globals after a large header surface with
declaration-only `extern` arrays:

```text
GameMode_t gamemode = indetermined;
GameMission_t gamemission = doom;
Language_t language = english;
boolean modifiedgame;
```

The compiler now ignores declaration-only `extern` arrays when deciding whether
a globals-only unit is an unsupported data definition, while still rejecting
actual unsupported data-table definitions. It also resolves enum constants in
global initializers and evaluates simple checked parenthesized enum arithmetic
chains such as `(8+16+32)`, using C operator precedence for the supported
arithmetic operators.

Regression coverage added:

```text
compiler_accepts_doomstat_enum_globals_slice
compiler_accepts_doomstat_globals_after_header_extern_arrays_slice
compiler_rejects_unsupported_data_only_translation_unit
enum_global_initializer_matches_host_c_compiler_exit_code
enum_arithmetic_initializer_matches_host_c_compiler_exit_code
enum_additive_chain_initializer_matches_host_c_compiler_exit_code
enum_mixed_precedence_initializer_matches_host_c_compiler_exit_code
```

Manual single-file QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX -I /tmp/c99inrust-doom-src/linuxdoom-1.10 /tmp/c99inrust-doom-src/linuxdoom-1.10/doomstat.c -o /tmp/c99inrust-doomstat.s
observed=_gamemode .long 4
observed=_gamemission .long 0
observed=_language .long 0
observed=_modifiedgame .long 0
```

The repeatable scan script was run in tmux against the pinned official Doom
checkout without `tmux kill-server`.

```text
tmux_session=c99inrust-doom-scan-1779105971
scan=/tmp/c99inrust-doom-scan-1779105971.txt
command=tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-scan-1779105971.txt
ok=9
fail=53
OK doomdef.c
OK doomstat.c
OK i_main.c
OK m_argv.c
OK m_bbox.c
OK m_fixed.c
OK m_random.c
OK m_swap.c
OK r_sky.c
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7052:28: unsupported global integer initializer
FAIL d_items.c
  error: translation unit has no supported function definitions
FAIL m_cheat.c
  error: 107:13: expected punctuator )
FAIL p_inter.c
  error: unsupported function parameter
```

This remains a compile-progress milestone only. Full Doom compile/link/run/play
evidence is still missing.

## Compile Scan After Scalar Initializer Recheck

`compile -S` now accepts unparenthesized numeric scalar global initializers such
as the `am_map.c` frame-buffer height definition:

```text
static int finit_height = 200 - 32;
```

Regression coverage added:

```text
compiler_accepts_unparenthesized_global_integer_initializer_slice
unparenthesized_global_initializer_matches_host_c_compiler_exit_code
```

The repeatable scan script was run in tmux against the pinned official Doom
checkout without `tmux kill-server`.

```text
tmux_session=c99inrust-doom-scan-1779106492
scan=/tmp/c99inrust-doom-scan-1779106492.txt
command=tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-scan-1779106492.txt
ok=9
fail=53
```

This did not add a new OK translation unit, but it moved `am_map.c` to the next
initializer blocker:

```text
before: FAIL am_map.c
  error: 7052:28: unsupported global integer initializer
after: FAIL am_map.c
  error: 7104:29: unsupported global integer initializer
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7104:29: unsupported global integer initializer
FAIL i_net.c
  error: 3870:16: unsupported global integer initializer
FAIL m_cheat.c
  error: 107:13: expected punctuator )
FAIL p_inter.c
  error: unsupported function parameter
```

This remains a compile-progress milestone only. Full Doom compile/link/run/play
evidence is still missing.

## Compile Scan After netinet Port Macro Recheck

The preprocessor now supplies the Doom-era `IPPORT_USERRESERVED` constant for
`<netinet/in.h>`, covering `i_net.c`'s global port definition:

```text
int DOOMPORT = (IPPORT_USERRESERVED +0x1d );
```

Regression coverage added:

```text
preprocessor_provides_doom_netinet_port_base
```

The repeatable scan script was run in tmux against the pinned official Doom
checkout without `tmux kill-server`.

```text
tmux_session=c99inrust-doom-scan-1779106926
scan=/tmp/c99inrust-doom-scan-1779106926.txt
command=tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-scan-1779106926.txt
ok=9
fail=53
```

This did not add a new OK translation unit, but it moved `i_net.c` into body
parsing:

```text
before: FAIL i_net.c
  error: 3870:16: unsupported global integer initializer
after: FAIL i_net.c
  error: 3905:5: expected expression
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7104:29: unsupported global integer initializer
FAIL i_net.c
  error: 3905:5: expected expression
FAIL m_cheat.c
  error: 107:13: expected punctuator )
FAIL p_inter.c
  error: unsupported function parameter
```

This remains a compile-progress milestone only. Full Doom compile/link/run/play
evidence is still missing.

## Compile Scan After Fixed-Point Initializer Recheck

`compile -S` now accepts Doom-shaped decimal fixed-point global initializer
expressions such as:

```text
static fixed_t scale_mtof = (.2*(1<<16));
```

The initializer is parsed through the expression parser, evaluates the leading
decimal literal as an exact rational value for the supported arithmetic slice,
and truncates the final scalar value to the integer global storage used by
`fixed_t`.

Regression coverage added:

```text
compiler_accepts_fixed_point_global_initializer_slice
fixed_point_global_initializer_matches_host_c_compiler_exit_code
```

The repeatable scan script was run in tmux against the pinned official Doom
checkout without `tmux kill-server`.

```text
tmux_session=c99inrust-doom-scan-1779107716
scan=/tmp/c99inrust-doom-scan-1779107716.txt
command=tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-scan-1779107716.txt
ok=9
fail=53
```

This did not add a new OK translation unit, but it moved `am_map.c` to the next
global initializer blocker:

```text
before: FAIL am_map.c
  error: 7104:29: unsupported global integer initializer
after: FAIL am_map.c
  error: 7117:32: unsupported global integer initializer
```

The new `am_map.c` blocker is a struct-style global initializer:

```text
static cheatseq_t cheat_amap = { cheat_amap_seq, 0 };
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7117:32: unsupported global integer initializer
FAIL i_net.c
  error: 3905:5: expected expression
FAIL m_cheat.c
  error: 107:13: expected punctuator )
FAIL p_inter.c
  error: unsupported function parameter
```

This remains a compile-progress milestone only. Full Doom compile/link/run/play
evidence is still missing.

## Compile Scan After Aggregate Global Recheck

`compile -S` now stops treating brace-initialized aggregate globals as scalar
integer globals. This covers the top-level `am_map.c` cheat-sequence state:

```text
static unsigned char cheat_amap_seq[] = { 0xb2, 0x26, 0x26, 0x2e, 0xff };
static cheatseq_t cheat_amap = { cheat_amap_seq, 0 };
```

The unsigned byte array still lowers as supported data. The aggregate typedef
global is skipped before supported function bodies, matching the existing
translation-unit behavior for unsupported data declarations while still
rejecting unsupported data-only translation units.

Regression coverage added:

```text
compiler_skips_aggregate_global_initializer_before_supported_function
aggregate_global_initializer_slice_matches_host_c_compiler_exit_code
compiler_rejects_unsupported_data_only_translation_unit
```

The repeatable scan script was run in tmux against the pinned official Doom
checkout without `tmux kill-server`.

```text
tmux_session=c99inrust-doom-scan-1779108257
scan=/tmp/c99inrust-doom-scan-1779108257.txt
command=tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-scan-1779108257.txt
ok=9
fail=53
```

This did not add a new OK translation unit, but it moved `am_map.c` into the
first supported function body:

```text
before: FAIL am_map.c
  error: 7117:32: unsupported global integer initializer
after: FAIL am_map.c
  error: 7142:11: expected punctuator ;
```

The new `am_map.c` blocker is a multi-declarator local declaration:

```text
void AM_getIslope(mline_t* ml, islope_t* is)
{
    int dx, dy;
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7142:11: expected punctuator ;
FAIL i_net.c
  error: 3905:5: expected expression
FAIL m_cheat.c
  error: 107:13: expected punctuator )
FAIL p_inter.c
  error: unsupported function parameter
```

This remains a compile-progress milestone only. Full Doom compile/link/run/play
evidence is still missing.

## Compile Scan After Local Multi-Declarator Recheck

`compile -S` now accepts comma-separated local integer declarations in supported
function bodies:

```text
int dx, dy;
```

The parser expands the declaration into a no-new-scope declaration list, so each
name is lowered through the existing local-slot path without changing real block
scope behavior.

Regression coverage added:

```text
compiler_accepts_multi_declarator_local_int_slice
multi_declarator_local_int_slice_matches_host_c_compiler_exit_code
```

The repeatable scan script was run in tmux against the pinned official Doom
checkout without `tmux kill-server`.

```text
tmux_session=c99inrust-doom-scan-1779108675
scan=/tmp/c99inrust-doom-scan-1779108675.txt
command=tools/doom-compile-scan.sh /tmp/c99inrust-doom-src /tmp/c99inrust-doom-scan-1779108675.txt
ok=9
fail=53
```

This did not add a new OK translation unit, but it moved `am_map.c` to the next
expression blocker:

```text
before: FAIL am_map.c
  error: 7142:11: expected punctuator ;
after: FAIL am_map.c
  error: 7144:12: expected punctuator ;
```

The new `am_map.c` blocker is pointer/member access:

```text
dy = ml->a.y - ml->b.y;
```

Representative next blockers:

```text
FAIL am_map.c
  error: 7144:12: expected punctuator ;
FAIL i_net.c
  error: 3905:5: expected expression
FAIL m_cheat.c
  error: 107:13: expected punctuator )
FAIL p_inter.c
  error: unsupported function parameter
```

This remains a compile-progress milestone only. Full Doom compile/link/run/play
evidence is still missing.

## Compile Scan After Member Access And Compound Assignment Slice

`compile -S` now records simple anonymous `typedef struct { ... } name;`
layouts, including comma-separated scalar fields and nested known struct fields.
Typed pointer parameters retain their struct referent so nested Doom member
expressions can lower to fixed byte-offset loads and stores. This covers the
`AM_getIslope` slice:

```c
dy = ml->a.y - ml->b.y;
dx = ml->b.x - ml->a.x;
is->islp = dx + dy;
```

Compound assignment operators are also parsed into the existing assignment IR
for scalar lvalues. This moved `am_map.c` past `m_x += m_w/2` and `m_x -= m_w/2`
in `AM_activateNewScale`.

Regression coverage added:

```text
compiler_accepts_doom_member_access_slice
doom_member_access_slice_matches_host_c_compiler_exit_code
compiler_accepts_compound_assignment_slice
compound_assignment_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/am_map.c \
  -o /tmp/c99inrust-am_map.s
error: 7290:5: expected expression
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779109827`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779109827.txt
ok=9
fail=53
```

The files still reaching assembly generation are unchanged:

```text
doomdef.c
doomstat.c
i_main.c
m_argv.c
m_bbox.c
m_fixed.c
m_random.c
m_swap.c
r_sky.c
```

Representative moved blocker:

```text
FAIL am_map.c
  before member access slice: 7144:12: expected punctuator ;
  before compound assignment slice: 7158:9: expected punctuator ;
  after this slice: 7290:5: expected expression
```

The next `am_map.c` blocker is a local static aggregate declaration:

```c
static event_t st_notify = { ev_keyup, ((('a'<<24)+('m'<<16)) | ('e'<<8)) };
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Global Matrix Slice

The Doom `g_game.c` and `r_bsp.c` translation units now reach assembly
generation. This slice adds support for the global data shapes that blocked
`g_game.c`: nested global int matrix initializers with row padding, global
pointer initializers that take the address of a global array element, and
`sizeof` on global struct objects. The same global int matrix support also
moved `r_bsp.c` past its reported expression blocker.

Regression coverage added:

```text
compiler_accepts_global_int_matrix_slice
compiler_accepts_global_pointer_subscript_initializer_slice
compiler_accepts_global_struct_object_with_struct_array_field_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S \
  -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/g_game.c \
  -o /tmp/c99inrust-g_game.s
```

The focused compile now reaches assembly generation.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779150882`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779150882.txt
ok=47
fail=15
```

Moved translation units:

```text
OK g_game.c
OK r_bsp.c
```

Remaining blockers:

```text
FAIL hu_stuff.c
  error: 5841:5: expected expression
FAIL i_net.c
  error: 3905:5: expected expression
FAIL i_sound.c
  error: 4750:43: expected expression
FAIL i_system.c
  error: 4552:5: expected expression
FAIL info.c
  error: translation unit has no supported function definitions
FAIL m_misc.c
  error: 5541:5: expected expression
FAIL p_mobj.c
  error: struct member value is not supported
FAIL p_setup.c
  error: assignment to non-pointer subscript targets is not supported
FAIL p_switch.c
  error: pointer member access requires a typed pointer
FAIL s_sound.c
  error: pointer member access requires a typed pointer
FAIL sounds.c
  error: 450:3: expected expression
FAIL st_stuff.c
  error: 8149:5: expected expression
FAIL v_video.c
  error: 5145:5: expected expression
FAIL w_wad.c
  error: 602:5: expected expression
FAIL wi_stuff.c
  error: unsupported global integer initializer
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After X11 Video Slice

The Linux video translation unit now reaches assembly generation. This slice
adds the Doom-specific X11/System V surface needed by `i_video.c`: opaque X11
scalar typedefs, minimal X11 struct layouts for the accessed fields, local
`struct shmid_ds` declarations, function-pointer casts, X11/SysV preprocessor
constants, extern byte matrices, global unsigned/double stretch tables,
many-argument calls, and post-increment of pointer elements stored in pointer
arrays.

Regression coverage added:

```text
compiler_accepts_many_argument_calls
compiler_accepts_x11_opaque_doom_video_slice
preprocessor_provides_doom_x11_and_sysv_constants
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S \
  -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/i_video.c \
  -o /tmp/i_video.s
```

The focused compile now reaches assembly generation.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779149967`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779149967.txt
ok=45
fail=17
```

Moved translation unit:

```text
OK i_video.c
```

Remaining blockers:

```text
FAIL g_game.c
  error: 9119:5: expected expression
FAIL hu_stuff.c
  error: 5841:5: expected expression
FAIL i_net.c
  error: 3905:5: expected expression
FAIL i_sound.c
  error: 4750:43: expected expression
FAIL i_system.c
  error: 4552:5: expected expression
FAIL info.c
  error: translation unit has no supported function definitions
FAIL m_misc.c
  error: 5541:5: expected expression
FAIL p_mobj.c
  error: struct member value is not supported
FAIL p_setup.c
  error: assignment to non-pointer subscript targets is not supported
FAIL p_switch.c
  error: pointer member access requires a typed pointer
FAIL r_bsp.c
  error: 5355:5: expected expression
FAIL s_sound.c
  error: pointer member access requires a typed pointer
FAIL sounds.c
  error: 450:3: expected expression
FAIL st_stuff.c
  error: 8149:5: expected expression
FAIL v_video.c
  error: 5145:5: expected expression
FAIL w_wad.c
  error: 602:5: expected expression
FAIL wi_stuff.c
  error: unsupported global integer initializer
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Doom Action Function Pointer Slice

Doom action-function pointer typedefs are now treated as pointer types in casts,
struct/union fields, local declarations, and parameters. The compiler also
parses `typedef union { ... } actionf_t`, keeps tagged struct aliases such as
`line_s` for `struct line_s**` referents, lowers pointer casts of function
symbols to symbol addresses, supports indirect calls through pointer-valued
members, records prototype-only pointer-return functions for `getSide(...)->x`
style member access, and merges extern struct-object declarations with their
definitions.

Regression coverage added:

```text
compiler_accepts_doom_action_function_pointer_slice
compiler_accepts_doom_action_function_pointer_call_slice
compiler_accepts_member_access_on_pointer_return_call_slice
compiler_accepts_tagged_struct_pointer_referent_slice
compiler_merges_extern_struct_object_with_definition_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_ceilng.c \
  -o /tmp/c99inrust-p_ceilng.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_doors.c \
  -o /tmp/c99inrust-p_doors.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_floor.c \
  -o /tmp/c99inrust-p_floor.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_plats.c \
  -o /tmp/c99inrust-p_plats.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_saveg.c \
  -o /tmp/c99inrust-p_saveg.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_tick.c \
  -o /tmp/c99inrust-p_tick.s
```

All focused compiles above now reach assembly generation. `p_mobj.c` moved past
the action-function pointer call parse blocker and now stops later on a
struct-member value gap.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779141874`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779141874.txt
ok=35
fail=27
```

Newly green since the previous scan:

```text
p_ceilng.c
p_doors.c
p_floor.c
p_inter.c
p_lights.c
p_plats.c
p_pspr.c
p_saveg.c
p_spec.c
p_telept.c
p_tick.c
p_user.c
z_zone.c
```

Representative remaining blockers:

```text
FAIL p_enemy.c
  error: unknown local or global: PIT_VileCheck
FAIL p_map.c
  error: unknown local or global: PIT_StompThing
FAIL p_mobj.c
  error: struct member value is not supported
FAIL r_main.c
  error: assignment to undeclared local or global: colfunc
FAIL r_things.c
  error: assignment to undeclared local or global: colfunc
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Doom Function Designator Callback Slice

Bare function identifiers that name known functions now lower to symbol
addresses when used as callback values. This covers Doom calls such as
`P_BlockThingsIterator(bx, by, PIT_VileCheck)` and
`P_BlockThingsIterator(bx, by, PIT_StompThing)` without requiring an explicit
function-pointer cast.

Regression coverage added:

```text
compiler_accepts_doom_function_designator_callback_argument_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_enemy.c \
  -o /tmp/c99inrust-p_enemy.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_map.c \
  -o /tmp/c99inrust-p_map.s
```

`p_enemy.c` now reaches assembly generation. `p_map.c` moved past
`PIT_StompThing` and now stops later on the typed-struct blocker
`unknown struct: intercept_t`.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779142396`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779142396.txt
ok=36
fail=26
```

Newly green since the previous scan:

```text
p_enemy.c
```

Representative remaining blockers:

```text
FAIL p_map.c
  error: unknown struct: intercept_t
FAIL p_mobj.c
  error: struct member value is not supported
FAIL r_main.c
  error: assignment to undeclared local or global: colfunc
FAIL r_things.c
  error: assignment to undeclared local or global: colfunc
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Doom Named Inner Union Slice

Struct typedef parsing now handles named inner aggregate fields such as
`union { ... } d;` and records a nested layout for the field. The top-level
struct/union decision also only looks at the aggregate keyword before the
top-level `{`, so a struct containing a nested union is no longer laid out as if
the whole parent were a union.

Regression coverage added:

```text
compiler_accepts_doom_named_inner_union_member_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_map.c \
  -o /tmp/c99inrust-p_map.s
```

`p_map.c` now reaches assembly generation after moving past `intercept_t` member
chains such as `in->d.line` and `in->d.thing`.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779143073`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779143073.txt
ok=37
fail=25
```

Newly green since the previous scan:

```text
p_map.c
```

Representative remaining blockers:

```text
FAIL p_maputl.c
  error: 5810:10: expected expression
FAIL p_mobj.c
  error: struct member value is not supported
FAIL r_main.c
  error: assignment to undeclared local or global: colfunc
FAIL r_things.c
  error: assignment to undeclared local or global: colfunc
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After `p_maputl.c` Pointer Flow Slice

Parenthesized pointer-member assignments, raw function-pointer parameters, and
direct calls through function-pointer parameter bindings are now accepted.
Pointer referents also track `short` as a 2-byte element, and the native
backends emit signed halfword loads plus halfword stores for `short*`
subscripts.

Regression coverage added:

```text
compiler_accepts_parenthesized_pointer_member_assignment_slice
compiler_accepts_function_pointer_parameter_call_slice
compiler_accepts_short_pointer_arithmetic_dereference_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_maputl.c \
  -o /tmp/c99inrust-p_maputl.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_data.c \
  -o /tmp/c99inrust-r_data.s
```

Both focused compiles now emit assembly.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779144130`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779144130.txt
ok=39
fail=23
```

Newly green since the previous scan:

```text
p_maputl.c
r_data.c
```

Representative remaining blockers:

```text
FAIL f_wipe.c
  error: 4492:5: expected expression
FAIL i_video.c
  error: unsupported function parameter
FAIL p_setup.c
  error: assignment to non-pointer subscript targets is not supported
FAIL r_main.c
  error: assignment to undeclared local or global: colfunc
FAIL r_things.c
  error: assignment to undeclared local or global: colfunc
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After `f_wipe.c` Function-Pointer Array Slice

Local function-pointer arrays with function-designator initializers are now
accepted, block-scope function prototypes are skipped, and dereferenced
function-pointer callees lower through the indirect-call path. This covers the
`wipe_ScreenWipe` local table and calls such as `(*wipes[wipeno*3])(...)`.

Regression coverage added:

```text
compiler_accepts_local_function_pointer_array_call_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/f_wipe.c \
  -o /tmp/c99inrust-f_wipe.s
```

Focused `f_wipe.c` compile now emits assembly.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779144793`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779144793.txt
ok=40
fail=22
```

Newly green since the previous scan:

```text
f_wipe.c
```

Representative remaining blockers:

```text
FAIL i_video.c
  error: unsupported function parameter
FAIL p_setup.c
  error: assignment to non-pointer subscript targets is not supported
FAIL r_main.c
  error: assignment to undeclared local or global: colfunc
FAIL r_things.c
  error: assignment to undeclared local or global: colfunc
FAIL wi_stuff.c
  error: unsupported global integer initializer
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After `m_menu.c` Callback Matrix Slice

The preprocessor now supplies the Doom-era `<fcntl.h>` open flags, global
`char` matrices decay to row pointers with nested byte subscripts, file-scope
function pointers are emitted as pointer storage, and function-pointer struct
fields are recorded as pointer fields. This covers the `m_menu.c`
`savegamestrings`, `detailNames`, `messageRoutine`, `menuitem_t.routine`, and
`menu_t.routine` shapes.

Regression coverage added:

```text
preprocessor_provides_doom_fcntl_open_constants
compiler_accepts_global_char_matrix_row_decay_slice
compiler_accepts_global_function_pointer_assignment_call_slice
compiler_accepts_struct_function_pointer_field_call_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/m_menu.c \
  -o /tmp/c99inrust-m_menu.s
```

Focused `m_menu.c` compile now emits assembly.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779145824`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779145824.txt
ok=41
fail=21
```

Newly green since the previous scan:

```text
m_menu.c
```

Representative moved blockers:

```text
FAIL r_main.c
  before this slice:
    error: assignment to undeclared local or global: colfunc
  after this slice:
    error: unknown local or global: R_DrawColumn

FAIL r_things.c
  before this slice:
    error: assignment to undeclared local or global: colfunc
  after this slice:
    error: unknown local or global: R_DrawTranslatedColumn
```

Representative remaining blockers:

```text
FAIL i_video.c
  error: unsupported function parameter
FAIL p_setup.c
  error: assignment to non-pointer subscript targets is not supported
FAIL r_main.c
  error: unknown local or global: R_DrawColumn
FAIL r_things.c
  error: unknown local or global: R_DrawTranslatedColumn
FAIL wi_stuff.c
  error: unsupported global integer initializer
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After `d_items.c` Struct Array Declaration Merge

This slice moved `d_items.c` from the conflicting `weaponinfo` declaration
blocker to assembly generation.

New covered Doom shape:

```c
extern weaponinfo_t weaponinfo[NUMWEAPONS];
weaponinfo_t weaponinfo[NUMWEAPONS] = { ... };
```

Regression coverage added:

```text
compiler_accepts_extern_struct_array_before_definition_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/d_items.c \
  -o /tmp/c99inrust-d_items.s
```

Focused `d_items.c` compile now succeeds and emits assembly.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779131186`. The session exited naturally; no
`tmux kill-server` was used:

```text
scan=/tmp/c99inrust-doom-scan-1779131186.txt
ok=13
fail=49
OK d_items.c
```

Representative moved blocker:

```text
FAIL d_items.c
  before this slice:
    error: conflicting global declaration: weaponinfo
  after this slice:
    OK d_items.c
```

Next visible blockers include `d_main.c` unsupported declaration parsing,
enum-sized arrays, old-style function definitions, function-pointer
declarations, and unsupported expression forms.

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After `am_map.c` Struct Address Sweep

This slice moved `am_map.c` from the typed pointer-subscript member blocker to
assembly generation.

New covered Doom shapes include:

```c
lines[i].v1->x;
markpoints[i].x;
m_paninc.x;
plr->powers[pw_allmap];
((memblock_t *)p)->id;
&l.a.x;
cheat_player_arrow;
```

Regression coverage added:

```text
compiler_accepts_pointer_subscript_struct_member_slice
compiler_accepts_extern_pointer_subscript_struct_member_slice
compiler_accepts_global_struct_array_member_slice
compiler_accepts_global_struct_object_member_slice
compiler_accepts_extern_struct_array_address_slice
compiler_accepts_extern_int_array_slice
compiler_accepts_sizeof_struct_typedef_slice
compiler_accepts_typed_pointer_cast_member_slice
compiler_accepts_standard_stream_global_slice
compiler_accepts_struct_array_field_subscript_slice
compiler_accepts_struct_member_address_slice
compiler_accepts_global_struct_array_decay_slice
compiler_accepts_aggregate_global_address_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/am_map.c \
  -o /tmp/c99inrust-am_map.s
```

Focused `am_map.c` compile now succeeds and emits assembly.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779129982`. The session exited naturally; no
`tmux kill-server` was used:

```text
scan=/tmp/c99inrust-doom-scan-1779129982.txt
ok=12
fail=50
OK am_map.c
```

Representative moved blocker:

```text
FAIL am_map.c
  before this slice:
    error: member access requires a struct base
  after this slice:
    OK am_map.c
```

Next visible blockers include conflicting declarations such as
`weaponinfo`, enum-sized arrays, old-style function definitions,
function-pointer declarations, and unsupported expression forms.

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After `am_map.c` Local Control-Flow Sweep

`compile -S` now moves the focused `am_map.c` path past several local function
body blockers:

```c
static event_t st_notify = { ev_keyup, AM_MSGENTERED };
break;
char namebuf[9];
I_Error("Z_CT at " __FILE__ ":%i", __LINE__);
static int lastlevel = -1, lastepisode = -1;
static nexttic = 0;
switch (ev->data1) { case '-': ... }
```

The implementation covers these as narrow compile-progress slices:

```text
compiler_accepts_local_static_aggregate_address_slice
compiler_accepts_break_statement_slice
compiler_accepts_local_char_array_decay_slice
compiler_concatenates_adjacent_string_literals_slice
preprocessor_expands_file_and_line_builtins_after_macros
compiler_accepts_local_static_scalar_declaration_slice
compiler_accepts_switch_case_break_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/am_map.c \
  -o /tmp/c99inrust-am_map.s
```

Focused `am_map.c` compile now reaches the next unsupported local array:

```text
error: only local char arrays are supported
```

That line is:

```c
static int litelevels[] = { 0, 4, 7, 10, 12, 14, 15, 15 };
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779124809`, then that session was closed with `exit`
without `tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779124809.txt
ok=11
fail=51
```

Representative moved blocker:

```text
FAIL am_map.c
  before this sweep:
    error: 7290:5: expected expression
  after this sweep:
    error: only local char arrays are supported
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After `am_map.c` Local Array And Enum Sweep

`compile -S` now moves the focused `am_map.c` path past the local light-level
array and the first local declarations in `AM_clipMline`:

```c
static int litelevels[] = { 0, 4, 7, 10, 12, 14, 15, 15 };
if (litelevelscnt == sizeof(litelevels)/sizeof(int)) litelevelscnt = 0;
enum { LEFT = 1, RIGHT = 2, BOTTOM = 4, TOP = 8 };
register outcode1 = 0;
```

Regression coverage added:

```text
compiler_accepts_local_int_array_sizeof_slice
compiler_accepts_local_enum_and_register_implicit_int_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/am_map.c \
  -o /tmp/c99inrust-am_map.s
```

Focused `am_map.c` compile now reaches the next unsupported local struct
object:

```text
error: 7692:14: expected punctuator ;
```

That line is:

```c
fpoint_t tmp;
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779125648`, then that session was closed with `exit`
without killing the tmux server:

```text
scan=/tmp/c99inrust-doom-scan-1779125648.txt
ok=11
fail=51
```

Representative moved blocker:

```text
FAIL am_map.c
  before this sweep:
    error: only local char arrays are supported
  after this sweep:
    error: 7692:14: expected punctuator ;
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After `am_map.c` Struct Local And Continue Sweep

`compile -S` now moves the focused `am_map.c` path past the local struct object
and nearby control-flow/global declaration blockers:

```c
fpoint_t tmp;
tmp.x = fl->a.x + (dx*(fl->a.y))/dy;
fl->a = tmp;
static fline_t fl;
continue;
angle_t a;
static fixed_t m_x, m_y;
```

Regression coverage added:

```text
compiler_accepts_local_struct_object_member_slice
compiler_accepts_static_local_struct_object_slice
compiler_accepts_continue_statement_slice
compiler_accepts_angle_t_parameter_slice
compiler_accepts_global_int_declarator_list_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/am_map.c \
  -o /tmp/c99inrust-am_map.s
```

Focused `am_map.c` compile now reaches typed global pointer/member support:

```text
error: unknown local or global: plr
```

That declaration is:

```c
static player_t *plr;
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779126907`; the session exited naturally after the scan
without killing the tmux server:

```text
scan=/tmp/c99inrust-doom-scan-1779126907.txt
ok=11
fail=51
```

Representative moved blocker:

```text
FAIL am_map.c
  before this sweep:
    error: 7692:14: expected punctuator ;
  after this sweep:
    error: unknown local or global: plr
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After `am_map.c` Typed Pointer Sweep

`compile -S` now moves the focused `am_map.c` path past the file-scope typed
player pointer and nested typed pointer members:

```c
static player_t *plr;
plr->mo->x;
plr->mo->y;
```

The implementation preserves struct referents for file-scope pointer globals,
pointer-valued struct fields, and struct typedefs containing untracked typedef
scalars or simple array declarators.

Regression coverage added:

```text
compiler_accepts_typed_global_struct_pointer_member_slice
compiler_accepts_struct_fields_with_typedef_and_array_slice
compiler_accepts_nested_typed_pointer_member_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/am_map.c \
  -o /tmp/c99inrust-am_map.s
```

Focused `am_map.c` compile now reaches typed pointer-subscript struct bases:

```text
error: member access requires a struct base
```

The representative family is:

```c
lines[i].v1->x;
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779127759`; the session exited naturally after the scan
without killing the tmux server:

```text
scan=/tmp/c99inrust-doom-scan-1779127759.txt
ok=11
fail=51
```

Representative moved blocker:

```text
FAIL am_map.c
  before this sweep:
    error: unknown local or global: plr
  after this sweep:
    error: member access requires a struct base
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Pointer Walk Expression Slice

`compile -S` now accepts the pointer-walk expression forms used by the middle of
`f_wipe.c`:

```c
short* dest;
dest = (short*) Z_Malloc(width*height*2, 1, 0);
if (*w != *e) { ... }
while (ticks--) { ... }
y = (int *) Z_Malloc(width*sizeof(int), 1, 0);
s = &((short *)wipe_scr_end)[i*height+y[i]];
```

The implementation keeps this intentionally narrow: local `*` declarators lower
as pointer locals, pointer casts lower through the existing cast path,
dereference lowers as pointer subscript index zero, `sizeof(type)` folds to a
constant, post-decrement parses through assignment expression machinery, and
address-of-subscript lowers as pointer plus index.

Regression coverage added:

```text
compiler_accepts_local_pointer_declaration_slice
local_pointer_declaration_matches_host_c_compiler_exit_code
compiler_accepts_pointer_dereference_slice
pointer_dereference_slice_matches_host_c_compiler_exit_code
compiler_accepts_sizeof_type_slice
sizeof_type_slice_matches_host_c_compiler_exit_code
compiler_accepts_post_decrement_condition_slice
post_decrement_condition_slice_matches_host_c_compiler_exit_code
compiler_accepts_address_of_subscript_slice
address_of_subscript_slice_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/f_wipe.c \
  -o /tmp/c99inrust-f_wipe.s
error: 4486:5: expected expression
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779111285`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779111285.txt
ok=9
fail=53
```

Representative moved blocker:

```text
FAIL f_wipe.c
  before pointer declaration slice: 4273:5: expected expression
  after pointer declaration slice: 4314:6: expected expression
  after sizeof/post--/address-of slice: 4486:5: expected expression
```

The next `f_wipe.c` blocker is a local static function-pointer array:

```c
static int (*wipes[])(int, int, int) =
{
    wipe_initColorXForm, wipe_doColorXForm, wipe_exitColorXForm,
    wipe_initMelt, wipe_doMelt, wipe_exitMelt
};
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Unsigned Cast Slice

`compile -S` now accepts the unsigned cast syntax used by Doom's draw and cheat
code paths:

```c
(unsigned)dc_x >= SCREENWIDTH
cheat_xlate_table[(unsigned char)key]
```

This slice is intentionally syntactic and narrow. The parser maps supported
unsigned integer casts onto the current integer scalar ABI, so positive-value
oracle slices match host C, but full unsigned conversion and wraparound
semantics are still future work.

Regression coverage added:

```text
compiler_accepts_unsigned_cast_slice
unsigned_cast_slice_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/m_cheat.c \
  -o /tmp/c99inrust-m_cheat.s
error: 134:5: expected expression

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_draw.c \
  -o /tmp/c99inrust-r_draw.s
error: 5741:5: expected expression
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779111884`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779111884.txt
ok=9
fail=53
```

Representative moved blockers:

```text
FAIL m_cheat.c
  before unsigned cast slice: 113:22: expected expression
  after unsigned cast slice: 134:5: expected expression

FAIL r_draw.c
  before unsigned cast slice: 5723:10: expected expression
  after unsigned cast slice: 5741:5: expected expression
```

The next `m_cheat.c` blocker is a mixed local pointer/scalar declaration:

```c
unsigned char *p, c;
```

The next `r_draw.c` blocker is a `do` loop:

```c
do
{
    *dest = dc_colormap[dc_source[(frac>>16)&127]];
    dest += 320;
    frac += fracstep;
} while (count--);
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Local Declaration Specifier Slice

`compile -S` now accepts local declaration specifier sequences where Doom mixes
unsigned integer specifiers, pointer declarators, and scalar declarators:

```c
unsigned char *p, c;
unsigned short* colofs;
```

This extends the existing local declarator-list path. It still lowers
`unsigned char` and `unsigned short` through the current integer scalar ABI, so
full unsigned-width storage and conversion semantics remain future work.

Regression coverage added:

```text
compiler_accepts_mixed_pointer_scalar_local_declaration_slice
mixed_pointer_scalar_local_declaration_slice_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/m_cheat.c \
  -o /tmp/c99inrust-m_cheat.s
error: 137:24: expected expression
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779112512`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779112512.txt
ok=9
fail=53
```

Representative moved blockers:

```text
FAIL m_cheat.c
  before declaration specifier slice: 134:5: expected expression
  after declaration specifier slice: 137:24: expected expression

FAIL r_data.c
  before declaration specifier slice: 6556:5: expected expression
  after declaration specifier slice: 6570:14: expected punctuator ;
```

The next `m_cheat.c` blocker is a dereference around a pointer post-increment:

```c
while (*(p++) != 1);
```

The next `r_data.c` blocker is the comma expression in a `for` initializer and
post expression:

```c
for (i=0 , patch = texture->patches;
     i<texture->patchcount;
     i++, patch++)
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Post-Increment Value Slice

`compile -S` now accepts post-increment expressions as values for direct `int`
and pointer lvalues, and it accepts empty statements. This covers the next
`m_cheat.c` loop condition shape:

```c
while (*(p++) != 1);
```

The expression lowering now leaves the old value in the result register while
writing back the incremented value. Pointer increments still use the compiler's
current unscaled pointer arithmetic model.

Regression coverage added:

```text
compiler_emits_post_increment_value_slice
compiler_accepts_pointer_post_increment_dereference_slice
post_increment_value_slice_matches_host_c_compiler_exit_code
empty_while_post_increment_slice_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/m_cheat.c \
  -o /tmp/c99inrust-m_cheat.s
error: 139:5: expected expression
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779113438`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779113438.txt
ok=9
fail=53
```

Representative moved blocker:

```text
FAIL m_cheat.c
  before post-increment value slice: 137:24: expected expression
  after post-increment value slice: 139:5: expected expression
```

The next `m_cheat.c` blocker is a `do` loop:

```c
do
{
    c = *p;
    *(buffer++) = c;
    *(p++) = 0;
}
while (c && *p!=0xff );
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Do-While Slice

`compile -S` now accepts `do { ... } while (...)` statements and lowers them to
a body-first loop. This covers the next `m_cheat.c` loop shape:

```c
do
{
    c = *p;
    *(buffer++) = c;
    *(p++) = 0;
}
while (c && *p!=0xff );
```

Regression coverage added:

```text
compiler_emits_back_edges_for_do_while_loops
compiler_accepts_doom_do_while_pointer_copy_slice
do_while_loop_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/m_cheat.c \
  -o /tmp/c99inrust-m_cheat.s
error: unknown local or global: cheat_xlate_table

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_draw.c \
  -o /tmp/c99inrust-r_draw.s
error: 5903:6: expected expression
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779114005`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779114005.txt
ok=9
fail=53
```

Representative moved blockers:

```text
FAIL m_cheat.c
  before do-while slice: 139:5: expected expression
  after do-while slice: unknown local or global: cheat_xlate_table

FAIL r_draw.c
  before do-while slice: 5741:5: expected expression
  after do-while slice: 5903:6: expected expression
```

The next `m_cheat.c` blocker is the static unsigned byte array used by the
cheat translation table:

```c
static unsigned char cheat_xlate_table[256];
...
cheat_xlate_table[(unsigned char)key]
```

The next `r_draw.c` blocker is prefix increment in `R_DrawFuzzColumn`:

```c
if (++fuzzpos == 50)
    fuzzpos = 0;
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Byte Array Slice

`compile -S` now accepts zero-filled static unsigned-byte globals and assignment
through byte-array subscripts. This covers the next `m_cheat.c` table shape:

```c
static unsigned char cheat_xlate_table[256];
...
for (i=0;i<256;i++) cheat_xlate_table[i] = SCRAMBLE(i);
```

Regression coverage added:

```text
compiler_accepts_m_cheat_xlate_table_slice
m_cheat_xlate_table_slice_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/m_cheat.c \
  -o /tmp/c99inrust-m_cheat.s
error: struct member value is not supported
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779114786`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779114786.txt
ok=9
fail=53
```

Representative moved blocker:

```text
FAIL m_cheat.c
  before byte array slice: unknown local or global: cheat_xlate_table
  after byte array slice: struct member value is not supported
```

The next `m_cheat.c` blocker is reading pointer fields from `cheatseq_t`:

```c
if (!cht->p)
    cht->p = cht->sequence;
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Pointer Member Slice

`compile -S` now accepts pointer-valued struct member loads and stores. This
covers the `m_cheat.c` `cheatseq_t` field shape:

```c
if (!cht->p)
    cht->p = cht->sequence;
```

Regression coverage added:

```text
compiler_accepts_pointer_struct_member_values_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/m_cheat.c \
  -o /tmp/c99inrust-m_cheat.s
error: post-increment expression supports direct lvalues only
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779115498`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779115498.txt
ok=9
fail=53
```

Representative moved blocker:

```text
FAIL m_cheat.c
  before pointer member slice: struct member value is not supported
  after pointer member slice: post-increment expression supports direct lvalues only
```

The next `m_cheat.c` blocker is a post-increment expression on a pointer field:

```c
*(cht->p++) = key;
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Pointer Field Post-Increment Slice

`compile -S` now accepts post-increment expressions on pointer struct members.
This covers the final current `m_cheat.c` blocker:

```c
*(cht->p++) = key;
```

Regression coverage added:

```text
compiler_accepts_pointer_member_post_increment_value_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/m_cheat.c \
  -o /tmp/c99inrust-m_cheat.s
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779116128`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779116128.txt
ok=10
fail=52
```

Representative moved blocker:

```text
FAIL m_cheat.c
  before pointer field post-increment slice:
    post-increment expression supports direct lvalues only
OK m_cheat.c
```

The next broad Doom blocker is still outside `m_cheat.c`; for example,
`r_draw.c` currently stops at prefix increment in `R_DrawFuzzColumn`:

```c
if (++fuzzpos == 50)
    fuzzpos = 0;
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Prefix Increment Slice

`compile -S` now accepts prefix increment and decrement expressions by parsing
them through the existing assignment-expression path. This covers the previous
`r_draw.c` blocker:

```c
if (++fuzzpos == 50)
    fuzzpos = 0;
```

Regression coverage added:

```text
compiler_accepts_prefix_increment_condition_slice
prefix_increment_condition_slice_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_draw.c \
  -o /tmp/c99inrust-r_draw.s
```

Focused `r_draw.c` compile now moves to a later parser blocker:

```text
error: 6207:15: expected punctuator ;
```

The corresponding preprocessed source is the local char-array string
initializer in `R_FillBackScreen`:

```c
char name1[] = "FLOOR7_2";
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779116723`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779116723.txt
ok=10
fail=52
```

Representative moved blocker:

```text
FAIL r_draw.c
  before prefix increment slice:
    error: 5903:6 expected expression
  after prefix increment slice:
    error: 6207:15: expected punctuator ;
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Local Char Array String Slice

`compile -S` now accepts local `char name[] = "literal"` declarations by
lowering them as pointer locals initialized from existing string-literal
storage. This covers the previous focused `r_draw.c` blocker:

```c
char name1[] = "FLOOR7_2";
char name2[] = "GRNROCK";
```

Regression coverage added:

```text
compiler_accepts_local_char_array_string_initializer_slice
local_char_array_string_initializer_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_draw.c \
  -o /tmp/c99inrust-r_draw.s
```

Focused `r_draw.c` compile now moves to a later parser blocker:

```text
error: unsupported function parameter
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779117283`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779117283.txt
ok=10
fail=52
```

Representative moved blocker:

```text
FAIL r_draw.c
  before local char array string slice:
    error: 6207:15: expected punctuator ;
  after local char array string slice:
    error: unsupported function parameter
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Unsigned Parameter Slice

`compile -S` now accepts plain `unsigned` parameters as integer parameters.
This covers the previous focused `r_draw.c` blocker:

```c
void
R_VideoErase
( unsigned ofs,
  int count )
```

Regression coverage added:

```text
compiler_accepts_unsigned_parameter_slice
unsigned_parameter_slice_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_draw.c \
  -o /tmp/c99inrust-r_draw.s
```

Focused `r_draw.c` compile now reaches lowering and stops at:

```text
error: unknown local or global: ylookup
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779117824`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779117824.txt
ok=10
fail=52
```

Representative moved blockers:

```text
FAIL r_draw.c
  before unsigned parameter slice:
    error: unsupported function parameter
  after unsigned parameter slice:
    error: unknown local or global: ylookup
FAIL r_things.c
  after unsigned parameter slice:
    error: 5564:3: expected punctuator ,
FAIL tables.c
  after unsigned parameter slice:
    error: 178:5: expected expression
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Global Pointer Array Slice

`compile -S` now accepts zero-initialized global pointer arrays and supports
loading and storing individual pointer elements. This covers the previous
focused `r_draw.c` blocker:

```c
byte* ylookup[MAXHEIGHT];
```

Regression coverage added:

```text
compiler_accepts_global_pointer_array_slice
global_pointer_array_slice_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_draw.c \
  -o /tmp/c99inrust-r_draw.s
```

Focused `r_draw.c` compile now reaches the next global array:

```text
error: unknown local or global: columnofs
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779118857`, then that session was closed without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779118857.txt
ok=10
fail=52
```

Representative moved blocker:

```text
FAIL r_draw.c
  before global pointer array slice:
    error: unknown local or global: ylookup
  after global pointer array slice:
    error: unknown local or global: columnofs
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Global Int Array Slice

`compile -S` now accepts zero-initialized global int arrays and supports
loading and storing individual int elements. This covers the previous focused
`r_draw.c` blocker:

```c
int columnofs[MAXWIDTH];
```

Regression coverage added:

```text
compiler_accepts_global_int_array_slice
global_int_array_slice_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_draw.c \
  -o /tmp/c99inrust-r_draw.s
```

Focused `r_draw.c` compile now reaches the next global:

```text
error: unknown local or global: dc_colormap
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779119769`, then that session was closed with `exit`
without `tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779119769.txt
ok=10
fail=52
```

Representative moved blocker:

```text
FAIL r_draw.c
  before global int array slice:
    error: unknown local or global: columnofs
  after global int array slice:
    error: unknown local or global: dc_colormap
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Lighttable Typedef Slice

`compile -S` now treats Doom's `lighttable_t` typedef as a supported scalar,
which lets existing global pointer parsing cover:

```c
lighttable_t* dc_colormap;
```

Regression coverage added:

```text
compiler_accepts_lighttable_pointer_global_slice
lighttable_pointer_global_slice_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_draw.c \
  -o /tmp/c99inrust-r_draw.s
```

Focused `r_draw.c` compile now reaches the next global:

```text
error: unknown local or global: fuzzoffset
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779120355`, then that session was closed with `exit`
without `tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779120355.txt
ok=10
fail=52
```

Representative moved blocker:

```text
FAIL r_draw.c
  before lighttable typedef slice:
    error: unknown local or global: dc_colormap
  after lighttable typedef slice:
    error: unknown local or global: fuzzoffset
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Initialized Int Array Slice

`compile -S` now accepts global int arrays with integer initializer lists,
including negative and parenthesized constant expressions. This covers the
previous focused `r_draw.c` blocker:

```c
int fuzzoffset[FUZZTABLE] = { FUZZOFF, -FUZZOFF, ... };
```

Regression coverage added:

```text
compiler_accepts_initialized_global_int_array_slice
initialized_global_int_array_slice_matches_host_c_compiler_exit_code
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_draw.c \
  -o /tmp/c99inrust-r_draw.s
```

Focused `r_draw.c` compile now reaches the next global:

```text
error: unknown local or global: screens
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779120950`, then that session was closed with `exit`
without `tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779120950.txt
ok=10
fail=52
```

Representative moved blocker:

```text
FAIL r_draw.c
  before initialized int array slice:
    error: unknown local or global: fuzzoffset
  after initialized int array slice:
    error: unknown local or global: screens
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Extern Pointer Array And Struct Pointer Local Slice

`compile -S` now accepts extern global pointer arrays without emitting storage.
This covers both literal and symbolic Doom bounds:

```c
extern byte* screens[5];
extern char *sprnames[NUMSPRITES];
```

Function-body parsing also accepts local pointers to typedef'd structs whose
typedef has already been parsed, covering the next focused `r_draw.c` blocker:

```c
patch_t* patch;
```

Regression coverage added:

```text
compiler_accepts_extern_global_pointer_array_slice
compiler_accepts_extern_global_pointer_array_symbolic_length
extern_global_pointer_array_slice_matches_host_c_compiler_exit_code
compiler_accepts_local_struct_pointer_declaration_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_draw.c \
  -o /tmp/c99inrust-r_draw.s
```

Focused `r_draw.c` compile now succeeds and emits assembly.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779121990`, then that session was closed with `exit`
without `tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779121990.txt
ok=11
fail=51
```

Representative moved blocker:

```text
FAIL r_draw.c
  before this slice:
    error: unknown local or global: screens
  after this slice:
    OK r_draw.c
```

Next visible blockers include symbolic/sized non-pointer arrays, local static
aggregate declarations, function-pointer arrays, and several unsupported
expression forms.

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Struct Aggregate Array Skip

The parser now rejects known struct typedefs from the global integer-array
classifier. That keeps aggregate data such as:

```c
mline_t player_arrow[] = { ... };
```

out of the integer-array storage path, so supported functions later in the same
translation unit can still be parsed.

Regression coverage added:

```text
compiler_skips_struct_array_initializer_before_supported_function
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/am_map.c \
  -o /tmp/c99inrust-am_map.s
```

Focused `am_map.c` compile now reaches the expected local static aggregate
blocker:

```text
error: 7290:5: expected expression
```

That line is:

```c
static event_t st_notify = { ev_keyup, AM_MSGENTERED };
```

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779122912`, then that session was closed with `exit`
without `tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779122912.txt
ok=11
fail=51
```

Representative moved blocker:

```text
FAIL am_map.c
  before struct aggregate array skip:
    error: expected unsigned char array length
  after struct aggregate array skip:
    error: 7290:5: expected expression
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After d_main Declaration Slice

`compile -S` now accepts the focused `d_main.c` translation unit. This slice
adds the Doom declaration and expression forms reached by `d_main.c`, including
enum typedef scalars, opaque `FILE*` locals, local pointer arrays,
comma-expression `for` initializers/posts, block-scope extern arrays, local
`char name[23][8]` matrices with row decay, global plain `char` arrays, byte
loads/stores through `char*` pointer arithmetic and nested `char**` subscripts,
and the libc macro constants `R_OK`, `SEEK_SET`, `SEEK_CUR`, `SEEK_END`, and
`NULL`.

Regression coverage added:

```text
compiler_accepts_doom_enum_typedef_scalar_slice
compiler_accepts_local_pointer_array_slice
compiler_accepts_for_comma_expression_slice
compiler_accepts_block_extern_int_array_slice
compiler_accepts_local_char_matrix_row_decay_slice
compiler_accepts_pointer_to_pointer_subscript_address_slice
compiler_accepts_nested_extern_struct_array_address_slice
compiler_accepts_global_char_array_decay_slice
compiler_accepts_opaque_file_pointer_local_slice
compiler_emits_byte_access_for_char_pointer_dereference_slice
compiler_emits_byte_access_for_char_pointer_nested_subscript_slice
preprocessor_provides_doom_unistd_access_constant
preprocessor_provides_doom_stdio_seek_constants
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/d_main.c \
  -o /tmp/c99inrust-d_main.s
```

Focused `d_main.c` compile now succeeds and emits assembly.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779134022`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779134022.txt
ok=15
fail=47
```

Translation units currently reaching assembly:

```text
am_map.c
d_items.c
d_main.c
doomdef.c
doomstat.c
i_main.c
m_argv.c
m_bbox.c
m_cheat.c
m_fixed.c
m_random.c
m_swap.c
r_draw.c
r_sky.c
st_lib.c
```

Representative moved blocker:

```text
FAIL d_main.c
  before this slice:
    error: 7678:26: expected punctuator ;
  after this slice:
    OK d_main.c
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After d_net Flow Slice

`compile -S` now accepts the focused `d_net.c` translation unit. This slice
adds the Doom declaration, control-flow, literal, and struct-copy forms reached
by `d_net.c`, including plain `unsigned` local declarations, `goto` labels,
suffixed integer literals such as `0x12345678l`, struct arrays inside struct
fields, address-taking and assignment through those struct-array fields, global
struct object assignment from a pointer dereference, and unsigned 32-bit mask
immediates.

Regression coverage added:

```text
compiler_accepts_plain_unsigned_local_declaration_slice
compiler_accepts_goto_label_slice
lexer_accepts_integer_literal_suffixes
compiler_accepts_struct_array_field_subscript_address_slice
compiler_accepts_global_struct_object_assignment_from_pointer_slice
compiler_accepts_unsigned_32_bit_mask_literals_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/d_net.c \
  -o /tmp/c99inrust-d_net.s
```

Focused `d_net.c` compile now succeeds and emits assembly.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779135763`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779135763.txt
ok=17
fail=45
```

Translation units currently reaching assembly:

```text
am_map.c
d_items.c
d_main.c
d_net.c
doomdef.c
doomstat.c
i_main.c
m_argv.c
m_bbox.c
m_cheat.c
m_fixed.c
m_random.c
m_swap.c
r_draw.c
r_sky.c
st_lib.c
tables.c
```

Representative moved blockers:

```text
FAIL d_net.c
  before this slice:
    error: 4134:5: expected expression
  after this slice:
    OK d_net.c

FAIL tables.c
  before this slice:
    error: 178:5: expected expression
  after this slice:
    OK tables.c
```

Next visible blockers include file-scope pointer string initializers in
`dstrings.c` and `f_finale.c`, function-pointer arrays, enum-sized arrays, and
old-style function definitions.

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After String Pointer Global Slice

`compile -S` now accepts the focused `dstrings.c` and `f_finale.c` translation
units. This slice adds file-scope pointer string initializers, file-scope
pointer string-array initializers, typed referents for extern pointer arrays,
and simple struct typedef aliases such as `typedef post_t column_t;`.

Regression coverage added:

```text
compiler_accepts_global_pointer_string_initializer_slice
compiler_accepts_global_pointer_string_array_initializer_slice
compiler_accepts_extern_typed_pointer_array_member_slice
compiler_accepts_struct_typedef_alias_pointer_member_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/dstrings.c \
  -o /tmp/c99inrust-dstrings.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/f_finale.c \
  -o /tmp/c99inrust-f_finale.s
```

Focused `dstrings.c` and `f_finale.c` compiles now succeed and emit assembly.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779136701`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779136701.txt
ok=19
fail=43
```

Translation units currently reaching assembly:

```text
am_map.c
d_items.c
d_main.c
d_net.c
doomdef.c
doomstat.c
dstrings.c
f_finale.c
i_main.c
m_argv.c
m_bbox.c
m_cheat.c
m_fixed.c
m_random.c
m_swap.c
r_draw.c
r_sky.c
st_lib.c
tables.c
```

Representative moved blockers:

```text
FAIL dstrings.c
  before this slice:
    error: translation unit has no supported function definitions
  after this slice:
    OK dstrings.c

FAIL f_finale.c
  before this slice:
    error: unknown local or global: e1text
  after this slice:
    OK f_finale.c

FAIL r_segs.c
  before this slice:
    error: unknown local or global: column_t
  after this slice:
    error: unknown local or global: MAXSHORT
```

Next visible blockers include function-pointer arrays in `f_wipe.c`,
enum-sized arrays in several modules, old-style function definitions, and
missing Doom constants such as `MAXSHORT`.

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After MAXSHORT And Short Array Slice

`compile -S` now accepts the focused `r_segs.c` translation unit. This slice
adds the Doom-era `<values.h>` `MAXSHORT`/short/char limit macros and accepts
global `short` arrays through the current integer-array compile path, covering
extern declarations such as:

```c
extern short ceilingclip[SCREENWIDTH];
```

Regression coverage added:

```text
preprocessor_provides_doom_values_h_integer_limits
compiler_accepts_extern_short_array_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_segs.c \
  -o /tmp/c99inrust-r_segs.s
```

Focused `r_segs.c` compile now succeeds and emits assembly.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779137071`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779137071.txt
ok=20
fail=42
```

Translation units currently reaching assembly:

```text
am_map.c
d_items.c
d_main.c
d_net.c
doomdef.c
doomstat.c
dstrings.c
f_finale.c
i_main.c
m_argv.c
m_bbox.c
m_cheat.c
m_fixed.c
m_random.c
m_swap.c
r_draw.c
r_segs.c
r_sky.c
st_lib.c
tables.c
```

Representative moved blockers:

```text
FAIL r_segs.c
  before this slice:
    error: unknown local or global: MAXSHORT
  after this slice:
    OK r_segs.c

FAIL r_plane.c
  before this slice:
    error: 5666:1: unsupported function definition: R_FindPlane
  after this slice:
    error: 5511:18: expected unsigned char array length
```

Next visible blockers include function-pointer arrays in `f_wipe.c`,
enum-sized arrays in several modules, old-style function definitions, and
unsupported expression forms.

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After r_plane Pointer Return Slice

`compile -S` now accepts the focused `r_plane.c` translation unit. This slice
lets global array bounds use integer initializer expressions such as `320*64`
and accepts pointer-returning function signatures such as:

```c
visplane_t* R_FindPlane(...);
```

Regression coverage added:

```text
compiler_accepts_global_short_array_expression_length_slice
compiler_accepts_pointer_return_signatures
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_plane.c \
  -o /tmp/c99inrust-r_plane.s
```

Focused `r_plane.c` compile now succeeds and emits assembly.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779137932`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779137932.txt
ok=21
fail=41
```

Translation units currently reaching assembly:

```text
am_map.c
d_items.c
d_main.c
d_net.c
doomdef.c
doomstat.c
dstrings.c
f_finale.c
i_main.c
m_argv.c
m_bbox.c
m_cheat.c
m_fixed.c
m_random.c
m_swap.c
r_draw.c
r_plane.c
r_segs.c
r_sky.c
st_lib.c
tables.c
```

Representative moved blocker:

```text
FAIL r_plane.c
  before this slice:
    error: 5666:1: unsupported function definition: R_FindPlane
  after this slice:
    OK r_plane.c
```

Next visible blockers include function-pointer arrays in `f_wipe.c`,
enum-sized arrays in several modules, unsupported function parameters,
function-pointer assignments, and old-style function definitions.

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Enum Constant Array Slice

This slice lets enum constants participate in global and local array lengths,
global integer array initializer values, and unsized enum-typed global arrays
with initializer-derived lengths. The scan count is unchanged, but the enum
blockers now move to later unsupported forms.

Regression coverage added:

```text
compiler_accepts_global_enum_sized_int_array_slice
compiler_accepts_global_enum_sized_pointer_array_slice
compiler_accepts_local_int_array_global_enum_initializers_slice
compiler_accepts_unsized_global_enum_int_array_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/i_sound.c \
  -o /tmp/c99inrust-i_sound.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_enemy.c \
  -o /tmp/c99inrust-p_enemy.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/s_sound.c \
  -o /tmp/c99inrust-s_sound.s
```

Focused compiles still fail, but now fail after their enum constant blockers.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779138676`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779138676.txt
ok=21
fail=41
```

Representative moved blockers:

```text
FAIL i_sound.c
  before this slice:
    error: identifier NUMSFX is not an integer initializer
  after this slice:
    error: 4750:43: expected expression

FAIL p_inter.c
  before this slice:
    error: identifier NUMAMMO is not an integer initializer
  after this slice:
    error: unknown struct: player_s

FAIL p_enemy.c
  before this slice:
    error: expected unsigned char array length
  after this slice:
    error: 7052:15: expected punctuator ;

FAIL s_sound.c
  before this slice:
    error: identifier mus_e3m4 is not an integer initializer
  after this slice:
    error: pointer member access requires a typed pointer

FAIL st_stuff.c
  before this slice:
    error: identifier NUMCARDS is not an integer initializer
  after this slice:
    error: assignment to undeclared local or global: st_clock
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Enum Typedef Local Slice

The parser now records typedef names introduced by top-level enum typedefs and
uses them as scalar local declaration types while parsing supported function
bodies. This moves local declarations such as `dirtype_t d[3]` past the
previous parse error.

Regression coverage added:

```text
compiler_accepts_local_array_of_enum_typedef_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_enemy.c \
  -o /tmp/c99inrust-p_enemy.s
```

Focused `p_enemy.c` still fails, but now fails after the enum typedef local
array blocker.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779139236`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779139236.txt
ok=21
fail=41
```

Representative moved blockers:

```text
FAIL p_enemy.c
  before this slice:
    error: 7052:15: expected punctuator ;
  after this slice:
    error: 7262:39: expected punctuator )

FAIL p_ceilng.c
  before this slice:
    error: 6350:14: expected punctuator ;
  after this slice:
    error: unsupported function parameter

FAIL p_doors.c
  before this slice:
    error: 6581:14: expected punctuator ;
  after this slice:
    error: unsupported function parameter

FAIL p_floor.c
  before this slice:
    error: 6499:14: expected punctuator ;
  after this slice:
    error: unsupported function parameter

FAIL p_plats.c
  before this slice:
    error: 6680:14: expected punctuator ;
  after this slice:
    error: unsupported function parameter
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Enum Typedef Parameter Slice

Known enum typedef names are now accepted as scalar function parameter types.
This moves the `p_*` enum parameter blockers to later function-pointer cast
assignment forms.

Regression coverage added:

```text
compiler_accepts_enum_typedef_parameter_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_ceilng.c \
  -o /tmp/c99inrust-p_ceilng.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_doors.c \
  -o /tmp/c99inrust-p_doors.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_floor.c \
  -o /tmp/c99inrust-p_floor.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/p_plats.c \
  -o /tmp/c99inrust-p_plats.s
```

Focused compiles still fail, but now fail after their enum typedef parameter
blockers.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779139669`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779139669.txt
ok=21
fail=41
```

Representative moved blockers:

```text
FAIL p_ceilng.c
  before this slice:
    error: unsupported function parameter
  after this slice:
    error: 6502:47: expected punctuator ;

FAIL p_doors.c
  before this slice:
    error: unsupported function parameter
  after this slice:
    error: 6804:45: expected punctuator ;

FAIL p_floor.c
  before this slice:
    error: unsupported function parameter
  after this slice:
    error: 6573:46: expected punctuator ;

FAIL p_plats.c
  before this slice:
    error: unsupported function parameter
  after this slice:
    error: 6805:45: expected punctuator ;

FAIL p_switch.c
  before this slice:
    error: unsupported function parameter
  after this slice:
    error: pointer member access requires a typed pointer
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Draw Prototype Designator Slice

The parser now keeps prototype-only function names as function designators
separate from pointer-return metadata. The lowerer also maps typed
`ptr +/- integer` expressions to scaled pointer-offset IR before member
access. This moves Doom draw callback assignments and sprite-list pointer
arithmetic through assembly generation.

Regression coverage added:

```text
compiler_accepts_prototype_function_designator_assignment_slice
compiler_accepts_struct_pointer_arithmetic_member_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_main.c \
  -o /tmp/r_main.s

target/debug/c99inrust compile -S \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_things.c \
  -o /tmp/r_things.s
```

Both focused compiles now reach assembly generation.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779146521`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779146521.txt
ok=43
fail=19
```

Moved translation units:

```text
OK r_main.c
OK r_things.c
```

Remaining blockers:

```text
FAIL g_game.c
  error: 9119:5: expected expression
FAIL hu_lib.c
  error: member dereference requires a pointer
FAIL hu_stuff.c
  error: 5841:5: expected expression
FAIL i_net.c
  error: 3905:5: expected expression
FAIL i_sound.c
  error: 4750:43: expected expression
FAIL i_system.c
  error: 4552:5: expected expression
FAIL i_video.c
  error: unsupported function parameter
FAIL info.c
  error: translation unit has no supported function definitions
FAIL m_misc.c
  error: 5541:5: expected expression
FAIL p_mobj.c
  error: struct member value is not supported
FAIL p_setup.c
  error: assignment to non-pointer subscript targets is not supported
FAIL p_switch.c
  error: pointer member access requires a typed pointer
FAIL r_bsp.c
  error: 5355:5: expected expression
FAIL s_sound.c
  error: pointer member access requires a typed pointer
FAIL sounds.c
  error: 450:3: expected expression
FAIL st_stuff.c
  error: 8149:5: expected expression
FAIL v_video.c
  error: 5145:5: expected expression
FAIL w_wad.c
  error: 602:5: expected expression
FAIL wi_stuff.c
  error: unsupported global integer initializer
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After HU Double Pointer Parameter Slice

Parameter pointer referents now preserve pointer depth, so a parameter such as
`patch_t** font` is tracked as a pointer to `patch_t*` rather than directly to
`patch_t`. That lets indexed double-pointer member chains such as
`font[0]->height` lower as a pointer load followed by struct member access.

Regression coverage added:

```text
compiler_accepts_double_pointer_parameter_member_slice
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/hu_lib.c \
  -o /tmp/hu_lib.s
```

The focused compile now reaches assembly generation.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779147078`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779147078.txt
ok=44
fail=18
```

Moved translation unit:

```text
OK hu_lib.c
```

Remaining blockers:

```text
FAIL g_game.c
  error: 9119:5: expected expression
FAIL hu_stuff.c
  error: 5841:5: expected expression
FAIL i_net.c
  error: 3905:5: expected expression
FAIL i_sound.c
  error: 4750:43: expected expression
FAIL i_system.c
  error: 4552:5: expected expression
FAIL i_video.c
  error: unsupported function parameter
FAIL info.c
  error: translation unit has no supported function definitions
FAIL m_misc.c
  error: 5541:5: expected expression
FAIL p_mobj.c
  error: struct member value is not supported
FAIL p_setup.c
  error: assignment to non-pointer subscript targets is not supported
FAIL p_switch.c
  error: pointer member access requires a typed pointer
FAIL r_bsp.c
  error: 5355:5: expected expression
FAIL s_sound.c
  error: pointer member access requires a typed pointer
FAIL sounds.c
  error: 450:3: expected expression
FAIL st_stuff.c
  error: 8149:5: expected expression
FAIL v_video.c
  error: 5145:5: expected expression
FAIL w_wad.c
  error: 602:5: expected expression
FAIL wi_stuff.c
  error: unsupported global integer initializer
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Doom Libc And WAD Slice

The parser now accepts Doom's libc/network struct locals (`struct stat`,
`struct timeval`, `struct timezone`, `struct sockaddr_in`, and
`struct hostent*`), local `struct ...*` and `void*` declarations, variadic
function definitions with `...`, and the `va_list` scalar typedef. The
preprocessor supplies the socket/ioctl/errno constants used by the Linux
network backend, and IR lowering exposes `errno` as an external integer
binding. The parser also recognizes the local anonymous `name8` union used by
`W_CheckNumForName`.

Regression coverage added:

```text
compiler_accepts_doom_libc_struct_local_slice
compiler_accepts_errno_global_slice
compiler_accepts_variadic_function_definition_slice
compiler_accepts_local_void_pointer_declaration_slice
compiler_accepts_doom_name8_union_slice
preprocessor_provides_doom_socket_constants
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/i_net.c \
  -o /dev/null

target/debug/c99inrust compile -S \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/i_system.c \
  -o /dev/null

target/debug/c99inrust compile -S \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/m_misc.c \
  -o /dev/null

target/debug/c99inrust compile -S \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/w_wad.c \
  -o /dev/null
```

All four focused compiles now reach assembly generation.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779152692`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779152692.txt
ok=51
fail=11
```

Moved translation units:

```text
OK i_net.c
OK i_system.c
OK m_misc.c
OK w_wad.c
```

Remaining blockers:

```text
FAIL hu_stuff.c
  error: 5841:5: expected expression
FAIL i_sound.c
  error: 4750:43: expected expression
FAIL info.c
  error: translation unit has no supported function definitions
FAIL p_mobj.c
  error: struct member value is not supported
FAIL p_setup.c
  error: assignment to non-pointer subscript targets is not supported
FAIL p_switch.c
  error: pointer member access requires a typed pointer
FAIL s_sound.c
  error: pointer member access requires a typed pointer
FAIL sounds.c
  error: 450:3: expected expression
FAIL st_stuff.c
  error: 8149:5: expected expression
FAIL v_video.c
  error: 5145:5: expected expression
FAIL wi_stuff.c
  error: unsupported global integer initializer
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.

## Compile Scan After Full Doom Assembly Slice

The remaining Doom translation-unit blockers moved past assembly generation.
The latest slice adds coverage for Doom's timer/signal surface in `i_sound.c`,
including `ITIMER_REAL`, `SIGALRM`, `SA_RESTART`, `struct itimerval`, and
`struct sigaction`. It also covers global pointer-name initializers such as the
`wi_stuff.c` animation tables, global pointer and struct matrices, 2D struct
field arrays, forward struct typedef/tag linking, empty old-style parameter
definitions, braced char and byte initializers, and `X_OK` from `<unistd.h>`.

Regression coverage added:

```text
compiler_accepts_local_char_array_braced_initializer_slice
compiler_accepts_global_unsigned_char_numeric_matrix_initializer_slice
compiler_accepts_global_char_array_braced_initializer_slice
compiler_accepts_parenthesized_product_before_shift_slice
compiler_accepts_empty_parameter_function_definition_slice
compiler_accepts_forward_struct_typedef_then_tag_definition_slice
compiler_accepts_global_struct_array_arrow_decay_slice
compiler_accepts_global_struct_array_subscript_pointer_member_chain_slice
compiler_accepts_global_pointer_matrix_assignment_slice
compiler_accepts_global_sizeof_struct_array_initializer_slice
compiler_accepts_global_pointer_name_array_initializer_slice
compiler_merges_extern_and_defined_global_matrices_slice
compiler_accepts_global_struct_matrix_member_slice
compiler_accepts_struct_two_dimensional_array_field_assignment_slice
compiler_accepts_pointer_element_array_parameter_member_slice
preprocessor_provides_doom_timer_signal_constants
preprocessor_provides_doom_unistd_access_constant
```

Focused CLI QA:

```text
target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/st_stuff.c -o /tmp/st_stuff.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/wi_stuff.c -o /tmp/wi_stuff.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/d_net.c -o /tmp/d_net.s

target/debug/c99inrust compile -S -D NORMALUNIX -D LINUX \
  -I /tmp/c99inrust-doom-src/linuxdoom-1.10 \
  /tmp/c99inrust-doom-src/linuxdoom-1.10/r_main.c -o /tmp/r_main.s
```

All four focused compiles reached assembly generation.

Current compile scan was run inside tmux session
`c99inrust-doom-scan-1779162206`, then the session exited naturally without
`tmux kill-server`:

```text
scan=/tmp/c99inrust-doom-scan-1779162206.txt
ok=62
fail=0
```

Moved translation units:

```text
OK hu_stuff.c
OK i_sound.c
OK info.c
OK p_mobj.c
OK p_setup.c
OK p_switch.c
OK s_sound.c
OK sounds.c
OK st_stuff.c
OK v_video.c
OK wi_stuff.c
```

This is still not a playable Doom claim. Full success still requires compiling
all translation units, linking the Doom executable, and manually running a
playable public Doom target.
