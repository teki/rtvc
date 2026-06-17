; -----------------------------------------------------------------------------
; TVC BASIC 1.2 SYS upper ROM overlay
; Source: roms/TVC12_D3.64K
; ORG: E000H
; Size: 8192 bytes
; Symbols: data/rom_symbols_1_2.json
; Comments: data/rom_comments_1_2.json
; Data ranges: E6BDH-E6C6H, E7E4H-E811H, E87BH-E895H, FB5BH-FD73H, FD74H-FF46H, FF47H-FF4EH, FFB1H-FFE9H, FFEDH-FFFFH
; Auto labels: branch and call targets are emitted as Lxxxx.
; -----------------------------------------------------------------------------

ORG E000H

    OR C
    IN A,(E6H)
    ADD A,C
    CP 01H
    JP NZ,FD5AH
    BIT 2,(IX+00H)
    JR Z,E02EH
    EXX
    PUSH HL
    PUSH DE
    EXX
    POP HL
    DEC HL
    BIT 7,(HL)
    JR Z,E04EH
    RES 7,(HL)
    SET 2,(HL)
    INC HL
    LD DE,(170CH)
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    POP DE
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    LD (1726H),HL

LE02E:
    EXX
    LD C,01H

LE031:
    DEC C
    DB 3EH

LE033:
    INC C

LE034:
    LD A,(HL)
    CP FEH
    JP NC,E04BH
    INC HL
    CP 96H
    JR Z,E033H
    CP 95H
    JR Z,E031H
    CP FDH
    JR NZ,E034H
    INC C
    DEC C
    JR NZ,E034H

LE04B:
    JP DB81H

LE04E:
    RST 08H
    RRCA

LE050:
    CALL FC43H

; KL: DIM routine.
    OR A
    JP M,FD5AH
    BIT 0,A
    JP Z,FD5AH
    EXX
    LD A,C
    AND 88H
    JR Z,E04EH
    AND 08H
    CALL NZ,F40BH
    DEC DE
    LD A,(DE)
    XOR 81H
    LD (1701H),A
    LD (DE),A
    INC DE
    LD A,FFH
    LD (DE),A
    PUSH DE
    INC DE
    LD (1726H),DE
    EXX
    CALL FC43H
    BIT 1,(IX+01H)
    JR NZ,E0A3H
    CP A8H
    JR NZ,E0A3H
    CALL FB16H
    INC A
    JR Z,E0E4H
    DEC A
    LD C,A
    POP HL
    LD (1726H),HL
    DEC HL
    RES 0,(HL)
    CALL F3C1H

LE099:
    EXX
    LD A,B
    EXX
    CP A4H
    JR Z,E050H
    JP DBB4H

LE0A3:
    LD A,96H
    CALL FD54H
    SCF

LE0A9:
    CALL NC,FC43H
    CALL FAC4H
    JP M,FB14H
    INC HL
    EX DE,HL
    LD HL,(1726H)
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    LD (1726H),HL
    EX DE,HL
    CALL FA2BH
    POP HL
    INC (HL)
    PUSH HL
    CALL NZ,F512H
    EXX
    LD A,B
    EXX
    CP A4H
    JR Z,E0A9H
    LD A,95H
    CALL FD54H
    POP HL
    INC (HL)
    BIT 1,(IX+01H)
    JR NZ,E0E9H
    CP A8H
    LD A,12H
    CALL Z,FB16H
    INC A

LE0E4:
    JP Z,FB14H
    DEC A
    LD C,A

LE0E9:
    PUSH BC
    RST 18H
    INC C
    ADD A,L
    LD H,CDH
    SUB E
    OR F2H
    OR C
    CALL M,C3CDH
    DB FAH, C1H

LE0F8:
    PUSH HL
    CALL F3BBH
    POP HL
    DEC HL
    LD A,L
    OR H
    JR NZ,E0F8H
    JR E099H

; KL: ELSE routine.
    BIT 0,(IX+02H)
    JP Z,FD5AH
    JP DBBBH

; KL: END routine.
    LD HL,0000H
    LD (170EH),HL
    JP DADAH

; KL: EXT routine.
    CALL FB1BH
    CP 07H
    JP NC,FB14H
    ADD A,A
    PUSH AF
    EXX
    LD A,B
    EXX
    CP A4H
    JR NZ,E13EH
    CALL FABAH
    EX DE,HL
    CP A4H
    JR NZ,E13EH
    CALL FABAH
    CP A4H
    JR NZ,E13EH
    PUSH HL
    CALL FABAH
    LD C,L
    LD B,H
    POP HL

LE13E:
    EX (SP),HL
    PUSH DE
    LD E,H
    LD D,00H
    LD HL,0021H
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    LD A,D
    OR E
    JP Z,DBB1H
    LD (0016H),DE
    POP DE
    LD HL,DBB1H
    EX (SP),HL
    EX DE,HL
    JP 0015H

; BASIC_FOR - Implements the BASIC FOR statement.
; usage: trace
BASIC_FOR:

; KL: FOR routine.
    CP 03H
    JR Z,E167H
    AND 82H
    JP NZ,FD5AH
    RST 08H
    DB 0EH

LE167:
    CALL F42EH
    LD A,9AH
    CALL FD54H
    CALL FC8EH
    PUSH HL
    CALL F0A7H
    POP HL
    PUSH HL
    CALL FB28H
    LD A,B4H
    CALL FD54H
    CALL F0A7H
    LD DE,FFEEH
    ADD IY,DE
    CP B8H
    JR Z,E195H
    EX AF,AF'
    LD HL,0001H
    CALL FA2BH
    EX AF,AF'
    SCF

LE195:
    CALL NC,F0A4H
    CP FDH
    JP C,FD5AH
    PUSH IY
    POP HL
    POP DE
    DEC HL
    LD (HL),D
    DEC HL
    LD (HL),E
    LD DE,(170CH)
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
    JR NZ,E1C2H
    JP FFA4H

; BASIC_INPUT - Implements the BASIC INPUT statement.
; usage: trace
BASIC_INPUT:

; KL: INPUT routine.
    EX AF,AF'
    SCF
    EX AF,AF'
    LD HL,E2EBH
    LD (IX+05H),20H
    SCF

LE1D6:
    CALL NC,FC43H
    CP BAH
    JR NZ,E1FCH

; KL: INPUT PROMPT routine.
    CALL FC43H
    CALL F294H
    PUSH IY
    CALL FA21H
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
    JR Z,E1D6H
    CP FDH
    JR Z,E204H
    JR NC,E207H

LE1FA:
    RST 08H
    DB 01H

LE1FC:
    CALL FBFBH
    JR Z,E1EDH
    EX AF,AF'
    JR NC,E1FAH

LE204:
    CALL NC,FC43H

LE207:
    LD A,(1705H)
    CP 20H
    CALL Z,FE7FH
    RST 30H
    INC H
    CALL FF4FH
    JR NZ,E1BEH
    INC HL
    LD (1716H),HL
    OR 37H
    EX AF,AF'
    EXX
    LD A,B
    EXX
    SCF

LE221:
    CALL NC,FC43H
    CP FDH
    JP NC,DBB4H
    AND 81H
    CP 01H
    JP NZ,FD5AH
    EX AF,AF'
    JR NC,E27CH
    LD HL,(1714H)
    LD A,L
    OR H
    JR Z,E268H

LE23A:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,E23AH
    DEC HL
    CP 21H
    JR Z,E25EH
    CP FDH
    JR C,E27CH

LE249:
    EXX
    PUSH HL
    PUSH DE
    PUSH BC
    CALL FCC5H
    EXX
    POP BC
    POP DE
    POP HL
    EXX
    INC HL

LE256:
    CP FBH
    JR Z,E23AH
    CP FEH
    JR C,E249H

LE25E:
    LD HL,(1712H)
    LD E,(HL)
    LD D,00H
    ADD HL,DE
    LD (1712H),HL

LE268:
    LD HL,(1712H)
    LD A,(HL)
    OR A
    JR Z,E27AH
    INC HL
    INC HL
    INC HL

LE272:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,E272H
    JR E256H

LE27A:
    RST 08H
    RLCA

LE27C:
    EX AF,AF'
    PUSH HL
    EXX
    PUSH BC
    CALL F42FH
    POP BC
    BIT 1,C
    EX (SP),HL
    JR NZ,E294H
    CALL F8A1H
    EX (SP),HL
    CALL FB41H
    JR E2B0H

LE292:
    RST 08H
    INC C

LE294:
    CALL FC8EH
    CALL F914H
    EX AF,AF'
    JR NC,E2A6H
    EX AF,AF'
    JR NC,E292H
    RLA
    JP C,F912H
    JR E2ACH

LE2A6:
    EX AF,AF'
    PUSH HL
    CALL EC7FH
    POP HL

LE2AC:
    EX (SP),HL
    CALL FB28H

LE2B0:
    POP HL
    EX AF,AF'
    JR NC,E2CEH
    EX AF,AF'

LE2B5:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,E2B5H
    CP 2CH
    JR Z,E2C9H
    DEC HL
    CP 21H
    JR Z,E2C9H
    BIT 7,(HL)
    JP Z,E292H

LE2C9:
    LD (1714H),HL

LE2CC:
    JR E2DCH

LE2CE:
    EX AF,AF'

LE2CF:
    LD A,(HL)
    INC HL
    CP 2CH
    JR Z,E2D9H
    INC A
    JR NZ,E2CFH
    DEC HL

LE2D9:
    LD (1716H),HL

LE2DC:
    EXX
    LD A,B
    EXX
    CP FDH
    JP NC,DBB4H
    CP A4H
    JP Z,E221H
    RST 08H
    LD BC,3F02H
    DB 20H

; BASIC_IF - Implements the BASIC IF statement.
; usage: trace
BASIC_IF:

; KL: IF routine.
    SET 0,(IX+02H)
    CALL F0A7H
    LD A,(IY+06H)
    OR A
    EX AF,AF'
    CALL FA1BH
    EXX
    PUSH HL
    EXX
    POP HL
    LD A,B5H
    CALL FD54H
    EX AF,AF'
    JR Z,E31CH
    EX AF,AF'
    JR E314H

LE30C:
    INC HL
    PUSH HL
    EXX
    POP HL
    EXX
    CALL FC43H

LE314:
    CP 02H
    JP Z,E3B2H
    JP DB81H

LE31C:
    LD IY,(171AH)

LE320:
    CALL FCC5H
    CP F4H
    JR Z,E30CH
    CP FEH
    JP NC,DBBBH
    PUSH HL
    EXX
    POP HL
    EXX
    JR E320H

; KL: ON routine.
    SET 0,(IX+02H)
    CALL FB1BH
    LD B,A
    EXX
    LD A,B
    EXX
    CP EFH
    JR Z,E3AFH
    CP F0H
    JR NZ,E369H
    CALL E357H
    CALL FBDEH
    LD IY,(171AH)
    PUSH HL
    CALL FCD1H
    EX DE,HL
    POP HL
    JR E38EH

LE357:
    CALL FC43H
    DEC B
    RET Z
    EXX
    PUSH HL
    EXX
    POP HL
    CALL FC43H
    JR NC,E36BH
    CP A4H
    JR Z,E357H

LE369:
    RST 08H
    DB 01H

LE36B:
    POP DE
    LD IY,(171AH)

LE370:
    EXX
    CALL FCC5H
    CP F4H
    JR Z,E30CH
    CP FEH
    JP NC,FB14H
    PUSH HL
    EXX
    POP HL
    JR E370H

; BASIC_GOSUB - Implements the BASIC GOSUB statement.
; usage: trace
BASIC_GOSUB:

; KL: GOSUB routine.
    CALL FBDEH
    CALL FC43H
    JR C,E369H
    EXX
    PUSH HL
    EXX
    POP DE

LE38E:
    PUSH HL
    CALL FC8EH
    PUSH IY
    POP HL
    DEC HL
    LD A,(1702H)
    LD (HL),A
    LD BC,(170CH)
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
    JR E3B9H

