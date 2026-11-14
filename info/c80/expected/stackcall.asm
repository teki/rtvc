FN8_ADD:
        PUSH IX
        LD IX,0
        ADD IX,SP
FN8_ADD_B1:
        LD L,(IX+4)
        LD H,(IX+5)
        LD E,(IX+6)
        LD D,(IX+7)
        ADD HL,DE
        LD SP,IX
        POP IX
        RET
FN17_MIX:
        PUSH IX
        LD IX,0
        ADD IX,SP
FN17_MIX_B2:
        LD A,(IX+4)
        LD C,(IX+8)
        ADD A,C
        LD SP,IX
        POP IX
        RET
        ; preserves B D E H L (inferred)
FN24_USE_ADD:
FN24_USE_ADD_B3:
        LD HL,3
        PUSH HL
        LD HL,2
        PUSH HL
        CALL FN8_ADD
        INC SP
        INC SP
        INC SP
        INC SP
        RET
FN32_USE_MIX:
FN32_USE_MIX_B4:
        LD A,4
        LD L,A
        LD H,0
        PUSH HL
        LD HL,4369
        PUSH HL
        LD A,1
        LD L,A
        LD H,0
        PUSH HL
        CALL FN17_MIX
        INC SP
        INC SP
        INC SP
        INC SP
        INC SP
        INC SP
        RET
