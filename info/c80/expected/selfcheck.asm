ORG 0x3000
G6_SEEN:
        DB 0,0,0,0,0,0
G8_FAILED:
        DB 0,0,0,0,0,0
G9_CHECKS:
        DW 0
G10_PASSES:
        DB 0
G11_BASIC_DONE:
        DB 0
G12_FAILS:
        DB 0
G14_UA:
        DB 255
G16_UB:
        DB 1
G19_SA:
        DB 128
G21_SB:
        DB 1
G23_SC:
        DB 127
G25_WA:
        DW 65535
G27_WB:
        DW 1
G29_WC:
        DW 32768
G32_IA:
        DW 32768
G34_IB:
        DW 1
G36_IC:
        DW 32767
G38_K3:
        DB 3
G40_K4:
        DB 4
G42_K8:
        DB 8
G44_K9:
        DB 9
G46_K16:
        DB 16
G48_K20:
        DB 20
G50_N5:
        DB 5
G52_N37:
        DB 55
G54_N5A:
        DB 90
G56_I1:
        DB 1
G58_I2:
        DB 2
FN95_MARK:
        LD B,C
        LD C,A
FN95_MARK_B1:
        AND 7
        LD D,A
        LD A,1
        INC D
        DEC D
        JR Z,FN95_MARK_C1
FN95_MARK_C0:
        ADD A,A
        DEC D
        JR NZ,FN95_MARK_C0
FN95_MARK_C1:
        LD E,A
        LD A,C
        RRCA
        RRCA
        RRCA
        AND 31
        LD D,A
        LD HL,G6_SEEN
        LD A,B ; spill
        LD (FN95_MARK_S),A ; spill
        LD B,H
        LD C,L
        LD L,D
        LD H,0
        ADD HL,BC
        LD A,(HL)
        OR E
        LD (HL),A
        LD A,(FN95_MARK_S) ; spill
        OR A
        JR NZ,FN95_MARK_B4
FN95_MARK_B2:
        LD HL,G8_FAILED
        LD B,H
        LD C,L
        LD L,D
        LD H,0
        ADD HL,BC
        LD A,(HL)
        OR E
        LD (HL),A
        LD A,(G12_FAILS)
        INC A
        LD (G12_FAILS),A
FN95_MARK_B3:
FN95_MARK_B4:
        LD HL,(G9_CHECKS)
        INC HL
        LD (G9_CHECKS),HL
        RET
        ; spill report: 1 slot bytes, saves (bytes per site) none, 30 T-states of spill code
FN95_MARK_S:
        DB 0 ; static slots
FN338_COMPARES:
FN338_COMPARES_B7:
        LD A,(G14_UA)
        LD C,A
        LD A,(G16_UB)
        LD B,A
        LD A,C
        CP B
        LD A,0
        JR Z,FN338_COMPARES_C0
        JR C,FN338_COMPARES_C0
        LD A,1
FN338_COMPARES_C0:
        LD C,A
        LD A,16
        CALL FN95_MARK
        LD A,(G19_SA)
        LD C,A
        LD A,(G21_SB)
        XOR 128
        LD B,A
        LD A,C
        XOR 128
        CP B
        LD A,1
        JR C,FN338_COMPARES_C1
        XOR A
FN338_COMPARES_C1:
        LD C,A
        LD A,17
        CALL FN95_MARK
        LD A,(G23_SC)
        LD C,A
        LD A,(G19_SA)
        XOR 128
        LD B,A
        LD A,C
        XOR 128
        CP B
        LD A,0
        JR Z,FN338_COMPARES_C2
        JR C,FN338_COMPARES_C2
        LD A,1
FN338_COMPARES_C2:
        LD C,A
        LD A,18
        CALL FN95_MARK
        LD HL,(G29_WC)
        LD DE,32767
        OR A
        SBC HL,DE
        LD A,0
        JR Z,FN338_COMPARES_C3
        JR C,FN338_COMPARES_C3
        LD A,1
FN338_COMPARES_C3:
        LD C,A
        LD A,19
        CALL FN95_MARK
        LD HL,(G32_IA)
        LD D,H
        LD E,L
        LD HL,(G36_IC)
        LD B,H
        LD C,L
        LD H,D
        LD L,E
        LD A,H
        XOR 128
        LD H,A
        LD A,B
        XOR 128
        LD B,A
        OR A
        SBC HL,BC
        LD A,1
        JR C,FN338_COMPARES_C4
        XOR A
