ORG 0x3000
G71_STATE:
        DB 0
G72_PSCORE:
        DB 0
G73_ASCORE:
        DB 0
G74_PY:
        DB 0
G75_AY:
        DB 0
G76_BX:
        DB 0
G77_BY:
        DB 0
G78_VX:
        DB 0
G79_VY:
        DB 0
G80_DRAW_COL:
        DB 0
G81_DRAW_VAL:
        DB 0
G82_DRAW_H:
        DB 0
FN151_FILL_ROWS:
        LD C,A
FN151_FILL_ROWS_B3:
        LD A,(G82_DRAW_H)
        LD B,A
        OR A
        JR Z,FN151_FILL_ROWS_B6
FN151_FILL_ROWS_B5:
        ; inline put_row
        LD L,C
        LD H,0
        XOR A
        SRL H
        RR L
        RRA
        SRL H
        RR L
        RRA
        LD H,L
        LD L,A
        LD A,H
        ADD A,128
        LD H,A
        LD A,(G80_DRAW_COL)
        LD D,H
        LD E,L
        LD L,A
        LD H,0
        ADD HL,DE
        LD A,(G81_DRAW_VAL)
        LD (HL),A
        INC C
        DJNZ FN151_FILL_ROWS_B5
FN151_FILL_ROWS_B4:
FN151_FILL_ROWS_B6:
        RET
FN176_PLACE_BALL:
FN176_PLACE_BALL_B8:
        LD A,32
        LD (G76_BX),A
        LD A,118
        LD (G77_BY),A
        XOR A
        LD (G78_VX),A
        XOR A
        LD (G79_VY),A
        XOR A
        LD (G71_STATE),A
        RET
        ; preserves B C D E H L (inferred)
FN196_HITS:
        LD C,A
FN196_HITS_B9:
        LD A,(G77_BY)
        LD B,A
        LD A,C
        ADD A,24
        LD E,A
        LD A,B
        CP E
        JR NC,FN196_HITS_B12
FN196_HITS_B13:
        LD A,(G77_BY)
        ADD A,4
        LD B,A
        LD A,C
        CP B
        JR NC,FN196_HITS_B12
FN196_HITS_B10:
        LD A,1
        RET
FN196_HITS_B11:
FN196_HITS_B12:
        XOR A
        RET
        ; preserves D H L (inferred)
FN267_SCORED:
        LD C,A
FN267_SCORED_B14:
        LD A,4
        LD (G82_DRAW_H),A
        LD A,255
        LD (G81_DRAW_VAL),A
        LD A,C
        OR A
        JR Z,FN267_SCORED_B16
FN267_SCORED_B15:
        LD A,(G72_PSCORE)
        INC A
        LD (G72_PSCORE),A
        LD A,(G72_PSCORE)
        LD B,A
        XOR A
        LD (G80_DRAW_COL),A
        JP FN267_SCORED_B17
FN267_SCORED_B16:
        LD A,(G73_ASCORE)
        INC A
        LD (G73_ASCORE),A
        LD A,(G73_ASCORE)
        LD B,A
        LD A,63
        LD (G80_DRAW_COL),A
FN267_SCORED_B17:
        LD A,B
        ADD A,A
        ADD A,A
        ADD A,A
        PUSH BC ; spill
        CALL FN151_FILL_ROWS
        POP BC ; spill
        LD A,B
        CP 5
        JR NZ,FN267_SCORED_B20
FN267_SCORED_B18:
        LD A,C
        OR A
        JR Z,FN267_SCORED_B22
FN267_SCORED_B21:
        LD A,2
        LD (G71_STATE),A
        JP FN267_SCORED_B23
FN267_SCORED_B22:
        LD A,3
        LD (G71_STATE),A
FN267_SCORED_B23:
        RET
FN267_SCORED_B19:
FN267_SCORED_B20:
        JP FN176_PLACE_BALL
        ; spill report: 0 slot bytes, saves (bytes per site) 2, 21 T-states of spill code
FN375_MOVE_PLAYER:
FN375_MOVE_PLAYER_B24:
        ; inline read_row
        LD A,8
        OUT (3),A
        IN A,(88)
        LD C,A
        AND 2
        OR A
        JR NZ,FN375_MOVE_PLAYER_B27
FN375_MOVE_PLAYER_B25:
        LD A,(G74_PY)
        CP 2
        JR C,FN375_MOVE_PLAYER_B27
FN375_MOVE_PLAYER_B28:
        LD A,(G74_PY)
        SUB 2
        LD (G74_PY),A
        LD A,2
        LD (G80_DRAW_COL),A
        LD (G82_DRAW_H),A
        LD A,255
        LD (G81_DRAW_VAL),A
        LD A,(G74_PY)
        PUSH BC ; spill
        CALL FN151_FILL_ROWS
        POP BC ; spill
        XOR A
        LD (G81_DRAW_VAL),A
        LD A,(G74_PY)
        ADD A,24
        PUSH BC ; spill
        CALL FN151_FILL_ROWS
        POP BC ; spill
