# 0066 — Stage 9a deterministic claim ledger

- **Status:** `IN PROGRESS` (claimed 2026-08-15)
- **Owner / workspace:** Codex lead / `/Users/bytedance/code/Whitefoot`, `main`
- **Base revision:**
  `1d0633e85154876ca0244cdb631b3f3a9aade029`
- **Authority:** the ACTIVE Current Plan, Workstream 9a `deterministic claim
  ledger`, under Direction Outline revision 39 item `PROOF-8`

## Goal

Expose one deterministic, read-only checked-program ledger for every named
claim: stable in-program source identity, name and predicate, justification,
lifecycle disposition, exact obligations whose retained canonical derivations
use that claim, and existing provenance disposition. Change no acceptance,
diagnostic, lowering, runtime, specification, or protected-compliance behavior.

## Method and boundary

1. At the existing unique claim close, retain the already-selected positive,
   negative, or contradiction derivation for redundant/refuted lifecycle
   outcomes; retained claims have no lifecycle proof. Root and remap it through
   the existing function-local DAG without another closure or semantic walk.
2. After provenance succeeds and every function DAG is finalized, build one
   checked-program-only observational ledger by traversing existing canonical
   root parents. Link a claim only when its exact S3 event is actually an
   ancestor of that retained proof; never guess support or copy a proof graph.
3. Join the existing protected-leaf, direct-demand, requirement-bridge, and
   call provenance identities exactly. A missing required mapping is an
   internal failure, not a best-effort omission.
4. Order functions, claims, obligations, links, and provenance deterministically
   using existing checked identities and source order. Add no serialized
   artifact, hash protocol, CLI, replay verifier, portable identity, optimizer
   input, or Stage 9b semantics.

Primary code owners are `compiler/src/semantic/{entailment.rs,check.rs,model.rs}`,
`semantic/entailment/{flow.rs,state.rs}`, and focused semantic tests. Ordinary
README, acceptance, roadmap, plan, MCTS, and this record change only at terminal
closure. No specification, conformance corpus, runner, adapter, lowering,
backend, runtime, ABI, or gate-wiring file is in scope.

## Validation and done-when

Synthetic retained, redundant, refuted, contradiction, kill, join, call, loop,
generic, and repeated-build controls must prove exact canonical links and dense
remap. UTF-8, four-source raw-DEFLATE, and wfgrep must enumerate the complete
installed claim population from the ledger itself with deterministic counts and
ordering. Existing CLM diagnostics, acceptance, LLVM, runtime behavior, proof
metrics, full compiler/repository gates, and the adapter boundary must remain
unchanged.

Stop if completeness needs per-claim reanalysis, a second closure, guessed
support, copied derivation authority, durable identity machinery, or a language
or protected-compliance change. Done means the checked-program ledger and
bounded cost evidence are green and Stage 9b's exact candidate can be formed
from measured claims without reopening Stage 9a.
