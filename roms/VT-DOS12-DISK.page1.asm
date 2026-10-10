; -----------------------------------------------------------------------------
; VT-DOS 1.2 HBF disk ROM page 1
; Source: roms/VT-DOS12-DISK.ROM
; HBF ROM page: 1
; File offset: 1000H
; CPU window: C000H-CFFFH while this HBF ROM page is selected
; HBF ROM page 1 is mapped at C000H-CFFFH only after the card selects page 1. It holds the CLI message table.
; ORG: C000H
; Size: 4096 bytes
; Comments: roms/VT-DOS12-DISK.page1.comments.json
; Data ranges: C000H-C00AH, C0A2H-C0A8H, C86FH-C881H, CC3EH-CFD4H
; Auto labels: branch and call targets are emitted as Lxxxx.
; -----------------------------------------------------------------------------

ORG C000H


; hbf: MOPS signature and the length-prefixed name DISK. Page 0, not this page, supplies the C00BH initializer and the C00DH function table.
    DB 4DH, 4FH, 50H, 53H, 04H, 44H, 49H, 53H, 4BH, 6FH, DBH                        ; |MOPS.DISKo.|
    NOP
    NOP
    LD BC,DBC1H

LC010:
    CALL C024H
    CALL DB2EH
    DB FAH, C0H                                                                     ; |..|

LC018:
    PUSH HL
    EX (SP),IX
    LD (IX+09H),64H
    EX (SP),IX
    POP HL
    OR A
    RET

LC024:
    PUSH AF
    PUSH DE
    PUSH HL
    LD DE,0009H
    ADD HL,DE
    LD A,02H
    CP (HL)
    POP HL
    JR C,C04BH
    PUSH BC
    CALL Z,C04EH
    JR Z,C04AH
    PUSH IX

LC039:
    CALL C05FH
    CALL C072H
    JR Z,C048H
    LD A,B4H
    CALL CAA6H
    JR C039H

LC048:
    POP IX

LC04A:
    POP BC

LC04B:
    POP DE
    POP AF
    RET

LC04E:
    LD A,02H
    CALL DB2EH
    JP M,05C0H
    RET Z
    INC B
    RET NZ
    LD A,(D023H)
    INC A
    SCF
    RET

LC05F:
    PUSH BC
    PUSH DE

LC061:
    LD A,03H
    CALL DB2EH
    OR C0H
    JR Z,C06FH
    CALL CAA6H
    JR C061H

LC06F:
    POP DE
    POP BC
    RET

LC072:
    PUSH HL
    CALL C088H
    EX DE,HL
    POP HL
    PUSH HL
    LD BC,0019H
    ADD HL,BC
    CALL C0ADH
    JR NZ,C086H
    LD A,(HL)
    CP (IX+15H)

LC086:
    POP HL
    RET

LC088:
    PUSH IX
    POP HL
    LD DE,0040H
    ADD HL,DE
    LD DE,C0A2H
    LD B,06H

LC094:
    LD A,(DE)
    CP (HL)
    JR NZ,C09EH
    INC HL
    INC DE
    DJNZ C094H
    INC HL
    RET

LC09E:
    LD HL,C0A9H
    RET

; hbf: NUL-terminated VOL_ID string.
    DB 56H, 4FH, 4CH, 5FH, 49H, 44H, 00H                                            ; |VOL_ID.|
    RST 38H
    RST 38H
    RST 38H
    RST 38H

LC0AD:
    LD B,04H

LC0AF:
    LD A,(DE)
    SUB (HL)
    RET NZ
    INC DE
    INC HL
    DJNZ C0AFH
    RET
    LD B,00H
    LD IX,D46EH

LC0BD:
    LD A,C
    CALL DB2EH
    SUB L
    RET NZ
    LD C,A
    LD (D06EH),A
    LD E,(HL)
    INC HL
    LD D,(HL)
    LD A,D
    OR E
    LD A,ABH
    RET Z
    PUSH DE
    LD HL,0007H
    ADD HL,DE
    LD E,(HL)
    LD D,00H
    LD HL,D0AAH
    ADD HL,DE
    LD A,(HL)
    CP C
    JR Z,C0ECH
    OR A
    JR Z,C0ECH
    CALL C45EH
    LD A,B2H
    EX (SP),HL
    CALL CAA6H
    EX (SP),HL

LC0EC:
    LD (HL),C
    DEC B
    JR NZ,C154H

LC0F0:
    PUSH IX
    POP HL
    LD BC,002CH
    ADD HL,BC
    EX DE,HL
    POP HL
    PUSH IX
    PUSH HL
    LD C,09H
    ADD HL,BC
    LD A,(HL)
    OR A
    JR Z,C114H
    LD C,10H
    ADD HL,BC
    LD C,A
    PUSH DE
    CALL C0ADH
    POP DE
    JR Z,C11DH
    LD A,(D06EH)
    CALL C45EH

LC114:
    POP HL
    CALL C05FH
    PUSH HL
    LD C,01H
    JR C130H

LC11D:
    POP HL
    PUSH HL
    DEC C
    JR Z,C12CH
    DEC C
    JR NZ,C145H
    PUSH DE
    CALL C04EH
    POP DE
    JR Z,C145H

LC12C:
    CALL C05FH
    LD C,H

LC130:
    PUSH DE
    CALL C088H
    POP DE
    CALL C0ADH
    POP HL
    PUSH HL
    JR NZ,C147H
    DEC C
    JR NZ,C145H
    CALL C225H
    CALL C46DH

LC145:
    JR C19CH

LC147:
    LD A,B3H
    CALL CAA8H
    POP HL
    JR NZ,C186H
    POP IX
    PUSH HL
    JR C0F0H

LC154:
    POP HL
    CALL C45DH
    XOR A
    DEC B
    RET Z
    PUSH IX
    PUSH HL
    LD BC,0009H
    ADD HL,BC
    LD A,(HL)
    SUB 02H
    LD D,A
    POP HL
    JR C,C174H
    JR NZ,C18CH
    PUSH DE
    CALL C04EH
    POP DE
    JR C,C174H
    JR Z,C18CH

LC174:
    CALL C05FH
    INC D
    INC D
    JR Z,C186H
    CALL C072H
    JR NZ,C186H
    DEC DE
    LD A,(DE)
    BIT 7,A
    JR Z,C18CH

LC186:
    CALL C225H
    CALL C46DH

LC18C:
    POP DE
    PUSH DE
    PUSH HL
    LD BC,0019H
    ADD HL,BC
    EX DE,HL
    LD C,2CH
    ADD HL,BC
    EX DE,HL
    LD C,04H
    LDIR

LC19C:
    POP HL
    POP IX
    XOR A
    JP C018H
    POP HL
    LD A,98H
    RET
    LD HL,D46EH
    PUSH HL
    LD A,C
    CALL DB2EH
    XOR D
    RET NZ
    EX DE,HL
    POP DE
    LD A,98H
    RET Z
    PUSH DE
    LD A,B
    LD BC,0008H
    LDIR
    PUSH AF
    CALL C45EH
    CALL C46EH
    POP AF
    CALL DB2EH
    XOR D
    RET NZ
    POP HL
    LD A,98H
    RET Z
    PUSH DE
    LD C,08H

