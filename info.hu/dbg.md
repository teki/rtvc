# Socket hibakereső interfész specifikáció és használati útmutató

Az `rtvc` emulátor natív módban futtatva egy TCP-alapú hibakereső (debugger) felületet biztosít a `localhost` (127.0.0.1) címen. Ez lehetővé teszi külső scriptek, tesztkörnyezetek vagy AI ágensek számára az emulátor vezérlését, állapotának lekérdezését és a CPU lépésenkénti végrehajtását.

A hibakereső mind **headless futtatási módban** (`--headless`), mind a hagyományos **natív GUI módban** működik.

---

## Parancssori beállítások

A socket hibakereső portja és a futtatási mód az alábbi CLI kapcsolókkal konfigurálható:

- `-p`, `--port <port>`: Megadja a kötni kívánt TCP portot (alapértelmezett: `8080`).
- `-H`, `--headless`: Az emulátort az `egui`/`eframe` GUI loop nélkül futtatja, teljesen egy háttérbeli lekérdező loopban futva.

Például az emulátor indítása grafikus felülettel és a debuggerrel a `8089`-es porton:
```bash
cargo run --bin rtvc -- --port 8089
```

Headless emulátor indítása a `8080`-as porton:
```bash
cargo run --bin rtvc -- --headless --port 8080
```

---

## TCP protokoll specifikáció

A TCP debugger **újsor-karakterrel elválasztott JSON objektumokon** keresztül kommunikál. A kliens által küldött minden kérésnek egyetlen, új sorral (`\n`) lezárt JSON objektumból kell állnia. Az emulátor válasza szintén egyetlen új sorral lezárt JSON objektum lesz.

### 1. Parancsok

#### `status`
Lekérdezi a Z80 CPU regisztereit, ciklusszámlálóját és aktuális állapotát.
- **Kérés**: `{"cmd": "status"}`
- **Válasz**:
  ```json
  {
    "status": "ok",
    "running": false,
    "halted": false,
    "cycles": 124508,
    "pc": 3450,
    "sp": 65535,
    "af": 65535,
    "bc": 0,
    "de": 0,
    "hl": 16384,
    "ix": 65535,
    "iy": 65535
  }
  ```

#### `stats`
Jelenti a befejezett emulációs képkockák átlagos sebességét egy gördülő, öt másodperces gazdagép-időablakban. Az emulátor indítása utáni első öt másodpercben az ablak rövidebb; a szüneteltetett idő beleszámít, ezért az átlag csökken, ha az emuláció nem tartja a valós idejű sebességet vagy szünetel.
- **Kérés**: `{"cmd": "stats"}`
- **Válasz**:
  ```json
  {
    "status": "ok",
    "running": true,
    "average_fps": 49.8,
    "window_seconds": 5.0,
    "frames": 249
  }
  ```

#### `close_app`
Bezárja az emulátor alkalmazást. GUI módban a szokásos alkalmazásleállítás fut le, beleértve az alkalmazásállapot mentését; headless módban a futási ciklus kilép.
- **Kérés**: `{"cmd": "close_app"}`
- **Válasz**: `{"status": "ok"}`

#### `step`
Egy vagy több Z80 CPU utasítást hajt végre. Ez automatikusan frissíti a rendszertidőzítőket, a kazetta-lejátszást, a hanggenerálást és az órajelciklusokat.
- **Kérés**: `{"cmd": "step", "count": 5}` (ahol a `"count"` egy opcionális egész szám, alapértelmezett értéke `1`)
- **Válasz**: `{"status": "ok"}`

#### `continue`
Folytatja a valós idejű CPU emulációt.
- **Kérés**: `{"cmd": "continue"}`
- **Válasz**: `{"status": "ok"}`

#### `pause`
Szünetelteti a valós idejű CPU emulációt.
- **Kérés**: `{"cmd": "pause"}`
- **Válasz**: `{"status": "ok"}`

#### `reset`
Hardveres alaphelyzetbe állítást (reset) hajt végre a CPU-n, az MMU-n és a perifériákon, majd leállítja a CPU-t.
- **Kérés**: `{"cmd": "reset"}`
- **Válasz**: `{"status": "ok"}`

#### `breakpoint_add`
Végrehajtási töréspontot (breakpoint) ad hozzá egy megadott 16 bites címhez.
- **Kérés**: `{"cmd": "breakpoint_add", "addr": 256}`
- **Válasz**: `{"status": "ok"}`

#### `breakpoint_remove`
Eltávolít egy végrehajtási töréspontot egy megadott 16 bites címről.
- **Kérés**: `{"cmd": "breakpoint_remove", "addr": 256}`
- **Válasz**: `{"status": "ok"}`

#### `breakpoint_list`
Kilistázza az összes jelenleg aktív töréspontot.
- **Kérés**: `{"cmd": "breakpoint_list"}`
- **Válasz**: `{"status": "ok", "breakpoints": [256, 1024]}`

#### `read_memory`
Lekérdezi a leképezett vagy fizikai memória tartalmát.
- **Kérés**: `{"cmd": "read_memory", "addr": 0, "len": 4, "bank": "sys"}`
  - `"addr"`: Kezdő 16 bites cím.
  - `"len"`: A kiolvasni kívánt bájtok száma.
  - `"bank"`: (Opcionális sztring) Közvetlenül egy adott fizikai memóriabankból olvas, megkerülve az aktív MMU lapleképezéseket. Elérhető bankok:
    - RAM bankok: `"u0"`, `"u1"`, `"u2"`, `"u3"`
    - Videomemória bankok: `"vid0"`, `"vid1"`, `"vid2"`, `"vid3"`
    - Boot ROM: `"sys"`
    - Cartridge ROM: `"cart"`
    - Bővítő ROM: `"exth"`
