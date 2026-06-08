# Snapshotformátum

Az `rtvc` saját, verziózott snapshotformátumot használ az emulátor állapotának mentésére és betöltésére. A formátum TVC-specifikus; nem próbálja újrahasznosítani a Spectrum `.sna`, Spectrum `.z80`, CPC `.sna` vagy RetroArch mentési állapotokat.

## Miért saját formátum?

Az általánosan ismert snapshotformátumok az eredeti gépükhöz kötődnek:

- A ZX Spectrum `.sna` és `.z80` Spectrum-specifikus memóriaképet és hardverállapotot ír le.
- A CPC `.sna` az Amstrad CPC hardveréhez és CRTC-állapotához készült.
- A RetroArch mentések libretro-magok szerializált állapotai, nem hordozható, emulátorok közti formátumok.

A TVC-hez saját állapotmodell kell: Z80 regiszterek, TVC MMU bankok, videomemória, CRTC/videó állapot, bővítőhardver és későbbi eszközállapotok.

## Fájlstruktúra

A snapshotfájl eleje:

```text
RTVCSNAP
u16 version
```

A fájl többi része chunkok sorozata:

```text
u8[4] chunk_id
u32   chunk_length
u8[]  chunk_payload
```

A chunkok tartalma little-endian. Az ismeretlen chunkokat a betöltő figyelmen kívül hagyja, így későbbi verziók opcionális állapotot adhatnak hozzá.

## 1-es verziójú chunkok

- `META` - géptípus, videomodell, emulátoróra, képkocka-kész jelző.
- `CPUZ` - Z80 regisztertömbök és interrupt/HALT állapot.
- `MMU ` - TVC RAM, videó RAM, ROM/cartridge bankok, lapozóregiszterek és plusz modell állapot.
- `VID ` - kiválasztott videomód, CRTC-regiszterek, paletta és keretszín.
- `HBF ` - opcionális VT-DOS/HBF bővítőállapot, bővítő RAM-mal és FDC/lemezkép állapottal.
- `BUS ` - függő interrupt, bővítőleképezés, kazettatranszport és hangidőzítő állapot.
- `EMUT` - opcionális natív UI gépválasztás (`64K`/`64K+`, ROM-verzió, VT-DOS jelenlét). A mag és a WASM betöltő ismeretlen chunkként kihagyja.
- `EMUI` - opcionális natív UI médiaválasztás, jelenleg a kiválasztott `progs/` fájlnév.

A billentyűzet és a napló állapota snapshot betöltéskor szándékosan alaphelyzetbe kerül.

## Futásidejű API-k

- [Tvc::save_snapshot](../src/tvc.rs) snapshot bájtokat ad vissza.
- [Tvc::load_snapshot](../src/tvc.rs) snapshot bájtokból állít vissza állapotot.
- [Emu::save_snapshot](../src/emu.rs) és [Emu::load_snapshot](../src/emu.rs) a natív kódhoz csomagolja a mag API-ját.
- [WasmTvc::saveSnapshot](../src/wasm.rs) és [WasmTvc::loadSnapshot](../src/wasm.rs) a JavaScript felülethez ad API-t.

A natív snapshotok `EMUT` chunkot tartalmaznak, hogy betöltéskor a pontos UI gépválasztás is visszaálljon. Régebbi, `EMUT` nélküli snapshotoknál a betöltő a magban tárolt gépcsaládból következtet, és megtartja az aktuális ROM-verziót, ha azt a snapshot nem rögzítette.

A natív snapshotok `EMUI` chunkot is tartalmaznak, hogy a programválasztó visszaálljon a kiválasztott kazetta- vagy lemezarchívumra. Ha a média még elérhető a `progs/` alatt, a natív betöltő újracsatolja.

## Tömörítés

A natív mentés/betöltés nyers `.rtvcsnap` és `.rtvcsnap.zip` fájlokat támogat. A tömörített snapshot zip archívum, benne egy `.rtvcsnap` bejegyzéssel.

Indítás közvetlenül snapshotból:

```bash
cargo run --bin rtvc -- snapshots/load_tape.rtvcsnap.zip
```

A zip-tömörítés szándékosan nincs benne a könnyű WASM buildben. Webes csomag tartalmazhat tömörített snapshotot, de a böngészőoldali JavaScript bontja ki, mielőtt meghívja a `WasmTvc::loadSnapshot` függvényt.

A felhasználói snapshot- és webes csomagparancsok a [README.hu.md](../README.hu.md) fájlban vannak dokumentálva.

## Könnyű webes csomagok

`cargo bundle-web path/to/game.rtvcsnap` lefordítja a könnyű WASM célpontot, és önálló statikus lejátszót készít a `dist/<snapshot-name>-web/` könyvtárba. A megadott snapshot `snapshot.rtvcsnap` vagy `snapshot.rtvcsnap.zip` néven kerül bele.

`cargo xtask bundle-web-skeleton` ugyanazt a lejátszót készíti el beágyazott snapshot nélkül, alapértelmezés szerint `dist/rtvc-web-skeleton/` alá. Példa explicit kimeneti könyvtárra:

```bash
cargo xtask bundle-web-skeleton dist/rtvc-snapshot-player
```

A release archívumok a teljes webes emulátort tartalmazzák `web/` néven. A normál böngészős emulátorfelülethez használd a `cargo xtask bundle-web-full [out-dir]` parancsot; a `bundle-web` és `bundle-web-skeleton` parancsok a könnyű, snapshot-specifikus oldalakhoz maradnak.
