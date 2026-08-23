# Breaking proof-derived parallelism v1 with realistic layout shapes

The realistic-shape gap hunt of 2026-08-21, run against this branch
(`par/proof-derived-parallelism`) at the close of batch 0074. The worktree was
read only throughout; the hunt wrote no file in it.

**Where the probes are.** The nine most load-bearing probe sources —
`min_stack.wf`, `bt_skew.tmpl`, `p1a.wf`, `p1b.wf`, `q4.wf`, `bt.wf`, `p6.wf`,
`p7_dyn.wf`, `zero_elig.wf` — are landed beside this document in `probes/`, and
every probe named below is cited by its bare filename. The remaining sources
(`base.wf`, `p4.wf`, `p5.wf`, `p5b.wf`, `scale.wf`, the generated
`ms_*.wf` and `bt_skew_*.wf` sweeps) and all the IR, binaries, and run logs stay
in the research scratch area outside the repository until a dig promotes them;
they are not durable, and a claim below that rests only on one of them is
reproducible from the commands quoted, not from a checked-in artifact.
`./wfc.sh` in the commands below wraps `compiler/target/release/whitefootc`.

Machine: Apple M4, 10 cores (4P+6E), macOS 26.5.2, main stack 8176 KB. A neighbour suite used ~1 core throughout. Timings are min-of-N interleaved (N=5 or 7, `timeit.zsh`); ratios inside 1.20x marked `(u)` and claim nothing. Exit status read directly from the process, never through a pipeline.

## Verdict summary

The verdicts below are the hunt's own, taken at the close of batch 0074. The
right-hand column was added on 2026-08-22 by the 0076/0077 batch audit, which
found the table still reading as present tense after batch 0076 had worked
through most of it. Where a disposition names a batch record, that record
carries the evidence and nothing here restates it.

| # | Finding | Severity | Verdict (0074) | Disposition (2026-08-22) |
|---|---|---|---|---|
| F1 | `--par` shrinks max recursion depth ~4x; overflow is a bare SIGSEGV/SIGBUS | **CRITICAL** | BUG | **Partly closed**, 0076 Dig 1: the frame moved to the lane and the pool-*off* ceiling matches the sequential build. The pool-on ceiling is still roughly a third of it, and since 0077's L1 the pool is on by default — see that record's flagged default-behavior entry. |
| F2 | Hand-out is unconditional; no grain control; fine folds up to 48.6x slower | **CRITICAL** (perf) | GAP | **Partly closed**, 0076 Dig 2 stage 1: work stealing turned the 48.6x cell into 1.99x *faster* than one lane. Stage 2 (C6) was built and measured and did **not** land. |
| F3 | Adjacency is brittle and its loss is silent | HIGH | GAP | **Closed**, 0076 Dig 8 (`974d5513`): an interposed statement is judged inside a window instead of ending the enumeration. |
| F6 | Dynamically sized per-node allocation can never actualize | HIGH (design limit) | WORKS-AS-DESIGNED | Unchanged. |
| F4 | Ledger reports pairs, never runs, never what was handed out | MEDIUM | GAP | Partly addressed — Dig 8 added `PAR chain` lines for runs — but no batch dispositioned it, so read it as open. |
| F5 | Allocation-heavy lanes cost ~3x more per hand-out than compute lanes | MEDIUM | GAP | **Dissolved** per 0076 Dig 3/4. |
| F7 | Ledger cites a filename that does not exist | LOW | BUG | **Open, untouched.** Re-verified at the branch tip on 2026-08-22: a bare relative path reports correctly, while `./p1b.wf` and any absolute path both report `input0.wf`. No commit in 0076 or 0077 touched `logical_path`. |

**Correctness never broke.** Across 12 probe programs and worker counts 1, 2, 4, 8, 64, 65 plus every malformed setting, output was byte-identical to the sequential build in every run. No deadlock, no livelock, no wrong bytes, no hang. Every finding is a resource, performance, or reporting defect.

## Ledger-vs-IR agreement, per probe

Eligible-pair count from `--par-ledger` vs `try_fork` count in the `--par --emit-llvm` module.

