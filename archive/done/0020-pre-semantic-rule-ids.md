# 0020 — Rule ids for pre-semantic rejections

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `9db80a6` (ff), 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 3

## Outcome

Every frontend stage now publishes the rule its DIAG-1 attribution already
selected: rule-id mappings restated 1:1 from existing branches at Lexing,
TerminalClassification (reviewed scope addition — the remaining pre-semantic
stop), CanonicalSource, Parsing, and Resolution, funneled through one
constructor so no stage can publish a source rejection without its rule.
Plumbing only — no judgment, text, location, or verdict moved. Lane delta on
the branch: Pass 242 → 276; the moved set is exactly 0014's bucket-1 45
(set-diffed; zero new failures).

## Findings routed onward (visible only now; not fixed here)

Eleven attribution divergences, disjoint from buckets 2-4: three OP-family
cases citing OP-1 where OP-2/OP-7/OP-8 are expected; two FORM-3 cases citing
GRAM-2; one EFF-1 case rejected at OWN-3; and five GRAM-10 cases — including
TWO POSITIVE PROGRAMS (`own13-pos-uniq-match-payloads`,
`own1-pos-match-copy-payload-reuse`) rejected by a binder-vs-paired-field
spelling check whose GRAM-10 grounding is unconfirmed: a possible
compiler over-rejection of valid programs, flagged for priority
investigation alongside bucket 4.

## Evidence and validation

- Landed commit: `9db80a6`. Both gates green by unpiped exit codes; the
  adapter's stale `#[ignore]` reason is deliberately left for reconciliation
  after 0019/0021 land (three concurrent tasks move the same tally).
