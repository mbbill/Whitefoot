# 0036 — FLOOR-5 grammar path and corpus migration

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** owner approval 2026-08-07 (`governance/APPROVALS.md`); the
  candidate `governance/spec-evolution/spelling-relief-candidate.md`
- **Owner / workspace:** exec-0036 / `/Users/bytedance/do_not_scan/wf-0036`
  on branch `task/0036-floor5-grammar-and-migration`
- **Base revision:** b345e2c
- **Dependency:** none

## Goal

0031-style atomic activation prep for the FLOOR-5 spelling batch, on one
task branch: (1) assemble `kernel-spec-v0.23-candidate.md` from active
v0.22 plus the FLOOR-5 delta; (2) extend the grammar path — two keywords,
twenty operator spellings, the left-factored `expr`/`infix_tail`, the
if/value_if productions, deletions for targs and let annotations, FORM-2
layout; expected verifier total 69 productions (reviewer carry-forward:
production `infix_tail` maps to node kind `infix`, a name mismatch the
generator may not expect — make it a success criterion, not a surprise);
(3) repoint identity pins; (4) migrate the corpus: 1353 targ deletions,
1748 let annotations, 257 Bool matches to if/else with mandatory else-if
flattening, ~384 infix respells — scripted in scratch, every file passing
the branch compiler's parse + FORM-2 canonical audit, plus a REVIEW PACKET
(diffstat, ten representative before/after excerpts, verdict-meaning
statement); (5) evidence: verifier green on branch, `make -C compiler
check` and repo `make check` exit 0 (direct exit codes), adapter
comparison; (6) STOP before merge with the candidate SHA-256.

Discoveries outside the candidate stop the task with evidence.