LE3AF:
    CALL E357H

LE3B2:
; BASIC_GOTO - Implements the BASIC GOTO statement.
; usage: trace
BASIC_GOTO:

; KL: GOTO routine.
    CALL FBDEH
    LD IY,(171AH)

LE3B9:
    JP DE29H
    DEC HL
    EXX
    CALL FC43H

; BASIC_LET - Implements BASIC assignment.
; usage: trace
BASIC_LET:

; KL: LET routine.
    AND FDH
    CP 01H
    JP NZ,FD5AH
    EXX
    BIT 2,C
    JP NZ,FD5AH
    BIT 3,C
    CALL NZ,F40BH
    LD (IX+01H),C
    EXX
    CALL F42EH
    EXX
    LD A,B
    EXX
    PUSH HL
    CALL E3E8H
    POP HL
    CALL FB3BH
    JP DBB1H

LE3E8:
    SUB 96H
    JR NZ,E44AH
    BIT 1,C
    JP NZ,FD5AH
    INC HL
    PUSH HL
    LD C,(HL)
    LD B,A
    CALL FD27H
    LD A,D
    CP E
    JR NC,E3FEH
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
    JR C,E417H
    JR Z,E417H
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
    CALL E44AH
    POP BC
    POP DE
    POP HL
    LD A,E
    PUSH IY
    POP DE
    INC DE
    CP 02H
    JR C,E432H
    DEC A
    CP C
    JR NC,E42BH
    LD C,A

LE42B:
    ADD HL,BC
    INC C
    DEC C
    JR Z,E432H
    LDDR

LE432:
    POP HL
    DEC HL
    OR A
    SBC HL,DE
    INC H
    DEC H
    JP NZ,F912H
    INC L
    JP Z,F912H
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
    CALL FD54H
    JP F28DH

; KL: LOMEM routine.
    CALL FAC4H
    LD DE,(1720H)
    OR A
    SBC HL,DE
    ADD HL,DE
    JP C,FB14H
    LD DE,(1722H)
    SBC HL,DE
    JP Z,DBB4H
    PUSH HL
    CALL C,FC86H
    LD C,L
    LD B,H
    EX DE,HL
    JR C,E485H
    POP DE
    PUSH DE
    PUSH HL
    INC D
    JP Z,FB14H
    CALL FC99H
    POP HL
    PUSH HL
    CALL DCC9H
    POP HL
    ADD HL,BC
    JR E48DH

LE485:
    OR A
    SBC HL,BC
    PUSH HL
    CALL DCE9H
    POP HL

LE48D:
    LD (1722H),HL
    POP BC
    BIT 2,(IX+00H)
    JR Z,E4A3H
    LD HL,(170CH)
    ADD HL,BC
    LD (170CH),HL
    PUSH BC
    EXX
    POP DE
    ADD HL,DE
    EXX

LE4A3:
    CALL DCFCH
    JP DBB1H

LE4A9:
    RST 08H
    EX AF,AF'

LE4AB:
    EXX
    LD A,B
    EXX
    CP A4H
    JP NZ,DBB4H
    CALL FC43H

; BASIC_NEXT - Implements the BASIC NEXT statement.
; usage: trace
BASIC_NEXT:

; KL: NEXT routine.
    SET 2,(IX+00H)
    JR NC,E4E3H
    CP 03H
    JR Z,E4C7H
    AND 82H
    JP NZ,FD5AH
    RST 08H
    DB 0EH

LE4C7:
    CALL F42EH
    EX DE,HL
    DB 21H

LE4CC:
    ADD IY,BC
    LD A,2BH
    CALL FCE7H
    JR C,E4A9H
    LD C,05H
    ADD HL,BC
    LD C,2BH
    LD A,(HL)
    CP E
    JR NZ,E4CCH
    INC HL
    LD A,(HL)
    CP D
    JR NZ,E4CCH

LE4E3:
    LD A,2BH
    CALL FCE7H
    JR C,E4A9H
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
    CALL FA63H
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
    CALL F493H
    POP AF
    POP HL
    PUSH AF
    PUSH IY
    CALL FB28H
    POP IY
    CALL F693H
    POP BC
    POP DE
    POP HL
    LD A,C
    RLA
    JR NC,E533H
    JR Z,E536H
    JP P,E4ABH
    XOR A

LE533:
    JP M,E4ABH

LE536:
    LD (170CH),DE
    LD DE,FFD5H
    ADD IY,DE
    JP DB81H

; BASIC_OUT - Implements the BASIC OUT statement.
; usage: trace
BASIC_OUT:

; KL: OUT routine.
    CALL FB1BH
    LD C,A
    LD A,A4H
    CALL FD54H
    CALL FB1BH
    OUT (C),A
    JP DBB1H

; BASIC_POKE - Implements the BASIC POKE statement.
; usage: trace
BASIC_POKE:

; KL: POKE routine.
    CALL FCFFH
    LD A,A4H
    CALL FD54H
    CALL FB1BH
    LD C,A
    CALL FFE7H
    JR C,E567H
    DI
    OUT (02H),A

LE567:
    LD (HL),C
    LD A,70H
    OUT (02H),A
    EI
    JP DBB1H

; KL: LPRINT routine.
    LD C,40H
    DB 11H

; BASIC_PRINT - Implements BASIC PRINT and OUTPUT.
; usage: trace
BASIC_PRINT:
    LD C,20H
    LD (IX+05H),C
    EX AF,AF'
    XOR A
    EX AF,AF'
    SCF

LE57C:
    CALL NC,FC43H
    CP AFH
    JR NZ,E5A6H
    CALL FC43H
    CALL F294H
    LD HL,1831H
    LD (HL),7FH
    PUSH HL
    CALL FB41H
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
    JR E5BAH

LE5A6:
    CP B0H
    JR NZ,E5CCH
    CALL FB16H
    LD C,A
    LD A,A4H
    CALL FD54H
    CALL FB1BH
    LD B,A
    CALL E68BH

LE5BA:
    EXX
    LD A,B
    EXX
    EX AF,AF'
    OR A
    EX AF,AF'
    DB FEH

LE5C1:
    AND H
    JR Z,E57CH
    CP FDH
    JR Z,E5D6H
    JR NC,E5D9H

LE5CA:
    RST 08H
    DB 01H

LE5CC:
    CALL FC00H
    JR Z,E5BDH
    EX AF,AF'
    JR C,E5CAH
    EX AF,AF'
    SCF

LE5D6:
    CALL NC,FC43H

LE5D9:
    EX AF,AF'
    SCF
    EX AF,AF'

LE5DC:
    EX AF,AF'
    BIT 0,A
    PUSH AF
    CALL NZ,E69CH
    POP AF
    EX AF,AF'
    EXX
    LD A,B
    EXX
    CP A4H
    JR NZ,E5F2H
    LD A,09H
    CALL FE9AH
    DB C2H

LE5F2:
    CP A0H
    JR NZ,E5FEH
    CALL FC43H
    EX AF,AF'
    OR A
    JP E67FH

LE5FE:
    CP B6H
    JR NZ,E617H
    LD A,96H
    CALL FD51H
    CALL FB1BH
    LD B,A
    LD A,95H
    CALL FD54H
    LD C,00H
    CALL E68BH
    JR E67DH

LE617:
    CP FDH
    JR NC,E683H
    EX AF,AF'
    BIT 0,A
    PUSH AF
    JR Z,E638H
    EX AF,AF'
    LD HL,(172EH)
    LD DE,(1730H)
    OR A
    SBC HL,DE
    JR C,E639H
    LD HL,1833H
    LD (172EH),HL
    CALL E69CH
    EX AF,AF'

LE638:
    EX AF,AF'

LE639:
    EXX
    LD A,B
    EXX
    CP 02H
    JR NC,E665H
    CALL F294H
    RLA
    JR NC,E669H
    POP AF
    LD HL,FF9CH
    JP NZ,FFF0H
    OR FFH
    PUSH IY
    POP HL
    INC HL
    PUSH AF
    CALL FE7FH
    CALL FA21H

LE65A:
    POP AF
    DEC A
    JP M,E67DH
    PUSH AF
    CALL FEC7H
    JR E65AH

LE665:
    CALL F0A7H
    RLA

LE669:
    JP NC,FD5AH
    POP AF
    LD HL,FCB5H
    JP NZ,FFF0H
    PUSH IY
    POP HL
    INC HL
    CALL FEBAH
    CALL FA1BH

LE67D:
    EX AF,AF'
    SCF

LE67F:
    EX AF,AF'
    JP E5DCH

LE683:
    EX AF,AF'
    CALL C,FE93H
    EX AF,AF'
    JP DBB4H

LE68B:
    LD A,(1705H)
    CP 20H
    JR Z,E695H
    CP 00H
    RET NZ

LE695:
    OR 03H
    CALL 001BH
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
    CALL FE9AH
    INC HL
    LD (172EH),HL
    JR E6A3H

; KL: Special format-control characters used by PRINT USING.
    DB 3CH, 3EH, 23H, 2AH, 25H, 2BH, 2DH, 24H, 5EH, 2EH

; KL: RANDOMIZE routine.
    LD A,R
    LD (1709H),A
    LD HL,(0B1DH)
    LD (170AH),HL
    CALL E6D8H
    JP DBB1H

LE6D8:
    LD B,10H
    LD A,(1709H)
    LD HL,(170AH)

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
    DJNZ E6E0H
    LD (170AH),HL
    LD (1709H),A
    RET

; KL: RESTORE routine.
    LD HL,(1722H)
    CP 02H
    JR NZ,E701H
    CALL FBDEH
    CALL FC43H

LE701:
    LD (1712H),HL
    LD HL,0000H
    LD (1714H),HL
    JP DBB4H

LE70D:
    RST 08H
    ADD HL,BC

; BASIC_RETURN - Implements the BASIC RETURN statement.
; usage: trace
BASIC_RETURN:

; KL: RETURN routine.
    LD A,06H
    CALL FCE7H
    JR C,E70DH
    SET 2,(IX+00H)
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    LD C,(HL)
    INC HL
    LD B,(HL)
    LD (170CH),BC
    INC HL
    LD A,(HL)
    LD (1702H),A
    INC HL
    PUSH HL
    POP IY
    EX DE,HL
    JP DB81H

; BASIC_GRAPHICS - Implements the BASIC GRAPHICS statement.
; usage: trace
BASIC_GRAPHICS:

; KL: GRAPHICS routine.
    CALL FB1BH
    LD C,00H
    CP 02H
    JR Z,E747H
    INC C
    CP 04H
    JR Z,E747H
    INC C
    CP 10H
    JP NZ,FB14H

LE747:
    RST 30H
    INC B
    RST 10H
    JP DBB1H

LE74D:
    RST 30H
    EX AF,AF'

LE74F:
    CALL FC43H
    JR NC,E78DH

; BASIC_PLOT - Implements the BASIC PLOT statement.
; usage: trace
BASIC_PLOT:

; KL: PLOT routine.
    JR NC,E782H
    CP BEH
    JR NZ,E75FH

; KL: PLOT PAINT routine.
    RST 30H
    LD A,(BC)
    RST 10H
    JR E77AH

LE75F:
    CP A0H
    JR Z,E74DH
    CP A4H
    JR Z,E784H
    CALL FAC4H
    LD C,L
    LD B,H
    LD A,A4H
    CALL FD54H
    CALL FAC4H
    EX DE,HL
    PUSH AF
    RST 30H
    LD B,D7H
    POP AF

LE77A:
    CP A0H
    JR Z,E74DH
    CP A4H
    JR Z,E784H

LE782:
    RST 30H
    EX AF,AF'

LE784:
    RST 30H
    ADD HL,BC
    EXX
    LD A,B
    EXX
    CP FDH
    JR C,E74FH

LE78D:
    JP DBB4H

LE790:
; BASIC_SET - Dispatches BASIC SET subcommands such as MODE, INK, PAPER, and PALETTE.
; usage: trace
BASIC_SET:
    LD HL,E7E4H
    CALL FD12H
    JR C,E7DAH
    CALL FB1BH

