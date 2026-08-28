---
name: whitefoot-parallelism-doctrine
description: "Owner's 2026-08-20 parallelism frame: permission-from-proof primary, actualization secondary, resources out of the language, loops+recursion focus, DOM/grep as validation scenarios"
metadata: 
  node_type: memory
  type: project
  originSessionId: ab659700-c818-4271-84f2-e403a962b810
  modified: 2026-08-23T07:15:52.349Z
---

Owner's design statement (2026-08-20, after the 16-agent parallelism research
sweep), stated as durable doctrine for the parallelism lane:

1. **Three distinct things, never conflated:** parallelism (overlapping
   computation for performance), concurrency (independent external
   communication effects), threads (a resource mapping work to hardware).
2. **Resources are NOT language concepts.** Thread is just one shallow
   resource abstraction; big/little core placement and cache allocation are
   others. The language must not model hardware. (This kills any `task`/
   `Workers`/thread-construct surface and dissolves the "is thread creation
   `external`?" deadlock — no construct exists to carry the row.)
3. **Permission before actualization.** WF is proof-centric: derive
   non-interference pervasively (most statements, loop iterations, recursive
   calls) → "parallel permission conditions" attributed by the checker.
   Turning permission into actual parallel execution is runtime capability,
   possibly plus optional keywords — but keywords must NEVER be the
   precondition for whether parallelism is possible. Analogy the owner gave:
   `claim` does not drive proof/trap generation; it is constrained by checker
   power (a claim a better checker can discharge becomes a compile error).
   Same subordination for any `par` keyword.
4. **Profitability is runtime's call, code-invisible** (`a=1; b=2` should not
   spawn threads — splitting is usually slower). This is consistent with the
   recorded auto-parallelism rejection: what was rejected is compiler-static
   discovery of PROFITABLE parallelism; legality attribution + runtime
   actualization is the surviving half.
5. **Loops and recursion cover nearly all real parallelism needs.** Solve
   those two shapes first.
6. **Validation scenario:** a realistic application, not a framework — the
   browser DOM pipeline (style/layout/rendering over one tree: parent-child
   recursion, bottom-up merges, genuinely-sequential inline flow that must be
   correctly DENIED permission) and wfgrep. "如果真正的能让浏览器的 layout 和
   rendering 在 dom tree 上自动的并行化,就是非常漂亮的结果。"
7. **Priority: parallelism > concurrency > threaded-runtime surface.**

**Why:** the owner sees WF's proof system as the differentiator; parallel
permission should grow automatically as the checker grows, and actualization
must not contaminate the language with hardware modeling.

**OWNER RULINGS 2026-08-21 night (supersede parts of the debate verdict
below):** (1) claim doctrine (see [[claim-doctrine]]) ⇒ traps only occur in
review-failed (defective) programs ⇒ "我们应该不需要考虑这种情况" — the
trap-arbitration machinery is DEPRIORITIZED; v1 eligibility = transitively
claim-free + external/blocks-free lanes (PAL's stance vindicated; my
constitution derivation mooted). Consequence: divergence clause AND EFF-4
two-half ruling both drop out of v1 (join waits all lanes; no trap sites ⇒
no hang-to-trap conversion possible). The 4th P condition (no exit edge of
s1 skips s2) REMAINS required. (2) IMPLEMENT NOW: overnight branch
delegation per the new merge-boundary process — worktree + branch, spec
candidate + compiler + tests + demo, never block, merge packet ready by
morning; Opus writes the code, Fable only for the very hardest.

**OWNER RULING 2026-08-23 (claim-conditional guarantee — CHARTERED, MUST
LAND, do not lose):** a correct WF program cannot trap (checker complete →
no trap; incomplete + all claims humanly proven → no trap); a trap proves
the program itself defective, and "程序设计不应该为错误的程序垫背" — never
withhold a legal program's optimization to preserve behavior of a defective
one. Claims are the one place trust goes to the non-author reviewer; assume
them true for optimization. Execution plan (designed 2026-08-23, pending
implementation after batch B): (1) [PAR-1]/[PAR-2] amendment — delete
eligibility=claim-free, add the erroneous-execution clause (schedule
guarantee conditional on contract compliance, mirroring SCOPE-3's
conditional form; erroneous executions promise exactly one well-formed
DIAG-3 record of A false claim, which one may vary); (2) runtime
first-trap-wins latch (atomic flag, one coherent record, no arbitration);
(3) WF_WORKERS=0 = deterministic sequential reproduction (two worlds,
free); (4) delete claim_closure gating + not-actualizable verdict class;
(5) elision-rank arbitration moves to .alt as rejected-for-defective-
programs. Batch record quotes the owner's message verbatim.

