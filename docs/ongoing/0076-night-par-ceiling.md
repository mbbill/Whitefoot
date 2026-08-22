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

**Flagged default-behavior change, landed — the merge packet must present
it.** An unset (or empty) `WF_WORKERS` in a `--par` binary now asks for one
lane per logical CPU instead of silently meaning sequential. The count is the
machine's logical CPUs, not the performance-core count this record first
proposed: probe A1 found the coarse configurations improve monotonically past
the four performance cores, and the measurement below shows no oracle cell
falls below its own sequential build at ten. `WF_WORKERS=0`, `=1`, and any
unparsable value keep today's meaning and start no pool. No observable
changes: [PAR-1] makes the worker count unobservable and every cell published
identical bytes.

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
  0.5677 s, default 0.5053 s, `WF_WORKERS=1` 0.6477 s — the default is 1.26x of
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

## Outcome

(Filled at closure.)
