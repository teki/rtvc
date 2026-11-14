# C80 Compiler Hardening — Progress

Plan: [c80-hardening.md](c80-hardening.md). Findings:
[c80-compiler-findings.md](c80-compiler-findings.md) (F-018 onward).

## Status

- **H0 — Differential conformance suite:** done (2026-11-13). No compiler
  behaviour changed; only tests were added.
- **H1 — No silent miscompiles:** done (2026-11-13). The ledger has no
  `WrongValue` entry and no random `W`; a permanent test asserts both.
- **HX1 — Examples as tests:** done (2026-11-13). No compiler behaviour
  changed; the examples gained directives and golden listings.
- **H2a — Spill instead of reject (static slots, saves):** done (2026-11-13).
  No unannotated function uses IX; every prog Pressure entry passes.
- **H2b — Strict `@fastcall`, pins, `static` locals:** done (2026-11-13).
  Unannotated functions emit the same bytes; every golden listing except the
  new `registers.asm` is unchanged.
- **H2c — Register contracts and CLI locations:** done (2026-11-13).
  `@preserves`, inference for private callees, harness contract checks, and
  located CLI diagnostics. Bytes changed only where a private callee leaves
  registers alone (listed below).
- **H3 — Shifts:** done (2026-11-13). Constant and variable shifts on all
  four types; the ledger has no rejected shift.
- **H3a — Addresses in static initializers:** done (2026-11-13).
- **H3i — Inlining:** done (2026-11-13). `@inline`, automatic inlining,
  `@noinline`.
- **H4 — End-to-end TVC check:** done (2026-11-14). No compiler change; the
  self-check battery and Pong pass on an emulated TVC 1.2 in both modes. New
  finding F-032 (rejected, not miscompiled).
- **H5 — Code quality:** done (2026-11-14). Every quality-table row improved
  or held; F-032 to F-036 resolved.
- **H6 — Documentation:** done (2026-11-14). Reference reorganised by topic,
  documentation code blocks and links tested, plus three leftovers
  (diagnostic cascades, label scheme, test time).

**Final state.** `cargo test -p rtvc-c80`: lib 240, bin 1, docs 2,
examples 4, rtvc_c80 9, tvc 4, all passing; `KNOWN_GAPS` and `RANDOM_GAPS`
empty. The final quality table is under [H5 Results](#h5-results) (H6
changed no generated instruction). Open findings: F-029 (C80 cannot call
ASM exports). Follow-ups are listed in [TODO.md](../../TODO.md) under
"Mature the C80 compiler".

## H0 — What Was Done

- [conformance_tests.rs](../../c80/src/conformance_tests.rs), registered in
  [lib.rs](../../c80/src/lib.rs) as a `#[cfg(test)]` module. Every case is
  compiled at origin `0x8000` (`GenericZ80`) with `optimize` true and false and
  `f` runs through `harness::execute_function_with` (2,000,000 instructions /
  50,000,000 T-states). A case passes when it does not time out, IX is `0x1111`,
  IY is `0x2222`, and `A` (byte/bool) or `HL` (word) matches a Rust reference.
- Groups, one `#[test]` each so they run in parallel:
  - `op.<T>` for `u8 i8 u16 i16`: `+ - & | ^` and their compound forms,
    `<< >>` with a variable `u8` count (byte values plus 9, 15, 16, 17), their
    compound forms, constant counts 0, 1, 3, 7, 8, 9, 15, 16, 17, 100, 255, the
    six comparisons, `-a`, `~a`, `!bool(a)`, `!(a < b)`. `op.bool`: `!`, `==`,
    `!=`, `&&`, `||`, a mixed expression. `cast`: all 25 pairs of
    `bool u8 i8 u16 i16`.
  - `shape.<T>`: `(a OP b) OP2 (a OP3 b)`, the same through two locals, a
    single parameter feeding two locals, a local reused with both parameters,
    and `s = a OP b; return s CMP a;`.
  - `prog`: the Appendix A fixtures, a word variant of the clobbering-callee
    loop, a shift-free bubble sort, a runtime-linked struct walk, and minimal
    repros of D1, D3, D5, F-024, and F-025.
  - Seeded random expressions (own LCG, seed `family seed + type index`):
    `rand` (plan: depth ≤ 4, 300 per type), `rsmall` (depth ≤ 2, 300 per type,
    added because `rand` is almost entirely rejected for register pressure),
    and `rshift` (depth ≤ 3 with shift nodes, 100 per type). Programs are
    `T f(T a, T b) { T c = E; T d = E; return E; }`; nodes include
    `T(l CMP r)` and `T(U(l) OP U(r))` for every integer `U`.
- Ledger: `KNOWN_GAPS` holds `(case id, Rejected | WrongValue, cause)`; an id
  without suffix covers both modes, `@opt`/`@base` one mode (none needed: both
  modes agree on every case today). A `Rejected` entry's first error must
  match its cause. `RANDOM_GAPS` stores one `.`/`R`/`W` character per random
  program and mode, compared exactly. Stale ids, extra passes, and changed
  kinds all fail. `C80_CONFORMANCE_DUMP=1` prints the observed ledger lines
  (`C80_CONFORMANCE_DUMP_DETAILS=1` adds the failing random sources).
- [c80/tests/rtvc_c80.rs](../../c80/tests/rtvc_c80.rs)
  `tutorial_examples_build_without_errors`: every `info/c80/*.c80` at origin
  `0x8000` and every `info/c80/*/rtvc-c80.toml` (loaded like the CLI's
  `compile_input`), in both optimize modes.

## H0 Ledger Baseline

`KNOWN_GAPS`: 518 entries, 219 `Rejected` and 299 `WrongValue`.

| Kind | Cause | Entries |
| --- | --- | --- |
| Rejected | register pressure (D5, F-022) | 114 |
| Rejected | shift typing, `u16`/`i8`/`i16` (D4, F-021) | 72 |
| Rejected | shift codegen, `u8` and compound forms (D4, F-021) | 32 |
| Rejected | address in static initializer (F-023) | 1 |
| WrongValue | live local in `A`/`HL` overwritten (D1, F-018) | 291 |
| WrongValue | loop-carried register local across a call (D3, F-020) | 3 |
| WrongValue | spill slot collision (D2, F-019) | 1 |
| WrongValue | signed byte compare stale scratch (F-024) | 2 |
| WrongValue | byte view of `HL` destroyed (F-025) | 2 |

Only `array_sum` and `str_walk` of the Appendix A fixtures pass today. All
operator-matrix and cast cases pass except shifts.

Random families (identical in both modes):

| Family | u8 pass/R/W | i8 | u16 | i16 |
| --- | --- | --- | --- | --- |
| `rand` (300) | 37/257/6 | 27/263/10 | 6/294/0 | 4/295/1 |
| `rsmall` (300) | 158/108/34 | 116/138/46 | 26/273/1 | 22/278/0 |
| `rshift` (100) | 7/93/0 | 6/93/1 | 1/99/0 | 1/99/0 |

Every random rejection is register pressure or a shift. Of the 99 random
`WrongValue` programs, 93 pass when every variable is a global instead
(consistent with D1), 2 still fail (F-024, F-025), and 4 are then rejected for
pressure (unclassified).

## Validation

- `cargo test -p rtvc-c80`: lib 141 passed (117 existing + 24 new), bin 1
  passed, `tests/rtvc_c80.rs` 8 passed (7 existing + 1 new), doc 0; 0 failed.
  Wall time about 8.4 s with a warm build; the conformance tests take about
  1 s of that.
- Ledger sensitivity checked by hand: removing an entry, changing a kind or
  cause, adding an unknown id, and editing one random character each made the
  suite fail; the file was then restored.
- `cargo fmt -p rtvc-c80 -- --check` and `git diff --check` clean.

## H1 — What Was Done

All changes are in [lower.rs](../../c80/src/lower.rs) unless noted.

1. **Clobber guard.** [z80.rs](../../c80/src/z80.rs) `Z80Op::writes()`
   gives the 8-bit registers each op writes (`Call` writes all; `OR A`,
   `AND A`, and `CP` write none). `Lowerer::emit` checks each non-synthetic
   op: an occupant of a written register that is live after the current IR
   op and is not being defined by it is a `cg-internal` error at the op's
   span ("internal: live value in A would be overwritten by `…`"). Inline asm
   checks its dirty set after the input moves. Ops that redefine keys on
   purpose push them to `defining`: `StoreLocal`, home restores, and the
   borrowed byte in the signed word compare. `finish_op` clears the per-op
   state and returns the error. The test
   `clobber_guard_reports_unparked_live_values` switches parking off through
   `lower::test_hooks` and checks that the guard fires on byte and word D1.
2. **D1 / F-018, F-025.** In-place lowerings use `scratch_a`, `scratch_hl`,
   or `scratch_rr`:
   - Live occupants of the register leave it first (`park_live`). This
     covers locals and `H`/`L` byte views, and moves every key bound to the
     same location together (`relocate` is group-aware).
   - The register is then reserved for the rest of the op.
   - A dead operand is copied without rebinding.

   This covers byte and word binary ops, unary ops, casts (bool, signed and
   unsigned widen), `ADD HL,imm`, word loads and stores through a pointer
   (`INC HL` walks a copy), and compares. Other changes:
   - `StoreLocal` into an existing home first moves live copies of the old
     value out.
   - When `HL` cannot be freed, word `+ - & | ^` go through `A` into the pair
     of an operand that dies (`word_alu_in_pair`).
   - `relocate` can free a pair by moving byte values to free single
     registers (`make_pair`).
   - A local that is dead at a point does not occupy its register.
3. **D2 / F-019.** `alloc_spill` takes the first slot not used by any current
   binding; `spill_keys` gives one slot per location. The per-call
   `next_spill` restart is gone (`lower_call`, `lower_ldir`, inline asm, and
   asm capture slots). The spill area size is unchanged.
4. **D3 / F-020.** `local_live_after` computes a backward fixed point over
   the CFG and gives the live-after set of locals for every op.
   `live_across_any_call`, `locals_needing_frame`, `drop_dead_after`, the
   call, `ldir`, and inline-asm save loops, and eviction use it. Vregs keep
   the linear index; they are block-local (`vreg_loc` is cleared per block,
   and a vreg without a location is an internal error).
5. **F-024 and the stale-location audit.** The signed byte compare biases
   scratch copies only. The signed word compare does the same: it copies the
   operand when another value shares the pair, and otherwise flips in place
   and back behind `PUSH AF`/`POP AF`. Equality compares are unbiased and
   symmetric. The audit fixed:
   - `ensure_in_r8`/`ensure_in_rr` and the unsigned widen now re-read the
     source after the claim (the widen read was really stale once `make_pair`
     existed: `rsmall.u8.42`);
   - `make_pair` never moves the kept key;
   - `emit_move` Byte→Word evicts `H` before writing `L` and never relocates
     into `L`;
   - `push_stack_arg` reserves its pair before loading `A`;
   - `lower_addr_local` reserves `HL` before claiming `A`;
   - the byte store reserves `A` before loading `HL`;
   - `assign_stable_homes` breaks a move cycle with the general parallel move
     instead of `unwrap_or(0)`.
