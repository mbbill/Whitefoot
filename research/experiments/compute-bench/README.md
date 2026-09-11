<!-- Serves compute-bench: the reader's entry point. It states the one question
     the bundle answers, how to run it, how to read the table it prints, what
     each reference's grain policy is and what it is not, the two flag sets and
     the asymmetry between them, the one compiled-Whitefoot standing the old
     research bundle left behind, and where a table that matters is recorded.
     Nothing here is a gate and nothing here decides a result. -->

# compute-bench

One question, one table per host:

> **For each kernel, at each width, is the Whitefoot program built by this
> tree's `whitefootc` with plain `--par` the fastest thing in the row?**

Everything in this directory exists to make that one comparison honest, and
nothing else is here at all. No number printed by this bundle fails a build, a
check or a job: there is no band, no threshold, no timeout, no budget and no
heuristic anywhere that selects a result.

## What "WF" means here, and what it does not

The `wf` row is the module `whitefootc` emits from the kernel's `.wf` source
under **plain `--par --emit-llvm` and no other flag**, linked with
`compiler/src/backend/sched/{core,prim_host,entry}.c` and
`compiler/src/backend/wf_floor.c` **from this same tree**, built with the flags
whitefootc itself passes clang, and called through a host adapter of at most
eighteen lines of LLVM IR that does nothing but build buffer descriptors and
forward. The adapter is LLVM IR rather than a C prototype because every user
function the compiler emits has internal linkage.

- No C adapter to any runtime is labelled WF.
- No research copy of the Whitefoot runtime is carried here; the runtime
  sources come from `../../../compiler/` by relative path.
- Nothing selects the chunk count: no flag, no environment variable, no source
  edit. The row is the program the compiler produces or it is nothing.

`wf-seq` is the same source compiled `--no-overlap --emit-llvm` and run at width
one. It is the sequential **control**, not a reference: it never enters the
ratio column. It answers "did `--par` buy anything at all", which is a different
question from "is `--par` the fastest".

Two link-time assertions make a `wf` row a `wf` row. The emitted module carries
**weak no-op stubs for every `wf__par_*` symbol**, so a link that loses the
scheduler sources would still link, still run, still produce correct output, and
be silently sequential. The harness therefore references `wf__par_grants()`,
which has no weak stub, and the Makefile requires a strong definition of
`wf__par_publish` and `wf__par_split_budget` in every image after the link.

## Running it

Prerequisites: `clang`, `clang++`, `cmake`, a stable Rust toolchain, and one
network-enabled run of `make deps` (its `fetch` step is the only step in the
bundle that touches the network; everything after it is offline).

```sh
cd research/experiments/compute-bench
export WHITEFOOT_SCRATCH_ROOT=$HOME/do_not_scan   # the default; nothing is
                                                  # ever written in the repo
make deps      # fetch (network) and build the pinned schedulers
make build     # the compiler, both modules per kernel, one image per kernel
make verify    # every form at every emitted width, correctness only
make compare   # the timed passes and the table, into a fresh RESULTS
```

`make deps`, `make build` and `make verify` are the only targets that can fail
on anything other than a missing or malformed row.

Each kernel image also answers directly:

```sh
BUILD=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/build
$BUILD/mandelbrot list                     # forms, grain policies, widths
WF_WORKERS=4 $BUILD/mandelbrot verify wf 4
WF_WORKERS=4 $BUILD/mandelbrot time wf 4 0 5
```

`WF_WORKERS` must be set and must equal the `WIDTH` argument for every form,
native ones included; the driver cross-checks the two before anything else
happens, because a disagreement would silently compare two different widths.

`make programs-check` is the one target the repository's `make check` runs. It
compiles each program in exactly the two modes the table uses and asserts that
`--par` emits a publish site and `--no-overlap` emits none. It takes about two
seconds, links nothing, and needs no dependency.

## Where results go, and the fresh-directory rule

