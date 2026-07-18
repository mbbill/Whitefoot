# E0.1a ownership-route paired experiment protocol

Status: superseded historical draft, suspended before Lock A, 2026-07-13. This
document is not a preregistration. D11 replaces its immediate next step. The
bounded G0-Core and a dense-family Lock A described in
`../../optimizer-language-research/implementation/general-purpose-data-structure-capability-RESEARCH.md`
must close before an owner may reconsider the pause, but they do not
automatically restart this narrowing. The dense-family lock must explicitly
retain, revise, or supersede every relevant arm and measurement below. This
document authorizes no G0-Core work, family-lock work, candidate
implementation, execution, scored timing, production change, wfc migration,
pattern change, default teaching, or external disclosure. The locks below are
historical draft requirements and may not be entered while the suspension is in
force.

This protocol answers an upstream question left open by `PROTOCOL.md`: which of
two concrete ownership mechanisms is worth carrying into the fixed-record
storage experiment? It does not decide whether AoS should replace any existing
SoA tape. Capability adoption, wfc layout migration, and default teaching remain
separate decisions.

## 1. Decision question and non-arms

The paired candidate arms are named by mechanism rather than by letters, because
the existing research report already assigns letters B, C, and D differently:

- `DECLARATIVE_COPY`: a compiler-checked `copyable struct` declaration makes only
  selected records Copy and reuses the existing fully initialized
  `buffer_new<T>` operation.
- `AFFINE_FIXED_BUILDER`: records remain nominally affine; a compiler-derived
  fixed-storage predicate admits selected records to `buffer<T>`; a transient
  linear builder constructs a fully initialized fixed buffer.

`CURRENT` is the unchanged-language identity baseline. It cannot express either
record-buffer task and is not a performance competitor on new-feature fixtures.
It is the authoritative comparator for unchanged-source semantics and code
shape.

Pure automatic structural Copy is a proposed non-arm for this run. Layout does
not determine whether an all-Copy-leaf record is ordinary data or a nominal
protocol value, and structural inference would silently reclassify existing
records after field changes. Copy-by-default plus an optional negative `affine`
marker is another
proposed non-arm: omitting a positive `copyable` marker fails closed with a
checker diagnostic, whereas omitting a negative marker can silently accept an
unintended duplication. This experimental narrowing is not a production ruling
and does not update the design tree. Lock A requires the owner to approve only
the scope of this paired run; all production routes remain open until an owner
redecision.

The experiment compares complete mechanism bundles. In particular,
`DECLARATIVE_COPY` repeated fill and `AFFINE_FIXED_BUILDER` fresh per-slot
construction are not an ownership-only causal contrast. Initialization work,
checks, source shape, and compiler surface are reported separately; the primary
result is explicitly the total route effect.

### 1.1 Relationship to the existing E0.1 protocol

For this paired screening run, this document replaces `PROTOCOL.md`'s old
pre-repair baseline, its requirement to select an ownership route before
constructing an F-AOS arm, and its serial E0.1a/E0.1b treatment of a builder.
It imports that document's fixed-storage-applicable correctness, workload,
zero-tax, capability-adoption, wfc-migration, pattern, and default-teaching
gates unless this document tightens them. Reserve/grow, arbitrary-drop, and R/D
sequence gates remain closed and are not silently imported into the transient
builder screen.

For this paired run, this document also has global precedence over
`PROTOCOL.md` for authorization status, baseline, lock order, candidate timing,
profiling, and external calls. The older permission for isolated implementation
and smoke experiments did not authorize either candidate here. The historical
draft would have required exact Lock A review and owner approval. D11 supersedes
that next state; only a future dense-family lock may retain, revise, or supersede
these requirements, and this document authorizes no candidate work.

`AFFINE_FIXED_BUILDER` imports inaccessible partial initialization only inside a
transient constructor and returns no initialized-prefix owner. It therefore
couples enough E0.1a/E0.1b machinery to require this replacement protocol, but
it does not select a growable or partially initialized sequence. If either
bundle advances, a later owner-reviewed amendment must bind it back into the
absolute capability experiment in `PROTOCOL.md`. This run can screen a bundle;
it cannot establish that the capability itself has earned production existence.

## 2. Constitutional hypotheses

The hypotheses are opposing and falsifiable.

### H-COPY

For ordinary fixed-record data, an explicit declaration is the smallest
fail-closed extension. It preserves current affinity by default, reuses GRAM-9
and `buffer_new`, and should impose less checker, grammar, lowering, and writer
state than a partial-initialization mechanism.

It loses if storage eligibility should not grant implicit duplication in every
assignment, argument, return, payload, and fill; if weak writers add `copyable` as a
diagnostic escape on a real nominal-uniqueness task; or if default code
materializes unapproved whole-record copies.

### H-AFFINE

Fixed storage and contraction are independent predicates. Keeping OWN-1
unchanged prevents accidental semantic duplication and makes ownership stable
under field evolution.

It loses if the only demonstrated consumers are ordinary data while its builder
adds retained checks, state errors, source repair, or implementation surface
without a compensating P0, W1, or required-expressiveness result.

