# Aktuální akceptace Q01–Q12 a nový snímek Draivix

**Celková akceptace zůstává otevřená.** Čerstvý audit s implementací `87e5c5a`
zpracoval 17 067 podporovaných zdrojů za 86,78 s při špičce 3 530 952 KiB RSS.
Devět skutečných dotazů MCP potvrdilo použitelnost vybraných nálezů, společnou
generaci a výslovné neúplné pokrytí. Zbývají nedoložené chybové a souběžné scénáře,
skutečná inkrementalita parsování a hodnocení i přesnost některých heuristik.
Tento dokument aktualizuje stav [původních podmínek](2026-09-09-quality-reaudit.md),
nemění jejich rozsah. Podrobnosti a otisky zachycuje
[strojový doklad](2026-09-09-current-acceptance-evidence.json).

[Kontrola identity capture z 10. září](2026-09-10-capture-identity.md) doplňuje
shodu parsovaných bajtů se scanem, přísnější klíč fast-load a explicitní chyby
čtení prostředí. Reálná změna JSON vstupu na kopii Draivixu cache odmítla;
sémantický graf a scanner na stabilním korpusu zůstaly shodné.
[Navazující oprava pluginů](2026-09-10-plugin-capture.md) odstraňuje jejich pozdní
čtení zdrojů z disku. Celková stabilita capture a schválené CI jsou nadále otevřené.
[Rozlišení signálů](2026-09-10-signal-bindings.md) dále odstranilo přesně pět
chybných vazeb Vue → PHP a jedno navazující upozornění na nestabilní závislost;
deterministické nálezy detektorů a inventář kontraktů zachovaného korpusu se nezměnily.

[Potvrzení editací z 10. září](2026-09-10-edit-receipts.md) nyní přiděluje revizi
ještě před doručením filesystemové události; `verify_change` umí počkat na tuto
revizi a zveřejňuje skutečný stav baseline. Reálný MCP běh potvrdil chování při
startu i po zápisu do běžícího indexu. Stabilita capture, chybové/souběžné scénáře
a schválené CI zůstávají součástí otevřené akceptace.

Navazující [inkrementální sekundární scan](2026-09-09-incremental-scanning.md)
zkrátil dvě řízené aktualizace ze zhruba 80 na 58 sekund při zachování porovnaných
MCP dat. Je volitelný a dosud bez schváleného CI; Q11 i celková akceptace zůstávají
otevřené. Nové měření rovněž dokládá okno před doručením události watcheru,
kdy okamžitý dotaz ještě vrátí starou revizi bez příznaku stale.

Nový snímek vznikl 9. září 2026 ve 20:18:04 UTC ze skutečného pracovního stromu
Draivix, včetně necommitovaných souborů. Obsahuje 23 296 běžných souborů o celkové
velikosti 301 695 383 bytů. Proti předchozímu korpusu přibylo 130 souborů,
53 se změnilo a žádný nezmizel; všech 183 cest leží pod `clients/mitel`.
Z toho přibylo 54 podporovaných zdrojů. Nejde o změny vytvořené tímto auditem.

Rozsah kopie je stejný jako u předchozího zachycení: bez závislostí, generovaných
oblastí, skrytých adresářů a symbolických odkazů; přesný seznam výluk je v dokladu.
Nejde o inventář určený Gitem. Kopie má vlastní prázdnou hranici Git a širokou
konfiguraci scanu, která pouze v kopii nahrazuje původní omezenou konfiguraci.
Existující doctrine byla zkopírována beze změny; policy a rules nebyly přítomny.

Dva úplné inventáře během zachycení souhlasily s hashi zkopírovaných souborů.
Po analýze a MCP dotazech další inventáře ve 20:44:58 UTC opět potvrdily shodu
všech 23 296 cest, velikostí a SHA-256 v kopii i původním stromu. Původní konfigurace,
HEAD a otisk pracovního stavu Git se nezměnily. Vyloučené odkazy nebyly přeneseny,
proto jejich seznam v kopii záměrně neodpovídá originálu. Jde o opakovaně ověřenou
stabilitu vymezeného rozsahu, nikoli atomický filesystem snapshot nebo trvalý
příslib čerstvosti živého repozitáře. Původní Draivix byl pouze čten.

