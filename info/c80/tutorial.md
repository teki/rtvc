# C80 Tutorial

C80 is a small C-like language for Z80 programs in rtvc. It is not ISO C: no
heap, no recursion, no implicit promotions, and no `*` `/` `%`. This tutorial
walks through every implemented feature with sources in this directory. The
authoritative contract is the [C80 Language Reference](../c80.md).

Compile every example from this folder:

```bash
./compile.sh
```

On Windows:

```bat
compile.bat
```

The scripts look for `rtvc-c80` and `rtvc-tocas` on `PATH`, then `RTVC_C80` /
`RTVC_TOCAS`, then `cargo run -p rtvc-c80` from the repository root. Outputs go to `out/`
(gitignored): assembly and binary for the standalone sources, segment TOML
for projects, and sibling `.cas` files for the TVC mixed examples. A
successful compile is the check that an example is valid C80. Load a `.cas`
with Tape inject or `rtvc -i`; standalone `.bin` files are a separate load.

From the repo root you can also compile one file:

```bash
cargo run -p rtvc-c80 -- build info/c80/hello.c80 --origin 0x8000 --emit-asm /tmp/hello.asm
```

`--origin` is required when a unit emits bytes. At least one of `--emit-asm`,
`--emit-segments`, or `--emit-bin` is required. Optimization is on by default;
`--no-optimize` keeps baseline `JP` lowering. A lone `.c80` file uses its stem
as the unit name, so the stem must be an ASCII identifier (`hello.c80`, not
`01-hello.c80`).

## 1. Hello: register leaves

[hello.c80](hello.c80) is two functions and no frame:

```c
u8 id(u8 x) { return x; }
u16 add(u16 a, u16 b) { return a + b; }
```

The default ABI puts the first word argument in `HL` and the second in `DE`. A
byte result returns in `A`; a word result in `HL`. `id` lowers to `RET`. `add`
lowers to `ADD HL,DE` then `RET`.

## 2. Scalars, wrapping, and casts

[scalars.c80](scalars.c80) covers `bool`, `u8`, `i8`, `u16`, `i16`,
`const`, `sizeof`, bitwise ops, and shifts.

- Storage is 1 byte for `bool`/`u8`/`i8` and 2 bytes for `u16`/`i16`.
- Mixed typed operands are errors (`u8 + u16`). Cast explicitly: `u16(x)`.
- `+` and `-` wrap at width. `return 255 + 1` in a `u8` function is `0`.
- Untyped literals take the assignment or return context if they fit.
  `-32768` is a valid `i16`. `bool b = 1` is not; use `bool(1)`.
- Locals have no implicit init. File-scope `const` needs a compile-time
  initializer and has no storage.

## 3. Control flow and globals

[control.c80](control.c80) uses `if`/`else`, `while`, `break`,
`continue`, `&&`/`||`, and a mutable global.

Conditions must be `bool`. Write `if (x != 0)`, not `if (x)`. `&&` and `||`
short-circuit: a skipped operand is not evaluated, so a store on that path
does not run.

Globals sit in source order with functions, not in a separate section. Each
load or store of a mutable global is emitted; the compiler does not fold
repeated reads or coalesce successive writes.

Every reachable exit of a non-void function needs `return` with a value. Void
functions may fall through. Direct and mutual recursion are errors.

## 4. Calls and `@fastcall`

[calls.c80](calls.c80) nests register calls. Word arguments take `HL`,
then `DE`, then `BC`. Byte and `bool` arguments then take `A`, `C`, `B`, `E`,
`D`, `L`, `H`, skipping halves already used by a word. A signature that does
not fit is an error suggesting `@stackcall`; the compiler does not invent
hidden stack arguments.

`@fastcall` is an optional spelling of the default register convention and may
sit before or after `pub`. Nested calls preserve live caller-saved values.
Tiny callees stay frame-free.

## 5. `@stackcall`

[stackcall.c80](stackcall.c80) uses the explicit stack ABI:
caller-cleaned, right-to-left, two-byte slots. The callee saves `IX` and
addresses parameters at `IX+4`. Locals and spills use negative displacements
that must fit a signed offset (at most 128 bytes of frame).

