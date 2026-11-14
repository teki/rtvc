ORG 0x3000
G3_STATE:
        DB 0
G4_PSCORE:
        DB 0
G5_ASCORE:
        DB 0
G6_PY:
        DB 0
G7_AY:
        DB 0
G8_BX:
        DB 0
G9_BY:
        DB 0
G10_VX:
        DB 0
G11_VY:
        DB 0
G12_DRAW_COL:
        DB 0
G13_DRAW_VAL:
        DB 0
G14_DRAW_H:
        DB 0
FN83_FILL_ROWS:
        LD C,A
FN83_FILL_ROWS_B3:
        LD A,(G14_DRAW_H)
        LD B,A
        OR A
        JR Z,FN83_FILL_ROWS_B6
FN83_FILL_ROWS_B5:
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
        LD A,(G12_DRAW_COL)
        LD D,H
        LD E,L
        LD L,A
        LD H,0
        ADD HL,DE
        LD A,(G13_DRAW_VAL)
        LD (HL),A
        INC C
        DJNZ FN83_FILL_ROWS_B5
FN83_FILL_ROWS_B4:
FN83_FILL_ROWS_B6:
        RET
FN130_PLACE_BALL:
FN130_PLACE_BALL_B11:
        LD A,32
        LD (G8_BX),A
        LD A,118
        LD (G9_BY),A
        XOR A
        LD (G10_VX),A
        XOR A
        LD (G11_VY),A
        XOR A
        LD (G3_STATE),A
        RET
        ; preserves B C D E H L (inferred)
FN150_HITS:
        LD C,A
FN150_HITS_B12:
        LD A,(G9_BY)
        LD B,A
        LD A,C
        ADD A,24
        LD E,A
        LD A,B
        CP E
        JR NC,FN150_HITS_B15
FN150_HITS_B16:
        LD A,(G9_BY)
        ADD A,4
        LD B,A
        LD A,C
        CP B
        JR NC,FN150_HITS_B15
FN150_HITS_B13:
        LD A,1
        RET
FN150_HITS_B14:
FN150_HITS_B15:
        XOR A
        RET
        ; preserves D H L (inferred)
FN192_SCORED:
        LD C,A
FN192_SCORED_B17:
        OR A
        JR Z,FN192_SCORED_B19
FN192_SCORED_B18:
        LD A,(G4_PSCORE)
        INC A
        LD (G4_PSCORE),A
        LD A,(G4_PSCORE)
        CP 5
        JR NZ,FN192_SCORED_B20
FN192_SCORED_B21:
        LD A,2
        LD (G3_STATE),A
        RET
FN192_SCORED_B22:
FN192_SCORED_B23:
FN192_SCORED_B19:
        LD A,(G5_ASCORE)
        INC A
        LD (G5_ASCORE),A
        LD A,(G5_ASCORE)
        CP 5
        JR NZ,FN192_SCORED_B20
FN192_SCORED_B24:
        LD A,3
        LD (G3_STATE),A
        RET
FN192_SCORED_B25:
FN192_SCORED_B26:
FN192_SCORED_B20:
        JP FN130_PLACE_BALL
        ; preserves B D E H L (inferred)
FN293_MOVE_PLAYER:
FN293_MOVE_PLAYER_B27:
        ; inline read_row
        LD A,8
        OUT (3),A
        IN A,(88)
        LD C,A
        AND 2
        OR A
        JR NZ,FN293_MOVE_PLAYER_B30
FN293_MOVE_PLAYER_B28:
        LD A,(G6_PY)
        CP 2
        JR C,FN293_MOVE_PLAYER_B30
FN293_MOVE_PLAYER_B31:
        LD A,(G6_PY)
        SUB 2
        LD (G6_PY),A
        LD A,2
        LD (G12_DRAW_COL),A
        LD (G14_DRAW_H),A
        LD A,255
        LD (G13_DRAW_VAL),A
        LD A,(G6_PY)
        PUSH BC ; spill
        CALL FN83_FILL_ROWS
        POP BC ; spill
        XOR A
        LD (G13_DRAW_VAL),A
        LD A,(G6_PY)
        ADD A,24
        PUSH BC ; spill
        CALL FN83_FILL_ROWS
        POP BC ; spill
