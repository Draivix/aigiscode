# Místní funkce JS/TS místo globálních odhadů

Parser nově vytváří symboly pro funkce přiřazené proměnným a zaznamenává vazbu
volání na deklaraci podle skutečného AST scope. Parametr, stínicí proměnná nebo
přepsaná hodnota může mít explicitně neznámý cíl; resolver pak nesmí vybrat cizí
funkci podle jména. Podrobnosti, indexování referencí, práci s cache a hranice
podpory popisuje [kontrakt](LEXICAL_BINDING_CONTRACT.md).

Na zachované kopii Draivixu jsou nyní oba odkazy
`app/Modules/PasswordVault/Resources/web-addons/vault-addon/src/popup/app.ts:244`
a `:396` navázané na místní `copyText` z řádku 186. Mají `SameFile`, `Hard`
a důvod `call:lexical-binding`. Dříve mířily jako `Global/Inferred` na funkci
v `app/Modules/Website/resources/js/Pages/Cms/Show.vue`. Volání z řádku 244 má
nově také přesnějšího pojmenovaného vlastníka `renderResults`.

MCP načetl novou diskovou generaci se skutečně použitým fast-load a
`inputs_match_index: true`. `find_symbol` našel místní definici a `symbol_usages`
vrátil právě řádky 244 a 396 z popupu. Původní Vue funkci zachoval její dvě
vlastní použití na řádcích 1110 a 1117. Tím je doložen i pozitivní kontrolní
případ; nejde o plošné odstranění všech vazeb na jméno `copyText`.

| Pozorování | Výsledek |
| --- | --- |
| Finální produkční sestavení | 48,43 s, úspěšné |
| CLI audit 17 067 podporovaných zdrojů | 85,47 s; 3 550 780 KiB peak RSS |
| Symboly / reference / hrany | 130 737 / 1 093 353 / 328 501 |
| Nové symboly funkcí přiřazených proměnným | 656 |
| První použitelný přehled dokončeného MCP spojení | 43,99 s |
| Následné vyhledání / dva přehledy použití | 38,18 ms / 3,15 a 3,03 ms |

První MCP pokus použil výchozí čekání a po 30 s dostal typovanou odpověď
`index_state: indexing`, `retryable: true`, bez falešného hotového grafu. Klient
spojení ukončil. Nové spojení s explicitním čekáním do 120 s provedlo všech osm
požadavků bez chyby; oba servery skončily 0. Jde o jednorázové snímky s
`watcher: disabled`, nikoli o ověření souběžného watch provozu.

Čtení celého grafu zkontrolovalo 73 509 záznamů vazeb: 27 133 známých a 9 788
neznámých cílů volání, 379 známých a 36 209 neznámých vazeb prvního argumentu.
Všech 2 785 scoped symbolů existuje. Nebyl nalezen neplatný index, duplicitní
slot ani známý cíl mimo vlastní soubor. Tato kontrola referenční integrity sama
neprokazuje správnost všech pravidel scope. Úplné posloupnosti symbolů, referencí
a hran pro PHP a Python mají před změnou a po ní shodné SHA-256.
Surový výstup sekundárního scanneru zůstal bajtově shodný. Graf nadále obsahuje
79 opakovaných výskytů 64 identifikátorů symbolů z předchozího snímku; žádný
známý cíl nových vazeb na ně neodkazuje. Jejich původ tato oprava neřeší.

Všech 19 silných cyklických komponent zůstalo stejných. Celkový počet komponent
klesl z 29 na 24; některé skupiny se zmenšily. Počet architektonických upozornění
se naopak změnil ze tří na čtyři při přepočtu provázanosti. Nově se objevuje
`tests → clients`; jde o podnět k posouzení testovacích závislostí, nikoli o důkaz
nové chyby Draivixu. Jeho zdroje se nezměnily. Podrobné rozdíly obsahuje
[doklad](2026-09-10-lexical-bindings-evidence.json).

Vlastní audit AigisCode trval 2,96 s při 162 332 KiB RSS. Oba CLI audity skončily
1 s výslovným neúplným pokrytím; Draivix nadále obsahuje 80 zotavených zdrojů,
792 skriptově omezených Vue souborů a 146 nepodporovaných zdrojů. Uvedené časy
jsou jednotlivá měření na sdíleném stroji, ne statistický výkonový závěr.

Čtyři nové regrese pokrývají místní a importované funkce, bloky na jednom řádku,
stínění, přepsání hodnot, soukromé jméno funkčního výrazu, přepočet cache při
změně vazby a callbacky Signals. Testy nebyly spuštěny a CI zůstává pozastavené.
Konečný inventář všech 23 296 souborů kopie včetně SHA-256 stále odpovídá
zachovanému snímku; konfigurace se nezměnily. Původní Draivix nebyl upraven.
Celková akceptace Q01–Q12, stabilita celého capture a interaktivní rychlost
zůstávají otevřené.
