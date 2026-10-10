; -----------------------------------------------------------------------------
; VT-DOS 1.2 HBF disk ROM page 3
; Source: roms/VT-DOS12-DISK.ROM
; HBF ROM page: 3
; File offset: 3000H
; CPU window: C000H-CFFFH while this HBF ROM page is selected
; HBF ROM page 3 is mapped at C000H-CFFFH only after the card selects page 3. It holds the CLI command table.
; ORG: C000H
; Size: 4096 bytes
; Comments: roms/VT-DOS12-DISK.page3.comments.json
; Data ranges: C000H-C00AH, C1DCH-C29FH, C2A0H-C2A8H, C545H-C54CH, CC83H-CC8BH, CF81H-CFFFH
; Auto labels: branch and call targets are emitted as Lxxxx.
; -----------------------------------------------------------------------------

ORG C000H


; hbf: MOPS signature and the length-prefixed name DISK. Page 0, not this page, supplies the C00BH initializer and the C00DH function table.
    DB 4DH, 4FH, 50H, 53H, 04H, 44H, 49H, 53H, 4BH, 6FH, DBH                        ; |MOPS.DISKo.|
    NOP
    NOP
    LD BC,DBC1H
    LD A,C
    LD (0D6DH),A
    CALL DB2EH
    LD B,C
    RST 00H
    LD (0D7EH),SP
    XOR A
    LD (0D7CH),A
    LD (0D7DH),A
    LD (0D80H),A
    LD (0D81H),A
    LD A,(DE)
    OR A
    JR Z,C040H
    LD HL,0D7DH
    DEC (HL)
    LD A,(DE)
    LD C,A
    AND 80H
    JP NZ,C0C4H
    XOR A
    LD B,A
    INC DE
    EX DE,HL
    JP C0CEH

LC040:
    LD IY,(DFF5H)
    XOR A
    LD (0D6DH),A
    CALL CD2DH
    LD A,0BH
    CALL CD42H
    CALL CD2DH

LC053:
    LD HL,C2A8H
    LD DE,0D7BH
    LD BC,000AH
    LDDR
    INC DE
    LD C,DFH
    CALL DB2EH
    LD H,A
    RST 00H
    RST 30H
    DEC BC
    CALL DB2EH
    LD B,C
    RST 00H
    LD C,(IY-62H)
    LD B,(IY-5FH)
    LD HL,DC0FH
    LD (DFF8H),HL
    INC C
    DEC C
    PUSH BC
    LD A,(0D6DH)
    AND 01H
    LD (0D6DH),A
    LD A,03H
    JR NZ,C08DH
    CALL DB2EH
    LD B,(HL)
    DB CDH                                                                          ; |.|

LC08D:
    POP BC
    LD C,A
    LD A,B
    ADD A,40H
    CALL CCD8H
    OR C
    JR NZ,C0A6H
    LD A,3AH
    CALL CCD8H
    LD A,5CH
    CALL CCD8H
    EX DE,HL
    CALL CD48H

LC0A6:
    LD A,DFH
    CALL CCD8H
    LD HL,0000H
    LD (DFF8H),HL
    CALL CD75H
    JP NZ,C10FH
    JP C,C14DH
    LD A,(0BF5H)
    OR A
    JR Z,C053H
    CP 7FH
    JR NZ,C0C8H

LC0C4:
    LD A,90H
    JR C10FH

LC0C8:
    LD C,A
    XOR A
    LD B,A
    LD HL,0BF6H

LC0CE:
    LD DE,0BF5H
    PUSH DE
    LDIR
    LD (DE),A
    POP HL
    CALL CC8CH
    INC HL

LC0DA:
    JP Z,C053H
    CP 2AH
    JR Z,C0DAH
    LD B,A
    LD A,(HL)
    CP 3AH
    JR NZ,C108H
    INC HL
    CALL CC8CH
    LD A,91H
    JP NZ,C10FH
    LD A,B
    CALL C1D3H
    SUB 40H
    LD B,A
    PUSH BC
    LD A,03H
    CALL DB2EH
    LD B,(HL)
    CALL 20C1H
    DEC C
    LD A,B
    LD (IY-5FH),A
    JR C12FH

LC108:
    DEC HL
    CALL C165H
    OR A
    JR Z,C12FH

LC10F:
    PUSH AF
    LD HL,0D6DH
    RES 1,(HL)
    LD HL,0D7DH
    BIT 0,(HL)
    JR NZ,C147H
    SUB F5H
    JR NZ,C12BH
    CALL CD2DH
    LD (0B16H),A
    CALL DB2EH
    LD L,DCH

LC12B:
    POP AF
    CALL CEBDH

LC12F:
    CALL C13CH
    LD HL,0D7DH
    BIT 0,(HL)
    JR NZ,C150H
    JP C053H

LC13C:
    LD BC,FFFFH
    LD A,0EH
    CALL DB2EH
    LD B,(HL)
    DB CDH, C9H                                                                     ; |..|

LC147:
    CALL C13CH
    POP AF
    JR C156H

LC14D:
    CALL CD2DH

LC150:
    XOR A
    JP C156H
    LD A,8EH

LC156:
    CALL DB2EH
    LD H,A
    RST 00H
    LD HL,(0D80H)
    PUSH HL
    POP BC
    LD DE,(0BF0H)
    RET

LC165:
    DEC HL
    XOR A
    LD (HL),A
    PUSH HL
    LD B,FFH

LC16B:
    INC B
    INC HL
    CALL CC5CH
    JR NZ,C16BH
    POP DE
    PUSH HL
    EX DE,HL
    LD A,B
    LD (HL),A
    EX DE,HL
    LD IX,0C79H
    LD A,01H
    LD (0D6FH),A
    XOR A
    LD (IX+06H),A
    LD (IX+07H),A
    LD (IX+00H),FFH
    LD HL,0D6BH
    RES 4,(HL)
    LD HL,C1DAH

LC194:
    JR NZ,C1B5H
    INC HL
    INC HL
    LD A,(HL)
    OR A
    JR Z,C1B5H
    CALL C1B9H
    JR C,C194H
    LD A,(HL)
    INC HL
    LD H,(HL)
    LD L,A
    EX DE,HL
    POP HL
    PUSH DE
    CALL CBBEH
    CALL CC8CH
    XOR A
    LD (0D71H),A
    LD C,A
    DEC A
    RET

LC1B5:
    LD A,91H
    POP DE
    RET

LC1B9:
    PUSH BC
    PUSH DE
    LD A,B
    LD B,(HL)
    CP B
    JR C,C1D0H
    INC B

LC1C1:
    JR C,C1C6H
    SUB (HL)
    ADD A,FFH

LC1C6:
    INC HL
    INC DE
    LD A,(DE)
    CALL C1D3H
    DJNZ C1C1H
    SBC A,A
    INC A