The tag-only Copy amendment is prior evidence, not a verdict. It removed a
measured 1.6-1.8x loss caused by whole-value affine Bool dataflow. A 24-, 56-, or
88-byte record has different copy visibility and use patterns. This protocol
therefore measures record use sites and materialization rather than extrapolating
from tag size.

The declarative arm is not selected by Rust analogy. Its experimental ground is
the whitefoot-specific fail-closed omission direction and reuse of one current Copy
judgment. The opposing evidence is non-Rust as well: Swift's noncopyable-value
work shows that an all-scalar representation need not be semantically
duplicable, while Move separates storage ability from copy ability. D3 remains
a production gate, and the paired experiment must name its eventual P0, W1, or
W3 delta over Rust before any route can advance beyond capability screening.

## 3. Baseline and staged lock discipline

The candidate baseline must be one clean commit descended from
`e8538d9d165a53be6e7a1874e223acbaf3292462`, so it includes the strict GRAM-9,
recursive-projection, and expression-context ownership repairs. The old
`58baa71` prototype baseline is not admissible for a new route experiment.

Lock A is committed before candidate implementation. It records:

- baseline commit, complete source-tree hash, and this protocol hash;
- explicit owner approval of the two-arm experimental narrowing, the transient
  builder coupling, and their non-effect on the still-open production choice;
- exact semantic supplements, source fixtures, independent expected results,
  state oracles, corpus frame, and candidate delta schemas;
- the complete current-source reserved-token collision census;
- teaching excerpts, task generator/frame, prompts, evaluators, seeds, writer
  experimental unit, and proposed external destination;
- direct contrast registry, endpoint weights, margins, exclusions, confidence
  and power methods, every scored sample count, bootstrap resample count/seed,
  and multiplicity family;
- whether a real owner-ratified nominal-affinity storage task exists and the
  exact post-construction contraction invariant it requires.

Before owner approval, the exact proposed Lock A bytes receive independent
ownership/state, layout/codegen, and benchmark/statistics hostile reviews. A
material correction changes the hash and repeats all three reviews. Only the
reviewed bytes may be committed as Lock A.

Lock B is committed only after both candidate implementations pass untimed
correctness and hostile review. It freezes the exact recoverable patches,
binaries, compiler and linker versions, target/OS/allocator manifests,
diagnostic and analysis schemas, candidate-produced fixture artifacts, request
fingerprints, and schedule. It attests that every Lock A sample count is
unchanged and executable on the locked environment; it cannot choose or revise a
count. Changing a scored count requires a newly reviewed and owner-approved Lock
A before implementation. No candidate timing, profiling, or external writer
request may occur before Lock B. Any candidate-code change after Lock B creates
a new run identity and requires a new Lock B.

For each writer task, Run Freeze records the first correctness-green source or
the registered terminal non-green outcome, the full trajectory, and hashes.
Campaign Freeze occurs only after every scheduled bundle/task trajectory has a
Run Freeze. Hidden timing, IR, assembly, transfer accounting, and code-shape
evaluation may not be generated or inspected before Campaign Freeze. Until
then, operators capable of affecting requests or campaign handling receive only
the deterministic correctness diagnostics authorized for the repair loop.
Hidden results are never returned to the writer. Independently derived expected
outputs belong to Lock A; candidate-produced IR and executable hashes belong to
Lock B or Run Freeze, never to the earlier design lock.

There are three isolated, unconditional toolchains built from that baseline:
`CURRENT`, `DECLARATIVE_COPY`, and `AFFINE_FIXED_BUILDER`. A feature flag, hidden
environment switch, runtime type branch, or one binary containing both candidate
semantics invalidates the experiment. The two candidates must share a
byte-identical target-DataLayout record-layout and field-lowering substrate. The
candidate-specific ownership and initialization code begins only above that
audited common substrate.

Candidate branches and executables remain disposable, but their exact patches
do not. Before Lock B, each patch is archived with its base commit,
SHA-256, build manifest, and test manifest. A hash without recoverable bytes is
not durable evidence.

## 4. Frozen experimental semantics

These semantics are candidate supplements, not changes to
`spec/kernel-spec-v0.6.md`.

### 4.0 Shared record domain and layout rule

Both arms use one closed structural domain so an implementation cannot make one
candidate look stronger by silently accepting more field types.

`RecordDomain(T)` is target-independent and true exactly when T is a concrete,
non-generic, non-empty record whose declaration graph is acyclic and every field
is one of:

- an integer or floating-point primitive, excluding `unit`;
- a tag-only enum already classified Copy by OWN-1; or
- an earlier record satisfying `RecordDomain`.

Every other type constructor is outside this experimental domain: `unit`, an
empty or recursively zero-sized record, payload enum, array, buffer, box, arena,
slice, borrow/region-bearing form, generic/unknown type, builder, or by-value
cycle. These exclusions are prototype bounds and blocking production debt, not
claims that the forms can never receive fixed storage or declared Copy.

`DeclaredCopy(T)` is target-independent and true exactly when T has the
experimental declaration marker, satisfies `RecordDomain`, and every
record-typed field recursively satisfies `DeclaredCopy`. Numeric primitive and
tag-only Copy fields are leaves. An outer marker never makes an unmarked inner
record Copy. Target support does not change whether bare use or `move` is legal.
Separately, `FixedLayout(T, target)` requires a target-derived positive size,
alignment, stride, and field offsets for T.