FN375_MOVE_PLAYER_B29:
FN375_MOVE_PLAYER_B30:
FN375_MOVE_PLAYER_B26:
FN375_MOVE_PLAYER_B27:
        LD A,C
        AND 4
        OR A
        JR NZ,FN375_MOVE_PLAYER_B33
FN375_MOVE_PLAYER_B31:
        LD A,(G74_PY)
        CP 214
        JR NC,FN375_MOVE_PLAYER_B33
FN375_MOVE_PLAYER_B34:
        LD A,(G74_PY)
        ADD A,2
        LD (G74_PY),A
        LD A,2
        LD (G80_DRAW_COL),A
        LD (G82_DRAW_H),A
        XOR A
        LD (G81_DRAW_VAL),A
        LD A,(G74_PY)
        SUB 2
        CALL FN151_FILL_ROWS
        LD A,255
        LD (G81_DRAW_VAL),A
        LD A,(G74_PY)
        ADD A,22
        CALL FN151_FILL_ROWS
FN375_MOVE_PLAYER_B35:
FN375_MOVE_PLAYER_B36:
FN375_MOVE_PLAYER_B32:
FN375_MOVE_PLAYER_B33:
        RET
        ; spill report: 0 slot bytes, saves (bytes per site) 2 2, 42 T-states of spill code
FN630_BALL_SYNC:
        LD B,C
        LD C,A
FN630_BALL_SYNC_B58:
        LD A,(G76_BX)
        CP C
        JR NZ,FN630_BALL_SYNC_B61
FN630_BALL_SYNC_B59:
        LD A,(G77_BY)
        CP B
        JR NZ,FN630_BALL_SYNC_B64
FN630_BALL_SYNC_B62:
        RET
FN630_BALL_SYNC_B63:
FN630_BALL_SYNC_B64:
        LD A,(G76_BX)
        LD (G80_DRAW_COL),A
        LD A,(G77_BY)
        CP B
        JR NC,FN630_BALL_SYNC_B66
FN630_BALL_SYNC_B65:
        LD A,(G77_BY)
        LD C,A
        LD A,B
        SUB C
        LD (G82_DRAW_H),A
        LD A,255
        LD (G81_DRAW_VAL),A
        LD A,(G77_BY)
        CALL FN151_FILL_ROWS
        XOR A
        LD (G81_DRAW_VAL),A
        LD A,(G77_BY)
        ADD A,4
        CALL FN151_FILL_ROWS
        JP FN630_BALL_SYNC_B67
FN630_BALL_SYNC_B66:
        LD A,(G77_BY)
        SUB B
        LD (G82_DRAW_H),A
        XOR A
        LD (G81_DRAW_VAL),A
        PUSH BC ; spill
        LD A,B
        CALL FN151_FILL_ROWS
        POP BC ; spill
        LD A,255
        LD (G81_DRAW_VAL),A
        LD A,B
        ADD A,4
        CALL FN151_FILL_ROWS
FN630_BALL_SYNC_B67:
        RET
FN630_BALL_SYNC_B60:
FN630_BALL_SYNC_B61:
        LD A,4
        LD (G82_DRAW_H),A
        LD A,C
        LD (G80_DRAW_COL),A
        XOR A
        LD (G81_DRAW_VAL),A
        LD A,B
        CALL FN151_FILL_ROWS
        LD A,(G76_BX)
        LD (G80_DRAW_COL),A
        LD A,255
        LD (G81_DRAW_VAL),A
        LD A,(G77_BY)
        JP FN151_FILL_ROWS
        ; spill report: 0 slot bytes, saves (bytes per site) 2, 21 T-states of spill code
FN757_PHYSICS:
FN757_PHYSICS_B68:
        LD A,(G78_VX)
        OR A
        JR NZ,FN757_PHYSICS_B71
FN757_PHYSICS_B69:
        RET
FN757_PHYSICS_B70:
FN757_PHYSICS_B71:
        LD A,(G76_BX)
        LD C,A
        LD A,(G77_BY)
        LD B,A
        LD A,(G79_VY)
        XOR 128
        CP 128
        JR NC,FN757_PHYSICS_B73
FN757_PHYSICS_B72:
        LD A,(G77_BY)
        OR A
        JR NZ,FN757_PHYSICS_B76
FN757_PHYSICS_B75:
        LD A,1
        LD (G79_VY),A
        JP FN757_PHYSICS_B74
