; -----------------------------------------------------------------------------
; TVC BASIC 1.2 SYS low ROM
; Source: roms/TVC12_D4.64K
; ORG: C000H
; Size: 8192 bytes
; Symbols: data/rom_symbols_1_2.json
; Comments: data/rom_comments_1_2.json
; Data ranges: C003H-C228H, C334H-C337H, C4ACH-C4B6H, C545H-C572H, C5B4H-C973H, C974H-C98EH, C9EAH-C9F1H, CB7FH-CBDCH, CF98H-D012H, D170H-D190H, D7BFH-D905H, D92AH-D9C7H, DA84H-DB05H, DBF6H-DC20H
; Auto labels: branch and call targets are emitted as Lxxxx.
; -----------------------------------------------------------------------------

ORG C000H


; RESET_VECTOR - Reset vector; jumps to BASIC cold start.
; usage: trace
RESET_VECTOR:

; LL: Reset starts with JP 0229H; at power-on the SYS ROM is also visible at page 0, so this is executed as address 0000H.
    JP 0229H

; BCD_MULTIPLICATION_TABLE - BCD products for decimal digits 0 through 9.
; usage: data
BCD_MULTIPLICATION_TABLE:

; KL: Multiplication table from 00 to 99; each byte is a BCD value.
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 01H, 02H, 03H, 04H, 05H
    DB 06H, 07H, 08H, 09H, 00H, 02H, 04H, 06H, 08H, 10H, 12H, 14H, 16H, 18H, 00H, 03H
    DB 06H, 09H, 12H, 15H, 18H, 21H, 24H, 27H, 00H, 04H, 08H, 12H, 16H, 20H, 24H, 28H
    DB 32H, 36H, 00H, 05H, 10H, 15H, 20H, 25H, 30H, 35H, 40H, 45H, 00H, 06H, 12H, 18H
    DB 24H, 30H, 36H, 42H, 48H, 54H, 00H, 07H, 14H, 21H, 28H, 35H, 42H, 49H, 56H, 63H
    DB 00H, 08H, 16H, 24H, 32H, 40H, 48H, 56H, 64H, 72H, 00H, 09H, 18H, 27H, 36H, 45H
    DB 54H, 63H, 72H, 81H

; BASIC_STATEMENT_JUMP_TABLE - Jump table for primary BASIC statement tokens.
; usage: trace,data
BASIC_STATEMENT_JUMP_TABLE:

; KL: BASIC statement jump table; contains routine addresses for tokens FFH down to D0H.
    DB BBH, DBH, BBH, DBH, 80H, DBH, BBH, DBH, F2H, DFH, 9BH, E8H, FCH, DFH, 65H, DDH
    DB 02H, E0H, 93H, DDH, 53H, E0H, 04H, E1H, 0EH, E1H, 5CH, E1H, 10H, E9H, 82H, E3H
    DB B2H, E3H, 33H, E7H, EEH, E2H, CBH, E1H, C1H, E3H, 85H, DDH, 80H, DDH, 51H, E9H
    DB 52H, E4H, 08H, DEH, B6H, E4H, 06H, DBH, 32H, E3H, C9H, E8H, 73H, E5H, 42H, E5H
    DB 54H, E7H, 53H, E5H, 73H, E5H, C7H, E6H, 1BH, E2H, F4H, E6H, 0FH, E7H, 1BH, DEH
    DB 82H, E9H, 90H, E7H, 33H, E8H, A3H, FFH, 31H, DEH, D3H, E9H, 17H, E1H, 70H, E5H

; RST18_JUMP_TABLE - Jump table for RST 18H arithmetic operations 0 through 14.
; usage: trace,data
RST18_JUMP_TABLE:

; KL: RST 18H arithmetic routine jump table; contains entries 0 through 14.
    DB 93H, F4H, FBH, F5H, 12H, F5H, 8EH, F4H, 26H, F7H, 82H, EAH, 9FH, EAH, 9AH, EAH
    DB D2H, EAH, CDH, EAH, C3H, EAH, BEH, EAH, 92H, FAH, 28H, FBH, 68H, EAH, 43H, 6FH
    DB 70H, 79H, 72H, 69H, 67H, 68H, 74H, 20H, 28H, 63H, 29H, 20H, 31H, 39H, 38H, 34H
    DB 20H, 20H, 49H, 6EH, 74H, 65H, 6CH, 6CH, 69H, 67H, 65H, 6EH, 74H, 20H, 53H, 6FH
    DB 66H, 74H, 77H, 61H, 72H, 65H, 20H, 4CH, 74H, 64H, 00H, 00H, 00H, 00H, 00H, 50H
    DB 3FH, 00H, 00H, 00H, 00H, 00H, 10H, 40H, 31H, 24H, 19H, 49H, 79H, 26H, 3FH, 57H
    DB 07H, 08H, 05H, 32H, 17H, 40H, 69H, 75H, 80H, 50H, 20H, 73H, 3FH, 74H, 48H, 34H
    DB 08H, 40H, 14H, C0H, 98H, 88H, 84H, 26H, 00H, 72H, BFH, 19H, 89H, 03H, 25H, 20H
    DB 43H, 40H, 99H, 45H, 58H, 22H, 52H, 47H, 40H, 07H, 38H, 96H, 88H, 85H, 86H, 3FH
    DB 00H, 00H, 00H, 00H, 51H, 11H, 40H, 23H, 70H, 49H, 46H, 25H, 29H, 3CH, 06H, 95H
    DB 88H, 64H, 44H, 50H, 42H, 63H, 75H, 99H, 82H, 00H, 14H, 41H, 16H, 65H, 64H, 73H
    DB 28H, 33H, 3EH, 01H, 79H, 97H, 92H, 08H, 10H, 43H, 97H, 10H, 08H, 94H, 20H, 11H
    DB 42H, 99H, 92H, 50H, 58H, 02H, 23H, 40H, 90H, 37H, 14H, 68H, 15H, 29H, C0H, 57H
    DB 15H, 49H, 03H, 63H, 31H, 40H, 78H, 14H, 60H, 81H, 35H, 67H, BFH, 42H, 95H, 06H
    DB 04H, 07H, 10H, C1H, 21H, 40H, 81H, 69H, 96H, 16H, 41H, 67H, 54H, 04H, 80H, 90H
    DB 81H, C0H, 07H, 38H, 96H, 88H, 85H, 86H, 3FH, 88H, 60H, 66H, 66H, 66H, 16H, BFH
    DB 56H, 20H, 07H, 33H, 33H, 83H, 3DH, 31H, 82H, 32H, 08H, 84H, 19H, BCH, 78H, 06H
    DB 71H, 39H, 52H, 27H, 3AH, 60H, 40H, 46H, 83H, 86H, 23H, B8H, 00H, 00H, 00H, 07H
    DB 36H, 22H, 3FH, 00H, 00H, 00H, 27H, 44H, 89H, 3FH, 17H, 60H, 76H, 27H, 62H, 31H
    DB 3FH, 31H, 51H, 79H, 57H, 29H, 57H, 41H, 59H, 53H, 26H, 59H, 41H, 31H, 40H, 79H
    DB 26H, 63H, 79H, 70H, 15H, 40H, 20H, 51H, 75H, 19H, 47H, 10H, 40H, 98H, 55H, 77H
    DB 98H, 35H, 52H, 3FH, 00H, 00H, 00H, 80H, 76H, 32H, 44H, 00H, 99H, 99H, 99H, 99H
    DB 99H, 7EH

; BASIC_COLD_START - Cold-start entry for BASIC 1.2.
; usage: trace
BASIC_COLD_START:

; LL: Cold-start initialization: sets paging, tests RAM, initializes hardware, and rebuilds RAM-resident system routines.
    DI
    IM 1
    LD A,40H
    OUT (02H),A
    JP 0233H
    LD A,C0H
    OUT (02H),A
    JP F13DH
    LD A,40H

; KL: Memory paging: S U V S page layout.
    OUT (02H),A
    JP C241H

LC241:
    LD A,50H

; KL: Memory paging: U U V S page layout.
    OUT (02H),A
    XOR A
    LD BC,0460H

LC249:
    OUT (C),A
    INC C
    DJNZ C249H
    LD HL,C545H
    LD B,10H

LC253:
    LD A,B
    DEC A
    OUT (70H),A
    LD A,(HL)
    OUT (71H),A
    INC HL
    DJNZ C253H
    LD A,80H

; KL: Send one STROBE pulse.
    OUT (06H),A
    LD A,(0B22H)
    INC A
    JR NZ,C26BH
    LD (0B21H),A
    DB 3EH

LC26B:
    EX AF,AF'
    INC A
    JR Z,C2BBH
    LD SP,BFFFH
    LD HL,0000H
    CALL C33EH
    JR Z,C27FH

LC27A:
    DEC A
    OUT (00H),A
    JR C27AH

LC27F:
    LD SP,16ACH
    LD HL,8000H
    CALL C33EH
    JR Z,C28EH
    LD A,88H
    OUT (C),A

LC28E:
    LD A,70H

; KL: Memory paging: U U U S page layout.
    OUT (02H),A
    LD HL,4000H
    CALL C33EH
    CALL Z,C33EH
    DEC HL
    LD (0B19H),HL
    LD A,40H
    OUT (02H),A
    LD SP,BFFFH
    JP 02A9H
    LD A,80H
    OUT (02H),A
    LD HL,C000H
    CALL 0338H
    LD A,00H
    JR Z,C2B8H
    DEC A

LC2B8:
    EX AF,AF'
    JR C2C2H

LC2BB:
    LD H,A
    LD L,A

LC2BD:
    DEC HL
    LD A,H
    OR L
    JR NZ,C2BDH

LC2C2:
; WARM_RESET - Warm-reset path that attempts to preserve user memory.
; usage: trace
WARM_RESET:
    LD A,40H
    OUT (02H),A
    JP 02C9H
    LD A,C0H
    OUT (02H),A
    LD SP,16ACH
    JP F000H
    XOR A
    LD (0B11H),A
    OUT (03H),A

; KL: Clear cursor/sound interrupt.
    OUT (07H),A
    OUT (58H),A
    OUT (59H),A
    OUT (5AH),A
    OUT (5BH),A
    OUT (00H),A
    LD (0B4FH),A
    LD A,80H
    OUT (06H),A
    LD (0B13H),A
    LD HL,C555H
    LD BC,0470H

LC2F5:
    PUSH BC
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    PUSH HL
    EX DE,HL
    CALL C321H
    POP HL
    POP BC
    LD A,C

; KL: Memory paging: U U U S page layout.
    OUT (02H),A
    DJNZ C2F5H

; CARTRIDGE_AUTOSTART - Checks for the MOSP cartridge signature and transfers control to it.
; usage: trace
CARTRIDGE_AUTOSTART:

; KL: Cartridge autostart check; if the side cartridge begins with the MOPS signature, control passes to its fifth byte.
    LD A,60H
    OUT (02H),A
    JP 030DH
    LD A,20H

; KL: Memory paging: S U U C page layout.
    OUT (02H),A
    LD HL,C000H
    LD DE,0334H
    LD B,04H

LC319:
    LD A,(DE)
    CP (HL)
    JR NZ,C322H
    INC DE
    INC HL
    DJNZ C319H

LC321:
; JUMP_HL - Transfers control to the address in HL.
; usage: call
JUMP_HL:
    JP (HL)

LC322:
    LD A,60H
    OUT (02H),A
    JP C329H

LC329:
    LD A,70H

; KL: Memory paging: U U U S page layout.
    OUT (02H),A
    LD (0003H),A
    EI
    JP D9EFH

; KL: MOSP signature text used to recognize autostart cartridges.
    DB 4DH, 4FH, 50H, 53H
    PUSH HL
    CALL 0348H
    JR C342H

LC33E:
; MEMORY_TEST - Memory-test entry used during initialization.
; usage: trace
MEMORY_TEST:
    PUSH HL
    CALL C348H

LC342:
    POP DE
    RET NZ
    EX DE,HL
    LD A,AAH
    DB 01H

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
    JR C358H

; RST30_DISPATCH - Dispatches an operating-system function requested through RST 30H.
; usage: trace
RST30_DISPATCH:

; KL: RST 30H entry point.
    PUSH HL
    PUSH IX
    PUSH IY
    EXX
    PUSH BC
    PUSH DE
    PUSH HL
    EXX
    EX AF,AF'
    CALL C37EH

; RST30_RETURN - Common return path for RST 30H operating-system calls.
; usage: trace
RST30_RETURN:

; KL: RST 30H return path.
    EXX
    POP HL
    POP DE
    POP BC
    EXX
    POP IY
    POP IX
    POP HL
    JP 0B37H

LC37E:
    PUSH AF
    PUSH BC
    PUSH DE
    LD C,A
    OR A
    EX AF,AF'
    LD HL,0B01H
    LD A,(HL)
    CP 02H
    JR NZ,C38DH
    DEC (HL)

LC38D:
    LD DE,0B00H
    LD A,C
    RLCA
    JR C,C397H
    LD DE,0B08H

LC397:
    AND E0H
    RLCA
    RLCA
    RLCA
    LD B,A
    LD A,C
    AND 0FH
    LD C,A
    LD A,B
    CP 07H
    JR Z,C3D8H
    LD L,B
    XOR A
    LD H,A
    ADD HL,DE
    DEC A
    CP (HL)
    JR NZ,C3B3H
    LD A,C
    CP 03H
    JR NC,C3D0H