Both candidate record-buffer paths use this allocation algorithm, which is an
experimental extension beyond current OP-9 rather than a claim about its present
text:

1. derive positive stride S and alignment A from the locked target DataLayout;
2. reject the record-buffer use if A exceeds the locked allocator's guaranteed
   alignment in this slice;
3. before allocation, trap in fixed order if `count * S` overflows u64, exceeds
   the target allocator size type, or exceeds the signed maximum of the target
   pointer-index type used by record GEPs;
4. otherwise call the same allocator and apply the same OOM policy in both arms;
5. for count zero, call the allocator with zero bytes, retain its returned
   pointer without dereferencing it, and pass that same pointer exactly once to
   the common record-buffer deallocator on a normal drop.

The stricter checks are confined to new record-buffer paths. Existing primitive
buffer OP-9 lowering remains byte-identical. The 32-bit lane validates candidate
record layout and arithmetic; it is not mislabeled as unchanged-baseline
identity if it exposes an independent current primitive-buffer defect.

Both arms share one candidate-only compiler-derived record-buffer drop path on
normal exits: exactly one raw free and no element loop. Trap is abort and has no
cleanup post-state. The current stage-0 omission of general primitive-buffer
drops remains separate production debt and cannot be repaired inside the
unchanged-source identity lane.

### 4.1 `DECLARATIVE_COPY`

The only added declaration spelling is:

```whitefoot
copyable struct TokenRow {
    kind: TokenKind;
    start: u64;
    end: u64;
}
```

The experiment reserves the currently collision-free candidate word `copyable`
and inserts it immediately before `struct`. Lock A repeats the collision census
over every frozen Whitefoot source; a collision requires owner review of a new
spelling rather than renaming baseline code. The marker names the checked
duplicability invariant and deliberately does not reuse Rust's `Copy` token.
There is no inferred Copy record, negative `affine` marker, call-site copy
marker, user conformance to Copy, Copy enum declaration, or second spelling.

A declared record is OWN-1 Copy exactly when `DeclaredCopy(T)` holds. Target
padding is permitted but never observable. A marked declaration outside
`RecordDomain` receives one deterministic experiment-scope diagnostic.

Eligibility is checked at the declaration. Adding one ineligible field makes
the declaration fail locally; the compiler never silently removes Copy or
reclassifies the type. `move` on a declared Copy record remains a hard OWN-1
error. Bare use copies uniformly in every existing ownership context:
the closed table below covers the exact contexts, including payload binding and
both fixed-fill operations. Field projection remains a scalar access.

The existing Copy predicate is used uniformly: `buffer<T>`, `buffer_new<T>`,
`array<T, N>`, and `array_new<T, N>` admit a valid declared Copy record.
Aggregate-array construction, frame limits, fill accounting, and lowering are
candidate fixtures. No other storage or collection rule changes. Existing
primitive array and buffer paths remain byte-identical.

Declared-Copy record arrays retain the current inline frame storage class. At
monomorphization, their target stride and alignment come from the shared
DataLayout substrate, and `N * stride` is checked before layout. An overflow of
u64, the target size type, or the signed pointer-index range, excessive
alignment, or a result above the frozen frame limit is a deterministic
compile-time rejection. `array_new` performs exactly N semantic Copy
initializations, including zero when N is zero, and the resulting no-resource
array has no element drop loop. These candidate-only arrays are ceiling evidence
and never enter the direct route contrast.

The closed ownership-context table is shared by checker conformance, diagnostics,
and copy accounting. It contains let initialization; set RHS; operation and user
arguments; struct and enum construction fields; return; `give`; buffer and array
fill; complete indexed replacement; and enum-payload binding/propagation through
match and `try`. Place origins cover an own binding, shared dereference, unique
dereference, buffer/array index, and nested projection. A record itself is not a
legal match scrutinee; aggregate-payload match and `try` are checker-only until
their separately priced ABI lowering exists. A Copy record nested in an affine
outer value remains covered. Any newly discovered grammar/AST ownership context
invalidates the closed table and Lock A rather than being silently omitted.

The candidate emits a deterministic analysis artifact for every semantic record
copy. Each entry contains source node, function, closed-table context, type,
target size and stride, stable source-site identifier, and static/dynamic
multiplicity. A many-to-many provenance map links semantic sites to lowering
operations without forcing overlapping labels into one category. It records
aggregate and scalarized field operations, `memcpy`, `byval`, `sret`, spills,
and attributable bytes. Physical cost is one of `proven_eliminated`,
`memory_materialized(bytes)`, or `indeterminate`; indeterminate provenance earns
no performance credit and fails a protected lane. Reporting on/off must produce
identical generated code. The same transfer accounting is applied to affine
builder pushes and complete-row replacements.

### 4.2 `AFFINE_FIXED_BUILDER`

OWN-1 does not change. Every record remains affine, and `move` remains required
at each consumption site.