`make compare` writes `manifest.txt`, `raw.tsv`, `table.txt` and `logs/` into
`$(RESULTS)`, which defaults to a UTC-timestamped directory under the scratch
root. **A run never overwrites an earlier table: `compare` refuses a `RESULTS`
path that already exists.** Re-running by hand therefore needs a fresh
`RESULTS=<path>` — that refusal is the rule, not a bug:

```sh
make compare RESULTS=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/results/try-2
```

`make clean` also clears the way, and it is the wrong reach for that: it is
`rm -rf $WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench`, which takes the build
tree, the installed dependencies and **every earlier table under
`results/`** with it. Use it to start from nothing, not to free up one path.

Nothing is written inside the repository. Job logs and artifacts expire, so a
table that matters is copied by hand, with its CI run id, into
[`../../investigations/compute-runtime/RESULTS.md`](../../investigations/compute-runtime/RESULTS.md),
which also carries the reproduce recipe and the two non-pooling rules.

`manifest.txt` records `uname -a`, the online CPU count, the **inherited** CPU
mask and cgroup state (recorded, never narrowed, and marked `unqualified` when
a file is absent rather than reported as "no limit"), the compiler revision,
every toolchain version — the whole of `rustc -vV`, host triple, commit and
LLVM version included, one `rustc: ` line each — the exact flag strings, the
three dependency pins, `BENCH_ARCH`, and the SHA-256 of every kernel image
and every emitted `.ll` before and after the run. If any of those hashes moved
during the run the table is not a measurement of one build and `compare` says
so and fails.

### Reproduce recipe

```sh
export WHITEFOOT_SCRATCH_ROOT=$HOME/do_not_scan
make -C research/experiments/compute-bench deps     # once, with a network
make -C research/experiments/compute-bench build
make -C research/experiments/compute-bench verify
make -C research/experiments/compute-bench compare PASSES=5 CALLS=5
```

The same four commands run in `.github/workflows/compute-bench.yml` on
`ubuntu-24.04` and `macos-14`.

## How to read the table

```
kernel       w form              median_us  mad%  p10..p90_us  ratio  lower  steals  note
...one row per form of this kernel at this width, ascending by median...
<kernel>   <w> BEST REFERENCE = <form>   FASTEST = <form>   WF fastest: yes|no
  <form>: <that form's grain policy, the whole string the binary printed>
  <form>: ...
```

- **`median_us`** is a median of medians: the median over the five passes of
  each process's median over its five recorded calls. A pass is one sweep over
  every cell; each cell is a fresh process; the sweep order is rotated by
  `(pass + kernel index)` and reversed on odd passes, so no implementation ever
  holds a fixed position in the sequence.
- **`mad%`** and **`p10..p90_us`** are spread, as a percentage of the median and
  in microseconds. **Nothing fails on either.** Spread is information: the
  end-to-end Mandelbrot check behind this bundle's sizing spread 11.3 to 13.4 ms
  on a quiet four-CPU box at width four, which is about eighteen percent.
- **`ratio`** is filled only on `wf` rows. It is the **median of within-pass
  matched pairs**: for each pass, WF's process median divided by the lowest
  process median among the parallel references at that same width, printed with
  its `[min-max]` and, on the verdict line, the modal name of the reference that
  supplied the minimum. **Recomputing a ratio from two printed medians does not
  reproduce this number**, and it is not meant to. `serial` and `wf-seq` live in
  the width-one block and never enter the ratio: a parallel row beating a serial
  row answers nothing.
- **`lower`** is how many of the paired passes had WF lower, because a median
  can hide adverse pairs — a wall ratio of 0.999 with three of five pairs lower
  is on the record from the earlier bundle.
- **`steals`** is the median per-call `wf__par_grants()` delta on the `wf` row.
  A `wf` row with a zero steal median above width one is marked `no-lanes` in
  `note`, so a compiled program that did not split reads as that rather than as
  a slow scheduler. It is a mark, never a failure, and it must never be
  "repaired" by reaching for a split-budget environment variable: there is none
  in this compiler, and inventing one would stop the row being the plain-`--par`
  program the table is about.