- **Válasz**: `{"status": "ok", "data": [195, 41, 2, 0]}`

#### `disassemble`
Visszafejti a Z80 utasításokat assembly mnemonikokká.
- **Kérés**: `{"cmd": "disassemble", "addr": 0, "len": 4}`
- **Válasz**:
  ```json
  {
    "status": "ok",
    "instructions": [
      { "addr": 0, "bytes": [195, 41, 2], "len": 3, "text": "JP 0229H" },
      { "addr": 3, "bytes": [0], "len": 1, "text": "NOP" }
    ]
  }
  ```

#### `assemble`
Egyetlen Z80 utasítást kódol anélkül, hogy módosítaná az emulált memóriát. A
cím a `JR` és `DJNZ` relatív eltolásainak kiszámításához szükséges.
- **Kérés**: `{"cmd": "assemble", "addr": 32768, "source": "LD A,42"}`
- **Válasz**:
  ```json
  {
    "status": "ok",
    "addr": 32768,
    "len": 2,
    "bytes": [62, 42],
    "next_addr": 32770
  }
  ```

#### `save_snapshot` / `load_snapshot`
Ment egy tömörített/nyers emulátorállapot-snapshotot, vagy betölt egyet.
- **Kérés**: `{"cmd": "save_snapshot", "path": "data/snapshots/save.rtvcsnap.zip"}`
- **Kérés**: `{"cmd": "load_snapshot", "path": "data/snapshots/save.rtvcsnap.zip"}`
- **Válasz**: `{"status": "ok"}` (vagy hiba esetén `{"status": "error", "message": "..."}`)

#### `save_screenshot`
Elment egy 4:3-as arányú PNG képet a TVC aktuális képkockapufferéből.
- **Kérés**: `{"cmd": "save_screenshot", "path": "screenshot.png"}`
- **Válasz**: `{"status": "ok"}`

#### `key`
Szimulál egy billentyűzet-eseményt.
- **Kérés**: `{"cmd": "key", "action": "press", "char": "A"}`
- **Kérés**: `{"cmd": "key", "action": "down", "code": 65}`
- **Kérés**: `{"cmd": "key", "action": "up", "code": 65}`
  - `"action"`: `"down"` (billentyű lenyomás), `"up"` (billentyű felengedés), vagy `"press"` (karakter gépelés).
  - `"code"`: JavaScript billentyűkód egész szám (kötelező a `"down"` és `"up"` műveletekhez).
  - `"char"`: A gépelni kívánt karakter (kötelező a `"press"` művelethez).
- **Válasz**: `{"status": "ok"}`

---

### 2. Aszinkron eseményértesítések

Ha az emulátor futás állapotban van (`"running": true`) és elér egy aktív töréspontot, akkor automatikusan szüneteltetett állapotba vált, és egy aszinkron JSON eseményt küld a TCP csatornán a kliens értesítésére:

```json
{"event": "breakpoint", "pc": 256}
```

---

## Interaktív Python REPL kliens

Egy interaktív CLI kliens is rendelkezésre áll a [rtvc_debug.py](../scripts/rtvc_debug.py) fájlban. Ez támogatja a tabulátoros kiegészítést, parancselőzményeket, a regiszterek táblázatos megjelenítését és a memória hexadecimális kiíratását.

### A kliens indítása
A kliens indítása és csatlakozás az alapértelmezett helyi debuggerhez (`127.0.0.1:8080`):
```bash
python3 scripts/rtvc_debug.py
```

Csatlakozás egyedi kiszolgálóhoz vagy porthoz:
```bash
python3 scripts/rtvc_debug.py --host 127.0.0.1 --port 8089
```

### REPL parancsok

| Parancs | Alias | Leírás |
|---|---|---|
| `status` | `s` | Kiírja a regisztereket (AF, BC, DE, HL, IX, IY, SP, PC), a ciklusszámot és az állapotokat. |
| `stats` | `fps` | Kiírja az átlagos emulációs FPS-t a gördülő öt másodperces időablakban. |
| `close_app` | `close` | Bezárja az emulátort és kilép a hibakereső konzolból. |
| `step [count]` | `t` | Lépteti a CPU-t `count` utasítással és megjeleníti az új regiszterállapotokat. |
| `continue` | `c` | Folytatja a valós idejű emulációt. |
| `pause` | `p` | Szünetelteti a CPU futását. |
| `reset` | | Alaphelyzetbe állítja az emulátort. |
| `bp list` | | Kilistázza az aktív töréspontokat. |
| `bp add <addr>` | | Töréspontot ad hozzá (decimális vagy `0x...` formátumú címen). |
| `bp rm <addr>` | | Eltávolít egy töréspontot. |
| `read <addr> <len> [bank]` | `m` | Hexadecimális dumpot készít a memóriáról ASCII nézettel. |
| `disasm <addr> <len>` | `d` | Assembly mnemonikokra bontja a memóriaterületet. |
| `asm [addr]` | `a` | Interaktív, egysoros assembler módot indít a megadott címen, cím nélkül pedig az aktuális PC-n, és kiírja a kódolt bájtokat. |
| `save <path>` | | Snapshotot ment a megadott fájlba. |
| `load <path>` | | Snapshotot tölt be a megadott fájlból. |
| `screenshot <path>` | | Képernyőmentést készít 4:3-as PNG formátumban. |
| `key press <char>` | | Karakterlenyomást szimulál. |
| `key down <val>` | | Billentyűkód lenyomást szimulál. |
| `key up <val>` | | Billentyűkód felengedést szimulál. |
| `exit` | `q` | Kilép a REPL konzolból. |
