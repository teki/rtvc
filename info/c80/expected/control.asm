G1_G:
        DB 0
FN25_COUNT:
        LD C,A
FN25_COUNT_B1:
        LD B,0
        JR FN25_COUNT_B2
FN25_COUNT_B3:
        INC B
        DEC C
FN25_COUNT_B2:
        LD A,C
        OR A
        JP NZ,FN25_COUNT_B3
FN25_COUNT_B4:
        LD A,B
        RET
FN44_CHOOSE:
        LD C,A
FN44_CHOOSE_B5:
        OR A
        JR NZ,FN44_CHOOSE_B8
FN44_CHOOSE_B6:
        LD A,1
        RET
FN44_CHOOSE_B7:
FN44_CHOOSE_B8:
        LD A,C
        CP 1
        JR NZ,FN44_CHOOSE_B11
FN44_CHOOSE_B9:
        LD A,2
        RET
FN44_CHOOSE_B10:
FN44_CHOOSE_B11:
        LD A,3
        RET
FN60_NEST:
        LD B,C
FN60_NEST_B12:
        OR A
        JR Z,FN60_NEST_B15
FN60_NEST_B13:
        LD A,B
        OR A
        JR Z,FN60_NEST_B18
FN60_NEST_B16:
        LD A,1
        RET
FN60_NEST_B17:
FN60_NEST_B18:
        LD A,2
        RET
FN60_NEST_B14:
FN60_NEST_B15:
        LD A,3
        RET
FN96_SKIPPED:
        LD C,A
FN96_SKIPPED_B19:
        LD B,0
        JR FN96_SKIPPED_B20
FN96_SKIPPED_B21:
        DEC C
        LD A,C
        CP 2
        JR NZ,FN96_SKIPPED_B25
FN96_SKIPPED_B23:
        JP FN96_SKIPPED_B20
FN96_SKIPPED_B24:
FN96_SKIPPED_B25:
        LD A,C
        OR A
        JR NZ,FN96_SKIPPED_B28
FN96_SKIPPED_B26:
        JP FN96_SKIPPED_B22
FN96_SKIPPED_B27:
FN96_SKIPPED_B28:
        INC B
FN96_SKIPPED_B20:
        LD A,C
        OR A
        JP NZ,FN96_SKIPPED_B21
FN96_SKIPPED_B22:
        LD A,B
        RET
FN116_BOTH:
        LD B,C
        LD C,A
FN116_BOTH_B29:
        OR A
        JR Z,FN116_BOTH_B32
FN116_BOTH_B33:
        LD A,B
        OR A
        JR Z,FN116_BOTH_B32
FN116_BOTH_B30:
        LD A,1
        RET
FN116_BOTH_B31:
FN116_BOTH_B32:
        LD A,C
        OR A
        JR NZ,FN116_BOTH_B34
FN116_BOTH_B37:
        LD A,B
        OR A
        JR Z,FN116_BOTH_B36
FN116_BOTH_B34:
        LD A,2
        RET
FN116_BOTH_B35:
FN116_BOTH_B36:
        XOR A
        RET
FN124_BUMP:
FN124_BUMP_B38:
        LD A,(G1_G)
        INC A
        LD (G1_G),A
        RET
