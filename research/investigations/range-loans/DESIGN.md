# Compute range loans

The first consumer is a runtime-sized two-dimensional Jacobi stencil. A
recursive subdivision program checks that the same ownership rule composes
through nested calls. These programs exercise the ordinary compiler path;
neither a program name nor a kernel shape grants a source permission.

The owner selected compute expressiveness before I/O in the range-loan
discussion. The scope here is range formation, subrange reborrowing,
signature-based effects, and counted-loop range permission. Pending operations,
stored loans, cancellation, and I/O scheduling remain outside this change.

## Requirements and evidence

The outgoing v0.54 specification [OWN-7] distinguishes complete resolved paths and
unequal literal subscripts. [VIEW-2] forms whole contiguous views. [PAR-2]
admits one constant-coefficient element map per written root, and denies
an iteration's exclusive loan of enclosing storage. Consequently a helper
cannot receive one row of a shared output buffer in a parallel counted loop.

A row is a runtime-width range. Supporting only literal widths would leave
that consumer blocked. Recursive subdivision additionally requires two
adjacent subranges with a runtime split point, and a child view which can be
subdivided again. The purpose is to preserve contiguous storage and ordinary
calls without copies, runtime overlap tests, or source scheduling constructs.

The relevant standing grounds are single-owner storage, explicit regions,
signature-complete callable boundaries, one shared loan/parallel overlap
judgment, and deterministic proof discharge. The proposed amendments name
the decisions this change revises; no live-tree line is changed without the
owner's ruling.

## Proposed source rule

Both existing view formers gain an optional pair of positional endpoints:
`slice_of(&source, start, end)` and
`mut_slice_of(&uniq source, start, end)`. The one-operand form retains its
whole-view meaning. Endpoints are ordinary own `u64` atoms evaluated once
in source order. Formation requires proof of
`start <= end <= len_of(source)`; no runtime failure outcome is introduced.

The viewed storage is a half-open interval relative to the source view or
run. Its descriptor contains the adjusted data address and `end - start`
length. The source storage and its layout remain unchanged. An empty range
reaches no element storage, while its source must still exist and its endpoint
domain must still be proved.

A view may be subdivided at its existing loan strength; an exclusive parent
also permits shared children. Child ranges stay inside the parent, their
regions do not outlive it, and all paths still resolve to ultimate storage.
The parent cannot provide a conflicting usable access while a child is live.
Disjoint siblings can coexist. Ending the child loans restores the parent's
access. Moving the backing owner or changing its storage descriptor remains
forbidden while a descendant loan is live.

For two live usable accesses into the same storage, conflict requires both
incompatible strengths and an overlapping extent. Proved
`left.end <= right.start` or `right.end <= left.start` establishes
disjointness. Endpoint facts denote values captured at formation; assigning
to the bindings that supplied them never retargets an existing loan.

Calls project a view parameter's declared effects onto the actual view's
range. No callee-body origin summary is introduced. A helper can read or
write inside the range it receives, and the ordinary bounds obligations
still govern each access. The direct-view result ceiling stays conservative:
a signature alone must support every retained result origin.

## Counted-loop permission

An iteration may hold a child loan covering a proved subset of its assigned
range. The first automatic family is adjacent constant-stride ranges
`[stride*i + base, stride*(i+1) + base)`, with nonnegative stride and base
fixed throughout the counted loop. Stride and base may be runtime values.
All endpoint arithmetic must already have its domain proof.

The family is a specified monotonicity argument over distinct counted
indices, not a claim that a product of two runtime values is an ordinary
affine expression. The checker retains the argument from checked value
images; source spelling, optimizer recognition, and target thread count
supply no proof. Reads and writes through the iteration's view stay within
that assignment. An access outside it, an unresolved range, or an enclosing
write not covered by the existing accumulator rule denies loop permission.

The selected image mechanism traverses recorded exact products and transparent
copy handles, with affine sums as its ordinary representation. Preheader
values surviving continuing kills are fixed; a pure exact product of fixed
operands is fixed as well. A product with one index-dependent operand is
admitted only when its resulting stride and base remain affine. This covers a
runtime stride derived as `width + padding`, an invariant runtime base, and
equivalent endpoint computations without adding general nonlinear search.
Unknown body loads and loop-carried replacements cannot become fixed values
merely by retaining the same binding name.