The compiler derives an internal predicate named
`FixedBufferElementEligibleRecord(T, target)`. It is true exactly when
`RecordDomain(T)` and `FixedLayout(T, target)` hold. This is Option D's
protocol-local narrowed Flat-storage idea, not a fourth ownership route and not
a surface capability. `buffer<T>` admits this record predicate, while
`buffer_new<T>` and record arrays continue to require Copy because this arm adds
no affine array initializer.

The one experimental construction spelling adds the built-in type form
`buffer_builder<T>` and these exact structural operation signatures:

```text
buffer_build_new<T>(u64) -> own buffer_builder<T>
    allocates(heap), traps
buffer_build_push<'r, T>(&uniq 'r buffer_builder<T>, own T) -> own unit
    reads('r), writes('r), traps
buffer_build_finish<T>(own buffer_builder<T>) -> own buffer<T>
    traps
```

The canonical calls are positional operation-table calls:

```whitefoot
let builder: own buffer_builder<TokenRow> =
  buffer_build_new<TokenRow>(capacity);
region 'push {
  buffer_build_push<'push, TokenRow>(&uniq 'push builder, move row);
}
let rows: own buffer<TokenRow> =
  buffer_build_finish<TokenRow>(move builder);
```

The push region must obey OWN-11; a loop introduces the call-scoped region
inside its body. A live unique borrow prevents finish until that region ends.

This experimental builder is deliberately confined rather than fully
compositional. It is legal only as an `own` local initialized directly by
`buffer_build_new`. It cannot appear in an item signature, return, `give`,
field, enum payload, generic argument, array, buffer, box, arena, slice, or
another aggregate; it cannot be moved to another binding or owned argument.
It cannot be a `set` target or a `set` right-hand side, so an open allocation
cannot be overwritten or transferred through destructive assignment.
Its only unique borrow is the call-scoped push operand, and its only consuming
use is `buffer_build_finish`. These restrictions are part of the measured
bundle and blocking production debt; a task that requires factoring an open
builder across calls is an expressiveness failure for this arm.

The independently modeled state machine is:

```text
new(c)              -> Open(pointer, c, 0)
push(Open(p,c,i),v) -> Open(p,c,i+1)  when i < c; v is consumed once
finish(Open(p,c,c)) -> dead builder + Buffer(p,c)
normal scope exit   -> dead builder + raw_free(p) exactly once
trap                -> process abort; no cleanup or post-state
```

The semantic rules are:

- `buffer_build_new` accepts only
  `FixedBufferElementEligibleRecord(T, target)`, uses the shared allocation
  algorithm, and starts in `Open(pointer, capacity, 0)`;
- the builder owns `{pointer, capacity, initialized}` and is not indexable,
  sliceable, field-projectable, or observable through `len`;
- `buffer_build_push` checks `initialized < capacity`, consumes any live
  `own T` exactly once, writes it to the next slot, and then increments
  `initialized`;
- no element, initialized prefix, or raw tail can be read or borrowed;
- `buffer_build_finish` checks `initialized == capacity`, consumes the builder,
  and transfers the same pointer into the existing two-word
  `{pointer, length=capacity}` `buffer<T>` with no tag, bitmap, spare header
  word, or element loop;
- in accepted source, underfill and over-push are runtime traps; an existing
  machine-verified proof may elide a check only when it proves success at that
  exact site, and this slice adds no candidate-specific compile-time rejection;
  trap-abort paths owe no deallocation;
- every non-trap edge leaving the builder's scope, including fallthrough,
  `return`, `break`, `give`, and an ERR-3 `try` propagation return, raw-frees the
  allocation exactly once with no initialized-prefix drop loop;
- capacity zero uses the shared zero-count allocation rule and may finish
  immediately;
- no grow, reserve, take, hole, reallocation, arbitrary affine drop, or
  initialized-prefix sequence escapes this builder;
- nested records use ordinary GRAM-9 ANF construction: construct each inner
  value once, move it into one outer value, then move that outer value once into
  one slot.

After finish, field reads and writes use ordinary buffer bounds checks and scalar
projection. A bare whole-row read is rejected by OWN-1. Moving a row out is
rejected because it would leave a hole. Complete-row replacement by moving one
live eligible record into a uniquely writable slot makes the previous
resource-free affine row dead exactly once and makes the moved row the slot's
sole value. The semantic replacement/death and all backend traffic enter the
transfer accounting. Replacement may not become an extraction operation or add
an element drop loop.

The builder's capacity check is never weakened for speed. The artifacts report
whether an existing machine-verified proof removes it. A retained check is
measured; an unproved elision is a soundness failure. The experiment does not
assume the current prover can discharge a variable-trip push loop.

### 4.3 Candidate delta ledger

Before implementation, each arm must have a machine-readable supplement with
one row for every current normative rule and every related formal ownership or
state-reconciliation obligation. Each row is classified
`unchanged-reused`, `candidate-amended`, or `not-applicable` and records its
rationale, semantic delta, implementation component, conformance cases, and
report surface. The same supplement enumerates changed grammar productions,
reserved tokens and operation names, diagnostics, teaching tokens, and known
production debt. Counts remain unweighted; they are not collapsed into a
subjective maintenance score.