FN757_PHYSICS_B76:
        LD A,(G77_BY)
        DEC A
        LD (G77_BY),A
        JP FN757_PHYSICS_B74
FN757_PHYSICS_B77:
FN757_PHYSICS_B73:
        LD A,(G77_BY)
        CP 236
        JR C,FN757_PHYSICS_B79
FN757_PHYSICS_B78:
        LD A,255
        LD (G79_VY),A
        JP FN757_PHYSICS_B74
FN757_PHYSICS_B79:
        LD A,(G77_BY)
        INC A
        LD (G77_BY),A
FN757_PHYSICS_B80:
FN757_PHYSICS_B74:
        LD A,(G78_VX)
        XOR 128
        CP 128
        JR NC,FN757_PHYSICS_B82
FN757_PHYSICS_B81:
        LD A,(G76_BX)
        CP 3
        JR Z,FN757_PHYSICS_B85
FN757_PHYSICS_B84:
        LD A,(G76_BX)
        DEC A
        LD (G76_BX),A
        JP FN757_PHYSICS_B83
FN757_PHYSICS_B85:
        LD A,(G74_PY)
        LD D,B ; spill
        LD L,C ; spill
        CALL FN196_HITS
        LD C,L ; spill
        LD B,D ; spill
        OR A
        JR Z,FN757_PHYSICS_B88
FN757_PHYSICS_B87:
        LD A,1
        LD (G78_VX),A
        JP FN757_PHYSICS_B83
FN757_PHYSICS_B88:
        PUSH BC ; spill
        XOR A
        CALL FN267_SCORED
        POP BC ; spill
        JP FN757_PHYSICS_B83
FN757_PHYSICS_B89:
FN757_PHYSICS_B86:
FN757_PHYSICS_B82:
        LD A,(G76_BX)
        CP 60
        JR Z,FN757_PHYSICS_B91
FN757_PHYSICS_B90:
        LD A,(G76_BX)
        INC A
        LD (G76_BX),A
        JP FN757_PHYSICS_B83
FN757_PHYSICS_B91:
        LD A,(G75_AY)
        LD D,B ; spill
        LD L,C ; spill
        CALL FN196_HITS
        LD C,L ; spill
        LD B,D ; spill
        OR A
        JR Z,FN757_PHYSICS_B94
FN757_PHYSICS_B93:
        LD A,255
        LD (G78_VX),A
        JP FN757_PHYSICS_B83
FN757_PHYSICS_B94:
        PUSH BC ; spill
        LD A,1
        CALL FN267_SCORED
        POP BC ; spill
FN757_PHYSICS_B95:
FN757_PHYSICS_B92:
FN757_PHYSICS_B83:
        LD A,C
        LD C,B
        JP FN630_BALL_SYNC
        ; spill report: 0 slot bytes, saves (bytes per site) 2 2, 74 T-states of spill code
FN990_PLAY:
FN990_PLAY_B98:
        ; inline setup (auto)
        XOR A
        OUT (96),A
        LD A,20
        OUT (97),A
        LD A,68
        OUT (98),A
        LD A,85
        OUT (99),A
        LD A,32
        OUT (0),A
        LD HL,2835
        LD A,(HL)
        AND 252
        OR 1
        LD (HL),A
        LD A,(HL)
        OUT (6),A
        LD A,80
        OUT (2),A
        LD A,10
        OUT (112),A
        LD A,3
        OUT (113),A
        LD A,11
        OUT (112),A
        LD A,3
        OUT (113),A
        LD A,14
        OUT (112),A
        OUT (113),A
        LD A,15
        OUT (112),A
        LD A,255
        OUT (113),A
        XOR A
        LD HL,32768
        LD (HL),A
        LD DE,32769
        LD BC,15359
        LDIR
        XOR A
        LD (G72_PSCORE),A
        XOR A
        LD (G73_ASCORE),A
        LD A,108
        LD (G74_PY),A
        LD (G75_AY),A
        CALL FN176_PLACE_BALL
        LD A,255
        LD (G81_DRAW_VAL),A
        LD A,24
        LD (G82_DRAW_H),A
        LD A,2
        LD (G80_DRAW_COL),A
        LD A,(G74_PY)
        CALL FN151_FILL_ROWS
        LD A,61
        LD (G80_DRAW_COL),A
        LD A,(G75_AY)
        CALL FN151_FILL_ROWS
        LD A,4
        LD (G82_DRAW_H),A
        LD A,(G76_BX)
        LD (G80_DRAW_COL),A
        LD A,(G77_BY)
        CALL FN151_FILL_ROWS
FN990_PLAY_B99:
        LD A,(G71_STATE)
        CP 2
        JP NC,FN990_PLAY_B101
