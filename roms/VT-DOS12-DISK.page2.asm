; -----------------------------------------------------------------------------
; VT-DOS 1.2 HBF disk ROM page 2
; Source: roms/VT-DOS12-DISK.ROM
; HBF ROM page: 2
; File offset: 2000H
; CPU window: C000H-CFFFH while this HBF ROM page is selected
; HBF ROM page 2 is mapped at C000H-CFFFH only after the card selects page 2.
; ORG: C000H
; Size: 4096 bytes
; Comments: roms/VT-DOS12-DISK.page2.comments.json
; Data ranges: C000H-C00AH, CEACH-CEC1H
; Auto labels: branch and call targets are emitted as Lxxxx.
; -----------------------------------------------------------------------------

ORG C000H


; hbf: MOPS signature and the length-prefixed name DISK. Page 0, not this page, supplies the C00BH initializer and the C00DH function table.
    DB 4DH, 4FH, 50H, 53H, 04H, 44H, 49H, 53H, 4BH, 6FH, DBH                        ; |MOPS.DISKo.|
    NOP
    NOP
    LD BC,DBC1H
    PUSH HL
    PUSH BC
    RES 2,C
    RES 7,C
    EX DE,HL
    CALL DB2EH
    DB DDH, D6H, E1H                                                                ; |...|
    JR NZ,C036H
    BIT 0,L
    JR NZ,C035H
    EXX
    PUSH IX
    POP HL
    LD BC,0008H
    ADD HL,BC
    EX DE,HL
    LD HL,D49EH
    LD BC,000BH
    LDIR
    EXX

LC035:
    XOR A

LC036:
    POP HL
    RET
    EX DE,HL
    XOR A
    LD C,81H
    CALL DB2EH
    POP DE
    SUB C0H
    OR C
    LD A,AAH
    RET NZ
    LD BC,001EH
    ADD HL,BC
    LD DE,(D06FH)
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    EX DE,HL
    CALL C05CH

LC056:
    LD A,(HL)
    LDI
    OR A
    JR NZ,C056H

LC05C:
    LD HL,D4ABH
    RET
    LD A,B
    CALL DB2EH
    SUB L
    RET NZ
    LD B,A
    OR A
    LD A,ABH
    RET Z
    XOR A
    LD C,80H
    LD DE,DB6EH
    CALL DB2EH
    OUT (D6H),A
    JR C05CH
    BIT 3,(IX+06H)
    LD A,A8H
    RET NZ
    LD A,10H
    EX AF,AF'
    PUSH HL
    PUSH BC
    CALL C090H
    POP BC
    POP HL
    CALL NZ,C161H
    RET NZ
    EX AF,AF'
    JR C0AAH

LC090:
    LD HL,0008H
    ADD HL,DE
    LD A,2EH
    CP (HL)
    RET NZ
    INC HL
    CP (HL)
    LD A,20H
    JR Z,C0A0H
    CP (HL)
    RET NZ

LC0A0:
    LD B,09H

LC0A2:
    INC HL
    CP (HL)
    RET NZ
    DJNZ C0A2H
    RET
    LD A,80H

LC0AA:
    LD (D09CH),A
    CALL DB2EH
    LD C,A
    SUB C0H
    OR A
    PUSH BC
    JR C0C7H
    LD (IY+1CH),80H
    CALL DB2EH
    AND (HL)
    SUB C0H
    PUSH BC
    CALL DB2EH
    INC B
    DB DAH                                                                          ; |.|

LC0C7:
    CALL C21CH
    JR C,C0D9H
    BIT 7,(IY+1CH)
    LD A,A1H
    CALL Z,C24EH
    JR NZ,C112H
    JR C0F7H
    BIT 7,(IY+1CH)
    JR NZ,C130H
    BIT 2,A
    JR NZ,C0F0H
    AND 10H
    LD A,9CH
    JR NZ,C131H
    BIT 4,(IY+1CH)
    LD A,9BH
    DB DAH                                                                          ; |.|

LC0F0:
    LD A,9DH
    CALL Z,C3BDH
    JR NZ,C131H

LC0F7:
    LD A,(IX+08H)
    CP 2EH
    LD A,A7H
    JR Z,C152H
    LD A,(D09CH)
    AND 10H
    LD B,A
    LD C,A
    SET 5,A
    JR Z,C116H
    SET 2,(IY+07H)
    CALL C285H

LC112:
    JR NZ,C152H
    LD A,10H

LC116:
    PUSH AF
    CALL DB2EH
    EI
    EXX
    PUSH IX
    POP HL
    POP AF
    PUSH BC
    LD B,A
    LD A,(IX+06H)
    AND 08H
    OR B
    LD BC,0008H
    ADD HL,BC
    POP BC
    CALL C1A7H

LC130:
    XOR A

LC131:
    POP HL
    PUSH HL
    PUSH AF
    PUSH DE
    PUSH IX
    POP DE
    LD (HL),FEH
    INC HL
    INC DE
    LD BC,0007H
    EX DE,HL
    LDIR
    POP HL
    LD A,(HL)
    CP 05H
    JR NZ,C14AH
    LD A,E5H

LC14A:
    LD (DE),A
    INC HL
    INC DE
    LD C,1FH
    LDIR
    POP AF

LC152:
    CALL DB2EH
    JP EBD6H
    LDIR
    PUSH IX
    POP DE
    POP BC
    JP C05CH

LC161:
    BIT 3,(IX+06H)
    JR NZ,C198H
    PUSH BC
    PUSH IX
    EX (SP),HL
    LD BC,0008H
    ADD HL,BC
    LD B,08H
    LD A,20H
    CALL C17EH
    LD B,03H
    CALL Z,C17EH
    POP HL
    POP BC
    RET

LC17E:
    CP (HL)
    JR Z,C1A3H

LC181:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,C19CH
    CALL DB2EH
    LD D,L
    RET C
    JR Z,C1A3H
    CP 3FH
    JR Z,C1A3H
    CP 2AH
    JR Z,C1A3H
    DJNZ C181H

LC198:
    XOR A
    RET

LC19A:
    LD A,(HL)
    INC HL

LC19C:
    SUB 20H
    JR NZ,C1A3H
    DJNZ C19AH
    RET

LC1A3:
    LD A,A9H
    OR A
    RET

LC1A7:
    PUSH DE
    PUSH BC
    LD B,A
    LD A,(HL)
    CP E5H
    JR NZ,C1B1H
    LD A,05H

LC1B1:
    LD (DE),A
    LD A,B
    INC HL
    INC DE
    LD BC,000AH
    LDIR
    LD (DE),A
    EX DE,HL
    LD B,14H

LC1BE:
    INC HL
    LD (HL),C
    DJNZ C1BEH
    POP DE
    BIT 7,D
    JR Z,C1C9H
    LD D,B
    LD E,B

