# Kontrola identity vstupů před parsováním a použitím cache

Parser a fast-load nyní ověřují, že načtené zdrojové bajty odpovídají scanu.
Změněný nebo zmizelý zdroj vyvolá typovanou chybu; watch indexer ji sám zaznamená
jako novější změnu a naplánuje další capture. Chyby čtení deklarovaných konfigurací
již nezastupuje nulový hash. Cache navíc vyžaduje shodný úplný inventář přijatých
vstupů a otisk deklarovaného prostředí, který se nově ukládá do identity snímku.

[Kontrakt](INPUT_CAPTURE_CONTRACT.md) rozlišuje tuto kontrolu od dosud nedokončené
stability celé analýzy. [Doklad](2026-09-10-capture-identity-evidence.json) obsahuje
otisky a skutečná měření. Q01–Q12 nejsou touto změnou uzavřené a CI zůstává
pozastavené podle Davidova pokynu.

Pro ověření vznikla další oddělená kopie zachycení 23 296 souborů. Pouze
`clients/mitel/i18n/cs_CZ/Task.json` zpočátku obsahoval starší zachycenou verzi
o 1 769 bytech. Po prvním MCP přehledu byla nahrazena novější verzí o 2 543 bytech.
Jde o skutečnou změnu překladového vstupu; zdroje PHP/JS/TS zůstaly stejné.
Původní Draivix ani zachované korpusy nebyly upraveny.

| Pozorování | Výsledek |
| --- | --- |
| Produkční sestavení | 47,82 s, úspěšné |
| Úplná CLI analýza 17 067 podporovaných zdrojů | 81,42 s; 3 499 704 KiB peak RSS |
| První použitelný přehled MCP nad nezměněnou kopií | 44,56 s; doložený fast-load, `inputs_match_index: true` |
| Aktualizace po změně JSON vstupu | 53,78 s; doložený plný Parse/Resolve, index 1 → 6 |
| Vztah nové analýzy k původní diskové generaci | `inputs_match_index: false` |

Inventární otisk se změnil z `0e4f0147b5e7f76c` na `cfca57f2dc4e058e`.
Otisk parsovaných zdrojů zůstal `6f443d3783ff1c41a49651c388b3201f` a otisk
deklarovaného prostředí `26806311af4518c26d16c38049fca478`. Server tak odlišil
nezměněný kód od změněné identity celého přijatého korpusu. Trace potvrzuje jedno
načtení grafu z cache a následné nové parsování po změně datového souboru.

Tato podmínka cache je záměrně konzervativní. Měření nedokazuje, že starší kód
kvůli tomuto překladu vytvořil nesprávnou hranu. Naopak celý sémantický graf
a surový `ast-grep-scan.json` nové CLI analýzy jsou bajtově shodné s předchozím
auditem zachovaného čerstvého korpusu. Úplné MCP pohledy findings, coverage a quality
jsou před změnou JSON a po ní rovněž bajtově shodné. Vedlejší scanner použil všech
17 067 nezměněných zdrojových výsledků znovu; nebylo nutné ho znovu provádět.
Plné parsování/rozlišení tedy probíhá i při některých datových změnách, které na
graf dnes nemají vliv. Uvedené časy jsou jednotlivá pozorování na sdíleném stroji,
nikoli statistické porovnání výkonu.

CLI skončilo 1 a guard zůstává blokující kvůli neúplnosti. Nativní pokrytí stále
uvádí 80 zotavených zdrojů, 792 skriptově omezených Vue souborů a 146 nepodporovaných
zdrojů. Úspěšné použití cache ani přepočet nejsou prohlášením o čistotě programu.
Konečná kopie i zachovaný čerstvý snímek mají všech 23 296 cest, velikostí a SHA-256
shodných s uloženým inventářem; scan a doctrine konfigurace se nezměnily.

Vlastní studený audit AigisCode trval 2,94 s při 160 584 KiB RSS a skončil 1;
nenahrazuje úplnou akceptaci ani schválené CI.

Tři nové regrese pokrývají změnu zdroje po scanu včetně stejně dlouhého obsahu
a odstranění, datové změny/přidání/odebrání při fast-load a neplatný typ konfigurace.
Existující watcher regrese byla rozšířena o změnu již existující skryté Cargo
konfigurace. Testy nebyly spuštěny. Chybové a souběžné větve proto zůstávají bez
schváleného CI ověření; zde nebyly nahrazeny uměle vyvolanými lokálními selháními.

Další konkrétní práce zůstává u pozdních filesystemových čtení pluginů pro
WordPress/signály, propojení konfigurací s jedním capture a závěrečné kontroly
stability celé vstupní sady. Nová kontrola před parsováním tyto pozdější fáze
neprohlašuje za atomické ani dokončené.
