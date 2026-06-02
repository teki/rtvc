# Z80 opcode referencia

Ez a fájl megőrzi a [../info/opcodes.txt](../info/opcodes.txt) egy utasítás per sor elrendezését, és kiegészíti a [../info/z80href.txt](../info/z80href.txt), [../info/z80inst.txt](../info/z80inst.txt), valamint a kompakt disassembler ([src/disasm.rs](../src/disasm.rs)) metaadataival.

Metaadat-jelmagyarázat:

- `T`: T-state szám. Feltételes időzítésnél `teljesül/nem teljesül` alakot használ, például `12/7`.
- `F`: érintett flagek `SZHPNC` sorrendben.
- Flagkarakterek: `-` változatlan, `*` módosul, `0` törölve, `1` beállítva, `P` paritás, `V` overflow, `?` nem definiált vagy nem dokumentált.

```text
NN      EQU     1234H           ; egy tizenhat bites szám
N       EQU     56H             ; egy nyolc bites szám

        NOP                     ; 00         ; T=4     F=------ nincs művelet
        LD BC,NN                ; 01 XX XX   ; T=10    F=------ betöltés: cél=forrás
        LD (BC),A               ; 02         ; T=7     F=------ betöltés: cél=forrás
        INC BC                  ; 03         ; T=6     F=------ növelés: xx=xx+1
        INC B                   ; 04         ; T=4     F=***V0- növelés: s=s+1
        DEC B                   ; 05         ; T=4     F=***V1- csökkentés: s=s-1
        LD B,N                  ; 06 XX      ; T=7     F=------ betöltés: cél=forrás
        RLCA                    ; 07         ; T=4     F=--0-0* körkörös balra forgatás: A=A<-
        EX AF,AF'               ; 08         ; T=4     F=------ csere: AF<->AF'
        ADD HL,BC               ; 09         ; T=11    F=--?-0* összeadás: HL=HL+ss
        LD A,(BC)               ; 0A         ; T=7     F=------ betöltés: cél=forrás
        DEC BC                  ; 0B         ; T=6     F=------ csökkentés: xx=xx-1
        INC C                   ; 0C         ; T=4     F=***V0- növelés: s=s+1
        DEC C                   ; 0D         ; T=4     F=***V1- csökkentés: s=s-1
        LD C,N                  ; 0E XX      ; T=7     F=------ betöltés: cél=forrás
        RRCA                    ; 0F         ; T=4     F=--0-0* körkörös jobbra forgatás: A=->A
        DJNZ $+2                ; 10         ; T=13/8  F=------ csökkentés és ugrás nem nulla esetén: B=B-1 till B=0
        LD DE,NN                ; 11 XX XX   ; T=10    F=------ betöltés: cél=forrás
        LD (DE),A               ; 12         ; T=7     F=------ betöltés: cél=forrás
        INC DE                  ; 13         ; T=6     F=------ növelés: xx=xx+1
        INC D                   ; 14         ; T=4     F=***V0- növelés: s=s+1
        DEC D                   ; 15         ; T=4     F=***V1- csökkentés: s=s-1
        LD D,N                  ; 16 XX      ; T=7     F=------ betöltés: cél=forrás
        RLA                     ; 17         ; T=4     F=--0-0* akkumulátor forgatása balra: A={CY,A}<-
        JR $+2                  ; 18         ; T=12    F=------ feltétel nélküli ugrás: PC=PC+e
        ADD HL,DE               ; 19         ; T=11    F=--?-0* összeadás: HL=HL+ss
        LD A,(DE)               ; 1A         ; T=7     F=------ betöltés: cél=forrás
        DEC DE                  ; 1B         ; T=6     F=------ csökkentés: xx=xx-1
        INC E                   ; 1C         ; T=4     F=***V0- növelés: s=s+1
        DEC E                   ; 1D         ; T=4     F=***V1- csökkentés: s=s-1
        LD E,N                  ; 1E XX      ; T=7     F=------ betöltés: cél=forrás
        RRA                     ; 1F         ; T=4     F=--0-0* akkumulátor forgatása jobbra: A=->{CY,A}
        JR NZ,$+2               ; 20         ; T=12/7  F=------ feltételes ugrás: ha cc, JR
        LD HL,NN                ; 21 XX XX   ; T=10    F=------ betöltés: cél=forrás
        LD (NN),HL              ; 22 XX XX   ; T=16    F=------ betöltés: cél=forrás
        INC HL                  ; 23         ; T=6     F=------ növelés: xx=xx+1
        INC H                   ; 24         ; T=4     F=***V0- növelés: s=s+1
        DEC H                   ; 25         ; T=4     F=***V1- csökkentés: s=s-1
        LD H,N                  ; 26 XX      ; T=7     F=------ betöltés: cél=forrás
        DAA                     ; 27         ; T=4     F=***P-* decimális korrekció: A=BCD format
        JR Z,$+2                ; 28         ; T=12/7  F=------ feltételes ugrás: ha cc, JR
        ADD HL,HL               ; 29         ; T=11    F=--?-0* összeadás: HL=HL+ss
        LD HL,(NN)              ; 2A XX XX   ; T=16    F=------ betöltés: cél=forrás
        DEC HL                  ; 2B         ; T=6     F=------ csökkentés: xx=xx-1
        INC L                   ; 2C         ; T=4     F=***V0- növelés: s=s+1
        DEC L                   ; 2D         ; T=4     F=***V1- csökkentés: s=s-1
        LD L,N                  ; 2E XX      ; T=7     F=------ betöltés: cél=forrás
        CPL                     ; 2F         ; T=4     F=--1-1- komplemens: A=~A
        JR NC,$+2               ; 30         ; T=12/7  F=------ feltételes ugrás: ha cc, JR
        LD SP,NN                ; 31 XX XX   ; T=10    F=------ betöltés: cél=forrás
        LD (NN),A               ; 32 XX XX   ; T=13    F=------ betöltés: cél=forrás
        INC SP                  ; 33         ; T=6     F=------ növelés: xx=xx+1
        INC (HL)                ; 34         ; T=11    F=***V0- növelés: s=s+1
        DEC (HL)                ; 35         ; T=11    F=***V1- csökkentés: s=s-1
        LD (HL),N               ; 36 XX      ; T=10    F=------ betöltés: cél=forrás
        SCF                     ; 37         ; T=4     F=--0-01 carry flag beállítása: CY=1
        JR C,$+2                ; 38         ; T=12/7  F=------ feltételes ugrás: ha cc, JR
        ADD HL,SP               ; 39         ; T=11    F=--?-0* összeadás: HL=HL+ss
        LD A,(NN)               ; 3A XX XX   ; T=13    F=------ betöltés: cél=forrás
        DEC SP                  ; 3B         ; T=6     F=------ csökkentés: xx=xx-1
        INC A                   ; 3C         ; T=4     F=***V0- növelés: s=s+1
        DEC A                   ; 3D         ; T=4     F=***V1- csökkentés: s=s-1
        LD A,N                  ; 3E XX      ; T=7     F=------ betöltés: cél=forrás
        CCF                     ; 3F         ; T=4     F=--*-0* carry flag invertálása: CY=~CY
        LD B,B                  ; 40         ; T=4     F=------ betöltés: cél=forrás
        LD B,C                  ; 41         ; T=4     F=------ betöltés: cél=forrás
        LD B,D                  ; 42         ; T=4     F=------ betöltés: cél=forrás
        LD B,E                  ; 43         ; T=4     F=------ betöltés: cél=forrás
        LD B,H                  ; 44         ; T=4     F=------ betöltés: cél=forrás
        LD B,L                  ; 45         ; T=4     F=------ betöltés: cél=forrás
        LD B,(HL)               ; 46         ; T=7     F=------ betöltés: cél=forrás
        LD B,A                  ; 47         ; T=4     F=------ betöltés: cél=forrás
        LD C,B                  ; 48         ; T=4     F=------ betöltés: cél=forrás
        LD C,C                  ; 49         ; T=4     F=------ betöltés: cél=forrás
        LD C,D                  ; 4A         ; T=4     F=------ betöltés: cél=forrás
        LD C,E                  ; 4B         ; T=4     F=------ betöltés: cél=forrás
        LD C,H                  ; 4C         ; T=4     F=------ betöltés: cél=forrás
        LD C,L                  ; 4D         ; T=4     F=------ betöltés: cél=forrás
        LD C,(HL)               ; 4E         ; T=7     F=------ betöltés: cél=forrás
        LD C,A                  ; 4F         ; T=4     F=------ betöltés: cél=forrás
        LD D,B                  ; 50         ; T=4     F=------ betöltés: cél=forrás
        LD D,C                  ; 51         ; T=4     F=------ betöltés: cél=forrás
        LD D,D                  ; 52         ; T=4     F=------ betöltés: cél=forrás
        LD D,E                  ; 53         ; T=4     F=------ betöltés: cél=forrás
        LD D,H                  ; 54         ; T=4     F=------ betöltés: cél=forrás
        LD D,L                  ; 55         ; T=4     F=------ betöltés: cél=forrás
        LD D,(HL)               ; 56         ; T=7     F=------ betöltés: cél=forrás
        LD D,A                  ; 57         ; T=4     F=------ betöltés: cél=forrás
        LD E,B                  ; 58         ; T=4     F=------ betöltés: cél=forrás
        LD E,C                  ; 59         ; T=4     F=------ betöltés: cél=forrás
        LD E,D                  ; 5A         ; T=4     F=------ betöltés: cél=forrás
        LD E,E                  ; 5B         ; T=4     F=------ betöltés: cél=forrás
        LD E,H                  ; 5C         ; T=4     F=------ betöltés: cél=forrás
        LD E,L                  ; 5D         ; T=4     F=------ betöltés: cél=forrás
        LD E,(HL)               ; 5E         ; T=7     F=------ betöltés: cél=forrás
        LD E,A                  ; 5F         ; T=4     F=------ betöltés: cél=forrás
        LD H,B                  ; 60         ; T=4     F=------ betöltés: cél=forrás
        LD H,C                  ; 61         ; T=4     F=------ betöltés: cél=forrás
        LD H,D                  ; 62         ; T=4     F=------ betöltés: cél=forrás
        LD H,E                  ; 63         ; T=4     F=------ betöltés: cél=forrás
        LD H,H                  ; 64         ; T=4     F=------ betöltés: cél=forrás
        LD H,L                  ; 65         ; T=4     F=------ betöltés: cél=forrás
        LD H,(HL)               ; 66         ; T=7     F=------ betöltés: cél=forrás
        LD H,A                  ; 67         ; T=4     F=------ betöltés: cél=forrás
        LD L,B                  ; 68         ; T=4     F=------ betöltés: cél=forrás
        LD L,C                  ; 69         ; T=4     F=------ betöltés: cél=forrás
        LD L,D                  ; 6A         ; T=4     F=------ betöltés: cél=forrás
        LD L,E                  ; 6B         ; T=4     F=------ betöltés: cél=forrás
        LD L,H                  ; 6C         ; T=4     F=------ betöltés: cél=forrás
        LD L,L                  ; 6D         ; T=4     F=------ betöltés: cél=forrás
        LD L,(HL)               ; 6E         ; T=7     F=------ betöltés: cél=forrás
        LD L,A                  ; 6F         ; T=4     F=------ betöltés: cél=forrás
        LD (HL),B               ; 70         ; T=7     F=------ betöltés: cél=forrás
        LD (HL),C               ; 71         ; T=7     F=------ betöltés: cél=forrás
        LD (HL),D               ; 72         ; T=7     F=------ betöltés: cél=forrás
        LD (HL),E               ; 73         ; T=7     F=------ betöltés: cél=forrás
        LD (HL),H               ; 74         ; T=7     F=------ betöltés: cél=forrás
        LD (HL),L               ; 75         ; T=7     F=------ betöltés: cél=forrás
        HALT                    ; 76         ; T=4     F=------ Halt
        LD (HL),A               ; 77         ; T=7     F=------ betöltés: cél=forrás
        LD A,B                  ; 78         ; T=4     F=------ betöltés: cél=forrás
        LD A,C                  ; 79         ; T=4     F=------ betöltés: cél=forrás
        LD A,D                  ; 7A         ; T=4     F=------ betöltés: cél=forrás
        LD A,E                  ; 7B         ; T=4     F=------ betöltés: cél=forrás
        LD A,H                  ; 7C         ; T=4     F=------ betöltés: cél=forrás
        LD A,L                  ; 7D         ; T=4     F=------ betöltés: cél=forrás
        LD A,(HL)               ; 7E         ; T=7     F=------ betöltés: cél=forrás
        LD A,A                  ; 7F         ; T=4     F=------ betöltés: cél=forrás
        ADD A,B                 ; 80         ; T=4     F=***V0* összeadás: A=A+s
        ADD A,C                 ; 81         ; T=4     F=***V0* összeadás: A=A+s
        ADD A,D                 ; 82         ; T=4     F=***V0* összeadás: A=A+s
        ADD A,E                 ; 83         ; T=4     F=***V0* összeadás: A=A+s
        ADD A,H                 ; 84         ; T=4     F=***V0* összeadás: A=A+s
        ADD A,L                 ; 85         ; T=4     F=***V0* összeadás: A=A+s
        ADD A,(HL)              ; 86         ; T=7     F=***V0* összeadás: A=A+s
        ADD A,A                 ; 87         ; T=4     F=***V0* összeadás: A=A+s
        ADC A,B                 ; 88         ; T=4     F=***V0* Add with Carry: A=A+s+CY
        ADC A,C                 ; 89         ; T=4     F=***V0* Add with Carry: A=A+s+CY
        ADC A,D                 ; 8A         ; T=4     F=***V0* Add with Carry: A=A+s+CY
        ADC A,E                 ; 8B         ; T=4     F=***V0* Add with Carry: A=A+s+CY
        ADC A,H                 ; 8C         ; T=4     F=***V0* Add with Carry: A=A+s+CY
        ADC A,L                 ; 8D         ; T=4     F=***V0* Add with Carry: A=A+s+CY
        ADC A,(HL)              ; 8E         ; T=7     F=***V0* Add with Carry: A=A+s+CY
        ADC A,A                 ; 8F         ; T=4     F=***V0* Add with Carry: A=A+s+CY
        SUB B                   ; 90         ; T=4     F=***V1* kivonás: A=A-s
        SUB C                   ; 91         ; T=4     F=***V1* kivonás: A=A-s
        SUB D                   ; 92         ; T=4     F=***V1* kivonás: A=A-s
        SUB E                   ; 93         ; T=4     F=***V1* kivonás: A=A-s
        SUB H                   ; 94         ; T=4     F=***V1* kivonás: A=A-s
        SUB L                   ; 95         ; T=4     F=***V1* kivonás: A=A-s
        SUB (HL)                ; 96         ; T=7     F=***V1* kivonás: A=A-s
        SUB A                   ; 97         ; T=4     F=***V1* kivonás: A=A-s
        SBC B                   ; 98         ; T=4     F=***V1* Subtract with Carry: A=A-s-CY
        SBC C                   ; 99         ; T=4     F=***V1* Subtract with Carry: A=A-s-CY
        SBC D                   ; 9A         ; T=4     F=***V1* Subtract with Carry: A=A-s-CY
        SBC E                   ; 9B         ; T=4     F=***V1* Subtract with Carry: A=A-s-CY
        SBC H                   ; 9C         ; T=4     F=***V1* Subtract with Carry: A=A-s-CY
        SBC L                   ; 9D         ; T=4     F=***V1* Subtract with Carry: A=A-s-CY
        SBC (HL)                ; 9E         ; T=7     F=***V1* Subtract with Carry: A=A-s-CY
        SBC A                   ; 9F         ; T=4     F=***V1* Subtract with Carry: A=A-s-CY
        AND B                   ; A0         ; T=4     F=***P00 Logical AND: A=A&s
        AND C                   ; A1         ; T=4     F=***P00 Logical AND: A=A&s
        AND D                   ; A2         ; T=4     F=***P00 Logical AND: A=A&s
        AND E                   ; A3         ; T=4     F=***P00 Logical AND: A=A&s
        AND H                   ; A4         ; T=4     F=***P00 Logical AND: A=A&s
        AND L                   ; A5         ; T=4     F=***P00 Logical AND: A=A&s
        AND (HL)                ; A6         ; T=7     F=***P00 Logical AND: A=A&s
        AND A                   ; A7         ; T=4     F=***P00 Logical AND: A=A&s
        XOR B                   ; A8         ; T=4     F=***P00 Logical Exclusive OR: A=Axs
        XOR C                   ; A9         ; T=4     F=***P00 Logical Exclusive OR: A=Axs
        XOR D                   ; AA         ; T=4     F=***P00 Logical Exclusive OR: A=Axs
        XOR E                   ; AB         ; T=4     F=***P00 Logical Exclusive OR: A=Axs
        XOR H                   ; AC         ; T=4     F=***P00 Logical Exclusive OR: A=Axs
        XOR L                   ; AD         ; T=4     F=***P00 Logical Exclusive OR: A=Axs
        XOR (HL)                ; AE         ; T=7     F=***P00 Logical Exclusive OR: A=Axs
        XOR A                   ; AF         ; T=4     F=***P00 Logical Exclusive OR: A=Axs
        OR B                    ; B0         ; T=4     F=***P00 Logical inclusive OR: A=Avs
        OR C                    ; B1         ; T=4     F=***P00 Logical inclusive OR: A=Avs
        OR D                    ; B2         ; T=4     F=***P00 Logical inclusive OR: A=Avs
        OR E                    ; B3         ; T=4     F=***P00 Logical inclusive OR: A=Avs
        OR H                    ; B4         ; T=4     F=***P00 Logical inclusive OR: A=Avs
        OR L                    ; B5         ; T=4     F=***P00 Logical inclusive OR: A=Avs
        OR (HL)                 ; B6         ; T=7     F=***P00 Logical inclusive OR: A=Avs
        OR A                    ; B7         ; T=4     F=***P00 Logical inclusive OR: A=Avs
        CP B                    ; B8         ; T=4     F=***V1* összehasonlítás: A-s
        CP C                    ; B9         ; T=4     F=***V1* összehasonlítás: A-s
        CP D                    ; BA         ; T=4     F=***V1* összehasonlítás: A-s
        CP E                    ; BB         ; T=4     F=***V1* összehasonlítás: A-s
        CP H                    ; BC         ; T=4     F=***V1* összehasonlítás: A-s
        CP L                    ; BD         ; T=4     F=***V1* összehasonlítás: A-s
        CP (HL)                 ; BE         ; T=7     F=***V1* összehasonlítás: A-s
        CP A                    ; BF         ; T=4     F=***V1* összehasonlítás: A-s
        RET NZ                  ; C0         ; T=11/5  F=------ Conditional visszatérés: If cc RET
        POP BC                  ; C1         ; T=10    F=------ pop: register=[SP]+
        JP NZ,$+3               ; C2         ; T=10    F=------ feltételes ugrás: If cc JP
        JP $+3                  ; C3         ; T=10    F=------ feltétel nélküli ugrás: PC=nn
        CALL NZ,NN              ; C4 XX XX   ; T=17/10 F=------ Conditional hívás: If cc CALL
        PUSH BC                 ; C5         ; T=11    F=------ push: -[SP]=register
        ADD A,N                 ; C6 XX      ; T=7     F=***V0* összeadás: A=A+s
        RST 0                   ; C7         ; T=11    F=------ restart: (p=0H,8H,10H,...,38H)
        RET Z                   ; C8         ; T=11/5  F=------ Conditional visszatérés: If cc RET
        RET                     ; C9         ; T=10    F=------ visszatérés: PC=[SP]+
        JP Z,$+3                ; CA         ; T=10    F=------ feltételes ugrás: If cc JP
        RLC B                   ; CB 00      ; T=8     F=**0P0* körkörös balra forgatás: m=m<-
        RLC C                   ; CB 01      ; T=8     F=**0P0* körkörös balra forgatás: m=m<-
        RLC D                   ; CB 02      ; T=8     F=**0P0* körkörös balra forgatás: m=m<-
        RLC E                   ; CB 03      ; T=8     F=**0P0* körkörös balra forgatás: m=m<-
        RLC H                   ; CB 04      ; T=8     F=**0P0* körkörös balra forgatás: m=m<-
        RLC L                   ; CB 05      ; T=8     F=**0P0* körkörös balra forgatás: m=m<-
        RLC (HL)                ; CB 06      ; T=15    F=**0P0* körkörös balra forgatás: m=m<-
        RLC A                   ; CB 07      ; T=8     F=**0P0* körkörös balra forgatás: m=m<-
        RRC B                   ; CB 08      ; T=8     F=**0P0* körkörös jobbra forgatás: m=->m
        RRC C                   ; CB 09      ; T=8     F=**0P0* körkörös jobbra forgatás: m=->m
        RRC D                   ; CB 0A      ; T=8     F=**0P0* körkörös jobbra forgatás: m=->m
        RRC E                   ; CB 0B      ; T=8     F=**0P0* körkörös jobbra forgatás: m=->m
        RRC H                   ; CB 0C      ; T=8     F=**0P0* körkörös jobbra forgatás: m=->m
        RRC L                   ; CB 0D      ; T=8     F=**0P0* körkörös jobbra forgatás: m=->m
        RRC (HL)                ; CB 0E      ; T=15    F=**0P0* körkörös jobbra forgatás: m=->m
        RRC A                   ; CB 0F      ; T=8     F=**0P0* körkörös jobbra forgatás: m=->m
        RL  B                   ; CB 10      ; T=8     F=**0P0* forgatás balra: m={CY,m}<-
        RL  C                   ; CB 11      ; T=8     F=**0P0* forgatás balra: m={CY,m}<-
        RL  D                   ; CB 12      ; T=8     F=**0P0* forgatás balra: m={CY,m}<-
        RL  E                   ; CB 13      ; T=8     F=**0P0* forgatás balra: m={CY,m}<-
        RL  H                   ; CB 14      ; T=8     F=**0P0* forgatás balra: m={CY,m}<-
        RL  L                   ; CB 15      ; T=8     F=**0P0* forgatás balra: m={CY,m}<-
        RL  (HL)                ; CB 16      ; T=15    F=**0P0* forgatás balra: m={CY,m}<-
        RL  A                   ; CB 17      ; T=8     F=**0P0* forgatás balra: m={CY,m}<-
        RR  B                   ; CB 18      ; T=8     F=**0P0* forgatás jobbra: m=->{CY,m}
        RR  C                   ; CB 19      ; T=8     F=**0P0* forgatás jobbra: m=->{CY,m}
        RR  D                   ; CB 1A      ; T=8     F=**0P0* forgatás jobbra: m=->{CY,m}
        RR  E                   ; CB 1B      ; T=8     F=**0P0* forgatás jobbra: m=->{CY,m}
        RR  H                   ; CB 1C      ; T=8     F=**0P0* forgatás jobbra: m=->{CY,m}
        RR  L                   ; CB 1D      ; T=8     F=**0P0* forgatás jobbra: m=->{CY,m}
        RR  (HL)                ; CB 1E      ; T=15    F=**0P0* forgatás jobbra: m=->{CY,m}
        RR  A                   ; CB 1F      ; T=8     F=**0P0* forgatás jobbra: m=->{CY,m}
        SLA B                   ; CB 20      ; T=8     F=**0P0* aritmetikai balra shift: m=m*2
        SLA C                   ; CB 21      ; T=8     F=**0P0* aritmetikai balra shift: m=m*2
        SLA D                   ; CB 22      ; T=8     F=**0P0* aritmetikai balra shift: m=m*2
        SLA E                   ; CB 23      ; T=8     F=**0P0* aritmetikai balra shift: m=m*2
        SLA H                   ; CB 24      ; T=8     F=**0P0* aritmetikai balra shift: m=m*2
        SLA L                   ; CB 25      ; T=8     F=**0P0* aritmetikai balra shift: m=m*2
        SLA (HL)                ; CB 26      ; T=15    F=**0P0* aritmetikai balra shift: m=m*2
        SLA A                   ; CB 27      ; T=8     F=**0P0* aritmetikai balra shift: m=m*2
        SRA B                   ; CB 28      ; T=8     F=**0P0* Shift Right Arith.: m=m/2
        SRA C                   ; CB 29      ; T=8     F=**0P0* Shift Right Arith.: m=m/2
        SRA D                   ; CB 2A      ; T=8     F=**0P0* Shift Right Arith.: m=m/2
        SRA E                   ; CB 2B      ; T=8     F=**0P0* Shift Right Arith.: m=m/2
        SRA H                   ; CB 2C      ; T=8     F=**0P0* Shift Right Arith.: m=m/2
        SRA L                   ; CB 2D      ; T=8     F=**0P0* Shift Right Arith.: m=m/2
        SRA (HL)                ; CB 2E      ; T=15    F=**0P0* Shift Right Arith.: m=m/2
        SRA A                   ; CB 2F      ; T=8     F=**0P0* Shift Right Arith.: m=m/2
        SRL B                   ; CB 38      ; T=8     F=**0P0* logikai jobbra shift: m=->{0,m,CY}
        SRL C                   ; CB 39      ; T=8     F=**0P0* logikai jobbra shift: m=->{0,m,CY}
        SRL D                   ; CB 3A      ; T=8     F=**0P0* logikai jobbra shift: m=->{0,m,CY}
        SRL E                   ; CB 3B      ; T=8     F=**0P0* logikai jobbra shift: m=->{0,m,CY}
        SRL H                   ; CB 3C      ; T=8     F=**0P0* logikai jobbra shift: m=->{0,m,CY}
        SRL L                   ; CB 3D      ; T=8     F=**0P0* logikai jobbra shift: m=->{0,m,CY}
        SRL (HL)                ; CB 3E      ; T=15    F=**0P0* logikai jobbra shift: m=->{0,m,CY}
        SRL A                   ; CB 3F      ; T=8     F=**0P0* logikai jobbra shift: m=->{0,m,CY}
        BIT 0,B                 ; CB 40      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 0,C                 ; CB 41      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 0,D                 ; CB 42      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 0,E                 ; CB 43      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 0,H                 ; CB 44      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 0,L                 ; CB 45      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 0,(HL)              ; CB 46      ; T=12    F=?*1?0- Test Bit: m&{2^b}
        BIT 0,A                 ; CB 47      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 1,B                 ; CB 48      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 1,C                 ; CB 49      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 1,D                 ; CB 4A      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 1,E                 ; CB 4B      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 1,H                 ; CB 4C      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 1,L                 ; CB 4D      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 1,(HL)              ; CB 4E      ; T=12    F=?*1?0- Test Bit: m&{2^b}
        BIT 1,A                 ; CB 4F      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 2,B                 ; CB 50      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 2,C                 ; CB 51      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 2,D                 ; CB 52      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 2,E                 ; CB 53      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 2,H                 ; CB 54      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 2,L                 ; CB 55      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 2,(HL)              ; CB 56      ; T=12    F=?*1?0- Test Bit: m&{2^b}
        BIT 2,A                 ; CB 57      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 3,B                 ; CB 58      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 3,C                 ; CB 59      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 3,D                 ; CB 5A      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 3,E                 ; CB 5B      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 3,H                 ; CB 5C      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 3,L                 ; CB 5D      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 3,(HL)              ; CB 5E      ; T=12    F=?*1?0- Test Bit: m&{2^b}
        BIT 3,A                 ; CB 5F      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 4,B                 ; CB 60      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 4,C                 ; CB 61      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 4,D                 ; CB 62      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 4,E                 ; CB 63      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 4,H                 ; CB 64      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 4,L                 ; CB 65      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 4,(HL)              ; CB 66      ; T=12    F=?*1?0- Test Bit: m&{2^b}
        BIT 4,A                 ; CB 67      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 5,B                 ; CB 68      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 5,C                 ; CB 69      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 5,D                 ; CB 6A      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 5,E                 ; CB 6B      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 5,H                 ; CB 6C      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 5,L                 ; CB 6D      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 5,(HL)              ; CB 6E      ; T=12    F=?*1?0- Test Bit: m&{2^b}
        BIT 5,A                 ; CB 6F      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 6,B                 ; CB 70      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 6,C                 ; CB 71      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 6,D                 ; CB 72      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 6,E                 ; CB 73      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 6,H                 ; CB 74      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 6,L                 ; CB 75      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 6,(HL)              ; CB 76      ; T=12    F=?*1?0- Test Bit: m&{2^b}
        BIT 6,A                 ; CB 77      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 7,B                 ; CB 78      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 7,C                 ; CB 79      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 7,D                 ; CB 7A      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 7,E                 ; CB 7B      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 7,H                 ; CB 7C      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 7,L                 ; CB 7D      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        BIT 7,(HL)              ; CB 7E      ; T=12    F=?*1?0- Test Bit: m&{2^b}
        BIT 7,A                 ; CB 7F      ; T=8     F=?*1?0- Test Bit: m&{2^b}
        RES 0,B                 ; CB 80      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 0,C                 ; CB 81      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 0,D                 ; CB 82      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 0,E                 ; CB 83      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 0,H                 ; CB 84      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 0,L                 ; CB 85      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 0,(HL)              ; CB 86      ; T=15    F=------ Reset bit: m=m&{~2^b}
        RES 0,A                 ; CB 87      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 1,B                 ; CB 88      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 1,C                 ; CB 89      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 1,D                 ; CB 8A      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 1,E                 ; CB 8B      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 1,H                 ; CB 8C      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 1,L                 ; CB 8D      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 1,(HL)              ; CB 8E      ; T=15    F=------ Reset bit: m=m&{~2^b}
        RES 1,A                 ; CB 8F      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 2,B                 ; CB 90      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 2,C                 ; CB 91      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 2,D                 ; CB 92      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 2,E                 ; CB 93      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 2,H                 ; CB 94      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 2,L                 ; CB 95      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 2,(HL)              ; CB 96      ; T=15    F=------ Reset bit: m=m&{~2^b}
        RES 2,A                 ; CB 97      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 3,B                 ; CB 98      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 3,C                 ; CB 99      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 3,D                 ; CB 9A      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 3,E                 ; CB 9B      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 3,H                 ; CB 9C      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 3,L                 ; CB 9D      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 3,(HL)              ; CB 9E      ; T=15    F=------ Reset bit: m=m&{~2^b}
        RES 3,A                 ; CB 9F      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 4,B                 ; CB A0      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 4,C                 ; CB A1      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 4,D                 ; CB A2      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 4,E                 ; CB A3      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 4,H                 ; CB A4      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 4,L                 ; CB A5      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 4,(HL)              ; CB A6      ; T=15    F=------ Reset bit: m=m&{~2^b}
        RES 4,A                 ; CB A7      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 5,B                 ; CB A8      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 5,C                 ; CB A9      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 5,D                 ; CB AA      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 5,E                 ; CB AB      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 5,H                 ; CB AC      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 5,L                 ; CB AD      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 5,(HL)              ; CB AE      ; T=15    F=------ Reset bit: m=m&{~2^b}
        RES 5,A                 ; CB AF      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 6,B                 ; CB B0      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 6,C                 ; CB B1      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 6,D                 ; CB B2      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 6,E                 ; CB B3      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 6,H                 ; CB B4      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 6,L                 ; CB B5      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 6,(HL)              ; CB B6      ; T=15    F=------ Reset bit: m=m&{~2^b}
        RES 6,A                 ; CB B7      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 7,B                 ; CB B8      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 7,C                 ; CB B9      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 7,D                 ; CB BA      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 7,E                 ; CB BB      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 7,H                 ; CB BC      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 7,L                 ; CB BD      ; T=8     F=------ Reset bit: m=m&{~2^b}
        RES 7,(HL)              ; CB BE      ; T=15    F=------ Reset bit: m=m&{~2^b}
        RES 7,A                 ; CB BF      ; T=8     F=------ Reset bit: m=m&{~2^b}
        SET 0,B                 ; CB C0      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 0,C                 ; CB C1      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 0,D                 ; CB C2      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 0,E                 ; CB C3      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 0,H                 ; CB C4      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 0,L                 ; CB C5      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 0,(HL)              ; CB C6      ; T=15    F=------ Set bit: m=mv{2^b}
        SET 0,A                 ; CB C7      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 1,B                 ; CB C8      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 1,C                 ; CB C9      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 1,D                 ; CB CA      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 1,E                 ; CB CB      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 1,H                 ; CB CC      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 1,L                 ; CB CD      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 1,(HL)              ; CB CE      ; T=15    F=------ Set bit: m=mv{2^b}
        SET 1,A                 ; CB CF      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 2,B                 ; CB D0      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 2,C                 ; CB D1      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 2,D                 ; CB D2      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 2,E                 ; CB D3      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 2,H                 ; CB D4      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 2,L                 ; CB D5      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 2,(HL)              ; CB D6      ; T=15    F=------ Set bit: m=mv{2^b}
        SET 2,A                 ; CB D7      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 3,B                 ; CB D8      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 3,C                 ; CB D9      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 3,D                 ; CB DA      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 3,E                 ; CB DB      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 3,H                 ; CB DC      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 3,L                 ; CB DD      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 3,(HL)              ; CB DE      ; T=15    F=------ Set bit: m=mv{2^b}
        SET 3,A                 ; CB DF      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 4,B                 ; CB E0      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 4,C                 ; CB E1      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 4,D                 ; CB E2      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 4,E                 ; CB E3      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 4,H                 ; CB E4      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 4,L                 ; CB E5      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 4,(HL)              ; CB E6      ; T=15    F=------ Set bit: m=mv{2^b}
        SET 4,A                 ; CB E7      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 5,B                 ; CB E8      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 5,C                 ; CB E9      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 5,D                 ; CB EA      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 5,E                 ; CB EB      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 5,H                 ; CB EC      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 5,L                 ; CB ED      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 5,(HL)              ; CB EE      ; T=15    F=------ Set bit: m=mv{2^b}
        SET 5,A                 ; CB EF      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 6,B                 ; CB F0      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 6,C                 ; CB F1      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 6,D                 ; CB F2      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 6,E                 ; CB F3      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 6,H                 ; CB F4      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 6,L                 ; CB F5      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 6,(HL)              ; CB F6      ; T=15    F=------ Set bit: m=mv{2^b}
        SET 6,A                 ; CB F7      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 7,B                 ; CB F8      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 7,C                 ; CB F9      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 7,D                 ; CB FA      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 7,E                 ; CB FB      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 7,H                 ; CB FC      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 7,L                 ; CB FD      ; T=8     F=------ Set bit: m=mv{2^b}
        SET 7,(HL)              ; CB FE      ; T=15    F=------ Set bit: m=mv{2^b}
        SET 7,A                 ; CB FF      ; T=8     F=------ Set bit: m=mv{2^b}
        CALL Z,NN               ; CC XX XX   ; T=17/10 F=------ Conditional hívás: If cc CALL
        CALL NN                 ; CD XX XX   ; T=17    F=------ Unconditional hívás: -[SP]=PC,PC=nn
        ADC A,N                 ; CE XX      ; T=7     F=***V0* Add with Carry: A=A+s+CY
        RST 8H                  ; CF         ; T=11    F=------ restart: (p=0H,8H,10H,...,38H)
        RET NC                  ; D0         ; T=11/5  F=------ Conditional visszatérés: If cc RET
        POP DE                  ; D1         ; T=10    F=------ pop: register=[SP]+
        JP NC,$+3               ; D2         ; T=10    F=------ feltételes ugrás: If cc JP
        OUT (N),A               ; D3 XX      ; T=11    F=------ kimenet: [port]=r
        CALL NC,NN              ; D4 XX XX   ; T=17/10 F=------ Conditional hívás: If cc CALL
        CALL NC,NN              ; D4 XX XX   ; T=17/10 F=------ Conditional hívás: If cc CALL
        PUSH DE                 ; D5         ; T=11    F=------ push: -[SP]=register
        SUB N                   ; D6 XX      ; T=7     F=***V1* kivonás: A=A-s
        RST 10H                 ; D7         ; T=11    F=------ restart: (p=0H,8H,10H,...,38H)
        RET C                   ; D8         ; T=11/5  F=------ Conditional visszatérés: If cc RET
        EXX                     ; D9         ; T=4     F=------ csere: qq<->qq' (except AF)
        JP C,$+3                ; DA         ; T=10    F=------ feltételes ugrás: If cc JP
        IN A,(N)                ; DB XX      ; T=11    F=------ bemenet: A=[n]
        CALL C,NN               ; DC XX XX   ; T=17/10 F=------ Conditional hívás: If cc CALL
        ADD IX,BC               ; DD 09      ; T=15    F=--?-0* összeadás: IX=IX+pp
        ADD IX,DE               ; DD 19      ; T=15    F=--?-0* összeadás: IX=IX+pp
        LD IX,NN                ; DD 21 XX XX; T=14    F=------ betöltés: cél=forrás
        LD (NN),IX              ; DD 22 XX XX; T=20    F=------ betöltés: cél=forrás
        INC IX                  ; DD 23      ; T=10    F=------ növelés: xx=xx+1
        ADD IX,IX               ; DD 29      ; T=15    F=--?-0* összeadás: IX=IX+pp
        LD IX,(NN)              ; DD 2A XX XX; T=20    F=------ betöltés: cél=forrás
        DEC IX                  ; DD 2B      ; T=10    F=------ csökkentés: xx=xx-1
        INC (IX+N)              ; DD 34 XX   ; T=23    F=***V0- növelés: s=s+1
        DEC (IX+N)              ; DD 35 XX   ; T=23    F=***V1- csökkentés: s=s-1
        LD (IX+N),N             ; DD 36 XX XX; T=19    F=------ betöltés: cél=forrás
        ADD IX,SP               ; DD 39      ; T=15    F=--?-0* összeadás: IX=IX+pp
        LD B,(IX+N)             ; DD 46 XX   ; T=19    F=------ betöltés: cél=forrás
        LD C,(IX+N)             ; DD 4E XX   ; T=19    F=------ betöltés: cél=forrás
        LD D,(IX+N)             ; DD 56 XX   ; T=19    F=------ betöltés: cél=forrás
        LD E,(IX+N)             ; DD 5E XX   ; T=19    F=------ betöltés: cél=forrás
        LD H,(IX+N)             ; DD 66 XX   ; T=19    F=------ betöltés: cél=forrás
        LD L,(IX+N)             ; DD 6E XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IX+N),B             ; DD 70 XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IX+N),C             ; DD 71 XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IX+N),D             ; DD 72 XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IX+N),E             ; DD 73 XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IX+N),H             ; DD 74 XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IX+N),L             ; DD 75 XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IX+N),A             ; DD 77 XX   ; T=19    F=------ betöltés: cél=forrás
        LD A,(IX+N)             ; DD 7E XX   ; T=19    F=------ betöltés: cél=forrás
        ADD A,(IX+N)            ; DD 86 XX   ; T=19    F=***V0* összeadás: A=A+s
        ADC A,(IX+N)            ; DD 8E XX   ; T=19    F=***V0* Add with Carry: A=A+s+CY
        SUB (IX+N)              ; DD 96 XX   ; T=19    F=***V1* kivonás: A=A-s
        SBC A,(IX+N)            ; DD 9E XX   ; T=19    F=***V1* Subtract with Carry: A=A-s-CY
        AND (IX+N)              ; DD A6 XX   ; T=19    F=***P00 Logical AND: A=A&s
        XOR (IX+N)              ; DD AE XX   ; T=19    F=***P00 Logical Exclusive OR: A=Axs
        OR (IX+N)               ; DD B6 XX   ; T=19    F=***P00 Logical inclusive OR: A=Avs
        CP (IX+N)               ; DD BE XX   ; T=19    F=***V1* összehasonlítás: A-s
        RLC (IX+N)              ; DD CB XX 06; T=23    F=**0P0* körkörös balra forgatás: m=m<-
        RRC (IX+N)              ; DD CB XX 0E; T=23    F=**0P0* körkörös jobbra forgatás: m=->m
        RL (IX+N)               ; DD CB XX 16; T=23    F=**0P0* forgatás balra: m={CY,m}<-
        RR (IX+N)               ; DD CB XX 1E; T=23    F=**0P0* forgatás jobbra: m=->{CY,m}
        SLA (IX+N)              ; DD CB XX 26; T=23    F=**0P0* aritmetikai balra shift: m=m*2
        SRA (IX+N)              ; DD CB XX 2E; T=23    F=**0P0* Shift Right Arith.: m=m/2
        BIT 0,(IX+N)            ; DD CB XX 46; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 1,(IX+N)            ; DD CB XX 4E; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 2,(IX+N)            ; DD CB XX 56; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 3,(IX+N)            ; DD CB XX 5E; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 4,(IX+N)            ; DD CB XX 66; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 5,(IX+N)            ; DD CB XX 6E; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 6,(IX+N)            ; DD CB XX 76; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 7,(IX+N)            ; DD CB XX 7E; T=20    F=?*1?0- Test Bit: m&{2^b}
        RES 0,(IX+N)            ; DD CB XX 86; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 1,(IX+N)            ; DD CB XX 8E; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 2,(IX+N)            ; DD CB XX 96; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 3,(IX+N)            ; DD CB XX 9E; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 4,(IX+N)            ; DD CB XX A6; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 5,(IX+N)            ; DD CB XX AE; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 6,(IX+N)            ; DD CB XX B6; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 7,(IX+N)            ; DD CB XX BE; T=23    F=------ Reset bit: m=m&{~2^b}
        SET 0,(IX+N)            ; DD CB XX C6; T=23    F=------ Set bit: m=mv{2^b}
        SET 1,(IX+N)            ; DD CB XX CE; T=23    F=------ Set bit: m=mv{2^b}
        SET 2,(IX+N)            ; DD CB XX D6; T=23    F=------ Set bit: m=mv{2^b}
        SET 3,(IX+N)            ; DD CB XX DE; T=23    F=------ Set bit: m=mv{2^b}
        SET 4,(IX+N)            ; DD CB XX E6; T=23    F=------ Set bit: m=mv{2^b}
        SET 5,(IX+N)            ; DD CB XX EE; T=23    F=------ Set bit: m=mv{2^b}
        SET 6,(IX+N)            ; DD CB XX F6; T=23    F=------ Set bit: m=mv{2^b}
        SET 7,(IX+N)            ; DD CB XX FE; T=23    F=------ Set bit: m=mv{2^b}
        POP IX                  ; DD E1      ; T=14    F=------ pop: register=[SP]+
        EX (SP),IX              ; DD E3      ; T=23    F=------ csere: [SP]<->register
        PUSH IX                 ; DD E5      ; T=15    F=------ push: -[SP]=register
        JP (IX)                 ; DD E9      ; T=8     F=------ feltétel nélküli ugrás: PC=[register]
        LD SP,IX                ; DD F9      ; T=10    F=------ betöltés: cél=forrás
        SBC A,N                 ; DE XX      ; T=7     F=***V1* Subtract with Carry: A=A-s-CY
        RST 18H                 ; DF         ; T=11    F=------ restart: (p=0H,8H,10H,...,38H)
        RET PO                  ; E0         ; T=11/5  F=------ Conditional visszatérés: If cc RET
        POP HL                  ; E1         ; T=10    F=------ pop: register=[SP]+
        JP PO,$+3               ; E2         ; T=10    F=------ feltételes ugrás: If cc JP
        EX (SP),HL              ; E3         ; T=19    F=------ csere: [SP]<->register
        CALL PO,NN              ; E4 XX XX   ; T=17/10 F=------ Conditional hívás: If cc CALL
        PUSH HL                 ; E5         ; T=11    F=------ push: -[SP]=register
        AND N                   ; E6 XX      ; T=7     F=***P00 Logical AND: A=A&s
        RST 20H                 ; E7         ; T=11    F=------ restart: (p=0H,8H,10H,...,38H)
        RET PE                  ; E8         ; T=11/5  F=------ Conditional visszatérés: If cc RET
        JP (HL)                 ; E9         ; T=4     F=------ feltétel nélküli ugrás: PC=[register]
        JP PE,$+3               ; EA         ; T=10    F=------ feltételes ugrás: If cc JP
        EX DE,HL                ; EB         ; T=4     F=------ csere: DE<->HL
        CALL PE,NN              ; EC XX XX   ; T=17/10 F=------ Conditional hívás: If cc CALL
        IN B,(C)                ; ED 40      ; T=12    F=***P0- bemenet: r=[C]
        OUT (C),B               ; ED 41      ; T=12    F=------ kimenet: [port]=r
        SBC HL,BC               ; ED 42      ; T=15    F=**?V1* Subtract with Carry: HL=HL-ss-CY
        LD (NN),BC              ; ED 43 XX XX; T=20    F=------ betöltés: cél=forrás
        NEG                     ; ED 44      ; T=8     F=***V1* negálás: A=-A
        RETN                    ; ED 45      ; T=14    F=------ visszatérés from NMI: PC=[SP]+
        IM 0                    ; ED 46      ; T=8     F=------ interrupt mód: (n=0,1,2)
        LD I,A                  ; ED 47      ; T=9     F=------ betöltés: cél=forrás
        IN C,(C)                ; ED 48      ; T=12    F=***P0- bemenet: r=[C]
        OUT (C),C               ; ED 49      ; T=12    F=------ kimenet: [port]=r
        ADC HL,BC               ; ED 4A      ; T=15    F=**?V0* Add with Carry: HL=HL+ss+CY
        LD BC,(NN)              ; ED 4B XX XX; T=20    F=------ betöltés: cél=forrás
        RETI                    ; ED 4D      ; T=14    F=------ visszatérés from Interrupt: PC=[SP]+
        IN D,(C)                ; ED 50      ; T=12    F=***P0- bemenet: r=[C]
        OUT (C),D               ; ED 51      ; T=12    F=------ kimenet: [port]=r
        SBC HL,DE               ; ED 52      ; T=15    F=**?V1* Subtract with Carry: HL=HL-ss-CY
        LD (NN),DE              ; ED 53 XX XX; T=20    F=------ betöltés: cél=forrás
        IM 1                    ; ED 56      ; T=8     F=------ interrupt mód: (n=0,1,2)
        LD A,I                  ; ED 57      ; T=9     F=**0*0- Load: A=i
        IN E,(C)                ; ED 58      ; T=12    F=***P0- bemenet: r=[C]
        OUT (C),E               ; ED 59      ; T=12    F=------ kimenet: [port]=r
        ADC HL,DE               ; ED 5A      ; T=15    F=**?V0* Add with Carry: HL=HL+ss+CY
        LD DE,(NN)              ; ED 5B XX XX; T=20    F=------ betöltés: cél=forrás
        IM 2                    ; ED 5E      ; T=8     F=------ interrupt mód: (n=0,1,2)
        IN H,(C)                ; ED 60      ; T=12    F=***P0- bemenet: r=[C]
        OUT (C),H               ; ED 61      ; T=12    F=------ kimenet: [port]=r
        SBC HL,HL               ; ED 62      ; T=15    F=**?V1* Subtract with Carry: HL=HL-ss-CY
        RRD                     ; ED 67      ; T=18    F=**0P0- forgatás jobbra 4 bits: {A,[HL]}=->{A,[HL]}
        IN L,(C)                ; ED 68      ; T=12    F=***P0- bemenet: r=[C]
        OUT (C),L               ; ED 69      ; T=12    F=------ kimenet: [port]=r
        ADC HL,HL               ; ED 6A      ; T=15    F=**?V0* Add with Carry: HL=HL+ss+CY
        RLD                     ; ED 6F      ; T=18    F=**0P0- forgatás balra 4 bits: {A,[HL]}={A,[HL]}<-
        SBC HL,SP               ; ED 72      ; T=15    F=**?V1* Subtract with Carry: HL=HL-ss-CY
        LD (NN),SP              ; ED 73 XX XX; T=20    F=------ betöltés: cél=forrás
        IN A,(C)                ; ED 78      ; T=12    F=***P0- bemenet: r=[C]
        OUT (C),A               ; ED 79      ; T=12    F=------ kimenet: [port]=r
        ADC HL,SP               ; ED 7A      ; T=15    F=**?V0* Add with Carry: HL=HL+ss+CY
        LD SP,(NN)              ; ED 7B XX XX; T=20    F=------ betöltés: cél=forrás
        LDI                     ; ED A0      ; T=16    F=--0*0- Load and Increment: [DE]=[HL],HL=HL+1,BC=BC-1
        CPI                     ; ED A1      ; T=16    F=****1- Compare and Increment: A-[HL],HL=HL+1,BC=BC-1
        INI                     ; ED A2      ; T=16    F=?*??1- Input and Increment: [HL]=[C],HL=HL+1,B=B-1
        OUTI                    ; ED A3      ; T=16    F=?*??1- Output and Increment: [C]=[HL],HL=HL+1,B=B-1
        LDD                     ; ED A8      ; T=16    F=--0*0- Load and Decrement: [DE]=[HL],HL=HL-1,BC=BC-1
        CPD                     ; ED A9      ; T=16    F=****1- Compare and Decrement: A-[HL],HL=HL-1,BC=BC-1
        IND                     ; ED AA      ; T=16    F=?*??1- Input and Decrement: [HL]=[C],HL=HL-1,B=B-1
        OUTD                    ; ED AB      ; T=16    F=?*??1- Output and Decrement: [C]=[HL],HL=HL-1,B=B-1
        LDIR                    ; ED B0      ; T=21/16 F=--000- Load, Inc., Repeat: LDI till BC=0
        CPIR                    ; ED B1      ; T=21/16 F=****1- Compare, Inc., Repeat: CPI till A=[HL]or BC=0
        INIR                    ; ED B2      ; T=21/16 F=?1??1- Input, Inc., Repeat: INI till B=0
        OTIR                    ; ED B3      ; T=21/16 F=?1??1- Output, Inc., Repeat: OUTI till B=0
        LDDR                    ; ED B8      ; T=21/16 F=--000- Load, Dec., Repeat: LDD till BC=0
        CPDR                    ; ED B9      ; T=21/16 F=****1- Compare, Dec., Repeat: CPD till A=[HL]or BC=0
        INDR                    ; ED BA      ; T=21/16 F=?1??1- Input, Dec., Repeat: IND till B=0
        OTDR                    ; ED BB      ; T=21/16 F=?1??1- Output, Dec., Repeat: OUTD till B=0
        XOR N                   ; EE XX      ; T=7     F=***P00 Logical Exclusive OR: A=Axs
        RST 28H                 ; EF         ; T=11    F=------ restart: (p=0H,8H,10H,...,38H)
        RET P                   ; F0         ; T=11/5  F=------ Conditional visszatérés: If cc RET
        POP AF                  ; F1         ; T=10    F=------ pop: register=[SP]+
        JP P,$+3                ; F2         ; T=10    F=------ feltételes ugrás: If cc JP
        DI                      ; F3         ; T=4     F=------ Disable Interrupts
        CALL P,NN               ; F4 XX XX   ; T=17/10 F=------ Conditional hívás: If cc CALL
        PUSH AF                 ; F5         ; T=11    F=------ push: -[SP]=register
        OR N                    ; F6 XX      ; T=7     F=***P00 Logical inclusive OR: A=Avs
        RST 30H                 ; F7         ; T=11    F=------ restart: (p=0H,8H,10H,...,38H)
        RET M                   ; F8         ; T=11/5  F=------ Conditional visszatérés: If cc RET
        LD SP,HL                ; F9         ; T=6     F=------ betöltés: cél=forrás
        JP M,$+3                ; FA         ; T=10    F=------ feltételes ugrás: If cc JP
        EI                      ; FB         ; T=4     F=------ Enable Interrupts
        CALL M,NN               ; FC XX XX   ; T=17/10 F=------ Conditional hívás: If cc CALL
        ADD IY,BC               ; FD 09      ; T=15    F=--?-0* összeadás: IY=IY+rr
        ADD IY,DE               ; FD 19      ; T=15    F=--?-0* összeadás: IY=IY+rr
        LD IY,NN                ; FD 21 XX XX; T=14    F=------ betöltés: cél=forrás
        LD (NN),IY              ; FD 22 XX XX; T=20    F=------ betöltés: cél=forrás
        INC IY                  ; FD 23      ; T=10    F=------ növelés: xx=xx+1
        ADD IY,IY               ; FD 29      ; T=15    F=--?-0* összeadás: IY=IY+rr
        LD IY,(NN)              ; FD 2A XX XX; T=20    F=------ betöltés: cél=forrás
        DEC IY                  ; FD 2B      ; T=10    F=------ csökkentés: xx=xx-1
        INC (IY+N)              ; FD 34 XX   ; T=23    F=***V0- növelés: s=s+1
        DEC (IY+N)              ; FD 35 XX   ; T=23    F=***V1- csökkentés: s=s-1
        LD (IY+N),N             ; FD 36 XX XX; T=19    F=------ betöltés: cél=forrás
        ADD IY,SP               ; FD 39      ; T=15    F=--?-0* összeadás: IY=IY+rr
        LD B,(IY+N)             ; FD 46 XX   ; T=19    F=------ betöltés: cél=forrás
        LD C,(IY+N)             ; FD 4E XX   ; T=19    F=------ betöltés: cél=forrás
        LD D,(IY+N)             ; FD 56 XX   ; T=19    F=------ betöltés: cél=forrás
        LD E,(IY+N)             ; FD 5E XX   ; T=19    F=------ betöltés: cél=forrás
        LD H,(IY+N)             ; FD 66 XX   ; T=19    F=------ betöltés: cél=forrás
        LD L,(IY+N)             ; FD 6E XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IY+N),B             ; FD 70 XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IY+N),C             ; FD 71 XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IY+N),D             ; FD 72 XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IY+N),E             ; FD 73 XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IY+N),H             ; FD 74 XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IY+N),L             ; FD 75 XX   ; T=19    F=------ betöltés: cél=forrás
        LD (IY+N),A             ; FD 77 XX   ; T=19    F=------ betöltés: cél=forrás
        LD A,(IY+N)             ; FD 7E XX   ; T=19    F=------ betöltés: cél=forrás
        ADD A,(IY+N)            ; FD 86 XX   ; T=19    F=***V0* összeadás: A=A+s
        ADC A,(IY+N)            ; FD 8E XX   ; T=19    F=***V0* Add with Carry: A=A+s+CY
        SUB (IY+N)              ; FD 96 XX   ; T=19    F=***V1* kivonás: A=A-s
        SBC A,(IY+N)            ; FD 9E XX   ; T=19    F=***V1* Subtract with Carry: A=A-s-CY
        AND (IY+N)              ; FD A6 XX   ; T=19    F=***P00 Logical AND: A=A&s
        XOR (IY+N)              ; FD AE XX   ; T=19    F=***P00 Logical Exclusive OR: A=Axs
        OR (IY+N)               ; FD B6 XX   ; T=19    F=***P00 Logical inclusive OR: A=Avs
        CP (IY+N)               ; FD BE XX   ; T=19    F=***V1* összehasonlítás: A-s
        RLC (IY+N)              ; FD CB XX 06; T=23    F=**0P0* körkörös balra forgatás: m=m<-
        RRC (IY+N)              ; FD CB XX 0E; T=23    F=**0P0* körkörös jobbra forgatás: m=->m
        RL (IY+N)               ; FD CB XX 16; T=23    F=**0P0* forgatás balra: m={CY,m}<-
        RR (IY+N)               ; FD CB XX 1E; T=23    F=**0P0* forgatás jobbra: m=->{CY,m}
        SLA (IY+N)              ; FD CB XX 26; T=23    F=**0P0* aritmetikai balra shift: m=m*2
        SRA (IY+N)              ; FD CB XX 2E; T=23    F=**0P0* Shift Right Arith.: m=m/2
        BIT 0,(IY+N)            ; FD CB XX 46; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 1,(IY+N)            ; FD CB XX 4E; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 2,(IY+N)            ; FD CB XX 56; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 3,(IY+N)            ; FD CB XX 5E; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 4,(IY+N)            ; FD CB XX 66; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 5,(IY+N)            ; FD CB XX 6E; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 6,(IY+N)            ; FD CB XX 76; T=20    F=?*1?0- Test Bit: m&{2^b}
        BIT 7,(IY+N)            ; FD CB XX 7E; T=20    F=?*1?0- Test Bit: m&{2^b}
        RES 0,(IY+N)            ; FD CB XX 86; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 1,(IY+N)            ; FD CB XX 8E; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 2,(IY+N)            ; FD CB XX 96; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 3,(IY+N)            ; FD CB XX 9E; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 4,(IY+N)            ; FD CB XX A6; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 5,(IY+N)            ; FD CB XX AE; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 6,(IY+N)            ; FD CB XX B6; T=23    F=------ Reset bit: m=m&{~2^b}
        RES 7,(IY+N)            ; FD CB XX BE; T=23    F=------ Reset bit: m=m&{~2^b}
        SET 0,(IY+N)            ; FD CB XX C6; T=23    F=------ Set bit: m=mv{2^b}
        SET 1,(IY+N)            ; FD CB XX CE; T=23    F=------ Set bit: m=mv{2^b}
        SET 2,(IY+N)            ; FD CB XX D6; T=23    F=------ Set bit: m=mv{2^b}
        SET 3,(IY+N)            ; FD CB XX DE; T=23    F=------ Set bit: m=mv{2^b}
        SET 4,(IY+N)            ; FD CB XX E6; T=23    F=------ Set bit: m=mv{2^b}
        SET 5,(IY+N)            ; FD CB XX EE; T=23    F=------ Set bit: m=mv{2^b}
        SET 6,(IY+N)            ; FD CB XX F6; T=23    F=------ Set bit: m=mv{2^b}
        SET 7,(IY+N)            ; FD CB XX FE; T=23    F=------ Set bit: m=mv{2^b}
        POP IY                  ; FD E1      ; T=14    F=------ pop: register=[SP]+
        EX (SP),IY              ; FD E3      ; T=23    F=------ csere: [SP]<->register
        PUSH IY                 ; FD E5      ; T=15    F=------ push: -[SP]=register
        JP (IY)                 ; FD E9      ; T=8     F=------ feltétel nélküli ugrás: PC=[register]
        LD SP,IY                ; FD F9      ; T=10    F=------ betöltés: cél=forrás
        CP N                    ; FE XX      ; T=7     F=***V1* összehasonlítás: A-s
        RST 38H                 ; FF         ; T=11    F=------ restart: (p=0H,8H,10H,...,38H)

```