At minimum, `DECLARATIVE_COPY` must price LEX-1/FORM-3 and GRAM-2 spelling,
TYPE-2, OWN-1 and every ownership expression context, OP-1/OP-4/OP-9,
`array_new`, `buffer_new`, target layout and frame rejection, STOR-1/STOR-3,
DIAG-2/DIAG-3 copy reporting, aggregate ABI/lowering, generic/payload-ABI debt,
and formal ownership reconciliation. `AFFINE_FIXED_BUILDER` must price
FORM-3/GRAM-3/OP-1 and the three operation rows, TYPE-2, OWN-1 and
OWN-5/6/10/11/12, OP-4 row access/replacement/extraction, STOR-1/STOR-3,
OP-9-equivalent layout hazards, EFF-2, ERR-4, DIAG-2/DIAG-3, builder state and
cleanup, local-only confinement, generic/payload-ABI debt, check accounting,
and formal ownership/state reconciliation. This ledger is part of Lock A and
must pass its exact-byte reviews before any candidate code exists.

## 5. Pre-implementation corpus census

Before candidate code, freeze a repository corpus containing production Whitefoot
sources, the current wfc unit, and the two completed first-green default-floor
artifacts. Tests written to exercise a proposed route are excluded from
prevalence counts.

For every record declaration and use site, two independent read-only audits
record:

- recursive structural eligibility under each candidate;
- target size, alignment, and stride on every locked DataLayout;
- field-only reads/writes and whole-value assignment, argument, return, match,
  construction, replacement, and reuse counts;
- whether the source already uses a mechanical SoA encoding;
- whether the owner identifies a real post-construction one-value contraction
  invariant.

Disagreements are resolved before candidate implementation and the frozen table
is archived. Declaration count alone is not prevalence evidence. The existing
manual observation that roughly 13 of 23 wfc records have only Copy leaves is a
prior to verify, not a registered result.

A synthetic scalar protocol token is always permitted as an adversarial checker
fixture, but it is not prevalence evidence and cannot by itself reject
`DECLARATIVE_COPY`. A nominal-affinity result becomes a decision gate only if the
owner identifies and freezes a real project requirement before either candidate
is implemented. The lock must state the required invariant and why ordinary
affinity is part of the task contract. Current closed-unit Whitefoot has no private
constructor or module boundary, so this lane may claim only that one issued value
cannot be duplicated after construction; it may not claim that fresh values are
unforgeable.

The verified census and a `CURRENT` semantic-event trace freeze the weights for
repeated fill, independent construction, field-only, whole-row, and replacement
cells. If no defensible event weight exists, every atomic cell remains a separate
guardrail under an intersection rule; no ungrounded pooled average is allowed.
Because current wfc is SoA, Lock A must freeze a machine-readable mapping from
column events to logical row events, including temporal grouping, corpus
identity, and an independently audited trace. An ambiguous mapping invokes the
unweighted intersection fallback rather than an inferred row weight.

## 6. Fixture matrix

Every fixture is compiled by both candidate toolchains where its semantics
exist and by `CURRENT` for the unchanged-source identity lanes. Independently
derived source and expected-result hashes are Lock A items; candidate-produced
artifact hashes are Lock B items.

### 6.1 Common layout and value shapes

- target strides of 8, 16, 24, 56, and 88 bytes;
- a padding-sensitive `u8`/`u64` record and reordered control;
- a record containing `f32` and `f64` leaves;
- a two-level nested record;
- empty, `unit`-only, mixed `unit`/numeric, and recursively zero-sized negative
  controls for the shared `RecordDomain` boundary;
- `DECLARATIVE_COPY` arrays at the same aggregate sizes, reported outside the
  direct common-feature contrast;
- zero capacity, one element, exact capacity, over-capacity, and maximum-size
  boundary cases;
- target layouts for x86_64, AArch64, and at least one 32-bit pointer-index
  DataLayout.

### 6.2 Access and ownership contexts

- row producer with independently constructed values;
- identical-value fixed fill, reported separately from independent construction;
- field-only read, field-only write, and mixed row/column access;
- whole-value let/assignment, reuse, operation and user arguments, construction
  fields, return, and `give`;
- enum-payload match and `try` checker lanes, with native lowering explicitly
  excluded until separately priced;
- own, shared-deref, uniq-deref, index, and nested-projection place origins;
- complete-row replacement;
- field evolution from eligible to ineligible and back;
- nested construction under strict GRAM-9;
- invalid `move` of Copy, missing `move` of affine, extraction that leaves a
  hole, use after move, and borrow overlap.

### 6.3 Candidate-specific negative cases

`DECLARATIVE_COPY` must reject a field outside `RecordDomain`, generic
declaration in the experimental slice, recursive declaration, payload enum
field, and every attempt to use `move` on the resulting Copy value. Removing
`copyable` must make attempted bare duplication fail, not silently preserve
Copy. A marked outer record containing an unmarked inner record is a required
negative; marking both eligible declarations is the paired positive.