Ordinary recursive sibling calls instead consume the normal range judgment:
for a proved `lo <= mid <= hi`, their output ranges
`[lo, mid)` and `[mid, hi)` are disjoint regardless of how mid was computed.
There is no equal-size requirement on that use of [PAR-1].

## Compiler representation

Structural checking currently checks borrow conflicts before the arithmetic
flow pass. The proposed representation carries range formation and potential
range conflicts as required, source-positioned obligations into the existing
ProofContext, like bounds obligations. A checked program is produced only
after these obligations succeed. Ordinary whole-place conflicts keep their
existing disposition.

The flow pass captures immutable endpoint images at each formation, checks
domain and overlap in their control-flow context, and retains the successful
facts for parallel permission. Neither permission nor lowering reruns the
proof. The representation must preserve aliases, nested origins, argument
evaluation accesses, and backing-storage lifetime through every consumer.

The implementation details remain provisional until the complete programs
and negative cases exercise them. In particular, branch joins, result origin
ceilings, and same-statement child lifetime must not gain precision merely
because a convenient caller has one possible origin.

## Alternatives and selection ground

- Literal-only or equal-size-only ranges leave runtime rows or recursive
  uneven subdivisions unexpressible. They do not meet the two consumers.
- Separate storage per row changes the allocation and locality of the
  algorithm. Whole-buffer copies or runtime disjointness checks also fail
  the required storage and proof-erasure properties.
- A new split/join resource protocol would duplicate range ownership.
  Ordinary views with verified extents already express the required access.
- General automatic nonlinear or index-injectivity search is not selected.
  The automatic family is finite and specification-fixed; harder supported
  arithmetic uses the existing explicit certificate mechanism.
- A separate parallel alias analysis would duplicate the ownership judgment.
  Checked range evidence must serve both ordinary loans and parallel calls.

These are proposed grounds, not a claim of measured speed or a general
solution for arbitrary scatter, irregular task dependencies, or early exit.

The v0.55 specification delta is numbered rules +0/-0 (revising OWN-5,
OWN-7, VIEW-2, VIEW-6, OP-1, PAR-2, ENT-3.S6 and ENT-6), lexical tokens
+0/-0, operation spellings +0/-0, grammar productions +0/-0, and exceptions
+0/-0. Bounded operands and reborrows extend the same two operation records;
no workload-specific exception or additional declaration record is added.
The selection ground is evidence-selected for bounded child loans: the two
complete consumers below are blocked by the outgoing rules. Reusing those
view formers and limiting automatic partition images to the stated finite
family are minimality-selected, given the required runtime rows and recursive
subdivision. Performance superiority is not a selection ground for this
language amendment.

## Validation criteria, fixed before measurement

The complete stencil must handle runtime dimensions, preserve boundaries,
perform repeated updates, and call an ordinary row helper. Correctness is
checked against a separately written native implementation, including
non-square and narrow/tall or wide/short domains, zero steps, and more than
one update. The recursive witness must split unevenly, subdivide children,
and regain parent access after they finish.

Negative cases must reject actual overlap, out-of-bounds endpoints, invalid
parent access, backing-storage mutation, and escape. Distinct values at every
position and nonzero stencil input must make a mistaken range observable.
A changed endpoint binding must not retarget a formed view.

The emitted parallel module must contain an actualized row loop; the
sequential module must contain none. Inspect the lowering for descriptor-only
formation and the absence of runtime domain/overlap checks.

Measure the same algorithm and storage policy in Whitefoot and native code
at one, two, and four workers. Record correctness, wall time, process CPU,
problem sizes, flags, and the exact measured revision. Measure saturated
multi-step work and a small domain separately. A repeatable slowdown above
five percent warrants attribution before calling the result competitive;
a smaller result is not evidence of a universal no-loss guarantee.

The compiler gate and existing compute program checks remain in force.
The stencil joins the maintained compute benchmark once its ordinary
compiler path works; its source lives there from the start. This document
owns the design and resulting evidence until a later investigation
supersedes it. The amendments are removed only by an owner ruling.

## Baseline, 2026-09-13

