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
