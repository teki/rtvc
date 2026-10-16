; -----------------------------------------------------------------------------
; TVC BASIC 1.2 SYS upper ROM overlay - TVC12_D3.64K
; Source: roms/TVC12_D3.64K
; Comments: roms/TVC12_D3.64K.comments.json
; Symbols: roms/TVC12_D3.64K.symbols.json
; ORG: E000H
; Size: 8192 bytes
; Instructions use CPU-visible addresses at ORG; the ROM bank is recorded separately.
; Physical bank: SYS offset 2000H
; CPU-visible aliases: E000H, 2000H
; Auto labels: branch and call targets are emitted as Lxxxx.
; This is a standalone listing; all required technical explanations are embedded here.
; Technical descriptions are based on the Kaszanyiczki and Ludanyi TVC ROM references.
; -----------------------------------------------------------------------------

; =============================================================================
; TVC BASIC RUNTIME REFERENCE
; =============================================================================
; This is a standalone implementation reference for TVC BASIC 1.2. It consolidates the runtime
; model described by the Kaszanyiczki ROM listing and the TVC ROM program book; ROM bytes and the
; current disassembly remain authoritative for exact instruction boundaries.
; The BASIC is a tokenizing interpreter, not a text-to-machine-code compiler. Input is first
; converted to a compact intermediate form, then a statement or function dispatcher executes that
; form while maintaining a separate BASIC stack and symbol store.
; The reference is attached to the SYS upper-ROM listing anchor so it can be shown as contextual
; assembler documentation even when the described state lives in U0 RAM or another ROM overlay. No
; behavior depends on this anchor address.

; =============================================================================
; PROGRAM LINE STORAGE
; =============================================================================
; A stored BASIC line is a length-prefixed record: the first byte is the record length, the next
; two bytes are the binary line number, and the remaining bytes contain tokenized statements and
; literal characters.
; The payload ends with FFH, the line terminator. A zero length byte at the next record marks the
; end of the program; it is not a zero-length ordinary line.
; Statements within a line are separated by the colon token. The REM and exclamation-comment forms
; cause the interpreter to skip payload until a statement terminator or the line terminator, so
; comment text need not be parsed as expressions.
; Line numbers are stored as ordinary little-endian binary data rather than printable digits. The
; records are kept in ascending line-number order, which lets insertion, replacement, LIST,
; DELETE, GOTO, and GOSUB locate lines by scanning the length chain.
; The current program base is VLOMEM; TEXT identifies the program start. The line record itself is
; compact, but the length byte includes the stored tokenized payload and terminator and therefore
; must be treated as the movement unit when compacting memory.

; =============================================================================
; TOKENIZATION AND COMPACT SOURCE
; =============================================================================
; The editor delivers an ASCII paragraph into BUFFER. The tokenizer walks it left to right,
; recognizes keywords using the ROM keyword table, and emits one-byte tokens for keywords,
; punctuation, and selected operators while retaining ordinary characters and digits.
; Keyword-table entries are stored as character strings with the final character marked in bit 7;
; the associated token is emitted after a match. This permits abbreviations and the compact
; representation used by the interpreter without making the stored program depend on display text.
; Quoted strings are copied as data and are not keyword-expanded. Numeric text is converted into
; the BASIC numeric representation during expression evaluation; a numeric literal in a tokenized
; line is therefore not necessarily a sequence of printable decimal digits at execution time.
; The tokenizer returns the beginning of the compact COMMAND record. It reports malformed input
; through the common error path; the caller can then list the offending line, discard the
; uncommitted tail, and request another editor paragraph.
; A useful disassembler invariant is that token bytes and ASCII bytes can coexist in one payload.
; Do not decode every byte above 7FH as text: primary statement tokens, secondary keywords,
; operators, and encoded messages occupy that same byte space.

; =============================================================================
; STATEMENT AND FUNCTION DISPATCH
; =============================================================================
; The first token of a statement is its primary token. A primary-token jump table selects the
; routine that implements the statement; secondary tokens such as TO, STEP, AT, USING, commas, and
; parentheses are meaningful only while that primary routine is active.
; A statement routine consumes tokens through the shared next-token machinery, validates the
; required punctuation and operand types, then either performs the operation or leaves the
; interpreter at the next statement boundary.
; The same tokenized representation serves immediate commands and stored program lines. The
; difference is execution context: an immediate COMMAND record is run once, while a stored record
; updates current-line and next-line state and returns to the line scheduler.
; Function calls use the expression evaluator rather than the statement dispatcher. A function
; routine receives its argument on the BASIC stack, performs any type/range checks, and leaves one
; result in the stack representation expected by the surrounding expression.
; The primary-token table and the RST18 table are compact indirection layers. Their entries are
; data-driven dispatch, so a raw listing should label the table and its selectors separately from
; the routines they reach.

; =============================================================================
; BASIC WORKSPACE AND EXECUTION POINTERS
; =============================================================================
; The U0 BASIC work area records interpreter state independently of the CPU stack. FLAG at 1700H
; includes running-program, open-file, trace, and OK-message state; TYPE at 1708H records the
; current symbol type; and 1701H records the expected operand type (string or number).
; START (170CH) is the current line address, the following-line pointer is at 170EH, and the next
; statement address is at 1710H. DATA line and item pointers occupy 1712H-1715H; the current INPUT
; data pointer is at 1716H.
; The RST18 continuation pointer at 1718H identifies the next function byte in a sequence. The
; saved BASIC-stack pointer at 171AH is used at statement boundaries and while unwinding immediate
; commands.
; VLOMEM (1720H) is the program base, TEXT (1722H) is the first program byte, CHAIN (1724H) links
; the symbol store, and TOP (1726H) is the next free symbol-table byte. COMMAND begins at 1732H
; and BUFFER at 1831H.
; X at 19C0H and Y at 19C7H are the two floating-point work registers used by RST18 arithmetic.
; The file-name and header buffers follow them; changing the BASIC base changes which RAM is
; available to the program, stack, and symbols.

; =============================================================================
; BASIC STACK AND NUMERIC REPRESENTATION
; =============================================================================
; The BASIC stack is a typed, variable-size data stack in high RAM. It grows downward from HIMEM
; and is addressed by IY, not by the Z80 SP. It holds expression operands, strings, FOR and GOSUB
; frames, return information, and temporary values.
; Every element begins with a type byte. Numeric values use the 09H element type and occupy nine
; bytes in the arithmetic stack format; string elements carry a string type and a length/data
; body. Control frames have their own type and layout and must not be mistaken for numbers.
; A numeric element contains packed-decimal mantissa data and a final exponent/sign byte. The
; exponent is biased around 40H and the high sign bit is used for negative values; zero is
; represented by a zero mantissa and is normalized by the arithmetic helpers.
; The stack format is decimal-oriented and supports a broad exponent range. Arithmetic routines
; align exponents, operate on packed decimal digits with DAA correction, normalize the result, and
; report overflow or division-by-zero through the BASIC error path.
; Most binary arithmetic consumes the upper operand and replaces the lower operand with the
; result, advancing IY by one element size. Any caller that borrows stack storage must preserve
; the element type and restore IY exactly at the boundary it promises.

; =============================================================================
; RST18 BASIC-STACK LANGUAGE
; =============================================================================
; RST 18H is followed by a sequence of BASIC function bytes. The dispatcher saves the continuation
; address in 1718H, executes each selected operation, and continues until the high bit of the
; final function byte is set.
; The sequence behaves like a small postfix language: operands are pushed, then operations consume
; the top values. Two-operand arithmetic leaves one result; unary operations modify or replace the
; top number; stack movement operations copy values to X, Y, or a symbol entry.
; The documented primitive vocabulary includes addition, subtraction, multiplication, division,
; NEG, loading a ROM numeric constant, loading X or Y, copying to X/Y, destructive copies, DUP,
; VAR, and NUM for evaluating a parenthesized expression.
; The X and Y work registers are six/seven-byte numeric work areas used by transcendental and
; conversion routines; they are not independent BASIC variables. RST18 callers must assume that
; arithmetic helpers use them and the BASIC workspace unless a routine explicitly documents
; preservation.
; A function-byte sequence is compact but context-sensitive: it expects IY to identify a valid
; BASIC stack element and may call the expression evaluator or error handler. It is useful for
; hand-written ROM clients, but it is not a general Z80 ABI.

; =============================================================================
; SYMBOL TABLE AND VARIABLE LIFETIME
; =============================================================================
; Symbols are kept in a linked chain between CHAIN and TOP. Each entry begins with a little-endian
; link to the next entry, followed by a name length and name bytes, then a type byte and
; type-specific data.
; The type byte distinguishes scalar versus array, string versus numeric, ordinary variable versus
; DEFined function, and user-created versus built-in symbol. The interpreter uses these bits both
; for lookup and for rejecting incompatible assignments or redeclarations.
; A lookup can find an existing built-in symbol and create a user symbol with the same spelling
; only where the statement rules permit it. DEF and DIM explicitly check for duplicate
; declarations and raise the Variable declared twice error when the type/name rules disallow
; reuse.
; Scalar numeric data is stored in the numeric format; scalar strings carry a length and character
; data. VARPTR and machine-code interfaces expose addresses of data, but such pointers become
; invalid if symbol storage is compacted or the BASIC workspace is reinitialized.
; NEW and any operation that changes the stored program reinitialize the symbol administration,
; DATA pointers, and arithmetic stack. The old bytes may remain in RAM, but the live chain and TOP
; no longer describe them.

; =============================================================================
; ARRAYS AND STRINGS
; =============================================================================
; DIM creates a symbol-table entry marked as an array and appends its dimension metadata and
; element storage at TOP. Dimension bounds are inclusive from zero, so DIM A(10) allocates eleven
; numeric elements.
; A numeric array may have multiple dimensions. The evaluator consumes each index expression,
; checks its range, and folds the indices into the array's linear element address; a bad or
; missing subscript reaches the standard error handler.
; String arrays carry an element maximum length after the dimension information. Undimensioned
; strings use the default short-string capacity; DIM permits larger fixed-capacity elements, so
; the assignment and concatenation paths must enforce the stored capacity rather than merely the
; current length.
; String values are length-prefixed or otherwise length-tracked in their stack and symbol
; representations. Operations copy only the declared/current character count, and string
; comparisons stop at the shorter operand before applying length ordering.
; The string and numeric paths share token and stack infrastructure but are not interchangeable. A
; type mismatch is raised when an expression, assignment, INPUT field, or function argument
; selects the wrong representation.

; =============================================================================
; EXPRESSION EVALUATION AND PRECEDENCE
; =============================================================================
; The numeric evaluator reads tokenized source through HL' and pushes each completed value onto
; the BASIC stack. Parenthesized arguments are evaluated recursively; the caller supplies the
; closing-token expectation and receives IY at the resulting value.
; The evaluator has distinct paths for numeric literals, quoted strings, symbol references, array
; elements, built-in functions, and user DEF functions. It records the expected type before
; dispatch so a string result cannot silently enter numeric arithmetic.
; Parentheses bind first, followed by exponentiation, multiplication/division,
; addition/subtraction, relational comparisons, NOT, AND, and OR/XOR. Operators at a common level
; are normally consumed left to right by the token-walking code.
; Relational operators produce a numeric truth value suitable for IF and ON. Numeric comparisons
; use sign, biased exponent, and packed digits; string comparisons use character order and then
; length when the common prefix is equal.
; Expression errors include malformed delimiters, missing arguments, undefined or incorrectly
; typed symbols, bad subscripts, divide by zero, and numeric overflow. The evaluator leaves the
; error path rather than returning an ambiguous partial stack value.

; =============================================================================
; PROGRAM EDITING AND STORAGE MAINTENANCE
; =============================================================================
; After tokenization, the line insertion routine scans the ordered program records for the target
; number. An existing record with the same number is removed before the replacement is inserted; a
; line whose payload is immediately terminated is treated as a deletion.
; Insertion creates room by moving the remainder of the program toward higher addresses. Deletion
; compacts the remainder toward lower addresses. The movement length is derived from the old
; program end and record length, so stale pointers into the program must not be retained across
; the operation.
; The BASIC workspace initializer is called after a program mutation. It clears the arithmetic
; stack, resets END/running state, reinitializes DATA traversal, clears user symbols, reseeds the
; random state, and moves TOP to the new end of the symbol area.
; LIST and LLIST locate records by line range and send the tokenized payload through the output
; path; DELETE uses the same line-range scanner and compaction helper. A missing line is a search
; result, not a memory fault.
; NEW marks the active program empty by writing a zero at its base and reinitializes the
; associated state; it need not erase every old program byte. This is an implementation caveat for
; memory inspectors, not a supported recovery interface.

; =============================================================================
; EXECUTION STATE AND CONTROL FLOW
; =============================================================================
; The line scheduler follows START, the next-line pointer, and the next-statement pointer. At each
; boundary it can service TRACE, test STOP-FLAG, discard transient expression values, and select
; the next primary token.
; RUN initializes execution state, closes or resets file context as required, sets the
; running-program flag, and starts at TEXT or at the requested line. Immediate commands use the
; COMMAND buffer and return after restoring the saved BASIC-stack depth.
; FOR frames and GOSUB frames live on the BASIC stack, not the CPU stack. END/STOP and error
; unwinding inspect those typed frames so the interpreter can restore or discard nested control
; state without confusing it with numeric operands.
; GOTO and GOSUB resolve a binary line number through the ordered record scanner. RETURN requires
; a matching GOSUB frame; NEXT requires a matching FOR frame and updates the control variable
; before deciding whether to loop or discard the frame.
; CONTINUE reuses saved current-line and next-statement state after a stoppable interruption when
; the required execution context still exists. If the program is not resumable, it raises Cannot
; Continue rather than guessing a location.

; =============================================================================
; LOGICAL I/O AND FUNCTION CLASSES
; =============================================================================
; TVC I/O is addressed to logical function classes rather than hard-wired devices. The input and
; output assignment tables at 0B00H-0B0FH map video, keyboard, editor, sound, printer,
; cassette/storage, expansion cards, and kernel services to implementations.
; A logical class can be redirected by changing its assignment byte. This is why PRINT, INPUT,
; LIST, editor reads, and file operations share common front ends while still reaching different
; low-level routines.
; The first three device function ordinals have a uniform meaning across classes: interrupt
; service, single-character transfer, and block transfer. Device-specific functions follow those
; common entries and may be unsupported for a given assignment.
; The class selected by a BASIC peripheral qualifier is kept in the BASIC workspace. A separate
; selected-device byte and the assignment table determine the eventual implementation, so a
; peripheral number in source is not itself a ROM routine address.
; Kernel class calls are not redirected. Expansion-card assignments are validated specially and
; may transfer into the extension ROM; invalid class/device combinations return a function-call
; error before the device routine is entered.

; =============================================================================
; RST30 FUNCTION-CALL CONVENTION
; =============================================================================
; An OS function call is encoded as RST 30H followed by one function byte. The dispatcher extracts
; direction from bit 7, function class from bits 6-4, and the routine ordinal from bits 3-0.
; At the common call boundary, DE and BC carry operation parameters or lengths and may receive
; auxiliary results. The selected low-level routine returns A=00 for success; any nonzero A is a
; device- or dispatcher-defined error code.
; The dispatcher saves the function byte and the relevant CPU state, resolves the class through
; the input or output assignment table, validates the target device, and then invokes the selected
; ordinal. Callers must not assume that a successful device call preserves all general registers.
; Character I/O normally uses C for the byte; block I/O uses BC for the count and HL for the
; implementation entry in the device interface. Exact register preservation belongs to the
; individual device routine, while A's error contract belongs to the common dispatcher.
; RST30 is a compact TVC OS ABI, not an ordinary CALL to a stable address. It depends on the
; page-0 RAM gateway and current assignment tables, so a standalone machine-code client should
; preserve the documented page state and handle nonzero A.

; =============================================================================
; PRINT, LPRINT, AND USING
; =============================================================================
; PRINT selects the current output class, optionally processes AT and a peripheral qualifier, then
; walks a list of expressions separated by comma, semicolon, or TAB controls. LPRINT follows the
; same engine with printer class selected by default.
; A comma advances to the next output tabulation position; a semicolon suppresses the separator;
; TAB evaluates its argument and positions the selected video/editor-compatible output. A bare
; PRINT emits a line ending, while a trailing separator suppresses the normal final line ending.
; Numeric values are converted from the nine-byte BASIC stack format into ASCII in the
; numeric-format workspace before output. Zero, sign, decimal placement, and optional exponent
; notation are decided by this conversion rather than by the tokenized literal spelling.
; USING copies its format string into a dedicated BASIC buffer, scans literal characters
; separately from format controls, and remembers the first unused format byte between values. It
; then applies the next numeric or string value to that format and emits padding or literal text
; through the normal character output path.
; The format language includes digit/decimal placeholders, exponent selection, forced or trailing
; signs, fill characters, and left/right string alignment. A malformed format, wrong value type,
; or insufficient/invalid format structure returns through the BASIC error path.

; =============================================================================
; INPUT AND EDITOR LINE ACQUISITION
; =============================================================================
; BASIC line input obtains an edited paragraph through the editor/logical input path, stores it in
; BUFFER, and then tokenizes it into COMMAND. The same route serves immediate commands and
; numbered program lines.
; The editor's character input is stateful: an initial call starts a paragraph, subsequent calls
; can return more characters, and a terminating RETURN completes the paragraph. The caller must
; honor the editor's continuation and error status rather than treating every return as a complete
; line.
; CTRL+ESC sets STOP-FLAG and aborts the current input/execution path. End-of-file and device
; errors are returned as status values and are converted to BASIC errors or a controlled retry by
; the higher-level INPUT routine.
; INPUT parses comma-separated fields according to the destination symbol type. Numeric fields
; accept signed decimal/exponent syntax and are converted through ASCII_TO_FP; string fields
; retain character data subject to the destination capacity and quoting rules.
; The input parser assigns each field before requesting the next one. Missing fields, excess
; variables, malformed numbers, type mismatches, and a device-level failure are distinct cases
; even when the user-visible recovery is another input prompt.

; =============================================================================
; ERROR MODEL, MESSAGES, AND STOP
; =============================================================================
; RST 08H is the BASIC error gateway. A code may follow the RST instruction in ROM, or a caller
; may enter the alternate gateway with the code already in A. Code zero is a no-error return for
; the latter path.
; The general handler unwinds the active interpreter context, selects the appropriate output
; class, prints the error prefix and message, and appends line context when a stored program was
; running. It then restores a safe immediate-command state or leaves the program stopped.
; Messages are stored compactly as code/length/text records. Shared fragments such as Cannot, No,
; Bad, Argument, and missing are combined with token-specific text, so the byte table is not a
; simple array of null-terminated C strings.
; The BASIC table includes errors for unrecognized input, missing lines or arguments, bad
; arguments/subscripts, no memory, divide by zero, overflow, type mismatch, duplicate
; declarations, file failures, and BASIC corruption. Device and extension codes may use a separate
; system-error presentation.
; CHECK_STOP_FLAG polls 0B16H. When set, STOP clears the flag, records the current execution
; context, prints STOP and the current line, and returns through the same resumable machinery used
; by CONTINUE. A STOP is therefore a controlled interpreter transition, not a CPU reset.

; =============================================================================
; DIRECT CALLS, EXTENSIONS, AND CAVEATS
; =============================================================================
; The page-0 RAM gateway contains the copied BASIC error entries, the RST18 dispatcher entry, and
; the extension/function-call forwarding code. These addresses are initialized at startup and
; should be treated as gateways rather than immutable ROM routines.
; EXT selects one of seven user subroutine slots in USRTAB at 0021H-002EH. The table holds
; little-endian addresses; BASIC can pass values in HL, DE, and BC, and a user routine must return
; with RET. The slot contents are cleared or reinitialized by BASIC setup operations.
; USR(address [, parameter]) enters machine code at the requested address with the parameter in HL
; and interprets the returned HL as the numeric result. The call crosses the BASIC numeric
; conversion boundary, so a routine should preserve the required interpreter state and return a
; valid signed integer result.
; RST18 is suitable for compact numeric helpers when the caller owns a valid BASIC stack frame.
; RST30 is suitable for OS/device services when the caller follows the function-byte and A-status
; contract. Neither interface guarantees that arbitrary RAM pointers survive program edits, NEW,
; LOMEM, or symbol-table compaction.
; ROM overlays and extension mappings matter at every direct-call boundary. A routine reached
; through a page switch may return with a different mapping expectation than it had on entry; use
; the current ROM listing and the gateway code as the authority for exact paging, register
; preservation, and re-entry behavior.
; When extending this reference, prefer semantic invariants and data-layout diagrams over copying
; listing prose. Keep byte-level labels in the address-local annotation fragments and use this
; sectioned file for the cross-routine contracts that remain useful outside one disassembly page.

; =============================================================================
; SYMBOLS USED BY THIS LISTING
; =============================================================================
; U0 RAM cells and system variables referenced below. Names follow the workspace notes
; above; addresses whose meaning is not documented stay numeric in the code.
P_SAVE             EQU 0003H              ; saved port-02 page map (written by every page switch)
RST18_OP_JP        EQU 0015H              ; JP opcode patched by the RST 18H dispatcher
RST30_A_ENTRY      EQU 001BH              ; RAM stub: RST 30H with the function code taken from A
RST30_A_FN         EQU 001FH              ; function byte patched into the RST30_A_ENTRY stub
USRTAB             EQU 0021H              ; seven USR/EXT subroutine addresses
PORT06_MIRROR      EQU 0B13H              ; copy of port 06: STROBE, volume, colour mode
TONE_REPLACE       EQU 0B15H              ; FFH: a new tone replaces the current one
STOP_FLAG          EQU 0B16H              ; FFH after CTRL+ESC (cooperative break)
PAINT_STACK_MIN    EQU 0B17H              ; lowest stack address PAINT may use
TIME               EQU 0B1DH              ; software clock incremented by the interrupt
INK                EQU 0B4DH              ; logical ink colour
PAPER              EQU 0B4EH              ; logical paper colour
BORDER             EQU 0B4FH              ; border colour (I.G.R.B in bits 7,5,3,1)
CAS_BUFFER         EQU 0B6BH              ; cassette mode: 00H unbuffered, FFH buffered
CAS_OUT_NAME       EQU 0D15H              ; output file name
VALUE_TYPE         EQU 1701H              ; pending value type: 01H string, 03H number
COND_FLAG          EQU 1702H              ; conditional-execution flag
RST18_ARG          EQU 1704H              ; A as passed to the current RST 18H operation
FUNC_CLASS         EQU 1705H              ; function class
AUTORUN            EQU 1707H              ; FFH: autorun pending
SYMBOL_TYPE        EQU 1708H              ; symbol type
RND_STATE          EQU 1709H              ; RND generator state
CUR_LINE           EQU 170CH              ; current program line pointer
NEXT_LINE          EQU 170EH              ; next program line pointer
NEXT_STMT          EQU 1710H              ; next statement pointer
DATA_LINE          EQU 1712H              ; DATA line pointer
DATA_PTR           EQU 1714H              ; DATA byte pointer
INPUT_PTR          EQU 1716H              ; INPUT data pointer
RST18_NEXT         EQU 1718H              ; next RST 18H bytecode byte
BASIC_SP           EQU 171AH              ; BASIC stack pointer snapshot
VLOMEM             EQU 1720H              ; low-memory/base boundary
TEXT_START         EQU 1722H              ; start of the BASIC program
SYMBOL_END         EQU 1724H              ; end of the chained symbol area
SYMBOL_TOP         EQU 1726H              ; next free symbol byte
SOUND_PITCH        EQU 172AH              ; current sound PITCH
INPUT_BUFFER       EQU 1831H              ; INPUT buffer
FP_X               EQU 19C0H              ; floating-point work register X
FP_Y               EQU 19C7H              ; floating-point work register Y
CAS_HEADER         EQU 19DFH              ; cassette header workspace
BASIC_RAM_START    EQU 19EFH              ; first byte after the ROM workspace

; SYS ROM routines and tables defined in TVC12_D4.64K.asm; both halves of the SYS bank are
; visible together whenever SYS is mapped.
NUMBERED_FP_CONSTANTS          EQU C111H
BASIC_REM_ENTRY                EQU DBBBH
BASIC_FREE_SPACE               EQU DCC9H
BASIC_WORKSPACE_INIT           EQU DCFCH
BASIC_PRINT_LINE               EQU DD2DH
BASIC_KEYWORD_TABLE            EQU DE6DH

ORG E000H, SYS0, 2000H

    DB B1H, DBH                           ; operand bytes of the instruction at DFFFH, which starts in the previous ROM chip

; DEF routine.

LE002:
    AND 81H
    CP 01H
    JP NZ,LFD5A
    BIT 2,(IX+00H)
    JR Z,LE02E
    EXX
    PUSH HL
    PUSH DE
    EXX
    POP HL
    DEC HL
    BIT 7,(HL)
    JR Z,LE04E
    RES 7,(HL)
    SET 2,(HL)
    INC HL
    LD DE,(CUR_LINE)
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    POP DE
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    LD (SYMBOL_TOP),HL

LE02E:
    EXX
    LD C,01H

LE031:
    DEC C
    DB 3EH                                ; = LD A,0CH when run from here, hiding the entry below

LE033:
    INC C

LE034:
    LD A,(HL)
    CP FEH
    JP NC,LE04B
    INC HL
    CP 96H
    JR Z,LE033
    CP 95H
    JR Z,LE031
    CP FDH
    JR NZ,LE034
    INC C
    DEC C
    JR NZ,LE034

LE04B:
    JP DB81H

LE04E:
    RST 08H
    DB 0FH                                ; error code 0FH: Variable declared twice

LE050:
    CALL LFC43

; DIM routine.

LE053:
    OR A
    JP M,LFD5A
    BIT 0,A
    JP Z,LFD5A
    EXX
    LD A,C
    AND 88H
    JR Z,LE04E
    AND 08H
    CALL NZ,SYMBOL_CREATE_VALUE
    DEC DE
    LD A,(DE)
    XOR 81H
    LD (VALUE_TYPE),A
    LD (DE),A
    INC DE
    LD A,FFH
    LD (DE),A
    PUSH DE
    INC DE
    LD (SYMBOL_TOP),DE
    EXX
    CALL LFC43
    BIT 1,(IX+01H)
    JR NZ,LE0A3
    CP A8H
    JR NZ,LE0A3
    CALL STACK_NUMBER_TO_BYTE
    INC A
    JR Z,LE0E4
    DEC A
    LD C,A
    POP HL
    LD (SYMBOL_TOP),HL
    DEC HL
    RES 0,(HL)
    CALL LF3C1

