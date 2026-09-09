# Potvrzení uložených editací a ověřování revize MCP

Agent nyní může po uložení změn zavolat `record_changed_paths` a získanou revizi
předat `verify_change`. Potvrzení invalidace nečeká na dokončení indexu ani na
doručení filesystemové události. Na kopii Draivixu trvaly dvě skutečné RPC odpovědi
0,393 a 0,387 ms, měřeno klientem včetně dekódování a uložení odpovědi.
Následné ověření správně odlišilo starý index od požadované
nové revize. Jde o jednotlivá pozorování, nikoli garantovanou odezvu.

[Kontrakt](MCP_EDIT_RECEIPTS.md) popisuje použití, validaci cest, čekání a meze
záruky. [Strojový doklad](2026-09-10-edit-receipts-evidence.json) obsahuje otisky
binárky a zdrojů, skutečné požadavky a odpovědi, porovnání dat a kontrolu kopie.
Celková akceptace Q01–Q12 zůstává otevřená; testy a CI jsou nadále zastavené.

Změna řeší okno pozorované při
[předchozím měření](2026-09-09-incremental-scanning.md): okamžitý dotaz po zápisu
mohl vrátit starý index s `is_stale: false`, protože watcher ještě nezaznamenal
událost. Nové explicitní hlášení po uložení přidělí revizi a probudí společného
indexera. Pracovní úloha s dřívějším cílem nemůže svou publikací splnit novou
revizi potvrzení. `verify_change` přebírá `min_revision` a `wait_ms` na společné
hranici požadavku a pracuje s jedním připnutým snímkem.

Validace přijímá nejvýše 128 relativních cest, normalizuje oddělovače a duplicity
a odmítá prázdné, nadlimitní, absolutní či průchodové cesty a NUL. Celý vstup se
ověří před invalidací. Služba bez aktivního watch indexera potvrzení nevydá.
Neprobíhá zápis zdrojových souborů ani změna pravidel rozsahu scanu.

Chyba analýzy je nově spojená s revizí konkrétního pokusu. Staré selhání proto
neukončí čekání na novější, již nahlášenou opravu jako na definitivně selhaný stav.
Diagnostika zůstává viditelná do úspěšné publikace. Tato větev má napsanou regresi;
vynucený chybový scénář nebyl za trvajícího zastavení testů spuštěn.

Ověřovací API zároveň zveřejňuje skutečný stav a identitu artefaktové baseline.
Původní text nesprávně sliboval porovnání posledních dvou analýz, přestože baseline
nemusí odpovídat předchozí watch revizi. Opravené filtrování respektuje komponenty
cest místo podřetězců. Odvozený rozsah používá všechny čekající cesty, nikoli jen
jejich padesátipoložkový náhled; `scope_path_count` a `truncated` popisují ořez
zobrazeného rozsahu. Čekající cesty po indexaci mohou zmizet, proto klient předává
cesty vrácené potvrzením výslovně.

Ověření používá oddělenou kopii zachycení 23 296 souborů s 17 067 podporovanými
zdroji. Stejně jako u předchozího porovnání začíná se dvěma staršími zachycenými
soubory Mitel, které postupně nahrazuje novějšími verzemi:

| Situace | Skutečná odpověď finální binárky |
| --- | --- |
| Uložení `Task.hooks.php` během prvotního indexování | Potvrzení vrací `min_revision: 2`, index 0, watcher `starting`, jednu čekající cestu a `is_stale: true`. RPC trvá 0,393 ms. |
| Bezprostřední `verify_change` s revizí 2 a `wait_ms: 0` | Explicitní `indexing`, `retryable: true`, nesplněná konzistence; nevzniká fiktivní ověřovací výsledek. |
| Čekání na revizi 2 | Přijde index 2 se splněnou konzistencí a bez stale. Od začátku zápisu včetně prvotní analýzy uplynulo 76,04 s. |
| Uložení `MitelShowcaseFixtureService.php` po první publikaci | Potvrzení vrací revizi 3 při indexu 2 a okamžitě značí stale; RPC trvá 0,387 ms. Čítač se v tomto okamžiku zvýšil právě explicitním hlášením. |
| Bezprostřední ověření revize 3 bez čekání | Vrací starý index 2 s `consistency_satisfied: false`, `is_stale: true` a výslovným sdělením, že ověření změny není dokončené. |
| Čekání na revizi 3 | Vrátí se index 6 se splněnou konzistencí; aktualizace trvá 58,54 s. Čísla revizí zahrnují také filesystemové události, nejsou počtem editací. |

Obě dokončená ověření mají `baseline.comparison: initial_snapshot`,
`previous: null` a nulové počty regresí i oprav. Výstupní adresář tohoto běhu
neobsahoval baseline a server pracoval s `--no-write`. Nula proto neznamená, že
se zdroj nezměnil nebo že byl prokázán čistý výsledek. Guard zůstává `block`;
pokrytí nadále uvádí 80 zotavených zdrojů, 792 skriptově omezených Vue souborů
a 146 nepodporovaných zdrojů.

Celkem proběhlo 26 požadavků: 25 běžných odpovědí a jedna očekávaná explicitní
odpověď o dosud nehotovém indexu. Proces skončil 0. Použitá cache sekundárního
scanu při startu zpracovala všech 17 067 zdrojů a při druhé změně jeden soubor;
zbývajících 17 066 výsledků použila znovu. Fast-load a cache resolveru byly vypnuté.

Pro oba dosažené zdrojové stavy bylo porovnáno devět úplných MCP pohledů s
předchozím během bez editových potvrzení: overview, findings, coverage, quality,
contracts, guard, convergence, graph packets a repository topology. Po nahrazení
známých kořenů kopií a oddělení freshness/pracovních čítačů se shoduje deset
z osmnácti pohledů. Zbývajících osm se liší pouze výslovně uvedeným otiskem jiné
binárky, otiskem konfigurace zahrnujícím absolutní cesty nebo časem vytvoření
topologie. Po oddělení těchto metadat se shoduje všech osmnáct; pole a jejich
pořadí jsou zachované. Nejde o porovnání celého surového sémantického grafu.

Po finálním běhu souhlasí všech 23 296 cest, velikostí a SHA-256 kopie se
zachovaným novým korpusem, stejně jako scan a doctrine konfigurace. Zachovaný
snímek zůstal beze změny. Do původního Draivixu tato práce nezapisovala.
Přechodové kopie nejsou úplné historické revize a nevypovídají o pozdějších
změnách živého checkoutu.

Finální produkční sestavení prošlo za 53,30 s. Vlastní studený audit trval 2,96 s
při 160 408 KiB RSS; všech 119 sekundárních vstupů má stav scan/prefilter bez
mezery a původní falešné identitní nálezy vlastněné `artifacts.rs` zůstávají nulové.
Analytické CLI skončilo 1 kvůli nativní neúplnosti; nejde o úspěšnou úplnou akceptaci.

Sedm nových regresí pokrývá cesty, neaktivního indexera, koalescenci probuzení,
chybu staršího pokusu, prvotní indexování a platné srovnání s baseline včetně
rozsahu přes padesát cest. Jedna původní regrese byla opravena: první snímek bez
baseline již nesmí očekávat regresi. Tyto testy nebyly spuštěny. Zbývají schválené
CI, chybové a souběžné scénáře, stabilita vstupů během capture a úplná akceptace
Q05/Q06 i ostatních Q01–Q12. Potvrzení řeší vlastní uložené editace při dodržení
protokolu; není univerzální synchronizací disku ani důkazem pokrytí každé uvedené cesty.
