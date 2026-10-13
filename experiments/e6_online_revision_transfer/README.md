# Athlesia E6 — online revision and relational transfer

Futtatható, standard Python könyvtárra épülő, determinisztikus akciómodell-tanuló referencia. A tanuló nem használ neurális hálót, LLM-et, tanított numerikus súlyokat vagy külső szolgáltatást. A környezeti törvényt megfigyelt átmenetekből és választott kísérletekből azonosítja egy előre rögzített, kicsi hipotézisosztályon belül.

Ez az Athlesia repó izolált E6 kísérlete. Az első két elfogadási kaput valósítja meg és méri: maszkolt környezeti API; több, összekevert átlátszatlan akció hatásmodelljének külön indukciója; ellentmondásdetektálás; diszkriminatív aktív próba; minimális, akciólokális szabályrevízió; valamint szerkezeti transzfer. Nem világelsőségi, AGI-, robotikai vagy általános zero-shot bizonyítás.

## Indítás

Linux, Python 3.11 vagy újabb. Futásidejű külső Python-csomag nem kell.

```bash
python -m unittest discover -s tests -v
python -m athlesia.benchmark --cases 1 --output results/my_day2.json
python -m athlesia.benchmark --cases 100 --output results/my_day2_fuzz100.json
```

A parancsokat ebből a könyvtárból futtasd. A benchmark külön solver- és környezetfolyamatokat indít, JSONL üzeneteket közvetít, minden esetet naplóz. A `.json` összegzés, a hozzá tartozó `.jsonl` a részletes tényleges eseménynapló. A generált mérési fájlokat a Git szándékosan figyelmen kívül hagyja.

Opcionális statikus típusellenőrzés fejlesztői környezetben:

```bash
python -m pip install mypy
python -m mypy athlesia
```

## Mi történik a demonstrációban?

1. Hét megfigyelt átmenet érkezik három akcióhoz. A célt adó akciót az első demonstráció jelöli ki; a másik kettő csak általános `acts_on` kötést kap, hatásuk nincs a nyilvános gráfba kódolva.
2. A solver a before/after deltákból három külön hatássémát és három külön hipotézisteret épít. Az idempotens példák is megmaradnak bizonyítékként.
3. A broker négy, tanításkor nem látott vegyes állapotból kéri a két segédakció teljes utódállapotát, végrehajtja az akciókat a privát környezetben, és mezőnként ellenőrzi a jóslatokat. Ez a feketedobozos Day-2 sikerkritérium része.
4. A célakció három pozitív példája után több lehetséges előfeltételmodell marad; a legegyszerűbbet ideiglenesen használja.
5. Minden állapotcsatorna hamis. A várt kapunyitás elmarad. `CONTRADICTION_FOUND`.
6. A feltétel nélküli modell kiesik. A választott beavatkozások minimax módon választják szét a még konzisztens feltételmodelleket.
7. A nyertes guard-literált atomi módon hozzáadja kizárólag a megcáfolt akcióhoz. A két zavaró akció szabályának, hipotéziseinek és tanúinak kanonikus SHA-256 lenyomata változatlan.
8. Új világban minden objektum- és akcióazonosító, a megjelenítési méret, az elhelyezés és a felsorolási sorrend megváltozik. Az egzakt gráfmegfeleltetés után a tanult feltételt helyreállítja, majd az első feladatakcióval eléri a célt. A teljes tanult memória SHA-256 értéke a transzfer alatt változatlan.

**Pontos költség a 17-es seed háromcsatornás alapfutásában:** 7 demonstrációs akció; 4 feketedobozos predikció-ellenőrző akció; a revízióban 3 célakció-próba és 4 beállító/visszaállító akció; az új világban 1 előkészítés + 1 feladatakció. Összesen 20 állapotmódosító akció és 13 külön megfigyelő lekérés. Az „első próbálkozás” az új világ első célt elérni próbáló akciójára vonatkozik; az előkészítés és az ellenőrzés nem ingyenes.

## Ténylegesen mért eredmény

2026-10-13; Linux x86-64, AMD EPYC 9V74 szerver, Python 3.12.14. Egy logikai CPU engedélyezve az értékelőnek és öröklődően a gyermekfolyamatoknak. Ez nem i3-5005U mérés.

| Mutató | Eredmény |
|---|---:|
| Egység-/negatív teszt | 31 sikeres |
| Kevert, átnevezett világpárok, 3–5 állapotcsatorna | 100/100 siker |
| Indukált akciósémák esetenként | 3 |
| Ismeretlen vegyes állapotokon egzakt segédakció-jóslat | 400/400 |
| Solver teljes CPU-idő, medián | 80,715 ms |
| Solver teljes CPU-idő, p95 | 148,073 ms |
| Teljes eset falióraideje, folyamatindításokkal, medián | 1 201,322 ms |
| Teljes eset falióraideje, folyamatindításokkal, p95 | 1 693,271 ms |
| Solver legnagyobb peak RSS | 17 690 624 byte |
| Solver + környezet + broker peak RSS-ek legnagyobb összege | 53 071 872 byte |
| Szerializált tanult állapot, bizonyítékokkal | 7 632–16 879 byte |
| Diszkriminatív kísérletek száma | 2–5 |
| Mért solverforrás SHA-256 | `0c1a34b43d2a5d1fa4487a6c2f71f90723a508d33941962fcfebe988e9b1fbcb` |

