; -----------------------------------------------------------------------------
; VT-DOS 1.2 HBF disk ROM page 0
; Source: roms/VT-DOS12-DISK.ROM
; HBF ROM page: 0
; File offset: 0000H
; CPU window: C000H-CFFFH while this HBF ROM page is selected
; HBF ROM page 0 holds the MOPS entry contract: initializer at C00BH and counted function table at C00DH.
; ORG: C000H
; Size: 4096 bytes
; Comments: roms/VT-DOS12-DISK.page0.comments.json
; Data ranges: C000H-C008H, C009H-C00AH, C00BH-C00CH, C00DH-C00DH, C00EH-C00FH, C010H-C011H, C012H-C013H, C014H-C015H, C016H-C017H, C018H-C019H, C01AH-C01BH, C01CH-C01DH, C01EH-C01FH, C020H-C021H, C022H-C023H, C024H-C025H, C026H-C02DH, C168H-C174H, C21BH-C21BH, C31CH-C332H, C48EH-C498H, CC88H-CC95H, CEFCH-CFFFH
; Auto labels: branch and call targets are emitted as Lxxxx.
; -----------------------------------------------------------------------------

ORG C000H


; hbf: MOPS signature. C004H is the name length 04H and C005H-C008H is the card name DISK. The same nine bytes open every HBF ROM page.
    DB 4DH, 4FH, 50H, 53H, 04H, 44H, 49H, 53H, 4BH                                  ; |MOPS.DISK|

; hbf: Two constant bytes, 6FH DBH, present on every page and not part of the DISK name.
    DB 6FH, DBH                                                                     ; |o.|

; hbf: Little-endian initializer. EXT_INIT reads (C00BH) and calls it through FFF9H while this page is mapped. The target is C21CH.
    DB 1CH, C2H                                                                     ; |..|

; hbf: Counted function table. The count byte is 0CH, so function numbers 0 through 0BH are valid. Each entry is a little-endian address.
    DB 0CH                                                                          ; |.|

; hbf: Function 0 and the card IRQ vector. The target DBC1H is in HBF private RAM at D000H-DFFFH.
    DB C1H, DBH                                                                     ; |..|

; hbf: Function 1 at C499H.
    DB 99H, C4H                                                                     ; |..|

; hbf: Function 2 at C4FFH.
    DB FFH, C4H                                                                     ; |..|

