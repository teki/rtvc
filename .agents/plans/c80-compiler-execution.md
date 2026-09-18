# C80 Compiler Execution Plan for Terra

## Assignment and Scope

Implement the standalone C80 compiler incrementally using `gpt-5.6-terra`.
This file is an execution guide, not an instruction to create another task,
spawn agents, switch the running task's model, or start implementation merely
because the file exists. Start when the user assigns an increment.

The authoritative design is [C80 Compiler and Integrated Source View](c80-compiler.md).
E00-E12 cover its Phase 1; E13/E14 add explicit CAS packaging follow-ups in
rtvc-tocas. Editor panes, live compilation, emulator load UI, source breakpoints/
stepping, release publishing, and remote operations are outside scope. Preserve
source provenance so future consumers can be added.

Follow [AGENTS.md](../../AGENTS.md) and the
[development skill](../skills/development/SKILL.md). Do not stage, commit, or push
without the user's separate written authorization. Preserve unrelated changes.
Do not delegate this assignment unless explicitly instructed by the user.

## Required Progress Files

Use both files from the beginning, including before writing compiler code:

- [Status and continuation record](c80-compiler-progress.md): current increment,
  per-increment state, exact last successful gate, changed paths, validation
  commands/results, blockers, and the next concrete action.
- [Findings and decisions](c80-compiler-findings.md): numbered findings with
  evidence, affected design/execution section, impact, proposed resolution,
  resolution authority, and resolved/open state. Keep historical findings; do
  not erase a blocker merely because a later implementation takes another path.

Read both on every resume. Update status before starting an increment, after
its gate, before any handoff, and immediately when blocked. Append material
discoveries to findings as they occur. Never claim a test ran without its actual
result. Distinguish implementation complete, tests passed, and checks unavailable.
Record baseline failures separately from regressions introduced by this work.

## Stop Rule: Issues With This Execution File

**Stop implementation if this file is inconsistent, incorrect, incomplete in a
way that requires a semantic/design choice, impossible against the repository,
or conflicts with the authoritative design or user instructions.**

1. Preserve the current changes; do not reset or discard them.
2. Set the active increment to `BLOCKED` in the status file.
3. Add a finding quoting/linking the problematic requirement, the evidence,
   the affected increments, and a concrete proposed correction or decision.
4. Report the issue to the user and end the implementation turn. Do not proceed
   to dependent or unrelated increments to work around the blocker.
5. Resume only when the user resolves the issue or authorizes an execution-plan
   correction. Record that resolution before continuing.

Do not rewrite this execution file or weaken acceptance tests to make a failing
implementation appear compliant. Ordinary implementation bugs should be fixed
within the increment; their existence is not itself a plan defect. Local names,
module splits, and test-helper organization are implementation choices. New
language semantics, changed ABI/layout, weaker memory observability, or broader
scope are not. Missing tools/dependencies that prevent a required gate must be
reported as blocked verification, never silently treated as a pass.

## Working Loop and Increment Boundaries

Execute E00 first, then increments in order. Each increment is independently
reviewable and ends with a buildable tree, focused tests, updated documentation
for implemented behavior, and a status checkpoint. If only one increment was
assigned, stop there. If a range or the whole plan was assigned, continue through
passing gates within that authorization. Never skip a failed gate.

At each increment:

1. Inspect `git status --short`, the relevant source, and prior findings.
2. Identify its inputs, deliverables, test IDs, and explicit non-goals.
3. Implement the smallest complete slice. Reject unsupported source constructs
   with located diagnostics; do not return plausible stub code.
4. Run focused tests, then required integration checks for touched boundaries.
5. Inspect the diff for accidental edits and run `git diff --check`.
6. Update progress with exact commands, outcomes, test counts, limitations,
   and next steps. An intermediate checkpoint is not a compiler release.

## Non-Negotiable Design Constraints

- Source order determines code/data placement within each unit. Never collect
  global text/data/BSS sections or silently move data between mapping windows.
- Default calls use registers; `@stackcall` is explicit. Do not route normal
  expressions, joins, or loops through the stack as a universal lowering scheme.
- Mutable source-memory reads and writes are observable and ordered. Pure
  scalar locals and compiler-private spills remain optimizable.