LC1D0:
    POP DE
    POP BC
    RET

LC1D3:
    BIT 7,A
    RET NZ
    BIT 6,A
    RET Z
    RES 5,A
    RET

; hbf: CLI command table. Each record is a length, the name, and a little-endian handler. A 00H length ends the table. Names: CD, MD, RD, DEL, ERA, VOL, DIR, DOS, REN, CLS, VAR, LDIR, COPY, DATE, HELP, TIME, MOVE, TYPE, LTYPE, MVDIR, RNDIR, CHDIR, ERASE, MKDIR, RMDIR, ERASE, FORMAT, RENAME.
    DB 02H, 43H, 44H, A9H, C2H, 02H, 4DH, 44H, 33H, C3H, 02H, 52H, 44H, D2H, C2H, 03H ; |.CD...MD3..RD...|
    DB 44H, 45H, 4CH, D3H, C2H, 03H, 45H, 52H, 41H, D3H, C2H, 03H, 56H, 4FH, 4CH, 57H ; |DEL...ERA...VOLW|
    DB C5H, 03H, 44H, 49H, 52H, 16H, C4H, 03H, 44H, 4FH, 53H, 24H, CAH, 03H, 52H, 45H ; |..DIR...DOS$..RE|
    DB 4EH, 4BH, C3H, 03H, 43H, 4CH, 53H, 0BH, CAH, 03H, 56H, 41H, 52H, 40H, C7H, 04H ; |NK..CLS...VAR@..|
    DB 4CH, 44H, 49H, 52H, 13H, C4H, 04H, 43H, 4FH, 50H, 59H, A5H, C7H, 04H, 44H, 41H ; |LDIR...COPY...DA|
    DB 54H, 45H, 9BH, C5H, 04H, 48H, 45H, 4CH, 50H, 47H, CAH, 04H, 54H, 49H, 4DH, 45H ; |TE...HELPG..TIME|
    DB 6EH, C6H, 04H, 4DH, 4FH, 56H, 45H, 48H, C3H, 04H, 54H, 59H, 50H, 45H, 9FH, C7H ; |n..MOVEH..TYPE..|
    DB 05H, 4CH, 54H, 59H, 50H, 45H, 9CH, C7H, 05H, 4DH, 56H, 44H, 49H, 52H, 47H, C3H ; |.LTYPE...MVDIRG.|
    DB 05H, 52H, 4EH, 44H, 49H, 52H, 4AH, C3H, 05H, 43H, 48H, 44H, 49H, 52H, A9H, C2H ; |.RNDIRJ..CHDIR..|
    DB 05H, 45H, 52H, 41H, 53H, 45H, D3H, C2H, 05H, 4DH, 4BH, 44H, 49H, 52H, 33H, C3H ; |.ERASE...MKDIR3.|
    DB 05H, 52H, 4DH, 44H, 49H, 52H, D2H, C2H, 05H, 45H, 52H, 41H, 53H, 45H, D3H, C2H ; |.RMDIR...ERASE..|
    DB 06H, 46H, 4FH, 52H, 4DH, 41H, 54H, ACH, C3H, 06H, 52H, 45H, 4EH, 41H, 4DH, 45H ; |.FORMAT...RENAME|
    DB 4BH, C3H, 00H, 00H                                                           ; |K...|

; hbf: Eight-byte character bitmap placed immediately before the CD handler at C2A9H.
    DB 30H, 18H, 0CH, 06H, 0CH, 18H, 30H, 00H, 00H                                  ; |0.....0..|
    LD C,01H
    CALL CB4EH
    CALL CA95H
    RET NZ
    BIT 0,B
    LD A,02H
    JP NZ,CB38H
    LD B,(IX+07H)
    LD A,03H
    CALL CB38H
    RET NZ
    LD DE,CD2DH
    PUSH DE
    EX DE,HL

LC2C7:
    CALL CD36H
    LD A,5CH
    CALL CCD8H
    JP CD48H
    SCF
    CALL CBDCH
    CALL Z,CA95H
    RET NZ
    LD A,(0D6FH)
    AND FEH
    LD (0D6FH),A
    CALL CAB7H
    RET C
    LD HL,0C81H
    LD B,0BH
    LD A,3FH

LC2ED:
    CP (HL)
    JR NZ,C2FBH
    INC HL
    DJNZ C2EDH

LC2F3:
    LD A,18H
    CALL C30CH
    JR NC,C2F3H
    RET NZ

LC2FB:
    LD HL,0C79H
    LD DE,0CA9H
    LD A,08H
    CALL CB38H
    CALL NC,CB8FH
    JR NC,C2FBH
    RET

LC30C:
    CALL CD42H
    CALL DB2EH
    SUB D
    IN A,(C0H)
    LD A,H
    CP 20H
    CALL NC,CCD8H
    CALL CD2DH
    LD A,0DH
    CALL DB2EH
    ADD A,D
    IN A,(9FH)
    LD A,00H
    RET NZ
    LD A,0CH
    CALL DB2EH
    ADD A,D
    IN A,(9FH)
    INC A
    RET
    CALL CB4EH
    CALL CA8AH
    PUSH HL
    CALL Z,CA95H
    POP HL
    RET NZ
    CALL CBBEH
    LD A,07H
    JP CB38H
    SCF
    INC A
    LD B,37H
    CALL CBDCH
    RET NZ
    PUSH BC
    PUSH DE
    CALL CAB7H
    POP HL
    POP BC
    RET C
    LD (HL),C
    PUSH HL
    CALL CBCAH
    LD A,(IX-59H)
    LD (IX+07H),A
    PUSH DE
    EXX
    POP HL
    EXX
    POP HL
    CALL CC8CH
    LD C,00H
    EX AF,AF'
    JR NZ,C371H
    LD C,01H

LC371:
    EX AF,AF'
    CALL CB4EH
    CALL Z,CA95H
    RET NZ
    LD IX,0C79H
    EX AF,AF'
    PUSH AF
    EX AF,AF'
    POP AF
    JR NZ,C389H
    PUSH HL
    EXX
    POP HL
    EXX
    JR C391H

LC389:
    BIT 1,B
    LD A,8EH
    CALL Z,CA8AH
    RET NZ

LC391:
    CALL CBBEH

LC394:
    EX AF,AF'
    LD L,0AH
    JR Z,C39AH
    DEC L

LC39A:
    EX AF,AF'
    LD A,L
    EX DE,HL
    LD E,C
    LD D,B
    EXX
    PUSH HL
    EXX
    POP BC
    CALL CB38H
    CALL NC,CB8FH
    JR NC,C394H
    RET
    SET 3,(IX+06H)
    CALL CB4EH
    PUSH BC
    LD HL,C40CH
    CALL CBFDH
    CALL Z,CA95H
    JR NZ,C3CCH
    LD HL,(0BF2H)
    LD DE,1964H
    AND A
    SBC HL,DE
    LD A,8EH
    JR NC,C3D6H

