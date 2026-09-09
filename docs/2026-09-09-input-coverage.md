# Úplnost vstupů a diagnostiky parserů — 2026-09-09

Skutečný audit celého AigisCode nyní přiznal dvě parserová zotavení a dva
nepodporované zdrojové soubory. Uložil úplnou rodinu artefaktů, vrátil kód 1 a
guard `Block` s výslovným důvodem chybějících důkazů. Nulový počet dead-code
nálezů označil jako odloženou kontrolu, nikoli čistý výsledek.

Obě zotavení se týkají výrazu `&raw` v `ingestion/pipeline.rs:212` a
`mcp/contracts.rs:2277`. Produkční Rust build stejného kódu prošel bez varování.
Diagnostika tree-sitter tedy sama o sobě nedokazuje chybu zdrojového kódu;
může označovat omezení gramatiky. Tento rozdíl je součástí kontraktu.

Nepodporované vstupy jsou `install.sh` a `install.ps1`. Bylo zaznamenáno 110
parse outcomes pro 110 podporovaných zdrojových souborů, bez souboru chybějícího
v evidenci parsování. Dalších 92 skenovaných vstupů spadá mimo podporované nebo
rozpoznané nepodporované zdrojové přípony. Neznámá přípona tím není prohlášena
za bezpečný datový soubor.

Audit trval 2,29 s, maximum RSS bylo 144 740 KiB. Všech 655 hran dotýkajících
se souborů s parserovým zotavením dostalo sílu `Inferred`, confidence nejvýše
500 a důvod `parser_recovery`. Tyto hrany zůstávají ve výstupním grafu jako
částečný podklad; nemají vytvářet potvrzené silné závislosti.

[Strojový důkaz](2026-09-09-input-coverage-evidence.json) obsahuje diagnostiky,
počty, guard rozhodnutí a hashe binárky a manifestu. Úplné lokální artefakty jsou
v `target/reliability-2026-09-09/self-q07-final/`.

## Kontrakt

- `semantic-graph.json.parse_outcomes` obsahuje pro každý parsovaný soubor
  parser, rozsah, příznak zotavení, počet diagnostik, jejich pozice a příznak
  zkrácení seznamu. Evidence vzniká ze stejného stromu jako extrakce grafu;
  zdroj se kvůli ní neparsuje podruhé.
- Rozlišují se `error_node` a `missing_node`. Řádky a UTF-8 bajtové sloupce
  jsou od jedničky; koncové souřadnice jsou exkluzivní. U každého souboru je
  nejvýše 32 diagnostik, skutečný počet a příznak zkrácení zůstávají zachovány.
  Rozdíl obou uzlů vysvětluje
  [dokumentace tree-sitter](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html#special-nodes).
- `unsupported_sources` odděluje rozpoznané jazyky bez parseru od parsovaných
  souborů. Ostatní vstupy mají samostatný počet. Inventář není tvrzením, že
  katalog přípon rozpoznává každý možný programovací jazyk nebo skript bez přípony.
- `input_coverage` v reportu, raw findings, architecture surface a convergence
  má stav `complete`, `incomplete`, `no_supported_sources` nebo `unknown`.
  Přehled obsahuje počty a omezené náhledy; detail zůstává v sémantickém grafu.
  `complete` popisuje pokrytí rozpoznaných vstupů syntaxí, nikoli sémantickou
  správnost všech hran, úplnost framework modelů nebo bezpečnost projektu.
- Vue adapter explicitně uvádí `vue_script_only`: template bindings a další
  oblasti SFC nepovažuje za kompletně analyzované. I syntakticky čistý script
  proto nezaručuje úplný graf komponenty.
- Při neúplném pokrytí se odkládají `unused_imports`, `unused_private_functions`,
  `orphan_modules` a `confirmed_orphan_files`. Suroví kandidáti s nulovým počtem
  vstupních hran zůstávají dostupní, ale nepovyšují se na potvrzený orphan dluh.
- Guard blokuje automatické přijetí výsledku kvůli chybějící evidenci. Po zápisu
  částečných artefaktů vrací analytická CLI cesta kód 1; stejný stav platí také
  pro analýzu bez zápisu. Chyba načtení zdroje nebo chybějící parse tree zůstává
  řádnou chybou analýzy.
- MCP overview, coverage a quality nesou `input_coverage`; všechny tool/resource
  odpovědi mají navíc stručné `_meta["aigiscode/input_coverage"]`. Čerstvost je
  oddělená vlastnost: čerstvý snapshot může být neúplný. Quality nesmí nulové
  odložené kontroly zobrazit jako nízké riziko; používá `unknown` a výslovný popis.
- Sémantická revize manifestu je 10. Starší graf bez parse evidence se nesmí
  použít přes fast-load. Fast-load obnovuje také inventář nepodporovaných a
  ostatních vstupů, i když se podporované zdrojové soubory nezměnily.

## Meze ověření

Ověřen je produkční release build a nový produktový audit reálného repozitáře
včetně JSON a Markdown artefaktů, návratového kódu a síly odvozených hran.
Automatizované testy ani CI se podle Davidova pokynu nespouštěly. Existující
testovací vstupy byly přizpůsobeny novému datovému kontraktu, ale jejich výsledky
nejsou nové ověřené důkazy. MCP metadata, ostatní jazyky a dynamické změny cache
nebyly v tomto kroku samostatně přehrány.

Dalším bodem je Q09: existence, identita a srovnatelnost baseline. Změnu počtu
nálezů při odložení kontrol nelze považovat za prokázané vyřešení dluhu. Před
přijetím celé opravy ještě zbývá tuto část convergence zpřesnit a provést nové
úplné vyhodnocení Draivix. Jeho zdrojové soubory zůstaly beze změny.