FN338_COMPARES_C4:
        LD C,A
        LD A,20
        CALL FN95_MARK
        LD C,21
        LD HL,(G32_IA)
        LD D,H
        LD E,L
        LD HL,(G34_IB)
        LD (FN338_COMPARES_S),HL ; spill
        LD H,D
        LD L,E
        LD A,H
        XOR 128
        LD H,A
        LD DE,(FN338_COMPARES_S) ; spill
        LD A,D
        XOR 128
        LD D,A
        OR A
        SBC HL,DE
        LD A,1
        JR C,FN338_COMPARES_C5
        XOR A
FN338_COMPARES_C5:
        OR A
        JR Z,FN338_COMPARES_B10
FN338_COMPARES_B8:
        LD HL,(G36_IC)
        LD D,H
        LD E,L
        LD HL,(G32_IA)
        LD (FN338_COMPARES_S),HL ; spill
        LD H,D
        LD L,E
        LD A,H
        XOR 128
        LD H,A
        LD DE,(FN338_COMPARES_S) ; spill
        LD A,D
        XOR 128
        LD D,A
        OR A
        SBC HL,DE
        LD A,1
        JR C,FN338_COMPARES_C6
        XOR A
FN338_COMPARES_C6:
        XOR 1
        JP FN338_COMPARES_B9
FN338_COMPARES_B10:
        XOR A
FN338_COMPARES_B9:
        LD B,C
        LD C,A
        LD A,B
        CALL FN95_MARK
        LD B,22
        LD A,(G16_UB)
        LD C,A
        LD A,(G14_UA)
        LD E,A
        LD A,C
        CP E
        LD A,1
        JR C,FN338_COMPARES_C7
        JR Z,FN338_COMPARES_C7
        XOR A
FN338_COMPARES_C7:
        OR A
        JR Z,FN338_COMPARES_B13
FN338_COMPARES_B11:
        LD A,(G14_UA)
        LD C,A
        LD A,(G14_UA)
        LD E,A
        LD A,C
        CP E
        LD A,1
        JR NC,FN338_COMPARES_B12
        XOR A
FN338_COMPARES_C8:
        JP FN338_COMPARES_B12
FN338_COMPARES_B13:
        XOR A
FN338_COMPARES_B12:
        OR A
        JR Z,FN338_COMPARES_B16
FN338_COMPARES_B14:
        LD A,(G16_UB)
        LD C,A
        LD A,(G14_UA)
        LD E,A
        LD A,C
        CP E
        LD A,1
        JR NC,FN338_COMPARES_C9
        XOR A
FN338_COMPARES_C9:
        XOR 1
        JP FN338_COMPARES_B15
FN338_COMPARES_B16:
        XOR A
FN338_COMPARES_B15:
        LD C,A
        LD A,B
        CALL FN95_MARK
        LD E,23
        LD HL,(G29_WC)
        LD A,H
        XOR 128
        LD H,A
        LD BC,32768
        OR A
        SBC HL,BC
        LD A,1
        JR C,FN338_COMPARES_C10
        XOR A
FN338_COMPARES_C10:
        OR A
        JR Z,FN338_COMPARES_B19
FN338_COMPARES_B17:
        LD A,(G14_UA)
        XOR 128
        CP 128
        LD A,1
        JR C,FN338_COMPARES_B18
        XOR A
FN338_COMPARES_C11:
        JP FN338_COMPARES_B18
FN338_COMPARES_B19:
        XOR A
FN338_COMPARES_B18:
        LD C,A
        LD A,E
        JP FN95_MARK
        ; spill report: 2 slot bytes, saves (bytes per site) none, 72 T-states of spill code
FN338_COMPARES_S:
        DB 0,0 ; static slots
FN525_NINE:
FN525_NINE_B42:
        LD B,A
        ADD A,C
        LD E,A
        XOR B
        LD D,A
        INC A
        LD L,A
        AND 63
        LD H,A
        OR C
        LD (FN525_NINE_S),A ; spill
        SUB E
        LD (FN525_NINE_S+1),A ; spill
        ADD A,D
        ADD A,L
        LD (FN525_NINE_S+2),A ; spill
        LD A,B
        ADD A,C
        ADD A,E
        ADD A,D
        ADD A,L
        ADD A,H
        LD BC,(FN525_NINE_S) ; spill
        ADD A,C
        LD BC,(FN525_NINE_S+1) ; spill
        ADD A,C
        LD BC,(FN525_NINE_S+2) ; spill
        ADD A,C
        RET
        ; spill report: 3 slot bytes, saves (bytes per site) none, 99 T-states of spill code
FN525_NINE_S:
        DB 0,0,0 ; static slots