`AFFINE_FIXED_BUILDER` compile-time negatives cover an ineligible T; every
forbidden embedding or signature position; read, index, borrow, or `len` before
finish; move-to-binding; owned-argument passing; use after finish; double
finish; builder copy; builder as either side of `set`; row extraction; active
uniq borrow at finish; and every raw-tail access. Runtime underfill and over-push
cases must emit the registered deterministic report and abort. Normal
fallthrough, `return`, `break`, `give`, and ERR-3 `try` propagation with an open
builder must raw-free exactly once. The corresponding successful-finish cases
must transfer the pointer and later free the buffer exactly once. Complete-row
replacement must make the prior row dead once and cannot expose it.

## 7. Correctness and hostile soundness gates

Correctness is decided before timing. Findings have three disjoint outcomes:

- an independently demonstrated contradiction in the frozen bundle semantics is
  an intrinsic route falsifier;
- an implementation, harness, archive, build, identity, or provenance defect
  invalidates that artifact/campaign and supplies no comparative route evidence;
- a valid measured P0/W1 result is comparative evidence.

An invalid campaign never advances its rival. A repaired artifact receives a new
Lock B. A T1/T2 violation caused by the frozen semantics, rather than a
repairable implementation defect, is intrinsic.

Both arms must pass:

1. root `make check` and `make -C compiler check`;
2. complete candidate conformance in every entry of the closed ownership-context
   table, with checker-only lanes labeled as such;
3. at least 10,000 independently generated candidate-state programs checked
   against a separate dynamic ownership/initialization oracle;
4. ASan and UBSan native runs, plus MSan where a supported runner exists;
5. cross-target layout and overflow tests, including one 32-bit pointer-index
   target;
6. deterministic diagnostic and analysis-artifact byte identity;
7. three independent hostile reviews: ownership/state, target layout/codegen,
   and benchmark/attribution.

For `DECLARATIVE_COPY`, accepting one structurally ineligible declaration or
misclassifying one ownership context invalidates the artifact and triggers an
intrinsic review. If Lock A contains
a real nominal-affinity task, one accepted silent protocol duplication also
falsifies the bundle for that registered requirement. Without that
owner-ratified requirement, such a synthetic case
is reported as policy evidence rather than mislabeled as a memory-soundness
failure.

For `AFFINE_FIXED_BUILDER`, returning an incomplete buffer, exposing any tail
slot, accepting builder duplication/state reuse, leaking or double-freeing the
allocation on a normal execution, dropping a slot twice, or converting one
moved value into multiple rows invalidates the artifact and triggers an
intrinsic review. Trap-abort cases have no cleanup post-state and are excluded
from leak assertions.

## 8. Zero-tax and aggregate-code-shape gates

Before a new-feature result is considered, `CURRENT` and both candidate
toolchains compile the complete unchanged-source fixture set and current wfc.
For each candidate, all of the following must be identical to `CURRENT`:

- checker verdict and diagnostic bytes;
- raw LLVM bytes;
- optimized object and protected hot-function assembly bytes;
- call set, trap sites, alias metadata, and vectorization;
- existing primitive-buffer and SoA runtime representation.

No candidate declaration, builder runtime symbol, capacity field, branch,
drop loop, type test, or report hook may enter an unchanged program. This is
exact identity, not statistical non-inferiority.

For new-feature field-only paths, raw IR before SROA must contain a bounds
check, row address, field address, and scalar load/store only. An aggregate
load/store, `memcpy`, `byval`, or `sret` in a protected field-only path
invalidates that artifact; later scalarization does not cure the violation.

All semantic record-copy and affine-transfer sites and all backend
materializations are inventoried with the Section 4.1 many-to-many schema.
Dynamic execution counts reconcile attributable bytes with runtime counters.
Fieldwise scalar traffic that implements a whole-record copy or move remains
record-wide traffic; absence of an aggregate opcode is not credited as absence
of cost. Unresolved provenance is `indeterminate` and invalidates a protected
lane. Either candidate has an artifact failure for an unregistered semantic
transfer or backend record-wide transfer in a protected field-only or
field-update workload. Only Lock A-registered initialization and complete-row
replacement traffic is exempted, and it remains visible and measured rather
than relabeled as a defect. Physical load bytes and store bytes are reported
separately.

Semantic dynamic multiplicity comes from the independent oracle's exact event
trace over the same frozen source, input, and run identity. Backend
materialization counts come either from direct tracing of the scoring binary or
from a counter-instrumented clone derived only after scoring code generation is
frozen. After stripping counter operations and their private data, that clone
must have an identical machine CFG, basic-block/successor map, and
materialization-site map. Stable semantic-site and backend-site identifiers join
through the run/input identity. An unmatched site, block, edge, or path count is
`indeterminate` and invalidates the protected lane. Instrumentation never enters
the Lock B scoring binary; timings and machine-code claims come only from that
uninstrumented binary.

`AFFINE_FIXED_BUILDER` must finish as the existing two-word buffer and must have
no per-slot tag, bitmap, element drop loop, or hidden allocator path after
allocation. Every retained push/finish check is reported with its proof status
and machine-code site. Retention is a P0 result, not permission to elide it
without proof.

Analysis-reporting on/off must produce identical raw IR and machine code for
both candidate feature fixtures, not only for unchanged sources.

## 9. Performance and memory workloads

Timing begins only after correctness, hostile review, and code-shape gates pass.
Lock A contains an explicit contrast registry:

