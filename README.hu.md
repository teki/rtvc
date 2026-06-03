# rtvc

Nyelv: [English](README.md) | [Magyar](README.hu.md)

Az `rtvc` egy nyílt forráskódú, több platformon futó, Rust-alapú emulátor a Videoton TV Computerhez (TVC).

A projekt aktív fejlesztés alatt áll. A CPU futtatása, a TVC memóriakezelése, a billentyűzetkezelés, a videokimenet, a kazettabetöltés, a HBF/VT-DOS lemeztámogatás és a natív snapshotok már működnek, de a hardverpontosságon még folyik a munka.

## Miért?

Főként azért, hogy tanuljak a Rust ökoszisztémáról és egy nyílt forráskódú alkalmazás publikálásáról.

Másodlagos cél egy olyan eszköz készítése, amely segíti a TVC-s programok létrehozását és portolását.

## A TVC-ről

A Videoton TV Computer, röviden TVC, egy magyar 8 bites otthoni és iskolai számítógép volt, amelyet a Videoton gyártott az 1980-as évek második felében. A gép az Enterprise vonalhoz kapcsolódó licencelt terven alapult, és a magyar oktatási felhasználáshoz igazították.

A gép Z80 CPU-t használt 3,125 MHz-es órajellel, és főként három változatban került forgalomba: 32K, 64K és 64K+. A 64K+ modell több videomemóriát és újabb BASIC 2.2 ROM-ot kapott. A beépített TVC OS és BASIC ROM-ban lakott; az alaprendszer kazettás tárolást támogatott, míg az UPM és a VT-DOS fejlettebb lemezorientált környezetet adott.

Sok korabeli otthoni géptől eltérően a TVC-nek nem volt külön, csak szöveges kijelzési módja. A szöveg a grafikus rendszeren keresztül jelent meg, 512x240-es 2 színű, 256x240-es 4 színű és 128x240-es 16 színű módokkal. A videokimenetet egy 6845 CRTC állította elő. A hang egyetlen programozható csatornából állt, amely a rendszerórajelből származott, 16 hangerőszinttel és 4 bites D/A móddal, amikor a frekvenciaosztó ki volt kapcsolva.

A TVC bővíthető gépnek készült. Volt rajta kazetta-, RGB-, TV-, nyomtató-, joystick-, cartridge- és felső bővítőcsatlakozó; gyakori bővítések voltak a memóriakártyák, floppyvezérlők, soros kártyák és EPROM-programozók. Nagyjából 12 000 darab készült, főként iskolák számára, mielőtt a gyártás néhány év után véget ért.

