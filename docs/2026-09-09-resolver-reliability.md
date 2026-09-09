# Oprava importů a konfigurace resolveru — 2026-09-09

Nový skutečný audit celého AigisCode potvrdil opravu obou chybných importů
`Language` z Q01. `LanguagePage.tsx:21` a `LanguagesPage.tsx:18` nyní odkazují
na `interface:website/src/content/languages.ts:Language`, jehož deklarace začíná
na řádku 5. Původní cílový Rust enum už není jejich cílem. Nejde jen o odstranění
hrany nebo náhradu odkazem na celý modul.

Audit dokončil 109 zdrojových souborů, vytvořil 2 611 symbolů a 5 793 hran.
Ve výsledném grafu nebyla žádná hrana mezi nesouvisejícími jazykovými rodinami;
JavaScript a TypeScript se při tomto vyhodnocení považují za jednu rodinu.
Naměřeno 2,17 s, maximum RSS 148 976 KiB, návratový kód 0. To je měření tohoto
repozitáře, nikoli nové měření Draivix.

[Strojový důkaz](2026-09-09-resolver-reliability-evidence.json) obsahuje celé
importní hrany, cílovou deklaraci, hash binárky a hash zdrojového manifestu.
Úplné lokální artefakty jsou v `target/reliability-2026-09-09/self-q01-types/`.
Předchozí opravované chování popisuje [hloubkový audit Q01](2026-09-09-quality-reaudit.md).

## Implementace

- Import musí mít doložený cílový soubor ve stejné jazykové rodině. Nevyřešený
  pojmenovaný import neumožní ani navazujícím odkazům použít náhodný globální
  symbol. Samotná shoda jména importu už nevytváří hranu.
- Resolver načítá konfigurace v adresářích importujících souborů a jejich
  předcích. Rozlišuje `extends` a samostatné referencované projekty, zpracovává
  JSONC komentáře a koncové čárky, dědičnost cest a konfigurace z balíčků.
  Nečitelná, neplatná nebo cyklická konfigurace vrací chybu analýzy.
- Výběr aliasu respektuje přesnou shodu a nejdelší prefix před wildcardem;
  alternativní cíle se zkoušejí v uvedeném pořadí. Relativní importy používají
  pořadí kandidátů včetně náhrady přípon JavaScriptu TypeScriptem. Neexistuje
  automatické odhadování nezadaného aliasu z náhodného souboru v kořeni či `src`.
- Parser vytváří symboly interface, enum a abstraktních tříd. TSX používá TSX
  gramatiku. Dědičnost zaznamenává jednotlivé rodiče a odlišuje `implements`.
- Manifest přebírá fingerprint skutečně přečtených konfiguračních vstupů z
  resolveru. Nezískává jej dodatečným čtením jiného stavu disku při zápisu reportu.
  Fingerprint zahrnuje i nepřítomné kandidáty, zděděné konfigurace a reference.
  Sémantická revize je 9; starší cache proto nelze použít.
- Watcher registruje také rodiče použitých konfiguračních vstupů v ignorovaných
  adresářích nebo mimo kořen repozitáře. Změna těchto vstupů vyvolá nové sestavení
  registrací i grafu.

Pravidla pro relativní cesty a přepisování voleb vycházejí z dokumentace
[TypeScript extends](https://www.typescriptlang.org/tsconfig/extends.html).
Pořadí aliasů, náhrady přípon a fallback cílů popisuje
[TypeScript module resolution](https://www.typescriptlang.org/docs/handbook/modules/reference.html).
Reference jsou samostatné projekty podle
[dokumentace project references](https://www.typescriptlang.org/docs/handbook/project-references.html).

## Meze ověření a zbývající práce

Produkční release build prošel bez varování. Automatizované testy ani CI se podle
Davidova pokynu nespouštěly. Nový audit prokazuje konkrétní správné importy a stav
celého vlastního grafu; neprokazuje všechny kombinace JSONC, dědičnosti,
projektových referencí, cache ani souběžných filesystem událostí.

Resolver zatím není úplnou implementací TypeScript compiler resolution. Zůstávají
mezery například v podmíněných package exports, workspace package resolution,
`rootDirs`, type aliases a jednoznačném přiřazení souboru více projektům s
odlišnými konfiguracemi. Konfliktní konfigurace aliasů se nesmí rozhodnout
náhodným pořadím; takový cíl zůstává nevyřešený. Typové přípony a soubory se
indexují jen v rozsahu skutečně analyzovaných vstupů.

Při prvním běhu této opravy audit odhalil, že resolver již našel správný soubor,
ale parser v něm nevytvářel interface symbol. Tento mezikrok byl opraven před
výše uvedeným závěrečným měřením. Draivix zůstal beze změny; jeho nové úplné
měření a přijetí dalších bodů Q01–Q12 stále zbývá.
