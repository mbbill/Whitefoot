# Operational workflows

This is the sole operational guide for advancing Whitefoot. It holds two loops.

The **experiment loop** is the trunk: P0 is why the project exists, so work
starts from a falsifiable question about machine code. The **language-change
loop** is a branch entered only from the experiment loop's routing stage, when a
measured result names a specification gap as its concrete blocker. That ordering
is the 2026-07-24 roadmap reorientation made self-enforcing: an amendment cannot
start without the experiment that needs it.

A specification update is one coordinated state transition across governance,
the numbered specification, compiler-independent evidence, the compiler, and
live documentation. None of those parts has an independent update lifecycle.

The directories named below contain records, resources, and tools used by this
workflow. The shared rules and resource contracts live here at their common
parent; participating directories do not carry their own workflow README.

## Authority

- `docs/roadmap.md` alone defines current status, authorization, and next work.
- The active numbered file selected there is the sole language authority.
- `docs/constitution.md` is project law and `docs/patterns.md` defines writer
  forms.
- `mcts_mem/` records durable design choices, rejected alternatives, and the
  evidence behind them. Consult or modify it only through the installed
  `mcts-mem-use` skill; never treat its files as ordinary Markdown.
- `governance/APPROVALS.md` records explicit owner approval for protected
  changes.
- Compiler behavior, tests, candidates, and archived records never define
  language behavior.

## Parts of the loop

| Path | Role in a language change |
|---|---|
| `WORKFLOW.md` | Defines this complete cross-directory process and every participating resource contract |
| `docs/roadmap.md` | Opens the work, names the active specification, and records the result and next work |
| `governance/spec-evolution/` | Holds the one exact successor candidate reviewed by the owner |
| `governance/APPROVALS.md` | Records exact-byte and protected-evidence approval |
| `spec/` | Holds immutable released specifications and supporting derivation evidence |
| `tests/conformance/` | Holds compiler-independent source-to-verdict evidence for the active specification |
| `compiler/` | Implements the active specification through the normal compiler path |
| `docs/patterns.md` and other live docs | Teach and describe the active language |
| `mcts_mem/` | Preserves why a durable design won over real alternatives, and why a rejected one lost |
| `research/experiments/` | Holds one directory per experiment: sources, run script, and RESULTS.md |
| Root `Makefile` and `governance/hooks/` | Run the repository gate and protect released specification bytes |

`governance/` itself contains only the approval record, exact successor
candidates, and the tracked hook. Historical transition logs and superseded
review material live under `archive/governance/` and cannot authorize current
work.

## The experiment loop

One hypothesis per loop. No bundling: the v0.18 candidate moved thirty-one rules
at once and became unreviewable.

| # | Stage | Produces | Fails when |
|---|---|---|---|
| 0 | Select | the chosen channel, taken from the roadmap's ranked list | the choice is interest-led rather than ranked |
| 1 | Pre-register | the hypothesis, written before any intervention | it is edited after the intervention starts — that is a new loop |
| 2 | Instrument | baseline codegen and remarks; a canary shown *failing*; a workload where the transform applies | the instrument cannot observe the predicted effect yet |
| 3 | Probe | cheapest decisive check, e.g. a hand-patched `.ll` plus a codegen diff | treated as evidence for the claim instead of for the decision to build |
| 4 | Route | compiler-only work, a Phase 8 prerequisite, or entry to the language-change loop | a specification change is reached for before a blocker is named |
| 5 | Implement | the change on the one normal path, facts-off identity intact | a second lowering path or a writer-facing toggle appears |
| 6 | Verify | correctness, attribution, and magnitude, separately | a magnitude is claimed without attribution |
| 7 | Judge | keeper, parity, negative, or inconclusive | the result is read against hope rather than the pre-registration |
| 8 | Record | `mcts_mem/`, RESULTS.md, roadmap, and a standing canary, in one change | a parity or negative outcome goes unrecorded |
| 9 | Re-rank | the updated channel ranking that selects the next loop | the loop ends without choosing what follows |

