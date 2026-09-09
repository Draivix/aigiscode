# Hodnocení složitosti Vue a modulů JS/TS na Draivix

AigisCode nyní zahrnuje skripty Vue a modulové přípony JS/TS do nativního hodnocení
složitosti. Na uloženém korpusu vzniklo 64 nových záznamů: 59 ve Vue a pět v `.mjs`.
Současně zmizelo jedno dosavadní varování nad malými pevnými seznamy operátorů.
Počet záznamů složitosti se změnil z 1 536 na 1 599; nejde o 64 prokázaných
výkonnostních závad ani o změnu zdrojového kódu Draivix.

[Strojový doklad](2026-09-09-vue-complexity-evidence.json) obsahuje otisky binárky,
artefaktů a vybraných zdrojů, negativní kontroly, ověření pozic a grafových cest,
skutečné odpovědi MCP a měření. Všech 23 166 souborů zachovaného snímku stále
odpovídá původnímu inventáři velikostí a SHA-256. Živý Draivix nebyl upraven;
pozdější změny Mitel tento uložený snímek neobsahuje.

Výběr jazyků využívá společný lexikální masker, včetně `.mjs`, `.cjs`, `.mts`,
`.cts` a velikostních variant přípon. Vue prochází stejnou extrakcí skriptů jako
sekundární scanner. Šablony, styly a komentáře proto nevytvářejí nativní smyčky.
Hodnocení operací, odvození grafové cesty a příprava toku operací nyní mají vlastní
modul `assessment/complexity.rs`; původní `assessment/mod.rs` má o 978 řádků méně.
Sdílené typy a orchestrace zůstaly v původním modulu. Celkový počet řádků není
měřítkem kvality této změny; podstatné je oddělení konkrétní odpovědnosti.

Scanner zachovává všech 9 978 dosavadních stop. Osm z nich nově nese syntaktický
důkaz `bounded_membership`: horní mez počtu primitivních literálů a pozici metody
`includes`. Hodnocení potlačuje tlak pouze u doložených malých seznamů s horní mezí
nejvýše 16. Surová stopa zůstává dostupná. Počítání čárek zahrnuje i prázdná místa
a koncovou čárku, proto jde o konzervativní horní mez. Spread, výpočtové položky,
callbacky a větší seznamy zůstávají předmětem kontroly. Chybějící důkaz neznamená
neomezený vstup. Přesná pozice metody brání potlačení jiného volání na témže řádku
nebo uvnitř argumentů; podrobnosti stanoví [kontrakt](SECONDARY_COVERAGE_CONTRACT.md).

Zdrojové kontroly ukazují různé významy nalezených struktur:

| Případ | Pozorování a význam |
| --- | --- |
| `GlobalSearch.vue:1143` | Regex vzniká uvnitř smyčky přes slova dotazu; jde o kandidáta pro měření opakované konstrukce. |
| `ReportFilterEditor.vue:98,104` | Opakované hledání v `props.fields` a kontrola narůstajícího seznamu přes `some` mají konkrétní místa v kódu. |
| `ItemsGrid.vue:962,1192` | Dvě kontroly šesti pevných hodnot zůstávají surovým důkazem, ale soubor nemá žádný záznam složitosti. Ani `v-for` v šabloně na řádcích 2065, 2104 a 2134 nevytváří nativní varování. |
| `counterWidgetState.ts:35,37,39` | Seznamy pěti, tří a tří operátorů jsou pevné. Původní varování bylo odstraněno bez změny zdroje. |
| `GlobalSearch.vue:268` | Průchod skupinami a jejich položkami může být lineární v celkové velikosti vstupu. Samotné vnoření neprokazuje kvadratické chování. |
| `build-native-shell.mjs:30,37` | Kontroly souborů patří k sestavení nativní aplikace; zdroj má seznam pěti souborů a jednoho adresáře. Nejde o důkaz pomalé uživatelské odezvy. |

Dva skriptové negativní případy doplňují komponenty bez skriptu `StackNode.vue`
a `CleanRoomIcon.vue`: ani jedna nemá záznam složitosti. Přehled nevydává všechny
nové kandidáty za závady. Text vysvětlení nyní výslovně žádá prověřit meze dat
a změřit dopad před refaktoringem; zachovává označení přesnosti `heuristic`.

Kontrola nových cest odhalila odhadnutou vazbu z `vault-addon/src/popup/app.ts:244`
na `Cms/Show.vue:copyText`. Zdroj popupu přitom definuje vlastní lokální helper
na řádku 186. Surová hrana zůstává `Inferred`, s rozlišením `Global` a důvěrou 500;
opravena je její nesprávná role při zvyšování priority, nikoli samotné rozlišení
lokální funkce. Hodnocení nyní nepoužívá odhadnuté nebo čistě typové hrany ani cesty
přes soubory klasifikované jako testy, migrace nebo seedery. Modelované runtime
vztahy mohou zůstat použitelné. Ani způsobilá grafová závislost není důkazem spuštění.

Pro 496 záznamů s cestou bylo ověřeno všech 1 157 přechodů, tedy 513 unikátních
kombinací zdrojového a cílového souboru, řádku a vztahu. Každá odpovídá alespoň jedné
neodhadnuté a netypové hraně v úplném nativním grafu; v cestách nejsou soubory
vyloučené uvedenou klasifikací. Oproti mezilehlému běhu bez tohoto filtru se u 542
záznamů změnila cesta nebo skóre: 254 má nižší skóre, 90 vyšší a 198 stejné skóre
s jinou cestou. Výběr jiné způsobilé cesty může změnit podporu oběma směry.

Hlavní pozice všech 1 599 záznamů odpovídají zachycenému místu operace. Například
vnořená smyčka v GlobalSearch ukazuje na řádek 268 místo náhradního řádku 1.
Bez doložené vstupní cesty zůstává lokální důkaz operace dostupný. Celý sémantický
graf je bajtově shodný s předchozím snímkem; nezměnily se ani části `graph_analysis`,
`security_analysis` a `hardwiring` v deterministickém výsledku.

Devět skutečných dotazů MCP ověřilo přehled, seznamy, podrobnosti vybraných nálezů,
kvalitu a pokrytí. Použitelný přehled přišel po 45,12 s, předtím server výslovně
vrátil stav indexování. Podrobnosti GlobalSearch obsahují řádky 268 a 1143 i nové
opatrnější vysvětlení. Dotaz na ItemsGrid vrací osm souvisejících nálezů jiných
souborů, žádný vlastní záznam složitosti; dotaz na counterWidgetState má nulový
výsledek. Filtr cesty zahrnuje i související soubory, proto osm výsledků nelze
zaměnit za osm závad ItemsGrid. Metadata všech dotazů odpovídají témuž snímku.

Finální úplná analýza trvala 86,16 s při špičce 3 506 168 KiB RSS; sekundární scan
trval 20,632 s. Vlastní audit AigisCode trval 2,99 s při 158 372 KiB RSS a zpracoval
118 sekundárních vstupů bez sekundární mezery. Jde o jednotlivá pozorování na
sdíleném stroji. Draivix stále vykazuje 792 skriptově omezených Vue souborů,
nativní pokrytí je neúplné, guard zůstává `Block` a obě analytická CLI končí 1.

Produkční sestavení prošlo. Osm nových regresí a dvě přemístěné existující kontroly
nebyly spuštěny; automatické testy a CI zůstávají podle Davidova pokynu pozastavené.
Úplné hodnocení šablon, přesnější posouzení mezí smyček a provozního kontextu,
čerstvá akceptace živého Draivix a celkové uzavření Q01–Q12 tím nejsou prokázány.