LC1D1:
    LDIR
    POP HL
    PUSH HL
    LD DE,0009H
    ADD HL,DE
    LD (HL),D
    LD E,15H
    ADD HL,DE
    LD (HL),FFH
    POP HL
    XOR A
    RET
    LD HL,D0DDH
    LD A,B
    OR A
    JR Z,C1FFH
    CP 11H
    LD A,ABH
    RET NC
    LD E,B
    LD D,00H
    ADD HL,DE
    LD A,C
    OR A
    JR Z,C1FCH
    CP 11H
    LD A,ABH
    RET NC
    LD B,C

LC1FC:
    LD (HL),B
    XOR A
    RET

LC1FF:
    LD (HL),A
    INC HL
    INC A
    CP 11H
    JR NZ,C1FFH
    XOR A
    RET
    LD A,B
    BIT 7,A
    JR NZ,C212H
    CALL DB2EH
    SUB L
    RET NZ

LC212:
    AND 7FH
    CALL DB2EH
    XOR D
    RET NZ
    LD A,ABH
    RET Z
    LD HL,0007H
    ADD HL,DE
    LD C,(HL)
    INC HL
    LD B,(HL)
    XOR A
    RET

LC225:
    PUSH HL
    LD BC,000AH
    ADD HL,BC
    LD A,(IX+0DH)
    DEC A
    LD (HL),A
    INC HL
    LD C,FFH

LC232:
    INC C
    RRCA
    JR C,C232H
    LD (HL),C
    INC HL
    LD E,(IX+0EH)
    LD (HL),E
    INC HL
    LD D,(IX+0FH)
    LD (HL),D
    INC HL
    PUSH DE
    LD B,(IX+10H)
    LD (HL),B
    INC HL
    LD E,(IX+11H)
    LD D,(IX+12H)
    LD A,E
    AND 0FH
    LD (HL),A
    LD A,04H

LC254:
    SRL D
    RR E
    DEC A
    JR NZ,C254H
    CP (HL)
    INC HL
    LD (HL),E
    INC HL
    JR NC,C262H
    INC DE

LC262:
    LD A,(IX+16H)
    LD (HL),A
    INC HL
    EX (SP),HL
    PUSH DE
    LD E,A
    XOR A
    LD D,A

LC26C:
    ADD A,E
    JR NC,C270H
    INC D

LC270:
    DJNZ C26CH
    LD E,A
    ADD HL,DE
    EX DE,HL
    POP HL
    ADD HL,DE
    EX (SP),HL
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    POP DE
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    PUSH HL
    LD L,(IX+13H)
    LD H,(IX+14H)
    OR A
    SBC HL,DE
    INC C

LC28C:
    DEC C
    JR Z,C295H
    SRL H
    RR L
    JR C28CH

LC295:
    INC HL
    INC HL
    EX DE,HL
    POP HL
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    PUSH HL
    CALL C088H
    POP DE
    DEC HL
    LD BC,0005H
    LDIR
    EX DE,HL
    LD A,(IX+15H)
    LD (HL),A
    POP HL
    PUSH HL
    PUSH BC
    LD BC,001FH
    ADD HL,BC
    POP BC
    XOR A
    BIT 7,(HL)
    JR NZ,C2BDH
    LD (HL),A
    DEC HL
    LD (HL),A

LC2BD:
    POP HL
    RET

LC2BF:
    LD A,(HL)
    INC HL
    OR (HL)
    DEC HL
    RET Z
    PUSH DE
    PUSH BC
    PUSH HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    PUSH DE
    XOR A
    LD (HL),A
    DEC HL
    LD (HL),A
    LD B,H
    LD C,L
    LD HL,D067H
    CALL C30BH
    EX DE,HL
    POP DE
    LD (HL),E
    INC HL
    LD (HL),D
    DEC HL
    CALL C30BH

LC2E0:
    EX DE,HL
    LD (HL),C
    INC HL
    LD (HL),B
    POP HL
    POP BC

LC2E6:
    POP DE
    RET

LC2E8:
    CALL C31AH

LC2EB:
    PUSH DE
    LD DE,(D067H)
    OR A
    SBC HL,DE
    ADD HL,DE
    JR Z,C2E6H
    PUSH BC
    PUSH HL
    LD (D067H),HL
    LD C,(HL)
    INC HL
    LD B,(HL)
    PUSH BC
    LD (HL),D
    DEC HL
    LD (HL),E
    EX DE,HL
    LD B,D
    LD C,E
    CALL C30BH
    POP BC
    JR C2E0H

LC30B:
    LD E,(HL)
    INC HL
    LD D,(HL)
    DEC HL
    EX DE,HL
    LD A,H
    OR L
    RET Z
    SBC HL,BC
    ADD HL,BC
    JR NZ,C30BH
    OR A
    RET

LC31A:
    SET 2,(IY+22H)
    PUSH HL
    INC HL
    INC HL
    LD A,(HL)
    OR A
    JR Z,C38BH
    INC HL
    BIT 7,(HL)
    JR Z,C38BH
    RES 7,(HL)
    PUSH DE
    PUSH BC
    PUSH IX

LC330:
    PUSH HL
    BIT 6,(HL)
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    LD B,(HL)
    INC HL
    JR Z,C346H
    LD A,E

LC33D:
    ADD A,(HL)
    JR NC,C341H
    INC D

LC341:
    DJNZ C33DH
    LD E,A
    LD B,01H

LC346:
    PUSH DE
    INC HL
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    EX DE,HL
    PUSH DE
    POP IX
    LD C,01H
    POP DE

LC354:
    PUSH DE
    PUSH BC
    LD BC,01FFH
    CALL C38DH
    CALL C010H
    POP BC
    POP DE
    JR NZ,C364H
    INC C

LC364:
    CP AFH
    JR NZ,C36BH
    LD BC,0101H

LC36B:
    PUSH AF
    LD A,E
    ADD A,(IX-04H)
    LD E,A
    JR NC,C374H
    INC D

LC374:
    POP AF
    DJNZ C354H
    DEC C
    JR NZ,C384H
    BIT 1,(IX-08H)
    JR NZ,C381H
    SCF

LC381:
    CALL CAA9H

LC384:
    POP HL
    JR Z,C330H
    POP IX
    POP BC
    POP DE

LC38B:
    POP HL
    RET

LC38D:
    LD A,(D025H)
    OR A
    LD A,05H
    RET NZ
    INC A
    RET

LC396:
    LD B,01H
    LD A,(IX+0EH)
    BIT 0,(IX+18H)
    JR Z,C3A6H
    DEC A
    JR NZ,C3A6H

LC3A4:
    LD A,01H

LC3A6:
    LD (D0A7H),A
    LD C,(IX+08H)
    LD HL,(D069H)
    LD A,H
    SUB 01H
    CALL NC,C44BH
    RET Z
    LD HL,(D067H)
    PUSH HL