FN293_MOVE_PLAYER_B32:
FN293_MOVE_PLAYER_B33:
FN293_MOVE_PLAYER_B29:
FN293_MOVE_PLAYER_B30:
        LD A,C
        AND 4
        OR A
        JR NZ,FN293_MOVE_PLAYER_B36
FN293_MOVE_PLAYER_B34:
        LD A,(G6_PY)
        CP 214
        JR NC,FN293_MOVE_PLAYER_B36
FN293_MOVE_PLAYER_B37:
        LD A,(G6_PY)
        ADD A,2
        LD (G6_PY),A
        LD A,2
        LD (G12_DRAW_COL),A
        LD (G14_DRAW_H),A
        XOR A
        LD (G13_DRAW_VAL),A
        LD A,(G6_PY)
        SUB 2
        CALL FN83_FILL_ROWS
        LD A,255
        LD (G13_DRAW_VAL),A
        LD A,(G6_PY)
        ADD A,22
        CALL FN83_FILL_ROWS
FN293_MOVE_PLAYER_B38:
FN293_MOVE_PLAYER_B39:
FN293_MOVE_PLAYER_B35:
FN293_MOVE_PLAYER_B36:
        RET
        ; spill report: 0 slot bytes, saves (bytes per site) 2 2, 42 T-states of spill code
FN537_BALL_SYNC:
        LD B,C
        LD C,A
FN537_BALL_SYNC_B61:
        LD A,(G8_BX)
        CP C
        JR NZ,FN537_BALL_SYNC_B64
FN537_BALL_SYNC_B62:
        LD A,(G9_BY)
        CP B
        JR NZ,FN537_BALL_SYNC_B67
FN537_BALL_SYNC_B65:
        RET
FN537_BALL_SYNC_B66:
FN537_BALL_SYNC_B67:
        LD A,(G8_BX)
        LD (G12_DRAW_COL),A
        LD A,(G9_BY)
        CP B
        JR NC,FN537_BALL_SYNC_B69
FN537_BALL_SYNC_B68:
        LD A,(G9_BY)
        LD C,A
        LD A,B
        SUB C
        LD (G14_DRAW_H),A
        LD A,255
        LD (G13_DRAW_VAL),A
        LD A,(G9_BY)
        CALL FN83_FILL_ROWS
        XOR A
        LD (G13_DRAW_VAL),A
        LD A,(G9_BY)
        ADD A,4
        CALL FN83_FILL_ROWS
        JP FN537_BALL_SYNC_B70
FN537_BALL_SYNC_B69:
        LD A,(G9_BY)
        SUB B
        LD (G14_DRAW_H),A
        XOR A
        LD (G13_DRAW_VAL),A
        PUSH BC ; spill
        LD A,B
        CALL FN83_FILL_ROWS
        POP BC ; spill
        LD A,255
        LD (G13_DRAW_VAL),A
        LD A,B
        ADD A,4
        CALL FN83_FILL_ROWS
FN537_BALL_SYNC_B70:
        RET
FN537_BALL_SYNC_B63:
FN537_BALL_SYNC_B64:
        LD A,4
        LD (G14_DRAW_H),A
        LD A,C
        LD (G12_DRAW_COL),A
        XOR A
        LD (G13_DRAW_VAL),A
        LD A,B
        CALL FN83_FILL_ROWS
        LD A,(G8_BX)
        LD (G12_DRAW_COL),A
        LD A,255
        LD (G13_DRAW_VAL),A
        LD A,(G9_BY)
        JP FN83_FILL_ROWS
        ; spill report: 0 slot bytes, saves (bytes per site) 2, 21 T-states of spill code
