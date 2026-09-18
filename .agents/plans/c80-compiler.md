# C80 Compiler and Integrated Source View Plan

## Goal

Add a deliberately small C-like language for Z80 development, called C80,
together with editor integration that makes generated code visible
and understandable. A developer should be able to edit C80 source, see the
corresponding Z80 assembly, bytes, addresses, and static T-state information,
load a successful build into the active emulator, and debug it through a
bidirectional source-to-machine map.

The compiler is not an ISO C implementation. It is a transparent, Z80-aware
language whose small scope is a feature: source constructs should have
predictable generated code, inline assembly must remain available, and every
generated instruction must retain enough provenance for editor and debugger
integration.

This plan expands the high-level [Developer Workspace backlog](../../TODO.md#developer-workspace)
item. It records the design discussed in the
[shared compiler conversation](https://chatgpt.com/share/6a8aaaad-9a68-83ec-b521-998025b6e674),
but is self-contained so implementation does not depend on that external page
remaining available.

## Product Principles

1. Prefer a coherent small language over a partial implementation of ISO C.
2. Treat source provenance as a compiler output, not a later debugging add-on.
3. Keep the compiler core independent of egui and emulator scheduling.
4. Generate ordinary rtvc-compatible Z80 assembly and reuse the checked-in
   assembler rather than introducing a second instruction encoder.
5. Represent generated operations structurally inside the compiler even when
   the initial assembler boundary is rendered text.
6. Make generated code and its cost visible; predictable code is more valuable
   than an opaque optimizer in the first versions.
7. Keep compilation quick and deterministic enough for an idle-debounced live
   assembly view without disturbing 50 Hz emulation.
8. Preserve the simple emulator UI. Compiler and editor features live in the
   optional Developer Workspace.
9. Support both native and `wasm-full`. Do not pull editor/compiler UI into the
   lightweight WASM library targets.
10. C80 is a small step above assembly: minimize stack traffic, keep data near
    its defining code, and make hardware calls and interrupt management explicit.

## Delivery Boundary

Phase 1 is only the compiler: a reusable compiler library plus `rtvc-c80`
command-line tool. It accepts single-file or project input and emits assembly,
loadable segments, diagnostics, symbols, source maps, and static size/timing
metadata. It has no egui pane, live recompilation, emulator loading, breakpoint,
or source-stepping UI.

Editor needs are nevertheless designed into Phase 1 outputs. Source spans,
instruction provenance, stable per-compilation IDs, final addresses, bytes, and
timings must be real compiler results rather than reconstructed by Phase 2.

Current implementation planning focuses on the compiler. Phase 2 sections record
future consumers and architectural boundaries, not additional prerequisites for
the compiler release. Do not expand editor or debugger design during Phase 1.

The Phase 1 workflow is:

1. Write one `.c80` source file or a TOML project containing several units.
2. Run `rtvc-c80` with a target and origin/project configuration.
3. Receive diagnostics or generated assembly and loadable segment output.
4. Optionally inspect emitted symbols, source-map metadata, bytes, and static
   instruction timings.
5. Load the result using existing rtvc assembler/debugger tooling when desired;
   the compiler itself does not control the emulator.

## Worked Phase-One Compiler Example

This section makes the intended behavior concrete. Names such as the project
filename, CLI flags, and register syntax are governed by the
[Frozen Implementation Contracts](#frozen-implementation-contracts); older
illustrative snippets do not override those contracts.

### Example Project

```text
demo/
  rtvc-c80.toml
  src/
    main.c80
    video.c80
    game_data.c80
    screen.c80
```

```toml
# rtvc-c80.toml
target = "tvc"
entry = "main::main"

[[unit]]
name = "main"
path = "src/main.c80"
origin = 0x2000

[[unit]]
name = "video"
path = "src/video.c80"
origin = 0x2800

[[unit]]
name = "game_data"
path = "src/game_data.c80"
origin = 0x3000

[[unit]]
name = "screen"
path = "src/screen.c80"
```

The addresses are illustrative project placement, not a canonical TVC memory
map. A real project must choose origins compatible with its active TVC mapping.

`game_data.c80` is an ordinary data-only unit:

```c
pub u8 frame_counter;
pub u8 positions[16];
pub str enemy_name = "hello world";
```

With declaration-order layout and no padding requirement between these byte
objects, it produces:

```text
3000              frame_counter       1 zero-initialized byte
3001..3010        positions           16 zero-initialized bytes
3011..301C        enemy_name          0B + "hello world"
```

The loadable data bytes are therefore conceptually:

```text
00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
0B 68 65 6C 6C 6F 20 77 6F 72 6C 64
```

`screen.c80` exports typed constants describing memory owned by the machine
rather than the program:

```c
pub const ptr<u8> bytes = ptr<u8>(0x8000);
pub const u16 byte_count = 16384;
```

It contributes no code or storage, so this unit needs no `origin` and produces
no loadable segment. `bytes` is a 16-bit compile-time pointer value. Indexing it
performs a memory access at `$8000 + index`. All indirect pointer reads and
writes are observable in the initial language, so the compiler emits each
evaluated access in source order.

`video.c80` imports both units:

```c
import game_data;
import screen;

pub void clear_first_row(u8 colour) {
    u16 offset = 0;

    while (offset < 80) {
        screen::bytes[offset] = colour;
        offset = offset + 1;
    }
}

pub void print_name() {
    u8 index = 0;

    while (index < game_data::enemy_name.len) {
        cpu::out(0x0006, game_data::enemy_name[index]);
        index = index + 1;
    }
}
```

The pointer rule makes every loop iteration perform a real memory write even if
later optimizer analysis thinks the value is redundant.
The string loop reads the prefix for `.len` and reads payload byte `index + 1`.
Port output cannot be expressed as a memory array because Z80 port I/O is a
separate address space. This is an illustrative hardware write, not a text
printing routine: TVC port 06 controls sound/printer/video fields.

For comparison only, adding `@stackcall` to `clear_first_row` permits the
following direct stack-based lowering. It is not the normal output required
for the unannotated register-call function above:

```asm
video__clear_first_row:
    push ix
    ld   ix,0
    add  ix,sp
    dec  sp             ; reserve the two-byte local `offset`
    dec  sp

    xor  a
    ld   (ix-2),a       ; offset low byte = 0
    ld   (ix-1),a       ; offset high byte = 0

.loop:
    ld   l,(ix-2)
    ld   h,(ix-1)
    ld   de,80
    or   a              ; clear carry
    sbc  hl,de
    jr   nc,.done       ; unsigned offset >= 80

    ld   l,(ix-2)
    ld   h,(ix-1)
    ld   de,8000H       ; compile-time value of screen::bytes
    add  hl,de
    ld   a,(ix+4)       ; low byte of the u8 argument slot: colour
    ld   (hl),a         ; observable indirect write

    inc  (ix-2)
    jr   nz,.loop
    inc  (ix-1)
    jr   .loop

.done:
    ld   sp,ix          ; discard locals
    pop  ix
    ret
```

This deliberately plain form exposes the ABI and maps closely to source. It is
correct but much more expensive than the Z80 needs. Once Phase 1E recognizes
that `offset` is a private induction variable with the constant range `0..79`,
the same function can become:

```asm
video__clear_first_row:
    push ix
    ld   ix,0
    add  ix,sp
    ld   a,(ix+4)       ; colour
    ld   hl,8000H       ; screen::bytes
    ld   b,80

.loop:
    ld   (hl),a         ; still exactly 80 observable writes, in order
    inc  hl
    djnz .loop

    pop  ix
    ret
```

The stack ABI still requires IX here because Z80 has no ordinary
stack-pointer-relative argument load. The Phase 1B `@fastcall` convention, if it
passes an eight-bit argument in A could omit the entire frame and prologue. The
exact instruction selection is not a language guarantee; assembly snapshots
and byte/T-state tests should make backend quality visible as it improves.

`main.c80` demonstrates public calls, array access, control flow, and explicit
conversion:

```c
import game_data;
import video;

void main() {
    u16 score = 1000;
    u8 slot = 0;

    // A width-changing conversion is explicit.
    game_data::frame_counter = u8(score);
    game_data::frame_counter = game_data::frame_counter + 1;

    while (slot < 16) {
        game_data::positions[slot] = slot;
        slot = slot + 1;
    }

    if (game_data::frame_counter != 0) {
        video::clear_first_row(1);
        video::print_name();
    }
}
```

The compiler rejects the same assignment without conversion:

```c
game_data::frame_counter = score;
```

```text
error[C80-TYPE-004]: cannot assign u16 to u8 without an explicit conversion
 --> src/main.c80:9:32
  |
9 |     game_data::frame_counter = score;
  |                                ^^^^^ use u8(score) if truncation is intended
```

It also rejects access to a non-public declaration:

```text
error[C80-NAME-007]: `game_data::scratch` is private to unit `game_data`
```

and rejects implicit integer-to-pointer conversion:

```c
// Invalid: hardware addresses must be explicit.
pub const ptr<u8> bytes = 0x8000;
```

```text
error[C80-TYPE-009]: cannot convert u16 to ptr<u8> implicitly
                     use ptr<u8>(0x8000)
```

### Example CLI and Outputs

The proposed Phase 1 CLI shape is:

```text
rtvc-c80 build demo/rtvc-c80.toml \
  --emit-asm demo/build/program.asm \
  --emit-segments demo/build/program.toml
```

A successful command writes ordinary helper assembly and loadable segments. It
also writes or exposes through the library a versioned metadata result. The
exact external map encoding should be chosen after the in-process model works,
but its information is concrete:

```text
unit main, source expression `game_data::frame_counter + 1`
  -> generated instruction IDs 41, 42, 43
  -> final addresses 2034..203B inclusive
  -> bytes 3A 00 30 C6 01 32 00 30
  -> timings 13 T, 7 T, 13 T

address 2037
  -> instruction ID 42
  -> expression `game_data::frame_counter + 1`
  -> statement `game_data::frame_counter = game_data::frame_counter + 1;`
```

The byte sequence above is illustrative rather than a promise for that source
line. Compiler tests should use exact byte assertions for small canonical
cases, while the plan requires the mapping relationship regardless of later
code-generation improvements.

If unit code grows into the next configured origin, linking fails before any
output is loaded:

```text
error[C80-LAYOUT-006]: unit `main` ($2000..$2874) overlaps unit `video`
                       ($2800..$2A31)
```

### Example Stack and Fastcall Calls

For the explicit stack ABI (attribute spelling provisional):

```c
pub @stackcall u16 add(u16 left, u16 right) {
    return left + right;
}

void example() {
    u16 result = add(1, 2);
}
```

the caller pushes right-to-left 16-bit slots and cleans them after `CALL`.
Conceptually:

```asm
    ld   hl,2
    push hl             ; right
    ld   hl,1
    push hl             ; left
    call math__add
    pop  bc             ; discard left slot
    pop  bc             ; discard right slot
    ; result is in HL
```

With IX established after saving the caller's IX, `left` is at `IX+4` and
`right` at `IX+6`:

```asm
math__add:
    push ix
    ld   ix,0
    add  ix,sp
    ld   l,(ix+4)
    ld   h,(ix+5)
    ld   e,(ix+6)
    ld   d,(ix+7)
    add  hl,de
    pop  ix
    ret
```

An 8-bit argument occupies the same 16-bit slot: `u8` is zero-extended, `i8`
is sign-extended, and `bool` is canonicalized to 0 or 1. This makes stack
offsets regular.

For an ordinary function using the default register ABI, the proposed
convention uses HL and DE:

```c
pub u16 add_fast(u16 left, u16 right) {
    return left + right;
}
```

```asm
    ld   hl,1            ; left
    ld   de,2            ; right
    call math__add_fast
    ; result is in HL

math__add_fast:
    add  hl,de
    ret
```

This register assignment is frozen by F002. The important
behavior is that the resolved calling convention is part of the exported
signature and all callers use it. Ordinary functions use register calling;
`@stackcall` explicitly selects stack arguments.

## Phase-Two Integrated User Workflow

1. Open a C80 Source pane in the Developer Workspace.
2. Create or open one source file and select a target profile and load origin.
3. Edit source with syntax highlighting and inline diagnostics.
4. After a short idle debounce, inspect the last successful generated assembly
   in a toggleable right-hand pane.
5. Move the source cursor or select a source statement to highlight and reveal
   its generated instructions. Selecting assembly highlights the originating
   source span.
6. Build and Load the assembled segments into mapped writable memory.
7. Run, pause, set a source breakpoint, or step source while the current PC is
   highlighted in both views.

This workflow is explicitly Phase 2. It consumes the already-tested Phase 1
compiler API and metadata without changing the language or code generator to
serve UI-specific needs.

## Version-One Language

### Lexical and File Model

- UTF-8 source, with identifiers restricted initially to ASCII letters,
  digits, and underscore so symbol spelling is unambiguous to the assembler.
- `//` line comments and `/* ... */` block comments.
- Decimal and hexadecimal integer literals. Choose and document one canonical
  hexadecimal spelling while accepting at least `0x1234`. Support character
  literals and string literals for prefixed `str` declarations/call arguments
  in the first version; general byte-array literal initialization can wait.
- One source file is one independently placeable compilation unit. A unit can
  still be compiled and loaded by itself; a project build combines multiple
  units as described below.
- No textual preprocessor or C-style header files.
- Source locations use UTF-8 byte offsets internally and derive line/column
  information for display. Every token, AST node, diagnostic, IR operation,
  and generated instruction carries a `SourceSpan` or explicit synthetic
  provenance.

### Types

The first end-to-end slice supports:

- `void`, `bool`, `u8`, `i8`, `u16`, and `i16`;
- 16-bit `ptr<T>` values;
- immutable size-prefixed `str` values backed by static storage;
- global scalar variables, fixed arrays of scalar elements, and constants;
- array indexing;
- function parameters and scalar local variables; and
- statically bound functions.

The Z80 does not have distinct signed and unsigned storage. The backend has
only byte and word storage classes:

| Source type | Stored representation |
| --- | --- |
| `bool`, `u8`, `i8` | one byte |
| `u16`, `i16`, `ptr<T>` | one little-endian word |

`i8` and `i16` are source-level interpretations carried by loaded values, not
different variable layouts or load/store instructions. Keeping the
interpretation in a variable, parameter, or return type is still useful:

```c
u8 raw = 0xFE;
i8 delta = i8(raw);       // same bits, interpreted as -2

bool a = raw < 1;         // false: unsigned comparison of 254 and 1
bool b = delta < 1;       // true: signed comparison of -2 and 1
```

For example, assume an imported function returns a signed joystick displacement
in A and the following code appears inside a function with an IX frame:

```c
i8 dx = input::read_joystick_x();

if (dx < 0) {
    movement::move_left();
}
```

A direct lowering that preserves the source local can be:

```asm
    call input__read_joystick_x ; i8 result in A
    ld   (ix-1),a               ; dx occupies one ordinary byte

    ld   a,(ix-1)
    bit  7,a                    ; signed value is negative iff bit 7 is set
    jr   z,.not_negative
    call movement__move_left
.not_negative:
```

If `dx` is unused afterward, local-value propagation can eliminate its stack
slot entirely:

```asm
    call input__read_joystick_x
    bit  7,a
    jr   z,.not_negative
    call movement__move_left
.not_negative:
```

There is no signed load or signed byte representation here. The `i8` result
tells the compiler that `< 0` means a sign-bit test. An explicitly unsigned
spelling such as `(u8(dx) & 0x80) != 0` could generate the same instructions; the
signed type records that interpretation in the function contract and avoids
repeating it at each use.

Signedness selects comparison lowering, arithmetic versus logical right shift,
sign versus zero extension, and literal range checking. Addition, subtraction,
loads, stores, and same-width representation are otherwise identical. The
typed IR should therefore separate a value's interpretation from its byte/word
storage class instead of inventing signed Z80 storage.

Conversions make the interpretation change explicit. A same-width signedness
conversion preserves all bits; widening sign-extends an `i8` source and
zero-extends a `u8` source; narrowing keeps the low bits. This is fully defined
and does not inherit C's integer-promotion or overflow rules.

Add structs, pointers to structs, and field access in the language-usefulness
milestone after the scalar calling convention and source map are stable. These
are planned language features, but they should not delay the first compiled and
debuggable program.

Arrays do not decay implicitly to pointers. Constant indexes outside the
declared length are compile errors. Dynamic indexing does not add runtime bounds
checks in the baseline low-level language; an optional checked operation can be
considered later without changing ordinary array cost.

Pointers are always represented as `u16` addresses at runtime, but remain a
distinct compile-time type. Keep pointer semantics intentionally small:

```c
ptr<u8> source = ptr<u8>(0x9000);
ptr<u8> registers = ptr<u8>(0xBF00);

u8 first = source[0];
u8 second = *(source + 1);
registers[3] = 0x80;
```

- `ptr<T>(address)` is the explicit integer-to-pointer conversion;
- `u16(pointer)` is the explicit pointer-to-integer conversion;
- `*pointer` and `pointer[index]` produce an lvalue of `T`;
- adding/subtracting an integer scales by the size of `T` and wraps in the
  16-bit address space;
- equality/inequality are supported, while pointer ordering and pointer
  subtraction are omitted initially;
- every evaluated indirect read/write is emitted and remains in source order
  relative to other indirect accesses, port I/O, calls, and inline assembly; and
- arrays do not decay to pointers: use `&array[0]` explicitly.

This needs type checking and addressing rules, but no runtime pointer object,
allocator, ownership model, alias-analysis framework, or `volatile` qualifier.
Direct-access optimization follows the observable-memory rules below;
it must never bypass aliasing or interrupt-sharing rules. Introduce a second,
optimizable pointer kind later only if measured code demonstrates that the
extra language distinction is worthwhile.

`str` is deliberately not a dynamic C string. A global declaration:

```c
pub str enemy_name = "hello world";
```

emits one length byte followed by the encoded payload and no terminator:

```text
0B 68 65 6C 6C 6F 20 77 6F 72 6C 64
```

The encoded payload is limited to 255 bytes. Initially accept printable ASCII,
common escapes, and `\xNN` byte escapes; add target character-set mapping later
instead of silently storing UTF-8 bytes that the emulated machine may not
interpret correctly.

A global `str` owns immutable storage. A `str` parameter or local value is a
16-bit reference to that size-prefixed storage, passed using the normal 16-bit
ABI. Support `value.len` as a `u8` read of the prefix and `value[index]` as a
`u8` payload read at prefix-plus-one. Constant out-of-range indexes are errors;
dynamic indexing follows the ordinary unchecked array rule. Do not support
string mutation, concatenation, allocation, or assignment into owned string
storage. String literals may be passed directly to `str` parameters by placing
anonymous prefixed data immediately after the owning function's executable
body, preserving unit locality and preventing execution from falling into data.

Do not support:

- `float`, `double`, wider integers, unions, bitfields, enums, or implicit C
  integer-promotion rules;
- function pointers, varargs, recursion, heap allocation, or runtime global
  initialization;
- C's full declarator grammar, implicit declarations, undefined signed
  overflow, or unspecified evaluation order; or
- a textual preprocessor or macro language.

All scalar sizes and conversions are defined by C80. Conversions between
different widths, signedness, integer and pointer types, or integer and `bool`
must be explicit except for a literal proven to fit its destination. Arithmetic
wraps at the declared width. Signed comparison and right-shift behavior follow
F001 and must be tested rather than inherited accidentally from Rust or C.

Prefer type-constructor conversion syntax over C casts:

```c
u16 wide = u16(small);
bool ready = bool(status);  // false only when status is zero
u8 bit = u8(ready);         // always 0 or 1
```

Type-constructor conversion syntax is adopted. Implicit conversion must not
be reintroduced as a convenience during code generation.

Do not support source multiplication, division, or remainder, or silently insert
arithmetic runtime helpers. Add compile-time `sizeof(T)` using the compiler's
actual packed layout. Typed values always require explicit conversions when
width or interpretation changes. A literal acquires its context's type only if
it fits, without any conversion of an already typed value.

### Local Initialization and Observable Memory

Scalar locals have no implicit initialization. Report a compile error on any
read not definitely preceded by an assignment on every incoming path. This
emits no initialization instructions and allows declaration before assignment.
An explicit assembly output counts as an assignment; an input or in/out operand
requires a previously assigned value. Taking an address does not prove that an
opaque assembly block or call initialized that local; the programmer must use
an explicit output contract or initialize it first. Uninitialized reads are
errors, not arbitrary register/stack contents, and the compiler inserts no
automatic zeroing for locals.

For interrupt sharing, making only writes observable is insufficient: a polling
loop must also re-read its shared value. Every evaluated
access to mutable source-level memory (globals, arrays, dereferenced pointers,
and addressable locals) is observable and remains ordered. Do not cache those
reads across source accesses or eliminate their writes, even when the address
is known. No separate volatile spelling is necessary for shared memory.
Immutable constants/data may still be folded when the language permits it.

Pure scalar locals are values eligible for registers; compiler-private spills
are implementation storage, not observable source memory. Optimize both freely
subject to value semantics and aliasing. Once a local's address is exposed,
apply the source-memory rule consistently. This distinction keeps loops over
local counters fast while making explicit memory access predictable. The cost
is that repeated global accesses remain real accesses. Neither observable reads
nor writes promise multi-byte atomicity.

### Struct Layout (Phase 1D)

Structs and pointers to structs arrive in Phase 1D. Their intended use is:

```c
struct Sprite {
    u8 x;
    u8 y;
    u16 bitmap;
    bool visible;
};

pub Sprite enemies[8];

void move(ptr<Sprite> sprite, i8 dx) {
    sprite->x = sprite->x + u8(dx);
}

ptr<Sprite> player = ptr<Sprite>(0x9000);
```

Struct layout is declaration-order and packed unless a future explicit padding
construct says otherwise. In this example `x` is offset 0, `y` offset 1,
`bitmap` offsets 2–3 in Z80 little-endian order, `visible` offset 4, and the
computed `Sprite` size is 5. There is no implicit integer-to-pointer
conversion; `ptr<Sprite>(0x9000)` makes the absolute-address interpretation
visible. The pointer-constructor spelling and allowed conversions are fixed by F001.

### Statements and Expressions

The first end-to-end slice includes:

- blocks, variable declarations, expression statements, assignment, `if`,
  `else`, `while`, `break`, `continue`, `return`, and function calls;
- unary `!`, `~`, unary `-`, address-of `&`, and dereference `*`;
- array indexing, `+`, `-`, bitwise operators, shifts, comparisons, equality,
  `&&`, and `||`;
- pointer indexing, scaled pointer addition/subtraction, and pointer equality;
- `str.len` and read-only `str[index]`;
- compound assignment and increment/decrement only after their single-
  evaluation semantics are covered by tests.

Add `do/while` and `for` as straightforward lowering conveniences after the
core control-flow implementation. `switch`, `goto`, comma expressions,
ternary expressions, and C-compatible sequence-point rules are non-goals for
version one.

Evaluation order is always left-to-right. `&&` and `||` short-circuit. This is
part of the language contract and gives both users and the compiler a stable
model.

### Target-Specific Operations

Avoid pretending that machine I/O is portable C. Provide typed compiler
intrinsics for operations such as mapped-memory access and Z80 port input and
output. Machine-specific libraries can wrap those primitives later.

#### Proposed Port Input and Output Form (Phase 1D / E10)

Use built-in call expressions with these signatures (no declaration or import
required):

```c
u8 cpu::in(u8 port);
void cpu::out(u8 port, u8 value);
void cpu::di();
void cpu::ei();
void cpu::ldir(/* HL source */ ptr-or-u16 hl, /* DE dest */ ptr-or-u16 de, u16 bc);
```

These are compiler intrinsics in the reserved `cpu` namespace, not ordinary
linked functions. They cannot be imported or taken as function values. Keep
`in` and `out` as inline-assembly operand vocabulary (`in:` / `out:`) rather
than adding statement keywords. Bare names `in` / `out` / `di` / `ei` /
`ldir` are not reserved. The implemented contract is in
[info/c80.md](../../info/c80.md).

```c
u8 read_device(u8 port) {
    return cpu::in(port);
}

void write_device(u8 port, u8 value) {
    cpu::out(port, value);
}

void write_pair(u8 port, u16 value) {
    cpu::out(port, u8(value));
    cpu::out(port + 1, u8(value >> 8));
}
```

The port is an 8-bit port number; each operation transfers exactly one byte.
This matches the TVC's low-byte device decoding; see
[Port decode](../../info/tvc.md#port-decode). There is no implicit word transfer
or memory-pointer interpretation. The ordinary conversion rules apply:
fitting literals such as `cpu::out(0x06, 0x80)` and typed `u8` ports are accepted;
typed `u16` ports or values require explicit narrowing with `u8(...)`.
Reject wrong arity/types, out-of-range literals, and use of the void output
result as a value. Port arithmetic follows ordinary `u8` wrapping rules.
For TVC, only the low address byte is contractual; the high bus byte is
unspecified. For GenericZ80 and Zx82, zero-extend the byte port to BC for a
deterministic address. Devices requiring a nonzero high byte use explicit
inline assembly, including expansion-specific decoding. These conveniences
do not claim to cover every Z80 device.

Evaluate arguments exactly once, left-to-right: for output, finish evaluating
the port, then the value, then perform the write. Input is an expression and
may also be a discarded-result statement. Every evaluated input or output is
observable, including an unused input result or repeated identical writes.
Preserve order relative to other I/O, observable source-memory accesses, calls,
and inline assembly. Never fold, merge, eliminate, speculate, or hoist an I/O
operation; short-circuited or otherwise unexecuted expressions perform no I/O.
These operations do not imply interrupt masking or atomic read/modify/write.

Represent input/output explicitly in the typed IR with effect and source-span
metadata. Dynamic-port lowering puts the port in C and uses `IN A,(C)` for input
and `OUT (C),A` for output. Model register overlap and flag effects accurately,
preserve live values during operand setup, and preserve the evaluated port if
the value expression contains a call or another input. They are inline
operations, with no CALL, runtime helper, or callee stack allowance; any
preservation spills still count toward the ordinary stack bound. Include setup
and I/O instructions in source maps and final byte/timing metadata.

For TVC constant ports, select `IN A,(n)` or `OUT (n),A` when beneficial:
TVC devices ignore their high address byte. For dynamic TVC ports, B need not
be initialized solely for I/O. On GenericZ80 and Zx82 initialize B to zero and
retain BC-addressed lowering. Track the selected instruction's actual flag
effects. Block I/O and explicit flag results remain available through inline
assembly; they are outside these two built-ins.

Validate low-byte port numbers and byte values with an instrumented TVC bus,
including immediate and dynamic forms. For GenericZ80/Zx82 verify the zero
high address byte. Reject port literals above 255 unless explicitly narrowed.
Cover constant/dynamic ports, explicit conversions and diagnostics, unused
inputs, repeated writes, polling, nested `cpu::out(port_expr(), cpu::in(other))`,
short-circuiting, live registers/flags, and ordering with mutable memory.
Compare baseline and optimized effect traces and verify assembler round trips,
source provenance, and instruction timing metadata.

Use a target profile in compiler options rather than hard-coding TVC behavior
into the frontend:

```rust
pub enum C80Target {
    GenericZ80,
    Tvc,
    Zx82,
}
```

The first editor load path targets TVC mapped writable memory. The compiler
core and command-line tool should remain usable with `GenericZ80`; Zx82 load
integration can follow using the same segment result.

### Multiple Compilation Units

Use a project build rather than C headers or a general object-file linker. Each
`.c80` file is a named unit that emits at most one contiguous memory image. An
`origin` is required when the unit emits code or storage; a constants-only unit
needs none. A unit exposes functions, variables, constants, and later types
with `pub`; other units refer to them through an imported, qualified module
name:

```c
// video.c80
pub u8 frame_counter;

pub void draw_sprite(u16 address, u8 x, u8 y) {
    // ...
}
```

```c
// main.c80
import video;

void main() {
    u8 frame = video::frame_counter;
    video::draw_sprite(0x9000, 10, 20);
}
```

`pub` changes visibility, not storage or calling convention. A public variable
is ordinary mutable storage at its project-assigned final address; reads,
writes, address-taking, width, signedness, and later struct layout are checked
exactly as for a private variable. `pub const` exposes a compile-time value and
does not allocate storage unless its addressable form is introduced later.

Use `import video;` and `video::symbol` as the initial module syntax:

1. Parse every unit and collect its public function, global, constant, and
   later type signatures.
2. Resolve imports and type-check cross-unit references against that collected
   interface. There are no duplicated prototypes to drift out of sync.
3. Lower each unit independently, namespace private assembler symbols by unit,
   and preserve the unit in all source-map records.
4. Preserve top-level declaration order within each unit: functions and owned
   data stay where they are defined. Do not collect project-wide text/data/BSS
   sections or move data to another mapping window. Put anonymous literals
   immediately after their owning function's executable body, without executable
   fallthrough into them. Combine units with one `ORG` per configured origin.
5. Run one final `assemble_program` call so absolute `CALL`, `JP`, and data
   references resolve across units and all final addresses are authoritative.
6. Reject duplicate exports, missing imports, signature mismatches, segment
   overlap, address overflow, and target-profile disagreement before loading.

An import is a semantic dependency, not a filesystem include. The project
manifest maps the unit name `video` to a path. The build coordinator reads and
parses every listed source once, collects all public interfaces, and then
analyzes bodies. Live compilation may cache unchanged parsed units by source
revision/content hash, but a full parse of small files should remain the
correctness baseline.

Because all units are assembled together, no relocation/object format or
general linker is required initially. Mutually referring units are valid at
the symbol-resolution level, but direct or mutual recursive function calls are
rejected by the statically known call graph.

A project build rebuilds/reassembles the complete project and emits all
segments. This matters because code growth inside one unit can move an
exported function and therefore change `CALL` operands in its callers. Truly
independent hot replacement would require stable exported entry addresses or a
jump table; defer that mechanism until a real workflow needs it.

Use a small TOML project file as the authoritative source list and memory
placement description:

```toml
target = "tvc"
entry = "main::main"

[[unit]]
name = "main"
path = "src/main.c80"
origin = 0x8000

[[unit]]
name = "video"
path = "src/video.c80"
origin = 0x8400
```

Source files do not contain `ORG`; placement is external so the same unit can be
reused at another address. Reject `ORG` inside inline assembly as an attempt to
escape the unit's assigned sections. Single-file compilation continues to take
target and one origin directly from the CLI/editor. Keep explicit unit placement
initially. Any later automatic placement must move whole units within explicitly
compatible mapping regions, preserving their internal code/data order; automatic
global text/data/BSS splitting is not the intended memory model. Initialized and
zero-initialized globals both emit their bytes in place initially. Non-emitted
reservations such as a stack are separate project allocations.

### Data and Absolute-Memory Units

A data section is just a normal unit containing public or private globals and
no functions:

```c
// game_data.c80
pub u8 score;
pub u8 sprite_buffer[256];
pub u16 row_offsets[24];
pub str enemy_name = "hello world";
```

Its project `origin` places the resulting initialized and zero-initialized
storage. Other units use `import game_data;` and qualified variable/array
access exactly as they use exported functions.

Use pointer indexing for an absolute RAM range, bank window, or the occasional
memory-mapped device:

```c
// video_memory.c80
pub const ptr<u8> pixels = ptr<u8>(0x8000);
pub const u16 pixel_count = 16384;
```

Another unit can write `video_memory::pixels[offset] = colour;`. The pointer is
a compile-time 16-bit value and indexing uses normal pointer addressing. The
language's conservative indirect-access rule preserves the access without a
separate qualifier. The `pixel_count` constant provides a bound when code wants
to check one; raw pointers do not carry hidden runtime length metadata.

A constants-only hardware-description unit emits no code or storage and
therefore does not need an `origin` in the project:

```toml
[[unit]]
name = "video_memory"
path = "src/video_memory.c80"
```

An `origin` becomes required if that unit later emits a function or owns
storage. `ORG` remains project-managed; an explicit pointer constant is a value
in the language rather than a request to place generated output. Normal Z80
device access still uses typed `cpu::in`/`cpu::out` intrinsics because ports are a
separate CPU address space; pointers are not intended to disguise port I/O as
memory access.

### Static Reservations and Mixed BASIC/C80 Projects

Most C80 programs preallocate their memory. The build reports exact emitted
code/data ranges and explicit non-emitted reservations, including stack space
and hardware buffers. Taking an absolute pointer does not reserve that range;
projects declare external storage separately. Validate overlaps using ranges
wide enough to represent an exclusive end of 65536 without wrapping.

A TVC project may combine one BASIC source file with multiple C80 units. Here
"objects" means compiler-produced units in the same project build; a general
relocatable object-file format remains out of scope. Reuse
[`tokenize_program`](../../src/emulator/basic.rs) for BASIC. Resolve symbolic
BASIC references to final explicitly exported callable addresses before tokenization,
then validate the exact tokenized program size against its assigned region.
F004/F006 define explicit symbol references that avoid substitutions in strings,
comments, or unrelated identifiers. Fixed C80 origins avoid a layout
cycle caused by the length of rendered BASIC address literals.

Initial BASIC integration is deliberately small: allow one BASIC file to call
explicit assembly/C80 entry points and expose generated build constants for
addresses and the required memory reservation. Choose a narrow substitution
syntax, not a general macro preprocessor. The BASIC source explicitly performs
the required target-specific reservation/setup using those constants; the
compiler does not synthesize ROM wrappers or take over the interpreter.

Include a small checked runnable example using the existing BASIC load path.
Its setup must reserve C80 memory before BASIC allocations can overwrite it.
Link-time range validation cannot guarantee safety if that setup is omitted or
later undone. Loading token bytes alone does not initialize interpreter state.
Defer automatic BASIC workspace management and cassette bootstrap generation;
do not concatenate C80 bytes onto an ordinary BASIC CAS payload and assume they
will be loaded at the linked addresses.

The mixed artifact is explicitly the same `rtvc-asm-v1` TOML used for C80/ASM:
`--emit-segments` includes the tokenized BASIC payload at `[basic].origin` as
well as all C80/ASM segments. F004/F006 specify serialization and execution
acceptance. An in-process BASIC buffer alone is not the mixed-project output.

C80 callable routines share BASIC's CPU stack and contribute to its reserved
stack budget; BASIC's evaluation stack is a separate allocation. Freestanding
startup owns the program's CPU stack reservation. Compute compiler-generated
stack requirements from final call/frame lowering as described in the backend
strategy below, adding declared external-call and interrupt allowances. Static
placement does not imply that every runtime stack requirement is known.

## Inline Assembly

Inline assembly with explicit register inputs and outputs is a Phase 1B
capability. It is the primary boundary for ROM calls, hardware operations, and
assembly routines. Do not generate automatic BASIC/ROM wrappers. Users write
any adaptation explicitly, including result registers, flags, mapping changes,
and preservation of external state.

The first form is a statement block containing normal rtvc helper-assembler
syntax:

```c
asm {
    di
    out (5), a
    ei
}
```

Initial semantics:

- the block may refer to assembler labels defined inside the block;
- individual assembly statements retain spans inside the C80 source;
- the compiler treats AF, BC, DE, and HL as clobbered, invalidates temporary
  register knowledge, and does not assume flags survive;
- IX remains the active frame pointer when a function has a frame and IY is
  reserved. The block must restore IX/IY and SP before returning to C80 code.
  Balanced saves/restores and returning CALL/RST operations are permitted with
  explicit clobber and stack-usage contracts; no unbalanced stack change is
  allowed inside an ordinary function;
- branch targets within a block are local. A returning call is permitted;
  arbitrary jumps/returns out of the block are not. Standalone startup and
  interrupt entry/exit assembly use their own explicit entry contract; and
- compiler variable names are not interpolated into raw assembly initially.

For example:

```c
u8 next = counter + 1;
bool was_zero = value == 0;

asm {
    xor a
    ld hl, 1234H
}

if (was_zero) {
    counter = next;
}
```

Before entering the raw block, the compiler must preserve any live value held
only in AF, BC, DE, or HL. It may keep a value that already has an authoritative
memory location there, or spill a register-only temporary to its stack frame.
After the block it reloads values when needed. Conceptually, generated code can
look like:

```asm
    ; Compute values and preserve them across asm.
    ld   a,(counter)
    inc  a
    ld   (ix-1),a       ; next

    ld   a,(value)
    or   a
    ld   a,0
    jr   nz,.not_zero
    inc  a
.not_zero:
    ld   (ix-2),a       ; materialized was_zero, not just Z flag

    ; User's raw block may replace A, F, B, C, D, E, H, and L.
    xor  a
    ld   hl,1234H

    ; Recreate register and flag state from preserved values.
    ld   a,(ix-2)
    or   a              ; establish fresh flags for the C80 condition
    jr   z,.done
    ld   a,(ix-1)
    ld   (counter),a
.done:
```

The exact instruction sequence is not contractual. The contract is that the
compiler never carries a register value or a condition-code assumption across
raw assembly. This is safe but can create spills; the later constrained form
lets the programmer describe inputs, outputs, and narrower clobbers so the
compiler can avoid unnecessary preservation.

The first executable milestone also supports explicit operands and clobbers.
Provisional syntax for a copy whose final pointer/count values are discarded:

```c
asm(in: hl = src, in: de = dst, in: bc = count,
    clobber: hl, de, bc, flags, memory) {
    ldir
}
```

Inputs establish values at block entry and may be listed as clobbered afterward.
Outputs assign C80 destinations from explicitly named registers at block exit;
in/out operands model values both consumed and replaced. Declare flag outputs
explicitly when converting carry/zero results to C80 booleans. Resolve operand
moves without destroying other inputs or outputs. Keep syntax small and named
after actual registers, not an opaque constraint language. Memory is a
conservative barrier by default; a future narrower memory-effects form is not
required initially. Verify zero-count LDIR semantics in examples rather than
assuming a count of zero copies nothing.

## Calling Convention and Stack Model

### Confirmed Runtime Direction

Freestanding programs are the primary use case. Calling C80 routines from
BASIC or existing machine code uses explicit programmer-authored assembly when
adaptation is needed. No compiler-generated BASIC/ROM wrappers are required.
Where input/output registers already match, use the entry directly subject to
the external preservation contract; do not assume the entire ABI matches.

Stack size is configured at project level and the linker reserves its range.
Expose stack bounds as build symbols. Stack setup is manual: programmer-written
startup loads SP from the exported stack-top symbol and explicitly transfers
control to C80 code. No generated SP setup or startup wrapper is required.
BASIC/assembly calls inherit the caller's stack. The selected entry must
distinguish a startup address from an ordinary callable function; setting PC
does not supply a RET address. Manual startup also defines what happens if the
called entry returns. The compiler never changes interrupt state implicitly.

Interrupt entry and exit are always managed manually, including vectors,
DI/EI, return instructions, nesting, and any mapper state. The compiler does
not generate interrupt prologues/epilogues or automatic critical sections.
Interrupt handlers are responsible for preserving the interrupted register
state, including flags and any alternate/index registers they use, and for
restoring SP. Ordinary generated code does not save registers merely because
an interrupt could occur. A handler calling C80 code must preserve the
registers that the called ABI may clobber as well as its own clobbers. This
responsibility does not make multi-byte accesses atomic. Interrupt-shared
globals follow the observable-memory rules: every evaluated read and write is
emitted, independently of the handler's register-preservation responsibility.

### Stack and Register Calls

Register calling (previously called fastcall) is the default C80 convention.
Use an explicit `@stackcall` attribute when stack arguments are needed; its
spelling is fixed by F002. An optional `@fastcall` spelling may document the
default but is not required. Always record the resolved convention in exported
function signatures.

Keep this simple stack ABI as the explicit alternative:

- arguments are pushed right-to-left as 16-bit stack slots, including 8-bit
  values;
- the caller removes argument slots after the call;
- `u8`, `i8`, and `bool` return in A;
- pointers, `u16`, and `i16` return in HL;
- AF, BC, DE, and HL are caller-saved;
- IX is a callee-saved frame pointer when a frame is required;
- IY is reserved for future target/runtime use; and
- SP must be balanced at every control-flow merge and function return.

Fastcall is required in the first executable milestone (Phase 1B), alongside
the stack ABI. Useful register calls must not depend on Phase 1E optimization.
Register calling is part of the function's public type, so every caller,
including a caller in another unit, must use the same convention. Validate the
F002 register assignment with representative 8-bit, 16-bit, mixed-argument,
nested-call, and inline-assembly examples. Adding or
removing `@stackcall` is an ABI change. Do not silently switch a function to
stack calling when its arguments exceed the register budget; define the
excess-argument policy explicitly.

Canonical small fastcall leaf routines must accept arguments in registers and
return without argument stack slots or an unnecessary IX frame. CALL/RET still
use the hardware stack; fastcall avoids argument/frame overhead, not all stack
use. Nested calls may require spills to preserve live values. Evaluate argument
expressions left-to-right before arranging them in ABI registers or the stack
ABI's right-to-left slots. Freeze the register allocation and excess-argument
policy before implementing call lowering, with byte/T-state acceptance tests
for small leaf routines and correctness tests for nested calls.

Locals that need memory initially use an IX-relative frame; pure scalar locals
may remain in registers. Reject a frame whose offsets cannot
be encoded safely by the selected instruction sequences. Leaf functions with
no locals may omit the frame from the start; broader frame elimination belongs
to the later code-generation phase.

Reject direct and mutual recursion using the statically known call graph. This
does not prohibit normal nested calls or interrupt entry. Generated functions
must not use hidden global temporaries that make ordinary nested calls fail.

### ROM Evidence and Proposed Register Assignment

The checked-in TVC BASIC 1.2 listings demonstrate register-oriented,
routine-specific contracts rather than a single uniform ROM ABI:

- [TABLE_LOOKUP_WORD](../../roms/TVC12_D4.64K.asm) takes a table pointer in HL
  and index in A, and returns a word in DE. Its instructions double A, add the
  offset to HL, and load E/D from memory without an argument frame.
- [CHECK_X_COORDINATE](../../roms/TVC12_D4.64K.asm) takes BC and returns
  `03FFH - BC` in HL plus an out-of-range indication in carry. Its complete
  body is `LD HL,03FFH; OR A; SBC HL,BC; RET`.
- [OUT_CHARS_SAFE](../../roms/TVC12_D4.64K.asm) uses BC for count, DE for the
  source pointer, HL for a device routine, C for each outgoing character, and A
  for completion/error status. Register arguments do not eliminate the saves
  required around nested calls.
- [BASIC_USR](../../roms/TVC12_D3.64K.asm) passes the converted integer argument
  in HL and returns through the BASIC integer-result conversion continuation
  with HL. This makes a one-word-in/one-word-out C80 routine a useful callable
  wrapper fixture; interpreter preservation still needs a verified contract.
- [VIDEO_PAGE_GUARD and VIDEO_PAGE_RETURN](../../roms/TVC12_D4.64K.asm) splice
  a restoration continuation into the stack and restore the memory mapper
  while retaining the returned AF. ROM-call contracts include paging and
  flags, not only parameter registers.

These observations support a predictable C80 register ABI plus explicit inline
assembly contracts. They do not justify treating arbitrary ROM routines as ordinary C80
functions or changing the C80 return register to match each ROM routine.
User-written assembly must adapt inputs, outputs (including flag results), clobbers, and
mapping requirements. The evidence here is from TVC BASIC 1.2; no compatibility
with other ROM versions or Spectrum routines is implied.

ABI v1 assignment, adopted in F002 and validated during implementation:

1. Reserve HL, DE, then BC for word/pointer/str arguments, in their declaration
   order among those arguments.
2. Assign byte/bool arguments in their declaration order to A, C, B, E, D, L,
   then H, skipping all halves of pairs reserved by step 1. Pair reservation is
   computed from the complete signature; it does not reorder evaluation.
3. Return byte/bool values in A and word/pointer/str references in HL. Flags
   are caller-clobbered, not an additional implicit C80 return value.
4. Reject a register signature that does not fit and suggest explicit
   `@stackcall`; do not add hidden stack arguments in the initial convention.

This gives `(ptr<u8>, u8)` HL+A, `(u16, u16)` HL+DE, and
`(ptr<u8>, ptr<u8>, u16)` HL+DE+BC. The ordering of additional byte registers
is a C80 design decision, not a convention established by the ROM. Test mixed
signatures, register permutation at calls, nested argument evaluation, and
register pressure as implementation acceptance tests.

## Compiler Architecture (Phase 1)

Add a pure compiler subsystem under `src/compiler/` (exact module names may be
adjusted to match implementation pressure):

```text
src/compiler/
  mod.rs          public compile API and CompilationResult
  source.rs       FileId, SourceSpan, line index, source files
  project.rs      units, imports, exports, placement, build orchestration
  token.rs        token kinds and located tokens
  lexer.rs
  ast.rs
  parser.rs
  diagnostic.rs
  types.rs
  semantics.rs    names, types, constants, layouts
  ir.rs           typed control-flow and value operations
  lower.rs        AST to IR
  z80.rs          ABI-aware IR lowering to structured Z80 items
  source_map.rs   provenance joins and bidirectional indexes
```

Keep a narrow entry point:

```rust
pub fn compile(input: CompileInput<'_>) -> CompilationResult;
```

Compilation should return diagnostics instead of panicking or using UI
callbacks. The result owns all data needed by a CLI, editor, assembler view,
or tests:

```rust
pub struct CompilationResult {
    pub diagnostics: Vec<Diagnostic>,
    pub assembly: Option<GeneratedAssembly>,
    pub assembled: Option<AssembledProgram>,
    pub symbols: CompilerSymbols,
    pub source_map: SourceMap,
}
```

Use stable IDs allocated within one compilation for AST/IR/generated items.
IDs need not survive recompilation. Editor selection is restored by source
span and nearest enclosing statement, not by assuming IDs remain stable.

### Frontend

Use a hand-written lexer and recursive-descent/Pratt parser. The grammar is
small enough that parser behavior and recovery are more valuable than a parser
generator dependency.

Separate parsing from semantic analysis. The parser constructs located syntax
without consulting emulator state. Semantic analysis builds scopes, resolves
names, computes struct/array layouts, checks lvalues and conversions, and
produces a typed representation.

Error recovery is a first-order editor requirement. Synchronize at semicolons,
closing braces, and top-level declaration starters. An incomplete expression
should produce a focused diagnostic while allowing later functions to parse.
Cap cascading diagnostics and distinguish errors from warnings and notes.

### Typed IR

Do not emit Z80 directly from parser actions. Use a compact typed IR with
explicit blocks and control flow. It does not need SSA initially, but it must
make evaluation order, widths, signedness, loads, stores, calls, and branches
explicit.

Each IR operation contains:

- its operation and typed operands;
- a stable `IrId` for this compilation;
- the most specific useful source span;
- an optional enclosing statement span; and
- provenance describing whether it is source-derived or compiler-synthetic.

This layer is the right place for constant folding and unreachable-code
diagnostics. Avoid optimization that merges unrelated source spans until the
assembly/source selection behavior is defined.

For example, this source:

```c
game_data::frame_counter = game_data::frame_counter + 1;
```

can lower to typed, provenance-carrying operations like:

```text
%17 = LoadGlobal<u8>  game_data::frame_counter   span `game_data::frame_counter`
%18 = Const<u8>       1                          span `1`
%19 = AddWrap<u8>     %17, %18                   span `game_data::frame_counter + 1`
      StoreGlobal<u8> game_data::frame_counter, %19
                                                  span whole assignment
```

The exact enum spelling is not contractual. The important details are that the
width and wrapping operation are explicit, the load occurs before the add and
store, and expression/statement spans survive lowering.

### Structured Z80 Output and Existing Assembler

The compiler backend emits structured items, not arbitrary concatenated text:

```rust
pub enum Z80Item {
    Label(LabelId),
    Instruction(GeneratedInstruction),
    Data(GeneratedData),
}

pub struct GeneratedInstruction {
    pub id: AsmInstructionId,
    pub op: Z80Op,
    pub source: SourceProvenance,
}
```

`Z80Op` only needs variants the compiler can generate. Its renderer produces
one canonical rtvc assembly statement per instruction. Labels and data are
rendered as normal helper-assembler source. Feed the rendered program to
[`assemble_program`](../../src/emulator/asm.rs), then join its per-line
`AssembledLine` metadata back to `AsmInstructionId`.

The example IR above can become structured items such as:

```text
Instruction 41: Ld8(A, Absolute(game_data::frame_counter))
Instruction 42: Add8(A, Immediate(1))
Instruction 43: Ld8(Absolute(game_data::frame_counter), A)
```

The renderer may display:

```asm
    ld a,(game_data__frame_counter)
    add a,1
    ld (game_data__frame_counter),a
```

but instruction IDs and source provenance stay attached to the structured
items; they are not inferred by parsing these display strings.

This staged boundary gives the compiler typed operations and stable provenance
without first rewriting the entire existing text assembler. Extend
`AssembledLine` or add an assembler listing API so every emitted instruction
or data item reports its address, length, bytes, and original rendered line.
Do not parse the displayed assembly back to reconstruct compiler provenance.

If rendering exposes important limitations later, extract a shared structured
instruction encoder from `asm.rs`; do not maintain two opcode tables.

## Code Generation Strategy (Phase 1)

Use a small deterministic backend with explicit value locations and Z80
instruction constraints. Register calls and inexpensive small leaf routines
are baseline requirements, not optional late optimizations. The initial backend
does not need SSA, global graph coloring, or static allocation of function
temporaries into shared global memory.

### Pass Order and Backend Contracts

1. Lower typed expressions into virtual byte/word values and explicit basic
   blocks. Keep loads, stores, calls, and observable effects ordered; retain
   source provenance on every operation.
2. Select legal instruction patterns with declared input/output registers,
   scratch requirements, flags read/written, and memory effects. Keep labels
   symbolic and frame slots abstract.
3. Compute block use/def sets and live-in/live-out sets to a fixed point,
   including loops. Choose compatible register locations across straight-line
   edges and simple loops; reconcile differing predecessor locations with edge
   moves. Crossing a block boundary alone must not force a stack home.
4. Lower ABI argument/return moves, call preservation, and edge stores/reloads.
   Allocate spill slots and finalize frame size after scratch requirements are
   known. If lowering creates more temporaries, include them before finalizing
   the frame rather than silently borrowing an occupied register.
5. Resolve frame addressing, emit necessary prologue/epilogue instructions,
   and verify stack balance, register constraints, and symbol uniqueness.
6. Apply proven local simplifications, render canonical assembler input, and
   assemble. Join final bytes/addresses to provenance, timing, and layout data.

The allocator and instruction selector cooperate through constraints; selecting
an instruction must never assume A, HL, or a scratch pair is freely available.
Both the original value and its location must remain identifiable after spills,
reloads, and ABI moves. Invalid backend state produces a located internal
diagnostic rather than silently generating incorrect code.

### Register Allocation and Frames

Track A, B, C, D, E, H, and L as overlapping resources with BC, DE, and HL.
Writing H invalidates any recorded value in HL; reserving HL reserves both
halves. AF is not a general word-value register. Track flags separately as
short-lived condition results; alternate registers are not allocated initially.
IX is reserved for frames and IY for target/runtime use under both ABIs.
AF, BC, DE, and HL remain caller-saved under register calling as well as
explicit stack calling; preserve IX when used and leave IY untouched.

Prefer existing argument locations, then instruction-compatible free registers.
When pressure requires eviction, prefer a dead value, then a value safely
rematerializable without memory access, then the value with the farthest next
use. Use a fixed tie-break order so identical input produces identical code.
Spill live values to compiler-owned frame slots. Reuse slots only when liveness
proves their lifetimes do not overlap.

Pure scalar locals may remain in registers for their entire lifetime. Locals
whose address is taken need stable storage for their source lifetime. Preserve
register locations across compatible edges and simple loop backedges in Phase
1B. Use explicit moves at joins and spill only for actual register pressure,
addressable storage, or values that must survive clobbers. Stack-based lowering
of every expression/control-flow merge does not meet the baseline quality goal.
Expose spill/frame costs in the listing so users can simplify register-heavy
code when necessary; do not hide them behind a fastcall label.

Create an IX frame only when stack addressing actually needs it. A register-only
leaf with no spills or addressable locals emits no frame. Check all byte offsets
of locals, spills, and stack parameters against indexed-addressing limits;
reject unsupported frames/parameter layouts with a diagnostic. Do not truncate
an offset into an eight-bit displacement. Keep any temporary PUSH/POP balanced
and include it in stack accounting.

### Expressions, Conditions, and Addressing

- Preserve left-to-right evaluation, including the address of an assignment
  destination before its right-hand value. Evaluate an indexed destination
  once; compound assignment later adds exactly one read and one write.
- Prefer A for eight-bit ALU work and HL with DE/BC for word operations, moving
  values only when the selected instruction requires it. Use immediate operands
  where legal and fold only expressions with fully specified C80 semantics.
- Lower comparisons directly to control flow when used as conditions. Use
  unsigned carry/zero tests and correct signed comparisons; subtraction's sign
  flag alone is not a signed less-than test when overflow occurs. Materialize
  a canonical 0/1 only when the boolean is needed as a value. Never retain a
  flags-based condition across an instruction that destroys those flags.
- Implement short-circuit operators as branches. Variable-count shifts use
  bounded lowering consistent with F001's count semantics; constant shifts can
  use specialized instruction sequences.
- Use absolute addressing for known globals when legal. Compute indirect
  addresses in a supported pair, with little-endian word loads/stores and
  explicit scaling by element size. Constant packed-struct sizes may require
  shift/add scaling even while general source multiplication is unsupported.
- Preserve a destination address across right-hand calls or register pressure.
  Address arithmetic wraps at 16 bits; image placement arithmetic must not.

Unknown pointer writes, calls, and raw assembly conservatively invalidate
possibly aliased cached memory values. Flush preceding direct stores that an
indirect access or call could observe. Never optimize `x = 1; *p = 2; return x;`
to return 1. Preserve source-level mutable-memory reads/writes in IR order even
when aliasing analysis could prove an earlier value: a later access is still
observable. Restrict mutable load reuse/store elimination to compiler-private
storage. Raw assembly is also a memory barrier, not only a register clobber.
Interrupt-shared globals receive the same observable reads/writes as other
mutable source memory; handler register preservation is a separate obligation.

### Calls and Interoperability

Evaluate argument expressions left-to-right into tracked values, preserving
earlier arguments across later argument calls. Then perform a parallel move
into the callee's resolved ABI locations. Resolve register-move cycles with a
proven free temporary or spill slot; sequential naive moves can destroy an
argument when registers are swapped. For `@stackcall`, push evaluated arguments
right-to-left and remove slots without destroying the returned A/HL value.

Preserve only live values held in caller-clobbered registers. Dead arguments
need no saves. Do not automatically spill every register argument on function
entry. Return a value directly from its current location when it already matches
the return ABI. Explicit inline assembly supplies external register contracts
and clobber/stack metadata; no ROM wrappers are synthesized. Automatic inlining, tail calls,
and per-function inferred register conventions are deferred initially.

### Labels, Branches, and Layout

Generate assembler-safe unique names from unit/function/block/item IDs, with
optional readable suffixes. Do not rely on case distinctions, concatenated user
names separated only by underscores, or dot-local label scope: the existing
assembler uppercases symbols and has no local scopes. Namespace raw assembly
labels separately and validate its allowed instructions/directives before
rendering; user assembly must not change project placement.

Start with absolute JP for generated control-flow branches. Emit JR or DJNZ only
when their final displacement is proven valid. Later branch shortening must
iterate layout to stability and then reassemble; cross-unit fixed origins and
all instruction-size changes must be included in the check. An out-of-range
generated short branch must fall back to a correct long sequence, not reject an
otherwise valid source function. Keep user-written raw branch range errors
as source diagnostics.

Check final assembled segments against emitted and reserved project ranges.
Report per-function code bytes, frame bytes, and stack bounds separately. Layout
and timing always describe the final emitted instructions, not estimated IR.

### Stack Bound Calculation

For each function, measure maximum additional stack depth below its entry SP.
Include saved IX, frame slots, temporary pushes, argument slots, and preservation
spills. At a call site, add the site's current depth, the two-byte CALL return
address, and the callee's bound; take the maximum over all reachable sites and
non-call paths. Count each allocation once, including argument slots already
present at the call site. Analyze the acyclic C80 call graph bottom-up after
lowering, and include startup/wrapper overhead at the root.

External/ROM calls and raw assembly require verified or declared bounds; unknown
usage remains unknown and cannot be reported as a proven safe reservation.
Add interrupt-entry return-address and handler-save/call requirements to the
interrupted path, with an explicit nesting policy. Interrupt re-entry can defeat
an otherwise acyclic function-call bound. BASIC-owned CPU stack capacity and its
existing interpreter depth also need a target-specific allowance.

### Baseline Quality and Optimization Gates

Phase 1B must produce a two-word register `add` as `ADD HL,DE; RET` under the
proposed HL/DE ABI, with no argument pushes or IX prologue. A byte identity
function should be just RET. These tiny byte/size/timing fixtures complement
semantic execution tests for nested calls and register pressure; they are not
a demand that all source programs match one assembly template.

Phase 1B must also keep simple local counters and pointer walks in registers
across loop backedges when register pressure allows, with no per-iteration stack
traffic. Add a byte-buffer fill fixture and a compare/branch loop fixture;
trivial ADD/RET examples alone do not establish useful generated-code quality.

Phase 1E extends propagation to more complex control flow, eliminates dead
compiler-private spills, and adds proven loop induction/address strength reduction, branch
shortening, and redundant load/move elimination. A rewrite must preserve flags
that remain live, observable access order/count, stack balance, and the union of
its source provenance. Prefer fewer T-states without increasing bytes for the
first local rewrites; record speed/size tradeoffs before adding competing modes.
Do not implement the worked loop's DJNZ optimization until bounds, wraparound,
register liveness, and relative-branch reach are proven.

Every pass retains origin IDs; merged instructions retain all contributing
origins and a primary display span, while eliminated operations have an explicit
no-code mapping. Final source maps must not invent executable breakpoints for
eliminated statements. Compare baseline and optimized execution on the same
fixtures, including signed boundaries, aliases, effects, and nested calls.

## T-State and Size Metadata (Phase 1)

The assembler listing is authoritative for final addresses and bytes. Derive
instruction timing through the existing disassembler metadata in
[`disasm.rs`](../../src/emulator/disasm.rs) initially, using the final address
and emitted bytes. Refactor the timing table into a shared assembler/disassembler
instruction metadata API if round-tripping becomes awkward.

Represent timing as data rather than only display strings:

```rust
pub enum StaticTiming {
    Exact(u16),
    Branch { not_taken: u16, taken: u16 },
    Unknown,
}
```

The assembly view shows timing per instruction from the first integrated
version. Source-level aggregation follows these rules:

- straight-line spans may show an exact sum;
- conditional spans may show a minimum/maximum only when both paths are
  represented completely;
- loops show condition/body cost or cost per iteration, not a misleading
  finite total; and
- calls show local call overhead separately unless a callee cost is known and
  intentionally included.

Do not delay the live assembly view for whole-statement cost analysis. Exact
instruction size, bytes, and timing already provide useful feedback.

For example:

```c
if (x != 0) {
    foo();
}
```

with this possible lowering:

```asm
    ld   a,(x)         ; 13 T
    or   a             ;  4 T
    jr   z,.done       ;  7 T not taken / 12 T taken
    call foo           ; 17 T plus callee
.done:
```

has a 29 T zero path and a 41 T nonzero path plus the callee. The compiler
should report those paths rather than a single misleading total. Similarly,
this loop:

```asm
.loop:
    ld   a,(count)     ; 13 T
    or   a             ;  4 T
    jr   z,.done       ;  7/12 T
    dec  a             ;  4 T
    ld   (count),a     ; 13 T
    jr   .loop         ; 12 T
.done:
```

is described as 53 T per executed iteration plus a 29 T final exit test, not
as a statically bounded total unless the compiler can prove the iteration
count.

## Source Map (Phase 1)

The source map is a first-class part of `CompilationResult`. Preserve this
chain:

```text
SourceSpan <-> AstId/IrId <-> AsmInstructionId <-> logical address range
```

At minimum, each final listing entry records:

```rust
pub struct MappedInstruction {
    pub id: AsmInstructionId,
    pub address: u16,
    pub bytes: Vec<u8>,
    pub text: String,
    pub timing: StaticTiming,
    pub expression_span: Option<SourceSpan>,
    pub statement_span: Option<SourceSpan>,
    pub function: FunctionId,
}
```

Build explicit indexes for:

- source offset/span to generated instruction IDs;
- instruction ID to expression and statement spans;
- address range to instruction ID and source spans; and
- function/global symbols to logical addresses and declared types.

Overlapping source spans are expected. Selection uses the smallest containing
expression span first and falls back to the enclosing statement. The UI must
be able to distinguish instructions with no direct source expression, such as
function prologues, branch glue, and stack cleanup; associate these with an
enclosing statement/function and mark them synthetic.

Initial addresses are logical 16-bit CPU addresses. TVC bank-qualified source
breakpoints depend on the broader debugger address model already listed in
[TODO.md](../../TODO.md#debugger). Until that model exists, Build and Load must
record the active mapping and reject or clearly label source breakpoints that
cannot be represented safely by the current address-only breakpoint set.

## Command-Line Tool (Phase 1)

Add an `rtvc-c80` binary before the editor depends on the compiler. It should:

- compile one C80 source file or a multi-unit project file;
- accept target and unit-origin options;
- write generated assembly and optionally raw binary or the existing
  `rtvc-asm-v1` TOML segment format;
- for mixed projects, include BASIC program bytes in the same segment output;
  this is an addressed memory image, not a CAS or an automatically runnable
  interpreter snapshot;
- emit human-readable diagnostics with file, line, column, and source context;
- optionally emit a machine-readable compiler metadata file containing source
  maps, symbols, bytes, and timings once that schema is stable; and
- return nonzero for compile or assembly errors.

Do not stabilize a JSON/TOML source-map format before the in-process types have
been exercised by the editor. Version any eventual external schema explicitly.

Update the [Development and Testing Skill](../skills/development/SKILL.md) when
the command becomes real.

## Editor and Live Assembly View (Phase 2)

Add compiler/editor state outside `DebuggerUi`, preferably in a focused
`src/ui/source_editor.rs` controller. The debugger consumes compiled maps but
should not own source buffers or compilation scheduling.

Add a `C80 Source` workspace tab. Its initial layout contains:

- a toolbar with New/Open/Save, target, origin, Build, Build and Load, and the
  generated-assembly toggle;
- a multiline `egui::TextEdit::code_editor()` source buffer;
- a small custom C80 syntax highlighter based on `LayoutJob`, without adding a
  full syntax framework initially;
- a diagnostics area with clickable errors and warnings; and
- a toggleable right-hand generated assembly listing.

The assembly listing is read-only and row-oriented rather than another
editable `TextEdit`. Each row can show address, bytes, canonical instruction,
size, and T-states. This makes source-map selection, current-PC highlighting,
and scrolling deterministic.

For example, placing the source cursor inside the addition highlights all three
rows produced for the assignment:

```text
SOURCE                                      GENERATED Z80

game_data::frame_counter =                  2034  3A 00 30  ld a,(3000H)  13 T
    game_data::frame_counter + 1;            2037  C6 01     add a,1        7 T
                                             2039  32 00 30  ld (3000H),a  13 T
```

Clicking address `$2037` selects only the expression
`game_data::frame_counter + 1`; clicking a synthetic prologue row selects the
enclosing function because it has no narrower source expression. If PC is
`$2039`, the final store row and the whole assignment statement receive the
current-execution highlight.

Required interactions:

- source cursor/selection highlights all mapped assembly rows and scrolls the
  first relevant row into view;
- clicking an assembly row selects and reveals the most specific source span;
- diagnostics select their source span;
- the current PC highlights the matching assembly row and source statement;
- the last successful assembly remains visible and dimmed when the current
  source has errors; and
- Build and Load is disabled when the current source has no successful result.

A line-number/breakpoint gutter is valuable but not required for the first
editor slice. Add it with source breakpoint integration rather than building a
decorative gutter that later needs replacement.

### Compilation Scheduling

Use a 150 ms idle debounce as the starting value and make it an implementation
constant, not a user preference initially.

- Native builds compile an owned source snapshot off the UI path and return
  results through a channel tagged with a monotonically increasing revision.
  Discard results older than the newest requested revision.
- `wasm-full` may compile synchronously after the debounce while programs are
  small; measure UI frame time before introducing web workers.
- Explicit Build bypasses the debounce.
- Compilation must not borrow emulator state or egui objects.
- Keep the last successful `CompilationResult` separate from diagnostics for
  the current revision.

Do not call this implementation incremental compilation. Recompile the whole
small translation unit until measurements justify more complexity.

### Persistence Boundary

Do not store source text or project file paths inside
`rtvc-workspace.json`; that file describes dock layout. The existing
[developer-project backlog](../../TODO.md#developer-workspace) is the eventual
home for source files, open buffers, target/origin settings, and source
breakpoints.

Before project management exists, support one session buffer plus ordinary
Open/Save behavior. Native builds may remember an active path only for the
running session. Browser builds use the existing supported file-dialog/download
patterns and must not pretend to persist an inaccessible host path.

## Build, Load, and Debugger Integration (Phase 2)

Build and Load uses assembled segments, not the rendered listing text:

1. Require the emulator to be paused or explicitly pause it.
2. Validate that every segment fits the 16-bit address space and is writable
   through the active machine's mapped-memory interface.
3. Write all segments as one operation from the UI's point of view. If a write
   fails validation, write nothing.
4. Record the compilation revision, target, segment ranges, and current machine
   mapping as the active loaded program.
5. Set PC to the explicitly selected freestanding startup entry only through an explicit
   Run/Set Entry action; Build and Load alone should not silently start code.
   Callable routines require their documented caller/wrapper environment rather
   than direct PC assignment to an ordinary function.

Add source-level debugger operations after address mapping is reliable:

- PC to source highlighting while paused and while stepping;
- source breakpoint toggle mapped to the first executable instruction of a
  statement;
- source step that continues until PC enters a different mapped statement,
  with a bounded instruction count and normal breakpoint/interrupt handling;
- assembly instruction step through the existing debugger path; and
- symbol exposure so compiler globals/functions can be navigation targets.

Source-level stepping must define behavior for inline assembly, synthetic
prologues, calls into code without source, interrupts, and optimized spans.
Implement PC highlighting and source breakpoints before source stepping.

## Symbol and Typed Memory Integration (Phase 2)

Compiler symbols should record logical address, size, type, declaration span,
and scope. Initially expose them to navigation and the existing memory view.
A typed variable inspector for structs/arrays is a later extension and should
consume compiler layout metadata rather than duplicate type interpretation in
the debugger.

Do not merge compiler symbols into the immutable ROM symbol database. Treat
them as active developer-program symbols and eventually persist them through
the developer project.

## Diagnostics and Failure Behavior

Diagnostics contain severity, stable code, message, primary span, and optional
related spans/notes. Cover at least lexical, syntax, duplicate-name,
unresolved-name, type, constant-range, stack-frame, inline-assembly, assembler,
and load-validation failures.

An assembler error in generated code is a compiler/backend failure unless it
originates in an inline assembly block. Generated-code failures should include
the related source span and generated assembly row so they are actionable
without exposing an internal panic.

The compiler must not emit a partially loadable program after an error.
Warnings do not disable Build and Load unless explicitly documented.

## Implementation Phases

### Phase 1: Standalone Compiler

Phase 1 delivers the complete non-UI compiler library and CLI. Implement it in
the following internal milestones; these are not separate product phases.

#### Phase 1A: Language Contract and Frontend

1. Choose the final language name/file extension and write a checked-in grammar
   and semantic examples as compiler tests.
2. Add source files/spans, located tokens, diagnostics, lexer, parser, AST, and
   error recovery.
3. Implement scopes, scalar and basic pointer types, address/dereference and
   pointer indexing, fixed scalar arrays/indexing, prefixed strings, constants,
   compile-time `sizeof(T)`, function signatures, explicit conversions,
   definite assignment, and typed expression/statement validation. Cover the
   [frozen type rules](#f001--expressions-types-and-definite-assignment) with
   compact semantic fixtures.
4. Add unit interfaces, `pub` exports, imports, qualified lookup, and call-graph
   recursion rejection.
5. Document evaluation order, overflow, conversions, and rejected C syntax.

Exit criterion: valid scalar programs produce a typed representation;
incomplete and invalid editor-like inputs produce bounded, located diagnostics
without panics.

#### Phase 1B: IR, ABI, and First Executable Programs

1. Add typed IR and lowering for scalar globals, basic pointers, fixed
   arrays/indexing, prefixed strings, locals, expressions, functions, calls,
   `if`, `while`, and `return`.
2. Implement the stack ABI, frame layout, labels, control-flow validation, and
   structured Z80 items following the [code generation strategy](#code-generation-strategy-phase-1).
   Add block liveness, overlapping-register tracking, constrained instruction
   selection, deterministic spills, and absolute branches as the baseline.
3. Define and implement the default register ABI, including register-only
   argument passing and frame-free small leaf routines, plus explicit
   `@stackcall`. Validate nested calls and left-to-right argument
   evaluation under both conventions.
4. Add project placement, loadable and constants-only units, namespaced private
   symbols, cross-unit references, and overlap validation.
5. Render canonical helper assembly, assemble all units together with
   `assemble_program`, and return final segments and symbols.
6. Add `rtvc-c80` with single-file/project assembly and binary/TOML outputs.
7. Add project stack reservation and exported bounds plus an explicit assembly
   startup fixture; stack setup and entry/return behavior are manual.
   Add explicit register in/out inline assembly and manual entry/exit fixtures.
   Define the project assembly-unit interface described under
   [manual assembly units](#f004--project-assembly-units-reservations-and-output).
8. Report final code/data sizes, compiler stack bounds, and external allowances;
   validate explicit stack/buffer reservations along with emitted segments.

Exit criterion: command-line fixtures compile, assemble, load through existing
debugger tooling, run to completion, and produce expected memory/register
results in a `FakeBus` or machine test.

#### Phase 1C: Provenance, Listing, and Timing

1. Carry expression/statement provenance through IR and Z80 items.
2. Extend assembler listing metadata and join IDs to final addresses/bytes.
3. Produce bidirectional source maps and compiler symbols.
4. Add structured timing metadata and per-instruction size/T-state display data.
5. Test one-to-many, many-to-one, synthetic, branch, and data-item maps. Add
   inline-assembly map cases for the Phase 1B register in/out interface.

Exit criterion: every generated byte is owned by a data item or mapped
instruction; every displayed instruction maps back to source or is explicitly
synthetic.

#### Phase 1D: Complete the Planned Language

1. Add richer array initializers, structs, layout metadata, field access, and
   pointer-to-struct access on top of the basic pointer operations from the
   first executable milestone.
2. Add `for`, `do/while`, compound assignment, increment/decrement, and useful
   constant/data declarations.
3. Extend Phase 1B inline assembly from measured examples, retaining explicit
   registers, outputs, clobbers, and memory effects.
4. Add typed `cpu::in(u8) -> u8` and `cpu::out(u8, u8) -> void` port intrinsics
   with the byte-port and observable-effect contract above. Explicit inline
   assembly remains available for other hardware/ROM operations.
5. Add the mixed BASIC/C80 project mode using the existing BASIC tokenizer,
   symbolic callable references, and generated BASIC memory-reservation constants.
   Supply explicit BASIC/assembly setup examples rather than automatic wrappers
   or interpreter-memory management.
   Emit one `rtvc-asm-v1` TOML containing C80/ASM segments and the BASIC payload
   at its configured origin. Execute that serialized artifact with explicit
   interpreter setup in the E11 test; cassette packaging remains deferred.

Exit criterion: representative TVC routines can express structured data,
loops, function calls, hardware access, and optimized inline assembly without
unsupported compiler workarounds.

#### Phase 1E: Code Quality and Static Cost

1. Add local constant folding, dead-block removal, branch simplification, and
   conservative peephole passes that preserve provenance.
2. Extend baseline cross-block/loop register allocation to more complex control
   flow and eliminate unnecessary compiler-private loads/stores.
3. Improve frame/spill elimination beyond the required baseline and calls within the
   already-defined register and explicit stack ABIs.
4. Add source-level static cost summaries where control flow permits honest
   values.

Optimization is successful only when byte/T-state tests improve without
breaking source provenance or debug metadata. Optimization depth need not block
the initial Phase 1 release once the generated code is correct and transparent.

Phase 1 is complete when `rtvc-c80` can compile the planned language from a
single file or project, emit normal assembly/loadable segments, and return the
diagnostics, symbols, bidirectional source map, bytes, and static timing data
that Phase 2 will consume. No editor code is required for this milestone.

### Phase 2: Editor and Debugger Integration

Phase 2 adds UI and emulator interaction without moving parsing, compilation,
mapping, or timing logic into egui/debugger modules.

#### Phase 2A: Developer Workspace Editor

1. Add the C80 Source pane, syntax highlighter, diagnostics, and file actions.
2. Add idle-debounced revisioned compilation without blocking native emulation.
3. Add the toggleable assembly listing with bidirectional selection.
4. Preserve the last successful listing when current source is invalid.
5. Add native and `wasm-full` UI tests or focused controller tests for revision
   ordering, stale-result rejection, and selection mapping.

Exit criterion: editing a representative program keeps the emulator responsive
and updates the correct assembly rows and diagnostics predictably.

#### Phase 2B: Load and Debug

1. Add target/origin settings and transactional Build and Load validation.
2. Add current-PC highlighting and compiler symbol navigation.
3. Add source breakpoints, including clear behavior for unmappable/banked code.
4. Add source-level stepping after highlighting and breakpoints are stable.
5. Connect active compiled-program state to future developer-project
   persistence without coupling it to dock layout.

Exit criterion: a user can compile, load, breakpoint, run, and step a C80
program while source and assembly views track the emulator PC.

#### Phase 2C: Measured Runtime Cost

Join instruction-trace execution counts and T-states to source-map IDs for
measured hot spots. Keep compiler-provided static cost and emulator-measured
runtime cost visibly distinct. This is a Phase 2 extension, not part of the
standalone compiler milestone.

## Validation Strategy

### Phase 1 Compiler Tests

- lexer/parser golden tests, including incomplete input and recovery;
- scope, type, conversion, width, signedness, and constant-range tests;
- IR snapshots for evaluation order and control flow;
- ABI tests for arguments, returns, frames, nested calls, and register clobbers;
- register-pair/byte overlap, parallel-move cycles, spill pressure, and loop
  live-in/live-out tests;
- canonical frame-free register leaf code, long branches beyond JR range,
  symbol case/scope collisions, and frame displacement boundaries;
- aliasing and memory-barrier execution tests, including indirect mutation of
  directly accessed locals/globals and argument evaluation with nested calls;
- stack-bound tests checked against execution high-water measurements for known
  call graphs, with explicit unknown external bounds and interrupt allowances;
- mixed BASIC/C80 symbol resolution, exact tokenized size, reserved-range
  overlap, memory-limit setup, and BASIC USR execution tests;
- generated assembly that round-trips through the existing assembler;
- execution tests using `FakeBus` and the Z80 core for representative programs;
- inline-assembly acceptance, diagnostic, clobber, and source-span tests;
- source-map coverage tests proving every emitted byte/listing row has known
  provenance;
- compiler CLI exit codes, project builds, and output formats; and
- existing assembler/disassembler and Z80 FUSE regression suites.

Prefer semantic assertions over large fragile text snapshots. Use small
canonical assembly snapshots where readability is the behavior under test.

### Phase 2 Editor and Debugger Checks

- transactional mapped-memory loading and rejection of ROM/overflow ranges;
- source/assembly selection in both directions;
- current-PC highlighting and breakpoint address resolution;
- stale live-compilation result rejection;
- native and `wasm-full` compilation; and
- existing debugger and workspace regression tests.

During Phase 1, measure representative compile latency and set a practical
initial target such as under 50 ms for a few-thousand-line single-file program
on a development machine. During Phase 2, measure UI frame impact separately.
Replace both targets with measured repository fixtures rather than treating
them as hard language guarantees.

## Documentation

When the first language slice is stable:

- add `info/c80.md` as the authoritative language, ABI, inline assembly,
  compiler CLI, generated metadata, and editor workflow reference;
- update [info/rtvc.md](../../info/rtvc.md) for architecture, UI integration,
  build/load behavior, persistence, and WASM boundaries;
- update [info/assembler.md](../../info/assembler.md) for any new listing or
  structured metadata API that also affects assembler users;
- update [README.md](../../README.md) with the concise user workflow;
- update the [Development and Testing Skill](../skills/development/SKILL.md)
  with compiler commands and validation; and
- extend the [Hungarian documentation tree](../../info.hu/) once terminology
  and syntax are stable rather than translating a rapidly changing draft
  language.

## Non-Goals

- ISO C compatibility or compiling existing C codebases unchanged.
- Floats, wider integers, unions, bitfields, enums, function pointers, varargs,
  recursion, heap allocation, runtime global initialization, implicit C
  promotions, or C's full declarator and implicit-conversion rules.
- LLVM, SSA-based global optimization, a general linker, object files, or a
  macro preprocessor.
- IDE-scale editing features such as multicursor, folding, semantic rename,
  language-server protocol, or very large-file rope storage.
- Hiding generated assembly or promising optimal Z80 output.
- Whole-program worst-case execution-time analysis.
- Runtime profiling before static maps and instruction-trace integration are
  reliable.
- Making compiler/editor state part of global workspace-layout persistence.

## Frozen Implementation Contracts

The following contracts resolve C80-F001 through C80-F006 under the user's
instruction to finish the decisions before implementation resumes. They are
normative, not suggestions requiring another approval. They supersede earlier
provisional examples where wording differs. Tests validate these decisions;
the absence of an implementation/test is not an unresolved design choice.
Module layout, internal enum names, diagnostic numbering beyond existing codes,
and test organization remain implementer choices.

### F001 — Expressions, Types, and Definite Assignment

Use the E01 parser's precedence for accepted operators. From weakest to
strongest: assignment; ||; &&; |; ^; &; ==/!=; </<=/>/>=; <</>>;
+/-; prefix; postfix/call/index/member. Assignment and prefix operators associate
right-to-left; other binary operators associate left-to-right. Multiplicative
syntax may be retained for a focused unsupported-operation diagnostic but never
executes. Add missing accepted forms to the parser in their owning increment;
E01's current syntax coverage is not a frozen language limitation.

| Operation | Operand types | Result and rule |
| --- | --- | --- |
| +, -, &, ^, \| | Same integer type, after contextual literal typing | Same type, fixed-width bits; addition/subtraction wrap |
| unary +, -, ~ | Integer | Same type; negate wraps, including signed minimum |
| ==, != | Same integer/bool type or same pointer type | bool |
| <, <=, >, >= | Same integer type | bool, signedness of operands |
| !, &&, \|\| | bool only | bool; && and \|\| short-circuit |
| <<, >> | Integer left, u8 count | Left type; no promotion |
| assignment | Writable scalar/reference lvalue and matching value | Assigned value, no extra memory read |
| pointer +/- integer | ptr<T> and any integer offset | ptr<T>, scaled offset with 16-bit address wrap |
| pointer[index] | ptr<T> and integer index | T lvalue |
| array[index] | Fixed array and integer index | Element lvalue; known negative/out-of-range index is an error |
| sizeof(T) | Complete supported value type | u16 compile-time byte count; void/incomplete/oversized type is an error |

Conditions in if/while/for/do-while require bool. Use bool(value) explicitly for
an integer/pointer condition. No integer truthiness or bool arithmetic. bool
loads from arbitrary mutable byte memory interpret zero as false and any other
byte as true; stores and scalar/ABI results use exactly 0 or 1. Such loads remain
observable even if the resulting boolean is predictable.

Integer literal typing is local and deterministic:

1. An assignment initializer, return, or function parameter supplies its exact
   expected type to untyped integer literal expressions. Arithmetic/bitwise
   expressions propagate an integer expected type to their untyped operands.
   Comparison operands use the type of a typed peer; their bool result context
   does not give an integer operand a bool type.
2. A typed operand fixes the type of its otherwise untyped peer. Two differing
   typed operands are errors even if one currently contains a small constant.
   Character literals are typed u8; true/false are typed bool.
3. With no contextual or typed-peer integer type, a literal-only integer subtree
   uses u16, or i16 if it contains a syntactically negative integer literal.
   Unary minus directly on an integer literal, allowing parentheses, forms that
   signed literal before range checking. Thus -32768 is representable as i16.
   If both comparison operands are untyped, apply this rule to them together.
4. Each literal must fit the selected integer type before arithmetic. Operations
   then use that type's wrapping semantics even during constant folding. Integer
   literals do not implicitly become bool or pointers.
5. An explicit cast first types its argument independently (without using the
   destination as an expected type), then converts it. Thus u8(1000) is allowed
   and gives 232, while u8 x = 1000 is a range error. u8(65536) is a source
   literal range error; C80 does not introduce wider source integers for casts.

Examples: u8 x = 255 + 1 gives 0; 255 + 1 without context is u16 value 256;
i8 x = -128 is valid; i8 x = 128 is an error; i8(128) is -128;
-128 < 1 is a signed i16 comparison and true; bool b = 1 is an error.
A typed constant never loses its type to avoid an explicit conversion.

Integer casts preserve bits at the same width, discard high bits on narrowing,
and sign-extend a signed source or zero-extend an unsigned source on widening,
regardless of destination signedness. bool(integer/pointer) tests nonzero;
integer(bool) gives 0/1. ptr<T>(u16) and u16(pointer) are explicit bit-preserving
conversions. Other integer widths must first be explicitly converted to u16 for
a pointer cast. ptr<U>(ptr<T>) explicitly reinterprets the same address.
No ptr<void>, str/integer, or str/pointer conversions are supported.

Shift count literals have u8 context; typed counts must be u8 or explicitly
cast to it. Count zero returns the left value, still evaluating both operands
and any observable accesses. Counts at least the left width produce zero for
left shift/logical right shift, or all sign bits for signed right shift.
This is defined repeated-shift behavior, not a masked count or undefined
behavior; clamp lowering at the width. Negative literal counts and 256 are
range errors. Signed right shift is arithmetic; left shift wraps at width.

Evaluate ordinary operands and call arguments left-to-right. For assignment,
evaluate and retain the destination address before evaluating the RHS, then
store once. Compound assignment evaluates the address, reads its old value,
evaluates the RHS, and writes once; its result is the stored value without
rereading. Prefix ++/-- returns the updated value; postfix returns the saved old
value. These operations support integer and pointer lvalues; pointer steps are
one element. Accepted compound operators are +=, -=, &=, |=, ^=, <<=, >>=;
pointer compound operations are only +=/-=. No *=, /=, or %=.
Array/struct whole-value assignment is not implicitly added by these rules.

Local declaration without initialization emits no initialization and leaves its
value unassigned. Parameters are assigned on entry. Analyze definite assignment
with path intersections over the CFG, including backedges and short circuiting;
a loop that may run zero times does not initialize a value afterward.
Assignment on unreachable paths does not satisfy a reachable read. Assembly out
assigns on normal block exit, while in/inout requires prior assignment.
Every reachable non-void function exit needs return value; void fallthrough
emits return. Function prototypes without bodies are rejected in version one:
all C80 calls bind to project-defined functions, and external calls use assembly.

### F002 — Frozen Register and Stack ABI

The proposed register allocation is adopted as ABI v1. Ordinary functions use
register calls; @stackcall selects stack slots; @fastcall is an optional alias
for the default. Accept attributes before or after pub, canonically
@stackcall pub u16 f(...). Reject duplicate/conflicting or unknown attributes.

Reserve HL, DE, BC for word/pointer/str arguments in their relative declaration
order. Then allocate byte/bool arguments in relative declaration order to
A, C, B, E, D, L, H, skipping halves of reserved pairs. Evaluate arguments in
source order independently of this assignment. Reject a signature that does
not fit; suggest @stackcall. No hidden stack overflow arguments.

Returns: byte/bool in A, word/pointer/str in HL, void has no result. AF/BC/DE/HL
are caller-clobbered; IX is preserved and IY untouched. Alternate registers are
not compiler-allocated and must be preserved by assembly that temporarily uses
them. Flags are not an implicit additional C80 result.

Examples: (u8,u8) uses A,C; (u16,u16) uses HL,DE;
(u8,u16,u8) uses A,HL,C; (u16,u16,u16,u8) uses HL,DE,BC,A.
The last signature plus another byte is rejected. Return-location overlap
with an argument is legal.

@stackcall uses caller-cleaned, right-to-left two-byte slots. u8 zero-extends,
i8 sign-extends, bool uses 0/1. With saved IX and IX established at that SP,
parameter bytes begin at IX+4; locals/spills use negative offsets.
No hidden aggregate return pointers or stack parameters.
Test ABI moves/spills/nested calls during E03/E05; test completion does not
require a further ABI approval.

### F003 — Pointers, Strings, Aggregates, and Initialization

Pointers are plain 16-bit machine addresses. Pointees may be bool, integer,
another supported pointer, or a complete struct (from E10); not void, str, or
an array type. &scalar and &array[index]/&field are supported; &array and
address-taking of constants/string payloads are rejected. There is no array
decay. Pointer offsets may be any integer type: interpreting a signed offset
and scaling/wrapping the address is the specified addressing operation, not
an implicit general-purpose integer conversion.

C80 has no borrow checker or automatic lifetime management. Taking a local's
address gives it stable stack storage for its source lifetime; preserve it
through that lifetime and do not alias it with private spill slots.
Pointers can escape; the programmer is responsible for not using them after
the object's lifetime or mapping ends. Diagnose directly returning &local with
a warning, not an error. Do not introduce C-style undefined-behavior optimizer
assumptions: dereferences remain real ordered machine-memory operations, but
the compiler does not promise that a departed local's former bytes retain a
value. Test the diagnostic and physical addressing, not stable dangling data.

Global scalars/arrays/structs without an initializer emit zero bytes in place.
Their initializers must be compile-time expressions, including allowed symbolic
addresses of static storage. Resolve address fixups during final assembly and
detect constant/type-layout dependency cycles. Scalar locals are never zeroed
implicitly. Local const requires a compile-time initializer and has no storage.

str globals own prefix+payload immutable bytes and require a string literal
initializer. str local/parameter/return values are two-byte references to such
static data or anonymous literals. Local reference assignment/rebinding and
return are supported; replacing a global owned str or mutating bytes is not.
No user-created str from arbitrary pointers, str comparison, arrays of str, or
str struct fields in v1. sizeof(str) is 2 (reference representation); symbol
metadata for an owned string reports its actual 1+payload storage size.
str.len is u8 and indexing returns u8. Check constant bounds only when length
is statically known; otherwise index unchecked. Static immutability permits
folding .len/payload values. An arbitrary pointer write into immutable string
storage violates that declaration's contract; no promise of observing it through
str reads. Mutable pointer reads themselves are always observable.

Arrays and struct objects are global/static in v1; scalar, pointer, and str
reference locals are supported, but local aggregates/by-value aggregate
arguments/returns/copies are not. Arrays are one-dimensional with length 1..65535,
subject to total size fitting 65535 bytes. Struct fields may be scalar/pointer,
a nested struct, or a fixed array of supported elements. Layout is packed in
declaration order. Reject empty structs and recursive by-value layouts; recursive
pointer types are allowed. Field . and pointer -> produce ordinary lvalues.
sizeof accepts a named array type only if later type-alias syntax exists;
v1 sizeof(T) covers scalar/pointer/str/named struct types, not expression syntax.

Static aggregate initializers use positional braces, may nest, and may omit
trailing elements/fields, which zero-fill. Reject excess initializers and
designators. Whole aggregates stay at declaration position within their unit.
No source multiplication is needed for compiler-generated address scaling.

### F004 — Project, Assembly Units, Reservations, and Output

Manifest version is 1; absent version defaults to 1. Reject unknown versions,
unknown fields, duplicate unit names, unreadable files, and invalid ranges.
Unit names and source identifiers are case-sensitive ASCII names; reserve
project as the built-in namespace. Resolve every path relative to the manifest.
Defaults: filename rtvc-c80.toml, target generic-z80, unit kind c80. Accepted
targets are generic-z80, tvc, zx82. target is a code/validation profile, not
an instruction to change emulator mapping. E11 BASIC requires tvc.

~~~toml
version = 1
target = "tvc"
entry = "boot::start"       # optional; an address, never synthesized startup

[stack]
base = 0xB800
size = 0x0800               # initial SP symbol is C000H; first PUSH uses BFFE/BFFF
interrupt_allowance = 0     # declared additional bytes for manual IRQ use

[[reserve]]
name = "screen"
base = 0x8000
size = 0x3800               # ends at B800H, adjacent to the stack

[[unit]]
name = "boot"
kind = "asm"
path = "src/boot.asm"
origin = 0x2000
exports = ["start"]
stack_extra = 64           # programmer-declared bound, including called routines

[[unit]]
name = "main"
path = "src/main.c80"
origin = 0x2200
~~~

Reservations do not establish a mapping. Increasing screen size to 0x4000 in
this example overlaps the stack and is rejected. [stack] is optional for callable/library
builds. When absent, report computed requirements but do not claim an allocated
safe stack. interrupt_allowance is a caller-declared bound for all allowed
interrupt nesting, including interrupt return addresses and saved registers;
omission means unknown, not zero. Explicit zero means the programmer asserts no
additional interrupt stack usage. Reserve names must be unique; use
@{project::reserve_NAME_base}, _size, and _end symbols for named reservations.

Each [[unit]] has name/path/kind and requires origin if it emits bytes. c80 units
cannot use exports or stack_extra; pub controls exports. asm exports lists labels,
default empty. stack_extra for a standalone ASM entry is an optional declaration
of total additional depth below entry SP over all its own paths/calls; omitted
means unknown. The initial unit-level declaration applies to every exported entry.
The declaration is trusted programmer metadata, not a compiler proof.

Preserve C80 top-level declaration order and manual ASM order; no ORG,
BASIC_START, imports of files, or other placement-changing directives inside
either kind. ASM units support existing encoder instructions and EQU/DB/DW/DS
and aliases. Their labels are case-insensitive within that unit as in the helper
assembler; exported spelling is exactly the manifest name and must identify an
existing local label. Namespace all private labels and reject local case collisions.
ASM code may control SP/IX/IY, return instructions, and interrupts without generated
prologues. Its declared entry/export is an address, not a typed C80 declaration.

Use @{unit::export} for shared references inside ASM code/expressions. Replace
them with generated assembler labels for functions/data or numeric constants as
appropriate, before final assembly. Do not substitute in quoted literals or
comments. No includes/macros or arbitrary code expansion. C80 uses ordinary
import/qualified names for C80 declarations; calling an ASM export is explicit
inline assembly, not an implicit typed function.

Built-ins project::stack_base, stack_size, stack_top, stack_end are compile-time
u16 values when representable. stack_top is (base+size) modulo 65536; stack_end
has no C80 u16 value if it is 65536 and is then an explicit range diagnostic when
referenced from C80. ASM substitutions may emit layout integers up to 65536,
with operand range validation left to the assembler. Base/size arithmetic for
reservations always uses a wider host integer: base 0..65535, size 1..65535,
base+size <=65536. These allocations emit no bytes. Two ranges may be adjacent
but never overlap; BASIC program bytes are the sole contained allocation inside
their declared BASIC region, described below.

Link-time checks cover logical ranges under the configured common mapping;
bank-overlaid projects and arbitrary runtime page safety are out of scope.
The programmer must keep live code, data, and stack accessible. Do not infer
reservations from integer pointer values.

A stack report has additional_bytes: optional nonnegative integer, provenance:
proven/declared/unknown, plus frame_bytes and local_peak_bytes for C80 functions.
Proven means all contributing usage was derived from compiler lowering; any
trusted external bound makes an otherwise finite result declared; an unbounded
contribution makes it unknown. Project totals include explicit root overhead and
interrupt allowance once, not once per C80 call. Known bounds exceeding the
reserved stack are errors. Unknown bounds produce a warning and valid code;
they do not certify safety or block unrelated compilation.

CLI: rtvc-c80 build INPUT [--target TARGET] [--origin ADDRESS]
[--emit-asm PATH] [--emit-segments PATH] [--emit-bin PATH].
A .c80 input uses its file stem as unit name (require an ASCII identifier), with
--origin required if it emits bytes. A .toml input is a manifest; --origin is
invalid there and --target must agree if supplied. At least one output flag is
required; diagnostics go to stderr; exit 0 for success including warnings,
nonzero for parse/type/link/IO errors. --emit-map is deferred: maps are mandatory
library data, not a prematurely frozen external format.

Segment output uses rtvc-asm-v1; ASM output is normal resolved helper assembly.
Here "complete loadable image" means the complete collection of addressed
emitted bytes, serialized by `--emit-segments` as `rtvc-asm-v1` TOML. E07 covers
C80/ASM programs; E11 adds the BASIC payload to that same image. No CAS output,
CPU register snapshot, automatic startup, or interpreter initialization is
implied by this term.

For a mixed project, append the exact tokenized BASIC payload, including its
final program terminator, as a segment beginning at `[basic].origin`. Do not
emit its entire reserved workspace as zero padding. Preserve the ordinary
segment-format fields and make `project::basic_base` and
`project::basic_program_size` available in the exported symbols so consumers
can identify the BASIC byte range even if adjacent ranges are coalesced.
Keep the BASIC allocation as a contained subrange of its own reservation;
all C80/ASM and other reserved ranges must remain disjoint from that reservation.
Do not reject BASIC for overlapping the workspace that intentionally owns it.

For mixed `--emit-asm`, render the same BASIC bytes using ORG/DB so reassembly
reproduces the entire segment image. The raw-binary contiguity rule below also
includes BASIC bytes; it does not silently omit them. No separate BASIC CAS
file or new container schema is required for E11.
Raw binary requires a contiguous union of emitted ranges; coalesce adjacent
segments, reject gaps/multiple separated images, and never implicitly pad or
concatenate. Assembly and segment outputs retain origins and symbols.
Compile and validate all requested outputs before writing. Render to temporary
siblings first, then replace destinations; on a replacement failure return an
IO error naming any outputs already replaced. Do not claim cross-file transaction
atomicity. Compiler errors do not replace existing outputs. CLI integration tests
must verify these behaviors and discover the actual test command.

In-process metadata remains allowed to evolve. It must own the source snapshots,
build identity, diagnostics, symbols, segments, maps, size/timing, and stack report.
Success contains the complete valid program; failure contains diagnostics and
non-loadable partial analysis only. Internal type naming is an implementation
choice; no separate approval for a Rust schema is required.

### F005 — Explicit Inline Assembly Contract

Use this grammar (clause order is source order; a trailing comma is allowed):

~~~text
asm [ "(" clause { "," clause } ")" ] "{" assembly-source "}"
clause := "in:" register "=" expression
        | "out:" output-register "=" lvalue
        | "inout:" register "=" lvalue
        | "clobber:" name { "," name }
        | "stack:" nonnegative-integer-literal
register := a | b | c | d | e | h | l | bc | de | hl
output-register := register | carry | zero
clobber-name := register | flags | memory
~~~

A comma followed by a clause keyword and colon starts a new clause; other
comma-separated names continue clobber. Keywords/register names are lowercase
in the C80 header; ASM body retains helper-assembler case-insensitivity.
Input/output width must match its register: bytes for single registers, words/
pointers/str references for pairs. Literals get register-width context (u8/u16).
Flags out require bool destinations and capture carry/zero without inversion.
inout is shorthand for entry read plus exit assignment to the same lvalue.
No I/O operand for SP/IX/IY/alternate registers; standalone ASM handles those.

Input register resources cannot overlap another input; outputs cannot overlap
another output. Input and output may overlap each other (replacement), including
a pair and its halves across the boundary. inout occupies both sets. Reject a
clobber overlapping an output or duplicate clobber resource; an input may also
be clobbered. carry/zero outputs may coexist and may coexist with flags clobber:
the flag outputs describe the final flags, while clobber invalidates prior flags.
All writes to general registers must be listed as outputs/inout/clobbers in an
operand-bearing block. Registers omitted by a complete explicit contract are
preserved by the block; an opaque call's contract is the programmer's obligation.
Plain asm { ... } conservatively clobbers AF/BC/DE/HL and memory.
Any block is a conservative memory barrier regardless of whether memory is
listed. A flags-only clobber does not silently cover A.

Evaluate header expressions and destination addresses left-to-right once before
entering the block. For inout, read its old value at that position. Before any
output writeback, capture all result registers and flag outputs without destroying
others. Write destinations in clause order. bool register outputs normalize to
0/1 at the C80 boundary. Supplying outputs does not magically initialize memory
written through an input pointer for definite-assignment purposes.

Examples:

~~~c
u8 result;
asm(in: a = value, out: a = result, clobber: flags) {
    inc a
}
bool carry_set;
asm(in: hl = left, in: de = right, out: hl = sum,
    out: carry = carry_set, clobber: flags) {
    add hl,de
}
asm(in: hl = src, in: de = dst, in: bc = count,
    clobber: hl, de, bc, flags, memory) {
    ldir
}
~~~

ASM locals/branches are namespaced per block. Allow normal instructions,
local labels, local EQU, balanced PUSH/POP, and returning CALL/RST. Reject RET/
RETI/RETN or JP/JR to outside the block, indirect jumps, ORG/BASIC_START,
DB/DW/DS/raw-byte directives, and assembler expressions that escape ordinary
control-flow validation. Calls can use literal addresses, local subroutines only
if representable without forbidden exits (normally use standalone units), or
@{unit::export}; they must return to the block continuation. RST is an explicit
returning external call and requires the same stack/clobber declaration.
Inline payload bytes after RST are not supported in this first form; use an
explicit standalone assembly routine for such firmware protocols.

Reject explicit SP adjustments and unsupported writes to IX/IY/alternate
registers inside ordinary inline blocks. PUSH/POP IX/IY for balanced preservation
is allowed only if the original value is restored; use conservative validation,
not guesses through external calls. The programmer must preserve these registers
through called external code. Net SP displacement must be zero at every normal
exit. For straight/local-branch PUSH/POP, verify compatible stack heights at
merges and no unbalanced cycle; calls require stack:N to claim a bound.

stack:N declares the maximum additional hardware stack bytes used inside the
block, including CALL/RST return addresses, saves, and all nested callees.
A statically evident peak above N is an error; otherwise N is a trusted bound.
Omission on a block with opaque calls gives unknown usage and a warning, not a
compile error. Pure non-calling blocks may have proven bounds from validated
stack effects. Inline assembly preserves observable memory ordering, even when
the programmer's body contains writes not expressible in C80.

### F006 — Minimal BASIC Linking and Target Fixture

Support [basic] only for target="tvc", at most once:

~~~toml
[basic]
path = "src/main.bas"
origin = 0x4000
size = 0x8000
~~~

The half-open region [origin,origin+size) reserves BASIC program plus its dynamic
workspace/evaluation stack, not C80's CPU stack. Enforce ordinary range checks.
Tokenized bytes occupy the start of this region; require payload_size+0x100 <=
size as the BASIC 1.2 minimum gap check, without claiming that it budgets arbitrary
arrays/variables. All other units/reservations must be disjoint. Export
project::basic_base, basic_size, basic_end, basic_himem (end-1) and
basic_program_size. basic_end=65536 follows the same wide-substitution restriction
as stack_end. Do not allow C80 layout/initializers to depend on basic_program_size,
which is determined only after C80 addresses and BASIC substitutions.

Use the same @{unit::export}/@{project::symbol} markers in BASIC expression
positions. Resolve to unsigned decimal integers after final C80 assembly.
Unknown/malformed active markers are errors. Leave markers in quoted strings,
REM/! tails, and DATA fields literal; resume normal substitution after an
unquoted DATA colon. Use the tokenizer's lexical rules for quote/comment/data
boundaries; do not perform unrestricted textual replacement. Marker spelling is
case-sensitive like C80; normal BASIC case handling follows the existing tokenizer.

The target fixture is TVC BASIC 1.2 with the checked-in D4/D3/D7 ROMs, standard
64K mapping. Use C80 @fastcall pub i16 echo(i16 value) returning value, then
a BASIC line LET R=USR(@{math::echo},42), followed by a visible/result-memory
assertion and another BASIC statement proving interpreter continuation.
Also test -1 and signed-boundary inputs/results. This contract passes one signed
16-bit integer in HL and receives HL as signed 16-bit, matching the ROM's USR
path. No arbitrary multi-argument marshaling or generated BASIC wrapper.

For reservation setup, use manual BASIC LOMEM to establish the selected program
base before installing C80 bytes below it. The fixture may type/enter the BASIC
source after LOMEM through the existing emulator command/keyboard path; it must
not invoke a tape injector that resets the program origin afterward. With the
example BASIC region 4000H..BFFFH, place C80 at 3000H, use the existing BASIC
CPU stack unchanged, and install C80 only after relocating/entering BASIC.
HI-MEM remains BFFFH in this selected fixture. Verify actual TEXT/program base
and C80 bytes before and after USR and exercise normal BASIC allocations.
Record the manual commands in the example documentation.

#### Serialized Mixed-Image Acceptance (E11/T11)

The execution test must consume the actual CLI-produced TOML, not only the
compiler's in-memory structures or a separately rebuilt BASIC program:

1. Build the mixed fixture with `--emit-segments`, read the file back using
   the supported segment format, and locate BASIC through its exported range
   symbols. Assert exact BASIC payload bytes/terminator and C80/ASM ranges.
2. Start the selected TVC BASIC 1.2 environment. Establish the manual LOMEM
   reservation and valid interpreter workspace before loading C80 memory.
   A test helper may detokenize the BASIC bytes read from the artifact and
   enter those lines through the ROM's existing input path to establish its
   program pointers; it must not rebuild from the original `.bas` source.
3. Pause and load every serialized segment at its address using the existing
   mapped-memory path, then verify memory matches the file. The helper's
   earlier interpreter setup must agree with that same payload/base/length.
   Do not reset/reinject a tape program afterward and overwrite the linked layout.
4. Issue BASIC RUN, observe the compiled USR entry and return, check 42/-1 and
   signed-boundary results, and assert a subsequent BASIC statement executes.
   Bound execution and assert the protected C80 bytes remain intact.

Supply a reproducible manual/helper loading recipe with the fixture. It must
explicitly describe interpreter setup before `loadasm`; `loadasm` alone writes
bytes and does not initialize BASIC. No general loader UI, automatic compiler
wrapper, or new production interpreter-management layer is required. Tests that
only tokenize, inspect segments, or call machine code without BASIC do not pass
this E11 gate. The existing E07 pure-C80/ASM harness remains valid for E07.

This requirement closes an output/integration specification gap. An earlier
E11 completion based only on in-process tokenization/substitution must be
revalidated against this artifact test; the existing linker need not be replaced.

Other regions may require additional explicit BASIC/assembly setup by the user;
the compiler emits symbols/checks, not automatic interpreter-state writes.
Failure of a test is implementation/research work within this fixed target
contract, not a requirement for a separate fixture-approval step. Stop only if
verified ROM behavior contradicts the specified USR/LOMEM contract. Do not
change ROM/CPU implementation to force the fixture to pass.

## CAS Follow-Up: Linear Image First, Compression Second

CAS packaging follows the compiler's segment output in two separate increments
(E13/E14). It belongs in `rtvc-tocas`, not the C80 backend or linker. Earlier
references to deferred CAS packaging mean deferred beyond E12, not excluded
from these follow-up increments.

E13 accepts `rtvc-asm-v1` TOML and builds one linear byte block. Sort and validate
segments, preserve every byte at its linked address, and fill inter-segment gaps
with zeroes. Wrap that block using the existing TVC CAS encoder. This intentional
CAS padding does not change `rtvc-c80 --emit-bin`, which still rejects gaps.
Do not add a relocation loader or compressor in the first cut.

The first supported cassette profile is the existing BASIC-loadable image at
19EFH. Require the first segment to start there and contain a valid tokenized
BASIC program or an explicit BASIC launch stub; reject arbitrary binary or a
relocated BASIC-only image at 4000H rather than emitting a misleading runnable
CAS. The user supplies the launch/setup code and links its machine-code targets
at their final addresses. Package through the highest emitted end address, with
all gaps materialized; reject overflow and images outside the supported 64K TVC
RAM load range (exclusive end at most C000H). Report payload and padding sizes.
The linear load writes the gaps too: this profile requires exclusive ownership
of the entire load interval, even where TOML originally contained no segment.
User startup/reservation rules still apply after loading.

E14 adds opt-in [ZX0](https://github.com/einar-saukas/ZX0) compression over
exactly the same linear block, including zero padding. Port upstream's optimal
compressor and stream writer to native Rust, with no C FFI, external executable,
or C toolchain required for normal builds/use. Keep this packaging code outside
the C80 backend/linker. Use forward ZX0 v2 streams and adapt upstream's standard
Z80 decoder to the repository assembler. Classic v1, backwards streams, prefix
dictionaries, and alternative decoder variants are outside initial E14 scope.
Pin and record the upstream commit used for the port and fixtures, preserve
source notices and the [BSD-3-Clause license](https://github.com/einar-saukas/ZX0/blob/main/LICENSE),
and include ZX0 attribution in distributed documentation.

A BASIC launch stub invokes decompression, reconstructing the original bytes
before the explicitly selected program continuation. Require explicit scratch
memory for staged input, decoder, live bootstrap, and active stack, disjoint from
the output span. Validate initial cassette loading and staging as well as decoding;
reject unsafe overlap or insufficient space. ZX0 overlapping decompression is
not part of the initial path. Compression changes tape storage size, not linked
addresses or final RAM footprint. Keep uncompressed output as the default with
explicit `--compression none|zx0` selection. Report total cassette size including
bootstrap/staging overhead, even when compression makes the result larger. Do
not introduce general multi-bank relocation or ROM-call wrappers.

Use bounded Rust compression and host validation, rejecting empty input and
requiring the decoded length to equal the E13 image size. Cross-check with pinned
upstream fixtures and execute the adapted Z80 decoder on Rust output; host round
trips alone are insufficient. The codec choice is settled by this plan and needs
no further product approval. See the [E14 execution contract](c80-compiler-execution.md#e14--optional-zx0-compression-of-the-linear-cas-image).

Both increments must be tested by loading the resulting CAS through the real
cassette path and executing the program. Direct TOML memory injection alone
does not validate cassette packaging.

## Implementation Authority and Remaining Work

Implement E02-E12 using these contracts, then E13/E14 when those packaging
increments are assigned. The earlier open findings are resolved
as design decisions; their implementation tests remain to be written/run.
E01 parsing remains a completed increment, not proof it already handles every
later syntax form. Extend it in the increment that introduces the form and
update info/c80.md to distinguish implemented behavior from planned behavior.

No further product or contract approval is required for these increments.
Continue to stop for genuinely contradictory requirements, newly discovered
infeasibility, or missing external permissions, and record evidence. Do not
classify internal API naming, writing the specified tests, or an ordinary
implementation bug as a reason to stop for a design decision.
