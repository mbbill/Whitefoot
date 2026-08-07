# 0030 — compiler grammar-path extension for batch 1

This is a temporary live coordination record, not execution authority.

- **Status:** `BLOCKED` (plan defect: the written scope cannot reach the
  written success criterion; see Blocker)
- **Authority:** `ACTIVE` `docs/current-plan.md` selected slice items 1 and 3,
  and the owner's 2026-08-07 approval of
  `governance/spec-evolution/obligation-discharge-batch1-candidate.md`
  (`governance/APPROVALS.md`)
- **Owner / workspace:** exec-0030 /
  `/Users/bytedance/do_not_scan/wf-0030-worktree`, branch
  `task/0030-grammar-path-extension`
- **Base revision:** d459b49
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

Orientation complete; stopped before any compiler edit. No file under
`compiler/` is touched by this branch. Baseline `make -C compiler check`
green at base revision d459b49 (v0.20 identity
`b082ef3f…312dc1`, 120 rules).

## Blocker

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

Reached. The task resumes only under a repaired plan item that names the
mechanism and the generator.

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