- No implicit initialization of locals; reads require definite assignment.
  Explicit assembly outputs establish values. Typed conversions require casts;
  fitting literals acquire their contextual type.
- No multiplication/division/remainder, heap, recursion, implicit promotions,
  automatic ROM/BASIC wrappers, or automatic interrupt management.
- Project reserves stack bytes; programmer-written startup sets SP. Calls from
  BASIC/assembly may inherit the caller's stack. Report unknown external stack
  usage honestly.
- Reuse `assemble_program` for encoding and disassembler metadata for timing.
  IDs and source spans travel through the backend; do not infer provenance from
  disassembly text.
- No UI/emulator ownership in the compiler API. Do not modify CPU behavior to
  accommodate incorrect generated code.

## Repository Integration Map

| Concern | Existing entry point |
| --- | --- |
| Library modules/features | [src/lib.rs](../../src/lib.rs), [Cargo.toml](../../Cargo.toml) |
| Assembler results and encoding | [asm.rs](../../src/emulator/asm.rs), [asm_tests.rs](../../src/emulator/asm_tests.rs) |
| Timing/disassembly | [disasm.rs](../../src/emulator/disasm.rs), [disasm_tests.rs](../../src/emulator/disasm_tests.rs) |
| CPU execution and test memory | [z80.rs](../../src/emulator/z80.rs), [bus.rs](../../src/emulator/bus.rs) |
| CLI/segment output precedent | [rtvc_asm.rs](../../src/bin/rtvc_asm.rs), [assembler reference](../../info/assembler.md) |
| BASIC tokenizer/CLI | [basic.rs](../../src/emulator/basic.rs), [rtvc_basic.rs](../../src/bin/rtvc_basic.rs) |
| TVC integration | [tvc_tests.rs](../../src/emulator/tvc_tests.rs), [rtvc reference](../../info/rtvc.md) |
| Verified ROM contracts | [ROM listing index](../../roms/README.md) |

Create compiler modules under `src/compiler/`; keep filesystem access in the
CLI/project input adapter and compile owned/in-memory source snapshots in the
core. Proposed tests live under that subsystem with small checked-in fixtures;
choose actual paths in E00 and record them. Do not create a second assembler,
CPU interpreter, or BASIC tokenizer.

## Increment Schedule

| ID | Deliverable | Design phase |
| --- | --- | --- |
| E00 | Repository baseline and contract readiness | 1A prerequisites |
| E01 | Sources, diagnostics, lexer/parser | 1A |
| E02 | Scalar type checking and definite assignment | 1A |
| E03 | Register leaf code and executable harness | 1B subset |
| E04 | Branches, observable memory, register-resident loops | 1B subset |
| E05 | Calls, spills, explicit stack ABI, stack accounting | 1B subset |
| E06 | Arrays, pointers, strings, constants | 1A/1B completion |
| E07 | Units, placement, manual assembly entry, CLI | 1B |
| E08 | Explicit inline assembly and external contracts | 1B completion |
| E09 | Complete maps, symbols, listings, timing | 1C |
| E10 | Structs and remaining language conveniences | 1D |
| E11 | Minimal mixed BASIC/C80 project | 1D |
| E12 | Code quality, cross-target validation, handoff | 1E/release readiness |
| E13 | TOML to one zero-padded CAS memory block | Packaging follow-up |
| E14 | Optional compression and Z80 decompression | Packaging follow-up |

The subdivisions of 1B are private development checkpoints. Do not describe
Phase 1B as delivered until E08 passes. Provenance is carried from E01 onward;
E09 completes indexing/metadata rather than retrofitting source spans.

## E00 — Baseline and Contract Readiness

**Work:** Read the design's Remaining Implementation Contracts, this file, the
progress/findings files, and relevant repository references. Inventory existing
uncommitted changes. Establish focused assembler/BASIC and no-UI build baselines.
Record actual test discovery and available Rust/WASM targets.

Propose a `compiler` feature enabled by `native`, `wasm-full`, and `cli-tools`,
but not lightweight `wasm`. The new binary must have appropriate required
features. Verify this fits current Cargo structure before editing it. Keep TOML
and metadata serialization dependencies out of lightweight builds.

