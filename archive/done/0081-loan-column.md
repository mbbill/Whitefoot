# Batch 0081 — the loan column: ownership under order relaxation

Branch: `par/claims-column`, from main at `03a17d5c`.
Deliverables: the loans half of the overlap permission (commit `db543775`),
spec v0.36 activating it as rule text (commit `e4c8ba08`), this record.

## Charter

The owner found the hole, chartered the research pipeline, and made the
naming ruling. Verbatim, 2026-08-24:

> 假设这两个函数都拿uniq对象进去,都不写,在线形的时候是合法的,这两个uniq
> 不相交。但是并行是不行的,他们在两个线程里面跑,生命周期相交了。而且因为
> 他们都不写入,所以目前的rules似乎并不会禁止他们?而且我看到目前的rules
> 不考虑ownership,所以这里可能还有其他类似的问题。

> 我感觉这还是有点修修补补的意思 … 应该反过来,让ownership能够覆盖多线程
> 的范围?不过这似乎有一个先后问题,ownership checker发生在后端之前,除非
> 让ownership知道两个函数会被分发,它才能合理的做判定。

> 派发Opus做调研。1. 思考有没有更优雅的方案。2. 对最佳方案做漏洞攻击,
> 审计。3. 在1和2完成以后拿着这个方案再做一次audit,看看加上这个方案的
> 并行是不是真的完备。4. 没问题的话就开始实现,不要等我 … 5. 实现完以后
> 开agent攻击它 … 6. 添加测试用例覆盖所有需要覆盖的内容。7. 用explain
> code写文档报告。

> claim这个名字已经用了,而且那个claim特别重要。所以我们是不是最好换个
> 名字?

The name chosen is **loan**, [OWN-5]'s own word (`"loan state"`,
`"another live loan"`, `check_loan_access`); `claim` belongs exclusively
to the CLM-1 statement.

## The holes, all verified by compiling probes

The judgment projected only effect rows onto footprints, so a borrow whose
callee row said nothing about it contributed nothing. Five reachable
shapes, every one permitted before this batch:

1. Two `&uniq` of one place, both rows reads-only (the owner's example).
2. Two `&uniq` of one place, both rows `pure` — zero footprint entirely.
3. A `pure` shared borrow of a box against the `move` that frees it: the
   overlap would run a use-after-free shape had rows been inexact.
4. Interposed statements: a window statement between the members could
   form a borrow, or write a place a member's borrow spans, unjudged.
5. The loop judgment shared the blindness, and it is the actualized half:
   a body taking `&uniq` of one loop-invariant outer cell with a
   reads-only row was permitted *and emitted as a split* — N workers
   concurrently holding what the source spells as N exclusive borrows of
   one place. A body statement binding a bare borrow of outer storage was
   the same hole one layer down, found only when the spec drafting forced
   the question.

None is a reachable data race today, because [EFF-2] row exactness is
enforced in both directions: a claim-free borrow genuinely touches
nothing. What they break is [OWN-5]'s unconditional exclusivity
invariant ("no two live usable `&uniq` borrows have overlapping resolved
places") and [OWN-9]'s optimizer license ("one usable mutable path per
place") under the overlapped schedule. The backend does not cash that
license yet — no aliasing attribute is emitted anywhere — so the fix
lands before the license is ever exploited rather than after.

## Research (owner's steps 1–3)

Two Opus agents ran independently, then a third audited the synthesis.
Their probe suites live under `do_not_scan` scratch directories; every
load-bearing finding was re-verified by the lead on the real compiler.

- **design-scout** ranked five candidates by building or refuting them:
  the loan column wins; reliance edges are the same fact stored in a
  worse place (the checker has no admission event to hang an edge on —
  a call-site temporary borrow is checked by finding *nothing* in the
  loan map, and nothing is recorded); schedule-parametric ownership puts
  a scheduling question into acceptance, which [PAR-1] forbids
  ("whether an overlap was performed at all is not observable");
  uniq-as-write was built and measured to miss both shared-loan holes;
  weakening [OWN-5] does not close the shared-side holes at all. It
  also found the vocabulary already existed at every call boundary
  ([OWN-12]'s per-call claim overlap in `calls/user.rs`), making the
  design a lift of an existing concept, not an invention.
- **claims-attacker** broke the first draft four ways (interposed
  statements outside the column; the loop path consuming only the
  written half; unresolvable loans silently dropped behind the
  row-gated resolution; syntax-keyed loans missing holder pass-through)
  and measured **zero** permitted-set regression across every probe,
  loop, and bench source in the repository.
- **completeness-auditor** confirmed the amended design guards every
  channel it enumerated (values, footprints, ownership invariants,
  [OWN-9], external actions, exits and trap identity, CLM-1 reach,
  drops, resource records, nondeterminism, two worlds) — in the
  compiler — and found the remaining defects were all *rule text*:
  [PAR-1]'s "exactly when" list constrained nothing between the
  members; [PAR-2]:2042's three combination-tree disclaimers each
  asserted the opposite of the only implementation; [PAR-1]:2021
  promised source-order values at an abandoned point no overlapping
  implementation can deliver (the refused-lane path runs s1 after s2);
  [PAR-1] lacked [PAR-2]'s system-operation clause.

## The design, as landed

Each window statement's footprint carries, beside its row-projected
uses, its **loans**: for every borrow-moded argument, keyed on the
parameter's mode (never on expression syntax), a loan on the place
`argument_place` resolves — exclusive for `&uniq`, shared for `&`.
The conflict rule is `check_loan_access`'s own matrix lifted to the
window: an exclusive loan excludes every overlapping loan or use, a
shared loan excludes overlapping writes, two shared loans coexist. Both
directions are judged; the operand-evaluation asymmetry stays for
operand reads (they precede the fork under both schedules) and never
gates a loan. An unresolvable loan denies. A non-call window statement
that forms a borrow denies as a form — the checked tree erases a
written borrow's mode, and an unloaned borrow would widen permission —
and the same refusal guards the loop body. The loop judgment gains its
own loans path and denial slot: an exclusive loan on storage the
iteration does not introduce denies; shared loans stay free, keeping
read-only sharing across iterations permitted. `ConflictKind` becomes
the product of two `FootprintHalf`s, and loans are cited ahead of uses
so a denial names the cause rather than the row projection downstream
of it.

Measured on the whole corpus after landing: permitted pairs 65, loop
verdict lines 19, chains 55, splits 5 — identical to baseline; the only
diffs are already-denied pairs whose cited reason improved. The
flagship shapes are untouched: `layout`'s two members sharing `words`
(shared∩shared), the tree folds' disjoint `move` binders, and the
bisection chain over one read-only buffer.

## Spec v0.36 (activation in this branch, approval at merge)

[PAR-1] gains the loans half with its derivation in rule text —
overlapping two statements makes both statements' borrows simultaneously
live and usable, so permission requires of the union loan state exactly
what [OWN-5] requires of one statement holding it — plus the repairs the
audit demanded: the window-statement conditions, the system-operation
clause, and an abandoned-continuation sentence scoped to what survives
an abort. [PAR-2] gains the loans condition and combination-tree
sentences that admit the emitted split: every admitted operation is
commutative with a two-sided identity on its type's complete value set,
and admitted trees carry the leaves in any order plus any number of
identity leaves. Rule count stays 137; the grammar is untouched; the
permitted-overlap set only narrows, so no acceptance or conformance
verdict moves and the conformance corpus is untouched. v0.35 is
archived byte-exact; chain, identity, qualification review, and digest
anchors are updated; canonical `make check` is green end to end.

## Approval classes for the merge

- Specification bytes change (v0.36 activation): the merge-time record
  is in `governance/APPROVALS.md` and becomes effective with the
  owner's merge approval of this exact revision.
- No conformance content changes.
- No new root entries.

## The implementation attack (owner's step 5)

A fourth Opus agent attacked the landed batch with a differential rig: it
built the pre-change parent compiler from a `git archive` export and ran
every probe on both. Verdict: no unsoundness found; every closed hole
stays closed, chains truncate on non-adjacent loan conflicts, and the
emitted split tree matches v0.36's combination sentences at the IR level.
It found and this batch repaired:

- A new wrong denial: the loop-body borrow guard refused borrows of
  iteration-own storage, which the pre-change compiler permitted and
  split; the corpus could not see it because the whole repository
  contains exactly one borrow-binding `let`. The guard now resolves the
  borrowed place and admits it when the iteration introduces it; a
  shared borrow of enclosing storage stays knowingly denied, because the
  checked tree erases the borrow's mode (restoring it is future work,
  pinned by a test).
- Two over-statements in the new [PAR-1] window sentence, both places
  the rule demanded more than any implementation needs: two intervening
  statements owe each other nothing (they run in source order on one
  thread under both schedules), and the s1-operand exception now also
  names loans. The rule text was repaired; the compiler was already
  right.
- The interposed `replace` arm gets the borrow guard the other two forms
  had, dead today by [SET-2]'s region-free target rule and commented so.
- The system-operation clause was held up by the operand walk's
  fail-closed catch-all alone; a test now pins that denial so the clause
  cannot vanish silently under a smarter operand walk.

Known over-refusals, accepted deliberately: an interposed `let` binding a
borrow of member-untouched storage denies the window (no iteration-own
notion exists there), and a body-bound shared borrow of enclosing storage
denies the loop.

## Named dependencies (soundness leans on these; breaking one breaks it
silently)

- [EFF-2] row exactness in both directions keeps loans-without-uses
  unobservable at value level.
- [OWN-13] suspension and the statement-scoped child-region shape keep
  sibling reborrows structurally unreachable; the extension path is
  test-only today.
- Slice unwritability (spec's "no writable target path may traverse a
  slice") keeps descriptor-anchored slice loans sound.
- The backend emits no aliasing attributes; this batch is what makes
  cashing [OWN-9] legal later, not what cashes it.
- Arena appends stay behind the arena lane's fail-closed unsupported
  gate; `Access::Arena` never overlaps a place.
- The `OwnershipJoin` unsupported gate on a borrowed `match` inside a
  counted loop: the loop's iteration-own test lists match arm binders as
  introduced, and borrow-mode binders anchor at themselves rather than
  at their scrutinee's place, so lifting that gate would widen the
  writes half and the loans half at once. Lift it only together with
  binder-place anchoring.
- Three structural facts close the bypass shapes the attack could not
  break: a statement-scoped child reborrow needs its own one-statement
  region block, so two can never be adjacent candidates; [GRAM-9] makes
  call arguments atoms, so no borrow or call hides inside an operand
  expression; and a slice's shared loan is held region-wide by the
  borrow checker, not per statement.
