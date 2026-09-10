# Parsed implementation comparisons

The existing PHP, JavaScript/TypeScript, Vue, Rust, Python and Ruby parsers
capture bounded function-body facts from their already parsed trees. Facts live
in `semantic-graph.json.function_behaviors`: parameters, literal/property
selectors, branch and return expressions, calls, writes, error handling and
structural fingerprints. Expression locations refer to the captured source;
fingerprints distinguish source text while normalized shapes propose comparisons.
These facts do not establish runtime effects or semantic equivalence.

`deterministic-findings.json.architectural_assessment.behavior` contains the
comparison coverage and retained comparisons. The consolidated report,
architecture overview and MCP coverage expose the counters. Matching graph and
agent task packets carry the bounded comparison records and source anchors,
including compatible existing helpers and captured consumer files.

The comparison distinguishes repeated-decision candidates, cross-language
contract candidates, different declared contracts, unverified input contracts
and shared primitives. Only the first two enter the existing DuplicateMechanism
review family. Similarity or a matching name never proves a violation. Different
schema adapters remain review evidence without a DRY finding. A shared direct
delegation is evidence for an existing primitive, subject to receiver resolution.

Candidate search uses meaningful names, selectors and structural fingerprints,
with limits of 64 members per bucket, 50,000 candidate pairs and 2,048 retained
comparisons. Skipped buckets or cropped results set
`candidate_search_truncated`. Body capture excludes incomplete/truncated bodies
from comparison; its separate coverage distinguishes complete, partial and
unavailable capture. Captured production callers exclude recognized test paths
and Rust test guards; their absence does not prove application unreachability.

An agent must compare concrete inputs, defaults, errors and effects, choose and
justify a surviving owner, identify consumer migration, preserve required
behavior and state missing evidence using the
[architectural review contract](ARCHITECTURAL_REVIEW_CONTRACT.md). Native source
validation anchors the resulting proposal; it does not execute that proposal or
certify runtime behavior. Cross-language candidates require explicit wire-format
and boundary comparison before any consolidation proposal.

Regression cases cover actor-key disagreement, distinct declared input types,
different array schemas, shared normalization and bounded capture. Tests and CI
remain paused under David's instruction; these cases have not been executed.

## Observed Draivix cases

The configured source slice on 2026-09-10 yielded 50,830 function/method symbols,
48,969 captured bodies and 43,882 usable bodies. The bounded search considered
50,000 pairs and retained 2,048 comparisons; both capture and search remain
incomplete. The [native observation excerpt](2026-09-10-behavior-comparison.json)
preserves the counters and the actor comparison.

`HookAwareMapper::resolveActorId` and
`AssignmentNotificationService::resolveActorId` now form a comparison despite
only 163/1000 structural similarity. The record distinguishes the mapper-only
`actorId` selector and `Auth::id` call, branch/return differences, and three
versus two captured call sites. Related implementations include
`ActorContext::id` with zero captured callers, plus the deletion, realtime and
status-transition variants. The helper is an integration candidate, not a
deletion recommendation or an automatically chosen canonical owner.

A separate native observation of the cropped `app/Support` directory retained
`SystemRelationFields::isRelationDefinitionTo` versus `isRelationTo` as
`unverified_input_contract`, with no associated DRY finding. Their array schemas
use `type` and `relation` differently. That observation compared 2,244 pairs and
reported the truncated repository boundary. Both CLI observations returned exit
1 with coverage explanations; neither is a clean whole-application analysis.

Native builds completed. The final helper-search change reorders equivalent
filters to avoid repeated language lookups; no performance improvement is
claimed from these observations. Draivix source files were not changed.
