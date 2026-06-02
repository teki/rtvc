# Billentyűzetmátrix és kiosztás

Ez a dokumentum nyelvfüggetlen útmutató a TVC billentyűzet-alrendszeréhez. A kapcsolódó Rust implementáció: [src/key.rs](../src/key.rs).

## Áttekintés

A TVC billentyűzete **11 sorból és 8 oszlopból** álló kapcsolómátrix. Mivel a host gép billentyűzetkiosztása eltérhet a cél TVC magyar kiosztásától, az emulátor **dinamikus automatikus leképezést** használ. Az első lenyomáskor a host billentyűt hozzárendeli a megfelelő TVC mátrixkoordinátához.

## Hardveres billentyűzetmátrix

Az állapot 11 bájtos tömbként tárolható, `_state[0..10]` formában. Minden bájt egy sor 8 oszlopát írja le.

- `1` bit: a billentyű felengedett.
- `0` bit: a billentyű lenyomott.

### Mátrixkiosztás

Az alábbi karakterkiosztást a statikus normál és shifted táblák adják:

| Sor | Oszlop 0 | Oszlop 1 | Oszlop 2 | Oszlop 3 | Oszlop 4 | Oszlop 5 | Oszlop 6 | Oszlop 7 |
|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **0** | 5 | 3 | 2 | 0 | 6 | í | 1 | 4 |
| **1** | ^ | 8 | 9 | ü | * | ó | ö | 7 |
| **2** | t | e | w | ; | z | @ | q | r |
| **3** | ] | i | o | ő | [ | ú | p | u |
| **4** | g | d | s | \ | h | < | a | f |
| **5** | _space_ | k | l | á | _space_ | ű | é | j |
| **6** | b | c | x | _space_ | n | _space_ | y | v |
| **7** | _space_ | , | . | _space_ | _space_ | _space_ | - | m |

A szóközök olyan helyeket jelölnek, amelyekhez nincs közvetlen karakterkötés vagy fenntartottak.

### Rendszerbillentyűk

| Billentyű | TVC koordináta | Leírás |
|---|---|---|
| **Backspace** | `5. sor, 0. oszlop` | előző karakter törlése |
| **Delete** | `5. sor, 0. oszlop` | következő karakter törlése |
| **Return** | `5. sor, 4. oszlop` | Enter |
| **Shift** | `6. sor, 3. oszlop` | Shift módosító |
| **Lock** | `6. sor, 5. oszlop` | Caps lock |
| **Alt** | `7. sor, 0. oszlop` | Alt |
| **Esc** | `7. sor, 3. oszlop` | Escape |
| **Ctrl** | `7. sor, 4. oszlop` | Control |
| **Space** | `7. sor, 5. oszlop` | szóköz |
| **Fel** | `8. sor, 1. oszlop` | felfelé |
| **Le** | `8. sor, 2. oszlop` | lefelé |
| **Tab / Fire** | `8. sor, 3. oszlop` | tab vagy joystick tűz |
| **Jobbra** | `8. sor, 5. oszlop` | jobbra |
| **Balra** | `8. sor, 6. oszlop` | balra |

## I/O portok

A billentyűzet a gépbuszon keresztül érhető el, lásd [src/tvc.rs](../src/tvc.rs).

1. **Sorválasztás (`0x03`)**: a CPU ide írva választja ki az olvasandó sort (`val & 0x0F`).
2. **Oszlopolvasás (`0x58`)**: a CPU innen olvassa a kiválasztott sor oszlopbitjeit. Beállítatlan sor esetén `0xFF`, vagyis minden billentyű felengedett.

## Dinamikus automatikus leképezés

A cél az, hogy US, német vagy magyar host kiosztáson is a TVC-nek megfelelő karakter jelenjen meg. A folyamat két eseményt használ:

1. **Key down**: a host fizikai billentyűazonosítót ad. Az emulátor ezt eltárolja aktív fizikai billentyűként.
2. **Szövegbevitel**: ha a billentyű karaktert eredményez, a host Unicode karaktert küld.
3. **Mátrixkeresés**: ha a karakter még nincs leképezve, az emulátor megkeresi a normál vagy shifted táblában, és a fizikai billentyűhöz köti.
4. **Regisztráció**: a koordináta és a módosítóflag bekerül a keymapbe, így későbbi lenyomásoknál közvetlenül használható.

## Shift-kompenzáció

Előfordul, hogy egy karakter a host kiosztáson Shiftet igényel, de a TVC-n nem, vagy fordítva. Ezt két flag kezeli:

- **`KSADD`**: a TVC Shiftet mesterségesen lenyomva tartja, ha a TVC-n shifted karakter kell.
- **`KSDEL`**: a TVC Shiftet mesterségesen felengedi, ha a hoston Shift kellett, de a TVC-n nem.

## Életciklus és beragadás elleni védelem

A driver három platformfüggetlen horgot használ:

- **`keyDown`**: fizikai billentyű lenyomása, módosítóállapot frissítése.
- **`keyPress`**: Unicode karakter alapján új koordinátakötés létrehozása.
- **`keyUp`**: fizikai billentyű felengedése és takarítás.

Ha a felhasználó előbb engedi fel a host Shiftet, mint a karakterbillentyűt, a felengedéskori lookup rossz módosítótáblába nézne, és a TVC mátrixban beragadhatna a billentyű. Ezért felengedéskor a kód minden módosítótáblán végigmegy, és minden megtalált koordinátát felenged.