LC3B3:
    LD A,(HL)
    RES 7,A
    CP 07H
    JR C,C3C0H
    LD A,FEH

LC3BC:
    POP DE
    POP BC
    JR C3FCH

LC3C0:
    BIT 7,(HL)
    JR Z,C3CAH
    LD HL,F166H

LC3C7:
    JP FFF0H

LC3CA:
    LD A,C
    CP 03H
    JR NC,C3D0H
    LD B,(HL)

LC3D0:
    LD A,B
    CP 06H
    LD HL,F16CH
    JR Z,C3C7H

LC3D8:
    LD DE,C55DH
    LD L,B
    LD H,00H
    ADD HL,HL
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    EX DE,HL
    LD A,(HL)
    DEC A
    CP C
    LD A,FFH
    JR C,C3BCH
    INC HL
    EX DE,HL
    LD L,C
    LD H,00H
    ADD HL,HL
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    EX DE,HL
    EX AF,AF'
    POP DE
    POP BC
    CALL C321H

LC3FC:
    OR A
    POP HL
    RET Z
    PUSH AF
    LD A,(0B20H)
    INC A
    JR Z,C410H
    PUSH BC
    PUSH DE
    LD A,H
    LD HL,F1E6H
    JR C3C7H
    POP DE
    POP BC

LC410:
    POP AF
    RET

; IRQ_HANDLER - Main interrupt service routine entered from the RAM RST 38H stub.
; usage: trace
IRQ_HANDLER:

; KL: RST 38H interrupt entry; interrupt mode 1 dispatches here.
    LD A,(0003H)
    PUSH AF
    PUSH HL
    PUSH DE
    PUSH BC
    PUSH IX
    PUSH IY
    EX AF,AF'
    PUSH AF
    EXX
    PUSH HL
    PUSH DE
    PUSH BC
    CALL C437H
    POP BC
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
    POP AF
    JP 0B41H

LC437:
; IRQ_CORE - Core interrupt dispatch after the entry setup.
; usage: trace
IRQ_CORE:

; KL: Core interrupt handler.
    LD A,FFH

; KL: Set INTFLAG while servicing an interrupt.
    LD (0B20H),A
    LD HL,(0B1DH)

; KL: Increment HL.
    INC HL
    LD (0B1DH),HL

; KL: Load BORDER system variable into A.
    LD A,(0B4FH)
    OUT (00H),A
    IN A,(59H)
    LD C,A
    BIT 4,A
    SET 4,C

; KL: Clear cursor/sound interrupt.
    OUT (07H),A
    CALL Z,C47DH
    LD A,C
    OR F0H
    INC A
    JR Z,C478H
    LD A,(0B1FH)
    RRCA
    RRCA
    LD H,C
    LD L,C
    LD BC,0458H
    LD D,A

LC465:
    XOR A
    RRC D
    RRA
    RRC H
    JR C,C46FH
    OUT (C),A

LC46F:
    INC C
    DJNZ C465H
    LD C,L
    LD HL,C478H
    JR C4A3H

LC478:
    XOR A
    LD (0B20H),A
    RET

LC47D:
    PUSH BC
    LD A,(0B10H)
    LD C,A
    LD B,04H

LC484:
    RR C
    PUSH BC
    JR C,C499H
    LD A,04H
    SUB B
    LD B,A
    LD C,00H
    LD HL,C499H
    PUSH HL
    PUSH HL
    PUSH HL
    PUSH HL
    JP C3D8H

LC499:
    LD A,70H
    OUT (02H),A
    POP BC
    DJNZ C484H
    LD HL,C4AAH

LC4A3:
    PUSH HL
    LD HL,F227H
    JP FFF0H
    POP BC
    RET

; KERNEL_JUMP_TABLE - Counted jump table for kernel functions.
; usage: trace,data
KERNEL_JUMP_TABLE:

; KL: Kernel routine jump table; first byte is the routine count, followed by routine addresses.
    DB 05H, 09H, C5H, B7H, C4H, D0H, C4H, E2H, C4H, 0EH, C5H

; HI_MEM_SET - Reserves memory above BASIC's usable high-memory limit.
; usage: trace,call
HI_MEM_SET:

; KL: HI_MEM_SET expects the number of bytes to reserve above HI_MEM in DE; on success DE returns the new HI_MEM+1.
    LD HL,(0B19H)
    LD A,FBH
    OR A
    SBC HL,DE
    RET C
    PUSH HL
    LD DE,1EACH
    SBC HL,DE
    POP HL
    RET C
    LD D,H
    LD E,L
    INC DE
    LD (0B19H),HL
    XOR A
    RET

; SLOT_ASN - Assigns an expansion-card unit to function class 6.
; usage: trace,call
SLOT_ASN:
    LD HL,0B07H
    INC B
    JR Z,C4D9H
    LD HL,0B0FH

LC4D9:
    PUSH HL
    CALL C50EH
    POP HL
    OR A
    RET NZ
    LD (HL),C
    RET

; IO_ASN - Assigns an input or output device to a function class.
; usage: trace,call
IO_ASN:
    LD HL,0B00H
    INC D
    JR Z,C4EBH
    LD HL,0B08H

LC4EB:
    PUSH HL
    LD A,06H
    CP B
    JR C,C50AH
    LD E,B
    LD D,00H
    ADD HL,DE
    LD E,(HL)
    INC E
    JR Z,C50AH
    POP DE
    PUSH HL
    EX DE,HL
    CP C
    JR C,C50AH
    LD E,C
    XOR A
    LD D,A
    ADD HL,DE
    LD E,(HL)
    INC E
    JR Z,C50AH
    POP HL
    LD (HL),C
    RET

LC50A:
    POP HL
    LD A,FEH
    RET

LC50E:
; SLOT_NUM - Finds the slot containing a specified expansion-card unit.
; usage: trace,call
SLOT_NUM:
    PUSH DE
    LD IX,0040H
    LD B,04H

LC515:
    PUSH IX
    POP HL
    PUSH BC
    LD A,(DE)
    INC A
    LD B,A

LC51C:
    LD A,(DE)
    CP (HL)
    JR NZ,C537H
    INC HL
    INC DE
    DJNZ C51CH
    POP BC
    PUSH IX
    POP HL
    LD DE,0007H
    ADD HL,DE
    LD A,(HL)
    CP C
    JR NZ,C538H
    LD A,04H
    SUB B
    LD C,A
    XOR A

LC535:
    POP DE
    RET

LC537:
    POP BC

LC538:
    LD DE,0030H
    ADD IX,DE
    POP DE
    PUSH DE
    DJNZ C515H
    LD A,FDH
    JR C535H

; KL: Initial values for the 6845 video controller registers after reset.
    DB FFH, 0EH, 00H, 00H, 03H, 03H, 03H, 00H, 42H, 3CH, 02H, 4DH, 32H, 4BH, 40H, 63H

; KL: Startup initialization routine table.
    DB F2H, C9H, ECH, D5H, 60H, D9H, E2H, D9H

; DEVICE_JUMP_TABLE_POINTERS - Pointers to video, keyboard, editor, sound, printer, cassette, and kernel jump tables.
; usage: trace,data
DEVICE_JUMP_TABLE_POINTERS:
    DB 74H, C9H, E3H, D5H, 98H, CFH, 2AH, D9H, FFH, D8H, BBH, D9H, 00H, 00H, ACH, C4H
    DB EBH, E5H, D5H, C5H, E5H, D5H
    EX DE,HL
    LD HL,(0B19H)
    OR A
    SBC HL,DE
    LD A,FAH
    POP DE
    POP HL
    RET C
    LD C,(HL)
    EX DE,HL
    CALL C58EH
    POP BC
    POP DE
    POP HL
    OR A
    RET NZ
    CPI
    RET PO
    JR C56EH

LC58E:
    JP (HL)

LC58F:
    PUSH HL
    PUSH DE
    PUSH BC
    PUSH HL
    PUSH DE
    LD HL,(0B19H)
    OR A
    SBC HL,DE
    LD A,FAH
    POP DE
    POP HL
    RET C
    CALL C58EH
    EX AF,AF'
    LD A,C
    POP BC
    POP DE
    POP HL
    EX AF,AF'
    OR A
    RET NZ
    EX AF,AF'
    LD (DE),A
    XOR A
    EX DE,HL
    CPI
    EX DE,HL
    RET PO
    JR C58FH

