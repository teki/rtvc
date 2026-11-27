<!-- pdf 1 -->

TVC hardver

![](_page_0_Picture_1.jpeg)

VIDEOTON TV-Computer

<!-- pdf 2 -->

BENEDEK ANTAL

MONOK ZOLTÁN

TV-COMPUTER HARDWARE

NOVOTRADE RT.

<!-- pdf 3 -->

Lektorálta: Filp András

Szerkesztette: Stankovicsné dr. Henczl Ilona

A kiadásért felel RÉNYI GÁBOR, a Novotrade Rt. vezérigazgatója

Budapest, 1988

ISBN 963 02 5545

Copyright © Benedek András, Monok Zoltán, 1988

Készült a Somogy Megyei Nyomdaipari Vállalat kaposvári üzemében

Felelős vezető: MIKE FERENC igazgató

<!-- pdf 4 -->

### TARTALOMJEGYZÉK

- 1. Bevezetés – 9
- 2. A TV-COMPUTER alapegység működési elve – 11
  - 2.1 A TVC alapegység felépítése – 11
    - 2.1.1 Az alapegység műszaki jellemzői – 11
    - 2.1.2 Az alapegység funkcionális egységeinek kapcsolatai – 13
  - 2.2 A TVC alapegység funkcionális egységei – 15
    - 2.2.1 Központi egység – 17
    - 2.2.2 Memóriák – 20
    - 2.2.3 I/O részegységek – 33
    - 2.2.4 A TVC működésének időzítése – 47
    - 2.2.5 A megjelenítő logika – 55
    - 2.2.6 Tv-csatoló áramkörök – 74
- 3. A TV-COMPUTER alapegység áramkörei – 77
  - 3.1 Központi egység – 79
  - 3.2 A memóriák áramkörei – 81
    - 3.2.1 A lapozó áramkör – 81
    - 3.2.2 Felhasználói RAM – 85
    - 3.2.3 A rendszer ROM áramkörei – 90
    - 3.2.4 A video RAM áramkörei – 93
    - 3.2.5 WAIT áramkör – 95
  - 3.3 I/O részegységek áramkörei – 96
    - 3.3.1 I/O dekóder áramkörei – 96
    - 3.3.2 Billentyűzet csatoló – 97
    - 3.3.3 Magnetofon csatoló – 98
    - 3.3.4 Nyomtató csatoló – 99
    - 3.3.5 Hangáramkörök – 99
    - 3.3.6 A megjelenítő logika I/O áramkörei – 103

<!-- pdf 5 -->

- 3.4 Az időzítő áramkörök – 105
  - 3.4.1 A RESET áramkör – 105
  - 3.4.2 Az órajel-generátor áramkörei – 106
  - 3.4.3 A video időzítő áramkörök – 107
  - 3.4.4 A CLOCK STRETCH áramkör – 108
- 3.5 A megjelenítő egység áramkörei – 109
- 3.6 A televízió bemenőjeleit előállító áramkörök – 116
  - 3.6.1 A PAL kódoló – 116
  - 3.6.2 Az UHF modulátor áramkörei – 121
- 4. HARDWARE-SOFTWARE interface – 123
  - 4.1 Memóriakezelés – 123
    - 4.1.1 Memóriatérkép – 123
    - 4.1.2 A videomemória használata – 124
  - 4.2 Perifériakezelés – 125
    - 4.2.1 I/O címkiosztás – 125
    - 4.2.2 Az I/O címek készülékfunkciók szerinti szétosztása – 127
  - 4.3 Táblázatok – 130
- 5. Az alapegység felépítése (bontás, összeszerelés) – 143
- 6. A TV-COMPUTER nagyintegráltságú alkatrészei – 145
  - 6.1 A Z80A mikroprocesszor – 145
    - 6.1.1 A Z80A főbb jellemzői – 145
    - 6.1.2 A CPU jeleinek megnevezése és értelmezése – 147
    - 6.1.3 A CPU belső felépítése – 151
    - 6.1.4 A mikroprocesszor működésének idődiagramjai – 162
    - 6.1.5 A CPU utasításkészlete – 162
  - 6.2 A 6845-ös CRTC – 163
    - 6.2.1 A tv raszterrendszerű CRT display-k – 164
    - 6.2.2 A 6845-ös CRTC-vel kapcsolatos ismeretek – 175
- 7. A tápegység – 195
  - 7.1 A tápegység működési elve – 195
  - 7.2 A tápegység felépítése – 196

<!-- pdf 6 -->

  - 7.2.1 A tápegység vezérlő- és védőáramköre – 196
  - 7.3 A TV-Computer tápegységeinek áramköri ismertetése – 198
  - 7.4 A tápegység felépítése (bontás, összeszerelés) – 200
- 8. Ábrák – 201
- 9. Melléklet

<!-- pdf 7 -->

<!-- pdf 8 -->

# 1. BEVEZETÉS

A VIDEOTON TV-Computer (a továbbiakban: TVC) vásárlói számára a készülékekkel együtt átadott két könyv (Kezelési útmutató és BASIC programozói segédlet) csak a használatbavételhez és kezeléshez elengedhetetlenül fontos ismereteket tartalmazza.

Az Olvasó által most kézben tartott könyv azokhoz szól, akik a készülék felépítésével, működésével is szeretnének alaposabban megismerkedni.

A könyvben igyekeztünk az összes információt összegyűjteni, amit a TVC hardware működésének megértéséhez fontosnak találtunk, hogy ne kelljen hivatkozni más szakirodalomra. Reméljük, hogy így a számítógépek hardware felépítésében kevésbé jártas olvasóinknak is jól használható eszközt adunk a kezükbe.

A közölt információk jelentős segítséget nyújthatnak a programkészítéshez, elsősorban természetesen a gépi kódú programozáshoz.

A TVC két különálló egysége közül az alapegység lényegesen összetettebb, mint a tápegység, ezért amíg az alapegységgel öt fejezet foglalkozik, a tápegységről csak a 7. fejezet beszél.

A 2. fejezet az alapegységet felépítő funkcionális egységeket és főbb működési elveket ismerteti. A megértést sok ábra segíti.
A 3. fejezet az áramköri megoldásokat tárgyalja részletesen. Ennek a fejezetnek tanulmányozásához célszerű magunk elé teríteni a kapcsolási rajzot, ill. rajzokat és azon követni a szövegben közölteket, mivel ez a fejezet tulajdonképpen a kapcsolási rajz magyarázata.

<!-- pdf 9 -->

A 4. fejezetben foglaltuk össze azokat az ismereteket, amelyek a berendezés egyes funkcióinak a program általi kezeléséhez fontosak (hardware-software interface).
Az 5. fejezet a TVC alapegység mechanikus felépítéséről, össze- és szétszereléséről tájékoztat.
A 6. fejezet adatlapként használható a TVC alapegység két legfontosabb nagyintegráltságú alkatrészéhez, a mikroprocesszorhoz és a CRTC-hez.

Az egyes fejezetek egymásra épülnek. Igyekeztünk a későbbi fejezetekre való hivatkozásokat csak ott alkalmazni, ahol feltétlenül szükséges, viszont valamennyi fejezetben leírtak megértéséhez általában szükségesek a korábbiakban levő ismeretek.

Általános hivatkozásként elmondható, hogy a 3. fejezet és a 2.2 alfejezet pontjai között szoros kapcsolat van. A 3. fejezet valamely alfejezetében ismertetett áramkörök blokkvázlat szintű ismertetését, működési elvét a 2.2 alfejezet megfelelő pontja tartalmazza, pl. a 2.2.1 pont a központi egység felépítését, a 3.1 alfejezet a központi egység áramköreit ismerteti.

Igyekeztünk a felépítésre és működésre vonatkozó ismeretek minél teljesebb átadására, ennek ellenére azt tanácsoljuk, hogy aki a TVC javítására vagy hardware bővítésére vállalkozik, az a könyv áttanulmányozása után is nagyon óvatosan járjon el és lehetőleg kérje ki valamelyik VIDEOTON szerviz tanácsait.

<!-- pdf 10 -->

# 2. A TV-COMPUTER ALAPEGYSÉG MŰKÖDÉSI ELVE

## 2.1 A TVC alapegység felépítése

### 2.1.1 AZ ALAPEGYSÉG MŰSZAKI JELLEMZŐI

A működés ismertetése előtt célszerű áttekinteni a TVC fontosabb jellemzőit.

A TVC három változatban készült: 32k, 64k és 64k+.

A 32k és 64k változat között csak az alapegységben található felhasználói RAM kapacitásában van különbség. Így a 32k verzió egy RAM bővítő modul alkalmazásával teljesen kompatibilissé tehető a 64k verzióval.

A 64k+ változat az előző kettőtől elsősorban a videomemória méretében tér el, 16 kbyte helyett 64 kbyte a memória. A nagyobb videomemória egyidejűleg négy teljes kép tárolását teszi lehetővé. Megjeleníteni egyszerre csak egyet lehet.

A TVC egy 8 bites mikroprocesszorra alapozott számítógép. A legegyszerűbb kiépítésben az együtt vásárolt alapegység és tápegység mellé csak egy tv-készülék szükséges. Már ez az összeállítás is nagyon sok célra használható.

A tv-készülék azért kell, hogy a gépben levő adatok a kezelő számára elérhetővé váljanak. A tv lehet fekete-fehér vagy színes.

A tv-készülékhez a TVC az antennabemeten keresztül csatlakozhat. Jobb

a képminőség, ha a tv rendelkezik összetett videojel bemenettel, vagy <!-- pdf 11 --> méginkább ún. RGB bemenettel. A TVC mindhárom típusú kapcsolatra képes. Természetesen a tv-készülék helyett szabványos CRT monitor is használható.

A kezelő a gépnek szánt utasításokat, adatokat, programokat a TVC beépített billentyűzetét használva juttathatja a gépbe.

A begépelt programok kimentése közönséges kazettás magnetofonra történhet, erre az alapegységre magnetofon csatolója teremt lehetőséget. A magnetofonról természetesen kész programok is betölthetők.

Programbevitel történhet ROM áramköröket tartalmazó dugaszolható programmodulok segítségével is, ha a program mérete nem haladja meg a 16 kbyte-ot.

A programok és adatok tárolására az alapegységben a következő memóriák állnak rendelkezésre:

| | |
|------|-------|
| Rendszer ROM (az OS és a BASIC számára) | 20 kbyte (max. 24 kbyte) |
| Felhasználói RAM | 32 kbyte (bőv. 64 kbyte-ig) |
| A kijelzésre szánt adatok számára a video RAM | 16 kbyte vagy 64 kbyte |
| A bővítésként jelentkező rendszerelemek kezelőprogramjai számára bővítésenként (max. 4 db) | 8 kbyte max. |
| Programmodul | 16 kbyte max. |

Az alapegység hangképző áramköreinek jele a tv-hangszóróján keresztül hangeffektusok létrehozását teszi lehetővé.

A képernyőtartalom, vagy egyéb adatok kinyomtatására a nyomtató csatoló nyújt lehetőséget, ha Centronics típusú interface-szel rendelkező nyomtatót kapcsolunk hozzá.

<!-- pdf 12 -->

A felhasználást segítő egyéb rendszerelemek (soros-, párhuzamos interface, floppy csatoló stb.) bővítésként csatlakoztathatók a TVC e célra kialakított négy csatlakozójába.

A TVC alapegységének áramkörei a fenti jellemzőket valósítják meg.

### 2.1.2 AZ ALAPEGYSÉG FUNKCIONÁLIS EGYSÉGEINEK KAPCSOLATAI

A TVC logikai-funkcionális részegységeit és a közöttük levő kapcsolatokat az 1. ábra mutatja.

A készülék működését egy mikroprocesszor (típusa: Z80A) irányítja. A mikroprocesszor és a kivezetéseire csatlakozó buszmeghajtók alkotják az ábrán központi egység néven jelölt blokkot.

A központi egységhez az egyéb részegységek a buszrendszeren keresztül kapcsolódnak. A buszrendszer a memória és I/O címeket hordozó címbuszból, a kétirányú adatbuszból és a busz adatforgalmát irányító vezérlőjelekből áll.

A Z80A – hasonlóan más mikroprocesszorhoz – a működése során többször környezetéhez fordul. A környezetét alkotó egységeket, áramköröket két csoportra szoktuk osztani. A megosztás szempontját az adja, hogy az illető eszközt a program milyen utasításokkal kezeli.

Az egyik csoporthoz tartoznak a memóriák, amelyek használatához nagyon sok utasítással rendelkezik a processzor. A memóriaként kezelt külső eszközök tárolóelemeit a belső regiszterekhez hasonlóan használhatja a mikroprocesszor.

A környezetet alkotó egységek másik csoportját az előzőektől eltérő, viszonylag kevés számú utasítással érheti el. Ezek a perifériák vagy a könyvben gyakran használt másik nevükön: I/O eszközök.

<!-- pdf 13 -->

A részegységek közül az alábbiak memóriák, ill. memóriát is tartalmaznak:

- RAM (32 kbyte vagy 64 kbyte)
- display áramkörök (video RAM: 16 kbyte vagy 64 kbyte)
- programmodul (az ábrán csak a csatlakozója, max. 16 kbyte)
- bővítőegységek (az ábrán csak a csatlakozójuk, 4 db max. 8 kbyte)

A fizikailag létező, ill. kiépíthető memóriák összkapacitása meghaladja a használt mikroprocesszor közvetlen címzési lehetőségeit, emiatt a fizikai memóriáknak a processzor címtartományaihoz való hozzárendelése változtatható a program által a lapozó áramkör segítségével.

Az alkalmazott memória IC-k hozzáférési idejétől függően az egyes fizikai memóriaterületek megnyújthatják a központi egység memóriahozzáférési ciklusait. Az ehhez szükséges WAIT áramkör csak a 32k és 64k verzióban van kiépítve, a 64k+ verziónál az áramkör a bővítő kártyákon építhető ki.

A rendszer egyéb elemeit a mikroprocesszor I/O műveletekkel kezeli.

A processzor I/O címeiből az I/O dekóder alakítja ki az egyes funkciók kiváltását, ill. indítását végző I/O szelekciós jeleket.

I/O műveletekkel kezelt részegységek:

- a lapozó áramkör
- billentyűzet
- magnetofon csatoló
- nyomtató csatoló
- hangáramkörök
- a megjelenítő egységek egyes funkcióit is I/O műveletek irányítják, továbbá
- a bővítő egységek is tartalmazhatnak I/O vezérelt áramköröket

<!-- pdf 14 -->

Egyes I/O eszközök kiszolgálási igényüket megszakítás (IT) kéréssel jelezhetik a központi egységnek. A megjelenítő áramkörök IT kérése az adatmegjelenítési folyamat meghatározott állapotában ciklikusan következik (20 ms periódusidővel). A hangáramkör IT kérése programozható időzítést tesz lehetővé. A bővítő egységek is kérhetnek megszakítást.

A display áramkörök videojelét és a hangáramkörök jelét a tv csatoló alakítja a kommersz tv által megkívánt formátumúra. A csatoló két funkcionális egysége a PAL kódoló és a csatorna adapter.

Az időzítő áramkör az egész rendszer működéséhez szükséges órajeleket állítja elő.

## 2.2 A TVC alapegység funkcionális egységei

Az egységek csoportosítását az alábbi szempontok szerint végeztük el:

Az alapegység funkcionális egységeinek döntő többsége a CPU és ezzel a program irányítása alatt áll. Az első három pontban ezért a mikroprocesszoros rendszer elemeit ismertetjük. Mint már a korábbiakban is tettük, megkülönböztetjük a memóriákat és az I/O egységeket (perifériákat). A központi egység ismertetése utáni két pont ezért a memóriákat és az I/O egységeket foglalja össze.

A WAIT áramkört a memóriáknál tárgyaljuk. Bár WAIT kérésre a perifériák kezelésénél is szükség lehetne, a TVC-ben a perifériák nem kérnek WAIT állapotot.

Az IT áramkört logikusnak tartottuk a perifériáknál tárgyalni.

A további pontok a TVC-nek a CPU által közvetlenül el nem érhető részegységeit ismertetik.

<!-- pdf 15 -->

Nem függ a futó programtól az egész rendszer működésének ütemezését végző időzítő áramkör.

A megjelenítő logika egyes részegységeit a CPU I/O parancsokkal vezérli, ezenkívül található benne memória (videomemória) is, ugyanakkor a megjelenítési folyamat fix kép esetén a CPU-tól függetlenül zajlik. A megjelenítő logika ilyen jellemzői miatt külön pontot kapott. Meg kell jegyezni, hogy a videomemória a memóriák között, a megjelenítő logika I/O parancsokkal kezelhető egységei a perifériák között is szerepelnek, ily módon az egyes fejezetek bizonyos mértékig átfedik egymást.

A tv csatoló az alapegység áramköreitől jellegében (elsősorban jeleinek analóg mivoltában) is eltér, ezekívül külön kártyákon is foglal helyet, emiatt mindenképpen indokolt, hogy külön pont tárgyalja.

A 2.2 alfejezethez tartozó összes ábrán a funkcionális egységek közötti kapcsolatot biztosító jelek elnevezése megegyezik a kapcsolási rajzon található jelelnevezésekkel.

Az összes jelelnevezésre vonatkozik, hogy a szövegben a jelnév elé tett N betűvel jelöljük azt a tényt, hogy a jel 0 szintje az aktív, vagy, hogy valamely jel invertált változatáról van szó. Ugyanezt a tényt az ábrákon és a kapcsolási rajzon a jelnév fölötti felülhúzás jelzi.

A leírás során végig pozitív logikát használunk, tehát a logikai 0 érték az alacsony (L) szinthez, a logikai 1 érték a magas (H) szinthez tartozik.

A címek és adatok értékét decimálisan vagy hexadecimálisan (16-os számrendszerben) adjuk meg. A hexadecimális értéket a szám után írt H betűvel jelöljük.

<!-- pdf 16 -->

### 2.2.1 KÖZPONTI EGYSÉG

A központi egység felépítését a 2. ábra mutatja.

A TVC-ben használt mikroprocesszor (Z80A) részletesebb ismertetése a 6.1 alfejezetben található.

A mikroprocesszor működéséhez szükséges egy folyamatos órajel, ezt a TVC-ben  φ  (görög fi) jellel jelöljük, frekvenciája 3,125 MHz.

A mikroprocesszort alapállapotba állítja az NRESET jel.

A processzor a működése során a memóriájában tárolt programnak megfelelően kapcsolatba lép a környezetét alkotó áramkörökkel. A kapcsolat szempontjából alapvetően eltér a memóriákkal való művelet a rendszer egyéb elemeivel való információcserétől. A memóriákkal kapcsolatban a processzor a következő műveleteket végezheti:

- a memóriából beolvassa a következő utasítás kódját
- a memóriából adatot olvas be
- a dinamikus memóriát adatfelfrissítés céljából kezeli
- a memóriába adatot ír

A 8 bites adatok a kétirányú adatbuszon (D0-D7 jelek) mozognak.

A processzor a memóriának egy-egy 8 bites egységét kívánja elérni. A kívánt memóriarekeszt a 16 bites címbuszon levő bitkombináció választja ki.

A cím- és adatbuszon levő információ értelmezését a processzor által kiadott vezérlőjelek végzik.

- Az utasításlehívás folyamatát az NMREQ, az NM1 és az NRD jelek együtt jelzik. (Az NMREQ bármilyen memóriaműveletnél, az NRD bármilyen olvasási műveletnél aktív, az NM1 jelzi az utasítás beolvasását.) A címbuszra az utasításkódot tartalmazó memóriarekesz címét <!-- pdf 17 --> helyezi a processzor. Az adatbuszra a memóriából a megcímzett rekesz tartalma kerül.

- A memóriából történő adatbeolvasást az NMREQ és az NRD jelek aktív állapota jelzi. A címbuszra a kívánt memóriarekesz címét adja a processzor, az adatbuszra a memória kapcsolódik, és a megcímzett rekesz tartalmát helyezi oda.
- A dinamikus memória felfrissítési folyamata során a processzor minden utasításlehívást követően a címbusz alacsonyabb helyi értékű 8 vezetékén egy, minden utasítás lehívásakor inkrementálódó memóriacímet ad ki, ezt az NMREQ és az NRFSH jelekkel érvényesíti.
- A memóriába történő írást az NMREQ és az NWR jelek végzik. Az adatbuszon a processzor által a memóriába írni kívánt adatbyte található, a címbuszon a kijelölt memóriarekesz címe.

A mikroprocesszor kivezetése számára a TVC egységei túl nagy áramköri terhelést jelentenének, ezért a jelek buszmeghajtókon keresztül jutnak hozzájuk. A cím- és a vezérlőjelek egyirányú, az adatbuszjelek kétirányú meghajtókon keresztül kapcsolódnak a részegységekhez.

A vezérlőjeleket és az adatbusz jeleit a részegységek közül néhány közvetlenül kapja. Ez azt jelenti, hogy a készüléken belül két adatbusz és két vezérlőbusz található. A vezérlőjelekkel együtt a mikroprocesszor órajele és az NRESET jel is megjelenik erősített formában is. Az erősített jeleket a mikroprocesszor kivezetéseinek jelétől a jelnév elején található B betű különbözteti meg. A két busz jelei között a buszmeghajtó késleltetésének megfelelő fáziseltolódás van, ez a tény meghatározza, hogy melyik részegység melyik buszra kapcsolható.

A kétirányú adatbuszmeghajtó csak olvasási folyamatnál (NRD=0) engedi az adatokat a processzor felé, minden egyéb esetben fordított az irányítottsága. Olvasáskor sem kapcsolódik a processzor közvetlen adatbuszára (D0-D7), ha a rendszer ROM-ból (NROM és NROM5 jelek), vagy a CRTC-ből (NCRTC) történik az olvasás, mivel ezek az egységek a közvetlen <!-- pdf 18 --> buszra kapcsolódnak. A 32k és 64k változatnál a programkazetta ROM (NCART) is a közvetlen buszra adja az adatát, ezért ott az NCART jel is meggátolja, hogy a kétirányú adatbuszmeghajtón az adatáramlás a CPU felé irányuljon.

Az ábrán jelölt valamennyi buszmeghajtó 3 állapotú (TRI-STATE). Nagyimpedanciás állapotba akkor kerülnek, ha a processzor NBUSAK jele aktivizálódik. Erre akkor kerülhet sor, ha a TVC valamely, a későbbiekben esetleg kifejlesztésre kerülő egysége az NBUSRQ jelen keresztül jelzi a mikroprocesszornak, hogy át kívánja venni a buszrendszer irányítását (pl. DMA). A TVC alapegység áramkörei ezt a lehetőséget nem használják, sőt a jelenleg gyártott, ill. fejlesztés alatt álló bővítő egységek sem.

A 64k+ TVC változatnál az NBUSAK jel DMA jelnéven a bővítő csatlakozóra ki van vezetve.

A nem memóriaként kezelt részegységekkel (perifériákkal) az adatforgalom a következőképpen zajlik le:

 Valamely perifériáról való olvasást a processzor NIORQ és NRD jeleinek együttes aktív állapota jelzi.

A periféria címe a memóriacímzésre is használt címbusz alacsonyabb helyi értékű 8 címvezetékén található. A perifériáról olvasott adat a memóriák által is használt adatbuszra kerül.

- Valamely perifériára történő írást a NIORQ és NWR jelek jelzik. A cím az olvasáshoz hasonló, az adatbuszra a processzor helyezi a kiküldendő adatot.

A perifériák közül néhány (az előző alfejezetben említettek) IT kérést generálhat. Ez azt jelenti, hogy a periféria a processzor NINT bemenetén keresztül jelez a processzornak. A processzor felfüggeszti az éppen folyamatban levő program végrehajtását, és az IT-t kérő perifériához rendelt programrészen folytatja a műveleteket.

<!-- pdf 19 -->

Az IT kérés elfogadása néhány, vagy minden perifériára vonatkozóan letiltható (ún. maszkolás). A Z80A rendelkezik egy NNMI bemenettel is. Ezen a bemeneten érkező IT kérés a programból nem tiltható le.

A TVC alapegység áramkörei csak a processzor NINT bemenetén keresztül kérhetnek IT-t, a bővítő egységeknél lehetőség van nem maszkolható IT kérésre is (NNMI bemenet).

A lassú memóriák a CPU memóriaolvasási folyamatát megnyújthatják. A 32k és 64k alapegységben levő WAIT áramkör csak az utasításlehívási ciklust képes egy óraütem erejéig megnyújtani. A 64k+ alapegységhez nincs WAIT áramkör. A bővítő egységben kialakítható olyan áramkör, amely egyéb esetekben (memóriaírás vagy -olvasás; perifériaírás vagy -olvasás) is WAIT kérést generál. A 64k+ változatnál csak így történhet a WAIT kérés. Az áramkör tetszőleges számú óraütem idejéig fennálló WAIT állapotot hozhat létre.

### 2.2.2 MEMÓRIÁK

Mint már említettük, a TVC mikroprocesszorának és a memóriáknak a kapcsolatát alapvetően meghatározza az a tény, hogy a rendszerben levő kiépített vagy kiépíthető memóriák összkapacitása meghaladja a felhasznált Z80A közvetlen memóriacímzési képességét. A kiépíthető memóriák összkapacitása 152 kbyte (ill. 200 kbyte a 64k+ változatban), szemben a mikroprocesszor 16 címvezetéke által címezhető 64 kbyte-tal. Az ellentmondás úgy oldható fel, hogy a módosítani lehet a fizikai memóriák hozzárendelését a központi egység egyes címtartományaihoz. Egyidejűleg természetesen csak maximum 64 kbyte memória érhető el a processzor számára. Amennyiben a programfutás során más fizikai memória használatára lenne szükség, akkor a program meghatározott utasításokkal a kívánt memóriát a kívánt címtartományhoz kapcsolja, miután az addig ott elérhető memóriát lekapcsolja.

A fizikai memóriák mindegyike egy működést engedélyező jelvezetékkel rendelkezik.

<!-- pdf 20 -->

A lehetséges fizikai memóriák a következők:

| Memória | Engedélyező jel |
|---|---|
| Felhasználói RAM 0. 16 kbyte-os egység (U0) | NP0 |
| Felhasználói RAM 1. 16 kbyte-os egység (U1) | NP1 |
| Felhasználói RAM 2. 16 kbyte-os egység (U2) | NP2 |
| Felhasználói RAM 3. 16 kbyte-os egység (U3) | NP3 |
| Rendszer ROM 16 kbyte | NROM |
| Video RAM 16 kbyte (vagy 4*16 kbyte) | NVRAM |
| Programkazetta ROM 16 kbyte | NCART |
| Rendszer ROM kiterjesztés 8 kbyte | NROM5 |
| Bővítő egységek RAM/ROM területe az egyes bővítő csatolókhoz rendelve: CH1 8 kbyte | NEXP1 |
| CH2 8 kbyte | NEXP2 |
| CH3 8 kbyte | NEXP3 |
| CH4 8 kbyte | NEXP4 |

A fizikai memóriáknak a központi egység címtartományaihoz való lehetséges hozzárendeléseit a hardware-software interface-ben levő memóriatérkép 4.3 alfejezet (1. táblázat) mutatja.

A táblázatból megállapítható, hogy egy-egy 16 kbyte-os címtartományhoz több fizikai memória rendelhető, de egyes fizikai memóriák több címtartományban is megjelenhetnek.

A központi egység és a memóriák kapcsolatát a 3. ábra mutatja. Az ábrából látszik, hogy a memóriák a közvetlen és az erősített busz jeleit eltérő módon használják. A felhasználói RAM és a videomemória a közvetlen adatbuszról (D0-7) kapja az adatokat, kimenetük viszont az erősített adatbuszra (BD0-7) kapcsolódik. A rendszer ROM a közvetlen adatbuszra adja adatát, a programmodul a 32k és 64k verzióban a közvetlen, a 64k+ változatnál az erősített adatbuszt használja. A bővítő egységek memóriái be- és kimeneti célra egyaránt az erősített adatbuszt veszik igénybe.

<!-- pdf 21 -->

A memóriáknak az alapegységben található áramköreiről a következő fejezetekben részletesebben szólunk. Itt csak a következőkre hívjuk fel a figyelmet:

- A lapozó áramkör NP2 és NP3 jele a felhasználói RAM két 16 kbyte-os egységének (U2 és U3) szelekciós jelei. Ezek a jelek a bővítő csatlakozókra is ki vannak vezetve. A kivezetésnek csak a 32k verziónál van értelme, mert a másik két változatnál a felhasználói RAM mind a négy 16 kbyte-os egysége az alapegységen belül található.
- A WAIT áramkört csak a 32k és 64k változatnál találhatjuk meg. A bővítő egységek az NSLOWEXP, a programmodul az NFAST jellel kérhetnek WAIT állapotot az utasítás lehívási ciklusban (NM1). A rendszer ROM szelekciós jelei, az NROM és NROM5 az alapegység kártyán beköthetők a WAIT áramkörbe, ha szükséges. A gyárilag beépített memóriaáramkörök nem igénylik a WAIT áramkör működését, ezért is hiányzik az áramkör a legújabb TVC változatból.

#### Memórialapozás

A címtartomány fizikai memória hozzárendelést, azaz tulajdonképpen a fizikai memóriák engedélyező jeleinek kialakítását a memórialapozó áramkör végzi, amelynek elvi felépítése a 4. ábrán látható.

A lapdekóder a központi egység két legmagasabb helyi értékű címjeléből (A14, A15) négy jelet állít elő, amelyek mindegyike a memóriatérképen szereplő négy címtartomány közül egyet-egyet reprezentál.

A négy jel mindegyikét inaktív állapotba vezérelheti az NPHANTOM nevű jel, amely a bővítő csatlakozóról érkezik. Felhasználásával a lapozó áramkörön keresztül az összes memória használatát tiltani lehet. A jelenleg létező és fejlesztés alatt álló bővítő egységek ezt a lehetőséget nem használják.

Az aktuális címtartomány-memória hozzárendelésre vonatkozó információt <!-- pdf 22 --> a processzor OUT utasítással (perifériaírás) a lapozást vezérlő regiszterbe írja. A regiszter a 32k és 64k változatnál 5, a 64k+ változatnál 6 bites, az írást az I/O dekóder által előállított NWR2 jel végzi el.

Az 5 (ill.6) bit felosztása a címtartományok között a memóriatérkép figyelembevételével a következő:

- 2 bit szükséges a 0. laphoz, mivel 4 különböző memória rendelhető hozzá
- nem kell váltani az 1. lap hozzárendeléseit a 32k és 64k változatnál, mert mindig csak a felhasználói RAM 1. egysége érhető el itt; a 64k+ változatnál a regiszter többlet bitje itt fejti ki hatását két memória közül szelektálva
- 1 bit információ kell a 2. laphoz, mert két memória tartozik hozzá
- 2 bit információ kell a 3. laphoz, mert négy fizikai memória rendelhető hozzá.

A lapdekóder által kijelölt (a CPU által épp címzett) 16 kbyte-os címtartományhoz rendelt jel a demultiplexerekre jut. A demultiplexerek a jelet a lapozást vezérlő regiszter megfelelő bitjei által meghatározott fizikai memóriaterülethez juttatják engedélyező jelként.

A 32k és 64k változatnál az 1. laphoz nem tartozik ilyen demultiplexer, mivel ott ebben a címtartományban mindig az U1 felhasználói RAM érhető el. A lapdekódernek az 1. laphoz tartozó kimenőjele a TVC változatnál közvetlenül adja az NP1 engedélyező jelet.

A memóriatérképen látható módon a programkazetta ROM, a rendszer ROM és az U3 RAM a 0. és a 3. lapon is megjelenhet. Ezért az NCART, az NROM és NP3 engedélyező jelek kialakítását a 0. és a 3. lapok demultiplexerei is végezhetik kapuáramkörön keresztül. Hasonló a helyzet az NVRAM jel kialakításával is a 64k+ TVC változatnál: az 1. és 2. lap demultiplexerei is befolyásolják, tekintettel arra, hogy a videomemória két címtartományban is megjelenhet.

<!-- pdf 23 -->

A NCART és NROM jelek aktív állapotának kialakításához a memóriaműveletet jelző NMREQ jel aktív állapota is szükséges, hasonlóan az NEXP jelhez (a RAM memóriák esetében egyéb egységeken keresztül hat az NMREQ jel.)

A csak egy címtartományban megjelenő fizikai memóriáknak az engedélyező jelei a demultiplexerek kimenetén közvetlenül kialakulnak (NP0, NP1 és NP2).

A kapuáramkörökön kialakuló NEXP jel két egészen eltérő funkciójú 8 kbyte-os fizikai memóriát reprezentál. Az egyik a rendszer ROM kiterjesztés (a rendszer ROM egy 16 kbyte-os egységből és a 8 kbyte-os kiterjesztésből épül fel, így lesz max. 24 kbyte), a másik a bővítő egységek RAM/ROM területe. A NEXP jel ezért egy demultiplexerre jut, amelyet az A13 címvezeték vezérel. Egyik állapotában a rendszer ROM kiterjesztés NROM5 engedélyező jele alakul ki, a másikban a bővítő egységek valamelyikének memóriaterülete van engedélyezve.

Azt, hogy a négy bővítő csatlakozó melyikén található bővítő egységet érheti el a processzor, a bővítő egységek lapozó regisztere határozza meg. Ez a regiszter 2 bites, a processzor szintén I/O művelettel (NWR3 jel) tölti fel. Tartalma kijelöli, hogy az EXP demultiplexer melyik kimenetére jusson engedélyező jel, azaz a NEXP1,2,3 vagy 4 alakuljon-e ki.

Néhány példa a lapozó áramkör működésére:

- A készülék bekapcsolásakor vagy a RESET nyomógomb működtetésekor a RESET jelen megjelenő impulzus a lapregisztert olyan helyzetbe állítja, hogy a 0. laphoz (0-3FFFH címtartományhoz) a rendszer ROM legyen hozzárendelve. A mikroprocesszor a 0000 címet adja a címbuszra. Az A14-15 jelek 0 állapota miatt a lapdekóder 0. lap elnevezésű kimenetén alakul ki impulzus. A lapregiszter 3. és 4. kimenetei a 0. lap demultiplexerének 0. kimenetére irányítják ezt az impulzust, amely így a kapuáramkörön át az NROM jel aktív állapotát hozza létre. A címbusz alacsonyabb helyi értékű bitjei a rendszer

<!-- pdf 24 -->

ROM címvezetékeire rákerülnek, az általuk kiválasztott rekesz tartalmát a rendszer ROM az NROM jel engedélyével az adatbuszra helyezi.

- A program tartalmazhat olyan utasításokat, hogy a 02H I/O címre új hozzárendelési kódot küldjön a processzor, pl. a 0. laphoz a programkazetta ROM legyen hozzárendelve. Ekkor, ha a 0-3FFFH tartományba eső címet ad ki a mikroprocesszor, és a lapdekóder 0. lap elnevezésű kimenete aktív, akkor a demultiplexer a lapregiszter bitjei által olyan vezérlést kap, hogy a jelet a 2. kimenetére továbbítsa. Ez a kimenet a kapuáramkörön át az NCART jellel, a programkazetta ROM engedélyező jelével van kapcsolatban, így onnan érkezhetnek adatok a processzorhoz.
- A lapregisztert úgy is be lehet állítani, hogy a 7. és 6. bitje a 3. lap demultiplexerének 1. kimenetét választja ki. Ekkor a 3. lapon, azaz a C000H-FFFFH címtartományban a rendszer ROM jelentkezik. A lapdekóder az A14-A15 címvezetékek logikai 1-es állapotában a 3. lap elnevezésű kimenetét vezérli aktív állapotba. Ezt a jelet a 3. lap demultiplexere az 1. kimenetre irányítja, a kapuáramkörön át az NROM jelhez jut, azaz ilyenkor a rendszer ROM a 3. laphoz van rendelve.

Az ábrát és a memóriatérképet együtt vizsgálva a többi hozzárendeléshez is meg lehet találni az adatutakat.

#### Felhasználói RAM

A TVC két változata (32 kbyte-os és 64 kbyte-os) e memória méretében tér el egymástól.

A TVC-nek ez a memóriája dinamikus RAM. A dinamikus RAM egyik legfontosabb jellemzője, hogy a valamely memóriarekesz kiválasztásához szükséges címjelek nem egyidejűleg jutnak a memóriába. A címjelek egy részét sorcímeknek, a többit oszlopcímeknek nevezzük. A TVC 32 kbyte-os változatában alkalmazott 16 kbites memória áramköröknél például

<!-- pdf 25 -->

7-7 bites a sor- és oszlopcím, a 64 kbyte-os változatában található 64 kbites memória áramköröknél 8-8 bites a sor- és oszlopcím. A sor- és oszlopcímek időmultiplex módon a memóriának ugyanazon bemenetein át kerülnek a memória belsejébe.

A dinamikus memória másik fontos jellemzője, hogy meghatározott, viszonylag rövid idő alatt (a TVC-ben használt memóriánál 2 ms) valamennyi sorcímet a memóriára kell adni, különben elvész a benne lévő információ. A folyamatot felfrissítésnek nevezzük és történhet periodikus írás vagy olvasás formájában, amikor is az oszlopcímeket is használjuk, de a legtöbb esetben külön művelet keretében csak a sorcímeket adjuk a memóriára.

Az ilyen dinamikus memóriákkal való kapcsolattartás a következő jelvezetékeken keresztül történik:

- Címvezetékek, amelyeken keresztül a sor- és oszlopcímek is a memóriához jutnak. A 16 kbites memóriánál 7, a 64 kbit-esnél 8 címvezeték van.
- NRAS jel, amely a memória felé azt jelzi, hogy a címvezetékeken sorcím található.
- NCAS jel, amely az oszlopcímeket érvényesíti.
- NWR jel, amely megkülönbözteti az írási folyamatot az olvasástól.
- IN jel, amelyen keresztül a memóriába kerül a beírni kívánt adat.
- OUT jel, amelyen megjelenik a megcímzett memóriarekesz tartalma.

A jeleknek időbeli viszonyait mutatja az ábra a TVC-ben használt három folyamat (írás, olvasás és felfrissítés) alatt.

Az előbbiek alapján értelmezhető a felhasználói RAM blokkvázlata, amely a 6. ábrán látható.

A RAM vezérlő áramkör a központi egység NBMREQ és NRFSH jeleiből és a lapozó áramkör NP0-NP3 jeleiből a processzor jelét időzítésre felhasználva kialakítja a memóriakezeléshez szükséges jeleket.

<!-- pdf 26 -->

A 32 kbyte-os TVC változat két, egyenként 16 kbyte-os memóriaegységet (2x8 db 16k x 1 bites memóriachip), a 64 kbyte-os változat egy 64 kbyte-os egységet (8 db 64k x 1 bites memóriachip) tartalmaz.

Az NRAS jel valamennyi memóriaelem bemenetén ugyanaz.

Az NCAS jelből kettő van. A 32 kbyte-os változatnál a két 16 kbyte-os memóriaegység (U0 és U1) közül az egyik az NCAS64, a másik az NCAS jelet használja. Az NCAS64 jel kialakulásának feltételei a két változatnál eltérnek. A 32 kbyte-os változatnál csak az NP0 jel hatására alakul ki impulzus rajta, a 64 kbyte-os változatnál az NP0-NP3 jelek bármelyike aktivizálja.

A sor- és oszlopcímek közötti váltást is a RAM vezérlő végzi a cím multiplexerre adott MUX jellel. A cím multiplexer alakítja ki a címbusz A0-A15 jeleiből a sor- és oszlopcímeket egyaránt hordozó DA0-DA7 memóriacímeket. A 32 kbyte-os változatnál az A14-A15 jeleknek és a DA7 jelnek nincs hatása.

A 64k+ TVC változatnál az ábrán látható módon a DA7 címbit egy módosító áramkörön megy keresztül. A módosításra azért van szükség, hogy a felhasználói RAM U3 egysége a lapregiszter átírásával a 0. lapon is elérhető legyen. A 32k változatnál a 64 kbyte felhasználói RAM 4 db 16 kbyte-os egységből áll (ebből kettő az alapegységben, kettő a memóriabővítésben lehet), amelyek mindegyike külön kap oszlopcím-érvényesítő NCAS jelet. Az NCAS jelek kialakításában részt vesznek a lapozó áramkör által a 4 db 16 kbyte-os felhasználói RAM egység (U0-U3) kiválasztása céljából előállított NP0-NP3 jelek. A 32k változatból és memóriabővítésből álló 64 kbyte-os TVC-nél ezért elég az, hogy a 0. laphoz tartozó címtartomány megcímzésekor az NP3 jelet aktivizáljuk (amelyet az előző fejezet szerint a lapozó áramkör mindhárom TVC változatnál el tud végezni).

A 64 kbit-es memóriachip-ekből felépített felhasználói RAM esetében ez nem járható út, ott a legmagasabb helyi értékű memóriacímbitet 0 értékre kell állítani akkor, ha a lapregiszter 3-as bitje (PB3) 0, (ez <!-- pdf 27 --> azt jelenti, hogy az U3 memóriaegységnek a 0. lapon kell megjelennie), és a 0. lap lett megcímezve (NPG0=0, ez a jel a lapdekóder 0. laphoz tartozó kimenete).

A címmódosító áramkör csak a 64k+ változatba lett beépítve, ily módon az U3 jelű RAM csak a 32k és 64k+ változatoknál érhető el a 0-3FFFH címtartományban (természetesen a 32k változatnál memóriabővítés szükséges). A 64k változatnál kisebb módosítást kell elvégezni, amelyeket a 3.2.2 pontban ismertetünk.

Az U3 átlapozási lehetőségére csak akkor van szükség, ha VT-DOS operációs rendszert kíván a felhasználó alkalmazni, egyébként a 64k verziónál a lehetőség hiánya semmilyen zavart, vagy kényelmetlenséget nem okoz.

A memória IN bemenetére az adatbusz D0-D7 jele kerül. A beírást a processzor NWR jele végzi.

A memória kimenetei egy adatbuszmeghajtóra kerülnek, amely az adatot akkor engedi az erősített adatbuszra (BD0-BD7), ha olvasás történik (NRD jel aktív) és az NCAS és NCAS64 jelek valamelyike aktív.

#### Rendszer ROM

Az operációs rendszer és a BASIC interpreter számára a 24 kbyte-nyi ROM terület áll rendelkezésre. Mivel a lapozó áramkör 16 kbyte-os fizikai memóriákat kezel, ezért a rendszer ROM egy 16 kbyte-os és egy 8 kbyte-os részre oszlik. Az előbbi az NROM, az utóbbi az NROM5 szelekciós jeleket kapja. A 16 kbyte-os ROM a 0. és 3. lapon jelentkezhet, a 8 kbyte-os csak a 3. lapon, a bővítő egységek 8 kbyte-os memóriájával együtt.

A rendszer ROM különböző kapacitású memóriachipekből épülhet fel. A 32k és 64k TVC változatnál 5 db 4 kbyte-os, 3 db 8 kbyte-os vagy 1 db 16 kbyte-os és 1 db 8 kbyte-os tok felhasználására van lehetőség, a 64k+ verziónál 4 kbyte-os chipek használatának lehetősége megszűnt. A <!-- pdf 28 --> különböző kapacitású elemek használatához a vezérlési feltételeket átkötésekkel kell módosítani. A 16 kbyte-os terület egyes memóriatokjainak engedélyező jelét egy dekóder állítja elő.

A 16 kbyte-os és 8 kbyte-os ROM külön-külön beállítható módon képes WAIT kérést generálni utasítás lehíváskor.

#### Video RAM

A video RAM feladata a mindenkori képtartalomra vonatkozó információ tárolása, bár a TVC felépítése azt sem zárja ki, hogy programtárolásra használjuk.

Kapacitása a 32k és 64k változatban 16 kbyte, a 64k+ változatban 64 kbyte.

A videomemóriához tartozó, és azt körülvevő funkcionális egységek a 7. ábrán láthatók.

Maga a memóriablokk ugyanolyan dinamikus RAM, mint amelyet a 2.2.2 pont felhasználói RAM szakaszában ismertettünk. A címbiteket sor- és oszlopcímekre osztva, multiplexelve kapja (VDA0-7). A sor- és oszlopcímek közötti szelekciót az NVMUX jel vezérli, a sorcímeket az NVRAS, az oszlopcímeket az NVCAS jel érvényesíti. Az írást az olvasástól az NVWR jel különbözteti meg.

A memória adatbemenetére a közvetlen adatbuszon (D0-7) küld adatot a CPU. A memóriakimenetek jelét az NVCAS jel felfutó (hátsó) éle a kimeneti regiszterbe írja, ahonnan a CPU beolvashatja az erősített adatbuszon (BD0-7) át (az NBMREQ, az NRD és NVRAM jelek egyszerre jelentkező 0 szintű impulzusával).

Az eddig elmondottak teljesen hasonlóak a felhasználói RAM vezérléséhez. A lényeges különbséget a videomemória és a felhasználói RAM között az okozza, hogy a videomemóriát két eszköz használja, a CPU és a megjelenítő logika (CRTC).

<!-- pdf 29 -->

Mindkettő 14 címbitet szolgáltat, a CPU az A0-A13, a CRTC az MA0-MA11 és RA0-RA1 jelekkel címzi a videomemóriát. A cím multiplexer nemcsak a sor- és oszlopcímek, hanem a CPU-CRTC címek között is választ. Ez utóbbi szelekciót a DEB jel végzi. A cím multiplexer ez esetben a memória címbemeneteire jutó jelek (VDA0-7) mindegyikére négy külső jel valamelyikét engedi.

A CRTC a videomemóriát folyamatosan, ciklikusan olvassa. A memória olyan gyors memóriachipekből van felépítve, hogy két ilyen olvasás között (640 ns időszakonként 320 ns időtartamot vesz igénybe egy ilyen olvasás) lehetőség van a CPU memóriahozzáférési igények kielégítésére.

Az egész folyamat nagyon szoros időzítést kíván. Az időzítő áramkör biztosítja a DEB jel és a videomemória vezérlő jeleinek megfelelő szinkronizmusát. A 2.2.4 pont részletesen ismerteti a jeleket és előállításukat.

A videomemória további kiegészítő egységei csak a 64k+ változatnál találhatók meg.

A 16 kbyte-os memóriának csak 7 címbitre van szüksége (VDA0-6), a 64 kbyte-os egy többletcímbitet (VDA7) kíván. A VDA7 címvezetéken jelentkező sor- és oszlopcím négy kombinációja a 64 kbyte-os video-memóriát négy 16 kbyte-os egységre osztja. Jelöljük ezeket V0-V3 névvel.

A CPU címtartományában a videomemória az 1-es és a 2-es lapon (4000H-7FFFH, ill. 8000H-BFFFH címtartomány) jelenhet meg. A video lapozó regiszter 2 bitje azt jelöli ki, hogy a V0-V3 közül melyiket érje el a CPU az 1-es lapon, a regiszter másik 2 bitje a 2. lapon megjelenő videomemória egységet határozza meg. Szükség van továbbá a megjeleníteni kívánt 16 kbyte-os egység kiválasztására is, ehhez további két bitre van szükség. A video lapozó regiszter ily módon 6 bites. A három szelekció teljesen független egymástól, lehetséges pl., hogy a V0 jelentkezik az 1. és 2. lapon is, és az is jelenik meg a képernyőn, <!-- pdf 30 --> (ez az állapot áll be bekapcsolás, vagy RESET nyomógomb működtetése hatására, amikor is az NRESET jel a video lapozó regisztert törli).

A megjelenítendő terület kijelölését a video lapozó regiszter egyértelműen elvégezheti, a CPU címtartományban való megjelenéshez azonban a lapozó áramkör megfelelő állapota is szükséges. A lapozó áramkör határozza meg például, hogy az 1. lapon a felhasználói RAM vagy a videomemória jelenjen-e meg és a videolapozásnak csak akkor van szerepe, ha a lapozó áramkör a videomemóriát választja ki.

A videolapozó regiszterbe a CPU I/O utasítással tölti be a 6 bites információt. Az órajel kialakító tulajdonképpen csak kiegészítése az I/O címdekódernek (l. 2.2.3 pont), a tőle érkező NPG0 dekódolt címjelet az A2 és A3 címbitekkel és az NBWR jellel kapuzva a 0FH I/O címre történő íráskor állít elő a video lapozó regiszter töltését eredményező impulzust.

Az erősített adatbusz hat vezetékének szerepe a lapozó regiszterbe töltött információ vonatkozásában a következő:

<table>
<tr><td colspan="2">Megjelenítés</td><td colspan="2">2. lap</td><td colspan="2">1. lap</td></tr>
<tr><td>oszlop</td><td>sor</td><td>oszlop</td><td>sor</td><td>oszlop</td><td>sor</td></tr>
<tr><td>BD5</td><td>BD4</td><td>BD3</td><td>BD2</td><td>BD1</td><td>BD0</td></tr>
</table>

A VDA7 memória címvezetékre a video lapozó multiplexer a lapozó regiszter hat kimenőjelének valamelyikét kapcsolja. A szelekcióhoz ugyanazt az NVMUX és DEB jeleket használja, mint a címmultiplexer, azonban mivel a CPU műveletnél az is érdekes, hogy az 1. vagy 2. lapot címzi-e a CPU, ezért a lapozó multiplexer az NPG1 jelet is megkapja. Az NPG1 jel akkor 0, ha a CPU az 1. lapot címzi, egyébként 1 (l. 2.2.2 pont).

<!-- pdf 31 -->

#### Lassú memóriák használata

A mikroprocesszor egy memóriaolvasást három óraütem alatt hajt végre. Az első óraütemben kiadja a memóriacímet, majd az NMREQ és NRD jelekkel kijelöli a művelet jellegét. A második óraütemben ezt a buszállapotot fenntartja. A harmadik óraütem közepén mintát vesz az adatbuszállapotáról feltételezve, hogy a memória ekkorra már érvényes adatot helyezett oda. A memória számára – a jelek késleltetését is figyelembe véve – ily módon mintegy két óraütemnyi idő áll rendelkezésre, hogy a cím megérkezése után az adatot a buszra helyezze. A TVC-ben az óraütem 320 ns. A memóriák rendelkezésére álló 640 ns időt azonban jelentősen csökkenti az, hogy a lapozó áramkörön jelentős időveszteséggel alakul ki az egyes fizikai áramkörök jelentős engedélyező jele. Lassú (nagy hozzáférési idejű) memóriaelemek használatakor elképzelhető lenne, hogy a memória még nem helyezte az adatot a buszra, amikor a processzor már mintát vesz.

A probléma megoldását az teszi lehetővé, hogy a Z80A mikroprocesszor rendelkezik egy WAIT jelű bemenettel. A memóriaolvasás második óraütemében a processzor megvizsgálja a bemenet állapotát és amennyiben ott aktív (0) szintet talál, akkor nem fejezi be a memóriaolvasási folyamatot, hanem további egy óraütemig változatlanul hagyja a buszjeleket (WAIT állapot). Az NWAIT bemenet figyelése addig tart, amíg azon inaktív állapot áll be. Ezután a processzor befejezi a memóriaolvasást. A memória számára ily módon a CPU óraütem egész számű többszörösének megfelelő többletidő biztosítható.

A memória olvasáshoz hasonlóan a memóriaírás és a perifériaműveletek is megnyújthatók.

Különösen nagy jelentősége van a WAIT kérés lehetőségének az utasításlehívás (FETCH) folyamatában, ugyanis az ekkor lezajló memóriaolvasásra egy fél óraütemmel kevesebb időt szán a processzor.

A TVC-ben használt memóriachipek hozzáférési ideje még utasításlehíváskor sem teszi szükségessé a WAIT kérést, a 32k és a 64k változat <!-- pdf 32 --> azonban fel van készítve arra, hogy a lassúbb memóriák (elsősorban ROM-ok) használhatók legyenek.

A 3. ábrán megtalálhatjuk a WAIT áramkört a hozzá vezető jelekkel.

Az NM1 jel utasításlehíváskor aktív. Ez a jel biztosítja, hogy a WAIT áramkör csak utasításlehíváskor működjön. A mikroprocesszor órája a WAIT kérést mintavételezi, ezzel megakadályozza, hogy az NWAIT jelen változás történjen abban az időszakban, amikor a processzor figyelembe veszi a jel állapotát.

Három részegység WAIT kérését kezeli az áramkör: a bővítő kártyák (NSLOWEXP jel), a programmodul (NFAST) és a rendszer ROM (NROM és NROM5 jelek). A rendszer ROM két szelekciós jele az alapegységen belül beköthető a WAIT áramkörbe, de az eddig gyártott készülékeknél ez a bekötés nem szükséges.

A 64k+ változatból a WAIT áramkör a korábbi tapasztalatokra alapozva kimaradt. A bővítő kártyákon természetesen szükség esetén fel lehet építeni.

### 2.2.3 I/O RÉSZEGYSÉGEK

A Z80A mikroprocesszor a TVC-ben a perifériák címzésére a címbuszának alacsonyabb helyi értékű 8 bitjét használja (l. a Z80A leírása, 6.1 alfejezet). Perifériát érintő műveletnél a processzor a program által kijelölt perifériacímet a címbuszra helyezi, majd az NIORQ jelen logikai 0 szintű impulzust generál. Az impulzussal gyakorlatilag egyidejűleg vagy az NRD, vagy az NWR jelen is 0 szintű impulzus jelenik meg attól függően, hogy a perifériát olvasásra vagy írásra használja-e a CPU. Fontos jellemzője a perifériát érintő műveleteknek, hogy ilyenkor az NM1 jel inaktív, mivel az NIORQ és NM1 jelek együttes aktivitása nem perifériaműveletet, hanem IT kezelési folyamatot jelez.

<!-- pdf 33 -->

A 8 bites címből következően 256 perifériacím létezik, természetesen külön-külön írásra és olvasásra.

#### I/O címdekóder

A TVC-ben a központi egység perifériacímeit egy dekóder áramkör fogadja. Amikor az áramkör a buszon I/O műveletet jelző állapotot érez, az I/O cím által kijelölt kimenetén impulzus jelenik meg, amely indítja az e címhez rendelt áramkör működését.

A dekóder kimeneti jelei között vannak olyanok, amelyek egyetlen 8 bites címkombinációt jeleznek, azt is figyelembe véve, hogy írás vagy olvasás történik-e. Ezek a jelek közvetlenül kiváltják a hozzájuk rendelt feladat végrehajtását. Más kimenetek több címkombináció esetén, esetleg input és output műveletnél is aktivizálódnak. E kimenetek jelét – ha szükséges – a jelet felhasználó részegységekben a címbusz egyes jeleivel, vagy a központi egység NWR, ill. NRD jeleivel kapuzzák össze.

A TVC a mikroprocesszor által kijelölhető 256 perifériacím közül csak 128-at használ.

Az I/O dekóder kimeneti jeleit a hozzá tartozó címeket és a címzett áramköröket a következő táblázat tartalmazza (vö. a Hardware–software interface című fejezet I/O címkiosztás című pontjával):

| Jel | I/O cím | Irány | Címzett áramkör |
|---|---|---|---|
| NPGD | 0-0FH | OUT/IN | Video lapregiszter |
| NWR0 | 0 | OUT | Borderregiszter |
| NWR1 | 01H | OUT | Nyomtató adatregiszter |
| NWR2 | 02H | OUT | Lapregiszter |
| NWR3 | 03H | OUT | Billentyűzetregiszter<br>Bővítő lapregiszter |
| NWR4 | 04H | OUT | Hangfrekvencia-regiszter L byte |

<!-- pdf 34 -->

| Jel | I/O cím | Irány | Címzett áramkör |
|---|---|---|---|
| NWR5 | 05H | OUT | Hangfrekvencia-regiszter 4 bit<br>Hangoszcillátor és IT eng. |
| NWR6 | 06H | OUT | Magnetofon motorkapcsolás<br>Video üzemmód-regiszter<br>Hang amplitúdóregiszter<br>Nyomtató csatoló adatérvényesítés |
| NWR7 | 07H | OUT | Cursor- és hang IT kérés törlése |
| NSLOT0 | 10H-1FH | OUT/IN | CH1 bővítő csatlakozó |
| NSLOT1 | 20H-2FH | OUT/IN | CH2 bővítő csatlakozó |
| NSLOT2 | 30H-3FH | OUT/IN | CH3 bővítő csatlakozó |
| NSLOT3 | 40H-4FH | OUT/IN | CH4 bővítő csatlakozó |
| NMFCK | 50H-57H | OUT/IN | Magnetofon felé adat |
| NRDTAST | 58H, 5CH | IN | Billentyűzet állapotszó |
| NCLRINT0 | 58H, 5CH | OUT | CH1 IT kérés törlése |
| NRDINT | 59H, 5DH | IN | IT állapotszó |
| NCLRINT1 | 59H, 5DH | OUT | CH2 IT kérés törlése |
| NIDENT | 5AH, 5EH | IN | Bővítőket azonosító állapotszó |
| NCLRINT2 | 5AH, 5EH | OUT | CH3 IT kérés törlése |
| INSTART | 5BH, 5FH | IN | Hangoszcillátor RESET |
| NCLRINT3 | 5BH, 5FH | OUT | CH4 IT kérés törlése |
| NWCOL | 60H-6FH | OUT/IN | Szín- (paletta-) regiszter |
| NCRTC | 70H-7FH | IN/OUT | CRTC regiszterei |

A dekóder vázlata a 8. ábrán látható. A működés megértéséhez célszerű az I/O címkiosztást (4.2.1 pont), a dekódolt I/O jeleknek a pont elején található listáját és a 8. ábrán található vázlatot együtt vizsgálni. Mint a korábbiakból ismert, a CPU a perifériák (I/O eszközök) címzésére a címbuszának alacsonyabb helyi értékű 8 címvezetékét (A0-A7) használja, megkülönböztetve az írást és olvasást, így 256 input és 256 output I/O eszközt tud kezelni. A TVC-ben azonban a 256 címból álló címtartománynak csak az egyik felét, a 0-tól a 127-ig terjedő tartományt használjuk. Ugyanez a tartomány a szokásos hexadecimális számrendszerben a 0-7FH értékek közötti címeket foglalja magába.

<!-- pdf 35 -->

A hexadecimálisan leírt I/O címek 16-os helyi értékű (első) jegyét a 8 bites cím magasabb helyi értékű négy bitje (A4-A7) hordozza, pontosabban csak az A4-A6 bitek, mivel a TVC-ben az A7 mindig 0 kell, hogy legyen.

A 8. ábrán elsődleges dekóder elnevezéssel jelölt blokk ezt az első hexadecimális számjegyet határozza meg. A kimenetein szereplő számok a hexadecimális I/O cím első számjegyének felelnek meg. Az elsődleges dekóder megkapja az A4-A6 címbiteket, amelyek lehetséges 8 bitkombinációjának mindegyike 8 kimenet valamelyikét jelöli ki. Ahhoz azonban, hogy a kimenetek valamelyikén aktív (0 szintű) impulzus jelenjen meg, az A7 címbitnek 0 értékűnek kell lennie. Az impulzus kialakulásához az NIORQ jelen 0, az NM1 jelen 1 érték is szükséges (az NIORQ=0 jelzi az érvényes I/O címet kivéve, ha az NM1=0 is fennáll, mert utóbbi IT lekezelési állapotot jelez).

Az elsődleges dekóder egy-egy kimenete 16 input és 16 output címet reprezentál. Ezeket még szét lehet választani, felhasználva az alacsonyabb helyi értékű címbiteket (A0-A3), valamint az NRD és NWR jeleket. A szétválasztás azonban csak a 0. és 5. kimenet esetében történik meg, a többi jelet az I/O eszközök, vagy azok csoportja közvetlenül kapja. Ennek kétféle oka lehet:

- A kimenet jelét olyan nagyobb integráltságú eszköz kapja, amely igényli a CPU bizonyos számú címvezetékét és azok jelét belül dekódolja. Ilyen az NWCOL jel, amely a palettaregiszter (l. megjelenítő egység) I/O szelekciós jele. A palettaregiszter egy tokban 4 db, egyenként 4 bites regisztert tartalmaz, és a négy regiszter megkülönböztetésére a CPU A0 és A1 címvezetékét is megkapja. Az NWCOL jel a regiszter előtt kapcsolatba kerül az NWR jellel, így csak írásra használható.

  Az NCRTC jel a CRTC nagyintegráltságú IC szelekciós jele, egyaránt használható a CRTC-be való adatírásra és belőle olvasásra, de a CRTC a belső regisztereinek megkülönböztetésére használja az A0 címbitet is. Tekintettel arra, hogy a palettaregiszter kettő, a CRTC egy

<!-- pdf 36 -->

  címbitet használ, a palettaregiszter négy, a CRTC két címet foglal el a címtartományban.

  Az NWCOL és az NCRTC jelek által képviselt 16 címből álló tartományban ezért a palettaregiszter egyes belső regiszterei minden második címen, összesen nyolcszor jelentkeznek. Például a CRTC ún. címregisztere a következő címeken érhető el: 70H, 72H, 74H, 76H, 78H, 7AH, 7CH és 7EH. Természetesen célszerű mindig ugyanazt a címet (pl. 70H) használni.

- A bővítő kártyahelyekbe helyezett rendszerelemek I/O cím igényét előre nem lehet meghatározni, hiszen egyre újabb elemek kerülhetnek fejlesztésre, azon kívül a legtöbb elem bármelyik kártyahelyre kerülhet. Az NSLOT-3 jelek ezért nincsenek tovább bontva. A bontást csak a bővítő kártyán lehet elvégezni a szükségleteknek megfelelően. A lehetőség adott: a bővítő kártyahelyeken az összes címvezeték, valamint az NBWR és NBRD jelek is megtalálhatók.

A 8. ábrán látható módon az elsődleges dekóder 0. és 5. kimenete további bontásra kerül.

A másodlagos dekóder 0. kimeneti jele alakítja ki az NWR0-7 jeleket. Ehhez az A0-A3 címbiteket használja, pontosabban a dekóder csak az A3=0 esetén engedélyezi kimenetei működését. Az NWR jel használata biztosítja, hogy a dekóder által előállított jelek csak periféria íráskor aktívak. A dekóder kimeneteire írt számok a hexadecimális cím második helyi értékével egyeznek meg, tehát pl. az NWRl jel akkor aktív, ha a CPU a 01H címre akar írni. Az elsődleges dekóder 0. kimenetének jele (NPGD) a video lapozó regiszterhez is kerül (l. 7. ábra), ahol az NBWR és az A2, A3 címbitek kapuzzák, így a regiszter írása a 0CH (valamint 0DH-0FH) címeken történhet.

A 8. ábrán látható kapuáramkör az 5. dekóderkimenetet az A3 címbittel kapuzza. Az így létrejött NMFCK jel (magnetofon csatoló kimenő adatát meghatározó ütemjel) az 50H-57H tartományban bármely címen az írás vagy olvasás esetén is aktív.

<!-- pdf 37 -->

A másodlagos dekóder 5W az 58H-5BH címek és az NWR jel 0 szintjének megjelenése esetén alakítja ki a bővítő egységek IT kérését törlő NCLRINT0-3 jeleket. Mivel ez a dekóder az A2 címbitet nem használja, ezért az NCLRINT0-3 jelek impulzusának kialakítása az 5CH-5FH címekre való írással is történhet.

A másodlagos dekóder 5R az 58H-5BH input címek (NRD=0) dekódolását végzi. A címtartomány ismétlődésével kapcsolatban ugyanaz a helyzet, mint az 5W dekódernél. A kimenő jelek közül az NRDTAST a billentyűzet állapotszó, az NRDINT az IT állapotszó, az NIDENT a bővítő kártya azonosító állapotszó beolvasását, az INSTART pedig a hanggeneráló számlánc törlését végzi.

#### Billentyűzet

A billentyűzetcsatoló vázlata a 9. ábrán látható.

Valamely nyomógomb benyomásakor a mechanika egy gumiharangot szorít egy NYÁK lemezhez. A gumiharang belsejében egy elektromos vezető gumiérintkező található. Ez az érintkező a NYÁK lemezhez szorítva rövidrezár két alkalmasan kialakított vezető sínt.

A billentyűk logikailag egy mátrixban helyezkednek el. A mátrix sorait és oszlopait vezetékek alkotják. Minden nyomógomb alatt egy sor- és egy oszlopvezeték halad el. A gomb lenyomásakor a vezetőgumi érintkező a sor- és oszlopvezetőt zárja rövidre.

A sorvezetékek közül egyidejűleg egyre logikai 0 szintet lehet állítani. Amennyiben egy gomb lenyomott állapotban van, és a hozzá tartozó sorvezeték 0 állapotú, akkor ez a 0 szint megjelenik a gombhoz tartozó oszlopvezetéken is.

A CPU a sorregiszter feltöltésével kijelöl egy mátrixsort. A regiszterben 4 biten kódolva jelenik meg a sor címe. Ebből a kódból a sordekóder a mátrix 10 sorának kijelölését végző jeleket állít elő. A CPU által kijelölt sorvezeték állapota 0 lesz. A CPU ezután beolvas egy <!-- pdf 38 --> állapotszót, amely a 8 oszlopvezeték állapotát tartalmazza. Amennyiben a kijelölt sorhoz tartozó valamelyik billentyű benyomott állapotban van, akkor a neki megfelelő oszlop állapotbitje 0 értékű lesz. A sor és oszlop azonosítása a billentyű azonosítását is lehetővé teszi.

A sorregiszter tartalmának inkrementálásával a mátrix valamennyi sora végig vizsgálható.

A billentyűk elhelyezkedése a mátrixban a 10. ábrán látható.

Az egyes nyomógombok nincsenek diódákkal szétválasztva, ezért 2-nél több nyomógomb egyidejű lenyomása hibás működést eredményezhet.

A 8. és 9. mátrixsor a botkormányok érintkezőihez tartozik (kivétel az INS billentyű). A 8. sor a billentyűzeten található botkormányt és a jobb oldalinak deklarált botkormányt is azonosítja, tehát ez a két eszköz párhuzamosan működik. Látható, hogy a beépített botkormány sem tűz, sem gyorsító gombbal nem rendelkezik.

A sorregisztert az NWR3 jel (03H című írás) tölti, az állapotszó beolvasását az NRDTAST jel (58H című olvasás) engedélyezi.

A 9. ábra a csatlakozókat is jelöli. A CH9 és CH8 csatlakozók a billentyűzet csatlakoztatására valók, és csak az alapegység szétszerelése után láthatók a készülék belsejében, a CH6 és CH7 csatlakozók a készülék bal oldalán egymás mellett találhatók. Az ábrán megtalálhatók a csatlakozó érintkezőinek és a jeleknek az összerendelése.

#### Magnetofoncsatoló

A magnetofonszalagra rögzítés céljából a TVC az adatokat sorba rendezi és frekvenciamodulált jel formájában juttatja a magnetofonhoz. A frekvenciamoduláció ez esetben ún. frekvenciabillentyűzést jelent, azaz a logikai 0 szintnek megfelel egy rögzített frekvencia, a logikai 1 szintnek egy másik.

<!-- pdf 39 -->

Betöltéskor a magnetofon által kiadott frekvenciamodulált jelet vissza kell alakítani 0 és 1 szinteket váltogató bitfolyammá, majd ezt feldolgozva a memóriába a megfelelő helyre helyezni.

A TVC hardware a fenti feladatokhoz csak nagyon kevés támogatást nyújt, a munkát a software végzi el.

A magnetofon felé menő adat úgy áll elő, hogy a CPU megfelelő ütemben I/O paranccsal egy flip-flop-ot ellenkező állapotúra tud váltogatni. A flip-flop jele feszültségosztás és szűrés után közvetlenül a magnetofonbemenetre kerül. Ez azt jelenti, hogy az adatok sorba rendelését a program végzi el, és az adatbiteknek megfelelően a flip-flop-ot gyorsabb vagy lassabb ütemben billegteti, hogy létrejöjjön a frekvenciabillentyűzés.

A magnetofonról bejövő adat szintbeállítás után egy állapotbiten keresztül a CPU számára elérhető. A jelváltások sűrűségéből állapítja meg a program a logikai értéket.

A motorvezérlést nagyáramú meghajtók végzik, amelyek beállításához egy kétbites regiszter tartalma nyújt információt.

A regisztert az NWR5 jel (05H írás) tölti, a kimenő adat flip-flop-ot az NMFCK jel (50H írás vagy olvasás) billenti, a bejövő adat az IT állapotszóban (NRDINT jel, 59H olvasás) kerül a processzorhoz.

#### Nyomtatócsatoló

A TVC Centronics típusú nyomtató csatolója a következő jelekkel kommunikál a nyomtatóval:

| D0-D7 | adatbitek |
|---|---|
| NDSTRB | adatérvényesítő jel a TVC-től a nyomtató felé |
| NACK | a nyomtató vételt nyugtázó jele |

<!-- pdf 40 -->

Az adatvonalak ponált értékeket továbbítanak, a két vezérlőjel aktív szintje a logikai 0.

Az adatbyte-ok átvitele a következőképpen zajlik:

A TVC a D0-D7 adatvonalakra helyezi az adatot (karakterkód), majd az NDSTRB jelet 0 szintbe állítva jelez a nyomtató felé, hogy az adat érvényes.

A nyomtató az adat átvétele után a NACK jel 0 szintű impulzusával nyugtázza a vételt. A TVC ennek hatására megszünteti az adatérvényesítő NDSTRB jel aktív állapotát. A TVC a következő karakterkódot csak akkor küldi, ha az NACK jel újra logikai 1 szintbe állt.

A nyomtatócsatoló és a CPU közötti kapcsolatot biztosító jeleket a 11. ábrán láthatjuk, amely az I/O áramköröket mutatja be.

A CPU a nyomtatócsatolót az alábbiak szerint vezérli:

A csatoló egy 8 bites adatregisztert tartalmaz, amelyet a CPU a BD0-BD7 erősített adatbuszról a NWR1 I/O szelekciós jellel tölt fel.

Az NDSTRB jelet egy D flip-flop (NDSTRB flip-flop) kimenete szolgáltatja. 0 szintbe állítását a CPU az NWR6 jellel a BD7 adatbuszjelről való betöltéssel tudja elvégezni.

Az NDSTRB jel 0 szintje egy másik D tárolót (NACK flip-flop) 0 szintbe állít. E tároló kimeneti jele az SB7, amelyet az IT állapotszóban a CPU be tud olvasni.

Az NACK jel 0 szintű impulzusa az NDSTRB flip-flop-ot 1 szintbe állítja, tehát a CPU-nak nem kell foglalkoznia az adatérvényesítés megszüntetésével.

Az NACK jel felfutó éle (az impulzus hátsó éle) az NACK flip-flop-ot 1 szintbe állítja. Ezt beolvasva a CPU tudomást szerez róla, hogy újabb adatot lehet küldeni.

<!-- pdf 41 -->

Az NRESET jel az adatregisztert törli (az összes adatbit 0 értékű lesz), az NACK flip-flop-ot 1 szintbe állítja.

#### Hangképzés

A TVC alapegység hangképzési lehetőségeit a 12. ábrán látható vázlat alapján követhetjük végig.

A megszólaltatni kívánt hang frekvenciájára vonatkozó 12 bites információt a CPU két perifériaírással (NWR4 és NWR5) a hangfrekvencia regiszterbe tölti. A regiszter kimenete egy programozható számlánc osztásarányát határozza meg. A számlánc a 3M125 jel éleit számolja. 12 bites programozható és 4 bites fix osztásarányú része van. A hang előállításához szükséges jel mellett a soros vonali interface adásivételi órajelét (TxRxCLK) is előállítja.

A számlánc kimenőjele az E/D kapukra kerül. Továbbjutását az E/D regiszter állapota engedélyezi vagy tiltja. A regiszter 2 bites, az NWR5 jel tölti. Egyik bit a számlánc jelének IT kérésre való felhasználását engedélyezheti (SDINT jel). Az IT kérés lehetőségének az a jelentősége, hogy a program számára pontos időzítést tesz lehetővé (TIME OUT). A számlánc az INSTART jelen kialakuló impulzus hatására a frekvenciaregiszter által meghatározott állapotból kiindulva kezd számolni. A számlánc telítésekor az engedélyezett IT jelzést ad a processzornak.

Az E/D regiszter másik bitje a számlánc jelének hanggenerálásra való felhasználását engedélyezheti.

A számlánc jele az E/D kapuktól az amplitúdó szabályozóra jut. Ez az áramkör egy programozott feszültségosztó, amely a számlánc jelének szintjét 16 lehetséges érték valamelyikére állítja be. A 16 szint közül a 4 bites amplitúdóregiszter tartalma választ. A regisztert az NWR6 jellel a CPU tölti fel.

<!-- pdf 42 -->

- A palettaregiszter kétszínű és négyszínű üzemmódban két, ill. négy színt választ ki a lehetséges 16 szín közül. Négy darab négybites regisztert tartalmaz. Feltöltését az NWCOL jel végzi, összekapuzva az NWR jellel. A négy regiszter megkülönböztetése az A0 és A1 címbitek felhasználásával történik, így ezek I/O címe rendre: 60H, 61H, 62H és 63H. Mivel az NWCOL jel egy 16 címből álló tartományt jelöl ki, így mindegyik regiszter négy-négy címen érhető el, pl. a 60H címen elérhető regiszter elérhető a 64H, 68H és 6CH címen is. Az adatok a BD0, BD2, BD4 és BD6 adatvezetékekről kerülnek a regiszterekbe.
- A borderregiszter a képkeret színére vonatkozó információt tárolja négy biten. A BD1, BD3, BD5 és BD7 adatvezetékek állapotát az NWR0 jel írja a regiszterbe. Az NRESET jel törli ezt a regisztert, a fekete színnek megfelelő állapotot állítva be.
- A videolapozó regiszter csak a 64k+ változatú TVC-ben található meg, feladatát a 2.2.2 pont részletesen ismerteti. A 6 bites regiszterbe a BD0-5 adatbiteket az NPGD jel írja, összekapuzva az A2 és A3 címbitekkel és az NBWR jellel. A regisztert a CPU ily módon a 0CH címen érheti el, de a 0DH, 0EH és 0FH címek is e regisztert jelölik ki.

#### Egyéb I/O eszközök

A 11. ábra alapján megállapítható, hogy még nem beszéltünk rendszerbővítő eszközök és a lapozó áramkör I/O eszközeiről, továbbá két állapotszó beolvasását lehetővé tevő áramkörökről.

A bővítő eszközök mindegyike 16 input és 16 output perifériacímet alkalmazhat. A négy bővítő csatlakozó egy-egy NSLOT jelet kap, amely kijelöli a hozzá csatlakoztatott bővítő eszköz I/O címtartományát. A legtöbb bővítő eszköz bármelyik csatlakozóba helyezhető, de programozáskor tekintetbe kell venni, hogy ilyenkor az I/O címek megváltoznak. Pl. ha a CH1 csatlakozóba helyezve valamely funkció a 11H címen érhető

<!-- pdf 43 -->

A számlánc jelét használó hangjelzést letiltva az amplitudóregiszter és az amplitúdószabályozó egy másfajta hangképzést tesz lehetővé. Az amplitúdóregiszterbe megfelelően megválasztott ütemben váltakozó értéket írva az amplitúdószabályozó kimenetén egy lépcsős jel keletkezik többé-kevésbé tetszőleges hangeffektust kiváltva. Ilyenkor az amplitúdószabályozó digitál/analóg konverterként működik.

#### A megjelenítő áramkörök I/O eszközei

A megjelenítő áramkörökről a 2.2.5 pont részletesen szól (az I/O eszközökről is), itt csak az I/O kezelés ismertetésének teljessége kedvéért foglaljuk össze azt, hogy a CPU miképpen kezeli a megjelenítő logikát.

A 11. ábrán látható, hogy a megjelenítő áramkörök öt I/O eszköze melyik jeleket használja:

- Az adatmegjelenítés folyamatot vezérlő CRTC az I/O címtartományból két címet igényel. Az I/O dekódertől érkező NCRTC jel a 70H-7FH címtartomány megcímzése esetén aktív. A CRTC a két címmegkülönböztetésére az A0 címbitet is megkapja. A0=0 feltétel a CRTC címregiszterét jelöli ki, az A0=1 a CRTC többi regiszterét. Az írási folyamatot az olvasástól a CRTC-re vezetett NWR jel különbözteti meg. A CPU és a CRTC közötti adatforgalom a D0-D7 adatbuszon zajlik. (A CRTC az egyetlen I/O eszköz, amely nem az erősített adatbuszon kommunikál a CPU-val.)

  Mivel az NCRTC jel 16 címet reprezentál és a CRTC ebből csak kettőt használ, a 70H-7FH tartományban a CPU két címe nyolcszor jelentkezik (pl. a címregiszter a páros címek mindegyikén).

- Az egyidejűleg megjeleníthető színek számát a kétbites üzemmódregiszter határozza meg. A BD0 és BD1 adatbiteket az NWR6 jel tölti a regiszterbe.

<!-- pdf 44 -->

el, akkor a bővítő egységet a CH2 csatlakozóba helyezve ugyanazon funkció címe 21H lesz.

A lapozó áramkörről a 2.2.2 pont részletes ismertetést ad, kitérve a lapozó regiszter és a bővítő lapozó regiszter kezelésére is. Eszerint a lapozó regiszterbe a BD3-BD7 adatvezetékek tartalmát (64k+ verziónál a BD2-BD7 adatvezetékeket) az NWR2 jel tölti, a bővítő egységek lapozó regiszterébe pedig az NWR3 jel a BD6 és BD7 adatbiteket.

Az azonosító állapotszó 8 bitje közül 2-2 a bővítő kártyahelyekhez (csatlakozóhoz) van rendelve. A két bit négy kombinációja közül három a soros vonali interface, a floppy csatoló és a videojáték modul azonosítását teszi lehetővé a program számára. A negyedik kombináció (11) az összes bővítő egységnél fennáll, ez esetben az azonosítás egyéb módon történik.

A bővítő csatlakozóktól érkező adatbitek egy buszmeghajtón keresztül a BD0-BD7 adatbuszra kerülnek, ha ezt az NIDENT jel lehetővé teszi (5AH input cím).

Az IT állapotszó 5 bitje a lehetséges IT okok azonosítását teszi lehetővé. Négy IT a bővítő csatlakozókról érkezhet.

A CRTC ún. cursor IT-je és a hangáramkörök által generált IT egy bitet foglal el az állapotszóban. Megkülönböztetésük csak a software által készített adminisztrációval lehetséges.

Az állapotszó további három bitje a következő jeleket továbbítja a CPU-hoz:

- a nyomtató csatoló SB7 jelét, amely a nyomtató adatfogadó készségét jelzi
- a magnetofonról érkező soros adatjelet
- a színkikapcsoló (colour-killer) állapotjelét.

<!-- pdf 45 -->

Az IT állapotszót az NRDINT jel (59H input cím) kapcsolja az adatbuszra.

#### IT kezelés

A Z80A CPU a TVC-ben ún. 1. IT-módban működik. Ez azt jelenti, hogy IT kérés esetén a CPU a 38H címre ugrik. Az ott kezdődő programrészlet beolvas egy állapotszót, amelynek vizsgálata eldönti, hogy melyik IT jelentkezett. A vizsgálat eredményétől függően ágazik el a program az aktuális IT alprogram felé.

Az alapegység áramkörei közül IT-t kérhet a hangáramkör és a CRTC. A hangáramkör IT kérésével kapcsolatban elmondtuk, hogy időzítési célt szolgál. A CPU a hangáramkör számláncát egy, a kívánt időzítéstől függő meghatározott helyzetbe állítja. A beállított, kiindulóhelyzettől függő idő múlva a számlánc kimeneti jele IT kérésként jelentkezik a CPU-nál.

A CRTC CD jelű kimenetét használjuk a TVC-ben IT kérésre. Az IT kérés 20 ms időszakonként periodikusan megérkezik. Az IT kérés jelentősége egyrészt az, hogy a CPU a megjelenítési folyamat fázisáról információt szerez, másrészt az IT kérés periodikus mivolta miatt használni lehet bizonyos rendszeresen ellátandó feladatok időzítésére, pl. a billentyűzet lekérdezése is a CRTC IT kérését kiszolgáló alprogramban történik.

A CRTC CD kimenete egyébként cursor megjelenítésére szolgál. A 6845-ös CRTC a cursort világító téglalap formájában jelenítheti meg. A téglalap villogó vagy nem villogó, vízszintes mérete megegyezik a karakterpozíció méretével, függőlegesen pedig az egy rasztersor (tv-sor) szélességtől rasztersoronként a karaktersor méretéig terjedhet.

A TVC-ben a CRTC inicializálása alapján a cursor a karakter utolsó tv-sorában lévő nem villogó vonal. A megjelenítésére kijelölt karakter-pozíció a képernyő jobb alsó sarkában található. Mindezek alapján a CRTC CD kimenetén 20 ms (ez a képfelfrissítési ütem) időszakonként egy <!-- pdf 46 --> karakterütemnyi (640 ns) szélességű impulzus jelenik meg akkor, amikor az elektronsugár az utolsó karaktersor utolsó karakterének utolsó tv-sorát rajzolja.

A cursor és a hang IT kapcsolódik a bővítő kártyahelyekről érkező, IT generálásra képes jelekhez, ily módon összesen 6 db IT ok bármelyike kérhet kiszolgálást. Mindégyik IT kérő jel egy állapotszóba is be van kötve. A cursor és a hang IT az állapotszóban közös állapotbittel rendelkezik. Az IT elfogadásakor a CPU beolvassa az állapotszót, majd annak vizsgálata után az azonosított IT kérő áramkört kiszolgáló programrészre ugrik.

### 2.2.4 A TVC MŰKÖDÉSÉNEK IDŐZÍTÉSE

Az időzítő áramkörök az eddigi fejezetekben tárgyalt logikailag jól bontható TVC részegységektől eltérően egy meglehetősen bonyolult, sokszorosan hurkolt szekvenciális hálózatot valósítanak meg. Különösen igaz ez az órajelgenerátor, a video időzítő és a CPU órajelét előállító áramkörök esetében, amelyek kölcsönösen befolyásolják egymás működését. E szempontokat figyelembe véve az időzítő áramköröket már itt, a 2.2 alfejezetben is részletesebben tárgyaljuk. Emiatt találkozunk a többi részegység blokkvázlataival ellentétben az időzítés leírásához tartozó ábrákon flip-flop-okkal és kapuáramkörökkel. A könnyebb áttekintés érdekében az időzítő áramköröknek a kapcsolási rajzon sokszorosan összehurkolt részegységeit külön ábrán mutatjuk be. A működés megértéséhez a legfontosabbnak az idődiagramok alapos áttanulmányozását tartjuk.

#### A RESET funkció megvalósítása

Mint minden számítógépben, a TVC-ben is szükséges, hogy a készülék bekapcsolása után egy ismert alapállapotból induljon. Szükség lehet arra is, hogy ezt az alapállapotot a készülék kikapcsolása nélkül is elő tudjuk állítani.

<!-- pdf 47 -->

A fenti célokat a RESET áramkör valósítja meg. Az általa előállított aktív 0 szintű NRESET és aktív 1 szintű RESET jel a TVC-ben regisztereket, flip-flop-okat töröl és elvégzi a CPU alapállapotba állítását is.

A TVC három változatában nagyon hasonló a RESET funkció megvalósítása.

A TVC bekapcsolásakor egy ellenálláson át egy kondenzátor töltődik. A kondenzátor feszültsége vezérel egy TTL áramkört. Az R-C elemek megválasztása biztosítja, hogy mire a kondenzátor feszültsége eléri a TTL áramkör kapcsolási szintjét, addigra a készülék áramkörei már stabil tápfeszültségen vannak. A komparálási szint elérésekor a TTL áramkör kellő hosszúságú RESET jelet állít elő.

A 32k és 64k változatban az R-C elemek egy viszonylag nagy integráltságú IC (az oszcillátornál is használt 8224) bemenetére kapcsolódnak, a 64k+ esetében kapuáramkört vezérelnek.

A működés közben szükségessé vált RESET funkció kiváltása a RESET nyomógomb működtetésével érhető el. A nyomógomb egy, a bekapcsolási RESET áramkörhöz hasonló R-C tagot működtet, kisütve a kondenzátort. A nyomógombos RESET áramkör teljesen azonos a három változatnál.

#### Órajel-generátor

Az órajel-generátor áramkörei a TVC különböző funkcionális egységei számára a programtól és a képtartalomtól független fix frekvenciájú, egymáshoz képest rögzített fázisú négyszöghullámokat állítanak elő.

Az oszcillátor 25 MHz frekvenciájú a TVC mindhárom verziójában, az áramköri megvalósításában azonban eltérés mutatkozik. A 32k és 64k változatnál egy viszonylag nagy integráltságú IC (8224) tartalmazza az oszcillátort, a 64k+ változatnál diszkrét elemek, RC tagok és inverterek alkotják.

A 25 MHz-es órajelből négy flip-flop-ból álló osztó áramkör alakítja ki a 12,5 MHz-es, a 6,25 MHz-es és a 3,125 MHz-es jeleket. Az osztó <!-- pdf 48 --> áramkört a 13. ábra, a működését bemutató idődiagramot a 14. ábra tartalmazza.

Az 1. és 2. jelű flip-flop ún. J-K típusú. Az ábrán látható bekötés esetén, azaz akkor, ha a J és K bemenetén fix logikai 1 szint van, a flip-flop-ok a CP jelű bemeneteiken megjelenő jel minden lefutó élére ellenkező állapotba billennek.

A 3. és 4. flip-flop-ok ún. D típusúak. A CP bemenetük jelének felfutó élével mintavételezik a D bemenetük állapotát, ez jelenik meg a Q kimenetükön és a következő felfutó élig tárolják is azt.

Az idődiagram és a kapcsolási vázlat alapján a működés végigkövethető.

Az 1. flip-flop a 25 MHz-es jel frekvenciáját felezi. A flip-flop jelének újabb, a 2. flip-flop által végrehajtott frekvenciafelezésével állnak elő a 6,25 MHz-es jelek.

A 3,125 MHz-es jel nem a 6,25 MHz-es jelből áll elő frekvenciafelezéssel, hanem a 12,5 MHz-esből a 3. és 4. flip-flop-ok által végrehajtott frekvencianegyedeléssel. Ezt az indokolja, hogy a 3,125 MHz-es jelnek különböző fázisú változataira is szükség van. A jelek fázisát egymáshoz képest az rögzíti, hogy az 1., 3. és 4. flip-flop-ok a bekapcsolási RESET hatására rögzített állapotból indulnak.

A 6,25 MHz-es jelek fázisát az 1. kapu állítja be a 3,125 MHz-es jelhez. Az idődiagram azt mutatja, hogy a 6M25 jel rossz fázisban indul. Az 1. kapun kialakuló 0 szintű impulzus a 2. flip-flop-ot törli (hamis impulzus kialakulhat), ezután viszont beáll a szinkron. A későbbiek során az 1. kapu jele mindig olyankor törli a 2. flip-flop-ot, amikor az amúgyis 0 állapotú.

#### Video időzítő

A video időzítő áramkörök feladatainak megértéséhez az e fejezetben közölt információkat célszerű összevetni a 2.2.5 pontban a megjelenítő egységről és a 2.2.2 pontban a video RAM-ról elmondottakkal. A video <!-- pdf 49 --> időzítő áramkör a 15. ábrán, az általa létrehozott jelek idődiagramja a 16. ábrán látható.

A TVC az adatokat egy tv-készüléken, vagy egyéb CRT monitoron jeleníti meg, így megjelenítő egysége tulajdonképpen egy CRT display, mégpedig ún. bitmap grafikus display. A display áramkörök központi része egy ún. felfrissítő memória, amely az aktuális képtartalmat tárolja. Az üzemmódtól függően 1, 2 vagy 4 memóriabit egy-egy képpont színét határozza meg.

A felfrissítő memóriába a központi egység tölti be az adatokat. A központi egység olvashat is a memóriából.

A megjelenítést egy nagyintegráltságú IC, egy ún. CRT controller (CRTC) vezérli.

A video időzítő áramkör a CRTC számára órajelet biztosít. Az órajel az NCCLK.

A megjelenítéshez a CRTC 20 ms ütemidővel (felfrissítési ciklus) a videomemóriának egy képre vonatkozó teljes tartalmát kiolvassa byte egységekben.

A videomemória rövid hozzáférési idejű dinamikus RAM. A CRTC által kiolvasott byte megjelenítési ideje több mint kétszerese a RAM ciklusidejének. Ez lehetővé teszi, hogy a CPU-CRTC kettős hozzáférést időmultiplex módon lehessen megoldani.

A video időzítő áramkör a DEB jelet állítja elő a hozzáférés vezérlésére. Logikai 1 állapota esetén a CPU, 0 állapota esetén a CRTC férhet a felfrissítő memóriához.

A 16. ábrából látható, hogy a DEB jel és az NCCLK jel inverz kapcsolatban van.

Mivel a felfrissítő memória dinamikus RAM, ezért a 2.2.2 pontban el <!-- pdf 50 --> mondottak szerint a sor- és oszlopcím érvényesítő jeleket (NVRAS és NVCAS), továbbá a sor- és oszlopcímek multiplexelését vezérlő jelet (NVMUX) is elő kell állítania a video időzítő áramköröknek.

A 16. ábrán látható a három jel időzítése. Az NVRAS jel 1-0 átmenete után 40 ns idővel vált az NVMUX (0 állapota a sor-, 1 állapota az oszlopcímeket engedi a memóriára), majd újabb 40 ns múlva az NVCAS jel az oszlopcímeket érvényesíti az 1-0 átmenetével.

A memóriaírást jelző NVWR jel is az ábrán látható. A CPU számára kiosztott időszak, azaz a DEB jel 1 állapota idején az NVRAS és az NVWR jeleken attól függően alakul ki aktív (0 szintű) impulzus, hogy a CPU kíván-e foglalkozni a memóriával írás vagy olvasás céljából.

A CRTC által a memóriából kiolvasott adat egy párhuzamos beírású, soros kiolvasású regiszterbe kerül. A beírást az NLOAD nevű jel vezérli. Az idődiagramból látható, hogy beírásra minden CRTC hozzáférési művelet végén kerül sor.

A regiszterből az adatok (a képpontokra vonatkozó információ) kiléptetése az üzemmódtól függő frekvenciával történik. 2 szintű üzemmódban a 12M5, 4 színűben a 6M25, 16 színűben a 3M125 jel ütemében történik a kiléptetés.

A video időzítő jeleket előállító áramkör a 15. ábrán látható.

A CPU-CRTC memória-hozzáférés között választó DEB jelet a 3. flip-flop állítja elő. A J-K flip-flop az NVMUX jel lefutó éleire billen ellenkező állapotba, így a DEB jel az NVMUX-ból frekvencia felezéssel áll elő.

Az NVCAS jel előállítását az 1. és 2. D flip-flop-ok végzik. A hozzá tartozó időadatokat a 17. ábrán láthatjuk.

Az OR jelet az órajel-generátor 13. ábráján látható 3. flip-flop-ja <!-- pdf 51 --> hozza létre. 3,125 MHz frekvenciájú, csak fázisban tér el a 3M125 jeltől, il. az annak inverzeként létező NVMUX jeltől.

Az OR jelet a 2. és 1. flip-flop az N12M5 jel felfutó élével mintavételezve két ütemmel késlelteti. Az 1. flip-flop NQ1 kimenetén így áll elő az NVCAS.

Az NVRAS jel a 2. flip-flop NQ2 kimeneti jelének és a 3M125 jelnek az ÉS kapcsolatából áll elő. A 3. NAND kaput a 4. NAND kapu invertálja, ez biztosítja az ÉS kapcsolatot.

Az NVRAS jelen CRTC műveletnél mindig létre kell hozni a 0 szintű impulzust. Az 5 NAND kapun a DEB=0 feltétel 1 szintet állít be, így a 4. kapu átengedi az NVRAS jelet.

DEB=1 feltétel esetén, tehát a CPU számára fenntartott időben csak akkor kell az NVRAS-t aktivizálni, ha a CPU tényleg foglalkozik a videomemóriával és a CPU memória vezérlőjelei és a DEB jel között a szinkron létrejött. Ezt az STB jel 1 szintje jelzi. Az STB jelet a 18. ábrán látható áramkör állítja elő. A jelet a 3M125 jel mintavételezi, így a DEB jel következő váltásáig stabil marad az NQ4 kimenet. Érvényes CPU műveletnél STB=1, NQ4=0, így az 5. kapu átengedi az NVRAS jelet.

Az NLOAD jel az 1. NAND kapun alakul ki a 19. ábra szerint. Látható, hogy csak a CRTC műveletnél aktív. Az NVWR a 2. NAND kapun jön létre a 19. ábra idődiagramjának megfelelően. A WR jel a CPU NWR jelének negáltja, az STA jelet a 18. ábrán található áramkör szolgáltatja. Akkor 1 az értéke, ha a CPU a videomemóriát címzi és létrejött a szinkron a CPU és a videomemória vezérlés között.

#### A mikroprocesszor órajelének előállítása

A TVC mikroprocesszor a 3,125 MHz-es órajellel működik. Az órajel azonban nem egyezik meg a 2.2.4 pont második szakaszában ismertetett

<!-- pdf 52 -->

3M125 jellel. A mikroprocesszor órájára hatással van a videomemóriához való hozzáférés.

Mint az előző fejezetből ismert, a TVC-ben a videomemóriához való kétoldali (központi egység-megjelenítő egység) hozzáférés időmultiplex módon történik. A DEB jel 1 szintjének idején 320 ns ideig a központi egység foglalkozhat a memóriával, a következő 320 ns idő alatt (amit a DEB jel 0 állapota jelez) a megjelenítő egység. A memóriavezérlés szervezését azonban jelentősen bonyolítja az a tény, hogy a Z80A mikroprocesszor által végrehajtott videomemóriára írási és olvasási folyamatot nem lehet rögzített módon összeszinkronozni a DEB jellel, mivel a Z80A eltérő számú óraütem alatt hajtja végre a különböző utasításokat. A szinkront minden egyes videomemória műveletnél különkülön kell létrehozni.

A szinkronizálás úgy történik, hogy amikor a processzor a videomemóriához fordul, akkor a DEB jel és a processzor vezérlőjeleinek egymáshoz viszonyított időbeli helyzetétől függően egy áramkör a processzor órajelét leállítja. A leállás fél óraütemes (160 ns-os egységekben számolva 1-4 egység lehet (ún. CLOCK STRETCH művelet).

Tekintettel arra, hogy a DEB jel ciklusideje kétszerese a processzor órajelütemének, a memóriaművelet kezdetét jelentő T1 ciklus (l. Z80A ismertetőt, 6.1 alfejezet) négy helyzetet vehet fel a DEB jelhez viszonyítva.

A 20. ábra mutatja ezeket a lehetséges helyzeteket, valamint az órajel-módosításokat a különböző helyzetekben. Az ábrán is látható, hogy az órajel-módosítás eltérő módon történik a memória írás-olvasás és az utasítás lehívás során (a videomemória programtárolásra is alkalmas).

Az órajel-módosítás mértékét a következő szempontok határozzák meg:

- A videomemória kimenő adatát az NVCAS jel felfutó éle minden esetben egy regiszterbe írja, amely regiszter tartalma a processzor által <!-- pdf 53 --> beolvasható. A regiszterbe a beírás akkor is megtörténik, ha a megjelenítő logika olvas, valamint akkor is, ha a processzoron lenne a sor, de az nem kíván foglalkozni a videomemóriával. Ez utóbbi esetben hamis adat kerül a regiszterbe.

- A regiszter beolvasása a processzor által az utasításlehívási folyamatnál a T3 ciklus felfutó órajelére, a memóriaolvasásnál a T3 ciklus lefutó órajelére történik. Az adatnak meghatározott idővel (setup time) az él előtt rendelkezésre kell állnia.

A fenti két szempontból következik, hogy utasítás-lehívásnál a T2 ciklust kell megnyújtani úgy, hogy az órajelet 0 értéken kell tartani annyi ideig, hogy a T3 felfutó él a DEB jel egy 1-0 váltását kövesse (a DEB=1: a mikroprocesszoré a videomemória). Memóriaolvasásnál a T3 ciklust kell megnyújtani úgy, hogy az órajelet 1 értéken kell tartani annyi ideig, hogy a T3 lefutó él a DEB jel egy 1-0 váltását kövesse. Ily módon biztosítható, hogy a processzor számára kijelölt hozzáférési időtartam kezdetén a videomemórián legyen a cím, és hogy a memória kimeneti regiszterébe írt, a címhez tartozó byte olvasását a processzor elvégezze, mielőtt a regiszterbe az NVCAS jel a megjelenítő egység által kiolvasott adatot írja.

A CLOCK STRETCH áramkört a 18. ábra mutatja.

A 4. számú J-K flip-flop bemeneteit vezérlő 4. számú inverterkimenet mindaddig 1 szintű, amíg a processzor nem fordul a videomemóriához. A J-K flip-flop ilyenkor az N6M25 órajel lefutó éleire folyamatosan változtatja kimeneti állapotát, azaz frekvenciát felezve létrehozza a 3,125 MHz-es órajelet. Az 5. inverternek a feladata az órajel megfelelő szintjének biztosítása.

A 4. inverterkimenet 1 szintje az 1. kapukimenetet 0 szinten tartja.

Ha a processzor a videomemóriát címzi írás vagy olvasás céljából, az NVRAM jel (dekódolt memóriacím a lapozó áramkörtől) 0 szintű lesz. Ugyancsak 0 szintű az NMREQ jel is, az 1. kapukimenet pedig a fentebb

<!-- pdf 54 -->

említett okok miatt 0. A 2. NOR kapu kimenetén (STB jellel jelöltük) megjelenő logikai 1 szint a következőkben a 6,25 MHz frekvenciának megfelelő 160 ns ütemidővel végighalad az 1., 2. és 3. flip-flop-on. Az 1. és 2. flip-flop-ok felfutó élre billennek, a J-K flip-flop lefutóra, de ez utóbbi az N6M25 jelet kapja.

Amikor a mindhárom flip-flop NQ kimenete 0 értékű lesz, akkor a 4 inverter is 0 értéket ad a 4. flip-flop J-K bemeneteire, mivel a 3. NOR kapu a három 0 értékből 1 értéket állít elő. Ez az állapot mindig a T3 ciklus elején következik be. A flip-flop az ilyen vezérlés esetén a korábbi állapotot tartja, azaz nem következik be az 1-0 átmenet. Ez az állapot oly módon szűnik meg, hogy az 1. kapuáramkör bemenetén az NDEB és NVCAS jel egyidejű 0 szintje áll elő, amely a CPU számára a videomemória használatra kijelölt idő második felében következik be.

Mivel a 4. inverter is 0 állapotú, az 1. kapukimenet 1, a 2. kapukimenet 0 állapotba kerül. A 6M25 jel következő felfutó éle azt az 1. flip-flop-ba írja. A flip-flop invertált kimenetén megjelenő 1 érték a 3. kapuáramkörön és a 4. inverteren át a 4. flip-flop következő órajelére való billenését váltja ki.

Amikor a videomemóriát programtárolásra használják és utasításlehívás történik, akkor az NM1 jel a 3. flip-flop-ot a törlőbemenetén át 0 értékbe kényszeríti, így a processzor órajel lemerevedéséhez egy fél óraütemmel rövidebb idő kell, a 4. inverter a 4. flip-flop-ot 0 értéken tartja, egyébként a folyamat a memória írás-olvasásnál elmondottakhoz hasonló.

A 18. ábrán az órajelet kialakító áramkör látható. A működés megértéséhez a 21. és a 22. ábrán látható idődiagramok nyújtanak segítséget. A két ábra a 20. ábrán (1) és (2) jellel jelölt helyzetet mutatja.

### 2.2.5 A MEGJELENÍTŐ LOGIKA

A megjelenítő egységek azokat az áramköröket tartalmazzák, amelyek <!-- pdf 55 --> lehetővé teszik, hogy a TV-Computer kép- (video-) memóriájában eltárolt képpontok színes televízión vagy nagyfelbontású megjelenítő eszközön: ún. színes display monitoron megjelenhessenek.

#### A megjelenítő logika blokkvázlata

A megjelenítő egység blokkvázlata a 23. ábrán látható.

A blokkvázlat alapvető egységei:

1. a videomemória
2. a videomemória-cím multiplexere
3. a display vezérlő és áramkörei
4. a párhuzamos/soros átalakító
5. a színkódoló és színregiszterek
6. a PAL kódoló
7. az UHF tv-modulátor

Mielőtt az egyes blokkokról bővebben beszélnénk, kicsit részletesebben tekintsük át a megjelenítés főbb kérdéseit!

Azt valamennyien tudjuk, hogy az otthoni tv-készülékeken látható kép nem statikus jelenség, hanem időben gyorsan változó folyamat eredménye. Másodpercenként 25 páros és 25 páratlan sorból álló ún. félkép jelenik meg szemünk előtt, azaz 50 teljes kép. Ennek a szemünk tehetetlensége miatt nem villogóképnek a frekvenciája tehát 50 Hz.

A tv-kép sorokból (tv-raszter) épül fel, amelyet egy kétirányú (képirányú, sorirányú) eltérítő rendszer elektronsugár eltérítésével rajzol a luminescens bevonatú képernyőfelületre. A kép-, ill. sorirányú eltérítés két szakaszból áll: egy lineáris (egyenletes sebességű) lassú előrefutási és egy gyors visszafutási szakaszból. A kép felrajzolása az egyes sorok előrefutása alatt történik. Az előrefutás végét egy ún. sorvégjel jelzi, amely a következő sorelőrefutás azonos fázisú indulását is biztosítja.

<!-- pdf 56 -->

Hasonlóan a kép végén egy képvégjel jelzi a képvisszafutás kezdetét. Mivel ezek a jelek a sorok, ill. a képek megfelelő időzítését biztosítják, a nevük sor-, ill. képszinkronjel. Az időfüggvény neve, amely az egyes képponti jelek amplitudójának időbeli értékeit adja: video-, azaz képjel.

A kép- és sorszinkronjeleket összegezve az ún. kompozit, azaz összetett szinkronjelet, ezt pedig a videojelhez adva a kompozit videojelet kapjuk.

A képi információt a sorelőrefutás alatt átvitt videojel szolgáltatja, amelynek amplitudója az egyes képelemek világosságát határozza meg (világosságjel, jele Y).

Mindezeket a jeleket a TV-Computernél is megtaláljuk, hiszen a megjelenítés itt is normál, ill. színes tv-n vagy monitoron történik, ezért a videojelre vonatkozó paraméterek csaknem azonosak.

A TV-Computernél alapállapotban nem páratlan/páros, azaz nem váltottsoros letapogatást használnak. Az ilyen üzemmódú megjelenítést non interlace üzemmódnak is nevezik.

__Megjegyzés:__ a TV-Computer kompozit videojele a nem váltottsoros (non interlace) letapogatás miatt nem tartalmazza a páratlan/páros félkép azonosításhoz szükséges kiegyenlítő jeleket és szinkronjel "befűrészeléseket" sem.

Hasznos soroknak nevezzük a képfelületen grafikai vagy karakteres megjelenítés céljára használt területet, amelyet az egyszerűség kedvéért papírnak nevezünk (PAPER).

A képelőrefutás alatt átviszünk még néhány tv-sort, ezek azonban már nem a hasznos, hanem az ún. képhatár sorait adják (BORDER) és grafikai vagy karakteres információt nem tartalmaznak. Ezen a képfelületen mindössze színbeállítás valósítható meg.

<!-- pdf 57 -->

A TV-Computer színes videojelet szolgáltat. Ezért - néhány fogalom erejéig - beszélnünk kell a színes megjelenítésről is.

A gyakorlatban előforduló színek többsége előállítható a vörös (RED, jele: R), a zöld (GREEN, jele: G) és a kék (BLUE, jele: B) színek keverékéből. Az R-G-B színhármast alapszíneknek nevezzük.

A színek azonosítása legegyszerűbben a Nemzetközi Világításügyi Bizottság (CIE) színháromszögében (SZÍNPATKÓ) valósítható meg, ahol minden színt egy derékszögű koordináta-rendszerben egy x és egy y koordinátával adhatunk meg.

Az R-G-B színháromszöget (24. ábra) határoló görbe az ún. színpatkógörbe minden egyes pontja egy 100 % színtelítettségű, egyszínű (monokromatikus), ún. spektrálszínt tartalmaz a patkó alakú szakaszon. A patkó két végpontját összekötő egyenes mentén az ún. bíborvonalon két spektrálszín keverékéből adódó bíborszínek helyezkednek el.

A színpatkó görbéje, ill. egyenese a reális színeket határolja. Az R-G-B színhármas R, G, B összetevők intenzitását megváltoztatva a reális színek nagyrésze kikeverhető.

Ennek alapján a mai tv-gyakorlatban a három alapszín a színpatkóban a következő x és y koordinátákkal jellemezhető:

| | | | |
|---|---|---|---|
| VÖRÖS | (R) | x=0,645 | y=0,335 |
| ZÖLD | (G) | x=0,29 | y=0,6 |
| KÉK | (B) | x=0,15 | y=0,065 |

A fehér szín például a színpatkóban (referencia ún. D fehér)

| | | | |
|---|---|---|---|
| FEHÉR | (W) | x=0,313 | y=0,329 |

koordinátákkal adható meg.

Mivel a fekete-fehér rendszerrel való kompatibilitás szükségszerű <!-- pdf 58 --> követelmény, ezért definiáltak egy ún. világosságjelet (Y), amelyet a három alapszínjel az R-G-B lineáris összefüggésével lehet megadni:

Y=0,299*R+0,587*G+0,114*B

amelyből teljes fehér képfelületet alapul véve, amikor az R=G=B=1 adódik: Y=1. Ez a fehér fekete-fehér és színes képjel esetére is azonos, amely a kompatibilitás egyik fontos feltétele.

A TV-Computer is ennek megfelelően képzett R-G-B jelet állít elő a színes display monitorok számára (a kompozit szinkronjellel és hangcsatornával együtt).

Abban az esetben, hogyha rendelkezünk ún. R-G-B bemenettel is kiépített színes televízióval, úgy azon igen jól élvezhető színes képvisszaadás valósítható meg, amely mindenképpen jobb minőségű, mint a kompozit videobemeneten vagy az antennabemeneten keresztül átvitt képi információ.

Tekintettel arra, hogy a kereskedelmi forgalomban lévő színes tv-k általában nem rendelkeznek ilyen lehetőséggel, gyakran nincs beépített R-G-B bemeneti egységük, ezért a TV-Computer még további két video-(kép-) kimenettel rendelkezik (tv-csatoló áramkörök).

Az egyik ezek közül alapsávi PAL kódolású kompozit videojelet szolgáltat PAL vagy PAL/SECAM rendszerű színes képmagnók és videobemenettel is rendelkező színes vagy fekete-fehér televíziók számára.

Megjegyzendő, hogy ez a videojel a tv nagyfrekvenciás bemeneti fokozatait kikerülve jut a készülékbe, az így kapott kép és hang zajmentesebb és minőségileg lényegesen jobb, mint az antennabemeneten keresztül továbbított kép/hang.

A másik kimenet AM modulált, nagyfrekvenciás, az UHF sávba transzponált PAL rendszerű kompozit videot és FM modulált hanginformációt szolgáltat kommersz PAL, vagy PAL/SECAM normás színes és fekete-fehér rendszerű televíziók számára. A színes tv rendszerekben nem kerül <!-- pdf 59 --> átvitelre mindhárom alapszínjel. Abban az esetben, hogyha előállítjuk az alapszínjelek és a világosságjel különbségeit, a következőket kapjuk:

R-Y azaz vörös színkülönbségi jel

G-Y azaz zöld színkülönbségi jel

B-Y azaz kék színkülönbségi jel

Ezek a színkülönbségi jelek ugyanúgy jellemezhetik a színes képet, mint az alapszínjelek.

Elegendő átvinni két színkülönbségi jelet és az Y, azaz világosságjelet, hogy a vételi oldalon a színes tv-ben egyszerű összegzéssel egy mátrix áramkör segítségével az alapszínjeleket előállítsuk.

Felírható, hogy:

Y+(R-Y)=R  Y+(G-Y)=G  és  Y+(B-Y)=B

a színes tv-adásnál az Y, az R-Y és a B-Y jeleket továbbítják, mert az előbbiek, valamint az Y=0,3*R+0,59*G+0,11*B egyenletekből következően a vételi oldalon ezekből viszonylag egyszerű módon a színkülönbségi mátrix áramkör segítségével:

G-Y=-(0,3/0,59)*(R-Y)-(0,11/0,59)*(B-Y)

Y+(G-Y)=G

Y+(R-Y)=R

Y+(B-Y)=B-t kapunk, azaz előállítottuk a három alapszínjelet.

A színkülönbségi jelek alkalmazását több szempont is indokolja, így a szükséges kis sávszélesség az átvitel során, valamint az, hogy feketefehér képrészletek esetén a színkülönbségi jelek eltűnnek:

R-Y=B-Y=G-Y=0, mivel ekkor R=Y, G=Y, B=Y

<!-- pdf 60 -->

A színkülönbségi jeleket csökkentett sávszélességgel viszik át, amplitúdóban pedig a túlmoduláció elkerülése érdekében súlyozva, a következő módon:

A súlyozott kék színkülönbségi jel: U=0,493*(B-Y)
A súlyozott vörös színkülönbségi jel: V=0,877*(R-Y)

Mivel a két színkülönbségi jelet PAL rendszernél egy ún. színsegédvivő kvadratúra modulációjával viszik át, ennek a színsegédvivőnek a zavaró (interferencia) hatása a fekete-fehér képfelületeknél megszűnik.

PAL rendszer esetén (a TV-Computer is ezt a rendszert használja) a kvadratúra moduláció során a V kvadratúra összetevő fázisát televíziós soronként 180 fokkal átkapcsolják, hogy az eredő fázishiba minimális legyen, azaz ne legyen számottevő színtorzulás (l. 25. ábra). Így a fázishiba nem színtorzulásban nyilvánul meg, hanem csupán az eredő vektor hosszának ingadozásában, ami viszont csak alig észrevehető színtelítettség-ingadozást eredményez (PAL rendszer).

Ezt könnyebb megérteni, hogyha a színeket olyan koordináta-rendszerben ábrázoljuk, amelynek vízszintes tengelyére a B-Y, a függőleges tengelyére a R-Y kerül, azaz a színeket a színkülönbségi jelek koordináta-rendszerében adjuk meg.

Ebben az esetben az egyes színek egy olyan vektorral jellemezhetők, amelynek hossza a szín telítettségét, a fázisszöge pedig a színét adja meg (l. 26. ábra).

A tv-technikában többféle színes televíziórendszert dolgoztak ki, az USA-ban az NTSC-nek nevezett rendszert, Európában a PAL és SECAM rendszert stb.

A TV-Computer PAL rendszerű színes rendszert használ (ez egy olyan kiegészítő áramköri elemekkel ellátott NTSC rendszer), amely kiküszöböli az átvitel során létrejövő fázishibák jelentős részét, és – mint láttuk – meghatározza a színek minőségét.

<!-- pdf 61 -->

Tekintettel arra, hogy a Magyarországon kereskedelmi forgalomba kerülő színes tv-készülékek zöme kétnormás: PAL/SECAM rendszerű, ezért a fenti megoldás műszakilag is előnyösnek tekinthető.

E kis kitérő után tekintsük a megjelenítő rendszer blokkvázlatát!

Tételezzük fel, hogy GRAPHICS 16, azaz 16 színű üzemmódban dolgozunk a TV-Computeren, vagyis a képernyőn a minimális felbontásban láthatjuk a PAPER területen megjelenő grafikus vagy szöveges információt.

Ebben az esetben a blokkvázlaton 1-gyel jelölt display memóriában a látott kép memóriabeli megfelelője úgy alakul, hogy minden egyes grafikus pontnak a memóriában 4 bit információ felel meg:

I - G - R - B

azaz

|||
|-|-|
| intenzitás | I |
| zöld| G |
| vörös| R |
| kék| B |
|||

<!-- pdf 62 -->

E négy bit segítségével  $2^4$ , azaz 16 szín határozható meg digitálisan a következő módon:

| I | G | R | B | SZÍN                |
|---|---|---|---|---------------------|
| 0 | 0 | 0 | 0 | (sötét) fekete      |
| 0 | 0 | 0 | 1 | sötétkék            |
| 0 | 0 | 1 | 0 | sötétvörös          |
| 0 | 0 | 1 | 1 | sötétlila           |
| 0 | 1 | 0 | 0 | sötétzöld           |
| 0 | 1 | 0 | 1 | sötétcián           |
| 0 | 1 | 1 | 0 | sötétsárga          |
| 0 | 1 | 1 | 1 | sötétfehér (szürke) |
| 1 | 0 | 0 | 0 | (világos) fekete    |
| 1 | 0 | 0 | 1 | világoskék          |
| 1 | 0 | 1 | 0 | világosvörös        |
| 1 | 0 | 1 | 1 | világoslila         |
| 1 | 1 | 0 | 0 | világoszöld         |
| 1 | 1 | 0 | 1 | világoscián         |
| 1 | 1 | 1 | 0 | világossárga        |
| 1 | 1 | 1 | 1 | világosfehér        |

A megjelenítő memória tehát minden byte-on két komplett grafikai pont összes jellemzőjét képes tárolni az alábbiaknak megfelelő bitelrendezésben:

| 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|---|---|---|---|---|---|---|---|
| I | G | R | B | I | G | R | B |

Tekintettel arra, hogy ennél a felbontásnál vízszintesen 128 pont adható meg, ehhez 128*4 bit, azaz 64 memóriabyte szükséges. Függőleges irányban 240 pont adható meg, így összesen ehhez 64\*240, azaz 15360 byte, vagyis 15360/1024=15 kbyte szükséges. A gyakorlatban meg<!-- pdf 63 -->valósított memória 16 kbites (ill. 64 kbites, TV-Computer +) dinamikus memóriachipekből áll (8 db). Így a 16 kbyte-os memóriakapacitás utolsó 1 kbyte-ja megjelenítés szempontjából kihasználatlan.

Más vízszintes felbontás esetén is a videomemória kapacitása ugyanekkora, ezért 256 pont/sor (GRAPHICS 4) vízszintes felbontás során egy grafikai pontelemet már csak két bit jellemez a memóriában. Ilyenkor a színeket nem közvetlenül a videomemóriából kapott bitek határozzák meg mint a 16 színű üzemmód esetén, hanem egy pontelemhez tartozó 2 memóriabit által megcímzett 4 darab négybites regiszterből nyerjük az adott pont I-G-R-B bitjeit.

Ebben az esetben tehát a videomemória egy byte-ja 4 db kétbites pontot határoz meg:

| 7  | 6  | 5  | 4  | 3  | 2  | 1  | 0  |
|----|----|----|----|----|----|----|----|
| d1 | d0 | c1 | c0 | b1 | b0 | a1 | a0 |

ahol a, b, c, d jelenti a tárolt képelemek bitjeit, így pl. az a pont bitjei a következőek:

<table>
<tr><th colspan="2">Képponti bitek a videomemóriából<br>(palettaregiszter-cím)</th><th colspan="4">Palettaregiszterek tartalma<br>(a tárolt színbitekkel)</th></tr>
<tr><td>a1</td><td>a0</td><td>3</td><td>2</td><td>1</td><td>0</td></tr>
<tr><td>0</td><td>0</td><td>I0</td><td>G0</td><td>R0</td><td>B0</td></tr>
<tr><td>0</td><td>1</td><td>I1</td><td>G1</td><td>R1</td><td>B1</td></tr>
<tr><td>1</td><td>0</td><td>I2</td><td>G2</td><td>R2</td><td>B2</td></tr>
<tr><td>1</td><td>1</td><td>I3</td><td>G3</td><td>R3</td><td>B3</td></tr>
</table>

<!-- pdf 64 -->

Az előbbiek alapján látható, hogy 4-színű üzemmódban a videomemóriában egy képpont két bitje nem közvetlenül határozza meg a színt, hanem csupán kijelöli a négy, 4 bites palettaregiszter közül azt, amelyik a kívánt I-G-R-B-t tárolja.

Tekintettel arra, hogy a palettaregisztereket előzetes írás során a rendelkezésre álló 16 közül tetszés szerinti 4 szín I-G-R-B adataival feltölthetjük, ezért egy képelem két bitje négy színt határozhat meg.

Ebben az üzemmódban – az azonos memóriakapacitás miatt – az egy tv-sorban megjeleníthető elemi képpontok száma a 16-színűnek éppen a két-szerese: 256 pont/sor.

2-színű, azaz GRAPHICS 2-es üzemmódban a 4-színűhöz hasonlóan a memória byte-ok bitjei nem közvetlenül a képpontok színbitjeit adják, hanem a palettaregiszter címet.

Ebben az esetben azonban egy elemi képpont 4 bites jellemzőinek meghatározására (I, G, R, B) már csak egy bit áll rendelkezésre.

Ez az egy bit – 0, ill. 1 állapotának megfelelően – 2, 4 bites palettaregisztert szelektálhat, amelyeket előzőleg feltöltöttünk a kívánt I-G-R-B bitekkel.

Így már világos, hogy miért nevezzük ezt 2-színű üzemmódnak. Az egy videomemória byte-on ilyenkor tárolható 8 képpont jelenti a legnagyobb felbontást a TV-Computer esetében, ilyenkor ugyanis egy tv-sorban, amely változatlanul 64 byte videomemória információt jelent: 512 elemi képpont jelezhető ki. A felbontás a 16-színű üzemmódnak éppen a négyszerese, a 4-színű üzemmódnak pedig a kétszerese.

A függőleges felbontás mindhárom üzemmódban változatlanul – a 240 tv-sor/hasznos képnek megfelelően – 240 képpont. (Hasznos kép = papírfelület.)

<!-- pdf 65 -->

Tekintsünk át néhány ismeretet a színlátásról, a színérzékelésről és a színek alapvető tulajdonságairól!

Szemgolyónk belsejében a képlátást a retina teszi lehetővé, amelyet több mint 120 millió fényérzékeny receptor: pálcika, és csap borít. A pálcikák tulajdonképpen "színvakok", ezek az ideghártyának a szemtengelytől távolabb eső részein helyezkednek el és a rodopszinnek nevezett szerves vegyület fény hatására történő elbomlása során a látóidegben keletkezett ingerület által érzékelik a fényt. Az érzékelés maximuma az 510-520 nanométer hullámhosszúságú fénytartományba esik. A másik, színlátást is lehetővé tévő receptort jellemző alakjáról csapnak nevezik, a szemgolyó optikai tengelyében az ún. látógödörben helyezkedik el és mintegy 7-8 millió van belőle.

A látás fő irányában ezek az éleslátást biztosító elemek, érzékenységük maximuma 550-560 nanométer hullámhossz tartományba esik, azonban gyenge: 1-10 lux-nál gyengébb megvilágítása esetén már nem érzékelnek. Ilyenkor csupán a pálcikák "színvak" érzékelése adott. (Félhomályban színeket nem látunk.)

A csapocskáknak három fajtája van, ezek leginkább

1. zöld
2. vörös
3. kék

színű fényt érzékelnek.

Az optikai prizma által felbontott fehér fényből szivárvány színskálát láthatunk. A színek folyamatosan, ugrás nélkül változnak a sötétkéktől a zöld, sárga, narancs árnyalatain át a vörösig. Ezek az ún. spektrálszínek, amelyeknek önálló nevük is lehet, pl. cián, ultramarin, fűzöld stb.

A több mint 20000 színárnyalat érzékeléséhez mindössze három színt megkülönböztető receptorra (vörös, zöld, kék) van szükség. A színár<!-- pdf 66 -->nyalatok megkülönböztetéséhez e háromféle receptornak különböző arányú gerjesztése elegendő!

A fehér fény spektrumába nem tartozó, csupán két szín különböző arányú keverékéből kapott színárnyalatokat, mint pl. az ibolya, lila, püspöklila stb. bíborszíneknek nevezzük.

Ilyen bíborszíneket akkor látunk, amikor szemünket egyszerre különböző arányú vörös és kék színű fény éri.

A színlátásnak az előbbiekben tárgyalt jellegzetességei lehetővé teszik, hogy a három ún. alapszínből a spektrálszíneket, valamint a bíborszíneket is előállíthassuk.

Erre kétféle, egymástól eltérő lehetőség kínálkozik, ezeket összefoglaló szóval színkeverésnek nevezzük.

Az első színkeverési módszert additív (összegző) színkeverésnek, a másodikat substraktív (kivonó) színkeverésnek nevezzük. Az első módszert a színes tv és monitorrendszerek alkalmazzák, az utóbbit főleg a színes fényképezés és fotolitográfia: a színes nyomdatechnika alkalmazza.

Additív színkeverést alkalmazunk a TV-Computer színpalettájának előállítása során, amikor pl. az R-G-B alapszínjelekkel vezéreljük színes televíziónkat és a képcső 3 elektronnyalábja a vörös, a zöld és a kék fénypor pontokat a vezérlőjeleknek megfelelő arányban gerjeszti. A 3 igen közel lévő színes képpont szemünk korlátozott felbontóképessége miatt egyetlen színes additív módon kevert képpontként látható.

A leírtakat jól szimbolizálja a 27. ábra, amely a három alapszín additív keverését mutatja be. Az ábrán a három alapszín aránya olyan, hogy additív keverékük fehér színt eredményez.

Tekintsük a blokkvázlatot: 23. ábra.

<!-- pdf 67 -->

A 3-as display vezérlő áramkör feladata, hogy ciklikusan megcímezze a videomemóriát, hogy annak párhuzamos 8 bites kimenetén időben rendelkezésre álljon a megjelenítendő képponti információ (16-színű üzemmód), ill. az azt címző bitelrendezés (4- és 2-színű üzemmód).

L. még a 28. ábra 3-as egységét, ill. a 29. ábra 1-es egységét.

Tekintettel arra, hogy a megjelenítés időfüggvénynek tekinthető, ezért a videomemória címzésének nagy pontossággal és szabályosan ismétlődve kell megtörténnie a karakterórajelnek megfelelően.

A memória kimenetén megjelenő byte azonban különböző jellegű információt hordoz a kiválasztott felbontástól függően, hiszen – mint ahogyan ezt az előbbiekben már megbeszéltük – 2-színű és 4-színű üzemmódban a byte bitjei palettaregiszter címek, míg 16-színű üzemmódban maguk az I-G-R-B bitek. Másrészt, 2-színű üzemmódban a memóriabyte 8 db képpontot jelent, 4-színű üzemmódban már csak 4 db-ot, 16-színű üzemmódban pedig csupán kettőt!

A blokkvázlaton 4-gyel jelölt párhuzamos/soros átalakítónak - amelynek feladata a videojel időfüggvényének az előállítása - az alkalmazott felbontásnak megfelelően különböző léptetési sebességgel kell kiléptetni a memóriabyte adatbitjeit.

L. még a 29. ábra 6-os egységét.

Ha nem így történne, akkor a hasznos (PAPER) képterületen az egyes sorok hossza nem lenne azonos a három különböző felbontásban. Ezért, a párhuzamos/soros átalakító 2-, 4- és 16-színű üzemmódban eltérő frekvenciájú léptető órajelet kap:

|||
|-|-|
|2-színű üzemmód: | 12,5 MHz|
|4-színű üzemmód: | 6,25 MHz|
|16-színű üzemmód: | 3,125 MHz|
|||

Ezt az órajelet raszterpont órajelnek nevezzük (RPCLK).

<!-- pdf 68 -->

16-színű üzemmódban a párhuzamos/soros átalakító párhuzamos kimenetein négy lépésenként megjelenő 4 bites információ maga a megjelenítendő I-G-R-B, ezért ez a blokkvázlat 5-ös színkódoló és színregiszter áramkörén áthaladva közvetlenül a színes videojelet szolgáltatja.

4-színű és 2-színű üzemmódban a párhuzamos/soros átalakító 4 bites párhuzamos kimenete az aktuális képpont(ok)-hoz tartozó palettaregiszter címét tartalmazza, ezért ezek a bitek a blokkvázlat 5-ös színkódoló és színregiszter áramkörénél a palettaregisztert címzik és annak 4 bites párhuzamos kimenetei adják az aktuális képpont I-G-R-B jellemzőit. L. 30. ábra 1-es és 3-as egységét.

Maga a palettaregiszter 4 db 4 bites, a CPU által írható, a megjelenítő logika által pedig az üzemmódtól függően kiolvasott regiszter, amelyet csak 2- és 4-színű üzemmódban címez a párhuzamos/soros shift-regiszterből kilépő képponti információ (a kiolvasás során).

A színkódoló és színregiszter áramkör a fentieken kívül egy 4 bites - a CPU által csak írható, a megjelenítő logika által a képkeret (BORDER) időtartama alatt olvasott - regisztert is tartalmaz, amelyet border-színregiszternek nevezünk. Egy CPU írás művelet során ez feltölthető a 16 közül bármelyik I-G-R-B színkombinációval és ez lesz a képkeret színe. L. a 30. ábrán lévő blokkvázlat 2-es egységét.

A színkódoló és színregiszter áramkör kétféle R-G-B kimenettel rendelkezik. Az egyikre akkor van szükség, amikor a PAL kódolót látjuk el meghajtó jellel. Ehhez két TTL szintű R-G-B jelet használunk: nem intenzív (I=0) R-G-B, és intenzív (I=1) R-G-B. Ezek TTL szintű jelek, amelyek a blokkvázlat PAL kódoló áramkörének bemenő jelei. L. a 30. ábrán lévő blokkvázlat 8-as egységét ill. a 31. ábrán lévő blokkvázlat 1-es egységét.

A másik R-G-B jel, amelyet a színkódoló előállít nem TTL szintű jel, hanem a fentiekben tárgyalt kétféle R-G-B jelnek ellenállás hálózattal összegzett jele. Az így kapott R-G-B színhármas amplitudója függ az intenzitásbit értékétől.

<!-- pdf 69 -->

Ezt az R-G-B jelet a kompozit szinkronjellel együtt a TV-Computer hátlapi csatlakozójára (5 pólusú DIN) vezetjük további felhasználás (pl. színes R-G-B monitor) céljából.

Térjünk vissza néhány szó erejéig a videomemóriához. Ennek a kettős elérésű memóriának a CPU által írhatónak és olvashatónak, a megjelenítő logika által pedig olvashatónak kell lennie. A megjelenítő memória 16 kbites dinamikus RAM áramkörökből épül fel (TV-Computer 32k, a TV-Computer 64k és 64 kbites dinamikus RAM áramkörökből a TV-Computer + esetében) (8 db). Ezeket a memóriákat csak több lépésben lehet megcímezni, ezért az ún. sor (row) és oszlop (column) címet multiplexelve, azaz különböző időben kell a memória IC-k címző bemenetére juttatni egy címet érvényesítő mintavételező jellel (row address strobe, column address strobe) együtt.

Másrészt - a kettős hozzáférés miatt - biztosítani kell azt, hogy a CPU, valamint CRTC (display vezérlő egység) felől érkező sor-, ill. oszlopcímek a megfelelő időpontban kapcsolódhassanak a videomemória címző bemenetére.

A blokkvázlaton ezt a feladatot a videomemória-cím multiplexere jelzésű egység látja el, amely tehát a két irányból (CPU, CRTC) érkező sor/oszlop címek kapcsolását végzi el. L. a 29. ábrán lévő blokkvázlat 2-es, 3-as egységét.

A videomemóriát a display vezérlő (CRTC) címzi ciklikusan. Ez 640 ns-onként kiolvassa az aktuális, soron következő megjelenítendő memória-byte-ot és ezzel a dinamikus memória tartalmának ún. frissítő olvasását is elvégzi.

A TV-Computer 2-színű üzemmódban egy karaktersorban 64 karaktert jeleníthet meg. Minden karaktersor 10 tv-rasztersorból áll. Egy karakter a tv-sorban vízszintesen 8 raszterpontból áll. (Beleértve a karakterközi szünet üres képpontjait is.) Induljunk ki abból, hogy a fentiekben említett raszterpont-frekvencia 12,5 MHz, azaz 1/12,5=0,080 µs-onként <!-- pdf 70 --> kapunk egy raszterpontot. Ezt 8-cal szorozva 0,64 µs-ot a karakter ütemidőt kapjuk.

Ennek reciproka: CCLK=1/0,64=1,5625 MHz a karakter órafrekvencia. A TV-Computer 2-színű üzemmódjában ilyen ütemben jelenik meg a képernyőn az egymást követő karakterek 8 pontja.

__Megjegyzés:__ az egyes karakterek fix pontmátrixát a ROM memóriában elhelyezkedő karaktergenerátor szolgáltatja.

A karaktersorok 10 tv-soros felépítése teljesen software által meghatározott, valójában a display vezérlő (CRTC) 4 tv-sorból álló karaktersorokkal dolgozik (fiktív), és ezért hardware szempontból a hasznos képfelület (PAPER) 60 db 4 tv-soros fiktív karaktersorból áll (60\*4=240 tv-sor). L. az M 6845 CRTC inicializálásánál.

További izgalmas kérdés az, hogyan valósul meg a képterület hasznos (PAPER) és képhatár (BORDER) területre osztása. Már a korábbiakban láttuk, hogy a kép felrajzolása soronként történik a sorelőrefutási idő alatt. A sorelőrefutási időben BORDER-PAPER-BORDER tartományokat különböztetünk meg, amelyeket a sorvisszafutás követ.

Helyettesítsük az egyes tartományokat az eltelt idővel arányos vonalszakaszokkal n-nel jelölve az aktuális tv-sort:

![Border-paper szakaszok a tv-sorokban](p070-fig1.png)
<!-- pdf 71 -->

Látható, hogy a border a paper szakasz végén kezdődik és átnyúlik a sorvisszafutási szakaszon át a következő sorelőrefutás elejére is.

Így a sorok border vége szakasszal kezdődnek, a hasznos paper szakasszal folytatódnak, amelyet ismét border (ti. border kezdete) szakasz követ. A sor végén a sorvisszafutási szakasz kezdődik, amely alatt folytatódik a border is bár ez utóbbiból a sorvisszafutás alatt semmi sem látszik.

Mindezt azért beszéltük meg, hogy könyebben megérthessük a display vezérlő (CRTC) blokkvázlatán látható (28. ábra) áramköri elemek működését. A blokkvázlaton a border kialakításában a következő egységeknek van szerepe:

- A 3-as CRTC, amelynek display engedélyező (DE) jele a kép- és sor-előrefutás alatt magas aktív, a kép- és sorvisszafutás alatt alacsony.
- A 2-es border flip-flop, amelynek WSE (write select enable) jele vezérli azt, hogy a border I-G-R-B jele jusson tovább, vagy pedig a hasznos paper szakasz aktuális I-G-R-B jele érvényesüljön - l. még a 30. ábra 6-os demultiplexer és regiszter áramköre.
- A 6-os monostabil multivibrátor egység, amelynek késleltetése a tv-sor végén (sorvisszafutás kezdete) 25,26 µs ideig tiltja az összes videojel megjelenését a sorvisszafutás alatt. A tiltójel alacsony aktív NVRCL (Video Register Clear) jel, amely a 8-as egység kimenetén jelenik meg, és a 30. ábra 8-as video regiszterének törlését (nincs video) végzi el.

Természetesen nemcsak sor irányban kell a képhatárt kialakítani, hanem függőlegesen is, hiszen itt is szükséges a border-paper-border tagozódás.

A border itt a hasznos kép végén kezdődik (240. tv-sor vége), és aktív marad a képvisszafutásig. sőt a képvisszafutás kezdete után a teljes <!-- pdf 72 --> képvisszafutás tartama alatt, valamint a következő képelőrefutás elején is (border vége szakasz).

A funkció megvalósítása során két feladatot kell megoldani:

- a megfelelő pillanatban (a hasznos kép végén) a border színregiszter tartalmát kell a kimenet felé engedélyezni és ezt a border egész tartama alatt fenn kell tartani
- a képvisszafutás alatt (és természetesen a sorvisszafutás alatt is) le kell tiltani minden videojelet a kimenet felé.

Az első feladatot a 28. ábrán lévő blokkvázlaton látható 2-es D flipflop valósítja meg, amelyet a CRTC DE, azaz display engedélyező jele vezérel. Ez a jel csupán a hasznos képfelület előrefutása alatt magas aktív, egyébként alacsony. A D flip-flop WSE, azaz íráskiválasztás engedélyezés jele a 30. ábrán lévő blokkvázlat 6-os multiplexerén a border színregiszter I-G-R-B tartalmát engedi tovább – biztosítva ezzel a border színét.

A második feladatot – a videojel letiltását a képvisszafutás alatt – a 28. ábrán lévő blokkvázlat 3-as, 4-es, 8-as, és a 30. ábrán lévő blokkvázlat 8-as áramköre teljesíti.

A videojel letiltását a CRTC által a 268. tv-raszter idején (a 78. 4 tv-soros karaktersor) kiküldött VS, azaz képszinkronjel kezdeményezi azáltal, hogy a 28. ábrán lévő blokkvázlat 8-as R-S flip-flop-ját úgy billenti át, hogy a 8-as kimenetén a NVRCL jel alacsony: aktív lesz, és ez a jel a 30. ábrán lévő blokkvázlat 8-as videoregiszterét törli (nincs videojel).

A videojel újbóli engedélyezését a CRTC egy alkalmasan kiválasztott címvezetéke biztosítja (MA9), amelyik a 28. ábrán lévő blokkvázlat 8-as R-S flip-flop-ját ellenkező állapotába billenti (az NVRCL jel magas: aktív lesz).

<!-- pdf 73 -->

A 28. ábrán lévő blokkvázlat egyéb áramkörei:

- az 1-es IT kérő áramkör, amely a CRTC CD jelének, a hang előállításáért felelős áramkör IT-jének, valamint a külső egységek IT-jének hatására NINT alacsony aktív IT kérést állít elő a mikroprocesszor számára
- a 7-es szinkronjel összegző áramkör, amely a 4-es és 5-ös a kép- és sorszinkronjel szélességét biztosító - monostabilok kimenőjelét adja össze és ezzel biztosítja a PAL kódoló, és az R-G-B kimenet számára a kompozit szinkronjelet (NCSYNC).

### 2.2.6 TV-CSATOLÓ ÁRAMKÖRÖK

#### PAL kódoló áramkör

A blokkvázlat PAL kódoló áramköre a korábban már tárgyalt funkciókat valósítja meg. Három alapvető részből áll, amelyeket később még részletesebben is megbeszélünk:

- a színkülönbségi és világosság jelet (R-Y, B-Y, Y) előállító áramkör l. a 31. ábrán lévő blokkvázlat 1-es egység
- PLL színsegédvivő oszcillátor (4,4336 MHz) és kvadratúra modulátor l. a 31. ábrán lévő blokkvázlat 7-es, 5-ös, 6-os egységeit
- kompozit PAL videojelet előállító áramkör. L. a 31. ábrán lévő blokkvázlat 8-as és 9-es egységét.

A PAL kódoló kimenőjele tehát színes, kompozit videojel, amelyet középső érintkezőt is tartalmazó 5 pólusú DIN csatlakozóra vezetünk a hangfrekvenciás jellel (SOUND), valamint egy +12 V-os ún. videokapcsoló jellel együtt.

Ez utóbbinak az a feladata, hogy a hozzákapcsolt színes televízióban a <!-- pdf 74 --> nem használt fokozatot kiiktassa, a TV-Computer videojelét pedig a video fokozathoz kapcsolja.

#### UHF modulátor áramkör

A PAL kódoló kompozit videojelét a blokkvázlatban láthatóan egy ún. UHF modulátorra is rávezetjük. Ennek az a feladata, hogy PAL kódolású kompozit videót a tv UHF 36-os csatornájára keverje fel (amplitúdó moduláció), a hangjellel együtt (frekvencia moduláció).

Az UHF modulátor kimenete 75 Ohm-os impedanciájú, aszimmetrikus koax csatlakoztatású. Innen a TV-Computer hátlapi csatlakozójáról 75 Ohm-os koax kábellel vezethetjük a színes tv antennabemenetére az UHF sávba felkevert színes videojelet és hangot. L. a 23. ábrán lévő blokkvázlat 7-es, egysége.

<!-- pdf 75 -->

<!-- pdf 76 -->

# 3. A TV-COMPUTER ALAPEGYSÉG ÁRAMKÖREI

Az áramkörök nyomtatott áramköri (NYÁK) lemezre épülnek. A lemezeket a rajtuk levő áramkörökkel kártyáknak nevezzük. A TVC alapegység négy kártyát tartalmaz:

- HBA (32k), HBA-1 (64k) és HBA-2 (64k+) kártyák valamelyikét a változattól függően. E kártyák tartalmazzák az alapegység áramköreit a tv-csatoló és a billentyűzet kivételével.
- HBP kártyát a PAL kóderrel.
- HBC kártyát a csatorna adapterrel.
- HOBBY kártyát a billentyűzet mechanikus elemeivel.

A kártyák áramköri felépítését a mellékelt kapcsolási rajzok mutatják (a HOBBY kártyához kapcsolási rajz nem tartozik, mert csak mechanikus elemek épülnek rá).

A kapcsolási rajzok használatához a következőket kell tudni:

Az áramköri elemeket szimbólumokkal jelöljük. A szimbólumok vagy valamilyen szokványos rajzjelek (pl. kapuáramkörök, flip-flop-ok, diszkrét elemek), vagy - a nagyobb integráltságú alkatrészeknél – kis téglalapok, amelyekben többek között az alkatrész típusára vonatkozó információ is van. Meg kell jegyezni, hogy ez a jelölés a leggyakoribb szállító típusjele és lehetséges, hogy a ténylegesen beépített elem nem ettől a szállítótól származik. Pl. a legtöbb integrált áramkör a rajzon a TEXAS INSTRUMENTS cég jelölését hordozza, a vele kompatibilis szovjet IC egészen eltérő típusszámmal van feliratozva.

A kapcsolási rajzon látható és a kártyán található alkatrész azonosítását a pozíciószám alapján végezhetjük el. Ez minden elemnél megta<!-- pdf 77 -->lálható a rajzon is, ill. a NYÁK-lemezen is. A leírásban erre a pozíciószámra hivatkozunk.

Felhívjuk a figyelmet arra, hogy a HBA és a HBA-1 kártyák ugyanarra a NYÁK-lemezre épülnek, a HBA-2 kártya NYÁK-lemeze viszont más. A három kártya kapcsolási rajza viszonylag kis eltéréstől eltekintve megegyezik. Az első két kártyán a közös NYÁK-lemezből kifolyólag az alkatrészek pozíciószámai is megegyeznek, a HBA-2 kártyán viszont eltérnek a pozíciószámok. A fentiek miatt két kapcsolási rajzot mellékelünk.

A leírás folyamán a mindegyik kártyán megtalálható áramkörök pozíciószámát a HBA-1 kártyára vonatkozóan adjuk meg – ez megegyezik a HBA kártya pozíciószámával – és zárójelben utánatesszük a HBA-2 kártya ugyanazon funkciójú alkatrészének pozícióját. Például: "A mikroprocesszor az E10 (F7) pozíciójú alkatrész". A mondat értelme: a HBA és HBA-1 kártyán az E10 pozícióban, a HBA-2 kártyán az F7 pozícióban kell keresni a mikroprocesszort.

A diszkrét elemek szimbólumain a bemeneti oldalon található nyíl 0-1 átmenetre való érzékenységet jelöl (órajelek). A nyíl kis körrel a negatív élre történő műveletvégzés jele. A kis kör önmagában azt jelenti, hogy a 0 állapot váltja ki a funkciót.

Az alkatrészek közötti kapcsolatokat biztosító főbb jeleknek külön elnevezésük van. A jelnév fölötti felülvonás azt jelzi, hogy a jel valamely másikból invertálással keletkezett vagy, hogy a jel aktív szintje a logikai 0. Ugyanezt a tényt a leírásban a jelnév elé helyezett N betűvel jelöljük.

Azoknak a kapcsolatoknak az azonosítása, amelyekhez nem rendeltünk jelnevet, a kibocsátó vagy fogadó alkatrész pozíciójának és kimenetének feltüntetésével történik. Pl. a G2/12 jel a G2 pozíciójú kapuáramkör 12 számú kivezetését, ill. a rajta lévő jelet azonosítja.

A HBA kártyák kapcsolási rajza több (11) lapból áll. A másik lapra vagy lapokra is kerülő jeleket szimbolizáló vonalak minden esetben a <!-- pdf 78 --> lap széléig vannak húzva. A lapról elmenő jeleket kifelé, a bejövőket befelé mutató nyíl jelzi (a buszjeleknél kétirányú a nyíl). A végződések mellett a jelnév található, továbbá zárójelben azoknak a lapoknak a sorszáma, ahol a vonal folytatódik.

A kapcsolási rajzon szereplő két LSI áramkör (Z80A CPU és 6845 CRTC) ismertetése külön fejezetet kapott. Az MSI áramkörökről szöveg közben adunk ismertetőt. A több funkcionális egységben is használt MSI IC-k ismertetését a szövegből kiemelten végezzük el azon a helyen, ahol az ismertetésre először szükség van, és a további fejezetekben csak hivatkozunk rá.

## 3.1 Központi egység

A HBA kártyák kapcsolási rajzainak 1. lapján található.

A Z80A mikroprocesszor az E10 (F7) pozíciójú tok (amelyről külön adatlapot közlünk a 6. fejezetben). A címbuszt a D2(D3) és a D1(D2) pozíciójú, SN74LS244 típusú buszmeghajtók erősítik. Az E9(F6) regiszterek a legmagasabb helyi értékű két címbit (A14, A15) értékét a memóriaolvasás műveletének idejére (NBMREQ=0) rögzítik. E két bit a memórialapozásban játszik szerepet.

#### SN74LS244:

Az SN74LS244 típusú buszmeghajtó az A jelű bemenetein lévő adatot az Y jelű kimeneteire engedi, ha az NG jelű bemenetei 0 állapotban vannak. Ha az NG bemenet 1 szintű, akkor az Y kimenetek nagyimpedanciás (TRISTATE) alapállapotúak.

A vezérlőjelek meghajtója F9(G6) pozíciójú, SN74LS244 típusú, az adatbuszmeghajtó C1(C2) pozíciójú, típusa: SN74LS245.

<!-- pdf 79 -->

#### SN74LS245:

Ez a tok kétirányú működésre képes. Ha az NENG jelű bemenete 1 szintű, az A és B kivezetések mindegyike nagyimpedanciás (TRISTATE) állapotú. A DIR bemenet 0 állapota a B kivezetéseket bemenetnek, az A kivezetéseket kimenetnek jelöli ki, 1-es állapota az A és B kivezetésekre fordított szerepekét oszt.

A C1(C2) adatbuszmeghajtó irányítottságát az F3/8(G2/12) és F6/1(J21/1) kapuk határozzák meg. Az F6/1(J21/1) kapun az adatbuszmeghajtók adatáramlását a CPU felé irányító logikai 1 állapot áll be olvasáskor, kivéve, ha a közvetlen adatbuszt használó eszközről történik az olvasás, F3/8(G2/12)=1.

A HBA-1 és HBA-2 kapcsolási rajzot együtt vizsgálva láthatjuk, hogy a 64k+ verziónál (HBA-2) az NCART jel nem akadályozza meg, hogy olvasáskor (NBRD=0) az adatbuszmeghajtón a jelek iránya a CPU felé irányuljon.

Mindhárom buszmeghajtó a buszkérés elfogadásakor (CPU NBUSAK kimenete = DMA jel = 0) nagyimpedanciás állapotba kerül. A 32k és 64k verziónál a DMA jel máshol nincs használva, a 64k+ verziónál a bővítő kártyahelyekre is ki van vezetve.

<!-- pdf 80 -->

## 3.2 A memóriák áramkörei

### 3.2.1 A LAPOZÓ ÁRAMKÖR

Az áramkörhöz tartozó alkatrészek többsége a kapcsolási rajzok 3. lapján található.

A lapdekódert az F8(E5) pozíciójú, SN74LS139 típusú IC egyik fele alkotja.

#### SN74LS139:

Az SN74LS139 IC két darab független 2-ből 4 formájú dekódert tartalmaz. Amennyiben az NG jelű bemenet 0 értékű, akkor az A és B bemeneten lévő bitkombináció által kijelölt Y jelű kimenet is 0 értékű lesz. A kimenet kijelölésénél az A bemenet az alacsonyabb helyi értékű. Az áramkör demultiplexerként is használható, ebben az esetben az NG bemeneten megjelenő jelet az A és B által kijelölt kimenetre irányítja.

A lapdekóderként szereplő F8(E5) IC engedélyező jele az NPHANTOM jel B1/4(J1/10) inverter által előállított negáltja. Ez azt jelenti, hogy ha a NPHANTOM=0, akkor a lapdekóder valamennyi kimenete 1 állapotú (inaktív). A dekóderkimenet kijelölését (a lap kiválasztást) a CPU A15 és A14 címvezetékei végzik.

A lapregisztert a C2(D1) pozíciójú SN74LS175 típusú IC és a H1/8(J3/8) flip-flop alkotja.

#### SN74LS175:

Az SN74LS175 IC négy darab D típusú flip-flop-ot tartalmaz közös órajellel (CP) és közös törlőbemenettel (CL). A CL bemenetén megjelenő 0 szintű impulzus hatására mind a négy Q kimenete 0 szintű lesz. A CP bemenetén megjelenő 0-1 átmenet hatására a D bemenetén <!-- pdf 81 --> lévő jelből mintát vesz és azt a Q kimenetére helyezi. A Q kimenet ezt az információt tárolja a következő felfutó órajelig. Az NQ kimeneteken természetesen a megfelelő Q kimenet jelének negáltja található.

Jelen esetben, lapregiszterként alkalmazva a D bemenetekre az erősített adatbusz BD4-BD7 vezetékei, a H1/8(J3/8) flip-flop D bemenetére a BD3 vezeték csatlakozik. Órajelként az NWR2 jel szerepel, a CL bemenetekre pedig az NRESET jel van kötve.

A lapdekóder 0. laphoz tartozó F8/4(E5/4) kimenete a G10(F2) demultiplexer engedélyező bemenetére jut (demultiplexer 0. lap). A demultiplexer A és B bemenete a H1/8(J3/8) és a C2/3(D1/3) regiszterkimenetek jelét kapja, amelyek a BD3, ill. BD4 adatbitek értékét tárolják (negálva!). A demultiplexer kimenetek, az adatbitek és a kiválasztott fizikai memóriák kapcsolata:

| BD3 | BD4 | Kimenet | Memória szelekciós jel |
|-----|-----|---------|------------------------|
| 0   | 0   | 1Y3     | NROM                   |
| 0   | 1   | 1Y2     | NP0                    |
| 1   | 0   | 1Y1     | NCART                  |
| 1   | 1   | 1Y0     | NP3                    |

A szelekciós jelek kapuáramkörökre kerülnek.

A lapdekóder 1. laphoz tartozó F8/5(E5/5) kimenete a 64k+ változatnál más eljárás alá kerül, mint a másik kettőnél. A 32k és 64k verziónál PG1 jelnéven egy, a 4 lapon lévő kapuáramkörre kerül, amely a demultiplexer 1. lap elnevezésű blokknak felel meg a 4. ábrán. Ezt a kapuáramkörökből felépített demultiplexert a J12/9 flip-flop vezérli, amely a lapregiszter többletbitjét tárolja. A flip-flop-ba ugyanazon az NWR2 jel, amely a lapregiszter másik két IC-jének az órajelét is adja, a BD2 adatbit állapotát írja.

<!-- pdf 82 -->

- BD2=0 esetén J18/8 kimenet (NP1) lesz 0 szintű
- BD2=1 esetén a J15/13 kapun át az NVRAM jel válthat 0 szintbe.

A lapdekóder 2. laphoz tartozó F8/6(E5/6) jele a két kapuból álló demultiplexer 2. lap elnevezésű egységre kerül. A 32k és 64k változatnál a két kapu, E8/3 és E8/11 a 3. lapon található, a 64k+ verziónál az egyik (F4/6) a 3., a másik (J15/4) a 4. lapon van.

A lapregiszter C2/6(D1/6) és C2/7(D1/7) kimenete határozza meg, hogy az NP2 vagy az NVRAM jel impulzusa alakuljon-e ki a 2. lap megcímzésekor. A regiszterkimenetek a BD5 adatbit állapotát tárolják.

- BD5=0 esetén E8/3(F4/6) kapu, NP2 aktív
- BD5=1 esetén E8/11(J15/4) kapu, NVRAM aktív

Az NVRAM jel kialakulásába az 1. laphoz tartozó demultiplexer is beleszól.

A lapdekóder 3. laphoz tartozó F8/7(E5/7) kimenete a G10(F2) demultiplexer engedélyező jelét adja. A demultiplexer A és B bemenetét a BD6 és BD7 adatbiteket tároló C2(D1) lapregiszter egységek vezérlik:

| BD7 | BD6 | Kimenet | Memória szelekciós jel |
|-----|-----|---------|------------------------|
| 0   | 0   | 2Y0     | NCART                  |
| 0   | 1   | 2Y1     | NROM                   |
| 1   | 0   | 2Y2     | NP3                    |
| 1   | 1   | 2Y3     | NEXP                   |

A szelekciós jelek még kapuáramkörökre kerülnek.

A programmodul NCART szelekciós jeléhez a G2/11(F3/8) ÉS kapu fogja össze a 0. és 3. lap demultiplexerének egy-egy jelét (ezeken a lapokon <!-- pdf 83 --> érhető el a modul). Az E8/6(F4/8) VAGY kapu az NMREQ jelet kapuzza be a szelekciós jelbe.

A rendszer ROM NROM jelét a G12/8(F3/3) ÉS kapu, és az E8/8(F4/3) VAGY kapu alakítja ki a 0. és 3. lap demultiplexerének jeleiből és az NMREQ jelből.

Az U3 felhasználói RAM szelekciós jelét (NP3) a G2/8(F3/11) ÉS kapu alakítja ki 0. és 3. lap demultiplexerének jeleiből. Az NMREQ jel ennél a memóriánál az NRAS és NCAS jeleken keresztül hat.

Az NVRAM jel a 32k és 64k változatnál a demultiplexer E8/11 kimenetén közvetlenül kialakul, a 64k+ változatnál a 4. lapon található NOR kapu (J15/1) alakítja ki az 1. és 2. demultiplexer jeléből.

Az NP0 jel a 0. lap demultiplexerének G10/6(F2/6) kimenetén közvetlenül kialakul. A 32k és 64k változatnál a D11/12 inverter a jel negáltját, a P0 jelet alakítja ki.

Az NP1 jel a 32k és 64k változatnál a lapdekóder F8/5 kimenete, a 64k+ változatnál az 1. laphoz tartozó demultiplexerkimeneten (J18/8), a kapcsolási rajz 4. lapján alakul ki.

Az NP2 jel a 2. laphoz tartozó E8/3(F4/6) demultiplexer kimenet jele.

Az NEXP jel a 3. lap G10/9(F2/9) demultiplexerkimenetének és az NMREQ jelnek az A4/1(F4/11) kapu által képzett VAGY kapcsolatából alakul ki.

Az NEXP jel további feldolgozásra kerül. A kapcsolási rajz 6. lapján található F1(G1) regiszter két bitje alkotja a bővítő lapregisztert. Az IC típusa: SN74LS174. Ez hasonló az e fejezetben ismertetett SN74LS175-höz, csak négy helyett hat D flip-flop-ot tartalmaz, és a flip-flop-ok negált kimenete nincs kivezetve.

<!-- pdf 84 -->

Az F1(G1) IC az NWR3 jelet kapja órajelként (a kapcsolási rajzon nincs külön feltüntetve), a BD6 és BD7 adatbiteken érkező információt tárolja és adja tovább EXPSEL0 és EXPSEL1 jelként.

A kapcsolási rajz 10. lapján található E6(J4) pozíciójú, SN74LS139 típusú IC alkotja a bővítő lapdemultiplexert. Az NEXP jel az N1G bemenetet vezérli. A demultiplexer 1A bemenete az A13 címvezetékre, az 1B bemenete a tápfeszültségre van kötve, ily módon csak az 1Y2 és 1Y3 kimenetei aktiválódhatnak, méghozzá A13=0 esetén 1Y2 és A13=1 esetén 1Y3. Az 1Y2 engedélyező jelként az N2G bemenetet vezérli, tehát A13=0 esetén a bővítő egységek memóriáinak valamelyike kapcsolódik a CPUhoz. Az 1Y3 kimenet a rendszer ROM kiterjesztés 8 kbyte-os memóriaterület szelekciós jelét (NROM5) állítja elő.

A bővítő egységek memóriái közül az EXPSEL0 és EXPSEL1 jelek választanak a következő módon:

| EXPSEL1 (BD7) | EXPSEL0 (BD6) | Szelekciós jel | Bővítő csatlakozó |
|---------------|---------------|----------------|-------------------|
| 0             | 0             | NEXP1          | CH1               |
| 0             | 1             | NEXP2          | CH2               |
| 1             | 0             | NEXP3          | CH3               |
| 1             | 1             | NEXP4          | CH4               |

### 3.2.2 FELHASZNÁLÓ RAM

A 6. ábra blokkvázlatán szereplő blokkok közül a RAM vezérlő, a cím multiplexer és a címmódosító a kapcsolási rajzok 1. lapján találhatók.

Az E12/5 (J12/5) flip-flop és a köré épített kapuáramkörök az NRAS és NCAS jelek előállítását végzik.

<!-- pdf 85 -->

Az NRAS jel, amelyet a 32k változatnál az alapegységben található mindkettő 16 kbyte-os felhasználói RAM egység (U0 és U1) megkap, az NBMREQ jellel egyezik meg. A G11/2 és G11/4 (J14/2 és J14/4) inverterek csak terhelés szempontjából választják el a memória IC bemeneteket az NBMREQ jeltől. Az R40 (R58) ellenállás a vezetéken esetleg fellépő lengéseket csillapítja. Ilyen ellenállás egyébként a memória összes bemeneti jelvezetékén megtalálható, kivéve az adatbemeneteken.

Az E12/5 (I12/5) flip-flop a BMREQ jelből a CPU órajelével vesz mintát, létrehozva az NRAS jel és a sor- és oszlopcímek közötti váltást biztosító MUX jel között a szükséges mértékű késleltetést. A flip-flop NQ kimenetén megjelenő jelet az R38 (R29) ellenállásból és a C4 (C8) kondenzátorból álló integráló áramkör tovább késlelteti. Az F12/6 (I17/1 kapu kimeneti jele ilyenformán a MUX jelhez képest is késik és felhasználható az NCAS jel kialakításához.

Az NCAS jel (ill. jelek) kialakításának további áramkörei eltérnek a HBA és HBA-1, ill. HBA-2 kártyán:

- A HBA kártyán az F12/6 kapu jelét az F12/3 kapu hozza kapcsolatba az NP1 jellel, így alakul ki az NCAS jel, amelyet a csak a 32k verziónál meglévő, az U1 jelű 16 kbyte-os egység használ.

Az NCAS64 jel azt a fizikai memóriát választja ki működésre, amely a 32k változatnál 16 kbit-es IC-kből, a 64k változatnál 64 kbit-es IC-kből áll. A TB7 jelzésű átkötési lehetőség a 32k változatnál az 1-es jelű oldalon van bekötve, így a D12/13 NOR kapu csak az U1 RAM megcímzésekor (P0=1) vált 0 értékre és engedi az F12/11 kapun át az NCAS64 jelen 0 szintű impulzus kialakulását.

A 64k változatnál az F11/12 kapu kimenete a D12/11 kapubemenettel van összekapcsolva. A hárombemenetű NAND kapu kimenete 1 szintű lesz, ha az NP1-NP3 jelek közül bármelyik 0 szintű (U1-U3 RAM egységek valamelyike van címezve). A D12/13 kimenet ily módon bármelyik 16 kbyte-os felhasználói RAM egység megcímzésekor az NCAS64 jel aktív impulzusának létrejöttét lehetővé teszi.

<!-- pdf 86 -->

- A 64k+ változatnál az J17/1 NOR kapun az NCAS jel kialakításához szükséges jel negáltja található (1 szintű impulzus alakul ki memóriaírás vagy -olvasás műveletnél). Az J2/8 NAND kapu kimenete 1 szintű, ha az NP0-NP3 jelek bármelyike 0 (felhasználói RAM címzés). A J2/8 és J17/1 kimenetek egyidejű 1 állapota a J12/6 NAND kapun kialakítja az NCAS64 jel 0 szintű impulzusát.

Az E12/1 (J12/1) flip-flop bemenetre (Cl=törlés) a CPU NRFSH jele van vezetve. Ez a jel a dinamikus memóriafelfrissítés folyamatát jelzi. E folyamatnál a CPU alacsonyabb helyi értékű hét címvezetékén egy inkrementálódó memóriacímet ad ki, amit sorcímként kell a memóriához juttatni oszlopcím nélkül. Ez azt jelenti, hogy az NRAS jelen ki kell alakítani az impulzust, de a MUX jelet nem kell átváltani és az NCAS jelnek is 1 szinten kell maradnia. Az NRFSH jel aktív (0) szintje a flip-flop törlőbemenetén keresztül a flip-flop-ot a sorcím átadására jellemző helyzetben tartja.

A CPU címvezetékeinek sor- és oszlopcímekre osztását a D8 és D9 (D9 és D10) multiplexerek végzik.

Az SN74157 típusú IC a SEL jelű bemenetének 0 állapota esetén az A jelű bemeneteinek jelét engedi az Y jelű kimeneteire, a SEL=1 feltétel esetén a B jelű bemenetekről érkező jel jut a kimenetekre. Az ST jelű bemenetekre adott 1 szinttel valamennyi kimenet 0 értékre kényszeríthető a többi bemenet állapotától függetlenül.

Az ST bemenetet jelen esetben fix 0 szinten tartjuk, a SEL bemenetet pedig az előzőekben ismertetett MUX jel vezérli.

A MUX jel a bővítő csatlakozókra is eljut, a memóriabővítő modul is használja.

A felhasználói RAM legmagasabb helyi értékű címvezetékének jele (DA7) a 32k verziónál nincs kihasználva. A 64k+ változatnál a jelet a J17/4 és J17/13 kapukból álló címmódosító befolyásolja.

<!-- pdf 87 -->

Az áramkör lehetővé teszi, hogy az U3 RAM a 0. lapon is elérhető legyen a CPU számára.

A DA7 jel – a DA0-6 jelekhez hasonlóan – két címbitet továbbít a memóriához. A két bit kombinációi jelölik ki az U0-U3 RAM területet, pl. a 00 kombináció az U3, az 11 kombináció az U0 területet (a J17/13 kapu invertálja a multiplexer kimenőjelét!).

A szokásos lapozás beállításakor (amikor az U0 RAM a 0. az U3 RAM a 3. lapon érhető el) a J17/4 kapukimenet 0 állapotú, a DA7 jelet a D9 multiplexeren át a CPU A14 és A15 jele határozza meg. A 0. lap megcímzésekor A15=A14=0, a DA7 jel sor- és oszlopcímként is 1 értékű, azaz az U0 RAM van kiválasztva. A 3. lap megcímzésekor A15=A14=1, a DA7 jel sor- és oszlopcímként is 0 értéket ad a memóriára, azaz az U3 RAM van kiválasztva.

Amikor az U3 RAM-ot a 0. lapon is el akarjuk érni, a lapregiszter 3-as bitjét 1 értékre kell állítani (l. 3.2.1 pont). A regiszter 3-as kimenő bitjének negáltja a PB3 jel, tehát jelen esetben 0 értékű. A 0. lapot címezve a lapdekóder 0. laphoz tartozó kimenőjele (NPG0) is 0 értékű lesz. A J17/4 kapukimenet emiatt 1 értékű, a J17/13 kimenet pedig a multiplexertől függetlenül 0 értékű lesz, ami az U3 RAM kijelölésével egyenértékű.

Mint már említettük, a címmódosító áramkör a 64k változatban nincs kiépítve, emiatt az U3 RAM fentiekben ismertetett átlapozása nem hajtható végre. Ez a tény problémát csak akkor jelent, ha a 64k változatú gépet VT-DOS operációs rendszerrel kívánjuk használni. A címmódosító áramkört ez esetben fel kell építeni. A megoldáshoz egy db SN74LS02 típusú IC-t többletként el kell helyezni valamelyik 14 lábú IC hátán oly módon, hogy a 7-es (felülnézetből bal alsó) és 14-es (jobb felső) lábakat az alatta lévő IC ugyanazon lábaihoz forrasztjuk.

A logikai jelek bekötésére vonatkozóan célszerű a HBA-2 kártya kapcsolási rajza szerint eljárni.

<!-- pdf 88 -->

A D9/12 kimenetén és az R34 ellenállás közötti fóliát el kell vágni. A D9/12 kivezetést kössük az újonnan beépített IC 12-es lábára, az IC 13-as lábát pedig a R34 ellenállás szabaddá tett végére.

Az új IC 4-es és 11-es kivezetését kössük össze. Az 5-ös lábára az F8/4, a 6-os lábára a H1/8 kivezetést kapcsoljuk.

Hangsúlyozzuk, hogy célszerűbb a problémával valamelyik VIDEOTON szervizt megkeresni.

A felhasználói RAM egyéb áramkörei a kapcsolási rajzok 4-es lapján találhatók.

A 32k változatnál az U0 RAM-ot az E01-E08 pozíciójú tokok, az U1 RAM-ot az F20-F27 pozíciójú tokok alkotják. Az E01-E08 IC-k 8-as kivezetése a +12 V-hoz, a 9-es kivezetése a +5 V-hoz kapcsolódik (hasonlóan a második sor elemeihez). Mindkét sor memória IC ugyanazt az NRAS jelet és ugyanazt az NWR jelet kapja (a CPU NWR jelét a G14/3 és G14/6 kapuk választják le terhelés szempontjából). A felső sor az NCAS64, az alsó az NCAS jelet használja.

Az IC-k DI bemenetére a közvetlen adatbusz jelei kerülnek. DO kimeneteik páronként össze vannak kötve. Ezt az teszi lehetővé, hogy a kimenet TRISTATE, és amelyik IC nem kap NCAS impulzust, az nagyimpedanciás állapotban van.

A 64k verziónál a felső sorba a 4116-33 típusú, 16k\*1 bites memóriachipek helyett 4164-3 típusú, 64k\*1 bites memóriachipek kerülnek. 8-as lábuk +5 V-ot kap, a 9-esre pedig a többlet címvezeték, a DA7 jut. Az alsó sor üres.

A memóriakimeneteket a C6 pozíciójú buszmeghajtó (SN74LS244, l. 3.1 alfejezet) kapcsolhatja az erősített adatbuszra, ha memóriaolvasás történik (RD=1) és az NCAS64 és NCAS jelek bármelyike aktív (0) szintű.

<!-- pdf 89 -->

A 64k+ változatnál csak egy sor (8 db) IC-ből áll a felhasználói RAM. Pozíciójuk F00-F07. A buszmeghajtó C6 pozíciójú.

### 3.2.3 A RENDSZER ROM ÁRAMKÖREI

Ezek az áramkörök a kapcsolási rajzok 3. lapján találhatók.

A HBA és HBA-1 kártyákon 5, a HBA-2 kártyán 3 ROM IC-nek van helye. A különbség abból adódik, hogy a régebbi változatoknál kisebb (4 kbyte) kapacitású ROM-ok felhasználására is fel kellett készülni, az újabbnál már csak 8, ill. 16 kbyte-os chipek jöhetnek szóba.

Az alkalmazható EPROM típusok: 2732 (4 kbyte), 2764 (8 kbyte) és 27128 (16 kbyte).

Valamennyi típusra vonatkozik, hogy az A0-A13 címbemeneteken lévő bit-kombináció által kijelölt 1 byte-os memóriarekeszük tartalmát a D0-D7 kimenetükre adják, ha az NCE (CHIP ENABLE) és NOE (OUTPUT ENABLE) bemenetük mindegyike 0 állapotú, egyébként a kimenetek nagyimpedanciás állapotban vannak.

A ROM-ok mindegyike a CPU NRD jelét kapja az NOE bemenetére, az éppen működő egyetlen ROM kiválasztása tehát a NCE bemeneteken keresztül történik.

A D7 (D7) pozíciójú integrált áramkör alkotja a rendszer ROM kibővítést, azért kapja engedélyező jelként az NROM5 jelet.

A 32k és 64k változatnál ebbe a pozícióba elhelyezhetünk 2732 (4 kbyte) és 2764 (8 kbyte) típusú tokokat is. A 2732 24, a 2764 28 lábú IC. A kapcsolási rajzon ezért a kivezetéseknél két-két számot tüntettünk fel és ahol eltérés van a funkcióban, ott a kivezetés megnevezésénél is. Az első szám vagy jel a 2732 IC-re vonatkozik. Pl.: A

<!-- pdf 90 -->

8-10 számú kivezetésre (A0) vonatkozóan a 2732 felhasználása esetén a 8-as, a 2764 felhasználása esetén a 10-es szám az érvényes.

A 24-26 számú Vcc-NC bemenetet úgy kell érteni, hogy a 2732 IC 24-es lába Vcc (tápfeszültség +5 V), a 2764 IC 26-os lába NC (közömbös, nincs bekötve belül). A ROM-ok számára a kártyákon foglalatok vannak elhelyezve, nyilvánvalóan a nagyobb méretű tokra számítva 28 lábúak. Ha 24 lábú IC-t helyezünk beléjük, akkor felül kell üresen hagyni a foglalatot (a 28 lábú IC foglalat 1-2-27-28 lába üres), így elérjük azt, hogy a kapcsolási rajzon a két eltérő kapacitású tokra feltüntetett lábszámok éppen megfelelően találkoznak (pl.: a 24 lábú tok 1-es lába a 28 lábú tok 3. lába helyére kerül és ez mindkét ROM-nál az A7 címbemenetet).

A 64k+ változatnál csak 8 kbyte-os ROM-ot használhatunk fel ezen a helyen.

Az alapegységben található többi ROM működését a lapozó áramkör NROM jelének 0 szintje engedélyezi. Az NROM jel 16 kbyte-nyi címterületet reprezentál. 4 kbyte-os ROM IC-ből 4 db, 8 kbyte-osból 2 db, 16 kbyte-osból 1 db vezérlésére lehet használni. Ha a 16 kbyte-nyi kapacitást több ROM IC tartalmazza, akkor közülük is választani kell. A 16 kbyte-on belüli két 8 kbyte-os memóriaterület között az A13 CPU címbit választ. A 16 kbyte-on belüli 4 kbyte-os memóriaterület szelekciójában az A13 és A12 címbitek szükségesek. A címbiteket egy dekóderre vezetjük, amelynek pozíciója: F8 (E5). A dekóder SN74LS139 típusú, a 3.2.1 pont tartalmazza rövid ismertetését. A dekóder N2G bemenetére engedélyező jelként az NROM jel kerül. A dekóderkimenetek szolgáltatják a ROM IC-k NCE bemenetének vezérlését.

A HBA és HBA-1 kártyán a dekóder szelekciós jeleinek vezérlését a TB2-TB3-TB4 átkötések szabják meg.

<!-- pdf 91 -->

- 4 kbyte-os memóriachipekből felépített ROM területhez a TB2 és TB3 átkötés szükséges, azaz a dekóder 2A bemenetét az A12, a 2B bemenetét az A13 címbit vezérli. A két címbit négy kombinációjában D3, D4, D5 és D6 pozíciójú 4 kbyte-os ROM IC engedélyező jele tartozik.
- 8 kbyte-os memóriachipek esetén a TB3 és TB4/2 átkötés szükséges. A dekóder 2A bemenete fix 1 szintű, a 2B bemenetet az A13 vezérli, ily módon csak a 2Y1 és a 2Y3 kimenet válthat 0 szintbe, a D3 és a D4 pozíciójú 8 kbyte-os IC-k közül választva.
- A 16 kbyte-os memóriachiphez a TB4/1 és TB4/2 átkötés tartozik, a dekóder mindkét bemenete fix 1, csak a 2Y3 kimenete válthat 0 szintbe, megegyezve az NROM jellel. A D3 pozíciójú tok lehet a 16 kbyteos ROM.

A D3 és D4 pozíciójú tokokra is vonatkoznak a D7 tokról a lábszámozás és lábjelölés vonatkozásában elmondottak, azzal kiegészítve, hogy a D3 tok háromféle típus lehet, így találhatók hármas jelölésű lábak is. Ez esetben az első a 2732, a második a 2764 és a harmadik a 27128 típusra vonatkozik.

A D3 toknál a ROM típuscsere az előzőekben ismertetett átkötéseken túl a TB8 átkötés módosítását is megkívánja:

- 4 kbyte-os memóriánál TB8/1 átkötés szükséges
- 8 kbyte-os memóriánál átkötés nem kell
- 16 kbyte-os memóriánál TB8/2 átkötés kell

A HBA-2 kártyára értelemszerűen vonatkoztathatók az eddig elmondottak.

Az E5 dekóder bemenete fixen 1 értékre van kötve, annak megfelelően, hogy 4 kbyte-os ROM-ok nem használhatók. A rendszer ROM 16 kbyte-os része vagy D4 és D6 pozíciójú 8 kbyte-os chipekből áll, ehhez a TB1/1 és TB2/1 átkötés tartozik, vagy a 04 <!-- unclear --> pozíciójú 16 kbyte-os PROM alkotja a rendszer ROM-ot, ehhez a TB1/2 és TB2/2 átkötés szükséges.

<!-- pdf 92 -->

A ROM-ok kivezetéseinek eddig nem említett jelölései:

- GND: föld
- Vcc: +5 V
- Vpp: tápfeszültség programozáshoz
- NPGM: programozó impulzus
- NC: nem használt kivezetés

### 3.2.4 A VIDEO RAM ÁRAMKÖREI

A kapcsolási rajzok 8. lapján találjuk meg őket. A 16k\*1 bites, ill. 64k\*1 bites memória IC-k a VR0-VR7 (VR0-VR7) pozíciót foglalják el.

A DA0-6 (DA0-7) bemenetek fogadják a sor- és oszlopcímeket, amelyeket az NRAS, ill. NCAS jelű kimenetre adott 0 szintű impulzus érvényesít. Az NW jelű bemenetre adott 0 szintű impulzus a DI bemenetre adott adatbitet tölti a cím által kijelölt rekeszbe. A TRISTATE DO kimeneten az NCAS jel impulzusa hatására van érvényes adat.

A 16 kbites IC-k három, a 64 kbitesek egy tápfeszültséget igényelnek.

A címbemenetekre a G16-E13-F13-G17 (D12-F12-F11-E12) multiplexerek kimenete kapcsolódik. A HBA és HBA-1 kártyán a címvezetékeket ellenállások kötik a +5 V-os tápfeszültséghez, megemelve a logikai 1 feszültségszintjét. Az RN4 ellenálláshálózat, a R58 és R59 diszkrét ellenállás.

A HBA-2 kártyán a tápfeszültségre kapcsolódó ellenállásokon (R50-R56) kívül soros ellenállások is találhatók a címvezetékeken, ez utóbbiak az esetleg fellépő feszültségingadozásokat (belengés) csökkentik.

A multiplexerek SN74LS153 típusúak. Ez az integrált áramkör az 1C0-1C3 bemenetek jele közül egyet-egyet az 1Y kimenetre, a 2C0-2C3 bemenetek jele közül egyet-egyet az 2Y kimenetre továbbít. A négy-négy adatbemenet közül való választást az A és B bemenetre adott bitkombináció <!-- pdf 93 --> végzi. Az ST1 és ST2 bemenetekre adott logikai 1 szinttel az 1Y, ill. 2Y kimenetet az egyéb bemenetek állapotától függetlenül 0 szintre lehet állítani.

A jelen esetben az ST1 és ST2 fixen 0 szintű. Az A bemeneteket a DEB, a B bemeneteket az NVMUX jel vezérli. Az adatbemenetekre a CPU és CRTC címvezetékei kapcsolódnak. A DEB jel választ a CPU és a CRTC (DEB=1 : CPU), az NVMUX a sor- és oszlopcímek (NVMUX=1 : oszlopcím) közül.

A memóriakimeneti regiszter C7 (C8) pozíciójú. Az SN74LS374 IC TRISTATE kimenetű 8 bites D tároló közös órajelbemenettel. A CLK bemeneten jelentkező 0-1 átmenet hatására a regiszterben tárolódik a D jelű bemenetek állapota. A QC bemeneten 0 szintnek kell lennie, hogy a Q kimeneteken érvényes adat legyen, különben azok nagyimpedanciás állapotban vannak.

A C7 (C8) regiszterbe az NVCAS jel írja a memória kimenő adatát. Meg kell jegyezni, hogy ez akkor is megtörténik, amikor a CRTC olvas a memóriából, csak nyilvánvalóan az eltárolt állapot nem jut a CPU-hoz. Az adatbuszra (BD0-7) akkor kerül a regiszter tartalma, ha a D11/4, D12/10 és DE1/11 (J14/6, J15/10 és J19/3) kapukból álló áramkör az NRD, NBMREQ és NVRAM jelek egyidejű 0 állapotának hatására a regiszter QC bementét 0 szintbe vezérelve ezt lehetővé teszi.

A HBA-2 kártyán még a videomemóriához tartozik egy áramkör, amely a 64 kbyte-os memória többletcímbitjét (VDA7) állítja elő. Ez az áramkör a kapcsolási rajz 4. lapján található.

A C1 pozíciójú, SN74LS174 típusú 6 bites D tárolóba a BD0-BD5 adatbitek értékét tölti a J19/8, J18/2 és J18/6 kapukból álló órajel-előállító áramkör, ha a CPU a 0CH I/O címre ír.

A C1 pozíciójú IC a video lapozó regiszter, a B1 pozíciójú tok pedig a video lapozó multiplexer. Az SN74LS151 típusú integrált áramkör a D0-D7 bemeneteire kapcsolt jelek valamelyikét továbbítja az Y kimenetére <!-- pdf 94 --> (a W kimenetére is, de oda invertálva). A bemenetek kiválasztása az A-B-C bemenetekre adott bitkombinációval történik. Az E jelű bemenetre adott 1 szinttel az Y kimenetet 0 szintbe lehet kényszeríteni.

A szelektorbemeneteket az NDEB, az NVMUX és az NPG1 jelek vezérlik a következőképpen:

| NDEB | NPG1 | NVMUX | Szelektált<br>adatbemenet | Szelektált címbit<br>adatbuszjel |
|------|------|-------|---------------------------|----------------------------------|
| 0    | 0    | 0     | D0                        | BD0 1. lap oszlopcím             |
| 0    | 0    | 1     | D1                        | BD1 1. lap sorcím                |
| 0    | 1    | 0     | D2                        | BD2 2. lap oszlopcím             |
| 0    | 1    |   1   | D3                        | BD3 2. lap sorcím                |
| 1    | 0    | 0     | D4                        | BD4 Megjelenítési oszlopcím      |
| 1    | 0    | 1     | D5                        | BD5 Megjelenítési sorcím         |
| 1    | 1    | 0     | D6                        | BD6 Megjelenítési oszlopcím      |
| 1    | 1    | 1     | D7                        | BD7 Megjelenítési sorcím         |

A VDA7 jelen keresztül a multiplexer ily módon az aktuális memória művelethez kijelöli a video lapozó regiszter állapotának megfelelő 16 kbyte-os egységet a videomemóriából.

### 3.2.5 WAIT ÁRAMKÖR

Csak a HBA és HBA-1 kártyán található. A 3. lapon keressük. Az F7/7 J-K flip-flop J bemenete fix állapotú, az NK (vigyázat, negált!) bemenetet az NM1 jel vezérli. A flip-flop órajele a CPU órajelével egyezik. Minden utasításlehíváskor egy CPU óraütem időtartamra az addig 0 értékű NQ kimenet 1 értékre vált (csak egy WAIT ciklust kérhet az áramkör). Ez az 1 szint az E11/8 nyitott kollektoros NAND kapu kimenetét 0 szintbe vezérli, ez jut a CPU WAIT bemenetére, pontosabban az

<!-- pdf 95 -->

E12/8 kimenet 0 szintre állásának az is feltétele, hogy valamilyen eszköz kérje a WAIT állapotot.

Az F5/8 NAND kapura a programmodultól (FAST), a bővítő egységektől (NSLOWEXP), a TB1/1 átkötés esetén a rendszer ROM 16 kbyte-os részétől (NROM) vagy a TB9 átkötés megléte esetén a rendszer ROM 8 kbyte-os kiterjesztéséről (NROM5) érkezhet 0 szint, ami az F5/8 kimenetet 1 értékre állítja. Az E7/7 flip-flop kimenet és az F5/8 kapukimenet egyidejű 1 szintje váltja ki a WAIT kérést.

## 3.3 I/O részegységek áramkörei

### 3.3.1 I/O DEKÓDER ÁRAMKÖREI

Az I/O dekóder áramkörei a kapcsolási rajzok 6. lapján találhatók.

A 8. ábrán szereplő blokkok közül az elsődleges dekódernek a kapcsolási rajzon az E5(E4) pozíciójú IC felel meg. A B5(F5) dekóder áramkör a másodlagos dekóder 0 egységet, az NWR0-7 jelek előállítását valósítja meg.

A B5(F5) dekóder Y2 és Y4 jelű kimenetén nem tüntettünk fel jelneveket, mivel ezeket a jeleket lapon belül használjuk. Az Y2 kimenet a C3(C3) regiszterbe (nyomtató csatoló adatregiszter) írást vezérli és a blokkvázlaton az NWR1 jelnévvel szerepel. Az Y4 kimenet az F1(G1) regiszterbe (billentyűzetregiszter), ill. bővítő lapregiszter egy tokban) tölti az adatbuszon lévő adatot és NWR3 jelnévvel azonosítjuk a vázlatokon.

Az E5(E4) dekóder Y6 kimenete (amely az 5 bemenő bitkombinációnak felel meg) az F6/13(J17/10) kapun az A3 címbittel együtt alakítja ki az NMFCK jelet, azaz a kapu felel meg a blokkvázlaton kapuáramkör névvel jelölt egységnek.

<!-- pdf 96 -->

Az Y6 kimenet jele a G14/11(J1/4) kapun át az F11/6-8(G2/6-8) kapukra jut, ahol az A3=1 feltétel esetén (58H-5FH címtartomány) az RD vagy a WR 1 szintű impulzusának idejére az E4(E3) 2x2-ből 4 típusú dekóder működését engedélyező 0 szint alakul ki.

Olvasáskor az F11/8(G2/8) kapu a dekóder 1Y kimeneteinek aktivitását engedélyezi, íráskor az F11/6(G11/6) kapu a 2Y jelű kimenetek működését teszi lehetővé. Az A0 és A1 címbitek jelölik ki, hogy melyik kimenet válthat 0 szintbe.

A kimenőjelek elnevezése a blokkvázlatokból ismert, kivéve az E4(E3) tok 1Y0 jelzésű kimenetét. A jel az El(F1) buszmeghajtót (billentyűzet oszlop adatok) kapcsolja az adatbuszra és a vázlatokon NRDTAST jelnévvel azonosítottuk.

### 3.3.2 BILLENTYŰZET CSATOLÓ

A kapcsolási rajzok 6. lapján található.

A 9. ábrán található sorregiszter a kapcsolási rajz F1(G1) pozíciójú tokjában foglal helyet. A 6 bites regiszter fennmaradó két bit tárolási lehetősége a bővítő lapozó regisztert valósítja meg.

A regiszterbe az NWR3 jel tölti az adatot (a kapcsolási rajzon a jelnév nincs feltüntetve).

A sordekóder a G1(H1) pozíciójú IC. Látható, hogy kimenetei a CH9 csatlakozóra jutnak, amelyen keresztül a billentyűzet csatlakozik.

Az E1(F1) buszmeghajtó kapcsolja a billentyűzet mátrixból, ill. a botkormánytól érkező állapotszót a BD0-BD7 adatbuszra. A rákapcsolást az NRDTAST jel végzi, amely csak a blokkvázlaton van feltüntetve, a kapcsolási rajzokon nem.

A billentyűzetről és a botkormánytól érkező vezetékek az RN1 ellen<!-- pdf 97 -->állás-hálózaton keresztül tápfeszültségre vannak kapcsolva. Ez azt biztosítja, hogy a billentyűzet mátrix azon oszlopvezetékei, amelyek nincsenek sorvezetékekhez kötve, egyértelmű logikai 1 szinten vannak.

A billentyűzet mátrix oszlopvezetékeit a CH8, a botkormányok érintkezőit a CH7 és CH6 csatlakozó kapcsolja a HBA kártyára.

A D3-D14 diódák azt akadályozzák meg, hogy az egyik botkormány lekérdezését a másik zavarja.

### 3.3.3 MAGNETOFON CSATOLÓ

A csatoló a kapcsolási rajzok 5. lapján található.

A kimenőadat előállítására szolgáló flip-flop E7/10(J20/10) pozíciójú. Felhívjuk a figyelmet arra, hogy a HBA kártyán található többi J-K flip-flop-tól eltérően a flip-flop-nál a K bemenet negált, tehát a flip-flop az itt található kapcsolásban az NMFCK bemeneten érkező 0-1 átmenetek hatására ellenkező állapotba billen.

Az R8 és R9 (R35 és R34) ellenállások a kimenő szintet állítják be, a C110(C4) kondenzátornak zavarszűrő funkciója van.

A magnetofonmotor indítását, beállítását az A1(B12) pozíciójú, nagyáramú tranzisztorokat tartalmazó IC-k végzik. A tranzisztorok nyitott vagy zárt állapotát a B1/8 és B1/2 (C11/4 és C11/12) invertereken át a C4(C4) regiszter 2 bitje határozza meg. A regisztert az NWR5 jel tölti.

A CH12 és CH13 csatlakozók mindegyikére egy-egy magnetofon kapcsolható. A két csatlakozó látható módon a motorvezérlő jeltől eltekintve teljesen párhuzamosan van kötve.

A magnetofonról érkező adat az A3(A7) műveleti erősítőre kerül.

<!-- pdf 98 -->

A TTL szintre beállított jel (ECIN) a kapcsolási rajzok 10. lapján található E3(E1) buszmeghajtón át az IT állapotszóval együtt kerül a CPU adatbuszára.

### 3.3.4 NYOMTATÓ CSATOLÓ

A kapcsolási rajzok 6. lapján kereshetjük meg.

Az adatregisztert a C3(C3) tok alkotja. Betöltése az NWR1 jellel történik (a kapcsolási rajzon nincs feltüntetve). A regisztert az NRESET jel törli, így a nyomtató felé az ASCII null karakter jelenik meg.

Az adatbyte-nak a regiszterbe töltése után a CPU NWR6 jellel a 7-es adatbiten (BD7) küldött 0 értéket a B2/9(B2/9) flip-flop-ba írja. A flip-flop kimenete az NDSTRB adatérvényesítő jel a nyomtató felé. A B2/5(B2/5) flip-flop kimenetet 0 értékűre állítja.

A nyomtató, miután a TVC által küldött adatot elfogadta, az NACK jel 0 szintű impulzusával nyugtázza azt. Az impulzus 1 szintbe állítja a B2/9(B2/9) flip-flop-ot (megszűnik az adatérvényesítési állapot az NDSTRB jelen), hátsó, azaz felfutó éle 1 értékre állítja a B2/5(B2/5) flip-flop-ot, azaz az SB7 jelet. Ez a jel – amelyet egyébként az NRESET jel is 1 értékre állít – az IT állapotszóban eljut a CPU-hoz. A program ezt a jelet vizsgálva állapítja meg, hogy a nyomtató kész-e újabb adat fogadására. Amennyiben a nyomtató vételkészsége fennáll, újra indulhat az adatbyte töltése és az adatérvényesítés.

### 3.3.5 HANGÁRAMKÖRÖK

A kapcsolási rajzok 5. lapján találhatjuk ezeket az áramköröket.

A C5(C5) IC és a C4(C4) IC egyik fele alkotja a frekvenciaregisztert, így áll össze a 12 bit, amely az A2, B7 és B6 (B5, B6 és B4) tokokból felépített programozható számlánc osztásarányát meghatározza.

<!-- pdf 99 -->

A számlánc a következőképpen működik:

Mindhárom IC órajelbemenetén (CP) ugyanaz a 3M125 jel jelenik meg. Az IC-k azonban csak akkor veszik ezt az órajelet figyelembe, ha az ET és EP bemenetük mindegyike logikai 1 szinten van. Az A2(B5) IC esetében ez mindig teljesül, ezért ez a tok az órajel valamennyi felfutó élére eggyel megnövelt értékre áll be. Az értéket a párhuzamos kimenetein (QA-QD) olvashatnánk le. Amikor a párhuzamos kimeneteinek mindegyike 1 értékű (hexadecimális F állapot), akkor a CY kimenetén is logikai 1 szint jelenik meg. Ez engedélyezi, hogy a B7(B6) tok a következő órajelélre egyet számoljon. A B7(B6) tok a következőkben csak akkor számolhat, ha az A2(B5) újra megtelik. Hasonlóképpen vezérli a B7(B6) IC a B6(B4) IC-t.

Mindezek alapján – egyéb vezérlés nélkül – az A2(B5) minden órajelélre számol és minden 16. óraütemben 1 állapotú a CY kimenete. A B7(B6) minden 16. órajelélre számol és minden 256. óraütemben 1 értékű a CY kimenete. A B6(B4) minden 256. órajelélre számol és minden 4096. ütemben 1 értékű a CY kimenete.

A számlánc programozhatósága a következőt jelenti:

Az L jelű bemenetére kötött 0 szint hatására az áramkör a felfutó órajelél megjelenésekor nem a következő értéket veszi fel, hanem a párhuzamos adatbemenetén lévő bitkombinációt helyezi a kimenetre. Az L bemenetet ezután 1 szintbe vezérelve a számlánc az így beállított értéktől számol tovább.

__Egy példa:__ A számlánc CH állapotban van, a bemenetén pedig pl. 3. Ha az L bemenet 1 értékű, akkor az órajelél hatására a DH állapot következik, ha az L bemeneten 0 szint van, akkor a 3-as állapotba áll a számlánc. Ezután addig, ameddig L=0, a számlánc mindig a 3-as állapotban marad. Az L bemenetre 1 szintet adva a következő léptetésre a 4-es állapot következik. Kössük most az áramkör CY kimenetét egy inverteren át az L bemenetre, és adjunk egy periodikus órajelet a CP bemenetre. A számlánc ilyenkor elszámol az FH értékig, itt 1 érték jelenik meg a CY <!-- pdf 100 --> kimeneten, az inverter ennek hatására 0-ba vezérli az L bemenetet. Legyen a párhuzamos bemeneten pl. 5. A számlánc az FH értékről 5-re áll, ismét számol újra FH értékig, ahonnét újra 5-ös állapotba kerül. A CY kimeneten megjelenő jel periódusideje 16-5=11 óraütem szélességű, azaz a CY jelének frekvenciája az órafrekvenciának tizenegyed része. A párhuzamos bemenetekre más értéket beállítva 2-től 16-ig bármilyen osztásarányt beállíthatunk.

A hangáramkörben ez a programozási lehetőség 12 biten áll fenn. A B6(B4) IC CY kimenete kerül az F6/10 (C12/1) kapun át mind a három IC L bemenetére. A párhuzamos adatbemenetekre a hangfrekvencia-regiszter, a C5 és C4 (C5 és C4) regiszterek kimenete kerül. A regiszterekbe a CPU tetszőleges értéket írhat, így 2-től 4096-ig bármilyen osztásarányt beállíthatunk. Ha a regiszterek tartalma 4094 (FFEH), akkor a 3M125 jel frekvenciáját felezi a számlánc, ha 0 a regisztertartalom, akkor 4096-tal osztja.

A programozható számlánc jele a D10(A1) aszinkron számláncra kerül. A C8 és R21 (C9 és R19) integráló áramkör az esetlegesen fellépő zavaró impulzusokat (ún. hazard) nyomja el.

A D10(A1) áramkör egy egybites és egy hárombites részre oszlik. A QA kimeneten megjelenő jel frekvenciája fele a programozható számlánc kimenő jele frekvenciájának. Ez a jel négyszöghullám és a soros vonali interface adási és vételi órajeleként használatos (TX/RXCLK).

A hárombites számlánc további 8-as osztást hoz létre, így keletkezik a hang előállításához szükséges jel.

Az INSTART jel, amely az I/O dekódertől érkezik, a D10(A1) számláncot 0 értékre állítja, a programozható számláncnál viszont az F6/10(C12/1) kapun át párhuzamos töltési állapotot hoz létre. A következő órajelélre a programozható számlánc a frekvenciaregiszter által meghatározott értékre áll be.

A hang előállításához szükséges jel két ÉS kapura, a G12/3 és G12/11 <!-- pdf 101 --> (A2/8 és A2/6) kapukra jut. A kapukon való átjutást a C4(C4) regiszter két kimenete engedélyezheti vagy tilthatja. Ez a kétbites regiszter a blokkvázlaton E/D regiszternévvel jelzett blokk.

A G12/11 (A2/6) kapun átjutva a jel IT-t generálhat (SDINT). Az INSTART jel fentiekben vázolt hatását figyelembe véve a program úgy állíthat elő széles intervallumba eső pontos időzítést, hogy a frekvenciaregisztert betölti a kívánt időzítésnek megfelelően, engedélyezi a hang IT beérkezését a C4(C4) regiszter 6Q jelű kimenetének 1 szintbe állításával, majd az INSTART jelet aktivizálja. A programozható számlánc nyolcadik alkalommal való megtelésekor a D10(A1) számlánc QD kimenetén megjelenő 1 szint IT kérésként jelentkezik a CPU-nál.

A G12/3(A2/8) kapu az amplitúdóbeállítóra engedheti a számlánc jelét.

Az amplitúdószabályozót a B3(A3) NOR kapuk és az R3-R7 (R20-R24) ellenállások alkotják.

Amikor valamely kapukimenet 0 szintű, akkor a hozzá tartozó ellenálláson át nem folyik áram, mivel az ellenállás másik vége az 1,2 kOhm-os ellenálláson át földre van kötve.

Az 1 állapotú (+5 V szintű) kimenetek ellenállásán át az ellenállás értékétől függő áram folyik az 1,2 kOhm-os ellenállás felé. Az ellenállás értékek olyanok, hogy a B3/11(A3/10) kimenet áramát egységnyinek véve a B3/10(A3/4) kimenet árama 2 egységnyi, a B3/4(A3/11) kimeneté 4 egységnyi és a B3/3(A3/3) kimeneté 8 egységnyi. Az összáram ily módon 0-tól 15 egységig vehet fel értékeket. Az áram mindig ugyanazon az 1,2 kOhm-os ellenálláson folyik át, így annak feszültsége (azaz a SOUND jel szintje) 16 különböző értéket vehet fel, amely értékek közel egyenlő távolságra vannak egymástól.

A működésről a fentiekben elmondottak természetesen egyszerűsített képet rajzolnak a programozható feszültségosztóról. Nyilvánvaló egyrészt, hogy a 10 kOhm-os ellenálláson a föld felé folyó áramot az 1,2 kOhm-os ellenállás jobban befolyásolja, mint a 82 kOhm-os ellen<!-- pdf 102 -->állás áramát, másrészt amikor vannak 1 és 0 szintű kapukimenetek is, akkor a 0 szintűek felé is folyik áram. Az ellenállásértékek azonban olyanok, hogy ezek a hatások nem befolyásolják az alapvető működést olyan mértékben, hogy a hang amplitúdó lépcsők méretében füllel észlelhető különbség lenne.

Azt, hogy a négy NOR kapu melyikének kimenetére juthat a számlánc jele, a B4(B3) regiszterkimenetek szabják meg. Ez a regiszter a blokkvázlat amplitúdóregiszterének felel meg.

A C4(C4) regiszter 5Q kimenetét 0 szintre állítva a számlánc jele helyett fix érték jut a NOR kapuk egyik bemenetére. Ilyenkor a SOUND jel szintjét csak az amplitúdóregiszter határozza meg. Megfelelő ütemben változtatva az amplitúdóregiszter tartalmát, egy 16 szintű lépcsős jelet állíthatunk elő, amely különböző hangeffektusokat hoz létre a hangszóróban.

A SOUND jel keveredve a bővítő csatlakozóról érkező hangjellel (pl. videojáték modul) a PAL kóderhez kerül, ill. a CH14 csatlakozón is megjelenik. Ez utóbbi esetben az RGB monitorhoz közvetlenül kerül a hangjel.

### 3.3.6 A MEGJELENÍTŐ LOGIKA I/O ÁRAMKÖREI

A CRTC a kapcsolási rajzok 7. lapján található, pozíciója E14(D11).

Az üzemmód-, a paletta- és a borderregiszter a kapcsolási rajzok 9. lapján kereshetők meg.

Az üzemmódregiszter az E9(F6) pozíciójú, SN74LS75 típusú IC egyik felében található. Az áramkör jellemzője, hogy amíg az E jelű bemenetén logikai 1 szint található, addig a Q jelű kimenete követi a D jelű bemenet szintváltozásait. Az NQ kimeneten természetesen mindenkor a Q kimenet jelének negáltja található. Az E bemeneten létrejövő 1-0 átmenet idején a D bemeneten fennálló állapotot rögzíti a Q kimeneten.

<!-- pdf 103 -->

A fentiek alapján érthető, hogy a regiszter az NWR6 jelet a kapcsolási rajz 6. lapján található B1/10(J1/12) inverter által negálva kapja az üzemmódregiszter.

A palettaregiszter a B8(B7) pozíciójú, SN74LS670 típusú tok. A D1-D4 jelű bemenetein megjelenő információt a GW jelű bemenetére adott 0 szintű impulzus a négy belső regiszter közül a WA és WB bemeneten lévő bitkombináció által kijelöltbe írja. Ha a GR bemenet 0 szintű, akkor a Q1-Q4 kimeneteken a belső regiszterek közül az RA és RB által kijelöltnek az adata jelenik meg. GR=1 feltétel esetén a kimenetek nagyimpedanciás állapotban vannak (TRISTATE).

A GW bemenet itt az NWCOL jel vezérli, az  A4/11(A4/3) kapu által kapcsolatba hozva az NWR jellel.

A borderregisztert a B11(B8) IC alkotja. Az SN74LS175 IC a CP jelű bemenetén megjelenő 0-1 átmenetkor a D jelű bemenetein lévő állapotot a Q jelű kimenetre helyezi. A C bemeneteire adott 0 szint a Q kimeneteket is 0 szintbe állítja. Itt a CP bemenet az NWR0, a CL bemenetet az NRESET jel vezérli.

A video lapozó regiszter csak a HBA-2 kártyán található. A kapcsolási rajzot a 4. lap tartalmazza. A C1 pozícióban lévő SN74LS174 típusú IC ugyanolyan D típusú flip-flop-ból áll, mint az előbbiekben ismertetett SN74LS175, csak négy helyett 6 ilyen flip-flop-ot tartalmaz. A regiszter mintavételező jelét (CP) a J19/8, a J18/3 és a J18/6 kapuk állítják elő.

<!-- pdf 104 -->

## 3.4 Az időzítő áramkörök

### 3.4.1 A RESET ÁRAMKÖR

A kapcsolási rajzok 2. lapján található.

A HBA kártyán a G9 pozíciójú, 8224 típusú IC 2. lábára kapcsolódó C108 kondenzátor a készülék bekapcsolásakor az R46 ellenálláson keresztül töltődik. A kondenzátoron lévő lassú felfutási jelből a G9 IC az 1. számú kimenetén egy H szintű impulzust állít elő. A D2 diódán át a kondenzátoron kikapcsoláskor gyorsan kisül.

A G9/1 impulzust a G11/12 inverter 0 szintű impulzussá alakítja.

a G2/3–6 kapuknak terhelésleválasztó funkciójuk van. Kimeneti jelük állítja az időzítő áramköröket alapállapotba.

A G11/12 inverter kimeneti jelén keletkező felfutó él hatására a G13 monostabil multivibrátor a C5 kondenzátor és a G13 belső ellenállása által meghatározott szélességű 1 szintű (RESET jel), ill. 0 szintű (NRESET) impulzust állít elő.

A K2 kapcsoló (RESET nyomógomb) működtetésekor az R52 ellenálláson át a C13 kondenzátor kisül. A nyomógomb felengedésekor a kondenzátor töltődni kezd. Feszültségszintjét az E12 flip-flop-on a CPU NM1 jele mintavételezi, biztosítva az NRESET jel fázisának a CPU működéséhez való igazítását. A flip-flop kimenete egy ugyanolyan impulzust generál a G13 áramkörön, mint a bekapcsoláskor a G9/1 kimeneti jel.

A HBA-2 kártyán a fentiektől csak annyi az eltérés, hogy a 8224 IC funkcióját az R6-C22-D1 elemekből felépített áramkör veszi át. Természetesen egyéb áramköröknek a pozíciói is mások, pl. a flip-flop G5, a monostabil multivibrátor J13 pozíciójú.

<!-- pdf 105 -->

### 3.4.2 AZ ÓRAJEL-GENERÁTOR ÁRAMKÖREI

A kapcsolási rajzok 2. lapját vegyük magunk elé. A HBA kártyán a G9 pozíciójú, 8224 típusjelű integrált áramkörön alapszik az oszcillátor. Frekvenciáját az IC XTAL1 és XTAL2 bemenete közé helyezett, X1 pozíciójú 25 MHz-es kvarckristály határozza meg. A kristály soros rezonanciafrekvencián rezeg, a 25 MHz a harmadik harmonikusnak felel meg. A TANK jelű bemenetre kapcsolt, az L1 induktivitásból és a C10 kondenzátorból álló rezgőkör segíti elő a harmadik harmonikuson való berezgést. A C10 kapacitás változtatható, ez a frekvencia finombeállítását teszi lehetővé.

Az oszcillátor hangolásakor ügyeljünk arra, hogy a G8/5 flip-flop kimenetre kapcsolódva a 12,5 MHz-es jelet figyelve végezzük a hangolást. A 8224 OSC jelű kimenetére helyezve a frekvenciamérő csatlakozóját a műszer kapacitása meghamisíthatja a mérést.

A HBA-2 kártyán diszkrét elemekből és inverterekből álló oszcillátort találunk. Az inverterek: J5/6, J5/2 és J5/12. Az oszcillátor frekvenciáját az X1 pozíciójú kristály szabja meg, amely ugyanolyan, mint amilyen a HBA kártyán található. Az L1 és C2 elemek a harmadik harmonikuson való berezgéshez szükségesek, az L1 állítható induktivitás a pontos frekvenciabeállítást teszi lehetővé.

Az R10, R11 és R12 ellenállások az invertereket az aktív munkapont közepébe állítják, ugyanis ebben a kapcsolásban az inverterek analóg eszközként működnek.

Az R9 ellenállás szerepe a pozitívan visszacsatolt kör csillapításának beállítása.

Az oszcillátor behangolásával kapcsolatban a HBA kártyánál elmondottak itt is érvényesek.

Az órajel-generátor további áramkörei a HBA és HBA-2 kártyán megegyeznek. Működésüket a 2.2.4 pontban áramköri szinten részletesen ismer<!-- pdf 106 -->tettük, itt csak a 13. ábrán és a kártyákon található áramköri elemek azonosítását adjuk meg:

| 13. ábra     | HBA kártya | HBA-2 kártya |
|--------------|------------|--------------|
| 1. flip-flop | G8/5       | J6/5         |
| 2. flip-flop | G5/3       | J7/3         |
| 3. flip-flop | G6/5       | H3/5         |
| 4. flip-flop | G6/9       | H3/9         |
| 1. NAND kapu | G4/3       | G3/8         |

### 3.4.3 A VIDEO IDŐZÍTŐ ÁRAMKÖRÖK

A kapcsolási rajzok 2. lapján találhatók. Az áramköröket a 2.2.4 pont részletesen ismerteti.

A 15. ábra áramköri elemei és a kártyákon található elemek közötti kapcsolat:

| 15. ábra                    | HBA kártya | HBA-2 kártya |
|-----------------------------|------------|--------------|
| 1. flip-flop                | G7/8       | H4/8         |
| 2. flip-flop                | G7/5       | H4/5         |
| 3. flip-flop| G5/5       | J7/5         |
| 4. flip-flop                | H1/6       | G5/6         |
| 1. kapu                     | F5/12      | G4/12        |
| 2. kapu                     | F5/6       | G4/11        |
| 3. kapu                     | G4/11      | G3/11        |
| 4. kapu                     | G4/8       | G3/6         |
| 5. kapu                     | G4/6       | G3/3         |
| 6. kapu                     | G12/6      | F6/6         |
|                             |            |              |

<!-- pdf 107 -->

### 3.4.4 A CLOCK STRETCH ÁRAMKÖR

A kapcsolási rajzok 2. lapján találhatók.

A 2.2.4 pont írja le a működést. A 18. ábra és a kapcsolási rajzok közötti összefüggés:

| 18. ábra                    | HBA kártya | HBA-2 kártya |
|-----------------------------|------------|--------------|
| 1. flip-flop                | F2/6       | J9/6         |
| 2. flip-flop                | F2/9       | J9/9         |
| 3. flip-flop| G3/3       | J10/3        |
| 4. flip-flop                | G3/6       | J10/6        |
| 1. kapu                     | F4/8       | J8/12        |
| 2. kapu                     | F4/12      | J8/6         |
| 3. kapu                     | F4/6       | J8/8         |
| 4. inverter                 | G11/10     | J11/10       |
| 5. inverter                 | G11/8      | J11/8        |
|                             |            |              |

<!-- pdf 108 -->

## 3.5 A megjelenítő egység áramkörei

A megjelenítő logika két fő eleme: a CRTC és a videomemória köré épül.

Az E14 (D11) CRTC fő feladata a videomemória ciklikus olvasása és a megjelenítés időzítő jeleinek az előállítása.

A memória címzése E14 (D11) MA0-MA11 címei és Ra0-RA1 (ROW ADDRESS-TV sorcím) segítségével, valamint a NVCAS, NVRAS, DEB és VMUX (N - alacsony szinten aktív, V - VIDEO) jelek segítségével történik.

A dinamikus 16 (64) kbites, 150 ns hozzáférési idejű, 320 ns ciklusidejű 4116 (4164) videomemória címzése többszörösen multiplexelve történik (74LS153), a multiplexelést az NVMUX jel és a DEB jel vezérli:

| DEB | L | H | L | H |
|-----|---|---|---|---|
| NVMUX | L | L | H | H |
| RAM címző bemenet | RAM bemenetére kapcsolt címvezetékek | | | |
| DA0 | MA0 | A0 | RA1 | A7 |
| DA1 | MA1 | A1 | MA6 | A8 |
| DA2 | MA2 | A2 | MA7 | A9 |
| DA3 | MA3 | A3 | MA8 | A10 |
| DA4 | MA4 | A4 | MA9 | A11 |
| DA5 | MA5 | A5 | MA10 | A12 |
| DA6 | RA0 | A6 | MA11 | A13 |
| A cím forrása: | CRTC | CPU | CRTC | CPU |

<!-- pdf 109 -->

A CRTC által szolgáltatott cím:

| 13 | 12 | 11 | 10 | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|----|----|----|----|---|---|---|---|---|---|---|---|---|---|
| MA 11 | MA 10 | MA 9 | MA 8 | MA 7 | MA 6 | RA 1 | RA 0 | MA 5 | MA 4 | MA 3 | MA 2 | MA 1 | MA 0 |

A videomemória ilyen cím multiplexelése lehetővé teszi annak CRTC és a Z80 CPU felüli elérését is.

A memóriachipek kimenete C7 (C8) (74LS374) D bemeneteire és C8 (C9), C9 (C10) (74LS195) párhuzamos/soros átalakító bemeneteire kapcsolódik. C7 (C8) kimenete a meghajtott (BUFFERELT) adatbuszra olvasható a CPU által egy videomemória olvasással (NRD, NBMREQ, NVRAM).

A megjelenítő memóriaadatokat a párhuzamos/soros átalakító C8 (C9), C8 (C9)/QD1, C9 (C10)/QB2, C9 (C10)QD2 kimeneteiről kapjuk.

| I | G | R | B |
|---|---|---|---|

16-színű üzemmódban ezeket a jeleket közvetlenül felhasználjuk, ezért ezek E9 (E6, F6) (74LS75) üzemmód flip-flop által meghatározott módon, azaz a B9 (B11) (74LS244) forrásszelektor kimenetén megjelenhetnek (B8 (B7) palettaregiszter: 74LS670 4+4 bites regiszter kimenetei ugyanekkor TRISTATE állapotban vannak és így nem vesznek részt a színek kialakításában.

Innen az IGRB bitek a B10 (B9) demultiplexer és regiszter (74LS298) egyik 4 bites bemenetére kerülnek (B10 (B9)/A1-B1-C1-D1).

A WSE jel szelektálja, hogy a regiszterbe az 1-es vagy a 2-es jelű <!-- pdf 110 --> bemenetek tartalma kerüljön beírásra az órabemenetre érkező (B10(B9)/11) órajel hatására.

A WSE jel végeredményben a CRTC display engedélyező DE jeléből származik: G15 (D13)/6.

A B10(B9)-es regiszter beíró órajele üzemmódtól függő sebességű kell, hogy legyen, erről az üzemmód flip-flop által vezérelt F7 (H5) óra demultiplexer gondoskodik.

Ugyancsak ez a demultiplexer (F7(H5)/7) biztosítja a C8-C9 (C9-C10) párhuzamos/soros átalakító órajelét, valamint G8 (J6) J-K flip-flop-on keresztül C8-C9 (C9-C10) léptetés/töltés vezérlését (RL).

Hogyan történik a BORDER szín előállítása?

A BORDER tartama alatt a B10 (B9) regiszterbe nem a memória I-G-R-B jele, hanem a B11 (B8) (74LS175) BORDER színrégiszter aktuális I-G-R-B tartalma kerül, mert ekkor a bemeneteket szelektáló jel az A2-B2-C2-D2 bemenetek tartalmát engedélyezi és az íródik az üzemmódnak megfelelő órajel hatására a B10 (B9) regisztereibe.

Az aktuális BORDER szín a B11 (B8)-as BORDER színregiszterbe korábban a Z80A CPU által OUT művelettel beírt I-G-R-B érték:

| 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |  |
|---|---|---|---|---|---|---|---|--|
| I | X | G | Х | R | X | В | X |  |

X = közömbös

A használt I/O cím = 0 (NW0)

<!-- pdf 111 -->

Például intenzív kék színt állítsunk be!

| 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|---|---|---|---|---|---|---|---|
| 1 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |

LD A,82H OUT A,(0)

Abban az esetben, ha 2- vagy 4-színű üzemmódot állítottunk be (amit E9 (E6-F6) üzemmód flip-flop kétbites kimenete jelez):

| E9(E6)/16 | E9(F6)/15 |                  |
|-----------|-----------|------------------|
| 0         | 0         | 2-színű üzemmód  |
| 0         | 1         | 4-színű üzemmód  |
| 1         | 0         | 16-színű üzemmód |
| 1         | 1         | 16-színű üzemmód |
|           |           |                  |

akkor a videomemória tartalma csupán regisztercímzési (PALETTE) feladatokat lát el. Ilyenkor – mint már korábban láttuk – a C8–C9(C9–C10) párhuzamos/soros regiszter kimenetei nem jutnak B9(B11)-en keresztül B10(B9) bemenetére.

Az történik ugyanis, hogy a C7(C8), azaz a videomemória kimenetén lévő SN74LS374 regiszter szolgáltatja az aktuális képpont(-ok) I-G-R-B értékeinek megfelelő palettaregiszter B8(B7) címet.

Az üzemmódregiszter most B9(B11) helyett a B8(B7) palettaregiszter olvasását engedélyezi.

<!-- pdf 112 -->

Pontosabban, annak a palettaregiszternek a tartalma kerül kiolvasásra a 4\*4 bites palettaregiszterből, amelynek címe éppen megjelenik a párhuzamos/soros C8-C9(C9-C10) átalakító QD2 és QD1 kimenetein. Ez a két bit szelektálja B8(B7) 4 regiszterét (2-színű üzemmódban csak a kettőt: ilyenkor B8(B7)/4 inaktív marad).

Természetesen a palettaregisztereit egy korábbi Z80-as CPU írásművelettel fel kell tölteni a kívánt I-G-R-B bitekkel:

A palettaregiszterek címei (NWCOL, AO, A1):

| A1 | AO | PALETTAREGISZTER      | I/O CÍM |  |
|----|----|-----------------------|---------|--|
| 0  | 0  | 0-ás palettaregiszter | 60H     |  |
| 0  | 1  | 1-es palettaregiszter | 61H     |  |
| 1  | 0  | 2-es palettaregiszter | 62H     |  |
| 1  | 1  | 3-as palettaregiszter | 63H     |  |

A palettaregiszter bitelrendezése:

| 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|---|---|---|---|---|---|---|---|
| X | I | X | G | X | R | X | В |

X = közömbös

Mielőtt a palettaregisztereket feltöltenénk a kívánt színinformációval, előzőleg az üzemmódregisztert be kell állítani a megfelelő üzemmódba:

Az üzemmód-regiszter címe: 06H

<!-- pdf 113 -->

Beállítása:

| 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |  |
|---|---|---|---|---|---|---|---|--|
|   |   |   |   |   |   | 0 | 0 | 2-színű |
|   |   |   |   |   |   | 0 | 1 | 4-színű |
|   |   |   |   |   |   | 1 | 0 | 16-színű |
|   |   |   |   |   |   | 1 | 1 | 16-színű |

Megjegyzés: ebben az esetben a 3-7 bitek értéke nem közömbös, mert D7 a nyomtató csatoló mintavételező (STROBE) jele, D2-5 pedig a hang amplitúdó beállítására szolgál.

Állítsunk be pl. 4-színű üzemmódot, majd rendre zöld, piros, kék és végül intenzív cián palettaregiszter tartalmat!

1. 4-színű üzemmód beállítása: LD A,1 OUT A,(6)

2. Palettaregiszter írása: LD A,10H ; sötétzöld

OUT A,(60H)

LD A,4 ; sötétvörös

OUT A,(61H)

LD A,1 ; sötétkék

OUT A,(62H)

LD A,51H ; világoscián

OUT A,(63H)

A megjelenítő áramkör B10(B9)-es (demultiplexer és regiszter) G-R-B kimenete egyrészt közvetlenül, másrészt A5(A8) AND kapuval az intenzitás bithez kapuzva A6(A10) (SN744273) videoregiszter bemeneteire kapcsolódik.

A regiszterbe a bemeneti információt a már tárgyalt F7(H5)/9 felől érkező és C11/10(C11/6)-n megjelenő órajel írja be A6(A10) videore<!-- pdf 114 -->giszterbe. A kimeneten megjelenő információt a visszafutások alatt az NVRCL (valamennyit videót letiltó) jel törli.

Térjünk vissza az időzítő jelek előállításáért felelős E14(D11) CRTC-hez. Az IC 39-es lábán jelenik meg a blokkvázlatoknál már tárgyalt HS, azaz sorszinkron (HORIZONTAL SYNC.) jel.

E14(D11) 40-es lábán a VS, azaz képszinkron jelet kapjuk.

E két vezérlőjel szélességét C10(J22), ill. B12(H13) monostabil multivibrátor állítja be a szükséges szélességre (SN74LS123).

C10(J22)/12-n így kapjuk az NHSYNC sorszinkronjelet. D11(C11)/10-n pedig az NVSYNC képszinkronjelet.

A két szinkronjelet (HSYNC, VSYNC) összekapuzva D12(C12) NOR kapuval, annak 4(13)-as kimenetén az NCSYNC, tehát a kompozit szinkronjelet kapjuk.

A C10(J22), B12(H13) monostabilból, valamint a C11(J21) NOR kapuból felépülő R-S flip-flop és kapu állítja elő az NVRCL jelet, amely a videoregiszter törlését végzi.

Az R-S flip-flop-ot E14(D11)/13, azaz MA9-es címvezetéke billenti be, ill. B12/5(H13/13) billenti vissza.

<!-- pdf 115 -->

## 3.6 A televízió bemenőjeleit előállító áramkörök

### 3.6.1 A PAL KÓDOLÓ

#### A PAL kódoló kártya

A PAL kódoló a TVC HBA nyomtatott áramköri alaplemezére csatlakoztatható kisméretű nyomtatott áramköri kártyán épül fel (HBP kártya). A 31. ábrán lévő blokkvázlat alapján könnyen követhetjük a kapcsolási rajzot, hiszen a modern LSI áramkörökből építkező áramköri technika szinte hűen követi a blokkvázlatot.

A megjelenítő logika R-G-B, ill. Ry-GY-BY színjelei, valamint a kompozit szinkron (NCSYNC) és sorszinkron (HSYNC) jel, továbbá a színkikapcsoló (KILLER) jel szolgáltatja a kódoló áramkör bemenőjeleit.

#### A HBP kártya áramkörei

A 31. ábrán lévő blokkvázlat 1-es TV. VIDEO mátrix D/A átalakítója a HBP kártya kapcsolási rajzán az A2-es pozícióban lévő LM 1886 IC, amely az A2/3-as és A2/4-es lábon az R-Y, ill. B-Y jeleket állítja elő, valamint az Y világosságjelét. (Mindhárom analóg jel, 1. 32. ábra.)

Az Y jel az Y=0,30\*R+0,59\*G+0,11\*B egyenlet alapján generálódik, az R-Y és B-Y jelek a túlmoduláció elkerülése céljából súlyozottak. Ez azt jelenti, hogy abból a célból, hogy megakadályozzuk azt, hogy az Y (világosság) jelre szuperponált 75% amplitúdójú színjel csúcsértéke túlmenjen a fehér szintnél, a színkülönbségi jelek amplitúdóját csökkenteni kell. (A szorzótényező: 1/2,03=0,493, ill. 1/1,14=0,877.)

Ezért a kék színkülönbségi vektor: U=0,493\*(B-Y) A piros színkülönbségi vektor: V=0,877\*(R-Y)

Az LM 1886 9 R-G-B bemenete közül (R0, R1, R2, G0, G1, G2, B0, B1, B2)

<!-- pdf 116 -->

csupán 6-ot használunk: R0-R1-et, G0-G1-et, B0-B1-et ugyanis párhuzamosan kapcsolva használjuk az R2, G2, B2 bemenetekkel együtt.

Így lemondunk olyan színekről, mint például a barna, narancs, égszínkék stb.

Az LM 1886-os IC-t elsősorban NTSC célra fejlesztették ki, de használható PAL kódoló felépítésére is.

Az áramkör beépített SZÍNSZINKRONJEL polaritás kapcsolót is tartalmaz, valamint egy bemenetet, amely a H/2 azaz a sorszinkronjel frekvenciájának a felét fogadja.

A kapcsolási rajzon a H/2 jelet a B3-D flip-flop 5-ös kimenetén kapjuk. Ez a D flip-flop a HSYNC jel frekvenciáját felezi.

#### A SZÍNSZINKRONJEL

A BURST SZÍNSZINKRONJEL kapcsoló vezérlését a C3 SN74123N dual monostabil multivibrátor végzi (késleltetés-szélesség áramkör), amely a sorszinkron jelet (HSYNC) kapja vezérlőjelként. A két monostabil multivibrátor tehát meghatározza a BURST pontos helyét és szélességét. (A BURST a sorszinkronjel kezdő élét követően 5,6±0,1 µs-mal kezdődik és 2,26±0,23 µs ideig tart, 1. 33. ábra.)

A BURST jel azért kell, hogy a színsegédvivő fázisát referenciaként átvigyük a PAL jel vételi oldali demoduláláshoz.

Maga a BURST színszinkronjel (szakaszosan továbbított pilotvivő) meghatározott periódusszámú modulálatlan szinuszos színsegédvivő, amelynek fázisa hordozza az információt. A kvadratúra modulált videojel csak a pontos fázis ismeretében demodulálható a vétel helyén, vagyis a televízióban, mert ez adja meg az ott folyamatosan működő referencia-oszcillátor számára az aktuális fázist. Ez tv-soronként ellentétes fázisú jel: 135, ill. 225 fok.

<!-- pdf 117 -->

A PAL kódolású jelnél a piros színkülönbségi jel fázisát tv-soronként szükséges ellentétes fázisúra átkapcsolni, hiszen – mint azt már korábban megbeszéltük – ilyen módon elérhető, hogy az eredő fázishibák kompenzálódjanak (feltéve, hogy a képtartalom két egymást követő tv-sorban nem változik meg ugrásszerűen).

A vételi oldalon a sorszinkronjelet követő BURST jel fázisa referencia arra vonatkozóan, hogy az aktuális tv-sorban invertált, vagy normál fázisú-e a piros színkülönbségi vektor.

A BURST jel fázisa nem invertált V vektorú tv-sorok előtt 135 fok, míg az invertált tv-sorokat megelőzően 225 fok.

Az LM 1886-os IC 3-as és 4-es kimenetén megjelenő színkülönbségi jel közül az R-Y jel tv-soronként fázist vált, (és így minden második tv-sorban a fázisa azonos) míg a B-Y jel fázisát nem kapcsolják át.

Az R-Y színkülönbségi jel fázisának átkapcsolását a H/2 jel végzi, amely az LM 1886-os 2-es lábára a B3/5 felől érkezik és minden második tv-sorban magas.

A színkülönbségi jeleket az A1-es video modulátor (LM 1889) integrált áramkör 2-es és 4-es bemenetére vezetjük. A B2-es 4 bites analóg kapcsoló (CD 4066) bekapcsolt állapotban a 10-es és 11-es, valamint a 2-es és 1-es lábait kapcsolja össze.

Ez azt jelenti, hogy R17 ellenállás az A1-es IC 2-es és 3-as lába közé van kapcsolva, R16 pedig A1 4-es és 3-as bemenete közé.

A BURST ideje alatt az analóg kapcsoló kikapcsolt állapotban van, mert C3/5 B2/12-es és 13-as vezérlő bemenetét Q4-en keresztül alacsony állapotban tartja.

Ez a kikapcsolás R-Y és B-Y bemeneteken egy negatív ugrást eredményez, mivel A1/2-es és 3-as, valamint 4-es és 3-as lába között most nincs ellenállás. Ez szinkronizált BURST-öt eredményez.

<!-- pdf 118 -->

Az A1 video modulátorhoz egy külső elemekből felépített PLL oszcillátor kapcsolódik. Ez 4,4336 MHz-es X1 kvarc-ból, fázistoló R-C hálózatból (R1-R2-C13-R4-R3-C3-C4-C6), egy egyenirányító diódából (D2), egy varicap diódából (D1), C1 NAND kapukból, C2 monostabil multivibrátorból, B1 számláncból, Q1 tranzisztorból, valamint R-C elemekből épül fel.

Az LM 1889-es modulátor 1-es és 18-as (LEAD-LAG) bemenete között 180 fokos fáziseltolás szükséges – ezt biztosítja a már említett fázistoló hálózat.

A PLL hurokra azért van szükség, hogy a színsegédvivő fázisát a sorszinkronjel fázisához rögzítsük, megakadályozva ezzel a színsegédvivő által okozott zavarok kellemetlen vándorlását a képernyőn.

A PLL úgy működik, hogy a kártyához érkező HSYNC jelet előbb 4-gyel osztjuk (B3/9), majd a leosztott jelet C2 monostabil multivibrátorra vezetjük (SN74121).

A monostabil C2/6-os kimenete adja a C1 NAND kapukból kialakított fázisösszehasonlító áramkör egyik bemenő jelét.

Az A1-es modulátor 17-es oszcillátor kimenőjelét Q1 emitterkövetővel illesztve B1/2 számlánc órajelbemenetére vezetjük.

A programozható (párhuzamosan tölthető) B1 számlánc (SN74LS163) párhuzamos bemenetére fix 0 1 0 0 információt vezettünk, a számlánc Qc kimenete pedig a párhuzamos betöltést vezérli (LOAD).

Így a számlánc 4-et oszt és a B1/13-as Qb kimenetéről elvezetett, leosztott színsegédvivő adja a fáziskomparátor másik bemenőjelét.

A fáziskomparátor kimenetén lévő passzív aluláteresztő szűrő a fáziseltéréssel arányos előfeszítést biztosít a D1 kapacitásdióda számára.

Mindennek az lesz az eredménye, hogy a fáziseltéréssel arányosan <!-- pdf 119 --> elhúzzuk X1 kvarc frekvenciáját, biztosítva ezzel a sorszinkron jellel fázisban lévő színsegédvivő frekvenciát.

Az LM 1889 beépített hangmodulátorát nem használjuk fel, ugyanis a hangmodulációt nem a PAL kódoló kártyán, hanem az UHF modulátor áramkörnél végezzük.

Az A1 modulátor 13-as kimenete az R-Y és B-Y színkülönbségi jelekkel kvadratúra modulált színsegédvivőt szolgáltatja.

A modulált színsegédvivő a Q3 tranzisztor kollektoráról a B2 analóg kapcsoló 4-es bemenetére kapcsolódik, ahonnan B2/5 magas állapotában – analóg kapcsoló bekapcsolva – a jel a Q2 összegző tranzisztor bázisára jut. Ugyancsak ide kapcsolódik A2 felől az A2/6-os kimenetről érkező világosságjel.

Az összegzést és egyben leválasztást végző Q2 emitterkövető a PAL kódolású kompozit (összetett) videojelet (CVID) szolgáltatja.

Abban az esetben, hogyha a TV-Computer hátoldalán található színkikapcsoló gombját megnyomjuk, a PAL kártya KILLER jelét ezzel alacsonyra húzzuk. Ez a jel B2/5-ön keresztül az analóg kapcsolót kikapcsolja, így a Q2 összegző tranzisztor bázisára csupán az Y, azaz világosságjel kerül.

Ez nem csupán azzal jár, hogy a színek megszűnnek a képernyőn, hanem a színsegédvivő hiánya miatt nem lesz a fekete-fehér képen színsegédvivő zavar sem.

A PAL kódoló kártya a tápfeszültséggel szemben támasztott magas követelmények miatt lokális feszültségstabilizátort tartalmaz: ST1 (7805 integrált 3 pontos soros típusú feszültségstabilizátor).

<!-- pdf 120 -->

### 3.6.2 AZ UHF MODULÁTOR ÁRAMKÖREI

Az UHF modulátor a TV-Computer HBC jelű kártyáján épül fel és feladata, hogy az alapsávi, kompozit PAL rendszerű videojelet AM modulációval, a hangjelet SOUND) pedig FM modulációval a televízió UHF 36-os csatornájára keverje, kialakítva ott a szabvány szerinti sávszélességű és alakú átviteli görbét.

Az áramkör alapja a TDA 5660 típusú integrált áramkör (Siemens), amely csaknem az egész UHF modulátort integrálva tartalmazza.

#### A modulátor felépítése

A modulátor felépítését a 34. ábrán lévő blokkvázlat alapján követhetjük nyomon.

A blokkvázlat egyes elemeinek jelentése:

- 1. hangvivő oszcillátor
- 2. hangfrekvenciás előerősítő FM modulátor
- 3.-4. hangmodulációs mélységszabályozás, AM hangmodulátor, első keverő
- 5. belső feszültség stabilizátor és referencia feszültség áramkör
- 6. rádiófrekvenciás kimenet meghajtó áramköre
- 7. balansz keverő (második keverő)
- 8. UHF oszcillátor meghajtó egysége
- 9. UHF vivő oszcillátor
- 10. videomodulációs mélységszabályozó áramkör
- 11. videomeghajtó áramkör 12.
- 12.-13.-15. videoszabályozó áramkör
- 14. rádiófrekvenciás keverő aszimmetria kompenzáló áramkör

<!-- pdf 121 -->

#### Áramköri ismertetés

A PAL kódoló felől érkező kompozit videojelet az N1-es integrált áramkör 10-es videobemenetére R1-R2 feszültségosztó után egy C1 kapacitással vezetjük.

A videojel amplitúdóját a modulátor számára az R5-P1-R6 osztó segítségével állíthatjuk be a P1 potenciométerrel. Az R5-P1-R6 feszültségosztó a belső referencia feszültségforráshoz kapcsolódik (2-es láb).

Az UHF oszcillátor számára L4 induktivitásból és C17 kapacitásból álló, az UHF 36-os csatornára fixen lehangolt párhuzamos rezgőkört kapacitívan csatoljuk az N1 IC-hez.

A hangfrekvenciás SOUND jelet szintén kapacitív csatolással, C2-R8 komplexummal vezetjük az FM hangbemenetre

Az 1-es hangbemenet R9 ellenálláson keresztül a 2-es lábon megjelenő belső 7,5 V-os referenciafeszültséghez kapcsolódik.

Az N1-es IC 18-as és 17-es lábára L3-C24 hangvivő frekvenciára hangolt párhuzamos rezgőkör kapcsolódik, R7 ellenállással együtt, amely a szükséges sávszélességet biztosítja.

C10-es kondenzátor a fel nem használt AM hangbemenetet hidegíti. Az IC 9-es lábára kapcsolódó R10-P2-C13 komplexum segítségével – amennyiben ez szükséges – kompenzálható az RF keverő aszimmetriája.

A szimmetrikus rádiófrekvenciás kimenetre az induktivitásokból, kapacitásokból és ellenállásokból felépülő L1-L2-R3-R4-C16-C17-C4-C5 komplexum, valamint egy bakun transzformátoros aszimmetrizáló kapcsolódik, biztosítva ezzel a televízióhoz csatlakoztatást 75 Ohm-os koax kábellel.

<!-- pdf 122 -->

# 4. HARDWARE-SOFTWARE INTERFACE

Ebben a fejezetben gyűjtöttük össze azokat az információkat, amelyek a TVC hardware funkcionális egységeinek a programból való vezérlésével kapcsolatosak. Ezeket az információkat az előző fejezetek is tartalmazzák, de csak a hardware ismertetéséhez szükséges mértékben.

Az információkat a 4.3 Táblázatok című alfejezetben adjuk meg. Az alábbiak csak a táblázatok használatához szükséges magyarázatokat tartalmazzák.

## 4.1 Memóriakezelés

A memóriák használatánál legfontosabb tudnivalók a lapozással kapcsolatosak. Ezen kívül csak a videomemória használatához kellenek speciális információk.

A fejezethez tartozó táblázatokon helyhiány miatt a szokásostól eltérően a hexadecimális számokat nem jelöljük H karakterrel.

### 4.1.1 MEMÓRIATÉRKÉP

Az 1. táblázaton találhatjuk a térképet, amely azt mutatja, hogy a fizikai memóriák a CPU címtartományban hol foglalhatnak helyet.

A lapok megnevezése (LAP0-LAP3) mellett feltüntetjük a címtartományt hexadecimális és decimális kezdő- és végcímekkel.

<!-- pdf 123 -->

A memóriatérképen található fizikai memóriák jelölése:

SYS Rendszer (SYSTEM) ROM 16 kbyte-os rész

U0-U3 Felhasználói (USER) RAM 4 db 16 kbyte-os terület

CART Programmodul (CARTRIDGE)

VID A videomemória valamelyik 16 kbyte-os területe

IOMEM Valamelyik bővítő egység 8 kbyte-os memóriája

EXT Rendszer ROM kiterjesztés

RESET hatására a feltüntetett konfiguráció áll be.

Az 1. táblázat alsó részén a memórialapozással kapcsolatos I/O műveletek láthatók (l. a 4.2.2 pont első szakaszát).

### 4.1.2 A VIDEOMEMÓRIA HASZNÁLATA

A videomemória 64 byte-ját olvassa ki a megjelenítő logika minden tv-sorban. A teljes 16 kbyte 256 sor tárolását biztosítja, de csak 240 tv-sor a megjelenített terület. A CPU címtartományában a LAP2 területre lapozva a képernyő bal felső sarkát meghatározó byte címe: 8000H. (A LAP1-re állítva a videomemóriát, nyilván 4000H ez a cím.)

A byte-ok címsorrendben vannak megjelenítve, balról jobbra és felülről lefelé, ily módon ha 256 tv-sort jelenítenénk meg, akkor a jobb alsó sarokban a BFFFH (7FFFH LAP1-en) lenne látható.

Minden tv-sorban a 64 byte fel van osztva 128 4 bites csoportra (16-színű üzemmód) vagy 256 2 bites csoportra (4-színű üzemmód), vagy 512 különálló bitre (2-színű üzemmód). Minden csoport a csoport bitjeinek számától függő számú színben megjelenített raszterpontot szolgáltat, 128 vagy 256 vagy 512 raszterpont vízszintes felbontást téve lehetővé.

A 2. táblázatban foglaltuk össze a videomemória és a képernyő közötti kapcsolatot.

<!-- pdf 124 -->

A felső táblázat a tv-sorok és a sorokon belüli byte-ok (karakterek) memóriacímét tartalmazza abban az esetben, ha a videomemóriát a LAP2-n kezeli a CPU. A LAP1-en nyilvánvalóan a címek első hexadecimális számjegye 4.

A mátrix szélén feltüntettük a CRTC fiktív karaktersorainak számát.

A mátrix alatt a memóriabyte-ok bitjeinek és a raszterpontoknak az összefüggését adjuk meg különböző üzemmódban. A C0 és C1 jel a palettaregiszter címbitjeit azonosítja.

## 4.2 Perifériakezelés

A fejezetben és a fejezethez tartozó táblázatokban az I/O címeket hexadecimálisan és – zárójelben – decimálisan is megadtuk.

A táblázatokban a nem használt biteket "-", a más funkcióhoz tartozó biteket "+" jellel jelöltük. Az utóbbi esetben az írásnál ügyelni kell arra, hogy a biteket a software megfelelően adminisztrálja és valamely funkcióhoz tartozó bitek írásakor a "+" jelű biteket változatlanul küldje ki.

### 4.2.1 I/O CÍMKIOSZTÁS

| Cím     | Bitek    | R/W | Funkció                                                 |
|---------|----------|-----|---------------------------------------------------------|
| 00H (0) | 7-5-3-1  | W   | Border-színregiszter                                    |
| 01H (1) | 76543210 | W   | Nyomtató párhuzamos adatregiszter                       |
| 02H (2) | 76543--- | W   | Memórialapozás vezérlő regiszter                        |
| 03H (3) | ++--3210 | W   | Billentyűzet lekérdezését irányító regiszter (sorok kijelölése) |

<!-- pdf 125 -->

| Cím            | Bitek    | R/W | Funkció                                              |
|----------------|----------|-----|------------------------------------------------------|
| 03H (3)        | 76--++++ | W   | Bővítő egysége RAM/ROM lapozó regisztere             |
| 04H (4)        | 76543210 | W   | Hangfrekvencia-regiszter (L byte)                    |
| 05H (5)        | ++++3210 | W   | Hangfrekvencia-regiszter (magasabb helyi értékű 4 bit) |
| 05H (5)        | ++54++++ | W   | Hangoszcillátor és hang IT enable-disable regiszter  |
| 05H (5)        | 76++++++ | W   | Magnetofon motorkapcsolást végző regiszter           |
| 06H (6)        | +-++++10 | W   | Video üzemmód-regiszter (színek számának kiválasztása) |
| 06H (6)        | +-5432++ | W   | Hang amplitúdó regiszter                             |
| 06H (6)        | 7-++++++ | W   | Nyomtató csatoló adatérvényesítő jele                |
| 07H (7)        | -------- | W   | A cursor/hang IT kérés törlése                       |
| 08-0BH (8-11)  |          |     | Nem használt                                         |
| 0CH (12)       |          |     | Video lapozó regiszter (csak 64k+)                   |
| 0D-0FH (13-15) |          |     | Ugyanaz, mint a 0CH                                  |
| 10-1FH (16-31) | 76543210 | R/W | A 0. bővítő kártyahely címei                         |
| 20-2FH (32-47) | 76543210 | R/W | Az 1. bővítő kártyahely címei                        |
| 30-3FH (48-63) | 76543210 | R/W | A 2. bővítő kártyahely címei                         |
| 40-4FH (64-79) | 76543210 | R/W | A 3. bővítő kártyahely címei                         |
| 50H (80)       | -------- | R/W | Magnetofon kimenőadat                                |
| 50-57H (81-87) |          |     | Ugyanaz, mint 50.                                    |
| 58H (88)       | 76543210 | R   | Billentyűzetállapot (oszlopok beolvasása             |
| 58H (88)       | 7------- | W   | A 0. bővítő kártyahely IT kérésének törlése          |
| 59H (89)       | +++43210 | R   | Függőben lévő IT kérések regisztere                  |

<!-- pdf 126 -->

| Cím              | Bitek    | R/W | Funkció                                                                                                                                       |
|------------------|----------|-----|-----------------------------------------------------------------------------------------------------------------------------------------------|
| 59H (89)         | 765+++++ | R   | 7: Nyomtató interface; nyomtató adatnyugtázó jele<br>6: fekete-fehér/színes kapcsoló jele<br>5: magnetofonbemenet |
| 59H (89)         | 7------- | W   | Az 1. bővítő kártyahely IT kérésének törlése |
| 5AH (90)         | 76543210 | R   | Bővítő kártya azonosító adatbyte |
| 5AH (90)         | 7------- | W   | A 2. bővítő kártyahely IT kérésének törlése |
| 5BH (91)         | -------- | R   | Hangoszcillátor RESET |
| 5BH (91)         | 7------- | W   | A 3. bővítő kártyahely IT kérésének törlése |
| 5C-5FH (92-95)   |          |     | Ugyanaz, mint 58-5B |
| 60-63H (96-99)   | -6-4-2-0 | W   | Színregiszter (palettaregiszter) |
| 64-6FH (100-111) |          |     | Ugyanaz, mint 60-63 |
| 70-71H (112-113) | 76543210 | R/W | 6845 CRTC regiszterei |
| 72-7FH (114-127) |          |     | Ugyanaz, mint 70-71 |
| 80-9FH (128-159) | 76543210 | R/W | Fenntartva a video játék modul számára |
| A0-FFH (160-255) |          |     | Nem használt |

### 4.2.2 AZ I/O CÍMEK KÉSZÜLÉKFUNKCIÓK SZERINTI SZÉTOSZTÁSA

#### Memórialapozás

Az 1. táblázat alsó részén található a lapozással kapcsolatos három I/O eszköz.

Az I/O címekhez tartozó táblázatokban az együtt hatásos bitek mellett az általuk kijelölt fizikai memória jele található.

<!-- pdf 127 -->

Az egész 1. táblázatra érvényes, hogy a fizikai memóriák jele után zárójelben egy hexadecimális számot láthatunk. Segítségükkel egyszerűen meghatározhatjuk egy adott lapozási állapot kialakításához az I/O címre küldendő adatot, csak a fizikai memóriák melletti számokat kell összeadni.

Példaként tekintsük meg az alábbi konfigurációt:

| LAP0 | SYS  | 00 |
|------|------|----|
| LAP1 | VID  | 04 |
| LAP2 | U2   | 20 |
| LAP3 | CART | 00 |
| Összeg: |   | 24 |

Ezt a számot kell a 02H címre küldeni.

#### Nyomtató és magnetofon interface

A 3. táblázat a két csatoló kezeléséhez szükséges I/O műveleteket tartalmazza.

#### Hang és billentyűzet

A 4. táblázatban találhatók az I/O műveletek. A táblázatot a következőkkel kell kiegészíteni:

A 04H címen írt byte és a 05H címen B0-B3 biteken írt 4 bit együtt egy 12 bites bináris számot határoz meg (n). Az írások elvégzése a frekvenciát 195.312,5/(4096-n) Hz értékre állítja be (0 ≤ n ≤ 4095). Ha n=4095, a hangoszcillátor leáll. A hang IT kezelését l. a 4.2.2 pont 7. szakaszában.

A billentyűzetmátrixot a 10. ábrán találhatjuk meg.

<!-- pdf 128 -->

#### Bővítő egységek

Az 5. táblázat tartalmazza a tudnivalókat. A CSTL0-CSTL3 a bővítő csatlakozók jelzése. A CSTL0 a jobb oldali, a CSTL3 a bal oldali csatlakozó.

A táblázat elemei közül az IOMEM lapozás az 1. táblázatban, a CSTL IT törlése és az IT állapotszó beolvasása a 8. táblázatban megtalálható.

#### A megjelenítő egység

A 6. táblázat használatához az eddig nem használt jelölések értelme:

I: intenzitás

G: zöld színjel

R: piros színjel

B: kék színjel

#### A CRTC

A 7. táblázat alsó részében a CRTC két I/O címe található. A felső részen a CRTC regiszterek funkciójának és jellemzőinek táblázatos összefoglalása mellett az is látható, hogy a TVC alapsoftware milyen értékkel tölti fel e regisztereket.

A CRTC regiszterek részletes leírását l. a 6.2 alfejezetben.

#### IT kezelés

A 8. táblázatból a szükséges információk kiolvashatók.

A CRTC R10 regiszterének írása azért került ebbe a táblázatba, mert – mint ismeretes – a CRTC cursor előállítására szánt jel IT kérésre van használva a TVC-ben. A cursor IT-t az R10 regiszter feltöltésével engedélyezhetjük vagy tilthatjuk.

<!-- pdf 129 -->

## 4.3 Táblázatok

1. táblázat MEMÓRIALAPOZÁS

<table>
<tr><td>LAP0</td><td>0000<br>3FFF</td><td>0<br>16383</td><td>SYS (00)<br>RESET</td><td>U0 (10)</td><td>CART (08)</td><td>U3 ** (18)</td></tr>
<tr><td>LAP1</td><td>4000<br>7FFF</td><td>16384<br>32767</td><td>U1 (00)<br>RESET</td><td>VID* (04)</td><td colspan="2" rowspan="2"></td></tr>
<tr><td>LAP2</td><td>8000<br>BFFF</td><td>32768<br>49151</td><td>VID (00)<br>RESET</td><td>U2 (20)</td></tr>
<tr><td rowspan="2">LAP3</td><td>C000<br>DFFF</td><td>49152<br>57343</td><td rowspan="2">CART (00)<br>RESET</td><td rowspan="2">SYS (40)</td><td rowspan="2">U3 (80)</td><td>IOMEM</td></tr>
<tr><td>E000<br>FFFF</td><td>57344<br>65535</td><td>(C0)<br>EXT</td></tr>
</table>

<!-- pdf 130 -->

I/O cím: 02H (2) írás: Lapozó regiszter

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td colspan="2">LAP3<br>00:CART (00)<br>01:SYS (40)<br>10:U3 (80)<br>11:EXT (C0)</td><td>LAP2<br>0:VID (00)<br>1:U2 (20)</td><td colspan="2">LAP0<br>00:SYS (00)<br>01:CART (08)<br>10:U0 (10)<br>11:U3 **(18)</td><td>LAP1<br>0:U1 (00)<br>1:VID*(04)</td><td>-</td><td>-</td></tr>
</table>

I/O cím: 03H (3) írás: Bővítő lapozó regiszter

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td colspan="2">IOMEM<br>00:CST0 (00)<br>01:CST1 (40)<br>10:CST2 (80)<br>11:CST3 (C0)</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td></tr>
</table>

I/O cím: 0FH (15) írás*: Video lapozó regiszter

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>-</td><td>-</td><td colspan="2">CRT<br>00:VID0 (00)<br>01:VID1 (10)<br>10:VID2 (20)<br>11:VID3 (30)</td><td colspan="2">LAP2<br>00:VID0 (00)<br>01:VID1 (04)<br>10:VID2 (08)<br>11:VID3 (0C)</td><td colspan="2">LAP1<br>00:VID0 (00)<br>01:VID1 (01)<br>10:VID2 (02)<br>11:VID3 (03)</td></tr>
</table>

\* Csak TVC 64k+

\*\* Csak TVC 64k+ és TVC 32

<!-- pdf 131 -->

2. táblázat VIDEOMEMÓRIA

"Karakter"

<table>
<tr><th>Tv-<br>sor</th><th>0</th><th>1</th><th>2</th><th>3</th><th></th><th>61</th><th>62</th><th>63</th><th>"Karakter"<br>sor</th></tr>
<tr><td>0</td><td>8000</td><td>8001</td><td>8002</td><td>8003</td><td>- - -</td><td>803D</td><td>803E</td><td>803F</td><td rowspan="4">0</td></tr>
<tr><td>1</td><td>8040</td><td>8041</td><td>8042</td><td>8043</td><td>- - -</td><td>807D</td><td>807E</td><td>807F</td></tr>
<tr><td>2</td><td>8080</td><td>8081</td><td>8082</td><td>8083</td><td>- - -</td><td>80BD</td><td>80BE</td><td>80BF</td></tr>
<tr><td>3</td><td>80C0</td><td>80C1</td><td>80C2</td><td>80C3</td><td>- - -</td><td>80FD</td><td>80FE</td><td>80FF</td></tr>
<tr><td>4</td><td>8100</td><td>8101</td><td>8102</td><td>8103</td><td>- - -</td><td>813D</td><td>813E</td><td>813F</td><td rowspan="4">1</td></tr>
<tr><td>5</td><td>8140</td><td>8141</td><td>8142</td><td>8143</td><td>- - -</td><td>817D</td><td>817E</td><td>817F</td></tr>
<tr><td>6</td><td>8180</td><td>8181</td><td>8182</td><td>8183</td><td>- - -</td><td>81BD</td><td>81BE</td><td>81BF</td></tr>
<tr><td>7</td><td>81C0</td><td>81C1</td><td>81C2</td><td>81C3</td><td>- - -</td><td>81FD</td><td>81FE</td><td>81FF</td></tr>
<tr><td>8</td><td>8200</td><td>8201</td><td>8202</td><td>8203</td><td>- - -</td><td>823D</td><td>823E</td><td>823F</td><td rowspan="4">2</td></tr>
<tr><td>9</td><td>8240</td><td>8241</td><td>8242</td><td>8243</td><td>- - -</td><td>827D</td><td>827E</td><td>827F</td></tr>
<tr><td>10</td><td>8280</td><td>8281</td><td>8282</td><td>8283</td><td>- - -</td><td>82BD</td><td>82BE</td><td>82BF</td></tr>
<tr><td>11</td><td>82C0</td><td>82C1</td><td>82C2</td><td>82C3</td><td>- - -</td><td>82FD</td><td>82FE</td><td>82FF</td></tr>
<tr><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td></tr>
<tr><td>232</td><td>BA00</td><td>BA01</td><td>BA02</td><td>BA03</td><td>- - -</td><td>BA3D</td><td>BA3E</td><td>BA3F</td><td rowspan="4">58</td></tr>
<tr><td>233</td><td>BA40</td><td>BA41</td><td>BA42</td><td>BA43</td><td>- - -</td><td>BA7D</td><td>BA7E</td><td>BA7F</td></tr>
<tr><td>234</td><td>BA80</td><td>BA81</td><td>BA82</td><td>BA83</td><td>- - -</td><td>BABD</td><td>BABE</td><td>BABF</td></tr>
<tr><td>235</td><td>BAC0</td><td>BAC1</td><td>BAC2</td><td>BAC3</td><td>- - -</td><td>BAFD</td><td>BAFE</td><td>BAFF</td></tr>
<tr><td>236</td><td>BB00</td><td>BB01</td><td>BB02</td><td>BB03</td><td>- - -</td><td>BB3D</td><td>BB3E</td><td>BB3F</td><td rowspan="4">59</td></tr>
<tr><td>237</td><td>BB40</td><td>BB41</td><td>BB42</td><td>BB43</td><td>- - -</td><td>BB7D</td><td>BB7E</td><td>BB7F</td></tr>
<tr><td>238</td><td>BB80</td><td>BB81</td><td>BB82</td><td>BB83</td><td>- - -</td><td>BBBD</td><td>BBBE</td><td>BBBF</td></tr>
<tr><td>239</td><td>BBC0</td><td>BBC1</td><td>BBC2</td><td>BBC3</td><td>- - -</td><td>BBFD</td><td>BBFE</td><td>BBFF</td></tr>
</table>

<!-- pdf 132 -->

2-színű üzemmód (8 pont/byte, pontsorrend: 1 2 3 4 5 6 7 8)

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>1.p. C0</td><td>2.p. C0</td><td>3.p. C0</td><td>4.p. C0</td><td>5.p. C0</td><td>6.p. C0</td><td>7.p. C0</td><td>8.p. C0</td></tr>
<tr><td colspan="8">C1=0</td></tr>
</table>

4-színű üzemmód (4 pont/byte, pontsorrend: 1 2 3 4)

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>1.p. C0</td><td>2.p. C0</td><td>3.p. C0</td><td>4.p. C0</td><td>1.p. C1</td><td>2.p. C1</td><td>3.p. C1</td><td>4.p. C1</td></tr>
</table>

16-színű üzemmód (2 pont/byte, pontsorrend: 1 2)

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>1.p. I</td><td>2.p. I</td><td>1.p. G</td><td>2.p. G</td><td>1.p. R</td><td>2.p. R</td><td>1.p. B</td><td>2.p. B</td></tr>
</table>

3. táblázat NYOMTATÓ ÉS MAGNETOFON INTERFACE

I/O cím: 59H (89) olvasás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>NACK</td><td>+</td><td>ECIN</td><td>+</td><td>+</td><td>+</td><td>+</td><td>+</td></tr>
</table>

NACK: nyomtató nyugtázó jele, RESET után READY állapotba (logikai 1 áll be

ECIN: magnetofonról érkező soros adat

<!-- pdf 133 -->

I/O cím: 06H (6) írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>NDSTRB</td><td>-</td><td>+</td><td>+</td><td>+</td><td>+</td><td>+</td><td>+</td></tr>
</table>

NDSTRB: a nyomtató csatoló adatérvényesítése<br>
0: adat érvényes<br>
RESET hatására 1 értékre áll be

I/O cím: 05H (5) írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>TK2</td><td>TK1</td><td>+</td><td>+</td><td>+</td><td>+</td><td>+</td><td>+</td></tr>
</table>

TK1: CH13 csatlakozóhoz magnetofon motorvezérlés<br>
TK2: CH12 csatlakozóhoz magnetofon motorvezérlés<br>
0: motor kikapcsol 1: motor bekapcsol<br>
RESET hatására kikapcsolt állapot áll be

I/O cím: 01H (1) írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td colspan="8">Karakterkód a nyomtatónak</td></tr>
</table>

RESET hatására ASCII "null" karakter áll be

<!-- pdf 134 -->

4. táblázat HANG ÉS BILLENTYŰZET

I/O cím: 04H (4) írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td colspan="8">Hangfrekvencia-regiszter alacsonyabb helyi értékű byte</td></tr>
</table>

I/O cím: 05H (5) írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>+</td><td>+</td><td>+</td><td>E/D</td><td colspan="4">Hangfrekvencia magasabb helyi értékű 4 bit</td></tr>
</table>

E/D: Hangoszcillátor engedélyezés (1)/tiltás (0)<br>
RESET hatására tiltás

I/O cím: 06H (6) írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>+</td><td>-</td><td colspan="4">Hang amplitúdó</td><td>+</td><td>+</td></tr>
</table>

RESET hatására 0 értékű amplitúdó

I/O cím: 03H (3) írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td colspan="8">Bill. adat (oszlop) beolvasás, aktív 0</td></tr>
</table>

<!-- pdf 135 -->

5. táblázat BŐVÍTŐ EGYSÉGEK

A bővítő egységek I/O címtartománya:

0. csatlakozó: 10H ... 1FH (16 ... 31)<br>
1. csatlakozó: 20H ... 2FH (32 ... 47)<br>
2. csatlakozó: 30H ... 3FH (48 ... 63)<br>
3. csatlakozó: 40H ... 4FH (64 ... 79)

I/O cím: 03H (3) írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td colspan="2">IOMEM lapozás<br>00:CSTL0 (00)<br>01:CSTL1 (40)<br>10:CSTL2 (80)<br>11:CSTL3 (C0)</td><td>-</td><td>-</td><td>+</td><td>+</td><td>+</td><td>+</td></tr>
</table>

I/O cím: 58H (88) írás: CSTL0<br>
59H (89) írás: CSTL1<br>
5AH (90) írás: CSTL2<br>
5BH (91) írás: CSTL3

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>CSTL IT<br>0: törl<br>tilt<br>1: eng</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td></tr>
</table>

<!-- pdf 136 -->

I/O cím: 59H (89) olvasás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td rowspan="2">+</td><td rowspan="2">+</td><td rowspan="2">+</td><td colspan="5">Függőben lévő IT kérések (0:aktív)</td></tr>
<tr><td>CU/HANG</td><td>CSTL3</td><td>CSTL2</td><td>CSTL1</td><td>CSTL0</td></tr>
</table>

I/O cím: 5AH (90) olvasás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td colspan="2">3. csatlakozó</td><td colspan="2">2. csatlakozó</td><td colspan="2">1. csatlakozó</td><td colspan="2">0. csatlakozó</td></tr>
<tr><td colspan="8">kártyaazonosító</td></tr>
<tr><td colspan="8">00: soros vonatl (RS232) 01: nem használt<br>10: floppy csatoló 11: üres csatlakozó v. nem specifikált</td></tr>
</table>

6. táblázat MEGJELENÍTŐ EGYSÉG

I/O cím: 00H (0) írás: border szín

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>I</td><td>-</td><td>G</td><td>-</td><td>R</td><td>-</td><td>B</td><td>-</td></tr>
</table>

I/O cím: 06H (6) írás: grafikus szín

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>+</td><td>-</td><td>+</td><td>+</td><td>+</td><td>+</td><td colspan="2">00: 2 szín<br>01: 4 szín<br>1X: 16 szín</td></tr>
</table>

<!-- pdf 137 -->

I/O cím: 59H (89) olvasás: színkapcsoló

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>+</td><td>0:F/F<br>1:szín.</td><td>+</td><td>+</td><td>+</td><td>+</td><td>+</td><td>+</td></tr>
</table>

I/O cím: 60H (96) írás: 0. paletta<br>
61H (97) írás: 1. paletta<br>
62H (98) írás: 2. paletta<br>
63H (99) írás: 3. paletta

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>-</td><td>I</td><td>-</td><td>G</td><td>-</td><td>R</td><td>-</td><td>B</td></tr>
</table>

Színek

<table>
<tr><th rowspan="2">I G R B</th><th rowspan="2">Szín</th><th colspan="2">GR. 2, 4 paletta</th><th colspan="2">Border</th></tr>
<tr><th>hex.</th><th>dec.</th><th>hex.</th><th>dec.</th></tr>
<tr><td>0 0 0 0</td><td>Fekete</td><td>00</td><td>0</td><td>00</td><td>0</td></tr>
<tr><td>0 0 0 1</td><td>Sötétkék</td><td>01</td><td>1</td><td>02</td><td>2</td></tr>
<tr><td>0 0 1 0</td><td>Sötétvörös</td><td>04</td><td>4</td><td>08</td><td>8</td></tr>
<tr><td>0 0 1 1</td><td>Sötétlila</td><td>05</td><td>5</td><td>0A</td><td>10</td></tr>
<tr><td>0 1 0 0</td><td>Sötétzöld</td><td>10</td><td>16</td><td>20</td><td>32</td></tr>
<tr><td>0 1 0 1</td><td>Sötét kékeszöld</td><td>11</td><td>17</td><td>22</td><td>34</td></tr>
<tr><td>0 1 1 0</td><td>Sötétsárga</td><td>14</td><td>20</td><td>28</td><td>40</td></tr>
<tr><td>0 1 1 1</td><td>Szürke</td><td>15</td><td>21</td><td>2A</td><td>42</td></tr>
<tr><td>1 0 0 0</td><td>Fekete</td><td>40</td><td>64</td><td>80</td><td>128</td></tr>
<tr><td>1 0 0 1</td><td>Kék</td><td>41</td><td>65</td><td>82</td><td>130</td></tr>
<tr><td>1 0 1 0</td><td>Vörös</td><td>44</td><td>68</td><td>88</td><td>136</td></tr>
<tr><td>1 0 1 1</td><td>Lila</td><td>45</td><td>69</td><td>8A</td><td>138</td></tr>
</table>

<!-- pdf 138 -->

<table>
<tr><th rowspan="2">I G R B</th><th rowspan="2">Szín</th><th colspan="2">GR. 2, 4 paletta</th><th colspan="2">Border</th></tr>
<tr><th>hex.</th><th>dec.</th><th>hex.</th><th>dec.</th></tr>
<tr><td>1 1 0 1</td><td>Kékeszöld</td><td>51</td><td>81</td><td>A2</td><td>162</td></tr>
<tr><td>1 1 1 0</td><td>Sárga</td><td>54</td><td>84</td><td>A8</td><td>168</td></tr>
<tr><td>1 1 1 1</td><td>Fehér</td><td>55</td><td>85</td><td>AA</td><td>170</td></tr>
</table>

<!-- pdf 139 -->

7. táblázat A CRT VEZÉRLŐ REGISZTEREI

<table>
<tr><th>Reg.</th><th>R/W</th><th>Egység</th><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th><th>Hex.</th><th>Dec.</th></tr>
<tr><td>R0</td><td>W</td><td>Kar.</td><td colspan="8">(Összes karakter)/sor - 1</td><td>63</td><td>99</td></tr>
<tr><td>R1</td><td>W</td><td>Kar.</td><td colspan="8">(Megjelenített karakter)/sor</td><td>40</td><td>64</td></tr>
<tr><td>R2</td><td>W</td><td>Kar.</td><td colspan="8">Vízszintes szinkron pozíció - 1</td><td>4B</td><td>75</td></tr>
<tr><td>R3</td><td>W</td><td>Tv-sor<br>Kar.</td><td colspan="4">* Függ. szinkron szélesség (tv-sor)</td><td colspan="4">Vízsz. szinkron szélesség (kar.)</td><td>32</td><td>50</td></tr>
<tr><td>R4</td><td>W</td><td>K.sor</td><td>-</td><td colspan="7">(Összes kar.sor)/kép - 1</td><td>4D</td><td>77</td></tr>
<tr><td>R5</td><td>W</td><td>Tv-sor</td><td>-</td><td>-</td><td>-</td><td colspan="5">Kiegészítő tv-sorsz.</td><td>02</td><td>2</td></tr>
<tr><td>R6</td><td>W</td><td>K.sor</td><td>-</td><td colspan="7">(Megjelenített karaktersor)/kép</td><td>3C</td><td>60</td></tr>
<tr><td>R7</td><td>W</td><td>K.sor</td><td>-</td><td colspan="7">Függőleges szinkron poz. - 1</td><td>42</td><td>66</td></tr>
<tr><td>R8</td><td>W</td><td></td><td colspan="2">*Cursor eltolás</td><td colspan="2">+DISPTMG eltolás</td><td>-</td><td>-</td><td colspan="2">Váltott sor<br>X0: nincs<br>01: van</td><td>00</td><td>0</td></tr>
<tr><td>R9</td><td>W</td><td>Tv-sor</td><td>-</td><td>-</td><td>-</td><td colspan="5">(Tv-sor)/(kar.sor) - 1</td><td>03</td><td>3</td></tr>
<tr><td>R10</td><td>W</td><td>Tv-sor</td><td>-</td><td colspan="2">Cursor<br>00: eng.<br>01: tilt.</td><td colspan="5">Cursor kezdő tv-sor a karaktersorban</td><td>03</td><td>3</td></tr>
<tr><td>R11</td><td>W</td><td>Tv-sor</td><td>-</td><td>-</td><td>-</td><td colspan="5">Cursor befejező tv-sor</td><td>03</td><td>3</td></tr>
<tr><td>R12</td><td>R/W</td><td></td><td>0</td><td>0</td><td colspan="6">Képkezdő memóriacím (H)</td><td>00</td><td>0</td></tr>
<tr><td>R13</td><td>R/W</td><td></td><td colspan="2"></td><td colspan="6">Képkezdő memóriacím (L)</td><td>00</td><td>0</td></tr>
<tr><td>R14</td><td>R/W</td><td></td><td>0</td><td>0</td><td colspan="6">Cursorpozíció (H)</td><td>0E</td><td>14</td></tr>
<tr><td>R15</td><td>R/W</td><td></td><td colspan="2"></td><td colspan="6">Cursorpozíció (L)</td><td>FF</td><td>255</td></tr>
<tr><td>R16</td><td>R</td><td></td><td>0</td><td>0</td><td colspan="6">Fénytollpozíció (H)</td><td></td><td></td></tr>
<tr><td>R17</td><td>R</td><td></td><td colspan="2"></td><td colspan="6">Fénytollpozíció (L)</td><td></td><td></td></tr>
</table>

Szállítótól és típusváltozótól függően<br>
- a * jelű funkciók hiányozhatnak<br>
- R12 és R13 csak írható (W)

<!-- pdf 140 -->

I/O cím: 70H (112) írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>-</td><td>-</td><td>-</td><td colspan="5">CRT vezérlő regiszter cím</td></tr>
</table>

I/O cím: 71H (113) írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td colspan="8">CRT vezérlő regiszter adat</td></tr>
</table>

8. táblázat MEGSZAKÍTÁSVEZÉRLÉS

I/O cím: 05H (5) írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>+</td><td>+</td><td>Hang IT<br>0: tilt<br>1: eng</td><td>+</td><td>+</td><td>+</td><td>+</td><td>+</td></tr>
</table>

CRT controller R10 írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td rowspan="2">0</td><td colspan="2" rowspan="2">00: cursor eng.<br>01: cursor tilt</td><td>0</td><td>0</td><td>0</td><td>1</td><td>1</td></tr>
<tr><td colspan="5">Cursor kezdősor a karaktersorban</td></tr>
</table>

<!-- pdf 141 -->

I/O cím 07H (7) írás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td></tr>
<tr><td colspan="8">Cursor/hang IT törlés</td></tr>
</table>

I/O cím 58H (88) írás: CSTL0<br>
59H (89) írás: CSTL1<br>
5AH (90) írás: CSTL2<br>
5BH (91) írás: CSTL3

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>CSTL IT<br>0: törl<br>tilt<br>1: eng</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td></tr>
</table>

I/O cím: 59H (89) olvasás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td rowspan="2">+</td><td rowspan="2">+</td><td rowspan="2">+</td><td colspan="5">Függőben lévő IT kérések (0: aktív)</td></tr>
<tr><td>CU/HANG</td><td>CSTL3</td><td>CSTL2</td><td>CSTL1</td><td>CSTL0</td></tr>
</table>

I/O cím: 5BH (91) olvasás

<table>
<tr><th>B7</th><th>B6</th><th>B5</th><th>B4</th><th>B3</th><th>B2</th><th>B1</th><th>B0</th></tr>
<tr><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td><td>-</td></tr>
<tr><td colspan="8">Hanggenerátor frekvenciaosztó törlés (pontos időzítés)</td></tr>
</table>

<!-- pdf 142 -->

# 5. AZ ALAPEGYSÉG FELÉPÍTÉSE (BONTÁS, ÖSSZESZERELÉS)

Az alapegység elektronikai egységei két műanyag fődarab között helyezkednek el. A két fődarabhoz több kiegészítő műanyag alkatrész csatlakozik. Ilyenek a bővítő egységek zárólapjai, a nyomtató interface takarólapja és a botkormány csatlakozóinak fedelei. A műanyag alkatrészek festett kivitelűek.

Az alapegység bontása a következő sorrendben történik:

A berendezésről a kiegészítő műanyag zárófedeleket eltávolítjuk és úgy fektetjük, hogy a billentyűzet nyomógombjai lefelé álljanak. A süllyesztett furatokban lévő lemezcsavarokat kicsavarva, a műanyag fődarab leemelhetővé válik. Az alkatrész eltávolítása után az elektronikai NYÁK lemezének forrasztási oldala hozzáférhető lesz, egyben leválasztható a billentyűzet alaplemezéről. Az elektronika NYÁK és a billentyűzet alaplemeze között a mechanikus és elektromos kapcsolatot egy 18 pólusú csatlakozó biztosítja. A két alaplemez szétválasztását kellő gondossággal kell végezni a csatlakozó elemek megóvása érdekében.

A billentyűzet alaplemezét a komplett billentyűzettel és a kezelői botkormánnyal 8 db lemezcsavar rögzíti a műanyag burkolat másik fődarabjához. A csavarok eltávolítása után az egység kiemelhető. A kezelői botkormány működtető karja a műanyag félnél marad a megvezető csapok tájolásának megfelelően.

A billentyűzet további bontásához ki kell csavarni a 2.2x6.5 mm átmérőjű lemezcsavarokat, amelyek az alaplemezt rögzítik a mozgó csapokat megvezető házhoz. A megvezető házban lévő mozgó csapok bontják a gombsapkákat és működtetik az alaplemezre helyezett vezető felületű gumikorongokat.

<!-- pdf 143 -->

Az alapegység összeszerelése a bontás sorrendjének fordítottja szerint végezhető el.

Figyelmet kell fordítani a billentyűzet alaplemezének összeszerelésénél a gumiharangok elhelyezkedésére, a billentyűzet alaplemez felfogatásánál a kezelői botkormány működtető karjának tájolására, az alapegység illesztésénél a 18 pólusú csatlakozó pozicionálására és összenyomására.

A teljes összeszerelés a műanyag fődarab felcsavarozásával és a kiegészítő műanyag alkatrészek behelyezésével fejeződik be.

<!-- pdf 144 -->

# 6. A TV-COMPUTER NAGYINTEGRÁLTSÁGÚ ALKATRÉSZEI

## 6.1 A Z80A mikroprocesszor

A TV-Computerben a számítógép elemei egy Z80A mikroprocesszor köré épülnek, amely a hazánkban készült mikrogépek leggyakrabban alkalmazott 8 bites mikroprocesszora. Ezért igen terjedelmes azoknak a jól használható leírásoknak, szakkönyveknek a köre, amelyek ennek a mikroprocesszornak a vezérlésével, utasításkészletével, alkalmazásával részletesen foglalkoznak. Itt csupán azokkal az ismeretekkel foglalkozunk, amelyek a TV-Computer megértéséhez nélkülözhetetlenek. Mindezek előrebocsátása után tekintsük át a Z80A főbb jellemzőit.

### 6.1.1 A Z80A FŐBB JELLEMZŐI

A Z80A a ZILOG cég Z80 típusú, 8 bit adatszélességű mikroprocesszorának 4 MHz órafrekvenciájú változata. A mikroprocesszor belső felépítését a 35. ábra mutatja.

A mikroprocesszor kifelé, a külső áramköri egységekkel (beviteli/kiviteli, memória) való kapcsolattartás érdekében ún. buszrendszereket, párhuzamos jelvezetékeket használ.

#### Adatbusz

8 bites kétirányú, háromállapotú (TRISTATE) magas aktív busz (D0-D7). A három állapot: logikai 0, logikai 1, valamint nagyimpedanciás állapot lehet. A nagyimpedanciás állapotot gyakran lebegő (FLOATING) állapotnak is nevezik.

<!-- pdf 145 -->

#### A címbusz

16 bites kifelé irányuló, háromállapotú magas aktív busz, amely 64 kbyte memória közvetlen címzésére alkalmas (A0-A15) és ezen kívül I/O egységek (256 db) címzését is lehetővé teszi (A0-A7).

#### A vezérlő busz jelei

5 db kifelé irányuló, alacsony aktív, a külső elemek és a mikroprocesszor együttműködését biztosító ún. rendszervezérlő busz (SYSTEM CONTROL BUS) jelei: NM1, NMREQ, NIORQ, NRD, NWR, NRFSH.

A fenti jeleken kívül az alacsony aktív mikroprocesszor és buszvezérlő jelek: NHALT, NWAIT, NINT, NNMI, NRESET, NBUSRQ, NBUSAK biztosítják még a rendszer helyes működését.

#### A CPU órajele

A mikroprocesszor működésének külső időalapja egy egyfázisú, nem TTL szintű órajel. Ezt külső, általában kvarcvezérelt óragenerátor biztosítja. Z80A-nál az óra frekvenciája max. 4 MHz. (A TV-Computer esetében ennek a jelnek a frekvenciája: 3,125 MHz.) Valamennyi jel - az órajel kivételével - TTL szintű.

#### Z80A kivezetéseinek elrendezése

Az integrált áramkör 40 lábú, kétsoros lábelrendezésű, ún. DIL (DUAL IN LINE), műanyag tokozású változatát használjuk a TV-Computerben. Ennek a lábelrendezését a 36. ábra mutatja.

<!-- pdf 146 -->

### 6.1.2 A CPU JELEINEK MEGNEVEZÉSE ÉS ÉRTELMEZÉSE

#### NM1 gépi utasításlehívási ciklus (MACHINE FETCH CYCLE) jel

Ez egy alacsony aktív kimenet. Azt jelzi, hogy a mikroprocesszor utasítást hív le a memóriából. Abban az esetben, ha az utasítás műveleti kódja nem egy, hanem két byte-ból áll, ez a kimenet mind a két byte lehívása esetén aktív. Egy megszakítás elfogadása esetén NM1 az NIORQ jellel együtt lesz aktív.

#### NMREQ memóriahozzáférés kérése (MEMORY REQUEST) jel

Alacsony aktív, háromállapotú (TRISTATE) kimenet, amely azt jelzi, hogy mikroprocesszor (CPU) címbuszán érvényes memóriacím van (A0-A15).

#### NIORQ I/O hozzáférés kérése (INPUT/OUTPUT REQUEST) jel

Alacsony aktív, háromállapotú kimenet. Azt jelzi, hogy a CPU a címbusz alsó helyi értékű 8 bitjén az input/output egység címét küldi ki, és az érvényes. Ha ez a jel az NM1-gyel egyidejűleg aktív (NM1 és NIORQ), akkor kell a CPU adatbuszára rákapcsolni (kapuzni) az IT-t kérő perifériális egységnek a megszakítási vektorcím alacsony helyi értékű 8 bitjét.

#### NRD olvasás (READ) jel

Alacsony aktív, háromállapotú kimenet. Aktív állapota azt jelzi, hogy a mikroprocesszor olvasási (beviteli) műveletet hajt végre a memóriából vagy az input/output egységből. Az adatbusz iránya: bemenet. A memória ill. az I/O egység adatainak kapuzására használható.

#### NWR írás (WRITE) jel

Alacsony aktív, háromállapotú kimenet, aktív állapota azt jelzi, hogy a mikroprocesszor írási (kiviteli) műveletet hajt végre a memória vagy az input/output egység irányába. Az adatbusz iránya: kimenet (OUTPUT).

<!-- pdf 147 -->

#### NRFSH felfrissítés (REFRESH) jel

Alacsony aktív, háromállapotú kimenet. Azt jelzi, hogy a mikroprocesszor címbuszának alacsony helyi értékű 7 bitjén a felfrissítő R (REFRESH) regiszter tartalma van jelen, amelyet dinamikus memória felfrissítésére használhatunk. Ha NRFSH és NMREQ egyidejűleg aktív, akkor a dinamikus memóriacellák adatfrissítő olvasása folyik.

#### NHALT állj (HALT) jel

Alacsony aktív kimenet, amely HALT utasítás végrehajtása során azt jelzi, hogy a CPU egy EI utasítással korábban engedélyezett maszkolható, avagy egy nem maszkolható (NMI) megszakításra várakozik. A várakozás azt jelenti, hogy a CPU NOP (NO OPERATION, azaz nem tényleges művelet) ciklusokat hajt végre.

#### NWAIT várakozás (WAIT) jel

Alacsony aktív bemenet, amely azt jelzi a CPU számára, hogy a memória vagy az I/O egység még nem áll készen az adat átvitelére. Az NWAIT jel segítségével a CPU hozzászinkronizálható egy lassúbb elérésű külső memóriához vagy I/O egységhez.

#### NINT megszakításkérő (INTERRUPT REQUEST) jel

Alacsony aktív, maszkolható megszakításkérés bemenet. Akkor vált ki megszakítást, hogyha korábban a megszakítást egy SW. "EI" (ENABLE INTERRUPT, azaz megszakításengedélyezés) utasítás már engedélyezte, továbbá, ha egy külső egység nem kéri a CPU-tól, hogy adja át a buszok (cím, adat, vezérlő busz) vezérlését a NBUSRQ (buszkérés) jellel. Ha a megszakítás kérése elfogadható, akkor a CPU ezt a következő utasításciklusban két vezérlőjelének egyidejű aktívba állításával jelzi a megszakítást kérő egység felé (NIORQ és NM1).

Megszakításkor a futó program végrehajtása utasításhatáron megszakad. Az aktuális jellemzők közül a PC tartalma a STACK-be (veremmemóriába)

<!-- pdf 148 -->

mentődik. A PC-be a megszakítási szubrutin címe töltődik és a megszakítási szubrutin futása kezdődik. Általában a szubrutin elején elmentjük a korábbi program fontosabb jellemzőit: pl. az F (FLAG) byte-ot (programjelző regisztereket tartalmazó byte-ot), a BC, DE stb. regiszterek tartalmát a STACK-be, vagy a belső fő és alternatív regiszterekkel cserét hajtunk végre.

Ezután a megszakítási szubrutin programlépései következnek. A szubrutin végén az elmentett jellemzők visszatöltése következik a STACK-ből a CPU regisztereibe, mégpedig fordított sorrendben, mint ahogyan kimentettük korábban, vagy a belső fő és alternatív regiszterblokk cseréjére kerül sor.

Amennyiben a további megszakításokat is figyelembe kívánjuk venni, újra engedélyezzük a maszkolható megszakítást EI (ENABLE INTERRUPT, azaz megszakítás engedélyezése) utasítással. A megszakítási szubrutin kötelezően RETI (RETURN FROM INTERRUPT, azaz visszatérés megszakítási szubrutinból) utasításra végződik.

A megszakítás tárolására a CPU két belső flip-flop-ot tartalmaz (IFF1, IFF2). Ezek lehetővé teszik a megszakítás beállítását, másolását (mentését), valamint tiltását.

#### NNMI nem maszkolható megszakítás (NON MASKABLE INTERRUPT) jel

Lefutó élre aktív, megszakításkérés-bemenet, amely függetlenül a megszakításengedélyező belső flip-flop (IFF1, IFF2) állapotától, automatikusan RESTART szubrutint hív a memória 66H címéről. Mint minden szubrutinhívás, ez is automatikusan elmenti a programszámláló (PC) tartalmát a memóriaverembe (STACK) és a szubrutin hívási címét (hexa 66) a CPU PC regiszterébe tölti. A megszakítási szubrutin kötelezően RETN (RETURN FROM NON MASKABLE INTERRUPT, azaz visszatérés nem maszkolható megszakítási szubrutinból) utasításra végződik. Ez biztosítja a stack-be mentett utasításszámláló-tartalom visszamentését a mikroprocesszor PC regiszterébe.

<!-- pdf 149 -->

#### NRESET törlés (RESET) jel

Alacsony aktív bemenet, amely a programszámlálót (PC) nullázza, tiltja a maszkolható megszakítást engedélyező flip-flop-ot, O-ás megszakítási módot állít be, törli az R (REFRESH, azaz felfrissítő) regisztert, valamint az I (INTERRUPT, azaz megszakítási vektor lapcím) regisztert. Mindaddig, amíg az NRESET bemenet alacsony, addig a címbusz, az adatbusz lebegő (nagyimpedanciás) állapotban marad, a vezérlőjel kimenetek inaktívak. Miután a buszok és a vezérlőjel kimenetek inaktívak, valamint az R regiszter nullázódik, az NRESET tartama alatt természetesen a CPU nem biztosítja a külső dinamikus memóriák felfrissítését. Ezért vagy a RESET időtartamát kell korlátozni, vagy külső eszközzel szükséges a felfrissítést biztosítani.

#### NBUSRQ buszkérés (BUS REQUEST) jelbemenet

Alacsony aktív bemenet, lehetővé teszi, hogy a buszok (cím, adat, vezérlő) vezérlését a CPU-tól egy külső vezérlő eszköz: pl. egy Z80A DMA (DIRECT MEMORY ACCESS CONTROLLER? azaz közvetlen memória hozzáférés vezérlő) elkérje. Az NBUSRQ, az NNMI, az NINT bemenetek a felsorolás szerint balról jobbra csökkenő prioritásúak.

#### NBUSAK buszkérés nyugtázása (BUS ACKNOWLEDGE)

Alacsony aktív kimenet, a buszkérésre adott válaszjel. A buszokat elkérő külső egység felé azt jelzi, hogy a CPU a buszok vezérlését átadta, azaz a CPU buszrendszerei nagyimpedanciás (TRISTATE), azaz inaktív állapotban vannak, így a buszok vezérlését a kérő eszköz átveheti. Tekintettel a buszrendszerek inaktív állapotára a NBUSAK tartama alatt a CPU felől a memóriafelfrissítés szünetel, így a frissítést külső eszköznek kell biztosítania. NBUSAK alatt a CPU a NBUSRQ jelet továbbra is figyeli és mindaddig, ameddig ez aktív a buszokat elengedve tartja.

<!-- pdf 150 -->

#### Az órajel

A Z80A CPU egyetlen nem TTL szintű, egyfázisú órajel bemenete, amely azonban nyitott kollektoros (OPEN COLLECTOR) inverterrel vagy tranzisztoros áramkörrel egyszerűen meghajtható.

### 6.1.3 A CPU BELSŐ FELÉPÍTÉSE

A CPU a külvilággal a 8 bit szélességű, kétirányú adatbuszon keresztül kommunikál, a 16 bit szélességű címbusz a memóriát, ennek alsó helyi értékű 8 bitje a perifériális elemeket címzi.

A CPU belsejében az adatbuszhoz egy belső buszrendszer kapcsolódik, amely az adatmozgatásokat teszi lehetővé a mikroprocesszor belső áramköreinek egységei között.

A CPU nagyszámú belső regisztert tartalmaz, amelyek közül igen sok a felhasználó számára is elérhető az utasításkészlet segítségével.

Megkülönböztetünk fő (MAIN) és alternatív (ALTERNATE) regisztereket, valamint általános célú és speciális célú regisztereket.

#### A CPU regiszterei

A (8 bit) AKKUMULÁTOR (fő)<br>
A' (8 bit) AKKUMULÁTOR' (alternatív)<br>
F (8 bit) Programjelző bitek (FLAG-ek) regisztere (fő)<br>
F' (8 bit) Programjelző bitek (FLAG'-ek) regisztere (alternatív)

Általános célú regiszterek

B (8 bit), C (8 bit) reg. vagy BC reg.-pár (16 bit) (fő)<br>
B' (8 bit), C' (8 bit) reg. vagy BC' reg.-pár (16 bit) (alternatív)<br>
D (8 bit), E (8 bit) reg. vagy DE reg.-pár (16 bit) (fő)<br>
D' (8 bit), E' (8 bit) reg. vagy DE' reg.-pár (16 bit) (alternatív)

<!-- pdf 151 -->

H (8 bit), L (8 bit) reg. vagy HL reg.-pár (16 bit) (fő)

H' (8 bit), L' (8 bit) reg. vagy HL' reg.-pár (16 bit) (alternatív)

Ezekből a regiszterekből – amint láthatjuk – két készlet is van, fő- és alternatív (') regiszterek, amelyek gyors csere- (CHANGE) műveletekkel felcserélhetők és így aktuálissá tehetők. Így szemben a viszonylag lassú, több memória-hozzáfordulást igénylő verem memóriaműveletekkel (PUSH, POP), a programváltások egyszerű regisztertömbcserékkel igen gyorsan megvalósíthatók.

Speciális célú regiszterek

I (8 bit) megszakítási lapcímregiszter (INTERRUPT VEKTOR REG.)

R (7 bit) memóriafrissítő regiszter (REFRESH REG.)

IX h (8 bit), IX l (8 bit) vagy IX indexregiszter (16 bit)

IY h (8 bit), IY l (8 bit) vagy IY indexregiszter (16 bit)

SP (16 bit) veremcímmutató regiszter (STACK POINTER)

PC (16 bit) programszámláló regiszter (PROGRAM COUNTER)

#### A CPU regisztereinek részletezése

Beszéljünk kissé részletesebben ezekről a regiszterekről! Az A regiszter többcélú regiszter, amelyet az aritmetikai vagy logikai műveletvégzés előtt az egyik operandussal kell feltöltenünk, a műveletvégzés után pedig általában az eredménnyel felülíródik, tehát a művelet eredményét tartalmazza. Az akkumulátort gyakran az INPUT/OUTPUT műveleteknél forrás- (SOURCE), ill. cél- (DESTINATION) regiszterként használjuk. Ezért kiviteli (OUT) írásműveletek előtt az akkumulátornak tartalmaznia kell a kiküldendő byte-ot, hiszen innen mint forrásregiszterből kerül ki a megcímzett perifériás egységre a kiküldendő adat.

Beviteli (NP) olvasásműveleteknél ide az akkumulátorba mint célregiszterbe kerül a megcímzett perifériális egységről az adat.

<!-- pdf 152 -->

#### F programjelző bitek regisztere (állapotregiszter)

A mikroprocesszor aritmetikai/logikai műveletvégző egységének (ALU) 6 speciális: ún. FLAG kimenete van. Ezek az ALU kimenetek az utasítások végrehajtása során fontos eredményeket szolgáltatnak: jelzik, hogy az ALU műveletek nyomán keletkezett-e negatív vagy zérus eredmény, volt-e félátvitel, a paritás páros-e, volt- túlcsordulás, összeadás/kivonás volt-e, átvitel keletkezett-e? Ezeket a kimeneteket programjelzőknek (FLAG) nevezzük.

#### Jelölésük a következő:

- S (SIGN) előjel
- Z (ZERO) zérus
- H (HALF CARRY) félátvitel
- P/V (PARITY/OVERFLOW) paritás/túlcsordulás
- C (CARRY) átvitel.

Az F regiszterben a programjelző bitek a következő elrendezésben találhatók:

| 7     | 6 | 5 | 4 | 3 | 2   | 1 | 0 |
|-------|---|---|---|---|-----|---|---|
| S | Z | X | Н | X | P/V | N | С |

#### X: közömbös

A programjelző bitek képezik az alapját a mikroprocesszor feltételektől függő utasítás végrehajtásának.

A feltétel flag-ek az aritmetikai/logikai utasítások végrehajtása során a műveletvégzés eredményétől függően beállítódnak. Ezek után a programozó feltételtől függő utasítások céltudatos használatával biztosítja a szükséges elágazásokat a programjában.

<!-- pdf 153 -->

A programjelzők értéke a műveletvégzés után a következő lesz:

#### Előjel S flag

S=1, ha a műveletvégzés után az A regiszter (eredmény) legmagasabb helyi értékű bitje 1-es:

| 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |             |
|---|---|---|---|---|---|---|---|-------------|
| 1 | . | . | . | . | . | . | . | A regiszter |

ellenkező esetben S=0

#### Zérus Z flag

Z=1, ha az ALU művelet után az eredmény zérus.

| 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |             |
|---|---|---|---|---|---|---|---|-------------|
| 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | A regiszter |

ha nem zérus, akkor Z=0

#### Félátvitel H flag

H=1, ha tetrádátvitel keletkezett a 3-ról a 4. bitre. Ha nem keletkezett tetrádátvitel, akkor H=0 lesz.

#### Paritás/túlcsordulás P/V flag

P/V=1, ha logikai műveletvégzés után az eredmény paritása páros, P/V=0, ha logikai műveletvégzés után az eredmény páratlan.

P/V=1, ha aritmetikai műveletvégzés után túlcsordulás keletkezett,

<!-- pdf 154 -->

P/V=0, ha az aritmetikai műveletvégzés után nem keletkezett túlcsordulás.

#### Összeadás/kivonás N flag

N=1, ha a művelet kivonás volt, N=0, ha a művelet összeadás volt.

#### Átvitel C flag

C=1, ha a művelet végrehajtása során igaz (1) értékű átvitel keletkezett a legmagasabb helyi értékű bitről, ellenkező esetben C=0 lesz.

#### A flagek felhasználása:

A feltételtől függő utasítások egyik részében az utasítás célja akkor valósul meg, ha a feltételnek kiválasztott flagbit értéke igaz.

Pl. ha S=1, akkor ugrik vagy szubrutint hív, vagy szubrutinból visszatér.

A feltételtől függő utasítások másik részében az utasítás célja akkor valósul meg, ha az aktuális flagbit értéke hamis.

Pl. ha S=0, akkor ugrik vagy szubrutint hív, vagy szubrutinból visszatér.

Ha az utasítás célja (ugrás, szubrutinhívás vagy szubrutinból visszatérés) nem valósul meg, akkor a következő utasítást hajtja végre a mikroprocesszor.

#### Az ALU

Tekintettel arra, hogy mindezek a mikroprocesszor műveletvégző egységéhez kapcsolódnak, szóljunk néhány szót az aritmetikai/logikai műveletvégző egységről (ALU).

<!-- pdf 155 -->

Az ALU-nak két 8 bites bemenete van az operandusok fogadására, továbbá egy 8 bites kimenete, amelyen keresztül a művelet eredménye kapcsolódhat a CPU belső buszrendszerére.

Az ALU-nak 6 db egy bites programjelző flag kimenete van, amelyről már beszéltünk.

Az ALU művelet eredményét, amely 8 bites bináris szám, egy további, az ALU-hoz kapcsolódó áramköri egység segítségével átírhatjuk binárisan kódolt decimális számalakba (SW. DAA utasítás).

Pl. ha az A regiszter tartalma:

|     | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |          |
|-----|---|---|---|---|---|---|---|---|----------|
| BIN | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | = DEC 16 |

BCD átírást alkalmazva DAA utasítással:

|     | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|-----|---|---|---|---|---|---|---|---|
| BCD | 0 | 0 | 0 | 1 | 0 | 1 | 1 | 0 |

ALU műveletek

Az ALU műveletvégző a következő aritmetikai/logikai műveleteket hajtja végre:

- a, összeadás
- b, kivonás
- c, logikai ÉS kapcsolat
- d, logikai VAGY kapcsolat
- e, kizáró VAGY kapcsolat
- f, komparálás
- g, logikai/aritmetikai léptetés

<!-- pdf 156 -->

- h, inkrementálás
- i, dekrementálás
- j, bitbeállítás
- k, bittörlés
- l, bittesztelés

A, B és C, a D és E, a H és L regiszterek általános célú 8 bites regiszterek, amelyek 16 bites regiszterpárként is használhatók:

BC

DE

HL

Ebben az esetben a 8-15 biteket az első betűjelű regiszter pl. a B, a 0-7 biteket a második betűjelű regiszter tartalmazza:

| 15 | 14 | 13 | 12 | 11 | 10 | 9 | 8 |
|----|----|----|----|----|----|---|---|

| 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|---|---|---|---|---|---|---|---|

Gyakran, indirekt regiszteres címzéseknél az operandus helyének memóriacímét a HL regiszterpár tartalmazza: a H regiszter a cím magas helyi értékű részét, az L regiszter a cím alacsonyabb helyi értékű részét tartalmazza, innen a H (HIGH) és az L (LOW) elnevezés.

az IX és IY regiszterek:

Az IX, valamint IY 16 bites regiszterek lehetővé teszik, hogy a CPU az ún. indexregiszteres címzésmódot használja.

Ilyenkor az ún. báziscímet valamelyik indexregiszter tartalmazza és ehhez a címhez mint bázishoz képes egy numerikus konstans eltolást alkalmazhatunk, hogy az operandus helyét meghatározzuk.

<!-- pdf 157 -->

Megjegyzés: címzésmódoknak nevezzük az operandus (amellyel a műveletet végezzük) elérésének a lehetséges módjait: pl. ha az egyik operandus a memóriában van, ennek elérésére többféle lehetőség is kínálkozik. Elérhető pl. az ún. közvetlen címzésmódú utasítások segítségével. Ilyenkor az utasítás műveleti kódját követően a memóriában az a két byte-os cím található, ahol az operandus a memóriában elérhető. De elérhető más címzésmódú utasítással is, pl. a korábban már említett indirekt regiszteres címzéssel ilyenkor a HL regiszterpár 16 bites tartalma adja meg az operandus elérési helyét a memóriában stb.

Az aritmetikai, logikai, adatmozgató stb. utasításoknak többféle utasításváltozatai vannak. Ezek címzésmódjukban különböznek egymástól.

Így egy utasítástípuson belül - pl. összeadás - többféle címzésmódú összeadó utasítások szerepelnek. Ez fokozza az utasítások számát és hatékonyságát.

#### I vektorregiszter

A mikroprocesszor I vektorregisztere egy 16 bites cím magas helyi értékű 8 bitjét tartalmazza. Az I vektorregisztert az IT 2-es megszakítási üzemmódban használjuk.

A Z80A mikroprocesszornak 3 megszakításkezelő üzemmódja van:

IT0

IT1

IT2

Bekapcsolás után a CPU automatikusan IT0 üzemmódba kerül, de az üzemmód software (IM0) utasítással is beállítható. Ez azt jelenti, hogy ha maszkolható megszakítást kap (és az SW-ből EI utasítással engedélyezett), akkor olyan megszakítási ciklus kezdődik, amelynél a mikropro<!-- pdf 158 -->cesszor az adatbuszra kapcsolt utasítást (az ún. megszakítási ciklusban bekényszerített utasítást) hajtja végre.

IT1-es üzemmód SW utasítással állítható be (IM1) és ebben az esetben, ha engedélyezett megszakításkérés érkezik az NINT bemenetre, akkor a CPU a 38H címről szubrutinhívást hajt végre.

IT2-es üzemmód SW utasítással állítható be (IM2). Ebben az üzemmódban a megszakításkérés kezelésében a Z80A mikroprocesszor család elemeire is szükség van.

A Z80A családelemek rendelkeznek a mikroprocesszorhoz hasonlóan megszakítási vektorregiszterekkel.

A CPU I vektorregiszter egy memóriacím magasabb helyi értékű ún. laprészét (8-15-ig), a megszakítást kérő perifériális egység (családelem) pedig a vektor alacsony helyi értékű címbitjeit tartalmazza (0-7-ig).

A megszakításkérés tiltása után a CPU az NIORQ és NM1 jel egyidejű aktív állapotával jelzi az IT-t kérő perifériális családelem felé, hogy az adatbuszon keresztül elküldheti a CPU-nak a vektor alacsonyabb helyi értékű 8 bitjét.

A CPU a PC tartalmát a STACK-be menti, majd a két vektor tartalmából egy 16 bites memóriacímet képez. A memóriának erről a címéről és az 1-gyel megnövelt címéről 2 byte-ot elhelyez a PC-be.

A memóriában egy címtáblát helyeznek el és a 16 bites vektorcím tulajdonképpen ennek a táblának az aktuális (IT szintnek megfelelő) elemére mutat. A címtábla aktuális eleme azt a memóriacímet tartalmazza, ahol az IT-t kiszolgáló szubrutin kezdődik.

Mindezekből következik, hogy a mikroprocesszor I vektorregiszterébe és az aktuális perifériális családelem vektorregiszterébe olyan címet kell előzőleg tölteni, amely a címtáblázatnak arra az elemére mutat, amely az adott IT-t kiszolgáló szubrutin kezdetét jelöli ki.

<!-- pdf 159 -->

Megjegyzés: a TV-Computer jelenleg nem használja az IT2 módot.

#### Az R regiszter

Az R regiszter a mikroprocesszor felfrissítő regisztere. Ez azt a 7 bites dinamikus RAM felfrissítő címet tartalmazza, amelyik frissítés alatt megjelenik a CPU címbuszának alacsony helyi értékű 7 bitjén és tartalma minden utasításkezdéskor 1-gyel növekszik.

Megjegyzés: felfrissítéskor, a címbusz magas helyi értékű címbitjein

(A8–A15) az I regiszter tartalma megjelenik, miközben a

címbusz alacsony helyi értékű címbitjén az R regiszter

tartalma található.

#### Az SP veremmutató regiszter (STACK POINTER), másként: veremcímregiszter

A veremcímregiszter a memóriában létrehozott verem éppen aktuális elemét jelöli ki. Az SP tartalma a verem műveleteknél úgy változik, hogy a verem memóriába íráskor az SP tartalma csökken, a verem kiolvasásakor pedig növekszik. A veremről így mindig a legutoljára beírt elem olvasható ki.

Az ilyen jellegű tárolást, amelyet az SP segítségével megvalósítunk LIFO (LIFO: LAST IN FIRST OUT, azaz amit utoljára be azt legelőször ki) tárolásnak nevezzük.

Veremműveleteket gyakran használunk, amikor egy program pillanatnyi jellemzőit:

Pl. a programszámláló tartalmát, az akkumulátor tartalmát, a programjellemzőket vagy regiszterpárokat: a BC, DE vagy a HL tartalmát átmenetileg a memória veremrészébe mentjük. Ezeket később, ha szükség van rájuk, fordított sorrendben kivehetjük a veremből, vissza a mikroprocesszor megfelelő regisztereibe.

<!-- pdf 160 -->

Mindezeket a veremműveleteket SW utasítások biztosítják a programozó számára, azonban ezek éppen a gyakori memória-hozzáférés miatt meglehetősen lassúak.

#### A PC programszámláló-regiszter (PROGRAM COUNTER)

Ez a 16 bites regiszter a mikroprocesszor működése során azt az aktuális, azaz pillanatnyi memóriacímet tartalmazza, ahonnan majd a CPU az utasítást lehívja. A program futása során tartalma változik az utasítások végrehajtása szerint. Azt mutatja tehát meg, hogy hol tart a program végrehajtása. (A következő utasítás memória címére mutat vagy új értéket vehet fel pl. ugró utasítás végrehajtásakor, vagy éppen nullázódhat pl. aktív HW NRESET esetén.)

#### A CPU egyéb áramkörei

A mikroprocesszor működésében más, a felhasználó által közvetlenül nem elérhető regisztereknek és áramköri elemeknek is fontos szerepe van.

#### Az utasításregiszter

Az utasításregiszter a CPU adatbuszán, majd belső buszrendszerén keresztül kapja és átmenetileg tárolja az utasítás műveleti kódját. Erre azért van szükség, hogy az utasítás dekódolásának, ill. végrehajtásának ideje alatt az utasítás műveleti kódja rendelkezésre álljon.

#### Az utasításdekódoló és -vezérlő áramkör

Az utasításdekódoló, valamint az időzítő és vezérlő logika rendkívül fontos része a CPU-nak. Az utasításdekódoló az utasítás műveleti kódját vezérlő jelekké alakítja és az időzítő és vezérlő logikával együtt gondoskodik arról, hogy az utasítást a szükséges számú ún. T ütemben és ciklusban a mikroprocesszor egységei végrehajtsák.

<!-- pdf 161 -->

T: a gépi ütemidő egységét jelenti, ami egy külső óraperiódus idejével azonos idő. Ezért az egyes utasítások a műveleti kódjuktól függő számú és időtartamú ciklusokban hajthatók végre. Abban az esetben, hogyha a külső órajel kvarc pontosságú, akkor a mikroprocesszor utasításvégrehajtási ideje is ilyen lesz. A T ütemekből és az ezekből felépülő ciklusok számából kiszámítható az adott utasítás végrehajtási ideje. Ez fontos lehet, hiszen így pontosan meghatározható egy program futásának ideje is.

Megjegyzés: a TV-Computerben az ún. óranyújtó áramkör alkalmazása miatt az utasítások végrehajtási ideje nem minden esetben számolható ilyen egyszerűen, mert ha a videomemóriát a display vezérlő és a CPU is egyidejűleg címezné, akkor egy logika a mikroprocesszor órajelét megnyújtja. Ilyenkor a program végrehajtási ideje megnövekszik.

### 6.1.4 A MIKROPROCESSZOR MŰKÖDÉSÉNEK IDŐDIAGRAMJAI

Tekintsük meg mindezek után a mikroprocesszor működését bemutató idődiagramokat! 37.–54. ábra.

### 6.1.5 A CPU UTASÍTÁSKÉSZLETE

A CPU utasításkészletét az 55. és 56. ábra mutatja be.

<!-- pdf 162 -->

## 6.2 A 6845-ös CRTC

A katódsugárcsöves (CRT) display-k a számítógépek, számítógép-rendszerek legáltalánosabban használt adatkijelző berendezései. Teljesen elektronikus, mozgó alkatrészektől mentes felépítésük és ennek következményeként fennálló zajtalan, nagy megbízhatóságú, a környezettel szemben nem túl igényes üzemük miatt a kisebb számítógépekben beépítve, a nagyobbak mellett különálló perifériaként mindenhol megtalálhatók.

A TVC is CRT display-t használ adatmegjelenítésre. A display logikai áramkörei az alapegységen belül helyezkednek el, csak a CRT monitort (amely esetleg kommersz tv-készülék) kell kívülről hozzákapcsolni.

Az eltérő specifikációval rendelkező különböző display-k belső szerkezetének nagyfokú hasonlósága és az alkatrészgyártás fejlődése lehetővé tette, hogy olyan nagyintegráltságú alkatrészek készüljenek, amelyek a display-k zömét tartalmazzák. Ez a tény a display-k kialakítását és szerkezetét nagymértékben leegyszerűsíti, hiszen csak viszonylag egyszerű elektronikát kell köréjük építeni, ezenkívül belső állapotregisztereiket a kívánt specifikációnak megfelelően feltölteni (inicializálás) és már működőképes adatmegjelenítő logika áll rendelkezésre.

A 6845-ös egy ilyen célra készült nagyintegráltságú alkatrész.

Mielőtt az IC ismertetésére térnénk át, összefoglaljuk a CRT displaykkel kapcsolatos azon tudnivalókat, amelyek a CRTC felépítésének, működésének megértéséhez szükségesek.

<!-- pdf 163 -->

### 6.2.1 A TV RASZTERRENDSZERŰ DISPLAY-K

#### A CRT display-k általános jellemzése

A CRT display-kben az információt hordozó kép a tv-képhez hasonlóan egy katódsugárcső képernyőjén jelenik meg, ezért minden CRT display tartalmaz egy ún. CRT monitort. A CRT monitor olyan elektronikus berendezés, amely egy (fekete-fehér vagy színes) képcsövet, videoerősítőt, valamint a vízszintes és függőleges eltérítő áramköröket és szerelvényeket tartalmaz. A képcső fluoreszkáló bevonattal ellátott felületére egy elektronsugár (a színes képcsőnél három elektronsugár) rajzolja a képet. Az elektronsugár vagy sugarak intenzitását, ezzel a képelem fényerejét a videoerősítő, a sugár becsapódásának pozícióját a vízszintes és függőleges eltérítő szerelvények határozzák meg.

A fekete-fehér CRT monitor a display megjelenítő logika áramköreitől három jelet vár: sor- és képeltérítő jelet, valamint videojelet. A videojel a videoerősítőt vezérli, így a fényerőt határozza meg, a sor- és képeltérítő jelek az eltérítő szerelvényen keresztül az elektronsugarat pozicionálják.

A színes monitornak három videojel-bemenete van, ugyanis három alapszín keveréséből állítja elő az összes színt.

A CRT display-k döntő többsége a tv raszterrendszerű megjelenítést alkalmazza. Ebben a rendszerben az elektronsugár – ugyanúgy, mint a tv-készülékben – pontról pontra végigtapogatja a képernyő felületét. Az elektronsugár megfelelő pozícióban a display áramkörök az elektronsugár intenzitását ugrásszerűen megnövelik. A képernyőn ennek következtében intenzívebben világító pontok jelennek meg és ezek a pontok alkotják az információt hordozó képet. A vízszintes és függőleges eltérítő jelek az ilyen berendezésben teljesen függetlenek a mindenkori képtartalomtól. A képtartalmat meghatározó információt kizárólag a videojel (vagy jelek) hordozzák, ez különbözteti meg a tv raszterrendszerű display-ket az egyéb CRT display-ktől.

<!-- pdf 164 -->

A továbbiakban csak tv raszterrendszerű display-kről lesz szó.

Az elektronsugár által a képernyőre egyszer felrajzolt kép nagyon hamar eltűnik, stabil képet csak a képnek az elektronsugár által történő állandó felfrissítésével lehet elérni. A képre vonatkozó információt emiatt el kell tárolni egy megfelelő memóriában. Ezt a memóriát felfrissítő memóriának nevezzük. A display logikai áramkörei az információt minden képfelfrissítési ciklusban kiveszik a memóriából, és a videojelen át közlik a CRT monitorral.

A display-k két nagy csoportja: az alfanumerikus és grafikus display-k.

A következőkben ismertetni kívánjuk az alfanumerikus és grafikus display-k alapvető jellemzőit, felépítésük közös elemeit. Természetesen a különböző display-k tartalmaznak olyan elemeket is, amelyek az alapfunkciókon túlmenő szolgáltatásokat nyújtanak, és egyedi megoldásokat tartalmaznak. Az ilyen kiegészítő szolgáltatások közül csak azoknak a fő jellemzőit ismertetjük, amelyek megvalósításához a jelen leírás tárgyát képező CRTC segítséget nyújt.

A TVC grafikus display-t tartalmaz, ennek ellenére az alfanumerikus display-k ismertetését fontosnak tartjuk, ugyanis a 6845-ös alfanumerikus CRTC, csak a TVC-ben speciálisan van inicializálva. A működésének megértéséhez emiatt az alfanumerikus display-k felépítését is ismerni kell.

#### Az alfanumerikus display-k

#### Az alfanumerikus display-k képszervezése

Az alfanumerikus display-k elsősorban alfanumerikus szimbólumok (karakterek) és speciális szimbólumok megjelenítésére alkalmasak. A karakterek a képernyőn csak fix pozíciókban (karaktersorok és -oszlopok) jelenhetnek meg. Ez más szóval azt jelenti, hogy a képernyő hasznos (a képet hordozó) felülete vízszintesen és függőlegesen nagyobb <!-- pdf 165 --> egységekre, karakterpozíciókra van osztva. A felfrissítő memória rekeszei és a karakterpozíciók között kölcsönösen egyértelmű kapcsolat áll fenn. A felfrissítő memória szervezése meg kell, hogy feleljen a képszervezésnek. A memóriarekeszek száma a képernyőn egyidejűleg megjeleníthető karakterek számát, a rekeszek nagysága (a bitek száma) a megjeleníthető karakterformák számát határozza meg. Az egy memóriarekeszben lévő bitek egy adott kombinációja egy adott karakterformához tartozik, és karakterkódnak hívjuk. Alfanumerikus display esetén tehát a felfrissítő memóriában karakterkódok találhatók.

A karakterformák a tv raszterrendszer miatt csak egy adott mátrix pontjaiból alakíthatók ki. A mátrix mérete és benne a karakterformák kialakítása display-típusonként eltérő lehet. A karakterek pontmátrixára jellemző információt az alfanumerikus display egy másik memóriájában, az ún. karaktergenerátorban tárolják el. Ez a memória az esetek többségében ROM.

A karaktermátrixot (kiegészítve a karakterek és karaktersorok közötti szünettel) megadják a karakterpozíciók méretét. A karakterpozíciók mérete (raszterpontokban számolva), továbbá a karaktersorok és oszlopok száma meghatározza a teljes kép pontmátrixának méretét.

A teljes képszervezést az eddigieken kívül még az befolyásolja, hogy az elektronsugárnak egy vízszintes sor felrajzolása után vissza kell térnie a képernyő másik (általában a bal) oldalára, hogy a következő tv-sort elkezdhesse rajzolni (sorvisszafutás). A kép felrajzolása után pedig a képernyő felső részére kell beállnia (képvisszafutás).

#### A képrajzolás időbeli lefolyása

Az elektronsugár útja egy tv raszterrendszerű alfanumerikus displayben, önkényesen kiválasztva azt az időpontot, amikor a képet elkezdi rajzolni, azaz a képernyő hasznos felületének bal felső sarkához csapódik be:

Elkezdi rajzolni a legfelső karaktersor karakterei legfelső mátrixso<!-- pdf 166 -->rának raszterpontjait. Végigrajzolja a karaktersor összes karakterére vonatkozóan ezeket a mátrixsorokat, így elér a képernyő hasznos felületének jobb szélső mátrixpontjáig. Ezután visszatér a bal oldalra, hogy elkezdhesse ugyanezen karaktersor következő mátrixsorának rajzolását. Egy előrefutásból és egy visszafutásból álló útját tv-sornak nevezzük.

Az elektronsugár egy tv-sor idő alatt függőleges irányban is elmozdul, így a következő tv-sort az elsőtől meghatározott távolsággal lejjebb rajzolja.

Egy teljes karaktersor felrajzolásához annyi tv-sort kell az elektronsugárnak befutnia, amennyi rasztersorból áll egy karakterpozíció (a karakterek pontmátrix sorainak száma + a karaktersorokat elválasztó üres tv-sorok száma).

A karaktersor felrajzolása után az elektronsugár a következő karaktersorra tér át.

Az összes karaktersor felrajzolása után eléri a képernyő a hasznos felületének jobb alsó sarkát. Ezután meghatározott idő alatt visszatér a kiindulóponthoz és újra kezdi rajzolni a képet. A teljes kép kialakításához szükséges időt felfrissítési ciklusnak nevezzük.

Az előzőek alapján a megjelenítés folyamatának jellegzetes időtartamai:

- raszterpontütem: az az idő, ameddig az elektronsugár egy raszterpontnyi utat fut be a képernyőn
- karakterütem: az az idő, ameddig az elektronsugár egy karakter egy mátrixsorát és a karakterek közötti szünetet felrajzolja
- tv-sorütem: egy tv-sor (előrefutás és visszafutás) befutásának időtartama (egy adott display-re jellemző a sorelőrefutási és visszafutási idő abszolút értéke és aránya is)

<!-- pdf 167 -->

- karaktersorütem: egy karaktersor felrajzolásához szükséges időtartam
- felfrissítési ütem: egy teljes kép egyszeri felrajzolásához szükséges idő. Képelőrefutási és képvisszafutási időre oszlik.

#### Az alfanumerikus display áramkörei

A display áramköreinek feladata, hogy az előzőekben vázolt folyamatban biztosítsa a monitor számára szükséges eltérítő (szinkron) jeleket és videojelet.

A display áramköröknek minden tv raszterrendszerű display-ben meglévő négy funkcionális egysége

- felfrissítő memória
- karaktergenerátor
- párhuzamos-soros átalakító
- vezérlőszámlánc

A felfrissítő memória tárolja a karakterkódokat. A karaktergenerátor tárolja a kódokhoz rendelt karakterek pontmátrixára vonatkozó információt és kimenetén párhuzamosan egy adott időben egy karakter egy rasztersorára vonatkozó bitkombináció jelenik meg. Ez azt jelenti, hogy a karaktergenerátornak, mint memóriának, címként szüksége van az aktuális karakterkódra, valamint arra az információra, hogy az elektronsugár a karaktersornak éppen melyik tv-sorát rajzolja.

A karaktergenerátor párhuzamos bitkombinációját sorba rendezve megkapjuk a tulajdonképpeni videojelet. Ezt a feladatot a párhuzamos-soros átalakító végzi el.

Az egész megjelenítési folyamathoz időzítő jeleket szolgáltat a vezérlő számlánc, melynek állapotai kölcsönösen egyértelműen megfelelnek az elektronsugár pillanatnyi helyzetének. A korábbiakban leírtak szerint a vezérlő számlánc öt részegységet tartalmaz:

<!-- pdf 168 -->

- Oszcillátor.
- Raszterpont számlánc: egy karakter egy tv-sorának mátrixpontjait számolja, beleértve a karakterek közötti szünetet is. Léptetőjelét általában a megjelenítő logika alaposzcillátora adja.
- Karakter számlánc: egy karaktersor egy tv-sorának karakterütemeit számolja, beleértve a sorvisszafutásra szánt karakterütemeket is. Léptetőjelét a raszterpont számlánc adja.
- Tv-sor számlánc: egy karaktersor tv-sorait számolja. Léptetőjele a karakter számlánctól származik.
- Karaktersor számlánc: egy tv-kép karaktersor-ütemeit számolja, beleértve a képvisszafutásra szánt karaktersor-ütemeket is. Léptetőjelét a tv-sor számlánc szolgáltatja.

A vezérlő számlánc a többi egység felé a következő szolgáltatásokat nyújtja:

- biztosítja a felfrissítő memória megfelelő sorrendben való kiolvasását
- karaktergenerátor számára jelzi a karaktersor éppen rajzolt tv-sorának számát (a tv-sor számlánc jelei)
- vezérli a párhuzamos-soros átalakítóba a karaktergenerátor jeleinek párhuzamos beléptetését, majd sorban történő kiléptetését
- a monitor számára a sor- és képszinkron jelet biztosítja.

A felfrissítő memória kiolvasásához az előzőekben ismertetett öt részegységen kívül egy további részegység tartozik a vezérlő számlánchoz abban az esetben, ha a felfrissítő memória RAM és a display nem tartalmaz sorpuffert. Ez a további részegység a memóriacímeket állítja <!-- pdf 169 --> elő, és tulajdonképpen egy speciális, programozható számlánc. Léptetőjele megegyezik a karakter számláncéval.

A számlánc az elektronsugár futásával szinkronban 0 értéktől kezdve annyi karakterütemet számol, ahány karakter található egy sorban, azután visszatér 0 értékre. Ezt a számlálási ciklust megismétli annyiszor, ahány tv-sorból áll egy karaktersor. A karaktersor utolsó tv-sorában nem tér vissza 0 értékre, hanem a következő karaktersor első karakterének címére lép. A tv-soronkénti ciklust most már a második karaktersor első és utolsó karakterének címei között hajtja végre.

A számláncnak olyan méretűnek kell lennie, hogy az előző bekezdésben leírt folyamatot a tv-kép összes karaktersorára le tudja folytatni.

#### A display áramkörök kapcsolata

Az elektronsugár futása közben a display áramkörökben a következő játszódik le.

Az elektronsugárnak a hasznos képernyőfelület bal felső sarkába érkezését megelőzően a felfrissítő memóriának a kép bal felső karakterpozíciójához tartozó rekesze lesz kiválasztva. A memóriakimeneten megjelenik az ebben a pozícióban megjeleníteni kívánt karakter kódja. Ez a kód a tv-sor számlánc jeleinek állapota által szolgáltatott, a karaktersor legfelső tv-sorát kijelölő információval együtt a karaktergenerátorra kerül. A karaktergenerátor kimenetén megjelenik a karakter legfelső mátrixsorának bitkombinációja. Abban az időpontban, amikor az elektronsugár eléri a képernyő bal felső mátrixpontját, a karaktergenerátor jeleinek állapota a párhuzamos-soros átalakítóba, a bal szélső pontnak megfelelő bit ezzel a videoerősítőre jut.

Az első karakterütem alatt az előzőekben belépett bitsorozat sorra a videojelre kerül. Közben a felfrissítő memória következő rekesze címződik, az innen kikerülő karakterkód a karaktergenerátorra jut és mire az első karakter utolsó raszterpontja is megjelenik a képernyőn.

<!-- pdf 170 -->

a második karakterre vonatkozó információ már a párhuzamos-soros átalakító bemenetén van és a második karakterütem kezdetén oda beíródik.

A folyamat ismétlődik növekvő memóriacímmel egészen a karaktersor utolsó karakterének megjelenítéséig. Sorvisszafutás alatt a felfrissítő memória címzésének csak annyiban van értelme, hogy elő kell készíteni a következő tv-sor első karakterének pontsorához tartozó információt. A következő tv-sorban a felfrissítő memória sorra ugyanazokat a karakterkódokat szolgáltatja, mint az első tv-sorban, azonban a karaktergenerátorra a tv-sor számlánc eggyel megnövelt értékű jele jut, így az a karakterek második rasztersorának megfelelő bitkombinációkat adja.

A felfrissítő memória mindaddig az első karaktersor karakterkódjait adja ki, amíg az elektronsugár a karaktersor összes tv-sorát fel nem rajzolta. A második karaktersor karakterkódjai csak azután következnek, és szintén annyiszor jelentkeznek, ahány tv-sorból áll a karaktersor.

Az összes karaktersor felrajzolása után kezdődik elölről a folyamat.

#### A cursor megjelenítése

Az alfanumerikus display-kben a valamilyen szempontból kijelölt karakterpozíciót (pl. a következő beírt karakter helyét) jelző képelemet nevezzük cursornak. A cursort meg lehet jeleníteni teljesen software úton (pl. egy adott szimbólum periodikus írásával-törlésével, vagy egyszerűen egy erre a célra fenntartott szimbólum beírásával), de a legtöbb display hardware elemeket is tartalmaz valamilyen cursor megjelenítésére. Ebben az esetben valamely tároló elemben található a cursornak a képernyőn elfoglalt helyét meghatározó információ. A tároló elem tartalma inkrementálható, dekrementálható, vagy tetszés szerinti új értékre változtatható.

<!-- pdf 171 -->

A tároló elem tartalmát a vezérlő számlánc állapotaival való összehasonlítás lehetővé teszi a cursor megjelenítéséhez szükséges videojeleknek a kialakításával.

#### Grafikus display-k

A tv raszterrendszerű grafikus display-k közül legnagyobb jelentősége az ún. bit-map display-nek van. Ebben a típusban a képernyő hasznos felülete egyetlen nagy pontmátrixot alkot. A felfrissítő memóriában a pontmátrix egyes elemeire vonatkozó információ található. Abban az esetben, ha a pontok két állapotot vehetnek fel (világít-nem világít), a képernyő minden egyes pontjához egy bit tartozik a felfrissítő memóriában. Amennyiben a pontokat több formában (pl. intenzitásváltás, villogtatás vagy színes megjelenítés) akarjuk megjeleníteni, az egy ponthoz rendelt bitek számát növelni kell.

A bit-map display felépítése az előzőekben ismertetett alfanumerikustól abban tér el, hogy hiányzik a karaktergenerátor, a felfrissítő memóriából kiolvasott adatok közvetlenül a párhuzamos-soros átalakítóra kerülnek. Természetesen a vezérlő számlánc is módosul, a karakter számlánc és a tv-sor számlánc helyett egyetlen olyan számlánc szerepel, amely a kép összes tv-sorát számolja.

#### A felfrissítő memória feltöltése

A korábbiakban feltételeztük, hogy a képtartalomnak megfelelő információ már a felfrissítő memóriában van és állandó. Nyilvánvaló azonban, hogy a felfrissítő memória tartalmát időről időre módosítani kell.

Az adatokat a felfrissítő memóriába vagy valamilyen célhardware juttatja, vagy a display mikroprocesszort tartalmaz és az adatok bejuttatása a felfrissítő memóriába a mikroprocesszor feladata.

Az adatbevitelen kívül az adatok kiolvasására is szükség lehet, pl. hard copy készítés vagy vonalra történő továbbítás, esetleg helyi feldolgozás céljából.

<!-- pdf 172 -->

Az előzőekből következik, hogy a felfrissítő memóriához – amely az esetek döntő többségében RAM – két irányból kell hozzáférni:

- a megjelenítés céljából a megjelenítő logika periodikusan kiolvassa a felfrissítő memóriát
- a mikroprocesszor (esetleg a célhardware) változtatni akarja a memóriatartalmat vagy olvasni akar a felfrissítő memóriából.

A kétirányú hozzáférés megszervezése minden display-tervezés során alapvető feladat. Több megoldás ismeretes, ezek közül a display jellegének megfelelő megoldás kiválasztása alapos mérlegelést igényel.

#### A szokásos megoldások:

- Csak alfanumerikus display-k esetében használható megoldás az, hogy egy shiftregiszterekből felépített recirkulációs tárat, ún. sorpuffert hoznak létre. A sorpuffer egy karaktersor karakterkódjainak tárolására képes. A RAM felfrissítő memóriából DMA, vagy más megoldás segítségével a sorpuffert minden karaktersor utolsó tv-sorában feltöltik a következő megjelenítendő karaktersor kódjaival. Ezek a kódok azután a karaktersor megjelenítése során karakterütemenként lépnek egyet és annyiszor járják körbe a sorpuffer rekeszeit, ahány tv-sorból áll a karaktersor. A karaktergenerátor számára a sorpuffer kimenete szolgáltatja a karakterkódot. A felfrissítő memória a sorpuffer töltési idejétől eltekintve korlátozás nélkül a mikroprocesszor rendelkezésére áll.
- A mikroprocesszor elsőbbséget élvezhet a memória-hozzáférés szempontjából. Ebben az esetben a megjelenítést a processzor memória-műveletei idejére le kell állítani. Ez a megoldás a processzor működését legkevésbé zavarja, de a képernyőn képminőség-romlást (villogást) eredményez azokban az időtartamokban, amelyekben a processzor gyakran fordul a felfrissítő memóriához. Dinamikus RAM felfrissítő memória esetén a megoldás csak korlátozva használható.

<!-- pdf 173 -->

- Az előző megoldást ki lehet egészíteni oly módon, hogy a processzor csak sor- és képvisszafutás idején fordul a felfrissítő memóriához.
   A sor- és képvisszafutás idejéről IT kérés tájékoztatja a processzort. Ez a megoldás különösen jó felbontású grafikus display esetében jelentősen lelassítja a processzor működését és így a képszerkesztést.
- Megoldás lehet az is, hogy a CPU-hoz WAIT kérés érkezik mindazon esetben, amikor a felfrissítő memóriához fordul és azt épp a megjelenítő logika használja. Ez a megoldás is lassítja a programfutást.
- Abban az esetben, ha a megjelenítő logika memória-hozzáférései közötti szünetben lehetőség nyílik a processzor igényeinek kielégítésére.

A TVC a legutóbbi megoldást alkalmazza.

#### Fényceruza használata

A fényceruza olyan eszköz, amely a képernyő egy adott pontjára helyezve képes érzékelni, hogy az elektronsugár az adott pontot éppen eltalálja és erről a tényről jelzést képes adni a display mikroprocesszora felé. A processzor kiértékelheti az elektronsugárnak a jelzéshez tartozó pozícióját és a felfrissítő memóriának a pozícióhoz rendelt rekeszébe tetszőleges információt tölthet. Ily módon a fényceruzát a képernyőn mozgatva képmódosítást hozhatunk létre.

<!-- pdf 174 -->

### 6.2.2 A 6845-ÖS CRTC-VEL KAPCSOLATOS ISMERETEK

#### A CRTC általános ismertetése

A 6845-ös CRTC (a továbbiakban: CRTC) egy nagyintegráltságú áramkör, amely a legkülönbözőbb CRT display-k építésére alkalmas a nagyon egyszerű alfanumerikus display-től a nagyfelbontású bit-map grafikus display-ig.

Felépítése, működési feltételei miatt egy mikroprocesszoros rendszerbe kell illeszteni. A mikroprocesszor a képalkotás paramétereit széles skálán tudja változtatni. Fel kell azonban a figyelmet hívni arra, hogy egy adott display-be építve a programozási lehetőségek szűkülnek, mivel egyrészt a display egyéb áramköreinek, másrészt a monitornak a paraméterei nem tesznek lehetővé tetszőleges képszervezést. Többek között például a monitor sorkimenő transzformátorának átviteli sávjába kell, hogy essen a programozott vízszintes szinkronjel frekvenciája.

A CRTC a tv raszterrendszerű display előzőekben ismertetett funkcionális egységei közül a következőket tartalmazza:

- karakter számlánc, tv-sor számlánc, karaktersor számlánc
- memóriacímző számlánc
- cursor áramkör
- fényceruza-érzékelő áramkör

#### A CRTC interface jelei

A CRTC és CPU és a megjelenítő logikai áramkörök között helyezkedik el. A CPU-hoz 13, a megjelenítő logikához 25 jellel kapcsolódik. A logikai földdel és az 5 V-os táppal együtt a CRTC-nek 40 kivezetése van.

<!-- pdf 175 -->

A kivezetések ismertetésénél az első sorban a következő adatokat közöljük pontosvesszővel elválasztva:

Megnevezés; jelnév; kivezetés száma; a jel iránya;

#### A CPU-hoz kapcsolódó jelek

- Kétirányú adatbusz; D0-D7; 33.-26. kivezetések; ki- és bemenet: A CPU és a CRTC közötti adatátvitelre szolgál. Outputként háromállapotú és nagyimpedanciás állapotban van, kivéve azt az esetet, amikor a CPU a CRTC-ből olvas.
- Read/Write; R/W; 22. láb; bemenet: A CPU és a CRTC közötti adatforgalom irányát jelzi. Az R/W jel H szintje a CRTC-től a CPU-hoz való adattovábbítást, L szintje fordított irányt jelez.
- Chip select; CS; 25. láb; bemenet: L szintje engedélyezi az adatforgalmat a CPU és CRTC között. Általában a CPU I/O címének dekódolásával előálló jel vezérli.
- Regiszter Select; RS; 24. láb; bemenet: A CRTC a CPU I/O címeiből kettőt foglal el, a két cím közül az RS választ. Általában a CPU címbusz legalacsonyabb helyi értékű bitje kerül erre a bemenetre. L állapotában a CRTC belső címregiszterét éri el a CPU, H állapotában a címregiszter által megcímzett belső regisztert. 18 ilyen belső vezérlő regisztere van a CRTC-nek.
- Enable; E; 23. láb; bemenet: Adatérvényesítő (STROBE) funkciót tölt be a CPU által kezdeményezett, a CRTC belső regisztereivel kapcsolatos R/W műveleteknél.
- Reset; RES; 2. láb; bemenet: A CRTC alapállapotba állását kiváltó jel. L szintje a CRTC-t a következő állapotba állítja:

<!-- pdf 176 -->

- a CRTC összes számlálója törlődik, és a CRTC leállítja a displayműveleteket
- minden kimenet L állapotba áll
- a vezérlő regiszterek állapotát a RES jel változatlanul hagyja
- A RES jel hatásának még a következő jellegzetességei vannak:
- a RES jel csak akkor hatásos, ha az LPSTB (fényceruza STROBE) jel L szintű
- a CRTC a display-műveleteket közvetlenül a RES jel H szintbe állása után indítja
- RES L állapotba állása után az MA0-MA13 és RA0-RA4 jelek a CLK jellel szinkronizálva állnak L állapotba, így legalább egy CLK ciklus ideig kell a RES jel L szintű impulzusának tartani.

#### A megjelenítő logikához kapcsolódó jelek

- Karakter órajel; CLK; 21. láb; bemenet:
  - A display oszcillátorának jelét a raszterpont számlánc osztja, számolva egy karakter egy tv-sorának raszterpontjait. A raszterpont számláncot CRTC-n kívül kell létrehozni. A CLK jel a raszterpont számlánc egy számolási ciklusával megegyező ütemidejű (karakterütem) jel.
- Vízszintes szinkronjel; HSYNC; 39. láb; kimenet:
   Aktív H állapotú jel a monitor vízszintes eltérítő áramköreinek vezérléséhez.
- Függőleges szinkronjel; VSYNC; 40. láb; kimenet:
   Aktív H állapotú jel a monitor függőleges eltérítő áramköreinek vezérléséhez.
- Megjelenítés időzítés; DISPTMG; 18. láb; kimenet: A vízszintes és függőleges előre- és visszafutásra szánt idők meghatározását végző jel. A videojel engedélyezésére-tiltására használható. H állapotában szabad aktív videojelet engedni a monitorra.

<!-- pdf 177 -->

- Felfrissítő memóriacím; MA0-MA13; 4.-17. láb; kimenetek:
   A felfrissítő memóriacímeket adják a jelek a megjelenítési célból történő kiolvasáshoz. A 6.2.1 pontban - az alfanumerikus display-k áramkörei - ismertetett módon, periodikusan címzik a memóriát. 16 kszó nagyságú felfrissítő memória használatát teszi lehetővé.
- Rasztercímek; RA0-RA4; 38.-34. láb; kimenetek:
   A tv-sor számlánc jelei. A karaktergenerátor számára jelzik, hogy az elektronsugár az adott karaktersor melyik tv-sorát rajzolja.
- Cursormegjelenítés; CUDISP; 19. láb; kimenet: Aktív H szintű jel, amely a párhuzamos-soros átalakító kimenőjeléhez kapcsolva a cursor megjelenítést végzi. A DISPTMG jel L szintje idejére a CUDISP jel H szintű impulzusai tiltva vannak.
  - A TVC-nél nem cursor megjelenítésére, hanem IT kérésre használjuk.
- Fényceruza STROBE; LPSTB; 3. láb; bemenet:

  Aktív H szintű jel, amely a fényceruzától érkezik. Impulzusa hatására a felfrissítő memóriacímek (MA0-MA13) tárolódnak a 14 bites fényceruza regiszterben. E regiszter állapotát beolvasva kap információt a processzor a fényceruzának a képernyőn elfoglalt helyzetéről. A regiszterben tárolt felfrissítő memóriacímet software úton korrigálni kell, mivel a megjelenítő áramkörök, a monitor, a fényceruza áramkörök késleltetése összeadódik, így a tárolt érték és a fényceruza pozíciójához tartozó valódi felfrissítő memóriacím között jelentős, de egy adott display-nél állandó eltérés van.
  - A TVC bővítő csatlakozóira ki van vezetve a CRTC ezen kivezetése. Használatához kérje ki a VIDEOTON szerviz tanácsát!

<!-- pdf 178 -->

#### A CRTC állapotregiszterei

A CPU a CRTC-t I/O eszközként kezeli. Két I/O címet igényel a CRTC kezelése. Az egyik címen a CRTC 5 bites címregisztere érhető el. A CRTC 18 belső vezérlő regisztere közül a címregiszter tartalma által kijelölt regiszterrel teremthet kapcsolatot a CPU.

A 18 regiszter használatával a display képjellemzői nagyon széles tartományban változtathatók. Meg kell jegyezni azonban, hogy valamely készülékbe épített CRTC vezérlő regisztereinek beállításakor ügyelni kell a készülék egyéb áramkörei által meghatározott feltételekre is.

A bitkiosztásnál az X jel közömbös értéket jelöl.

#### Címregiszter (AR)

Ez a regiszter a 18 belső regiszter (R0-R17) közül egy címét tartalmazza, bináris értékkel. 17-nél nagyobb értéket írva a címregiszterbe a CPU egyik regisztert sem érheti el. Bármelyik vezérlő regisztert (RO-R17) a CPU úgy érheti el, hogy először a címregiszterbe írja az elérni kívánt regiszter címét (a CRTC CS és RS bemeneteinek L értékbe állításával), majd a CS bemenet L és az RS bemenet H értékénél elvégzi a kívánt írást vagy olvasást.

Programozási feltételek:

RS = 0 ; Csak írható regiszter

Bitkiosztás

```
B7 B6 B5 B4 B3 B2 B1 B0
X  X  X  | regisztercím |
```

#### A teljes tv-sor karaktereinek regisztere (R0)

A regiszter tartalma meghatározza az egy tv-soron belüli (sor előre- és visszafutást is beleértve) karakterütemek számát.

<!-- pdf 179 -->

Figyelem: ha a karakterütemek számát M értékre akarjuk állítani, M-1 értéket kell a regiszterbe írni.

Programozási feltételek:

RS = 1 ; Címregiszter : 00000 ; Csak írható regiszter

Bitkiosztás

```
B7 B6 B5 B4 B3 B2 B1 B0
|Karakterszám a tv-sorban - 1|
```

A programozás egysége: karakterütem.

#### A tv-sor megjelenített karaktereinek regisztere (R1)

A sorelőrefutás karakterütemeinek, azaz az egy sorban lévő karaktereknek a számát határozza meg e regiszter tartalma. R1 tartalmának értéke kisebb kell, hogy legyen az R0 regiszter értékénél.

Programozási feltételek:

RS = 1 ; Címregiszter : 00001 ; Csak írható regiszter

Bitkiosztás:

```
B7 B6 B5 B4 B3 B2 B1 B0
|Megjelenített karakterszám a tv-sorban|
```

A programozás egysége: karakterütem.

#### A vízszintes szinkronjel pozíciójának regisztere (R2)

A sorelőrefutás kezdetétől számolva megadja annak a karakterütemnek a számát, amelynek tartama alatt a vízszintes szinkronjel (sorvégjel) H szintbe áll. A kívánt értéknél eggyel kevesebbet kell a regiszterbe írni. Ha a regiszterbe nagyobb számot töltünk, a kép a képernyőn <!-- pdf 180 --> balra, ha kisebbet, akkor jobbra tolódik, ily módon a kívánt vízszintes képhelyzet beállítható. Tartalmát kisebbre kell állítani, mint R0.

Programozási feltételek:

RS = 1 ; Címregiszter : 00010 ; Csak írható regiszter

Bitkiosztás

```
B7 B6 B5 B4 B3 B2 B1 B0
|Sorvégjel kezdő karakterpozíciója - 1|
```

A programozás egysége: karakterütem.

#### Szinkronjelszélesség regisztere (R3)

A vízszintes- és függőleges szinkronjel szélességét lehet beállítani e regiszter feltöltésével. A 8 bites regiszter alacsonyabb helyi értékű 4 bitje a sorvégjel szélességét határozza meg karakterütemben számolva, binárisan. 0 érték programozása tilos! A magasabb helyi értékű 4 bit a képvégjel (függőleges szinkron) szélességét adja, tv-sorütemben számolva, binárisan. 0 értéket programozva a képvégjel 16 tv-sorütem széles lesz.

Programozási feltételek:

RS = 1 ; Címregiszter : 00011 ; Csak írható regiszter

Bitkiosztás

```
B7 B6 B5 B4 B3 B2 B1 B0
V3 V2 V1 V0 H3 H2 H1 H0
```

<!-- pdf 181 -->

H0-H3: sorvégjel szélességének beállítása:

| H3 | H2 | H1 | H0 | Sorvégjel szélessége |
|----|----|----|----|----------------------|
| 0  | 0  | 0  | 0  | Tilos érték          |
| 0  | 0  | 0  | 1  | 1 karakterütem       |
| 0  | 0  | 1  | 0  | 2 karakterütem       |
| 0  | 0  | 1  | 1  | 3 karakterütem       |
| 0  | 1  | 0  | 0  | 4 karakterütem       |
| 0  | 1  | 0  | 1  | 5 karakterütem       |
| 0  | 1  | 1  | 0  | 6 karakterütem       |
| 0  | 1  | 1  | 1  | 7 karakterütem       |
| 1  | 0  | 0  | 0  | 8 karakterütem       |
| 1  | 0  | 0  | 1  | 9 karakterütem       |
| 1  | 0  | 1  | 0  | 10 karakterütem      |
| 1  | 0  | 1  | 1  | 11 karakterütem      |
| 1  | 1  | 0  | 0  | 12 karakterütem      |
| 1  | 1  | 0  | 1  | 13 karakterütem      |
| 1  | 1  | 1  | 0  | 14 karakterütem      |
| 1  | 1  | 1  | 1  | 15 karakterütem      |

<!-- pdf 182 -->

V0-V3: képvégjel szélességének beállítása:

| V3 | V2 | V1  | V0 | Képvégjel szélessége |
|----|----|-----|----|----------------------|
| 0  | 0  | 0   | 0  | 16 tv-sorütem        |
| 0  | 0  | 0   | 1  | 1 tv-sorütem         |
| 0  | 0  | 1   | 0  | 2 tv-sorütem         |
| 0  | 0  | 1   | 1  | 3 tv-sorütem         |
| 0  | 1  | 0   | 0  | 4 tv-sorütem         |
| 0  | 1  | 0   | 1  | 5 tv-sorütem         |
| 0  | 1  | 1   | 0  | 6 tv-sorütem         |
| 0  | 1  | 1  | 1  | 7 tv-sorütem         |
| 1  | 0  | 0   | 0  | 8 tv-sorütem         |
| 1  | 0  | 0   | 1  | 9 tv-sorütem         |
| 1  | 0  | 1   | 0  | 10 tv-sorütem        |
| 1  | 0  | 1   | 1  | 11 tv-sorütem        |
| 1  | 1  | 0   | 0  | 12 tv-sorütem        |
| 1  | 1  | 0   | 1  | 13 tv-sorütem        |
| 1  | 1  | 1   | 0  | 14 tv-sorütem        |
| 1  | 1  | 1   | 1  | 15 tv-sorütem        |

#### A teljes kép karaktersorainak regisztere (R4)

A képelőrefutás és képvisszafutás karaktersorainak összegét határozza meg ez a regiszter. A kívánt értéknél eggyel kevesebbet kell a regiszterbe tölteni.

Programozási feltételek:

RS = 1 ; Címregiszter : 00100 ; Csak írható regiszter

<!-- pdf 183 -->

Bitkiosztás

```
B7 B6 B5 B4 B3 B2 B1 B0
X  | A tv-kép karaktersorainak |
   | száma - 1                 |
```

Programozási egység: karaktersorütem.

#### Függőleges kiegészítő regiszter (R5)

A teljes tv-kép tv-sorainak meghatározása az R4 regiszter segítségével csak akkora egységben lehetséges, amennyi tv-sorból áll egy karaktersor. Az R5 regiszter tetszőleges tv-sorszám beállítását teszi lehetővé. A teljes tv-kép tv-sorainak a számát ezek alapján a következőképpen lehet meghatározni:

(R4 tartalma + 1) X tv-sorok száma egy karaktersorban + R5 tartalma.

Programozási feltételek:

RS = 1 ; Címregiszter : 00101 ; Csak írható regiszter

Bitkiosztás

```
B7 B6 B5 B4 B3 B2 B1 B0
X  X  X  | Kiegészítő tv-sorok száma |
```

Programozási egység: tv-sorütem.

#### Megjelenített karaktersorok regisztere (R6)

A hasznos kép (sorelőrefutás) karaktersorainak számát határozza meg. Értéke mindig kisebb kell, hogy legyen, mint R4-é.

Programozási feltételek:

RS = 1 ; Címregiszter: 00110 ; Csak írható regiszter

Bitkiosztás

```
B7 B6 B5 B4 B3 B2 B1 B0
X  | Megjelenített karaktersorok száma |
```

Programozási egység: karaktersorütem

<!-- pdf 184 -->

#### A függőleges szinkronjel pozíciójának regisztere (R7)

A képelőrefutás kezdetétől számítva kijelöli azt a karaktersorütemet, amelynek fennállása alatt a függőleges szinkronjel (képvégjel) H szintbe áll. A regiszter 7 bites, tartalma kisebb kell hogy legyen, mint R4-é. A regiszter tartalmát növelve a kép a képernyőn felfelé, a tartalmat csökkentve lefelé csúszik.

A regiszterbe a kívánt pozíciónál eggyel kisebb értéket kell írni.

Programozási feltételek:

RS = 1 ; Címregiszter: 00111 ; Csak írható regiszter

Bitkiosztás

```
B7 B6 B5 B4 B3 B2 B1 B0
X  | Képvégjel kezdő karakterpozíciója - 1 |
```

A programozás egysége: karaktersorütem

#### Üzemmód- és időzítő regiszter (R8)

A két legalacsonyabb helyi értékű bit meghatározza a megjelenítés üzemmódját:

| V | S | Magyarázat                    |
|---|---|-------------------------------|
| 0 | 0 | Nem váltott soros letapogatás |
| 1 | 0 | Nem váltott soros letapogatás |
| 0 | 1 | Váltott soros letapogatás I. |
| 1 | 1 | Váltott soros letapogatás II. |

<!-- pdf 185 -->

A három megjelenítési mód értelmezéséhez különböztessük meg az egymás után következő felfrissítési ciklusokat, páros és páratlan sorszámú felfrissítési ciklusokat definiáljunk.

Nem váltottsoros letapogatás esetén a páros és páratlan sorszámú felfrissítésű ciklusok teljesen hasonlóak, az elektronsugár ugyanazokat a képernyőpontokat találja el. A legtöbb display ilyen módon működik, ezért a display működéséről korábban elmondottakat szorosan véve az ilyen display-kre érvényesek.

A váltottsoros letapogatás I. típusának nevezett megjelenítési mód esetén a páros és páratlan sorszámú felfrissítési ciklusok lerajzolása során a videojel-állapot a ciklusok megfelelő állapotaiban azonos, az elektronsugár a két egymás utáni ciklusban azonos képet rajzol, azonban a két kép függőleges irányban egymáshoz képest el van csúsztatva, oly módon, hogy a páros sorszámú ciklusokban a tv-sorok a páratlan sorszámú ciklusok tv-sorai közé kerülnek. Ennek az üzemmódnak az az előnye a nem váltottsoros megjelenítéshez képest, hogy mivel a képpontok közelebb kerülnek egymáshoz, a függőleges vonalak folytonosnak látszanak, így a kép jobb minőségű. Ugyanilyen képminőség eléréséhez nem váltottsoros letapogatás esetén meg kellene duplázni a felfrissítési ciklushoz tartozó tv-sorok számát, amely vagy a felfrissítési ciklus idejét növelné meg, vagy a monitor és a display áramköreitől kívánna meg sokkal gyorsabb működést (magasabb videofrekvencia és sorfrekvencia). A felfrissítési ciklusidő növelésével a képrajzolás láthatóvá válna, a kép villogna.

A váltottsoros letapogatás II. típusúnak nevezett megjelenítési mód esetén a páros és páratlan sorszámú felfrissítési ciklusokban a tv-sorok elhelyezkedése az előző bekezdésben leírtakkal megegyezik, de a páros és páratlan felfrissítési ciklusban rajzolt kép eltér. Ez az üzemmód felel meg a televízió készülékek képalkotásának. Mivel a két felfrissítési ciklusban eltérő vezérlést kaphat a videoerősítő, ezért úgy is felfoghatjuk, hogy a felhasználható tv-sorok száma megduplázódott. Ezt fel lehet használni vagy a felbontás növelésére (a karakterek pontmátrixának vízszintes irányban való növelésére), így <!-- pdf 186 --> finomabb karakterformák kialakítására vagy pl. a karaktersorok számának növelésére. Egy teljes kép felrajzolása most két felfrissítési ciklus alatt történik, azaz a képrajzolás tényleges ideje megnőtt, ennek ellenére a képen a villogás nem vehető észre, mert az egymás után következő sorok megadása történik nagy időkülönbséggel.

A CRTC a három üzemmódot a következőképpen jelzi a display logika egyéb áramköreinek:

- A VSYNC jel váltottsoros letapogatás esetén a páratlan sorszámú felfrissítési ciklusokban a tv-sor számlánc állapotváltásával egyidejűleg vált állapotot, a páros sorszámú felfrissítési ciklusokban a tvsor számlánc két állapotváltása között, egyenlő távolságra a két állapotváltástól. Nem váltottsoros letapogatás esetén a VSYNC állapotváltásai mindig egybeesnek a tv-sor számlánc állapotváltásaival.
- A tv-sor számlánc jelei (RA0-RA4) nem váltottsoros letapogatás és váltottsoros letapogatás I. üzemmódokban a páros és páratlan sorszámú felfrissítési ciklusokban egyaránt 0-tól egyesével változtatják állapotaikat az R9 regiszter által meghatározott értékig.

Váltottsoros letapogatás II. üzemmódban a tv-sor számlánc jelei kettesével változtatják állapotaikat. Az egyik felfrissítési ciklusban egy adott karaktersor páros sorszámú tv-sorait, a következő felfrissítési ciklusban a karaktersor páratlan sorszámú tv-sorait rajzolja az elektronsugár.

Abban az esetben, ha a karaktersorok páros számú tv-sorokból állnak, akkor a páros sorszámú felfrissítési ciklusokban az összes karaktersor páros sorszámú tv-sorai kerülnek rajzolásra, a páratlan sorszámú felfrissítési ciklusokban pedig az összes karaktersor páratlan sorszámú tv-sorai kerülnek rajzolásra.

Abban az esetben, ha a karaktersor páratlan számú tv-sorból áll, akkor a páros sorszámú felfrissítési ciklusban a páros sorszámú karaktersoroknak a páros sorszámú tv-sorai, a páratlan sorszámú <!-- pdf 187 --> karaktersoroknak a páratlan sorszámú tv-sorai kerülnek a képernyőre. A páratlan sorszámú felfrissítési ciklusokban a páros sorszámú karaktersorok páratlan sorszámú tv-sorai, a páratlan sorszámú karaktersoroknak a páros sorszámú tv-sorai kerülnek a rajzolásban sorra, ahogy az alábbi táblázat mutatja:

<table>
<tr><th rowspan="2">A karakter összes tv-sorainak száma</th><th rowspan="2">A karaktersor sorszáma</th><th colspan="2">Felfrissítési ciklus, ha a rajzolt tv-sor sorszáma</th></tr>
<tr><th>páros</th><th>páratlan</th></tr>
<tr><td>Páros</td><td>Páros</td><td>Páros</td><td>Páratlan</td></tr>
<tr><td>Páros</td><td>Páratlan</td><td>Páros</td><td>Páratlan</td></tr>
<tr><td>Páratlan</td><td>Páros</td><td>Páros</td><td>Páratlan</td></tr>
<tr><td>Páratlan</td><td>Páratlan</td><td>Páratlan</td><td>Páros</td></tr>
</table>

A 4. és 5. bit a regiszterben a DISPTMG jel időzítését végzi:

| D1 | D0 |  |
|----|----|--|
| 0 | 0 | Nincs késleltetés |
| 0 | 1 | 1 karakterütem késleltetés |
| 1 | 0 | 2 karakterütem késleltetés |
| 1 | 1 | A jel nem jelenik meg |

A 6. és 7. bit a regiszterben a CUDISP jel időzítését végzi.

| C1 | C0 |  |
|----|----|--|
| 0 | 0 | Nincs késleltetés |
| 0 | 1 | 1 karakterütem késleltetés |
| 1 | 0 | 2 karakterütem késleltetés |
| 1 | 1 | A jel nem jelenik meg |

Az időzítési lehetőségnek az a jelentősége, hogy a jelek fázisát a megjelenítés fázishelyzetével összhangba hozza. Késleltetés nélkül a jelek állapotváltozásai megfelelnek a felfrissítő memória állapotai<!-- pdf 188 -->nak: pl. ha a cursor a képernyő bal felső sarkában lévő karakterpozíciót jelöli, akkor a CUDISP jel impulzusa a felfrissítő memóriának 0 állapotával egyidejűleg jelenik meg és a DISPTMG jel is ezzel egyidőben engedélyezi a megjelenítést, holott ezután még ki kell olvasni a felfrissítő memóriát, majd annak tartalmával a karaktergenerátort. Ez a két memóriaolvasás a display felépítésétől függően egy vagy két karakterütem alatt megy végbe, ennek megfelelően a CUDISP és DISPTMG jelek állapotváltozásai is csak ilyen késleltetéssel következhetnek be.

Programozási feltételek:

RS = 1 ; Címregiszter : 01000 ; Csak írható regiszter

Bitkiosztás

```
B7 B6 B5 B4 B3 B2 B1 B0
C1 C0 D1 D0 X  X  V  S
```

#### A karaktersor tv-sorainak regisztere (R9)

A karaktersor tv-sorainak számát lehet a regiszter beállításával meghatározni, beleértve a karaktersorok közötti szüneteket is.

A regiszterbe töltendő érték adott tv-sorszám esetén függ az üzemmódtól is.

- Nem váltottsoros letapogatás esetén a kívánt tv-sorok számánál 1-gyel kevesebbet kell tölteni.
- Váltottsoros letapogatás I. üzemmód esetén a karaktersor egy felfrissítési ciklusban levő tv-sorai számánál 1-gyel kevesebbet kell tölteni. Ebben az esetben a karaktersorhoz összesen tartozó tv-sorszám: 2x (betöltött érték + 1).

<!-- pdf 189 -->

Váltottsoros letapogatás II. üzemmód esetén a kívánt tv-sorszámnál 2-vel kisebb értéket kell tölteni. A kívánt tv-sorszám a páros és páratlan felfrissítési ciklusban együttesen alakul ki.

Programozási feltételek:

RS = 1 ; Címregiszter : 01001 ; Csak írható regiszter

Bitkiosztás

```
B7 B6 B5 B4 B3 B2 B1 B0
X  X  X  | Karaktersor tv-sorainak |
         | száma - 1 vagy 2        |
```

Programozási egység: tv-sorütem.

#### A cursor kezdő tv-sorának regisztere (R10)

CRTC a cursort meghatározott tv-sorból álló villogó, vagy nem villogó kis téglalap formájában jeleníti meg.

A regiszter 5. és 6. bitje a cursortípust jelöli ki:

| B | P |  |
|---|---|--|
| 0 | 0 | Nem villogó cursor |
| 0 | 1 | A cursor nincs megjelenítve |
| 1 | 0 | 16 felfrissítési ciklusidővel villogó cursor |
| 1 | 1 | 32 felfrissítési ciklusidővel villogó cursor |

A regiszter alacsonyabb helyi értékű 5 bitje a karaktersoron belül kijelöli azt a tv-sort, amely a cursor téglalapjának legfelső sora.

Programozási feltételek:

RS = 1 ; Címregiszter : 01010 ; Csak írható regiszter

<!-- pdf 190 -->

Bitkiosztás

```
B7 B6 B5 B4 B3 B2 B1 B0
X  B  P  | Cursor kezdő tv-sorának |
         | száma                   |
```

Programozási egység: tv-sorütem.

#### A cursor utolsó sorának regisztere (R11)

Tartalma kijelöli a karaktersoron belül a cursor téglalapjának utolsó tv-sorát.

Programozási feltételek:

RS = 1 ; Címregiszter : 01011 ; Csak írható regiszter

Bitkiosztás

```
B7 B6 B5 B4 B3 B2 B1 B0
X  X  X  | Cursor utolsó tv- |
         | sorának száma     |
```

Programozás egysége: tv-sorütem.

#### Kezdő memóriacím-regiszter (R12, R13)

A képernyő kezdő karakterpozíciójához tartozó memóriacímet jelöli ki. A beállításával könnyen lehet SCROLL funkciót megvalósítani. Az R12 regiszter magasabb helyi értékű két bitjének állapota közömbös, ill. beolvasáskor 0 értéket a CPU számára.

Programozási feltételek:

R12 regiszter: RS = 1 ; Címregiszter : 01100 ; írható, olvasható

<!-- pdf 191 -->

Bitkiosztás:

```
      B7 B6 B5 B4 B3 B2 B1 B0
írás:    X  X  | Memória kezdőcím, magasabb |
olvasás: 0  0  | helyi értékű bitek       |
```

R13 regiszter: RS = 1 ; Címregiszter : 01101 ; írható, olvasható

Bitkiosztás:

```
B7 B6 B5 B4 B3 B2 B1 B0
| Memória kezdőcím alacsonyabb |
| helyi értékű bitek           |
```

#### Cursorregiszter (R14, R15)

A cursor aktuális karakterpozícióját tárolja. Az R14 regiszter magasabb helyi értékű két bitjének állapota íráskor közömbös, olvasáskor 0 értéket ad.

Programozási feltételek:

R14 regiszter: RS = 1 ; Címregiszter : 01110 ; írható, olvasható

Bitkiosztás:

```
      B7 B6 B5 B4 B3 B2 B1 B0
írás:    X  X  | Cursorcím magasabb helyi |
olvasás: 0  0  | értékű bit               |
```

R15 regiszter: RS = 1 ; Címregiszter : 01111 ; írható, olvasható

Bitkiosztás:

```
B7 B6 B5 B4 B3 B2 B1 B0
| Cursorcím alacsonyabb helyi értékű |
| bitek                              |
```

#### Fényceruza-regiszter (R16, R17)

Az LPSTB impulzus hatására a CRTC ebbe a regiszterbe írja át a felfrissítő memória aktuális címét. Az R16 regiszter magasabb helyi értékű két bitje mindig 0 értékű.

<!-- pdf 192 -->

A regiszter értékét a software-nek korrigálnia kell a betöltési folyamatban lévő késleltetések miatt.

Programozási feltételek:

R16 regiszter: RS = 1 ; Címregiszter : 10000 ; Csak olvasható reg.

Bitkiosztás:

```
B7 B6 B5 B4 B3 B2 B1 B0
0  0  | Fényceruza-pozíció magasabb |
      | helyi értékű bitek          |
```

R17 regiszter: RS = 1 ; Címregiszter : 10001 ; Csak olvasható reg.

Bitkiosztás:

```
B7 B6 B5 B4 B3 B2 B1 B0
| Fényceruza-pozíció, alacsonyabb helyi |
| értékű bitek                          |
```

#### A 6845-ös mint grafikus CRTC

Mint az előzőektől látható, a 6845-ös CRTC elsősorban alfanumerikus display-kben való felhasználásra készült. Bit-map display-k építésénél is használható oly módon, hogy a felfrissítő memória címzésére az MA jeleken kívül az RA jeleket is használjuk.

Az RA jelek használatára nem lenne szükség, ha a karaktersorok számát kellően nagyra lehetne választani (R6 regiszter). Ebben az esetben a karaktersor tv-sorait 1-re állítanánk be (R9 regiszter), ekkor az MA jelek folyamatosan címeznék a felfrissítő memória rekeszeit, sorra kiolvasva a képpontokra vonatkozó információt.

Mivel a karaktersorok száma korlátozott, a karaktersorok tv-sorainak számát nagyobbra (4-re vagy 8-ra) választjuk és a memóriacímzésére a tv-sor számlánc RA jeleit is használjuk.

Példaként tekintsük meg egy 512x240 pontmátrixú bit-map display memóriacímzését (TVC).

<!-- pdf 193 -->

A felfrissítő memória byte-onként érhető el. Egy tv-sorban ezek szerint 64 byte-ot kell kiolvasni. A 64 byte kiolvasásához 6 bit információra van szükség, ezért a felfrissítő memória címének alsó hat helyi értékű bitjét az MA0-MA5 jelek adják.

Az R9 regiszterbe (a karaktersor tv-sorainak száma) 4 tv-soros karaktersornak megfelelő információt írunk és a felfrissítő memória következő két címbitjét az RA0-RA1 jelekkel vezéreljük.

Az R6 regiszterbe (a megjelenített karaktersorok száma) 60 karaktersornak megfelelő információt írunk és a felfrissítő memória következő hat címbitjét az MA6-MA11 jelekkel vezéreljük.

A CRTC jelek és a felfrissítő memóriacímek összerendelése összefoglalva:

| CRTC jelek | Memóriacímek |
|-------------|--------------|
| MA0 - MA5 | ma0 - ma5 |
| RA0 - RA1 | ma6 - ma7 |
| MA6 - MA11 | ma8 - ma13 |

Hasonló módon tetszőleges pontmátrixú grafikus display-t állíthatunk össze.

Abban az esetben, ha ez egy tv-sorban lévő byte-ok száma nem 2 hatványa, akkor a címzés nem folytonos, tehát kihasználatlan memóriaterületek keletkeznek.

<!-- pdf 194 -->

# 7. A TÁPEGYSÉG

A TV-Computer tápegysége kisméretű, nagyteljesítményű áramkör, amely a TV-Computer alapegységétől különálló műanyag dobozban lett elhelyezve.

A relatíve nagy teljesítményigény, a kedvező hatásfok és kis disszipáció miatt a tápegység impulzusüzemű, ún. záróüzemű teljesítmény stabilizátor.

## 7.1 A tápegység működési elve

A kapcsolóüzemű tápegység váltakozóáramot konvertál egyenárammá (AC/DC konverter).

Ennek érdekében a bemeneti (hálózati) feszültséget egyenirányítjuk és egy kapcsoló tranzisztorral periodikusan a terhelésre kapcsoljuk.

A kimeneti transzformátor és szűrő úgy működik, mint egy DC/DC konverter. Egy összehasonlító áramkör állandóan figyeli ennek a kimenetét és növeli a kapcsolási ciklus munkaütemét, ha a kimeneti feszültség csökken és csökkenti a munkaütem szélességét, ha a kimeneti feszültség növekszik.

Ellentétben a szokásos soros, áteresztő tranzisztoros szabályozókkal, amelyek jelentős teljesítményt disszipálnak, itt csupán a veszteségekből és a kapcsolási folyamat véges sebességéből adódó melegedéssel kell számolnunk.

A kapcsolási frekvencia az ultrahang-frekvenciás tartományban van, <!-- pdf 195 --> ezért az egész tápegység a kisméretű transzformátor és hűtőfelület miatt szintén kisméretű.

A tápegység által előállított feszültségek:

- +5 V
- +12 V
- -12 V

## 7.2 A tápegység felépítése

A tápegység áramkörei, a TDA 1060 integrált kapcsolóüzemű szabályozó és védőáramkör köré épülnek (MULLARD-PHILIPS, VALVO SIGNETICS, ill. NDK gyártmányú megfelelője a B260D típusú integrált áramkör).

### 7.2.1 A TÁPEGYSÉG VEZÉRLŐ- ÉS VÉDŐÁRAMKÖRE

A tápegység vezérlőáramköre a következő áramköri részeket tartalmazza:

- a kapcsoló üzemmódot vezérlő stabil multivibrátor
- a kitöltési tényező lineáris változtatására alkalmas impulzusszélesség modulátor
- stabil, kompenzált referencia feszültséget előállító áramkör, amely lehetővé teszi a tápegység kimeneti feszültségével való összehasonlítást
- a vezérlő hurok stabilitását biztosító hibafeszültség erősítő
- a bemeneti feszültség változásait kompenzáló előrecsatoló áramkör

<!-- pdf 196 -->

##### Védelmi funkciók

A fenti vezérlési funkciókon kívül a tápegységben több, védelmi funkciót betöltő elem is megtalálható, amelyek a következő feladatokat oldják meg:

- maximális kitöltési tényező határolás
- a kapcsoló tranzisztor bekapcsolási áramát korlátozó ún. lágyindítás
- túlfeszültség és túlterhelés védelem
- inkorrekt működés elleni védelem, amely a vezérlőáramkör túl alacsony tápfeszültsége esetén hatásos
- védelem a visszacsatoló hurok hibája esetére (rövidzár, szakadás)
- védelem a kimenő transzformátor telítődése ellen

#### A TDA 1060 vezérlő/védő áramköreinek blokkvázlata (l. 67. ábra)

Tekintsük ezek után a tápegységünk alapáramkörének, a TDA 1060 integrált áramkörnek a konkrét jellemzőit!

#### Az IC lábainak jelentése:

- 1 belső stabilizátor bemenet
- 2 +8, 4 V-os stabilizátor referencia feszültség
- 3-4 hibavédelmi vezérlőhurok (3 hibaerősítő bemenet, 4 hibaerősítő kimenet)
- 5 az impulzus szélessége modulátor közvetlen vezérlő bemenete
- 6 a meghajtó impulzus maximális kitöltési tényezőjét meghatározó bemenet
- 7 a kapcsolási frekvencia külső beállítására szolgáló bemenet
- 8 a kapcsolási frekvencia külső beállítására szolgáló bemenet
- 9 külső szinkron bemenet
- 10 kitöltési tényező kezdeti redukciója, gyors kikapcsolás, lágyindítás bemenet
- 11 túláram védelem bemenet
- 12 belső referencia tápegység bemenet (föld)

<!-- pdf 197 -->

- 13 bemenet a kimeneti impulzus azonnali letiltására túlfeszültség esetén, vagy kimenőtrafó telítődésnél
- 14 meghajtó kapcsolótranzisztor emittere
- 15 meghajtó kapcsolótranzisztor kollektora
- 16 előreszabályozás (feed-forward) bemenet a bemeneti feszültségváltozás kompenzálására

## 7.3 A TV-Computer tápegységeinek áramköri ismertetése

A TV-Computer tápegység egy HBT-01 jelű nyomtatott áramköri kártyára épül (l. 66. ábra).

A hálózati feszültség F1 – 400 mA-es lomha olvadó biztosítékon keresztül egy mindkét pólust megszakító K1 hálózati kapcsolón, valamint egy SZ1 hálózati szűrőn keresztül közvetlenül a D9-D12 kétutas egyenirányító AC pontjaira kerül.

A TH1-es 82 ohm-os tárcsatranzisztor hidegen cc. 82 ohm-os, míg üzemi hőmérsékleten 2-3 ohm-os ellenállást képvisel és ezzel megvédi a bekapcsolási lökésszerű igénybevételtől az áramköri elemeket.

R24-C15-R23-C16-R8 a leosztott bemeneti feszültséget vezeti az N1 TDA 1060 előszabályozó bemenetére, kompenzálva ezzel a bemeneti feszültség változását.

A bemeneti DC feszültség R21 termisztoron, C13-R15-C6 bemeneti zavarszűrő elemeken keresztül N1 belső, tápbemenetéhez kapcsolódik. (Ha az N1/1-es láb 9, 5 V alatt van, a tápegység nem indul el.)

Ugyancsak ide kapcsolódik N1 meghajtó tranzisztorának kollektora (15-ös láb/N1), valamint Q1 külső meghajtó tranzisztor TR2 meghajtó transzformátoron keresztül.

<!-- pdf 198 -->

N1 belső kapcsolótranzisztorának emitter kimenete kapcsolójellel hajtja meg Q1-et, továbbá TR2-n keresztül Q2 teljesítmény kapcsoló tranzisztort (BU326A).

A kis meghajtóimpedanciáról (gyors, kis disszipációval járó átkapcsolásról) TR2 gondoskodik, amely Q1 kollektorköri impedanciáját kis impedanciára transzformálja. A TR2-es transzformátor szekunder körének parazita belengéseit R25 csillapítja.

Az N1 által szolgáltatott maximális impulzusszélességet az N1/2 referencia feszültség (8,5 V) R4-R6-os leosztásával határoljuk be. Az osztót C7 hidegíti.

A kapcsolási frekvenciát (cc. 30–35 kHz) N1/7 pontjára kapcsolódó R7 ellenállással, valamint az N1/8-as lábra kötött C8-as kapacitással határozzuk meg.

Az áramhatárolást az R12-R13-C10-R17-Q2 kapcsolótranzisztor emitterkörében elhelyezett komplexum R12-es potenciométerével állítjuk be, amely R13-on keresztül kapcsolódik N1/11-re.

A TR1 transzformátor (EC 41-es magon) szekunder tekercsei egyenirányító diódákhoz D1-D2-D3, valamint szűrőtagokhoz: FT ... -C2-C2-C3-C4 kapcsolódnak. Itt kapjuk a stabilizált egyenfeszültségeket (+5 V, ±12 V).

A TR1 8-9-es tekercsén megjelenő feszültség az R16-C11-D7-C12-R18-R19-R20 egyenirányító, szűrő, osztó után az R19-es potenciométerről N1/3-ra, azaz N1 visszacsatoló bemenetére kerül.

R18-R19-R20 feszültségosztón a kimeneti teljesítménnyel arányos feszültséget nyerünk. Így R19 potenciométerrel az adott terhelőáram mellett beállítható a kívánt feszültség.

<!-- pdf 199 -->

## 7.4 A tápegység felépítése (bontás, összeszerelés)

A tápegységet két festett műanyag alkatrész burkolja.

A tápegység alsó felén két csavar található, amelynek kicsavarozása után a felső fél eltávolítható. A további bontáshoz a billenőkapcsolót ki kell húzni fészkéből, valamint a kábeleket rögzítő bilincsek csavarjait ki kell lazítani vagy csavarni.

Az egység összeszerelésénél először az alaplemezt illesztjük a helyére, majd a billenőkapcsolót és ezután rögzítjük a kábeleket. A fedél ráhelyezése után a két összefogó csavart becsavarozzuk.

<!-- pdf 200 -->

# 8. ÁBRÁK

![1. ábra A TVC funkcionális egységei](p200-fig1.png)

1. ábra A TVC funkcionális egységei

![2. ábra Központi egység](p200-fig2.png)

2. ábra Központi egység

<!-- pdf 201 -->

![3. ábra A központi egység és a memóriák](p201-fig1.png)

3. ábra A központi egység és a memóriák

![4. ábra A lapozó áramkör vázlata](p201-fig2.png)

4. ábra A lapozó áramkör vázlata

<!-- pdf 202 -->

![5. ábra A dinamikus RAM vezérlésének időviszonyai](p202-fig1.png)

5. ábra A dinamikus RAM vezérlésének időviszonyai

![6. ábra Felhasználói RAM](p202-fig2.png)

6. ábra Felhasználói RAM

![7. ábra Video RAM](p202-fig3.png)

7. ábra Video RAM

<!-- pdf 203 -->

![8. ábra I/O dekóder](p203-fig1.png)

8. ábra I/O dekóder

![9. ábra A billentyűzetcsatoló](p203-fig2.png)

9. ábra A billentyűzetcsatoló

<!-- pdf 204 -->

![10. ábra Billentyűzetmátrix](p204-fig1.png)

10. ábra Billentyűzetmátrix

![11. ábra I/O áramkörök](p204-fig2.png)

11. ábra I/O áramkörök

<!-- pdf 205 -->

![12. ábra Hangképző áramkör](p205-fig1.png)

12. ábra Hangképző áramkör

![13. ábra Az órajel-generátor](p205-fig2.png)

13. ábra Az órajel-generátor

<!-- pdf 206 -->

![14. ábra Az órajel-generátor idődiagramja](p206-fig1.png)

14. ábra Az órajel-generátor idődiagramja

![15. ábra Video időzítőjelek előállítása](p206-fig2.png)

15. ábra Video időzítőjelek előállítása

<!-- pdf 207 -->

![16. ábra A video időzítő jelek](p207-fig1.png)

16. ábra A video időzítő jelek

![17. ábra VRAS és VCAS előállítása](p207-fig2.png)

17. ábra VRAS és VCAS előállítása

<!-- pdf 208 -->

![18. ábra A CLOCK STRETCH áramkör](p208-fig1.png)

18. ábra A CLOCK STRETCH áramkör

![19. ábra A LOAD és a VWR előállítása](p208-fig2.png)

19. ábra A LOAD és a VWR előállítása

<!-- pdf 209 -->

![20. ábra A CPU órajele videomemória használatnál](p209-fig1.png)

20. ábra A CPU órajele videomemória használatnál

![21. ábra A CLOCK STRETCH áramkör működése Φ (1) esetben](p209-fig2.png)

21. ábra A CLOCK STRETCH áramkör működése Φ (1) esetben

<!-- pdf 210 -->

![22. ábra A CLOCK STRETCH áramkör működése Φ (5) esetben](p210-fig1.png)

22. ábra A CLOCK STRETCH áramkör működése Φ (5) esetben

![23. ábra A megjelenítő egység blokkvázlata](p210-fig2.png)

23. ábra A megjelenítő egység blokkvázlata

<!-- pdf 211 -->

![24. ábra CIE "színpatkó"](p211-fig1.png)

24. ábra CIE "színpatkó"

![25. ábra PAL színsegédvivő burst jel vektorábrája](p211-fig2.png)

25. ábra PAL színsegédvivő burst jel vektorábrája

<!-- pdf 212 -->

![26. ábra Vektorábra](p212-fig1.png)

26. ábra Vektorábra

![27. ábra Additív színkeverés](p212-fig2.png)

27. ábra Additív színkeverés

<!-- pdf 213 -->

![28. ábra](p213-fig1.png)

28. ábra

<!-- pdf 214 -->

![29. ábra](p214-fig1.png)

29. ábra

<!-- pdf 215 -->

![30. ábra](p215-fig1.png)

30. ábra

<!-- pdf 216 -->

![31. ábra](p216-fig1.png)

31. ábra

<!-- pdf 217 -->

![32. ábra Csökkenő világosság sorrendű, 75 % amplitúdójú 100 %-os telítettségű színsávjel](p217-fig1.png)

32. ábra Csökkenő világosság sorrendű, 75 % amplitúdójú 100 %-os telítettségű színsávjel

<!-- pdf 218 -->

![33. ábra PAL burst (szinkronjel)](p218-fig1.png)

33. ábra PAL burst (szinkronjel)

<!-- pdf 219 -->

![34. ábra UHF modulátor blokkvázlata](p219-fig1.png)

34. ábra UHF modulátor blokkvázlata

<!-- pdf 220 -->

![35. ábra A Z80A mikroprocesszor blokkvázlata](p220-fig1.png)

35. ábra A Z80A mikroprocesszor blokkvázlata

<!-- pdf 221 -->

![36. ábra A Z80A kivezetéseinek számozása](p221-fig1.png)

36. ábra A Z80A kivezetéseinek számozása

![37. ábra A Z80A CPU alapvető időzítése](p221-fig2.png)

37. ábra A Z80A CPU alapvető időzítése

<!-- pdf 222 -->

![38. ábra Az utasítás műveleti kódjának lehívása](p222-fig1.png)

38. ábra Az utasítás műveleti kódjának lehívása

![39. ábra Az utasítás műveleti kódjának lehívása WAIT állapotokkal](p222-fig2.png)

39. ábra Az utasítás műveleti kódjának lehívása WAIT állapotokkal

![40. ábra Memóriaolvasás vagy -írás ciklus](p222-fig3.png)

40. ábra Memóriaolvasás vagy -írás ciklus

<!-- pdf 223 -->

![41. ábra Memóriaolvasás vagy -írás ciklus WAIT állapotokkal](p223-fig1.png)

41. ábra Memóriaolvasás vagy -írás ciklus WAIT állapotokkal

![42. ábra Bevitel/kivitel ciklus](p223-fig2.png)

42. ábra Bevitel/kivitel ciklus

![43. ábra Bevitel/kivitel WAIT ciklusokkal](p223-fig3.png)

43. ábra Bevitel/kivitel WAIT ciklusokkal

<!-- pdf 224 -->

![44. ábra Megszakítás kérés/nyugtázás ciklus](p224-fig1.png)

44. ábra Megszakítás kérés/nyugtázás ciklus

![45. ábra Megszakítás kérés/nyugtázás WAIT ciklusokkal](p224-fig2.png)

45. ábra Megszakítás kérés/nyugtázás WAIT ciklusokkal

![46. ábra Nem maszkolható megszakítás kérés művelet](p224-fig3.png)

46. ábra Nem maszkolható megszakítás kérés művelet

<!-- pdf 225 -->

![47. ábra Busz kérés/nyugtázás ciklus](p225-fig1.png)

47. ábra Busz kérés/nyugtázás ciklus

![48. ábra HALT időzítés](p225-fig2.png)

48. ábra HALT időzítés

![49. ábra Z80A megszakításkérés folyamatábrája](p225-fig3.png)

49. ábra Z80A megszakításkérés folyamatábrája

<!-- pdf 226 -->

![50. ábra Nem maszkolható megszakítás folyamatábrája](p226-fig1.png)

50. ábra Nem maszkolható megszakítás folyamatábrája

![51. ábra Nem maszkolható megszakításkérés művelet](p226-fig2.png)

51. ábra Nem maszkolható megszakításkérés művelet

![52. ábra Megszakítás engedélyezés/tiltás FLIP/FLOP-ok](p226-fig3.png)

52. ábra Megszakítás engedélyezés/tiltás FLIP/FLOP-ok

<!-- pdf 227 -->

![53. ábra Megszakítás üzemmódok folyamatábrái](p227-fig1.png)

53. ábra Megszakítás üzemmódok folyamatábrái

![54. ábra Vektor folyamat hatásvázlata](p227-fig2.png)

54. ábra Vektor folyamat hatásvázlata

<!-- pdf 228 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>LD r,r'</td><td>r ← r'</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>01 r r'</td><td>1</td><td>1</td><td>4</td><td rowspan="5">r,r'&emsp;Reg.<br>000&emsp;B<br>001&emsp;C<br>010&emsp;D<br>011&emsp;E<br>100&emsp;H<br>101&emsp;L<br>111&emsp;A</td></tr>
<tr><td>LD r,n</td><td>r ← n</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 r 110<br>← n →</td><td>2</td><td>2</td><td>7</td></tr>
<tr><td>LD r,(HL)</td><td>r ← (HL)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>01 r 110</td><td>1</td><td>2</td><td>7</td></tr>
<tr><td>LD r,(IX+d)</td><td>r ← (IX+d)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>01 r 110<br>← d →</td><td>3</td><td>5</td><td>19</td></tr>
<tr><td>LD r,(IY+d)</td><td>r ← (IY+d)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>01 r 110<br>← d →</td><td>3</td><td>5</td><td>19</td></tr>
<tr><td>LD (HL),r</td><td>(HL) ← r</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>01 110 r</td><td>1</td><td>2</td><td>7</td><td></td></tr>
<tr><td>LD (IX+d),r</td><td>(IX+d) ← r</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>01 110 r<br>← d →</td><td>3</td><td>5</td><td>19</td><td></td></tr>
<tr><td>LD (IY+d),r</td><td>(IY+d) ← r</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>01 110 r<br>← d →</td><td>3</td><td>5</td><td>19</td><td></td></tr>
<tr><td>LD (HL),n</td><td>(HL) ← n</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 110 110<br>← n →</td><td>2</td><td>3</td><td>10</td><td></td></tr>
<tr><td>LD (IX+d),n</td><td>(IX+d) ← n</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>00 110 110<br>← d →<br>← n →</td><td>4</td><td>5</td><td>19</td><td></td></tr>
<tr><td>LD (IY+d),n</td><td>(IY+d) ← n</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>00 110 110<br>← d →<br>← n →</td><td>4</td><td>5</td><td>19</td><td></td></tr>
<tr><td>LD A,(BC)</td><td>A ← (BC)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 001 010</td><td>1</td><td>2</td><td>7</td><td></td></tr>
<tr><td>LD A,(DE)</td><td>A ← (DE)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 011 010</td><td>1</td><td>2</td><td>7</td><td></td></tr>
<tr><td>LD A,(nn)</td><td>A ← (nn)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 111 010<br>← n →<br>← n →</td><td>3</td><td>4</td><td>13</td><td></td></tr>
<tr><td>LD (BC),A</td><td>(BC) ← A</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 000 010</td><td>1</td><td>2</td><td>7</td><td></td></tr>
<tr><td>LD (DE),A</td><td>(DE) ← A</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 010 010</td><td>1</td><td>2</td><td>7</td><td></td></tr>
</tbody>
</table>

<!-- pdf 229 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>LD (nn),A</td><td>(nn) ← A</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 110 010<br>← n →<br>← n →</td><td>3</td><td>4</td><td>13</td><td></td></tr>
<tr><td>LD A,I</td><td>A ← I</td><td>•</td><td>↕</td><td>IFF</td><td>↕</td><td>0</td><td>0</td><td>11 101 101<br>01 010 111</td><td>2</td><td>2</td><td>9</td><td></td></tr>
<tr><td>LD A,R</td><td>A ← R</td><td>•</td><td>↕</td><td>IFF</td><td>↕</td><td>0</td><td>0</td><td>11 101 101<br>01 011 111</td><td>2</td><td>2</td><td>9</td><td></td></tr>
<tr><td>LD I,A</td><td>I ← A</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 101 101<br>01 000 111</td><td>2</td><td>2</td><td>9</td><td></td></tr>
<tr><td>LD R,A</td><td>R ← A</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 101 101<br>01 001 111</td><td>2</td><td>2</td><td>9</td><td></td></tr>
</tbody>
</table>

55. ábra 8 bites load (töltő) utasítások

Megjegyzés: r,r' - az A, B, C, D, E, H, L regiszterek bármelyikét jelenti.

IFF - az (IFF) megszakításengedélyező flip-flop tartalma a P/V jelzőbe (flagbe) másolódik át.

A jelzőre (flag) vonatkozó megjegyzések: • = a jelzőt nem befolyásolja,

0 = a jelző töltődik,

1 = a jelző beállítódik,

X = a jelző közömbös,

↕ = a jelző értéke a művelet eredményétől függ.

<!-- pdf 230 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>LD dd,nn</td><td>dd ← nn</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 dd0 001<br>← n →<br>← n →</td><td>3</td><td>3</td><td>10</td><td rowspan="2">dd&emsp;Pair<br>00&emsp;BC<br>01&emsp;DE<br>10&emsp;HL<br>11&emsp;SP</td></tr>
<tr><td>LD IX,nn</td><td>IX ← nn</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>00 100 001<br>← n →<br>← n →</td><td>4</td><td>4</td><td>14</td></tr>
<tr><td>LD IY,nn</td><td>IY ← nn</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>00 100 001<br>← n →<br>← n →</td><td>4</td><td>4</td><td>14</td><td></td></tr>
<tr><td>LD HL,(nn)</td><td>H ← (nn+1)<br>L ← (nn)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 101 010<br>← n →<br>← n →</td><td>3</td><td>5</td><td>16</td><td></td></tr>
<tr><td>LD dd,(nn)</td><td>dd<sub>H</sub> ← (nn+1)<br>dd<sub>L</sub> ← (nn)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 101 101<br>01 dd1 011<br>← n →<br>← n →</td><td>4</td><td>6</td><td>20</td><td></td></tr>
<tr><td>LD IX,(nn)</td><td>IX<sub>H</sub> ← (nn+1)<br>IX<sub>L</sub> ← (nn)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>00 101 010<br>← n →<br>← n →</td><td>4</td><td>6</td><td>20</td><td></td></tr>
<tr><td>LD IY,(nn)</td><td>IY<sub>H</sub> ← (nn+1)<br>IY<sub>L</sub> ← (nn)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>00 101 010<br>← n →<br>← n →</td><td>4</td><td>6</td><td>20</td><td></td></tr>
<tr><td>LD (nn),HL</td><td>(nn+1) ← H<br>(nn) ← L</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 100 010<br>← n →<br>← n →</td><td>3</td><td>5</td><td>16</td><td></td></tr>
<tr><td>LD (nn),dd</td><td>(nn+1) ← dd<sub>H</sub><br>(nn) ← dd<sub>L</sub></td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 101 101<br>01 dd0 011<br>← n →<br>← n →</td><td>4</td><td>6</td><td>20</td><td></td></tr>
</tbody>
</table>

<!-- pdf 231 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>LD (nn),IX</td><td>(nn+1) ← IX<sub>H</sub><br>(nn) ← IX<sub>L</sub></td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>00 100 010<br>← n →<br>← n →</td><td>4</td><td>6</td><td>20</td><td></td></tr>
<tr><td>LD (nn),IY</td><td>(nn+1) ← IY<sub>H</sub><br>(nn) ← IY<sub>L</sub></td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>00 100 010<br>← n →<br>← n →</td><td>4</td><td>6</td><td>20</td><td></td></tr>
<tr><td>LD SP,HL</td><td>SP ← HL</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 001</td><td>1</td><td>1</td><td>6</td><td></td></tr>
<tr><td>LD SP,IX</td><td>SP ← IX</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>11 111 001</td><td>2</td><td>2</td><td>10</td><td></td></tr>
<tr><td>LD SP,IY</td><td>SP ← IY</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>11 111 001</td><td>2</td><td>2</td><td>10</td><td rowspan="3">qq&emsp;Pair<br>00&emsp;BC<br>01&emsp;DE<br>10&emsp;HL<br>11&emsp;AF</td></tr>
<tr><td>PUSH qq</td><td>(SP-2) ← qq<sub>L</sub><br>(SP-1) ← qq<sub>H</sub></td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 qq0 101</td><td>1</td><td>3</td><td>11</td></tr>
<tr><td>PUSH IX</td><td>(SP-2) ← IX<sub>L</sub><br>(SP-1) ← IX<sub>H</sub></td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>11 100 101</td><td>2</td><td>4</td><td>15</td></tr>
<tr><td>PUSH IY</td><td>(SP-2) ← IY<sub>L</sub><br>(SP-1) ← IY<sub>H</sub></td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>11 100 101</td><td>2</td><td>4</td><td>15</td><td></td></tr>
<tr><td>POP qq</td><td>qq<sub>H</sub> ← (SP+1)<br>qq<sub>L</sub> ← (SP)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 qq0 001</td><td>1</td><td>3</td><td>10</td><td></td></tr>
<tr><td>POP IX</td><td>IX<sub>H</sub> ← (SP+1)<br>IX<sub>L</sub> ← (SP)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>11 100 001</td><td>2</td><td>4</td><td>14</td><td></td></tr>
<tr><td>POP IY</td><td>IY<sub>H</sub> ← (SP+1)<br>IY<sub>L</sub> ← (SP)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>11 100 001</td><td>2</td><td>4</td><td>14</td><td></td></tr>
</tbody>
</table>

56. ábra 16 bites load (töltő) utasítások

Megjegyzés: dd - bármelyik regiszterpár a BC, DE, HL, SP közül.

qq - bármelyik regiszterpár az AF, BC, DE, HL közül.

(PAIR)<sub>H</sub>·(PAIR)<sub>L</sub> - külön-külön a regiszter magas, illetve alacsony helyi értékű 8 bitjére vonatkozik.

Pl.: BC<sub>L</sub> = C, AF<sub>H</sub> = A

<!-- pdf 232 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>EX DE,HL</td><td>DE ↔ HL</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 101 011</td><td>1</td><td>1</td><td>4</td><td></td></tr>
<tr><td>EX AF,AF'</td><td>AF ↔ AF'</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 001 000</td><td>1</td><td>1</td><td>4</td><td></td></tr>
<tr><td>EXX</td><td><img src="p232-fig1.png" alt="BC, DE, HL ↔ BC, DE, HL"></td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 001</td><td>1</td><td>1</td><td>4</td><td>Regisztertömb és segédregisztertömb csere</td></tr>
<tr><td>EX(SP),HL</td><td>H ↔ (SP+1)<br>L ↔ (SP)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 100 011</td><td>1</td><td>5</td><td>19</td><td></td></tr>
<tr><td>EX(SP),IX</td><td>IX<sub>H</sub> ↔ (SP+1)<br>IX<sub>L</sub> ↔ (SP)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>11 100 011</td><td>2</td><td>6</td><td>23</td><td></td></tr>
<tr><td>EX(SP),IY</td><td>IY<sub>H</sub> ↔ (SP+1)<br>IY<sub>L</sub> ↔ (SP)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>11 100 011</td><td>2</td><td>6</td><td>23</td><td></td></tr>
<tr><td>LDI</td><td>(DE) ← (HL)<br>DE ← DE+1<br>HL ← HL+1<br>BC ← BC-1</td><td>•</td><td>•</td><td>① 1</td><td>•</td><td>0</td><td>0</td><td>11 101 101<br>10 100 000</td><td>2</td><td>4</td><td>16</td><td>(HL) betöltése (DE)-be, címmutató növelése, és a byte-számláló (BC) csökkentése</td></tr>
<tr><td>LDIR</td><td>(DE) ← (HL)<br>DE ← DE+1<br>HL ← HL+1<br>BC ← BC-1<br>Ismétlés<br>addig amíg<br>BC = 0</td><td>•</td><td>•</td><td>0</td><td>•</td><td>0</td><td>0</td><td>11 101 101<br>10 110 000</td><td>2<br>2</td><td>5<br>4</td><td>21<br>16</td><td>If BC ≠ 0<br>If BC = 0</td></tr>
<tr><td>LDD</td><td>(DE) ← (HL)<br>DE ← DE-1<br>HL ← HL-1<br>BC ← BC-1</td><td>•</td><td>•</td><td>① 1</td><td>•</td><td>0</td><td>0</td><td>11 101 101<br>10 101 000</td><td>2</td><td>4</td><td>16</td><td></td></tr>
</tbody>
</table>

<!-- pdf 233 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>LDDR</td><td>(DE) ← (HL)<br>DE ← DE-1<br>HL ← HL-1<br>BC ← BC-1<br>Ismétlés<br>addig amíg<br>BC = 0</td><td>•</td><td>•</td><td>0</td><td>•</td><td>0</td><td>0</td><td>11 101 101<br>10 111 000</td><td>2<br>2</td><td>5<br>4</td><td>21<br>16</td><td>If BC ≠ 0<br>If BC = 0</td></tr>
<tr><td>CPI</td><td>A - (HL)<br>HL ← HL+1<br>BC ← BC-1</td><td>•</td><td>② ↕</td><td>① ↕</td><td>↕</td><td>1</td><td>↕</td><td>11 101 101<br>10 100 001</td><td>2</td><td>4</td><td>16</td><td></td></tr>
<tr><td>CPIR</td><td>A - (HL)<br>HL ← HL+1<br>BC ← BC-1<br>Ismétlés<br>addig amíg<br>A = (HL)<br>vagy<br>BC = 0</td><td>•</td><td>② ↕</td><td>① ↕</td><td>↕</td><td>1</td><td>↕</td><td>11 101 101<br>10 110 001</td><td>2<br>2</td><td>5<br>4</td><td>21<br>16</td><td>If BC ≠ 0 és A ≠ (HL)<br>If BC = 0 vagy A = (HL)</td></tr>
<tr><td>CPD</td><td>A - (HL)<br>HL ← HL-1<br>BC ← BC-1</td><td>•</td><td>② ↕</td><td>① ↕</td><td>↕</td><td>1</td><td>↕</td><td>11 101 101<br>10 101 001</td><td>2</td><td>4</td><td>16</td><td></td></tr>
</tbody>
</table>

<!-- pdf 234 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>CPDR</td><td>A - (HL)<br>HL ← HL-1<br>BC ← BC-1<br>Ismétlés<br>addig amíg<br>A = (HL)<br>vagy<br>BC = 0</td><td>•</td><td>② ↕</td><td>① ↕</td><td>↕</td><td>1</td><td>↕</td><td>11 101 101<br>10 111 001</td><td>2<br>2</td><td>5<br>4</td><td>21<br>16</td><td>If BC ≠ 0 és A ≠ (HL)<br>If BC = 0 vagy A = (HL)</td></tr>
</tbody>
</table>

57. ábra Cserélő, valamint adatblokkátvivő és -kereső utasítások

Megjegyzés: ① P/V jelző 0, ha a BC - 1 = 0, egyébként P/V = 1

② Z jelző 1, ha A = (HL), egyébként Z = 0

<!-- pdf 235 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>ADD A,r</td><td>A ← A+r</td><td>↕</td><td>↕</td><td>V</td><td>↕</td><td>0</td><td>↕</td><td>10 <span style="border:1px solid">000</span> r</td><td>1</td><td>1</td><td>4</td><td rowspan="5">r&emsp;Reg.<br>000&emsp;B<br>001&emsp;C<br>010&emsp;D<br>011&emsp;E<br>100&emsp;H<br>101&emsp;L<br>111&emsp;A</td></tr>
<tr><td>ADD A,n</td><td>A ← A+n</td><td>↕</td><td>↕</td><td>V</td><td>↕</td><td>0</td><td>↕</td><td>11 <span style="border:1px solid">000</span> 110<br>← n →</td><td>2</td><td>2</td><td>7</td></tr>
<tr><td>ADD A,(HL)</td><td>A ← A+(HL),</td><td>↕</td><td>↕</td><td>V</td><td>↕</td><td>0</td><td>↕</td><td>10 <span style="border:1px solid">000</span> 110</td><td>1</td><td>2</td><td>7</td></tr>
<tr><td>ADD A,(IX+d)</td><td>A ← A+(IX+d)</td><td>↕</td><td>↕</td><td>V</td><td>↕</td><td>0</td><td>↕</td><td>11 011 101<br>10 <span style="border:1px solid">000</span> 110<br>← d →</td><td>3</td><td>5</td><td>19</td></tr>
<tr><td>ADD A,(IY+d)</td><td>A ← A+(IY+d)</td><td>↕</td><td>↕</td><td>V</td><td>↕</td><td>0</td><td>↕</td><td>11 111 101<br>10 <span style="border:1px solid">000</span> 110<br>← d →</td><td>3</td><td>5</td><td>19</td></tr>
<tr><td>ADC A,s</td><td>A ← A+s+CY</td><td>↕</td><td>↕</td><td>V</td><td>↕</td><td>0</td><td>↕</td><td><span style="border:1px solid">001</span></td><td></td><td></td><td></td><td rowspan="7">s - az r, n, (HL),(IX+d),(IY+d) bármelyikét jelenti, ahogy azt az ADD utasításnál látjuk.<br>A jelzett bitek a 000-t felváltják az alábbi ADD utasításokban.</td></tr>
<tr><td>SUB s</td><td>A ← A - s</td><td>↕</td><td>↕</td><td>V</td><td>↕</td><td>1</td><td>↕</td><td><span style="border:1px solid">010</span></td><td></td><td></td><td></td></tr>
<tr><td>SBC A,s</td><td>A ← A-s-CY</td><td>↕</td><td>↕</td><td>V</td><td>↕</td><td>1</td><td>↕</td><td><span style="border:1px solid">011</span></td><td></td><td></td><td></td></tr>
<tr><td>AND s</td><td>A ← A ∧ s</td><td>0</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>1</td><td><span style="border:1px solid">100</span></td><td></td><td></td><td></td></tr>
<tr><td>OR s</td><td>A ← A ∨ s</td><td>0</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td><span style="border:1px solid">110</span></td><td></td><td></td><td></td></tr>
<tr><td>XOR s</td><td>A ← A ⊕ s</td><td>0</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td><span style="border:1px solid">101</span></td><td></td><td></td><td></td></tr>
<tr><td>CP s</td><td>A - s</td><td>↕</td><td>↕</td><td>V</td><td>↕</td><td>1</td><td>↕</td><td><span style="border:1px solid">111</span></td><td></td><td></td><td></td></tr>
<tr><td>INC r</td><td>r ← r + 1</td><td>•</td><td>↕</td><td>V</td><td>↕</td><td>0</td><td>↕</td><td>00 &emsp;&emsp; <span style="border:1px solid">100</span></td><td>1</td><td>1</td><td>4</td><td></td></tr>
<tr><td>INC (HL)</td><td>(HL) ← (HL)+1</td><td>•</td><td>↕</td><td>V</td><td>↕</td><td>0</td><td>↕</td><td>00 110 <span style="border:1px solid">100</span></td><td>1</td><td>3</td><td>11</td><td></td></tr>
<tr><td>INC (IX+d)</td><td>(IX+d) ←<br>(IX+d)+1</td><td>•</td><td>↕</td><td>V</td><td>↕</td><td>0</td><td>↕</td><td>11 011 101<br>00 110 <span style="border:1px solid">100</span><br>• d •</td><td>3</td><td>6</td><td>23</td><td></td></tr>
<tr><td>INC(IY+d)</td><td>(IY+d) ←<br>(IY+d)+1</td><td>•</td><td>↕</td><td>V</td><td>↕</td><td>0</td><td>↕</td><td>11 111 101<br>00 110 <span style="border:1px solid">100</span><br>• d •</td><td>3</td><td>6</td><td>23</td><td></td></tr>
</tbody>
</table>

<!-- pdf 236 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>DEC m</td><td>m ← m -1</td><td>•</td><td>↕</td><td>V</td><td>↕</td><td>1</td><td>↕</td><td><span style="border:1px solid">101</span></td><td></td><td></td><td></td><td>m - mint az INC utasításoknál láttuk - az r, (HL),(IX+d),(IY+d) bármelyike lehet, azonos formátummal és státusszal, mint az INC-nél csupán a műveleti kódban az 100-at 101 váltja fel.</td></tr>
</tbody>
</table>

58. ábra 8 bites aritmetikai és logikai utasítások

Megjegyzés: A V szimbólum a P/V jelző oszlopában azt jelzi, hogy a műveletvégzés eredményeként túlcsordulás keletkezett, amit a P/V jelző tartalmaz. Hasonlóan a P szimbólum a paritást jelzi, V=1 jelenti, hogy túlcsordulás van, V=0 jelenti, hogy nincs túlcsordulás, P=1 jelenti, hogy az eredmény paritása páros, P=0 jelenti, hogy az eredmény paritása páratlan.

<!-- pdf 237 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>ADD HL,ss</td><td>HL ← HL+ss</td><td>↕</td><td>•</td><td>•</td><td>•</td><td>0</td><td>X</td><td>00 ss1 001</td><td>1</td><td>3</td><td>11</td><td rowspan="3">ss&emsp;Reg.<br>00&emsp;BC<br>01&emsp;DE<br>10&emsp;HL<br>11&emsp;SP</td></tr>
<tr><td>ADC HL,ss</td><td>HL ← HL+ss+CY</td><td>↕</td><td>↕</td><td>V</td><td>↕</td><td>0</td><td>X</td><td>11 101 101<br>01 ss1 010</td><td>2</td><td>4</td><td>15</td></tr>
<tr><td>SBC HL,ss</td><td>HL ← HL-ss-CY</td><td>↕</td><td>↕</td><td>V</td><td>↕</td><td>1</td><td>X</td><td>11 101 101<br>01 ss0 010</td><td>2</td><td>4</td><td>15</td></tr>
<tr><td>ADD IX,pp</td><td>IX ← IX+pp</td><td>↕</td><td>•</td><td>•</td><td>•</td><td>0</td><td>X</td><td>11 011 101<br>00 pp1 001</td><td>2</td><td>4</td><td>15</td><td>pp&emsp;Reg.<br>00&emsp;BC<br>01&emsp;DE<br>10&emsp;IX<br>11&emsp;SP</td></tr>
<tr><td>ADD IY,rr</td><td>IY ← IY+rr</td><td>↕</td><td>•</td><td>•</td><td>•</td><td>0</td><td>X</td><td>11 111 101<br>00 rr1 001</td><td>2</td><td>4</td><td>15</td><td>rr&emsp;Reg.<br>00&emsp;BC<br>01&emsp;DE<br>10&emsp;IY<br>11&emsp;SP</td></tr>
<tr><td>INC ss</td><td>ss ← ss + 1</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 ss0 011</td><td>1</td><td>1</td><td>6</td><td></td></tr>
<tr><td>INC IX</td><td>IX ← IX + 1</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>00 100 011</td><td>2</td><td>2</td><td>10</td><td></td></tr>
<tr><td>INC IY</td><td>IY ← IY + 1</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>00 100 011</td><td>2</td><td>2</td><td>10</td><td></td></tr>
<tr><td>DEC ss</td><td>ss ← ss - 1</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 ss1 011</td><td>1</td><td>1</td><td>6</td><td></td></tr>
<tr><td>DEC IX</td><td>IX ← IX - 1</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>00 101 011</td><td>2</td><td>2</td><td>10</td><td></td></tr>
<tr><td>DEC IY</td><td>IY ← IY - 1</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>00 101 011</td><td>2</td><td>2</td><td>10</td><td></td></tr>
</tbody>
</table>

59. ábra 16 bites aritmetikai utasítások

Megjegyzés: ss - a BC, DE, HL, SP regiszterpárok bármelyike

pp - a BC, DE, IX, SP regiszterpárok bármelyike

rr - a BC, DE, IY, SP regiszterpárok bármelyike

<!-- pdf 238 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>RLCA</td><td><img src="p238-fig1.png" alt="RLCA"></td><td>↕</td><td>•</td><td>•</td><td>•</td><td>0</td><td>0</td><td>00 000 111</td><td>1</td><td>1</td><td>4</td><td>Az akkumulátor körbeforgatása balra.</td></tr>
<tr><td>RLA</td><td><img src="p238-fig2.png" alt="RLA"></td><td>↕</td><td>•</td><td>•</td><td>•</td><td>0</td><td>0</td><td>00 010 111</td><td>1</td><td>1</td><td>4</td><td>Az akkumulátor balra forgatása.</td></tr>
<tr><td>RRCA</td><td><img src="p238-fig3.png" alt="RRCA"></td><td>↕</td><td>•</td><td>•</td><td>•</td><td>0</td><td>0</td><td>00 001 111</td><td>1</td><td>1</td><td>4</td><td>Az akkumulátor körbeforgatása jobbra.</td></tr>
<tr><td>RRA</td><td><img src="p238-fig4.png" alt="RRA"></td><td>↕</td><td>•</td><td>•</td><td>•</td><td>0</td><td>0</td><td>00 011 111</td><td>1</td><td>1</td><td>4</td><td>Az akkumulátor jobbra forgatása.</td></tr>
<tr><td>RLCr</td><td rowspan="4">} <img src="p238-fig5.png" alt="RLC r,(HL),(IX+d),(IY+d)"></td><td>↕</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td>11 001 011<br>00 <span style="border:1px solid">000</span> r</td><td>2</td><td>2</td><td>8</td><td>Az r regiszter körbeforgatása.</td></tr>
<tr><td>RLC (HL)</td><td>↕</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td>11 001 011<br>00 <span style="border:1px solid">000</span> 110</td><td>2</td><td>4</td><td>15</td><td rowspan="3">r&emsp;Reg.<br>000&emsp;B<br>001&emsp;C<br>010&emsp;D<br>011&emsp;E<br>100&emsp;H<br>101&emsp;L<br>111&emsp;A</td></tr>
<tr><td>RLC(IX+d)</td><td>↕</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td>11 011 101<br>11 001 011<br>← d →<br>00 <span style="border:1px solid">000</span> 110</td><td>4</td><td>6</td><td>23</td></tr>
<tr><td>RLC(IY+d)</td><td>↕</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td>11 111 101<br>11 001 011<br>← d →<br>00 <span style="border:1px solid">000</span> 110</td><td>4</td><td>6</td><td>23</td></tr>
</tbody>
</table>

<!-- pdf 239 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>RL m</td><td><img src="p239-fig1.png" alt="RL m"></td><td>↕</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td><span style="border:1px solid">010</span></td><td></td><td></td><td></td><td rowspan="6">Az utasítás formátuma és státusza ugyanolyan, mint az RLC utasításnál. Az új műveleti kód formátumánál az RLC utasítás 000-ját felváltja a jelzett kód.</td></tr>
<tr><td>RRC m</td><td><img src="p239-fig2.png" alt="RRC m"></td><td>↕</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td><span style="border:1px solid">001</span></td><td></td><td></td><td></td></tr>
<tr><td>RR m</td><td><img src="p239-fig3.png" alt="RR m"></td><td>↕</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td><span style="border:1px solid">011</span></td><td></td><td></td><td></td></tr>
<tr><td>SLA m</td><td><img src="p239-fig4.png" alt="SLA m"></td><td>↕</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td><span style="border:1px solid">100</span></td><td></td><td></td><td></td></tr>
<tr><td>SRA m</td><td><img src="p239-fig5.png" alt="SRA m"></td><td>↕</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td>101</td><td></td><td></td><td></td></tr>
<tr><td>SRL m</td><td><img src="p239-fig6.png" alt="SRL m"></td><td>↕</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td><span style="border:1px solid">111</span></td><td></td><td></td><td></td></tr>
</tbody>
</table>

<!-- pdf 240 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>RLD</td><td><img src="p240-fig1.png" alt="RLD: A és (HL) 7-4 / 3-0 bitek forgatása"></td><td>•</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td>11 101 101<br>01 101 111</td><td>2</td><td>5</td><td>18</td><td>Digit balra és jobbra forgatása az akkumulátor és a (HL) között.</td></tr>
<tr><td>RRD</td><td><img src="p240-fig2.png" alt="RRD: A és (HL) 7-4 / 3-0 bitek forgatása"></td><td>•</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>0</td><td>11 101 101<br>01 100 111</td><td>2</td><td>5</td><td>18</td><td>Az akkumulátor felső felének a tartalma változatlan.</td></tr>
</tbody>
</table>

60. ábra Forgató és léptető utasítások

<!-- pdf 241 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>BIT b,r</td><td>Z ← <span style="text-decoration:overline">r</span><sub>b</sub></td><td>•</td><td>↕</td><td>X</td><td>X</td><td>0</td><td>1</td><td>11 001 011<br>01 b r</td><td>2</td><td>2</td><td>8</td><td rowspan="8"><table><tr><th>r</th><th>Reg.</th></tr><tr><td>000</td><td>B</td></tr><tr><td>001</td><td>C</td></tr><tr><td>010</td><td>D</td></tr><tr><td>011</td><td>E</td></tr><tr><td>100</td><td>H</td></tr><tr><td>101</td><td>L</td></tr><tr><td>111</td><td>A</td></tr></table><br><table><tr><th>b</th><th>Tesztelt bit</th></tr><tr><td>000</td><td>0</td></tr><tr><td>001</td><td>1</td></tr><tr><td>010</td><td>2</td></tr><tr><td>011</td><td>3</td></tr><tr><td>100</td><td>4</td></tr><tr><td>101</td><td>5</td></tr><tr><td>110</td><td>6</td></tr><tr><td>111</td><td>7</td></tr></table></td></tr>
<tr><td>BIT b,(HL)</td><td>Z ← <span style="text-decoration:overline">(HL)</span><sub>b</sub></td><td>•</td><td>↕</td><td>X</td><td>X</td><td>0</td><td>1</td><td>11 001 011<br>01 b 110</td><td>2</td><td>3</td><td>12</td></tr>
<tr><td>BIT b,<br>(IX+d)</td><td>Z ← <span style="text-decoration:overline">(IX+d)</span><sub>b</sub></td><td>•</td><td>↕</td><td>X</td><td>X</td><td>0</td><td>1</td><td>11 011 101<br>11 001 011<br>← d →<br>01 b 110</td><td>4</td><td>5</td><td>20</td></tr>
<tr><td>BIT b,<br>(IY+d)</td><td>Z ← <span style="text-decoration:overline">(IY+d)</span><sub>b</sub></td><td>•</td><td>↕</td><td>X</td><td>X</td><td>0</td><td>1</td><td>11 111 101<br>11 001 011<br>← d →<br>01 b 110</td><td>4</td><td>5</td><td>20</td></tr>
<tr><td>SET b,r</td><td>r<sub>b</sub> ← 1</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 001 011<br><span style="border:1px solid">11</span> b r</td><td>2</td><td>2</td><td>8</td></tr>
<tr><td>SET b,(HL)</td><td>(HL)<sub>b</sub> ← 1</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 001 011<br><span style="border:1px solid">11</span> b 110</td><td>2</td><td>4</td><td>15</td></tr>
<tr><td>SET b,<br>(IX+d)</td><td>(IX+d)<sub>b</sub> ← 1</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>11 001 011<br>← d →<br><span style="border:1px solid">11</span> b 110</td><td>4</td><td>6</td><td>23</td></tr>
<tr><td>SET b,<br>(IY+d)</td><td>(IY+d)<sub>b</sub> ← 1</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>11 001 011<br>← d →<br><span style="border:1px solid">11</span> b 110</td><td>4</td><td>6</td><td>23</td></tr>
</tbody>
</table>

<!-- pdf 242 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>RES b,m</td><td>s<sub>b</sub> ← 0<br>m ≡ r,(HL),<br>(IX+d),<br>(IY+d)</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td><span style="border:1px solid">10</span></td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>Az új műveleti kódban a SET utasítás 11 értékét 10 váltja fel. A jelzők és állapotidők a SET utasításéval azonosak.</td></tr>
</tbody>
</table>

61. ábra 8 bites beállító, törlő és tesztelő utasítások

Megjegyzés: s<sub>b</sub> - s b-edik bitjét jelöli.

<!-- pdf 243 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>DAA</td><td>Az akkumulátor tartalmának pakolt BCD kóddá konvertálása, pakolt BCD operandusú összeadást vagy kivonást követően.</td><td>↕</td><td>↕</td><td>P</td><td>↕</td><td>•</td><td>↕</td><td>00 100 111</td><td>1</td><td>1</td><td>4</td><td>Az akkumulátor decimális beállítása.</td></tr>
<tr><td>CPL</td><td>A ← <span style="text-decoration:overline">A</span></td><td>•</td><td>•</td><td>•</td><td>•</td><td>1</td><td>1</td><td>00 101 111</td><td>1</td><td>1</td><td>4</td><td>Az akkumulátor komplementálása (1-es komplemens)</td></tr>
<tr><td>NEG</td><td>A ← 0 -A</td><td>↕</td><td>↕</td><td>V</td><td>↕</td><td>1</td><td>↕</td><td>11 101 101<br>01 000 100</td><td>2</td><td>2</td><td>8</td><td>Akkumulátor negálása (2-es komplemens)</td></tr>
<tr><td>CCF</td><td>CY ← <span style="text-decoration:overline">CY</span></td><td>↕</td><td>•</td><td>•</td><td>•</td><td>0</td><td>X</td><td>00 111 111</td><td>1</td><td>1</td><td>4</td><td>Carry jelző komplementálása</td></tr>
<tr><td>SCF</td><td>CY ← 1</td><td>1</td><td>•</td><td>•</td><td>•</td><td>0</td><td>0</td><td>00 110 111</td><td>1</td><td>1</td><td>4</td><td>Carry jelző beállítása</td></tr>
<tr><td>NOP</td><td>Nem művelet</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 000 000</td><td>1</td><td>1</td><td>4</td><td></td></tr>
<tr><td>HALT</td><td>CPU beállítva</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>01 110 110</td><td>1</td><td>1</td><td>4</td><td></td></tr>
<tr><td>DI</td><td>IFF ← 0</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 110 011</td><td>1</td><td>1</td><td>4</td><td></td></tr>
<tr><td>EI</td><td>IFF ← 1</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 011</td><td>1</td><td>1</td><td>4</td><td></td></tr>
</tbody>
</table>

<!-- pdf 244 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>IM 0</td><td>0-ás megszakítási mód beállítása</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 101 101<br>01 000 110</td><td>2</td><td>2</td><td>8</td><td></td></tr>
<tr><td>IM 1</td><td>1-es megszakítási mód beállítása</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 101 101<br>01 010 110</td><td>2</td><td>2</td><td>8</td><td></td></tr>
<tr><td>IM 2</td><td>2-es megszakítási mód beállítása</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 101 101<br>01 011 110</td><td>2</td><td>2</td><td>8</td><td></td></tr>
</tbody>
</table>

62. ábra Általános célú aritmetikai és CPU vezérlő utasítások

Megjegyzés: IFF - a megszakításengedélyező flip-flop állapotát jelzi

CY - a carry flip-flop állapotát jelzi

<!-- pdf 245 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>JP nn</td><td>PC ← nn</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 000 011<br>← n →<br>← n →</td><td>3</td><td>3</td><td>10</td><td></td></tr>
<tr><td>JP cc,nn</td><td>Ha a cc feltétel igaz PC ← nn, egyébként a következő utasítás</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 cc 010<br>← n →<br>← n →</td><td>3</td><td>3</td><td>10</td><td rowspan="2"><table><tr><th>cc</th><th colspan="2">Feltétel</th></tr><tr><td>000</td><td>NZ</td><td>nem zéró</td></tr><tr><td>001</td><td>Z</td><td>zéró</td></tr><tr><td>010</td><td>NC</td><td>nincs átvitel</td></tr><tr><td>011</td><td>C</td><td>átvitel</td></tr><tr><td>100</td><td>PO</td><td>paritás páratlan</td></tr><tr><td>101</td><td>PE</td><td>paritás páros</td></tr><tr><td>110</td><td>P</td><td>előjel pozitív</td></tr><tr><td>111</td><td>M</td><td>előjel negatív</td></tr></table></td></tr>
<tr><td>JR e</td><td>PC ← PC+e</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>00 011 000<br>← e-2 →</td><td>2</td><td>3</td><td>12</td></tr>
<tr><td rowspan="2">JR C,e</td><td rowspan="2">Ha C = 0,<br>a következő<br>Ha C = 1,<br>PC ← PC+e</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">00 111 000<br>← e-2 →</td><td>2</td><td>2</td><td>7</td><td>Ha a feltétel nem igaz</td></tr>
<tr><td>2</td><td>3</td><td>12</td><td>Ha a feltétel igaz</td></tr>
<tr><td rowspan="2">JR NC,e</td><td rowspan="2">Ha C = 1,<br>a következő<br>Ha C = 0,<br>PC ← PC+e</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">00 110 000<br>← e-2 →</td><td>2</td><td>2</td><td>7</td><td>Ha a feltétel nem igaz</td></tr>
<tr><td>2</td><td>3</td><td>12</td><td>Ha a feltétel igaz</td></tr>
<tr><td rowspan="2">JR Z,e</td><td rowspan="2">Ha Z = 0,<br>a következő<br>Ha Z = 1,<br>PC ← PC+e</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">00 101 000<br>← e-2 →</td><td>2</td><td>2</td><td>7</td><td>Ha a feltétel nem igaz</td></tr>
<tr><td>2</td><td>3</td><td>12</td><td>Ha a feltétel igaz</td></tr>
<tr><td rowspan="2">JR NZ,e</td><td rowspan="2">Ha Z = 1,<br>a következő<br>Ha Z = 0,<br>PC ← PC+e</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">00 100 000<br>← e-2 →</td><td>2</td><td>2</td><td>7</td><td>Ha a feltétel nem igaz</td></tr>
<tr><td>2</td><td>3</td><td>12</td><td>Ha a feltétel igaz</td></tr>
<tr><td>JP (HL)</td><td>PC ← HL</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 101 001</td><td>1</td><td>1</td><td>4</td><td></td></tr>
</tbody>
</table>

<!-- pdf 246 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>JP (IX)</td><td>PC ← IX</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 101<br>11 101 001</td><td>2</td><td>2</td><td>8</td><td></td></tr>
<tr><td>JP (IY)</td><td>PC ← IY</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 111 101<br>11 101 001</td><td>2</td><td>2</td><td>8</td><td></td></tr>
<tr><td rowspan="2">DJNZ,e</td><td rowspan="2">B ← B-1<br>Ha B = 0,<br>a következő<br>Ha B ≠ 0,<br>PC ← PC+e</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">00 010 000<br>← e-2 →</td><td>2</td><td>2</td><td>8</td><td>Ha B = 0</td></tr>
<tr><td>2</td><td>3</td><td>13</td><td>Ha B ≠ 0</td></tr>
</tbody>
</table>

63. ábra Ugróutasítások

Megjegyzés: e - az ugrás távolságát jelzi relatív címzésmódban

e - egy előjeles kettes komplemensű szám a <-126,129> tartományban

e-2 - a műveleti kódban lehetővé teszi a PC+e-nek effektív címzését, mivel a PC 2-vel inkrementálódik az e hozzáadását megelőzően.

<!-- pdf 247 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>CALL nn</td><td>(SP-1)←PC<sub>H</sub><br>(SP-2)←PC<sub>L</sub><br>PC ← nn</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 001 101<br>← n →<br>← n →</td><td>3</td><td>5</td><td>17</td><td></td></tr>
<tr><td rowspan="2">CALL cc,nn</td><td rowspan="2">Ha a cc feltétel nem igaz, akkor a következő utasítás, egyébként azonos a CALL nn-nel</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">11 cc 100<br>← n →<br>← n →</td><td>3</td><td>3</td><td>10</td><td>Ha cc nem igaz</td></tr>
<tr><td>3</td><td>5</td><td>17</td><td>Ha cc igaz</td></tr>
<tr><td>RET</td><td>PC<sub>L</sub>←(SP)<br>PC<sub>H</sub>←(SP+1)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 001 001</td><td>1</td><td>3</td><td>10</td><td></td></tr>
<tr><td rowspan="2">RET cc</td><td rowspan="2">Ha a cc feltétel hamis, akkor a következő utasítás, egyébként azonos a RET-tel</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">•</td><td rowspan="2">11 cc 000</td><td>1</td><td>1</td><td>5</td><td>Ha cc nem igaz</td></tr>
<tr><td>1</td><td>3</td><td>11</td><td>Ha cc igaz</td></tr>
</tbody>
</table>

<!-- pdf 248 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>RETI</td><td>Visszatérés megszakításból</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 101 101<br>01 001 101</td><td>2</td><td>4</td><td>14</td><td rowspan="2"><table><tr><th>cc</th><th colspan="2">Feltétel</th></tr><tr><td>000</td><td>NZ</td><td>nem zéró</td></tr><tr><td>001</td><td>Z</td><td>zéró</td></tr><tr><td>010</td><td>NC</td><td>nincs átvitel</td></tr><tr><td>011</td><td>C</td><td>átvitel</td></tr><tr><td>100</td><td>PO</td><td>paritás páratlan</td></tr><tr><td>101</td><td>PE</td><td>paritás páros</td></tr><tr><td>110</td><td>P</td><td>előjel pozitív</td></tr><tr><td>111</td><td>M</td><td>előjel negatív</td></tr></table></td></tr>
<tr><td>RETN</td><td>Visszatérés nem maszkolható megszakításból</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 101 101<br>01 000 101</td><td>2</td><td>4</td><td>14</td></tr>
<tr><td>RST<sub>p</sub></td><td>(SP-1)←PC<sub>H</sub><br>(SP-2)←PC<sub>L</sub><br>PC<sub>H</sub> ← 0<br>PC<sub>L</sub> ← P</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 t 111</td><td>1</td><td>3</td><td>11</td><td><table><tr><th>t</th><th>P</th></tr><tr><td>000</td><td>00H</td></tr><tr><td>001</td><td>08H</td></tr><tr><td>010</td><td>10H</td></tr><tr><td>011</td><td>18H</td></tr><tr><td>100</td><td>20H</td></tr><tr><td>101</td><td>28H</td></tr><tr><td>110</td><td>30H</td></tr><tr><td>111</td><td>38H</td></tr></table></td></tr>
</tbody>
</table>

64. ábra Szubrutinhívó és a szubrutinból visszatérő utasítások

<!-- pdf 249 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>IN A,(n)</td><td>A ← (n)</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 011 011<br>← n →</td><td>2</td><td>3</td><td>11</td><td>n - A<sub>0</sub>-A<sub>7</sub><br>Acc - A<sub>8</sub>-A<sub>15</sub></td></tr>
<tr><td>IN r,(C)</td><td>r ← (C)<br>kizárólag<br>a jelzők<br>változnak<br>meg</td><td>•</td><td>↕</td><td>P</td><td>↕</td><td>0</td><td>↕</td><td>11 101 101<br>01 r 000</td><td>2</td><td>3</td><td>12</td><td>C - A<sub>0</sub>-A<sub>7</sub><br>B - A<sub>8</sub>-A<sub>15</sub></td></tr>
<tr><td>INI</td><td>(HL)←(C)<br>B ← B-1<br>HL←HL+1</td><td>•</td><td>↕&nbsp;①</td><td>X</td><td>X</td><td>1</td><td>X</td><td>11 101 101<br>10 100 010</td><td>2</td><td>4</td><td>16</td><td>C - A<sub>0</sub>-A<sub>7</sub><br>B - A<sub>8</sub>-A<sub>15</sub></td></tr>
<tr><td rowspan="2">INIR</td><td rowspan="2">(HL)←(C)<br>B ← B-1<br>HL←HL+1<br>Ismétlés<br>amíg B=0</td><td rowspan="2">•</td><td rowspan="2">1</td><td rowspan="2">X</td><td rowspan="2">X</td><td rowspan="2">1</td><td rowspan="2">X</td><td rowspan="2">11 101 101<br>10 110 010</td><td>2</td><td>5<br>(Ha B≠0)</td><td>21</td><td>C - A<sub>0</sub>-A<sub>7</sub><br>B - A<sub>8</sub>-A<sub>15</sub></td></tr>
<tr><td>2</td><td>4<br>(Ha B=0)</td><td>16</td><td></td></tr>
<tr><td>IND</td><td>(HL)←(C)<br>B ← B-1<br>HL←HL-1</td><td>•</td><td>↕&nbsp;①</td><td>X</td><td>X</td><td>1</td><td>X</td><td>11 101 101<br>10 101 010</td><td>2</td><td>4</td><td>16</td><td>C - A<sub>0</sub>-A<sub>7</sub><br>B - A<sub>8</sub>-A<sub>15</sub></td></tr>
<tr><td rowspan="2">INDR</td><td rowspan="2">(HL)←(C)<br>B ← B-1<br>HL←HL-1<br>Ismétlés<br>amíg B=0</td><td rowspan="2">•</td><td rowspan="2">1</td><td rowspan="2">X</td><td rowspan="2">X</td><td rowspan="2">1</td><td rowspan="2">X</td><td rowspan="2">11 101 101<br>10 111 010</td><td>2</td><td>5<br>(Ha B≠0)</td><td>21</td><td>C - A<sub>0</sub>-A<sub>7</sub><br>B - A<sub>8</sub>-A<sub>15</sub></td></tr>
<tr><td>2</td><td>4<br>(Ha B=0)</td><td>16</td><td></td></tr>
<tr><td>OUT (n),A</td><td>(n) ← A</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 010 011<br>← n →</td><td>2</td><td>3</td><td>11</td><td>n - A<sub>0</sub>-A<sub>7</sub><br>Acc - A<sub>8</sub>-A<sub>15</sub></td></tr>
<tr><td>OUT (C),r</td><td>(C) ← r</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>•</td><td>11 101 101<br>01 r 001</td><td>2</td><td>3</td><td>12</td><td>C - A<sub>0</sub>-A<sub>7</sub><br>B - A<sub>8</sub>-A<sub>15</sub></td></tr>
</tbody>
</table>

<!-- pdf 250 -->

<table>
<thead>
<tr><th rowspan="2">Mnemonik</th><th rowspan="2">Szimbolikus műv.</th><th colspan="6">Jelzők (flagek)</th><th>Műveleti kód</th><th rowspan="2">Byte szám</th><th rowspan="2">Mem. cikl. szám</th><th rowspan="2">"T" cikl. szám</th><th rowspan="2">Megjegyzés</th></tr>
<tr><th>C</th><th>Z</th><th>P/V</th><th>S</th><th>N</th><th>H</th><th>76 543 210</th></tr>
</thead>
<tbody>
<tr><td>OUTI</td><td>(C)←(HL)<br>B ← B-1<br>HL←HL+1</td><td>•</td><td>↕&nbsp;①</td><td>X</td><td>X</td><td>1</td><td>X</td><td>11 101 101<br>10 100 011</td><td>2</td><td>4</td><td>16</td><td>C - A<sub>0</sub>-A<sub>7</sub><br>B - A<sub>8</sub>-A<sub>15</sub></td></tr>
<tr><td rowspan="2">OTIR</td><td rowspan="2">(C)←(HL)<br>B ← B-1<br>HL←HL+1<br>Ismétlés<br>amíg B=0</td><td rowspan="2">•</td><td rowspan="2">1</td><td rowspan="2">X</td><td rowspan="2">X</td><td rowspan="2">1</td><td rowspan="2">X</td><td rowspan="2">11 101 101<br>10 110 011</td><td>2</td><td>5<br>(Ha B≠0)</td><td>21</td><td>C - A<sub>0</sub>-A<sub>7</sub><br>B - A<sub>8</sub>-A<sub>15</sub></td></tr>
<tr><td>2</td><td>4<br>(Ha B=0)</td><td>16</td><td></td></tr>
<tr><td>OUTD</td><td>(C)←(HL)<br>B ← B-1<br>HL←HL-1</td><td>•</td><td>↕&nbsp;①</td><td>X</td><td>X</td><td>1</td><td>X</td><td>11 101 101<br>10 101 011</td><td>2</td><td>4</td><td>16</td><td>C - A<sub>0</sub>-A<sub>7</sub><br>B - A<sub>8</sub>-A<sub>15</sub></td></tr>
<tr><td rowspan="2">OTDR</td><td rowspan="2">(C)←(HL)<br>B ← B-1<br>HL←HL-1<br>Ismétlés<br>amíg B=0</td><td rowspan="2">•</td><td rowspan="2">1</td><td rowspan="2">X</td><td rowspan="2">X</td><td rowspan="2">1</td><td rowspan="2">X</td><td rowspan="2">11 101 101<br>10 111 011</td><td>2</td><td>5<br>(Ha B≠0)</td><td>21</td><td>C - A<sub>0</sub>-A<sub>7</sub><br>B - A<sub>8</sub>-A<sub>15</sub></td></tr>
<tr><td>2</td><td>4<br>(Ha B=0)</td><td>16</td><td></td></tr>
</tbody>
</table>

65. ábra Be- és kiviteli utasítások

Megjegyzés: ① ha a B-1 eredménye zéró, akkor a Z-jelző 1 lesz, egyébként törlődik.

<!-- pdf 251 -->

![66. ábra A TVC tápegysége](_page_250_Figure_0.jpeg)

66. ábra A TVC tápegysége

<!-- pdf 252 -->

![67. ábra A TDA 1060 blokkvázlata](_page_251_Figure_0.jpeg)

67. ábra A TDA 1060 blokkvázlata

<!-- pdf 253 -->

# 9. Melléklet

A Mellékletben szereplő 30 db kapcsolási rajzot a VIDEOTON Számítástechnikai Gyára bocsátotta rendelkezésünkre.

Megjegyzés: A HBA-1 megnevezésű rajz a HBA és HBA-1 TVC 32k és 64k alapkártya rajza.

<!-- pdf 254 -->

267,- Ft

VIDEOTON

ELEKTRONIKAI VÁLLALAT SZÁMÍTÁSTECHNIKAI GYÁRA
