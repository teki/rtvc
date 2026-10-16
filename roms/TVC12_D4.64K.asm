; -----------------------------------------------------------------------------
; TVC BASIC 1.2 SYS lower ROM - TVC12_D4.64K
; Source: roms/TVC12_D4.64K
; Comments: roms/TVC12_D4.64K.comments.json
; Symbols: roms/TVC12_D4.64K.symbols.json
; ORG: C000H
; Size: 8192 bytes
; Instructions use CPU-visible addresses at ORG; the ROM bank is recorded separately.
; Physical bank: SYS offset 0000H
; CPU-visible aliases: C000H, 0000H
; Auto labels: branch and call targets are emitted as Lxxxx.
; This is a standalone listing; all required technical explanations are embedded here.
; Technical descriptions are based on the Kaszanyiczki and Ludanyi TVC ROM references.
; -----------------------------------------------------------------------------

; =============================================================================
; CPU ADDRESS SPACE AND TVC PAGING
; =============================================================================
; The Z80 sees one 64 KiB address space at 3.125 MHz; every address in an ASM listing is a CPU
; address after the current page mapping has been applied.
; The four 16 KiB CPU pages are 0000H-3FFFH, 4000H-7FFFH, 8000H-BFFFH, and C000H-FFFFH. Port 02
; selects the physical segment visible in each page.
; Physical segments are SYS (system ROM), EXT (extension ROM), VID (video RAM), CART (cartridge
; ROM), and U0-U3 (four RAM pages). A physical segment is not a CPU address: the same CPU range
; can expose SYS or EXT.
; U0, U1, U2, and U3 may occupy only CPU pages 0, 1, 2, and 3 respectively. VID is page 2 only;
; EXT is page 3 only. SYS and CART can be mapped in page 0 or page 3.
; The normal 64K arrangement exposes SYS at C000H-FFFFH, U0 at 0000H-3FFFH, U1 at 4000H-7FFFH, U2
; at 8000H-BFFFH, and U3 at C000H-FFFFH when the corresponding page is selected.
; The lower half of the EXT page can instead expose expansion-card I/O memory. Port 03 bits 7-6
; select which card IOMEM is visible there; preserve the keyboard-row bits in the same register.
; A ROM routine that calls or returns through a RAM bridge must treat the page register as part of
; its calling convention. Do not infer a SYS/EXT identity from a CPU address alone.
; The U0 page is the fixed RAM workspace: RST bridges, I/O assignment table, system variables,
; editor buffers, and the Z80 stack live there before BASIC's high-memory allocations.
; Physical offsets in standalone annotations identify the ROM bank and offset. The CPU address is
; a view established by paging and is deliberately not the identity of a SYS or EXT annotation.

; =============================================================================
; TVC HARDWARE PORT CONTRACTS
; =============================================================================
; Port 00 is the border latch. Port 01 is the Centronics printer data register. Port 02 is the
; four-page memory mapper.
; Port 03 combines expansion-card selection in bits 7-6 with keyboard matrix row selection in bits
; 3-0. Port 04 is the low byte of the 12-bit sound/serial divider; port 05 contains its high
; nibble and enable bits.
; Port 05: bits 3-0 are PITCH high nibble, bit 4 enables sound, bit 5 enables sound IRQ, bit 6
; controls the left cassette motor, and bit 7 controls the right cassette motor.
; Port 06: bits 6-2 select sound volume, bit 7 produces the printer STROBE pulse, and bits 1-0
; select the colour mode (00=2 colour, 01=4 colour, 10 or 11=16 colour).
; Ports 10/11, 20/21, 30/31, and 40/41 are the four serial-card USART data and command/status
; pairs. The slot's card IOMEM must be selected before accessing its pair.
; Port 50 drives the cassette write signal by toggling its output. Port 58 reads the selected
; keyboard row. Port 59 reads keyboard/cassette/printer status: bit 5 is cassette input, bit 6 is
; the colour switch, and bit 7 is printer ACK.
; Ports 5A and 5B provide the joystick/light-pen or analogue input interface used by the system
; I/O routines; their exact interpretation depends on the selected input operation.
; Ports 60-63 are palette entries. Port 70 selects a 6845 CRT-controller register and port 71
; reads or writes that register.
; The ROM mirrors writable port state in U0 variables 0B11H-0B13H. Update a mirror and use
; read-modify-write when changing one field so keyboard selection, motors, sound, printer, and
; colour bits do not get lost.
; I/O reads and writes may be routed through the RST30 class dispatcher; a routine that reaches
; hardware directly still has to preserve the mapper and any port mirror required by its caller.

; =============================================================================
; RAM-RESIDENT CALL BRIDGES
; =============================================================================
; U0 0B00H-0B0FH is the device assignment table. Input selectors occupy 0B00H-0B07H and output
; selectors 0B08H-0B0FH; each byte names a device class and its selected slot.
; The default input classes are video, keyboard, editor, sound, printer, cassette, cards, and
; connector select at 0B00H-0B07H. The output table at 0B08H-0B0FH follows the same order.
; The RST30 function byte is encoded as direction in bit 7, device class in bits 6-4, and routine
; number in bits 3-0. Bit 7 set selects the input assignment (0B00H-0B07H); clear selects output.
; For each class, routine 0 is the interrupt service, routine 1 is character I/O, and routine 2 is
; block I/O unless the class documents a narrower set. The dispatcher supplies the selected device
; context.
; RST30 is bridged through RAM because a RST instruction cannot itself select the desired ROM
; page. The bridge reads the byte after the RST, saves the current page at 0003H, maps the
; selected SYS/U0/U1/U2 page, and enters the ROM dispatcher.
; The normal bridge entry is U0 0B23H. It consumes the post-RST function byte and enters the
; common SYS dispatch path (the SYS-side implementation is reached at the documented C363H entry).
; The bridge return at U0 0B37H restores the saved page and AF before returning to the interrupted
; caller. Code invoking RST30 must leave the inline function byte immediately after the
; instruction.
; The interrupt bridge at U0 0B41H performs the corresponding page restore, restores AF, enables
; interrupts, and returns. Interrupt handlers must not bypass this tail unless they restore the
; same state themselves.
; RST18 is BASIC's token/function dispatch bridge: BASIC maintains the next RST18 code pointer in
; its workspace and uses the tokenized stream to select the implementation routine.
; Because these bridges are RAM code, their bytes and labels belong to the U0 physical workspace
; in a listing even though callers often see them as fixed CPU addresses.

; =============================================================================
; SYSTEM VARIABLES AND INTERRUPT STATE
; =============================================================================
; 0B10H INT-DES is the active-low interrupt-source mask: bit 0 video/cursor, bit 1 keyboard, bit 2
; editor, bit 3 sound, and bits 4-7 expansion cards 3 through 0. A zero bit enables that source.
; 0B11H mirrors port 03. Bits 7-6 select the expansion card IOMEM and bits 3-0 select the keyboard
; matrix row; all unrelated bits must be preserved when either field changes.
; 0B12H mirrors port 05. Bits 7-6 are the cassette motor controls, bit 5 sound IRQ enable, bit 4
; sound enable, and bits 3-0 the divider high nibble.
; 0B13H mirrors port 06. Bit 7 is printer STROBE, bit 6 is hardware-dependent, bits 5-2 are sound
; volume, and bits 1-0 are the 2/4/16-colour mode selector.
; 0B14H SOUND-ACT is FF while a timed tone is active; 0B15H TONE-REPLACE is FF when a new tone
; should replace an existing one instead of waiting for it to finish.
; 0B16H STOP-FLAG becomes FF on CTRL+ESC and is polled by long-running BASIC, editor, graphics,
; and file operations to provide a cooperative break.
; 0B17H-0B18H hold the minimum stack address reserved by paint/fill algorithms (normally 0F10H).
; U0's CPU stack starts near 0EACH and leaves roughly 100 bytes for nested calls.
; 0B19H-0B1AH HI-MEM is the highest usable RAM address, normally BFFFH on a 64K machine; BASIC's
; downward-growing stack and allocations must not cross it.
; 0B1BH U3-STAT is zero when U3 RAM is good and FF when it failed the memory test. 0B1CH records
; the assigned serial-card base slot or FF for no assignment.
; 0B1DH-0B1EH TIME is a two-byte software clock incremented by the periodic interrupt. 0B1FH
; IRQ-STAT enables cursor/video (bit 0), sound (bit 1), and card 0-3 (bits 2-5).
; 0B20H INT-FLAG is FF while the interrupt routine owns the shared state. 0B21H WARM-FLAG is FF
; during a warm reset; 0B22H COLD-FLAG requests a cold reset and is consumed by initialization.

; =============================================================================
; VIDEO WORKSPACE AND DRAWING STATE
; =============================================================================
; 0B49H-0B4AH is a temporary saved SP used by graphics routines. 0B4BH L-MODE selects overwrite
; (0), OR (1), AND (2), or XOR (3) raster composition.
; 0B4CH L-STYLE selects the line style; 0B4DH INK and 0B4EH PAPER hold logical colours; 0B4FH
; BORDER packs intensity in bit 7, green in bit 5, red in bit 3, and blue in bit 1.
; 0B50H V-FLAG selects character-cell overwrite behaviour: 0 replaces fully, 1 preserves old
; pixels where the new character is background, 2 draws inverse, and 3 leaves the cell unchanged.
; 0B73H is the current colour mode (00=2 colour, 01=4 colour, 02=16 colour). 0B74H is the pen
; state (FF down, 00 up) and 0B75H is the fill byte used by area operations.
; 0B76H-0B77H hold the current video-RAM address. Logical X at 0B78H-0B79H and logical Y at
; 0B7AH-0B7BH are transformed to physical X at 0B7CH-0B7DH and physical Y at 0B7EH-0B7FH.
; The horizontal coordinate is mode scaled: physical pixels per logical unit are 2, 4, or 8 for
; 2-, 4-, and 16-colour modes. The physical raster is 1024 by 960 pixels; logical Y is
; quarter-height.
; 0B83H is the line-pattern byte. The drawing code interprets it as one of the available
; dashed/solid styles and advances the pattern as pixels are emitted.
; The 6845 registers at ports 70H/71H determine display timing and the visible text/graphics base.
; Palette writes use 60H-63H; changing mode also requires the port-06 mirror to agree with the
; hardware.
; Editor row descriptors and cursor state use the mode-derived row length and character width;
; graphics routines should not assume a fixed 40-column text stride.
; Video routines commonly save and restore the mapper around VID access. A pointer is meaningful
; only while page 2 exposes VID and the corresponding video address registers/work variables are
; current.

; =============================================================================
; KEYBOARD WORKSPACE AND MATRIX SCAN
; =============================================================================
; The keyboard is a 10-row by 8-column matrix. Select a row by writing the low nibble of port 03
; and read the active-low column bits from port 58.
; 0B51H-0B5AH PICTURE stores the current ten-row matrix image; 0B5BH-0B64H OLD-PIC stores the
; previous image. Difference scanning identifies newly pressed and released keys.
; 0B65H DELAY-KEY is the initial key-repeat delay in 20 ms units (default 1EH, approximately 0.6
; s). 0B67H RATE-KEY is the repeat period (default 03H, approximately 60 ms).
; 0B66H LOCK-KEY records the modifier lock state: CTRL bit 0, SHIFT bit 1, and ALT bit 3. 0B68H
; HOLD-KEY is FF when CTRL+P hold processing is disabled.
; 0BE5H is the pending-key marker (00 none, FF a translated key is waiting at 0BE9H). 0BE7H
; records lock activity, 0BE8H the current modifier (00/02/04/08 for shift/ctrl/alt), and 0BE9H
; the key code.
; 0BEAH and 0BEBH are repeat-delay and repeat-rate counters. 0BECH-0BEDH identify the differing
; matrix address and 0BEEH contains the single differing bit mask.
; A scan must preserve the selected expansion-card bits in port 03 while changing the keyboard
; row. The interrupt path acknowledges a key by copying the new matrix into OLD-PIC after deciding
; whether it is a press, release, or repeat.
; The translated key code is consumed by editor/BASIC input through the keyboard device class, not
; by reading port 58 directly. CTRL+ESC sets STOP-FLAG for cooperative break.
; Keyboard and cursor interrupts share the INT-DES and IRQ-STAT gating variables; code that polls
; the matrix while interrupts are active must account for a concurrently updated PICTURE image.

; =============================================================================
; CASSETTE AND FILE WORKSPACE
; =============================================================================
; 0BF0H saves the border while cassette I/O is active. 0BF1H VERIFY is nonzero for
; compare-with-memory mode. 0BF2H saves the first byte of the interrupt vector while tape timing
; temporarily installs its own handler.
; 0BF3H records an open-read file: 00 none, 01 buffered, 03 unbuffered. 0BF4H-0C04H is the
; requested filename (length byte followed by up to 16 characters); 0C05H-0C15H is the name read
; from tape.
; 0C16H-0D04H is the input buffer. 0D05H-0D06H counts bytes read, 0D07H-0D08H points to the next
; input byte, and 0D09H-0D0AH counts bytes remaining.
; 0D0BH is the input error; 0D0CH is protection; 0D0DH is the sector number; 0D0EH is the
; sector-end marker (00 intermediate, FF final); 0D0FH distinguishes header (FF) from data (00).
; 0D10H-0D11H retain the first destination address. 0D13H is the read phase (FF opening a file, 00
; continuing an existing file).
; 0D14H is the output file state (00 none), 0D15H-0D25H the output filename, and 0D26H-0E25H the
; output buffer. 0E26H-0E27H counts bytes to store and 0E28H-0E29H points at the next output
; address.
; 0E2AH is output error; 0E2CH-0E2DH is source start; 0E2EH is current output character; 0E2FH is
; output type (01 buffered, 03 unbuffered); 0E30H is protection; 0E32H is write phase (FF header,
; 00 data).
; A cassette sector carries a 256-byte data payload plus framing, type, sequence, protection, and
; CRC information. MUDDLE at 0B6FH-0B70H is the CRC seed and can act as a file protection
; password.
; 0B6BH BUFFER selects unbuffered (00) or buffered (FF) file handling. 0B6CH REMRED selects
; motor/head routing: 00 left read/right write, 40 right read/write, 80 left read/write, C0 left
; write/right read.
; 0B6DH PROTECT is nonzero when writes are inhibited; 0B6EH EOF becomes nonzero after the final
; byte; 0B71H SER-OK is zero when the divider is synchronized for serial and FF after sound/tape
; invalidates that timing.
; Tape input uses port 59 bit 5 and writes by toggling port 50. Motor controls are port 05 bits
; 6-7; tape timing shares the periodic interrupt and must restore the saved vector, border,
; divider, and mapper on exit.

; =============================================================================
; EDITOR WORKSPACE AND CPU STACK
; =============================================================================
; 0E48H is the cursor blink counter; 0E49H is cursor Y and 0E4AH cursor X. 0E4BH-0E4CH points into
; the ASCII line buffer, while 0E4DH records the cursor/line position state.
; 0E4EH-0E4FH saves the prior cursor position. 0E50H-0E67H contains 24 row descriptors used to
; translate editor rows into video addresses and wrap rules.
; 0E68H-0E6AH is a mode-dependent jump or dispatch value. 0E6BH is row length (40, 20, or 10);
; 0E6CH is character width (1, 2, or 4) in the selected colour mode.
; 0E6DH-0E94H stores the saved cursor glyph/attributes (40 bytes). 0E95H and 0E96H are the ink and
; paper lines used when restoring the cursor cell.
; 0EACH-16ABH is the CPU stack area. The stack grows downward from its high end; paint/fill
; routines use 0B17H as a lower safety limit so a large recursive operation cannot overwrite
; workspace.
; 0C16H-0D04H and 0D26H-0E25H are also editor/file buffers. Routines must preserve the ownership
; convention: the editor may reuse a buffer only when no cassette operation has it open.
; Editor input is device-class routed. The keyboard interrupt writes a translated code to 0BE9H
; and the editor consumes it, updating cursor coordinates, row descriptors, and video RAM through
; the active mode.
; When switching video modes, recompute row length and character width and rebuild row
; descriptors; retaining a 40-column descriptor table in a 16-colour mode produces incorrect
; cursor and wrap addresses.

; =============================================================================
; BASIC WORKSPACE AND TOKENIZED PROGRAM FORMAT
; =============================================================================
; BASIC work variables occupy 1700H-19EFH. 1700H flags TRACE (bit 0), suppress-OK (bit 1), running
; (bit 2), and file-open (bit 3).
; 1701H pending-value type is 01 string or 03 number; 1702H is the conditional-execution flag;
; 1704H holds the byte after RST18; 1705H is function class; 1706H is the selected device number.
; 1707H AUTORUN is FF when an autorun request is pending. 1708H is symbol type; 1709H-170BH hold
; RND state. 170CH current-line, 170EH next-line, and 1710H next-statement pointers walk the
; tokenized program.
; 1712H-1714H are DATA line and byte pointers; 1716H is the INPUT data pointer; 1718H points to
; the next RST18 code; 171AH snapshots the BASIC stack pointer.
; 1720H VLOMEM is the low-memory/base boundary; 1722H is program/TEXT start; 1724H is the end of
; the chained symbol area; 1726H TOP is the next free symbol byte; 172AH holds the current sound
; PITCH.
; 1732H-1830H is the command buffer. 1831H-192FH is the INPUT buffer. 19C0H-19C6H and 19C7H-19D0H
; are floating-point X and Y registers.
; 19CEH-19DEH holds the current filename; 19DFH-19EEH is the cassette header workspace. 19EFH is
; reserved by the ROM, so user/free U0 allocation starts after it.
; A stored BASIC line begins with its byte length, then a two-byte binary line number, then
; tokenized text, and an FF line terminator. A 00 length marks the end of the program.
; The BASIC evaluation stack grows downward from HI-MEM. Each element starts with a type marker,
; allowing numeric and string values to share the stack while variable-length payloads are
; addressed relative to that marker.
; The tokenizer and statement handlers exchange pointers rather than source strings: current byte,
; next statement, DATA, INPUT, and RST18 pointers must remain consistent when a handler skips or
; consumes a clause.

; =============================================================================
; DEVICE CLASSES AND RST30 DISPATCH SEMANTICS
; =============================================================================
; The eight device classes are video, keyboard, editor, sound, printer, cassette, cards, and
; connector/slot selection. Their input selector bytes are U0 0B00H-0B07H and output selector
; bytes U0 0B08H-0B0FH.
; The documented default selector values are input FF, 01, 02, FF, FF, 05, 06, and a
; connector-select value; output FF, FF, 02, FF, 04, 05, 06, and FF or a serial slot.
; The function byte after RST30 encodes class and direction, so BASIC and editor code can use the
; same bridge for console, printer, tape, sound, and card operations without knowing the concrete
; device routine address.
; Routine number 0 is the class interrupt hook, 1 is character transfer, and 2 is block transfer.
; A block call commonly receives a pointer/count pair in registers or workspace and returns a
; count or error code according to the class.
; Character input returns the translated character or a no-data indication; character output
; consumes the character and may block until the selected device accepts it. Callers should test
; the documented carry/error convention before advancing a BASIC pointer.
; Card dispatch first selects the card through port 03 bits 7-6 and then accesses the card's USART
; or IOMEM. Connector selection is separate from the logical cards class because it controls which
; physical slot is presented in the EXT lower half.
; The dispatcher may change the current page and shared port mirrors. RST30 callers therefore
; return through the bridge and must not assume a page, AF, or device-selection register survives
; a failed operation.
; Interrupt entry and device calls share INT-DES, IRQ-STAT, INT-FLAG, and the 0B41H tail. A
; routine that masks a source should restore the prior active-low mask rather than blindly
; enabling all devices.

; =============================================================================
; SOUND AND SERIAL CLOCK CONTRACT
; =============================================================================
; The 12-bit PITCH divider is written as high nibble in port 05 bits 3-0 and low byte in port 04.
; The usable divisor is 4096 minus PITCH; zero divisor is not a valid operating point.
; The divider output is 3125/(4096-PITCH) kHz. For serial cards the clock is 1562500/(4096-PITCH)
; Hz, and the tone/IRQ output is 195312.5/(4096-PITCH) Hz.
; Port 05 bit 4 enables the sound output and bit 5 enables sound interrupts. Port 06 bits 6-2
; select volume. The ROM stores the same fields in 0B12H and 0B13H.
; 0B14H SOUND-ACT and 0BEFH duration counter let the periodic interrupt terminate a timed tone.
; 0B15H TONE-REPLACE controls whether a new tone takes over immediately.
; 0B69H BAUD encodes 110, 150, 300, 600, 1200, 2400, 4800, 9600, and 19200 baud as values 00H
; through 08H.
; 0B6AH FORMAT defaults to EEH: two stop bits, no parity, eight data bits, and a 16-times clock.
; Serial initialization translates BAUD and FORMAT into the selected USART command/mode registers.
; Sound or cassette activity changes the shared divider. 0B71H SER-OK is cleared only when the
; divider remains synchronized for serial; serial code must reprogram or reject a transfer after
; an invalidating device operation.
; A sound routine should preserve the port-05 high-nibble and motor bits while changing PITCH or
; sound enables. Direct port writes that omit the mirror can silently stop tape motors or leave
; the interrupt mask inconsistent.

; =============================================================================
; CASSETTE SIGNAL, FRAMING, AND CRC CONTRACT
; =============================================================================
; Cassette input is sampled at port 59 bit 5; output is generated by toggling port 50. The left
; and right motor controls are port 05 bits 6 and 7 and are routed by REMRED at 0B6CH.
; The tape front-end records a header block followed by one or more data blocks. A block
; identifies its type, sector number, end marker, protection value, payload length, and CRC; the
; final block has an FF end marker.
; MUDDLE at 0B6FH-0B70H seeds the checksum/CRC transformation. Matching the seed is part of the
; protection check, so VERIFY and protected loads must not overwrite it before the header is
; accepted.
; VERIFY at 0BF1H selects compare mode: the reader decodes tape data into its buffer and compares
; it with the destination instead of storing it. EOF at 0B6EH is set only after the final block
; has been consumed.
; Buffered mode accumulates a sector in 0C16H-0D04H or 0D26H-0E25H and updates the next-byte
; pointer/count fields. Unbuffered mode streams bytes while still maintaining the header,
; protection, and error state.
; The tape interrupt temporarily replaces the vector byte at 0038H and saves its original first
; byte at 0BF2H. Every exit path, including checksum or motor errors, must restore that byte and
; the previous interrupt/page state.
; Input and output error bytes at 0D0BH and 0E2AH are sticky for the current operation. A caller
; should inspect them before closing the file or advancing the BASIC program pointer.
; The border save at 0BF0H is part of the user-visible tape contract: cassette routines may change
; border colour while synchronizing and restore it after motors stop.

; =============================================================================
; CALLER-SAFE STATE AND ERROR HANDLING
; =============================================================================
; ROM entry points generally preserve the mapper only through the RAM bridge or their documented
; return path. A direct SYS/EXT call must establish the expected page and restore it before
; returning to BASIC or the editor.
; Shared state includes port mirrors, interrupt masks, tape buffers, cursor/video variables, and
; BASIC pointers. Treat these as live workspaces rather than constants: interrupts can update them
; between any two instructions unless the source is masked.
; STOP-FLAG is the common cooperative cancellation mechanism. Long loops should poll it at a safe
; point, restore hardware state, and return the class-specific error rather than jumping directly
; to warm reset.
; Cassette and serial errors are represented in workspace bytes and often leave the carry/error
; result set at the API boundary. Callers must not assume a failed block consumed the requested
; count.
; BASIC handlers must update current-line/current-statement and DATA/INPUT pointers only after
; successful parsing or transfer. On syntax, type, device, or tape error, preserve enough state
; for the error reporter to identify the current statement.
; The stack, U0 buffers, and HI-MEM boundary are shared by graphics and BASIC. Before reserving a
; temporary buffer, compare its end with HI-MEM and the paint stack floor at 0B17H-0B18H.
; Use the ROM image as byte authority when a prose map and a disassembly label disagree. The
; physical bank/offset is authoritative for annotations; CPU addresses and book names are
; explanatory views.

; =============================================================================
; SYMBOLS USED BY THIS LISTING
; =============================================================================
; U0 RAM cells and system variables referenced below. Names follow the workspace notes
; above; addresses whose meaning is not documented stay numeric in the code.
P_SAVE             EQU 0003H              ; saved port-02 page map (written by every page switch)
USRTAB             EQU 0021H              ; seven USR/EXT subroutine addresses
IO_ASSIGN_IN       EQU 0B00H              ; input device assignment bytes, one per class
IO_ASSIGN_OUT      EQU 0B08H              ; output device assignment bytes, one per class
INT_DES            EQU 0B10H              ; active-low interrupt source mask
PORT03_MIRROR      EQU 0B11H              ; copy of port 03: card select (7-6), keyboard row (3-0)
PORT05_MIRROR      EQU 0B12H              ; copy of port 05: motors, sound enables, pitch high nibble
PORT06_MIRROR      EQU 0B13H              ; copy of port 06: STROBE, volume, colour mode
SOUND_ACT          EQU 0B14H              ; FFH while a timed tone is playing
TONE_REPLACE       EQU 0B15H              ; FFH: a new tone replaces the current one
STOP_FLAG          EQU 0B16H              ; FFH after CTRL+ESC (cooperative break)
PAINT_STACK_MIN    EQU 0B17H              ; lowest stack address PAINT may use
HI_MEM             EQU 0B19H              ; highest usable RAM address
TIME               EQU 0B1DH              ; software clock incremented by the interrupt
IRQ_STAT           EQU 0B1FH              ; interrupt service enables (cursor, sound, cards)
INT_FLAG           EQU 0B20H              ; FFH while the interrupt routine owns shared state
WARM_FLAG          EQU 0B21H              ; FFH during a warm reset
COLD_FLAG          EQU 0B22H              ; cold-reset request, consumed by initialization
RST30_BRIDGE_RET   EQU 0B37H              ; U0 RST 30H bridge return (restores page and AF)
IRQ_BRIDGE_TAIL    EQU 0B41H              ; U0 interrupt return tail (page, AF, EI, RET)
L_MODE             EQU 0B4BH              ; pixel combine mode: 0 put, 1 OR, 2 AND, 3 XOR
L_STYLE            EQU 0B4CH              ; line style index
INK                EQU 0B4DH              ; logical ink colour
PAPER              EQU 0B4EH              ; logical paper colour
BORDER             EQU 0B4FH              ; border colour (I.G.R.B in bits 7,5,3,1)
V_FLAG             EQU 0B50H              ; bit 0: skip ink pixels, bit 1: skip paper pixels
KB_PICTURE         EQU 0B51H              ; current keyboard matrix image
KB_OLD_PIC         EQU 0B5BH              ; previous keyboard matrix image
DELAY_KEY          EQU 0B65H              ; key repeat delay, 20 ms units
LOCK_KEY           EQU 0B66H              ; modifier lock state
RATE_KEY           EQU 0B67H              ; key repeat period, 20 ms units
HOLD_KEY           EQU 0B68H              ; FFH: CTRL+P hold disabled
SER_OK             EQU 0B71H              ; 00H while the divider is set up for serial
COLOR_MODE         EQU 0B73H              ; current mode: 00H 2, 01H 4, 02H 16 colours
B_STAT             EQU 0B74H              ; pen state: FFH down, 00H up
PIXEL_MASK         EQU 0B75H              ; mask of the beam pixel in its video byte
VID_ADDR           EQU 0B76H              ; current video RAM address
X_LOGICAL          EQU 0B78H              ; logical beam X
Y_LOGICAL          EQU 0B7AH              ; logical beam Y
X_PHYSICAL         EQU 0B7CH              ; physical beam X
Y_PHYSICAL         EQU 0B7EH              ; physical beam Y
CHAR_PIXEL_W       EQU 0B82H              ; physical units per pixel: 2, 4 or 8
LINE_PATTERN       EQU 0B83H              ; rotating line-style bit pattern
LINE_STEP_1        EQU 0B84H              ; line stepper entry used when the error term is negative
LINE_STEP_2        EQU 0B86H              ; line stepper entry used when the error term is positive
PAINT_SP_SAVE      EQU 0B88H              ; SP saved on PAINT entry
PAINT_TARGET       EQU 0B8AH              ; PAINT: colour being replaced, as a byte pattern
PAINT_SPAN         EQU 0B8BH              ; PAINT: C = mask, B = bytes to the right edge
PAINT_ADDR         EQU 0B8DH              ; PAINT: video address being scanned
PAINT_WIDTH        EQU 0B8FH              ; PAINT: parent run pixels left to scan
PAINT_RECORD       EQU 0B91H              ; PAINT: width word of the run being scanned
PAPER_PATTERN      EQU 0B93H              ; PAPER as a byte pattern
INK_PATTERN        EQU 0B94H              ; INK as a byte pattern (PAINT: INK XOR target)
KEY_PENDING        EQU 0BE5H              ; FFH: a translated key waits in KEY_CODE
HOLD_STATE         EQU 0BE6H              ; FFH while CTRL+P hold is active
KEY_LOCK_ACT       EQU 0BE7H              ; FFH while LOCK is held after setting a lock mode
KEY_MODIFIER       EQU 0BE8H              ; current modifier 00/02/04/08H
KEY_CODE           EQU 0BE9H              ; translated key code
KEY_DELAY_CNT      EQU 0BEAH              ; repeat delay counter
KEY_RATE_CNT       EQU 0BEBH              ; repeat rate counter
KEY_DIFF_ADDR      EQU 0BECH              ; matrix address of the changed key
KEY_DIFF_MASK      EQU 0BEEH              ; single changed matrix bit
TONE_DURATION      EQU 0BEFH              ; remaining timed-tone duration
CAS_OUT_NAME       EQU 0D15H              ; output file name
CU_BLINK           EQU 0E48H              ; cursor blink counter
CU_Y               EQU 0E49H              ; cursor row
CU_X               EQU 0E4AH              ; cursor column
ED_LINE_PTR        EQU 0E4BH              ; pointer into the ASCII line buffer
CU_STATE           EQU 0E4DH              ; cursor/line position state
CU_SAVED_POS       EQU 0E4EH              ; saved cursor position
ED_ROW_DESC        EQU 0E50H              ; editor row descriptors
ED_MODE_VECTOR     EQU 0E68H              ; mode-dependent jump/dispatch value
ED_ROW_LEN         EQU 0E6BH              ; characters per row: 40, 20 or 10
ED_CHAR_WIDTH      EQU 0E6CH              ; bytes per character cell: 1, 2 or 4
CU_SAVE_AREA       EQU 0E6DH              ; saved glyph/attributes under the cursor
CU_INK_LINE        EQU 0E95H              ; ink line used when restoring the cursor cell
CU_PAPER_LINE      EQU 0E96H              ; paper line used when restoring the cursor cell
STACK_TOP          EQU 16ACH              ; initial Z80 stack pointer (stack grows down)
BASIC_FLAGS        EQU 1700H              ; TRACE, suppress-OK, running, file-open bits
FUNC_CLASS         EQU 1705H              ; function class
DEVICE_NUM         EQU 1706H              ; selected device number
RND_STATE          EQU 1709H              ; RND generator state
CUR_LINE           EQU 170CH              ; current program line pointer
NEXT_LINE          EQU 170EH              ; next program line pointer
NEXT_STMT          EQU 1710H              ; next statement pointer
DATA_LINE          EQU 1712H              ; DATA line pointer
DATA_PTR           EQU 1714H              ; DATA byte pointer
BASIC_SP           EQU 171AH              ; BASIC stack pointer snapshot
VLOMEM             EQU 1720H              ; low-memory/base boundary
TEXT_START         EQU 1722H              ; start of the BASIC program
SYMBOL_END         EQU 1724H              ; end of the chained symbol area
SYMBOL_TOP         EQU 1726H              ; next free symbol byte
CMD_BUFFER         EQU 1732H              ; command line buffer
INPUT_BUFFER       EQU 1831H              ; INPUT buffer

; SYS ROM routines and tables defined in TVC12_D3.64K.asm; both halves of the SYS bank are
; visible together whenever SYS is mapped.
LE002                          EQU E002H
LE053                          EQU E053H
LE104                          EQU E104H
LE10E                          EQU E10EH
LE117                          EQU E117H
BASIC_FOR                      EQU E15CH
BASIC_INPUT                    EQU E1CBH
LE21B                          EQU E21BH
BASIC_IF                       EQU E2EEH
LE332                          EQU E332H
BASIC_GOSUB                    EQU E382H
BASIC_GOTO                     EQU E3B2H
BASIC_LET                      EQU E3C1H
LE452                          EQU E452H
BASIC_NEXT                     EQU E4B6H
BASIC_OUT                      EQU E542H
BASIC_POKE                     EQU E553H
LE570                          EQU E570H
BASIC_PRINT                    EQU E573H
LE6C7                          EQU E6C7H
LE6F4                          EQU E6F4H
BASIC_RETURN                   EQU E70FH
BASIC_GRAPHICS                 EQU E733H
BASIC_PLOT                     EQU E754H
BASIC_SET                      EQU E790H
BASIC_SOUND                    EQU E833H
LE89B                          EQU E89BH
BASIC_OPEN                     EQU E8C9H
LE910                          EQU E910H
BASIC_LOAD                     EQU E951H
BASIC_SAVE                     EQU E982H
LE9D3                          EQU E9D3H
EVAL_NUMERIC_ARGUMENT          EQU EA68H
LEA82                          EQU EA82H
LEA9A                          EQU EA9AH
LEA9F                          EQU EA9FH
LEABE                          EQU EABEH
LEAC3                          EQU EAC3H
LEACD                          EQU EACDH
LEAD2                          EQU EAD2H
BASIC_FREE                     EQU EC4CH
FN_REC_VERNUM                  EQU F094H
FP_SUB                         EQU F48EH
FP_ADD                         EQU F493H
FP_MUL                         EQU F512H
FP_DIV                         EQU F5FBH
FP_NEGATE                      EQU F726H
ASCII_TO_FP                    EQU F914H
DISCARD_NUMBER                 EQU FA1BH
LFA92                          EQU FA92H
STACK_NUMBER_TO_INTEGER        EQU FAC3H
LFB28                          EQU FB28H
FOLD_CASE_DE                   EQU FBBFH
POP_LINE_NUMBER_FIND           EQU FBDEH
CHANNEL_ONLY                   EQU FBECH
CHANNEL_ONLY_C                 EQU FBEEH
MESSAGE_COLOURS                EQU FC18H
CLOSE_TAPE_FILES               EQU FC3EH
GET_TOKEN                      EQU FC43H
CHECK_MEMORY_256               EQU FC8EH
ERROR_NOT_UNDERSTOOD           EQU FD5AH
PRINT_INLINE_TEXT              EQU FE79H
PRINT_NEWLINE_CLR              EQU FE8EH
PRINT_CRLF                     EQU FE93H
PRINT_CHAR                     EQU FE9AH
PRINT_NUMBER_TEXT              EQU FEC1H
PRINT_SPACE                    EQU FEC7H
LIST_TEXT                      EQU FEDDH
PRINT_INTEGER_HL               EQU FF19H
PRINT_INTEGER_B                EQU FF1BH
BASIC_LINE_INPUT               EQU FF4FH
CHECK_STOP_FLAG                EQU FF9DH
LFFA3                          EQU FFA3H
STOP_FROM_ERROR                EQU FFA3H
CALL_EXTENSION_HL              EQU FFF0H

ORG C000H, SYS0, 0000H


; -----------------------------------------------------------------------------
; RESET VECTOR
; -----------------------------------------------------------------------------
;
; Transfers control from the Z80 reset address to the BASIC system initializer.
;
; At power-on the SYS ROM is mapped into page 0, so the processor fetches this instruction at CPU
; address 0000H. The same physical byte normally appears at C000H when SYS occupies page 3.
;
; The target is written as BASIC_COLD_START@SYS0 because execution is still using the
; page-0 SYS mapping. It is the page-0 alias of BASIC_COLD_START at C229H.
;
; Entry:
;   PC = 0000H after reset, with SYS mapped into page 0.
;
; Exit:
;   Control transfers to BASIC_COLD_START through its 0229H alias.
;
; Effects:
;   Begins machine initialization.
; -----------------------------------------------------------------------------
RESET_VECTOR:
    JP BASIC_COLD_START@SYS0