; KL: Built-in character matrix table for character codes 32-127, ten bytes per character.
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 18H, 18H, 18H, 18H, 18H
    DB 00H, 18H, 00H, 00H, 00H, 36H, 36H, 36H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 36H
    DB 36H, 7FH, 36H, 7FH, 36H, 36H, 00H, 00H, 00H, 18H, 3EH, 58H, 3CH, 1AH, 7CH, 18H
    DB 00H, 00H, 00H, 60H, 66H, 0CH, 18H, 30H, 66H, 06H, 00H, 00H, 00H, 10H, 28H, 28H
    DB 30H, 54H, 48H, 34H, 00H, 00H, 00H, 18H, 18H, 30H, 00H, 00H, 00H, 00H, 00H, 00H
    DB 00H, 0CH, 18H, 30H, 30H, 30H, 18H, 0CH, 00H, 00H, 00H, 30H, 18H, 0CH, 0CH, 0CH
    DB 18H, 30H, 00H, 00H, 00H, 00H, 10H, 54H, 38H, 38H, 54H, 10H, 00H, 00H, 00H, 00H
    DB 18H, 18H, 7EH, 18H, 18H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 18H, 18H
    DB 30H, 00H, 00H, 00H, 00H, 00H, 7CH, 7CH, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H
    DB 00H, 00H, 18H, 18H, 00H, 00H, 00H, 00H, 06H, 0CH, 18H, 30H, 60H, 00H, 00H, 00H
    DB 00H, 3CH, 66H, 6EH, 7EH, 76H, 66H, 3CH, 00H, 00H, 00H, 18H, 38H, 18H, 18H, 18H
    DB 18H, 18H, 00H, 00H, 00H, 3CH, 66H, 06H, 1CH, 30H, 60H, 7EH, 00H, 00H, 00H, 7EH
    DB 06H, 0CH, 1CH, 06H, 46H, 3CH, 00H, 00H, 00H, 0CH, 1CH, 2CH, 4CH, 7EH, 0CH, 0CH
    DB 00H, 00H, 00H, 7EH, 60H, 7CH, 06H, 06H, 46H, 3CH, 00H, 00H, 00H, 3CH, 60H, 60H
    DB 7CH, 66H, 66H, 3CH, 00H, 00H, 00H, 7EH, 06H, 0CH, 18H, 30H, 60H, 60H, 00H, 00H
    DB 00H, 3CH, 66H, 66H, 3CH, 66H, 66H, 3CH, 00H, 00H, 00H, 3CH, 66H, 66H, 3EH, 06H
    DB 0CH, 38H, 00H, 00H, 00H, 00H, 00H, 18H, 18H, 00H, 18H, 18H, 00H, 00H, 00H, 00H
    DB 00H, 18H, 18H, 00H, 18H, 18H, 30H, 00H, 00H, 06H, 0CH, 18H, 30H, 18H, 0CH, 06H
    DB 00H, 00H, 00H, 00H, 00H, 7CH, 00H, 7CH, 00H, 00H, 00H, 00H, 00H, 30H, 18H, 0CH
    DB 06H, 0CH, 18H, 30H, 00H, 00H, 00H, 3CH, 66H, 06H, 0CH, 18H, 00H, 18H, 00H, 00H
    DB 00H, 3EH, 63H, 67H, 6BH, 6FH, 60H, 3CH, 00H, 00H, 00H, 1CH, 3EH, 63H, 63H, 7FH
    DB 63H, 63H, 00H, 00H, 00H, 7EH, 63H, 63H, 7EH, 63H, 63H, 7EH, 00H, 00H, 00H, 3EH
    DB 63H, 60H, 60H, 60H, 63H, 3EH, 00H, 00H, 00H, 7EH, 33H, 33H, 33H, 33H, 33H, 7EH
    DB 00H, 00H, 00H, 7EH, 60H, 60H, 7CH, 60H, 60H, 7EH, 00H, 00H, 00H, 7EH, 60H, 60H
    DB 7CH, 60H, 60H, 60H, 00H, 00H, 00H, 3EH, 63H, 60H, 60H, 67H, 63H, 3EH, 00H, 00H
    DB 00H, 63H, 63H, 63H, 7FH, 63H, 63H, 63H, 00H, 00H, 00H, 3CH, 18H, 18H, 18H, 18H
    DB 18H, 3CH, 00H, 00H, 00H, 06H, 06H, 06H, 06H, 66H, 66H, 3CH, 00H, 00H, 00H, 63H
    DB 66H, 6CH, 78H, 6CH, 66H, 63H, 00H, 00H, 00H, 60H, 60H, 60H, 60H, 60H, 60H, 7EH
    DB 00H, 00H, 00H, 63H, 77H, 6BH, 63H, 63H, 63H, 63H, 00H, 00H, 00H, 66H, 66H, 76H
    DB 6EH, 66H, 66H, 66H, 00H, 00H, 00H, 3EH, 63H, 63H, 63H, 63H, 63H, 3EH, 00H, 00H
    DB 00H, 7EH, 63H, 63H, 7EH, 60H, 60H, 60H, 00H, 00H, 00H, 3EH, 63H, 63H, 63H, 6BH
    DB 67H, 3EH, 01H, 00H, 00H, 7EH, 63H, 63H, 7EH, 6CH, 66H, 63H, 00H, 00H, 00H, 3EH
    DB 63H, 60H, 3EH, 03H, 63H, 3EH, 00H, 00H, 00H, 7EH, 5AH, 18H, 18H, 18H, 18H, 18H
    DB 00H, 00H, 00H, 63H, 63H, 63H, 63H, 63H, 63H, 3EH, 00H, 00H, 00H, 63H, 63H, 63H
    DB 63H, 36H, 1CH, 08H, 00H, 00H, 00H, 63H, 63H, 63H, 6BH, 6BH, 3EH, 14H, 00H, 00H
    DB 00H, 66H, 66H, 3CH, 18H, 3CH, 66H, 66H, 00H, 00H, 00H, 66H, 66H, 3CH, 18H, 18H
    DB 18H, 18H, 00H, 00H, 00H, 7EH, 06H, 0CH, 18H, 30H, 60H, 7EH, 00H, 00H, 00H, 3CH
    DB 30H, 30H, 30H, 30H, 30H, 3CH, 00H, 00H, 00H, 00H, 60H, 30H, 18H, 0CH, 06H, 00H
    DB 00H, 00H, 00H, 3CH, 0CH, 0CH, 0CH, 0CH, 0CH, 3CH, 00H, 00H, 00H, 18H, 3CH, 66H
    DB 42H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 7EH, 00H, 00H
    DB 00H, 30H, 30H, 18H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 3CH, 06H, 3EH
    DB 66H, 3EH, 00H, 00H, 00H, 60H, 60H, 7CH, 66H, 66H, 66H, 7CH, 00H, 00H, 00H, 00H
    DB 00H, 1EH, 30H, 30H, 30H, 1EH, 00H, 00H, 00H, 06H, 06H, 3EH, 66H, 66H, 66H, 3EH
    DB 00H, 00H, 00H, 00H, 00H, 3CH, 66H, 7EH, 60H, 3CH, 00H, 00H, 00H, 0CH, 18H, 18H
    DB 3CH, 18H, 18H, 18H, 00H, 00H, 00H, 00H, 00H, 3EH, 66H, 66H, 66H, 3EH, 06H, 3CH
    DB 00H, 60H, 60H, 7CH, 66H, 66H, 66H, 66H, 00H, 00H, 00H, 18H, 00H, 38H, 18H, 18H
    DB 18H, 3CH, 00H, 00H, 00H, 18H, 00H, 38H, 18H, 18H, 18H, 18H, 18H, 70H, 00H, 60H
    DB 60H, 66H, 6CH, 78H, 6CH, 66H, 00H, 00H, 00H, 18H, 18H, 18H, 18H, 18H, 18H, 18H
    DB 00H, 00H, 00H, 00H, 00H, 76H, 6BH, 6BH, 6BH, 6BH, 00H, 00H, 00H, 00H, 00H, 7CH
    DB 66H, 66H, 66H, 66H, 00H, 00H, 00H, 00H, 00H, 3CH, 66H, 66H, 66H, 3CH, 00H, 00H
    DB 00H, 00H, 00H, 7CH, 66H, 66H, 66H, 7CH, 60H, 60H, 00H, 00H, 00H, 3EH, 66H, 66H
    DB 66H, 3EH, 06H, 06H, 00H, 00H, 00H, 36H, 38H, 30H, 30H, 30H, 00H, 00H, 00H, 00H
    DB 00H, 1EH, 30H, 1CH, 06H, 3CH, 00H, 00H, 00H, 18H, 18H, 3CH, 18H, 18H, 18H, 0CH
    DB 00H, 00H, 00H, 00H, 00H, 66H, 66H, 66H, 66H, 3EH, 00H, 00H, 00H, 00H, 00H, 66H
    DB 66H, 66H, 3CH, 18H, 00H, 00H, 00H, 00H, 00H, 63H, 63H, 6BH, 3EH, 14H, 00H, 00H
    DB 00H, 00H, 00H, 66H, 3CH, 18H, 3CH, 66H, 00H, 00H, 00H, 00H, 00H, 66H, 66H, 66H
    DB 66H, 3EH, 06H, 3CH, 00H, 00H, 00H, 7EH, 0CH, 18H, 30H, 7EH, 00H, 00H, 00H, 0EH
    DB 18H, 18H, 70H, 18H, 18H, 0EH, 00H, 00H, 00H, 18H, 18H, 18H, 00H, 18H, 18H, 18H
    DB 00H, 00H, 00H, 70H, 18H, 18H, 0EH, 18H, 18H, 70H, 00H, 00H, 00H, 00H, 00H, 33H
    DB 6BH, 66H, 00H, 00H, 00H, 00H, 00H, 7EH, 7EH, 7EH, 7EH, 7EH, 7EH, 7EH, 00H, 00H

; VIDEO_JUMP_TABLE - Counted jump table for video OS functions.
; usage: trace,data
VIDEO_JUMP_TABLE:

; KL: VIDEO routine jump table; first byte is the routine count, followed by routine addresses.
    DB 0DH, A9H, C9H, 94H, CCH, 86H, CCH, 4BH, CFH, F4H, C9H, 49H, CAH, DAH, CAH, D7H
    DB CAH, F3H, CBH, FFH, CBH, 48H, CDH, 2CH, CFH, 38H, CAH

LC98F:
; CALL_WITH_SYS_PAGED - Pages SYS into page 3, invokes the routine in HL, then restores paging.
; usage: trace
CALL_WITH_SYS_PAGED:
    POP HL
    LD A,(0003H)
    PUSH AF
    LD A,50H
    LD (0003H),A
    OUT (02H),A
    PUSH HL
    LD HL,C9A1H
    EX (SP),HL
    JP (HL)
    EX AF,AF'
    POP AF
    LD (0003H),A
    OUT (02H),A
    EX AF,AF'
    RET

LC9AA:
; JUMP_TABLE_DISPATCH - Selects a target from a ROM jump table.
; usage: trace
JUMP_TABLE_DISPATCH:
    ADD A,A
    LD E,A
    LD D,00H
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    RET

LC9B3:
    LD HL,03FFH
    OR A
    SBC HL,BC
    RET

LC9BA:
    LD HL,03BFH
    OR A
    SBC HL,DE
    RET

LC9C1:
    LD A,(0B73H)
    LD B,A
    INC B

LC9C6:
; SHIFT_HL_RIGHT - Divides HL by a power of two.
; usage: call
SHIFT_HL_RIGHT:
    SRL H
    RR L
    DJNZ C9C6H
    RET

LC9CD:
    CALL CB6DH

LC9D0:
    CALL CC05H
    LD (0B93H),A
    LD DE,0040H

LC9D9:
    CALL CB8FH
    CALL CC0DH

LC9DF:
    LD (0B94H),A

LC9E2:
    LD A,(0B75H)
    LD C,A
    LD HL,(0B76H)
    RET

; KL: Initial palette bytes when the hardware color switch is on.
    DB 00H, 50H, 44H, 41H, 00H, 55H, 50H, 44H

; SET_4_COLOR_MODE - Selects the four-color video mode.
; usage: trace,call
SET_4_COLOR_MODE:
    LD C,01H

; VID_MODE - Sets the video display mode.
; usage: trace,call
VID_MODE:

; KL: VMODE routine.
    LD A,C
    CP 03H
    LD A,F7H
    RET NC
    XOR A
    LD (0B4BH),A
    LD (0B4EH),A
    LD (0B50H),A
    INC A
    LD (0B4CH),A
    CP C
    JR NC,CA15H
    IN A,(59H)
    RLCA
    RLCA
    LD A,0FH
    JR NC,CA15H
    LD A,0CH

LCA15:
    LD (0B4DH),A
    LD A,C
    LD (0B73H),A
    PUSH BC
    CALL CA49H
    POP BC
    LD A,C
    LD HL,0B13H
    XOR (HL)
    AND FCH
    XOR C
    LD (HL),A
    OUT (06H),A
    IN A,(59H)
    RLCA
    RLCA
    LD DE,C9EEH
    JR NC,CA38H
    LD DE,C9EAH

LCA38:
; PAL_DEF - Defines the active video palette.
; usage: trace,call
PAL_DEF:

; KL: PAL routine.
    LD A,(DE)
    OUT (60H),A
    INC DE
    LD A,(DE)
    OUT (61H),A
    INC DE
    LD A,(DE)
    OUT (62H),A
    INC DE
    LD A,(DE)
    OUT (63H),A
    XOR A
    RET

LCA49:
; VID_CLS - Clears the video display.
; usage: trace,call
VID_CLS:

; KL: CLS routine.
    CALL C98FH
    CALL CFD4H
    CALL CC05H
    LD HL,8000H
    LD DE,8001H
    LD (HL),A
    LD BC,3BFFH
    LDIR
    CALL CBFFH
    LD B,A
    LD C,A
    LD D,A
    LD E,A

LCA65:
; PIXEL_ADDRESS - Converts physical pixel coordinates to a video-memory address and bit mask.
; usage: call
PIXEL_ADDRESS:
    LD (0B7EH),DE

LCA69:
    LD (0B7CH),BC
    LD HL,(0B7CH)
    PUSH HL
    LD B,04H
    CALL C9C6H
    EX (SP),HL
    CALL C9C1H
    LD (0B78H),HL
    LD C,A
    LD A,L
    LD HL,CAB4H
    ADD HL,BC
    AND 07H
    INC A
    LD B,A
    LD A,(HL)
    DB 1EH

LCA89:
    RRCA
    DJNZ CA89H
    LD (0B75H),A
    LD DE,(0B7EH)
    LD HL,03BFH
    XOR A
    SBC HL,DE
    LD B,02H
    CALL C9C6H
    LD (0B7AH),HL
    LD H,L
    SRL H
    RRA
    SRL H
    RRA
    LD L,A
    POP BC
    ADD HL,BC
    LD DE,8000H
    ADD HL,DE
    LD (0B76H),HL
    XOR A
    RET
    ADD A,B
    ADC A,B
    XOR D

LCAB7:
    JR NC,CAC5H
    AND 01H
    JR Z,CAC0H
    RET

LCABE:
    JR NC,CAC8H

LCAC0:
    LD A,(0B94H)
    JP (IY)

LCAC5:
    AND 02H
    RET NZ

LCAC8:
    LD A,(0B93H)
    JP (IY)
    AND (HL)
    XOR (HL)
    AND C
    XOR (HL)
    LD (HL),A
    RET
    AND C
    OR (HL)
    LD (HL),A
    RET

; B_REL - Moves the graphics pen by a relative offset.
; usage: trace,call
B_REL:

; KL: BREL routine.
    CALL CC7AH

; B_ABS - Moves the graphics pen to an absolute position.
; usage: trace,call
B_ABS:

; KL: BABS routine.
    CALL C98FH
    LD A,F9H
    CALL C9B3H
    CALL NC,C9BAH
    RET C
    LD A,(0B74H)
    OR A
    JP Z,CA65H

; DRAW_LINE - Core line-drawing routine.
; usage: trace
DRAW_LINE:
    EXX
    LD BC,(0B78H)
    LD DE,(0B7AH)
    PUSH BC
    PUSH DE
    CALL C9CDH
    EXX
    CALL CA65H
    LD BC,(0B78H)
    LD DE,(0B7AH)
    XOR A
    EX DE,HL
    POP DE
    CALL CB61H
    POP DE
    PUSH HL
    LD H,B
    LD L,C
    CALL CB61H
    PUSH HL
    LD HL,CBDBH
    CALL C9AAH
    LD (0B86H),DE
    POP HL
    POP DE
    PUSH HL
    SBC HL,DE
    POP HL
    JR C,CB28H
    EX DE,HL

LCB28:
    LD B,H
    LD C,L
    PUSH DE
    ADC A,00H
    SBC HL,DE
    PUSH HL
    SRL D
    RR E
    ADD HL,DE
    PUSH HL
    LD HL,CBE3H
    CALL C9AAH
    LD (0B84H),DE
    POP HL
    POP DE
    EXX
    EX (SP),HL
    LD A,H
    LD B,L
    INC B
    INC A
    EX AF,AF'
    POP HL
    JR CB50H

LCB4C:
    EX AF,AF'

LCB4D:
    DB CDH, A8H

LCB4F:
    DB CBH

LCB50:
    LD A,(0B83H)
    RLCA
    LD (0B83H),A
    CALL CABEH
    DJNZ CB4DH
    EX AF,AF'
    DEC A
    JR NZ,CB4CH
    RET

LCB61:
    SBC HL,DE
    JR NC,CB6BH
    EX DE,HL
    LD HL,0001H
    SBC HL,DE

LCB6B:
    RLA
    RET

LCB6D:
    LD A,(0B4CH)
    DEC A
    AND 0FH
    LD C,A
    LD HL,CB7FH
    LD B,00H
    ADD HL,BC
    LD A,(HL)
    LD (0B83H),A
    RET