| Podmínka | Doložený stav | Co zůstává otevřené |
| --- | --- | --- |
| Q01 — správné importy a konfigurace balíčku | Poslední vlastní audit stejnou binárkou obsahuje oba správné `TypeImport` cíle: `LanguagePage.tsx:21` a `LanguagesPage.tsx:18` míří na TypeScript interface v `content/languages.ts`, oba `Hard/ImportScoped`. Resolver omezuje importy jazykem a modulem; konfiguraci vybírá podle importujícího souboru a zpracovává JSONC, extends a references. | Úplná současná regresní matice aliasů a hranic balíčků nebyla po zastavení CI spuštěna. Správné dva cíle nejsou důkazem správnosti všech referencí. |
| Q02 — skutečný kód a kotvy nálezů | V témže vlastním auditu jsou nulové původní `SplitIdentityModel` a `CompatibilityScar` nálezy vlastněné `artifacts.rs`. Sdílený lexikální masker a kotvy detektoru nahrazují původní textový odhad. | Nové kombinace rozsahů a zachování produkčních pozitiv potřebují současnou regresní akceptaci. |
| Q03 — pravdivý stav externích scannerů | Implementovány typované chyby, validace reportů, individuální exit semantics a přenos neúplných externích kontrol do výsledku a guardu. | Tento nový audit externí nástroje nespouštěl (`not_requested`). Reálné procesní a formátové regrese po opravě nemají běh CI. |
| Q04 — výstupy a životní cyklus procesů | `external/process.rs` vlastní oddělené úplné raw soubory, omezený náhled a unixový úklid skupiny procesů. | Velké oba výstupy, timeout, selhání zápisu a potomci nejsou v současné verzi runtime ověřeni. Mimo Unix kontrakt zaručuje pouze přímé dítě. |
| Q05 — watcher a čerstvost vstupů | Watcher se armuje před analýzou, sdílí definici vstupů, zaznamenává změnu v callbacku a zveřejňuje svůj stav. Dřívější skutečné změny metadat a hooku Mitel vyvolaly nové revize. | Závody při startu/rebuildu, rename, konfigurace a degradace watcheru čekají na regresní ověření. Čerstvý MCP zde běžel jednorázově s `watcher: disabled`. |
| Q06 — počáteční stav MCP | Skutečný pomalý start vrátil po 30,01 s explicitní `indexing`, `retryable: true`, revision 0; použitelný přehled následoval po 45,75 s s revision 1. Implementace rozlišuje pending/ready/failed a probouzí čekající při chybě. | Tento běh nedokládá vynucené selhání prvotní analýzy ani všechny deadline scénáře. |
| Q07 — neúplné parsování | CLI a všechny odpovědi MCP zveřejňují 80 zotavených zdrojů, 792 skriptově omezených Vue souborů a 146 nepodporovaných zdrojů. Kontroly založené na absenci jsou odložené, guard je `Block`, analytické CLI končí 1. | Úplné parsování tohoto korpusu dosaženo není; uvedení mezery samo neřeší chybějící jazykovou či šablonovou sémantiku. |
| Q08 — zachování doctrine při externí analýze | `ProjectAnalysis` uchovává načtený kontext a oba CLI toky používají společné `set_external_analysis`. Chyba požadované konfigurace se propaguje. V nové kopii je původní doctrine zachována. | Přítomnost validní doctrine neověřuje současnou kombinaci externího selhání, neplatné doctrine a analýzy bez zápisu; integrační regrese zůstávají nespouštěné. |
| Q09 — baseline a vynucení výsledku | Nová generace má `missing/initial_snapshot`, `previous: null`, všech 3 435 konsolidovaných záznamů `FirstObserved`, nulové počty nových/zhoršených nálezů a null grafové i kontraktové delty. CLI rozlišuje neúplné kontroly od úspěšného reportování. | Neplatné, nesrovnatelné a souběžně publikované baseline potřebují aktuální regresní běh. Samotný dokončený reporting s verdiktem `Block` není univerzální vynucující CI brána. |
| Q10 — zelené CI | Původní blokátory byly odstraněny: historický běh na `2721fcb` úspěšně provedl 408 testů ještě před Davidovým zastavením testů. | Commit `87e5c5a` nemá CI běh. Historický úspěch ani produkční build nepotvrzují současnou regresní sadu. |
| Q11 — výkon, změny, souběh a generace | Vlastní audit: 118 podporovaných vstupů, 2,99 s. Nový Draivix: 17 067, 86,78 s; MCP 45,75 s. Dřívější dvě skutečné změny: 51,98/62,13 s. Publikace má společnou generaci 20 členů a nativní čtenáři ji připínají; všech devět nových MCP odpovědí ukazuje shodnou generaci. | Opt-in cache resolveru není inkrementální parsování a hodnocení; běžně je vypnutá. Chybí měření paměti pod více současnými klienty, důkaz chování při přerušení a souběžných zápisech i přijatelná interaktivní odezva. |
| Q12 — odpovědnosti a sekundární pokrytí | Odděleny konkrétní moduly pro externí procesy/reporty, tsconfig, baseline, publikaci, triage, Kuzu a složitost. Sekundární scanner má u všech 17 067 vstupů stav scan/prefilter, žádnou velikostní výluku. Volitelná Kuzu větev již používá nativní Rust/C++ integraci. | Vue zůstává omezené na skripty. Nativní Cypher byl reálně ověřen na předchozím korpusu, nikoli této nové generaci. Další soustředěné odpovědnosti a celková kvalita implementace tím nejsou automaticky vyřešeny. |