FN532_TWICE_PUB:
FN532_TWICE_PUB_B43:
        LD C,A
        ADD A,C
        RET
FN544_KEEP:
FN544_KEEP_B44:
        PUSH BC ; spill
        CALL FN532_TWICE_PUB
        POP BC ; spill
        ADD A,C
        RET
        ; spill report: 0 slot bytes, saves (bytes per site) 2, 21 T-states of spill code
FN566_TIMES:
        LD B,C
        LD C,A
FN566_TIMES_B45:
        LD E,0
        LD A,B
        OR A
        JR Z,FN566_TIMES_B48
FN566_TIMES_B47:
        LD A,E
        ADD A,C
        LD E,A
        DJNZ FN566_TIMES_B47
FN566_TIMES_B46:
FN566_TIMES_B48:
        LD A,E
        RET
        ; preserves D H L (inferred)
FN573_INC:
FN573_INC_B49:
        INC A
        RET
        ; preserves B C (declared)
FN594_COUNT:
        LD B,A
FN594_COUNT_B50:
        LD C,0
        OR A
        JR Z,FN594_COUNT_B53
FN594_COUNT_B52:
        LD A,C
        CALL FN573_INC
        LD C,A
        DJNZ FN594_COUNT_B52
FN594_COUNT_B51:
FN594_COUNT_B53:
        LD A,C
        RET
FN603_NEXT:
FN603_NEXT_B54:
        LD A,(G596_NEXT_C)
        INC A
        LD (G596_NEXT_C),A
        LD A,(G596_NEXT_C)
        RET
        ; preserves B C D E H L (inferred)
G596_NEXT_C:
        DB 0
FN624_MIX:
        PUSH IX
        LD IX,0
        ADD IX,SP
FN624_MIX_B56:
        LD A,(IX+4)
        LD C,(IX+8)
        ADD A,C
        LD L,(IX+6)
        LD H,(IX+7)
        LD L,H
        ADD A,L
        LD SP,IX
        POP IX
        RET
        ; preserves B D E (inferred)
FN629_ID:
FN629_ID_B57:
        RET
        ; preserves B C D E H L (inferred)
FN637_ADD8:
FN637_ADD8_B58:
        ADD A,C
        RET
        ; preserves B C D E H L (inferred)
G656_SPRITES:
        DB 10,20,52,18,30,40,120,86,50,60,188,154
G664_FIRST:
        DB 1
        DW G669_SECOND
G669_SECOND:
        DB 2
        DW G674_THIRD
G674_THIRD:
        DB 4,0,0
FN701_LIST_SUM:
FN701_LIST_SUM_B59:
        LD HL,G664_FIRST
        LD C,0
        JR FN701_LIST_SUM_B60
FN701_LIST_SUM_B61:
        LD A,(HL)
        LD B,A
        LD A,C
        ADD A,B
        LD C,A
        INC HL
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
FN701_LIST_SUM_B60:
        LD D,H
        LD E,L
        LD A,C ; spill
        LD (FN701_LIST_SUM_S),A ; spill
        LD BC,0
        OR A
        SBC HL,BC
        LD H,D
        LD L,E
        LD C,A ; spill
        JP NZ,FN701_LIST_SUM_B61
FN701_LIST_SUM_B62:
        RET
        ; spill report: 1 slot bytes, saves (bytes per site) none, 34 T-states of spill code
FN701_LIST_SUM_S:
        DB 0 ; static slots
G703_NAME:
        DB 3,84,86,67
G710_SRC:
        DB 1,2,3,4
G712_DST:
        DB 0,0,0,0
FN817_CALLS:
FN817_CALLS_B63:
        LD A,(G52_N37)
        LD C,A
        LD A,(G54_N5A)
        LD B,C
        LD C,A
        LD A,B
        CALL FN525_NINE
        CP 62
        LD A,1
        JR Z,FN817_CALLS_C0
        XOR A
FN817_CALLS_C0:
        LD C,A
        LD A,32
        CALL FN95_MARK
        LD A,(G50_N5)
        LD C,A
        LD A,(G16_UB)
        LD B,C
        LD C,A
        LD A,B
        CALL FN544_KEEP
        CP 11
        LD A,1
        JR Z,FN817_CALLS_C1
        XOR A
FN817_CALLS_C1:
        LD C,A
        LD A,33
        CALL FN95_MARK
        LD A,7
        LD C,6
        CALL FN566_TIMES
        CP 42
        LD A,1
        JR Z,FN817_CALLS_C2
        XOR A
FN817_CALLS_C2:
        LD C,A
        LD A,34
        CALL FN95_MARK
        LD A,(G50_N5)
        CALL FN594_COUNT
        CP 5
        LD A,1
        JR Z,FN817_CALLS_C3
        XOR A
