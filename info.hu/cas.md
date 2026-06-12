# Kazettatámogatás (`.cas`)

Ez a dokumentum a TVC `.cas` kazettafájlok kezelését írja le az `rtvc` emulátorban.

## Nagysebességű közvetlen RAM-betöltés

Az emulátor használhat **közvetlen RAM-injektálást** (HLE betöltés). Ez kihagyja az eredeti gép lassú kazettaolvasó rutinjait, és a programot azonnal betölti.

### `.cas` fájlszerkezet

Egy TVC kazettakép 144 bájtos fejlécből és a nyers programadatokból áll:

1. **Fejléc (0-143)**: metaadatok, fájltípus, hossz és név.
2. **Típusjelzők**: például a `0x81` offseten lévő fájltípus.
3. **Payload (144-től)**: a program vagy BASIC adat.

### Betöltési logika

A médiabetöltési útvonal a [src/tvc.rs](../src/tvc.rs), [src/emu.rs](../src/emu.rs) és [src/wasm.rs](../src/wasm.rs) fájlokban található.

```text
if extension == ".cas":
    aktuális MMU map mentése
    MMU váltás teljes RAM mapre: 0xB0
    a 144 bájtos fejléc utáni payload másolása RAM-ba 0x19EF címtől
    korábbi MMU map visszaállítása
```

Az `0x19EF` a TVC BASIC programterületének alapértelmezett kezdőcíme (`TXTTAB`).

## Kazettamotor és folyamatjelző

A kazettamotor vezérlése a csak írható `05H` port 6. és 7. bitjén történik. Ha
valamelyik motorbit magas, az emulált kazettatranszport pozíciója halad; mindkét
bit alacsony állapotában megáll.

Aktív lejátszáskor az alsó állapotsor százalékosan mutatja a generált
szalagjelben elfoglalt pozíciót. A százalék csak bekapcsolt motor mellett halad.

## WAV/kazetta jelgenerálás

A [src/cas.rs](../src/cas.rs) és [src/cas2wav.rs](../src/cas2wav.rs) a CAS képből TVC-kompatibilis hullámformát készít. A kimenet 44,1 kHz-es unsigned 8 bites PCM, a TVC 3,125 MHz-es CPU-órájához igazított impulzushosszokkal.

| Jel | Darab | Magas ciklus | Alacsony ciklus | Megjegyzés |
|---|---:|---:|---:|---|
| Csend | névleges másodpercenként 22 052 minta | N/A | N/A | középszintű jel |
| Előhang | 9 | `638` | `638` | pilot/preamble |
| Sync | 17 | `1205` | `1205` | blokk szinkron |
| Bit 0 | 11 | `779` | `779` | nulla bit |
| Bit 1 | 8 | `567` | `567` | egyes bit |

## TVC CRC

A TVC ROM egyedi, bitenként számolt 16 bites CRC-t használ.

```text
crc = 0

update_crc(bit):
  bh = high byte of crc
  al = 0x80 when bit is 1, otherwise 0x00
  carry = ((al xor bh) bit 7) != 0
  if carry:
    crc = crc xor 0x0810
  crc = (crc << 1) & 0xffff
  if carry:
    crc = (crc | 1) & 0xffff
```

A bájtok **legkisebb helyiértékű bittel kezdve** kerülnek a bitfolyamba:

```text
write_byte(byte, calculate_crc):
  for bit_index in 0..8:
    bit = (byte >> bit_index) & 1
    write_bit(bit)
    if calculate_crc:
      update_crc(bit)
```

## Blokkszerkezet

### Fejlécblokk

- 2 másodperc csend.
- 10 240 előhang-impulzus.
- 1 sync impulzus.
- adatbájtok: `0x00`, CRC reset, `0x6A`, fejlécblokk azonosító, fájljellemzők, név, típus, méret, kitöltők, CRC.
- 5 záró előhang-impulzus.

### Adatblokk

- 1 másodperc csend.
- 5 120 előhang-impulzus.
- 1 sync impulzus.
- blokkfej: `0x00`, CRC reset, `0x6A`, adatblokk azonosító, fájljellemzők és szektorszám.

### Adatszektorok

Minden szektor tartalmazza a szektorszámot, méretbájtot, payload adatot, kitöltőbájtot és CRC-t. Teljes 256 bájtos szektornál a méret `0`, részleges utolsó szektornál a tényleges maradék méret.

## Integráció az emulátorba

Alacsony szintű kazettaemulációhoz a `.cas` fejlécből intervallistát lehet generálni Z80 ciklusokban. A TVC busz a kazetta bemeneti port olvasásakor ebből mintavételezi az aktuális jelszintet.

I/O viselkedés:

- `0x59` olvasás: a kazettajel bitje a 5. biten jelenik meg, ha a motor és a lejátszás aktív.
- `0x50 - 0x57` olvasás vagy írás: a kazetta kimeneti flip-flop váltása.
- `0x05` írás: a 6-7. bit vezérli a kazettamotort; a szállítási ciklus csak bekapcsolt motor mellett halad.