LC1C9:
    LD BC,FFFCH
    ADD HL,BC
    LD (HL),D
    DEC HL
    LD (HL),E
    POP DE

LC1D1:
    EX AF,AF'

LC1D2:
    PUSH HL
    PUSH BC
    PUSH DE
    LD HL,0016H
    ADD HL,DE
    CALL C1E5H
    CALL DB2EH
    LD B,D
    DB D4H                                                                          ; |.|

LC1E1:
    POP DE
    POP BC
    POP HL
    RET

LC1E5:
    PUSH HL
    CALL DB2EH
    AND C
    RST 28H
    LD A,L
    ADD A,A
    ADD A,A
    LD B,03H

LC1F0:
    ADD A,A
    RL H
    DJNZ C1F0H
    SRL D
    ADD A,D
    LD C,H
    POP HL
    LD (HL),A
    INC HL
    LD (HL),C
    INC HL
    PUSH HL
    CALL DB2EH
    SCF
    RST 28H
    LD BC,F844H
    ADD HL,BC
    LD A,D
    OR A
    RRA
    RRA
    RRA
    RRA
    RL L
    ADD A,E
    LD C,L
    POP HL
    LD (HL),A
    INC HL
    LD (HL),C
    RET

LC217:
    CALL DB2EH
    INC B
    DB DAH                                                                          ; |.|

LC21C:
    JR C,C238H
    PUSH IX
    EX (SP),HL
    LD BC,0008H
    ADD HL,BC
    CALL DB2EH
    AND L
    EXX
    POP HL
    RET C
    JR NZ,C217H
    BIT 0,(IY+1CH)
    CALL Z,C23AH
    OR A
    JR NZ,C217H
    XOR A
    RET

LC23A:
    PUSH HL
    PUSH DE
    CALL DB2EH
    JP 62D6H
    LD L,E
    ADD HL,BC
    EX DE,HL
    LDIR
    SET 0,(IY+1CH)
    POP DE
    POP HL
    RET

LC24E:
    SET 2,(IY+22H)
    BIT 0,(IY+1CH)
    JR Z,C266H
    PUSH HL
    CALL DB2EH
    JP 62D6H
    LD L,E
    ADD HL,BC
    LDIR
    POP HL
    XOR A
    RET

LC266:
    LD DE,(D071H)
    BIT 7,D
    LD A,A4H
    RET NZ
    CALL C285H
    RET NZ
    PUSH BC
    CALL DB2EH
    RRCA
    PUSH DE
    POP BC
    LD (D073H),BC
    CALL DB2EH
    INC B
    JP C,C9AFH

LC285:
    LD BC,0001H
    SET 1,(IY+07H)

LC28C:
    PUSH DE
    LD DE,0001H
    CALL DB2EH
    RET Z
    CALL NC,11D5H
    DB 01H, 00H                                                                     ; |..|

LC299:
    PUSH BC
    PUSH DE

LC29B:
    INC DE
    PUSH HL
    LD BC,0016H
    ADD HL,BC
    LD A,(HL)
    INC HL
    LD H,(HL)
    LD L,A
    SBC HL,DE
    POP HL
    JR Z,C2E3H
    PUSH DE
    CALL DB2EH
    RET Z
    CALL NC,B37AH
    POP DE
    JR NZ,C29BH
    LD B,D
    LD C,E
    POP DE
    PUSH BC
    CALL DB2EH
    RRCA
    PUSH DE
    POP DE
    PUSH DE
    LD B,D
    LD C,E
    BIT 1,(IY+07H)
    CALL NZ,C2FDH
    POP DE
    POP BC
    DEC BC
    LD A,B
    OR C
    JR NZ,C299H
    LD BC,FFFFH
    CALL DB2EH
    LD (DE),A
    PUSH DE
    LD DE,0001H
    CALL DB2EH
    RET Z
    CALL NC,18AFH
    LD A,(BC)

LC2E3:
    POP DE
    POP DE
    LD DE,0001H
    CALL C364H
    LD A,A3H
    POP BC
    PUSH AF
    PUSH DE
    LD DE,0001H
    CALL DB2EH
    LD (DE),A
    PUSH DE
    POP BC
    POP AF
    OR A
    POP DE
    RET

LC2FD:
    PUSH BC
    LD D,B
    LD E,C
    PUSH HL
    LD BC,000AH
    ADD HL,BC
    LD B,(HL)
    POP HL
    LD A,B
    CALL DB2EH
    POP AF
    CALL NC,3C78H
    POP BC
    PUSH HL
    EX (SP),IX
    PUSH BC
    LD B,A

LC315:
    PUSH BC
    JR NZ,C31DH
    CALL DB2EH
    EX DE,HL
    DB D2H                                                                          ; |.|

LC31D:
    LD B,00H
    CALL DB2EH
    AND H
    OUT (E5H),A
    LD BC,000BH
    ADD HL,BC
    XOR A

LC32A:
    LD (HL),A
    INC HL
    LD (HL),A
    INC HL
    DJNZ C32AH
    CALL DB2EH
    LD B,D
    CALL NC,C1E1H
    DEC DE
    XOR A
    DJNZ C315H
    POP BC
    BIT 2,(IY+07H)
    JR Z,C360H
    LD DE,000BH
    ADD HL,DE
    EX DE,HL
    LD HL,DB63H
    LD A,12H
    CALL C1A7H
    LD HL,0020H
    ADD HL,DE
    EX DE,HL
    LD HL,DB62H
    LD BC,(D06FH)
    LD A,12H
    CALL C1A7H

LC360:
    EX (SP),IX
    POP HL
    RET

LC364:
    PUSH DE
    CALL DB2EH
    RET Z
    CALL NC,D5C1H
    LD D,B
    LD E,C
    LD BC,0000H
    CALL DB2EH
    LD (DE),A
    PUSH DE
    POP DE
    LD A,D
    OR E
    RET Z
    BIT 7,D
    JR Z,C364H
    RET
    LD B,H
    LD C,L
    CALL DB2EH
    LD A,C
    SUB C0H
    BIT 4,A
    JR Z,C3B8H
    CALL C410H
    RET C
    CALL DB2EH
    ADD A,L
    EXX
    CALL DB2EH
    SBC A,D9H
    JR Z,C3A8H
    CALL C410H
    LD A,9EH
    RET NZ
    CALL DB2EH
    INC B
    JP C,F130H

LC3A8:
    PUSH IX
    POP DE
    LD H,B
    LD L,C
    CALL DB2EH
    LD A,C
    SUB C0H
    CALL DB2EH
    XOR (HL)
    DB D2H                                                                          ; |.|

LC3B8:
    CALL DB2EH
    EX (SP),HL
    PUSH DE

