# I/O model — implementation handoff

For a fresh agent with zero prior context. Everything you need is named
here; read in the order given, then start Phase A. **Run all phases to
completion in one pass — A, its activation and implementation, B, C —
without stopping for mid-run approval. The single owner gate is the
final merge into `main`: present the finished, `make check`-green
revision with every flagged decision listed, and wait only there.**
Delete this file in the change that completes or supersedes the last
phase.

## Read first, in this order

1. `AGENTS.md` at the repository root — project law: the four
   branch/merge rules, repository hygiene, test integrity, compiler
   rules. Binding on you.
2. `docs/constitution.md` — especially theorem **T3** (defective
   executions never tax correct programs, with its premise) and W3's
   claim discipline. Two prior reviewers independently re-proposed a
   trap-free gate this project had already removed; T3 exists so you
   do not become the third. Any proposal that narrows permission for
   correct programs to stabilize a defective execution's observables is
   pre-refuted.
3. `research/investigations/io-model/DESIGN.md` (this directory) — the
   design, revision 2. It is the authority for what you build. §10
   lists five flagged decisions: adopt the RECOMMENDED option for each,
   record the adoption in your batch record as a flagged decision, and
   keep going — the owner's merge approval of the final exact revision
   is the approval for all of them. Never stop to ask mid-run.
4. `research/investigations/io-model/reviews/` — the three adversarial
   reviews behind revision 2. `spec-sweep.md`'s per-sentence table is
   your Phase A working material. Note the header of each: the
   trap-gate disposition those reports proposed was overruled by T3.
5. `docs/current-plan.md` W2, and the active `spec/kernel-spec.md`
   (v0.36): [EFF-1..5], [SYS-*] (~line 2230 on), [PAR-1]/[PAR-2]
   (~line 2000 on), [TRAP-1], the entry form [FN-7].
6. `docs/done/0081-loan-column.md` — the most recent worked example of
   a complete specification batch: research → implementation → spec
   activation mechanics (archive the outgoing version byte-exact,
   digest chain in `governance/APPROVALS.md`, regenerate
   `compiler/src/spec_identity.rs` via
   `cargo run --release --bin whitefoot-spec -- --emit-identity src/spec_identity.rs`,
   transcribe the two literals in `compiler/src/spec.rs`, update the six
   digest anchors that `make spec-digest-sync` names, bump
   `REVIEWED_FOR` in `compiler/src/backend/qualification.rs` with a
   dated review note, update the header META-5 delta declaration).

## State at handoff (2026-08-25)

- `main` carries: spec v0.36 (loan-conditioned overlap permission), the
  work-stealing parallel runtime, the exhaustion floor, the personal-
  path gate in `make repository-invariants` (never introduce an
  absolute home path in tracked content), and this investigation.
- Settled, do not reopen: the completion model over readiness (§2);
  the loan column and its vocabulary — the ownership concept is named
  **loan**, `claim` belongs exclusively to the CLM-1 statement; T3;
  research is done — your job is execution, not re-research.
- The canonical gate is `make check` at the repository root, and the
  exact revision merged into `main` must pass it; every merge into
  `main` needs the owner's approval of the exact revision. Work freely
  on branches. Keep a batch record in `docs/ongoing/` while you work
  (next batch number: 0082) and move it to `docs/done/` when the batch
  closes. All repository artifacts in English.

## Phase A — the specification migration, on paper first (DESIGN §3,
falsifier 1)

Goal: a candidate spec delta against v0.36 implementing the
conservative-first migration (DESIGN §3e option 1): delete the
`external`/`blocks` row atoms; introduce world-region kinds and
capability-carried world identities (§3a–§3d); join one conservative
global world-order domain to every former-`external` operation so
v0.36's [EFF-5] order promise is preserved byte-for-byte in behavior;
generalize `blocks` into target-action metadata (§3g); rewrite the
[PAR-1]/[PAR-2] erroneous-execution sentences per §3f, carrying T3's
direction clause into the rule text.