**Stage 1 — the hypothesis** states the verified fact; the mechanism, meaning
which pass consumes it; the predicted observable, meaning which missed-transform
remark flips or which assembly shape appears; a magnitude band or an explicit
"directional only"; the R0 clause, naming the delta over Rust or C and whether
it lands on P0, W1, or W3; and the kill condition that retires it.

**Stage 6 — the three parts are not interchangeable.** *Correctness*: the gate
is green, required checks are retained, trap records are byte-identical.
*Attribution*: the facts-on/facts-off codegen diff is non-empty, the predicted
remarks flipped, the canary now passes. *Magnitude*: the number, with protocol,
machine, and caveats. The instrument is guilty until proven innocent, and no
magnitude may be cited without attribution.

**Stage 7 — the four outcomes.** A *keeper* needs attribution proven, a delta
outside the noise band on a named kernel, no acceptance change, no required
check removed without proof, and complexity proportionate to the win. *Parity*
means the fact was consumed and paid nothing; it is a real result, and it forces
one explicit sub-decision — keep the emission because it is free and correct, or
revert it because it carries complexity. *Negative* means consumed and worse:
revert and record. *Inconclusive* means the instrument broke; it is not a result
and is never filed as parity.

**Stage 8 — negative results are recorded as durably as wins.** That is what
`mcts_mem/`'s rejected alternatives exist for. An unrecorded dead channel gets
retried weeks later; sunk cost is not evidence, and neither is forgetting.

## First classify the problem

Reached from stage 4, or directly for a reported defect. Start from the active
numbered specification, not from compiler behavior or a test expectation.

- If the active specification already determines the behavior and the compiler
  disagrees, this is a compiler defect. Keep the specification and existing
  expectations unchanged, add the smallest useful regression, and fix the
  normal compiler path.
- If an existing conformance verdict contradicts the active specification,
  this is protected-evidence correction. Stop and obtain owner approval before
  changing or removing it. Do not change the language to preserve a bad test.
- If the specification is ambiguous, incomplete, or intentionally needs new
  behavior, enter the complete language-change loop below. Its stage 1 requires
  the naming of current roadmap work, which for a performance-driven amendment
  is the experiment-loop hypothesis and probe that identified the blocker.
- If the compiler cannot yet implement valid specified behavior, report the
  capability as unsupported. An internal error, timeout, crash, or missing
  feature is never a source-language rejection and never changes an expected
  verdict.

An additive conformance or compiler regression for behavior already fixed by
the active specification does not require a specification version. It must
cite the existing rule and must not alter a protected expectation.

## The complete language-change loop

### 1. Open and bound the change

Confirm that the change unlocks current roadmap work. Use the `mcts-mem-use`
skill to consult the relevant live MCTS node and rejected alternatives. State
the smallest coherent semantic change and the behavior that remains unchanged.

Before drafting, inventory every potentially affected surface and mark it
`change`, `unchanged` with a reason, or `not applicable`:

- grammar, lexical classes, canonical bytes, and syntax-node structure;
- name resolution, typing, ownership, effects, constants, and diagnostics;
- runtime values, traps, ABI behavior, and required safety checks;
- conformance sources, expected verdicts, and runnable/pending/xfail status;
- compiler identities, generated syntax data, frontend, semantics, lowering,
  backend, and runtime;
- writer patterns, examples, derivation evidence, and live documentation; and
- existing protected cases, verdicts, or approval boundaries.

This impact inventory is part of the owner-review packet. It is not a new
repository document or a second proposal artifact.

### 2. Draft one successor candidate

A specification proposal and its candidate are one file:

```text
governance/spec-evolution/kernel-spec-vN-candidate.md
```

Copy the active specification and apply the smallest complete change. Update
the version, status, prior-version description, normative rules, examples, and
internal references together. Never edit, rename, or delete a released
`spec/kernel-spec-v*.md` file.