LC3BD:
    CALL DB2EH
    EI
    EXX
    PUSH HL
    LD HL,000BH
    ADD HL,DE
    LD A,(HL)
    BIT 0,A
    LD A,9AH
    POP HL
    RET NZ
    PUSH DE
    PUSH HL
    LD HL,001AH
    ADD HL,DE
    LD E,(HL)
    INC HL
    LD D,(HL)
    LD A,(DFF3H)
    OR A
    JR Z,C3ECH
    LD A,(D06EH)
    CP (IY+1FH)
    JR NZ,C3ECH
    LD HL,(D0A0H)
    SBC HL,DE
    JR Z,C40AH

LC3EC:
    POP HL
    LD A,D
    OR E
    CALL NZ,C364H
    POP DE
    CALL DB2EH
    EI
    EXX
    PUSH HL
    LD HL,000CH
    ADD HL,DE
    LD A,(DE)
    LD (HL),A
    LD A,E5H
    LD (DE),A
    CALL DB2EH
    LD B,D
    CALL NC,AFE1H
    RET

LC40A:
    LD A,8AH
    POP HL
    POP DE
    OR A
    RET

LC410:
    PUSH HL
    PUSH BC
    LD HL,DB63H
    CALL DB2EH
    AND L
    EXX
    JR Z,C424H
    LD HL,DB62H
    CALL DB2EH
    AND L
    EXX

LC424:
    POP BC
    POP HL
    LD A,A7H
    RET
    PUSH HL
    CALL DB2EH
    LD A,C
    SUB 20H
    LD E,A
    BIT 4,A
    JR Z,C43AH
    CALL DB2EH
    XOR (HL)
    DB D2H                                                                          ; |.|

LC43A:
    CALL C410H
    JR C,C490H
    PUSH BC
    EXX
    PUSH IX
    POP HL
    LD BC,0007H
    ADD HL,BC
    EX DE,HL
    POP HL
    ADD HL,BC
    LD A,(DE)
    CP (HL)
    LD A,ABH
    JR NZ,C490H
    LD B,0BH

LC453:
    INC DE
    INC HL
    LD A,(HL)
    CP 3FH
    JR Z,C45BH
    LD (DE),A

LC45B:
    DJNZ C453H
    EXX
    CALL C161H
    JR NZ,C490H
    CALL DB2EH
    SBC A,D9H
    CALL C21CH
    POP HL
    LD A,A2H
    RET C
    PUSH IX
    POP DE
    CALL DB2EH
    LD A,C
    SUB C0H
    PUSH IX
    POP HL
    LD BC,0008H
    ADD HL,BC
    LD A,(HL)
    CP E5H
    JR NZ,C486H
    LD A,05H

LC486:
    LD (DE),A
    INC HL
    INC DE
    LD C,0AH
    JP C594H

LC48E:
    LD A,A8H

LC490:
    POP HL
    RET
    PUSH HL
    CALL DB2EH
    LD A,C
    SUB 20H
    OR DDH
    RLC (HL)
    LD E,(HL)
    JR NZ,C48EH
    BIT 3,A
    JR NZ,C48EH
    PUSH BC
    LD C,81H
    BIT 4,A
    JR Z,C4C1H
    SET 2,C
    PUSH HL
    LD HL,001AH
    ADD HL,DE
    LD A,(HL)
    LD (IY+1DH),A
    INC HL
    LD A,(HL)
    LD (IY+1EH),A
    POP HL
    CALL DB2EH
    XOR (HL)
    DB D2H                                                                          ; |.|

LC4C1:
    CALL C410H
    POP DE
    JR C,C490H
    LD (IY+1CH),00H
    LD A,(IX+07H)
    EX AF,AF'
    CALL DB2EH
    DB DDH, D6H, 20H                                                                ; |.. |
    CP D
    OR C
    LD A,AAH
    JR NZ,C490H
    EX AF,AF'
    CP (IX+07H)
    LD A,ABH
    JR NZ,C490H
    BIT 6,(IY+1CH)
    LD A,9FH
    JR NZ,C490H
    CALL DB2EH
    SBC A,D9H
    CALL C21CH
    LD A,A2H
    JR C,C490H
    CALL C24EH
    JR NZ,C490H
    CALL C23AH
    POP HL
    PUSH IX
    POP DE
    CALL DB2EH
    LD A,C
    SUB C0H
    BIT 4,A
    PUSH AF
    JR Z,C513H
    CALL DB2EH
    XOR (HL)
    DB D2H                                                                          ; |.|

LC513:
    PUSH HL
    LD HL,D46EH
    PUSH HL
    PUSH DE
    EX DE,HL
    LD BC,0020H
    LDIR
    POP HL
    LD (HL),E5H
    CALL DB2EH
    LD B,D
    CALL NC,E1D1H
    PUSH DE
    CALL C24EH
    CALL DB2EH
    EI
    EXX
    EX (SP),HL
    LD BC,0020H
    PUSH DE
    LDIR
    CALL DB2EH
    LD B,D
    CALL NC,E1D1H
    POP AF
    LD A,B
    RET Z
    LD BC,(D06FH)
    PUSH BC
    LD IX,DB5AH
    CALL DB2EH
    ADD A,L
    EXX
    CALL DB2EH
    SBC A,D9H
    CALL C21CH
    POP BC
    RET NC
    AND 10H
    RET Z
    LD HL,001AH
    ADD HL,DE
    LD (HL),C
    INC HL
    LD (HL),B
    JR C596H
    PUSH DE
    CALL DB2EH
    LD A,C
    SUB E1H
    RET NZ
    LD BC,0013H
    ADD HL,BC
    EX DE,HL
    LD C,0BH
    ADD HL,BC
    DEC B
    BIT 3,A
    JR NZ,C584H
    LD B,FDH
    BIT 4,A
    JR NZ,C584H
    LD B,D8H

LC584:
    LD A,(DE)
    LD C,A
    XOR (HL)
    AND B
    LD A,A8H
    RET NZ
    LD (HL),C
    LD BC,000BH
    ADD HL,BC
    EX DE,HL
    ADD HL,BC
    LD C,04H

LC594:
    LDIR

LC596:
    CALL DB2EH
    LD B,D
    CALL NC,C9AFH

LC59D:
    LD A,A1H
    RET
    LD HL,C66FH
    PUSH HL
    LD E,(IX+1FH)
    LD D,(IX+20H)
    BIT 6,D
    RET Z
    RES 6,D
    RES 5,D
    LD (D06FH),DE
    LD C,(IX+07H)
    LD B,01H
    CALL DB2EH
    CP L
    RET NC
    OR A
    RET NZ

LC5C1:
    CALL DB2EH
    SBC A,D9H
    CALL C21CH
    JR NC,C59DH
    AND 07H
    LD A,A8H
    RET NZ
    CALL C1D2H
    PUSH DE
    PUSH IX
    EX (SP),HL
    LD BC,0017H
    ADD HL,BC
    EX DE,HL
    LD C,0BH
    ADD HL,BC
    SET 5,(HL)
    LD C,0FH
    ADD HL,BC
    LD A,(IX+21H)
    LD (HL),A
    INC HL
    LD A,(IX+22H)
    LD (HL),A
    INC HL
    EX DE,HL
    LD C,04H
    LDIR
    POP HL
    CALL DB2EH
    LD E,L
    CALL NC,CDD1H
    LD L,DBH
    EI
    EXX
    LD B,00H

