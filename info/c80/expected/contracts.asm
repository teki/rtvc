FN7_INC:
FN7_INC_B1:
        INC A
        RET
        ; preserves B C (declared)
FN28_COUNT:
        LD B,A
FN28_COUNT_B2:
        LD C,0
        OR A
        JR Z,FN28_COUNT_B5
FN28_COUNT_B4:
        LD A,C
        CALL FN7_INC
        LD C,A
        DJNZ FN28_COUNT_B4
FN28_COUNT_B3:
FN28_COUNT_B5:
        LD A,C
        RET
FN35_BUMP:
FN35_BUMP_B6:
        INC A
        RET
        ; preserves B C D E H L (inferred)
FN42_TWICE_PUB:
FN42_TWICE_PUB_B7:
        LD C,A
        ADD A,C
        RET
FN54_AFTER_PRIVATE:
FN54_AFTER_PRIVATE_B8:
        CALL FN35_BUMP
        ADD A,C
        RET
FN66_AFTER_PUB:
FN66_AFTER_PUB_B9:
        PUSH BC ; spill
        CALL FN42_TWICE_PUB
        POP BC ; spill
        ADD A,C
        RET
        ; spill report: 0 slot bytes, saves (bytes per site) 2, 21 T-states of spill code
FN74_ADD:
FN74_ADD_B10:
        ADD HL,DE
        RET
        ; preserves D E (declared)
FN83_WRAP:
FN83_WRAP_B11:
        PUSH BC ; spill
        CALL FN42_TWICE_PUB
        POP BC ; spill
        INC A
        RET
        ; spill report: 0 slot bytes, saves (bytes per site) 2, 21 T-states of spill code
        ; preserves B C (declared)
G84_TOTAL:
        DW 0
FN94_ACC:
FN94_ACC_B12:
        ADD HL,DE
        LD (G84_TOTAL),HL
        RET
        ; preserves B C D E (declared)
