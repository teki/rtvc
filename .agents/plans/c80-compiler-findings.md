# C80 Compiler Findings

Last updated: 2026-11-14

Numbered findings are kept historically. Do not delete a finding because a later
path supersedes it; mark it resolved and point at the replacement.

## F-001 — Contract-readiness table (E00)

**Status:** open (table complete; no E00 semantic blocker)  
**Affected design:** [Frozen Implementation Contracts](c80-compiler.md#frozen-implementation-contracts)  
**Affected execution:** E00 gate; later increments consume the named contracts  
**Impact:** E01 has no unresolved language-contract prerequisite. Missing
implementation is not an open design choice.  
**Resolution authority:** frozen contracts F001–F006; E00 records owners only  
**Proposed resolution:** treat the table below as the E00 contract inventory.

| Topic | Contract owner | Specified now? | Pending decision? | Blocks |
| --- | --- | --- | --- | --- |
| Operator precedence/associativity | F001 | Yes | No | — |
| Integer literal contextual typing | F001 | Yes | Canonical hex spelling is an E01 documentation choice (`0x` is accepted and used throughout the design) | E01 docs, not E00 |
| Shift counts, wrapping, signed `>>` | F001 | Yes | No | — |
| Definite assignment, returns, recursion | F001 | Yes | Cross-unit recursion coverage is E07 work, not a new rule | E02 / E07 |
| Register and `@stackcall` ABI | F002 | Yes | No | E03 / E05 |
| Pointer lifetime / no borrow checker | F003 | Yes | Optimizer must not invent UB; dangling is warning + physical addressing | E06 |
| `str` prefix, `.len`, immutability | F003 | Yes | Exact “common” string escapes documented in E01 (`\\n \\r \\t \\\\ \\' \\" \\0 \\xNN`) | E01 lexer / E06 |
| Aggregates, packed structs, init | F003 | Yes | Local aggregates remain v1-rejected | E06 / E10 |
| Manifest, reservations, CLI, stack report | F004 | Yes | In-process metadata types may evolve; `--emit-map` deferred | E07 / E09 |
| Inline assembly operands/units | F005 | Yes | No | E08 (implemented) |
| BASIC substitutions and USR fixture | F006 | Yes | Stop only if verified ROM contradicts USR/LOMEM | E11 |
| Multiplication/division/remainder | Non-goals + F001 | Reject; multiplicative tokens may exist only for a diagnostic | No | E01/E02 diagnostics |
| Heap, implicit promotions, ROM wrappers | Non-goals | Reject | No | — |

**Resolved/open:** table recorded. No contradiction that invalidates execution order.

## F-002 — No `toml` crate in the current Cargo graph

**Status:** resolved in E07
**Evidence:** optional `toml` 0.8 plus `serde` sit on the `compiler` feature (`native`, `cli-tools`, `wasm-full`; not lightweight `wasm`). Manifests use serde `deny_unknown_fields`. Segment output stays a handwritten `rtvc-asm-v1` emitter matching [`rtvc_asm.rs`](../../src/bin/rtvc_asm.rs) enough for `loadasm`.
**Affected:** E07 manifest parse and segment output
**Impact:** none remaining
**Proposed resolution:** implemented as proposed.
**Resolution authority:** implementer (E07).

## F-003 — Design text that E01 “remains completed”

**Status:** resolved for this worktree  
**Evidence:** [Implementation Authority](c80-compiler.md#implementation-authority-and-remaining-work) says E01 parsing remains a completed increment. This tree has no `src/compiler/` and no parser tests.  
**Affected:** E01  
**Impact:** None. That sentence means later syntax is added in later increments, not that this checkout already contains a parser.  
**Proposed resolution:** implement E01 here from the execution plan.  
**Resolution authority:** execution plan E01; user assigned this tree.

## F-004 — Execution file names `gpt-5.6-terra`

**Status:** resolved  
**Evidence:** [c80-compiler-execution.md](c80-compiler-execution.md) assignment line vs user instruction to start in this session. The same file forbids spawning/switching models merely because the file exists.  
**Affected:** process only  
**Impact:** none on language or tests  
**Proposed resolution:** follow the user’s assignment in this session; do not delegate.  
**Resolution authority:** user instruction.

## F-005 — `asm::` test filter also matches `disasm::`

**Status:** resolved (discovery recorded; use a unique filter)  
**Evidence:** `cargo test --lib --no-default-features --features cli-tools asm::` listed 22 tests (15 assembler + 7 disassembler). `asm::tests::` still matches `disasm::tests::` because `disasm` contains the substring `asm`. Unique assembler run: `asm::tests:: -- --skip disasm` (15 passed). Unique disassembler: `disasm::tests::` (7 passed).  
**Affected:** E00 evidence rules; later assembler regressions  
**Impact:** A misspelled or overlapping filter can inflate counts. `compiler::` does not currently collide.  
**Proposed resolution:** Record assembler regressions as `asm::tests:: -- --skip disasm`. Never treat a 0-test `compiler::` run as a pass.  
**Resolution authority:** implementer / E00.

## F-006 — `cli-tools` does not currently enable `serde`

**Status:** resolved in E07
**Evidence:** `compiler = ["dep:toml", "dep:serde"]` and `cli-tools = ["dep:serde_json", "compiler"]`.
**Affected:** E07 manifest/segment serialization
**Impact:** none remaining
**Proposed resolution:** implemented; serde stays off lightweight `wasm`.
**Resolution authority:** implementer (E07).

## F-007 — Prologue ABI copies must not clobber later arguments

**Status:** resolved in E04  
**Evidence:** Sequential `LD C,A` then `LD B,C` for `(bool, bool)` overwrote the second argument. Nested `if (a) if (b)` returned the true branch when `b` was false.  
**Affected:** E04 register homes for multi-block functions; E05 parallel ABI moves  
**Impact:** Argument rehomes after entry must be ordered so a destination is not a still-pending source (cycle-breaking belongs to E05).  
**Proposed resolution:** emit pending ABI-to-stable moves in dest-not-a-source order. Implemented in `assign_stable_homes`.  
**Resolution authority:** implementer (E04).

## F-008 — IX frame size 128 is a valid `-128` displacement

**Status:** resolved in E05  
**Evidence:** `-(frame_used as i8)` panics in debug when `frame_used == 128` because `128 as i8` wraps to `-128` and negation overflows. A 128-byte frame is encodable as `IX-128`.  
**Affected:** E05 indexed-frame boundary  
**Impact:** Reject only sizes above 128; convert through `i16` (`i8::try_from(-(next as i16))`).  
**Resolution authority:** implementer (E05).

## F-009 — Harness stack high-water must ignore SP above entry

**Status:** resolved in E05  
**Evidence:** `RET` raises SP above the function entry SP. `entry.wrapping_sub(sp)` then looks like a huge depth (`65534`) and inflates `sp_used`.  
**Affected:** T05 bound vs high-water comparison  
**Impact:** Count depth only when `0 < entry.wrapping_sub(sp) < 0x8000`.  
**Resolution authority:** implementer (E05).

## F-010 — Untyped integer beside a pointer is an offset, not a pointer

**Status:** resolved in E06  
**Evidence:** `p = p + 1` typed the literal `1` with the assignment's `ptr<T>`
expected type (`integer literal used in a non-integer context`). F003 says a
pointer ± integer offset is scaled addressing, not a general conversion.  
**Affected:** E06 pointer arithmetic  
**Impact:** When the typed peer is a pointer, give the untyped integer `u16`
context, then scale.  
**Resolution authority:** implementer (E06).

## F-011 — `ADD HL,imm` must not require a second register pair

**Status:** resolved in E06  
**Evidence:** `p = p + 1` in a loop with byte locals occupying `BC` and `p` in
`DE` failed with register pressure because `word_src_rr` tried to materialize
the immediate in `DE`/`BC`.  
**Affected:** E06 pointer walks / T06 buffer-fill fixture  
**Impact:** Immediate word adds use `INC HL` for 1/2 and `ADD`/`ADC A` for other
constants, so a pointer increment can stay in registers without an IX frame.  
**Resolution authority:** implementer (E06).

## F-012 — Asm headers need a lone `:` token and a raw body slice

**Status:** resolved in E08  
**Evidence:** The lexer only emitted `ColonColon`. `in: a = x` cannot parse
without a single-colon token. Assembler `$` prefixes and `;` comments inside
`asm { }` are not C80 tokens; lexing the body as C80 produced diagnostics.
**Affected:** E08 parser  
**Impact:** Tokenize `:` separately from `::`. Copy the source between matching
braces (treating `@{...}` as a marker) and drop lexer diagnostics that fall
inside that span.  
**Resolution authority:** implementer (E08).

## F-013 — IX-frame prologue must not destroy incoming `HL`

**Status:** resolved in E08  
**Evidence:** `LD HL,-n / ADD HL,SP / LD SP,HL` ran before ABI-to-home copies.
`add_carry(0xFFFF, 1)` captured CF=0 because `left` in `HL` had already been
replaced by the frame pointer. `ADD HL,DE` itself was correct.  
**Affected:** any IX-frame function with an `HL` (or `H`/`L`) argument, including
inline asm flag capture  
**Impact:** Stash `HL` in a free `DE`/`BC`, or `DEC SP` n times when both pairs
are incoming.  
**Resolution authority:** implementer (E08).

## F-014 — `LDIR` with `BC=0` may return instead of hitting the insn limit

**Status:** resolved in E08 (test contract)  
**Evidence:** Raw `LDIR` copies 65536 bytes and overwrites the function, then
hits a `RET`. The harness returned `Ok` with many `DataWrite`s rather than a
timeout.  
**Affected:** T08 zero-count LDIR  
**Impact:** Assert extra memory writes, not `timed_out`. Do not special-case
`BC=0` in codegen.  
**Resolution authority:** implementer (E08).

## F-015 — In-place 8-bit ALU clobbers a live binary `lhs` in `A`

**Status:** resolved in E10  
**Evidence:** `a[i++] += 5` lowered the old index into a vreg still live across
the `ADD`, but `ADD A,n` reused `A` for the byte result. The later store used
the clobbered index (`a[1]`).  
**Affected:** T10 dest-once compound/inc  
**Impact:** Copy a still-live 8/16-bit binary `lhs` off `A`/`HL` before in-place
ALU (`park_live_src`). Variable `enemies[i]` indexing in a tight register leaf
can still trip register pressure; use a pointer walk (`p += 1`) or a constant
index for stride tests.  
**Resolution authority:** implementer (E10).

## F-016 — BASIC `-32768` is not a typed integer literal

**Status:** resolved in E11 (fixture spelling)
**Evidence:** `LET R=USR(addr,-32768)` stops the interpreter; `42`, `-1`,
`32767`, and `-32767` round-trip through USR/`HL`. Negating `32768` overflows
the BASIC 1.2 integer range before the call.
**Affected:** T11 USR signed-boundary fixture
**Impact:** Use `-32767` as the BASIC-source negative boundary. `i16` `0x8000`
remains representable in HL if constructed without that literal.
**Resolution authority:** implementer (E11); ROM contract unchanged.

## F-017 — Taken `JR` costs more T-states than `JP`; `DJNZ` wraps at zero

**Status:** resolved in E12 (policy)
**Evidence:** `JR` taken is 12 T-states vs `JP` 10; not-taken `JR` is 7 vs 10.
Converting backward loop edges or the `JP NZ, body` enter-jump to `JR` grew
`count(5)` from 371 to 380 T-states. `DJNZ` with `B=0` iterates 256 times, but
`while (n != 0)` with `n == 0` must iterate zero times.
**Affected:** E12 branch shortening; T12 loop fixtures
**Impact:** Collapse `JP cc, then; JP else` to `JP !cc, else` when `then` is
next, convert only forward conditional `JP` to `JR`, and keep backward/
unconditional `JP`. Do not emit `DJNZ`.
**Resolution authority:** implementer (E12); matches design “prefer fewer
T-states without growing bytes” and the deferred-DJNZ note.

## C80-FIND-021 — Mixed project output is the segment TOML

**Status:** resolved in E11 revalidation  
**Evidence:** Earlier E11 treated tokenized BASIC as in-process only. The
clarified F004/F006 contract is one `rtvc-asm-v1` TOML containing C80/ASM
and the BASIC payload. CAS remains E13 (`rtvc-tocas`). E11 must be
revalidated against the serialized mixed-image procedure before reaffirming
E12.  
**Affected:** E07 artifact wording; E11 `--emit-segments` / T11 USR fixture  
**Impact:** Attach BASIC bytes and `project::basic_base` /
`project::basic_program_size` to the existing linker image. Tests load that
TOML. Do not add a relocation loader or CAS emit in E11. E13 later packs
that TOML through `rtvc-tocas`.  
**Resolution authority:** user plan correction; implementer (E11).

## C80-FIND-022 — CAS flatten is the 19EFH load profile

**Status:** resolved in E13  
**Evidence:** Mixed tutorial/pong images keep BASIC at `19EFH` (`[basic]`
has `path` and `size` only) and C80 at `3000H` so `rtvc-tocas` can flatten
them. Tape inject always writes `19EFH`. Instant `load_cas` copies the linear
payload and, when the payload starts with tokenized BASIC, sets TEXT/CHAIN/TOP
(`1722H`/`1724H`/`1726H`) so `LIST`/`RUN` see the injected program. CLI `-i`
types `RUN` when the CAS autostart byte is set.  
**Affected:** E13 `rtvc-tocas` TOML path; T13 cassette USR  
**Impact:** No relocating loader. `[basic]` is a path; BASIC is `19EFH` and
must not overlap linked modules.  
**Resolution authority:** design CAS follow-up; implementer (E13).

## C80-FIND-023 — Pong vsync must ACK before waiting

**Status:** resolved in pong example  
**Evidence:** `setup()` does `DI` then a long `LDIR`. Port `59H` bit 4
(cursor/sound, active low) can stay pending because nothing ACKs it. Waiting
for idle then a falling edge deadlocks in the idle poll. ACK with
`OUT (07H)` first, wait until bit 4 is pending, ACK again. Reprogram CRTC
R10/R11=3 and R14/R15=`0x0EFF` so the cursor still ticks at 50 Hz after
VT-DOS (`0x0AFF`). Injected `pong.cas` plus `RUN` then serves on Space and
moves the left paddle on joystick up.  
**Affected:** `info/c80/pong/pong.c80` `wait_frame` / `setup`  
**Impact:** Any C80 bitmap loop that `DI`s must ACK bit 4 before polling.  
**Resolution authority:** pong playtest after E13.

## C80-FIND-024 — Native and wasm-full no longer enable `compiler`

**Status:** resolved  
**Evidence:** E00 proposed `compiler` on `native`, `wasm-full`, and `cli-tools`.
The emulator and full-web UI never call the compiler, so a C80 compile error
blocked `cargo build --bin rtvc`. `native` and `wasm-full` no longer enable
`compiler`. `cli-tools` still does. Direct `--features compiler` builds
`rtvc-c80` without the desktop UI. Tutorial `compile.sh` / `compile.bat` pass
`--no-default-features --features compiler`.  
**Affected:** [Cargo.toml](../../Cargo.toml) features; native/wasm-full size;
C80 CLI commands  
**Impact:** `cargo run --bin rtvc-c80` with default features is skipped until
`compiler` is enabled. Lightweight `wasm` was already without C80.  
**Resolution authority:** user request to make C80 standalone of the emulator.

## C80-FIND-025 — C80 is a separate workspace crate

**Status:** resolved  
**Evidence:** Feature-gating still compiled C80 whenever `compiler`/`cli-tools`
was on, and rust-analyzer/`cargo check` of the `rtvc` package could still see
`src/compiler/`. The compiler now lives in [`c80/`](../../c80/) as package
`rtvc-c80`. Workspace `default-members = ["."]`, so `cargo build` does not
build C80. `parse_rtvc_asm_v1` moved to `rtvc_core::asm` (`asm-toml` feature)
so `rtvc-tocas` does not depend on the compiler crate.  
**Affected:** workspace layout; `cargo run -p rtvc-c80`; tutorial compile scripts  
**Impact:** A C80 type error cannot fail `cargo build --bin rtvc`. Use
`cargo test -p rtvc-c80` for compiler tests.  
**Resolution authority:** user request to split C80 from rtvc.

## F-018 — In-place ALU destroys a live local that shares `A`/`HL` (D1)

**Status:** resolved in H1 (occupant parking and scratch copies; guard)
**Evidence:** `u8 f(u8 a, u8 b) { u8 s = a + b; u8 t = a & b; return s - t; }`
lowers to `ADD A,C / AND C / LD C,A / SUB C / RET` and always returns 0. The
plain expression `u8 f(u8 a, u8 b) { return (a + b) + (a + b); }` is
`ADD A,C / ADD A,C / LD C,A / ADD A,C` (f(0,1) = 4), and the word form does the
same in `HL`. `IrOp::LoadLocal` binds the vreg to the local's own location;
`park_live_src` only asks whether the vreg is live, so the local is
overwritten. The signed byte compare path has the same defect: its in-place
`XOR 80H` on `A` corrupts a live local
(`u8 f(i8 a, i8 b) { bool x = a < b; return u8(a) + u8(x); }`, f(1,2) = 82H).
The H0 suite records 291 `WrongValue` cases with this cause (every shape
group), plus 93 of 99 random `WrongValue` programs that pass once all
variables are globals.
**Affected:** `lower_binary8`, word binary, compare lowering, `park_live_src`
in [lower.rs](../../c80/src/lower.rs); H0 `shape.*`, `prog.d1_*`
**Impact:** Any single-block, frame-free function that reuses a parameter or
local after an operation on it can be miscompiled silently.
**Resolution authority:** [c80-hardening.md](c80-hardening.md) H1 items 1–2.
**Resolution:** every in-place lowering works on a scratch copy
(`scratch_a`/`scratch_hl`). Values still live after the operation leave the
register first (`park_live`), moving with every alias at the same location,
and the register stays reserved for the rest of the operation. A store into a
local's home moves other live copies of the old value out first. When `HL`
cannot be freed, word `+ - & | ^` runs through `A` into the pair of an
operand that dies. The structural guard in `Lowerer::emit` reports any
remaining overwrite as `cg-internal`.

## F-019 — Call spill slots collide (D2)

**Status:** resolved in H1
**Evidence:** `add(sub(x, y), add(add(x, 1), sub(y, 2)))` with `x=50, y=8`
returns 108, not 99; f(0,0) returns 0, not FFH. Both live call results are
stored to `(IX-18)` because `lower_call` restarts `next_spill` at the base of
the spill area for each call.
**Affected:** `lower_call` in [lower.rs](../../c80/src/lower.rs); H0
`prog.nested_calls`
**Impact:** Nested calls with more than one pending result return wrong
values.
**Resolution authority:** [c80-hardening.md](c80-hardening.md) H1 item 3.
**Resolution:** `alloc_spill` takes the first slot in the spill area that no
current binding (vreg, local, or pending capture) uses, so a slot belongs to
its value until the value dies. Keys at one location share a slot. The area
size is unchanged. If it is too small, the function is rejected rather than
miscompiled. Regression: `prog.d2_calls_keep_spills`.

## F-020 — Loop-carried register local is not saved across a call (D3)

**Status:** resolved in H1
**Evidence:** In `while (n != 0) { n = n - 1; s = g(s) + 1; }` the counter `n`
stays in `C` (byte) or `DE` (word) across `CALL g`. A callee that clobbers
them ends the loop early: f(3) = 1 for the byte loop, f(5) = 1 and 3 for the
two clobbering-callee fixtures. `liveness_fn` records one linear last use per
local, so a use before the call that is reached again through the back edge
is treated as dead.
**Affected:** `liveness_fn`, `locals_needing_frame`, `live_across_any_call`,
`lower_call`; H0 `prog.d3_loop_call`, `prog.clobbering_callee_loop`,
`prog.clobbering_callee_loop_word`
**Impact:** Any loop with a call and a register-resident counter depends on
what the callee happens to touch. The plan's D3 repro (`s = g(s)` with
`ld bc,0`) returns the right value by accident; the suite adds `+ 1` so the
early exit is visible.
**Resolution authority:** [c80-hardening.md](c80-hardening.md) H1 item 4.
**Resolution:** `local_live_after` computes per-op live-after sets of locals
by a backward fixed point over the CFG. `live_across_any_call`,
`locals_needing_frame`, `drop_dead_after`, and the call, `ldir`, and
inline-asm save loops use it. The H0 fixture
`prog.clobbering_callee_loop_word` also expected `900 + 44` for `n = 300`;
the loop adds 2 per iteration, so the right value is `600 + 44`. That
expectation is corrected.

## F-021 — Shifts never compile (D4)

**Status:** resolved in H3
**Evidence:** `check_binary` runs the same-type check before the `u8` count
rule, so every shift whose left operand is not `u8` is rejected as
`ty-mismatch: operands have different types` — `i8` included, not only words.
`u8` shifts and every compound `<<=`/`>>=` (whose count is already typed `u8`)
pass typing and fail in codegen with `unsupported byte operator` /
`unsupported word operator`. The H0 ledger has 72 typing and 32 codegen
entries; the bubble-sort and shift-add-multiply fixtures need shifts too.
**Affected:** `check_binary` in [semantics.rs](../../c80/src/semantics.rs);
`lower_binary8` and the word binary path in [lower.rs](../../c80/src/lower.rs)
**Impact:** `<<`/`>>` are documented as accepted but are unusable.
**Resolution authority:** [c80-hardening.md](c80-hardening.md) H3.
**Resolution (H3):** shifts type and compile for all four integer types,
constant and variable counts, plain and compound, in any place. The count
is `u8`; a typed non-`u8` count is "shift count must be u8, found T" at the
count. Constant counts take the fastest of a few templates (table in the
H3 section of [c80-hardening-progress.md](c80-hardening-progress.md));
variable counts loop behind a zero guard (`INC c / DEC c / JR Z`, then
`DJNZ` or `DEC c / JR NZ`). Counts above the width run all their steps and
are not clamped: the result is already the F001 value after `width` steps
(the argument is in the progress file), and a clamp would cost every
in-range shift a compare. Every shift ledger entry (104 operator cases,
`bubble_sort`, `shift_add_multiply`, the `rshift` random rows) passes.
Compiler-generated address scaling uses the same shifts.

## F-022 — "Register pressure" rejects ordinary functions (D5)

**Status:** resolved in H2a; strict `@fastcall` rejection cases added in H2b
**Evidence:** `u16 f(u16 a, u16 b) { u16 c = a + b; return a + b + c; }` and
11 of the 16 Appendix A fixtures (array index/store, struct array, the
many-locals fixtures, call in a loop, `@stackcall` with five arguments,
pointer walk, signed loop, bubble sort, shift-add multiply), plus the
shift-free bubble sort and the run-time linked list, fail with `register pressure in 'f' needs a larger IX frame or
@stackcall`. It is 114 `KNOWN_GAPS` entries and almost all random rejections:
the depth-4 random family compiles 4–37 of 300 programs per type, and even
depth-2 word programs are 91–93 % rejected. `relocate` can only move values
between registers; only `DE`/`BC` are word homes.
**Affected:** `relocate`, `plan_homes`, `assign_stable_homes`,
`fits_stable_registers` in [lower.rs](../../c80/src/lower.rs)
**Impact:** Most real functions do not compile; `@stackcall` does not help.
**Resolution authority:** [c80-hardening.md](c80-hardening.md) H2.
**H1 progress:** 10 `KNOWN_GAPS` pressure entries remain (from 114). The
H1 changes that helped:
- A frame-free lowering that runs out of registers is retried with an IX
  frame (`lower_function`).
- A single-block function is no longer sent to IX up front by the stable-home
  count.
- A local that is dead at a point does not occupy its register.
- `relocate` can free a pair by moving the byte values that sit in it.
- Word `& | ^` take constant operands directly.
- Equality compares are symmetric and unbiased.

Random rejections are still mostly pressure.
**H2a resolution:** IX frames are gone from register-ABI functions. Values
that do not fit go to `PUSH`/`POP` around calls, `LDIR` and clobbering `asm`
blocks, or to per-function static slots after the function (spill on
eviction by farthest next use, reload on use; slot homes for locals without
a register and for address-taken locals). All 10 remaining pressure entries
and every pressure-caused random rejection pass; `prog.bubble_sort` and
`prog.shift_add_multiply` are now rejected only for shifts (F-021). The
diagnostic remains only where no register can be freed at all (see
[info/c80.md](../../info/c80.md)); no conformance case reaches it. Details
and the cost table are in [c80-hardening-progress.md](c80-hardening-progress.md).
**H2b:** every former pressure source (8 `prog` fixtures: u16 array index
and store, struct array, seven word locals, call in a loop, signed loop,
shift-free bubble sort, run-time linked list) has a `@fastcall` variant in
`STRICT_REJECTIONS` ([conformance_tests.rs](../../c80/src/conformance_tests.rs)).
The plain variant must pass and the strict one must be rejected with "must
fit in registers", in both modes.

## F-023 — Address of a global in a static initializer

**Status:** resolved in H3a (language decision by the user, 2026-11-13)
**Evidence:** `struct N { u8 v; ptr<N> next; }; N c = {3, ptr<N>(0)};
N b = {2, &c};` is rejected with `ty-mismatch: static initializer must be a
compile-time value`. The language reference neither allows nor forbids
address-of-global in static initializers. Building the same list at run time
works since H2a (`prog.linked_struct_runtime`; it was rejected for register
pressure, F-022).
**Affected:** `eval_const_expr` / static initializer in
[semantics.rs](../../c80/src/semantics.rs); H0 `prog.linked_struct_static`
**Impact:** Static linked data needs run-time setup.
**Proposed resolution:** decide whether `&global` (and `&global[k]`,
`&global.field`) is a link-time constant; until then it stays a `Rejected`
ledger entry.
**Resolution authority:** user (language scope).
**H3a:** a static initializer (global or `static` local, scalar or inside
braces) may hold a link-time address: `&global`, `&array[K]`,
`&global.field`, `&array[K].field`, `&unit::global`, in `ptr<U>(...)` or
`u16(...)`, typed like the run-time `&`. `lower_global` emits
`DW label+offset`; the helper assembler already resolved forward and
cross-segment labels in `DW` (test
`helper_assembler_resolves_forward_and_cross_org_word_references`), so the
root crate is unchanged. `CompilerMap::data_refs` names each target.
`prog.linked_struct_static` passes and `KNOWN_GAPS` is empty.

## F-024 — Signed byte compare reads a stale scratch register

**Status:** resolved in H1
**Evidence:** `bool f(i8 a, i8 c) { return c >= i8(i16(a)); }` returns 1 for
f(1, 0). Output: `LD L,A / ADD A,A / SBC A,A / LD H,A / LD A,C / XOR 128 /
LD B,A / LD C,A / LD A,L / XOR 128 / LD B,A / LD A,B / CP B`. In
`lower_compare_value` the flipped left operand is copied to `tmp` (`B`), then
`ensure_a(rhs)` relocates the left operand's binding out of `A` to `C`, which
frees `B`; the right operand is then allocated to `B` and the final
`LD A,tmp` reads the right operand. `c >= i8(w)` with `w: u16` fails the same
way. Equal-typed operands (`c >= a`) pass by allocation luck. The flipped
value is also bound as the operand's location although it no longer holds
the operand.
**Affected:** `lower_compare_value` (signed byte branch) in
[lower.rs](../../c80/src/lower.rs); H0 `prog.f024_*`
**Impact:** Signed byte comparisons whose operands come from casts or other
registers can return the wrong result. The H1 clobber guard would not see
this (no live value is overwritten; the code reads the wrong register).
**Proposed resolution:** keep the flipped copy as an unbound scratch value,
or re-read `tmp`'s current location after `ensure_a(rhs)`; add an assertion
that `tmp` is still owned by the left operand before the `CP`.
**Resolution authority:** implementer (H1).
**Resolution:** the compare biases scratch copies only. The right operand is
copied to `A`, flipped, and moved to a reserved free register; the left
operand is then copied to `A` and flipped; `CP tmp` follows. That is 8 bytes
instead of 11, and nothing is rebound. A signed word compare flips its right
operand in place only when nothing live shares the pair. Otherwise it flips a
copy in a free pair. With no free pair it flips the operand in place and flips
it back behind `PUSH AF`/`POP AF`. Equality compares skip the bias.

## F-025 — In-place word op destroys a live byte view of `L`

**Status:** resolved in H1
**Evidence:** `u8 f(u16 a) { u8 x = u8(a); u16 y = a + 1; return x + u8(y); }`
returns `2a + 2` instead of `2a + 1`; `bool f(u16 a) { return u8(a) <
u8(a + 1); }` lowers to `INC HL / LD A,L / CP L` (always false). The narrowing
cast binds the byte value to `L` of `a`'s `HL`; `a + 1` is then done in place
on `HL` because `a` itself is dead, although the byte view is not.
**Affected:** `add_hl_imm`, word binary, and `park_live_src` in
[lower.rs](../../c80/src/lower.rs); H0 `prog.f025_*`
**Impact:** Same class as F-018 but through a sub-register alias: the H1
occupant test must treat `H`/`L` occupants as occupants of `HL` (and the
reverse) or this survives the D1 fix.
**Resolution authority:** [c80-hardening.md](c80-hardening.md) H1 items 1–2.
**Resolution:** occupancy is per 8-bit register, so a byte view in `L` is an
occupant of `HL`. Parking moves it, and when a word moves, byte views of its
halves are rebound to the new pair.

## F-026 — Stable register homes overlap

**Status:** resolved in H1
**Evidence:** `fits_stable_registers` counted bytes (at most 6) and words (at
most 2) separately, and the byte and word home lists share `C B E D`. A word
local and a byte local could therefore get the same register.
`u16 f(u16 a, u8 b) { u8 c = b; u16 d = a; u8 e = 0; while (e < b) { d += a; e += 1; } return d + u16(c); }`
hit the H1 guard. The tutorial project
[mixed/buf.c80](../../info/c80/mixed/buf.c80) was miscompiled the same way:
`sum` had `p` in `BC` and `n` in `C`, so `LD C,16` destroyed `p`. No test
checked those results; the mixed USR test only calls `echo`.
**Affected:** `fits_stable_registers`, `assign_stable_homes`, `plan_homes`
in [lower.rs](../../c80/src/lower.rs)
**Resolution:** `HomeRegs::plan` assigns homes without overlap: words take
`DE BC` first, then bytes take the remaining registers (`L H` only when
frame-free). When everything fits frame-free, one pointer local gets `HL`. A
parameter that is never read gets no home. Regressions:
`prog.f026_overlapping_homes`, and `prog.f026_tutorial_buf`, which runs
`fill`/`bump`/`sum`. The fixed `buf.c80` is smaller, at 124 bytes instead of
159. `fill` and `bump` stay frame-free, and `sum` keeps the 1-byte IX frame
it already had.
**Resolution authority:** implementer (H1).

## F-027 — Spellings for pinned registers, `static` locals, strict `@fastcall`

**Status:** decided and implemented in H2b
**Question:** [c80-hardening.md](c80-hardening.md) H2 items 9 and 10 propose
`u8 n @ b;` for a pinned register and `static T x;` / `static T a[N];` for
memory locals, to be checked against attribute `@name`, the `@{...}` marker
syntax, and inline-`asm` headers.
**Checked:**
- `@` is one token (`TokenKind::At`). Today it is legal only before a
  function (attributes, before or after `pub`). A pin comes after the
  declared name, a position where `@` is currently a parse error, so one
  token of lookahead tells the two apart. Whitespace does not matter to the
  lexer (`n @ b` and `n @b` are the same tokens).
- `@{unit::export}` / `@{project::name}` markers exist only in ASM unit text
  and BASIC source, and inside inline-`asm` bodies, which the parser skips as raw
  text. A pin is never inside a body, so the two
  cannot meet. A `{` after `@` in a pin position is a plain parse error.
- Inline-`asm` headers use `in: b = expr` / `out: b = lvalue`: register
  names as contextual identifiers after a clause keyword, with no `@`. Pins
  reuse exactly that register list (`a b c d e h l bc de hl`, lowercase),
  as contextual identifiers after `@`. A variable may still be called `b`.
- `static` is not used as an identifier in any example or test.
**Decision:**
- Pin: `type name @ reg` on a local or a parameter, before any array length
  or initializer: `u8 n @ b = 16;`, `void f(u8 x @ c)`. Canonical spelling
  has spaces around `@`, so it does not read like an attribute. The
  attribute position (before the type) and the pin position (after the
  name) never overlap, so the grammar stays LL(1). Rejected alternatives:
  `u8 n : b` (reads like a type annotation or an asm clause), `u8 n in b`
  (`in` is an asm clause keyword and `cpu::in`), `register(b) u8 n` (longer,
  C-like, and a new keyword for no gain).
- Memory local: `static type name [ "[" N "]" ] [ "=" const-init ] ;` inside
  a function body. `static` is a keyword. It is not allowed at top level
  (globals are already static), in a `for` initializer (the initializer runs
  once, not per loop), with `const`, or with a pin.
- Strict functions keep the existing spelling `@fastcall`; it is now a flag
  of its own (`Function::strict_registers`) instead of an alias.
**Resolution authority:** implementer (H2b), per the plan's instruction to
fix spellings in a finding.
**Implemented as decided**, plus two rules found while writing the
examples:
- A pinned local's register is free for computing the pin's own new value
  while the old value is dead and a store into the pin later in its scope
  redefines it. Without this, `s = s + a` with `s @ hl` was rejected ("needs
  hl for something else") although `ADD HL,DE` in place is exactly what the
  pin asks for. A pin is otherwise never moved, spilled, or shared.
- A static's non-constant initializer reports one error and the static is
  still declared (zeroed), so later uses do not cascade into "unresolved
  name".

## F-028 — Spelling and meaning of `@preserves`

**Status:** decided and implemented in H2c
**Question:** [c80-hardening.md](c80-hardening.md) H2 item 11 proposes
`@preserves(bc, de)`. How does it sit with the other attributes and pins,
what can be listed, and what does "preserved" mean for code the compiler
generates itself?
**Decision:**
- **Spelling:** `@preserves(reg, ...)`, an attribute in the same positions as
  `@fastcall`/`@stackcall` (before or after `pub`), combinable with either.
  Register names are the pin and inline-`asm` list (`a b c d e h l bc de
  hl`). Halves and pairs normalise: `b, c` is `bc`. An empty list, an
  unknown name, and a second `@preserves` are `parse-expected` errors.
- **Result registers:** a byte or `bool` result travels in `A`, a word in
  `HL`, so those cannot be listed (`ty-mismatch`: "'f' returns u8 in a, so
  it cannot preserve a"). A `void` function may preserve `a` and `hl`.
- **Parameters:** a parameter that arrives in a preserved register is the
  caller's value. It may be read in place and comes back unchanged. A pin on
  a preserved register is an error, since a pin's register is written.
- **Meaning for codegen:** each preserved register is a pinned pseudo-value
  for the whole body. The allocator never uses it for homes or scratch. An
  operation that needs it is an error at that statement. A call or `asm`
  block that may change it is saved with a marked `PUSH`/`POP` in a default
  function, and is an error in a strict `@fastcall` one. The H1 guard sees
  it as always live, and the run-time harness check catches anything else.
- **Callers:** a call clobbers the callee's non-preserved registers plus the
  argument and result registers. A value live across the call stays in a
  preserved register at no cost. Moving a value into such a register is
  an `LD r,r` marked `; spill`, and strict functions allow it, because it
  uses no stack or memory.
- **Inference** (item 3 as narrowed by item 11): only for functions without
  `pub` and without `@preserves`.
  - The contract is the set of registers the lowered code never writes.
    Calls count their callee's clobbers, `asm` blocks their declared
    registers, and result registers never count as preserved.
  - Lowering runs callees first: a post-order DFS from functions in source
    order. This is deterministic, and a function's contract depends only on
    its own code and its callees.
  - A private function can only be called from its own unit.
**Rejected alternatives:**
- `@clobbers(...)`: it lists the complement, and an omission is unsafe.
- A preserves list inside the parameter list: it mixes ABI and contract.
- Inferring for `pub` functions: an edit in one unit would silently change
  code in another.
**Resolution authority:** implementer (H2c), per the H2c brief.

## F-029 — C80 cannot call ASM unit exports

**Status:** open (found in H2c); the inline-asm marker is a located error since H3
**Evidence:**
- In a project with an ASM unit `boot` that exports `start`,
  `import boot; void f() { boot::start(); }` reports `ty-unresolved-name:
  unknown unit 'boot'` twice. `collect_exports` only reads C80 units.
- `asm (clobber: a) { CALL @{boot::start} }` reaches the assembler with the
  marker still in it. The result is `cg-internal: assembler rejected
  generated code: line 7: invalid mapped expression '@{BOOT::START}'`, an
  internal error located at `boot.asm:1:1`.
  - `@{...}` substitution only runs on ASM unit text and BASIC.
  - F-027 described markers as appearing "inside inline-`asm` bodies"; they
    are not substituted there.
**Affected:** [project.rs](../../c80/src/project.rs) (`collect_exports`,
marker substitution), the inline-asm lowering.
**Impact:**
- The manifest register contract for ASM exports (H2 item 11) has nothing
  to attach to, so it was not added.
- A user error is reported as `cg-internal`, at the wrong file.
**Resolution authority:** a later increment. Either typed extern
declarations for ASM exports (with `stack_extra` and a declared contract),
or `@{unit::export}` substitution in inline `asm`. Until then, at least
reject the marker in inline `asm` with a located user error.
**H3:** the marker in inline `asm` is now a located `cg-unsupported` error
at the marker in the C80 source ("markers such as '@{boot::start}' are not
substituted in inline asm; ..."). No substitution was added; calling ASM
exports from C80 stays open.

## F-030 — A unit that references another unit gets no JR shortening

**Status:** resolved in H3i (c80 only; the assembler is unchanged)
**Evidence:** in a two-unit project (`main` at `0x8000`, `lib` at `0x8100`),
`u8 f(u8 x) { if (x == 0) { x = 5; } return x; }` in `main` compiles to
`... CP 0 / JR NZ,...`, but `if (x == 0) { x = lib::g(); }` keeps
`JP NZ,...` (`C2`). The optimizer's `shorten_jumps` assembles one unit's
chunks alone (`assemble_current`); a label of another unit is undefined
there, the assembly fails, and `shorten_jumps` returns `None` without a
diagnostic. Any cross-unit call or global access, and now any H3a
`DW other_unit_label`, turns JR shortening off for the whole unit.
**Affected:** [optimize.rs](../../c80/src/optimize.rs) (`shorten_jumps`,
`assemble_current`), [project.rs](../../c80/src/project.rs) (per-unit
`optimize_chunks`).
**Impact:** larger, slightly slower code in multi-unit projects; values are
correct (the JP form is always valid).
**Proposed resolution:** give the per-unit assembly the other units' labels
as placeholder `EQU`s and never convert a jump whose target is one of
them, or shorten after the project-wide assembly. A c80-only change; the assembler is fine.
**Resolution authority:** implementer (H5 or a small fix after review).
**Resolution:** the first option. `project.rs` collects the labels each
unit defines and passes the other units' labels to `optimize_chunks` as
`externals`. `assemble_current` appends `LABEL EQU 0` for each (an `EQU`
emits no line, so the line-to-address pairing is unchanged), and
`shorten_jumps` never shortens a jump whose target is one of them: its
distance in the real layout is unknown there. Single-file builds pass none.
Test: `unit_calling_another_unit_still_gets_jr` (a `CALL lib::g` and a
`JR NZ` in the same function; runs). No golden changed: the `project/` and
`mixed/` examples have no branch that was kept long by this.

## F-031 — Spelling and meaning of `@inline` and `@noinline`

**Status:** decided and implemented in H3i
**Question:** [c80-hardening.md](c80-hardening.md) H3i adds `@inline`. How
does it combine with the other attributes, what do `@preserves`, strict
`@fastcall` and pins mean for a body that is pasted into another function,
and how does a user keep a call that the optimizer would expand?
**Decision:**
- **Spelling:** `@inline` and `@noinline` are attributes in the same
  positions as `@fastcall` (before or after `pub`), combinable with
  `@fastcall` and `@preserves`. `@inline @stackcall` is a `parse-expected`
  error ("an inlined body has no frame or stack arguments"), as are
  `@inline @noinline`, a repeated attribute, and either one on a global.
- **Guarantee:** every C80 call of an `@inline` function is expanded, with
  and without `--no-optimize`. An expansion that cannot fit (a pin
  conflict, below) is an error at the call. A private `@inline` function
  emits no out-of-line body; `pub @inline` also emits one, because BASIC
  markers and ASM units can only name `pub` functions. C80 has no function
  addresses, so a private function is reachable only through C80 calls.
- **Cross-unit:** `compile_project` builds one typed program for all units,
  so `pub @inline` in one unit is expanded in another unit's caller.
- **`@preserves` on an `@inline` function** describes the out-of-line
  body, which is always lowered (also when it is not emitted) and checked
  as in F-028. Inside a caller, the expansion is the caller's own code: the
  caller's allocator keeps every value the caller needs across it, which is
  at least what the contract promises, so a strict caller's reasoning is
  the same whether or not the callee is inlined. A caller's own
  `@preserves` covers the expansion like any other statement of its body.
- **Strictness** is a property of the function being lowered. A strict
  `@fastcall` callee is checked strictly on its out-of-line body; an
  expansion is held to its caller's rule. A diagnostic raised inside an
  expansion gets "(in 'g', inlined into 'f')" and a related span for each
  call site it came through ("'g' is inlined here (@inline)" or
  "(automatic)").
- **Pins inside an inlined body** apply within the expansion. One that
  overlaps a caller pin live at the call, or a register the caller
  preserves, is a `ty-duplicate-name` error at the call, with related spans
  at both pins. A caller's ordinary register homes are not pins: the
  allocator moves them.
- **`static` locals** stay one location for every expansion and the
  out-of-line copy: a static is a global.
- **Automatic inlining** (optimized builds only) never touches `pub`,
  `@stackcall`, `@noinline`, or a body whose inline `asm` has `CALL`/`RST`.
  A private function left with no C80 call emits no body. `@noinline` is
  for code whose CALL is the point (call-convention examples, a routine an
  ASM unit is expected to find by its label through a `pub` wrapper, timing
  experiments).
**Rejected alternatives:**
- Enforcing a callee's `@preserves` inside each expansion: it would forbid
  the caller from reusing a register the expansion frees, and the caller's
  allocator already guarantees the caller-visible effect.
- Keeping out-of-line bodies of automatically inlined functions: a private
  function called once would cost its body twice (Pong grew by about 500
  bytes that way). Test harnesses that call such a function by name use
  `compile_keeping_bodies` / `compile_project_keeping_bodies`, which differ
  only by the extra bodies.
- `@inline(always)` / `@inline(never)`: one attribute with an argument reads
  worse next to `@fastcall` and invites a third mode nobody asked for.
**Resolution authority:** implementer (H3i), per the H3i brief.

## F-032 — A value computed before `&&`/`||` in the same expression has no location

**Status:** resolved in H5 (found in H4)
**Evidence:** both build modes reject this with
`cg-unsupported: internal: value has no location` at the call:

```c
u8 a = 1;
u8 b = 2;
@noinline void use(u8 x, bool t) {}
void f() {
    use(a, b == 2 && a == 1);
}
```

The left side of a binary operator fails the same way
(`return a + u8(b == 2 && a == 1);`), and any other value evaluated before
a short-circuit operand of the same expression is expected to.
`bool t = b == 2 && a == 1; use(a, t);` compiles.
**Cause:** `check_short_circuit` in [semantics.rs](../../c80/src/semantics.rs)
splits the expression into rhs, skip and join blocks. The lowerer in
[lower.rs](../../c80/src/lower.rs) clears every value's location at a block
start; only the bool phi operands are carried across
(`lower_jump` / `lower_bool_phi`). A value defined before the split and
used after the join therefore has no location.
**Affected:** IR construction of `&&`/`||` and block-boundary value
locations. Not a miscompile: the compiler stops with a located diagnostic.
**Impact:** ordinary C expressions such as `mark(37, f(x) == 63 && g == 10)`
must be split by hand. The wording ("internal") tells the user nothing.
**Proposed resolution:** either evaluate values that are live across the
split into block-crossing temporaries (the same home/spill mechanism as
locals), or lower `&&`/`||` inside a value context without new blocks
(flag-based select). Add an H0 shape case for a value live across a
short-circuit, in both modes.
**Resolution authority:** implementer (next hardening step that touches IR
construction), with a conformance case first.
**Resolution:** `inline::carry_block_values` runs before inline expansion
in both build modes. Every vreg defined in one block and used in another
(except the operands of a short-circuit join, which the lowerer passes in
`A`) is stored to a fresh local right after its definition and loaded at the
start of each later block that uses it, the same mechanism inline expansion
uses for its splits. Pin scopes are shifted past the inserted ops. Fixtures
`prog.f032_*` (call argument, binary left side, word `||`, nested logic in a
three-argument call) and the random family `rlogic` (150 programs per type,
short-circuit nodes mostly as the right operand of an ALU op) pass in both
modes; the self-check example writes its conditions straight into `mark()`.

## F-033 — Static slot 0 loaded into the high half of a pair

**Status:** resolved in H5 (exposed by the `rlogic` random family)
**Evidence:** with a byte spilled to slot 0 and reloaded into `B`, `D` or `H`
while `A` is busy and the low half is free, `slot_load8` emitted
`LD rr,(F_S-1)` so the slot byte lands in the high half. The assembler
rejects `F_S-1` ("word is outside 0..65535"), so the function fails with
`cg-internal`. Example (both modes):
`i8 f(i8 a, i8 b) { i8 c = ((a + i8(((a - i8((a != 0) || (4 != 0))) != 0) && (a != 0))) - a); i8 d = i8((-(i8((1 != 0) && (a != 0))) != 0) && (i8(i8(u8(c) != u8(a)) == a) != 0)); return (-(i8(u16(c) < u16(c))) & c); }`.
**Affected:** `slot_load8` in [lower.rs](../../c80/src/lower.rs). A rejection,
not a miscompile.
**Resolution:** the pair trick is used for the high half only when the slot
offset is above 0; slot 0 goes through `A` (saved with `PUSH AF` if needed).
`rlogic` covers it.
**Resolution authority:** implementer (H5).

## F-034 — The parser loops forever on a statement that consumes nothing

**Status:** resolved in H5
**Evidence:** `void f() { { 1, 2 }; }` never finished compiling: the
expression statement stops at `,`, `expect(';')` reports an error without
consuming it, and `parse_block` asks for the next statement at the same
token forever, appending a diagnostic each time.
**Affected:** `parse_block` in [parser.rs](../../c80/src/parser.rs).
**Resolution:** a statement that consumed no token is followed by skipping
one token. `parse_tests::stray_tokens_in_a_block_terminate` covers four
malformed inputs.
**Resolution authority:** implementer (H5).

## F-035 — A function label that reads as a hex number

**Status:** resolved in H5 (workaround); made robust by construction in H6
**Evidence:** found by the H5 tail-call test. A function `h` got the label
`F16_H`; the helper assembler drops `_` inside numbers and reads a trailing
`H` as hexadecimal, so `CALL F16_H` (or the tail call `JP F16_H`) went to
0x0F16 and the run timed out. Any name whose label was hex digits plus a
final `H` (`each` -> `F1_EACH`) was affected, in both build modes.
**Assembler rule** (`number()` in [asm.rs](../../src/emulator/asm.rs), root
crate, unchanged): `_` is removed, then a term is a number when it is
decimal digits, `0X`/`$` plus hex digits, hex digits plus a final `H`,
`0B` plus binary digits, or binary digits plus a final `B`. A term that
parses as a number is a number even when a label of that name is defined.
Register and condition names are recognized only as a whole operand, and
`(IX`/`(IY` only as a prefix inside parentheses.
**Affected:** `asm_label` in [z80.rs](../../c80/src/z80.rs).
**Resolution:** H5 appended `_L` to a label that would read as a number.
H6 replaced that check with a scheme that cannot collide: every generated
label starts with a prefix holding a letter that is not a hex digit
(`FN<id>_` for functions and their `_B`, `_C`, `_S` labels, `G<id>_` for
globals and statics, `IA<n>_` for inline `asm` labels, `U<file>_` for ASM
units), so no number form and no register or condition name can match it,
and the id keeps it unique. `codegen_tests` compiles and runs functions and
globals named after every register, condition code, and hex-looking
identifier (`functions_named_like_registers_conditions_and_numbers_are_called_correctly`,
`globals_and_statics_named_like_registers_conditions_and_numbers_are_distinct`);
the first fails with the old `F<id>_` scheme.
**Resolution authority:** implementer (H5, H6). Whether the assembler should
prefer a defined label over a number is a root-crate question left open.

## F-036 — A parallel-move scratch register held a live carried value

**Status:** resolved in H5
**Evidence:** the random `rlogic` family, once H5 folding turned `(15 != 0)
|| x` into `true || x`; reproducible before H5 in both modes with a literal
`true`:
`i8 f(i8 a, i8 b) { i8 c = a; i8 d = (i8(i16(b) + i16(i8((c != 0) || true)))
& i8(i16(b) - i16((b + i8(true || (c != 0)))))); return a & i8(c != b); }`
was rejected by the H1 clobber guard ("live value in D would be overwritten
by `LD D,B`"). Restoring homes before a branch broke a register cycle
through a scratch byte chosen only from registers outside the move set; D
held a value carried across the second `||` at its home.
**Affected:** `parallel_moves_inner` in [lower.rs](../../c80/src/lower.rs).
**Resolution:** the scratch also skips registers with a live occupant that
is not part of the move set (otherwise the slot fallback is used).
`codegen_tests::a_parallel_move_scratch_never_holds_a_live_carried_value`.
**Resolution authority:** implementer (H5).