LC601:
    PUSH BC
    PUSH IX
    POP HL
    LD BC,0017H
    ADD HL,BC
    EX DE,HL
    LD C,1CH
    ADD HL,BC
    LD C,04H
    PUSH HL
    LDIR
    POP HL
    DEC HL
    LD B,(HL)
    DEC HL
    LD C,(HL)
    PUSH BC
    LD (D0A0H),BC
    LD A,(D06EH)
    LD (D09FH),A
    DEC HL
    DEC HL
    LDI
    LDI
    DEC HL
    DEC HL
    DEC HL
    DEC HL
    LDI
    LDI
    EX DE,HL
    POP BC
    LD DE,(D06FH)
    RES 6,D
    RES 5,D
    POP AF
    BIT 0,A
    JR Z,C641H
    SET 5,D

LC641:
    LD (HL),E
    INC HL
    LD (HL),D
    INC HL
    LD (HL),C
    INC HL
    LD (HL),B
    INC HL
    LD (HL),C
    INC HL
    LD (HL),B
    INC HL
    XOR A
    LD (HL),A
    INC HL
    LD (HL),A
    RET
    LD BC,C66FH
    PUSH BC
    CALL DB2EH
    LD A,C
    SUB C0H
    LD B,A
    AND 18H
    LD A,A8H
    RET NZ
    CALL C601H
    INC HL
    INC HL
    LD (HL),A
    INC HL
    LD (HL),A
    INC HL
    LD (HL),A
    INC HL
    LD (HL),A
    RET
    PUSH IX
    POP DE
    RET
    INC A
    LD (D087H),A
    LD (D096H),DE
    PUSH HL
    PUSH IX
    POP BC
    LD (D088H),BC
    LD A,B
    ADD A,H
    JR C,C688H
    ADD A,H

LC688:
    LD A,A6H
    JR C,C6ACH
    LD C,L
    CALL DB2EH
    OR A
    RET NC
    OR A
    JR NZ,C6ACH
    CALL DB2EH
    LD E,L
    CALL NC,2ECDH
    IN A,(6DH)
    CALL NC,C5C1H
    CALL C7CFH
    CALL DB2EH
    LD L,L
    CALL NC,AFE1H
    DB 11H                                                                          ; |.|

LC6AC:
    POP HL
    LD H,00H
    LD IX,(D088H)
    LD DE,(D096H)
    RET
    INC A
    INC A
    LD (D087H),A
    XOR A
    LD (D08CH),A
    LD (D08DH),A
    LD (D08AH),BC
    PUSH DE
    EX (SP),IX
    POP HL
    LD (D088H),HL
    CALL C6FEH
    LD BC,(D08CH)
    LD HL,(D088H)
    PUSH HL
    EX (SP),IX
    POP DE
    EX AF,AF'
    PUSH HL
    LD HL,0028H
    ADD HL,DE
    LD A,(HL)
    ADD A,C
    LD (HL),A
    INC HL
    LD A,(HL)
    ADC A,B
    LD (HL),A
    JR NC,C6F2H
    INC HL
    INC (HL)
    JR NZ,C6F2H
    INC HL
    INC (HL)

LC6F2:
    POP HL
    EX AF,AF'
    OR A
    RET NZ
    BIT 7,(IY+07H)
    RET Z
    LD A,ECH
    RET

LC6FE:
    LD A,B
    OR C
    RET Z
    DEC BC
    ADD HL,BC
    LD A,A6H
    RET C
    LD C,(IX+07H)
    LD B,01H
    CALL DB2EH
    CP L
    RET NC
    OR A
    RET NZ
    CALL C9D5H
    LD DE,(D08EH)
    CALL Z,CB17H
    LD (D094H),DE
    RET NZ
    LD BC,(D091H)
    LD A,B
    OR C
    CALL NZ,C7A6H
    RET NZ

LC72B:
    LD C,(IY+0BH)
    SRL C
    JR Z,C78AH
    CALL C8BCH
    RET NZ
    SUB (IY+10H)
    LD B,A
    LD DE,(D094H)
    CALL C8F2H

LC741:
    LD A,B
    ADD A,(IY+13H)
    LD B,A
    CP C
    JR NC,C76AH
    PUSH DE
    CALL DB2EH
    RET Z
    CALL NC,B7E3H
    INC HL
    SBC HL,DE
    JR NZ,C761H
    INC (IY+0EH)
    JR NZ,C75EH
    INC (IY+0FH)

LC75E:
    POP HL
    JR C741H

LC761:
    ADD HL,DE
    EX DE,HL
    DEC DE
    POP HL
    LD A,(D093H)
    JR C770H

LC76A:
    LD A,(D093H)
    SUB B
    ADD A,C
    LD B,C

LC770:
    DEC A
    LD (D090H),A
    LD (D094H),DE
    PUSH BC
    CALL C7CFH
    POP BC
    SLA B
    LD C,00H
    CALL C9C3H
    SET 6,(IY+07H)
    JR C72BH

LC78A:
    LD BC,0000H
    CALL C7A6H
    RET NZ
    LD DE,(D094H)
    LD (IX+23H),E
    LD (IX+24H),D
    LD DE,(D08EH)
    LD (IX+25H),E
    LD (IX+26H),D
    RET

LC7A6:
    PUSH HL
    LD HL,0200H
    OR A
    SBC HL,BC
    EX DE,HL
    LD HL,(D08AH)
    SBC HL,DE
    JR NC,C7B8H
    ADD HL,DE
    LD E,L
    LD D,H

LC7B8:
    LD A,D
    OR E
    POP HL
    RET Z
    CALL C8BCH
    RET NZ
    CALL C8F2H
    CALL C905H
    SET 6,(IY+07H)
    CALL C9C3H
    XOR A
    RET

LC7CF:
    XOR A
    CP B
    RET Z
    PUSH HL
    LD DE,(D088H)
    LD A,(DFF7H)
    OR A
    LD HL,FE00H
    JR NZ,C7E3H
    LD HL,BE00H

LC7E3:
    SBC HL,DE
    JR C,C84FH
    LD A,H
    SRL A

LC7EA:
    INC A
    CP B
    JR C,C7EFH
    LD A,B

LC7EF:
    LD C,A
    POP HL
    SUB B
    NEG
    LD B,A
    PUSH BC
    LD B,C
    LD C,FFH
    PUSH BC