- **`note`** carries the WF row's emitted chunk count, so a reader never has to
  recompute the split formula to know what was compared, and a reference's own
  fixed mark where it has one. Today the only reference mark is the `static`
  row's `excursions retained`: that reference has no stealing, its skewed-shape
  medians carry multi-millisecond excursions, and nothing in this bundle
  removes an outlier or discards a process.
- Each block ends with `BEST REFERENCE = ... FASTEST = ... WF fastest: yes|no`,
  and under that line a **legend**: one line per form in the block, in the same
  order as the rows above it, reading `  <form>: <grain policy>`. The policy is
  the string the binary itself prints and is **never truncated**, so a
  reference's policy and its role both survive into `table.txt`. The `wf` entry
  reads `compiler-chosen`, because it is. It is a legend and not a column
  because a column has to be as wide as the longest policy string — the
  `static` row's, which is a whole sentence — and that made every data row
  about 200 characters wide, past any terminal that has to read the numbers.
  Nothing is abbreviated to win that width back: the rows carry the numbers and
  the legend carries the sentences, once per block instead of once per row.

The reducer's exit status is nonzero only on a missing or malformed row — a
planned cell with no process, a wrong row count, a non-dense call index, a
`first` phase in the wrong place, a header/trailer mismatch. Never on a ratio,
never on spread, never on an ordering.

### Widths, and which block is the recorded one

The width set is `{1, 2, 4, 8}`. The harness reads the online CPU count once per
process and emits every width at most that count, plus the smallest width above
it when there is one. A width above the count is still emitted, but its header
carries `oversubscribed=1` and the reducer prints `n/a (oversubscribed)` instead
of a verdict: oversubscription rewards schedulers that yield and changes which
one wins, so such a block cannot answer this bundle's question.

**The recorded table for a host is its highest width block that is not
oversubscribed.** A four-CPU host runs 1, 2, 4 and an oversubscribed 8 and
records `W=4`; a three-CPU host runs 1, 2 and an oversubscribed 4 and records
`W=2`; an eight-CPU host records `W=8`. Neither hosted CI leg produces a
qualified `W=8` block; that needs a larger runner or a local host.