FN817_CALLS_C3:
        LD C,A
        LD A,35
        CALL FN95_MARK
        CALL FN603_NEXT
        LD C,A
        CALL FN603_NEXT
        LD B,A
        SUB C
        CP 1
        LD A,1
        JR Z,FN817_CALLS_C4
        XOR A
FN817_CALLS_C4:
        LD C,A
        LD A,36
        CALL FN95_MARK
        LD E,37
        LD A,(G14_UA)
        ; inline col
        RRCA
        RRCA
        AND 63
        CP 63
        LD A,1
        JR Z,FN817_CALLS_C5
        XOR A
FN817_CALLS_C5:
        OR A
        JR Z,FN817_CALLS_B66
FN817_CALLS_B64:
        ; inline col
        LD A,1
        JP FN817_CALLS_B65
FN817_CALLS_B66:
        XOR A
FN817_CALLS_B65:
        LD C,A
        LD A,E
        CALL FN95_MARK
        LD A,(G16_UB)
        LD C,A
        LD A,4
        LD L,A
        LD H,0
        PUSH HL
        LD HL,4352
        PUSH HL
        LD A,C
        LD L,A
        LD H,0
        PUSH HL
        CALL FN624_MIX
        INC SP
        INC SP
        INC SP
        INC SP
        INC SP
        INC SP
        CP 22
        LD A,1
        JR Z,FN817_CALLS_C6
        XOR A
FN817_CALLS_C6:
        LD C,A
        LD A,38
        CALL FN95_MARK
        LD A,(G16_UB)
        CALL FN629_ID
        LD C,A ; spill
        LD A,2
        CALL FN629_ID
        LD B,C
        LD C,A
        LD A,B
        CALL FN637_ADD8
        LD C,A ; spill
        LD A,3
        CALL FN629_ID
        LD B,A
        LD A,(G14_UA)
        LD E,C ; spill
        LD C,A
        LD A,B
        CALL FN637_ADD8
        LD C,A
        LD A,E
        CALL FN637_ADD8
        CP 5
        LD A,1
        JR Z,FN817_CALLS_C7
        XOR A
FN817_CALLS_C7:
        LD C,A
        LD A,39
        JP FN95_MARK
        ; spill report: 0 slot bytes, saves (bytes per site) none, 12 T-states of spill code
FN905_DATA:
FN905_DATA_B67:
        LD C,40
        LD A,(G58_I2)
        LD HL,G656_SPRITES
        LD D,H
        LD E,L
        LD L,A
        LD H,0
        ADD HL,HL
        ADD HL,HL
        ADD HL,DE
        INC HL
        LD A,(HL)
        CP 60
        LD A,1
        JR Z,FN905_DATA_C0
        XOR A
FN905_DATA_C0:
        OR A
        JR Z,FN905_DATA_B70
FN905_DATA_B68:
        LD A,(G56_I1)
        LD HL,G656_SPRITES
        LD D,H
        LD E,L
        LD L,A
        LD H,0
        ADD HL,HL
        ADD HL,HL
        ADD HL,DE
        INC HL
        INC HL
        LD A,(HL)
        INC HL
        LD H,(HL)
        LD L,A
        LD DE,22136
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN905_DATA_B69
        XOR A
FN905_DATA_C1:
        JP FN905_DATA_B69
FN905_DATA_B70:
        XOR A
FN905_DATA_B69:
        OR A
        JR Z,FN905_DATA_B73
FN905_DATA_B71:
        LD HL,G656_SPRITES
        LD A,(HL)
        CP 10
        LD A,1
        JR Z,FN905_DATA_B72
        XOR A
FN905_DATA_C2:
        JP FN905_DATA_B72
FN905_DATA_B73:
        XOR A
FN905_DATA_B72:
        LD B,C
        LD C,A
        LD A,B
        CALL FN95_MARK
        CALL FN701_LIST_SUM
        CP 7
        LD A,1
        JR Z,FN905_DATA_C3
        XOR A
FN905_DATA_C3:
        LD C,A
        LD A,41
        CALL FN95_MARK
        LD B,42
        LD HL,G703_NAME
        LD A,(HL)
        CP 3
        LD A,1
        JR Z,FN905_DATA_C4
        XOR A
FN905_DATA_C4:
        OR A
        JR Z,FN905_DATA_B76
FN905_DATA_B74:
        LD HL,G703_NAME
        INC HL
        INC HL
        LD A,(HL)
        CP 86
        LD A,1
        JR Z,FN905_DATA_B75
        XOR A
