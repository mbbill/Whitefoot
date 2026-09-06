# Native grammar-path extension for the v0.18 candidate

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `a9c6e1a`, 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 1
  (verifier step), base `7cc6302`/`85c0f5c`

## Outcome

`whitefoot-grammar` now verifies the v0.18 candidate through a staged grammar
path: the compiler carries two committed table sets behind one shared
lexer/classifier/parser engine, selected by contract identity. The active
v0.17 set is byte-untouched (two never-referenced `Production` enum variants
excepted), the production driver always passes the active hash, and
`finalize` fails closed on staged derivations, so production behavior is
unchanged. The staged set describes the complete candidate grammar (64
productions, 67 fixed terminals adding `as`/`external`/`blocks`, 74
strong-LL(2) decisions); a committed byte-exact snapshot of the candidate's
three frontend-contract sections (`staged_frontend.md`, SHA-256 selection
token) pins the contract, and any candidate drift fails closed until
re-extension — a property demonstrated in the field when the hostile-review
amendments landed mid-task.

## Evidence and validation

- Landed commits: `4ea068a` (staged grammar path), `a9c6e1a` (snapshot
  re-extension to the amended candidate).
- Verifier on main: `staged candidate contract verified by the active
  compiler's staged grammar path: 64 productions, 74 decisions, 75 terminal
  predicates`; the active spec still verifies unchanged (62/72/72).
- `make -C compiler check` and `make check` green on main after landing; 360
  tests, 0 failed. Focused tests cover staged parsing of the kind-declaring
  entry and effect rows, reservation of the three new spellings, active-path
  spot checks (a `program_kind` entry and an `external` row still reject
  under v0.17; the three spellings remain active identifiers), staged
  fail-closed near-misses, and finalize fail-closed.
- Lead review covered the selection mechanism, the enum-only active-table
  delta, the finalize gate, and the re-extension delta.

## Follow-ups

- The offline table generator sits at `<scratch-root>/wf-grammar-gen`
  (validated by reproducing the committed v0.17 tables exactly); the
  activation task decides whether to wire it in or regenerate independently.
- If the candidate's frontend sections are amended again before activation,
  the verifier fails closed until `staged_frontend.md` and its hash are
  re-extended — intended behavior, owned by whoever amends.
- Activation (after exact approval) promotes the staged tables to active and
  retires the staged selection path in the same change.