LC3CC:
    POP BC
    RET NZ
    LD A,B
    AND 01H
    RET Z
    LD A,B
    JP C563H

LC3D6:
    EXX
    LD A,C
    OR A
    RRA
    XOR F9H
    LD B,A
    EXX
    LD A,(0D6DH)
    BIT 0,A
    JR NZ,C3F9H

LC3E5:
    LD A,21H
    CALL CD42H
    LD A,(IX+07H)
    CALL CD36H
    LD A,22H
    CALL C30CH
    JR NC,C3E5H
    JR NZ,C3CCH

LC3F9:
    EXX
    LD A,B
    EXX
    LD B,A
    LD A,(IX+07H)
    LD C,A
    LD DE,(0BF0H)
    LD A,17H
    CALL CB38H
    JR C3CCH
    LD SP,4802H
    EX AF,AF'
    JR C,C416H
    NOP
    CALL CCCBH

LC416:
    LD (0D71H),A
    CALL CB44H
    PUSH HL
    PUSH BC
    LD HL,C54EH
    CALL CBFDH
    CALL Z,CA95H
    JR NZ,C437H
    LD HL,0D7CH
    SET 3,(IX+06H)
    CALL C563H
    RES 3,(IX+06H)

LC437:
    POP BC
    POP HL
    RET NZ
    EXX
    BIT 7,C
    JR NZ,C447H
    LD A,C
    AND F3H
    OR 10H
    LD (0D6FH),A

LC447:
    LD B,01H
    LD A,(0D6DH)
    BIT 0,A
    JR NZ,C456H
    LD A,(0B73H)
    OR A
    JR NZ,C45CH

LC456:
    LD A,0CH
    LD B,04H
    JR C46BH

LC45C:
    DEC A
    JR NZ,C46AH
    LD B,02H
    LD A,C
    AND 04H
    JR NZ,C46AH
    OR 08H
    JR C46BH

LC46A:
    LD A,C

LC46B:
    LD (0D6EH),A
    LD C,00H
    EXX
    LD A,19H
    CALL CD42H
    CALL CAB7H
    JR Z,C480H
    CP A1H
    RET NZ
    XOR A
    SCF

LC480:
    EX AF,AF'
    CALL CD2DH
    CALL CD2DH
    LD HL,0C75H
    LD (HL),A
    INC HL
    LD (HL),A
    INC HL
    LD (HL),A
    INC HL
    LD (HL),A
    EX AF,AF'

LC492:
    JR C,C506H
    CALL CBBEH
    CALL CDB0H
    LD HL,0CCDH
    PUSH HL
    LD DE,0C75H
    CALL CF4EH
    POP DE
    LD A,(0D6FH)
    AND 80H
    JR NZ,C4F1H
    CALL CDE3H
    LD C,(IX+43H)
    BIT 0,C
    LD A,72H
    CALL CDE1H
    BIT 1,C
    LD A,68H
    CALL CDE1H
    LD A,(0D6EH)
    BIT 3,A
    JR Z,C4D9H
    BIT 4,(IX+43H)
    JR NZ,C4D3H
    LD A,20H
    CALL CEF5H
    XOR A

LC4D3:
    LD DE,C545H
    CALL NZ,CD48H

LC4D9:
    LD A,(0D6EH)
    BIT 2,A
    JR Z,C4ECH
    LD HL,0CC9H
    CALL CE66H
    LD HL,0CC8H
    CALL CE25H

LC4EC:
    CALL CD2DH
    JR C501H

LC4F1:
    EXX
    INC C
    LD A,C
    SUB B
    JR NZ,C4FBH
    LD C,A
    CALL CD2DH

LC4FB:
    EXX
    LD A,1EH
    CALL NZ,CD42H

LC501:
    CALL CB8FH
    JR C492H

LC506:
    RET NZ
    EXX
    OR C
    CALL NZ,CD2DH
    CALL CDE3H
    LD HL,0C75H
    CALL CDE7H
    LD A,1BH
    CALL CD42H
    LD HL,(0D69H)
    LD A,1CH
    CALL CDFAH
    LD A,1DH
    CALL CD42H
    LD B,(IX+07H)
    LD A,15H
    CALL CB38H
    RET NZ
    DB 3EH                                                                          ; |>|

LC531:
    ADD HL,HL
    SRL B
    JR NZ,C531H
    SRL H
    RR L
    CALL CEE8H
    CALL CCD6H
    LD A,1FH
    JP CD2AH

; hbf: NUL-terminated directory tag: space, <dir>, two spaces.
    DB 20H, 3CH, 64H, 69H, 72H, 3EH, 20H, 20H                                       ; | <dir>  |
    NOP
    LD D,E
    EX AF,AF'
    LD D,H
    INC B
    LD D,A
    ADD A,B
    LD C,B
    LD (BC),A
    NOP
    SET 3,(IX+06H)
    CALL CB4EH
    CALL CA95H
    RET NZ
    LD A,B

LC563:
    CALL CBBEH
    AND 01H
    LD A,06H
    JP NZ,CB38H
    LD A,04H
    CALL CB38H
    CP A1H
    JR Z,C578H
    OR A
    RET NZ

LC578:
    EX AF,AF'
    LD A,26H
    CALL CD42H
    CALL CD36H
    EX AF,AF'
    OR A
    LD A,27H
    JP NZ,CD2AH
    LD A,28H
    CALL CD42H
    LD HL,0CBCH
    LD (HL),00H
    LD DE,0CB1H
    CALL CD48H
    JP CD2DH
    AND (HL)
    JR NZ,C5FDH
    LD A,1BH
    CALL CB38H
    RET NZ
    PUSH HL
    PUSH DE
    PUSH BC
    LD A,0FH
    CALL CD42H
    POP BC
    LD B,00H
    LD HL,C667H
    ADD HL,BC
    LD A,(HL)
    CALL CD42H
    POP DE
    POP HL
    LD BC,F844H
    ADD HL,BC
    LD A,D
    ADD A,A
    ADD A,A
    ADD A,A
    ADD A,A
    ADD A,A
    RL L
    OR E
    LD D,L
    LD HL,0C76H
    LD (HL),D
    DEC HL
    LD (HL),A
    CALL CE66H
    CALL CD2DH

LC5D3:
    LD A,(0D6DH)
    BIT 0,A
    LD A,00H
    RET NZ
    LD A,10H
    CALL CD42H
    CALL CD51H
    RET NZ
    JR NC,C5EAH
    CALL CD2DH
    RET

