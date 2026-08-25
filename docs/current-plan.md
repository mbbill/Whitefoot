# Current Plan — proof-derived parallelism

Status: PROPOSED (branch `par/proof-derived-parallelism`, 2026-08-21; updated
on the branch 2026-08-22 for merge review, again 2026-08-23 on
`par/loop-permission` for batch 0078, and again after the rebase onto `main`).
This proposal authorizes no execution. Batches 0074, 0076, 0077 and 0078 were
carried out under the owner's chartering directions of 2026-08-21/22/23, quoted
verbatim in their records; this plan is the durable sequencing those directions
imply, and it becomes ACTIVE only with the owner's approval at merge.

Derived from Direction Outline revision 50 and main at
`18d332e7`. Supersedes both the completed claim-only trap-surface plan and
main's implemented claim-residual-canonicality plan, in place; that plan's
remaining sequence is carried forward below rather than dropped.
Active language authority: v0.36 at `spec/kernel-spec.md`, SHA-256
`fd57cfc4bfcf685f14b073c98e149c8a44a201dc79fbd76075ebd49a87995c62`,
activated by the owner-approved merge of 2026-08-23; it carries [PAR-1] v2
and [PAR-2], and supersedes v0.34, whose bytes are archived as
`spec/kernel-spec-v0.34.md`.

## Objective

Make Whitefoot's existing proofs pay for concurrency. The compiler already
derives, for acceptance, exactly the facts a legality judgment for overlapped
execution needs: resolved places, exact effect rows, the [OWN-7] overlap
relation, the [EFF-2] call-boundary projection, and the call graph. Read a
permission off those proofs, state the law that a permitted overlap is
unobservable, and let a runtime decide profitability. No writer construct
declares parallelism, no resource is a language concept, and no accepted
program changes.

## Workstreams

- **W1 — compute lane v1 (landed on the branch; the first item this plan
  covers).** The permission judgment P over sibling call pairs, the
  non-normative `--par-ledger` developer output, runtime actualization behind
  `WF_WORKERS`, the spec CANDIDATE v0.35 [PAR-1] rule, compiler tests
  including each named counterexample shape, and the measured demo. Recorded
  in `docs/ongoing/0074-proof-derived-parallelism.md`.
- **W1b — the optimization campaign (landed on the branch, batches 0076 and
  0077, chartered by the owner's directions of 2026-08-21/22).** Against a
  paired benchmark oracle with a Rust/rayon twin: the hand-out frame moved
  off the activation record; the lane scan replaced by per-thread
  work-stealing deques; two-world compilation selecting a sequential clone
  once per process; the permission window generalized over interposed
  statements; the `band` discharge read through its proving binding; the
  ENT-4 closure made ~5x faster (compile time only, emission byte-identical);
  the shipped default changed so an unset `WF_WORKERS` in a `--par` binary
  runs its parallel world (default-behavior change, flagged for owner
  approval at merge); a second oracle family (`grid`, recursive index-split)
  and a counted-loop ledger hint. End state on the N=18 authoritative
  rotation: rayon wins zero cells; matched worker counts 14 WF wins / 25
  parity / 0 losses; each language's shipped default 11 / 2 / 0. Records
  `docs/ongoing/0076-par-optimization-digs.md` and
  `docs/ongoing/0077-night-par-ceiling.md` carry the digs, the adversarial
  audit's dispositions, and one recorded invariant breach (a w1-only
  code-placement regression on three cells, attributed, W>=2 unaffected).
  **Batch 0078 continues W1b on `par/loop-permission`, chartered by the
  owner's two directions of 2026-08-23 and recorded in
  `docs/ongoing/0078-loop-permission.md`.** It carries the first item of W4
  forward — see W4 below — and it redirected the claim doctrine: the
  claim-free actualizability gate is deleted from both permission judgments,
  and `wf_trap` carries a first-trap-wins latch instead. A second protected
  conformance annotation, for [PAR-2], is prepared and flagged there.
