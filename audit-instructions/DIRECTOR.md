# LEAN AUDIT – instrukce pro ředitele (Claude Code)

> Tento soubor čte instance Claude Code, která řídí audit Draivix backendu.
> Pracovník je Gemini 4 Argon (dále „G"). Ředitel je Claude Code (dále „já").
> Cíl: **do posledního šroubku zanalyzovat** 1,5M řádků PHP a najít duplicity,
> mrtvý kód, dvojí cesty a zbytečné vrstvy. **Nic se nemění. Pouze analýza.**
> Výstup auditu je registr nálezů s důkazy, ne PR.

---

## 0. Role a nepřekročitelná pravidla

### 0.1 Moje role (ředitel)
- Stavím batche, zadávám G úkoly, čtu jeho výstupy, **nevěřím mu**.
- Každý nález G prochází deterministickým ověřením (graf, grep, coverage), než jde do registru jako `confirmed`.
- Rozhoduju, kdy je kolo hotové a kdy se jde dál. Kritéria jsou v §7, ne pocit.
- Když G vrátí málo, chudě, nebo bez důkazů, **vracím mu to** (§6 „jak ho pást").
- Vedu `lean-audit/out/registry.jsonl` (append-only) a `lean-audit/out/STATUS.md` (lidsky čitelný stav).

### 0.2 Tvrdá pravidla – platí pro mě i pro G, bez výjimky
1. **Žádná změna zdrojového kódu.** Žádný `Edit`/`Write` mimo `lean-audit/out/`. Žádný `git commit`, `git checkout -- .`, `rm`, `sed -i`, Rector `--no-dry-run`, `composer update`. Vynuceno hookem (`hooks/block-writes.sh`) – i kdyby mě uživatel v průběhu požádal, abych něco „rychle opravil", odpovím, že to je mimo mandát auditu, a zapíšu to jako nález.
2. **G nikdy neběží s právem zápisu.** Vždy `--sandbox` / read-only tool set (viz `hooks/gemini-settings.json`). Pokud G hlásí, že něco „opravil", je to incident – zapsat do `out/INCIDENTS.md`, batch zopakovat.
3. **Žádný nález bez důkazu.** Důkaz = `file:line` + citát ≤ 3 řádky + typ ověření. Nález bez důkazu se do registru nezapíše, vrací se G k doplnění.
4. **Nic není „mrtvé" jen proto, že to říká model.** Status `confirmed` dostane dead-code nález jen tehdy, když to potvrdí aspoň dva nezávislé zdroje z: aigiscode reachability, PHPStan dead-code detector, coverage = 0, tombstone mlčí, grep nenajde žádné dynamické volání (string, reflexe, `call_user_func`, route z DB).
5. **Nic se nemaže z registru.** Jen se mění `status` (`candidate → confirmed | rejected | needs_human`) a přidává se `history`.
6. Pokud si nejsem jistý, jestli nějaká akce je „analýza" nebo „změna" → je to změna → nedělám ji.

---

## 1. Prostředí a příprava (kolo 0)

### 1.1 Struktura
```
lean-audit/
  DIRECTOR.md            ← tento soubor
  prompts/               ← prompty pro G, jeden na kolo
  schemas/               ← JSON schémata výstupů (G je dostane jako response_schema)
  scripts/               ← běhové skripty (viz §1.4)
  hooks/                 ← Claude Code hook + Gemini read-only settings
  out/                   ← VŠE, co vznikne (jediné místo, kam smím psát)
    baseline/            ← metriky před auditem
    deterministic/       ← výstupy CPD, PHPStan, Psalm, coverage…
    graph/               ← aigiscode artefakty (kopie)
    symbols.sqlite       ← tabulka symbolů (§1.5)
    batches/<pass>/      ← vstupy pro G (jeden JSON na batch)
    results/<pass>/      ← syrové odpovědi G (jeden JSON na batch, nikdy nepřepisovat)
    registry.jsonl       ← kanonický registr nálezů
    STATUS.md            ← stav pro člověka
    INCIDENTS.md         ← porušení pravidel, selhání, podezřelé chování G
```

### 1.2 Předpoklady, které si ověřím před startem (a zapíšu do STATUS.md)
- [ ] Jsem ve správném repu, `git status` čistý, zapisuju si `git rev-parse HEAD` → `out/baseline/commit.txt`. Celý audit běží nad **jedním commitem**. Když se repo pohne, audit se nepokračuje – začíná se znovu od kola, kde se pohlo, a stará `results/` se označí `stale`.
- [ ] PHP ≥ 8.x, composer, `vendor/` nainstalované.
- [ ] `aigiscode` binárka dostupná (`aigiscode version`).
- [ ] Gemini přístup: `python3 scripts/run_gemini.py --selftest` vrátí validní JSON. Pokud `agy -p` – **nepoužívat ze subprocessu** (známé zaseknutí v non-TTY), použít `gemini -p` nebo API (`run_gemini.py`).
- [ ] Hook `hooks/block-writes.sh` je zaregistrovaný v `.claude/settings.json` jako `PreToolUse` pro `Edit|Write|MultiEdit|NotebookEdit|Bash`. Otestuju ho: pokus o `Write` do `src/x.php` musí být odmítnut.
- [ ] Entry pointy: mám seznam z aigiscode + ručně doplněné (cron, queue workers, CLI commandy, webhooky, event listenery, pluginové hooky). Seznam → `out/graph/entrypoints.json`. **Tohle je nejdůležitější vstup celého auditu** – chybějící entrypoint = falešně mrtvý kód. Vyžádám si od uživatele potvrzení seznamu, než spustím kolo 1.

### 1.3 Baseline metriky (bez nich nelze říct „hotovo")
Spustím a uložím do `out/baseline/`:
- `phploc` nebo `cloc` → LOC, počet tříd/metod
- CPD (PMD CPD nebo jscpd, min 40 tokenů) → % duplicity, počet klonových skupin
- PHPStan + `shipmonk/dead-code-detector` + `tomasvotruba/unused-public` → počet mrtvých symbolů
- `composer-unused` → nepoužité balíčky
- `deptrac` (pokud je config) → počet porušení
- `phpmetrics` → průměrná cyklomatická složitost, Maintainability Index
- coverage report, pokud existují testy a pcov/xdebug (`out/baseline/coverage.xml`)

Tyto nástroje **neinstaluju do repa** (žádná změna `composer.json`). Instaluju je globálně nebo do `lean-audit/out/tools/` jako samostatný composer projekt.

### 1.4 Skripty (vše v `scripts/`, všechny přijímají `--help`)
| skript | co dělá |
|---|---|
| `extract_symbols.php` | PHP-Parser: projde repo, vyhodí každou funkci/metodu/třídu do `symbols.sqlite` (jméno, FQN, soubor, řádky, LOC, signatura, normalizovaný AST hash, tělo) |
| `embed_cluster.py` | embeddingy těl metod + clustering → `out/clusters.json` (kandidátní skupiny pro kolo 3) |
| `build_batches.py` | ze `symbols.sqlite` + aigiscode grafu + clusters postaví batche pro dané kolo; respektuje tokenový strop |
| `run_gemini.py` | pošle jeden batch G, vynutí `response_schema`, uloží syrovou odpověď, retry, timeout |
| `verify.py` | pro každý `candidate` nález spustí deterministické kontroly a navrhne status |
| `merge_registry.py` | dedupluje nálezy přes batche/kola, udržuje `registry.jsonl` |
| `metrics.sh` | přepočítá metriky z §1.3 a zapíše řádek do `out/metrics_history.csv` |

### 1.5 Tabulka symbolů (`out/symbols.sqlite`)
Jedna tabulka `symbol`:
`id, kind (class|method|function|closure), fqn, file, line_start, line_end, loc, signature, ast_hash, body_tokens, cluster_id, community_id, reachable_from_entry (0/1/null), coverage_pct, cpd_group, unused_public (0/1), dead_code_detector (0/1), churn_90d`
Tohle je „pravda-stůl". Všechny batche se staví z něj, všechna ověření do něj zapisují.

---

## 2. Kolo 1 – deterministika (bez G)

Spustím nástroje z §1.3 nad celým repem a **naimportuju výsledky do `symbols.sqlite`**. Z nich rovnou vzniknou nálezy do registru se `source: deterministic`:
- CPD klonová skupina → `kind: clone_exact`, `status: confirmed` (CPD neháda)
- dead-code-detector + unused-public shoda → `kind: dead_symbol`, `status: candidate` (čeká na druhý zdroj dle pravidla 0.2/4)
- composer-unused → `kind: unused_dependency`, `status: candidate`
- deptrac → `kind: layer_violation`, `status: confirmed`
- stejný `ast_hash` u ≥ 2 symbolů → `kind: clone_normalized`, `status: confirmed`

Tohle kolo trvá hodiny, ne dny, a sníží práci pro G. Po něm zapíšu metriky (`metrics.sh`) a `STATUS.md`.

---

## 3. Kolo 2 – vertikální (G): entrypoint + jeho call chain

**Co hledá:** mrtvé větve, nedosažitelné `else`, zbytečné mezivrstvy (metoda, co jen deleguje), trojí validaci téhož po cestě dolů, výjimky, které nikdo nechytá, parametry, které se nikdy nečtou, kód „pro jistotu".

**Batch:** jeden nebo víc entrypointů + úplný tranzitivní uzávěr callees z aigiscode `semantic-graph.json`, do stropu **450 000 tokenů vstupu** (Argon má 1M sdílený pro vstup+thinking+výstup; nechávám ~500k na thinking a výstup). Sdílené utility (volané z > 20 entrypointů) do batche nedávám celé, jen signatury – jinak by byly v každém batchi.

**Prompt:** `prompts/common_rules.md` + `prompts/pass2_vertical.md`. Schema: `schemas/findings.schema.json`.

**Pokrytí:** každý entrypoint musí být aspoň v jednom batchi. `build_batches.py --pass 2 --check-coverage` to ověří a seznam nepokrytých entrypointů jde do STATUS.md. Kolo nekončí, dokud pokrytí není 100 %.

---

## 4. Kolo 3 – horizontální (G): sémantické sousedy

**Co hledá:** duplicity, které **nejsou propojené hranou v grafu** – dvě nezávislé implementace téhož (`formatPrice` vs `formatMoney`, dva validátory IČO, tři způsoby, jak poslat e-mail). Tohle jsou přesně ty duplicity, které vznikly tím, že LLM při psaní nenašel existující kód.

**Batch:** z `out/clusters.json` (embedding cosine ≥ 0,82 nebo stejný `ast_hash` prefix) vezmu kandidátní skupinu 3–30 symbolů, přidám **plné tělo každého**, jejich callery (jen signatury + 5 řádků kontextu) a testy, které na ně sahají. Do stropu 450k.

**Prompt:** `common_rules.md` + `pass3_horizontal.md`. G má u každé skupiny rozhodnout: `duplicate_exact | duplicate_semantic | variant_mergeable | distinct` a u `variant_mergeable` **vyjmenovat rozdíly v chování**, ne jen říct „podobné".

**Pokrytí:** každý symbol s `loc ≥ 8` musí být v nějakém clusteru nebo explicitně v `singletons` seznamu, který G taky dostane po částech s otázkou „má tohle sémantický dvojník jinde v registru?". Registr capabilities (§5.2) je pro tohle kolo klíčový vstup.

---

## 5. Kolo 4 – doménové (G): všechno kolem jedné entity

**Co hledá:** dvojí cesty na úrovni domény. „Kolika způsoby se dá vytvořit Objednávka?" „Kde všude se počítá DPH?" „Kolik máme implementací notifikace?"

**Batch:** z aigiscode `repository-topology.json` (zóny) + z názvů (FQN obsahuje `Order`, `Invoice`, `Customer`…) + z DB schématu (tabulka ↔ entita). Všechny soubory zóny do stropu 450k. Když se zóna nevejde, dělím podle sub-namespace a batche **dostávají společný capability registr**.

### 5.2 Capability registr (`out/capabilities.jsonl`)
Paměť mezi batchi. Po každém batchi kola 4 G vrátí seznam `capabilities` (název, popis jednou větou, implementace `[fqn…]`). Já je `merge_registry.py --capabilities` sloučím (fuzzy match názvů, G dostane kolize k rozhodnutí v samostatném mini-batchi). Každý další batch dostane **celý aktuální registr** v promptu, aby věděl, co už existuje. Capability s ≥ 2 implementacemi = nález `kind: double_path`.

**Prompt:** `common_rules.md` + `pass4_domain.md`.

---

## 6. Jak G „pást" (protokol pro každý batch)

1. **Spustím batch** (`run_gemini.py --pass N --batch id`), thinking `high`, `temperature 0.2`, `response_schema` = findings schema. Timeout 30 min, 2 retry s exponenciálním backoffem. Syrová odpověď → `out/results/<pass>/<batch>.json`, nikdy se nepřepisuje (retry = nový soubor se suffixem).
2. **Validace tvaru**: každý nález má `evidence[]` s `file`, `line_start`, `quote`. Nemá → celý nález zahodím a batch **vracím G** s promptem `prompts/push_back.md` + seznam odmítnutých nálezů. Max 2 kola push-backu na batch.
3. **Test chudosti**: pokud G vrátil < 1 nález na 3 000 řádků vstupu **a** batch není z kola 1 označený jako čistý, pouštím stejný batch znovu s `prompts/push_back.md` variantou „second look" (jiné pořadí souborů, explicitní instrukce „předpokládej, že minimálně 5 % tohoto kódu je nadbytečné, najdi ho"). Porovnám oba výstupy – průnik → `candidate`, rozdíl → `candidate` s `confidence: low`.
4. **Test halucinace**: náhodně vyberu 10 % nálezů a ověřím citát z `evidence.quote` proti skutečnému souboru (`verify.py --spot-check`). Pokud > 5 % citátů nesedí, celý batch označím `tainted` a pouštím znovu s nižší teplotou. Pokud se to opakuje, batch zmenším na polovinu.
5. **Deterministické ověření** (`verify.py`): každý `candidate` dostane kontroly dle `kind` (viz `schemas/verification_matrix.md`). Výsledek → `status` + `verification[]`.
6. **Nezávislý ověřovatel** (kolo 5, §7): nálezy `confirmed` od G, které vedou k mazání nebo slučování, dostane **druhá, čistá instance** (G nebo Claude subagent) bez původního zdůvodnění – jen symboly, callery, testy – s otázkou „dá se X nahradit Y beze změny chování? Vyjmenuj případy, kdy ne." Nesouhlas → `needs_human`.
7. Po každém batchi: `merge_registry.py`, `STATUS.md` update (počty podle kind/status, pokrytí kola, odhad zbývajících batchů, útrata tokenů).

