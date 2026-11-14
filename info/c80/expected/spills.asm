FN7_TWICE:
FN7_TWICE_B1:
        LD C,A
        ADD A,C
        RET
        ; preserves B D E H L (inferred)
FN19_KEEP:
FN19_KEEP_B2:
        LD B,C ; spill
        CALL FN7_TWICE
        ADD A,B
        RET
        ; spill report: 0 slot bytes, saves (bytes per site) none, 4 T-states of spill code
FN58_MIX:
        LD (FN58_MIX_S),DE ; spill
        LD D,H
        LD E,L
        LD (FN58_MIX_S+2),A ; spill
FN58_MIX_B3:
        LD HL,0 ; spill
        LD (FN58_MIX_S+3),HL ; spill
        LD BC,1
        JR FN58_MIX_B4
FN58_MIX_B5:
        LD HL,(FN58_MIX_S+3) ; spill
        ADD HL,DE
        LD (FN58_MIX_S+3),HL ; spill
        LD H,B
        LD L,C
        LD BC,(FN58_MIX_S) ; spill
        ADD HL,BC
        LD B,H
        LD C,L
        LD H,D
        LD L,E
        LD A,H
        XOR B
        LD H,A
        LD A,L
        XOR C
        LD L,A
        LD D,H
        LD E,L
        LD A,(FN58_MIX_S+2) ; spill
        DEC A
        LD (FN58_MIX_S+2),A ; spill
FN58_MIX_B4:
        LD A,(FN58_MIX_S+2) ; spill
        OR A
        JP NZ,FN58_MIX_B5
FN58_MIX_B6:
        LD HL,(FN58_MIX_S+3) ; spill
        ADD HL,BC
        RET
        ; spill report: 5 slot bytes, saves (bytes per site) none, 166 T-states of spill code
FN58_MIX_S:
        DB 0,0,0,0,0 ; static slots
FN75_VIA_PTR:
        LD C,A
FN75_VIA_PTR_B7:
        XOR A ; spill
        LD (FN75_VIA_PTR_S),A ; spill
        LD HL,FN75_VIA_PTR_S
        LD A,C
        LD (HL),A
        LD A,(FN75_VIA_PTR_S) ; spill
        INC A
        RET
        ; spill report: 1 slot bytes, saves (bytes per site) none, 33 T-states of spill code
FN75_VIA_PTR_S:
        DB 0 ; static slots
