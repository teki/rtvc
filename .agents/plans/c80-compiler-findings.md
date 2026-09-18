# C80 Compiler Findings

Last updated: 2026-09-08

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