LC7FA:
    PUSH BC
    PUSH DE
    PUSH DE
    EX (SP),IX
    LD A,04H
    BIT 0,(IY+07H)
    JR NZ,C80CH
    CALL DB2EH
    ADC A,L
    DB D3H                                                                          ; |.|

LC80C:
    LD DE,(D096H)
    PUSH DE
    DEC C
    INC C
    JR Z,C81AH
    CALL DB2EH
    DJNZ C7EAH

LC81A:
    EX AF,AF'
    POP DE
    POP IX
    PUSH HL
    LD L,B
    LD H,00H
    ADD HL,DE
    LD (D096H),HL
    POP HL
    POP DE
    LD A,D
    ADD A,B
    ADD A,B
    LD D,A
    LD A,B
    POP BC
    SUB B
    NEG
    LD B,A
    EX AF,AF'
    JR Z,C840H
    CALL DB2EH
    XOR B
    JP C,0228H
    LD C,00H
    JR C7FAH

LC840:
    POP BC
    CALL C865H
    POP BC
    LD A,(IY+09H)
    ADD A,C
    ADD A,C
    LD (IY+09H),A

LC84D:
    JR C7CFH

LC84F:
    POP HL
    DEC B
    PUSH BC
    LD DE,0200H
    LD B,E
    LD C,E
    CALL C905H
    POP BC
    INC (IY+16H)
    JR NZ,C863H
    INC (IY+17H)

LC863:
    JR C84DH

LC865:
    CALL DB2EH
    ADD A,E
    CALL NC,D9C5H
    CALL DB2EH
    OR E
    CALL NC,4628H
    LD BC,0004H
    ADD HL,BC
    LD E,(HL)
    INC HL
    LD D,(HL)
    EX DE,HL
    LD BC,(D096H)
    SBC HL,BC
    POP BC
    LD A,H
    INC A
    JR NZ,C89DH
    LD A,L
    ADD A,B
    JR NC,C89DH
    DEC DE
    DEC DE
    EX DE,HL
    BIT 0,(IY+07H)
    JR NZ,C8A0H
    DEC HL
    LD (HL),00H
    DEC HL
    DEC HL
    CALL DB2EH
    EX DE,HL
    DB D2H                                                                          ; |.|

LC89D:
    EXX
    JR C86AH

LC8A0:
    BIT 7,(HL)
    JR Z,C89DH
    EXX
    PUSH DE
    EXX
    LD BC,0008H
    ADD HL,BC
    EX (SP),HL
    LD D,E
    LD E,B
    ADD HL,DE
    ADD HL,DE
    EX DE,HL
    POP HL
    LD BC,0200H
    LDIR
    JR C89DH
    EXX
    POP BC
    RET

LC8BC:
    XOR A
    BIT 6,(IY+07H)
    RET Z
    RES 6,(IY+07H)
    LD A,(D090H)
    INC A
    CP (IY+13H)
    JR NZ,C8EDH
    PUSH DE
    LD DE,(D08EH)
    INC DE
    LD (D08EH),DE
    LD DE,(D094H)
    CALL DB2EH
    RET Z
    CALL NC,53EDH
    SUB H
    RET NC
    BIT 7,D
    POP DE
    LD A,A5H
    RET NZ
    XOR A

LC8ED:
    LD (D090H),A
    XOR A
    RET

LC8F2:
    PUSH DE
    LD A,(D090H)
    LD DE,(D094H)
    CALL DB2EH
    POP AF
    CALL NC,53EDH
    SUB (HL)
    RET NC
    POP DE
    RET

LC905:
    PUSH HL
    EX (SP),IX
    PUSH DE
    PUSH BC
    BIT 0,(IY+07H)
    JR NZ,C935H
    BIT 1,D
    JR NZ,C938H
    BIT 1,(IY+07H)
    JR NZ,C935H
    PUSH DE
    LD HL,(D08AH)
    LD DE,(D098H)
    OR A
    SBC HL,DE
    POP DE
    JR C,C932H
    LD A,B
    OR C
    JR NZ,C935H
    SBC HL,DE
    JR C,C938H
    JR C935H

LC932:
    ADD HL,BC
    JR NC,C938H

LC935:
    LD B,03H
    DB 21H                                                                          ; |!|

LC938:
    LD B,02H
    LD DE,(D096H)
    CALL DB2EH
    AND H
    OUT (D1H),A
    ADD HL,DE
    LD BC,000BH
    ADD HL,BC
    POP BC
    PUSH DE
    PUSH BC
    LD DE,(D088H)
    PUSH DE
    PUSH BC
    PUSH HL
    LD A,(DFF7H)
    OR A
    LD HL,FFFFH
    JR NZ,C95FH
    LD HL,BFFFH

LC95F:
    SBC HL,DE
    JR C,C986H
    DEC BC
    SBC HL,BC
    JR NC,C96BH
    ADD HL,BC
    LD B,H
    LD C,L

LC96B:
    INC BC
    POP HL
    PUSH BC
    BIT 0,(IY+07H)
    JR NZ,C975H
    EX DE,HL

LC975:
    LDIR
    JR NZ,C97AH
    EX DE,HL

LC97A:
    POP BC
    EX (SP),HL
    OR A
    SBC HL,BC
    LD B,H
    LD C,L
    POP HL
    JR Z,C9A0H
    PUSH BC
    PUSH HL

LC986:
    POP HL
    RES 7,D
    RES 6,D
    DI
    LD A,F8H
    OUT (02H),A
    BIT 0,(IY+07H)
    JR NZ,C997H
    EX DE,HL

LC997:
    LDIR
    LD A,(DFFFH)
    OUT (02H),A
    EI
    POP BC

LC9A0:
    POP HL
    POP BC
    ADD HL,BC
    LD (D088H),HL
    BIT 0,(IY+07H)
    JR NZ,C9B1H
    CALL DB2EH

LC9AF:
    LD B,D
    DB D4H                                                                          ; |.|

LC9B1:
    POP HL
    ADD HL,BC
    BIT 1,H
    JR Z,C9BFH
    LD HL,(D069H)
    CALL DB2EH
    RET PE
    DB D2H                                                                          ; |.|

LC9BF:
    EX (SP),IX
    POP HL
    RET

LC9C3:
    PUSH HL
    LD HL,(D08CH)
    ADD HL,BC
    LD (D08CH),HL
    LD HL,(D08AH)
    SBC HL,BC
    LD (D08AH),HL
    POP HL
    RET

LC9D5:
    PUSH HL
    LD C,(IX+28H)
    LD B,(IX+29H)
    LD E,(IX+2AH)
    LD D,(IX+2BH)
    PUSH DE
    PUSH BC
    CALL CAE1H
    JR NZ,CA07H
    LD (D08EH),DE
    LD (D090H),A
    LD (D091H),BC
    POP HL
    LD DE,(D08AH)
    DEC DE
    ADD HL,DE
    EX (SP),HL
    POP BC
    EX DE,HL
    EX (SP),HL
    JR NC,CA02H
    INC DE

