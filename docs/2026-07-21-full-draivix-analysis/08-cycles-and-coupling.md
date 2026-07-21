# 08 — Cycles & Coupling

**8 strong cycles** this run. Two classes: one structural knot (redesign
territory) and seven small actionable pairs.

## The entity-core knot (redesign, not a sweep)

`Team → User → ActivityHistoryLoader → HasDynamicProperties → …` — the same
18-file `_Core` knot documented in the July rounds
(`draivix/2026-07-04-core-knot-deep-analysis.md`): entity-definition engine
mutually recursive with tenant DB layer (`EntityManager`, `EntityRegistry`,
`FieldLoader/Normalizer`, `RelationLoader`, `TenantManager`, `TenantDb`,
`SchemaAutoSync`), dominant relations `call/import/container_resolution`.

**Status:** still intact, no regression, no progress — it was never going to
move without the staged redesign those docs lay out. **Not re-analyzed here**;
the ledger references the existing plan instead of duplicating it.

## Seven small cycles (actionable)

| Pair | Shape |
|---|---|
| `SyncAccountJob ⇄ SyncFolderJob ⇄ EmailSyncManager` (105 edges) | Job↔manager mutual calls (chaining) |
| `DeleteExpiredMattermostMessageJob ⇄ MattermostExpiringMessageService` (58) | job↔service |
| `ProcessMattermostPerSpeakerRecordingManifestJob ⇄ ...ManifestService` (40) | job↔service |
| `SatelliteService ⇄ ServerSshKeyDeploymentService ⇄ SshKeyVaultMirror` | deployment chain |
| `AttachmentService ⇄ ThumbnailService` | service↔service |
| `ActivityHostAccessResolver ⇄ PermissionChecker` (79) | resolver needs permissions, permission layer needs resolver |
| `FilterJsonbHandler ⇄ FilterQueryBuilder` | filter pair |

**Pattern:** each is a two-way call where one direction is one or two call
sites. Breaking candidate per pair: extract an interface for the callback
direction, or pass a closure/value object instead of the service. Sized per
pair at **0.5–2 days** — real fixes, real coupling, but each needs its own
careful change, not a sweep. Priority: the two `⇄ PermissionChecker` and
`⇄ FilterQueryBuilder` structural pairs (core paths), then the job↔service
loops (chaining style choice: dispatch self vs event).

## Ledger line

- 1 reference to the existing knot plan (no new work here)
- 7 pair-break candidates, ranked above
