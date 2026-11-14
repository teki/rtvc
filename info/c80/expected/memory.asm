G2_BUF:
        DB 0,0,0,0,0,0,0,0
G9_WORDS:
        DB 100,0,200,0,44,1,144,1
G11_HELLO:
        DB 2,104,105
FN38_FILL:
        LD B,C
        LD C,A
FN38_FILL_B1:
        JR FN38_FILL_B2
FN38_FILL_B3:
        LD A,B
        LD (HL),A
        INC HL
        DEC C
FN38_FILL_B2:
        LD A,C
        OR A
        JP NZ,FN38_FILL_B3
FN38_FILL_B4:
        RET
FN45_GET:
FN45_GET_B5:
        LD HL,G2_BUF
        LD D,H
        LD E,L
        LD L,A
        LD H,0
        ADD HL,DE
        LD A,(HL)
        RET
FN58_PAIR_SUM:
FN58_PAIR_SUM_B6:
        LD HL,G9_WORDS
        LD D,H
        LD E,L
        LD L,A
        LD H,0
        ADD HL,HL
        ADD HL,DE
        LD C,A
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        LD A,C
        INC A
        LD D,H
        LD E,L
        LD HL,G9_WORDS
        LD B,H
        LD C,L
        LD L,A
        LD H,0
        ADD HL,HL
        ADD HL,BC
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        ADD HL,DE
        RET
FN65_START:
FN65_START_B7:
        LD HL,G2_BUF
        RET
FN70_HELLO_LEN:
FN70_HELLO_LEN_B8:
        LD HL,G11_HELLO
        LD A,(HL)
        RET
FN77_HELLO_AT:
FN77_HELLO_AT_B9:
        LD HL,G11_HELLO
        INC HL
        LD D,H
        LD E,L
        LD L,A
        LD H,0
        ADD HL,DE
        LD A,(HL)
        RET
FN83_LIT_LEN:
FN83_LIT_LEN_B10:
        LD A,(HL)
        RET
        ; preserves B C D E H L (inferred)
FN89_PASS_LIT:
FN89_PASS_LIT_B11:
        LD HL,G2147483649_S1
        JP FN83_LIT_LEN
G2147483649_S1:
        DB 2,120,121
FN95_PTR_WIDTH:
FN95_PTR_WIDTH_B12:
        LD HL,4
        RET
G100_MID:
        DW G2_BUF+4
G113_ENDS:
        DW G9_WORDS
        DW G9_WORDS+6
FN117_MID_PTR:
FN117_MID_PTR_B13:
        LD HL,(G100_MID)
        RET
FN125_LAST_WORD:
FN125_LAST_WORD_B14:
        LD HL,G113_ENDS
        INC HL
        INC HL
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        RET