LE099:
    EXX
    LD A,B
    EXX
    CP A4H
    JR Z,LE050
    JP DBB4H

LE0A3:
    LD A,96H
    CALL LFD54
    SCF

LE0A9:
    CALL NC,LFC43
    CALL LFAC4
    JP M,LFB14
    INC HL
    EX DE,HL
    LD HL,(SYMBOL_TOP)
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    LD (SYMBOL_TOP),HL
    EX DE,HL
    CALL INTEGER_TO_STACK
    POP HL
    INC (HL)
    PUSH HL
    CALL NZ,FP_MUL
    EXX
    LD A,B
    EXX
    CP A4H
    JR Z,LE0A9
    LD A,95H
    CALL LFD54
    POP HL
    INC (HL)
    BIT 1,(IX+01H)
    JR NZ,LE0E9
    CP A8H
    LD A,12H
    CALL Z,STACK_NUMBER_TO_BYTE
    INC A

LE0E4:
    JP Z,LFB14
    DEC A
    LD C,A

LE0E9:
    PUSH BC
    RST 18H
    DB 0CH, 85H, 26H                      ; RST 18H: op0C, CONST 38
    CALL FP_COMPARE
    JP P,LFCB1
    CALL STACK_NUMBER_TO_INTEGER
    POP BC

LE0F8:
    PUSH HL
    CALL INIT_STRING_OR_ARRAY_SYMBOL
    POP HL
    DEC HL
    LD A,L
    OR H
    JR NZ,LE0F8
    JR LE099

; ELSE routine.

LE104:
    BIT 0,(IX+02H)
    JP Z,LFD5A
    JP BASIC_REM_ENTRY

; END routine.

LE10E:
    LD HL,0000H
    LD (NEXT_LINE),HL
    JP DADAH

; EXT routine.

LE117:
    CALL LFB1B
    CP 07H
    JP NC,LFB14
    ADD A,A
    PUSH AF
    EXX
    LD A,B
    EXX
    CP A4H
    JR NZ,LE13E
    CALL LFABA
    EX DE,HL
    CP A4H
    JR NZ,LE13E
    CALL LFABA
    CP A4H
    JR NZ,LE13E
    PUSH HL
    CALL LFABA
    LD C,L
    LD B,H
    POP HL

LE13E:
    EX (SP),HL
    PUSH DE
    LD E,H
    LD D,00H
    LD HL,USRTAB
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    LD A,D
    OR E
    JP Z,DBB1H
    LD (RST18_OP_JP+1),DE
    POP DE
    LD HL,DBB1H
    EX (SP),HL
    EX DE,HL
    JP RST18_OP_JP

; FOR routine.

; BASIC_FOR - Implements the BASIC FOR statement.
; Entry: Tokenized loop variable, limits, and optional step
; Effects: Pushes a FOR frame on the BASIC stack.
BASIC_FOR:
    CP 03H
    JR Z,LE167
    AND 82H
    JP NZ,LFD5A
    RST 08H
    DB 0EH                                ; error code 0EH: Type mismatch

LE167:
    CALL STACK_TO_NUMERIC_REGS
    LD A,9AH
    CALL LFD54
    CALL LFC8E
    PUSH HL
    CALL EVAL_NUMERIC_EXPRESSION
    POP HL
    PUSH HL
    CALL LFB28
    LD A,B4H
    CALL LFD54
    CALL EVAL_NUMERIC_EXPRESSION
    LD DE,FFEEH
    ADD IY,DE
    CP B8H
    JR Z,LE195
    EX AF,AF'
    LD HL,0001H
    CALL INTEGER_TO_STACK
    EX AF,AF'
    SCF

LE195:
    CALL NC,LF0A4
    CP FDH
    JP C,LFD5A
    PUSH IY
    POP HL
    POP DE
    DEC HL
    LD (HL),D
    DEC HL
    LD (HL),E
    LD DE,(CUR_LINE)
    DEC HL
    LD (HL),D
    DEC HL
    LD (HL),E
    EXX
    PUSH HL
    EXX
    POP DE
    DEC HL
    LD (HL),D
    DEC HL
    LD (HL),E
    DEC HL
    LD (HL),2BH
    PUSH HL
    POP IY
    JP DBB1H

LE1BE:
    CP F5H
    RST 10H
    EXX

LE1C2:
    DEC HL
    LD A,(HL)
    CP ECH
    JR NZ,LE1C2
    JP LFFA4

; INPUT routine.

; BASIC_INPUT - Implements the BASIC INPUT statement.
; Entry: Tokenized variable list and optional prompt
; Effects: Reads text and assigns BASIC variables.
BASIC_INPUT:
    EX AF,AF'
    SCF
    EX AF,AF'
    LD HL,E2EBH
    LD (IX+05H),20H
    SCF

LE1D6:
    CALL NC,LFC43
    CP BAH
    JR NZ,LE1FC

; INPUT PROMPT routine.
    CALL LFC43
    CALL EVAL_STRING_EXPRESSION
    PUSH IY
    CALL DISCARD_STRING
    POP HL
    INC HL
    EXX
    LD A,B
    EXX

LE1ED:
    EX AF,AF'
    OR A
    EX AF,AF'
    CP A4H
    JR Z,LE1D6
    CP FDH
    JR Z,LE204
    JR NC,LE207

LE1FA:
    RST 08H
    DB 01H                                ; error code 01H: Not understood

LE1FC:
    CALL LFBFB
    JR Z,LE1ED
    EX AF,AF'
    JR NC,LE1FA

LE204:
    CALL NC,LFC43

LE207:
    LD A,(FUNC_CLASS)
    CP 20H
    CALL Z,PRINT_LENGTH_TEXT
    RST 30H
    DB 24H                                ; RST 30H fn 24H: editor 4 -> CU_FIX
    CALL BASIC_LINE_INPUT
    JR NZ,LE1BE
    INC HL
    LD (INPUT_PTR),HL
    DB F6H                                ; = OR 37H when run from here, hiding the entry below

; READ routine.

LE21B:
    SCF
    EX AF,AF'
    EXX
    LD A,B
    EXX
    SCF

LE221:
    CALL NC,LFC43
    CP FDH
    JP NC,DBB4H
    AND 81H
    CP 01H
    JP NZ,LFD5A
    EX AF,AF'
    JR NC,LE27C
    LD HL,(DATA_PTR)
    LD A,L
    OR H
    JR Z,LE268

LE23A:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,LE23A
    DEC HL
    CP 21H
    JR Z,LE25E
    CP FDH
    JR C,LE27C

LE249:
    EXX
    PUSH HL
    PUSH DE
    PUSH BC
    CALL LFCC5
    EXX
    POP BC
    POP DE
    POP HL
    EXX
    INC HL

LE256:
    CP FBH
    JR Z,LE23A
    CP FEH
    JR C,LE249

LE25E:
    LD HL,(DATA_LINE)
    LD E,(HL)
    LD D,00H
    ADD HL,DE
    LD (DATA_LINE),HL

LE268:
    LD HL,(DATA_LINE)
    LD A,(HL)
    OR A
    JR Z,LE27A
    INC HL
    INC HL
    INC HL

LE272:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,LE272
    JR LE256

LE27A:
    RST 08H
    DB 07H                                ; error code 07H: No DATA

LE27C:
    EX AF,AF'
    PUSH HL
    EXX
    PUSH BC
    CALL STACK_OPERAND_WALK
    POP BC
    BIT 1,C
    EX (SP),HL
    JR NZ,LE294
    CALL STRING_TO_STACK
    EX (SP),HL
    CALL LFB41
    JR LE2B0

LE292:
    RST 08H
    DB 0CH                                ; error code 0CH: Cannot READ

LE294:
    CALL LFC8E
    CALL ASCII_TO_FP
    EX AF,AF'
    JR NC,LE2A6
    EX AF,AF'
    JR NC,LE292
    RLA
    JP C,LF912
    JR LE2AC

LE2A6:
    EX AF,AF'
    PUSH HL
    CALL LEC7F
    POP HL

LE2AC:
    EX (SP),HL
    CALL LFB28

LE2B0:
    POP HL
    EX AF,AF'
    JR NC,LE2CE
    EX AF,AF'

LE2B5:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,LE2B5
    CP 2CH
    JR Z,LE2C9
    DEC HL
    CP 21H
    JR Z,LE2C9
    BIT 7,(HL)
    JP Z,LE292

LE2C9:
    LD (DATA_PTR),HL

    JR LE2DC

LE2CE:
    EX AF,AF'

LE2CF:
    LD A,(HL)
    INC HL
    CP 2CH
    JR Z,LE2D9
    INC A
    JR NZ,LE2CF
    DEC HL

LE2D9:
    LD (INPUT_PTR),HL

LE2DC:
    EXX
    LD A,B
    EXX
    CP FDH
    JP NC,DBB4H
    CP A4H
    JP Z,LE221
    RST 08H
    DB 01H                                ; error code 01H: Not understood
    DB 02H, 3FH, 20H                                                                  ; |.? |

; IF routine.

; BASIC_IF - Implements the BASIC IF statement.
; Entry: Tokenized condition and branch statement
; Effects: Changes interpreter control flow.
BASIC_IF:
    SET 0,(IX+02H)
    CALL EVAL_NUMERIC_EXPRESSION
    LD A,(IY+06H)
    OR A
    EX AF,AF'
    CALL DISCARD_NUMBER
    EXX
    PUSH HL
    EXX
    POP HL
    LD A,B5H
    CALL LFD54
    EX AF,AF'
    JR Z,LE31C
    EX AF,AF'
    JR LE314

LE30C:
    INC HL
    PUSH HL
    EXX
    POP HL
    EXX
    CALL LFC43

LE314:
    CP 02H
    JP Z,BASIC_GOTO
    JP DB81H

LE31C:
    LD IY,(BASIC_SP)

LE320:
    CALL LFCC5
    CP F4H
    JR Z,LE30C
    CP FEH
    JP NC,BASIC_REM_ENTRY
    PUSH HL
    EXX
    POP HL
    EXX
    JR LE320

; ON routine.

LE332:
    SET 0,(IX+02H)
    CALL LFB1B
    LD B,A
    EXX
    LD A,B
    EXX
    CP EFH
    JR Z,LE3AF
    CP F0H
    JR NZ,LE369
    CALL LE357
    CALL LFBDE
    LD IY,(BASIC_SP)
    PUSH HL
    CALL LFCD1
    EX DE,HL
    POP HL
    JR LE38E

LE357:
    CALL LFC43
    DEC B
    RET Z
    EXX
    PUSH HL
    EXX
    POP HL
    CALL LFC43
    JR NC,LE36B
    CP A4H
    JR Z,LE357

LE369:
    RST 08H
    DB 01H                                ; error code 01H: Not understood

LE36B:
    POP DE
    LD IY,(BASIC_SP)

LE370:
    EXX
    CALL LFCC5
    CP F4H
    JR Z,LE30C
    CP FEH
    JP NC,LFB14
    PUSH HL
    EXX
    POP HL
    JR LE370

; GOSUB routine.

; BASIC_GOSUB - Implements the BASIC GOSUB statement.
; Entry: Target line from tokenized statement
; Effects: Pushes a return frame and branches to a BASIC line.
BASIC_GOSUB:
    CALL LFBDE
    CALL LFC43
    JR C,LE369
    EXX
    PUSH HL
    EXX
    POP DE

LE38E:
    PUSH HL
    CALL LFC8E
    PUSH IY
    POP HL
    DEC HL
    LD A,(COND_FLAG)
    LD (HL),A
    LD BC,(CUR_LINE)
    DEC HL
    LD (HL),B
    DEC HL
    LD (HL),C
    DEC HL
    LD (HL),D
    DEC HL
    LD (HL),E
    DEC HL
    LD (HL),06H
    PUSH HL
    POP IY
    POP HL
    JR LE3B9

LE3AF:
    CALL LE357

; GOTO routine.

; BASIC_GOTO - Implements the BASIC GOTO statement.
; Entry: Target line from tokenized statement
; Effects: Branches to a BASIC line.
BASIC_GOTO:
    CALL LFBDE
    LD IY,(BASIC_SP)

LE3B9:
    JP DE29H

LE3BC:
    DEC HL
    EXX
    CALL LFC43

; LET routine.

; BASIC_LET - Implements BASIC assignment.
; Entry: Tokenized variable and expression
; Effects: Updates a BASIC variable.
BASIC_LET:
    AND FDH
    CP 01H
    JP NZ,LFD5A
    EXX
    BIT 2,C
    JP NZ,LFD5A
    BIT 3,C
    CALL NZ,SYMBOL_CREATE_VALUE
    LD (IX+01H),C
    EXX
    CALL STACK_TO_NUMERIC_REGS
    EXX
    LD A,B
    EXX
    PUSH HL
    CALL LE3E8
    POP HL
    CALL LFB3B
    JP DBB1H

LE3E8:
    SUB 96H
    JR NZ,LE44A
    BIT 1,C
    JP NZ,LFD5A
    INC HL
    PUSH HL
    LD C,(HL)
    LD B,A
    CALL LFD27
    LD A,D
    CP E
    JR NC,LE3FE
    LD A,E
    DEC A

LE3FE:
    LD D,A
    LD A,C
    SUB D
    POP HL
    PUSH IY
    PUSH HL
    PUSH DE
    PUSH BC
    JR C,LE417
    JR Z,LE417
    ADD HL,BC
    LD C,A
    PUSH IY
    POP DE
    DEC DE
    LDDR
    INC DE
    PUSH DE
    POP IY

LE417:
    CALL LE44A
    POP BC
    POP DE
    POP HL
    LD A,E
    PUSH IY
    POP DE
    INC DE
    CP 02H
    JR C,LE432
    DEC A
    CP C
    JR NC,LE42B
    LD C,A

LE42B:
    ADD HL,BC
    INC C
    DEC C
    JR Z,LE432
    LDDR

LE432:
    POP HL
    DEC HL
    OR A
    SBC HL,DE
    INC H
    DEC H
    JP NZ,LF912
    INC L
    JP Z,LF912
    DEC L
    EX DE,HL
    LD (HL),E
    DEC HL
    LD (HL),01H
    PUSH HL
    POP IY
    RET

LE44A:
    LD A,9AH
    CALL LFD54
    JP NUMERIC_STRING_GATE

; LOMEM routine.

LE452:
    CALL LFAC4
    LD DE,(VLOMEM)
    OR A
    SBC HL,DE
    ADD HL,DE
    JP C,LFB14
    LD DE,(TEXT_START)
    SBC HL,DE
    JP Z,DBB4H
    PUSH HL
    CALL C,LFC86
    LD C,L
    LD B,H
    EX DE,HL
    JR C,LE485
    POP DE
    PUSH DE
    PUSH HL
    INC D
    JP Z,LFB14
    CALL LFC99
    POP HL
    PUSH HL
    CALL BASIC_FREE_SPACE
    POP HL
    ADD HL,BC
    JR LE48D

LE485:
    OR A
    SBC HL,BC
    PUSH HL
    CALL DCE9H
    POP HL

LE48D:
    LD (TEXT_START),HL
    POP BC
    BIT 2,(IX+00H)
    JR Z,LE4A3
    LD HL,(CUR_LINE)
    ADD HL,BC
    LD (CUR_LINE),HL
    PUSH BC
    EXX
    POP DE
    ADD HL,DE
    EXX

LE4A3:
    CALL BASIC_WORKSPACE_INIT
    JP DBB1H

LE4A9:
    RST 08H
    DB 08H                                ; error code 08H: No FOR

LE4AB:
    EXX
    LD A,B
    EXX
    CP A4H
    JP NZ,DBB4H
    CALL LFC43

; NEXT routine.

; BASIC_NEXT - Implements the BASIC NEXT statement.
; Entry: Optional loop variable from tokenized statement
; Effects: Updates or removes a FOR loop frame.
BASIC_NEXT:
    SET 2,(IX+00H)
    JR NC,LE4E3
    CP 03H
    JR Z,LE4C7
    AND 82H
    JP NZ,LFD5A
    RST 08H
    DB 0EH                                ; error code 0EH: Type mismatch

LE4C7:
    CALL STACK_TO_NUMERIC_REGS
    EX DE,HL
    DB 21H                                ; = LD HL,09FDH when run from here, hiding the entry below

LE4CC:
    ADD IY,BC
    LD A,2BH
    CALL LFCE7
    JR C,LE4A9
    LD C,05H
    ADD HL,BC
    LD C,2BH
    LD A,(HL)
    CP E
    JR NZ,LE4CC
    INC HL
    LD A,(HL)
    CP D
    JR NZ,LE4CC

LE4E3:
    LD A,2BH
    CALL LFCE7
    JR C,LE4A9
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    PUSH DE
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    PUSH DE
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    PUSH DE
    INC HL
    EX DE,HL
    LD DE,0022H
    ADD IY,DE
    CALL LFA63
    DEC DE
    LD HL,FFF7H
    ADD HL,DE
    LD BC,0009H
    LDDR
    INC DE
    PUSH DE
    POP IY
    BIT 7,(IY+08H)
    PUSH AF
    CALL FP_ADD
    POP AF
    POP HL
    PUSH AF
    PUSH IY
    CALL LFB28
    POP IY
    CALL FP_COMPARE
    POP BC
    POP DE
    POP HL
    LD A,C
    RLA
    JR NC,LE533
    JR Z,LE536
    JP P,LE4AB
    XOR A

LE533:
    JP M,LE4AB

LE536:
    LD (CUR_LINE),DE
    LD DE,FFD5H
    ADD IY,DE
    JP DB81H

; OUT routine.

; BASIC_OUT - Implements the BASIC OUT statement.
; Entry: Port and byte expressions
; Effects: Writes a hardware I/O port.
BASIC_OUT:
    CALL LFB1B
    LD C,A
    LD A,A4H
    CALL LFD54
    CALL LFB1B
    OUT (C),A
    JP DBB1H

; POKE routine.

; BASIC_POKE - Implements the BASIC POKE statement.
; Entry: Address and byte expressions
; Effects: Writes memory, including video-memory address translation.
BASIC_POKE:
    CALL LFCFF
    LD A,A4H
    CALL LFD54
    CALL LFB1B
    LD C,A
    CALL LFFE7
    JR C,LE567
    DI
    OUT (02H),A

LE567:
    LD (HL),C
    LD A,70H
    OUT (02H),A
    EI
    JP DBB1H

; LPRINT routine.

LE570:
    LD C,40H
    DB 11H                                ; = LD DE,200EH when run from here, hiding the entry below

; BASIC_PRINT - Implements BASIC PRINT and OUTPUT.
; Entry: Tokenized print list and formatting
; Effects: Writes to the selected output device.
BASIC_PRINT:
    LD C,20H
    LD (IX+05H),C
    EX AF,AF'
    XOR A
    EX AF,AF'
    SCF

LE57C:
    CALL NC,LFC43
    CP AFH
    JR NZ,LE5A6
    CALL LFC43
    CALL EVAL_STRING_EXPRESSION
    LD HL,INPUT_BUFFER
    LD (HL),7FH
    PUSH HL
    CALL LFB41
    POP HL
    INC HL
    LD E,(HL)
    LD D,00H
    INC HL
    LD (172EH),HL
    ADD HL,DE
    LD (HL),D
    LD (1730H),HL
    EX AF,AF'
    SET 0,A
    EX AF,AF'
    JR LE5BA

LE5A6:
    CP B0H
    JR NZ,LE5CC
    CALL STACK_NUMBER_TO_BYTE
    LD C,A
    LD A,A4H
    CALL LFD54
    CALL LFB1B
    LD B,A
    CALL LE68B

LE5BA:
    EXX
    LD A,B
    EXX

LE5BD:
    EX AF,AF'
    OR A
    EX AF,AF'

    CP A4H
    JR Z,LE57C
    CP FDH
    JR Z,LE5D6
    JR NC,LE5D9

LE5CA:
    RST 08H
    DB 01H                                ; error code 01H: Not understood

LE5CC:
    CALL LFC00
    JR Z,LE5BD
    EX AF,AF'
    JR C,LE5CA
    EX AF,AF'
    SCF

LE5D6:
    CALL NC,LFC43

LE5D9:
    EX AF,AF'
    SCF
    EX AF,AF'

LE5DC:
    EX AF,AF'
    BIT 0,A
    PUSH AF
    CALL NZ,LE69C
    POP AF
    EX AF,AF'
    EXX
    LD A,B
    EXX
    CP A4H
    JR NZ,LE5F2
    LD A,09H
    CALL LFE9A
    DB C2H                                ; = JP NZ,A0FEH when run from here, hiding the entry below

LE5F2:
    CP A0H
    JR NZ,LE5FE
    CALL LFC43
    EX AF,AF'
    OR A
    JP LE67F

LE5FE:
    CP B6H
    JR NZ,LE617
    LD A,96H
    CALL LFD51
    CALL LFB1B
    LD B,A
    LD A,95H
    CALL LFD54
    LD C,00H
    CALL LE68B
    JR LE67D

LE617:
    CP FDH
    JR NC,LE683
    EX AF,AF'
    BIT 0,A
    PUSH AF
    JR Z,LE638
    EX AF,AF'
    LD HL,(172EH)
    LD DE,(1730H)
    OR A
    SBC HL,DE
    JR C,LE639
    LD HL,INPUT_BUFFER+2
    LD (172EH),HL
    CALL LE69C
    EX AF,AF'

LE638:
    EX AF,AF'

LE639:
    EXX
    LD A,B
    EXX
    CP 02H
    JR NC,LE665
    CALL EVAL_STRING_EXPRESSION
    RLA
    JR NC,LE669
    POP AF
    LD HL,FF9CH                           ; EXT ROM routine LFF9C, entered through the FFF0H gateway
    JP NZ,CALL_EXTENSION_HL
    OR FFH

LE64F:
    PUSH IY
    POP HL
    INC HL
    PUSH AF
    CALL PRINT_LENGTH_TEXT
    CALL DISCARD_STRING

LE65A:
    POP AF
    DEC A
    JP M,LE67D
    PUSH AF
    CALL LFEC7
    JR LE65A

LE665:
    CALL EVAL_NUMERIC_EXPRESSION
    RLA

LE669:
    JP NC,LFD5A
    POP AF
    LD HL,FCB5H                           ; EXT ROM routine LFCB5, entered through the FFF0H gateway
    JP NZ,CALL_EXTENSION_HL
    PUSH IY
    POP HL
    INC HL
    CALL LFEBA

LE67A:
    CALL DISCARD_NUMBER

LE67D:
    EX AF,AF'
    SCF

LE67F:
    EX AF,AF'
    JP LE5DC

LE683:
    EX AF,AF'
    CALL C,LFE93
    EX AF,AF'
    JP DBB4H

LE68B:
    LD A,(FUNC_CLASS)
    CP 20H
    JR Z,LE695
    CP 00H
    RET NZ

LE695:
    OR 03H
    CALL RST30_A_ENTRY
    RST 10H
    RET

LE69C:
    LD DE,(1730H)
    LD HL,(172EH)

LE6A3:
    LD A,(HL)
    OR A
    SBC HL,DE
    RET NC
    ADD HL,DE
    PUSH HL
    LD HL,E6BDH
    LD BC,000AH
    CPIR
    POP HL
    RET Z
    CALL LFE9A
    INC HL
    LD (172EH),HL
    JR LE6A3

; Special format-control characters used by PRINT USING.
    DB 3CH, 3EH, 23H, 2AH, 25H, 2BH, 2DH, 24H, 5EH, 2EH                               ; |<>#*%+-$^.|

; RANDOMIZE routine.

LE6C7:
    LD A,R
    LD (RND_STATE),A
    LD HL,(TIME)
    LD (RND_STATE+1),HL
    CALL LE6D8
    JP DBB1H

LE6D8:
    LD B,10H
    LD A,(RND_STATE)
    LD HL,(RND_STATE+1)

LE6E0:
    LD C,A
    RRCA
    RRCA
    RRCA
    XOR C
    RLA
    RLA
    ADC HL,HL
    LD A,C
    ADC A,A
    DJNZ LE6E0
    LD (RND_STATE+1),HL
    LD (RND_STATE),A
    RET

; RESTORE routine.

LE6F4:
    LD HL,(TEXT_START)
    CP 02H
    JR NZ,LE701
    CALL LFBDE
    CALL LFC43

LE701:
    LD (DATA_LINE),HL
    LD HL,0000H
    LD (DATA_PTR),HL
    JP DBB4H

LE70D:
    RST 08H
    DB 09H                                ; error code 09H: No GOSUB

; RETURN routine.

; BASIC_RETURN - Implements the BASIC RETURN statement.
; Entry: BASIC stack contains a GOSUB frame
; Effects: Restores BASIC execution position.
BASIC_RETURN:
    LD A,06H
    CALL LFCE7
    JR C,LE70D
    SET 2,(IX+00H)
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    LD C,(HL)
    INC HL
    LD B,(HL)
    LD (CUR_LINE),BC
    INC HL
    LD A,(HL)
    LD (COND_FLAG),A
    INC HL
    PUSH HL
    POP IY
    EX DE,HL
    JP DB81H

; GRAPHICS routine.

; BASIC_GRAPHICS - Implements the BASIC GRAPHICS statement.
; Entry: Tokenized graphics parameters
; Effects: Calls graphics OS routines.
BASIC_GRAPHICS:
    CALL LFB1B
    LD C,00H
    CP 02H
    JR Z,LE747
    INC C
    CP 04H
    JR Z,LE747
    INC C
    CP 10H
    JP NZ,LFB14

LE747:
    RST 30H
    DB 04H                                ; RST 30H fn 04H: video 4 -> VID_MODE
    RST 10H
    JP DBB1H

LE74D:
    RST 30H
    DB 08H                                ; RST 30H fn 08H: video 8 -> B_ON

LE74F:
    CALL LFC43
    JR NC,LE78D

; PLOT routine.

