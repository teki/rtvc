FN7_COL_CALL:
FN7_COL_CALL_B1:
        RRCA
        RRCA
        AND 63
        RET
        ; preserves B C D E H L (inferred)
FN19_COLS_CALL:
FN19_COLS_CALL_B2:
        CALL FN7_COL_CALL
        LD B,A ; spill
        LD A,C
        CALL FN7_COL_CALL
        LD C,A
        LD A,B
        ADD A,C
        RET
        ; spill report: 0 slot bytes, saves (bytes per site) none, 4 T-states of spill code
FN26_COL:
FN26_COL_B3:
        RRCA
        RRCA
        AND 63
        RET
FN38_COLS:
FN38_COLS_B4:
        ; inline col
        RRCA
        RRCA
        AND 63
        ; inline col
        SRL C
        SRL C
        ADD A,C
        RET
FN49_COL_PLUS_TEN:
FN49_COL_PLUS_TEN_B5:
        ; inline col
        RRCA
        RRCA
        AND 63
        ; inline col
        ADD A,10
        RET
FN77_COUNT:
        LD B,A
FN77_COUNT_B7:
        LD C,0
        OR A
        JR Z,FN77_COUNT_B10
FN77_COUNT_B9:
        ; inline inc
        INC C
        DJNZ FN77_COUNT_B9
FN77_COUNT_B8:
FN77_COUNT_B10:
        LD A,C
        RET