`explicit_shutdown` in each process trailer is 1 where the pool can be joined
and torn down (`static`, `parlay`) and 0 where it cannot (`tbb`'s arena,
Rayon's `use_current_thread` registry). `shutdown_ns` is timed after the batch
and is never inside a measured interval.

## Sizes, chunk counts and the sizing window

Each kernel has exactly one size constant. It is chosen from the chunk count the
Whitefoot loop must reach at the highest width the table records, subject to two
bounds: the `wf-seq` median at width one lands in **[5 ms, 60 ms]**, and the `wf`
median at the highest recorded width stays **above 1 ms**, so fork, join and
dispatch cannot dominate the thing being compared. It is never chosen from the
minimum admission threshold: a loop that is merely admitted may still get two
chunks.

The compiler splits an independent map into `2^budget` chunks with

```
chunks = 2^floor(log2(min(16 * lanes, span / ceil(1,200,000 / weight))))
```

so the oversubscription term scales with the width, and the result is rounded
**down** to a power of two — seventeen affordable chunks are sixteen chunks.

| kernel | shipped size | emitted weight | chunks W=2 | chunks W=4 | chunks W=8 | reference chunks |
|---|---|---|---|---|---|---|
| mandelbrot | 98,304 points, limit 256, shape `trailing` | 219 | 16 | 16 | 16 | 1,536 at grain 64 |
| records | 131,072 records, `max_length` 255, shape `unicode` | 812 | 32 | 64 | 64 | 8,192 at grain 16 |
| fir | K=64 taps, N=524,288 outputs | 150 | 32 | 64 | 64 | 2,048 at grain 256 |
| quadrature | M=64 integrations, tol `0x1p-54`, depth 24 | none | n/a | n/a | n/a | frontier depth 8 |

**`max_length` is 255, not 256, and that is a contract bound rather than a
transcription slip.** `summarize_records` in `programs/records.wf` carries
`requires input_count <= 16777216` on the byte buffer, which is proof-only
source evidence erased before lowering, so the C caller is what has to
establish it. At 131,072 records seeded 812381 a limit of 256 generates
16,799,739 bytes — 22,523 over that precondition — while 255 generates
16,698,303 bytes and leaves 78,913 bytes of margin, and it is the largest value
of the constant for which the shipped fixture satisfies the source's own
requirement. Nothing else moves: the record count, and therefore the chunk
count, is untouched, and `prepare()` re-derives the byte total from the batch
its generator actually produced and fails on it rather than trusting this
paragraph. The table header and the `workload` string both print
`max_length=255`, so a recorded row never disagrees with this line.

**The weights are measured facts, not constants.** They were read from modules
emitted by this tree's `whitefootc` on 2026-09-11, and the Makefile re-derives
each one from the emitted `--par` module at every build, into
`$(BUILD)/<kernel>_split.h`, so a codegen change moves the reported chunk count
instead of silently invalidating a number written here. Quadrature emits no
`wf__par_split_budget` call at all: it parallelizes by recursion structure and
has no minimum-size admission threshold.

**Confirm the window on the host that records a table**, before recording it:
the `wf-seq` median inside [5 ms, 60 ms], the `wf` median above 1 ms at the
recorded width, and `steals > 0` on the `wf` row at every parallel width. If a
bound fails, change that one size constant by the smallest power-of-two step
that fixes it, re-derive the chunk count from the formula, and record the new
count here and in the table header. Nothing else moves.

**The window was confirmed on the first host to record a table**, a
four-logical-CPU `x86_64` Linux machine with mask `0-3`, on 2026-09-11 at
compiler revision `33ed2c00`; the run is the dated section in
[`../../investigations/compute-runtime/RESULTS.md`](../../investigations/compute-runtime/RESULTS.md).
Its measured `wf-seq` medians at width one were **mandelbrot 26,674.2 us,
quadrature 16,398.7 us, records 36,113.0 us and fir 26,803.3 us** — all four
inside [5 ms, 60 ms]. Its `wf` medians at the recorded W=4 were 7,154.4 /
24,327.9 / 9,508.3 / 7,398.1 us, all above 1 ms, and every `wf` row reported
`steals > 0` at every parallel width. No size constant moved. These are that
host's numbers, not a property of the sizes: another host confirms the window
again before it records a table, and the specification's own derivation, on a
four-logical-CPU Intel Xeon 2.80 GHz Linux host, was `--no-overlap` 29.6-30.1 ms
for 65,536 Mandelbrot points at limit 256, scaling to about 45 ms at the shipped
98,304.

## Grain: a fixed policy on one side, the compiler's choice on the other

Each reference keeps **one fixed grain policy, not calibrated per host**. The WF
row's chunk count is whatever the compiler chooses. **The table compares
programs, not grains.** Per-host grain calibration is refused deliberately: it
would put a tuning loop in front of every recorded table and make two hosts'
rows incomparable.

| form | grain policy |
|---|---|
| `serial` | none: one thread, a loop over all callbacks. Width 1 only. |
| `static` | `equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew` — quoted verbatim from the string the binary prints, because that role is the half readers of this row keep getting wrong. Its helpers are heap-allocated at the first run and sized from `width`, there is one dispatch per call, and idle lanes busy-poll: that CPU is charged to this reference, not hidden. Its `note` column reads `excursions retained`. |
| `tbb` | oneTBB `parallel_for` with `auto_partitioner` and range grain 1, in a process-lifetime `task_arena(width, 1)` under `global_control`. |
| `parlay` / `parlay-left` | ParlayLib native scheduler, `parallel_for` granularity 1, fixed (not automatic) grain; right- and left-offer fork. |
| `rayon-join` / `rayon-join-left` | rayon 1.12.0 `join`, bisecting the chunk range to one callback per leaf — structurally the fork shape the compiler's splitter emits; right- and left-offer. |
| `rayon-iter` | rayon's own adaptive parallel-iterator splitting. Not a form of quadrature; see that kernel's block below. |

**What the three callback-grain numbers are, and what they are not.** They are
not per-host optima and no calibration was run:

- **Mandelbrot, 64 points per callback** is the old bundle's fixed-grain panel
  setting. That bundle's own per-shape optima for the timed `trailing` shape
  were **16** points per callback on its Linux x86-64 EPYC 7763 host (W=4 core
  median 316.817 us) and **256** on its Apple M1 Pro (227.162 us), over a sweep
  of 1/16/64/256/1024.
- **FIR, 256 outputs per callback** is a choice made here, with no measurement
  behind it at all. The only FIR grain evidence in the old bundle is a WF
  `tile_size` sweep of 257/4,096/65,536 whose best at K=64, N=262,144 was 4,096,
  and that knob belongs to a tile-tree representation this bundle does not carry.
- **records, 16 records per callback** is the one number with a same-shaped
  screen behind it: the old bundle's `group4`/`group16` comparison moved the
  result by about a factor of three in either direction depending on shape.

Both fork directions are reported for every library that has one. Fork direction
is a first-order effect on skewed input, not noise: at M1 depth 4 the reciprocal
ParlayLib ratios were 2.138 and 0.470. Keeping only the direction that matches
generated Whitefoot would flatter the WF row, and "the best reference" would
stop being the best reference.

The automatic-granularity ParlayLib variant is not carried in any kernel: the
old bundle's own grain panel records it 2.865x [2.773-3.071] slower than fixed
grain on trailing input in all five pairs.

## The two flag sets, and the asymmetry between them

This is the one place every compiler flag is written, quoted from the
`Makefile` with its longer comments trimmed:

```make
ROOT   := $(abspath ../../..)
WHITEFOOT_SCRATCH_ROOT ?= $(HOME)/do_not_scan
WORK   := $(WHITEFOOT_SCRATCH_ROOT)/whitefoot-compute-bench
BUILD  := $(WORK)/build
DEPS   := $(WORK)/deps
RESULTS ?= $(WORK)/results/$(shell date -u +%Y%m%dT%H%M%SZ)
WFC    := $(ROOT)/compiler/target/gate/whitefootc
FLOOR  := $(ROOT)/compiler/src/backend/wf_floor.c
SCHED  := $(ROOT)/compiler/src/backend/sched
CC     := /usr/bin/clang
CXX    := /usr/bin/clang++
ARCH   := $(shell uname -m)

# Never -march=native: Clang 18 infers an invalid AVX10 combination on one
# hosted runner and that failure is on the record.
BENCH_ARCH := $(if $(filter x86_64,$(ARCH)),-march=x86-64-v3,)
LOOP       := $(if $(filter x86_64,$(ARCH)),-falign-loops=32,)
TARGET_CPU := $(if $(filter x86_64,$(ARCH)),-C target-cpu=x86-64-v3,)

WARN   := -Wall -Wextra -Werror -Wpedantic
SCALAR := -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize \
          -fno-lto
BASE   := -O3 -g $(WARN) -pthread $(LOOP) $(SCALAR) $(BENCH_ARCH)
CFLAGS   := -std=c11 $(BASE)
CXXFLAGS := -std=c++17 -DNDEBUG $(BASE)
RFLAGS := -C no-vectorize-loops -C no-vectorize-slp -C lto=off \
          -C symbol-mangling-version=v0 $(TARGET_CPU)

# Exactly what whitefootc itself passes clang.
WF_FLAGS := -std=c11 -pthread -O2 -Wno-override-module

KERNELS ?= mandelbrot quadrature records fir
PASSES ?= 5
CALLS  ?= 5
```

`-fno-lto` is repeated on every link line, not only on compiles.

**The two flag sets are an admitted asymmetry, and it is disclosed rather than
glossed.** `WF_FLAGS` applies to `<kernel>-par.o`, `<kernel>-seq.o` and the four
Whitefoot runtime units; `CFLAGS`/`CXXFLAGS` apply to every reference, every
oracle and the harness. So **the WF module and the Whitefoot runtime are built
at `-O2` with no `-march` and no loop alignment, because that is what a
Whitefoot program gets, while every reference is built at `-O3` with both.**
Building the runtime any other way would measure a runtime no Whitefoot program
ever gets, and `-Wpedantic -Werror` over the compiler's own sources is a build
that can fail for reasons that have nothing to do with the table. The asymmetry
cuts **against** the WF row — the row this bundle is trying not to flatter — and
a reader who finds the WF row slow should see this immediately rather than
derive it from two Makefile variables. Loop alignment is not cosmetic here: code
placement is this bundle's largest confound, and six byte-identical kernel
bodies at different offsets once split a width-one median 7.3 ms against 10.7 ms
with no scheduler involved.

**What is not normalized, and cannot be.** `-march=x86-64-v3` and
`-falign-loops=32` reach the C and C++ kernels and no Whitefoot translation
unit. The separately built oneTBB shared library gets only the three scalar
flags, with no `-march` and no alignment. The Rust staticlib gets only its four
rustflags plus `target-cpu` where `BENCH_ARCH` names it, and Rust's precompiled
standard library is not rebuilt with them. Library scheduler internals are
therefore not code-placement-matched with the kernels.

**Vectorization is off for every implementation of every kernel**, in C, C++ and
Rust alike. The emitted module's `fmul.strict` and `fadd.strict` cannot be
reassociated, so a vectorized reference would compare code generation against
scalar code generation rather than one decomposition against another. This is
why the old bundle's lane-blocked SIMD FIR kernels — bit-identical to the scalar
kernel and roughly nine times faster with SLP enabled — are not carried, and it
is a deliberate departure from that bundle's FIR flag set.

## The four kernels

**quadrature** — recursive, unequal subtrees. Adaptive Simpson over a left-peak
Lorentz profile, run as M=64 independent integrations **in sequence**, each one
internally parallel. That restriction binds the references too: a form that ran
the M integrations in parallel with each other would have replaced this kernel's
decomposition with an independent map, and the panel already has three of those.
`rayon-iter` is **absent by construction**: a parallel iterator needs an
enumerable range, an adaptive recursion has none until it is flattened, and a
flattened-frontier iterator is a different algorithm. It is not silently
dropped. `quadrature list` prints it as a row with class `n/a` and that reason
in place of a grain, `make compare` saves that listing as
`$(RESULTS)/logs/quadrature.forms`, and the cell list drops every `n/a` row, so
the form is disclosed and never run. It carries **no row in `table.txt`**,
because nothing timed it and this bundle does not put unmeasured rows in a
table of measurements. The fact is stated in `quadrature_bench.c`, about this
kernel's decomposition; the same rayon backend is a real reference for the three
independent maps, and nothing in the driver reads a form's name to decide it.
The tolerance cannot
be the size knob — the tree saturates at 149,831 nodes because `combined - whole`
underflows to exactly zero — and `depth` cannot be raised past the source's own
`requires depth <= 24`.

**records** — variable-length UTF-8 record batches, irregular per item.

> **The one compiled-Whitefoot records standing the old bundle left, quoted from
> `research/experiments/compute-runtime/README.md:900-908` as of commit
> `70aa8e5b` on branch `codex/compute-runtime` — that tree is not on this
> branch, so read it with
> `git show 70aa8e5b:research/experiments/compute-runtime/README.md`:** on an
> EPYC 7763 VM (two physical cores, four SMT logical CPUs, mask 0-3), at **256
> long-Unicode records**, the compiled Whitefoot program's warm core medians
> were **7,869.056 / 7,910.887 / 7,931.259 microseconds** at requested
> **0 / 2 / 4** lanes, against native `state` and `word` anchors of
> **7,205.072** and **5,230.540 microseconds**. **Every two- and four-lane
> sample in that cell reported no started pool and no steals.**

That is the record of a **refusal, not a result**, and it is not a prior for
anything here. The reason is the admission rule, not the scheduler: at weight
812 the work divisor is 1,478, so 256 records is far below the 2,956-record
threshold, the loop was never split, and the two parallel columns are the
sequential column measured three times. 65,536 records is the first size at
which the compiled program is asked the question at all, and the shipped 131,072
is the first at which it is asked it at the width the table records. The old
bundle has no other compiled-Whitefoot records standing.

**fir** — data-parallel loop, small tasks. `programs/fir.wf` is the one
Whitefoot source here that is not carried verbatim: the old bundle's
`fir.wf`/`fir_direct.wf` pair parallelizes by recursive tile halving into a
`box<FirTiles>` tree whose every sample is read through an accessor call, and
that bundle's own record is explicit that the accessor, not the scheduler, is
why the grouped WF form won every core pair and lost every cycle pair. The flat
form here makes the WF program's observable work the same as every reference's;
the per-output arithmetic and its operation order are preserved and checked bit
for bit, only the representation differs. **Consequence: no recorded FIR number
in the old bundle is comparable with anything here, and its tile form is not
carried.**

**mandelbrot** — loop with imbalance, timed on the `trailing` shape where the
interior points come last. Granularity, not the dispatcher, decides this row: at
65,536 points plain `--par` emits eight chunks at every width, which on
`trailing` is exactly two heavy chunks and a 2x ceiling no lane count can lift.
98,304 points is the middle of the band that yields sixteen chunks, and the
sizing window caps it there — thirty-two chunks would need 175,360 points and
about 80 ms of `wf-seq`, outside [5 ms, 60 ms]. **At W=8 those sixteen chunks
are four heavy chunks over eight lanes, which is a property of the fixture**,
and this block says so rather than letting a reader take sixteen chunks for a
scheduling result. The `static` row here is a strong **regular-work** reference
and not a dynamic scheduling ceiling for skew — which is exactly why the skewed
shape is the timed one — and its medians in the old bundle carried
multi-millisecond excursions with 65-117 involuntary context switches.
**Excursions are retained, never discarded**: there is no outlier removal and no
discarded process anywhere in this bundle.

## Equality

Bit-for-bit against one independent oracle, per call, after the timer stops.
Never a tolerance, never one implementation against another — comparing every
implementation to the same oracle makes cross-implementation equality transitive
and exact for free. No expected count, PASS string or fixture total is stored:
every count is re-derived at run time and printed, and `verify` requires only
that something was compared.

**One verify fixture per flat-map kernel is above the split admission floor.**
The linked scheduler splits an independent map only from
`2 * ceil(1,200,000 / weight)` iterations upward, and every fixture in
Mandelbrot's and FIR's grids is a few thousand elements — below that floor at
every width. Without more, the `--par` module would take its unsplit path
through the whole of `verify`, and the chunked path the table times would never
be the path the oracle checks. So each of those two kernels carries **one extra
fixture** sized from the floor its own emitted module implies: nothing is
stored, a change of weight moves the fixture rather than quietly dropping it
back below the floor, and `verify` prints the weight, the floor, the size and
the chunk count the run produced —

```
# mandelbrot split fixture: weight=219 floor=10960 points=11008 chunks=2
# fir split fixture: weight=150 floor=16000 outputs=16128 chunks=2
```

— `chunks=0` at width one, where the scheduler does not split at all. This is
the one place the two verify grids go beyond the fixture lists the bundle's
implementation specification names.

The timed interval is identical for every implementation of a kernel:
**allocate the output buffer, zero-fill it, compute, join every participant.**
Release and free are outside it. The allocation and the zero-fill are inside
because Whitefoot's `buffer_new(count, 0)` allocates and zero-fills, so every
reference calls `calloc`, or `malloc` plus `memset`, inside the interval. A
re-cut that quietly lets the native path allocate uninitialized compares
allocators, not schedulers.

## Removal conditions

Every file here serves one of: a kernel's Whitefoot program, a kernel's oracle,
a reference implementation, the harness, the reducer, the build, or the record.
If a reference is ever judged uninformative, its `backend_*` file goes and the
table loses one row. If a kernel is ever dropped, its `programs/*.wf`,
`*_bench.c` and `*_host.ll` go together. The bundle goes when the question at
the top of this file stops being worth asking. Nothing here outlives its row.
