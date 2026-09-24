# Slab library costs

The later [same-source inactive-storage compiler comparison](../map-library/RESULTS.md#completed-comparison-gains-with-unresolved-regressions)
preserves this experiment's complete matrix in
[raw samples](measurements-inactive-raw.tar.gz) and a
[paired summary](measurements-inactive-summary.csv). It finds wide-churn
improvements and retained scalar lookup regressions; its results and compiler
identities are separate from the historical measurements below. The owner
rejected that compiler optimization and its bounded SSA follow-up; production
retains baseline clearing, while these samples preserve the negative evidence.

This explicit experiment bundles [`slab.wf`](../../../../lib/containers/slab.wf).
It is outside daily correctness CI. `make check` verifies the operation traces;
`make measure` checks them first and writes interleaved timing samples under
`.build/`. Retire this experiment when a maintained successor covers the same
contract and preserves its comparison evidence.

## Contract and criterion before timing

The selected Slab reserves one backing, materializes slots lazily, stores
each payload inline in `Slots<T,1>`, and reuses stable index/generation handles.
Removal increments the generation and relinks a vacancy, except that its
maximum generation is permanently retired. This experiment uses the full u64
generation domain; the formal library caller forces retirement with a small
limit. It does not claim stable physical addresses, cross-slab handle
authentication, or unrestricted static retained membership.

Compare WF with C using its same one-slot-window layout and with a native
tagged cell that overlays the vacant next link with the live payload. Both C
controls validate index, occupancy, and generation and return ordinary
aggregate result/option values at retained helper boundaries. The layout
comparison therefore does not disguise return transfers by changing the
result ABI to success flags plus output pointers. Neither side boxes payloads.
No projection or unsafe capability is proposed from a byte-size comparison.

Here "same layout" means the cells, not every function result ABI. WF's
insertion result keeps both the Handle and T payload regions; native C uses
a union. Handle arguments and small option results also have different native
calling conventions. The two C controls share those result/call conventions,
so their difference isolates the cell representation. The WF/window-C gap
also contains result layout, helper ABI and lowering costs.

Two traces separate prefilled lookup from remove/consume, stale-handle lookup,
and reinsertion. The offered owner is recovered when the initial full slab
refuses one further insertion. Returned handles and every payload word enter
an order-sensitive checksum; an independent arithmetic oracle derives the
expected values and outcomes without a slab. Final teardown consumes live
payloads in reverse materialized index order. Counts 16, 256, and 4096 use
8-byte scalars or 256-byte `nocopy` records. Correctness adds empty/singleton
and irregular counts, zero rounds, and wrapping seeds.

A separate `setup-cleanup` measurement uses zero workload rounds: reserve,
materialize the initial values, recover the refused offer, consume all values,
and release the backing. It repeats `32768 / count` complete traces per sample
to avoid timing a single tiny allocation. This prices setup plus cleanup; it
is not subtracted from other timings to invent isolated operation latencies.

Setup and cleanup are included and amortized, never labeled isolated lookup
latency. Compare exact allocation counts and per-layout requested/peak bytes,
require final zero live bytes, and inspect optimized WF/C IR for preserved
helpers and element transfers. Normal and retained timings expose sensitivity
to helper boundaries and optimization visibility. A byte gap alone does not
establish an application-level performance gap; no universal parity threshold
or workload frequency is assumed.

## Results

The corrected baseline has 4,752 accepted timing samples in
[`measurements.csv`](measurements.csv), preserved at checkpoint `5a5481b1`
before the merge of main `7127bcb6`. Both normal and retained executables
passed 288 correctness configurations, each run through all three
implementations: 1,728 executions in total. Every execution checked the
independent checksum, allocation requests, requested/peak bytes and final
zero live bytes. The retained optimized IR contains 14 WF helper call sites
and 28 C sites across its two representations.

### Storage and allocation

| Quantity | WF / window C, scalar | Tagged C, scalar | WF / window C, record | Tagged C, record |
| --- | ---: | ---: | ---: | ---: |
| Payload bytes | 8 | 8 | 256 | 256 |
| Cell stride | 32 | 24 | 280 | 272 |
| Backing bytes at capacity n | 16 + 32n | 16 + 24n | 16 + 280n | 16 + 272n |
| Insertion-result bytes | WF 32 / C 24 | 24 | WF 280 / C 264 | 264 |

Each complete trace makes exactly one allocation. Its requested bytes equal
its peak bytes; repeating setup traces multiplies requests and total bytes,
not peak bytes. These are payload/backing requests, excluding the observer's
private accounting header and allocator metadata. The 8-byte cell difference
comes from overlaying the free link with the payload in the native control;
it is not evidence that an ordinary tag alone costs an additional word.

### Amortized trace costs

The time column is WF nanoseconds per logical lookup or reuse step at
`count = 4096`. Ratio columns cover all three counts and all four run/cohort
medians. Values above 1 mean WF took longer. Ranges describe the measured
cohorts, not confidence intervals. Setup and final cleanup remain included.

| Mode | Payload bytes | Trace | WF ns/step, n=4096 | WF / window C | WF / tagged C |
| --- | ---: | --- | ---: | ---: | ---: |
| normal | 8 | prefilled lookup | 1.617–1.740 | 1.000–1.209 | 1.000–1.200 |
| normal | 8 | reuse churn | 4.120–4.303 | 1.311–1.352 | 1.302–1.348 |
| retained | 8 | prefilled lookup | 3.845–3.998 | 1.044–1.169 | 1.048–1.186 |
| retained | 8 | reuse churn | 11.749–12.238 | 0.981–1.091 | 1.006–1.100 |
| normal | 256 | prefilled lookup | 38.269–38.483 | 1.053–1.112 | 1.061–1.110 |
| normal | 256 | reuse churn | 57.159–58.899 | 1.454–1.509 | 1.510–1.588 |
| retained | 256 | prefilled lookup | 38.025–38.910 | 0.968–1.012 | 0.975–1.016 |
| retained | 256 | reuse churn | 62.775–64.667 | 1.090–1.162 | 1.097–1.137 |

The controlled C layout comparison is much smaller than the normal WF reuse
gap: in normal mode, window C / tagged C is 0.979–1.010 for scalar reuse and
1.026–1.062 for record reuse. That supports retaining the ordinary one-slot
representation as a usable first library while recording its storage premium. It does not
justify calling the library native parity or attributing its whole time gap
to layout. Returning owning values through enums remains a measurable
lowering/ABI concern.

### Zero-round setup and cleanup

Times below are WF microseconds per complete reserve/materialize/refuse/free
trace, with the four cohort median range at each count. Ratios span all three
counts. This includes the callback checksum and owner cleanup.

| Mode | Payload bytes | n=16, us/trace | n=256, us/trace | n=4096, us/trace | WF / window C |
| --- | ---: | ---: | ---: | ---: | ---: |
| normal | 8 | 0.093–0.095 | 0.852–0.859 | 12.500–12.625 | 1.075–1.780 |
| retained | 8 | 0.194–0.202 | 2.609–2.648 | 42.000–44.000 | 1.265–1.377 |
| normal | 256 | 0.853–0.876 | 12.680–12.711 | 206.000–216.875 | 1.316–1.370 |
| retained | 256 | 0.840–0.863 | 12.688–13.398 | 206.125–215.125 | 1.154–1.177 |

### Optimized transfer evidence

The retained record specialization makes these copies after O2. Counts are
executed per successful operation, not static copies across every branch.

| Path | WF | Window C |
| --- | --- | --- |
| Remove then consume | 2 x 256-byte copies in `slab_remove` (slot to local to Option), then 1 x 256-byte copy from Option to the consumer local in the trace | 1 x 256-byte copy into the result; the trace passes its payload address to the consumer |
| Final live-cell cleanup | 1 x 264-byte copy of the one-slot length and payload into a local | 1 x 256-byte payload copy into a local |
| Insert | No memcpy intrinsic, but explicit vector/scalar loads and stores transfer the payload | Payload is transferred into its cell |

The relevant WF functions in `slab-library-retained.opt.ll` are
`wf_slab_remove$instance$49`, `wf_slab_library_trace$instance$40`,
`wf_slab_free$instance$50`, and `wf_slab_insert$instance$47`; the C counterparts
are `record_window_remove`, `record_window_trace`, and `record_window_free`.
Zero memcpy intrinsics do not mean zero transfer. Scalar option results use
a hidden WF result pointer versus a native C register aggregate, and WF passes
Handle by address where C uses a two-word value. The result-size table above
also distinguishes WF's non-overlaid insertion result from the C union.

The copy counts establish extra transfers; these measurements do not isolate
their exact share of elapsed time. Normal versus retained additionally changes
inlining, visibility, and native calling conventions, so its ratio is not a
copy-only counterfactual. The next bounded improvement is to test removal and
cleanup placement while preserving this source contract, not to introduce a
new storage primitive from these timings alone.

### Instrumentation and source checks

Allocator observation originally renamed `malloc` to `wf_cost_allocate` but
left WF's declaration without a return attribute. Clang inferred a fresh,
nonnull result from the C wrapper's body while the separately optimized WF
module lost the known `malloc` information. Those provisional timings are
not evidence of a compiler or language cost. The adapter now declares the
wrapper's proven `noalias nonnull` return: it returns a fresh allocation after
an accounting header and exits on failure. No argument alias promise or
memory-effect attribute is added. The allocation formulas and independent
checksum are checked again after this correction.

The first source admission also exposed a compiler defect: nested forwarding
of `fn SlabElement::visit` to `slab_visit` with `R = unit` rechecked the callee's
generic `move observed` spelling as a concrete copy move and rejected OWN-1.
A direct `SlabVisit<u64, Env, unit>` binding accepted. FN-2 requires the written
generic spelling judgment to be preserved. The compiler repair admits this
unchanged caller; no result-type workaround or library capability weakening
is part of the experiment.

### Reproduction and units

The host is Apple M1 Pro, eight logical CPUs, arm64 macOS 26.6.2 / Darwin
25.6.0, with Apple Clang 21.0.0 (`clang-2100.3.34.2`). Construction, checks,
and measurement run under `perl .github/run-check.pl`; no other heavy
verification task owns the shared guard at the same time. The local command is
`make -C research/experiments/container-representation/slab-library measure`.
Normal mode permits ordinary O2 inlining; retained mode marks library,
workload, and member helpers noinline. C keeps one external trace boundary
even in normal mode to match the separately optimized WF module. Compiler-owned
primitive bodies remain eligible for optimization on both sides.

Two complete runs reverse the normal/retained executable order. Within each,
two cohorts reverse implementation order and eleven samples rotate the first
implementation. Seeds vary by sample; all implementations use the same seeds
and logical oracle. The CSV's `run`, `cohort`, and `traces` columns preserve
these distinctions. A nonzero-round figure divides elapsed time by
`rounds * count`; a setup figure divides by `traces`. Ratios compare medians
within the same run/cohort, not unrelated samples or a subtraction of setup
time. The record checksum reads all 32 words, so its cost is part of every
record lookup/consumption and these are not pure memory-bandwidth timings.

The corrected run took 1.43 seconds for incremental experiment construction
(WF emission, native compilation/linking and optimized IR), then 5.42 seconds
for `make measure` including its correctness checks. These do not include a
Rust compiler build or a cold runtime-object build. The timer is
`CLOCK_MONOTONIC`; the recorded host samples have microsecond granularity,
so normalized decimal places do not imply a sub-nanosecond clock. Differences
around one percent are not a precise gain or parity claim; timer quantization,
cohort variation and the checksum's share of record work limit that inference.

The measured compiler SHA-256 was
`4311534e72d2df6ccb2f6043465a801f42f98a70f940f1249755267da2b8c31e`.
The subsequent const-forwarding repair's compiler
`5d3a09f7f07b09ac333615830340878a24daa81ef42d5e61c3a5200098aa84c5`
emitted byte-identical raw LLVM for the measured source. The experiment makes
no specification change. Artifact SHA-256 values fix the corrected baseline;
the Slab source is the version at commit `2d699b69`, before its subsequent
alias-only cleanup:

| Artifact | SHA-256 |
| --- | --- |
| `spec/kernel-spec.md` | `2b4df9e688befb4d7ea1a58d7b95dfe56f436ab0739caee9db934763abd21b35` |
| `lib/containers/slab.wf` | `59138e8f87112cbb651389378f09a8ad6874b4c2aec65a56284af6139e683d6d` |
| `slab-library.wf` | `a39d85db7348a0a51c433c658ac894bf81d552d383e7acc3f1b435d027b0a536` |
| `slab-costs.c` | `fe348ebd56dadc645ab25f25d28656d99635e0bd5d583fb440da66d1611ff2b4` |
| `Makefile` | `25019a08f9977323335004ed2589051e0475aa0cb98165a0a5beed1508a0cd2f` |
| Raw compiler LLVM | `6d606576f56fe4a635b94b64f90a38a83c7c7583182f5c22bf0a814de8c3e496` |
| `measurements.csv` | `60a2463b52f7af66e106b4eb5311dd00722965475baa8ec0fc4f457fb35fbe5b` |

### Integration verification at dcbfdc0f

After merging main `7127bcb6` and removing Slab's temporary const alias, the
compiler at `dcbfdc0f` compiled the unchanged benchmark source with the updated
library. Compiler SHA-256 was
`9653362a116997dd34558b9da6436dfb343018631b4fabcaa255e420a0c80005`;
the current library source was
`0a8c104e0323a5ffcc10dd2938e0c280d99941cde18dac5a1537f50c3b341eae`.
The workload, C driver, Makefile and original CSV retained the hashes above.

The normal and retained optimized LLVM differ from the preserved baseline
only in local SSA names (`%v46` becomes `%v45`, including its inlined form).
The operations and metadata are unchanged. Each old/current optimized module
was then compiled with Apple Clang 21.0.0 on the same arm64 host using
`clang -O2 -Wno-override-module -S -x ir`; both complete assembly files compare
byte-for-byte equal, without normalization. The two C controls' optimized IR
also remains byte-identical.

| Current artifact | SHA-256 |
| --- | --- |
| Raw compiler LLVM | `6a71943e1d04f9fe256ed9b3783866b7f3cc4096d20e59ecd71a1114601694ed` |
| Normal optimized LLVM | `ba7ffd1ebf0f8754df96016463fba9ab164782d3d5c1926dd9c6c9055bfb5aa3` |
| Retained optimized LLVM | `78e7eae9c503cb84bbeb6b4f6240503be360102cae1d6f097c90da539704aa73` |
| Normal old/current assembly | `a751f36f3f1a1d5605d6fc18a6f438a31c74e065f95a563ffe905f8bc04ca98e` |
| Retained old/current assembly | `cb709317e39f17fb0c6365a72c487c2955b73048cbb04997303493825d218c8e` |

This compares the emitted WF module via optimized IR, not the complete linked
native image. The linked runtime's C, header and LLVM sources and
`compiler/runtime.mk` did not change between the checkpoint and this revision.
The current normal/retained harnesses passed all 1,728 correctness executions
again. Incremental construction took 2.34 seconds; those executions and the
retained-call checks took 0.71 seconds, under the shared guard. The combined
Slab/Deque integration check, assembly generation and optional Deque probe
took 7.51 seconds.

No new timing run was needed for the name-only IR change. The original CSV
remains historical timing evidence, not a fresh measurement of this revision;
the code comparison is specific to these source instantiations and this
Clang/arm64 configuration, not a claim about other toolchains or programs.

Requalification after main PR #80 used merge `4da1710e` (main `95b21cfd`)
and compiler SHA-256
`cb399df104e975c607973f151dd87a4caf2e4a0e9606a468b4ea69d683345a48`.
The library, workload, C driver, Makefile, runtime inputs and CSV were
unchanged. Fresh raw, normal-optimized and retained-optimized WF LLVM were
byte-identical to the saved `dcbfdc0f` artifacts; the C controls' optimized
modules retained their exact hashes too. The artifact table above therefore
also identifies the newly emitted WF modules. No normalization was needed.

The guarded `slab-deque-pr80-emission` command requested these Make targets:

```sh
make -C research/experiments/container-representation/slab-library \
  .build/slab-library-normal.opt.ll .build/slab-library-retained.opt.ll \
  .build/control-normal.opt.ll .build/control-retained.opt.ll
```

Slab emission/optimization took 0.69 seconds; the combined Slab/Deque guard
interval took 1.03 seconds. The unchanged C targets were already current.
This byte-identity check reuses the earlier assembly comparison and native
checks. No native relink, execution, probe or timing was repeated, and the
original CSV remains the historical baseline rather than a new PR #80
performance result.

Requalification after main PR #88 used merge `ce9a3870` (main `e8e1c411`),
which releases v0.64. The gate compiler SHA-256 is
`b68db16443f0606ef5d3217014fbdb1bdae0ea01bc23452eff27e2626bbc24ff`;
the active specification is
`bc4d465d698a63518d4c768bfa0b4147afa15e328e32f8adac3f27980c7a21ed`.
The library, workload, C driver, Makefile and original CSV retain their
previously recorded identities. Fresh raw LLVM changed to
`97818ef7d593144335b0a6980fb079689549ad1b57e6f2b9767acb3ed653998f`:
the emitted module now includes the POSIX resource-record EINTR retry helper.
This is not raw-module identity with the earlier revision.

Both complete optimized WF modules are nevertheless byte-identical to the
`dcbfdc0f` entries above, without normalization; the two C-control optimized
modules are unchanged too. The reporting helper is removed by optimization
under the accounting allocator's nonnull return contract. The zero-stride
addressing changes do not change these positive-stride Slab instantiations.
This reuses the earlier emitted-module assembly correspondence, without a
claim of a newly linked image or new timings. Native runtime C/header/LLVM
sources and `compiler/runtime.mk` are unchanged across the integration.

The guarded `slab-deque-v64-build-emission` interval built the compiler using
`make -C compiler build` (gate profile), then requested the same four Slab
Make targets shown above. Compiler construction took 46.94 seconds and Slab
emission/optimization took 0.76 seconds. The complete interval, also including
Deque emission and its changed-code assembly comparison, took 48.38 seconds.
No Slab relink, correctness execution or measurement was repeated; the
original CSV remains historical evidence qualified by this exact
optimized-module comparison on Apple Clang 21 / arm64.

Requalification after main PR #87 used merge `fd41dcc8` (main `8d6da723`),
which releases v0.65, and compiler SHA-256
`0ab0f5730590828c511d0a0d4d90d3131654377ea020e5634f5a0800ff9edb96`.
Fresh normal and retained optimized WF modules compare byte-for-byte equal
to the saved v0.64 artifacts, without normalization; both C-control modules
are unchanged too. The library, workload, harness, native runtime inputs
and CSV retain their identities. Gate-profile compiler construction took
46.75 seconds; Slab emission/optimization through the same four Make targets
took 0.72 seconds, and the shared `slab-deque-v65-emission` interval took
1.14 seconds. This confirms emitted-module correspondence on the recorded
Apple Clang 21 / arm64 configuration. No native relink, execution, probe or
timing was repeated, and no historical measurement is relabelled as a new
v0.65 result.
