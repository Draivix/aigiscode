# PASS 4 – DOMAIN (everything about one entity / zone)

This batch contains all code the topology map assigns to one domain zone (e.g. everything about `Order`, `Invoice`, `Customer`, `Notification`). The call graph is NOT your guide here – business meaning is.

Produce a **capability inventory** first, findings second.

## Step 1 – Capability inventory (`capabilities[]`)
List every distinct business capability this zone implements. A capability is an answer to "what can the system do with/for this entity?" – e.g. "create order from cart", "recalculate order totals", "cancel order", "send order confirmation", "export order to accounting", "validate VAT number".

For each capability:
- `name` (short, verb + object)
- `description` (one sentence)
- `implementations[]` – EVERY fqn that implements it, including partial ones, helpers that duplicate part of it, and entry points that trigger it
- `entry_points[]` – how it is reachable from outside
- `is_double_path: true` if `implementations` contains ≥ 2 independent implementations (not one calling the other)

You are given `known_registry.capabilities` from previous batches. Do not duplicate; **extend**: if this zone adds an implementation to an existing capability, output it with the existing `capability_id` and the new implementation. If you think two registry entries are really one capability, say so in `merge_suggestions[]`.

## Step 2 – Double paths → findings
Every `is_double_path` capability becomes a `double_path` finding with all implementations as `symbols`, evidence from each, and `behavior_differences[]` between the paths. State which path is canonical and what the others do that it does not.

## Step 3 – Zone-level waste
- **Orphan helpers**: functions in the zone that no capability uses → `dead_symbol` candidate (with `checks_done`).
- **Cross-zone leakage**: logic about another entity living here (e.g. invoice math inside Order) → `suspicious` with explanation.
- **Parallel models**: two DTO/entity/array shapes for the same concept → `clone_semantic`.
- **Config/constants duplicated** across files in the zone → `clone_exact`.

## Step 4 – Coverage statement
`files_fully_read[]` and `files_skimmed[]`. Be truthful. If you skimmed, say so – the director will re-batch skimmed files. A batch with everything in `files_fully_read` and only 3 findings will be sent back.

Rule of thumb for this pass: a domain of 30k+ lines has at least 10 capabilities and at least 2 double paths. If you found fewer, look at: validation, formatting, serialization/export, notification, state transitions, permission checks – these are where double paths hide.