; BASIC_PLOT - Implements the BASIC PLOT statement.
; Entry: Tokenized coordinates and optional PAINT modifier
; Effects: Updates graphics position and video memory.
BASIC_PLOT:
    JR NC,LE782
    CP BEH
    JR NZ,LE75F

; PLOT PAINT routine.
    RST 30H
    DB 0AH                                ; RST 30H fn 0AH: video 10 -> PAINT
    RST 10H
    JR LE77A

LE75F:
    CP A0H
    JR Z,LE74D
    CP A4H
    JR Z,LE784
    CALL LFAC4
    LD C,L
    LD B,H
    LD A,A4H
    CALL LFD54
    CALL LFAC4
    EX DE,HL
    PUSH AF
    RST 30H
    DB 06H                                ; RST 30H fn 06H: video 6 -> B_ABS
    RST 10H
    POP AF

LE77A:
    CP A0H
    JR Z,LE74D
    CP A4H
    JR Z,LE784

LE782:
    RST 30H
    DB 08H                                ; RST 30H fn 08H: video 8 -> B_ON

LE784:
    RST 30H
    DB 09H                                ; RST 30H fn 09H: video 9 -> B_OFF
    EXX
    LD A,B
    EXX
    CP FDH
    JR C,LE74F

LE78D:
    JP DBB4H

; BASIC_SET - Dispatches BASIC SET subcommands such as MODE, INK, PAPER, and PALETTE.
; Entry: Tokenized SET subcommand and parameters
; Effects: Updates video, editor, or keyboard settings.
BASIC_SET:
    LD HL,E7E4H
    CALL LFD12
    JR C,LE7DA
    CALL LFB1B

LE79B:
    LD E,00H

LE79D:
    LD (RST30_A_FN),A
    LD HL,1941H
    PUSH HL
    LD B,7FH

LE7A6:
    DEC E
    JP P,LE7B4
    EXX
    LD A,B
    EXX
    CP A4H
    JR NZ,LE7CD
    CALL LFC43

LE7B4:
    CP 02H
    JR NC,LE7C5
    CALL EVAL_STRING_EXPRESSION
    LD HL,193FH
    LD (HL),7FH
    CALL LFB41
    JR LE7D2

LE7C5:
    CALL LFB1B
    LD (HL),A
    INC HL
    DJNZ LE7A6
    INC B

LE7CD:
    LD (HL),00H
    INC HL
    DJNZ LE7CD

LE7D2:
    POP DE
    CALL 001EH
    RST 10H
    EXX
    LD A,B
    EXX

LE7DA:
    CP A0H
    JP NZ,DBB4H
    CALL LFC43
    JR BASIC_SET

; SET subcommand dispatch table; token plus relative routine selector.

; SET MODE routine.

; SET STYLE routine.

; SET INK routine.

; SET PAPER routine.

; SET DELAY routine.

; SET RATE routine.
    DB C3H, 11H, B7H, 12H, C4H, 13H, BCH, 14H, C7H, 15H, B9H, 16H, C8H, 24H, BDH, 2DH ; |.............$.-|
    DB AEH, 34H, 00H, 1EH, 00H, 21H, 1EH, 01H, 21H, 1EH, 02H, 21H, 1EH, 03H, 21H, 1EH ; |.4...!..!..!..!.|
    DB 1AH, 21H, 1EH, 1CH, 16H, 00H, 21H, 4BH, 0BH, 19H, CDH, 1BH, FBH, 77H           ; |.!....!K.....w|

    EXX
    LD A,B
    EXX
    RET

; SET CHARACTER routine.
    POP HL
    POP HL
    CALL LFB1B
    LD C,A
    LD A,0BH
    JP LE79B

; SET PALETTE routine.
    POP HL
    POP HL
    LD A,0CH
    LD E,01H
    JP LE79D

; SET BORDER routine.
    CALL LFB1B
    ADD A,A
    LD (BORDER),A
    JR E812H

; SOUND routine.

; BASIC_SOUND - Implements the BASIC SOUND statement.
; Entry: Tokenized pitch, volume, duration, and related parameters
; Effects: Programs sound through the OS sound routine.
BASIC_SOUND:
    LD HL,CAS_OUT_NAME
    LD (SOUND_PITCH),HL
    LD HL,3207H
    LD (USRTAB+11),HL
    LD HL,TONE_REPLACE
    LD (HL),FFH
    SCF

LE845:
    CALL NC,LFC43
    CP A0H
    JR Z,LE86F
    LD HL,E87BH
    CALL LFD12
    CP A4H
    JR Z,LE845
    PUSH AF
    LD DE,(SOUND_PITCH)
    LD BC,(USRTAB+11)
    RST 30H
    DB 33H                                ; RST 30H fn 33H: sound 3 -> TONE_SET
    XOR A
    LD (TONE_REPLACE),A
    POP AF
    CP FDH
    JR NC,LE878
    CP A0H
    JP NZ,LFD5A

LE86F:
    XOR A
    LD (TONE_REPLACE),A
    CALL LFC43
    JR C,LE845

LE878:
    JP DBB4H

; SOUND subcommand dispatch table; same format as SET table.
    DB BBH, 05H, C6H, 0CH, B3H, 0BH, 00H, CDH, C4H, FAH, 22H, 2AH, 17H, F0H, CFH, 04H ; |.........."*....|
    DB F6H, 37H, 11H, 2CH, 00H, 38H, 01H, 13H, CDH, 1BH, FBH                          ; |.7.,.8.....|
    LD (DE),A
    EXX
    LD A,B
    EXX
    RET

; BASIC CLOSE routine.

LE89B:
    LD C,50H
    CALL LFBEE
    CP E1H
    JR Z,LE8AF
    SET 7,C
    CP FDH
    JR NC,LE8B2
    CP ECH
    JP NZ,LFD5A

LE8AF:
    CALL LFC43

LE8B2:
    LD A,C
    AND 7FH
    CP 50H
    JR Z,LE8BD
    CP 60H
    JR NZ,LE8C7

LE8BD:
    LD A,C
    OR 04H
    CALL RST30_A_ENTRY
    RST 10H
    JP DBB1H

LE8C7:
    RST 08H
    DB FFH                                ; error code FFH

; OPEN BASIC routine.

; BASIC_OPEN - Implements the BASIC OPEN statement.
; Entry: Tokenized file and device parameters
; Effects: Opens a file through the selected OS device.
BASIC_OPEN:
    LD C,50H
    CALL LFBEE
    EX AF,AF'
    LD A,C
    CP 50H
    JR Z,LE8D8
    CP 60H
    JR NZ,LE8C7

LE8D8:
    OR 03H
    LD C,A
    EX AF,AF'
    LD DE,19CEH
    CP E1H
    JR Z,LE8F2
    SET 7,C
    CP 02H
    JR C,LE8F9
    CP FDH
    JR NC,LE905
    CP ECH
    JP NZ,LFD5A

LE8F2:
    CALL LFC43
    CP 02H
    JR NC,LE905

LE8F9:
    PUSH BC
    CALL EVAL_STRING_EXPRESSION
    PUSH IY
    CALL DISCARD_STRING
    POP DE
    INC DE
    POP BC

LE905:
    LD A,C
    LD (CAS_BUFFER),A
    CALL RST30_A_ENTRY
    RST 10H
    JP DBB1H

; GET routine.

LE910:
    LD C,10H
    CALL LFBEE
    EX AF,AF'
    LD A,C
    OR 81H
    CALL RST30_A_ENTRY
    PUSH IY
    POP HL
    CP ECH
    JR Z,LE929
    OR A
    RST 10H
    DEC HL
    LD (HL),C
    INC A
    DB 0EH                                ; = LD C,AFH when run from here, hiding the entry below

LE929:
    XOR A
    DEC HL
    LD (HL),A
    DEC HL
    LD (HL),01H
    EX AF,AF'
    CP FDH
    JP NC,DBB4H
    PUSH HL
    POP IY
    CP 01H
    JR NZ,LE93F
    EXX
    BIT 2,C

LE93F:
    JP NZ,LFD5A
    BIT 3,C
    CALL NZ,SYMBOL_CREATE_VALUE
    EXX
    CALL STACK_TO_NUMERIC_REGS
    CALL LFB41
    JP DBB1H

; LOAD routine.

; BASIC_LOAD - Implements the BASIC LOAD statement.
; Entry: Tokenized filename and options
; Effects: Loads a BASIC program or data through an OS device.
BASIC_LOAD:
    CALL LEA35
    CALL DE10H
    PUSH HL
    LD DE,(CAS_HEADER+2)
    PUSH DE
    CALL LFC99
    POP BC
    POP DE
    PUSH DE
    LD A,(FUNC_CLASS)
    OR 82H
    CALL RST30_A_ENTRY
    PUSH AF
    CALL NZ,DE10H
    POP AF
    RST 10H
    CALL BASIC_WORKSPACE_INIT
    CALL LEA5A
    POP HL
    LD A,(CAS_HEADER+4)
    OR A
    JP Z,DADAH
    JP DE23H

; SAVE routine.

; BASIC_SAVE - Implements the BASIC SAVE statement.
; Entry: Tokenized filename and options
; Effects: Saves a BASIC program or data through an OS device.
BASIC_SAVE:
    XOR A
    CALL LE9FB
    LD DE,BASIC_RAM_START
    LD B,10H
    XOR A

LE98C:
    DEC DE
    LD (DE),A
    DJNZ LE98C
    PUSH DE
    INC A
    LD (CAS_HEADER+1),A
    LD A,(AUTORUN)
    LD (CAS_HEADER+4),A
    CALL DD41H
    INC HL
    LD DE,(TEXT_START)
    XOR A
    LD (AUTORUN),A
    SBC HL,DE
    LD (CAS_HEADER+2),HL
    EX (SP),HL
    PUSH DE
    LD B,10H
    LD A,(FUNC_CLASS)
    OR 01H
    LD (RST30_A_FN),A

LE9B8:
    LD C,(HL)
    INC HL
    PUSH BC
    CALL 001EH
    RST 10H
    POP BC
    DJNZ LE9B8
    POP DE
    POP BC
    LD A,(FUNC_CLASS)
    OR 02H
    CALL RST30_A_ENTRY
    RST 10H
    CALL LEA5A
    JP DBB1H

; VERIFY routine.

LE9D3:
    CALL LEA35
    CALL DD41H
    INC HL
    LD DE,(TEXT_START)
    OR A
    SBC HL,DE
    LD BC,(CAS_HEADER+2)
    SBC HL,BC
    JP NZ,LEA58
    LD A,(FUNC_CLASS)
    OR 85H
    CALL RST30_A_ENTRY
    RST 10H
    CALL LEA5A
    JP DBB1H

LE9F9:
    LD A,80H

LE9FB:
    SET 3,(IX+00H)
    PUSH AF
    LD C,50H
    CALL LFBEE
    EX AF,AF'
    LD A,C
    CP 50H
    JR Z,LEA10
    CP 60H
    JP NZ,LE8C7

LEA10:
    POP BC
    OR B
    LD (FUNC_CLASS),A
    OR 03H
    LD (RST30_A_FN),A
    EX AF,AF'
    LD DE,19CEH
    CP 02H
    JR NC,LEA2C
    CALL EVAL_STRING_EXPRESSION
    PUSH IY
    CALL DISCARD_STRING
    POP DE
    INC DE

LEA2C:
    XOR A
    LD (CAS_BUFFER),A
    CALL 001EH
    RST 10H
    RET

LEA35:
    CALL LE9F9
    LD A,(FUNC_CLASS)
    OR 81H
    LD (RST30_A_FN),A
    LD HL,CAS_HEADER
    PUSH HL
    LD B,10H

LEA46:
    PUSH BC
    CALL 001EH
    RST 10H
    LD (HL),C
    INC HL
    POP BC
    DJNZ LEA46
    POP HL
    OR (HL)
    JR NZ,LEA58
    INC HL
    INC A
    SUB (HL)
    RET Z

LEA58:
    RST 08H
    DB 10H                                ; error code 10H: Bad file

LEA5A:
    LD A,(FUNC_CLASS)
    OR 04H
    CALL RST30_A_ENTRY
    RST 10H
    RES 3,(IX+00H)
    RET

; EVAL_NUMERIC_ARGUMENT - Evaluates a parenthesized numeric function argument onto the BASIC
; stack.
; Entry: HL' points into tokenized BASIC source
; Exit: Numeric value pushed on BASIC stack; IY updated
; Effects: Consumes source tokens and grows the BASIC stack.
EVAL_NUMERIC_ARGUMENT:
    LD A,96H
    CALL LFD54
    CALL EVAL_NUMERIC_EXPRESSION

LEA70:
    LD A,95H
    JP LFD54

LEA75:
    LD A,96H
    CALL LFD54
    CALL EVAL_STRING_EXPRESSION
    LD A,95H
    JP LFD54

LEA82:
    LD HL,(RST18_NEXT)
    LD A,(HL)
    INC HL
    LD (RST18_NEXT),HL
    LD L,A
    XOR A
    LD H,A
    PUSH HL
    ADD HL,HL
    ADD HL,HL
    ADD HL,HL
    POP DE
    SBC HL,DE

    LD DE,NUMBERED_FP_CONSTANTS
    ADD HL,DE
    JR LEAA2

LEA9A:
    LD HL,FP_Y
    JR LEAA2

LEA9F:
    LD HL,FP_X

LEAA2:
    CALL LFC8E
    LD DE,0006H
    ADD HL,DE
    PUSH IY
    POP DE
    DEC DE
    LDD
    XOR A
    LD (DE),A
    DEC DE
    LD BC,0006H
    LDDR
    LD A,09H
    LD (DE),A
    PUSH DE
    POP IY
    RET

LEABE:
    LD DE,FP_Y
    JR LEAC6

LEAC3:
    LD DE,FP_X

LEAC6:
    CALL LEAD5
    PUSH HL
    POP IY
    RET

LEACD:
    LD DE,FP_Y
    JR LEAD5

LEAD2:
    LD DE,FP_X

LEAD5:
    PUSH IY
    POP HL
    INC HL
    LD BC,0006H
    LDIR
    INC HL
    LDI
    RET

FN_REC_ABS:
    DW 0000H                              ; link to previous function record
    DB 3, "ABS", 0AH                      ; name, then type (numeric result)

; BASIC_ABS - Implements the BASIC ABS function.
; Entry: Argument on BASIC stack
; Exit: Absolute value on BASIC stack
BASIC_ABS:
    CALL EVAL_NUMERIC_ARGUMENT
    RES 7,(IY+08H)
    RET

LEAF1:
    CALL LFC43
    CALL EVAL_NUMERIC_ARGUMENT
    LD BC,0000H
    LD A,(IY+08H)
    AND 80H
    LD C,A
    PUSH BC
    CALL NZ,FP_NEGATE
    RST 18H
    DB 08H, 85H, 01H                      ; RST 18H: op08, CONST 1
    CALL FP_COMPARE
    JP M,LEB1A
    JR Z,LEB1A
    POP BC
    LD B,02H
    PUSH BC
    RST 18H
    DB 05H, 01H, 06H, 01H, 8AH            ; RST 18H: CONST 1, op06, FP_DIV, op0A

LEB1A:
    RST 18H
    DB 06H, 85H, 02H                      ; RST 18H: op06, CONST 2
    CALL FP_COMPARE
    JP M,LEB3C
    JR Z,LEB3C
    POP BC
    INC B
    PUSH BC
    RST 18H
; RST 18H: CONST 4, op06, FP_MUL, CONST 0, FP_SUB, CONST 0, FP_SUB, op06, FP_ADD, CONST 3, op06, FP_ADD, FP_DIV, op0A
    DB 05H, 04H, 06H, 02H, 05H, 00H, 03H, 05H, 00H, 03H, 06H, 00H, 05H, 03H, 06H, 00H ; |................|
    DB 01H, 8AH                                                                       ; |..|

LEB3C:
    RST 18H
; RST 18H: op06, op0C, op0C, op0C, FP_MUL, op08, CONST 6, FP_MUL, CONST 5, FP_ADD, op06, FP_MUL, op06, CONST 8, FP_ADD, op06, FP_MUL, CONST 7, FP_ADD, FP_DIV, FP_MUL, FP_ADD
    DB 06H, 0CH, 0CH, 0CH, 02H, 08H, 05H, 06H, 02H, 05H, 05H, 00H, 06H, 02H, 06H, 05H ; |................|
    DB 08H, 00H, 06H, 02H, 05H, 07H, 00H, 01H, 02H, 80H                               ; |..........|
    POP BC
    LD A,B
    CP 02H
    CALL NC,FP_NEGATE
    PUSH BC
    LD HL,C214H
    DEC B
    JR Z,LEB71
    LD HL,C206H
    DEC B
    JR Z,LEB71
    LD HL,C20DH
    DEC B
    JR NZ,LEB77

LEB71:
    CALL LEAA2
    CALL FP_ADD

LEB77:
    POP BC
    LD A,C
    XOR (IY+08H)
    LD (IY+08H),A
    RET

FN_REC_CHR_STR:
    DW FN_REC_ABS                         ; link to previous function record
    DB 4, "CHR$", 08H                     ; name, then type (string result)

BASIC_CHR_STR:
    CALL EVAL_NUMERIC_ARGUMENT
    CALL LFB1A
    PUSH IY
    POP HL
    DEC HL
    LD (HL),A
    DEC HL
    LD (HL),01H
    DEC HL
    LD (HL),01H
    PUSH HL
    POP IY
    RET

FN_REC_COS:
    DW FN_REC_CHR_STR                     ; link to previous function record
    DB 3, "COS", 0AH                      ; name, then type (numeric result)

; BASIC_COS - Implements the BASIC COS function.
; Entry: Argument on BASIC stack
; Exit: Cosine on BASIC stack
BASIC_COS:
    CALL EVAL_NUMERIC_ARGUMENT
    RST 18H
    DB 05H, 23H, 80H                      ; RST 18H: CONST 35, FP_ADD
    JP LEE5E

FN_REC_EXP:
    DW FN_REC_COS                         ; link to previous function record
    DB 3, "EXP", 0AH                      ; name, then type (numeric result)

BASIC_EXP:
    CALL EVAL_NUMERIC_ARGUMENT

LEBB8:
    RST 18H
    DB 0CH, 05H, 09H, 82H                 ; RST 18H: op0C, CONST 9, FP_MUL
    CALL LECAF
    LD A,(IY+08H)
    LD C,A
    AND 7FH
    CP 43H
    JR C,LEBD4
    OR C
    JP P,LF912

LEBCE:
    CALL DISCARD_NUMBER
    JP FP_CLEAR_NUMERIC

LEBD4:
    CALL LFA92
    CALL STACK_NUMBER_TO_INTEGER
    LD E,L
    LD D,H
    LD BC,007EH
    OR A
    ADC HL,BC
    JP M,LEBCE
    SLA C
    SBC HL,BC
    JP NC,LF912
    PUSH DE
    RST 18H

; RST 18H: op09, CONST 10, FP_MUL, FP_SUB, op07, CONST 11, FP_MUL, FP_SUB, op0C, op0C, FP_MUL, op08, CONST 16, FP_ADD, op06, FP_MUL, CONST 15, FP_ADD, op0B, CONST 14, op06, FP_MUL, CONST 13, FP_ADD, op06, FP_MUL, CONST 12, FP_ADD, FP_MUL, op08, op07, op06, FP_SUB, FP_DIV, CONST 0, FP_ADD, op0C, FP_ADD, op0A
    DB 09H, 05H, 0AH, 02H, 03H, 07H, 05H, 0BH, 02H, 03H, 0CH, 0CH, 02H, 08H, 05H, 10H ; |................|
    DB 00H, 06H, 02H, 05H, 0FH, 00H, 0BH, 05H, 0EH, 06H, 02H, 05H, 0DH, 00H, 06H, 02H ; |................|
    DB 05H, 0CH, 00H, 02H, 08H, 07H, 06H, 03H, 01H, 05H, 00H, 00H, 0CH, 00H, 8AH      ; |...............|
    POP DE
    BIT 0,E
    JR Z,LEC37
    PUSH DE
    RST 18H

    DB 06H, 05H, 20H, 02H, 8AH            ; RST 18H: op06, CONST 32, FP_MUL, op0A
    POP DE

    INC D
    DEC D
    INC DE
    JR Z,LEC37
    JP M,LEC37
    LD HL,FP_X+6
    INC (HL)
    DEC DE

LEC37:
    LD HL,FP_X+6
    XOR A
    OR E
    RRA
    ADD A,(HL)
    AND 7FH
    LD (HL),A
    JP LEA9F

FN_REC_FREE:
    DW FN_REC_EXP                         ; link to previous function record
    DB 4, "FREE", 0AH                     ; name, then type (numeric result)

BASIC_FREE:
    PUSH IY
    POP HL
    LD DE,(SYMBOL_TOP)
    OR A
    SBC HL,DE
    LD DE,0100H
    SBC HL,DE
    LD DE,7FFFH
    SBC HL,DE
    PUSH DE
    CALL INTEGER_TO_STACK
    POP HL
    CALL INTEGER_TO_STACK
    JP FP_ADD

FN_REC_IN:
    DW FN_REC_FREE                        ; link to previous function record
    DB 2, "IN", 0AH                       ; name, then type (numeric result)

; BASIC_IN - Implements the BASIC IN function.
; Entry: Port expression on BASIC stack
; Exit: Port byte on BASIC stack
; Effects: Reads a hardware I/O port.
BASIC_IN:
    CALL EVAL_NUMERIC_ARGUMENT
    CALL LFB1A
    LD C,A
    IN L,(C)
    LD H,00H
    JP INTEGER_TO_STACK

LEC7F:
    RET NC
    RLA
    RET NC
    CALL DISCARD_NUMBER
    RST 18H
    DB 85H, 27H                           ; RST 18H: CONST 39
    RET

LEC89:
    CALL LFC43
    RST 30H
    DB 93H                                ; RST 30H fn 93H: keyboard 3, input -> KB_STATUS
    RST 10H
    PUSH IY
    POP HL
    DEC HL
    LD A,C
    OR A
    JR Z,LEC9D
    RST 30H
    DB 91H                                ; RST 30H fn 91H: keyboard 1, input -> KB_CHIN
    RST 10H
    LD (HL),C
    DEC HL
    INC A

LEC9D:
    LD (HL),A
    DEC HL
    LD (HL),01H
    PUSH HL
    POP IY
    RET

FN_REC_INT:
    DW FN_REC_IN                          ; link to previous function record
    DB 3, "INT", 0AH                      ; name, then type (numeric result)

BASIC_INT:
    CALL EVAL_NUMERIC_ARGUMENT

LECAF:
    LD A,(IY+08H)
    AND 7FH
    SUB 40H
    JR C,LECD8
    INC A
    PUSH AF
    CALL LF707
    CALL FP_TEST_ZERO
    POP AF
    ADD A,03H
    LD B,A

LECC4:
    LD A,00H
    CALL C,FP_SET_BCD_NIBBLE
    INC B
    LD A,B
    CP 10H
    JR C,LECC4
    CALL LF707
    CALL FP_TEST_ZERO
    JP FP_NORMALIZE

LECD8:
    LD A,(IY+08H)
    OR A
    JP P,FP_CLEAR_NUMERIC
    CALL LFA08
    JP FP_NEGATE

FN_REC_IO:
    DW FN_REC_INT                         ; link to previous function record
    DB 2, "IO", 0AH                       ; name, then type (numeric result)

BASIC_IO:
    CALL EVAL_NUMERIC_ARGUMENT
    CALL LFB1A
    LD L,A
    LD H,00H
    LD DE,1941H
    ADD HL,HL
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    EX DE,HL
    JP INTEGER_TO_STACK

FN_REC_LEN:
    DW FN_REC_IO                          ; link to previous function record
    DB 3, "LEN", 0AH                      ; name, then type (numeric result)

BASIC_LEN:
    CALL LEA75

    PUSH IY
    CALL DISCARD_STRING
    POP HL

LED10:
    INC HL
    LD L,(HL)

LED12:
    LD H,00H
    JP INTEGER_TO_STACK

FN_REC_LOG:
    DW FN_REC_LEN                         ; link to previous function record
    DB 3, "LOG", 0AH                      ; name, then type (numeric result)

BASIC_LOG:
    CALL EVAL_NUMERIC_ARGUMENT

LED21:
    CALL LEAC3
    LD HL,FP_X+5
    XOR A
    LD D,A
    OR (HL)
    JP Z,LFB14
    INC HL
    LD E,(HL)
    INC E
    DEC E
    JP M,LFB14
    LD (HL),3FH
    PUSH DE
    PUSH HL
    RST 18H
    DB 06H, 85H, 20H                      ; RST 18H: op06, CONST 32
    CALL FP_COMPARE
    POP HL
    JR Z,LED45
    JP P,LED49

LED45:
    POP DE
    DEC DE
    INC (HL)
    PUSH DE

LED49:
    RST 18H
; RST 18H: op06, CONST 0, FP_SUB, CONST 0, FP_SUB, op06, CONST 1, FP_ADD, FP_DIV, op08, op0C, op0C, FP_MUL, op08, CONST 20, FP_MUL, CONST 19, FP_ADD, op06, FP_MUL, CONST 18, FP_ADD, op06, FP_MUL, op06, CONST 23, FP_ADD, op06, FP_MUL, CONST 22, FP_ADD, op06, FP_MUL, CONST 21, FP_ADD, FP_DIV, CONST 24, FP_ADD, FP_MUL
    DB 06H, 05H, 00H, 03H, 05H, 00H, 03H, 06H, 05H, 01H, 00H, 01H, 08H, 0CH, 0CH, 02H ; |................|
    DB 08H, 05H, 14H, 02H, 05H, 13H, 00H, 06H, 02H, 05H, 12H, 00H, 06H, 02H, 06H, 05H ; |................|
    DB 17H, 00H, 06H, 02H, 05H, 16H, 00H, 06H, 02H, 05H, 15H, 00H, 01H, 05H, 18H, 00H ; |................|
    DB 82H                                                                            ; |.|
    POP HL
    LD DE,003FH
    OR A
    SBC HL,DE
    CALL INTEGER_TO_STACK
    RST 18H
    DB 00H, 05H, 11H, 82H                 ; RST 18H: FP_ADD, CONST 17, FP_MUL
    RET