The candidate is non-authoritative and mutable during review. Every byte change
invalidates its earlier hash and review. Do not create a separate
`PROPOSAL.md`, patch document, generated duplicate, or per-version workflow.
Normal compilation and tests remain bound to the installed active
specification; only an explicitly invoked proposal-checking tool may read the
candidate before activation.

### 3. Prepare evidence before approval

Derive the expected behavior change from the candidate before implementing it.
Review every row of the impact inventory, including negative and near-miss
cases that prove required checks remain. Identify every existing conformance
expectation that would need protected modification; do not silently apply
those changes before approval.

For a grammar or syntax change, run the compiler-sharing verifier:

```sh
cargo run --manifest-path compiler/Cargo.toml --bin whitefoot-grammar -- \
  governance/spec-evolution/kernel-spec-vN-candidate.md
```

The verifier uses the production frontend contract and fails closed on a
structural difference it cannot validate. Extend that shared native path when
the proposed grammar genuinely requires it; do not create an independent
parser or a script fork for the new version. Proposal tooling may inspect the
candidate, but it must not switch normal compilation to unapproved semantics.

Review the complete candidate for internal consistency, retained safety
checks, constitutional conflicts, MCTS conflicts, diagnostic determinism, and
the full derived-material impact. Implementation convenience is not evidence
for language behavior.

### 4. Obtain exact owner approval

Present one review packet containing:

- the candidate path, complete SHA-256, and concise semantic delta;
- the completed impact inventory;
- grammar-verifier and other independent evidence;
- every requested protected verdict or status change; and
- any remaining limitation or unsupported compiler capability.

Owner approval covers only the exact candidate bytes and explicitly listed
protected changes. A direction, plan, partial excerpt, or earlier hash is not
approval. If any candidate byte or protected-change boundary changes, return to
this step.

After explicit approval, append the exact artifact or change boundary, hash,
reason, and evidence pointer to `governance/APPROVALS.md`. Use the
`mcts-mem-use` skill to record a durable design choice only when a real
alternative existed. Never edit `mcts_mem/` without first loading and following
that skill; its structure, provenance, paired-move, and lint requirements are
part of the data format. Keep approval bookkeeping and implementation activity
out of the tree.

### 5. Activate the approved language as one repository change

Copy the approved candidate byte-for-byte to the new immutable
`spec/kernel-spec-vN.md`. In the same cohesive activation change:

1. switch `docs/roadmap.md` and every active specification identity to the new
   numbered file and digest;
2. update compiler syntax data, frontend, semantic rules, diagnostics,
   lowering, backend, and runtime wherever the impact inventory requires it;
3. update conformance sources, manifest expectations, statuses, coverage
   annotations, and active-spec identity;
4. update writer patterns, examples, derivation evidence, and live docs; and
5. append the approval record and use the `mcts-mem-use` skill for any MCTS
   update required by the approved change.

The specification may describe a capability the research compiler does not yet
support only when the roadmap says so explicitly. Such a gap remains an
unsupported compiler capability; conformance expectations still state the
language result and may not be rewritten as rejection. Moving an existing case
from runnable to pending or xfail is a protected weakening and must be part of
the owner's approval boundary.

Approved and installed candidates remain in `governance/spec-evolution/` as
compact exact-byte review evidence. A rejected or abandoned candidate that was
never installed moves to `archive/governance/spec-evolution/`; active source,
builds, tests, and tools must not depend on it.

### 6. Verify and close the loop

First prove that the installed specification is the approved object:

```sh
cmp governance/spec-evolution/kernel-spec-vN-candidate.md \
  spec/kernel-spec-vN.md
```

Then run the relevant component checks and the complete repository gate:

```sh
make -C compiler check
make conformance
make check
```

Run the exact grammar-candidate check as well when grammar or syntax changed.
Inspect the final diff against every impact-inventory row. A green gate proves
only the behavior it exercises; it does not excuse an omitted derived update.

Commit the activation as one cohesive state transition. Record completed
status and exact next work in `docs/roadmap.md`. Do not leave the repository in
a committed state where the active spec, compiler identity, or conformance
identity name different language versions.