Method: walk `reviews/spec-sweep.md`'s table row by row; every `OK` row
takes its listed rewrite, every `RESISTS` row must be discharged by a
rule your delta actually contains — the sixteen amendments at the end
of that report are the checklist. Then the conformance migration: the
report enumerates 42 case files and 7 verdict-sensitive manifest
records; no verdict changes silently, `reject-syseff-declared-
unexhibited` keeps testing the declared-but-unexhibited direction with
a well-bound world row, the same-sink [EFF-5] runtime witness keeps
passing under conservative aliasing, provenance-only cases keep their
verdicts (the [PRV-1] `external` is a homonym — flagged decision 5
covers its rename; adopt the recommendation and continue).

Acceptance: every sweep row dispositioned; a complete draft spec text;
the conformance ledger; the compiler-side work sized (the EFF-2
projection extension and the capability world-region representation are
the two real pieces, DESIGN §3a/§3d). Then continue straight into
activation and implementation: activating the spec on your work branch
is ordinary branch work under the four rules — only the merge into
`main` waits for the owner.

## Phase B — the kqueue prototype (DESIGN §4, falsifier 2)

Goal: measure whether the completion runtime earns a batch. In the
runtime (`compiler/src/backend/par_runtime.c` + a new completion
translation unit), split `list_once`/`read_once` into submit/complete
behind the existing pool for the directory-walk workload, on this
machine's kqueue backend: one never-blocking waiter thread, a
fixed-depth blocking disk pool, an MPSC mailbox with preallocated
per-frame completion nodes. Honor every §4 requirement as a checklist —
progress-then-rescan parking, announce-then-recheck, release/acquire
publication, generation-tagged frames, ring/mailbox affinity to the
executing lane, bounded join helping, in-flight loans to terminal
state. Re-measure the directory-walk speedup whose recorded 2.83x
carries a measurement-artifact caveat (the machine's security daemon;
see `docs/current-plan.md` W2). Deliverable: the honest number, the
overhead of serving completion from readiness, and any shape the loan
machinery could not cover — recorded in the batch record and carried in
the final merge packet. Do not stop here: proceed directly into Phase C
and the runtime batch regardless of the number; if the measurement
argues against the direction, say so prominently in the packet and let
the owner weigh it at the single final gate.

## Phase C — the full backend matrix (owner direction, 2026-08-25)

Not optional and not Linux-gated. Write the completion backend for all
three hosts against one shared contract:

- **macOS / kqueue** — the Phase B implementation, finished to
  production shape.
- **Linux / io_uring** — per-lane rings, POLL_ADD-on-eventfd multishot
  with re-arm discipline, the unified single parking point; this is
  where DESIGN §9's falsifier 3 runs (park law under mixed load,
  measured against the condvar path).
- **Windows / IOCP** — the same contract over a completion port.

Development and the full local test pass happen on this macOS machine;
Linux and Windows are exercised through GitHub Actions: add a workflow
that builds the runtime translation units and a C-level harness driving
the shared contract (submit/complete, park/wake state machine, mailbox
invariants, in-flight loan discipline) on `macos`, `ubuntu`, and
`windows` runners. Where the compiler does not yet qualify a target
(check `compiler/src/backend/qualification.rs` — target rows are
per-triple and review-gated), the CI still compiles and tests the
backend C against the harness, so the code is proven before the target
row lands; extending target qualification is its own reviewed step, not
a CI side effect. The backend selection is a build/TCB matter and never
appears in the language.

## Standing cautions

- Verify before you assert: compile probes, quote ledgers, cite
  file:line. A reviewer's report — including the three in `reviews/` —
  is a claim until you have reproduced its anchor.
- Fail closed everywhere: an unresolved place, an unclassified form, an
  unproven disjointness denies. Widening permission silently is the one
  failure direction this project does not tolerate.
- The corpus cannot see every regression (the whole repository contains
  almost no let-bound borrows, and no I/O overlap yet): construct new
  probes for both sides of every line you draw, and diff verdicts
  against a pre-change compiler built from the parent commit.
