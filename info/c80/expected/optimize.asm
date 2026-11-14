G1_G:
        DB 0
FN13_SHORT_IF:
FN13_SHORT_IF_B1:
        OR A
        JR NZ,FN13_SHORT_IF_B4
FN13_SHORT_IF_B2:
        LD A,1
        RET
FN13_SHORT_IF_B3:
FN13_SHORT_IF_B4:
        LD A,2
        RET
FN37_COUNT:
        LD C,A
FN37_COUNT_B5:
        LD B,0
        JR FN37_COUNT_B6
FN37_COUNT_B7:
        INC B
        DEC C
FN37_COUNT_B6:
        LD A,C
        OR A
        JP NZ,FN37_COUNT_B7
FN37_COUNT_B8:
        LD A,B
        RET
FN55_BLINK:
        LD B,A
FN55_BLINK_B9:
        LD C,0
        OR A
        JR Z,FN55_BLINK_B12
FN55_BLINK_B11:
        INC C
        DJNZ FN55_BLINK_B11
FN55_BLINK_B10:
FN55_BLINK_B12:
        LD A,C
        RET
        ; preserves D E H L (inferred)
FN64_MASK:
FN64_MASK_B13:
        LD A,31
        RET
FN71_AGAIN:
FN71_AGAIN_B14:
        JP FN55_BLINK
FN85_WRITES:
FN85_WRITES_B15:
        LD A,1
        LD (G1_G),A
        LD A,2
        LD (G1_G),A
        LD A,3
        LD (G1_G),A
        RET