**OWNER RULING 2026-08-21 (performance bar, binding on the whole lane):**
"如果只是等同于Rayon那这个方案没有意义。这个方案的理论上限明显高于Rayon,
所以如果不能明显快于Rayon那么就是失败" — rayon PARITY IS FAILURE; the
deliverable is clearly-faster in ABSOLUTE wall time on the paired oracle.
The structural argument for the higher ceiling (use it, don't re-derive):
rayon is a library with a ~4.5 ns/fork floor (type-erased job + deque
protocol + unwind machinery it cannot elide because it cannot see the
program); WF's compiler owns every fork site and can emit two-version code
(parallel driver + zero-overhead sequential clone, switch keyed on runtime
scheduler state only) reaching the Cilk work-first bound T_P ~ T_seq/P +
O(steals) — per-fork cost ~0 on unpromoted paths. Plus no dynamic-safety
machinery (claims abort, frames statically sized), plus compiler-controlled
allocation locality as a further seq-floor lever. Never present a
rayon-matching result as success.

**Round-3 debate verdict (2026-08-21, debate/ corpus + a3-compare):** v0
proposal died in 4 places by its own defense (all three divergence options;
claim-count eligibility; "no coordinator"; parallelism-first priority).
Surviving core = **elision-rank join arbitration** (lanes park records in
own slots; join scans in elision order, waits on lower-rank unterminated
lanes — preserves hangs with NO termination judgment, byte-identical trap
records for ANY claim-site count; Cilk hold-and-reraise precedent; one
draft bug found+fixed: arbitrate once some operand parks and all lower-rank
resolved). This SUPERSEDES the claim-count policy below (claim count is now
only a machinery-cost tier). P needs a 4th condition (no exit edge of s1
skips s2 — g2_propagate.wf compiling counterexample; early-exit ≡
divergence). Owner decisions pending: D1 resequence I/O lane first
(2.83x vs 0.15%, both sides agree); D2 bank Stage-1 judgment+ledger+
non-authoritative marker (PAL's pal = endorsed form) as measurement
instrument, one plan approval; D3 law packet gated on: arbitration
adversarial pass, EFF-4 two-half ruling (deferred delivery + siblings
continue past failure), protected-premises amendment (DIAG-3 poverty,
allocator neutrality, elision-total-order), Amdahl share ≥~30% threshold
with standing kill condition. Key unmeasured premise: wall-time share of
P-satisfying phases (one-afternoon profile test). Debate probes must land
in a durable repo home with any decision (evidence mortality).

**Claim policy (derived from constitution, 2026-08-21, owner prompted the
derivation after rejecting PAL's claim-free rule as over-restrictive):**
single claim-site identity per overlapped region is admitted free
(DIAG-3 records carry no lane/iteration data, so first-to-fire is
byte-identical; passing-claim ORDER is a non-observable — PAL protected a
non-observable); multi-identity regions are not actualized in v0 but the
denial must be VISIBLE (W1 gates every slower-but-accepted divergence;
PAR-4 forbids hidden serialization) with mechanical fixes (dominating-claim
merge, fission); deterministic claim election at joins is the specified
escape, built only on W1-audit evidence (Balance rule: P0 wins). Healing is
monotone: stronger checker ⇒ fewer claims ⇒ wider eligibility, zero source
edits. Full text: do_not_scan/wf-parallelism-research/claim-policy-derivation.md.

**Related artifact:** `research/investigations/proof-derived-parallelism/PAL.md`
(untracked, another agent, 2026-08-20) — same frame, adds: the `pal` marker as
a NON-authoritative structural obligation (unmarked code analyzed identically;
pal only makes "no nontrivial plan derivable" a rejection at that site — the
claim-dual, and the conformance-anchor answer); the P/D/C/M four-layer split
(Decomposition = finite verified plan families, checked plan IR + serial
interpreter differential path); the scan family (measure→scan→place) as the
constructive repair for inline-flow denials; structural-guard vs profit-guard;
diagnostic taxonomy (proved-dependence / unknown / unsupported / unprofitable).
Its gaps: divergence hole identical to ours (termination absent from its
observation list); claim policy stricter than ours (claim-free closures vs our
verified single-claim-site class) — a real design fork to adjudicate.

**How to apply:** frame all parallelism work as (a) determinism-of-observables
as the enabling law (runtime choice must be unobservable or facts-off
correctness breaks), (b) permission-condition derivation from ownership/
effects/proofs, (c) runtime actualization with proof-discharged (trap-free)
regions as the zero-protocol fast path. Fully-discharged obligations don't
just elide checks — they unlock parallel actualization. See
[[whitefoot-purpose]], [[compiler-search-findings]] (legality attribution
pays; profitability attribution doesn't).

**Correction 2026-08-27 (feedback):** while ranking rewrite targets I filtered out
programs that "already use all cores" (nginx workers, HAProxy nbthread, Varnish,
ImageMagick OpenMP) as having no performance headroom. Owner: "并不是说一个程序占
满了所有核心他就到了性能上限，这是两回事。wf性能上限极高，很多时候远超手工制造的多线程，
比如par能自动的把整个程序里可以并行的部分全部并行，这可不是手写可以替代的". Core
occupancy measures how coarsely a program was cut, not how much independent work
remains serialized inside each piece. Never use "already multithreaded" as a
headroom filter. The right question for a target: how much fine-grained
independent work (per request, record, block, pixel, node) does its hot path
serialize, versus how much is inherently sequential (range coding, sequential
guest-program semantics inside an interpreter). Also requested that day: find
credible public submit-and-rank benchmarks/competitions as a WF promotion venue.

**Owner thesis 2026-08-27:** "它的par和io都需要在大量的代码下面才能显出威力。代码量越大，
手工编排多线程越难，而wf的自动化par优势更大。io也一样。所以可能还是得从中等到大规模项目
上面入手". Consequence: micro-benchmarks (1BRC-style single hot loop, plb2) are the
WORST place to demonstrate WF; the demonstration must be a medium-to-large
program with many independent units and real I/O. I had proposed 1BRC as the
anchor the same day; retracted. Ladder under discussion: a medium network
program with natural fan-out (Mosquitto publish->N subscribers, dnsmasq) then
Redis. Crux to flag every time: cross-command parallelism on dynamic keys
needs runtime-actualized disjointness (PAR-4), beyond what [PAR-1]/[PAR-2]
prove statically today.
