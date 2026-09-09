# Opětovné použití sekundárního scanu při změnách Draivix

Volitelná cache sekundárního scanneru zkrátila dvě pozorované aktualizace velkého
snímku Draivix z 80,11/79,18 s na 57,74/58,74 s. Při každé změně přepočítala jeden
soubor a použila úplné výsledky ostatních 17 066 zdrojů. Obsah porovnaných MCP
pohledů odpovídá běhu bez cache po oddělení výslovně uvedených metadat.
Paměť zůstává přibližně stejná; celá interaktivní odezva je stále pomalá.

Zapnutí: `AIGISCORE_INCREMENTAL_SCAN=1 aigiscode mcp /repo --watch`.
Výchozí stav je vypnuto, protože schválené diferenční CI dosud neběželo.
Cache je nezávislá na `AIGISCORE_INCREMENTAL_RESOLVE`; při těchto měřeních byl
resolver cache i fast-load vypnutý v obou bězích. Měřena je stejná produkční
binárka, nikoli dva různé vývoje detektoru. Otisky, skutečné požadavky a výstupy,
zdrojové změny i měření jsou v [dokladu](2026-09-09-incremental-scanning-evidence.json).

Scanner uchovává kompletní výsledek pro relativní cestu a otisk skutečně načteného
obsahu: pozitivní i negativní nálezy, prefiltry, použité rule IDs, příznaky scanu
a omezení pokrytí. Při změně obsahu či cesty znovu provede běžný scan; odstraněné
nebo vyloučené cesty zahodí. Pravidla a gramatiky jsou pevnou součástí procesu,
cache se neukládá na disk a po pádu pracovní úlohy se zahodí. Nový nebo změněný
velký soubor stále projde sekvenční větví. Úplný výsledek se sestavuje společnou
cestou s běžným scanem, v pořadí aktuálních vstupů a se stejným řazením nálezů.

Počítadla `repo_overview.ast_grep_work` odlišují skutečně provedenou práci od
pokrytí. Například `scanned_files` dál zahrnuje platné výsledky použitých AST scanů;
neznamená počet nově postavených stromů v této revizi. Zachované mezery Vue ani
nepodporované vstupy se nemění na čistý výsledek. Globální graf, kontrakty,
bezpečnostní a architektonické hodnocení, doctrine a review se přepočítávají.
Podrobnosti určuje [kontrakt scanneru](SECONDARY_COVERAGE_CONTRACT.md).

Obě oddělené pracovní kopie vycházejí z nového zachycení 23 296 souborů se
17 067 podporovanými zdroji. Pouze dva soubory v počátečním stavu mají starší,
dříve zachycený obsah. Postupně dostávají novější zachycené verze:

| Změna | Zdrojový a pozorovaný dopad |
| --- | --- |
| `clients/mitel/Entities/Task/Task.hooks.php` | Starý obsah 35 567 bytů nahrazuje současných 883 bytů. Počet surových stop scanneru v celém korpusu klesá z 10 039 na 10 029 a mizí stará vnořená smyčka tohoto souboru z review. |
| `clients/mitel/Services/MitelShowcaseFixtureService.php` | Soubor roste ze 112 737 na 191 010 bytů, tedy přechází přes hranici pro sekvenční scan. Surové stopy přibývají na 10 031; review správně ukazuje nové kotvy řazení na 1398 a členství na 3432. Jde o přípravu ukázkových dat, nikoli důkaz pomalé produkční odezvy. |

Po druhé změně obě kopie přesně odpovídají všem 23 296 cestám, velikostem a
SHA-256 nového zachycení. Shodují se i scan a doctrine konfigurace. Původní
Draivix nebyl upraven, zachované zdrojové snímky zůstaly beze změny. Mezistavy
jsou řízené kombinace skutečných souborů pro sledování aktualizací; nejde o úplné
historické revize ani o tvrzení, že se živý checkout mezitím nemohl změnit.

