# Signals: odstranění pěti chybných vazeb Vue → PHP

Plugin Signals již nepřevádí kvalifikovaný callback na jeho poslední jméno
a nehledá takto zkrácené jméno v celém repozitáři. Callback musí odpovídat místní
deklaraci nebo rozlišenému importu; kvalifikovaná metoda zachovává původ své třídy.
Přijímač uchovává celou cestu a původ importu včetně exportovaného jména, takže
aliasy lze propojit bez směšování nesouvisejících stejnojmenných exportů.
Sémantická revize 13 zneplatňuje staré grafy. Přesné hranice popisuje
[kontrakt pluginů](PLUGIN_SYSTEM.md#signal-binding-evidence).

Reálný audit zachované kopie Draivixu odstranil právě těchto pět vazeb:

- `EmailAccountForm.vue:235`: `oauthAdapter.value.connect(oauthRedirectUri.value)`
  již nevytváří odběr signálu s cílem v nesouvisející PHP funkci `value`.
- `ChatVoiceComposer.vue:1048`, `1600`, `2488` a `2700`: volání
  `liveSocket.value.send(...)` již přes společný člen `value` nepublikují do této
  PHP funkce v `build-module-surface-scenario-map.php:126`.

Porovnání zpracovalo oba celé grafy. Ze starého odstranilo pouze pět přesně
zaznamenaných serializovaných hran, každou právě jednou; takto upravené bajty
odpovídají celému novému grafu velikostí i SHA-256. Neprovádělo řazení ani vyloučení
metadat. Nový graf má 793 163 640 bajtů a 328 139 hran, předchozí měl 328 144 hran.
Celý surový výstup sekundárního scanneru zůstal bajtově shodný.

Odstranění falešného spojení `resources → docs` opravilo související metriky
provázanosti. Zmizelo upozornění `UnstableDependency` pro `clients → resources`
a závažnost upozornění `app → resources` klesla z 338 na 302. Počet těchto
architektonických upozornění klesl ze čtyř na tři. Změny analýzy grafu se omezují
na tato upozornění, metriky provázanosti a počet hran. Nezměnily se celé oddíly architektonického hodnocení,
dead code, hardwiring, bezpečnostních nálezů, scanneru ani inventáře kontraktů.

Produkční sestavení uspělo za 46,89 s. CLI audit 17 067 podporovaných zdrojů trval
83,16 s při špičce 3 501 508 KiB RSS. Zachoval 130 081 symbolů a 1 092 834
referencí. Skončil stavem 1 s výslovným neúplným nativním a sekundárním pokrytím.
[Doklad](2026-09-10-signal-bindings-evidence.json) uchovává surové výstupy, otisky,
všech pět odstraněných hran, postup porovnání a změny návazných metrik.

Dva nové regresní testy pokrývají nesouvisející callbacky, celé přijímače,
návratovou hodnotu factory a kvalifikované callbacky z modulů či tříd. Existující
Python test byl rozšířen o alias a stejnojmenný export z jiného modulu. Testy
nebyly spuštěny; CI zůstává pozastavené. Tento skutečný korpus dokládá odstranění
falešných vazeb, nenahrazuje ověření pozitivních fixture případů.

Všech 23 296 souborů kopie včetně velikostí a SHA-256 stále odpovídá zachovanému
inventáři; konfigurace se nezměnily. Původní Draivix ani zdroje kopie nebyly
upraveny. Dynamické aliasy instancí, přepisování lokálních vazeb a jejich stínění
nejsou tímto file/import modelem prokázané. Celková akceptace Q01–Q12 zůstává otevřená.