Call it from ordinary register-ABI functions; the compiler inserts the
argument pushes and the caller cleanup.

## 6. Arrays, pointers, and strings

[memory.c80](memory.c80) fills a global `u8 buf[8]`, walks it with
`ptr<u8>`, and reads a length-prefixed `str`.

- `ptr<T>` is a 16-bit address. `T` may be a scalar, a struct, or another
  pointer; not `void`, `str`, or an array.
- Arrays do not decay. Use `&buf[0]`, not `buf`. Local arrays and by-value
  aggregate arguments are rejected.
- Pointer ± integer scales by `sizeof(T)` with shift-and-add (there is still
  no source `*`). Equality is allowed; ordered pointer compares are not.
- `str` globals own a length byte plus payload (0..255). `s.len` is `u8`.
  `s[i]` reads payload at `ref+1+i`. `sizeof(str)` is 2 (the reference).
  Owned string payloads are immutable. Passing `"xy"` to a `str` parameter
  places an anonymous literal after the caller.

## 7. Packed structs, `for`, `do`, and compound assignment

[structs.c80](structs.c80) defines a 5-byte `Sprite` and walks an array
with `ptr<Sprite>`.

Structs are packed in declaration order with no padding. Fields may be
scalars, pointers, nested structs, or fixed arrays of those. `str` fields and
empty structs are rejected. `.` and `->` are ordinary lvalues.

`for (init; cond; update)`: an empty condition is true; `continue` runs
`update` then the condition. `do { } while (cond);` always runs the body once.

Compound assignment (`+= -= &= |= ^= <<= >>=`) evaluates the destination
address once. Pointers allow only `+=`/`-=`, stepping one element. Prefix
`++`/`--` returns the new value; postfix returns the old. Indexing
`enemies[i]` in a tight register leaf can exhaust registers; a pointer walk
(`p += 1`) stays in registers.

## 8. Inline assembly

[inline_asm.c80](inline_asm.c80) increments a byte and captures carry from `ADD`.

`asm { }` without a header conservatively clobbers `AF`/`BC`/`DE`/`HL` and
memory. With a header, list `in`, `out`, `inout`, `clobber`, and optional
`stack:N`. Register names are lowercase. Flag outputs `carry` and `zero` are
`bool`.

The body is [helper assembler](../assembler.md) text, including `$` and `;`
comments. `RET`, `ORG`, `DB`/`DW`/`DS`, and jumps out of the block are
rejected. `CALL`/`RST` without `stack:N` warn and leave stack usage unknown.
`LDIR` with `BC=0` copies 65536 bytes; that is raw Z80, not a compiler skip.

## 9. CPU builtins

[ports.c80](ports.c80) uses `cpu::in(u8)`, `cpu::out(u8, u8)`, `cpu::di()`,
`cpu::ei()`, and `cpu::ldir(hl, de, bc)`. They are not ordinary functions: do
not import `cpu` or take them as values. Each `in`/`out` transfers one byte.
A TVC constant port becomes `IN A,(n)` / `OUT (n),A`; a variable port uses
`C`. `in:` / `out:` stay asm operand keywords.

`cpu::ldir` follows Z80 register order: HL is source, DE is dest, BC is
count. BC=0 copies 65536 bytes.

Pong's keyboard row read is `cpu::out(0x03, row); return cpu::in(0x58);`.
Setup uses `cpu::di()`, `cpu::ldir` to clear VID0, then `cpu::ei()` in
restore. It also writes CRTC `R14-R15 = 0x0EFF` because VT-DOS moves the
cursor IRQ to `0x0AFF` (raster 175). Sprites keep their `y` as the source of
truth: a move only stamps the new rows and clears the ones that fell off.

## 10. Optimization

[optimize.c80](optimize.c80) is compiled twice: default, then
`--no-optimize` to `out/optimize.unopt.asm`.

Default builds:

- drop identity `LD r,r` and jumps to the next label;
- turn `JP cc, then; JP else` into `JP !cc, else` when `then` is next;
- shorten forward conditional `JP` to `JR` when the displacement fits
  `-128..=127`.