FN905_PLAY:
FN905_PLAY_B103:
        ; inline setup (auto)
        DI
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
        LD (G4_PSCORE),A
        XOR A
        LD (G5_ASCORE),A
        LD A,108
        LD (G6_PY),A
        LD (G7_AY),A
        CALL FN130_PLACE_BALL
        LD A,255
        LD (G13_DRAW_VAL),A
        LD A,24
        LD (G14_DRAW_H),A
        LD A,2
        LD (G12_DRAW_COL),A
        LD A,(G6_PY)
        CALL FN83_FILL_ROWS
        LD A,61
        LD (G12_DRAW_COL),A
        LD A,(G7_AY)
        CALL FN83_FILL_ROWS
        LD A,4
        LD (G14_DRAW_H),A
        LD A,(G8_BX)
        LD (G12_DRAW_COL),A
        LD A,(G9_BY)
        CALL FN83_FILL_ROWS
        JP FN905_PLAY_B104
FN905_PLAY_B105:
        ; inline wait_frame (auto)
        XOR A
        OUT (7),A
FN905_PLAY_B112:
FN905_PLAY_B111:
        IN A,(89)
        AND 16
        OR A
        JP NZ,FN905_PLAY_B112
FN905_PLAY_B113:
        XOR A
        OUT (7),A
        CALL FN293_MOVE_PLAYER
        ; inline serve (auto)
        ; inline read_row
        LD A,7
        OUT (3),A
        IN A,(88)
        LD C,A
        LD A,(G10_VX)
        OR A
        JR NZ,FN905_PLAY_B118
FN905_PLAY_B115:
        LD A,C
        AND 32
        OR A
        JR NZ,FN905_PLAY_B118
FN905_PLAY_B116:
        LD A,1
        LD (G10_VX),A
        LD (G11_VY),A
        LD (G3_STATE),A
FN905_PLAY_B117:
FN905_PLAY_B118:
        ; inline move_ai (auto)
        LD A,(G10_VX)
        OR A
        JR NZ,FN905_PLAY_B121
FN905_PLAY_B120:
        JP FN905_PLAY_B135
FN905_PLAY_B121:
        LD A,(G9_BY)
        ADD A,2
        LD B,A
        LD A,(G7_AY)
        ADD A,12
        LD E,A
        LD A,B
        CP E
        JR NC,FN905_PLAY_B127
FN905_PLAY_B123:
        LD A,(G7_AY)
        OR A
        JR Z,FN905_PLAY_B135
FN905_PLAY_B124:
        LD A,(G7_AY)
        DEC A
        LD (G7_AY),A
        LD A,61
        LD (G12_DRAW_COL),A
        LD A,1
        LD (G14_DRAW_H),A
        LD A,255
        LD (G13_DRAW_VAL),A
        LD A,(G7_AY)
        CALL FN83_FILL_ROWS
        XOR A
        LD (G13_DRAW_VAL),A
        LD A,(G7_AY)
        ADD A,24
        CALL FN83_FILL_ROWS
        JP FN905_PLAY_B135
FN905_PLAY_B125:
FN905_PLAY_B126:
FN905_PLAY_B127:
        LD A,B
        CP E
        JR Z,FN905_PLAY_B135
        JR C,FN905_PLAY_B135
FN905_PLAY_B128:
        LD A,(G7_AY)
        CP 216
        JR NC,FN905_PLAY_B135
FN905_PLAY_B129:
        LD A,(G7_AY)
        INC A
        LD (G7_AY),A
        LD A,61
        LD (G12_DRAW_COL),A
        LD A,1
        LD (G14_DRAW_H),A
        XOR A
        LD (G13_DRAW_VAL),A
        LD A,(G7_AY)
        DEC A
        CALL FN83_FILL_ROWS
        LD A,255
        LD (G13_DRAW_VAL),A
        LD A,(G7_AY)
        ADD A,23
        CALL FN83_FILL_ROWS
FN905_PLAY_B130:
FN905_PLAY_B131:
FN905_PLAY_B132:
FN905_PLAY_B133:
FN905_PLAY_B134:
FN905_PLAY_B135:
        ; inline physics (auto)
        LD A,(G10_VX)
        OR A
        JR NZ,FN905_PLAY_B138
