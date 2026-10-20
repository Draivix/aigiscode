# Verifikační matice – co musí sedět, aby nález dostal `confirmed`

`verify.py` čte registr a pro každý `candidate` spustí kontroly podle `kind`. Status se mění jen podle téhle tabulky, nikdy ručně bez záznamu v `history`.

| kind | kontrola A (deterministická) | kontrola B (nezávislá) | `confirmed` když | `rejected` když | jinak |
|---|---|---|---|---|---|
| `clone_exact` | normalizovaný `ast_hash` shoda **nebo** CPD skupina | – | A | A selže a `quote` se liší víc než názvy/literály | `needs_human` |
| `clone_semantic` | embedding cosine ≥ 0,80 a stejný návratový typ | pass-5 verifier `safe`/`safe_with_adapter` | A ∧ B | B = `unsafe` | `needs_human` |
| `variant_mergeable` | `behavior_differences[]` neprázdné a každý `input` je spustitelný popis | pass-5 verifier `safe_with_adapter` | A ∧ B | B = `unsafe` | `needs_human` |
| `dead_symbol` | aigiscode `reachable_from_entry = 0` **a** (dead-code-detector = 1 **nebo** unused-public = 1) | grep na dynamické vzory negativní **a** coverage = 0 (pokud coverage existuje) | A ∧ B ∧ pass-5 `dead` | aigiscode dosažitelné **nebo** grep najde string/reflexi | `needs_human` (doporučit tombstone) |
| `dead_branch` | PHPStan/Psalm hlásí unreachable na stejných řádcích | `quote` sedí | A ∧ B | PHPStan nehlásí a verifier najde cestu | `needs_human` |
| `unused_param` | PHPStan `unused parameter` / Psalm `UnusedParam` na stejném symbolu | – | A | symbol implementuje interface/abstract (parametr je kontrakt) | `needs_human` |
| `redundant_layer` | tělo metody = jedno volání + return (AST check) | žádný interceptor/decorator/middleware na té třídě (grep DI config) | A ∧ B | tělo dělá cokoli navíc (log, transformace, try/catch) | `needs_human` |
| `repeated_validation` | oba `evidence` body leží na jedné cestě v grafu | – | A ∧ `quote` sedí | body na různých cestách | `needs_human` |
| `double_path` | capability má ≥ 2 implementace, které se vzájemně nevolají (graf) | pass-5 verifier popíše rozdíly | A | implementace se volají (je to jedna cesta) | `needs_human` |
| `unused_dependency` | composer-unused **nebo** PHPStan unused property/constructor param | – | A | – | `needs_human` |
| `suspicious` | – | – | nikdy automaticky | – | vždy `needs_human` |

## Společné podmínky pro všechny kinds
- `evidence[].quote` musí být **doslovně** nalezen v souboru v rozsahu `line_start-2 … line_end+2` (tolerance posunu). Jinak `rejected: evidence_mismatch` a nález jde zpět G přes push-back.
- Symboly v `symbols[]` musí existovat v `symbols.sqlite`. Neexistující → `rejected: hallucinated_symbol`.
- `estimated_loc_saved` se přepočítá z `symbols.sqlite` (součet LOC ne-kanonických členů); hodnota od G se ignoruje, ukládá se jen pro statistiku kalibrace G.

## Kalibrace G (sleduje `merge_registry.py --calibration`)
Po každém kole: kolik `high` nálezů skončilo `rejected`? Pokud > 10 %, snižuju váhu `confidence` od G pro další kolo a zvyšuju spot-check na 20 %.