Forrás: [VIDEOTON TVC történeti áttekintés](http://tvc.hu/html/tvc_attekintes.html).

## Funkciók

- Z80 CPU-emuláció FUSE és ZEX tesztkészletekkel.
- TVC 64K és 64K+ gépváltozatok.
- ROM 1.2 és ROM 2.2 gépválasztás, opcionális VT-DOS/HBF bővítéssel.
- MC6845-alapú TVC videokimenet gyors képkockás és interleaved renderelési móddal.
- TVC billentyűzetmátrix-kezelés a natív egui felületen.
- CAS kazetta-lejátszás/betöltés és DSK lemezkép-támogatás.
- Natív snapshot mentés/betöltés `.rtvcsnap` és `.rtvcsnap.zip` formátumban.
- Statikus webes snapshot-csomagok készítése böngészőben futó demókhoz.

## Letöltés

Töltsd le a legfrissebb kiadást a [GitHub Releases oldalon](https://github.com/teki/rtvc/releases).

Release archívumok Windows x64, macOS x64 és macOS Apple Silicon rendszerekhez érhetők el. Tartalmazzák a natív emulátort, a ROM-okat, mellékelt programokat és egy `web/` snapshot-lejátszót. Csomagold ki a zipet, majd Windows alatt indítsd el az `rtvc.exe`, macOS alatt pedig az `RTVC.app` alkalmazást. A macOS alkalmazás ad hoc aláírást kap, ezért az első indításhoz szükség lehet a Control-kattintásra vagy jobb kattintásra, majd az Open megnyomására.

A webes lejátszó használatához másolj egy tömörített snapshotot `web/snapshot.rtvcsnap.zip` néven, szolgáld ki a `web/` könyvtárat bármilyen statikus webszerverrel, majd nyisd meg böngészőben:

```bash
cd web
python -m http.server 8000
```

Próbáld ki a webes demót: [teki.one/rtvc](http://teki.one/rtvc/)

## A natív emulátor futtatása

```bash
cargo run --bin rtvc
```

Indítás közvetlenül snapshotból:

```bash
cargo run --bin rtvc -- snapshots/load_tape.rtvcsnap.zip
```

Futtatás előtt helyezd a ROM-fájlokat a `roms/` könyvtárba. A natív felület jelenleg ezeket a gépválasztásokat támogatja:

- `64k+ 1.2, VT-DOS`
- `64k+ 2.2, VT-DOS`
- `64k  1.2`
- `64k+ 1.2`
- `64k+ 2.2`

A projekt által használt gyakori ROM-fájlnevek:

- `TVC12_D3.64K`
- `TVC12_D4.64K`
- `TVC12_D7.64K`
- `TVC22_D4.64K`
- `TVC22_D6.64K`
- `TVC22_D7.64K`
- `C_TVCDOS.128`
- `D_TVCDOS.128`
- `C_DOS12.128`
- `D_DOS12.128`

Az opcionális programarchívumok és médiafájlok a `progs/` könyvtárba kerülhetnek.

## Támogatott fájlok

| Fájltípus | Cél |
| --- | --- |
| `.cas` | TVC kazettakép. |
| `.dsk` | Floppy lemezkép HBF/VT-DOS használathoz. |
| `.zip` | Programarchívum, amely `.cas` vagy `.dsk` fájlt tartalmaz. |
| `.rtvcsnap` | Nyers rtvc snapshot. |
| `.rtvcsnap.zip` | Tömörített rtvc snapshot. |

## Natív snapshotok

A natív GUI snapshot-gombokat tartalmaz:

- A `Save Snapshot` alapértelmezés szerint tömörített `.rtvcsnap.zip` fájlt ír.
- A `Load Snapshot` `.rtvcsnap.zip` és nyers `.rtvcsnap` fájlokat is be tud olvasni.
- A natív alkalmazás az első parancssori argumentumként opcionális snapshot-útvonalat is elfogad.
- A `Save Screenshot` a jelenlegi TVC képkockapuffert 4:3 arányú PNG-ként menti (`768x576`).

A tömörített snapshotok hagyományos zip-fájlok, amelyek egy `snapshot.rtvcsnap` bejegyzést tartalmaznak.

## Dokumentáció

- [Snapshotformátum](info.hu/snapshot.md)
- [TVC gépmag](info.hu/tvc.md)
- [Z80 CPU](info.hu/z80.md)
- [Z80 opcode referencia](info.hu/z80opcodes.md)
- [Memóriakezelő egység](info.hu/mmu.md)
- [Videovezérlő](info.hu/vid.md)
- [Billentyűzetmátrix](info.hu/key.md)
- [Kazettatámogatás](info.hu/cas.md)
- [HBF floppy kártya és FD1793 vezérlő](info.hu/hbf.md)

## Közreműködés

Hibajegyeket és pull requesteket szívesen fogadunk. Az emulátorpontossági jelentések akkor a leghasznosabbak, ha tartalmaznak egy kis reprodukciót: géptípust, médiafájlt, snapshotot, a TVC-n beírt parancsot, valamint minden releváns port- vagy interruptnaplót.

Kérjük, az emulátor viselkedését érintő változtatásokat ahol ésszerű, fedd le célzott tesztekkel, és frissítsd az `info/` dokumentációt, ha az alaparchitektúra, a snapshotformátum, a médiakezelés vagy a buildfolyamat változik.

## Köszönetnyilvánítás

Az `rtvc` a korábbi [teki/jstvc](https://github.com/teki/jstvc) JavaScript implementáció portolásából indult. A CPU-tesztelési folyamat nyilvános Z80 validációs anyagokat használ, például FUSE és ZEX tesztprogramokat. A projekt történeti TVC hardverinformációkra és megőrzési anyagokra is támaszkodik.

## Licenc

Az emulátor kódja az [MIT licenc](LICENSE) alatt érhető el.

A ROM-ok, kazetta- és lemezképek, snapshotok, képernyőmentések, kézikönyvek és más történeti vagy harmadik féltől származó gépanyagok megőrzési, kompatibilitástesztelési vagy kényelmi céllal szerepelhetnek a projektben. Ezekre nem vonatkozik az MIT licenc, hacsak ez nincs külön jelezve.
