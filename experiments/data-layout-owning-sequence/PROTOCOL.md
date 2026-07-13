# E0.1 proposed protocol

Status: research draft for owner review, 2026-07-13.  It is not preregistered.
Isolated correctness/code-shape/smoke experiments are authorized; production
implementation and scored timing are not.

This protocol is intentionally stricter than a normal feature benchmark.  E0.1
may add an explicit capability only if it is zero-tax for programs that do not
use it.  It may change xlc's taught or source-level default only if the default
shape is also non-regressing on the complete compiler workload.  A capability
win is not evidence for a default migration.

No candidate may be implemented as a feature flag.  The baseline and candidate
are separate, single-semantics toolchains built from an immutable baseline
revision and a disposable candidate branch/worktree.  Experimental language or
compiler semantics do not enter the production toolchain; reproducibility
infrastructure may live under `experiments/`.  A production implementation
starts only after explicit owner confirmation following report review, and then
lands atomically across every normative surface.

## 1. Questions and exclusions

E0.1 is split into two serial decisions.

### E0.1a — flat records in fixed storage

Test a fixed-capacity AoS representation using a compiler-verified `Flat(T)`
record.  `Flat` means fixed-size, region-free, borrow-free, and without a drop
obligation.  It does not mean that every bit pattern is valid, that padding is
observable, that bytewise equality is valid, or that the type has a stable FFI
ABI.

`Flat` is deliberately separate from the existing implicit-Copy class.  Bare
uses of an aggregate must not acquire hidden copying, and `Flat` alone must not
license even an explicit contraction of one affine record into N records.  In
particular, `buffer_new<Record>(n, move seed)` is rejected: spelling the move
does not make duplication of its result legal.  A possible first initializer is
a recursively fresh row recipe containing only Copy leaves and no `move`, whose
semantics constructs one fresh row per slot; that recipe is still an unresolved
design item, not an approved rule.  Field projection remains scalar.  Existing
`struct`, primitive `buffer<T>`, and the current SoA tapes keep their semantics
and representation.

### E0.1b — initialized-prefix owning sequence

Only after E0.1a receives an adopt/reject/defer decision, test an opaque affine
sequence with `{data, len, capacity}` semantics.  Exactly `[0, len)` is
initialized; `[len, capacity)` is inaccessible and is not dropped.  The writer
never receives raw or `MaybeUninit` storage.

Growth and no-growth append are distinct operations.  The measured hot shape is
`with_capacity`/`reserve` followed by `push_within_capacity`; proof of
`len < capacity` must reduce a push to an element store plus a length increment,
with no allocator slow path.  An auto-growing library wrapper is not part of the
first kernel decision.

The first sequence prototype admits only `Flat` elements.  Arbitrary affine
elements are deferred until exact-once drop elaboration (STOR-3), element
reborrows, and moved-storage raw deallocation are complete.  General partial
initialization, public `MaybeUninit`, hole-producing `take(index)`, user-defined
drop, pinning, and automatic SoA/AoS conversion are out of scope.

## 2. Frozen baseline and changed surface

The source baseline is Git commit
`58baa71fb4c36a4728dd42aea6b05ce4be7aa0b1`.  The canonical compiler unit is
constructed exactly like `compiler/test_lexer.py::compiler_source`:

- byte length: `1,029,044`;
- SHA-256: `17c28914ec3cd109f0411cc8a83423623c1541be239e753e91144a66bea93f65`;
- observed output: `211,374` tokens and `105,550` AST nodes.

The scored implementation manifest must also record the source tree hash, every
variant diff hash, stage-0 hash, generated LLVM hash, compiler/clang version,
target triple, OS, machine, allocator, flags, and corpus hashes.  Dirty or
unrecorded inputs invalidate a scored run.

Only TokenTape and AstTape may differ between layout arms.  The validation,
symbol, type, and fact tapes stay source- and representation-identical.  Parser
algorithms, error priority, capacity policy, facts setting, allocator, target,
and optimization flags may not be opportunistically changed in a layout arm.

The production lane is stage-0 facts off and `/usr/bin/clang -O2`, matching the
existing compiler tests.  A secondary facts-on `-O3` generic-target lane is
reported separately.  Native-CPU tuning, LTO, PGO, and per-arm flags are
forbidden.

## 3. Arms and attribution controls