LC5EA:
    LD A,(0BF5H)
    OR A
    RET Z
    LD HL,0BF6H
    PUSH HL
    LD C,A
    XOR A
    LD B,A
    ADD HL,BC
    LD (HL),A
    POP HL
    CALL CC8CH
    RET Z

LC5FD:
    EXX
    LD A,1BH
    CALL CB38H
    RET NZ
    EXX
    LD A,(IY-61H)
    ADD A,A
    JR NZ,C60DH
    LD A,36H

LC60D:
    ADD A,A
    LD B,03H
    PUSH AF
    JR C61AH

LC613:
    PUSH BC
    CALL CC3DH
    POP BC
    JR C,C658H

LC61A:
    CALL CC92H
    PUSH AF
    LD A,C
    EXX
    POP BC
    LD C,A
    POP AF
    RLCA
    RLCA
    PUSH AF
    INC B
    DEC B
    JR NZ,C646H
    AND 03H
    JR Z,C646H
    DEC A
    JR Z,C645H
    DEC A
    JR Z,C643H
    LD A,C
    SUB 50H
    JR NC,C63BH
    ADD A,64H

LC63B:
    LD C,A
    LD B,00H
    LD HL,07BCH
    ADD HL,BC
    LD C,D

LC643:
    LD D,C
    DB 0EH                                                                          ; |.|

LC645:
    LD E,C

LC646:
    EXX
    DJNZ C613H
    CALL CC8CH
    JR NZ,C658H
    POP AF
    EXX
    LD A,1CH
    CALL CB38H
    RET Z
    JR C65BH

LC658:
    POP AF
    LD A,97H

LC65B:
    LD HL,0D6DH
    BIT 0,(HL)
    RET NZ
    CALL CEBDH
    JP C5D3H
    RLA
    LD DE,1312H
    INC D
    DEC D
    LD D,A6H
    JR NZ,C6B7H
    LD A,1DH
    CALL CB38H
    RET NZ
    LD A,24H
    CALL CD42H
    LD A,L
    ADD A,A
    ADD A,A
    LD B,03H

LC681:
    ADD A,A
    RL H
    DJNZ C681H
    LD C,H
    LD HL,0C75H
    LD (HL),A
    INC HL
    LD (HL),C
    CALL CE25H
    CALL CD2DH

LC693:
    LD A,(0D6DH)
    BIT 0,A
    LD A,00H
    RET NZ
    LD A,25H
    CALL CD42H
    CALL CD51H
    RET C
    RET NZ
    LD HL,0BF5H
    LD A,(HL)
    INC HL
    OR A
    RET Z
    PUSH HL
    LD C,A
    XOR A
    LD B,A
    ADD HL,BC
    LD (HL),A
    POP HL
    CALL CC8CH
    RET Z

LC6B7:
    EXX
    LD A,1DH
    CALL CB38H
    RET NZ
    EXX
    CALL CC92H
    JR NZ,C6C8H
    LD A,C
    EXX
    LD H,A
    EXX

LC6C8:
    CALL CC3DH
    SBC A,A
    JR NZ,C6D1H
    CALL CC92H

LC6D1:
    JR NZ,C6D7H
    LD A,C
    EXX
    LD L,A
    EXX

LC6D7:
    CALL CC3DH
    SBC A,A
    JR NZ,C6E0H
    CALL CC92H

LC6E0:
    JR NZ,C6E6H
    LD A,C
    EXX
    LD D,A
    EXX

LC6E6:
    CALL CC3DH
    SBC A,A
    JR NZ,C6EFH
    CALL CC92H

LC6EF:
    JR NZ,C6F5H
    LD A,C
    EXX
    LD E,A
    EXX

LC6F5:
    CALL CC8CH
    CALL C1D3H
    CP 41H
    JR Z,C70EH
    CP 50H
    SCF
    JR Z,C70EH
    OR A
    EXX
    JR Z,C72EH
    DB 3EH                                                                          ; |>|

LC709:
    POP AF

LC70A:
    LD A,96H
    JR C734H

LC70E:
    PUSH AF
    INC HL
    CALL CC8CH
    JR NZ,C709H
    POP AF
    EXX
    LD B,0CH
    LD A,H
    JR C,C726H
    CP B
    JR Z,C722H
    JR NC,C70AH
    DB D2H                                                                          ; |.|

LC722:
    LD H,00H
    JR C72EH

LC726:
    OR A
    JR Z,C70AH
    CP B
    JR NC,C72DH
    ADD A,B

LC72D:
    LD H,A

LC72E:
    LD A,1EH
    CALL CB38H
    RET Z

LC734:
    LD HL,0D6DH
    BIT 0,(HL)
    RET NZ
    CALL CEBDH
    JP C693H
    AND (HL)
    LD A,8DH
    RET Z
    CALL CC92H
    RET NZ
    CALL CCBDH
    RET NZ
    OR (HL)
    JR NZ,C761H
    LD E,C
    LD B,00H
    LD A,1AH
    CALL CB38H
    RET NZ
    LD L,D
    LD H,A
    CALL CEE8H
    CALL CD2DH
    RET

LC761:
    PUSH BC
    EX DE,HL
    DEC DE
    LD HL,C795H
    LD BC,0200H
    CALL C1B9H
    JR NC,C776H
    INC B
    DEC C
    CALL C1B9H
    JR C,C779H

LC776:
    INC DE
    DJNZ C776H

LC779:
    INC DE
    EX DE,HL
    SBC A,A
    JR Z,C783H
    CALL CC92H
    JR NZ,C786H

LC783:
    CALL CCBDH

LC786:
    POP DE
    RET NZ
    OR (HL)
    LD A,8DH
    RET NZ
    LD B,FFH
    LD D,C
    LD A,1AH
    CALL CB38H
    RET
    LD (BC),A
    LD C,A
    LD C,(HL)
    INC BC
    LD C,A
    LD B,(HL)
    LD B,(HL)
    CALL CCCBH
    XOR A
    LD (0D6CH),A
    JR C7ABH
    LD A,20H
    LD (0D6CH),A
    XOR A

LC7AB:
    CALL C8C7H
    PUSH HL
    PUSH BC
    LD HL,C8C4H
    CALL CBEBH
    JR NZ,C800H
    CALL CA8AH
    JR NZ,C800H
    EXX
    LD B,C
    EXX
    EX DE,HL
    CALL CA03H
    JR NZ,C7D7H
    PUSH HL

LC7C7:
    LD A,(HL)
    CALL CC5DH
    INC HL
    JR C,C7C7H
    JR Z,C7D6H
    LD A,8DH
    POP HL
    POP BC
    POP BC
    RET

LC7D6:
    POP HL

LC7D7:
    CALL CBCAH
    CALL CC8CH
    XOR A
    CALL CA03H
    JR NZ,C7E8H
    EXX
    LD C,A
    EXX
    JR C7F8H

