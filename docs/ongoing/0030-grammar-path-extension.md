# 0030 — compiler grammar-path extension for batch 1

This is a temporary live coordination record, not execution authority.

- **Status:** `BLOCKED` on one item only — the grammar path is complete and
  the verifier is green at 65/74/77; `make -C compiler check` cannot close
  until the derivation ledger carries META-6 rows for the eight new rules
  (see Progress). The original blocker below is resolved by the owner's
  atomic-activation ruling.
- **Authority:** `ACTIVE` `docs/current-plan.md` selected slice items 1 and 3,
  and the owner's 2026-08-07 approval of
  `governance/spec-evolution/obligation-discharge-batch1-candidate.md`
  (`governance/APPROVALS.md`)
- **Owner / workspace:** exec-0030 /
  `/Users/bytedance/do_not_scan/wf-0030-worktree`, branch
  `task/0030-grammar-path-extension`
- **Base revision:** 76b59f7 (rebased 2026-08-07)
- **Dependency:** none (candidate approved; this task gates v0.21 candidate
  generation per ruling O1)

## Goal

Extend the compiler's native grammar path so the batch-1 grammar delta
verifies: add the `claim_stmt` production (per the candidate §2), the two
tokens `claim` and `because`, and the `index_get` reserved-name row, through
the lexer, parser, and generated syntax data. Success criterion: the native
grammar verifier accepts the candidate grammar with 65 productions and 77
terminal predicates (the candidate §3's post-extension expectation), while
the approved v0.20 grammar continues to verify unchanged (64/74/75).
Grammar path only — no checker, entailment, or lowering semantics; those
are later tasks in this slice. `make -C compiler check` green before and
after.

## Progress

Round 1 (2026-08-07): orientation only; stopped before any compiler edit and
reported the mechanism blocker recorded below. Baseline
`make -C compiler check` green at d459b49.

Round 2, under the atomic-activation ruling: the grammar path is extended and
every identity pin repointed on this branch (commit 0156826).

- The verifier accepts `kernel-spec-v0.21-candidate.md` at **65 productions,
  74 decisions, 77 terminal predicates**, exit 0. The decision count is
  unchanged because `claim_stmt`, like `check_stmt`, is a pure sequence that
  owns no predictive decision. Main is untouched and still verifies the v0.20
  candidate at 64/74/75.
- Candidate SHA-256 on this branch:
  `815dea4c60de56c2d32c0b52ba0062912ace5420f2c1d5100cff7c7de985ca85`.
- Table delta: fixed spellings 67 -> 69, terminal predicates 75 -> 77,
  productions 64 -> 65, SELECT_2 rows 1925 -> 1959. The 34 new rows are
  exactly the check/else mirror the isomorphism predicts; no LL(2) conflict
  appeared, and cross-arm disjointness holds (`claim` and `because` are new
  fixed terminals, so they no longer satisfy IDENT).
- 462 compiler lib tests green; `cargo fmt`, `clippy -D warnings`, and
  `cargo doc -D warnings` green; the standalone conformance corpus is
  unaffected and stays pinned to v0.20 at 120/120.
- One-shot generator used for the table renumbering and deleted with its
  scratch directory; nothing generated-by-script lands in the repository.

**Remaining blocker (one).** `whitefoot-spec` fails: the derivation ledger
has no row for `[CLM-1]`, `[CLM-2]`, or `[ENT-1]`..`[ENT-6]`. The rule-count
pin is already moved to 128. Authoring those eight META-6 rows is
specification-drafting judgment — each row states a derivation chain from the
constitution and a derived / existence-only status that feeds the ledger's
statistics line — not an identity repoint, and the ledger lives under
`spec/`, which this task is instructed not to touch. Precedent: at the v0.20
activation (18359d5) the ledger rows already existed from the candidate stage
and activation only relabelled and recounted them; the v0.21 candidate on main
carries no such rows. Fabricating chains for an audit artifact would be worse
than leaving them undone, so this stops here for the lead and the drafting
agent.

## Blocker (round 1, resolved by the atomic-activation ruling)

The success criterion cannot be reached by the written scope, and the
obstacle is not an LL(2) conflict. `whitefoot-grammar` has only a
single-contract, grammar-preserving mode, so its two conjuncts are
unsatisfiable in one build:

- The reported triple is computed by `verify_compiler_grammar()`
  (`compiler/src/bin/grammar.rs:134`), which takes no argument and reads
  only the committed tables. One build therefore prints exactly one
  triple for every accepted input; it cannot print 65/77 for the candidate
  and 64/74/75 for v0.20.
- Acceptance is decided before the tables are consulted:
  `verify_candidate()` (`compiler/src/bin/grammar.rs:96`) rejects unless
  the candidate's three `FRONTEND_SECTIONS` are byte-identical to
  `ACTIVE_KERNEL_SPEC_BYTES` (= `spec/kernel-spec-v0.20.md`). `[GRAM-4]`
  lies inside the first of those sections (offsets: `[FORM-1]` 18531,
  `[GRAM-4]` 33924, `## 4. Types` 41209), so any candidate carrying the
  §2 delta fails closed no matter what the lexer, parser, or generated
  data contain.

Reproduction (both runs at d459b49, before any change):

```sh
cargo run --manifest-path compiler/Cargo.toml --bin whitefoot-grammar -- \
  governance/spec-evolution/kernel-spec-v0.20-candidate.md
# -> 64 productions, 74 decisions, 75 terminal predicates; exit 0
cargo run --manifest-path compiler/Cargo.toml --bin whitefoot-grammar -- \
  /Users/bytedance/do_not_scan/wf-0030/batch1-grammar-probe.md
# -> candidate changes the lexer or source grammar of the active
#    specification; exit 1   (probe = v0.20 bytes + exactly the §2 delta)
```

Extending the *active* tables in place instead would break three things
at once: the parser would admit `claim`/`because` statements v0.20 does
not define; `index_get` in the resolution catalog's dotless-operation
reservation (`compiler/src/resolution/catalog.rs:978`) would reject
declarations spelled `index_get` that v0.20 accepts — the candidate §1
lists that shrink as a v0.21 change; and the active-grammar assertion at
`compiler/src/bin/grammar.rs:339` (64/74/75) would become false, i.e. this
task's own second conjunct.

Precedent for the missing mechanism: the last grammar-extending
candidate used dual-contract *staged* tables selected by the SHA-256 of a
byte-exact candidate frontend snapshot (4ea068a, task 0005 — 3893-line
`staged.rs` plus selection threaded through terminal classification,
parser engine and diagnostics, finalize, driver, and the verifier; ~5000
lines). That mechanism was deliberately retired at v0.18 activation
(9768bae) and nothing of it survives in the tree. Its table data came
from an offline one-shot generator that is not in the repository, and the
data is not hand-editable: the tables carry 1925 SELECT_2 rows over a
358-atom pool with FOLLOW-driven position-1 predicates — `else`, which
occurs only after `check`'s `expr`, already appears in position 1 of 14
rows, so the new `because` terminator propagates comparably across the
expression decisions.

Open decision for the lead and owner (this task record contains none of
them, and an executor selects none): rebuild the staged dual-contract
path, change the verifier's acceptance model for a grammar-extending
candidate, or resequence so the grammar extension lands with activation.
Whichever is chosen also needs a decision about the table generator.

## Stop condition

Round 1's stop condition was cleared by the owner ruling. The current stop is
the eight missing derivation-ledger rows; the task resumes when they land, at
which point `make -C compiler check` is expected to close with no further
compiler change.

## Plan repair (lead, 2026-08-07, owner ruling "3")

Blocker resolved by owner ruling: atomic activation. Re-scoped goal — on
the task branch: (1) grammar tables extended by mirroring the else-row
pattern for `because` (one-shot generator, deleted after use, as fallback);
(2) verifier green at 65 productions / 77 terminal predicates against the
generated `kernel-spec-v0.21-candidate.md` (assembled by the drafting
agent, landing on main first); (3) every identity pin repointed on the
branch (including the grammar.rs 64/74/75 assertion), 0029-style; (4) STOP
before merge — deliver the full-document SHA-256 for the owner's step-4
exact-byte approval; installation and integration only after that entry.
Main stays 64/74/75 green throughout. Success criterion supersedes the
original record's; APPROVALS.md 2026-08-07 sequencing amendment is the
authority.
