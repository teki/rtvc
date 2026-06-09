# rtvc

Nyelv: [English](README.md) | [Magyar](README.hu.md)

Az `rtvc` egy nyílt forráskódú, több platformon futó emulátor a Videoton TV
Computerhez (TVC), az 1980-as évek magyar 8 bites otthoni és iskolai
számítógépéhez.

Az emulátor aktív fejlesztés alatt áll. Már futtat TVC 64K és 64K+ gépeket
billentyűzetkezeléssel, videóval, hanggal, kazettabetöltéssel, HBF/VT-DOS
lemezképekkel, snapshotokkal és natív asztali felülettel.

Próbáld ki a webes demót: [teki.one/rtvc](http://teki.one/rtvc/)

## A TVC-ről

A Videoton TV Computer, röviden TVC, egy magyar 8 bites otthoni és iskolai
számítógép volt, amelyet a Videoton gyártott az 1980-as évek második felében.
Z80 CPU-t, beépített BASIC-et, kazettás tárolást, külön szöveges mód helyett
grafikus megjelenítési módokat, valamint opcionális bővítéseket, például
floppy támogatást használt.

További történeti háttér a
[VIDEOTON TVC weboldalon](http://tvc.hu/html/tvc_attekintes.html) olvasható.

## Funkciók

- TVC 64K és 64K+ gépváltozatok.
- ROM 1.2 és ROM 2.2 gépválasztás, opcionális VT-DOS/HBF bővítéssel.
- Z80 CPU-emuláció FUSE és ZEX validációs tesztkészletekkel.
- MC6845-alapú videokimenet gyors képkockás és interleaved renderelési móddal.
- Natív billentyűzet-, video- és hangkezelés az asztali felületen.
- CAS kazettabetöltés és DSK lemezkép-támogatás.
- Snapshot mentés/betöltés `.rtvcsnap` és `.rtvcsnap.zip` formátumban.
- Teljes böngészős egui webalkalmazás, valamint könnyű snapshot-csomagok
  önálló demókhoz.
- TCP socket hibakereső natív GUI és headless használathoz.

## Letöltés

Töltsd le a legfrissebb kiadást a
[GitHub Releases oldalon](https://github.com/teki/rtvc/releases).

Release archívumok ezekhez érhetők el:

- Windows x64
- macOS x64
- macOS Apple Silicon

Csomagold ki az archívumot, majd indítsd el:

- Windows alatt az `rtvc.exe` fájlt
- macOS alatt az `RTVC.app` alkalmazást

A release csomagok tartalmazzák az emulátort, a ROM-fájlokat, mellékelt
programokat és a teljes böngészős verziót a `web/` könyvtárban.

### Első indítás macOS-en

A macOS alkalmazás ad hoc aláírást kap, nincs notarizálva. Ha a macOS blokkolja
letöltés után, töröld a böngésző által hozzáadott karantén jelölést a
kicsomagolt alkalmazásról:

```bash
xattr -dr com.apple.quarantine RTVC.app
```

A kiadási archívum terminálból is letölthető, ami általában elkerüli a
böngészős karantén jelölést:

```bash
curl -L https://github.com/teki/rtvc/releases/latest/download/rtvc-macos-arm64.zip | ditto -x -k - $HOME/Downloads/rtvc
```

## Az emulátor használata

A natív alkalmazás menüket ad a géptípus kiválasztásához, kazetta- vagy
lemezképek betöltéséhez, snapshotok mentéséhez és betöltéséhez,
képernyőmentések mentéséhez, valamint az I/O napló megjelenítéséhez.

Támogatott felhasználói fájlok:

| Fájltípus | Cél |
| --- | --- |
| `.cas` | TVC kazettakép. |
| `.dsk` | Floppy lemezkép HBF/VT-DOS használathoz. |
| `.zip` | Programarchívum, amely `.cas` vagy `.dsk` fájlt tartalmaz. |
| `.rtvcsnap` | Nyers rtvc snapshot. |
| `.rtvcsnap.zip` | Tömörített rtvc snapshot. |

A snapshot a legegyszerűbb módja az aktuális gépállapot megőrzésének. A natív
alkalmazás tud tömörített `.rtvcsnap.zip` fájlokat menteni, `.rtvcsnap` és
`.rtvcsnap.zip` fájlokat betölteni, és közvetlenül snapshot-útvonalról indulni.

## Futtatás forrásból

Telepíts egy friss Rust toolchaint, majd futtasd:

```bash
cargo run --bin rtvc
```

Indítás snapshotból:

```bash
cargo run --bin rtvc -- snapshots/load_tape.rtvcsnap.zip
```

Média betöltése indításkor:

```bash
# Floppy lemezkép csatlakoztatása
cargo run --bin rtvc -- -d utvonal/lemez.dsk

# Kazetta csatlakoztatása standard betöltéshez
cargo run --bin rtvc -- -t utvonal/kazetta.cas

# Kazetta közvetlen memóriába töltése
cargo run --bin rtvc -- -i utvonal/kazetta.cas
```

Forrásból futtatáskor helyezd a ROM-fájlokat a `roms/` könyvtárba. Az
opcionális programarchívumok és médiafájlok a `progs/` könyvtárba kerülhetnek.

## Webes emulátor

A release archívum tartalmazza az emulátor teljes böngészős verzióját.
Használatához szolgáld ki a `web/` könyvtárat, majd nyisd meg böngészőben:

```bash
cd web
python -m http.server 8000
```

Fejlesztők ugyanezt a webalkalmazást így építhetik és szolgálhatják ki helyben:

```bash
cargo install wasm-bindgen-cli --version 0.2.122
# A webes csomag felépítése a docs/ könyvtárba
cargo xtask bundle-web-full docs
# A docs/ könyvtár kiszolgálása (macOS/Linux alatt)
python scripts/serve_docs.py
# Vagy a docs/ könyvtár kiszolgálása (Windows alatt)
scripts\serve_docs.bat
```

A webes emulátor helyi CAS, DSK, ZIP és snapshot fájlokat is meg tud nyitni. A
kisebb beállítások `localStorage`-ba kerülnek; a legutóbbi kazetta- és
lemezadatok IndexedDB-ben tárolódnak.

## Fejlesztői jegyzetek

Hasznos parancsok:

```bash
cargo build
cargo run --bin fuse_test
cargo run --bin perf_test
```

A socket hibakereső natív GUI-val és headless módban is használható:

```bash
# Natív UI hibakeresővel a 8089-es porton
cargo run --bin rtvc -- -p 8089

# Headless emulátor hibakeresővel a 8080-as porton
cargo run --bin rtvc -- -H -p 8080
```

A teljes fejlesztési munkafolyamathoz lásd:
[.agents/skills/development/SKILL.md](.agents/skills/development/SKILL.md).

## Dokumentáció

- [Projektáttekintés](info/project_overview.md)
- [Snapshotformátum és webes csomagok](info.hu/snapshot.md)
- [TVC gépmag](info.hu/tvc.md)
- [Z80 CPU](info.hu/z80.md)
- [Z80 opcode referencia](info.hu/z80opcodes.md)
- [Memóriakezelő egység](info.hu/mmu.md)
- [Videovezérlő](info.hu/vid.md)
- [Hang](info/sound.md)
- [Billentyűzetmátrix](info.hu/key.md)
- [Kazettatámogatás](info.hu/cas.md)
- [HBF floppy kártya és FD1793 vezérlő](info.hu/hbf.md)
- [Socket hibakereső](info.hu/dbg.md)

## Közreműködés

Hibajegyeket és pull requesteket szívesen fogadunk. Az
emulátorpontossági jelentések akkor a leghasznosabbak, ha tartalmaznak egy kis
reprodukciót: géptípust, médiafájlt, snapshotot, a TVC-n beírt parancsot,
valamint minden releváns port- vagy interruptnaplót.

Kérjük, az emulátor viselkedését érintő változtatásokat ahol ésszerű, fedd le
célzott tesztekkel, és frissítsd az `info/` dokumentációt, ha az
alaparchitektúra, a snapshotformátum, a médiakezelés vagy a buildfolyamat
változik.

## Köszönetnyilvánítás

Az `rtvc` a korábbi [teki/jstvc](https://github.com/teki/jstvc) JavaScript
implementáció portolásából indult. A CPU-tesztelési folyamat nyilvános Z80
validációs anyagokat használ, például FUSE és ZEX tesztprogramokat. A projekt
történeti TVC hardverinformációkra és megőrzési anyagokra is támaszkodik.

## Licenc

Az emulátor kódja az [MIT licenc](LICENSE) alatt érhető el.

A ROM-ok, kazetta- és lemezképek, snapshotok, képernyőmentések, kézikönyvek és
más történeti vagy harmadik féltől származó gépanyagok megőrzési,
kompatibilitástesztelési vagy kényelmi céllal szerepelhetnek a projektben.
Ezekre nem vonatkozik az MIT licenc, hacsak ez nincs külön jelezve.
