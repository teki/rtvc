; TVC chunky hardware scroll demo.
;
; Inspired by Kevin Thacker's (arnoldemu) CPC "chunky" R12/R13 demos in
; https://forum.cpcwiki.eu/t/hardware-scrolling-help/488 (MIT). CPC sources
; live under tmp/hwscroll/ after download; this file is a TVC-native rewrite.
;
; Starts scrolling immediately (no key wait). Border flashes each frame so a
; hang in WAIT_FRAME is obvious. Auto-scrolls one byte per frame.
;
; Assemble and inject from a booted snapshot:
;   mkdir -p target/coding
;   cargo run --bin rtvc-asm -- --format cas \
;     coding/hwscroll-chunky.asm \
;     -o target/coding/hwscroll-chunky.cas
;   cargo run --bin rtvc -- snapshots/boot12dos.rtvcsnap.zip \
;     -i target/coding/hwscroll-chunky.cas
;
; Click the Screen pane for keys:
;   O/P or left/right  horizontal
;   Q/A or up/down     vertical (one CRTC char row = 4 scanlines)
;   Space              auto horizontal on/off

        BASIC_START

VID_MODE EQU C9F4H
CRTC_AD  EQU 70H
CRTC_DT  EQU 71H
CHAR_W   EQU 64
LAST_OFF EQU 0EFFH
VSTEP    EQU 64

        EI
        LD C,00H
        CALL VID_MODE
        XOR A
        OUT (60H),A
        LD A,55H
        OUT (61H),A
        LD A,02H
        OUT (00H),A

        CALL PAGE_VID
        CALL FILL_PATTERN
        CALL PAGE_REST

        ; Take over the frame tick before any wait. VT-DOS leaves R14/R15 at
        ; 0AFFH; without restoring a cursor in the visible area WAIT_FRAME hangs.
        DI
        XOR A
        LD (SCROLL),A
        LD (SCROLL+1),A
        CALL SET_SCROLL
        XOR A
        OUT (07H),A

MAIN:
        CALL WAIT_FRAME
        ; Frame alive marker: toggle border each tick.
        LD A,(BORDER)
        XOR 02H
        LD (BORDER),A
        OUT (00H),A

        CALL POLL_KEYS
        LD A,(AUTO)
        OR A
        JR Z,NO_AUTO
        LD HL,(SCROLL)
        INC HL
        LD (SCROLL),HL
NO_AUTO:
        CALL SET_SCROLL
        JR MAIN

;------------------------------------------------------------------------------
WAIT_FRAME:
        XOR A
        OUT (07H),A
WF_WAIT:
        IN A,(59H)
        AND 10H
        JR NZ,WF_WAIT
        XOR A
        OUT (07H),A
        RET

;------------------------------------------------------------------------------
; Cursor must stay on the last visible CRTC address (start + 0EFFH). Do not
; truncate the sum to 12 bits: MA is 14-bit, and masking made the IRQ miss
; once SCROLL passed 0100H, which hung WAIT_FRAME.
FIX_IRQ_CURSOR:
        LD HL,(SCROLL)
        LD A,H
        AND 0FH
        LD H,A
        LD DE,LAST_OFF
        ADD HL,DE
        LD A,0EH
        OUT (CRTC_AD),A
        LD A,H
        AND 3FH
        OUT (CRTC_DT),A
        LD A,0FH
        OUT (CRTC_AD),A
        LD A,L
        OUT (CRTC_DT),A
        RET

SET_SCROLL:
        LD HL,(SCROLL)
        LD A,H
        AND 0FH
        LD H,A
        LD (SCROLL),HL
        LD A,0CH
        OUT (CRTC_AD),A
        LD A,H
        OUT (CRTC_DT),A
        LD A,0DH
        OUT (CRTC_AD),A
        LD A,L
        OUT (CRTC_DT),A
        CALL FIX_IRQ_CURSOR
        RET

