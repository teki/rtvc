# C80 Compiler Hardening Plan

## Goal

Make `rtvc-c80` trustworthy for real TVC programs. Today it builds its
tutorial examples, but a review on 2026-11-12 found that it silently
miscompiles ordinary expressions, cannot lower shifts, rejects many normal
functions with "register pressure", emits naive code, and documents some of
this incorrectly.

This plan does not change the C80 language. The authoritative design is still
[C80 Compiler and Integrated Source View](c80-compiler.md); this plan brings the
implementation into line with that design's
"Code Generation Strategy (Phase 1)" section, which already requires spill
slots, fixed-point liveness, direct condition lowering, and an internal
diagnostic instead of wrong code. Corresponding backlog entry:
[TODO.md](../../TODO.md), "Mature the C80 compiler".

Phase 2 (integrated source view) should not start before H1–H3 pass.

## Status (2026-11-14)

All increments are done; details, measurements, and validation are in
[c80-hardening-progress.md](c80-hardening-progress.md).

| Increment | Status | Where the plan was wrong or changed |
| --- | --- | --- |
| H0 conformance suite | done | The crate test run later grew to about 14 s; H6 brought `--lib` back to about 9 s by skipping the auto-inline trial lowering when no call can be inlined. |
| H1 no silent miscompiles | done | — |
| HX examples as tests | done | — |
| H2a spills | done | — |
| H2b `@fastcall`, pins, `static` | done | — |
| H2c contracts | done | The manifest contract for ASM exports was not added: C80 cannot call ASM exports at all (F-029, open). |
| H3 shifts | done | — |
| H3a static addresses | done | — |
| H3i inlining | done | — |
| H4 TVC check | done | The test lives in `c80/tests/tvc.rs`, not beside `rtvc_tocas.rs`: the root crate cannot depend on the compiler. |
| H5 code quality | done | `DJNZ` was done, not deferred: a rotated `while` keeps its own guard at the entry, which covers the zero-trip case. Loop rotation makes zero- and one-pass loops 10 T and 3 T slower (accepted). Bubble sort and multiply need register allocation, which no H5 item reaches. Project builds keep the pre-optimization stack bound. |
| H6 documentation | done | `KNOWN_GAPS` is empty since H3, so the "unsupported" list is tested by `c80 error=` code blocks in the reference instead of being generated from it. "Generated code" became the reference's "Calling conventions" and "Where Values Live" sections. The Hungarian reference is a follow-up in `TODO.md`. |

## Purpose and Yardstick (user, 2026-11-13)

C80 exists as an alternative to a macro assembler: the bet was that a C-like
language could give comparable code size and speed with less effort. That
makes hand-written Z80, not other C compilers, the yardstick. Consequences
for this plan:

- Predictability matters as much as average quality. The programmer must be
  able to tell from the source what the compiler will emit, and to forbid the
  expensive fallbacks (strict `@fastcall`, H2).
- Every fallback that costs cycles (spill, save around a call, frame) must be
  visible in the listing and in a per-function report.
- H5's fixture table carries a third column beside bytes and T-states: a
  hand-written reference for the same routine, kept in the test as assembler
  source and run through the same harness. Report the ratio per fixture. The
  target is parity on tiny leaves and within about 1.3× on loops; record the
  real numbers rather than tuning fixtures to hit them.
- Inlining is part of this plan (H3i): a macro has no `CALL`/`RET`, while a
  C80 call costs 27 T-states before it does anything.
- **C-likeness is not a goal (user, 2026-11-13).** Where an explicit
  construct with extra restrictions makes the compiler simpler and the source
  more honest about cost, prefer it to a clever inference. Candidates, not
  yet scheduled unless marked:
  - *Pinned registers, explicit `static` locals, declared `@preserves`
    contracts:* scheduled in H2 (items 9–11).
  - *Explicit saves:* `save (bc) { ... }` emitting a visible `PUSH`/`POP`.
  - *Z80-shaped loops:* `repeat (n) { ... }` lowering to `DJNZ`, with the
    1..256 count semantics stated in the language rather than proven per
    loop.
- Follow-ups outside this plan that the purpose points to, in order of value:
  1. **Register-pinned locals** (a way to say "this counter lives in `B`"),
     so a hot loop reads like assembly where the programmer wants it to.
  2. **Cycle budgets and profiling** (static `@budget(N)` on bounded paths;
     emulator-side T-state profile folded through the source map; a TVC frame
     is about 62,500 T-states).

## Review Evidence

All repros are single files built with
`rtvc-c80 build X.c80 --origin 0x8000 --emit-asm X.asm`, optimized and
`--no-optimize` alike. The 125 existing tests pass throughout, so none of this
is covered.

### D1 — In-place ALU destroys a live local that shares `A` (wrong code)

```c
u8 f(u8 a, u8 b) { u8 s = a + b; u8 t = a & b; return s - t; }   // always 0
u8 f(u8 a) { u8 b = a + 1; u8 c = a + 2; return b ^ c; }         // always 0
```

Output for the first: `ADD A,C / AND C / LD C,A / SUB C / RET`.

Root cause: `IrOp::LoadLocal` binds the new vreg to the *same* location as the
local ([lower.rs](../../c80/src/lower.rs), `lower_op`). `lower_binary8` then
calls `park_live_src(lhs, dst)`, which only asks whether the **vreg** is live.
The vreg dies at the `ADD`, but the local `a` that also lives in `A` does not,
so nothing is moved and `A` is overwritten. F-015 fixed the vreg half of this
in E10. It only shows in single-block, frame-free functions, because "stable"
homes never use `A` or `HL`.

### D2 — Call spill slots collide (wrong code)

```c
u8 add(u8 a, u8 b) { return a + b; }
u8 sub(u8 a, u8 b) { return a - b; }
u8 f(u8 x, u8 y) { return add(sub(x, y), add(add(x, 1), sub(y, 2))); }
// f(50, 8) returns 108, not 99
```

Both temporaries are stored to `(IX-18)`.

Root cause: `lower_call` restarts `next_spill` at the base of the spill area
for every call and skips values already in a `Loc::Frame`. A value spilled at
the first call still owns the base slot when the second call spills a new value
into it.

### D3 — Loop-carried register local is not saved across a call (wrong code)

```c
u8 g(u8 x) { asm(inout: a = x, clobber: b, c, d, e, h, l, flags) { ld bc,0 } return x; }
u8 f(u8 n) { u8 s = 0; while (n != 0) { n = n - 1; s = g(s) + 1; } return s; }
// f(3) returns 1, not 3
```

`n` stays in `C` across `CALL g` and is neither spilled nor given a frame home.
The loop is only correct when the callee happens not to touch `C`.

Root cause: `liveness_fn` records one linear "last use index" per local.
`n`'s last textual use precedes the call, so `locals_needing_frame`,
`live_across_any_call`, and `lower_call` all treat it as dead there, although
it is live around the back edge.

### D4 — Shifts never compile

- `u8 f(u8 a) { return a << 1; }` → `cg-unsupported: unsupported byte operator
  in this increment` (`lower_binary8` has no `Shl`/`Shr` arm).