LE79B:
    LD E,00H

LE79D:
    LD (001FH),A
    LD HL,1941H
    PUSH HL
    LD B,7FH

LE7A6:
    DEC E
    JP P,E7B4H
    EXX
    LD A,B
    EXX
    CP A4H
    JR NZ,E7CDH
    CALL FC43H

LE7B4:
    CP 02H
    JR NC,E7C5H
    CALL F294H
    LD HL,193FH
    LD (HL),7FH
    CALL FB41H
    JR E7D2H

LE7C5:
    CALL FB1BH
    LD (HL),A
    INC HL
    DJNZ E7A6H
    INC B

LE7CD:
    LD (HL),00H
    INC HL
    DJNZ E7CDH

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
    CALL FC43H
    JR E790H

; KL: SET subcommand dispatch table; token plus relative routine selector.
    DB C3H, 11H, B7H, 12H, C4H, 13H, BCH, 14H, C7H, 15H, B9H, 16H, C8H, 24H, BDH, 2DH
    DB AEH, 34H, 00H, 1EH, 00H, 21H, 1EH, 01H, 21H, 1EH, 02H, 21H, 1EH, 03H, 21H, 1EH
    DB 1AH, 21H, 1EH, 1CH, 16H, 00H, 21H, 4BH, 0BH, 19H, CDH, 1BH, FBH, 77H

LE812:
    EXX
    LD A,B
    EXX
    RET

; KL: SET CHARACTER routine.
    POP HL
    POP HL
    CALL FB1BH
    LD C,A
    LD A,0BH
    JP E79BH

; KL: SET PALETTE routine.
    POP HL
    POP HL
    LD A,0CH
    LD E,01H
    JP E79DH

; KL: SET BORDER routine.
    CALL FB1BH
    ADD A,A
    LD (0B4FH),A
    JR E812H

; BASIC_SOUND - Implements the BASIC SOUND statement.
; usage: trace
BASIC_SOUND:

; KL: SOUND routine.
    LD HL,0D15H
    LD (172AH),HL
    LD HL,3207H
    LD (002CH),HL
    LD HL,0B15H
    LD (HL),FFH
    SCF

LE845:
    CALL NC,FC43H
    CP A0H
    JR Z,E86FH
    LD HL,E87BH
    CALL FD12H
    CP A4H
    JR Z,E845H
    PUSH AF
    LD DE,(172AH)
    LD BC,(002CH)
    RST 30H
    INC SP
    XOR A
    LD (0B15H),A
    POP AF
    CP FDH
    JR NC,E878H
    CP A0H
    JP NZ,FD5AH

LE86F:
    XOR A
    LD (0B15H),A
    CALL FC43H
    JR C,E845H

LE878:
    JP DBB4H

; KL: SOUND subcommand dispatch table; same format as SET table.
    DB BBH, 05H, C6H, 0CH, B3H, 0BH, 00H, CDH, C4H, FAH, 22H, 2AH, 17H, F0H, CFH, 04H
    DB F6H, 37H, 11H, 2CH, 00H, 38H, 01H, 13H, CDH, 1BH, FBH
    LD (DE),A
    EXX
    LD A,B
    EXX
    RET

; KL: BASIC CLOSE routine.
    LD C,50H
    CALL FBEEH
    CP E1H
    JR Z,E8AFH
    SET 7,C
    CP FDH
    JR NC,E8B2H
    CP ECH
    JP NZ,FD5AH

LE8AF:
    CALL FC43H

LE8B2:
    LD A,C
    AND 7FH
    CP 50H
    JR Z,E8BDH
    CP 60H
    JR NZ,E8C7H

LE8BD:
    LD A,C
    OR 04H
    CALL 001BH
    RST 10H
    JP DBB1H

LE8C7:
    RST 08H
    RST 38H

; BASIC_OPEN - Implements the BASIC OPEN statement.
; usage: trace
BASIC_OPEN:

; KL: OPEN BASIC routine.
    LD C,50H
    CALL FBEEH
    EX AF,AF'
    LD A,C
    CP 50H
    JR Z,E8D8H
    CP 60H
    JR NZ,E8C7H

LE8D8:
    OR 03H
    LD C,A
    EX AF,AF'
    LD DE,19CEH
    CP E1H
    JR Z,E8F2H
    SET 7,C
    CP 02H
    JR C,E8F9H
    CP FDH
    JR NC,E905H
    CP ECH
    JP NZ,FD5AH

LE8F2:
    CALL FC43H
    CP 02H
    JR NC,E905H

LE8F9:
    PUSH BC
    CALL F294H
    PUSH IY
    CALL FA21H
    POP DE
    INC DE
    POP BC

LE905:
    LD A,C
    LD (0B6BH),A
    CALL 001BH
    RST 10H
    JP DBB1H

; KL: GET routine.
    LD C,10H
    CALL FBEEH
    EX AF,AF'
    LD A,C
    OR 81H
    CALL 001BH
    PUSH IY
    POP HL
    CP ECH
    JR Z,E929H
    OR A
    RST 10H
    DEC HL
    LD (HL),C
    INC A
    DB 0EH

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
    JR NZ,E93FH
    EXX
    BIT 2,C

LE93F:
    JP NZ,FD5AH
    BIT 3,C
    CALL NZ,F40BH
    EXX
    CALL F42EH
    CALL FB41H
    JP DBB1H

; BASIC_LOAD - Implements the BASIC LOAD statement.
; usage: trace
BASIC_LOAD:

; KL: LOAD routine.
    CALL EA35H
    CALL DE10H
    PUSH HL
    LD DE,(19E1H)
    PUSH DE
    CALL FC99H
    POP BC
    POP DE
    PUSH DE
    LD A,(1705H)
    OR 82H
    CALL 001BH
    PUSH AF
    CALL NZ,DE10H
    POP AF
    RST 10H
    CALL DCFCH
    CALL EA5AH
    POP HL
    LD A,(19E3H)
    OR A
    JP Z,DADAH
    JP DE23H

; BASIC_SAVE - Implements the BASIC SAVE statement.
; usage: trace
BASIC_SAVE:

; KL: SAVE routine.
    XOR A
    CALL E9FBH
    LD DE,19EFH
    LD B,10H
    XOR A

LE98C:
    DEC DE
    LD (DE),A
    DJNZ E98CH
    PUSH DE
    INC A
    LD (19E0H),A
    LD A,(1707H)
    LD (19E3H),A
    CALL DD41H
    INC HL
    LD DE,(1722H)
    XOR A
    LD (1707H),A
    SBC HL,DE
    LD (19E1H),HL
    EX (SP),HL
    PUSH DE
    LD B,10H
    LD A,(1705H)
    OR 01H
    LD (001FH),A

LE9B8:
    LD C,(HL)
    INC HL
    PUSH BC
    CALL 001EH
    RST 10H
    POP BC
    DJNZ E9B8H
    POP DE
    POP BC
    LD A,(1705H)
    OR 02H
    CALL 001BH
    RST 10H
    CALL EA5AH
    JP DBB1H

; KL: VERIFY routine.
    CALL EA35H
    CALL DD41H
    INC HL
    LD DE,(1722H)
    OR A
    SBC HL,DE
    LD BC,(19E1H)
    SBC HL,BC
    JP NZ,EA58H
    LD A,(1705H)
    OR 85H
    CALL 001BH
    RST 10H
    CALL EA5AH
    JP DBB1H

LE9F9:
    LD A,80H

LE9FB:
    SET 3,(IX+00H)
    PUSH AF
    LD C,50H
    CALL FBEEH
    EX AF,AF'
    LD A,C
    CP 50H
    JR Z,EA10H
    CP 60H
    JP NZ,E8C7H

LEA10:
    POP BC
    OR B
    LD (1705H),A
    OR 03H
    LD (001FH),A
    EX AF,AF'
    LD DE,19CEH
    CP 02H
    JR NC,EA2CH
    CALL F294H
    PUSH IY
    CALL FA21H
    POP DE
    INC DE

LEA2C:
    XOR A
    LD (0B6BH),A
    CALL 001EH
    RST 10H
    RET

LEA35:
    CALL E9F9H
    LD A,(1705H)
    OR 81H
    LD (001FH),A
    LD HL,19DFH
    PUSH HL
    LD B,10H

LEA46:
    PUSH BC
    CALL 001EH
    RST 10H
    LD (HL),C
    INC HL
    POP BC
    DJNZ EA46H
    POP HL
    OR (HL)
    JR NZ,EA58H
    INC HL
    INC A
    SUB (HL)
    RET Z

LEA58:
    RST 08H
    DB 10H

LEA5A:
    LD A,(1705H)
    OR 04H
    CALL 001BH
    RST 10H
    RES 3,(IX+00H)
    RET

LEA68:
; EVAL_NUMERIC_ARGUMENT - Evaluates a parenthesized numeric function argument onto the BASIC stack.
; usage: trace,call
EVAL_NUMERIC_ARGUMENT:
    LD A,96H
    CALL FD54H
    CALL F0A7H

LEA70:
    LD A,95H
    JP FD54H

LEA75:
    LD A,96H
    CALL FD54H
    CALL F294H
    LD A,95H
    JP FD54H
    LD HL,(1718H)
    LD A,(HL)
    INC HL
    LD (1718H),HL
    LD L,A
    XOR A
    LD H,A
    PUSH HL
    ADD HL,HL
    ADD HL,HL
    ADD HL,HL
    POP DE
    SBC HL,DE
    DB 11H

LEA95:
    LD DE,19C1H
    JR EAA2H

LEA9A:
    LD HL,19C7H
    JR EAA2H

LEA9F:
    LD HL,19C0H

LEAA2:
    CALL FC8EH
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
    LD DE,19C7H
    JR EAC6H

LEAC3:
    LD DE,19C0H

LEAC6:
    CALL EAD5H
    PUSH HL
    POP IY
    RET

LEACD:
    LD DE,19C7H
    JR EAD5H

LEAD2:
    LD DE,19C0H

LEAD5:
    PUSH IY
    POP HL
    INC HL
    LD BC,0006H
    LDIR
    INC HL
    LDI
    RET
    NOP
    NOP
    INC BC
    LD B,C
    LD B,D
    LD D,E
    LD A,(BC)

; BASIC_ABS - Implements the BASIC ABS function.
; usage: trace
BASIC_ABS:
    CALL EA68H
    RES 7,(IY+08H)
    RET

LEAF1:
    CALL FC43H
    CALL EA68H
    LD BC,0000H
    LD A,(IY+08H)
    AND 80H
    LD C,A
    PUSH BC
    CALL NZ,F726H
    RST 18H
    EX AF,AF'
    ADD A,L
    LD BC,93CDH
    OR FAH
    LD A,(DE)
    EX DE,HL
    JR Z,EB1AH
    POP BC
    LD B,02H
    PUSH BC
    RST 18H
    DEC B
    LD BC,0106H
    ADC A,D

LEB1A:
    RST 18H
    LD B,85H
    LD (BC),A
    CALL F693H
    JP M,EB3CH
    JR Z,EB3CH
    POP BC
    INC B
    PUSH BC
    RST 18H
    DEC B
    INC B
    LD B,02H
    DEC B
    NOP
    INC BC
    DEC B
    NOP
    INC BC
    LD B,00H
    DEC B
    INC BC
    LD B,00H
    DB 01H, 8AH

LEB3C:
    RST 18H
    LD B,0CH
    INC C
    INC C
    LD (BC),A
    EX AF,AF'
    DEC B
    LD B,02H
    DEC B
    DEC B
    NOP
    LD B,02H
    LD B,05H
    EX AF,AF'
    NOP
    LD B,02H
    DEC B
    RLCA
    NOP
    LD BC,8002H
    POP BC
    LD A,B
    CP 02H
    CALL NC,F726H
    PUSH BC
    LD HL,C214H
    DEC B
    JR Z,EB71H
    LD HL,C206H
    DEC B
    JR Z,EB71H
    LD HL,C20DH
    DEC B
    JR NZ,EB77H

