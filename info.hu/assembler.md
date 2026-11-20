# rtvc assembler referencia

Ez a dokumentum a [Z80 segédassemblert](../src/emulator/asm.rs), az
[`rtvc-asm`](../src/bin/rtvc_asm.rs) és
[`rtvc-disasm`](../src/bin/rtvc_disasm.rs) parancssori programokat, valamint a
[`rtvc_debug.py`](../scripts/rtvc_debug.py) hibakereső betöltési folyamatát
ismerteti.

Az assembler szándékosan kicsi. Hibakereső javításokhoz, TVC
segédrutinokhoz és Spectrum-portolási illesztésekhez készült; nem teljes értékű
makróassembler.

## Belépési pontok

### Rust API

Az `asm.rs` a következő belépési pontokat adja:

- az `assemble_line(source, pc)` egyetlen utasítást vagy `DB`/`DEFB` sort
  fordít le a megadott programszámlálónál;
- az `assemble_program(source, origin)` lefuttatja a kétmenetes assemblert, és
  visszaadja a szegmenseket, szimbólumokat, forrássor-metaadatokat, az
  összefűzött bájtokat és a `next_addr` értéket; fájlokhoz nem fér hozzá;
- az `assemble_program_with(source, origin, name, files)` ugyanezt teszi a
  `name` nevű forrással, és az `INCLUDE`, `INCBIN`, `IMPORT` fájlokat egy
  `AsmFiles` megvalósításon keresztül olvassa. Maga az assembler sosem nyúl
  a fájlrendszerhez, így a hibakeresőben és WebAssembly buildben is fut.

A TCP hibakereső `assemble` parancsa az `assemble_program` függvényt használja,
így egy kérés egy utasítást vagy egy kis forrásblokkot is tartalmazhat.

### Parancssori assembler

Segédforrás fordítása TOML formátumba:

```cmd
rtvc-asm --origin 8000H helper.asm -o helper.toml
```

A `-` bemeneti útvonal a szabványos bemenetről olvas. A `-o` elhagyásakor a
kimenet a szabványos kimenetre kerül.