LCA02:
    PUSH DE
    PUSH BC
    CALL CAE1H

LCA07:
    JR NZ,CA6FH
    PUSH DE
    LD C,(IX+17H)
    LD B,(IX+18H)
    LD E,(IX+19H)
    LD D,(IX+1AH)
    LD A,B
    OR C
    DEC BC
    JR NZ,CA1EH
    OR D
    OR E
    DEC DE

LCA1E:
    PUSH DE
    PUSH BC
    CALL NZ,CAE1H
    EXX
    POP HL
    POP DE
    EXX
    POP BC
    JR NZ,CA6FH
    EXX
    POP BC
    XOR A
    SBC HL,BC
    EX (SP),HL
    EX DE,HL
    SBC HL,DE
    EXX
    BIT 7,D
    EX AF,AF'
    BIT 0,(IY+07H)
    JP NZ,CAC2H
    BIT 5,(IX+20H)
    LD A,9AH
    JR NZ,CA70H
    SET 6,(IX+20H)
    PUSH HL
    LD H,B
    LD L,C
    INC DE
    OR A
    SBC HL,DE
    LD C,L
    LD B,H
    INC BC
    POP HL
    JR C,CA85H
    CALL C28CH
    JR NZ,CA70H
    DEC DE
    BIT 7,D
    JR NZ,CA73H
    PUSH BC
    CALL CB17H
    POP BC
    JR NZ,CA70H
    CALL DB2EH
    LD (DE),A
    PUSH DE
    JR CA85H

LCA6F:
    POP DE

LCA70:
    POP DE
    POP DE
    RET

LCA73:
    LD (IX+21H),C
    LD (IX+22H),B
    LD (IX+23H),C
    LD (IX+24H),B
    LD (IX+25H),A
    LD (IX+26H),A

LCA85:
    EXX
    EX AF,AF'
    JR NZ,CA8BH
    JR NC,CA9DH

LCA8B:
    INC BC
    LD A,B
    OR C
    JR NZ,CA91H
    INC DE

LCA91:
    LD (IX+17H),C
    LD (IX+18H),B
    LD (IX+19H),E
    LD (IX+1AH),D

LCA9D:
    POP BC
    POP DE
    LD A,B
    OR C
    DEC BC
    JR NZ,CAA5H
    DEC HL

LCAA5:
    BIT 7,H
    JR Z,CAB9H
    LD A,H
    AND L
    INC A
    LD HL,FFFFH
    JR NZ,CABCH
    INC HL
    SBC HL,BC
    JR NZ,CABCH
    DEC HL
    JR CABCH

LCAB9:
    LD HL,0000H

LCABC:
    LD (D098H),HL
    EXX
    XOR A
    RET

LCAC2:
    EX AF,AF'
    POP DE
    EX (SP),HL
    JR NZ,CACEH
    JR NC,CADEH
    EXX
    LD A,H
    AND L
    INC A
    EXX

LCACE:
    LD A,ECH
    JR NZ,CADEH
    ADD HL,DE
    JR NC,CADEH
    INC HL
    LD (D08AH),HL
    SET 7,(IY+07H)
    XOR A

LCADE:
    POP HL
    OR A
    RET

LCAE1:
    PUSH HL
    LD A,B
    SRL D
    RR E
    RRA
    LD B,00H
    RL B
    PUSH BC
    LD BC,000AH
    ADD HL,BC
    PUSH AF
    AND (HL)
    LD C,A
    LD A,(HL)
    INC A
    LD (D093H),A
    POP AF
    INC HL
    LD B,(HL)
    INC B
    JR CB04H

LCAFF:
    SRL D
    RR E
    RRA

LCB04:
    DJNZ CAFFH
    INC D
    DEC D
    JR NZ,CB12H
    BIT 7,E
    JR NZ,CB12H
    LD D,E
    LD E,A
    LD A,C
    DB 21H                                                                          ; |!|

LCB12:
    LD A,A5H
    POP BC
    POP HL
    RET

LCB17:
    PUSH HL
    EX DE,HL
    LD E,(IX+23H)
    LD D,(IX+24H)
    LD A,D
    OR E
    JR Z,CB44H
    LD C,(IX+25H)
    LD B,(IX+26H)
    SBC HL,BC
    JR NC,CB34H
    ADD HL,BC
    LD E,(IX+21H)
    LD D,(IX+22H)

LCB34:
    EX (SP),HL
    POP BC

LCB36:
    LD A,B
    OR C
    RET Z
    DEC BC
    CALL DB2EH
    RET Z
    CALL NC,7ACBH
    JR Z,CB36H
    DB 3EH                                                                          ; |>|

LCB44:
    POP HL
    LD A,A5H
    OR A
    RET

LCB49:
    PUSH IY
    LD IY,D080H
    LD (IY-4AH),6EH
    LD (IY-4FH),A
    LD (IY-4DH),C
    LD (IY-4BH),E
    LD (IY-4CH),D
    PUSH DE
    PUSH IX
    POP DE
    PUSH DE
    PUSH BC
    LD C,(IY-55H)
    OR A
    JR Z,CB86H
    SUB 07H
    JP C,CBFCH
    JR Z,CB81H
    DEC A
    JR Z,CBB3H

LCB75:
    LD A,80H
    LD L,00H

LCB79:
    POP BC

LCB7A:
    POP IX
    POP DE
    POP IY
    OR A
    RET

LCB81:
    POP BC
    LD B,A
    LD L,A
    JR CB7AH

LCB86:
    LD A,B
    AND 0FH
    PUSH IY

LCB8B:
    SRL A
    JR NC,CB93H
    LD (IY-53H),FFH

LCB93:
    INC IY
    JR NZ,CB8BH
    POP IY
    IN A,(C)
    RRCA
    JR NC,CBA2H
    LD A,D0H
    OUT (C),A

LCBA2:
    XOR A
    CP B
    JR NZ,CB79H
    XOR A
    SET 2,C
    OUT (C),A
    LD (DFFBH),A
    LD (IY-54H),A
    JR CB79H

LCBB3:
    LD (IY-4EH),40H
    LD A,B
    NEG
    AND B
    AND 0FH
    JR Z,CBEBH
    CALL CDE0H
    CALL CE47H
    LD D,32H

LCBC7:
    LD B,58H
    CALL CDF7H
    DEC D
    JR NZ,CBC7H
    CALL CE47H

LCBD2:
    LD B,78H
    CALL CDF7H
    JR NZ,CBE0H
    INC D
    JR NZ,CBD2H
    LD A,01H
    JR CBF6H

LCBE0:
    LD L,00H
    LD A,D
    SUB 31H
    LD A,L
    JR C,CBF6H
    DEC L
    JR CBF6H

LCBEB:
    INC A
    CALL CDE0H
    SUB 02H
    LD A,02H
    CALL Z,CDE0H

