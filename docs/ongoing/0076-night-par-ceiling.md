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

## Outcome

(Filled at closure.)