Create a contract-readiness table in findings for operator/literal/shift rules,
pointer lifetime/str/aggregate rules, exact ABI, assembly operands/source units,
manifest/reservation symbols, and BASIC substitutions. Record which are already
specified and which have a decision pending. Do not invent missing behavior.
Missing semantics block the increment that requires them; contradictions that
invalidate the execution order block E00. Report known future decision points
now, not only when their code is half written.

**Gate T00:** Relevant baselines are recorded with command/results; integration
paths and contract owners are identified; E01 has no unresolved prerequisite.
No compiler implementation is required for E00.

## E01 — Source Model, Diagnostics, and Parser

**Work:** Add feature/module plumbing, FileId/SourceSpan byte offsets and line
indexing, stable per-build IDs, diagnostics, located tokens, and a recoverable
parser for the first scalar/function syntax. Document the accepted grammar in
the planned `info/c80.md` reference as implemented scope. Keep unsupported later
constructs explicit. Do not add assembler emission yet.

**Tests T01:** UTF-8 offsets with ASCII identifiers; comments/escapes; malformed
literals; unterminated strings/comments; precedence/associativity for settled
operators; incomplete declarations/blocks; bounded recovery reaching a later
valid function. Assert located diagnostics and progress, not only no panic.

**Gate:** Parser tests and headless library check pass. A source snapshot can
produce AST plus diagnostics without filesystem, UI, or emulator state.

## E02 — Scalar Semantics and Definite Assignment

**Work:** Resolve scopes/functions/scalar constants. Add typed values with
separate storage width/signed interpretation, explicit conversions, and typed
control-flow IR. Implement scalar sizeof, lvalue validation, missing returns,
short-circuit semantics, and definite assignment. Reject direct/mutual recursion
in the currently visible function graph. Cover later cross-unit recursion in E07.
Before implementing unresolved operator edge cases, apply the stop rule.

**Tests T02:** Signed/unsigned limits, narrowing and widening, bool conversion,
mixed typed operands rejected, contextual literals, local shadowing/duplicates,
branches assigning on one versus all paths, zero-trip loops, break/continue,
use before assignment, void/non-void returns, recursive calls, rejected C syntax.

**Gate:** Small scalar programs yield typed IR; invalid ones yield located
diagnostics without an assembled result. Semantic rules used are documented.

## E03 — First Register-Only Executable Functions

**Work:** Add structured Z80 instructions with register/flag constraints and
symbolic labels. Implement constants, moves, scalar arithmetic/bitwise operations,
and return in the settled register ABI for small leaf functions. Reuse the
assembler; retain originating IDs/spans in every emitted item. Add a bounded
execution harness using the existing Z80 and FakeBus/Bus interface.

**Harness:** Load segments, initialize registers/SP explicitly, push a known
return sentinel for callable fixtures, and stop when control reaches it. Assert
instruction/time limits, final SP, registers, and relevant memory. A timeout is
a failure, never a successful termination. Keep instruction fetches distinct
from assertions about source data accesses by address/range or trace context.

**Tests T03:** Byte identity emits RET; HL/DE add emits ADD HL,DE followed by RET;
no IX frame or argument slots. Execute zero/max/carry/sign-boundary inputs.
Round-trip canonical bytes through assembler/disassembler. Check deterministic
assembly/IDs for identical inputs within the documented per-build model.

**Gate:** First compiled functions execute correctly and pass exact tiny
byte/size/T-state fixtures. Do not implement calls by spilling everything.

## E04 — Control Flow and Useful Register Loops

**Work:** Add basic-block liveness, overlapping register resources, edge moves,
comparisons, conditional branches, while/break/continue, and ordered global
loads/stores. Keep local values in compatible registers across branches and
loop backedges; use JP before optional branch shortening. Add a private pointer
walk fixture once the E06 addressing subset is available, without claiming E04
alone completes pointer support.

**Tests T04:** Signed comparisons across overflow boundaries; both short-circuit
paths and suppressed side effects; zero/one/many iterations; nested branches;
long branches beyond JR range; repeated writes to one global must all occur.
A bus-controlled shared byte changes between polling reads and must be reread.
Simple local-counter loop has no per-iteration stack reads/writes and no needless
IX frame. Record its bytes and measured loop T-states.

**Gate:** Register-preserving loop and observable-memory tests pass. Global
memory accesses must not be optimized into scalar-local behavior.