Each arm is a distinct native executable and fresh process.

| ID | Layout | Storage/initialization | Purpose |
|---|---|---|---|
| `F-SOA` | current SoA | fixed `source_len + 1`, full eager fill | sole production baseline |
| `F-SOA-P` | column segments in one tape allocation | same fixed capacity and fill | coallocation/header diagnostic composite only |
| `F-AOS` | flat record AoS | same fixed capacity and semantic initialization | fixed/full-init total representation effect against `F-SOA` |
| `R-SOA` | one logical SoA sequence owner | `reserve_exact(source_len + 1)`, initialized prefix | reserve-exact SoA policy total effect |
| `R-AOS` | record sequence | identical element-capacity reserve policy | reserve-exact AoS policy total effect |
| `D-SOA` | one logical SoA sequence owner | empty plus frozen doubling/minimum-capacity policy | doubling SoA policy total effect |
| `D-AOS` | record sequence | identical element-capacity growth policy | doubling AoS policy total effect |

`F-SOA-P` is not a language candidate or a single-variable control.  Coallocating
columns also changes page/TLB placement, segment padding/alignment, pointer
derivation, lifetime, and possibly alias/vectorization information.  It may be
reported as a diagnostic composite, but no subtraction involving it may claim
to isolate allocator calls, headers, locality, or padding.  The primary fixed
comparison is `F-SOA` versus `F-AOS`, explicitly interpreted as the total
representation effect under fixed/full initialization.  Allocator attribution,
if later needed, requires direct call/time/usable-byte counters and a separate
construction-only diagnostic that preserves alignment and alias facts.

Likewise, `F-AOS` versus `R-AOS` changes owner, header, API, checks, lifetime,
and destruction as well as initialization; `R-*` versus `D-*` changes final
capacity trajectory, allocation count, bytes moved, allocator size classes, and
check frequency.  They are end-to-end storage-policy comparisons, not causal
estimates of “initialization only” or “growth only.”  A causal initialization
diagnostic must hold owner/API/allocation/capacity fixed and vary only full-tail
fill versus prefix initialization.  A valid `R-SOA`/`D-SOA` arm is one logical
owner with a shared `len`, element capacity, and growth decision; independently
growing per-column vectors is a different policy and is not an admissible pair.
Every reserve/doubling arm freezes and reports its complete capacity trajectory.

AoS naturally combines per-row capacity checks that are separate in SoA.  This
is a legitimate representation effect, but trap/check counts must be reported
so it is not mislabeled as cache locality.

The doubling rule, minimum capacity, allocation rounding, alignment, failure
policy, zero-sized-type rule, and maximum object size are written into the
implementation manifest before any pilot.  Pilots may estimate noise only; they
may not select these policies.

## 4. Correctness and soundness gates

Correctness is evaluated before and independently of timing.  Any failure below
ends the arm; its performance is not scored.

1. Run the repository-wide verification gate and the complete compiler gate.
2. Reuse the lexer differential corpus, parser malformed/capacity tests,
   deterministic self-parse, and retained
   success -> lex failure -> parse failure -> semantic failure -> success
   sequence with guards intact.
3. Canonicalize every result field-by-field and compare status, error span,
   token/node count, root, links, symbols, reports, and final digest.  Raw record
   `memcmp`/hash is forbidden because padding is not semantic.
4. Test capacities and lengths at 0, 1, `cap-1`, `cap`, and `cap+1`; repeated
   growth; arithmetic overflow; target pointer-index/`isize` object-size limit;
   layout rounding; alignment; poison canaries; and allocation failure.
5. Statically reject borrow-then-grow, a borrow-containing element, growth with
   any live element/slice borrow, and overlapping source/destination replacement.
6. For affine test elements, count construction, move, replacement, drop, and
   raw deallocation.  Every live element is dropped exactly once, moved elements
   in retired storage are dropped zero times, and every allocation is raw-freed
   exactly once.  Allocation failure leaves the old sequence unchanged.
7. Run ASan and UBSan.  Run MSan where a supported Linux runner is available.
   Cross-target layout tests cover at least `aarch64` and `x86_64` and pin size,
   alignment, stride, and field offsets.
8. Reject every path that contracts one affine record into several records,
   including moved fill seeds and a fresh outer constructor containing a nested
   `move`.  Whole-row reads/moves are checked in every expression context,
   including match scrutinees, and every index atom is ownership/type checked.
