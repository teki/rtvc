ORG 0x2100
FN13_INC:
FN13_INC_B1:
        INC A
        RET
FN18_CANVAS:
FN18_CANVAS_B2:
        LD HL,40
        RET
FN23_STACK_TOP:
FN23_STACK_TOP_B3:
        LD HL,0
        RET
ORG 0x2000
; Assembly entry: CALL a public C80 export, then continue.
; stack_extra on this unit covers the CALL (return address + callee use).

U2_START:
        LD A, 7
        CALL FN13_INC
        LD HL, 0
        RET
