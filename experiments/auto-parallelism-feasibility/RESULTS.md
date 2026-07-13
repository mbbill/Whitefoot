# Study 3 — Auto-parallelism feasibility: RAW RESULTS

Research workflow: 4 prior-art surveys -> adversarial verdict. Verbatim.


## FEASIBILITY VERDICT

# XL Feasibility Verdict: "Write Sequential, Get Safe Automatic Parallelism"

**One-line verdict: NO on the strong claim, narrow PARTIAL yes on a scoped feature. Do not bet the project on auto-parallelism. The scoped win is real, congruent with XL's actual goal, but it is a feature, not a mission — and XL's own showcase kernel is not even decided by the effect system alone.**

Confidence: high on the structural argument (four independent survey clusters agree and XL's own prior study corroborates); medium on exact x-factors (grounded to the one measured kernel family).

---

## 1. Does XL's design provide what's NEEDED?

**What regions + effect rows DO give you (real, and better than C):** Two calls `f(); g()` are provably independent — reorderable and parallelizable — iff their effect footprints satisfy Bernstein's conditions over regions: `writes(f) ∩ (reads(g) ∪ writes(g)) = ∅` and symmetrically. XL's per-fn rows (`reads('r)`, `writes('r)`, `pure`, `allocates`, `traps`) encode exactly this at **region granularity**, from signatures alone. So yes — the compiler can prove *coarse, task-level* independence with no interprocedural body analysis. This is genuinely more than C's may-alias fog gives a compiler, and it is what DPJ/Legion also had.

Two XL-specific bonuses worth noting: `traps` in the row lets the compiler *see* observable-panic effects and refuse reorderings that would change which trap fires first — a real determinism asset most Bernstein-style systems lack. `pure` + `allocates` cleanly separate the reorderable-anything case.

**The decisive gap — region granularity is too coarse for the volume case.** Effect rows prove *task* disjointness (call A on region X ∥ call B on region Y). They do **not** prove *element* disjointness within one region. A data-parallel loop `for i: out[perm[i]] = f(in[i])` is, to the effect system, "writes('out)" on every iteration — one region, so the rows say *conflict*. The fact that makes it parallel — `perm` is a bijection, so the element footprints are disjoint — is **not expressible in the effect vocabulary XL currently has.** It needs index-parameterized regions / injectivity refinement (DPJ's `[e]` RPLs and "distinctions from the left/right"), which is precisely the heavy machinery that broke DPJ's annotation budget.

So: **XL's own lone measured structural win (disjoint scatter, 1.13–1.51×) is NOT provable from regions+effects alone.** It requires injectivity reasoning that lives *outside* the effect system. That is the single most important technical finding here.

**What is MISSING beyond the type system:**
- **A cost model — entirely absent.** Effect rows carry no notion of *how much work* `f` does. Independence ≠ profitability. This is unsolved by any type system (Halide's lesson: independence is the easy 5%; scheduling/granularity is the 95% that needed learned search even when independence was free).
- **A parallel-safe runtime scheduler.** To decide granularity dynamically you need a work-stealing runtime (Cilk/rayon-style). The type system contributes zero here.
- **Element-level disjointness / injectivity** (see above) — the load-bearing missing piece for anything past task-level.
- **Reduction determinism.** FP reductions are non-associative; reordering changes results. Rows don't capture associativity/commutativity, so deterministic reductions need a commutativity annotation — and DPJ's was *unchecked/trust-based*, which its own authors called "highly unsatisfactory."

---

## 2. The four traps against XL, concretely

**Trap 1 — Annotation burden. XL escapes the COARSE tax, inherits the FINE tax.** This is where XL is genuinely differently positioned than DPJ, and it must be stated precisely:
- DPJ's regions existed *only* for the parallelism proof — pure overhead a programmer would never otherwise write (hence ~10.7% of lines, up to 22.6%, and the DPJizer inference tool). 
- XL's regions/`&'r`/`&uniq 'r` are **already mandatory for the memory-safety story (D1).** The task-level parallelism proof rides for *free* on annotations the programmer already pays for aliasing/safety. **This is a real structural advantage over DPJ at task granularity.**
- BUT the amortization only covers the coarse names. The moment XL wants per-iteration / loop-level parallelism (where the actual volume of parallelism lives), it needs the *same* fine machinery that sank DPJ — index-parameterized regions, region parameters threaded through signatures, injectivity proofs. Those are *not* amortized by the safety story. So XL escapes DPJ's coarse burden and inherits DPJ's fine burden exactly in proportion to how "pervasive" it wants to be. The design lever that decides survival: **effect rows must be inferred intraprocedurally and written only at module boundaries.** Hand-written rows at every signature = DPJ's effect summaries = death.

**Trap 2 — Granularity/cost model. This kills the strong claim outright.** XL contributes *nothing* to this trap that DPJ didn't. Proving disjointness gives zero granularity guidance (the whole cluster confirms: nobody's type system decided granularity; it was always solved by a runtime work-stealer or punted to the human). If XL removes the human's parallelize-or-not decision (the "auto" in auto-parallel), it *owns* the trap that defeated every general auto-parallelizer for 40 years, and the HELIX limits result says even a *perfect* disjointness oracle exposes only modest parallelism in general-purpose code because the code is genuinely, truly-data-dependently sequential. XL's effects are that oracle; the oracle was never the bottleneck.

**Trap 3 — Determinism. Cheap in the data-parallel domain, not a differentiator, fights perf only at the edges.** In the array/scatter domain determinism is essentially free (autovec cluster: determinism never cost perf there because the domain is embarrassingly parallel). XL's `traps`-ordering is a bonus. But it's *not* a differentiator — every array system already has it — and the moment you want the fast non-deterministic reduction you're back to a trust-based commutativity hole. XL cannot be simultaneously guaranteed-deterministic, fast at disjoint-scatter, *and* soundly fast at reductions.

**Trap 4 — Adoption. The cluster's revealed preference is brutal and consistent.** Low-ceremony + good runtime scheduler won massively (Cilk → TBB/OpenMP-tasks/rayon). Sound region+effect determinism systems (DPJ, sound polyhedral auto-par) are off-by-default or dead. Soundness never drove adoption; DPJ was sound, inference-assisted, deterministic, performant — and got ~zero uptake, and its own creator (Bocchino, WoDet 2013) walked away from the effect-annotation layer as "the wrong layer." XL's realistic competitor is `.par_iter()`: one token, already *ties* XL's ceiling (prior study), already solves granularity via work-stealing.

---

## 3. Strong vs weak claim — which is it, honestly?

**It is the WEAK version, unambiguously.** XL will not make genuinely sequential general-purpose code auto-parallel. What it can actually do:

- **(a) Coarse task-level:** auto-spawn two calls with disjoint region footprints. Real, DPJ-level, and cheaper than DPJ because the regions are safety-amortized — but straight-line code rarely contains large independent calls, the compiler almost never finds *more* than the human would, and without a cost model it can't tell if it's profitable. Low practical value.
- **(b) Marked data-parallel loop, safety automated:** the programmer still writes the loop shape (`for i … out[i] = f(in[i])`); XL proves it safe to run parallel *without* the programmer having to establish Send/Sync/injectivity or drop to `unsafe`. This is the ISPC/Futhark shape: **automate the SAFETY of programmer-exposed parallelism, not the DISCOVERY of parallelism.** Same category as every prior system.

So the "sequential" in "write sequential, get parallel" is really "write the obvious loop shape and skip the concurrency ceremony." **What XL removes is the ceremony (Send/Sync/Arc/Mutex/`unsafe`), not the decision.**

**Is the weak version a real Rust win? Yes, but narrow.** On the disjoint scatter/gather band — provable index bijections — safe Rust's borrow checker *cannot prove `perm` injective*, so it forces either `unsafe` or a reformulation that pays redundant memory traffic. XL makes the naive form safe, deterministic, and parallel. Measured residual: **1.13–1.51× (memory-bound) over best-effort-safe Rust**, plus the ergonomic delta of no-`unsafe`/no-ceremony. That is the ISPC/Futhark win shape, and it is durable because it's a *structural* borrow-checker limitation in Rust, not an effort gap.

---

## 4. VERDICT

**Strong claim — "compiler discovers parallelism in sequential code and beats Rust": NO.** Killed by Trap 2 (no cost model, no granularity — the type system cannot supply it) and by the HELIX result (an oracle disjointness proof does not manufacture independent work in general code). Secondarily undercut by the fact that XL's *own* best kernel isn't decided by effect rows alone.

**Scoped claim — "make the naive data-parallel shape safe + deterministic + parallel without ceremony/`unsafe`": PARTIAL YES.**
- **Code class:** data-parallel array kernels, specifically disjoint scatter/gather with provable index bijections (particle reorder, radix/bucket sort, spatial sort) — the band where safe Rust structurally can't apply rayon without `unsafe`.
- **Size:** ~1.1–1.5× memory-bound over best-effort-safe Rust on that family, plus the ergonomic/safety delta. Modest and narrow, not an integer-factor general win.
- **Language MUST provide:** (1) a work-stealing runtime with a dynamic granularity threshold — copy Cilk/rayon; the type system will not solve granularity; (2) element-level disjointness reasoning (index-parameterized regions / injectivity refinement), kept **inferred and boundary-only** or it reintroduces DPJ's fatal annotation burden; (3) effect-row inference (write at module boundaries, infer inside); (4) a determinism policy that exploits `traps`-ordering and provides a *checked* (not DPJ-trust) commutativity story for reductions.

**Which trap kills what:** Trap 2 (granularity) kills the strong/general claim regardless of how good the region system is. Trap 1 (fine annotation burden) kills the *pervasive/loop-level* version unless element-disjointness is fully inferred. The scoped scatter feature survives both only because its shape is narrow and its injectivity fact is small.

---

## 5. Worth betting the project on?

**No.** Do not re-aim the project around auto-parallelism. Four independent survey clusters plus XL's own prior study converge on the same conclusion with unusual consistency: soundness was never the bottleneck, the winners won on low ceremony + runtime scheduling, and the region+effect auto-par lineage (DPJ, general polyhedral, classic auto-par) is uniformly off-by-default or dead. DPJ is XL's near-twin — sound, deterministic, inference-assisted — and it died, with its creator disavowing the layer. Betting the project on "auto-parallel beats Rust" is betting on the single thesis in this space with the longest, clearest track record of failure. XL's own R0 evidence already shows rayon *ties* its ceiling and the only structural residual is one 1.1–1.5× kernel.

**But keep the scoped feature** — it is congruent with the project's actual stated goal (AI codegen + performance, shift-left/D1 safety). The honest, survivable framing: *"XL makes the naive data-parallel shape safe, deterministic, and parallel without `unsafe` or Send/Sync ceremony"* — positioned against rayon on the injective-scatter band rayon can't do safely, **not** against a mythical auto-parallelizer. Cite DPJ as the cautionary precedent and refuse to charge the programmer 10% of their lines for a proof they didn't ask for.

**Smallest decisive next test (do this before building anything):** A **hand-simulated auto-par pass, using ONLY the current region/effect vocabulary, over ONE real mid-size sequential XL-shaped program** (not a kernel — a parser, an interpreter loop, a small sim). Formalize the independence rule (Bernstein over effect rows) on paper, apply it by hand, and count three numbers:
1. How many independent, parallelizable sites do effect rows *alone* expose?
2. Of those, how many are *profitable* (need a work estimate the rows don't provide)?
3. Of the profitable ones, how many need element-level/injectivity reasoning *beyond* the current rows?

This is a 1–2 day design-spike/paper exercise, no implementation, and it is **falsifiable**: my prediction (from HELIX + the whole cluster) is that effect-rows-alone expose near-zero profitable, human-non-obvious parallel sites in real straight-line code, and that everything valuable clusters in (3) — the injective-array band that needs machinery you haven't specified. If that's the result, the strong thesis is dead on paper for the cost of two days, and you keep only the scoped scatter feature. If it surprisingly finds pervasive cheap parallelism, *then* escalate to the LLM annotation-burden study (M3 harness) — but only then. Cheap falsifier first; do not build the runtime to find out.

**Skeptic's bottom line: sound but a dead end as a general capability; a real but narrow feature on the disjoint-array subset. The distinction between those two is the whole ballgame, and XL's marketing must land on the second or it repeats DPJ's grave.**

---

Grounding: XL-specific x-factors and the "scatter is the lone structural win, not provable from effect rows" claim are from `/Users/bytedance/Dev/xlang/optimizer-language-research/notes/regions-effects-vs-safe-rust-2026-07-08.md` (measured, Apple M4, rustc 1.91.1). All prior-art claims (DPJ Table 3 ~10.7%/≤22.6%, Bocchino 2013 pivot, HELIX limits, Halide/Futhark/ISPC/polyhedral, rayon SPAA 2024 11/60/29 split) are from the four survey clusters supplied, which cite primary PDFs.


---

# Prior-art surveys (primary evidence)


---

# Prior-art surveys (primary evidence, verbatim)


## Prior-art survey: region-parallel-langs (Regent/Legion, Chapel, X10)

# Survey Cluster: region-parallel-langs — AUTO vs MANUAL parallelism

**Bottom line up front:** Across this entire cluster, *none* of these systems made genuinely sequential code auto-parallel. Every one required the programmer to **express the parallel structure** — either via explicit parallel constructs (DPJ `foreach`, Chapel `forall`, X10 `async`, Cilk `spawn`) or via explicit task decomposition + region annotations (Regent, Sequoia). What the effect/region systems actually automate is **safety proof + dependence discovery + scheduling**, not the *decision to parallelize* or the *decomposition*. That distinction is fatal to a naive reading of XL's "write sequential, get parallel" claim, and it is the central finding below.

---

## Per-system analysis

### 1. Legion / Regent (Stanford) — the closest thing to the XL dream
**Mechanism:** Programs are decomposed into **tasks**; each task declares **region privileges** (`reads`, `writes`, `reduces`) over its region arguments — these privileges *are* a read/write effect system. Task calls are written in ordinary **program order (sequential-looking control flow)**. The Legion runtime performs *dynamic* dependence analysis across the sequence of issued tasks (a "sequential semantics" guarantee) and executes non-conflicting tasks in parallel, also using privileges to drive data movement across the memory/node hierarchy.

**AUTO vs MANUAL — the crux:** This is the *most* "sequential-looking → parallel" system in the cluster, and it is still fundamentally **programmer-directed decomposition + automatic scheduling**. AUTO: the discovery of *which* task instances are independent, the dependence graph, the parallel schedule, and data movement. MANUAL: (a) carving the computation into tasks, and (b) annotating every task's region privileges. So the *control flow* reads sequentially, but the *decomposition and effect annotation* are explicit. It does **not** take an unannotated sequential loop and parallelize it.

**Four traps:**
- *Annotation:* Moderate-heavy. Privileges + region partitioning are mandatory and non-trivial; Regent (the language) exists specifically to reduce Legion-C++ boilerplate, which was severe. Partitioning logic is often the hardest part of a Legion program.
- *Granularity/cost:* **Explicitly punted out of the type system** into a separate **Mapping interface** — a distinct, manually-written (or heuristic) layer deciding where/when tasks run and at what granularity. This is strong evidence that proving independence ≠ deciding to parallelize; the granularity decision was hard enough to need its own subsystem.
- *Determinism:* Sequential semantics gives deterministic-by-default results; `reduces` privileges and relaxed coherence are the escape hatches for performance. Reasonably well balanced.
- *Adoption:* **The cluster's success story, but niche.** Real DOE/HPC use (S3D/combustion, the FlexFlow ML framework is built on Legion). Confined to elite HPC teams; not a general-purpose language. Confidence: high.

Sources: [Legion publications](https://legion.stanford.edu/publications/), [Regent 2015 paper](http://regent-lang.org/images/regent2015.pdf), [Legion GitHub](https://github.com/StanfordLegion/legion).

### 2. Chapel (Cray/HPE) — explicit data-parallelism, automatic *distribution*
**Mechanism:** Explicit `forall` loops and forall-expressions for data parallelism; explicit `begin`/`cobegin` for task parallelism; some *implicit* idioms (whole-array assignment, operator/function promotion, reductions, scans). Distributed arrays (domain maps) mean data-parallel ops over them *automatically* run across the locales owning the data.

**AUTO vs MANUAL:** The programmer explicitly says "this is parallel" (`forall`, promotion). What is automated is the **mapping of that expressed parallelism onto locales/threads and the communication** — not the discovery that a loop *could* be parallel. Whole-array promotion is the most "implicit" it gets, but that is still an explicitly parallel-semantics construct, not auto-parallelization of a sequential `for`.

**Four traps:** Annotation: light-to-moderate (multiresolution — you can stay high-level). Granularity: runtime + domain-map controlled; generally decent for regular arrays. Determinism: **not guaranteed** — `forall` explicitly permits races if you write them; Chapel chose performance/flexibility over determinism-by-default. Adoption: the healthiest of the pure research languages here — active development, HPE backing, real HPC users, but still small vs MPI/OpenMP. Confidence: high.

Sources: [Chapel data parallelism spec](https://chapel-lang.org/docs/language/spec/data-parallelism.html), [Chapel: Built for Parallelism](https://chapel-lang.org/parallel/), [Locality-based optimizations in the Chapel compiler](https://link.springer.com/chapter/10.1007/978-3-030-99372-6_1).

### 3. X10 (IBM) — fully explicit, no auto-extraction
**Mechanism:** APGAS model. `place` = locality domain; `async S` spawns a lightweight activity; `finish S` waits for all activities spawned within. `at(p)` shifts execution to a place. More general than Cilk's spawn/sync.

**AUTO vs MANUAL:** Essentially **all manual**. The programmer explicitly spawns every activity and explicitly bounds concurrency with `finish`. No effect proof of independence; no auto-parallelization. Safety (race freedom) is *not* guaranteed by the type system — hence a research literature on *may-happen-in-parallel* analysis and race detection *bolted on afterward*.

**Four traps:** Annotation: heavy and explicit by design. Granularity: programmer-controlled activity spawning + work-stealing runtime. Determinism: none by default. Adoption: **largely faded** — influential on the APGAS model and on Java's concurrency thinking, but the language itself is effectively dormant; IBM wound it down. Confidence: high on "faded," medium on exact status.

Sources: [X10 async-finish](http://www.kadix.ca/x10/doc/concepts/asyncfinishparallelism-1.html), [Work-first/help-first scheduling for async-finish (Rice)](https://www.cs.rice.edu/~yguo/pubs/PID824943.pdf), [May-happen-in-parallel analysis of X10](https://www.academia.edu/2234819/May_happen_in_parallel_analysis_of_x10_programs).

### 4. Sequoia (Stanford, Fatahalian) — explicit task hierarchy + explicit machine mapping
**Mechanism:** Algorithms expressed as **hierarchies of tasks**; leaf tasks compute on arrays that fit in a given memory-hierarchy level; data movement is *only* via subtask parameter passing. The programmer additionally supplies an explicit **mapping** of the task tree onto a specific machine's memory hierarchy.

**AUTO vs MANUAL:** Fully **explicit decomposition + explicit mapping**. Note the strong statement from the paper: *"Explicit memory management is necessary for program correctness, not just performance."* Sequoia deliberately rejected the auto-approach. Nothing sequential becomes parallel on its own.

**Four traps:** Annotation: heavy (task tree + per-machine mapping). Granularity: solved by the explicit mapping — good portability across memory hierarchies, but at the cost of writing the mapping. Determinism: functional/hierarchical structure gives predictable results. Adoption: **dead** — a clean, influential idea (its "programming the memory hierarchy" thinking fed directly into Legion), but zero durable user base. Confidence: high.

Sources: [Sequoia: Programming the Memory Hierarchy (SC06)](https://graphics.stanford.edu/papers/sequoia/sequoia_sc06.pdf), [Programming the Memory Hierarchy Revisited (Bauer, PPoPP11)](https://ppl.stanford.edu/papers/ppopp11-bauer.pdf).

### 5. Cilk (MIT) — explicit spawn, no effect proof, the *adoption* winner
**Mechanism:** `spawn`/`sync` express fork-join parallelism; a provably-efficient randomized **work-stealing** scheduler load-balances. Processor-oblivious. Crucially, **no static effect/region proof** — the programmer is responsible for race-freedom; the "Nondeterminator" is a *dynamic, per-input debugging* tool, explicitly not a verifier.

**AUTO vs MANUAL:** Manual expression of parallelism (`spawn`), automatic *scheduling*. This is the honest inverse of DPJ/XL: it automates the hard *scheduling/load-balancing* problem and gives up on *static safety*.

**Four traps:** Annotation: minimal (just `spawn`/`sync`). Granularity: handled superbly by work-stealing at runtime — the strongest granularity/cost story in the cluster, and it needed *no* type system to get it. Determinism: none guaranteed (races possible; dynamic detection only). Adoption: **by far the biggest real-world footprint.** Cilk's model is the direct ancestor of Cilk Plus, Intel TBB, OpenMP tasks, .NET TPL, and — relevantly — **Rust's rayon**. Confidence: high.

Sources: [Efficient Detection of Determinacy Races in Cilk (Feng & Leiserson)](https://homes.cs.washington.edu/~mernst/teaching/6.893/readings/feng-spaa97.pdf), [Cilk publications (MIT)](https://cilk.mit.edu/publications/).

### Reference point — DPJ (the ghost XL must not become)
DPJ is XL's near-exact ancestor: hierarchical regions + per-method read/write effect summaries, compiler-checked disjointness, **deterministic-by-default**. Two facts matter for XL:
1. **Parallelism in DPJ was still EXPLICIT** — `foreach` and `cobegin`. The effect system proved the programmer's *asserted* parallelism was disjoint/deterministic; it did **not** extract parallelism from sequential code. Same as everyone else in this cluster.
2. **DPJizer** could *infer* effect/region annotations to cut the burden — and it *still* got ~zero adoption. Sound, inference-assisted, deterministic, and dead. Confidence: high on model + adoption outcome; the "why" is the standard research-language story (Java-fork ecosystem cost, HPC preferring MPI/OpenMP, no killer app).

Sources: [DPJ current design](https://dpj.cs.illinois.edu/DPJ/Current_Design.html), [Cornell CS6120 DPJ writeup](https://www.cs.cornell.edu/courses/cs6120/2020fa/blog/parallel-java/), [Parallel Programming Must Be Deterministic by Default (HotPar09)](https://dpj.cs.illinois.edu/DPJ/Home_files/DPJ-HotPar-2009.html).

And the broader ground truth on true auto-parallelization: it has failed for decades precisely on *dependence analysis of pointers/indirection/recursion*, and separately because *"code does not always benefit from parallel execution"* (the granularity/cost-model trap) — per the standard surveys. Confidence: high. Source: [Automatic parallelization — Wikipedia](https://en.wikipedia.org/wiki/Automatic_parallelization) and [Automatic Parallelization: Fundamental Compiler Techniques](https://link.springer.com/book/10.1007/978-3-031-01736-0).

---

## Cross-cluster verdict on the four traps

| Trap | Cluster evidence |
|---|---|
| **Annotation burden** | Ranges from light (Cilk `spawn`) to heavy (Regent privileges+partitioning, Sequoia mapping). The *effect/region* systems (DPJ, Regent) sit at the heavy end — and inference (DPJizer) did not rescue adoption. |
| **Granularity/cost model** | *The universally under-solved trap.* Nobody's type system decided granularity. It was solved *outside* the effect system — by a runtime work-stealer (Cilk, the best), a separate mapping layer (Legion mapper, Sequoia mapping), or the programmer's loop structure (DPJ, Chapel). **Proving independence bought zero granularity guidance.** |
| **Determinism vs peak perf** | Determinism-by-default (DPJ, Legion sequential-semantics) is achievable and did not obviously *forbid* peak perf — but every system kept explicit escape hatches (`reduces`, relaxed coherence, non-deterministic `forall`) because someone always wants the last 20%. The perf ceiling of the deterministic subset was rarely the thing that killed adoption. |
| **Adoption** | Inverse-correlated with static-safety ambition. **Cilk (no effect proof, minimal annotation, killer scheduler) won massively. DPJ/Sequoia/X10 (sound and/or elaborate) died. Legion/Chapel (heavy but with real HPC value + funding) survive in a niche.** The market paid for *scheduling + low ceremony*, not for *static determinism proofs*. |

---

## The single most important lesson for XL

**"Write sequential, get safe automatic parallelism" is the wrong framing, and this cluster proves it in two ways.** First, empirically, *not one* of these systems — including the region+effect ones built for exactly this (DPJ, Regent) — auto-extracts parallelism from sequential code; they all require the programmer to **express the parallel structure**, and the effect/region system only proves that expression *safe and deterministic*. Second, the thing that actually determined survival was **scheduling quality + low ceremony (Cilk), or funded HPC value (Legion/Chapel)** — *never* the soundness of a static effect proof (DPJ was sound and inference-assisted, and died).

So XL's durable, honest advantage over Rust is **not** "the compiler discovers parallelism sequential code never asked for." It is the *narrower, defensible* claim: **once the programmer expresses parallel structure (a parallel loop / task), XL's effect+region types make it safe and deterministic-by-default with far less ceremony than Rust's `Send`/`Sync`/`Arc<Mutex>`/channel dance** — i.e., XL competes with *rayon-plus-guaranteed-determinism*, not with a mythical auto-parallelizer. If XL markets automation-of-decomposition, it walks straight into DPJ's grave. Its live shot is: pervasive, ceremony-free, *deterministic* parallelism on *explicitly-expressed* parallel structure — and it must still solve the granularity/cost trap with a runtime scheduler (as in Cilk/rayon work-stealing), because the type system will not solve it. Confidence: high.

## Prior-art survey: rust-parallelism (rayon, threads, async)

# rust-parallelism cluster — survey for XL's "write sequential, get safe automatic parallelism" ambition

All parallelism in Rust is **opt-in and manual**. The language/borrow-checker's contribution is *safety of parallel code once you write it* ("fearless concurrency"), not *deciding to parallelize*. Nothing in Rust ever turns sequential-looking code parallel on its own. That is the bar XL wants to clear. Below, per mechanism: how it works, what's auto vs manual, measured reality, and the four traps.

---

## 1. Rayon (data parallelism) — the real bar

**Mechanism.** A work-stealing runtime plus parallel-iterator adapters. You change `iter()`→`par_iter()`, `fold`/`sum`/`for_each` gain parallel variants. Rayon relies on the *existing* type system for safety: closures/data must be `Send`/`Sync`; the borrow checker forbids aliased `&mut`, so a data race simply won't compile. ([rayon](https://github.com/rayon-rs/rayon), [Red Hat: how Rust makes Rayon magical](https://developers.redhat.com/blog/2021/04/30/how-rust-makes-rayons-data-parallelism-magical))

**Auto vs manual.** *Safety* is automatic (compiler-proved via `Send`/`Sync`/borrow rules). *Everything else is manual*: the programmer decides **where** to parallelize (which loop gets `par_iter`), and implicitly **whether** it's worth it. Rayon does auto-manage granularity of splitting *once you've opted in* (adaptive splitting + sequential fallback), but it never chooses the parallelization site.

**Measured wins.** For "regular" parallelism (map/reduce/prefix-sum over slices) rayon is genuinely one-token ergonomic and competitive — this is exactly why the prior XL study found "safe rayon ties XL's ceiling" on a bothered-to-parallelize kernel. The SPAA 2024 port of 14 C++ benchmarks found Rust+rayon "delivers fearlessness for program phases comprising only regular parallelism, e.g., prefix-sum," and the perf gap to C++ was "not significant." ([SPAA 2024, "When Is Parallelism Fearless and Zero-Cost with Rust?"](https://dl.acm.org/doi/10.1145/3626183.3659966); [author PDF](https://utoronto.scholaris.ca/bitstreams/8321e66b-89f1-4ff1-bfb0-dc0fdb17f2dd/download))

**Four traps:**
- **Annotation burden — LOW (its superpower).** No regions, no effect rows. You reuse `Send`/`Sync`, which are *auto-derived* from field composition, so most types are thread-safe for free. This is the single most important fact for XL: the easy 80% of parallelism in Rust already costs ~one token and zero annotation. *(High confidence.)*
- **Granularity/cost model — UNSOLVED, punted to human.** Rayon adaptively splits, but the *decision to parallelize at all* is the programmer's, and getting it wrong makes code slower than sequential: fine-grained tasks let scheduling/sync overhead dominate. Community guidance is essentially "measure, and don't parallelize tiny work." ([rayon splitting discussion](https://github.com/rayon-rs/rayon/discussions/1134); [Endignoux: 10x faster with/without rayon](https://gendignoux.com/blog/2024/11/18/rust-rayon-optimized.html); [tiny-tasks tradeoff paper](https://arxiv.org/pdf/2202.11464)) *(High confidence.)*
- **Determinism — PARTIAL, and not guaranteed.** `par_iter().map().sum()` reorders float reductions (non-associative → non-deterministic results); side-effecting iterators run in unspecified order. Rayon gives *race-freedom*, not *result-determinism*. ([rayon docs / Shuttle guide](https://www.shuttle.dev/blog/2024/04/11/using-rayon-rust)) *(High confidence.)*
- **Adoption — VERY HIGH.** Rayon is the de-facto standard, widely used in production. This is the strongest evidence in the whole cluster: *low-ceremony, safety-only, opt-in* parallelism is what actually got adopted. *(High confidence.)*

**What rayon CAN'T do (the friction XL might attack).** Irregular/pointer-chasing parallelism — graphs, worklists, mutable shared structures, dynamic dependencies. The borrow checker's "one `&mut` xor many `&`" is too coarse for "these two threads touch *disjoint* nodes of the same graph." SPAA 2024's headline breakdown of parallel access patterns across their benchmarks: **~11% handled by safe Rust, ~60% required interior-unsafe abstractions + static checks, ~29% unsupported or needing high-overhead dynamic checks** (e.g. `Mutex`, atomics, or `unsafe`). Their verdict: for *any* irregular parallelism the programmer "must choose between unsafe code or high-overhead dynamic checks… as scary with Rust as with its predecessors." ([SPAA 2024 abstract/PDF](https://utoronto.scholaris.ca/bitstreams/8321e66b-89f1-4ff1-bfb0-dc0fdb17f2dd/download)) *(Numbers from the paper's abstract/search snippet; I could not re-extract them from the PDF body directly — moderate-high confidence on the exact percentages, high confidence on the qualitative split.)*

---

## 2. std threads + channels (`std::thread`, `scope`, `mpsc`)

**Mechanism.** Manual `spawn`; `thread::scope` (stabilized 1.63) lets threads borrow non-`'static` stack data safely; `mpsc`/`crossbeam` channels for message passing.

**Auto vs manual.** Fully manual: you choose thread count, decomposition, and communication. Compiler only enforces `Send` on moved data and lifetime soundness of scoped borrows.

**Four traps.** Annotation: none, but *decomposition burden* is total. Granularity: entirely the programmer's problem (raw threads have high spawn cost → you hand-build pools). Determinism: none provided; interleavings are yours to tame. Adoption: high for coarse/structured concurrency, but people reach for rayon/tokio for real workloads. *Verdict: this is the "manual" baseline XL is implicitly contrasted against — high skill required, and most code just… doesn't do it.* *(High confidence.)*

---

## 3. `Arc<Mutex<T>>` / shared mutable state

**Mechanism.** `Arc` = atomic refcount for shared ownership across threads; `Mutex`/`RwLock` for interior mutability; atomics for lock-free.

**Auto vs manual.** Safety auto-checked (`Arc<Mutex<T>>: Send+Sync` composes); *correctness of locking discipline is manual* — Rust prevents data races but **not deadlocks, not lost-update logic errors, not lock-ordering bugs**.

**Four traps.** Annotation: low. Granularity/overhead: this is the *cost* path — this is precisely the "high-overhead dynamic checks" of the SPAA 29%: runtime locking serializes and can erase the parallel win. Determinism: none. Adoption: ubiquitous but widely regarded as the "I gave up on static disjointness" fallback. *Verdict: the existence of this fallback is what makes irregular Rust parallelism possible at all — and its overhead is exactly the gap XL claims to close with static region-disjointness.* *(High confidence.)*

---

## 4. async/await + work-stealing executors (Tokio) — NOT parallelism

**Mechanism.** `async fn` compiles to state-machine `Future`s driven by an executor (Tokio). Multi-threaded executors *can* run tasks on a thread pool, so there's incidental parallelism, but the model is **concurrency** (interleaving many I/O-bound tasks), not CPU parallelism.

**Why it's a high bar / irrelevant to XL's claim.**
- **Function coloring.** `async` and sync are different "colors"; you can't call `async` from sync without an executor, and the split is viral. This is real and acknowledged even by defenders. ([Kobzol: async is about concurrency not performance](https://kobzol.github.io/rust/2025/01/15/async-rust-is-about-concurrency.html); [thecodedmessage: function colors are Rusty](https://www.thecodedmessage.com/posts/async-colors/)) There's a genuine counter-view that Rust's coloring is *deliberate/useful* (like `?`/`await` marking cancellation points), so treat "coloring = pure defect" as contested, not settled. *(High confidence it's contested.)*
- **Async ≠ multithreading.** A single-threaded runtime runs many futures with zero parallelism; the benefit is expressing concurrency cheaply, "performance is a second-order effect." ([Kobzol](https://kobzol.github.io/rust/2025/01/15/async-rust-is-about-concurrency.html))

**Four traps.** Annotation: high (viral `async`, `Send` bounds on futures, `Pin`, lifetime pain in async are notorious). Granularity: executor-managed, but tuned for I/O not compute. Determinism: none. Adoption: very high for servers/I/O. *Verdict: orthogonal to XL's pitch. XL should not claim to beat async — different problem (I/O concurrency vs data parallelism). Conflating them would be a rhetorical error.* *(High confidence.)*

---

## 5. Auto-parallelizing Rust (research) — thin, and telling

There is **no established auto-parallelizing Rust compiler**. Research notes ownership/`Send`/`Sync` *could* be a foundation for safe auto-parallelization, and structured-parallelism DSLs exist on top of rayon, but nobody ships "write sequential Rust, compiler parallelizes it." ([lib.rs concurrency](https://lib.rs/concurrency); [structured stream parallelism for Rust](https://www.sciencedirect.com/science/article/abs/pii/S2590118421000332)) The broader auto-parallelization field's own retrospective is brutal: decades of work, "only limited success," tools "overly strict, often failing to recognize opportunities," and killed as much by *economics and the granularity/cost-model problem* as by soundness. ([auto-par overview / Wikipedia parallel computing](https://en.wikipedia.org/wiki/Parallel_computing))

**The DPJ cautionary comparator (XL's near-twin).** DPJ had *exactly* XL's region+effect model — named regions partition the heap, per-method effect summaries, compiler proves non-interference, deterministic-by-default with explicit `_nd` escape. It was technically sound and performed well (matched/beat non-deterministic multithreaded Java). It got ~zero adoption. The documented reasons map straight onto XL's risk surface:
- **Annotation burden was the acknowledged central cost.** DPJ's own authors call out "the overhead of writing the DPJ type and effect annotations" as the price of determinism, and built **DPJizer** (an Eclipse effect-inference plugin) specifically because manual region/effect annotation was too heavy — and *even then* fell back to runtime techniques when "the annotation burden is not justified by the performance gains." ([DPJ Current Design](https://dpj.cs.illinois.edu/DPJ/Current_Design.html); [DPJizer](http://dpj.cs.illinois.edu/DPJ/DPJizer.html); [Cornell CS6120 review](https://www.cs.cornell.edu/courses/cs6120/2020fa/blog/parallel-java/))
- **It still didn't parallelize *for* you** — you wrote `foreach`/`cobegin` explicitly; the effect system *proved them safe*, it didn't *find* them. Same as rayon, but with a much heavier annotation tax.

*(High confidence on DPJ mechanism and the annotation-burden framing; the "~zero adoption" claim is from training knowledge — I found no adoption retrospective online, so treat adoption as inferred from the total absence of production use, not a cited failure post-mortem. Moderate confidence on cause-of-death specifics.)*

---

## The single most important lesson for XL

**Rayon already ate the easy 80% at ~one token of ceremony — so XL's automation has almost no room to win on the cases where automatic parallelization is *easy*, and must win precisely on the cases where it is *historically hardest*.** Break it down:

1. **The regular-parallelism win is already priced in.** `par_iter()` is safe, adopted, and competitive. XL saying "you write `map` and we parallelize it" saves the programmer *one token and one decision* over rayon. That is a marginal convenience, not a durable advantage — and it *removes the human's granularity decision*, which drops XL straight into **Trap 2 (cost model)**, the trap that actually killed classic auto-parallelization. Proving work *can* run parallel is the part XL's effects/regions do well; deciding it *should* (overhead vs benefit, per call site, data-size-dependent) is unsolved by effect systems and is where the field dies. Rayon's honest answer is "the human decides"; if XL removes the human, it owns a problem DPJ and every auto-parallelizer punted on.

2. **The only *big* practical win is the irregular 29–89% rayon can't do safely** — disjoint mutations of shared graphs/trees where the borrow checker is too coarse and Rust forces you to `unsafe` or pay `Mutex`/dynamic-check overhead (SPAA 2024). XL's regions *can* in principle express "these tasks touch disjoint sub-regions" statically, turning that overhead into a compile-time proof. **This is the genuine, defensible thesis.** But this is *also* exactly where DPJ's region/effect annotation burden exploded, because expressing "disjoint partitions of a recursive/irregular structure" is what needs region parameters, index-parameterized regions, and effect summaries threaded through every method — i.e. the ceremony that broke "write it normally" and needed a whole inference tool to paper over.

**So XL is caught in a vise, and the cluster names both jaws:** where parallelism is *easy to prove*, rayon already wins on ergonomics and adoption (Trap 1 solved, Trap 4 solved) and XL only inherits the cost-model problem (Trap 2). Where parallelism is *valuable to prove* (irregular, the thing rayon can't do safely), the static disjointness proofs demand exactly the annotation burden that made DPJ — a sound, performant, near-identical system — unadopted (Trap 1 returns with force).

**The falsifiable bar for XL:** it must show that its region/effect system proves *irregular* disjointness (graph/tree/worklist mutation) **safely, deterministically, AND at annotation cost low enough that ordinary code still looks sequential** — with the granularity decision handled by something better than "trust the human" (rayon) or "fall back to runtime checks" (DPJ). If XL can only auto-parallelize the regular `map`/`reduce` cases, it has reinvented rayon with a heavier type system and inherited auto-parallelization's cost-model curse. If it needs DPJ-level region annotations to reach the irregular cases, it has reinvented DPJ and will share its adoption fate. The advantage is real *only* in the narrow band where static region disjointness is both **expressible cheaply** and **decidable profitably** — and no prior system has been shown to hit that band. Skeptic's prior: **"sound but a dead end unless the annotation burden for irregular disjointness is driven near zero, which DPJ proved is the hard part."**

**Sources:**
- [SPAA 2024, "When Is Parallelism Fearless and Zero-Cost with Rust?"](https://dl.acm.org/doi/10.1145/3626183.3659966) · [author PDF](https://utoronto.scholaris.ca/bitstreams/8321e66b-89f1-4ff1-bfb0-dc0fdb17f2dd/download)
- [rayon repo](https://github.com/rayon-rs/rayon) · [Red Hat: how Rust makes Rayon magical](https://developers.redhat.com/blog/2021/04/30/how-rust-makes-rayons-data-parallelism-magical) · [rayon granularity discussion](https://github.com/rayon-rs/rayon/discussions/1134) · [Endignoux rayon optimization](https://gendignoux.com/blog/2024/11/18/rust-rayon-optimized.html)
- [Kobzol: async is about concurrency, not performance](https://kobzol.github.io/rust/2025/01/15/async-rust-is-about-concurrency.html) · [thecodedmessage: function colors are Rusty](https://www.thecodedmessage.com/posts/async-colors/)
- [DPJ Current Design](https://dpj.cs.illinois.edu/DPJ/Current_Design.html) · [DPJizer effect inference](http://dpj.cs.illinois.edu/DPJ/DPJizer.html) · [Cornell CS6120 DPJ review](https://www.cs.cornell.edu/courses/cs6120/2020fa/blog/parallel-java/)
- [tiny-tasks granularity tradeoff](https://arxiv.org/pdf/2202.11464) · [Wikipedia: parallel computing (auto-par history)](https://en.wikipedia.org/wiki/Parallel_computing)

Confidence: High on rayon mechanics/ergonomics/adoption, async framing, and the DPJ mechanism + annotation-burden framing. Moderate-high on the SPAA 11/60/29% split (from abstract, not re-verified in PDF body). Moderate on DPJ's precise cause-of-death and "~zero adoption" (inferred from absence of production use; no cited retrospective found).

## Prior-art survey: data-parallel-autovec (Futhark, ISPC, Halide, polyhedral)

# SURVEY CLUSTER: data-parallel-autovec — evidence and verdict for XL

Bottom line up front: every system in this cluster that actually beats hand-tuned C does so by **restricting the language to a domain where independence is a property of the operator, not a fact proven after-the-fact about arbitrary sequential code**. None of them auto-parallelize general-purpose code (a compiler, a server). XL's effect/region system is a *safety* oracle, not a *parallelism-availability* oracle or a *cost model* — and the historical record says the latter two are the walls, not safety.

---

## 1. Polyhedral auto-parallelization (Polly/LLVM, PLuTo, GCC Graphite, ICC auto-par)

**Mechanism.** Model affine loop nests over arrays as integer polyhedra; compute exact dependence polyhedra; find a schedule (via ILP, e.g. Feautrier/PLuTo algorithm) that exposes parallel loops + tiling + fusion. Fully automatic on the subset it accepts.

**Auto vs manual.** Maximally auto — *zero annotations*. This is the purest "write sequential, get parallel" system in the cluster, and that is precisely why its limits are the most instructive for XL.

**Measured wins.** On dense affine kernels (stencils, GEMM-like, Polybench) polyhedral tiling+parallelization delivers large speedups and can beat naive C by integer factors. Real and reproducible — *on that code class*.

**Four traps.**
- *Annotation (best in class):* none required. But the price is a hard gate: code must be a **Static Control Part (SCoP)** — affine loop bounds and subscripts, no data-dependent control flow, no pointer aliasing, no calls with unknown effects. A trivial edit (a `break`, an indirect index `a[b[i]]`, a pointer) silently drops the loop out of the model. So the "no annotation" benefit only exists inside a fragile island.
- *Granularity/cost model (the killer):* proving a loop *can* run parallel ≠ deciding it *should*. Polyhedral tools have weak, brittle profitability models; the schedule that minimizes dependence distance is not the one that maximizes wall-clock, and tiling sizes/parallel-vs-locality tradeoffs are hardware-specific.
- *Determinism:* not an issue here — affine transforms preserve semantics exactly; determinism is free. **Determinism was never what held polyhedral back.**
- *Adoption:* **This is the decisive datum for XL.** Polly (LLVM) and Graphite (GCC) have existed for ~15 years and remain **opt-in flags, never default at -O2/-O3** ([Polly docs](https://polly.llvm.org/docs/UsingPollyWithClang.html); [Polly homepage](https://polly.llvm.org/)). Documented reasons: narrow applicability (dense regular numerics only), inability to model sparse/irregular code, no production polyhedral optimizer supports **reductions** ([Polly reductions paper, arXiv:1505.07716](https://arxiv.org/pdf/1505.07716)), compile-time cost, and inconsistent/negative results on real programs. ICC's auto-parallelizer (`-parallel`) is the most mature and is still routinely ignored because it triggers rarely and unpredictably.

**Lesson from polyhedral:** the one system that truly needs no annotations pays with a razor-thin, fragile applicability domain and an unsolved profitability model — and after 15 years the compiler community's revealed preference is "off by default." Confidence: **high** (documented, longstanding).

---

## 2. The load-bearing negative result: limits of dependence analysis

**Evidence.** Campanoni et al., *Limits of Dependence Analysis for Automatic Parallelization* ([CPC 2015 PDF](https://users.cs.northwestern.edu/~simonec/files/Research/papers/HELIX_CPC_2015.pdf)). Finding: even with an **oracle (perfect) dependence analysis**, general-purpose SPEC programs expose only modest parallelism, and the dominant limiter is **true data dependences inherent in the algorithm**, not analysis imprecision.

**Why this matters directly to XL.** XL's effects+regions are, at best, a *precise disjointness oracle* — exactly the thing this paper hands the compiler for free and then measures. The result: precision is not the bottleneck for general-purpose code; the code is genuinely sequential. So XL's central mechanism attacks a barrier (aliasing uncertainty / safety) that is **not** the binding constraint on non-array code. Confidence: **high** on the qualitative conclusion; **medium** on exact magnitudes (the PDF's numeric figures didn't extract cleanly, so I won't quote specific speedup numbers).

---

## 3. NESL (Blelloch)

**Mechanism.** First-class **nested** data parallelism; the **flattening/vectorization** transform compiles nested `apply-to-all` into flat vector (segmented) operations. Crucially, NESL had a **formal work–depth cost model** — one of the very few languages where you can *reason about* parallel cost from the source.

**Auto vs manual.** You write parallel comprehensions explicitly (`{f(x) : x in A}`) — so parallelism is *expressed*, not *recovered from sequential code*. The **auto** part is the hard part: mapping arbitrary nesting onto flat SIMD/vector hardware.

**Wins.** Demonstrated good asymptotic parallelism on irregular algorithms (sparse matrix-vector, tree/graph algorithms) — notable because it handled *nested/irregular* structure, unlike polyhedral.

**Four traps.** Annotation: light but you must think in data-parallel combinators. Granularity: the **work–depth model is the standout contribution** — it makes cost *predictable*, a lesson XL should steal. Determinism: deterministic by construction. Adoption: **research vehicle, ~zero production use**; flattening produces asymptotically-optimal but constant-factor-poor code (bad locality, materializes large intermediates). Confidence: **high**.

---

## 4. Data Parallel Haskell (DPH)

**Mechanism.** NESL-style flattening/vectorisation embedded in GHC over parallel arrays, plus fusion to fix constant factors.

**Auto vs manual.** You opt into `[:e:]` parallel arrays; the vectoriser flattens nested to flat parallelism ([Haskell wiki](https://wiki.haskell.org/GHC/Data_Parallel_Haskell); [status report PDF, CMU](https://www.cs.cmu.edu/~damp/finalPapers/chakravarty.pdf)).

**Outcome — effectively a dead end.** DPH shipped only as add-on cabal packages for GHC ~7.x and was **shelved/abandoned**. Documented failure modes: could not mix vectorised and non-vectorised code in a module, required a **feature-deprived special Prelude**, and had **poor performance from missing optimisations**. The durable survivor is the **`vector` library** (flat, non-nested) — i.e., the *general* nested-parallel ambition died; the *narrow, regular* piece lived. That pattern repeats across the whole cluster.

**Four traps.** Annotation: moderate but viral (special Prelude). Granularity/cost: flattening's constant factors were the technical killer; fusion never fully tamed them. Determinism: free. Adoption: **failed**. Confidence: **high** that DPH-proper is abandoned; the "vector survived" framing is **medium-high**.

---

## 5. Futhark

**Mechanism.** Pure functional array language; parallelism expressed via **SOACs** (`map`/`reduce`/`scan`/`filter`); compiler does flattening + aggressive **fusion** + moderate flattening of nested parallelism, emitting GPU (CUDA/OpenCL) and multicore code ([PLDI'17 PDF](https://futhark-lang.org/publications/pldi17.pdf); [performance page](https://futhark-lang.org/performance.html)).

**Auto vs manual.** Parallelism is **explicitly expressed** in SOACs ("the user is required to use special constructs to inform the compiler where the parallelism is" — [Futhark book](https://futhark-book.readthedocs.io/en/latest/practical-matters.html)). Auto = mapping/fusing/scheduling to hardware. So: *not* "write sequential, get parallel" — it's "write in data-parallel combinators, get a great backend."

**Wins.** Matches or beats hand-written CUDA/OpenCL on regular benchmarks (Rodinia, FinPar).

**Real limits (directly relevant).** **No irregular/ragged arrays** — "Futhark does not support non-regular arrays, as they complicate size analysis a great deal," and a FinPar benchmark was *excluded* for having irregular parallelism. All intermediate sizes must be **symbolically pre-computable** (GPUs forbid dynamic allocation in-kernel). It is a **standalone DSL** you call via FFI — you rewrite kernels, you don't parallelize your existing program.

**Four traps.** Annotation: you rewrite in the DSL (heavy, but that *is* the language). Granularity: good, because whole-array ops are coarse and the backend owns scheduling. Determinism: deterministic by construction, and it does **not** cost peak perf here — because the domain is embarrassingly parallel, so determinism and speed don't conflict. Adoption: modest but real in niches; a genuine research-to-practice success *within array computing*. Confidence: **high**.

---

## 6. ISPC (Intel SPMD Program Compiler)

**Mechanism.** SPMD-on-SIMD: source *looks* serial (a "program instance"), but a **gang** of instances runs across SIMD lanes; `uniform`/`varying`, `foreach`, cross-lane ops ([Pharr & Mark, InPar'12 PDF](https://pharr.org/matt/assets/ispc.pdf); [llvm.org pub](https://llvm.org/pubs/2012-05-13-InPar-ispc.html)).

**Auto vs manual.** Subtle and important for XL: the *syntax* is sequential-looking, but the **parallelism model is explicit and opted-into** — you chose ISPC, you mark `uniform` vs `varying`, you write `foreach`. The compiler auto-vectorizes *within the declared SPMD model*; it does **not** recover parallelism from ordinary C.

**Wins.** 3× on 4-wide SSE, 5–6× on 8-wide AVX, without intrinsics — reliable and repeatable.

**Four traps.** Annotation: light and local (`uniform`/`varying`). Granularity: the gang maps 1:1 to SIMD width — the cost model is essentially "there is none needed," which is *why* it works. Determinism: yes. Adoption: **a real success** — shipped in Embree, used in AAA game engines and film rendering. Confidence: **high**.

**ISPC's lesson for XL:** you can get "looks sequential" ergonomics — but ISPC achieves it by making the *execution model* explicit and hardware-shaped, not by proving independence of arbitrary code. The ergonomic win came from a fixed cost model (SIMD width), not from a smarter analysis.

---

## 7. Halide

**Mechanism.** Decouple **algorithm** (pure functional over grids) from **schedule** (tiling/vectorize/parallelize/fuse) ([CACM'18](https://cacm.acm.org/magazines/2018/1/223877-halide/fulltext); [andrew.adams.pub PDF](https://andrew.adams.pub/halide_cacm.pdf)).

**Auto vs manual — the honest history.** The algorithm is auto-parallelizable *because it's pure and data-parallel by construction*, but the **schedule was originally MANUAL and expert-written** — and the manual schedule is where all the performance lives. Auto-scheduling came later: Mullapudi 2016 ([CMU](http://graphics.cs.cmu.edu/projects/halidesched/mullapudi16_halidesched.pdf)), then Adams et al. 2019 with a **learned cost model + tree search** ([PDF](https://halide-lang.org/papers/halide_autoscheduler_2019.pdf)) that finally matched/beat expert schedules.

**Wins.** Matches/beats hand-tuned SIMD image pipelines in a few lines; auto-scheduler competitive with experts.

**Four traps.** Annotation: algorithm is clean; getting perf historically required a whole second manual artifact (the schedule). Granularity/cost: **Halide is the clearest proof that the cost model is the actual problem** — the semantics were trivially parallel from day one, yet it took *years and machine learning* to automate the *scheduling* decision. Determinism: free. Adoption: **major success** — Google (Android camera/HDR+, YouTube), Adobe, Qualcomm. Confidence: **high**.

**Halide's lesson for XL — the sharpest one:** proving independence is the *easy* 5%. Halide had perfect independence knowledge for free (pure functions on grids) and *still* the entire research program was about the **cost/scheduling model** — and it needed learned search to crack it. XL's effects/regions give you the easy 5% (safety/independence). The 95% — granularity, tiling, locality, is-it-worth-it — is untouched by an effect system.

---

## What was AUTO vs MANUAL across the cluster (the pattern)

| System | Independence known because… | What's auto | What's manual/restricted |
|---|---|---|---|
| Polyhedral | affine model *proves* it | schedule+parallel+tile | must be a SCoP; no reductions; fragile |
| NESL/DPH | data-parallel operator | flattening to flat SIMD | constant factors; DPH abandoned |
| Futhark | SOAC operator | fusion+GPU codegen | rewrite in DSL; regular arrays only |
| ISPC | SPMD model declared | SIMD vectorization | you write ISPC; uniform/varying |
| Halide | pure grid functions | (later) schedule search | schedule was the whole game |

The common thread: **independence is guaranteed by the restricted operator/domain, not recovered by analysis of general code.** The two systems that *did* recover it from general code (polyhedral, classic auto-par) are the two that stayed off-by-default or failed.

---

## THE SINGLE MOST IMPORTANT LESSON FOR XL

**An effect/region system is a *safety-and-disjointness oracle*, and the history of this cluster proves that disjointness was never the binding constraint on delivering pervasive automatic parallelism. The three walls that actually decide success are: (1) is there enough independent work — which for general-purpose code is answered "no" even by an oracle ([HELIX limits result](https://users.cs.northwestern.edu/~simonec/files/Research/papers/HELIX_CPC_2015.pdf)); (2) the cost/granularity model — which Halide shows is the real research problem *even when independence is free*; (3) locality/constant factors — which killed flattening in NESL/DPH.**

Concretely for XL's "write sequential, get safe automatic parallelism" pitch:

- The **wins in this cluster only transfer to numeric/array kernels** (regular, affine, data-parallel). They do **not** transfer to a compiler or a server. XL running general-purpose code will hit the same wall polyhedral and classic auto-par hit — and that wall is *true dependences*, which no effect/region annotation can remove.
- XL's effects **remove the aliasing/safety uncertainty** that stops a compiler from *daring* to parallelize. That is a real and non-trivial improvement over C's may-alias fog — but it is necessary, not sufficient. It gets you to the *starting line* that Halide/Futhark reach by construction; it does **not** give you their cost model.
- Determinism is a **red herring as a cost.** Across this entire cluster, determinism was free and never the thing that lost performance — because these domains are embarrassingly parallel. Determinism only "costs peak perf" when you'd otherwise exploit nondeterministic reductions/racy relaxations; in the data-parallel domain that tradeoff barely bites. So XL should not expect a determinism-vs-speed tax *if it stays in the array domain* — and should not claim determinism as its differentiator, because everyone here already had it.
- **The realistic, defensible XL win is narrow, and it is the ISPC/Futhark shape, not the polyhedral shape:** give ordinary-looking code a *guaranteed-safe* path to `parallel_for`/reduce over regions the effect system proves disjoint, so that the programmer *opts in cheaply* and the compiler *never has to be conservative about aliasing*. That is a genuine ergonomic edge over Rust's `Send/Sync/Arc<Mutex>`/rayon ceremony **for the data-parallel subset**. It is **not** "your compiler/server gets auto-parallelized," and the moment XL markets it that way it is repeating the polyhedral and DPJ mistake: sound, elegant, and adopted by nobody, because the annotation/cost/availability economics don't close on general-purpose code.

Adoption verdict for the cluster, as XL's cautionary mirror: the systems that won (**ISPC, Halide, Futhark**) won by being **domain-restricted DSLs with a fixed or learned cost model**; the systems that tried to be **general and analysis-driven (polyhedral auto-par, classic auto-par, and by extension DPJ)** are off-by-default or dead. XL's effect+region system is technically closer to the second group. Its only route to the first group's success is to accept the same restriction — **be great at the data-parallel array/loop subset and stop promising the rest.**

Overall confidence: **high** on the structural lesson and per-system adoption verdicts; **medium** on precise speedup magnitudes (I deliberately avoided quoting numbers I couldn't ground). Where I leaned on training knowledge (NESL cost model, DPH's `vector`-survived framing, ISPC/Halide industrial deployments) I've flagged it; the searches corroborated the direction of each.

Sources: [Polly docs](https://polly.llvm.org/docs/UsingPollyWithClang.html) · [Polly reductions, arXiv:1505.07716](https://arxiv.org/pdf/1505.07716) · [Limits of Dependence Analysis, CPC'15](https://users.cs.northwestern.edu/~simonec/files/Research/papers/HELIX_CPC_2015.pdf) · [Futhark PLDI'17](https://futhark-lang.org/publications/pldi17.pdf) · [Futhark book, practical matters](https://futhark-book.readthedocs.io/en/latest/practical-matters.html) · [DPH status report](https://www.cs.cmu.edu/~damp/finalPapers/chakravarty.pdf) · [DPH HaskellWiki](https://wiki.haskell.org/GHC/Data_Parallel_Haskell) · [ISPC InPar'12](https://pharr.org/matt/assets/ispc.pdf) · [Halide CACM'18](https://andrew.adams.pub/halide_cacm.pdf) · [Halide auto-scheduler 2019](https://halide-lang.org/papers/halide_autoscheduler_2019.pdf) · [Halide auto-scheduler, Mullapudi 2016](http://graphics.cs.cmu.edu/projects/halidesched/mullapudi16_halidesched.pdf) · [Automatic parallelization, Wikipedia](https://en.wikipedia.org/wiki/Automatic_parallelization)

## Prior-art survey: dpj-effects (Deterministic Parallel Java + effect-system lineage)

I have enough grounded material. Returning the survey.

---

# SURVEY CLUSTER: dpj-effects — region/effect systems for parallelism vs XL's "write sequential, get safe auto-parallelism"

**Headline finding (high confidence):** The closest analog to XL, DPJ, **never attempted the thing XL wants**. DPJ did *not* auto-parallelize. The programmer wrote **explicit** `foreach`/`cobegin`; the region+effect system only *proved that the manually-marked parallelism was non-interfering and deterministic*. XL's stated ambition — compiler *finds* the parallelism in sequential-looking code *and decides to run it in parallel* — is strictly **harder** than DPJ, because it stacks DPJ's annotation-burden problem on top of the auto-parallelization cost-model problem that has failed for 40 years. DPJ, the sound and technically-admired system, got ~zero adoption, and **its own creator (Bocchino) publicly walked away from the full-effect approach in 2013** as too heavyweight. That is the central lesson of this cluster.

---

## 1. DPJ (Deterministic Parallel Java) — Bocchino & Adve, OOPSLA 2009 — the direct analog

**Mechanism (how it proved non-interference).** DPJ partitions the heap into a hierarchy of named **regions** and gives every method an **effect summary** (`reads`/`writes` a region-list; default = arbitrary effects for legacy compat). Four devices carry the weight:
- **Region parameters** on classes/methods (like generics: `class TreeNode<region P>`).
- **Region Path Lists (RPLs)** — hierarchical region names with `*` for "any suffix," enabling effects on nested structure and *disjointness* proofs (`P1 # P2`).
- **Index-parameterized arrays** — `[e]` RPL elements let the compiler prove that `array[i]` and `array[j]` name provably-distinct objects, so `foreach` over an array is non-interfering. Plus "distinctions from the left/right" for tree recursion.
- **Commutativity annotations** — `commuteswith` lets logically-commuting operations (e.g. a reduction `Adder.add`) run in parallel even though they read-write shared state.

Two disjoint effect sets ⇒ compiler proves the two tasks cannot interfere ⇒ deterministic. Formalized as "Core DPJ" with a soundness proof. (High confidence — read directly from the OOPSLA 2009 PDF.)

**AUTO vs MANUAL (the decisive point).** *Parallelism is entirely manual.* The programmer writes `foreach` (data parallel) and `cobegin` (task parallel). The compiler's job is **safety checking, not parallelism discovery**. It never converts a `for` loop to a parallel loop and never makes the should-I-parallelize granularity decision. (High confidence — Cornell CS6120 review + paper.)

**Measured wins (from the paper, 24-core commodity SMP, self-relative speedups at up to 22 cores):**
- Benchmarks: parallel merge sort, Monte Carlo (Java Grande), IDEA encryption, Barnes-Hut force computation, k-means, Collision Tree.
- Barnes-Hut **superlinear**; merge sort near-ideal; IDEA ~13.65× and others reaching ~23.9× / 23.33× at 22 cores; "moderate to excellent." In the 3 cases with a hand-written Java-threads version, DPJ matched it closely. Determinism overhead "negligible."
- **So: DPJ tied hand-tuned manual parallelism.** This mirrors the prior XL study's finding that safe rayon *ties* XL's parallel ceiling on a kernel. Neither system's value was ever raw per-kernel speed.

**Annotation burden (Table 3, the paper's own numbers):** fraction of source **lines changed** to add DPJ types/effects: 12.9%, 7.6%, 10.5%, 11.7%, **22.6%**, 1.0% → **~10.7% overall**, and up to 22.6% on the hardest case. Most of it is RPL type arguments and method effect summaries. They built **DPJizer** (an Eclipse inference tool) specifically because the burden was too high to write by hand. (High confidence.)

**Expressiveness limits the authors themselves admit:**
- `foreach` "only allows regular arrays."
- They **cannot express moving an element from position i to position j** in an array — i.e. **permutation/scatter** requires copying with "performance overhead." (Note: this is *exactly* the one kernel the prior XL study found to be XL's sole structural win over safe Rust. So even XL's best case is the case DPJ found hardest.)
- **Commutativity annotations are unchecked** — the compiler trusts the programmer's `commuteswith`; the paper itself calls propagating an unchecked annotation "highly unsatisfactory." Determinism-by-proof leaks into determinism-by-trust exactly where real reductions live.

**Four-trap verdict for DPJ:**
1. **Annotation burden — FAILS.** ~10.7% of lines, up to 22.6%; needed a whole inference tool; creator later called it "nontrivial." This breaks "write it normally."
2. **Granularity/cost model — N/A → PUNTED.** DPJ never decided *whether* to parallelize; the human did via `foreach`/`cobegin`. It sidestepped the trap XL must walk into.
3. **Determinism vs peak perf — MIXED.** Determinism cost was ~negligible in speed, but cost *expressiveness*: no in-place permutation, and the escape hatch (`commuteswith`) is unsound-by-trust. Determinism was paid in the coin of "what you can write," not "how fast it runs."
4. **Adoption — FAILED, decisively.** v1.0 released, then the line effectively ended. **Bocchino's 2013 retrospective ("High-Level Abstractions for Safe Parallelism," WoDet)** states these systems "suffer from potential barriers to adoption in that (1) they rely upon complex and/or restrictive features that may be difficult for programmers to understand and use; and (2) they impose a nontrivial annotation burden… The cost of these guarantees, however, can be high." He pivoted to **framework APIs with "far less user-side annotation"** — no effect annotations, no uniqueness annotations, just "understand and use the API." **The person who built the region+effect system concluded the region+effect system was the wrong layer.** (High confidence — quoted from the 2013 PDF.)

---

## 2. FX (Gifford & Lucassen, 1986–88) — the origin of type-and-effect for parallelism

**Mechanism (moderate confidence, training + historical):** FX introduced type-and-effect systems precisely to *identify side-effect-free / non-interfering expressions the compiler could safely reorder or run in parallel*. Effects were `read`/`write`/`alloc` over regions, much like DPJ (DPJ is FX's grandchild).
**Auto vs manual:** more automatic in *intent* than DPJ — the effect analysis was meant to *find* parallelizable expressions. **Traps:** it foundered on granularity/cost (proving something *can* run in parallel says nothing about whether it *should*) and never left the research setting. **Adoption: none.** The lasting contribution was the *effect-system idea itself*, not parallelization. Lesson: "prove independence" has been separable from "profitably parallelize" since the very first system.

---

## 3. Rust Send/Sync + rayon — the incumbent XL must beat

**Mechanism (high confidence):** `Send`/`Sync` are effect-like auto-traits; `&mut` exclusivity guarantees no data races in safe code (`nomicon/races.html`). This is a *type-level fact about sharing*, exactly analogous to a coarse effect system.
**Auto vs manual:** parallelism is **manual but nearly free ergonomically** — rayon's `.par_iter()` is a **one-token change** from `.iter()`, and the type system rejects it if the closure isn't `Send`/`Sync`. Channels/`Arc<Mutex>`/`spawn` are the heavier manual paths; async is concurrency, not parallelism (correctly flagged in the prompt).
**Traps:** (1) annotation ≈ zero for the common case (derive/auto). (2) granularity — rayon uses runtime **work-stealing** with a cost threshold, i.e. it solved the granularity problem *dynamically* rather than statically. (3) determinism — rayon is *not* guaranteed deterministic across reduction orders, but `par_iter` over disjoint indices is; the programmer chooses. (4) **adoption — massive.** This is the skeptic's hammer against XL: **the delta between "write sequential, get auto-parallel" and "change `iter` to `par_iter`" is one token, and rayon already ties XL's speed.** XL's automation win has to be worth *more than one token of typing* to the human, and its determinism guarantee has to be worth *more than rayon's opt-in determinism*. That is a very thin wedge.

---

## 4. Pony — reference capabilities + actors

**Mechanism (high confidence on caps, moderate on adoption):** six reference capabilities (`iso, trn, ref, val, box, tag`) statically guarantee data-race- and deadlock-freedom; only `iso` (uniquely mutable) and `val` (immutable) may cross actor boundaries.
**Auto vs manual:** **not auto-parallelism at all** — it's the actor model (manual concurrency). The runtime schedules actors across cores, but the programmer decomposes the program into actors and message sends.
**Traps:** (1) annotation/cognitive burden is **high** — refcaps are widely reported as a steep learning curve; the `consume`/`recover` discipline is subtle. (2) granularity = per-actor, chosen by the programmer. (3) determinism = **not guaranteed** (actor interleaving is nondeterministic; Pony guarantees race-freedom, not determinism). (4) **adoption — very limited.** The commercial backer (Sendence/Wallaroo Labs, fintech stream processing) wound down its Pony work around 2018–2019; Pony persists as a small niche/research community. Lesson for XL: *safety without determinism* (Pony) also didn't get adopted — safety alone isn't the adoption driver; ergonomics is.

---

## 5. Encore (Uppsala, Wrigstad et al.) & Koka (Leijen, MS Research) — briefly

**Encore (moderate confidence):** actors + the **Kappa** capability system + explicit *parallel combinators*. Parallelism is expressed via combinators/`async`, not discovered. Pure research vehicle; **no production adoption.** Confirms the pattern: capability/effect soundness ≠ uptake.

**Koka (high confidence on effects, high on non-parallelism claim):** **algebraic effect handlers** with row-typed effects (`div, exn, st, …`). Effects here track *control flow and side effects*, and support *concurrency* via handlers — **they are not an automatic-parallelization mechanism.** Algebraic effects are genuinely gaining traction (OCaml 5, research), but for *composable control*, not for "compiler parallelizes your loop." Relevance to XL: don't conflate "effect system" (Koka's success story) with "effect-driven auto-parallelism" (DPJ's dead end) — they are different value propositions that happen to share the word "effect."

---

## The four traps, scored across the cluster

| System | Annotation burden | Granularity/cost model | Determinism | Adoption |
|---|---|---|---|---|
| **DPJ** | **Fails** (~10.7%, ≤22.6% lines; needed DPJizer) | Punted to human (`foreach`) | Yes, but costs expressiveness + unsound `commuteswith` escape | **~0** (creator disavowed 2013) |
| FX | High | Unsolved | n/a | 0 (idea survived, system didn't) |
| Rust+rayon | ~0 (auto-traits) | Solved *dynamically* (work-stealing) | Opt-in | **Massive** |
| Pony | High (refcaps) | Per-actor (human) | **No** | Niche/declining |
| Encore | High | Combinators (human) | Partial | ~0 |
| Koka | Moderate | Not a parallelizer | n/a | Growing (as effects, not parallelism) |

**Cross-cutting evidence on auto-parallelization itself (high confidence):** the compiler literature is blunt — production compilers "usually fail to parallelize even simple sequential programs" because static dependence analysis is "extremely sensitive to syntactic variations" and breaks on pointers and complex control flow; auto-parallelization "has not succeeded much beyond the confines of the polyhedral model" (regular affine array loops). XL's region/effect system is, in effect, a bet that *ownership+regions supply the disjointness facts static analysis normally can't recover.* That is the real, honest thesis — but note it only helps where DPJ already worked (regular arrays / tree recursion) and stops exactly where DPJ stopped (irregular permutation, pointer soup, data-dependent access).

---

## THE SINGLE MOST IMPORTANT LESSON FOR XL

**Soundness was never the bottleneck — DPJ *had* the region+effect proof, and it still died. The bottleneck is that the human must (a) write the annotations and (b) still decide what/whether to parallelize, and DPJ made (a) too expensive while never touching (b). XL proposes to keep (a) (own/`&'r`/effect rows are DPJ's regions+effects under new names — mind the R4/D3 Rust-influence guard, but the *shape* is the same) and additionally take on (b), the granularity/cost-model decision that has defeated auto-parallelizers for four decades.**

Concretely, XL should treat this cluster as three hard gates it must pass and DPJ failed:

1. **The annotation must be nearly invisible, or inferred.** DPJ's 10.7% was already fatal, *and DPJ only annotated for a safety proof, not for parallelism discovery*. If XL's effect rows are load-bearing for auto-parallelism, they cannot cost more than DPJ's did. If they are inferred (as in DPJizer), then you are back to whole-program inference — brittle exactly where the prompt's D2 teachability directive says complexity must stay in the spec, not the program.

2. **You must beat *one token*, not beat *nothing*.** The realistic Rust baseline is `.par_iter()`, and it already *ties XL's speed* (prior study) and *auto-solves granularity* via work-stealing. "Write sequential, get parallel" only wins if the parallelism XL finds is parallelism the programmer *would not have marked* — and the auto-parallelization literature says the compiler mostly finds *less* than the human, not more, outside regular-array kernels.

3. **Determinism is a real, defensible differentiator — but a narrow one, and it fights performance at the edges.** DPJ's determinism was cheap in cycles but bought with expressiveness (no in-place permutation — *the very kernel that is XL's only structural win over safe Rust*) and a trust-based `commuteswith` hole. XL cannot have it both ways: guaranteed-deterministic *and* the fast disjoint-scatter *and* fully sound reductions.

**Skeptic's bottom line:** This cluster is the strongest available evidence that "write sequential, get safe automatic parallelism" via regions+effects is **sound but a dead end as a general-purpose value proposition.** The honest, survivable version of XL's claim is *not* "the compiler auto-parallelizes ordinary code better than Rust." It is the much narrower **shift-left/AI-codegen thesis already in the local note** (`notes/regions-effects-vs-safe-rust-2026-07-08.md`): XL makes the *naive* shape safe-to-parallelize on a *specific* family of kernels (disjoint scatter/gather, provable index bijections) where safe Rust's borrow checker cannot prove injectivity and rayon therefore can't be applied without `unsafe`. That is real (measured 1.13×–1.51× memory-bound), durable, and *narrow* — and it is a determinism/expressiveness story, not an "automatic parallelism beats Rust" story. Pitch that, cite DPJ as the cautionary precedent, and do not repeat DPJ's mistake of charging the programmer 10%+ of their lines for a proof they didn't ask for.

---

**Key local files:** `/Users/bytedance/Dev/xlang/optimizer-language-research/notes/regions-effects-vs-safe-rust-2026-07-08.md` (the prior adversarial study — scatter is XL's lone structural win, which is *exactly* DPJ's admitted expressiveness gap) and `/Users/bytedance/Dev/xlang/optimizer-language-research/notes/phase2-concurrency-findings.jsonl` (C004: Rust Send/Sync as the incumbent effect-ish baseline).

**Sources:**
- [Bocchino et al., "A Type and Effect System for Deterministic Parallel Java," OOPSLA 2009 (PDF)](https://rob-bocchino.net/Professional/Bocchino-OOPSLA-2009.pdf) — mechanism, Table 3 annotation counts (~10.7%, ≤22.6%), Fig 15 speedups, admitted permutation/`commuteswith` limits.
- [Bocchino, Mehnert, Aldrich, "High-Level Abstractions for Safe Parallelism," WoDet 2013 (PDF)](https://rob-bocchino.net/Professional/Bocchino-WoDet-2013.pdf) — the creator's retrospective: "nontrivial annotation burden," pivot to framework APIs with "far less user-side annotation."
- [Bocchino & Adve et al., "Parallel Programming Must Be Deterministic by Default," HotPar 2009](https://dpj.cs.illinois.edu/DPJ/Home_files/DPJ-HotPar-2009.html)
- [Cornell CS6120 critical review of DPJ (2020)](https://www.cs.cornell.edu/courses/cs6120/2020fa/blog/parallel-java/) — `foreach`/`cobegin` are manual; 10% code modification; trust-based correctness.
- [DPJizer inference tool](http://dpj.cs.illinois.edu/DPJ/DPJizer.html) — evidence the annotation burden needed tooling.
- [Pony reference-capability guarantees](https://tutorial.ponylang.io/reference-capabilities/guarantees.html) — data-race-free by types, actor concurrency (not auto-parallelism, no determinism).
- [Rust Nomicon: data races / Send-Sync](https://doc.rust-lang.org/nomicon/races.html) — the incumbent effect-ish baseline.
- ["Hunting the Mythical Automatic Parallelizing Compiler," The Next Platform](https://www.nextplatform.com/2015/04/01/hunting-the-mythical-automatic-parallelizing-compiler/) and [Wikipedia: Automatic parallelization](https://en.wikipedia.org/wiki/Automatic_parallelization) — decades-long underdelivery; static dependence analysis breaks on pointers, limited to polyhedral/affine loops.

**Confidence:** DPJ mechanism/numbers/retrospective — **high** (read from primary PDFs). Rust/rayon, Pony refcaps — **high**. FX, Encore internals, Koka-as-non-parallelizer — **moderate** (training + secondary search, primary sources not re-read this session). Adoption/decline narratives (Pony/Wallaroo, Encore) — **moderate**.