FN990_PLAY_B100:
        ; inline wait_frame (auto)
        ei
        halt
        CALL FN375_MOVE_PLAYER
        ; inline serve (auto)
        ; inline read_row
        LD A,7
        OUT (3),A
        IN A,(88)
        LD C,A
        LD A,(G78_VX)
        OR A
        JR NZ,FN990_PLAY_B109
FN990_PLAY_B106:
        LD A,C
        AND 32
        OR A
        JR NZ,FN990_PLAY_B109
FN990_PLAY_B107:
        LD A,1
        LD (G78_VX),A
        LD (G79_VY),A
        LD (G71_STATE),A
FN990_PLAY_B108:
FN990_PLAY_B109:
        ; inline move_ai (auto)
        LD A,(G78_VX)
        OR A
        JR NZ,FN990_PLAY_B112
FN990_PLAY_B111:
        JP FN990_PLAY_B126
FN990_PLAY_B112:
        LD A,(G77_BY)
        ADD A,2
        LD B,A
        LD A,(G75_AY)
        ADD A,12
        LD E,A
        LD A,B
        CP E
        JR NC,FN990_PLAY_B118
FN990_PLAY_B114:
        LD A,(G75_AY)
        OR A
        JR Z,FN990_PLAY_B126
FN990_PLAY_B115:
        LD A,(G75_AY)
        DEC A
        LD (G75_AY),A
        LD A,61
        LD (G80_DRAW_COL),A
        LD A,1
        LD (G82_DRAW_H),A
        LD A,255
        LD (G81_DRAW_VAL),A
        LD A,(G75_AY)
        CALL FN151_FILL_ROWS
        XOR A
        LD (G81_DRAW_VAL),A
        LD A,(G75_AY)
        ADD A,24
        CALL FN151_FILL_ROWS
        JP FN990_PLAY_B126
FN990_PLAY_B116:
FN990_PLAY_B117:
FN990_PLAY_B118:
        LD A,B
        CP E
        JR Z,FN990_PLAY_B126
        JR C,FN990_PLAY_B126
FN990_PLAY_B119:
        LD A,(G75_AY)
        CP 216
        JR NC,FN990_PLAY_B126
FN990_PLAY_B120:
        LD A,(G75_AY)
        INC A
        LD (G75_AY),A
        LD A,61
        LD (G80_DRAW_COL),A
        LD A,1
        LD (G82_DRAW_H),A
        XOR A
        LD (G81_DRAW_VAL),A
        LD A,(G75_AY)
        DEC A
        CALL FN151_FILL_ROWS
        LD A,255
        LD (G81_DRAW_VAL),A
        LD A,(G75_AY)
        ADD A,23
        CALL FN151_FILL_ROWS
FN990_PLAY_B121:
FN990_PLAY_B122:
FN990_PLAY_B123:
FN990_PLAY_B124:
FN990_PLAY_B125:
FN990_PLAY_B126:
        CALL FN757_PHYSICS
        JP FN990_PLAY_B99
FN990_PLAY_B101:
        ; inline restore (auto)
        LD A,112
        OUT (2),A
        LD A,(G71_STATE)
        CP 2
        JR NZ,FN990_PLAY_B104
FN990_PLAY_B102:
        LD HL,1
        RET
FN990_PLAY_B103:
FN990_PLAY_B104:
        LD HL,65535
        RET
ORG 0x2FC0
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

U1_START:
        DI
        LD A,(0038H)
        LD (U1_SAVED),A
        LD DE,(0039H)
        LD (U1_SAVED+1),DE
        LD A,0C3H               ; JP FRAME
        LD (0038H),A
        LD DE,U1_FRAME
        LD (0039H),DE
        CALL FN990_PLAY      ; HL in and out; returns with interrupts off
        LD A,(U1_SAVED)
        LD (0038H),A
        LD DE,(U1_SAVED+1)
        LD (0039H),DE
        EI                      ; the ROM handler takes any pending request
        RET

; Frame interrupt: the CRTC cursor at the last paper byte (port 59H bit 4).
; play keeps interrupts disabled except for EI / HALT in wait_frame, so the
; only thing to do is acknowledge. OUT (n),A changes no register or flag.
; No EI: interrupts stay off after RET until the next wait_frame.
U1_FRAME:
        OUT (07H),A
        RET

U1_SAVED:
        DS 3
ORG 0x19EF
        DB 0x14, 0x0A, 0x00, 0xEB, 0x20, 0x52, 0x9A, 0x55, 0x53, 0x52, 0x96, 0x31, 0x32, 0x32, 0x32, 0x34
        DB 0xA4, 0x30, 0x95, 0xFF, 0x07, 0x14, 0x00, 0xDD, 0x20, 0x52, 0xFF, 0x00
