; USR entry for pong: owns the IM 1 vector while the game runs.
;
;   10 LET R=USR(@{irq::start},0)
;
; HL (the USR argument) goes through to pong::play, and play's HL result
; comes back to BASIC.
;
; 0038H is a RAM stub in U0 (PUSH AF / LD A,70H / OUT (02H),A / JP C412H)
; that enters the ROM handler. The game draws everything and scans the
; keyboard itself, so no ROM code needs to run: start saves the stub's
; first three bytes, writes JP FRAME over them, and puts them back when
; play returns. Interrupts are off while the vector is half written.
;
; C80 cannot take a function's address or write an interrupt entry, so the
; handler and the hook live here. ASM can call C80, not the other way.

start:
        DI
        LD A,(0038H)
        LD (SAVED),A
        LD DE,(0039H)
        LD (SAVED+1),DE
        LD A,0C3H               ; JP FRAME
        LD (0038H),A
        LD DE,FRAME
        LD (0039H),DE
        CALL @{pong::play}      ; HL in and out; returns with interrupts off
        LD A,(SAVED)
        LD (0038H),A
        LD DE,(SAVED+1)
        LD (0039H),DE
        EI                      ; the ROM handler takes any pending request
        RET

; Frame interrupt: the CRTC cursor at the last paper byte (port 59H bit 4).
; play keeps interrupts disabled except for EI / HALT in wait_frame, so the
; only thing to do is acknowledge. OUT (n),A changes no register or flag.
; No EI: interrupts stay off after RET until the next wait_frame.
FRAME:
        OUT (07H),A
        RET

SAVED:
        DS 3
