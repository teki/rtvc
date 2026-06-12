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

## 2-es verziójú chunkok

- `META` - géptípus, videomodell, emulátoróra, képkocka-kész jelző.
- `CPUZ` - Z80 regisztertömbök és interrupt/HALT állapot.
- `MMU ` - 64 KiB TVC RAM, a géptípusnak megfelelő videó RAM, lapozóregiszterek és plusz modell állapot.
- `VID ` - kiválasztott videomód, CRTC-regiszterek, paletta és keretszín.
- `HBF ` - opcionális VT-DOS/HBF módosítható állapot, benne a 4 KiB RAM és az FDC regiszterei.
- `BUS ` - függő interrupt, bővítőleképezés, kazettatranszport és hangidőzítő állapot.
- `EMUT` - opcionális emulátorgép-választás (`64K`/`64K+`, ROM-verzió, VT-DOS jelenlét). A natív és a könnyű WASM burkoló ebből tölti be a szükséges ROM-erőforrásokat.
- `EMUI` - opcionális UI médiahivatkozások: a kiválasztott `progs/` fájlnév és a csatlakoztatott lemez fájlneve. A mag betöltője figyelmen kívül hagyja.

A billentyűzet és a napló állapota snapshot betöltéskor szándékosan alaphelyzetbe kerül.

A ROM- és lemezképbájtok nem kerülnek a snapshotba. A betöltő a normál
ROM-erőforrásokból építi fel a kiválasztott gépet, a hozzáférhető lemezt pedig
fájlnév alapján csatlakoztatja újra, szükség esetén a legutóbbi lemezek között
is keresve. Egy 64K 1.2 gép egyetlen 16 KiB-os videóbankot tárol; a Plus gépek
mind a négy videóbankot mentik.

A 2-es formátum szándékosan nem tölti be az 1-es verziójú snapshotokat.

A `BUS ` hangrésze tárolja a frekvencia- és vezérlőregisztereket, az
időzítőszámlálót, a futási jelzőt, az amplitúdóregisztert, az oszcillátor
fázisát és a PCM mintavételezés törtállapotát. A frontendben várakozó
hangminták nem kerülnek mentésre.

## Futásidejű API-k

- [Tvc::save_snapshot](../src/tvc.rs) snapshot bájtokat ad vissza.
- [Tvc::load_snapshot](../src/tvc.rs) snapshot bájtokból állít vissza állapotot.
- [Emu::save_snapshot](../src/emu.rs) és [Emu::load_snapshot](../src/emu.rs) a natív kódhoz csomagolja a mag API-ját.
- [WasmTvc::saveSnapshot](../src/wasm.rs) és [WasmTvc::loadSnapshot](../src/wasm.rs) a JavaScript felülethez ad API-t.

A natív snapshotok `EMUT` chunkot tartalmaznak, hogy betöltéskor az öt UI
géptípus közül pontosan a mentett változat álljon vissza. A csak magállapotot
tartalmazó, `EMUT` nélküli 2-es snapshotokhoz a hívónak előre létre kell hoznia
a megfelelő gépet és be kell töltenie a ROM-erőforrásokat.

A natív snapshotok `EMUI` chunkot is tartalmaznak, hogy a programválasztó
visszaálljon a kiválasztott kazetta- vagy lemezarchívumra. Ha a kiválasztott
lemez vagy archívum még elérhető a `progs/` alatt vagy a legutóbbi médiák
között, a natív betöltő újracsatolja. A kazettaválasztás is visszaáll, így a
Play újra létrehozhatja a szalagjel-generátort az eredeti fájlból.

## Tömörítés

A natív mentés/betöltés nyers `.rtvcsnap` és `.rtvcsnap.zip` fájlokat támogat. A tömörített snapshot zip archívum, benne egy `.rtvcsnap` bejegyzéssel.

Indítás közvetlenül snapshotból:

```bash
cargo run --bin rtvc -- snapshots/boot12dos.rtvcsnap.zip
```

A repóban található
[boot12dos.rtvcsnap.zip](../snapshots/boot12dos.rtvcsnap.zip) egy tiszta,
teljesen elindított TVC 1.2 VT-DOS tesztállapot. Az indulási folyamatot nem
vizsgáló tesztek stabil, indítás utáni állapotból kezdhetnek vele. A snapshot a
natív és teljes webes alkalmazásba is be van ágyazva, és a Gamebase programok
induló állapotaként szolgál.

A zip-tömörítés szándékosan nincs benne a könnyű WASM buildben. Webes csomag tartalmazhat tömörített snapshotot, de a böngészőoldali JavaScript bontja ki, mielőtt meghívja a `WasmTvc::loadSnapshot` függvényt.

A felhasználói snapshot- és webes csomagparancsok a [README.hu.md](../README.hu.md) fájlban vannak dokumentálva.

## Könnyű webes csomagok

`cargo bundle-web path/to/game.rtvcsnap` lefordítja a könnyű WASM célpontot, és önálló statikus lejátszót készít a `dist/<snapshot-name>-web/` könyvtárba. A megadott snapshot `snapshot.rtvcsnap` vagy `snapshot.rtvcsnap.zip` néven kerül bele.

`cargo xtask bundle-web-skeleton` ugyanazt a lejátszót készíti el beágyazott snapshot nélkül, alapértelmezés szerint `dist/rtvc-web-skeleton/` alá. Példa explicit kimeneti könyvtárra:

```bash
cargo xtask bundle-web-skeleton dist/rtvc-snapshot-player
```

A release archívumok a teljes webes emulátort tartalmazzák `web/` néven. A normál böngészős emulátorfelülethez használd a `cargo xtask bundle-web-full [out-dir]` parancsot; a `bundle-web` és `bundle-web-skeleton` parancsok a könnyű, snapshot-specifikus oldalakhoz maradnak.