FN905_PLAY_B137:
        JP FN905_PLAY_B104
FN905_PLAY_B138:
        LD A,(G8_BX)
        LD D,A
        LD A,(G9_BY)
        LD L,A
        LD A,(G11_VY)
        XOR 128
        CP 128
        JR NC,FN905_PLAY_B144
FN905_PLAY_B140:
        LD A,(G9_BY)
        OR A
        JR NZ,FN905_PLAY_B142
FN905_PLAY_B141:
        LD A,1
        LD (G11_VY),A
        JP FN905_PLAY_B148
FN905_PLAY_B142:
        LD A,(G9_BY)
        DEC A
        LD (G9_BY),A
        JP FN905_PLAY_B148
FN905_PLAY_B143:
FN905_PLAY_B144:
        LD A,(G9_BY)
        CP 236
        JR C,FN905_PLAY_B146
FN905_PLAY_B145:
        LD A,255
        LD (G11_VY),A
        JP FN905_PLAY_B148
FN905_PLAY_B146:
        LD A,(G9_BY)
        INC A
        LD (G9_BY),A
FN905_PLAY_B147:
FN905_PLAY_B148:
        LD A,(G10_VX)
        XOR 128
        CP 128
        JR NC,FN905_PLAY_B157
FN905_PLAY_B149:
        LD A,(G8_BX)
        CP 3
        JR C,FN905_PLAY_B150
        JR NZ,FN905_PLAY_B152
FN905_PLAY_B150:
        LD A,(G6_PY)
        CALL FN150_HITS
        OR A
        JR Z,FN905_PLAY_B152
FN905_PLAY_B151:
        LD A,1
        LD (G10_VX),A
        JP FN905_PLAY_B165
FN905_PLAY_B152:
        LD A,(G8_BX)
        OR A
        JR NZ,FN905_PLAY_B154
FN905_PLAY_B153:
        XOR A
        CALL FN192_SCORED
        JP FN905_PLAY_B165
FN905_PLAY_B154:
        LD A,(G8_BX)
        DEC A
        LD (G8_BX),A
        JP FN905_PLAY_B165
FN905_PLAY_B155:
FN905_PLAY_B156:
FN905_PLAY_B157:
        LD A,(G8_BX)
        CP 60
        JR C,FN905_PLAY_B160
FN905_PLAY_B158:
        LD A,(G7_AY)
        CALL FN150_HITS
        OR A
        JR Z,FN905_PLAY_B160
FN905_PLAY_B159:
        LD A,255
        LD (G10_VX),A
        JP FN905_PLAY_B165
FN905_PLAY_B160:
        LD A,(G8_BX)
        CP 63
        JR C,FN905_PLAY_B162
FN905_PLAY_B161:
        LD A,1
        CALL FN192_SCORED
        JP FN905_PLAY_B165
FN905_PLAY_B162:
        LD A,(G8_BX)
        INC A
        LD (G8_BX),A
FN905_PLAY_B163:
FN905_PLAY_B164:
FN905_PLAY_B165:
        LD A,D
        LD C,L
        CALL FN537_BALL_SYNC
FN905_PLAY_B104:
        LD A,(G3_STATE)
        CP 2
        JP C,FN905_PLAY_B105
FN905_PLAY_B166:
FN905_PLAY_B106:
        ; inline restore (auto)
        LD A,112
        OUT (2),A
        EI
        LD A,(G3_STATE)
        CP 2
        JR NZ,FN905_PLAY_B109
FN905_PLAY_B107:
        LD HL,1
        RET
FN905_PLAY_B108:
FN905_PLAY_B109:
        LD HL,65535
        RET
ORG 0x19EF
        DB 0x14, 0x0A, 0x00, 0xEB, 0x20, 0x52, 0x9A, 0x55, 0x53, 0x52, 0x96, 0x31, 0x32, 0x36, 0x38, 0x35
        DB 0xA4, 0x30, 0x95, 0xFF, 0x07, 0x14, 0x00, 0xDD, 0x20, 0x52, 0xFF, 0x00
