# Focused Execution Trace

Status: proposed extension, 2026-09-08.

Backlog: [Debugger](../../TODO.md#debugger). Existing implementation history:
[Instruction Trace Progress](instruction-trace-progress.md).

## Objective and scope

Capture deliberately selected execution intervals into bounded memory, stop and
resume from the debugger, and export all retained records to a file. Include
registers, historical memory maps, mapping changes, and compact presentations of
repeating block instructions such as `LDIR`. Disabled tracing must do no capture
work and cause no measurable execution-performance regression.

This extends existing tracing; it is not reverse execution or deterministic replay.
General register before/after diffs, memory read logs, automatic capture triggers,
and execution coverage maps can follow later.

## Research context

The intended project is [SkoolKit](https://skoolkit.ca/). Its
[command reference](https://skoolkit.ca/docs/skoolkit/commands.html) documents
importing Fuse execution profiles into `sna2ctl.py`, and its own `trace.py`
simulator for chronological instruction/register traces. These are distinct
facilities. This design needs no Fuse or SkoolKit dependency.

## Existing foundation

- [instruction_trace.rs](../../src/emulator/instruction_trace.rs) already provides
  a bounded ring (1,000–1,000,000 entries), pre-instruction registers, four opcode
  bytes, clock/cycle counts, an interrupt marker, TVC raw mapper values, and
  memory/port writes. Start clears history; stop retains it.
- [tvc.rs](../../src/emulator/tvc.rs) and
  [zx82.rs](../../src/emulator/zx82.rs) capture at instruction boundaries, using
  optional bus effect collectors. Interrupt service writes/cycles currently belong
  to the preceding instruction record.
- [machine.rs](../../src/emulator/machine.rs) and
  [emu.rs](../../src/emulator/emu.rs) dispatch shared trace operations.
- [debug_ui.rs](../../src/debugger/debug_ui.rs) has an Instruction Trace pane;
  [debugger.rs](../../src/debugger/debugger.rs) and
  [rtvc_debug.py](../../scripts/rtvc_debug.py) provide start/stop/clear/status/list.
  List returns only the newest entries, capped at 10,000.
- [ui.rs](../../src/ui/ui.rs) provides native save-dialog and browser download
  patterns to reuse.

The gaps are resume, full export, decoded historical maps and mapping transitions,
block summaries, explicit loss accounting, and measured disabled performance.
Allocation-free existing hooks alone do not establish zero overhead.

## Lifecycle and controls

Use shared core operations for the pane and TCP debugger:

| Action | Behavior |
| --- | --- |
| New recording | Clear retained data, apply limits, and begin a session; preserve existing start semantics. |
| Stop | Finish the current atomic instruction boundary, then retain data. Does not pause the CPU. |
| Resume | Append to stopped data with a new segment and fresh register/map context. |
| Clear | Discard records and loss counts, retaining recording/stopped mode as today; active capture begins a fresh segment. |
| Export | Available while recording is stopped; save all retained data without consuming it. |

Label the UI explicitly: New recording, Resume, Stop, Clear, Export. Display state,
retained count, allocated bytes, limits, and overwritten/truncated counts. Paused
CPU stepping is captured while recording; ordinary breakpoints do not clear data.

Reset, snapshot load, frame-history restore, and machine replacement stop and
clear capture, with visible notification if data was discarded. This intentionally
revises current clear-only reset/load behavior. Traces remain outside snapshots
and persistent UI state. Debugger memory/register edits establish a new segment
before execution resumes; unrecorded execution must never appear continuous.

## Structured records and maps

Add a versioned session header, machine/profile identity, capture options, and
segment records with initial registers/map and clock origin. Instruction records
reference segment and map IDs and retain pre-state, bytes, timing, and effects.
Sequence numbers increase within a session; segment markers identify capture gaps.

Capture bytes using side-effect-free inspection, not device reads that can mutate
the machine. Audit four-byte lookahead for repeated prefixes and instructions
crossing mapping boundaries. Explicitly mark unsupported/truncated decoding;
never disassemble historical records by reading current memory. Self-modifying
code must display its captured bytes.

Provide machine-owned map descriptions: CPU range, physical bank/device, offset,
and separate read/write routing. TVC describes RAM, ROM, video and expansion
routing, retaining raw mapper bytes as details. Zx82 48K describes its fixed map.
Derive this from actual MMU routing, avoiding a second mapping implementation.

Capture mapping-change events at the actual change, with ordered effect position,
causal instruction/PC, port/value if applicable, and old/new context IDs. Sampling
only the next instruction would miss a change at the end of capture. Qualify writes
by their physical destination at write time, including ignored writes. Keep enough
map context to interpret the first retained record after eviction; release unused
contexts so map history is bounded too.

Distinguish opcode cycles/writes from interrupt service cycles/stack writes through
an explicit interrupt boundary or structured subrecord. Represent HALT waiting
separately from execution of HALT. Both are boundaries for block grouping.

## Memory and disabled performance

Keep overwrite-oldest ring behavior and the existing entry limit. Add a byte budget
(proposed default 64 MiB) covering allocated entry storage, effect payloads, map
contexts, and pending capture. Count capacity rather than only logical lengths.
Bound per-entry effects; a too-large entry is explicitly marked truncated with
omission counts. Export retained sequence ranges and loss counters. Never allow
variable write arrays or map churn to bypass the budget.

Disabled capture performs no register copies, map decoding, disassembly, formatting,
serialization, trace allocation, or file I/O. Audit current optional write-collector
checks as well as the instruction hook. If needed, select traced/untraced execution
at an outer run/step boundary using a shared generic implementation and a no-op
observer that compiles away. Avoid dynamic dispatch per bus access or duplicate CPU
instruction semantics. Toggles take effect at the next existing execution boundary.

Measure release performance against the pre-change baseline with identical fixed
emulated workloads: both machines, both applicable video modes, and write-heavy
code. Alternate repeated baseline/candidate runs and report variability. Require
zero disabled trace allocations and no reproducible regression beyond noise;
investigate a repeatable >1% difference. Inspect that capture work is absent from
the fast path. Report traced throughput/memory separately. A runtime toggle cannot
honestly promise literally zero extra CPU instructions on every platform.

## Repeating block instructions

Initially retain raw iteration records and collapse only a shared UI/text-export
projection. JSONL keeps raw records. This avoids changing CPU execution, interrupt
acceptance, contention, writes, or breakpoints just to shorten output.

Support LDIR/LDDR, CPIR/CPDR, INIR/INDR, OTIR/OTDR. Group only consecutive iterations
of the same decoded opcode at the same physical instruction location, within one
segment and compatible map context. Require observed repeat-PC/register continuity,
not just matching PCs. Split at interrupts, map/byte changes, edits, gaps and loss.
Never merge ISR activity or a later invocation at the same address.

Summaries contain first pre-state registers, iteration count, total cycles,
relevant final registers, and completion/partial reason. Capture the small final
register subset for block operations while recording: the next row may be missing
or follow an interrupt. Handle BC=0 transfer counts (65,536), B=0 block I/O counts,
early compare termination, wrapping, direction, and partially retained runs.
Keep ordered effects in raw output; text may show effect counts with truncation
notices. General register deltas remain a later optional schema extension.

Illustrative output (final syntax fixed by golden fixtures):

```text
MAP m0 ...decoded CPU ranges and read/write destinations...
#120 t=8400 map=m0 RAM:8000 ED B0 LDIR iterations=256 cycles=5371 BC=0100->0000 HL=9000->9100 DE=A000->A100 complete
#376 t=13771 map=m0 ... OUT (...) ...
MAP-CHANGE cause=#376 port=... value=... m0->m1 ...changed ranges...
```

## Export and integration

Add a UI-independent streaming formatter, proposed
`src/emulator/instruction_trace_export.rs`, accepting `std::io::Write`. Export
chronological UTF-8 text by default, with collapsed blocks, or versioned JSONL with
raw structured records. Include session/segment context, map definitions/events,
loss notices and exported counts. Identical captures/options give identical output;
formatting uses captured data only.

Native UI uses a save dialog and buffered writes, preferably a temporary file plus
successful replacement. Full-web uses the existing download helper, with an export
size bound accounting for serialization and Blob copies. Use worker/incremental
formatting if needed for responsiveness. Freeze the stopped capture during export
or use immutable shared ownership without copying the entire ring. Cancel/failure
retains capture, reports errors, and never reports a partial file as successful.

Preserve existing TCP commands. Add resume and bounded chronological pagination
with session identity, sequence cursor and next cursor. Explicitly reject stale
sessions or overwritten cursors. Full export must not reuse the newest-10,000 list
endpoint. Add `trace export <path> [text|jsonl]` to Python, writing to the client's
filesystem. Expose bounded formatted export chunks from Rust so Python does not
reimplement grouping. Freeze or invalidate export sessions on concurrent mutations;
never mix captures. No arbitrary server-side file destination is needed.

## Implementation order

1. Extend the existing progress file, preserving historical completed work. Add
   lifecycle, segment, byte-budget and loss model; shared Resume/status operations.
2. Audit inspection reads, interrupts and HALT; implement machine map descriptors,
   mapping transitions, physical writes, and bounded context ownership.
3. Implement/measure disabled execution path. Add a machine tracing benchmark if
   the existing CPU benchmark cannot measure these hooks.
4. Implement block projection and final block register capture.
5. Implement deterministic text/JSONL export and stable bounded TCP export chunks.
6. Wire pane controls, native save, browser download, Python export.
7. Document delivered behavior in [info/rtvc.md](../../info/rtvc.md), new workflows
   and benchmarks in the [development skill](../skills/development/SKILL.md), and
   translations as required by [L10N.md](../../L10N.md). Complete backlog/progress
   only after validation. Hardware docs need changes only for verified corrections.

## Validation and acceptance

- Lifecycle: start/stop/resume/clear, empty capture, paused stepping, breakpoints,
  reset/load/restore/switch, debugger edits, and multiple segments.
- Bounds: eviction, variable effects, oversized records, map churn, retained map
  context after eviction, loss markers, and bounded export memory.
- Accuracy: TVC mapping changes including the final instruction, separate read/write
  maps, Zx82 fixed mapping, self-modification, boundary-crossing bytes, interrupts,
  and HALT. Trace on/off must yield identical CPU, memory, devices and timing.
- Blocks: each family, zero counters, early comparison exit, wrapping, interrupts,
  map changes, stop and eviction mid-repeat. Check raw/summary count/cycle agreement.
- Export: golden text/JSONL, all retained entries beyond 10,000, cursor invalidation,
  writer failure/cancellation, and non-destructive saves.
- Run focused trace tests, library tests and native/full-web/lightweight WASM checks
  from the development skill. Run FUSE vectors if CPU execution dispatch changes.
  Smoke-test native save, browser download and TCP/Python export.
- Record release benchmark results and actual allocation accounting. Existing
  progress-file test results do not validate this proposed extension.

Done means users can capture, stop, resume another interval, and save all retained
data with readable historical maps and safe block summaries, while untraced
execution meets the performance criterion.
