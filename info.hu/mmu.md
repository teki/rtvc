# Memóriakezelő egység (MMU)

Ez a dokumentum nyelvfüggetlen architekturális útmutató a Videoton TV Computer (TVC) emulátor memóriakezelő egységéhez. A Rust implementáció alapja: [src/mmu.rs](../src/mmu.rs).

## Áttekintés

A TVC Z80 CPU-ja 64 KB-os, 16 bites címtartományt lát. A gépben ennél több fizikai memória lehet: RAM, videó RAM, rendszer-ROM és cartridge ROM. Emiatt a memória bankkapcsolással működik.

A 64 KB-os címtér négy darab 16 KB-os lapra oszlik. A programok hardver I/O portokra írással választják ki, mely fizikai bank jelenjen meg az egyes lapokon.

## Címtér és lapozás

| Lap | Címtartomány | Méret | Vezérlés |
|---|---|---:|---|
| 0 | `0x0000 - 0x3FFF` | 16 KB | fő map 3., 4. bit |
| 1 | `0x4000 - 0x7FFF` | 16 KB | fő map 2. bit, videó map 0., 1. bit |
| 2 | `0x8000 - 0xBFFF` | 16 KB | fő map 5. bit, videó map 2., 3. bit |
| 3 | `0xC000 - 0xFFFF` | 16 KB | fő map 6., 7. bit |

## Memóriabankok

Minden blokk 16 KB, kivéve az `EXT` és `EXTH` félablakokat.

- **U0-U3**: írható/olvasható RAM bankok. Az U3 bővítő RAM-ként is szerepelhet.
- **VID0-VID3**: videó RAM bankok. A VID1-VID3 csak TVC 64K+ gépen létezik.
- **SYS**: rendszer-ROM, benne OS és BASIC.
- **CART**: cartridge ROM.
- **EXT** (`0xC000-0xDFFF`): alsó 8 KB bővítőeszköz-ablak.
- **EXTH** (`0xE000-0xFFFF`): felső 8 KB bővítő-ROM, például DOS ROM.

## Fő lapozóregiszter

Legyen `M` a fő memórialapozó portra írt bájt.

### 0. lap (`0x0000 - 0x3FFF`)

`M & 0x18`:

- `0x00`: **SYS**
- `0x08`: **CART**
- `0x10`: **U0**
- `0x18`: **U3** TVC 64K+ gépen, különben **U0**

### 1. lap (`0x4000 - 0x7FFF`)

`M & 0x04`:

- `0x04`: TVC 64K+ gépen videó RAM bank, a videó lapozóregiszter szerint.
- `0x00`: **U1**

### 2. lap (`0x8000 - 0xBFFF`)

`M & 0x20`:

- `0x20`: **U2**
- `0x00`: videó RAM. Alap TVC-n mindig **VID0**, TVC 64K+ gépen a videó lapozóregiszter választ.

### 3. lap (`0xC000 - 0xFFFF`)

`M & 0xC0`:

- `0x00`: **CART**
- `0x40`: **SYS**
- `0x80`: **U3**
- `0xC0`: **EXT/EXTH** bővítőtér

## Videó lapozóregiszter (TVC 64K+)

Legyen `V` a videó lapozóportra írt bájt. Csak TVC 64K+ gépen van hatása.

- 1. lap videóbankja: `V & 0x03` választja a VID0-VID3 bankot.
- 2. lap videóbankja: `V & 0x0C` választja a VID0-VID3 bankot.
- A CRTC által megjelenített aktív videóbank: `V & 0x30`.

## Olvasási és írási szemantika

A Rust `TvcMmu` 8 bites belső memóriaolvasást és -írást ad (`r8`, `w8`). A teljes Z80 címtérért, beleértve az I/O-t és a bővítőkártyákat, a [src/bus.rs](../src/bus.rs) `CpuBus` traitje és a `TvcBus` felel.

CPU cím elérésekor:

1. `page_index = address >> 14`
2. `offset = address & 0x3FFF`
3. Az aktuális map alapján ki kell választani az aktív bankot.

Írásnál RAM és videó RAM módosul, ROM írás ignorálódik. `EXT` esetén a 3. lap alsó fele a bővítőkártyához kerül, a felső fele `EXTH` ROM.

Olvasásnál RAM/ROM közvetlenül olvasható. `EXT` alsó fél a kártyától olvas, kártya nélkül `0xFF`; felső fél az `EXTH` ROM.

## 16 bites hozzáférés

A Z80 little-endian:

- `r16(address) = r8(address) | (r8(address + 1) << 8)`
- `w16(address, value)` előbb az alsó, majd a felső bájtot írja.
- `w16reverse(address, value)` speciális utasításokhoz előbb a felső, majd az alsó bájtot írja.

## Implementációs stratégia

- Ne másolj memóriát lapozáskor; tarts négy mutatót/referenciát az aktuálisan látható bankokra.
- Portíráskor frissítsd ezt a négy bankreferenciát.
- Az `r8`/`w8` forró útvonal: a lapindex számítása és a bufferhozzáférés legyen nagyon olcsó.
- Az induló map értéke legyen olyan őrszemérték, amely biztosan különbözik az első valódi beállítástól.
- Az `EXT` alsó 8 KB-os kivételét olcsó segédfüggvénnyel érdemes felismerni.