LED8B:
    CALL LFAA6
    CALL LEAD2
    CALL FP_DIV
    CALL LECAF
    RST 18H
    DB 02H, 03H, 8CH                      ; RST 18H: FP_MUL, FP_SUB, op0C
    RES 7,(IY+08H)
    CALL LEA9F
    CALL FP_COMPARE
    RET M
    JP FP_CLEAR_NUMERIC

LEDA9:
    CALL LFC43
    CALL LEA75
    PUSH IY
    CALL DISCARD_STRING
    POP HL
    INC HL
    LD A,(HL)
    OR A
    JP Z,LFB14
    JP LED10

FN_REC_PEEK:
    DW FN_REC_LOG                         ; link to previous function record
    DB 4, "PEEK", 0AH                     ; name, then type (numeric result)

; BASIC_PEEK - Implements the BASIC PEEK function.
; Entry: Address expression on BASIC stack
; Exit: Byte value on BASIC stack
; Effects: Reads memory, including video-memory address translation.
BASIC_PEEK:
    CALL EVAL_NUMERIC_ARGUMENT
    CALL LFD02
    CALL LFFE7
    JR C,LEDD4
    DI

; Memory paging: U U V S page layout.
    OUT (02H),A

LEDD4:
    LD L,(HL)
    LD A,70H
    OUT (02H),A
    EI
    JP LED12

FN_REC_PI:
    DW FN_REC_PEEK                        ; link to previous function record
    DB 2, "PI", 0AH                       ; name, then type (numeric result)

BASIC_PI:
    RST 18H
    DB 85H, 22H                           ; RST 18H: CONST 34
    RET

FN_REC_RND:
    DW FN_REC_PI                          ; link to previous function record
    DB 3, "RND", 0AH                      ; name, then type (numeric result)

BASIC_RND:
    CP 96H
    JR Z,LEE02
    CALL LE6D8
    CALL INTEGER_TO_STACK
    RST 18H
    DB 05H, 26H, 00H, 05H, 26H, 0CH, 00H, 81H ; RST 18H: CONST 38, FP_ADD, CONST 38, op0C, FP_ADD, FP_DIV
    RET

LEE02:
    CALL EVAL_NUMERIC_ARGUMENT
    CALL STACK_NUMBER_TO_INTEGER
    BIT 7,H
    JP NZ,LFB14
    LD A,H
    OR L
    JP Z,LFB14
    PUSH HL
    XOR A

LEE14:
    INC A
    ADD HL,HL
    JR NC,LEE14
    DEC A
    POP HL
    DEC HL

LEE1B:
    PUSH HL
    PUSH AF
    CALL LE6D8
    POP AF
    LD B,A

LEE22:
    SRL H
    RR L
    DJNZ LEE22
    EX DE,HL
    POP HL
    OR A
    SBC HL,DE
    ADD HL,DE
    JR C,LEE1B
    EX DE,HL
    JP INTEGER_TO_STACK

FN_REC_SGN:
    DW FN_REC_RND                         ; link to previous function record
    DB 3, "SGN", 0AH                      ; name, then type (numeric result)

BASIC_SGN:
    CALL EVAL_NUMERIC_ARGUMENT
    LD A,(IY+06H)
    OR A
    RET Z
    LD A,(IY+08H)
    PUSH AF
    CALL LFA08
    POP AF
    AND 80H
    OR (IY+08H)
    LD (IY+08H),A
    RET

FN_REC_SIN:
    DW FN_REC_SGN                         ; link to previous function record
    DB 3, "SIN", 0AH                      ; name, then type (numeric result)

; BASIC_SIN - Implements the BASIC SIN function.
; Entry: Argument on BASIC stack
; Exit: Sine on BASIC stack
BASIC_SIN:
    CALL EVAL_NUMERIC_ARGUMENT

LEE5E:
    RST 18H
    DB 05H, 22H, 05H, 22H, 80H            ; RST 18H: CONST 34, CONST 34, FP_ADD
    CALL LED8B
    LD A,(IY+08H)
    OR A
    RRA
    PUSH AF
    RST 18H
    DB 8AH                                ; RST 18H: op0A

LEE6F:
    RST 18H
    DB 06H, 85H, 23H                      ; RST 18H: op06, CONST 35
    POP AF
    PUSH AF
    CALL M,FP_NEGATE
    CALL FP_COMPARE
    POP BC
    PUSH BC
    JR Z,LEE9A
    XOR B
    JP M,LEE93
    RST 18H
    DB 06H, 85H, 22H                      ; RST 18H: op06, CONST 34
    POP AF
    XOR 40H
    PUSH AF
    CALL P,FP_NEGATE
    RST 18H
    DB 00H, 8AH                           ; RST 18H: FP_ADD, op0A
    JR LEE6F

LEE93:
    POP AF
    XOR 80H
    PUSH AF
    JP M,LEE6F

LEE9A:
    RST 18H
; RST 18H: op06, op0C, op0C, op0C, FP_MUL, op08, CONST 29, FP_MUL, CONST 28, FP_ADD, op06, FP_MUL, CONST 27, FP_ADD, op06, FP_MUL, CONST 26, FP_ADD, op06, FP_MUL, CONST 25, FP_ADD, op06, FP_MUL, FP_MUL, FP_ADD
    DB 06H, 0CH, 0CH, 0CH, 02H, 08H, 05H, 1DH, 02H, 05H, 1CH, 00H, 06H, 02H, 05H, 1BH ; |................|
    DB 00H, 06H, 02H, 05H, 1AH, 00H, 06H, 02H, 05H, 19H, 00H, 06H, 02H, 02H, 80H      ; |...............|
    POP AF
    ADD A,A
    RET P
    JP FP_NEGATE

FN_REC_SQR:
    DW FN_REC_SIN                         ; link to previous function record
    DB 3, "SQR", 0AH                      ; name, then type (numeric result)

; BASIC_SQR - Implements the BASIC SQR function.
; Entry: Argument on BASIC stack
; Exit: Square root on BASIC stack
BASIC_SQR:
    CALL EVAL_NUMERIC_ARGUMENT
    BIT 7,(IY+08H)
    JP NZ,LFB14
    LD A,(IY+06H)
    OR A
    RET Z
    CALL LEAC3
    LD HL,FP_X+6
    LD A,(HL)
    SUB 3FH
    PUSH AF
    LD (HL),3FH
    RST 18H
    DB 06H, 05H, 1FH, 02H, 05H, 1EH, 00H, 8BH ; RST 18H: op06, CONST 31, FP_MUL, CONST 30, FP_ADD, op0B
    LD B,04H

LEEED:
    PUSH BC
    RST 18H
    DB 07H, 06H, 07H, 01H, 07H, 03H, 05H, 00H, 02H, 00H, 8BH ; RST 18H: op07, op06, op07, FP_DIV, op07, FP_SUB, CONST 0, FP_MUL, FP_ADD, op0B
    POP BC
    DJNZ LEEED
    POP AF
    BIT 0,A
    JR Z,LEF0B
    PUSH AF
    RST 18H
    DB 07H, 05H, 20H, 02H, 8BH            ; RST 18H: op07, CONST 32, FP_MUL, op0B
    POP AF

    INC A

LEF0B:
    SRL A
    LD HL,FP_Y+6
    ADD A,(HL)
    AND 7FH
    LD (HL),A
    JP LEA9A

LEF17:
    LD A,(IY+06H)
    OR A
    JR NZ,LEF23
    CALL DISCARD_NUMBER
    JP LFA08

LEF23:
    LD A,(IY+0FH)
    OR A
    JP Z,DISCARD_NUMBER
    LD A,(IY+08H)
    AND 7FH
    CP 43H
    JR NC,LEF7D
    CALL LFA92
    CALL STACK_NUMBER_TO_INTEGER
    PUSH HL
    CALL LFA92
    CALL FP_ROUND_BCD
    POP HL
    PUSH HL
    CALL INTEGER_TO_STACK
    CALL FP_COMPARE
    POP HL
    JR NZ,LEF7D
    BIT 7,H
    PUSH AF
    CALL NZ,LFC86
    PUSH HL
    RST 18H
    DB 0AH, 0AH, 85H, 01H                 ; RST 18H: op0A, op0A, CONST 1

LEF57:
    POP HL
    PUSH HL
    BIT 0,L
    JR Z,LEF63
    CALL LEA9F
    CALL FP_MUL

LEF63:
    POP HL
    SRL H
    RR L
    LD A,H
    OR L
    JR Z,LEF74
    PUSH HL
    RST 18H
    DB 06H, 06H, 02H, 8AH                 ; RST 18H: op06, op06, FP_MUL, op0A
    JR LEF57

LEF74:
    POP AF
    RET Z
    RST 18H
    DB 0BH, 05H, 01H, 07H, 81H            ; RST 18H: op0B, CONST 1, op07, FP_DIV
    RET

LEF7D:
    CALL LEABE
    CALL LED21
    CALL LEA9A
    CALL FP_MUL
    JP LEBB8

FN_REC_STR_STR:
    DW FN_REC_SQR                         ; link to previous function record
    DB 4, "STR$", 08H                     ; name, then type (string result)

BASIC_STR_STR:
    CALL EVAL_NUMERIC_ARGUMENT
    CALL FP_TO_ASCII
    CALL DISCARD_NUMBER
    JP LFA7D

FN_REC_STRING_STR:
    DW FN_REC_STR_STR                     ; link to previous function record
    DB 7, "STRING$", 08H                  ; name, then type (string result)

BASIC_STRING_STR:
    LD A,96H
    CALL LFD54
    CALL LFB1B
    PUSH AF
    LD A,A4H
    CALL LFD54
    CP 02H
    JR NC,LEFD0
    CALL EVAL_STRING_EXPRESSION
    LD A,(IY+01H)
    OR A
    JR Z,LEFCD
    LD A,(IY+02H)
    CALL DISCARD_STRING
    DB 01H                                ; = LD BC,F5C1H when run from here, hiding the entry below

LEFCD:
    POP BC
    PUSH AF
    SCF

LEFD0:
    CALL NC,LFB1B
    PUSH AF
    CALL LEA70
    POP DE
    PUSH IY
    POP HL
    DEC HL
    POP AF
    INC A
    JP Z,LF912
    DEC A
    JR Z,LEFE9
    LD B,A

LEFE5:
    LD (HL),D
    DEC HL
    DJNZ LEFE5

LEFE9:
    LD (HL),A
    DEC HL
    LD (HL),01H
    PUSH HL
    POP IY
    RET

FN_REC_TAN:
    DW FN_REC_STRING_STR                  ; link to previous function record
    DB 3, "TAN", 0AH                      ; name, then type (numeric result)

BASIC_TAN:
    CALL EVAL_NUMERIC_ARGUMENT
    CALL LEACD
    CALL LEE5E
    RST 18H
    DB 07H, 05H, 23H, 80H                 ; RST 18H: op07, CONST 35, FP_ADD
    CALL LEE5E
    JP FP_DIV

FN_REC_USR:
    DW FN_REC_TAN                         ; link to previous function record
    DB 3, "USR", 0AH                      ; name, then type (numeric result)

; USR converts the BASIC argument to integer form, calls user machine code, then converts HL back
; to a BASIC number.

; -----------------------------------------------------------------------------
; BASIC USR FUNCTION
; -----------------------------------------------------------------------------
;
; Evaluates the USR machine-code function and converts its result back to a BASIC number.
;
; USR accepts a machine-code address and a numeric argument from the BASIC expression machinery.
; It validates the argument types, converts the BASIC number to the integer representation
; expected by user code, calls the supplied address, and converts the returned integer in HL back
; into the nine-byte BASIC floating-point format.
;
; Entry:
;   Machine-code address and numeric argument prepared by the expression evaluator.
;
; Exit:
;   A numeric result is pushed on the BASIC stack.
;
; Effects:
;   Calls arbitrary user machine code; user code may alter machine state and memory.
;
; Destroys:
;   AF, BC, DE, HL, IY and user-defined registers according to the called code.
; -----------------------------------------------------------------------------
BASIC_USR:
    LD A,96H
    CALL LFD54
    CALL EVAL_NUMERIC_EXPRESSION
    CP 95H
    JR Z,LF029
    LD A,A4H
    CALL LFD54
    CALL LFAC4
    LD A,95H

LF029:
    CALL LFD54
    LD DE,INTEGER_TO_STACK
    PUSH DE
    PUSH HL
    CALL LFD02
    EX (SP),HL
    RET

FN_REC_VAL:
    DW FN_REC_USR                         ; link to previous function record
    DB 3, "VAL", 0AH                      ; name, then type (numeric result)

BASIC_VAL:
    CALL LEA75
    PUSH IY
    POP HL
    INC HL
    PUSH HL
    LD C,(HL)
    LD B,00H
    ADD HL,BC
    INC HL
    LD A,(HL)
    LD (HL),B
    EX (SP),HL
    PUSH AF
    INC HL
    CALL ASCII_TO_FP
    CALL LEC7F
    POP AF
    POP HL
    LD (HL),A
    DEC HL
    EX DE,HL
    PUSH IY
    POP HL
    LD BC,0009H
    ADD HL,BC
    DEC HL
    LDDR
    INC DE
    PUSH DE
    POP IY
    RET

FN_REC_VARPTR:
    DW FN_REC_VAL                         ; link to previous function record
    DB 6, "VARPTR", 0AH                   ; name, then type (numeric result)

BASIC_VARPTR:
    LD A,96H
    CALL LFD54
    AND FDH
    CP 01H

LF07C:
    JP NZ,LFB14
    EXX
    LD A,C
    AND 7FH
    LD (SYMBOL_TYPE),A
    AND 0CH
    JR NZ,LF07C
    EXX
    CALL STACK_TO_NUMERIC_REGS
    CALL INTEGER_TO_STACK
    JP LEA70

FN_REC_VERNUM:
    DW FN_REC_VARPTR                      ; link to previous function record
    DB 6, "VERNUM", 0AH                   ; name, then type (numeric result)

BASIC_VERNUM:
    LD HL,000CH
    JP INTEGER_TO_STACK

LF0A4:
    CALL LFC43

; Numeric expression precedence is implemented as nested layers: relations, +/-, */,
; exponentiation, functions, parentheses, and variables.

; -----------------------------------------------------------------------------
; NUMERIC EXPRESSION EVALUATOR
; -----------------------------------------------------------------------------
;
; Evaluates the token stream for a numeric expression using BASIC operator precedence.
;
; The evaluator is layered from OR/XOR through AND, NOT, relations, addition/subtraction,
; multiplication/division, exponentiation, functions, parentheses, and variables. Each layer
; consumes the next token only when its operator is present, recursively evaluates higher-priority
; terms, then applies the operation to values on the BASIC stack.
;
; The routine is entered at different points depending on whether the first token has already been
; fetched. It leaves the expression result on the BASIC stack and carries the source pointer in
; the interpreter workspace.
;
; Entry:
;   HL' points into tokenized BASIC source; interpreter flags select fetched/not-fetched entry.
;
; Exit:
;   One numeric value remains on the BASIC stack; source pointer advances past the expression.
;
; Effects:
;   Consumes tokens and grows/shrinks the BASIC stack.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers, IY as stack pointer.
; -----------------------------------------------------------------------------
EVAL_NUMERIC_EXPRESSION:
    EXX
    LD A,B
    EXX
    CALL EXPR_AND_LAYER

; The OR/XOR stage is a left-fold over already evaluated AND terms: fetch the operator, evaluate
; one right term, combine the integer pairs, and repeat. It is not a precedence-free scan of the
; whole source.

; -----------------------------------------------------------------------------
; OR/XOR EXPRESSION LOOP
; -----------------------------------------------------------------------------
;
; Consumes repeated OR and XOR operators after the higher-priority expression layer has produced a
; value.
;
; The loop fetches the next token, evaluates the right operand through the AND layer, combines the
; two integer-like values, and repeats while another OR/XOR token is present.
; -----------------------------------------------------------------------------
EXPR_OR_XOR_LOOP:
    EXX
    LD A,B
    EXX
    CP BFH
    JR Z,LF0B8
    CP B2H
    RET NZ
    SCF

LF0B8:
    PUSH AF
    CALL LFC43
    CALL EXPR_AND_LAYER
    CALL LFABF
    POP AF
    JR C,LF0CC
    LD A,L
    OR E
    LD L,A
    LD A,H
    OR D
    JR LF0D1

LF0CC:
    LD A,L
    XOR E
    LD L,A
    LD A,H
    OR D

LF0D1:
    LD H,A
    CALL INTEGER_TO_STACK
    JR EXPR_OR_XOR_LOOP

; AND is evaluated below OR/XOR but above NOT. The loop leaves the source pointer at the first
; token that belongs to the relation or statement layer, allowing the enclosing parser to continue
; without refetching it.

; -----------------------------------------------------------------------------
; AND EXPRESSION LAYER
; -----------------------------------------------------------------------------
;
; Bridges the OR/XOR layer to repeated bitwise AND operations.
;
; This entry evaluates the next lower-priority term, then lets the adjacent loop recognize AND
; tokens and combine the two stack integers without disturbing the source continuation state.
; -----------------------------------------------------------------------------
EXPR_AND_LAYER:
    CALL EXPR_NOT_LAYER

LF0DA:
    EXX
    LD A,B
    EXX
    CP C9H
    RET NZ
    CALL LFC43
    CALL EXPR_NOT_LAYER
    CALL LFABF
    LD A,L
    AND E
    LD L,A
    LD A,H
    AND D
    LD H,A
    CALL INTEGER_TO_STACK
    JR LF0DA

; NOT works on the integer conversion of the numeric stack value. The complemented pair is
; converted back to the BASIC numeric form; this is why NOT is a numeric/bitwise operator rather
; than a Boolean-only branch.

; -----------------------------------------------------------------------------
; NOT EXPRESSION STAGE
; -----------------------------------------------------------------------------
;
; Evaluates a NOT operand and complements its integer representation.
;
; The operand is converted from the BASIC stack to an integer pair, complemented, and placed back
; as a normalized numeric result. A non-integer or malformed operand follows the normal
; argument/type error path.
; -----------------------------------------------------------------------------
EXPR_NOT_LAYER:
    CP C2H
    JR NZ,EXPR_RELATION_LAYER
    CALL LFC43
    CALL EXPR_NOT_LAYER
    CALL STACK_NUMBER_TO_INTEGER
    LD A,L
    CPL
    LD L,A
    LD A,H
    CPL
    LD H,A
    JP INTEGER_TO_STACK

; Relation parsing deliberately has separate numeric and string branches. Both branches return
; flags that the common relation decoder can turn into a BASIC numeric truth value.

; -----------------------------------------------------------------------------
; RELATIONAL EXPRESSION STAGE
; -----------------------------------------------------------------------------
;
; Recognizes numeric and string relation operators and maps their comparison result to BASIC truth
; values.
;
; The stage first evaluates the additive operand, then dispatches to numeric or string comparison
; according to the values on the stack. It handles the six ordered/equality relations through one
; flag-to-result decoder.
; -----------------------------------------------------------------------------
EXPR_RELATION_LAYER:
    CP 02H
    JR C,EXPR_STRING_RELATION
    CALL EVAL_ADD_SUB
    CP 99H
    RET C
    CP 9FH
    RET NC
    PUSH AF
    CALL LFC43
    CALL EVAL_ADD_SUB
    CALL FP_COMPARE
    JR LF13A

; For a string relation, the right expression is evaluated as a descriptor before comparison. The
; string data remains length-tracked, so embedded zero bytes do not terminate the comparison.

; -----------------------------------------------------------------------------
; STRING RELATION PATH
; -----------------------------------------------------------------------------
;
; Evaluates and compares a string right operand when the relation operator is applied to strings.
;
; The string expression is evaluated before the comparison helper consumes the two descriptors.
; The result is converted to the same numeric truth convention used by numeric relations.
; -----------------------------------------------------------------------------
EXPR_STRING_RELATION:
    CALL EVAL_STRING_EXPRESSION
    CP 99H
    JP C,LF35D
    CP 9FH
    JP NC,LF35D
    PUSH AF
    CALL LFC43
    CALL EVAL_STRING_EXPRESSION
    CALL STRING_COMPARE

LF13A:
    POP HL
    LD A,H
    CALL RELATION_RESULT_DECODE
    JP INTEGER_TO_STACK

; The rotate/branch decoder maps the relation token and CY/S/Z flags to the true/false numeric
; result, then reclaims both operands through the BASIC-stack integer path.

; -----------------------------------------------------------------------------
; RELATION FLAG DECODER
; -----------------------------------------------------------------------------
;
; Turns comparison flags and the relation token into a Boolean numeric result.
;
; The compact rotate/branch sequence classifies equality, less-than, greater-than, and their
; negated forms. It then pushes one numeric result while discarding the compared operands.
; -----------------------------------------------------------------------------
RELATION_RESULT_DECODE:
    LD HL,FFFFH
    RRCA
    JR NC,LF149
    RET M

LF149:
    RRCA
    JR NC,LF14D
    RET Z

LF14D:
    RRCA
    JR NC,LF153
    JR Z,LF153
    RET P

LF153:
    INC HL
    RET

; -----------------------------------------------------------------------------
; ADDITION/SUBTRACTION EXPRESSION LAYER
; -----------------------------------------------------------------------------
;
; Recognizes + and - tokens and combines the two evaluated numeric operands.
;
; The layer first evaluates the higher-priority multiplicative term. A minus operation is
; represented by negating the second operand before calling the shared floating-point addition
; routine, so addition and subtraction share the same stack operation.
;
; Entry:
;   Token stream and first operand on the BASIC stack.
;
; Exit:
;   The combined numeric result replaces the two operands.
;
; Effects:
;   Advances the source pointer and consumes one stack operand.
;
; Destroys:
;   AF, BC, DE, HL, IY.
; -----------------------------------------------------------------------------
EVAL_ADD_SUB:
    CALL EXPR_ADD_SUB_OPERATOR
    CALL NZ,EVAL_MUL_DIV
    INC E
    CALL NZ,FP_NEGATE

LF15F:
    CALL EXPR_ADD_SUB_OPERATOR
    RET NZ
    INC E
    CALL Z,FP_ADD
    CALL NZ,FP_SUB
    JR LF15F

; The additive loop remembers whether the operator was plus or minus while evaluating the next
; multiplicative term. Minus is implemented by negating that term and entering the same FP_ADD
; machinery.

; -----------------------------------------------------------------------------
; ADD/SUBTRACT OPERATOR SCAN
; -----------------------------------------------------------------------------
;
; Fetches and classifies the next additive operator before the right term is evaluated.
;
; The helper distinguishes plus from minus, records the operation in E, and leaves the source
; pointer positioned for the multiplicative layer. Subtraction later reuses addition after sign
; inversion.
; -----------------------------------------------------------------------------
EXPR_ADD_SUB_OPERATOR:
    EXX
    LD A,B
    EXX
    LD E,FFH
    CP 98H
    JR Z,LF179
    CP A2H
    RET NZ
    INC E

LF179:
    CALL LFC43
    CALL EVAL_MUL_DIV
    XOR A
    RET

; -----------------------------------------------------------------------------
; MULTIPLICATION/DIVISION EXPRESSION LAYER
; -----------------------------------------------------------------------------
;
; Recognizes * and / tokens and invokes the corresponding floating-point operation.
;
; After higher-priority factors have been evaluated, this layer checks for multiplication or
; division and retains the operation token while the right operand is evaluated. The selected
; routine then normalizes the result and reports divide-by-zero or range errors through the normal
; BASIC error path.
;
; Entry:
;   Token stream and left operand on BASIC stack.
;
; Exit:
;   Product or quotient replaces the two operands.
;
; Effects:
;   Consumes one BASIC-stack value and advances the source pointer.
;
; Destroys:
;   AF, BC, DE, HL, IY.
; -----------------------------------------------------------------------------
EVAL_MUL_DIV:
    CALL EXPR_POWER_LAYER

LF184:
    EXX
    LD A,B
    EXX
    CP A8H
    JR Z,LF18E
    CP A1H
    RET NZ

LF18E:
    LD D,A
    CALL LFC43
    CALL EXPR_POWER_LAYER
    PUSH DE
    LD A,D
    CP A8H
    CALL Z,FP_MUL
    CALL NZ,FP_DIV
    POP DE
    JR LF184

; Exponentiation is nested inside the multiplicative layer. Its right operand is a primary term,
; giving the parser a distinct power stage instead of treating ^ like * or /.

; -----------------------------------------------------------------------------
; POWER EXPRESSION STAGE
; -----------------------------------------------------------------------------
;
; Evaluates exponentiation with a higher-priority primary term on the right.
;
; The stage preserves the left value while it evaluates a primary exponent, calls the power
; implementation, and loops for any following power token. Its placement gives exponentiation
; priority over multiplication and division.
; -----------------------------------------------------------------------------
EXPR_POWER_LAYER:
    PUSH DE
    CALL EVAL_PRIMARY

LF1A6:
    POP DE
    EXX
    LD A,B
    EXX
    CP 9FH
    RET NZ
    PUSH DE
    CALL LFC43
    CALL EVAL_PRIMARY
    CALL LEF17
    JR LF1A6

LF1B9:
    RST 08H
    DB 03H                                ; error code 03H: Argument missing

; -----------------------------------------------------------------------------
; PRIMARY EXPRESSION TERMS
; -----------------------------------------------------------------------------
;
; Dispatches numeric functions, parentheses, constants, variables, and user-defined functions.
;
; This is the highest-level numeric-term parser. It recognizes built-in function tokens such as
; ORD and ATN, evaluates parenthesized expressions, resolves variables and symbols, and invokes
; the DEF-function machinery when a user-defined function token is encountered. Invalid token
; forms branch to NOT UNDERSTOOD, ARGUMENT MISSING, or TYPE MISMATCH errors.
;
; Entry:
;   Next token in the BASIC source.
;
; Exit:
;   A primary numeric value is placed on the BASIC stack.
;
; Effects:
;   May call function evaluators and symbol-table lookup routines.
;
; Destroys:
;   AF, BC, DE, HL, IY, interpreter temporaries.
; -----------------------------------------------------------------------------
EVAL_PRIMARY:
    PUSH AF
    CALL LFC8E
    POP AF
    CP C0H
    JP Z,LEDA9
    CP B1H
    JP Z,LEAF1
    CP 02H
    JP C,LF35D
    JP Z,LFC43
    CP FDH
    JR NC,LF1B9
    CP 96H
    JR NZ,PRIMARY_SYMBOL_DISPATCH
    CALL LF0A4
    CP 95H
    JP Z,LFC43