LC7E8:
    LD C,A
    CALL C8C7H
    PUSH HL
    EXX
    POP DE
    EXX
    LD HL,C8C6H
    CALL CBEBH
    JR NZ,C800H

LC7F8:
    CALL CA95H
    JR Z,C800H
    POP BC
    POP HL
    RET

LC800:
    LD DE,FFA0H
    ADD IX,DE
    POP BC
    POP HL
    RET NZ
    EXX
    LD A,B
    OR C
    LD (0D6FH),A
    EXX
    CALL CAB7H
    RET NZ
    LD HL,0D6CH
    RES 6,(HL)

LC818:
    CALL CBBEH
    EX DE,HL
    LD E,C
    LD D,B
    LD A,0CH
    CALL CB38H
    RET NZ
    EXX
    LD BC,(0BF0H)
    LD HL,(0BF2H)
    LD A,H
    AND FEH
    LD H,A
    LD L,00H
    LD A,8CH
    RET Z
    CALL CA03H
    JR NZ,C840H
    LD A,18H
    CP H
    JR NC,C840H
    LD H,A

LC840:
    EXX
    CALL C8E3H
    PUSH AF
    LD DE,0D09H
    LD HL,0CD9H
    PUSH HL
    POP IX
    CALL CA03H
    JR Z,C8A6H
    BIT 7,(IX+14H)
    JR NZ,C8A6H
    LD A,0DH
    CALL CB38H
    LD B,A
    POP AF
    JR NZ,C863H
    OR B

LC863:
    PUSH AF
    JR Z,C870H
    PUSH IX
    POP HL
    LD A,08H
    CALL CB38H
    JR C8A6H

LC870:
    LD A,(IX-1DH)
    SET 5,A
    LD (IX+43H),A
    PUSH DE
    LD HL,0021H
    ADD HL,DE
    EX DE,HL
    LD HL,FF9BH
    ADD HL,DE
    PUSH HL
    LDD
    LDD
    POP HL
    INC HL
    INC HL
    LDD
    LDD
    POP DE
    PUSH IX
    POP HL
    LD A,0BH
    CALL CB38H
    LD B,A
    POP AF
    JR NZ,C89CH
    OR B

LC89C:
    PUSH AF
    LD B,(IX+07H)
    LD C,A
    LD A,0EH
    CALL Z,CB38H

LC8A6:
    POP AF
    RET NZ
    CALL CA03H
    JR NZ,C8B3H
    CALL C9FBH
    CALL Z,CD2DH

LC8B3:
    LD DE,FFA0H
    ADD IX,DE
    BIT 5,(IX+13H)
    RET Z
    CALL CB8FH
    JP NC,C818H
    RET
    LD C,B
    LD (BC),A
    NOP

LC8C7:
    EX AF,AF'
    LD A,(HL)
    EXX
    LD L,A
    EXX
    CALL CB4EH
    LD A,C
    CP 3AH
    JR NZ,C8D9H
    LD (DE),A
    EXX
    LD A,L
    EXX
    LD (HL),A

LC8D9:
    XOR A
    SCF
    SET 7,(IX+14H)
    LD (IX+13H),B
    RET

LC8E3:
    CALL CBBEH
    SET 7,(IX+74H)
    BIT 5,(IX+13H)
    JR Z,C8FCH
    SCF
    CALL C9FBH
    JR NZ,C8FCH
    CALL CDB0H
    CALL CD2DH

LC8FC:
    XOR A

LC8FD:
    EX AF,AF'
    EXX
    PUSH HL
    EXX
    POP BC
    LD A,0FH
    CALL C9E2H
    RET NZ
    LD DE,0060H
    ADD IX,DE
    CALL CA03H
    JR NZ,C93AH
    LD L,C
    LD H,B
    EXX
    PUSH BC
    EXX
    POP DE

LC918:
    LD A,H
    OR L
    JP Z,C9D5H
    LD A,(DE)
    CP 1AH
    JR NZ,C929H
    XOR FEH
    EX AF,AF'
    XOR A
    JP Z,C9D5H

LC929:
    INC DE
    DEC HL
    CALL C9FBH
    CALL Z,CCD8H
    LD A,(DFF4H)
    OR A
    JR Z,C918H
    LD A,F5H
    RET

LC93A:
    BIT 7,(IX+14H)
    JP Z,C9CDH
    PUSH BC

LC942:
    LD HL,0D83H
    EX DE,HL
    PUSH DE
    LD HL,FF5EH
    ADD HL,DE
    PUSH HL
    PUSH HL
    LD BC,000DH
    LDIR
    POP DE
    LD HL,FFD0H
    ADD HL,DE
    LD B,0BH

LC959:
    LD A,(DE)
    CP 3FH
    JR NZ,C960H
    LD A,(HL)
    LD (DE),A

LC960:
    INC HL
    INC DE
    DJNZ C959H
    CALL CBBEH
    EXX
    PUSH DE
    EXX
    POP HL
    BIT 6,(IX-60H)
    JR Z,C976H
    LD A,FFH
    LD (DFF3H),A

LC976:
    LD A,06H
    PUSH HL
    LD HL,0D6CH
    BIT 6,(HL)
    SET 6,(HL)
    POP HL
    JR NZ,C9B4H
    CALL CB38H
    JR Z,C9B9H
    CP 9CH
    JR NZ,C9C3H
    BIT 5,(IX+13H)
    JR NZ,C9C3H
    LD HL,0060H
    ADD HL,DE
    PUSH HL
    EXX
    POP DE
    EXX
    EX DE,HL
    LD BC,0030H
    LDIR
    CALL CB75H
    LD A,(IX+73H)
    OR B
    LD (IX+13H),A
    LD A,(IX+74H)
    LD (IX+14H),A
    POP DE
    POP HL
    JR C942H

LC9B4:
    CALL CB38H
    JR NZ,C9C3H

LC9B9:
    LD HL,0030H
    ADD HL,DE
    EX DE,HL
    LD A,0CH
    CALL CB38H

LC9C3:
    POP DE
    POP HL
    LD BC,000DH
    LDIR
    POP BC
    OR A
    RET NZ

LC9CD:
    RES 7,(IX+14H)
    CALL C9E0H
    RET NZ

LC9D5:
    LD DE,FFA0H
    ADD IX,DE
    EX AF,AF'
    JP Z,C8FDH
    XOR A
    RET

LC9E0:
    LD A,10H

LC9E2:
    PUSH BC
    CALL CBBEH
    LD E,C
    LD D,B
    POP BC
    EXX
    PUSH BC
    EXX
    EX (SP),IX
    CALL CB38H
    POP IX
    RET Z
    CP ECH
    RET NZ
    OR A
    EX AF,AF'
    XOR A
    RET

LC9FB:
    PUSH HL
    LD HL,0D6DH
    BIT 0,(HL)
    POP HL
    RET