FN905_DATA_C5:
        JP FN905_DATA_B75
FN905_DATA_B76:
        XOR A
FN905_DATA_B75:
        LD C,A
        LD A,B
        CALL FN95_MARK
        LD HL,G712_DST
        XOR A
        LD (HL),A
        LD A,L
        ADD A,3
        LD L,A
        LD A,H
        ADC A,0
        LD H,A
        XOR A
        LD (HL),A
        LD HL,G710_SRC
        LD D,H
        LD E,L
        LD HL,G712_DST
        LD BC,4
        EX DE,HL
        LDIR
        LD E,43
        LD HL,G712_DST
        LD A,(HL)
        CP 1
        LD A,1
        JR Z,FN905_DATA_C6
        XOR A
FN905_DATA_C6:
        OR A
        JR Z,FN905_DATA_B79
FN905_DATA_B77:
        LD HL,G712_DST
        LD A,L
        ADD A,3
        LD L,A
        LD A,H
        ADC A,0
        LD H,A
        LD A,(HL)
        CP 4
        LD A,1
        JR Z,FN905_DATA_B78
        XOR A
FN905_DATA_C7:
        JP FN905_DATA_B78
FN905_DATA_B79:
        XOR A
FN905_DATA_B78:
        LD C,A
        LD A,E
        JP FN95_MARK
FN925_BATTERY:
FN925_BATTERY_B80:
        ; inline ops8 (auto)
        LD A,(G14_UA)
        LD C,A
        LD A,(G16_UB)
        LD B,A
        LD A,C
        ADD A,B
        OR A
        LD A,1
        JR Z,FN925_BATTERY_C0
        XOR A
FN925_BATTERY_C0:
        LD C,A
        XOR A
        CALL FN95_MARK
        LD A,(G16_UB)
        LD C,A
        LD A,(G14_UA)
        LD B,A
        LD A,C
        SUB B
        CP 2
        LD A,1
        JR Z,FN925_BATTERY_C1
        XOR A
FN925_BATTERY_C1:
        LD C,A
        LD A,1
        CALL FN95_MARK
        LD A,(G14_UA)
        AND 15
        LD C,A
        LD A,(G16_UB)
        XOR 128
        LD B,A
        LD A,C
        OR B
        CP 143
        LD A,1
        JR Z,FN925_BATTERY_C2
        XOR A
FN925_BATTERY_C2:
        LD C,A
        LD A,2
        CALL FN95_MARK
        LD A,(G16_UB)
        CPL
        CP 254
        LD A,1
        JR Z,FN925_BATTERY_C3
        XOR A
FN925_BATTERY_C3:
        LD C,A
        LD A,3
        CALL FN95_MARK
        LD A,(G23_SC)
        LD C,A
        LD A,(G21_SB)
        LD B,A
        LD A,C
        ADD A,B
        CP 128
        LD A,1
        JR Z,FN925_BATTERY_C4
        XOR A
FN925_BATTERY_C4:
        LD C,A
        LD A,4
        CALL FN95_MARK
        LD A,(G19_SA)
        LD C,A
        LD A,(G21_SB)
        LD B,A
        LD A,C
        SUB B
        CP 127
        LD A,1
        JR Z,FN925_BATTERY_C5
        XOR A
FN925_BATTERY_C5:
        LD C,A
        LD A,5
        CALL FN95_MARK
        LD A,(G19_SA)
        NEG
        CP 128
        LD A,1
        JR Z,FN925_BATTERY_C6
        XOR A
FN925_BATTERY_C6:
        LD C,A
        LD A,6
        CALL FN95_MARK
        LD A,(G19_SA)
        LD C,A
        LD A,(G21_SB)
        LD B,A
        LD A,C
        OR B
        CP 129
        LD A,1
        JR Z,FN925_BATTERY_C7
        XOR A
FN925_BATTERY_C7:
        LD C,A
        LD A,7
        CALL FN95_MARK
        ; inline ops16 (auto)
        LD HL,(G25_WA)
        LD D,H
        LD E,L
        LD HL,(G27_WB)
        ADD HL,DE
        LD DE,0
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C8
        XOR A
FN925_BATTERY_C8:
        LD C,A
        LD A,8
        CALL FN95_MARK
        LD HL,(G27_WB)
        LD D,H
        LD E,L
        LD HL,(G25_WA)
        LD B,H
        LD C,L
        LD H,D
        LD L,E
        OR A
        SBC HL,BC
        LD DE,2
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C9
        XOR A
