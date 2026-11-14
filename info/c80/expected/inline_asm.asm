FN9_INC8:
FN9_INC8_B1:
        inc a
        RET
FN29_ADD_CARRY:
        LD B,C
        LD C,A
FN29_ADD_CARRY_B2:
        add a, b
        PUSH AF
        LD (FN29_ADD_CARRY_S),A ; spill
        POP HL
        LD A,L
        AND 1
        LD (FN29_ADD_CARRY_S+1),A ; spill
        OR A
        JR Z,FN29_ADD_CARRY_C0
        LD A,1
FN29_ADD_CARRY_C0:
        LD DE,(FN29_ADD_CARRY_S) ; spill
        OR A
        JR Z,FN29_ADD_CARRY_B5
FN29_ADD_CARRY_B3:
        LD A,E
        INC A
        RET
FN29_ADD_CARRY_B4:
FN29_ADD_CARRY_B5:
        LD A,E
        RET
        ; spill report: 2 slot bytes, saves (bytes per site) none, 59 T-states of spill code
FN29_ADD_CARRY_S:
        DB 0,0 ; static slots
