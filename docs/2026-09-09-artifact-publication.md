# Complete artifact generations on Draivix

Two full audits of the unchanged Draivix snapshot now publish separate, complete
20-member generations. After the second commit, native `info` selects the new
generation while an explicit old-generation path still reads the first. The old
seal is unchanged, all old members pass native verification, and every current
flat compatibility copy matches the new generation by SHA-256.

The [receipt](2026-09-09-artifact-publication-evidence.json) records both generation
IDs, member hashes, returned paths, timings, baseline state and actual MCP metadata.
No Draivix source or configuration was modified. The corpus is the broad source
capture documented in the [Draivix review](2026-09-09-draivix-revalidation.md).

| Observation | Result |
| --- | --- |
| First publication, migrating legacy artifacts | 78.80 s; 3,490,740 KiB peak RSS |
| Second publication to the same output root | 81.62 s; 3,509,088 KiB peak RSS |
| Native analytical members per generation | 20 |
| First generation retained after the second commit | Yes, including successful native `info` verification |
| Current compatibility copies equal committed members | All 20 |
| First useful cached MCP overview | 42.46 s |

The semantic graph fingerprint and both run summaries are unchanged. Both audits
return 1 because native and secondary coverage remains incomplete. All 3,218
logical findings stay `NotCompared`, with null graph/contract deltas and no claimed
improvement or resolution. Publication completeness is independent of audit coverage.

## Reader and publisher behavior

The writer assembles a private generation, seals its complete inventory, refreshes
independent compatibility copies, then replaces one commit marker. Publishers use
an OS lock; old committed generations remain available to readers. Native baseline,
fast-load and `info` readers pin one directory. Artifact path construction is shared
between CLI, MCP and the writer rather than copied in each layer.

MCP also identifies the stored generation and whether its captured inputs match the
served index. The real Draivix session returned `inputs_match_index=true`. A separate
real self-repository session served a fresh index over newer code while retaining an
older disk generation, correctly returning `false`. No synthetic source edits were
made to obtain that negative case; the code changed during this implementation.
The flag covers captured source, inventory and interpreted configuration identity;
it does not claim equality of baseline-dependent reports or Git diff context.

The [publication contract](ARTIFACT_PUBLICATION_CONTRACT.md) documents the marker,
retention, integrity errors, legacy behavior and new generation paths. The documented
flat paths still exist, but multi-file consumers must pin a generation. Project
configuration remains under the project's `.aigiscode`, independent of output root.

## Tradeoffs and verification limits

Retaining generations and separate compatibility copies consumes storage. The two
measured generations plus the current flat member copies occupy 4,986,318,227 logical
bytes, excluding small control files. This is logical size, not measured physical
allocation. No automatic pruning is added because external readers may retain paths.
These timings are individual observations on a shared host, not a speedup claim.

The production release build passed without warnings. Regression cases cover old
pins, compatibility-copy failure, invalid commit paths, corrupted members and native
immutability, but they were not run under David's test/CI stop instruction. Existing
legacy/cache regressions were adapted to the new path contract. Actual observations
cover normal migration, two successful publications, retained old data, compatibility
copies, `info`, and MCP. Process-kill, simultaneous-publisher, Windows and power-loss
scenarios were not dynamically replayed here.

This implements the generation mechanism required by Q11. Approved adversarial CI,
incremental processing, concurrent-load measurements, remaining Q12 work and overall
Q01–Q12 acceptance remain open.
