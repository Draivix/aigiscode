# PHP anonymní třídy mají vlastní identitu

Parser připisoval metody anonymních tříd okolnímu kontejneru. Na zachované kopii
Draivixu to způsobovalo 46 kolidujících PHP ID, celkem 56 opakovaných výskytů.
Nešlo o duplicitně navštívený zdroj: byly to různé definice ve skutečných tělech
`new class`. Oprava vytváří vlastní třídu už při parsování a přes ni vede
konstruktory, metody, dědičnost a typ `$this`. Argumenty konstruktoru ponechává
v okolním scope. Podrobnosti a omezení stanoví
[kontrakt](PHP_ANONYMOUS_CLASS_CONTRACT.md).

V `InvoiceHooksPaymentContractTest.php` jsou třídy na řádcích 27, 45 a 56 nyní
oddělené. Jejich konstruktory na řádcích 35, 48 a 63 už nesdílejí ID. Dědičnost
vede na `Entity`, `EntityManager` a `InvoiceAggregateService`; existující
mechanismus přepisování metod z těchto faktů nově vytváří odpovídající vazby.

Chyba se týkala také produkčního kódu. V
`app/Http/Controllers/Api/DebugController.php` byly `__construct` z řádku 182 a
`getRoles` z řádku 185 nesprávně označené jako metody `DebugController`.
Nově patří anonymnímu objektu z řádku 179. Skutečný konstruktor controlleru
z řádku 37 a jeho devět dalších metod zůstaly zachované. MCP `module_design`
vrací dva kontejnery s 10 a 2 metodami. Migrace Wiki
`2026_08_03_000100_create_wiki_pages_table.php:15` má vlastní třídu s metodami
`up` a `down`; odkaz na externí `Migration` zůstává referencí, protože vendor
třída není součástí analyzovaného zdroje.

| Pozorování nad stejnými vstupy | Výsledek |
| --- | --- |
| Plný CLI audit | 84,87 s; 3 591 064 KiB peak RSS |
| Podporované zdroje | 17 067 |
| Symboly | 130 737 → 132 657; přibylo 1 920 anonymních tříd |
| Reference / hrany | 1 097 074 / 331 131 |
| Kolize PHP ID | 46 → 0 |
| Zbývající kolidující ID | 17 TypeScript + 1 Python, celkem 23 opakování |
| MCP | 16 úspěšných požadavků; první použitelný přehled za 45,34 s |

Celý graf má platné cíle i vlastníky všech hran. Všech 1 920 nových tříd má
vazbu konstrukce. Úplné posloupnosti symbolů, referencí a hran ostatních jazyků
mají shodné SHA-256. Lexikální vazby JS/TS si po posunu indexů zachovaly platné
odkazy. Surový sekundární scan je bajtově shodný. Seznamy všech 24 cyklických
komponent, včetně 19 silných, zůstaly stejné. Změna nevytvořila ani neopravila
aplikační cyklus; zpřesnila jeho vstupní graf.

MCP skutečně použil fast-load a zveřejnil `inputs_match_index: true`.
`find_symbol` a `symbol_usages` odlišily tři účetní třídy, objekt v controlleru
i migraci; každé použití ukázalo na její vlastní místo konstrukce. Jednalo se
o jednorázový snímek s vypnutým watcherem. Časy jsou jednotlivá měření, nikoli
statistický výkonový závěr.

CLI skončilo 1 kvůli již známému neúplnému pokrytí: 80 zotavených zdrojů,
792 skriptově omezených Vue souborů a 146 nepodporovaných zdrojů. Nová regrese
zahrnuje dva objekty se stejným názvem metody, argument s okolním `$this`,
promovanou vlastnost, oddělení vnějšího scope, `self` a identity ve dvou souborech.
Byla napsána a při čtení opravena, ale nebyla spuštěna. Testy i CI zůstávají
pozastavené. Doklad rozlišuje binárku použitou při pozorování od závěrečného
sestavení po opravě testového tvrzení; produkční část parseru je shodná.
Závěrečné produkční sestavení prošlo za 48,17 s. Vlastní audit finální binárkou
trval 3,38 s při 160 440 KiB RSS; nadále hlásí dvě známá zotavení Rust parseru
a dva nepodporované instalační skripty, nikoli úplné pokrytí.

Všech 23 296 souborů kopie nadále souhlasí se zachovaným inventářem podle cest,
velikostí a SHA-256. Konfigurace se nezměnila. Původní Draivix nebyl upraven ani
znovu zachycován. Zbývající jazykové kolize, úplná stabilita capture, souběžný
provoz, interaktivní rychlost a celková akceptace Q01–Q12 zůstávají otevřené.
Naměřené hodnoty a otisky obsahuje
[strojový doklad](2026-09-10-anonymous-classes-evidence.json).
