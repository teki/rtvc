FN11_ADD3:
FN11_ADD3_B1:
        ADD HL,DE
        ADD HL,BC
        RET
FN33_TIMES:
        LD B,C
        LD C,A
FN33_TIMES_B2:
        LD E,0
        LD A,B
        OR A
        JR Z,FN33_TIMES_B5
FN33_TIMES_B4:
        LD A,E
        ADD A,C
        LD E,A
        DJNZ FN33_TIMES_B4
FN33_TIMES_B3:
FN33_TIMES_B5:
        LD A,E
        RET
FN55_SCALE:
        LD D,H
        LD E,L
        LD B,A
FN55_SCALE_B6:
        LD HL,0
        OR A
        JR Z,FN55_SCALE_B9
FN55_SCALE_B8:
        ADD HL,DE
        DJNZ FN55_SCALE_B8
FN55_SCALE_B7:
FN55_SCALE_B9:
        RET
FN62_TWICE:
FN62_TWICE_B10:
        LD C,A
        ADD A,C
        RET
        ; preserves B D E H L (inferred)
FN74_KEEP:
        LD B,A
FN74_KEEP_B11:
        PUSH BC ; spill
        CALL FN62_TWICE
        POP BC ; spill
        ADD A,C
        RET
        ; spill report: 0 slot bytes, saves (bytes per site) 2, 21 T-states of spill code
FN83_NEXT:
FN83_NEXT_B12:
        LD A,(G76_NEXT_COUNT)
        INC A
        LD (G76_NEXT_COUNT),A
        LD A,(G76_NEXT_COUNT)
        RET
        ; preserves B C D E H L (inferred)
G76_NEXT_COUNT:
        DB 0
FN94_THIRD:
FN94_THIRD_B13:
        CALL FN83_NEXT
        CALL FN83_NEXT
        JP FN83_NEXT
FN108_SQUARE:
FN108_SQUARE_B14:
        LD HL,G102_SQUARE_TABLE
        LD D,H
        LD E,L
        LD L,A
        LD H,0
        ADD HL,DE
        LD A,(HL)
        RET
G102_SQUARE_TABLE:
        DB 0,1,4,9
FN140_SUM:
        LD B,H
        LD C,L
FN140_SUM_B15:
        LD (G111_SUM_LEFT),A
        LD DE,0
        JR FN140_SUM_B16
FN140_SUM_B17:
        LD A,(BC)
        LD L,A
        LD H,0
        ADD HL,DE
        LD D,H
        LD E,L
        INC BC
        LD A,(G111_SUM_LEFT)
        DEC A
        LD (G111_SUM_LEFT),A
FN140_SUM_B16:
        LD A,(G111_SUM_LEFT)
        OR A
        JP NZ,FN140_SUM_B17
FN140_SUM_B18:
        LD H,D
        LD L,E
        RET
G111_SUM_LEFT:
        DB 0
G147_DATA:
        DB 128,1,128,128
