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

## VT-DOS File I/O from Assembly

The ROM variable at `1705H` stores the I/O device selector. A ROM-list comment
describes it as the I/O device number multiplied by `10H`; the low nibble is
then used for the operation number. Do not inherit the value found there at the
BASIC prompt: `20H` selects the editor, not the disk-compatible cassette
device.

VT-DOS replaces cassette I/O transparently. For disk input, BASIC selects
device `50H`, adds the `80H` input bit, and stores `D0H` at `1705H`. The useful
selectors are therefore:

| Operation | Output | Input |
| --- | ---: | ---: |
| Character | `51H` | `D1H` |
| Block | `52H` | `D2H` |
| Open/create | `53H` | `D3H` |
| Close | `54H` | `D4H` |

Dispatch through the writable RAM trampoline at `001BH`; it stores `A` in the
inline function byte at `001FH`, executes `RST 30H`, and returns. Assembly
filenames use a length byte rather than a zero terminator:

```asm
        XOR A
        LD (0B6BH),A            ; non-buffered file
        LD DE,FILE_NAME
        LD A,0D0H
        LD (1705H),A             ; VT-DOS disk input
        OR 03H
        CALL 001BH               ; D3H: open input
        OR A
        JR NZ,ERROR

        LD BC,0010H
        LD DE,19DFH
        LD A,(1705H)
        OR 82H
        CALL 001BH               ; D2H: read application header
        OR A
        JR NZ,CLOSE_ERROR

        LD BC,PAYLOAD_SIZE
        LD DE,PAYLOAD_ADDRESS
        LD A,(1705H)
        OR 82H
        CALL 001BH               ; D2H: read payload at DE
        PUSH AF

CLOSE:
        LD A,(1705H)
        OR 04H
        CALL 001BH               ; D4H: close
        POP AF
        OR A
        JR NZ,ERROR

FILE_NAME:
        DB 3,"LS2"
```

Files copied into a VT-DOS DSK retain the complete 144-byte host CAS container.
The disk device consumes that outer container and exposes the 16-byte TVC
application header first. In the Laser Squad probe, a 16-byte block read to
`19DFH` followed by a 30-byte block read to `5E28H` reproduced the assembled
payload exactly and transferred control to its entry point.
