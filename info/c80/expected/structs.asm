G11_ONE:
        DB 10,20,52,18,1
G13_ENEMIES:
        DB 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0
FN37_STEP:
        LD C,A
FN37_STEP_B1:
        JR FN37_STEP_B2
FN37_STEP_B3:
        LD A,(HL)
        INC A
        LD (HL),A
        LD A,L
        ADD A,5
        LD L,A
        LD A,H
        ADC A,0
        LD H,A
        DEC C
FN37_STEP_B2:
        LD A,C
        OR A
        JP NZ,FN37_STEP_B3
FN37_STEP_B4:
        RET
FN53_NUDGE:
FN53_NUDGE_B5:
        LD HL,G13_ENEMIES
        LD D,H
        LD E,L
        LD L,A
        LD H,0
        PUSH AF ; spill
        LD A,C ; spill
        LD (FN53_NUDGE_S),A ; spill
        POP AF ; spill
        LD B,H
        LD C,L
        ADD HL,HL
        ADD HL,HL
        ADD HL,BC
        ADD HL,DE
        LD C,A
        LD A,(HL)
        PUSH AF ; spill
        LD A,(FN53_NUDGE_S) ; spill
        LD B,A ; spill
        POP AF ; spill
        ADD A,B
        LD (HL),A
        LD L,C
        LD H,0
        LD B,H
        ADD HL,HL
        ADD HL,HL
        ADD HL,BC
        ADD HL,DE
        LD A,(HL)
        RET
        ; spill report: 1 slot bytes, saves (bytes per site) none, 76 T-states of spill code
FN53_NUDGE_S:
        DB 0 ; static slots
FN72_WALK:
        LD C,A
FN72_WALK_B6:
        LD B,0
        LD E,0
        JR FN72_WALK_B7
FN72_WALK_B8:
        INC B
FN72_WALK_B9:
        INC E
FN72_WALK_B7:
        LD A,E
        CP C
        JP C,FN72_WALK_B8
FN72_WALK_B10:
        LD A,B
        RET
FN86_ONCE:
FN86_ONCE_B11:
        LD C,0
FN86_ONCE_B12:
        INC C
FN86_ONCE_B13:
        LD A,C
        CP 1
        JP C,FN86_ONCE_B12
FN86_ONCE_B14:
        RET
FN91_FIRST_X:
FN91_FIRST_X_B15:
        LD HL,G11_ONE
        LD A,(HL)
        RET