- **W2 — the I/O concurrency lane (first among the remaining work).** A
  completion-based family for overlapping host waits. This is where
  the investigation found the profit: 2.83x on the dominant term of a
  directory-walking workload that is 86% I/O, against a compute-lane delta of
  roughly 0.15% of the same frame — with the recorded caveat that part of that
  2.83x is an artifact of the measuring machine's security daemon and must be
  re-measured before the number is used to justify anything. It is a separate
  packet: it needs its own spec question
  (`blocks` rows are exactly what W1's row gate refuses), its own runtime, and
  its own owner approval. Nothing in W3 precedes it. **W4's first item does:**
  this plan sequenced W2 ahead of all of W4, and the owner's chartering
  direction of 2026-08-23 put counted-loop permission first instead. That
  direction governs; the sequencing sentence is corrected here rather than
  left to contradict the branch.
- **W3 — the `pal` marker.** The writer-visible structural obligation of
  PAL.md §6: non-authoritative, never gating legality, and therefore a grammar
  plus FORM-table plus teaching-text packet of its own. It buys the writer a
  gradient the ledger currently supplies by hand.
- **W4 — permission growth.** Indexed-loop permission (Tier A) and
  buffer-view splitting (Tier B), each with its recorded hazard ([OWN-9]
  granularity; the c2-F4 aliasing case), and claim-bearing actualization with
  the trap-arbitration ruling already on file. ~~Each widening is a [PAR-1]
  amendment~~, which was stated as the cost W1's necessary-condition form
  imposes and the first thing to revisit if it bites.

  **Tier A landed on `par/loop-permission` as batch 0078, and it bit.** The
  widening is a **new rule [PAR-2]**, not a [PAR-1] amendment. The reason is
  recorded at
  `research/investigations/proof-derived-parallelism/loop/DESIGN.md:86-88`:
  the pair conditions and the quantified loop conditions read badly
  interleaved, and a separate rule keeps the byte surface the owner reviews
  minimal. That is a deliberate departure from this line, and it makes
  **[PAR-2] a second protected annotation** at merge rather than the single
  [PAR-1] one the Exclusions below name.

  **Claim-bearing actualization also landed, and not by arbitration.** The
  owner's second direction of 2026-08-23 refused the trap-arbitration ruling
  this line pointed at: a claim is the writer's always-true lemma, so an
  execution that reaches a trap is erroneous and the program is defective, and
  a correct program must not pay to make a defective one's report
  reproducible. The claim-free actualizability gate is deleted from both
  judgments; the elision-rank arbitration alternative is refused rather than
  deferred, with the evidence promoted to
  `research/investigations/proof-derived-parallelism/debate/d1-defense.md`.

## Boundaries and invariants

Permission is never an obligation: a build without `--par` is byte-identical
to today, an explicit opt-out (`WF_WORKERS=0` or `=1`) selects the sequential
world, and every test that passes sequentially must pass under overlap. Since
batch 0077 an unset `WF_WORKERS` in a `--par` binary defaults to one lane per
logical CPU — published bytes stay identical at every worker count, but the
default execution is parallel; that default stands only with the owner's
merge approval. Worker count, schedule, and
thread identity are outside the language and outside every rule. Acceptance is
untouched in both directions — P consults typing, rows, places, the CFG, and
~~the call graph~~, never the entailment fact state, so facts-on and facts-off
behavior are identical by construction. (Batch 0078 removed the call graph from
that list: deleting the claim closure took its last consumer, so
`permission.rs` now carries the functions and their signatures alone.)

~~No arbitration machinery, parked lane, or coordinator is built while eligible
regions are claim-free.~~ **Replaced by batch 0078's claim redirect.** Eligible
regions are no longer required to be claim-free, and the guarantee [PAR-1]
makes is now conditional on contract compliance in the sense [SCOPE-4] fixes:
for a correct program, which reaches no trap, nothing changes. The boundary
this sentence drew becomes: **no arbitration machinery and no coordinator is
built, and the one latch that exists lives in the overlapped world only.** A
module emits it exactly when it both carries a `claim` and hands a call out to
a worker lane; the default build and a `--par` build that actualizes nothing
emit the pre-latch trap path unchanged. It parks only a losing thread of an
already-erroneous execution, and the winner's abort takes the process down with
it, so exactly one well-formed [DIAG-3] record is written under any
interleaving.

## Acceptance

- A real program compiles, is judged, hands work to lanes, and produces
  byte-identical output at every worker count, with the granted-lane count
  measured rather than assumed.
- Each of the four permission conditions denies its own named counterexample,
  and each denial cites the condition that actually judged it.
- The ledger explains every analyzed site, so a sequentialized region is
  visible rather than silent.
- The repository gate is green with the candidate declared, and the owner
  packet carries the exact candidate SHA-256, diff, impact inventory,
  verifier output, and the protected coverage annotation [PAR-1] needs.

## Carried forward from the claim-residual-canonicality plan

That plan was IMPLEMENTED AND MIGRATED when this branch forked, and superseding
it in place must resolve its remaining sequence rather than drop it. Its six
items, with their state at this branch tip:

1. *Review the final branch diff for regressions.* Open — it is this branch's
   merge review, and it now covers the union of both programs.
2. *Preserve the measured no-regression result and record the inherited
   per-mask whole-program residuality risk; re-run compile-cost probes if
   later code changes touch that path.* **Engaged.** This branch changed that
   path: the rebase kept main's ENT-4 closure index and dropped this branch's
   own dense-matrix variant of it, so the compile-cost probes are re-run here
   rather than inherited.
3. *Bring the live roadmap and batch record to the same implemented state.*
   Done for main's program: roadmap revision 50 records claim locality as
   landed, and `docs/done/0075-claim-residual-canonicality.md` stays main's
   own live record, untouched by this branch.
4. *Freeze the exact ACTIVE branch revision and finish the specification and
   conformance before/after content.* Open, and now over v0.34 to v0.35:
   ACTIVE v0.34 has digest `cb747505…`, its outgoing immutable v0.33 archive
   has digest `fc6b5a10…`, and the v0.35 candidate's digest and both merge-time
   recipe digests are recorded in batch 0078.
5. *Commit the final bytes, then run canonical root `make check` on that exact
   revision.* Open — the merge packet reports it.
6. *Present that exact tested revision for the single owner approval.* Open.

Main's acceptance criteria for claim authority remain in force and are not
restated here; nothing in this plan relaxes one. The one criterion this branch
touches directly is that expected failures and runtime observations use
ordinary control or typed outcomes rather than deliberately false claims — the
trap-latch cases now meet it by fault injection into checked IR.

## Exclusions

No writer parallelism construct, no thread or task type, no heartbeat
profitability policy, no `reduce`-clause regrouping, no arbitration for
trapping lanes, and no generic-container work.

Two exclusions this plan carried are overtaken by batch 0078 and are corrected
rather than left standing:

- ~~no arbitration for claim-bearing regions~~ — claim-bearing regions are now
  overlapped like any other. What stays excluded is *arbitration*: no rank, no
  coordinator, no wakeup protocol. The latch is not arbitration; it is one
  `cmpxchg` on a path a correct program never executes.
- ~~no protected conformance change beyond the single [PAR-1] coverage
  annotation~~ — there are **two** annotations for the merge approval to name,
  [PAR-1] and [PAR-2]. Neither is landed on the branch; both are prepared as
  exact bytes in `docs/ongoing/0078-loop-permission.md`, and each is applied
  only together with its own rule text.
