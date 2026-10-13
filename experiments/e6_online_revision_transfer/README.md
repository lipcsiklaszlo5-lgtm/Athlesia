# Athlesia E6 — online revision and relational transfer

Futtatható, standard Python könyvtárra épülő, determinisztikus akciómodell-tanuló referencia. A tanuló nem használ neurális hálót, LLM-et, tanított numerikus súlyokat vagy külső szolgáltatást. A környezeti törvényt megfigyelt átmenetekből és választott kísérletekből azonosítja egy előre rögzített, kicsi hipotézisosztályon belül.

Ez az Athlesia repó izolált E6 kísérlete. Az első elfogadási kaput valósítja meg és méri: maszkolt környezeti API, akciómodell-indukció, ellentmondásdetektálás, diszkriminatív aktív próba, minimális lokális szabályrevízió és szerkezeti transzfer. Nem világelsőségi, AGI-, robotikai vagy általános zero-shot bizonyítás.

## Indítás

Linux, Python 3.11 vagy újabb. Futásidejű külső Python-csomag nem kell.

```bash
python -m unittest discover -s tests -v
python -m athlesia.benchmark --cases 1 --output results/my_day1.json
python -m athlesia.benchmark --cases 100 --output results/my_fuzz100.json
```

A parancsokat ebből a könyvtárból futtasd. A benchmark külön solver- és környezetfolyamatokat indít, JSONL üzeneteket közvetít, minden esetet naplóz. A `.json` összegzés, a hozzá tartozó `.jsonl` a részletes tényleges eseménynapló. A `results/day1.*` és `results/fuzz100.*` a fejlesztés során ténylegesen lefutott mérések.

Opcionális statikus típusellenőrzés fejlesztői környezetben:

```bash
python -m pip install mypy
python -m mypy athlesia
```

## Mi történik a demonstrációban?

1. Három megfigyelt sikeres akcióból változatlan hatást tanul. Négy lehetséges előfeltételmodell még megmarad; a legegyszerűbbet ideiglenesen használja.
2. Mindkét állapotcsatorna hamis. A várt kapunyitás elmarad. `CONTRADICTION_FOUND`.
3. A feltétel nélküli modell kiesik. A megmaradó hipotézisek: A feltétel, B feltétel, A ÉS B.
4. A választott kísérlet A-t igazra állítja, B-t hamisan hagyja, majd megismétli az ismeretlen akciót. A siker csak az A-feltételes modellel egyezik.
5. Egy új literált atomi módon hozzáad. A korábbi három sikeres példát és az összes új bizonyítékot ellenőrzi.
6. Új világban minden objektum- és akcióazonosító, a megjelenítési méret, az elhelyezés és a felsorolási sorrend megváltozik. Az egzakt gráfmegfeleltetés után a tanult feltételt helyreállítja, majd az első feladatakcióval eléri a célt. A tanult memória SHA-256 értéke változatlan.

**Pontos költség a mellékelt alapfutásban:** 3 demonstrációs akció; 1 cáfolt próbálkozás; 1 beállítás + 1 kísérleti próbálkozás; az új világban 1 előkészítés + 1 feladatakció. Összesen 8 állapotmódosító akció, 5 megfigyelő lekérés. Az „első próbálkozás” az első célt elérni próbáló akcióra vonatkozik, az előkészítés nem ingyenes.

## Ténylegesen mért eredmény

2026-10-13; Linux x86-64, AMD EPYC 9V74 szerver, Python 3.12.14. Egy logikai CPU engedélyezve az értékelőnek és öröklődően a gyermekfolyamatoknak. Ez nem i3-5005U mérés.

| Mutató | Eredmény |
|---|---:|
| Egység-/negatív teszt | 28 sikeres |
| Mypy strict | 8 modul, 0 hiba |
| Átnevezett világpárok, 2–4 állapotcsatorna | 100/100 siker |
| Solver teljes CPU-idő, medián | 16,211 ms |
| Solver teljes CPU-idő, p95 | 26,985 ms |
| Teljes eset falióraideje, folyamatindításokkal, medián | 311,201 ms |
| Solver legnagyobb peak RSS | 15 466 496 byte |
| Solver + környezet + broker peak RSS-ek legnagyobb összege | 46 399 488 byte |
| Szerializált tanult állapot, bizonyítékokkal | 3 618–6 582 byte |
| Kísérletek száma a különböző esetekben | 1–4 |

Az összesített peak-RSS összeg konzervatív felső korlát, nem egy időpillanatban mért összeg. A szerializált memória mérete nem a folyamat RAM-használata. A JSONL auditnapló külön, lemezen van. A Gradio/frontend memóriaigénye nincs benne.

Az alap, kétcsatornás egyetlen futásban a tanítás 4,658 ms, a revízió 2,372 ms, a transzfer 2,115 ms solver CPU-időt igényelt. Ezek tartalmaznak Python- és protokollfeldolgozási költséget. Nem közlünk elképzelt szubmilliszekundumos végponttól végpontig értéket.

## Mi van előre megadva, és mit tanul?