Az összesített peak-RSS összeg konzervatív felső korlát, nem egy időpillanatban mért összeg. A szerializált memória mérete nem a folyamat RAM-használata. A JSONL auditnapló külön, lemezen van. A Gradio/frontend memóriaigénye nincs benne.

Az alap, háromcsatornás futásban a hét kevert trace három akciósémájának indukciója 33,459 ms, a revízió 8,398 ms, a transzfer 7,218 ms solver CPU-időt igényelt. A teljes solver-idő ezen felül tartalmazza a négy feketedobozos predikciós kérést. A statikus `mypy` ellenőrzést ezen a futtatón nem ismételtük meg, mert a csomag nincs telepítve; a fenti 31 futásidejű teszt és a benchmark ténylegesen lefutott. Nem közlünk elképzelt szubmilliszekundumos végponttól végpontig értéket.

## Mi van előre megadva, és mit tanul?

Előre adott: a szimbolikus érzékelő interfész, az objektum-/akció-/állapotcsatorna-típusok, a látható fizikai kapcsolatok, a beállító és visszaállító műveletek nyilvános jelentése, továbbá az a nyelvtan, hogy az ismeretlen hatást legfeljebb két konjunktív feltétel szabályozza.

Tanult: mely teljes állapotdelta tartozik mindhárom bemutatott akcióhoz; mely feltétel vagy feltételek szükségesek a célt adó akcióhoz a megengedett hipotézisosztályon belül; mely aktív próbával választhatók szét az aktuálisan lehetséges modellek. A solver a titkos helyes feltételeket és a két segédakció hatását nem kapja meg. A célakciót az első demonstráció jelöli ki; ez nem tanult célindukció.

Az állapotcsatornák műszerrel láthatók. **A rejtett dolog a törvény, nem maga az áramellátás állapotának megfigyelhetősége.** Nem adunk elő képfelismerést, predikátumfeltalálást vagy teljes részleges megfigyelhetőségű problémamegoldást.

Az új világban az azonosítók teljesen újak, de a nyilvános API relációtípusai közösek. Ha ezek jelentését és az összes akció szemantikáját is minden kapaszkodó nélkül felcserélnénk, garantált első-próbás transzfer nem lenne azonosítható. Azonos gráfalak eltérő belső törvénnyel szintén nem elég: külön negatív teszt bizonyítja ezt.

## Adatfolyam és izoláció

`protocol.py` a közös szerződés. `environment.py` és a titkos konfiguráció csak az értékelő oldalon él. `learner.py` kizárólag az `ActionPort.send_action(token)` metódust használja. Nincs `get_rules()`, seed, célprogram, kitüntetett „helyes feltétel”, jutalommagyarázat vagy belső hibaszöveg a solver bemenetén.

A solver ideiglenes könyvtárába a környezetmodult nem másoljuk át. Külön folyamatok, szűrt környezeti változók, külön címtér, szigorúan ellenőrzött JSON-séma, üzenet-/akció-/időkorlátok vannak. A titkos konfiguráció az environment saját induló stdin üzenetén érkezik, ezt a broker soha nem továbbítja a solvernek.

**Az alap subprocess mód megbízható kód adatfolyam-szeparációja; nem biztonsági sandbox rosszindulatú solver ellen.** Az azonos felhasználóval futó folyamatok, hostfájlok, `/proc` stb. miatt a Python-osztályhatár vagy a subprocess önmagában nem jelent hermetikus védelmet.

Ehhez mellékelve van egy külön solver Docker-image, amely nem tartalmaz környezetkódot:

```bash
docker build -f Dockerfile.solver -t athlesia-day1-solver .
python -m athlesia.benchmark --cases 1 --docker-solver --output results/docker_day2.json
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
- `tests/test_core.py`: 31 célzott teszt, köztük vegyes akciós, akciólokalitási és cáfoló kontrollok.
- `SPEC_HU.md`: matematikai specifikáció, kutatási audit, hatnapos protokoll.

## Határok

Ez korlátos logikai keresés: jelenleg legfeljebb 8 feltételjelöltből legfeljebb 2 literálos konjunkciók, legfeljebb 37 hipotézis. A kísérletválasztás legfeljebb 256 Boolean konfigurációt értékel. Az izomorfizmus-keresés 10 000 állapotnál megáll. Ez nem „keresés nélküli intelligencia”, hanem kicsi és auditálható keresőtér.

Nem veszünk el csendben régi bizonyítékot: 64 különböző normalizált tanú után a motor kapacitáshibával megáll. A véges memóriakorlátot elutasításokkal és határokkal tartjuk, nem korlátlan, veszteségmentes tanulás ígéretével.

A 100 eset ugyanazon deklarált feladatcsalád átnevezett példáit vizsgálja. Mindegyik eset három akciómodellt tartalmaz, de ez továbbra sem 100 egymástól független új iparág vagy mechanizmus. A próba nem demonstrál LLM-ekkel szembeni fölényt, általános robotvezérlést vagy új tudományos elsőbbséget.

## Day 3 — Online transition validation

A Day-3 helyi, reprodukálható ellenőrzési jegyzőkönyv: [`DAY3_EVIDENCE.md`](DAY3_EVIDENCE.md).
Ez a mérés az E6 izolált akciómodell-tanulási feladatcsaládra vonatkozik, nem az ARC-AGI-3 hivatalos értékelésére.
A futtatott parancsok, a géptől függő CPU-/memóriaadatok és az eseményszámok a jegyzőkönyvben vannak.
