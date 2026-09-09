# Unicode-safe scanner snippets

The secondary scanner truncated normalized matches at byte 157 without checking
the UTF-8 boundary. A multibyte character spanning that offset could panic while
constructing a finding. Truncation now backs up to a character boundary before
adding the ellipsis. The 160-byte output budget and ASCII behavior are preserved.

A scanner regression covers two-, three- and four-byte characters crossing that
offset through actual `eval` pattern extraction. It was added but not run under
David's test/CI stop instruction. The boundary case was identified by source
inspection, not dynamically reproduced in a local test.

The production release build passed without warnings. A real whole-AigisCode
audit completed 113 supported sources in 2.22 seconds, using 151,492 KiB peak RSS.
It returned exit 1 with explicit existing native and secondary coverage gaps;
this audit does not independently exercise the Unicode boundary case.

Evidence is retained in `target/reliability-2026-09-09/self-unicode-snippet/`, its
adjacent summary/timing logs, and `build-unicode-snippet.txt`. Engine fingerprint:
`a615c189230043e2`. Binary SHA-256:
`358de0da84fdf4925bf9e4866b70cd1fed837c892c7ee9708e2f6c71a41741de`.
Draivix was not modified. Broader Q01–Q12 acceptance remains open.