; KL: Line style bit-pattern table.
    DB FFH, AAH, CCH, EEH, 88H, DAH, E4H, F6H, FAH, FEH, FCH, F8H, F0H, EAH, FFH, FFH
    DB D5H, 3AH, 4BH, 0BH, E6H, 03H, 21H, A0H, CBH, CDH, AAH, C9H, D5H, FDH, E1H, D1H
    DB C9H, CEH, CAH, D3H, CAH, CDH, CAH, CFH, CAH, D9H, CBH, 7CH, 28H, 08H, 09H, D9H
    DB DDH, 2AH, 84H, 0BH, DDH, E9H, 19H, D9H, DDH, 2AH, 86H, 0BH, DDH, E9H, B7H, EDH
    DB 52H, CBH, 01H, D0H, 2DH, C9H, CBH, 09H, 30H, 01H, 2CH, B7H, EDH, 52H, C9H, CBH
    DB 01H, 30H, 01H, 2DH, 19H, C9H, 19H, CBH, 09H, D0H, 2CH, C9H, D5H, CBH
    ADC A,CBH
    PUSH BC
    RES 7,L
    SET 2,(HL)
    SET 2,E
    SET 0,B
    SET 2,E
    SET 2,(HL)
    SET 1,D
    SET 0,B
    SET 1,D
    DB CBH

; B_ON - Lowers the graphics pen.
; usage: trace,call
B_ON:

; KL: BON routine.
    CALL C98FH
    CALL C9D9H
    CALL CAC0H
    LD A,FFH
    DB 26H

LCBFF:
; B_OFF - Raises the graphics pen.
; usage: trace,call
B_OFF:

; KL: BOFF routine.
    XOR A
    LD (0B74H),A
    XOR A
    RET

LCC05:
    LD A,(0B4EH)
    CALL CC10H
    LD B,A
    RET

LCC0D:
    LD A,(0B4DH)

LCC10:
    LD HL,(0B73H)
    SRL L
    JR C,CC20H
    SRL L
    JR C,CC31H
    AND 01H
    RRA
    SBC A,A
    RET

LCC20:
    AND 03H
    RRA
    RL L
    RRA
    SBC A,A
    RR L
    RRA
    SRA A
    SRA A
    SRA A
    RET

LCC31:
    AND 0FH
    LD H,04H

LCC35:
    RRA
    RR L
    SRA L
    DEC H
    JR NZ,CC35H
    LD A,L
    RET

LCC3F:
; READ_PIXEL_COLOR - Reads the color code of the most recently addressed pixel.
; usage: call
READ_PIXEL_COLOR:
    CALL C9E2H
    LD A,(0B73H)
    LD B,A
    LD A,C
    AND (HL)
    SRL B
    JR C,CC56H
    SRL B
    JR C,CC65H
    ADD A,FFH
    RLA
    AND 01H
    RET

LCC56:
    ADD A,F0H
    RL C
    AND 0FH
    ADD A,FFH
    RLA
    RR C
    RLA
    AND 03H
    RET

LCC65:
    ADD A,C0H
    RLA
    AND 7FH
    ADD A,E0H
    RLA
    AND 3FH
    ADD A,F0H
    RLA
    AND 1FH
    ADD A,F8H
    RLA
    AND 0FH
    RET

LCC7A:
    LD HL,(0B7CH)
    ADD HL,BC
    LD B,H
    LD C,L
    LD HL,(0B7EH)
    ADD HL,DE
    EX DE,HL
    RET

; VID_BKOUT - Writes a block of text to the video device.
; usage: trace,call
VID_BKOUT:
    EX DE,HL

LCC87:
    PUSH HL
    PUSH BC
    LD C,(HL)
    CALL CC94H
    POP BC
    POP HL
    CPI
    RET PO
    JR CC87H

LCC94:
; VID_CHOUT - Writes one character to the video device.
; usage: trace,call
VID_CHOUT:
    CALL C98FH
    LD A,C
    CP 20H
    JR NC,CCA8H
    CP 0AH
    JP Z,CD31H
    CP 0DH
    JP Z,CD2BH
    XOR A
    RET

LCCA8:
    CP E0H
    LD A,00H
    RET NC
    EXX
    LD DE,FFDCH
    LD A,(0B73H)
    LD B,A
    LD A,01H
    INC B

LCCB8:
    ADD A,A
    DJNZ CCB8H
    LD (0B82H),A
    LD L,A
    ADD A,A
    ADD A,L
    ADD A,A
    ADD A,L
    LD C,A
    XOR A
    CALL CC7AH
    CALL C9BAH
    RET C
    CALL C9B3H
    JR NC,CCD7H
    CALL CD2BH
    CALL CD31H

LCCD7:
; VID_CHOUT_AT - Writes character C at the supplied pixel position without validation.
; usage: call
VID_CHOUT_AT:
    CALL C9D0H
    EXX
    LD DE,C474H
    BIT 7,C
    JR Z,CCE5H
    LD DE,0740H

LCCE5:
    RES 7,C
    LD B,00H
    LD H,B
    LD L,C
    ADD HL,HL
    ADD HL,HL
    ADD HL,BC
    ADD HL,HL
    ADD HL,DE
    LD B,0AH
    LD A,(0B50H)
    LD E,A
    DEC HL

LCCF7:
    INC HL
    LD C,(HL)
    EXX
    LD B,08H
    JR CD01H

LCCFE:
    CALL CBD6H

LCD01:
    EXX
    RL C
    LD A,E
    EXX
    CALL CAB7H
    DJNZ CCFEH
    CALL CBD3H
    LD B,07H

LCD10:
    CALL CBC0H
    DJNZ CD10H
    EXX
    DJNZ CCF7H
    LD A,(0B82H)
    ADD A,A
    ADD A,A
    ADD A,A
    LD C,A
    CALL CC7AH
    CALL C9B3H
    JR NC,CD2EH
    LD BC,CD31H
    PUSH BC

LCD2B:
    LD BC,0000H

LCD2E:
    JP CA69H

LCD31:
    LD DE,FFB4H
    CALL CC7AH
    XOR A
    CALL C9BAH
    RET C
    LD DE,FFD8H
    LD BC,0000H
    CALL CC7AH
    JP CA65H

; PAINT - Fills a closed graphics region.
; usage: trace,call
PAINT:

; KL: FILL routine.
    CALL C98FH
    LD (0B88H),SP
    LD IY,CACFH
    CALL CC3FH
    CALL CC10H
    LD (0B8AH),A
    LD E,A
    CALL CC0DH
    XOR E
    LD HL,0000H
    RET Z
    LD (0B91H),HL
    PUSH HL
    LD HL,(0B7CH)
    LD B,04H
    CALL C9C6H
    PUSH HL
    CALL C9DFH
    LD A,40H
    POP DE
    SUB E
    LD B,A
    LD (0B8DH),HL
    LD (0B8BH),BC
    LD B,01H
    JR CDA3H

LCD85:
    POP HL
    LD A,H
    OR L
    RET Z
    POP BC
    LD (0B91H),HL
    LD (0B8BH),BC
    POP HL
    LD DE,0040H
    ADD HL,DE
    LD (0B8DH),HL
    LD B,02H
    LD DE,BC00H
    SBC HL,DE
    JP NC,CE63H

LCDA3:
    EXX

LCDA4:
    CALL CE92H
    JR C,CD85H
    LD HL,(0B91H)
    LD A,H
    RES 7,H
    RES 6,H
    LD (0B8FH),HL
    EX DE,HL
    BIT 6,A
    JR Z,CDC2H
    RLCA
    EXX
    XOR B
    EXX
    AND 01H
    JP NZ,CE62H

LCDC2:
    LD HL,(0B8DH)
    LD BC,(0B8BH)
    LD A,(0B8AH)
    XOR (HL)
    AND C
    JR NZ,CE2FH
    LD A,40H
    SUB B
    INC A
    LD B,A
    LD DE,0000H
    JR CDE5H

LCDDA:
    LD A,(0B8AH)
    XOR (HL)
    AND C
    JR NZ,CDECH
    CALL CAC0H
    INC DE

LCDE5:
    RLC C
    JR NC,CDDAH
    DEC L
    DJNZ CDDAH

LCDEC:
    LD A,40H
    SUB B
    INC A
    LD B,A
    RRC C
    JR NC,CDF7H
    INC L
    DEC B

LCDF7:
    PUSH HL
    PUSH BC
    PUSH DE
    LD HL,(0B8DH)
    LD BC,(0B8BH)
    CALL CE7AH
    EX (SP),HL
    LD A,H
    OR L
    JR Z,CE0BH
    SET 7,H

LCE0B:
    ADD HL,DE
    EX (SP),HL

LCE0D:
    PUSH HL
    LD HL,(0B8FH)
    OR A
    SBC HL,DE
    POP DE
    EX (SP),HL
    PUSH AF
    LD A,80H
    BIT 7,H
    JR NZ,CE26H
    JR C,CE28H
    EXX
    RRC B
    RLC B
    EXX
    RRA

LCE26:
    XOR H
    LD H,A

LCE28:
    POP AF
    EX (SP),HL
    JR C,CE62H
    JR Z,CE62H
    EX DE,HL

LCE2F:
    CALL CE92H
    JP C,CD85H
    LD A,E
    OR A
    JR NZ,CE3AH
    DEC D

LCE3A:
    LD A,(0B8AH)
    XOR (HL)
    AND C
    JR Z,CE4CH
    DEC E
    JR Z,CE5EH

LCE44:
    RRC C
    JR NC,CE3AH
    INC L
    DEC B
    JR CE3AH

LCE4C:
    LD A,E
    OR A
    JR NZ,CE51H
    INC D

LCE51:
    LD (0B8FH),DE
    PUSH HL
    EX (SP),HL
    PUSH BC
    CALL CE7AH
    PUSH DE
    JR CE0DH

LCE5E:
    DEC D
    JP P,CE44H

LCE62:
    EXX

LCE63:
    DEC B
    JP Z,CD85H
    EXX
    LD HL,(0B8DH)
    LD DE,FF80H
    ADD HL,DE
    LD (0B8DH),HL
    BIT 7,H
    JP Z,CD85H
    JP CDA4H

LCE7A:
    LD DE,0000H

LCE7D:
    LD A,(0B8AH)
    XOR (HL)
    AND C
    RET NZ
    INC DE
    LD A,(0B94H)
    AND C
    XOR (HL)
    LD (HL),A
    RRC C
    JR NC,CE7DH
    INC L
    DJNZ CE7DH
    RET

LCE92:
    PUSH HL
    PUSH DE
    LD HL,FFEAH
    ADD HL,SP
    LD DE,(0B17H)
    SBC HL,DE
    JP NC,CEC9H
    PUSH BC
    EXX
    PUSH BC
    LD HL,(0B88H)
    DEC HL
    DEC HL
    DEC HL
    LD D,H
    LD E,L

LCEAC:
    LD C,L
    LD B,H
    LD HL,0009H
    ADD HL,SP
    OR A
    SBC HL,BC
    LD H,B
    LD L,C
    JR NZ,CECCH
    LD BC,000AH
    LDDR
    EX DE,HL
    LD SP,HL
    INC SP
    OR A
    EX DE,HL
    SBC HL,DE
    CCF
    POP BC
    EXX
    POP BC

LCEC9:
    POP DE
    POP HL
    RET

LCECC:
    CALL CEFEH
    ADD HL,BC
    LD BC,BC00H
    SBC HL,BC
    JR NC,CEDDH
    ADD HL,BC
    CALL CF14H
    JR Z,CEECH

LCEDD:
    CALL CEFDH
    OR A
    SBC HL,BC
    LD BC,8000H
    SBC HL,BC
    ADD HL,BC
    CALL NC,CF14H

LCEEC:
    EXX
    LD BC,0006H
    JR NZ,CEF7H
    LDDR
    JP CEACH

LCEF7:
    OR A
    SBC HL,BC
    JP CEACH

LCEFD:
    EXX

LCEFE:
    PUSH HL
    EXX
    POP HL
    LD D,(HL)
    DEC HL
    LD E,(HL)
    DEC HL
    DEC HL
    LD B,(HL)
    DEC HL
    LD A,(HL)
    AND 3FH
    DEC HL
    LD L,(HL)
    LD H,A
    LD A,B
    LD BC,0040H
    EX DE,HL
    RET

LCF14:
    LD C,A
    LD A,(0B8AH)
    LD B,A

LCF19:
    LD A,B
    XOR (HL)
    AND C
    RET Z
    DEC E
    JR Z,CF27H

LCF20:
    RRC C
    JR NC,CF19H
    INC HL
    JR CF19H

LCF27:
    DEC D
    JP P,CF20H
    RET

; CH_DEF - Defines a programmable character glyph.
; usage: trace,call
CH_DEF:

; KL: DEFC routine.
    LD A,C
    CP E0H
    JR NC,CF32H
    OR A

LCF32:
    LD A,F8H
    RET P
    RES 7,C
    LD B,00H
    LD L,C
    LD H,B
    ADD HL,HL
    ADD HL,HL
    ADD HL,BC
    ADD HL,HL
    LD BC,0740H
    ADD HL,BC
    EX DE,HL
    LD BC,000AH
    LDIR
    XOR A
    RET

