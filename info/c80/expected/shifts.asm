FN7_DBL:
FN7_DBL_B1:
        ADD A,A
        RET
FN14_TIMES64:
FN14_TIMES64_B2:
        RRCA
        RRCA
        AND 192
        RET
FN21_HALF_U:
FN21_HALF_U_B3:
        SRL A
        RET
FN28_HALF_I:
FN28_HALF_I_B4:
        SRA A
        RET
FN35_SIGN8:
FN35_SIGN8_B5:
        ADD A,A
        SBC A,A
        RET
FN42_SIGN8_FAR:
FN42_SIGN8_FAR_B6:
        ADD A,A
        SBC A,A
        RET
FN49_GONE:
FN49_GONE_B7:
        XOR A
        RET
FN56_W3:
FN56_W3_B8:
        ADD HL,HL
        ADD HL,HL
        ADD HL,HL
        RET
FN63_W8:
FN63_W8_B9:
        LD H,L
        LD L,0
        RET
FN70_W12:
FN70_W12_B10:
        LD A,H
        RRCA
        RRCA
        RRCA
        RRCA
        AND 15
        LD L,A
        LD H,0
        RET
FN77_W7:
FN77_W7_B11:
        XOR A
        ADD HL,HL
        RLA
        LD L,H
        LD H,A
        RET
FN84_SIGN16:
FN84_SIGN16_B12:
        LD A,H
        RLA
        SBC A,A
        LD H,A
        LD L,A
        RET
FN91_SIGN16_FAR:
FN91_SIGN16_FAR_B13:
        LD A,H
        RLA
        SBC A,A
        LD H,A
        LD L,A
        RET
FN99_SHL16:
FN99_SHL16_B14:
        LD B,A
        INC B
        DEC B
        JR Z,FN99_SHL16_C1
FN99_SHL16_C0:
        ADD HL,HL
        DJNZ FN99_SHL16_C0
FN99_SHL16_C1:
        RET
FN107_SAR16:
FN107_SAR16_B15:
        LD B,A
        INC B
        DEC B
        JR Z,FN107_SAR16_C1
FN107_SAR16_C0:
        SRA H
        RR L
        DJNZ FN107_SAR16_C0
FN107_SAR16_C1:
        RET
FN115_SHR8:
FN115_SHR8_B16:
        LD B,C
        INC B
        DEC B
        JR Z,FN115_SHR8_C1
FN115_SHR8_C0:
        SRL A
        DJNZ FN115_SHR8_C0
FN115_SHR8_C1:
        RET
FN123_SAR8:
FN123_SAR8_B17:
        LD B,C
        INC B
        DEC B
        JR Z,FN123_SAR8_C1
FN123_SAR8_C0:
        SRA A
        DJNZ FN123_SAR8_C0
FN123_SAR8_C1:
        RET
FN135_PIN_SHL:
        LD B,C
        LD C,A
FN135_PIN_SHL_B18:
        SLA B
        LD A,B
        ADD A,C
        RET
FN148_PIN_SHR:
        LD E,C
        LD C,A
FN148_PIN_SHR_B19:
        INC E
        DEC E
        JR Z,FN148_PIN_SHR_C1
FN148_PIN_SHR_C0:
        SRL B
        DEC E
        JR NZ,FN148_PIN_SHR_C0
FN148_PIN_SHR_C1:
        LD A,B
        ADD A,C
        RET
G152_CELL:
        DB 0,0,0
G153_FLAGS:
        DB 0
FN169_ROTATE_CELL:
FN169_ROTATE_CELL_B20:
        LD HL,G152_CELL
        INC HL
        LD D,H
        LD E,L
        LD C,A
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        LD B,C
        INC B
        DEC B
        JR Z,FN169_ROTATE_CELL_C1
FN169_ROTATE_CELL_C0:
        ADD HL,HL
        DJNZ FN169_ROTATE_CELL_C0
FN169_ROTATE_CELL_C1:
        LD A,C
        LD B,D
        LD C,E
        LD D,H
        LD E,L
        LD H,B
        LD L,C
        LD (HL),E
        INC HL
        LD (HL),D
        LD C,A
        LD A,(G153_FLAGS)
        LD B,C
        INC B
        DEC B
        JR Z,FN169_ROTATE_CELL_C3
FN169_ROTATE_CELL_C2:
        ADD A,A
        DJNZ FN169_ROTATE_CELL_C2
FN169_ROTATE_CELL_C3:
        LD (G153_FLAGS),A
        RET