LC3BA:
    CALL C44BH
    JR Z,C435H
    LD A,(HL)
    INC HL
    LD H,(HL)
    LD L,A
    OR H
    JR NZ,C3BAH
    POP HL
    INC HL
    INC HL
    BIT 7,(HL)
    DEC HL
    LD A,(HL)
    DEC HL
    JR Z,C3D2H
    LD L,(HL)
    LD H,A

LC3D2:
    CALL C31AH
    PUSH HL
    INC HL
    INC HL
    LD (HL),C
    INC HL
    LD A,B
    AND 02H
    LD (HL),A
    INC HL
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    LD A,(IY+27H)
    LD (HL),A
    INC HL
    LD C,(IX+11H)
    LD (HL),C
    INC HL
    PUSH DE
    INC HL
    INC HL
    INC HL
    PUSH HL
    EX (SP),IX
    POP HL
    LD (IX-02H),L
    LD (IX-01H),H
    POP DE
    BIT 0,B
    JR Z,C431H

LC400:
    PUSH BC
    PUSH DE
    LD B,(IY+27H)
    JR C40DH

LC407:
    LD A,E
    ADD A,C
    LD E,A
    JR NC,C40DH
    INC D

LC40D:
    PUSH DE
    PUSH BC
    LD BC,01FFH
    LD A,04H
    CALL C010H
    POP BC
    POP DE
    JR Z,C42FH
    CP AFH
    JR Z,C421H
    DJNZ C407H

LC421:
    POP DE
    POP BC
    OR A
    BIT 1,B
    JR NZ,C429H
    SCF

LC429:
    CALL CAA9H
    JR Z,C400H
    DB CAH                                                                          ; |.|

LC42F:
    POP DE
    POP BC

LC431:
    EX (SP),HL
    POP IX
    DB FEH                                                                          ; |.|

LC435:
    POP BC
    SET 2,(IY+22H)
    LD (D069H),HL
    CALL C2BFH
    XOR A
    RET

LC442:
    LD HL,(D069H)
    INC HL
    INC HL
    INC HL
    SET 7,(HL)
    RET

LC44B:
    PUSH HL
    INC HL
    INC HL
    LD A,(HL)
    SUB C
    JR NZ,C45BH
    INC HL
    INC HL
    LD A,(HL)
    SUB E
    JR NZ,C45BH
    INC HL
    LD A,(HL)
    SUB D

LC45B:
    POP HL
    RET

LC45D:
    DB F6H                                                                          ; |.|

LC45E:
    SCF
    CALL C481H
    PUSH HL

LC463:
    CALL C4B3H
    JR Z,C4B1H
    CALL C31AH
    JR C463H

LC46D:
    DB F6H                                                                          ; |.|

LC46E:
    SCF
    CALL C481H
    PUSH HL

LC473:
    CALL C4B3H
    JR Z,C4B1H
    CALL C2EBH
    INC HL
    INC HL
    LD (HL),00H
    JR C473H

LC481:
    JR C,C48CH
    PUSH HL
    PUSH BC
    LD BC,0008H
    ADD HL,BC
    LD A,(HL)
    POP BC
    POP HL

LC48C:
    PUSH HL
    PUSH DE
    PUSH BC
    LD B,A
    LD HL,(D067H)

LC493:
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    LD A,(HL)
    INC HL
    RES 0,(HL)
    OR A
    JR Z,C4AAH
    CP B
    JR Z,C4A8H
    RLCA
    JR C,C4AAH
    LD A,B
    INC A
    JR NZ,C4AAH

LC4A8:
    SET 0,(HL)

LC4AA:
    LD A,D
    OR E
    EX DE,HL
    JR NZ,C493H
    POP BC
    POP DE

LC4B1:
    POP HL
    RET

LC4B3:
    LD HL,(D067H)

LC4B6:
    PUSH HL
    INC HL
    INC HL
    INC HL
    BIT 0,(HL)
    RES 0,(HL)
    POP HL
    RET NZ
    LD A,(HL)
    INC HL
    LD H,(HL)
    LD L,A
    OR H
    JR NZ,C4B6H
    RET

LC4C8:
    CALL C56BH
    JR Z,C4D6H

LC4CD:
    LD A,B0H
    CALL CAA8H
    JR Z,C4CDH
    JR C4EDH

LC4D6:
    PUSH HL
    LD A,(DE)
    LD L,A
    INC DE
    LD A,(DE)
    JR NC,C4E2H
    LD H,A
    CALL C5CDH
    LD L,H

LC4E2:
    AND 0FH
    LD H,A
    EX DE,HL
    LD HL,0FF7H
    SBC HL,DE
    POP HL
    RET NC

LC4ED:
    LD DE,FFFFH
    RET

LC4F1:
    PUSH HL
    PUSH BC
    LD BC,000BH
    ADD HL,BC
    LD B,(HL)
    INC B
    EX DE,HL
    DEC HL
    DEC HL
    DB 0EH                                                                          ; |.|

LC4FD:
    ADD HL,HL
    DJNZ C4FDH
    ADD A,L
    LD L,A
    EX DE,HL
    LD C,09H
    ADD HL,BC
    LD A,(HL)
    INC HL
    LD H,(HL)
    LD L,A
    ADD HL,DE
    EX DE,HL
    POP BC
    POP HL
    RET
    CALL C5D7H
    PUSH DE
    LD A,B
    CP 10H
    JR C,C51BH
    LD BC,0FFFH

LC51B:
    CALL C56BH
    JR Z,C529H

LC520:
    LD A,B0H
    CALL CAA8H
    JR Z,C520H
    JR C569H

LC529:
    PUSH HL
    JR C,C536H
    LD A,C
    LD (DE),A
    INC DE
    LD A,(DE)
    AND F0H
    OR B
    LD (DE),A
    JR C543H

LC536:
    LD H,B
    LD L,C
    CALL C5CDH
    LD A,(DE)
    AND 0FH
    OR L
    LD (DE),A
    INC DE
    LD A,H
    LD (DE),A

LC543:
    CALL C442H
    BIT 0,(IY+22H)
    JR Z,C568H
    LD BC,0008H
    ADD HL,BC
    LD (HL),A
    DEC DE
    LD A,(DE)
    LD DE,(D0A3H)
    EX (SP),IX
    PUSH AF
    CALL C396H
    CALL C442H
    LD BC,0207H
    ADD HL,BC
    POP AF
    LD (HL),A
    EX (SP),IX

LC568:
    POP HL

LC569:
    POP DE
    RET

LC56B:
    PUSH IX
    PUSH BC
    PUSH HL
    PUSH HL
    POP IX
    RES 0,(IY+22H)
    LD L,(IX+16H)
    LD H,(IX+17H)
    DEC HL
    XOR A
    SBC HL,DE
    JR C,C5C8H
    LD H,D
    LD L,E
    ADD HL,HL
    ADD HL,DE
    SRL H
    RR L
    PUSH AF
    PUSH HL
    LD E,H
    SRL E
    LD D,A
    LD L,(IX+0CH)
    LD H,(IX+0DH)
    ADD HL,DE
    EX DE,HL
    CALL C396H
    LD BC,000BH
    ADD HL,BC
    POP BC
    LD A,B
    AND 01H
    LD B,A
    ADD HL,BC
    ADD A,C
    JR NC,C5C3H
    LD A,(HL)
    SET 0,(IY+22H)
    LD (D0A3H),DE
    LD (D0A5H),A
    INC DE
    CALL C396H
    LD BC,000BH
    ADD HL,BC
    LD A,(HL)
    LD (D0A6H),A
    LD HL,D0A5H