; A primary token is either converted as a literal, resolved through the symbol chain, or
; dispatched to a built-in/user function. Terminators are rejected here so a missing expression
; becomes an argument error at the correct boundary.

; -----------------------------------------------------------------------------
; PRIMARY SYMBOL DISPATCH
; -----------------------------------------------------------------------------
;
; Separates literals, built-in terms, and symbol references at the primary-expression boundary.
;
; After token classification, this path enters numeric conversion for literals or symbol lookup
; for names. It also rejects statement terminators and invalid token classes before they can
; corrupt the BASIC stack.
; -----------------------------------------------------------------------------
PRIMARY_SYMBOL_DISPATCH:
    CP 03H
    JP NZ,LFD5A
    CALL STACK_TO_NUMERIC_REGS
    BIT 3,C
    JP NZ,DBADH
    BIT 2,C
    JP Z,LFA63

; Array and DEF references temporarily reuse global BASIC pointers. The saved current-line, CHAIN,
; TOP, and type values are the protection against nested function evaluation corrupting the
; enclosing expression.

; -----------------------------------------------------------------------------
; ARRAY/DEF SYMBOL PATH
; -----------------------------------------------------------------------------
;
; Builds the execution frame for an array element or user-defined function reference.
;
; The routine saves the current line, symbol-chain pointers, and expected type while it evaluates
; subscripts or DEF arguments. The saved context permits nested expressions to use the same global
; workspace and still return to the caller.
; -----------------------------------------------------------------------------
ARRAY_OR_DEF_SYMBOL:
    LD D,(IX+01H)
    PUSH DE
    PUSH HL
    SUB 96H
    SUB 01H
    SBC A,A
    PUSH AF
    LD A,C
    JR Z,DEF_ARGUMENT_FRAME
    EX AF,AF'
    CALL LFC43
    OR A
    JP M,LFD5A
    LD (IX+01H),A
    CALL NUMERIC_STRING_GATE
    LD A,95H
    CALL LFD54
    EX AF,AF'

; A DEF call records the caller's BASIC execution context before it evaluates formal arguments and
; the function body. The return path must restore the symbol-chain boundary as well as the source
; pointer.

; -----------------------------------------------------------------------------
; DEF ARGUMENT FRAME SETUP
; -----------------------------------------------------------------------------
;
; Stores the current BASIC execution context while a user-defined function body is evaluated.
;
; The frame records the caller line and statement pointers, symbol-chain boundary, and TOP value,
; then evaluates the formal/actual arguments. A type-bit check prevents a string function from
; entering the numeric return path.
; -----------------------------------------------------------------------------
DEF_ARGUMENT_FRAME:
    EX AF,AF'
    POP AF
    LD HL,(SYMBOL_TOP)
    EX (SP),HL
    PUSH HL
    LD HL,(SYMBOL_END)
    EX (SP),HL
    EXX
    PUSH HL
    LD HL,(CUR_LINE)
    PUSH HL
    PUSH BC
    EXX
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    LD (CUR_LINE),DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    EX DE,HL
    EXX
    LD C,A
    CALL LFC43
    SUB 96H
    SUB 01H
    SBC A,A
    XOR C
    JP NZ,LFD5A
    XOR C
    JR Z,RESTORE_DEF_CONTEXT
    CALL LFC43
    AND 81H
    CP 01H
    JP NZ,LFD5A
    EXX
    BIT 7,C
    CALL Z,SYMBOL_CREATE_VALUE
    LD A,C
    XOR (IX+01H)
    AND 02H
    JP NZ,LF35D
    PUSH DE
    EXX
    POP HL
    CALL LFB3B
    CALL LFC43
    LD A,95H
    CALL LFD54

; Restoring the saved context is part of the function ABI: leaving the temporary DEF symbol or
; changed TOP visible would make later variable lookup depend on call history.

; -----------------------------------------------------------------------------
; RESTORE DEF CONTEXT
; -----------------------------------------------------------------------------
;
; Restores the caller's line, symbol, and type state after a DEF function returns.
;
; The saved pointers are popped in the reverse order used by the function frame. Restoring TOP and
; CHAIN is essential because temporary function symbols must not remain visible to the enclosing
; expression.
; -----------------------------------------------------------------------------
RESTORE_DEF_CONTEXT:
    LD A,9AH
    CALL LFD54
    EX AF,AF'
    LD (IX+01H),A
    CALL NUMERIC_STRING_GATE
    POP BC
    POP HL
    LD (CUR_LINE),HL
    POP HL
    EXX
    POP HL
    LD (SYMBOL_END),HL
    POP HL
    LD (SYMBOL_TOP),HL
    POP AF
    LD (VALUE_TYPE),A
    RET

; -----------------------------------------------------------------------------
; NUMERIC/STRING EVALUATOR GATE
; -----------------------------------------------------------------------------
;
; Selects the evaluator path from the expected operand type stored in the interpreter state.
;
; The gate branches to numeric expression evaluation for numeric contexts and to the string
; evaluator for string contexts. This shared boundary is why assignment and function routines can
; request a typed expression without duplicating token walking.
; -----------------------------------------------------------------------------
NUMERIC_STRING_GATE:
    BIT 1,(IX+01H)
    JP NZ,EVAL_NUMERIC_EXPRESSION

; -----------------------------------------------------------------------------
; STRING EXPRESSION EVALUATOR
; -----------------------------------------------------------------------------
;
; Evaluates concatenation and string terms onto the BASIC stack.
;
; The string evaluator mirrors the numeric precedence structure but operates on string descriptors
; and data. It resolves string variables, literals, and concatenation, moving descriptors and
; payloads while preserving the stack's length-prefixed representation.
;
; Entry:
;   HL' points into tokenized BASIC source.
;
; Exit:
;   A string value is placed on the BASIC stack.
;
; Effects:
;   Allocates stack space and may copy string bytes.
;
; Destroys:
;   AF, BC, DE, HL, IY.
; -----------------------------------------------------------------------------
EVAL_STRING_EXPRESSION:
    EXX
    LD A,B
    EXX
    CALL STRING_TERM_LAYER

; String concatenation is an in-place stack compaction operation. Length is checked before bytes
; are moved; the new descriptor is committed only after the combined payload fits.

; -----------------------------------------------------------------------------
; STRING CONCATENATION LOOP
; -----------------------------------------------------------------------------
;
; Combines successive string terms while retaining a single length-tracked stack value.
;
; The loop locates both string payloads, adds their lengths, moves bytes to close the old stack
; gap, and writes a new descriptor. Overflow is detected before the result is committed, so a
; failed concatenation does not leave a partially valid string.
; -----------------------------------------------------------------------------
STRING_CONCAT_LOOP:
    EXX
    LD A,B
    EXX
    CP 97H
    RET NZ
    CALL LFC43
    CALL STRING_TERM_LAYER
    PUSH IY
    POP HL
    INC HL
    LD E,L
    LD D,H
    LD C,(HL)
    LD B,00H
    ADD HL,BC
    INC HL
    PUSH HL
    POP IY
    INC HL
    LD C,(HL)
    CALL LFC8E
    PUSH HL
    ADD HL,BC
    LD A,C
    OR A
    LD A,(DE)
    JR Z,LF2C2
    LDDR

LF2C2:
    POP HL
    ADD A,(HL)
    LD (DE),A
    JP C,LF912
    CP FFH
    JP Z,LF912
    LD C,(HL)
    LD E,L
    LD D,H
    DEC DE
    DEC DE
    ADD HL,BC
    EX DE,HL
    LD C,A
    INC BC
    LDDR
    EX DE,HL
    LD (HL),01H
    PUSH HL
    POP IY
    JR STRING_CONCAT_LOOP

; The string term layer keeps descriptor length separate from source-token length. This
; distinction is essential when a string contains encoded characters or an embedded byte that
; resembles a BASIC token.

; -----------------------------------------------------------------------------
; STRING TERM LAYER
; -----------------------------------------------------------------------------
;
; Evaluates a string term and prepares it for concatenation or comparison.
;
; This stage handles the string primary path and returns with IY at a length-prefixed value. The
; caller can then test the concatenation token without confusing string length bytes with source
; tokens.
; -----------------------------------------------------------------------------
STRING_TERM_LAYER:
    CALL STRING_PRIMARY_DISPATCH

LF2E3:
    EXX
    LD A,B
    EXX
    CP 96H
    RET NZ
    CALL LFD27
    PUSH IY
    POP HL
    INC HL
    LD A,(HL)
    LD C,A
    LD B,00H
    CP D
    JR C,STRING_COPY_TRUNCATE
    LD A,D

; -----------------------------------------------------------------------------
; STRING LENGTH CONTROL
; -----------------------------------------------------------------------------
;
; Copies only the permitted portion of a string operand when a destination length limits it.
;
; The routine compares source and requested lengths, selects the smaller count, and moves the
; selected bytes into the new stack location. It preserves the descriptor convention used by
; subsequent string functions.
; -----------------------------------------------------------------------------
STRING_COPY_TRUNCATE:
    LD D,A
    OR A
    JR Z,LF309
    LD A,C
    CP E
    JR C,LF309
    INC E
    DEC E
    JR NZ,LF305
    INC E

; When a string result is shortened, the copy path selects the permitted count first and then
; moves bytes backward into the new stack slot. The source descriptor is not modified until the
; new value is complete.

LF305:
    LD A,D
    SUB E
    JR NC,LF30D

LF309:
    ADD HL,BC
    XOR A
    JR LF318

LF30D:
    INC A
    PUSH HL
    ADD HL,BC
    EX DE,HL
    LD C,H
    POP HL
    ADD HL,BC
    LD C,A
    LDDR
    EX DE,HL

LF318:
    LD (HL),A
    DEC HL
    LD (HL),01H
    PUSH HL
    POP IY
    JR LF2E3
    CALL LFAC4
    LD D,00H
    INC H
    RET Z
    DEC D
    DEC H
    RET NZ
    LD D,L
    RET

; String primary dispatch rejects numeric symbol types before allocation. The same check is used
; by string functions and assignment, producing Type mismatch instead of interpreting numeric
; bytes as characters.

; -----------------------------------------------------------------------------
; STRING PRIMARY DISPATCH
; -----------------------------------------------------------------------------
;
; Dispatches string literals, string functions, and string symbol references.
;
; The dispatch validates the token class, recognizes the string function prefix, then asks symbol
; lookup for the name/type descriptor. Numeric symbols and malformed delimiters are rejected
; before string storage is allocated.
; -----------------------------------------------------------------------------
STRING_PRIMARY_DISPATCH:
    CALL LFC8E
    EXX
    PUSH BC
    EXX
    POP BC
    LD A,B
    CP 01H
    JP C,LFC43
    JR Z,STRING_SYMBOL_DISPATCH
    CP 04H
    JR C,LF35D
    CP FDH
    JP C,LFD5A
    RST 08H
    DB 03H                                ; error code 03H: Argument missing

; -----------------------------------------------------------------------------
; STRING SYMBOL DISPATCH
; -----------------------------------------------------------------------------
;
; Resolves a string symbol and selects scalar, array, or function data handling.
;
; Type bits decide whether the descriptor is copied directly, indexed through array metadata, or
; passed to a function path. The helper is shared by string assignment and expression evaluation.
; -----------------------------------------------------------------------------
STRING_SYMBOL_DISPATCH:
    LD A,C
    CP C5H
    JP Z,LEC89
    CALL STACK_TO_NUMERIC_REGS
    BIT 3,C
    JP NZ,DBADH
    BIT 2,C
    JP NZ,ARRAY_OR_DEF_SYMBOL
    JP LFA7C

LF35D:
    RST 08H
    DB 0EH                                ; error code 0EH: Type mismatch

; -----------------------------------------------------------------------------
; CREATE SYMBOL TABLE ENTRY
; -----------------------------------------------------------------------------
;
; Creates or extends a linked BASIC symbol entry and records its type and value address.
;
; The symbol routines maintain the linked table used for numeric and string variables. Names are
; copied into the symbol area, a type byte records the value kind, and the data pointer identifies
; the storage or function body. Memory exhaustion and duplicate declarations are reported through
; BASIC errors.
;
; Entry:
;   Parsed symbol name and type in interpreter work variables.
;
; Exit:
;   HL/DE identify the new symbol entry and its data area.
;
; Effects:
;   Allocates and writes symbol-table memory.
;
; Destroys:
;   AF, BC, DE, HL, IY.
; -----------------------------------------------------------------------------
SYMBOL_CREATE:
    CALL LFC8E
    LD DE,(SYMBOL_END)
    LD HL,(SYMBOL_TOP)
    LD (SYMBOL_END),HL
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    LD C,00H
    PUSH HL
    LD DE,(1728H)

; Name creation copies only the BASIC identifier alphabet and counts the name separately from its
; type suffix. TOP is updated after the length byte is written, so an interrupted creation cannot
; expose a half-name through CHAIN.

; -----------------------------------------------------------------------------
; SYMBOL NAME COPY LOOP
; -----------------------------------------------------------------------------
;
; Copies an input symbol name into the linked symbol-table entry while counting valid characters.
;
; Characters are accepted from the BASIC name alphabet, with dollar and period handled as
; type/name delimiters. The count is written before finalization so lookup can distinguish a
; complete name from a prefix.
; -----------------------------------------------------------------------------
SYMBOL_NAME_COPY:
    LD A,(DE)
    CP FDH
    JR NC,LF3A3
    CP 20H
    JR C,LF39C
    CP 24H
    JR Z,LF39D
    CP 2EH
    JR Z,LF39C
    CP 30H
    JR C,LF3A3
    CP 3AH
    JR C,LF39C
    CP 3FH
    JR C,LF3A3
    CP 60H
    JR C,LF39C
    CP A9H
    JR C,LF3A3

LF39C:
    SCF

LF39D:
    INC DE
    INC C
    INC HL
    LD (HL),A
    JR C,SYMBOL_NAME_COPY

LF3A3:
    LD A,(HL)
    INC HL
    LD (SYMBOL_TOP),HL
    EX (SP),HL
    LD (HL),C
    POP HL
    RET

; -----------------------------------------------------------------------------
; INITIALIZE SYMBOL DATA
; -----------------------------------------------------------------------------
;
; Initializes the fixed numeric data area for a newly created scalar or numeric array entry.
;
; The helper clears the initial bytes and advances TOP according to the selected symbol kind.
; Clearing is deliberate: an undefined scalar reads as numeric zero rather than stale RAM.
; -----------------------------------------------------------------------------
INIT_NUMERIC_SYMBOL:
    CALL LFC8E
    LD HL,(SYMBOL_TOP)
    LD B,06H
    XOR A

LF3B5:
    LD (HL),A
    INC HL
    DJNZ LF3B5
    JR LF3CE

; -----------------------------------------------------------------------------
; INITIALIZE STRING/ARRAY DATA
; -----------------------------------------------------------------------------
;
; Reserves and initializes variable-sized string or array symbol storage.
;
; The path records the element length/dimension information, advances TOP over the requested data,
; and leaves the symbol descriptor pointing at the allocated region. Memory exhaustion is reported
; before the table link is made live.
; -----------------------------------------------------------------------------
INIT_STRING_OR_ARRAY_SYMBOL:
    BIT 1,(IX+01H)
    JR NZ,INIT_NUMERIC_SYMBOL

LF3C1:
    CALL LFC8E
    LD HL,(SYMBOL_TOP)
    LD (HL),C
    INC HL
    LD B,00H
    LD (HL),B
    ADD HL,BC
    INC HL

LF3CE:
    LD (SYMBOL_TOP),HL
    RET

; -----------------------------------------------------------------------------
; LOOK UP BASIC SYMBOL
; -----------------------------------------------------------------------------
;
; Searches the linked symbol table for a name and returns its descriptor.
;
; The lookup compares the source name against each linked entry, handling quoted and tokenized
; characters and stopping at name boundaries. A matching entry returns its type and data address;
; a missing symbol follows the variable-creation or undefined-name error path selected by the
; caller.
;
; Entry:
;   Name pointer and symbol-table chain.
;
; Exit:
;   HL = symbol entry/data address; C = type byte or status.
;
; Effects:
;   Reads symbol-table memory.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
SYMBOL_LOOKUP:
    LD (1728H),HL
    LD HL,SYMBOL_END+1
    DB 3EH                                ; = LD A,E1H when run from here, hiding the entry below

; Symbol lookup follows little-endian links from CHAIN toward older entries. A zero link is the
; defined end condition; it is not an instruction address or a valid empty symbol.

; -----------------------------------------------------------------------------
; ADVANCE SYMBOL CHAIN
; -----------------------------------------------------------------------------
;
; Loads the next linked symbol entry while scanning for a name match.
;
; The link is read little-endian from the preceding entry. A zero link ends the search; otherwise
; the candidate name length and bytes become the comparison window.
; -----------------------------------------------------------------------------
SYMBOL_CHAIN_NEXT:
    POP HL
    LD D,(HL)
    DEC HL
    LD E,(HL)
    LD A,E
    OR D
    LD HL,(1728H)
    RET Z
    EX DE,HL
    INC HL
    PUSH HL
    INC HL
    LD B,(HL)

; The name comparison checks the delimiter after a matching prefix. This prevents a short variable
; name from aliasing a longer one and handles $/token terminators as part of the BASIC name
; grammar.

; -----------------------------------------------------------------------------
; COMPARE SYMBOL NAME
; -----------------------------------------------------------------------------
;
; Compares input name bytes with a candidate entry and enforces the name boundary.
;
; A full byte match is not enough: the following source byte must terminate the name or carry its
; type delimiter. This avoids treating A as a match for ARRAY and handles tokenized keyword bytes
; safely.
; -----------------------------------------------------------------------------
SYMBOL_NAME_COMPARE:
    LD A,(DE)
    INC DE
    INC HL
    CP (HL)
    JR NZ,SYMBOL_CHAIN_NEXT
    DJNZ SYMBOL_NAME_COMPARE
    CP 24H
    JR Z,SYMBOL_MATCH_RETURN
    LD A,(DE)
    CP 20H
    JR Z,SYMBOL_MATCH_RETURN
    CP FDH
    JR NC,SYMBOL_MATCH_RETURN
    CP A9H
    JR NC,SYMBOL_CHAIN_NEXT
    RLA
    JR NC,SYMBOL_CHAIN_NEXT

; A matched entry has its construction marker cleared before the type byte is returned. Callers
; therefore see a stable scalar/array/string/function classification, not the temporary marker
; used while allocating the entry.

; -----------------------------------------------------------------------------
; RETURN SYMBOL MATCH
; -----------------------------------------------------------------------------
;
; Returns the matched type byte and data address to the expression or statement caller.
;
; The high-bit marker used during table construction is cleared before the type is exposed. The
; returned pointer is advanced past the descriptor header so callers can read scalar data or array
; metadata directly.
; -----------------------------------------------------------------------------
SYMBOL_MATCH_RETURN:
    POP AF
    INC HL
    RES 7,(HL)
    SCF
    JR LF428

; Symbol creation writes the descriptor and initializes its data area before publishing the link.
; This ordering is what lets an out-of-memory or duplicate-declaration error leave the live chain
; consistent.

; -----------------------------------------------------------------------------
; CREATE SYMBOL VALUE AREA
; -----------------------------------------------------------------------------
;
; Completes a newly created symbol by choosing its type and allocating its initial value area.
;
; The helper distinguishes string, numeric, and function descriptors, writes the final type byte,
; and invokes the appropriate initializer. The symbol is linked only after the name and descriptor
; are coherent.
; -----------------------------------------------------------------------------
SYMBOL_CREATE_VALUE:
    CALL SYMBOL_CREATE
    SUB 24H
    JR Z,LF414
    LD A,02H

LF414:
    LD (HL),A
    PUSH HL
    INC HL
    LD (SYMBOL_TOP),HL
    JR NZ,LF422
    LD C,12H
    CALL LF3C1
    XOR A

LF422:
    CALL NZ,INIT_NUMERIC_SYMBOL
    POP HL
    SET 7,(HL)

LF428:
    LD C,(HL)
    INC HL
    EX DE,HL
    RET

LF42C:
    RST 08H
    DB 05H                                ; error code 05H: Bad subscript

; BASIC numeric stack elements are nine bytes: identifier, overflow byte, exponent/sign byte, and
; seven packed BCD mantissa bytes.

; -----------------------------------------------------------------------------
; LOAD NUMBERS FROM BASIC STACK
; -----------------------------------------------------------------------------
;
; Converts the top stack numbers into the CPU register pairs used by arithmetic routines.
;
; The helper reads the second and first nine-byte numeric elements, invalidates their old stack
; positions as appropriate, and returns the operands in DE and HL. It is the common bridge between
; the compact BASIC-stack format and arithmetic code.
;
; Entry:
;   IY points at the BASIC-stack boundary; two numeric elements are present.
;
; Exit:
;   DE and HL contain the two numeric operands.
;
; Effects:
;   Advances or temporarily relocates the BASIC stack pointer.
;
; Destroys:
;   AF, BC, IY; DE and HL contain converted values.
; -----------------------------------------------------------------------------
STACK_TO_NUMERIC_REGS:
    EXX

; Stack operand walking is type-aware. Arithmetic callers cannot simply subtract nine from IY when
; a string or control frame is interposed; the helper follows the stored element layout.

; -----------------------------------------------------------------------------
; STACK OPERAND WALKER
; -----------------------------------------------------------------------------
;
; Walks typed BASIC-stack elements to locate operands for arithmetic and conversion helpers.
;
; The walker follows the element type and length rather than assuming every value is nine bytes.
; It is the bridge used when a numeric expression is embedded beside strings or control frames.
; -----------------------------------------------------------------------------
STACK_OPERAND_WALK:
    PUSH DE
    BIT 0,C
    LD A,C
    EXX
    POP HL
    LD C,A
    JP Z,LFC43
    LD A,96H
    CALL LFD51
    XOR A
    LD E,A
    LD D,A
    LD A,(HL)
    INC HL
    EX DE,HL
    PUSH BC

; The operand scan relocates selected values into the register/workspace arrangement expected by
; arithmetic code, then returns IY aligned at the next live stack element. This is the hidden
; contract behind many RST18 operations.

; -----------------------------------------------------------------------------
; SCAN STACK ELEMENTS
; -----------------------------------------------------------------------------
;
; Scans and relocates the requested number of stack elements before a conversion or arithmetic
; call.
;
; The loop skips elements by their type-dependent size, copies the selected value into the
; expected workspace, and leaves IY aligned on the next live element. This prevents temporary
; strings from being interpreted as floating-point operands.
; -----------------------------------------------------------------------------
STACK_ELEMENT_SCAN:
    PUSH HL
    PUSH AF
    LD A,A4H
    CALL NZ,LFD54
    CALL LFAC4
    EX DE,HL
    LD C,(HL)
    INC HL
    LD B,(HL)
    INC HL
    EX DE,HL
    OR A
    SBC HL,BC
    JR NC,LF42C
    ADD HL,BC
    POP AF
    PUSH AF
    PUSH DE

LF45E:
    DEC A
    JR Z,LF46F
    EX DE,HL
    LD C,(HL)
    INC HL
    LD B,(HL)
    INC HL
    PUSH HL
    PUSH AF
    CALL LFCB3
    POP AF
    POP DE
    JR LF45E

LF46F:
    POP DE
    POP AF
    POP BC
    ADD HL,BC
    DEC A
    JR NZ,STACK_ELEMENT_SCAN
    LD A,95H
    CALL LFD54
    EX DE,HL
    POP BC
    BIT 1,C
    LD BC,0006H
    JR NZ,LF487
    LD C,(HL)
    INC BC
    INC BC

LF487:
    PUSH HL
    CALL LFCB3
    POP DE
    ADD HL,DE
    RET

; -----------------------------------------------------------------------------
; FLOATING-POINT SUBTRACTION
; -----------------------------------------------------------------------------
;
; Subtracts the second nine-byte BASIC number from the first.
;
; Subtraction is implemented by negating the second operand and entering the shared addition
; algorithm at F493H. The operands remain in the packed BCD representation while exponents and
; signs are aligned, mantissas are added, and the result is normalized.
;
; Entry:
;   Two nine-byte BASIC numeric elements at the top of the stack.
;
; Exit:
;   The first operand contains x-y; one stack element is consumed and IY advances by nine.
;
; Effects:
;   Rewrites the first numeric element and may signal overflow.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
; -----------------------------------------------------------------------------
FP_SUB:
    PUSH AF
    CALL FP_NEGATE
    POP AF

; -----------------------------------------------------------------------------
; FLOATING-POINT ADDITION
; -----------------------------------------------------------------------------
;
; Adds two packed-BCD BASIC numbers with exponent alignment and normalization.
;
; The routine compares exponents, shifts the smaller mantissa, handles opposite signs as
; subtraction, and adds seven packed mantissa bytes with BCD correction. Carry and leading-zero
; handling adjust the exponent; the normalized nine-byte result is left in the first operand slot.
;
; Entry:
;   Two nine-byte BASIC numeric elements at the top of the stack.
;
; Exit:
;   The first operand contains x+y; one stack element is consumed.
;
; Effects:
;   Rewrites stack data and can raise Overflow.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
;
; Note:
;   Numbers use an identifier byte, overflow byte, exponent/sign byte, and seven packed BCD
;   mantissa bytes.
; -----------------------------------------------------------------------------
FP_ADD:
    PUSH AF
    PUSH IY
    LD DE,0009H
    ADD IY,DE
    EX (SP),IY
    LD A,(IY+08H)
    AND 7FH
    LD C,A
    LD A,(IY+11H)
    AND 7FH
    SUB C
    JP P,FP_ALIGN_OR_SUBTRACT
    EX (SP),IY
    PUSH AF
    NEG
    CALL FP_DECIMAL_SHIFT
    POP BC
    EX (SP),IY
    LD A,(IY+11H)
    SUB B
    LD (IY+11H),A
    XOR A

; Floating addition first aligns the smaller exponent and then decides add versus subtract from
; sign bits. Opposite-sign cancellation can create leading zeroes, so the normalizer must run even
; when no arithmetic carry occurred.