| probe | ledger eligible | IR `try_fork` | agree |
|---|---|---|---|
| `base.wf` | 2 | 2 | yes |
| `p1a.wf` | 1 | 1 | yes |
| `p1b.wf` | 3 | 3 | yes |
| `q4.wf` | 6 | 6 | yes |
| `bt.wf` | 3 | 3 | yes |
| `p4.wf` | 3 | 3 | yes |
| `p5.wf` | 11 | 11 | yes |
| `p5b.wf` | 4 | 3 | **NO — F4b** |
| `p6.wf` | 3 | 3 | yes |
| `p7_dyn.wf` | 0 | 0 | yes |
| `zero_elig.wf` | 0 | 0 | yes |
| `scale.wf` | 200 | 200 | yes |

One disagreement in twelve, isolated in F4b.

---

# F1 — CRITICAL — `--par` shrinks maximum recursion depth ~4x; the overflow is a bare signal

**Verdict: BUG.** Source: `min_stack.wf` (minimal), `bt_skew.tmpl` / `bt_skew_*.wf` (realistic).

A recursion that succeeds without `--par` dies with an unhandled SIGSEGV when compiled with `--par` **and the pool disabled**. Lanes on move the crash to a worker and make it SIGBUS.

**Minimal repro** — `min_stack.wf`, one deep recursion, one eligible pair:

```
fn spine(depth: own u64, v: own f64) -> result: own f64 pure {
  let done = ieq(depth, 0_u64);
  if done { return v; }
  let next = depth -wrap 1_u64;
  let a = spine(depth: next, v: v);
  let b = leafval(v: v);
  return fadd.strict(a, b);
}
```

**Commands**

```
sed "s/DEPTH/$d/" min_stack.wf > ms_$d.wf
./wfc.sh       -o bin/ms_seq_$d ms_$d.wf
./wfc.sh --par -o bin/ms_par_$d ms_$d.wf
./bin/ms_seq_$d                   ; echo $?
env -u WF_WORKERS ./bin/ms_par_$d ; echo $?
WF_WORKERS=4      ./bin/ms_par_$d ; echo $?
```

**Observed**

| depth | sequential | `--par`, pool off | `--par`, `W=4` |
|---|---|---|---|
| 60 000 / 100 000 / 130 000 | 0 | 0 | 0 |
| 140 000 | 0 | **139 SIGSEGV** | **138 SIGBUS** |
| 200 000 / 300 000 / 400 000 / 500 000 | 0 | 139 | 138 |
| 700 000 / 1 000 000 | 139 | 139 | 138 |

Sequential ceiling 500k–700k. `--par` ceiling 130k–140k, **whether or not the pool starts**.

**Expected.** `par_runtime.c` documents `WF_WORKERS` unset as leaving "the pool unstarted, so every try_fork returns NULL and every program runs exactly the sequential schedule it runs today." A program that runs today should still run.

**Mechanism.** The sequential module has **zero** allocas anywhere. `--par` adds exactly one per function with an eligible pair — the hand-out frame — live across the recursive call:

```
$ grep -c "= alloca" ir/skew_seq.ll
0
$ awk '/^define /{fn=$0;sub(/^define[^@]*@/,"",fn);sub(/\(.*/,"",fn)}
       /= alloca/{printf "  %-20s %s\n", fn, $0}' ir/skew_par.ll
  wf_build_bt            %t0 = alloca { i64, double, ptr }
  wf_build_skew          %t0 = alloca { i64, double, ptr }
  wf_bfold               %t1 = alloca { ptr, double, double }
```

arm64 prologue of `wf_spine` (`otool -tV bin/ms_seq_500000` vs `bin/ms_par_130000`):

```
sequential                              --par
_wf_spine:                              _wf_spine:
  cbz  x0, ...                            cbz  x0, ...
  stp  x29, x30, [sp, #-0x10]!            sub  sp, sp, #0x40
  sub  x0, x0, #0x1                       stp  d9, d8, [sp, #0x20]
  bl   _wf_spine                          stp  x29, x30, [sp, #0x30]
```

**16-byte frame becomes 64 bytes**, plus two callee-saved FP spills. Four times the stack per frame, one quarter the depth.

**Reproduces on a real tree** (`bt_skew_*.wf`, left-spine `BNode` fold):

