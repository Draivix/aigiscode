# AigisCode: opakovaný hloubkový audit kvality

## Závěr

AigisCode je použitelný pro asistované mapování a rozbor architektury velkých
repozitářů. Poslední revize prokazatelně zlepšila důkazní cesty cyklů, rozlišení
závislostí a rychlost. Pro samostatné rozhodování o bezpečnosti, mazání kódu nebo
blokování změn však dosud nemá dostatečně spolehlivou analýzu ani provozní kontrakt.
Pozici „#1“ současné důkazy nepotvrzují.

Největší další přínos přinese odstranění falešně úspěšných výsledků a chyb v
existujících závěrech. Rozšiřování počtu detektorů před těmito opravami by zvýšilo
množství výstupů, kterým zatím nelze bez kontroly důvěřovat.

## Rozsah a ověření

Na Davidův pokyn byla větev `feat/audit-trust-and-scale` sloučena posunem `main`
bez konfliktů. `origin/main` přešel z `72953e0` na `717c2e7`. Audit níže hodnotí
produktový kód tohoto commitu. Samotný audit nemění implementaci.

Podklady:

- Nové skutečné spuštění `aigiscode analyze` nad vlastním repozitářem; kontrola
  vygenerovaných grafů, nálezů, guard rozhodnutí a rozsahu sekundárního scanneru.
- Čtení implementace parserů, resolveru, assessment vrstvy, externích integrací,
  publikace artefaktů, CLI, MCP a watcheru včetně jejich chybových větví.
- Nové CI po merge a porovnání diagnostik s posledním CI původního `main`.
- Již provedené měření na stejných 17 013 souborech Draivix a skutečné MCP dotazy;
  jejich výsledky jsou [v předchozím měření](2026-09-09-audit-revision.md).

Lokální automatizované testy, lint ani externí quality-gate scannery se
nespouštěly. Nový běh byl uživatelem požadovaný produktový audit. Dynamické
chybové scénáře níže nebyly přehrány jako lokální testovací sada; takové závěry
jsou označené jako kontrola zdroje. Draivix nebyl upraven.

[Strojový záznam](2026-09-09-quality-reaudit-evidence.json) uchovává fingerprint
zdrojového manifestu, hash použité binárky, metriky, konkrétní falešné hrany,
nálezy z testovacích ukázek a srovnání CI. Kompletní lokální artefakty jsou pod
`target/quality-reaudit-2026-09-09/`.

| Oblast | Stav a praktický význam |
| --- | --- |
| Vysvětlení cyklů | Výrazně lepší: skutečné uzavřené cesty, původ hran, oddělený silný pohled. |
| Sémantická správnost | Neúplná: nový vlastní audit stále našel nemožné vazby mezi jazyky. |
| Signál architektonických nálezů | Smíšený: některé vysoké nálezy vznikají z testovacích textů. |
| Bezpečnostní integrace | Blokující chyba důvěry: chyba scanneru může skončit jako `Passed`. |
| MCP při změnách | Počáteční dotazy fungují; živá čerstvost má nepokryté změny a chyby watcheru. |
| Výkon dávkového auditu | Naměřený významný posun; paměť a objem artefaktů zůstávají vysoké. |
| Regresní ověření | CI je červené; testovací krok se nespustil. |
| Udržovatelnost | Typované vrstvy pomáhají, ale velké moduly a duplicitní převody zvyšují riziko vynechání důkazů. |

## Konkrétní otevřené nálezy

### Q01 — P1: importy stále mohou navázat nesouvisející jazyk

**Pozorováno v novém grafu.** `website/src/pages/LanguagePage.tsx:21` a
`LanguagesPage.tsx:18` importují typ `Language` z `@/content/languages`. Obě
hrany míří na Rust enum `Language` v `rust/crates/aigiscore/src/graph/mod.rs`.
Správná deklarace je TypeScript interface v `website/src/content/languages.ts:5`.
Chybné hrany mají `TypeImport`, `Global`, `Inferred`, confidence 500: silný graf
nepoškozují, ale plný graf a navigace stále obsahují falešnou vazbu.