FN925_BATTERY_C9:
        LD C,A
        LD A,9
        CALL FN95_MARK
        LD HL,(G25_WA)
        LD A,H
        AND 15
        LD H,A
        LD A,L
        AND 15
        LD L,A
        LD D,H
        LD E,L
        LD HL,(G29_WC)
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
        LD DE,36623
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C10
        XOR A
FN925_BATTERY_C10:
        LD C,A
        LD A,10
        CALL FN95_MARK
        LD HL,(G29_WC)
        LD A,H
        CPL
        LD H,A
        LD A,L
        CPL
        LD L,A
        LD DE,32767
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C11
        XOR A
FN925_BATTERY_C11:
        LD C,A
        LD A,11
        CALL FN95_MARK
        LD HL,(G36_IC)
        LD D,H
        LD E,L
        LD HL,(G34_IB)
        ADD HL,DE
        LD DE,32768
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C12
        XOR A
FN925_BATTERY_C12:
        LD C,A
        LD A,12
        CALL FN95_MARK
        LD HL,(G32_IA)
        LD D,H
        LD E,L
        LD HL,(G34_IB)
        LD B,H
        LD C,L
        LD H,D
        LD L,E
        OR A
        SBC HL,BC
        LD DE,32767
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C13
        XOR A
FN925_BATTERY_C13:
        LD C,A
        LD A,13
        CALL FN95_MARK
        LD HL,(G32_IA)
        XOR A
        SUB L
        LD L,A
        LD A,0
        SBC A,H
        LD H,A
        LD DE,32768
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C14
        XOR A
FN925_BATTERY_C14:
        LD C,A
        LD A,14
        CALL FN95_MARK
        LD HL,(G32_IA)
        LD D,H
        LD E,L
        LD HL,(G34_IB)
        LD B,H
        LD C,L
        LD H,D
        LD L,E
        LD A,H
        OR B
        LD H,A
        LD A,L
        OR C
        LD L,A
        LD DE,32769
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C15
        XOR A
FN925_BATTERY_C15:
        LD C,A
        LD A,15
        CALL FN95_MARK
        CALL FN338_COMPARES
        ; inline shifts (auto)
        LD C,24
        LD A,(G14_UA)
        RRCA
        RRCA
        RRCA
        RRCA
        AND 15
        CP 15
        LD A,1
        JR Z,FN925_BATTERY_C16
        XOR A
FN925_BATTERY_C16:
        OR A
        JR Z,FN925_BATTERY_B83
FN925_BATTERY_B82:
        LD A,(G16_UB)
        RRCA
        AND 128
        CP 128
        LD A,1
        JR Z,FN925_BATTERY_B84
        XOR A
FN925_BATTERY_C17:
        JP FN925_BATTERY_B84
FN925_BATTERY_B83:
        XOR A
FN925_BATTERY_B84:
        LD B,C
        LD C,A
        LD A,B
        CALL FN95_MARK
        LD A,(G19_SA)
        SRA A
        SRA A
        SRA A
        CP 240
        LD A,1
        JR Z,FN925_BATTERY_C18
        XOR A
FN925_BATTERY_C18:
        LD C,A
        LD A,25
        CALL FN95_MARK
        LD B,26
        LD HL,(G29_WC)
        LD A,H
        RLCA
        AND 1
        LD L,A
        LD H,0
        LD DE,1
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C19
        XOR A
FN925_BATTERY_C19:
        OR A
        JR Z,FN925_BATTERY_B86
FN925_BATTERY_B85:
        LD HL,(G27_WB)
        LD A,L
        RRCA
        AND 128
        LD H,A
        LD L,0
        LD DE,32768
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_B87
        XOR A
FN925_BATTERY_C20:
        JP FN925_BATTERY_B87
FN925_BATTERY_B86:
        XOR A
FN925_BATTERY_B87:
        LD C,A
        LD A,B
        CALL FN95_MARK
        LD HL,(G32_IA)
        LD A,H
        RLA
        SBC A,A
        LD H,A
        LD L,A
        LD DE,65535
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C21
        XOR A
FN925_BATTERY_C21:
        LD C,A
        LD A,27
        CALL FN95_MARK
        LD E,28
        LD A,(G14_UA)
        LD C,A
        LD A,(G38_K3)
        LD B,A
        LD A,C
        INC B
        DEC B
        JR Z,FN925_BATTERY_C23
FN925_BATTERY_C22:
        SRL A
        DJNZ FN925_BATTERY_C22
FN925_BATTERY_C23:
        CP 31
        LD A,1
        JR Z,FN925_BATTERY_C24
        XOR A
FN925_BATTERY_C24:
        OR A
        JR Z,FN925_BATTERY_B89