LEB71:
    CALL EAA2H
    CALL F493H

LEB77:
    POP BC
    LD A,C
    XOR (IY+08H)
    LD (IY+08H),A
    RET
    JP PO,04EAH
    LD B,E
    LD C,B
    LD D,D
    INC H
    EX AF,AF'
    CALL EA68H
    CALL FB1AH
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
    ADD A,B
    EX DE,HL
    INC BC
    LD B,E
    LD C,A
    LD D,E
    LD A,(BC)

; BASIC_COS - Implements the BASIC COS function.
; usage: trace
BASIC_COS:
    CALL EA68H
    RST 18H
    DEC B
    INC HL
    ADD A,B
    JP EE5EH
    SBC A,L
    EX DE,HL
    INC BC
    LD B,L
    LD E,B
    LD D,B
    LD A,(BC)
    CALL EA68H

LEBB8:
    RST 18H
    INC C
    DEC B
    ADD HL,BC
    ADD A,D
    CALL ECAFH
    LD A,(IY+08H)
    LD C,A
    AND 7FH
    CP 43H
    JR C,EBD4H
    OR C
    JP P,F912H

LEBCE:
    CALL FA1BH
    JP F9F9H

LEBD4:
    CALL FA92H
    CALL FAC3H
    LD E,L
    LD D,H
    LD BC,007EH
    OR A
    ADC HL,BC
    JP M,EBCEH
    SLA C
    SBC HL,BC
    JP NC,F912H
    PUSH DE
    RST 18H
    ADD HL,BC
    DEC B
    LD A,(BC)
    LD (BC),A
    INC BC
    RLCA
    DEC B
    DEC BC
    LD (BC),A
    INC BC
    INC C
    INC C
    LD (BC),A
    EX AF,AF'
    DEC B
    DJNZ EBFFH

LEBFF:
    LD B,02H
    DEC B
    RRCA
    NOP
    DEC BC
    DEC B
    LD C,06H
    LD (BC),A
    DEC B
    DEC C
    NOP
    LD B,02H
    DEC B
    INC C
    NOP
    LD (BC),A
    EX AF,AF'
    RLCA
    LD B,03H
    LD BC,0005H
    NOP
    INC C
    NOP
    ADC A,D
    POP DE
    BIT 0,E
    JR Z,EC37H
    PUSH DE
    RST 18H
    LD B,05H
    JR NZ,EC2AH

LEC28:
    ADC A,D
    POP DE

LEC2A:
    INC D
    DEC D
    INC DE
    JR Z,EC37H
    JP M,EC37H
    LD HL,19C6H
    INC (HL)
    DEC DE

LEC37:
    LD HL,19C6H
    XOR A
    OR E
    RRA
    ADD A,(HL)
    AND 7FH
    LD (HL),A
    JP EA9FH
    XOR (HL)
    EX DE,HL
    INC B
    LD B,(HL)
    LD D,D
    LD B,L
    LD B,L
    LD A,(BC)
    PUSH IY
    POP HL
    LD DE,(1726H)
    OR A
    SBC HL,DE
    LD DE,0100H
    SBC HL,DE
    LD DE,7FFFH
    SBC HL,DE
    PUSH DE
    CALL FA2BH
    POP HL
    CALL FA2BH
    JP F493H
    LD B,H
    CALL PE,4902H
    LD C,(HL)
    LD A,(BC)

; BASIC_IN - Implements the BASIC IN function.
; usage: trace
BASIC_IN:
    CALL EA68H
    CALL FB1AH
    LD C,A
    IN L,(C)
    LD H,00H
    JP FA2BH

LEC7F:
    RET NC
    RLA
    RET NC
    CALL FA1BH
    RST 18H
    ADD A,L
    DAA
    RET

LEC89:
    CALL FC43H
    RST 30H
    SUB E
    RST 10H
    PUSH IY
    POP HL
    DEC HL
    LD A,C
    OR A
    JR Z,EC9DH
    RST 30H
    SUB C
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
    LD L,E
    CALL PE,4903H
    LD C,(HL)
    LD D,H
    LD A,(BC)
    CALL EA68H

LECAF:
    LD A,(IY+08H)
    AND 7FH
    SUB 40H
    JR C,ECD8H
    INC A
    PUSH AF
    CALL F707H
    CALL F9DFH
    POP AF
    ADD A,03H
    LD B,A

LECC4:
    LD A,00H
    CALL C,F7E3H
    INC B
    LD A,B
    CP 10H
    JR C,ECC4H
    CALL F707H
    CALL F9DFH
    JP F734H

LECD8:
    LD A,(IY+08H)
    OR A
    JP P,F9F9H
    CALL FA08H
    JP F726H
    AND L
    CALL PE,4902H
    LD C,A
    LD A,(BC)
    CALL EA68H
    CALL FB1AH
    LD L,A
    LD H,00H
    LD DE,1941H
    ADD HL,HL
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    EX DE,HL
    JP FA2BH
    PUSH HL
    CALL PE,4C03H
    LD B,L
    LD C,(HL)
    LD A,(BC)
    CALL EA75H

LED0A:
    PUSH IY
    CALL FA21H
    POP HL

LED10:
    INC HL
    LD L,(HL)

LED12:
    LD H,00H
    JP FA2BH
    NOP
    DB EDH,03H
    LD C,H
    LD C,A
    LD B,A
    LD A,(BC)
    CALL EA68H

LED21:
    CALL EAC3H
    LD HL,19C5H
    XOR A
    LD D,A
    OR (HL)
    JP Z,FB14H
    INC HL
    LD E,(HL)
    INC E
    DEC E
    JP M,FB14H
    LD (HL),3FH
    PUSH DE
    PUSH HL
    RST 18H
    LD B,85H
    JR NZ,ED0AH
    SUB E
    OR E1H
    JR Z,ED45H
    JP P,ED49H

LED45:
    POP DE
    DEC DE
    INC (HL)
    PUSH DE

LED49:
    RST 18H
    LD B,05H
    NOP
    INC BC
    DEC B
    NOP
    INC BC
    LD B,05H
    LD BC,0100H
    EX AF,AF'
    INC C
    INC C
    LD (BC),A
    EX AF,AF'
    DEC B
    INC D
    LD (BC),A
    DEC B
    INC DE
    NOP
    LD B,02H
    DEC B
    LD (DE),A
    NOP
    LD B,02H
    LD B,05H
    RLA
    NOP
    LD B,02H
    DEC B
    LD D,00H
    LD B,02H
    DEC B
    DEC D
    NOP
    LD BC,1805H
    NOP
    ADD A,D
    POP HL
    LD DE,003FH
    OR A
    SBC HL,DE
    CALL FA2BH
    RST 18H
    NOP
    DEC B
    LD DE,C982H

LED8B:
    CALL FAA6H
    CALL EAD2H
    CALL F5FBH
    CALL ECAFH
    RST 18H
    LD (BC),A
    INC BC
    ADC A,H
    RES 7,(IY+08H)
    CALL EA9FH
    CALL F693H
    RET M
    JP F9F9H

LEDA9:
    CALL FC43H
    CALL EA75H
    PUSH IY
    CALL FA21H
    POP HL
    INC HL
    LD A,(HL)
    OR A
    JP Z,FB14H
    JP ED10H
    RLA
    DB EDH,04H
    LD D,B
    LD B,L
    LD B,L
    LD C,E
    LD A,(BC)

; BASIC_PEEK - Implements the BASIC PEEK function.
; usage: trace
BASIC_PEEK:
    CALL EA68H
    CALL FD02H
    CALL FFE7H
    JR C,EDD4H
    DI

; KL: Memory paging: U U V S page layout.
    OUT (02H),A

LEDD4:
    LD L,(HL)
    LD A,70H
    OUT (02H),A
    EI
    JP ED12H
    CP (HL)
    DB EDH,02H
    LD D,B
    LD C,C
    LD A,(BC)
    RST 18H
    ADD A,L
    LD (DDC9H),HL
    DB EDH,03H
    LD D,D
    LD C,(HL)
    LD B,H
    LD A,(BC)
    CP 96H
    JR Z,EE02H
    CALL E6D8H
    CALL FA2BH
    RST 18H
    DEC B
    LD H,00H
    DEC B
    LD H,0CH
    NOP
    ADD A,C
    RET

LEE02:
    CALL EA68H
    CALL FAC3H
    BIT 7,H
    JP NZ,FB14H
    LD A,H
    OR L
    JP Z,FB14H
    PUSH HL
    XOR A

LEE14:
    INC A
    ADD HL,HL
    JR NC,EE14H
    DEC A
    POP HL
    DEC HL

LEE1B:
    PUSH HL
    PUSH AF
    CALL E6D8H
    POP AF
    LD B,A

LEE22:
    SRL H
    RR L
    DJNZ EE22H
    EX DE,HL
    POP HL
    OR A
    SBC HL,DE
    ADD HL,DE
    JR C,EE1BH
    EX DE,HL
    JP FA2BH
    RST 20H
    DB EDH,03H
    LD D,E
    LD B,A
    LD C,(HL)
    LD A,(BC)
    CALL EA68H
    LD A,(IY+06H)
    OR A
    RET Z
    LD A,(IY+08H)
    PUSH AF
    CALL FA08H
    POP AF
    AND 80H
    OR (IY+08H)
    LD (IY+08H),A
    RET
    INC (HL)
    XOR 03H
    LD D,E
    LD C,C
    LD C,(HL)
    LD A,(BC)

; BASIC_SIN - Implements the BASIC SIN function.
; usage: trace
BASIC_SIN:
    CALL EA68H

LEE5E:
    RST 18H
    DEC B
    LD (2205H),HL
    ADD A,B
    CALL ED8BH
    LD A,(IY+08H)
    OR A
    RRA
    PUSH AF
    RST 18H
    ADC A,D

LEE6F:
    RST 18H
    LD B,85H
    INC HL
    POP AF
    PUSH AF
    CALL M,F726H
    CALL F693H
    POP BC
    PUSH BC
    JR Z,EE9AH
    XOR B
    JP M,EE93H
    RST 18H
    LD B,85H
    LD (EEF1H),HL
    LD B,B
    PUSH AF
    CALL P,F726H
    RST 18H
    NOP
    ADC A,D
    JR EE6FH

LEE93:
    POP AF
    XOR 80H
    PUSH AF
    JP M,EE6FH

LEE9A:
    RST 18H
    LD B,0CH
    INC C
    INC C
    LD (BC),A
    EX AF,AF'
    DEC B
    DEC E
    LD (BC),A
    DEC B
    INC E
    NOP
    LD B,02H
    DEC B
    DEC DE
    NOP
    LD B,02H
    DEC B
    LD A,(DE)
    NOP
    LD B,02H
    DEC B
    ADD HL,DE
    NOP
    LD B,02H
    LD (BC),A
    ADD A,B
    POP AF
    ADD A,A
    RET P
    JP F726H
    LD D,H
    XOR 03H
    LD D,E
    LD D,C
    LD D,D
    LD A,(BC)

; BASIC_SQR - Implements the BASIC SQR function.
; usage: trace
BASIC_SQR:
    CALL EA68H
    BIT 7,(IY+08H)
    JP NZ,FB14H
    LD A,(IY+06H)
    OR A
    RET Z
    CALL EAC3H
    LD HL,19C6H
    LD A,(HL)
    SUB 3FH
    PUSH AF
    LD (HL),3FH
    RST 18H
    LD B,05H
    RRA
    LD (BC),A
    DEC B
    LD E,00H
    ADC A,E
    LD B,04H

LEEED:
    PUSH BC
    RST 18H
    RLCA
    LD B,07H
    LD BC,0307H
    DEC B
    NOP
    LD (BC),A
    NOP
    ADC A,E
    POP BC
    DJNZ EEEDH
    POP AF
    BIT 0,A
    JR Z,EF0BH
    PUSH AF
    RST 18H
    RLCA
    DEC B
    JR NZ,EF0AH
    ADC A,E
    POP AF