LCBF6:
    CALL CE47H
    JP CB79H

LCBFC:
    LD A,(DFFBH)
    AND 40H
    LD (IY-4EH),A
    LD A,B
    NEG
    AND B
    AND 0FH
    JP Z,CB75H
    PUSH DE
    LD E,A
    PUSH IY
    POP IX

LCC13:
    INC IX
    RRCA
    JR NC,CC13H
    LD D,(IX-54H)
    BIT 0,(IY-4DH)
    JR Z,CC23H
    SET 7,E

LCC23:
    BIT 5,H
    JR Z,CC29H
    SET 5,E

LCC29:
    SET 6,E
    LD A,E
    LD (DFFBH),A
    SET 2,C
    OUT (C),A
    RES 2,C
    XOR (IY-54H)
    AND 0FH
    JR Z,CC42H
    INC (IY-4EH)
    LD (IY-54H),E

LCC42:
    INC D
    JR NZ,CC4CH
    INC D
    CALL CDF5H
    CALL CE47H

LCC4C:
    DEC D
    LD A,(IY-4CH)
    LD E,A
    INC A
    JR NZ,CC55H
    LD E,D

LCC55:
    PUSH DE
    BIT 3,H
    JR Z,CC5EH
    SLA D
    SLA E

LCC5E:
    INC C
    OUT (C),D
    INC C
    LD A,(IY-4BH)
    OUT (C),A
    INC C
    OUT (C),E
    DEC C
    DEC C
    DEC C
    LD A,D
    CP E
    JR Z,CC75H
    SET 2,(IY-4EH)

LCC75:
    LD B,18H
    CALL CDF7H
    POP DE
    INC C
    OUT (C),E
    DEC C
    LD (IX-54H),E
    LD A,(IY-4FH)
    LD D,A
    SUB 04H
    CP 02H
    JR NC,CC8EH
    LD L,01H

LCC8E:
    LD B,L
    EX (SP),HL
    CALL CE4FH
    PUSH AF

LCC94:
    LD A,82H
    BIT 0,(IY-4DH)
    JR Z,CC9EH
    SET 3,A

LCC9E:
    DEC D
    JR Z,CCB8H
    ADD A,20H
    DEC D
    JR Z,CCEAH
    SUB 20H
    DEC D
    JP Z,CD15H
    DEC D
    LD A,F8H
    JR Z,CCEAH
    DEC D
    LD A,E8H
    JR Z,CCB8H
    LD A,C8H

LCCB8:
    CALL CDB6H
    JP NZ,CD46H
    SET 2,C
    LD DE,9B93H

LCCC3:
    JR CCC5H

LCCC5:
    IN A,(C)
    JP M,CCDAH
    DEC DE
    IN A,(C)
    JP M,CCDAH
    LD A,D
    OR E
    JP Z,CD46H
    IN A,(C)
    JP P,CCC3H

LCCDA:
    DEC C
    IN A,(C)
    LD (HL),A
    INC HL
    INC C

LCCE0:
    IN A,(C)
    JR Z,CCE0H
    JP M,CCDAH
    JP CD5CH

LCCEA:
    CALL CDB6H
    JR NZ,CD46H
    SET 2,C
    LD DE,DF82H

LCCF4:
    DEC DE
    LD A,D
    OR E
    JR Z,CD46H
    IN A,(C)
    JP M,CD02H
    JR NZ,CD46H
    JR CCF4H

LCD02:
    DEC C
    LD E,C

LCD04:
    INC C
    LD D,(HL)
    INC HL

LCD07:
    IN A,(C)
    JR Z,CD07H
    LD C,E
    OUT (C),D
    JP M,CD04H
    INC C
    DEC HL
    JR CD5CH

LCD15:
    CALL CDB6H
    JR NZ,CD46H
    SET 2,C
    LD DE,9B93H

LCD1F:
    JR CD21H

LCD21:
    IN A,(C)
    JP M,CD35H
    DEC DE
    IN A,(C)
    JP M,CD35H
    LD A,D
    OR E
    JR Z,CD46H
    IN A,(C)
    JP P,CD1FH

LCD35:
    DEC C
    IN A,(C)
    INC C
    CP (HL)
    JR NZ,CD52H
    INC HL

LCD3D:
    IN A,(C)
    JR Z,CD3DH
    JP M,CD35H
    JR CD5CH

LCD46:
    LD D,01H
    SET 2,C
    IN A,(C)
    AND 01H
    JR NZ,CD5EH
    JR CD54H

LCD52:
    LD D,02H

LCD54:
    LD A,D0H
    RES 2,C
    OUT (C),A
    JR CD80H

LCD5C:
    LD D,00H

LCD5E:
    RES 2,C
    IN A,(C)
    AND DCH
    JR NZ,CD7FH
    OR D
    JR NZ,CD7FH
    DEC B
    JR Z,CD7FH
    INC C
    INC C
    IN A,(C)
    INC A
    OUT (C),A
    DEC C
    DEC C
    LD (IY-4EH),40H
    LD D,(IY-4FH)
    JP CC94H

LCD7F:
    LD D,A

LCD80:
    LD A,(IY-4FH)
    SUB 02H
    JR Z,CD8DH
    SUB 02H
    JR NZ,CD90H
    LD A,BFH

LCD8D:
    DEC A
    JR NZ,CD8DH

LCD90:
    LD (IY-4AH),6EH
    POP AF
    JR C,CD98H
    EI

LCD98:
    POP HL
    LD A,L
    SUB B
    LD L,A
    LD A,(IY-49H)
    INC (IY-49H)
    AND 03H
    LD C,A
    LD B,00H
    PUSH IY
    POP IX
    ADD IX,BC
    LD A,R
    LD (IX-59H),A
    LD A,D
    JP CB79H

LCDB6:
    LD DE,2925H
    BIT 6,(IY-4EH)
    JR NZ,CDC2H
    LD DE,B24EH

LCDC2:
    PUSH AF

LCDC3:
    EX (SP),HL
    EX (SP),HL
    DEC DE
    LD A,D
    OR E
    JR Z,CDDCH
    IN A,(C)
    JP M,CDC3H
    POP AF
    DB FDH                                                                          ; |.|

LCDD1:
    RES 6,D
    LD D,(HL)
    JR Z,CDD8H
    SET 2,A

LCDD8:
    OUT (C),A
    XOR A
    RET

LCDDC:
    POP AF
    XOR A
    DEC A
    RET

LCDE0:
    LD B,A
    XOR A
    CALL CB49H
    LD A,(DFFBH)
    XOR B
    AND F0H
    XOR B
    LD (DFFBH),A
    SET 2,C
    OUT (C),A
    RES 2,C

LCDF5:
    LD B,08H