9. Exercise every supported pointer-index width.  A 64-bit-only experiment must
   say so and may not make a target-generic claim; pointer facts such as
   `align`/`dereferenceable` must come from the selected target DataLayout.

An immediate hard rejection is caused by wrong output, an uninitialized read,
leak/double drop in claimed production semantics, dangling borrow, invalid alias
metadata, missing required trap, observable padding, non-atomic growth failure,
or a hot field-only path that copies the whole record.

The allocation-layout rule must cover `count * sizeof(T)`, alignment rounding,
and the target pointer-index/`isize` maximum.  A mere unsigned-64 multiplication
check is insufficient for `inbounds GEP`.

## 5. Workloads

The primary workload is the exact canonical compiler unit above.  Auxiliary
corpora are mandatory and all results are reported:

- token-dense valid input;
- AST-dense valid input;
- whitespace-heavy sparse input;
- malformed inputs failing early, midway, and late;
- approximately 16 KiB, 256 KiB, and 1 MiB scale points.

Generated corpora use preregistered seeds and hashes.  They cannot be regenerated
after seeing an arm's timing.

Microkernels protect opposing access patterns:

- row-wise producer append;
- kind-only scan;
- span-only scan;
- two-write/six-read mixed pass;
- child-link traversal;
- full-row traversal.

Timed compiler phases are reported separately: construction/initialization,
lexing, parsing, validation/semantic consumers, cold full frontend, retained
full frontend, and destruction/lifecycle.  Three lifetime lanes are distinct:

- **cold public-wrapper latency:** a fresh process makes one call; both arms keep
  the owner alive until the clock and correctness digest stop.  This is not a
  lifecycle result because the current wrapper retains allocations;
- **lifecycle:** construction through equivalent destruction/raw reclamation is
  timed separately, at the same owner-lifetime boundary in both arms;
- **retained:** every arm × corpus × sample starts a fresh process with the same
  initial capacity, warm-up, reset, and iteration order.  Capacity may not be
  inherited across corpora.  Construction and warm-up cost is reported even
  when excluded from the steady-state interval.

Memory is sampled at registered, equivalent lifetime points in all arms.  The
public wrapper is never used as an in-process benchmark loop.

## 6. Measurements and statistics

Every raw sample is preserved as machine-readable JSON and includes run order,
binary/source/corpus hashes, phase, wall time, and correctness digest.  Where the
host supports reliable counters, also record:

- instructions, cycles, branches/misses, L1/LLC/TLB events;
- requested/live/peak bytes by tape and allocation site;
- allocation/reallocation/free counts and bytes moved;
- initialized/fill bytes, touched pages, minor faults, peak RSS/physical
  footprint, final length/capacity/utilization;
- bounds/capacity trap sites and retained branches;
- runtime alias guards and vectorizer remarks;
- raw and optimized IR size, hot-function machine-code size, static frame size,
  measured stack high-water/touched pages,
  vector width, calls, `memset`, `memmove`, `memcpy`, `byval`, and `sret`.

RSS alone is not accepted: zero pages, `calloc`, and `bzero` can hide semantic
initialization.  Component-level byte and allocation counters are mandatory.

Use a balanced randomized permutation/Latin schedule.  A baseline-only pilot
selects sample count from observed noise, after which the count and exclusion
rules freeze.  Final ratios use stratified paired bootstrap 99% confidence
intervals.  Thermal/power state and every excluded sample are recorded; there
is no discretionary outlier removal.

Before any scored run, a machine-readable endpoint registry freezes: every
primary workload and corpus hash; phase/lane/scale aggregation; the ratio
direction (`candidate / baseline`, lower is better); one primary memory metric;
maintenance edit tasks; non-inferiority margins; sample count; and the
multiple-comparison family/hierarchical gate.  Every guardrail uses a confidence
bound against its registered margin, never “not statistically significant.”
The 64 MiB Darwin virtual stack reservation is equal across arms but cannot hide
stack growth: static frame and measured high-water are hard guardrails, and an
xlc-migration claim must also pass the target's production/default stack limit.

## 7. Zero-tax generated-code gate

Before judging any feature benefit, compile the frozen unchanged-source fixture
set with the parent and candidate toolchains.

