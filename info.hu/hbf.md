# High Boot Floppy (HBF) kártya és FD1793 vezérlő

Ez a dokumentum nyelvfüggetlen áttekintés a TVC HBF floppy bővítőkártyájáról és a Western Digital FD1793 FDC emulációjáról. A kapcsolódó Rust kód: [src/hbf.rs](../src/hbf.rs) és [src/fd1793.rs](../src/fd1793.rs).

## Áttekintés

A High Boot Floppy (HBF) kártya hardveres bővítőkártya, általában a 0. kártyafoglalatban. Floppy boot támogatást ad a TVC-hez.

Fő részei:

1. **16 KB boot ROM**, benne TVC-DOS.
2. **4 KB saját RAM**, amelyet a TVC-DOS pufferként használ.
3. **Western Digital FD1793** floppy vezérlőchip, legfeljebb négy 5.25" vagy 3.5" meghajtóhoz.

A gépmag a kártyát a memória- és I/O buszra csatolja, és akkor továbbít hozzá olvasást/írást, amikor a bővítőszegmens aktív.

## HBF memória-leképezés

Ha a fő MMU az **EXT** bankot a 3. lapra (`0xC000 - 0xFFFF`) kapcsolja, a HBF kártya a lap alsó 8 KB-os tartományában válaszol (`0xC000 - 0xDFFF`, relatív `0x0000 - 0x1FFF`).

```text
EXT tér (0xC000 - 0xDFFF):
+-----------------------------------+-----------------------------------+
|   4 KB ROM lap (0xC000-0xCFFF)    |   4 KB saját RAM (0xD000-0xDFFF) |
|        (ROM0..ROM3 közül)         |                                   |
+-----------------------------------+-----------------------------------+
 0x0000                              0x1000                              0x1FFF
```

Az alsó 4 KB a kiválasztott ROM-lap, az ide írás ignorálódik. A felső 4 KB saját RAM, olvasható és írható.

## HBF I/O regiszterek

Slot 0 portok: `0x10 - 0x1F`, slot 1 portok: `0x20 - 0x2F`. A port offset `port_number & 0x0F`.

| Offset | R/W | Cél | Leírás |
|:---:|:---:|:---:|---|
| **0** | R | FDC | státuszregiszter, törli az INTRQ-t |
| **0** | W | FDC | parancsregiszter |
| **1** | R/W | FDC | sávregiszter |
| **2** | R/W | FDC | szektorregiszter |
| **3** | R/W | FDC | adatregiszter, olvasás törli a DRQ-t |
| **4** | R | FDC | gyors státusz: INTRQ bit 0, DRQ bit 7 |
| **4** | W | FDC | paraméterregiszter: meghajtó és oldal választása |
| **8** | W | HBF | ROM-lap választó: 4-5. bit választja a 4 KB-os ROM bankot |

### FDC paraméterregiszter (4-es port írás)

```text
Bit: [  7   ]  [  6   ]  [  5   ]  [  4   ]  [  3   ]  [  2   ]  [  1   ]  [  0   ]
     [ Side ]  [ MON  ]  [ DDEN ]  [ HLD  ]  [ DS3  ]  [ DS2  ]  [ DS1  ]  [ DS0  ]
```

- **Side**: oldal/fej választás.
- **MON**: motorvezérlés.
- **DDEN**: dupla sűrűség.
- **HLD**: fejbetöltés állapota.
- **DS0-DS3**: aktív meghajtó. Ha nincs beállított bit, az alapértelmezés a 0. meghajtó.

## FD1793 emuláció

Az FD1793 implementáció regiszterállapotokat tart fenn (`status`, `track`, `sector`, `data`, `intrq`) és az aktív lemez állapotgépét hajtja.

Fő státuszjelzők:

- `0x80`: nem kész / üres meghajtó / leállt motor.
- `0x40`: írásvédett vagy íráshiba.
- `0x20`: fej betöltve vagy rekordtípus.
- `0x10`: seek hiba vagy rekord nem található.
- `0x08`: CRC hiba.
- `0x04`: track 0 vagy adatvesztés.
- `0x02`: index vagy DRQ.
- `0x01`: busy.

Támogatott parancsok:

- **Restore (`0x00`)**: fej a 0. sávra, `INTRQ`.
- **Seek (`0x01`)**: a data regiszterben megadott sávra áll.
- **Read Sector (`0x08 / 0x09`)**: szektor olvasása, `ST_BUSY`, majd `DRQ`.
- **Read Address (`0x0C`)**: a következő szektor fizikai azonosítóját adja.

## Lemezkép-struktúra

A [src/fd1793.rs](../src/fd1793.rs) nyers, MS-DOS-kompatibilis `.dsk` szektordumpokat kezel. Betöltéskor a FAT12 boot szektorból olvassa a geometriát:

- szektorméret: offset 11-12
- szektor/sáv: offset 24-25
- fejek száma: offset 26-27
- összes szektor: offset 19-20

Szektor byte-offset:

$$\text{Byte Offset} = \left(\text{Track} \times (\text{Sectors/Track} \times \text{Heads}) + (\text{Sectors/Track} \times \text{Side}) + (\text{Sector} - 1)\right) \times \text{Sector Size}$$