- unchanged programs compare each candidate to `CURRENT` by exact identity;
- common new-feature mechanism and writer lanes use the direct paired contrast
  `AFFINE_FIXED_BUILDER / DECLARATIVE_COPY`;
- the wfc-shaped lane uses that direct candidate contrast for route screening
  and reports each candidate versus current SoA descriptively for the later
  absolute capability gate.

Route selection never compares “significant versus CURRENT” for one arm with
“not significant versus CURRENT” for the other. All route inferences come from
the one direct paired candidate contrast. The endpoint registry contains two
workload strata.

### 9.1 Mechanism microbenchmarks

For every locked size and target-native layout:

- identical repeated initialization;
- independent per-row construction;
- construction followed by field-only scans;
- construction followed by whole-row traversal;
- replacement-heavy and read-heavy lanes;
- nested-record construction;
- separate construction, steady-state access, and destruction intervals.

The report records instructions, cycles, branches/misses, allocation/free
counts, requested/initialized/touched bytes, retained checks, aggregate-copy
bytes, `memset`/`memcpy`, code size, and peak/live primary memory. RSS alone is
not a primary memory metric. Atomic cells are guardrails unless their weights
were derived from the pre-implementation census and event trace; repeated fill
and independent construction are never pooled with preference-selected weights.

### 9.2 Frozen wfc-shaped lane

On the locked native 64-bit timing target, the exact TokenRow and AstRow logical
workloads use the existing 24- and 56-byte strides, current fixed capacities,
and identical field values. Correctness/codegen lanes derive and report their
own strides on 32-bit targets rather than reusing 24/56.
Only the two tape representations and their required candidate initializer may
differ. Parser algorithm, capacity policy, failure priority, facts setting,
allocator, optimization flags, and all other tapes remain identical.

This lane is a total-route comparison. It reports full eager initialization,
lexing/parsing access, cold frontend, retained frontend, and lifecycle
separately. It may not attribute a total difference solely to ownership,
locality, allocation count, or checks. A later full wfc layout migration still
requires the independent migration gate in `PROTOCOL.md`.

The original protocol's token-dense, AST-dense, whitespace-heavy, early/mid/late
malformed, and approximately 16 KiB/256 KiB/1 MiB corpora are mandatory. Cold,
lifecycle, and retained boundaries keep their exact definitions, and every
sample uses a fresh process where that protocol requires one. A source
transformation manifest proves that candidate sources become identical after
erasing only the route-required declaration marker, the exact registered
initialization region, and ownership-mandated `move` tokens. An AST-level
allowlist verifies that no candidate-only array or unrelated source difference
enters this lane. Semantic event traces and correctness digests are paired
exactly.

Lock A freezes sample count without observing a candidate result. It uses
existing `CURRENT` SoA/primitive-buffer noise with a preregistered conservative
variance inflation and requires at least 90% target power at every directional
or non-inferiority boundary. Insufficient realized precision yields `DEFER`; no
post-result extension is allowed. Final execution uses a balanced seeded order,
no discretionary outlier removal, and a stratified paired bootstrap with the
Lock A resample count and seed to form 99% confidence intervals. The registry
names one primary direct-candidate time family, one primary memory metric,
all guardrails, and family-wise or hierarchical multiplicity handling.

## 10. Benchmark-blind writer lane

This lane is blocked until the owner gives fresh experiment-specific permission
to send the task prompts, candidate teaching supplements, candidate code, and
correctness diagnostics to the external model service. The prior percent-decode
and utf8parse permissions do not apply.

The proposed writer is exact model `gpt-5.6-terra`, medium reasoning, default
service tier, one initial response and at most three sequential repair
responses per bundle/task. Lock A records the Codex client and adapter
versions and every remaining sampling parameter. There are no restarts,
parallel candidates, alternate seeds, best-of-N selection, human source edits,
or cross-arm repair hints.

Every bundle/task uses the repository's existing default-floor isolation
boundary: one ephemeral context, empty workspace, no repository access, one
accepted assistant response per round, and invalidation on unregistered tool or
filesystem use. Paired requests are counterbalanced and temporally interleaved;
request/model/service fingerprints are recorded. A service-identity change
aborts the campaign or creates a separately reported run identity.

Each paired task has one behavior contract and two independently presented
candidate-specific teaching supplements. The model never sees the rival
supplement, benchmark results, timing, profiler output, IR, assembly, copy
accounting, or performance hints. A repair receives only the frozen prompt, the
immediately previous complete answer, deterministic compiler/checker diagnostics,
and frozen correctness failures. Source freezes at first correctness-green.
After Campaign Freeze, the locked hidden corpus measures every exact frozen
generated source, including performance, semantic/physical transfer accounting,
retained checks, and code shape. Before that point, these artifacts are neither
generated nor visible to the writer, campaign operators, or anyone able to alter
requests or stopping. None of those results enters a repair prompt.
Hand-authored mechanism fixtures are ceiling evidence; only the frozen Terra
artifacts answer the default-code-shape question.

