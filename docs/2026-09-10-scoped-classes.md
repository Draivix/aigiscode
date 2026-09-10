# Lokální třídy a typy JS/TS zůstávají ve svém scope

Lokální třídy v TypeScriptu měly ID sestavené pouze ze souboru a jména.
Stejnojmenné třídy v různých callbackách proto sdílely identitu i ID metod.
Oprava přidává deklaraci její scope a pozici; konstrukce `new` se váže přes
lexikální jméno. Metody lokálních tříd už nejsou kandidáty pro nesouvisející
souborové, globální ani callbackové vyhledávání. Doplňující pravidla pro přímé
metody, přepsané objekty, `this`, typové jméno a dědičnost popisuje
[kontrakt](LEXICAL_BINDING_CONTRACT.md).

Úplné čtení artefaktu potvrdilo odstranění všech 17 kolidujících TypeScript ID.
V grafu je 39 lokálních tříd, 2 997 scoped symbolů a 3 560 vazeb typových
či dědičnostních referencí. Indexy vazeb, cíle symbolů i vlastníci hran jsou
platné. Zůstává jediná kolize Python ID (`FakePw` ve třech lokálních definicích);
PHP a Python mají shodné posloupnosti souborů, symbolů, referencí a hran podle
SHA-256. Předchozí oprava místního `copyText` zůstala zachovaná.

`MockAnalyserNode` na řádcích 1 384 a 1 461 a `MockMediaStreamSource` na
řádcích 1 393 a 1 470 v `useCallsClientRuntimeBootstrap.spec.ts` mají oddělené
konstrukční vazby na řádcích 1 400 / 1 477 a 1 404 / 1 481. Konstrukce
v `CallsClient.spec.ts:828,840` míří na třídu z řádku 793, nikoli na
stejnojmennou deklaraci z řádku 706.

Výsledek má dopad na skutečnou architekturu grafu. Dvě slabé cyklické komponenty
vznikaly chybnými vazbami:

- `MapView.vue:1111` se vázal na `MockGeoJsonSource.setData` v
  `MapViewSelectionClear.spec.ts:55`; `MapView.vue:1299` na
  `getClusterExpansionZoom` v `MapViewCoordinateField.spec.ts:27`. Obě třídy
  jsou deklarované uvnitř callbacku `vi.mock('maplibre-gl', ...)`.
  Produkční komponenta nevolá tyto testové deklarace. Původní slabé vazby
  zmizely; skutečné importy komponenty z testů zůstaly.
- `useAgentUiActionDispatcher.ts:536` se typově vázal na `ApplyFilterPayload`
  v `useAiCommands.ts`, přestože dispatcher má vlastní alias tohoto jména
  na řádku 46. Zachycení aliasu nyní blokuje cizí odhad. Alias zatím nemá
  samostatný symbol; oprava netvrdí úplné rozlišení jeho struktury.

Nejde o opravu aplikačního zdroje ani vyřešení jeho reálných cyklů. Z grafu byly
odstraněny nesprávné vstupní hrany. Všech 19 silných cyklických komponent zůstalo
shodných. Celkový počet komponent klesl z 24 na 22.

Finální audit nad zachovanou kopií Draivixu zpracoval 17 067 podporovaných
souborů za 86,14 s při 3 581 160 KiB peak RSS. Počet symbolů a referencí zůstal
132 657 a 1 097 074; hran je 331 060 místo 331 131. Produkční sestavení finální
binárky prošlo za 49,50 s. Meziprůchody vedly k doplnění blokování neznámých
instancí a samostatného scope generických parametrů aliasu; doklad výsledků
se vztahuje k finální binárce.

MCP skutečně použil fast-load, měl `inputs_match_index: true` a první užitečný
přehled vrátil za 44,31 s. Všech 17 požadavků prošlo. `symbol_usages`
pro pět vybraných tříd potvrdil výše uvedená místa konstrukce; `module_design`
odlišil sedm kontejnerů v bootstrap souboru, včetně dvou samostatných
`MockAudioContext` se třemi metodami. `show_cycles` vrátil 19 silných a
22 celkových komponent. Jednalo se o jednorázový snímek s vypnutým watcherem,
nikoli doklad editací nebo souběhu.

Sekundární scanner je bajtově shodný a obsahuje 10 031 nálezů. Všech
23 296 souborů kopie nadále souhlasí se zachovaným inventářem cest, velikostí
a SHA-256; konfigurace se nezměnila. Vlastní audit finální binárkou trval
3,00 s při 162 192 KiB RSS. Dvě známá zotavení Rust parseru a dva
nepodporované instalační skripty přetrvávají; sekundární pokrytí vlastního
projektu je úplné. Časy jsou jednotlivá pozorování na sdíleném stroji.

CLI skončilo 1 kvůli známému neúplnému pokrytí: 80 zotavených zdrojů,
792 skriptově omezených Vue souborů a 146 nepodporovaných zdrojů. Dva regresní
scénáře jsou napsané, ale nebyly spuštěné ani sestavené jako testové cíle.
Testy, CI a lokální kontroly kvality zůstávají pozastavené. Tento korpus
neobsahoval kladnou přímou lexikální vazbu na vlastní metodu lokální třídy;
nové větve pro `this`, statické metody a instance jsou proto doložené pouze
sestavením a čtením kódu, nikoli kladným běhovým případem. Přímé vazby
neprokazují děděný dispatch, libovolné aliasy, návraty továren, změny globálů,
pořadí inicializace ani dynamickou náhradu metod.

Původní Draivix nebyl upraven ani znovu zachycován. Celková akceptace Q01–Q12,
stabilita capture a konfigurace, souběžný provoz, schválené CI a interaktivní
rychlost zůstávají otevřené. Měření, zdrojové kotvy a SHA-256 zachycuje
[strojový doklad](2026-09-10-scoped-classes-evidence.json).