; -----------------------------------------------------------------------------
; ALIGN FLOATING OPERANDS
; -----------------------------------------------------------------------------
;
; Aligns exponents and selects addition or magnitude subtraction according to operand signs.
;
; The smaller exponent is shifted toward the larger, while the sign XOR determines whether packed
; BCD addition or subtraction is required. Opposite-sign results are corrected for leading zeroes
; before the shared normalizer runs.
; -----------------------------------------------------------------------------
FP_ALIGN_OR_SUBTRACT:
    CALL NZ,FP_DECIMAL_SHIFT
    LD A,(IY+08H)
    XOR (IY+11H)
    RLCA
    CALL NC,FP_BCD_ADD_MANTISSA
    EX (SP),IY
    JR NC,LF4EE
    CALL LF71C
    EX (SP),IY
    CALL LF71C
    CALL FP_BCD_ADD_MANTISSA
    EX (SP),IY
    LD A,(IY+07H)
    OR A
    JR Z,LF4EE
    CALL LF70C
    LD A,80H
    XOR (IY+08H)
    LD (IY+08H),A

LF4EE:
    POP AF

LF4EF:
    CALL FP_NORMALIZE
    POP AF
    RET

; The mantissa loop is decimal: ADC followed by DAA propagates carries in packed BCD. Treating
; these bytes as binary integers will give plausible-looking but incorrect emulator results.

; -----------------------------------------------------------------------------
; ADD PACKED BCD MANTISSAS
; -----------------------------------------------------------------------------
;
; Adds seven packed BCD mantissa bytes with decimal carry propagation.
;
; The byte loop uses ADC followed by DAA, preserving decimal digits rather than binary nibbles. A
; final carry changes the exponent and is subsequently folded into the normalized result.
; -----------------------------------------------------------------------------
FP_BCD_ADD_MANTISSA:
    PUSH AF
    PUSH IY
    POP DE
    LD HL,0009H
    ADD HL,DE
    LD B,07H

LF4FE:
    INC HL
    INC DE
    LD A,(DE)
    ADC A,(HL)
    DAA
    LD (HL),A
    DJNZ LF4FE
    POP AF
    RET

LF508:
    LD DE,0009H
    ADD IY,DE
    CALL FP_CLEAR_NUMERIC
    POP AF
    RET

; Multiplication expands mantissas to decimal digits and accumulates sixteen working digits before
; repacking.

; -----------------------------------------------------------------------------
; FLOATING-POINT MULTIPLICATION
; -----------------------------------------------------------------------------
;
; Multiplies two BASIC floating-point values using sixteen decimal working digits.
;
; Zero operands are handled first. Otherwise signs and exponents are combined, each seven-byte BCD
; mantissa is expanded into individual digits, and a table-driven long multiplication accumulates
; sixteen decimal digits. The result is packed, rounded/normalized, and placed in the first
; operand slot.
;
; Entry:
;   Two nine-byte BASIC numeric elements on the stack.
;
; Exit:
;   The first operand contains the product; one operand is consumed.
;
; Effects:
;   Uses temporary stack/memory working storage and can report overflow.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
; -----------------------------------------------------------------------------
FP_MUL:
    PUSH AF
    LD A,(IY+0FH)
    OR A
    JR Z,LF508
    LD A,(IY+06H)
    OR A
    JR Z,LF508
    LD A,(IY+11H)
    LD L,(IY+08H)
    XOR L
    LD H,A
    XOR L
    AND 7FH
    LD L,A
    LD A,(IY+08H)
    AND 7FH
    ADD A,L
    SUB 40H
    JR C,LF508
    JP M,LF912
    INC A
    JP M,LF912
    DEC A
    LD L,A
    PUSH HL
    EXX
    EX (SP),HL
    PUSH DE
    PUSH BC
    PUSH HL
    PUSH IY
    POP HL
    LD E,L
    LD D,H
    LD BC,0010H
    ADD HL,BC
    CALL LF5E8
    PUSH DE
    LD (HL),A
    DEC HL
    LD (HL),A
    DEC HL
    CALL LF5E8
    LD HL,000AH
    ADD HL,DE
    LD C,L
    LD B,H
    LD DE,0013H
    ADD HL,DE
    POP DE
    INC DE
    PUSH HL
    OR 02H
    EX AF,AF'
    EXX
    LD C,0CH
    EXX

; Multiplication keeps guard digits beyond the seven stored mantissa bytes. Those digits allow the
; final pack/normalize stage to round at the correct decimal boundary instead of truncating each
; partial product.

; -----------------------------------------------------------------------------
; MULTIPLICATION DIGIT LOOP
; -----------------------------------------------------------------------------
;
; Accumulates one decimal digit product into the sixteen-digit multiplication workspace.
;
; Each source BCD digit selects a small product contribution, which is added into the working
; digits with decimal correction. The outer loop advances across both operands and leaves guard
; digits available for rounding.
; -----------------------------------------------------------------------------
FP_MUL_DIGIT_PRODUCT:
    POP HL
    PUSH HL
    PUSH BC
    LD A,(DE)
    EXX
    EX AF,AF'
    PUSH AF
    EX AF,AF'
    ADD A,A
    JR Z,FP_MUL_PACK_RESULT
    LD D,A
    ADD A,A
    ADD A,A
    ADD A,D
    LD D,A
    LD H,C0H
    LD E,00H
    LD B,10H
    EX AF,AF'

LF583:
    JP P,FP_MUL_TABLE_LOOKUP
    INC E
    DEC E
    JR Z,FP_MUL_PACK_RESULT

LF58A:
    EX AF,AF'
    XOR A
    JR LF59B

; The product inner loop uses a ROM lookup for a selected BCD digit. Zero digits bypass
; accumulation, which is both an optimization and a useful clue when distinguishing the table data
; from executable code.

; -----------------------------------------------------------------------------
; MULTIPLICATION TABLE LOOKUP
; -----------------------------------------------------------------------------
;
; Maps a packed BCD nibble to the product contribution used by the long-multiplication loop.
;
; The nibble is converted to an address in the multiplication table; unused/zero digits take a
; short path. The table-driven form keeps the inner decimal accumulation small enough for ROM.
; -----------------------------------------------------------------------------
FP_MUL_TABLE_LOOKUP:
    CP 0CH
    JR NC,LF58A
    EX AF,AF'
    EXX
    LD A,(BC)
    EXX
    ADD A,D
    ADD A,03H
    LD L,A
    LD A,(HL)

LF59B:
    ADD A,E
    DAA
    EXX
    ADD A,(HL)
    DAA
    LD (HL),00H
    RLD
    INC HL
    INC BC
    EXX
    RLCA
    RLCA
    RLCA
    RLCA
    LD E,A
    EX AF,AF'
    DEC A
    DJNZ LF583

; The product is packed only after all digit rows have been accumulated. Sign/exponent restoration
; occurs after packing, followed by the same FP_NORMALIZE path used by addition and division.

; -----------------------------------------------------------------------------
; PACK MULTIPLICATION RESULT
; -----------------------------------------------------------------------------
;
; Packs the accumulated decimal product back into the nine-byte stack format.
;
; The sixteen working digits are shifted into seven packed bytes, the sign and exponent are
; restored, and the result is placed at the surviving operand slot. Normalization then removes any
; leading zero or carry displacement.
; -----------------------------------------------------------------------------
FP_MUL_PACK_RESULT:
    POP AF
    INC A
    EX AF,AF'
    DEC C
    EXX
    POP BC
    DEC BC
    INC DE
    JR NZ,FP_MUL_DIGIT_PRODUCT
    POP HL
    LD BC,000FH
    ADD HL,BC
    LD E,L
    LD D,H
    CALL LF5DA
    PUSH HL
    POP IY
    POP HL
    LD A,H
    AND 80H
    OR L
    LD (IY+08H),A
    LD (IY+00H),09H

LF5D3:
    POP BC
    POP DE
    POP HL
    EXX
    JP LF4EF

LF5DA:
    LD B,07H

LF5DC:
    LD A,(DE)
    DEC DE
    RLD
    LD A,(DE)
    DEC DE
    RLD
    DEC HL
    DJNZ LF5DC
    RET

LF5E8:
    LD B,07H

LF5EA:
    XOR A
    RLD
    LD (DE),A
    DEC DE
    XOR A
    RLD
    LD (DE),A
    DEC DE
    DEC HL
    DJNZ LF5EA
    XOR A
    RET

LF5F9:
    RST 08H
    DB 0BH                                ; error code 0BH: Cannot divide by 0

; Division generates up to twelve quotient digits by repeated shifted subtraction; zero divisor
; raises error 0BH.

; -----------------------------------------------------------------------------
; FLOATING-POINT DIVISION
; -----------------------------------------------------------------------------
;
; Divides two BASIC floating-point values by repeated BCD subtraction and quotient-digit
; generation.
;
; A zero divisor raises Cannot divide by 0; a zero dividend returns zero. The routine aligns
; exponents and signs, repeatedly subtracts the divisor from a shifted remainder to generate
; twelve quotient digits, packs the result, and normalizes it. The compact implementation uses the
; BASIC stack and scratch area as a multi-precision decimal workspace.
;
; Entry:
;   Two nine-byte BASIC numeric elements on the stack.
;
; Exit:
;   The first operand contains x/y; one operand is consumed.
;
; Effects:
;   Uses scratch memory and can raise divide-by-zero or overflow errors.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
; -----------------------------------------------------------------------------
FP_DIV:
    PUSH AF
    LD A,(IY+06H)
    OR A
    JR Z,LF5F9
    LD A,(IY+0FH)
    OR A
    JP Z,LFA1A
    EXX
    PUSH HL
    PUSH DE
    PUSH BC
    PUSH IY
    LD DE,0009H
    ADD IY,DE
    LD L,(IY-01H)
    LD H,(IY+08H)
    EX (SP),HL
    LD (171CH),HL
    PUSH HL
    EX (SP),IY
    CALL LF9F1
    LD B,04H

; Division prepares a shifted decimal remainder rather than using the CPU divide instruction. The
; scratch layout is multi-precision state and may overlap reclaimed BASIC-stack bytes while the
; operation is active.

; -----------------------------------------------------------------------------
; DIVISION REMAINDER SETUP
; -----------------------------------------------------------------------------
;
; Aligns the divisor and remainder before generating quotient digits.
;
; The routine copies the divisor into the scratch layout, adjusts the exponent difference, and
; prepares the shifted remainder. The quotient loop can then compare/subtract fixed seven-byte BCD
; fields.
; -----------------------------------------------------------------------------
FP_DIV_SHIFTED_REMAINDER:
    LD HL,(171CH)
    EXX
    LD BC,FFF7H
    LD HL,(171CH)
    LD E,L
    LD D,H
    OR A
    SBC HL,BC
    EX DE,HL
    ADD HL,BC
    ADD HL,BC

; Each quotient digit is obtained by trial subtraction of a seven-byte BCD divisor. A failed trial
; restores the remainder; a successful trial advances the quotient digit before the next decimal
; shift.

; -----------------------------------------------------------------------------
; DIVISION BCD SUBTRACTION LOOP
; -----------------------------------------------------------------------------
;
; Subtracts the aligned divisor from the decimal remainder and records one quotient digit.
;
; Seven BCD bytes are compared/subtracted with decimal borrow handling. A nonnegative remainder
; increments the quotient digit; a negative trial restores the prior remainder and advances the
; shift instead.
; -----------------------------------------------------------------------------
FP_DIV_BCD_SUBTRACT:
    LD B,07H
    OR A

LF63B:
    INC DE
    LD A,(DE)
    EXX
    INC HL
    SBC A,(HL)
    EXX
    DAA
    INC HL
    LD (HL),A
    DJNZ LF63B
    AND F0H
    JR NZ,FP_DIV_FINALIZE
    EXX
    CALL FP_GET_BCD_NIBBLE
    INC A
    CALL FP_SET_BCD_NIBBLE
    LD HL,(171CH)
    EXX
    LD BC,0007H
    LDDR
    JR FP_DIV_BCD_SUBTRACT

; Division finalization combines the generated quotient, exponent difference, and sign XOR only
; after the remainder loop ends. This ordering explains why a zero dividend has a short path while
; a zero divisor raises error 0BH immediately.

; -----------------------------------------------------------------------------
; FINALIZE QUOTIENT
; -----------------------------------------------------------------------------
;
; Packs generated quotient digits, applies signs, and returns the division result to the stack.
;
; After the fixed number of quotient positions, the scratch remainder is discarded, the exponent
; difference is biased, and the sign XOR is written into the result. A final normalizer handles
; carry or leading-zero displacement.
; -----------------------------------------------------------------------------
FP_DIV_FINALIZE:
    EX (SP),IY
    CALL FP_SHIFT_RIGHT_BCD
    EX (SP),IY
    EXX
    INC B
    LD A,B
    CP 10H
    JR C,FP_DIV_SHIFTED_REMAINDER
    EXX
    LD HL,(171CH)
    DEC HL
    DEC HL
    LD BC,0008H
    LDDR
    POP IY
    POP HL
    LD A,H
    XOR L
    AND 80H
    LD C,A
    RES 7,H
    RES 7,L
    LD A,H
    SUB L
    ADD A,40H
    XOR C
    LD (IY+08H),A
    XOR C

LF68B:
    JP P,LF5D3
    CALL FP_CLEAR_NUMERIC
    JR LF68B

; Numeric comparison returns CY/S/Z like subtraction: less-than sets CY and S, equality sets Z.

; -----------------------------------------------------------------------------
; COMPARE BASIC NUMBERS
; -----------------------------------------------------------------------------
;
; Compares two packed floating-point values and returns Z/S/C flags without destroying their bytes
; immediately.
;
; Signs are compared first, then exponents and finally fourteen BCD mantissa digits. For x<y the
; routine returns carry and sign set; equality sets Z; x>y clears all three. The expression
; evaluator uses these flags to implement the six relation tokens.
;
; Entry:
;   Two numeric stack elements.
;
; Exit:
;   CY, S, and Z describe x versus y.
;
; Effects:
;   Adjusts IY to the first operand for the next stack operation.
;
; Destroys:
;   AF, DE, HL; stack values remain readable until overwritten.
; -----------------------------------------------------------------------------
FP_COMPARE:
    PUSH IY
    POP HL
    LD BC,0009H
    ADD HL,BC
    LD E,L
    LD D,H
    DEC DE
    ADD HL,BC
    PUSH HL
    POP IY
    DEC HL
    LD A,(HL)
    AND 80H
    RRCA
    LD C,A
    LD A,(DE)
    AND 80H
    RRCA
    SUB C
    RET NZ
    ADD A,C
    JR Z,FP_COMPARE_MAGNITUDE
    EX DE,HL

; Numeric comparison ignores the sign bit while comparing exponents and mantissas. Sign ordering
; is handled first, preventing a negative number with a large magnitude from being mistaken for a
; positive one.

; -----------------------------------------------------------------------------
; COMPARE NUMERIC MAGNITUDE
; -----------------------------------------------------------------------------
;
; Compares exponent and magnitude after equal signs have been established.
;
; The exponent bytes are compared without their sign bits, then the packed BCD digits are checked
; from most significant to least significant. Early return preserves the relation flags needed by
; the caller.
; -----------------------------------------------------------------------------
FP_COMPARE_MAGNITUDE:
    LD A,(DE)
    DEC DE
    AND 7FH
    LD C,A
    LD A,(HL)
    DEC HL
    AND 7FH
    SUB C
    RET NZ
    LD B,07H

; Packed mantissa comparison checks high nibble then low nibble for each byte. The first differing
; decimal digit determines the relation; no binary reinterpretation is involved.

; -----------------------------------------------------------------------------
; COMPARE BCD DIGITS
; -----------------------------------------------------------------------------
;
; Compares each high and low nibble of the mantissa in decimal order.
;
; The loop separates each packed byte into nibbles and exits on the first difference. Reaching the
; end without a difference sets equality, allowing relation decoding to share one path.
; -----------------------------------------------------------------------------
FP_COMPARE_BCD_DIGITS:
    LD A,(DE)
    AND F0H
    RRCA
    LD C,A
    LD A,(HL)
    AND F0H
    RRCA
    SUB C
    RET NZ
    LD A,(DE)
    AND 0FH
    LD C,A
    LD A,(HL)
    AND 0FH
    SUB C
    RET NZ
    DEC DE
    DEC HL
    DJNZ FP_COMPARE_BCD_DIGITS
    RET

; -----------------------------------------------------------------------------
; COMPARE BASIC STRINGS
; -----------------------------------------------------------------------------
;
; Compares two length-prefixed strings lexicographically and returns relation flags.
;
; The shorter length limits byte comparison. Equal prefixes compare as equal until the remaining
; length decides the result. CY/S/Z are arranged like numeric comparison so the same relation
; evaluator can produce -1 or 0.
;
; Entry:
;   Two string elements on the BASIC stack.
;
; Exit:
;   CY, S, Z and A describe the lexical relation.
;
; Effects:
;   Leaves source strings intact for the immediate comparison, but the next stack write may
;   reclaim them.
;
; Destroys:
;   AF, BC, DE, HL, IY.
; -----------------------------------------------------------------------------
STRING_COMPARE:
    PUSH IY
    POP HL
    INC HL
    PUSH HL
    LD C,(HL)
    LD B,00H
    ADD HL,BC
    INC HL
    INC HL
    PUSH HL
    LD A,(HL)
    CP C
    JR C,STRING_COMPARE_LENGTH
    LD A,C

; String comparison stores the shorter length as the common-prefix bound but retains both original
; lengths. Thus equal prefixes are ordered by length, while a differing character wins
; immediately.

; -----------------------------------------------------------------------------
; LIMIT STRING COMPARISON
; -----------------------------------------------------------------------------
;
; Chooses the common-prefix length before bytewise string comparison.
;
; The shorter descriptor length bounds the CPI loop. The original lengths are retained so equal
; prefixes can be ordered by length after byte comparison.
; -----------------------------------------------------------------------------
STRING_COMPARE_LENGTH:
    LD C,(HL)
    ADD HL,BC
    INC HL
    PUSH HL
    POP IY
    LD C,A
    INC BC
    POP DE
    POP HL

; The byte comparison returns flags compatible with numeric relation handling. Length subtraction
; is performed only after CPI exhausts the shared prefix.

; -----------------------------------------------------------------------------
; COMPARE STRING BYTES
; -----------------------------------------------------------------------------
;
; Compares the common string prefix and then resolves a length-only difference.
;
; CPI scans the shared bytes; if they match, the saved lengths are subtracted to produce the
; relation. The result flags deliberately match numeric comparison semantics.
; -----------------------------------------------------------------------------
STRING_COMPARE_BYTES:
    LD A,(DE)
    INC DE
    CPI
    JP PO,LF6FB
    JR Z,STRING_COMPARE_BYTES

LF6FB:
    DEC HL
    LD C,(HL)
    LD L,A
    LD H,B
    SBC HL,BC
    LD A,H
    RET Z
    RET M
    LD A,01H
    RET

LF707:
    BIT 7,(IY+08H)
    RET Z

LF70C:
    PUSH IY
    POP HL
    LD DE,0007H
    OR A

LF713:
    INC HL
    LD A,D
    SBC A,(HL)
    DAA
    LD (HL),A
    DEC E
    JR NZ,LF713
    RET

LF71C:
    BIT 7,(IY+08H)
    RET Z
    CALL LF70C
    JR LF72B

; Arithmetic routine 4: negates the sign of the 9-byte number on the BASIC Stack.

; -----------------------------------------------------------------------------
; NEGATE BASIC NUMBER
; -----------------------------------------------------------------------------
;
; Toggles the sign bit of a nonzero BASIC number.
;
; Zero is left unchanged. For a nonzero number the sign bit in the exponent/sign byte is inverted;
; mantissa and exponent remain untouched.
;
; Entry:
;   IY points to a BASIC numeric element.
;
; Exit:
;   The same value with its sign inverted.
;
; Effects:
;   Writes one stack byte.
;
; Destroys:
;   AF.
; -----------------------------------------------------------------------------
FP_NEGATE:
    LD A,(IY+06H)
    OR A
    RET Z

LF72B:
    LD A,80H
    XOR (IY+08H)
    LD (IY+08H),A
    RET

; -----------------------------------------------------------------------------
; NORMALIZE BASIC NUMBER
; -----------------------------------------------------------------------------
;
; Removes leading zero BCD digits or shifts a result until its exponent/mantissa are canonical.
;
; The normalizer handles zero, positive and negative exponent adjustments, and significant-digit
; shifts. It is shared by addition, multiplication, division, integer conversion, and decimal
; parsing; range violations use the BASIC overflow path.
;
; Entry:
;   IY points to a numeric element.
;
; Exit:
;   The element is normalized in place.
;
; Effects:
;   Rewrites exponent and packed mantissa bytes.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
FP_NORMALIZE:
    CALL FP_TEST_ZERO
    RET Z
    LD A,(IY+07H)
    OR A
    JR Z,FP_SHIFT_LEFT_NORMALIZE
    LD A,01H
    CALL FP_DECIMAL_SHIFT
    LD A,(IY+08H)
    AND 7FH
    CP 7EH
    JP Z,LF912
    INC (IY+08H)
    RET

; Normalization handles cancellation and underflow by decimal shifting, not by changing the type
; byte. If every mantissa byte becomes zero, it explicitly canonicalizes the exponent/sign to
; zero.

; -----------------------------------------------------------------------------
; LEFT-SHIFT NORMALIZATION
; -----------------------------------------------------------------------------
;
; Shifts a result toward its canonical leading digit while decreasing its exponent.
;
; When the most significant packed digits are zero, the helper shifts the mantissa left in decimal
; nibble steps and decrements the biased exponent. Reaching zero clears the complete numeric
; element instead of leaving a signed zero.
; -----------------------------------------------------------------------------
FP_SHIFT_LEFT_NORMALIZE:
    LD A,(IY+06H)
    AND F0H
    RET NZ
    CALL FP_SHIFT_RIGHT_BCD
    LD A,(IY+08H)
    DEC (IY+08H)
    AND 7FH
    JR NZ,FP_SHIFT_LEFT_NORMALIZE
    JP FP_CLEAR_NUMERIC

; Rounding increments packed BCD digits with decimal carry. A carry out of the retained mantissa
; can change the exponent, so rounding is an arithmetic state transition rather than a
; display-only operation.

; -----------------------------------------------------------------------------
; ROUND PACKED DECIMAL
; -----------------------------------------------------------------------------
;
; Rounds a mantissa after decimal conversion or an arithmetic result exceeds the retained digits.
;
; The guard digit is examined and the retained BCD digits are incremented with DAA. Carry can
; propagate across the mantissa and re-enter normalization, so formatting and arithmetic share the
; same edge-case behavior.
; -----------------------------------------------------------------------------
FP_ROUND_BCD:
    PUSH HL
    PUSH IY
    POP HL
    INC HL
    LD A,(HL)
    CP 50H
    LD (HL),00H
    JR C,LF782
    LD B,06H

LF775:
    INC HL
    LD A,(HL)
    ADD A,01H
    DAA
    LD (HL),A
    JR NC,LF77F
    DJNZ LF775

LF77F:
    CALL FP_NORMALIZE

LF782:
    POP HL
    RET

; RLD-based shifts preserve packed decimal nibbles while moving the mantissa. The displaced nibble
; is available to the caller as a guard digit for later rounding decisions.

; -----------------------------------------------------------------------------
; RIGHT-SHIFT DECIMAL MANTISSA
; -----------------------------------------------------------------------------
;
; Shifts packed BCD digits right when an exponent or multiplication carry requires it.
;
; RLD rotates successive nibbles through the mantissa while preserving decimal packing. The
; discarded nibble becomes a rounding candidate rather than silently disappearing.
; -----------------------------------------------------------------------------
FP_SHIFT_RIGHT_BCD:
    PUSH IY
    POP HL
    LD B,07H
    XOR A

LF78A:
    INC HL
    RLD
    DJNZ LF78A
    RET

; Decimal shifts are shared by exponent alignment, parsing, and formatting. Large requested shifts
; take the zero/overflow path rather than looping through unbounded memory.

; -----------------------------------------------------------------------------
; DECIMAL SHIFT HELPER
; -----------------------------------------------------------------------------
;
; Moves a BASIC mantissa by a requested number of decimal digit positions.
;
; Small shifts are implemented with nibble rotations and zero fill; larger shifts use the
; zero/overflow path. The helper is called by exponent alignment, normalization, and conversion
; code.
; -----------------------------------------------------------------------------
FP_DECIMAL_SHIFT:
    CP 0EH
    JR NC,LF7C4
    LD BC,00FFH
    PUSH IY
    POP HL
    PUSH HL
    LD E,L
    LD D,H

LF79D:
    INC HL
    SUB 02H
    INC C
    JR NC,LF79D
    JR Z,LF7B7
    INC DE
    PUSH AF
    LD A,07H
    SUB C
    LD C,A
    LDIR
    EX DE,HL
    SUB 08H
    CPL

LF7B1:
    LD (HL),B
    INC HL
    DEC A
    JR NZ,LF7B1
    POP AF

LF7B7:
    POP HL
    INC A
    RET NZ
    LD C,07H
    ADD HL,BC
    LD B,C

LF7BE:
    RRD
    DEC HL
    DJNZ LF7BE
    RET

LF7C4:
    LD C,(IY+08H)
    CALL FP_CLEAR_NUMERIC
    LD (IY+08H),C
    RET

LF7CE:
    LD HL,LFEB6
    PUSH HL

; -----------------------------------------------------------------------------
; READ BCD NIBBLE
; -----------------------------------------------------------------------------
;
; Returns a selected high or low mantissa nibble for formatting or arithmetic.
;
; The nibble index is converted to a byte address relative to IY, then the requested half-byte is
; selected. Out-of-range positions read as zero, simplifying leading-zero suppression.
; -----------------------------------------------------------------------------
FP_GET_BCD_NIBBLE:
    PUSH HL
    PUSH BC
    CALL LF7FD
    JR C,LF7DE
    LD A,(HL)
    RLCA
    RLCA
    RLCA
    RLCA

LF7DE:
    AND 0FH
    POP BC
    POP HL
    RET

