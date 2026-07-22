# Governance

This is the sole operational guide for repository governance. Other Markdown
files in this directory are records or exact review artifacts, not additional
procedure. Do not create a second governance how-to; update this file instead.

## Authority

- `docs/roadmap.md` alone defines current status, authorization, and next work.
- The active numbered file selected there is the language specification.
- `docs/constitution.md` is project law and `docs/patterns.md` defines writer
  forms.
- `mcts_mem/` records durable design choices, rejected alternatives, and the
  evidence behind them.
- `APPROVALS.md` records explicit owner approval for protected changes.
- Historical transition logs and superseded review material live under
  `archive/governance/` and cannot authorize current work.

## Contents

| Path | Purpose |
|---|---|
| `README.md` | This procedure and folder map |
| `APPROVALS.md` | Append-only protected approval record |
| `spec-evolution/` | One exact, versioned candidate file per reviewed successor specification |
| `hooks/pre-commit` | Invoke the Makefile's staged append-only check before a commit |

The root Makefile owns the two small repository checks. The
`repository-invariants` target compares the two agent-instruction files and
requires the canonical roadmap marker. The `spec-append-only` targets ask Git
directly whether a released numbered specification was modified, renamed, or
removed. Governance has no separate Python tooling.

## One candidate per successor

A specification proposal and its candidate are one artifact:

```text
governance/spec-evolution/kernel-spec-vN-candidate.md
```

The file contains the complete proposed successor specification bytes. It is
the exact object reviewed, hashed, approved, and later installed. There is no
separate `PROPOSAL.md`, generated copy, patch document, or second candidate
representation. The candidate's version header states the proposed delta;
durable rationale and real alternatives belong in `mcts_mem/`.

A candidate is non-authoritative until the owner approves its exact bytes. It
may change during review, but every change invalidates prior hashes and review
results. After exact approval it is immutable. Installation copies those bytes
unchanged to `spec/kernel-spec-vN.md`.

Approved and installed candidates remain in `spec-evolution/` as compact,
version-addressed audit evidence. An active tool may bind the current candidate,
as `whitefoot-spec` does. A rejected or abandoned candidate that was never
installed moves to `archive/governance/spec-evolution/`; active source, builds,
tests, and tools must not depend on the archived copy.

## Changing the language

1. Confirm that the change unlocks current roadmap work and consult the relevant
   live MCTS node and rejected alternatives.
2. Create the single complete versioned candidate by copying the active numbered
   specification and applying the smallest coherent change. Never edit, rename,
   or delete a released `spec/kernel-spec-v*.md` file.
3. Update the candidate's version, status, prior-version description, rule text,
   and internal references as one exact document.
4. Before presenting a grammar change, run the compiler-sharing verifier:

   ```sh
   cd compiler
   cargo run --bin whitefoot-grammar -- \
     ../governance/spec-evolution/kernel-spec-vN-candidate.md
   ```

   The verifier currently accepts an unchanged frontend contract and fails
   closed on structural changes. Extend that compiler-sharing path first when a
   real grammar change requires it; do not revive an independent parser.
5. Review the exact candidate for semantic consistency, safety-check retention,
   derived-material impact, and conflicts with the constitution and current
   design memory.
6. Present the exact file and SHA-256 to the owner. A plan or general direction
   does not approve candidate bytes.
7. After explicit approval, append the approval boundary, hash, reason, and
   evidence pointer to `APPROVALS.md`.
8. Install the candidate byte-for-byte as the new numbered specification. In the
   same change, update the active identity, compiler frontend and checker,
   conformance cases and verdicts, focused reference models, tests, and live
   documentation derived from the specification.
9. Run `make -C compiler check`, the exact grammar-candidate check when relevant,
   and `make check` before committing the completed slice.

If the candidate changes after approval, stop: the earlier approval no longer
covers it. Repeat exact review and approval for the new bytes.

## Changing protected tests or reference material

Adding a new regression or conformance case is allowed. Changing or removing an
existing conformance verdict, protected case, reference judgment, or frozen
oracle requires explicit owner agreement and an `APPROVALS.md` entry. Never
weaken expected behavior merely to make a compiler check pass. An unsupported
compiler capability is not a source-language rejection.

## Recording a decision or approval

- Do not create or append to a directives log. Route each new ruling directly
  to the authority that owns it; the retired mixed log is preserved at
  `archive/governance/directives.md`.
- Update `docs/roadmap.md` only when current status, authorization, or next work
  changes.
- Update `mcts_mem/` only for a durable design choice with a real alternative;
  keep implementation activity and approval bookkeeping out of the tree.
- Append to `APPROVALS.md` only for an explicit protected owner approval. New
  entries record the date, owner, exact artifact or change boundary, SHA-256
  when bytes are approved, reason, and evidence location.
- Keep commits cohesive; do not use instructions or README files as progress
  logs.

## Checks and hooks

Install the tracked pre-commit hook once per worktree:

```sh
make install-hooks
```

Run the repository gate before committing a completed slice:

```sh
make check
```

The hook invokes `make spec-append-only-staged`; the ordinary repository gate
invokes `make repository-invariants` and `make spec-append-only`. These checks
use only Make, Git, `cmp`, and `grep`. The hook enforces only that released
numbered specifications are append-only. Keeping the compiler, conformance
corpus, reference models, tests, and docs consistent with the active
specification remains the implementer's responsibility.
