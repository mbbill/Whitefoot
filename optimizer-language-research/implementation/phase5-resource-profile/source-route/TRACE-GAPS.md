# Source-route trace-gap ledger

Status: NON-AUTHORITATIVE EVIDENCE PREREQUISITE. This ledger names work that
must exist before this route can emit a complete 33-field measurement receipt.
The current route never substitutes zero, a formula, an observed production
counter, or a conservative bound for a missing algorithm trace.

| field | independently replayed schedule still required | inputs this replay may consume |
|---|---|---|
| 9 `max_lexical_scan_work` | two immutable scanner passes; every byte/one-past-end probe, fixed/suffix/escape/class comparison, UTF-8 byte validation, raw commit, and emission conversion | exact source bytes and this route's independent token spans |
| 14 `max_parser_stack_entries` | prospective `tasks + frames + 1` at every push in a separately derived generated-grammar replay | independent tokens/tree and exact successor grammar bytes |
| 15 `max_list_members` | every successful `RepeatZero` and `RepeatOne` member selection before scheduling | the same independent generated-grammar replay |
| 16 `max_expected_terminals` | DIAG-1 probe descent, row revisits, bounded overrides, and distinct expected-set construction | independent tokens, exact successor grammar bytes, and the selected syntax failure |
| 17 `max_syntax_work` | one nonresetting meter across terminal classification, generated parsing, finalization, and canonical source audit | independent source bytes, tokens, tree, and generated grammar facts derived inside this route |
| 33 `max_resolution_work` | FN-8 scan, mixed preflight, construction, four stable sorts, key comparisons, inventory ranks, indexed queries, exactly two origin scans, and diagnostic materialization | this route's mixed traversal, scopes, roles, spelling intervals, and selected semantic diagnostic |

The new replay may not import production compiler crates, the proposal's
`diagnostic_evidence` models, another measurement route, expected counts or
work, an extracted grammar from another route, or anything under `archive/`.
It must derive and identity-bind its own grammar tables from the exact
successor candidate. Its output must differential-match the future second
independent route before numerical profile approval.
