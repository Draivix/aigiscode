# PASS 3 – HORIZONTAL (semantic neighbors, no call-graph edge between them)

This batch packs **several candidate groups** (`groups[]`, each with a `cluster_id`) of functions/methods that a similarity index flagged as possibly doing the same thing. Members of a group are usually NOT connected by any call: each was written without knowledge of the others. That is exactly why they exist.

Answer **every** group in the batch; a group you skip counts as a missing answer. For each group, do this in order:

1. **Normalize mentally.** Rename variables to `$a,$b,…`, replace literals with `LIT`, ignore comments and formatting. Now compare behavior, not text.
2. **Classify the whole group** with one entry in `group_verdicts[]` (`cluster_id`, `verdict` ∈ `duplicate_exact | duplicate_semantic | variant_mergeable | distinct | mixed`, `canonical_candidate`). If `mixed`, split into sub-groups in that entry's `subgroups[]` and classify each. Every finding you emit for the group names the `cluster_id` in its rationale.
   **A verdict is not a finding.** Every group whose verdict is `duplicate_exact`, `duplicate_semantic` or `variant_mergeable`, and every such subgroup of a `mixed` group, MUST also appear as at least one entry in `findings[]` (kind matching the verdict, all member symbols, verbatim evidence from each member, `behavior_differences` for variants, `cluster_id` in the rationale). The director counts verdicts against findings; a mergeable verdict without its finding is an unanswered group and the batch is sent back.
3. **For every pair you call equivalent**, state the equivalence as a claim: "for all inputs X, A(X) == B(X)" and list the edge cases you checked: null, empty array/string, negative numbers, timezone, locale, rounding, exceptions thrown vs. returned null, mutation of arguments, side effects (logging, DB, events).
4. **For `variant_mergeable`**, `behavior_differences[]` is mandatory and must be concrete: input → A gives / B gives. "Slightly different" is rejected.
5. **Pick `canonical_candidate`**: the implementation that should survive. Justify with: most callers, best tests, most complete edge-case handling, lives in the right layer. If none is good, say `none` and explain.
6. **Caller impact**: for each non-canonical member, using `graph_context`, list how many callers would need to move and whether any caller depends on the behavioral difference.
7. **Test coverage**: list which members have tests in the material. A canonical candidate without tests is a `low`-confidence recommendation.
8. **Check the registry.** `known_registry.capabilities` lists capabilities already discovered. If this group implements one of them, reference its `capability_id`. If it is a new capability, propose it in `new_capabilities[]` (name, one-sentence description, implementations).

Singletons: if the batch includes a `singletons[]` list (symbols with no cluster), for each one answer `has_twin_in_registry: true|false|unsure` with the registry entry it matches. Do not analyze singleton bodies for anything else.

A group where you cannot decide is a finding too: `suspicious` with `confidence: low` and a precise `open_questions` entry saying what evidence would settle it.