| depth | sequential | `--par`, pool off | `--par`, `W=2` |
|---|---|---|---|
| 20 000 / 100 000 | 0 | 0 | 0 |
| 105 000 | 0 | **139** | 0 |
| 110 000 / 120 000 / 130 000 | 0 | 139 | **138** |
| 135 000 | 139 | 139 | 138 |

23% depth loss on a real fold. Note 105 000: the `--par` binary crashes with the pool **off** and succeeds with a lane, because the lane moves the deep half to a second stack. Turning lanes on can rescue a crash that turning `--par` on caused.

**Why it matters.** (1) It contradicts the runtime's own invariant that a lane "may recurse exactly as deep as it would have on the calling thread" — the loss is on the calling thread, before any lane exists, so no worker-stack policy can restore it. (2) It contradicts the opt-in cost control: same schedule, different resource envelope. (3) A language whose premise is that memory corruption is unrepresentable reports this as an unhandled signal with no diagnostic, trap, or message.

The worker-stack floor is **not** at fault: in the same binary the worker ceiling is marginally *above* the main thread's (105 000 succeeds on a lane, fails pool-off), consistent with the 8 MB floor exceeding the 8176 KB `RLIMIT_STACK`. The defect is the compiler emitting the frame unconditionally.

---

# F2 — CRITICAL (performance) — hand-out is unconditional; there is no grain control

**Verdict: GAP.** Source: `q4.wf`, `bt.wf`, `base.wf` (control).

```
./timeit.zsh 5 "q4_seq=-:bin/q4_seq" "q4_w1=1:bin/q4_par" "q4_w2=2:bin/q4_par" \
               "q4_w4=4:bin/q4_par" "q4_w8=8:bin/q4_par" "q4_w64=64:bin/q4_par"
```

**`q4.wf`** — quad tree depth 7, 4000 reps, ~65.5M offers. All six cells sha `4400442d0dce55de`:

| cell | min | max | spread | vs `W=1` |
|---|---|---|---|---|
| sequential | 0.5865 s | 0.7798 | 33.0% | — |
| `W=1` | 0.5251 s | 0.7443 | 41.7% | 1.00x |
| `W=2` | 2.7855 s | 5.1536 | 85.0% | **5.3x slower** |
| `W=4` | 1.3003 s | 1.8023 | 38.6% | 2.5x slower |
| `W=8` | 7.1325 s | 7.9170 | 11.0% | **13.6x slower** |
| `W=64` | 25.5468 s | 30.8433 | 20.7% | **48.6x slower** |

Non-monotone: 2 lanes is worse than 4.

**`bt.wf`** — binary depth 16, 200 reps, all sha `67c43a57dab7a5cc`: seq 0.1739 · `W=1` 0.1565 · `W=2` 0.3137 · `W=4` 0.3598 · `W=8` 0.3992 · `W=64` 4.0218 s.

**The discriminator is grain, not thread count.** `base.wf` has the same fork structure but its per-node work includes an 8192-element scan:

| program | per-node work | `W=1` | `W=4` | |
|---|---|---|---|---|
| `base.wf` | 8192-word scan | 0.8887 s | 0.4545 s | **1.96x faster** |
| `bt.wf` | ~24 flops | 0.1565 s | 0.3598 s | 2.3x slower |
| `q4.wf` | ~24 flops | 0.5251 s | 1.3003 s | 2.5x slower |

8 lanes on 10 cores is not oversubscription, so scheduling does not explain it.

**Cost is kernel time, not user time:**

```
$ /usr/bin/time -p env WF_WORKERS=65 ./bin/bt_par
real 4.07   user 12.28   sys 26.58
$ /usr/bin/time -p env WF_WORKERS=8 ./bin/bt_par
real 0.44   user 1.19    sys 0.72
$ /usr/bin/time -p env WF_WORKERS=1 ./bin/bt_par
real 0.17   user 0.16    sys 0.00
```

38.9 s of CPU for 0.17 s of work — 223x amplification. Per-offer overhead rises with lane count (lower bounds, q4 delta over `W=1` ÷ 65.5M offers): ~12 ns at 4, ~101 ns at 8, ~382 ns at 64.

