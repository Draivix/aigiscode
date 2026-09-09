# Pluginy používají zdroje zachycené parserem

WordPress a Signals již po parsování znovu nečtou zdrojové soubory z disku.
`RepoContext` dostává stejné vypůjčené bajty jako parser a poskytuje společný
líně vytvářený index řádků. Odpadá kopírování zdrojových řetězců do dvou lokálních
cache i dosavadní nahrazování chyby čtení prázdným obsahem. Výřezy odmítají nulové
a neplatné řádky a bezpečně omezují kontext na konec zdroje. Sémantická revize 12
zabraňuje opětovnému použití grafů vzniklých podle starého kontraktu pluginů.

Jde o opravu vlastnictví vstupů popsanou v [kontraktu](INPUT_CAPTURE_CONTRACT.md).
Nezajišťuje atomické zachycení celého repozitáře; konfigurace a závěrečná kontrola
stability vstupní sady zůstávají otevřené.

Produkční sestavení uspělo za 48,39 s. Skutečná CLI analýza zachované kopie Draivixu
zpracovala 17 067 podporovaných zdrojů za 83,35 s při špičce 3 508 132 KiB RSS.
Výsledkem je 130 081 symbolů a 328 144 hran. Celý sémantický graf o 793 166 985
bajtech a celý surový výstup scanneru s 10 031 stopami jsou bajtově shodné
s předchozí analýzou. Otisky, příkaz, surové výstupy a inventář zachycuje
[doklad](2026-09-10-plugin-capture-evidence.json).

CLI skončilo stavem 1 kvůli neúplnému nativnímu pokrytí: 80 zotavených zdrojů,
792 skriptově omezených Vue souborů a 146 nepodporovaných zdrojů. Shoda grafu
neznamená, že všechny jeho hrany jsou správně. Kontrola konkrétních signal hran
odhalila pět již existujících chybných vazeb:

- `EmailAccountForm.vue:235` volá `oauthAdapter.value.connect(oauthRedirectUri.value)`.
  Plugin vytvoří odběr signálu `value` s cílem v nesouvisející PHP funkci `value`
  v `docs/evidence/2026-07-12-manufacturing-audit/tools/build-module-surface-scenario-map.php:126`.
- Čtyři volání WebSocket `liveSocket.value.send(...)` v `ChatVoiceComposer.vue`
  pak přes tentýž zkrácený název přijímače míří do této PHP funkce.

Tyto hrany se touto opravou nezměnily; vyžadují opravu rozlišení callbacku
a identity přijímače v pluginu Signals. Doklad obsahuje všech pět úplných hran.
WordPress hrany v tomto korpusu nejsou, takže tento běh jeho chování přímo
neověřuje.

Regresní test výřezů byl přidán a existující WordPress test nyní zachycuje zdroj
před zkrácením a odstraněním souboru. Testy nebyly spuštěny; CI zůstává podle
Davidova pokynu pozastavené. Selhání souborů nebyla lokálně uměle vyvolávána.

Po auditu všech 23 296 cest, velikostí a SHA-256 kopie odpovídá zachovanému
inventáři; konfigurace scan/doctrine se nezměnily. V této jednotce nebyly změněny
zdroje kopie ani původního Draivixu. Nejde o nový snímek současného originálu.
Celková akceptace Q01–Q12 zůstává otevřená.
