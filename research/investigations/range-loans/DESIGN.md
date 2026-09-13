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

The current specification [OWN-7] distinguishes complete resolved paths and
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