; -----------------------------------------------------------------------------
; WRITE BCD NIBBLE
; -----------------------------------------------------------------------------
;
; Updates one packed mantissa nibble without disturbing its neighbor.
;
; The helper masks or rotates the target nibble into place and writes it back to the numeric
; element. It is used by decimal parsing, multiplication packing, and integer conversion.
; -----------------------------------------------------------------------------
FP_SET_BCD_NIBBLE:
    PUSH HL
    PUSH BC
    CALL LF7FD
    LD B,F0H
    JR C,LF7F7
    SLA C
    SLA C
    SLA C
    SLA C
    LD A,(HL)
    LD B,0FH

LF7F7:
    AND B
    OR C
    LD (HL),A
    POP BC
    POP HL
    RET

LF7FD:
    PUSH IY
    POP HL
    SRL B
    PUSH AF
    LD A,08H
    SUB B
    LD C,A
    LD B,00H
    ADD HL,BC
    POP AF
    LD C,A
    LD A,(HL)
    RET

; Number formatting writes the decimal result into the BASIC formatting workspace beginning at
; 1931H.

; -----------------------------------------------------------------------------
; FORMAT NUMBER AS ASCII
; -----------------------------------------------------------------------------
;
; Converts the top BASIC number into the decimal text used by PRINT and error reporting.
;
; The formatter handles zero, sign, exponent, decimal placement, and fixed versus scientific
; notation. It extracts BCD digits, suppresses insignificant leading/trailing zeros, applies the
; configured formatting limits, and writes the result into the BASIC output workspace at 1931H.
;
; Entry:
;   IY points to a normalized BASIC number.
;
; Exit:
;   ASCII representation and length are stored in the formatting workspace.
;
; Effects:
;   Uses numeric formatting buffers and may round the displayed value.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
; -----------------------------------------------------------------------------
FP_TO_ASCII:
    LD A,(IY+06H)
    OR A
    LD HL,1931H
    LD (HL),30H
    JR Z,LF897
    BIT 7,(IY+08H)
    JR Z,FORMAT_FIXED_OR_EXPONENT
    LD (HL),2DH
    INC HL
    CALL FP_NEGATE

; The number formatter decides fixed versus E notation from exponent and significant-digit
; placement before writing digits. It therefore knows the final shape and length while emitting
; the mantissa.

; -----------------------------------------------------------------------------
; CHOOSE NUMBER DISPLAY FORM
; -----------------------------------------------------------------------------
;
; Chooses fixed-point or scientific notation from the exponent and significant-digit position.
;
; The formatter scans decimal positions, decides where the point belongs, and switches to an E
; suffix when the value cannot be represented compactly in fixed form. The decision occurs before
; digit emission so the output length byte remains correct.
; -----------------------------------------------------------------------------
FORMAT_FIXED_OR_EXPONENT:
    CALL FP_ROUND_BCD
    LD B,0EH

LF82A:
    DEC B
    LD A,B
    CP 04H
    CALL NC,FP_GET_BCD_NIBBLE
    JR Z,LF82A
    LD C,B
    LD B,04H
    LD E,(IY+08H)
    LD A,E
    SUB 3BH
    LD D,A
    LD A,E
    CP 4AH
    JR NC,LF858
    CP 40H
    JR NC,LF850
    SUB C
    JR C,LF859
    CP 32H
    JR C,LF859
    LD B,D
    JR LF85B

LF850:
    LD A,C
    CP D
    JR NC,LF85B
    LD C,D
    OR A
    JR LF85C

LF858:
    SCF

LF859:
    LD D,05H

LF85B:
    INC C

LF85C:
    PUSH AF
    LD A,B

LF85E:
    CP D
    JR NZ,FORMAT_DIGIT_LOOP
    LD (HL),2EH
    INC HL

; Digit emission reads BCD nibbles and adds 30H; the decimal point is inserted when the selected
; position is reached. Leading zero suppression changes the loop start, not the stored numeric
; value.

; -----------------------------------------------------------------------------
; EMIT FORMATTED DIGITS
; -----------------------------------------------------------------------------
;
; Extracts BCD nibbles, inserts the decimal point, and emits ASCII digits into the output
; workspace.
;
; Leading zero suppression and the first significant digit determine the loop bounds. Each
; selected BCD nibble receives 30H and is written sequentially; a deferred point is inserted at
; the chosen position.
; -----------------------------------------------------------------------------
FORMAT_DIGIT_LOOP:
    LD A,B
    SUB 04H
    LD A,00H
    CALL P,FP_GET_BCD_NIBBLE
    ADD A,30H
    LD (HL),A
    INC HL
    INC B
    LD A,B
    CP C
    JR NZ,LF85E
    POP AF
    JR NC,LF898
    LD (HL),45H
    INC HL
    LD (HL),2BH
    LD A,E
    SUB 40H
    JR NC,FORMAT_EXPONENT_DIGITS
    LD (HL),2DH
    NEG

; The exponent suffix is emitted as a signed decimal magnitude after E. The final count is derived
; from the workspace pointer, which keeps PRINT and error messages compatible with variable-width
; exponents.

; -----------------------------------------------------------------------------
; EMIT EXPONENT SUFFIX
; -----------------------------------------------------------------------------
;
; Appends the signed decimal exponent when scientific notation was selected.
;
; The biased exponent is converted to a signed magnitude and emitted after E and its sign. The
; final length is computed from the workspace base rather than from an assumed fixed width.
; -----------------------------------------------------------------------------
FORMAT_EXPONENT_DIGITS:
    INC HL
    LD B,FFH

LF889:
    INC B
    LD C,A
    SUB 0AH
    JR NC,LF889
    LD A,B
    ADD A,30H
    LD (HL),A
    INC HL
    SUB B
    ADD A,C
    LD (HL),A

LF897:
    INC HL

LF898:
    LD DE,1930H
    SCF
    SBC HL,DE
    EX DE,HL
    LD (HL),E
    RET

; -----------------------------------------------------------------------------
; COPY STRING TO BASIC STACK
; -----------------------------------------------------------------------------
;
; Copies a length-prefixed source string into newly allocated BASIC-stack space.
;
; The routine checks available stack memory, copies string bytes backwards where necessary to
; avoid overlap, writes the length and string identifier, and advances IY. It is used for literals
; and for converting input buffers into BASIC values.
;
; Entry:
;   HL = source string buffer.
;
; Exit:
;   A string element is allocated on the BASIC stack.
;
; Effects:
;   Allocates and copies stack memory; can raise Overflow.
;
; Destroys:
;   AF, BC, DE, HL, IY.
; -----------------------------------------------------------------------------
STRING_TO_STACK:
    PUSH DE
    PUSH BC
    LD E,FFH
    LD C,00H

LF8A7:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,LF8A7
    CP 22H
    JR NZ,LF8B4
    INC C

LF8B2:
    LD A,(HL)
    INC HL

LF8B4:
    INC E
    INC C
    DEC C
    JR Z,LF8C7
    CP FFH
    JR Z,LF8D3
    CP 22H
    JR NZ,LF8B2
    CP (HL)
    INC HL
    JR Z,LF8B2
    JR LF8D3

LF8C7:
    CP 2CH
    JR Z,LF8D3
    CP 21H
    JR Z,LF8D3
    CP FDH
    JR C,LF8B2

LF8D3:
    DEC HL
    PUSH HL
    PUSH DE
    LD B,E
    PUSH IY
    POP DE
    CALL LFC8E
    INC B

LF8DE:
    DEC B
    DEC DE
    JR Z,LF8F8
    DEC HL
    LD A,(HL)
    CP 22H
    JR NZ,LF8EE
    INC C
    DEC C
    JR Z,LF8EE
    DEC HL
    LD A,(HL)

LF8EE:
    LD (DE),A
    CP 20H
    JR NC,LF8DE
    OR 80H
    LD (DE),A
    JR LF8DE

LF8F8:
    EX DE,HL
    POP DE
    LD (HL),E
    DEC HL
    LD (HL),01H
    PUSH HL
    POP IY
    POP HL
    POP BC
    POP DE
    RET

LF905:
    LD A,(HL)
    CP 22H
    JR Z,STRING_TO_STACK
    CALL ASCII_TO_FP
    JP NC,LFD5A
    RLA
    RET NC

LF912:
    RST 08H
    DB 0DH                                ; error code 0DH: Overflow

; ASCII parsing accepts sign, decimal point, and optional signed E exponent; insignificant
; underflow becomes zero.

; -----------------------------------------------------------------------------
; PARSE ASCII NUMBER
; -----------------------------------------------------------------------------
;
; Parses decimal digits, a decimal point, and an optional E exponent into BASIC floating-point
; format.
;
; The parser skips spaces, records sign, collects significant digits into packed BCD bytes,
; adjusts the exponent for the decimal point, and accepts an optional signed E exponent. It
; retains the required precision, rounds/normalizes the result, and turns excessively small values
; into zero; malformed input and oversized exponents use the standard error status.
;
; Entry:
;   HL = first byte of an ASCII numeric string.
;
; Exit:
;   A normalized numeric element is pushed on the BASIC stack; source pointer/status indicate
;   success.
;
; Effects:
;   Allocates stack space and uses conversion workspace.
;
; Destroys:
;   AF, BC, DE, HL, IY.
; -----------------------------------------------------------------------------
ASCII_TO_FP:
    PUSH BC
    PUSH DE
    CALL FP_ALLOCATE_NUMERIC
    PUSH HL
    PUSH IY
    POP HL
    LD DE,0007H
    ADD HL,DE
    EX (SP),HL
    LD E,3FH
    LD B,0BH

; ASCII parsing tracks sign, decimal point, and significant-digit state in flags while packing
; digits into the numeric workspace. Once the precision limit is reached, later digits still
; influence rounding/overflow state even if they are not stored.

; -----------------------------------------------------------------------------
; SCAN INPUT SIGN AND DIGITS
; -----------------------------------------------------------------------------
;
; Consumes leading spaces/sign and packs decimal digits while parsing an ASCII number.
;
; The state flags distinguish sign, decimal point, and whether a significant digit has been seen.
; Each accepted digit is inserted into the seven-byte BCD mantissa until the precision limit is
; reached.
; -----------------------------------------------------------------------------
ASCII_SCAN_SIGN_DIGITS:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,ASCII_SCAN_SIGN_DIGITS
    CP 2BH
    JR Z,LF936
    CP 2DH
    JR NZ,LF938
    SET 1,D

LF936:
    LD A,(HL)
    INC HL

LF938:
    CP 2EH
    JR Z,LF969
    SUB 3AH
    JR NC,LF971
    ADD A,0AH
    JR NC,LF971
    LD C,A
    SET 7,D
    LD A,D
    RRCA
    JR NZ,ASCII_DECIMAL_STATE
    BIT 2,D
    JR NZ,LF956
    JR NC,LF936
    DEC E
    JR LF936

; The decimal point changes the provisional exponent rather than moving already packed digits. A
; second point or a nonnumeric terminator exits through the parser's validation state.

; -----------------------------------------------------------------------------
; TRACK DECIMAL POINT
; -----------------------------------------------------------------------------
;
; Adjusts the provisional exponent as digits are consumed before and after the decimal point.
;
; Digits before the point advance the significant position; fractional digits decrease the
; exponent contribution. The state prevents a second decimal point from being accepted as part of
; the same number.
; -----------------------------------------------------------------------------
ASCII_DECIMAL_STATE:
    SET 2,D

LF956:
    JR C,LF959
    INC E

LF959:
    LD A,B
    DEC A
    JR Z,LF936
    LD B,A
    EX (SP),HL
    RRCA
    JR C,LF963
    DEC HL

LF963:
    LD A,C
    RLD
    EX (SP),HL
    JR LF936

LF969:
    SET 7,D
    BIT 0,D
    SET 0,D
    JR Z,LF936

LF971:
    EX (SP),HL
    DEC B
    BIT 0,B
    JR Z,ASCII_SCAN_EXPONENT
    XOR A
    RLD

; An optional E exponent is parsed after the mantissa and combined with the decimal-point
; adjustment. The parser accepts a signed exponent but rejects an exponent marker with no valid
; digits.

; -----------------------------------------------------------------------------
; SCAN E EXPONENT
; -----------------------------------------------------------------------------
;
; Recognizes an optional E exponent and folds its signed value into the numeric exponent.
;
; The parser accepts an optional exponent sign, accumulates decimal exponent digits, applies the
; sign, and combines the result with the decimal-point adjustment. Excessive exponent magnitude is
; flagged before final normalization.
; -----------------------------------------------------------------------------
ASCII_SCAN_EXPONENT:
    POP HL
    DEC HL
    LD A,(HL)
    CALL LFBC8
    CP 45H
    LD A,E
    JR NZ,ASCII_FP_FINALIZE
    SET 7,D
    INC HL
    LD A,(HL)
    CP 98H
    JR Z,LF99B
    CP 2BH
    JR Z,LF99B
    CP A2H
    JR Z,LF999
    CP 2DH
    JR NZ,LF99C

LF999:
    SET 5,D

LF99B:
    INC HL

LF99C:
    LD B,00H
    DEC HL

LF99F:
    INC HL
    LD A,(HL)
    SUB 3AH
    JR NC,LF9B7
    ADD A,0AH
    JR NC,LF9B7
    LD C,A
    LD A,B
    ADD A,A
    LD B,A
    ADD A,A
    ADD A,A
    ADD A,B
    ADD A,C
    LD B,A
    JP P,LF99F
    SET 6,D

LF9B7:
    LD A,B
    BIT 5,D
    JR Z,LF9BE
    NEG

LF9BE:
    ADD A,E
    CP 7FH
    JR C,ASCII_FP_FINALIZE
    SET 6,D

; Parsed values are finalized by writing the biased exponent/sign byte, applying the leading
; minus, testing zero, and normalizing. This makes tiny underflow results canonical zero and keeps
; sign handling consistent with arithmetic.

; -----------------------------------------------------------------------------
; FINALIZE PARSED NUMBER
; -----------------------------------------------------------------------------
;
; Stores sign/exponent state, normalizes the parsed mantissa, and returns parser status.
;
; The finalizer writes the biased exponent/sign byte, toggles the sign for a leading minus,
; detects an all-zero mantissa, and calls the common normalizer. Underflow therefore becomes
; canonical zero rather than a denormal stack value.
; -----------------------------------------------------------------------------
ASCII_FP_FINALIZE:
    PUSH DE
    LD (IY+08H),A
    BIT 1,D
    CALL NZ,FP_NEGATE
    CALL FP_TEST_ZERO
    POP DE
    LD A,D
    RRA
    AND D
    AND 20H
    LD A,D
    RLA
    CALL NZ,FP_CLEAR_NUMERIC
    POP DE
    POP BC
    RET

; The zero test scans all mantissa bytes and clears the exponent/sign byte when none is nonzero.
; It is called after cancellation and parsing, so negative zero cannot survive as a distinct BASIC
; value.

; -----------------------------------------------------------------------------
; TEST NUMERIC ZERO
; -----------------------------------------------------------------------------
;
; Scans all mantissa bytes and canonicalizes an all-zero value.
;
; A zero mantissa forces the exponent/sign byte to zero. This shared check keeps zero signless
; after parsing, arithmetic cancellation, and conversion.
; -----------------------------------------------------------------------------
FP_TEST_ZERO:
    PUSH HL
    PUSH IY
    POP HL
    LD B,07H
    XOR A

LF9E6:
    INC HL
    OR (HL)
    JR NZ,LF9EF
    DJNZ LF9E6
    LD (IY+08H),A

LF9EF:
    POP HL
    RET

LF9F1:
    CALL LFC8E

; Numeric allocation advances IY by exactly nine bytes. Every caller must initialize the
; identifier and all numeric fields before another routine treats the new stack element as valid.

; -----------------------------------------------------------------------------
; ALLOCATE NUMERIC STACK ELEMENT
; -----------------------------------------------------------------------------
;
; Makes room for a nine-byte numeric element immediately above the current BASIC stack value.
;
; The helper moves IY by the fixed numeric element size and is used by ASCII parsing, integer
; conversion, and arithmetic result construction. The allocation is logical; callers must
; initialize every byte before exposing the element.
; -----------------------------------------------------------------------------
FP_ALLOCATE_NUMERIC:
    LD DE,FFF7H
    ADD IY,DE

; Clearing a numeric element is a logical constructor: type 09H is written first, then eight data
; bytes are zeroed. The arithmetic routines depend on this invariant for zero fast paths.

; -----------------------------------------------------------------------------
; CLEAR NUMERIC ELEMENT
; -----------------------------------------------------------------------------
;
; Initializes a newly allocated numeric element as a zero value.
;
; The identifier byte is set to 09H and the eight following bytes are cleared. Later conversion
; code can fill only the mantissa/exponent fields while retaining a valid type marker.
; -----------------------------------------------------------------------------
FP_CLEAR_NUMERIC:
    PUSH IY
    EX (SP),HL
    LD (HL),09H
    LD B,08H
    XOR A

LFA01:
    INC HL
    LD (HL),A
    DJNZ LFA01
    POP HL
    SCF
    RET

LFA08:
    CALL FP_CLEAR_NUMERIC
    LD (IY+08H),40H
    LD (IY+06H),10H
    RET

LFA14:
    LD HL,LFEB6
    PUSH HL
    JR FP_CLEAR_NUMERIC

LFA1A:
    POP AF

; -----------------------------------------------------------------------------
; DISCARD STACK NUMBER
; -----------------------------------------------------------------------------
;
; Reclaims one nine-byte BASIC numeric element by moving IY backward.
;
; Discarding a value only changes the BASIC stack boundary; its bytes are not cleared. This
; inexpensive primitive is used when integer conversion, failed parsing, or expression evaluation
; no longer needs the original value.
;
; Entry:
;   IY points above a numeric element.
;
; Exit:
;   IY is moved back by nine bytes.
;
; Effects:
;   Reclaims stack space without memory clearing.
;
; Destroys:
;   IY.
; -----------------------------------------------------------------------------
DISCARD_NUMBER:
    LD DE,0009H
    ADD IY,DE
    RET

; -----------------------------------------------------------------------------
; DISCARD STACK STRING
; -----------------------------------------------------------------------------
;
; Reclaims a length-prefixed BASIC string from the stack.
;
; The length byte is read and IY is moved backward over the identifier, length, and payload. As
; with numeric discard, the data is logically dead but not erased.
;
; Entry:
;   IY points above a string element.
;
; Exit:
;   IY points to the preceding stack element.
;
; Effects:
;   Reclaims variable-sized stack space.
;
; Destroys:
;   AF, IY.
; -----------------------------------------------------------------------------
DISCARD_STRING:
    LD E,(IY+01H)
    LD D,00H
    INC DE
    INC DE
    ADD IY,DE
    RET

; -----------------------------------------------------------------------------
; PUSH INTEGER ON BASIC STACK
; -----------------------------------------------------------------------------
;
; Converts signed HL into the normalized nine-byte BASIC numeric representation.
;
; The integer is shifted into decimal digit positions, with carry becoming an additional digit
; when required. The routine chooses an exponent, writes the identifier/sign/mantissa bytes, and
; finishes through normalization.
;
; Entry:
;   HL = signed integer.
;
; Exit:
;   HL's value is pushed as a BASIC number.
;
; Effects:
;   Allocates nine bytes on the BASIC stack.
;
; Destroys:
;   AF, BC, DE, HL, IY.
; -----------------------------------------------------------------------------
INTEGER_TO_STACK:
    CALL LF9F1
    INC IY
    LD (171CH),IY
    DEC IY
    PUSH HL
    CALL LFC83
    SCF

LFA3B:
    ADC HL,HL
    JR NC,LFA3B
    LD C,01H
    JR LFA52

LFA43:
    INC B
    INC C
    DB 3EH                                ; = LD A,41H when run from here, hiding the entry below

LFA46:
    LD B,C

LFA47:
    INC DE
    LD A,(DE)
    ADC A,A
    DAA
    LD (DE),A
    DJNZ LFA47
    JR C,LFA43
    ADC HL,HL

LFA52:
    LD DE,(171CH)
    JR NZ,LFA46
    POP AF
    AND 80H
    OR 49H
    LD (IY+08H),A
    JP FP_NORMALIZE

LFA63:
    CALL LFC8E
    LD BC,0005H
    ADD HL,BC
    PUSH IY
    POP DE
    DEC DE
    LDD
    XOR A
    LD (DE),A
    DEC DE
    INC BC
    LDDR
    LD (DE),A
    LD A,09H
    DEC DE
    JR LFA8D

LFA7C:
    INC HL

LFA7D:
    LD C,(HL)
    LD B,00H
    CALL LFC8E
    ADD HL,BC
    INC BC
    PUSH IY
    POP DE
    DEC DE
    LDDR
    LD A,01H

LFA8D:
    LD (DE),A
    PUSH DE
    POP IY
    RET

LFA92:
    CALL LFC8E
    PUSH IY
    POP DE
    DEC DE
    LD HL,0009H
    LD C,L
    LD B,H
    ADD HL,DE
    LDDR
    INC DE
    PUSH DE
    POP IY
    RET

LFAA6:
    CALL LFC8E
    PUSH IY
    POP HL
    DEC HL
    LD E,L
    LD D,H
    LD BC,0012H
    ADD HL,BC
    LDDR
    INC DE
    PUSH DE
    POP IY
    RET

LFABA:
    CALL LFC43
    JR LFAC4

LFABF:
    CALL STACK_NUMBER_TO_INTEGER
    EX DE,HL

; -----------------------------------------------------------------------------
; CONVERT STACK NUMBER TO INTEGER
; -----------------------------------------------------------------------------
;
; Converts a BASIC number to signed HL with range checking.
;
; The exponent determines how many BCD digits participate. The routine repeatedly multiplies the
; accumulating HL by ten, adds each digit, applies the sign, and discards the original stack
; number. Values outside the signed integer range raise Bad argument or Overflow through the
; caller's selected path.
;
; Entry:
;   IY points to a BASIC numeric element.
;
; Exit:
;   HL = signed integer; the source stack element is reclaimed.
;
; Effects:
;   Reads and invalidates stack data.
;
; Destroys:
;   AF, BC, DE, HL, IY.
; -----------------------------------------------------------------------------
STACK_NUMBER_TO_INTEGER:
    DB F6H                                ; = OR 37H when run from here, hiding the entry below

LFAC4:
    SCF

LFAC5:
    PUSH BC
    PUSH DE
    CALL C,EVAL_NUMERIC_EXPRESSION
    LD A,(IY+08H)
    PUSH AF
    AND 7FH
    SBC HL,HL
    CP 45H
    JR NC,LFB14
    SUB 40H
    JR C,LFB06
    ADD A,05H
    LD C,A
    LD B,04H

LFADF:
    LD A,H
    AND E0H
    JR NZ,LFB14
    ADD HL,HL
    LD E,L
    LD D,H
    ADD HL,HL
    ADD HL,HL
    ADD HL,DE
    JR C,LFB14
    CALL FP_GET_BCD_NIBBLE
    LD E,A
    LD D,00H
    ADD HL,DE
    JR C,LFB14
    INC B
    LD A,C
    CP B
    JR NZ,LFADF
    POP AF
    PUSH AF
    OR A
    CALL M,LFC86
    POP AF
    XOR H
    JP M,LFB14
    DB 3EH                                ; = LD A,F1H when run from here, hiding the entry below

LFB06:
    POP AF
    LD DE,0009H
    ADD IY,DE
    LD A,H
    OR A
    EXX
    LD A,B
    EXX
    POP DE
    POP BC
    RET

LFB14:
    RST 08H
    DB 04H                                ; error code 04H: Bad argument

; -----------------------------------------------------------------------------
; CONVERT STACK NUMBER TO BYTE
; -----------------------------------------------------------------------------
;
; Converts a BASIC number to A after requiring an integer byte-sized value.
;
; The helper delegates to integer conversion, verifies that the high byte is empty or
; sign-compatible, and returns the low byte. It is used for ports, character codes, and other
; byte-valued BASIC arguments.
;
; Entry:
;   Numeric element on BASIC stack.
;
; Exit:
;   A = byte value; flags/status indicate validity.
;
; Effects:
;   Reclaims the source numeric element.
;
; Destroys:
;   AF, BC, DE, HL, IY.
; -----------------------------------------------------------------------------
STACK_NUMBER_TO_BYTE:
    CALL LFC43
    DB 3EH                                ; = LD A,F6H when run from here, hiding the entry below

LFB1A:
    DB F6H                                ; = OR 37H when run from here, hiding the entry below

LFB1B:
    SCF
    PUSH HL
    CALL LFAC5
    INC H
    DEC H
    JP NZ,LFB14
    LD A,L
    POP HL
    RET

; Arithmetic routine 13: moves from BASIC Stack to the address in HL; 9 bytes freed from Stack.

LFB28:
    CALL FP_ROUND_BCD
    EX DE,HL
    PUSH IY
    POP HL
    INC HL
    INC HL
    LD BC,0005H
    LDIR
    INC HL
    LDI
    JR LFB57

LFB3B:
    BIT 1,(IX+01H)
    JR NZ,LFB28

LFB41:
    LD C,(HL)
    INC HL
    PUSH IY
    POP DE
    INC DE
    LD A,(DE)
    INC C
    JR NZ,LFB4C
    DEC C

LFB4C:
    CP C
    JP NC,LF912
    LD C,A
    LD B,00H
    INC BC
    EX DE,HL
    LDIR

LFB57:
    PUSH HL
    POP IY
    RET

; Bytes copied into RAM at initialization (addresses 0008H-002FH): error handlers, RST 18H entry,
; and function call dispatcher.

; Character matrix table for character codes 128-160, ten bytes per character.
    DB E1H, 7EH, B7H, E5H, 00H, 00H, 00H, 00H, C8H, E1H, C3H, 5CH, FDH, C3H, 00H, 00H ; |.~.........\....|
    DB C3H, 82H, FBH, 32H, 1FH, 00H, F7H, 00H, C9H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |...2............|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H                                              ; |.......|

; RST 18 operation bytes are threaded through the jump table and advance the next-byte pointer at
; 1718H-1719H.
; RST18 dispatch saves the continuation pointer in 1718H-1719H and executes a sequence of
; operation bytes. The high bit marks the final byte; intermediate operations may push, copy,
; delete, or combine typed stack values.