LEF0A:
    INC A

LEF0B:
    SRL A
    LD HL,19CDH
    ADD A,(HL)
    AND 7FH
    LD (HL),A
    JP EA9AH

LEF17:
    LD A,(IY+06H)
    OR A
    JR NZ,EF23H
    CALL FA1BH
    JP FA08H

LEF23:
    LD A,(IY+0FH)
    OR A
    JP Z,FA1BH
    LD A,(IY+08H)
    AND 7FH
    CP 43H
    JR NC,EF7DH
    CALL FA92H
    CALL FAC3H
    PUSH HL
    CALL FA92H
    CALL F767H
    POP HL
    PUSH HL
    CALL FA2BH
    CALL F693H
    POP HL
    JR NZ,EF7DH
    BIT 7,H
    PUSH AF
    CALL NZ,FC86H
    PUSH HL
    RST 18H
    LD A,(BC)
    LD A,(BC)
    ADD A,L
    DB 01H

LEF57:
    POP HL
    PUSH HL
    BIT 0,L
    JR Z,EF63H
    CALL EA9FH
    CALL F512H

LEF63:
    POP HL
    SRL H
    RR L
    LD A,H
    OR L
    JR Z,EF74H
    PUSH HL
    RST 18H
    LD B,06H
    LD (BC),A
    ADC A,D
    JR EF57H

LEF74:
    POP AF
    RET Z
    RST 18H
    DEC BC
    DEC B
    LD BC,8107H
    RET

LEF7D:
    CALL EABEH
    CALL ED21H
    CALL EA9AH
    CALL F512H
    JP EBB8H
    RET NZ
    XOR 04H
    LD D,E
    LD D,H
    LD D,D
    INC H
    EX AF,AF'
    CALL EA68H
    CALL F80EH
    CALL FA1BH
    JP FA7DH
    ADC A,H
    RST 28H
    RLCA
    LD D,E
    LD D,H
    LD D,D
    LD C,C
    LD C,(HL)
    LD B,A
    INC H
    EX AF,AF'
    LD A,96H
    CALL FD54H
    CALL FB1BH
    PUSH AF
    LD A,A4H
    CALL FD54H
    CP 02H
    JR NC,EFD0H
    CALL F294H
    LD A,(IY+01H)
    OR A
    JR Z,EFCDH
    LD A,(IY+02H)
    CALL FA21H
    DB 01H

LEFCD:
    POP BC
    PUSH AF
    SCF

LEFD0:
    CALL NC,FB1BH
    PUSH AF
    CALL EA70H
    POP DE
    PUSH IY
    POP HL
    DEC HL
    POP AF
    INC A
    JP Z,F912H
    DEC A
    JR Z,EFE9H
    LD B,A

LEFE5:
    LD (HL),D
    DEC HL
    DJNZ EFE5H

LEFE9:
    LD (HL),A
    DEC HL
    LD (HL),01H
    PUSH HL
    POP IY
    RET
    AND B
    RST 28H
    INC BC
    LD D,H
    LD B,C
    LD C,(HL)
    LD A,(BC)
    CALL EA68H
    CALL EACDH
    CALL EE5EH
    RST 18H
    RLCA
    DEC B
    INC HL
    ADD A,B
    CALL EE5EH
    JP F5FBH
    POP AF
    RST 28H
    INC BC
    LD D,L
    LD D,E
    LD D,D
    LD A,(BC)

; BASIC_USR - Implements the BASIC USR function.
; usage: trace
BASIC_USR:
    LD A,96H
    CALL FD54H
    CALL F0A7H
    CP 95H
    JR Z,F029H
    LD A,A4H
    CALL FD54H
    CALL FAC4H
    LD A,95H

LF029:
    CALL FD54H
    LD DE,FA2BH
    PUSH DE
    PUSH HL
    CALL FD02H
    EX (SP),HL
    RET
    INC C
    RET P
    INC BC
    LD D,(HL)
    LD B,C
    LD C,H
    LD A,(BC)
    CALL EA75H
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
    CALL F914H
    CALL EC7FH
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
    LD (HL),F0H
    LD B,56H
    LD B,C
    LD D,D
    LD D,B
    LD D,H
    LD D,D
    LD A,(BC)
    LD A,96H
    CALL FD54H
    AND FDH
    CP 01H

LF07C:
    JP NZ,FB14H
    EXX
    LD A,C
    AND 7FH
    LD (1708H),A
    AND 0CH
    JR NZ,F07CH
    EXX
    CALL F42EH
    CALL FA2BH
    JP EA70H
    LD L,C
    RET P
    LD B,56H
    LD B,L
    LD D,D
    LD C,(HL)
    LD D,L
    LD C,L
    LD A,(BC)
    LD HL,000CH
    JP FA2BH

LF0A4:
    CALL FC43H

LF0A7:
    EXX
    LD A,B
    EXX
    CALL F0D7H

LF0AD:
    EXX
    LD A,B
    EXX
    CP BFH
    JR Z,F0B8H
    CP B2H
    RET NZ
    SCF

LF0B8:
    PUSH AF
    CALL FC43H
    CALL F0D7H
    CALL FABFH
    POP AF
    JR C,F0CCH
    LD A,L
    OR E
    LD L,A
    LD A,H
    OR D
    JR F0D1H

LF0CC:
    LD A,L
    XOR E
    LD L,A
    LD A,H
    OR D

LF0D1:
    LD H,A
    CALL FA2BH
    JR F0ADH

LF0D7:
    CALL F0F4H

LF0DA:
    EXX
    LD A,B
    EXX
    CP C9H
    RET NZ
    CALL FC43H
    CALL F0F4H
    CALL FABFH
    LD A,L
    AND E
    LD L,A
    LD A,H
    AND D
    LD H,A
    CALL FA2BH
    JR F0DAH

LF0F4:
    CP C2H
    JR NZ,F10AH
    CALL FC43H
    CALL F0F4H
    CALL FAC3H
    LD A,L
    CPL
    LD L,A
    LD A,H
    CPL
    LD H,A
    JP FA2BH

LF10A:
    CP 02H
    JR C,F123H
    CALL F155H
    CP 99H
    RET C
    CP 9FH
    RET NC
    PUSH AF
    CALL FC43H
    CALL F155H
    CALL F693H
    JR F13AH

LF123:
    CALL F294H
    CP 99H
    JP C,F35DH

; KL: Memory paging: U U U E page layout.
    CP 9FH
    JP NC,F35DH
    PUSH AF
    CALL FC43H
    CALL F294H
    CALL F6D7H

LF13A:
    POP HL
    LD A,H
    CALL F142H
    JP FA2BH

LF142:
    LD HL,FFFFH
    RRCA
    JR NC,F149H
    RET M

LF149:
    RRCA
    JR NC,F14DH
    RET Z

LF14D:
    RRCA
    JR NC,F153H
    JR Z,F153H
    RET P

LF153:
    INC HL
    RET

LF155:
    CALL F16CH
    CALL NZ,F181H
    INC E
    CALL NZ,F726H

LF15F:
    CALL F16CH
    RET NZ
    INC E
    CALL Z,F493H
    CALL NZ,F48EH
    JR F15FH

LF16C:

; KL: Expansion output dispatch.
    EXX
    LD A,B
    EXX
    LD E,FFH
    CP 98H
    JR Z,F179H
    CP A2H
    RET NZ
    INC E

LF179:
    CALL FC43H
    CALL F181H
    XOR A
    RET

LF181:
    CALL F1A2H

LF184:
    EXX
    LD A,B
    EXX
    CP A8H
    JR Z,F18EH
    CP A1H
    RET NZ

LF18E:
    LD D,A
    CALL FC43H
    CALL F1A2H
    PUSH DE
    LD A,D
    CP A8H
    CALL Z,F512H
    CALL NZ,F5FBH
    POP DE
    JR F184H

LF1A2:
    PUSH DE
    CALL F1BBH

LF1A6:
    POP DE
    EXX
    LD A,B
    EXX
    CP 9FH
    RET NZ
    PUSH DE
    CALL FC43H
    CALL F1BBH
    CALL EF17H
    JR F1A6H

LF1B9:
    RST 08H
    INC BC

LF1BB:
    PUSH AF
    CALL FC8EH
    POP AF
    CP C0H
    JP Z,EDA9H
    CP B1H
    JP Z,EAF1H
    CP 02H
    JP C,F35DH
    JP Z,FC43H
    CP FDH
    JR NC,F1B9H
    CP 96H
    JR NZ,F1E2H
    CALL F0A4H
    CP 95H
    JP Z,FC43H

LF1E2:
    CP 03H
    JP NZ,FD5AH
    CALL F42EH
    BIT 3,C
    JP NZ,DBADH
    BIT 2,C
    JP Z,FA63H

LF1F4:
    LD D,(IX+01H)
    PUSH DE
    PUSH HL
    SUB 96H
    SUB 01H
    SBC A,A
    PUSH AF
    LD A,C
    JR Z,F216H
    EX AF,AF'
    CALL FC43H
    OR A
    JP M,FD5AH
    LD (IX+01H),A
    CALL F28DH
    LD A,95H
    CALL FD54H
    EX AF,AF'

LF216:
    EX AF,AF'
    POP AF
    LD HL,(1726H)
    EX (SP),HL
    PUSH HL
    LD HL,(1724H)
    EX (SP),HL
    EXX
    PUSH HL
    LD HL,(170CH)
    PUSH HL

; KL: Expansion-card interrupt dispatch.
    PUSH BC
    EXX
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    LD (170CH),DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    EX DE,HL
    EXX
    LD C,A
    CALL FC43H
    SUB 96H
    SUB 01H
    SBC A,A
    XOR C
    JP NZ,FD5AH
    XOR C
    JR Z,F26DH
    CALL FC43H
    AND 81H
    CP 01H
    JP NZ,FD5AH
    EXX
    BIT 7,C
    CALL Z,F40BH
    LD A,C
    XOR (IX+01H)
    AND 02H
    JP NZ,F35DH
    PUSH DE
    EXX
    POP HL
    CALL FB3BH
    CALL FC43H
    LD A,95H
    CALL FD54H

LF26D:
    LD A,9AH
    CALL FD54H
    EX AF,AF'
    LD (IX+01H),A
    CALL F28DH
    POP BC
    POP HL
    LD (170CH),HL
    POP HL
    EXX
    POP HL
    LD (1724H),HL
    POP HL
    LD (1726H),HL
    POP AF
    LD (1701H),A
    RET

LF28D:
    BIT 1,(IX+01H)

; KL: Serial-line routine jump table; first byte is the routine count, followed by routine addresses.
    JP NZ,F0A7H

LF294:
    EXX
    LD A,B
    EXX
    CALL F2E0H

LF29A:
    EXX
    LD A,B
    EXX
    CP 97H
    RET NZ
    CALL FC43H
    CALL F2E0H
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
    CALL FC8EH
    PUSH HL
    ADD HL,BC
    LD A,C
    OR A
    LD A,(DE)
    JR Z,F2C2H
    LDDR

LF2C2:
    POP HL
    ADD A,(HL)
    LD (DE),A
    JP C,F912H
    CP FFH
    JP Z,F912H
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
    JR F29AH

LF2E0:
    CALL F32DH

LF2E3:
    EXX
    LD A,B
    EXX
    CP 96H
    RET NZ
    CALL FD27H
    PUSH IY
    POP HL
    INC HL
    LD A,(HL)
    LD C,A
    LD B,00H
    CP D
    JR C,F2F8H
    LD A,D

LF2F8:
    LD D,A
    OR A
    JR Z,F309H
    LD A,C
    CP E
    JR C,F309H
    INC E
    DEC E
    JR NZ,F305H
    INC E

LF305:
    LD A,D
    SUB E
    JR NC,F30DH

