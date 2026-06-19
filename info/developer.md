# rtvc Developer Notes

This document collects practical development findings that are useful while
working on rtvc but do not belong in the implementation-neutral TVC hardware
reference. Add short, verified notes here when an experiment uncovers a workflow
sharp edge, debugger trick, or repository-specific convention.

## CAS Injection and BASIC Startup

The `-i` / `--inject` command-line option writes a TVC CAS payload directly into
the active machine's BASIC memory before normal emulation starts. For
`BASIC_START` helper programs this means:

```bash
cargo run --bin rtvc -- -i data/porting/test.cas
```

is not reliable from a cold boot. The CAS bytes are injected before BASIC has
finished cold-start initialization, and BASIC may later clear or reinitialize
the program area around `19EFH`.

Use a booted snapshot when testing injected `BASIC_START` programs:

```bash
cargo run --bin rtvc -- data/snapshots/boot12dos.rtvcsnap.zip -i data/porting/test.cas
```

This starts from initialized BASIC/VT-DOS state and then injects the tokenized
BASIC stub and machine-code payload. A mapped memory read can verify the result:

```text
19EFH: 0F 0A 00 43 9A 55 53 52 96 36 37 30 34 95 FF 00 ...
1A30H: machine-code payload emitted after BASIC_START
```

The first bytes at `19EFH` are the one-line `USR(6704)` BASIC launcher emitted
by `rtvc-asm`; `1A30H` is the machine-code entry point.