LC5C3:
    EX DE,HL
    POP AF
    SBC A,A
    CP A
    RRCA

LC5C8:
    POP HL
    POP BC
    POP IX
    RET

LC5CD:
    XOR A
    ADD HL,HL
    RLA
    ADD HL,HL
    RLA
    ADD HL,HL
    RLA
    ADD HL,HL
    RLA
    RET

LC5D7:
    PUSH HL
    EX (SP),IX
    BIT 0,(IX+18H)
    JR Z,C64BH
    XOR A
    JR C5F4H
    PUSH HL
    EX (SP),IX
    BIT 7,(IX+19H)
    JR NZ,C64BH
    BIT 0,(IX+18H)
    JR NZ,C64BH
    LD A,01H

LC5F4:
    PUSH BC
    PUSH DE
    LD B,(IX+0EH)
    DEC B
    JR Z,C62BH
    LD E,(IX+0CH)
    LD D,(IX+0DH)
    LD B,(IX+11H)

LC605:
    PUSH BC
    PUSH AF
    DEC A
    LD B,00H
    JR NZ,C611H
    CALL C3A4H
    JR C61EH

LC611:
    CALL C396H
    CALL C31AH
    PUSH HL
    CALL C442H
    SET 6,(HL)
    POP HL

LC61E:
    CALL C2E8H
    INC HL
    INC HL
    LD (HL),00H
    POP AF
    POP BC
    INC DE
    DJNZ C605H
    LD B,A

LC62B:
    LD A,B
    CP (IX+18H)
    JR Z,C649H
    PUSH BC
    LD DE,0000H
    LD B,01H
    CALL C3A4H
    CALL C2EBH
    CALL C442H
    LD DE,004EH
    ADD HL,DE
    POP BC
    LD (HL),B
    LD (IX+18H),B

LC649:
    POP DE
    POP BC

LC64B:
    EX (SP),IX
    POP HL
    RET
    LD A,(DE)
    INC A
    JR Z,C663H
    CALL C679H
    RET NZ
    AND 10H
    CP 10H
    LD A,A8H
    RET NZ
    CALL C985H
    JR C674H

LC663:
    PUSH DE
    POP IX
    PUSH BC
    EX DE,HL
    LD C,81H
    CALL C6DDH
    JR NZ,C672H
    OR C
    LD A,AAH

LC672:
    POP BC
    RET NZ

LC674:
    CALL C9DEH
    CP A
    RET

LC679:
    LD A,(DE)
    CP FEH
    LD A,ACH
    RET NZ
    PUSH DE
    EX DE,HL
    CALL C6A6H
    POP DE
    RET NZ
    PUSH BC
    PUSH DE
    PUSH HL
    LD BC,0019H
    ADD HL,BC
    EX DE,HL
    LD C,2CH
    ADD HL,BC
    EX DE,HL
    LD C,04H
    LDIR
    POP HL
    CALL C9FBH
    PUSH HL
    LD HL,000BH
    ADD HL,DE
    LD A,(HL)
    POP HL
    POP IX
    POP BC
    CP A
    RET

LC6A6:
    PUSH BC
    PUSH DE
    POP IX
    LD C,(IX+07H)
    LD B,01H
    CALL C0BDH
    OR A
    JR NZ,C6BDH
    PUSH HL
    CALL C6C3H
    LDIR
    POP HL
    XOR A

LC6BD:
    POP BC
    SET 2,(IY+22H)
    RET

LC6C3:
    LD DE,D06FH
    PUSH IX
    POP HL
    LD BC,0017H
    ADD HL,BC

LC6CD:
    LD BC,000CH
    RET
    LD B,00H
    LD IX,D46EH
    LD (IX+06H),A
    LD (IX+07H),B

LC6DD:
    RES 1,C
    EXX
    LD B,04H
    CALL C882H
    JR Z,C6FEH
    CALL C84AH
    JR NZ,C6FBH
    SUB 40H
    LD C,A
    CALL C882H
    JR Z,C6FBH
    CP 3AH
    JR Z,C70AH
    CALL C89CH

LC6FB:
    CALL C89CH

LC6FE:
    LD A,(IX+07H)
    LD C,(IY-5FH)
    OR A
    JR Z,C708H
    LD C,A

LC708:
    RES 2,B

LC70A:
    LD (IX+07H),C
    PUSH BC
    XOR A
    LD B,A
    EXX
    BIT 7,C
    EXX
    JR Z,C719H
    CALL C0BDH

LC719:
    POP BC
    OR A
    RET NZ
    BIT 3,(IX+06H)
    PUSH AF
    JR NZ,C72EH
    CALL C882H
    JR Z,C731H
    CP 5CH
    JR NZ,C731H
    SET 0,B

LC72E:
    SET 1,B
    XOR A

LC731:
    CALL NZ,C89CH
    EXX
    BIT 7,C
    EXX
    CALL NZ,C8A0H
    EXX
    LD (D09AH),DE
    EXX
    POP AF
    JR NZ,C77FH

LC744:
    CALL C7ADH
    CP 5CH
    JR NZ,C756H
    CALL C911H
    OR A
    JR NZ,C763H
    CALL C882H
    JR C744H

LC756:
    XOR A
    EXX
    BIT 0,C
    EXX
    JR Z,C763H
    LD A,B
    AND 18H
    CALL NZ,C911H

LC763:
    PUSH AF
    CALL C882H
    CALL NZ,C89CH
    LD C,A
    POP AF
    EXX
    LD B,00H
    BIT 7,C
    JR Z,C776H
    LD (HL),B
    INC HL
    LD (HL),B

LC776:
    LD HL,(D09AH)
    LD (HL),B
    PUSH DE
    EXX
    POP DE
    OR A
    RET

LC77F:
    EXX
    SET 1,C
    EXX
    PUSH HL
    LD HL,D49EH
    LD C,0BH

LC789:
    INC C

LC78A:
    CALL C882H
    JR Z,C7A1H
    CP 2FH
    JR Z,C79EH
    SET 0,B
    SET 3,B
    DEC C
    JR Z,C789H
    LD (HL),A
    INC HL
    JR C78AH

LC79E:
    CALL C89CH

LC7A1:
    DEC C
    JR Z,C7A9H
    LD (HL),20H
    INC HL
    JR C7A1H

LC7A9:
    POP HL
    XOR A
    JR C763H