Historický [úspěšný běh CI](https://github.com/Draivix/aigiscode/actions/runs/34326319636)
byl dokončen 9. září v 07:58 UTC. Jeho stav byl nyní pouze přečten; nový běh nebyl
vyvolán. Na základě Davidova pokynu zůstávají automatické testy i CI zastavené.

Zdrojová kontrola nového korpusu přináší konkrétní podklady pro rozhodování:

| Zdroj a kotva | Význam pro opravu |
| --- | --- |
| `GlobalSearch.vue:1143` | Konstrukce regexu je skutečně uvnitř smyčky přes slova dotazu. MCP vrací správnou kotvu a označení `heuristic`. Kandidát pro měření opakované konstrukce; úprava musí zachovat escapování a zvýraznění. |
| `ReportFilterEditor.vue:98,104,124` | Opakované `find` v katalogu polí a `some` nad narůstajícím seznamem mají doložené operace. Pro velké vstupy má smysl prověřit index polí a množinu ID, při zachování porovnávání přes `String` a pořadí. |
| `clients/mitel/Hooks/Task/TaskWorkflowHook.php:315–329` | Pro každý výjezd probíhá `findBy` protokolů a následný průchod do prvního podepsaného protokolu. Detektor kotví vnitřní smyčku na 329; ruční čtení navíc ukazuje dotaz na 316. Ověřit počty výjezdů a SQL dotazů před rozhodnutím o dávkovém načtení. Průchod podřízenými seznamy sám neprokazuje kvadratickou složitost. |
| `clients/mitel/Services/TaskRejectAndDiscardInvoiceService.php:156–173` | Vnější seznam obsahuje přesně čtyři závislosti, každý filtr má jednu nebo dvě položky. Záznam vnořené iterace na 168 zůstává viditelný, ale není důkazem superlineárního růstu. Ochranné kontroly se nemají odstraňovat podle tohoto varování. |
| `clients/mitel/Console/Commands/WorkflowSyncCommand.php:353–369` | Vnořený průchod tiskne rozdíly konfigurace při administrátorském příkazu. Bez měření četnosti a objemu jej nelze označit za problém uživatelské odezvy. |
| `clients/mitel/tools/upload_brand_logo.py:57–70` | Dvě přihlašovací možnosti obsahují nejvýše dvacet kontrol stavu. Jde o omezené čekání v pomocném browser skriptu, nikoli neomezenou kvadratickou smyčku. Skript nebyl spuštěn. |
| `ItemsGrid.vue` a `counterWidgetState.ts` | Původní malé pevné seznamy stále nevytvářejí vlastní nález složitosti. MCP filtr ItemsGrid vrací osm nálezů jiných primárních souborů, kde je komponenta pouze související cesta; counterWidgetState vrací nulu. |

Celkem je nyní 1 606 záznamů složitosti proti 1 599 v předchozí kopii. Ruční
pomocný výběr podle druhu, cesty a podtypu našel osm nových skupin; není to úplný
diff všech jednotlivých nálezů a neznamená osm nových runtime závad. Surový scanner
obsahuje 10 031 stop proti 9 978. Část nových zdrojů slouží přípravě ukázek nebo
provozním pomocným úlohám; jejich viditelnost se nesmí zaměňovat s prioritou oprav.
Přesnější vyhodnocení pevných mezí a provozního kontextu je další konkrétní mezera.

MCP `quality_evaluation` popisuje 4 183 viditelných nálezů jako částečné důkazy.
Tento počet se liší od 3 435 konsolidovaných otisků v convergence a od 1 606
záznamů jedné rodiny; nejde o tři alternativní počty prokázaných závad.
Pokrytí uvádí také 691 530 nevyřešených referenčních míst. Shoda jména s lokálním
symbolem u 237 124 z nich je vodítko pro resolver audit, nikoli důkaz dostupného
správného cíle. Zbývajících 454 406 je klasifikováno jako externí/stdlib podle
absence takové shody; ani to samo neprokazuje původ každé reference.

Měření jsou jednotlivá pozorování na sdíleném stroji. Nový korpus se od starého
liší, takže 86,16 → 86,78 s není čisté porovnání výkonu implementací. Opravy
Draivixu, spuštění jeho pomocných skriptů ani obnovení testů tento audit neprovádí.
Požadavek na spolehlivý, přesný a použitelný analyzer proto zůstává otevřený;
aktuální důkazy neodůvodňují tvrzení o stoprocentní spolehlivosti.
