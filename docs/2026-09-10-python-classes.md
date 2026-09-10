# Oddělení lokálních tříd Pythonu

Tři třídy `FakePw` v `tools/satellite/tests/test_authorized_keys_audit.py`
měly společné ID, přestože jsou definované ve třech různých funkcích. Oprava
přidává lokální třídě skutečného lexikálního rodiče a pozici deklarace.
Konstrukční volání se váže přes Python scope; lokální třída ani její potomci
nejsou kandidáty pro cizí souborové, globální nebo callbackové vyhledávání.
Rozsah a omezení stanoví [kontrakt](PYTHON_LOCAL_CLASS_CONTRACT.md).

| Deklarace `FakePw` | Vlastník | Konstrukce |
| --- | --- | --- |
| Řádek 99 | `test_audit_account_reads_real_home` | Řádek 104 |
| Řádek 121 | `test_audit_account_symlinked_authorized_keys_refused` | Řádek 126 |
| Řádek 133 | `test_audit_account_missing_ssh_dir_is_empty` | Řádek 138 |

Úplné čtení finálního grafu potvrdilo nulový počet duplicitních symbolových ID
a platné cíle i vlastníky všech hran. Tři nové callee bindingy směřují na
vlastní třídy; celkový počet scoped symbolů vzrostl z 2 997 na 3 000.
Posloupnosti souborů, symbolů, referencí a hran PHP i JS/TS jsou shodné podle
SHA-256.

Tyto zdroje byly pouze analyzované; jejich testovací funkce se nespouštěly.
Draivix nebyl upraven. Oprava tedy mění rekonstrukci grafu, nikoli chování
aplikace nebo výsledek jejích testů.

Produkční sestavení prošlo za 49,24 s. Plný audit zachované kopie Draivixu
trval 85,82 s při 3 623 748 KiB peak RSS. Zpracoval 17 067 podporovaných
zdrojů, 132 657 symbolů, 1 097 074 referencí a 331 060 hran. Souhrnné počty
nálezů a cyklů zůstaly stejné jako před opravou.

MCP skutečně použil fast-load a vrátil `inputs_match_index: true`. První
užitečný přehled přišel za 44,90 s; všech 13 požadavků prošlo.
`symbol_usages` vrátil pro každou ze tří tříd právě jednu hranu na vlastní
místo konstrukce a `module_design` tři samostatné kontejnery. Watcher byl
vypnutý; není to doklad editací ani souběžného provozu.

Sekundární scanner je bajtově shodný a obsahuje 10 031 nálezů. Obě celé
posloupnosti cyklů zůstaly stejné: 19 silných a 22 celkových komponent.
Všech 23 296 souborů kopie souhlasí s uloženým inventářem podle cest, velikostí
a SHA-256; konfigurace se nezměnila. Původní strom nebyl znovu zachycován.
Vlastní audit finální binárkou trval 3,09 s při 162 012 KiB RSS a znovu
ukázal pouze dvě známá zotavení Rust parseru a dva nepodporované instalační
skripty. Sekundární pokrytí vlastního projektu je úplné. Časy jsou jednotlivá
pozorování na sdíleném stroji.

Dva regresní scénáře jsou napsané a zkontrolované čtením, ale nebyly spuštěné
ani sestavené jako testové cíle. Zahrnují oddělené konstruktory a uzavřené scope,
stínění, `global`/`nonlocal`, výchozí parametry, typy, dědičnost, oddělení
namespace tříd od metod a neznámé comprehension. Složitější větve nemají tímto
pozorováním potvrzené běhové chování. Testy, CI a lokální kontroly kvality
zůstávají pozastavené.

Rozlišení instancí přes aliasy, továrny a dědičnost zůstává otevřené. Dosavadní
řetězcový typ nestačí k důkazu, kterou lokální třídu objekt nese. Nepodložený
member call proto nemá nahrazovat přesnou vazbu odhadem na cizí metodu.
Oprava také neprokazuje pořadí inicializace, dynamické přepisování, chování
metatříd, úplnou sémantiku comprehension ani verzované annotation scope.

CLI nadále končí 1 kvůli známému neúplnému pokrytí: 80 zotavených zdrojů,
792 skriptově omezených Vue souborů a 146 nepodporovaných zdrojů. Celková
akceptace Q01–Q12, stabilita capture a konfigurace, souběh, schválené CI a
interaktivní rychlost zůstávají otevřené. Podrobnosti a SHA-256 zachycuje
[strojový doklad](2026-09-10-python-classes-evidence.json).
