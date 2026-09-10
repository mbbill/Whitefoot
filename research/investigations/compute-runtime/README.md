# Compute runtime qualification

## Current owner direction (2026-09-09)

The owner reviewed the [comparison](PURE-COMPUTE-COMPARISON.md) and authorized
production integration on 2026-09-09. Use historical main `9051576f`'s ordinary
worker stacks and current-stack join/help/steal as the base, with atomic deque
cells, corrected thief ordering and startup/lifetime fixes. Deliver through
ordinary `whitefootc`, in `compiler/`, with one runtime for all programs.
The unified implementation remains an exact
[research checkpoint](../io-model/UNIFIED-RUNTIME-CHECKPOINT.md).

Compute workers must not run arbitrary may-suspend WF calls. Such calls execute
ordinarily; direct typed I/O operations still submit and join through the
existing io_uring, IOCP or helper backends, waiting on the current stack.
No managed-stack pool, continuation migration or ready queue belongs in the
compute path. The public ordinary-call ABI and proof judgments stay unchanged.
This deliberately retires generic staged user-call concurrency: network fanout
that needs independently suspended WF activations is deferred. Sequential TCP,
direct independent I/O submission, result/error handling, cleanup and exhaustion
remain required. Tests of the retired mechanism must be replaced or retired
with this explanation, never silently treated as equivalent functionality.

The implementation review covers scheduler sources and platform leaves,
completion record publication/waiting, floor entry, compiler hand-out selection,
staged-loop lowering and their tests. Reuse the existing workload/reference
panels for five-target native CI qualification below; do not build another
benchmark framework or tune I/O. Historical safety evidence has the limited
scope stated in the comparison, not a claim that every earlier audit was wrong.

## Compute-first measurements at b87e7548 (2026-09-09)