Backward and unconditional jumps stay `JP` so a taken loop edge does not pay
JR’s extra T-states. Far branches stay `JP`. `DJNZ` is not used: `while (n !=
0)` with `n == 0` must iterate zero times.

`writes` still stores `1`, then `2`, then `3`. The optimizer does not invent
undefined behavior by CSE or store-forwarding through aliases.

## 11. Projects, imports, and assembly entry

[project/rtvc-c80.toml](project/rtvc-c80.toml) links three units:

| Unit | Kind | Role |
| --- | --- | --- |
| `sizes` | C80 constants | `pub const` values, no origin, no bytes |
| `math` | C80 | `import sizes;` and `project::stack_top` |
| `boot` | ASM | `CALL @{math::inc}` then `RET` |

Unqualified names stay in the current unit. `other::name` looks up **public**
symbols of an imported unit. `project` is a reserved builtin namespace:
`stack_base`, `stack_size`, `stack_top`, and `stack_end` when a `[stack]`
table is present. List a constants-only unit **before** units that import it
so `pub const` values are available during checking.

ASM units have no C80 prologue. `@{unit::export}` and `@{project::…}` are
replaced outside quotes and comments. `exports = ["start"]` marks the labels
other units may reference. `stack_extra` is a trusted extra-stack bound for
that ASM unit. Calling an ASM export from C80 is not a typed call.

Each emitting unit starts at its own `ORG`. `--origin` on the CLI is invalid
with a manifest. Segment output is `rtvc-asm-v1` (`--emit-segments`). Raw
`--emit-bin` requires a contiguous union of emitted ranges; the boot/math
split here uses assembly plus segments instead.

Optional `[[reserve]]` entries occupy no bytes. Ranges may touch but must not
overlap C80/ASM output, `[stack]`, or `[basic]`.

## 12. Mixed BASIC and C80

[mixed/](mixed/) is a small TVC program: BASIC drives C80 through `USR`,
`PEEK`, and `POKE`. Layout is TVC-only: BASIC at `19EFH`, C80 at `3000H`.
`[basic]` is only a path; the linker errors if the tokenized payload overlaps
a module. `[basic]` is rejected on other targets.

[mixed/buf.c80](mixed/buf.c80) exports a public `cells[16]` buffer plus
`fill`, `sum`, and `bump`. Each entry is `@fastcall pub i16` so the argument
and result travel in `HL`, matching BASIC `USR`.
[mixed/main.bas](mixed/main.bas) substitutes addresses after assembly:

- `USR(@{buf::fill},7)` fills the buffer with 7 and prints `PEEK` of the
  first and last bytes.
- `USR(@{buf::sum},0)` returns the 16-bit total (16×7 = 112).
- `POKE @{buf::cells},1` then `sum` again (106).
- `@{buf::COUNT}` is a `pub const`, so BASIC sees the integer `16`, not an
  address.
- `PRINT "@{buf::fill}"` and the `REM` / `DATA` field keep the marker text;
  an unquoted `DATA` colon starts a new statement, so `USR(@{buf::bump},0)`
  on that line is still substituted.

Tokenized BASIC is part of the same `rtvc-asm-v1` image (`--emit-segments`).
`--emit-asm` appends `ORG`/`DB` so a reassembly of that listing includes the
BASIC bytes. `--emit-bin` still requires a contiguous union; these examples
leave a gap from the short BASIC payload to C80 at `3000H`, so they emit
segments plus CAS instead of a raw binary. `rtvc-tocas` flattens the TOML
from `19EFH` (zero-filled gaps, exclusive end at most `C000H`). The compiler
does not poke interpreter state or wrap USR.

`compile.sh` writes `out/mixed.cas`. Inject it into a booted TVC 1.2 (File
inject, or `rtvc -- snapshots/boot12dos.rtvcsnap.zip -i info/c80/out/mixed.cas`).
`rtvc-tocas` sets CAS autostart, so `-i` types `RUN`. Cold-boot inject can race
BASIC init; use the snapshot. BASIC cannot use the literal `-32768`; `-32767`
is the negative boundary.