[`resolve_reference`](../rust/crates/aigiscore/src/resolve/mod.rs#L356) vrací
importy zvláštní cestou před jazykovým filtrem. `resolve_import_reference`
pak může použít globální shodu podle jména. Současně
[`load_resolve_config`](../rust/crates/aigiscore/src/resolve/mod.rs#L308) načítá
jen konfigurace v kořeni auditu, takže přehlédne skutečný
`website/tsconfig.json`. Načítání tsconfig používá striktní JSON a nezpracovává
`extends` ani project references. Toto je zásadní mezera pro monorepozitáře.

**Podmínka opravy:** statické importy musí respektovat jazykovou rodinu a hranici
balíčku. Konfigurace musí odpovídat importujícímu souboru. Nevyřešený alias nesmí
vytvořit hranu na náhodný globální symbol. Ověřit správný cíl těchto dvou importů,
nikoli pouze zmizení chybných hran.

### Q02 — P1: testovací ukázky vytvářejí architektonický dluh produkce

**Pozorováno ve výstupu a ověřeno ve zdroji.** Nový vlastní audit hlásí
`Split identity model` a vysoce závažný `Compatibility scar` nad `artifacts.rs`
kvůli názvům `assignedUser`, `assignedUserId`, `assigned_user_id` a jejich metodám.
Tyto názvy jsou v Python ukázkách uvnitř Rust testu na
[`artifacts.rs:8593`](../rust/crates/aigiscore/src/artifacts.rs#L8593).
Nejde o identitní model produkčního AigisCode. Nález compatibility scar navíc
odkazuje na `artifacts.rs:232`, kde použitá identitní ukázka není.

[`detect_split_identity_models`](../rust/crates/aigiscore/src/assessment/mod.rs#L2008)
používá vlastní jazykově nerozlišené maskování textu. Repo již má
`lexmask`, včetně Rust raw stringů; oba mechanismy se rozešly. Oddělení testů podle
cesty nestačí pro Rust `#[cfg(test)]` moduly ve stejném souboru.

**Podmínka opravy:** identifikátory a primární kotvy musí pocházet ze skutečného
kódu v odpovídajícím jazyce a rozsahu. Testovací ukázka má zmizet z produkčního
nálezu, zatímco skutečně rozporné produkční identifikátory musí zůstat zjistitelné.

### Q03 — P1: chyba externího scanneru může být prezentována jako úspěch

**Kontrola zdroje.**
[`parse_sarif_output`](../rust/crates/aigiscore/src/external/mod.rs#L961) vrací
prázdný seznam pro prázdný i neplatný SARIF.
[`completed_run`](../rust/crates/aigiscore/src/external/mod.rs#L805) pak nastaví
`Passed`, pokud je seznam prázdný, bez vyhodnocení `exit_code`.
[`run_sarif_tool`](../rust/crates/aigiscore/src/external/mod.rs#L235) tyto dvě
funkce spojuje i pro proces ukončený chybou. Exit code a stderr mohou být
uchované, ale autoritativní status jim odporuje.

**Podmínka opravy:** rozlišit validní čistý výsledek, nálezy, nepoužitelný výstup,
nedostupnost, timeout a chybu nástroje. Exit semantics patří jednotlivým
adaptérům; prosté pravidlo „nenulový exit = chyba“ také nestačí. Selhání nesmí
zlepšit celkové bezpečnostní hodnocení. Ověření patří do CI s reálnými procesy
a formáty, bez náhradních úspěšných odpovědí.

### Q04 — P1: větší stdout/stderr může zablokovat externí scanner

**Kontrola zdroje.**
[`run_command`](../rust/crates/aigiscore/src/external/mod.rs#L903) přesměruje oba
výstupy do pipes, opakovaně čeká na ukončení procesu a teprve potom volá
`wait_with_output`. Proces, který zaplní pipe, čeká na čtenáře; rodič čeká na
ukončení procesu. Platný větší výstup tak může skončit timeoutem. Timeout navíc
ukončuje přímé dítě, nikoli výslovně celý strom potomků.

**Podmínka opravy:** průběžně číst oba výstupy, omezit paměť a uchovat úplné raw
artefakty. Zajistit úklid procesů i při chybě. Toto je zvlášť podstatné právě
pro scannery nad velkými repozitáři.

### Q05 — P1: watcher nedokládá čerstvost všech vstupů analýzy

**Kontrola zdroje.**
[`is_ignored`](../rust/crates/aigiscore/src/mcp/watch.rs#L59) ignoruje všechny
skryté komponenty cest včetně `.aigiscode/scan.json`, policy a doctrine.
Změna těchto vstupů proto sama nevyvolá rebuild. Filtr navíc není odvozený od
skutečného `ScanConfig`; i povolená skrytá zdrojová složka zůstane ignorovaná.

Watcher se spouští až po prvním publikování
([`mcp/mod.rs:149`](../rust/crates/aigiscore/src/mcp/mod.rs#L149)), takže editace
během počáteční analýzy může uniknout. Callback potlačuje chyby streamu událostí
a selhání spuštění watcheru pouze vypíše na stderr. Události během rebuildu se
nejprve hromadí ve frontě a do `observed_revision` se propisují až později v
rebuild smyčce. `Freshness` přitom stav samotného watcheru neobsahuje.

**Podmínka opravy:** armování před capture, společná definice sledovaných vstupů,
okamžitá evidence pozorovaných změn a výslovný stav degradovaného watcheru.
Vlastní zápisy artefaktů se musí filtrovat po souborech či výstupní oblasti,
nikoli ignorováním celé konfigurační složky.

### Q06 — P1: chyba prvotní analýzy neukončí čekající MCP dotaz řádnou chybou

**Kontrola zdroje.** Po chybě první analýzy se zavolá `record_error`, ale
[`state()`](../rust/crates/aigiscore/src/mcp/mod.rs#L334) stále čeká na revision 1
až 15 minut. Výsledek čekání ignoruje a vrátí `ReadyState`.
[`snapshot()`](../rust/crates/aigiscore/src/mcp/mod.rs#L362) pak používá `expect`
na chybějící index. Čekající dotaz tak nemá zajištěné včasné, typované oznámení
počáteční chyby. Doba případné paniky nebyla lokálně přehrána.

**Podmínka opravy:** explicitní stav pending/ready/failed, probuzení čekajících
při selhání a respektování požadovaného deadline. Počáteční chyba musí být
rozpoznatelná od dlouho běžící analýzy.

### Q07 — P1: chyby syntaxe nemají vlastní kontrakt neúplné analýzy

**Kontrola zdroje.** Parsery rozlišují nenahranou gramatiku a chybějící strom,
ale nekontrolují tree-sitter error/missing uzly a nevracejí jejich diagnostiky.
Například [`parse_php_to_graph`](../rust/crates/aigiscore/src/parsing/php.rs#L21)
po získání stromu rovnou vytváří graf. Částečně zotavený strom se tak může započíst
mezi analyzované soubory bez varování o kvalitě parsování. Nejde o tvrzení, že
v nynějším korpusu byl nalezen konkrétní syntakticky poškozený soubor.

**Podmínka opravy:** publikovat parse diagnostics a stupeň úplnosti. Zotavené
části mohou být užitečné, ale závěry vyžadující úplnost nesmějí předstírat, že
žádná parse chyba nenastala. Nepodporovaný jazyk musí být oddělený od čistého
výsledku podporovaného jazyka.

### Q08 — P1: přidání externího scanneru může odstranit kontrolu vrstev

**Kontrola zdroje, konkrétní tok dat.** Běžná pipeline používá
`build_architectural_assessment_full` s `doctrine_layers`. Po externím skenu však
[`run_project_analysis_command`](../rust/crates/aigiscore/src/cli.rs#L872)
přepíše celý assessment výsledkem wrapperu
[`build_architectural_assessment_with_ast_grep_and_graph`](../rust/crates/aigiscore/src/assessment/mod.rs#L166),
který předává prázdné `layers`. `detect_layer_contract_violations` pro prázdné
vrstvy vrací prázdný výsledek. Zapnutí bezpečnostního rozšíření tedy může oslabit
již provedenou architektonickou kontrolu.

Samostatně pipeline při načítání doctrine převádí chybu na prázdné vrstvy
([`pipeline.rs:343`](../rust/crates/aigiscore/src/ingestion/pipeline.rs#L343)).
Standardní zápis artefaktů doctrine načítá znovu a chybu vrací; toto pozdější
zachycení však nechrání všechny cesty, například analýzu bez zápisu.

**Podmínka opravy:** jeden společný způsob sestavení assessmentu se stejným
kontextem. Neplatná požadovaná doctrine musí být chybou nebo explicitně
neúplnou analýzou, nikoli nepozorovaným vypnutím ochrany.

### Q09 — P2: první snapshot se popisuje jako změna proti minulému stavu

**Pozorováno v čerstvém výstupním adresáři.** Při prvním self-auditu je
`previous_findings = 0`, ale guard píše o aktuálním diffu, 83 změnách kontraktů
a zvýšení počtu nálezů o 841. Přitom nemá načtený předchozí stav.
[`build_convergence_history_artifact`](../rust/crates/aigiscore/src/artifacts.rs#L3555)
převádí neexistující předchozí findings na prázdný seznam. Chybí jasné rozlišení
„baseline nebyla poskytnuta“ a „ověřená baseline byla čistá“.

CLI současně po úspěšném zápisu vrací 0 bez ohledu na guard verdict
([`cli.rs:926`](../rust/crates/aigiscore/src/cli.rs#L926)). To může být platný
kontrakt reportovacího příkazu, ale `analyze` samotné proto není vynucující CI
brána a nesmí se tak prezentovat.

**Podmínka opravy:** existence a identita baseline v kontraktu; první snapshot
bez tvrzení o regresi. Explicitně definovat způsob, kterým automatizace
vyhodnotí `Allow/Warn/Block` a stav neúplných kontrol.

### Q10 — P1 pro vydání: testy stále nemají úspěšný CI výsledek

[CI po merge](https://github.com/Draivix/aigiscode/actions/runs/34323575769)
prošlo webovým buildem/type kontrolou a `cargo fmt --check`. Clippy skončilo
39 chybami knihovny a 41 chybami testovacího cíle; `cargo test` bylo přeskočeno.
Nejde o 41 selhaných testů.

Porovnání s [předchozím CI](https://github.com/Draivix/aigiscode/actions/runs/29822587518)
nenašlo novou dvojici diagnostická zpráva/soubor. Příklady: zbytečné reference,
neinlineované format argumenty, položky za testovacím modulem, nepoužitá mutabilita
a příliš složitý typ. Posuny řádků nejsou nové chyby.

**Podmínka opravy:** odstranit uvedené blokátory bez oslabení gate, potom nechat
CI skutečně provést regresní sadu. Úspěšné lokální sestavení produkční binárky
nenahrazuje kompilaci a běh všech testů. Tento audit nepřidává plošný lint cleanup.

### Q11 — P2: dávkový výkon je lepší, interaktivní škálování zůstává slabé

**Měření a kontrola zdroje.** Na zachovaném korpusu Draivix se celý audit zrychlil
z 214,68 na 79,90 s. Peak RSS zůstává přibližně 3,3 GiB; artefakty mají celkem
1 685 767 571 bytů. Poslední MCP start přes první dvě odpovědi trval 68,08 s
i s ověřenou cache grafu. Tato čísla jsou jedno pozorované párové měření a
samostatná API ověření, nikoli statistický ani konkurenční benchmark.

[`watch.rs`](../rust/crates/aigiscore/src/mcp/watch.rs#L1) při změně znovu analyzuje
celý projekt. Velké výstupní pohledy se staví současně v paměti a mnoho kontraktů
opakuje cesty a metadata. Jedna atomicky publikovaná snapshot hodnota nezajišťuje
atomickou publikaci celé souborové sady; současná ochrana je pouze po souborech.

**Podmínka opravy:** změřit škálování na více velikostech, první odpověď, odezvu
na jedinou změnu a paměť pod souběžnými požadavky. Podle výsledků zavést skutečné
zpracování změněných částí a menší reprezentace. Přidat společnou generaci
artefaktů, aby čtenář nemíchal různé běhy.

### Q12 — P2: koncentrace implementace současně omezuje sekundární kontrolu

**Měření vlastního zdroje.** `artifacts.rs` má 9 179 řádků, `assessment/mod.rs`
6 964, `agentic.rs` 5 558, `resolve/mod.rs` 4 266 a `mcp/mod.rs` 4 039. Čísla
obsahují testy a sama neprokazují špatný návrh; v `artifacts.rs` je však před
testovacím modulem přes 7 300 řádků a řada různých odvozených kontraktů.

Sekundární ast-grep scanner má
[`AST_GREP_MAX_FILE_BYTES = 150_000`](../rust/crates/aigiscore/src/scanners/ast_grep.rs#L666).
Při self-auditu kvůli velikosti přeskočil sedm souborů, včetně `artifacts.rs`,
`assessment/mod.rs`, `agentic.rs` a `mcp/mod.rs`. Tato výluka je poctivě uvedená
v artefaktu; 65 všech přeskočených souborů zahrnuje i běžné prefiltry. Není
správné všech 65 označit za selhání parseru. Nativní grafová analýza těchto
velkých souborů běžela.

**Podmínka opravy:** oddělit konkrétní odpovědnosti a sjednotit duplicitní
projekce/maskování; pouhé rozdělení podle počtu řádků nepomůže. Sekundární scanner
musí mít použitelnou strategii pro velké soubory nebo zřetelnou mezeru pokrytí
na úrovni výsledného hodnocení. Volitelná Kuzu větev navíc stále používá Node
bridge; úplný native Rust cíl proto není splněn pro všechny volitelné cesty.

## Co je již podložené

Nová revize skutečně řeší dříve ověřené chyby přiřazení receiveru, parser-owned
PHP container argumentů a typových cyklů. V zachovaném velkém korpusu bylo
ověřeno 97 kroků uzavřených cyklických cest bez chybějící hrany v exportovaném
dependency pohledu. Skutečné MCP odpovědi přenesly všechny ověřované cesty
i tři generované výluky. Cache váže graf na obsah a revizi sémantiky. Tyto
výsledky nejsou znehodnocené tím, že jiné cesty zatím zůstávají chybné.

Vlastní audit nyní zpracoval 105 podporovaných souborů ze 192 prohlédnutých,
2 419 symbolů a 5 586 vyřešených hran za 2,41 s. Nehlásí silný cyklus; tato
nula není potvrzením architektonické čistoty. Eviduje 738 hardwiring nálezů,
62 complexity hotspotů a výše doložené falešné nálezy. Počty nejsou počet
potvrzených defektů ani měření precision/recall.

## Doporučené pořadí další práce

1. Uzavřít důvěryhodnost výsledku: Q03–Q08 a CI Q10. Selhání, neúplnost,
   změněná doctrine a nefunkční watcher musí být explicitní.
2. Opravit ověřené chyby signálu: Q01–Q02 a baseline Q09. Pro každou rodinu
   vyhodnocovat ručně ověřené pozitivní i negativní příklady; měřit precision
   a recall samostatně podle jazyka, frameworku a typu důkazu.
3. Prokázat použití ve velkém monorepozitáři: Q11–Q12, obnova po přerušení,
   změny během indexování, čas do první užitečné odpovědi a společný označený
   korpus pro srovnání s ostatními nástroji.

Do té doby lze AigisCode využít jako podklad pro lidský nebo agentní rozbor
s kontrolou konkrétních zdrojů. Bezpečnostní potvrzení a nevratné automatické
zásahy vyžadují další důkazy. Samostatný existující trust-remediation work lock
zůstává otevřený.
