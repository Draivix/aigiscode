# Sdílené zachycení konfigurace a ověření před publikací

Scan, resolver a policy/doctrine dříve četly konfiguraci samostatně v různých
fázích. Hotová analýza nekontrolovala pozdější změny celého inventáře.
Při editaci za běhu tak mohly být vedle sebe údaje získané z různých verzí
stejného vstupu.

Nyní tyto vrstvy sdílejí bajty nebo nepřítomnost souboru podle normalizované
absolutní cesty. Jednou přečtená konfigurace se během analýzy nezamění za
novější obsah. Zachycené kontroly adresářů zahrnují také balíčky rozšířené
TypeScript konfigurace a kořeny Python/Ruby; jejich existence patří do otisku
resolveru. Otisk scope výslovně zahrnuje `skip_hidden`. Změna nepřidává
závislost ani cestu artefaktu.

Před dokončením grafu či analýzy se kontroluje shoda konfigurace s počátečním
inventářem, znovu projde celý přijatý inventář a následně porovná zachycená
konfigurace i adresářové podmínky. Stejná kontrola probíhá před zveřejněním
připravené generace, před vrácením MCP stavu a paměťového agentního kontextu.
Zjištěná změna vrací `InputChanged`. Writer pak ponechá předchozí generaci
vybranou a odstraní nepublikovaný staging; watcher rozpozná i tuto zabalenou
chybu a zařadí nové zachycení. Podmínky a hranice stanoví
[kontrakt](INPUT_CAPTURE_CONTRACT.md).

Finální produkční sestavení prošlo bez varování za 49,89 s. Stejná binárka
provedla skutečný plný audit zachované kopie Draivixu za 85,33 s při
3 566 292 KiB peak RSS. Fáze `VerifyInputs` trvala 305 ms; kontrola při
publikaci je zahrnutá v celkovém čase. Předchozí běh trval 85,37 s.
Jde o jednotlivá pozorování na sdíleném stroji, nikoli benchmark režie.

Všechna věcná pole deterministických nálezů a všechny souhrnné počty jsou
shodné s předchozím sestavením. Mění se pouze `timings`. Celé sémantické,
dependency a evidence grafy, inventář kontraktů i sekundární scanner jsou
bajtově shodné. Zůstává 17 067 podporovaných zdrojů, 132 657 symbolů,
1 097 074 referencí, 331 060 hran, 19 silných a 22 celkových cyklických
komponent a 10 031 sekundárních nálezů. Otisky sestavení, scope a resolveru
se změnily očekávaně; otisky zdrojů, inventáře, sémantického prostředí
a policy/doctrine se nezměnily.

MCP obnovilo nativní analýzu za 98 ms a navazující validace trvala 268 ms.
První užitečná odpověď přišla za 18,04 s oproti předchozím 16,61 s; tuto
celou odchylku nelze připsat samotné kontrole vstupů. Všech 13 požadavků
prošlo a proces skončil 0. Celé odpovědi na tři lokální třídy `FakePw`,
jejich použití, návrh modulu, cykly, coverage a quality jsou stejné.
Přehled se liší pouze cestami generace, freshness a baseline guardu;
samostatný guard pouze identitou baseline. `inputs_match_index` je `true`.
Watcher byl vypnutý a baseline nadále hlásí `not_compared` kvůli pokrytí.

Všech 23 296 souborů kopie odpovídá uloženému inventáři podle cest, velikostí
a SHA-256. Konfigurace je beze změny. Původní Draivix nebyl upraven ani
znovu zachycen. Vlastní audit stejnou binárkou trval 3,61 s při 169 476 KiB
RSS: 123 podporovaných souborů, 2 875 symbolů a 6 361 hran. Sekundární
pokrytí je úplné, dvě známá zotavení Rust parseru a dva nepodporované
instalační skripty zůstávají. Vlastní audit předchází tomuto dokumentu,
dokladu a poslední aktualizaci akceptačního odstavce.

Dvě nové regrese pokrývají skutečnou aliasovou vazbu při změně konfigurace
mezi scanem a resolverem, následné odmítnutí capture, změnu datového souboru,
nový/odstraněný vstup, vznik dříve nepřítomné policy, prázdný adresář
resolveru a zachování výjimky pro generované výstupy. Rozšířená regrese
doctrine ověřuje zachování předchozí publikované generace při odmítnutí
změněného vstupu. Očekávání časových fází jsou upravená. Tyto testy nebyly
spuštěné ani sestavené jako testové cíle; CI a lokální kontroly zůstávají
pozastavené. Stabilní běh dokládá zachování výsledků, nikoli běhové
ověření uvedených chybových a souběžných větví.

Validace není atomický snapshot filesystemu. Změny po poslední kontrole
daného vstupu nebo celé změny a návraty mezi kontrolami mohou uniknout.
Doplňkový dead-code průchod a externí programy mají vlastní čtení mimo
tento zachycený soubor vstupů; jejich úplnost tím není prokázaná.
Výstupní adresář musí být mimo přijaté vstupy, jinak vlastní zápis změní
inventář a validace selže. CLI nadále končí 1 kvůli 80 zotaveným zdrojům,
792 skriptově omezeným Vue souborům a 146 nepodporovaným zdrojům.
Souběžná akceptace Q05, celková Q01–Q12, inkrementalita a schválené CI
zůstávají otevřené.

[Strojový doklad](2026-09-10-input-stability-evidence.json) obsahuje otisky
implementace a binárky, celé porovnání polí/artefaktů, časy, změny identity,
parametry a otisky skutečných MCP odpovědí i kontrolu integrity kopie.
