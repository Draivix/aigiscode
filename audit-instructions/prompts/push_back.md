# PUSH-BACK (sent when the director rejects a batch result)

Your previous answer for this batch was rejected. Reason: `{{reason}}`.

Rejected items:
{{rejected_items}}

Common reasons and what to do:
- **missing_evidence** – a finding had no verbatim `quote` or the quote was not found in the input. Re-read the files. Re-report the finding ONLY with an exact copy of the lines. If you cannot find the lines, the finding does not exist – drop it.
- **too_thin** – you returned fewer than 1 finding per 3,000 lines of input. The director assumes this batch is average, i.e. ~5% of it is redundant. Go function by function. For every function, write (internally) one line: "what it does, who calls it, what else does the same". Report everything that fails that test. The files are provided again below in a different order.
- **vague** – findings like "could be simplified" were dropped. Replace each with concrete symbols, lines, and a claim that can be checked.
- **skimmed** – you listed files in `files_skimmed`. Those files are re-sent here in full and alone. Read every function.
- **verdict_without_finding** – a `group_verdicts` entry (or a subgroup of a `mixed` group) says duplicate_exact / duplicate_semantic / variant_mergeable, but `findings[]` has no finding naming that `cluster_id`. The rejected items list those clusters. Write the finding for every one of them (kind matching the verdict, all member symbols, verbatim evidence from each member, `behavior_differences` for variants) and keep every finding of your previous answer.
- **hallucinated_symbol** – you cited a symbol not in the input. Remove it or move it to `open_questions`.

This is attempt `{{attempt}}` of 3. Output the complete corrected result (not a delta), same schema.