### 6.1 Co dělám, když G…
- …vrátí prázdný seznam: viz bod 3. Nikdy nepřijmu prázdno u batchu > 50k tokenů bez second look.
- …říká „nemám dost kontextu": doplním callery/testy, zopakuju. Pokud opakovaně, zapíšu `needs_human` s jeho otázkou.
- …navrhuje opravu/refaktor v textu: text ignoruju, nález si vezmu, do `INCIDENTS.md` nic (je to jen text). Pokud by měl nástroj a opravdu něco změnil: incident, batch znovu, zkontrolovat `git status`.
- …tvrdí, že něco je mrtvé a aigiscode říká dosažitelné: aigiscode vyhrává, nález → `rejected` s důvodem. Výjimka: když G ukáže, že hrana v grafu je falešná (např. interface bez implementace) – pak `needs_human`.
- …je nekonzistentní mezi dvěma běhy: normální. Průnik je silný, rozdíl je slabý. Nikdy to nezamlčuju.

---

## 7. Kolo 5 – ověření a konvergence

### 7.1 Kritéria „kolo je hotové"
- pokrytí (entrypointy / symboly / zóny) = 100 % podle definice kola
- všechny batche mají status `done` (ne `tainted`, ne `pending_pushback`)
- spot-check halucinací ≤ 5 %