Contributing shapes in `compiler/src/backend/par_runtime.c`: `wf__par_try_fork` linearly CAS-scans every lane, so a **refused** offer is O(lanes); `struct wf__par_worker` is ~136 bytes unpadded, so `worker[i].claimed` can share a line with `worker[i+1].lock`; every grant does an unconditional atomic add on the process-global `wf__par_grants`, which by its own comment exists only for measurement; every completion broadcasts whether or not anyone waits.

The writer has no knob — `--par` is whole-program, and the only opt-out also disables the sites that pay off.

---

# F3 — HIGH — adjacency is brittle, and losing it is silent

**Verdict: GAP.** Source: `p1a.wf`, `p1b.wf` against `base.wf`.

`p1a.wf` is `par_layout` with one benign pure statement between the recursive `layout` calls, used only after both:

```
      let a = layout<'b, 'w>(node: move l, words: words, inh: child_inh);
      let gap = fmul.strict(child_inh, 1.5_f64);      <-- inserted
      let b = layout<'b, 'w>(node: move r, words: words, inh: child_inh);
      let kids = fadd.strict(a, b);
      let mine = fadd.strict(own_h, m);
      let withgap = fadd.strict(kids, gap);
      let total = fadd.strict(withgap, mine);
```

```
$ ./wfc.sh --par-ledger base.wf | grep layout
PAR permitted   input0.wf:116  pair(layout, layout)  eligible

$ ./wfc.sh --par-ledger p1a.wf | grep layout
(no output)

$ grep -c 'call ptr @wf__par_try_fork' ir/base.ll   # 2
$ grep -c 'call ptr @wf__par_try_fork' ir/p1a.ll    # 1
```

The pair is **not denied — it disappears.** The judgment never looks at it, so there is nothing to report and nothing for a writer to act on. Expected: a pure float multiply has no storage footprint and cannot conflict; at minimum the pair should still be judged and, if refused, say so.

**What the interleaving costs.** `p1b.wf` wraps the identical arithmetic in a pure user function; `p1a` and `p1b` emit the same bytes (sha `2c4d496258ec3e06`):

```
fn scale_up(x: own f64) -> result: own f64 pure { return fmul.strict(x, 1.5_f64); }
...
      let gap = scale_up(x: child_inh);
```

```
$ ./wfc.sh --par-ledger p1b.wf | grep -E "layout|scale_up"
PAR permitted   p1b.wf:120  pair(layout, scale_up)  eligible
PAR permitted   p1b.wf:121  pair(scale_up, layout)  eligible
```

| program | statement between | ledger | forks | `W=1` | `W=4` | speedup |
|---|---|---|---|---|---|---|
| `base.wf` | none | `pair(layout, layout)` eligible | 2 | 0.8887 s | 0.4545 s | 1.96x |
| `p1a.wf` | `fmul.strict` builtin | *no pair reported* | 1 | 0.7504 s | 0.7521 s | **1.00x** |
| `p1b.wf` | same op, `pure` user fn | 3-chain, both eligible | 3 | 0.8798 s | 0.5320 s | 1.65x |

**Two byte-identical programs are 1.41x apart at 4 lanes purely on whether an operation was spelled as a builtin or wrapped in a user function.**

Root cause: `Program::analyze_block` (`compiler/src/semantic/permission.rs:409`) grows `group` only from *consecutive* candidates, and `candidate_of` (`permission.rs:797`) returns `Some` only for a `let` whose RHS is exactly one `UserCall`.

---

# F4 — MEDIUM — the ledger reports pairs, never runs, never what was handed out

**Verdict: GAP.** Three symptoms of one gap: the ledger describes the judgment, not the outcome.

**(a) Chains are invisible.** `q4.wf` prints three adjacent pairs per function; the checker built a `PermissionRun` of four and the backend emitted three hand-outs in each (`wf_build4` 3 forks, `wf_qfold` 3 forks). `render_ledger` (`permission_ledger.rs:43`) walks `permissions.pairs` and never `permissions.runs`, so a reader cannot distinguish one run of four from unrelated pairs, and the non-adjacent ordered pairs the run analysis judged (`(ra,rc)`, `(ra,rd)`, `(rb,rd)`) are never shown.

**(b) The backend silently narrows what the ledger called eligible.** `p5b.wf` — ledger reports **4** eligible pairs, IR has **3** hand-outs:

