; rtvc host file card ROM.
;
; Original rtvc code (MIT). Registers a MOPS device named FILE whose cassette-class
; routines forward to the card I/O ports; CAS container handling is done by the
; emulator. Port protocol: info/rtvc.md, "Host file card".
;
; Assemble:
;   cargo run --bin rtvc-asm -- --format bin --origin C000H \
;       roms/RTVC-HOSTFILE.asm -o roms/RTVC-HOSTFILE.ROM

P_DATA          EQU 1
P_NAME          EQU 2
P_FLAGS         EQU 3
P_PARAM         EQU 4
P_CARD          EQU 8

C_RESET         EQU 01H
C_NAME_CLEAR    EQU 02H
C_OPEN_IN       EQU 03H
C_OPEN_OUT      EQU 04H
C_CLOSE_IN      EQU 05H
C_CLOSE_OUT     EQU 06H
C_IN_SEEK       EQU 07H
C_OUT_SEEK      EQU 0AH

IN_SELECT       EQU 0B05H           ; cassette class input selector
OUT_SELECT      EQU 0B0DH           ; cassette class output selector
BUFFER          EQU 0B6BH           ; cassette buffered-file flag
EOF_FLAG        EQU 0B6EH           ; FFH after the last input byte

ST_VERIFY       EQU 0E8H

        ORG C000H

        DB "MOPS"
        DB 4
        DB "FILE"
        DW IRQ                      ; C009H
        DW INIT                     ; C00BH
        DB 9                        ; C00DH: function count
        DW IRQ                      ; 0: interrupt
        DW CHARIO                   ; 1: character I/O
        DW BLOCKIO                  ; 2: block I/O
        DW OPEN                     ; 3: open input / create output
        DW CLOSE                    ; 4: close
        DW VERIFY                   ; 5: verify
        DW SEEKSET                  ; 6: seek from start
        DW SEEKCUR                  ; 7: seek from current position
        DW SEEKEND                  ; 8: seek from end

; A = slot number derived from IX (card table entry or entry+8). Keeps other registers.
SLOTNUM:
        PUSH IX
        EX (SP),HL
        LD A,L
        POP HL
        AND 0F0H
        SUB 10H
        RLCA
        RLCA
        AND 03H
        RET

; A = card port base (10H/20H/30H/40H). Keeps other registers.
PORTB:  CALL SLOTNUM
        INC A
        RLCA
        RLCA
        RLCA
        RLCA
        RET

; Updates EOF_FLAG from the card's input-exhausted flag. Keeps all registers.
EOFUPD: PUSH AF
        PUSH BC
        CALL PORTB
        ADD A,P_FLAGS
        LD C,A
        IN A,(C)
        RRCA
        SBC A,A
        LD (EOF_FLAG),A
        POP BC
        POP AF
        RET

; Init: IX = card table entry. Closes files and optionally claims channel 5.
INIT:   PUSH BC
        CALL PORTB
        LD C,A
        LD A,C_RESET
        OUT (C),A
        LD A,C
        ADD A,P_CARD
        LD C,A
        IN A,(C)
        RRCA
        JR NC,INITEND
        CALL SLOTNUM
        OR 80H
        LD (IN_SELECT),A
        LD (OUT_SELECT),A
INITEND:
        POP BC
IRQ:    XOR A
        RET

; Character I/O. Input returns C = byte; output sends C.
CHARIO: JP P,CHOUT
        PUSH BC
        CALL PORTB
        INC A
        LD C,A
        IN B,(C)
        DEC C
        IN A,(C)
        CALL EOFUPD
        LD H,A
        LD A,B
        POP BC
        LD C,A
        LD A,H
        OR A
        RET

CHOUT:  PUSH BC
        CALL PORTB
        LD B,C
        INC A
        LD C,A
        OUT (C),B
        DEC C
        IN A,(C)
        POP BC
        OR A
        RET

; Block I/O. DE = buffer, BC = length. Returns DE = next address, BC = bytes left.
BLOCKIO:
        EX DE,HL
        LD D,B
        LD E,C
        LD B,A
        CALL PORTB
        INC A
        LD C,A
        LD A,B
        RLA
        JR NC,BLKOUT