6. **F-026 (new): overlapping stable homes.** `HomeRegs::plan` assigns
   non-overlapping homes. Words take `DE BC`, bytes take the rest (`L H` only
   when frame-free), and one pointer gets `HL` when everything fits. Unused
   parameters get no home. See
   [findings F-026](c80-compiler-findings.md#f-026--stable-register-homes-overlap).
7. **IX only when needed.** `lower_function` retries with an IX frame when
   the frame-free lowering runs out of registers; those cases were rejected
   before. A single-block function is no longer sent to IX by the stable-home
   count. No function of the tutorial examples gained a frame; `copy_block` in
   `ports.c80` lost one.

## H1 Ledger

`KNOWN_GAPS`: 115 entries, all `Rejected` (was 518: 219 `Rejected`, 299
`WrongValue`). It was regenerated from a real dump run. Kind changes:

| Before | After | Entries |
| --- | --- | --- |
| WrongValue D1 | Pass | 291 |
| WrongValue D3 | Pass | 3 |
| WrongValue F-024 | Pass | 2 |
| WrongValue F-025 | Pass | 2 |
| WrongValue D2 | Pass | 1 |
| Rejected Pressure | Pass | 104 |

No entry changed to a different failing kind, and no passing case became
`Rejected`. The remaining `Rejected` entries are shift typing (72), shift
codegen (32), register pressure (10), and the static-address initializer (1).
The pressure entries are `u16_array_index`, `u16_array_store`,
`struct_array`, `seven_word_locals`, `call_in_loop`, `signed_loop`,
`bubble_sort`, `bubble_sort_no_shift`, `shift_add_multiply`, and
`linked_struct_runtime`. Both optimize modes agree on every case.

Random families (pass/R/W, both modes identical):

| Family | u8 | i8 | u16 | i16 |
| --- | --- | --- | --- | --- |
| `rand` (300) | 86/214/0 | 76/224/0 | 17/283/0 | 16/284/0 |
| `rsmall` (300) | 268/32/0 | 248/52/0 | 149/151/0 | 152/148/0 |
| `rshift` (100) | 8/92/0 | 9/91/0 | 6/94/0 | 6/94/0 |

Of the 99 H0 `W` programs, 98 now pass and 1 (`rand.i8.45`) is rejected for
register pressure. No `.` became `R`.

New fixtures (all pass):
- `prog.d1_word_sum_and`;
- `prog.d1_signed_word_compare_loop` (the rhs home was flipped in place);
- `prog.d1_store_over_copy` (HEAD: `ADD A,5 / LD C,A / XOR C`, always 0);
- `prog.d1_postfix_old_value`;
- `prog.d2_calls_keep_spills` (HEAD stored two results to `(IX-14)`);
- `prog.d3_loop_call_word`;
- `prog.f026_overlapping_homes`;
- `prog.f026_tutorial_buf`.

Named tests `h1_*` run each defect's repros in both modes.
`conformance_ledger_has_no_wrong_values` is the gate.

## H1 Code Size

The byte-exact fixtures in
[codegen_tests.rs](../../c80/src/codegen_tests.rs) are unchanged. Tutorial
examples (HEAD → H1, bytes), all other examples identical:

| Example | Change |
| --- | --- |
| `ports.c80` | 73 → 39 (`copy_block` lost its IX frame; it is `LDIR / RET`) |
| `memory.c80` | 87 → 79 |
| `structs.c80` | 120 → 112 |
| `mixed/` C80 payload | 564 → 529. `fill` 34 → 26 (and correct), `bump` 42 → 30, `sum` 83 → 68 (correct; keeps its 1-byte frame) |
| `pong/` C80 payload | 1353 → 1317. `serve` −9, `move_ai` −16, `ball_sync` −7, `physics` −2, `play` −2 |

T-states were not measured.

## Validation

- `cargo test -p rtvc-c80`: lib 152 passed, bin 1, `tests/rtvc_c80.rs` 8, doc
  0; 0 failed.
- `sh info/c80/compile.sh` succeeds.
- `cargo fmt -p rtvc-c80 -- --check` and `git diff --check` are clean.

## HX1 — What Was Done

- [c80/tests/examples.rs](../../c80/tests/examples.rs) runs the `//!`
  directives of every example in `info/c80/` in both optimize modes. The
  grammar is in [tutorial.md](../../info/c80/tutorial.md#examples-check-themselves).
  The forms are `expect`, `expect-mem` (memory, or `out(p)` port writes), the
  budgets `expect-bytes` and `expect-tstates`, `expect-skip` (with a reason),
  and `expect-error` (only in `*.fail.c80`). A `with` clause presets globals,
  raw bytes, or scripted `in(p)` reads.
- Malformed or unknown directives panic with `file:line`.
- Budgets check only the optimized build. `expect-tstates` runs with zero
  arguments, counts the final `RET`, and is rejected when the listing has a
  jump, call, `DJNZ`, conditional `RET`, or `LDIR`/`LDDR`.
- Coverage gate: every value-returning function in an example source has at
  least one `expect`, or an `expect-skip`.
- There are five fail files: `scalars`, `control`, `calls`, `memory` and
  `inline_asm` `.fail.c80`. The test compares the reported error codes with
  the `expect-error` list as a multiset, and requires a span on each error.
  `compile.sh`, `compile.bat` and `tutorial_examples_build_without_errors`
  skip these files.
- [info/c80/expected/](../../info/c80/expected/) holds 15 golden listings:
  10 single-file examples, 4 projects, and `optimize.unopt.asm`. They are
  byte-identical to `compile.sh`'s `out/*.asm`. `C80_UPDATE_EXPECTED=1`
  rewrites them, and a stale golden file without an example fails the test.
  Labels come from node ids, so comment-only edits do not change a listing.
- Perturbation checks:
  - Editing `expected/hello.asm` fails with the first differing line.
  - An extra `expected/bogus.asm` fails.
  - Dropping `id`'s expect fails the coverage gate.
  - A wrong value fails in both modes.
  All were restored afterwards.
- Every example has new header and function comments.
  - Stale or wrong comments were fixed:
    - pong's "six ADD A,A": it is `LD D,H / LD E,L / ADD HL,DE`, six times.
    - pong's claim that `fill_rows(y, oy-by)` is a bad ABI fit: the two-byte
      version compiles, with the same IX frame.
    - The tutorial said `scalars.c80` covers shifts.
  - Limits that still hold now use the H6 grep phrases on one line:
    - structs: "run out of registers";
    - pong: "no `<<` yet".

| Example | expect | expect-mem | bytes | tstates | skip |
| --- | ---: | ---: | ---: | ---: | ---: |
| hello | 3 | 0 | 2 | 2 | 0 |
| scalars | 9 | 0 | 0 | 0 | 0 |
| control | 15 | 2 | 0 | 0 | 0 |
| calls | 7 | 0 | 0 | 0 | 0 |
| stackcall | 4 | 0 | 0 | 0 | 0 |
| memory | 8 | 2 | 0 | 0 | 0 |
| structs | 4 | 2 | 0 | 0 | 0 |
| inline_asm | 5 | 0 | 0 | 0 | 0 |
| ports | 1 | 4 | 1 | 0 | 0 |
| optimize | 4 | 1 | 0 | 0 | 0 |
| project/math | 4 | 0 | 0 | 0 | 0 |
| tvc-usr/echo | 2 | 0 | 1 | 0 | 0 |
| mixed/buf | 6 | 2 | 0 | 0 | 0 |
| pong | 5 | 28 | 0 | 0 | 1 |

Fail files: `scalars` 2 errors, `control` 2, `calls` 1, `memory` 2,
`inline_asm` 1.

### HX1 Compiler Observations (not fixed)

All expected values were worked out from the semantics before running.
None revealed a wrong result or a guard hit.

- `enemies[i].x` (a struct array indexed in a leaf) is still `cg-unsupported`
  for register pressure. This is an H2 target.
- Frames are bigger than needed. `fill_rows` and `inline_asm::add_carry`
  reserve 14 bytes (`LD HL,65522`) but use 1–2. `inline_asm::inc8` reserves 2
  bytes and uses none.
- Some copies are redundant:
  - `a += a` on a u16 is `LD D,H / LD E,L / ADD HL,DE` rather than
    `ADD HL,HL` (pong `put_row`);
  - `stackcall::use_add` has `LD D,H / LD E,L` between its pushes;
  - `inc8` has `LD B,A / LD A,B`.
- Constants are not folded: `wrap_add` is `LD A,255 / ADD A,1`, and `sizes`
  is `LD HL,1` followed by three `INC HL`.
- A call in tail position is `CALL / RET`, not a jump.
- Diagnostics cascade:
  - after "local arrays ... not supported", every use of that local also
    reports `ty-use-before-assign`;
  - a local that is read before assignment is reported at every read, not
    once.

## HX1 Validation

- `cargo test -p rtvc-c80`, 8.4 s wall with nothing to rebuild (21.9 s
  including a rebuild); `tests/examples.rs` alone runs in 1.2 s: lib 152, bin 1,
  `tests/examples.rs` 4, `tests/rtvc_c80.rs` 8, doc 0; 0 failed.
- `sh info/c80/compile.sh` succeeds.
- `cargo fmt -p rtvc-c80 -- --check` and `git diff --check` are clean.

## H2a — What Was Done

- [lower.rs](../../c80/src/lower.rs): non-`@stackcall` functions no longer
  build an IX frame. A value that does not fit in a register goes to a static
  slot `Loc::Slot`, addressed as `NAME_S+n`. The slot area is emitted right
  after the function as a synthetic `DB 0,...` item: it is part of the
  function's size, zero in the image, and placed by the normal linker like any
  label. `@stackcall` keeps IX for its arguments (`frame_error` still rejects
  displacements past +127).
- Values live across a `CALL`, `LDIR`, or `asm` block are saved by
  `save_around`, choosing the cheapest option (table below). Pushed pairs
  that come back holding a result are popped into a free pair or spilled.
- Eviction (`free_r8_or_spill` / `free_rr_or_spill`) chooses the value with
  the farthest next use, spills it, and reloads it on its next use. A register
  that holds an operand of the current op is never evicted
  (`reg_holds_operand`).
- Homes: `plan_homes` tries the H1 plan (Packed) first. For a stable
  register-convention function that still needs a fallback, it also tries
  Spread (weighted, keeps HL free) and Dense (weight per register). It keeps
  the lowest weighted T-state cost (`approx_tstates`, loop blocks ×8); a tie
  keeps the earlier plan. Locals that have their address taken, or that get
  no register, get word or byte slot homes.
- Parallel moves break a cycle through a slot when no temporary register is
  free.
- Stack bounds: `emit` tracks the push depth (PUSH/POP/AF/IX/INC SP/DEC SP).
  The depth at each `CALL` is the call-site depth; the peak is `asm_stack`.
  The conformance runner and `tests/examples.rs` fail when `sp_used` is
  greater than a Proven bound.
- Visibility: every fallback instruction carries `; spill`. A function that
  has any ends with `; spill report: N slot bytes, saves (bytes per site) ...,
  T T-states of spill code`. `GeneratedFunction.spill` (`SpillReport`) holds
  the same numbers, and the slot `DB` and spill instructions are synthetic
  map entries.
- Side effect: dead compare operands and pushed stack arguments are now
  marked consumed, which removes useless `LD r,A` / `LD B,D` / `LD C,E`
  relocations.
- Examples: new [spills.c80](../../info/c80/spills.c80).
  [structs.c80](../../info/c80/structs.c80) adds `nudge` (indexed
  `enemies[i].x`, spills). [memory.c80](../../info/c80/memory.c80) adds
  `words[4]` and `pair_sum` (a 2-byte slot). Comments in `calls.c80` and
  `pong.c80` were updated. The tutorial has a new section 10, "When values do
  not fit", and sections 11–14 were renumbered. Goldens were regenerated, and
  the diffs (calls, inline_asm, memory, mixed, pong, stackcall, structs) were
  reviewed: IX frames are replaced by PUSH/POP or slots, and dead moves are
  removed.
- Tests: `many_word_locals_use_static_slots_not_ix`,
  `spills_reserve_exactly_their_slots`,
  `stackcall_arguments_spill_when_pairs_run_out`.
  `indexed_frame_boundary_is_rejected` is now a `@stackcall` case. The
  synthetic-map test expects the slot `DB`. `run_mode` asserts that no IX is
  used outside `@stackcall`.

### Fallback cost table (T-states, save + restore)

| Fallback | Cost | Used when |
| --- | ---: | --- |
| move to a free register not touched by the clobber | 8 | a register is free across the site |
| A into a free half of a pair being pushed anyway | 4 + pair | that pair is already pushed |
| `PUSH rr` / `POP rr` | 21 | a pair survives a call |
| `PUSH AF` / `POP AF` | 21 | A alone, no flags needed after |
| A into a half of another pair, then push it | 25 | |
| byte slot `LD (nn),A` / `LD A,(nn)` | 26 | eviction, or a site with no other option |
| word slot `LD (nn),HL` / `LD HL,(nn)` | 32 | |
| word slot through BC/DE (`ED` forms) | 40 | |

Within a loop, every cost is weighted ×8 when homes are chosen.

## H2a Ledger

- Random gaps: 3520 rejections across modes before (710 remaining after H1
  plus 2810 Pressure) → 710 after. All of the remaining ones are `rshift.*`
  (ShiftCodegen). `RANDOM_GAPS` was regenerated from a real run.
- Prog ledger: 11 entries before (10 Pressure + `linked_struct_static`
  StaticAddr) → 3 after: `bubble_sort` and `shift_add_multiply`
  (ShiftCodegen), `linked_struct_static` (StaticAddr). Every Pressure entry
  passes. `Cause::Pressure` is kept (`#[allow(dead_code)]`) with its sources
  for H2b.
- No `W` or `R→W` in any mode.

## H2a Code Size and Speed

Of 3074 conformance functions, 2954 were frame-free after H1. Of these, 877
changed, all smaller: 558 by 1 byte, down to 2 functions by 10 bytes. Two
more changed addresses only. Every removed byte is a dead relocation move
(`LD r,A`, `LD B,D`, `LD C,E`). No frame-free function got bigger. The 120
functions that used IX went from 11208 to 7196 bytes, and none got bigger.

| Measurement | Bytes before → after | T-states before → after |
| --- | ---: | ---: |
| pong `fill_rows(10)`, h=2 | 59 → 28 | 988 → 710 |
| pong `fill_rows(10)`, h=24 | 59 → 28 | 9722 → 7926 |
| pong `put_row(3)` | 43 → 43 | 228 → 228 |
| structs `hits(100)` | 50 → 48 | 171 → 163 |
| structs `step(&enemies[0], 2)` | 26 → 26 | 225 → 225 |
| calls `add_carry(200, 100)` | 58 → 40 | 277 → 165 |
| calls nested `f(50, 8)` | 69 → 39 | 472 → 320 |

Pong C80 code + globals: 1352 → 1170 bytes. `pong.cas` payload: 1317 →
1135 bytes.

## H2a — Where the Pressure Diagnostic Remains

The message is now "register pressure in 'f': no register can be freed for
this operation". It is raised only:

- by `free_r8_or_spill` / `free_rr_or_spill` when every candidate register is
  reserved, blocked by a pending move, or holds an unconsumed operand of the
  current op;
- by `push_pair_or_spill` when no remaining `@stackcall` argument is in a
  register;
- as internal checks: `relocate` finding no location, `word_alu_in_pair`, the
  first arm of casts, and `store_indirect`.

No conformance case or example reaches any of these. `frame_error` (the
`@stackcall` displacement limit) is separate.

### H2a Observations (not fixed)

- Static slots make a function non-reentrant, both for recursion and when it
  is called from an interrupt handler while also running in the main program.
  Ordinary nested calls are fine, since each function has its own slots.
  This is documented in `info/c80.md` and in the plan paragraph.
- `nudge` and `pair_sum` spill because address arithmetic takes three pairs.
- `add_carry` stores the flag slot and then loads it straight back.

## H2a Validation

- `cargo test -p rtvc-c80`: lib 155, bin 1, `tests/examples.rs` 4,
  `tests/rtvc_c80.rs` 8, doc 0; 0 failed. 19.5 s wall with nothing to rebuild,
  about 35 s with a rebuild (lib alone 14.1 s).
- `cargo test --bin rtvc-tocas --no-default-features --features cli-tools`:
  8 passed.
- `sh info/c80/compile.sh` succeeds (`pong.cas` 1135 payload bytes; the mixed
  and tvc-usr projects link).
- `cargo fmt -p rtvc-c80 -- --check` and `git diff --check` are clean.

## H2b — What Was Done

Spellings are decided in F-027: `type name @ reg`, `static` inside function
bodies only, and `@fastcall` as the strict flag.

**Test time.** `cargo test -p rtvc-c80` with nothing to rebuild takes 7.3–7.5 s
(it was 19.5 s after H2a). The lib tests take 4.3 s (14.1 s before). No test was
removed or made smaller. Almost all of the time went to compiling the random
`i16` families: about 10 s in `rand.i16`.
- `shorten_jumps` ([optimize.rs](../../c80/src/optimize.rs)) now converts
  every forward conditional `JP` that fits in one pass. Converting a jump
  only shortens the others, so this reaches the same fixed point as
  converting one jump per pass. The next pass's assemble doubles as the
  check: if it fails, the batch is reverted and a `cg-internal` error is
  reported. The final assembly is returned to `lower_program` and reused
  instead of being assembled again.
- The listing mapper ([lower.rs](../../c80/src/lower.rs) `map_z80_item`)
  reuses one `FakeBus` per mapping run instead of allocating one per
  instruction.

The goldens are byte-identical across these changes.

**Strict `@fastcall`** (`Function::strict_registers` → `GeneratedFunction::strict_registers`):
- `emit` rejects any `; spill` note, `PUSH`/`POP` (including `AF`), `IX`, and
  `INC SP`/`DEC SP`.
- Earlier, more specific errors come from `spill_loc` (no free register),
  `save_around_inner` (value live across a call or clobbering op), and
  `plan_homes` (a local without a register home, or an address-taken local).
- The ABI is unchanged. Pong `play`, `echo`, `fill`, and `bump` compile
  strict. `mixed/buf.c80` `sum` does not: it needs one static slot. It lost
  `@fastcall`, and its comment explains why. Its listing is unchanged.

**Pins** (`Param.pin`, `VarDecl.pin` → `TypedLocal.pin: RegPin`):
- Semantics checks width, overlap with any in-scope pin, `&pinned`, and a
  pinned `const`. It records scopes as flat op ranges: the whole function
  for a parameter, and for a local from its initializing store to the end
  of its block.
- Lowering:
  - Homes are fixed to the pin. The home planners exclude pinned registers.
  - Every candidate search (`spill_or_swap`, `free_*_or_spill`, `make_pair`)
    skips pinned registers. `relocate`, `spill_loc`, and saves report
    `pin_conflict` instead of moving a pin.
  - A pinned `@stackcall` parameter is loaded from its frame slot once. To
    allow this, the prologue is now emitted before home planning.
- A pin owns its register while in scope (`pin_occupies`). The one
  exception is when its old value is dead and a later store in scope
  redefines it: then the new value may be computed in the pin's register.
  The guard sees the same liveness through `local_live`.

**`static` locals** are hidden globals:
- Symbol key `func\0id`, reported as `func.local`, label `G<id>_FUNC_LOCAL`.
- Placed through `anon_globals`, right after the function and its slot area.
- `check_global` handles the initializer and layout. Arrays and structs work.
- Access is an ordinary global load or store and is never coalesced.
- An automatic local array or struct is still rejected, and the message now
  suggests `static`.

**Examples and docs:**
- [registers.c80](../../info/c80/registers.c80), plus
  [registers.fail.c80](../../info/c80/registers.fail.c80) for declaration
  errors and [registers_cg.fail.c80](../../info/c80/registers_cg.fail.c80)
  for pin conflicts and strict errors. There are two fail files because
  code generation does not run on a file with type errors.
- Tutorial section 11 (later sections renumbered).
- [info/c80.md](../../info/c80.md): keywords, grammar, and a new section.
- Comments in `calls.c80`, `echo.c80`, `pong.c80`, and `memory.fail.c80`.

**Tests:**
- Codegen:
  - `pinned_counter_stays_in_b_through_the_loop` (byte-exact);
  - pinned parameter entry moves (register and `@stackcall`);
  - pin conflicts (including `A` and an ALU op);
  - disjoint pins sharing a register;
  - a pin across a call (pushed by default, rejected under strict);
  - strict accepts and rejects;
  - statics persist, initialize, index, and address;
  - static rule errors.
- Conformance:
  - `STRICT_REJECTIONS`: the 8 former pressure sources with `@fastcall`
    added must be rejected with "must fit in registers", while the plain
    versions pass, in both modes;
  - `h2b_*` program fixtures: a pinned loop, a pin across a call, a static
    counter, a static array, and a strict leaf.
- The conformance IX check now matches whole tokens. It used to flag a
  function named `mix`.

### H2b Diagnostics

| Code | Example message |
| --- | --- |
| cg-unsupported | `@fastcall 'sum_strict' must fit in registers, but 'n' has no free register at the cast u8 to u16; pin values to free registers, declare a local 'static', or remove @fastcall` |
| cg-unsupported | `@fastcall 'keep' must fit in registers, but 'y' is live across the call to 'twice', which may change every register; keeping it in a register needs a callee register contract (`@preserves`, not implemented yet); pin values ...` |
| cg-unsupported | `@fastcall 'f' must fit in registers, but 'x' has its address taken, so it needs memory; ...` / `... 'x' gets no register home (more locals are live than registers); ...` / `... the call to 'g' needs `PUSH BC`; ...` |
| cg-unsupported | `'s' is pinned to hl, but the u16 `+` needs hl for something else; pin 's' to another register, or remove the pin` |
| ty-mismatch | `'w' is u16 but 'a' is a byte register; a byte pins to a b c d e h l, a word or pointer to bc de hl` |
| ty-duplicate-name | `pin 'b @ bc' overlaps 'a @ b', which is still in scope` |
| ty-not-lvalue | `cannot take the address of 'v': it is pinned to register c; declare it 'static' to give it an address` |
| ty-mismatch | `a const has no storage and cannot be pinned` |
| ty-mismatch | `a static initializer must be a compile-time value; it is stored once in the image, not run on each call` |
| ty-mismatch | `local arrays and structs need storage; declare them 'static' (a fixed address after the function), or use a global or a pointer` |
| parse-expected | `'static' is for locals inside a function body; a global already has a fixed address` |
| parse-expected | `'static const' is not allowed: a const has no storage` / `a static local lives in memory and cannot be pinned to a register` / `a 'for' initializer cannot be static; declare the static before the loop` |
| parse-expected | `unknown register 'x' in a pin; use one of a b c d e h l bc de hl` / `a register pin applies only to a local or a parameter` |

### H2b Observations (not fixed)

- Every pointer load uses `LD A,(HL)`. A pointer pinned to `bc` or `de` is
  copied into `HL` for each dereference, and a word accumulator cannot be
  pinned to `hl` in a pointer loop. `LD A,(BC)`/`(DE)` would help, but they
  would change the bytes of unannotated functions, so they are left for H3.
- The CLI prints diagnostics without file and line (true before H2b as well).
  The spans are present in `CompilationResult`.

## H2b Validation

- `cargo test -p rtvc-c80`: lib 164, bin 1, `tests/examples.rs` 4,
  `tests/rtvc_c80.rs` 8, doc 0; 0 failed. Warm wall time 7.3–7.5 s.
  - The first run after `cargo fmt` rebuilt the binary. In that run
    `multi_unit_cli_build_emits_segments` failed once. It passed on its own
    and in the two full runs after it.
- `cargo test --bin rtvc-tocas --no-default-features --features cli-tools`:
  8 passed.
- `sh info/c80/compile.sh` succeeds, including `registers`.
  - `pong.cas` has 1135 payload bytes, the same as H2a.
  - The mixed and tvc-usr projects link.
- `cargo fmt -p rtvc-c80 -- --check` and `git diff --check` are clean.

## H2c — What Was Done

**A. CLI diagnostic locations.**
- `render_diagnostic` ([diagnostic.rs](../../c80/src/diagnostic.rs)) prints
  `path:line:col: severity code: message`, the source line, a `^~~~`
  underline, and the related spans as indented notes.
- `rtvc-c80` uses it for every diagnostic. For a single file, the path is
  the one given on the command line.
- Two unit tests check the format. Two CLI tests check the path for a
  relative `.c80` path and for a manifest unit.
- The cascade suppression part of H6 item 8 is not done.

**B. `@preserves`** (F-028).
- Parser: an attribute beside `@fastcall`/`@stackcall`.
- Semantics: result registers are rejected, pins may not use a preserved
  register, and the list is normalised to a mask
  (`TypedFunction::preserves`).
- Lowering: each preserved register is a pinned pseudo-local for the whole
  function (`preserve_pins`).
  - The H2b pin machinery then gives the required behaviour: no homes or
    scratch, a `pin_conflict` for an operation that needs the register,
    saves around clobbering calls and `asm` in default functions, and
    errors in strict ones.
  - Non-stable lowering must not drop these keys (`drop_dead_after`). Without
    that fix a call clobbered a preserved register unsaved. The bug was
    found in a smoke test before any test existed.
- `Z80Op::Call` carries the callee's clobbers. A call's changed set is the
  clobbers plus the argument and result registers. A `@stackcall` call is
  treated as changing everything.
- `GeneratedFunction::preserves` and `contract` (`Declared`, `Inferred`,
  `None`).
- Listing lines read `; preserves B C (declared)`. An inferred line is only
  shown when the function is called.
- Strict functions may now use a register-to-register move marked
  `; spill`.
- The strict "live across the call" message names the callee's preserved
  registers, or says it has none and suggests `@preserves(r)`.

**C. ASM exports:** skipped. C80 cannot call them at all (F-029), so a
manifest contract would have nothing to attach to.

**D. Inference.**
- Functions are lowered callees first (`callee_first`), and output order is
  unchanged.
- A private, undeclared function's contract is the registers its code never
  writes, minus its result registers (`contract_of`).
- A shuffle-only function now gets the `; spill report` line, so the
  "every function with `; spill` lines has a report" rule holds.

**Guard.**
- `execute_function_with` records the preserved registers on every `CALL`
  into a function with a contract, and on entry to the tested function.
- It compares them on the matching return.
- A broken contract is an `ExecError`: "function 'inc' broke its inferred
  register contract: B was A5 on entry and 00 on return".
- Every conformance and example execution therefore checks every contract
  it passes through.
- A test patches a contract to prove the guard fires, both on the entry
  function and on a nested call.

**Tests.**
- Codegen:
  - a strict counter across a preserving leaf (byte-exact);
  - violations: direct, through a callee, through `asm`, declarations, and
    parse errors;
  - a default function saving around a clobbering call;
  - a parameter in a preserved register, read in place and unchanged;
  - inference for private callees only;
  - source-order independence;
  - the guard firing;
  - a cross-unit call that saves without a declaration and does not save
    with one.
- Conformance: five `h2c_*` program fixtures, baseline and optimized.

**Examples:**
- [contracts.c80](../../info/c80/contracts.c80),
  [contracts.fail.c80](../../info/c80/contracts.fail.c80) and
  [contracts_cg.fail.c80](../../info/c80/contracts_cg.fail.c80).
- Updated comments in `registers.c80`, `registers_cg.fail.c80`,
  `spills.c80` and `pong.c80`.
- Tutorial section 12 (later sections renumbered), plus the section 4 and 10
  texts.
- `info/c80.md`:
  - a contracts section and the grammar;
  - CLI diagnostics;
  - the strict rule;
  - the ASM-export statement;
  - the CLI link (`../c80/src/bin/rtvc_c80.rs`).

### H2c Golden Changes

| Golden | Change | Why |
| --- | --- | --- |
| calls | `nest`: `PUSH BC`/`POP BC` gone (−2 bytes, −21 T); 6 `; preserves` lines | `id` is private and never writes `C`, so `x` stays in `C` |
| pong | `fill_rows` −2 bytes; `physics` +4 bytes; listing lines | see below |
| spills | `keep`: `PUSH BC`/`POP BC`/`ADD A,C` → `LD B,C`/`ADD A,B` (7 → 6 bytes, 25 → 8 T) | `twice` never writes `B` |
| memory, registers, stackcall | `; preserves` lines only | inferred contracts of called private functions |
| contracts | new | |

Pong: `fill_rows(10)` is 28 → 26 bytes, 710 → 668 T for h=2, and 7926 →
7422 T for h=24.
- `put_row` is private and never writes `B` or `C`, so the `PUSH BC`/`POP BC`
  around its call disappeared, as predicted.
- `put_row` itself is unchanged (43 bytes, 228 T).
- `physics` is 220 → 224 bytes, the only function that got bigger. At its
  two calls to `hits` the cost model now prefers moving `BC` to `D`/`L` and
  back (16 T, 4 bytes) over `PUSH BC`/`POP BC` (21 T, 2 bytes). At its two
  calls to `scored` it is `LD E,C`/`LD C,E` (8 T, 2 bytes).
- It is faster on every path and 4 bytes larger. This is the H2a
  T-states-first policy, not a regression in speed.
- Pong code + globals: 1170 → 1172 bytes; `pong.cas` payload 1135 → 1137.

### H2c Observations (not fixed)

- The return-to-home moves after a shuffled call (`LD C,L`, `LD B,D`) are
  not marked `; spill`, so the report's T-states undercount shuffles (a
  pre-H2a quirk of block-end home restores).
- `ty-return-local-addr` still warns on `return &v` when `v` is pinned,
  next to the real error.
- F-029: C80 cannot call ASM exports; the `@{...}` marker in inline `asm`
  is reported as `cg-internal` at the wrong file.

## H2c Validation

- `cargo test -p rtvc-c80`: lib 173, bin 1, `tests/examples.rs` 4,
  `tests/rtvc_c80.rs` 10, doc 0; 0 failed. 7.5 s wall with nothing to
  rebuild (11.3 s for the first run after a build).
- `cargo test --bin rtvc-tocas --no-default-features --features cli-tools`:
  8 passed.
- `sh info/c80/compile.sh` succeeds (`pong.cas` 1137 payload bytes).
- `cargo fmt -p rtvc-c80 -- --check` and `git diff --check` are clean.

## H3 — What Was Done

**Two folded-in fixes first.**
- Block-end restores after a spill shuffle are marked `; spill`. Locals moved
  by spill code (`bind_key` under a note, `restore_saved`) are remembered;
  `restore_homes` emits their way back home under `spilling`. Only pong
  `physics` changed: `LD C,L`, `LD B,D`, `LD C,E` are marked, and its report
  went from 24 to 48 T-states.
- An `@{unit::name}` marker in an inline `asm` body is a located
  `cg-unsupported` error at the marker ("markers such as '@{m::f}' are not
  substituted in inline asm; ..."), found before the body is parsed and
  skipping `;` comments. No substitution. A codegen test checks the span;
  [inline_asm.fail.c80](../../info/c80/inline_asm.fail.c80) has a case.

**Typing.** `check_binary` handles `<<`/`>>` before the same-type check: the
left operand must be an integer (else "`<<` requires integer operands"), the
count must be `u8`, the result has the left type. An untyped literal count
is checked as `u8` (`ty-literal-range` if it does not fit); a typed count
keeps its type, so a `u16`/`i8` count reports "shift count must be u8, found
T" at the count. Compound `<<=`/`>>=` use the same rule. The old, now
unreachable count check was removed.

**Instructions.** New `Z80Op`s: `Sla`, `Srl`, `Sra`, `Rl8`, `Rr8` (8 T, 2
bytes, write their register), `Rlca`, `Rrca`, `Rra` (4 T, write `A`),
`Inc8`, `Dec8` (4 T), `Djnz` (writes `B`, 13/8 T). Listing T-states come
from the disassembler as for every op; a test assembles each rendered form
with the helper assembler, checks the length against the new
`Z80Op::size_bytes`, and checks the disassembler's T-states (`DJNZ` 13/8).
`jump_target` in `source_map.rs` now knows `DJNZ`, so a span with the shift
loop reports `has_loop` and is incomplete.

**Constant counts** ([shift.rs](../../c80/src/shift.rs)). A few templates
per kind, the cheapest by (T-states, bytes). Templates that use `A` as
scratch are offered only when `A` holds nothing live after the op. Counts
at or above the width fold to the constant 0 for `<<` and unsigned `>>`;
signed `>>` clamps the count to width-1 (sign fill). `k == 0` is the operand
itself. Both constant operands fold. Unit tests run every sequence on a
small register model for all inputs (bytes) and 14 word patterns, every
count, in `A`/`B`/`E` and `HL`/`DE`/`BC`, with and without `A`, against
F001; without `A` no sequence writes `A`.

Byte in `A` (T, bytes):

| k | `<<` | `>>` unsigned | `>>` signed |
| --- | --- | --- | --- |
| 1 | `ADD A,A` 4/1 | `SRL A` 8/2 | `SRA A` 8/2 |
| 2 | `ADD A,A` ×2 8/2 | `RRCA` ×2, `AND 63` 15/4 | `SRA A` ×2 16/4 |
| 3 | ×3 12/3 | `RRCA` ×3, `AND 31` 19/5 | ×3 24/6 |
| 4 | ×4 16/4 | `RRCA` ×4, `AND 15` 23/6 | ×4 32/8 |
| 5 | `RRCA` ×3, `AND 224` 19/5 | `RLCA` ×3, `AND 7` 19/5 | `RLCA` ×3, `AND 7`, `XOR 4`, `SUB 4` 33/9 |
| 6 | `RRCA` ×2, `AND 192` 15/4 | `RLCA` ×2, `AND 3` 15/4 | `RLCA` ×2, `AND 3`, `XOR 2`, `SUB 2` 29/8 |
| 7 | `RRCA`, `AND 128` 11/3 | `RLCA`, `AND 1` 11/3 | `ADD A,A`, `SBC A,A` 8/2 |
| ≥8 | constant 0 | constant 0 | as 7 |

A byte in another register whose result goes back there (or that is dead
and `A` is busy) shifts in place, `SLA`/`SRL`/`SRA r` ×k (8k T, 2k bytes),
or through `A` (`LD A,r` + the table + `LD r,A`) when that is cheaper and `A`
is free.

Word in `HL`, `A` free (T, bytes):

| k | `<<` | `>>` unsigned | `>>` signed |
| --- | --- | --- | --- |
| 1–5 | `ADD HL,HL` ×k, 11k/k | `SRL H / RR L` ×k, 16k/4k (k ≤ 4) | `SRA H / RR L` ×k (k ≤ 4) |
| 5 | | `XOR A`, (`ADD HL,HL / RLA`) ×3, `LD L,H / LD H,A` 57/9 | `LD A,H / RLA / SBC A,A`, (`ADD HL,HL / RLA`) ×3, `LD L,H / LD H,A` 65/11 |
| 6 | `XOR A`, (`SRL H / RR L / RRA`) ×2, `LD H,L / LD L,A` 52/13 | as 5 with ×2: 42/7 | as 5 with ×2: 50/9 |
| 7 | as 6 with ×1: 32/8 | as 5 with ×1: 27/5 | as 5 with ×1: 35/7 |
| 8 | `LD H,L / LD L,0` 11/3 | `LD L,H / LD H,0` 11/3 | `LD A,H / LD L,A / RLA / SBC A,A / LD H,A` 20/5 |
| 9 | `LD H,L / LD L,0 / SLA H` 19/5 | `LD L,H / LD H,0 / SRL L` 19/5 | `LD A,H`, `SRA A`, `LD L,A / RLA / SBC A,A / LD H,A` 28/7 |
| 10 | `LD A,L`, byte `<< 2`, `LD H,A / LD L,0` 23/6 | `LD L,H / LD H,0 / SRL L` ×2 27/7 | as 9 with byte `>> 2`: 36/9 |
| 11 | via `A`: 27/7 | `LD A,H`, byte `>> 3`, `LD L,A / LD H,0` 34/9 | 44/11 |
| 12 | 31/8 | 38/10 | 52/13 |
| 13 | 34/9 | 34/9 | 53/14 |
| 14 | 30/8 | 30/8 | 49/13 |
| 15 | `LD A,L / RRCA / AND 128 / LD H,A / LD L,0` 26/7 | `LD A,H / RLCA / AND 1 / LD L,A / LD H,0` 26/7 | `LD A,H / RLA / SBC A,A / LD H,A / LD L,A` 20/5 |
| ≥16 | constant 0 | constant 0 | as 15 |

With `A` busy the word table falls back to steps (`<< 6` 66/6, `<< 7`
77/7, `>> 5..7` 80–112 T), byte moves plus in-place `SLA H`/`SRL L`
(`<< 10` 27/7 ... `<< 15` 67/17), and for signed counts 8–14 `LD L,H` plus
`SRA L` ×(k-8) and `SRA H` ×7 (60–108 T); signed 15 is `ADD HL,HL / SBC
HL,HL` (26/3). The full output is the `print_constant_shift_table` test in
`shift.rs` (`cargo test -p rtvc-c80 --lib print_constant_shift_table --
--nocapture`). A word in `DE`/`BC` whose result returns there uses the same
templates with `SLA lo / RL hi` (16 T) as the left step.

**Variable counts.** The count is copied into a counter register: `B` when
nothing live is there (and it is not the operand's register), else the
count's own register when it dies there, else any free register, else `B`
or `C`/`D`/`E` with the normal eviction (a spill, or the strict error in
`@fastcall`). Then:

    INC c / DEC c / JR Z,done     ; 8 T + JR; a zero count runs no step
  loop:
    <one step>
    DJNZ loop                     ; c = B: 13 T taken, 8 T at the end
    (DEC c / JR NZ,loop)          ; another counter: 16 T taken, 11 at the end
  done:

The value is shifted in `A`/`HL`, or in place in its own register for a
store-back (`x >>= n` with `x @ b` is `SRL B` in the loop) and for `<<` when
the accumulator is busy. `B`-counter steps: byte `<<` 17 T per count in `A`,
`>>` 21 T; word `<<` 24 T, `>>` 29 T.

**Not clamped.** For all six kinds the value stops changing after `width`
steps: `<<` and unsigned `>>` have shifted every bit out and shift 0 to 0;
signed `>>` has reached all sign bits (0 or 0xFF/0xFFFF), and an arithmetic
shift of 0 is 0 and of all ones is all ones. So running `min(n, width)`
steps gives the same result as `n` steps, and clamping would be correct.
It is not done: the clamp needs a compare against the width on every shift
(`LD A,c / CP 9 / JR C / LD c,8` is about 23 T with `A` free, and the byte
case has its value in `A`), which every in-range count pays to bound a rare
worst case (255 steps: 4.3k–5.4k T for a byte, 6.1k–7.4k T for a word). The
user's priority is speed for the usual count; a later opt-in can add it.

**Placement.** Shifts work wherever the operand lives: register locals,
pins (`x <<= 1` with `x @ b` is `SLA B`; `w <<= 1` with `w @ de` is `SLA E
/ RL D`), globals, `static` locals, fields, and through pointers (load,
shift, store). The lowerer peeks at the next IR op: when it stores the
result into the local whose register holds the operand, the in-place form
competes with the `A`/`HL` form plus the move back (counted only when the
home is fixed: stable homes or a pin).

**Address scaling.** `scale_u16` emits one `<< k` per run of zero bits
between the set bits of the element size, then adds. A scale of 2 is one
`ADD HL,HL` instead of an addition through a second pair, 5 is
`(i << 2) + i`, 64 is the 52 T sequence above, 256 is `LD H,L / LD L,0`. The
memory accesses are unchanged. Effects: `memory.c80` `pair_sum` lost its
2-byte slot and spill code; `structs.c80` `nudge` is 71 → 47 bytes (3 → 1
slot bytes, 148 → 76 T of spill code); `strict.u16_array_index` now fits in
registers and left `STRICT_REJECTIONS`.

## H3 Ledger

- `KNOWN_GAPS`: 107 entries before (104 operator-matrix shift entries: 72
  `ShiftType`, 32 `ShiftCodegen`; `prog.bubble_sort` and
  `prog.shift_add_multiply` `ShiftCodegen`; `prog.linked_struct_static`
  `StaticAddr`) → 1 after: `prog.linked_struct_static` (H3a). Regenerated
  from a `C80_CONFORMANCE_DUMP=1` run, which printed only that line.
- `RANDOM_GAPS`: the 8 `rshift.{u8,i8,u16,i16}@{opt,base}` rows (710
  rejections across modes) → empty; every random program passes.
- `Cause::ShiftCodegen` and `Cause::ShiftType` are deleted; a comment says a
  `Rejected` gap needs the user's sign-off.
- `STRICT_REJECTIONS` lost `u16_array_index` (it passes strict now).
- No `W` in any mode.

## H3 pong `put_row`

`a = u16(y) << 6` replaces the six `a += a`. The six additions compiled to
`LD D,H / LD E,L / ADD HL,DE` each (114 T, 18 bytes); `<< 6` is
`XOR A / (SRL H / RR L / RRA) ×2 / LD H,L / LD L,A` (52 T, 13 bytes). Even
six `ADD HL,HL` written by hand would be 66 T. `put_row` is 228 → 166 T and
43 → 38 bytes (`expect-tstates`/`expect-bytes` lines in the file); the
`pong.cas` payload is 1137 → 1132 bytes. `<< 6` is faster than the hand
version, so no sequence fix was needed.

Not done: the emulated-TVC check of the new cassette. In headless mode
`rtvc --headless -i pong.cas` left no BASIC program in memory (`RUN` printed
`ok`, `19EFH` was zero) and `-t` plus `LOAD` stopped with "System error
161", so the check needs the GUI as before. The harness checks of `put_row`
(`expect-mem` rows 3 and 239) pass.

### H3 Golden Changes

| Golden | Change | Why |
| --- | --- | --- |
| pong | `put_row` −5 bytes, −62 T; BASIC `USR` address 13279 → 13274; label numbers | `<< 6` |
| structs | `nudge` 71 → 47 bytes, one spill slot instead of three | `5 * i` is `(i << 2) + i` with `ADD HL,HL` |
| memory | `pair_sum`: no slot, no spill code | `i * 2` is one `ADD HL,HL` |
| shifts | new | |
| (fix 1) pong | `physics`: three `; spill` marks | restores after a shuffle |

### H3 Observations (not fixed)

- `u16(y) << 6` on a zero-extended byte could use that `H` is 0
  (`RRCA / RRCA` and two masks, about 38 T instead of `LD H,0` + 52 T). A
  cast-and-shift peephole belongs in H5.
- `x + x` on a word is still `LD D,H / LD E,L / ADD HL,DE`, not
  `ADD HL,HL` (H5).
- `p + 0x8000` is still `LD A,L / ADD A,0 / LD L,A / LD A,H / ADC A,128 /
  LD H,A` (H5).
- In `rotate_cell` (shifts.c80) the field store after the loop shuffles
  three pairs; correct, not minimal.

## H3 Validation

- `cargo test -p rtvc-c80`: lib 183, bin 1, `tests/examples.rs` 4,
  `tests/rtvc_c80.rs` 10, doc 0; 0 failed. 8.0 s wall with nothing to
  rebuild (24.8 s right after `compile.sh`, which rebuilds the root crate).
- `cargo test --bin rtvc-tocas --no-default-features --features cli-tools`:
  8 passed.
- `sh info/c80/compile.sh` succeeds (`pong.cas` 1132 payload bytes).
- `cargo fmt -p rtvc-c80 -- --check` and `git diff --check` are clean.

## H3a — What Was Done

**Flaky CLI tests first.** `scratch()` in
[tests/rtvc_c80.rs](../../c80/tests/rtvc_c80.rs) named its directory
`rtvc-c80-cli-<pid>-<SystemTime nanos>`. The tests run on parallel threads
of one process, and this Mac's wall clock has microsecond resolution:
200,000 `time.time_ns()` samples were all multiples of 1000, and 185,539 of
them repeated the previous value. Two tests starting in the same
microsecond got the same directory; both write `main.c80` /
`rtvc-c80.toml` there and `remove_dir_all` it at the end, so one deleted or
overwrote the other's files. That matches the symptom (a different pair
each time, first run after a rebuild when the tests start closest
together). The panic texts from the failing runs were not kept, so the
cause is confirmed from the code and the clock, not from a message. Fix: a
process-wide `AtomicU32` counter in the name. 10/10 on three runs in a
row and in every run since.

**Addresses in static initializers (F-023).**
- [ir.rs](../../c80/src/ir.rs): `TypedGlobal.relocs: Vec<StaticReloc>`
  (`offset`, `target: GlobalId`, `addend`); the data bytes there are zero.
- [semantics.rs](../../c80/src/semantics.rs): `eval_static_value` returns
  `Bits` or `Addr { target, offset }`. An initializer without `&` goes
  through `eval_const_expr` as before. With `&` it is first typed by
  `check_expr` (so `&array`, `&const`, a string payload and a pinned local
  get the run-time diagnostics, and `u16 x = &g` is the usual mismatch),
  then `static_addr` / `static_place` reduce it to a global and a byte
  offset: names (globals, statics in scope via `static_key`),
  `unit::name`, `[K]` with `K` from `eval_const_expr` and checked against
  the length, `.field`, and `ptr<U>(...)` / `u16(...)`. Anything else is a
  located error. Scalar globals, brace elements (`write_static_init`) and
  static locals use it; a static local's initializer sees the enclosing
  scopes plus itself (`Analyzer.static_scopes`). A `const` holding an
  address is rejected.
- [lower.rs](../../c80/src/lower.rs): `lower_global` emits `DB` runs and
  `DW label` / `DW label+n` lines (one `Data` item per line); globals
  without relocations emit exactly the old text.
- Map: `GeneratedGlobal.refs: Vec<DataRef>` and
  `CompilerMap.data_refs: Vec<MappedDataRef>` (address, containing global,
  target C80 name, label, addend) with `data_ref_at(addr)`.
- Assembler: no change. A new c80 test assembles
  `DW LATER+3, OTHER+2, OTHER` with `LATER` defined later and `OTHER` in a
  second `ORG` block. While checking the cross-unit case, found F-030:
  the per-unit JR pass silently skips a unit that names another unit's
  label (pre-existing; cross-unit calls already hit it).
- Ledger: `prog.linked_struct_static` passes; `KNOWN_GAPS` and
  `RANDOM_GAPS` are empty, `Cause::StaticAddr` is gone (the machinery,
  `Pressure`, and the no-wrong-values test stay).
- Tests: `static_initializer_addresses_match_the_symbol_table`,
  `static_linked_data_walks_at_run_time` (both modes; forward, self,
  struct array, static locals), `static_initializer_address_rejections_are_located`
  (12 forms, one error each, span checked),
  `static_address_of_another_units_global` (project; plus the private-name
  error), `helper_assembler_resolves_forward_and_cross_org_word_references`.
- Examples: new [tables.c80](../../info/c80/tables.c80) (linked list with a
  forward reference, sprite table, `u16` address table, `&cfg.score`,
  self-linked ring, static-local pointer; `expect` lines walk the data,
  comments quote the `DW` lines) and
  [tables.fail.c80](../../info/c80/tables.fail.c80) (10 rejected forms);
  [memory.c80](../../info/c80/memory.c80) gained `mid = &buf[4]` and a
  2-entry `u16` address table at the end. Tutorial section 9 (later
  sections renumbered 10–18, anchors fixed);
  [c80.md](../../info/c80.md) "Addresses in static initializers (H3a)".

### H3a Diagnostics

| Code | Example | Message |
| --- | --- | --- |
| ty-mismatch | `ptr<u8> p = &grid;` | `cannot take the address of array 'grid'; use &grid[0]` |
| ty-not-lvalue | `ptr<u8> p = &SIZE;` | `cannot take the address of a constant` |
| ty-not-lvalue | `ptr<u8> p = &title[1];` | `cannot take the address of a string payload` |
| ty-mismatch | `ptr<u8> p = &grid[1] + 1;` | `a static initializer holds an address only as &global, &array[K] or &global.field (K constant), optionally in ptr<T>(...) or u16(...); arithmetic on an address is not allowed` |
| ty-literal-range | `ptr<u8> p = &grid[4];` | `constant index is outside the array or string` |
| ty-mismatch | `ptr<u8> p = &cursor[1];` | `a static address can index only an array: a pointer's value is not known until run time` |
| ty-mismatch | `ptr<u8> p = &grid[i];` | `a static address needs a constant index` |
| ty-mismatch | `u16 p = &grid[0];` | `expected u16, found ptr<u8>` |
| ty-mismatch | `const ptr<u8> P = &grid[0];` | `a const cannot hold an address: it is fixed only when the program is linked; declare a global instead` |
| ty-not-lvalue | `static ptr<u8> q = &n;` (automatic `n`) | `a static initializer cannot take the address of automatic local 'n': it has no fixed address; declare it 'static'` |
| ty-not-lvalue | `static ptr<u8> q = &r;` (`u8 r @ b`) | `cannot take the address of 'r': it is pinned to register b; declare it 'static' to give it an address` |

### H3a Golden Changes

| Golden | Change | Why |
| --- | --- | --- |
| memory | +23 lines at the end only (`G100_MID: DW G2_BUF+4`, `G113_ENDS`, `mid_ptr`, `last_word`) | source edited |
| tables | new | |

Before regenerating, the golden test reported only `memory.asm` (differs)
and `tables.asm` (missing); no other listing changed.

## H3a Validation

- `cargo test -p rtvc-c80`: lib 188, bin 1, `tests/examples.rs` 4,
  `tests/rtvc_c80.rs` 10, doc 0; 0 failed. 8 s wall with nothing to
  rebuild (21 s right after `compile.sh`, which rebuilds the root crate
  with other features; one run measured 1040 s wall with the same test
  times, so it spent that time waiting outside the tests, most likely on
  the cargo build lock).
- `cargo test --bin rtvc-tocas --no-default-features --features cli-tools`:
  8 passed.
- `sh info/c80/compile.sh` succeeds, `tables` included (`pong.cas` 1132
  payload bytes, unchanged).
- `cargo fmt -p rtvc-c80 -- --check` and `git diff --check` are clean.
- Root-crate assembler untouched, so its `asm::` tests were not needed.

## H3i — What Was Done

**F-030 first** (c80 only). `optimize_chunks` takes `externals`: the labels
the other units define. `assemble_current` appends `LABEL EQU 0` for each
(no listing line, so line/address pairing is unchanged) and
`shorten_jumps` never shortens a jump to one of them. Single-file builds
pass none. Test `unit_calling_another_unit_still_gets_jr`. No golden
changed.

**Surface (F-031).** `@inline` and `@noinline` beside `@fastcall` /
`@preserves`, before or after `pub`. Errors: `@inline @stackcall`,
`@inline @noinline`, duplicates, either on a global. Decisions on
`@preserves`, strictness, pins, statics: see F-031 in
[c80-compiler-findings.md](c80-compiler-findings.md).

**Mechanism.** `compile_project` already builds one `TypedProgram` for all
units, so cross-unit `pub @inline` works with no extra plumbing.
[inline.rs](../../c80/src/inline.rs) runs before lowering (`lib.rs`
`inline_program`, called by `compile` and `compile_project`), callees
first:
- arguments are already evaluated once, left to right, by the call op;
  parameters become fresh caller locals, or read the argument directly when
  the body never writes or addresses them (an immediate for a constant);
- a single-block callee is spliced into the calling block; otherwise the
  block is split, callee blocks are cloned with fresh ids, each `return`
  stores `{callee}.result` and jumps to the continuation, and caller vregs
  that cross the split are carried through fresh locals (vregs stay
  block-local; no lowering invariant changed);
- constant folding for unary/cast/binary ops and `Branch` on a constant;
  unreachable cloned blocks are dropped; trivial chains are merged and a
  fresh local stored once and loaded in the same block is forwarded;
- pin scopes are remapped; a pin that overlaps a caller pin or the caller's
  `@preserves` is a located error at the call naming both;
- inline `asm`: fresh node and `IA<node>_` label prefix per copy;
- provenance: `IrBlock.origin` indexes `TypedFunction.inline_sites`
  (call frames, outermost first). Lowering emits `; inline NAME [(auto)]`,
  sets `MappedInstruction.inlined`, fills `GeneratedFunction.inlined`
  (`InlineReport`: callee, call span, auto, bytes; zero-byte expansions
  included), and appends "(in 'g', inlined into 'f')" plus related call
  spans to diagnostics raised inside an expansion.
- Out-of-line bodies: `TypedFunction.out_of_line` is false for a private
  `@inline` function and, after automatic inlining, for a private function
  that had calls and has none left. Such a function is still lowered (its
  own contract and strictness are checked) but not emitted.
  `compile_keeping_bodies` / `project::compile_project_keeping_bodies`
  (`#[doc(hidden)]`) emit them anyway; the examples test uses that build
  only for directives whose function the real build lacks.

**Automatic inlining** (`auto_inline`, optimized builds only). Candidates:
private, not `@inline`/`@noinline`/`@stackcall`, no inline `asm` with
`CALL`/`RST`. Each site is expanded on a copy and both programs are
measured by `lower::measure_functions` (each function lowered, optimized
and assembled alone; other functions' labels are externals). Accepted when
the caller's spill (slot bytes + save bytes) does not grow and either its
size does not grow, or the callee has exactly one C80 call and the caller
grows by at most the callee's body (which then disappears). Fixed order:
callers callees-first, sites in op order. A rejected site is not retried.

**INC/DEC.** Needed for the byte-exact `inc` fixture: `x = x ± 1` on a byte
whose result goes straight back into the same register is `INC r` /
`DEC r` (lowering, both modes). This changes existing unannotated code.

**Tests.** New [inline_tests.rs](../../c80/src/inline_tests.rs) (18):
surface errors and combinations; `inc` in a loop is one 1-byte `INC`
(both modes, `GeneratedFunction.inlined` checked); `cpu::in` argument read
once and two arguments read left to right; several returns; a loop in the
callee; three-level nesting (frames `a3 > a2 > a1`, span in `a1`, map
covers every byte); inline `asm` label inlined twice (two distinct labels);
`pub @inline` emitted and callable; constant arguments fold (no
`CP`/`JR`/`JP`; `add(a, 5)` is `ADD A,5 / RET`); shared `static` (one
global, used by every expansion and the out-of-line copy, `f` returns 3);
strict caller message and related span; pin conflicts (caller pin, caller
`@preserves`) and a pin that fits; contracts of functions containing
expansions (declared contract checked at compile time and by the harness
guard, inferred contract includes the expansion's writes, a caller relying
on it); stack bound drops the `CALL` (0 vs 2); `--no-optimize` expands;
automatic rules (A, B, never `pub`/`@noinline`/`@stackcall`/asm `CALL`,
never in `--no-optimize`, bodies-kept build identical for `f`); no new
spill; determinism. Conformance: `conformance_program_fixtures_inlined`
re-runs the 17 program fixtures that have helpers with every helper marked
`@inline` (both modes, same results, no `CALL` to a helper). Project:
`pub_inline_expands_across_units` (expansion in another unit, exported
copy called through an ASM unit's `@{lib::inc}` marker). Codegen tests
that are about calls now mark their helpers `@noinline` (5 tests).

**Examples.** New [inline.c80](../../info/c80/inline.c80) (`col_call` vs
`col`: 114 T / 12 bytes against 45 T / 10 bytes, both pinned with
`expect-tstates` / `expect-bytes`; constant fold; `inc` in a loop),
[inline.fail.c80](../../info/c80/inline.fail.c80) (3 attribute errors),
[inline_cg.fail.c80](../../info/c80/inline_cg.fail.c80) (2 pin conflicts).
`expect-tstates` now accepts an unconditional `CALL` of a generated
function whose own listing is straight-line (tutorial grammar updated).
`@noinline` added where the prose is about the call: `calls.c80` (`id`,
`add8`, `add16`, `tagged`), `contracts.c80` (`bump`), `registers.c80` and
`spills.c80` (`twice`), `memory.c80` (`lit_len`). The `nest` comment in
`calls.c80` was stale (it described a `PUSH BC` that H2c had removed) and
now matches the listing. Budgets and listings in comments updated for
`DEC B`: `contracts.c80` `count` 22→19, `registers.c80` `times` 21→18,
`scale` 20→17.

**Pong.** `put_row` and `read_row` are `@inline`; header comment rewritten.
The functions `play` calls once (`setup`, `wait_frame`, `serve`,
`move_ai`, `physics`, `restore`) are inlined automatically and lose their
bodies; their directives run against the bodies-kept build.

| Measure (optimized) | Before H3i | After |
| --- | --- | --- |
| `fill_rows`, `draw_h` = 2 | 544 T | 438 T |
| `fill_rows`, `draw_h` = 24 | 5934 T | 4662 T |
| `fill_rows` bytes | 26 (+38 `put_row`) | 53 |
| `move_player`, idle / up / down | 130 / 1462 / 1427 T | 103 / 1223 / 1188 T |
| `move_player` bytes | 125 (+5 `read_row`) | 126 |
| payload (`pong.cas`) | 1132 bytes | 1087 bytes |

(`move_player` with `py = 10`, port `58H` scripted `FF`/`FD`/`FB`.)
`play` cannot run in the harness (it never returns); it grew from 118 to
611 bytes by absorbing 524 bytes of bodies, and each absorbed call lost
its `CALL`/`RET`. Emulator: `target/debug/rtvc --headless --port 8765
snapshots/boot12dos.rtvcsnap.zip -i info/c80/out/pong.cas`, screenshot
after 12 s: green border, black playfield, both paddles at mid-height, the
ball in the centre.

### H3i Golden Changes

Before (`C80_TMP_*` switches, removed again) and after, every directive
run of every example in both modes: 66 runs faster, 335 unchanged, 0
slower. Function sizes, optimized:

| Golden | Change | Why | Size | T-states |
| --- | --- | --- | --- | --- |
| contracts | `LD A,B / SUB 1 / LD B,A` → `DEC B` | INC/DEC | `count` 22→19 | `count(5)` 473→418 |
| control | `INC B`, `DEC C` | INC/DEC | `count` 21→15, `skipped` 37→31 | `count(5)` 338→228, `skipped(5)` 482→394 |
| memory | `DEC C` | INC/DEC | `fill` 18→15 | `fill` 221→188 |
| mixed | `DEC B`, `DEC C`; BASIC `USR` addresses move 3 bytes | INC/DEC | `bump` 30→27, `fill` 26→23 | `fill` 1030→854 |
| optimize, optimize.unopt | `INC B`, `DEC C` | INC/DEC (both modes) | `count` 21→15 | `count(3)` 222→156 |
| registers | `DEC B` twice | INC/DEC | `times` 21→18, `scale` 20→17 | `times(7,6)` 382→316 |
| structs | `DEC C`, `INC B/E/C` | INC/DEC | `step` 26→23, `walk` 22→16, `once` 14→11 | `walk(3)` 217→151 |
| pong | `@inline` `put_row`/`read_row`; six functions auto-inlined into `play`; `; preserves` lines of the absorbed helpers gone | H3i | see the table above | see above |
| inline | new | | | |

calls, spills, hello, scalars, shifts, stackcall, ports, inline_asm,
tables, project and tvc-usr did not change. Without `@noinline`, calls,
contracts, memory, registers and spills would also have changed (their
helpers were auto-inlined); those were faster too (e.g. `nest` 91→17 T,
`after_private` 48→21 T), but the prose is about the call.

## H3i Validation

- `cargo test -p rtvc-c80`: lib 209, bin 1, `tests/examples.rs` 4,
  `tests/rtvc_c80.rs` 10, doc 0; 0 failed (lib 7.1 s).
- `cargo test --bin rtvc-tocas --no-default-features --features cli-tools`:
  8 passed.
- `sh info/c80/compile.sh` succeeds (`pong.cas` 1087 payload bytes).
- `cargo fmt -p rtvc-c80 -- --check` and `git diff --check` are clean.
- Root crate untouched.

## H4 — What Was Done

- **Where:** [c80/tests/tvc.rs](../../c80/tests/tvc.rs), with boot, cassette
  and typing helpers in [c80/tests/support/mod.rs](../../c80/tests/support/mod.rs)
  (shared with [rtvc_c80.rs](../../c80/tests/rtvc_c80.rs), whose Pong start
  test moved into `tvc.rs`). The plan put the test next to
  [rtvc_tocas.rs](../../src/bin/rtvc_tocas.rs), but the root crate cannot
  depend on `rtvc-c80` (it would be a cycle and would pull the compiler into
  emulator builds). An integration test of `rtvc-c80` already sees both: it
  compiles in process, flattens with `rtvc_core::cas`, and drives
  `rtvc_core::tvc::Tvc`. Every API used is public; the root crate is
  unchanged.
- **Start state:** a cold boot of the TVC 1.2 ROMs (`TVC12_D4/D3/D7`, fast
  boot), as the existing Pong test did. The `boot12dos` snapshot needs the
  native-only zip reader and adds VT-DOS state the test does not need.
- **[selfcheck/](../../info/c80/selfcheck/):** `main.bas` calls
  `USR(@{selfcheck::run},@{selfcheck::PASSES})`, prints the result, the pass
  count and the six `failed` bytes, then `POKE`s `basic_done`. 44 checks,
  indexes listed in [selfcheck.c80](../../info/c80/selfcheck/selfcheck.c80)
  and in `CHECK_NAMES` in the test:
  0-3 u8 operators, 4-7 i8, 8-11 u16, 12-15 i16 (wrap/overflow points),
  16-23 compares at signed/unsigned boundaries, 24-27 constant shifts,
  28-31 variable shifts incl. counts >= width, 32 static-slot spill,
  33 PUSH/POP save around a call, 34 strict `@fastcall` + pin,
  35 `@preserves(bc)` callee keeps a pinned counter, 36 `static` local,
  37 `@inline`, 38 `@stackcall`, 39 nested calls, 40 struct array fields,
  41 static linked list, 42 `str`, 43 `cpu::ldir`. Expected values are
  hand-derived literals; inputs are globals so nothing folds. The golden
  listing confirms the spill (`nine`: 3 static slot bytes) and the save
  (`keep`: `PUSH BC`/`POP BC` around `CALL`).
- **Self-check test** (both modes): types `RUN`, then steps instruction by
  instruction until BASIC `POKE`s `basic_done` (budget 600 frames, failure
  names PC and progress). Asserts no `failed` bit (names each failing
  check), every `seen` bit, `passes == 40`, `checks == 1760`, at least 5
  interrupts accepted with PC in generated code, and SP after `run`
  returns == entry SP + 2. An accepted interrupt is detected as IFF1 1->0
  on an instruction that is not `DI`.
- **Stack:** entry SP is SP at `run`'s first instruction (return address
  already pushed, the same definition as `stack_bound`); the low-water mark
  is sampled only while PC is in generated code, so the ISR's own pushes
  (PC 0038H onward, in ROM) are excluded. The interrupt's pushed return
  address lands below the C80 frame and is ROM's cost, covered by BASIC's
  stack, not by `stack_bound`. The test asserts observed <= proven.
- **Pong test** (both modes): loop passes are counted at the first
  instruction of `wait_frame` (its own function in `--no-optimize`, found
  through the inline frames of `play` otherwise). Checks the start layout
  (paddles at raster 108, ball at 32/118), that the ball waits for Space,
  that Space serves and the ball moves one column and one raster per pass,
  that joystick down for 10 passes moves the paddle 20 rasters and up for 6
  passes moves it 12, then 300 passes of an autopilot that follows the ball.
  Every few passes and at the end, all 0x3C00 bytes of VID0 must be 0xFF at
  exactly the two paddles and the ball (from `py`, `ay`, `bx`, `by`) and 0
  elsewhere; PC must stay inside generated code (play runs with
  interrupts off), and the game must not end.

## H4 Results

| | interrupts (in C80) | frames | stack bound | observed | run time |
| --- | --- | --- | --- | --- | --- |
| selfcheck, optimized | 24 (17) | 24 | 14 (proven) | 14 | 2.8 s |
| selfcheck, `--no-optimize` | 25 (17) | 25 | 16 (proven) | 16 | 1.9 s |

Entry SP 1698H in both modes (BASIC's stack), low 168AH / 1688H. The proven
bound is exact. Pong: 300 autopilot passes per mode, no point scored (the
autopilot returns every ball, so paddle hits ran), 6.8 s optimized and
5.3 s baseline. `cargo test -p rtvc-c80 --test tvc` takes about 7 s in
debug (the four tests run in parallel). Nothing failed on the machine that
passed in the harness.

F-032 (new): a value computed before `&&`/`||` in the same expression is
rejected with `internal: value has no location`. The self-check routes
such conditions through a `bool t` local.

## H4 Validation

- `cargo test -p rtvc-c80`: lib 209, bin 1, `tests/examples.rs` 4,
  `tests/rtvc_c80.rs` 9, `tests/tvc.rs` 4; 0 failed.
- `cargo test --bin rtvc-tocas --no-default-features --features cli-tools`:
  8 passed.
- `sh info/c80/compile.sh` succeeds (`selfcheck.cas` 2320 payload bytes).
- `cargo fmt -p rtvc-c80 -- --check` and `git diff --check` are clean.
- Root crate untouched.

## H5 — What Was Done

Each item landed alone, with lib, examples and tvc green in both modes and
the quality table re-recorded:

- **F-032** fixed (`inline::carry_block_values`), plus F-033 (slot 0 into a
  high half) and F-034 (parser loop) found on the way. Fixtures
  `prog.f032_*`, random family `rlogic`; the self-check writes its `&&`
  conditions inline again.
- **Step 0:** [quality_tests.rs](../../c80/src/quality_tests.rs) — compiler
  vs hand-written reference per fixture, same harness, same results and
  ordered effects; `BUDGET` rows may only go down.
- **Item 1** (direct condition lowering, both modes): `check_branch` in
  semantics.
- **Item 2** (flag-aware peepholes, optimized only):
  [liveness.rs](../../c80/src/liveness.rs) + `peephole` in optimize.rs.
- **Item 3** (redundant moves and reloads, optimized only):
  `track_registers` in optimize.rs — per-register value numbers for
  constants, symbol addresses and compiler-private slots (`F.._S`), never
  globals, statics, pointers or ports. Reset at labels something can reach
  (any jump, another chunk, or any `asm`/data text naming it), at
  `RET`/`JP`, inline `asm` and data; slot knowledge dropped at stores
  through pointers, `CALL`, `LDIR`. `RET` liveness is the returned and
  preserved registers only, so dead entry moves go.
- **Item 4** (pointers through BC/DE, lowering, both modes): new ops
  `LD A,(BC|DE)`, `LD (BC|DE),A`, `INC BC|DE`. A byte load/store uses the
  pair the pointer already occupies; `p = p + 1|2` stored straight back
  into a BC/DE home is `INC rr` in place. A fourth home plan,
  `PackedPairs` (no pointer gets HL first), is tried for functions with a
  pointer local; the cheaper estimate wins, ties keep `Packed`. `sum`
  1686 -> 1518 T, 37 -> 35 bytes; bubble sort 8828 -> 8756 T. The old
  `sum_pinned` rejection (`s @ hl`) now compiles, so it left
  registers_cg.fail.c80.
- **Item 5** (control flow, optimized only): `thread_jumps` (a `JP` to a
  label whose code is `JP y` goes to `y`), `remove_unreachable` (code after
  `JP`/`JR`/`RET` up to a label something names), `rotate_loops` (`top: T;
  JP cc,exit; body: B; JP top` -> `JR top; body: B; top: T; JP !cc,body`;
  T straight-line, no `asm`/data in B; the back `JP` becomes the entry
  jump, dropped when nothing falls in), `tail_calls` (`CALL x` directly
  followed by `RET` -> `JP x`). Stack bounds are recomputed after
  optimizing (`fill_stack_bounds`, single-unit builds): a tail call adds the
  callee's bound, not bound + 2. Project builds keep the pre-optimization
  bound (still an upper bound). Rotation costs 10 T for a loop that runs
  zero times and 3 T for one pass and saves 7 T per further pass
  (zero_one_many: 45 -> 55, 78 -> 81, 210 -> 185). count 1950 -> 1595 T,
  fill 3012 -> 2601, walk 1905 -> 1564, sum 1518 -> 1464, fill_rows h=24
  4227 -> 4069; bytes unchanged.
- **F-035** (found by the tail-call test): the assembler drops `_` in
  numbers and reads a trailing `H` as hex, so the label `F16_H` of a
  function `h` assembled as 0x0F16 and `CALL h` went there. `asm_label`
  appends `_L` to a label that would read as a number.
- **Item 6** (constant folding, optimized only): [fold.rs](../../c80/src/fold.rs)
  runs after inlining. Within a block, `Unary`/`Binary`/integer `Cast` on
  `Const` operands becomes a `Const` (wrapping at the result width, F001
  shifts, compares by operand signedness; mixed operand types are left
  alone), and a `Branch` on a constant becomes a `Jump` unless it leads
  into a `&&`/`||` value join. Unreachable IR blocks are dropped
  (optimized only). `255 + 1` is `XOR A`, the scalars/memory `sizeof`
  sums are `LD HL,4`; the selfcheck's inlined `col(40) == 10` folds. No
  quality fixture contains a literal-only expression, so the table is
  unchanged.
- **F-036** (found by the random `rlogic` family once folding produced a
  `true || x` shape; reproducible in both modes with a literal `true`
  before H5): breaking a register cycle in `parallel_moves_inner` picked a
  scratch byte without checking for a live value there (a carried local at
  its home in D), and the H1 clobber guard rejected the function. The
  scratch now skips registers with live occupants outside the move set.
- **Item 7** (`DJNZ`, optimized only) — decision: used, in two shapes, both
  on B and both gated on A and all flags dead after the loop test
  (`use_djnz` in optimize.rs). Do-while: `DEC B / LD A,B / OR A / JP
  NZ,body` -> `DJNZ body`. Rotated while: the source's `n != 0` guard moves
  to the entry (`LD A,B / OR A / JP Z,exit`, tracking then drops `LD A,B`
  when A already holds n) and `DEC B` + test become `DJNZ`; zero-trip
  safety is that guard, run once before the first pass. Only when the test
  label has no other entrant and the body (no `asm`/data) fits a relative
  jump. A counter the allocator put in another register stays `DEC r / JP
  NZ` (count/walk/fill fixtures: counter in C). fill_rows 4069 -> 3829 T
  (h=24), 48 -> 45 bytes; move_player 2358 -> 2190 T; times 17 -> 15
  bytes, scale 16 -> 13, contracts count 18 -> 15.

## H5 Results

[quality_tests.rs](../../c80/src/quality_tests.rs), optimized build vs the
hand-written reference (bytes / T-states; same harness, inputs, results and
ordered effects; `--no-optimize` checked for the same results):

| Fixture | Before H5 | After H5 | Reference | Ratio before -> after |
| --- | --- | --- | --- | --- |
| count | 15 / 2124 | 14 / 1595 | 8 / 994 | 1.88 / 2.14 -> 1.75 / 1.60 |
| walk | 16 / 1905 | 16 / 1564 | 8 / 960 | 2.00 / 1.98 -> 2.00 / 1.63 |
| fill | 15 / 3210 | 14 / 2601 | 8 / 1689 | 1.88 / 1.90 -> 1.75 / 1.54 |
| put_row | 38 / 332 | 34 / 302 | 23 / 202 | 1.65 / 1.64 -> 1.48 / 1.50 |
| fill_rows h=2 | 53 / 438 | 45 / 353 | 34 / 259 | 1.56 / 1.69 -> 1.32 / 1.36 |
| fill_rows h=24 | 53 / 4662 | 45 / 3829 | 34 / 2723 | 1.56 / 1.71 -> 1.32 / 1.41 |
| hits | 48 / 617 | 28 / 393 | 25 / 345 | 1.92 / 1.79 -> 1.12 / 1.14 |
| step | 68 / 732 | 49 / 573 | 44 / 512 | 1.55 / 1.43 -> 1.11 / 1.12 |
| bubble sort | 170 / 9125 | 159 / 8616 | 27 / 1340 | 6.30 / 6.81 -> 5.89 / 6.43 |
| shift-add multiply | 80 / 9376 | 80 / 9178 | 21 / 2560 | 3.81 / 3.66 -> 3.81 / 3.59 |
| sum (registers.c80) | 45 / 1983 | 35 / 1464 | 30 / 1384 | 1.50 / 1.43 -> 1.17 / 1.06 |
| t[i] u8 | 11 / 114 | 11 / 114 | 10 / 94 | 1.10 / 1.21 (same) |
| t[i] u16 | 15 / 170 | 15 / 170 | 13 / 154 | 1.15 / 1.10 (same) |
| `&&`/`\|\|` condition | 54 / 608 | 21 / 265 | 16 / 204 | 3.38 / 2.98 -> 1.31 / 1.30 |
| pong move_player (no ref) | 132 / 2580 | 123 / 2190 | - | - |

Per item (table rows that moved): item 1 hits, step, condition; item 2
count, fill, put_row, fill_rows, hits, bubble, sum, condition; item 3
bubble 169 -> 162, sum 43 -> 39 bytes, move_player; item 4 sum 1686 ->
1518 T, bubble; item 5 count, walk, fill, fill_rows, multiply, sum,
bubble; item 6 none (no literal-only expression in a fixture); item 7
fill_rows 4069 -> 3829 T, move_player 2358 -> 2190 T.

`pong.cas`: 959 payload bytes (1087 before H5). `selfcheck.cas`: 2239
(its source also changed in H5: the F-032 workaround is gone).

Bubble sort and the multiply stay far from hand-written code: their
locals live in static slots and every access is a load/store; that is
register allocation, not something the H5 items reach.

### H5 Fixture and Budget Changes

All budget moves are improvements.

- `expect-bytes`: contracts `inc` 3 -> 2, `count` 19 -> 15; control
  `both` 26 -> 25 (new); memory `pair_sum` 36 -> 35; registers `times` 18
  -> 15, `scale` 17 -> 13, `sum` 45 -> 35, `keep` 11 -> 8; shifts `gone` 3
  -> 2; spills `mix` 72 -> 70, `via_ptr` 18 -> 16; structs `nudge` 47 -> 41;
  pong `put_row` 38 -> 34.
- `expect-tstates`: pong `put_row` 166 -> 151, spills `via_ptr` 75 -> 69.
- registers_cg.fail.c80: `sum_pinned` (`s @ hl` with `*p`) now compiles
  (the pointer moves to DE) and was removed; 2 expected errors remain.
- optimize.c80 rewritten (blink: DJNZ, mask: folding, again: tail call).
- codegen_tests: `listing_maps…` (`INC A`, 4 T), `pinned_counter…` and
  `strict_caller…` (`OR A` entry guard + `DJNZ`), `default_functions…`
  (`INC A`), `zero_one_many…` (45/78/210 -> 55/81/185 T: rotation costs
  10 T at zero passes and 3 T at one, then saves 7 T a pass; this function's
  counter is not in B, so no DJNZ), `pinned_parameter…` (the dead entry
  move goes in optimized builds; still exactly one in `--no-optimize`),
  `call_and_repeat_timing…` (wrapper made non-tail), two BC=0 LDIR tests
  (layout-dependent timeout accepted). inline_tests `has_call` counts a
  tail `JP f`; project test accepts `JP` (0xC3) as the cross-unit call.
  semantics_tests checks the fold in optimized builds and the `u8` add in
  `--no-optimize`.
- Goldens in `info/c80/expected/` regenerated after every item; every diff
  reviewed.

## H5 Validation

- `cargo test -p rtvc-c80`: lib 237, bin 1, examples 4, rtvc_c80 9, tvc 4
  (selfcheck stack bound 16 = observed 16 in both modes; pong 300
  autopilot passes in both modes). Conformance ledgers empty.
- `cargo fmt -p rtvc-c80 -- --check`, `git diff --check`, trailing
  whitespace in untracked files: clean.
- `sh info/c80/compile.sh` succeeds; `cargo test --bin rtvc-tocas
  --no-default-features --features cli-tools`: 8 passed.
- Root crate untouched; `coding/` untouched.

## H6 — What Was Done

Leftovers first:

1. **Diagnostic cascades.** A use before assignment is reported once per
   local and function, and the error ends "(N further uses in this
   function not reported)"; an unresolved name likewise once per name and
   function. A local rejected for needing storage (an automatic array or
   struct) adds no further errors. `return &pinned` no longer adds the
   `ty-return-local-addr` warning next to the `ty-not-lvalue` error (nor for
   a `const`). Tests: `semantics_tests::one_error_per_unassigned_rejected_or_unresolved_name`,
   `address_of_a_pinned_local_is_one_error_without_the_dangling_warning`.
2. **Labels (F-035).** Checked every generated label shape against the
   helper assembler's `number()`, `reg8`/`reg16`/`condition_code`, and
   `indexed`: only function labels (prefix `F`, a hex digit) could read as a
   number; `G`, `IA`, and `U` prefixes cannot, and no label can equal a
   register or condition name because each starts with a letter prefix and
   a digit. Function labels are now `FN<id>_NAME` (block `_B<n>`, call
   `_C<n>`, slot `_S` suffixes follow), so no number form can match; the
   `_L` workaround is gone. `slot_key` in the optimizer follows the prefix.
   Two tests compile and run 54 functions and 54 globals (plus statics)
   named `a b c d e h l i r af bc de hl sp ix iy ixh ... nz z nc po pe p m
   dead beef cafe x0f h0 fh ffh deadh abh ... b1 b0 fb fn fnh`; the function
   test fails with the old `F<id>_` scheme. Goldens changed only in labels.
3. **Test time.** Profiling `conformance_random_i16` showed most time in
   `measure_functions`, the auto-inline trial lowering, which assembles the
   whole program per function even when there is no call to inline. It now
   runs only when some call targets an auto-inline candidate (the loop that
   uses it would find no site otherwise, so output is identical; goldens
   unchanged). `--lib` 14.0 s -> 8.7-9.3 s, with no test or case removed.

Then H6 proper:

- [info/c80.md](../../info/c80.md) rewritten by topic: Overview; Lexical
  Form; Grammar; Types and Conversions; Expressions and Operators (with
  Shifts); Statements; Data (globals, arrays, pointers, strings, structs,
  static initializers and addresses); Functions (calling conventions table,
  strict `@fastcall`, pins, `static` locals, contracts, inlining); Where
  Values Live (cost table, slots, homes, stack bounds, reading the
  listing); Inline Assembly; CPU Builtins; Units, Manifests, and Mixed BASIC
  Projects; CLI and Outputs; Optimization (each pass, in order); Limits and
  Diagnostics (unsupported table, every diagnostic code with meaning and
  fix); Testing; Library Entry. No E-/H-number headings remain.
- [c80/tests/docs.rs](../../c80/tests/docs.rs): every fenced block in the
  reference and the tutorial needs an info string; `c80` blocks compile in
  both modes, `c80 error=...` blocks must give exactly those codes, `text
  output` blocks must match the previous block's messages, `asm
  golden=FILE` blocks must appear in the golden listing, and `c`/unmarked
  blocks fail. It also checks the relative links (files and heading
  anchors) of the C80 docs, `README.md`, `TODO.md`, `CHANGES.md`, the plan
  files, and the development skill. 35 C80 blocks (14 compiling, 21
  rejected), 1 golden block, 4 output blocks, 360+ links.
- Tutorial: one section per example, each linking its source, golden
  listing, and rejected-forms file; new section 19 for `tvc-usr`; new
  section 22 "When the compiler says no" with real messages; the old "What
  C80 does not do" list (which still claimed `DJNZ` was not used for loops)
  folded into it with a link to the reference.
- Stale claims fixed: tutorial "`DJNZ` lowering of `while` loops" as
  unsupported, "no editor in this milestone"; reference "in this slice",
  E-/H-number headings, the "Explicitly unsupported" list (only `*`/`/`/`%`
  were listed); `calls.fail.c80` "Frames are not reentrant";
  `registers.c80` "(Before H5 ...)"; `spills.c80` slot label spelling; the
  parser's "is not accepted in this compiler slice" messages (now say what
  to do instead).
- [c80-compiler.md](c80-compiler.md): dated "Status After Hardening
  (2026-11)" section and "(Superseded: ...)" notes on the `@fastcall`,
  frame-slot, IX-frame, stack-bound, and `DJNZ` paragraphs.
- Repo docs: development skill (docs test, conformance dump, tocas flags,
  fmt, timings), `TODO.md` (hardening done, follow-ups), `CHANGES.md`
  (Unreleased), `README.md` (reference description),
  [plans README](README.md) (progress link; the dead `cleanup-laser-port1`
  entry removed), the skill's dead `src/wasm.rs` link fixed.
  [info/rtvc.md](../../info/rtvc.md) unchanged: no build target,
  dependency, or crate layout it describes changed.

## H6 Validation

- `cargo test -p rtvc-c80`: lib 240 (9.3 s), bin 1, docs 2, examples 4
  (6.7 s), rtvc_c80 9 (3.1 s), tvc 4 (7.2 s).
- `cargo fmt -p rtvc-c80 -- --check`, `git diff --check`, trailing
  whitespace in untracked files: clean.
- `sh info/c80/compile.sh` succeeds (selfcheck 2239 payload bytes);
  `cargo test --bin rtvc-tocas --no-default-features --features cli-tools`:
  8 passed.
- Quality table unchanged from H5.
- Root crate untouched; `coding/` and `info.hu/` untouched.

## Next Step

None in this plan. F-029 (calling ASM exports from C80) is open; the
follow-ups are in [TODO.md](../../TODO.md). Project builds keep the
pre-optimization stack bound (loose by 2 bytes per tail-called level), and
the assembler still prefers a number over a defined label (F-035).