C80 may read `project::basic_base`. It cannot depend on `basic_program_size`,
`basic_size`, `basic_end`, or `basic_himem` (those exist only after
tokenization).

[tvc-usr/](tvc-usr/) is the smaller ABI check: `echo` round-trips `42` and
`-32767` through `USR` with no buffer.

## 13. Pong: BASIC starts, C80 plays

[pong/](pong/) is a complete (chunky) game. BASIC is only the launch:

```basic
10 LET R=USR(@{pong::play},0)
20 PRINT R
```

`play` does not return until someone reaches `WIN` (5). It maps video RAM,
disables interrupts so the ROM cursor does not stamp the bitmap, programs the
CRTC cursor to the last visible byte (`0x0EFF`) so port `59H` bit 4 ticks at
50 Hz (VT-DOS otherwise leaves that IRQ on raster 175), then restores map
`70H` and `EI`. `PRINT` is `1` if you won and `-1` if the AI won.

Controls: TVC joystick up/down (keyboard row 8, the same matrix as the cursor
keys) moves the left paddle. Space serves. The right paddle tracks the ball a
little slower than the player.

C80 cannot shift yet, so the playfield is 4-colour **bytes**: a paddle is one
byte wide (four pixels) and the ball is one byte by four rasters. Vertical
motion writes only the new edge and clears the old one (two rows for the
player, one for the AI, one for the ball). A horizontal ball step erases the
old four bytes and stamps the new column.

Same cassette layout as the mixed example: BASIC at `19EFH`, C80 at
`3000H`. `--emit-segments` writes `out/pong.toml`; `rtvc-tocas` writes
`out/pong.cas`. Rebuild that file after editing `pong.c80` (`./compile.sh`).
From the repo root:

```bash
cargo run --bin rtvc -- snapshots/boot12dos.rtvcsnap.zip -i info/c80/out/pong.cas
```

Autostart inject types `RUN`. Click the Screen pane so Space and the joystick
keys reach the TVC.

## What C80 does not do

These are rejected with a located diagnostic, not silently ignored:

- `*` `/` `%` and `*=` `/=` `%=`
- `<<` `>>` and `<<=` `>>=` (parsed and typed, not yet lowered to Z80)
- local arrays, by-value structs as arguments/returns, whole-array assignment
- recursion, a heap, implicit promotions, ROM helper wrappers
- `DJNZ` lowering of counted loops

Listing maps, instruction timing, and stack provenance are available in
process through `CompilationResult::map()`. There is no `--emit-map` flag.

The compiler is the `rtvc-c80` crate (`cargo run -p rtvc-c80`). It is not part
of native desktop, `wasm-full`, or lightweight `wasm` builds of `rtvc`.
There is no editor in this milestone.

## Files

| Path | Topic |
| --- | --- |
| [hello.c80](hello.c80) | Identity and word add |
| [scalars.c80](scalars.c80) | Types, casts, wrapping, `sizeof` |
| [control.c80](control.c80) | `if`/`while`/`&&`/`||`, globals |
| [calls.c80](calls.c80) | Register ABI and `@fastcall` |
| [stackcall.c80](stackcall.c80) | `@stackcall` |
| [memory.c80](memory.c80) | Arrays, pointers, `str` |
| [structs.c80](structs.c80) | Packed structs, `for`/`do`, `++` |
| [inline_asm.c80](inline_asm.c80) | Inline `asm` operands |
| [ports.c80](ports.c80) | `cpu::in` / `cpu::out` / `cpu::di` / `cpu::ei` / `cpu::ldir` |
| [optimize.c80](optimize.c80) | Default vs `--no-optimize` |
| [project/](project/) | Imports, `pub const`, ASM entry, `[stack]` |
| [mixed/](mixed/) | BASIC `USR`/`PEEK`/`POKE` a C80 buffer |
| [tvc-usr/](tvc-usr/) | Minimal `USR` echo ABI |
| [pong/](pong/) | BASIC `USR` into a C80 game loop |
| [compile.sh](compile.sh) / [compile.bat](compile.bat) | Build all of the above |