;------------------------------------------------------------------------------
POLL_KEYS:
        LD A,02H
        OUT (03H),A
        IN A,(58H)
        BIT 6,A
        JR NZ,NO_Q
        CALL SCROLL_UP
NO_Q:
        LD A,04H
        OUT (03H),A
        IN A,(58H)
        BIT 6,A
        JR NZ,NO_A
        CALL SCROLL_DOWN
NO_A:
        LD A,03H
        OUT (03H),A
        IN A,(58H)
        LD B,A
        BIT 2,B
        JR NZ,NO_O
        CALL SCROLL_LEFT
NO_O:
        BIT 6,B
        JR NZ,NO_P
        CALL SCROLL_RIGHT
NO_P:
        LD A,08H
        OUT (03H),A
        IN A,(58H)
        LD B,A
        BIT 1,B
        JR NZ,NO_CU
        CALL SCROLL_UP
NO_CU:
        BIT 2,B
        JR NZ,NO_CD
        CALL SCROLL_DOWN
NO_CD:
        BIT 6,B
        JR NZ,NO_CL
        CALL SCROLL_LEFT
NO_CL:
        BIT 5,B
        JR NZ,NO_CR
        CALL SCROLL_RIGHT
NO_CR:
        LD A,07H
        OUT (03H),A
        IN A,(58H)
        BIT 5,A
        JR NZ,SP_UP
        CALL TOGGLE_AUTO
        RET
SP_UP:
        XOR A
        LD (SPPREV),A
        RET

SCROLL_LEFT:
        LD HL,(SCROLL)
        DEC HL
        LD (SCROLL),HL
        XOR A
        LD (AUTO),A
        RET

SCROLL_RIGHT:
        LD HL,(SCROLL)
        INC HL
        LD (SCROLL),HL
        XOR A
        LD (AUTO),A
        RET

SCROLL_UP:
        LD HL,(SCROLL)
        LD DE,VSTEP
        OR A
        SBC HL,DE
        LD (SCROLL),HL
        XOR A
        LD (AUTO),A
        RET

SCROLL_DOWN:
        LD HL,(SCROLL)
        LD DE,VSTEP
        ADD HL,DE
        LD (SCROLL),HL
        XOR A
        LD (AUTO),A
        RET

TOGGLE_AUTO:
        LD A,(SPPREV)
        OR A
        RET NZ
        LD A,01H
        LD (SPPREV),A
        LD A,(AUTO)
        XOR 01H
        LD (AUTO),A
        RET

;------------------------------------------------------------------------------
; Narrow vertical bars (1 on / 3 off) plus a full horizontal rule every 4
; character rows so both axes read clearly while scrolling.
FILL_PATTERN:
        LD HL,8000H
        LD B,00H
FP_BLOCK:
        LD C,00H
FP_RASTER:
        LD D,00H
FP_COL:
        LD A,D
        AND 03H
        LD A,0FFH
        JR Z,FP_BYTE
        LD A,00H
FP_BYTE:
        LD (HL),A
        INC HL
        INC D
        LD A,D
        CP CHAR_W
        JR C,FP_COL
        INC C
        LD A,C
        CP 04H
        JR C,FP_RASTER
        LD A,B
        AND 03H
        JR NZ,FP_NEXT
        PUSH BC
        PUSH HL
        LD DE,0FFC0H
        ADD HL,DE
        LD B,CHAR_W
FP_RULE:
        LD (HL),0FFH
        INC HL
        DJNZ FP_RULE
        POP HL
        POP BC
FP_NEXT:
        INC B
        LD A,B
        CP 40H
        JR C,FP_BLOCK
        RET

PAGE_VID:
        LD A,(0003H)
        LD (PAGESV),A
        LD A,50H
        LD (0003H),A
        OUT (02H),A
        RET

PAGE_REST:
        LD A,(PAGESV)
        LD (0003H),A
        OUT (02H),A
        RET

SCROLL: DW 0
AUTO:   DB 1
SPPREV: DB 0
BORDER: DB 02H
PAGESV: DB 0