LC7AD:
    PUSH DE
    PUSH HL
    LD HL,D49EH
    LD C,08H
    LD A,B
    AND 07H
    LD B,A
    CALL C882H
    JR Z,C7FBH
    CP 2EH
    JR NZ,C7F8H
    LD D,01H
    CALL C882H
    JR Z,C7E5H
    CALL C855H
    JR NZ,C7F5H
    CP 2EH
    JR NZ,C7E2H
    SET 7,B
    INC D
    CALL C882H
    JR Z,C7E5H
    CALL C855H
    JR NZ,C7F2H
    CP 2EH
    JR Z,C7F2H

LC7E2:
    CALL C89CH

LC7E5:
    LD (HL),2EH
    INC HL
    DEC C
    DEC D
    JR NZ,C7E5H
    LD A,B
    OR 49H
    LD B,A
    JR C7FBH

LC7F2:
    CALL C89CH

LC7F5:
    CALL C89CH

LC7F8:
    CALL C89CH

LC7FB:
    CALL C814H
    DEC D
    JR NZ,C803H
    SET 3,B

LC803:
    CP 2EH
    JR NZ,C80CH
    SET 4,B
    CALL C882H

LC80C:
    LD C,03H
    CALL C814H
    POP HL
    POP DE
    RET

LC814:
    LD D,00H

LC816:
    INC C

LC817:
    CALL C882H
    JR Z,C843H
    CALL C855H
    JR Z,C840H
    CP 2AH
    JR Z,C836H
    CP 3FH
    JR NZ,C82BH

LC829:
    SET 5,B

LC82B:
    SET 0,B
    LD D,01H
    DEC C
    JR Z,C816H
    LD (HL),A
    INC HL
    JR C817H

LC836:
    LD A,C

LC837:
    LD C,A
    DEC A
    JR Z,C829H
    LD (HL),3FH
    INC HL
    JR C837H

LC840:
    CALL C89CH

LC843:
    DEC C
    RET Z
    LD (HL),20H
    INC HL
    JR C843H

LC84A:
    CP 41H
    RET C
    CP 5BH
    JR NC,C853H
    CP A
    RET

LC853:
    OR A
    RET

LC855:
    CP 20H
    JR NC,C85BH
    CP A
    RET

LC85B:
    PUSH HL
    PUSH BC
    LD BC,0013H
    JR C867H
    PUSH HL
    PUSH BC
    LD BC,0002H

LC867:
    LD HL,C86FH
    CPIR
    POP BC
    POP HL
    RET

; hbf: 19-byte filename punctuation table, scanned with CPIR.
    DB 20H, 09H, 3AH, 3BH, 2EH, 2CH, 3DH, 2BH, 23H, 5CH, 21H, 27H, 3CH, 3EH, 7CH, 2FH ; | .:;.,=+#\!'<>|/|
    DB 22H, 5BH, 5DH                                                                ; |"[]|

LC882:
    EXX
    LD A,(DE)
    OR A
    JR Z,C89AH
    INC DE
    BIT 1,C
    JR NZ,C89AH
    CALL CFECH
    CP 21H
    JR Z,C897H
    CP 27H
    JR NZ,C899H

LC897:
    LD A,5CH

LC899:
    OR A

LC89A:
    EXX
    RET

LC89C:
    EXX
    DEC DE
    EXX
    RET

LC8A0:
    PUSH HL
    BIT 1,B
    JR NZ,C8E9H
    LD DE,001EH
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    BIT 7,D
    JR NZ,C8E9H
    LD A,D
    OR E
    EX (SP),HL
    EXX
    POP HL
    PUSH HL
    PUSH BC
    PUSH DE
    INC HL
    EX DE,HL
    SET 7,C
    JR Z,C8C0H
    RES 7,C

LC8C0:
    EXX
    PUSH BC
    PUSH DE
    CALL C8E8H
    POP DE

LC8C7:
    CALL C7ADH
    CALL C91DH
    OR A
    JR NZ,C8D6H
    CALL C882H
    JR NZ,C8C7H
    DB C2H                                                                          ; |.|

LC8D6:
    LD D,FFH
    POP BC
    EXX
    POP DE
    POP BC
    EXX
    EX (SP),HL
    LD (HL),D
    DEC HL
    LD (HL),E
    JR Z,C8F8H
    INC HL
    INC HL
    LD (HL),00H
    POP HL

LC8E8:
    PUSH HL

LC8E9:
    EXX
    PUSH DE
    LD HL,D4EBH
    LD (HL),3AH
    LD DE,FFBFH
    ADD HL,DE
    POP DE
    EXX

LC8F6:
    LD D,FFH

LC8F8:
    LD (D06FH),DE
    EXX
    BIT 2,C
    EXX
    JR Z,C90EH
    LD HL,(D09DH)
    OR A
    SBC HL,DE
    JR NZ,C90EH
    SET 6,(IY+1CH)

LC90E:
    XOR A
    POP HL
    RET

LC911:
    XOR A
    SET 1,B
    EXX
    BIT 7,C
    LD (D09AH),DE
    EXX
    RET Z

LC91D:
    PUSH DE
    LD DE,D49EH
    LD A,B
    EXX
    PUSH BC
    PUSH DE
    EX DE,HL
    LD C,A
    BIT 6,C
    JR Z,C940H
    BIT 7,C
    JR Z,C959H

LC92F:
    DEC DE
    LD A,(DE)
    CP 5CH
    JR Z,C959H
    OR A
    JR NZ,C92FH

LC938:
    EX DE,HL
    POP DE
    POP BC

LC93B:
    LD A,AAH
    EXX
    POP DE
    RET

LC940:
    EXX
    PUSH DE
    EXX
    POP HL
    LD B,09H
    LD A,5CH
    CALL C99BH
    JR Z,C938H
    BIT 4,C
    LD A,2EH
    LD B,04H
    CALL NZ,C99BH
    OR A
    JR Z,C938H

LC959:
    EX DE,HL
    LD A,C
    POP DE
    POP BC
    AND 28H
    SUB 08H
    JR NZ,C93BH
    BIT 7,C
    EXX
    POP DE
    RET Z
    CALL C9DEH

LC96B:
    JR Z,C982H
    PUSH HL
    PUSH BC
    LD HL,D49EH
    CALL C9A5H
    POP BC
    POP HL
    JR NC,C97DH
    BIT 4,A
    JR NZ,C985H

LC97D:
    CALL CA04H
    JR NC,C96BH

LC982:
    LD A,A0H
    RET

LC985:
    PUSH HL
    LD HL,001AH
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    LD A,D
    OR E
    JP NZ,C8F8H
    JP C8F6H

LC995:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,C9A1H

LC99B:
    LD (DE),A
    INC DE
    LD A,(DE)
    SUB 3AH
    RET Z

LC9A1:
    DJNZ C995H
    OR A
    RET

LC9A5:
    LD A,(DE)
    OR A
    RET Z
    CP E5H
    RET Z
    PUSH DE
    CP 05H
    JR NZ,C9B2H
    LD A,E5H

LC9B2:
    LD C,(IX+06H)
    LD B,0BH

LC9B7:
    BIT 3,C
    JR NZ,C9CCH
    PUSH DE
    CALL CFECH
    LD E,A
    LD A,(HL)
    CALL CFECH
    CP E
    POP DE
    JR Z,C9CCH
    SUB 3FH
    JR NZ,C9DBH

