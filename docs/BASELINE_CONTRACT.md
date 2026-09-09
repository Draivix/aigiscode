# Baseline and convergence contract

Convergence compares verified analysis captures. Absence of a baseline is not an
empty baseline, and disappearance from an incomplete analysis does not prove that
a finding was resolved.

`convergence-history.json.baseline` distinguishes two separate questions:

| Field | Values | Meaning |
| --- | --- | --- |
| `availability` | `missing`, `partial`, `unverified`, `inconsistent`, `verified`, `unknown` | Whether the required baseline family is present and matches its manifest. |
| `comparison` | `initial_snapshot`, `comparable`, `not_compared` | Whether temporal changes may be inferred from these inputs. |
| `reasons` | Typed reason codes | Missing seal/member, hash mismatch, publication during reading, different root/engine/scope/tool selection, or incomplete current/previous checks. |

The assessment records current and previous snapshot identities. An identity
contains the canonical analysis root, source fingerprint, effective scan-scope
fingerprint, captured resolver configuration fingerprint, engine version/build
fingerprint, semantic revision and selected external tools. Source and resolver
configuration may change in a legitimate comparison. The analysis root, scope,
engine and external tool selection must match, and both native input coverage and
requested external checks must be complete.

Engine identity includes the crate sources, package/workspace manifests, resolved
lockfile, build configuration and Rust compiler version. A stdlib-only Rust build
script embeds this fingerprint, so changing tool implementation does not silently
reuse the identity of another `0.1.0` build. This uses Cargo's supported
[build-script metadata mechanism](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-env).
The fingerprint describes the captured build inputs; it is not a binary signature.

## Manifest seal

The artifact writer publishes `scan-manifest.json` last. Its `snapshot_identity`
and `baseline_hashes` bind the architecture surface, review surface and contract
inventory to one capture. The reader hashes the same streams it deserializes and
checks the manifest again after reading the three members. Missing members or a
changed/mismatched seal make the baseline unavailable for comparison. Malformed
JSON and reachable read failures remain explicit errors rather than empty data.

These non-cryptographic hashes detect local inconsistency; they do not authenticate
an adversarial author. The seal also does not make publication of the entire
artifact family atomic. Readers of other artifact combinations still need the
family-publication work tracked separately.

The semantic revision is 11. Fast-load requires the current engine fingerprint,
canonical root and scope as well as the source/configuration/graph hashes. An old
unsealed manifest can be read as historical input but cannot establish a verified
baseline.

## Finding states and unavailable deltas

- `FirstObserved` means a finding was observed without a supplied baseline. It is
  not a newly introduced defect.
- `NotCompared` means temporal classification is unavailable. It cannot be treated
  as `Unchanged`, `Improved` or `Resolved`.
- `New`, `Worsened`, `Improved`, `Unchanged` and `Resolved` require a comparable
  baseline. They describe observed analysis findings; configuration or review-policy
  changes can affect findings without proving a source-code fix.
- `graph_delta` and `contract_delta` are `null` when comparison is unavailable.
  Null does not mean zero change. `summary.previous_findings` is null when there is
  no verified same-root prior finding inventory.
- Convergence summary counts refer to unique logical fingerprints. Raw review
  findings retain individual occurrences. A finding delta exposes
  `current_occurrences` and `previous_occurrences`; its representative is selected
  deterministically by visibility, severity, confidence and ID, not input order.

Topology labels, MCP contracts and agent packets preserve these states. Initial
observations still supply review attention and graph context without inventing
regressions. Agent commands use the context produced by the writer directly; they
do not reread a newly written current snapshot as its own predecessor. Read-only
agent/MCP paths use the same baseline reader.

## Guard and automation

The guard includes the baseline assessment and
`pressure.comparison_available`. Comparison pressure fields are only meaningful
when this is true. Missing comparison produces review obligations rather than a
claim about the current diff. Visible high-severity security evidence can block
on its own, independently of when it was introduced. Incomplete native or requested
external evidence also blocks, with a missing-evidence reason.

An analytical CLI exit code of 0 means that the requested analysis completed; it
does not mean guard `Allow`. Exit code 1 covers analysis failures or incomplete
requested evidence, after writing partial artifacts when possible. A consumer
that requires automatic approval must additionally require `Allow` in
`guard-decision.json` and decide how to handle `Warn` through its review policy.
This contract adds no implicit approval or publication action.

The baseline is the previous observed analysis in the selected output directory,
not automatically an approved release. Rerunning an audit does not establish that
existing debt was approved. See the dated reliability evidence for the specific
runtime behavior verified during implementation; automated tests and CI remain
stopped at the user's request.
