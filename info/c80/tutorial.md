# C80 Tutorial

> **Work in progress:** C80 is experimental. The language, ABI, and output
> formats may change without notice, and it has not yet been used for real
> programs beyond its own tests and tutorial examples.

C80 is a small C-like language for Z80 programs in rtvc. It is not ISO C: no
heap, no recursion, no implicit promotions, and no `*` `/` `%`. This tutorial
walks through every implemented feature with sources in this directory, one
section per example. Each section links the source and its golden listing
in [expected/](expected/), the code the compiler produces today. The
authoritative contract is the [C80 Language Reference](../c80.md). The C80
code blocks on this page are compiled by the documentation test.

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
for projects, and sibling `.cas` files for the TVC mixed examples. Load a
`.cas` with Tape inject or `rtvc -i`; standalone `.bin` files are a separate
load. The examples are also tests: see
[Examples check themselves](#examples-check-themselves).

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

Source: [hello.c80](hello.c80). Listing: [expected/hello.asm](expected/hello.asm).

[hello.c80](hello.c80) is two functions and no frame:

```c80
u8 id(u8 x) { return x; }
u16 add(u16 a, u16 b) { return a + b; }
```

The default ABI puts the first word argument in `HL` and the second in `DE`. A
byte result returns in `A`; a word result in `HL`. `id` lowers to `RET`. `add`
lowers to `ADD HL,DE` then `RET`.

## 2. Scalars, wrapping, and casts

Source: [scalars.c80](scalars.c80). Listing: [expected/scalars.asm](expected/scalars.asm). Rejected forms: [scalars.fail.c80](scalars.fail.c80).

[scalars.c80](scalars.c80) covers `bool`, `u8`, `i8`, `u16`, `i16`,
`const`, `sizeof`, casts, and bitwise `&`. Shifts have their own section
next.

- Storage is 1 byte for `bool`/`u8`/`i8` and 2 bytes for `u16`/`i16`.
- Mixed typed operands are errors (`u8 + u16`). Cast explicitly: `u16(x)`.
- `+` and `-` wrap at width. `return 255 + 1` in a `u8` function is `0`.
- Untyped literals take the assignment or return context if they fit.
  `-32768` is a valid `i16`. `bool b = 1` is not; use `bool(1)`.
- Locals have no implicit init. File-scope `const` needs a compile-time
  initializer and has no storage.

## 3. Shifts

Source: [shifts.c80](shifts.c80). Listing: [expected/shifts.asm](expected/shifts.asm). Rejected forms: [shifts.fail.c80](shifts.fail.c80).

[shifts.c80](shifts.c80) shifts all four integer types by constant and
variable counts, compares signed and unsigned `>>`, and shifts a pinned
register in place.

- The count is always a `u8`. An untyped literal count is a `u8`; any other
  typed count is an error ("shift count must be u8, found u16"). The result
  has the left operand's type, so `u16 << u8` needs no cast.
- `<<` drops the bits that leave the type. `>>` fills with zeros on
  `u8`/`u16` and copies the sign bit on `i8`/`i16`.
- A count at or above the width is defined, not masked: the result is 0,
  or all sign bits (`-1`) for a negative signed value. `u8 x >> 9` is 0,
  never `x >> 1`.
- `+` binds tighter than `<<`, as in C: `a << b + 1` is `a << (b + 1)`.

Constant counts get the fastest sequence the compiler knows (T-states
first, then bytes). One step is `ADD A,A` (4 T) or `ADD HL,HL` (11 T) to the
left, `SRL`/`SRA` (8 T) to the right. Larger counts rotate and mask
(`u8 << 6` is `RRCA / RRCA / AND 192`, 15 T instead of 24 T), move bytes
(`u16 << 8` is `LD H,L / LD L,0`, 11 T), or shift the other way and move
bytes (`u16 >> 7` is 27 T instead of 112 T). `i8 >> 7` and `i16 >> 15` are
the sign, two or five instructions. The listings in the example comments
are pinned by `expect-tstates` lines.

A variable count runs a loop: the count goes to `B`, a zero count skips the
loop, and `DJNZ` repeats one step. A count of 200 really runs 200 steps (the
value stops changing after the width, so the result is still right). Keep
counts small in hot code, or use a constant.

`x <<= 1` on a pinned `x @ b` is `SLA B`: a shift whose result goes back to
the same register runs there instead of through `A`. Shifts work on any
place: globals, `static` locals, struct fields, and through pointers.

Compiler-made address arithmetic uses the same shifts: `words[i]` doubles
the index with one `ADD HL,HL`, and `enemies[i]` with a 5-byte element is
`(i << 2) + i`.

## 4. Control flow and globals

Source: [control.c80](control.c80). Listing: [expected/control.asm](expected/control.asm). Rejected forms: [control.fail.c80](control.fail.c80).

[control.c80](control.c80) uses `if`/`else`, `while`, `break`,
`continue`, `&&`/`||`, and a mutable global.

Conditions must be `bool`. Write `if (x != 0)`, not `if (x)`. `&&` and `||`
short-circuit: a skipped operand is not evaluated, so a store on that path
does not run. In a condition, `&&`, `||`, `!` and comparisons compile to
jumps: `if (a < 10 && b != 0)` is `CP 10 / JR NC,...` followed by the test of
`b`, with no 0/1 built. A comparison or `&&` used as a value
(`bool t = a < b;`) is built in `A` as 1 or 0.

Globals sit in source order with functions, not in a separate section. Each
load or store of a mutable global is emitted; the compiler does not fold
repeated reads or coalesce successive writes.

Every reachable exit of a non-void function needs `return` with a value. Void
functions may fall through. Direct and mutual recursion are errors.

## 5. Calls and `@fastcall`

Source: [calls.c80](calls.c80). Listing: [expected/calls.asm](expected/calls.asm). Rejected forms: [calls.fail.c80](calls.fail.c80).

[calls.c80](calls.c80) nests register calls. Word arguments take `HL`,
then `DE`, then `BC`. Byte and `bool` arguments then take `A`, `C`, `B`, `E`,
`D`, `L`, `H`, skipping halves already used by a word. A signature that does
not fit is an error suggesting `@stackcall`; the compiler does not invent
hidden stack arguments.

`@fastcall` uses the same register convention and may sit before or after
`pub`; it also requires the body to fit in registers (see
[Taking control of registers](#13-taking-control-of-registers)). Nested
calls preserve live caller-saved values:
`nest` keeps `x` in `C` across `id(1)` with a single `LD C,A`, because `id`
is private and never writes `C` (see
[Register contracts](#14-register-contracts) and
[When values do not fit](#12-when-values-do-not-fit)). Callees stay
frame-free.

## 6. `@stackcall`

Source: [stackcall.c80](stackcall.c80). Listing: [expected/stackcall.asm](expected/stackcall.asm).

[stackcall.c80](stackcall.c80) uses the explicit stack ABI:
caller-cleaned, right-to-left, two-byte slots. The callee saves `IX` and
addresses parameters at `IX+4`; the last parameter must sit within `IX+127`
(at most 62 word parameters). `IX` holds parameters only: locals use
registers and static slots like in any other function.

Call it from ordinary register-ABI functions; the compiler inserts the
argument pushes and the caller cleanup.

## 7. Arrays, pointers, and strings

Source: [memory.c80](memory.c80). Listing: [expected/memory.asm](expected/memory.asm). Rejected forms: [memory.fail.c80](memory.fail.c80).

[memory.c80](memory.c80) fills a global `u8 buf[8]`, walks it with
`ptr<u8>`, and reads a length-prefixed `str`. Its last globals are a static
pointer `ptr<u8> mid = &buf[4];` and a small address table; see
[Static tables](#9-static-tables-addresses-in-initializers).

- `ptr<T>` is a 16-bit address. `T` may be a scalar, a struct, or another
  pointer; not `void`, `str`, or an array.
- Arrays do not decay. Use `&buf[0]`, not `buf`. By-value aggregate
  arguments are rejected, and a local array must be `static` (see
  [Taking control of registers](#13-taking-control-of-registers)).
- Pointer ± integer scales by `sizeof(T)` with shift-and-add (there is still
  no source `*`). Equality is allowed; ordered pointer compares are not.
- `str` globals own a length byte plus payload (0..255). `s.len` is `u8`.
  `s[i]` reads payload at `ref+1+i`. `sizeof(str)` is 2 (the reference).
  Owned string payloads are immutable. Passing `"xy"` to a `str` parameter
  places an anonymous literal after the caller.

## 8. Packed structs, `for`, `do`, and compound assignment

Source: [structs.c80](structs.c80). Listing: [expected/structs.asm](expected/structs.asm).

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
`enemies[i].x` by a variable works (`nudge`); it recomputes
`enemies + 5 * i` on each access and spills in a small leaf, so a pointer
walk (`p += 1`, `step`) is the faster loop.

## 9. Static tables: addresses in initializers

Source: [tables.c80](tables.c80). Listing: [expected/tables.asm](expected/tables.asm). Rejected forms: [tables.fail.c80](tables.fail.c80).

[tables.c80](tables.c80) builds a linked list, a sprite table, and a few
pointers into arrays entirely in the image, with no start-up code.

A global or `static` local initializer may hold an address whose value is
fixed when the program is linked:

| Form | Example | Listing |
| --- | --- | --- |
| `&global` | `Node first = { 1, &second };` | `DW G13_SECOND` |
| `&array[K]` | `{ 30, 40, &frames[4] }` | `DW G60_FRAMES+4` |
| `&global.field` | `ptr<u16> score = &cfg.score;` | `DW G135_CFG+1` |
| `&array[K].field` | `&sprites[1].frame` | `DW G84_SPRITES+6` |
| `&unit::global` | `ptr<u8> p = &lib::tab[2];` | label from the other unit |

`K` is a constant inside the array. The result is typed exactly like the
same `&` in code, so `ptr<T>(...)` reinterprets it and `u16(...)` stores it
in a `u16` (a plain `u16 x = &g;` is a type error, as in code). Names may be
declared later in the file (`first` names `second`), and a struct may point
at itself (`Node ring = { 9, &ring };`). A `static` local may take the
address of another `static` in scope or of a global.

Arrays hold numbers and structs, not pointers, so a table of addresses is
either an array of structs with a pointer field (`sprites`) or a `u16` array
filled with `u16(&...)` and read back with `ptr<T>(...)` (`rows`).

Rejected with a located diagnostic ([tables.fail.c80](tables.fail.c80)):
`&array` without an index (use `&array[0]`), the address of a `const` or of
a `str` payload, a constant index outside the array, indexing a pointer
(its value is known only at run time), an automatic or pinned local in a
`static` initializer, a `const` holding an address, and any arithmetic on
an address (`&grid[1] + 1`; write `&grid[2]`).

## 10. Inline assembly

Source: [inline_asm.c80](inline_asm.c80). Listing: [expected/inline_asm.asm](expected/inline_asm.asm). Rejected forms: [inline_asm.fail.c80](inline_asm.fail.c80).

[inline_asm.c80](inline_asm.c80) increments a byte and captures carry from `ADD`.

`asm { }` without a header conservatively clobbers `AF`/`BC`/`DE`/`HL` and
memory. With a header, list `in`, `out`, `inout`, `clobber`, and optional
`stack:N`. Register names are lowercase. Flag outputs `carry` and `zero` are
`bool`.

The body is [helper assembler](../assembler.md) text, including `$` and `;`
comments. `RET`, `ORG`, `DB`/`DW`/`DS`, and jumps out of the block are
rejected. `CALL`/`RST` without `stack:N` warn and leave stack usage unknown.
`LDIR` with `BC=0` copies 65536 bytes; that is raw Z80, not a compiler skip.

## 11. CPU builtins

Source: [ports.c80](ports.c80). Listing: [expected/ports.asm](expected/ports.asm).

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

## 12. When values do not fit

Source: [spills.c80](spills.c80). Listing: [expected/spills.asm](expected/spills.asm).

[spills.c80](spills.c80) shows what the compiler does when values do not fit
in registers. Values stay in registers whenever they can; a fallback is used
only where a value must survive a call (or `cpu::ldir`, or an `asm` block
that clobbers its register), or where more values are live than registers.
It picks the cheapest option in T-states, counting loop bodies as hot:

| Fallback | Save + restore |
| --- | --- |
| Move to a register the operation leaves alone | 8 T |
| `PUSH rr` / `POP rr` around the call | 21 T |
| Byte in a static slot, `LD (nn),A` / `LD A,(nn)` | 26 T |
| Word in a static slot through `HL` | 32 T |
| Word in a static slot through `BC`/`DE` (ED forms) | 40 T |

A static slot is a byte in the function's slot area, placed right after its
`RET` as `NAME_S: DB 0,...`. Slots are reused once their value dies, so the
area is only as large as the peak need. It is part of the function's size,
zero in the image, and covered by `--emit-bin` and the project linker like
any other function byte. Pushes count toward the proven stack bound; slots do
not use the stack. Because slots have fixed addresses, functions are not
reentrant (C80 has no recursion). No function uses `IX` except to reach
`@stackcall` parameters.

Reading the listing:

- every fallback instruction ends in `; spill`;
- a function with fallback code ends with
  `; spill report: N slot bytes, saves (bytes per site) ..., T T-states of spill code`,
  where the T-states are a static sum of the `; spill` lines, each counted
  once;
- in `CompilationResult::map()`, `; spill` instructions and the slot `DB`
  are `synthetic`, and `GeneratedFunction::spill` has the same numbers.

`keep` moves `y` from `C` to `B` before `CALL twice` (4 T), because `twice` is
private and never writes `B`; a `pub` callee would make it `PUSH BC`/`POP BC`
instead. `mix`
has nine bytes of live values in a loop: the hottest stay in `BC`/`DE`, `HL`
stays free for the arithmetic, and the rest live in five slot bytes.
`via_ptr` takes `&cell`, so `cell` lives in a slot for the whole function and
`&cell` is `LD HL,NAME_S`.

## 13. Taking control of registers

Source: [registers.c80](registers.c80). Listing: [expected/registers.asm](expected/registers.asm). Rejected forms: [registers.fail.c80](registers.fail.c80).

[registers.c80](registers.c80) shows three ways to state register use in the
source instead of leaving it to the compiler. Each one turns a silent cost
into a compile error you can see and fix. None changes the calling
convention, so callers cannot tell the difference.

**Strict `@fastcall`.** The body must fit in registers: no `; spill` line, no
`PUSH`/`POP` save, no `IX`. `CALL` and `RET` are the only stack use. Anything
else is a located `cg-unsupported` error that names the value and the
operation:

```text
@fastcall 'sum_strict' must fit in registers, but 'n' has no free register
at the cast u8 to u16; pin values to free registers, declare a local
'static', or remove @fastcall
```

A value that is still in a register after a call is allowed in a `@fastcall`
function only if the callee leaves that register alone: see
[Register contracts](#14-register-contracts). Moving it into such a
register first (`LD B,C`) is allowed; saving it with `PUSH`/`POP` is not.

**Pins.** `u8 n @ b` keeps `n` in `B` for its whole scope, which is the
function for a parameter and the enclosing block for a local, starting at its
declaration. Bytes pin to `a b c d e h l`; words and pointers to `bc de hl`.
The compiler never moves, spills, or shares a pinned register:

- A pinned parameter whose ABI register differs costs one entry move (`n`
  arrives in `C`, so `times` starts with `LD B,C`: 1 byte, 4 T).
- A statement that stores into the pin, such as `s = s + a`, may compute the
  new value in the pin's register, because the old value is dead. That is
  how `scale` runs `ADD HL,DE` in place with `s @ hl`.
- An operation that needs the pinned register for something else is an
  error, never a hidden move:

  ```text
  's' is pinned to hl, but the u16 `+` needs hl for something else; pin 's'
  to another register, or remove the pin
  ```

  Pin to the register the instruction leaves its result in, or to one it
  does not use. A byte load or store through a pointer uses `LD A,(HL)`, or
  `LD A,(BC)` / `LD A,(DE)` when the pointer already lives in that pair
  (word accesses always go through `HL`), so a pointer you dereference is
  best left unpinned or pinned to `bc`/`de`.
- Across a call, a default function saves a pinned value with a marked
  `PUSH`/`POP`; a `@fastcall` function reports an error.
- Overlapping pins in the same scope, `&` of a pinned value, and a width
  mismatch (`u16 w @ a`) are errors. Pins in disjoint blocks may share a
  register.

**`static` locals.** `static u8 count = 0;` is stored right after the
function and keeps its value between calls. The initializer must be a
compile-time value (it is stored in the image, not run on each call); without
one the static is zero. A static is always initialized, can have its address
taken, and every access is a real load or store. Local arrays must be
`static`: `static u8 table[4] = {0, 1, 4, 9};`. Statics appear among the
globals as `function.name`. They make code non-reentrant, like static slots.

`static` is a keyword. It is only accepted inside a function body: not at
file scope, not in a `for` initializer, not with `const`, and not with a pin.

The rejected forms and their messages are in
[registers.fail.c80](registers.fail.c80) (declaration errors) and
[registers_cg.fail.c80](registers_cg.fail.c80) (pin conflicts and strict
`@fastcall`).

## 14. Register contracts

Source: [contracts.c80](contracts.c80). Listing: [expected/contracts.asm](expected/contracts.asm). Rejected forms: [contracts.fail.c80](contracts.fail.c80).

[contracts.c80](contracts.c80) is about what a `CALL` leaves alone. Without
more information a call may change every register, so a value that must
survive it is moved or pushed. A function's **contract** lists the registers
it returns unchanged, and callers keep values there with no save at all.

**Declared: `@preserves(bc)`.** An attribute beside `@fastcall`/`@stackcall`,
with the register names of pins. Pairs and halves mean the same (`b, c` is
`bc`). The listed registers come back with their entry values, including a
parameter that arrives in one (it may be read, not changed). The compiler
checks the body:

- it never picks a preserved register for a value or a scratch;
- an operation that needs one is an error at that statement
  (`'acc' declares @preserves(hl), but the u16 `+` needs hl; ...`);
- a call or `asm` block that may change one is wrapped in a marked
  `PUSH`/`POP` in a default function, and is an error in a `@fastcall` one;
- a result register (`a` for a byte result, `hl` for a word) cannot be
  listed, and a pin cannot use a preserved register.

The contract is part of the signature: callers in other units rely on it,
and it appears in the listing as `; preserves B C (declared)` and in
`GeneratedFunction::preserves`/`contract`.

**Inferred, for private functions.** A function without `pub` is only called
from its own file, so callers there use the registers its final code never
writes (a call counts its callee's contract; an `asm` block its declared
inputs, outputs, and clobbers). The listing shows these as
`; preserves ... (inferred)` on functions that are called. A `pub` function
without `@preserves` promises nothing, so editing its body cannot change
code in another unit. In `contracts.c80`, `after_private` needs no save
around the private `bump`, while `after_pub` pushes `BC` around the `pub`
`twice_pub`.

The test harness checks every contract at run time: each call into a
function with a contract must return with those registers unchanged.

Violations are in [contracts.fail.c80](contracts.fail.c80) (declarations)
and [contracts_cg.fail.c80](contracts_cg.fail.c80) (bodies and strict
callers).

## 15. Inlining

Source: [inline.c80](inline.c80). Listing: [expected/inline.asm](expected/inline.asm). Rejected forms: [inline.fail.c80](inline.fail.c80).

[inline.c80](inline.c80) shows the same helper as a call and inlined. A
macro has no `CALL`/`RET`; `@inline` is how a small hot routine costs what a
macro would. Every C80 call to an `@inline` function is expanded in place,
with and without `--no-optimize`, and the expansion keeps a call's meaning:
arguments are evaluated once, left to right, a `return` continues after the
call, and a `static` local is still one location. The listing marks each
expansion with `; inline NAME`; its bytes belong to the caller, and the
source map points them at the helper's lines.

```c80
@inline pub u8 col(u8 x) {
    return x >> 2;
}

u8 cols(u8 a, u8 b) {
    return col(a) + col(b);
}
```

The listing of `cols` (from [expected/inline.asm](expected/inline.asm)) has
no `CALL`; the first copy works in `A`, the second in `C`:

```asm golden=inline.asm
        ; inline col
        RRCA
        RRCA
        AND 63
        ; inline col
        SRL C
        SRL C
        ADD A,C
        RET
```

`cols` is 45 T and 10 bytes; the same body behind `@noinline` costs 114 T
and 12 bytes in the caller plus the 5-byte helper. A constant argument
becomes an immediate (`col(40)` is `ADD A,10`), and in [inline.c80](inline.c80)
`s = inc(s)` in a loop is one `INC C`.

- `@inline` combines with `@fastcall` and `@preserves`; `@inline
  @stackcall` is an error ([inline.fail.c80](inline.fail.c80)).
- A private `@inline` function has no out-of-line copy. `pub @inline` keeps
  one, so BASIC markers and ASM units can still call it.
- The expansion is the caller's code: the caller's strictness and
  `@preserves` apply to it, and a message from inside it names the helper
  and the call ("(in 'g', inlined into 'f')"). A pin in the helper applies
  inside the expansion; one that collides with a pin of the caller is an
  error at the call ([inline_cg.fail.c80](inline_cg.fail.c80)).

Optimized builds also inline on their own (`; inline NAME (auto)`): a call
to a private function is expanded when a trial lowering shows no new spill
and the caller does not grow, or when it is the function's only call. A
private function left with no call has no copy of its own. `@noinline`
keeps a function out of this; [calls.c80](calls.c80) and
[contracts.c80](contracts.c80) use it because their point is the `CALL`.
In [pong/](pong/), `put_row` and `read_row` are `@inline`, and the
functions `play` calls once are expanded into it automatically.

## 16. Optimization

Source: [optimize.c80](optimize.c80). Listing: [expected/optimize.asm](expected/optimize.asm). Baseline: [expected/optimize.unopt.asm](expected/optimize.unopt.asm).

[optimize.c80](optimize.c80) is compiled twice: default, then
`--no-optimize` to `out/optimize.unopt.asm`.

Both modes lower a condition straight to a branch (section 4), keep a
byte pointer that already sits in `BC` or `DE` there (`LD A,(BC)`, `LD
(DE),A`, `INC DE`), and write `x = x + 1` on a byte register as `INC r`
(`x - 1` as `DEC r`).

Default builds also:

- fold literal-only arithmetic (`(1 << 5) - 1` is `LD A,31`; `255 + 1` in
  a `u8` wraps to 0 the same way the machine would);
- inline small private functions (see [Inlining](#15-inlining));
- use the cheaper form when the flags it changes are not read afterwards:
  `OR A` for `CP 0`, `INC A` / `DEC A` for `ADD A,1` / `SUB 1`, `XOR A`
  for `LD A,0`;
- drop identity `LD r,r`, a register load whose value is never read, and a
  reload of a value a register already holds (constants, addresses, and
  the compiler's own slots; never a global, a static, a pointer target or
  a port);
- send a `JP` that lands on another `JP` straight to the final target,
  drop code nothing can reach, drop jumps to the next label, and turn `JP
  cc, then; JP else` into `JP !cc, else` when `then` is next;
- rotate a `while` loop: the entry jumps to the test at the bottom, and
  each pass ends with one conditional branch back to the body;
- close a loop whose counter is in `B` with `DJNZ`. The source's own `n !=
  0` guard runs once before the first pass, so a zero count still skips
  the loop (`blink`);
- turn `CALL f` directly followed by `RET` into `JP f` (`again`); the
  reported stack bound (single-file builds) drops by the 2 bytes of return
  address;
- shorten forward conditional `JP` to `JR` when the displacement fits
  `-128..=127`.

Backward and unconditional jumps stay `JP` so a taken loop edge does not pay
JR’s extra T-states. Far branches stay `JP`. A rotated loop that runs zero
times costs 10 T more than before rotation (one pass: 3 T more); every
further pass saves 7 T.

`writes` still stores `1`, then `2`, then `3`. The optimizer does not invent
undefined behavior by CSE or store-forwarding through aliases.

## 17. Projects, imports, and assembly entry

Source: [project/rtvc-c80.toml](project/rtvc-c80.toml). Listing: [expected/project.asm](expected/project.asm).

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

## 18. Mixed BASIC and C80

Source: [mixed/rtvc-c80.toml](mixed/rtvc-c80.toml). Listing: [expected/mixed.asm](expected/mixed.asm).

[mixed/](mixed/) is a small TVC program: BASIC drives C80 through `USR`,
`PEEK`, and `POKE`. Layout is TVC-only: BASIC at `19EFH`, C80 at `3000H`.
`[basic]` is only a path; the linker errors if the tokenized payload overlaps
a module. `[basic]` is rejected on other targets.

[mixed/buf.c80](mixed/buf.c80) exports a public `cells[16]` buffer plus
`fill`, `sum`, and `bump`. Each entry is `pub i16` with one `i16`
parameter, so the argument and result travel in `HL`, matching BASIC `USR`.
`fill` and `bump` are also `@fastcall`; `sum` is not, because its loop keeps
one value in a static slot, which strict `@fastcall` rejects.
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

## 19. The `USR` ABI in one function

Source: [tvc-usr/echo.c80](tvc-usr/echo.c80). Listing:
[expected/tvc-usr.asm](expected/tvc-usr.asm).

[tvc-usr/](tvc-usr/) is the smallest mixed project: one C80 unit named
`math` (from [echo.c80](tvc-usr/echo.c80)) and a BASIC program. `echo` is
`@fastcall pub i16 echo(i16 value)`: BASIC `USR` passes its argument in `HL`
and reads the result from `HL`, so the body is a single `RET` (pinned by
`expect-bytes: echo == 1`). [tvc-usr/main.bas](tvc-usr/main.bas) prints
`USR(@{math::echo},42)` and `USR(@{math::echo},-32767)`, which come back
unchanged. `compile.sh` writes `out/tvc-usr.cas`.

## 20. Pong: BASIC starts, C80 plays

Source: [pong/pong.c80](pong/pong.c80) and [pong/irq.asm](pong/irq.asm). Listing: [expected/pong.asm](expected/pong.asm).

[pong/](pong/) is a complete (chunky) game. BASIC is only the launch:

```basic
10 LET R=USR(@{irq::start},0)
20 PRINT R
```

The game draws everything and scans the keyboard itself, so it takes over
the frame interrupt: no ROM code runs while it plays. `irq.asm` is the
`USR` entry. With interrupts off it saves the first three bytes of the IM 1
stub at `0038H`, writes `JP FRAME` there, and calls `pong::play`; when
`play` returns it puts the bytes back and `EI`s. `FRAME` is `OUT (07H),A`
(acknowledge) and `RET`. The handler and the hook are assembly because C80
has no interrupt entry and cannot take a function's address; ASM can call
C80, not the other way.

`play` does not return until someone reaches `WIN` (5). It maps video RAM and
programs the CRTC cursor to the last visible byte (`0x0EFF`) so port `59H`
bit 4 ticks at 50 Hz (VT-DOS otherwise leaves that IRQ on raster 175). The
game logic runs with interrupts disabled; `wait_frame` is `EI` / `HALT`, the
only window where one is taken. The request is latched, so a frame that ends
while the logic still runs wakes the next `HALT` at once, and `EI`'s one
instruction delay keeps it from landing between `EI` and `HALT`. `PRINT` is
`1` if you won and `-1` if the AI won.

Magic numbers are `const` (folded, no storage): state codes, sizes, columns,
ports. C80 folds `ROWS - PAD_H - P_STEP`, but `py + PAD_H - P_STEP` is
`(py + 24) - 2` at run time; write `py + (PAD_H - P_STEP)`.

Controls: TVC joystick up/down (keyboard row 8, the same matrix as the cursor
keys) moves the left paddle. Space serves. The right paddle tracks the ball a
little slower than the player.

Sprites are whole 4-colour **bytes** (no pixel masks): a paddle is one byte
wide (four pixels) and the ball is one byte by four rasters. `put_row` (one
byte of a raster) and `read_row` (one keyboard row) are `@inline`, so the
drawing loop in `fill_rows` has no `CALL`. Vertical
motion writes only the new edge and clears the old one (two rows for the
player, one for the AI, one for the ball). A horizontal ball step erases the
old four bytes and stamps the new column. The ball bounces or scores on the
column next to a paddle, so it never overlaps one, and each point adds a pip
in column 0 (you) or 63 (the AI), outside the ball's reach.

Same cassette layout as the mixed example: BASIC at `19EFH`, C80 at
`3000H`, and `irq.asm` just below at `2FC0H`. `--emit-segments` writes
`out/pong.toml`; `rtvc-tocas` writes `out/pong.cas`. Rebuild that file after editing `pong.c80` or `irq.asm` (`./compile.sh`).
From the repo root:

```bash
cargo run --bin rtvc -- snapshots/boot12dos.rtvcsnap.zip -i info/c80/out/pong.cas
```

Autostart inject types `RUN`. Click the Screen pane so Space and the joystick
keys reach the TVC.

## 21. Testing your C80 code on the machine

Source: [selfcheck/selfcheck.c80](selfcheck/selfcheck.c80). Listing: [expected/selfcheck.asm](expected/selfcheck.asm).

The harness behind `//! expect` runs one function on a bare Z80 with flat
memory. On the TVC the same code shares the machine with the ROM: the 50 Hz
interrupt fires in the middle of it, the stack is the one BASIC called
`USR` with, and the memory map is whatever BASIC left. [selfcheck/](selfcheck/)
shows how to check generated code there.

```basic
20 LET R=USR(@{selfcheck::run},@{selfcheck::PASSES})
30 PRINT "fails=";R;" passes=";PEEK(@{selfcheck::passes})
40 FOR I=0 TO 5
50 PRINT PEEK(@{selfcheck::failed}+I);
60 NEXT I
80 POKE @{selfcheck::basic_done},1
```

The C80 side is a battery of 44 numbered checks: operators on all four
integer types at their wrap points, signed and unsigned compares, constant
and variable shifts (including counts of 8 and 16 or more), and one check
each for a static-slot spill, a `PUSH`/`POP` save, strict `@fastcall` with a
pin, a `@preserves` callee, a `static` local, `@inline`, `@stackcall`,
nested calls, struct arrays, a linked list, `str`, and `cpu::ldir`. The list
is at the top of [selfcheck.c80](selfcheck/selfcheck.c80).

A few rules make such a battery useful:

- **Expected values are literals you worked out yourself.** A check that
  compares the compiler's answer with the compiler's answer proves nothing.
  The comment beside each check shows the arithmetic.
- **Inputs come from globals.** `mark(0, 0xFF + 1 == 0)` folds at compile
  time and tests nothing at run time; `mark(0, ua + ub == 0)` with
  `u8 ua = 0xFF` does not fold.
- **Each check has a fixed index.** `mark(i, good)` sets bit `i & 7` of
  `seen[i >> 3]`, and of `failed[i >> 3]` when `good` is false. A failing
  bit names the feature, and `seen` shows that the check ran at all.
- **Run long enough to be interrupted.** `run(PASSES)` repeats the battery
  40 times, about 25 frames, so the ROM interrupt lands inside generated
  code many times.
- **Keep the harness checks too.** The same file has `//! expect` lines, so
  `cargo test -p rtvc-c80 --test examples` covers it on the bare Z80.

`./compile.sh` writes `out/selfcheck.cas`. Loaded and `RUN`, it prints the number
of failed checks (0), the number of passes (40) and the six `failed` bytes
(all 0). The automated version is
[c80/tests/tvc.rs](../../c80/tests/tvc.rs):

```bash
cargo test -p rtvc-c80 --test tvc
```

It builds the project in process in both optimize modes, boots a TVC 1.2
from ROM, loads the cassette, types `RUN`, and reads `seen`, `failed`,
`checks` and `passes` from emulated memory. While `run` executes it counts
interrupts accepted inside generated code, checks that `USR` returns with
BASIC's stack where it was, and compares the lowest stack pointer reached
in generated code with the compiler's proven stack bound for `run`. The
same file plays Pong: it serves with Space, moves the paddle with the
joystick, and checks every byte of video RAM against the game's paddle and
ball positions.

## 22. When the compiler says no

Every rejected form gets a located diagnostic with a stable code; nothing
is silently miscompiled or skipped. The CLI prints the file, line, and
column, the source line, and an underline. These are real messages for the
rejected examples in this directory (the documentation test compiles each
block and compares the messages):

```c80 error=parse-unsupported,ty-mismatch
u8 times(u8 a, u8 b) {
    return a * b;
}
u16 mixed(u8 a, u16 b) {
    return a + b;
}
```

```text output
error parse-unsupported: '*' is not supported: C80 has no multiply, divide, or remainder; use shifts and adds
error ty-mismatch: expected u16, found u8
```

There is no `*`: multiply by a constant with shifts and adds
(`(a << 2) + a` is `a * 5`), and convert explicitly (`u16(a) + b`).

```c80 error=ty-mismatch,ty-use-before-assign
u8 truthy(u8 x) {
    if (x) { return 1; }
    return 0;
}
u8 maybe(bool c) {
    u8 s;
    if (c) { s = 1; }
    return s;
}
```

```text output
error ty-mismatch: expected bool, found u8
error ty-use-before-assign: 's' is used before assignment
```

Conditions are `bool` (`if (x != 0)`), and a local must be assigned on every
path before it is read. A local that is read several times is reported
once, with a count of the other uses.

```c80 error=ty-mismatch
u8 sum4(u8 x) {
    u8 tmp[4];
    tmp[0] = x;
    return tmp[0];
}
```

```text output
error ty-mismatch: local arrays and structs need storage; declare them 'static' (a fixed address after the function), or use a global or a pointer
```

A local array needs `static`: `static u8 tmp[4];`.

```c80 error=cg-unsupported
u16 total;
@preserves(hl) void acc(u16 a, u16 b) {
    total = a + b;
}
```

```text output
error cg-unsupported: 'acc' declares @preserves(hl), but the u16 `+` needs hl; remove hl from @preserves
```

Code-generation errors (`cg-*`) name the value, the register, and the
operation, and end with the fixes that apply. They are reported only for a
file without type errors.

All the rejected examples, one `*.fail.c80` file per topic, with the
message quoted above each function:

| File | Shows |
| --- | --- |
| [scalars.fail.c80](scalars.fail.c80) | `*`, mixed-type operands |
| [shifts.fail.c80](shifts.fail.c80) | a shift count that is not `u8` |
| [control.fail.c80](control.fail.c80) | a non-`bool` condition, use before assignment |
| [calls.fail.c80](calls.fail.c80) | recursion |
| [memory.fail.c80](memory.fail.c80) | pointer and array misuse |
| [tables.fail.c80](tables.fail.c80) | addresses in initializers |
| [inline_asm.fail.c80](inline_asm.fail.c80) | `RET` and jumps out of an `asm` block |
| [registers.fail.c80](registers.fail.c80) | pins and `static` declarations |
| [registers_cg.fail.c80](registers_cg.fail.c80) | pin conflicts and strict `@fastcall` |
| [contracts.fail.c80](contracts.fail.c80) | `@preserves` declarations |
| [contracts_cg.fail.c80](contracts_cg.fail.c80) | contract violations in bodies and strict callers |
| [inline.fail.c80](inline.fail.c80) | `@inline` attribute misuse |
| [inline_cg.fail.c80](inline_cg.fail.c80) | an inlined pin that collides at the call |

What C80 does not have at all (`*` `/` `%`, automatic local arrays,
recursion, by-value structs, arrays of pointers, calling ASM exports,
interrupt handlers, and more) is listed with each diagnostic code and its
usual fix under
[Limits and Diagnostics](../c80.md#limits-and-diagnostics) in the
reference.

## Examples check themselves

Every example is compiled and run by `cargo test -p rtvc-c80 --test examples`,
in both the default and the `--no-optimize` build. `//!` comment lines state
what a function does; anything after `//!` that is not one of the forms below
fails the test.

```text
//! expect: add(0x1234, 0x0101) == 0x1335
//! expect: hits(100) == true with by = 110
//! expect: start() == &buf[0]
//! expect-mem: fill(&buf[2], 3, 7) => buf[0..8] == 0, 0, 7, 7, 7, 0, 0, 0
//! expect-mem: write_device(0x60, 0x14) => out(0x60) == 0x14
//! expect-mem: read_row(8) => out(0x03) == 8 with in(0x58) = 0x7F
//! expect-bytes: add == 2
//! expect-bytes: add <= 2
//! expect-tstates: add == 21
//! expect-skip: play - loops until someone reaches WIN
```

- `expect: f(args) == v` calls `f` and compares the result (`A` for a byte,
  `HL` for a word). Arguments and values are decimal, `0x` hex, negative,
  `true`/`false`, or `&g` / `&g[k]` (the address of global `g` or its
  element `k`).
- `expect-mem: f(args) => place == v1, v2, ...` checks memory or port output
  after the call. A place is `g`, `g[i]` or `g[i..j]` (element indices, end
  exclusive; a struct array is indexed by byte), `[addr]` or `[a..b]` (raw
  bytes), or `out(p)` (every byte written to port `p`, in order).
- `with place = v, ...` sets memory before the call; several values are
  separated by spaces. `in(p) = v1 v2` scripts what `IN` from port `p` reads.
- `expect-bytes: f == N` or `<= N` and `expect-tstates: f == N` are budgets
  for the default optimized build. T-states include the final `RET` and are
  measured with all-zero arguments; `expect-tstates` is rejected on a
  function whose listing has a jump, `DJNZ`, conditional `RET`, block
  instruction, or a call of anything but a generated function whose own
  listing passes the same check.
- `expect-skip: f - reason` covers a function that cannot run in a test
  (such as Pong's `play`); the reason is required.
- Every function that returns a value needs at least one `expect` (or an
  `expect-skip`).

A `*.fail.c80` file shows code that is rejected on purpose. It holds only
`//! expect-error: code` lines, and the test checks that compiling it reports
exactly those diagnostic codes (as a multiset, in any order), each with a
source location.
`compile.sh` and `compile.bat` skip these files.

The optimized listing of every single-file example and every project is
kept in [expected/](expected/) (plus `optimize.unopt.asm`). The test fails
when a listing changes; after a deliberate code-generation change, rerun with
`C80_UPDATE_EXPECTED=1` and review the diff.

## Files

| Path | Topic |
| --- | --- |
| [hello.c80](hello.c80) | Identity and word add |
| [scalars.c80](scalars.c80) | Types, casts, wrapping, `sizeof` |
| [shifts.c80](shifts.c80) | `<<`, `>>`, `<<=`, `>>=`: counts, costs, pins |
| `*.fail.c80` | Rejected on purpose, with the diagnostic codes |
| [control.c80](control.c80) | `if`/`while`/`&&`/`||`, globals |
| [calls.c80](calls.c80) | Register ABI and `@fastcall` |
| [stackcall.c80](stackcall.c80) | `@stackcall` |
| [memory.c80](memory.c80) | Arrays, pointers, `str` |
| [structs.c80](structs.c80) | Packed structs, `for`/`do`, `++` |
| [tables.c80](tables.c80) | Addresses in static initializers: lists, sprite tables |
| [spills.c80](spills.c80) | `PUSH`/`POP` saves and static slots |
| [registers.c80](registers.c80) | Strict `@fastcall`, register pins, `static` locals |
| [contracts.c80](contracts.c80) | `@preserves` and inferred register contracts |
| [inline_asm.c80](inline_asm.c80) | Inline `asm` operands |
| [ports.c80](ports.c80) | `cpu::in` / `cpu::out` / `cpu::di` / `cpu::ei` / `cpu::ldir` |
| [inline.c80](inline.c80) | `@inline`, `@noinline`, automatic inlining |
| [optimize.c80](optimize.c80) | Default vs `--no-optimize` |
| [project/](project/) | Imports, `pub const`, ASM entry, `[stack]` |
| [mixed/](mixed/) | BASIC `USR`/`PEEK`/`POKE` a C80 buffer |
| [tvc-usr/](tvc-usr/) | Minimal `USR` echo ABI |
| [pong/](pong/) | BASIC `USR` into a C80 game loop |
| [selfcheck/](selfcheck/) | A check battery that runs under the TVC ROM |
| [expected/](expected/) | Golden optimized listings the tests compare against |
| [compile.sh](compile.sh) / [compile.bat](compile.bat) | Build all of the above |