LCA03:
    PUSH HL
    LD HL,0D6CH
    BIT 5,(HL)
    POP HL
    RET
    AND (HL)
    LD A,8DH
    RET NZ
    LD A,(0D6DH)
    AND 01H
    LD A,00H
    RET NZ
    CALL DB2EH
    LD H,A
    RST 00H
    RST 30H
    DEC B
    CALL DB2EH
    LD B,C
    RST 00H
    RET
    AND (HL)
    LD A,8DH
    RET NZ
    LD HL,DFF2H
    LD A,(HL)
    OR A
    LD A,8BH
    RET NZ
    LD A,(0D6DH)
    BIT 0,A
    JR NZ,CA41H
    PUSH HL

LCA38:
    LD A,20H
    CALL C30CH
    JR NC,CA38H
    POP HL
    RET NZ

LCA41:
    DEC (HL)
    CALL DB2EH
    LD D,E
    JP 3EA6H
    ADC A,L
    RET NZ
    LD A,23H
    CALL CD2AH
    LD A,(0D6DH)
    RRCA
    JR C,CA5BH
    LD A,(0B73H)
    SUB 01H

LCA5B:
    LD A,40H
    JR C,CA63H
    RRCA
    JR Z,CA63H
    RRCA

LCA63:
    LD E,A
    LD D,00H
    LD HL,C1DCH

LCA69:
    LD A,(HL)
    OR A
    JP Z,CD2DH
    INC A
    ADD A,D
    CP E
    LD A,D
    CALL NC,CD2DH
    LD D,A
    LD B,(HL)
    INC HL

LCA78:
    LD A,(HL)
    CALL CCD8H
    INC D
    INC HL
    DJNZ CA78H
    LD A,20H
    CALL CCD8H
    INC D
    INC HL
    INC HL
    JR CA69H

LCA8A:
    LD A,B
    AND 05H
    SUB 01H
    SBC A,A
    AND 8DH
    RET

LCA93:
    JR NZ,CAA0H

LCA95:
    LD A,C

LCA96:
    INC DE
    CALL CC5DH
    JR C,CAA0H
    RET Z
    LD A,8DH
    RET

LCAA0:
    LD A,(DE)
    LD C,A
    JR CA96H

LCAA4:
    LD A,(HL)
    INC HL
    CP 20H
    JR Z,CAACH

LCAAA:
    LD (DE),A
    INC DE

LCAAC:
    DJNZ CAA4H
    RET

LCAAF:
    EX AF,AF'
    POP HL
    LD HL,0D6FH
    SET 4,(HL)
    RET

LCAB7:
    LD DE,CB97H
    PUSH DE
    PUSH BC
    LD BC,0000H
    LD (0D69H),BC
    CALL CBBEH
    CALL CB36H
    EX AF,AF'
    JR C,CAAFH
    EX AF,AF'
    EX DE,HL
    POP BC
    LD C,00H
    JR Z,CAD8H
    CP A1H
    SCF
    JR NZ,CB24H

LCAD8:
    LD HL,0D71H
    LD C,(HL)
    JR C,CB24H
    SCF
    BIT 5,B
    JR NZ,CB24H
    BIT 4,(IX+43H)
    JR Z,CB24H
    LD A,C
    OR A
    JR Z,CB24H
    PUSH DE
    PUSH BC
    LD C,B
    DEC DE

LCAF1:
    LD A,(DE)
    INC DE
    OR A
    JR NZ,CAF1H
    DEC DE
    BIT 6,C
    JR Z,CB0AH
    BIT 7,C
    JR Z,CB1DH

LCAFF:
    DEC DE
    LD A,(DE)
    CP 5CH
    JR Z,CB1DH
    OR A
    JR Z,CB1DH
    JR CAFFH

LCB0A:
    LD HL,0CB1H
    LD A,5CH
    LD B,09H
    CALL CAAAH
    BIT 4,C
    LD A,2EH
    LD B,04H
    CALL NZ,CAAAH

LCB1D:
    XOR A
    EX DE,HL
    LD (HL),A
    INC HL
    LD (HL),A
    POP BC
    POP DE

LCB24:
    PUSH AF
    INC C
    DEC C
    CALL NZ,C2C7H
    CALL CBBEH
    POP AF
    RET C
    CALL CB75H
    SET 5,(IX+13H)

LCB36:
    LD A,04H

LCB38:
    PUSH IX
    CALL DB2EH
    LD B,(HL)
    CALL E1DDH
    RET Z
    SCF
    RET

LCB44:
    JR NC,CB4DH
    PUSH HL
    LD HL,0D6BH
    SET 4,(HL)
    POP HL

LCB4D:
    EX AF,AF'

LCB4E:
    LD A,01H
    CALL CB38H
    LD A,B
    AND 18H
    JR NZ,CB70H

LCB58:
    PUSH BC
    PUSH DE
    CALL CBBEH
    LD A,08H
    ADD A,E
    LD E,A
    JR NC,CB64H
    INC D

LCB64:
    LD A,3FH
    LD B,0BH

LCB68:
    LD (DE),A
    INC DE
    DJNZ CB68H
    POP DE
    POP BC
    SET 5,B

LCB70:
    LD (IX+13H),B
    XOR A
    RET

LCB75:
    CALL CBBEH
    PUSH DE
    PUSH BC
    LD L,C
    LD H,B
    LD B,30H

LCB7E:
    LD A,(DE)
    LD C,(HL)
    LD (HL),A
    LD A,C
    LD (DE),A
    INC HL
    INC DE
    DJNZ CB7EH
    POP HL
    POP DE
    CALL CB58H
    LD C,L
    LD B,H
    RET

LCB8F:
    CALL CBBEH

LCB92:
    LD A,05H
    CALL CB38H
    OR A
    CALL Z,CC23H
    JR C,CB92H
    LD HL,(0D69H)
    JR NZ,CBA7H
    INC HL
    LD (0D69H),HL
    RET

LCBA7:
    CP A1H
    SCF
    RET NZ
    LD A,H
    OR L
    SUB 01H
    SBC A,A
    SCF
    RET Z
    LD A,(0D6BH)
    BIT 4,A
    LD A,A0H
    RET NZ
    SBC A,A
    LD A,A1H
    RET

LCBBE:
    PUSH HL
    PUSH IX
    POP DE
    LD HL,0030H
    ADD HL,DE
    LD C,L
    LD B,H
    POP HL
    RET

LCBCA:
    LD IX,0CD9H
    XOR A
    PUSH IX
    POP DE
    LD (IX+06H),A
    LD (IX+07H),A
    LD C,A
    DEC A
    LD (DE),A
    RET

LCBDC:
    CALL CB44H
    CALL CA8AH
    PUSH HL
    LD HL,C554H
    CALL Z,CBFDH
    POP HL
    RET

