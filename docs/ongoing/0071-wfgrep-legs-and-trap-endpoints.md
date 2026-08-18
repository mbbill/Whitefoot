# 0071 — wfgrep functional legs and trap-endpoint closure

Owner: lead. Workspace: `batch-0071` branch with executor worktrees on
file-disjoint briefs. Base: main at the plan-activation commit.
Registered: 2026-08-18 under the ACTIVE Current Plan (W1, W2, W4).

## Authority

The ACTIVE `docs/current-plan.md` (owner approval 2026-08-18, "批了").
Protected surfaces produced here — the v0.32 candidate bytes and any
conformance family — are marked candidates awaiting the owner's
exact-byte approval; nothing activates on this branch.

## Scope

- W1: affine-element buffer lowering (`buffer_vacant` construction,
  element replace/vacate, per-element drop loop); byte-string program;
  wfgrep recursive-traversal slice (or the honest system-surface gap
  report if traversal needs declarations v0.31 lacks).
- W2: one v0.32 candidate composed by the lead from three deltas —
  division/remainder zero-divisor obligations (#48), check dissolution
  (#47), declaration-site rejection of ambiguous-provenance borrow
  returns (#50). Executors deliver delta documents, implementations
  behind default-off switches, ordinary tests, and migration
  inventories; the spec file has one writer (the lead).
- W4: batch-end adversarial audit and the owner approval packet.

## Out of scope

Everything the plan excludes; any activation; any unmarked protected
change; merging to main. Executors do not edit `spec/kernel-spec.md`,
`tests/conformance/`, `governance/`, or the plan/roadmap.

## Morning review (2026-08-18, batch end)

Branch `batch-0071`, ~60 commits over main `3a9204bf`, linear. Eight
executor briefs (five parallel + three follow-ons) plus one repair
executor; three transient API-stall casualties, all resumed with zero
work lost. Full `make check` green at the tip; adapter Pass=489
Skip=1 Fail=0; coverage 135/135 (0 uncovered); canonical corpus 3/3.

### What landed, by workstream

- **W1 capability legs.** Affine-element buffers real end to end
  (`buffer_vacant` construction, element replace/vacate through SET-2,
  per-element drop loop, OP-9 size trap verified at runtime;
  `option_slots.wf` runs clean — one manual `leaks --atExit` run showed
  zero leaks; the harness test observes drop-loop shape, not
  leak-freedom). Byte-string program over the growable-vector layer
  (`byte_string.wf`; the searcher functions hold `deny_claims` and emit
  zero `wf_trap`). The directory-traversal system surface: SYS-14,
  `DirectoryList`/`ListOutcome`, three operations, and `dir_walk.wf`
  performing a REAL recursive directory enumeration with deterministic
  sorted output. HONEST GAP: the plan's W1 evidence sentence ("wfgrep
  compiles and runs a real recursive search") is NOT met and is not
  achievable in this candidate — no operation opens a file by an
  enumerated name (`open_read` still takes a `RelativePath`), so the
  deliverable is traversal-plus-listing; `wfgrep.wf` itself is
  byte-unchanged. File-open-by-name is the recorded next gap.
- **W2 trap-endpoint candidate.** v0.32 CANDIDATE at digest
  `efaf0ec4e2d7c31518f4e817faa55fcb412f8a8cec542883b4c051917b06e1f3`,
  135 rules, grammar 74/96/99 two-path verified, zero activation
  artifacts. Check dissolution (#47): body `check_stmt` leaves the
  statement alternation, the production survives as the contract
  final; ENT-3.S2 retired; measured semantic delta showed migration is
  branch-for-branch behavior-preserving and refutation upgrades a
  defective always-false check from runtime abort to compile-time
  rejection. Division dissolution (#48): the divisor class (unsigned,
  or one constant operand) carries a two-conjunct obligation;
  signed two-variable sites keep their trap (the safe condition is a
  disjunction L0 cannot state). Declaration-site provenance (#50):
  ambiguous borrow-result signatures reject at the declaration's
  rtype; the binding-side inert state and its OWN-6 rejection are
  deleted; laws recorded: bindable iff usable; a declaration whose
  result no caller can use is itself the error.
- **Protected conformance candidate.** Corpus 461 → 490 case files;
  138 leg-A migrations (byte-exact, audited 235/235 lines), 17
  subject/trap decision cases, 7 refutation-guard repairs, 28 new
  cases (12 division, 4 declaration-provenance, 10 SYS-14, strict-in-U
  FN-8, borrowed-arena STOR-4), EX-1 reproduced byte-for-byte, 5
  verdict flips + 17 citation rows all surfaced as decision rows in
  the marked commit messages. Every conformance-touching commit is
  titled PROTECTED CANDIDATE; verdict-drift re-run confirms zero
  undisclosed changes.

### Exit audit and dispositions

Four adversarial finders (spec integrity, protected surface, compiler
semantics, process/false-green) + executor-level refutation. Nine
majors, seventeen minors; every finding dispositioned before closure:

- REPAIRED in the candidate: open_directory's name range now carries
  the SYS-8 validation that backs its traps row; the status line
  discloses the division narrowing; DIAG-2's restructuring string,
  FN-8/FN-9 grammar recaps, FN-9 body-check residue, SYS-10 paragraph
  placement, SYS-14 house-style paragraphs, QUAL-3 in-place-rewrite
  reconciliation.
- REPAIRED in the compiler: the generic-divisor EFF-2 dead end (a
  symbolic-type bare `/` was unwritable at any row; the body-syntactic
  contribution is now judged once on the written body per the new
  OP-2 sentence); the unclaimable signed zero-divisor conjunct
  (written constant zero now interns as the distinguished Z term, so
  the diagnostic's own mechanical fix works); the two tautological
  traversal differential tests now compare the false/true inventories
  for real; dead switch residue (DECLARATION_PROVENANCE fully retired
  like CHECK_DISSOLUTION), stale switch prose, redundant hand-pins,
  and module-wide guard counts.
- REFUTED by reproduction: the claimed FN-1 acceptance widening for
  region-bearing borrow results — STOR-4 already rejects a borrow-mode
  arena result at the rtype (either mode, now stated explicitly) and a
  borrow-of-arena parameter is a capability stop; the FN-1 sentence
  became a scoping clause and the corpus gained the pinning cases.
- RECORDED for the packet (below): the plan-expansion deviation, the
  #50 rule widening, the dead strict-in-U clauses, and the owner-level
  conformance decisions.

### Final state

`make check` exit 0 at tip; lib 952/0; programs 46/46; adapter
Pass=489 Skip=1 Fail=0; coverage 135/135; canonical 3/3; candidate
digest `efaf0ec4…e1f3` (supersedes the audit-window digests
`1936cd2e…`/`342af789…`, both historical); chain 23 unbroken, no
activation artifacts. Nine frozen `research/**` probe `.wf` files no
longer parse under the candidate BY DESIGN — recorded evidence is
never restated at a version it was not measured under.

## Owner approval packet

THE SINGLE ACT: approve batch 0071 = merge `batch-0071` to main plus
the v0.32 activation commit (archive v0.31, flip the status line,
chain line, identity), naming candidate SHA-256
`efaf0ec4e2d7c31518f4e817faa55fcb412f8a8cec542883b4c051917b06e1f3`.

Decisions folded into that review, each stated once:

1. **Plan-expansion ratification.** The ACTIVE plan's W2 authorized
   three deltas; W1's traversal contingency authorized a gap report.
   The lead folded the traversal surface (SYS-14, +25 declaration
   records — the candidate's entire rule-count delta) into the
   candidate under the owner's overnight direction ("不要block等我批…
   把代码,测试,所有的事情都推进到位"), without a plan amendment.
   Approving the packet ratifies it; rejecting it means the lead
   strips SYS-14 into a separate candidate.
2. **#50 rule widening confirmation.** The landed rule also rejects a
   same-region parameter of the other borrow kind and any parameter
   whose written type names the result region — wider than the
   owner's literal sentence, on v0.31's own soundness argument (a
   shared result may derive from a unique parameter through a nested
   call; treating the same-kind parameter as sole debtor would
   mis-root the loan).
3. **Protected decision rows.** 5 verdict flips (three CLM-3, the
   zero-divisor trap→reject, the OWN-6→FN-1 provenance case), 17
   citation rows, 7 refutation-guard repairs, and the trap-case
   consolidation (four cases now share the one legal always-false
   claim spelling). Deferred renames (case ids that no longer name
   their subject) and the OWN-6/OP-5 attribution questions (OWN-6
   negative coverage 1→0; OP-5 positive citations 9→0 while sixteen
   contract finals still exercise it) are listed for rulings, not
   silently resolved.
4. **Dead strict clauses (future candidate).** The strict-in-U
   OP-4/OP-2 rejection clauses are unreachable under v0.32 (U blinds
   only S3; CLM-3 fires first) — dead normative text recommended for
   retirement in the next candidate; the reachable FN-8-in-U path is
   now corpus-pinned.
5. **Tool ruling.** `whitefoot-migrate` is a v0.22→v0.23 one-shot
   nine versions past its purpose and cannot migrate the constructs
   it names; recommend deletion next batch.
6. **Protected wiring correction.** `compiler/tests/conformance/`
   docstring still records the pre-batch tally (untouched: gate
   wiring); correct it under this approval or defer.

Recorded follow-ups (next plan, no approval needed now): the
file-open-by-name operation (real searching wfgrep), Linux
enumeration mapping, Option niche layout (#46), the OWN-6
InvalidChildReborrow unpinned rejection surface.