```
PAR permitted   p5b.wf:24  pair(mkval, mkval)  eligible     <-- no fork emitted
PAR permitted   p5b.wf:27  pair(bump, bump)    eligible
PAR permitted   p5b.wf:36  pair(mkval, mkval)  eligible
PAR permitted   p5b.wf:81  pair(shape_promote, shape_plain)  eligible

  IR wf_shape_promote  1 forks      <-- ledger claims 2 eligible pairs here
```

Confirmed in `wf_shape_promote` — both `mkval` calls plain, only `bump` offered:

```
  %v2 = alloca i64
  %v4 = alloca i64
  %t0 = alloca { ptr, i64 }
  %v1 = call i64 @wf_mkval(double %v0)
  %v3 = call i64 @wf_mkval(double %v0)
  %t2 = call ptr @wf__par_try_fork(ptr @wf__par_thunk_0, ptr %t0)
  %v6 = call i64 @wf_bump(ptr %v4)
```

Cause: `a`/`b` are address-taken by `&uniq 'c a`, so `IrBuilder::overlaps` (`compiler/src/lowering/builder.rs:534`) drops every member but the last. Correct and deliberate — simply never disclosed.

**(c) An empty ledger is ambiguous.** `./wfc.sh --par-ledger zero_elig.wf` prints nothing, exits 0 — byte-identical to what a writer sees if the flag silently failed. Given F3, that is the common case.

---

# F5 — MEDIUM — allocation-heavy lanes cost ~3x more per hand-out than compute lanes