The compiler at main revision `8909feb1` built with
`cargo build --profile gate --locked --offline` (exit 0). The complete sources
`research/experiments/compute-bench/programs/stencil.wf` and
`research/experiments/compute-bench/programs/range_split.wf` pass its parsing
and canonical-source checks. Running each with `whitefootc --par --emit-llvm`
stops with exit 1 and an OP-1 source diagnostic at its first three-operand
view formation. The former stops at `slice_of(&current, before, first)`;
the latter at `mut_slice_of(&uniq output, 0_u64, middle)`.

These are proposed-amendment consumers, not valid programs under v0.54.
No executable or performance result for either program exists at this point.
The source files remain in their final benchmark home while implementation
and gate integration proceed. `make design-lint` passes with three pending
amendments and no live-tree changes.

## Range formation implementation, 2026-09-13

The v0.55 amendment adds the optional endpoint pair to both existing view
formers. The compiler submits both VIEW-2 conjuncts to the existing
ProofContext and installs the successful formation's length as the captured
affine difference. Lowering adjusts one descriptor's pointer and length; it
does not introduce a runtime range test or allocate storage.

The complete stencil now passes semantic checking and its native smoke test
observes the expected two-step interior and boundary values. A second native
test sums a runtime subrange of distinct values, changes an endpoint binding
after formation, and exercises an empty range at the source end. Three
negative formations cover reversed endpoints and an end outside the source,
including an empty range outside it.

This establishes range formation and sequential execution. No parallel
performance result is claimed by this first implementation milestone.

## Recursive loans implementation, 2026-09-13

Bounded views now retain relative range identities through owned and borrowed
view parameters. Arithmetic-dependent conflicts become mandatory OWN-5
obligations in the existing flow checker. The checker captures endpoint value
images and checks the fixed four alternatives: either range ends before the
other starts, or either range is empty. The shared resolved-place judgment
consumes formation-time separation proofs for effects and parallel calls.
Unequal relative indices alone cannot separate different range frames.

The recursive witness forms adjacent exclusive children, subdivides each by
ordinary recursive calls, and reads and writes the parent after both children
are consumed, within the same region. Native sequential and parallel
executions pass. The parallel ledger permits and actualizes the pair of
recursive calls. Negative compiler cases cover overlap, parent access while
children live, reassignment of a captured endpoint, and a parent consumed
later in the same call as its child. Affine consumption releases the child
after the statement, preserving the full statement's simultaneous loan set.

The library suite passed 1591 tests before the final same-statement lifetime
refinement; the 25 targeted slice tests then passed with that refinement.
PAR-2 range permission, the independent dimension-matrix oracle, conformance
integration, and performance measurements remain in progress.

## Counted ranges and independent native oracle, 2026-09-13

PAR-2 now consumes the retained adjacent-range images and their two sign
proofs, with both proofs rooted and remapped in the existing derivation
ledger. The loop survey traces helper effects and descendant loans through
the shared place representation. Positive containment requires matching path
selections; absence of proved divergence alone is insufficient. A shifted
second partition can be disjoint in one iteration and overlap the first in a
different iteration, and the permission tests reject that case.

Incoming formal views now anchor their storage origins so recursive ranges
remain attributable. Two shared calls through one formal origin may overlap;
two exclusive calls still conflict. The previous unknown-parameter fixture
was changed to a returned view, whose origin remains unresolved, and its
fail-closed verdict is retained. This is a representation extension with
paired positive and negative coverage, not removal of the unresolved-origin
check.

The stencil now initializes both grids with distinct positive finite
binary64 encodings derived from squared positions. This makes missing work
and mistaken row offsets observable throughout the grid, beyond the initial
impulse smoke test. The C oracle stores its grid column-major and compares
every result bit. Its 42 dimension/step combinations cover narrow/tall,
wide/short, rectangular and larger grids with 0, 1, 2, 3, 7 and 16 steps.
Sequential and parallel native modules pass that matrix at widths 1, 2 and 4.
All benchmark reference forms also passed it at every emitted host width.

Before timing, the saturated fixture was fixed at 1024 by 2048 for 16 steps.
The row loop's emitted weight is 294 and the shipped scheduler's work unit is
150000, so its split floor is 1022 rows. A 768-row grid would leave the row
loop unsplit; 2046 interior rows afford four chunks and exercise the requested
one/two/four-worker comparison. This selection follows the emitted mechanism,
not a measured ranking. The separate small fixture is 17 by 13 for 3 steps.
Neither measurement changes a compiler flag, runtime work unit, or source
chunk count.

