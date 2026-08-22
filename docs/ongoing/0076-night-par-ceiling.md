# Batch 0076 — overnight par-ceiling expansion

Branch: `par/proof-derived-parallelism` (stacks after batch 0075; enters the
same merge review). Authority: owner chartering direction, 2026-08-22
(evening), verbatim:

> 我要去睡觉了,所以在收官以后我希望你能把目前还没有做的很完善的地方,或者
> 还不够好的地方继续完善。性能可以挖的方向继续挖。给自己设定一个合适的目标,
> 不要浪费今天晚上的时间。

refined by:

> 文章不用重写。然后par性能是最重要的,其次才是编译时间。超过rayon很好,但
> 我们的上限其实很高,我觉得还有可以更加挖掘的地方,要多发散,多一些想法。
> 你可以开一些agent去脑爆和研究,然后再尝试

## Method

Six-lens divergent ideation (read-only agents over the Dig 0-10 evidence)
produced ranked idea dossiers; the lead synthesized them into a falsifier
battery (Phase A: six throwaway probes, cheapest first, nothing ships) and
a landing queue (Phase B: only measurement-surviving ideas, one writer at a
time, each byte-identity-verified and protocol-measured). The governing
model under test: T_P ~ (T_seq + forks x 4.8ns) / ceiling(P) — the grid
shows no residual scheduler-quality term, so the levers are cheaper forks,
fewer forks, a truer ceiling, and new eligible workloads. Full plan:
lead scratch, night-plan of 2026-08-22.

## Standing invariants (every landing)

rayon wins zero oracle cells; no cell regresses outside the band; output
bytes identical at every worker count; no source-shape keying; no static
grain constants; gate green; the [PAR-1] schedule-unobservability envelope
untouched.

## Approval classes

No spec bytes planned. No protected conformance/compliance changes. No new
repository root entries.

**Repository gate at tip:** `make check` exits nonzero at the conformance step,
135/136 rules, the uncovered rule being CANDIDATE [PAR-1] itself. Disposition
(lead): the one-line protected conformance annotation prepared in the 0074
record (exact bytes recorded there) closes coverage; it is protected-class
material presented for owner approval at merge, and this branch requests merge
WITH that line in the packet rather than presenting a green repository gate.

**Flagged default-behavior change, landed — the merge packet must present
it.** An unset (or empty) `WF_WORKERS` in a `--par` binary now asks for one
lane per logical CPU instead of silently meaning sequential. The count is the
machine's logical CPUs, not the performance-core count this record first
proposed: probe A1 found the coarse configurations improve monotonically past
the four performance cores, and the measurement below shows no oracle cell
falls below its own sequential build at ten. `WF_WORKERS=0`, `=1`, and any
unparsable value keep today's meaning and start no pool.

**What changes and what does not — corrected after the batch audit.** No
*published bytes* change: [PAR-1] makes the worker count unobservable and every
cell published identical bytes at every count. But this record originally said
"no observable changes" flatly, and that is too wide. A program that runs
unconfigured now runs in the overlapped world, and that world reaches roughly a
third of the sequential build's recursion depth: on the checked-in `min_stack`
probe at an ordinary 8 MB stack the default build passes 400 000 frames while
the pool-on run fails by 185 000, and the death is a bare SIGSEGV. So a deep
recursion that completed at the old default can abort at the new one with no
source change and no configuration, and the owner is approving that. Two facts
bound it, both re-derived by the audit's refuters: [PAR-1]'s own text places
exhaustion of the resources an implementation spends on overlapping under
[SCOPE-3] and outside the rule's observables, and the default build already
dies the same way at its own higher ceiling — so the change moves a threshold
and introduces no new failure class. The blind spot found beside it, that no
test measured depth at the shipped default, is closed in the audit-repair entry
below.

## Executor log

(One line per landing at completion; probe verdicts recorded when the
battery reports.)