**Verdict: GAP** (secondary to F2). Source: `p4.wf` — builds a fresh depth-12 tree 4000 times (16.4M `box_new`) and walks it with a deliberately non-forking fold (`sum_v`'s child calls separated by one builtin, exploiting F3 as the only available cutoff knob), so measured hand-outs are allocation work only. IR confirms `wf_sum_v` has 0 forks.

| cell | min | spread | note |
|---|---|---|---|
| sequential | 0.5567 s | 10.4% | |
| `W=1` | 0.5884 s | 69.5% | (u) |
| `W=2` | 0.5876 s | 10.3% | (u) |
| `W=4` | 0.8806 s | 6.1% | 1.50x slower |
| `W=8` | 1.5412 s | 4.9% | 2.62x slower |

All cells sha `6ec87d5aab54fb6a`; max RSS flat at 1.9 MB over 4000 iterations, so boxes are freed correctly.

| phase | offers | `W=8`−`W=1` | per hand-out | CPU at `W=8` |
|---|---|---|---|---|
| allocation (`p4`) | 16.4M | 0.953 s | **~58 ns** | user 2.03 / sys 2.14 |
| compute-only fold (`bt`) | 13.1M | 0.243 s | ~18 ns | user 1.19 / sys 0.72 |

Both dominated by kernel time; allocation adds ~40 ns on top — the cross-magazine case where a lane allocates a subtree and the owning thread frees it.

---

# F6 — HIGH (design limit) — realistic per-node allocation can never actualize

**Verdict: WORKS-AS-DESIGNED**, but it bounds what v1 can do for a layout phase. Source: `p7_dyn.wf`.

A buffer sized by a parameter raises an undischarged `buffer_fits` obligation, dischargeable only by a `claim`; any claim in the closure makes the pair not actualizable. So the natural shape of a style-resolve phase — each node allocates a buffer sized by its own content — is permitted and permanently out of reach.

```
fn mkbuf_dyn(n: own u64, v: own f64) -> result: own buffer<f64> allocates(heap), traps {
  claim node_buffer_fits: buffer_fits<f64>(n) because "...";
  return buffer_new(n, v);
}
```

```
PAR permitted   p7_dyn.wf:9  pair(mkbuf_dyn, mkbuf_dyn)  not-actualizable: 1 claim site via mkbuf_dyn
```

Dropping the claim is not an option — without it the program does not compile (`[OP-9] UndischargedAllocationFitObligation`, residual `buffer_fits<Float(F64)>(n)`). The only claim-free allocation helper has a literal size. Same structure `par_layout` documents for `measure_band`, but landing on the *allocation* path every realistic build phase uses.

---

# F7 — LOW — the ledger cites a filename that does not exist

**Verdict: BUG** (cosmetic).

```
$ ./wfc.sh --par-ledger p5b.wf | head -1
PAR permitted   p5b.wf:24    pair(mkval, mkval)  eligible
$ ./wfc.sh --par-ledger ./p5b.wf | head -1
PAR permitted   input0.wf:24 pair(mkval, mkval)  eligible
$ ./wfc.sh --par-ledger "$PWD/p5b.wf" | head -1
PAR permitted   input0.wf:24 pair(mkval, mkval)  eligible
```

Any argument containing a path separator is reported as `inputN.wf` — a file that does not exist — with line numbers attributed to it.

---

# Verified working, no action needed

**Nested hand-out on worker threads — WORKS-AS-DESIGNED.** `bt.wf` depth 16, hand-out offered at 16 nested levels:

```
$ /usr/bin/time -p env WF_WORKERS=1 ./bin/bt_par   → real 0.16 user 0.15 sys 0.00
$ /usr/bin/time -p env WF_WORKERS=8 ./bin/bt_par   → real 0.41 user 1.18 sys 0.57
$ ./timeit.zsh 5 "bt_w64=64:bin/bt_par"
bt_w64  min 4.0218s  max 4.3509s  spread 8.2%  fails 0  sha 67c43a57dab7a5cc
```

1.75 s CPU in 0.41 s wall ≈ 4.3 concurrent streams. A binary fold can only exceed two streams if workers themselves fork and join — they demonstrably do, correctly, 16 levels deep. Byte-identical at `W=8` and `W=64`; 5/5 exit 0 at `W=64`; no deadlock, no livelock. Skewed variant byte-identical to sequential at every depth below F1's ceiling (20 000 / 60 000 / 100 000 all `same_bytes=yes` at `W=2`).

**Chains and N-ary — WORKS-AS-DESIGNED.** `q4.wf`: four adjacent eligible calls in both `build4` and `qfold`; run of four, N−1 = 3 handed out in each (6 `try_fork`, 6 `wf__par_join`); byte-identical at `W` 2 and 8 (and 1, 4, 64). Three adjacent verified separately in `p1b.wf`.

**Match arms, loop bodies, region bodies, results feeding a later region — WORKS-AS-DESIGNED.** `p5.wf`: pairs in a loop body, a region body, both match arms, a pair whose results feed a later region, plus a 5-chain in `main`. 11 eligible pairs, 11 forks — `wf_shape_loop` 1, `wf_shape_region` 1, `wf_shape_match` 2, `wf_shape_addressed` 2, `wf_shape_value` 1, `wf_main` 4. Byte-identical at `W` 2, 4, 8, 64.

**Ledger quality at scale — WORKS-AS-DESIGNED.** `scale.wf`, 1616 lines, 200 functions each with one eligible pair:

```
$ ./wfc.sh --par-ledger scale.wf | wc -l            # 200
$ grep -c 'call ptr @wf__par_try_fork' ir/scale.ll  # 200
$ sort -n -C logs/scale_lines.txt                   # monotone by line
$ for i in 1 2 3 4 5; do ./wfc.sh --par-ledger scale.wf > logs/scale_d$i.txt; done
$ shasum -a256 logs/scale_d*.txt | awk '{print $1}' | sort -u | wc -l   # 1
```

Complete (200/200), deterministic (1 distinct output over 5 runs), monotone source order. Also deterministic on `p5.wf` over 8 runs, and identical with `-o`, with `--emit-llvm`, or alongside `--par`. Clean at `W` 2, 8, 64.

**Denial accuracy — WORKS-AS-DESIGNED.** `p6.wf`, all five conditions. Conditions 3 and 4 are reachable only after unmasking earlier ones (the first draft had both masked, by a condition-2 write/write on `out` and a condition-1 dataflow):

```
condition 2: writes overlap at &uniq 'c cell vs &uniq 'c cell
condition 2: the write of s1 overlaps the read of s2 at &uniq 'c cell vs &'c cell
condition 2: the read of s1 overlaps the write of s2 at &'c cell vs &uniq 'c cell
condition 2: the write of s1 overlaps the operand read of s2 at &uniq 'c cell vs cell
condition 3: the row of s1 carries external, blocks
condition 4: Err edge of s1 skips s2
condition 1: an argument of s2 uses the result of s1     (base.wf)
```

Every message names the access that actually refused the pair, including all four condition-2 sub-kinds. The caller-side operand read — the subtle one the module header calls out — is correctly identified and distinguished from a callee read.

**`WF_WORKERS` edge values — WORKS-AS-DESIGNED.** All exit 0, bytes identical. Whether the pool actually started was determined by kernel time, not assumed:

| value | pool | evidence |
|---|---|---|
| unset, `''`, `0`, `1`, `-1`, `-5` | off | `sys 0.00` |
| `abc`, `3x`, `2.9`, `0x8` | off | `sys 0.00` |
| `'4 '`, `'  8  '` (trailing space) | off | `sys 0.00` |
| `2`, `3`, `4`, `+4`, `' 4'` | on | `real 0.38 user 0.84 sys 0.16` |
| `64` | on, 64 lanes | |
| `65`, `999999999999999999999`, `100` | on, clamped to 64 | `real 4.07 user 12.28 sys 26.58` |

Fails closed in every ambiguous case. Leading whitespace and `+` accepted (`strtol`); trailing whitespace silently disables the pool. The huge value overflows `strtol` to `LONG_MAX` with `errno` unchecked, but the `> WF_PAR_MAX_WORKERS` clamp catches it — latent, not reachable.

**Zero eligible sites links no runtime — WORKS-AS-DESIGNED.** `zero_elig.wf` (`min_stack.wf` with the pair broken by one builtin):

```
$ grep -cE "wf__par_try_fork|wf__par_join" ir/zero_elig.ll   # 0
$ nm bin/zero_par | grep -c wf__par                          # 0
$ nm -u bin/zero_par | grep -c pthread                       # 0
$ stat -f%z bin/zero_par bin/zero_seq                        # 33768  33768
```

---

# Coverage against the brief — nothing NOT-RUN

| brief item | status | where |
|---|---|---|
| 1. Adjacency brittleness, ledger before/after, cost quantified | done | F3 |
| 2. Chains / N-ary, 3 and 4 adjacent, chain reported?, N−1 handed out?, bytes at `W` 2 and 8 | done | F4a + Chains |
| 3. Nested hand-out on workers, depth 14+, `W` 8 and 64, skewed tree, stack stress | done | F1 + Nested hand-out |
| 4. Allocation-heavy lanes, bytes and cost at `W` 1 vs 4 vs compute-only fold | done | F5 |
| 5. Match-arm and region shapes; ledger vs IR for **every** probe | done | F4b + Match arms + agreement table |
| 6. Ledger quality at scale: deterministic, complete, denials name real accesses | done | Ledger at scale + Denial accuracy |
| 7. `WF_WORKERS` edges 0/1/2/64/65/garbage/huge; zero-eligible link check | done | Edge values + Zero eligible |

Two limits on what this establishes, stated rather than hidden:

- **Chain reporting was answered by reading the source, not only the output.** The ledger prints no run line at all, so "does the ledger report a chain" is answered *no* from `render_ledger` walking `pairs` only, corroborated by IR fork counts. There is no positive output to quote.
- **The F2 per-offer nanosecond figures are lower bounds** — wall-clock delta over `W=1` divided by offer count, ignoring whatever parallel speedup was simultaneously earned. The wall-clock ratios themselves are direct measurements and do not depend on that arithmetic.

# File index

Landed in `probes/`, durable: `p1a.wf`,`p1b.wf` (F3) · `q4.wf` (F2, F4a) ·
`bt_skew.tmpl` (F1 realistic) · `min_stack.wf` (F1 minimal) · `bt.wf` (F2 grain,
nested hand-out) · `p6.wf` (denials) · `p7_dyn.wf` (F6) · `zero_elig.wf`
(linkage).

Still in the research scratch area, not durable: `base.wf` (control) ·
`bt_skew_*.wf` and `ms_*.wf` (the generated F1 depth sweeps) ·
`p4.wf` (F5) · `p5.wf` (shapes) · `p5b.wf` (F4b) · `scale.wf` (ledger at scale) ·
`wfc.sh` · `ir/`, `bin/`, `out/`, `logs/`. The interleaved min-of-N timer the
timings above were taken with is landed as `bench/timeit.zsh`.