LCBEB:
    JR C,CBFDH
    INC HL
    INC HL
    INC HL
    INC HL
    JR CBFDH

LCBF3:
    LD A,(0D6FH)
    OR (HL)
    LD (0D6FH),A
    INC DE
    POP HL
    DB 0EH                                                                          ; |.|

LCBFD:
    XOR A
    SET 0,A
    LD (0D6FH),A
    EXX
    LD C,A
    EXX
    PUSH HL
    CALL CA93H
    LD A,C
    CP 2FH
    JR NZ,CC1FH

LCC0F:
    LD A,(DE)
    CALL C1D3H
    CP (HL)
    INC HL
    JR Z,CBF3H
    INC HL
    LD A,(HL)
    OR A
    JR NZ,CC0FH
    OR 8EH
    DB CAH                                                                          ; |.|

LCC1F:
    DEC DE
    XOR A
    POP HL
    RET

LCC23:
    LD A,(IX+43H)
    CPL
    LD L,A
    LD A,(0D6FH)
    OR L
    CPL
    AND 17H

LCC2F:
    LD A,A1H
    SCF
    RET NZ
    LD A,(0D6BH)
    AND L
    AND 10H
    JR NZ,CC2FH
    RET

LCC3C:
    DB 06H                                                                          ; |.|

LCC3D:
    LD B,00H
    CALL CC50H
    INC C
    JR Z,CC4AH
    CALL CC4AH
    OR A
    RET

LCC4A:
    CALL CC67H
    RET C
    RET Z
    INC HL

LCC50:
    LD C,00H

LCC52:
    DEC C
    SCF
    RET Z
    CALL CC5CH
    RET NC
    INC HL
    JR CC52H

LCC5C:
    LD A,(HL)

LCC5D:
    CP 20H
    SCF
    RET Z
    CP 09H
    SCF
    RET Z
    OR A
    RET

LCC67:
    PUSH HL
    PUSH BC
    LD HL,CC80H
    LD C,0CH
    INC B
    DEC B
    JR Z,CC78H
    LD HL,CC83H
    LD BC,0009H

LCC78:
    CPIR
    POP BC
    POP HL
    SCF
    RET NZ
    OR A
    RET
    LD A,(2E2DH)

; hbf: Nine-byte separator table, scanned with CPIR.
    DB 20H, 09H, 3BH, 2CH, 3DH, 2FH, 5BH, 5DH, 00H                                  ; | .;,=/[].|

LCC8C:
    PUSH BC
    CALL CC50H
    POP BC
    RET

LCC92:
    CALL CCB5H
    JR C,CCADH
    XOR A

LCC98:
    LD C,A
    CALL CCB5H
    LD B,A
    CCF
    SBC A,A
    RET Z
    INC HL
    LD A,C
    CALL CCB1H
    CALL NC,CCB1H
    JR C,CCADH
    ADD A,B
    JR NC,CC98H

LCCAD:
    LD A,8FH
    OR A
    RET

LCCB1:
    ADD A,C
    RET C
    ADD A,A
    RET

LCCB5:
    LD A,(HL)
    SUB 30H
    RET C
    CP 0AH
    CCF
    RET

LCCBD:
    PUSH BC
    CALL CC3CH
    POP BC
    SBC A,A
    AND 8EH
    RET NZ
    CALL CC8CH
    XOR A
    RET

LCCCB:
    PUSH AF
    LD A,(0D6DH)
    OR 02H
    LD (0D6DH),A
    POP AF
    RET

LCCD6:
    LD A,4BH

LCCD8:
    PUSH HL
    PUSH DE
    PUSH AF
    LD A,(0D6DH)
    BIT 0,A
    JR Z,CD0AH
    LD HL,(0D80H)
    PUSH HL
    EX DE,HL
    LD HL,(0BF2H)
    XOR A
    SBC HL,DE
    JR Z,CD03H
    LD HL,(0BF0H)
    POP DE
    XOR A
    ADC HL,DE
    POP AF
    LD (HL),A
    POP DE
    LD HL,(0D80H)
    INC HL
    LD (0D80H),HL
    POP HL
    XOR A
    RET

LCD03:
    LD SP,(0D7EH)
    JP C0C4H

LCD0A:
    BIT 1,A
    JR Z,CD16H
    POP AF
    CALL DB2EH
    RET PO
    IN A,(18H)
    DB 11H                                                                          ; |.|

LCD16:
    POP AF
    PUSH AF
    CP 0DH
    JR NZ,CD21H
    LD A,0BH
    CALL CCD8H

LCD21:
    POP AF
    CALL DB2EH
    CALL D1DBH
    POP HL
    RET

LCD2A:
    CALL CD42H

LCD2D:
    LD A,0DH
    CALL CCD8H
    LD A,0AH
    JR CCD8H

LCD36:
    LD A,(IX+07H)
    ADD A,40H
    CALL CCD8H
    LD A,3AH
    JR CCD8H

LCD42:
    CALL DB2EH
    EI
    IN A,(C9H)

LCD48:
    LD A,(DE)
    INC DE
    OR A
    RET Z
    CALL CCD8H
    JR CD48H

LCD51:
    LD HL,CDAFH
    LD DE,0D7BH
    LD BC,000AH
    LDDR
    INC DE
    LD C,DFH
    CALL DB2EH
    LD H,A
    RST 00H
    RST 30H
    DEC BC
    CALL DB2EH
    LD B,C
    RST 00H
    LD A,DFH
    CALL CCD8H
    LD A,20H
    CALL CCD8H

LCD75:
    CALL DB2EH
    LD H,A
    RST 00H

LCD7A:
    LD HL,0BF6H
    XOR A

LCD7E:
    LD (0BF5H),A

LCD81:
    RST 30H
    AND C
    JR NZ,CDA0H
    LD A,C
    CP DFH
    JR Z,CD7AH
    SUB 0DH
    JR Z,CDA0H
    LD A,C
    SUB 1BH
    SCF
    JR Z,CDA0H
    LD A,(0BF5H)
    INC A
    JP M,CD81H
    LD (HL),C
    INC HL
    JP CD7EH

LCDA0:
    CALL DB2EH
    LD B,C
    RST 00H
    RET
    NOP
    NOP
    NOP
    JR CDC3H
    NOP
    JR CDC6H
    NOP
    NOP

LCDB0:
    SBC A,A
    LD HL,0008H
    ADD HL,BC
    LD B,08H
    JR NZ,CDC9H
    CALL CDC1H
    CALL CDE3H

LCDBF:
    LD B,03H

LCDC1:
    LD A,(HL)
    INC HL

LCDC3:
    CALL CCD8H

LCDC6:
    DJNZ CDC1H
    RET

LCDC9:
    LD B,08H