- **L1 — the shipped default is a pool.** `wf__par_requested_lanes` answers an
  absent setting with `wf__par_default_lanes()`: `hw.logicalcpu` on Darwin,
  `_SC_NPROCESSORS_ONLN` elsewhere, clamped to the existing 64-lane ceiling,
  and 0 on a machine that reports fewer than two. Dig 7's bootstrap follows
  without its own rule because it asks through the same function.
  **No-pessimization check at the new default** (M4, 10 logical / 4P+6E;
  `WF_WORKERS` genuinely unset; interleaved min-of-7 against each
  configuration's own sequential build; corporate agents active, load average
  3.64 before and 4.85 after): every one of the twelve oracle configurations is
  at or above its sequential build — `bal_d8_w16` 1.11x, `bal_d8_w64` 2.08x,
  `bal_d8_w192` 3.70x, `bal_d10_w16` 2.12x, `bal_d10_w64` 3.34x, `bal_d10_w192`
  4.71x, `bal_d12_w16` 3.23x, `bal_d12_w64` 4.12x, `bal_d12_w192` 5.14x,
  `skew_d16_w16` 2.72x, `skew_d16_w64` 4.28x, `skew_d16_w192` 6.22x. Every
  configuration's default run published the bytes its sequential build
  published (SHA-256 prefixes equal, twelve of twelve).
  **What the default costs the fine half, recorded rather than smoothed.** The
  same pass at four lanes reads `bal_d8_w16` 2.43x, `bal_d8_w64` 3.17x,
  `bal_d10_w16` 3.19x, `bal_d10_w64` 3.52x, `skew_d16_w16` 3.37x — so five of
  the twelve are faster at four lanes than at ten, and the finest,
  `bal_d8_w16`, gives up more than half its speedup (2.43x to 1.11x, the latter
  inside the protocol's unresolved band). The other seven are faster at ten,
  and the coarsest gains most (`skew_d16_w192` 4.69x to 6.22x). There is no
  worker count that is best for both halves and the runtime measures nothing,
  so the default is a choice between two known mistakes; ten keeps the floor at
  or above sequential everywhere and takes the coarse win. A focused min-of-11
  on the marginal cell (spreads 3.9%/6.7%/1.0%) reads `bal_d8_w16` sequential
  0.5677 s, default 0.5053 s, `WF_WORKERS=1` 0.6477 s — the default is 1.28x of
  the same binary's own opt-out, so the floor holds in both framings.
  **Test expectations changed, with the reason.** `run_with_stack` now names
  `WF_WORKERS=1` instead of removing the variable: absent is the default and
  would hand that fixture's deep side to a thief on an 8 MB worker stack, where
  the stack limit the case sets means nothing and it would pass without
  measuring anything. `the_layout_program_publishes_one_byte_sequence_at_every_worker_count`
  takes `WF_WORKERS=1` as its reference and compares the absent setting against
  it as one more run, because an absent reference would have compared parallel
  executions with each other. New case
  `an_absent_worker_setting_starts_the_pool_and_an_explicit_opt_out_does_not`
  reads the runtime's own grant counter: absent grants lanes, `0`/`1`/`abc`
  grant none, and all four publish identical bytes. It was checked against a
  reverted runtime and fails there, so it is not vacuous.

- **L2a — the index-split row in the oracle.** The bench grows a second
  workload family, `grid`: a Mandelbrot escape count over a 2^21-point index
  range at an orbit cap of 256, split by recursive halving, one row
  (`grid_d21_w256`). Its point is that the ordinary data-parallel shape — a
  counted loop over a range with an accumulator, which the judgment gives
  nothing — is eligible today when written as a recursion, with no compiler
  change: `whitefootc --par-ledger` reports `pair(tile, tile)  eligible`. The
  four parameters keep their meanings across both families, so `gen_wf.sh`,
  `build_wf.sh`, `run_bench.zsh`, `compare_outputs.zsh`, and `make_tables.zsh`
  carry it as one more row and every future rotation includes it. The twelve
  layout sources regenerate byte-identically after the generator change
  (checked by `cmp`, twelve of twelve).
  **Measurement** (interleaved min-of-7, load average 3.49 before and 3.39
  after, corporate agents active): Whitefoot sequential 0.5241 s; `--par` at
  1/2/4/8 lanes 0.5249 / 0.2747 / 0.1516 / 0.0927 s, that is 1.91x / 3.46x /
  5.65x of its own sequential build, and 0.0861 s (6.09x) at the new default.
  Rust sequential 0.4980 s; `rayon::join` at every split 0.2666 / 0.1500 /
  0.0954 s; the depth-5 cutoff 0.2574 / 0.1564 / 0.1050 s. Whitefoot against
  rayon at matched lanes reads 1.03x, 1.01x, 0.97x and best-against-best 0.90x
  — every one of them inside the protocol's 0.83x-1.20x band, so the honest
  statement is that **rayon resolves no win on this row**, not that Whitefoot
  wins it. Sequential parity 1.05x, also unresolved. All thirteen cells of the
  row published one byte sequence, `000000000033517d`, across both languages
  and every worker count.
  **Wiring verified end to end** by a three-round rotation of the whole grid:
  `compare_outputs.zsh` reports every run of every configuration identical in
  both languages and across them, and the grid row appears in
  `t_inventory`, `t_seq_parity`, `t_wf_par`, `t_rayon`, `t_rayoncut`, and
  `t_headline`. The authoritative N=18 rotation is Phase C's.
  **Not done, and named:** the rotation's twelve implementation cells still
  name every worker count explicitly, so the shipped default that L1 introduced
  is in no table. Adding a `wf_par/default` cell changes the protocol and the
  table shapes, which is a measurement-protocol decision rather than a landing.

- **L2b — the counted-loop split hint.** `--par-ledger` gains a third line
  form, `PAR hint`, in a new `semantic/loop_hint.rs`. It reports one line per
  counted loop whose body is claim-free, calls nothing whose row carries
  `external` or `blocks`, takes no exit that leaves the loop, and carries
  nothing across its iterations except accumulators combined under an
  exactly-associative operation: `+wrap`, `*wrap`, `iand`, `ior`, `ixor`,
  `imin`, `imax`, and boolean `and`/`or`/`xor`. The line names the operation,
  because that is what makes the advised rewrite safe to take.
  **No float operation is admitted, and that is the point.** `fadd.strict` is
  not associative, so a writer who split a float fold would publish different
  bytes at a different worker count. Denial fixture
  `a_counted_loop_reducing_under_a_float_operation_is_told_nothing` in
  `driver.rs` pins it, and pins the identical loop over an integer accumulator
  getting the line, so the silence is attributable to the operation. A second
  denial fixture pins a counted loop that writes into a buffer: the split of a
  parallel map writes two index ranges of one buffer, condition 2 reads those
  as one place, and advising a rewrite the judgment then refuses is worse than
  silence.
  **Zero semantic change, verified rather than asserted.** The ledger of all
  **698** sources of the Dig 9 sweep corpus (`tests/programs`,
  `tests/conformance/cases`, `tests/codegen`, all of `research/`) was taken
  with the pre-hint binary and with this one: with the `PAR hint` lines
  filtered out, every ledger is byte-identical — no verdict changed anywhere.
  The gate is green before and after.
  **Where it fires, reported honestly.** Two lines, both in one conformance
  case, `ent3-pos-s11-counted-range-run.wf`, on its two `+wrap` reduction
  loops; its third loop carries a `break` and is correctly refused. **Zero
  lines in all 25 `tests/programs/` sources and all 13 bench sources.** That is
  a finding, not a defect: every counted loop in the corpus is either a map
  into a buffer (`byte_string`, `growable_vec`, `dir_walk`, `wfgrep`,
  `raw_deflate_boundary`), a loop whose callee traps (`bs_push`, `vec_push`),
  or a sequential recurrence whose accumulator is read several times per
  iteration (`sha256_abc`'s compression rounds). Each is correctly refused for
  its own reason, and together they say that the loop-carried blocker is not
  only the judgment's statement-pair shape but what real counted loops do.
  **Boundary, deliberate:** only the `for` form is considered. A bare `loop`
  has no index range, so the advice would not mean anything about one, and
  reading a range out of a hand-written counter would be exactly the
  source-shape keying this project forbids. It is also unreachable in practice:
  a manual counter is itself carried state that is read several times per
  iteration, so the carried-state rule refuses every such loop anyway —
  including both of `mandelbrot_grid.wf`'s. A writer who wants the advice
  writes the counted form, which is the form that carries the fact.
  **Two smaller things a reviewer will notice.** The ledger used to be
  suppressed entirely when no function had an analyzed pair, so a program of
  nothing but loops produced no output at all; that gate now also asks for
  hints. And a provably zero-trip range (`for @empty i in 4_u64..4_u64`) still
  gets a line — noise, not error, and suppressing it would mean reading
  constants for no safety gain.

- **L3 — two corrections and one anomaly, all re-measured here rather than
  forwarded.**
  **The Dig 6 rig's byte identity, narrowed in the 0075 record.** Reproduced by
  linking one module three ways: `a/prog` and `b/prog` are byte-equal, and
  `a/prog2` differs in 429 bytes — `LC_UUID` at offset 1528 plus the ad-hoc
  code signature that hashes it — because `clang` derives the UUID from the
  output file's name. `otool -tV` minus its filename header is identical in
  every direction. The claim that carries the rig's meaning is text identity;
  the recorded byte identity held only because the rebuilds reused the oracle's
  file names.
  **The q4 over-forking arithmetic is withdrawn.** The night battery's A4 probe
  found q4's claims are mostly refused; re-measured here with an independent
  claim counter on a scratch copy of the runtime, q4 attempts exactly
  **65,548,383** claims — deterministic, and matching the derived figure — of
  which **6.59-6.67 M are granted at `WF_WORKERS=4` (89.9% refused)** and
  **11.31-11.38 M at `W=8` (82.7% refused)**, over three runs each. The refusal
  is `wf__par_claim` finding the lane's 64-slot free list exhausted. So the
  headline "q4 pays 65.5 M x 4.8 ns = 315 ms of protocol = 69% of T_seq" prices
  forks that were never published and is withdrawn; the published-fork cost is
  an order of magnitude smaller, and q4 is already self-limiting rather than
  over-forked. Grant counts vary with worker count and between runs because
  refusal is schedule-dependent — my `W=8` reading is 11.3 M against the
  battery's 7.6 M, which is the same phenomenon and a caution against quoting
  any single grant count as a constant. (Note for the next reader: this is
  *not* the `wf__par_grants` counter the gate-integrity test reads. That one
  counts **steals** — 0.34 M at `W=4`, 0.84 M at `W=8` on q4 — and is a
  different statistic from claim grants.)
  **Darwin QoS priority inheritance, recorded so the next E-core experiment
  does not lose a night to it.** A thread that self-sets
  `QOS_CLASS_BACKGROUND` runs at performance-core speed whenever a higher-QoS
  thread is blocked joining it, because Darwin propagates the joiner's priority
  to it. Measured three ways on the A5 probe (`bal_d12_w192`, reps=200, load
  average 5.16): worker self-sets background and the main thread joins it at
  the default QoS, **0.156-0.182 s**; the same worker with the whole process
  put in Darwin background so the joiner has nothing to lend, **0.463-0.467 s**;
  the measured thread self-sets background with nobody joining it,
  **0.465-0.478 s**. The two configurations without an inheriting joiner agree
  to 2%, and the one with a joiner is 2.9x faster. **Consequence:** A5's
  "ALL-BACKGROUND E-only discriminator" sweep measured performance cores
  wearing a background label, not efficiency cores, and its ceiling column
  should not be read as an E-core bound. An experiment that wants E cores must
  keep the observer off the join — poll a flag, or put the observer at the same
  QoS.

- **Verification over the four compiler-affecting landings.**
  `make -C compiler check` exit 0 at the tip and before the first landing.
  **No emitted code moved for the diagnostic**: the 66 modules the corpus and
  the bench emit under both the default and `--par` compilations are
  byte-identical between the pre-hint binary and the tip (5 of the 38 sources
  are not standalone-compilable and emit nothing in either mode, so the count
  is 33 sources x 2 modes). **Byte identity
  across the whole grid at the tip**: a three-round rotation of all thirteen
  configurations, 195 Whitefoot outputs and 273 Rust outputs compared by `cmp`,
  reports every run of every configuration identical within each language and
  across them. **rayon resolves no win on any cell**: every best-Whitefoot
  against best-Rust ratio in `t_headline` is inside the 0.83x-1.20x band and
  marked unresolved, on the twelve layout configurations and on the new grid
  row alike (load average 3.46, corporate agents active). No spec bytes, no
  protected conformance or compliance change, and no new repository root entry
  in any of the four commits. This section was written at `1e103492`;
  `b87d20bb` landed afterwards as a fifth change and touches no `compiler/`
  path, so it moves none of the results above. The audit-repair commit that
  follows it does touch `compiler/`, and carries its own verification in the
  audit-repair entry below.

- **Protocol amendment and the campaign's authoritative rotation (lead
  decision, C1).** The measured grid gains a `default` worker cell for both
  languages, because every cell measured so far names a worker count and the
  number an unconfigured program actually gets appeared in no table.
  `wf_par/default` runs the `--par` binary with `WF_WORKERS` genuinely unset
  (the harness clears it from its own environment first); `rs_rayon/default`
  takes a new `default` argument that skips `ThreadPoolBuilder` entirely so the
  work lands on rayon's own global pool. Both resolve to 10 threads of
  execution here. `cells.awk` and `compare_outputs.zsh` needed no change and
  were left alone — verified by running, not by reading: the awk keys the
  threads column as an opaque string and the comparison globs `out/${tag}__wf_*`
  and `__rs_*`, so the new cells joined both automatically.
  **The rotation.** Full protocol at HEAD `1e103492`, N = 18, 13 configurations
  x 14 cells = 182 cells per round, **3,276 runs, every one exit 0**. Byte
  comparison over the real published bytes of every run: **1,404 Whitefoot and
  1,872 Rust outputs, every run of every configuration identical within each
  language and across them.** Snapshot landed as `bench/baseline-20260822/`;
  `bench/baseline/` is untouched and retained as the 2026-08-21 record.
  **Load, stated honestly.** The pass ran 04:14-04:36 with corporate agents
  (Defender `wdavdaemon`, `epsext`, `netext`, CorpLink) active throughout.
  Per-round one-minute load averages: 2.68, 6.23, 6.10, 4.27, 5.65, 6.26, 7.12,
  4.79, 4.12, 4.71, 4.91, 4.80, 4.22, 8.44, 7.45, 7.77, 7.48, 7.46 — rounds
  14-18 under the heaviest contention. All 182 cells exceed 20% spread and the
  mean is 112%, which is why only minima are quoted; the min-of-18 numbers
  reproduce a load-2.9 one-round probe to within a few percent.
  **WF against rayon at matched worker counts** (`wf_par/N` vs `rs_rayon/N`,
  N in 2/4/8, 39 cells): **14 WF wins outside the band, 25 parity, 0 losses.**
  Widest is `bal_d8_w16`/8 at 0.48x (0.4676 s vs 0.9710 s).
  **At the defaults** (13 cells): **11 WF wins, 2 parity, 0 losses.** Widest is
  `bal_d8_w16` at 0.19x (0.5070 s vs 2.6388 s); the two parity cells are
  `bal_d12_w192` 0.89x and `grid_d21_w256` 1.02x. The comparison is untuned WF
  against untuned idiomatic rayon, and `rs_cut` — a hand-picked depth-5 cutoff —
  is a tuning choice deliberately excluded from it.
  **Per-family best cells.** `bal`: `bal_d12_w192`/default 0.1140 s, 5.21x over
  its own sequential. `skew`: `skew_d16_w192`/default 0.1239 s, 6.32x. `grid`:
  `grid_d21_w256`/default 0.0788 s, 6.57x, against rayon's default 0.0775 s —
  1.02x, parity, so the index-split family scales like the data-structure
  families and neither language wins it. `wf_par/default` is the best Whitefoot
  cell on 8 of 13 configurations. In `t_headline`, best-WF against best-Rust is
  unresolved on 12 of 13 and resolves only on `bal_d8_w64` (0.83x, WF faster);
  **there is still no configuration where Rust's best resolves faster than
  Whitefoot's best.**
  **NAMED RISK, found by this pass — the opt-in cost regressed on `bal`, and it
  is not load noise.** `w1/seq` is now **1.14x-1.30x on all nine `bal`
  configurations and monotone in the words parameter** (w16 1.14-1.15x, w64
  1.22x, w192 1.29-1.30x). The 2026-08-21 baseline measured 0.85x-1.04x on the
  same cells. `skew` is unchanged (0.77-0.88x, was 0.72-0.78x) and `grid` is
  1.00x. The absolute `wf_seq` minima barely moved (`bal_d12_w192` 0.6515 ->
  0.5934) while `w1` rose (0.6077 -> 0.7634, +26%), so it is the `--par` build's
  pool-off path that got slower, not the machine. A clean monotone dependence on
  a workload parameter across three tree depths is not what contention produces.
  Not bisected when this paragraph was written. **It has since been bisected by
  the batch audit and the cause is named: L1 (`62e30831`), not the counted-loop
  landing this paragraph first pointed at.** `36963401` is excluded by byte
  identity — the 66 modules above are identical across it, and the `--par`
  executables of `bal_d12_w192`, `bal_d8_w16` and `skew_d16_w192` link
  byte-identically between `165f8b5e` and the tip once the output basename is
  held equal (`clang` derives `LC_UUID` from it). Between Dig 7 (`d4e674c3`)
  and the tip the twelve oracle sources emit 24/24 byte-identical modules and
  the only change to `par_runtime.c` is `62e30831`. The mechanism is the
  code-placement sensitivity Dig 3 attributed and Dig 7 measured at 1.19x on a
  layout program: one LLVM module linked with the *pre-L1* runtime reads
  0.5994 s, with the tip runtime 0.7769 s, and with the pre-L1 runtime and the
  two `clang` inputs reversed 0.7773 s. So it is where the linker put the code
  and not work the runtime does — at `WF_WORKERS=1` the new
  `wf__par_default_lanes` is never reached, and the binaries differ by 144
  bytes. See the invariant-breach entry below.
  **Consequence for the recorded per-fork number:** 0075's
  formula takes `w1/4` as the ideal, and an inflated `w1` makes that ideal
  unreachable — at `bal_d12_w192`/4 it now yields **-9.44 ns/fork**, which
  measures the regression rather than fork cost. Against `wf_seq/4`, the
  like-for-like baseline rayon is already scored on, the same cell gives
  **+5.39 ns/fork against rayon's +5.13 ns/fork** — the two are at parity and
  the fork path itself did not regress.
  No spec bytes, no protected conformance or compliance change, and no new
  repository root entry.

- **The `w1` regression is a breach of this batch's own standing invariant,
  and is recorded as one.** "No cell regresses outside the band" is stated at
  the head of this record for every landing, and the night plan states it
  again; L1 broke it on three cells. `wf_par/1` minima, `bench/baseline/`
  against `bench/baseline-20260822/`: `bal_d8_w192` 0.6133 -> 0.7768 s
  (**1.267x**), `bal_d10_w192` 0.6286 -> 0.7787 s (**1.239x**), `bal_d12_w192`
  0.6077 -> 0.7634 s (**1.256x**). The other six `bal` cells run 1.099x-1.199x,
  inside the band; `skew` is 0.93x-1.01x and `grid` is 1.00x. The attribution is
  the one stated above: L1's runtime edit re-rolled the link layout of every
  `--par` binary, and byte-identity sweeps exclude every other commit in range.
  **What the breach does not touch.** Every `W >= 2` cell and every
  cross-language comparison are unaffected. The `wf_seq` minima barely moved,
  the matched-worker and default win counts above stand, and rayon still
  resolves faster on no cell. The breach is confined to the opt-out path of a
  `--par` build at one worker.
  **Why the landing's own evidence could not see it.** L1's no-pessimization
  check compares the new default against the *same tip's* sequential build and
  against four lanes. It takes no before/after on any cell, so a placement
  regression introduced by that same commit was invisible to it by
  construction. A landing that changes the size of a `--par` binary needs a
  before/after on the opt-out cells, and this one did not take one.
  **Standing item, unlocated — no code fix attempted.** The cause is link
  layout, which Dig 7 established re-rolls on any size change to a `--par`
  binary and which nothing in the tree locates or pins. It stays open as this
  campaign's one unlocated stall. It also supersedes Dig 7's headline —
  "`--par` at one worker now reads 1.00x-1.01x on all twelve configurations"
  (`0075:727`) — which nine of twelve configurations no longer meet.

- **Adversarial batch audit of 0075 and 0076, and the repairs it produced
  (2026-08-22).** Six finder lenses — hygiene and drift, protected classes and
  the approval surface, relevance and attribution, soundness of the semantic
  changes, record-against-tree truthfulness, and gate and test integrity —
  raised 57 findings over the range `8a41dbf5..b87d20bb`. Six refuter passes
  re-derived every one of them independently, in their own clones and with
  their own builds.
  **What the refuters did to them.** Three findings were refuted outright: that
  both records are defective for carrying an unfilled `## Outcome` (that is the
  expected state of a live record, and this audit is a precondition of filling
  it); that `spec-digest-sync` will fail after the rebase (a refuter three-way
  merged the branch with `main` and ran the real target — exit 0); and that a
  Dig 10 status line is stale (it labels its own log entry correctly). A fourth
  was refuted as stated and re-scoped:
  `handing_calls_out_keeps_the_sequential_recursion_depth` is not "a tautology
  under its own name", since keeping the sequential depth is exactly what it
  still asserts — what survives is its 2.82 s cost. Several sub-claims fell
  too, the load-bearing one being the assertion that gap-hunt F7 had healed; it
  has not, and it is recorded as open below. Four findings raised as CRITICAL
  were downgraded to MAJOR on evidence, and one — the `probes/README.md` drift
  — was *upgraded*, the finder having reported one stale string where four
  load-bearing paragraphs were false. One CRITICAL survived refutation, and it
  is not this batch's: the branch's CANDIDATE spec is numbered v0.34 while
  `main` has since ACTIVATED its own v0.34, which a refuter reproduced against
  the archive-integrity target itself.
  **Repaired in this commit.** The invariant breach and its attribution above,
  with 0075's superseded Dig 7 headline corrected in place; the false safety
  claim in the counted-loop hint, where `CalleeFacts::admits` ignored the
  callee's `writes` row so a `fadd.strict` fold one call frame away could be
  reported as a `+wrap` reduction — one root cause closing three findings, now
  refused and pinned by
  `a_counted_loop_whose_callee_writes_carried_state_is_told_nothing`; the
  recursion-depth blind spot at the shipped default, closed by
  `the_shipped_default_keeps_a_deep_recursion`; this record's over-wide "no
  observable changes"; four stale durable documents (`probes/README.md`,
  `DESIGN.md`, `RESULTS.md` including its pre-Dig-8 ledger transcript,
  `bench/PROTOCOL.md`) superseded in place; the gap-hunt verdict table given
  dispositions; and the number corrections — 7 of 13 to 8, 1.26x to 1.28x, "10
  corpus sources" to 5 sources over 10 emissions, and the four-landings
  phrasing.
  **Verification of the repair.** `make -C compiler check` exit 0 before and
  after. The hint change is diagnostic-only and moves nothing: a sweep of all
  698 `.wf` sources outside `archive/` under `--par-ledger --par --emit-llvm`
  is **byte-identical before and after** — every module hash, every exit
  status, and every ledger line, including the two `PAR hint` lines the corpus
  emits. The new fixture was confirmed non-vacuous by removing the guard and
  watching it fail with exactly the defective line. The new depth case was
  sized from measurement rather than guessed: at an 8 MB limit the default
  reaches past 160 000 frames and fails by 180 000, and 60 000 passed 20 runs
  of 20.
  **Confirmed by refuters and NOT repaired here, so the packet must carry
  them.** (1) The spec version collision, which needs the candidate renumbered
  to v0.35 and re-derived on `main`'s v0.34 bytes; it is spec-class work and it
  moves the 0074 amendment recipe's digest, its META-5 delta and the rule
  count. (2) Two gate-integrity tests that are nondeterministic under load
  because `wf__par_grants` now counts steals rather than lane grants —
  `the_runtime_replaces_the_modules_weak_refusal` and
  `an_absent_worker_setting_starts_the_pool_and_an_explicit_opt_out_does_not`,
  the second flaky even on an idle machine and admitting an injected live
  defect a quarter of the time under load. Both are protected gate-integrity
  material, and 0075's recorded margin for the first is withdrawn there. (3)
  `docs/current-plan.md`, still `PROPOSED` and contradicted by L1; the lead
  authors it separately. (4) gap-hunt F7 — open, low, untouched in range. (5)
  The proportionality objection to `loop_hint` itself: 565 lines of permanent
  compiler surface for two advisory lines across 537 sources. That is an owner
  call rather than a defect, and it is carried here rather than argued away.

## Outcome

(Filled at closure.)