; CH_POS - Rounds the graphics pen position to a normal character position.
; usage: trace,call
CH_POS:

; KL: BTEXT routine.
    PUSH BC
    LD HL,0000H
    DEC B
    JP M,CF54H
    LD L,B

LCF54:
    LD A,(0B73H)
    ADD A,04H
    LD B,A

LCF5A:
    ADD HL,HL
    DJNZ CF5AH
    PUSH HL
    LD DE,FFD8H
    LD HL,03BFH
    LD B,C
    DEC B
    JP M,CF6EH
    JR Z,CF6EH

LCF6B:
    ADD HL,DE
    DJNZ CF6BH

LCF6E:
    EX DE,HL
    POP BC
    EXX
    LD BC,(0B7CH)
    LD DE,(0B7EH)
    EXX
    POP HL
    LD A,L
    OR A
    JR NZ,CF83H
    EXX
    PUSH DE
    EXX
    POP DE

LCF83:
    LD A,H
    OR A
    JR NZ,CF8BH
    EXX
    PUSH BC
    EXX
    POP BC

LCF8B:
    LD A,F9H
    CALL C9B3H
    RET C
    CALL C9BAH
    RET C
    JP CA65H

; EDITOR_JUMP_TABLE - Counted jump table for editor OS functions.
; usage: trace,data
EDITOR_JUMP_TABLE:

; KL: EDITOR routine jump table; first byte is the routine count, followed by routine addresses.
    DB 05H, A3H, CFH, 52H, D0H, 41H, D0H, 1DH, D0H, 13H, D0H

; ED_INT - Editor interrupt routine that manages cursor blinking.
; usage: trace
ED_INT:
    DB 3EH, 50H, D3H, 02H, 21H, 48H, 0EH, EDH, 4BH, 49H, 0EH, 34H, 3EH, 94H, 96H, 28H
    DB 1CH, FEH, 80H, C0H, 77H, 3AH, 66H, 0BH, 16H, 7FH, B7H, 28H, 0CH, 16H, 9EH, 0FH
    DB 38H, 07H, 16H, 9FH, 0FH, 38H, 02H, 16H, 8FH, 7AH, C3H, 20H, D4H, 77H, C3H, 91H
    DB D4H

; EDITOR_INIT - Initializes or clears the editor workspace.
; usage: trace,call
EDITOR_INIT:
    DB 21H, 48H, 0EH, 06H, 20H, AFH, 77H, 23H, 10H, FCH, 21H, 01H, 01H, 22H, 49H, 0EH
    DB 21H, 00H, 01H, 11H, 01H, 01H, 01H, FFH, 05H, 36H, 20H, EDH, B0H, 3AH, 73H, 0BH
    DB 87H, 87H, 21H, 07H, D0H, 4FH, 09H, 11H, 68H, 0EH, 3EH, C3H, 12H, 13H, 0EH, 04H
    DB EDH, B0H, C9H, ADH, D3H, 40H, 01H, BCH, D3H, 20H, 02H, D9H, D3H, 10H, 04H

; CU_FIX - Stores the current cursor position.
; usage: trace,call
CU_FIX:

; KL: CFIX routine.
    LD HL,(0E49H)
    LD (0E4EH),HL
    LD A,80H
    JR D039H

; CU_POS - Positions the editor cursor.
; usage: trace,call
CU_POS:
    LD HL,(0E49H)
    LD A,B
    OR A
    JR Z,D02BH
    LD A,(0E6BH)
    CP B
    JR C,D03EH
    LD H,B

LD02B:
    LD A,C
    OR A
    JR Z,D035H
    LD A,18H
    CP C
    JR C,D03EH
    LD L,C

LD035:
    LD (0E49H),HL

LD038:
    XOR A

LD039:
    LD (0E4DH),A
    XOR A
    RET

LD03E:
    LD A,F6H
    RET

; ED_BKIN_OUT - Editor block input/output entry.
; usage: trace,call
ED_BKIN_OUT:
    EXX
    CALL D449H
    EXX
    LD HL,D0B7H
    JP P,C56DH
    LD HL,D058H
    JP C58FH

; ED_CHIN_OUT - Editor character input/output entry.
; usage: trace,call
ED_CHIN_OUT:
    CALL D449H
    JP P,D0B7H
    LD A,50H
    LD (0003H),A
    OUT (02H),A
    LD A,(0E4DH)
    RRCA
    JP C,D0FAH

LD066:
    CALL D39CH
    LD BC,(0E49H)
    PUSH BC
    LD DE,(0E4EH)

LD072:
    LD A,C
    CP E
    JR C,D07CH
    JR NZ,D081H
    LD A,B
    CP D
    JR NC,D08BH

LD07C:
    CALL D038H
    JR D08BH

LD081:
    DEC C
    JR Z,D07CH
    DEC HL
    BIT 7,(HL)
    LD D,01H
    JR NZ,D072H

LD08B:
    POP BC
    CALL D477H
    LD A,13H
    LD (0E48H),A
    LD HL,0B10H
    RES 2,(HL)
    RST 30H
    SUB C
    SET 2,(HL)
    PUSH AF
    PUSH BC
    LD BC,(0E49H)
    CALL D491H
    POP BC
    POP AF
    RET NZ
    LD A,C
    CP 1BH
    JR Z,D038H
    CP 0DH
    JR Z,D0EBH
    CALL D124H
    JR D066H

LD0B7:
    LD A,50H
    LD (0003H),A
    OUT (02H),A
    LD HL,D038H
    PUSH HL
    LD A,C
    CP 0DH
    JR NZ,D0CDH
    LD A,01H
    LD (0E4AH),A
    RET

LD0CD:
    CP 0AH
    JR NZ,D0E6H
    LD BC,(0E49H)

LD0D5:
    LD A,C
    CALL D39FH
    INC C
    JR C,D0D5H
    DEC C
    PUSH BC
    CALL D363H
    POP AF
    LD (0E4AH),A
    RET

LD0E6:
    CALL D124H
    XOR A
    RET

LD0EB:
    LD A,(0E4DH)
    RLCA
    LD BC,(0E4EH)
    CALL NC,D370H
    LD (0E49H),BC

LD0FA:
    LD BC,(0E49H)
    LD A,C
    CALL D39FH
    CP B
    JR C,D11CH
    CALL D384H
    INC B
    LD A,(0E6BH)
    CP B
    JR NC,D112H
    LD B,01H
    INC C

LD112:
    LD (0E49H),BC
    LD C,(HL)
    LD A,01H
    JP D039H

LD11C:
    CALL D363H
    LD C,0DH
    JP D038H

LD124:
; EDITOR_CHAR_DISPATCH - Processes a printable or editor control character.
; usage: trace
EDITOR_CHAR_DISPATCH:
    CP 20H
    JR C,D157H
    CP E0H
    JR NC,D157H
    EX AF,AF'
    CALL D384H
    EX AF,AF'
    LD (HL),A
    LD BC,(0E49H)
    PUSH BC
    CALL D420H
    POP BC
    LD A,C
    CALL D39FH
    CP B
    JR NC,D143H
    LD (HL),B

LD143:
    INC B
    LD A,(0E6BH)
    CP B
    JR NC,D152H
    LD B,01H
    INC C
    BIT 7,(HL)
    CALL Z,D2CBH

LD152:
    LD (0E49H),BC
    RET

LD157:
    LD HL,D170H
    LD B,0BH

LD15C:
    CP (HL)
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    JR Z,D167H
    DJNZ D15CH
    RET

LD167:
    LD BC,(0E49H)
    LD A,(0E6BH)
    EX DE,HL
    JP (HL)

; KL: Built-in editor function jump table; entries contain control character and routine address.
    DB 13H, 91H, D1H, 04H, 98H, D1H, 05H, 95H, D1H, 18H, 9EH, D1H, 16H, 05H, D2H, 07H
    DB 87H, D2H, 08H, 78H, D2H, 19H, FDH, D1H, 0EH, F7H, D1H, 0BH, A8H, D1H, 09H, CDH
    DB D1H
    DEC B
    JR NZ,D1A3H
    LD B,A
    DEC C
    JR D1A2H
    INC B
    CP B
    JR NC,D1A3H
    LD B,01H
    INC C
    LD A,C
    CP 19H

LD1A2:
    RET Z

LD1A3:
    LD (0E49H),BC
    RET
    PUSH BC
    CALL D4ADH
    POP BC
    CALL D384H
    DEC B
    LD A,(0E6BH)
    SUB B

LD1B5:
    LD (HL),20H
    INC HL
    DEC A
    JR NZ,D1B5H
    CALL D39CH
    LD (HL),B
    INC C
    INC HL

LD1C1:
    RET NC

LD1C2:
    LD B,(HL)
    LD A,C
    EXX
    CALL D31EH
    EXX
    RLC B
    JR D1C1H
    CALL D39CH
    EX AF,AF'
    LD A,B
    DEC A
    AND F8H
    ADD A,09H
    LD B,A
    LD A,(0E6BH)
    CP B
    JR NC,D1E8H
    OR 80H
    LD (HL),A
    LD B,01H
    INC C
    EX AF,AF'
    CALL NC,D2CBH

LD1E8:
    LD A,C
    CALL D39FH
    DEC B
    CP B
    JR NC,D1F1H
    LD (HL),B

LD1F1:
    INC B
    LD (0E49H),BC
    RET
    CALL D370H
    JP D2D6H
    CALL D370H
    CALL D39CH
    JR D1C2H
    CALL D39CH
    CP B
    RET C
    LD E,01H
    JR D210H

LD20E:
    INC HL
    INC E

LD210:
    LD A,(HL)
    RLCA
    INC C
    JR C,D20EH
    INC (HL)
    LD A,(0E6BH)
    CP (HL)
    JR NZ,D233H
    LD A,E
    ADD A,C
    CP 31H
    JR NZ,D226H
    DEC (HL)
    SCF
    JR D234H

LD226:
    PUSH DE
    PUSH BC
    CALL D2CBH
    POP AF
    POP DE
    LD B,A
    LD A,C
    DEC A
    CALL D39FH

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
    JR Z,D240H
    LD B,01H

LD240:
    PUSH BC
    CALL D592H
    POP BC
    LD DE,0E6BH
    LD A,(DE)
    LD H,A
    LD L,C
    CALL D387H
    PUSH HL

LD24F:
    EX AF,AF'
    JR NC,D254H
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
    JR Z,D260H
    LDDR

LD260:
    POP HL
    LD BC,FFC0H
    ADD HL,BC
    LD B,(HL)
    POP AF
    JR NZ,D26BH
    LD B,20H

LD26B:
    LD A,B
    LD (DE),A
    POP HL
    POP BC
    POP DE
    DEC HL
    JR NZ,D234H
    LD (0E49H),BC
    RET
    DEC B
    JR NZ,D283H
    DEC C
    RET Z
    LD A,C
    CALL D39FH
    RET NC
    LD B,A

LD283:
    LD (0E49H),BC
    CALL D39CH
    CP B
    RET C

LD28C:
    EX AF,AF'
    LD A,(HL)
    AND 7FH
    SUB B
    JR C,D2C3H
    PUSH HL
    PUSH AF
    PUSH DE
    PUSH BC
    CALL D542H
    POP BC
    POP DE
    LD H,B
    LD L,C
    CALL D387H
    EX AF,AF'
    LD A,(HL)
    JR NC,D2A6H
    LD (DE),A

LD2A6:
    LD D,H
    LD E,L
    INC HL
    POP AF
    PUSH BC
    JR Z,D2B2H
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
    JR NZ,D28CH
    DEC HL
    DEC (HL)
    RET

LD2C3:
    DEC HL
    DEC (HL)
    RES 7,(HL)
    LD A,C
    JP D31EH

LD2CB:
    SET 7,(HL)
    LD A,C
    CP 19H
    JR Z,D311H
    INC HL
    LD A,(HL)
    OR A
    RET Z

LD2D6:
    PUSH BC
    CALL D509H
    POP BC
    PUSH BC
    LD A,19H
    SUB C
    LD HL,0E66H
    LD DE,0E67H
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
    DJNZ D304H
    POP BC
    LD HL,0E4EH
    LD A,(HL)
    SUB C
    RET C
    INC (HL)
    RET

LD311:
    LD A,01H
    CALL D31EH
    LD BC,0118H
    LD (0E49H),BC
    RET

LD31E:
    PUSH AF
    LD C,A
    LD HL,0E4EH
    SUB (HL)
    CALL Z,D039H
    JR NC,D32AH
    DEC (HL)

LD32A:
    CALL D4E0H
    POP BC
    LD A,B
    CALL D39FH
    PUSH HL
    LD L,B
    LD H,01H
    CALL D387H
    EX DE,HL
    LD HL,0040H
    ADD HL,DE
    LD A,18H
    SUB B
    PUSH AF
    JR Z,D34FH
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
    DJNZ D351H
    POP AF
    LD C,A
    POP HL
    LD D,H
    LD E,L
    INC HL
    JR Z,D360H
    LDIR

LD360:
    XOR A
    LD (DE),A
    RET

LD363:
    LD A,C
    CP 18H
    JR Z,D311H
    INC C
    LD B,01H
    LD (0E49H),BC
    RET

LD370:
    LD BC,(0E49H)

