# Obnovení nativní analýzy pro MCP

Dosavadní fast-load načetl ověřený graf, ale znovu spustil všechny nativní
analýzy. Na zachované kopii Draivixu zabral tento opakovaný krok 27,21 s
a první užitečná odpověď MCP přišla za 44,90 s. Existující
`deterministic-findings.json` přitom obsahuje potřebné typované výsledky.

Manifest nyní může obsahovat jejich kontrolní součet. Při shodě vstupů,
sestavení, grafu a aktuální policy/doctrine se výsledky obnoví ze stejné
ověřené generace. Dead-code analýza se přepočítá kvůli možnému čtení mimo
parsovaný výřez. Externí výsledky použití cache vylučují. Konvergence, guard
a agentní kontext se stále sestavují z aktuální analýzy a ověřené baseline.
Podmínky a fallback popisuje [kontrakt](NATIVE_ANALYSIS_CACHE_CONTRACT.md).

| Skutečné načtení MCP | První užitečná odpověď | Výsledek |
| --- | ---: | --- |
| Předchozí sestavení, původní fast-load grafu | 44,90 s | Graf načten, nativní analýza přepočtena |
| Nové sestavení, snapshot předchozího sestavení | 74,16 s | Nesoulad engine odmítl cache, celý graf a analýza přepočteny |
| Stejné nové sestavení, aktuální snapshot | 16,61 s | Graf i nativní výsledky obnoveny |

Obnovení nativních výsledků v posledním běhu trvalo 78 ms. Oproti předchozímu
fast-load klesl jednotlivě naměřený čas první odpovědi přibližně o 63 %.
Nejde o statistický benchmark; všechny časy pocházejí ze sdíleného stroje.
Zbývající čas zahrnuje ověření generace, načtení grafu a sestavení MCP kontextu.
Paměťovou úsporu MCP jsme neměřili.

Produkční sestavení prošlo za 49,57 s bez varování. Stejná finální binárka
provedla plný CLI audit za 85,37 s při 3 580 056 KiB peak RSS. Zpracovala
17 067 podporovaných zdrojů, 132 657 symbolů, 1 097 074 referencí a 331 060
hran. Celý sémantický graf, dependency/evidence grafy, inventář kontraktů
a sekundární scanner jsou bajtově shodné s výsledkem předchozího sestavení.
Porovnání všech polí deterministických nálezů odlišilo pouze `timings`;
všechny souhrnné počty CLI zůstaly stejné, včetně 19 silných a 22 celkových
cyklických komponent. Scanner obsahuje stejných 10 031 nálezů.

Oba nové MCP běhy dokončily po 13 požadavcích bez protokolové chyby a skončily
0. Dotazy na tři lokální `FakePw`, jejich použití, návrh modulu, cykly,
coverage a quality mají celé shodné datové odpovědi. Každá třída má právě
svůj konstruktor. Rozdíly přehledu jsou cesty generace, čas freshness a
baseline guardu. Starší generace má `inputs_match_index: false` a důvod
`different_engine`; aktuální generace má `inputs_match_index: true` a tento
důvod nemá. Ostatní data guardu se zachovala. Baseline zůstává
`not_compared` kvůli neúplnému pokrytí, nikoli falešně potvrzená.

Všech 23 296 souborů zachované kopie stále souhlasí podle cest, velikostí
a SHA-256 s uloženým inventářem; konfigurace je také beze změny. Původní
Draivix nebyl upraven ani znovu zachycován. Vlastní audit stejnou binárkou
trval 3,11 s při 166 180 KiB RSS: 122 podporovaných souborů, dvě známá
zotavení Rust parseru a dva nepodporované instalační skripty. Sekundární
pokrytí vlastního projektu je úplné. Vlastní audit předchází poslednímu
doplnění tohoto dokumentu, dokladu a akceptačního odstavce.

Dva existující regresní scénáře byly rozšířeny o shodu nativních výsledků,
odmítnutí nesouvisejících počtů i při aktualizovaném hashi a vyloučení
externího důkazu. Nebyly spuštěny ani sestaveny jako testové cíle. Testy,
CI a lokální kontroly kvality zůstávají pozastavené. Běhové pozorování
dokládá odmítnutí staršího sestavení a obnovení aktuálního snapshotu;
změny konfigurace, poškození nálezů, externí evidence, změny doplňkových
dead-code vstupů a souběžné mutace mají v tomto kroku pouze kontrolu čtením
implementace. Neúplný Draivix navíc absence-based dead-code kontroly odkládá,
takže tento běh neprokazuje doplňkový průchod mimo výřez.

CLI nadále končí 1 kvůli neúplnému pokrytí: 80 zotavených zdrojů,
792 skriptově omezených Vue souborů a 146 nepodporovaných zdrojů. Fast-load
zůstává volitelný. Watcher byl vypnutý; inkrementalita změněných vstupů,
souběžní klienti, stabilita konfigurace/capture, schválené CI a celková
akceptace Q01–Q12 zůstávají otevřené.

[Strojový doklad](2026-09-10-analysis-restore-evidence.json) obsahuje otisky
implementace a binárky, porovnání celých artefaktů a polí, časy, manifest,
parametry a otisky všech skutečných MCP odpovědí i kontrolu integrity kopie.