LCDCB:
    LD A,(HL)
    INC HL
    CP 20H
    CALL NZ,CCD8H
    DJNZ CDCBH
    LD A,(HL)
    CP 20H
    JR Z,CDDBH
    LD A,2EH

LCDDB:
    CALL CCD8H
    CALL CDBFH

LCDE1:
    JR NZ,CE22H

LCDE3:
    LD A,20H
    JR CE22H

LCDE7:
    PUSH HL
    LD E,(HL)
    INC HL
    LD D,(HL)
    INC HL
    LD A,(HL)
    INC HL
    OR (HL)
    JR NZ,CE0CH
    LD A,D
    AND FCH
    JR NZ,CE0CH
    EX DE,HL
    POP DE
    LD A,1AH

LCDFA:
    PUSH HL
    PUSH AF
    CALL CEE8H
    POP AF
    CALL CD42H
    POP HL
    DEC HL
    LD A,H
    OR L
    RET Z
    LD A,73H
    JR CE22H

LCE0C:
    LD C,0AH

LCE0E:
    PUSH HL
    LD B,04H
    OR A

LCE12:
    RR (HL)
    DEC HL
    DJNZ CE12H
    POP HL
    DEC C
    JR NZ,CE0EH
    POP DE
    XOR A
    CALL CEF5H
    LD A,4BH

LCE22:
    JP CCD8H

LCE25:
    LD A,20H
    CALL CCD8H
    LD B,A
    LD A,(HL)
    RRA
    RRA
    RRA
    AND 1FH
    LD C,A
    LD A,(IY-61H)
    OR A
    LD A,C
    JP M,CE49H
    LD B,61H
    CP 0CH
    JR C,CE44H
    SUB 0CH
    LD B,70H

LCE44:
    OR A
    JR NZ,CE49H
    LD A,0CH

LCE49:
    CALL CEACH
    LD A,3AH
    CALL CCD8H
    LD A,(HL)
    DEC HL
    RL (HL)
    RLA
    RL (HL)
    RLA
    RL (HL)
    RLA
    AND 3FH
    CALL CEACH
    XOR A
    OR B
    RET Z

LCE64:
    JR CE22H

LCE66:
    LD A,(HL)
    AND 1FH
    LD C,A
    LD A,(HL)
    INC HL
    LD H,(HL)
    SRL H
    RLA
    RLA
    RLA
    RLA
    AND 0FH
    LD L,A
    LD A,(IY-61H)
    ADD A,A
    ADD A,A
    JR NZ,CE7FH
    LD A,6CH

LCE7F:
    LD B,03H
    LD E,20H

LCE83:
    RLCA
    RLCA
    PUSH AF
    AND 03H
    LD D,A
    JR Z,CE96H
    LD A,C
    DEC D
    JR Z,CE96H
    LD A,L
    DEC D
    JR Z,CE96H
    LD A,H
    ADD A,50H

LCE96:
    LD D,A
    LD A,E
    CALL CCD8H
    LD A,D
    CP 64H
    JR C,CEA2H
    SUB 64H

LCEA2:
    CALL CEACH
    POP AF
    LD E,2DH
    DJNZ CE83H
    OR A
    RET

LCEAC:
    LD E,2FH

LCEAE:
    SUB 0AH
    INC E
    JR NC,CEAEH
    LD D,A
    LD A,E
    CALL CCD8H
    LD A,D
    ADD A,3AH
    JR CE64H

LCEBD:
    PUSH AF
    CALL CD2DH
    LD A,0EH
    CALL CD42H
    POP HL
    LD A,H
    LD DE,D42DH
    CALL DB2EH
    PUSH DE
    RST 18H
    JR NC,CEDDH
    LD A,0AH
    DB CDH                                                                          ; |.|

LCED5:
    LD B,D
    CALL CD7CH
    PUSH HL
    ADC A,18H
    DEC B

LCEDD:
    CALL DB2EH
    LD (BC),A
    DB DCH                                                                          ; |.|

LCEE2:
    JP CD2DH

LCEE5:
    LD L,A
    LD H,00H

LCEE8:
    EX DE,HL
    LD HL,0C78H
    XOR A
    LD (HL),A
    DEC HL
    LD (HL),A
    DEC HL
    LD (HL),D
    DEC HL
    LD (HL),E
    EX DE,HL

LCEF5:
    EXX
    LD E,A
    LD D,30H
    EXX
    LD HL,CF2DH

LCEFD:
    LD A,(HL)
    DEC A
    JR NZ,CF04H
    EXX
    LD E,D
    EXX

LCF04:
    XOR A
    LD C,A

LCF06:
    PUSH HL
    PUSH DE
    LD B,04H

LCF0A:
    LD A,(DE)
    SBC A,(HL)
    LD (DE),A
    INC HL
    INC DE
    DJNZ CF0AH
    POP DE
    POP HL
    INC C
    JR NC,CF06H
    CALL CF4EH
    LD A,C
    EXX
    DEC A
    JR Z,CF21H
    ADD A,D
    LD E,D
    DB 28H                                                                          ; |(|

LCF21:
    OR E
    JR Z,CF27H
    CALL CCD8H

LCF27:
    EXX
    LD A,(HL)
    INC A
    JR NZ,CEFDH
    RET
    ADD A,B
    SUB (HL)
    SBC A,B
    NOP
    LD B,B
    LD B,D
    RRCA
    NOP
    AND B
    ADD A,(HL)
    LD BC,1000H
    DAA
    NOP
    NOP
    RET PE
    INC BC
    NOP
    NOP
    LD H,H
    NOP
    NOP
    NOP
    LD A,(BC)
    NOP
    NOP
    NOP
    LD BC,0000H
    NOP
    RST 38H

LCF4E:
    PUSH DE
    LD B,04H
    OR A

LCF52:
    LD A,(DE)
    ADC A,(HL)
    LD (DE),A
    INC HL
    INC DE
    DJNZ CF52H
    POP DE
    RET
    JR CF62H
    CALL DB2EH
    LD (BC),A
    DB DCH                                                                          ; |.|

LCF62:
    JP CD2DH
    LD L,A
    LD H,00H
    EX DE,HL
    LD HL,0C78H
    XOR A
    LD (HL),A
    DEC HL
    LD (HL),A
    DEC HL
    LD (HL),D
    DEC HL
    LD (HL),E
    EX DE,HL
    EXX
    LD E,A
    LD D,30H
    EXX
    LD HL,CF2DH
    LD A,(HL)
    DEC A
    JR NZ,CFF3H

; hbf: Unused FFH fill through the end of the page.
    DB FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH ; |................|
    DB FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH ; |................|
    DB FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH ; |................|
    DB FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH ; |................|
    DB FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH ; |................|
    DB FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH ; |................|
    DB FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH ; |................|
    DB FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH, FFH    ; |...............|