; -----------------------------------------------------------------------------
; RST 18H BASIC STACK DISPATCH
; -----------------------------------------------------------------------------
;
; Executes the compact arithmetic and stack-operation bytecode following an RST 18H instruction.
;
; RST 18H is a threaded helper language used throughout BASIC. The dispatcher saves the
; return/source pointer, fetches each operation byte, and uses a jump table to perform stack
; arithmetic, conversions, duplication, deletion, symbol copies, and control helpers. The
; operation stream may contain several bytes, so the dispatcher advances 1718H-1719H until the
; terminating operation returns to the interpreter.
;
; The mechanism lets math-heavy BASIC routines express repeated stack transformations in a few
; bytes while keeping the actual algorithms in reusable subroutines.
;
; Entry:
;   The byte following RST 18H selects the operation; 1718H/1719H tracks the next byte.
;
; Exit:
;   Operation-specific values and stack pointer updates.
;
; Effects:
;   Manipulates BASIC stack elements, IY, and interpreter workspace.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers, IY.
;
; Note:
;   The first 48 bytes copied to U0 RAM include the RST 18H entry and the ordinary error/call
;   stubs.
; -----------------------------------------------------------------------------
RST18_DISPATCH:
    LD (171EH),HL
    LD (RST18_ARG),A
    LD HL,(RST18_NEXT)
    EX (SP),HL

LFB8C:
    LD A,(HL)
    INC HL
    LD (RST18_NEXT),HL
    ADD A,A
    PUSH AF
    ADD A,C7H
    LD L,A
    LD H,C0H
    LD A,(HL)
    INC HL
    LD H,(HL)
    LD L,A
    LD (RST18_OP_JP+1),HL
    LD HL,(171EH)
    LD A,(RST18_ARG)
    CALL RST18_OP_JP
    LD (RST18_ARG),A
    LD (171EH),HL
    LD HL,(RST18_NEXT)
    POP AF
    JR NC,LFB8C
    EX (SP),HL
    LD (RST18_NEXT),HL
    LD HL,(171EH)
    LD A,(RST18_ARG)
    RET

LFBBF:
    LD A,(DE)
    INC B
    DEC B
    RET NZ
    CP FFH
    RET Z
    AND 7FH

LFBC8:
    CP 20H
    JR C,LFBD5
    CP 61H
    RET C
    CP 7BH
    RET NC
    AND DFH
    RET

LFBD5:
    CP 10H
    RET C
    CP 19H
    RET NC
    AND EFH
    RET

LFBDE:
    CP 02H
    JP NZ,LFD5A
    CALL STACK_NUMBER_TO_INTEGER
    CALL DD44H
    RET NC
    RST 08H
    DB 02H                                ; error code 02H: Line missing

LFBEC:
    LD C,20H

LFBEE:
    CALL LFBFD
    RET NZ
    CP FDH
    JR Z,LFC43
    CP FEH
    RET NC
    RST 08H
    DB 01H                                ; error code 01H: Not understood

LFBFB:
    LD C,20H

LFBFD:
    LD (IX+05H),C

LFC00:
    EXX
    LD A,B
    EXX
    CP A7H
    RET NZ
    CALL STACK_NUMBER_TO_BYTE
    AND 07H
    ADD A,A
    ADD A,A
    ADD A,A
    ADD A,A
    LD (FUNC_CLASS),A
    LD C,A
    XOR A
    EXX
    LD A,B
    EXX
    RET

LFC18:
    LD (IX+05H),20H
    LD A,(PAPER)
    LD C,A
    LD A,(INK)
    CP C
    RET NZ
    INC A
    LD (INK),A
    LD A,(PORT06_MIRROR)
    BIT 1,A
    RET Z
    LD A,C
    CPL
    LD (INK),A
    RET

LFC35:
    BIT 3,(IX+00H)
    RET Z
    RES 3,(IX+00H)

LFC3E:
    RST 30H
    DB 54H                                ; RST 30H fn 54H: cassette 4 -> CAS_CLOSE
    RST 30H
    DB D4H                                ; RST 30H fn D4H: cassette 4, input -> CAS_CLOSE
    RET

LFC43:
    EXX

LFC44:
    LD A,(HL)
    LD B,A
    CP FEH
    JR NC,LFC7A
    INC HL
    CP C5H
    JR Z,LFC7E
    CP 80H
    JR NC,LFC7A
    CP 20H
    JR Z,LFC44
    DEC HL
    LD B,00H
    CP 22H
    SCF
    JR Z,LFC76
    SET 1,B
    CP 41H
    JR C,LFC76
    CP 5BH
    CCF
    JR C,LFC76
    CALL SYMBOL_LOOKUP
    CALL NC,SYMBOL_CREATE_VALUE
    LD A,C
    AND 02H
    OR 01H
    LD B,A

LFC76:
    CALL C,LF905

LFC79:
    LD A,B

LFC7A:
    EXX
    CP FDH
    RET

LFC7E:
    LD C,A
    LD B,01H
    JR LFC79

LFC83:
    BIT 7,H
    RET Z

LFC86:
    DEC HL
    LD A,L
    CPL
    LD L,A
    LD A,H
    CPL
    LD H,A
    RET

LFC8E:
    PUSH HL
    PUSH DE
    LD DE,0100H
    CALL LFC99
    POP DE
    POP HL
    RET

LFC99:
    LD HL,(SYMBOL_TOP)
    ADD HL,DE
    JR C,LFCB1
    EX DE,HL
    PUSH IY
    POP HL
    SBC HL,DE
    JR C,LFCB1
    SBC HL,HL
    ADD HL,SP
    LD DE,(PAINT_STACK_MIN)
    SBC HL,DE
    RET NC

LFCB1:
    RST 08H
    DB 06H                                ; error code 06H: No memory

LFCB3:
    XOR A
    LD L,A

LFCB5:
    LD H,A
    LD A,10H

LFCB8:
    DEC A
    RET M
    ADC HL,HL
    SLA C
    RL B
    JR NC,LFCB8
    ADD HL,DE
    JR LFCB8

LFCC5:
    CALL LFCD1

LFCC8:
    RET NC
    CP CAH
    RET NC
    CALL LFCD5
    JR LFCC8

LFCD1:
    EXX
    PUSH HL
    EXX
    POP HL

LFCD5:
    LD A,(HL)
    CP FEH
    RET NC
    INC HL
    CP FDH
    JR NZ,LFCD5

LFCDE:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,LFCDE
    DEC HL
    SCF
    RET

LFCE7:
    LD B,00H
    PUSH IY
    POP HL

LFCEC:
    LD C,(HL)
    CP C
    RET Z
    INC C
    DEC C
    SCF
    RET Z
    DEC C
    JR NZ,LFCF8
    INC HL
    LD C,(HL)

LFCF8:
    INC C
    ADD HL,BC
    PUSH HL
    POP IY
    JR LFCEC

LFCFF:
    CALL EVAL_NUMERIC_EXPRESSION

LFD02:
    LD HL,8000H
    PUSH HL
    CALL INTEGER_TO_STACK
    CALL FP_ADD
    CALL STACK_NUMBER_TO_INTEGER
    POP DE
    ADD HL,DE
    RET

LFD12:
    LD C,A

LFD13:
    LD A,(HL)
    OR A
    LD A,C
    RET Z
    SUB (HL)
    INC HL
    LD E,(HL)
    INC HL
    JR NZ,LFD13
    LD D,A
    ADD HL,DE
    CALL LFC43
    CALL DBADH
    SCF
    RET

LFD27:
    CALL LFC43
    LD E,01H
    JR Z,LFD36
    CALL LFD45
    LD E,D
    CP FDH
    JR NZ,LFD40

LFD36:
    CALL LFC43
    LD D,FFH
    CP 95H
    CALL NZ,LFD45

LFD40:
    LD A,95H
    JP LFD54

LFD45:
    CALL LFAC4
    LD D,00H
    INC H
    RET Z
    DEC D
    DEC H
    RET NZ
    LD D,L
    RET

LFD51:
    CALL LFC43

LFD54:
    EXX
    CP B
    EXX
    JP Z,LFC43

LFD5A:
    LD A,01H

; BASIC_ERROR receives the numeric error code in A, unwinds interpreter state, prints a matching
; message, and returns to the command loop.

; -----------------------------------------------------------------------------
; BASIC ERROR HANDLER
; -----------------------------------------------------------------------------
;
; Unwinds the current BASIC operation, selects an error message, and returns to the command loop.
;
; A contains the error code. The handler saves the interpreter context, recognizes STOP and
; special system errors, frees or resets transient stack state, and selects the matching
; length-prefixed message from the error table. It prints the message and current line context,
; restores the command environment, and resumes BASIC in its normal ready/error state.
;
; Entry:
;   A = BASIC error code; interpreter context and current line are active.
;
; Exit:
;   Control returns to the BASIC command loop after reporting the error.
;
; Effects:
;   Unwinds stack/interpreter state and writes screen output.
;
; Destroys:
;   AF, BC, DE, HL, IY and transient interpreter state.
; -----------------------------------------------------------------------------
BASIC_ERROR:
    LD IY,(BASIC_SP)
    CP F5H

; Error dispatch preserves enough current-line and command context to distinguish an immediate
; error from a program error. It unwinds transient stack state before emitting the message so
; stale expression values do not become live after recovery.
    JP Z,LFFA3
    PUSH AF
    CP 06H
    CALL Z,BASIC_WORKSPACE_INIT
    CALL LFC35
    CALL LFC18
    CALL PRINT_INLINE_TEXT

; " " text prefix for BASIC error messages.
    DB 06H, 0DH, 0AH, "*** "              ; inline text, 6 bytes
    POP AF
    BIT 7,A
    JR NZ,LFD98
    LD HL,FDC6H
    LD BC,FFFFH

LFD86:
    INC BC
    ADD HL,BC
    LD C,(HL)
    INC HL
    INC C
    DEC C
    JR Z,LFD92
    CP C
    LD C,(HL)
    JR NZ,LFD86

LFD92:
    INC HL
    CALL LFEDD
    JR LFDB0

LFD98:
    LD L,A
    CALL PRINT_INLINE_TEXT

; "System error" text.
    DB 0DH, "System error "               ; inline text, 13 bytes
    XOR A
    LD H,A
    LD B,A
    CALL LFF1B

LFDB0:
    CALL PRINT_INLINE_TEXT
    DB 04H, ".", 0BH, 0DH, 0AH            ; inline text, 4 bytes
    SET 1,(IX+00H)
    LD HL,(CUR_LINE)
    XOR A
    CALL BASIC_PRINT_LINE
    JP LE10E

; BASIC error message table; entries contain code, length, text, and FFH terminator.
    DB 01H, 0FH, 4EH, 6FH, 74H, 20H, 75H, 6EH, 64H, 65H, 72H, 73H, 74H, 6FH, 6FH, 64H ; |..Not understood|
    DB FFH, 02H, 06H, 4CH, 69H, 6EH                                                   ; |...Lin|

; Error-message records are code/length/text entries terminated by FFH; TVC token codes may appear
; inside the text.

; -----------------------------------------------------------------------------
; BASIC ERROR MESSAGE DATA
; -----------------------------------------------------------------------------
;
; Length-prefixed error texts indexed by BASIC error code.
;
; Entries contain an error code, a length, encoded text, and an FFH terminator. Messages include
; Not understood, Line, Argument missing, Bad argument, Subscript, Out of memory, Syntax,
; Overflow, Type mismatch, Variable declared twice, File, BASIC corrupted, and Cannot divide by 0.
; Some characters use TVC token/character codes rather than plain ASCII.
;
; Entry:
;   Selected by BASIC_ERROR.
;
; Exit:
;   Text span consumed by PRINT_LENGTH_TEXT.
;
; Effects:
;   Read-only ROM data.
;
; Destroys:
;   None.
; -----------------------------------------------------------------------------
BASIC_ERROR_MESSAGES:

; The message table is compact code/length/text data, not a null-terminated string array. Shared
; token-coded fragments are expanded by the print helper, which is why raw ROM bytes may not read
; as ordinary ASCII.
    DB 65H, 93H, FFH, 03H, 04H, 41H, 92H, 93H, FFH, 04H, 04H, 91H, 61H, 92H, FFH, 05H ; |e....A......a...|
    DB 0BH, 91H, 73H, 75H, 62H, 73H, 63H, 72H, 69H, 70H, 74H, FFH, 06H, 08H, 90H, 6DH ; |..subscript....m|
    DB 65H, 6DH, 6FH, 72H, 79H, FFH, 07H, 03H, 90H, FBH, FFH, 08H, 03H, 90H, F2H, FFH ; |emory...........|
    DB 09H, 03H, 90H, F0H, FFH, 0AH, 03H, 8FH, F8H, FFH, 0BH, 0DH, 8FH, 64H, 69H, 76H ; |.............div|
    DB 69H, 64H, 65H, 20H, 62H, 79H, 20H, 30H, FFH, 0CH, 03H, 8FH, DBH, FFH, 0DH, 09H ; |ide by 0........|
    DB 4FH, 76H, 65H, 72H, 66H, 6CH, 6FH, 77H, FFH, 0EH, 0EH, 54H, 79H, 70H, 65H, 20H ; |Overflow...Type |
    DB 6DH, 69H, 73H, 6DH, 61H, 74H, 63H, 68H, FFH, 0FH, 18H, 56H, 61H, 72H, 69H, 61H ; |mismatch...Varia|
    DB 62H, 6CH, 65H, 20H, 64H, 65H, 63H, 6CH, 61H, 72H, 65H, 64H, 20H, 74H, 77H, 69H ; |ble declared twi|
    DB 63H, 65H, FFH, 10H, 06H, 91H, 66H, 69H, 6CH, 65H, FFH, 00H, 0FH, 42H, 41H, 53H ; |ce....file...BAS|
    DB 49H, 43H, 20H, 63H, 6FH, 72H, 72H, 75H, 70H, 74H, 65H, 64H, FFH                ; |IC corrupted.|

; Inline strings begin at the return address with a length byte; the helper skips the embedded
; bytes before returning.

; -----------------------------------------------------------------------------
; PRINT INLINE TEXT
; -----------------------------------------------------------------------------
;
; Prints a length-prefixed string embedded immediately after the call site.
;
; The routine takes the return address as a pointer to a length byte, prints that many characters
; through the video device, advances past the inline bytes, and returns using the adjusted
; address. It is the compact mechanism used by error and status messages embedded in ROM code.
;
; Entry:
;   Return address points to a one-byte length followed by text.
;
; Exit:
;   Text is displayed; return address skips the embedded string.
;
; Effects:
;   Writes screen output and changes the effective return address.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
PRINT_INLINE_TEXT:
    EX (SP),HL
    CALL PRINT_LENGTH_TEXT
    EX (SP),HL
    RET

; -----------------------------------------------------------------------------
; PRINT LENGTH-PREFIXED TEXT
; -----------------------------------------------------------------------------
;
; Outputs a counted string, interpreting PRINT control characters and special formatting codes.
;
; HL points to a length byte followed by characters. The routine loops through the string, sends
; ordinary characters to the video output, handles CR/LF and special codes, and can format numeric
; values encountered by PRINT's inline control stream. It is shared by error messages, STOP
; reporting, and BASIC PRINT.
;
; Entry:
;   HL = length-prefixed text.
;
; Exit:
;   Text and formatting effects appear on the active output device.
;
; Effects:
;   Calls video/printer output paths and advances HL through the source.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
; -----------------------------------------------------------------------------
PRINT_LENGTH_TEXT:
    LD A,(HL)
    INC HL
    OR A
    RET Z
    PUSH BC
    LD B,A

LFE85:
    LD A,(HL)
    INC HL
    CALL LFE9A
    DJNZ LFE85
    POP BC
    RET

LFE8E:
    LD A,0BH
    CALL Z,LFE9A

LFE93:
    LD A,0DH
    CALL LFE9A
    LD A,0AH

LFE9A:
    PUSH AF
    PUSH BC
    PUSH DE
    LD C,A
    CALL LFEA6
    RST 10H
    POP DE
    POP BC
    POP AF
    RET

LFEA6:
    LD DE,007FH

; -----------------------------------------------------------------------------
; PRINT FORMAT DISPATCH
; -----------------------------------------------------------------------------
;
; Handles numeric formatting, separators, and PRINT special-character cases.
;
; This continuation recognizes the PRINT control stream, chooses numeric or string output, inserts
; spaces and line breaks, and routes special tokens through the compact table near FF11H. Decimal
; constants at FF19H support integer column formatting.
;
; Entry:
;   PRINT state and current token/value.
;
; Exit:
;   Formatted output is emitted; flags report line/field status.
;
; Effects:
;   Uses formatting workspace and output device routines.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
PRINT_NUMBER_OR_SPECIAL:
    LD A,(FUNC_CLASS)
    OR 01H
    OR D
    AND E

; PRINT's continuation chooses numeric, string, separator, and special-token paths while
; preserving the USING cursor between values. A trailing separator changes whether the common
; line-ending path runs.
    JP RST30_A_ENTRY

LFEB3:
    CALL LFE9A

LFEB6:
    POP HL
    JP CALL_EXTENSION_HL

LFEBA:
    BIT 7,(IY+08H)
    CALL Z,LFEC7

LFEC1:
    CALL FP_TO_ASCII
    CALL PRINT_LENGTH_TEXT

LFEC7:
    LD A,20H
    JR LFE9A

LFECB:
    ADD A,A
    CP 40H
    RRA
    CALL LFE9A
    CP 22H
    JR NZ,LFEDF
    CP C
    JR Z,LFEDB
    LD B,C
    DB 0EH                                ; = LD C,78H when run from here, hiding the entry below

LFEDB:
    LD A,B
    DB 0EH                                ; = LD C,AFH when run from here, hiding the entry below

LFEDD:
    XOR A
    LD C,A

LFEDF:
    LD A,(HL)
    INC HL
    INC A
    RET Z
    DEC A
    JP P,LFECB
    PUSH HL
    INC C
    DEC C
    PUSH AF
    CP FBH
    JR Z,LFEF3
    CP FCH
    JR C,LFEF5

LFEF3:
    LD C,FFH

LFEF5:
    CP FDH
    JR NZ,LFEFC
    POP AF
    INC C
    PUSH AF

LFEFC:
    POP AF
    JR NZ,LFF11
    CPL
    LD HL,BASIC_KEYWORD_TABLE

LFF03:
    BIT 7,(HL)
    INC HL
    JR Z,LFF03
    DEC A
    JR NZ,LFF03

LFF0B:
    LD A,(HL)
    INC HL
    BIT 7,A
    RES 7,A

LFF11:
    CALL LFE9A
    JR Z,LFF0B
    POP HL
    JR LFEDF

; Decimal place constants 1000, 100, 10, and 1 support compact integer output.

; -----------------------------------------------------------------------------
; PRINT INTEGER IN HL
; -----------------------------------------------------------------------------
;
; Formats a signed integer in HL as decimal text.
;
; The routine repeatedly subtracts decimal place constants 1000, 100, 10, and 1, suppresses
; leading zeroes, and emits the resulting digits through the text output helper. It is used for
; line numbers and compact integer portions of PRINT output.
;
; Entry:
;   HL = signed integer.
;
; Exit:
;   Decimal digits are emitted.
;
; Effects:
;   Writes output through PRINT_LENGTH_TEXT/VID_CHOUT.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
PRINT_INTEGER_HL:
    LD B,FFH

LFF1B:
    SBC A,A
    AND 20H
    LD C,A

; Integer output uses decimal place constants and suppresses leading zeroes before emitting
; digits. This helper is also used for line numbers, so its sign/width behavior affects
; diagnostics as well as PRINT.
    PUSH HL
    LD HL,FF47H

LFF23:
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    EX (SP),HL
    XOR A

LFF29:
    SBC HL,DE
    INC A
    JR NC,LFF29
    ADD HL,DE
    DEC A
    JR Z,LFF39
    LD C,30H
    ADD A,C
    CALL LFE9A
    LD A,C

LFF39:
    XOR C
    CALL NZ,LFE9A
    EX (SP),HL
    DEC E

; PRINT special-character dispatch table.
    JR NZ,LFF23
    POP HL
    LD A,C
    AND B
    JR NZ,LFEC7
    RET

; Binary constants 1000, 100, 10, and 1 for decimal conversion.
    DB E8H, 03H, 64H, 00H, 0AH, 00H, 01H, 00H                                         ; |..d.....|

; The edited BASIC line is accumulated at 1831H, translated to input/token codes, and terminated
; with FFH.

; -----------------------------------------------------------------------------
; READ AND TOKENIZE BASIC LINE
; -----------------------------------------------------------------------------
;
; Invokes the editor to read a command/program line and stores its compact representation in the
; BASIC input buffer.
;
; The routine supplies the editor with the 1831H input buffer, loops for edited characters,
; handles ESC/CTRL+ESC and STOP, translates accepted screen codes into BASIC token/input codes,
; enforces the maximum line length, and terminates the buffer with FFH. The resulting count byte
; and data are ready for the tokenizer and command interpreter.
;
; Entry:
;   Editor and keyboard state; destination buffer at 1831H.
;
; Exit:
;   BASIC input buffer contains length-prefixed tokenizable line; A=00H on success.
;
; Effects:
;   Uses the editor, writes the BASIC input buffer, and may return STOP/error status.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
BASIC_LINE_INPUT:
    LD HL,INPUT_BUFFER
    PUSH HL
    LD B,00H

LFF55:
    PUSH BC
    LD DE,80FFH

; Line input stores editor results in the 1831H buffer, translates accepted character codes,
; bounds the count, and appends FFH. CTRL+ESC and editor end/error status must terminate the
; acquisition before tokenization.
    CALL PRINT_NUMBER_OR_SPECIAL
    POP DE
    LD B,D
    JR Z,LFF72

LFF60:
    POP HL
    LD B,00H
    LD (HL),B
    INC HL
    LD (HL),FFH
    DEC HL
    CP F5H
    JR Z,LFF9B
    CP ECH
    JR Z,LFF9B
    OR A
    RST 10H

LFF72:
    LD A,(STOP_FLAG)
    OR A
    LD A,F5H
    JR NZ,LFF60
    LD A,C
    CP 0DH
    JR Z,LFF92
    CP 20H
    JR C,LFF55
    SUB 80H
    JR C,LFF8C
    CP 20H
    JR NC,LFF8C
    LD C,A

LFF8C:
    LD A,B
    CP FBH
    JR NC,LFF55
    INC B

LFF92:
    INC HL
    LD (HL),C
    JR NZ,LFF55
    LD (HL),FFH
    XOR A
    POP HL
    LD (HL),B

LFF9B:
    OR A

LFF9C:
    RET

; STOP-FLAG at 0B16H is polled by long-running routines; CTRL+ESC clears it and enters STOP
; reporting.

; -----------------------------------------------------------------------------
; STOP AND CTRL-ESC HANDLER
; -----------------------------------------------------------------------------
;
; Checks STOP-FLAG and reports STOP with the current BASIC line context.
;
; A zero STOP-FLAG returns immediately. When CTRL+ESC has set 0B16H, the routine clears the flag,
; restores the interpreter/output state, prints STOP and the current line information, and returns
; to the command loop. The same path is used by long-running numeric, I/O, and tape operations
; that poll for user interruption.
;
; Entry:
;   STOP-FLAG at 0B16H and current BASIC interpreter context.
;
; Exit:
;   Returns when no stop is pending; otherwise resumes BASIC after STOP handling.
;
; Effects:
;   Clears STOP-FLAG and emits STOP/line output.
;
; Destroys:
;   AF, BC, DE, HL, interpreter temporaries.
; -----------------------------------------------------------------------------
CHECK_STOP_FLAG:
    LD A,(STOP_FLAG)
    OR A
    RET Z
    EXX

; STOP routine.

LFFA3:
    EXX

; STOP polling is intentionally cheap: a zero byte returns immediately. When set, it is cleared
; before STOP text and line context are printed, preventing an old CTRL+ESC event from
; retriggering after CONTINUE.

LFFA4:
    XOR A
    LD (STOP_FLAG),A
    CALL LFC35
    CALL LFC18
    CALL PRINT_INLINE_TEXT

; "STOP" message text.
    DB 06H, 0DH, 0AH, "STOP"              ; inline text, 6 bytes
    BIT 2,(IX+00H)
    JR Z,LFFE0
    LD (NEXT_STMT),HL
    LD HL,(CUR_LINE)
    LD (NEXT_LINE),HL
    CALL PRINT_INLINE_TEXT

; " at line " message text.
    DB 09H, " at line "                   ; inline text, 9 bytes
    LD HL,(CUR_LINE)
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    EX DE,HL
    OR A
    CALL PRINT_INTEGER_HL

LFFE0:
    XOR A
    CALL LFE8E
    JP DADAH

LFFE7:
    LD A,H
    CP C0H
    RET C
    RES 6,H
    LD A,50H
    RET

; Extension gateway mapping: EXTH occupies page 3 and SYS is restored in page 0 before JP (HL).
; The extension bridge changes the page map before JP (HL): EXTH is selected in the high extension
; window while SYS remains available in page 0. Callers must treat the target's mapping and return
; convention as part of the ABI.

; -----------------------------------------------------------------------------
; BRIDGE TO EXTENSION ROM
; -----------------------------------------------------------------------------
;
; Switches the memory map and jumps to an EXTH routine addressed by HL.
;
; This fixed gateway saves AF, maps EXTH into page 3 and SYS into page 0 through port 02H,
; restores AF, and performs JP (HL). It is the standard bridge for BASIC and system code that
; needs extension-ROM services while preserving the U0-resident call convention.
;
; Entry:
;   HL = target address in the extension ROM mapping.
;
; Exit:
;   Control transfers to HL with EXTH selected in page 3 and SYS in page 0.
;
; Effects:
;   Changes memory paging; does not return unless the extension target returns through its own
;   convention.
;
; Destroys:
;   Memory mapping and normal return assumptions; AF is restored before the jump.
; -----------------------------------------------------------------------------
CALL_EXTENSION_HL:
    PUSH AF
    LD A,F0H
    LD (P_SAVE),A
    OUT (02H),A
    POP AF
    JP (HL)
    DB 79H, FEH, 0DH, 28H, 13H, FEH                                                   ; |y..(..|