Előre adott: a szimbolikus érzékelő interfész, az objektum-/akció-/állapotcsatorna-típusok, a látható fizikai kapcsolatok, a beállító és visszaállító műveletek nyilvános jelentése, továbbá az a nyelvtan, hogy az ismeretlen hatást legfeljebb két konjunktív feltétel szabályozza.

Tanult: mely állapotváltozás tartozik az ismeretlen feladatakcióhoz; mely feltétel(ek) szükséges(ek) a megengedett hipotézisosztályon belül; mely aktív próbával választhatók szét az aktuálisan lehetséges modellek. A solver a titkos helyes feltételeket nem kapja meg.

Az állapotcsatornák műszerrel láthatók. **A rejtett dolog a törvény, nem maga az áramellátás állapotának megfigyelhetősége.** Nem adunk elő képfelismerést, predikátumfeltalálást vagy teljes részleges megfigyelhetőségű problémamegoldást.

Az új világban az azonosítók teljesen újak, de a nyilvános API relációtípusai közösek. Ha ezek jelentését és az összes akció szemantikáját is minden kapaszkodó nélkül felcserélnénk, garantált első-próbás transzfer nem lenne azonosítható. Azonos gráfalak eltérő belső törvénnyel szintén nem elég: külön negatív teszt bizonyítja ezt.

## Adatfolyam és izoláció

`protocol.py` a közös szerződés. `environment.py` és a titkos konfiguráció csak az értékelő oldalon él. `learner.py` kizárólag az `ActionPort.send_action(token)` metódust használja. Nincs `get_rules()`, seed, célprogram, kitüntetett „helyes feltétel”, jutalommagyarázat vagy belső hibaszöveg a solver bemenetén.

A solver ideiglenes könyvtárába a környezetmodult nem másoljuk át. Külön folyamatok, szűrt környezeti változók, külön címtér, szigorúan ellenőrzött JSON-séma, üzenet-/akció-/időkorlátok vannak. A titkos konfiguráció az environment saját induló stdin üzenetén érkezik, ezt a broker soha nem továbbítja a solvernek.

**Az alap subprocess mód megbízható kód adatfolyam-szeparációja; nem biztonsági sandbox rosszindulatú solver ellen.** Az azonos felhasználóval futó folyamatok, hostfájlok, `/proc` stb. miatt a Python-osztályhatár vagy a subprocess önmagában nem jelent hermetikus védelmet.

Ehhez mellékelve van egy külön solver Docker-image, amely nem tartalmaz környezetkódot:

```bash
docker build -f Dockerfile.solver -t athlesia-day1-solver .
python -m athlesia.benchmark --cases 1 --docker-solver --output results/docker_day1.json
```

A futtató read-only fájlrendszert, hálózat nélküli konténert, nem root felhasználót, capability-elvonást, PID- és memóriakorlátot kér. Nincs hostvolume vagy Docker-socket a solverben. Csak stdin/stdout IPC-t kap. A konténerhálózattól független host-broker közvetít.

**A Docker módot ezen a gépen nem tudtuk lefuttatni, mert nincs Docker.** A telepített bubblewrap teljes hálózati izolációs próbáját a host tiltotta. A mellékelt eredmények ezért kifejezetten `trusted_code_subprocess_protocol_only` besorolásúak. Független biztonsági audit és valódi konténeres futás előtt ne állítsuk, hogy adversariális, hermetikus sandboxot igazoltunk.

## Fájlok

- `athlesia/protocol.py`: immutable, típusos adatmodell és szigorú wire parser.
- `athlesia/graph.py`: színfinomítás, korlátos CSP, teljes élellenőrzés, kétértelműség kezelése.
- `athlesia/learner.py`: hatásindukció, hipotézishalmaz, aktív kísérletválasztás, lokális atomi revízió, transzfer.
- `athlesia/environment.py`: az értékelő privát determinisztikus fixture-generátora.
- `athlesia/worker.py`: két külön JSONL worker; solverágban nincs environment-import.
- `athlesia/benchmark.py`: megbízható értékelő, közvetítő, folyamatindítás, erőforrásmérés.
- `athlesia/runtime.py`: egy-CPU affinitás és Linux címtérkorlát.
- `tests/test_core.py`: 28 célzott teszt, köztük cáfoló kontrollok.
- `SPEC_HU.md`: matematikai specifikáció, kutatási audit, hatnapos protokoll.

## Határok

Ez korlátos logikai keresés: jelenleg legfeljebb 8 feltételjelöltből legfeljebb 2 literálos konjunkciók, legfeljebb 37 hipotézis. A kísérletválasztás legfeljebb 256 Boolean konfigurációt értékel. Az izomorfizmus-keresés 10 000 állapotnál megáll. Ez nem „keresés nélküli intelligencia”, hanem kicsi és auditálható keresőtér.

Nem veszünk el csendben régi bizonyítékot: 64 különböző normalizált tanú után a motor kapacitáshibával megáll. A véges memóriakorlátot elutasításokkal és határokkal tartjuk, nem korlátlan, veszteségmentes tanulás ígéretével.

A 100 eset ugyanazon deklarált feladatcsalád átnevezett példáit vizsgálja. Nem 100 egymástól független új iparág vagy mechanizmus. A próba nem demonstrál LLM-ekkel szembeni fölényt, általános robotvezérlést vagy új tudományos elsőbbséget.
