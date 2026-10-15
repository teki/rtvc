# rtvc Implementációs és Használati Referencia

Ez a dokumentum a tárolóban található Rust emulátort írja le. Az implementációfüggetlen
gép-specifikációért lásd a [TVC Műszaki Referencia](tvc.md) dokumentumot.

## Tartalomjegyzék

- [Projektarchitektúra](#project-architecture)
- [Fordítási célok](#build-targets)
- [Gép végrehajtás](#machine-execution)
- [Videóemuláció](#video-emulation)
- [Hangemuláció](#sound-emulation)
- [Billentyűzetbemenet](#keyboard-input)
- [Média kezelése](#media-handling)
- [Parancssori BASIC-fordító](#command-line-basic-compiler)
- [Parancssori CAS-konverter](#command-line-cas-converter)
- [ROM betöltés és gyors rendszerindítás](#rom-loading-and-fast-boot)
- [Pillanatkép formátum](#snapshot-format)
- [Natív és webes felhasználói felület](#native-and-web-ui)
- [Hibakereső](#debugger)
- [ROM szimbólum adatbázis](#rom-symbol-database)
- [Konfiguráció és perzisztencia](#configuration-and-persistence)
- [Tesztelés és validáció](#testing-and-validation)

## Projektarchitektúra

Az `rtvc` egy Rust library crate natív, fej nélküli, könnyűsúlyú WebAssembly
és teljes webes frontenddel.

| Fájl | Felelősség |
| --- | --- |
| [src/z80.rs](../src/z80.rs) | Z80 végrehajtó mag |
| [src/bus.rs](../src/bus.rs) | CPU busz trait és lapos teszt busz |
| [src/mmu.rs](../src/mmu.rs) | TVC bankváltás és ROM elhelyezés |
| [src/emulator/crtc.rs](../src/emulator/crtc.rs) | közös 6845 CRTC számlálók és regiszterek (MC6845 és HD6845S változat) |
| [src/emulator/tvc_vid.rs](../src/emulator/tvc_vid.rs) | TVC videójel-generátor, televízió-vevő, megjelenítők |
| [src/tvc.rs](../src/tvc.rs) | TVC gép busz, időzítés, megszakítások, eszközök |
| [src/zx82.rs](../src/zx82.rs) | kezdeti Spectrum 48K memória, ULA, képkocka időzítés és teljes képkockás megjelenítő |
| [src/key.rs](../src/key.rs) | billentyűzetmátrix és host-billentyű adaptáció |
| [src/sound.rs](../src/sound.rs) | hangosztó, időzítő, DAC, PCM generálás |
| [src/cas.rs](../src/cas.rs) | CAS tároló kódolás és CAS-ból impulzus-intervallum konverzió |
| [src/emulator/basic.rs](../src/emulator/basic.rs) | TVC BASIC tokenizáló és visszafejtő |
| [src/tape.rs](../src/tape.rs) | kazetta szállítás és jelmintavételezés |
| [src/emulator/expansion.rs](../src/emulator/expansion.rs) | `SlotDevice` kártyafelület, `Card` enum, négyfoglalatos útválasztás és slot interruptok |
| [src/emulator/hbf.rs](../src/emulator/hbf.rs) | HBF kártya memória és regiszterek |
| [src/emulator/host_file.rs](../src/emulator/host_file.rs) | gazdagépes fájlkártya, portprotokoll, névleképezés és fájltárolók |
| [roms/RTVC-HOSTFILE.asm](../roms/RTVC-HOSTFILE.asm) | a gazdagépes fájlkártya saját fejlesztésű `FILE` eszköz ROM-ja |
| [src/fd1793.rs](../src/fd1793.rs) | FD1793 floppy vezérlő kétmeghajtós írási/olvasási támogatással |
| [src/emu.rs](../src/emu.rs) | gépválasztás, média, futási állapot |
| [src/machine.rs](../src/machine.rs) | explicit TVC/Zx82 gép határfelület és közös hibakereső műveletek |
| [src/ui.rs](../src/ui.rs) | közös natív/teljes-webes egui alkalmazás |
| [src/workspace.rs](../src/workspace.rs) | egyszerű/fejlesztői elrendezések |
| [src/debug_ui.rs](../src/debug_ui.rs) | integrált hibakereső panelek |
| [src/debugger.rs](../src/debugger.rs) | natív TCP hibakereső |
| [src/snapshot.rs](../src/snapshot.rs) | általános chunk olvasó/író |
| [src/tvc_snapshot.rs](../src/tvc_snapshot.rs) | TVC pillanatkép sorosítás |
| [src/wasm.rs](../src/wasm.rs) | könnyűsúlyú és teljes-webes kötések |
| [src/zx82_main.rs](../src/zx82_main.rs) | kísérleti natív/fej nélküli Zx82 futtató |

A CPU csak a `CpuBus` interfészt látja. A `FakeBus` lapos memóriát biztosít a CPU
tesztekhez; a `TvcBus` a valódi TVC memória és I/O viselkedést kínálja. Ez a Z80
validációt függetleníti a gépemulációtól.

## Fordítási célok

| Cél | Funkciók | Megjegyzések |
| --- | --- | --- |
| Natív asztali | alapértelmezett `native` | egui/eframe, cpal hang, fájlrendszer média, zip támogatás, TCP hibakereső |
| Natív parancssori segédprogramok | `cli-tools` alapértelmezett funkciók nélkül | lemez-, assembler-, BASIC-fordító-, CAS-konverter-, disassembler- és CAS–WAV segédprogramok asztali UI- és hangfüggőségek nélkül |
| Natív fej nélküli | alapértelmezett `native`, `--headless` CLI | gép ciklus és TCP hibakereső GUI nélkül |
| Integrált Zx82 | alapértelmezett `native` és `wasm-full` | Spectrum 48K állapot betöltés a közös alkalmazáson és hibakeresőn keresztül |
| Önálló Zx82 | alapértelmezett `native`, `cargo run --bin zx82` | fókuszált Spectrum mag futtató |
| Könnyűsúlyú web | `wasm,web-vid-simple` | kis wasm-bindgen API, JavaScript által birtokolt canvas és hang |
| Kompatibilitási könnyűsúlyú web | `wasm,web-vid-realistic` | ugyanaz az API; futásidejű videóválasztás elérhető marad |
| Teljes web | `wasm-full` | teljes egui felhasználói felület, böngésző fájlok, IndexedDB, AudioWorklet |

A [.cargo/config.toml](../.cargo/config.toml) minden `wasm32-unknown-unknown`
buildet 8 MiB-os veremmel linkel. Az alapértelmezett 1 MiB túlcsordul az
emulátorállapot felépítése közben, és a teljes webes vászon üres marad.

A könnyűsúlyú WASM cél szándékosan kizárja az egui, eframe, cpal, zip
és natív fájlrendszer kódot. A csak böngészőhöz tartozó függőségeknek webes
funkciók mögött kell maradniuk.

Rust edition 2024 van használatban, ami Rust 1.85-ös vagy újabb verziót igényel.

## Gép végrehajtás

Az `Emu` egy explicit `Machine` enumot birtokol. Az implementált változatok a `Tvc` és
`Zx82`; a közös ütemezési, framebuffer, bemeneti, töréspont, leképezett memória,
disassembly és léptetési műveletek ezen a határfelületen keresztül kerülnek kiosztásra.

A normál ütemező 62 500 T-állapotú host képkocka keretet használ:

1. végrehajt egy Z80 utasítást;
2. hozzáadja annak T-állapotait a gép órájához;
3. előreviszi a kazetta szállítást és a hangot ezekkel a T-állapotokkal;
4. előreviszi az átlapolt videót, ha az ki van választva;
5. tárolja az eszközmegszakításokat és meghívja a Z80 megszakítási útvonalat, ha elfogadásra kerül;
6. ellenőrzi a végrehajtási töréspontokat és az opcionális ROM nyomkövetési pontokat;
7. megáll a keretnél vagy egy hibakereső feltételnél.

A `debug_step_instruction()` ugyanazt az eszköz-előrehaladási útvonalat használja. A hibakereső
léptetés tehát megváltoztatja a videót, kazettát, hangot, megszakításokat és a gép idejét,
nem csak a CPU regisztereket.

A natív felhasználói felület folyamatosan kér újrarajzolást futás közben, de TVC
képkockákat 50 Hz-es valós idejű kapuzással generál. A gyorsabb host frissítések
újrafelhasználják az aktuális textúrát. Amikor az emuláció lemarad, a felhasználói felület
eldobja a hátralékot ahelyett, hogy több felzárkózó képkockát futtatna egy újrarajzolás során.

A `Zx82` a Spectrum ROM-ot egy rögzített 16 KiB ROM és 48 KiB RAM leképezéssel
futtatja, 69 888 T-állapotonként egy megszakítást kínál, és egy 352 x 296-os
framebuffert rajzol a bitmap és attribútum memóriából. A közös alkalmazás a host
billentyűket a nyolcszor ötös Spectrum mátrixra képezi le, és a Zx82-t a dokkoló
és TCP hibakeresőkön keresztül teszi elérhetővé. Mindkét `VidModel` érték megmarad, de a Zx82 jelenleg
mindkét választásnál egy teljes képkockát rajzol. Egyszerű 48K-s `.z80` 1-es, 2-es
és 3-as verziók támogatottak; a bővített gépi és periféria-függő állapotok
elutasításra kerülnek.

## Videóemuláció

A `VidModel` két futásidejű móddal rendelkezik.

### Átlapolt

Minden CPU utasítás után a `Vid::stream_some()` előreviszi a CRTC-t a
két T-állapotú karakteróra arány szerint. Karakterállapotot ír egy körkörös
folyamba, és a `render_stream()` úgy viselkedik, mint egy monitor, amely a HSYNC-re és
VSYNC-re reagál.

Ez a mód megőrzi a képkocka közbeni paletta, keret, mód, kezdőcím és CRTC
változásokat. A kurzor találat a közös megszakítást a megfelelő sugárpozíciónál tárolja;
az IRQ kiszolgálási idő szintén alkalmazásra kerül a videó előrehaladásra.

A monitor megjelenítő egy 608 x 288-as felületet állít elő. Szinkronra vár, alkalmazza
a várt TVC porch pozicionálást, és soronként 76 kimeneti karakterórát rajzol.
Ha az érvényes szinkron több host ütemre hiányzik, az rtvc egy fekete
szinkronvesztett felületet jelenít meg mozgó fehér csíkokkal, miközben folytatja az emulációt.

A natív `Tvc::new()` alapértelmezetten az Átlapolt módot használja.

### Gyors képkocka

A CPU a host képkocka keretig fut, majd a `Vid::draw_frame()` a teljes
608 x 288-as framebuffert megjeleníti az aktuális VRAM, paletta és CRTC állapot alapján.

Ez gyorsabb és egyszerűbb, de nem képes reprodukálni a képkocka közben végrehajtott
raszterváltozásokat. A könnyűsúlyú WASM konstruktorok alapértelmezetten a Gyors képkocka módot használják. A JavaScript meghívhatja
a `setVidModel("interleaved")` függvényt; a `simple` és `realistic` továbbra is elfogadott aliasok maradnak.

### Jelenlegi CRTC irányelv

Az rtvc implementálja a TVC port tükrözéseket és a TVC-kompatibilis CPU regiszter hozzáférést.
Az R12-R13-at olvashatóként/írhatóként, az R14-R15-öt olvashatóként/írhatóként, az R16-R17-et
csak olvashatóként kezeli, az írásvédett regiszterek olvasása pedig `0xFF` értéket ad.

A látható MC6845 kurzor alak/villogás nem kerül kirajzolásra, mert a TVC szoftverek általában
a kurzor kimenetet időzítésre használják, a látható kurzort pedig bitmap memóriában rajzolják ki.
Az átlapolás, kijelzésengedélyezési eltolás, kurzor eltolás és fényceruza stroboszkóp korlátozott
vagy elhalasztott marad.

## Hangemuláció

A `SoundTimer` a CPU ciklusok alapján halad előre, és modellezi:

- a 12 bites programozható periódust;
- a négybites követő számlálót;
- a számláló 3. bitjét mint oszcillátor kimenet;
- az amplitúdó regisztert és a közvetlen DAC módot;
- a közös hangmegszakítást;
- fázis újraindítást a `0x5B`/`0x5F` olvasásakor.

A mag mono 44,1 kHz-es `f32` PCM-et generál, és egy kis DC-blokkoló
felüláteresztő szűrőt alkalmaz az AC-csatolt kimeneti útvonal közelítéséhez. A függőben lévő PCM
egy másodpercre van korlátozva.

A natív kimenet a `cpal`-t használja, a monót az összes host csatornára másolja, átalakítja a
kiválasztott mintaformátumra, és könnyűsúlyú újramintavételezést végez, ha a 44,1 kHz nem
elérhető. A webes kimenet `AudioWorklet`-et használ; a hang egy böngésző
felhasználói gesztus után indul el.

A `Tvc::sound_sample_rate()` jelentési a mintavételi frekvenciát, a `Tvc::take_audio_samples()`
pedig kiolvassa a generált mintákat. A könnyűsúlyú WASM API ezzel egyenértékű metódusokat tesz elérhetővé.

## Billentyűzetbemenet

A mag az aktív-alacsony 10 x 8-as TVC mátrixot tárolja. A host adaptáció külön van választva:

- a natív bemenet az egui fizikai billentyűazonosítóit részesíti előnyben, és szöveges eseményeket használ
  a kiosztásfüggő karakterleképezéshez;
- a teljes web a `KeyboardEvent.code`-ot használja a fizikai azonosításhoz és a
  `KeyboardEvent.key`-t a generált karakterekhez;
- az AltGr külön van nyomon követve a hagyományos Alt-tól;
- a szintetizált TVC Shift kompenzál, amikor a host és a TVC kiosztás eltérő
  módosító állapotokat igényel;
- a billentyű felengedése törli az összes módosító-leképezés jelöltet a beragadt billentyűk megelőzése érdekében;
- a fókuszvesztés, a canvas elhomályosodása és a láthatóság elvesztése felengedi a lenyomva tartott billentyűket.

Fejlesztői módban a felhasználónak a Képernyő panelre kell kattintania a TVC bemenet rögzítéséhez.
Az Escape, a fókuszvesztés, a panel elrejtése vagy egy másik panelre kattintás felengedi a rögzítést.
Az Egyszerű mód közvetlenül irányítja a billentyűzetbemenetet.

## Média kezelése

### Kazetta lejátszás

A csatlakoztatott CAS fájlok CPU ciklusokban mért impulzus-intervallumokká konvertálódnak. A szalag pozíció
csak akkor halad előre, ha a lejátszás aktív és a motor bit be van állítva. A `0x59` port
az aktuális intervallum szintet mintavételezi.

A `cargo run --bin rtvc-cas2wav -- input.cas output.wav [tape-name]` kompatibilis
előjel nélküli 8 bites mono 44,1 kHz-es WAV kimenetet ír.

### Közvetlen kazetta befecskendezés

Az opcionális gyors befecskendezési útvonal egy emulátoros kényelmi funkció, nem TVC hardver:

1. elmenti az aktuális MMU leképezést;
2. beállítja a `0xB0` leképezést, hogy a RAM-ot az összes CPU ablakon keresztül láthatóvá tegye;
3. kihagyja a 144 bájtos CAS fejlécet;
4. a hasznos adatot a `0x19EF` BASIC programcímre másolja;
5. ha a hasznos adat tokenizált BASIC programmal kezdődik, beállítja a `TEXT`/`CHAIN`/`TOP`
   mutatókat (`1722H`/`1724H`/`1726H`), hogy a `LIST` és a `RUN` lássa a programot;
6. visszaállítja az előző leképezést.

Nem nulla CAS autostart bájt esetén a `-i` / `--inject` a befecskendezés után begépeli a `RUN`-t,
ugyanúgy, mint a Gamebase. A Tape menü Inject művelete a promptnál hagy. Sok gépi kódú program
tartalmaz egy kis BASIC betöltőt, amely a `0x1B00` közelében hív kódot.

### Floppy és archívumok

A DSK bájtok egy HBF kártya A: (0) és B: (1) meghajtóihoz csatlakoztathatók.
Több HBF kártya esetén az `Emu` kártyánként két lemezegységet számoz slot
sorrendben: a 0. és 1. egység az első kártya A: és B: meghajtója, a 2. és 3. a
másodiké, és így tovább. A Lemez menü és az állapotsor ilyenkor `ex<slot> A:`
formában nevezi meg az egységeket. A lemez
modell FAT12 boot szektor geometriát elemez, és támogatja a mellékelt szoftverek által
igényelt vezérlő útvonalakat, beleértve a visszaállítást, keresést, befelé léptetést, kifelé léptetést,
szektor olvasást, szektor írást, cím olvasást és kényszerített megszakítást. Az FD1793 viselkedés
még nem egy teljes cikluspontos implementáció.

Az `rtvc-dsk` segédprogram képes megvizsgálni olyan régi TVC/MSX-DOS stílusú FAT12 lemezképeket, amelyek
boot szektora nem tartalmazza a PC-s `55 AA` aláírást, és a későbbi BPB bájtokat boot
kódra használja fel.

A parancssor legfeljebb két `-d` argumentumot fogad el: az elsőt az A: meghajtóra, a
másodikat a B: meghajtóra csatlakoztatja. A Lemez menü meghajtónként Megnyitás, Új 360K lemez,
Új 720K lemez, Mentés és Kivétel alműveleteket kínál az A: és B: meghajtókhoz.
Az új lemezek TVC-kompatibilis boot szektor bájtokkal rendelkező FAT12 lemezképekként formázódnak.
A meglévő host elérési útról betöltött natív `.dsk` fájlok automatikusan visszaíródnak
az emulált szektor írások után. A böngészőből betöltött lemezek, ZIP tagok
és a nem mentett üres lemezek memóriában maradnak, amíg a felhasználó a Lemez mentése lehetőséget nem választja.

A natív fordítások képesek ZIP archívumokat megnyitni és rekurzívan kiválasztani a CAS vagy DSK tagokat.
A könnyűsúlyú WASM mag nem tartalmaz zip támogatást.

### Bővítőkártyák

Minden slot egy opcionális `Card` értéket (`Hbf` vagy `HostFile`) tartalmaz,
amely a `SlotDevice` trait-et valósítja meg: azonosítóbitek, EXT ablak
olvasás/írás, 16 bites portcímes I/O, interruptkérés, reset és pillanatkép
állapot. Bármely kártya bármely slotba tehető, és egy fajtából több is lehet.
A `0x58-0x5B` portok 7. bitje slotonként engedélyezi vagy törli az interruptot;
a `0x59`/`0x5D` 0-3. bitje az engedélyezett kártyakéréseket mutatja.

### Gazdagépes fájlkártya

A `hostfile` kártya (azonosító `11`) `FILE` eszközt ad a TVC programoknak, natív
módban egy gazdagépes mappával, a weben a böngésző tárolójával. ROM-ja, a
[RTVC-HOSTFILE.asm](../roms/RTVC-HOSTFILE.asm), az rtvc saját kódja; egy
egységteszt újrafordítja a forrást, és összeveti a `RTVC-HOSTFILE.ROM`
fájllal. Az ötlet az ep128emu `tvcfileio.s` fájljából jön, de kód nincs
átvéve: az rtvc kártyaportokat használ, a CAS kezelés Rustban van.

Ha a kártya alapértelmezett eszközként van beállítva, az init rutin a
`0x0B05`/`0x0B0D` bájtokba `0x80 | slot` értéket ír, így a BASIC `LOAD` és
`SAVE` a kártyát használja a kazetta helyett. A `.cas` fájlokból a 128 bájtos
tárolóblokk olvasáskor lekerül, íráskor újra elkészül; más kiterjesztésű
fájlok nyers bájtok. A TVC név kisbetűssé válik, a nem biztonságos karakterek
`_` jelre cserélődnek, és bemenetnél először `<név>.cas`, majd `<név>`
keresődik, kis- és nagybetűtől függetlenül. A portprotokoll, a parancsok és a
hibakódok részletes leírása az
[angol referenciában](../info/rtvc.md#host-file-card) található.

### Gamebase

A Gamebase indítások betöltik a beágyazott tiszta VT-DOS rendszerindító pillanatképet, kiválasztják a
megfelelő TVC 1.2 VT-DOS gépet, csatlakoztatják vagy befecskendezik a médiát, elindítják az emulációt, és
beírják a `RUN`-t CAS esetén vagy a `LOAD "*"`-ot DSK esetén.

A natív vagy a teljes webes ablakra húzott fájl ugyanezt az indítási utat
használja: a `.cas` befecskendeződik, a `.dsk` az A: meghajtóba kerül. ZIP
esetén az első `.rtvcsnap` tag pillanatképként töltődik be, ennek hiányában az
első `.cas` vagy `.dsk` tag indul. A `.rtvcsnap` és `.z80` fájlok újraindítás
nélkül, állapotként töltődnek be.

## Parancssori BASIC-fordító

Az `rtvc-basic input.bas -o output.cas` számozott TVC BASIC forrást tokenizál a
BASIC 1.2 kulcsszótábla alapján, és CAS képet ír. Az alapértelmezett fejléc a
BASIC `SAVE` mentésnek felel meg. A `--format bin` nyers payloadot ír, a
`--auto` az autostart bájtot állítja, a `-` a szabványos bemenetről olvas.
A fordító elutasítja a szabványos 64K TVC BASIC 42 256 bájtos mérethatárát
meghaladó programokat; a határ a lezáróbájtot is tartalmazza. A futásidejű
foglalások és a gép memóriabeállításai kisebb programot tehetnek szükségessé.

A memóriabeli sorformátumot és a tokenizálási szabályokat a
[basic.md](basic.md#tokenizalt-programformatum) ismerteti.

## Parancssori CAS-konverter

Az `rtvc-tocas input.bas helper.asm program.toml` egy vagy több `.bas`, `.asm`
és `rtvc-asm-v1` TOML forrást fordít melléjük írt `.cas` fájlokká. A `.bas`
bemenetek az `rtvc-basic`, az `.asm` bemenetek az `rtvc-asm --format cas`
útvonalát használják. A TOML szegmenseket cím szerint rendezi, `19EFH`-tól
a legmagasabb kizáró végéig lapítja, a hézagokat nullával tölti. Az első
szegmens tokenizált BASIC program vagy indító stub `19EFH`-n; a `4000H`-ra
helyezett BASIC nem kazetta. A kimenet a forrás útvonala `.cas`
kiterjesztéssel. Az egyéb kiterjesztésű fájlokat kiírja és kihagyja.

## ROM betöltés és gyors rendszerindítás

A `TvcMmu::add_rom()` az ismert ROM fájlneveket SYS és EXTH területre képezi le; az ismeretlen ROM
bájtok cartridge képként kezelődnek.

Az opcionális Gyors rendszerindítás beállítás védett, visszafordítható javításokat alkalmaz az ismert
BASIC 1.2 és 2.2 ROM bájtsorozatokra. Lecseréli a kétmintás RAM tesztet
nullázásra, és kihagyja a firmware rendszerindító képernyőt, miközben megőrzi a BASIC által
elvárt hívási szerződést. A javítások csak akkor kerülnek alkalmazásra, ha a fájlnév és az
eredeti bájtok is egyeznek, és a beállítás kikapcsolása visszaállítja az eredeti bájtokat.

Ez a funkció szándékosan itt kerül dokumentálásra, nem pedig a hardver
referenciában, mivel módosítja a firmware viselkedését.

## Pillanatkép formátum

A pillanatképek a következővel kezdődnek:

```text
RTVCSNAP
u16 version
```

A fennmaradó rész egy little-endian chunk folyam:

```text
u8[4] chunk_id
u32   payload_length
u8[]  payload
```

Az ismeretlen chunkok figyelmen kívül maradnak. A 2-es verzió a következőket használja:

| Chunk | Tartalom |
| --- | --- |
| `META` | plus modell, videó modell, gép óra, képkocka állapot |
| `CPUZ` | Z80 regiszterek, megszakítás és HALT állapot |
| `MMU ` | RAM, modellnek megfelelő VRAM, lapozási állapot |
| `VID ` | TVC mód, CRTC állapot, paletta, keret |
| `SLT0`-`SLT3` | foglalt slotonként: kártyafajta (`1` HBF, `2` gazdagépes fájlkártya) és kártyaállapot |
| `HBF ` | régi formátum: az első HBF kártya RAM és vezérlő állapota; továbbra is betöltődik |
| `BUS ` | megszakítás tároló, bővítő kiválasztás, kazetta és hang állapot, slot interrupt engedélyek |
| `EMUT` | opcionális felhasználói felület gépválasztás és ROM verzió |
| `EMUI` | opcionális kiválasztott média hivatkozások |

A billentyűzet állapot, naplók, függőben lévő frontend PCM, ROM bájtok és lemez bájtok nem
kerülnek sorosításra. A wrapper a kiválasztott gépet a normál ROM
erőforrásokból építi újra, és az elérhető lemez médiát fájlnév alapján csatolja vissza.

A 2-es verzió szándékosan elutasítja az 1-es verziójú pillanatképeket.

A natív mentés/betöltés támogatja a nyers `.rtvcsnap` és a ZIP-be csomagolt
`.rtvcsnap.zip` formátumokat. A verziókövetett
[boot12dos.rtvcsnap.zip](../snapshots/boot12dos.rtvcsnap.zip) egy stabil
rendszerindítás utáni fixture és a Gamebase indítás alapja.

A `cargo bundle-web <snapshot>` egy könnyűsúlyú statikus pillanatkép lejátszót hoz létre.
A `cargo xtask bundle-web-skeleton` beágyazott pillanatkép nélkül építi a lejátszót.
A `cargo xtask bundle-web-full` a teljes webes felhasználói felületet építi.

## Natív és webes felhasználói felület

### Módok és panelek

Az Egyszerű mód a 4:3-as TVC képernyőt mutatja. A Fejlesztői mód az `egui_dock`-ot használja; az
alapértelmezett elrendezés a Képernyőt az I/O Napló fölé helyezi.

A Hibakereső Elrendezés megnyitja a CPU, Disassembly, Memória, Töréspontok, ROM Szimbólumok,
Események, Képernyő és I/O Napló paneleket. A panel megjelenítése nem viszi előre az emulációt.
A memória/disassembly tartományok és eseményelőzmények korlátozva vannak.

### Perzisztencia

A natív beállítások az `rtvc.toml` fájlban tárolódnak, amely a munkakönyvtárban
majd a futtatható fájl mellett kerül keresésre. A verziózott dokkoló elrendezés külön
`rtvc-workspace.json` fájlként kerül tárolásra.

A teljes web a kis beállításokat a `localStorage`-ban, a legutóbbi média bájtokat az
IndexedDB-ben, a munkaterületet pedig `rtvc_workspace_v1` alatt tárolja. A könnyűsúlyú WASM nem rendelkezik
egui munkaterület függőséggel.

## Hibakereső

### Integrált hibakereső

A dokkoló hibakereső az aktív `Machine`-en keresztül, az `Emu`-n át működik, és elérhető
natív és teljes webes környezetben. Mind a TVC, mind a Zx82 biztosít futtatás/szünet/újraindítás, utasítás
léptetés, korlátozott futás-IRQ-ig, leképezett memória, disassembly és töréspontok funkciókat. A nyers
bankok, ROM szimbólumok, nyomkövetési tereptárgyak és I/O naplók TVC-specifikusak maradnak.

### TCP hibakereső

A natív GUI és fej nélküli módok újsorral tagolt JSON-t tesznek közzé localhost-on:

```bash
cargo run --bin rtvc -- --port 8089
cargo run --bin rtvc -- --headless --port 8080
```

| Parancs | Cél |
| --- | --- |
| `status` | CPU regiszterek, óra, futás/HALT állapot |
| `stats` | gördülő host-idő FPS |
| `step` | egy vagy több teljes gépi utasítás végrehajtása |
| `continue`, `pause`, `reset` | végrehajtás vezérlés |
| `breakpoint_add`, `breakpoint_remove`, `breakpoint_list` | töréspontok |
| `read_memory` | leképezett memória vagy nyers `u0`-`u3`, `vid0`-`vid3`, `sys`, `cart`, `exth` |
| `write_memory` | bájtok írása az aktív gép leképezett CPU címterébe |
| `disassemble`, `assemble` | Z80 fejlesztői eszközök |
| `save_snapshot`, `load_snapshot` | pillanatkép fájlok |
| `save_screenshot` | 4:3 PNG |
| `key` | billentyű lenyomás/felengedés vagy begépelt karakter |
| `close_app` | normál alkalmazás leállítás |

A kérések és válaszok soronként egy JSON objektumot tartalmaznak. Egy futó emulátor aszinkron módon
`{"event":"breakpoint","pc":...}` eseményt bocsát ki, amikor egy töréspont elérésre kerül.

Az interaktív kliens a [scripts/rtvc_debug.py](../scripts/rtvc_debug.py).

## ROM szimbólum adatbázis

A BASIC 1.2 végrehajtási tereptárgyak, hívható rutinok és adatok ROM-képenként
egy szimbólumfájlban vannak:
[TVC12_D4.64K.symbols.json](../roms/TVC12_D4.64K.symbols.json),
[TVC12_D3.64K.symbols.json](../roms/TVC12_D3.64K.symbols.json) és
[TVC12_D7.64K.symbols.json](../roms/TVC12_D7.64K.symbols.json).
A `cargo xtask rom-symbols` a [roms/](../roms/) önálló listáiból állítja elő őket;
a címeket nem szabad kézzel szerkeszteni. A címkék címei a valódi assemblerből
származnak, a gondozott leírások pedig `(bank, offset)` kulcs alapján egyesülnek.

Egy CPU cím önmagában nem stabil ROM azonosító, mert a SYS és EXTH
átfedő CPU tartományokat foglalhat el. A bejegyzések kulcsa a fizikai bank és az
eltolás, a hibakereső pedig az élő CPU címet az MMU-n keresztül oldja fel erre a
kulcsra, így minden tereptárgy abban a lapozási nézetben is jelez, amelyben a CPU
eléri. A hibakereső csak akkor annotál, ha a releváns bank éppen le van képezve.
A szimbólumlista disassembler és töréspont művelete azt a CPU-címet nyitja meg,
ahol a bájt a jelenlegi térképen látszik, és kikapcsolva marad, ha nem látszik.
A képenkénti megjegyzések ugyanazt a bankot és eltolást használják, ezért az
`E000H-FFFFH` ablak a ténylegesen kiválasztott SYS vagy EXTH jegyzetet mutatja.

A `usage` értékek: `trace`, `call` és `data`. A `call` címke nem jelent ABI
garanciát; a szoftvernek továbbra is teljesítenie kell a lapozási, regiszter, BASIC állapot és munka
változó követelményeket. A BASIC 2.2-höz külön illesztett adatbázisra van szükség, nem
pedig egy konstans címeltolásra.

## Konfiguráció és perzisztencia

A Gép menü Preset almenüje öt TVC előbeállítást tart meg: 64K+ BASIC 1.2 vagy
2.2, a HBF kártyával a 0. bővítőhelyen, és a három kártya nélküli gépet (64K
1.2, 64K+ 1.2, 64K+ 2.2). Az alatta lévő sorok a memóriát (64K vagy 64K+) és a
rendszer-ROM párt (BASIC 1.2 vagy 2.2, a SYS és az EXTH együtt) írják felül; a
Cartridge almenü a programmodult (`roms/*.crt` vagy üres), az Expansion almenü
pedig a négy bővítőcsatlakozó mindegyikére külön a benne lévő kártyát
választja: nincs, VT-DOS 1.2 HBF kártya vagy gazdagépes fájlkártya. Több HBF kártya is lehet. Az `ex0` hely az `0x10-0x1F` I/O
tartomány, a CH1 csatlakozó.

Amíg a gép egy előbeállításnak megfelel, az marad kijelölve. Minden más
összeállítás az azonosítójával jelenik meg, például
`64ktvc12+ex1:vtdos12+cart:pack_man`. Minden kártya egy `+ex<slot>:<kártya>`
részt ad hozzá (`vtdos12` vagy `hostfile`). Az `rtvc.toml` a gépazonosító
mellett a `host_file_dir` (natív mappa) és a `host_file_default` (alapértelmezett
eszköz, alapból `true`) kulcsot is tárolja. A programmodul 16 KiB-os `.crt` fájl
a `roms/` könyvtárban. A BASIC automatikusan elindítja azt a modult, amelynek
első négy bájtja `MOPS`; ilyen a `pack_man.crt` és a `VT-DOS12-CART.crt`.
A HBF kártya és a VT-DOS programmodul külön van: a kártya a BASIC
lemezparancsait szolgálja, a modul a `DOS` parancs CART bankja. A lemezképek
ahhoz a csatlakozóhoz kötődnek,
amelyikben a kártya van.

A natív és a teljes webes alkalmazás ezt az azonosítót menti az `rtvc.toml`
fájlba (vagy a böngésző helyi tárolójába), a videó modell, a gyors
rendszerindítás és a visszaállítható média hivatkozások mellett. A régebbi
előbeállítás-azonosítók, például a `64k-plus-1.2-vtdos`, továbbra is
betöltődnek. A pillanatképek ugyanezt a gépet rögzítik. A programmodul- és
helyválasztás előtti pillanatkép a megfelelő előbeállításként töltődik, a
HBF kártyával a 0. helyen és üres programmodul-foglalattal.

A natív alkalmazás a futásidejű ROM és program eszközöket először az aktuális
munkakönyvtárban, majd a futtatható fájl mellett keresi. A csomagolt macOS alkalmazások
és a kibontott kiadási archívumok így működnek anélkül, hogy függnének az indítási
könyvtártól.

## Tesztelés és validáció

A karbantartott parancsok és platform ellenőrzőlista a
[a fejlesztési skillben](../.agents/skills/development/SKILL.md) található.

A gyors CPU validációs útvonal az 1334 esetből álló FUSE tesztkészlet:

```bash
cargo run --bin fuse_test
```

A ZEXDOC/ZEXALL szigorúbb és lassabb:

```bash
cargo run --bin zex_test
```

A keresztcélú változtatásoknak legalább a natív, a könnyűsúlyú WASM,
az alternatív könnyűsúlyú WASM, a teljes-webes WASM, az xtask és a könnyűsúlyú
függőségi fa validálását el kell végezniük. A hardver viselkedés változásainál frissíteni kell a
[TVC Műszaki Referencia](tvc.md) dokumentumot; a tároló architektúra, formátumok vagy felhasználói felület
változásainál ezt a dokumentumot kell frissíteni.