LCDF7:
    PUSH HL
    LD A,H
    AND 03H
    OR B
    OUT (C),A
    LD E,(IY-4EH)
    LD A,(IY-4FH)
    RES 1,A
    DEC A
    JR Z,CE35H
    BIT 0,E
    JR Z,CE16H
    RES 0,E
    LD B,22H

LCE11:
    DEC A
    JR NZ,CE11H
    DJNZ CE11H

LCE16:
    BIT 6,E
    JR NZ,CE34H
    SET 6,E
    IN B,(C)
    LD HL,B631H

LCE21:
    IN A,(C)
    XOR B
    RLCA
    JR NC,CE2FH
    IN B,(C)
    LD A,0CH
    CP H
    JR NC,CE2FH
    LD H,A

LCE2F:
    DEC HL
    LD A,H
    OR L
    JR NZ,CE21H

LCE34:
    DB 21H                                                                          ; |!|

LCE35:
    RES 2,E
    LD (IY-4EH),E

LCE3A:
    LD (IY-4AH),6EH
    IN A,(C)
    RRCA
    JR C,CE3AH
    POP HL
    AND 02H
    RET

LCE47:
    LD B,18H

LCE49:
    DEC E
    JR NZ,CE49H
    DJNZ CE49H
    RET

LCE4F:
    XOR A
    PUSH AF
    POP AF
    LD A,R
    DI
    RET PE
    DEC SP
    DEC SP
    POP AF
    OR A
    RET NZ
    SCF
    RET
    LD HL,(0025H)
    LD A,H
    OR L
    JR NZ,CE84H
    LD HL,0E98H
    LD B,11H
    CALL CEA1H
    JR Z,CE78H
    LD HL,0100H
    LD B,03H
    CALL CEA1H
    JR NZ,CE84H

LCE78:
    LD HL,EE84H
    LD (DBCCH),HL
    LD HL,(DB26H)
    LD (0025H),HL

LCE84:
    LD HL,DB1EH
    LD A,(0B16H)
    CP (HL)
    JR Z,CE91H
    LD (HL),A
    LD (DFF4H),A

LCE91:
    LD A,(0003H)
    LD (DFFFH),A
    CALL CEC0H
    LD A,(DFFFH)
    LD (0003H),A
    RET

LCEA1:
    LD DE,CEACH

LCEA4:
    LD A,(DE)
    CP (HL)
    RET NZ
    INC DE
    INC HL
    DJNZ CEA4H
    RET

; hbf: Dollar-terminated banner: TV COMPUTER BASIC 1.!
    DB 54H, 56H, 20H, 43H, 4FH, 4DH, 50H, 55H, 54H, 45H, 52H, 20H, 42H, 41H, 53H, 49H ; |TV COMPUTER BASI|
    DB 43H, 20H, 31H, 2EH, 21H, 24H                                                 ; |C 1.!$|
    IN A,(34H)
    LD B,64H
    CALL CF2FH
    INC HL
    LD B,3CH
    CALL NC,CF2FH
    LD HL,DB22H
    CALL NC,CF2FH
    INC HL
    LD B,18H
    CALL NC,CF2FH
    JR C,CEFEH
    LD A,(DB1FH)
    LD L,A
    LD A,(DB21H)
    LD D,A
    CALL CFD7H
    LD HL,DB20H
    CALL CF2FH
    JR C,CEFEH
    INC (HL)
    INC HL
    LD B,0CH
    CALL CF2FH
    JR C,CEFEH
    INC (HL)
    LD HL,DB1FH
    INC (HL)

LCEFE:
    LD HL,D0BDH
    LD B,10H

LCF03:
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    LD A,D
    OR E
    JR Z,CF18H
    LD A,09H
    ADD A,E
    LD E,A
    JR NC,CF12H
    INC D

LCF12:
    LD A,(DE)
    CP 03H
    ADC A,FFH
    LD (DE),A

LCF18:
    DJNZ CF03H
    LD A,(D036H)
    OR A
    RET Z
    DEC A
    LD (D036H),A
    RET NZ
    LD B,A
    CALL DB2EH
    LD C,C
    EX DE,HL
    XOR A
    LD (D036H),A
    RET

LCF2F:
    LD A,(HL)
    LD (HL),00H
    INC A
    CP B
    RET NC
    LD (HL),A
    RET
    DI
    LD A,(DB1FH)
    LD DE,(DB20H)
    EI
    LD C,A
    LD B,00H
    LD HL,07BCH
    ADD HL,BC
    LD A,D
    CP 03H
    LD A,C
    SBC A,FCH
    AND FCH
    RRCA
    RRCA
    ADD A,C
    PUSH HL
    LD HL,CF63H
    LD C,D
    ADD HL,BC
    ADD A,(HL)
    POP HL
    ADD A,E

LCF5B:
    SUB 07H
    JR NC,CF5BH
    ADD A,07H
    LD C,A
    XOR A
    RET
    LD BC,0404H
    RLCA
    ADD HL,BC
    INC C
    LD C,11H
    INC D
    LD D,19H
    DEC DE
    DB 01H, BCH                                                                     ; |..|

LCF72:
    RLCA
    XOR A
    SBC HL,BC
    JR C,CF9EH
    OR H
    LD A,L
    LD B,64H
    CALL Z,CFD3H
    JR NZ,CF9EH
    LD A,D
    LD B,0DH
    CALL Z,CFD0H
    JR NZ,CF9EH
    CALL CFD7H
    LD A,E
    CALL CFD0H
    JR NZ,CF9EH
    DI
    LD A,L
    LD (DB1FH),A
    LD (DB20H),DE
    EI
    XOR A
    RET

LCF9E:
    LD A,97H
    RET
    DI
    LD HL,(DB22H)
    LD DE,(DB24H)
    EI
    XOR A
    RET
    LD A,E
    LD B,64H
    CALL CFD3H
    LD A,D
    LD B,3CH
    CALL Z,CFD3H
    LD A,L
    CALL Z,CFD3H
    LD A,H
    LD B,18H
    CALL Z,CFD3H
    LD A,96H
    RET NZ
    DI
    LD (DB22H),HL
    LD (DB24H),DE
    EI
    XOR A
    RET

LCFD0:
    OR A
    JR Z,CFD4H

LCFD3:
    CP B

LCFD4:
    CCF
    SBC A,A
    RET

LCFD7:
    PUSH HL
    PUSH DE
    LD A,L
    LD HL,CFEFH
    AND 03H
    LD E,D
    LD D,00H
    ADD HL,DE
    SUB 01H
    LD A,(HL)
    BIT 7,A
    JR Z,CFECH
    ADC A,9DH

LCFEC:
    LD B,A
    POP DE
    POP HL
    RET
    JR NZ,CF72H
    JR NZ,D013H
    JR NZ,D015H
    JR NZ,D018H
    RRA
    JR NZ,D01AH
    JR NZ,CFFDH

LCFFD:
    NOP
    NOP
    NOP
