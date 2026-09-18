; Assembly entry: CALL a public C80 export, then continue.
; stack_extra on this unit covers the CALL (return address + callee use).

start:
        LD A, 7
        CALL @{math::inc}
        LD HL, @{project::stack_top}
        RET