LD374:
    OR A
    DEC C
    LD A,C
    CALL NZ,D39FH
    JR C,D374H
    INC C
    LD B,01H
    LD (0E49H),BC
    RET

LD384:
    LD HL,(0E49H)

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
    LD (0E4BH),HL
    RET

LD39C:
    LD A,(0E49H)

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
    DJNZ D3B1H
    RET
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
    DJNZ D3BDH
    RET
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
    DJNZ D3DAH
    RET

LD420:
    EX AF,AF'
    CALL D459H
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
    JR Z,D439H
    LD BC,0240H

LD439:
    ADD HL,BC
    LD B,0AH
    EXX
    LD A,(0E96H)
    LD B,A
    LD A,(0E95H)
    LD C,A
    POP HL
    JP 0E68H

LD449:
    EX AF,AF'
    CALL CC05H
    LD (0E96H),A
    CALL CC0DH
    XOR B
    LD (0E95H),A
    EX AF,AF'
    RET

LD459:
    DEC B
    LD A,(0B73H)
    OR A
    JR Z,D465H

LD460:
    SLA B
    DEC A
    JR NZ,D460H

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

LD477:
    CALL D459H
    LD DE,0E6DH
    LD A,(0E6CH)
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
    DJNZ D483H
    RET

LD491:
    CALL D459H
    LD DE,0E6DH
    LD A,(0E6CH)
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
    DJNZ D49DH
    RET

LD4AD:
    CALL D459H
    LD A,(0E96H)
    EX AF,AF'
    LD A,3FH
    SUB L
    AND 3FH
    JR Z,D4D3H
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
    DJNZ D4BFH
    RET

LD4D3:
    LD DE,0040H
    LD B,0AH
    LD A,(0E96H)

LD4DB:
    LD (HL),A
    ADD HL,DE
    DJNZ D4DBH
    RET

LD4E0:
    LD A,C
    CP 18H
    JR Z,D4F9H
    LD B,01H
    CALL D459H
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
    LD A,(0E96H)
    LD (HL),A
    LDIR
    RET

LD509:
    LD A,18H
    SUB C
    JR Z,D4F9H
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
    JR D4FFH

LD52A:
    LD A,(0E6CH)
    LD C,A

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
    DJNZ D530H
    RET

LD542:
    JR Z,D562H
    PUSH HL
    PUSH BC
    LD E,A
    LD A,(0B73H)
    OR A
    JR Z,D552H

LD54D:
    SLA E
    DEC A
    JR NZ,D54DH

LD552:
    CALL D459H
    LD C,E
    LD D,H
    LD A,(0E6CH)
    ADD A,L
    LD E,A
    EX DE,HL
    CALL D52EH
    POP BC
    POP HL

LD562:
    LD B,(HL)
    BIT 7,B
    JR Z,D578H
    PUSH BC
    RES 7,B
    CALL D459H
    EX DE,HL
    POP BC
    INC C
    LD B,01H
    CALL D459H
    JP D52AH

LD578:
    CALL D459H
    LD C,0AH
    LD DE,0040H

LD580:
    LD A,(0E6CH)
    LD B,A
    LD A,(0E96H)
    PUSH HL

LD588:
    LD (HL),A
    INC HL
    DJNZ D588H
    POP HL
    ADD HL,DE
    DEC C
    JR NZ,D580H
    RET

LD592:
    PUSH AF
    LD A,(HL)
    AND 7FH
    SUB B
    JR Z,D5CEH
    LD E,A
    LD A,(0B73H)
    OR A
    JR Z,D5A5H

LD5A0:
    SLA E
    DEC A
    JR NZ,D5A0H

LD5A5:
    PUSH HL
    PUSH BC
    LD B,(HL)
    RES 7,B
    CALL D459H
    DEC HL
    LD C,E
    LD B,00H
    EX DE,HL
    LD A,(0E6CH)
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
    DJNZ D5BBH
    POP BC
    POP HL

LD5CE:
    POP AF
    JR Z,D578H
    EX DE,HL
    PUSH BC
    CALL D459H
    POP BC
    EX DE,HL
    DEC C
    DEC HL
    LD B,(HL)
    RES 7,B
    CALL D459H
    JP D52AH

; KEYBOARD_JUMP_TABLE - Counted jump table for keyboard OS functions.
; usage: trace,data
KEYBOARD_JUMP_TABLE:

; KL: KEYBOARD routine jump table; first byte is the routine count, followed by routine addresses.
    INC B
    DEC L
    SUB 18H
    SUB 2CH
    SUB 12H
    DB D6H

; KEYBOARD_INIT - Initializes keyboard state and work variables.
; usage: trace,call
KEYBOARD_INIT:
    LD A,1EH
    LD (0B65H),A
    LD A,03H
    LD (0B67H),A
    XOR A
    LD (0B66H),A
    LD HL,0B51H
    LD DE,0B52H
    LD BC,0013H
    LD (HL),A
    LDIR
    LD HL,0BE5H
    LD DE,0BE6H
    LD C,09H
    LD (HL),A
    LDIR
    RET

; KB_STATUS - Reports whether a key is available.
; usage: trace,call
KB_STATUS:

; KL: KB STATUS routine.
    LD A,(0BE5H)
    LD C,A
    XOR A
    RET

; KB_CHIN - Reads a character from the keyboard device.
; usage: trace,call
KB_CHIN:

; KL: KB CHIN routine.
    LD HL,0BE5H

LD61B:
    LD A,(HL)
    OR A
    JR Z,D61BH
    LD A,(0BE9H)
    LD C,A
    LD (HL),00H
    LD A,(0B16H)
    OR A
    RET Z
    LD A,F5H
    RET

; KB_INT - Keyboard interrupt routine that scans the key matrix.
; usage: trace
KB_INT:
    LD A,(0B66H)
    AND 0BH
    LD B,A
    NEG
    AND B
    LD (0B66H),A
    CALL D7A5H
    LD HL,0BE7H
    BIT 5,(IY+06H)
    JR Z,D652H
    LD A,(HL)
    OR A
    RET NZ
    DEC (HL)
    LD A,01H
    CALL D790H
    LD (0B66H),A
    RET NZ

LD652:
    LD (HL),B
    LD A,04H
    CALL D790H
    LD (0BE8H),A

LD65B:
    XOR A

LD65C:
    CALL D6C7H
    INC E
    JR Z,D672H
    LD A,C
    LD (0BE9H),A
    LD A,FFH
    LD (0BE5H),A
    LD A,(0B65H)
    LD (0BEAH),A
    RET

LD672:
    LD A,(HL)
    OR A
    JR NZ,D65CH
    LD HL,0BEAH
    CALL D6C0H
    INC HL
    CALL NC,D6C0H
    RET NC
    LD A,(0BEEH)
    LD C,A
    LD DE,(0BECH)
    LD HL,0B64H
    SBC HL,DE
    EX DE,HL
    JR Z,D69BH
    OR A
    EX DE,HL
    LD HL,0B64H
    SBC HL,DE
    EX DE,HL
    JR NZ,D6A4H

LD69B:
    LD D,66H
    AND D
    JR Z,D6A4H
    XOR D
    AND (HL)
    JR NZ,D6B5H

LD6A4:
    LD A,C
    AND (HL)
    JR NZ,D6B5H
    LD HL,0B5BH
    LD B,0AH

LD6AD:
    LD (HL),A
    INC HL
    DJNZ D6ADH
    LD (0BEBH),A
    RET

LD6B5:
    CPL
    AND (HL)
    LD (HL),A
    LD A,(0B67H)
    LD (0BEBH),A
    JR D65BH

LD6C0:
    LD A,(HL)
    OR A
    RET Z
    DEC (HL)
    RET NZ
    SCF
    RET

LD6C7:
; KEY_MATRIX_DECODE - Decodes the scanned keyboard matrix into a key code.
; usage: trace
KEY_MATRIX_DECODE:
    CALL NZ,D7A5H
    RES 3,(IY+06H)
    RES 4,(IY+07H)
    RES 0,(IY+07H)
    RES 5,(IY+06H)
    LD HL,0BE6H
    LD DE,000AH
    EXX
    LD HL,0B65H
    LD DE,0B5BH

LD6E7:
    DEC DE
    DEC HL
    EXX
    DEC E
    RET M
    EXX
    LD A,(DE)
    LD B,A
    XOR (HL)
    CPL
    AND (HL)
    LD (HL),A
    XOR B
    JR Z,D6E7H
    LD B,A
    NEG
    AND B
    LD B,A
    XOR (HL)
    LD (HL),A
    LD A,B
    LD (0BECH),HL
    LD (0BEEH),A
    LD A,50H

LD706:
    SUB 0AH
    SRL B
    JR NC,D706H
    EXX
    ADD A,E
    LD E,A
    LD A,(0BE8H)
    LD B,A
    LD A,(0B66H)
    XOR B
    LD B,A
    PUSH HL
    CALL D747H
    POP HL
    LD A,(0B68H)
    INC A
    RET Z
    LD A,(HL)
    OR A
    LD A,C
    JR Z,D73CH
    LD A,(0B11H)
    AND F0H
    OR 07H
    OUT (03H),A
    IN A,(58H)
    AND 18H
    JR Z,D740H
    XOR A
    LD (0B16H),A
    JR D740H

LD73C:
    CP 10H
    JR Z,D743H

LD740:
    LD (HL),00H
    RET

LD743:
    DEC (HL)
    LD E,FFH
    RET

LD747:
    LD HL,D784H
    PUSH HL
    LD HL,D85FH
    BIT 2,B
    RET NZ
    BIT 0,B
    JR Z,D774H
    LD HL,D7BFH
    ADD HL,DE
    LD A,(HL)
    CP 61H
    JR C,D774H
    CP 7BH
    JR C,D76AH
    CP 90H
    JR C,D774H
    CP 99H
    JR NC,D774H

LD76A:
    LD HL,D80FH
    BIT 1,B
    RET Z
    LD HL,D7BFH
    RET

LD774:
    LD HL,D80FH
    BIT 1,B
    RET NZ
    LD HL,D8AFH
    BIT 3,B
    RET NZ
    POP HL
    LD HL,D7BFH
    ADD HL,DE
    LD C,(HL)
    LD A,C
    CP FFH
    RET NZ
    LD (0B16H),A
    LD C,1BH
    RET

LD790:
    BIT 4,(IY+07H)
    RET NZ
    LD A,02H
    BIT 3,(IY+06H)
    RET NZ
    BIT 0,(IY+07H)
    LD A,08H
    RET NZ
    XOR A
    RET

LD7A5:
; KEY_MATRIX_READ - Performs the physical keyboard-matrix scan.
; usage: trace
KEY_MATRIX_READ:
    LD A,(0B11H)
    AND F0H
    LD C,A
    LD HL,0B51H
    PUSH HL
    POP IY
    LD B,0AH

LD7B3:
    LD A,C
    OUT (03H),A
    IN A,(58H)
    CPL
    LD (HL),A
    INC C
    INC HL
    DJNZ D7B3H
    RET

; KL: Keyboard matrix decode tables for normal, SHIFT, CTRL, and ALT modes.
    DB 34H, 37H, 72H, 75H, 66H, 6AH, 76H, 6DH, 2AH, 2AH, 31H, 94H, 71H, 70H, 61H, 91H
    DB 79H, 2DH, 13H, F3H, 92H, 93H, 40H, 96H, 3CH, 98H, 2AH, 20H, 04H, E4H, 36H, 2AH
    DB 7AH, 5BH, 68H, 0DH, 6EH, 2AH, 01H, E1H, 30H, 97H, 3BH, 95H, 5CH, 90H, 2AH, 1BH
    DB 06H, E6H, 32H, 39H, 77H, 6FH, 73H, 6CH, 78H, 2EH, 18H, F8H, 33H, 38H, 65H, 69H
    DB 64H, 6BH, 63H, 2CH, 05H, E5H, 35H, 5EH, 74H, 5DH, 67H, 08H, 62H, 2AH, 16H, 43H
    DB 21H, 3DH, 52H, 55H, 46H, 4AH, 56H, 4DH, 2AH, 2AH, 27H, 84H, 51H, 50H, 41H, 81H
    DB 59H, 5FH, 13H, F3H, 82H, 83H, 60H, 86H, 3EH, 88H, 2AH, 20H, 04H, E4H, 2FH, 23H
    DB 5AH, 7BH, 48H, 0DH, 4EH, 2AH, 01H, E1H, 26H, 87H, 24H, 85H, 7CH, 80H, 2AH, 1BH
    DB 06H, E6H, 22H, 29H, 57H, 4FH, 53H, 4CH, 58H, 3AH, 18H, F8H, 2BH, 28H, 45H, 49H
    DB 44H, 4BH, 43H, 3FH, 05H, E5H, 25H, 7EH, 54H, 7DH, 47H, 07H, 42H, 2AH, 16H, 49H
    DB 8BH, 9CH, 12H, 15H, 06H, 0AH, 16H, 0DH, 2AH, 2AH, 99H, DCH, 11H, 10H, 01H, D9H
    DB 19H, 1FH, 13H, F3H, DAH, DBH, 00H, DEH, 3CH, 98H, 2AH, 20H, 04H, E4H, 8CH, 8EH
    DB 1AH, 1BH, 08H, 0DH, 0EH, 2AH, 01H, E1H, 89H, DFH, 3BH, DDH, 1CH, CFH, 2AH, FFH
    DB 06H, E6H, 8AH, 9DH, 17H, 0FH, 13H, 0CH, 18H, 2EH, 18H, F8H, 9AH, 8DH, 05H, 09H
    DB 04H, 0BH, 03H, 2CH, 05H, E5H, 9BH, 1EH, 14H, 1DH, 07H, 08H, 02H, 2AH, 16H, 53H
    DB A4H, A7H, C2H, C5H, B6H, BAH, C6H, BDH, 2AH, 2AH, A1H, D4H, C1H, C0H, B1H, D1H
    DB C9H, ADH, 13H, F3H, D2H, D3H, B0H, D6H, ACH, D8H, 2AH, 20H, 04H, E4H, A6H, AAH
    DB CAH, CBH, B8H, 0DH, BEH, 2AH, 01H, E1H, A0H, D7H, ABH, D5H, CCH, D0H, 2AH, 1BH
    DB 06H, E6H, A2H, A9H, C7H, BFH, C3H, BCH, C8H, AEH, 18H, F8H, A3H, A8H, B5H, B9H
    DB B4H, BBH, B3H, AFH, 05H, E5H, A5H, CEH, C4H, CDH, B7H, 08H, B2H, 2AH, 16H, 4CH

