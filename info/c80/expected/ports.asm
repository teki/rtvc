FN8_READ_DEVICE:
FN8_READ_DEVICE_B1:
        LD C,A
        LD B,0
        IN A,(C)
        RET
FN18_WRITE_DEVICE:
FN18_WRITE_DEVICE_B2:
        LD E,C
        LD C,A
        LD B,0
        LD A,E
        OUT (C),A
        RET
FN37_WRITE_PAIR:
FN37_WRITE_PAIR_B3:
        LD E,B
        LD D,C
        LD C,A
        LD B,0
        LD L,A
        LD A,D
        OUT (C),A
        LD A,L
        INC A
        LD C,A
        LD A,E
        OUT (C),A
        RET
FN49_COPY_BLOCK:
FN49_COPY_BLOCK_B4:
        LDIR
        RET
FN59_INTS:
FN59_INTS_B5:
        DI
        EI
        RET