LF309:
    ADD HL,BC
    XOR A
    JR F318H

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
    JR F2E3H
    CALL FAC4H
    LD D,00H
    INC H
    RET Z
    DEC D
    DEC H
    RET NZ
    LD D,L
    RET

LF32D:
    CALL FC8EH
    EXX
    PUSH BC
    EXX
    POP BC
    LD A,B
    CP 01H
    JP C,FC43H
    JR Z,F347H
    CP 04H
    JR C,F35DH
    CP FDH
    JP C,FD5AH
    RST 08H
    INC BC

LF347:
    LD A,C
    CP C5H
    JP Z,EC89H
    CALL F42EH
    BIT 3,C
    JP NZ,DBADH
    BIT 2,C
    JP NZ,F1F4H
    JP FA7CH

LF35D:
    RST 08H
    DB 0EH

LF35F:
    CALL FC8EH
    LD DE,(1724H)
    LD HL,(1726H)
    LD (1724H),HL
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    LD C,00H
    PUSH HL
    LD DE,(1728H)

LF377:
    LD A,(DE)
    CP FDH
    JR NC,F3A3H
    CP 20H
    JR C,F39CH
    CP 24H
    JR Z,F39DH
    CP 2EH
    JR Z,F39CH
    CP 30H
    JR C,F3A3H
    CP 3AH
    JR C,F39CH
    CP 3FH
    JR C,F3A3H
    CP 60H
    JR C,F39CH
    CP A9H
    JR C,F3A3H

LF39C:
    SCF

LF39D:
    INC DE
    INC C
    INC HL
    LD (HL),A
    JR C,F377H

LF3A3:
    LD A,(HL)
    INC HL
    LD (1726H),HL
    EX (SP),HL
    LD (HL),C
    POP HL
    RET

LF3AC:
    CALL FC8EH
    LD HL,(1726H)
    LD B,06H
    XOR A

LF3B5:
    LD (HL),A
    INC HL
    DJNZ F3B5H
    JR F3CEH

LF3BB:
    BIT 1,(IX+01H)
    JR NZ,F3ACH

LF3C1:
    CALL FC8EH
    LD HL,(1726H)
    LD (HL),C
    INC HL
    LD B,00H
    LD (HL),B
    ADD HL,BC
    INC HL

LF3CE:
    LD (1726H),HL
    RET
    LD (1728H),HL
    LD HL,1725H
    DB 3EH

LF3D9:
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

LF3E8:
    LD A,(DE)
    INC DE
    INC HL
    CP (HL)
    JR NZ,F3D9H
    DJNZ F3E8H
    CP 24H
    JR Z,F404H
    LD A,(DE)
    CP 20H
    JR Z,F404H
    CP FDH
    JR NC,F404H
    CP A9H
    JR NC,F3D9H
    RLA
    JR NC,F3D9H

LF404:
    POP AF
    INC HL
    RES 7,(HL)
    SCF
    JR F428H

LF40B:
    CALL F35FH
    SUB 24H
    JR Z,F414H
    LD A,02H

LF414:
    LD (HL),A
    PUSH HL
    INC HL
    LD (1726H),HL
    JR NZ,F422H
    LD C,12H
    CALL F3C1H
    XOR A

LF422:
    CALL NZ,F3ACH
    POP HL
    SET 7,(HL)

LF428:
    LD C,(HL)
    INC HL
    EX DE,HL
    RET

LF42C:
    RST 08H
    DEC B

LF42E:
    EXX

LF42F:
    PUSH DE
    BIT 0,C
    LD A,C
    EXX
    POP HL
    LD C,A
    JP Z,FC43H
    LD A,96H
    CALL FD51H
    XOR A
    LD E,A
    LD D,A
    LD A,(HL)
    INC HL
    EX DE,HL
    PUSH BC

LF445:
    PUSH HL
    PUSH AF
    LD A,A4H
    CALL NZ,FD54H
    CALL FAC4H
    EX DE,HL
    LD C,(HL)
    INC HL
    LD B,(HL)
    INC HL
    EX DE,HL
    OR A
    SBC HL,BC
    JR NC,F42CH
    ADD HL,BC
    POP AF
    PUSH AF
    PUSH DE

LF45E:
    DEC A
    JR Z,F46FH
    EX DE,HL
    LD C,(HL)
    INC HL
    LD B,(HL)
    INC HL
    PUSH HL
    PUSH AF
    CALL FCB3H
    POP AF
    POP DE
    JR F45EH

LF46F:
    POP DE
    POP AF
    POP BC
    ADD HL,BC
    DEC A
    JR NZ,F445H
    LD A,95H
    CALL FD54H
    EX DE,HL
    POP BC
    BIT 1,C
    LD BC,0006H
    JR NZ,F487H
    LD C,(HL)
    INC BC
    INC BC

LF487:
    PUSH HL
    CALL FCB3H
    POP DE
    ADD HL,DE
    RET

LF48E:
; FP_SUB - Subtracts the top two numeric values on the BASIC stack.
; usage: trace,call
FP_SUB:
    PUSH AF
    CALL F726H
    POP AF

LF493:
; FP_ADD - Adds the top two numeric values on the BASIC stack.
; usage: trace,call
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
    JP P,F4BFH
    EX (SP),IY
    PUSH AF
    NEG
    CALL F790H
    POP BC
    EX (SP),IY
    LD A,(IY+11H)
    SUB B
    LD (IY+11H),A
    XOR A

LF4BF:
    CALL NZ,F790H
    LD A,(IY+08H)
    XOR (IY+11H)
    RLCA
    CALL NC,F4F4H
    EX (SP),IY
    JR NC,F4EEH
    CALL F71CH
    EX (SP),IY
    CALL F71CH
    CALL F4F4H
    EX (SP),IY
    LD A,(IY+07H)
    OR A
    JR Z,F4EEH
    CALL F70CH
    LD A,80H
    XOR (IY+08H)
    LD (IY+08H),A

LF4EE:
    POP AF

LF4EF:
    CALL F734H
    POP AF
    RET

LF4F4:
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
    DJNZ F4FEH
    POP AF
    RET

LF508:
    LD DE,0009H
    ADD IY,DE
    CALL F9F9H
    POP AF
    RET

LF512:
; FP_MUL - Multiplies the top two numeric values on the BASIC stack.
; usage: trace,call
FP_MUL:
    PUSH AF
    LD A,(IY+0FH)
    OR A
    JR Z,F508H
    LD A,(IY+06H)
    OR A
    JR Z,F508H
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
    JR C,F508H
    JP M,F912H
    INC A
    JP M,F912H
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
    CALL F5E8H
    PUSH DE
    LD (HL),A
    DEC HL
    LD (HL),A
    DEC HL
    CALL F5E8H
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

LF56C:
    POP HL
    PUSH HL
    PUSH BC
    LD A,(DE)
    EXX
    EX AF,AF'
    PUSH AF
    EX AF,AF'
    ADD A,A
    JR Z,F5B0H
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
    JP P,F58EH
    INC E
    DEC E
    JR Z,F5B0H

LF58A:
    EX AF,AF'
    XOR A
    JR F59BH

LF58E:
    CP 0CH
    JR NC,F58AH
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
    DJNZ F583H

LF5B0:
    POP AF
    INC A
    EX AF,AF'
    DEC C
    EXX
    POP BC
    DEC BC
    INC DE
    JR NZ,F56CH
    POP HL
    LD BC,000FH
    ADD HL,BC
    LD E,L
    LD D,H
    CALL F5DAH
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
    JP F4EFH

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
    DJNZ F5DCH
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
    DJNZ F5EAH
    XOR A
    RET

LF5F9:
    RST 08H
    DEC BC

LF5FB:
; FP_DIV - Divides the top two numeric values on the BASIC stack.
; usage: trace,call
FP_DIV:
    PUSH AF
    LD A,(IY+06H)
    OR A
    JR Z,F5F9H
    LD A,(IY+0FH)
    OR A
    JP Z,FA1AH
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
    CALL F9F1H
    LD B,04H

LF626:
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

LF638:
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
    DJNZ F63BH
    AND F0H
    JR NZ,F65DH
    EXX
    CALL F7D2H
    INC A
    CALL F7E3H
    LD HL,(171CH)
    EXX
    LD BC,0007H
    LDDR
    JR F638H

LF65D:
    EX (SP),IY
    CALL F784H
    EX (SP),IY
    EXX
    INC B
    LD A,B
    CP 10H
    JR C,F626H
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
    JP P,F5D3H
    CALL F9F9H
    JR F68BH

LF693:
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
    JR Z,F6B1H
    EX DE,HL

LF6B1:
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

LF6BE:
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
    DJNZ F6BEH
    RET

LF6D7:
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
    JR C,F6E8H
    LD A,C

LF6E8:
    LD C,(HL)
    ADD HL,BC
    INC HL
    PUSH HL
    POP IY
    LD C,A
    INC BC
    POP DE
    POP HL

LF6F2:
    LD A,(DE)
    INC DE
    CPI
    JP PO,F6FBH
    JR Z,F6F2H

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
    JR NZ,F713H
    RET

LF71C:
    BIT 7,(IY+08H)
    RET Z
    CALL F70CH
    JR F72BH

LF726:

; KL: Arithmetic routine 4: a BASIC Stack-ben levo 9 byte-os szam elojelet az ellenkezojere valtoztatja.
    LD A,(IY+06H)
    OR A
    RET Z

LF72B:
    LD A,80H
    XOR (IY+08H)
    LD (IY+08H),A
    RET

LF734:
    CALL F9DFH
    RET Z
    LD A,(IY+07H)
    OR A
    JR Z,F751H
    LD A,01H
    CALL F790H
    LD A,(IY+08H)
    AND 7FH
    CP 7EH
    JP Z,F912H
    INC (IY+08H)
    RET

LF751:
    LD A,(IY+06H)
    AND F0H
    RET NZ
    CALL F784H
    LD A,(IY+08H)
    DEC (IY+08H)
    AND 7FH
    JR NZ,F751H
    JP F9F9H

LF767:
    PUSH HL
    PUSH IY
    POP HL
    INC HL
    LD A,(HL)
    CP 50H
    LD (HL),00H
    JR C,F782H
    LD B,06H

LF775:
    INC HL
    LD A,(HL)
    ADD A,01H
    DAA
    LD (HL),A
    JR NC,F77FH
    DJNZ F775H

LF77F:
    CALL F734H

LF782:
    POP HL
    RET

LF784:
    PUSH IY
    POP HL
    LD B,07H
    XOR A

LF78A:
    INC HL
    RLD
    DJNZ F78AH
    RET

LF790:
    CP 0EH
    JR NC,F7C4H
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
    JR NC,F79DH
    JR Z,F7B7H
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
    JR NZ,F7B1H
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
    DJNZ F7BEH
    RET

LF7C4:
    LD C,(IY+08H)
    CALL F9F9H
    LD (IY+08H),C
    RET
    LD HL,FEB6H
    PUSH HL

LF7D2:
    PUSH HL
    PUSH BC
    CALL F7FDH
    JR C,F7DEH
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

LF7E3:
    PUSH HL
    PUSH BC
    CALL F7FDH
    LD B,F0H
    JR C,F7F7H
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

LF80E:
; FP_TO_ASCII - Converts a BASIC numeric value to ASCII text.
; usage: trace,call
FP_TO_ASCII:
    LD A,(IY+06H)
    OR A
    LD HL,1931H
    LD (HL),30H
    JR Z,F897H
    BIT 7,(IY+08H)
    JR Z,F825H
    LD (HL),2DH
    INC HL
    CALL F726H

LF825:
    CALL F767H
    LD B,0EH

LF82A:
    DEC B
    LD A,B
    CP 04H
    CALL NC,F7D2H
    JR Z,F82AH
    LD C,B
    LD B,04H
    LD E,(IY+08H)
    LD A,E
    SUB 3BH
    LD D,A
    LD A,E
    CP 4AH
    JR NC,F858H
    CP 40H
    JR NC,F850H
    SUB C
    JR C,F859H
    CP 32H
    JR C,F859H
    LD B,D
    JR F85BH