FN925_BATTERY_B88:
        LD A,(G16_UB)
        LD C,A
        LD A,(G38_K3)
        LD B,A
        LD A,C
        INC B
        DEC B
        JR Z,FN925_BATTERY_C26
FN925_BATTERY_C25:
        ADD A,A
        DJNZ FN925_BATTERY_C25
FN925_BATTERY_C26:
        CP 8
        LD A,1
        JR Z,FN925_BATTERY_B90
        XOR A
FN925_BATTERY_C27:
        JP FN925_BATTERY_B90
FN925_BATTERY_B89:
        XOR A
FN925_BATTERY_B90:
        LD C,A
        LD A,E
        CALL FN95_MARK
        LD D,29
        LD A,(G14_UA)
        LD C,A
        LD A,(G42_K8)
        LD B,A
        LD A,C
        INC B
        DEC B
        JR Z,FN925_BATTERY_C29
FN925_BATTERY_C28:
        SRL A
        DJNZ FN925_BATTERY_C28
FN925_BATTERY_C29:
        OR A
        LD A,1
        JR Z,FN925_BATTERY_C30
        XOR A
FN925_BATTERY_C30:
        OR A
        JR Z,FN925_BATTERY_B92
FN925_BATTERY_B91:
        LD A,(G16_UB)
        LD C,A
        LD A,(G44_K9)
        LD B,A
        LD A,C
        INC B
        DEC B
        JR Z,FN925_BATTERY_C32
FN925_BATTERY_C31:
        ADD A,A
        DJNZ FN925_BATTERY_C31
FN925_BATTERY_C32:
        OR A
        LD A,1
        JR Z,FN925_BATTERY_B93
        XOR A
FN925_BATTERY_C33:
        JP FN925_BATTERY_B93
FN925_BATTERY_B92:
        XOR A
FN925_BATTERY_B93:
        LD C,A
        LD A,D
        CALL FN95_MARK
        LD L,30
        LD A,(G19_SA)
        LD C,A
        LD A,(G44_K9)
        LD B,A
        LD A,C
        INC B
        DEC B
        JR Z,FN925_BATTERY_C35
FN925_BATTERY_C34:
        SRA A
        DJNZ FN925_BATTERY_C34
FN925_BATTERY_C35:
        CP 255
        LD A,1
        JR Z,FN925_BATTERY_C36
        XOR A
FN925_BATTERY_C36:
        OR A
        JR Z,FN925_BATTERY_B95
FN925_BATTERY_B94:
        LD A,(G23_SC)
        LD C,A
        LD A,(G42_K8)
        LD B,A
        LD A,C
        INC B
        DEC B
        JR Z,FN925_BATTERY_C38
FN925_BATTERY_C37:
        SRA A
        DJNZ FN925_BATTERY_C37
FN925_BATTERY_C38:
        OR A
        LD A,1
        JR Z,FN925_BATTERY_B96
        XOR A
FN925_BATTERY_C39:
        JP FN925_BATTERY_B96
FN925_BATTERY_B95:
        XOR A
FN925_BATTERY_B96:
        LD C,A
        LD A,L
        CALL FN95_MARK
        LD H,31
        LD C,H
        LD HL,(G25_WA)
        LD A,(G46_K16)
        LD B,A
        INC B
        DEC B
        JR Z,FN925_BATTERY_C41
FN925_BATTERY_C40:
        ADD HL,HL
        DJNZ FN925_BATTERY_C40
FN925_BATTERY_C41:
        LD DE,0
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C42
        XOR A
FN925_BATTERY_C42:
        LD H,C
        OR A
        JR Z,FN925_BATTERY_B98
FN925_BATTERY_B97:
        LD HL,(G32_IA)
        LD A,(G48_K20)
        LD B,A
        INC B
        DEC B
        JR Z,FN925_BATTERY_C44
FN925_BATTERY_C43:
        SRA H
        RR L
        DJNZ FN925_BATTERY_C43
FN925_BATTERY_C44:
        LD DE,65535
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C45
        XOR A
FN925_BATTERY_C45:
        LD H,C
        JP FN925_BATTERY_B99
FN925_BATTERY_B98:
        XOR A
FN925_BATTERY_B99:
        OR A
        JR Z,FN925_BATTERY_B101
FN925_BATTERY_B100:
        LD C,H
        LD HL,(G25_WA)
        LD A,(G40_K4)
        LD B,A
        INC B
        DEC B
        JR Z,FN925_BATTERY_C47
FN925_BATTERY_C46:
        SRL H
        RR L
        DJNZ FN925_BATTERY_C46