- checker verdict and diagnostics are identical;
- raw LLVM is byte-identical;
- optimized hot-function assembly bytes, call set, trap sites, vectorization,
  and alias metadata are identical;
- no capacity field, growth branch, allocator path, element-drop loop, runtime
  type test, or unused runtime declaration enters an old primitive-buffer path.

This is a hard identity gate, not a statistical non-inferiority claim.  It is
the guarantee that adding the explicit capability charges no runtime tax to the
existing/default SoA shape.

New-feature code has additional shape pins:

- field projection lowers to its field address and scalar load/store, never an
  aggregate load/copy in a field-only hot path;
- no whole-record copy is induced by field-only code, and no record duplication
  is licensed by `Flat`; any future explicit Repeat/Clone capability is a
  separate design decision;
- `sequence<Flat>` destruction is one backing allocation free with no element
  loop or runtime `needs_drop` branch;
- a proven no-grow push is a destination store plus length increment, with no
  realloc call or hidden slow path;
- the current per-column scoped-alias/vectorization fixtures remain green.

## 8. Decision rules

There are three distinct decisions.

### Capability adoption

E0.1a and E0.1b are decided separately.  The earlier shorthand “two non-toy
workloads win” is withdrawn because it permits selection from many corpora,
phases, and microkernels.  Before scoring, the endpoint registry must name the
complete primary family and guardrails.  For a registered benefit, the
conservative 99% confidence bound—not merely the point estimate—must establish
one of:

- elapsed-time ratio upper bound `<= 0.90`;
- primary-memory ratio upper bound `<= 0.85`, with every registered time
  guardrail inside its non-inferiority margin; or
- representation-maintenance ratio upper bound `<= 0.85`, with every registered
  runtime and code-shape guardrail inside its margin.

All other frozen primary workloads must pass their registered non-inferiority
bound.  A hierarchical gate or family-wise correction prevents choosing a
passing endpoint after observing the data.  Exact workload membership and
non-inferiority margins remain owner-review blockers; therefore this document
does not yet authorize scored timing.

Maintenance burden is limited to synchronized length/capacity invariants,
field-addition edit sites, and canonical source tokens in the data-structure
boundary.  It may not be substituted with an informal readability claim.

Passing makes the capability explicit and non-default.  It does not authorize
teaching it as the universal record/collection shape.

### xlc layout migration

Replacing current Token/Ast SoA requires the candidate/S0 99% CI upper bound to
be `<= 1.000` for both exact-compiler cold full frontend and retained full
frontend.  Every preregistered phase, access-pattern microkernel, scale point,
lifecycle, memory, and stack guardrail must meet its registered confidence-bound
non-inferiority margin.  It must additionally deliver a material gain:
at least 5% time, at least 15% peak/live memory, or at least 15% registered
maintenance reduction.  Mere parity does not justify migration.

If initialized-prefix SoA wins while AoS loses, the correct decision is to keep
SoA and consider only the owning-sequence substrate.  If fixed AoS wins only a
row-centric workload, it remains an explicit local choice.

### Default-writer teaching

No new shape enters default teaching until a benchmark-blind fixed low-tier
writer panel chooses among row-, column-, mixed-, and append-heavy tasks without
performance feedback.  Correct outputs are then scored against frozen
workloads.  Before that run, task-level rather than pooled aggregation, model
destination/version, sampling parameters, seeds, sample count, and multiplicity
handling are frozen.  Reject default teaching if correctness drops by more than 2
percentage points, normalized performance median drops more than 3%, p10 drops
more than 10%, or more than 20% of correct outputs select AoS on tasks where SoA
is measured at least 10% faster.

This external run is not currently authorized.  It requires new explicit
permission covering its prompt, teaching material, candidates, and correctness
diagnostics.  It cannot block an explicit non-default capability decision.

## 9. Review stop points

Three independent attack rounds are mandatory:

1. before scored timing and before any production implementation:
   ownership/soundness, data-layout/codegen, and benchmark-attribution review of
   this protocol;
2. after correctness and code-shape gates, before scored timing;
3. after timing: blind recomputation from raw logs and claim review.

Every blocker is recorded with a resolution or the candidate is rejected or
deferred.  No normative specification, default teaching, or xlc migration lands
from a prose summary or an unresolved review.
