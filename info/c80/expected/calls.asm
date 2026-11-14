FN5_ID:
FN5_ID_B1:
        RET
        ; preserves B C D E H L (inferred)
FN13_ADD8:
FN13_ADD8_B2:
        ADD A,C
        RET
        ; preserves B C D E H L (inferred)
FN21_ADD16:
FN21_ADD16_B3:
        ADD HL,DE
        RET
        ; preserves A B C D E (inferred)
FN31_NEST:
FN31_NEST_B4:
        LD C,A ; spill
        LD A,1
        CALL FN5_ID
        LD B,C
        LD C,A
        LD A,B
        JP FN13_ADD8
        ; spill report: 0 slot bytes, saves (bytes per site) none, 4 T-states of spill code
FN40_SWAPPED:
FN40_SWAPPED_B5:
        EX DE,HL
        JP FN21_ADD16
FN47_TAGGED:
FN47_TAGGED_B6:
        INC A
        RET
        ; preserves B C D E H L (inferred)
FN54_USE_TAGGED:
FN54_USE_TAGGED_B7:
        JP FN47_TAGGED