## E05 — Calls, Register Pressure, and Stack Bounds

**Work:** Add left-to-right argument evaluation, parallel ABI moves, live-value
preservation, deterministic spills, minimal frames, explicit @stackcall, and
return-value-safe cleanup. Finalize frame offsets after lowering. Compute stack
depth from lowered call sites and the acyclic call graph; distinguish proven,
declared, and unknown bounds. Do not add hidden static global temporaries.

**Tests T05:** Byte/word/mixed signatures; pairs conflicting with byte registers;
register move cycles; nested calls in later arguments preserving earlier values;
high-pressure expressions; stackcall slot extension/cleanup; caller IX/IY and
SP preservation; indexed-frame boundary rejection; recursive graph diagnostics.
Compare computed bounds to execution high-water for paths designed to hit the
maximum. Exercise sibling and nested calls without double-counting argument slots.

**Gate:** Calls execute under both conventions, tiny leaves remain frame-free,
and stack use is justified by live values rather than syntactic boundaries.

## E06 — Arrays, Pointers, Strings, and Static Data

**Work:** Add fixed arrays, address-taking/dereference, scaled pointer arithmetic,
immutable prefixed strings/references, and size/layout metadata. Preserve top-level
declaration order; literals follow their owning function without fallthrough.
Apply observable accesses to addressable locals and all mutable source memory.
Address arithmetic may wrap; placement arithmetic must not. Do not implement
aggregate copying or pointer-lifetime behavior without a settled contract.

**Tests T06:** Constant/dynamic indexing, constant bounds errors, pointer aliasing
direct locals/globals, ordered word-byte accesses, 16-bit address wrap, sizeof,
ASCII/escaped strings of length 0/255 and rejected 256, prefix versus payload,
known/unknown string-length checks. Check the worked data-only unit's exact
declaration-order addresses. Add a buffer fill/pointer-walk execution fixture
with no per-iteration stack traffic when register pressure permits.

**Gate:** Data layout and read/write traces match the language contract; no
global section regrouping or folded mutable-memory reads are introduced.

## E07 — Project Link, Assembly Units, and CLI

**Work:** Add manifest parsing/input adapters, pub/import lookup, whole-project
symbol resolution, declaration-order unit placement, collision-free names,
reservations, and project-owned origins. Add manual startup/interrupt assembly
units with explicit exports/references. They have no compiler prologue/epilogue.
Implement rtvc-c80 CLI assembly/segment output and safe error handling. Preserve
existing assembler behavior while extending listing metadata only as necessary.

**Artifact:** `--emit-segments PATH` writes a complete C80/ASM addressed memory
image as `rtvc-asm-v1` TOML. E07 does not require BASIC, CAS, or an interpreter
snapshot. "Loadable" means its bytes can be placed at the serialized addresses.

**Tests T07:** Private/public lookup, missing/duplicate units, relative manifest
paths from another working directory, case/underscore symbol collisions, constants-
only units, cross-unit calls/recursion, increasing callee size updates callers,
overlaps/overflow and adjacent legal ranges, reservations emitting no bytes,
stack ending at 10000H with intentional SP encoding. Execute a manual assembly
entry calling C80 and returning to its explicit continuation. Validate CLI errors,
output write failures, no successful partial build on error, and separated binary
range handling. Re-read TOML segments through the existing supported format.
For the startup execution test, load the CLI-produced TOML back into the test
bus, establish the explicit startup PC/environment, and run to a bounded
continuation. Do not test only an in-memory pre-serialization result.

**Gate:** Multi-unit project plus manual assembly startup builds through the CLI
and executes in the harness. No autogenerated startup or interrupt management.
Update development commands and architecture/CLI documentation now.

## E08 — Explicit Inline Assembly

**Work:** Implement settled in/out/inout registers, flag outputs, clobbers,
conservative memory effects, output destination capture, and declared stack
allowances. Preserve live values without corrupting assembly results. Validate
ordinary block control flow separately from standalone entry units. Permit
explicit returning calls under their contracts; do not generate ROM wrappers.

**Tests T08:** Overlapping operands rejected; legal input/output reuse; early
capture of flags; register permutations; output establishes definite assignment;
inout requires assignment; indexed output address evaluated once; output stores
cannot destroy other outputs; unknown external clobbers/stack use remain honest.
Reject ORG/BASIC_START placement escapes and illegal block exits. Execute explicit
port IO and LDIR with counts 1/many; test raw zero-count behavior distinctly.
Use a deterministic assembly stub for external-call tests before relying on ROM.