FN925_BATTERY_C47:
        LD DE,4095
        OR A
        SBC HL,DE
        LD A,1
        JR Z,FN925_BATTERY_C48
        XOR A
FN925_BATTERY_C48:
        LD H,C
        JP FN925_BATTERY_B102
FN925_BATTERY_B101:
        XOR A
FN925_BATTERY_B102:
        LD C,A
        LD A,H
        CALL FN95_MARK
        CALL FN817_CALLS
        JP FN905_DATA
FN981_RUN:
        LD D,H
        LD E,L
FN981_RUN_B81:
        LD C,E
        XOR A
        LD (G12_FAILS),A
        LD HL,0
        LD (G9_CHECKS),HL
        XOR A
        LD (G10_PASSES),A
        LD B,0
        JR FN981_RUN_B82
FN981_RUN_B83:
        LD HL,G6_SEEN
        LD D,H
        LD E,L
        LD L,B
        LD H,0
        ADD HL,DE
        XOR A
        LD (HL),A
        LD HL,G8_FAILED
        LD D,H
        LD E,L
        LD L,B
        LD H,0
        ADD HL,DE
        XOR A
        LD (HL),A
FN981_RUN_B84:
        INC B
FN981_RUN_B82:
        LD A,B
        CP 6
        JP C,FN981_RUN_B83
FN981_RUN_B85:
        JR FN981_RUN_B86
FN981_RUN_B87:
        PUSH BC ; spill
        CALL FN925_BATTERY
        POP BC ; spill
        LD A,(G10_PASSES)
        INC A
        LD (G10_PASSES),A
        DEC C
FN981_RUN_B86:
        LD A,C
        OR A
        JP NZ,FN981_RUN_B87
FN981_RUN_B88:
        LD A,(G12_FAILS)
        LD L,A
        LD H,0
        RET
        ; spill report: 0 slot bytes, saves (bytes per site) 2, 21 T-states of spill code
ORG 0x19EF
        DB 0x49, 0x0A, 0x00, 0xFC, 0x20, 0x43, 0x38, 0x30, 0x20, 0x73, 0x65, 0x6C, 0x66, 0x2D, 0x63, 0x68
        DB 0x65, 0x63, 0x6B, 0x2E, 0x20, 0x73, 0x65, 0x6C, 0x66, 0x63, 0x68, 0x65, 0x63, 0x6B, 0x2E, 0x63
        DB 0x38, 0x30, 0x20, 0x6C, 0x69, 0x73, 0x74, 0x73, 0x20, 0x77, 0x68, 0x61, 0x74, 0x20, 0x65, 0x61
        DB 0x63, 0x68, 0x20, 0x62, 0x69, 0x74, 0x20, 0x6F, 0x66, 0x20, 0x46, 0x41, 0x49, 0x4C, 0x45, 0x44
        DB 0x20, 0x63, 0x68, 0x65, 0x63, 0x6B, 0x73, 0x2E, 0xFF, 0x15, 0x14, 0x00, 0xEB, 0x20, 0x52, 0x9A
        DB 0x55, 0x53, 0x52, 0x96, 0x31, 0x34, 0x32, 0x35, 0x38, 0xA4, 0x34, 0x30, 0x95, 0xFF, 0x27, 0x1E
        DB 0x00, 0xDD, 0x20, 0x22, 0x66, 0x61, 0x69, 0x6C, 0x73, 0x3D, 0x22, 0xA0, 0x52, 0xA0, 0x22, 0x20
        DB 0x70, 0x61, 0x73, 0x73, 0x65, 0x73, 0x3D, 0x22, 0xA0, 0x50, 0x45, 0x45, 0x4B, 0x96, 0x31, 0x32
        DB 0x33, 0x30, 0x32, 0x95, 0xFF, 0x0D, 0x28, 0x00, 0xF2, 0x20, 0x49, 0x9A, 0x30, 0x20, 0xB4, 0x20
        DB 0x35, 0xFF, 0x14, 0x32, 0x00, 0xDD, 0x20, 0x50, 0x45, 0x45, 0x4B, 0x96, 0x31, 0x32, 0x32, 0x39
        DB 0x34, 0x98, 0x49, 0x95, 0xA0, 0xFF, 0x07, 0x3C, 0x00, 0xE5, 0x20, 0x49, 0xFF, 0x05, 0x46, 0x00
        DB 0xDD, 0xFF, 0x0D, 0x50, 0x00, 0xDE, 0x20, 0x31, 0x32, 0x33, 0x30, 0x33, 0xA4, 0x31, 0xFF, 0x00