; KL: PRINTER routine jump table.
    DB 03H, 29H, D9H, 0CH, D9H, 06H, D9H

; PAR_BKOUT - Writes a block to the parallel printer.
; usage: trace,call
PAR_BKOUT:
    LD HL,D90CH
    JP C56DH

LD90C:
; PAR_CHOUT - Writes one character to the parallel printer.
; usage: trace,call
PAR_CHOUT:
    LD A,(0B16H)
    INC A
    LD A,F5H
    RET Z
    IN A,(59H)
    RLCA
    JR NC,D90CH
    DI
    LD A,C
    OUT (01H),A
    LD A,(0B13H)
    AND 7FH
    OUT (06H),A
    OR 80H
    OUT (06H),A
    EI
    XOR A
    RET

; SOUND_JUMP_TABLE - Counted jump table for sound OS functions.
; usage: trace,data
SOUND_JUMP_TABLE:

; KL: SOUND routine jump table.
    DB 04H, 33H, D9H, 60H, D9H, 60H, D9H, 61H, D9H

; TONE_INT - Sound interrupt routine that advances timed sound generation.
; usage: trace
TONE_INT:
    DB 3AH, 14H, 0BH, 3CH, C0H, 3AH, 16H, 0BH, 3CH, 28H, 08H, 3AH, EFH, 0BH, 3DH, 32H
    DB EFH, 0BH, C0H, 32H, 14H, 0BH, 32H, EFH, 0BH, 3AH, 10H, 0BH, F6H, 08H, 32H, 10H
    DB 0BH, 3AH, 12H, 0BH, E6H, CFH, 32H, 12H, 0BH, D3H, 05H, 3EH, F5H, C9H

; TONE_SET - Programs a tone using the OS sound parameters.
; usage: trace,call
TONE_SET:
    DB 3AH, 15H, 0BH, 3CH, 28H, 07H, 3AH, EFH, 0BH, D6H, 02H, 30H, F3H, AFH, 32H, 14H
    DB 0BH, 3AH, 16H, 0BH, 3CH, 28H, CEH, 78H, B7H, 20H, 05H, CDH, 46H, D9H, AFH, C9H
    DB 32H, EFH, 0BH, 79H, E6H, 0FH, 07H, 07H, 4FH, 3AH, 13H, 0BH, E6H, C3H, B1H, 32H
    DB 13H, 0BH, D3H, 06H, 7AH, E6H, 0FH, F6H, 10H, 57H, 3AH, 12H, 0BH, E6H, C0H, B2H
    DB 32H, 12H, 0BH, D3H, 05H, 7BH, D3H, 04H, 3EH, FFH, 32H, 14H, 0BH, 32H, 71H, 0BH
    DB 3AH, 10H, 0BH, E6H, F7H, 32H, 10H, 0BH, AFH, C9H

; CASSETTE_JUMP_TABLE - Counted jump table for cassette OS functions.
; usage: trace,data
CASSETTE_JUMP_TABLE:

; KL: CASSETTE routine jump table.
    DB 06H, E7H, D9H, D2H, D9H, D7H, D9H, C8H, D9H, CDH, D9H, DCH, D9H

; CAS_OPEN_CREATE - Cassette open/create device entry.
; usage: trace,call
CAS_OPEN_CREATE:
    LD HL,F3E2H
    JR D9DFH

; CAS_CLOSE - Cassette close device entry.
; usage: trace,call
CAS_CLOSE:
    LD HL,F3E7H
    JR D9DFH

; CAS_CHIN_OUT - Cassette character input/output device entry.
; usage: trace,call
CAS_CHIN_OUT:
    LD HL,F3D8H
    JR D9DFH

; CAS_BKIN_OUT - Cassette block input/output device entry.
; usage: trace,call
CAS_BKIN_OUT:
    LD HL,F3DDH
    JR D9DFH

; CAS_VERIFY - Cassette verify device entry.
; usage: trace,call
CAS_VERIFY:

; KL: CAS VERIFY routine.
    LD HL,F3ECH

LD9DF:
    JP FFF0H

; CAS_INIT - Initializes cassette device work variables.
; usage: trace
CAS_INIT:
    LD HL,F3F1H
    JR D9DFH
    RET
    JR Z,DA4DH
    ADD HL,HL
    JR NZ,DA36H
    LD D,E
    LD C,H

LD9EF:
; BASIC_INIT - Performs full BASIC initialization.
; usage: trace
BASIC_INIT:
    LD HL,1700H
    PUSH HL
    POP IX
    LD A,(0B21H)
    OR A
    JP NZ,DAD3H
    LD BC,02EFH

LD9FF:
    LD (HL),A
    CPI
    JP PE,D9FFH
    LD (1720H),HL
    LD (1722H),HL
    LD HL,FB5BH
    LD DE,0008H
    LD BC,0027H
    LDIR
    CALL DE10H

; STARTUP_SCREEN - Displays the TV Computer BASIC startup screen.
; usage: trace
STARTUP_SCREEN:
    LD DE,DC15H
    RST 30H
    INC C
    LD A,02H
    LD (0B4FH),A
    LD (IX+05H),00H
    LD BC,0303H
    LD A,0CH

LDA2C:
    PUSH BC
    PUSH AF
    RST 30H
    INC BC
    CALL DBF2H
    POP AF
    PUSH AF
    DB CDH

LDA36:
    INC C
    CALL C,F2CDH
    IN A,(CDH)
    SUB E
    CP F1H
    POP BC
    INC B
    INC C
    INC C
    SUB 02H
    JR NC,DA2CH
    LD BC,0D15H
    RST 30H
    INC BC
    DB CDH

LDA4D:
    DB F2H, DBH

LDA4F:
    LD BC,0B12H
    RST 30H
    INC BC
    LD HL,DBFFH
    LD B,(HL)

LDA58:
    INC HL
    LD A,(HL)
    CALL FE9AH
    PUSH HL
    PUSH BC

LDA5F:
    CALL E6D8H
    AND 03H
    JR Z,DA5FH
    POP BC
    POP HL
    LD (0B4DH),A

LDA6B:
    DEC C
    JR NZ,DA6BH
    DJNZ DA58H
    RST 30H
    SUB E
    INC C
    JR NZ,DA4FH
    RST 30H
    SUB C
    LD C,01H
    RST 30H
    INC B
    LD (0B4FH),A
    CALL FC18H
    CALL FE79H

; KL: TVC BASIC 1.2 copyright sign-on text.
    DB 32H, 54H, 56H, 20H, 43H, 4FH, 4DH, 50H, 55H, 54H, 45H, 52H, 20H, 42H, 41H, 53H
    DB 49H, 43H, 20H, 31H, 2EH, 32H, 0DH, 0AH, 43H, 6FH, 70H, 79H, 72H, 69H, 67H, 68H
    DB 74H, 20H, 31H, 39H, 38H, 35H, 20H, 56H, 49H, 44H, 45H, 4FH, 54H, 4FH, 4EH, 0DH
    DB 0AH, 0DH, 0AH, CDH, 4CH, ECH, CDH, C1H, FEH, CDH, 1BH, FAH, CDH, 79H, FEH, 0FH
    DB 20H, 62H, 79H, 74H, 65H, 73H, 20H, 66H, 72H, 65H, 65H, 0DH, 0AH, 0DH, 0AH, AFH
    DB 32H, 21H, 0BH, CDH, FCH, DCH, 31H, ACH, 16H, 21H, 03H, 17H, 7EH, B7H, 36H, 00H
    DB C4H, 10H, DEH, DDH, 36H, 05H, 20H, DDH, CBH, 00H, 96H, DDH, CBH, 00H, 4EH, 20H
    DB 0AH, CDH, 18H, FCH, CDH, 79H, FEH, 03H, 6FH, 6BH, 0BH, DDH, CBH, 00H, 8EH, CDH
    DB 93H, FEH

LDB06:
; BASIC_COMMAND_LOOP - Displays OK and waits for a BASIC command or program line.
; usage: trace
BASIC_COMMAND_LOOP:
    CALL FF4FH
    CP F5H
    JP Z,FFA3H
    CP ECH
    JR Z,DAF5H
    CALL DC19H
    LD E,L
    LD D,H
    INC HL
    CALL F914H
    JR C,DB2AH
    LD A,(HL)
    CP A8H
    JR Z,DB25H
    INC A
    JR NZ,DB61H

LDB25:
    CALL FA1BH
    JR DB06H

LDB2A:
    RLA
    JR C,DB61H
    LD B,(HL)
    LD A,(IY+08H)
    CP 40H
    JR C,DB61H
    CP 44H
    JR NC,DB61H
    LD A,(IY+06H)
    OR A
    JR Z,DB61H

LDB3F:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,DB3FH
    DEC HL
    EX DE,HL
    LD A,(HL)
    INC HL
    ADD A,02H
    SBC HL,DE
    ADD A,L
    LD C,A
    CALL FAC3H
    EX DE,HL
    DEC HL
    LD (HL),D
    DEC HL
    LD (HL),E
    DEC HL
    LD (HL),C
    LD (170CH),HL
    CALL DCAFH
    JR DB06H

LDB61:
    EX DE,HL
    CALL FA1BH
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
    LD (170CH),HL
    INC HL
    BIT 0,(IX+00H)
    CALL NZ,DE4DH
    RES 0,(IX+02H)
    INC HL
    INC HL
    EXX

LDB80:
; BASIC_NEXT_STATEMENT - Interprets the next statement in the current BASIC line.
; usage: trace
BASIC_NEXT_STATEMENT:
    EXX

LDB81:
    CALL FF9DH
    LD (171AH),IY

LDB88:
    LD A,(HL)
    CP FEH
    JR NC,DBBBH
    INC HL
    CP 20H
    JR Z,DB88H
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
    CALL C,FC43H
    LD SP,16ACH
    JP (HL)

LDBAE:
    CALL FC43H

LDBB1:
    EXX
    LD A,B
    EXX
    CP FDH
    JR Z,DB80H
    JP C,FD5AH

LDBBB:

; KL: REM routine.
    LD HL,(170CH)
    LD C,(HL)
    XOR A
    LD B,A
    ADD HL,BC

LDBC2:
    OR (HL)
    JR NZ,DB6EH

LDBC5:
    LD A,(IY+00H)
    CP 2BH
    JR Z,DBDAH
    CP 06H
    JR Z,DBDAH

LDBD0:
    BIT 2,(IX+00H)
    JP Z,DADAH
    JP E10EH

LDBDA:
    LD E,(IY+03H)
    LD D,(IY+04H)
    LD HL,1831H
    SBC HL,DE
    JR C,DBD0H
    LD C,A
    LD B,00H
    ADD IY,BC
    LD (171AH),IY
    JR DBC5H

LDBF2:
    CALL FE79H
    EX AF,AF'

; KL: "VIDEOTON" sign-on text.
    DB 56H, 49H, 44H, 45H, 4FH, 54H, 4FH, 4EH, C9H, 0CH, 54H, 56H, 20H, 20H, 43H, 4FH
    DB 4DH, 50H, 55H, 54H, 45H, 52H, 3DH, F8H, F5H, CDH, C7H, FEH, F1H, 18H, F7H, 01H
    DB 44H, 54H, 51H

; TOKENIZE_BASIC_LINE - Converts an entered BASIC line to tokenized form.
; usage: trace
TOKENIZE_BASIC_LINE:
    DB FDH, E5H, EBH, FDH, 21H, 35H, 17H, 01H
    NOP
    NOP

LDC23:
    INC DE

LDC24:
    LD HL,DE6EH
    LD A,FEH
    EX AF,AF'

LDC2A:
    INC IY
    LD (171CH),DE
    CALL FBBFH
    LD (IY+00H),A
    INC A
    JR Z,DC9FH
    LD A,B
    OR A
    JR Z,DC41H
    CP 3AH
    JR NZ,DC4BH

