# C2 ordinary-call workload measurements

The first cohort was measured 2026-09-12, 20:27–20:28 PDT, during the then-named
v0.58 amendment on PR #30. The combined release is now v0.55 over main v0.54.
The worktree was based on `d695f38513a8513eac62b73108517f982435285f` with the
ongoing C2 changes. These are measurements, not acceptance or speed gates.
This note and [the 84 raw samples](c2-many-files-samples.csv) are one retained
measurement artifact, owned by this experiment; supersede the interpretation
when a later matched run measures the same question.

## Protocol and result

The existing `gen.c`, `workload.h`, `baseline.c`, and `runner.c` define the
experiment. There are 8192 files of 1–16 KiB, 71,024,640 bytes total, generated
from the existing index-derived seeds. Each file receives one positioned read
and the existing zero-seeded, position-weighted digest. Every variant on every
warm-up and recorded pass published exactly
`17098009301725298919 00000000000071024640`.

Each pass runs the complete twelve-line plan, reversing order on alternate
passes: two unrecorded warm-up passes, then seven recorded passes. There were
no discarded samples, retries, affinity settings, or explicit cache eviction.
The page-cache state is therefore the naturally warmed state after those
passes; no uncached claim is made. `WF_WORKERS`, `WF_IO_HELPERS`, and
`WF_IO_NOCACHE` were unset. The native controls retain their existing direct
loop and striped pthread pool algorithms. WF sources retain their existing
algorithms, including per-iteration scratch in `many_files_loop`.

| Variant | Median ms | Min ms | Max ms | Median user ms | Median system ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| C direct | 176.59 | 169.25 | 181.72 | 72.73 | 102.78 |
| C pool 2 | 119.01 | 99.82 | 142.36 | 75.80 | 155.63 |
| C pool 4 | 90.03 | 83.10 | 105.76 | 78.22 | 241.38 |
| C pool 8 | 88.88 | 79.28 | 98.63 | 81.23 | 344.92 |
| WF narrow, no overlap | 202.18 | 192.31 | 237.87 | 81.98 | 135.14 |
| WF narrow, default | 203.46 | 189.45 | 258.13 | 82.30 | 137.97 |
| WF loop, no overlap | 208.80 | 200.56 | 268.05 | 86.02 | 137.41 |
| WF loop, default | 213.88 | 200.09 | 318.49 | 86.72 | 142.76 |
| WF wide 4, no overlap | 218.98 | 190.92 | 299.98 | 83.58 | 164.68 |
| WF wide 4, default | 202.19 | 191.42 | 328.42 | 81.05 | 132.90 |
| WF wide 8, no overlap | 205.60 | 188.53 | 354.55 | 79.48 | 140.99 |
| WF wide 8, default | 212.88 | 185.94 | 345.48 | 78.38 | 145.68 |

The four default WF medians are 1.14–1.21 times the C direct median and
2.27–2.41 times the sampled eight-thread pool median. These describe this
cohort. They do not identify an optimal pool size or a change against a
historical machine. The host was shared with compiler validation work; its
1/5/15-minute load averages were 5.25/19.55/42.38 at the start and
5.67/18.47/41.33 at the end. The large spreads preclude fine percentage claims.

## What can be attributed

**Lost staging.** Each default/no-overlap pair emits byte-identical LLVM.
All eight modules contain zero `wf__par_` references. `many_files_loop` retains
the formerly staged counted workload, but PAR-3 is deleted. Its ordinary
PAR-2 ledger currently denies the loop at condition 2, because the body
contains a discarded expression statement. This is a permission observation,
not a source rejection. No staged pipeline exists in the measured executable.
The differing timings of identical modules expose run noise; they do not
price removal of the old staged implementation.

**Shared-factory serialization.** The wide modules contain four/eight
ordinary `wf_open_file` calls followed by four/eight `wf_read_at` calls. Every
call uses the same resolved `files` place. PRE-1 declares an exclusive factory
parameter and a write on it for both operations. Even a same-block pair with
otherwise distinct parameters cannot receive ordinary PAR-1 permission:

```wf
fn pair(files: &uniq HandleFactory, left: &uniq ReadFile, right: &uniq ReadFile,
        a: &uniq MutSlice<u8>, b: &uniq MutSlice<u8>) -> result: own unit
  reads(files, left, right, a, b), writes(files, left, right, a, b) contract {
  requires 1_u64 <= len_of(deref(a));
  requires 1_u64 <= len_of(deref(b));
} {
  doc "Two ordinary calls share one exclusive factory.";
  region {
    let first = read_at(factory: &uniq deref(files), file: &uniq deref(left), destination: &uniq deref(a), file_offset: 0_u64, start: 0_u64, end: 1_u64);
    let second = read_at(factory: &uniq deref(files), file: &uniq deref(right), destination: &uniq deref(b), file_offset: 0_u64, start: 0_u64, end: 1_u64);
  }
  return unit;
}
```

Compiling this witness with `--par --par-ledger --emit-llvm` succeeds and
reports `pair(read_at, read_at) condition 2: the exclusive loan of s1 overlaps
the exclusive loan of s2 at &uniq deref(files) vs &uniq deref(files)`.
Thus factory sharing is an independently demonstrated obstacle to ordinary
call overlap. It is not an independently timed component: this amendment also
removes staging, and the retained workloads change neither one in isolation.
The pool gap measures available native concurrency plus implementation
differences; assigning it entirely to either cause would be unsupported.

**Ordinary call transport.** Optimized `ordinary_values.ll` and arm64 assembly
quantify the current wrapper cost without inferring it from wall time:

| Boundary or successful path | Observed transport |
| --- | --- |
| `wf_open_file` to its C body | One 16-byte view materialization, one 32-byte stack frame including saved frame/link registers, one nested call |
| `wf_read_at` to its C body | The same; no file-content copy |
| Narrow/loop successful file iteration | Three 32-byte owner copies in optimized `wf_exercise`: two after open and one before close; 96 bytes per file |
| Open/read result destination | The caller passes a destination pointer; the 288/248-byte result is not copied wholesale on the successful narrow path |

The two view wrappers therefore materialize 262,144 descriptor bytes over the
8192-file run; the three retained owner copies account for 786,432 bytes.
These are structural byte/call counts, not time estimates. Failure-only enum
copies and whole-process launcher copies are outside those per-success counts.
The direct C control also differs in component validation, result construction,
native engine submission/join, cleanup, and WF source lowering. Its timing
difference cannot be assigned wholly to the ordinary ABI. A separately matched
native caller of the same ordinary functions remains needed to time that
component. No compiler optimization or API change was made for this reading.

## Reproduction and identity

Host: arm64 macOS 26.6.2 (25G83), Apple Clang 21.0.0
(`clang-2100.1.1.101`), Rust 1.98.1. Native C and emitted LLVM were compiled
with `-O2`, without LTO. The ordinary library used the same C/LLVM object list
and platform flags as `whitefootc`, via the existing
`../container-representation/native.mk`. Native controls and the generator
used the experiment's existing `-std=c11 -O2 -Wall -Wextra -Werror` commands.

Generate the default tree with `gen TREE 8192 16`. Build each of
`many_files_{narrow,loop,wide,wide8}.wf` normally and with `--no-overlap`.
The plan is C direct, C pool2, C pool4, C pool8, then each shape's no-overlap
and default executable in that order. Run from `TREE`:

```sh
runner PLAN.tsv 7 2 '17098009301725298919 00000000000071024640'
```

Raw recording used a disposable copy of the existing runner with only this
line inserted immediately before `line->wall[line->recorded] = sample.wall_ms;`:

```c
fprintf(stderr, "sample,%lu,%s,%.6f,%.6f,%.6f\n", pass - warmup + 1,
        line->label, sample.wall_ms, sample.user_ms, sample.system_ms);
```

The insertion is after the original timing interval and checksum validation;
pass order, timing boundaries, warm-ups and summaries are unchanged. The CSV
retains all those `sample` lines without their constant prefix. No temporary
runner, binary, generated tree, or log dump is added to the repository.

The following SHA-256 values identify the emitted default modules; each
corresponding no-overlap module has the same hash:

| Module | SHA-256 |
| --- | --- |
| `many_files_narrow` | `1cdd34d597b283c032c2b66720c97d9b4098b28796da49d23d3c8aa6a346c3e9` |
| `many_files_loop` | `bbdee1365623dee4767f84268e0c46d3340acb8673cafd66a1e26edb2689e65c` |
| `many_files_wide` | `5f8a1deee15b1ff36ac88589b84204d3414d0d8b0b4cbcfead50a70813048f2c` |
| `many_files_wide8` | `8d07a48cef791b59aa266d2e80fb98e38259519e92f5a58c4b8236066fbef579` |

Compiler executable SHA-256:
`5b897252ec1156ca2419d7babe543bd8d230db7b569054fed42f3306444f2464`.
Original runner source SHA-256:
`d265fcb587ad4d8ec158ef9c57db60b461a39d969c64406583a9fd6aab8b9cc4`.

This local reading does not include Linux TCP: its existing `netload` harness
uses epoll and requires native Linux CI. The previous PAR-3/completion tables
remain historical observations; this table neither reruns their old compiler
nor creates a before/after estimate from different hosts or dates.

## Windows CI on b5d10194

The native Windows job in [run 34735760872](https://github.com/mbbill/Whitefoot/actions/runs/34735760872/job/103666780433)
completed successfully on 2026-09-13 UTC at revision
`b5d1019425c216e172c4e013dab798ec6e0d6533`. Its complete
[150 recorded child samples](c2-windows-samples.tsv) are retained here, including
QPC wall time and process user/kernel time. Artifact `bench-windows-measured`
had SHA-256 `b75253c0e1c968dc66bd5c62e5ee68942adf9c110cb22c3acf1bfb4fd566d669`.
There was one fixed cohort per comparison, with two warm-up pairs followed by
15 recorded alternating pairs, without retrying a wide-spread cohort.

The host was Windows Server 2025 Datacenter 10.0.26100, image
`win25-vs2026 20260907.229.1`, AMD EPYC 7763, four visible logical processors,
17,174,360,064 bytes of memory, High performance power plan, Clang 20.1.8 and
Rust 1.98.1. The harness pinned children to mask `0xf`; parallel children used
four workers. Each compute child ran four batches. A sequential pre-read of
all eight files established the warm I/O cohort.

| Cohort | Reference / candidate | Reference median ms | Candidate median ms | Median paired ratio | Ratio p10–p90 |
| --- | --- | ---: | ---: | ---: | --- |
| Compute | Sequential / `--par` | 4863.954 | 2074.169 | 0.4251 | 0.3607–0.5094 |
| Warm I/O | `--no-overlap` / default | 226.245 | 223.768 | 0.9921 | 0.9430–1.0054 |
| Mixed default | `--no-overlap` / default | 289.528 | 290.842 | 1.0030 | 0.9921–1.0136 |
| Mixed parallel | Default / `--par` | 288.987 | 206.112 | 0.7129 | 0.7052–0.7237 |
| Mixed total | `--no-overlap` / `--par` | 292.153 | 207.285 | 0.7111 | 0.7031–0.7224 |

The compute ratio has a wide spread: relative median absolute deviation
14.89%, relative p10–p90 width 34.96%. The other widths are 6.28%, 2.15%,
2.60%, and 2.71% respectively; these descriptions are not speed gates.
All samples checked exit status, exact output and empty diagnostic output.
A separate observed mixed run reported `grants=1024`, and required native IOCP
submission and reaping. Ordinary compute parallelism therefore still runs
beside the retained native I/O implementation.

These comparisons all use the ordinary callable ABI. The two reads return
before the following statement, and shared factory state prevents their
ordinary parallel overlap. Near-unit default/no-overlap ratios do not measure
the old PAR-3 pipeline's benefit: that pipeline is absent from both candidates.
The mixed parallel gain demonstrates remaining ordinary compute parallelism;
it does not establish I/O overlap. Neither factory serialization nor the ABI's
own transport cost is isolated by these pairs. The independent place-overlap
and transport counts above supply structural attribution without assigning an
unsupported fraction of wall time to either component.

## Linux CI on b5d10194

The Linux job in [the same run](https://github.com/mbbill/Whitefoot/actions/runs/34735760872/job/103666780522)
completed the traversal table, then failed its network measurement at the
existing twelve-minute step limit. Its host was Linux 6.17.0-1022-azure,
Ubuntu x86-64, AMD EPYC 7763 with four visible processors, ext4 on `/dev/sda1`,
Clang 18.1.3 and Rust 1.98.1. `kernel.io_uring_disabled` was zero.
Artifact `bench-linux` had SHA-256
`cabcf80d1d9861936f3f2be3d54e64a4a7f5569609f7c27372d5e52f8c833842`.

Traversal retained the existing 8192-file input and digest, with two warm-ups
and nine recorded passes in alternating whole-plan order. Every line published
`17098009301725298919 00000000000071024640`. The complete table is below;
the artifact retained summaries rather than individual traversal samples.
`N` labels are native controls, `S` is WF with `--no-overlap`, and `C` is the
existing harness label for WF default lowering. The latter label does not
denote a surviving source completion class.

| Line | Median ms | Min ms | Max ms | User ms | System ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| N.direct | 104.95 | 104.79 | 105.69 | 70.42 | 34.21 |
| N.pool1 | 105.26 | 104.95 | 108.44 | 69.23 | 36.02 |
| N.pool2 | 53.78 | 53.52 | 54.19 | 70.89 | 35.95 |
| N.pool4 | 31.04 | 30.86 | 33.35 | 68.80 | 50.85 |
| N.pool8 | 34.00 | 31.72 | 36.24 | 75.55 | 44.16 |
| N.uring2 | 108.49 | 108.06 | 108.63 | 68.19 | 40.11 |
| N.uring4 | 129.74 | 129.08 | 131.48 | 65.91 | 62.92 |
| N.uring8 | 128.73 | 128.11 | 129.25 | 73.95 | 54.96 |
| N.uring16 | 129.29 | 128.67 | 131.58 | 68.08 | 61.07 |
| N.uring32 | 129.26 | 128.82 | 130.17 | 65.84 | 62.85 |
| S.narrow | 145.27 | 143.80 | 151.23 | 84.06 | 61.05 |
| S.loop | 151.20 | 150.71 | 154.16 | 84.32 | 66.25 |
| S.wide | 141.81 | 141.03 | 150.91 | 81.36 | 61.77 |
| S.wide8 | 143.07 | 140.80 | 148.51 | 78.69 | 65.74 |
| C.narrow.default | 143.43 | 142.26 | 145.95 | 82.90 | 61.08 |
| C.loop.default | 151.90 | 149.39 | 154.14 | 87.03 | 63.02 |
| C.wide.default | 143.33 | 140.03 | 145.51 | 81.70 | 61.82 |
| C.wide8.default | 142.74 | 141.15 | 145.11 | 81.00 | 60.99 |
| C.wide.w0.h0 | 143.35 | 140.68 | 145.53 | 79.92 | 63.23 |
| C.wide.w0.h1 | 143.66 | 140.81 | 145.49 | 81.14 | 63.10 |
| C.wide.w0.h2 | 142.32 | 141.08 | 145.15 | 86.07 | 57.26 |
| C.wide.w0.h4 | 144.77 | 140.97 | 147.69 | 84.26 | 61.28 |
| C.wide8.w0.h0 | 142.55 | 140.71 | 147.12 | 79.53 | 63.11 |
| C.wide8.w0.h1 | 142.06 | 140.76 | 145.24 | 77.66 | 65.15 |
| C.wide8.w0.h2 | 142.89 | 140.50 | 147.20 | 79.28 | 65.04 |
| C.wide8.w0.h4 | 142.73 | 141.28 | 146.75 | 82.43 | 61.70 |
| C.wide.w1.hdefault | 142.55 | 140.65 | 144.38 | 83.01 | 58.01 |
| C.wide.w2.hdefault | 141.90 | 140.70 | 146.71 | 79.18 | 61.80 |

WF default medians are 1.36–1.45 times the direct control and 4.60–4.89 times
the four-thread pool. The absence of gains across wide/helper variants is
consistent with the ordinary shared-factory row and deleted PAR-3 path. It
does not isolate their respective costs, nor make a historical before/after
comparison. The C controls retain their different direct, pool and io_uring
algorithms; their results are not measurements of the ordinary linked ABI.

### TCP noncompletion

The unchanged Linux network harness requested one warm-up and three recorded
passes, but no timed pass began. The step ran from 03:38:27 to 03:50:40 UTC;
GitHub reported `Run the Linux network protocol` timed out after twelve
minutes. The retained network output contains only the compiler build line,
without the four-connection correctness-complete message. Runner cleanup
reported live `wf_echo` and `netload` processes. Therefore there is no TCP rate
or latency table to report for this revision, including no completed C-control
timing cohort: the harness performs all correctness runs before timing.

The source explains the wait. In `netload.c`, `pump` marks a connection finished
without closing it; `client_main` waits at the final all-client barrier before
closing sockets. In `programs/tcp_echo_server.wf`, `serve_one` reads until EOF
before the caller accepts the next connection. With at least two peers, the
first peer waits for other peers to finish, while the server waits for the
first peer's EOF. This affects the initial four-connection verification,
independently of the larger connection counts or listen backlog.

A controlled local check on the current ordinary `--par` server confirmed
that dependency: four clients connected and each sent 64 bytes; the first
received its complete echo, the other three had no readable reply after two
seconds while all sockets remained open; closing the first socket immediately
allowed the second complete echo. This observation diagnoses the source
shape. It is not a Linux throughput sample and the two-second observation
window is not an acceptance or performance gate.

The library requests `WF_SOCKET_BACKLOG`, defined as the host `SOMAXCONN` when
available and 128 otherwise. This CI run did not record its compiled numeric
value or `net.core.somaxconn`, so neither is claimed here. The all-connect
barrier is a further risk if a future source reaches the 1024-connection case
with fewer accept-queue slots; it is not needed to explain this failure.
No protocol, backlog, timeout, server line, or correctness check was changed
to produce a timing table. Whether an ordinary source arrangement can serve
this matched protocol is a separate question; this measurement alone does not
establish that the ordinary object model cannot express it.

## Main current-stack integration on f1285555

These measurements use pushed revision
`f1285555281881dca7bfea8eb3b58648fe5acd5e`, which integrates main
`0b452d29` and its current-stack runtime. They describe this revision, without
turning comparisons between different hosts or worker policies into a C2
before/after claim. The earlier cohorts and TCP noncompletion above remain
historical evidence.

### Windows ordinary compute and I/O

The [Windows measurement job](https://github.com/mbbill/Whitefoot/actions/runs/34746273240/job/103694674940)
passed on an AMD EPYC 7763 guest with four visible logical processors, Windows
Server 2025 build 26100, image `win25-vs2026 20260907.229.1`, Clang 20.1.8,
Rust 1.98.1 and affinity mask `0xf`. Main's worker policy selects three workers
on this host. Each cohort has two warm-up triplets and fifteen recorded
position-balanced triplets, including the native Rust/Rayon compute control;
all five cohorts used one attempt. The cache was warmed by a complete
sequential read of the eight files. All [225 recorded children](c2-windows-main-samples.tsv)
are retained, including native controls, worker count and QPC/process times.
The native control indicates available CPU; it is not an I/O baseline.

| Cohort | Reference / candidate | Reference median ms | Candidate median ms | Median paired ratio |
| --- | --- | ---: | ---: | ---: |
| Compute | Sequential / `--par` | 4775.681 | 2007.710 | 0.4215 |
| Warm I/O | `--no-overlap` / default | 197.888 | 199.997 | 1.0120 |
| Mixed default | `--no-overlap` / default | 277.966 | 278.131 | 0.9997 |
| Mixed parallel | Default / `--par` | 278.048 | 158.838 | 0.5706 |
| Mixed total | `--no-overlap` / `--par` | 278.786 | 159.273 | 0.5703 |

The separately observed mixed run reported `grants=1024` and checked native
IOCP submission/reaping. The two reads still return before later statements;
the `--par` gain is ordinary compute parallelism, not restored PAR-3 permission.
These comparisons do not separately time the shared factory or ordinary ABI.
The preceding structural attribution remains distinct from these wall times.
Worker count and the native-control protocol differ from the earlier Windows
cohort, so the lower mixed median does not establish a compiler speedup.
The retained TSV changes only artifact CRLF line endings to LF; its SHA-256 is
`962088ecedcf370e08d9e9f1adf6574c7d357fd32315d7f4b93aaf0a907a2d34`.

### Linux traversal and TCP coverage

The [Linux measurement job](https://github.com/mbbill/Whitefoot/actions/runs/34746273240/job/103694674949)
passed on an AMD EPYC 9V45 with four visible processors, Linux
`6.17.0-1022-azure`, ext4 on `/dev/nvme0n1p1`, and io_uring enabled. It retained
the 8192-file input, two warm-ups, nine recorded passes, alternating plan order
and exact digest `17098009301725298919 00000000000071024640`.
These representative rows are drawn from the complete 28-line artifact table:

| Line | Median ms | Min ms | Max ms | User ms | System ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| N.direct | 76.14 | 74.42 | 81.86 | 52.15 | 23.06 |
| N.pool2 | 38.94 | 38.25 | 42.37 | 52.20 | 26.10 |
| N.pool4 | 21.86 | 21.54 | 24.59 | 56.25 | 29.13 |
| N.pool8 | 25.52 | 22.67 | 29.10 | 52.79 | 34.52 |
| N.uring4 | 77.51 | 76.26 | 83.50 | 47.58 | 31.72 |
| S.narrow | 100.60 | 99.85 | 106.76 | 54.74 | 45.78 |
| S.loop | 103.98 | 103.67 | 106.15 | 58.30 | 45.23 |
| S.wide | 98.30 | 97.52 | 102.63 | 50.67 | 48.68 |
| S.wide8 | 97.78 | 97.03 | 98.97 | 51.27 | 47.25 |
| C.narrow.default | 101.04 | 100.14 | 105.29 | 55.97 | 44.98 |
| C.loop.default | 104.14 | 103.80 | 106.58 | 54.72 | 49.36 |
| C.wide.default | 99.36 | 97.61 | 103.75 | 55.00 | 43.85 |
| C.wide8.default | 97.00 | 96.56 | 99.09 | 51.94 | 44.95 |

Default WF medians are 1.27–1.37 times the direct control and 4.44–4.76 times
the four-thread pool on this host. Those controls use different algorithms;
their gap does not isolate wrapper cost or either lost overlap permission.
The default/no-overlap pairs contain no restored staging mechanism.

Main already sets `NET_LINES="uring epoll"` in this job, because its ordered WF
server cannot complete the generator's concurrent-peer protocol. This run
therefore verifies and measures only those two native TCP servers: it contains
no WF TCP sample. Native four-peer verification passed and all three timed
passes completed, but that success does not close the WF TCP measurement gap
identified above. No C2 gate or correctness expectation was weakened here.

### Compute scoreboard and the baseline boundary

[Compute run 34746273228](https://github.com/mbbill/Whitefoot/actions/runs/34746273228)
completed its existing Linux and macOS verification and five-pass tables.
All four Linux kernels at four workers are shown here:

| Kernel | WF median us | Native reference | Reference median us |
| --- | ---: | --- | ---: |
| Mandelbrot | 5797.9 | oneTBB | 5672.5 |
| Quadrature | 6650.0 | Rayon join | 6737.7 |
| Records | 9586.8 | Rayon join | 8866.3 |
| FIR | 6800.8 | Static partition | 6739.0 |

These are scoreboard medians, not paired before/after results. The macOS runner
reports three CPUs; its four-worker rows are oversubscribed and its broad
distributions do not justify fine differences.

The separate [compute-regression run](https://github.com/mbbill/Whitefoot/actions/runs/34746273148/job/103694674579)
failed while building the baseline arm, before correctness or timing: main's
pre-C2 native library has no ordinary-values units. Its source entry also uses
the removed `command` form. The same-version twin passes 132 correctness rows,
and four emitted modules, four module objects and twelve native objects are
byte-identical with all twin controls empty. That instrument check does not
supply the missing cross-version comparison; its entry/library adaptation is
still an owner question, with no exemption or retry applied.

## Read-heavy CI on b9d79cd0

[Run 34747240600](https://github.com/mbbill/Whitefoot/actions/runs/34747240600)
completed all four measurement jobs at
`b9d79cd001202d06af2a0e8491a189991a135a6f`. The
[Linux read job](https://github.com/mbbill/Whitefoot/actions/runs/34747240600/job/103697856694)
used an AMD EPYC 7763, four visible processors, Linux
`6.17.0-1022-azure`, ext4 on `/dev/sda1`, and io_uring enabled. The
[macOS read job](https://github.com/mbbill/Whitefoot/actions/runs/34747240600/job/103697856865)
used macOS 14.8.9, an Apple M1 virtual machine with three processors and
7 GiB memory. Its initial load averages were 5.85/8.23/7.13. These are
different machines from the preceding traversal and Windows cohorts.

The unchanged `read-bench.sh` protocol generated eight 64 MiB files, then
made 32,768 positioned reads per process: 2 GiB at 64 KiB or 128 MiB at
4 KiB. Every line checked the existing digest before measurement and on
every timed pass. Each of the four tables per host had two warm-up passes
and seven recorded passes, reversing the whole plan on alternate passes.
The following fixed columns are medians in milliseconds; the artifacts
also retain every pool, io_uring and helper-count variant with min/max and
user/system summaries. No individual timing samples were uploaded by these
jobs, and the four-thread control is not a claim about the best pool size.

| Host and requested cache policy | Read size | C direct | C pool 4 | WF narrow, no overlap | WF narrow, default | WF wide 8, default |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Linux, evict at open | 64 KiB | 1717.07 | 1267.26 | 1786.54 | 1746.91 | 1777.31 |
| Linux, evict at open | 4 KiB | 3724.83 | 1496.82 | 3916.55 | 3835.01 | 3864.59 |
| Linux, warm | 64 KiB | 236.63 | 83.83 | 253.84 | 249.91 | 272.48 |
| Linux, warm | 4 KiB | 33.76 | 12.03 | 62.93 | 62.57 | 65.73 |
| macOS, uncached requested | 64 KiB | 1934.03 | 986.56 | 2182.44 | 2330.63 | 2100.47 |
| macOS, uncached requested | 4 KiB | 2257.17 | 1039.38 | 2494.84 | 2358.17 | 2230.64 |
| macOS, warm | 64 KiB | 217.59 | 79.31 | 234.71 | 225.03 | 228.55 |
| macOS, warm | 4 KiB | 39.27 | 20.48 | 39.20 | 40.66 | 43.89 |

The cache probes remain part of the result. Linux confirmed both pre-table
eviction probes; its 64 KiB after-probe found warm pages, as expected from
an eviction at open rather than an uncached descriptor mode. The macOS
64 KiB pre-table probe **refused** the uncached label (111 of 128 sampled
reads at or below 40 us); its after-probe confirmed it. That table therefore
does not establish uncached performance. The macOS 4 KiB table passed both
probes, and all warm tables passed both probes. Each probe samples 64 KiB
reads even beside a 4 KiB workload; none observes every measured read.

The Linux warm 4 KiB medians expose a material total-path gap: WF narrow
62.57 ms (60.95–65.89) and wide 8 65.73 ms (62.98–69.04), versus direct C
33.76 ms (33.61–34.45). That is 1.85–1.95 times the direct median, not
parity. The macOS distributions are much broader: its warm wide-8 4 KiB
range alone is 37.24–102.89 ms. These measurements use the ordinary call
path, with no restored PAR-3 permission. They price the complete source
and native paths; they do not separate factory serialization, native engine
overhead, ownership transport and wrapper calls into timed components.

The same macOS job also completed the unchanged 8192-file traversal and
digest with two warm-ups and seven recorded passes. Direct C measured
217.48 ms (152.90–229.30), pool 4 90.62 ms (61.98–124.61), and default WF
narrow/loop/wide/wide8 188.42/229.44/226.75/220.75 ms. Its full 36-line table
is in `bench-many.txt`; the wide distributions do not establish a narrow
speedup or a regression against a different host.

Artifact identities: `bench-linux-read` SHA-256
`e5ea76c2a428c41ba7e8464a988173e73724564dbad8ae413033de3ba0295a37`;
`bench-macos-read` SHA-256
`c3ac03543dfcced033ff2d76f1429cb8aa8da0a7a894242b3389f991f6a34b67`.
Their host records and complete tables are attached to the linked run.
These successful jobs do not supply WF TCP samples or repair the separate
pre-C2 compute-regression baseline boundary described above.