- `u16 f(u16 a) { return a << 3; }` → `ty-mismatch: operands have different
  types u16 and u8`. In `check_binary`
  ([semantics.rs](../../c80/src/semantics.rs), near the "operands have
  different types" message) the generic same-type check runs before the
  shift-specific `u8` count check, so every word shift is rejected.
- H0 correction: `i8` shifts are also rejected by the type checker; only
  `u8` shifts reach codegen. Compound `<<=`/`>>=` pass typing for all four
  types and fail in codegen.
- `fold_binary` already folds constant shifts; only typing and lowering are
  missing. `Z80Op` has no `SLA`/`SRL`/`SRA`/`RL`/`RR` variants yet.

### D5 — "Register pressure" rejects ordinary functions

```c
u16 f(u16 a, u16 b) { u16 c = a + b; return a + b + c; }
u16 t[5]; u16 f(u8 i) { return t[i] + t[i + 1]; }
u16 mul(u16 a, u16 b) { u16 r = 0; while (b != 0) { ... } return r; }
```

`@stackcall`, which the message recommends, does not help.

Root cause: `relocate` can only move a value to another register. Values are
written to the frame in exactly two cases: locals chosen up front by
`plan_homes`, and live values at a call. A temporary can never be spilled
mid-expression, and only `DE`/`BC` are word homes, so three live words fail.

### D6 — Frames are oversized and slow

`max_call_spill_bytes` charges two bytes for every vreg whose last use is
later than a call, including immediates and values already in the frame.
`fill_rows` in the pong example reserves 14 bytes for two byte locals. Any
function containing a call puts all locals that are live across it in
`(IX+d)` slots for the whole function, including loop counters.

### D7 — Naive instruction selection

Seen throughout `info/c80/out/*.asm`:

- `ADD A,1` / `SUB 1` instead of `INC`/`DEC`; `CP 0` instead of `OR A`;
  `LD A,0` instead of `XOR A`; `LD A,C / SUB 1 / LD C,A` instead of `DEC C`;
- `&&`/`||` and compare results are materialized as 0/1 in `A` and re-tested
  with `OR A`, although the design requires direct branch lowering;
- a global is reloaded immediately after it was stored from `A`;
- `JP` to a block that contains only a `JP`; `CALL x / RET` instead of `JP x`;
- `ADD A,0` emitted for the low half of a `+ 0x8000` word add.

### D8 — Test gaps

No operator × type × value matrix, no shift test, no test with two byte values
live across an ALU operation, no loop with a call and a register counter, no
test that builds the tutorial examples.

### D9 — Documentation gaps

- [info/c80.md](../../info/c80.md) lists shifts and `<<=`/`>>=` as accepted and
  never says they do not compile. Only
  [the tutorial](../../info/c80/tutorial.md) says so.
- Register pressure, the main practical limit, is mentioned only in a comment
  in `structs.c80`.
- `c80.md` links the CLI as `../src/bin/rtvc_c80.rs`; the file is
  `c80/src/bin/rtvc_c80.rs`.
- `c80.md` is organised by build increment (E01–E12), not by topic.

## Design Decisions

1. **Wrong code is never acceptable; a diagnostic is.** Every increment keeps
   the invariant that a program either compiles correctly or is rejected with a
   located diagnostic. H1 adds a structural guard so a future allocator bug
   becomes `cg-internal`, not a silent miscompile.
2. **Tests first.** H0 lands the differential suite with the current failures
   recorded, so each later increment is measured by the list shrinking.
3. **Keep the on-the-fly allocator; add what the design already asks for.**
   No SSA or graph colouring. Add real liveness, frame spills, and a
   post-lowering frame size. A rewrite is out of scope unless H2 proves the
   current structure cannot carry spills (see Risks).
4. **Correctness before quality.** H5 (peephole and selection) starts only
   after H1–H3 pass, and every rewrite must keep the differential suite green
   in both optimize modes.
5. **No changes to expressions, types, or statements.** `/`, `%`, local
   arrays, recursion, and by-value aggregates stay unsupported. The only
   surface additions control placement and lowering, not computation:
   strict `@fastcall`, pinned registers, `static` locals, and `@preserves`
   (H2), and `@inline` (H3i).
6. **Memory observability is untouched.** Loads and stores of source-visible
   memory, port I/O, and inline assembly stay ordered and uncoalesced, as the
   design and E12 require. Only registers, flags, and compiler-private frame
   slots are fair game.

## Increments

Run in order. Each ends with a buildable tree, the gate below, updated
documentation for behaviour that changed, and a checkpoint in
`c80-hardening-progress.md` (create it when H0 starts, per the
[plans README](README.md)). Record new discoveries as F-018 onward in
[c80-compiler-findings.md](c80-compiler-findings.md); D1–D5 become F-018 to
F-022 in H0.

### H0 — Differential conformance suite

**Work**

- New test module `c80/src/conformance_tests.rs`, registered in
  [lib.rs](../../c80/src/lib.rs) beside the existing three.
- Helper: compile a snippet with `compile(CompileInput { .. })` for
  `optimize` true and false, run `f` through
  `harness::execute_function_with` with raised limits (2,000,000 instructions,
  50,000,000 T-states), assert `!timed_out`, IX `0x1111`, IY `0x2222`, and the
  expected `A` or `HL`.
- **Operator matrix.** For each of `u8`, `i8`, `u16`, `i16`: `+ - & | ^ << >>`,
  the six comparisons, unary `- ~ !`, and every cast pair, each as
  `T f(T a, T b) { return a OP b; }` against a Rust closure. Values:
  bytes `0 1 2 3 7 8 0x55 0x7F 0x80 0x81 0xFE 0xFF`; words
  `0 1 2 0xFF 0x100 0x101 0x1234 0x7FFF 0x8000 0x8001 0xFF00 0x00FF 0xFFFE
  0xFFFF`. Shift counts additionally `9 15 16 17 255`.
- **Shape matrix.** The same operators with operands arranged so each one is
  live afterwards: `(a OP b) OP2 (a OP3 b)`, a local reused after an in-place
  operation, a parameter reused twice, both for bytes and words.
- **Program fixtures** (appendix A): array sum, `u16` array index and store,
  struct array field access, nine byte locals, seven word locals, nested
  calls, call in a loop with a register counter and a callee that clobbers
  every register, `@stackcall` with five arguments, pointer walk, signed
  counting loop, bubble sort, shift-add multiply, `str` walk, linked struct
  walk.
- **Seeded random expressions.** A small LCG (no new dependency) generates
  expression trees of depth ≤ 4 over two parameters and two locals per type,
  evaluated by a reference interpreter in the test. Fixed seeds, about 300
  programs per type, so failures are reproducible by seed and index.
- **Expected-failure ledger.** A `KNOWN_GAPS` table of `(case id, kind)` where
  kind is `Rejected` or `WrongValue`. The suite fails when an unlisted case
  fails *and* when a listed case starts passing, so the table can only shrink
  deliberately. From H1 on, a separate test asserts the table holds no
  `WrongValue` entry.
- **Examples build test** in [c80/tests/rtvc_c80.rs](../../c80/tests/rtvc_c80.rs):
  compile every `info/c80/*.c80` and every `info/c80/*/rtvc-c80.toml` in
  process and assert no errors, replacing reliance on `compile.sh`.

**Gate:** suite runs in both optimize modes; ledger matches reality exactly;
existing 125 tests unchanged. Keep the whole crate test run under about ten
seconds in debug.

### H1 — No silent miscompiles

**Work**

1. **Structural clobber guard.** Give `Z80Op` a `writes()` register set
   (including `A` for ALU ops, `HL` for `ADD HL`/`SBC HL`, both halves for
   pairs, everything for `Call`). In `Lowerer::emit`, when not `synthetic`,
   check each written register: any occupant returned by `occupants(r, ..)`
   other than the declared result of the current operation must be dead at
   `op_index`. A violation returns
   `DiagCode::CgInternal` ("internal: live value in A would be overwritten")
   with the current span. Callers that legitimately redefine a register pass
   the key being defined. This turns D1-class bugs into diagnostics before any
   targeted fix lands, and stays on permanently.
2. **Fix D1.** Replace `park_live_src`'s vreg-only test with an occupant test:
   before an in-place operation on `A` or `HL`, relocate every *other* live
   occupant of the destination (locals included) with `relocate`, and rebind
   locals in `local_loc`. Apply the same rule in `lower_unary`, `lower_cast`,
   `add_hl_imm`, `lower_load_indirect`, and compare lowering, which use the
   same in-place pattern.
3. **Fix D2.** Make spill slots owned for the value's lifetime: keep a
   `spill_free: Vec<(disp, width)>` free list on the `Lowerer`, take a slot
   when a value is spilled, and return it in `drop_dead_after` when the owner
   dies. Remove the per-call `next_spill` restart. Size the area from the
   maximum simultaneous occupancy (see H2 for the post-lowering frame size; in
   H1 a conservative over-estimate is acceptable).
4. **Fix D3 with real liveness.** Replace `local_last` with block-level
   live-in/live-out sets for locals computed to a fixed point over the CFG,
   plus a per-op "live after" query. Vregs are block-local (`vreg_loc` is
   cleared at each block), so the linear index remains valid for them; assert
   that in debug builds. Update `locals_needing_frame`,
   `live_across_any_call`, `max_call_spill_bytes`, `drop_dead_after`, and
   `lower_call` to use the new query.

5. **Fix F-024 (found in H0): stale scratch read in signed byte compare.**
   `lower_compare_value` picks a scratch register, then an `ensure_*` call
   relocates an operand and frees or reuses that scratch, so
   `LD A,tmp / CP B` compares a value with itself. The clobber guard cannot
   see this (it is a stale read, not an overwrite). Fix it, then audit every
   lowering routine for the same shape: a register chosen or a location read
   *before* a call that can relocate values, and used after it. Re-read
   locations after any `ensure_*`/`claim_*`/`relocate`.
6. **Fix F-025 (found in H0): byte view of a word register.** A narrowing
   cast leaves a live byte value bound to `L` while the word still owns `HL`;
   an in-place word operation then destroys it. The occupant test in item 2
   must treat an occupant of `H` or `L` as an occupant of `HL`, and the
   signed-compare `XOR 80H` bias must go through the same test.

**Tests:** the D1–D3 repros as named regression tests with exact expected
values; the callee-clobbers-everything loop for byte and word counters; the
guard itself, by a unit test that drives `emit` with a live occupant.

**Gate:** zero `WrongValue` entries in the ledger; every remaining gap is
`Rejected`. Baseline and optimized agree on all cases. Tutorial examples still
build byte-identically unless a fix required a change, in which case record
the size difference in the progress file.

### H2 — Registers first, cheap spills as a fallback, strict `@fastcall`

**Direction (user, 2026-11-13).** Registers are the goal; memory and stack
traffic are what make compiled Z80 code slow. Spilling exists so ordinary code
compiles, not as the normal path, and the programmer must be able to forbid it
per function. Three tiers:

| Function | Body may use | When it does not fit |
| --- | --- | --- |
| `@fastcall` (explicit) | registers only: no frame, no static slot, no `PUSH`/`POP`; `CALL`/`RET` return addresses are the only stack use | `cg-unsupported` naming the value and the operation that needed a spill |
| unannotated (default) | registers first; then `PUSH`/`POP` around calls; then static slots | always compiles, spills are marked in the listing |
| `@stackcall` | IX frame for parameters, otherwise as default | as default |

Strict `@fastcall` functions are the "macro-like" ones: small, hand-tuned
routines called very often, where even one `PUSH`/`POP` pair is too much. The
contract covers both sides: the body never spills, and (through the clobber
sets in item 3) a caller does not save registers the routine leaves alone.

`@fastcall` and the default share the same register calling ABI (F002), so
adding or removing `@fastcall` is not an ABI change; it only constrains how
the body is lowered. Today's "register pressure" rejection becomes the
`@fastcall` behaviour instead of everyone's. The tutorial `tvc-usr` and
`pong` entry points are already `@fastcall` and trivially fit. (H2b: true
for `tvc-usr` and pong, but `mixed/buf.c80` `sum` needs a static slot and
lost `@fastcall`.)

**Work**

0. **Parse `@fastcall` as its own flag.** The parser currently maps it to
   `CallConv::Register` and forgets it. Keep the convention, add a
   `strict_registers` flag on `Function`/`TypedFunction`, and report it in the
   exported signature. Every spill path below checks the flag first.
1. **Static slots, not IX frames, for spills and overflow locals.** Recursion
   is rejected, so each function's spill slots and frame-resident locals get
   fixed absolute addresses, emitted as zero-initialised data directly after
   the function (source-order placement is preserved; the bytes belong to that
   function in the listing and size report). `LD A,(nn)` is 13 T-states
   against 19 for `(IX+d)`, word load/store via `HL` is 16, and there is no
   prologue or epilogue. IX frames remain only for `@stackcall` parameters
   and for locals whose address is taken with `&` (their address must be
   stable, and a static slot satisfies that too, so prefer static there as
   well unless a test shows otherwise). Constraints to record in the
   reference: a function with static slots is not re-entrant (no interrupt
   handler support exists yet; revisit when it does), and code must be in
   RAM, which is already true for every TVC output.
2. **`PUSH`/`POP` around calls** for register values that are live across a
   call and are only needed after it: `PUSH rr` + `POP rr` is 21 T-states
   against 32 for a static word store and reload. Byte values pair up in one
   push where two are live. Pushes must balance on every path and count in
   the stack bound. Use a static slot instead when the value is also read
   between pushes in a way that would need unbalanced access.
3. **Callee clobber sets.** The call graph is acyclic and the whole project
   is compiled together, so compute, bottom-up, the set of registers each C80
   function actually writes (including through its own calls). A caller
   keeps a live value in a register the callee does not touch, with no save at
   all. Inline `asm` contributes its declared clobbers; ASM units and unknown
   targets clobber everything. This is not a change to the argument ABI. It
   is what lets a strict `@fastcall` function call small leaves in a loop.
   Stability rule: a clobber set may only be relied on within one build; do
   not expose it as part of a function's public type.
4. **Spill on eviction.** When `relocate` finds no register, and the function
   is not strict, move the value to a static slot from the H1 free list
   instead of returning `pressure`.
   Selection order follows the design: a dead value, then a rematerializable
   one (`Imm8`/`Imm16`, a symbol address), then the farthest next use, with a
   fixed register tie-break for determinism.
5. **Reload on use.** `ensure_in_r8`/`ensure_in_rr`/`alu_src_of`/
   `word_src_rr` already accept `Loc::Frame` sources; make them the single
   reload path and allow `ADD A,(IX+d)`-style operands where the instruction
   has one, to avoid a scratch register.
6. **Word homes.** Allow a word local to live in the frame when `DE` and `BC`
   are taken, rather than failing in `assign_stable_homes`. Remove
   `fits_stable_registers` as a hard gate; it becomes a hint for the optimistic
   pass.
7. **Right-size the spill area (D6).** Count only register-resident values
   that are actually spilled. Add a fixture asserting `fill_rows`-style
   functions reserve exactly their locals.
8. **Diagnostics and visibility.** `pressure` is reported only for a strict
   `@fastcall` function, and names the value and operation that needed the
   spill so the programmer can restructure or use inline assembly. Drop the
   `@stackcall` suggestion, which does not help. In `--emit-asm` output and
   the listing map, mark every spill store, reload, and save/restore as
   synthetic with a `; spill` comment, and report per function: static slot
   bytes, pushes per call site, and spill T-states on the straight-line path.

**Explicit constructs (user, 2026-11-13: "Yes, into H2").** C80 need not be
C-like; the source states where values live and the compiler checks it.
Spellings below are proposals. Fix them in a finding at the start of H2,
checking for clashes with attribute `@name` and the `@{...}` marker syntax,
before writing parser code.

9. **Pinned registers: `u8 n @ b;`** on locals and parameters. Register names
   are those of inline-`asm` headers (`a b c d e h l bc de hl`); width must
   match the type.
   - The value lives in that register for the variable's whole scope. The
     allocator never moves, spills, or aliases it, and never hands the
     register (or an overlapping half/pair) to anything else in that scope.
     (H2b refinement, F-027: while the old value is dead because a store
     into the pin later in its scope redefines it, the pin's new value may be
     computed in its register. Otherwise `s = s + a` with `s @ hl` is
     rejected instead of being one `ADD HL,DE`.)
   - Two pins that overlap in register and scope are an error. `&pinned` is
     an error.
   - An operation that needs the pinned register as its fixed operand or
     scratch (for example `A` for 8-bit ALU results, `HL` for word adds,
     `B`/`C` for port forms, `BC`/`DE`/`HL` for `cpu::ldir`) while the pin is
     live and is not itself the operand is an error naming the pin and the
     operation. It is never resolved by a hidden move.
   - A pinned parameter whose ABI register (F002) differs gets one entry
     move; the ABI itself does not change.
   - Across a call that may write the register: in a strict `@fastcall`
     function this is an error unless the callee's contract preserves it; in
     an unannotated function the compiler saves it with a marked
     `PUSH`/`POP`.
   - Pins are honoured in every function kind. A strict function with all
     hot values pinned needs no allocation decisions at all.
10. **Explicit memory locals: `static T x;` and `static T a[N];`** inside a
    function body.
    - Storage is a fixed address emitted directly after the function, in the
      same region as the compiler's static slots, zero-initialised in the
      image or set by a compile-time initializer.
    - It is real static storage: the value persists between calls, counts as
      definitely assigned, and may have its address taken. Say so plainly in
      the reference; this is not a hidden spill.
    - `static` local arrays lift the current "no local arrays" restriction
      for this storage class only. Automatic local arrays stay rejected.
    - In a strict `@fastcall` function, `static` is the only way a local gets
      memory. Its accesses cost what a global's do and appear as ordinary
      loads and stores, ordered and uncoalesced like any source memory.
    - `static` becomes a keyword; add it to the lexer list and the reference.
11. **Declared register contracts: `@preserves(bc, de)`** on a function
    (C80), and the same field for ASM unit exports in the manifest.
    - For a C80 function the compiler checks the body: a preserved register
      may not be written, directly or through a callee that does not itself
      preserve it, unless the write is inside a balanced save. Violation is
      an error at the offending statement.
    - Callers keep live values in preserved registers across the call with no
      save.
    - A contract is part of the function's exported signature and appears in
      the per-function report. Changing it can break callers at compile time,
      never silently.
    - For ASM exports the contract is trusted, like `stack:N`, and reported
      as declared, not proven.
      (H2c: not done. C80 cannot call an ASM export at all, see F-029, so
      the manifest field would have nothing to attach to.)
    - **Inference is limited** (this narrows item 3): an inferred clobber set
      is used only for private functions called from the same unit, where an
      edit and its effect are side by side. `pub` functions and ASM exports
      have exactly their declared contract, defaulting to "clobbers
      everything".
12. **Strict-function diagnostics** use these constructs in their advice:
    a value that does not fit says which register conflicted and suggests
    pinning it elsewhere, declaring it `static`, or adding `@preserves` to
    the callee.

**Order within H2.** H2 is three reviewable steps, each with the full suite
green before the next:

- **H2a — fallbacks:** items 1, 2, 4–8 (static slots, `PUSH`/`POP` at calls,
  spill on eviction, reload, word homes, right-sizing, visibility). Ends with
  no pressure rejections for unannotated functions.
- **H2b — explicit placement:** items 0, 9, 10, 12 (strict `@fastcall`,
  pins, `static` locals, diagnostics).
- **H2c — contracts:** items 3 and 11 (`@preserves`, limited inference).

In the list above, read "frame slot" as "static slot" in items written before
this direction was set (reload on use, word homes, right-sizing).

**Tests:** all `Rejected` ledger entries caused by pressure now pass for
unannotated functions; the same sources with `@fastcall` added are still
rejected, with the new message, and are kept in the suite as permanent
expected-rejection cases rather than removed from it; pins: a counter pinned
to `B` stays in `B` through a loop (byte-exact), overlapping pins, `&pinned`,
width mismatch, and an ALU conflict with a pin on `A` are each located
errors, a pinned value across a non-preserving call is saved in a default
function and rejected in a strict one; `static` locals persist across calls,
can be initialised, indexed, and addressed, and a `static` array works where
an automatic one is rejected; `@preserves` violations are caught directly
and through a callee, a caller keeps a value in a preserved register with no
save, an inferred set is used for a private same-unit callee and not for a
`pub` one; a strict function calling a leaf that leaves
`BC` alone keeps its counter in `B` with no save (byte-exact fixture); stack
bound tests (`StackProvenance::Proven`) still match measured `sp_used`,
including pushes; `byte_identity_is_ret` and `word_add_is_add_hl_de` are
unchanged; `indexed_frame_boundary_is_rejected` still fires for `@stackcall`.

**Gate:** no ledger entry for an unannotated function mentions register
pressure. A strict pong variant (hot helpers `@fastcall` with pins and
`@preserves`, no spill anywhere in the frame loop) compiles and matches the
default build's behaviour in H4's smoke check once that exists. No
unannotated fixture uses IX unless it is `@stackcall`. Stack
bounds proven by the compiler are ≥ the harness high-water mark on every
fixture. Record bytes and T-states of `fill_rows` and `put_row` before and
after.

### H3 — Shifts

Semantics are fixed by F001 in the design: the count is `u8`; left shift wraps
at the operand width; right shift is logical for unsigned and arithmetic for
signed; a count ≥ width gives 0 (or all sign bits) by defined repeated-shift
behaviour.

**Work**

1. **Typing.** In `check_binary`, exempt `Shl`/`Shr` from the same-type check
   so the existing "shift count must be u8" rule is the only constraint.
   Result type is the left operand's type. Check compound `<<=`/`>>=` the same
   way (the compound path already types the count as `u8`).
2. **Instructions.** Add `Z80Op` variants for `SLA r`, `SRL r`, `SRA r`,
   `RL r`, `RR r`, `ADD HL,HL` (already expressible as `AddHl(Hl)`), and
   `DJNZ`. Confirm timing metadata comes from the disassembler as for other
   ops and that [the helper assembler](../../info/assembler.md) accepts each
   form.
3. **Constant counts.** `u8 << k`: `ADD A,A` × k (k ≤ 7), 0 when k ≥ 8.
   `u8 >> k`: `SRL A` × k; `i8 >> k`: `SRA A` × min(k, 7). `u16 << k`:
   `ADD HL,HL` × k, with a byte move for k ≥ 8 (`LD H,L / LD L,0` then the
   remainder). `u16 >> k`: `SRL H / RR L`; `i16`: `SRA H / RR L`; k ≥ 8 by byte
   move plus zero or sign fill. Choose by T-states, record the table in the
   progress file.
4. **Variable counts.** Count in `B`: `INC B / DEC B / JR Z,done`, then
   `loop: <one step> / DJNZ loop`. The zero test makes `DJNZ` safe here
   (F-017's objection was an unguarded `B == 0`). Up to 255 iterations is the
   defined behaviour; do not mask the count. Report timing as `Branch`, and
   make `SpanCost` incomplete for the span.
5. **Address scaling** may use the new shift ops where they are cheaper than
   the current shift-and-add, without changing observable accesses.
6. **Pong.** Replace the six `a += a` lines in
   [pong.c80](../../info/c80/pong/pong.c80) with `<< 6` and fix its comments.
   Re-verify the cassette on the emulated TVC.

**Tests:** the H0 shift rows (all four types, counts `0..=17` and `255`,
constant and variable, plus compound forms); a byte-exact fixture for
`u16 << 1` (`ADD HL,HL / RET`) and `u8 >> 1`.

**Gate:** no ledger entry mentions shifts; `KNOWN_GAPS` contains only
constructs the language reference lists as unsupported.

**H3 notes (implementer).**
- Item 3's sequences are not the fastest for every count: `u8 << 5..7`
  and `u8 >> 2..7` rotate and mask, `i8 >> 7` is `ADD A,A / SBC A,A`,
  word counts 5–7 shift the other way by `8-k` and move bytes, and word
  counts ≥ 10 shift the moved byte in `A`. The chosen table is in the
  progress file.
- Item 4: `B` may be pinned or busy; the counter is any free register
  (`DEC c / JR NZ` when it is not `B`).
- The gate's "only constructs the reference lists as unsupported" does not
  hold for the remaining `StaticAddr` entry: the reference neither allows
  nor forbids it (F-023). It is H3a.

### H3a — Addresses in static initializers (small; after H3)

Today `N b = {2, &c};` fails with "static initializer must be a compile-time
value" (F-023). The address of a global is a link-time constant, and without
it pointer tables (sprite lists, linked static data, jump-free dispatch data)
have to be filled in at run time. Approved by the user on 2026-11-13.

**Accepted forms** in a global scalar, array-element, or struct-field
initializer of pointer type (and of `u16` only through an explicit
`u16(&x)` cast, consistent with the existing explicit-conversion rule):

- `&global` for a scalar or struct global;
- `&array[K]` and `&global.field`, `&array[K].field`, with `K` a compile-time
  constant inside the array bounds;
- the same through a public name of another unit, `&other::global`;
- `ptr<U>(&x)` reinterpretation, as already allowed at run time.

Still rejected: `&array` without an index (arrays do not decay), the address
of a `const` (it has no storage), of a `str` payload, of a local, and any
arithmetic on the address beyond the constant index/field offset above.

**Work**

1. In static-initializer checking in
   [semantics.rs](../../c80/src/semantics.rs), add a "link-time address"
   value alongside compile-time integers: `(GlobalId, byte offset)`. Type
   checking is the same as for the run-time `&` expression.
2. In `lower_global` ([lower.rs](../../c80/src/lower.rs)), emit such a field
   as `DW label+offset` instead of `DB` bytes. Confirm the helper assembler
   accepts `label+n` in `DW` and resolves forward references and labels in
   other units' segments; if either is missing, record a finding before
   working around it.
3. Decide and document forward references: a global may take the address of
   one declared later in the same unit or in another unit. Source-order
   placement is unchanged; only the emitted word depends on the final layout.
4. Listing and data metadata: the two bytes are data with a symbol
   reference, not opaque bytes, so the map can show the target name.
5. Reference and tutorial: document the accepted forms and add a pointer
   table to `memory.c80` or `structs.c80`.

**Tests:** the H0 linked-struct fixture (`N c = {3, ptr<N>(0)}; N b = {2, &c};
N a = {1, &b};` walk returns 6) leaves `KNOWN_GAPS`; `&array[3]` and
`&ps[2].w` produce the right addresses (checked against the symbol table);
forward and cross-unit references; each rejected form has a located
diagnostic; out-of-bounds constant index is an error.

**Gate:** F-023 resolved; `KNOWN_GAPS` has no "static initializer" entry.

**Notes from implementation (2026-11-14):**
- "of a local": a function-local `static` initializer may take the address
  of another `static` in scope (itself included); automatic and pinned
  locals stay rejected.
- C80 has no arrays of pointers (`ArrayElem` is scalars and structs), so a
  "pointer table" is an array of structs with a pointer field or a `u16`
  array of `u16(&...)` values.
- Item 2 holds: the helper assembler already resolved `DW label+n` forward
  and across `ORG` blocks; no root-crate change. The per-unit optimizer
  pass, however, assembles one unit alone and skips JR shortening when the
  unit names another unit's label (pre-existing, F-030).
- A `const` initialized with an address is rejected: a const is folded
  into code, and the address exists only at link time.

### H3i — Inlining (runs after H3, before H4)

A macro expands in place; a C80 call costs 27 T-states for `CALL`/`RET`
plus argument moves and any caller saves. Inlining is what lets a small
routine cost what a macro would.

**Language surface**

- New attribute `@inline`, accepted where `@stackcall`/`@fastcall` are (before
  or after `pub`). The grammar becomes
  `attr := "@stackcall" | "@fastcall" | "@inline"` and a function may carry
  `@inline` together with `@fastcall`. `@inline @stackcall` is an error. The
  parser currently accepts a single attribute per position; extend it to a
  set and keep rejecting duplicates and unknown names.
- `@inline` is a guarantee, not a hint: every call from C80 is expanded, in
  optimized and `--no-optimize` builds alike. If a call cannot be expanded
  the compile fails with a located diagnostic at the call site.
- A private `@inline` function emits no out-of-line body. A `pub @inline`
  function also emits one, so BASIC `@{unit::name}` markers and ASM units can
  still call it.
- `@inline` does not change the function's type or ABI. Arguments are still
  evaluated exactly once, left to right, before the body.

**Work**

1. **IR pass before lowering** (new `c80/src/inline.rs`, run on
   `TypedProgram` after all units are analysed, callees first in call-graph
   order; the graph is acyclic). For each call to an inlinable callee:
   - give each parameter a fresh caller local and store the argument vregs
     into them in order;
   - clone the callee's blocks with fresh `BlockId`s, `VReg`s, and `LocalId`s;
   - split the caller block at the call; replace each callee `Return` with a
     store to a fresh result local (when non-void) and a jump to the
     continuation block; load the result local there into the call's `dst`.
     Vregs do not survive block boundaries, so the result must be a local.
   - keep every cloned op's original `SourceSpan` and add the call-site span
     as a related origin.
   Confirm first that `compile_project` hands all units to one
   `TypedProgram` so cross-unit inlining needs no extra plumbing; if units
   are lowered separately, record a finding and restrict H3i to same-unit
   calls.
2. **Inline `asm` blocks** inside an inlined body: local labels are
   namespaced per block instance; give each clone a new namespace.
3. **Automatic inlining** (optimized builds only), conservative and
   deterministic: a private function with exactly one call site; or a body
   whose lowered size is no larger than the call it replaces (measure with a
   trial lowering, do not estimate from IR). Never auto-inline `pub`,
   `@stackcall`, or functions containing inline `asm` with `CALL`/`RST`.
   Report each decision in the per-function report.
4. **Strict callers.** Inlining into an `@fastcall` function can exhaust its
   registers. The H2 diagnostic then names both the inlined callee and the
   call site, so the programmer knows which expansion did it.
5. **Constants.** When an argument is a compile-time constant, the parameter
   local should behave as one in the clone (immediate operands, folded
   branches that become unreachable are dropped). Do the minimum that makes
   `put_row`-style helpers collapse; a general constant-propagation pass is
   not in scope.
6. **Reports.** Stack bounds drop the `CALL` at inlined sites. The listing
   attributes inlined bytes to the caller's address range and the callee's
   source lines. Per-function size reports show inlined growth separately.
7. **Reference and tutorial:** document `@inline`, the guarantee, and the
   `pub` rule; add a tutorial example that shows the same helper as a call
   and inlined, with both listings.

**Tests:** every H0 program fixture again with each helper marked `@inline`
(results unchanged, no `CALL` to the helper in the caller's bytes); argument
evaluated once (a `cpu::in` argument produces one port read); callee with
several `return`s and with a loop; nested inlining three levels deep; inline
`asm` with a local label inlined twice into one caller; `pub @inline` still
callable through a project export; `@inline @stackcall` rejected; a
byte-exact fixture where `u8 inc(u8 x) { return x + 1; }` inlined into a
loop is a single instruction; listing map covers every byte and resolves
inlined instructions to the callee's lines.

**Gate:** H0 suite passes in both modes with and without `@inline` on the
helpers. Pong with `put_row` and `read_row` marked `@inline`: record bytes
and T-states per frame-loop iteration before and after, and re-verify on the
emulated TVC.

### H4 — End-to-end TVC check

**Work**

- Add an integration test next to the existing cassette `USR` test in
  [rtvc_tocas.rs](../../src/bin/rtvc_tocas.rs) that builds a mixed project in
  process, loads it with `Tvc::load_cas`, runs it, and reads results from
  memory. The C80 side runs a self-check routine (operators, shifts, a
  frame-spilling function, nested calls) and stores a pass/fail bitmap that
  BASIC `PEEK`s. This proves generated code under the real ROM, interrupts,
  and BASIC-owned stack, which the bare harness does not.
- Add a pong smoke check: after `USR`, run a fixed number of frames with no
  input and assert the paddles and ball are present in VID0 at the expected
  addresses.

**Gate:** both tests pass with and without optimization.

### H5 — Code quality

**Outcome:** done; every fixture row improved or held (table in the
progress file). The H5 brief renumbered the items (its 1–7 are items 1, 2,
3, 7, 4, 8, and 6 below). Item 5 needed no separate work: since H2a no
local has an `(IX+d)` home, and a loop counter stays in a register with a
save only around calls.

Establish numbers first: a table test records bytes and worst-path T-states
for a fixed fixture set (`count`, `walk`, `fill_rows`, `put_row`, `hits`,
`step`, bubble sort, shift-add multiply). Every item below must not regress
any row and must keep H0 and H4 green in both modes. Record before/after in
the progress file.

1. **Direct condition lowering.** Lower `&&`, `||`, `!`, and comparisons used
   as branch conditions straight to control flow in
   [semantics.rs](../../c80/src/semantics.rs) IR construction, so no 0/1 is
   built. Materialize only when the boolean is used as a value. This is a
   design requirement, not an optional optimization.
2. **Flag-aware peepholes** in [optimize.rs](../../c80/src/optimize.rs), each
   gated on flag liveness computed backwards over the item list:
   - `ADD A,1`→`INC A`, `SUB 1`→`DEC A` (carry differs: only when carry is
     dead);
   - `CP 0`→`OR A` (same Z and S; carry cleared in both);
   - `LD A,0`→`XOR A` only when all flags are dead;
   - `LD A,r / ADD A,1 / LD r,A`→`INC r` when `A` and carry are dead after;
   - `ADD A,0` and `ADC A,0`-after-known-no-carry removal for word immediates
     with a zero low byte (`INC H`-style high-half arithmetic).
   Needs `Z80Op` variants for `INC r`/`DEC r`/`INC rr`/`DEC rr`.
3. **Redundant moves.** Track register contents within a block: drop
   `LD H,D / LD L,E` when `HL` already equals `DE`, and a register reload of a
   value still held. Never drop a load from source-visible memory.
4. **Control flow.** Thread `JP` to an unconditional `JP`; turn
   `CALL x / RET` into `JP x` when the function has no frame and no
   `@stackcall` cleanup (update stack-bound accounting: the tail call adds no
   return address); remove blocks made empty.
5. **Loop registers.** With H1 liveness, keep a loop counter in a register
   across the loop and save it only around calls, instead of a permanent
   `(IX+d)` home, when that is cheaper by the fixture table.
7. **Pointer loads through `BC`/`DE`** (found in H2b). Every byte load
   through a pointer is `LD A,(HL)`, so a pointer pinned to `bc`/`de` is
   copied to `HL` on each dereference and a word accumulator cannot sit in
   `HL` in a pointer loop. Use `LD A,(BC)`/`LD A,(DE)` and the matching
   stores where the pointer already lives there.
8. **Constant folding and trivial identities** seen in HX1: `255 + 1`,
   `LD HL,1` followed by `INC HL`s, word `a += a` as `ADD HL,HL`, a slot
   store immediately reloaded.
6. **`DJNZ` for counted loops** stays deferred unless the zero-trip case is
   proven per loop; record the decision either way.

Provenance rules from E09/E12 hold: merged instructions keep all contributing
origins; removed ones get a no-code mapping; listing maps cover every byte.

**Gate:** fixture table improves or holds on every row; total bytes of
`pong` drop measurably (record the figure); no change in any memory or port
trace in `baseline_and_optimized_match_effects_and_improve_cost` and
`port_io_optimized_matches_baseline_trace`.

### HX — Examples as tests and teaching material (cross-cutting)

The examples under [info/c80/](../../info/c80/) have two jobs (user,
2026-11-13): they **test** features, by pinning the code the compiler emits
and the results it computes, and they **teach**, as commented source a
newcomer reads to learn the language. Every increment that adds or changes a
feature updates them in the same change; HX1 builds the machinery once.

**HX1 — machinery (do with H1, before features start landing)**

1. **Self-checking examples.** A comment directive states expected results
   next to the code it describes, for example
   `//! expect: add(2, 3) == 5` and `//! expect: wrap(255) == 0`. The
   examples test parses these, calls the function through the harness in both
   optimize modes, and compares. Optional forms as needed:
   `//! expect-error: cg-unsupported` on a deliberately rejected snippet kept
   in a separate `*.fail.c80` file, and `//! expect-mem: buf[0..4] == 1,2,3,4`
   for effects. Keep the directive grammar tiny and document it in the
   tutorial, since readers will see it.
2. **Golden listings.** Check in the optimized `--emit-asm` output of every
   example under `info/c80/expected/` (the existing `info/c80/out/` stays
   ignored build output). A test compiles each example and compares with the
   golden file; `C80_UPDATE_EXPECTED=1` rewrites them. A codegen change then
   shows up as a reviewable diff of real listings, and the tutorial can link
   to a listing instead of pasting one that goes stale.
3. **Budget lines.** Where an example makes a cost claim, pin it:
   `//! expect-bytes: put_row <= 24` or `//! expect-tstates: inc == 4`,
   checked from the listing map. Use sparingly, on the claims the comments
   actually make.
4. **Projects** (`project/`, `mixed/`, `tvc-usr/`, `pong/`): golden listing
   of the C80/ASM segments, plus the H4 emulator checks for behaviour.
5. `compile.sh`/`compile.bat` keep working for users; the tests do not
   depend on them.

**Comment standard for every example**

- A header saying what the file teaches and what to look at in the listing.
- Above each function: what it demonstrates, the registers its arguments
  arrive in, and what it compiles to when that is the point ("this is a
  single `RET`").
- Where a feature has a cost, say the cost in bytes or T-states and pin it
  with a budget line.
- Where a construct is rejected on purpose, show the rejected form in a
  comment with the diagnostic it produces and the accepted alternative.
- No comment may describe a limit the compiler no longer has; the H6 gate
  greps for the known stale phrases ("no `<<` yet", "run out of registers").

**Examples to add or revise, with the increment that owns them**

| File | Teaches | Owner |
| --- | --- | --- |
| all existing files | add `expect` lines and the comment standard | HX1 |
| `spills.c80` (new) | what happens when values do not fit: a marked `PUSH`/`POP`, a static slot, and how to read them in the listing | H2a |
| `registers.c80` (new) | strict `@fastcall`, `u8 n @ b` pins, `static` locals and arrays, a pin conflict and its fix | H2b |
| `contracts.c80` (new) | `@preserves`, a caller keeping a counter live across a call, a violation | H2c |
| `shifts.c80` (new) | constant and variable shifts on all four types, signed vs unsigned right shift, count ≥ width, `<<=` | H3 |
| `structs.c80`, `memory.c80` | drop the "walk with a pointer because indexing runs out of registers" advice; add indexed struct arrays and a static pointer table | H2a, H3a |
| `tables.c80` (new) | `&global`, `&array[K]`, `&s.field` in static initializers; a linked list and a sprite table | H3a |
| `inline.c80` (new) | the same helper as a call and `@inline`, with both listings and the T-state difference pinned | H3i |
| `pong/` | `<< 6` instead of six adds; then hot helpers `@inline`/strict with pins; comments updated each time | H3, H3i |
| `optimize.c80` | revise to show what the H5 passes do; keep the `--no-optimize` comparison | H5 |

**Gate (checked at every increment from HX1 on):** every example has at
least one `expect` line per function that returns a value; all `expect`,
budget, and golden checks pass in the modes they declare; a feature is not
"done" until an example teaches it and a golden listing pins its code.

### H6 — Documentation

**Outcome:** done. `info/c80.md` is organised by topic, every C80 block in
it and in the tutorial is compiled by `c80/tests/docs.rs`, and the tutorial
has one section per example plus "When the compiler says no". H6 also
finished the item 8 cascades, made generated labels unable to read as
numbers (F-035), and cut the test time.

1. **Restructure [info/c80.md](../../info/c80.md) by topic:** lexical form,
   grammar, types and conversions, expressions and operators, statements,
   data (arrays, pointers, `str`, structs), functions and calling conventions,
   frames and stack bounds, inline assembly, CPU builtins, units and
   manifests, CLI and outputs, mixed BASIC projects, optimization, limits and
   diagnostics, library entry. Drop the E-number headings; keep the facts.
2. **Add "Limits and diagnostics":** every `cg-*`, `ty-*`, `parse-*`, `ln-*`
   code with its meaning and the usual fix; frame displacement limit;
   unsupported operators; what forces an IX frame and what it costs.
3. **Add "Generated code":** the ABI table, when a frame appears, a short
   annotated listing, and how to read `--emit-asm` output.
4. **Fix** the CLI link, the shift status, and the "Explicitly unsupported"
   list so it matches the compiler exactly. Generate that list's test from
   `KNOWN_GAPS` so the two cannot drift.
5. **Tutorial:** one section per example file in HX's table, each linking
   the source and its golden listing, plus a "when the compiler says no"
   section and the updated pong walkthrough. The example updates themselves
   belong to HX and their owning increments, not to H6.
6. **Repo docs:** [development skill](../skills/development/SKILL.md) (new test
   commands), [info/rtvc.md](../../info/rtvc.md) if build targets or
   dependencies change, [TODO.md](../../TODO.md), and one line per
   user-visible change in [CHANGES.md](../../CHANGES.md).
7. **Hungarian:** see Open Decisions.

Do H6 items 4 and 2 (accuracy) alongside H1–H3 as behaviour changes; the
restructure waits until H3 so it is written once.

8. **CLI diagnostics need a location** (found in H2b): `rtvc-c80` prints the
   code and message without `file:line:column`, although the span is in
   `CompilationResult`. Print `path:line:col: code: message` and the source
   line. Also stop cascades seen in HX1 (one error per undeclared or
   unassigned local, not one per use). Do this early, with H2c, since every
   new diagnostic depends on it.

**Gate:** every code block in `c80.md` and the tutorial that claims to
compile is compiled by a test; no doc statement contradicts `KNOWN_GAPS`.

## Order and Dependencies

| Step | Depends on | Unblocks |
| --- | --- | --- |
| H0 | — | everything |
| H1 | H0 | H2, H3, H4 |
| H2 | H1 (slot ownership, liveness) | H3 (word shifts need scratch), H5 |
| H3 | H2 | H4, H6 restructure |
| H3a | H1 (independent of H2/H3 otherwise) | H4 self-check data |
| H3i | H2 (strict mode, clobber sets), H3 | H4, H5 |
| H4 | H1 (useful earlier with a smaller self-check), H3i for full coverage | — |
| HX1 | H0 | golden listings for every later increment |
| H5 | H1–H3i | — |
| H6 | accuracy fixes with each step; restructure after H3 | — |

## Validation Commands

```bash
cargo test -p rtvc-c80 --lib
cargo test -p rtvc-c80 --test rtvc_c80
cargo test -p rtvc-c80 --bin rtvc-c80
cargo test --bin rtvc-tocas --no-default-features --features cli-tools
sh info/c80/compile.sh
git diff --check
```

After H3 and H5, run a mixed cassette on the emulated TVC:

```bash
cargo run --bin rtvc -- --headless --port 8080 snapshots/boot12dos.rtvcsnap.zip -i info/c80/out/tvc-usr.cas
```

and capture the screen with `scripts/rtvc_debug.py` (`screenshot`). The
expected output is `42` and `-32767`.

## Non-Goals

- New language features, including `*`, `/`, `%`, local arrays, recursion,
  by-value aggregates, and a runtime library.
- SSA, graph colouring, or changes to the argument-passing ABI.
- Phase 2 editor and debugger integration, `--emit-map`, and E14 compression.
- Changing the ABI, struct layout, source-order placement, or the
  observability of memory and port accesses.

## Decisions (resolved by the user, 2026-11-13)

1. **Multiply/divide.** Allow `*` by a compile-time constant only (shift-add,
   no runtime helper), as a follow-up after H3. `/`, `%`, and `*` by a
   variable stay unsupported. This is outside H0–H6.
2. **Hungarian C80 reference.** Translate after the H6 restructure, not
   before.
3. **Ledger strictness.** After H3, no `Rejected` gap may be added back to
   `KNOWN_GAPS` without the user's sign-off.
4. **Addresses in static initializers** are allowed (H3a).

## Risks

- **The allocator may not carry spills cleanly.** Locations live in two maps
  keyed by vreg and local, with aliasing between them (the cause of D1). If H2
  needs more than local changes to `relocate`/`ensure_*`, stop and record a
  finding proposing explicit value numbering (one owner per register, locals
  and vregs as names for a value) before continuing. Do not patch call sites
  one by one.
- **Flag-liveness mistakes in H5** reintroduce wrong code. The H1 guard does
  not see flags. Mitigation: peepholes run only on patterns whose flag effect
  is proven, the random-expression suite runs in both modes, and each
  peephole lands separately.
- **Static slots change the design's stated default.** The original design
  says locals that need memory "initially use an IX-relative frame" and that
  static allocation of temporaries is not required. H2 moves to static slots
  on the user's direction; update that paragraph of
  [c80-compiler.md](c80-compiler.md) when H2 lands.
- **Clobber sets couple callers to callee bodies.** A callee edit can change
  a caller's code. That is acceptable for whole-project builds but makes
  byte-exact caller fixtures sensitive; keep such fixtures small.
- **Byte-exact fixtures** in `codegen_tests.rs` (six `func_bytes` checks) will
  change in H5. Update them deliberately with the before/after recorded, never
  to make a failing semantic test pass.

## Appendix A — Program Fixtures for H0

Entry point is `f`; expected results are for the listed arguments.

```c
// array sum -> 36
u8 t[8] = {1,2,3,4,5,6,7,8};
u8 f() { u8 s = 0; for (u8 i = 0; i < 8; i++) { s += t[i]; } return s; }

// u16 array index: f(2) -> 700
u16 t[5] = {100,200,300,400,500};
u16 f(u8 i) { return t[i] + t[i + 1]; }

// u16 array store: f(3) -> 0x2469
u16 t[5];
u16 f(u8 i) { t[i] = 0x1234; t[i+1] = t[i] + 1; return t[i] + t[i+1]; }

// struct array: f(3) -> 1021
struct P { u8 x; u16 w; u8 y; };
P ps[4];
u16 f(u8 i) { ps[i].w = 1000; ps[i].x = 7; ps[i].y = 9; ps[1].w = 5;
              return ps[i].w + u16(ps[i].x) + u16(ps[i].y) + ps[1].w; }

// nine byte locals: compare with the same arithmetic in Rust
u8 f(u8 a, u8 b) { u8 c = a + b; u8 d = c ^ a; u8 e = d + 1; u8 g = e & 0x3f;
                   u8 h = g | b; u8 i = h - c; u8 j = i + d + e;
                   return a + b + c + d + e + g + h + i + j; }

// seven word locals: same shape with u16 and 0x3fff

// nested calls: f(50, 8) -> 99
u8 add(u8 a, u8 b) { return a + b; }
u8 sub(u8 a, u8 b) { return a - b; }
u8 f(u8 x, u8 y) { return add(sub(x, y), add(add(x, 1), sub(y, 2))); }

// call in loop: f(10) -> 385
u16 sq(u16 n) { u16 s = 0; u16 i = n; while (i != 0) { s += n; i -= 1; } return s; }
u16 f(u8 n) { u16 t = 0; for (u8 i = 1; i <= n; i++) { t += sq(u16(i)); } return t; }

// loop counter across a clobbering callee: f(5) -> 5
u8 g(u8 x) { asm(inout: a = x, clobber: b, c, d, e, h, l, flags) { ld bc,0
             ld de,0
             ld hl,0
             inc a } return x; }
u8 f(u8 n) { u8 s = 0; while (n != 0) { n = n - 1; s = g(s); } return s; }

// stackcall: f(100) -> 204
@stackcall u16 m(u16 a, u16 b, u16 c, u16 d, u8 e) { return a + b - c + d + u16(e); }
u16 f(u16 x) { return m(x, 2, 3, x, 5); }

// pointer walk: f(16) -> 136
u8 buf[16];
u16 f(u8 n) { ptr<u8> p = &buf[0]; u8 i = 0;
              while (i < n) { *p = i + 1; p += 1; i++; }
              u16 s = 0; for (u8 k = 0; k < n; k++) { s += u16(buf[k]); } return s; }

// signed loop: f(10) -> 110
i16 f(i16 n) { i16 s = 0;
               for (i16 i = -n; i <= n; i += 1) { if (i < 0) { s -= i; } else { s += i; } }
               return s; }

// bubble sort -> 1 + (9 << 4)
u8 a[6] = {5,3,9,1,7,2};
u8 f() { for (u8 i = 0; i < 5; i++) { for (u8 j = 0; j < 5 - i; j++) {
           if (a[j] > a[j+1]) { u8 t = a[j]; a[j] = a[j+1]; a[j+1] = t; } } }
         return a[0] + (a[5] << 4); }

// shift-add multiply: f(123) -> 4551
u16 mul(u16 a, u16 b) { u16 r = 0;
                        while (b != 0) { if ((b & 1) != 0) { r += a; } a <<= 1; b >>= 1; }
                        return r; }
u16 f(u16 x) { return mul(x, 37); }

// str walk -> (72+69+76+76+79) & 0xFF
str s = "HELLO";
u8 f() { u8 t = 0; for (u8 i = 0; i < s.len; i++) { t += s[i]; } return t; }
```

A linked-struct walk (`N a = {1, &b};`) is rejected today with "static
initializer must be a compile-time value". Address-of-global in a static
initializer is not in the language reference either way; record it in the
ledger as `Rejected` and raise it as a finding rather than deciding it here.