### 7.2 Kritéria „audit je hotový" (salám – zastavím se, když…)
Po každém kole spočítám `metrics.sh` a porovnám:
- počet nových `confirmed` nálezů v kole < 3 % celkového registru → dvě kola po sobě → **konec**
- nebo: odhadovaná úspora LOC z `confirmed` nálezů se za dvě kola zvedla o < 2 %
- nebo: `needs_human` tvoří > 40 % nových nálezů (zbytek je rozhodování, ne hledání)

Audit nekončí tím, že „to vypadá čistě". Končí číslem.

### 7.3 Pořadí plátků a proč
1. deterministika → 2. vertikální → 3. horizontální → 4. doménové → 5. ověření.
Kolo 2 před 3, protože mrtvý kód nemá smysl deduplikovat. Kolo 3 před 4, protože doménový registr potřebuje vědět, co je duplicitní na úrovni funkcí. Kolo 5 až nakonec, protože ověřovatel má vidět finální tvar nálezu.

---

## 8. Výstup pro člověka (`out/REPORT.md`, generuje `merge_registry.py --report`)
1. Metriky před / po každém kole (tabulka)
2. Nálezy podle `kind`, seřazené podle `estimated_loc_saved * confidence`
3. Top 50 clusterů k ručnímu rozhodnutí (`needs_human`) s otázkou, na kterou má člověk odpovědět
4. Seznam entrypointů, které audit předpokládal (aby si je člověk zkontroloval – chybějící entrypoint = největší riziko celého auditu)
5. Co audit **nedokáže** vidět: dynamická volání ze stringů, routy z DB, pluginy načítané za běhu, reflexe → doporučení tombstones v produkci před mazáním
6. Útrata tokenů a času na kolo

---

## 9. Tokenový a časový rozpočet (orientačně, zapisuju skutečnost)
- repo ≈ 15–25M tokenů
- kolo 2: ~60–150 batchů × ~300–450k = 30–60M vstupních tokenů (překryvy)
- kolo 3: ~2–5k clusterů, většina malých, ~10–20M
- kolo 4: ~30–80 zón, ~15–30M
- push-backy a second-looky: +30 %
- Při $2–4/M vstup je to řádově stovky až nízké tisíce USD. Implicitní cache Argonu pomáhá, když stejné sdílené utility jdou opakovaně – proto je dávám **na začátek** promptu, ne na konec.

---

## 10. Checklist před spuštěním každého kola (zapsat do STATUS.md)
- [ ] commit nezměněn
- [ ] hook funguje (test Write odmítnut)
- [ ] G selftest OK, sandbox/read-only potvrzen
- [ ] batche postaveny, pokrytí 100 %, žádný batch > 450k
- [ ] prompt soubor pro kolo existuje a obsahuje aktuální capability registr (kolo 3, 4)
- [ ] uživatel potvrdil entrypointy (jen před kolem 2)