LC9CC:
    INC HL
    INC DE
    LD A,(DE)
    DJNZ C9B7H
    EX DE,HL
    LD A,C
    XOR (HL)
    AND 08H
    JR NZ,C9DBH
    LD A,(HL)
    SCF
    DB 06H                                                                          ; |.|

LC9DB:
    OR A
    POP DE
    RET

LC9DE:
    LD DE,(D06FH)
    BIT 7,D
    JR Z,CA3EH
    PUSH BC
    PUSH HL
    LD BC,000FH
    ADD HL,BC
    LD C,(HL)
    INC HL
    LD A,(HL)
    INC HL
    INC HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    POP HL
    LD B,FFH
    PUSH BC
    PUSH BC
    JR CA58H

LC9FB:
    INC (IY-08H)
    PUSH BC
    LD BC,0000H
    JR CA08H

LCA04:
    PUSH BC
    LD BC,0020H

LCA08:
    PUSH HL
    EX DE,HL
    ADD HL,BC
    LD DE,(D075H)
    LD A,(D078H)
    DEC A
    JR Z,CA1EH
    BIT 2,(IY+22H)
    JP Z,CA9DH
    JR CA7DH

LCA1E:
    EX (SP),IX
    CALL C3A4H
    CALL C2EBH
    EX (SP),IX
    POP HL
    INC DE
    LD A,(D077H)
    DEC A
    JR NZ,CA65H
    CP (IY-06H)
    JR NZ,CA67H
    LD DE,(D073H)
    BIT 7,D
    SCF
    POP BC
    RET NZ

LCA3E:
    PUSH BC
    PUSH DE
    CALL C4C8H
    LD B,D
    LD C,E
    POP DE
    PUSH DE
    XOR A
    CALL C4F1H
    PUSH DE
    EX DE,HL
    LD HL,000AH
    ADD HL,DE
    LD A,(HL)
    INC A
    EX DE,HL
    POP DE
    PUSH BC
    LD C,00H

LCA58:
    LD (IY-06H),C
    POP BC
    LD (D073H),BC

LCA60:
    POP BC
    LD (D071H),BC

LCA65:
    LD B,10H

LCA67:
    OR A
    JR NZ,CA71H
    LD B,(IY-06H)
    LD (D07AH),A
    INC A

LCA71:
    LD (D077H),A
    LD (D075H),DE
    LD (IY-07H),B
    LD A,B
    PUSH HL

LCA7D:
    EX (SP),IX
    PUSH AF
    LD B,01H
    CALL C3A4H
    POP BC
    EX (SP),IX
    LD DE,FFEBH
    ADD HL,DE
    LD A,(D079H)
    INC A
    SUB B
    LD DE,0020H

LCA94:
    ADD HL,DE
    DEC A
    JR NZ,CA94H
    LD A,B
    RES 2,(IY+22H)

LCA9D:
    LD (D078H),A
    EX DE,HL
    POP HL
    POP BC
    LD A,(DE)
    OR A
    RET

LCAA6:
    SCF
    DB 30H                                                                          ; |0|

LCAA8:
    OR A

LCAA9:
    PUSH HL
    PUSH DE
    PUSH BC
    PUSH IX
    LD B,A
    SBC A,A
    LD (D06DH),A
    LD DE,0008H
    ADD HL,DE
    LD C,(HL)
    INC HL
    LD A,(HL)
    OR A
    JR Z,CABFH
    LD (HL),01H

LCABF:
    LD A,B
    CP F5H
    JR Z,CAE0H
    CP AFH
    LD A,02H
    JR Z,CAE2H
    PUSH BC
    LD HL,(DFF8H)
    LD A,H
    OR L
    JR Z,CAD7H
    CALL DB2EH
    DB DCH, CDH                                                                     ; |..|

LCAD7:
    POP HL
    OR A
    CALL Z,CB01H
    CP 04H
    JR C,CAE2H

LCAE0:
    LD A,01H

LCAE2:
    POP IX
    POP BC
    POP DE
    DEC A
    JR Z,CAECH
    POP HL
    DEC A
    RET

LCAEC:
    LD (IY-5AH),H
    POP HL
    CALL C46DH
    LD A,ADH
    LD HL,0000H
    LD B,H
    LD C,L
    LD D,H
    LD E,L
    CALL DB2EH
    LD A,B
    DB CDH                                                                          ; |.|

LCB01:
    CALL CBC6H
    LD A,H
    CP B2H
    LD A,08H
    JR Z,CB24H
    LD A,H
    LD DE,D42DH
    CALL CFD5H
    JR NC,CB1EH
    LD A,0AH
    CALL CBECH
    LD A,H
    CALL CC0DH
    SCF

LCB1E:
    CALL NC,CBF2H
    LD A,01H
    OR A

LCB24:
    PUSH AF
    CALL CBECH
    LD A,02H
    CALL CBECH
    LD A,L
    ADD A,40H
    CALL CBCDH
    LD A,3AH
    CALL CBCDH
    CALL CBC6H

LCB3B:
    POP AF
    PUSH AF
    LD A,09H
    JR Z,CB4BH
    LD A,(D06DH)
    OR A
    LD A,03H
    JR Z,CB4BH
    LD A,04H

LCB4B:
    CALL CBECH
    CALL CB92H
    POP BC
    LD A,01H
    RET NZ
    PUSH BC
    LD A,H
    CP 20H
    CALL NC,CBCDH
    CALL CBC6H
    POP AF
    PUSH AF
    JR NZ,CB6CH
    LD A,H
    CP 0DH
    JR NZ,CB3BH
    POP AF
    LD A,02H
    RET

LCB6C:
    LD L,00H
    LD A,05H
    CALL CB82H
    LD A,06H
    CALL NC,CB82H
    LD A,07H
    CALL NC,CB82H
    JR NC,CB3BH
    POP AF
    LD A,L
    RET

LCB82:
    INC L
    LD DE,D42DH
    CALL CFD5H

LCB89:
    LD A,(DE)
    OR A
    RET Z
    INC DE
    CP H
    JR NZ,CB89H
    SCF
    RET

LCB92:
    CALL DB2EH
    LD H,A
    RST 00H
    LD H,7FH

LCB99:
    LD C,H
    RST 30H
    LD HL,130EH
    RST 30H
    LD HL,0011H
    INC B

LCBA3:
    PUSH DE
    RST 30H
    SUB E
    POP DE
    JR NZ,CBC0H
    OR C
    JR NZ,CBB7H
    DEC DE
    LD A,D
    OR E
    JR NZ,CBA3H

LCBB1:
    LD A,H
    XOR 5FH
    LD H,A
    JR CB99H

LCBB7:
    BIT 6,H
    JR NZ,CBB1H
    RST 30H
    SUB C
    JR NZ,CBC0H
    LD H,C

LCBC0:
    CALL DB2EH
    LD B,C
    RST 00H
    RET

LCBC6:
    LD A,0DH
    CALL CBCDH
    LD A,0AH

