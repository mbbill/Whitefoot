<!-- Serves compute-bench: the reader's entry point. It states the one question
     the bundle answers, how to run it, how to read the table it prints, what
     each reference's grain policy is and what it is not, the three A/B
     handles and the baseline twin a fourth builds, the call cadence and the
     gap mode, the two flag sets and the asymmetry between them, the one
     compiled-Whitefoot standing the old research bundle left behind, where a
     table that matters is recorded, and the one rule here that decides a
     pass/fail: the compute regression check, a separate pull-request check
     that is not part of `make check`. -->

# compute-bench

One question, one table per host:

> **For each kernel, at each width, is the Whitefoot program built by this
> tree's `whitefootc` with plain `--par` the fastest thing in the row?**

Everything in this directory exists to make that one comparison honest, and
nothing else is here at all. **No number in a table printed by this bundle
fails a build or a check**: there is no band, no threshold, no timeout, no
budget and no heuristic anywhere that selects a row, a ranking or a ratio, and
`compare` fails only on a missing or malformed row.

One rule here does decide a pass/fail, and it is deliberately kept to one
place, one input and one job: [the compute regression
check](#the-compute-regression-check) compares this tree against the tree a
branch started from, using the A/B twin below, and fails a pull request that
made the compiled Whitefoot program slower. It reads the twin lines of one
table and nothing else, it is a separate required check rather than a stage of
the repository's `make check`, and it changes nothing the recorded scoreboard
measures.

## What "WF" means here, and what it does not

The range-loan consumer is selected with `make KERNELS=stencil verify` or
`make KERNELS=stencil compare`. Its runtime-sized grids use distinct positive
finite bit patterns derived from squared positions and retain their initial
boundary values across repeated Jacobi steps. The C oracle uses column-major storage and compares every
binary64 result bit with the row-major implementations. The compiler's native
tests run that same dimension/step matrix in sequential and parallel modes at
one, two, and four workers; `programs-check` also includes `stencil.wf` and the
recursive `range_split.wf` consumer.

The default timed stencil is 1024 by 4096 for 16 steps. Set
`WFB_STENCIL_GRID=original` for the initial 1024 by 2048 comparison, or
`WFB_STENCIL_GRID=small` for the separate 17 by 13, three-step overhead
measurement. The default `large` fixture affords four row chunks under the
shipped split floor; the original fixture affords two after the direct-view
permission correction. The exact dimensions are printed in every process's
workload header. Both WF and references allocate
and zero two grids, initialize them, compute and join every step, and release
the inactive grid inside the timed interval. The returned grid is checked and
released outside it. Native references submit one interior row per callback;
WF uses the compiler's default decomposition. Several nested loops and
initialization have different spans, so the stencil reports no single chunk
count, while actual scheduler grants are still measured.

The default comparison retains the four existing kernels so that its baseline
twin can compile them with the outgoing language version. Stencil requires the
range-loan amendment and has no executable result under that baseline. Its
native comparison and evidence are recorded in
[`range-loans/DESIGN.md`](../../investigations/range-loans/DESIGN.md).

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
  edit. The row is the program the compiler produces or it is nothing. The two
  A/B handles below can build a different compiler control or a different
  runtime constant, and a table taken under either says so in its own header
  and is never recorded as a plain one.

Quadrature's recursive component is cut by the recursion budget the runtime
answers at its entry, and `--par-recursive-frontier off` is the control that
withholds that cut; the three map kernels reach the runtime through a
synthesized splitter, which no budget can touch, so their modules are
byte-identical under every setting of it.

`wf-seq` is the same source compiled `--no-overlap --emit-llvm` and run at width
one. It is the sequential **control**, not a reference: it never enters the
ratio column. It answers "did `--par` buy anything at all", which is a different
question from "is `--par` the fastest".

### The three A/B handles, and why none of them appears in a recorded table

`WF_PAR_CONTROL_FLAGS` is appended to the `--par` emission of the **twin image**
described in the next subsection, and is **empty by
default**. It exists so that one tree can answer "what would this compiler
control be worth here" — two images differing in exactly that flag, built from
one tree, measured in one `compare` run so the reference rows are shared:

```sh
make compare RESULTS=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/results/ab-off \
     WF_PAR_CONTROL_FLAGS='--par-recursive-frontier off'
```

The emitted module depends on this value the way it depends on the source and
on the compiler binary: the Makefile keeps the value in a stamp file the module
rule reads, so setting the variable or clearing it again re-emits, and a
control-flag run can never silently re-time the plain image the last run left.

It is a measurement control and not a grain knob. **A table recorded in
`RESULTS.md` is always taken with it empty**, because the `wf` row of a
recorded table is the program plain `--par` produces and nothing else; a
setting that made a row nicer would be measuring a program no Whitefoot user
gets. Two things keep the two kinds of table apart without anyone having to
remember: `manifest.txt` records `WF_PAR_CONTROL_FLAGS` on every run, empty or
not, and a non-empty setting adds a `WF --par control flags=` line to the table
header saying in the table itself that it is not the plain program. The
bundle's gate target, `programs-check`, never reads the variable.

`WF_RUNTIME_CONTROL_FLAGS` is the same handle on the other side of the link,
under the same discipline. It is appended to the compile of the four Whitefoot
runtime translation units — `floor.o`, `sched_core.o`, `sched_prim_host.o` and
`sched_entry.o` — so one tree can answer "what would this runtime constant be
worth on this block of kernels" without editing the constant per run:

```sh
make compare RESULTS=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/results/unit-300k \
     WF_RUNTIME_CONTROL_FLAGS=-DWF_PAR_SPLIT_WORK_UNIT=300000
```

Not every value it carries is a constant to sweep.
`-DWF_PAR_IDLE_WINDOW_ON_ASYMMETRIC=1` is a twin knob rather than a number. The
runtime withholds the idle window from a machine whose CPUs are not all of one
performance level; this flag withdraws that test and nothing else, so on such a
machine the twin arm is the older behaviour and the `wf` row is the current one,
and `A/B  wf-b/wf  wall` above 1.000 reads the current rule ahead. On a machine
whose CPUs are alike the flag changes nothing and the pair is a null. The Apple
M1 Pro section at `8c296c91` in
[`../../investigations/compute-runtime/RESULTS.md`](../../investigations/compute-runtime/RESULTS.md)
is the table it was built for.

It is **empty by default**, kept in its own stamp file that the twin's four
runtime object rules depend on — so setting it or clearing it recompiles and
relinks the twin rather than re-timing the one the last run left — recorded in
`manifest.txt` on every run, and announced in the table header as a
`WF runtime control flags=` line when it is not empty. **A table recorded in
`RESULTS.md` is always taken with both variables empty**, for the same reason:
the `wf` row of a recorded table is the program plain `--par` produces linked
with the runtime this tree ships. It reaches no reference, no oracle and not
the harness, because none of those is the Whitefoot runtime; the harness asks
the linked runtime for the split at run time instead of carrying a copy of the
rule, so the `note` column's chunk count and the two split verify fixtures
follow the image they describe — under a changed work unit and under a changed
oversubscription cap alike.

`WF_MODULE_CONTROL_FLAGS` is the third handle, and the one that moves how the
Whitefoot side of the twin is **compiled** rather than what it says. It is
appended to the twin's compile of the emitted `--par` module object and of its
four Whitefoot runtime units — both sides of the link that `WF_FLAGS` owns — so
one tree can answer "what would this code generation flag be worth on exactly
the translation units `whitefootc` builds":

```sh
make compare RESULTS=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/results/aligned \
     WF_MODULE_CONTROL_FLAGS='-falign-functions=64 -falign-loops=32'
```

It is **empty by default**, kept in its own stamp file that the twin's module
object and its four runtime objects depend on — so setting it or clearing it
recompiles and relinks the twin rather than re-timing the one the last run left
— recorded in `manifest.txt` on every run, and announced in the table header as
a `WF module control flags=` line when it is not empty. **A table recorded in
`RESULTS.md` is always taken with all three variables empty**, for the same
reason as the two above: the `wf` row of a recorded table is the program plain
`--par` produces, built the way `whitefootc` builds it. It reaches no reference,
no oracle, not the harness and not the shared `--no-overlap` control object,
because none of those is the Whitefoot side of the link.

Its first use is on the record, and it is why the asymmetry below still stands.
The `wf` row is built with no alignment flag while every reference gets
`-falign-loops=32`, so "what would alignment do to the Whitefoot rows" is an A/B
this handle asks directly: it reads all sixteen twin lines inside [0.954, 1.010]
on this bundle's development host — nothing worse than one percent at any width,
and less than the same host's own placement spread in either direction. The
driver was left alone on that reading
([`RESULTS.md`](../../investigations/compute-runtime/RESULTS.md), loop and
function alignment on the Whitefoot side).

### The placement-only arm

Every candidate the twin has measured so far differed in bytes as well as in
behaviour, so a reading could never be attributed to one rather than the other.
`compiler/src/backend/sched/core.c` therefore carries a never-called,
non-inlined 587-byte function under `#ifdef WF_PLACEMENT_PAD`, which nothing
that ships defines and `whitefootc` never passes. Through the runtime handle it
gives the twin an arm that is **identical in behaviour and shifted in
placement**:

```sh
make compare RESULTS=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/results/null-shift \
     PASSES=5 CALLS=5 WF_RUNTIME_CONTROL_FLAGS='-DWF_PLACEMENT_PAD=1'
```

The pad sits ahead of every function in the scheduler core, so the core's own
functions and the two runtime objects the link places after it —
`sched_prim_host-b.o` and `sched_entry-b.o` — move by its size, and nothing else
about either image changes. What it does **not** move is the emitted module:
that object is linked ahead of the four runtime ones, so the kernel's own code
sits at the same offsets in both arms and what the arm shifts is the scheduler
the kernel calls into. Read against `WF_AB=1`, whose arms are byte-identical,
the difference between the two is what that placement alone is worth on the
host. A quadrature row is the cleanest reading of it: that kernel emits no split
call, so its two arms do identical work in every pass whatever the runtime
handle carries. What it read here is in
[`RESULTS.md`](../../investigations/compute-runtime/RESULTS.md), what code
placement alone is worth, with a shifted null arm: a block median moved 11.7
percent and one kernel's five paired readings spanned 0.68 to 1.30 over
identical work, which is the bound on what a single A/B line can select on this
host.

### The A/B twin: what the controls actually build, and how to read it

No control moves the plain image. Setting any of the three — or setting
`WF_AB=1` with none of them — builds a **second image per kernel**,
`$(BUILD)/<kernel>-b`, from the same sources with the controls applied, and
times it in the same passes as
the form **`wf-b`**. `$(BUILD)/<kernel>` is always built with all three
controls empty.

**Why a twin and not a second run.** On a host whose run-to-run spread is wider
than the effect being looked for, two separate `compare` runs cannot select
between two candidates, and this bundle's development host is such a host: six
runs of *byte-identical* images read the quadrature W=4 ratio anywhere from
0.916 to 1.035 — thirteen percent — while records and fir W=4 spread 8.5 and 8.8
percent over the same six. The reducer already solved this for the references.
Within each pass every form runs as its own process in a rotated, alternating
order, and the `wf` row's `ratio` is the median of **within-pass matched pairs**
rather than a quotient of two run medians. The twin puts a second Whitefoot
image into those same passes, so the two arms of an A/B are separated by
minutes of one machine's own schedule instead of by two runs.

```sh
# the oversubscription cap and the work unit, one candidate against the shipped
# runtime, in one set of passes
make compare RESULTS=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/results/cap-256 \
     PASSES=5 CALLS=5 \
     WF_RUNTIME_CONTROL_FLAGS='-DWF_PAR_SPLIT_OVERSUBSCRIBE=256 -DWF_PAR_SPLIT_WORK_UNIT=75000'

# the instrument's own null check: a twin that differs in nothing
make compare RESULTS=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/results/null \
     PASSES=2 WF_AB=1
```

**What is doubled and what is shared.** The twin gets its own emitted module
(`<kernel>-par-b.ll`), its own object from it, its own kernel object with the
weight read out of *that* module (`$(BUILD)/b/<kernel>_split.h`), and its own
four Whitefoot runtime objects (`floor-b.o`, `sched_core-b.o`,
`sched_prim_host-b.o`, `sched_entry-b.o`). It shares `harness.o`, every
`backend_*.o` and the `--no-overlap` control object with the plain image,
because no control reaches those and a second copy of the same bytes under
another name would move code placement, which is this bundle's largest
confound. The two link-time assertions described just below are applied to the
twin exactly as to the plain image: a twin that silently lost the scheduler
would read as a runtime constant worth a great deal.

**How the table reads.** `wf-b` gets a row in every block, with its own
`ratio`, `cpu_r`, `lower`, `steals` and `note`, computed against the best
reference of each pass exactly as `wf`'s are — neither Whitefoot row is ever a
reference for the other, so `wf`'s ratio is the number it would have been with
no twin in the run. Under each block, one line gives the twin verdict directly:

```
mandelbrot   4 A/B  wf-b/wf  wall 0.981 [0.96-1.01]  lower 4/5  cpu 1.002
```

`wall` is the **median of the within-pass paired ratios** of `wf-b` to `wf` with
its `[min-max]`, `lower` is how many of those pairs had the twin faster, and
`cpu` is the same pairing over process CPU. It is printed at every width the
block carries, width one included, because "the twin changed nothing at width
one" is a thing an A/B run has to be able to say. `BEST REFERENCE`, `FASTEST`
and `WF fastest` skip the twin: they answer whether the program *this tree*
produces is the fastest thing in its row, and a twin built from a control flag
is not a thing this tree produces.

**A recorded table never contains a `wf-b` row.** The twin is an instrument, not
a result: the tables copied into `RESULTS.md` as the record of what this tree
produces are taken with all three controls empty and `WF_AB` unset, which builds no
twin, runs no extra process and leaves the bundle exactly as it was. An A/B run
is recorded as the *arms of an experiment*, with the control flags in its
heading and the `A/B` lines quoted in its reading. A push to the hosted
workflow sets none of the three and therefore builds no twin; a manual dispatch
of `.github/workflows/compute-bench.yml` takes the three as inputs, beside the
call cadence of the section below, which is how an A/B pair is read on a quiet
runner (its table is an experiment, never a recorded plain one). The two
targets the repository's `make check` runs — `programs-check` and
`verdict-test` — read none of these handles nor the cadence: one compiles
programs and the other feeds the regression rule crafted table fragments.

**A fourth handle hands the same twin a different tree.** `WF_B_SCHED_DIR`,
`WF_B_FLOOR` and `WF_B_WFC` name where the twin's Whitefoot runtime and its
emitting compiler come from, each defaulting to this tree's own. Pointed at a
worktree of the merge base they make `wf-b` the program the branch started
from, which is the whole of [the compute regression
check](#the-compute-regression-check). Nothing about the twin's rules, its link
line or its assertions changes; only its inputs do.

Two link-time assertions make a `wf` row a `wf` row. The emitted module carries
**weak no-op stubs for every `wf__par_*` symbol**, so a link that loses the
scheduler sources would still link, still run, still produce correct output, and
be silently sequential. The harness therefore references `wf__par_grants()`
and `wf__sched_split_work()`, neither of which has a weak stub, and the
Makefile requires a strong definition of `wf__par_publish`,
`wf__par_split_budget` and `wf__par_recursion_budget` in every image after the
link — the twin's image included. `wf__par_split_budget`, which the harness
also calls for the `note` column's chunk count, *does* have a weak stub, so it
is that post-link strong-binding assertion and not the reference that keeps the
count from being a stub's zero.

## The compute regression check

The twin above answers "what would this flag be worth here" from one tree. Hand
it a different tree and it answers the question a regression check asks:

> **Is the Whitefoot program this branch produces slower than the one the
> branch started from?**

That is `.github/workflows/compute-regression.yml`, a **required** check on
pull requests. It runs on every one of them and carries no `paths:` filter,
which is a consequence of being required rather than a change of mind about
what is worth measuring: a workflow filtered on paths does not run at all on a
pull request touching none of them, so it never reports, and a required check
that never reports leaves that pull request waiting forever on a run nobody
will start.

The path decision moved inside the job instead. Its first step, after a
full-history checkout, diffs the merge base with the base branch against `HEAD`
and matches the result against the set that can move the two arms apart — the
scheduler runtime, the floor, the emitter, the lowering, the driver, this
bundle, and the workflow file itself. A pull request that touches none of them
prints one line saying the measurement was skipped, and the job ends green in
seconds: no dependencies, no cache, no build. A pull request that touches one
of them is measured exactly as before. Either way it is one job and one check
name, and it reports. A manual `workflow_dispatch` always measures.

It is **not** a stage of the repository's `make check` and never will be:
`make check` is the merge gate and has to be a property of the tree rather
than of a runner's load. The rule's own unit test is in `make check` —
`verdict-test.sh`, through the root `research-tests` stage — so the logic that
fails a pull request is itself checked on every gate run, on a host that
measures nothing.

### What it builds

Three variables point the twin's Whitefoot side at another checkout. Each
defaults to this tree's own, so a bundle that sets none of them is the bundle
that was here before:

| variable | default | what it names |
|---|---|---|
| `WF_B_SCHED_DIR` | `../../../compiler/src/backend/sched` | the twin's `core.c`, `prim_host.c`, `entry.c` and their headers |
| `WF_B_FLOOR` | `../../../compiler/src/backend/wf_floor.c` | the twin's floor translation unit |
| `WF_B_WFC` | `../../../compiler/target/gate/whitefootc` | the compiler that emits the twin's `--par` module |
| `WF_B_SOURCE` | `this tree` | a label for `manifest.txt`; the merge-base revision on a gate run |

Naming any of the first three is by itself a request for the twin: a regression
run sets no control flag and still gets its second image. **This is one build
path, not a second one** — the baseline twin is the twin the three control
flags already built, handed different inputs, through the same rules, the same
link line and the same three post-link assertions. A `.wf` source and the
harness always come from this tree; only the runtime and the emitter move.

The workflow exports the merge-base with the pull request's base branch as a
git worktree under `$RUNNER_TEMP` — never a path inside the repository — builds
that revision's `whitefootc` beside it, and sets the four variables at it. The
merge base rather than the base branch's tip, because a branch is answerable
for what it changed and not for what landed on `main` while it was open.

`manifest.txt` records `WF_B_SOURCE`, the three paths and the SHA-256 of every
baseline source the twin was built from, and the table header carries a
`WF A/B twin source=` line, so a table from a regression run says in its own
first lines that its `wf-b` rows are another tree's program. The same hashes
are what make a changed baseline rebuild the twin: they are kept in a stamp the
twin's module and its four runtime objects depend on, so a moved merge base
re-emits and recompiles the `-b` side and nothing else, while the same bytes
under another path rebuild nothing.

### The rule

```
FAIL when a block's  A/B  wf-b/wf  wall  median is below 0.97
     and the baseline was the faster arm in at least 4 of the 5 pairs,
     at W in {1, 2, 4}, oversubscribed blocks excluded.
```

Both halves are required. A median can sit under the band on one adverse pair,
and a count of adverse pairs says nothing about their size; the twin exists
because this host class cannot resolve a single reading.

**CPU is a report and never a failure.** A paired `cpu` ratio below 0.90 under
the same count is printed under the table, marked `*` on its row, and changes
no exit status. It borrows the wall `lower` count because the reducer prints
only that one; a CPU signal read against a wall-pair count is worth looking at
and is not worth failing a branch over.

**What it excludes, and why.**

- **W=8 and above.** The hosted Linux runners have four CPUs, so those blocks
  are oversubscribed, and oversubscription rewards schedulers that yield —
  it changes which implementation wins rather than measuring one. The reducer
  already refuses to name a winner there, and the rule skips any block it
  marked oversubscribed even at a recorded width, which is what a two-CPU
  runner's `W=4` would be.
- **macOS.** That runner cannot resolve below about twenty percent, which is
  wider than anything this rule is looking for. The check runs on
  `ubuntu-24.04` alone.
- **Runner heterogeneity.** The `ubuntu-24.04` pool is several machine
  classes, so a comparison across two jobs measures the pool. Everything the
  rule reads is a within-run, within-pass pairing of two processes on one
  machine, which resolves about one percent at W=2 and W=4.

**It refuses rather than passing vacuously.** A table with no `A/B` line means
no twin was built; a table whose only `A/B` lines are at unrecorded widths
means nothing the rule reads; a line backed by fewer than five pairs is not the
rule. Each of those exits non-zero with `REFUSED`, because a gate that reads
absent evidence as good news is a gate that switches itself off.

There is **no automatic re-run**. A re-run is a person's decision: a job that
retried until it agreed would be selecting its own result.

### Running it locally

```sh
export WHITEFOOT_SCRATCH_ROOT=${TMPDIR:-/tmp}/whitefoot
base=$(git merge-base origin/main HEAD)
git worktree add --detach "$WHITEFOOT_SCRATCH_ROOT/baseline" "$base"
cargo build --manifest-path "$WHITEFOOT_SCRATCH_ROOT/baseline/compiler/Cargo.toml" \
    --profile gate --bin whitefootc --locked --offline

cd research/experiments/compute-bench
export WF_B_SCHED_DIR="$WHITEFOOT_SCRATCH_ROOT/baseline/compiler/src/backend/sched"
export WF_B_FLOOR="$WHITEFOOT_SCRATCH_ROOT/baseline/compiler/src/backend/wf_floor.c"
export WF_B_WFC="$WHITEFOOT_SCRATCH_ROOT/baseline/compiler/target/gate/whitefootc"
export WF_B_SOURCE="$base"
make build && make verify
make compare PASSES=5 CALLS=5 RESULTS="$WHITEFOOT_SCRATCH_ROOT/regression"
make verdict RESULTS="$WHITEFOOT_SCRATCH_ROOT/regression"
```

`verdict` is a separate make invocation, so it has to be told which run to
read: `RESULTS` is a fresh timestamped directory per invocation, and the
default table it would otherwise look for is one that was never written. Pass
the `compare` run's own directory, or a table directly with
`VERDICT_TABLE=<path>/table.txt`. The bands are variables too —
`VERDICT_WIDTHS`, `VERDICT_WALL`, `VERDICT_CPU`, `VERDICT_LOWER`,
`VERDICT_PAIRS` — so a local reading can be taken at another width or another
band; the workflow sets none of them and gets the rule above.

A local run on a quiet machine is worth more than a hosted one and is the right
place to check a suspected regression by hand. A hosted verdict is evidence
about a hosted runner, and the `raw.tsv` behind it is uploaded with every run.

## Running it

Prerequisites: `clang`, `clang++`, `cmake`, a stable Rust toolchain, and one
network-enabled run of `make deps` (its `fetch` step is the only step in the
bundle that touches the network; everything after it is offline).

```sh
cd research/experiments/compute-bench
export WHITEFOOT_SCRATCH_ROOT=${TMPDIR:-/tmp}/whitefoot   # the default; nothing is
                                                          # ever written in the repo
make deps      # fetch (network) and build the pinned schedulers
make build     # the compiler, both modules per kernel, one image per kernel
make verify    # every form at every emitted width, correctness only
make compare   # the timed passes and the table, into a fresh RESULTS
```

The default scratch root is the system temporary directory, which the host may
clear on reboot: the fetched and built dependencies and any `RESULTS` left
there are then gone and have to be rebuilt, so copy out anything worth keeping
(the tables that matter are transcribed into `RESULTS.md` by hand anyway), or
set `WHITEFOOT_SCRATCH_ROOT` to a durable directory outside the repository.

`make deps`, `make build` and `make verify` are the only targets that can fail
on anything other than a missing or malformed row.

Each kernel image also answers directly:

```sh
BUILD=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/build
$BUILD/mandelbrot list                     # forms, grain policies, widths
WF_WORKERS=4 $BUILD/mandelbrot verify wf 4
WF_WORKERS=4 $BUILD/mandelbrot time wf 4 0 5
WF_WORKERS=4 WFB_GAP_US=2000 $BUILD/mandelbrot time wf 4 0 5   # a 2 ms gap
```

`WF_WORKERS` must be set and must equal the `WIDTH` argument for every form,
native ones included; the driver cross-checks the two before anything else
happens, because a disagreement would silently compare two different widths.
`WFB_GAP_US` is the other variable the table sets and is optional: unset is the
back-to-back cadence, and "The gap between calls" below is what a value does.

`make programs-check` and `make verdict-test` are the two targets the
repository's `make check` runs, and neither times anything. `programs-check`
compiles each program in exactly the two modes the table uses and asserts that
`--par` emits a publish site and `--no-overlap` emits none; it takes about two
seconds, links nothing, and needs no dependency. `verdict-test` feeds the
regression rule crafted table fragments and asserts its verdicts; it needs no
compiler at all.

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

`manifest.txt` records `uname -a`, the online CPU count, the CPU topology — one
`topology: cpu<N> core=<id> package=<id> siblings=<list>` line per online CPU on
Linux, `topology: physicalcpu=<n> logicalcpu=<n>` on macOS, and `unqualified`
where neither source is readable, beside an `smt:` line read from
`/sys/devices/system/cpu/smt/active` — the **inherited** CPU
mask and cgroup state (recorded, never narrowed, and marked `unqualified` when
a file is absent rather than reported as "no limit"), the compiler revision,
every toolchain version — the whole of `rustc -vV`, host triple, commit and
LLVM version included, one `rustc: ` line each — the exact flag strings, all
three A/B control variables whether they were set or empty, what the twin was
built from — `WF_B_SOURCE` on every run, and on a baseline run the three paths
and one `WF_B_SOURCE_SHA256` line per baseline source — the
three dependency pins, `BENCH_ARCH`, and the SHA-256 of every kernel image
and every emitted `.ll` before and after the run. If any of those hashes moved
during the run the table is not a measurement of one build and `compare` says
so and fails.

The topology and `smt:` lines are there because hosted runs land on
`ubuntu-24.04` runners of more than one machine class, and four online CPUs may
be four cores or two cores with SMT siblings. Without them a recorded table
cannot be classified by runner, and the lane-placement question — whether
Whitefoot's two lanes landed on the SMT siblings of one core, which is what a
W=2 row spending two lanes' CPU on one lane's work would look like — cannot be
asked of a run after the fact.

### Reproduce recipe

```sh
export WHITEFOOT_SCRATCH_ROOT=${TMPDIR:-/tmp}/whitefoot
make -C research/experiments/compute-bench deps     # once, with a network
make -C research/experiments/compute-bench build
make -C research/experiments/compute-bench verify
make -C research/experiments/compute-bench compare PASSES=5 CALLS=5
```

The same four commands run in `.github/workflows/compute-bench.yml` on
`ubuntu-24.04` and `macos-14`.

## The gap between calls

Five calls per process, back to back, with microseconds between them: that is
the cadence every recorded table is taken at, and it is the cadence a runtime
that keeps an idle lane hot for a window before it parks is measured at its
best in, because the next call always arrives while the lanes are still hot.
Real programs have gaps between their parallel regions.

`WFB_GAP_US` is that gap, in microseconds, per process. It is **zero by
default**, which is the back-to-back cadence and exactly what the bundle did
before the mode existed.

```sh
make compare RESULTS=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/results/gap-2ms \
     PASSES=5 CALLS=5 WFB_GAP_US=2000
```

Between two consecutive timed calls the driver waits that many microseconds
**outside the measured interval**, on a monotonic-clock busy-wait and not a
sleep: the point of the mode is that the calling thread stays running, as a
program doing its own sequential work between parallel regions does, while the
runtime's helper lanes go idle and park. A driver that slept would hand its CPU
back and measure something else. The wait is the last thing before the clock
starts — after the previous call's verification and its printf — so no part of
a gap is inside any reported wall or CPU figure, and the whole of what the gap
did to a call is in that call's own numbers.

**It reaches every form identically, references included.** A gap that reached
only the `wf` row would compare one scheduler's idle policy against another
scheduler's warm one, which is not a comparison; what a gapped table reads is
how the compiled program and each reference alike behave when their work
arrives sparsely.

It is a **run-time setting and nothing else**: no stamp, no emission, no
compile and no link reads it, on the plain image or on the twin, so changing it
rebuilds nothing and the images a gapped run times are byte-identical to the
ones a gap-free run times. That is what lets it compose with the A/B twin, and
the composition is the run the mode was added for:

```sh
make compare RESULTS=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/results/gap-2ms-nowindow \
     PASSES=5 CALLS=5 \
     WFB_GAP_US=2000 WF_RUNTIME_CONTROL_FLAGS=-DWF_PAR_IDLE_WINDOW_US=0
```

— the runtime this tree ships against the same runtime with its idle window
withheld, both at a 2 ms gap, in one set of passes, paired within each pass by
the `A/B wf-b/wf` line. That is the sparse-cadence reading
[`RESULTS.md`](../../investigations/compute-runtime/RESULTS.md) records as open
beside the idle window, and the question the bundle could not ask before: how
much of a win measured on back-to-back calls survives a gap, and what the
references do across the same gap.

**A table taken at a non-zero gap is never a candidate record for the W
blocks**, on exactly the terms of the three control flags above: the recorded
table for a host is its highest non-oversubscribed block at the back-to-back
cadence, and a table at another cadence measures a different question about the
same programs. Three things keep the two apart without anyone having to
remember. `manifest.txt` records `WFB_GAP_US` on every run, empty or not. Every
process writes the gap it ran at into its own header and trailer in `raw.tsv`,
and the reducer **refuses to put two cadences in one table** — a disagreement
is a malformed stream, refused exactly as a header/trailer mismatch is, and
never a judgement about a measurement. And a non-zero gap puts `gap_us=` on the
table's `passes=` line with a disclosure line above it, saying in the table
itself that it is not the back-to-back one.

`verify` is handed the variable too and **waits nothing**. It drives no timed
call loop: the whole fixture sweep is one call into the kernel's own `verify`,
which no clock brackets and no table reports, so the only place a wait could go
is inside a grid that measures nothing, where it would lengthen `verify` and
check nothing. What `verify` does with it is read it, report it in its
`VERIFY PASS` line, and refuse a malformed value cheaply, before a long
`compare` pays for the same mistake. `programs-check` reads it no more than it
reads the three controls: it links nothing and runs no image.

A hosted run takes the gap as the `gap_us` input of
`.github/workflows/compute-bench.yml`, beside the three control inputs, and a
push sets none of the four.

## The lane trace

A gapped run says the compiled form gives up wall time at a sparse cadence on
some hosted runners; it does not say where that time goes, and a table cannot.
`WF_PAR_TRACE` is the runtime's own answer: defined,
`compiler/src/backend/sched/core.c` records a fixed per-lane ring of scheduler
events and prints it to **stderr** at process exit, one line per event. What it
found is in [`RESULTS.md`](../../investigations/compute-runtime/RESULTS.md),
the three `bench/wake-trace` sections.

```sh
make compare RESULTS=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/results/gap-2ms-trace \
     PASSES=5 CALLS=5 \
     WFB_GAP_US=2000 WF_RUNTIME_CONTROL_FLAGS=-DWF_PAR_TRACE=1
```

It is a runtime control like any other, so it reaches the **twin only**: the
`wf-b` cells trace and the `wf` cells are the runtime this tree ships, and the
`A/B wf-b/wf` line of the same table is what says whether carrying the
instrument moved the reading. Nothing else changes. The events go to stderr,
which `compare` already keeps per cell in
`results/logs/<kernel>-wf-b-w<W>-p<pass>.log`, so the parsed stdout stream and
`raw.tsv` are untouched and the workflow uploads the trace with the rest of
`logs/`.

```
wf-trace lane=<n> ev=<kind> t_us=<monotonic us> cpu=<sched_getcpu> v=<payload> seq=<n>
```

| kind | when | `v` | `seq` |
| --- | --- | --- | --- |
| `call_head` | the splitter's entry query, once per top-level call | the idle mask then | 0 |
| `root_publish_first` | the first publish of a burst that finds lanes parked | the idle mask | 0 |
| `park` | entering the condvar wait, in either idle loop | spin/yield rounds done | 0 |
| `wake` | returning from that wait | the publish epoch seen | 0 |
| `steal_ok` | the lane's first successful steal after a wake or a call head | 0 | 0 |
| `chunk` | one executed callback; the timestamp is when it finished | its duration in ns | its index in the call |
| `probe` | a fixed dependent chain of 20,000 steps, timed | its duration in ns | 0 straight out of a park and at a call head, else the chunk it followed |
| `root_join_done` | the release that closes the call on the offering lane | 0 | 0 |

Per call and per lane that gives wake latency (`wake` minus `call_head`), steal
latency, how long the lane had been parked, the CPU it parked on against the
CPU it woke on and the offering lane's own, the work it did as a time series of
chunk durations against their position in the call, and — from `probe` — how
fast the core it is on was running at each of those points. Lines are grouped
by lane and ordered within a lane; sort by `t_us` to interleave them. A final
`wf-trace-probe sink=<n> iterations=<n>` line carries the chain's accumulated
result, so its work is visibly consumed.

The probe rides on chunks 1, 2, 4, 8, … so its cost grows with the logarithm of
the chunk count; on the hosted runners it added **3 to 4 %** to a `wf-b` cell at
two and four lanes. It also lengthens every latency measured from the call
head: the offering lane runs one probe before it publishes anything, and each
helper runs one before its first scan, so head-to-`wake` reads about 39 µs where
a build without the probe reads 21, and head-to-`steal_ok` 57 where it reads 22.
**Read wake and steal latency off a run built without the probe.** `chunk`
events are capped at 64 per lane per call, above what any map kernel here
reaches; a recursive kernel that runs a thousand leaves a call is truncated at
that count rather than filling the ring, and shows it by its chunk events
stopping while its park, wake and join events continue.

It is a **measurement instrument**. No shipped build, gate target or test
defines it, `whitefootc` never passes it, and with it undefined the scheduler
core and the host primitives compile to the same object bytes they did before
the instrument existed.

Pairing it with a wider idle window asks whether any of it is the parking:

```sh
make compare RESULTS=$WHITEFOOT_SCRATCH_ROOT/whitefoot-compute-bench/results/gap-2ms-trace-hot \
     PASSES=5 CALLS=5 WFB_GAP_US=2000 \
     WF_RUNTIME_CONTROL_FLAGS="-DWF_PAR_TRACE=1 -DWF_PAR_IDLE_WINDOW_US=3000"
```

— the same instrument over lanes that stay hot across the gap, so the two runs
differ in whether a lane parked and in nothing else.

## How to read the table

```
kernel       w form              median_us  mad%  p10..p90_us  cpu_us  ratio  cpu_r  lower  steals  note
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
- **`cpu_us`** is the same median of medians over **process CPU time** rather
  than wall, read around the same interval the wall clock brackets, so it counts
  every thread the form started, spinning and parked ones included. The source
  is chosen per host and the driver line of `raw.tsv` names the one that was
  read as `cpu_clock=`: `CLOCK_PROCESS_CPUTIME_ID` on Linux, **`task_info` on
  Darwin**, `getrusage(RUSAGE_SELF)` as the fallback anywhere the chosen source
  is absent or refuses. Darwin is not on the POSIX clock because it answers that
  clock from the task's accounting for threads that have already exited: a pool
  whose workers are still alive contributes nothing, so the column read about
  one lane's worth however many lanes ran, and the M1 Pro tables in `RESULTS.md`
  recorded `tbb` spending 5,896 us of CPU for a 2,441 us wall on eight threads
  beside a four-lane `static` row spending 2,904 us against a 2,866 us wall. The
  Darwin source is therefore the task-level pair that does consult the live
  threads when asked: `task_info(TASK_THREAD_TIMES_INFO)` for the threads that
  still exist plus `task_info(TASK_BASIC_INFO)` for the ones that have exited,
  each `time_value_t` seconds and microseconds, summed. That source is held on a
  reading and not on its documentation: the hosted `macos-14` leg of run
  34668036736, a three-CPU runner, printed `cpu_clock=task_info` on every driver
  line and returned CPU that grows with the lanes and stops where the CPUs do —
  mandelbrot `static` at W=4 read **107,878 us of CPU against a 36,495 us wall**
  and `tbb` at W=4 **26,072 against 8,887**, 2.96 and 2.93 times their own walls
  on three CPUs, while the four W=1 serial rows sat inside half a percent of
  their own wall. `proc_pid_rusage` was tried between the two and rejected on
  the same kind of evidence: on the hosted `macos-14` runner of run 34667394566
  its figures read 0.02 to 0.07 times their own wall and hardly moved with the
  work, which is not a CPU figure and is not a unit error either. The wall clock
  stays the outermost pair and the two CPU reads are nested inside it, so no CPU
  a call spends can fall outside the wall interval; the nested pair was timed at
  757 ns on the Linux host where that was measured (`RESULTS.md`, 2026-09-11),
  against per-call intervals of milliseconds. A form whose wall time is bought
  by burning four lanes is indistinguishable from one that is simply fast in
  `median_us` and is not in `cpu_us`. It is a measurement, never a pass/fail
  input.
- **`ratio`** is filled only on `wf` rows. It is the **median of within-pass
  matched pairs**: for each pass, WF's process median divided by the lowest
  process median among the parallel references at that same width, printed with
  its `[min-max]` and, on the verdict line, the modal name of the reference that
  supplied the minimum. **Recomputing a ratio from two printed medians does not
  reproduce this number**, and it is not meant to. `serial` and `wf-seq` live in
  the width-one block and never enter the ratio: a parallel row beating a serial
  row answers nothing.
- **`cpu_r`** is the CPU ratio, paired exactly as `ratio` is: within each pass,
  WF's process CPU median divided by the CPU median of **the reference that was
  fastest by wall in that pass**, and the median of those pairs. The two ratios
  are therefore about the same pairs and can be read side by side --- `ratio`
  says whether WF finished first, `cpu_r` says what it spent to. Pairing CPU
  against whichever reference happened to burn least CPU would answer a
  different question and would not line up with the verdict line. It sits beside
  `ratio` as a column rather than in `note` because it is the number the
  process-CPU target is read off, and a reader comparing it with the wall ratio
  should not have to cross the row to do it; the two extra columns take a data
  row to at most 141 characters -- the widest a run has printed, the `static`
  row carrying its own mark -- well inside the width the legend below was
  introduced to protect.
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

The width set is `{1, 2, 4, 8, 16, 32}`. The harness reads the online CPU count
once per process and emits every width at most that count, plus the smallest
width above it when there is one; 16 and 32 exist for hosts with that many
logical CPUs (a 32-thread desktop records `W=32`) and change nothing on a
smaller host, which still emits its own widths and one oversubscribed block. A
width above the count is still emitted, but its header carries
`oversubscribed=1` and the reducer prints `n/a (oversubscribed)` instead
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

Each kernel has exactly one size constant. It is chosen so the Whitefoot loop
reaches a chunk count worth timing at the highest width the table records,
subject to two bounds: the `wf-seq` median at width one lands in **[5 ms,
60 ms]**, and the `wf` median at the highest recorded width stays **above
1 ms**, so fork, join and dispatch cannot dominate the thing being compared. It
is never chosen from the minimum admission threshold: a loop that is merely
admitted may still get two chunks. Which of the two terms in the rule below
actually sets a count depends on the work unit the runtime ships; at the
current one it is mostly the cap, as the note under the table says.

The compiler splits an independent map into `2^budget` chunks with

```
chunks = 2^floor(log2(min(16 * lanes, span / ceil(150,000 / weight))))
```

so the oversubscription term scales with the width, and the result is rounded
**down** to a power of two — 143 affordable chunks are 128 chunks.

The `affordable` column below is the work term alone, `span / ceil(150,000 /
weight)`; the per-width columns are the whole formula.

| kernel | shipped size | emitted weight | affordable | W=2 | W=4 | W=8 | W=16 | W=32 | reference chunks |
|---|---|---|---|---|---|---|---|---|---|
| mandelbrot | 98,304 points, limit 256, shape `trailing` | 219 | 143 | 32 | 64 | 128 | 128 | 128 | 1,536 at grain 64 |
| records | 131,072 records, `max_length` 255, shape `unicode` | 812 | 708 | 32 | 64 | 128 | 256 | 512 | 8,192 at grain 16 |
| fir | K=64 taps, N=524,288 outputs | 150 | 524 | 32 | 64 | 128 | 256 | 512 | 2,048 at grain 256 |
| quadrature | M=64 integrations, tol `0x1p-54`, depth 24 | none | n/a | n/a | n/a | n/a | n/a | n/a | frontier depth 8 |

**Which term binds, and therefore what a size constant still controls.** At the
shipped work unit of 150,000 the oversubscription cap `16 * lanes` is the
binding term for all three maps at W=2, W=4 and W=8, and for records and fir at
W=16 and W=32 as well; only mandelbrot returns to the work term, from W=16 up,
where its 143 affordable chunks sit under the cap and hold the count at 128. So
at the widths a four- or eight-CPU host records, these counts follow the lane
count rather than the size: a size step moves one only where it drops the
affordable count below the cap. The size constants therefore stand on the two
window bounds alone, and they are unchanged.

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
has no minimum-size admission threshold. What it asks the runtime instead is
`wf__par_recursion_budget()`, once per call into its recursive component, for
the levels below which that component runs its sequential clone.

**Confirm the window on the host that records a table**, before recording it:
the `wf-seq` median inside [5 ms, 60 ms], the `wf` median above 1 ms at the
recorded width, and `steals > 0` on the `wf` row at every parallel width. If a
bound fails, change that one size constant by the smallest power-of-two step
that fixes it, re-derive the chunk count from the formula — the affordable
column and every width column — and record the new counts here and in the
table header. A step may leave the recorded counts unchanged, because the cap
binds below W=16; record what the formula gives either way. Nothing else
moves.

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
WHITEFOOT_SCRATCH_ROOT ?= $(patsubst %/,%,$(if $(TMPDIR),$(TMPDIR),/tmp))/whitefoot
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
WFB_GAP_US ?=
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
interior points come last. Granularity, not the dispatcher, decides this row,
and which term sets the granularity has changed. Under the work unit of
1,200,000 this fixture could afford only 17 chunks, so `--par` emitted 16 at
every width — on `trailing` that is four heavy chunks, and at W=8 a ceiling no
lane count could lift; an eight-CPU host's W=8 wall equalled its W=4 wall
there, which is why the work unit moved to 150,000 (the scheduler's own note
beside `WF_PAR_SPLIT_WORK_UNIT`). At 150,000 the same 98,304 points afford 143,
so the count follows the lanes: 32 chunks at W=2, 64 at W=4, 128 at W=8, and
128 again at W=16 and W=32, where the work term binds. The size constant did
not move; it stands on the [5 ms, 60 ms] `wf-seq` window. **At W=8 the heavy
interior quarter lands in 32 of those 128 chunks, four per lane, which is a
property of the fixture**, and this block says so rather than letting a reader
take a chunk count for a scheduling result. The `static` row here is a strong
**regular-work** reference and not a dynamic scheduling ceiling for skew — which is exactly why the skewed
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
`2 * ceil(150,000 / weight)` iterations upward, and the smaller fixtures in
Mandelbrot's and FIR's grids sit below that floor at every width (at the
earlier work unit of 1,200,000 every grid fixture did). Without a fixture the
floor is known to admit, the `--par` module could take its unsplit path through
the whole of `verify`, and the chunked path the table times would never be the
path the oracle checks. So each of those two kernels carries **one extra
fixture** sized from the floor its own emitted module implies: nothing is
stored, a change of weight moves the fixture rather than quietly dropping it
back below the floor, and `verify` prints the weight, the floor, the size and
the chunk count the run produced —

```
# mandelbrot split fixture: weight=219 floor=1370 points=1408 chunks=2
# fir split fixture: weight=150 floor=2000 outputs=2048 chunks=2
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
a reference implementation, the harness, the reducer, the build, the regression
rule, or the record. If a reference is ever judged uninformative, its
`backend_*` file goes and the table loses one row. If a kernel is ever dropped,
its `programs/*.wf`, `*_bench.c` and `*_host.ll` go together. `verdict.awk` and
`verdict-test.sh` go together with `.github/workflows/compute-regression.yml`,
whenever that check stops being worth its runner minutes; the `WF_B_*` variables
go with them, because nothing else builds a twin from another tree. The bundle
goes when the question at the top of this file stops being worth asking.
Nothing here outlives its row.