Task families cover 24-byte rows, 56-byte rows, nested records, field-only use,
whole-value use, field evolution, diagnostic repair, and exact-capacity
construction. Marker omission, invalid marker addition, missing/extra moves,
builder underfill/overfill, builder reuse, and accepted-wrong output are
classified separately. A real nominal-affinity task is included only if frozen
under Section 5; a synthetic token cannot be presented later as prevalence.

Hashes establish identity, not statistical independence. Lock A defines the
task-generation frame, the paired task as an observation, and the template
family as the resampling cluster. Correctness, repair rounds, and registered
ownership-policy edits all use an exact preregistered cluster-aware paired
randomization, permutation, or bootstrap method; a task-independent exact
binary test is forbidden unless there is exactly one paired observation per
template-family cluster. Power and multiplicity calculations use the number of
independent clusters, with 99% confidence and at least 90% target power at the
registered margins. Non-green tasks, zero/zero counts, and a zero denominator
have explicit uncensored handling; zero/zero supplies no directional win. An
underpowered convenience panel is a pilot and cannot select a bundle.
Primary writer endpoints are correctness within the repair budget, first-green
round, diagnostic categories, registered ownership-policy edits, hidden runtime,
and hidden transfer/check accounting. Source token count and readability are
descriptive only. Claims apply to the locked task frame and model run; they do
not become a population claim over models or arbitrary programs.

## 11. Decision rule

No prose preference, expert vote, or unregistered endpoint breaks a tie.

Let R be the paired metric
`AFFINE_FIXED_BUILDER / DECLARATIVE_COPY`, with lower better. Both directional
tests are transformations of this one paired contrast, never separate
significance tests:

- an affine time win requires the 99% upper bound of R to be `<= 0.90`;
- a declarative-Copy time win requires the 99% lower bound of R to be
  `>= 1 / 0.90`;
- the corresponding primary-memory boundaries are `<= 0.85` and
  `>= 1 / 0.85`;
- the registered repair/edit boundary uses `0.85` symmetrically, with the
  Lock A zero-count rule.

The verdict proceeds in this order:

1. An artifact/campaign invalidation yields no bundle verdict and cannot advance
   the rival. Repair requires a new Lock B.
2. An intrinsic contradiction in one frozen bundle removes only that bundle. If
   the other bundle has a valid campaign, it may advance to the later absolute
   capability experiment; it does not enter production.
3. If a real owner-ratified contraction requirement was locked and
   `DECLARATIVE_COPY` cannot both store the value and preserve that requirement,
   the declarative bundle is intrinsically inexpressive for that task.
   `AFFINE_FIXED_BUILDER` still must pass every registered P0 and writer
   guardrail before advancing.
4. With two valid bundles, one advances only when it has one registered
   directional P0 or weak-writer W1 win, the rival has none, writer correctness
   is within the registered paired non-inferiority margin, and every other
   runtime/code-shape guardrail is non-inferior. W1 here means only a Lock
   A-registered Terra correctness, repair, or ownership-policy-edit endpoint;
   static source size, token count, and expert maintenance opinion cannot win.
5. Conflicting directional wins return to owner review with the conflict
   exposed; they are not combined by arbitrary weights.
6. If neither bundle establishes a directional result, the decision is
   `DEFER`. Smaller syntax and cleaner theory remain measured evidence but do
   not replace R3's experiment under an unresolved tradeoff.

The result is always named for the complete `DECLARATIVE_COPY` or
`AFFINE_FIXED_BUILDER` bundle; it cannot claim that Copy or affinity wins under
a different initializer. It can intrinsically reject both bundles. It cannot
select automatic structural Copy,
an opaque initialized-prefix sequence, wfc migration, an AoS default, a PATTERNS
change, or production semantics.

## 12. Review and stop points

The protocol has six stop points:

1. independent ownership/state, layout/codegen, and benchmark/statistics hostile
   reviews of the exact proposed Lock A bytes;
2. owner review and commit of those unchanged bytes as Lock A;
3. independent hostile review of both semantic implementations, state oracles,
   and recoverable patches before Lock B;
4. correctness, soundness, code-shape, and provenance review followed by commit
   of Lock B before any external request or timing;
5. every per-task Run Freeze followed by Campaign Freeze before hidden
   evaluation;
6. blind recomputation from raw writer and timing logs before any bundle verdict.

Every completed lock, census, hostile-review disposition, and result is one
durable commit plus one appended `decision-gates.md` entry. No evidence remains
only in a disposable worktree.

Any change to a locked prompt, fixture, candidate semantics, eligibility
boundary, endpoint, margin, or exclusion after candidate implementation begins
invalidates scoring. A revised experiment receives a new lock and run identity;
it does not silently repair this one.

Advancement is only input to the absolute capability decision in `PROTOCOL.md`.
The surviving bundle must still establish a named P0, W1, or W3 delta over Rust
for R0; earn its construct under R1; satisfy current-SoA absolute and migration
gates; and receive an owner-reviewed AoS/SoA selection rule in PATTERNS before
production or default teaching. Production handoff also requires the full
META-5/spec/design-tree disposition. In particular, the built-in heap-owning
`buffer_builder<T>` changes STOR-1/STOR-3 even though no initialized-prefix owner
escapes. This protocol changes none of those artifacts.