LCBCD:
    PUSH DE
    PUSH BC
    CALL DB2EH
    LD H,A
    RST 00H
    LD C,A
    RST 30H
    DB 21H                                                                          ; |!|

LCBD7:
    CALL DB2EH
    LD B,C
    RST 00H
    POP BC
    POP DE
    XOR A
    RET
    PUSH DE
    PUSH BC
    CALL DB2EH
    LD H,A
    RST 00H
    LD C,A
    RST 30H
    LD B,C
    JR CBD7H

LCBEC:
    LD DE,D42DH
    CALL CFD5H

LCBF2:
    LD A,(DE)
    OR A
    RET Z
    INC DE
    CALL CBCDH
    JR CBF2H
    LD DE,D42DH
    CALL CFD5H
    RET C
    LD A,(DE)
    OR A
    RET Z
    INC DE
    CALL DB2EH
    RET C
    CALL M,F518H

LCC0D:
    LD BC,0064H
    CALL CC1BH
    LD C,0AH
    CALL CC1BH
    LD BC,3001H

LCC1B:
    LD D,FFH

LCC1D:
    SUB C
    INC D
    JR NC,CC1DH
    LD E,A
    LD A,D
    JR Z,CC27H
    LD B,30H

LCC27:
    OR B
    CALL NZ,CBCDH
    LD A,E
    ADD A,C
    RET
    PUSH DE
    PUSH BC
    CALL DB2EH
    LD H,A
    RST 00H

LCC35:
    RST 30H
    SUB E
    OR C
    JR Z,CBD7H
    RST 30H
    SUB C
    JR CC35H

