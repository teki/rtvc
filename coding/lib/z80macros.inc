; z80macros.inc: Z80 sequences the CPU has no single instruction for.
;
; INCLUDE "z80macros.inc" (with `rtvc-asm -I coding/lib`). Every macro is a
; plain sequence of real instructions; its comment gives the T-states, the
; bytes, and everything it changes, so the cost stays visible at the call
; site. "Changes" lists every register and flag the macro writes. F means
; the flags are left in an undefined or unhelpful state unless the comment
; says what they mean. The test `z80macros_match_their_documented_costs`
; runs each one and checks these numbers.

; ---------------------------------------------------------------------------
; 16-bit register moves. LD_PAIR takes the halves: LD_PAIR H,L,D,E is
; LD HL,DE.

; LD_PAIR dhi, dlo, shi, slo: copy the pair shi:slo into dhi:dlo.
; 8 T, 2 bytes. Changes: dhi, dlo.
        MACRO LD_PAIR dhi, dlo, shi, slo
        LD dhi,shi
        LD dlo,slo
        ENDM

; LD_HL_DE: HL = DE.
; 8 T, 2 bytes. Changes: HL.
        MACRO LD_HL_DE
        LD_PAIR H,L,D,E
        ENDM

; LD_HL_BC: HL = BC.
; 8 T, 2 bytes. Changes: HL.
        MACRO LD_HL_BC
        LD_PAIR H,L,B,C
        ENDM

; LD_DE_HL: DE = HL. EX DE,HL (4 T, 1 byte) is cheaper when HL may change.
; 8 T, 2 bytes. Changes: DE.
        MACRO LD_DE_HL
        LD_PAIR D,E,H,L
        ENDM

; LD_DE_BC: DE = BC.
; 8 T, 2 bytes. Changes: DE.
        MACRO LD_DE_BC
        LD_PAIR D,E,B,C
        ENDM

; LD_BC_HL: BC = HL.
; 8 T, 2 bytes. Changes: BC.
        MACRO LD_BC_HL
        LD_PAIR B,C,H,L
        ENDM

; LD_BC_DE: BC = DE.
; 8 T, 2 bytes. Changes: BC.
        MACRO LD_BC_DE
        LD_PAIR B,C,D,E
        ENDM

; ---------------------------------------------------------------------------
; 16-bit arithmetic on HL. pair is BC, DE, HL or SP.

; SUB16_HL pair: HL = HL - pair, without the carry SBC would subtract.
; 19 T, 3 bytes. Changes: HL, F (C set when HL was below pair).
        MACRO SUB16_HL pair
        OR A
        SBC HL,pair
        ENDM

; CP16_HL pair: compare HL with pair and keep HL.
; 30 T, 4 bytes. Changes: F (C set when HL < pair, Z set when equal).
        MACRO CP16_HL pair
        OR A
        SBC HL,pair
        ADD HL,pair
        ENDM

; NEG_HL: HL = 0 - HL.
; 24 T, 6 bytes. Changes: HL, A, F.
        MACRO NEG_HL
        XOR A
        SUB L
        LD L,A
        SBC A,A
        SUB H
        LD H,A
        ENDM

; ADD_A_TO_PAIR hi, lo: hi:lo = hi:lo + A, A unsigned.
; 20 T, 5 bytes. Changes: hi, lo, A, F.
        MACRO ADD_A_TO_PAIR hi, lo
        ADD A,lo
        LD lo,A
        ADC A,hi
        SUB lo
        LD hi,A
        ENDM

; ADD_A_TO_HL: HL = HL + A, A unsigned.
; 20 T, 5 bytes. Changes: HL, A, F.
        MACRO ADD_A_TO_HL
        ADD_A_TO_PAIR H,L
        ENDM

; ADD_A_TO_DE: DE = DE + A, A unsigned.
; 20 T, 5 bytes. Changes: DE, A, F.
        MACRO ADD_A_TO_DE
        ADD_A_TO_PAIR D,E
        ENDM

; ADD_A_TO_BC: BC = BC + A, A unsigned.
; 20 T, 5 bytes. Changes: BC, A, F.
        MACRO ADD_A_TO_BC
        ADD_A_TO_PAIR B,C
        ENDM

; ---------------------------------------------------------------------------
; Loops and unrolled block instructions.

; DEC_BC_JR_NZ target: BC = BC - 1, then JR NZ,target while BC is not 0.
; DEC BC alone sets no flags, which is why this needs A.
; 21 T falling through, 26 T jumping; 5 bytes. Changes: BC, A, F.
        MACRO DEC_BC_JR_NZ target
        DEC BC
        LD A,B
        OR C
        JR NZ,target
        ENDM

; DELAY_B count: busy-wait with DJNZ; count 1-256 (0 is 256).
; 13*count+2 T, 4 bytes. Changes: B (0). Flags are kept.
        MACRO DELAY_B count
        LD B,LOW(count)
.wait:  DJNZ .wait
        ENDM

; OUTI_N count: count unrolled OUTI, sending (HL) to port C.
; 16*count T, 2*count bytes. Changes: HL, B, F.
        MACRO OUTI_N count
        REPT count
        OUTI
        ENDR
        ENDM

; LDI_N count: count unrolled LDI, copying (HL) to (DE).
; 16*count T, 2*count bytes. Changes: HL, DE, BC, F.
        MACRO LDI_N count
        REPT count
        LDI
        ENDR
        ENDM