The corpus adds eleven cases for range domains, captured endpoints, empty
backing lifetime, strength and escape errors, recursive parent restoration,
and runtime-stride helper calls. The native conformance adapter passed 758
cases with the one existing expected failure and one pending case unchanged;
coverage remains 160/160 rules. Full repository checks and timing are still
pending at this checkpoint.

The first canonical gate attempt passed all 1601 library tests but could not
complete the existing program suite: sandboxed loopback listeners were refused
and the process's 256-descriptor soft limit stopped its deep-tree fixture.
The complete gate is rerun with loopback access and a 4096-descriptor soft
limit; neither change alters a test or compiler setting.

## Native follow-up, 2026-09-13

At `3f6db217f0cf47123d64817c3bd7999e229dae22`, canonical `make check` passed:
1601 library tests, the complete program and research suites, native
conformance Pass=758/Xfail=1/Skip=1, and snapshot Pass=484/Flip=0. The existing
loopback and deep-tree fixtures passed with the host permissions described
above. The corpus's expected failure and pending case are unchanged.

The first saturated measurement used Apple M1 Pro, eight physical/logical
CPUs, Apple clang 21.0.0, the bundle's unchanged flags and pinned references,
five rotated/reversed passes and five warm calls per process. Its
[complete call stream](../../experiments/compute-bench/stencil-2026-09-13-before.tsv)
is retained as the pre-correction observation and is read by the existing
`reduce.awk` with `passes=5` and `calls=5`. WF's wall/process-CPU medians in
microseconds at W=1/2/4 were 18234.5/18228.0, 10877.7/20725.0 and
7463.5/24060.0. The W=4 paired best-reference ratio was 1.172, above the
predeclared attribution threshold; the sequential WF control was 14182.0 us.

Inspecting the nested parallel module exposed a specification discrepancy:
the existing element-map selector also admitted direct `SliceIndex` writes,
although PAR-2's element family names direct array/buffer storage. Such view
writes must use the new adjacent-range assignment or remain sequential. The
stencil helper's inner loop was therefore actualized outside the specified
permission, even though its independently checked outputs were correct. The
regression `direct_view_element_writes_need_a_range_assignment` fails on that
implementation with `PermittedEligible`. The element selector is corrected
without changing source acceptance, the written program, the range family,
or any measured scheduler constant. The row assignment and its helper effects
retain their PAR-2 permission. Timing after this correction is a new
measurement, not a replacement of this observation or selection of a faster
language rule. The measured call streams serve this comparison and may be
removed when the benchmark question is retired or superseded by an equivalent
retained measurement.

The corrected row-loop weight is 292. The shipped rule's minimum chunk is
`ceil(150000 / 292) = 514` rows, so 2046 interior rows afford only three
chunks before rounding down to a power of two: two chunks. Before measuring
the corrected implementation, the saturated fixture is therefore extended to
1024 by 4096 for the same 16 steps; its 4094 interior rows afford four chunks.
The original 1024 by 2048 fixture remains a matched-size control, and the small
fixture remains 17 by 13 for three steps. One benchmark selector,
`WFB_STENCIL_GRID=large|original|small`, names these cases, with `large` the
default. This changes neither the compiled Whitefoot source nor the scheduler
work unit and records the limited parallelism of the original size rather
than hiding it in a renamed saturated result.

The separate [CI regression run 34755631998](https://github.com/mbbill/Whitefoot/actions/runs/34755631998)
failed at `records` W=2 (baseline/new wall ratio 0.969, baseline lower in 4/5
pairs). Its retained manifest identifies the synthetic PR merge revision
`b1ad35cf73142f0e2215e2e2d24ba3a1a6df4704` and baseline `8909feb1`.
Both `records` executables have SHA-256
`4034a588eff3c08974f588e2717c6142b12d8ad5d85e8ed3e9d4710168071429`
before and after the run; every other old kernel's paired executable hashes
also match exactly. Thus this reported slowdown is a difference between two
executions of identical bytes, not generated-code regression. The failed run
and its raw artifact remain the evidence; no threshold or check is weakened.
