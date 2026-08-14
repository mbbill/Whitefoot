# 0058 — Stage 8b verified-postcondition semantic freeze

- **Status:** `DONE` (2026-08-14)
- **Authority:** the `ACTIVE` Current Plan selected 2026-08-14, Workstream 8b
  `verified normal-return postconditions`, under Direction Outline item
  `PROOF-8`
- **Initial claim base:**
  `fa689611162452614eac79cfcf35fed85a9a16eb`
- **Resumed execution base and candidate parent:**
  `55a7b5f386d5a2a7a0b9c0e3a950f374112e7f22`
- **Held reviewed candidate commit:**
  `7a293861ddb0aad0cd8d494ee8f4bfa5f6cbba29`

## Outcome

Task 0058 froze the complete v0.28 semantic candidate for verified
normal-return postconditions, the narrow selected-`Ok` payload receiver chosen
by the owner, direct call-result publication, and `value_if`-only bounded
delivery. It resolved the surface, complete/U/B proof views, entry identities,
concrete-SCC scheduling without a summary fixed point, PRV failure atomicity,
support and kill ordering, deterministic diagnostics, and the single DIAG-2
derivation authority. Named or pending outcomes, general `set` equality,
`value_match` delivery, expression terms, a third fact source, and runtime or
lowering fallbacks remain excluded.

The frozen 14/20 real-caller replay is complete on paper. The fourteen
`read_bits` rows use the selected-payload receiver and an explicit `mask`
contract; the twenty `append_slice` rows use direct same-binding result
publication on two distinct declarations. The wfgrep A10 repair keeps the
prior length across the child region, selects `candidate_length` or the prior
length through `value_if`, and uses only `bounded_length` thereafter. Both
append bodies retain the invalid-domain `return filled` behavior with no
destination write.

The candidate is a held, non-authoritative branch artifact. Its internal
`ACTIVE v0.28` installation wording describes the exact eventual activated
specification; it does not change the repository's active v0.27 language
authority. No candidate specification, archive, compiler, real-source,
protected-conformance, approval-ledger, gate-wiring, or MCTS byte landed on the
integration branch through this task.

## Candidate identity and validation

- `spec/kernel-spec.md`: SHA-256
  `08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`,
  1,291 lines and 414,101 bytes.
- Candidate outgoing `spec/kernel-spec-v0.27.md`: SHA-256
  `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`,
  byte-identical to the still-active v0.27 specification.
- `spec/derivation/derivation-ledger.md`: SHA-256
  `5207e1d46cfa4d03dcedb4fcd80a8fa07c30510417a2da2d6573a13dad10fc4b`,
  with an amendment bound to the exact candidate specification.
- The candidate contains 132 rules: 83 derived and 49 existence-only. Its
  grammar inventory is 73 productions, 90 decisions, 89 fixed terminals, and
  97 terminal predicates.
- The specification diff has 43 mechanically reproducible zero-context hunks:
  two header/status hunks, one complete FN-9 insertion, and 40 anchored edits
  across twenty existing rules.
- The active-v0.27 native verifier accepts archive-to-archive at `70/85/96`
  and rejects archive-to-candidate only because the compiler still embeds the
  v0.27 frontend contract. Task 0059 owns the corresponding `73/90/97`
  frontend installation and must make that same verifier green before an
  owner packet exists.
- Three independent frozen-byte reviews reported P0=0, P1=0, and P2=0.
  Archive identity, hashes, rule arithmetic, grammar arithmetic, diagnostics,
  complete/U/B publication, all result routes, `value_if` discrimination,
  the 34 caller rows, and temporary-file hygiene were independently checked.

## Registered handoffs and remaining dependency

Planned tasks 0059–0064 form one strict, isolated held stack: frontend and
FN-9 surface; callee proof and summaries; call publication and receivers;
`value_if`-only delivery; real-consumer migration; then additive protected
conformance and the combined exact owner packet. Each task may begin only from
the named lead-reviewed held commit produced by its predecessor. None of those
candidate, compiler, source, or protected bytes may enter the active-v0.27
integration branch independently.

Task 0064 must stop in `WAITING` after freezing the exact combined
specification and protected-conformance packet. Only explicit owner approval
of those unchanged bytes permits a later activation task to be allocated.
That task will materialize the exact final H6 tree as one coherent main change,
without making H0–H6 intermediate commits main ancestors, and will install the
archive and digest chain, approval entry, documentation, MCTS update, and first
genuine full green activation gates.