; hbf: CLI message table. Each record is a length byte, a code byte, the text, and a NUL. An FFH sentinel follows the last record.
    DB 09H, BCH, 4EH, 6FH, 74H, 20H, 72H, 65H, 61H, 64H, 79H, 00H, 0CH, BAH, 56H, 65H ; |..Not ready...Ve|
    DB 72H, 69H, 66H, 79H, 20H, 65H, 72H, 72H, 6FH, 72H, 00H, 0AH, B9H, 44H, 61H, 74H ; |rify error...Dat|
    DB 61H, 20H, 65H, 72H, 72H, 6FH, 72H, 00H, 10H, B8H, 53H, 65H, 63H, 74H, 6FH, 72H ; |a error...Sector|
    DB 20H, 6EH, 6FH, 74H, 20H, 66H, 6FH, 75H, 6EH, 64H, 00H, 14H, B7H, 57H, 72H, 69H ; | not found...Wri|
    DB 74H, 65H, 20H, 70H, 72H, 6FH, 74H, 65H, 63H, 74H, 65H, 64H, 20H, 64H, 69H, 73H ; |te protected dis|
    DB 6BH, 00H, 10H, B6H, 55H, 6EH, 66H, 6FH, 72H, 6DH, 61H, 74H, 74H, 65H, 64H, 20H ; |k...Unformatted |
    DB 64H, 69H, 73H, 6BH, 00H, 0EH, B5H, 4EH, 6FH, 74H, 20H, 61H, 20H, 44H, 4FH, 53H ; |disk...Not a DOS|
    DB 20H, 64H, 69H, 73H, 6BH, 00H, 11H, B1H, 49H, 6EH, 63H, 6FH, 6DH, 70H, 61H, 74H ; | disk...Incompat|
    DB 69H, 62H, 6CH, 65H, 20H, 64H, 69H, 73H, 6BH, 00H, 03H, 01H, 20H, 2DH, 2DH, 00H ; |ible disk... --.|
    DB 07H, 02H, 20H, 64H, 72H, 69H, 76H, 65H, 20H, 00H, 20H, 03H, 41H, 62H, 6FH, 72H ; |.. drive . .Abor|
    DB 74H, 2CH, 20H, 52H, 65H, 74H, 72H, 79H, 20H, 6FH, 72H, 20H, 49H, 67H, 6EH, 6FH ; |t, Retry or Igno|
    DB 72H, 65H, 20H, 28H, 41H, 2FH, 52H, 2FH, 49H, 29H, 3FH, 20H, 00H, 16H, 04H, 41H ; |re (A/R/I)? ...A|
    DB 62H, 6FH, 72H, 74H, 20H, 6FH, 72H, 20H, 52H, 65H, 74H, 72H, 79H, 20H, 28H, 41H ; |bort or Retry (A|
    DB 2FH, 52H, 29H, 3FH, 20H, 00H, 02H, 05H, 41H, 61H, 00H, 02H, 06H, 52H, 72H, 00H ; |/R)? ...Aa...Rr.|
    DB 02H, 07H, 49H, 69H, 00H, 0FH, 08H, 49H, 6EH, 73H, 65H, 72H, 74H, 20H, 64H, 69H ; |..Ii...Insert di|
    DB 73H, 6BH, 20H, 66H, 6FH, 72H, 00H, 1CH, 09H, 50H, 72H, 65H, 73H, 73H, 20H, 52H ; |sk for...Press R|
    DB 45H, 54H, 55H, 52H, 4EH, 20H, 74H, 6FH, 20H, 63H, 6FH, 6EH, 74H, 69H, 6EH, 75H ; |ETURN to continu|
    DB 65H, 2EH, 2EH, 2EH, 20H, 00H, 06H, 0AH, 45H, 72H, 72H, 6FH, 72H, 20H, 00H, 10H ; |e... ...Error ..|
    DB F5H, 43H, 54H, 52H, 4CH, 2DH, 45H, 53H, 43H, 20H, 70H, 72H, 65H, 73H, 73H, 65H ; |.CTRL-ESC presse|
    DB 64H, 00H, 16H, ADH, 44H, 69H, 73H, 6BH, 20H, 6FH, 70H, 65H, 72H, 61H, 74H, 69H ; |d...Disk operati|
    DB 6FH, 6EH, 20H, 61H, 62H, 6FH, 72H, 74H, 65H, 64H, 00H, 14H, 91H, 55H, 6EH, 72H ; |on aborted...Unr|
    DB 65H, 63H, 6FH, 67H, 6EH, 69H, 73H, 65H, 64H, 20H, 63H, 6FH, 6DH, 6DH, 61H, 6EH ; |ecognised comman|
    DB 64H, 00H, 0EH, A1H, 46H, 69H, 6CH, 65H, 20H, 6EH, 6FH, 74H, 20H, 66H, 6FH, 75H ; |d...File not fou|
    DB 6EH, 64H, 00H, 13H, A0H, 44H, 69H, 72H, 65H, 63H, 74H, 6FH, 72H, 79H, 20H, 6EH ; |nd...Directory n|
    DB 6FH, 74H, 20H, 66H, 6FH, 75H, 6EH, 64H, 00H, 0DH, ABH, 49H, 6EH, 76H, 61H, 6CH ; |ot found...Inval|
    DB 69H, 64H, 20H, 64H, 72H, 69H, 76H, 65H, 00H, 0CH, 97H, 49H, 6EH, 76H, 61H, 6CH ; |id drive...Inval|
    DB 69H, 64H, 20H, 64H, 61H, 74H, 65H, 00H, 0CH, 96H, 49H, 6EH, 76H, 61H, 6CH, 69H ; |id date...Invali|
    DB 64H, 20H, 74H, 69H, 6DH, 65H, 00H, 09H, A3H, 44H, 69H, 73H, 6BH, 20H, 66H, 75H ; |d time...Disk fu|
    DB 6CH, 6CH, 00H, 11H, 8EH, 49H, 6EH, 76H, 61H, 6CH, 69H, 64H, 20H, 70H, 61H, 72H ; |ll...Invalid par|
    DB 61H, 6DH, 65H, 74H, 65H, 72H, 00H, 1CH, 8DH, 49H, 6EH, 76H, 61H, 6CH, 69H, 64H ; |ameter...Invalid|
    DB 20H, 6EH, 75H, 6DH, 62H, 65H, 72H, 20H, 6FH, 66H, 20H, 70H, 61H, 72H, 61H, 6DH ; | number of param|
    DB 65H, 74H, 65H, 72H, 73H, 00H, 13H, 8BH, 4EH, 6FH, 20H, 56H, 54H, 2DH, 44H, 4FH ; |eters...No VT-DO|
    DB 53H, 20H, 63H, 61H, 72H, 74H, 72H, 69H, 64H, 67H, 65H, 00H, 1CH, 8AH, 43H, 61H ; |S cartridge...Ca|
    DB 6EH, 6EH, 6FH, 74H, 20H, 63H, 6FH, 70H, 79H, 20H, 66H, 69H, 6CH, 65H, 20H, 6FH ; |nnot copy file o|
    DB 6EH, 74H, 6FH, 20H, 69H, 74H, 73H, 65H, 6CH, 66H, 00H, 0EH, 0BH, 56H, 54H, 2DH ; |nto itself...VT-|
    DB 44H, 4FH, 53H, 20H, 43H, 4CH, 49H, 20H, 31H, 2EH, 32H, 00H, 02H, 0CH, 59H, 79H ; |DOS CLI 1.2...Yy|
    DB 00H, 02H, 0DH, 4EH, 6EH, 00H, 04H, 0EH, 2AH, 2AH, 2AH, 20H, 00H, 10H, 0FH, 43H ; |...Nn...*** ...C|
    DB 75H, 72H, 72H, 65H, 6EH, 74H, 20H, 64H, 61H, 74H, 65H, 20H, 69H, 73H, 20H, 00H ; |urrent date is .|
    DB 0EH, 10H, 45H, 6EH, 74H, 65H, 72H, 20H, 6EH, 65H, 77H, 20H, 64H, 61H, 74H, 65H ; |..Enter new date|
    DB 00H, 03H, 11H, 4DH, 6FH, 6EH, 00H, 03H, 12H, 54H, 75H, 65H, 00H, 03H, 13H, 57H ; |...Mon...Tue...W|
    DB 65H, 64H, 00H, 03H, 14H, 54H, 68H, 75H, 00H, 03H, 15H, 46H, 72H, 69H, 00H, 03H ; |ed...Thu...Fri..|
    DB 16H, 53H, 61H, 74H, 00H, 03H, 17H, 53H, 75H, 6EH, 00H, 17H, 18H, 45H, 72H, 61H ; |.Sat...Sun...Era|
    DB 73H, 65H, 20H, 61H, 6CH, 6CH, 20H, 66H, 69H, 6CH, 65H, 73H, 20H, 28H, 59H, 2FH ; |se all files (Y/|
    DB 4EH, 29H, 3FH, 20H, 00H, 0DH, 19H, 44H, 69H, 72H, 65H, 63H, 74H, 6FH, 72H, 79H ; |N)? ...Directory|
    DB 20H, 6FH, 66H, 20H, 00H, 05H, 1AH, 20H, 62H, 79H, 74H, 65H, 00H, 04H, 1BH, 20H ; | of ... byte... |
    DB 69H, 6EH, 20H, 00H, 05H, 1CH, 20H, 66H, 69H, 6CH, 65H, 00H, 02H, 1DH, 20H, 20H ; |in ... file...  |
    DB 00H, 04H, 1EH, 20H, 20H, 20H, 20H, 00H, 05H, 1FH, 20H, 66H, 72H, 65H, 65H, 00H ; |...    ... free.|
    DB 14H, 20H, 45H, 6EH, 74H, 65H, 72H, 20H, 56H, 54H, 2DH, 44H, 4FH, 53H, 20H, 28H ; |. Enter VT-DOS (|
    DB 59H, 2FH, 4EH, 29H, 3FH, 20H, 00H, 10H, 21H, 44H, 65H, 73H, 74H, 72H, 6FH, 79H ; |Y/N)? ..!Destroy|
    DB 20H, 64H, 61H, 74H, 61H, 20H, 6FH, 6EH, 20H, 00H, 08H, 22H, 20H, 28H, 59H, 2FH ; | data on .." (Y/|
    DB 4EH, 29H, 3FH, 20H, 00H, 27H, 23H, 50H, 72H, 65H, 73H, 73H, 20H, 45H, 53H, 43H ; |N)? .'#Press ESC|
    DB 20H, 74H, 6FH, 20H, 65H, 78H, 69H, 74H, 20H, 43H, 4CH, 49H, 2EH, 0DH, 0AH, 43H ; | to exit CLI...C|
    DB 6FH, 6DH, 6DH, 61H, 6EH, 64H, 73H, 20H, 61H, 72H, 65H, 3AH, 0DH, 0AH, 00H, 10H ; |ommands are:....|
    DB 24H, 43H, 75H, 72H, 72H, 65H, 6EH, 74H, 20H, 74H, 69H, 6DH, 65H, 20H, 69H, 73H ; |$Current time is|
    DB 20H, 00H, 0EH, 25H, 45H, 6EH, 74H, 65H, 72H, 20H, 6EH, 65H, 77H, 20H, 74H, 69H ; | ..%Enter new ti|
    DB 6DH, 65H, 00H, 0AH, 26H, 56H, 6FH, 6CH, 75H, 6DH, 65H, 20H, 69H, 6EH, 20H, 00H ; |me..&Volume in .|
    DB 0CH, 27H, 20H, 68H, 61H, 73H, 20H, 6EH, 6FH, 20H, 6EH, 61H, 6DH, 65H, 00H, 04H ; |.' has no name..|
    DB 28H, 20H, 69H, 73H, 20H, 00H, FFH                                            ; |( is ..|

LCFD5:
    PUSH HL
    PUSH BC
    LD HL,CC3EH
    LD BC,0000H

LCFDD:
    ADD HL,BC
    LD C,(HL)
    INC C
    SCF
    JR Z,CFE9H
    INC HL
    CP (HL)
    INC HL
    JR NZ,CFDDH
    EX DE,HL

LCFE9:
    POP BC
    POP HL
    RET

LCFEC:
    CP 61H
    RET C
    CP 7BH
    JR C,CFFCH
    CP 90H
    RET C
    CP 99H
    RET NC
    SUB 10H
    RET

LCFFC:
    SUB 20H
    RET
    NOP
