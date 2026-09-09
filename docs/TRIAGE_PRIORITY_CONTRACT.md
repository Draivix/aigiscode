# Evidence behind triage priority

Repository topology ranks review signals, not proven defects. Its recommended
start, focus clusters and structured triage steps carry `priority_basis`:

- `evidence_id`: the finding or guardian packet supporting the selection;
- `priority`: that same signal's review priority;
- `precision`: that same signal's detector precision label.

Priority sorts first, then precision: `certain`/`exact`, `strong`/`modeled`,
`heuristic`, and unknown labels. Stable IDs and paths break ties. A high-priority
heuristic cannot borrow the precision of another low-priority finding on the same
file. The label and instruction come from the selected signal as well. Precision
labels are detector classifications, not calibrated probabilities or guarantees
that a code change is appropriate.

All linked findings and packets participate before presentation limits are
applied. A cluster's primary target must belong to its zone; findings involving
other zones remain linked context. Up to five clusters and ten previews of each
kind are retained per zone, including the evidence behind the chosen clusters.
Structured steps and their text projection follow the same selection. Zone-wide
relation counts and finding totals remain context and do not increase a file's
priority.

Naming-based abstraction-role groups remain visible with their existing IDs and
fingerprints. They have low review priority and explicitly require validation of
actual responsibilities and callers. Names alone do not establish redundant
behavior, and their packets cannot unconditionally instruct removal or consolidation.
The detector's existing numeric score is retained; it does not establish a need
to change the code.

`artifacts/triage.rs` owns this ordering and its projections. The parent artifact
module continues to own file emission. Existing artifact paths remain unchanged;
older topology objects without priority evidence deserialize with no basis rather
than an invented one.

The ranking cannot decide whether a supplied doctrine rule is appropriate. A
source reference may certainly violate a declared rule while implementing an
intended framework registration. Review must distinguish code defects from a
doctrine or classification mismatch before changing either.
