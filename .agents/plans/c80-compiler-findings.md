# C80 Compiler Findings

Last updated: 2026-09-07

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
| Inline assembly operands/units | F005 | Yes | No | E08 (header syntax may be parsed earlier as unsupported) |
| BASIC substitutions and USR fixture | F006 | Yes | Stop only if verified ROM contradicts USR/LOMEM | E11 |
| Multiplication/division/remainder | Non-goals + F001 | Reject; multiplicative tokens may exist only for a diagnostic | No | E01/E02 diagnostics |
| Heap, implicit promotions, ROM wrappers | Non-goals | Reject | No | — |

**Resolved/open:** table recorded. No contradiction that invalidates execution order.

## F-002 — No `toml` crate in the current Cargo graph

**Status:** open (future implementation choice, not an E00 blocker)  
**Evidence:** [Cargo.toml](../../Cargo.toml) has no `toml` dependency. [rtvc_asm.rs](../../src/bin/rtvc_asm.rs) emits `rtvc-asm-v1` by hand.  
**Affected:** E07 manifest parse and segment output  
**Impact:** Manifest input needs a parser; segment output can keep the existing handwritten emitter.  
**Proposed resolution:** add an optional `toml` (and only if needed `serde`) dependency behind `compiler` / CLI, never lightweight `wasm`. Reuse handwritten `rtvc-asm-v1` emission unless a parser is required to re-read it.  
**Resolution authority:** implementer (E07); not a language decision.

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

**Status:** open (not needed until E07)  
**Evidence:** `cli-tools = ["dep:serde_json"]` only. `native` and `wasm-full` already enable `serde` + `serde_json`.  
**Affected:** E07 manifest/segment serialization if serde-derive is used in the compiler crate  
**Impact:** A serde-based compiler type would fail to compile under `cli-tools` unless `compiler` also enables `dep:serde` or `cli-tools` is widened.  
**Proposed resolution:** Keep E01–E06 free of serde. When E07 needs TOML/serde, put those deps on `compiler`, not lightweight `wasm`.  
**Resolution authority:** implementer (E07).

## F-007 — Prologue ABI copies must not clobber later arguments

**Status:** resolved in E04  
**Evidence:** Sequential `LD C,A` then `LD B,C` for `(bool, bool)` overwrote the second argument. Nested `if (a) if (b)` returned the true branch when `b` was false.  
**Affected:** E04 register homes for multi-block functions; E05 parallel ABI moves  
**Impact:** Argument rehomes after entry must be ordered so a destination is not a still-pending source (cycle-breaking belongs to E05).  
**Proposed resolution:** emit pending ABI-to-stable moves in dest-not-a-source order. Implemented in `assign_stable_homes`.  
**Resolution authority:** implementer (E04).
