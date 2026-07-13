# E0.1a Flat-record design candidate

Status: research proposal revised after hostile review.  Isolated
experimentation is authorized; production implementation is not, and this file
does not amend the language specification.  The first detached prototype is
not a valid production candidate because its record-fill path duplicates an
affine value.

## Integration rule: no feature flag

The candidate must not be introduced behind a checker, compiler, CLI, teaching,
or runtime flag.  A flag would make one compiler maintain two language
semantics and would violate xlang's single canonical language rule.

Experiments use two isolated toolchains:

- the baseline toolchain is built from the frozen parent revision;
- a disposable candidate branch/worktree implements the proposed semantics
  unconditionally and is never merged merely to run the experiment.

Both compile the same unchanged baseline fixtures for the zero-tax identity
gate.  The candidate lives in a disposable detached worktree and is not a
production change.  After the report, it is either rejected and discarded, or,
only with explicit owner confirmation, reimplemented/landed atomically with
spec, checker, stage 0, xlc, conformance, teaching, and code-shape pins.
Baseline and candidate semantics never coexist in one tool.

## Candidate surface: no new declaration syntax

The smallest proposal adds no `copy struct`, `flat struct`, representation
attribute, automatic layout conversion, or stable-ABI promise.  The explicit
layout choice already appears in `buffer<Record>`: one record element is one
contiguous AoS row.  The checker derives whether `Record` is eligible.

All structs remain affine under OWN-1.  A bare aggregate use remains illegal and
`move` remains transfer.  Admitting record storage therefore does not make
assignment, argument passing, or return silently copy a large record.

## Candidate predicate

The existing `ImplicitCopy(T)` set remains unchanged.  A separate derived
predicate, provisionally called `Flat(T)`, means only fixed-size, region-free,
borrow-free, and without a drop obligation.

The narrow first experiment would admit exactly represented integer primitives,
tag-only enums, and non-empty named records whose fields recursively satisfy the
same predicate.  It would conservatively reject stage-0-inexact floats,
zero-sized records, payload enums, arrays, buffers, boxes, arenas, slices or
other borrow-bearing fields, cells, sequences, erased/generic values, unknown
types, finalizers, and by-value record cycles.

These are experiment limits, not a final claim about the language.  `Flat` does
not imply all-bit-pattern validity, zero validity, observable padding, bytewise
equality/hash/serialization, stable offsets, stable ABI, FFI compatibility, or
implicit Copy.

## Candidate fixed-buffer operations

The existing `buffer_new<T>(count, fill)` is sound because v0 requires Copy T:
it evaluates `fill` once and repeats the value.  Deriving `Flat(Record)` must not
silently relax that precondition.  In particular,
`buffer_new<Record>(count, move seed)` contracts one affine value into `count`
values; an outer fresh constructor containing a nested `move` has the same
problem.  Explicit spelling does not make contraction legal, so both forms are
rejected.

Initialization is therefore an open part of E0.1a, not a solved detail.  The
narrowest remaining candidate is a recursively fresh row recipe containing
only Copy leaves and no `move`, elaborated as one fresh construction per slot.
Its evaluation/effect rules and canonical operation spelling require owner
review before another prototype.  The alternatives are a separately opted-in
Repeat/Clone capability, or a safe per-slot builder/initialized-prefix owner;
neither follows from Flat alone.  Padding would have no semantic value.

Once safely initialized, the fixed buffer would remain fully initialized and
fixed-length: no capacity field, growth branch, slot tag, drop bitmap, or hidden
header change.  The first experiment would allow scalar field
projection/update, whole-row overwrite from one fresh construction, existing
conservative borrows, and `len`.

A whole-record bare read or `move index<Record>(...)` would remain illegal.
Moving an element would leave a hole in fully initialized storage; treating the
operation as a copy would silently expand Copy.  A future explicit row-copy
operation would need independent evidence.

## Required lowering if an experiment is approved

Target DataLayout must determine size, alignment, stride, and field offsets.
Allocation must check multiplication, alignment rounding, pointer-index bounds,
and failure.  An expression such as `index<Row>(rows, i).field` must lower to a
bounds check, row GEP, field GEP, and scalar load/store.  Field-only paths may not
materialize the record or emit `memcpy`, `byval`, or `sret`.

Existing primitive-buffer/SoA fixtures must reproduce their raw LLVM and native
machine code exactly under the candidate toolchain.  This is the zero-tax gate;
no flag is involved.
