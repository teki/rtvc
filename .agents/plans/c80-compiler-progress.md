# C80 Compiler Progress

Last updated: 2026-09-07

## Current increment

- **ID:** E05
- **State:** passed
- **Authorization:** user asked to commit after each phase, then continue

## Per-increment state

| ID | State | Last successful gate | Notes |
| --- | --- | --- | --- |
| E00–E04 | passed | T04 | Committed as 3c9e25e |
| E05 | passed | T05 | Calls, spills, `@stackcall`, stack bounds |
| E06–E12 | not started | — | — |

## Changed paths (E05)

- `src/compiler/` lexer/parser/AST/IR/ABI lowering/harness/tests
- `info/c80.md`, `info/rtvc.md`
- `.agents/plans/c80-compiler-progress.md`, `.agents/plans/c80-compiler-findings.md`

## Validation

```text
cargo test --lib --no-default-features --features cli-tools compiler::
  44 passed; 0 failed; 119 filtered out
cargo check --lib --no-default-features --features cli-tools
  ok
git diff --check
  clean
```

## Next action

Commit E05, then start E06 (arrays, pointers, strings).
