# Implementation Plans

This directory contains detailed, actionable plans for substantial emulator
work.

[TODO.md](../../TODO.md) is the high-level backlog: it records what should be
added or fixed. A plan in this directory explains how a selected item should be
implemented, including scope, design decisions, integration points,
implementation order, limitations, and focused validation.

## Conventions

- Use a short descriptive kebab-case filename, such as
  `frame-history-debugger.md`.
- Link to the corresponding [TODO.md](../../TODO.md) entry when one exists.
- Prefer clear module boundaries and explicit integration points over detailed
  pseudocode that will quickly become stale.
- Record significant scope and design decisions so later agents do not need to
  rediscover them.
- Update the plan when implementation changes an important decision. Keep
  user-facing and implementation reference documentation authoritative for
  completed behavior.
- When implementation begins, create a sibling `<plan-name>-progress.md` file.
  Keep it as a concise continuation record: current status, completed work,
  decisions discovered during implementation, validation already run, and the
  next concrete steps. Update it whenever a work session materially changes
  the implementation state.

## Current Plans

- [C80 Compiler and Integrated Source View](c80-compiler.md)
  ([Terra execution plan](c80-compiler-execution.md),
  [status](c80-compiler-progress.md), [findings](c80-compiler-findings.md))
- [C80 Compiler Hardening](c80-hardening.md) — correctness, spills, shifts,
  code quality, tests, and documentation fixes from the 2026-11-12 review
  (done; [progress](c80-hardening-progress.md))
- [Expansion Card Abstraction and Host File Device](expansion-cards-and-host-file-device.md)
  ([progress](expansion-cards-and-host-file-device-progress.md))
- [Frame History Debugger](frame-history-debugger.md)
  ([progress](frame-history-debugger-progress.md))
- [Focused Execution Trace](instruction-trace.md)
  ([existing implementation progress](instruction-trace-progress.md))
- [Multi-System Architecture](multi-system-architecture.md)
- [rtvc-asm Repetition and Cycle Timing](rtvc-asm-repetition-and-timing.md)
  ([progress](rtvc-asm-repetition-and-timing-progress.md))
- [rtvc-asm Workflow Improvements](rtvc-asm-workflow-improvements.md) —
  expressions, `PHASE`, includes and symbol import, debugger breakpoints,
  CAS output, local labels, structs, and macros, ranked from a survey of
  popular 8-bit assemblers
  ([progress](rtvc-asm-workflow-improvements-progress.md))
- [TVC Video Signal Pipeline and Television Sync](tvc-video-signal-pipeline.md)