**Gate:** Explicit machine interaction is usable, minimal, documented, and
source-mapped. E03-E08 together satisfy the first executable compiler milestone.

## E09 — Full Provenance, Listing, and Timing

**Work:** Complete bidirectional indexes, typed symbols, data ownership, synthetic
provenance, multi-origin merged items, and eliminated/no-code mappings. Join final
assembler metadata to compiler IDs; derive base instruction timing from existing
metadata. Keep calls/loops from being reported as falsely exact total costs.
Do not stabilize a public serialized map schema solely for this milestone.

**Tests T09:** Every emitted byte belongs to data or an instruction; final byte
addresses match segments; source-to-many and many-to-source mappings; synthetic
startup/frame/edge code; inline assembly spans; eliminated statements; changed
origins; branch timings; unsupported/repeating timing represented honestly.
Failed results cannot expose a valid partial loadable program. Metadata retains
the source snapshot/build identity needed by future editor consumers.

**Gate:** CLI/library clients can inspect complete maps, symbols, bytes, timing,
and stack status without reading UI objects or reparsing display text.

## E10 — Remaining Planned Language

**Work:** Add packed structs/fields/pointers-to-structs, richer static array data,
for/do-while, compound assignment, and increment/decrement after their semantics
are explicit. Add only documented operations; no multiplication or hidden helpers.
Reuse settled literal/type/sizeof and memory-observability rules.
Add `cpu::in(u8) -> u8` and `cpu::out(u8, u8) -> void` using the
[proposed port I/O contract](c80-compiler.md#proposed-port-input-and-output-form-phase-1d--e10)
(`cpu::` namespace; also `cpu::di`, `cpu::ei`, `cpu::ldir`).
Use effectful typed IR, immediate-port lowering for TVC constants, and
C-addressed dynamic I/O. Preserve evaluation order, live registers, and source
provenance; follow the documented target-specific high-byte contract.

**Tests T10:** Packed field offsets and sizes; array-of-struct strides including
non-power-of-two sizes; pointer fields; exact static data bytes; compound/indexed
assignment single evaluation; prefix/postfix result behavior; continue in for
executes the update, continue in do-while reaches the condition; nested breaks;
observable effects preserved. Negative tests for unsupported aggregates/operators.
For port I/O, assert byte port numbers/data and ordered bus effects, including
TVC immediate/dynamic forms, zero-extended addresses on other targets,
discarded input results, repeated outputs,
short-circuiting, polling, nested operand calls/input, memory ordering, and
register/flag preservation. Check type/arity/range diagnostics, explicit
conversions, assembler round trips, and source/timing metadata. Optimized builds
must preserve the same I/O trace.

**Gate:** Planned language is accepted or deliberately rejected exactly as its
reference specifies, and representative graphics/data routines execute correctly.

## E11 — Minimal BASIC/C80 Linking

**Work:** Reuse the BASIC tokenizer for one BASIC source per TVC project. Resolve
explicit address/reservation symbols after C80 placement, preserve strings/comments,
and validate exact tokenized size. Export reservation constants, not automatic
interpreter management. Supply manual BASIC/assembly setup and call examples.

**Artifact:** Extend `--emit-segments` to include the exact tokenized BASIC
payload, including its terminator, at `[basic].origin` in the same `rtvc-asm-v1`
TOML as C80/ASM. Export `project::basic_base` and `project::basic_program_size`.
Workspace reservations emit no padding. Mixed `--emit-asm` must reproduce the
same image via ORG/DB; `--emit-bin` applies its existing contiguity check to all
emitted ranges. CAS packaging remains deferred. Follow F004/F006 in the design.

**Tests T11:** Symbol changes after code growth; missing symbols; substitution
boundaries; exact BASIC size/overlap; no unintended string/REM changes. Execute
a small BASIC USR call through the existing TVC/BASIC load path, verifying HL
input/result and interpreter continuation. Verify manual reservation/setup before
C80 memory can be overwritten. Record ROM profile and fixture prerequisites.

The USR execution test must read the CLI-produced TOML and execute its actual
BASIC and C80 bytes. Follow F006's Serialized Mixed-Image Acceptance procedure:
explicit LOMEM/interpreter setup, load all artifact segments, BASIC RUN, compiled
call/return, checked results and interpreter continuation. A setup helper may
detokenize/enter BASIC from the serialized artifact to establish ROM-owned
pointers before reloading those exact segments. It must not bypass the artifact
by recompiling/retyping the original source. Provide the reproducible loading
recipe and verify code remains intact after normal BASIC activity.

**Gate:** One CLI-produced mixed TOML artifact works through explicit setup and
the actual BASIC interpreter. In-process substitution/tokenization alone is
insufficient. Revalidate any earlier E11 completion against this clarified gate;
record the result before reaffirming E12 completion. Do not claim
automatic cassette packaging, workspace protection, or arbitrary ROM compatibility.

## E12 — Code Quality and Final Compiler Gate

**Work:** Apply provenance-preserving optimizations beyond the baseline: redundant
private moves/spills, complex-edge propagation, proven induction/strength reduction,
and range-checked branch shortening. Keep mutable memory effects intact. Add a
representative compile-latency fixture and record machine/configuration/results.
Complete language/ABI/CLI/metadata documentation and native/headless/full-WASM
feature boundaries. Do not add an editor or release artifacts.

**Tests T12:** Execute baseline/optimized fixtures with identical initial states;
compare results, ordered source-memory/IO effects, SP behavior, and provenance.
Verify JR/DJNZ displacement limits after final layout, zero/one/max loop counts,
and no wrapping-induction mistakes. Record byte/T-state improvements rather than
claiming optimization from source appearance. Run the final regression matrix.

**Gate:** Every earlier gate passed; final matrix passed or completion remains
blocked; no open design/execution issues; documentation matches actual behavior.
Final response states implemented scope, test evidence, measured code quality,
limitations, and progress/findings paths. No staging/commit/push is implied.

## E13 — Linear TOML-to-CAS Packaging

This and E14 are follow-up packaging increments, separate from the E00-E12
standalone compiler gates. Their explicit scope overrides the earlier packaging
exclusion only for these two increments. Do not reopen or replace the linker.

**Work:** Extend `rtvc-tocas` to accept `rtvc-asm-v1` TOML alongside existing
.bas/.asm inputs. Validate and sort addressed segments; flatten the span from
19EFH through the highest emitted end, zero-filling gaps. Reuse `encode_tvc_cas`.
Keep current .bas/.asm behavior. No segment relocation, decompressor, or hidden
startup generation. Adopt the design's CAS Follow-Up load-profile restrictions;
the user supplies a tokenized BASIC entry/launch stub at 19EFH. Validate its
record structure through the BASIC terminator while allowing following machine
code/padding. Reject empty, overlapping, out-of-range, unsupported-origin, or
malformed images before replacing output. Emit a sibling .cas and report actual
payload/padding bytes; TOML conversion uses the existing ASM CAS autostart profile.

**Tests T13:** Exact preserved bytes and zero gaps for out-of-order segments;
adjacent segments; boundary/overlap/invalid BASIC errors; correct CAS header size
and origin; existing .bas/.asm conversion regressions. Load the resulting CAS
through cassette/ROM loading, run its explicit BASIC/ASM entry, and verify a
known memory result and interpreter/program continuation. Assert cassette-loaded
RAM matches the linear payload. Do not bypass the loader with segment injection.
Document supported origins, whole-span ownership, manual startup, and padding.

**Gate:** A real multi-segment linked TOML becomes one runnable linear CAS,
without a relocating loader. Record T13 evidence in status/findings. Update the
development skill and tool usage reference when this command becomes real.

## E14 — Optional ZX0 Compression of the Linear CAS Image

**Work:** Add `--compression none|zx0` to E13 (default `none`). Follow the
[design's ZX0 contract](c80-compiler.md#cas-follow-up-linear-image-first-compression-second):
port the pinned upstream optimal parser and stream writer to native Rust and
adapt its standard forward Z80 decoder to the helper assembler. Use ZX0 v2,
not a custom run-length format. Record upstream commit, provenance, license
notices, and attribution. Document bit ordering, offset/length limits, initial
last offset, length-bit backtracking, and termination from the pinned sources.
Provide deterministic compression with resource limits appropriate to the TVC
image size and a checked host decoder/validator. Reject empty input, invalid
references, overflow, truncated streams/missing termination, and decoded-size
mismatch. Upstream tools are development oracles, not normal build/runtime
dependencies; check in compatibility fixtures.

Reconstruct the exact E13 image, including zero gaps, at its linked addresses.
The launch stub decodes then invokes the explicitly configured continuation;
this bootstrap is separate from user interrupt/SP/program setup. Require scratch
memory for staged input, decoder, live bootstrap, and active stack, disjoint from
the destination. Validate all live ranges during initial cassette loading,
staging, and decoding; reject unsafe overlap or insufficient space. Document
register clobbers and verify peak stack use. The compact target decoder is for
packager-validated streams, not arbitrary malformed input. Report packed size
and total cassette size including bootstrap/staging overhead; retain the
uncompressed option when ZX0 gives no net savings.

**Tests T14:** Empty-input rejection; one-byte, all-zero, repeated-pattern,
literal-heavy, and maximum-supported images; offset/length and bit-byte
boundaries; host rejection of truncated/invalid streams and decoded-size
mismatches; deterministic Rust round trips. Decode upstream v2 golden streams
with both the host validator and assembled Z80 decoder, and verify upstream
decoding accepts Rust output. Record fixture provenance and oracle commands.
Require byte-identical compressed fixtures where upstream parsing/tie-breaking
are preserved; document intentional equivalent encodings.

Execute the assembled Z80 decoder on Rust-generated vectors with bounded
completion, exact output comparison, stack checks, and memory canaries. Load a
compressed CAS through the real cassette path, decompress, and run its
continuation; compare results to the uncompressed fixture. Test scratch
overlap/capacity rejection and report total size savings on a padding-heavy
image plus overhead on an incompressible image. Unknown execution or
decompression results cannot pass.

**Gate:** Rust ZX0 v2 output interoperates with upstream and the adapted Z80
decoder, reduces total cassette storage for the padded fixture, restores
byte-identical RAM, and executes without corrupting unread input, decoder state,
or reserved memory. Update status/findings and tool documentation when implemented.

## Test Commands and Evidence Rules

Use actual test-module filters discovered in E00; proposed compiler namespace is
`compiler::`. Check test counts so a misspelled filter with zero tests never passes
a gate. The commands below assume the feature wiring described above exists;
before E01 use the corresponding existing feature configuration for baselines.

```sh
cargo test --lib --no-default-features --features cli-tools compiler::
cargo test --lib --no-default-features --features cli-tools asm::
cargo test --lib --no-default-features --features cli-tools disasm::
cargo test --lib --no-default-features --features cli-tools basic::
cargo check --lib --no-default-features --features cli-tools
git diff --check
```

Run relevant focused tests after changes. Run assembler/disassembler regressions
when their APIs or metadata change, BASIC regressions when integration changes,
and the full matrix at E12. Add and record the actual CLI integration-test command
once its harness exists; `cargo test --lib` does not exercise CLI integration tests.

```sh
cargo test --no-default-features --features cli-tools
cargo run --no-default-features --bin fuse_test
cargo check
cargo check --bins
cargo check --lib --no-default-features --features wasm,web-vid-simple --target wasm32-unknown-unknown
cargo check --lib --no-default-features --features wasm,web-vid-realistic --target wasm32-unknown-unknown
cargo check --lib --no-default-features --features wasm-full --target wasm32-unknown-unknown
cargo check --manifest-path xtask/Cargo.toml
cargo tree --no-default-features --features wasm,web-vid-simple -e normal --target wasm32-unknown-unknown
```

Verify lightweight builds exclude compiler/editor dependencies and CLI builds do
not pull in desktop UI/audio. Run feature checks when feature wiring changes,
not solely at final completion. ZEX is not routine compiler validation; consult
the development skill if CPU changes become necessary and apply the stop rule
before expanding this assignment into CPU implementation work.

For each assigned T00-T14 gate record its passing command/count and representative fixture
names in progress. For execution tests assert actual results and bounded completion,
not merely successful assembly. Static byte fixtures are intentionally tiny;
larger programs should use semantic assertions and explicit quality thresholds.
