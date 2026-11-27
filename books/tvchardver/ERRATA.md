# Errata – corrections to the printed book

The transcription in `tvchardver.md` corrects the obvious errors of the printed book listed below.
OCR errors are not listed here, only errors in the original print.

## Corrected

| pdf | printed | corrected | reason |
| --: | --- | --- | --- |
| 5 | CLOCK STRECH | CLOCK STRETCH | typo STRETCH |
| 27 | jelent-kezhet | jelentkezhet | line-break hyphen left in |
| 30 | (1. 2.2.3 pont) | (l. 2.2.3 pont) | 'l.' (lásd) misread as '1.' |
| 30 | (1. 2.2.2 pont) | (l. 2.2.2 pont) | 'l.' (lásd) misread as '1.' |
| 32 | NROM és NROMS jelek | NROM és NROM5 jelek | signal is NROM5 |
| 33 | \| NNWR4 \| | \| NWR4 \| | signal name NWR4 |
| 34 | \| NSLOT3 \| 30H-3FH \| | \| NSLOT2 \| 30H-3FH \| | 30H-3FH slot should be NSLOT2 (sequence NSLOT0,1,?,4 duplicates numbering; slots are 0..3) |
| 34 | \| NSLOT4 \| 40H-4FH \| | \| NSLOT3 \| 40H-4FH \| | 4th slot should be NSLOT3 (text says NSLOT0-3) |
| 35 | i-gényli | igényli | line-break hyphen left in |
| 39 | magneto-fonbemenetre | magnetofonbemenetre | line-break hyphen left in |
| 41 | számláńc | számlánc | stray accent |
| 42 | ezek I/O címe rendre: 70H, 71H, 72H és 73H | ezek I/O címe rendre: 60H, 61H, 62H és 63H | palette regs are at 60H-63H (NWCOL 60H-6FH) |
| 42 | a 70H címen elérhető regiszter elérhető a 74H, 78H és 7C címen is | a 60H címen elérhető regiszter elérhető a 64H, 68H és 6CH címen is | palette at 60H-6FH |
| 43 | NCRTC jel a 60H-6FH címtartomány | NCRTC jel a 70H-7FH címtartomány | CRTC is at 70H-7FH |
| 43 | a 60H-6FH tartományban a CPU két címe | a 70H-7FH tartományban a CPU két címe | CRTC is at 70H-7FH |
| 48 | a 3,15 MHz-es jelnek | a 3,125 MHz-es jelnek | clock chain is 3,125 MHz |
| 52 | 160 ms-os | 160 ns-os | half clock period = 160 ns |
| 52 | CLOCK STRECH | CLOCK STRETCH | typo STRETCH |
| 53 | A CLOCK STRECH áramkört | A CLOCK STRETCH áramkört | typo STRETCH |
| 53 | a fentebb okok miatt 0. | a fentebb | duplicated words; sentence continues on next page |
| 59 | G-Y=(0,3/0,59)+(R-Y)-(0,11/0,59)+(B-Y) | G-Y=-(0,3/0,59)*(R-Y)-(0,11/0,59)*(B-Y) | G-Y = -(0.3/0.59)(R-Y) - (0.11/0.59)(B-Y); printed signs wrong (confirmed on scan) |
| 62 | 128+4 bit | 128*4 bit | 128 pixels x 4 bits = 512 bits = 64 bytes (confirmed on scan) |
| 65 | a zöls, sárga | a zöld, sárga | typo |
| 67 | bitlerendezés | bitelrendezés | typo |
| 69 | (coloumn) | (column) | English spelling |
| 69 | coloumn address strobe | column address strobe | English spelling |
| 69 | tartalmánaak | tartalmának | typo |
| 69 | vízszintesen 8 rasztersorból áll | vízszintesen 8 raszterpontból áll | 8 raster points (pixels) per character horizontally |
| 70 | inicializásánál | inicializálásánál | typo |
| 71 | I-G-R-G jele érvényesüljön | I-G-R-B jele érvényesüljön | border colour is I-G-R-B everywhere else (typo G for B) |
| 74 | amplitúdó mooduláció | amplitúdó moduláció | typo |
| 76 | filp-flop-ok | flip-flop-ok | typo |
| 84 | Az 1Y3 engedélyező jelként az N2G | Az 1Y2 engedélyező jelként az N2G | 1Y3 (A13=1) is NROM5; the N2G enable (A13=0 case) comes from 1Y2, as the text itself says A13=0 -> 1Y2 |
| 84 | az EXPSEL0 és EXPSEL0 jelek | az EXPSEL0 és EXPSEL1 jelek | print has EXPSEL1 |
| 85 | NBMEQ | NBMREQ | signal name typo |
| 88 | impuzust | impulzust | typo |
| 89 | A 27332 24 | A 2732 24 | 2732 is the 24-pin EPROM; 27332 is a typo |
| 90 | a 2732 IC 34-es lába Vcc | a 2732 IC 24-es lába Vcc | 24-pin chip has no pin 34; Vcc is pin 24 (heading says 24-26) |
| 91 | pozícicójú | pozíciójú | typo |
| 92 | 64k+1 bites | 64k\*1 bites | 64k*1 bit chips (4164), not +1 |
| 92 | a 2Y0-2Y3 bemenetek | a 2C0-2C3 bemenetek | 74LS153 section 2 data inputs are 2C0-2C3 (2Y is the output) |
| 93 | Aa DEB | A DEB | doubled letter typo |
| 107 | CLOCK STRECH | CLOCK STRETCH | typo |
| 113 | D3-6 pedig a hang amplitúdó | D2-5 pedig a hang amplitúdó | per I/O table 06H amplitude is bits 5-2, bit 6 unused |
| 115 | Y=0,03 | Y=0,30 | PAL luminance Y=0.30R+0.59G+0.11B |
| 116 | 5,6±0,1 ms-mal | 5,6±0,1 µs-mal | burst timing is microseconds |
| 116 | 2,26±0,23 ms | 2,26±0,23 µs | burst length is microseconds |
| 121 | kapacitive | kapacitívan | typo |
| 122 | Ezenk kívül | Ezen kívül | typo |
| 126 | 72-7FH (114-119) | 72-7FH (114-127) | 0x7F=127 |
| 126 | 80-9FH (128-155) | 80-9FH (128-159) | 0x9F=159 |
| 129 | 52343 | 57343 | DFFFH = 57343 |
| 139 | csak írható (R) | csak írható (W) | write-only should be (W) |
| 145 | kvarvezérelt | kvarcvezérelt | typo: quartz = kvarc |
| 145 | NIOREQ | NIORQ | signal name inconsistent with NIORQ used elsewhere |
| 146 | a a memóriából | a memóriából | doubled word |
| 146 | mikroporcesszor írási | mikroprocesszor írási | typo |
| 147 | NTERRUPT REQUEST | INTERRUPT REQUEST | missing letter |
| 147 | NBUSRRQ | NBUSRQ | signal name typo |
| 148 | CPU kér belső | CPU két belső | typo (two flip-flops IFF1, IFF2) |
| 148 | minkroprocesszor | mikroprocesszor | typo |
| 149 | DIREKT MEMORY ACCES | DIRECT MEMORY ACCESS | English term misspelled |
| 150 | CE reg.-pár | DE reg.-pár | register pair is DE |
| 150 | mikroporcesszor belső | mikroprocesszor belső | typo |
| 151 | főés alternatív | fő- és alternatív | missing hyphen/space |
| 158 | ZO8A | Z80A | typo |
| 158 | megszakíttási | megszakítási | typo |
| 158 | NIORQ és NMI jel | NIORQ és NM1 jel | M1 signal is NM1 in the book (NMI would be non-maskable interrupt) |
| 160 | utasítádekódoló | utasításdekódoló | typo |
| 164 | -oszlo-pok | -oszlopok | stray hyphenation |
| 176 | Vízszintes szinkronjel; HSYNC; 39. láb; kimenet: Aktív H állapotú jel a monitor függőleges | Vízszintes szinkronjel; HSYNC; 39. láb; kimenet: Aktív H állapotú jel a monitor vízszintes | HSYNC drives horizontal deflection |
| 177 | a fényceruza és a fényceruza és a fényceruza áramkörök | a fényceruza áramkörök | tripled/repeated words in the print |
| 178 | (RO-R17) közül | (R0-R17) közül | R-zero written as letter O |
| 179 | az RO regiszter értékénél | az R0 regiszter értékénél | R-zero written as letter O |
| 180 | mint RO. | mint R0. | R-zero written as letter O |
| 182 | \| V3 \| V2 \| V1 \| V0 \| Sorvégjel szélessége \| | \| V3 \| V2 \| V1 \| V0 \| Képvégjel szélessége \| | V table is for vertical sync width |
| 185 | cilushoz | ciklushoz | typo |
| 186 | RA0-RA5 | RA0-RA4 | MC6845 has RA0-RA4 |
| 188 | Címregiszter : 01010 | Címregiszter : 01000 | R8 has address 01000 |
| 191 | Az CPSTB | Az LPSTB | signal is LPSTB |
| 193 | 64 kbyte-ot | 64 byte-ot | a TV line is 64 bytes (next sentence) |
| 193 | következő három címbitjét | következő két címbitjét | RA0-RA1 are two bits (table: ma6-ma7) |
| 193 | következő öt címbitjét | következő hat címbitjét | MA6-MA11 are six bits (table: ma8-ma13) |
| 194 | szürő | szűrő | typo |
| 196 | (1. 67. ábra) | (l. 67. ábra) | 'l.' = lásd, digit one misread |
| 197 | freed-forward | feed-forward | typo |
| 208 | ![18. ábra A CLOCK STRECH | ![18. ábra A CLOCK STRETCH | typo |
| 208 | 18. ábra A CLOCK STRECH áramkör | 18. ábra A CLOCK STRETCH áramkör | typo STRETCH (caption) |
| 209 | ![21. ábra A CLOCK STRECH | ![21. ábra A CLOCK STRETCH | typo |
| 209 | 21. ábra A CLOCK STRECH áramkör | 21. ábra A CLOCK STRETCH áramkör | typo STRETCH (caption) |
| 210 | ![22. ábra A CLOCK STRECH | ![22. ábra A CLOCK STRETCH | typo |
| 210 | 22. ábra A CLOCK STRECH áramkör | 22. ábra A CLOCK STRETCH áramkör | typo STRETCH (caption) |
| 228 | (DE) ← A • • • • • • 00 110 010 | (DE) ← A • • • • • • 00 010 010 | LD (DE),A opcode is 00 010 010; 00 110 010 is LD (nn),A |
| 232 | HL ← HL+1 BC ← BC+1 | HL ← HL+1 BC ← BC-1 | LDI decrements BC |
| 232 | 21 16 If BC = 0 If BC = 0 | 21 16 If BC ≠ 0 If BC = 0 | LDIR repeats (21 T) when BC ≠ 0; printed = (same on scan) |
| 232 | • 1 1 4 Regisztertömb | • 11 011 001 1 1 4 Regisztertömb | EXX opcode missing in the print (Zilog: D9H = 11 011 001) |
| 233 | 21 16 If BC = 0 If BC = 0 | 21 16 If BC ≠ 0 If BC = 0 | LDDR repeats (21 T) when BC ≠ 0 |
| 233 | If BC =0 és A = (HL) If BC = 0 vagy A = (HL) | If BC ≠ 0 és A ≠ (HL) If BC = 0 vagy A = (HL) | CPIR 21 T case is BC ≠ 0 and A ≠ (HL) |
| 233 | CPI A ← (HL) | CPI A - (HL) | Compare instructions do A - (HL) (A is not loaded) |
| 233 | CPIR A ← (HL) | CPIR A - (HL) | Compare instructions do A - (HL) (A is not loaded) |
| 233 | CPD A ← (HL) | CPD A - (HL) | Compare instructions do A - (HL) (A is not loaded) |
| 233 | ↕ ↕ ↕ 11 101 101 10 100 001 | ↕ 1 ↕ 11 101 101 10 100 001 | CPI/CPIR/CPD/CPDR set N=1 (compare is a subtraction); book shows ↕ for N |
| 233 | ↕ ↕ ↕ 11 101 101 10 110 001 | ↕ 1 ↕ 11 101 101 10 110 001 | CPI/CPIR/CPD/CPDR set N=1 (compare is a subtraction); book shows ↕ for N |
| 233 | ↕ ↕ ↕ 11 101 101 10 101 001 | ↕ 1 ↕ 11 101 101 10 101 001 | CPI/CPIR/CPD/CPDR set N=1 (compare is a subtraction); book shows ↕ for N |
| 234 | If BC = 0 és A = (HL) | If BC ≠ 0 és A ≠ (HL) | CPDR 21 T case is BC ≠ 0 and A ≠ (HL) |
| 234 | CPDR A ← (HL) | CPDR A - (HL) | Compare instructions do A - (HL) (A is not loaded) |
| 234 | ↕ ↕ ↕ 11 101 101 10 111 001 | ↕ 1 ↕ 11 101 101 10 111 001 | CPI/CPIR/CPD/CPDR set N=1 (compare is a subtraction); book shows ↕ for N |
| 240 | RLD • ↕ P ↕ 0 0 11 101 101 | RLD • ↕ P ↕ 0 0 11 101 101 01 101 111 | Second opcode byte of RLD (ED 6F = 01 101 111) omitted in the book; 2-byte instruction listed with one opcode line |
| 243 | pakolt BDC kóddá | pakolt BCD kóddá | BCD misspelled as BDC |
| 243 | pakolt BDC operandusú | pakolt BCD operandusú | BCD misspelled as BDC |
| 249 | OUT (C),r (c) ← r | OUT (C),r (C) ← r | Register C written in lowercase; Zilog: (C) ← r |
| 250 | 10 101 011 2 5 21 | 10 101 011 2 4 16 | OUTD is 4 M-cycles / 16 T-states per Zilog (same as OUTI); 5/21 applies only to repeating OTDR |

## Possible errors, left as printed

| pdf | printed | suggestion | reason |
| --: | --- | --- | --- |
| 30 | a 0FH I/O címre történő íráskor | a 0CH I/O címre történő íráskor | register address is 0CH (0CH-0FH per later text) |
| 36 | Az NSLOT-3 jelek | Az NSLOT0-3 jelek | range |
| 36 | a palettaregiszter egyes belső regiszterei minden második címen, összesen nyolcszor jelentkeznek | a palettaregiszter egyes belső regiszterei minden negyedik címen, összesen négyszer, a CRTC regiszterei minden második címen, összesen nyolc… | palette uses 2 address bits so repeats 4 times in 16 addresses |
| 39 | A regisztert az NWR5 jel (05H írás) tölti | A regisztert az NWR6 jel (06H írás) tölti | motor control register is written by NWR6 (06H) per I/O table |
| 51 | il. az annak | ill. az annak | abbreviation |
| 52 | különkülön | külön-külön | typo |
| 54 | Amikor a mindhárom | Amikor mindhárom | extra article |
| 55 | azaz 50 teljes kép | azaz 25 teljes kép | 25+25 half-frames per second = 25 full frames (interlaced) |
| 85 | E12/5 (I12/5) | E12/5 (J12/5) | HBA-2 position is J12/5 elsewhere (E12/5 (J12/5) on previous page) |
| 85 | (I17/1 kapu | (J17/1 kapu | HBA-2 gate is J17/1 elsewhere |
| 91 | a 04 pozíciójú | a D4 pozíciójú | position letters here are D3-D6; print shows 04 for D4 (likely typo) |
| 93 | J18/2 és J18/6 | J18/3 és J18/6 | J18/3 is gate output (pin 3); other page (pdf 103) says J18/3 |
| 95 | Az E7/7 flip-flop | Az F7/7 flip-flop | the WAIT flip-flop was F7/7 two paragraphs earlier |
| 96 | El(F1) | E1(F1) | position E1 (as on next page); l/1 confusion |
| 96 | F11/6(G11/6) | F11/6(G2/6) | HBA-2 position of that gate is G2/6 (see 'F11/6-8(G2/6-8)' above) |
| 113 | a 3-7 bitek értéke | a 2-7 bitek értéke | amplitude bits start at B2 per table |
| 113 | SN744273 | SN74273 | 8-bit register 74273, extra 4 |
| 125 | 50-57H (81-87) | 50-57H (80-87) | 0x50=80 |
| 136 | soros vonatl | soros vonali | typo |
| 137 | 1 0 1 1 Lila 45 69 8A 138 | 1 0 1 1 Lila 45 69 8A 138 1 1 0 0 Zöld 50 80 A0 160 | colour table skips IGRB 1100 (bright green) |
| 145 | 5 db kifelé | 6 db kifelé | six signals are listed (M1, MREQ, IORQ, RD, WR, RFSH) |
| 151 | Beviteli (NP) | Beviteli (IN) | Z80 input instruction is IN |
| 158 | A megszakításkérés tiltása után | A megszakításkérés elfogadása után | interrupt acknowledge occurs after acceptance, not after disabling |
| 185 | vízszintes irányban való növelésére | függőleges irányban való növelésére | interlace doubles the number of TV lines, i.e. vertical resolution |
| 195 | stabil multivibrátor | astabil multivibrátor | switching oscillator is an astable multivibrator |
| 197 | tárcsatranzisztor | tárcsatermisztor | TH1 is a thermistor (NTC disc) |
| 198 | (8,5 V) | (8,4 V) | pin 2 reference is stated as 8,4 V on pdf 196 |