## Resource contracts

The participating directories are deliberately passive. They hold inputs,
evidence, records, and tools consumed by the loop above; none decides when or
how the language changes.

### Governance resources

`governance/` contains `APPROVALS.md`, one exact candidate per successor under
`spec-evolution/`, and the tracked pre-commit hook. It contains no directives
log, README, second how-to, free-standing proposal, generated approval
database, or independent work queue.

`APPROVALS.md` is an append-only record, not procedure. A candidate is mutable
until exact-byte approval and immutable afterward. Installed candidates remain
as review evidence; rejected or abandoned never-installed candidates move to
the archive.

### Specification resources

`spec/` contains released `kernel-spec-v*.md` resources and supporting
derivation or reconciliation evidence. Released numbered files are immutable.
The directory contains no mutable current plan, README, per-version update
script, or tool that selects a different active version.

### Design-memory resources

`mcts_mem/` is maintained only through the installed `mcts-mem-use` skill. The
skill owns traversal, admission, formatting, provenance, paired moves, and
linting or its documented manual fallback. Do not read or edit the tree as
ordinary repository Markdown, and do not invent a local replacement procedure.

### Experiment resources

`research/experiments/` holds one self-contained directory per experiment:
sources, a run script, and a RESULTS.md carrying the pre-registered hypothesis,
protocol, machine, measured numbers, attribution evidence, and honest caveats. A
parity or negative RESULTS.md is as durable as a winning one. Democ-era bundles
remain historical evidence and regression targets; they are not active gates,
and a script inside one may name a retired compiler.

### Conformance resources

`tests/conformance/` supplies compiler-independent source-to-verdict evidence:

- `cases/<id>.wf` is one canonical Whitefoot program and FORM-1/2 byte fixture;
- `manifest.jsonl` maps each case to rule identifiers, its expected result, and
  its current execution status, and carries explicit coverage annotations for
  specification properties no source program can exercise;
- `runner.py` validates active-spec identity and corpus structure, reports
  declared rule coverage, and owns the explicit compiler-adapter slot; and
- `test_runner.py` tests that corpus plumbing and active-spec binding.

An expectation is `accept`, exact-rule `reject`, `run` with an exit value, or
`trap`. A compiler failure, timeout, crash, or missing capability is never a
rejection verdict. Status describes execution availability only:

- `runnable` means the case and expectation are complete and any adapter that
  claims the required capability must produce the expectation; it does not by
  itself claim that the current compiler has been run against the case;
- `pending` means the compiler cannot yet execute the case; and
- `xfail` preserves the correct expectation while exposing a known compiler
  mismatch; an unexpected match is `XPASS`.

Changing an existing expectation, removing a case, or weakening runnable
status is protected work handled by this workflow. An additive case for
behavior already fixed by the active specification may land with ordinary
compiler work when it cites the existing rule and changes no protected result.

The corpus contains no language-design decision, compiler special case,
README, or independent release process. Any compiler adapter must drive source
through the normal compiler command path; it does not become a second semantic
implementation or stable protocol.

Conformance tools run from the repository root:

```sh
make conformance
python3 -B tests/conformance/runner.py coverage
make conformance-run
```

`make conformance-run` fails explicitly while the adapter slot is empty.

### Tool boundary

New scripts are not the workflow. Prefer the existing native compiler and root
gates. A genuinely compiler-independent conformance tool must remain
version-neutral and have an explicit caller. The workflow-participating
resource directories do not carry local workflow or guidance READMEs; update
this file when their shared contract changes.

## Checks and hooks

Install the tracked append-only hook once per worktree:

```sh
make install-hooks
```

The hook invokes `make spec-append-only-staged`. The ordinary repository gate
invokes `make repository-invariants` and `make spec-append-only` alongside the
compiler-independent evidence and compiler checks. These mechanisms protect
specific invariants; responsibility for completing the whole loop remains with
the person or agent changing the language.