; -----------------------------------------------------------------------------
; BCD DIGIT MULTIPLICATION TABLE
; -----------------------------------------------------------------------------
;
; BCD products for decimal digits 0 through 9.
;
; This is a 10 by 10 multiplication table. Rows select one decimal digit and columns select the
; other. Each result is stored as packed BCD rather than ordinary binary, so the product of 7 and
; 8 appears as 56H.
; -----------------------------------------------------------------------------
BCD_MULTIPLICATION_TABLE:
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 01H, 02H, 03H, 04H, 05H ; |................|
    DB 06H, 07H, 08H, 09H, 00H, 02H, 04H, 06H, 08H, 10H, 12H, 14H, 16H, 18H, 00H, 03H ; |................|
    DB 06H, 09H, 12H, 15H, 18H, 21H, 24H, 27H, 00H, 04H, 08H, 12H, 16H, 20H, 24H, 28H ; |.....!$'..... $(|
    DB 32H, 36H, 00H, 05H, 10H, 15H, 20H, 25H, 30H, 35H, 40H, 45H, 00H, 06H, 12H, 18H ; |26.... %05@E....|
    DB 24H, 30H, 36H, 42H, 48H, 54H, 00H, 07H, 14H, 21H, 28H, 35H, 42H, 49H, 56H, 63H ; |$06BHT...!(5BIVc|
    DB 00H, 08H, 16H, 24H, 32H, 40H, 48H, 56H, 64H, 72H, 00H, 09H, 18H, 27H, 36H, 45H ; |...$2@HVdr...'6E|
    DB 54H, 63H, 72H, 81H                                                             ; |Tcr.|

; -----------------------------------------------------------------------------
; BASIC STATEMENT DISPATCH TABLE
; -----------------------------------------------------------------------------
;
; Jump table for primary BASIC statement tokens.
;
; The table contains little-endian handler addresses for BASIC statement tokens from FFH down to
; D0H. The entries cover line termination and REM, followed by DATA, CLOSE, CLS, CONTINUE, DEF,
; DELETE, DIM, ELSE, END, FOR, GET, GOSUB, GOTO, GRAPHICS, IF, INPUT, LET, LIST, LLIST, LOAD,
; LOMEM, NEW, NEXT, OK, ON, OPEN, OUTPUT, OUT, PLOT, POKE, PRINT, RANDOMIZE, READ, RESTORE,
; RETURN, RUN, SAVE, SET, SOUND, STOP, TRACE, VERIFY, EXT, and LPRINT.
; -----------------------------------------------------------------------------
BASIC_STATEMENT_JUMP_TABLE:
    DW BASIC_REM_ENTRY                    ; FFH (no keyword)
    DW BASIC_REM_ENTRY                    ; FEH !
    DW BASIC_NEXT_STATEMENT               ; FDH :
    DW BASIC_REM_ENTRY                    ; FCH REM
    DW BASIC_DATA_ENTRY                   ; FBH DATA
    DW LE89B                              ; FAH CLOSE
    DW BASIC_CLS                          ; F9H CLS
    DW BASIC_CONTINUE                     ; F8H CONTINUE
    DW LE002                              ; F7H DEF
    DW LDD93                              ; F6H DELETE
    DW LE053                              ; F5H DIM
    DW LE104                              ; F4H ELSE
    DW LE10E                              ; F3H END
    DW BASIC_FOR                          ; F2H FOR
    DW LE910                              ; F1H GET
    DW BASIC_GOSUB                        ; F0H GOSUB
    DW BASIC_GOTO                         ; EFH GOTO
    DW BASIC_GRAPHICS                     ; EEH GRAPHICS
    DW BASIC_IF                           ; EDH IF
    DW BASIC_INPUT                        ; ECH INPUT
    DW BASIC_LET                          ; EBH LET
    DW BASIC_LIST                         ; EAH LIST
    DW BASIC_LLIST_ENTRY                  ; E9H LLIST
    DW BASIC_LOAD                         ; E8H LOAD
    DW LE452                              ; E7H LOMEM
    DW BASIC_NEW                          ; E6H NEW
    DW BASIC_NEXT                         ; E5H NEXT
    DW BASIC_COMMAND_LOOP                 ; E4H OK
    DW LE332                              ; E3H ON
    DW BASIC_OPEN                         ; E2H OPEN
    DW BASIC_PRINT                        ; E1H OUTPUT
    DW BASIC_OUT                          ; E0H OUT
    DW BASIC_PLOT                         ; DFH PLOT
    DW BASIC_POKE                         ; DEH POKE
    DW BASIC_PRINT                        ; DDH PRINT
    DW LE6C7                              ; DCH RANDOMIZE
    DW LE21B                              ; DBH READ
    DW LE6F4                              ; DAH RESTORE
    DW BASIC_RETURN                       ; D9H RETURN
    DW BASIC_RUN                          ; D8H RUN
    DW BASIC_SAVE                         ; D7H SAVE
    DW BASIC_SET                          ; D6H SET
    DW BASIC_SOUND                        ; D5H SOUND
    DW LFFA3                              ; D4H STOP
    DW BASIC_TRACE                        ; D3H TRACE
    DW LE9D3                              ; D2H VERIFY
    DW LE117                              ; D1H EXT
    DW LE570                              ; D0H LPRINT

; -----------------------------------------------------------------------------
; RST 18H ARITHMETIC DISPATCH TABLE
; -----------------------------------------------------------------------------
;
; Jump table for RST 18H arithmetic operations 0 through 14.
;
; Each little-endian address selects one operation in the BASIC arithmetic-stack interpreter. Most
; operations consume or produce a nine-byte stack value addressed by IY.
;
; Note:
;   Operations 0-3 perform add, divide, multiply, and subtract on the top two stack values and
;   advance IY by nine bytes. Operation 4 negates the top value. Operations 5-11 move numbered
;   constants or the X and Y arithmetic registers to and from the stack. Operation 12 copies the
;   top value down one slot, operation 13 stores the top value at HL, and operation 14 evaluates a
;   function argument onto the stack.
; -----------------------------------------------------------------------------
RST18_JUMP_TABLE:
    DW FP_ADD                             ; op 00H
    DW FP_DIV                             ; op 01H
    DW FP_MUL                             ; op 02H
    DW FP_SUB                             ; op 03H
    DW FP_NEGATE                          ; op 04H
    DW LEA82                              ; op 05H CONST: one extra byte, the constant number
    DW LEA9F                              ; op 06H
    DW LEA9A                              ; op 07H
    DW LEAD2                              ; op 08H
    DW LEACD                              ; op 09H
    DW LEAC3                              ; op 0AH
    DW LEABE                              ; op 0BH
    DW LFA92                              ; op 0CH
    DW LFB28                              ; op 0DH
    DW EVAL_NUMERIC_ARGUMENT              ; op 0EH

; -----------------------------------------------------------------------------
; INTELLIGENT SOFTWARE COPYRIGHT
; -----------------------------------------------------------------------------
;
; Copyright text embedded between the RST 18H table and the numeric constants.
;
; The bytes contain the ASCII text Copyright (c) 1984 Intelligent Software Ltd.
; -----------------------------------------------------------------------------
INTELLIGENT_SOFTWARE_COPYRIGHT:
    DB 43H, 6FH, 70H, 79H, 72H, 69H, 67H, 68H, 74H, 20H, 28H, 63H, 29H, 20H, 31H, 39H ; |Copyright (c) 19|
    DB 38H, 34H, 20H, 20H, 49H, 6EH, 74H, 65H, 6CH, 6CH, 69H, 67H, 65H, 6EH, 74H, 20H ; |84  Intelligent |
    DB 53H, 6FH, 66H, 74H, 77H, 61H, 72H, 65H, 20H, 4CH, 74H, 64H                     ; |Software Ltd|

; -----------------------------------------------------------------------------
; NUMBERED FLOATING-POINT CONSTANTS
; -----------------------------------------------------------------------------
;
; Forty constants stored in the BASIC arithmetic-stack number format.
;
; Each entry occupies seven bytes: a six-byte BCD mantissa followed by a characteristic byte. The
; arithmetic stack normally adds overflow and bookkeeping bytes around this representation, so the
; ROM table stores only the constant payload.
;
; The published listing identifies entry 0 as 0.5, entry 1 as 1.0, entry 34 as PI, and entry 35 as
; PI/2. Other entries are addressed by number from the RST 18H constant-loading operation.
; -----------------------------------------------------------------------------
NUMBERED_FP_CONSTANTS:
    DB 00H, 00H, 00H, 00H, 00H, 50H, 3FH, 00H, 00H, 00H, 00H, 00H, 10H, 40H, 31H, 24H ; |.....P?......@1$|
    DB 19H, 49H, 79H, 26H, 3FH, 57H, 07H, 08H, 05H, 32H, 17H, 40H, 69H, 75H, 80H, 50H ; |.Iy&?W...2.@iu.P|
    DB 20H, 73H, 3FH, 74H, 48H, 34H, 08H, 40H, 14H, C0H, 98H, 88H, 84H, 26H, 00H, 72H ; | s?tH4.@.....&.r|
    DB BFH, 19H, 89H, 03H, 25H, 20H, 43H, 40H, 99H, 45H, 58H, 22H, 52H, 47H, 40H, 07H ; |....% C@.EX"RG@.|
    DB 38H, 96H, 88H, 85H, 86H, 3FH, 00H, 00H, 00H, 00H, 51H, 11H, 40H, 23H, 70H, 49H ; |8....?....Q.@#pI|
    DB 46H, 25H, 29H, 3CH, 06H, 95H, 88H, 64H, 44H, 50H, 42H, 63H, 75H, 99H, 82H, 00H ; |F%)<...dDPBcu...|
    DB 14H, 41H, 16H, 65H, 64H, 73H, 28H, 33H, 3EH, 01H, 79H, 97H, 92H, 08H, 10H, 43H ; |.A.eds(3>.y....C|
    DB 97H, 10H, 08H, 94H, 20H, 11H, 42H, 99H, 92H, 50H, 58H, 02H, 23H, 40H, 90H, 37H ; |.... .B..PX.#@.7|
    DB 14H, 68H, 15H, 29H, C0H, 57H, 15H, 49H, 03H, 63H, 31H, 40H, 78H, 14H, 60H, 81H ; |.h.).W.I.c1@x.`.|
    DB 35H, 67H, BFH, 42H, 95H, 06H, 04H, 07H, 10H, C1H, 21H, 40H, 81H, 69H, 96H, 16H ; |5g.B......!@.i..|
    DB 41H, 67H, 54H, 04H, 80H, 90H, 81H, C0H, 07H, 38H, 96H, 88H, 85H, 86H, 3FH, 88H ; |AgT......8....?.|
    DB 60H, 66H, 66H, 66H, 16H, BFH, 56H, 20H, 07H, 33H, 33H, 83H, 3DH, 31H, 82H, 32H ; |`fff..V .33.=1.2|
    DB 08H, 84H, 19H, BCH, 78H, 06H, 71H, 39H, 52H, 27H, 3AH, 60H, 40H, 46H, 83H, 86H ; |....x.q9R':`@F..|
    DB 23H, B8H, 00H, 00H, 00H, 07H, 36H, 22H, 3FH, 00H, 00H, 00H, 27H, 44H, 89H, 3FH ; |#.....6"?...'D.?|
    DB 17H, 60H, 76H, 27H, 62H, 31H, 3FH, 31H, 51H, 79H, 57H, 29H, 57H, 41H, 59H, 53H ; |.`v'b1?1QyW)WAYS|
    DB 26H, 59H, 41H, 31H, 40H, 79H, 26H, 63H, 79H, 70H, 15H, 40H, 20H, 51H, 75H, 19H ; |&YA1@y&cyp.@ Qu.|
    DB 47H, 10H, 40H, 98H, 55H, 77H, 98H, 35H, 52H, 3FH, 00H, 00H, 00H, 80H, 76H, 32H ; |G.@.Uw.5R?....v2|
    DB 44H, 00H, 99H, 99H, 99H, 99H, 99H, 7EH                                         ; |D......~|

; -----------------------------------------------------------------------------
; SYSTEM INITIALIZATION
; -----------------------------------------------------------------------------
;
; Initializes the TVC after power-on and selects the warm- or cold-reset path.
;
; Initialization disables maskable interrupts, selects Z80 interrupt mode 1, establishes temporary
; SYS and EXTH mappings, clears the palette, programs the 6845 CRT controller, and decides whether
; destructive RAM tests are required.
;
; The EXTH routine at F13DH checks the RAM-resident U0 system stubs and WARM-FLAG. Its result is
; returned in the alternate accumulator and combined with COLD-FLAG at 0B22H. A valid U0 image
; permits warm reset; damaged U0 state or an explicit cold-reset request forces memory testing.
;
; Cold initialization tests U0, video RAM, U1, U2, and optional U3 RAM. U0 failure is fatal and
; produces a flashing border. The highest usable address discovered in U1/U2 is stored in HI-MEM
; at 0B19H. Warm initialization skips these destructive tests and uses a delay loop that allows a
; second RESET press to force the cold path.
;
; Entry:
;   Entered from RESET_VECTOR with SYS available through its page-0 alias.
;
; Exit:
;   Continues through WARM_RESET and EXTH initialization with RAM availability recorded in system
;   variables and AF'.
;
; Effects:
;   Changes paging and hardware state. The cold path destructively tests and clears RAM.
;
; Destroys:
;   AF, BC, DE, HL, SP, alternate AF, and the current memory mapping.
;
; Note:
;   The bytes at C26AH-C26BH deliberately form overlapping instructions. The cold path executes LD
;   A,08H from C26AH, while the warm-candidate path enters at C26BH and decodes the same 08H byte
;   as EX AF,AF'.
; -----------------------------------------------------------------------------
BASIC_COLD_START:
    DI
    IM 1
    LD A,40H
    OUT (02H),A
    JP LC233@SYS0
; Page-0 SYS continuation: M=40H keeps SYS in page 0, so this lands via the 0233H alias.
LC233:
    LD A,C0H
    OUT (02H),A
    JP F13DH                              ; EXT ROM F13DH RESET_INTEGRITY_CHECK (page 3 now maps EXT)
    LD A,40H

; Memory paging: S U V S page layout.
    OUT (02H),A
    JP C241H

    LD A,50H

; Memory paging: U U V S page layout.
    OUT (02H),A
    XOR A
    LD BC,0460H

    OUT (C),A
    INC C
    DJNZ C249H
    LD HL,CRTC_RESET_VALUES
    LD B,10H

    LD A,B
    DEC A
    OUT (70H),A
    LD A,(HL)
    OUT (71H),A
    INC HL
    DJNZ C253H
    LD A,80H

; Send one STROBE pulse.
    OUT (06H),A
    LD A,(COLD_FLAG)
    INC A
    JR NZ,C26BH
    LD (WARM_FLAG),A
    DB 3EH                                                                            ; |>|

    EX AF,AF'
    INC A
    JR Z,C2BBH
    LD SP,BFFFH
    LD HL,0000H
    CALL MEMORY_TEST
    JR Z,C27FH

    DEC A
    OUT (00H),A
    JR C27AH

    LD SP,STACK_TOP
    LD HL,8000H
    CALL MEMORY_TEST
    JR Z,C28EH
    LD A,88H
    OUT (C),A

    LD A,70H

; Memory paging: U U U S page layout.
    OUT (02H),A
    LD HL,4000H
    CALL MEMORY_TEST
    CALL Z,MEMORY_TEST
    DEC HL
    LD (HI_MEM),HL
    LD A,40H
    OUT (02H),A
    LD SP,BFFFH
    JP LC2A9@SYS0
; Page-0 SYS continuation: M=40H keeps SYS in page 0, so this lands via the 02A9H alias.
LC2A9:
    LD A,80H
    OUT (02H),A
    LD HL,C000H
    CALL MEMORY_TEST_PAGE0@SYS0
    LD A,00H
    JR Z,C2B8H
    DEC A

    EX AF,AF'
    JR WARM_RESET

    LD H,A
    LD L,A

    DEC HL
    LD A,H
    OR L
    JR NZ,C2BDH

; -----------------------------------------------------------------------------
; COMMON RESET CONTINUATION
; -----------------------------------------------------------------------------
;
; Converges the warm and cold reset paths and rebuilds the operating environment.
;
; SYS is restored in page 0, EXTH is selected in page 3, and control passes to EXT_INIT at F000H.
; The extension initializer rebuilds the U0 RAM stubs and initializes expansion-card descriptors
; before returning to SYS.
;
; After EXT_INIT, the routine acknowledges or disables interrupt sources, resets the border and
; printer/audio state, then calls the built-in video, keyboard, sound, and cassette initializers
; through the table at C555H.
;
; Entry:
;   AF' retains the U3 test result on the cold path; user RAM is preserved on the warm path.
;
; Exit:
;   Built-in devices and RAM-resident operating-system support are initialized.
;
; Effects:
;   Changes paging, stack position, I/O ports, and U0 system variables.
; -----------------------------------------------------------------------------
WARM_RESET:
    LD A,40H
    OUT (02H),A
    JP LC2C9@SYS0
; Page-0 SYS continuation: M=40H keeps SYS in page 0, so this lands via the 02C9H alias.
LC2C9:
    LD A,C0H
    OUT (02H),A
    LD SP,STACK_TOP
    JP F000H                              ; EXT ROM F000H EXT_INIT (page 3 now maps EXT)

LC2D3:
    XOR A
    LD (PORT03_MIRROR),A
    OUT (03H),A

; Clear cursor/sound interrupt.
    OUT (07H),A
    OUT (58H),A
    OUT (59H),A
    OUT (5AH),A
    OUT (5BH),A
    OUT (00H),A
    LD (BORDER),A
    LD A,80H
    OUT (06H),A
    LD (PORT06_MIRROR),A
    LD HL,STARTUP_DEVICE_INIT_TABLE
    LD BC,0470H

LC2F5:
    PUSH BC
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    PUSH HL
    EX DE,HL
    CALL JUMP_HL
    POP HL
    POP BC
    LD A,C

; Memory paging: U U U S page layout.
    OUT (02H),A
    DJNZ LC2F5

; -----------------------------------------------------------------------------
; SIDE-CARTRIDGE AUTOSTART
; -----------------------------------------------------------------------------
;
; Checks for the MOPS cartridge signature and transfers control to the cartridge entry point.
;
; The side cartridge is mapped into page 3 and its first four bytes are compared with the ASCII
; signature MOPS stored at C334H. If all bytes match, HL has advanced to cartridge offset 0004H
; and JP (HL) starts the cartridge immediately after the signature.
;
; If the signature is absent, the normal SYS mapping is restored, the mapping byte is saved in
; P-SAVE at 0003H, interrupts are enabled, and startup continues at BASIC_INIT.
;
; Exit:
;   Transfers to cartridge offset 0004H on success; otherwise continues normal BASIC startup.
;
; Effects:
;   Temporarily maps the side cartridge and may transfer control outside the system ROM.
; -----------------------------------------------------------------------------
CARTRIDGE_AUTOSTART:
    LD A,60H
    OUT (02H),A
    JP LC30D@SYS0
; Page-0 SYS continuation: M=60H keeps SYS in page 0, so this lands via the 030DH alias.
LC30D:
    LD A,20H

; Memory paging: S U U C page layout.
    OUT (02H),A
    LD HL,C000H
    LD DE,0334H
    LD B,04H

LC319:
    LD A,(DE)
    CP (HL)
    JR NZ,LC322
    INC DE
    INC HL
    DJNZ LC319

; JUMP_HL - Transfers control to the address in HL.
; Entry: HL = target address
; Exit: Control continues at the address in HL.
; Effects: Does not return by itself.
JUMP_HL:
    JP (HL)

LC322:
    LD A,60H
    OUT (02H),A
    JP LC329

LC329:
    LD A,70H

; Memory paging: U U U S page layout.
    OUT (02H),A
    LD (P_SAVE),A
    EI
    JP BASIC_INIT

; -----------------------------------------------------------------------------
; MOPS CARTRIDGE SIGNATURE
; -----------------------------------------------------------------------------
;
; Four-byte ASCII signature recognized by the side-cartridge autostart check.
;
; A cartridge beginning with MOPS is entered at the byte immediately after this four-character
; signature. The spelling is MOPS, matching the stored bytes 4DH, 4FH, 50H, 53H.
; -----------------------------------------------------------------------------
MOPS_SIGNATURE:
    DB 4DH, 4FH, 50H, 53H                                                             ; |MOPS|
; Page-0 entry to the RAM page test below: called with HL preset while SYS is in page 0
; (M=80H), so the call reaches C338H through its 0338H alias.
MEMORY_TEST_PAGE0:
    PUSH HL
    CALL LC348@SYS0
    JR LC342

; -----------------------------------------------------------------------------
; 16 KIB RAM PAGE TEST
; -----------------------------------------------------------------------------
;
; Destructively fills, verifies, and clears one complete 16 KiB memory page.
;
; The first pass fills the page with 55H. Every byte is decremented, compared with 54H, and
; cleared after a successful comparison. The second pass repeats the operation with AAH and
; expects A9H. This detects stuck bits and many address- or data-line faults.
;
; Two entry paths reach the same physical test body because SYS may be visible in page 0 or page
; 3. The sequence around C347H-C348H deliberately overlaps: the page-3 fall-through path treats
; 01H as the start of LD BC,553EH, while a direct call to C348H decodes LD A,55H.
;
; Entry:
;   HL = first address of the 16 KiB page under test.
;
; Exit:
;   Z is set when the full page passes. On failure NZ is set and the test stops at the first
;   mismatching cell. On success HL advances to the address after the page.
;
; Effects:
;   Overwrites the entire tested page and leaves every successfully tested byte cleared to zero.
;
; Destroys:
;   AF, BC, DE, HL, and all original contents of the tested page.
; -----------------------------------------------------------------------------
MEMORY_TEST:
    PUSH HL
    CALL LC348

LC342:
    POP DE
    RET NZ
    EX DE,HL
    LD A,AAH
    DB 01H                                ; = LD BC,553EH when run from here, hiding the entry below

LC348:
    LD A,55H
    PUSH HL
    LD E,L
    LD D,H
    INC DE
    LD (HL),A
    LD BC,3FFFH
    LDIR
    POP HL
    LD B,40H
    DEC A

LC358:
    DEC (HL)
    CPI
    DEC HL
    LD (HL),00H
    RET NZ
    INC HL
    RET PO
    JR LC358

; -----------------------------------------------------------------------------
; RST 30H OPERATING-SYSTEM DISPATCHER
; -----------------------------------------------------------------------------
;
; An OS call is RST 30H followed by one function byte:
;
;   bit 7     1 = use the input assignment table, 0 = the output table
;   bits 6-4  class: 0 video, 1 keyboard, 2 editor, 3 sound, 4 printer,
;             5 cassette, 6 expansion cards, 7 kernel
;   bits 3-0  function number within the class
;
; Parameters travel in BC and DE (for example the character in C); A returns
; the status, 00H for success. RST 30H jumps through 0030H to the U0 bridge at
; 0B23H, which the extension ROM copies from its template at reset:
;
;   0B23H  EX (SP),HL       ; HL = return address = address of the function byte
;          LD A,(HL)        ; A = function byte (the caller's A is not an input)
;          INC HL           ; return past the function byte
;          EX (SP),HL
;          EX AF,AF'        ; AF' = function byte
;          PUSH AF          ; save the caller's AF'
;          LD A,(P_SAVE)    ; save the caller's page map
;          PUSH AF
;          LD A,70H         ; U0 U1 U2 SYS
;          LD (P_SAVE),A
;          OUT (02H),A
;          JP C363H         ; RST30_DISPATCH
;
; and returns through 0B37H (RST30_BRIDGE_RET):
;
;   0B37H  EX AF,AF'        ; keep the result AF in AF'
;          POP AF           ; restore the caller's page map
;          LD (P_SAVE),A
;          OUT (02H),A
;          POP AF           ; the caller's AF' ...
;          EX AF,AF'        ; ... back in AF', the result in AF
;          RET
;
; How a call is routed (RST30_DECODE):
;
; - Class 7 goes straight to KERNEL_JUMP_TABLE.
; - Otherwise the class indexes the input (0B00H) or output (0B08H) selector
;   table. For functions 0-2 (interrupt, character, block I/O) a selector of
;   00H-06H redirects the call to that class; functions 3 and up always stay
;   with their own class. A selector with bit 7 set names an expansion slot
;   (00H-03H in bits 6-0) for every function. A selector of FFH accepts only
;   functions 3 and up.
; - Class 6 is always handled by the extension ROM, which takes the slot from
;   selector byte 7.
;
; DEVICE_CALL then indexes DEVICE_JUMP_TABLE_POINTERS by class and the class's
; counted jump table by function. The device routine runs with A = function
; byte (flags from OR A: S = input) and the caller's BC and DE.
;
; Entry:   from the 0B23H bridge; AF' = function byte; BC, DE = parameters.
; Exit:    A = status (00H = success), BC and DE as left by the device;
;          HL, IX, IY and the alternate BC, DE, HL are preserved.
; -----------------------------------------------------------------------------
RST30_DISPATCH:
    PUSH HL                             ; save the registers that are not
    PUSH IX                             ; parameters: HL, IX, IY ...
    PUSH IY
    EXX                                 ; ... and the alternate BC, DE, HL
    PUSH BC
    PUSH DE
    PUSH HL
    EXX
    EX AF,AF'                           ; A = function byte
    CALL RST30_DECODE                   ; route and call the device routine

; -----------------------------------------------------------------------------
; RST 30H COMMON RETURN
; -----------------------------------------------------------------------------
RST30_RETURN:
    EXX                                 ; restore the alternate set
    POP HL
    POP DE
    POP BC
    EXX
    POP IY
    POP IX
    POP HL
    JP RST30_BRIDGE_RET                 ; restore the page map and return

; Decode the function byte. The three pushes below are the frame that
; DEVICE_CALL and DEVICE_RETURN expect; INTERNAL_IRQ_DISPATCH fakes the same
; frame.

RST30_DECODE:
    PUSH AF                             ; frame: function byte (popped into HL later)
    PUSH BC                             ; frame: caller's BC
    PUSH DE                             ; frame: caller's DE
    LD C,A                              ; C = function byte
    OR A                                ; S = bit 7 (input), Z = function byte 00H
    EX AF,AF'                           ; keep it in AF' for the device routine
    LD HL,IO_ASSIGN_IN+1                ; keyboard input selector
    LD A,(HL)
    CP 02H                              ; routed to the editor?
    JR NZ,RST30_PICK_TABLE
    DEC (HL)                            ; yes: force it back to 01H, the keyboard

RST30_PICK_TABLE:
    LD DE,IO_ASSIGN_IN                  ; bit 7 set: input selectors
    LD A,C
    RLCA                                ; Cy = bit 7
    JR C,RST30_CLASS
    LD DE,IO_ASSIGN_OUT                 ; bit 7 clear: output selectors

RST30_CLASS:
    AND E0H                             ; class, now in bits 7-5
    RLCA
    RLCA
    RLCA                                ; class in bits 2-0
    LD B,A                              ; B = class
    LD A,C
    AND 0FH
    LD C,A                              ; C = function number
    LD A,B
    CP 07H                              ; kernel calls are never redirected
    JR Z,DEVICE_CALL
    LD L,B                              ; HL = DE + class
    XOR A
    LD H,A
    ADD HL,DE                           ; HL -> this class's selector
    DEC A                               ; A = FFH
    CP (HL)                             ; selector FFH?
    JR NZ,RST30_CHECK_SELECTOR
    LD A,C                              ; FFH: functions 3 and up go to the
    CP 03H                              ; class's own routine,
    JR NC,RST30_CLASS_CHECK             ; functions 0-2 fail below (7FH >= 07H)

RST30_CHECK_SELECTOR:
    LD A,(HL)
    RES 7,A                             ; selector without the card bit
    CP 07H
    JR C,RST30_ASSIGNED                 ; 00H-06H: valid
    LD A,FEH                            ; error FEH: bad assignment

RST30_FAIL:
    POP DE                              ; drop the frame's DE and BC
    POP BC                              ; (restoring the caller's)
    JR DEVICE_RETURN

RST30_ASSIGNED:
    BIT 7,(HL)                          ; selector names an expansion slot?
    JR Z,RST30_REDIRECT
    LD HL,F166H                         ; EXT F166H: call the card in slot A
                                        ; through its jump table
                                        ; (EXPANSION_INPUT_DISPATCH)

RST30_TO_EXT:
    JP CALL_EXTENSION_HL                ; EXT returns through DEVICE_RETURN

RST30_REDIRECT:
    LD A,C                              ; functions 0-2 follow the selector,
    CP 03H                              ; functions 3 and up stay put
    JR NC,RST30_CLASS_CHECK
    LD B,(HL)                           ; B = the class the selector names

RST30_CLASS_CHECK:
    LD A,B
    CP 06H                              ; class 6, expansion cards?
    LD HL,F16CH                         ; EXT F16CH: call the card in the slot
    JR Z,RST30_TO_EXT                   ; held in selector byte 7
                                        ; (EXPANSION_OUTPUT_DISPATCH)

; Call function C of device class B.

DEVICE_CALL:
    LD DE,DEVICE_JUMP_TABLE_POINTERS
    LD L,B                              ; HL = 2 * class
    LD H,00H
    ADD HL,HL
    ADD HL,DE
    LD E,(HL)                           ; DE = this class's jump table
    INC HL
    LD D,(HL)
    EX DE,HL                            ; HL -> function count
    LD A,(HL)
    DEC A                               ; highest function number
    CP C                                ; Cy if C is beyond it
    LD A,FFH                            ; error FFH: no such function
    JR C,RST30_FAIL
    INC HL                              ; HL -> first entry
    EX DE,HL
    LD L,C                              ; HL = 2 * function
    LD H,00H
    ADD HL,HL
    ADD HL,DE
    LD E,(HL)                           ; HL = routine address
    INC HL
    LD D,(HL)
    EX DE,HL
    EX AF,AF'                           ; A = function byte, S = input
    POP DE                              ; the caller's DE and BC
    POP BC                              ; are the routine's parameters
    CALL JUMP_HL                        ; run the device routine

; A = status from the device routine (or a dispatcher error code). The stack
; holds the frame's function byte, then the return address.

DEVICE_RETURN:
    OR A                                ; Z = success
    POP HL                              ; H = function byte
    RET Z                               ; done
    PUSH AF                             ; keep the error code
    LD A,(INT_FLAG)
    INC A                               ; inside the interrupt routine?
    JR Z,DEVICE_EXIT                    ; yes: just return the error
    PUSH BC
    PUSH DE
    LD A,H                              ; A = function byte
    LD HL,F1E6H                         ; EXT F1E6H: reset the selector of this
    JR RST30_TO_EXT                     ; class and direction to its default
                                        ; unless it names a card slot

DEVICE_RECOVERY_RET:
    POP DE                              ; EXT returns here through FFF0H
    POP BC

DEVICE_EXIT:
    POP AF                              ; A = error code, NZ
    RET

; -----------------------------------------------------------------------------
; RST 38H INTERRUPT ENTRY
; -----------------------------------------------------------------------------
;
; The Z80 runs in interrupt mode 1, so every interrupt executes the U0 RAM stub at
; 0038H, which the extension ROM copies from its template at reset:
;
;   0038H  PUSH AF          ; save the interrupted AF
;          LD A,70H         ; U0 U1 U2 SYS
;          OUT (02H),A      ; map SYS into page 3 (P_SAVE is not touched)
;          JP C412H         ; continue here
;
; Because the stub leaves P_SAVE alone, P_SAVE still describes the interrupted
; program's page map when this routine reads it. The handler saves that value and
; every register, including IX, IY and the alternate set, so IRQ_CORE and the
; device routines it calls may use any register.
;
; The U0 tail at 0B41H (IRQ_BRIDGE_TAIL) finishes the job:
;
;   0B41H  LD (P_SAVE),A    ; A = saved P_SAVE
;          OUT (02H),A      ; restore the interrupted page map
;          POP AF           ; the AF pushed by the stub
;          EI
;          RET
;
; Entry:   from the 0038H stub, interrupts disabled, SYS in page 3.
; Exit:    to IRQ_BRIDGE_TAIL with every register restored.
; -----------------------------------------------------------------------------
IRQ_HANDLER:
    LD A,(P_SAVE)                       ; page map of the interrupted program
    PUSH AF                             ; it is popped into A for IRQ_BRIDGE_TAIL
    PUSH HL                             ; save the main register set ...
    PUSH DE
    PUSH BC
    PUSH IX                             ; ... the index registers ...
    PUSH IY
    EX AF,AF'                           ; ... and the alternate set
    PUSH AF
    EXX
    PUSH HL
    PUSH DE
    PUSH BC
    CALL IRQ_CORE                       ; service every pending source
    POP BC                              ; restore in exactly the reverse order
    POP DE
    POP HL
    EXX
    POP AF
    EX AF,AF'
    POP IY
    POP IX
    POP BC
    POP DE
    POP HL
    POP AF                              ; A = interrupted page map
    JP IRQ_BRIDGE_TAIL                  ; restore it, POP AF, EI, RET

; -----------------------------------------------------------------------------
; SYSTEM INTERRUPT SERVICE
; -----------------------------------------------------------------------------
;
; Port 59H reports the interrupt sources, active low: bits 3-0 are expansion
; slots 3-0, bit 4 is the shared request of the CRTC cursor pulse and the sound
; timer. The cursor pulse is programmed to fire at the end of the visible picture,
; so bit 4 is the 50 Hz system tick.
;
; Every interrupt increments TIME and rewrites the border. A bit-4 request is
; acknowledged through port 07H and runs INTERNAL_IRQ_DISPATCH. Expansion-slot
; requests are acknowledged through ports 58H-5BH and passed to the extension
; ROM card dispatcher, which calls each requesting card's vector at C00EH.
;
; INT_FLAG is FFH for the whole routine. The device-call path at DEVICE_RETURN checks it:
; while it is set, a device error is returned as is instead of entering the
; extension ROM's error recovery at F1E6H.
;
; Entry:   from IRQ_HANDLER; all registers are free.
; Exit:    INT_FLAG = 00H.
; -----------------------------------------------------------------------------
IRQ_CORE:
    LD A,FFH                            ; mark interrupt service in progress
    LD (INT_FLAG),A
    LD HL,(TIME)                        ; advance the 16-bit software clock
    INC HL
    LD (TIME),HL
    LD A,(BORDER)                       ; reapply the border colour on every
    OUT (00H),A                         ; interrupt
    IN A,(59H)                          ; read the active-low source bits
    LD C,A                              ; and keep a copy in C
    BIT 4,A                             ; Z = cursor/sound tick pending
    SET 4,C                             ; (C bit 4 is not tested again)
    OUT (07H),A                         ; any write to 07H clears the bit-4 request
    CALL Z,INTERNAL_IRQ_DISPATCH        ; tick: device and card function 0
    LD A,C                              ; keep only the four slot bits,
    OR F0H                              ; forcing bits 7-4 high
    INC A                               ; FFH + 1 = 0: no slot requested
    JR Z,IRQ_CORE_EXIT

; At least one expansion slot requested service. For each requesting slot,
; write 80H to its port (58H + slot) when IRQ_STAT enables that slot and 00H
; when it does not; bit 7 clear disables and clears the slot's request.

    LD A,(IRQ_STAT)                     ; bits 5-2 enable slots 3-0
    RRCA
    RRCA                                ; now bits 3-0
    LD H,C                              ; H = request bits, rotated in the loop
    LD L,C                              ; L = unrotated copy for the dispatcher
    LD BC,0458H                         ; B = 4 slots, C = port 58H (slot 0)
    LD D,A                              ; D = enable bits, rotated in the loop

IRQ_CARD_ACK_LOOP:
    XOR A                               ; A = 00H
    RRC D                               ; Cy = this slot's enable bit
    RRA                                 ; A = 80H if enabled, 00H if not
    RRC H                               ; Cy = this slot's request bit (1 = idle)
    JR C,IRQ_CARD_ACK_NEXT              ; idle slot: leave its port alone
    OUT (C),A                           ; requesting slot: write its enable bit

IRQ_CARD_ACK_NEXT:
    INC C                               ; next port, 58H-5BH
    DJNZ IRQ_CARD_ACK_LOOP
    LD C,L                              ; C = the port 59H value again
    LD HL,IRQ_CORE_EXIT                 ; the card dispatcher returns there
    JR EXPANSION_IRQ_DISPATCH           ; call every requesting card's vector

IRQ_CORE_EXIT:
    XOR A                               ; interrupt service finished
    LD (INT_FLAG),A
    RET

; -----------------------------------------------------------------------------
; INTERNAL DEVICE INTERRUPTS
; -----------------------------------------------------------------------------
;
; Runs on every 50 Hz tick. INT_DES selects, active low, which devices get their
; function 0 (the interrupt-time routine):
;
;   bit 0  video     function 0 is a RET
;   bit 1  keyboard  KB_INT scans the matrix and handles auto-repeat
;   bit 2  editor    ED_INT blinks the cursor
;   bit 3  sound     TONE_INT times and stops a tone
;   bit 4+n          expansion card n, through the card dispatcher
;
; Each internal device is called through DEVICE_CALL, the device-table half of the
; RST 30H dispatcher. DEVICE_CALL expects the frame that LC37E builds for an RST 30H
; call: saved DE, BC and AF above a return address. The four pushes of
; INTERNAL_IRQ_NEXT fill those three slots and the return address, so after the
; device returns, by either the normal or the error path, control comes back to
; INTERNAL_IRQ_NEXT with the loop state intact.
;
; Entry:   C = port 59H value (saved and restored for IRQ_CORE).
; Exit:    to the caller of IRQ_CORE's CALL, through INTERNAL_IRQ_EXIT.
; -----------------------------------------------------------------------------
INTERNAL_IRQ_DISPATCH:
    PUSH BC                             ; keep the port 59H value for IRQ_CORE
    LD A,(INT_DES)                      ; active-low function-0 request mask
    LD C,A
    LD B,04H                            ; four internal devices

INTERNAL_IRQ_LOOP:
    RR C                                ; Cy = this device's INT_DES bit
    PUSH BC                             ; save the loop state
    JR C,INTERNAL_IRQ_NEXT              ; bit set: device not requested
    LD A,04H                            ; B counts 4..1; turn it into
    SUB B                               ; the device number 0..3
    LD B,A                              ; B = device (video, keyboard, editor, sound)
    LD C,00H                            ; C = function 0
    LD HL,INTERNAL_IRQ_NEXT             ; fake the RST 30H frame: DE, BC and AF
    PUSH HL                             ; slots, then the return address
    PUSH HL
    PUSH HL
    PUSH HL
    JP DEVICE_CALL                            ; look up and call device B, function 0

INTERNAL_IRQ_NEXT:
    LD A,70H                            ; U0 U1 U2 SYS again: the device may have
    OUT (02H),A                         ; mapped video RAM in
    POP BC                              ; recover the loop state
    DJNZ INTERNAL_IRQ_LOOP

; C has now been rotated four times, so its low nibble holds INT_DES bits 7-4:
; bit n set to 0 means card n wants its function 0 on every tick.

    LD HL,INTERNAL_IRQ_EXIT             ; the card dispatcher returns there

; -----------------------------------------------------------------------------
; EXPANSION INTERRUPT HANDOFF
; -----------------------------------------------------------------------------
;
; Calls the extension ROM card dispatcher at EXT F227H. For every clear bit in
; C bits 3-0 it selects that slot through port 03H, calls the address stored at
; C00EH in the card's ROM, restores the page map and finally rewrites port 03H
; from PORT03_MIRROR. Control then returns, through the FFF0H gateway, to the
; address in HL.
;
; IRQ_CORE uses this with the port 59H request bits; INTERNAL_IRQ_DISPATCH uses
; it with the INT_DES card bits.
;
; Entry:   C bits 3-0 = active-low slot mask; HL = continuation address.
; -----------------------------------------------------------------------------
EXPANSION_IRQ_DISPATCH:
    PUSH HL                             ; the continuation becomes the return address
    LD HL,F227H                         ; EXT ROM card dispatcher
    JP CALL_EXTENSION_HL                ; map EXT into page 3 and JP (HL)

INTERNAL_IRQ_EXIT:
    POP BC                              ; C = port 59H value for IRQ_CORE
    RET

; -----------------------------------------------------------------------------
; KERNEL FUNCTION TABLE
; -----------------------------------------------------------------------------
; Class 7 functions (RST 30H 70H-74H, or F0H-F4H). The kernel class is never
; redirected through the selector tables. Function 0 is the RET that ends
; IO_ASN; it exists only to keep the table layout uniform.
; -----------------------------------------------------------------------------
KERNEL_JUMP_TABLE:
    DB 05H                              ; five functions
    DW KERNEL_FN0_RET                   ; 0  (no operation)
    DW HI_MEM_SET                       ; 1  reserve memory below HI_MEM
    DW SLOT_ASN                         ; 2  assign a card to class 6
    DW IO_ASN                           ; 3  redirect a device class
    DW SLOT_NUM                         ; 4  find the slot holding a card

; -----------------------------------------------------------------------------
; KERNEL 01 - HI-MEM-SET
; -----------------------------------------------------------------------------
; Lowers HI_MEM by DE bytes and returns the reserved block, which runs from
; the new HI_MEM + 1 up to the old HI_MEM. The new HI_MEM may not go below
; 1EACH. On failure HI_MEM is unchanged.
;
; Entry:   DE = number of bytes to reserve.
; Exit:    A = 00H, DE = first byte of the block; or A = FBH.
; -----------------------------------------------------------------------------
HI_MEM_SET:
    LD HL,(HI_MEM)
    LD A,FBH                            ; error FBH unless both checks pass
    OR A                                ; clear Cy
    SBC HL,DE                           ; HL = proposed HI_MEM
    RET C                               ; DE is larger than HI_MEM
    PUSH HL
    LD DE,1EACH                         ; lowest HI_MEM allowed
    SBC HL,DE                           ; (Cy is clear here)
    POP HL
    RET C                               ; would go below 1EACH
    LD D,H                              ; DE = new HI_MEM + 1,
    LD E,L                              ; the start of the block
    INC DE
    LD (HI_MEM),HL                      ; commit
    XOR A                               ; success
    RET

; -----------------------------------------------------------------------------
; KERNEL 02 - SLOT-ASN
; -----------------------------------------------------------------------------
; Finds the card named by DE and C (see SLOT_NUM) and stores its slot number
; in selector byte 7 of the input or output table. Class 6 calls then go to
; that card.
;
; Entry:   DE -> counted card name; C = unit number; B = FFH for the input
;          table, anything else for the output table.
; Exit:    A = 00H, C = slot; or A = FDH (no such card), selector unchanged.
; -----------------------------------------------------------------------------
SLOT_ASN:
    LD HL,IO_ASSIGN_IN+7                ; input class-6 slot byte
    INC B                               ; B = FFH?
    JR Z,SLOT_ASN_FIND
    LD HL,IO_ASSIGN_OUT+7               ; no: output class-6 slot byte

SLOT_ASN_FIND:
    PUSH HL
    CALL SLOT_NUM                       ; C = slot if found
    POP HL
    OR A
    RET NZ                              ; FDH: not found
    LD (HL),C                           ; record the slot
    RET

; -----------------------------------------------------------------------------
; KERNEL 03 - IO-ASN
; -----------------------------------------------------------------------------
; Points the selector of class B at class C, so functions 0-2 of class B are
; served by class C (see RST30_DECODE). Both classes must be 0-6 and neither
; selector may be FFH (FFH marks a class with no character or block I/O).
; Cards cannot be assigned here; use SLOT_ASN and class 6.
;
; Entry:   B = class to redirect; C = class to use; D = FFH for the input
;          table, anything else for the output table.
; Exit:    A = 00H; or A = FEH, table unchanged.
; -----------------------------------------------------------------------------
IO_ASN:
    LD HL,IO_ASSIGN_IN
    INC D                               ; D = FFH?
    JR Z,IO_ASN_CHECK
    LD HL,IO_ASSIGN_OUT

IO_ASN_CHECK:
    PUSH HL                             ; table base
    LD A,06H
    CP B
    JR C,IO_ASN_FAIL                    ; B > 6
    LD E,B
    LD D,00H
    ADD HL,DE                           ; HL -> selector of class B
    LD E,(HL)
    INC E
    JR Z,IO_ASN_FAIL                    ; class B has selector FFH
    POP DE                              ; DE = table base
    PUSH HL                             ; keep the selector address
    EX DE,HL
    CP C                                ; A is still 06H
    JR C,IO_ASN_FAIL                    ; C > 6
    LD E,C
    XOR A
    LD D,A
    ADD HL,DE                           ; HL -> selector of class C
    LD E,(HL)
    INC E
    JR Z,IO_ASN_FAIL                    ; class C has selector FFH
    POP HL                              ; HL -> selector of class B
    LD (HL),C                           ; redirect it; A = 00H

KERNEL_FN0_RET:
    RET

IO_ASN_FAIL:
    POP HL                              ; drop the saved pointer
    LD A,FEH
    RET

; -----------------------------------------------------------------------------
; KERNEL 04 - SLOT-NUM
; -----------------------------------------------------------------------------
; Searches the four card descriptors that the extension ROM builds in U0 at
; reset, one per slot at 0040H + 30H * slot:
;
;   +0  counted card name (length byte, then up to six characters)
;   +7  unit number (cards of the same type are numbered 0, 1, ...)
;   +8  work area, passed in IX to the card's routines
;
; Entry:   DE -> counted card name; C = unit number.
; Exit:    A = 00H, C = slot 0-3, IX -> its descriptor; or A = FDH.
;          DE is preserved.
; -----------------------------------------------------------------------------
SLOT_NUM:
    PUSH DE                             ; keep the name pointer
    LD IX,0040H                         ; slot 0 descriptor
    LD B,04H                            ; four slots

SLOT_NUM_NEXT_SLOT:
    PUSH IX
    POP HL                              ; HL -> descriptor name
    PUSH BC
    LD A,(DE)                           ; compare the length byte and
    INC A                               ; that many characters
    LD B,A

SLOT_NUM_COMPARE:
    LD A,(DE)
    CP (HL)
    JR NZ,SLOT_NUM_NO_NAME
    INC HL
    INC DE
    DJNZ SLOT_NUM_COMPARE
    POP BC                              ; name matches
    PUSH IX
    POP HL
    LD DE,0007H
    ADD HL,DE                           ; HL -> unit number
    LD A,(HL)
    CP C
    JR NZ,SLOT_NUM_SKIP                 ; same card type, other unit
    LD A,04H                            ; B counts 4..1:
    SUB B                               ; turn it into the slot number
    LD C,A                              ; C = slot
    XOR A                               ; found

SLOT_NUM_EXIT:
    POP DE                              ; restore the name pointer
    RET

SLOT_NUM_NO_NAME:
    POP BC

SLOT_NUM_SKIP:
    LD DE,0030H                         ; next descriptor
    ADD IX,DE
    POP DE                              ; DE -> name again
    PUSH DE
    DJNZ SLOT_NUM_NEXT_SLOT
    LD A,FDH                            ; error FDH: no such card
    JR SLOT_NUM_EXIT

; Initial values for the 6845 video controller registers after reset.
; Sixteen bytes initialize CRTC registers R0 through R15 via ports 70H/71H.

; -----------------------------------------------------------------------------
; 6845 CRT CONTROLLER RESET VALUES
; -----------------------------------------------------------------------------
;
; Sixteen initial values written to the Motorola 6845-compatible CRT controller registers.
;
; The reset code selects controller registers 0 through 15 through port 70H and writes these bytes
; through port 71H: FFH, 0EH, 00H, 00H, 03H, 03H, 03H, 00H, 42H, 3CH, 02H, 4DH, 32H, 4BH, 40H,
; 63H.
;
; The table is ordered by CRTC register number, not by a software structure. Its values establish
; the TV timing, displayed character width/height, sync positions, and display start state used
; before the video OS takes over.
;
; Entry:
;   Consumed by the reset loop at C24EH-C25BH.
;
; Exit:
;   No direct return value; hardware registers are initialized.
;
; Effects:
;   Indirectly programs the CRT controller when the reset loop writes the table.
; -----------------------------------------------------------------------------
CRTC_RESET_VALUES:
    DB FFH, 0EH, 00H, 00H, 03H, 03H, 03H, 00H, 42H, 3CH, 02H, 4DH, 32H, 4BH, 40H, 63H ; |........B<.M2K@c|

; Startup initialization routine table: sets four-color mode, initializes keyboard, RET,
; initializes cassette workspace.
; Startup calls the four pointers in order: video, keyboard, sound, and cassette initialization.

; -----------------------------------------------------------------------------
; STARTUP DEVICE INITIALIZATION TABLE
; -----------------------------------------------------------------------------
;
; Four routine pointers used to initialize the built-in video, keyboard, sound, and cassette
; devices.
;
; The table contains C9F2H, D5ECH, D960H, and D9E2H. Reset iterates over these pointers with C=the
; mapping byte 70H and invokes each routine through C321H, allowing the same startup loop to
; initialize the devices in a uniform way.
;
; The first entry enters SET_4_COLOR_MODE, the second initializes the keyboard, the third is the
; sound initialization entry, and the fourth establishes cassette state. The values are pointers
; rather than calls so the reset code can keep the common paging and stack protocol in one place.
;
; Entry:
;   Read by the reset initialization loop at C2EFH.
;
; Exit:
;   Device initializers are called in table order.
;
; Effects:
;   Indirectly initializes video, keyboard, sound, and cassette state.
; -----------------------------------------------------------------------------
STARTUP_DEVICE_INIT_TABLE:
    DW SET_4_COLOR_MODE                   ; initialize video
    DW KEYBOARD_INIT                      ; initialize keyboard
    DW TONE_NOP                           ; initialize sound
    DW CAS_INIT                           ; initialize cassette

; Class pointers: video, keyboard, editor, sound, printer, cassette, expansion placeholder,
; kernel.

; -----------------------------------------------------------------------------
; DEVICE FUNCTION TABLE POINTERS
; -----------------------------------------------------------------------------
;
; Main table that maps the eight RST 30H function classes to their counted device jump tables.
;
; The eight little-endian pointers are the tables for video (C974H), keyboard (D5E3H), editor
; (CF98H), sound (D92AH), printer (D8FFH), cassette (D9BBH), expansion (0000H), and kernel
; (C4ACH). The 0000H expansion entry is a deliberate placeholder because expansion cards supply
; their own table through the EXT path.
;
; RST30_DISPATCH uses the class bits of the function code to index this table. Keeping all device
; table addresses together lets the common dispatcher apply the same count-and-function-number
; protocol to built-in and extension-backed devices.
;
; Entry:
;   Class number selected by the RST 30H dispatcher.
;
; Exit:
;   Pointer to the selected counted jump table.
;
; Effects:
;   Read-only table.
; -----------------------------------------------------------------------------
DEVICE_JUMP_TABLE_POINTERS:
    DW VIDEO_JUMP_TABLE                   ; class 0: video
    DW KEYBOARD_JUMP_TABLE                ; class 1: keyboard
    DW EDITOR_JUMP_TABLE                  ; class 2: editor
    DW SOUND_JUMP_TABLE                   ; class 3: sound
    DW PRINTER_JUMP_TABLE                 ; class 4: printer
    DW CASSETTE_JUMP_TABLE                ; class 5: cassette
    DW 0000H                              ; class 6: cards (handled by the extension ROM)
    DW KERNEL_JUMP_TABLE                  ; class 7: kernel

; -----------------------------------------------------------------------------
; BOUNDED BLOCK OUTPUT HELPER
; -----------------------------------------------------------------------------
; Calls a character-output routine once per byte of a memory block. Before each
; byte the source address is checked against HI_MEM.
;
; ROM bug: when the check fails, the RET C below leaves three words on the
; stack, so the CPU "returns" to the address held in BC (the remaining count)
; instead of to the caller with A = FAH. Checked in the emulator: an RST 30H
; printer block output (42H) with DE = C000H and BC = 6100H ends at 6100H.
;
; Entry:   HL = character routine (called with C = byte, DE -> byte);
;          DE -> block; BC = length, 1 or more (0 means 65536).
; Exit:    A = 00H after the whole block, or the routine's error code.
; -----------------------------------------------------------------------------
OUT_CHARS_SAFE:
    EX DE,HL                            ; HL -> block, DE = routine

OUT_CHARS_NEXT:
    PUSH HL                             ; saved across the call: source,
    PUSH DE                             ; routine and count
    PUSH BC
    PUSH HL
    PUSH DE
    EX DE,HL                            ; DE -> this byte
    LD HL,(HI_MEM)
    OR A
    SBC HL,DE                           ; Cy if the byte is above HI_MEM
    LD A,FAH                            ; error FAH: beyond HI_MEM
    POP DE
    POP HL
    RET C                               ; BUG: three words still pushed
    LD C,(HL)                           ; C = the byte
    EX DE,HL                            ; HL = routine, DE -> byte
    CALL CALL_HL                        ; output it
    POP BC
    POP DE
    POP HL
    OR A
    RET NZ                              ; the routine failed
    CPI                                 ; HL++, BC-- (the compare is unused)
    RET PO                              ; BC = 0: done, A = 00H
    JR OUT_CHARS_NEXT

; -----------------------------------------------------------------------------
; INDIRECT CALL PRIMITIVE
; -----------------------------------------------------------------------------
; CALL CALL_HL calls the routine at HL.
; -----------------------------------------------------------------------------
CALL_HL:
    JP (HL)

; -----------------------------------------------------------------------------
; BOUNDED BLOCK INPUT HELPER
; -----------------------------------------------------------------------------
; Calls a character-input routine once per byte and stores the character it
; returns in C. The destination address is checked against HI_MEM before each
; byte. The failed check has the same RET C bug as OUT_CHARS_SAFE. AF' is
; used as scratch.
;
; Entry:   HL = character routine (returns C = character, A = status);
;          DE -> buffer; BC = length, 1 or more.
; Exit:    A = 00H after the whole block, or the routine's error code; the
;          bytes before the failing one are stored.
; -----------------------------------------------------------------------------
IN_CHARS_SAFE:
    PUSH HL                             ; saved across the call: routine,
    PUSH DE                             ; destination and count
    PUSH BC
    PUSH HL
    PUSH DE
    LD HL,(HI_MEM)
    OR A
    SBC HL,DE                           ; Cy if the destination is above HI_MEM
    LD A,FAH                            ; error FAH: beyond HI_MEM
    POP DE
    POP HL
    RET C                               ; BUG: three words still pushed
    CALL CALL_HL                        ; read one character into C
    EX AF,AF'                           ; keep the status
    LD A,C                              ; A = the character
    POP BC
    POP DE
    POP HL
    EX AF,AF'                           ; A = status, A' = character
    OR A
    RET NZ                              ; the routine failed
    EX AF,AF'
    LD (DE),A                           ; store the character
    XOR A
    EX DE,HL
    CPI                                 ; DE++, BC--
    EX DE,HL
    RET PO                              ; BC = 0: done, A = 00H
    JR IN_CHARS_SAFE

; Built-in character matrix table for character codes 32-127, ten bytes per character.
; Ninety-six fixed glyphs, character codes 20H-7FH, occupy ten bytes per character.

; -----------------------------------------------------------------------------
; FIXED 8-BY-10 CHARACTER MATRIX
; -----------------------------------------------------------------------------
;
; The built-in 96-character glyph set for character codes 20H through 7FH.
;
; Each character occupies ten bytes. The first eight bytes are the visible 8-bit raster rows; the
; remaining two bytes provide the blank vertical spacing used by the character renderer. The table
; therefore occupies 960 bytes and ends immediately before VIDEO_JUMP_TABLE at C974H.
;
; The set covers ASCII space through DEL, including punctuation, digits, upper- and lower-case
; letters, and the TVC cursor glyph at code 7FH. The video character-output routine indexes this
; table after selecting the high-bit character bank for the alternate TVC glyph range.
;
; Entry:
;   Character code used as an index, with ten bytes per entry.
;
; Exit:
;   A ten-byte raster consumed by the video character renderer.
;
; Effects:
;   Read-only ROM data.
; -----------------------------------------------------------------------------
FIXED_CHARACTER_GLYPHS:
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 18H, 18H, 18H, 18H, 18H ; |................|
    DB 00H, 18H, 00H, 00H, 00H, 36H, 36H, 36H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 36H ; |.....666.......6|
    DB 36H, 7FH, 36H, 7FH, 36H, 36H, 00H, 00H, 00H, 18H, 3EH, 58H, 3CH, 1AH, 7CH, 18H ; |6.6.66....>X<.|.|
    DB 00H, 00H, 00H, 60H, 66H, 0CH, 18H, 30H, 66H, 06H, 00H, 00H, 00H, 10H, 28H, 28H ; |...`f..0f.....((|
    DB 30H, 54H, 48H, 34H, 00H, 00H, 00H, 18H, 18H, 30H, 00H, 00H, 00H, 00H, 00H, 00H ; |0TH4.....0......|
    DB 00H, 0CH, 18H, 30H, 30H, 30H, 18H, 0CH, 00H, 00H, 00H, 30H, 18H, 0CH, 0CH, 0CH ; |...000.....0....|
    DB 18H, 30H, 00H, 00H, 00H, 00H, 10H, 54H, 38H, 38H, 54H, 10H, 00H, 00H, 00H, 00H ; |.0.....T88T.....|
    DB 18H, 18H, 7EH, 18H, 18H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 18H, 18H ; |..~.............|
    DB 30H, 00H, 00H, 00H, 00H, 00H, 7CH, 7CH, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |0.....||........|
    DB 00H, 00H, 18H, 18H, 00H, 00H, 00H, 00H, 06H, 0CH, 18H, 30H, 60H, 00H, 00H, 00H ; |...........0`...|
    DB 00H, 3CH, 66H, 6EH, 7EH, 76H, 66H, 3CH, 00H, 00H, 00H, 18H, 38H, 18H, 18H, 18H ; |.<fn~vf<....8...|
    DB 18H, 18H, 00H, 00H, 00H, 3CH, 66H, 06H, 1CH, 30H, 60H, 7EH, 00H, 00H, 00H, 7EH ; |.....<f..0`~...~|
    DB 06H, 0CH, 1CH, 06H, 46H, 3CH, 00H, 00H, 00H, 0CH, 1CH, 2CH, 4CH, 7EH, 0CH, 0CH ; |....F<.....,L~..|
    DB 00H, 00H, 00H, 7EH, 60H, 7CH, 06H, 06H, 46H, 3CH, 00H, 00H, 00H, 3CH, 60H, 60H ; |...~`|..F<...<``|
    DB 7CH, 66H, 66H, 3CH, 00H, 00H, 00H, 7EH, 06H, 0CH, 18H, 30H, 60H, 60H, 00H, 00H ; ||ff<...~...0``..|
    DB 00H, 3CH, 66H, 66H, 3CH, 66H, 66H, 3CH, 00H, 00H, 00H, 3CH, 66H, 66H, 3EH, 06H ; |.<ff<ff<...<ff>.|
    DB 0CH, 38H, 00H, 00H, 00H, 00H, 00H, 18H, 18H, 00H, 18H, 18H, 00H, 00H, 00H, 00H ; |.8..............|
    DB 00H, 18H, 18H, 00H, 18H, 18H, 30H, 00H, 00H, 06H, 0CH, 18H, 30H, 18H, 0CH, 06H ; |......0.....0...|
    DB 00H, 00H, 00H, 00H, 00H, 7CH, 00H, 7CH, 00H, 00H, 00H, 00H, 00H, 30H, 18H, 0CH ; |.....|.|.....0..|
    DB 06H, 0CH, 18H, 30H, 00H, 00H, 00H, 3CH, 66H, 06H, 0CH, 18H, 00H, 18H, 00H, 00H ; |...0...<f.......|
    DB 00H, 3EH, 63H, 67H, 6BH, 6FH, 60H, 3CH, 00H, 00H, 00H, 1CH, 3EH, 63H, 63H, 7FH ; |.>cgko`<....>cc.|
    DB 63H, 63H, 00H, 00H, 00H, 7EH, 63H, 63H, 7EH, 63H, 63H, 7EH, 00H, 00H, 00H, 3EH ; |cc...~cc~cc~...>|
    DB 63H, 60H, 60H, 60H, 63H, 3EH, 00H, 00H, 00H, 7EH, 33H, 33H, 33H, 33H, 33H, 7EH ; |c```c>...~33333~|
    DB 00H, 00H, 00H, 7EH, 60H, 60H, 7CH, 60H, 60H, 7EH, 00H, 00H, 00H, 7EH, 60H, 60H ; |...~``|``~...~``|
    DB 7CH, 60H, 60H, 60H, 00H, 00H, 00H, 3EH, 63H, 60H, 60H, 67H, 63H, 3EH, 00H, 00H ; ||```...>c``gc>..|
    DB 00H, 63H, 63H, 63H, 7FH, 63H, 63H, 63H, 00H, 00H, 00H, 3CH, 18H, 18H, 18H, 18H ; |.ccc.ccc...<....|
    DB 18H, 3CH, 00H, 00H, 00H, 06H, 06H, 06H, 06H, 66H, 66H, 3CH, 00H, 00H, 00H, 63H ; |.<.......ff<...c|
    DB 66H, 6CH, 78H, 6CH, 66H, 63H, 00H, 00H, 00H, 60H, 60H, 60H, 60H, 60H, 60H, 7EH ; |flxlfc...``````~|
    DB 00H, 00H, 00H, 63H, 77H, 6BH, 63H, 63H, 63H, 63H, 00H, 00H, 00H, 66H, 66H, 76H ; |...cwkcccc...ffv|
    DB 6EH, 66H, 66H, 66H, 00H, 00H, 00H, 3EH, 63H, 63H, 63H, 63H, 63H, 3EH, 00H, 00H ; |nfff...>ccccc>..|
    DB 00H, 7EH, 63H, 63H, 7EH, 60H, 60H, 60H, 00H, 00H, 00H, 3EH, 63H, 63H, 63H, 6BH ; |.~cc~```...>ccck|
    DB 67H, 3EH, 01H, 00H, 00H, 7EH, 63H, 63H, 7EH, 6CH, 66H, 63H, 00H, 00H, 00H, 3EH ; |g>...~cc~lfc...>|
    DB 63H, 60H, 3EH, 03H, 63H, 3EH, 00H, 00H, 00H, 7EH, 5AH, 18H, 18H, 18H, 18H, 18H ; |c`>.c>...~Z.....|
    DB 00H, 00H, 00H, 63H, 63H, 63H, 63H, 63H, 63H, 3EH, 00H, 00H, 00H, 63H, 63H, 63H ; |...cccccc>...ccc|
    DB 63H, 36H, 1CH, 08H, 00H, 00H, 00H, 63H, 63H, 63H, 6BH, 6BH, 3EH, 14H, 00H, 00H ; |c6.....ccckk>...|
    DB 00H, 66H, 66H, 3CH, 18H, 3CH, 66H, 66H, 00H, 00H, 00H, 66H, 66H, 3CH, 18H, 18H ; |.ff<.<ff...ff<..|
    DB 18H, 18H, 00H, 00H, 00H, 7EH, 06H, 0CH, 18H, 30H, 60H, 7EH, 00H, 00H, 00H, 3CH ; |.....~...0`~...<|
    DB 30H, 30H, 30H, 30H, 30H, 3CH, 00H, 00H, 00H, 00H, 60H, 30H, 18H, 0CH, 06H, 00H ; |00000<....`0....|
    DB 00H, 00H, 00H, 3CH, 0CH, 0CH, 0CH, 0CH, 0CH, 3CH, 00H, 00H, 00H, 18H, 3CH, 66H ; |...<.....<....<f|
    DB 42H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 7EH, 00H, 00H ; |B............~..|
    DB 00H, 30H, 30H, 18H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 3CH, 06H, 3EH ; |.00..........<.>|
    DB 66H, 3EH, 00H, 00H, 00H, 60H, 60H, 7CH, 66H, 66H, 66H, 7CH, 00H, 00H, 00H, 00H ; |f>...``|fff|....|
    DB 00H, 1EH, 30H, 30H, 30H, 1EH, 00H, 00H, 00H, 06H, 06H, 3EH, 66H, 66H, 66H, 3EH ; |..000......>fff>|
    DB 00H, 00H, 00H, 00H, 00H, 3CH, 66H, 7EH, 60H, 3CH, 00H, 00H, 00H, 0CH, 18H, 18H ; |.....<f~`<......|
    DB 3CH, 18H, 18H, 18H, 00H, 00H, 00H, 00H, 00H, 3EH, 66H, 66H, 66H, 3EH, 06H, 3CH ; |<........>fff>.<|
    DB 00H, 60H, 60H, 7CH, 66H, 66H, 66H, 66H, 00H, 00H, 00H, 18H, 00H, 38H, 18H, 18H ; |.``|ffff.....8..|
    DB 18H, 3CH, 00H, 00H, 00H, 18H, 00H, 38H, 18H, 18H, 18H, 18H, 18H, 70H, 00H, 60H ; |.<.....8.....p.`|
    DB 60H, 66H, 6CH, 78H, 6CH, 66H, 00H, 00H, 00H, 18H, 18H, 18H, 18H, 18H, 18H, 18H ; |`flxlf..........|
    DB 00H, 00H, 00H, 00H, 00H, 76H, 6BH, 6BH, 6BH, 6BH, 00H, 00H, 00H, 00H, 00H, 7CH ; |.....vkkkk.....||
    DB 66H, 66H, 66H, 66H, 00H, 00H, 00H, 00H, 00H, 3CH, 66H, 66H, 66H, 3CH, 00H, 00H ; |ffff.....<fff<..|
    DB 00H, 00H, 00H, 7CH, 66H, 66H, 66H, 7CH, 60H, 60H, 00H, 00H, 00H, 3EH, 66H, 66H ; |...|fff|``...>ff|
    DB 66H, 3EH, 06H, 06H, 00H, 00H, 00H, 36H, 38H, 30H, 30H, 30H, 00H, 00H, 00H, 00H ; |f>.....68000....|
    DB 00H, 1EH, 30H, 1CH, 06H, 3CH, 00H, 00H, 00H, 18H, 18H, 3CH, 18H, 18H, 18H, 0CH ; |..0..<.....<....|
    DB 00H, 00H, 00H, 00H, 00H, 66H, 66H, 66H, 66H, 3EH, 00H, 00H, 00H, 00H, 00H, 66H ; |.....ffff>.....f|
    DB 66H, 66H, 3CH, 18H, 00H, 00H, 00H, 00H, 00H, 63H, 63H, 6BH, 3EH, 14H, 00H, 00H ; |ff<......cck>...|
    DB 00H, 00H, 00H, 66H, 3CH, 18H, 3CH, 66H, 00H, 00H, 00H, 00H, 00H, 66H, 66H, 66H ; |...f<.<f.....fff|
    DB 66H, 3EH, 06H, 3CH, 00H, 00H, 00H, 7EH, 0CH, 18H, 30H, 7EH, 00H, 00H, 00H, 0EH ; |f>.<...~..0~....|
    DB 18H, 18H, 70H, 18H, 18H, 0EH, 00H, 00H, 00H, 18H, 18H, 18H, 00H, 18H, 18H, 18H ; |..p.............|
    DB 00H, 00H, 00H, 70H, 18H, 18H, 0EH, 18H, 18H, 70H, 00H, 00H, 00H, 00H, 00H, 33H ; |...p.....p.....3|
    DB 6BH, 66H, 00H, 00H, 00H, 00H, 00H, 7EH, 7EH, 7EH, 7EH, 7EH, 7EH, 7EH, 00H, 00H ; |kf.....~~~~~~~..|

; VIDEO routine jump table; first byte is the routine count, followed by routine addresses.
; The video table has 13 callable functions; function 0 is the interrupt placeholder RET.

; -----------------------------------------------------------------------------
; VIDEO FUNCTION TABLE
; -----------------------------------------------------------------------------
;
; Counted jump table for the thirteen built-in video operating-system functions.
;
; The count byte is 0DH. The entries are the interrupt placeholder C9A9H, character output CC94H,
; block output CC86H, character positioning CF4BH, video mode C9F4H, clear screen CA49H, absolute
; pen movement CADAH, relative pen movement CAD7H, pen down CBF3H, pen up CBFFH, paint CD48H,
; character definition CF2CH, and palette definition CA38H.
;
; The common RST 30H dispatcher obtains this table through DEVICE_JUMP_TABLE_POINTERS. The first
; entry is reserved for function 0, which is used by the interrupt dispatcher and is a RET because
; the video device has no periodic interrupt work of its own.
;
; Entry:
;   Video function number selected by the operating-system dispatcher.
;
; Exit:
;   Target address for the selected video operation.
;
; Effects:
;   Read-only table.
; -----------------------------------------------------------------------------
VIDEO_JUMP_TABLE:
    DB 0DH                                ; 13 functions
    DW VID_INT_NOP                        ; 0  interrupt (no work)
    DW VID_CHOUT                          ; 1  character output
    DW VID_BKOUT                          ; 2  block output
    DW CH_POS                             ; 3  character position
    DW VID_MODE                           ; 4  select mode
    DW VID_CLS                            ; 5  clear screen
    DW B_ABS                              ; 6  move beam to X,Y
    DW B_REL                              ; 7  move beam by X,Y
    DW B_ON                               ; 8  pen down
    DW B_OFF                              ; 9  pen up
    DW PAINT                              ; 10 flood fill
    DW CH_DEF                             ; 11 define character
    DW PAL_DEF                            ; 12 define palette

; =============================================================================
; VIDEO AND GRAPHICS
; =============================================================================
; Coordinates. Graphics calls use physical coordinates: X 0-1023 (03FFH) from
; the left, Y 0-959 (03BFH) from the bottom. One logical pixel is 2, 4 or 8
; units wide in 2-, 4- and 16-colour mode (2 ^ (COLOR_MODE + 1); 512, 256 or
; 128 pixels across) and 4 units high (240 rows). X_PHYSICAL / Y_PHYSICAL hold
; the beam position, X_LOGICAL / Y_LOGICAL the same point as pixel column and
; row, with row 0 at the top.
;
; Video RAM. With page 2 mapped to VID (port 02H = 50H, U0 U1 VID SYS) the
; picture is at 8000H-BBFFH: 240 rows of 64 bytes, address = 8000H +
; 40H * row + X / 16. A byte holds 8, 4 or 2 pixels. PIXEL_MASK selects the
; current pixel's bits: 80H, 88H or AAH for the leftmost pixel, rotated right
; for the next one. A 4-colour pixel keeps colour bit 0 in the high nibble and
; bit 1 in the low nibble; a 16-colour pixel keeps colour bits 3-0 in bits
; 7, 5, 3, 1 (left pixel) or 6, 4, 2, 0 (right pixel).
;
; Colours are written as whole-byte patterns (COLOR_BYTE): the colour repeated
; for every pixel of a byte, so writing "pattern AND mask" sets one pixel.
; PAPER_PATTERN and INK_PATTERN hold the patterns for the current drawing.
;
; Characters are 8 x 10 pixels: 64, 32 or 16 per line and 24 lines, 40 units
; apart. X_PHYSICAL / Y_PHYSICAL give the top left pixel of the next cell.
; =============================================================================

; -----------------------------------------------------------------------------
; VIDEO RAM PAGING GUARD
; -----------------------------------------------------------------------------
; A video routine starts with CALL CALL_WITH_SYS_PAGED. The guard maps page 2
; to video RAM (port 02H = 50H), then continues the routine with the old page
; map and VIDEO_PAGE_RETURN pushed in front of its return address. When the
; routine returns, VIDEO_PAGE_RETURN restores the page map and returns to the
; routine's caller with the routine's AF.
;
; Destroys HL on entry and AF' on exit.
; -----------------------------------------------------------------------------
CALL_WITH_SYS_PAGED:
CALL_WITH_VIDEO_PAGED:
VIDEO_PAGE_GUARD:
    POP HL                              ; HL = the rest of the video routine
    LD A,(P_SAVE)
    PUSH AF                             ; keep the caller's page map
    LD A,50H                            ; U0 U1 VID SYS
    LD (P_SAVE),A
    OUT (02H),A
    PUSH HL
    LD HL,VIDEO_PAGE_RETURN             ; the routine's RET comes back here
    EX (SP),HL
    JP (HL)                             ; continue the routine

VIDEO_PAGE_RETURN:
    EX AF,AF'                           ; keep the routine's result
    POP AF                              ; A = the caller's page map
    LD (P_SAVE),A
    OUT (02H),A
    EX AF,AF'

VID_INT_NOP:
    RET

; -----------------------------------------------------------------------------
; WORD TABLE LOOKUP
; -----------------------------------------------------------------------------
; Entry:   HL -> table of words; A = index.
; Exit:    DE = entry A; HL -> its high byte; A = 2 * index.
; -----------------------------------------------------------------------------
JUMP_TABLE_DISPATCH:
JUMP_TABLE_LOOKUP:
TABLE_LOOKUP_WORD:
    ADD A,A
    LD E,A
    LD D,00H
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    RET

; Range checks. Cy if BC (X) or DE (Y) is off the screen; HL = the distance to
; the right or top edge otherwise.

CHECK_X_COORDINATE:
    LD HL,03FFH                         ; X 0-1023
    OR A
    SBC HL,BC
    RET

CHECK_Y_COORDINATE:
    LD HL,03BFH                         ; Y 0-959
    OR A
    SBC HL,DE
    RET

; HL = physical X / (2 ^ (COLOR_MODE + 1)): the pixel column. A = COLOR_MODE.

LOGICAL_TO_PHYSICAL_COORDINATE:
    LD A,(COLOR_MODE)
    LD B,A
    INC B

; HL = HL / (2 ^ B), unsigned.

SHIFT_HL_RIGHT:
    SRL H
    RR L
    DJNZ SHIFT_HL_RIGHT
    RET

; -----------------------------------------------------------------------------
; PREPARE DRAWING STATE
; -----------------------------------------------------------------------------
; Entries, each doing less than the one before:
;   PREPARE_DRAW_STATE    load the line style, then as PREPARE_COLOURS
;   PREPARE_COLOURS       PAPER_PATTERN; then as PREPARE_INK
;   PREPARE_INK           IY = the L_MODE pixel writer, INK_PATTERN = INK
;   SET_INK_PATTERN       INK_PATTERN = A (PAINT stores INK XOR the target)
; All end in GET_PIXEL_WORK_STATE and return DE = 0040H, the row stride
; (PREPARE_INK keeps the caller's DE).
;
; Exit:    A = C = PIXEL_MASK; HL = VID_ADDR.
; -----------------------------------------------------------------------------
PREPARE_GRAPHICS_POINT:
PREPARE_DRAW_STATE:
    CALL LOAD_LINE_STYLE

PREPARE_COLOURS:
    CALL PAPER_BYTE
    LD (PAPER_PATTERN),A
    LD DE,0040H                         ; one row down

PREPARE_INK:
    CALL SELECT_CROSSING_MODE           ; IY = pixel writer
    CALL INK_BYTE

SET_INK_PATTERN:
    LD (INK_PATTERN),A

GET_PIXEL_WORK_STATE:
    LD A,(PIXEL_MASK)
    LD C,A
    LD HL,(VID_ADDR)
    RET

; -----------------------------------------------------------------------------
; RESET PALETTE TABLE
; -----------------------------------------------------------------------------
; VID_MODE loads the first four bytes when port 59H bit 6 is 1, the second
; four when it is 0.
; -----------------------------------------------------------------------------
DEFAULT_PALETTE_TABLE:
    DB 00H, 50H, 44H, 41H                 ; port 59H bit 6 = 1
DEFAULT_PALETTE_ALT:
    DB 00H, 55H, 50H, 44H                 ; port 59H bit 6 = 0

; -----------------------------------------------------------------------------
; VIDEO 04 - SELECT DISPLAY MODE
; -----------------------------------------------------------------------------
; SET_4_COLOR_MODE is the reset entry (STARTUP_DEVICE_INIT_TABLE). VID_MODE
; resets L_MODE, PAPER and V_FLAG to 0 and L_STYLE to 1 (solid), sets INK to
; 1, or in 16-colour mode to 0FH (port 59H bit 6 = 0) or 0CH (bit 6 = 1),
; clears the screen (VID_CLS), sets the mode bits of port 06H and loads the
; default palette.
;
; Entry:   C = mode: 00H 2 colours, 01H 4 colours, 02H 16 colours.
; Exit:    A = 00H, or F7H for another value.
; -----------------------------------------------------------------------------
SET_4_COLOR_MODE:
    LD C,01H                            ; 4-colour mode

VID_MODE:
    LD A,C
    CP 03H
    LD A,F7H                            ; error F7H: no such mode
    RET NC
    XOR A
    LD (L_MODE),A                       ; put
    LD (PAPER),A                        ; paper colour 0
    LD (V_FLAG),A                       ; draw both ink and paper
    INC A
    LD (L_STYLE),A                      ; solid lines
    CP C
    JR NC,VID_MODE_SET_INK              ; 2 or 4 colours: INK = 1
    IN A,(59H)
    RLCA
    RLCA                                ; Cy = port 59H bit 6
    LD A,0FH
    JR NC,VID_MODE_SET_INK
    LD A,0CH

VID_MODE_SET_INK:
    LD (INK),A
    LD A,C
    LD (COLOR_MODE),A
    PUSH BC
    CALL VID_CLS                        ; clear the screen in the new mode
    POP BC
    LD A,C                              ; port 06H bits 1-0 = mode,
    LD HL,PORT06_MIRROR                 ; other bits unchanged
    XOR (HL)
    AND FCH
    XOR C
    LD (HL),A
    OUT (06H),A
    IN A,(59H)
    RLCA
    RLCA                                ; Cy = port 59H bit 6
    LD DE,DEFAULT_PALETTE_ALT
    JR NC,PAL_DEF
    LD DE,DEFAULT_PALETTE_TABLE

; -----------------------------------------------------------------------------
; VIDEO 12 - DEFINE PALETTE
; -----------------------------------------------------------------------------
; Entry:   DE -> four bytes for palette registers 0-3 (ports 60H-63H).
; Exit:    A = 00H.
; -----------------------------------------------------------------------------
PAL_DEF:
    LD A,(DE)
    OUT (60H),A                         ; palette 0
    INC DE
    LD A,(DE)
    OUT (61H),A                         ; palette 1
    INC DE
    LD A,(DE)
    OUT (62H),A                         ; palette 2
    INC DE
    LD A,(DE)
    OUT (63H),A                         ; palette 3
    XOR A
    RET

; -----------------------------------------------------------------------------
; VIDEO 05 - CLEAR SCREEN
; -----------------------------------------------------------------------------
; Resets the editor, fills the whole picture (8000H-BBFFH) with PAPER, raises
; the pen and moves the beam to X = 0, Y = 0 (bottom left), falling into
; PIXEL_ADDRESS.
; -----------------------------------------------------------------------------
VID_CLS:
    CALL CALL_WITH_SYS_PAGED
    CALL EDITOR_INIT
    CALL PAPER_BYTE                     ; A = PAPER as a byte pattern
    LD HL,8000H
    LD DE,8001H
    LD (HL),A
    LD BC,3BFFH                         ; 3C00H bytes: 240 rows x 64
    LDIR
    CALL B_OFF                          ; pen up, A = 00H
    LD B,A                              ; BC = X = 0, DE = Y = 0
    LD C,A
    LD D,A
    LD E,A

; -----------------------------------------------------------------------------
; PIXEL ADDRESS AND MASK
; -----------------------------------------------------------------------------
; Moves the beam to (BC, DE) without checking the range: stores the physical
; and logical coordinates, PIXEL_MASK and VID_ADDR. SET_BEAM_X changes only X.
;
; Entry:   BC = X; DE = Y (PIXEL_ADDRESS only).
; Exit:    A = 00H.
; -----------------------------------------------------------------------------
PIXEL_ADDRESS:
    LD (Y_PHYSICAL),DE

SET_BEAM_X:
    LD (X_PHYSICAL),BC
    LD HL,(X_PHYSICAL)
    PUSH HL
    LD B,04H
    CALL SHIFT_HL_RIGHT                 ; X / 16 = byte column 0-63
    EX (SP),HL                          ; keep it; HL = X
    CALL LOGICAL_TO_PHYSICAL_COORDINATE ; HL = pixel column, A = mode
    LD (X_LOGICAL),HL
    LD C,A                              ; BC = mode (B is 0)
    LD A,L
    LD HL,PIXEL_LEFT_MASKS
    ADD HL,BC                           ; HL -> the mode's leftmost mask
    AND 07H
    INC A
    LD B,A                              ; B = (column AND 7) + 1
    LD A,(HL)
    DB 1EH                                ; = LD E,0FH when run from here, hiding the entry below

PIXEL_MASK_ROTATE:
    RRCA                                ; rotate once per pixel: the LD E,0FH
    DJNZ PIXEL_MASK_ROTATE              ; above skips the first RRCA
    LD (PIXEL_MASK),A
    LD DE,(Y_PHYSICAL)
    LD HL,03BFH
    XOR A
    SBC HL,DE                           ; flip Y: 0 at the top
    LD B,02H
    CALL SHIFT_HL_RIGHT                 ; / 4 = row 0-239
    LD (Y_LOGICAL),HL
    LD H,L                              ; HL = row * 64
    SRL H
    RRA
    SRL H
    RRA
    LD L,A
    POP BC                              ; + byte column
    ADD HL,BC
    LD DE,8000H                         ; + start of video RAM
    ADD HL,DE
    LD (VID_ADDR),HL
    XOR A
    RET

PIXEL_LEFT_MASKS:
    DB 80H                                ; 2 colours: 8 pixels per byte
    DB 88H                                ; 4 colours: 4 pixels per byte
    DB AAH                                ; 16 colours: 2 pixels per byte

; -----------------------------------------------------------------------------
; PIXEL WRITERS
; -----------------------------------------------------------------------------
; PLOT_CHARACTER_PIXEL draws one character pixel: Cy = glyph bit (1 = ink);
; A = V_FLAG. V_FLAG bit 0 set: ink pixels are not drawn; bit 1 set: paper
; pixels are not drawn. PLOT_LINE_PIXEL draws a line pixel: Cy = line pattern
; bit, 1 = ink, 0 = paper. PLOT_INK / PLOT_PAPER draw the pixel in that
; colour. All write through IY, one of the four L_MODE writers below, which
; take A = colour pattern, C = PIXEL_MASK, HL = video address.
; -----------------------------------------------------------------------------
PLOT_CHARACTER_PIXEL:
    JR NC,PLOT_CHAR_PAPER
    AND 01H                             ; ink pixel: suppressed by V_FLAG bit 0?
    JR Z,PLOT_INK
    RET

PLOT_LINE_PIXEL:
    JR NC,PLOT_PAPER

PLOT_INK:
    LD A,(INK_PATTERN)
    JP (IY)

PLOT_CHAR_PAPER:
    AND 02H                             ; paper pixel: suppressed by bit 1?
    RET NZ

PLOT_PAPER:
    LD A,(PAPER_PATTERN)
    JP (IY)

; L_MODE writers (LINE_MODE_TABLE). Only the bits in C change.

PLOT_PIXEL_CROSSING:
    AND (HL)                            ; 2 AND: colour AND old, then as put

PLOT_PUT:
    XOR (HL)                            ; 0 put: old XOR ((colour XOR old) AND C)

PLOT_XOR:
    AND C                               ; 3 XOR: old XOR (colour AND C)
    XOR (HL)
    LD (HL),A
    RET

PLOT_OR:
    AND C                               ; 1 OR: old OR (colour AND C)
    OR (HL)
    LD (HL),A
    RET

; -----------------------------------------------------------------------------
; VIDEO 07 - MOVE BEAM RELATIVELY / VIDEO 06 - MOVE BEAM ABSOLUTELY
; -----------------------------------------------------------------------------
; Moves the beam to (BC, DE), or by (BC, DE) for B_REL. With the pen down
; (B_STAT not 00H) a line is drawn from the old position, both ends included.
;
; Entry:   BC = X, DE = Y (signed offsets for B_REL).
; Exit:    A = 00H, or F9H if the point is off the screen (beam unchanged).
; -----------------------------------------------------------------------------
B_REL:
    CALL ADD_BEAM_OFFSET                ; BC, DE += the beam position

B_ABS:
    CALL CALL_WITH_SYS_PAGED
    LD A,F9H                            ; error F9H: off the screen
    CALL CHECK_X_COORDINATE
    CALL NC,CHECK_Y_COORDINATE
    RET C
    LD A,(B_STAT)
    OR A
    JP Z,PIXEL_ADDRESS                  ; pen up: just move

; -----------------------------------------------------------------------------
; INTEGER LINE DRAWER
; -----------------------------------------------------------------------------
; Bresenham's algorithm in pixel coordinates. The major axis is the one with
; the larger difference. Each step takes the straight step along the major
; axis (LINE_STEP_1) while the error term in HL' is negative, otherwise the
; diagonal step (LINE_STEP_2). The direction picks the step routines:
;
;   LINE_DIAG_TABLE, index Y-up * 2 + X-left
;   LINE_AXIS_TABLE, index Y-up * 4 + X-left * 2 + Y-major
;
; Each pixel takes the next bit of LINE_PATTERN (from L_STYLE): 1 draws INK,
; 0 draws PAPER.
;
; Entry:   from B_ABS; BC, DE = new physical point, already checked.
; -----------------------------------------------------------------------------
DRAW_LINE:
    EXX
    LD BC,(X_LOGICAL)                   ; the old point, in pixels
    LD DE,(Y_LOGICAL)
    PUSH BC
    PUSH DE
    CALL PREPARE_GRAPHICS_POINT         ; this set: HL, C = old address, mask;
                                        ; DE = 0040H
    EXX
    CALL PIXEL_ADDRESS                  ; the beam moves to the new point
    LD BC,(X_LOGICAL)
    LD DE,(Y_LOGICAL)
    XOR A
    EX DE,HL                            ; HL = new row
    POP DE                              ; DE = old row
    CALL ABSOLUTE_DIFFERENCE            ; HL = |dy|, A bit 0 = up
    POP DE                              ; DE = old column
    PUSH HL
    LD H,B
    LD L,C                              ; HL = new column
    CALL ABSOLUTE_DIFFERENCE            ; HL = |dx|, A = up * 2 + left
    PUSH HL
    LD HL,LINE_DIAG_TABLE
    CALL JUMP_TABLE_DISPATCH            ; A doubles to up * 4 + left * 2
    LD (LINE_STEP_2),DE                 ; the diagonal step
    POP HL                              ; HL = |dx|
    POP DE                              ; DE = |dy|
    PUSH HL
    SBC HL,DE                           ; Cy = Y major
    POP HL
    JR C,LINE_AXES_SET                  ; HL = minor, DE = major
    EX DE,HL

LINE_AXES_SET:
    LD B,H
    LD C,L                              ; BC = minor
    PUSH DE                             ; keep major, the step count
    ADC A,00H                           ; + Y-major
    SBC HL,DE                           ; HL = minor - major
    PUSH HL
    SRL D
    RR E
    ADD HL,DE                           ; error = minor - major / 2
    PUSH HL
    LD HL,LINE_AXIS_TABLE
    CALL JUMP_TABLE_DISPATCH
    LD (LINE_STEP_1),DE                 ; the straight step
    POP HL                              ; HL' = error, DE' = minor - major,
    POP DE                              ; BC' = minor
    EXX
    EX (SP),HL                          ; HL = major, the old address on the stack
    LD A,H                              ; A:B = major + 1 points (two-level
    LD B,L                              ; count: B inner, A' outer)
    INC B
    INC A
    EX AF,AF'
    POP HL                              ; HL = old address
    JR LINE_PLOT                        ; plot the start point first

LINE_NEXT_OUTER:
    EX AF,AF'

LINE_NEXT:
    CALL STEP_LINE_PIXEL

LINE_PLOT:
    LD A,(LINE_PATTERN)
    RLCA                                ; Cy = next pattern bit
    LD (LINE_PATTERN),A
    CALL PLOT_LINE_PIXEL
    DJNZ LINE_NEXT
    EX AF,AF'
    DEC A
    JR NZ,LINE_NEXT_OUTER
    RET

; HL = |HL - DE|; A = A * 2 + (1 if HL < DE). Enter with Cy clear.

ABSOLUTE_DIFFERENCE:
    SBC HL,DE
    JR NC,ABS_DIFF_DONE
    EX DE,HL                            ; negative: HL = 1 - (HL - DE) - 1
    LD HL,0001H
    SBC HL,DE

ABS_DIFF_DONE:
    RLA
    RET

; LINE_PATTERN = LINE_STYLE_TABLE entry L_STYLE - 1 (1-16).

LOAD_LINE_STYLE:
    LD A,(L_STYLE)
    DEC A
    AND 0FH
    LD C,A
    LD HL,LINE_STYLE_TABLE
    LD B,00H
    ADD HL,BC
    LD A,(HL)
    LD (LINE_PATTERN),A
    RET

; Line patterns for L_STYLE 1-16, used from bit 7 down: 1 = ink, 0 = paper.

LINE_STYLE_TABLE:
    DB FFH, AAH, CCH, EEH, 88H, DAH, E4H, F6H, FAH, FEH, FCH, F8H, F0H, EAH, FFH, FFH ; |................|

; IY = the writer for L_MODE (bits 1-0). DE is preserved.

SELECT_CROSSING_MODE:
    PUSH DE
    LD A,(L_MODE)
    AND 03H
    LD HL,LINE_MODE_TABLE
    CALL JUMP_TABLE_DISPATCH
    PUSH DE
    POP IY
    POP DE
    RET

LINE_MODE_TABLE:
    DW PLOT_PUT                           ; L_MODE 0 put
    DW PLOT_OR                            ; L_MODE 1 OR
    DW PLOT_PIXEL_CROSSING                ; L_MODE 2 AND
    DW PLOT_XOR                           ; L_MODE 3 XOR

; -----------------------------------------------------------------------------
; LINE PIXEL STEPPER
; -----------------------------------------------------------------------------
; Advances the error term and calls the straight or diagonal step. The steps
; move HL (video address) and C (pixel mask) by one pixel; DE = 0040H. Up is
; towards the top of the screen (lower addresses). VID_CHOUT_AT also uses
; LINE_RIGHT, LINE_LEFT and LINE_DOWN.
; -----------------------------------------------------------------------------
STEP_LINE_PIXEL:
    EXX
    BIT 7,H                             ; error negative?
    JR Z,LINE_STEP_DIAGONAL
    ADD HL,BC                           ; error += minor
    EXX
    LD IX,(LINE_STEP_1)                 ; straight step
    JP (IX)

LINE_STEP_DIAGONAL:
    ADD HL,DE                           ; error += minor - major
    EXX
    LD IX,(LINE_STEP_2)                 ; diagonal step
    JP (IX)

LINE_UP_LEFT:
    OR A
    SBC HL,DE                           ; up a row, then left

LINE_LEFT:
    RLC C                               ; mask one pixel left
    RET NC
    DEC L                               ; into the previous byte
    RET

LINE_RIGHT_UP:
    RRC C                               ; right, then up
    JR NC,LINE_UP
    INC L

LINE_UP:
    OR A
    SBC HL,DE                           ; up a row
    RET

LINE_LEFT_DOWN:
    RLC C                               ; left, then down
    JR NC,LINE_DOWN
    DEC L

LINE_DOWN:
    ADD HL,DE                           ; down a row
    RET

LINE_DOWN_RIGHT:
    ADD HL,DE                           ; down, then right

LINE_RIGHT:
    RRC C                               ; mask one pixel right
    RET NC
    INC L                               ; into the next byte
    RET

LINE_DIAG_TABLE:
    DW LINE_DOWN_RIGHT                    ; down, right
    DW LINE_LEFT_DOWN                     ; down, left
    DW LINE_RIGHT_UP                      ; up, right
    DW LINE_UP_LEFT                       ; up, left

LINE_AXIS_TABLE:
    DW LINE_RIGHT                         ; down, right, X major
    DW LINE_DOWN                          ; down, right, Y major
    DW LINE_LEFT                          ; down, left, X major
    DW LINE_DOWN                          ; down, left, Y major
    DW LINE_RIGHT                         ; up, right, X major
    DW LINE_UP                            ; up, right, Y major
    DW LINE_LEFT                          ; up, left, X major
    DW LINE_UP                            ; up, left, Y major

; -----------------------------------------------------------------------------
; VIDEO 08 - LOWER BEAM / VIDEO 09 - RAISE BEAM
; -----------------------------------------------------------------------------
; B_ON plots the current point in INK and sets B_STAT = FFH, so beam moves
; draw. B_OFF sets B_STAT = 00H. Both return A = 00H.
; -----------------------------------------------------------------------------
B_ON:
    CALL CALL_WITH_SYS_PAGED
    CALL PREPARE_INK
    CALL PLOT_INK                       ; draw the current point
    LD A,FFH
    DB 26H                                ; = LD H,AFH when run from here, hiding the entry below

B_OFF:
    XOR A
    LD (B_STAT),A
    XOR A
    RET

; Colour patterns. PAPER_BYTE returns A = B = PAPER as a byte pattern,
; INK_BYTE A = INK, COLOR_BYTE A = colour A. Destroys HL.

PAPER_BYTE:
    LD A,(PAPER)
    CALL COLOR_BYTE
    LD B,A
    RET

INK_BYTE:
    LD A,(INK)

COLOR_BYTE:
    LD HL,(COLOR_MODE)                  ; L = mode
    SRL L
    JR C,COLOR_BYTE_4                   ; mode 1
    SRL L
    JR C,COLOR_BYTE_16                  ; mode 2
    AND 01H                             ; 2 colours: 00H or FFH
    RRA
    SBC A,A
    RET

COLOR_BYTE_4:
    AND 03H                             ; 4 colours: bit 0 in bits 7-4,
    RRA                                 ; bit 1 in bits 3-0
    RL L                                ; L bit 0 = colour bit 0
    RRA
    SBC A,A                             ; A = FFH if colour bit 1
    RR L
    RRA                                 ; bit 7 = colour bit 0
    SRA A                               ; copy it to bits 6-4
    SRA A
    SRA A
    RET

COLOR_BYTE_16:
    AND 0FH                             ; 16 colours: bit n in bits
    LD H,04H                            ; 2n+1 and 2n

COLOR_BYTE_16_BIT:
    RRA                                 ; next colour bit, from bit 0
    RR L
    SRA L                               ; doubled into the top two bits
    DEC H
    JR NZ,COLOR_BYTE_16_BIT
    LD A,L
    RET

; -----------------------------------------------------------------------------
; READ PIXEL COLOUR
; -----------------------------------------------------------------------------
; Exit:    A = colour number of the pixel at the beam.
; -----------------------------------------------------------------------------
READ_PIXEL_COLOR:
    CALL GET_PIXEL_WORK_STATE           ; C = mask, HL = address
    LD A,(COLOR_MODE)
    LD B,A
    LD A,C
    AND (HL)                            ; the pixel's bits
    SRL B
    JR C,READ_PIXEL_4
    SRL B
    JR C,READ_PIXEL_16
    ADD A,FFH                           ; 2 colours: Cy if the bit is set
    RLA
    AND 01H
    RET

READ_PIXEL_4:
    ADD A,F0H                           ; Cy = colour bit 0 (high nibble)
    RL C                                ; keep it in C
    AND 0FH
    ADD A,FFH                           ; Cy = colour bit 1 (low nibble)
    RLA
    RR C
    RLA
    AND 03H
    RET

READ_PIXEL_16:
    ADD A,C0H                           ; Cy = colour bit 3 (bits 7-6)
    RLA
    AND 7FH
    ADD A,E0H                           ; bit 2 (bits 5-4)
    RLA
    AND 3FH
    ADD A,F0H                           ; bit 1 (bits 3-2)
    RLA
    AND 1FH
    ADD A,F8H                           ; bit 0 (bits 1-0)
    RLA
    AND 0FH
    RET

; BC += X_PHYSICAL, DE += Y_PHYSICAL.

ADD_BEAM_OFFSET:
    LD HL,(X_PHYSICAL)
    ADD HL,BC
    LD B,H
    LD C,L
    LD HL,(Y_PHYSICAL)
    ADD HL,DE
    EX DE,HL
    RET

; -----------------------------------------------------------------------------
; VIDEO 02 - BLOCK OUTPUT
; -----------------------------------------------------------------------------
; Calls VID_CHOUT for each byte. There is no HI_MEM check and errors are not
; checked; A is the status of the last character.
;
; Entry:   DE -> text; BC = length.
; -----------------------------------------------------------------------------
VID_BKOUT:
    EX DE,HL

VID_BKOUT_NEXT:
    PUSH HL
    PUSH BC
    LD C,(HL)
    CALL VID_CHOUT
    POP BC
    POP HL
    CPI                                 ; HL++, BC--
    RET PO
    JR VID_BKOUT_NEXT

; -----------------------------------------------------------------------------
; VIDEO 01 - CHARACTER OUTPUT
; -----------------------------------------------------------------------------
; Draws character C at the beam and moves the beam one cell right. If the cell
; does not fit on the line, a CR and LF come first. 0DH (CR) moves to X = 0;
; 0AH (LF) moves down one line if another line fits below, otherwise does
; nothing. Codes 00H-1FH other than these, and E0H-FFH, are ignored. 20H-7FH
; come from FIXED_CHARACTER_GLYPHS, 80H-DFH from the RAM glyphs at 0740H
; (CH_DEF). The editor scrolls; this routine never does.
;
; Entry:   C = character.
; Exit:    A = 00H.
; -----------------------------------------------------------------------------
VID_CHOUT:
    CALL CALL_WITH_SYS_PAGED
    LD A,C
    CP 20H
    JR NC,VID_CHOUT_GLYPH
    CP 0AH
    JP Z,VID_LINE_FEED
    CP 0DH
    JP Z,VID_CARRIAGE_RETURN
    XOR A                               ; other control codes: ignored
    RET

VID_CHOUT_GLYPH:
    CP E0H
    LD A,00H
    RET NC                              ; E0H-FFH: ignored
    EXX                                 ; keep the character in C'
    LD DE,FFDCH                         ; DE = -36: the cell's bottom row
    LD A,(COLOR_MODE)
    LD B,A
    LD A,01H
    INC B

VID_PIXEL_WIDTH:
    ADD A,A                             ; A = 2 ^ (mode + 1), units per pixel
    DJNZ VID_PIXEL_WIDTH
    LD (CHAR_PIXEL_W),A
    LD L,A
    ADD A,A
    ADD A,L
    ADD A,A
    ADD A,L
    LD C,A                              ; BC = 7 * width: the cell's last column
    XOR A
    CALL ADD_BEAM_OFFSET
    CALL CHECK_Y_COORDINATE
    RET C                               ; below the bottom: draw nothing
    CALL CHECK_X_COORDINATE
    JR NC,VID_CHOUT_AT                  ; fits on this line
    CALL VID_CARRIAGE_RETURN            ; else start a new line first
    CALL VID_LINE_FEED

; -----------------------------------------------------------------------------
; RENDER GLYPH AT THE BEAM
; -----------------------------------------------------------------------------
; Draws the 8 x 10 glyph of C' at VID_ADDR, row by row, most significant bit
; leftmost, then advances the beam one cell.
; -----------------------------------------------------------------------------
VID_CHOUT_AT:
    CALL PREPARE_COLOURS                ; HL, C = cell address, mask; DE = 0040H
    EXX                                 ; this set: C = character
    LD DE,FIXED_CHARACTER_GLYPHS-140H   ; codes 20H-7FH ...
    BIT 7,C
    JR Z,VID_GLYPH_ADDRESS
    LD DE,0740H                         ; ... or 80H-DFH from RAM

VID_GLYPH_ADDRESS:
    RES 7,C
    LD B,00H
    LD H,B
    LD L,C
    ADD HL,HL
    ADD HL,HL
    ADD HL,BC
    ADD HL,HL                           ; HL = 10 * (code AND 7FH)
    ADD HL,DE                           ; -> the glyph
    LD B,0AH                            ; ten rows
    LD A,(V_FLAG)
    LD E,A                              ; E = V_FLAG
    DEC HL

VID_GLYPH_ROW:
    INC HL
    LD C,(HL)                           ; C = glyph row
    EXX
    LD B,08H                            ; eight pixels
    JR VID_GLYPH_PIXEL

VID_GLYPH_NEXT_PIXEL:
    CALL LINE_RIGHT

VID_GLYPH_PIXEL:
    EXX
    RL C                                ; Cy = next glyph bit
    LD A,E                              ; A = V_FLAG
    EXX
    CALL PLOT_CHARACTER_PIXEL
    DJNZ VID_GLYPH_NEXT_PIXEL
    CALL LINE_DOWN                      ; next row,
    LD B,07H                            ; back to the first column

VID_GLYPH_BACK:
    CALL LINE_LEFT
    DJNZ VID_GLYPH_BACK
    EXX
    DJNZ VID_GLYPH_ROW
    LD A,(CHAR_PIXEL_W)                 ; X += 8 * pixel width
    ADD A,A
    ADD A,A
    ADD A,A
    LD C,A
    CALL ADD_BEAM_OFFSET                ; (only the new X is used)
    CALL CHECK_X_COORDINATE
    JR NC,VID_SET_X                     ; still on the screen
    LD BC,VID_LINE_FEED                 ; past the right edge: CR, then LF
    PUSH BC

VID_CARRIAGE_RETURN:
    LD BC,0000H                         ; X = 0

VID_SET_X:
    JP SET_BEAM_X

VID_LINE_FEED:
    LD DE,FFB4H                         ; would the line after the next one
    CALL ADD_BEAM_OFFSET                ; still fit (Y - 76 >= 0)?
    XOR A
    CALL CHECK_Y_COORDINATE
    RET C                               ; no: stay
    LD DE,FFD8H                         ; Y -= 40
    LD BC,0000H
    CALL ADD_BEAM_OFFSET
    JP PIXEL_ADDRESS

; -----------------------------------------------------------------------------
; VIDEO 10 - FLOOD FILL
; -----------------------------------------------------------------------------
; Fills, with INK, the area of the beam pixel's colour (the target) that
; contains the beam: a scan-line fill. Each filled run is pushed on the Z80
; stack as a 6-byte record:
;
;   address  video address of the run's first byte (pushed first)
;   mask     C = PIXEL_MASK of the first pixel; B = bytes to the right edge
;   width    pixel count in bits 13-0, flags in bits 15-14 (popped first):
;              00  the run reaches beyond its parent run (left of the scan
;                  start or past the parent's end): scan both rows
;              01  inside the parent, found in the row below it
;              11  inside the parent, found in the row above it
;
; A zero word under the first record ends the work. Each popped run is
; scanned in the row below and the row above (PAINT_DIR' = 2, 1) for target
; pixels, which are filled and pushed in turn. For a run inside its parent,
; the row towards the parent is the parent itself, already filled, so
; PAINT_SCAN_PASS skips it and scans only the row away from the parent.
; (Checked in the emulator: filling a rectangle gives one 00 run, the seed;
; the others are 01 below and 11 above it.)
;
; Pixels are written with the XOR writer and INK_PATTERN = INK XOR target, so
; a target pixel becomes INK under any L_MODE. The stack may grow down to
; PAINT_STACK_MIN. When it gets within 22 bytes of that, PAINT_COMPACT drops
; the records whose rows above and below hold no target pixels any more; if
; none can be dropped, the run waiting to be pushed is abandoned (that part of
; the area may stay unfilled).
;
; Exit:    A = 00H (also when INK already equals the target colour).
; -----------------------------------------------------------------------------
PAINT:
    CALL CALL_WITH_SYS_PAGED
    LD (PAINT_SP_SAVE),SP               ; base of the record stack
    LD IY,PLOT_XOR                      ; target XOR (INK XOR target) = INK
    CALL READ_PIXEL_COLOR
    CALL COLOR_BYTE
    LD (PAINT_TARGET),A                 ; the target as a byte pattern
    LD E,A
    CALL INK_BYTE
    XOR E
    LD HL,0000H
    RET Z                               ; INK = target: nothing to do
    LD (PAINT_RECORD),HL                ; no parent run
    PUSH HL                             ; end-of-work marker
    LD HL,(X_PHYSICAL)
    LD B,04H
    CALL SHIFT_HL_RIGHT                 ; byte column
    PUSH HL
    CALL SET_INK_PATTERN                ; INK_PATTERN = INK XOR target;
                                        ; C = mask, HL = beam address
    LD A,40H
    POP DE
    SUB E
    LD B,A                              ; B = bytes to the right edge
    LD (PAINT_ADDR),HL
    LD (PAINT_SPAN),BC
    LD B,01H                            ; one pass: the beam row itself
    JR PAINT_SCAN_ROW

; Take the next run record: scan the row below it, then the row above.

PAINT_NEXT_RECORD:
    POP HL                              ; width word, or the 0000H marker
    LD A,H
    OR L
    RET Z                               ; done (A = 00H)
    POP BC
    LD (PAINT_RECORD),HL
    LD (PAINT_SPAN),BC
    POP HL                              ; address
    LD DE,0040H
    ADD HL,DE                           ; the row below
    LD (PAINT_ADDR),HL
    LD B,02H                            ; two passes: below, then above
    LD DE,BC00H
    SBC HL,DE                           ; below the picture?
    JP NC,PAINT_NEXT_DIR

PAINT_SCAN_ROW:
    EXX                                 ; B' = passes left (PAINT_DIR')

PAINT_SCAN_PASS:
    CALL PAINT_COMPACT                  ; room for another record?
    JR C,PAINT_NEXT_RECORD              ; no: drop this run
    LD HL,(PAINT_RECORD)
    LD A,H
    RES 7,H
    RES 6,H
    LD (PAINT_WIDTH),HL                 ; the parent run's width
    EX DE,HL
    BIT 6,A                             ; run inside its parent?
    JR Z,PAINT_SCAN_START               ; no (00): scan this row
    RLCA                                ; A bit 0 = found above (bit 15)
    EXX
    XOR B                               ; B' bit 0 = 1 in the "above" pass
    EXX
    AND 01H
    JP NZ,PAINT_NEXT_PASS               ; this is the parent's row: skip it

PAINT_SCAN_START:
    LD HL,(PAINT_ADDR)
    LD BC,(PAINT_SPAN)
    LD A,(PAINT_TARGET)
    XOR (HL)
    AND C                               ; is the start pixel the target?
    JR NZ,PAINT_FIND_RUN                ; no: look further right
    LD A,40H                            ; yes: fill leftwards first.
    SUB B                               ; B = bytes to the left edge
    INC A
    LD B,A
    LD DE,0000H                         ; DE = pixels filled to the left
    JR PAINT_LEFT_STEP

PAINT_LEFT:
    LD A,(PAINT_TARGET)
    XOR (HL)
    AND C
    JR NZ,PAINT_LEFT_END                ; not the target: run starts after it
    CALL PLOT_INK                       ; fill it
    INC DE

PAINT_LEFT_STEP:
    RLC C                               ; one pixel left
    JR NC,PAINT_LEFT
    DEC L
    DJNZ PAINT_LEFT                     ; until the left edge

PAINT_LEFT_END:
    LD A,40H                            ; B = bytes to the right edge again
    SUB B
    INC A
    LD B,A
    RRC C                               ; back to the run's first pixel
    JR NC,PAINT_RUN_LEFT_DONE
    INC L
    DEC B

PAINT_RUN_LEFT_DONE:
    PUSH HL                             ; record: address,
    PUSH BC                             ; mask and byte count,
    PUSH DE                             ; pixels so far
    LD HL,(PAINT_ADDR)
    LD BC,(PAINT_SPAN)
    CALL PAINT_FILL_RIGHT               ; fill rightwards from the start
    EX (SP),HL
    LD A,H
    OR L
    JR Z,PAINT_RUN_WIDTH
    SET 7,H                             ; flag: it grew left of the start

PAINT_RUN_WIDTH:
    ADD HL,DE                           ; width = left + right
    EX (SP),HL

; Subtract the run from the parent's remaining width and set the record's
; flags: 00 if the run grew left (bit 15 cleared again) or passed the
; parent's end; otherwise 01 (below pass) or 11 (above pass). Then continue
; with the rest of the parent.

PAINT_RUN_CHECK:
    PUSH HL
    LD HL,(PAINT_WIDTH)
    OR A
    SBC HL,DE                           ; parent width left - pixels filled
    POP DE                              ; DE = end address
    EX (SP),HL                          ; HL = the run's width word
    PUSH AF                             ; Cy: the run passed the parent's end
    LD A,80H
    BIT 7,H                             ; grew left?
    JR NZ,PAINT_FLAG_SET                ; yes: clear bit 15, flags 00
    JR C,PAINT_FLAG_DONE                ; passed the end: flags stay 00
    EXX
    RRC B                               ; Cy = B' bit 0: 1 in the above pass
    RLC B
    EXX
    RRA                                 ; A = C0H above, 40H below

PAINT_FLAG_SET:
    XOR H
    LD H,A

PAINT_FLAG_DONE:
    POP AF
    EX (SP),HL                          ; the width word back in the record
    JR C,PAINT_NEXT_PASS                ; parent width used up
    JR Z,PAINT_NEXT_PASS
    EX DE,HL

; Skip non-target pixels to the next run under the parent.

PAINT_FIND_RUN:
    CALL PAINT_COMPACT
    JP C,PAINT_NEXT_RECORD
    LD A,E
    OR A
    JR NZ,PAINT_FIND_LOOP
    DEC D

PAINT_FIND_LOOP:
    LD A,(PAINT_TARGET)
    XOR (HL)
    AND C
    JR Z,PAINT_FOUND_RUN                ; a target pixel: a new run
    DEC E
    JR Z,PAINT_FIND_COUNT

PAINT_FIND_STEP:
    RRC C                               ; one pixel right
    JR NC,PAINT_FIND_LOOP
    INC L
    DEC B
    JR PAINT_FIND_LOOP

PAINT_FOUND_RUN:
    LD A,E
    OR A
    JR NZ,PAINT_FOUND_COUNT
    INC D

PAINT_FOUND_COUNT:
    LD (PAINT_WIDTH),DE                 ; parent width left
    PUSH HL                             ; record: address,
    EX (SP),HL
    PUSH BC                             ; mask and byte count,
    CALL PAINT_FILL_RIGHT
    PUSH DE                             ; width
    JR PAINT_RUN_CHECK

PAINT_FIND_COUNT:
    DEC D
    JP P,PAINT_FIND_STEP                ; parent width left: keep looking

; This row is done: the next pass (the row above), or the next record.

PAINT_NEXT_PASS:
    EXX

PAINT_NEXT_DIR:
    DEC B
    JP Z,PAINT_NEXT_RECORD
    EXX
    LD HL,(PAINT_ADDR)
    LD DE,FF80H
    ADD HL,DE                           ; two rows up: the row above the run
    LD (PAINT_ADDR),HL
    BIT 7,H                             ; above the picture?
    JP Z,PAINT_NEXT_RECORD
    JP PAINT_SCAN_PASS

; Fill target pixels rightwards from HL / C until a different pixel or the
; right edge (B bytes). Exit: DE = pixels filled; HL, C, B at the stop.

PAINT_FILL_RIGHT:
    LD DE,0000H

PAINT_FILL_PIXEL:
    LD A,(PAINT_TARGET)
    XOR (HL)
    AND C
    RET NZ                              ; not the target: stop
    INC DE
    LD A,(INK_PATTERN)                  ; INK XOR target
    AND C
    XOR (HL)
    LD (HL),A                           ; the pixel becomes INK
    RRC C
    JR NC,PAINT_FILL_PIXEL
    INC L
    DJNZ PAINT_FILL_PIXEL
    RET

; -----------------------------------------------------------------------------
; PAINT STACK SPACE
; -----------------------------------------------------------------------------
; Exit:    NC if there is room for another record (SP - 22 >= PAINT_STACK_MIN),
;          after compacting the record stack if needed; Cy if nothing could be
;          freed. BC, DE, HL, BC' are preserved.
;
; Compaction walks the records from the oldest (just under PAINT_SP_SAVE) to
; the newest and copies down only those whose row below or above still holds
; a target pixel inside the run (PAINT_RUN_HAS_TARGET), then moves the saved
; registers and the return address down to the new top.
; -----------------------------------------------------------------------------
PAINT_COMPACT:
    PUSH HL
    PUSH DE
    LD HL,FFEAH                         ; HL = SP - 22
    ADD HL,SP
    LD DE,(PAINT_STACK_MIN)
    SBC HL,DE
    JP NC,PAINT_COMPACT_EXIT            ; room: nothing to do
    PUSH BC
    EXX
    PUSH BC
    LD HL,(PAINT_SP_SAVE)               ; HL = read pointer, DE = write
    DEC HL                              ; pointer, both at the oldest record
    DEC HL
    DEC HL
    LD D,H
    LD E,L

PAINT_COMPACT_LOOP:
    LD C,L
    LD B,H
    LD HL,0009H
    ADD HL,SP
    OR A
    SBC HL,BC                           ; reached the saved registers?
    LD H,B
    LD L,C
    JR NZ,PAINT_COMPACT_RECORD
    LD BC,000AH                         ; move the four saved words and the
    LDDR                                ; return address down
    EX DE,HL
    LD SP,HL
    INC SP                              ; the new stack top
    OR A
    EX DE,HL
    SBC HL,DE
    CCF                                 ; Cy if nothing was dropped
    POP BC
    EXX
    POP BC

PAINT_COMPACT_EXIT:
    POP DE
    POP HL
    RET

PAINT_COMPACT_RECORD:
    CALL PAINT_READ_RECORD_ALT          ; HL = address, DE = width, A = mask
    ADD HL,BC                           ; the row below
    LD BC,BC00H
    SBC HL,BC
    JR NC,PAINT_COMPACT_ABOVE           ; off the picture
    ADD HL,BC
    CALL PAINT_RUN_HAS_TARGET
    JR Z,PAINT_COMPACT_KEEP             ; still work below: keep

PAINT_COMPACT_ABOVE:
    CALL PAINT_READ_RECORD
    OR A
    SBC HL,BC                           ; the row above
    LD BC,8000H
    SBC HL,BC
    ADD HL,BC
    CALL NC,PAINT_RUN_HAS_TARGET

PAINT_COMPACT_KEEP:
    EXX
    LD BC,0006H
    JR NZ,PAINT_COMPACT_DROP
    LDDR                                ; keep: copy the record down
    JP PAINT_COMPACT_LOOP

PAINT_COMPACT_DROP:
    OR A
    SBC HL,BC                           ; drop: skip it
    JP PAINT_COMPACT_LOOP

; Read the record at HL' (top byte). Exit: HL = address, DE = width without
; flags, A = mask, BC = 0040H.

PAINT_READ_RECORD:
    EXX

PAINT_READ_RECORD_ALT:
    PUSH HL
    EXX
    POP HL
    LD D,(HL)                           ; address
    DEC HL
    LD E,(HL)
    DEC HL
    DEC HL                              ; (byte count)
    LD B,(HL)                           ; mask
    DEC HL
    LD A,(HL)                           ; width, flags removed
    AND 3FH
    DEC HL
    LD L,(HL)
    LD H,A
    LD A,B
    LD BC,0040H
    EX DE,HL
    RET

; Z if the DE pixels from HL / mask A include a target pixel.

PAINT_RUN_HAS_TARGET:
    LD C,A
    LD A,(PAINT_TARGET)
    LD B,A

PAINT_RUN_TEST:
    LD A,B
    XOR (HL)
    AND C
    RET Z                               ; found one
    DEC E
    JR Z,PAINT_RUN_COUNT

PAINT_RUN_NEXT:
    RRC C
    JR NC,PAINT_RUN_TEST
    INC HL
    JR PAINT_RUN_TEST

PAINT_RUN_COUNT:
    DEC D
    JP P,PAINT_RUN_NEXT
    RET                                 ; NZ: none

; -----------------------------------------------------------------------------
; VIDEO 11 - DEFINE CHARACTER
; -----------------------------------------------------------------------------
; Copies a 10-byte glyph (rows top to bottom, bit 7 leftmost) to the RAM glyph
; table at 0740H, which holds codes 80H-DFH (0740H-0AFFH).
;
; Entry:   C = code 80H-DFH; DE -> ten bytes.
; Exit:    A = 00H, or F8H for any other code.
; -----------------------------------------------------------------------------
CH_DEF:
    LD A,C
    CP E0H
    JR NC,CH_DEF_FAIL                   ; E0H-FFH (S clear here)
    OR A                                ; S = bit 7

CH_DEF_FAIL:
    LD A,F8H                            ; error F8H: not definable
    RET P                               ; 00H-7FH or E0H-FFH
    RES 7,C
    LD B,00H
    LD L,C
    LD H,B
    ADD HL,HL
    ADD HL,HL
    ADD HL,BC
    ADD HL,HL                           ; 10 * (code AND 7FH)
    LD BC,0740H
    ADD HL,BC
    EX DE,HL                            ; DE -> the glyph slot
    LD BC,000AH
    LDIR
    XOR A
    RET

; -----------------------------------------------------------------------------
; VIDEO 03 - CHARACTER POSITION
; -----------------------------------------------------------------------------
; Moves the beam to the top left of a character cell. Columns and rows count
; from 1; 0 keeps that coordinate. X = (column - 1) * 8 * pixel width,
; Y = 959 - 40 * (row - 1).
;
; Entry:   B = column; C = row.
; Exit:    A = 00H, or F9H if the cell is off the screen.
; -----------------------------------------------------------------------------
CH_POS:
    PUSH BC
    LD HL,0000H
    DEC B
    JP M,CH_POS_X                       ; column 0
    LD L,B                              ; column - 1

CH_POS_X:
    LD A,(COLOR_MODE)
    ADD A,04H
    LD B,A

CH_POS_X_SHIFT:
    ADD HL,HL                           ; * 2 ^ (mode + 4): one cell
    DJNZ CH_POS_X_SHIFT
    PUSH HL
    LD DE,FFD8H                         ; -40 per row
    LD HL,03BFH                         ; row 1 starts at the top, Y = 959
    LD B,C
    DEC B
    JP M,CH_POS_Y                       ; row 0
    JR Z,CH_POS_Y                       ; row 1

CH_POS_Y_STEP:
    ADD HL,DE
    DJNZ CH_POS_Y_STEP

CH_POS_Y:
    EX DE,HL                            ; DE = Y
    POP BC                              ; BC = X
    EXX
    LD BC,(X_PHYSICAL)                  ; the current position, for zeros
    LD DE,(Y_PHYSICAL)
    EXX
    POP HL                              ; H = column, L = row
    LD A,L
    OR A
    JR NZ,CH_POS_KEEP_X
    EXX                                 ; row 0: keep Y
    PUSH DE
    EXX
    POP DE

CH_POS_KEEP_X:
    LD A,H
    OR A
    JR NZ,CH_POS_SET
    EXX                                 ; column 0: keep X
    PUSH BC
    EXX
    POP BC

CH_POS_SET:
    LD A,F9H                            ; error F9H: off the screen
    CALL CHECK_X_COORDINATE
    RET C
    CALL CHECK_Y_COORDINATE
    RET C
    JP PIXEL_ADDRESS

; EDITOR routine jump table; first byte is the routine count, followed by routine addresses.

; -----------------------------------------------------------------------------
; EDITOR DEVICE JUMP TABLE
; -----------------------------------------------------------------------------
;
; Counted table of the five public editor functions.
;
; The count byte is 05H. Entries select ED_INT, ED_CHIN_OUT, ED_BKIN_OUT, CU_POS, and CU_FIX
; respectively. The operating-system function dispatcher uses this table in the same way as the
; VIDEO table, allowing callers to use function numbers instead of hard-coded implementation
; addresses.
;
; Entry:
;   Function number selected by the OS dispatcher.
;
; Exit:
;   A routine address from the table.
;
; Effects:
;   Read-only ROM data.
;
; Destroys:
;   None.
; -----------------------------------------------------------------------------
EDITOR_JUMP_TABLE:
    DB 05H                                ; 5 functions
    DW ED_INT                             ; 0  interrupt: blink the cursor
    DW ED_CHIN_OUT                        ; 1  character in/out
    DW ED_BKIN_OUT                        ; 2  block in/out
    DW CU_POS                             ; 3  position the cursor
    DW CU_FIX                             ; 4  save the cursor position

; Cursor blink phase at 0E48H: 00H..14H shows the saved cell and 80H..94H shows the cursor
; overlay.

; -----------------------------------------------------------------------------
; EDITOR 00 - CURSOR INTERRUPT
; -----------------------------------------------------------------------------
;
; Times the cursor blink and temporarily overlays the cursor glyph without damaging the underlying
; screen.
;
; The interrupt entry increments the cursor phase at 0E48H. Phases 00H..14H leave the saved
; character visible; phases 80H..94H show a cursor glyph. When the cursor is shown, LOCK_KEY
; selects the normal, inverse CTRL, inverse SHIFT, or inverse ALT cursor code (7FH, 9EH, 9FH, or
; 8FH).
;
; At the transition points the routine jumps to CU_SAVE_SCREEN at D420H or CU_RESTORE_SCREEN at
; D491H. Those helpers copy the complete character-shaped video area, not merely an ASCII code, so
; the cursor can overlay arbitrary graphics and restore them exactly.
;
; Entry:
;   No public arguments; invoked by the cursor/sound interrupt service.
;
; Exit:
;   No public return value.
;
; Effects:
;   May save, overwrite, and restore ten raster rows at the current cursor position; updates blink
;   phase.
;
; Destroys:
;   AF, BC, DE, HL, IY according to the save/restore path.
; -----------------------------------------------------------------------------
ED_INT:
    LD A,50H
    OUT (02H),A
    LD HL,CU_BLINK
    LD BC,(CU_Y)
    INC (HL)
    LD A,94H
    SUB (HL)
    JR Z,LCFD0
    CP 80H
    RET NZ
    LD (HL),A
    LD A,(LOCK_KEY)
    LD D,7FH
    OR A
    JR Z,LCFCC
    LD D,9EH
    RRCA
    JR C,LCFCC
    LD D,9FH
    RRCA
    JR C,LCFCC
    LD D,8FH

LCFCC:
    LD A,D
    JP CU_SAVE_SCREEN

LCFD0:
    LD (HL),A
    JP CU_RESTORE_SCREEN

; EDITOR_INIT installs the RAM renderer dispatch at 0E68H, row width at 0E6BH, and character width
; at 0E6CH.

; -----------------------------------------------------------------------------
; EDITOR INITIALIZATION
; -----------------------------------------------------------------------------
;
; Clears the editor workspace, homes the cursor, initializes row metadata, and installs
; mode-specific render parameters.
;
; The routine clears 0E48H..0E67H, which contains cursor state and one occupancy byte for each of
; the 24 character rows. It sets the cursor to row 1, column 1 and fills the ASCII screen buffer
; at 0100H with spaces.
;
; It then indexes EDITOR_MODE_TABLE at D007H using GR_MODE and copies the selected renderer entry,
; row width, and bytes-per-character into the editor work area. The three descriptors are mode 2:
; D3ADH/40 columns/1 byte, mode 4: D3BCH/20 columns/2 bytes, and mode 16: D3D9H/10 columns/4
; bytes. Subsequent editor operations therefore avoid repeated mode tests.
;
; Entry:
;   GR_MODE at 0B73H.
;
; Exit:
;   Cursor and editor workspace reset; 0E68H renderer dispatch and 0E6BH row width are installed.
;
; Effects:
;   Clears the editor ASCII buffer and internal row metadata; does not itself clear video RAM (CLS
;   does that around this call).
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
EDITOR_INIT:
    LD HL,CU_BLINK
    LD B,20H
    XOR A

LCFDA:
    LD (HL),A
    INC HL
    DJNZ LCFDA
    LD HL,0101H
    LD (CU_Y),HL
    LD HL,0100H
    LD DE,0101H
    LD BC,05FFH
    LD (HL),20H
    LDIR
    LD A,(COLOR_MODE)
    ADD A,A
    ADD A,A
    LD HL,EDITOR_MODE_TABLE
    LD C,A
    ADD HL,BC
    LD DE,ED_MODE_VECTOR
    LD A,C3H
    LD (DE),A
    INC DE
    LD C,04H
    LDIR
    RET

; Editor mode descriptor table; each four-byte entry holds routine address, row width, and bits
; per pixel.

; -----------------------------------------------------------------------------
; EDITOR MODE DESCRIPTORS
; -----------------------------------------------------------------------------
;
; Three four-byte descriptors containing renderer address, row width, and video bytes per
; character.
;
; Entries are D3ADH,40H,01H for mode 2; D3BCH,20H,02H for mode 4; and D3D9H,10H,04H for mode 16.
; EDITOR_INIT copies the selected descriptor into RAM so the editor can render and move rows using
; a mode-independent set of helpers.
;
; Entry:
;   Index = GR_MODE (0..2).
;
; Exit:
;   Renderer entry and geometry constants.
;
; Effects:
;   Read-only ROM data.
;
; Destroys:
;   None.
; -----------------------------------------------------------------------------
EDITOR_MODE_TABLE:
    DB ADH, D3H, 40H, 01H, BCH, D3H, 20H, 02H, D9H, D3H, 10H, 04H                     ; |..@... .....|

; CFIX routine (EDITOR 04); notes the current cursor position.
; Current and saved editor cursor positions are kept at 0E49H and 0E4EH as one-based column/row
; pairs.

; -----------------------------------------------------------------------------
; EDITOR 04 - SAVE CURSOR POSITION
; -----------------------------------------------------------------------------
;
; Copies the current cursor position to the saved-position pair and marks it valid for the next
; input operation.
;
; The current row/column pair at 0E49H is copied to 0E4EH. Writing 80H to 0E4DH tells ED_CHIN_OUT
; and ED_BKIN_OUT to begin returning characters at this saved position rather than at the start of
; the paragraph. This permits a caller to place a prompt or other prefix before a subsequent input
; field.
;
; Entry:
;   No public arguments.
;
; Exit:
;   No public return value.
;
; Effects:
;   Updates saved cursor position and input-start state.
;
; Destroys:
;   AF, HL.
; -----------------------------------------------------------------------------
CU_FIX:
    LD HL,(CU_Y)
    LD (CU_SAVED_POS),HL
    LD A,80H
    JR LD039

; -----------------------------------------------------------------------------
; EDITOR 03 - POSITION CURSOR
; -----------------------------------------------------------------------------
;
; Validates and installs a requested editor row and column.
;
; B is the one-based column and C the one-based row. A zero in either register leaves that
; coordinate unchanged. The accepted column range is 1..16, 1..32, or 1..64 depending on mode;
; rows range from 1 to 24. Invalid coordinates return F6H without changing the corresponding valid
; position.
;
; Entry:
;   B = column (mode-dependent); C = row; zero means retain the current coordinate.
;
; Exit:
;   A = 00H on success; A = F6H for an impossible position.
;
; Effects:
;   Updates 0E49H cursor coordinates and clears the cursor-positioning status.
;
; Destroys:
;   AF, HL.
; -----------------------------------------------------------------------------
CU_POS:
    LD HL,(CU_Y)
    LD A,B
    OR A
    JR Z,LD02B
    LD A,(ED_ROW_LEN)
    CP B
    JR C,LD03E
    LD H,B

LD02B:
    LD A,C
    OR A
    JR Z,LD035
    LD A,18H
    CP C
    JR C,LD03E
    LD L,C

LD035:
    LD (CU_Y),HL

LD038:
    XOR A

LD039:
    LD (CU_STATE),A
    XOR A
    RET

LD03E:
    LD A,F6H
    RET

; -----------------------------------------------------------------------------
; EDITOR 02 - BLOCK INPUT/OUTPUT
; -----------------------------------------------------------------------------
;
; Routes counted block transfers through the editor's character input or output path.
;
; The routine prepares INK and PAPER line bytes, selects the generic OUT-CHARS or IN-CHARS
; wrapper, and supplies the editor-specific character routine as the per-byte operation. Output
; stores characters into the editor screen at the cursor; input reads the paragraph represented by
; the editor's ASCII buffer.
;
; The documented return codes are 00H for success and FAH when the transfer crosses the allowed
; high-memory boundary. The reference notes a defect in the generic boundary-checking wrapper, so
; callers should not assume every overrun is recovered cleanly.
;
; Entry:
;   DE = transfer buffer; BC = byte count; OS direction selects input or output.
;
; Exit:
;   A = 00H or FAH/error from the block transfer.
;
; Effects:
;   Reads/writes caller memory and editor ASCII/video state.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
; -----------------------------------------------------------------------------
ED_BKIN_OUT:
    EXX
    CALL EDITOR_COLOR_BYTES
    EXX
    LD HL,LD0B7
    JP P,OUT_CHARS_SAFE
    LD HL,LD058
    JP IN_CHARS_SAFE

; Input-start state at 0E4DH: 00H starts a new edit, 01H requests the next character of a
; completed paragraph, and 80H uses CU_FIX's saved position.

; -----------------------------------------------------------------------------
; EDITOR 01 - CHARACTER INPUT/OUTPUT
; -----------------------------------------------------------------------------
;
; Implements interactive editor input and programmatic character output, including repeat reads
; from a completed paragraph.
;
; For output, the routine treats C as the character to insert and follows the same editor path as
; keyboard input. For input, it validates the saved cursor position, saves the underlying screen
; cell, enables the keyboard interrupt path, waits for a character, restores the screen cell, and
; then dispatches ESC, RETURN, or an ordinary printable/control code.
;
; A completed paragraph can be read back through repeated calls without another key press: the
; routine walks the editor ASCII buffer, returns each character, and finally emits RETURN while
; resetting the input state. ESC invalidates the paragraph and returns 1BH; CTRL+ESC propagates
; the STOP error.
;
; Entry:
;   For output, C = character code. For input, no character argument; the editor and keyboard
;   interrupt state provide it.
;
; Exit:
;   Input returns C = character and A = 00H, or an error code; output returns A = 00H on success.
;
; Effects:
;   Updates editor ASCII/video buffers, cursor state, and keyboard interrupt state.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
; -----------------------------------------------------------------------------
ED_CHIN_OUT:
    CALL EDITOR_COLOR_BYTES
    JP P,LD0B7

LD058:
    LD A,50H
    LD (P_SAVE),A
    OUT (02H),A
    LD A,(CU_STATE)
    RRCA
    JP C,LD0FA

LD066:
    CALL ROW_STATUS_ADDRESS
    LD BC,(CU_Y)
    PUSH BC
    LD DE,(CU_SAVED_POS)

LD072:
    LD A,C
    CP E
    JR C,LD07C
    JR NZ,LD081
    LD A,B
    CP D
    JR NC,LD08B

LD07C:
    CALL LD038
    JR LD08B

LD081:
    DEC C
    JR Z,LD07C
    DEC HL
    BIT 7,(HL)
    LD D,01H
    JR NZ,LD072

LD08B:
    POP BC
    CALL CU_SAVE_CELL
    LD A,13H
    LD (CU_BLINK),A
    LD HL,INT_DES
    RES 2,(HL)
    RST 30H
    DB 91H                                ; RST 30H fn 91H: keyboard 1, input -> KB_CHIN
    SET 2,(HL)
    PUSH AF
    PUSH BC
    LD BC,(CU_Y)
    CALL CU_RESTORE_SCREEN
    POP BC
    POP AF
    RET NZ
    LD A,C
    CP 1BH
    JR Z,LD038
    CP 0DH
    JR Z,LD0EB
    CALL EDITOR_CHAR_DISPATCH
    JR LD066

LD0B7:
    LD A,50H
    LD (P_SAVE),A
    OUT (02H),A
    LD HL,LD038
    PUSH HL
    LD A,C
    CP 0DH
    JR NZ,LD0CD
    LD A,01H
    LD (CU_X),A
    RET

LD0CD:
    CP 0AH
    JR NZ,LD0E6
    LD BC,(CU_Y)

LD0D5:
    LD A,C
    CALL LD39F
    INC C
    JR C,LD0D5
    DEC C
    PUSH BC
    CALL EDITOR_NEXT_ROW
    POP AF
    LD (CU_X),A
    RET

LD0E6:
    CALL EDITOR_CHAR_DISPATCH
    XOR A
    RET

LD0EB:
    LD A,(CU_STATE)
    RLCA
    LD BC,(CU_SAVED_POS)
    CALL NC,EDITOR_PARAGRAPH_START
    LD (CU_Y),BC

LD0FA:
    LD BC,(CU_Y)
    LD A,C
    CALL LD39F
    CP B
    JR C,LD11C
    CALL ASCII_CURSOR_ADDRESS
    INC B
    LD A,(ED_ROW_LEN)
    CP B
    JR NC,LD112
    LD B,01H
    INC C

LD112:
    LD (CU_Y),BC
    LD C,(HL)
    LD A,01H
    JP LD039

LD11C:
    CALL EDITOR_NEXT_ROW
    LD C,0DH
    JP LD038

; Editor state: 0E50H..0E67H contains 24 row occupancy bytes; low seven bits hold character count
; and bit 7 marks a full row. The ASCII screen itself is 24 rows of 40 bytes at 0100H.

; -----------------------------------------------------------------------------
; EDITOR CHARACTER AND COMMAND DISPATCH
; -----------------------------------------------------------------------------
;
; Stores printable characters in the ASCII screen, renders them, advances the cursor, and
; dispatches eleven editor commands.
;
; Codes 20H..DFH are written to the current ASCII-buffer position, rendered through the
; mode-specific routine at 0E68H, and followed by cursor advancement. When a row becomes full, the
; routine creates or shifts an editor row and preserves the row occupancy metadata.
;
; Control codes and codes above DFH are looked up in the table at D170H. The table maps LEFT,
; RIGHT, UP, DOWN, INS, DC, DEL, DL, IL, CEL, and TAB to their shared entry points. Each command
; maintains the paragraph model: row bytes at 0E50H..0E67H hold character counts, with bit 7
; marking a full row.
;
; Entry:
;   A = character or editor command code; 0E49H = current cursor position.
;
; Exit:
;   Editor buffers, display, and cursor reflect the character or command; flags indicate
;   command-specific outcomes.
;
; Effects:
;   Can move large portions of both the 0100H ASCII screen and video display; may insert/delete
;   rows and alter saved cursor state.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers, and stack temporaries.
;
; Note:
;   This is the main editor engine. D157H is its command-table search continuation; D191H-D1A3H
;   share the four cursor-arrow entry points.
; -----------------------------------------------------------------------------
EDITOR_CHAR_DISPATCH:
    CP 20H
    JR C,LD157
    CP E0H
    JR NC,LD157
    EX AF,AF'
    CALL ASCII_CURSOR_ADDRESS
    EX AF,AF'
    LD (HL),A
    LD BC,(CU_Y)
    PUSH BC
    CALL CU_SAVE_SCREEN
    POP BC
    LD A,C
    CALL LD39F
    CP B
    JR NC,LD143
    LD (HL),B

LD143:
    INC B
    LD A,(ED_ROW_LEN)
    CP B
    JR NC,LD152
    LD B,01H
    INC C
    BIT 7,(HL)
    CALL Z,EDITOR_INSERT_ROW

LD152:
    LD (CU_Y),BC
    RET

LD157:
    LD HL,EDITOR_COMMAND_TABLE
    LD B,0BH

LD15C:
    CP (HL)
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    JR Z,LD167
    DJNZ LD15C
    RET

LD167:
    LD BC,(CU_Y)
    LD A,(ED_ROW_LEN)
    EX DE,HL
    JP (HL)

; Built-in editor function jump table; entries contain control character and routine address.
; Control-code table: LEFT=13H, RIGHT=04H, UP=05H, DOWN=18H, INS=16H, DC=07H, DEL=08H, DL=19H,
; IL=0EH, CEL=0BH, TAB=09H.

; -----------------------------------------------------------------------------
; EDITOR COMMAND TABLE
; -----------------------------------------------------------------------------
;
; Eleven control-code/address pairs used by EDITOR_CHAR_DISPATCH.
;
; The entries select LEFT (13H), RIGHT (04H), UP (05H), DOWN (18H), INS (16H), DC (07H), DEL
; (08H), DL (19H), IL (0EH), CEL (0BH), and TAB (09H). Each pair stores the control code followed
; by a little-endian routine address.
;
; Entry:
;   A = editor control code.
;
; Exit:
;   Matching command entry address, or no action if the code is absent.
;
; Effects:
;   Read-only ROM data.
;
; Destroys:
;   None.
; -----------------------------------------------------------------------------
EDITOR_COMMAND_TABLE:
    DB 13H                                ; cursor left
    DW EDITOR_CURSOR_ARROWS
    DB 04H                                ; cursor right
    DW LD198
    DB 05H                                ; cursor up
    DW LD195
    DB 18H                                ; cursor down
    DW LD19E
    DB 16H                                ; INS
    DW EDITOR_INSERT_CHAR
    DB 07H                                ; SHIFT+DEL
    DW EDITOR_DELETE_AT
    DB 08H                                ; DEL
    DW EDITOR_DELETE_BEFORE
    DB 19H                                ; CTRL+Y
    DW LD1FD
    DB 0EH                                ; CTRL+N
    DW LD1F7
    DB 0BH                                ; CTRL+K
    DW EDITOR_CLEAR_TO_END
    DB 09H                                ; CTRL+I (TAB)
    DW EDITOR_TAB

; -----------------------------------------------------------------------------
; EDITOR CURSOR ARROW HANDLER
; -----------------------------------------------------------------------------
;
; Shared four-entry handler for LEFT, UP, RIGHT, and DOWN cursor movement.
;
; The four command-table targets are arranged to fall through one another. LEFT decrements the
; column and, at column zero, falls through to UP; UP decrements the row; RIGHT increments the
; column and, at the row end, falls through to DOWN; DOWN increments the row but refuses to pass
; row 24. The resulting coordinates are stored at 0E49H.
;
; Entry:
;   BC = current one-based column/row; entry point identifies direction; 0E6BH = active row width.
;
; Exit:
;   BC and 0E49H contain the accepted cursor position.
;
; Effects:
;   Updates cursor coordinates only; boundary moves are ignored.
;
; Destroys:
;   AF, BC.
; -----------------------------------------------------------------------------
EDITOR_CURSOR_ARROWS:
    DEC B
    JR NZ,LD1A3
    LD B,A

LD195:
    DEC C
    JR LD1A2

LD198:
    INC B
    CP B
    JR NC,LD1A3
    LD B,01H

LD19E:
    INC C
    LD A,C
    CP 19H

LD1A2:
    RET Z

LD1A3:
    LD (CU_Y),BC
    RET

; -----------------------------------------------------------------------------
; CEL - CLEAR TO PARAGRAPH END
; -----------------------------------------------------------------------------
;
; Deletes the text from the cursor through the end of its paragraph, including any now-empty rows.
;
; The current row is cleared from the cursor to its recorded end, the matching ASCII-buffer bytes
; are blanked, and the row occupancy byte is corrected. Following rows are removed with the
; row-delete helper until a non-full row marks the paragraph boundary.
;
; Entry:
;   Current cursor at 0E49H; row metadata at 0E50H..0E67H.
;
; Exit:
;   Paragraph tail is blank; cursor remains at its original position.
;
; Effects:
;   Writes video RAM and ASCII buffer; may delete multiple rows.
;
; Destroys:
;   AF, BC, DE, HL, stack temporaries.
; -----------------------------------------------------------------------------
EDITOR_CLEAR_TO_END:
    PUSH BC
    CALL CU_CLEAR_ROW_TAIL
    POP BC
    CALL ASCII_CURSOR_ADDRESS
    DEC B
    LD A,(ED_ROW_LEN)
    SUB B

LD1B5:
    LD (HL),20H
    INC HL
    DEC A
    JR NZ,LD1B5
    CALL ROW_STATUS_ADDRESS
    LD (HL),B
    INC C
    INC HL

LD1C1:
    RET NC

LD1C2:
    LD B,(HL)
    LD A,C
    EXX
    CALL EDITOR_DELETE_ROW
    EXX
    RLC B
    JR LD1C1

; -----------------------------------------------------------------------------
; TAB - ADVANCE TO NEXT TAB STOP
; -----------------------------------------------------------------------------
;
; Moves the cursor to the next tab position or starts a new row when the current row has no room.
;
; The current column is rounded up to the next multiple-of-eight tab stop in zero-based
; coordinates. If that position exceeds the active row width, the row is marked full and the
; cursor advances to the next row; otherwise the row occupancy metadata is extended when necessary
; and the new position is stored.
;
; Entry:
;   Current cursor and row occupancy metadata.
;
; Exit:
;   Cursor moves to the next tab stop; row state is updated.
;
; Effects:
;   May create a new editor row and alter ASCII/video buffers through row helpers.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
EDITOR_TAB:
    CALL ROW_STATUS_ADDRESS
    EX AF,AF'
    LD A,B
    DEC A
    AND F8H
    ADD A,09H
    LD B,A
    LD A,(ED_ROW_LEN)
    CP B
    JR NC,LD1E8
    OR 80H
    LD (HL),A
    LD B,01H
    INC C
    EX AF,AF'
    CALL NC,EDITOR_INSERT_ROW

LD1E8:
    LD A,C
    CALL LD39F
    DEC B
    CP B
    JR NC,LD1F1
    LD (HL),B

LD1F1:
    INC B
    LD (CU_Y),BC
    RET

LD1F7:
    CALL EDITOR_PARAGRAPH_START
    JP LD2D6

LD1FD:
    CALL EDITOR_PARAGRAPH_START
    CALL ROW_STATUS_ADDRESS
    JR LD1C2

; -----------------------------------------------------------------------------
; INS - INSERT CHARACTER SPACE
; -----------------------------------------------------------------------------
;
; Inserts one character position at the cursor and carries overflow through the rest of the
; paragraph.
;
; INS scans row occupancy to find the paragraph end, computes how many rows must shift, and
; inserts space by moving ASCII and video content from the bottom upward. The last row may be
; discarded only when the paragraph already occupies the full screen; carry reports that loss
; condition.
;
; On the final inserted row the cursor column is used as the insertion point; earlier rows begin
; at column one. Full-row markers, row counts, and the saved cursor position are adjusted while
; the character data propagates.
;
; Entry:
;   Current cursor position and row metadata.
;
; Exit:
;   A new blank character position exists at the cursor; carry signals a full-screen overflow.
;
; Effects:
;   Moves large ASCII and video ranges and may insert a display row.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers, stack temporaries.
; -----------------------------------------------------------------------------
EDITOR_INSERT_CHAR:
    CALL ROW_STATUS_ADDRESS
    CP B
    RET C
    LD E,01H
    JR LD210

LD20E:
    INC HL
    INC E

LD210:
    LD A,(HL)
    RLCA
    INC C
    JR C,LD20E
    INC (HL)
    LD A,(ED_ROW_LEN)
    CP (HL)
    JR NZ,LD233
    LD A,E
    ADD A,C
    CP 31H
    JR NZ,LD226
    DEC (HL)
    SCF
    JR LD234

LD226:
    PUSH DE
    PUSH BC
    CALL EDITOR_INSERT_ROW
    POP AF
    POP DE
    LD B,A
    LD A,C
    DEC A
    CALL LD39F

LD233:
    OR A

LD234:
    EX AF,AF'
    OR A
    DEC C
    DEC E
    PUSH DE
    PUSH BC
    PUSH HL
    PUSH AF
    JR Z,LD240
    LD B,01H

LD240:
    PUSH BC
    CALL EDITOR_SHIFT_RIGHT
    POP BC
    LD DE,ED_ROW_LEN
    LD A,(DE)
    LD H,A
    LD L,C
    CALL LD387
    PUSH HL

    EX AF,AF'
    JR NC,LD254
    DEC HL
    INC B

LD254:
    LD A,(DE)
    SUB B
    LD C,A
    LD B,00H
    LD D,H
    LD E,L
    DEC HL
    JR Z,LD260
    LDDR

LD260:
    POP HL
    LD BC,FFC0H
    ADD HL,BC
    LD B,(HL)
    POP AF
    JR NZ,LD26B
    LD B,20H

LD26B:
    LD A,B
    LD (DE),A
    POP HL
    POP BC
    POP DE
    DEC HL
    JR NZ,LD234
    LD (CU_Y),BC
    RET

; -----------------------------------------------------------------------------
; DEL - DELETE BEFORE CURSOR
; -----------------------------------------------------------------------------
;
; Moves the cursor left, then removes the character now under it.
;
; The compact entry decrements the column; at column zero it moves to the previous row's last
; occupied position when that row belongs to the paragraph. It then falls through to
; EDITOR_DELETE_AT, so DEL and DC share the same shift-left implementation.
;
; Entry:
;   Current cursor position and row occupancy metadata.
;
; Exit:
;   Cursor is moved left when possible and the character at the resulting position is removed.
;
; Effects:
;   May shift paragraph text and video contents left.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
EDITOR_DELETE_BEFORE:
    DEC B
    JR NZ,LD283
    DEC C
    RET Z
    LD A,C
    CALL LD39F
    RET NC
    LD B,A

LD283:
    LD (CU_Y),BC

; -----------------------------------------------------------------------------
; DC - DELETE AT CURSOR
; -----------------------------------------------------------------------------
;
; Deletes the character at the cursor and shifts the remainder of the paragraph left across row
; boundaries.
;
; If the cursor is already at paragraph end there is no work. Otherwise the routine computes the
; number of bytes from the deletion point to the row end, shifts the following characters left,
; and pulls the first character from the next full row into the vacated last position. It
; continues until a non-full row is reached, then clears the final spare character and updates row
; metadata.
;
; Entry:
;   Current cursor, row metadata, and editor buffers.
;
; Exit:
;   The character at the cursor is gone; the remaining paragraph is contiguous.
;
; Effects:
;   Writes video RAM and ASCII buffer; may delete a row.
;
; Destroys:
;   AF, BC, DE, HL, stack temporaries.
; -----------------------------------------------------------------------------
EDITOR_DELETE_AT:
    CALL ROW_STATUS_ADDRESS
    CP B
    RET C

LD28C:
    EX AF,AF'
    LD A,(HL)
    AND 7FH
    SUB B
    JR C,LD2C3
    PUSH HL
    PUSH AF
    PUSH DE
    PUSH BC
    CALL EDITOR_SHIFT_LEFT
    POP BC
    POP DE
    LD H,B
    LD L,C
    CALL LD387
    EX AF,AF'
    LD A,(HL)
    JR NC,LD2A6
    LD (DE),A

LD2A6:
    LD D,H
    LD E,L
    INC HL
    POP AF
    PUSH BC
    JR Z,LD2B2
    LD C,A
    LD B,00H
    LDIR

LD2B2:
    LD A,20H
    LD (DE),A
    POP BC
    INC C
    LD B,01H
    POP HL
    BIT 7,(HL)
    INC HL
    SCF
    JR NZ,LD28C
    DEC HL
    DEC (HL)
    RET

LD2C3:
    DEC HL
    DEC (HL)
    RES 7,(HL)
    LD A,C
    JP EDITOR_DELETE_ROW

; -----------------------------------------------------------------------------
; INSERT EDITOR ROW
; -----------------------------------------------------------------------------
;
; Makes a blank row at the requested position and rolls lower rows down when necessary.
;
; The requested row is marked occupied, then the next row is inspected. If the display bottom is
; reached, the routine performs a screen roll; otherwise it shifts lower video rows, row metadata,
; and the corresponding 40H-byte ASCII rows downward. The new row is cleared to spaces and any
; saved cursor row below the insertion is incremented.
;
; Entry:
;   HL = row-metadata address; C = one-based row number to make available.
;
; Exit:
;   A blank row exists at row C; metadata and saved cursor state are consistent.
;
; Effects:
;   Moves screen and ASCII-buffer rows; may discard the bottom row.
;
; Destroys:
;   AF, BC, DE, HL, stack temporaries.
; -----------------------------------------------------------------------------
EDITOR_INSERT_ROW:
    SET 7,(HL)
    LD A,C
    CP 19H
    JR Z,EDITOR_ROLL_SCREEN
    INC HL
    LD A,(HL)
    OR A
    RET Z

LD2D6:
    PUSH BC
    CALL EDITOR_INSERT_ROW_VIDEO
    POP BC
    PUSH BC
    LD A,19H
    SUB C
    LD HL,ED_ROW_DESC+22
    LD DE,ED_ROW_DESC+23
    PUSH DE
    LD B,00H
    LD C,A
    LDDR
    INC DE
    EX DE,HL
    LD (HL),C
    POP HL
    RES 7,(HL)
    RRA
    RR C
    RRA
    RR C
    LD B,A
    LD HL,06BFH
    LD DE,06FFH
    LDDR
    LD B,40H
    LD A,20H

LD304:
    INC DE
    LD (DE),A
    DJNZ LD304
    POP BC
    LD HL,CU_SAVED_POS
    LD A,(HL)
    SUB C
    RET C
    INC (HL)
    RET

; -----------------------------------------------------------------------------
; ROLL EDITOR SCREEN UP
; -----------------------------------------------------------------------------
;
; Deletes the first display row, shifts all following rows upward, and homes the cursor on the
; last row.
;
; This is the full-screen form of row deletion. It delegates to EDITOR_DELETE_ROW with row 1, then
; places the cursor at row 24, column 1. It is used when insertion reaches the bottom of a full
; display.
;
; Entry:
;   Editor row state.
;
; Exit:
;   Rows have moved up; cursor is at the first column of row 24.
;
; Effects:
;   Moves video and ASCII rows and discards the old top row.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
EDITOR_ROLL_SCREEN:
    LD A,01H
    CALL EDITOR_DELETE_ROW
    LD BC,0118H
    LD (CU_Y),BC
    RET

; -----------------------------------------------------------------------------
; DELETE EDITOR ROW
; -----------------------------------------------------------------------------
;
; Removes the selected row and shifts all lower rows upward in both representations of the screen.
;
; The routine adjusts or invalidates the saved cursor row, moves video rows and 40H-byte ASCII
; rows upward, clears the final row to spaces, and shifts row occupancy bytes. It can begin at any
; row, making it the common primitive for CEL, DC, insert overflow handling, and screen roll.
;
; Entry:
;   A = one-based row number to delete.
;
; Exit:
;   Rows below A occupy their preceding positions; the final row is blank.
;
; Effects:
;   Writes video RAM, ASCII buffer, row metadata, and saved cursor state.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
EDITOR_DELETE_ROW:
    PUSH AF
    LD C,A
    LD HL,CU_SAVED_POS
    SUB (HL)
    CALL Z,LD039
    JR NC,LD32A
    DEC (HL)

LD32A:
    CALL EDITOR_DELETE_ROW_VIDEO
    POP BC
    LD A,B
    CALL LD39F
    PUSH HL
    LD L,B
    LD H,01H
    CALL LD387
    EX DE,HL
    LD HL,0040H
    ADD HL,DE
    LD A,18H
    SUB B
    PUSH AF
    JR Z,LD34F
    LD C,00H
    RRA
    RR C
    RRA
    RR C
    LD B,A
    LDIR

LD34F:
    LD B,40H

LD351:
    DEC HL
    LD (HL),20H
    DJNZ LD351
    POP AF
    LD C,A
    POP HL
    LD D,H
    LD E,L
    INC HL
    JR Z,LD360
    LDIR

LD360:
    XOR A
    LD (DE),A
    RET

; -----------------------------------------------------------------------------
; MOVE CURSOR TO NEXT ROW
; -----------------------------------------------------------------------------
;
; Advances the cursor to column one of the next row, inserting a row if the display is full.
;
; At ordinary rows it increments C, sets B=1, and stores the position. At row 24 it falls through
; to the screen-roll path so typing can continue while preserving the editor's bottom-of-screen
; behavior.
;
; Entry:
;   Current cursor row in C.
;
; Exit:
;   Cursor is at column one of the following row.
;
; Effects:
;   May roll the screen and update buffers.
;
; Destroys:
;   AF, BC.
; -----------------------------------------------------------------------------
EDITOR_NEXT_ROW:
    LD A,C
    CP 18H
    JR Z,EDITOR_ROLL_SCREEN
    INC C
    LD B,01H
    LD (CU_Y),BC
    RET

; -----------------------------------------------------------------------------
; FIND PARAGRAPH START
; -----------------------------------------------------------------------------
;
; Moves the cursor to the first row of the paragraph containing it.
;
; The routine walks upward through row occupancy bytes until it finds an empty row or the top of
; the display. It then stores column one of the first occupied row as the cursor position.
; Paragraph boundaries are therefore defined by blank rows, not by explicit delimiters.
;
; Entry:
;   Current cursor position and row occupancy metadata.
;
; Exit:
;   0E49H points to column one of the containing paragraph's first row.
;
; Effects:
;   Updates cursor position only.
;
; Destroys:
;   AF, BC, HL.
; -----------------------------------------------------------------------------
EDITOR_PARAGRAPH_START:
    LD BC,(CU_Y)

LD374:
    OR A
    DEC C
    LD A,C
    CALL NZ,LD39F
    JR C,LD374
    INC C
    LD B,01H
    LD (CU_Y),BC
    RET

; -----------------------------------------------------------------------------
; CURSOR TO ASCII BUFFER ADDRESS
; -----------------------------------------------------------------------------
;
; Converts the one-based editor cursor position into an address in the 0100H ASCII screen.
;
; The row is multiplied by 40H and the column is added after converting both coordinates to
; zero-based values. The resulting address is stored in 0E4BH-0E4CH and returned in HL. This is
; the central bridge between cursor operations and the editor's logical screen.
;
; Entry:
;   0E49H = one-based cursor row/column.
;
; Exit:
;   HL and 0E4BH-0E4CH = address of the cursor's ASCII byte.
;
; Effects:
;   Updates the cached ASCII address.
;
; Destroys:
;   AF, HL.
; -----------------------------------------------------------------------------
ASCII_CURSOR_ADDRESS:
    LD HL,(CU_Y)

LD387:
    DEC H
    DEC L
    LD A,H
    ADD A,A
    ADD A,A
    LD H,L
    RR H
    RRA
    RR H
    RRA
    LD L,A
    LD A,H
    ADD A,01H
    LD H,A
    LD (ED_LINE_PTR),HL
    RET

; -----------------------------------------------------------------------------
; ROW STATUS LOOKUP
; -----------------------------------------------------------------------------
;
; Returns the occupancy byte and its address for the cursor's current row.
;
; The row number in 0E49H is converted to an address in 0E50H..0E67H. The returned A value has bit
; 7 normalized away as the character count; carry preserves whether the row was full.
;
; Entry:
;   A or 0E49H = current one-based row.
;
; Exit:
;   HL = row-status byte; A = count with bit 7 clear; carry = row-full flag.
;
; Effects:
;   No writes.
;
; Destroys:
;   AF, HL.
; -----------------------------------------------------------------------------
ROW_STATUS_ADDRESS:
    LD A,(CU_Y)

LD39F:
    ADD A,4FH
    LD L,A
    LD A,0EH
    ADC A,00H
    LD H,A
    LD A,(HL)
    RLCA
    OR A
    RR A
    RET

; Editor renderers write ten raster rows. Their widths are one, two, and four video bytes for
; modes 2, 4, and 16.

; -----------------------------------------------------------------------------
; EDITOR GLYPH RENDERER - MODE 2
; -----------------------------------------------------------------------------
;
; Renders one ten-row glyph using one video byte per character row.
;
; The renderer reads each glyph byte, combines it with the INK/PAPER line bytes, and writes one
; byte at each of ten video rows separated by 40H. Its geometry is 40 characters per row and one
; packed video byte per character.
;
; Entry:
;   HL = glyph source; DE/HL and B/C = editor-selected destination and colors.
;
; Exit:
;   One glyph is present in video RAM.
;
; Effects:
;   Writes ten video bytes.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
; -----------------------------------------------------------------------------
EDITOR_RENDER_MODE2:
    LD DE,0040H
    EXX

LD3B1:
    LD A,(HL)
    EXX
    AND C
    XOR B
    LD (HL),A
    ADD HL,DE
    EXX
    INC HL
    DJNZ LD3B1
    RET

; -----------------------------------------------------------------------------
; EDITOR GLYPH RENDERER - MODE 4
; -----------------------------------------------------------------------------
;
; Renders one glyph in four-color mode, packing two logical pixels per video byte.
;
; The renderer transforms each pair of glyph pixels into the two-bit color representation required
; by mode 4 and writes two adjacent bytes per raster row, for ten rows. It is selected through the
; descriptor copied by EDITOR_INIT.
;
; Entry:
;   Glyph source and editor color work state.
;
; Exit:
;   One mode-4 glyph is present in video RAM.
;
; Effects:
;   Writes twenty video bytes.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
; -----------------------------------------------------------------------------
EDITOR_RENDER_MODE4:
    EXX

LD3BD:
    LD A,(HL)
    EXX
    LD D,A
    RRCA
    RRCA
    RRCA
    RRCA
    XOR D
    AND C
    LD E,A
    AND 0FH
    XOR D
    AND C
    XOR B
    LD (HL),A
    INC L
    XOR E
    LD (HL),A
    LD DE,003FH
    ADD HL,DE
    EXX
    INC HL
    DJNZ LD3BD
    RET

; -----------------------------------------------------------------------------
; EDITOR GLYPH RENDERER - MODE 16
; -----------------------------------------------------------------------------
;
; Renders one glyph in sixteen-color mode, packing one two-bit glyph pair into each of four bytes
; per raster row.
;
; The mode-16 renderer expands each glyph row into four video bytes using the INK/PAPER pair and
; the TVC two-bit pixel encoding. Ten raster rows are written with a four-byte character width,
; giving ten characters per display row.
;
; Entry:
;   Glyph source and editor color work state.
;
; Exit:
;   One mode-16 glyph is present in video RAM.
;
; Effects:
;   Writes forty video bytes.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
; -----------------------------------------------------------------------------
EDITOR_RENDER_MODE16:
    EXX

LD3DA:
    LD A,(HL)
    EXX
    RLA
    LD E,A
    SBC A,A
    LD D,A
    RL E
    SBC A,A
    XOR D
    AND 55H
    XOR D
    AND C
    XOR B
    LD (HL),A
    INC L
    RL E
    SBC A,A
    LD D,A
    RL E
    SBC A,A
    XOR D
    AND 55H
    XOR D
    AND C
    XOR B
    LD (HL),A
    INC L
    RL E
    SBC A,A
    LD D,A
    RL E
    SBC A,A
    XOR D
    AND 55H
    XOR D
    AND C
    XOR B
    LD (HL),A
    INC L
    RL E
    SBC A,A
    LD D,A
    RL E
    SBC A,A
    XOR D
    AND 55H
    XOR D
    AND C
    XOR B
    LD (HL),A
    LD DE,003DH
    ADD HL,DE
    EXX
    INC HL
    DJNZ LD3DA
    RET

; The cursor backup buffer at 0E6DH..0E94H holds the raw video cell under the cursor.

; -----------------------------------------------------------------------------
; SAVE CHARACTER CELL UNDER CURSOR
; -----------------------------------------------------------------------------
;
; Copies the complete video character cell under the cursor to the cursor backup buffer.
;
; The routine derives the cursor's video address, selects the active character width (1, 2, or 4
; bytes), and copies ten raster rows into 0E6DH..0E94H. Saving raw video bytes preserves graphics
; or unusual colors that would be lost if the cursor remembered only an ASCII code.
;
; Entry:
;   A = cursor glyph code for subsequent rendering; BC = cursor position.
;
; Exit:
;   0E6DH..0E94H contains the original ten-row cell.
;
; Effects:
;   Reads video RAM and writes the cursor backup buffer.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
; -----------------------------------------------------------------------------
CU_SAVE_SCREEN:
    EX AF,AF'
    CALL EDITOR_VIDEO_ADDRESS
    PUSH HL
    EX AF,AF'
    LD C,A
    LD B,00H
    LD L,C
    LD H,B
    ADD HL,HL
    ADD HL,HL
    ADD HL,BC
    ADD HL,HL
    BIT 7,C
    LD BC,C474H
    JR Z,LD439
    LD BC,0240H

LD439:
    ADD HL,BC
    LD B,0AH
    EXX
    LD A,(CU_PAPER_LINE)
    LD B,A
    LD A,(CU_INK_LINE)
    LD C,A
    POP HL
    JP ED_MODE_VECTOR

; -----------------------------------------------------------------------------
; EDITOR INK/PAPER BYTE PREPARATION
; -----------------------------------------------------------------------------
;
; Derives the mode-specific INK and PAPER line bytes used by the editor renderers.
;
; The helper calls the video color conversion routines, stores PAPER at 0E96H, and stores INK XOR
; PAPER at 0E95H. The XOR form makes the renderer's pairwise pixel operations compact while
; retaining the displayed INK/PAPER colors.
;
; Entry:
;   Current video color variables and GR_MODE.
;
; Exit:
;   0E95H = transformed INK byte; 0E96H = PAPER byte.
;
; Effects:
;   Updates editor color work variables.
;
; Destroys:
;   AF, BC.
; -----------------------------------------------------------------------------
EDITOR_COLOR_BYTES:
    EX AF,AF'
    CALL PAPER_BYTE
    LD (CU_PAPER_LINE),A
    CALL INK_BYTE
    XOR B
    LD (CU_INK_LINE),A
    EX AF,AF'
    RET

; Video cell address formula: 8000H + (row*10)*40H + column*character_width.

; -----------------------------------------------------------------------------
; EDITOR CURSOR TO VIDEO ADDRESS
; -----------------------------------------------------------------------------
;
; Converts an editor row/column pair into the first video byte of its character cell.
;
; The column is zero-based and scaled by one, two, or four according to mode. The row is
; multiplied by ten raster lines and by 40H bytes per raster line; the base 8000H is folded in
; through the carry before the final address is assembled. The result is the cell's first video
; byte.
;
; Entry:
;   BC = one-based editor column/row; 0B73H = graphics mode.
;
; Exit:
;   HL = first video byte of the character cell.
;
; Effects:
;   No writes.
;
; Destroys:
;   AF, DE, HL.
; -----------------------------------------------------------------------------
EDITOR_VIDEO_ADDRESS:
    DEC B
    LD A,(COLOR_MODE)
    OR A
    JR Z,LD465

LD460:
    SLA B
    DEC A
    JR NZ,LD460

LD465:
    DEC C
    LD A,C
    ADD A,A
    ADD A,A
    ADD A,C
    ADD A,A
    LD H,A
    XOR A
    SRL H
    RRA
    SCF
    RR H
    RRA
    OR B
    LD L,A
    RET

; -----------------------------------------------------------------------------
; SAVE CURSOR CELL
; -----------------------------------------------------------------------------
;
; Copies the ten-raster-row cell at the cursor into the editor backup area.
;
; CU_SAVE_CELL calls EDITOR_VIDEO_ADDRESS, then copies the active character width from each of ten
; raster rows to 0E6DH. Its raw-cell strategy keeps the cursor overlay reversible even when the
; underlying cell is not text.
;
; Entry:
;   BC = cursor position; editor mode descriptor active.
;
; Exit:
;   Backup buffer contains the original cell.
;
; Effects:
;   Reads video RAM and writes 0E6DH..0E94H.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
CU_SAVE_CELL:
    CALL EDITOR_VIDEO_ADDRESS
    LD DE,CU_SAVE_AREA
    LD A,(ED_CHAR_WIDTH)
    LD C,A
    LD B,0AH

LD483:
    PUSH BC
    PUSH HL
    LD B,00H
    LDIR
    POP HL
    LD C,40H
    ADD HL,BC
    POP BC
    DJNZ LD483
    RET

; -----------------------------------------------------------------------------
; RESTORE CELL UNDER CURSOR
; -----------------------------------------------------------------------------
;
; Restores the saved ten-row cursor cell to video RAM after the cursor overlay is removed.
;
; The inverse of CU_SAVE_CELL: it derives the current cell address and copies the saved width
; bytes from 0E6DH..0E94H back into ten raster rows. ED_INT uses it at the end of the visible
; cursor phase.
;
; Entry:
;   BC = cursor position; backup buffer populated by CU_SAVE_CELL.
;
; Exit:
;   Underlying video cell is restored exactly.
;
; Effects:
;   Writes video RAM.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
CU_RESTORE_SCREEN:
    CALL EDITOR_VIDEO_ADDRESS
    LD DE,CU_SAVE_AREA
    LD A,(ED_CHAR_WIDTH)
    LD C,A
    LD B,0AH

LD49D:
    EX DE,HL
    PUSH BC
    PUSH DE
    LD B,00H
    LDIR
    EX DE,HL
    POP HL
    LD C,40H
    ADD HL,BC
    POP BC
    DJNZ LD49D
    RET

; -----------------------------------------------------------------------------
; CLEAR CHARACTER ROW TAIL
; -----------------------------------------------------------------------------
;
; Clears the cursor row from a selected character position through its right edge.
;
; The routine computes the number of character-width bytes remaining in the row, fills ten raster
; rows with the PAPER byte, and leaves the cursor's row visually blank to the right. It is used by
; insertion and deletion operations before the corresponding ASCII-buffer shift.
;
; Entry:
;   HL = cursor cell video address; editor mode and PAPER byte active.
;
; Exit:
;   The row tail is filled with PAPER.
;
; Effects:
;   Writes up to ten raster rows in video RAM.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
CU_CLEAR_ROW_TAIL:
    CALL EDITOR_VIDEO_ADDRESS
    LD A,(CU_PAPER_LINE)
    EX AF,AF'
    LD A,3FH
    SUB L
    AND 3FH
    JR Z,CU_CLEAR_CELL
    LD C,A
    LD A,L
    LD B,0AH

LD4BF:
    AND 3FH
    OR L
    LD L,A
    EX AF,AF'
    LD (HL),A
    LD D,H
    LD E,L
    INC E
    PUSH BC
    LD B,00H
    LDIR
    POP BC
    EX DE,HL
    EX AF,AF'
    DJNZ LD4BF
    RET

; -----------------------------------------------------------------------------
; CLEAR ONE CHARACTER CELL
; -----------------------------------------------------------------------------
;
; Fills one ten-raster-row character cell with the current PAPER byte.
;
; HL identifies the first video byte of the cell. The active character width determines how many
; adjacent bytes are filled on each of ten raster rows, with a 40H step between rows.
;
; Entry:
;   HL = cell start address.
;
; Exit:
;   Cell is filled with PAPER.
;
; Effects:
;   Writes video RAM.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
CU_CLEAR_CELL:
    LD DE,0040H
    LD B,0AH
    LD A,(CU_PAPER_LINE)

LD4DB:
    LD (HL),A
    ADD HL,DE
    DJNZ LD4DB
    RET

; Video row deletion moves the rows below the selected row upward, then fills the bottom
; ten-raster-row strip with PAPER.

; -----------------------------------------------------------------------------
; MOVE VIDEO ROWS UP
; -----------------------------------------------------------------------------
;
; Deletes a character row from the display by copying all following rows upward and clearing the
; bottom row.
;
; C is the one-based row. If it is row 24, only that row is cleared. Otherwise the routine
; computes the byte span from the next row through the final row and performs one bulk move, then
; fills the final ten-raster-row strip with PAPER.
;
; Entry:
;   C = character row 1..24.
;
; Exit:
;   Rows below C occupy the preceding row; the bottom row is blank.
;
; Effects:
;   Moves and clears video RAM.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
EDITOR_DELETE_ROW_VIDEO:
    LD A,C
    CP 18H
    JR Z,LD4F9
    LD B,01H
    CALL EDITOR_VIDEO_ADDRESS
    EX DE,HL
    LD HL,B980H
    OR A
    SBC HL,DE
    LD B,H
    LD C,L
    LD HL,0280H
    ADD HL,DE
    LDIR

LD4F9:
    LD HL,B980H
    LD DE,B981H

LD4FF:
    LD BC,027FH
    LD A,(CU_PAPER_LINE)
    LD (HL),A
    LDIR
    RET

; Video row insertion moves lower rows downward and clears the newly available ten-raster-row
; strip.

; -----------------------------------------------------------------------------
; MOVE VIDEO ROWS DOWN
; -----------------------------------------------------------------------------
;
; Inserts a blank character row on the display by shifting lower rows down.
;
; The routine computes the number of rows from C to the bottom, multiplies by ten raster lines and
; 40H bytes per line, moves the block backward, and clears the newly available row. Row 24 is
; simply cleared because there is no lower row to preserve.
;
; Entry:
;   C = character row 1..24.
;
; Exit:
;   A blank row exists at C; lower rows have moved down.
;
; Effects:
;   Moves and clears video RAM.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
EDITOR_INSERT_ROW_VIDEO:
    LD A,18H
    SUB C
    JR Z,LD4F9
    LD C,A
    ADD A,A
    ADD A,A
    ADD A,C
    ADD A,A
    LD B,A
    XOR A
    SRL B
    RRA
    SRL B
    RRA
    LD C,A
    LD HL,B97FH
    LD DE,BBFFH
    LDDR
    INC HL
    LD D,H
    LD E,L
    INC DE
    JR LD4FF

; -----------------------------------------------------------------------------
; MOVE CHARACTER CELLS
; -----------------------------------------------------------------------------
;
; Copies a rectangular run of character-width video cells across ten raster rows.
;
; The active mode supplies the cell width in bytes. For each of ten raster rows, the routine uses
; HL as source, DE as destination, and BC as byte count, then advances both addresses by 40H.
; Entering at D52EH/D530H bypasses the default setup and permits callers to move arbitrary byte
; widths.
;
; Entry:
;   HL = source cell; DE = destination; mode-derived width/count.
;
; Exit:
;   The requested cells are copied.
;
; Effects:
;   Writes video RAM; source remains unchanged.
;
; Destroys:
;   AF, BC, DE, HL.
;
; Note:
;   This primitive is reused for editor insertion, deletion, row shifts, and can support user
;   screen manipulation when called with valid parameters.
; -----------------------------------------------------------------------------
EDITOR_MOVE_CELLS:
    LD A,(ED_CHAR_WIDTH)
    LD C,A

; Alternate entry into MOVE_CELLS: callers can supply their own byte count and bypass the default
; character-width setup.

LD52E:
    LD B,0AH

LD530:
    PUSH BC
    PUSH HL
    PUSH DE
    LD B,00H
    LDIR
    LD C,40H
    POP HL
    ADD HL,BC
    EX DE,HL
    POP HL
    ADD HL,BC
    POP BC
    DJNZ LD530
    RET

; SHIFT_LEFT propagates a full-row overflow by importing the next row's first character into the
; freed final cell.

; -----------------------------------------------------------------------------
; SHIFT CHARACTERS LEFT
; -----------------------------------------------------------------------------
;
; Moves a row's suffix left from a selected position and carries a full-row overflow into the next
; row.
;
; A gives the number of characters to move, B the starting column, C the row, and HL the
; row-status byte. The routine converts the character count to mode-specific video bytes, calls
; MOVE_CELLS, clears the vacated tail, and, when the row was full, imports the next row's first
; character into the freed final cell.
;
; Entry:
;   A = character count (Z means zero); B = start column; C = row; HL = row-status byte.
;
; Exit:
;   The row suffix has moved left and occupancy metadata is corrected.
;
; Effects:
;   Writes video RAM and may propagate data across rows.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
EDITOR_SHIFT_LEFT:
    JR Z,LD562
    PUSH HL
    PUSH BC
    LD E,A
    LD A,(COLOR_MODE)
    OR A
    JR Z,LD552

LD54D:
    SLA E
    DEC A
    JR NZ,LD54D

LD552:
    CALL EDITOR_VIDEO_ADDRESS
    LD C,E
    LD D,H
    LD A,(ED_CHAR_WIDTH)
    ADD A,L
    LD E,A
    EX DE,HL
    CALL LD52E
    POP BC
    POP HL

LD562:
    LD B,(HL)
    BIT 7,B
    JR Z,EDITOR_CLEAR_CELL_AT
    PUSH BC
    RES 7,B
    CALL EDITOR_VIDEO_ADDRESS
    EX DE,HL
    POP BC
    INC C
    LD B,01H
    CALL EDITOR_VIDEO_ADDRESS
    JP EDITOR_MOVE_CELLS

; -----------------------------------------------------------------------------
; CLEAR CELL AT POSITION
; -----------------------------------------------------------------------------
;
; Clears one character cell at a supplied editor row and column.
;
; The routine computes the cell address from B/C, determines the active width (1, 2, or 4 bytes),
; fills that width with PAPER on each of ten raster rows, and advances by 40H between rows.
;
; Entry:
;   B = one-based column; C = one-based row.
;
; Exit:
;   Selected cell is blank.
;
; Effects:
;   Writes video RAM.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
EDITOR_CLEAR_CELL_AT:
    CALL EDITOR_VIDEO_ADDRESS
    LD C,0AH
    LD DE,0040H

LD580:
    LD A,(ED_CHAR_WIDTH)
    LD B,A
    LD A,(CU_PAPER_LINE)
    PUSH HL

LD588:
    LD (HL),A
    INC HL
    DJNZ LD588
    POP HL
    ADD HL,DE
    DEC C
    JR NZ,LD580
    RET

; SHIFT_RIGHT is the screen-side primitive used by INS; its Z flag chooses clearing versus
; importing the previous row's final character.

; -----------------------------------------------------------------------------
; SHIFT CHARACTERS RIGHT
; -----------------------------------------------------------------------------
;
; Moves a row suffix right to make an insertion slot, optionally importing the previous row's last
; character.
;
; Z selects whether the original position is cleared or filled from the previous row. The routine
; obtains the row's final occupied character, calculates the mode-specific byte count, copies
; cells rightward across ten raster rows, and then repairs the original position and row metadata.
; INS enters here for the first cell of a row.
;
; Entry:
;   Z = clear original slot or import previous-row character; B = starting column; C = row; HL =
;   row-status byte.
;
; Exit:
;   A right-shifted insertion space exists; the original position is repaired according to Z.
;
; Effects:
;   Writes video RAM and may move one character across a row boundary.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
EDITOR_SHIFT_RIGHT:
    PUSH AF
    LD A,(HL)
    AND 7FH
    SUB B
    JR Z,LD5CE
    LD E,A
    LD A,(COLOR_MODE)
    OR A
    JR Z,LD5A5

LD5A0:
    SLA E
    DEC A
    JR NZ,LD5A0

LD5A5:
    PUSH HL
    PUSH BC
    LD B,(HL)
    RES 7,B
    CALL EDITOR_VIDEO_ADDRESS
    DEC HL
    LD C,E
    LD B,00H
    EX DE,HL
    LD A,(ED_CHAR_WIDTH)
    LD L,A
    LD H,B
    ADD HL,DE
    EX DE,HL
    LD B,0AH

LD5BB:
    PUSH BC
    PUSH HL
    PUSH DE
    LD B,00H
    LDDR
    LD C,40H
    POP HL
    ADD HL,BC
    EX DE,HL
    POP HL
    ADD HL,BC
    POP BC
    DJNZ LD5BB
    POP BC
    POP HL

LD5CE:
    POP AF
    JR Z,EDITOR_CLEAR_CELL_AT
    EX DE,HL
    PUSH BC
    CALL EDITOR_VIDEO_ADDRESS
    POP BC
    EX DE,HL
    DEC C
    DEC HL
    LD B,(HL)
    RES 7,B
    CALL EDITOR_VIDEO_ADDRESS
    JP EDITOR_MOVE_CELLS

; -----------------------------------------------------------------------------
; KEYBOARD SERVICE TABLE
; -----------------------------------------------------------------------------
; Class 1. Function 2 has no block transfer; it is the RET that ends KB_CHIN.
; -----------------------------------------------------------------------------
KEYBOARD_JUMP_TABLE:
    DB 04H                              ; four functions
    DW KB_INT                           ; 0  interrupt-time scan
    DW KB_CHIN                          ; 1  wait for and read a key
    DW KB_BLOCK_NOP                     ; 2  (no operation)
    DW KB_STATUS                        ; 3  is a key waiting?

; -----------------------------------------------------------------------------
; KEYBOARD INITIALIZATION
; -----------------------------------------------------------------------------
; Sets the default repeat timing (DELAY_KEY 1EH = 30 ticks = 0.6 s before the
; first repeat, RATE_KEY 03H = 60 ms between repeats), clears the lock mode,
; both matrix images and the key work bytes 0BE5H-0BEEH.
; -----------------------------------------------------------------------------
KEYBOARD_INIT:
    LD A,1EH
    LD (DELAY_KEY),A                    ; first repeat after 30 ticks
    LD A,03H
    LD (RATE_KEY),A                     ; then every 3 ticks
    XOR A
    LD (LOCK_KEY),A                     ; no lock mode
    LD HL,KB_PICTURE                    ; clear KB_PICTURE and KB_OLD_PIC,
    LD DE,KB_PICTURE+1                  ; 20 bytes
    LD BC,0013H
    LD (HL),A
    LDIR
    LD HL,KEY_PENDING                   ; clear 0BE5H-0BEEH (B is 0 after LDIR)
    LD DE,KEY_PENDING+1
    LD C,09H
    LD (HL),A
    LDIR
    RET

; -----------------------------------------------------------------------------
; KEYBOARD 03 - STATUS
; -----------------------------------------------------------------------------
; Entry:   none.
; Exit:    A = 00H; C = FFH if KB_INT has queued a key, 00H if not.
; -----------------------------------------------------------------------------
KB_STATUS:
    LD A,(KEY_PENDING)
    LD C,A                              ; C = FFH: a key is waiting
    XOR A
    RET

; -----------------------------------------------------------------------------
; KEYBOARD 01 - CHARACTER INPUT
; -----------------------------------------------------------------------------
; Waits until KB_INT queues a key, then takes it. The wait relies on the
; interrupt running; there is no time-out.
;
; Entry:   none.
; Exit:    C = key code; A = 00H, or F5H if STOP_FLAG is set (CTRL+ESC).
; -----------------------------------------------------------------------------
KB_CHIN:
    LD HL,KEY_PENDING

KB_CHIN_WAIT:
    LD A,(HL)
    OR A
    JR Z,KB_CHIN_WAIT                   ; until the interrupt queues a key
    LD A,(KEY_CODE)
    LD C,A                              ; C = the key
    LD (HL),00H                         ; take it
    LD A,(STOP_FLAG)
    OR A
    RET Z                               ; A = 00H: normal key
    LD A,F5H                            ; error F5H: CTRL+ESC pressed

KB_BLOCK_NOP:
    RET                                 ; as function 2 this returns A = the
                                        ; function byte, which the dispatcher
                                        ; reports as an error

; -----------------------------------------------------------------------------
; KEYBOARD 00 - INTERRUPT SERVICE
; -----------------------------------------------------------------------------
; Runs on every 50 Hz tick. Reads the whole matrix into KB_PICTURE (1 bits =
; pressed) and queues at most one new key in KEY_CODE / KEY_PENDING.
;
; Lock modes. LOCK_KEY holds one of:
;   00H  none
;   01H  caps lock (CTRL+LOCK): letters only are shifted
;   02H  shift lock (SHIFT+LOCK): every key is shifted
;   08H  alt lock (ALT+LOCK)
; LOCK alone clears the mode. A held modifier works against the lock: SHIFT
; with shift lock or caps lock gives the unshifted key, ALT with alt lock gives
; the normal key (see KEY_CODE_TABLE_SELECT). While LOCK is held after setting
; a mode, KEY_LOCK_ACT is FFH and no keys are decoded.
;
; Auto-repeat. A newly accepted key reloads KEY_DELAY_CNT from DELAY_KEY. When
; that count reaches zero the last key (KEY_DIFF_ADDR / KEY_DIFF_MASK) is
; reported again if it is still held, and KEY_RATE_CNT (from RATE_KEY) then
; times the following repeats. On the joystick rows, holding two directions
; makes them alternate.
;
; HOLD. CTRL+P (code 10H) is not queued; it sets HOLD_STATE and KB_INT then
; keeps scanning inside the interrupt, freezing the machine, until another key
; is pressed. That key ends the hold, is queued, and clears STOP_FLAG unless
; CTRL+ESC is down. HOLD_KEY = FFH disables this.
; -----------------------------------------------------------------------------
KB_INT:
    LD A,(LOCK_KEY)
    AND 0BH                             ; valid lock bits are 0, 1 and 3;
    LD B,A
    NEG                                 ; keep only the lowest one that is set
    AND B
    LD (LOCK_KEY),A
    CALL KEY_MATRIX_READ                ; KB_PICTURE = keys down; IY -> it; B = 0
    LD HL,KEY_LOCK_ACT
    BIT 5,(IY+06H)                      ; LOCK (row 6, column 5) down?
    JR Z,KB_INT_KEYS
    LD A,(HL)
    OR A
    RET NZ                              ; LOCK still held after setting a mode
    DEC (HL)                            ; KEY_LOCK_ACT = FFH
    LD A,01H                            ; CTRL+LOCK gives 01H,
    CALL KEY_MODIFIER_STATE             ; SHIFT 02H, ALT 08H, LOCK alone 00H
    LD (LOCK_KEY),A                     ; new lock mode
    RET NZ                              ; a mode was set: decode nothing yet

KB_INT_KEYS:
    LD (HL),B                           ; KEY_LOCK_ACT = 00H
    LD A,04H                            ; CTRL gives 04H,
    CALL KEY_MODIFIER_STATE             ; SHIFT 02H, ALT 08H, none 00H
    LD (KEY_MODIFIER),A

KB_INT_DECODE:
    XOR A                               ; Z: KB_PICTURE is fresh, do not rescan

KB_INT_DECODE_SCAN:
    CALL KEY_MATRIX_DECODE              ; C = code, E = FFH if there is none
    INC E
    JR Z,KB_INT_NO_KEY
    LD A,C
    LD (KEY_CODE),A                     ; queue the key
    LD A,FFH
    LD (KEY_PENDING),A
    LD A,(DELAY_KEY)                    ; restart the repeat delay
    LD (KEY_DELAY_CNT),A
    RET

KB_INT_NO_KEY:
    LD A,(HL)                           ; HL = HOLD_STATE
    OR A                                ; NZ: holding; rescan (CALL NZ in the
    JR NZ,KB_INT_DECODE_SCAN            ; decoder) until a key arrives

; No new key: count down for auto-repeat.

    LD HL,KEY_DELAY_CNT
    CALL KEY_COUNTDOWN                  ; Cy when the delay just expired
    INC HL                              ; HL = KEY_RATE_CNT
    CALL NC,KEY_COUNTDOWN               ; otherwise count the rate
    RET NC                              ; neither expired
    LD A,(KEY_DIFF_MASK)                ; the key to repeat: bit ...
    LD C,A
    LD DE,(KEY_DIFF_ADDR)               ; ... in this KB_OLD_PIC row
    LD HL,KB_OLD_PIC+9                  ; Cy is set: HL - DE - 1 = 0
    SBC HL,DE                           ; for row 8
    EX DE,HL                            ; HL = row address
    JR Z,KEY_REPEAT_JOYSTICK
    OR A
    EX DE,HL
    LD HL,KB_OLD_PIC+9
    SBC HL,DE                           ; zero for row 9
    EX DE,HL                            ; HL = row address
    JR NZ,KEY_REPEAT_HELD

KEY_REPEAT_JOYSTICK:
    LD D,66H                            ; up, down, right, left bits
    AND D
    JR Z,KEY_REPEAT_HELD                ; fire, accel or INS: repeat normally
    XOR D                               ; the other directions ...
    AND (HL)                            ; ... that are still held
    JR NZ,KEY_REPEAT_AGAIN              ; report those instead

KEY_REPEAT_HELD:
    LD A,C
    AND (HL)                            ; is the key still held?
    JR NZ,KEY_REPEAT_AGAIN
    LD HL,KB_OLD_PIC                    ; released: forget every held key,
    LD B,0AH                            ; so keys still down are reported
                                        ; again as new presses

KEY_FORGET_ALL:
    LD (HL),A
    INC HL
    DJNZ KEY_FORGET_ALL
    LD (KEY_RATE_CNT),A                 ; no repeat pending
    RET

KEY_REPEAT_AGAIN:
    CPL                                 ; clear the bit(s) in KB_OLD_PIC so the
    AND (HL)                            ; decoder sees them as newly pressed
    LD (HL),A
    LD A,(RATE_KEY)
    LD (KEY_RATE_CNT),A                 ; time the next repeat
    JR KB_INT_DECODE

; Count (HL) down. Z, NC: already zero. NZ, NC: still running. Cy: reached
; zero now.

KEY_COUNTDOWN:
    LD A,(HL)
    OR A
    RET Z
    DEC (HL)
    RET NZ
    SCF
    RET

; -----------------------------------------------------------------------------
; KEYBOARD MATRIX DECODER
; -----------------------------------------------------------------------------
; Finds the first newly pressed key (rows 9 down to 0, lowest bit first),
; records it in KB_OLD_PIC, KEY_DIFF_ADDR and KEY_DIFF_MASK, and translates it
; with the code tables. The modifier and LOCK keys are removed from
; KB_PICTURE first, so they never produce a code. Keys released since the last
; scan are removed from KB_OLD_PIC.
;
; The table index is (7 - column) * 10 + row: each table holds the ten rows
; of column 7, then column 6, and so on.
;
; Entry:   NZ to rescan the matrix first, Z to use KB_PICTURE as it is.
; Exit:    E = FFH if no key; otherwise C = code.
;          HL = HOLD_STATE in both cases.
; -----------------------------------------------------------------------------
KEY_MATRIX_DECODE:
    CALL NZ,KEY_MATRIX_READ
    RES 3,(IY+06H)                      ; ignore SHIFT,
    RES 4,(IY+07H)                      ; CTRL,
    RES 0,(IY+07H)                      ; ALT
    RES 5,(IY+06H)                      ; and LOCK
    LD HL,HOLD_STATE                    ; alternate set: HL' = HOLD_STATE,
    LD DE,000AH                         ; E' = row counter, D' = 0
    EXX
    LD HL,KB_OLD_PIC+10                 ; HL -> just past KB_OLD_PIC
    LD DE,KB_OLD_PIC                    ; DE -> just past KB_PICTURE

KEY_DECODE_ROW:
    DEC DE                              ; DE -> KB_PICTURE row
    DEC HL                              ; HL -> KB_OLD_PIC row
    EXX
    DEC E                               ; E' = row 9..0
    RET M                               ; all rows done: E' = FFH, no key
    EXX
    LD A,(DE)                           ; A = keys down now
    LD B,A
    XOR (HL)
    CPL
    AND (HL)                            ; keys down before and now
    LD (HL),A                           ; drop released keys from KB_OLD_PIC
    XOR B                               ; keys down now but not before
    JR Z,KEY_DECODE_ROW
    LD B,A
    NEG
    AND B                               ; isolate the lowest new key
    LD B,A
    XOR (HL)
    LD (HL),A                           ; mark it as seen
    LD A,B
    LD (KEY_DIFF_ADDR),HL               ; remember it for auto-repeat
    LD (KEY_DIFF_MASK),A
    LD A,50H                            ; A = 80 - 10 * (column + 1)

KEY_DECODE_COLUMN:
    SUB 0AH
    SRL B
    JR NC,KEY_DECODE_COLUMN             ; = (7 - column) * 10
    EXX
    ADD A,E                             ; + row
    LD E,A                              ; DE = table index (D is 0)
    LD A,(KEY_MODIFIER)
    LD B,A
    LD A,(LOCK_KEY)
    XOR B                               ; B = held modifier XOR lock mode
    LD B,A
    PUSH HL                             ; HL = HOLD_STATE
    CALL KEY_CODE_TABLE_SELECT          ; C = code
    POP HL
    LD A,(HOLD_KEY)
    INC A
    RET Z                               ; HOLD disabled: return the key
    LD A,(HL)
    OR A
    LD A,C
    JR Z,KEY_HOLD_CHECK                 ; not holding: is this CTRL+P?
    LD A,(PORT03_MIRROR)                ; holding: this key ends the hold.
    AND F0H                             ; Read row 7 directly
    OR 07H
    OUT (03H),A
    IN A,(58H)
    AND 18H                             ; ESC (bit 3) and CTRL (bit 4), 0 = down
    JR Z,KEY_HOLD_END                   ; CTRL+ESC: keep STOP_FLAG
    XOR A
    LD (STOP_FLAG),A                    ; any other key cancels the stop
    JR KEY_HOLD_END

KEY_HOLD_CHECK:
    CP 10H                              ; CTRL+P?
    JR Z,KEY_HOLD_START

KEY_HOLD_END:
    LD (HL),00H                         ; not holding
    RET

KEY_HOLD_START:
    DEC (HL)                            ; HOLD_STATE = FFH
    LD E,FFH                            ; and report no key
    RET

; -----------------------------------------------------------------------------
; KEYBOARD CODE-TABLE SELECTOR
; -----------------------------------------------------------------------------
; B = held modifier XOR lock mode (bit 0 caps lock, bit 1 shift, bit 2 CTRL,
; bit 3 alt). Because of the XOR, a held SHIFT cancels shift lock and a held
; ALT cancels alt lock. Caps lock shifts only letters: a-z (61H-7AH) and the
; accented lower-case letters 90H-98H. With caps lock, SHIFT gives lower case.
;
;   CTRL held                       KEY_TABLE_CTRL
;   caps lock, letter               KEY_TABLE_SHIFT, or normal with SHIFT
;   otherwise shift                 KEY_TABLE_SHIFT
;   otherwise alt                   KEY_TABLE_ALT
;   otherwise                       KEYBOARD_CODE_TABLES (normal)
;
; A table byte of FFH (CTRL+ESC) sets STOP_FLAG and is returned as ESC (1BH).
;
; Entry:   DE = table index; B as above.
; Exit:    C = code.
; -----------------------------------------------------------------------------
KEY_CODE_TABLE_SELECT:
    LD HL,KEY_TABLE_READ                ; most paths RET into KEY_TABLE_READ
    PUSH HL                             ; with HL = the chosen table
    LD HL,KEY_TABLE_CTRL
    BIT 2,B                             ; CTRL held
    RET NZ
    BIT 0,B                             ; caps lock?
    JR Z,KEY_TABLE_NOT_CAPS
    LD HL,KEYBOARD_CODE_TABLES          ; look at the normal code:
    ADD HL,DE
    LD A,(HL)
    CP 61H                              ; is it a lower-case letter?
    JR C,KEY_TABLE_NOT_CAPS
    CP 7BH
    JR C,KEY_TABLE_CAPS                 ; a-z
    CP 90H
    JR C,KEY_TABLE_NOT_CAPS
    CP 99H
    JR NC,KEY_TABLE_NOT_CAPS            ; not 90H-98H

KEY_TABLE_CAPS:
    LD HL,KEY_TABLE_SHIFT               ; letter with caps lock: shifted ...
    BIT 1,B
    RET Z
    LD HL,KEYBOARD_CODE_TABLES          ; ... unless SHIFT is held
    RET

KEY_TABLE_NOT_CAPS:
    LD HL,KEY_TABLE_SHIFT
    BIT 1,B                             ; shift (held or locked)
    RET NZ
    LD HL,KEY_TABLE_ALT
    BIT 3,B                             ; alt (held or locked)
    RET NZ
    POP HL                              ; normal table: drop the return
    LD HL,KEYBOARD_CODE_TABLES          ; address and fall through

KEY_TABLE_READ:
    ADD HL,DE
    LD C,(HL)                           ; C = code
    LD A,C
    CP FFH                              ; CTRL+ESC?
    RET NZ
    LD (STOP_FLAG),A                    ; STOP_FLAG = FFH
    LD C,1BH                            ; and return ESC
    RET

; -----------------------------------------------------------------------------
; MODIFIER KEY STATE
; -----------------------------------------------------------------------------
; Entry:   A = value to return for CTRL; IY = KB_PICTURE.
; Exit:    A = that value if CTRL is down, else 02H for SHIFT, 08H for ALT,
;          00H for none; Z only for none.
; -----------------------------------------------------------------------------
KEY_MODIFIER_STATE:
    BIT 4,(IY+07H)                      ; CTRL (row 7, column 4)
    RET NZ
    LD A,02H
    BIT 3,(IY+06H)                      ; SHIFT (row 6, column 3)
    RET NZ
    BIT 0,(IY+07H)                      ; ALT (row 7, column 0)
    LD A,08H
    RET NZ
    XOR A
    RET

; -----------------------------------------------------------------------------
; PHYSICAL KEYBOARD SCAN
; -----------------------------------------------------------------------------
; Reads rows 0-9 into KB_PICTURE, inverted so that 1 = pressed. Port 03H keeps
; the card-select bits from PORT03_MIRROR; it is left selecting row 9 and the
; mirror is not updated.
;
; Exit:    HL -> KB_PICTURE+10; IY = KB_PICTURE; B = 0.
; -----------------------------------------------------------------------------
KEY_MATRIX_READ:
    LD A,(PORT03_MIRROR)
    AND F0H                             ; keep the card select, row 0
    LD C,A
    LD HL,KB_PICTURE
    PUSH HL
    POP IY                              ; IY = KB_PICTURE for the bit tests
    LD B,0AH                            ; ten rows

KEY_MATRIX_ROW:
    LD A,C
    OUT (03H),A                         ; select the row
    IN A,(58H)                          ; 0 = pressed
    CPL                                 ; 1 = pressed
    LD (HL),A
    INC C
    INC HL
    DJNZ KEY_MATRIX_ROW
    RET

; -----------------------------------------------------------------------------
; KEYBOARD CODE TABLES
; -----------------------------------------------------------------------------
; Four 80-byte tables: normal, shift, CTRL and alt. Each line below is one
; matrix column, rows 0-9; the comment names the keys in that column (row 8 is
; INS and the built-in joystick, row 9 the second joystick). Besides the '*'
; key, 2AH fills the modifier and LOCK positions and the positions with no
; key; the decoder never looks those up. Row 9 column 0 has no key; the four tables hold 'C', 'I',
; 'S', 'L' there. FFH in the CTRL table is CTRL+ESC (see KEY_TABLE_READ).
; -----------------------------------------------------------------------------
KEYBOARD_CODE_TABLES:
    DB 34H, 37H, 72H, 75H, 66H, 6AH, 76H, 6DH, 2AH, 2AH  ; C7: 4 7 r u f j v m - -
    DB 31H, 94H, 71H, 70H, 61H, 91H, 79H, 2DH, 13H, F3H  ; C6: 1 ö q p a é y - LT LT2
    DB 92H, 93H, 40H, 96H, 3CH, 98H, 2AH, 20H, 04H, E4H  ; C5: í ó @ ú < ű LOCK SPC RT RT2
    DB 36H, 2AH, 7AH, 5BH, 68H, 0DH, 6EH, 2AH, 01H, E1H  ; C4: 6 * z [ h RET n CTRL ACC ACC2
    DB 30H, 97H, 3BH, 95H, 5CH, 90H, 2AH, 1BH, 06H, E6H  ; C3: 0 ü ; ő \ á SHIFT ESC FIRE FIRE2
    DB 32H, 39H, 77H, 6FH, 73H, 6CH, 78H, 2EH, 18H, F8H  ; C2: 2 9 w o s l x . DN DN2
    DB 33H, 38H, 65H, 69H, 64H, 6BH, 63H, 2CH, 05H, E5H  ; C1: 3 8 e i d k c , UP UP2
    DB 35H, 5EH, 74H, 5DH, 67H, 08H, 62H, 2AH, 16H, 43H  ; C0: 5 ^ t ] g DEL b ALT INS -

KEY_TABLE_SHIFT:
    DB 21H, 3DH, 52H, 55H, 46H, 4AH, 56H, 4DH, 2AH, 2AH  ; C7: 4 7 r u f j v m - -
    DB 27H, 84H, 51H, 50H, 41H, 81H, 59H, 5FH, 13H, F3H  ; C6: 1 ö q p a é y - LT LT2
    DB 82H, 83H, 60H, 86H, 3EH, 88H, 2AH, 20H, 04H, E4H  ; C5: í ó @ ú < ű LOCK SPC RT RT2
    DB 2FH, 23H, 5AH, 7BH, 48H, 0DH, 4EH, 2AH, 01H, E1H  ; C4: 6 * z [ h RET n CTRL ACC ACC2
    DB 26H, 87H, 24H, 85H, 7CH, 80H, 2AH, 1BH, 06H, E6H  ; C3: 0 ü ; ő \ á SHIFT ESC FIRE FIRE2
    DB 22H, 29H, 57H, 4FH, 53H, 4CH, 58H, 3AH, 18H, F8H  ; C2: 2 9 w o s l x . DN DN2
    DB 2BH, 28H, 45H, 49H, 44H, 4BH, 43H, 3FH, 05H, E5H  ; C1: 3 8 e i d k c , UP UP2
    DB 25H, 7EH, 54H, 7DH, 47H, 07H, 42H, 2AH, 16H, 49H  ; C0: 5 ^ t ] g DEL b ALT INS -

KEY_TABLE_CTRL:
    DB 8BH, 9CH, 12H, 15H, 06H, 0AH, 16H, 0DH, 2AH, 2AH  ; C7: 4 7 r u f j v m - -
    DB 99H, DCH, 11H, 10H, 01H, D9H, 19H, 1FH, 13H, F3H  ; C6: 1 ö q p a é y - LT LT2
    DB DAH, DBH, 00H, DEH, 3CH, 98H, 2AH, 20H, 04H, E4H  ; C5: í ó @ ú < ű LOCK SPC RT RT2
    DB 8CH, 8EH, 1AH, 1BH, 08H, 0DH, 0EH, 2AH, 01H, E1H  ; C4: 6 * z [ h RET n CTRL ACC ACC2
    DB 89H, DFH, 3BH, DDH, 1CH, CFH, 2AH, FFH, 06H, E6H  ; C3: 0 ü ; ő \ á SHIFT ESC FIRE FIRE2
    DB 8AH, 9DH, 17H, 0FH, 13H, 0CH, 18H, 2EH, 18H, F8H  ; C2: 2 9 w o s l x . DN DN2
    DB 9AH, 8DH, 05H, 09H, 04H, 0BH, 03H, 2CH, 05H, E5H  ; C1: 3 8 e i d k c , UP UP2
    DB 9BH, 1EH, 14H, 1DH, 07H, 08H, 02H, 2AH, 16H, 53H  ; C0: 5 ^ t ] g DEL b ALT INS -

KEY_TABLE_ALT:
    DB A4H, A7H, C2H, C5H, B6H, BAH, C6H, BDH, 2AH, 2AH  ; C7: 4 7 r u f j v m - -
    DB A1H, D4H, C1H, C0H, B1H, D1H, C9H, ADH, 13H, F3H  ; C6: 1 ö q p a é y - LT LT2
    DB D2H, D3H, B0H, D6H, ACH, D8H, 2AH, 20H, 04H, E4H  ; C5: í ó @ ú < ű LOCK SPC RT RT2
    DB A6H, AAH, CAH, CBH, B8H, 0DH, BEH, 2AH, 01H, E1H  ; C4: 6 * z [ h RET n CTRL ACC ACC2
    DB A0H, D7H, ABH, D5H, CCH, D0H, 2AH, 1BH, 06H, E6H  ; C3: 0 ü ; ő \ á SHIFT ESC FIRE FIRE2
    DB A2H, A9H, C7H, BFH, C3H, BCH, C8H, AEH, 18H, F8H  ; C2: 2 9 w o s l x . DN DN2
    DB A3H, A8H, B5H, B9H, B4H, BBH, B3H, AFH, 05H, E5H  ; C1: 3 8 e i d k c , UP UP2
    DB A5H, CEH, C4H, CDH, B7H, 08H, B2H, 2AH, 16H, 4CH  ; C0: 5 ^ t ] g DEL b ALT INS -

; -----------------------------------------------------------------------------
; PRINTER SERVICE TABLE
; -----------------------------------------------------------------------------
; Class 4: the Centronics port. Function 0 is the RET that ends PAR_CHOUT.
; -----------------------------------------------------------------------------
PRINTER_JUMP_TABLE:
    DB 03H                                ; 3 functions
    DW PAR_INT_NOP                        ; 0  interrupt (no work)
    DW PAR_CHOUT                          ; 1  character output
    DW PAR_BKOUT                          ; 2  block output

; -----------------------------------------------------------------------------
; PRINTER 02 - BLOCK OUTPUT
; -----------------------------------------------------------------------------
; Entry:   DE -> text; BC = length.
; Exit:    A = 00H, F5H after CTRL+ESC, or FAH (see OUT_CHARS_SAFE).
; -----------------------------------------------------------------------------
PAR_BKOUT:
    LD HL,PAR_CHOUT                     ; one PAR_CHOUT call per byte
    JP OUT_CHARS_SAFE

; -----------------------------------------------------------------------------
; PRINTER 01 - CHARACTER OUTPUT
; -----------------------------------------------------------------------------
; Waits until the printer is ready (port 59H bit 7 = 1), puts the character
; on the data port 01H and pulses STROBE (port 06H bit 7) low. There is no
; time-out: without a printer this waits until CTRL+ESC. Interrupts are
; enabled on exit even if they were disabled on entry.
;
; Entry:   C = character.
; Exit:    A = 00H, or F5H if STOP_FLAG is set.
; -----------------------------------------------------------------------------
PAR_CHOUT:
    LD A,(STOP_FLAG)
    INC A                               ; Z if STOP_FLAG = FFH
    LD A,F5H
    RET Z                               ; CTRL+ESC: give up
    IN A,(59H)
    RLCA                                ; Cy = bit 7, printer ready
    JR NC,PAR_CHOUT                     ; busy: poll again
    DI                                  ; keep the strobe pulse short
    LD A,C
    OUT (01H),A                         ; data
    LD A,(PORT06_MIRROR)
    AND 7FH
    OUT (06H),A                         ; STROBE low ...
    OR 80H
    OUT (06H),A                         ; ... and high again
    EI
    XOR A                               ; success

PAR_INT_NOP:
    RET

; -----------------------------------------------------------------------------
; SOUND SERVICE TABLE
; -----------------------------------------------------------------------------
; Class 3. Functions 1 and 2 are RETs; they return the function byte, which the
; dispatcher reports as an error.
; -----------------------------------------------------------------------------
SOUND_JUMP_TABLE:
    DB 04H                                ; 4 functions
    DW TONE_INT                           ; 0  interrupt: time the tone
    DW TONE_NOP                           ; 1  (no operation)
    DW TONE_NOP                           ; 2  (no operation)
    DW TONE_SET                           ; 3  start a tone

; -----------------------------------------------------------------------------
; SOUND 00 - INTERRUPT SERVICE
; -----------------------------------------------------------------------------
; Called every tick while INT_DES bit 3 is clear, which TONE_SET arranges.
; Counts TONE_DURATION down and stops the tone at zero or on CTRL+ESC.
; The interrupt routine ignores the status this returns.
; -----------------------------------------------------------------------------
TONE_INT:
    LD A,(SOUND_ACT)
    INC A                               ; tone playing (FFH)?
    RET NZ
    LD A,(STOP_FLAG)
    INC A                               ; CTRL+ESC: stop now (A = 00H)
    JR Z,TONE_STOP
    LD A,(TONE_DURATION)                ; one tick less
    DEC A
    LD (TONE_DURATION),A
    RET NZ                              ; still playing

; Stop the tone. Enter with A = 00H. Returns A = F5H.

TONE_STOP:
    LD (SOUND_ACT),A                    ; no tone
    LD (TONE_DURATION),A
    LD A,(INT_DES)
    OR 08H                              ; no more sound ticks
    LD (INT_DES),A
    LD A,(PORT05_MIRROR)
    AND CFH                             ; clear sound enable (bit 4) and
    LD (PORT05_MIRROR),A                ; the sound interrupt (bit 5)
    OUT (05H),A
    LD A,F5H

TONE_NOP:
    RET

; -----------------------------------------------------------------------------
; SOUND 03 - START A TONE
; -----------------------------------------------------------------------------
; Unless TONE_REPLACE is FFH, first waits for the current tone to come down
; to its last tick. (This wait needs the interrupt running.) B = 0 just stops
; any tone. The divider is shared with the serial cards, so SER_OK is set to
; FFH: their clock is no longer valid.
;
; Entry:   B = duration in 20 ms ticks; C = volume 0-15 (bits 3-0);
;          DE = 12-bit PITCH divider (tone = 195312.5 / (4096 - PITCH) Hz).
; Exit:    A = 00H, or F5H if STOP_FLAG is set.
; -----------------------------------------------------------------------------
TONE_SET:
    LD A,(TONE_REPLACE)
    INC A                               ; FFH: replace the tone at once
    JR Z,TONE_SET_NOW
    LD A,(TONE_DURATION)
    SUB 02H                             ; wait while 2 or more ticks remain
    JR NC,TONE_SET

TONE_SET_NOW:
    XOR A
    LD (SOUND_ACT),A                    ; the interrupt stops counting
    LD A,(STOP_FLAG)
    INC A
    JR Z,TONE_STOP                      ; CTRL+ESC: silence, A = F5H
    LD A,B
    OR A
    JR NZ,TONE_SET_PLAY
    CALL TONE_STOP                      ; duration 0: silence,
    XOR A                               ; A = 00H
    RET

TONE_SET_PLAY:
    LD (TONE_DURATION),A
    LD A,C
    AND 0FH
    RLCA
    RLCA
    LD C,A                              ; volume in bits 5-2
    LD A,(PORT06_MIRROR)
    AND C3H                             ; keep STROBE, bit 6 and colour mode
    OR C
    LD (PORT06_MIRROR),A
    OUT (06H),A
    LD A,D
    AND 0FH                             ; PITCH bits 11-8
    OR 10H                              ; + sound enable
    LD D,A
    LD A,(PORT05_MIRROR)
    AND C0H                             ; keep the motor bits; sound IRQ off
    OR D
    LD (PORT05_MIRROR),A
    OUT (05H),A
    LD A,E
    OUT (04H),A                         ; PITCH bits 7-0
    LD A,FFH
    LD (SOUND_ACT),A                    ; tone playing
    LD (SER_OK),A                       ; serial clock no longer valid
    LD A,(INT_DES)
    AND F7H                             ; call TONE_INT every tick
    LD (INT_DES),A
    XOR A
    RET

; -----------------------------------------------------------------------------
; CASSETTE SERVICE TABLE
; -----------------------------------------------------------------------------
; Class 5. Everything except function 0 is implemented in the extension ROM.
; Each forwarder below loads an EXT address and jumps through the FFF0H
; gateway. The flags from the dispatcher (S = input, from the function byte's
; bit 7) survive the trip, and the EXT stubs pick the input or output routine
; with JP P / JP M. They return through EXT_DEVICE_RETURN to CAS_INT_NOP,
; whose RET goes back to the dispatcher.
; -----------------------------------------------------------------------------
CASSETTE_JUMP_TABLE:
    DB 06H                                ; 6 functions
    DW CAS_INT_NOP                        ; 0  interrupt (no work)
    DW CAS_CHIN_OUT                       ; 1  character in/out
    DW CAS_BKIN_OUT                       ; 2  block in/out
    DW CAS_OPEN_CREATE                    ; 3  open/create
    DW CAS_CLOSE                          ; 4  close
    DW CAS_VERIFY                         ; 5  verify

CAS_OPEN_CREATE:
    LD HL,F3E2H                           ; EXT: CAS_CREATE_WRITE (output) or
    JR CAS_TO_EXT                         ; CAS_OPEN_READ (input)

CAS_CLOSE:
    LD HL,F3E7H                           ; EXT: close the output file (flushing
    JR CAS_TO_EXT                         ; it) or the input file

CAS_CHIN_OUT:
    LD HL,F3D8H                           ; EXT: CAS_CHOUT or CAS_CHIN
    JR CAS_TO_EXT

CAS_BKIN_OUT:
    LD HL,F3DDH                           ; EXT: CAS_BKOUT or CAS_BKIN
    JR CAS_TO_EXT

CAS_VERIFY:
    LD HL,F3ECH                           ; EXT: CAS_VERIFY_DATA

CAS_TO_EXT:
    JP CALL_EXTENSION_HL

; Called once at reset (STARTUP_DEVICE_INIT_TABLE), not through RST 30H.

CAS_INIT:
    LD HL,F3F1H                           ; EXT: CAS_WORK_INIT
    JR CAS_TO_EXT

CAS_INT_NOP:
    RET                                   ; also the SYS return point of every
                                          ; EXT cassette call

    DB "(c) ISL"                          ; Intelligent Software Ltd. signature

; BASIC workspace begins here; IX is based at 1700H and cold start clears the workspace through
; 19EFH.

; -----------------------------------------------------------------------------
; BASIC WORKSPACE INITIALIZATION
; -----------------------------------------------------------------------------
;
; Builds the BASIC workspace, clears program state, initializes devices, and enters the sign-on
; path.
;
; IX is set to the 16-byte BASIC status area at 1700H. On a cold start the routine clears
; 1700H-19EFH, installs the initial error/dispatch stubs, and sets VLOMEM and TEXT to 19EFH. It
; then initializes program pointers, BASIC-stack state, DATA pointers, the symbol-chain boundary,
; random-number seed state, and TOP.
;
; After device initialization it invokes the startup display: the palette is loaded, the VIDEOTON
; and TV COMPUTER picture is drawn, and the routine waits for the first key. The sign-on text is
; emitted using embedded length-prefixed strings. Once input arrives it selects four-colour mode,
; prints the BASIC 1.2 copyright banner and free-byte count, and converges with the warm-reset
; continuation before entering the command loop.
;
; Entry:
;   Entry from reset/cartridge handling with warm/cold status already established.
;
; Exit:
;   Initialized BASIC environment; control eventually reaches BASIC_COMMAND_LOOP.
;
; Effects:
;   May erase the BASIC workspace and resets program, symbol, stack, editor, video, keyboard, and
;   cassette state.
;
; Destroys:
;   AF, BC, DE, HL, IX and stack contents used during initialization.
;
; Note:
;   The user program begins at TEXT/VLOMEM, while the BASIC stack grows down from HI-MEM. The
;   source books describe this routine as the boundary between system startup and the high-level
;   BASIC interpreter.
; -----------------------------------------------------------------------------
BASIC_INIT:
    LD HL,BASIC_FLAGS
    PUSH HL
    POP IX
    LD A,(WARM_FLAG)
    OR A
    JP NZ,LDAD3
    LD BC,02EFH

LD9FF:
    LD (HL),A
    CPI
    JP PE,LD9FF

; Initialize TEXT/VLOMEM, BASIC stack boundary, DATA pointers, symbol-chain boundary, and TOP.
    LD (VLOMEM),HL
    LD (TEXT_START),HL
    LD HL,FB5BH
    LD DE,0008H
    LD BC,0027H                         ; 39 bytes: 0008H-002EH
    LDIR
    CALL LDE10

; Startup animation uses RST 30H video calls and loops until a keyboard event is available.

; -----------------------------------------------------------------------------
; BASIC SIGN-ON SCREEN
; -----------------------------------------------------------------------------
;
; Constructs the animated TV Computer sign-on display and waits for the first key.
;
; The routine uses RST 30H video calls to position and print VIDEOTON, TV COMPUTER, and the
; multicolour title. It varies the colour parameters while emitting the title, pauses briefly, and
; repeats the animation until the keyboard service reports a key. The subsequent text and
; free-memory calculation establish the normal BASIC screen before command entry.
;
; Entry:
;   Video and keyboard OS services initialized; IX points to BASIC state.
;
; Exit:
;   Sign-on display shown; returns after a key has been detected and acknowledged.
;
; Effects:
;   Writes video state, palette/colour variables, and consumes the first key event.
; -----------------------------------------------------------------------------
STARTUP_SCREEN:
    LD DE,DC15H
    RST 30H
    DB 0CH                                ; RST 30H fn 0CH: video 12 -> PAL_DEF
    LD A,02H
    LD (BORDER),A
    LD (IX+05H),00H
    LD BC,0303H
    LD A,0CH

LDA2C:
    PUSH BC
    PUSH AF
    RST 30H
    DB 03H                                ; RST 30H fn 03H: video 3 -> CH_POS
    CALL LDBF2
    POP AF
    PUSH AF

    CALL LDC0C
    CALL LDBF2
    CALL PRINT_CRLF
    POP AF
    POP BC
    INC B
    INC C
    INC C
    SUB 02H
    JR NC,LDA2C
    LD BC,0D15H
    RST 30H
    DB 03H                                ; RST 30H fn 03H: video 3 -> CH_POS

    CALL LDBF2

LDA4F:
    LD BC,0B12H
    RST 30H
    DB 03H                                ; RST 30H fn 03H: video 3 -> CH_POS
    LD HL,DBFFH
    LD B,(HL)

LDA58:
    INC HL
    LD A,(HL)
    CALL PRINT_CHAR
    PUSH HL
    PUSH BC

LDA5F:
    CALL E6D8H
    AND 03H
    JR Z,LDA5F
    POP BC
    POP HL
    LD (INK),A

LDA6B:
    DEC C
    JR NZ,LDA6B
    DJNZ LDA58
    RST 30H
    DB 93H                                ; RST 30H fn 93H: keyboard 3, input -> KB_STATUS
    INC C
    JR NZ,LDA4F
    RST 30H
    DB 91H                                ; RST 30H fn 91H: keyboard 1, input -> KB_CHIN
    LD C,01H
    RST 30H
    DB 04H                                ; RST 30H fn 04H: video 4 -> VID_MODE
    LD (BORDER),A
    CALL MESSAGE_COLOURS
    CALL PRINT_INLINE_TEXT

; TVC BASIC 1.2 copyright sign-on text.
; Embedded BASIC 1.2 sign-on text: TV COMPUTER BASIC 1.2 and Copyright 1985 VIDEOTON.
    DB "2TV COMPUTER BASIC 1.2", 0DH, 0AH, "Copyright 1985 VIDEOTON", 0DH, 0AH, 0DH, 0AH ; inline text, 50 bytes
    CALL BASIC_FREE
    CALL PRINT_NUMBER_TEXT
    CALL DISCARD_NUMBER
    CALL PRINT_INLINE_TEXT

; "bytes free" text.
; Embedded bytes-free message and the code that computes the available BASIC workspace.
    DB 0FH, " bytes free", 0DH, 0AH, 0DH, 0AH ; inline text, 15 bytes

LDAD3:
    XOR A
    LD (WARM_FLAG),A
    CALL BASIC_WORKSPACE_INIT

BASIC_READY:
    LD SP,STACK_TOP
    LD HL,1703H
    LD A,(HL)
    OR A
    LD (HL),00H
    CALL NZ,LDE10
    LD (IX+05H),20H
    RES 2,(IX+00H)
    BIT 1,(IX+00H)
    JR NZ,LDAFF

LDAF5:
    CALL MESSAGE_COLOURS
    CALL PRINT_INLINE_TEXT

; "ok" text followed by clear-to-end-of-line control character.
; Embedded OK prompt followed by the editor clear-to-end-of-line control sequence.
    DB 03H, "ok", 0BH                     ; inline text, 3 bytes

LDAFF:
    RES 1,(IX+00H)
    CALL PRINT_CRLF

; Read a complete editor paragraph into INPUT at 1831H, tokenize it into COMMAND, and classify
; line-numbered versus immediate input.

; -----------------------------------------------------------------------------
; BASIC COMMAND INPUT LOOP
; -----------------------------------------------------------------------------
;
; Reads one edited input line, tokenizes it, and classifies it as a stored program line or
; immediate command.
;
; The loop prints the OK prompt through the video/editor services, obtains a complete line in the
; 1831H INPUT buffer, and handles empty input, CTRL+ESC, and file-end status. TOKENIZE_BASIC_LINE
; converts keywords to one-byte tokens and writes the compact result to the 1732H COMMAND buffer.
;
; The first value in COMMAND is parsed as a possible line number. A valid 1..9999 number selects
; the program-line insertion path at DB2AH; otherwise the routine constructs an immediate command
; and dispatches it. A blank line or a line beginning with the '*' escape returns to the prompt
; without interpretation.
;
; Entry:
;   Editor-produced line in 1831H INPUT buffer.
;
; Exit:
;   Stored/updated program line or immediate statement execution.
;
; Effects:
;   Updates COMMAND, BASIC stack, program text, and interpreter state.
;
; Destroys:
;   AF, BC, DE, HL, alternate registers.
; -----------------------------------------------------------------------------
BASIC_COMMAND_LOOP:
    CALL BASIC_LINE_INPUT
    CP F5H
    JP Z,STOP_FROM_ERROR
    CP ECH
    JR Z,LDAF5
    CALL TOKENIZE_BASIC_LINE
    LD E,L
    LD D,H
    INC HL
    CALL ASCII_TO_FP
    JR C,LDB2A
    LD A,(HL)
    CP A8H
    JR Z,LDB25
    INC A
    JR NZ,LDB61

LDB25:
    CALL DISCARD_NUMBER
    JR BASIC_COMMAND_LOOP

LDB2A:
    RLA
    JR C,LDB61
    LD B,(HL)
    LD A,(IY+08H)
    CP 40H
    JR C,LDB61
    CP 44H
    JR NC,LDB61
    LD A,(IY+06H)
    OR A
    JR Z,LDB61

LDB3F:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,LDB3F
    DEC HL
    EX DE,HL
    LD A,(HL)
    INC HL
    ADD A,02H
    SBC HL,DE
    ADD A,L
    LD C,A
    CALL STACK_NUMBER_TO_INTEGER
    EX DE,HL
    DEC HL
    LD (HL),D
    DEC HL
    LD (HL),E
    DEC HL
    LD (HL),C
    LD (CUR_LINE),HL
    CALL BASIC_INSERT_LINE
    JR BASIC_COMMAND_LOOP

LDB61:
    EX DE,HL
    CALL DISCARD_NUMBER
    LD C,(HL)
    XOR A
    LD (HL),A
    DEC HL
    LD (HL),A
    DEC HL
    INC C
    INC C
    LD (HL),C

LDB6E:
    LD (CUR_LINE),HL
    INC HL
    BIT 0,(IX+00H)
    CALL NZ,BASIC_TRACE_LINE
    RES 0,(IX+02H)
    INC HL
    INC HL
    EXX

; Primary-token interpreter dispatch: token values index the jump table and transfer control to a
; statement handler.

; -----------------------------------------------------------------------------
; BASIC STATEMENT DISPATCH
; -----------------------------------------------------------------------------
;
; Fetches the next tokenized BASIC statement and jumps through the primary-token handler table.
;
; The interpreter keeps the current token pointer in the alternate register set. It skips spaces,
; recognizes statement terminators and REM, and transforms the descending token value into an
; index into the primary statement jump table at the beginning of the ROM. The target address is
; loaded from the table and entered with the BASIC stack and interpreter state prepared.
;
; When a statement completes, handlers return here or to the related continuation labels. The same
; machinery executes immediate commands and stored program lines; the distinction is represented
; by the current line pointers and the interpreter flags rather than by a second evaluator.
;
; Entry:
;   HL' points into tokenized BASIC text; BASIC state and IX are valid.
;
; Exit:
;   Control transfers to the handler selected by the next primary token.
;
; Effects:
;   Updates current-statement pointers and uses the CPU stack as a handler-return boundary.
; -----------------------------------------------------------------------------
BASIC_NEXT_STATEMENT:
    EXX

LDB81:
    CALL CHECK_STOP_FLAG
    LD (BASIC_SP),IY

LDB88:
    LD A,(HL)
    CP FEH
    JR NC,BASIC_REM_ENTRY
    INC HL
    CP 20H
    JR Z,LDB88
    CP CAH
    JP C,E3BCH
    PUSH AF
    EXX
    CPL
    ADD A,A
    ADD A,67H
    LD L,A
    LD H,C0H
    LD A,(HL)
    INC L
    LD H,(HL)
    LD L,A
    POP AF
    CP FBH
    CALL C,GET_TOKEN
    LD SP,STACK_TOP

LDBAD:
    JP (HL)

LDBAE:
    CALL GET_TOKEN

LDBB1:
    EXX
    LD A,B
    EXX

LDBB4:
    CP FDH
    JR Z,BASIC_NEXT_STATEMENT
    JP C,ERROR_NOT_UNDERSTOOD

; REM routine.
; REM handling skips to the next colon, exclamation/REM boundary, or line terminator without
; executing the remainder.

; -----------------------------------------------------------------------------
; REM STATEMENT
; -----------------------------------------------------------------------------
;
; Skips the remainder of a BASIC statement line as a comment.
;
; REM is handled by advancing from the current line pointer over its length-prefixed body until a
; statement separator or FFH line end. No expression evaluation or output occurs. This same
; boundary logic supports the exclamation/comment spelling accepted by the tokenizer.
; -----------------------------------------------------------------------------
BASIC_REM_ENTRY:
    LD HL,(CUR_LINE)
    LD C,(HL)
    XOR A
    LD B,A
    ADD HL,BC

LDBC2:
    OR (HL)
    JR NZ,LDB6E

LDBC5:
    LD A,(IY+00H)
    CP 2BH
    JR Z,LDBDA
    CP 06H
    JR Z,LDBDA

LDBD0:
    BIT 2,(IX+00H)
    JP Z,BASIC_READY
    JP E10EH

LDBDA:
    LD E,(IY+03H)
    LD D,(IY+04H)
    LD HL,INPUT_BUFFER
    SBC HL,DE
    JR C,LDBD0
    LD C,A
    LD B,00H
    ADD IY,BC
    LD (BASIC_SP),IY
    JR LDBC5

LDBF2:
    CALL PRINT_INLINE_TEXT

; "VIDEOTON" sign-on text.
    DB 08H, "VIDEOTON"                    ; inline text, 8 bytes
    RET

; "TV COMPUTER" sign-on text.
    DB 0CH, 54H, 56H, 20H, 20H, 43H, 4FH, 4DH, 50H, 55H, 54H, 45H, 52H                ; |.TV  COMPUTER|

LDC0C:
    DEC A
    RET M
    PUSH AF
    CALL PRINT_SPACE
    POP AF
    JR LDC0C

; Cold-start sign-on picture palette bytes.
    DB 01H, 44H, 54H, 51H                                                             ; |.DTQ|

; TOKENIZE_BASIC_LINE writes compact output to COMMAND; quoted text is copied literally and
; keyword matching resumes after the closing quote.

; -----------------------------------------------------------------------------
; BASIC LINE TOKENIZER
; -----------------------------------------------------------------------------
;
; Converts editable ASCII input into the compact tokenized BASIC representation.
;
; HL starts at the editor's INPUT buffer and DE tracks the source characters. The tokenizer scans
; the descending keyword table at DE6DH, replacing recognized words with their token byte while
; preserving spaces, numbers, punctuation, and other non-keyword characters. Quoted strings and
; colon separators suppress keyword recognition where BASIC treats text literally.
;
; The high bit on the final character of each keyword table entry marks the end of that word. The
; output is written through IY into the COMMAND buffer at 1732H; the routine stores the resulting
; length and terminates the compact line with FFH. This is lexical compression only: statement
; validity is checked later by the handler selected from the token.
;
; Entry:
;   HL=editor INPUT buffer, whose first byte is its length and whose body ends with FFH.
;
; Exit:
;   HL/IY identify the compact COMMAND line at 1732H/1735H.
;
; Effects:
;   Writes tokenized BASIC text and updates command length.
;
; Destroys:
;   AF, BC, DE, HL, IY and alternate AF.
; -----------------------------------------------------------------------------
TOKENIZE_BASIC_LINE:
    PUSH IY
    EX DE,HL
    LD IY,CMD_BUFFER+3
    LD BC,0000H

LDC23:
    INC DE

LDC24:
    LD HL,DE6EH
    LD A,FEH
    EX AF,AF'

LDC2A:
    INC IY
    LD (171CH),DE
    CALL FOLD_CASE_DE
    LD (IY+00H),A
    INC A
    JR Z,LDC9F
    LD A,B
    OR A
    JR Z,LDC41
    CP 3AH
    JR NZ,LDC4B

LDC41:
    LD A,(DE)
    CP 22H
    JR NZ,LDC4B
    LD C,B
    LD B,A
    INC DE
    JR LDC2A

LDC4B:
    CALL FOLD_CASE_DE
    INC DE
    OR A
    JR Z,LDC60
    CP B
    JR NZ,LDC60
    LD B,C
    LD C,00H
    CP 22H
    JR Z,LDC24
    LD A,FDH
    JR LDC88

LDC60:
    INC B
    DEC B
    JR NZ,LDC2A
    XOR (HL)
    ADD A,A
    JR NZ,LDC8D
    INC HL
    JR NC,LDC4B
    EX AF,AF'
    LD C,B
    CP A6H
    JR Z,LDC79
    CP A5H
    JR Z,LDC79
    CP A3H
    JR NZ,LDC7B

; Keyword-table matches use the high bit on the final character; recognized words become one-byte
; tokens while numbers and punctuation remain unchanged.

LDC79:
    SUB 08H

LDC7B:
    CP FDH
    JR Z,LDC88
    CP FBH
    JR C,LDC88
    LD B,A
    JR NZ,LDC88
    LD B,3AH

LDC88:
    LD (IY+00H),A
    JR LDC24

LDC8D:
    LD DE,(171CH)

LDC91:
    BIT 7,(HL)
    INC HL
    JR Z,LDC91
    EX AF,AF'
    DEC A
    EX AF,AF'
    LD A,(HL)
    INC A
    JR Z,LDC23
    JR LDC4B

; Finalize tokenized line: store its length and append FFH as the line terminator.

LDC9F:
    PUSH IY
    POP HL
    INC HL
    LD (HL),A
    LD DE,CMD_BUFFER+3
    OR A
    SBC HL,DE
    EX DE,HL
    LD (HL),E
    POP IY
    RET

; Insert a new numbered line: build its stored length/line-number format, make room with block
; moves, and reinitialize BASIC pointers.

; -----------------------------------------------------------------------------
; INSERT BASIC PROGRAM LINE
; -----------------------------------------------------------------------------
;
; Builds and inserts a numbered line, moving later program text to make room.
;
; The command loop has already converted the line number to a two-byte binary value and
; constructed the stored line length. This helper finds the ordered insertion point, removes an
; old line with the same number when present, moves the remainder of the program, and copies the
; new line into the gap. Because addresses change, it finishes by reinitializing the BASIC stack,
; DATA pointers, random state, and user symbols.
;
; Entry:
;   HL=new line source; B=first body character offset; C=stored line length.
;
; Exit:
;   Program text contains the new line in ascending order.
;
; Effects:
;   Moves program bytes and invalidates user symbol definitions.
;
; Destroys:
;   AF, BC, DE, HL.
; -----------------------------------------------------------------------------
BASIC_INSERT_LINE:
    LD (171CH),HL
    PUSH BC
    CALL FIND_BASIC_LINE
    CALL NC,BASIC_DELETE_LINE
    POP BC
    INC B
    RET Z
    LD B,00H
    CALL BASIC_FREE_SPACE
    EX DE,HL
    LD HL,(171CH)
    LDIR
    JR BASIC_WORKSPACE_INIT

; -----------------------------------------------------------------------------
; MAKE ROOM IN BASIC PROGRAM
; -----------------------------------------------------------------------------
;
; Frees a region in the program area by shifting later bytes toward the high-memory end.
;
; The insertion path supplies a start address and byte count. This helper finds the current
; program end, computes the number of bytes that must move, and performs the overlapping transfer
; so the new line can be copied into the resulting gap. It is bounded by the BASIC memory layout
; and the HI-MEM stack boundary.
;
; Entry:
;   HL=region start; BC=number of bytes to reserve.
;
; Exit:
;   The requested gap is available at HL.
;
; Effects:
;   Moves program storage and causes subsequent addresses to change.
; -----------------------------------------------------------------------------
BASIC_FREE_SPACE:
    CALL BASIC_WORKSPACE_INIT
    CALL CHECK_MEMORY_256
    PUSH BC
    PUSH HL
    CALL LDD41
    POP DE
    PUSH HL
    OR A
    SBC HL,DE
    LD C,L
    LD B,H
    POP DE
    POP HL
    PUSH HL
    ADD HL,DE
    EX DE,HL
    INC BC
    LDDR
    INC HL
    POP BC
    RET

; -----------------------------------------------------------------------------
; DELETE BASIC PROGRAM LINE
; -----------------------------------------------------------------------------
;
; Deletes one stored line by moving the remainder of the program over it.
;
; Given the line's length-byte address, the helper computes the line end and the current program
; end, then performs an overlapping backward transfer to close the gap. The caller retains its
; LIST/LLIST/DELETE mode flags and continues range processing after the deletion.
;
; Entry:
;   HL=line length-byte address.
;
; Exit:
;   The line is absent and subsequent lines are contiguous.
;
; Effects:
;   Moves program bytes and changes the program end; callers normally reinitialize BASIC workspace
;   afterward.
; -----------------------------------------------------------------------------
BASIC_DELETE_LINE:
    LD C,(HL)
    LD B,00H

LDCE9:
    PUSH HL
    ADD HL,BC
    PUSH HL
    CALL LDD41
    POP DE
    OR A
    SBC HL,DE
    LD C,L
    LD B,H
    EX DE,HL
    POP DE
    PUSH DE
    INC BC
    LDIR
    POP HL

; -----------------------------------------------------------------------------
; RESET BASIC EXECUTION WORKSPACE
; -----------------------------------------------------------------------------
;
; Clears transient interpreter state after program edits or before execution.
;
; This common initializer places IY at the BASIC stack boundary, clears the END and DATA state
; pointers, restores the built-in symbol-chain boundary, seeds the random-number state, and sets
; the current program pointer and TOP. Program editing uses it because moving lines invalidates
; stack and symbol addresses; RUN uses the same reset before starting execution.
;
; Entry:
;   BASIC program pointers and HI-MEM are valid.
;
; Exit:
;   Transient workspace and execution pointers are reset.
;
; Effects:
;   Overwrites BASIC stack state and discards user symbols.
; -----------------------------------------------------------------------------
BASIC_WORKSPACE_INIT:
    PUSH HL
    PUSH DE
    PUSH BC
    LD IY,(HI_MEM)
    LD HL,0000H
    LD (IY+00H),L
    LD (NEXT_LINE),HL
    LD (DATA_PTR),HL
    LD HL,FN_REC_VERNUM
    LD (SYMBOL_END),HL
    LD HL,RND_STATE
    LD (HL),L
    LD (RND_STATE+1),HL
    LD HL,(TEXT_START)
    LD (DATA_LINE),HL
    CALL LDD41
    INC HL
    LD (SYMBOL_TOP),HL
    POP BC
    POP DE
    POP HL
    RET

; -----------------------------------------------------------------------------
; EMIT ONE STORED BASIC LINE
; -----------------------------------------------------------------------------
;
; Formats a stored line number and emits its tokenized body through the selected output class.
;
; The helper skips the line length byte, reads the binary line number, prints it in the selected
; device class, and then emits the tokenized body until its FFH line terminator. It is shared by
; LIST and by diagnostic/error paths that need to display a line. The carry result distinguishes a
; normal program listing from a line being edited or reported.
;
; Entry:
;   HL=stored line length byte; output class is already selected.
;
; Exit:
;   The line is formatted and emitted; carry identifies the caller's listing mode.
;
; Effects:
;   Uses the active OS output device and advances through the line.
; -----------------------------------------------------------------------------
BASIC_PRINT_LINE:
    PUSH AF
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    PUSH HL
    EX DE,HL
    CALL PRINT_INTEGER_HL
    POP HL
    CALL LIST_TEXT
    POP AF
    RET NC
    JP PRINT_NEWLINE_CLR

; Any program edit invalidates user symbols and resets the BASIC stack through the common
; workspace initializer.

LDD41:
    LD HL,FFFFH

LDD44:
    EX DE,HL

; Search length-prefixed program lines in ascending order; zero length marks the end and supplies
; the append position.

; -----------------------------------------------------------------------------
; FIND BASIC LINE
; -----------------------------------------------------------------------------
;
; Searches the ordered BASIC program for a line number or insertion point.
;
; The program is a sequence of length-prefixed lines beginning at TEXT (1722H). FIND_BASIC_LINE
; walks the length bytes and two-byte binary line numbers in ascending order. It stops on an exact
; match, on the first larger line, or at the zero length byte marking the program end.
;
; Entry:
;   DE=requested binary line number; 1722H points to the program.
;
; Exit:
;   HL points at the matching line's length byte or insertion position; Z marks an exact match and
;   carry distinguishes an intervening position.
;
; Effects:
;   Reads program text but does not modify it.
; -----------------------------------------------------------------------------
FIND_BASIC_LINE:
    LD HL,(TEXT_START)
    LD BC,0000H

LDD4B:
    ADD HL,BC
    LD C,(HL)
    INC C
    DEC C
    SCF
    RET Z
    PUSH DE
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    DEC HL
    DEC HL
    EX (SP),HL
    OR A
    SBC HL,DE
    ADD HL,DE
    EX DE,HL
    POP HL
    RET Z
    JR NC,LDD4B
    RET

LDD63:
    RST 08H
    DB 0AH                                ; error code 0AH: Cannot CONTINUE

; CONTINUE routine.
; CONTINUE restores the saved line and statement pointers; zero saved address is the END state and
; raises Cannot Continue.

; -----------------------------------------------------------------------------
; CONTINUE STATEMENT
; -----------------------------------------------------------------------------
;
; Resumes a stopped BASIC program from the saved next-statement pointer.
;
; CONTINUE rejects an already-running program and raises Cannot Continue when the saved pointer at
; 170EH is zero (the END state). Otherwise it sets the running flag, restores the saved line
; pointer at 170CH and statement pointer at 1710H, and re-enters the common interpreter dispatch.
;
; Entry:
;   Interpreter flags and saved execution pointers.
;
; Exit:
;   Program execution resumes or error 0AH is raised.
;
; Effects:
;   Sets the running-program flag and restores execution context.
; -----------------------------------------------------------------------------
BASIC_CONTINUE:
    BIT 2,(IX+00H)
    JP NZ,LDBB1
    SET 2,(IX+00H)
    LD HL,(NEXT_LINE)
    LD A,H
    OR L
    JR Z,LDD63
    LD (CUR_LINE),HL
    LD HL,(NEXT_STMT)
    JP LDB81

; LLIST routine.
; LLIST selects printer output and falls into the shared LIST range parser.

; -----------------------------------------------------------------------------
; LLIST ENTRY
; -----------------------------------------------------------------------------
;
; Selects printer-class output before entering the shared LIST machinery.
;
; LLIST differs from LIST only in its initial device-class flag. It marks printer output and then
; falls into the common range parser and line emitter, preserving the distinction in the flags
; carried through the shared code.
; -----------------------------------------------------------------------------
BASIC_LLIST_ENTRY:
    LD C,40H
    OR C
    JR LDD88

; LIST routine.
; LIST selects editor output unless a #device reference overrides it; LLIST, LIST, and DELETE
; share this range-processing core.

; -----------------------------------------------------------------------------
; LIST AND LLIST STATEMENTS
; -----------------------------------------------------------------------------
;
; Parses optional line ranges and emits selected BASIC lines to the chosen device.
;
; LIST selects the editor/video class by default, while LLIST enters through a flag selecting
; printer output. Optional '#' device syntax can replace that class. The shared parser accepts a
; start, an end, either one alone, or a start-end range; it walks the ordered program and emits or
; deletes lines according to the carried mode flags.
;
; For output, each line is decoded from its stored length and binary line number and sent through
; the common listing formatter. The range parser uses 0000H and FFFFH as open-ended bounds and
; treats a comma as the separator for another range. CTRL+ESC is checked through the output path
; so long listings remain abortable.
;
; Entry:
;   Tokenized LIST/LLIST parameters and program text at TEXT.
;
; Exit:
;   Selected source lines sent to the active output class.
;
; Effects:
;   Reads program storage, invokes character/block output, and updates temporary parser state.
; -----------------------------------------------------------------------------
BASIC_LIST:
    LD C,20H
    XOR A

LDD88:
    PUSH AF
    CALL CHANNEL_ONLY_C
    POP AF
    JR NZ,LDD94
    LD A,C
    CP 20H
    DB 16H                                ; = LD D,F6H when run from here, hiding the entry below

LDD93:
    DB F6H                                ; = OR 37H when run from here, hiding the entry below

LDD94:
    SCF
    EX AF,AF'
    EXX
    LD A,B
    EXX
    LD HL,0000H
    LD DE,FFFFH
    CP FDH
    JR NC,LDDD1
    JR LDDB6

LDDA5:
    EXX
    LD A,B
    EXX
    CP A4H
    JR Z,LDDB3
    EX AF,AF'
    JP C,LDBB1
    JP BASIC_READY

LDDB3:
    CALL GET_TOKEN

LDDB6:
    CALL BASIC_LIST_RANGE_PARAM
    INC D
    JR NZ,LDDC4
    CP A2H
    JP NZ,ERROR_NOT_UNDERSTOOD
    LD DE,0100H

LDDC4:
    DEC D
    PUSH DE
    CP A2H
    JR NZ,LDDD0
    CALL GET_TOKEN
    CALL BASIC_LIST_RANGE_PARAM

LDDD0:
    POP HL

LDDD1:
    PUSH DE
    CALL LDD44
    POP DE

LDDD6:
    LD A,(HL)
    OR A
    JR Z,LDDA5
    INC HL
    LD C,(HL)
    INC HL
    LD B,(HL)
    DEC HL
    DEC HL
    EX DE,HL
    SBC HL,BC
    ADD HL,BC
    EX DE,HL
    JR C,LDDA5
    PUSH DE
    EX AF,AF'
    JR NC,LDDF0
    CALL BASIC_PRINT_LINE
    JR LDDF4

LDDF0:
    CALL BASIC_DELETE_LINE
    OR A

LDDF4:
    EX AF,AF'
    CALL CHECK_STOP_FLAG
    POP DE
    JR LDDD6

; A single line-number parameter is converted to an open-ended range; comma introduces another
; range.

; -----------------------------------------------------------------------------
; PARSE LIST RANGE PARAMETER
; -----------------------------------------------------------------------------
;
; Converts an optional line-number parameter into an open or closed LIST range.
;
; The routine reads a number previously placed on the BASIC stack and returns it in DE; if no
; number is present it returns FFFFH as an open bound. LIST and LLIST use this helper for forms
; such as LIST 100, LIST -200, and LIST 100-200 while preserving the current range separator
; state.
;
; Entry:
;   BASIC stack may contain a parsed line number; A carries the preceding-token flag.
;
; Exit:
;   DE=line bound or FFFFH for an omitted bound.
; -----------------------------------------------------------------------------
BASIC_LIST_RANGE_PARAM:
    LD DE,FFFFH
    CP 02H
    RET NZ
    CALL STACK_NUMBER_TO_INTEGER
    EX DE,HL
    JP GET_TOKEN

; NEW routine.
; NEW marks the current program empty at TEXT, clears TRACE, and enters common BASIC workspace
; initialization.

; -----------------------------------------------------------------------------
; NEW STATEMENT
; -----------------------------------------------------------------------------
;
; Invalidates the current program and reinitializes the BASIC execution workspace.
;
; NEW clears TRACE, writes zero at the current program base (the zero length marks no program),
; resets TEXT and program pointers, and enters the common BASIC workspace initializer. The old
; bytes are not physically erased, but they are outside the live program after the boundary is
; reset.
;
; Effects:
;   Clears the current program logically, BASIC stack, DATA pointers, and user symbol chain.
; -----------------------------------------------------------------------------
BASIC_NEW:
    LD HL,BASIC_READY
    PUSH HL
    RES 0,(IX+00H)

LDE10:
    LD HL,(VLOMEM)
    LD (TEXT_START),HL
    LD (HL),00H
    JP BASIC_WORKSPACE_INIT

; RUN routine.
; RUN selects TEXT or an optional line, initializes execution/file state, sets the running flag,
; and dispatches the first statement.

; -----------------------------------------------------------------------------
; RUN STATEMENT
; -----------------------------------------------------------------------------
;
; Initializes execution and starts the current BASIC program, optionally at a specified line.
;
; RUN chooses the program start at TEXT or converts an optional line number into a line pointer.
; It invokes the common BASIC workspace/file initialization, sets the running-program flag, clears
; transient interpreter flags, and enters the statement dispatcher at the first selected line.
;
; Entry:
;   Optional tokenized line number.
;
; Exit:
;   Control transfers to the first statement to execute.
;
; Effects:
;   Resets execution state and closes/initializes device files as required.
; -----------------------------------------------------------------------------
BASIC_RUN:
    LD HL,(TEXT_START)
    CP 02H
    CALL Z,POP_LINE_NUMBER_FIND

LDE23:
    CALL BASIC_WORKSPACE_INIT
    CALL CLOSE_TAPE_FILES

LDE29:
    SET 2,(IX+00H)
    XOR A
    JP LDBC2

; TRACE parses ON/OFF and retains the selected trace device class separately from the general
; output class.

; -----------------------------------------------------------------------------
; TRACE ON/OFF
; -----------------------------------------------------------------------------
;
; Enables or disables line-number tracing for a running BASIC program.
;
; TRACE first accepts an optional output-class/device selection, then requires ON or OFF. The
; state is stored in the interpreter flags and the chosen trace class is kept separately at 1706H.
; When enabled, the per-line helper prints the current line number between angle brackets before
; executing its statements.
;
; Effects:
;   Updates trace-enable state and trace output class.
; -----------------------------------------------------------------------------
BASIC_TRACE:
    CALL CHANNEL_ONLY
    LD (IX+06H),C
    CP E3H
    JR Z,LDE46
    CP C1H
    JP NZ,ERROR_NOT_UNDERSTOOD

; TRACE OFF routine.
    RES 0,(IX+00H)
    JR LDE4A

; TRACE ON routine.

LDE46:
    SET 0,(IX+00H)

LDE4A:
    JP LDBAE

; When TRACE is enabled for a running program, emit '<', the current decimal line number, and '>'
; before its statements.

; -----------------------------------------------------------------------------
; TRACE LINE EMITTER
; -----------------------------------------------------------------------------
;
; Prints the current BASIC line number when TRACE is enabled.
;
; The helper returns immediately unless a program is running and TRACE is enabled. It temporarily
; applies the trace output class, writes '<', formats the binary line number in decimal through
; the BASIC output helpers, and writes '>'.
;
; Entry:
;   HL points at the current line number.
;
; Exit:
;   Trace text emitted to the configured device.
;
; Effects:
;   Uses the active output class and preserves interpreter continuation state.
; -----------------------------------------------------------------------------
BASIC_TRACE_LINE:
    BIT 2,(IX+00H)
    RET Z
    LD A,(DEVICE_NUM)
    LD (FUNC_CLASS),A
    PUSH HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    EX DE,HL
    LD A,3CH
    CALL PRINT_CHAR
    LD B,00H
    CALL PRINT_INTEGER_B
    POP HL
    LD A,3EH
    JP PRINT_CHAR

; BASIC keyword table ordered by descending token value; the high bit marks the last byte of each
; keyword.
; Keyword strings are stored with the final character's high bit set; the descending table covers
; statements, secondary words, and operators.

; -----------------------------------------------------------------------------
; BASIC KEYWORD TABLE
; -----------------------------------------------------------------------------
;
; Descending keyword strings used by the tokenizer to produce BASIC tokens.
;
; The table stores keyword characters in the order expected by TOKENIZE_BASIC_LINE. The high bit
; on the final character terminates each word; the table then continues with the next keyword and
; eventually punctuation/operator spellings. It includes primary statements, secondary keywords
; such as PITCH and VOLUME, and symbolic operators.
;
; Note:
;   This is data, not executable code. A disassembler must keep the high-bit terminators and the
;   immediately following token associations intact.
; -----------------------------------------------------------------------------
BASIC_KEYWORD_TABLE:
    DB FFH, A1H, BAH, 52H, 45H, CDH, 44H, 41H, 54H, C1H, 43H, 4CH, 4FH, 53H, C5H, 43H ; |...RE.DAT.CLOS.C|
    DB 4CH, D3H, 43H, 4FH, 4EH, 54H, 49H, 4EH, 55H, C5H, 44H, 45H, C6H, 44H, 45H, 4CH ; |L.CONTINU.DE.DEL|
    DB 45H, 54H, C5H, 44H, 49H, CDH, 45H, 4CH, 53H, C5H, 45H, 4EH, C4H, 46H, 4FH, D2H ; |ET.DI.ELS.EN.FO.|
    DB 47H, 45H, D4H, 47H, 4FH, 53H, 55H, C2H, 47H, 4FH, 54H, CFH, 47H, 52H, 41H, 50H ; |GE.GOSU.GOT.GRAP|
    DB 48H, 49H, 43H, D3H, 49H, C6H, 49H, 4EH, 50H, 55H, D4H, 4CH, 45H, D4H, 4CH, 49H ; |HIC.I.INPU.LE.LI|
    DB 53H, D4H, 4CH, 4CH, 49H, 53H, D4H, 4CH, 4FH, 41H, C4H, 4CH, 4FH, 4DH, 45H, CDH ; |S.LLIS.LOA.LOME.|
    DB 4EH, 45H, D7H, 4EH, 45H, 58H, D4H, 4FH, CBH, 4FH, CEH, 4FH, 50H, 45H, CEH, 4FH ; |NE.NEX.O.O.OPE.O|
    DB 55H, 54H, 50H, 55H, D4H, 4FH, 55H, D4H, 50H, 4CH, 4FH, D4H, 50H, 4FH, 4BH, C5H ; |UTPU.OU.PLO.POK.|
    DB 50H, 52H, 49H, 4EH, D4H, 52H, 41H, 4EH, 44H, 4FH, 4DH, 49H, 5AH, C5H, 52H, 45H ; |PRIN.RANDOMIZ.RE|
    DB 41H, C4H, 52H, 45H, 53H, 54H, 4FH, 52H, C5H, 52H, 45H, 54H, 55H, 52H, CEH, 52H ; |A.RESTOR.RETUR.R|
    DB 55H, CEH, 53H, 41H, 56H, C5H, 53H, 45H, D4H, 53H, 4FH, 55H, 4EH, C4H, 53H, 54H ; |U.SAV.SE.SOUN.ST|
    DB 4FH, D0H, 54H, 52H, 41H, 43H, C5H, 56H, 45H, 52H, 49H, 46H, D9H, 45H, 58H, D4H ; |O.TRAC.VERIF.EX.|
    DB 4CH, 50H, 52H, 49H, 4EH, D4H, A1H, A1H, A1H, A1H, A1H, A1H, 41H, 4EH, C4H, 43H ; |LPRIN.......AN.C|
    DB 48H, 41H, 52H, 41H, 43H, 54H, 45H, D2H, 44H, 45H, 4CH, 41H, D9H, 44H, 55H, 52H ; |HARACTE.DELA.DUR|
    DB 41H, 54H, 49H, 4FH, CEH, 49H, 4EH, 4BH, 45H, 59H, A4H, 49H, 4EH, CBH, 4DH, 4FH ; |ATIO.INKEY.IN.MO|
    DB 44H, C5H, 4EH, 4FH, D4H, 4FH, 46H, C6H, 4FH, 52H, C4H, 4FH, D2H, 50H, 41H, 49H ; |D.NO.OF.OR.O.PAI|
    DB 4EH, D4H, 50H, 41H, 4CH, 45H, 54H, 54H, C5H, 50H, 41H, 50H, 45H, D2H, 50H, 49H ; |N.PALETT.PAPE.PI|
    DB 54H, 43H, C8H, 50H, 52H, 4FH, 4DH, 50H, D4H, 52H, 41H, 54H, C5H, 53H, 54H, 45H ; |TC.PROMP.RAT.STE|
    DB D0H, 53H, 54H, 59H, 4CH, C5H, 54H, 41H, C2H, 54H, 48H, 45H, CEH, 54H, CFH, 56H ; |.STYL.TA.THE.T.V|
    DB 4FH, 4CH, 55H, 4DH, C5H, 58H, 4FH, D2H, 41H, 54H, CEH, 41H, D4H, 55H, 53H, 49H ; |OLUM.XO.AT.A.USI|
    DB 4EH, C7H, 42H, 4FH, 52H, 44H, 45H, D2H, A1H, A1H, A1H, A1H, A1H, AAH, A3H, 3DH ; |N.BORDE........=|
    DB BEH, 3EH, BCH, ACH, 3DH, BCH, ADH, AFH, BBH, DEH, 3EH, BDH, 3CH, BEH, BEH, 3CH ; |.>..=.....>.<..<|
    DB BDH, BDH, BCH, ABH, A6H, A8H, A9H, FFH, 20H, 6DH, 69H, 73H, 73H, 69H, 6EH, E7H ; |........ missin.|
    DB 72H, 67H, 75H, 6DH, 65H, 6EH, F4H, 42H, 61H, 64H, A0H, 4EH, 6FH, A0H, 43H, 61H ; |rgumen.Bad.No.Ca|
    DB 6EH, 6EH, 6FH, 74H, A0H                                                        ; |nnot.|

; DATA routine.
; DATA performs no immediate computation; advance to the next statement boundary and leave values
; for READ's DATA-pointer search.

; -----------------------------------------------------------------------------
; DATA STATEMENT
; -----------------------------------------------------------------------------
;
; Skips DATA contents until the next statement separator or line end.
;
; DATA statements are not executed at their definition point. This entry advances over their
; comma-separated text until it reaches a colon, REM marker, or the line terminator, leaving the
; DATA pointer machinery to READ.
; -----------------------------------------------------------------------------
BASIC_DATA_ENTRY:
    EXX

LDFF3:
    LD A,(HL)
    CP FDH
    JP NC,LDB81
    INC HL
    JR LDFF3

; CLS BASIC routine.
; CLS is a thin BASIC wrapper around the video clear-screen function.

; -----------------------------------------------------------------------------
; CLS STATEMENT
; -----------------------------------------------------------------------------
;
; Implements BASIC CLS by invoking the video-class clear-screen function.
;
; CLS is a thin BASIC wrapper around the operating-system video service. It supplies the video
; function selector and returns through the common statement continuation after the screen has
; been cleared.
;
; Effects:
;   Clears the selected display and updates video cursor state as defined by the video service.
; -----------------------------------------------------------------------------
BASIC_CLS:
    RST 30H
    DB 05H                                ; RST 30H fn 05H: video 5 -> VID_CLS
    RST 10H
    DB C3H                                ; JP LDBB1 - continues in the next ROM chip
