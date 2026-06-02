# Videovezérlő (MC6845 CRTC)

Ez a dokumentum a TVC videovezérlőjének architektúráját foglalja össze. A kapcsolódó Rust implementáció: [src/vid.rs](../src/vid.rs). A memóriakezelés részletei: [mmu.md](mmu.md).

## Áttekintés

A TVC **Motorola MC6845 CRTC** chipet használ a raszteres kijelzés időzítéséhez és a videomemória-címek generálásához.

Az MC6845 önmagában nem tartalmaz grafikus memóriát, karaktergenerátor ROM-ot vagy pixelkódolót. Számlálóalapú időzítő, amely ezeket adja:

1. **Memóriacímek (MA0-MA13)** a videomemória olvasásához.
2. **Rasztercímek (RA0-RA4)** az aktuális karakter soron belüli scanline-hoz.
3. **HSYNC és VSYNC** szinkronimpulzusok.
4. **Display Enable (DE)** jel, amikor a sugár az aktív képtartományban van.
5. **Cursor** jel, amikor az aktuális karaktercím egyezik a kurzorral.

A TVC ezt saját pixeldekódoló logikával, palettaregiszterekkel és interruptgenerátorral egészíti ki.

## Rendszeridőzítések

- **CPU órajel**: 3 125 000 Hz.
- **Karakteróra (CCLK)**: 1 562 500 Hz.
- **Arány**: 2 CPU ciklus / karakteróra.
- **Képfrekvencia**: 50 Hz.
- **CPU ciklus/képkocka**: 62 500.
- **PAL scanline**: a beállítások alapján 314 sor.

## MC6845 regiszterek

A CPU két portra írja a CRTC-t:

- `0x70`: címregiszter, kiválasztja az R0-R17 regisztert.
- `0x71`: adatregiszter, a kiválasztott regiszterbe ír.

Fontos TVC alapértékek:

| Reg | Név | Egység | TVC alap |
|:---:|---|---|---|
| R0 | Horizontal Total | karakter | 99 |
| R1 | Horizontal Displayed | karakter | 64 |
| R2 | Horizontal Sync Position | karakter | 75 |
| R3 | Sync Widths | kombinált | `0x32` |
| R4 | Vertical Total | karaktersor | 77 |
| R5 | Vertical Total Adjust | scanline | 2 |
| R6 | Vertical Displayed | karaktersor | 60 |
| R7 | Vertical Sync Position | karaktersor | 66 |
| R8 | Interlace & Skew | flag | 0 |
| R9 | Max Scan Line Address | scanline | 3 |
| R10-R11 | Cursor start/end | scanline | 3 |
| R12-R13 | Start address | cím | 0 |
| R14-R15 | Cursor address | cím | `0x0EFF` |
| R16-R17 | Light pen | read-only cím | mentett MA |

Scanline szám:

$$\text{Scanlines} = (\text{R4} + 1) \times (\text{R9} + 1) + \text{R5} = 314$$

## Videomemória-címfordítás

Az MC6845 lineáris címeket generál, de a TVC saját interleaving logikával képezi a 16 KB videó RAM fizikai címét.

Legyen:

- `ma`: a CRTC 12 bites karaktercíme.
- `rl`: a karakter soron belüli 5 bites rasztersor index.

```text
Generated Address Bits (14 bits):
[A13 A12 A11 A10 A9  A8 ]  [A7  A6 ]  [A5  A4  A3  A2  A1  A0 ]
  \___________________/      \____/     \___________________/
      ma[6..11] << 2        rl[0..1]         ma[0..5]
```

Referencia formula:

```text
addr = ((ma & 0x0fc0) << 2) | ((rl & 0x03) << 6) | (ma & 0x003f)
```

## Grafikus és színmódok

A TVC három fő képmódot támogat:

| Mód | Felbontás | Színek | Bit/pixel |
|---|---:|---:|---:|
| 2 szín | 512x240 | 2 | 1 |
| 4 szín | 256x240 | 4 | 2 |
| 16 szín | 128x240 | 16 | 4 |

A CRTC 64 karaktert jelenít meg soronként. A mód határozza meg, hogy egy videó bájtból hány pixel és milyen palettaindex lesz.

## Paletta és RGB formátum

A TVC színei IGRB jellegű 4 bites formátumot használnak: intenzitás, zöld, piros, kék. A keretszín és a palettaregiszterek portokon keresztül állíthatók.

## Interruptok

A CRTC kurzoregyezése interruptot generálhat. Az interleaved modellben ez az esemény CPU ciklusra igazítva történik, hogy az időzítésérzékeny programok az utolsó pixelhez közeli interruptból rajzolhassanak.

## Renderelési architektúra

Két ütemezési mód létezik:

### Interleaved / streaming mód

Minden CPU utasítás után a videómodell is megkapja az eltelt ciklusokat. Ez pontosabb, mert a CRTC és a CPU együtt halad, és az interruptok időzítése jobban közelíti a hardvert.

### Fast frame mód

A CPU egy képkockányi időt fut, majd a videó egyben rendereli a teljes framebuffer tartalmát. Ez gyorsabb és egyszerűbb, webes snapshot-lejátszáshoz hasznos.

### Szinkronvesztés

Ha az interleaved mód több host tick alatt sem kap szinkronizált CRTC képkockát, a UI nem vár végtelenül. Fekete monitorfelületet és mozgó fehér csíkokat jelenít meg, amíg újra nem zár a szinkron.