BLKIN:  LD A,D
        OR E
        JR Z,BLKINOK
        IN B,(C)
        DEC C
        IN A,(C)
        INC C
        OR A
        JR NZ,BLKINEND
        LD (HL),B
        INC HL
        DEC DE
        JR BLKIN
BLKINOK:
        XOR A
BLKINEND:
        CALL EOFUPD
        JR BLKEND

BLKOUT: LD A,D
        OR E
        JR Z,BLKOK
        LD B,(HL)
        OUT (C),B
        DEC C
        IN A,(C)
        INC C
        OR A
        JR NZ,BLKEND
        INC HL
        DEC DE
        JR BLKOUT

BLKOK:  XOR A
BLKEND: EX DE,HL
        LD B,H
        LD C,L
        OR A
        RET

; Verify. DE = buffer, BC = length. Stops with E8H at the first difference.
VERIFY: EX DE,HL
        LD D,B
        LD E,C
        CALL PORTB
        INC A
        LD C,A
VFLOOP: LD A,D
        OR E
        JR Z,BLKOK
        IN B,(C)
        DEC C
        IN A,(C)
        INC C
        OR A
        JR NZ,BLKEND
        LD A,(HL)
        CP B
        LD A,ST_VERIFY
        JR NZ,BLKEND
        INC HL
        DEC DE
        JR VFLOOP

; Open input or create output. DE = length-prefixed file name.
OPEN:   PUSH BC
        PUSH DE
        LD B,A
        CALL PORTB
        LD C,A
        LD A,C_NAME_CLEAR
        OUT (C),A
        INC C
        INC C
        LD A,(DE)
        OR A
        JR Z,OPNAMED
        EX DE,HL
        INC HL
        LD D,B
        LD B,A
        OTIR
        LD B,D
OPNAMED:
        DEC C
        DEC C
        LD A,B
        RLA
        JR NC,OPOUT
        LD A,C_OPEN_IN
        OUT (C),A
        IN A,(C)
        OR A
        JR NZ,OPEND
        LD A,C
        ADD A,P_PARAM
        LD C,A
        IN A,(C)
        SUB 11H
        LD (BUFFER),A
        CALL EOFUPD
        XOR A
        JR OPEND

OPOUT:  LD A,C
        ADD A,P_PARAM
        LD C,A
        LD A,(BUFFER)
        OR A
        LD A,11H
        JR Z,OPTYPE
        LD A,01H
OPTYPE: OUT (C),A
        LD A,C
        SUB P_PARAM
        LD C,A
        LD A,C_OPEN_OUT
        OUT (C),A
        IN A,(C)
OPEND:  POP DE
        POP BC
        OR A
        RET

; Close the input or output file.
CLOSE:  PUSH BC
        LD B,A
        CALL PORTB
        LD C,A
        LD A,B
        RLA
        LD A,C_CLOSE_IN
        JR C,CLDO
        LD A,C_CLOSE_OUT
CLDO:   OUT (C),A
        IN A,(C)
        POP BC
        OR A
        RET

; Seek. BC:DE = offset (signed for current/end). Returns BC:DE = new position.
SEEKSET:
        LD H,0
        JR SEEK
SEEKCUR:
        LD H,1
        JR SEEK
SEEKEND:
        LD H,2
SEEK:   LD L,A
        CALL PORTB
        ADD A,P_PARAM
        PUSH BC
        LD C,A
        OUT (C),E
        INC C
        OUT (C),D
        INC C
        POP DE
        OUT (C),E
        INC C
        OUT (C),D
        LD A,C
        SUB P_PARAM+3
        LD C,A
        LD A,L
        RLA
        LD A,C_IN_SEEK
        JR C,SKDIR
        LD A,C_OUT_SEEK
SKDIR:  ADD A,H
        OUT (C),A
        IN A,(C)
        LD L,A
        LD A,C
        ADD A,P_PARAM
        LD C,A
        IN E,(C)
        INC C
        IN D,(C)
        INC C
        IN H,(C)
        INC C
        IN B,(C)
        LD C,H
        LD A,L
        OR A
        RET