| Opció | Jelentés |
| --- | --- |
| `--origin <cím>` | Kezdő fordítási cím a forrásbeli `ORG` előtt; alapértéke `0`. |
| `--format <toml\|cas\|bin>` | Kimeneti formátum; alapértéke `toml`. A `cas` egy összefüggő szegmenst vár: `BASIC_START` programot, vagy bármely `ORG` címet `1A3EH` és `BFFFH` között. A `bin` egy összefüggő szegmenst vár. |
| `-d <NÉV=ÉRTÉK>`, `--define <NÉV=ÉRTÉK>` | Fordítás előtt lecseréli az assembly kódban lévő `%NÉV%` helyőrzőket. Többször is megadható; a hiányzó érték hiba. |
| `--no-breakpoints` | Kihagyja a kimenetből a [`BREAK`](#töréspontok) jelöléseket. |
| `--entry <szimbólum>` | `--format cas` esetén a címke, amelyre a másolórutin ugrik; alapértéke `START`, vagy az első bájt. |
| `-I <könyvtár>`, `--include-dir <könyvtár>` | Az `INCLUDE`, `INCBIN` és `IMPORT` fájlokat a megnevező fájl könyvtára után ebben a könyvtárban is keresi. Többször is megadható. |
| `-o <útvonal>`, `--output <útvonal>` | Kimeneti fájl. |
| `-` bemenetként | Olvasás a szabványos bemenetről. |

A `<cím>` lehet decimális, `0x` vagy `$` előtagú, illetve `H` utótagú
hexadecimális szám.

A build során meghatározott értékek `%NÉV%` helyőrzőkkel tarthatók külön az
assembly forrástól:

```asm
        LD BC,%BLOCK_SIZE%
```

```cmd
rtvc-asm -d BLOCK_SIZE=0748H helper.asm -o helper.toml
```

A definíciók neve nem érzékeny a kis- és nagybetűkre, értékük szövegesen kerül
behelyettesítésre. A fenti példában a `-d BLOCK_SIZE=...` elhagyása hibát ad.
A behelyettesítés csak az assembly kódra vonatkozik: a pontosvesszős
megjegyzésekben és az idézett szövegliterálokban lévő helyőrzők változatlanok
maradnak. Az idézett szövegen belüli pontosvessző szintén nem kezd megjegyzést.

### Parancssori disassembler

Bináris adat visszafordítása `rtvc-asm` forrássá:

```cmd
rtvc-disasm --origin C000H roms\TVC12_D4.64K -o rom.asm
```

ROM-szimbólumok és adatterületek is megadhatók, hogy az ismert táblák `DB`
sorokként, a kód pedig utasításokként jelenjen meg:

```cmd
rtvc-disasm --origin C000H --symbols roms\TVC12_D4.64K.symbols.json --comments roms\TVC12_D4.64K.comments.json --bank sys --bank-offset 0000H --data-range C003H-C228H roms\TVC12_D4.64K -o rom.asm
```

| Opció | Jelentés |
| --- | --- |
| `--origin <cím>` | Az első bemeneti bájt CPU-címe; alapértéke `0`. |
| `-o <útvonal>`, `--output <útvonal>` | Assembly kimeneti fájl. |
| `--title <szöveg>` | Címmegjegyzés hozzáadása. |
| `--symbols <útvonal>` | ROM-címkék és megjegyzések betöltése JSON-ból. |
| `--comments <útvonal>` | Címhez rendelt megjegyzések; többször is megadható. |
| `--bank <név>` | Szimbólumbank, például `sys` vagy `exth`; a `--symbols` mellett kötelező. |
| `--bank-offset <cím>` | Az első bemeneti bájtnak megfelelő fizikai bankeltolás. |
| `--data-range <kezdet-vég>` | Inkluzív CPU-címtartomány kiírása `DB` sorokként; ismételhető. |
| `-` bemenetként | Bináris adatok olvasása a szabványos bemenetről. |

Az `rtvc-disasm` az emulátor saját Z80 disassemblerét használja, és minden
kiírt utasítást visszaellenőriz az `assemble_line()` függvénnyel. A nem
támogatott vagy bemeneti határt átlépő alakok `DB` sorokká válnak, ezért a
generált forrás bájtpontosan újrafordítható. A bemeneti tartományon belüli ugrás-
és híváscélok `Lxxxx` címkét kapnak.

### Hibakereső kliens

A `scripts/rtvc_debug.py` parancsai:

```text
asm [cím]
asmfile helper.asm [kezdőcím]
loadasm helper.toml
```

- Az `asm` egyetlen interaktív javítóutasítást ír be.
- Az `asmfile` elküldi a forrást a hibakereső assemblerének, majd a
  szegmenseket a leképezett memóriába írja.
- A `loadasm` egy `rtvc-asm-v1` TOML fájl összes szegmensét tölti be.
- Mindkettő felveszi a [`BREAK`](#töréspontok) sorok töréspontjait.

## Forrásformátum

A forrás sororientált. Az idézett szövegen kívüli pontosvessző megjegyzést
kezd:

```asm
; megjegyzés
START:  LD HL,MSG   ; sorvégi megjegyzés
MSG:    DB "OK",0
```

A címkék `név:` alakúak és nem érzékenyek a kis- és nagybetűkre; tároláskor
nagybetűssé válnak. A név ASCII betűt, számjegyet, `_` és `.` karaktert
tartalmazhat, de nem kezdődhet számjeggyel.

A `.` karakterrel kezdődő címke helyi: a fölötte álló utolsó globális
címkéhez tartozik, vagyis az utolsó olyan `név:` címkéhez, amelynek neve nem
`.` karakterrel kezdődik. Tárolt neve `GLOBÁLIS.HELYI`, és a kifejezésben
írt `.helyi` ugyanezt a nevet jelenti, így minden rutinnak lehet saját
`.loop` címkéje. Máshonnan a teljes névvel hivatkozható:

```asm
CLEAR:  LD B,0
.loop:  LD (HL),A           ; CLEAR.LOOP
        INC HL
        DJNZ .loop
        RET
FILL:   LD B,C
.loop:  LD (HL),A           ; FILL.LOOP, egy másik címke
        ...
        JP CLEAR.loop
```

- Az `EQU` konstans nem kezd új hatókört; a `.név EQU érték` helyi
  konstanst definiál.
- A fordító a `.név` alakot szövegekben, karakterkonstansokban és
  megjegyzésekben nem cseréli. A listák a `.név` alakot mutatják, a
  hibaüzenetek és a szimbólumok a teljes nevet.
- Az első globális címke előtti helyi címke `.név` alakban marad.

| Direktíva | Alak | Megjegyzés |
| --- | --- | --- |
| `ORG` | `ORG kifejezés[, név, leképezett-cím ...]` | Beállítja az aktuális címet; a név/cím párok tartós címleképezéseket definiálnak erről az eredetről. Több `ORG` több kimeneti szegmenst hoz létre. |
| `BASIC_START` | `BASIC_START` | Tokenizált TVC BASIC autorun sort ír `19EFH` címre, `1A30H`-ig kitölt, és ott definiálja a `BASIC_START` címkét. |
| `EQU` | `CÍMKE EQU kifejezés` vagy `CÍMKE: EQU kifejezés` | Konstans szimbólumot definiál. |
| `DB`, `DEFB` | `DB érték[, érték...]` | Bájtokat, illetve ASCII szövegeket ír ki. |
| `DW`, `DEFW` | `DW érték[, érték...]` | Little-endian 16 bites szavakat ír ki. |
| `DS`, `DEFS` | `DS darab[, kitöltés]` | Nulla vagy a megadott értékű kitöltőbájtokat ír ki. |
| `REPT`, `ENDR` | `REPT darab[, számláló]` … `ENDR` | A közrezárt sorokat `darab`-szor fordítja le. Lásd: [Ismétlés](#ismétlés). |
| `ASSERT` | `ASSERT feltétel[, "üzenet"]` | Hibával leállítja a fordítást, ha a feltétel hamis. Lásd: [Ellenőrzések](#ellenőrzések). |
| `PHASE`, `DEPHASE` | `PHASE cím` … `DEPHASE` | A közrezárt kódot futási címre fordítja, a bájtok a betöltési címen maradnak. Lásd: [Fordítás futási címre](#fordítás-futási-címre). |
| `INCLUDE` | `INCLUDE "útvonal"` | Másik forrásfájlt fordít le a helyén. Lásd: [Fájlok](#fájlok-include-incbin-és-import). |
| `INCBIN` | `INCBIN "útvonal"[, eltolás[, hossz]]` | Egy fájl bájtjait írja ki. |
| `IMPORT` | `IMPORT "modul.toml"[, ELŐTAG]` | Egy másik modul TOML kimenetének szimbólumait definiálja. |
| `BREAK` | `BREAK` | A következő utasítást hibakereső-töréspontnak jelöli. Lásd: [Töréspontok](#töréspontok). |
| `STRUCT`, `ENDS` | `STRUCT név[, bázis]` … `ENDS` | Méretet és mezőeltolásokat definiál; bájtot nem ír. Lásd: [Struktúrák](#struktúrák). |

A `DB`/`DEFB` szövegliteráljai csak ASCII karaktereket tartalmazhatnak. Az
érvényes escape-ek: `\0`, `\n`, `\r`, `\t`, `\\`, `\"` és `\'`.

## Ismétlés

A `REPT darab` … `ENDR` blokk a közrezárt sorokat `darab`-szor fordítja le:

```asm
        REPT 20
        NOP
        ENDR
```

- A darabszám kifejezés, amelyet a fordító a `REPT` sor elérésekor értékel
  ki. Használhat `$`-t, `EQU` szimbólumokat és a fölötte definiált
  címkéket, a `REPT` sor saját címkéjét is. Később definiált szimbólum hiba.
- A nulla darabszám kihagyja a blokkot. A blokkok egymásba ágyazhatók.
- A `REPT darab, NÉV` alakban a számláló neve minden példányban a példány
  sorszámára cserélődik (`0`-tól `darab - 1`-ig, decimálisan). A csere
  csak egész szavakra vonatkozik, nem érzékeny a kis- és nagybetűkre, és
  kihagyja az idézőjeles szövegeket. A számláló nem szimbólum, és nem
  jelenik meg a kimenetben.
- A `REPT` sor címkéje az első példány címét, az `ENDR` sor címkéje az
  utolsó példány utáni címet jelöli. A blokkon belüli globális címke
  példányonként újra definiálódik, ezért egynél nagyobb darabszámnál
  ismételt címke hibát ad.
- A blokkon belül definiált [helyi címke](#forrásformátum) a saját
  példányához tartozik, így ciklusok is ismételhetők. Egy példányon belül a
  `.név` a példány saját címkéje; a blokkban nem definiált `.név` a
  befoglaló rutin címkéje. A példányok neve `GLOBÁLIS._Rn_c.NÉV`, ahol `n` a
  helyi címkét tartalmazó `REPT` blokkok sorszáma, `c` a példányé:

  ```asm
  WAIT3:  REPT 3
          LD B,40
  .delay: DJNZ .delay         ; WAIT3._R1_0.DELAY, WAIT3._R1_1.DELAY, ...
          ENDR
          RET
  ```
- Minden példány az eredeti forrássor számát viseli, a hibaüzenetekben és a
  TOML `[[lines]]` táblában is.
- A kifejtés összesen legfeljebb 1 048 576 sort eredményezhet.

## Struktúrák

A `STRUCT név` … `ENDS` blokk egy rekord szerkezetét írja le. Bájtot nem
ír; a `név` szimbólum a méret, a `név.MEZŐ` pedig az egyes tagok eltolása,
ami az `(IX+d)` és `(IY+d)` címzéshez illik:

```asm
        STRUCT POS
X       BYTE
Y       BYTE
        ENDS

        STRUCT SPRITE
AT      POS             ; beágyazva: SPRITE.AT, SPRITE.AT.X, SPRITE.AT.Y
FRAME   WORD
        BLOCK 2         ; névtelen kitöltés
FLAGS   BYTE
        ENDS            ; SPRITE = 7

        LD A,(IX+SPRITE.FLAGS)  ; LD A,(IX+6)
        LD DE,SPRITE            ; a méret, 7
SPRITES: DS SPRITE*8            ; nyolc sprite
```

- Egy tagsor alakja `MEZŐ FAJTA` vagy `MEZŐ: FAJTA`; mezőnév nélkül csak
  helyet foglal. A `FAJTA` lehet `BYTE` (1), `WORD` (2), `BLOCK méret`,
  vagy egy korábban definiált struktúra neve, amelynek mezői a tag alatt is
  létrejönnek (`SPRITE.AT.Y`).
- A `BLOCK méret` kifejezés a struktúra definiálásakor értékelődik ki, így
  használhatja a fölötte definiált szimbólumokat, más struktúrák méretét is.
- A `STRUCT név, bázis` alak minden mezőhöz hozzáadja a `bázis` értéket,
  rögzített elrendezésekhez, például rendszerváltozókhoz vagy I/O
  regiszterekhez: a `STRUCT VARS, 0B000H` első mezője `0B000H`. A méret nem
  változik.
- A nevek pontot tartalmazó közönséges globális szimbólumok, így nem
  ütköznek a ponttal kezdődő [helyi címkékkel](#forrásformátum). Már
  definiált struktúranév, mező vagy méret ismételt címke hibát ad.
- A blokkban csak tagsor, üres sor és megjegyzés állhat.
- A `LD A,(SPRITE.X)` a `SPRITE.X` címen lévő memóriaoperandus, nem egy
  példány mezője; lásd a zárójelekről szóló részt a
  [Kifejezések](#kifejezések) fejezetben. Helyesen `(IX+SPRITE.X)`, vagy az
  első példányra `(SPRITES+SPRITE.X)`.

## Ellenőrzések

Az `ASSERT feltétel[, "üzenet"]` hibával leállítja a fordítást, ha a
feltétel hamis:

```asm
LINE_START:
        ...
LINE_END:
        ASSERT LINE_END-LINE_START = 12, "megváltozott a másolóciklus mérete"
```

- A feltétel vagy két kifejezés egyetlen összehasonlítása, vagy egyetlen
  kifejezés, amelynek nem nullának kell lennie. Az összehasonlítások: `=`
  vagy `==`, `!=` vagy `<>`, `<`, `<=`, `>` és `>=`. A `<<` és `>>`
  léptetés, nem összehasonlítás.
- Az ellenőrzés a második menetben fut, amikor már minden szimbólum ismert,
  így a forrásban később szereplő címkékre is hivatkozhat. A `$` az
  `ASSERT` sor címe.
- A hibaüzenet tartalmazza a sort, az üzenetet (ennek hiányában a
  feltételt) és az összehasonlított értékeket, például
  `line 9: assertion failed: megváltozott a másolóciklus mérete (left is
  13, right is 12)`.

## Töréspontok

A `BREAK` a következő utasítást hibakereső-töréspontnak jelöli. Bájtot nem
ír ki:

```asm
START:  LD HL,TABLE
        BREAK               ; itt áll meg, ha a hibakeresőn át töltjük be
        LD A,(HL)
```

- A töréspont címe a `BREAK` sor futási címe, így a `PHASE` blokkon belüli
  `BREAK` ott állít meg, ahol az átmásolt kód fut.
- Minden `BREAK` szerepel a TOML `[[breakpoints]]` táblájában (`addr`,
  `line`, beillesztett fájlnál `file`), és a hibakereső `assemble`
  válaszában is.
- A hibakereső kliens `asmfile` és `loadasm` parancsa a kód beírása után
  felveszi ezeket a töréspontokat; a `bp list` kilistázza, a `bp rm`
  törli őket.
- Az `rtvc-asm --no-breakpoints` kihagyja őket, például kiadott
  buildekhez.

## Fájlok: INCLUDE, INCBIN és IMPORT

```asm
        INCLUDE "tvc-ports.inc"          ; másik forrás fordítása itt
        IMPORT "boot.toml", BOOT_        ; másik modul szimbólumai
        ORG 8000H
START:  CALL BOOT_KEY_SCAN
FONT:   INCBIN "font.bin"                ; a teljes fájl
GLYPH:  INCBIN "font.bin", 64*3, 64      ; 192-es eltolás, 64 bájt
```

- Az `INCLUDE "útvonal"` egy másik forrásfájl sorait fordítja le az
  `INCLUDE` sor helyén. Az `INCLUDE` sor címkéje a beillesztett kód címét
  jelöli. A beillesztett fájlok további fájlokat illeszthetnek be, legfeljebb
  32 mélységig; az önmagát közvetlenül vagy közvetve beillesztő fájl hiba. A
  `%NÉV%` helyőrzők a beillesztett fájlokra is vonatkoznak.
- Az `INCBIN "útvonal"[, eltolás[, hossz]]` egy fájl bájtjait írja ki. Az
  eltolás és a hossz kifejezés, amely a sor fölött definiált szimbólumokat
  és `REPT` számlálót is használhat: `REPT 4, I` … `INCBIN "font.bin",
  I*64, 8` … `ENDR`. Hossz nélkül a fájl hátralévő része kerül ki. A fájl
  végén túlnyúló tartomány hiba.
- Az `IMPORT "modul.toml"[, ELŐTAG]` egy `rtvc-asm` által írt
  `rtvc-asm-v1` TOML fájl minden szimbólumát konstansként definiálja.
  Előtaggal a `KEY` név `ELŐTAGKEY` lesz, így a `BOOT_` előtag `BOOT_KEY`
  nevet ad. A szimbólumok az első menetben az `IMPORT` sor alatt, a
  másodikban mindenhol használhatók. A már definiált név hiba. Az importált
  szimbólumok nem kerülnek a modul saját `[symbols]` táblájába.
- A relatív útvonalat a fordító az őt megnevező fájl mellett, majd sorban
  minden `-I` könyvtárban keresi. Az abszolút útvonal változatlanul
  használatos.
- A beillesztett fájlok a TOML `files` tömbjében szerepelnek, soraikat a
  `[[lines]]` `file` mezője jelöli; a bennük talált hibák az adott fájlt
  nevezik meg: `tvc-ports.inc:12: unknown symbol 'PORT'`.
- A hibakereső `assemble` parancsa nem fér hozzá fájlokhoz, ezért ott ezek
  a direktívák hibát adnak.

## Fordítás futási címre

A futás előtt átmásolt kód a futási címére fordítható, miközben a bájtjai a
betöltési címen maradnak. A blokkot a `PHASE cím` kezdi és a `DEPHASE`
zárja:

```asm
        ORG 8000H
COPY:   LD HL,SUPPORT_LOAD
        LD DE,SUPPORT_RUN
        LD BC,SUPPORT_LEN
        LDIR
        JP SUPPORT_RUN

SUPPORT_LOAD:
        PHASE 21BCH
SUPPORT_RUN:
        LD A,(FLAGS)        ; a FLAGS 21BCH-hoz képest számított cím
        ...
FLAGS:  DB 0
SUPPORT_END:
        DEPHASE
SUPPORT_LEN EQU SUPPORT_END-SUPPORT_RUN
```

- A blokkon belül a `$` és minden címke futási cím. A bájtok a betöltési
  címen folytatódnak, a környező kóddal azonos kimeneti szegmensben.
- A `DEPHASE` után a fordítás a blokk utáni betöltési címen folytatódik: a
  példában a `FLAGS` utáni bájton.
- A `PHASE` sor címkéje a blokk betöltési címe, a `DEPHASE` sor címkéje a
  blokk utáni futási cím.
- A futási cím csak a `PHASE` sor fölött definiált szimbólumokat
  használhatja.
- A `PHASE` nem ágyazható egymásba. Hiba a blokkon belüli `ORG` és
  `BASIC_START`, a `PHASE` nélküli `DEPHASE` és a lezáratlan `PHASE`.
- A `címke@leképezés` a futási címre vonatkozik, mint bármely más
  címkeérték.
- A TOML `[[lines]]` táblában a blokk sorainál az `addr` (betöltési cím)
  mellett `run_addr` is szerepel.

## Kifejezések

A kifejezések kiértékelése 32 bites előjeles egészekkel történik; a
tartományt a felhasználás helye ellenőrzi (bájtoperandus `0..255`, szó
`0..65535`).

Alapelemek:

- decimális számok;
- `0x1234`, `$1234` és `1234H` hexadecimális számok;
- `0b1010` és `1010B` bináris számok;
- címkék és `EQU` szimbólumok;
- `$` mint aktuális cím;
- `címke@leképezés` címátalakítások;
- zárójeles részkifejezések;
- függvényhívások (lásd [Függvények](#függvények)).

Ami számként olvasható, az szám: a `C000H` a `0C000H` érték, nem szimbólum.
A függvénynevek csak közvetlenül `(` előtt különlegesek.

### Függvények

| Függvény | Érték |
| --- | --- |
| `LOW(x)` | `x & 0FFH` |
| `HIGH(x)` | `(x >> 8) & 0FFH` |
| `ABS(x)` | abszolút érték |
| `MIN(a, b, …)`, `MAX(a, b, …)` | két vagy több érték közül a legkisebb, illetve a legnagyobb |
| `SIN(szög, periódus, amplitúdó)` | `amplitúdó × sin(2π × szög / periódus)`, kerekítve |
| `COS(szög, periódus, amplitúdó)` | ugyanez `cos` függvénnyel |

A `SIN` és a `COS` a szöget egy teljes kör `periódus` darab lépésében méri;
a periódus pozitív kell legyen, az egy körön túli szög körbefordul. Az
eredmény a feleket nullától elfelé kerekíti, így `SIN(1,12,1)`
(sin 30° = 0,5) értéke `1`. A számítás minden változatban azonos, ezért egy
táblázat a hibakeresőben és a parancssorban is ugyanazokat a bájtokat adja.
A [`REPT`](#ismétlés) direktívával táblázatok készíthetők:

```asm
SINE:   REPT 256, I
        DB 128+SIN(I, 256, 127)     ; 128 ± 127 szinuszhullám
        ENDR
```

Műveletek a legerősebbtől a leggyengébbig, C szerinti jelentéssel:

| Erősség | Műveletek | Megjegyzés |
| --- | --- | --- |
| 1 | egyoperandusú `-`, `+`, `~`, `!` | `~` bitenkénti tagadás; `!x` értéke 1, ha `x` nulla, különben 0 |
| 2 | `*`, `/`, `%` | `/` és `%` nulla felé csonkol; nullával osztás hiba |
| 3 | `+`, `-` | |
| 4 | `<<`, `>>` | a léptetés száma `0..31`; a `>>` megtartja az előjelet |
| 5 | `&` | |
| 6 | `^` | |
| 7 | `\|` | |

Az azonos erősségű műveletek balról jobbra kötnek, így `10-4-3` értéke `3`.
Összehasonlítás csak az `ASSERT` direktívában van.

```asm
GLYPHS      EQU 89
GLYPHS_SIZE EQU GLYPHS*64          ; 1640H
TABLE       EQU 0B000H
TABLE_PAGE  EQU HIGH(TABLE)        ; 0B0H
            LD A,(TABLE+2*2)       ; memóriaoperandus
            OUT (60H+INDEX),A
```

Ha egy operandus teljes szövege egyetlen zárójeles csoport, az Z80
memóriaoperandus: `LD A,(TABLE+4)` a `TABLE+4` címről tölt, az
`OUT (60H+1),A` a `61H` portra ír. A csak zárójellel kezdődő és végződő
operandus, például `(2+1)*(1+1)`, kifejezés. Zárójeles konstans közvetlen
értékként `+` vagy `0+` előtaggal írható: `LD A,+(X+1)`. A direktíváknak
(`DB`, `DW`, `EQU` stb.) nincs memóriaoperandusa.

A [struktúra](#struktúrák) mezője szám, így ugyanez a szabály érvényes: a
`LD A,(SPRITE.X)` a mező eltolásával egyenlő címről tölt. Bázist kell
hozzáadni: `LD A,(IX+SPRITE.X)` vagy `LD A,(PLAYER+SPRITE.X)`.

A `címke@leképezés` alak az `ORG` sorban rögzített eredetből a leképezett
címre alakít át; a későbbi `ORG` ezt nem módosítja. Az ismeretlen vagy
ismételten definiált leképezés assemblerhiba.

Az első menetben a később definiált szimbólumot használó kifejezés
helykitöltő értéket kap, és az esetleges aritmetikai hibái nem számítanak; a
második menet minden szimbólum ismeretében értékel.

## Futtatható TVC BASIC programok

Kis, automatikusan induló TVC BASIC programhoz `ORG` helyett `BASIC_START`
használható:

```asm
        BASIC_START

        LD A,02H
FLASH:  OUT (00H),A
        XOR 0AH
        JP FLASH
```

Közvetlen CAS-kimenet:

```cmd
rtvc-asm --format cas experiment.asm -o experiment.cas
```

Bármely más `ORG` címre fordított program is futtatható CAS-fájl lesz. Az
rtvc-asm ugyanazt a BASIC sort teszi elé, utána `1A30H`-n egy 14 bájtos
rutint, amely a programot az `ORG` címére másolja, és oda ugrik:

```asm
        ORG 8000H
START:  LD A,02H
FLASH:  OUT (00H),A
        XOR 0AH
        JP FLASH
```

- A program egyetlen összefüggő szegmens legyen `1A3EH` (közvetlenül a
  másolórutin után) és `BFFFH` között. A BASIC verem és a rendszerváltozók
  `19EFH` alatt vannak, így ez a tartomány a BASIC futása közben szabad.
- A CAS tartalma a BASIC sorral és a másolórutinnal együtt legfeljebb
  42 256 bájt lehet.
- A másolórutin a `START` címkére ugrik, ha az létezik, különben az első
  bájtra. A `--entry NÉV` a programon belül más címkét választ.
- A `RET` visszatér a BASIC-be, amely a `USR` eredményét a betöltött
  tartalom után tárolja. `RET` csak akkor használható, ha a program ezt a
  területet nem írja felül.

A `toml` formátum a hibakereső `loadasm` parancsához, a `bin` formátum pedig
az összefüggő nyers gépi kódhoz használható.

## Utasításkészlet

Az encoder többek között az alábbi alakokat támogatja:

- `LD`, `INC`, `DEC`;
- `ADD`, `ADC`, `SBC`, `SUB`, `AND`, `XOR`, `OR`, `CP`;
- `JP`, `JR`, `DJNZ`, `CALL`, `RET`, `RST`;
- `PUSH`, `POP`, `EX`, `EXX`;
- `IN`, `OUT`, `IM`;
- `BIT`, `RES`, `SET`, `RLC`, `RRC`, `RL`, `RR`, `SLA`, `SRA`, `SLL`, `SRL`;
- rögzített alakok, például `NOP`, `HALT`, `DI`, `EI`, `NEG`, `RETN`,
  `RETI`, `RRD`, `RLD`, `LDI`, `LDIR`, `LDD`, `LDDR`, `CPI`, `CPIR`, `CPD`,
  `CPDR`, `INI`, `INIR`, `IND`, `INDR`, `OUTI`, `OTIR`, `OUTD`, `OTDR`;
- a disassembler által kiírt nem dokumentált alakok: `SLL`, `IN F,(C)`,
  `OUT (C),0`, az IX/IY regiszterfelek (`IXH`, `IXL`, `IYH`, `IYL`) az `LD`,
  `INC`, `DEC` és a 8 bites aritmetikai-logikai utasításokban (például
  `LD IXH,12H`, `ADD A,IYL`), valamint az indexelt forgató/léptető, `RES` és
  `SET` utasítások, amelyek az eredményt egy regiszterbe is bemásolják
  (például `RLC (IX+5),B`, `SET 3,(IY-1),A`).

Minden utasítás, amelyet a disassembler visszafejt, ugyanarra az utasításra
fordul vissza. Indexregiszter-fél nem kombinálható a `H`, `L`, `(HL)`
operandussal vagy a másik indexregiszter felével, mert az előtag
megváltoztatná a jelentésüket.

A nem támogatott mnemonikok és operandusalakok hibát adnak. Nincsenek makrók,
feltételes fordítás, lokális címketartományok, relokációs rekordok,
listafájlok vagy más assemblerekkel kompatibilis szintaxis.

## TOML-kimenet

Az `rtvc-asm` verziózott, olvasható TOML fájlt ír:

```toml
format = "rtvc-asm-v1"
source = "helper.asm"
requested_origin = 0x7000
origin = 0x8000
next_addr = 0x8009

[symbols]
MSG = 0x8006
START = 0x8000

[[segments]]
addr = 0x8000
len = 0x09
bytes = [
  0x21, 0x06, 0x80, 0xC3, 0x00, 0x80, 0x4F, 0x4B, 0x00,
]
```

| Mező | Jelentés |
| --- | --- |
| `format` | Ennél a verziónál mindig `rtvc-asm-v1`. |
| `source` | Forrásútvonal, vagy szabványos bemenetnél `-`. |
| `requested_origin` | A CLI/API kezdőcíme a forrásbeli `ORG` előtt. |
| `origin` | Az első kibocsátott szegmens címe, üres kimenetnél a kért kezdőcím. |
| `next_addr` | Az utolsó utasítás vagy `ORG` utáni aktuális cím. |
| `files` | Csak beillesztett fájlok esetén: a fő forrás, majd minden beillesztett fájl. A beillesztett fájlból származó `[[lines]]` bejegyzéseknek `file` mezőjük van, amely ennek a tömbnek az indexe. |
| `segments` | Címzett bájttartományok; nem folytonos `ORG` esetén több elem. |
| `symbols` | Nagybetűs szimbólumnevek és 16 bites értékeik. |
| `mappings` | Az `ORG` sorok névvel ellátott leképezései: `name`, `source_base`, `mapped_base`. |
| `breakpoints` | A `BREAK` jelölések: `addr`, `line`, beillesztett fájlnál `file`. A `--no-breakpoints` kihagyja. |
| `lines` | A kibocsátott forrássorok sorszáma, címe (`addr`), bájthossza, és `PHASE` blokkon belül a futási cím (`run_addr`). |

A betöltők számára a `segments[].bytes` a mérvadó adat. A `segments[].len`
értékének egyeznie kell a bájttömb hosszával.

### Névvel ellátott címleképezések

Az `ORG` sor egy vagy több, az adott címtartományból kiinduló névvel ellátott
átalakítást definiálhat:

```asm
ORG C000H, SYS0, 0000H, SYS1, 4000H

START:  JP MAIN@SYS0
MAIN:   NOP

ORG C100H
TABLE:  DW TABLE@SYS0, TABLE@SYS1, MAIN

ORG D000H, DATA0, 8000H
OTHER:  JP TABLE@SYS0
```

Ez a `SYS0` leképezést `C000H -> 0000H`, a `SYS1` leképezést pedig
`C000H -> 4000H` értékekkel rögzíti. A leképezés deklarációja nem ír ki bájtokat.
`START` címe `C000H`, `MAIN` címe `C003H`, `TABLE` címe pedig `C100H`. A
leképezett hivatkozás jelentése:

```text
címke@leképezés = címke - forrás-alapcím + leképezett-alapcím
```

Ezért a `DW` értékei `0100H`, `4100H` és `C003H`, a kibocsátott little-endian
bájtok pedig `00 01 00 41 03 C0`. Az utolsó ugrásban a `TABLE@SYS0` értéke
`0100H`, ezért az utasítás bájtjai `C3 00 01`. A `DATA0` leképezés `D000H ->
8000H` értéket rögzít; a későbbi `ORG` sorok nem módosítják a korábbi
leképezéseket. A sima `TABLE` továbbra is `C100H`.

A leképezésnevek egy assemblált modulon belül egyediek. Ismeretlen vagy
ismételten definiált leképezés assemblerhibát okoz.

## Tipikus munkafolyamat

1. Írj egy kis segédforrást:

   ```asm
   ORG 8000H
   START:  LD HL,MSG
           JP START
   MSG:    DB "OK",0
   ```

2. Fordítsd le:

   ```cmd
   rtvc-asm --origin 7000H helper.asm -o helper.toml
   ```

3. Töltsd be az aktív gépbe:

   ```text
   rtvc> loadasm helper.toml
   ```

4. A `disasm`, `read`, töréspont- és léptetőparancsokkal vizsgáld vagy futtasd.

## Hibák

Az `rtvc-asm` a forráshibákat `fájl:sor: üzenet` alakban írja a
szabványos hibakimenetre, programnév-előtag nélkül, így a szerkesztők és a
build eszközök a hibás sorra ugorhatnak. A szabványos bemenet neve `-`. A
`%NÉV%` helyőrzők hibái ugyanígy jelennek meg:

```text
helper.asm:2: unknown symbol 'MISSING'
helper.asm:4: relative target 9000H is out of range from 8000H
helper.asm:7: missing definition for %BLOCK_SIZE%
-:1: unknown symbol 'Q'
```

A többi hiba (hibás opció, a kimeneti formátumnak nem megfelelő program)
megtartja az `rtvc-asm:` előtagot.

A hibakereső `assemble` parancsának nincs fájlneve, ezért ott
`line 2: unknown symbol 'MISSING'` alakú a hiba.

A Rust API-ban az `AsmError` külön tárolja a részeket: a `message()` a
szöveg, a `line()` az 1-től számozott forrássor (ha van), a `Display`
`line N: üzenet`, a `display_in(fájl)` pedig `fájl:N: üzenet` alakot ad. A
`REPT` példányain belüli hiba az eredeti forrássor számát kapja.

A `loadasm` ellenőrzi az `rtvc-asm-v1` formátumot, a legalább egy szegmenst, a
16 bites szegmenscímeket, a `0..255` tartományú bájtokat és – ha jelen van – a
`len` mező egyezését.
