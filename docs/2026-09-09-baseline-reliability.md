# Baseline: ověřené podklady a dostupnost srovnání — 2026-09-09

Nový audit AigisCode rozlišil ověřenou baseline od možnosti použít ji ke srovnání.
Podklady odpovídaly otiskům manifestu, ale předchozí i současná analýza měly
neúplné vstupy a změnilo se sestavení enginu. Výsledek proto uvádí `verified` /
`not_compared`, 688 logických nálezů se stavem `NotCompared` a `null` pro změny
grafu i kontraktů. Žádný nález neoznačil za vyřešený, zlepšený, nový ani zhoršený.

Při předcházejícím použití skutečné baseline z auditu Q07 byl starší manifest bez
ověřovacích otisků rozpoznán jako `unverified`, s důvodem `missing_seal`. Nebyl
považován za čistou nebo prázdnou baseline. Původní artefakty Q07 zůstaly zachovány;
pro nový audit byly použity kopie jeho baseline podkladů.

Závěrečný běh trval 2,34 s, maximum RSS bylo 155 268 KiB. Guard zůstal `Block`
kvůli neúplným vstupům; `pressure.comparison_available` je false. Návratový kód 1
vyjadřuje neúplnost analýzy, nikoli selhání zápisu artefaktů. Produkční release
build prošel bez varování.

[Strojový důkaz](2026-09-09-baseline-reliability-evidence.json) obsahuje oba stavy
baseline, identity enginů a vstupů, otisky souborů, čítače, null delty a příklad
nálezu se dvěma zachovanými výskyty. Úplná data jsou v
`target/reliability-2026-09-09/self-q09-upgrade/`; mezivýsledek migrace je zachován
v `target/reliability-2026-09-09/self-q09-legacy-result/`.

## Změna chování

- První snapshot používá `FirstObserved`, ne tvrzení o nově zavedené vadě.
  Chybějící nebo nesrovnatelná minulost nemá číselné delty představující nulu.
- Změny rootu, scope, enginu či výběru externích kontrol a neúplnost vstupů
  zabraňují časovým závěrům. Neplatné nebo smíchané baseline podklady se nesmí
  proměnit v prázdný předchozí stav.
- Manifest se zapisuje poslední. Čtečka ověřuje stejné bajty, které deserializuje,
  pro architecture surface, review surface a contract inventory; po načtení
  znovu kontroluje manifest.
- Convergence počítá logické fingerprinty, zatímco raw review zachovává jednotlivé
  výskyty. Deterministický výběr reprezentanta preferuje viditelnost, závažnost
  a confidence. Přidány jsou počty současných a předchozích výskytů.
- Agentové příkazy přebírají kontext právě vytvořený writerem. Nově zapsaný
  aktuální snapshot už nenačtou jako vlastní předchozí baseline. MCP používá
  stejnou čtečku a zachovává nové stavy v kontraktech i paketech pro agenty.
- Otisk sestavení enginu se zachytává během nativního Rust buildu bez nové
  závislosti. Fast-load kontroluje tento otisk i identitu scope; samotné shodné
  číslo verze produktu nestačí.

Podrobný strojový a provozní kontrakt včetně návratových kódů a samostatného
vyhodnocení `Allow` / `Warn` / `Block` je v [BASELINE_CONTRACT.md](BASELINE_CONTRACT.md).

## Meze důkazů

Ověřeny jsou skutečné audity celého vlastního repozitáře a jejich JSON/Markdown
výstupy, migrace staré neověřené baseline a odmítnutí srovnání ověřené, ale
nesrovnatelné baseline. Automatizované testy a CI zůstaly podle Davidova pokynu
vypnuté. Samostatné scénáře prázdné ověřené baseline, poškození během souběžného
zápisu a kompletní srovnatelné baseline nebyly v tomto kroku dynamicky přehrány.

Otisky chrání konzistenci těchto tří podkladů; nejsou autentizací původu ani
důkazem atomické publikace celé rodiny artefaktů. Následuje dokončení práce se
zdroji a velikostí analýzy a nové vyhodnocení Draivix. Jeho zdrojové soubory
zůstaly beze změny.
