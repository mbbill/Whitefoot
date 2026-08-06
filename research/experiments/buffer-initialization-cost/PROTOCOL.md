# Buffer initialization cost — preregistration

Run id: `buffer-init-cost-1`. Written before any number was observed.

## Authorization

Task `0016-cost-shape-and-hostile-test-gates`, under the `ACTIVE`
`docs/current-plan.md` Work item 2 ("the §9.1 cost and §12.2 hostile test
gates") and its Verification bullet ("the buffer gates use their two distinct
controls"). This bundle owns exactly one row of the dossier's §9.1 table and
authorizes nothing else. It cannot change the language, the compiler, the
specification, or protected evidence.

## Question

Dossier §9.1, initialization-cost row:

> the one-time fill at allocation is not material to steady-state throughput —
> compare steady-state throughput with buffer reuse against an equivalent
> native read loop over *uninitialized* storage, counting the one-time fill at
> allocation; stop for a separately proved initialization model only on a
> material loss

## The decision this can change

Dossier §11: "If initialized-buffer cost is material, work stops for a
separately proved initialization model before `wfgrep` grows around it." A
material loss here stops the project slice and returns a separately proved
initialization model to the owner as the next decision. It does not authorize
one.

## Why this row needs its own control

§9.1 deliberately gives the two buffer rows different controls. The
*initialized* control answers "does initialization happen once", which is
structural and is gated in
`compiler/src/backend/tests/cost_shape.rs::the_reused_buffers_are_initialized_once_at_allocation`.
It cannot answer this row, because an initialized control carries the same
one-time cost on both sides and cancels it. Only an uninitialized control can
say whether paying it at all is material.

## Hypothesis

The one-time fill of one reused page is immaterial to steady-state throughput:
a Whitefoot drain over an initialized reused buffer measures at practical
parity with the same drain in C over uninitialized storage, and the isolated
per-allocation cost of the fill is small enough that it cannot reach a
material share of any realistic run.

## Subjects and controls

One drain, expressed three ways. All three acquire the initial directory, open
one relative path under it, read into one reused 4096-byte buffer until the
end, touch byte zero of each delivered chunk, and report a witness status.

| Name | Storage | Role |
|---|---|---|
| `whitefoot` | `drain.wf`, `buffer_new<u8>(4096, 0)` — the language initializes it at allocation | subject |
| `uninit` | `control.c drain malloc` | the §9.1 control |
| `init` | `control.c drain calloc` | the same-source twin |

`uninit` and `init` are one C source and one binary differing only in which
allocation call runs, so their ratio isolates the fill from language, compiler,
and toolchain differences. `whitefoot` matches the C drain's work exactly,
including the one-time `SIGPIPE` normalization the command bootstrap performs.

`control.c fill <mode> <n>` measures the allocation alone, with no drain.

## Frozen boundary

- Corpus: 256 MiB from a fixed linear congruential stream, regenerated
  identically by the runner; the expected witness status is computed from the
  same bytes, so agreement between programs is checked against an oracle
  rather than against each other.
- Buffer: 4096 bytes, matching `tests/programs/wfgrep.wf`.
- Timed region: the whole process. §9.1 requires the one-time fill to be
  counted, so it cannot be excluded by an internal kernel timer.
- Toolchain: `/usr/bin/clang -O2` for both the C control and the emitted
  Whitefoot module; `rustc -C opt-level=2` for the runner.
- Compiler revision, sources, and corpus are fixed for the whole run.

## Observables

1. **Primary.** `uninit` elapsed / `whitefoot` elapsed. Above 1.0 means
   Whitefoot is faster.
2. **Secondary.** `uninit` elapsed / `init` elapsed — the same-source ablation.
   Below 1.0 would mean the fill costs steady-state throughput within one
   language and one binary.
3. **Decisive.** Per-allocation nanoseconds for `calloc` minus `malloc` over
   2,000,000 allocations, median of nine invocations. This is the whole
   quantity the row is about, measured directly.

Observable 3 is listed as decisive on purpose, and this is preregistered
rather than chosen after seeing results: a one-time fill of one page is far
below what whole-process timing resolves, so observables 1 and 2 can only fail
to refute the hypothesis. Reporting them alone would leave the §11 stop
condition indeterminate. Observable 3 makes it determinate by measuring the
cost itself and comparing it against the run it would have to be material to.

## Statistics

30 paired rounds. Each of the six execution orders over the three programs
appears exactly five times. Samples are never deleted or extended after a
result is seen. The point statistic is the median of the 30 within-round
ratios; a deterministic 10,000-resample bootstrap over complete rounds with
seed 20260806 reports a descriptive 95% percentile interval.

Interpretation bands, fixed in advance:

- practical parity: the whole interval lies within `[0.98, 1.02]`;
- material Whitefoot loss: the whole interval lies below `0.98`;
- material Whitefoot win: the whole interval lies above `1.02`;
- otherwise inconclusive.

A relative interval half-width above 2% is reported as precision-inconclusive
for this row even if a point estimate crosses a band, because the effect under
test is smaller than that.

For observable 3, the fill is **material** if the measured per-allocation
initialization cost reaches 1% of the elapsed time of the 256 MiB drain. The
report also states the input size at which it would reach 1%, so the claim is
falsifiable at any scale rather than only at the measured one.

## Stop condition

The bundle stops when all three observables are recorded and classified. Then:

- **Immaterial** (hypothesis held): record it, the §9.1 row is discharged, and
  task 0016 closes normally.
- **Material** (any band above shows a material loss, or observable 3 reaches
  1%): the executor stops, does not attempt a fix, and reports a blocker per
  the executor lane in `docs/WORKFLOW.md`. Dossier §11's condition has fired
  and the next decision — a separately proved initialization model — belongs
  to the owner and the lead, not to this task.
- **Inconclusive**: recorded as inconclusive. An inconclusive primary
  observable does not fire the stop condition on its own if observable 3 is
  determinate and immaterial, because observable 3 measures the same quantity
  with far more resolution.

## Threats to validity, stated in advance

- Whole-process timing includes process startup on both sides, which inflates
  both denominators and biases the primary observable toward parity. Stated
  here rather than discovered later; it is why observable 3 exists.
- The corpus is read from the page cache after the first round, so this
  measures the drain and the transfer path, not a storage device.
- `fill` allocates and frees repeatedly, so the allocator recycles one block
  and `calloc` must zero it every time. A single startup allocation may instead
  come from fresh, already-zero pages and cost less. The measured figure is
  therefore an upper bound on the fill `wfgrep` actually pays, which is the
  conservative direction for a stop condition.
- One host, one architecture, one allocator. The result is evidence about this
  target, not a portable law.

## Out of scope

No language change, no compiler change, no optimizer fact, no partial
initialization, no `wfgrep` change, and no comparison against any other
program. Do not add a favourable workload, a larger buffer, or a different
chunk size after seeing a result.