LDC41:
    LD A,(DE)
    CP 22H
    JR NZ,DC4BH
    LD C,B
    LD B,A
    INC DE
    JR DC2AH

LDC4B:
    CALL FBBFH
    INC DE
    OR A
    JR Z,DC60H
    CP B
    JR NZ,DC60H
    LD B,C
    LD C,00H
    CP 22H
    JR Z,DC24H
    LD A,FDH
    JR DC88H

LDC60:
    INC B
    DEC B
    JR NZ,DC2AH
    XOR (HL)
    ADD A,A
    JR NZ,DC8DH
    INC HL
    JR NC,DC4BH
    EX AF,AF'
    LD C,B
    CP A6H
    JR Z,DC79H
    CP A5H
    JR Z,DC79H
    CP A3H
    JR NZ,DC7BH

LDC79:
    SUB 08H

LDC7B:
    CP FDH
    JR Z,DC88H
    CP FBH
    JR C,DC88H
    LD B,A
    JR NZ,DC88H
    LD B,3AH

LDC88:
    LD (IY+00H),A
    JR DC24H

LDC8D:
    LD DE,(171CH)

LDC91:
    BIT 7,(HL)
    INC HL
    JR Z,DC91H
    EX AF,AF'
    DEC A
    EX AF,AF'
    LD A,(HL)
    INC A
    JR Z,DC23H
    JR DC4BH

LDC9F:
    PUSH IY
    POP HL
    INC HL
    LD (HL),A
    LD DE,1735H
    OR A
    SBC HL,DE
    EX DE,HL
    LD (HL),E
    POP IY
    RET

LDCAF:
    LD (171CH),HL
    PUSH BC
    CALL DD45H
    CALL NC,DCE6H
    POP BC
    INC B
    RET Z
    LD B,00H
    CALL DCC9H
    EX DE,HL
    LD HL,(171CH)
    LDIR
    JR DCFCH

LDCC9:
    CALL DCFCH
    CALL FC8EH
    PUSH BC
    PUSH HL
    CALL DD41H
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

LDCE6:
    LD C,(HL)
    LD B,00H
    PUSH HL
    ADD HL,BC
    PUSH HL
    CALL DD41H
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

LDCFC:
    PUSH HL
    PUSH DE
    PUSH BC
    LD IY,(0B19H)
    LD HL,0000H
    LD (IY+00H),L
    LD (170EH),HL
    LD (1714H),HL
    LD HL,F094H
    LD (1724H),HL
    LD HL,1709H
    LD (HL),L
    LD (170AH),HL
    LD HL,(1722H)
    LD (1712H),HL
    CALL DD41H
    INC HL
    LD (1726H),HL
    POP BC
    POP DE
    POP HL
    RET

LDD2D:
    PUSH AF
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    PUSH HL
    EX DE,HL
    CALL FF19H
    POP HL
    CALL FEDDH
    POP AF
    RET NC
    JP FE8EH

LDD41:
    LD HL,FFFFH

LDD44:
    EX DE,HL

LDD45:
; FIND_BASIC_LINE - Finds a BASIC program line by line number.
; usage: call
FIND_BASIC_LINE:
    LD HL,(1722H)
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
    JR NC,DD4BH
    RET

LDD63:
    RST 08H
    LD A,(BC)

; BASIC_CONTINUE - Implements the BASIC CONTINUE statement.
; usage: trace
BASIC_CONTINUE:

; KL: CONTINUE routine.
    BIT 2,(IX+00H)
    JP NZ,DBB1H
    SET 2,(IX+00H)
    LD HL,(170EH)
    LD A,H
    OR L
    JR Z,DD63H
    LD (170CH),HL
    LD HL,(1710H)
    JP DB81H

; KL: LLIST routine.
    LD C,40H
    OR C
    JR DD88H

; BASIC_LIST - Implements the BASIC LIST statement.
; usage: trace
BASIC_LIST:

; KL: LIST routine.
    LD C,20H
    XOR A

LDD88:
    PUSH AF
    CALL FBEEH
    POP AF
    JR NZ,DD94H
    LD A,C
    CP 20H
    LD D,F6H

LDD94:
    SCF
    EX AF,AF'
    EXX
    LD A,B
    EXX
    LD HL,0000H
    LD DE,FFFFH
    CP FDH
    JR NC,DDD1H
    JR DDB6H

LDDA5:
    EXX
    LD A,B
    EXX
    CP A4H
    JR Z,DDB3H
    EX AF,AF'
    JP C,DBB1H
    JP DADAH

LDDB3:
    CALL FC43H

LDDB6:
    CALL DDFBH
    INC D
    JR NZ,DDC4H
    CP A2H
    JP NZ,FD5AH
    LD DE,0100H

LDDC4:
    DEC D
    PUSH DE
    CP A2H
    JR NZ,DDD0H
    CALL FC43H
    CALL DDFBH

LDDD0:
    POP HL

LDDD1:
    PUSH DE
    CALL DD44H
    POP DE

LDDD6:
    LD A,(HL)
    OR A
    JR Z,DDA5H
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
    JR C,DDA5H
    PUSH DE
    EX AF,AF'
    JR NC,DDF0H
    CALL DD2DH
    JR DDF4H

LDDF0:
    CALL DCE6H
    OR A

LDDF4:
    EX AF,AF'
    CALL FF9DH
    POP DE
    JR DDD6H

LDDFB:
    LD DE,FFFFH
    CP 02H
    RET NZ
    CALL FAC3H
    EX DE,HL
    JP FC43H

; BASIC_NEW - Implements the BASIC NEW statement.
; usage: trace
BASIC_NEW:

; KL: NEW routine.
    LD HL,DADAH
    PUSH HL
    RES 0,(IX+00H)

LDE10:
    LD HL,(1720H)
    LD (1722H),HL
    LD (HL),00H
    JP DCFCH

; BASIC_RUN - Implements the BASIC RUN statement.
; usage: trace
BASIC_RUN:

; KL: RUN routine.
    LD HL,(1722H)
    CP 02H
    CALL Z,FBDEH
    CALL DCFCH
    CALL FC3EH
    SET 2,(IX+00H)
    XOR A
    JP DBC2H

; BASIC_TRACE - Implements BASIC TRACE ON and TRACE OFF.
; usage: trace
BASIC_TRACE:
    CALL FBECH
    LD (IX+06H),C
    CP E3H
    JR Z,DE46H
    CP C1H
    JP NZ,FD5AH

; KL: TRACE OFF routine.
    RES 0,(IX+00H)
    JR DE4AH

LDE46:

; KL: TRACE ON routine.
    SET 0,(IX+00H)

LDE4A:
    JP DBAEH

LDE4D:
    BIT 2,(IX+00H)
    RET Z
    LD A,(1706H)
    LD (1705H),A
    PUSH HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    EX DE,HL
    LD A,3CH
    CALL FE9AH
    LD B,00H
    CALL FF1BH
    POP HL
    LD A,3EH
    JP FE9AH

; BASIC_KEYWORD_TABLE - BASIC keywords ordered by descending token value.
; usage: data
BASIC_KEYWORD_TABLE:

; KL: BASIC keyword table ordered by descending token value; the high bit marks the last byte of each keyword.
    RST 38H
    AND C
    CP D
    LD D,D
    LD B,L
    CALL 4144H
    LD D,H
    POP BC
    LD B,E
    LD C,H
    LD C,A
    LD D,E
    PUSH BC
    LD B,E
    LD C,H
    OUT (43H),A
    LD C,A
    LD C,(HL)
    LD D,H
    LD C,C
    LD C,(HL)
    LD D,L
    PUSH BC
    LD B,H
    LD B,L
    ADD A,44H
    LD B,L
    LD C,H
    LD B,L
    LD D,H
    PUSH BC
    LD B,H
    LD C,C
    CALL 4C45H
    LD D,E
    PUSH BC
    LD B,L
    LD C,(HL)
    CALL NZ,4F46H
    JP NC,4547H
    CALL NC,4F47H
    LD D,E
    LD D,L
    JP NZ,4F47H
    LD D,H
    RST 08H
    LD B,A
    LD D,D
    LD B,C
    LD D,B
    LD C,B
    LD C,C
    LD B,E
    OUT (49H),A
    ADD A,49H
    LD C,(HL)
    LD D,B
    LD D,L
    CALL NC,454CH
    CALL NC,494CH
    LD D,E
    CALL NC,4C4CH
    LD C,C
    LD D,E
    CALL NC,4F4CH
    LD B,C
    CALL NZ,4F4CH
    LD C,L
    LD B,L
    CALL 454EH
    RST 10H
    LD C,(HL)
    LD B,L
    LD E,B
    CALL NC,CB4FH
    LD C,A
    ADC A,4FH
    LD D,B
    LD B,L
    ADC A,4FH
    LD D,L
    LD D,H
    LD D,B
    LD D,L
    CALL NC,554FH
    CALL NC,4C50H
    LD C,A
    CALL NC,4F50H
    LD C,E
    PUSH BC
    LD D,B
    LD D,D
    LD C,C
    LD C,(HL)
    CALL NC,4152H
    LD C,(HL)
    LD B,H
    LD C,A
    LD C,L
    LD C,C
    LD E,D
    PUSH BC
    LD D,D
    LD B,L
    LD B,C
    CALL NZ,4552H
    LD D,E
    LD D,H
    LD C,A
    LD D,D
    PUSH BC
    LD D,D
    LD B,L
    LD D,H
    LD D,L
    LD D,D
    ADC A,52H
    LD D,L
    ADC A,53H
    LD B,C
    LD D,(HL)
    PUSH BC
    LD D,E
    LD B,L
    CALL NC,4F53H
    LD D,L
    LD C,(HL)
    CALL NZ,5453H
    LD C,A
    RET NC
    LD D,H
    LD D,D
    LD B,C
    LD B,E
    PUSH BC
    LD D,(HL)
    LD B,L
    LD D,D
    LD C,C
    LD B,(HL)
    EXX
    LD B,L
    LD E,B
    CALL NC,504CH
    LD D,D
    LD C,C
    LD C,(HL)
    CALL NC,A1A1H
    AND C
    AND C
    AND C
    AND C
    LD B,C
    LD C,(HL)
    CALL NZ,4843H
    LD B,C
    LD D,D
    LD B,C
    LD B,E
    LD D,H
    LD B,L
    JP NC,4544H
    LD C,H
    LD B,C
    EXX
    LD B,H
    LD D,L
    LD D,D
    LD B,C
    LD D,H
    LD C,C
    LD C,A
    ADC A,49H
    LD C,(HL)
    LD C,E
    LD B,L
    LD E,C
    AND H
    LD C,C
    LD C,(HL)
    BIT 1,L
    LD C,A
    LD B,H
    PUSH BC
    LD C,(HL)
    LD C,A
    CALL NC,464FH
    ADD A,4FH
    LD D,D
    CALL NZ,D24FH
    LD D,B
    LD B,C
    LD C,C
    LD C,(HL)
    CALL NC,4150H
    LD C,H
    LD B,L
    LD D,H
    LD D,H
    PUSH BC
    LD D,B
    LD B,C
    LD D,B
    LD B,L
    JP NC,4950H
    LD D,H
    LD B,E
    RET Z
    LD D,B
    LD D,D
    LD C,A
    LD C,L
    LD D,B
    CALL NC,4152H
    LD D,H
    PUSH BC
    LD D,E
    LD D,H
    LD B,L
    RET NC
    LD D,E
    LD D,H
    LD E,C
    LD C,H
    PUSH BC
    LD D,H
    LD B,C
    JP NZ,4854H
    LD B,L
    ADC A,54H
    RST 08H
    LD D,(HL)
    LD C,A
    LD C,H
    LD D,L
    LD C,L
    PUSH BC
    LD E,B
    LD C,A
    JP NC,5441H
    ADC A,41H
    CALL NC,5355H
    LD C,C
    LD C,(HL)
    RST 00H
    LD B,D
    LD C,A
    LD D,D
    LD B,H
    LD B,L
    JP NC,A1A1H
    AND C
    AND C
    AND C
    XOR D
    AND E
    DEC A
    CP (HL)
    LD A,BCH
    XOR H
    DEC A
    CP H
    XOR L
    XOR A
    CP E
    SBC A,3EH
    CP L
    INC A
    CP (HL)
    CP (HL)
    INC A
    CP L
    CP L
    CP H
    XOR E
    AND (HL)
    XOR B
    XOR C
    RST 38H
    JR NZ,E044H
    LD L,C
    LD (HL),E
    LD (HL),E
    LD L,C
    LD L,(HL)
    RST 20H
    LD (HL),D
    LD H,A
    LD (HL),L
    LD L,L
    LD H,L
    LD L,(HL)
    CALL P,6142H
    LD H,H
    AND B
    LD C,(HL)
    LD L,A
    AND B
    LD B,E
    LD H,C
    LD L,(HL)
    LD L,(HL)
    LD L,A
    LD (HL),H
    AND B

; KL: DATA routine.
    EXX

LDFF3:
    LD A,(HL)
    CP FDH
    JP NC,DB81H
    INC HL
    JR DFF3H

; BASIC_CLS - Implements the BASIC CLS statement.
; usage: trace
BASIC_CLS:

; KL: CLS BASIC routine.
    RST 30H
    DEC B
    RST 10H
    DB C3H
