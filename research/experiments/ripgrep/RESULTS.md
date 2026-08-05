# RG-BASE Results

Status: ATTEMPT 1 INCONCLUSIVE — no comparator selected; baseline did not run

The frozen comparator-selection run completed every scheduled command and every
correctness oracle passed. It did not satisfy the preregistered 3% precision
gate for any of the nine cases, so the protocol selected no upstream binary and
made no performance claim.

## Attempt 1

- Run: `rg-base-selection-1`
- Frozen commit: `21857a8f5156c853e9a18c89603024c09b0e942f`
- Frozen manifest SHA-256:
  `9f68b1e811f92d634ae22528ac761e9ea335329e0c575d13efec6638e7d0b854`
- Started: `2026-08-05T03:37:58Z`
- Finished: `2026-08-05T04:17:52Z` (39 minutes 54 seconds)
- Raw evidence: [raw/rg-base-selection-1.jsonl](raw/rg-base-selection-1.jsonl),
  614 lines, 230,445 bytes, SHA-256
  `52a45aa40f178916a6255c0c3e108d6b4dfe83e83dd91168140106d7fa3e240b`

`official/native` is the ratio of the two medians. The CI is the paired
bootstrap 95% interval for that ratio. Every relative half-width exceeds the
frozen 3% limit.

| Case | Official median | Native median | official/native | 95% CI | Relative half-width |
|---|---:|---:|---:|---:|---:|
| `linux_literal` | 10.990 s | 11.324 s | 0.971 | 0.917–1.051 | 6.89% |
| `linux_required_regex` | 7.469 s | 7.803 s | 0.957 | 0.895–1.049 | 8.04% |
| `linux_unicode_class` | 10.418 s | 10.983 s | 0.949 | 0.861–1.052 | 10.08% |
| `llama_literal` | 0.110 s | 0.121 s | 0.909 | 0.849–0.997 | 8.15% |
| `llama_case_insensitive` | 0.263 s | 0.283 s | 0.930 | 0.865–1.023 | 8.49% |
| `llama_literal_set` | 0.102 s | 0.107 s | 0.954 | 0.904–0.971 | 3.49% |
| `subtitles_unicode_literal` | 0.531 s | 0.481 s | 1.104 | 0.555–1.872 | 59.67% |
| `subtitles_unicode_case_insensitive` | 0.517 s | 0.514 s | 1.005 | 0.963–1.067 | 5.15% |
| `subtitles_no_literal` | 3.416 s | 3.368 s | 1.014 | 0.942–1.061 | 5.87% |

Summing the per-case medians gives 33.816 seconds for the official binary and
34.985 seconds for the native binary. These sums explain the cost of one suite
pass; they are not an aggregate comparison because comparator selection failed.

## What the attempt established

The schedule alternated which binary ran first, but that did not make the warm
cache condition symmetric. For `subtitles_unicode_literal`, the first process
had a 0.654–0.669 second median and the second a 0.285–0.288 second median,
regardless of binary identity. That position effect explains the exceptionally
wide interval and makes the lower overall median untrustworthy as a binary
selection rule on this host.

The three Linux cases cost approximately 131–139 microseconds per searched
file despite materially different patterns. The single-file subtitle literal
case scans more bytes in about half a second. This is evidence that the frozen
many-file cases contain substantial traversal/open/read cost; it is not yet an
attribution to one OS, filesystem, runtime, or ripgrep component.

The owner chose not to repeat this 40-minute protocol during development. Fast
project fixtures are the edit-loop gate; a relevant frozen case may be run once
at a milestone, and the full paired suite is reserved for a later public-claim
candidate. Any future comparator selection needs a new owner-reviewed protocol
that names this failed attempt and fixes the cache-position problem before it
runs.

No comparator was selected. The selected-upstream baseline and profiles did
not run. There is no Whitefoot-versus-ripgrep result and no 2x claim.
