G8_FIRST:
        DB 1
        DW G13_SECOND
G13_SECOND:
        DB 2
        DW G18_THIRD
G18_THIRD:
        DB 4,0,0
FN45_LIST_SUM:
FN45_LIST_SUM_B1:
        LD HL,G8_FIRST
        LD C,0
        JR FN45_LIST_SUM_B2
FN45_LIST_SUM_B3:
        LD A,(HL)
        LD B,A
        LD A,C
        ADD A,B
        LD C,A
        INC HL
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
FN45_LIST_SUM_B2:
        LD D,H
        LD E,L
        LD A,C ; spill
        LD (FN45_LIST_SUM_S),A ; spill
        LD BC,0
        OR A
        SBC HL,BC
        LD H,D
        LD L,E
        LD C,A ; spill
        JP NZ,FN45_LIST_SUM_B3
FN45_LIST_SUM_B4:
        RET
        ; spill report: 1 slot bytes, saves (bytes per site) none, 34 T-states of spill code
FN45_LIST_SUM_S:
        DB 0 ; static slots
G60_FRAMES:
        DB 24,60,126,255,129,66,36,24
G84_SPRITES:
        DB 10,20
        DW G60_FRAMES
        DB 30,40
        DW G60_FRAMES+4
        DB 50,60
        DW G60_FRAMES+4
FN93_FIRST_ROW:
FN93_FIRST_ROW_B5:
        LD HL,G84_SPRITES
        LD D,H
        LD E,L
        LD L,A
        LD H,0
        ADD HL,HL
        ADD HL,HL
        ADD HL,DE
        INC HL
        INC HL
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        LD A,(HL)
        RET
FN101_FRAME_OF:
FN101_FRAME_OF_B6:
        LD HL,G84_SPRITES
        LD D,H
        LD E,L
        LD L,A
        LD H,0
        ADD HL,HL
        ADD HL,HL
        ADD HL,DE
        INC HL
        INC HL
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        RET
G119_ROWS:
        DW G60_FRAMES+1
        DW G60_FRAMES+3
        DW G60_FRAMES+5
FN128_ROW:
FN128_ROW_B7:
        LD HL,G119_ROWS
        LD D,H
        LD E,L
        LD L,A
        LD H,0
        ADD HL,HL
        ADD HL,DE
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        LD A,(HL)
        RET
G135_CFG:
        DB 3,232,3
G139_SCORE:
        DW G135_CFG+1
FN149_BUMP:
FN149_BUMP_B8:
        LD HL,(G139_SCORE)
        LD D,H
        LD E,L
        LD A,(HL)
        INC HL
        LD H,(HL)
        ADD A,10
        LD L,A
        LD A,H
        ADC A,0
        LD H,A
        LD B,D
        LD C,E
        LD D,H
        LD E,L
        LD H,B
        LD L,C
        LD (HL),E
        INC HL
        LD (HL),D
        LD HL,G135_CFG
        INC HL
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        RET
G154_RING:
        DB 9
        DW G154_RING
FN163_RING_TWICE:
FN163_RING_TWICE_B9:
        LD HL,G154_RING
        INC HL
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        INC HL
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        LD A,(HL)
        RET
FN180_PEEK:
FN180_PEEK_B10:
        LD HL,(G175_PEEK_LAST)
        LD A,(HL)
        RET
G170_PEEK_BUF:
        DB 11,22,33,44
G175_PEEK_LAST:
        DW G170_PEEK_BUF+3