The [five-target cohort](https://github.com/mbbill/Whitefoot/actions/runs/34422006371)
uses the maintained compute-first runtime through the compiler. All twelve
[gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34422006323) and both
[native completion jobs](https://github.com/mbbill/Whitefoot/actions/runs/34422006315)
passed. Exact revision `b87e75481c0791d0ba190a283e7024c15d34591a` also passed
local canonical `make check`. Performance qualification remains incomplete;
all five formal FIR screens failed their full matrix, including unresolved
noise, and only macOS AArch64 passed the ordinary-command screen. A screen
pass alone does not meet the broader acceptance criteria.

For one small FIR cell (4,096 outputs, tile 16), median paired ratios below
are candidate/reference; smaller is faster. Each cell uses five process pairs.
These are selected diagnostic cells, not a summary of every workload:

| Target | Participants | Core / historical | Full call / historical | Core / research | Core candidate/replica range |
| --- | ---: | ---: | ---: | ---: | --- |
| Linux x86-64 | 4 | 0.966 | 0.982 | 1.658 | 0.923–1.066 |
| Linux AArch64 | 4 | 0.995 | 1.004 | 0.981 | 0.982–1.030 |
| Windows x86-64 | 4 | 0.981 | 1.107 | unavailable | 0.961–1.038 |
| macOS x86-64 | 4 | 1.115 | 1.160 | 1.087 | 0.894–1.243 |
| macOS AArch64 | 2 | 0.888 | 0.825 | 0.927 | 0.718–1.646 |

The macOS replica variability prevents a resolved ranking. Linux x86-64
candidate/research core ratios are 1.589–1.724 across all five pairs; matching
historical main is therefore insufficient. The original research control is
POSIX-only. Full-call timing includes allocation, reading/checking results and
cleanup outside the measured core; it must not be relabeled scheduler time.

On Windows, restoring the historical `YieldProcessor` hint in the maintained
core's two existing empty-scan spin branches reduces core time by 33.3% and
full-call time by 13.0% against a frozen unhinted maintained core at `d39b4836`
in this same cell. Whole-batch CPU ratio is 1.007. These two images share the
candidate host object and all other sources; only the two hints differ in the
core. Compared with historical main, full-call and batch CPU ratios remain
1.107 and 1.128, so core parity does not qualify the whole call. POSIX generated
core assembly is unchanged by the Windows-only hint. Spin-duration and SMT
effects are not separated by this comparison.

Linux ordinary-command scheduling traces identify a placement hypothesis.
In the AArch64 work60000/32-repetition Mandelbrot observation, one worker runs
7.100 ms but spends 25.363 ms runnable; 6.228 ms of its execution shares CPU 2
with the 26.633-ms caller. Another CPU is mostly unused by the workload.
The native static observation places four participants on distinct CPUs and
has 0.494 ms total observed runnable delay. These are different traced
processes, not causal proof. Initial new-thread wakeups were not recorded;
the launcher is excluded from the compute participant count. Long-run CPU
samples also use a different duration and cannot explain the short run's
off-CPU time. The existing ordinary panel's matched per-thread placement
control tests whether runnable delay and the wall gap fall together. It does
not change production affinity, waiting policy or the default split budget.
Artifacts retain raw samples, source, binaries, host details and decoded perf
events; observer envelopes are not added to unobserved performance samples.

The [7e9c8ca9 placement cohort](https://github.com/mbbill/Whitefoot/actions/runs/34423914331)
completed binding/unbound correctness, all 120 observations per Linux target
and raw scheduling traces. Binding does not resolve the gap: work60000's
32-repetition median bound/unbound wall ratios are 1.107 on AArch64 and 1.057
on x86-64, with substantial replica variability. At 256 repetitions they are
1.025 and 1.018. This provides no basis for default production affinity.

External competition invalidates placement alone as an explanation. In the
AArch64 bound work60000 trace, external PID 1919 occupies CPU 1 for 18.640 ms
of the 36.900-ms workload window. Its execution overlaps 18.470 ms of the
bound worker's 18.581-ms observed runnable delay. The separate bound static
trace spans 23.767 ms with at most 0.549 ms from any single external task/CPU
pair. These traces did not receive equivalent background load; neither can
be used to subtract interference from the untraced timing samples. The next
control fixes affinity and varies both programs' compute-thread priorities
equally, testing interference sensitivity without changing unrelated CI
processes or the maintained runtime's waiting policy.

The repeated x86-64 FIR comparison still requires investigation: candidate/
research core ratio 1.613, five-pair range 1.566–1.697, while candidate/replica
ranges 0.965–1.043. Candidate/historical is 0.984. This independent cohort
confirms the comparison gap; the short-command interference traces do not
explain or excuse this different workload's loss.

The [b20852c6 priority cohort](https://github.com/mbbill/Whitefoot/actions/runs/34424786215)
completed both Linux diagnostic panels with every participant's requested
priority read back. Median paired high/default-priority wall ratios are:

| Target | Work threshold | 32 repetitions | 256 repetitions |
| --- | ---: | ---: | ---: |
| Linux x86-64 | 60,000 | 0.844 | 0.997 |
| Linux x86-64 | 120,000 | 0.901 | 1.005 |
| Linux AArch64 | 60,000 | 0.913 | 0.971 |
| Linux AArch64 | 120,000 | 0.888 | 0.969 |

Native static high/default ratios remain about 0.996–0.999. Short WF
replica ranges are wide; longer high-priority WF/static ratios remain about
1.08–1.11. The controls establish priority sensitivity, not complete removal
of background interference or production performance acceptance. Both
conditions use root, fixed affinity and the same measurement wrapper; no
correction is applied to ordinary-user samples. The actual child cgroup,
autogroup, UID and per-thread settings are retained in the artifact.

Independent examination of the x86-64 FIR images found matching
address-normalized instructions in historical/recovered worker, join,
publish, release, acquire and sampled WF compute/accessor functions. This
FIR program recursively publishes by tile and does not call `split_budget`.
The unresolved gap therefore does not establish a missing grain-policy or
idle-algorithm optimization. The bounded formal-panel control fixes shared
WF/host-consumer text addresses, retaining the original images and their
failures. It tests one layout influence, not all possible cache or scheduling
effects, and leaves production linking unchanged.

The [f220e288 layout cohort](https://github.com/mbbill/Whitefoot/actions/runs/34425771609)
on Linux x86-64 (AMD EPYC 7763, four logical CPUs/two cores, Clang 18.1.3)
completed all twenty fixed-layout observations with correct results and four
participants. Nineteen common non-weak functions have equal linked addresses
and sizes across the three cores; replicas are byte-identical. For the same
W4/4,096-output/tile16 cell, process medians are:

| Metric | Original candidate | Fixed-layout candidate | Original recovered | Fixed-layout recovered |
| --- | ---: | ---: | ---: | ---: |
| Core time, microseconds | 37.079 | 21.987 | 22.069 | 21.997 |
| Full call, microseconds | 141.495 | 107.124 | 102.880 | 105.757 |
| Voluntary context switches, whole batch | 11,308 | 402 | 303 | 501 |
| System CPU, whole batch milliseconds | 472.903 | 37.076 | 31.111 | 60.998 |

Candidate/recovered paired core ratios change from median 1.7041
[1.5896, 1.7963] in the original panel to 0.9905 [0.9791, 1.0170] in the layout
panel. Layout candidate/replica remains noisy at 1.0061 [0.9255, 1.0443].
Original and layout panels ran in separate windows: their absolute difference
is not an interleaved production speedup. The result weakens a fixed scheduler
algorithm explanation but does not separate code execution, inter-call idle
duration, and host interference. Exact `f220e288` passed local `make check`;
its full performance matrix is still unqualified.

Assembly inspection gives a more specific, exploratory hypothesis. The same
32-byte hot loop in the result accessor starts at `0x5bf0` in the original
candidate, `0x5ba0` in recovered, and `0x201f40` in fixed-layout candidate. Only
the first crosses a 32-byte boundary. This is not proof of a frontend stall or
of causation; code alignment elsewhere also changes. The next control
interleaves original, fixed-layout and general `-falign-loops=32` WF-object
images, keeping runtime sources unchanged and checking every result. If the
general option reproduces convergence, qualify it through normal compiler
output and the full workload/platform matrix before selecting it. If it does
not, do not tune runtime waits on the assumption that this loop explains them.

The [78a84e37 interleaved cohort](https://github.com/mbbill/Whitefoot/actions/runs/34426819037)
completed all fifty observations on an EPYC 9V74 host, not the previous 7763.
Clang 18.1.3 actually moved the accessor loop from `0x5bf0` to the 32-byte
boundary `0x5ca0`. All result/call/participant checks and three byte-identical
replicas passed. Process medians for the same W4/4,096/tile16 cell are:

| Image | Core, microseconds | Full call, microseconds | Voluntary switches per batch | System CPU per batch, milliseconds |
| --- | ---: | ---: | ---: | ---: |
| Original candidate | 23.341 | 118.481 | 146 | 84.788 |
| Fixed-layout candidate | 22.379 | 108.178 | 53 | 21.089 |
| WF-only loop32 candidate | 22.603 | 104.263 | 36 | 7.021 |
| Original recovered | 22.550 | 102.813 | 39 | 5.052 |

Loop32/original candidate paired ratios are 0.9684 [0.9558, 0.9969] for core
and 0.8810 [0.8698, 0.9277] for the full call. Loop32/original recovered ratios
are 0.9950 [0.9773, 1.0109] and 1.0082 [1.0024, 1.0300], respectively. The
original candidate core was already near recovered here; this does not
reproduce or resolve the 7763's large core loss. The WF object's reported
text size grows from 4,183 to 4,327 bytes; whole-image text grows from 27,913
to 28,073 bytes. This is a measured padding cost, not a free optimization.

The next maintained candidate sets 32-byte loop alignment through the shared
host-optimization arguments on x86-64. That existing compiler command compiles
both WF IR and runtime C, so it is deliberately broader than the WF-only
diagnostic and does not inherit its speedup claim. Stack-ledger assembly and
ordinary executable compilation use the same setting. AArch64 is unchanged.
The formal full matrix compares against the current candidate rebuilt with
the previous flags, alongside historical/recovered cores with matched flags.
The fixed-address diagnostic is retired from the active script; its data and
sources remain at the linked revision. Select the candidate only after normal
compiler and all-platform workload checks establish its effects, including
regressions and code-size cost. No wait threshold, ABI or source rule changes.

### Normal compiler alignment candidate b993edb4

The [b993edb4 cohort](https://github.com/mbbill/Whitefoot/actions/runs/34428128502)
completed all five native targets. Exact revision
`b993edb444445339f9e19147b66934e2c0e4aaec` passed local canonical `make check`,
all twelve [gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34428128494),
both [completion jobs](https://github.com/mbbill/Whitefoot/actions/runs/34428128506)
and all four [I/O jobs](https://github.com/mbbill/Whitefoot/actions/runs/34428128500).
Performance acceptance remains open. The formal FIR reducer reports the
following unresolved core-time comparisons, including failed replica checks:

| Native target | Full-window investigate / comparisons | First64 investigate / comparisons | Ordinary CLI screen |
| --- | ---: | ---: | --- |
| Linux x86-64 | 0 / 72 | 14 / 72 | failed |
| Linux AArch64 | 1 / 54 | 6 / 54 | failed |
| macOS x86-64 | 13 / 72 | 26 / 72 | failed |
| macOS AArch64 | 10 / 36 | 15 / 36 | passed |
| Windows x86-64 | 1 / 48 | 9 / 48 | failed |

First64 is a prefix of the same raw calls, not an independent replication.
These counts preserve the existing screen; later exploratory noise analysis
does not turn a failed job into a pass. The full-call and CPU analysis below
also covers costs outside the core-only reducer.

On Linux x86-64 (EPYC 9V74, two reported cores/four SMT CPUs, Clang 18.1.3),
the 450-process formal artifact `10133511311` contains 1,037,250 checked calls.
For W4/4,096 outputs/tile16, current/unaligned paired core time is
0.9507 [0.9386, 0.9796], and full-call time is 0.8843 [0.8760, 0.9051].
Current/historical and current/recovered full-call ratios are respectively
0.9910 [0.9678, 1.0090] and 0.9816 [0.9555, 1.0008]. Across all eighteen
configurations, no paired median full-call, whole-batch CPU or RSS ratio
against unaligned/historical/recovered exceeds 1.05. Some pairs remain noisy;
the fourteen first64 failures are retained. This is the scalar O3 attribution
panel, not an ordinary O2 CLI speedup claim.

Windows artifact `10133568391` (EPYC 7763, two cores/four SMT CPUs) contains
300 processes and 691,500 checked calls. In the same small cell, current/
unaligned core is 1.0089 [0.9599, 1.0259], whereas full call is
0.9190 [0.9159, 0.9322]. Current/historical full call remains
1.0536 [1.0473, 1.0781]. At W4/65,536/tile1024 it is
1.0774 [1.0542, 1.0982], with full-call A/A 1.0170 [0.9944, 1.0302].
The latter core ratio is only 1.0242; most of the remaining measured difference
lies outside that interval. The batch CPU A/A is noisy, and no Windows
context-switch observation exists, so this does not identify a wait primitive
or justify changing one. Peak memory does not show a matching stable increase.

Independent macOS Intel review of artifact `10133625128` verified all 519
manifest hashes, 450 raw reports and both 450-row mean tables. The two current
images are byte-identical. Their full-window core paired medians range from
0.9006 to 1.1106 across cells. W4/4,096/tile64's apparent current/unaligned
1.1018 loss accompanies current/replica 1.1040; replica/unaligned is 1.0311.
This does not establish an alignment regression. A weaker W1/4,096/tile64
lead remains: both aligned images lose about 8–10% in long core/full-call/CPU
medians, four of five pairs, but the current image's first64 direction differs.
The host reports `Macmini6,2`, four logical CPUs and Apple Clang 17.0.0;
physical topology was not captured. No scheduler change follows from this
inconclusive platform screen.

Ordinary Linux x86-64 artifact `10133538103` instead ran on an EPYC 7763.
Independent replay reproduces the summary and failing status from its 1,405
process rows. For
shape4/4,096/W4, current/native-static wall ratios are 3.3534 with the default
policy, 1.4603 with work60000, and 1.6131 with work120000. Corresponding CPU
ratios are 0.8719, 1.0246 and 1.0260. The default diagnostic starts no helpers;
lower work thresholds expose parallelism but do not remove the whole gap.
At 65,536/shape1/W4, all three helpers start and the wall median still reaches
1.3420, with noisy A/A. Native static remains a regular-work reference, not
a dynamic-runtime ceiling. The differing CPUs and optimization levels prohibit
combining the formal and ordinary panels into one alignment speedup.

The quadrature formal/recovered screen in artifact `10133668763` reports
17 investigate cells out of 80, retaining all 2,400 process rows and native
reference observations. Smooth/frontier4/W1 wall is 1.0893 [1.0541, 1.2165]
and process CPU 1.0830 [1.0516, 1.2120]. A one-participant loss is a concrete
reason to inspect generated-code/layout costs before adding scheduling
machinery. Several other failures are tiny or highly variable; none is erased
on that basis. This panel still does not cover Windows quadrature performance.

The next bounded comparison rebuilds the prior official compiler `b00bf240`
on each native CI host and runs ordinary `--par --no-vectorize` commands with
both versions. It changes neither source nor runtime to obtain the baseline.
All default-policy cells and the existing four-leaf cell are interleaved;
the latter has a byte-identical current replica under the same work policy.
Preserve default/native losses and treat noisy current/previous comparisons
as unresolved. Select the x86 alignment setting only with normal-path benefit
and explained regressions, rather than inheriting the O3 panel's conclusion.
Local M1 validation completed all 1,625 commands and all three missing-sample
negative checks; the reducer retained performance failures. Synthetic reducer
checks distinguish quiet/noisy four-leaf A/A from default-policy A/A and cover
one/two/four participants plus operation without the optional baseline. These
are harness checks, not native CI qualification or additional performance data.

### Ordinary compiler paired qualification at 7776c3cd

The [7776c3cd cohort](https://github.com/mbbill/Whitefoot/actions/runs/34430234703)
completed all nineteen compute jobs. Exact revision
`7776c3cdb4e1b72876e062b3d16f019328b5d2d7` passed local canonical `make check`.
The [repository gate](https://github.com/mbbill/Whitefoot/actions/runs/34430234778)
has eleven successful jobs and one cancelled macOS research job; the
[Linux and Windows completion jobs](https://github.com/mbbill/Whitefoot/actions/runs/34430234877)
passed. A cancelled job is not a successful all-platform gate. Performance
qualification remains open:

| Native target | Full-window investigate / comparisons | First64 investigate / comparisons | Ordinary CLI screen |
| --- | ---: | ---: | --- |
| Linux x86-64 | 0 / 72 | 11 / 72 | failed |
| Linux AArch64 | 2 / 54 | 4 / 54 | failed |
| macOS x86-64 | 13 / 72 | 28 / 72 | failed |
| macOS AArch64 | 9 / 36 | 15 / 36 | passed |
| Windows x86-64 | 1 / 48 | 3 / 48 | failed |

The counts are the existing core-time screen, including replica comparisons;
first64 overlaps the full window. They do not establish full-call parity or
permit discarding failed observations as noise.

Linux formal artifact `10134294731` ran on an EPYC 7763, reproducing the CPU
model with the earlier large gap. Independent replay checked all 450 processes
and 1,037,250 calls, source identity and both mean tables. At W4/4,096/tile16,
current/unaligned paired core time is 0.5561 [0.5422, 0.5990] and full call is
0.7075 [0.6950, 0.7207]. Current/recovered ratios are respectively
0.9785 [0.9764, 1.0076] and 0.9947 [0.9829, 1.0105]; current/replica full call
is 0.9989 [0.9816, 1.0174]. Across all eighteen configurations, no median
full-call, batch CPU or RSS ratio against unaligned/historical/recovered
exceeds 1.05. Whole-image text grows from 27,913 to 28,601 bytes. This supports
the general loop-placement change on this O3 workload; it does not isolate a
particular CPU frontend mechanism or establish ordinary O2 CLI improvement.

The ordinary Linux x86-64 artifact `10134301411` ran on a different host,
Xeon 8370C. Independent replay verified all 1,625 rows and reproduced the
failing summary. Against the prior official compiler `b00bf240`, twenty-nine
of forty-two default-policy wall medians exceed one, twelve in all five pairs.
Examples are shape1/4,096/W1 at 1.0320 [1.0247, 1.0374] and shape4/4,096/W4
at 1.0188 [1.0144, 1.0307]. The four-leaf wall ratio
1.1798 [0.9562, 1.3571] has noisy same-policy A/A, so it does not identify a
stable runtime regression. Text grows from 20,049 to 20,529 bytes. These
small repeatable losses remain costs; there is no general normal-CLI net-win
claim for loop alignment.

Linux AArch64 artifact `10134276221` instead has byte-identical current,
previous and replica executables. Its four-leaf current/previous wall ratio
is 1.1342 [0.8353, 1.2696], with own-policy A/A
1.1517 [1.0242, 1.2959]. This supplies a direct noise control, not a reason to
assume all ARM observations are noise. macOS AArch64 artifact `10134357602`
has identical current/previous `__text` bytes but different whole-file hashes;
the remaining image difference has not been classified.

Windows formal artifact `10134427787` retains its sources, binaries and raw
measurements, but its recorded `manifest.sha256` is empty. The
[job log](https://github.com/mbbill/Whitefoot/actions/runs/34430234703/job/102724030775)
reports `find: 'shasum': No such file or directory`. It therefore has an
additional provenance-generation failure, independent of its performance
screen. Do not claim a verified original manifest or replace it after the
fact. The next run uses the same Windows `sha256sum` path as the ordinary CLI
panel and requires a nonempty manifest.

Independent raw replay nevertheless verifies its 300 processes, 691,200 warm
calls, both mean tables, and local candidate/replica byte equality. At
W4/4,096/tile1024, current/historical core is 1.0202 [0.9945, 1.0277], full
call is 1.1221 [1.1156, 1.1275], and batch CPU is 1.1111 [1.0833, 1.1765].
Full-call A/A is 1.0023 [1.0003, 1.0093], with CPU median 1.0. The full-call
loss therefore exceeds the observed A/A spread in this cell. Its larger
outside-core component includes prefix preparation, result materialization
and release; it does not isolate a scheduler operation. The earlier
W4/65,536/tile1024 loss does not repeat clearly: full call is now
1.0312 [0.8714, 1.0879].

macOS Intel ordinary artifact `10134474673` contains 1,625 checked raw rows
and 1,062 exactly reproduced summary rows. No default current/previous cell
has wall time more than 5% worse in all five pairs. The same-policy four-leaf
wall/CPU medians are 0.9936/0.9939, with own A/A 1.0039/1.0071. Wall A/A ranges
from 0.9576 to 1.0566, so this four-leaf comparison remains `noisy-open`.
Against native static, its wall median is 1.0446 [0.9478, 1.0909], while RSS is 1.2458 and
higher in every pair. Fourteen of fifteen manifest entries are verifiable;
the current compiler binary was outside the uploaded directory. The next
ordinary build retains that compiler beside the already retained prior one,
so both recorded hashes can be checked from the artifact.

Quadrature artifact `10134438790` retains all 2,400 processes and reports
22 investigate WF cells out of 80. The prior W1 smooth/frontier4 and
left-peak/frontier4 signals become 0.9572 and 1.0032 with mixed pair directions.
Native-only controls still have same-direction cross-image losses at W1:
smooth/Rayon with spawn depth 4 is 1.1300 and left-peak/Parlay-left with
spawn depth 8 is 1.2169. This is not a measured runtime fix: scheduler/host
sources and the selected WF objects match the
previous cohort, but full images and native objects differ, the CPU changes
from EPYC 9V74 to 7763, and Rust changes from 1.98.0 to 1.98.1. Layout and
sampling remain unseparated; a larger failure count alone does not identify
an algorithmic regression.

### Compact waiting metadata candidate (not retained)

The maintained runtime permits only the offering thread to join and release
a slot, and the slot's `home` remains immutable until process exit. Its waiter
pointer therefore represents only two states: no waiter or `home`. Replacing
that redundant pointer with an atomic integer flag preserves the existing
SC registration, DONE publication and final waiting check, including the
relaxed clear under the owner wait lock. No new common-path RMW is introduced.
Local M1 debug layout changes from 304 to 288 bytes per slot; payload capacity
remains 256 bytes with sixteen-byte alignment. The ordinary task ABI, source
rules, worker configuration and native wait primitives do not change.

The alternatives are to keep the pointer or encode waiting in the completion
state with an exchange. The flag was provisional: it removes redundant storage
without adding an RMW to every completion. The exchange alternative is not
selected because its ARM cost is unmeasured and the current protocol already
provides the necessary ordering. After DONE, an old executor may see the new
generation's waiting flag and signal immutable `home`; the owner must recheck
the new generation's completion under its wait lock.

A deterministic smoke case forces registration, DONE, slot reuse and the old
notification while the new task remains pending. Its witness is ordered by
the actual notification under the wait lock, so an earlier spurious return
cannot satisfy it. Local ordinary, TSan and ASan/UBSan smoke checks and the
embedded compiler scheduler tests pass. The existing 200,000-operation deque
stress also passes with statistics enabled and disabled. Independent source
and test review closed two test-handshake findings. The subsequent native
Windows run passes this reuse case and both deque-statistics configurations;
all-platform performance remains unqualified below.

Local ordinary compilation with both the candidate and prior `7776c3cd`
passes the same 48-input oracle matrix at widths 1/2/4 and split-work settings
0/60,000/240,000/1,200,000. The local formal matrix completed all 450 processes
and retained its noisy performance failures. These establish execution and
harness coverage on M1, not native CI performance qualification.

The a8227af4 formal matrix compares the candidate with exact prior core
`7776c3cd` using the same WF/host objects, flags and platform leaves. The
ordinary matrix rebuilds that same prior official compiler and uses normal
CLI commands with matched default and four-leaf policies. Retain the historical,
recovered, native and A/A controls. Accept the flag only with preserved
correctness and no unexplained repeatable performance loss on any native
target; retain failed/noisy cells as open. Smaller storage alone is not a
speedup claim. The earlier alignment decision and existing workload losses
remain unresolved obligations, not benefits inherited by this candidate.

### Native qualification at a8227af4

Exact `a8227af4ed382a881b6c85f53e9553056f634a60` passes local canonical
`make check`. Its [compute cohort](https://github.com/mbbill/Whitefoot/actions/runs/34432805054)
completes all nineteen jobs, retaining the performance failures. The
[repository gate](https://github.com/mbbill/Whitefoot/actions/runs/34432805042)
has eleven successes and one cancelled macOS research job; both
[native I/O jobs](https://github.com/mbbill/Whitefoot/actions/runs/34432805064)
and all four [I/O benchmark jobs](https://github.com/mbbill/Whitefoot/actions/runs/34432805117)
pass. This is not an all-platform acceptance result.

| Native target | Full-window investigate / comparisons | First64 investigate / comparisons | Ordinary CLI reducer |
| --- | ---: | ---: | --- |
| Linux x86-64 | 0 / 90 | 11 / 90 | failed |
| Linux AArch64 | 2 / 72 | 9 / 72 | failed |
| macOS x86-64 | 22 / 90 | 22 / 90 | failed |
| macOS AArch64 | 11 / 48 | 15 / 48 | passed, with unresolved noisy cells |
| Windows x86-64 | 2 / 48 | 18 / 48 | failed |

These are core-time screen counts, including identical-image comparisons;
first64 overlaps the full window. Independent replay verifies the five formal
artifacts' manifests, source identities, call sequences and both mean tables.
The five ordinary artifacts now retain both compilers and verify all fifteen
manifest entries each; their raw rows and existing reducers also reproduce.
Neither reducer success nor same-version parity qualifies a noisy cell or
closes a loss against a stronger reference.

Linux x86-64 formal artifact `10135204595` verifies 540 processes and
1,244,700 calls. At W4/4,096/tile16, current/recovered full call is
0.9761 [0.9669, 1.0054], current/previous is 0.9716 [0.9582, 1.0037],
and current/replica is 0.9935 [0.9802, 1.0163]. No core or full-call median
against previous exceeds 1.05 across its eighteen configurations.

Linux AArch64 artifact `10135227185` verifies 450 processes and 1,037,250
calls. At 65,536/tile1024, current/previous full-call ratios are 1.0520
[1.0491, 1.0536], 1.0688 [1.0668, 1.0787] and 1.0896 [1.0800, 1.1137]
at W1/2/4; corresponding A/A medians are about 1.0005/1.0000/1.0012.
The W1 path executes the sequential WF entry, with no scheduler acquire,
publish, join, release or slot access. Shared WF/native/host instruction differences occur
only at relocations, while WF accessors move by 216 bytes. This rules out
execution of the waiting flag as the W1 cause, but does not establish a
layout explanation or excuse the measured full-call loss.

Windows artifact `10135217004` verifies 388 manifest entries, 300 processes
and 691,200 warm calls. At W4/4,096/tile1024, current/historical core is
0.9913, full call 1.1745 [1.1616, 1.1797], and batch CPU 1.2000
[1.1190, 1.3000]. Current/previous full call is 0.9864 [0.9623, 1.0388].
The per-process full-minus-core means have medians 32.791 microseconds for
current and 25.909 for historical; their paired ratio is 1.2681
[1.2488, 1.2870]. That interval includes preparation, result access and
release, with possible interference from workers; it is not a measurement
of a wait primitive. Current and previous shared functions occupy identical
addresses and differ only in relocation bytes. Historical shared code has
a different placement, so neither waiting policy nor layout is isolated.

macOS artifacts `10135293030` (ARM) and `10135354909` (Intel) verify 300 and
540 processes. ARM W1/4,096/tile64 current/recovered full call is 1.1084
[1.0465, 1.2652], with A/A 1.0756 [1.0419, 1.2681]; the replica/recovered
ratio is only 1.0200. Intel W4/65,536/tile1024 current/recovered is 1.0978
[1.0177, 1.2806], with A/A 1.0441 [0.9114, 1.0888]. Neither result can
be discarded as a pass or attributed to the waiting flag. Ordinary ARM's
successful reducer still contains a noisy shape0/4,096/W2 current/previous
wall ratio of 1.0876 [1.0510, 1.1090], with A/A 1.0687
[0.9803, 1.1294]. Ordinary Intel has a noisy shape1/65,536/W4 ratio
of 1.2452 [1.1553, 1.4307], with A/A 1.3100 [0.8754, 1.4723].

The ordinary Linux x86-64 four-leaf shape4/4,096/W4 control is near previous
at 0.9865 [0.9403, 1.1439], but still loses to native static by 1.4217
[1.3615, 1.6460]. Matching a prior compiler does not erase that application
gap. The quadrature artifact `10135238972` verifies all 2,400 processes and
reproduces 23 investigate WF cells out of 80. It has no WF cell with both a
wall median above 1.05 and all five pairs slower, but broad ranges and
native-only cross-image variation prevent acceptance. Those native controls
are not byte-identical A/A and cannot quantify sampling noise alone.

Native debug layouts confirm slot size 304 to 288 bytes; ordinary Linux ARM
BSS falls by 65,536 bytes. The ordinary Windows/ARM four-leaf paired RSS
medians are both 1.0. No stable speed or peak-resident-memory improvement is
established. These measurements do not qualify the compact flag; the return
to the prior representation below avoids extending this storage experiment.

The cancelled macOS research job rebuilds the compiler in two target
directories: the compute experiment first uses its private target, then the
container tests rebuild the same compiler under `compiler/target`. The second
build is interrupted at the workflow's eight-minute limit. The root research
target now builds the ordinary compiler once and supplies that executable to
the compute tests, allowing container tests to reuse the same Cargo output.
All research test targets and the CI time limit remain unchanged. The unchanged
local `make research-tests` passes with the supplied ordinary compiler; the
container stage's Cargo invocation reuses the same output in 0.00 seconds.
Exact `e46396127c82d889180ddf88eaed2044c9f315c9` subsequently passes all twelve
[native gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34435318201)
and both [completion jobs](https://github.com/mbbill/Whitefoot/actions/runs/34435318197).
The macOS research log builds the compiler once in 56.20 seconds; the container
stage reuses it in 0.01 seconds. This verifies the build-reuse fix, not the
later waiting-representation or Windows reference changes below.

### Restore the prior waiting representation

The compact flag fails its stated condition: it has no established speed or
peak-RSS benefit and retains repeatable full-call losses against its parent.
The sequential ARM evidence rules out executing the flag as that cell's cause;
it does not make the changed executable's performance acceptable. Restore the
`7776c3cd` waiter pointer and its exact registration/publication memory orders.
This returns each slot to 304 bytes, with the same 256-byte payload and
sixteen-byte alignment. Retain the new deterministic registered-wait/reuse
case, generation capture before DONE and notification witness under the owner
lock. No safety fix, source rule, public ABI, publication policy or I/O path
is removed. Keeping the flag and searching more layouts is not selected: the
optional storage saving does not justify prolonging runtime qualification.

The existing formal and ordinary comparisons now use exact `a8227af4` as
their immediate previous control, alongside the unchanged historical,
recovered, native and identical-image controls. Reversing the representation
does not by itself establish a performance recovery; full-call/CPU/memory
comparisons and all native correctness checks remain required.

Local ordinary, ThreadSanitizer and ASan/UBSan smoke checks pass with waiter
pointers, including the retained registered-wait/reuse witness. The concurrent
200,000-operation deque checks pass with and without counters, including TSan;
both embedded compiler scheduler tests also pass. Independent review verifies
that the production tokens, after excluding comments and test hooks, match
`7776c3cd`; same-input-path Apple Clang O2 assembly is byte-identical. These
checks do not replace native CI or exact-revision canonical validation.

Independent inspection also finds an actual Windows ordinary-reference build
asymmetry in artifact `10135281908`: `native.pdb` records `-debug`, with no
explicit OPT/INCREMENTAL overrides, and contains 2,826 incremental trampolines.
The WF par/previous/seq executables have no CodeView/PDB entry and the ordinary
compiler passes no `-g`. Microsoft's
[/DEBUG documentation](https://learn.microsoft.com/en-us/cpp/build/reference/debug-generate-debug-info?view=msvc-170)
confirms that it enables incremental linking and changes the REF/ICF defaults.
The native reference now explicitly restores `/INCREMENTAL:NO /OPT:REF
/OPT:ICF` while retaining PDB evidence. This corrects the reference's release
link settings, not the WF runtime. No speedup is inferred; native correctness
and new Windows binary metadata must verify the actual result. Keep the former
measurements with their debug-link limitation. The shared measurement runner,
POSIX flags and the mutually matched formal O3 images are unchanged.

### Native qualification at a130886a

Exact `a130886a7f1bfe85326476b86f2ebb6dc7c58b95` passes local canonical
`make check`, all twelve [repository gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34436252124),
both [native completion jobs](https://github.com/mbbill/Whitefoot/actions/runs/34436252103),
and all four [I/O benchmark jobs](https://github.com/mbbill/Whitefoot/actions/runs/34436252114).
The [compute cohort](https://github.com/mbbill/Whitefoot/actions/runs/34436252146)
finishes with nine successful and ten failed jobs. Performance qualification
remains incomplete; the runtime is held unchanged while these losses are
classified.

All five formal artifacts have verified manifests, source identities,
byte-identical replicas, complete raw matrices and reproduced means/reducers.
Windows source comparison normalizes checkout CRLF only; its manifest verifies
the original bytes. The counts below concern core-time screens, including
identical-image comparisons; first64 overlaps the full window.

| Native target | Artifact | Full-window investigate | First64 investigate | Ordinary CLI reducer |
| --- | ---: | ---: | ---: | --- |
| Linux x86-64 | 10136376116 | 0 / 90 | 21 / 90 | failed |
| Linux AArch64 | 10136361436 | 0 / 72 | 4 / 72 | failed |
| macOS x86-64 | 10136574594 | 25 / 90 | 32 / 90 | failed; artifact upload failed |
| macOS AArch64 | 10136475961 | 15 / 48 | 13 / 48 | passed, with unresolved noisy cells |
| Windows x86-64 | 10136382206 | 5 / 48 | 6 / 48 | failed |

Linux ARM's previously regressed 65,536/tile1024 full-call ratios against
`a8227af4` are now 0.9491, 0.9339 and 0.9195 at W1/2/4; against research
they are 0.9984, 0.9992 and 1.0003. All eighteen configurations have full-call
medians against historical/research/previous at most 1.05. The recovered
performance supports rejecting the compact representation; it does not prove
that executing a pointer wait is intrinsically faster, especially on the W1
path that never executes the scheduler.

Linux x64's eighteen full-call medians against previous range from 0.9798 to
1.0224; against historical/research none exceeds 1.02. At W4/4,096/tile16,
full-call/research is 0.9831 [0.9737, 0.9973], CPU is 0.9745 and full-call
A/A is 0.9889. Short-window failures remain, including W1/4,096/tile64
first64 full-call/historical 1.3542 with A/A 1.2885 and replica/historical
1.0132.

Windows verifies 388 manifest entries, 300 processes and 691,200 warm calls.
At W4/4,096/tile1024, full-call/historical is 1.1857 [1.1360, 2.2987],
core/historical 1.0219 [0.9257, 3.3385], and full-call/previous 1.0220
[0.9353, 1.9904]. The paired ratios of per-process full-minus-core means
are 1.2753 [1.2698, 1.6688] against historical and 0.9970
[0.9833, 1.0162] against the identical replica. This preserves the measured
loss outside the core interval, which includes preparation, result access and
release; it does not identify a wait primitive or the representation reversal
as its cause. W2 in that configuration also loses full-call/historical by
1.2083 [1.0861, 1.3425].

Neither macOS panel is qualified. Intel W2/65,536/tile1024 full-call/previous
is 1.0952 [1.0110, 1.1682], with A/A 1.0724 [0.7918, 1.1539] and
replica/previous 1.0433 [0.9491, 1.2867]. ARM W2/4,096/tile16
full-call/research is 1.1654 [0.9765, 1.2696], while A/A is 1.0501
[1.0019, 1.2422]. These are unresolved losses, not proof of a new runtime
defect or permission to discard the cells.

Four ordinary-command artifacts reproduce their raw matrices and reducers:
Linux x64 `10136362468`, Linux ARM `10136397091`, macOS ARM `10136379628`
and Windows `10136515258`. None has a default current/previous wall or CPU
cell with all five ratios above 1.05; this does not resolve application gaps.
Linux x64 shape4/4,096/W4 default/static remains 3.3446
[3.3329, 3.3641], with diagnostics confirming no helpers started. Its existing
four-leaf policy reduces that ratio to 1.4892 [1.4522, 1.6748], with CPU
1.0179. Windows four-leaf/static RSS is 1.1865 [1.1843, 1.1878]. The Intel
macOS job passes both compilers' correctness matrices and finishes the screen,
but artifact creation times out after five upload attempts; its raw results
cannot be independently replayed from this run.

The Windows ordinary native PDB matches its executable's GUID/age and records
`/INCREMENTAL:NO /OPT:REF /OPT:ICF`. It is nonincremental with zero trampolines,
versus 2,826 in the earlier debug-link reference. This verifies the reference
fix on Clang 20.1.8. The two Windows cohorts use different CPUs, so no
cross-cohort speedup is attributed to those flags.

Quadrature artifact `10136413144` verifies all eight manifest entries and
2,400 raw processes, including 633,600 calls with warmups. Its original
reducer reproduces 13/80 investigate cells. No WF cell has wall or CPU ratios
above 1.05 in every pair, but smooth/frontier W4/depth8 still has wall
1.0825 [1.0025, 1.2623] and CPU 2.0335 [0.9467, 3.7368]. Native cross-image
variation is not a byte-identical A/A control; qualification remains open.

The next bounded Windows diagnostic reuses each cohort's old/candidate/replica
binaries with 4,096 outputs in one leaf. The emitted leaf branch bypasses
all compute acquire/publish/join calls; actual-lane observations must confirm
no helpers started. A persisting difference would exclude executing task
scheduling as its cause. Disappearance would not identify that cause because
tree depth also changes. This adds no production change, instrumentation,
link option or replacement for the existing parallel acceptance matrix.
The added shell block runs locally against the native a130 macOS ARM images:
all fifteen processes and 61,455 calls pass, each with one actual lane.
Wrong-lane and missing-lane reports reject. This validates the diagnostic's
execution and checks; Windows execution and its performance result remain open.

### Windows execution attribution at 9253ceb3

Exact `9253ceb3db748a6569a031f0db27d16ee02362f6` passes local canonical
`make check`, all twelve [gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34438394092)
and both [native completion jobs](https://github.com/mbbill/Whitefoot/actions/runs/34438394162).
Its [compute cohort](https://github.com/mbbill/Whitefoot/actions/runs/34438394140)
finishes with eight successful and eleven failed jobs. The `compiler/` tree is
unchanged from a130; neither the failures nor the changed ratios establish a
new production regression or improvement.

Windows artifact `10137129963` verifies 404 manifest entries and all 315
processes/752,955 calls, including the one-leaf diagnostic. Its Xeon 8573C
differs from a130's EPYC 7763. Ratios below are paired current/historical
medians and five-pair ranges, on this host only:

| Input and actual participants | Full call | Full-minus-core interval |
| --- | --- | --- |
| Four leaves, W4 | 1.0767 [1.0567, 1.0781] | 1.1186 [1.0885, 1.1413] |
| One leaf, W1 candidate | 1.0428 [1.0206, 1.0660] | 1.1041 [1.0482, 1.1323] |
| One leaf, W1 replica | 1.0644 [1.0423, 1.0921] | 1.1272 [1.0720, 1.1974] |

The one-leaf images request four participants but actually start no helpers;
the emitted leaf branch executes no task operations. Both identical current
images retain a difference outside the core. An old/current difference can
therefore persist without executing task scheduling; this does not determine
how much of the four-leaf loss has the same cause. Its precise size is not a
fixed cost: one-leaf full-call A/A is 0.9792 [0.9675, 0.9994], and outer
A/A is 0.9463 [0.9352, 1.0182]. Four-leaf full-call A/A is tighter at
0.9992 [0.9888, 1.0010]; that original loss remains unresolved. Neither
subtracting the leaf ratios nor comparing the two CPU cohorts isolates a
scheduler cost.

The returning paths of `wf_research_fir_get` have identical leading 89 bytes
in the old and candidate Windows images; their addresses differ by 416 bytes.
The following abort-call relocation differs. This excludes extra getter
instructions on those paths, not code placement, cache effects, heap history
or costs elsewhere in the full-minus-core interval. It is not grounds for
changing the scheduler's waiting policy or searching production layouts.

The ordinary Intel macOS artifact now uploads successfully as `10137166008`;
it is new evidence for unchanged compiler sources, not recovery of a130's
missing raw data. It and Windows artifact `10137165293` each verify all
fifteen manifest entries, 1,625 raw processes and 1,062 reproduced summary
rows. Both reducers still fail. Neither has a default/previous wall or CPU
cell with all five ratios above 1.05. Windows shape4/4,096/W4 under the
existing four-leaf policy has wall/native-static 1.0225 [0.7024, 1.0380]
and RSS 1.1863 [1.1854, 1.1901]; the default policy's wall ratio remains
3.0921 [2.0793, 3.1126]. Intel four-leaf A/A is 1.2057
[1.0074, 2.0012], so its timing remains unqualified.

Mac ARM/Intel formal artifacts `10137264744`/`10137239933` also reproduce
their full matrices and reducers: long core investigate counts 15/48 and
14/90, first64 counts 19/48 and 26/90. Intel W4/65,536/tile64
full-call/historical is 1.1334 [1.0735, 1.3977], with A/A 1.0092
[0.9798, 1.2285] and replica/historical 1.1107 [0.9945, 1.2430]. This
loss remains open. ARM has no full-call cell with all five ratios above 1.05;
its noisy cells still do not qualify performance.

The one-leaf control has answered its limited question, but changes tree
depth. Replace it in place with `wf-seq`: the existing compiler-emitted
sequential entry, 4,096 outputs and tile1024, preserving the four-leaf tree.
The C host selects that entry outside the timed intervals; no runtime or
language path is added. Actual-lane and selected-world checks must confirm
sequential execution. This next control can test the original representation
without task execution; the ordinary parallel matrix remains authoritative.
Local macOS ARM execution of the exact added shell block verifies fifteen
processes and 61,455 calls, with all raw means reproduced. The guard rejects
wrong execution worlds, wrong/missing lane counts and missing headers. Both
historical and candidate images still pass the ordinary four-lane parallel
smoke. The emitted sequential recursion retains the branch/leaf constructors
and contains no task operations. Native Windows timing remains unverified.

### Same-tree control and five-target results at 7db76f23

Exact `7db76f238ab653b82266e74adcfeb5ea24b743a7` passes local canonical
`make check`, all twelve [gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34440572321)
and both [native completion jobs](https://github.com/mbbill/Whitefoot/actions/runs/34440572381).
The local gate uses the existing pinned `TBB_SOURCE` and `PARLAY_SOURCE`
checkouts; its initial invocation omitted those paths and stopped at dependency
discovery. Restoring the verified paths required no source or test change.
The [compute cohort](https://github.com/mbbill/Whitefoot/actions/runs/34440572427)
finishes with nine successful and ten failed jobs. Only macOS ARM passes the
ordinary-command job; the other four ordinary jobs and quadrature remain
failed. The formal results below do not qualify those application comparisons.

All five formal artifacts independently reproduce their raw matrices, means
and original reducers, with captured source identities and identical-image
replicas verified. These counts are core-time screens, including A/A controls;
the first64 window overlaps the long window and is not an independent sample.

| Target | Artifact | Long investigate | First64 investigate |
| --- | ---: | ---: | ---: |
| Linux x86-64 | 10137899684 | 1 / 90 | 15 / 90 |
| Linux AArch64 | 10137944975 | 2 / 72 | 5 / 72 |
| macOS AArch64 | 10137976134 | 15 / 48 | 16 / 48 |
| macOS x86-64 | 10138195492 | 17 / 90 | 33 / 90 |
| Windows x86-64 | 10137975056 | 0 / 48 | 5 / 48 |

The Windows artifact verifies 404 manifest entries and 315 processes/752,955
calls. Its fifteen new sequential processes all preserve the four-leaf tree,
select `wf-seq`, request four participants and observe one actual lane.
On this Xeon 8573C/Clang 20.1.8 host, paired current/historical ratios are:

| N4096/tile1024 execution | Core | Full call | Batch CPU |
| --- | --- | --- | --- |
| Sequential | 0.9586 [0.8851, 1.0004] | 0.9001 [0.8474, 0.9480] | 0.8571 [0.8571, 1.0000] |
| Four participants | 1.0165 [0.9654, 1.0280] | 0.9496 [0.9248, 0.9639] | 0.9167 [0.8750, 0.9375] |

Full-call A/A is respectively 1.0033 [0.9659, 1.0379] and 1.0005
[0.9773, 1.0272]. The former four-leaf full-call disadvantage does not
reproduce in this cohort. The runtime and compiler are unchanged from a130;
the host selection code and linked images changed. Neither the faster ratios
nor the reversal of the earlier ranking establish a runtime improvement or
identify code placement, heap history or another specific cause. This control
does not justify subtracting sequential from parallel ratios as scheduler cost.

Windows still has W4/N4096/tile16 full-call/previous 1.0415
[1.0309, 1.0512], CPU/previous 1.0458 [1.0131, 1.0604], and full-call
A/A 1.0038 [0.9912, 1.0186]. Replica/previous full call is also 1.0417
[1.0120, 1.0605]. Preserve this approximately four-percent difference even
though it is below the original threshold. W2/N65536/tile1024 CPU/previous
is 1.1136 [1.0426, 1.2326], but CPU A/A is 0.9800 [0.9800, 1.2927]
and replica/previous crosses one. CPU timing is coarse on this host; that
cell remains unresolved. No Windows full-call, CPU or RSS cell exceeds
historical/previous by five percent in every pair.

Linux x64 has a reproducible full-call loss at W4/N4096/tile64 against research:
1.1139 [1.0860, 1.1227], with CPU 1.1080 [1.0624, 1.1138]. Full-call
A/A is 1.0152 [0.9957, 1.0287], and replica/research is also slower at
1.0914 [1.0836, 1.1199]. Core/research is only 1.0169
[1.0081, 1.0404]; full-minus-core is 1.1369 [1.1076, 1.1492]. This
EPYC 7763/Clang 18.1.3 result prevents declaring parity from core alone.
The same cell's full-call/historical and full-call/previous medians are
1.0197 and 1.0326. No other cell has a full-call, CPU or RSS median more
than five percent above historical/research/previous.

Linux ARM has no full-call, CPU or RSS cell more than two percent above
historical/research/previous in every pair. N65536/tile1024 full-call/research at W1/2/4 is
0.9976, 1.0030 and 0.9986. W4/N4096/tile1024 core/historical is 1.0690
[0.9473, 1.1186], with core A/A 1.0330 [0.9735, 1.1630]; its original
failed screen remains open rather than being relabeled parity.

Neither Mac panel qualifies. ARM W2/N65536/tile16 full-call/previous is
1.1287 [1.0766, 1.1949], but A/A is 1.0638 [0.8151, 1.1886] and
replica/previous crosses one. Intel W2/N4096/tile64 full-call/research is
1.2416 [0.9428, 1.4524], with A/A 1.0625 [1.0207, 1.2930]. The earlier
Intel W4/N65536/tile64 loss also does not reproduce: full-call/historical is
0.9731 [0.8958, 1.0193]. These are changed observations under unchanged
production sources, not evidence that the host selector optimized the runtime.
The Linux x64 full-call loss and the noisy Mac comparisons still require
qualification; the Windows result neither explains nor removes them.

### Same-tree Linux attribution at bc646d58

At exact `bc646d580e5f5185096344577a0ada82c0fdc323`, all twelve
[gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34442812576)
and both [native completion jobs](https://github.com/mbbill/Whitefoot/actions/runs/34442812604)
pass. The [compute cohort](https://github.com/mbbill/Whitefoot/actions/runs/34442812625)
finishes with nine successes and ten failures: all five formal jobs, four
ordinary-command jobs and quadrature still fail. These results do not complete
performance qualification.

All five formal artifacts independently reproduce their raw means and original
reducers; captured sources, historical repairs and identical-image replicas
match. Counts below are the original core-time screens, not full-call verdicts.
The first64 samples overlap the long batches.

| Target | Artifact | Long investigate | First64 investigate |
| --- | ---: | ---: | ---: |
| Linux x86-64 | 10138666596 | 0 / 90 | 15 / 90 |
| Linux AArch64 | 10138668831 | 0 / 72 | 4 / 72 |
| macOS AArch64 | 10138630633 | 13 / 48 | 20 / 48 |
| macOS x86-64 | 10138759566 | 16 / 90 | 34 / 90 |
| Windows x86-64 | 10138676640 | 7 / 48 | 3 / 48 |

The Linux x64 diagnostic invokes the existing `wf-seq` entry with the same
N4096/tile64 tree, using five interleaved processes for each of
old/research/current/replica. All twenty sequential processes request four
participants and observe one actual lane. The pre-recorded discriminator is
whether the difference persists without task execution; disappearance leaves
parallel activity as a candidate without identifying its particular cost.
On this EPYC 7763/Clang 18.1.3 host, paired current/research ratios are:

| Same 64-leaf tree | Core | Full call | Batch CPU |
| --- | --- | --- | --- |
| Sequential entry | 0.9793 [0.9711, 1.0311] | 0.9835 [0.9753, 1.0464] | 0.9835 [0.9752, 1.0464] |
| Four participants | 0.9902 [0.9669, 1.0025] | 1.0596 [1.0358, 1.0847] | 1.0696 [1.0366, 1.0908] |

Full-call A/A is respectively 1.0051 [0.9853, 1.0186] and 0.9828
[0.9612, 0.9992]. Parallel replica/research also loses: full call 1.0776
[1.0473, 1.1077], CPU 1.0740 [1.0580, 1.1056]. The difference remains
principally outside the core interval. That interval boundary excludes prefix
preparation, accessor-based result materialization, next-history extraction
and destruction; aggregate CPU includes workers searching between calls.
This is not a measurement of scheduler cost alone. Sequential parity does not
isolate background-worker interference, allocation history or code placement.

The retained Linux WF/host/native objects and all five aligned runtime images
are byte-identical to 7db, including executable-section addresses. The changed
ratios therefore are not a compiler or runtime improvement. This FIR entry
does not call `split_budget`; the current and research result-accessor loops
have equal instructions and both fit within a 32-byte boundary. Neither fact
identifies the remaining loss or supports another grain or layout change.

Linux ARM has no full-call or CPU median above 1.0124 relative to
historical/research/previous, and no full-call/CPU/RSS cell exceeds 1.02 in
every pair. Its noisy short-window failures and RSS variation remain open.
Neither Mac qualifies: ARM W2/N4096/tile1024 full-call/previous is 1.1946
[0.9656, 1.7407], with A/A 1.0797 [0.9056, 1.2431]; replica/previous is
1.1017 [1.0662, 1.4003], so the entire cell cannot be dismissed as noise.
Intel W2/N4096/tile16 full-call/historical is 1.1559 [1.0267, 1.1734],
but replica/historical crosses one. Intel W4/N4096/tile16 full-call A/A
itself is 1.0710 [1.0521, 1.4346], preventing a resolved ranking.

Windows has no full-call/CPU/RSS cell more than five percent above
historical/previous in every pair. W4/N4096/tile1024 full-call/historical
is 0.9200 [0.9017, 0.9214]. Its sequential four-leaf control is unstable:
current/historical 0.9640 [0.8874, 1.0550], replica/historical 1.0789
[0.9170, 1.0987], and A/A 0.9678 [0.8591, 0.9866]. The host changed from
Xeon 8573C to EPYC 9V74; all executable-section addresses and bytes in the
checked WF/host/native objects and four aligned images match 7db, although
the complete object/EXE files differ. No runtime improvement follows from
this cohort's changed ratios, and sequential equivalence is not established.

### Existing-steal observation at 4fabd264

The [Linux observation](https://github.com/mbbill/Whitefoot/actions/runs/34445055343)
at `4fabd26448eddc9316499f5c58e66c556d9ca7cb` uses the same four
old/research/current/replica images at N4096/tile64 with `WF_SCHED_REPORT=2`.
Both POSIX controls and production already count successful steals; the setting
registers an exit-time report. Artifact `10139449436` verifies 658 manifest
entries and all twenty observation processes/81,940 calls. The selected original
and sequential controls bring the reviewed scope to seventy processes/286,790
calls. The original long/first64 readers still flag 6/90 and 25/90 cells.

The host is now Xeon 6973P-C, with two cores/four SMT CPUs. All checked
WF/host/native objects and five aligned runtime images are byte-identical to
bc, but this host does not reproduce the EPYC full-call loss: current/research
is 1.0019 [0.9575, 1.0060] in the original matrix and 1.0010
[0.9750, 1.0134] in the observation. Observation CPU/research is 0.9964
[0.9605, 1.0331]. Successful-steal totals, including all 4,097 calls, are:

| Image | Median [minimum, maximum] |
| --- | --- |
| Current | 34,260 [33,024, 35,213] |
| Research | 33,353 [31,946, 34,177] |
| Replica | 33,857 [33,691, 35,153] |

Paired current/research count ratios are 1.0337 [0.9872, 1.0677], with
A/A 1.0119 [0.9598, 1.0452]. They have no consistent direction. The intended
discriminator was a consistent claim difference accompanying the full-call
loss; its absence here cannot explain the earlier EPYC result. Counts also do
not identify allocation placement, failed steals or background-worker costs.

### Rejected known-owner steal-counter adjustment

At `bca257bb`, `wf__par_find` passed its existing current lane to the private
`wf__par_steal` helper instead of rereading `wf__par_self` through TLS for the
successful-claim counter. The same relaxed counter load/store remained
immediately after a successful claim. Queue operations, memory ordering, slot
layout, worker policy and ABI did not change. The experiment is rejected below;
the maintained core is restored exactly to `4fabd264`.

Local Clang 22.1.8 builds remove the successful-claim TLS access on Mac ARM
and Intel at O2/O3, with the normal 32-byte loop alignment on Intel. Static
instruction counts include padding and are not dynamic costs or measured
speedups. With `WF_SCHED_STATS=0`, complete before/after objects are byte-
identical on both targets and optimization levels. Native `sched-smoke` and
`sched-deque-test` pass, including startup failure, partial startup, nested
execution and 200,000 deque tasks with live counters both enabled and disabled.
The rebuilt ordinary compiler also passes the maintained 48-input scalar
Mandelbrot oracle panel at widths 1/2/4 and all four existing split settings.
The private-helper change also passes scoped independent ownership review.

The formal and ordinary-command comparisons used actual pre-change production
`4fabd264` as `previous`, replacing the completed compact-slot ablation; repaired
historical/research and replica controls remained. The Linux observation included
that previous image. Retention required native correctness and no reproducible
core/full-call/CPU regression against this baseline, in addition to the existing
historical/research qualification. Shorter local assembly was not a speedup.

The [five-target bca257bb run](https://github.com/mbbill/Whitefoot/actions/runs/34446440267)
completed with all formal and ordinary-command performance screens failing.
The exact local canonical `make check` passed, as did the Linux/Windows I/O
host checks and all four automatically triggered I/O benchmark jobs. Source,
manifest, oracle and reducer checks for the formal artifacts passed; performance
did not. The long and first64 core-only readers report:

| Target | Artifact | Long investigate | First64 investigate |
| --- | --- | --- | --- |
| Linux x86-64 | 10139970794 | 1 / 90 | 20 / 90 |
| Linux AArch64 | 10139947060 | 0 / 72 | 7 / 72 |
| macOS AArch64 | 10139938020 | 16 / 48 | 19 / 48 |
| macOS x86-64 | 10140237270 | 24 / 90 | 20 / 90 |
| Windows x86-64 | 10139951855 | 3 / 48 | 4 / 48 |

Full-call evidence independently rejects the adjustment. Linux AArch64 at
N65536/tile1024 gives current/previous ratios of 1.0533 [1.0494, 1.0555],
1.0732 [1.0709, 1.0744] and 1.0814 [1.0553, 1.1035] at widths 1/2/4.
Replica/previous medians are 1.0539, 1.0729 and 1.0861, while current/replica
medians are within 0.05% of one. Corresponding process CPU medians are
1.0520, 1.0486 and 1.0437. These losses cannot be dismissed because the long
core reader has no flagged cells.

Windows at W4/N4096/tile1024 gives full-call/previous 1.0889
[1.0610, 1.1086], replica/previous 1.0967 [1.0624, 1.1278], and A/A 0.9929
[0.9752, 1.0060]. Its outside-core interval is 1.1479 [1.1406, 1.1567];
core/previous is 0.9970 [0.9137, 1.0438]. CPU/previous is 1.1471
[0.9722, 1.2121], so a consistent CPU regression is not separately established.

The Linux x86-64 host is again EPYC 7763, two cores/four SMT CPUs. At
W4/N4096/tile64, full-call/research is 1.0885 [1.0541, 1.1177] and
CPU/research 1.0959 [1.0604, 1.1234]. Full-call/previous is 1.0092
[0.9771, 1.0265]; the adjustment did not resolve the existing loss.
The separate existing-counter observation also retains full-call/research
1.0718 [1.0474, 1.1293] while its successful-steal count ratio is 0.9698
[0.9363, 1.0467], without a consistent increase. Its same-tree sequential
control is 0.9964 [0.9792, 1.0100] for full calls. This does not support
more successful steals as the explanation, and still does not identify the
responsible consumer or worker operation.

Both Macs remain noisy. On ARM at W2/N4096/tile1024, full-call/previous is
1.1464 [0.9588, 1.4456] and replica/previous 1.3475 [1.0637, 1.8114].
On Intel at W2/N65536/tile64, the corresponding ratios are 1.0723
[0.8456, 1.6402] and 1.1986 [1.0523, 1.4630]. Broad A/A ranges prevent a
resolved magnitude or cause; the consistently slower replica pairs must still
be retained. Neither platform supports an improvement claim.

The Linux ARM single-participant loss is especially useful: that execution
does not update the successful-steal counter. Its common result-accessor
instructions are unchanged but move from address 0x4aa4 to 0x4a78 because the
runtime precedes the common WF object at link time. This identifies a code-
placement confound, not proof of its timing effect or justification for another
alignment search. The next bounded observation uses external Linux CPU samples
of the unchanged formal executables to locate complete-call CPU in result
access, allocation/destruction, computation or worker search. It preserves the
original timing matrix and does not add another runtime implementation.

### Restored core and external CPU observations at 849183a0

The [849183a0 cohort](https://github.com/mbbill/Whitefoot/actions/runs/34448631457)
uses the restored 4fab core. Exact local canonical `make check`, all twelve
gate jobs, both I/O host jobs and all four I/O benchmark jobs pass. All five
formal and ordinary-command performance screens still fail; this is not goal
completion. Formal long/first64 investigate counts are Linux x86-64 3/90 and
17/90, Linux ARM 0/72 and 1/72, Mac ARM 18/48 and 12/48, Mac Intel 16/90 and
20/90, and Windows 5/48 and 4/48.

Linux ARM N65536/tile1024 full-call/previous is now 0.9996
[0.9981, 1.0023], 1.0034 [0.9947, 1.0161] and 0.9977 [0.9936, 1.0152]
at widths 1/2/4. Windows W4/N4096/tile1024 full-call/old is 0.9107
[0.8198, 0.9411], with replica/old 0.8611 [0.7711, 0.9103] and wide
A/A 1.0338 [0.9617, 1.1227]. Previous/candidate executable text matches on
these platforms and both Macs, although debug information differs. Mac ARM
still has W2/N65536/tile64 replica/research full-call 1.0751
[1.0618, 1.2209]. Mac Intel's RSS and timing replicas remain unstable.

Both Linux external CPU observers run successfully on the original hashed
images. Each verifies its 249-entry profile manifest and all eighty plain/
sampled processes. x86-64 retains 327,760 calls and 28,991 samples; ARM retains
41,040 calls and 20,863 samples. All forty reports per target record zero lost
samples. Neither paired plain observation reproduces the earlier large loss:
W4 full-call/research is 1.0087 [1.0006, 1.0221] on x86-64 and 1.0069
[0.9972, 1.0172] on ARM. The x86-64 host is Xeon 8370C, not the earlier EPYC.

For the x86-64 N4096/tile64/W4 sample sets, worker-loop samples are
55.5%/56.4%/55.9% for historical/research/current, FIR tile samples
18.0%/17.6%/17.3%, and result-getter samples 14.5%/14.4%/14.8%.
Current worker hotspots map to the victim-search loop's index calculation,
top/bottom reads and scan branches. This is a common cost, not a measured
cause of the missing EPYC-specific gap. ARM's different, larger input spends
about 44–55% of samples in the tile and 28–37% in the getter; W4 worker-loop
shares are about 13.2–13.4%. Sampling percentages are not off-CPU delays or
hardware-stall measurements, and the two workloads cannot rank architectures.

The ordinary x86-64 CLI artifact in this cohort independently confirms that
current/previous compiler binaries and par/previous/replica programs are
byte-identical. Seven quiet default/static wall gaps remain. At small shape4,
N4096/W4, the default starts no helpers and takes 3.3023 [3.2705, 3.3921]
times static. The existing work60000 setting starts three helpers and takes
1.3965 [1.2051, 1.4565], with CPU/static 1.0078 [0.9860, 1.0301].
Large shape1/N65536/W2 already starts one helper but still takes 1.0725
[1.0598, 1.1377]. These separate insufficient publication from remaining
parallel execution costs; the current experiment does not change that policy.

### POSIX x86-64 spin-hint experiment at f950af4d (rejected)

The CPU observations gave a concrete reason to test the platform primitive,
without changing victim selection, the 4096-scan/16-yield limits or native
waiting. Windows already emits its historical `YieldProcessor` hint; POSIX
at `849183a0` left `wf_prim_spin_hint` empty. `f950af4d` added `_mm_pause()` only for POSIX
x86-64. It emits PAUSE, not a system call or SIMD computation; Clang's intrinsic
header and the [Intel optimization manual](https://cdrdv2-public.intel.com/821612/248966-Optimization-Reference-Manual-V1-050.pdf)
identify its spin-wait purpose. Processor-specific delay and execution costs
still require measurement; the unchanged scan count does not imply an unchanged
wall-clock spin duration. Busy-loop samples alone do not establish benefit.

The recorded hypothesis was reduced interference from empty searches on shared
physical cores without losing small-task responsiveness. Retention required
improved complete-call wall/CPU behavior against the unhinted maintained baseline
while preserving correctness, first64 responsiveness and the wider comparisons.
Retention also required no reproducible core, full-call or CPU regression
against that baseline across the retained matrix.
No all-platform improvement is assumed. The formal previous control freezes
the old primitive header as well as the core; otherwise a shared new header
would silently give the supposed baseline the new hint too. ARM and Windows
retain their existing primitive bodies. No counter, slot layout, public ABI,
I/O behavior or default publication policy changes in this experiment.

Local Clang 22.1.8 O2/O3 builds, with counters both on and off, emit exactly two
PAUSE sites in x86-64 runtime objects and none in the previous objects. ARM
before/after objects are byte-identical for those same settings. Native M1
smoke/deque checks and the ordinary compiler's 48-input Mandelbrot oracle panel
pass. These are code-generation and correctness checks, not native x86-64
performance qualification.

The [f950af4d native cohort](https://github.com/mbbill/Whitefoot/actions/runs/34452581006)
rejects this change. Exact local canonical `make check`, all twelve gate jobs,
both I/O host jobs and all four I/O benchmark jobs pass, but all five formal
performance screens fail. The ordinary Mac Intel reader passes; its raw
measurements do not establish a reliable performance win.

Linux x86-64 runs on EPYC 7763 with two physical cores/four hardware threads.
At W4/N4096/tile64, full-call/previous is 0.9186 [0.9089, 0.9720] and
CPU/previous is 0.9420 [0.9108, 0.9717]. At W4/N65536/tile1024, however,
full-call/previous is 1.0902 [1.0476, 1.0965] and CPU/previous is
1.2304 [1.2071, 1.2575]; replica CPU/previous is 1.2296
[1.1918, 1.2744], while CPU A/A is 1.0155 [0.9472, 1.0331].
The tile16/tile64 large-input CPU ratios are also 1.1068 and 1.1713, with
all five pairs above 1.05. The original long screen is 0/90 investigate,
but first64 is 17/90 and the independently checked CPU losses reject the hint.
The previous headers match exact `4fabd264`; disassembly confirms zero/two
processor PAUSE instructions in previous/current. The common POSIX `pause`
function used by exhaustion handling is a different operation.

Mac Intel confirms the wall/CPU tradeoff independently. W2/N4096/tile16
core/previous is 0.5098 [0.4719, 0.5264] and full-call/previous is
0.8899 [0.8238, 0.9480], but CPU/previous is 1.1683
[1.0827, 1.2501], with replica 1.1974 [1.1161, 1.2262]. At
W4/N65536/tile1024, CPU/previous is 1.2884 [1.2602, 1.3814],
replica 1.3349 [1.1492, 1.3710], and CPU A/A 1.0026 [0.9440, 1.1082].
Its long/first64 investigate counts are both 34/90. Linux ARM is 2/72
and 3/72, with no core/full-call/CPU cell above 1.05 in all five pairs;
Mac ARM is 11/48 and 22/48. Both ARM executable text images match their
previous controls. Mac ARM still has W2/N65536/tile16 full-call/research
1.2072 [1.0578, 1.2983], replica 1.1893 [1.1057, 1.3329]. That unresolved
gap cannot be attributed to the x86-64 hint.

Windows is 4/48 long and 9/48 first64 investigate. Its W4/N4096/tile1024
full-call/previous ratio is 0.9951 [0.9341, 1.0570], CPU/previous 1.0250
[0.9524, 1.0526], and full-call/old 0.8668 [0.8225, 0.9032]. Previous/current
executable text is identical; those timings do not measure a new Windows hint.

The ordinary Mac Intel panel marks 624 of 708 wall/CPU rows `noisy-open`.
Its four-leaf wall/previous ratio 0.8245 [0.6484, 0.8352] accompanies
wide wall A/A 0.8631 [0.7038, 1.0486], so its green reader is not a strong
gain. The Linux ordinary four-leaf control instead has CPU/previous 1.0595
[1.0504, 1.0777], with CPU A/A 1.0202 [1.0114, 1.0236]. Noisy wall
measurements must not hide that repeated CPU regression.

Restore the prior POSIX primitive under the recorded no-regression criterion.
Keep the complete previous-header freeze, raw artifacts and unchanged screening
thresholds. No queue, waiting-policy limit, ABI, I/O path or publication setting
changes as part of this rejection. All-platform performance qualification and
the ordinary compiler's wider workload gaps remain open.

### Restored core at 2ae2b37e: reproduced Linux full-call gap

The [2ae2b37e cohort](https://github.com/mbbill/Whitefoot/actions/runs/34456384659)
restores the exact `4fabd264` core and inline primitives. The exact local
canonical `make check`, all twelve gate jobs, both I/O host jobs and all four
I/O benchmark jobs pass. All five formal performance jobs still fail.
Their long/first64 investigate counts are Linux x86-64 1/90 and 15/90,
Linux ARM 0/72 and 10/72, Mac ARM 16/48 and 20/48, Mac Intel 20/90 and
36/90, and Windows 1/48 and 12/48. The ordinary Mac ARM reader passes,
but all 476 wall/CPU rows are `noisy-open`; that green job is not performance
qualification. No default publication policy changes in this cohort.

On Linux x86-64, again reporting EPYC 7763 with two cores/four hardware
threads, W4/N65536/tile1024 full-call/previous is 0.9961
[0.9840, 1.0037] and CPU/previous is 0.9972 [0.9933, 1.0023]. The
rejected hint's repeated large-input CPU regression is absent in this
same-host comparison with the unhinted control. Cross-cohort elapsed times
are not used to estimate the rollback's effect.

The unresolved W4/N4096/tile64 full-call/research ratio is 1.0856
[1.0441, 1.1048], CPU/research 1.0881 [1.0420, 1.1053], and core/research
0.9910 [0.9838, 0.9984]. This time the external observer reproduces the
gap on its original hashed images. Five paired process ratios are summarized
as median [minimum, maximum], without treating individual calls as independent
process replications:

| Observation | Full call (current/research) | Process CPU (current/research) | Core (current/research) |
|---|---:|---:|---:|
| Plain processes preceding sampling | 1.0790 [1.0600, 1.1149] | 1.0865 [1.0637, 1.0905] | 0.9974 [0.9597, 1.0313] |
| CPU-sampled processes | 1.0910 [1.0680, 1.0997] | 1.1031 [1.0643, 1.1428] | 0.9943 [0.9794, 1.0147] |
| Existing exit-time steal-counter observation | 1.0843 [1.0625, 1.1029] | 1.0948 [1.0606, 1.1053] | 1.0036 [0.9926, 1.0057] |
| Existing sequential-entry control | 1.0126 [0.9675, 1.0249] | 1.0126 [0.9681, 1.0242] | 1.0023 [0.9621, 1.0132] |

Independent replay verifies all eighty profile processes, 327,760 calls,
249 profile hashes and 26,483 samples; all forty perf reports record zero
lost samples. Current/replica sampled full-call A/A is 1.0000
[0.9864, 1.0198]. Summed W4 owner samples are 1458/1334 for current/research,
and helper samples are 4161/3781. Every pair has more samples in both roles;
worker-loop samples account for 3341/2979, and result-getter samples for
911/810. The getter identifies the actual calling thread; the three helper
TIDs execute the worker loop. These observations cannot distinguish more
expensive individual searches from longer consumption keeping helpers busy.
Sampled timings are perturbed and are not corrected by subtracting an observer
cost. Absence of a fifth sampled TID does not prove zero bootstrap CPU.

The separate serial/counter replay verifies 72 hashed files, 45 processes
and 184,365 oracle-checked calls. Successful steals/current over research
are 0.9729 [0.9223, 0.9768]; more successful steals cannot explain the
current image's higher CPU usage. These counts include startup and all warm
calls and do not count unsuccessful searches. Voluntary switch counts are
slightly higher in four pairs and equal in one, within the replica variation;
involuntary differences reverse direction. Neither count measures wait duration
or explains the full-call gap.
The sequential selector retains the owned result-tree form but uses a
different existing WF entry; it does not isolate scheduling from code placement
or allocation history. Its lack of a repeated full-call regression is a useful
constraint, not a causal proof about the parallel gap.

Mac ARM's W2/N65536/tile16 full-minus-core/research signal weakens in this
third cohort: current is 1.0579 [1.0193, 1.2090], replica is 1.0842
[0.9982, 1.2115], and A/A is 0.9980 [0.8420, 1.0472]. Prior stronger
cohorts remain evidence, but these results cannot yield a fixed runtime cost.
All-platform performance and wider ordinary-command workloads remain open.

The next bounded observation adds CPU identifiers to the existing external
Linux sampler and records guest-reported sibling/core/package topology. All
forty `2ae2b37e` records have sample type `0x107`, which omits the
[Linux CPU sample field](https://github.com/torvalds/linux/blob/v6.8/include/uapi/linux/perf_event.h#L130-L139).
No CPU placement or migration claim can be recovered from those records.
The [perf record option](https://github.com/torvalds/linux/blob/v6.8/tools/perf/Documentation/perf-record.txt#L324-L325)
adds that field without relinking a runtime or adding a hot-path counter.
The discriminating question is whether the reproduced full-call/CPU gap
coincides with consistent owner/helper CPU placement differences. If the gap
is absent during observation, or placement has no consistent differential,
the observation cannot select an affinity or waiting-policy change. Sample
transitions only bound migrations from below; guest topology does not reveal
physical-host contention. Runtime source, waiting limits, workload, plain
controls and performance acceptance remain unchanged.

### Startup readiness wait at 7f56548d (rejected)

The [dbbe0d1b observation](https://github.com/mbbill/Whitefoot/actions/runs/34461041448)
records CPU fields successfully but does not select affinity: all fifteen
four-worker current/research/replica processes cover the four guest CPUs with
their threads' dominant placements. Adjacent samples show 12/7/12 observed
CPU changes respectively, without a consistent paired relationship to time.
On this EPYC 9V74 host, plain full-call/research is 1.0363
[1.0207, 1.0544], sampled 1.0259 [1.0176, 1.0446]. The runtime is unchanged;
the smaller gap than on EPYC 7763 is not an optimization result. Neither
sampling success nor these selected cells qualifies the five-target matrix.

The next maintained candidate addresses a separate, bounded startup cost.
The formal core repeatedly calls the OS yield primitive until created workers
finish attaching their exhaustion handlers; recovered research waits on a
condition variable. Reuse the already initialized owner lane's wait during
startup, with readiness publication and the creator's predicate check under
that same lock. No task has been granted before the once-initializer returns,
so this requires no new wait object or task protocol. Preserve partial startup,
the all-created-workers-ready barrier, floor attachment, ordinary ABI, default
split allowance, and both existing compute/join idle limits.

This removes explicit startup busy yielding by construction; it does not
establish that startup explains the warm FIR gap or the ordinary-command
losses. The added delayed-startup smoke cases must reject busy yielding
and early return, including partial worker creation. The existing formal
panel retains the previous core, first-call rows, warm matrix and byte-identical
replica; the ordinary command panel retains its previous compiler and complete
wall/CPU envelope. Evaluate cold-call and whole-command cost on all five native
targets, retaining this candidate only with evidence of benefit and without
stable warm wall/CPU regressions. Do not substitute startup success for the
remaining performance qualification.

Local pre-CI evidence uses ordinary scalar compiler output on an unpinned
Apple M1 Pro (eight logical CPUs, macOS Darwin 25.6.0). All 120 timed processes
pass their independent expected digests; separate diagnostics confirm zero
helpers at W1 and three at W4. Five alternating process pairs compare current,
previous `dbbe0d1b`, a byte-identical current replica and native static, using
the existing work60000 setting, 4,096 points and limit256. Selected W4 ratios
against previous are wall/CPU 1.0299/1.0050 for the one-batch cheap-escape case,
0.9849/0.9922 for 32 uniform-heavy batches, and 1.0038/1.0013 for 256 batches.
The 32-batch wall range is [0.9840, 0.9964], but its A/A range is
[0.9809, 1.0057] and W1 also falls to 0.9920 without entering startup.
This is not a resolved startup benefit. Source-level startup tests pass with
all helpers and partial creation; the same tests abort against the old yield
loop and against a one-wake-only barrier for their respective intended reasons.
Native CI and full qualification remain required.

The [7f56548d native cohort](https://github.com/mbbill/Whitefoot/actions/runs/34464897987)
rejects this candidate. Its exact local canonical `make check` and both I/O
host jobs pass, but Linux ARM has a quiet warm full-call regression at
W2/N65536/tile1024: current/previous 1.0534 [1.0529, 1.0580], versus
current/replica 0.9998 [0.9938, 1.0024]. Batch CPU/previous is 1.0367
[1.0341, 1.0399]. W1 does not start helpers yet also loses 4.15% full-call
time and 4.05% CPU, so the executed readiness wait is not an established
cause of these image-level effects. The candidate still fails the stated
no-regression condition; a possible layout explanation does not waive it.

Windows W4/N4096/tile1024 independently loses full-call time against previous:
1.0998 [1.0524, 1.1635], with full-call A/A 1.0015 [0.9725, 1.0158].
Quantized process CPU/previous is 1.1765 [1.0571, 1.2500]; CPU A/A spans
[0.9744, 1.1143]. These are whole-batch CPU readings, not first-call CPU or
an isolated startup cost. Windows core/previous is 0.9615 with a wide
[0.8917, 1.1191] range, so looking only at core medians would miss the loss.

Linux x86-64 W4/N4096/tile64 improves: full-call/previous is 0.9405
[0.9120, 0.9737] and batch CPU/previous is 0.9342 [0.9094, 0.9831].
The byte-identical replica also improves against previous, with medians
0.9252 and 0.9238. This is a supported same-host warm-call benefit for
that cell, not proof of a startup cause or a portable improvement. Neither
Linux x86-64 nor macOS ARM has a cell where both current and replica beat
previous first full-call time by 5% in every pair. All five formal screens
remain unqualified; only macOS ARM passes the ordinary-command screen.
The twelve gate jobs, two I/O host jobs and four I/O benchmark jobs pass.

Restore the exact `4fabd264` core and inline primitives. Retain the new
delayed-floor/full-readiness tests, including partial creation, against the
restored polling protocol. The test-specific ban on startup yielding is
retired with the rejected condition-wait policy; it is not a safety rule.
The retained test releases helpers gradually when the creator polls and
still rejects a one-check-only readiness barrier. No production waiting
threshold, default publication policy, I/O path or ABI change survives.

The [fa77a029 restored cohort](https://github.com/mbbill/Whitefoot/actions/runs/34468351790)
does not reproduce the two selected warm regressions against its same-host
previous control. Linux ARM W2/N65536/tile1024 full-call/previous is 0.9996
[0.9984, 1.0056], CPU 0.9993 [0.9987, 1.0053]. Windows W4/N4096/tile1024
full-call/previous is 1.0024 [0.9965, 1.0204]; its quantized CPU ratio is
1.0278 [0.9167, 1.0588]. Current/previous executable code sections match on
both platforms. Windows now uses Xeon 6973P-C; cross-cohort absolute times
cannot establish a rollback speedup.

Linux x86-64 W4/N4096/tile64 still has a full-call/research difference:
1.0399 [1.0355, 1.0864], with CPU 1.0458 [1.0229, 1.0963] and core
0.9675 [0.9485, 1.0180]. Full-call A/A is 1.0018 [0.9944, 1.0123].
The three targets retain their original performance-screen failures. All
twelve gate jobs and both I/O host jobs pass; the separate Windows I/O
benchmark stops on its unchanged two-cohort timing-stability check, while
the other three I/O benchmark jobs pass. This is not full qualification.

### Locate the remaining full-call cost

The restored core's repeated Linux x86-64 W4/N4096/tile64 full-call loss
survives external sampling, while its core interval is near research parity.
The core interval includes WF computation, result allocation and joins; it
is not an isolated scheduler measurement. More getter and helper-loop samples
do not establish which makes the other run longer. Before another production
change, add one timestamp to the existing FIR caller, after result/history
reading and before result destruction. A separately built diagnostic caller
records this readout interval beside the existing core/full-call values,
retaining all output checks. The original images and timing matrix remain.

Use the same W4/N4096/tile64 input, five alternating process rounds and
old/research/current/byte-identical-replica controls on Linux x86-64. Each
diagnostic process is preceded by its original image as a perturbation control.
If the full-call gap repeats but readout does not carry it, investigate the
remaining prefix allocation/copy and destruction interval. If observation
removes the gap or the replica is unstable, the observation cannot select a
runtime change. A readout difference alone does not distinguish getter
instructions, allocation history, cache effects or competing helper work.
Batch CPU still includes all threads and between-call correctness checks;
it cannot be assigned to the new interval. No waiting or publication parameter
changes accompany this observation.

Local macOS ARM validation runs the same diagnostic driver block over forty
processes and 163,880 complete oracle-checked calls, with all forty process
means independently replayed. Both replicas are byte-identical; disabling the
diagnostic retains the previous caller's exact executable text. The reader
rejects eight malformed-log variants, including missing/invalid CPU metadata,
a changed header, an empty call index and readout outside the full interval.
This validates observation plumbing, not the Linux performance hypothesis.

The [c85e69af artifact](https://github.com/mbbill/Whitefoot/actions/runs/34469937880/artifacts/10149269507)
does not support a runtime change. On EPYC 9V74, four logical CPUs/two cores,
the five paired current/research ratios are median [minimum, maximum]:

| Image group | Full call | Core | Whole-batch CPU |
| --- | --- | --- | --- |
| Original matrix | 1.0404 [1.0283, 1.0630] | 0.9893 [0.9823, 0.9988] | 1.0371 [1.0186, 1.0583] |
| Observation panel, original images | 1.0312 [1.0247, 1.0501] | 0.9761 [0.9617, 0.9875] | 1.0324 [1.0240, 1.0493] |
| Observation panel, instrumented images | 0.9643 [0.9508, 0.9675] | 0.9752 [0.9743, 0.9899] | 0.9718 [0.9486, 0.9738] |

Instrumented/original full-call ratios are 1.0531 [1.0437, 1.0616] for research
and 0.9844 [0.9617, 0.9917] for current. This reverses the ranking; the observer
does not add a common cost that can be subtracted. Its readout current/research
ratio, 0.9488 [0.9350, 0.9571], cannot explain the original images' difference.
Full-call A/A ranges are [0.9732, 1.0222] in the original observation images
and [0.9589, 1.0090] when instrumented; instrumented CPU A/A extends to 1.1030
and core A/A down to 0.8616. These uncertainties remain.

Independent replay verifies all forty observation processes, 163,880 checked
calls, actual participants, metadata, timing boundaries, original reducer means,
source/object provenance and both byte-identical replicas. The original matrix
also retains all 540 processes, 1,244,700 checked calls and 718 manifest hashes;
its long/first64 screens still report 4/90 and 11/90 investigate comparisons.
Retire this observer because its perturbation prevents the intended attribution,
not because the original performance failure disappeared. Restore the original
caller and driver; no production source, threshold or baseline check changes.

Exact `c85e69afc7f00a25560bc1de4225fbae876f7260` passes local canonical
`make check` with the existing pinned oneTBB/Parlay source paths supplied.
All twelve [gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34469937814)
and both [I/O host jobs](https://github.com/mbbill/Whitefoot/actions/runs/34469937866)
pass. All five formal and all five ordinary-command performance screens, plus
quadrature, still fail; the other eight compute jobs pass. This result does
not qualify the whole runtime or locate the next optimization.

Independent replay of the fa77a029
[ordinary Linux x86-64 artifact](https://github.com/mbbill/Whitefoot/actions/runs/34468351790/artifacts/10148613837)
also preserves a larger, separate default-policy gap. All 1,625 processes and
1,062 summary rows replay exactly; current/previous compilers and their
par/previous/replica executables are byte-identical. Shape4/N4096/W4 default/
static wall is 3.3681 [2.7814, 3.4349], CPU 0.8694 [0.7513, 0.8857]. The emitted
weight is 219: the default threshold needs 5,480 iterations per chunk, so this
4,096-point call receives no split budget and starts no helpers. At work60000,
three helpers start and wall/default is 0.4582 [0.4268, 0.4713], with CPU/default
1.1964 [1.1887, 1.2176]. Its remaining wall/static ratio is 1.4967
[1.2822, 1.5841], not an isolated runtime cost. Conversely, shape5's quick exits
at the same size/width give work60000/default wall 1.4367 [1.4077, 1.4942], with
noisy default A/A. A global threshold reduction cannot be called a general win
from this evidence. Preserve the existing knob and original failed screen;
this does not select a new cost model or make default-policy performance pass.

### Equivalent victim-address traversal (rejected)

The c85e69af external Linux x86-64 sample has 2,602 of 4,501 W4 samples in
the worker loop. Its emitted scan recomputes a wrapped integer index, scales
it by the lane stride and compares the resulting pointer with the owner on
each iteration. This is a common cost, not an explanation of the remaining
current/research difference. Sampling does not assign an instruction's latency.
Test one equivalent implementation in the maintained core: start at the same
random victim, advance its pointer, and wrap at the same captured lane count.
Keep the random sequence, exact victim order, deque atomics, counters and
spin/yield/park thresholds unchanged. No new runtime storage or interface is
needed, including during partial startup.

First check generated code and existing concurrent deque/startup tests. If
address work is not reduced, discard the candidate before native timing.
Otherwise compare it with the prior maintained core and frozen controls in the
existing five-target formal and ordinary matrices, including complete-call and
CPU costs. A sample share or a shorter scan does not establish a net program
benefit: faster empty scans can change when the unchanged park threshold is
reached. Retain only measured useful gains without a repeatable regression;
do not compensate a loss by adjusting waits, grain or the acceptance screen.

Local macOS ARM smoke tests cover normal/refused/partial startup and delayed
readiness; both 200,000-task deque probes and all seventeen loop backend tests
pass. Independent review confirms bounds through the 64-lane tail pointer,
unchanged victim order and the existing stale-count startup argument. Native
ARM and cross-compiled macOS x86-64 O2 disassembly move lane scaling outside
the inner scan. Reported object text changes by -36 and +90 bytes respectively.
These are code-shape and correctness observations, not measured speedups or
five-platform qualification.

The [79170ebc cohort](https://github.com/mbbill/Whitefoot/actions/runs/34473362742)
does not select this candidate. The
[Windows artifact](https://github.com/mbbill/Whitefoot/actions/runs/34473362742/artifacts/10150686065)
has a repeated W4/N4096/tile1024 regression against the prior maintained core:

| Metric | Current / previous | Replica / previous | Current / replica |
| --- | --- | --- | --- |
| Full call | 1.1638 [1.0351, 1.2255] | 1.1518 [1.1015, 1.1708] | 1.0104 [0.9397, 1.0467] |
| Whole-batch CPU | 1.1500 [1.0750, 1.2632] | 1.1500 [1.1000, 1.2000] | 1.0435 [0.9348, 1.0667] |
| Core | 1.0011 [0.7766, 1.1724] | 0.9954 [0.9185, 1.0477] | 1.0058 [0.8454, 1.1190] |

Each entry is the median [minimum, maximum] of five process pairs. Windows
CPU readings are quantized, but every replica pair increases CPU by 10–20%
and full-call time by 10–17%; core parity cannot excuse that cost. The
[Linux ARM artifact](https://github.com/mbbill/Whitefoot/actions/runs/34473362742/artifacts/10150714447)
has a smaller local gain at W4/N4096/tile16: CPU/previous is 0.9600
[0.9392, 0.9715], replica 0.9580 [0.9481, 0.9688], with CPU A/A
[0.9879, 1.0051]. Full call is 1.0063 [0.9760, 1.0177], so this does not
establish a corresponding wall-time gain. Neither platform has a five-pair
RSS increase above 5%. These observations do not locate the regression's cause.

Independent audits verify all 407 Windows and 505 ARM manifest entries,
300/450 processes, 691,500/1,037,250 checked calls, exact current/prior/frozen
sources and repairs, means and the unchanged reducers. Their long/first64
investigate counts remain 1/48 and 10/48 on Windows, 4/72 and 3/72 on ARM.
Four ordinary-command artifacts (Linux x86-64/ARM, Windows and
macOS Intel) also replay their complete 1,625-process matrices and 1,062-row
summaries, with byte-identical current replicas. None establishes a useful
same-policy delivered gain; their original screens remain failed. The Windows
formal result already fails the candidate's selection condition; other target
results are not needed to reject it and are not counted as passing here.

Restore the exact prior `4fabd264` core, including its original index traversal.
Retain no new runtime parameter, wait change or benchmark check from this
candidate. Its exact revision `79170ebc49ff454e0d71c63c18bdf5f3d5384ed8` passes
local canonical `make check`, both I/O host jobs and all four I/O benchmark
jobs; correctness does not override the performance rejection. The overall
compute-first runtime's cross-platform performance qualification remains open.

### Repeated measurements of the restored core

After the traversal rollback at `31f47b75`, consolidate existing measurements
before selecting another production change. This is a descriptive comparison
of compatible cohorts, not a new performance gate. Keep each cell's five
paired process observations, original reducers and failed/noisy results. Do
not pool absolute times across hosts or treat calls within one process as
independent repetitions.

The reviewed set contains six Linux ARM, seven Linux x86-64, seven Windows
and seven per Mac architecture artifacts: 14,460 original-matrix processes
and 33,330,300 checked calls. Every included current runtime matches the
restored production sources. On Linux, FIR/caller/control sources and flags
also match exactly. The ARM set is 7db/bc/849/2ae/dbbe/fa77; the x86-64 set
adds 4fab. Exclude the changed a130/c85 callers and rejected f950
primitive from that strict set. Windows and Mac a130/9253 use an older caller;
keep them separate from 7db/bc/849/2ae/fa77, whose pre-timing selector differs.
The actual `previous` revision is a822 in Linux ARM 7db/bc, Linux x86-64
4fab/7db/bc, and Windows/Mac a130/9253/7db/bc; it is 4fab in the other
included cohorts. A common label does not make these previous controls equal.

Source and frozen-control repairs, manifests, complete raw matrices, metadata,
oracles, means and unchanged reducers were replayed. The Linux review also
checks every allocated ELF section: within a target, each matching current,
old or research image has identical section addresses, sizes, alignments and
file-backed bytes across the strict cohorts. Mac executable text and Windows
executable sections likewise match within each architecture/caller group.
Current and research remain different images. These checks exclude a changed
Linux allocated section or Mac/Windows executable code within the compared
groups; they do not exclude ASLR,
dynamic libraries, allocation history or different host interference.

Linux ARM's 108 cohort-by-cell current/research full-call medians range from
0.9714 to 1.0308, and CPU medians from 0.9733 to 1.0165. Its earlier large
reported gaps do not describe a repeated loss of this restored code. The
Linux x86-64 W4/N4096/tile64 difference is more specific. Each entry below
is a median of five within-host process pairs against repaired research:

| Host / cohort | Current full / CPU | Replica full / CPU | Full-call current/replica range |
| --- | --- | --- | --- |
| EPYC 7763 / 7db | 1.1139 / 1.1080 | 1.0913 / 1.0909 | [0.9957, 1.0287] |
| EPYC 7763 / bc | 1.0596 / 1.0696 | 1.0776 / 1.0740 | [0.9612, 0.9992] |
| EPYC 7763 / 2ae | 1.0856 / 1.0881 | 1.1058 / 1.1046 | [0.9587, 1.0015] |
| EPYC 9V74 / dbbe | 1.0428 / 1.0392 | 1.0402 / 1.0355 | [0.9868, 1.0335] |
| EPYC 9V74 / fa77 | 1.0399 / 1.0458 | 1.0388 / 1.0467 | [0.9944, 1.0123] |
| Xeon 8370C / 849 | 1.0078 / 1.0118 | 1.0102 / 1.0137 | [0.9774, 1.0186] |
| Xeon 6973P-C / 4fab | 1.0019 / 0.9918 | 0.9864 / 0.9795 | [0.9894, 1.0232] |

On 7763, all fifteen current/research pairs and all fifteen replica/research
pairs lose both full-call time and CPU. Current core medians are 1.0169,
0.9902 and 0.9910; there is no corresponding repeated core loss of that size.
The [existing external sampling](#restored-core-at-2ae2b37e-reproduced-linux-full-call-gap)
also retains the gap, but does not distinguish slower result consumption from
more costly helper searches. The full interval cannot be called scheduler
time, and changing CPUs is not an optimization result.

The other repeated comparisons constrain further work:

- Mac ARM W2/N65536/tile16 has a weaker CPU/old signal: current medians
  1.0444–1.1021 and replica medians 1.0458–1.0852 across the five newer-caller
  cohorts. All medians exceed one, but paired ranges and A/A remain unstable;
  this does not identify a fixed cost.
- Mac Intel's large differences often reverse or fail replication. At
  2ae W4/N65536/tile1024, full/research is 1.3424 for current but 0.9743 for
  its identical replica. This cannot select a production optimization.
- Windows's older-caller W4/N4096/tile1024 full/old losses, 1.1857 and 1.0767,
  do not persist in the five compatible newer-caller cohorts, whose medians
  are 0.9027–0.9496. All sixty newer-caller cohort-by-cell full/old medians
  range from 0.8992 to 1.0303. This neither proves the selector caused the
  change nor turns the retained failed screens into passes.

The [restored 31f47b75 gate](https://github.com/mbbill/Whitefoot/actions/runs/34474873189)
passes all twelve jobs, and both
[I/O host jobs](https://github.com/mbbill/Whitefoot/actions/runs/34474873265)
pass. All five formal performance screens remain failed. Three I/O benchmark
jobs pass; the Windows protocol reports compute timings unstable after two
complete cohorts. No new runtime change, default policy, ABI or performance
threshold is selected by this consolidation. Five-target qualification and
the ordinary-workload losses remain open.

### Per-lane counter grouping at 9a353fc5 (rejected)

The repeated EPYC 7763 W4/N4096/tile64 loss motivated a bounded layout
candidate in the maintained core. The existing 2ae images' DWARF records
show current lane size `0x5000` (20,480 bytes), versus research `0x4f80`
(20,352). The separately aligned per-lane counter adds 128 bytes: current
`steals` is at `0x380` and slots at `0x400`, while research slots start at
`0x380`. Corresponding top indices across current lanes, and likewise bottom
indices, repeat their respective offsets modulo 4 KiB. This is an address
fact, not measured cache-conflict attribution. The existing Linux host report
now captures guest cache geometry; neither geometry nor the CPU model proves
that conflicts explain the loss.

The candidate moved `steals` next to `seed` in existing metadata padding,
preserving its single writer and atomic observers, the separate wait alignment,
frame capacity, slot count, atomic ordering, victim sequence and waiting limits.
This removed
the counter's separate aligned region without a new pointer or counter scheme.
It also increased sharing: the final ring cell, allocation/random metadata
and statistics occupy the same cache line. Thieves can read the ring cell,
and a live observer can read the counter. This is not an owner-private line.

Qualification covered native layouts and generated address
arithmetic, including lane-pointer differences and startup/reporting scans,
at O2/O3 with statistics on/off; startup, reuse and live-observer deque tests;
and the unchanged five-target formal and ordinary matrices against exact 4fab
and the repaired frozen controls. A smaller structure does not guarantee fewer
instructions or better timing. The recorded retention criterion required
a repeatable full-call/CPU benefit at the existing loss, without a repeated
regression elsewhere; core, first64, RSS and default-policy losses remain
visible. A host that does not reproduce the original difference cannot
establish that this candidate resolves it. No public ABI, I/O behavior,
publication setting or benchmark threshold changed.

Local Clang 22.1.8 builds verify all sixteen native ARM/cross-compiled macOS
x86-64 objects at O2/O3 with statistics on/off. Lane size changes from 20,480
to 20,352 bytes, with the counter at 664; wait offset 768, 304-byte slots,
sixteen-byte frame alignment and 256-byte payload remain unchanged. Address
arithmetic does change: x86-64 uses an immediate multiply instead of a
scale/shift sequence, while ARM materializes a longer reciprocal before the
worker's main loop. Fewer static x86 instructions do not establish lower
latency. Existing startup/reuse smoke checks and both 200,000-task deque
probes pass, including both statistics settings under ThreadSanitizer with
live counter observation. These are local correctness/layout checks, not
native five-target performance qualification.

The [9a353fc5 cohort](https://github.com/mbbill/Whitefoot/actions/runs/34480592253)
does not meet that retention criterion. Review verifies all five formal
artifacts' source identities, previous core/headers, historical repairs,
manifests, raw matrices, actual widths, oracles, metadata, means and unchanged
reducers. Candidate and replica are byte-identical within each artifact.
The long/first64 core screens retain these investigate counts:

| Target | Long | First64 |
| --- | ---: | ---: |
| Linux x86-64 | 1/90 | 21/90 |
| Linux ARM | 0/72 | 3/72 |
| Mac Intel | 30/90 | 33/90 |
| Mac ARM | 11/48 | 17/48 |
| Windows | 2/48 | 9/48 |

Linux x86-64 again reports EPYC 7763, two cores/four SMT threads. At
W4/N4096/tile64, previous/research reproduces the original loss: full-call
1.0880 [1.0459, 1.1017] and CPU 1.0963 [1.0520, 1.1056]. Five paired
process ratios for the candidate are:

| Comparison | Full call | Process CPU |
| --- | ---: | ---: |
| Candidate / previous | 0.9916 [0.9406, 1.0218] | 0.9958 [0.9462, 1.0195] |
| Replica / previous | 0.9717 [0.9417, 0.9968] | 0.9684 [0.9049, 0.9928] |
| Candidate / research | 1.0687 [1.0363, 1.0825] | 1.0725 [1.0422, 1.0917] |
| Replica / research | 1.0535 [0.9848, 1.0845] | 1.0449 [0.9519, 1.0884] |

Candidate/previous core is 1.0010 and RSS 1.0000. Full-call A/A ranges
from 0.9890 to 1.0851, CPU from 0.9751 to 1.1268. The candidate's benefit
is not jointly confirmed by both identical images. Actual DWARF verifies the
smaller lane; the guest reports 32 KiB, eight-way, 64-set L1d caches. This
rejects the measured candidate under the stated criterion, without proving
that data layout never matters or attributing a cache-miss cost.

The other platforms do not settle selection. Linux ARM's eighteen
candidate/previous full-call medians range from 0.9830 to 1.0164. Mac Intel
has no jointly confirmed full-call/CPU benefit across its eighteen cells;
W4/N65536/tile64 still loses against old, with candidate full-call 1.1976
[1.0400, 1.2880] and CPU 1.2793 [1.0349, 1.3665]. Mac ARM retains a
large/coarse regression risk: W2/N65536/tile1024 full-call/previous medians
are 1.1441 and 1.1599 for candidate/replica, with full-call A/A
[0.8269, 1.0283] and both paired ranges crossing one. Windows has a
W2/N4096/tile1024 full-call benefit signal: candidate/previous 0.9111
[0.8265, 0.9667], replica 0.8956 [0.8682, 0.9973], with A/A
[0.9300, 1.0794] limiting its estimated magnitude. Its W4 full-call/previous
medians at that input are 1.0193 and 1.0182; better full-call/old medians,
0.8889 and 0.9287, do not establish a benefit of this revision. This signal
does not resolve the original Linux loss. No noisy or failed result is
promoted to a pass.

The independently replayed Linux x86-64 ordinary-command artifact also
retains 39 `gap` and 302 `noisy-open` rows. For shape4/N4096/W4,
default/previous full-command wall and CPU are 1.0004 and 1.0004;
replica/previous is 0.9993 and 0.9999. Default/static wall remains 3.3426
with zero helpers. Work60000 starts three helpers and reaches 1.3328
[1.2973, 1.3467] wall/static, CPU/static 1.0185 [1.0136, 1.0229]. These
are policy comparisons; the artifact contains no previous-work60000 or
replica-work60000 process samples to measure that policy's revision effect.

Restore the maintained core byte-for-byte to 4fab, including its separately
aligned counter. Keep the cache report, frozen controls, artifacts and
unchanged thresholds. Exact 9a passed local canonical `make check`, all twelve
[gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34480592271), both
[I/O host jobs](https://github.com/mbbill/Whitefoot/actions/runs/34480592472)
and all four [I/O benchmark jobs](https://github.com/mbbill/Whitefoot/actions/runs/34480592366).
These correctness results do not select the performance candidate.
All-platform qualification and the ordinary-workload losses remain open.

### Quadrature evidence constrains runtime attribution

The same 9a compute run's quadrature failure is its formal/recovered parity
screen, with 21/80 investigate rows. Replay verifies 103 manifest entries,
31 captured sources, 2,400 processes and 614,400 timed calls, including every
oracle, actual-width check and original reducer. The panel has no previous-4fab
or byte-identical replica control, so it cannot select the counter revision.
Timing includes result checks and excludes eight warmup calls and process
startup. It links the maintained runtime to the ordinary compiler's generated
compute object; the separate ordinary CLI invocation checks correctness.
Ratios below are medians of five process pairs [minimum, maximum]; CPU means
process CPU unless explicitly labeled caller CPU. WF/C++ and Rust/Rayon all
target x86-64-v3 with loop/SLP vectorization and LTO disabled. WF/C++ uses
strict FP without contraction; the Rust kernel uses no explicit SIMD or
`mul_add`. Toolchain versions differ: Clang LLVM 18.1.3 and Rust LLVM 22.1.8.
The guest reports EPYC 7763 with two cores/four SMT threads.

Within the same formal image at W4, default recursive WF/Cpp-seq on
center-peak is 1.118 [1.053, 1.173] for batch wall and 3.158 for CPU;
WF/Rayon-d4 is 1.364 [1.251, 1.480] wall, 1.331 CPU. On depth-cap,
WF/Rayon-d4 is 1.543 [1.459, 1.604] wall, 1.566 CPU. The existing
frontier8 control on center-peak instead reaches 0.852 [0.788, 0.904]
wall/Rayon-d4, CPU 0.877. Sequential WF/Cpp-seq wall is 0.997 on
center-peak and 0.938 on depth-cap. Each nonterminal recursive node attempts
an 88-byte acquisition; success publishes, joins and releases, while refusal
still enters the parallel recursive body. These controls expose
publication/recursion costs without
establishing that one frontier setting is best for all inputs or selecting
a new compiler default.

Cross-image losses have a different scope. The stable left-peak/wf-leaf-seq/W4
formal/recovered wall ratio is 1.1186 [1.0607, 1.2912], CPU 1.1185
[1.0607, 1.2939], but that clone executes no scheduler calls and reports
zero lanes. At W1 its wall ratio reverses to 0.9327 [0.7999, 0.9588].
The shared native Rust-seq branch also loses 1.2523 [1.0218, 1.3037]
across these images at W4. Source/object sharing does not remove image and
execution variability or attribute these losses to the runtime.

Outside-peak/frontier4/W4 has formal/recovered batch wall 1.0007,
caller CPU 1.0006, but process CPU 1.4071 [1.0515, 2.7311]. Extra CPU
belongs mainly to other threads; existing records cannot split it into
computation, searching and waiting. The existing worker profile observes
a sanitized research image, so it cannot fill that formal-runtime gap.
Any follow-up attribution needs thread/symbol observations at the same batch
boundary. This limitation and the publication costs remain open alongside FIR.

### Preserve the x86-64 victim sequence with a remainder fast path

The restored worker's actual Linux x86-64 code performs `DIVL` once per
victim sweep to compute `(seed >> 33) % count`. Mapping the existing 2ae
W4/N4096/tile64 CPU samples to their original ELF instructions gives
583/3,341 worker samples at that instruction for current, 594/3,327 for its
identical replica, and 440/2,979 for research. Sampling positions are not
isolated instruction costs or a predicted speedup, but this is a concrete
repeated operation in the observed busy path.

For a positive power-of-two count, masking with `count - 1` computes the
same remainder. Add that x86-64 fast path to the maintained `wf__par_find`,
preserving the count snapshot, seed update, exact victim order, general remainder
fallback, atomic operations, storage layout, waiting limits and ordinary ABI.
The count may be non-power-of-two after partial startup; a stale larger count
still has the existing initialized-empty-deque interpretation. No cached mask,
new shared field, publication policy or I/O change is needed.

Local Clang 22.1.8 O2/O3 inspection rejects the initially portable conditional:
on ARM it computes the general remainder unconditionally and then selects
between that and the mask. A branch-likelihood hint does not remove that
division. Retain ARM's original calculation and apply the fast path only to
x86-64, where generated control flow skips the division for power-of-two
counts. No instruction-latency claim or new compilation flag selects this
platform scope; check the actual native images again after CI.

Before selection, verify generated O2/O3 paths, ordinary native execution and
the existing lifetime/deque probes. Add an actual three-participant partial
startup case to the maintained smoke test and its existing callers. Add W3 to
the existing formal matrix where the host has at least three CPUs, to measure
the fallback path's added branch cost alongside the retained W1/2/4 cells.
Keep exact 4fab as previous, repaired historical/research and identical-image
controls, and all existing thresholds. Retention requires a repeated full-call
and CPU benefit at the established loss, with no repeated regression in the
other measured workloads or native targets. Core, first64 and memory results
remain visible. A code-size or instruction-count reduction is not qualification.

Local validation of the x86-64-only candidate passes all eight smoke scenarios,
including three participants, and both 200,000-task deque probes with live
counters. The two deque probes also pass under ThreadSanitizer.
Independent inspection confirms that
all four macOS-target x86-64 O2/O3/statistics combinations skip division on
the mask path; the four ARM text images equal their previous controls, and
all eight lane/slot layout pairs are unchanged. Linux/Windows native codegen
and all-platform timing remain to be verified by CI.

The x86-wide candidate `c8820e9868fdb07d76ea711922437a0a456d1d2d`
completed [compute run 34485715742](https://github.com/mbbill/Whitefoot/actions/runs/34485715742).
All five formal artifacts pass hash, captured-source, repaired-control,
oracle, actual-width and original-reducer replay: 2,940 processes and
6,776,700 checked calls. The performance screens remain failed:

| Native target | Artifact | ZIP SHA256 | Widths | Long / first64 investigate |
| --- | --- | --- | --- | --- |
| Linux x86-64 | 10156097379 | `e41ef86ea558e582d74f0b44c9d3b7708eb096b1cf18bc9dd3a457d24a6c2962` | 1/2/3/4 | 3/120; 20/120 |
| Linux ARM | 10155824224 | `5dca06b3a7dfdb53934ef9a396de5ace2cfb4b4661efe1903011b837e9c9bd8a` | 1/2/3/4 | 1/96; 5/96 |
| Mac Intel | 10156228062 | `3ce18a99fdbdfc5c67980bf36c14bd108afb194f8ff0a1d652fd0baaa2c11a96` | 1/2/3/4 | 35/120; 22/120 |
| Mac ARM | 10156203348 | `47c7de256c36360f8b3b06438955d5ed6219f4c02bfa57d9510e73febeec879e` | 1/2/3 | 16/72; 19/72 |
| Windows x86-64 | 10155883887 | `57de3bb180a3c73e88dc8b80da0acf49f1bda0fa3c8dd83e60d39a8cdf65f67e` | 2/3/4 | 3/72; 5/72 |

The native Linux, Mac Intel and Windows join/worker paths skip `DIVL` for
power-of-two counts and retain it otherwise. Their loop addressing also
changes, so an observed benefit cannot be isolated to division latency. Linux
ARM's executable sections and Mac ARM's complete section contents/addresses
equal previous; the ARM results do not measure a runtime code change.

On Linux's EPYC 7763 guest, W4/N4096/tile64 current/previous full-call is
0.9664 [0.9558, 0.9860] and process CPU 0.9683 [0.9614, 0.9882]. The
identical replica gives 0.9682 [0.9599, 0.9798] and 0.9716 [0.9557, 1.0052].
Ratios are medians of five process pairs [minimum, maximum]; full-call uses
warm-call means. Full-call A/A is [0.9882, 1.0112], CPU
A/A [0.9831, 1.0118]. Core is effectively unchanged (current median 1.0003),
and first64 full-call is inconclusive (1.0132 [0.9778, 1.2828]). Peak RSS is
1.0371 [0.9816, 1.0487], replica 1.0487 [1.0247, 1.0584]; storage layout
is unchanged, but that does not erase the process-memory observation.
Current/research full-call is still 1.0472 and CPU 1.0553.

The retained CPU profile verifies 249 hashes, 80 processes and 26,278 samples
with no reported lost samples. It has no previous control. Current/research
helper samples are 4,016/3,859, including worker-loop 3,178/3,001;
getter samples are 891/849. Remaining consumption and worker activity are
visible, but those sample counts neither independently confirm this revision's
benefit nor distinguish extra searching from workers kept busy by later reads.

The Mac Intel W3/N4096/tile64 fallback cell prevents x86-wide selection:
current/previous CPU is 1.1391 [1.0604, 1.2998], replica 1.1694
[0.9758, 1.2922], with CPU A/A [0.9647, 1.0867]. Full-call is noisy,
1.0208 [0.8891, 1.9134], and does not establish a latency regression.
Nevertheless the repeated CPU increase does not satisfy the retention rule.
Windows W4/N4096/tile64 gives full-call 0.9888 [0.9405, 1.0134] and CPU
1.0130 [0.9500, 1.0519]; it establishes no joint benefit. Windows has no
research execution control. Neither ARM target has a five-pair full/core/CPU/RSS
loss above 5% versus previous; their original failed screens remain visible.

Restrict the follow-up fast path to Linux x86-64. Restore the general remainder
on Mac Intel and Windows; retain the new partial-three lifetime test and W3
matrix on every applicable target. This preserves Linux's measured code path
without selecting the unqualified x86-wide expansion. Another native cohort
must confirm that scope and the Linux benefit; no threshold or reference changes.
The ordinary-command losses and all-platform qualification remain open.

All five ordinary-command artifacts replay 7,620 process results and their
original failed reducers; no matched default candidate/previous cell has
five-pair wall/CPU/RSS loss above 5%. Six Windows default cells have zero CPU
readings and remain unavailable for CPU comparison. This is not parity with native controls.
For shape4/N4096/W4, default/static full-command wall is 3.363 on Linux
x86-64, 3.702 on Linux ARM and 3.087 on Windows; the default diagnostic starts
zero helpers. On Linux x86-64, default/previous wall is 0.9991 and CPU 0.9995.
The existing work60000 form starts three helpers and reduces wall/static to
1.442, but has no previous-work60000 process pairs, so that result cannot
select this runtime change. Mac ARM's panel covers only W1/2; its
shape0/N4096/W2 default/static wall is 1.534 [1.308, 1.763], with
default/previous 1.019 [0.894, 1.036]. These remain compiler-policy and
end-to-end losses, separate from the measured runtime revision.

Local follow-up inspection reproduces previous macOS `__text` at O2/O3 with
statistics on/off for both architectures. Preprocessing the Linux-selected
core yields the same tokens as c882; this is a scope check on the Mac SDK,
not a replacement for native Linux compilation and timing.

Exact c882 passed canonical `make check`, all 12 gate jobs and both I/O-host
jobs. Three I/O-bench jobs passed; Windows compute timing remained unstable
after its two cohorts and failed. Correctness is not performance qualification.

The Linux-only revision `a7150751a4460e4cd51c03ffd84767013d6f98e3`
completed [confirmation run 34488895756](https://github.com/mbbill/Whitefoot/actions/runs/34488895756).
All five formal artifacts again replay 2,940 processes, 6,776,700 checked
calls, captured sources, repaired controls and original reducers. The failed
screens remain failed:

| Native target | Artifact | ZIP SHA256 | Widths | Long / first64 investigate |
| --- | --- | --- | --- | --- |
| Linux x86-64 | 10157276740 | `33a676f67061d79ad70e5e8dd77f50513fb219bc4318f7805f4d058727c90975` | 1/2/3/4 | 10/120; 24/120 |
| Linux ARM | 10157138828 | `17bb73f60161ae2dca3fa18da236376d051e1de01d6a8301a622d66f43bbef21` | 1/2/3/4 | 0/96; 8/96 |
| Mac Intel | 10157784914 | `661ac9de3c039e4b03ce976af023386a13dbff70c244531556a30e3645c75b23` | 1/2/3/4 | 14/120; 22/120 |
| Mac ARM | 10157364610 | `908d0c7dd79377779d7d1a3c28ecc768799f23b4c045a445144e7b75f165fb44` | 1/2/3 | 20/72; 28/72 |
| Windows x86-64 | 10157184792 | `d52ac1b0f247f4a81d4ca4e7ee3e479ed07e9ec342f988cefacd5cca4c2b8f50` | 2/3/4 | 4/72; 11/72 |

Linux x86-64's native executable sections equal c882's, confirming the
retained code path. On the same EPYC 7763 guest class, W4/N4096/tile64
current/previous full-call is 0.9681 [0.9438, 0.9928], CPU 0.9606
[0.9535, 0.9934]; replica gives 0.9643 [0.9477, 0.9991] and 0.9685
[0.9505, 0.9782]. Both images improve both measures in all five pairs.
Full-call A/A is [0.9902, 1.0039], CPU A/A [0.9864, 1.0156]. Core is
1.0021 [0.9701, 1.0119], RSS 1.0000 [0.9515, 1.0080]. First64 full-call
remains inconclusive at 1.0152 [0.9287, 1.0759]. Current/research still
costs 1.0524 full-call and 1.0520 CPU; the local benefit is not parity.

There is also a repeated phase-specific cost at Linux W4/N4096/tile16.
Current/previous core is 1.0654 [1.0299, 1.1432], replica 1.0765
[1.0063, 1.1222]. The preceding c882 cohort also has all five core pairs
above previous for both images, with medians 1.0469 and 1.0377.
Current full-call/CPU are 0.9959/0.9971 and replica 1.0044/0.9934 in
the confirmation cohort. Core A/A is broad [0.9178, 1.0840], but repeated
direction across two cohorts and images cannot be discarded because each
individual pair is not more than 5% slower. No corresponding full-call or
process-CPU increase is established; a shift between measured phases is
unexplained. This remains a qualification issue for the Linux candidate.

Linux ARM's executable sections, both Macs' complete section contents and
addresses, and Windows's complete `.text` match previous. The restored Mac
Intel W3/N4096/tile64 cell gives current/previous full-call 0.9948
[0.9656, 1.0534], CPU 1.0021 [0.9662, 1.0493]; replica gives 1.0247
[0.9600, 1.0463] and 1.0212 [0.9691, 1.0504]. The prior CPU risk is
not reproduced here. This is restoration evidence, not a cross-cohort speedup.
Mac Intel still has large identical-image drift: W3/N65536/tile64
replica/research full-call/CPU medians are 1.1778/1.1692, current
1.0184/1.0191, with full-call A/A [0.8310, 0.9632].

Mac ARM retains a clearer historical loss at W2/N65536/tile16:
current/old full-call 1.1124 [1.0784, 1.4059], CPU 1.1010
[1.0678, 1.1595]; replica 1.1259 [1.0773, 1.1968] and 1.0872
[1.0628, 1.1805]. Both images lose both measures by more than 5% in
every pair. Its unchanged candidate/previous code cannot attribute that
existing loss to this revision. Windows has no five-pair full/core/CPU/RSS
loss above 5% versus previous or old; this does not erase its failed screens.

The five ordinary-command artifacts independently replay 7,620 processes.
Linux x86-64, Linux ARM and Windows keep their failed reducers. Both Macs'
reducers exit zero while leaving extensive noisy-open rows: green exit alone
does not qualify them. Shape4/N4096 default/static full-command ratios are
3.3529 on Linux x86-64 W4, 3.6971 on Linux ARM W4, 3.0190 on Windows W4,
3.3092 on Mac Intel W4 and 1.7374 on Mac ARM W2. Default diagnostics start
zero helpers. Linux x86-64 default/previous wall is 0.9982
[0.9973, 1.0020], so its large static-reference loss remains a compiler-policy
gap, not a measured new runtime regression. Seven Windows default cells
contain zero CPU readings and remain unavailable for CPU comparison.
Mac Intel shape5/N65536/W2 also retains replica/previous wall/CPU medians
1.2261/1.2064, while current has 1.0978/1.1045 and wide A/A ranges;
the larger loss is not consistently reproduced by the identical current image.
All-platform performance qualification remains open.

Exact a715 passed local canonical `make check`, all 12
[gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34488895793), both
[I/O-host jobs](https://github.com/mbbill/Whitefoot/actions/runs/34488895858)
and all four [I/O-bench jobs](https://github.com/mbbill/Whitefoot/actions/runs/34488895728).

### Refuse execution when the exhaustion floor cannot be installed

The delivery review found inherited POSIX paths that continued after stack
bound queries, alternate-stack allocation, `sigaltstack`, `sigaction` or
page-size discovery failed. Native fault injection confirmed that failed
allocation and either signal facility still reached the command body.
This is a runtime-start limitation under SCOPE-3, not a source-proof defect.
The maintained floor now diagnoses setup failure and aborts before an
unprotected command or worker can execute. Failed alternate-stack installation
releases its new mapping; successful stack-attribute queries are always
destroyed. Normal scheduling, ABI and I/O paths are unchanged.

The maintained exhaustion test links the actual floor with substituted native
facilities, covering 13 command/worker cases: two normal controls and eleven
refusals. It requires the setup diagnostic, SIGABRT and no body output for
each refusal. All 23 exhaustion tests pass locally. The deep-recursion test
description is also corrected: nested helping uses an existing stack, so
equal stack reservations do not prove universally equal available depth.
Native Linux execution and the exact new revision's full gate remain required.

## Prior unified-runtime delivery scope (paused on 2026-09-09)

The owner requires delivery through ordinary `whitefootc --par source.wf -o
program`, using the maintained shared runtime in `compiler/src/backend/sched/`.
Refactor that implementation where its design or code quality obstructs compute
performance. Do not choose a different runtime because a module contains I/O.
The existing research runtime is a frozen comparison input during this work,
not the implementation home; retire the duplicate once its comparison purpose
is served. I/O optimization is deferred, but completion, mixed compute/I/O
progress, frame lifetimes and exhaustion handling must remain correct. Changes
to source execution semantics or public ABI require discussion with the owner.

The target set is the compiler's closed set in
[`target.rs`](../../../compiler/src/backend/target.rs), not the smaller set
currently covered by compute measurements:

| Target | Required evidence |
| --- | --- |
| `aarch64-apple-darwin` | Native CI correctness and performance |
| `x86_64-apple-darwin` | Native CI correctness and performance |
| `aarch64-unknown-linux-gnu` | Native CI correctness and performance |
| `x86_64-unknown-linux-gnu` | Native CI correctness and performance |
| `x86_64-pc-windows-msvc` | Native CI correctness and performance |

Completion requires measured acceptance on **every row**. Cross-compilation,
emulation, local M1 measurements, missing runners or correctness-only jobs do
not substitute for a row. Keep an unavailable or noisy row unresolved.

Before selecting runtime changes, use these experimental criteria:

- Compare candidate and baseline on the same CI machine, alternating order,
  with identical inputs, worker budgets, arithmetic, vectorization and build
  options. Scheduling comparisons disable SIMD and LTO on both sides. Retain
  exact revisions, sources, commands, binaries and raw samples.
- First isolate runtime cost with the identical generated WF object, then
  measure normal CLI executables with pure compute and compute before/after
  light I/O. An experimental linker comparison alone does not deliver the goal.
- Cover the distinct computations in [WORKLOADS.md](WORKLOADS.md), including
  fine/coarse work, skew, nested composition and repeated/bursty work. Check
  outputs with independent oracles. Retain startup, wall time, CPU cost,
  memory and scaling separately; instrumented runs explain ordinary timings.
- Start with five alternating process pairs per cell, keeping warm invocations
  inside a process distinct from independent process samples. Investigate a
  repeatable candidate/baseline wall-time ratio above 1.05 in any cell. Use
  further independent cohorts to resolve noise rather than discard outliers
  or average a loss away. An unresolved cell cannot pass acceptance.
- Matching the research baseline requires each cell to be within 5% on wall
  time, with repeatable CPU or memory increases above 5% separately explained
  and resolved; a wall-time win cannot hide excessive spinning. This 5% band
  is an initial engineering equivalence criterion, not a universal noise
  estimate. Native references from [BASELINES.md](BASELINES.md) still test
  whether the baseline itself leaves avoidable scheduling cost.
- The old research runtime is POSIX-only. Windows must use its existing formal
  runtime as the before-control and matched native references to qualify the
  resulting scheduler; an untested port of the old research code would not be
  a qualified baseline. Report this difference explicitly.

The first bounded candidate gave current-stack compute helping priority over
taking another stack at a compute join. A local M1 FIR screen with the same
scalar WF object found no consistent improvement; at four workers the 64-output
tile mean went from 40.9 to 47.4 microseconds, versus 15.1 for the recovered
control. That change was reverted. This screen used five processes per cell,
256 warm calls per process, 4,096 outputs and 16 taps; it is not CI acceptance.

The current candidate avoids locking the ready list merely to observe that it
is empty. List mutation stays locked, the head is read and written atomically,
and the wake-epoch protocol is unchanged. The native smoke and all four
scheduler enumeration configurations pass. Initial local timings are mixed,
so performance selection remains open: do not describe this as an accepted
optimization. A sampled long FIR run found substantial condition-variable
waiting, yielding and wake-mutex contention; ready-list locking alone does not
explain the gap. Sampling includes output verification and idle threads and
does not measure compute-region CPU fractions.

`make -C research/experiments/compute-runtime formal-screen OUT=<fresh-path>`
reproduces the initial POSIX screen through the current compiler, with fixed
formal-before revision `188088d41552d0d3bccf8368798dcc44702bf75c`, a checked
unchanged recovered baseline, and the candidate formal runtime. CI covers the
four POSIX targets and Windows. The Windows job invokes the same script with
the native MSVC-target compiler and existing Windows runtime leaves; it compares
the fixed formal-before scheduler/floor, candidate, frozen previous revision
and same-source longer-idle-window control. A byte-identical candidate replica
measures process/host variability. Previous keeps its own frozen Windows
host/completion sources; candidate and the oldest before control link current
host/completion sources beside matching private headers.
This overlay is needed by generated host diagnostics; "before"
does not mean an entirely historical Windows runtime. No research runtime
is ported. Its benchmark uses QueryPerformanceCounter for elapsed time,
GetProcessTimes for whole-process CPU time, and peak working set for memory;
context-switch counts are explicitly unavailable. CPU times' 100-ns units do
not imply that small batches have 100-ns accounting resolution. Strong native
parallel references and CPU acceptance remain required on Windows. The existing
mixed-runtime protocol remains in `io-bench.yml`. This screen is FIR-only, its normal CLI execution is a
correctness check, and its threshold evaluates warm core wall time only. It
cannot complete the broader workload, CPU, CLI timing or five-target goal.
Linked-image layout can also change despite using identical WF object bytes;
small differences require independent confirmation and attribution.

The separate [ordinary CLI Mandelbrot panel](../../experiments/compute-runtime/README.md#ordinary-cli-mandelbrot)
now measures normal executables, including startup, input generation, rendering,
digest and shutdown, against scalar native serial/static commands. Its
five-target CI jobs supplement the identical-object FIR comparison. The first
local cohort exposes missed parallelism in small expensive loops under the
maintained split-budget policy; it does not qualify the broader goal. Kernel
versus scheduling attribution and strong dynamic references remain necessary.

The first [four-target native CI screen at `708e3c4d`](https://github.com/mbbill/Whitefoot/actions/runs/34343071425)
completed its oracle, normal-CLI correctness and actual-pool-width checks, but
all four targets failed its performance band. Representative median paired
candidate/recovered ratios (16 taps, tile 64, five processes per cell) are:

| Native target | Participants | 4,096 outputs | 65,536 outputs |
| --- | ---: | ---: | ---: |
| Linux x86-64 | 4 | 1.741 | 1.390 |
| Linux AArch64 | 4 | 2.088 | 1.512 |
| macOS x86-64 | 4 | 5.444 | 1.206 |
| macOS AArch64 | 2 | 1.777 | 1.062 |

The macOS AArch64 runner exposes three CPUs, so the screen measures widths one
and two there. Linux x86-64 exposes four logical CPUs on two SMT cores; these
rows are within-host comparisons, not comparable four-physical-core machines.
The run's `formal-runtime-<target>` artifacts retain raw samples, sources,
binaries, build options and host identity. In one Linux four-participant small
batch, candidate voluntary/involuntary switches total 1,023 versus 50 for the
control; startup is 2.151 ms versus 0.210 ms and peak RSS 25.16 MB versus 2.33 MB.
These process measurements include verification and are not core-only CPU
profiles. The [Windows mixed timing run](https://github.com/mbbill/Whitefoot/actions/runs/34343071339)
failed because `io-warm` remained unstable across two complete cohorts.
[Linux and Windows completion correctness](https://github.com/mbbill/Whitefoot/actions/runs/34343071312)
passed; that does not make Windows performance qualified.

The next bounded candidate initializes only the configured lane prefix and the
trailing status/idle metadata. The original full-capacity clear touches about
20 MiB even with one participant. The prediction is lower startup time and RSS
without a warm regression. It preserves the original hot-data layout and all
public-call/task-frame ABIs. A first variant moved metadata before the lanes;
two independent local M1 cohorts found a repeatable roughly 6% small-input
loss, so that layout change was removed. Clearing the two live regions instead
retains the startup saving without needing an enumerator layout change. The
poisoned-storage smoke checks that live fields do not depend on pristine BSS.
Local FIR measurements use the same WF object, widths one/four, 16 taps,
4,096/65,536 outputs, tiles 64/1,024 and five alternating processes of 256 warm
calls per cell. Against the published shared-runtime control at `708e3c4d`, the
unchanged-layout candidate is within the wall-time band in seven cells; the
remaining small-input median ratio is 1.060 with a wide 0.775–1.116 paired range.
This is not parity with the recovered runtime. Warm acceptance remains open;
startup savings alone do not select it.

The compute-join hypothesis is that short stolen work finishes sooner than
the shared park/resume round trip. Give a compute join a bounded interval of
current-stack pop/steal/help and processor pauses before taking an EMPTY stack.
READY stacks remain immediately eligible, and every empty-handed turn still
drains I/O progress; after the interval the existing park/exhaustion protocol
applies. This changes internal scheduling policy, not calls or source semantics.
Compare zero, 16, 64 and 256 turns using identical WF object bytes; reject a
wall-time gain bought with a repeatable CPU-cost increase above the stated band.
The earlier rejected helper-first change had no bounded wait when no work was
available, so it did not test this short-completion hypothesis. Qualify mixed
progress and all enumerator configurations before publishing any selection.

The branch candidate uses 256 outer turns following the local zero/16/64/256
screen. This bounds neither callback duration nor recursive stack depth. The
final local `formal-screen` (five processes, 64 warm calls, widths one/two/four)
passes all 24 wall comparisons against the fixed formal-before control, but
fails 12 of 24 against the recovered runtime. Selected four-participant results
are below; CPU is the whole measured batch including checks, not core-only.

| FIR input / tile | Before core mean | Candidate core mean | Recovered core mean | Before / candidate batch CPU |
| --- | ---: | ---: | ---: | ---: |
| 4,096 / 64 | 36.59 us | 28.14 us | 14.53 us | 15.72 / 10.91 ms |
| 65,536 / 1,024 | 149.09 us | 135.75 us | 133.80 us | 72.78 / 59.78 ms |

For the first row, startup falls from 1.471 to 0.176 ms and peak RSS from
22.51 to 3.36 MB. Mean process context switches fall from 647 to 340; for the
second row they fall from 1,135 to 268. These are local candidate results, not
cross-platform acceptance. The screen still does not time the normal CLI path,
and its pool-width check is not a witness of useful work on every worker.

All four reduced-bound enumeration configurations pass with one help turn;
the (two-thread, four-stack) sweep explores 44,732,346 states. This checks the
protocol at that bound, not all production histories. The native smoke runs
32 successive sibling tasks on the caller's existing stack while the other
workers hold its join target; every result is checked and joined before frame
release. The default passes, while zero help turns fail its current-stack
assertion. Its mixed I/O cases and poisoned initialization also pass. Independent
scoped review found no remaining issue in these changes after correcting the
model-coverage claim.

The [native CI screen at `aeb35be5`](https://github.com/mbbill/Whitefoot/actions/runs/34346052104)
still fails the performance band on all four POSIX targets. Selected median
paired candidate/recovered wall ratios are below; widths are configured pool
participants, not proof of useful work on every participant.

| Target | Width | 4,096 / tile 64 | 65,536 / tile 64 | 65,536 / tile 1,024 |
| --- | ---: | ---: | ---: | ---: |
| Linux x86-64 | 4 | 1.857 | 1.412 | 1.353 |
| Linux AArch64 | 4 | 2.230 | 1.516 | 1.037 |
| macOS x86-64 | 4 | 2.818 | 1.965 | 0.940 |
| macOS AArch64 | 2 | 1.729 | 0.985 | 1.011 |

Linux x86-64 at 65,536 / tile 256 reaches 0.990, while smaller tasks still
lose substantially. Some paired ranges are wide: Linux AArch64 at 4,096 /
tile 64 spans 1.303–5.332. A favorable median in such a cell does not establish
stable equivalence. Canonical local `make check` passes at `aeb35be5`, including
the full native conformance and snapshot adapters; performance acceptance is
separate and remains open.

Native correctness review found two defects in the shared deque: plain cell
accesses race with stale thieves during ring reuse, and acquire-only thief
index loads lack the ordering needed against the owner's claim. The maintained
core now uses relaxed atomic cell loads/stores and sequentially consistent
thief top/bottom loads. For the duplicate-claim history, the SC order is owner
bottom decrement, owner top check, first thief CAS, second thief top read,
second thief bottom read. The last read cannot select the older bottom. This
argument assumes no complete 64-bit counter rollover; it is not a complete
weak-memory proof. The ordering issue and atomic-cell requirement agree with
the analysis in [Lê et al., PPoPP 2013](https://fzn.fr/readings/ppopp13.pdf).
Apple ARM code generation changes from `ldapr` to `ldar` for the index loads;
the repair's performance cost must be measured, not assumed zero.

Live diagnostic counters now use relaxed atomic reads and single-writer
load/store increments. The writer is the physical thread, reloaded after a
possible migration. These observations add no synchronization or scheduling
edge and are not one simultaneous snapshot. The enumerator does not branch on
counter accesses; it now does branch on ring-cell accesses and hashes the full
ring, retaining stale-reader and unpublished-push states. All four reduced
configurations pass after that change, with 44,819,639 states for two threads
and four stacks. This remains SC interleaving evidence, not weak-memory proof.

The compiler-owned `sched-deque-test` exercises the actual core on ordinary
host stacks with eight slots, 200,000 tasks, three thieves, owner pops and a
live observer. It requires exactly-once execution, complete slot return and
monotone observed steal counts. Both native M1 and ThreadSanitizer runs pass.
The same probe reports a counter race with the old core; after applying only
the counter repair to that old core, it reports the separate ring-cell race.
No race suppression is used. POSIX canonical checks and Windows native CI run
the probe; Linux CI additionally runs it under ThreadSanitizer. At `0f1603b2`,
the four POSIX native probes and both [host correctness jobs](https://github.com/mbbill/Whitefoot/actions/runs/34350981830)
pass, including Windows's native probe and Linux's ThreadSanitizer run.
The [partitioned repository CI](https://github.com/mbbill/Whitefoot/actions/runs/34350982089)
also passes on that revision. The canonical local `make check` for `0f1603b2`
completed successfully, including the full native conformance adapter.

The repair's local M1 cost comparison uses byte-identical WF object files,
alternating repaired/`aeb35be5` binaries, widths one/four, inputs 4,096/65,536,
tiles 64/1,024 and five processes with 256 warm calls each. All eight median
wall ratios are within 5% (0.933–1.034); whole-batch CPU median ratios span
1.011–1.050. The four-participant small-input/tile-64 wall ratios span
0.817–1.152, so this does not establish a speedup or stable equivalence there.
The broader 64-call FIR screen still fails. Correctness selects this repair;
native CI and further controlled comparisons must establish its cost.

The recovered runtime remains frozen, including its acquire-only thief index
reads (its ring cells already use atomics);
its timing is historical comparison evidence, not a correctness-qualified
implementation to restore. The shared runtime still updates counters while
the recovered timing build disables them. This accounting asymmetry and
broader fair native references remain unresolved before final qualification.

A local maintained-core layout experiment separated owner-written deque bottom
from the thieves' top and isolated each physical thread's observed counters
on 128-byte boundaries. The current layout packs 136-byte thread records,
allowing independent counter writers to share a line. Two alternating M1
cohorts used identical scalar WF objects, widths one/four, inputs 4,096/65,536,
tiles 64/1,024, five process pairs and 1,024/256 warm calls for small/large
inputs. At four participants, 65,536 / tile 64, candidate/original wall ratios
were 0.958 and 0.922, with CPU ratios 0.974 and 0.958. Small-input/tile-64 wall
ratios were 0.980 and 1.020. One single-participant large-input cell had an
unresolved 1.114 RSS ratio in the second cohort, with roughly 0.56 MB variation
inside both sets. The layout is not selected or retained in the implementation;
resolve the Windows policy regression and memory observation before reopening
this candidate. Native smoke and the 200,000-task deque probe passed; no claim
of cross-platform layout qualification follows.

The [first complete Windows FIR screen at `b4a3283d`](https://github.com/mbbill/Whitefoot/actions/runs/34349696349)
passes oracle, normal CLI and pool-width checks, but fails three of 24 wall
cells against its historical scheduler/floor control. At four participants,
4,096 / tile 1,024 has median paired ratio 2.781, range 2.712–2.981. Its five
process means are 60.94–63.04 us versus 21.12–22.95 us; pooled warm-call p50 is
60.9 versus 15.9 us and p95 is 76.5 versus 69.1 us. These are different sample
levels, not interchangeable confidence estimates. The host is Windows Server
2025, EPYC 7763, two cores/four logical CPUs, high-performance power plan,
Clang 20.1.8. QPC frequency is 10 MHz. Whole-process CPU readings for these
short batches jump in 15.625-ms multiples and include zeros; they cannot
establish CPU efficiency. Do not attribute the regression to helping alone:
the two controls contain several scheduler changes. The next causal control
holds current sources fixed and sets only compute helping to zero, with
separate longer diagnostic batches for current-runtime counters and CPU.

The first local M1 same-source help comparison completed all 480 process
samples and 48 separate diagnostic processes. At four participants and 65,536
outputs, candidate/help0 median wall ratios are 0.878 at tile 64 and 0.909 at
tile 1,024. Small-input ratios are less stable; this does not answer the
Windows regression. One 4,097-call diagnostic process at 4,096 / tile 1,024
records 677 parks for the candidate versus 5,841 for help0, with whole-batch
CPU 418.6 versus 522.3 ms. The report and exact lane/warm-call counts are
validated per diagnostic process. One process is explanatory evidence, not
CPU-performance qualification. The wall verdict is saved before diagnostics
so a later diagnostic failure cannot hide completed measurements.

The [five-target help control at `0f1603b2`](https://github.com/mbbill/Whitefoot/actions/runs/34350982086)
completed all formal screens and their diagnostics, but every platform still
has failing wall cells. At Windows four participants, 4,096 / tile 1,024,
candidate/before is 2.773 (paired range 1.507–3.130); candidate/help0 is 1.128
(0.565–1.788). The zero-help process means are 33.63–55.37 us, still well above
before's 19.22–24.38 us. Thus the helping budget alone does not account for the
regression. Windows artifact `10103808989` has ZIP SHA-256
`7cc55bf141adc868b9e8499d5cd08bea374365b5ff2fa9ba394d4001ba181bea`.

For that cell, separate 4,097-call diagnostic processes record candidate/help0
batch wall 355.4/340.1 ms and CPU 937.5/984.4 ms. Their live process-total
scheduler reports show stack parks 48/7,562, including startup/selection rather
than exactly the batch interval. Whole-batch costs include verification;
one diagnostic process per mode is not
independent CPU qualification. The enormous reduction in stack parks does not
produce a corresponding wall reduction. These are stack switches, not host
sleeps. The next measurement reads the existing current bridge's atomic wait
announcements and host wake signals at batch boundaries, including for the
historical scheduler overlay without reading its unsafe scheduler counters.
The getters live in the maintained compiler, do not initialize the bridge,
and add no updates to hot paths. An announcement may be cancelled before
sleeping, and a signal does not count awakened threads.

The next question is whether faster empty-ready checks shorten the fixed-round
idle window enough to put workers to sleep between bursts, making the next
call pay a host wake. This is a hypothesis, not an attribution. A large increase
in wait announcements/signals accompanying the regression would support a
focused idle-window control; comparable counts would send the investigation
back to other costs. No runtime policy was changed for this measurement.
POSIX attribution images still omit the completion bridge while Windows images
include it; full-link POSIX timing and ordinary CLI timing remain required.
The new getters and FIR instrumentation pass strict C syntax checks and
full-link M1 correctness smokes at one/four participants; these smokes ran
during the canonical check and are not performance measurements. Independent
review found no implementation defect in this diagnostic delta; its metadata
and process-total versus batch-boundary clarifications are incorporated.

The [Windows wait observations at `076a476b`](https://github.com/mbbill/Whitefoot/actions/runs/34352414576)
were recorded on an EPYC 9V74, two cores/four logical CPUs, Windows Server 2025,
Clang 20.1.8, high-performance power plan. This is a different processor from
the earlier EPYC 7763 runs; compare controls within this run. At four
participants, 4,096 / tile 1,024, the five candidate/before paired wall ratios
have median 2.146 and range 1.263–2.837. Each short process has 64 warm calls
plus one retained first call. Its observations are:

| Mode | Five warm process means (us) | Batch wait announcements | Batch wake signals |
| --- | --- | --- | --- |
| Before | 20.66, 16.10, 16.14, 16.59, 20.43 | 33, 29, 24, 26, 35 | 45, 48, 35, 43, 54 |
| Candidate | 44.34, 45.68, 43.92, 20.95, 32.58 | 198, 193, 196, 11, 111 | 194, 199, 196, 10, 108 |
| Zero help | 33.95, 36.34, 34.15, 32.32, 26.45 | 129, 114, 127, 129, 75 | 176, 154, 188, 183, 115 |

The candidate's slower processes coincide with many more wait announcements
and wake signals. The separate long diagnostic process instead has 81
announcements over 4,097 calls and mean warm time 18.16 us, compared with
before's 244 announcements and 19.23 us. It does not reproduce the short-run
loss, so neither ignoring the short samples nor attributing the difference to
call count alone is justified. Long-batch candidate/before CPU is 468.75/437.5
ms and wall is 179.74/177.85 ms; these include verification and remain one
process per mode. Artifact `10104466331` has ZIP SHA-256
`e820fa5a9d45ebc9db8490b939aec00c3f67294df7528229480948ae284fe9e5`.

This co-observation motivated the idle-window control. `idle4096` uses
the identical current runtime and WF object with only the existing
`WF_SCHED_IDLE_SPIN_ROUNDS` set to 4,096 instead of 256. The default is unchanged.
The predicted result is fewer host waits together with removal of the short
Windows loss. If waits fall without wall improvement, that hypothesis is
insufficient. Even a wall improvement cannot select the policy if longer
spinning creates an unresolved CPU regression. The same five-target screen
retains all other cells, CPU observations and existing controls. CI now also
triggers on completion and Windows host changes, whose code is part of the
Windows timing images.
The full-link M1 smoke passes with the longer window and reports 4,096 rounds;
diagnostic validation rejects that report when 256 rounds are expected.
Independent review confirmed the control's scope, wiring and recorded artifact
figures.

The [five-target screen at `6060cc67`](https://github.com/mbbill/Whitefoot/actions/runs/34353534080)
completed, with performance failures on every target. Its separate
[12-job gate](https://github.com/mbbill/Whitefoot/actions/runs/34353534106) and
[Linux/Windows I/O checks](https://github.com/mbbill/Whitefoot/actions/runs/34353534059)
passed. On Windows, the EPYC 7763 host had two cores/four logical CPUs,
Windows Server 2025, Clang 20.1.8 and the high-performance power plan. At four
participants, 4,096 / tile 1,024, the five paired candidate/idle4096 wall
ratios have median 3.7455, range 1.6301–4.4359; idle4096/before has median
0.803. The separate 4,097-call diagnostic processes recorded:

| Mode | Warm mean (us) | p50 / p95 (us) | Batch wall (ms) | Batch CPU (ms) | Wait announcements / signals |
| --- | ---: | ---: | ---: | ---: | ---: |
| Before | 17.326 | 15.1 / 27.5 | 186.052 | 734.375 | 258 / 362 |
| Candidate, idle 256 | 53.329 | 59.5 / 68.0 | 353.217 | 765.625 | 9,825 / 9,888 |
| Zero join help | 41.493 | 46.8 / 66.2 | 299.462 | 796.875 | 6,577 / 9,486 |
| Idle 4,096 | 13.999 | 13.0 / 17.7 | 189.939 | 750.000 | 0 / 0 |

This supports idle/wake overhead as a major loss in this short cell. It does
not select 4,096 as the default: at four participants and 65,536 / tile 64,
idle4096/candidate wall is 0.980 while diagnostic batch CPU is 1.261;
tile 256 is 0.985 wall and 1.203 CPU; tile 1,024 is 0.994 wall and 1.729 CPU.
Those cells still enter host waits after the longer spin. Increasing a fixed
window buys burst latency but spends CPU before long idle gaps. CPU comes
from one diagnostic process per mode, includes verification, and is quantized
in 15.625 ms increments; it is a reason to investigate, not repeated CPU
qualification. The default remains 256. Artifact `10105001028` has ZIP SHA-256
`59dd8807b83927faa6cd4a5a60c031744077258b1535ee0bc5cfa9aeeabd53c5`.

The four POSIX artifacts from the same run reinforce that the longer fixed
window is not a portable default. These are idle4096/candidate ratios; wall is
the median paired warm-core ratio, while CPU is the separate longer diagnostic
batch (one process per mode, including verification):

| Target | Participants | 4,096 / tile 1,024 wall / CPU | 65,536 / tile 64 wall / CPU |
| --- | ---: | ---: | ---: |
| Linux x64 | 4 | 0.635 / 0.941 | 1.186 / 1.415 |
| Linux ARM64 | 4 | 0.772 / 0.948 | 1.145 / 1.237 |
| macOS x64 | 4 | 0.746 / 1.594 | 1.019 / 1.117 |
| macOS ARM64 | 2 | 0.621 / 1.599 | 1.059 / 1.163 |

The small Linux cell's voluntary context switches fall from 8,769 to 27 on
x64 and 9,369 to 22 on ARM64. In the large tile-64 cell they remain similar
(1,606/1,588 and 2,119/2,192), despite longer spinning. These process-wide
counts support the idle-gap explanation without equating a context switch to
a particular runtime park. Darwin reports zero voluntary switches in these
samples, which is not evidence that no wait occurred. Other tiles also retain
CPU increases. Every extracted file matched its artifact manifest; ZIP hashes
are:

| Artifact | ZIP SHA-256 |
| --- | --- |
| `10104829158` (Linux x64) | `e309f48d11a2dc5bddea69eca754d0ba3ec66ee51c91941b6e40a84743eb8f2f` |
| `10104872709` (Linux ARM64) | `c6f3b1c811b5d37c003d178db649f81b6299203625652298b67291784d619b6a` |
| `10104910285` (macOS x64) | `a5501261e72fb2b20921f57c17cc8ffcbfbe37217d2dbea214cc640822b1935f` |
| `10105005239` (macOS ARM64) | `0a3b14bddb39736a3105c921ad31b4e32823bde4165fa6e497437aa63c80105c` |

A local four-participant M1 full-link/core-only comparison used the same scalar
WF object, inputs 4,096/65,536, tiles 64/1,024, and five alternating process
pairs with 1,024/256 warm calls. Median wall ratios were 0.833, 1.224, 1.006
and 0.963 respectively, with broad paired ranges (0.336–1.363 in the first
cell). This is unresolved local noise, not a selection ground for removing
the completion bridge or a substitute for normal CLI and mixed-program timing.

## Completion ordering candidate

The maintained core is testing release publication of DONE while retaining
the SC COMPLETING store, waiter observation/claim, and park registration and
recheck. Both the core's in-place idle recheck and the bridge's host-stack
fallback recheck now use SC. Their acquire-only forms did
not establish the following SC-order argument. No state, ABI, waiter ownership,
stack switch or I/O progress mechanism changes.

If the publisher misses a still-live registration, its SC waiter observation
precedes that registration in the SC order. The preceding COMPLETING store therefore
precedes the parker's subsequent SC state recheck, which cannot still observe
the old PENDING initialization. It sees COMPLETING or the later DONE and
avoids an unnotified sleep. If the publisher claims the registration, the
existing phase handshake owns its wake. A cancellation or replacement registration
does not inherit an old check: every new registration has its own SC recheck,
including after a failed publisher claim. DONE remains the final record access;
its release store and the joiner's acquiring read publish the result before
the joiner may release the frame. This uses the mixed SC/non-SC load rules in
[C11 draft N1570, 7.17.3 paragraph 6](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf).
Deleting COMPLETING instead is unsound: a publisher could see NULL, a parker
could register and read PENDING, and only then would DONE be stored with no
claimed waiter to wake.

The experimental criteria are unchanged protocol-enumerator and native
concurrency checks, code generation on all supported targets, and measured
task/FIR costs against the same current source with SC DONE. Enumeration
checks SC histories, not weak-memory executions; the ordering argument and
independent review remain necessary. A removed x86 locked instruction is a
cost hypothesis, not a measured end-to-end improvement. This candidate is
separate from the idle-window CI control and is not performance-qualified.

Focused M1 checks passed the native smoke, 200,000-task deque probe, its
ThreadSanitizer build, and all four SC enumeration configurations (the largest
visited 44,819,639 states). The completion default-route probe and its
ThreadSanitizer build also passed with the full bridge and localhost TCP.
The existing protocol-cost benchmark now also
links the maintained shared scheduler directly, using the same held-helper
setup, output oracle and zero-additional-steals assertion as its recovered
baseline. `check-protocol-cost` passed all three images at widths two/four and
depths one/eight/32. Only the recovered image is sanitizer-instrumented in that
target; the shared deque has its own ThreadSanitizer check. The existing
`protocol-cost-calibrate` target still calibrates the recovered runtime only.

A separate M1 owner-local comparison used five alternating process pairs,
four participants with three helpers held, 100 ms warmup, and eight warm
samples of 262,144 tasks each. Candidate/SC-before median task-time ratios
were 0.9994, 1.0020 and 0.9986 at depths one/eight/32, with all paired ratios
between 0.9930 and 1.0219. There is no demonstrated M1 gain. Cross-target
compilation of the actual core shows identical completion-function assembly
on both ARM targets, and final DONE publication changes from `xchg` to `mov`
on all three x64 targets. The Apple ARM in-place idle recheck changes from
`ldapr` to `ldar`. Cross-compilation is instruction inspection, not native
platform qualification.

The next five-target screen replaces the zero-help attribution control with
`previous`, frozen maintained scheduler/floor sources at `6060cc67`. Zero-help
already isolated the join-help question and does not isolate this memory-order
change. Both the original before and recovered controls remain, as does
idle4096; no performance threshold is relaxed. Every image uses the same WF
object. On Windows every historical scheduler is compiled with its matching
headers and an identical current host/completion overlay, including the SC
bridge recheck. Thus previous isolates the core change, not a difference in
Windows bridge source. All source snapshots, flags and binary hashes travel
with the artifact. Native results are recorded below; performance remains
unqualified.

The local M1 script run completed all oracle, normal-CLI, width and diagnostic
checks and returned the performance-failure status. Against previous, 22 of
24 median ratios were within the screen band; two participants at 65,536 /
tile 16 measured 1.0979 (range 1.0790–1.1183), and four participants at 4,096 /
tile 256 measured 1.0964 (0.9806–1.1570). These losses remain unresolved;
neither the protocol microbenchmark nor cross-target instruction inspection
overrides actual workload measurements. This is local evidence, not CI
acceptance.

Independent review of this delta covered the completion ordering, bridge
fallback, maintained-runtime protocol host, comparison provenance and recorded
measurements. It found no remaining blocking issue within that scope; the
diagnostic metadata now explicitly distinguishes before (reports disabled)
from previous/current controls (race-free reports enabled). This scoped review
does not certify the whole PR or the outstanding performance goal.

### Same-image variability control

The [five native screens at `c8384799`](https://github.com/mbbill/Whitefoot/actions/runs/34355813654)
completed their oracle, CLI, width and diagnostic checks; all five retain
performance failures. Its [12-job gate](https://github.com/mbbill/Whitefoot/actions/runs/34355813469)
and [Linux/Windows I/O checks](https://github.com/mbbill/Whitefoot/actions/runs/34355813476)
passed. The Windows EPYC 7763 screen has candidate/previous paired wall median
0.9250 (range 0.8780–0.9739) at four participants, 4,096 / tile 256, but its
separate long diagnostic instead records 51.185/37.521 us warm means. At two
participants, 4,096 / tile 1,024, the paired median is 1.2808
(0.7627–1.6418); its long diagnostic is also slower, 32.616/22.920 us.
These opposing observations do not qualify an overall speedup. Windows
artifact `10105858169` has ZIP SHA-256
`da4fe392b6c5c5e00033a3eb9273c9bd3a8e063cce3753ab7fec42360d8ab780`;
all extracted file hashes matched its manifest.

The Linux and Windows results retain large paired ranges,
including in one-participant cells. Reducing a locked instruction has not yet
established an end-to-end improvement. The next screen adds `replica`, a
byte-for-byte copy of the candidate executable invoked independently with the
same input, width and sample count. The script verifies binary identity; its
raw runtime label remains candidate. Candidate and replica are adjacent in
each pass and reverse order together. Their median wall ratio must lie within
`[1/1.05, 1.05]`; both faster and slower discrepancies flag investigation.
Existing before/recovered/previous/idle comparisons and thresholds remain.

This A/A control measures variability without a source or code-generation
difference. An adverse A/A result cannot excuse a candidate loss or pass a
platform: it says the measurement conditions need further work before a small
effect can be attributed. The same raw first/warm samples are retained. This
addition does not yet provide repeated CPU diagnostics or normal CLI timing.
Shell syntax and whitespace checks pass. The actual extracted summary program
accepts A/A medians 1.00, 0.98 and 1.03, rejects 1.08 and 0.94, and rejects a
missing replica. Independent review found no issue in the binary-copy order,
invocation labels, symmetric band or artifact coverage.

The [five-target A/A run at `5e3cbc24`](https://github.com/mbbill/Whitefoot/actions/runs/34357325383)
completed and failed every platform's performance screen. Replica comparisons
outside the symmetric band were Windows 2/24, Linux x86-64 11/24, Linux
AArch64 6/24, macOS x86-64 4/24 and macOS AArch64 3/16. Linux x86-64's
one-participant 4,096 / tile64 median was 1.3323 (range 0.7952–1.3897),
despite identical executable bytes. These short cohorts cannot reliably select
small runtime effects. They do not invalidate the recorded losses or grant
acceptance. That revision passed its local canonical `make check`, its
[12-job gate](https://github.com/mbbill/Whitefoot/actions/runs/34357325212), and
[Linux/Windows I/O checks](https://github.com/mbbill/Whitefoot/actions/runs/34357325252).

The next screen lengthens each independent process to 4,096 warm calls for
4,096 outputs and 512 calls for 65,536 outputs. Both the full batch and its
first64 prefix retain the same verdict criteria; either can fail. The prefix
is an overlapping view, not extra independent samples or an exact repeat of
the former 64-call process cohort. Raw first calls remain available. Five
processes remain the independent samples; the longer batch does not create
thousands of independent observations. CPU diagnostics remain separate and
normal CLI timing remains open.

### Coalescing condition-wait notifications

The maintained completion runtime and fallback host primitives now re-arm a
wake-needed flag when a waiter announces. Every publication still advances the
SC epoch. On the condition-variable-only path, the first notification for the
announced set takes the wait lock and signals; later notifications can skip
that lock until a new announcement. The flag/epoch SC pair closes the no-lock
missed-wake race. External callbacks retain the original per-publication
behavior: the flag stays set while an external waiter remains. This is one
shared runtime and its existing routes, not link-time runtime selection.

The candidate targets a measured cost: the Windows `c8384799` long
4-participant 65,536 / tile16 diagnostic counted 102,543 notification signals
for 1,551 announcements. That compute-only bridge uses condition variables;
signals are requests, not awakened threads or IOCP posts. Performance selection
requires lower notification/CPU cost without stable wall-time regressions;
results remain pending on all five targets. The previous control is frozen at
`5e3cbc24`, including its Windows bridge.

Review prevented extending coalescing to external I/O waits. A new IOCP park
can consume an older waiter's packet. Retaining and re-posting that packet
until all announcements withdraw would preserve the token but could keep the
queue permanently nonempty at the port's concurrency limit, preventing older
waiters from running. This follows the documented [IOCP concurrency behavior](https://learn.microsoft.com/en-us/windows/win32/fileio/i-o-completion-ports).
That proposed repair was removed before publication. Existing IOCP token
ownership/progress remains an unresolved correctness question; passing prior
I/O tests does not settle it. This round preserves the old external notifier
and token-consumption behavior instead of introducing an unqualified I/O change.

The harness checks coalescing, rearming, cancelled announcements, and unchanged
external notification counts. Existing real-thread wake tests now synchronize
past the final epoch recheck before requiring a wake result. The final narrowed
macOS harness passes at helper counts 0/1/4 and in no-cache mode; its bridge
ThreadSanitizer check also passes. Scheduler/deque tests passed before the
external-callback narrowing, which does not change those primitives. Final
native CI, canonical and all-platform performance acceptance remain required.

### Wake-coalescing measurements at 536abedd

The [five native screens](https://github.com/mbbill/Whitefoot/actions/runs/34360325388)
completed and all retain performance failures. Linux/Windows I/O checks passed;
the gate completed eleven jobs successfully but Linux unit was cancelled,
without a test failure reported in its log. Exact-revision local root
`make check` passed, including compiler, research, conformance and snapshot
checks. No platform is performance-qualified by these results.

For long batches at four participants and 4,096 outputs, candidate/previous
paired wall medians for tile16/64/256/1024 were respectively
0.8518/0.8355/0.9093/1.1362 on Linux x86-64 and
0.9106/0.9371/0.9174/0.8739 on Linux AArch64. The x86-64 coarse cell was slower
in all five pairs (range 1.1165–1.2346). At two participants, that same coarse
cell regressed on both Linux targets: medians 1.2407 and 1.2107, with minima
1.1450 and 1.1912. These losses prevent selecting coalescing as a portable win.

The Linux x86-64 diagnostic at two participants / 4,096 / tile1024 recorded
1,085 voluntary switches for the candidate versus 16 for previous; batch
user+system CPU was 313,576 versus 303,222 us. At four participants those
switch counts were 7,066 versus 3,332, CPU 693,577 versus 629,609 us. These
are separate diagnostic batches, not causal estimates or five extra samples.
Core stack-park counts differ from host waits and must not be substituted for
them. Artifact `10107962650` has ZIP SHA-256
`bc651e07c570e71fbdcd44f61af6ccbc983d514d143edf0d0bdf0153d2176e51`;
all 899 extracted manifest entries matched. The next attribution question is
why fewer notification opportunities coincide with more host waiting in these
coarse cells, while fine-grained cells improve.

A local M1 run of the same runtime source bytes (captured as a dirty tree over
5e3cbc24 before publication) also retained losses: four-participant small-cell
candidate/previous medians were 0.7852/0.7286/0.7381/0.8207, but corresponding
candidate/recovered medians remained 2.0369/1.4395/1.8415/1.6178. This is local
exploration, not another CI platform pass.

### Maintained-runtime quadrature coverage

The existing quadrature program/oracle now also executes on the maintained
scheduler, with the comparison protocol owned by the
[experiment](../../experiments/compute-runtime/README.md#adaptive-recursive-quadrature).
This adds recursive/skewed/depth-limited computations to the formal-runtime
comparison. Same-object attribution remains distinct from normal CLI delivery:
the ordinary CLI command is checked, but its timing remains unfinished.

Local full quadrature rebuilding succeeded. Its checks passed after granting
native CPU-topology access: 160 new formal/recovered processes plus all the
existing quadrature, sanitizer, exhaustion and batch-protocol checks. Review
caught and corrected nested source snapshots on repeated builds and invalid
caller-thread CPU subtraction across formal stack migration. The first
[native CI attempt at d5cd68b8](https://github.com/mbbill/Whitefoot/actions/runs/34363309499/job/102505558200)
stopped before calibration: the existing sanitizer validator rejected a
384-byte Rayon pool-build allocation report. That check remains enforced;
the run supplies no formal quadrature performance result. Windows, stronger
scaling and end-to-end delivery qualification remain open.

### Eliding obsolete wait announcements

The next maintained-runtime candidate checks the wake epoch before taking the
host wait lock. Fallback POSIX and Windows primitives also recheck under the
lock before announcing. Completion already had that locked check. A changed
epoch returns to the scheduler's work scan; it neither consumes a notification
nor clears the wake-needed flag. The existing post-announcement SC checks
remain the lost-wake protection. Ordinary calls, current-stack joins, public
ABI and I/O routing are unchanged. The frozen previous control advances to
`d5cd68b8` to isolate these early returns from the preceding coalescing change.

This targets avoidable mutex and announcement traffic, not the cost of a
necessary kernel sleep. An ephemeral local M1 million-call stale-epoch probe
measured 9.06–16.27 ns/call before and 2.17–5.79 ns/call after across five
sequential pairs. Order/frequency effects are visible; this is path-cost
evidence only, not an application speedup or cross-platform qualification.
Selection still requires the same application-level wall and CPU criteria,
including the coarse Linux regressions and unchanged I/O progress checks.

Scoped independent review found no blocking defect in the early-return
handshake. The modified tree passed the maintained completion harness with
0/1/4 helpers and no-cache mode, the default-route ThreadSanitizer probe,
scheduler smoke and Windows GNU cross-compilation. The published `f6e71c6a`
also passed exact-revision root `make check`. These correctness checks do not
qualify a platform's performance.

The local M1 screen completed all 720 process means and 72 diagnostic batches
but failed its performance criteria. At 4,096 outputs / tile1024, two/four
participant candidate/previous paired medians were 0.9309/0.9268, with every
pair below one. At 65,536 / tile16 they were 1.0580/1.3244; the four-participant
range was 0.9762–1.5283. Short-prefix views also retained failures. These are
the source bytes later published as f6e71c6a, captured while dirty over
d5cd68b8, not a clean-revision timing claim.

The [f6e71c6a Linux x86-64 screen](https://github.com/mbbill/Whitefoot/actions/runs/34364977223/job/102511242442)
did not reproduce that large/fine local regression: its four-participant
candidate/previous median was 0.9919 (0.9848–1.0121). However, small/coarse
candidate/recovered medians remained 1.4821 and 1.8206 at two/four participants.
The four-participant small/coarse A/A median was itself 1.0510. No platform is
accepted on the basis of this mixed evidence.

A local layout control has been built separately. The wait edit moved
entry/WF/native functions by 24 bytes in the original pair, while the core
functions stayed in place. Linking the changed primitive last with a common
Darwin order file holds every other text-symbol address fixed across that
pair; writable-data addresses also match, but a floor constant still moves.
Common host and computation objects are reused. This diagnoses a possible
layout confound; it neither changes the production linker nor erases the
original regressions.

### Equal-epoch notifications must rearm the next wait

The layout attribution did not finish: the run completed 67 process rows before
a f6e71c6a four-participant 4,096 / tile1024 process stalled. A native
sample showed all four threads in the scheduler loop's condition wait;
105 seconds of elapsed time had consumed only 0.33 seconds of CPU. Its partial
timings are not a completed layout experiment. The process was sampled and
then terminated; debugger attachment did not complete.

Independent review confirmed a legal missed-wake execution in notification
coalescing, predating the early-return optimization. A publisher can advance
the epoch before a new waiter captures it, but acquire the wait lock only
after that waiter sleeps. Its delayed broadcast clears wake-needed and wakes
the new waiter with an unchanged epoch. The old loop sleeps again without
rearming; the next publication can then skip its required signal.

Both POSIX/Windows fallback primitives and the shared completion condition
loop now rearm with SC ordering, then recheck the epoch with SC ordering,
before every repeated wait. Registration counts still describe park calls,
not each sleep attempt; timeout/error handling and external I/O routes remain.
A harness-only observer on return from the real condition wait lets the
regression deliver the delayed notifier's locked reset/broadcast tail, wait
for the actual return/re-sleep, then send a real notification. Resetting the
observer under the wait lock also excludes earlier spurious returns.

The final regression fails against the frozen f6e71c6a completion unit and
passes against the repair with otherwise matching test objects. The combined
working tree passes helper counts 0/1/4 and no-cache mode, and the completion
TSan harness passes helper counts 0/1/4. Both repaired Windows units compile
with the Windows GNU cross-toolchain; native MSVC execution remains required.
Separately, the
original frozen f6 compute/host objects with only the POSIX primitive repaired
completed 100 fresh processes of the stalled case, each verifying 4,097 calls.
All other text-symbol addresses match the stalled image. This supplies native
weak-primitive progress evidence as well as the deterministic completion test;
it is not performance qualification or conclusive attribution of every hang.

The [f6 quadrature job](https://github.com/mbbill/Whitefoot/actions/runs/34364977223/job/102511242571)
also stopped during formal calibration, at pass4 / four participants /
center-peak / wf-leaf, after its earlier correctness checks passed. Its last
summary row was written at 14:47 UTC; cancellation was at 14:59 UTC with a
formal process still present. Artifact `10110358723` ZIP SHA-256 is
`cb08a0b039e0ea9c3031d9d1f3d2ea45cc49fdb7889e5dcb3d8676be38148961`.
Linux AArch64's formal screen was also cancelled while comparing runtimes.
Those are incomplete measurements, not performance passes.

The timing reference returns to pre-coalescing `5e3cbc24`. Keeping the known
stalling f6/d5 implementations in every timing loop would prevent completing
the matrix and would not establish a qualified reference. Their evidence is
retained, and the old f6 completion unit now fails the new regression; no
acceptance threshold or workload was relaxed. Exact f2d9d0fa subsequently passes
local canonical `make check`: compiler 581 seconds, research 210, conformance
99 and snapshot 21, ending `WHITEFOOT ALL TESTS GREEN`. Linux and Windows native
I/O host checks pass. The Linux research CI gate still fails on the previously
seen Rayon caller-worker 384-byte LSan report; its validator expects the
associated indirect queue as well. The direct allocation matches the known
`use_current_thread` lifecycle, but why that queue is absent from this report
has not been established. No sanitizer criterion is relaxed.

The [f2d9d0fa compute run](https://github.com/mbbill/Whitefoot/actions/runs/34369584492)
completes all five native FIR screens, with performance
failures rather than stalled timing loops. Long-batch paired medians below
are candidate/reference; `previous` is pre-coalescing 5e3cbc24. Each ratio
uses five fresh-process pairs on its own host, not cross-host timings.

| Host | Workers | N / tile | Previous | Recovered | Identical replica |
|---|---:|---|---:|---:|---:|
| Linux x64 | 4 | 4096 / 1024 | 0.9039 | 1.4356 | 0.9925 |
| Linux x64 | 4 | 65536 / 16 | 0.9763 | 1.1428 | 0.9846 |
| Linux ARM64 | 4 | 4096 / 1024 | 0.8251 | 1.6742 | 1.0095 |
| Linux ARM64 | 4 | 65536 / 16 | 1.0023 | 1.2143 | 1.0212 |
| macOS ARM64 | 2 | 4096 / 1024 | 0.9274 | 1.6086 | 0.9944 |

All five coarse-cell pairs lose to recovered on each listed host. Long-batch
A/A has two cells outside its band on Linux x64, zero on Linux ARM64, and
five of sixteen on macOS ARM64; the latter does not qualify small deltas.
The same run's quadrature panel completes all 2,400 processes with 256 repeats
and its data verifier passes. Performance does not: center-peak / wf-leaf /
four workers has formal/recovered wall 1.2726 and CPU 1.4620. Successful data
collection after the wait repair is not performance acceptance.

An independent delivery review also confirms that these attribution images
do not close ordinary CLI timing. FIR and quadrature commands are tiny smoke
cases; records' command is manually linked and Mandelbrot lacks that normal
CLI panel. Substantial input generation and repetition live in C drivers.
Normal linking uses O2; attribution uses O3 with auto-vectorization disabled.
A general scalar control must also suppress the maintained lowering's explicit
wide byte probes; Clang vectorizer flags alone cannot remove those vectors.
These remain delivery gaps, not grounds for applying research timings to the
ordinary executable.

Windows also completes with a failed performance screen. Its four-worker
4,096 / tile1024 candidate/previous median is 0.8898, but candidate/identical
replica is 0.7542 (range 0.6036-1.3159), so the apparent improvement is not
qualified. Four of 24 long-batch A/A cells are outside the band. The research
recovered control still has no qualified Windows port; these ratios are
against the maintained-runtime control, not evidence of parity with recovered.
macOS x64 is also noisy: nine of 24 long-batch A/A cells exceed the band.
At four workers, 4,096 / tile1024 has candidate/previous 1.0174,
candidate/recovered 1.3533 and candidate/replica 1.3368; this does not qualify
a precise implementation delta. No native platform meets full acceptance.

### Completing an owned join target locally

The M1 large/fine diagnostic executed approximately 2.09 million tasks in
the owner's inline-join branch versus about 12,000 steals. Every inline task
still paid the generic completion handshake. The next maintained-core
candidate removes that handshake only after the owner successfully pops its
own join target. It calls the body and release-publishes DONE directly.
Generic helper/steal execution and I/O completion keep the full handshake.
The local comparison uses f2d9d0fa as its reference and the same repaired
wait primitive in both images; f6's defective wait is not a timing control.

The premise is the existing unique live joining continuation: the compiler
keeps the handle private and emits join, result read, then release. That
continuation is the direct caller, so it cannot simultaneously be parked on
this record. Nested calls or I/O may migrate the whole stack; their waits
name their own records. This does not admit concurrent joins on a shared
future, change public ABI, change callback execution or select a runtime.

The enumerator permits the direct PENDING-to-DONE edge only for the active
same-stack compute join, after its callback-return witness, with no waiter.
Both witnesses are checkpointed with the logical stack. It still requires
COMPLETING for I/O and generic completions. All four full sweeps pass;
the two-thread/four-stack S5 sweep observes 119 owner-DONE transitions after
migration, not 119 independent tasks. A negative case rejects DONE before
callback return. Native smoke holds thieves, defers its device completion
until the inner stack is SUSPENDED, then verifies one inline execution, one
park/resume, the result and a repeated join. It passes, as do the six Rust
scheduler tests, bridge ThreadSanitizer and Windows GNU core cross-compilation.
These focused checks preceded the waiter repair; the combined-tree completion
checks are recorded above. Exact 98c283cb subsequently passes local canonical
`make check` and all twelve gate CI jobs; its native performance results below
do not qualify the candidate. Selection requires application and CPU improvement without
losing these ownership and progress properties, not merely fewer instructions.

Two local FIR cohorts each complete 90 fresh processes: one/two/four workers,
4,096 / tile1024 and 65,536 / tile16, five alternating passes, fixed/inline/
identical-replica images. Every output is checked. They run after the exact
f2 canonical process exits, with common computation/host/primitive objects.
In the ordinary link layout, long compute ratios are near parity (two-worker
fine 0.9777; four-worker fine 0.9971), but whole-batch CPU rises 9-15%, even in
the sequential world that cannot execute the changed join branch. The one-worker
large case's core remains about 535-538 us while its full cycle grows from
about 1,598 to 1,818 us. The CPU observation includes verification; it cannot
be attributed solely to the scheduler.

The join edit shrinks text by 28 bytes and shifts later functions. A separate
Darwin order-file control places join last and holds every other text-symbol
address fixed. Under that layout, the extra batch CPU cost disappears. For
two-worker fine work, all five core and CPU pairs improve: medians 0.9662 and
0.9809; identical-image core ratio is 1.0113. Four-worker fine medians are
0.9937 core and 0.9974 CPU. Short coarse work still loses: two-worker first64
core ratio 1.0843, range 1.0570-1.1886. These are overlapping short views of
the same processes, not independent cohorts. The local layout experiment
diagnoses the confound; it neither changes the production linker nor qualifies
the candidate's overall application performance.

Two quadrature cohorts then complete 240 processes each, with 4,096 checked
repeats plus eight warmups: fixed/inline/replica, four inputs, one/four workers,
leaf and sequential-kernel forms, five alternating passes. Both use common
f2 gate computation/host/primitive objects; the second holds all non-join
text-symbol addresses fixed. Its candidate/fixed ratios are:

| Input | One-worker leaf wall / CPU | Four-worker leaf wall / CPU |
|---|---|---|
| Center peak | 0.9410 / 0.9412 | 0.9577 / 0.9489 |
| Left peak | 0.9421 / 0.9421 | 0.9694 / 0.9704 |
| Right peak | 0.9335 / 0.9335 | 0.9885 / 0.9941 |
| Depth cap | 0.9938 / 0.9934 | 0.9341 / 0.9279 |

All five one-worker peaked-input pairs improve in both layouts; their fixed
layout A/A wall medians are 0.9992-1.0033. This one-worker leaf form explicitly
executes the outlined task path on the core without helper threads; it is not
the ordinary CLI's automatic sequential-world selection. Separate sequential
kernel controls remain near parity. Four-worker medians also improve, but
individual wall/CPU pairs lose and A/A ranges are wide; no per-cell pass is
claimed from those medians. In the first layout, four-worker center-peak wall
is 0.9283 and depth-cap is 0.9462, showing why layout-conditioned results must
remain separate rather than pooled.

The repeated local-task benefit justifies native CI evaluation of this small
maintained-core shortcut; it does not resolve the FIR short-batch losses or
the much larger recovered-runtime gaps. The next five-target screen pins
`previous` to repaired f2d9d0fa to isolate the shortcut. Workloads, thresholds
and the recovered/identical-image controls remain unchanged. Full application
qualification and ordinary CLI timing remain open.

The combined-tree `whitefootc` binary is rebuilt through Cargo's gate profile.
Its normal `--par ... -o ...` FIR and quadrature executables pass at one/four
workers. This confirms current-source CLI integration and correctness, not
ordinary CLI performance qualification.

### Native result for 98c283cb: not qualified

The [five-target run](https://github.com/mbbill/Whitefoot/actions/runs/34373190141)
completes every native FIR screen and the 2,400-process quadrature panel.
All five FIR screens fail performance acceptance. Ordinary CLI FIR correctness,
native deque checks, the [gate](https://github.com/mbbill/Whitefoot/actions/runs/34373190119),
[I/O host checks](https://github.com/mbbill/Whitefoot/actions/runs/34373190112)
and [I/O benchmark checks](https://github.com/mbbill/Whitefoot/actions/runs/34373190123)
pass. The prior intermittent Rayon sanitizer report does not recur in this gate;
that is not a demonstrated lifecycle fix.

Selected long-batch candidate / f2d9d0fa paired wall medians follow. Ratios below
one favor the candidate. Each cell retains five independent processes per image;
the two workloads are coarse 4,096 / tile1024 and fine 65,536 / tile16.

| Native host | Coarse, 2 workers | Coarse, 4 workers | Fine, 2 workers | Fine, 4 workers | Long A/A cells outside band |
|---|---:|---:|---:|---:|---:|
| Linux x64 | 1.1646 | 1.2683 | 0.9562 | 0.9848 | 3 / 24 |
| Linux ARM64 | 0.9997 | 0.9968 | 0.9783 | 0.9770 | 0 / 24 |
| macOS ARM64 | 0.9292 | Not run: two CPUs | 1.0867 | Not run: two CPUs | 7 / 16 |
| macOS x64 | 0.9360 | 0.9909 | 0.9870 | 0.9501 | 4 / 24 |
| Windows x64 MSVC | 0.9646 | 1.1174 | 0.9632 | 1.0206 | 0 / 24 |

Linux x64 coarse work regresses in every pair: two-worker range
1.1516-1.2769 and four-worker 1.1624-1.2994, with corresponding A/A medians
0.9867 and 1.0021. Windows four-worker coarse work also needs investigation
(range 0.8992-1.1197). Mac gains or losses cannot be selected through the
substantial identical-image variation. Linux ARM64 avoids the new coarse
regression but remains 1.6038 times the recovered runtime at four workers;
Linux x64 is 1.8132 times that control. Matching the preceding maintained
revision is not matching the recovered baseline.

Quadrature still loses against the recovered runtime. Center-peak leaf/four
wall and CPU medians are 1.3692 and 1.5054; depth-cap leaf/four is 1.3993 and
1.4460. These ratios are from this host's own controls, not a cross-run comparison
of absolute times against f2's different CI host.

The Linux x64 [artifact](https://github.com/mbbill/Whitefoot/actions/runs/34373190141/artifacts/10113055288)
has ZIP SHA256 `3450c6d70a9462f99a9fd1306bed5117f305fdd10556ce3dec3b8fbf53038848`.
Its disassembly keeps the join prefix and stack-frame size unchanged, removes
the owner's generic completion tail call and shrinks join by 17 bytes. Later
text, including both FIR computation functions, moves by 16 bytes. The separate
four-worker coarse diagnostic reports candidate/previous inline counts
3,976/2,201, steals 8,315/10,090 and parks 15/28; both execute 12,291 tasks.
These instrumented, single-process counts are not paired timing evidence and
do not establish why the ordinary five-process cohort regresses. Fewer parks
alone does not explain or excuse the loss.

The next Linux-only diagnostic places join after the other executable sections
through a declaration and linker script, compiling the unchanged maintained
sources. Before timing, it requires identical addresses for every other text
symbol and a byte-identical candidate replica. Its separate 90-process cohort
covers one/two/four workers, the coarse/fine cells above and five alternating
passes. Symbol sizes, ELF maps and disassembly preserve possible instruction or
data-placement differences: matching function starts alone does not eliminate
all binary-layout effects. Ordinary placement remains measured with unchanged
acceptance criteria. This control tests the following-function address
explanation; it does not change the production
linker or repair the regression. Remove it once that causal question is resolved.

### Scalar builds through the ordinary compiler

`whitefootc --no-vectorize` now supplies the missing general scalar build option
in the maintained compiler. It suppresses explicit WF byte probes after semantic
checking; normal native linking and stack-ledger generation also pass Clang's
loop/SLP disabling flags. LLVM-only consumers must pass those host flags when
they subsequently compile the IR. Default optimization remains `-O2`, and the
default vectorization setting remains enabled. This is not a promise that
platform library internals contain no SIMD.

The option passes all-target Cargo checking, Clippy with warnings denied, and
four focused tests: scalar lowering retains the ordinary loop, invalid proofs
retain their diagnostics, CLI mode/ledger selection is independent, and the
actual native CLI executes a boundary-sensitive byte walk with both ledgers
enabled. Scoped independent review found one misleading LLVM-only documentation
sentence, now corrected; no remaining compiler-option finding. New exact-tree
canonical and cross-platform execution remain pending. FIR's five native CI
commands now request this option; their host-driven attribution objects stay
at the separately recorded `-O3` setting. Full ordinary CLI timing and broader
native workload coverage remain open.

### Follow-up native screen at 47efc919

Exact 47efc919 passes canonical `make check` with local loopback networking
permitted and all twelve [gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34375782088).
The first local invocation stopped at seven network tests because the sandbox
denied listener creation; those tests were retained and pass in the complete
rerun. The [compute run](https://github.com/mbbill/Whitefoot/actions/runs/34375782076)
completes all five formal screens, including normal scalar CLI FIR correctness.
Every formal screen still fails performance acceptance.

The new Linux x64 host is EPYC 9V74, whereas 98c283cb ran on EPYC 7763.
On 9V74, ordinary-layout coarse candidate/f2 medians are 0.9958 at two workers
and 0.9959 at four, before applying any placement control. The corresponding
recovered-runtime ratios remain 1.2069 and 1.2805. Thus this run does not
reproduce the old host's 16-27% regression and cannot establish that placement
resolved it. On Linux ARM64, ordinary four-worker coarse is 0.9932 versus f2
and 1.6048 versus recovered; the larger integration gap remains.

The separate ELF cohort completes on both Linux architectures and verifies
equal starts for every non-join text symbol. On x64 its two/four-worker coarse
wall medians are 0.9812/0.9979; A/A is 1.0023/0.9904. Whole-batch CPU medians
are 0.9775/1.0241, with mixed individual pairs. Four-worker coarse RSS has an
unresolved 1.2831 median ratio, despite equal text starts; this observation is
not normalized away. The first64 view also retains a four-worker fine loss
(1.0525). On ARM64, fixed-start four-worker coarse is 1.0154 and fine is 0.9801.
Neither layout cohort qualifies overall performance.

The x64 [artifact](https://github.com/mbbill/Whitefoot/actions/runs/34375782076/artifacts/10114114172)
has ZIP SHA256 `525d464e71eb1963374e71a9665c9b7b13e615b8fa70d4dfe53ccc711f5ea0db`.
Its maps retain equal data-section starts, while `.wf_join` and unwind metadata
sizes differ. Function-address equality is the checked invariant, not wholesale
instruction/data equality. The original-host loss stays unresolved; avoid a
production linker change selected only from the new host's near-parity cohort.

### Counter-isolation experiment

After 8b61e7c4, a maintained-core candidate aligned each physical thread's
statistics to 128 bytes. Counters remain atomic and enabled; this does not
change their live-observation contract. The thread record grows from 136 to
256 bytes, adding 7,680 bytes across its 64-element array and shifting later
core storage. The experiment therefore includes layout effects beyond counter
sharing. It is not a selected optimization or a qualified delivery.

Two local M1 cohorts use the same scalar WF object and byte-identical binaries
across repetitions: one/four workers, FIR sizes 4,096/65,536, tiles 16/1,024,
five alternating base/candidate/identical-replica process sets per cell,
4,096 warm calls for the small input and 512 for the large. Four-worker small
fine-work candidate/base core-wall medians are 0.9510 and 0.9610; large coarse
medians are 0.9904 and 0.9931. Large fine work is unstable: 1.0582 and 0.8826,
with first-cohort replica/candidate 1.1960. Neither cohort establishes general
improvement. First-cohort whole-batch CPU ratios include verification and must
not be attributed to scheduler cost alone; corresponding RSS medians range
0.9783–1.0097. Startup and memory effects remain part of qualification.

Native smoke, the 200,000-task deque/live-counter probe, and a rebuilt ordinary
`whitefootc --par --no-vectorize` FIR command at one/four workers pass locally.
Scoped independent review finds no storage/protocol blocker: typed static core
storage propagates alignment, and initialization/enumerator offsets follow
`sizeof`/`offsetof`. Five-platform native performance, broader workload coverage
and a stable causal comparison remain required before retaining the change.
Raw local cohorts are `count-isolation-run1` and `count-isolation-run2` under
`/private/tmp/whitefoot-formal-runtime`; they are not published CI artifacts.

The five-platform screen adds frozen 8b61e7c4 as `unaligned` while retaining
every existing ordinary reference and threshold. Selection requires stable
candidate/unaligned wall and CPU behavior beyond identical-image variation,
with startup/RSS losses explained, plus the wider goal's workload and native
reference coverage. The Linux join-placement cohort now freezes its subject
at 8b61e7c4 versus f2: changing thread layout invalidates its non-join-address
premise for the current candidate. Its original-host question remains open;
its separate verdict cannot qualify the new candidate.

### Counter-isolation CI at 59dd9c18

All five [native formal screens](https://github.com/mbbill/Whitefoot/actions/runs/34380683524)
complete and fail performance acceptance. Selected long-batch candidate/8b61e7c4
median ratios follow; values above one are slower. These are matched same-host
comparisons, not ratios across machines or runs.

| Platform | Small coarse, 2 workers | Small coarse, 4 workers | Large fine, 2 workers | Large fine, 4 workers |
| --- | ---: | ---: | ---: | ---: |
| Linux x64 | 0.9802 | 1.0432 | 0.8811 | 0.9444 |
| Linux ARM64 | 1.2858 | 1.0863 | 1.0069 | 0.9485 |
| Windows x64 MSVC | 1.2691 | 1.0126 | 0.9550 | 0.9627 |
| macOS ARM64 | 1.0717 | unavailable | 0.8980 | unavailable |
| macOS x64 | 0.9385 | 1.3034 | 0.9446 | 1.0113 |

Small/coarse is 4,096 outputs / tile 1,024; large/fine is 65,536 / tile 16.
The ARM64 Mac screen exercises one/two workers and has no four-worker cell.
The Intel Mac four-worker
coarse ratio has range 0.8116–1.9184 and candidate/replica 1.0730, so its median
cannot establish a stable regression. Windows two-worker coarse ranges
0.7137–1.3786. By contrast, Linux ARM64's two/four-worker coarse losses occur
in every pair and remain outside its near-unity identical-image medians.

The Linux ARM64 [artifact](https://github.com/mbbill/Whitefoot/actions/runs/34380683524/artifacts/10116059462)
is SHA256 `62082f14bfa9537f192fb822cd5799acf21df2052285c767cb94c6c8a865cd46`;
its host is four-core Neoverse-N2 without SMT. Two/four-worker coarse
whole-batch CPU ratios are 1.1361/1.0853, with all five pairs above one.
Two-worker candidate processes have 3,532–3,867 voluntary context switches,
versus 41–405 for the control. Separate diagnostic processes record 130 versus
14 parks and 4,079 versus 4,098 steals. These observations implicate additional
waiting/waking costs but do not isolate why a layout change caused them;
diagnostic counters are not paired timing samples. The layout is rejected as
a general default rather than compensating for these losses with fine-task wins.

All twelve [gate CI jobs](https://github.com/mbbill/Whitefoot/actions/runs/34380683471)
and both [I/O host jobs](https://github.com/mbbill/Whitefoot/actions/runs/34380683469)
pass. The Windows I/O benchmark stops because `io-warm` remains unstable after
two cohorts; its mixed observer records 1,024 grants. This is not an I/O
performance pass. Local canonical checking stops at missing pinned scheduler
source paths after compiler checks pass. The unchanged revision subsequently
passes canonical `make check` with verified TBB/Parlay pins and explicit source
paths, including the complete native conformance adapter and snapshot corpus.
This correctness pass does not change its failed performance verdict.

### Open owner-local slot experiment

The next maintained-core experiment is developed from 59dd9c18 but removes
its counter alignment before performance comparison. The Linux ARM64 CI at
59dd9c18 finds stable coarse small-input candidate/unaligned regressions:
two-worker median 1.2858 (range 1.2579–1.3560, A/A 0.9947), four-worker 1.0863
(1.0610–1.1063, A/A 1.0111). All five pairs lose in each cell. That prevents
selecting counter isolation as a general optimization. Task-slot changes are
compared against 8b61e7c4 for an unchanged counter layout. A comparison with
59dd9c18 would include the reversal and cannot isolate the slot change.

In the baseline, every successful acquire/release pair changes one shared
free-list head with two CAS
operations, including same-thread returns. The frozen recovered implementation
uses a local list; restoring plain accesses to the shared head would lose
foreign returns after a joining continuation migrates. Instead, separate the
owner's local free list from the existing atomic foreign-return list. Both use
the same fixed slot capacity. Acquisition prefers local slots and falls back
to the original atomic pop; release rechecks current physical-thread identity.
No stack policy, task-frame ABI, source acceptance or I/O progress rule changes.

The provisional test is whether avoiding the local CAS pair improves matched
scalar workloads without losses in foreign-return-heavy or exhausted cases.
Compare with frozen 8b61e7c4, retaining identical-image variation,
whole-process CPU/RSS and ordinary CLI delivery. Native smoke, bounded
enumeration, concurrent full-capacity reuse and live counter observation must
hold before performance evidence can select the change. The enumerator models
both heads, enforces local-list ownership and retains both free chains in its
state digest; no interleaving or failure check is removed. This is an open
experiment, not a selected optimization or cross-platform qualification.

The first local slot-only cohort completes 180 processes at one/two/four
workers, the same two input sizes/tiles and warm counts as the smaller local
counter experiment. Two-worker large/fine core-wall ratio is 0.9437
(0.9323–0.9620, replica/candidate 0.9962), but four-worker large/fine is 1.3036
(0.7512–1.4040, replica/candidate 1.0071). Four-worker small/fine is 1.0266;
small/coarse is 0.9784. This is not a uniform improvement. The four-worker
large/fine whole-batch CPU median is 1.0629; two-worker large/coarse RSS is
1.1200. Verification is outside core-wall timing but inside batch CPU, so
apparent batch CPU savings elsewhere cannot be attributed to scheduler work.
These raw samples are retained locally as `owner-slots-perf`; they are not
published CI artifacts or a portable reproduction reference.

The final combined candidate passes native smoke, 200,000-task reuse with
full-capacity refusal, the native deque probe under ThreadSanitizer, all four bounded enumeration
configurations, and a rebuilt ordinary scalar CLI FIR command at one/four
workers. Independent review finds no remaining correctness blocker after
adding a per-step two-list membership/cycle check before state pruning.
Cross-platform performance and full candidate canonical validation remain open.

The [native screen at bf7c56df](https://github.com/mbbill/Whitefoot/actions/runs/34383558863)
retains mixed results. These are long-batch median candidate/8b61e7c4 ratios;
coarse means 4,096 outputs / tile1024, fine means 65,536 outputs / tile16.
Values below one favor the candidate. The table is a representative subset,
not a replacement for the complete per-cell artifacts and short-view failures.

| Target | Coarse, 2 workers | Coarse, 4 workers | Fine, 2 workers | Fine, 4 workers |
| --- | ---: | ---: | ---: | ---: |
| Linux x64 | 0.8369 | 0.8082 | 0.9952 | 0.9980 |
| Linux ARM64 | 0.9910 | 0.9701 | 0.9664 | 0.9692 |
| Windows x64 | 0.9801 | 0.9077 | 0.9893 | 0.9986 |
| macOS ARM64 | 0.9268 | no four-worker cell | 1.0206 | no four-worker cell |
| macOS x64 | 1.1376 | 0.9626 | 0.9939 | 0.9833 |

All five platforms finish with performance failures. On Linux x64,
four-worker coarse improves against 8b61 in all five pairs (0.7769–0.8153;
candidate/replica median 0.9877), but still costs 1.4472 times the recovered
runtime. Linux ARM64's corresponding recovered ratio is 1.5219. macOS ARM64's
two-worker coarse recovered ratio is 1.2125, and some identical-image medians
are outside the band: single-worker 4,096 / tile64 is 1.1484. Local wins do
not resolve these platform or measurement failures. macOS x64's two-worker
coarse candidate/8b61 median is 1.1376 (1.0182–1.3961; candidate/replica
1.0163): all five pairs regress. Its four-worker fine identical-image ratio
is 1.1282, another unresolved noisy cell. The task-slot candidate is therefore
not selected as a performance-qualified default. The follow-up CI setup repair
retains its runtime bytes so a fresh cohort can check these results.

Windows four-worker coarse remains 3.0540 times the historical formal-before
control (2.9689–3.2050), with candidate/replica median 0.9953. Across the same
five timed processes, candidate wait announcements are 9,507–12,124 versus
436–469 before; signals are 7,208–9,340 versus 379–399. Whole-batch CPU
candidate/before has median 1.1346, range 0.9375–1.2083; this includes checking
and has quantized Windows accounting. Announcements are not actual sleeps,
and signals are not resumed-thread counts.

The same-source idle4096 control still exposes the waiting tradeoff. Windows
four-worker coarse candidate/idle4096 wall is 3.7550 (3.2696–4.2841), and
all five idle4096 processes have zero host wait announcements. But for 65,536
outputs at tiles16 and1024, idle4096/candidate batch CPU medians are 1.3065
(1.1212–1.4196) and 1.5094 (1.1833–1.8800), respectively. Their wall changes
are small: candidate/idle4096 is 0.9911 and 0.9603. Keeping the default at 256
avoids selecting a broad CPU regression to fix one burst-latency cell. These
repeat measurements strengthen the existing idle/wake hypothesis; they do not
identify a qualified adaptive policy. Windows artifact `10117283061` has
verified ZIP SHA-256
`92f7813c9d0af5e837db7cc21124efc0f81b14ed580eb2fa5428cf5a8e0d85dd`.

This revision's five extended Linux workload jobs fail during toolchain setup:
the runner's unrelated Chrome APT repository returns `Hash Sum mismatch`.
All six Linux gate jobs and the Linux I/O jobs are also affected; these jobs
supply no passing correctness or performance evidence. All six macOS gate
jobs and the Windows I/O host and benchmark jobs pass.
The follow-up workflow repair selects the runner's existing Ubuntu24.04
`ubuntu.sources` for required packages, retaining its signing configuration
and all tests. The follow-up execution below validates this setup repair;
the earlier failed jobs are not passing evidence.
The API integration also rejects job reruns and PR metadata writes with 403;
branch pushes remain available. Exact bf7c56df subsequently passes canonical
`make check` with the verified native dependency pins: compiler 610 seconds,
research 212, conformance 98 and snapshot 21. This does not qualify later trees.

The [dc383eef native cohort](https://github.com/mbbill/Whitefoot/actions/runs/34385490206)
has the same runtime bytes as bf7c56df and runs past setup on all platforms.
All five formal-runtime screens still fail performance acceptance. Long-view
candidate/8b61 medians (below one favors the candidate) are:

| Native host | 4096/tile1024 W2 | W4 | 65536/tile16 W2 | W4 |
|---|---:|---:|---:|---:|
| Linux x86_64 | 0.8528 | 0.8113 | 1.0084 | 1.0097 |
| Linux AArch64 | 0.9950 | 0.9848 | 0.9486 | 0.9679 |
| macOS AArch64 | 1.0743 | not run | 0.8552 | not run |
| macOS x86_64 | 1.0285 | 1.1190 | 1.0184 | 1.1034 |
| Windows x86_64 | 1.3366 | 0.9879 | 0.9954 | 1.0166 |

Linux x86_64 W4 coarse repeats its improvement over 8b61, with ratios
0.7738–0.8274 and A/A median 0.9833, but remains 1.4171 times recovered.
Linux AArch64 W4 coarse is 1.5991 times recovered. macOS AArch64 W2 coarse
is 1.6105 times recovered, and macOS x86_64 W4 coarse is 1.5656. Windows
W4 coarse remains 2.4568 times historical formal-before (2.1527–2.9292),
with noisy candidate/replica median 0.8600. Its W2 coarse regression against
8b61 has range 1.0012–1.4551 and A/A median 1.0406. Retain both cohorts;
these measurements do not establish portable acceptance of the slot change.

Records, scheduler, FIR and Mandelbrot Linux jobs pass; quadrature fails its
performance screen. The [gate](https://github.com/mbbill/Whitefoot/actions/runs/34385490336)
has eleven passing jobs and one Linux research failure: the already-known
Rayon caller-worker report contains only 384 bytes, which the old classifier
rejects. The experiment README records the corrected report-membership
invariant and its limits; this failing run is not retroactively marked green.
Both [I/O host jobs](https://github.com/mbbill/Whitefoot/actions/runs/34385490308)
and all four [I/O benchmark/read jobs](https://github.com/mbbill/Whitefoot/actions/runs/34385490307)
pass. These are evidence of preserved I/O function, not compute acceptance.

### Delayed idle registration: rejected as the general policy

The 461a7130 candidate compares with dc383eef, keeping task slots, counters,
spin/yield limits and the stack policy fixed. The original idle turn publishes
its idle bit before its bounded polling window. Every task publication that
sees any such bit calls the shared wake primitive, which advances an atomic
epoch even when notification coalescing avoids the host lock. The recovered
runtime polls before publishing its idle bit. This difference is a candidate
source of contention, not an established explanation of the measured losses.

Test polling without a registration, then retain the complete pre-sleep
sequence: publish the idle bit and optional in-place waiter, capture the
epoch, flush/progress, check ready work, status and deques, and park only on
that captured epoch. Keep a progress pass before polling as well, so pending
I/O is not delayed by a new polling window. No unregistered path may sleep,
and a path finding work must clear only a registration it actually made.
The final registered window retains the existing lost-wake argument; the
earlier scans cannot replace any of its checks.

Selection requires all existing native and bounded-interleaving checks,
mixed I/O progress, lower shared wake-epoch traffic in separate diagnostic
batches, and no unresolved wall/CPU/RSS regression against the same-host
dc383eef baseline. The full recovered-runtime and native-reference goals
remain. Unchanged limits isolate announcement timing from simply spinning
longer. If notification traffic falls without improving application costs,
this hypothesis is insufficient. Native instruction layout can still differ.

Local validation of the delayed-registration tree passes native smoke, all
four bounded schedule enumerations, core-read/default-route checks and the
default-route TSan check. The largest enumeration visits 98,769,875 states;
no state ceiling or stranded-work assertion changes. A rebuilt compiler's
ordinary `--par --no-vectorize` FIR command passes at one and four workers;
that small impulse program is correctness evidence only.

The local M1-series Mac cohort uses identical scalar computation objects,
dc383eef runtime sources, the modified maintained runtime, and a byte-identical
candidate replica. Five alternating passes cover all 12 cells (180 processes),
after other local tests finish, plus 24 separate diagnostic processes. The
`late-idle-perf-run2` raw rows, source copies, candidate patch, flags and
reproduction script are retained locally only, not as a published CI artifact. All
candidate/base wall medians lie between 0.9850 and 1.0242: no meaningful
overall improvement. W4 small/fine is 1.0242 (0.9271–1.0968), small/coarse
1.0119 (0.9913–1.0527), large/fine 0.9952 (0.9692–1.3198), large/coarse
0.9998 (0.9876–1.0027). Batch CPU includes verification: W1 large/fine
regresses to 1.0318 (1.0263–1.0340; CPU A/A 1.0000), and W2 large/fine
to 1.0249 (1.0101–1.0283; CPU A/A 0.9963). All RSS medians are at most
1.0223, without an allocation-policy change.

In separate W4 large/fine diagnostics, wake-epoch advances fall from 172,999
to 121,700, while parks stay at three and wall time barely changes. Small/fine
advances instead rise from 738,363 to 740,403. These count notification-epoch
advances, not task publications or kernel wakeups, and are not sampled inside
timed calls. The local result
does not select this policy; the CPU regressions remain open. The CI screen
adds frozen dc383eef as `slotbase` while preserving all previous controls and
acceptance limits, to test the different native hosts. Initial local setup and
compile failures occurred before timing and are retained separately; no timed
sample is discarded. Subsequent exact-revision checks and native results
follow below.

Independent review of this round over dc383eef covers the maintained idle
protocol, frozen comparison sources, diagnostic boundaries, sanitizer grammar
and its gate wiring, and the reported measurements. The reviewer checks shell
syntax/whitespace, reconciles all five CI tables and recomputes the local
wall/CPU/RSS results; supplied native/model/I/O logs pass. Both prose findings
are corrected, with no remaining finding in that scope. Current canonical
checking, cross-platform performance and normal-CLI timing are unverified;
the local CPU regressions are unresolved. This is not whole-PR approval or
completion of the delivery goal.

Exact 461a7130 subsequently passes local canonical `make check`: compiler
903 seconds, research 220, conformance 99 and snapshot 20. The
[native CI cohort](https://github.com/mbbill/Whitefoot/actions/runs/34388362153)
completes all five formal-runtime screens, all failing performance acceptance.
Long-view candidate/dc383eef paired medians are below; all five pairs and
replica controls remain in the artifacts, including noisy and losing cells.

| Native host | 4096/t64 W2 | W4 | 4096/t1024 W2 | W4 | 65536/t16 W2 | W4 |
|---|---:|---:|---:|---:|---:|---:|
| Linux x86_64 | 1.0350 | 1.0743 | 0.9736 | 1.0254 | 1.0086 | 1.0021 |
| Linux AArch64 | 1.0063 | 0.9706 | 0.8354 | 0.9840 | 1.0001 | 0.9965 |
| macOS AArch64 | 1.0670 | not run | 0.9201 | not run | 0.9855 | not run |
| macOS x86_64 | 1.1328 | 1.0867 | 0.9860 | 1.0260 | 0.9794 | 1.0137 |
| Windows x86_64 | 0.9859 | 1.1995 | 0.8083 | 1.0788 | 1.0020 | 1.0063 |

Windows W4 4096/t64 loses in every pair, 1.1078–1.3487, with A/A median
0.9602. Linux x86_64 at that cell also loses in every pair, 1.0542–1.0897,
with A/A 0.9989. Conversely Linux AArch64 W2 4096/t1024 gains in every
pair, 0.8006–0.8457, with A/A 1.0010; retain this genuine local benefit.
Windows W2 coarse has 0.7374–0.8356 but noisy A/A 0.9519
(0.8119–1.2152). macOS x86_64 has widespread A/A variability, including
W4 coarse 0.9063 (0.5447–1.2883). No platform-wide speedup follows from
these medians. Windows W4 coarse is still 3.3351 times historical formal-before;
Linux x86_64 W4 coarse is 1.2867 times recovered, and Linux AArch64 is
1.6192 times recovered. The original broad performance goal remains open.

The [partitioned gate](https://github.com/mbbill/Whitefoot/actions/runs/34388362074)
has nine passes and three cancellations at its configured eight-minute limit:
Linux/macOS unit and Linux static. Two Linux jobs still have an enumeration
process at cancellation; macOS unit completes its largest enumeration just
before cancellation. The new state space nearly doubles the earlier largest
enumeration. Keep the limit and full test coverage; the cancelled checks are
not passes. Both research jobs pass the corrected sanitizer classification.
Records, scheduler, FIR and Mandelbrot extended jobs pass; quadrature retains
its performance failure. Both [I/O host jobs](https://github.com/mbbill/Whitefoot/actions/runs/34388362124)
and all four [I/O benchmark/read jobs](https://github.com/mbbill/Whitefoot/actions/runs/34388362058)
pass.

The local CPU regression has a narrower attribution. In the first W1
65536/t16 cohort, core-time totals for 513 calls have medians 275.10 ms
before and 274.40 ms after; cycle time outside core changes from 509.68 to
534.94 ms, while checking outside cycle stays near 6.92 ms. That outside-core
interval includes prefix preparation, result access and release. The linked
WF/accessor text starts move by 32 bytes despite identical computation objects.
A second local 60-process cohort uses the same objects and five alternating
passes over fine/coarse W1 cells. Its normal layout repeats the fine batch-CPU
regression, median 1.0395 (1.0228–1.0411; CPU A/A 1.0025), and outside-core
cycle median 1.0584 (1.0425–1.0620). A Mach-O order-file control holds 16
WF/accessor/native text starts fixed, including outlined cold text. There the
fine CPU median is 1.0078 (0.9975–1.0474; A/A 1.0056) and outside-core
cycle median 1.0064 (0.9988–1.0525). Coarse CPU medians stay near 1.002
under both layouts. This supports a local code-layout contribution; matching
text starts does not match every relocation, data address or stub, and does
not explain other platforms' scheduler regressions. The first object-order-only
build fails its cold-text address invariant before timing; that failed build
is retained. The successful `late-idle-layout-run2` source snapshots, order file,
object hashes, raw rows and reproduction script remain local-only evidence.
Timing starts after the exact canonical process exits, without concurrent
local tests.

Under the stated cross-platform no-regression criterion, restore dc383eef's
idle-registration order in the maintained core. Keep the independent report
classification repair, wake-epoch diagnostics, frozen comparisons and all
limits. This rejects the change as the general default, not the ARM cell
benefit or every future adaptive policy. Independent selection review checks
the native ratios, gate outcomes and layout-control scope and finds no blocker
to that restoration. The restored tree still requires its own current checks;
461a7130's canonical success is not success of a later revision.

The restored tree passes native scheduler smoke, the 200,000-task concurrent
deque-reuse probe, and the 16,000-submission default I/O route probe. A rebuilt
ordinary compiler links the scalar `--par --no-vectorize` FIR command, which
runs successfully at one and four workers (correctness only). Final scoped
review confirms the two core files exactly match dc383eef, checks all five
CI table rows and recomputes the 60-process layout result, with no remaining
finding. Exact restored revision `815b97e9` passes local canonical `make check`
with a clean worktree: compiler 586 s, research 212 s, full native conformance
98 s, snapshots 20 s. Its CI qualification remains open; a local gate pass
does not establish the five-platform performance requirement.

The restored revision's [five-platform screen](https://github.com/mbbill/Whitefoot/actions/runs/34391106128)
finishes with all five formal-runtime jobs failing the declared performance
screen. For small/coarse FIR (`n=4096`, tile 1024), long-batch paired medians
are below; ratios below one favor the restored candidate. The macOS ARM host
has two participants; the other rows have four. These are runtime attribution
comparisons, not ordinary-CLI performance or a new native-reference ceiling.

| Host | Historical formal `before` | Recovered | Restored parent `slotbase` | Byte-identical A/A |
|---|---:|---:|---:|---:|
| Linux x64 | 1.0163 | 1.4956 | 1.0231 | 0.9934 |
| Linux ARM64 | 1.0726 | 1.7342 | 1.0139 | 1.0093 |
| macOS ARM64 | 0.9828 | 1.5506 | 1.0132 | 1.1430 |
| macOS x64 | 0.4264 | 1.0635 | 0.6566 | 0.8637 |
| Windows x64 | 1.0154 | unavailable | 1.0165 | 1.0336 |

Linux recovered comparisons are stable across these five pairs: x64
1.4571–1.5155, ARM64 1.6939–1.7507. Both macOS rows have large A/A drift and
cannot select a runtime policy from their medians. Windows small/finer tile
64 still costs 1.0906 of `before` (1.0495–1.1435; A/A 0.9787). Restoring the
previous policy therefore does not close the broader performance deficit.
The recovered control's previously documented memory-order defect remains;
its timing does not qualify that implementation's correctness.

The restored [gate](https://github.com/mbbill/Whitefoot/actions/runs/34391106075)
has eleven successful jobs and one cancelled Linux unit job at the existing
eight-minute ceiling. Its largest scheduler enumeration finishes before
cancellation; the frozen-source proof-root test is still outstanding. This
is an incomplete CI check, not a language rejection or evidence of a runtime
failure. Both [I/O host jobs](https://github.com/mbbill/Whitefoot/actions/runs/34391106143)
and all four [I/O benchmark jobs](https://github.com/mbbill/Whitefoot/actions/runs/34391106543)
pass. The limit and assertions remain unchanged.

### Open observational-counter cost experiment

The maintained runtime updates its physical-thread counters even when
`WF_SCHED_REPORT=0`; the frozen recovered timing build erases its counters.
This known accounting asymmetry needs a same-source measurement. An explicit
`WF_SCHED_STATS=0` build removes only counter increments and their observer
address lookups, leaving the record, lane and thread layouts unchanged.
Private grant queries return zero and scheduler reports are unavailable in
that build, rather than presenting zero counts as a measured idle schedule.
The current default remains one; no ordinary compiler performance improvement
is delivered or claimed by merely adding this experimental override.

Compare enabled/disabled builds of the restored maintained core using the same
scalar computation objects, participant counts and byte-identical replicas.
Keep diagnostic runs enabled and separate. Check native task/I/O behavior and
the absence of reported counters when disabled; inspect every erased argument
for scheduler side effects. Use fixed computation text placement as an
additional local attribution control, not a substitute for normal CLI timing.
The one-participant FIR path runs sequentially and records no scheduler
increments; treat any apparent gain there as a negative control for layout
and measurement noise, not a benefit from erasing counters.
Any adoption must reach the ordinary compiler link on every supported target
and retain instrumented correctness witnesses. No source acceptance, public
signature or execution-semantics change is intended.

The local native deque and ThreadSanitizer checks pass with both settings
(200,000 tasks per run), retaining the enabled counter assertions and requiring
remote execution independently of those counters. The disabled shared runtime
also passes the 16,000-submission default file/TCP route probe. The first route
attempt was denied a loopback listener by the execution sandbox; the same
assertions pass with that permission. Six separate FIR diagnostic processes
at one, two and four participants verify that enabled builds report counters,
disabled builds do not, and each starts the requested participant count.
Scoped independent review of the initial seven-file change finds no blocking issue:
all nine erased address expressions have no scheduler action or variable-length
array evaluation, and the Windows test loop retains both configurations.
Windows execution is covered by the subsequent CI result below; ordinary-CLI
adoption remains unselected. Local timing evidence follows below.

The local M1-series counter-cost cohort uses two layouts, one/two/four
participants, 4096/65536 samples, tile 16/1024, and five alternating passes
of enabled/disabled/byte-identical-disabled-replica: 360 processes per cohort.
Small inputs have 4096 warm calls per process; large inputs have 512. Both
use the same strict scalar WF/native objects. The fixed-layout pair matches
all 16 computation/accessor/native text starts. Timing starts after the exact
815b97e9 canonical process exits, with no concurrent local test or compilation.

The first timing cohort (`counter-cost-perf-run2`) retains all 360 complete,
oracle-checked outputs, but its driver exits with a parsing error because its
source was edited during execution to add a separate summarizer. It is not
reported as a successful driver run. The immutable replacement driver and
separate summarizer both pass for `counter-cost-perf-run3`, again checking
all call and actual-participant counts. The first build attempt, which lacked
the copied native header, failed before timing and is retained separately.

Large/fine FIR gives contradictory batch-to-batch evidence. Below are medians
of five paired process core means, disabled/enabled, with each range; the
first column is explicitly the complete data from the failed driver cohort.

| Layout / participants | First cohort data | Clean replacement cohort |
|---|---:|---:|
| Default / 2 | 0.9196 (0.9152–0.9239) | 0.9773 (0.9219–0.9893) |
| Fixed computation text / 2 | 0.9246 (0.9168–0.9845) | 1.0574 (0.9922–1.0718) |
| Default / 4 | 1.2470 (0.8747–1.2796) | 0.8532 (0.8449–1.2192) |
| Fixed computation text / 4 | 1.2607 (1.1850–1.3188) | 0.6648 (0.5835–0.6995) |

The fixed-layout W4 batch CPU medians likewise reverse from 1.1424 to 0.7948.
Extracted complete `__TEXT,__text` bytes match between cohorts for each
fixed-layout mode, so a different rebuilt instruction stream does not explain
that reversal. This does not establish equal runtime placement, allocation
addresses, OS scheduling or work distribution. The clean run's W4 fixed-layout
A/A core ratio is 0.9763 (0.9466–1.0256); within-batch A/A alone does not detect
the full cross-cohort instability. Do not select only the favorable cohort.

The sequential negative control is useful too: default-layout W1 large/fine
batch CPU ratios are 1.0338 and 1.0420 despite no scheduler increments; with
fixed computation text they are 0.9992 and 1.0006. That supports a placement
contribution to this non-scheduler cost, not a claim that counters make
sequential execution faster. There is no established general counter-erasure
benefit, and the compiler default remains enabled. The build override remains
an explicit attribution tool while scheduling/placement causes are unresolved.
Source snapshots, flags, symbol/text checks, hashes, raw process rows and
reproduction drivers for both cohorts remain local-only evidence.
An attempted follow-up macOS stack sample produced no report before the
benchmark exited; the sampler remained idle and was terminated. Those
instrumented runs are not timing evidence or an attribution result.

The existing five-platform formal screen now adds `nostats` from the same
maintained sources, with only the explicit counter override. All previous
controls, thresholds and participant checks remain. Separate diagnostics
request reports in both current modes: `nostats` must omit scheduler counts
while retaining the external wake-epoch observation. This comparison uses
ordinary link placement and may include layout effects; it does not by itself
isolate counter instruction cost. The subsequent cross-platform result is
mixed, and no new default is selected from the local reversal.
The exact updated diagnostic parser accepts all six existing local on/off
reports and rejects all six with the expected report availability inverted;
shell syntax and patch whitespace checks pass.
Final scoped independent review of the nine-file change recomputes both
360-process cohorts from raw output, verifies the five-platform CI table and
unchanged prior assertions, and finds no blocking issue. This does not certify
the whole PR, the new revision's full gate or its pending platform execution.

At `16dece48`, the [compute CI run](https://github.com/mbbill/Whitefoot/actions/runs/34394169877)
executed all five formal screens, including both counter configurations; all
five still fail overall performance acceptance. The exact local canonical
`make check` passes, all twelve gate jobs pass, and the existing I/O host and
benchmark jobs pass. These are correctness results, not compute acceptance.
Long-view ratios below are **enabled/disabled** (above one favors disabling),
five matched process pairs. Linux x64 W4, 4,096 outputs/tile256 is
**0.8441 [0.7770, 0.9057]**; Linux ARM64 W2, 4,096/tile1024 is
**1.1685 [1.1140, 1.2358]**. Windows W4, 65,536/tile16 is
**1.0077 [0.9970, 1.0217]**. macOS rows show substantial variability;
neither their nominal gains nor the cross-platform mixture establishes a
general counter-removal benefit. Runtime counters therefore remain enabled
by default. Ordinary command measurements are a separate panel above.

## Earlier investigation and evidence

The selected question is whether Whitefoot's proof-derived compute parallelism
can approach the fastest equivalent native implementations while tasks execute
on each worker's current call stack. Recover the local join/help/steal path,
remove completion scheduling from the compute execution path, and measure the
remaining costs separately from generated kernel code. Performance takes
precedence over sharing an execution mechanism with I/O.

This investigation starts at main revision
`6cc00984415a39c507fa74897c9269b10beebfee`. The source audit below identifies a
recovery control, now implemented and locally qualified in the
[compute runtime experiment](../../experiments/compute-runtime/README.md).
The normal compiler link path is unchanged. The experiment retains qualified
workloads, native references and dated performance results. The
[WF workload coverage](WORKLOADS.md) and [reference matrix](BASELINES.md)
distinguish that evidence from broader comparisons still missing.
Further compiler scheduling-policy development is deferred. The existing
scalar-leaf filter becomes the `--par` default at 16 nonconstant IR operations,
with an explicit `off` override. Recursive frontier and sequential refusal
remain opt-in; no universal recursive grain is selected. The
[compiler guide](../../../compiler/README.md#parallel-and-completion-lowering)
owns these current defaults and their evidence limits.
The immediate integration scope is the measured compute foundation and its
test/reference tools, before returning to I/O execution design.
The separate
[I/O investigation, PR #26](https://github.com/mbbill/Whitefoot/pull/26), is
paused and retains its implementation and measurements. Its stackless path is
not the base of this work.

This directory owns the compute runtime experiment's rationale and reference
selection. Keep it current while that experiment is active; consolidate or
remove superseded material when another investigation takes over the question.
The active [specification](../../../spec/kernel-spec.md) defines acceptance and
the [compiler guide](../../../compiler/README.md) describes implemented paths.
The recovery experiment adds executable checks under the root `make check`;
it changes no language rule, public ABI, or conformance expectation.

## Recovering the actual compute path

The relevant revisions are different controls, not interchangeable historical
performance baselines:

| Revision | Source evidence and role |
| --- | --- |
| `408dd34ebe4a385486002b32dbc3ecaf661b2aaa` | [Introduces per-thread work-stealing deques](https://github.com/mbbill/Whitefoot/commit/408dd34ebe4a385486002b32dbc3ecaf661b2aaa). It identifies the original mechanism; its dated performance claims do not qualify today's compiler or references. |
| `fee335654d9dea027f4636bbad448d57a4e84d08` | Last repository revision before the `17ec9458` I/O integration. Its [parallel runtime](https://github.com/mbbill/Whitefoot/blob/fee335654d9dea027f4636bbad448d57a4e84d08/compiler/src/backend/par_runtime.c) is the recovery reference for a compute worker pool without completion bridge integration. |
| `9051576f6a4d723b4eb072850f49859853decae7` | Parent of the POSIX replacement. Its [parallel runtime](https://github.com/mbbill/Whitefoot/blob/9051576f6a4d723b4eb072850f49859853decae7/compiler/src/backend/par_runtime.c) still helps on the current stack, but already exposes a help seam for the completion bridge. It is not the clean pre-I/O control. |
| `92b19e1a703159d461932792bc334c10d2e89b89` | [Replaces the POSIX compute runtime](https://github.com/mbbill/Whitefoot/commit/92b19e1a703159d461932792bc334c10d2e89b89) with `sched/core.c` and `sched/entry.c`, deletes `par_runtime.c`, and puts the program entry on a scheduler pool stack. |
| `6cc00984415a39c507fa74897c9269b10beebfee` | Current investigation base. Its [compute join](../../../compiler/src/backend/sched/core.c) retains the shared stack scheduler. This is the current-runtime control, not a restored compute runtime. |

At `fee33565`, `wf__par_join` owner-pops its task and calls it directly if it
has not been stolen. Otherwise it executes other local tasks and enters
`wf__par_wait`, which checks the target, owner-pops, steals, and runs available
work nested on the current stack. Only after repeated empty searches does it
yield and eventually wait on a condition variable. No suspended-stack pool or
I/O completion drain appears in that path. See the historical
[join](https://github.com/mbbill/Whitefoot/blob/fee335654d9dea027f4636bbad448d57a4e84d08/compiler/src/backend/par_runtime.c#L757)
and [wait](https://github.com/mbbill/Whitefoot/blob/fee335654d9dea027f4636bbad448d57a4e84d08/compiler/src/backend/par_runtime.c#L457).

The old implementation is not cost-free. Owner-pop has strong atomic ordering
and races with the thief on the last entry; publication checks idle workers
and may wake one; task completion has a waiter handshake; idle workers can
enter the kernel. Reclaiming one's own newest task avoids the shared completion
publication, but that does not make all offers or all joins free. Those costs
must be measured rather than inferred from the older comments.

At the current base, `wf_sched_join` checks DONE and attempts its own newest
child first. It then calls `wf_sched_take_target` and parks if another stack is
available. Nested owner-pop/steal execution is the no-target compute fallback.
This reversal of priority is visible in
[core.c](../../../compiler/src/backend/sched/core.c). Changing `WF_STACKS`
does not restore the old policy: the pool has a supported minimum and still
supplies switch targets. The earlier
[park-on-miss design, section 12](../io-model/PARK-ON-MISS.md#12-measurements-before-a-line-of-the-compiler-changes)
already identifies nested helping as the compute comparison.

## Where I/O currently enters a compute measurement

The link driver uses the union of parallel and completion requirements to
include the shared scheduler. `wf__floor_run` then delegates the program entry
to that scheduler when it is linked. The coupling therefore includes startup,
stack reservations, task completion, joining, idle progress, and shutdown; it
is larger than the stack-switch instruction sequence.

There is a second coupling: `par_layout.wf` publishes its checksum with
`write_once`, and the driver test explicitly requires that program to link
completion support. A computation with no I/O in its hot loop can therefore
still carry the whole shared scheduler. Inspect
[the driver and its runtime-selection test](../../../compiler/src/bin/whitefootc.rs)
and [the entry floor](../../../compiler/src/backend/wf_floor.c).

The restored path must be distinguished by its actual linked and executed
runtime, not by an I/O-free workload label. Its kernel should return a value
or fill caller-owned output that a host oracle checks outside the timed
compute region. A separately reported end-to-end measurement must include
equivalent setup and output costs. Do not replace a real computation with an
unchecked return code or silently subtract those costs from an end-to-end row.

Removing I/O costs from compute does not authorize weakening stack-exhaustion
handling, memory safety, or task-result lifetime rules. Recovery needs the
current compiler's ordinary task entry contract and result ownership; simply
linking the old runtime against new frames without checking layout and calling
conventions would not establish compatibility. Keep existing I/O tests intact
as coverage of that capability while its execution mechanism is separated.

## What the first measurements must distinguish

1. **Kernel quality:** current WF sequential code against optimized equivalent
   native sequential code, with identical arithmetic and output semantics.
2. **Scheduling policy:** the same emitted compute kernel and task boundaries
   with the current shared runtime and the restored compute runtime. Attribute
   any compiler or layout differences explicitly if identical code is not
   achievable.
3. **Unstolen cost:** both sequential-clone execution and actual parallel-path
   execution without successful steals. A one-worker setting that selects the
   sequential clone cannot measure task publication and local join costs.
4. **Resource cost:** worker counts, startup and shutdown, allocations, stack
   reservations versus resident memory, task publication, steals, kernel
   switches, and work performed while another task is joined. Timing and
   diagnostic observation use separate builds or runs.
5. **Competitive performance:** the fresh [reference matrix](BASELINES.md),
   including static partitioning where the workload permits it. A win against
   one dynamic scheduler is not an upper bound on native performance.

Use current accepted programs and equivalent native kernels. Historical
[proof-derived parallelism results](../proof-derived-parallelism/RESULTS.md)
and its [dated reference rotation](../proof-derived-parallelism/bench/baseline-20260823/README.md)
identify useful workloads and earlier issues. They do not replace a new
same-host comparison: compiler semantics, code generation, thread budgets,
toolchains, machines, and statistical treatment have changed.

`par_layout.wf` remains a historical regression sample. The new investigation
needs multiple substantive WF programs with different algorithms, layouts,
inputs, and parallel structures, as defined in [WORKLOADS.md](WORKLOADS.md).
Adding native references or increasing the layout repetition count does not
provide that coverage. The WF program suite and its native references are
separate implementation deliverables.

## Execution contract to preserve

Compute tasks remain ordinary calls on worker stacks. At a join, execute
available permitted compute work before sleeping; an I/O completion cannot be
a hidden dependency of that helping path. The existing ownership and proof
judgments remain the authority for legal overlap. Public signatures and
contracts must suffice for separate compilation; no hidden coroutine calling
convention is selected by inspecting a separately compiled function body.

Whether regular loops should use static partitioning, how coarse tasks should
be, and which idle policy minimizes measured cost remain experiment questions.
The target is explainable proximity to each workload's hardware and dependency
limits, with losses exposed. No universal fastest-runtime claim follows from
the historical audit or the amount of time spent on the preceding I/O work.