LF850:
    LD A,C
    CP D
    JR NC,F85BH
    LD C,D
    OR A
    JR F85CH

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
    JR NZ,F864H
    LD (HL),2EH
    INC HL

LF864:
    LD A,B
    SUB 04H
    LD A,00H
    CALL P,F7D2H
    ADD A,30H
    LD (HL),A
    INC HL
    INC B
    LD A,B
    CP C
    JR NZ,F85EH
    POP AF
    JR NC,F898H
    LD (HL),45H
    INC HL
    LD (HL),2BH
    LD A,E
    SUB 40H
    JR NC,F886H
    LD (HL),2DH
    NEG

LF886:
    INC HL
    LD B,FFH

LF889:
    INC B
    LD C,A
    SUB 0AH
    JR NC,F889H
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

LF8A1:
    PUSH DE
    PUSH BC
    LD E,FFH
    LD C,00H

LF8A7:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,F8A7H
    CP 22H
    JR NZ,F8B4H
    INC C

LF8B2:
    LD A,(HL)
    INC HL

LF8B4:
    INC E
    INC C
    DEC C
    JR Z,F8C7H
    CP FFH
    JR Z,F8D3H
    CP 22H
    JR NZ,F8B2H
    CP (HL)
    INC HL
    JR Z,F8B2H
    JR F8D3H

LF8C7:
    CP 2CH
    JR Z,F8D3H
    CP 21H
    JR Z,F8D3H
    CP FDH
    JR C,F8B2H

LF8D3:
    DEC HL
    PUSH HL
    PUSH DE
    LD B,E
    PUSH IY
    POP DE
    CALL FC8EH
    INC B

LF8DE:
    DEC B
    DEC DE
    JR Z,F8F8H
    DEC HL
    LD A,(HL)
    CP 22H
    JR NZ,F8EEH
    INC C
    DEC C
    JR Z,F8EEH
    DEC HL
    LD A,(HL)

LF8EE:
    LD (DE),A
    CP 20H
    JR NC,F8DEH
    OR 80H
    LD (DE),A
    JR F8DEH

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
    LD A,(HL)
    CP 22H
    JR Z,F8A1H
    CALL F914H
    JP NC,FD5AH
    RLA
    RET NC

LF912:
    RST 08H
    DEC C

LF914:
    PUSH BC
    PUSH DE
    CALL F9F4H
    PUSH HL
    PUSH IY
    POP HL
    LD DE,0007H
    ADD HL,DE
    EX (SP),HL
    LD E,3FH
    LD B,0BH

LF926:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,F926H
    CP 2BH
    JR Z,F936H
    CP 2DH
    JR NZ,F938H
    SET 1,D

LF936:
    LD A,(HL)
    INC HL

LF938:
    CP 2EH
    JR Z,F969H
    SUB 3AH
    JR NC,F971H
    ADD A,0AH
    JR NC,F971H
    LD C,A
    SET 7,D
    LD A,D
    RRCA
    JR NZ,F954H
    BIT 2,D
    JR NZ,F956H
    JR NC,F936H
    DEC E
    JR F936H

LF954:
    SET 2,D

LF956:
    JR C,F959H
    INC E

LF959:
    LD A,B
    DEC A
    JR Z,F936H
    LD B,A
    EX (SP),HL
    RRCA
    JR C,F963H
    DEC HL

LF963:
    LD A,C
    RLD
    EX (SP),HL
    JR F936H

LF969:
    SET 7,D
    BIT 0,D
    SET 0,D
    JR Z,F936H

LF971:
    EX (SP),HL
    DEC B
    BIT 0,B
    JR Z,F97AH
    XOR A
    RLD

LF97A:
    POP HL
    DEC HL
    LD A,(HL)
    CALL FBC8H
    CP 45H
    LD A,E
    JR NZ,F9C5H
    SET 7,D
    INC HL
    LD A,(HL)
    CP 98H
    JR Z,F99BH
    CP 2BH
    JR Z,F99BH
    CP A2H
    JR Z,F999H
    CP 2DH
    JR NZ,F99CH

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
    JR NC,F9B7H
    ADD A,0AH
    JR NC,F9B7H
    LD C,A
    LD A,B
    ADD A,A
    LD B,A
    ADD A,A
    ADD A,A
    ADD A,B
    ADD A,C
    LD B,A
    JP P,F99FH
    SET 6,D

LF9B7:
    LD A,B
    BIT 5,D
    JR Z,F9BEH
    NEG

LF9BE:
    ADD A,E
    CP 7FH
    JR C,F9C5H
    SET 6,D

LF9C5:
    PUSH DE
    LD (IY+08H),A
    BIT 1,D
    CALL NZ,F726H
    CALL F9DFH
    POP DE
    LD A,D
    RRA
    AND D
    AND 20H
    LD A,D
    RLA
    CALL NZ,F9F9H
    POP DE
    POP BC
    RET

LF9DF:
    PUSH HL
    PUSH IY
    POP HL
    LD B,07H
    XOR A

LF9E6:
    INC HL
    OR (HL)
    JR NZ,F9EFH
    DJNZ F9E6H
    LD (IY+08H),A

LF9EF:
    POP HL
    RET

LF9F1:
    CALL FC8EH

LF9F4:
    LD DE,FFF7H
    ADD IY,DE

LF9F9:
    PUSH IY
    EX (SP),HL
    LD (HL),09H
    LD B,08H
    XOR A

LFA01:
    INC HL
    LD (HL),A
    DJNZ FA01H
    POP HL
    SCF
    RET

LFA08:
    CALL F9F9H
    LD (IY+08H),40H
    LD (IY+06H),10H
    RET
    LD HL,FEB6H
    PUSH HL
    JR F9F9H

LFA1A:
    POP AF

LFA1B:
    LD DE,0009H
    ADD IY,DE
    RET

LFA21:
    LD E,(IY+01H)
    LD D,00H
    INC DE
    INC DE
    ADD IY,DE
    RET

LFA2B:
    CALL F9F1H
    INC IY
    LD (171CH),IY
    DEC IY
    PUSH HL
    CALL FC83H
    SCF

LFA3B:
    ADC HL,HL
    JR NC,FA3BH
    LD C,01H
    JR FA52H

LFA43:
    INC B
    INC C
    DB 3EH

LFA46:
    LD B,C

LFA47:
    INC DE
    LD A,(DE)
    ADC A,A
    DAA
    LD (DE),A
    DJNZ FA47H
    JR C,FA43H
    ADC HL,HL

LFA52:
    LD DE,(171CH)
    JR NZ,FA46H
    POP AF
    AND 80H
    OR 49H
    LD (IY+08H),A
    JP F734H

LFA63:
    CALL FC8EH
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
    JR FA8DH

LFA7C:
    INC HL

LFA7D:
    LD C,(HL)
    LD B,00H
    CALL FC8EH
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
    CALL FC8EH
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
    CALL FC8EH
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
    CALL FC43H
    JR FAC4H

LFABF:
    CALL FAC3H
    EX DE,HL

LFAC3:
    DB F6H

LFAC4:
    SCF

LFAC5:
    PUSH BC
    PUSH DE
    CALL C,F0A7H
    LD A,(IY+08H)
    PUSH AF
    AND 7FH
    SBC HL,HL
    CP 45H
    JR NC,FB14H
    SUB 40H
    JR C,FB06H
    ADD A,05H
    LD C,A
    LD B,04H

LFADF:
    LD A,H
    AND E0H
    JR NZ,FB14H
    ADD HL,HL
    LD E,L
    LD D,H
    ADD HL,HL
    ADD HL,HL
    ADD HL,DE
    JR C,FB14H
    CALL F7D2H
    LD E,A
    LD D,00H
    ADD HL,DE

; KL: Cassette work bytes used by cassette routines.
    JR C,FB14H
    INC B
    LD A,C
    CP B
    JR NZ,FADFH
    POP AF
    PUSH AF
    OR A
    CALL M,FC86H
    POP AF
    XOR H
    JP M,FB14H
    DB 3EH

LFB06:
    POP AF
    LD DE,0009H
    ADD IY,DE
    LD A,H
    OR A

; KL: Initialization bytes copied to RAM addresses 0B00H-0B48H.
    EXX
    LD A,B
    EXX
    POP DE
    POP BC
    RET

LFB14:
    RST 08H
    INC B

LFB16:
    CALL FC43H
    DB 3EH

LFB1A:
    DB F6H

LFB1B:
    SCF
    PUSH HL
    CALL FAC5H
    INC H
    DEC H
    JP NZ,FB14H
    LD A,L
    POP HL
    RET

LFB28:

; KL: Arithmetic routine 13: a BASIC Stack-bol a HL regiszter altal mutatott helyre mozgat. A Stackbol felszabadul 9 byte.
    CALL F767H
    EX DE,HL
    PUSH IY
    POP HL
    INC HL
    INC HL
    LD BC,0005H
    LDIR
    INC HL
    LDI
    JR FB57H

LFB3B:
    BIT 1,(IX+01H)
    JR NZ,FB28H

LFB41:
    LD C,(HL)
    INC HL
    PUSH IY
    POP DE
    INC DE
    LD A,(DE)
    INC C
    JR NZ,FB4CH
    DEC C

LFB4C:
    CP C
    JP NC,F912H
    LD C,A
    LD B,00H
    INC BC

; KL: "MOPS" cartridge signature text.
    EX DE,HL
    LDIR

LFB57:
    PUSH HL

; KL: "VGB" device identifier.
    POP IY
    RET

; KL: Bytes copied into RAM at initialization, including RST and system entry stubs.
    DB E1H, 7EH, B7H, E5H, 00H, 00H, 00H, 00H, C8H, E1H, C3H, 5CH, FDH, C3H, 00H, 00H

; KL: Character matrix table for character codes 128-160, ten bytes per character.
    DB C3H, 82H, FBH, 32H, 1FH, 00H, F7H, 00H, C9H, 00H, 00H, 00H, 00H, 00H, 00H, 00H
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H