; hbf: Function 3 at C35BH.
    DB 5BH, C3H                                                                     ; |[.|

; hbf: Function 4 at C53BH.
    DB 3BH, C5H                                                                     ; |;.|

; hbf: Function 5 at C5D5H.
    DB D5H, C5H                                                                     ; |..|

; hbf: Function 6 at C5F5H.
    DB F5H, C5H                                                                     ; |..|

; hbf: Function 7 at C5F9H.
    DB F9H, C5H                                                                     ; |..|

; hbf: Function 8 at C5FDH.
    DB FDH, C5H                                                                     ; |..|

; hbf: Function 9 at C6E8H.
    DB E8H, C6H                                                                     ; |..|

; hbf: Function 10 at C6FDH.
    DB FDH, C6H                                                                     ; |..|

; hbf: Function 11 at C703H.
    DB 03H, C7H                                                                     ; |..|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H                                       ; |........|
    JP CD46H
    CALL DB2EH
    LD C,C
    EX DE,HL
    RET

LC037:
    LD IY,D080H
    LD (DFF5H),IY
    LD HL,D067H
    LD DE,D4FAH
    LD B,04H

LC047:
    DEC B
    JR NZ,C04CH
    LD D,B
    LD E,B

LC04C:
    INC B
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    LD (HL),00H
    INC HL
    LD (HL),00H
    LD HL,020BH
    ADD HL,DE
    EX DE,HL
    DJNZ C047H
    OR A
    JR NZ,C06DH
    INC (IY-5FH)
    DEC (IY-5BH)
    LD B,A
    CALL DB2EH
    JP PO,AFD1H

LC06D:
    CALL C0C1H
    RET NZ
    LD BC,0201H
    CALL DB2EH
    AND A
    POP DE
    RET
    LD B,10H

LC07C:
    LD A,B
    CALL C095H
    LD E,(HL)
    INC HL
    LD D,(HL)
    LD A,D
    OR E
    JR Z,C091H
    LD HL,0009H
    ADD HL,DE
    LD A,(HL)
    OR A
    JR Z,C091H
    LD (HL),01H

LC091:
    DJNZ C07CH
    XOR A
    RET

LC095:
    CP 11H
    JR C,C09AH
    XOR A

LC09A:
    PUSH DE
    LD DE,D0DDH
    LD L,A
    LD H,00H
    ADD HL,DE
    LD A,(HL)
    PUSH AF
    CALL C0B3H
    POP AF
    POP DE
    RET
    CALL C0B3H
    LD E,(HL)
    INC HL
    LD D,(HL)
    LD A,D
    OR A
    RET

LC0B3:
    CP 11H
    JR C,C0B8H
    XOR A

LC0B8:
    LD E,A
    LD D,00H
    LD HL,D0BBH
    ADD HL,DE
    ADD HL,DE
    RET

LC0C1:
    PUSH AF
    LD B,04H
    LD DE,D0EEH
    LD HL,D0BDH

LC0CA:
    DI
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    PUSH HL
    EI
    LD HL,0006H
    ADD HL,DE
    LD A,04H
    SUB B
    LD (HL),A
    INC HL
    INC A
    LD (HL),A
    INC HL
    LD (HL),A
    INC HL
    LD (HL),00H
    LD DE,0016H
    ADD HL,DE
    LD (HL),FFH
    LD E,41H
    ADD HL,DE
    EX DE,HL
    POP HL
    DJNZ C0CAH
    LD HL,D0EEH
    POP AF
    SUB 01H
    SBC A,A
    INC A
    LD IX,D26EH

LC0FA:
    PUSH IX
    PUSH IY
    PUSH HL
    EXX
    EX AF,AF'
    PUSH HL
    PUSH DE
    PUSH BC
    PUSH AF
    EX AF,AF'
    EXX
    PUSH DE
    LD DE,0006H
    ADD HL,DE
    LD C,(HL)
    POP DE
    CALL C77FH
    EX AF,AF'
    CALL C123H
    POP AF
    EX AF,AF'
    EXX
    POP BC
    POP DE
    POP HL
    EXX
    POP HL
    POP IY
    POP IX
    OR A
    RET

LC123:
    LD A,(DFF4H)
    INC A
    RET NZ
    LD A,F5H
    EX AF,AF'
    CP A
    RET
    CALL DB2EH
    NOP
    NOP
    RET
    LD (DB1BH),A
    LD (DB1CH),DE
    EX (SP),HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    EX (SP),HL
    LD A,(DFFAH)
    PUSH AF
    LD A,D
    LD (DFFAH),A
    OUT (08H),A
    LD A,(DB1BH)
    PUSH HL
    LD HL,DB57H
    EX (SP),HL
    RES 4,D
    RES 5,D
    PUSH DE
    LD DE,(DB1CH)
    RET
    EX (SP),HL
    PUSH AF
    LD A,H
    LD (DFFAH),A
    OUT (08H),A
    POP AF
    POP HL
    RET
    DB 2EH                                                                          ; |.|

; hbf: Blank 8.3 name pad: two dots, spaces, and a NUL.
    DB 2EH, 20H, 20H, 20H, 20H, 20H, 20H, 20H, 20H, 20H, 20H, 00H, DDH              ; |.          ..|
    PUSH HL
    PUSH IY
    LD A,(DB1BH)
    LD HL,(DB1CH)
    PUSH AF
    PUSH HL
    LD A,(DFFFH)
    PUSH AF
    CALL DB2EH
    RET NZ
    XOR D3H
    RLCA
    POP AF
    LD (DFFFH),A
    OUT (02H),A
    POP HL
    POP AF
    LD (DB1CH),HL
    LD (DB1BH),A
    POP IY
    POP IX
    RET
    PUSH AF
    PUSH HL
    LD A,(DFFFH)
    LD (0003H),A
    LD HL,(DBA8H)
    LD (0039H),HL
    CALL 0000H
    DI
    LD HL,DB99H
    LD (0039H),HL
    LD A,(0003H)
    LD (DFFFH),A
    POP HL
    POP AF
    EI
    RET
    PUSH AF
    LD A,70H
    JP 003BH
    LD A,(DB1BH)
    LD HL,(DB1CH)
    PUSH AF
    PUSH HL
    CALL DB2EH
    LD E,L
    XOR E1H
    POP AF
    LD (DB1CH),HL
    LD (DB1BH),A
    RET
    PUSH AF
    PUSH BC
    PUSH DE
    PUSH HL
    PUSH IX
    PUSH IY
    EXX
    PUSH BC
    PUSH DE
    PUSH HL
    EX AF,AF'
    PUSH AF
    LD A,(0003H)
    PUSH AF
    LD A,F0H
    LD (0003H),A
    OUT (02H),A
    OUT (07H),A
    LD HL,(C00EH)
    CALL DC0EH
    POP AF
    LD (0003H),A
    OUT (02H),A
    POP AF
    EX AF,AF'
    POP HL
    POP DE
    POP BC
    EXX
    POP IY
    POP IX
    POP HL
    POP DE
    POP BC
    POP AF
    EI
    RET
    JP (HL)
    LD A,01H
    RET
    LD (BC),A
    LD B,H
    IN A,(5EH)

; hbf: Byte DBH sitting between the short routine at C214H and the initializer at C21CH.
    DB DBH                                                                          ; |.|

; hbf: Card initializer entered from the C00BH pointer. Disables interrupts and branches on the warm-reset flag at 0B21H.
    DI
    LD A,(0B21H)
    OR A
    JP NZ,C24CH
    LD HL,DB2EH
    LD DE,C133H
    LD B,06H

LC22C:
    LD A,(DE)
    CP (HL)
    INC HL
    INC DE
    JR NZ,C23AH
    DJNZ C22CH
    LD A,(DFF1H)
    OR A
    JR NZ,C24CH

LC23A:
    LD HL,D000H
    LD DE,D001H
    LD BC,0FF2H
    LD (HL),L
    LDIR
    LD A,FFH
    LD (DFF2H),A
    XOR A

LC24C:
    EX AF,AF'
    LD HL,DFF3H
    LD B,0DH
    XOR A

LC253:
    LD (HL),A
    INC HL
    DJNZ C253H
    LD (D020H),A
    PUSH IX
    POP HL
    LD A,L
    SUB 10H
    AND C0H
    LD (DFFEH),A
    RRCA
    RRCA
    ADD A,10H
    LD (D02BH),A
    LD DE,0008H
    ADD HL,DE
    EX DE,HL
    LD HL,C32CH
    LD BC,0006H
    LDIR
    LD (DB26H),DE
    LD C,21H
    LDIR
    LD HL,C12DH
    LD DE,DB28H
    LD BC,00EAH
    LDIR
    LD B,(HL)

LC28D:
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    EX DE,HL
    LD A,(D02BH)
    ADD A,(HL)
    LD (HL),A
    EX DE,HL
    DJNZ C28DH
    XOR A
    OUT (58H),A
    OUT (59H),A
    OUT (5AH),A
    OUT (5BH),A
    OUT (05H),A
    LD HL,C31CH
    LD B,10H
    LD C,71H

LC2AC:
    LD A,B
    DEC A
    OUT (70H),A
    OUTI
    JR NZ,C2ACH
    LD A,80H
    OUT (06H),A
    LD (DFFCH),A
    XOR A
    OUT (60H),A
    OUT (61H),A
    LD A,F0H
    LD (DFFFH),A
    LD A,C3H
    LD HL,DB99H
    LD (0038H),A
    LD (0039H),HL
    LD HL,DBD7H
    LD (DBA8H),HL
    EI
    EX AF,AF'
    CALL C037H
    DI
    LD A,F5H
    LD HL,703EH
    LD (0038H),A
    LD (0039H),HL
    LD HL,DBBBH
    LD (DBA8H),HL
    XOR A
    LD (DFF1H),A
    LD A,(D02BH)
    LD BC,087FH

LC2F7:
    INC C
    SLA B
    SUB 10H
    JR NZ,C2F7H
    LD A,B
    CPL
    LD HL,0B10H
    AND (HL)
    LD (HL),A
    LD A,C
    LD (0B05H),A
    LD (0B0DH),A
    OR 80H
    CALL C311H

LC311:
    CALL C6CCH
    XOR A
    LD (IX-33H),A
    LD (IX-01H),A
    RET

; hbf: Parameter block referenced by the initializer. The VT-DOS# text starts at C32CH.
    DB FFH, 0AH, 00H, 00H, 03H, 03H, 03H, 00H, 42H, 3CH, 02H, 4DH, 32H, 4BH, 40H, 63H ; |........B<.M2K@c|
    DB 56H, 54H, 2DH, 44H, 4FH, 53H, 23H                                            ; |VT-DOS#|
    LD DE,(1726H)
    LD A,(IY-01H)
    CP 40H
    JR NZ,C342H
    LD L,E
    LD H,D
    LD (HL),00H

LC342:
    PUSH IY
    EX (SP),HL
    DEC H
    SBC HL,DE
    LD C,L
    LD B,H
    RST 30H
    LD E,C
    RST 10H
    POP DE
    LD C,A
    RST 30H
    JP C,C9D7H
    DI
    LD A,E0H
    OUT (02H),A
    JP 0000H

; hbf: Function 3 entry.
    CALL C736H
    CALL C6CCH
    EX AF,AF'
    PUSH DE
    CALL C554H
    POP DE
    RET NZ
    PUSH IX
    POP BC
    LD HL,0030H
    ADD HL,BC
    PUSH HL
    EX DE,HL
    LD C,(HL)
    XOR A
    LD B,A
    CP C
    JR Z,C37AH
    INC HL
    LDIR

LC37A:
    LD (DE),A
    POP DE
    CALL C6D6H
    EX DE,HL
    XOR A
    PUSH IX
    POP DE
    LD (IX+06H),A
    LD (IX+07H),A
    DEC A
    LD (DE),A
    LD A,01H
    CALL C5CDH
    RET NZ
    LD A,C
    CALL C6DEH
    JR NZ,C39CH
    INC DE
    CALL C,C6D6H

LC39C:
    LD A,A9H
    RET NZ
    LD A,B
    EX AF,AF'
    LD A,06H
    JP P,C3C3H
    LD A,B
    AND 18H
    JR NZ,C3BCH
    PUSH IX
    EX (SP),HL
    LD DE,0008H
    ADD HL,DE
    EX DE,HL
    LD HL,C48EH
    LD BC,0008H
    LDIR
    POP HL

LC3BC:
    OR FFH
    LD (0B6BH),A
    LD A,04H

LC3C3:
    EX AF,AF'
    AND 10H
    JR NZ,C3D9H
    PUSH IX
    EX (SP),HL
    LD DE,0010H
    ADD HL,DE
    EX DE,HL
    LD HL,C496H
    LD BC,0003H
    LDIR
    POP HL

LC3D9:
    CALL C6AEH
    LD C,E
    LD B,D
    PUSH IX
    POP DE
    EX AF,AF'
    PUSH AF
    EX AF,AF'
    POP AF
    CALL C5CDH
    RET NZ
    LD (IX-01H),A
    LD (IX-03H),A
    BIT 2,(IX-20H)
    LD A,A8H
    RET NZ
    PUSH DE
    PUSH BC
    LD HL,0008H
    ADD HL,BC
    EX DE,HL
    LD HL,0BF5H
    LD C,00H
    LD B,08H
    CALL C482H
    LD A,(DE)
    CP 20H
    JR Z,C415H
    LD (HL),2EH
    INC HL
    INC C
    LD B,03H
    CALL C482H

LC415:
    LD A,C
    LD (0BF4H),A
    POP DE
    POP HL
    LD A,0CH
    CALL C5CDH
    CALL Z,C46CH
    JR NZ,C468H
    EX AF,AF'
    JP P,C463H
    LD BC,0080H
    CALL C6AEH
    LD A,01H
    LD (DFF7H),A
    LD A,0FH
    CALL C5CDH
    RET NZ
    LD A,(IX+00H)
    CP 11H
    JR Z,C445H
    DEC A
    LD A,E7H
    RET NZ

LC445:
    LD L,(IX+02H)
    LD H,(IX+03H)
    LD A,(IX+04H)
    CP 80H
    JR NZ,C454H
    INC HL
    XOR A

LC454:
    SRL H
    RR L
    RLA
    RRCA
    LD (IX-1AH),H
    LD (IX-1BH),L
    LD (IX-1CH),A

LC463:
    LD (IX-0BH),80H
    XOR A

LC468:
    LD DE,0BF4H
    RET

LC46C:
    PUSH IX
    POP HL
    LD DE,FFDDH
    ADD HL,DE
    LD DE,C496H
    LD B,03H

LC478:
    LD A,(DE)
    CP (HL)
    LD A,00H
    RET NZ
    INC HL
    INC DE
    DJNZ C478H
    RET

LC482:
    LD A,(DE)
    INC DE
    CP 20H
    JR Z,C48BH
    LD (HL),A
    INC HL
    INC C

LC48B:
    DJNZ C482H
    RET

; hbf: Default filename START.CAS as an 11-byte 8.3 name. The CAS extension begins at C496H.
    DB 53H, 54H, 41H, 52H, 54H, 20H, 20H, 20H, 43H, 41H, 53H                        ; |START   CAS|

; hbf: Function 1 entry.
    CALL C736H
    CALL C6B8H
    RET Z
    EX AF,AF'
    JP P,C4E1H

LC4A4:
    LD A,(IX-03H)
    OR A
    JR NZ,C4CCH
    CALL C5B8H
    RET NZ
    INC A
    LD (DFF7H),A
    CALL C6AEH
    LD BC,0100H
    LD A,0FH
    CALL C5CDH
    JR Z,C4C2H
    CP ECH
    RET NZ

LC4C2:
    LD A,B
    OR C
    LD A,ECH
    RET Z
    LD (IX-02H),00H
    LD A,C

LC4CC:
    DEC A
    LD (IX-03H),A
    CALL Z,C67BH
    LD C,(IX-02H)
    INC (IX-02H)
    XOR A
    LD B,A
    ADD IX,BC
    LD C,(IX+00H)
    RET

LC4E1:
    PUSH BC
    CALL C654H
    LD C,(IX-01H)
    PUSH IX
    POP HL
    XOR A
    LD B,A
    ADD HL,BC
    POP BC
    LD (HL),C
    INC (IX-01H)
    PUSH AF
    CALL C67BH
    POP AF
    RET NZ
    LD BC,0100H
    JP C5C0H

; hbf: Function 2 entry.
    EXX
    CALL C736H
    CALL C6B8H
    RET Z
    CALL C5B8H
    RET NZ
    CALL C654H
    EXX
    PUSH DE
    PUSH BC
    PUSH DE
    CALL C6AEH
    POP IX
    EX AF,AF'
    ADD A,80H
    SBC A,A
    ADD A,10H
    CALL CD46H
    POP HL
    JR Z,C52AH
    CP ECH
    JR Z,C52AH
    POP DE
    JR C52FH

LC52A:
    SBC HL,BC
    EX (SP),IX
    POP DE

LC52F:
    LD B,H
    LD C,L
    PUSH DE
    PUSH BC
    PUSH AF
    CALL C67BH
    POP AF
    POP BC
    POP DE
    RET

; hbf: Function 4 entry.
    CALL C736H
    CALL C6CCH
    CALL C554H
    LD (IX-33H),00H
    EX AF,AF'
    LD BC,FFFFH
    LD A,0EH
    CALL C5CDH
    RET NZ
    EX AF,AF'
    RET

LC554:
    PUSH IX
    LD IX,DC45H
    CALL C5B8H
    CALL Z,C56DH
    LD IX,DD78H
    CALL Z,C5B8H
    CALL Z,C56DH
    POP IX
    RET

LC56D:
    LD A,(IX-33H)
    OR A
    RET Z
    XOR A
    BIT 6,(IX-13H)
    RET Z
    CALL C46CH
    JR NZ,C5B1H
    PUSH IX
    POP HL
    LD (HL),11H
    INC HL
    LD (HL),A
    INC HL
    LD A,(IX-1CH)
    RLCA
    SRL A
    LD (IX+04H),A
    LD A,(IX-1BH)
    ADC A,A
    LD (HL),A
    INC HL
    LD A,(IX-1AH)
    ADC A,A
    LD (HL),A
    INC HL
    INC HL
    LD B,7BH
    XOR A

LC59E:
    LD (HL),A
    INC HL
    DJNZ C59EH
    LD DE,FF75H
    ADD HL,DE
    LD B,04H

LC5A8:
    LD (HL),A
    INC HL
    DJNZ C5A8H
    LD C,80H
    CALL C5C0H

LC5B1:
    CALL C6AEH
    LD A,0DH
    JR C5CDH

LC5B8:
    LD A,(IX-01H)
    OR A
    RET Z
    LD C,A
    LD B,00H

LC5C0:
    CALL C6AEH
    XOR A
    LD (IX-01H),A
    INC A
    LD (DFF7H),A
    LD A,10H

LC5CD:
    PUSH IX
    CALL CD46H
    POP IX
    RET

; hbf: Function 5 entry.
    CALL C736H
    CALL C6B8H
    RET Z

LC5DC:
    PUSH DE
    PUSH BC
    PUSH IX
    CALL C4A4H
    POP IX
    LD L,C
    POP BC
    POP DE
    RET NZ
    LD A,(DE)
    CP L
    LD A,E8H
    RET NZ
    INC DE
    DEC BC
    LD A,B
    OR C
    JR NZ,C5DCH
    RET

; hbf: Function 6 entry.
    LD A,02H
    JR C5FFH

; hbf: Function 7 entry.
    LD A,01H
    JR C5FFH

; hbf: Function 8 entry.
    LD A,00H

LC5FF:
    PUSH AF
    EX AF,AF'
    POP AF
    EXX
    CALL C736H
    CALL C6B8H
    RET Z
    CALL C5B8H
    RET NZ
    CALL C654H
    EXX
    PUSH BC
    EX AF,AF'
    SUB 01H
    LD A,B
    LD BC,FFE7H
    JR C,C628H
    LD BC,FFF8H
    JR Z,C628H
    SBC HL,HL
    LD C,L
    LD B,H
    ADD A,A
    JR C635H

LC628:
    PUSH IX
    POP HL
    ADD HL,BC
    LD B,(HL)
    DEC HL
    LD C,(HL)
    DEC HL
    ADD A,A
    LD A,(HL)
    DEC HL
    LD L,(HL)
    LD H,A

LC635:
    SBC A,A
    ADD HL,DE
    EX DE,HL
    POP HL
    ADC HL,BC
    LD C,L
    LD B,H
    LD L,A
    SBC A,A
    XOR L
    AND E5H
    RET NZ
    LD (IX-0BH),E
    LD (IX-0AH),D
    LD (IX-09H),C
    LD (IX-08H),B
    CALL C67BH
    XOR A
    RET

LC654:
    LD A,(IX-03H)
    OR A
    RET Z
    PUSH IX
    POP HL
    LD DE,FFF5H
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    EX DE,HL
    LD C,A
    XOR A
    LD B,A
    LD (IX-03H),A
    SBC HL,BC
    EX DE,HL
    LD (HL),D
    DEC HL
    LD (HL),E
    RET NC
    INC HL
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    DEC DE
    LD (HL),D
    DEC HL
    LD (HL),E
    RET

LC67B:
    PUSH IX
    POP HL
    LD DE,FFE7H
    ADD HL,DE
    PUSH HL
    LD DE,000EH
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    LD C,(HL)
    INC HL
    LD B,(HL)
    LD L,(IX-01H)
    LD H,00H
    ADD HL,DE
    EX DE,HL
    JR NC,C698H
    INC BC

LC698:
    POP HL
    LD A,B
    CP (HL)
    RET C
    DEC HL
    LD A,C
    CP (HL)
    RET C
    DEC HL
    LD A,D
    CP (HL)
    RET C
    DEC HL
    LD A,E
    CP (HL)
    RET C

LC6A8:
    LD HL,0B6EH
    LD (HL),FFH
    RET

LC6AE:
    PUSH IX
    EX (SP),HL
    LD DE,FFCDH
    ADD HL,DE
    EX DE,HL
    POP HL
    RET

LC6B8:
    LD L,A
    EX AF,AF'
    LD A,L
    OR A
    CALL C6C1H
    RET NZ
    XOR L

LC6C1:
    CALL C6CCH
    LD A,(IX-33H)
    OR A
    RET NZ
    LD A,E9H
    RET

LC6CC:
    LD IX,DC45H
    RET M
    LD IX,DD78H
    RET

LC6D6:
    LD A,(DE)
    CALL C6DEH
    RET NC
    INC DE
    JR C6D6H

LC6DE:
    CP 20H
    SCF
    RET Z
    CP 09H
    SCF
    RET Z
    OR A
    RET

; hbf: Function 9 entry.
    LD L,E
    LD H,D
    ADD HL,BC
    JR C,C6F0H
    LD A,BFH
    CP H

LC6F0:
    LD A,FCH

LC6F2:
    RET C
    LD (0BF0H),DE
    LD (0BF2H),BC
    XOR A
    RET

; hbf: Function 10 entry.
    CALL DB2EH
    DJNZ C6F2H
    RET

; hbf: Function 11 entry.
    CALL C741H
    LD A,C
    EX DE,HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    PUSH DE
    POP IX
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    PUSH DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    LD C,(HL)
    INC HL
    LD B,(HL)
    EX (SP),HL
    CALL CD46H
    EX (SP),HL
    LD (HL),B
    DEC HL
    LD (HL),C
    DEC HL
    LD (HL),D
    DEC HL
    LD (HL),E
    DEC HL
    POP DE
    LD (HL),D
    DEC HL
    LD (HL),E
    DEC HL
    PUSH IX
    POP DE
    LD (HL),D
    DEC HL
    LD (HL),E
    EX DE,HL
    JR C767H

LC736:
    LD HL,C759H
    EX (SP),HL
    PUSH HL
    PUSH AF
    XOR A
    LD (0B6EH),A
    POP AF

LC741:
    DI
    PUSH AF
    PUSH HL
    LD A,(0003H)
    LD (DFFFH),A
    LD A,C3H
    LD HL,DB99H
    LD (0038H),A
    LD (0039H),HL
    POP HL
    POP AF
    EI
    RET
    PUSH AF
    CP ECH
    CALL Z,C6A8H
    SUB F5H
    JR NZ,C766H
    LD (0B16H),A

LC766:
    POP AF

LC767:
    DI
    PUSH AF
    PUSH HL
    LD A,(DFFFH)
    LD (0003H),A
    LD A,F5H
    LD HL,703EH
    LD (0038H),A
    LD (0039H),HL
    EI
    POP HL
    POP AF
    RET

LC77F:
    LD (IY-48H),A
    SUB 02H
    JR C,C797H
    JR Z,C7C7H
    DEC A
    JR Z,C7E8H
    SUB 04H
    JP C,C9E9H
    DEC A
    JP Z,CBDEH
    LD A,BFH
    RET

LC797:
    EX AF,AF'
    XOR A
    LD B,0FH
    CALL DB2EH
    LD C,C
    EX DE,HL
    LD C,08H

LC7A2:
    LD B,C
    LD A,07H
    CALL DB2EH
    LD C,C
    EX DE,HL
    RRC C
    JR NC,C7A2H
    LD BC,0401H

LC7B1:
    LD (IY-41H),C
    RLC C
    INC IY
    DJNZ C7B1H
    EX AF,AF'
    CP FFH
    RET Z
    LD A,08H
    LD H,B
    CALL DB2EH
    LD C,C
    EX DE,HL
    RET

LC7C7:
    CALL CA79H
    BIT 3,(IX-3DH)
    JR Z,C7E4H
    LD A,07H
    CALL DB2EH
    LD C,C
    EX DE,HL
    JR NZ,C7E4H
    LD A,B
    OR A
    RET Z
    XOR A
    CP (IY-5CH)
    LD B,A
    RET Z
    INC B
    RET

LC7E4:
    LD B,FFH
    XOR A
    RET

LC7E8:
    CALL CA79H
    RES 3,(IX-3DH)
    XOR A
    CALL DB2EH
    LD C,C
    EX DE,HL
    LD DE,0001H
    LD C,D
    LD L,E
    EXX
    PUSH HL
    EXX
    CALL CB26H
    POP HL
    JR NZ,C836H
    PUSH HL
    PUSH BC
    CALL C858H
    POP BC
    JR NZ,C80FH
    POP BC
    JP C924H

LC80F:
    POP HL
    PUSH HL
    PUSH HL
    EXX
    POP HL
    EXX
    LD DE,0002H
    LD C,D
    LD L,01H
    CALL CB26H
    POP HL
    JR NZ,C84BH
    LD C,(HL)
    INC HL
    LD A,(HL)
    INC HL
    AND (HL)
    INC A
    JR NZ,C855H
    LD A,C
    CP F8H
    JR C,C855H
    DEC HL
    DEC HL
    CALL C8C1H
    JP C924H

LC836:
    AND 21H
    JR NZ,C84DH
    CALL CA95H
    AND 21H
    JR NZ,C84DH
    BIT 4,D
    JR Z,C855H
    LD DE,FF20H
    CALL CA99H

LC84B:
    AND 21H

LC84D:
    JP NZ,C9CEH
    BIT 4,D
    LD A,B6H
    RET NZ

LC855:
    LD A,B5H
    RET

LC858:
    LD DE,000BH
    ADD HL,DE
    PUSH HL
    CALL C898H
    JR NZ,C874H
    DEC A
    CP 0AH
    JR NC,C874H
    LD D,(HL)
    INC HL
    LD A,(HL)
    INC HL
    DEC A
    SRL A
    OR (HL)
    OR D
    JR NZ,C874H
    POP HL
    RET

LC874:
    POP HL
    DEC HL
    DEC HL
    DEC HL
    LD A,(HL)
    DEC A
    RET NZ
    LD DE,0048H
    ADD HL,DE
    CALL C898H
    RET NZ
    CP 03H
    JR NC,C896H
    EX DE,HL
    LD HL,FFBBH
    ADD HL,DE
    EX DE,HL
    LD BC,000FH
    LDDR
    INC HL
    CP A
    SCF
    RET

LC896:
    OR H
    RET

LC898:
    LD D,(HL)
    INC HL
    LD A,(HL)
    INC HL
    SUB 02H
    OR D
    RET NZ
    OR (HL)
    JR Z,C896H
    NEG
    AND (HL)
    CP (HL)
    INC HL
    RET NZ
    INC HL
    INC HL
    LD A,(HL)
    DEC A
    CP 07H
    JR NC,C896H

LC8B1:
    LD E,06H
    ADD HL,DE
    LD A,(HL)
    INC HL
    DEC A
    CP 0CH
    JR NC,C896H
    LD A,(HL)
    INC HL
    OR A
    LD A,(HL)
    INC HL
    RET

LC8C1:
    LD DE,0064H
    ADD HL,DE
    LD B,59H

LC8C7:
    DEC HL
    LD (HL),D
    DJNZ C8C7H
    PUSH HL
    LD B,02H
    INC HL
    LD (HL),B
    INC HL
    INC (HL)
    AND 05H
    SUB 04H
    JR Z,C8D9H
    INC (HL)

LC8D9:
    INC HL
    INC (HL)
    INC HL
    INC HL
    LD (HL),B
    INC HL
    LD (HL),40H
    OR A
    JR Z,C8E6H
    LD (HL),70H

LC8E6:
    INC HL
    INC HL
    PUSH HL
    LD A,C
    LD HL,0140H
    LD BC,0028H
    LD DE,0801H
    BIT 1,A
    JR NZ,C8F9H
    INC D
    ADD HL,BC

LC8F9:
    BIT 0,A
    JR Z,C8FFH
    INC E
    ADD HL,HL

LC8FF:
    BIT 2,A
    JR NZ,C904H
    ADD HL,HL

LC904:
    EX DE,HL
    EX (SP),HL
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    LD (HL),A
    INC HL
    INC (HL)
    CP FEH
    JR NC,C91AH
    SUB FAH
    JR Z,C91AH
    INC (HL)
    INC A
    JR NZ,C91AH
    INC (HL)

LC91A:
    INC HL
    INC HL
    POP DE
    LD (HL),D
    INC HL
    INC HL
    LD (HL),E
    POP HL
    OR A
    RET

LC924:
    EX AF,AF'
    LD BC,0008H
    ADD HL,BC
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    LD (IX-2DH),E
    LD (IX-29H),D
    LD B,(HL)
    INC HL
    INC HL
    INC HL
    LD C,(HL)
    INC HL
    EX AF,AF'
    JR C,C942H
    INC HL
    LD A,(HL)
    INC HL
    INC HL
    JR C94AH

LC942:
    CP 02H
    JR Z,C948H
    LD A,01H

LC948:
    LD C,09H

LC94A:
    LD (IX-39H),C
    EXX
    CP (IX-31H)
    JR Z,C97FH
    CP 01H
    JR Z,C97FH
    LD H,A

LC958:
    LD DE,FF00H
    LD C,01H
    LD B,(IX-41H)
    CALL CA9DH
    JR Z,C977H
    AND 21H
    JR NZ,C9BFH
    BIT 4,D
    JR NZ,C970H
    DEC H
    JR NZ,C958H

LC970:
    LD A,B1H
    DB 21H                                                                          ; |!|

LC973:
    LD A,B5H

LC975:
    OR A
    RET

LC977:
    LD A,(IY-46H)
    DEC A
    JR NZ,C970H
    LD A,02H

LC97F:
    LD (IX-31H),A
    EXX
    LD A,(HL)
    INC HL
    LD H,(HL)
    LD (IX-25H),A
    LD (IX-21H),H
    DEC DE
    CALL CA47H
    JR C,C973H
    LD A,D
    OR A
    RET NZ
    LD A,E
    DEC A
    CP FEH
    JR NC,C973H
    LD (IX-35H),E
    LD A,B
    CP (IX-1DH)
    JR Z,C9C8H
    LD A,E
    CP 33H
    LD A,(IX-3DH)
    RES 2,A
    JR C,C9C2H
    SET 2,A
    BIT 1,A
    JR NZ,C9C2H
    LD C,A
    PUSH BC
    LD B,(IX-41H)
    CALL CB85H
    POP BC
    JR C,C975H

LC9BF:
    JR NZ,C9CEH
    LD A,C

LC9C2:
    LD (IX-3DH),A
    LD (IX-1DH),B

LC9C8:
    SET 3,(IX-3DH)
    XOR A
    RET

LC9CE:
    LD HL,C9E0H

LC9D1:
    INC HL
    RRCA
    JR NC,C9D1H
    LD A,(HL)
    EXX

LC9D7:
    CP AFH
    JR NZ,C9DFH
    RES 3,(IX-3DH)

LC9DF:
    OR A
    RET
    CP H
    CP D
    CP C
    CP C
    CP B
    XOR A
    OR A
    CP A

LC9E9:
    CALL CA79H
    EXX
    LD B,00H
    LD C,A
    EXX
    BIT 3,(IX-3DH)
    JR Z,CA1CH
    LD L,A
    OR A
    JR Z,CA1EH
    XOR A
    BIT 0,(IX-3DH)
    CALL Z,CB85H
    JR C,CA1EH
    OR A
    JR NZ,C9CEH
    LD H,A
    DEC L
    ADD HL,DE
    JR C,CA19H
    PUSH BC
    LD C,(IX-2DH)
    LD B,(IX-29H)
    SBC HL,BC
    POP BC
    JR C,CA20H

LCA19:
    LD A,BDH
    DB 21H                                                                          ; |!|

LCA1C:
    LD A,AFH

LCA1E:
    EXX
    RET

LCA20:
    CALL CA47H
    LD D,E
    LD E,A

LCA25:
    LD A,(IX-39H)
    SUB E
    INC A
    EXX
    CP C
    JR C,CA2FH
    LD A,C

LCA2F:
    EXX
    LD L,A
    CALL CB26H
    JR NZ,C9CEH
    EXX
    OR C
    RET Z
    EXX
    LD E,01H
    INC C
    LD A,(IX-31H)
    SUB C
    JR NZ,CA45H
    LD C,A
    INC D

LCA45:
    JR CA25H

LCA47:
    XOR A
    LD L,(IX-25H)
    LD H,(IX-21H)
    ADD HL,DE
    EX DE,HL
    RET C
    LD C,(IX-39H)
    CALL CA67H
    LD C,(IX-31H)
    OR A
    DEC C
    JR Z,CA63H
    DEC C
    SRA D
    RR E

LCA63:
    RL C
    INC A
    RET

LCA67:
    PUSH BC
    LD B,10H
    XOR A

LCA6B:
    SLA E
    RL D
    RLA
    CP C
    JR C,CA75H
    SUB C
    INC E

LCA75:
    DJNZ CA6BH
    POP BC
    RET

LCA79:
    LD A,C
    CP 04H
    JR NC,CA8FH
    EXX
    LD D,00H
    LD E,A
    PUSH IY
    EX (SP),IX
    ADD IX,DE
    POP HL
    EXX
    LD A,B
    LD B,(IX-41H)
    RET

LCA8F:
    POP HL
    LD A,BEH
    LD B,00H
    RET

LCA95:
    LD D,FFH

LCA97:
    LD E,00H

LCA99:
    PUSH BC
    LD C,00H
    DB 3EH                                                                          ; |>|

LCA9D:
    PUSH BC
    PUSH HL
    PUSH IX
    LD IX,D039H
    LD A,(IY-5EH)
    AND 03H
    OR E
    LD H,A
    LD L,01H
    LD A,06H
    CALL DB2EH
    LD C,C
    EX DE,HL
    POP IX
    POP HL
    POP BC
    LD D,A
    RET

LCABB:
    PUSH HL
    LD A,(IY-5EH)
    AND 03H
    LD H,A
    LD A,(IX-3DH)
    AND 06H
    CP 02H
    JR NZ,CACDH
    SET 3,H

LCACD:
    LD A,(IX-35H)
    SRA A
    CP D
    JR NC,CAD7H
    SET 2,H

LCAD7:
    EXX
    PUSH HL
    EX (SP),IX
    EXX
    LD A,(IY-48H)
    CP 08H
    JR NZ,CAE6H
    RRCA
    JR CAEDH

LCAE6:
    CP 05H
    LD A,01H
    JR C,CAEDH
    INC A

LCAED:
    CALL DB2EH
    LD C,C
    EX DE,HL
    EX AF,AF'
    LD A,(IY-48H)
    SUB 06H
    JR NZ,CB07H
    OR L
    JR Z,CB07H
    LD A,03H
    CALL DB2EH
    LD C,C
    EX DE,HL
    JR Z,CB07H
    EX AF,AF'

LCB07:
    POP IX
    CP 08H
    LD A,E
    ADD A,L
    LD E,A
    LD A,L
    EXX
    LD E,A
    ADD A,H
    ADD A,E
    LD H,A
    LD A,B
    ADD A,E
    LD B,A
    LD A,C
    SUB E
    LD C,A
    EXX
    LD A,L
    POP HL
    ADD A,H
    LD H,A
    LD A,L
    EXX
    SUB E
    EXX
    LD L,A
    EX AF,AF'
    RET

LCB26:
    EXX
    LD D,04H
    EXX
    LD H,00H

LCB2C:
    CALL CABBH
    OR A
    JR Z,CB82H
    PUSH HL
    LD L,A
    AND 61H
    JR NZ,CB62H
    LD A,L
    AND 0EH
    JR NZ,CB65H
    EXX
    LD A,D
    EXX
    CP 04H
    LD A,L
    JR NZ,CB68H
    PUSH DE
    LD DE,FF00H
    CALL CA9DH
    POP DE
    JR Z,CB55H
    AND 20H
    JR NZ,CB62H
    JR CB5BH

LCB55:
    LD A,(IY-47H)
    CP D
    JR Z,CB5FH

LCB5B:
    CALL CB85H
    OR A

LCB5F:
    LD A,L
    JR Z,CB68H

LCB62:
    LD H,05H
    DB CAH                                                                          ; |.|

LCB65:
    LD H,01H
    DB CAH                                                                          ; |.|

LCB68:
    LD H,02H
    PUSH AF
    EXX
    LD A,D
    PUSH AF
    EXX
    SUB 04H
    JR Z,CB78H
    CALL DB2EH
    LD C,C
    EX DE,HL

LCB78:
    POP AF
    SUB H
    EXX
    LD D,A
    EXX
    POP HL
    LD A,H
    POP HL
    JR NC,CB2CH

LCB82:
    LD L,H
    OR A
    RET

LCB85:
    PUSH BC
    PUSH DE
    LD C,02H
    RES 0,(IX-3DH)
    RES 1,(IX-3DH)

LCB91:
    XOR A
    CALL DB2EH
    LD C,C
    EX DE,HL
    LD D,08H
    CALL CA97H
    JR Z,CBA9H
    AND 21H
    JR NZ,CBD3H

LCBA2:
    DEC C
    JR NZ,CB91H
    LD A,D
    OR A
    JR CBD3H

LCBA9:
    LD E,(IX-3DH)
    SET 1,E
    LD A,(IY-47H)
    CP 04H
    JR Z,CBCBH
    CP 08H
    JR Z,CBC4H
    CP 10H
    LD D,10H
    JR NZ,CBA2H
    LD A,B1H
    SCF
    JR CBD3H

LCBC4:
    BIT 2,E
    JR NZ,CBCDH
    RES 1,E
    DB C2H                                                                          ; |.|

LCBCB:
    RES 2,E

LCBCD:
    SET 0,E
    LD (IX-3DH),E
    XOR A

LCBD3:
    PUSH AF
    XOR A
    CALL DB2EH
    LD C,C
    EX DE,HL
    POP AF
    POP DE
    POP BC
    RET

LCBDE:
    CALL CA79H
    LD C,A
    CP F8H
    LD A,B5H
    RET C
    LD H,(IY-5EH)
    LD A,08H
    CALL DB2EH
    LD C,C
    EX DE,HL
    JP NZ,C9CEH
    LD D,03H
    INC L
    JR Z,CBFDH
    RES 1,D
    SET 2,C

LCBFD:
    BIT 2,C
    JR NZ,CC03H
    SET 2,D

LCC03:
    LD (IX-3DH),D
    LD D,A
    CALL CA97H
    AND A1H
    LD A,BCH
    RET NZ
    EXX
    PUSH HL
    LD DE,CD1CH
    LD B,05H
    CALL CD0EH
    LD C,09H
    LD A,01H
    EX AF,AF'

LCC1E:
    LD DE,CD26H
    LD B,04H
    CALL CD0EH
    EX AF,AF'
    LD (HL),A
    INC HL
    INC A
    EX AF,AF'
    LD B,0AH
    CALL CD0EH
    DEC C
    JR NZ,CC1EH
    LD B,02H
    CALL CD0EH
    POP HL
    EXX

LCC3A:
    CALL CC96H
    LD (IY-48H),A
    CALL CC96H
    LD A,C
    EXX
    PUSH HL
    LD C,A
    CALL C8C1H
    EX DE,HL
    LD HL,CC8FH
    LD BC,000BH
    DEC DE
    LDDR
    LD C,1FH
    EX DE,HL
    ADD HL,BC
    LD (HL),C9H
    LD C,22H
    ADD HL,BC
    EX DE,HL
    LD HL,CC90H
    LD C,06H
    LDIR
    INC DE
    PUSH IY
    POP HL
    LD BC,FFA7H
    ADD HL,BC
    LD BC,0004H
    LDIR
    POP HL
    EXX
    PUSH BC
    LD DE,0001H
    LD C,D
    LD L,E
    LD (IY-48H),05H
    CALL CB26H
    POP BC
    JR NZ,CCD0H
    RET
    EX DE,HL
    CP 90H

; hbf: Volume identity text VTDOS1.2VOL_ID.
    DB 56H, 54H, 44H, 4FH, 53H, 31H, 2EH, 32H, 56H, 4FH, 4CH, 5FH, 49H, 44H         ; |VTDOS1.2VOL_ID|

LCC96:
    POP HL
    LD E,00H

LCC99:
    LD A,C
    AND 01H
    LD D,A

LCC9D:
    PUSH HL
    PUSH DE
    EXX
    POP BC
    PUSH HL
    BIT 3,(IY-48H)
    JR Z,CCB7H
    LD A,09H
    LD DE,0098H

LCCAD:
    ADD HL,DE
    LD (HL),C
    INC HL
    LD (HL),B
    LD DE,028FH
    DEC A
    JR NZ,CCADH

LCCB7:
    POP HL
    PUSH HL
    EXX
    PUSH BC
    PUSH DE
    LD L,09H
    BIT 1,C
    JR Z,CCC3H
    DEC L

LCCC3:
    LD C,D
    LD D,E
    LD E,01H
    CALL CB26H
    POP DE
    POP BC
    EXX
    POP HL
    EXX
    POP HL

LCCD0:
    JP NZ,C9CEH
    CALL C123H
    RET Z
    DEC D
    JP P,CC9DH
    BIT 0,C
    JR Z,CD02H
    BIT 3,(IY-48H)
    JR Z,CD02H
    LD A,E
    OR E
    JR NZ,CD02H
    PUSH DE
    PUSH BC
    LD C,01H
    CALL CA9DH
    POP BC
    POP DE
    JR Z,CCFCH
    AND 21H
    JR NZ,CCD0H

LCCF8:
    DEC C
    JP CC3AH

LCCFC:
    LD A,(IY-46H)
    DEC A
    JR NZ,CCF8H

LCD02:
    LD A,28H
    BIT 2,C
    JR NZ,CD09H
    ADD A,A

LCD09:
    INC E
    SUB E
    JR NZ,CC99H
    JP (HL)

LCD0E:
    PUSH BC
    LD A,(DE)
    LD B,A
    INC DE
    LD A,(DE)
    INC DE

LCD14:
    LD (HL),A
    INC HL
    DJNZ CD14H
    POP BC
    DJNZ CD0EH
    RET
    INC A
    LD C,(HL)
    INC C
    NOP
    INC BC
    OR 01H
    CALL M,4E3CH
    INC C
    NOP
    INC BC
    PUSH AF
    LD BC,02FEH
    NOP
    LD BC,0102H
    RST 30H
    LD D,4EH
    INC C
    NOP
    INC BC
    PUSH AF
    LD BC,00FBH
    PUSH HL
    NOP
    PUSH HL
    LD BC,54F7H
    LD C,(HL)
    NOP
    LD C,(HL)
    DB CCH, 4EH                                                                     ; |.N|

LCD46:
    CP 20H
    JR NC,CD98H
    CP 0FH
    JR NC,CD51H
    PUSH DE
    POP IX

LCD51:
    PUSH IY
    EXX
    PUSH HL
    PUSH DE
    PUSH BC
    ADD A,A
    LD C,A
    LD B,00H
    EX AF,AF'
    PUSH AF
    LD IY,D080H
    LD (IY+07H),B
    LD (D06BH),SP
    LD HL,CD9CH
    ADD HL,BC
    LD E,(HL)
    INC HL
    LD D,(HL)
    LD (DB2BH),DE
    EXX
    XOR A
    CALL DB28H
    DB EDH                                                                          ; |.|

LCD79:
    LD A,E
    LD L,E
    RET NC
    EX AF,AF'
    CALL C123H
    JR NZ,CD86H

LCD82:
    XOR A
    LD (DFF4H),A

LCD86:
    XOR A
    LD (DFF7H),A
    LD (DFF3H),A
    POP AF
    EX AF,AF'
    EXX
    POP BC
    POP DE
    POP HL
    EXX
    POP IY
    OR A
    RET
    LD A,AEH
    OR A
    RET
    SBC A,B
    CALL E010H
    JR C,CD82H
    LD H,B
    RET PO
    XOR B
    RET PO
    OR A
    RET PO
    ADD A,C
    RET PO
    LD A,B
    RET PO
    LD A,A
    EX (SP),HL
    ADD HL,HL
    CALL PO,E492H
    LD H,A
    PUSH HL
    LD D,D
    AND A0H
    PUSH HL
    LD L,H
    ADC A,B9H
    AND BAH
    AND B8H
    AND 73H
    AND 74H
    AND CCH
    ADC A,91H
    ADC A,08H
    JP NC,CDE5H
    AND A
    POP DE
    JP PO,E7D1H
    ADC A,37H
    RST 28H
    LD (HL),B
    RST 28H
    AND C
    RST 28H
    XOR H
    RST 28H
    POP HL
    CALL 6D3AH
    RET NC
    OR A
    JP (HL)
    LD BC,0102H
    RET

LCDE5:
    CALL CE53H
    RET NZ
    PUSH BC
    LD A,08H
    CALL C0FAH
    POP BC
    JR Z,CDF8H
    CP BFH
    RET NZ
    LD A,ABH
    RET

LCDF8:
    CALL DB2EH
    OR A
    RET NC
    OR A
    RET NZ
    PUSH HL
    PUSH HL
    LD DE,001DH
    ADD HL,DE
    LD C,(HL)
    LD B,0FH
    LD H,C
    EX (SP),HL
    LD E,D
    CALL DB2EH
    LD (DE),A
    PUSH DE
    INC DE
    LD BC,FFFFH
    CALL DB2EH
    LD (DE),A
    PUSH DE

LCE19:
    INC DE
    LD BC,0000H
    CALL DB2EH
    LD (DE),A
    PUSH DE
    PUSH HL
    LD BC,0016H
    ADD HL,BC
    LD A,(HL)
    INC HL
    LD H,(HL)
    LD L,A
    DEC HL
    SBC HL,DE
    POP HL
    DB 20H                                                                          ; | |

LCE30:
    RET PE
    LD BC,0012H
    ADD HL,BC
    LD C,(HL)
    INC HL
    LD B,(HL)
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    LD H,D
    LD L,E
    SBC HL,BC
    POP AF
    CP FEH
    LD A,L
    POP HL
    DEC DE
    JR NC,CE4DH
    CALL DB2EH
    DJNZ CE30H

LCE4D:
    CALL DB2EH
    LD E,L
    DB D4H, C9H                                                                     ; |..|

LCE53:
    PUSH DE
    PUSH BC
    LD B,02H
    CALL DB2EH
    CP L
    RET NC
    POP BC
    POP IX
    OR A
    RET NZ
    CALL DB2EH
    LD L,L
    CALL NC,2ECDH
    IN A,(D4H)
    POP DE
    RET
    LD A,B
    INC B
    JR Z,CE7FH
    DEC B
    JR NZ,CE76H
    LD A,(D021H)

LCE76:
    CALL C095H
    OR A
    JR NZ,CE7FH
    LD A,ABH
    RET

LCE7F:
    LD B,A
    CALL DB2EH
    LD E,(HL)
    CALL NC,C8B1H
    LD A,B
    CALL DB2EH
    LD L,(HL)
    CALL NC,7ACDH
    RET NZ
    RET
    LD C,B
    CALL DB2EH
    OR A
    RET NC
    OR A
    RET NZ
    PUSH HL
    POP IX
    LD E,(IX+16H)
    LD D,(IX+17H)
    PUSH DE
    LD DE,0002H
    LD B,D
    LD C,D

LCEA8:
    PUSH DE
    CALL DB2EH
    RET Z
    CALL NC,B37AH
    JR NZ,CEB3H
    INC BC

LCEB3:
    POP DE
    INC DE
    EX (SP),HL
    SBC HL,DE
    ADD HL,DE
    EX (SP),HL
    JR NZ,CEA8H
    LD H,B
    LD L,C
    POP DE
    DEC DE
    DEC DE
    LD BC,0008H
    ADD IX,BC
    LD B,(IX+02H)
    INC B
    XOR A
    RET
    LD H,A
    LD L,A
    LD D,A
    LD E,A
    LD B,10H

LCED2:
    LD A,B
    EXX
    CALL C095H
    LD A,(HL)
    INC HL
    OR (HL)
    ADD A,FFH
    EXX
    ADC HL,HL
    RL E
    RL D
    DJNZ CED2H
    XOR A
    RET
    LD A,E
    CP 2BH
    LD A,99H
    RET NC
    LD A,D
    LD HL,D000H
    LD D,00H
    ADD HL,DE
    INC B
    DEC B
    JR Z,CEF9H
    LD (HL),A

LCEF9:
    LD D,(HL)
    XOR A
    RET

; hbf: Unused zero fill through the end of the page.
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H, 00H ; |................|
    DB 00H, 00H, 00H, 00H                                                           ; |....|