| Pozorování | Úplný scan | Použití cache |
| --- | ---: | ---: |
| První použitelný přehled od startu procesu | 75,66 s | 76,21 s |
| Aktualizace hooku od zápisu do čerstvé odpovědi | 80,11 s | 57,74 s |
| Sekundární fáze při změně hooku | 20,933 s | 0,040 s |
| Aktualizace velkého souboru | 79,18 s | 58,74 s |
| Sekundární fáze při změně velkého souboru | 21,254 s | 0,607 s |
| RSS při odpovědi po poslední změně | 4 587 820 KiB | 4 577 000 KiB |
| Pozorovaná špička procesu | 4 966 992 KiB | 4 978 020 KiB |

Každý proces obsloužil 37 skutečných požadavků bez protokolové chyby. Pro všechny
tři stavy byly porovnány přehled, úplný seznam viditelných nálezů, coverage,
quality, contracts, guard, convergence, graph packets, repository topology
a podrobnosti nálezů vztahujících se ke dvěma měněným souborům. Celkem jde o
30 porovnání celých zachycených datových pohledů. Seznamy nálezů nebyly zkrácené
a obsahovaly postupně 4 182, 4 181 a 4 183 viditelných záznamů.

Porovnání řadí klíče objektů a zachovává úplná pole i jejich pořadí. Nahrazuje
známé odlišné kořeny kopií a výstupů; freshness a počítadla provedené práce se
posuzují zvlášť. Takto se shoduje 18 z 30 pohledů. Zbývajících 12 má navíc pouze
odlišný čas vytvoření topologie nebo otisk resolver konfigurace, do kterého
`ConfigReader::fingerprint` záměrně zahrnuje absolutní cesty. Po vyloučení těchto
konkrétních polí se shoduje všech 30. Doklad zachovává i původní rozdíly a hashe;
nejde o tvrzení o bajtové shodě všech odpovědí nebo celého surového scanneru.

Okamžité dotazy po zápisu v obou bězích ještě vrátily předchozí revizi s
`is_stale: false`, protože watcher dosud nezaznamenal souborovou událost.
Měření proto výslovně čekalo na `min_revision` nejméně o jednu vyšší než před
zápisem a na splněnou konzistenci. Freshness vyjadřuje pozorované změny; samotný
okamžitý dotaz bez známé nové revize nedokládá shodu s právě zapsaným diskem.
Revize se posunuly 1 → 5 → 8 a 1 → 4 → 8; jejich rozdíl není počet editací.
Toto pozorování nezavírá zbývající akceptaci časových oken a chyb watcheru v Q05.

Časy jsou jednotlivá pozorování na sdíleném stroji, s diagnostickým trasováním
v obou bězích. Paměť byla vzorkována po 200 ms a při získání jednotlivých revizí;
tabulka uvádí nejvyšší zachycené kernelové `VmHWM`. Čtené MCP pohledy jsou součástí
zatížení. Nejde o statistický benchmark, měření více současných klientů ani
prokázaný paměťový přínos.

Produkční sestavení prošlo. Vlastní studený audit AigisCode trval 3,14 s při
156 612 KiB RSS; všech 118 sekundárních vstupů mělo pokrytí bez mezery a původní
falešné identitní nálezy v `artifacts.rs` zůstaly nulové. Nativní úplnost tím není
prokázaná a vlastní analytické CLI nadále končí 1.

Tři napsané diferenční regrese pokrývají změnu pozitiv/negativ, přidání a odstranění
vstupů, změnu cesty a jazyka, konfigurační hranici, velké soubory, pořadí vstupů
a omezení Vue. Nebyly spuštěny; automatické testy a CI zůstávají zastavené podle
Davidova pokynu. Úplnou shodu surových scanner výsledků při všech těchto změnách,
kombinaci s fast-load a resolver cache i chybové scénáře musí potvrdit schválené CI.
Inkrementální parsování a globální hodnocení, souběžná zátěž a celkové Q01–Q12
zůstávají otevřené. Tato změna řeší konkrétní opakovanou práci sekundární fáze.