; RST18_DISPATCH - Dispatches BASIC-stack arithmetic operations requested through RST 18H.
; usage: trace
RST18_DISPATCH:
    DB 22H, 1EH, 17H, 32H, 04H, 17H, 2AH, 18H, 17H, E3H, 7EH, 23H, 22H, 18H, 17H, 87H
    DB F5H, C6H, C7H, 6FH, 26H, C0H, 7EH, 23H, 66H, 6FH, 22H, 16H, 00H, 2AH, 1EH, 17H
    DB 3AH, 04H, 17H, CDH, 15H, 00H, 32H, 04H, 17H, 22H, 1EH, 17H, 2AH, 18H, 17H, F1H
    DB 30H, D8H, E3H, 22H, 18H, 17H, 2AH, 1EH, 17H, 3AH, 04H, 17H, C9H, 1AH, 04H, 05H
    DB C0H, FEH, FFH, C8H, E6H, 7FH, FEH, 20H, 38H, 09H, FEH, 61H, D8H, FEH, 7BH, D0H
    DB E6H, DFH, C9H, FEH, 10H, D8H, FEH, 19H, D0H, E6H, EFH, C9H, FEH, 02H, C2H, 5AH
    DB FDH, CDH, C3H, FAH, CDH, 44H, DDH, D0H, CFH, 02H, 0EH, 20H, CDH, FDH, FBH, C0H
    DB FEH, FDH, 28H, 4DH, FEH, FEH, D0H, CFH, 01H, 0EH, 20H, DDH, 71H, 05H, D9H, 78H
    DB D9H, FEH, A7H, C0H, CDH, 16H, FBH, E6H, 07H, 87H, 87H, 87H, 87H, 32H, 05H, 17H
    DB 4FH, AFH, D9H, 78H, D9H, C9H, DDH, 36H, 05H, 20H, 3AH, 4EH, 0BH, 4FH, 3AH, 4DH
    DB 0BH, B9H, C0H, 3CH, 32H, 4DH, 0BH, 3AH, 13H, 0BH, CBH, 4FH, C8H, 79H, 2FH, 32H
    DB 4DH, 0BH, C9H, DDH, CBH, 00H, 5EH, C8H, DDH, CBH, 00H, 9EH, F7H, 54H, F7H, D4H
    DB C9H, D9H, 7EH, 47H, FEH, FEH, 30H, 30H, 23H, FEH, C5H, 28H, 2FH, FEH, 80H, 30H
    DB 27H, FEH, 20H, 28H, EDH, 2BH, 06H, 00H, FEH, 22H, 37H, 28H, 17H, CBH, C8H, FEH
    DB 41H, 38H, 11H, FEH, 5BH, 3FH, 38H, 0CH, CDH, D2H, F3H, D4H, 0BH, F4H, 79H, E6H
    DB 02H, F6H, 01H, 47H, DCH, 05H, F9H, 78H, D9H, FEH, FDH, C9H, 4FH, 06H, 01H, 18H
    DB F6H, CBH, 7CH, C8H, 2BH, 7DH, 2FH, 6FH, 7CH, 2FH, 67H, C9H, E5H, D5H, 11H, 00H
    DB 01H, CDH, 99H, FCH, D1H, E1H, C9H, 2AH, 26H, 17H, 19H, 38H, 12H, EBH, FDH, E5H
    DB E1H, EDH, 52H, 38H, 0AH, EDH, 62H, 39H, EDH, 5BH, 17H, 0BH, EDH, 52H, D0H, CFH
    DB 06H, AFH, 6FH, 67H, 3EH, 10H, 3DH, F8H, EDH, 6AH, CBH, 21H, CBH, 10H, 30H, F6H
    DB 19H, 18H, F3H, CDH, D1H, FCH, D0H, FEH, CAH, D0H, CDH, D5H, FCH, 18H, F7H, D9H
    DB E5H, D9H, E1H, 7EH, FEH, FEH, D0H, 23H, FEH, FDH, 20H, F7H, 7EH, 23H, FEH, 20H
    DB 28H, FAH, 2BH, 37H, C9H, 06H, 00H, FDH, E5H, E1H, 4EH, B9H, C8H, 0CH, 0DH, 37H
    DB C8H, 0DH, 20H, 02H, 23H, 4EH, 0CH, 09H, E5H, FDH, E1H, 18H, EDH, CDH, A7H, F0H
    DB 21H, 00H, 80H, E5H, CDH, 2BH, FAH, CDH, 93H, F4H, CDH, C3H, FAH, D1H, 19H, C9H
    DB 4FH, 7EH, B7H, 79H, C8H, 96H, 23H, 5EH, 23H, 20H, F6H, 57H, 19H, CDH, 43H, FCH
    DB CDH, ADH, DBH, 37H, C9H, CDH, 43H, FCH, 1EH, 01H, 28H, 08H, CDH, 45H, FDH, 5AH
    DB FEH, FDH, 20H, 0AH, CDH, 43H, FCH, 16H, FFH, FEH, 95H, C4H, 45H, FDH, 3EH, 95H
    DB C3H, 54H, FDH, CDH, C4H, FAH, 16H, 00H, 24H, C8H, 15H, 25H, C0H, 55H, C9H, CDH
    DB 43H, FCH, D9H, B8H, D9H, CAH, 43H, FCH, 3EH, 01H

; BASIC_ERROR - General BASIC error handler.
; usage: trace,call
BASIC_ERROR:
    DB FDH, 2AH, 1AH, 17H, FEH, F5H, CAH, A3H, FFH, F5H, FEH, 06H, CCH, FCH, DCH, CDH
    DB 35H, FCH, CDH, 18H, FCH, CDH, 79H, FEH

; KL: " " text prefix for BASIC error messages.
    DB 06H, 0DH, 0AH, 2AH, 2AH, 2AH, 20H, F1H, CBH, 7FH, 20H, 18H, 21H, C6H, FDH, 01H
    DB FFH, FFH, 03H, 09H, 4EH, 23H, 0CH, 0DH, 28H, 04H, B9H, 4EH, 20H, F4H, 23H, CDH
    DB DDH, FEH, 18H, 18H, 6FH, CDH, 79H, FEH, 0DH, 53H, 79H, 73H, 74H, 65H, 6DH, 20H
    DB 65H, 72H, 72H, 6FH, 72H, 20H, AFH, 67H, 47H, CDH, 1BH, FFH, CDH, 79H, FEH, 04H
    DB 2EH, 0BH, 0DH, 0AH, DDH, CBH, 00H, CEH, 2AH, 0CH, 17H, AFH, CDH, 2DH, DDH, C3H
    DB 0EH, E1H, 01H, 0FH, 4EH, 6FH, 74H, 20H, 75H, 6EH, 64H, 65H, 72H, 73H, 74H, 6FH
    DB 6FH, 64H, FFH, 02H, 06H, 4CH, 69H, 6EH, 65H, 93H, FFH, 03H, 04H, 41H, 92H, 93H
    DB FFH, 04H, 04H, 91H, 61H, 92H, FFH, 05H, 0BH, 91H, 73H, 75H, 62H, 73H, 63H, 72H
    DB 69H, 70H, 74H, FFH, 06H, 08H, 90H, 6DH, 65H, 6DH, 6FH, 72H, 79H, FFH, 07H, 03H
    DB 90H, FBH, FFH, 08H, 03H, 90H, F2H, FFH, 09H, 03H, 90H, F0H, FFH, 0AH, 03H, 8FH
    DB F8H, FFH, 0BH, 0DH, 8FH, 64H, 69H, 76H, 69H, 64H, 65H, 20H, 62H, 79H, 20H, 30H
    DB FFH, 0CH, 03H, 8FH, DBH, FFH, 0DH, 09H, 4FH, 76H, 65H, 72H, 66H, 6CH, 6FH, 77H
    DB FFH, 0EH, 0EH, 54H, 79H, 70H, 65H, 20H, 6DH, 69H, 73H, 6DH, 61H, 74H, 63H, 68H
    DB FFH, 0FH, 18H, 56H, 61H, 72H, 69H, 61H, 62H, 6CH, 65H, 20H, 64H, 65H, 63H, 6CH
    DB 61H, 72H, 65H, 64H, 20H, 74H, 77H, 69H, 63H, 65H, FFH, 10H, 06H, 91H, 66H, 69H
    DB 6CH, 65H, FFH, 00H, 0FH, 42H, 41H, 53H, 49H, 43H, 20H, 63H, 6FH, 72H, 72H, 75H
    DB 70H, 74H, 65H, 64H, FFH

; PRINT_INLINE_TEXT - Prints length-prefixed text stored immediately after the CALL.
; usage: trace,call
PRINT_INLINE_TEXT:
    DB E3H, CDH, 7FH, FEH, E3H, C9H

; PRINT_LENGTH_TEXT - Prints a length-prefixed text string.
; usage: call
PRINT_LENGTH_TEXT:
    DB 7EH, 23H, B7H, C8H, C5H, 47H, 7EH, 23H, CDH, 9AH, FEH, 10H, F9H, C1H, C9H, 3EH
    DB 0BH, CCH, 9AH, FEH, 3EH, 0DH, CDH, 9AH, FEH, 3EH, 0AH, F5H, C5H, D5H, 4FH, CDH
    DB A6H, FEH, D7H, D1H, C1H, F1H, C9H, 11H, 7FH, 00H, 3AH, 05H, 17H, F6H, 01H, B2H
    DB A3H, C3H, 1BH, 00H, CDH, 9AH, FEH, E1H, C3H, F0H, FFH, FDH, CBH, 08H, 7EH, CCH
    DB C7H, FEH, CDH, 0EH, F8H, CDH, 7FH, FEH, 3EH, 20H, 18H, CFH, 87H, FEH, 40H, 1FH
    DB CDH, 9AH, FEH, FEH, 22H, 20H, 09H, B9H, 28H, 02H, 41H, 0EH, 78H, 0EH, AFH, 4FH
    DB 7EH, 23H, 3CH, C8H, 3DH, F2H, CBH, FEH, E5H, 0CH, 0DH, F5H, FEH, FBH, 28H, 04H
    DB FEH, FCH, 38H, 02H, 0EH, FFH, FEH, FDH, 20H, 03H, F1H, 0CH, F5H, F1H, 20H, 12H
    DB 2FH, 21H, 6DH, DEH, CBH, 7EH, 23H, 28H, FBH, 3DH, 20H, F8H, 7EH, 23H, CBH, 7FH
    DB CBH, BFH, CDH, 9AH, FEH, 28H, F5H, E1H, 18H, C6H, 06H, FFH, 9FH, E6H, 20H, 4FH
    DB E5H, 21H, 47H, FFH, 5EH, 23H, 56H, 23H, E3H, AFH, EDH, 52H, 3CH, 30H, FBH, 19H
    DB 3DH, 28H, 07H, 0EH, 30H, 81H, CDH, 9AH, FEH, 79H, A9H, C4H, 9AH, FEH, E3H, 1DH

; KL: PRINT special-character dispatch table.
    DB 20H, E2H, E1H, 79H, A0H, 20H, 81H, C9H

; KL: Binary constants 1000, 100, 10, and 1 for decimal conversion.
    DB E8H, 03H, 64H, 00H, 0AH, 00H, 01H, 00H

LFF4F:
; BASIC_LINE_INPUT - Reads and stores an edited BASIC command or program line.
; usage: trace
BASIC_LINE_INPUT:
    LD HL,1831H
    PUSH HL
    LD B,00H

LFF55:
    PUSH BC
    LD DE,80FFH
    CALL FEA9H
    POP DE
    LD B,D
    JR Z,FF72H

LFF60:
    POP HL
    LD B,00H
    LD (HL),B
    INC HL
    LD (HL),FFH
    DEC HL
    CP F5H
    JR Z,FF9BH
    CP ECH
    JR Z,FF9BH
    OR A
    RST 10H

LFF72:
    LD A,(0B16H)
    OR A
    LD A,F5H
    JR NZ,FF60H
    LD A,C
    CP 0DH
    JR Z,FF92H
    CP 20H
    JR C,FF55H
    SUB 80H
    JR C,FF8CH
    CP 20H
    JR NC,FF8CH
    LD C,A

LFF8C:
    LD A,B
    CP FBH
    JR NC,FF55H
    INC B

LFF92:
    INC HL
    LD (HL),C
    JR NZ,FF55H
    LD (HL),FFH
    XOR A
    POP HL
    LD (HL),B

LFF9B:
    OR A
    RET

; CHECK_STOP_FLAG - Checks the Ctrl-Esc stop flag and stops BASIC when set.
; usage: trace,call
CHECK_STOP_FLAG:
    LD A,(0B16H)
    OR A
    RET Z
    EXX

; KL: STOP routine.
    EXX

LFFA4:
    XOR A
    LD (0B16H),A
    CALL FC35H
    CALL FC18H
    CALL FE79H

; KL: "STOP" message text.
    DB 06H, 0DH, 0AH, 53H, 54H, 4FH, 50H, DDH, CBH, 00H, 56H, 28H, 22H, 22H, 10H, 17H
    DB 2AH, 0CH, 17H, 22H, 0EH, 17H, CDH, 79H, FEH, 09H, 20H, 61H, 74H, 20H, 6CH, 69H
    DB 6EH, 65H, 20H, 2AH, 0CH, 17H, 23H, 5EH, 23H, 56H, EBH, B7H, CDH, 19H, FFH, AFH
    DB CDH, 8EH, FEH, C3H, DAH, DAH, 7CH, FEH, C0H
    RET C
    RES 6,H
    DB 3EH, 50H, C9H

; CALL_EXTENSION_HL - Pages EXTH into page 3 and SYS into page 0, then jumps to HL.
; usage: trace
CALL_EXTENSION_HL:
    DB F5H, 3EH, F0H, 32H, 03H, 00H, D3H, 02H, F1H, E9H, 79H, FEH, 0DH, 28H, 13H, FEH
