# Container-family probes

These probes ask whether current Whitefoot can execute representative container
operations. They are capability examples, not prevalence evidence or complete
container implementations. `make check` compiles and runs each supported
Whitefoot source in the default, `--par`, and `--no-overlap` modes, checks the
intentional source rejection, the owning-map allocation observer, and the
matched priority-queue and byte-growth C controls. The original measurements used compiler revision
`3cd7a8ebbc459b52989806442eff73538f131b96`; the added sources and retained samples
are in this directory. Checks and measurements were run on 2026-09-08, arm64
macOS 26.6.2, using Apple Clang 21.0.0 and Rust 1.98.1.

The whole-effect refinement at `ebc6d059` restores `hashmap.wf` and
`boxed-helper-gap.wf` with their original source and assertions in all three
execution modes. The known-endpoint candidate additionally restores
`boxed-migration.wf`: back insertion retains the last appended owner's logical
slot through the later literal replacements. The maintained `make check`
runner now executes all eleven sources in all three modes, the rejection control,
the owning-map allocation observer, and the priority/growth correctness controls; the growth control reports
4,608 variant/input checks. These recovery runs add behavior
evidence, not new timing samples to the recorded cost comparisons below.

## Operation contracts and recorded runs

`hashmap.wf` implements an eight-slot open-addressed table with
`Option<Entry>` payloads and a separate tombstone bit. Its pure
`map_trace(seed, repetitions)` covers colliding inserts, replacement, lookup,
removal, lookup across a tombstone, tombstone reuse, a missing removal, and
lookups after reuse. The three-mode native trace exits zero and independently
folds the expected operation results to `15521492327637928880` for seed 5 and
three repetitions. The keys and values are copyable scalars; non-copy payload
transfer, growth/rehash, hashing and equality interfaces, and allocation
failure are outside this probe.
`size` is supplied alongside the run and updated using explicitly wrapping
arithmetic. The trace supplies consistent values; this is not a verified
representation invariant for an arbitrary caller-supplied size. The `Full` return
is implemented but not exercised by this trace, whose maximum live size is three.

The map reads an affine option through a small borrowed helper. The measured
`find` takes and returns the unchanged run; its original motivation was OWN-6's
then-required one-statement child region. The current rule admits the separate
shared-option witness below. The map source retains its measured shape rather
than using that old restriction as a claim that shared lookup is inexpressible.

`owning-map.wf` adds an eight-slot table whose ordinary slot enum holds either
vacancy, a tombstone, or a key and modern `Box<u64>` owner. Its helpers perform
runtime linear probing through a complete array. Lookup borrows the array;
insertion and removal consume and return it. Sixteen seeds cover all eight
home positions twice, with collisions, duplicate replacement after a tombstone,
lookup past deletion, reuse, absent lookup/removal, and all eight slots full.
A full insertion returns the original offered owner. Deleting from that full
table and reinserting the returned owner under another key exercises the final
tombstone path after a complete scan with no vacant slot. Both the new value and
the removed key's absence are checked. These are ordinary source operations,
not fixed-position migration or a special compiler container path.

The 2026-09-10 runs use the published v0.55 compiler from `e3924d7f`, whose
source is unchanged at `ad624ab7`. All three ordinary compilation modes exit
zero. `owning-map-observer.c` additionally executes the no-overlap output once
normally and once for refusal at each of its 192 allocation requests. The
normal run allocates and releases 192 owners. Each refused run exits 70,
makes no later allocation request, and releases exactly the preceding owners.
Across all 193 executions the observer checks 18,528 allocations and releases,
1,456 returns of the original full-table input owner, and no live owner at exit.
It checks each pointer's original payload, detects duplicate/foreign releases,
and quarantines released cells until the execution ends so address reuse cannot
mask a stale owner.

The existing native Rust adapter's `--observe-allocations` mode only redirects
the LLVM allocator, release, and command-entry symbols after checking their
declarations. The observer supplies allocation refusal; the algorithm, failure
branches, and cleanup remain emitted Whitefoot code. This observation changes
allocator address reuse and supplies no throughput result. The source still
has fixed capacity, concrete scalar keys, one owning payload type, and the
ordinary enum layout. Growth/rehash, generic hash/equality invocation, sparse
layout cost, and fallible construction directly into a vacant entry remain
separate requirements.

`priority.wf` is a fixed-capacity indexed min-heap trace with scalar push and
pop behavior checked against both an independent sorting oracle and a matched
C heap. The measured source retains its `empty_heap()` initialization helper.
The former owner-routing prototype stopped when a fresh run's image changed
on a loop backedge; current checking derives stable origin headers instead.
The semantic loop controls and retained-call native tests cover changing owners,
without making that helper a language requirement.
Direct numeric postconditions carry length, room, and head changes across heap
operations without tracking global mutation history.

`ordered.wf` is a full-leaf B+ tree split component: it sorts the
candidate key, allocates both owning child cells before publishing the split,
returns duplicate without allocation, and preserves explicit refusal on either
allocation failure. It is not a complete ordered map or a claim about recursive
capacity.
Its executed cases insert 25 and 5 into `[10,20,30]`, and reject duplicate 20.
The source handles allocator refusal, but this experiment does not inject either
allocation failure and therefore does not dynamically validate those cleanup edges.

`boxed-migration.wf` exercises resource-owning slots in one concrete function.
It prepares colliding keys 1, 5 and 9 in four initialized optional slots, replaces
key 5's box, removes key 1, and reuses that exact extracted owner for key 13.
The setup, replacement, deletion and pre-migration inspection use selected
positions; they are not a general insertion/search implementation. A real loop
then traverses source indices, extracts each present `Entry<Box<u64>>`, computes
`(key +wrap step) % 8`, probes destination state and moves into the first empty
slot. The final owners and payload values are independently checked by destructive
extraction at their expected positions: key 9/value 30 at 1, key 13/value 10 at 5,
and key 5/value 11 at 6. The replaced value 20 was checked earlier.

An observation after two visited source positions reads both the remaining
source owner and an already-migrated owner, then the same loop continues. This is
not a returned pause state, reentrant helper, scheduler suspension or cancellation
test. The full-destination branch restores a pending entry to the source before
returning an error; this trace does not exercise that branch or inject any of the
four box-allocation refusals. Native success checks values and transfers, not an
independent allocation ledger. No address-stability or generic-library claim is
made. The duplicate shared `same` parameter on `view_box_entry` supplies a second
region occurrence for FORM-8; it is an authoring workaround, not a recommended API.

This extends the scalar map witness to actual runtime-indexed non-copy migration.
It does not complete the matched sparse-map cost comparison. Two initialized
`FixedVector` owners, separate u64 state arrays, and an ordinary option
discriminant are retained; no compact projected control layout is implemented.

`packed-page.wf` stores variable-length records as a byte length followed by
payload bytes in a fully initialized 32-byte run. It validates and hashes complete
records, prepends by shifting the used bytes backwards, and removes the first
record by shifting the remaining bytes forwards. Both shifts deliberately overlap
their source ranges. The hash includes each length byte and the ordered payload,
with independently calculated expectations for the written byte sequences.
The executed cases include two successive prepends, deletion/compaction, a
truncated record, an oversized page extent, insufficient spare capacity, a payload
shorter than its requested record, and empty deletion. Refused mutation leaves
the input owner and used extent available; ordinary invalid-input branches supply
the bounds facts before any indexed access. `remove_first` validates the first
record only, and `prepend` preserves the existing bytes without validating their
format. Neither claims that arbitrary input becomes a wholly validated page.

This is a byte-codec and movement witness, not the SQLite or listpack format, a
typed-record overlay, an optimized bulk-copy result, or a growing page. The full
32-byte initialization and run descriptor are retained source costs. Validation
is repeated on each scan; there is no exported validated-view capability. This
probe shows that variable byte-record lengths and overlap movement alone do not
require arbitrary typed vacant storage. It does not price those operations.

## Deliberate boundaries and failures

`rejected-wrapper.wf` records the active FN-9 restriction on measures over a
field-selected result place. Its `len_of(table.slots)` postcondition is rejected
with `InvalidPostconditionSelector`; the direct-run map contracts therefore
publish `len_of(result)` instead of hiding the run in a nominal wrapper.

`shared-option-view.wf` retains the formerly rejected lookup body: one region
binds the result of a child reborrow call, then returns it in a second statement.
OWN-6 now ends that temporary argument loan at the call statement; the region
can contain both statements. The witness adds a command checking both present
and absent entries and belongs to the three-mode native-success loop. The old
file had no command and now reaches FN-7's missing-entry rejection instead of
its former OWN-6 rejection; that is not evidence against the revised region
rule. This supersedes the old negative expectation on its amended semantic
ground, preserving the original lookup and helper bodies.

`boxed-helper-gap.wf` is the nested helper-composition witness in the ordinary
three-mode native loop, which also executes the restored boxed migration.
`compose` passes two boxes with the same declared store
region to `build`, then passes its returned run and an entry made from its returned
box to `replace_one`. FN-2 substitution now reaches the nominal element under
`FixedVector<Option<Entry<'s>>, 1>` in call arguments, results and contract goals.
The former TYPE-5 discrepancy with identically printed types is repaired through
that structural substitution. FORM-8 therefore infers regions which occur in such
nested input positions; the four migration-helper calls no longer explicitly
write that inferable argument.

The command allocates payloads 11 and 22 and calls these helpers.
The public `compose` postcondition supplies the length fact required to extract
the updated entry. Its recorded execution checks the returned old payload is 11 and the new
payload is 22, then ordinary cleanup releases the owners. It is not a generic
map, and this family runner does not inject allocator refusal or count releases.
`make reproduce-region-gap` invokes the same maintained runtime witness.

`ordered-runtime-gap.wf` now belongs to the ordinary native-success loop in all
three modes. Its source expectation is unchanged: exit zero after successful
allocations and exit 70 on allocation refusal. The former compile-only exception
is removed; `make reproduce-runtime-gap` remains a direct invocation of the same
runtime witness. The family runner does not inject allocation refusal here.

The original defect was `replace deref(tree) = move replacement` through
`tree: &uniq Box<RuntimeGapNode>`. The callee received the box's object pointer,
freed the displaced object, and never updated the caller's owner. Box borrows
now address the owner's pointer slot through the general typed-place path.
Explicit nested dereferences also read that slot before reading the object.
Compiler execution tests separately check replacement through root and field
borrows, returned borrows and reborrows, with retained helper boundaries and an
allocation observer requiring each cell to be released once. A borrowed match
inside an owning tree cell also projects two Box payloads from the actual enum
storage. Replacing the second payload of the second variant preserves the first
payload, and the observer checks PROV-6's field-order cleanup after releasing the
displaced child. A separate borrowed Buffer payload case retains that type's
existing value ABI. No new tree operation or source-language rule is introduced.

The compiler's independent Box tests also exercise one unbounded helper across
general and extent stores. Nested owners pass through identity, reversed
two-region aggregates, ordered multi-results and explicit cell destructuring.
Retained calls and allocation observers check values and release order in all
three lowering modes, including allocation refusal. The implementation checks
the source declaration once under its existing fail-closed rules, then selects
physical function and type instances from a finite release-class environment.
This repairs representation and cleanup; it does not weaken source ownership
or effect requirements and adds no runtime class dispatch.

The modern Arena Box staged test separately exercises actual worker grants,
forced inline fallback and delayed publication/join. Capacities 64, 48 and 40
bytes cover success and failure at either allocation in a pair; earlier tasks
must retire before the enclosing storage leaves scope. The corresponding Heap
loop remains sequential under PAR-3 because allocation and cleanup write its
same external provider. These are lifecycle witnesses, not parallel-container
throughput measurements.

The `general_run_elements` native cases in
[`backend/tests/arrays.rs`](../../../../compiler/src/backend/tests/arrays.rs)
exercise deeper composition through the ordinary source path: three nested
owning runs cross retained generic helper calls, the same store-polymorphic
helper accepts nested modern Boxes from both Heap and Arena, and a recursive
node owns a `Vector` of nodes. Allocation observers check element-before-backing
cleanup, exact Heap releases, no Arena free, and both allocation refusal points
in the recursive constructor. An array-valued element also supports nested
indexing, replacement and its standing length, capacity, head and room facts.
Interned complete element types remove the
compiler's former one-level representation limit; descriptor layout and the
ownership release graph are separate traversals. These cases do not generalize
the legacy full-array element rule, remove aggregate transfers, or measure a
throughput change.

`make measure` writes new priority samples under `.build/`. The retained CSVs
beside the sources are reviewed observations, not regenerated by `make check`.

## Binary heap cost control

Each round generates 16 integers in `[0,97)` from a runtime seed, inserts them
into an initially empty min-heap, then pops all 16 and folds them into a checksum.
The next round continues the same random recurrence. `priority.wf` uses verified
length/room/head contracts and ordinary consume/return helpers; its heap always
has head zero. It performs the same sift-up and sift-down algorithm as
`priority-costs.c`. The native heap declares the same 16 payload words and two
descriptor words, although native optimization may remove unused metadata.
Both sources initialize their fresh heap storage. No heap allocation occurs.

An independent `qsort` oracle checks 64 seeds at round counts `{0,1,2,7,16}`:
320 input pairs for both Whitefoot and each native control. The sort is not timed
as a heap competitor. The Whitefoot command also directly checks sorted values,
duplicates, and reuse after emptying. These are scalar, fixed-capacity witnesses;
generic comparison dispatch, resource-owning payloads, larger heaps and arbitrary
interleaved updates are outside the comparison.

`abi.rs` checks and changes only two linkage declarations: export
`wf_priority_trace(u64,u64) -> u64` and rename C `main` to `wf_fixture_main`.
It changes no source judgment, LLVM instruction or optimization attribute. The
compiler emits with `--no-overlap`; both emitted IR and C are optimized with
Clang `-O2`, without LTO. The normal C control permits helper inlining; a second
build sets `RETAIN_HELPERS` to keep the pointer-based C push/pop calls. The
external Whitefoot trace and the native trace remain distinct direct calls in
the batch loop, and varying seeds plus observable checksums retain their work.

Each sample warms its own variant with 256 calls, then measures 4096 calls with
seeds 19 through 4114. Seven samples alternate variant order. Timing uses
`CLOCK_MONOTONIC` (QPC on Windows); the recorded host is macOS only. The two
control executables ran separately, with no CPU affinity, frequency control or
exclusive host isolation. Compare Whitefoot and C within the same run. The
one-round boundary-control samples have visible drift and are not evidence for
small differences between the C controls.

Median nanoseconds per complete trace call; ranges are sample minima/maxima,
not confidence intervals:

| Control run | Rounds | Whitefoot median (range) | C median (range) |
| --- | ---: | ---: | ---: |
| Ordinary helpers | 1 | 845.95 (811.77–897.71) | 277.83 (263.92–298.34) |
| Ordinary helpers | 16 | 13,078.61 (13,043.70–13,244.38) | 4,371.83 (4,326.17–4,442.87) |
| Retained C helper calls | 1 | 1,068.85 (953.13–1,280.52) | 392.58 (299.07–473.88) |
| Retained C helper calls | 16 | 13,196.04 (13,008.79–13,297.12) | 4,328.61 (4,304.69–4,410.40) |

The original ordinary run shows roughly three times the native elapsed time at
this size. Its optimized Whitefoot IR retains push/pop calls and nine 16-byte load/store pairs
after each call to move the returned 144-byte run back to its owner; callee
argument/result transfers remain as well. The native control accesses the heap
through its existing destination. The retained-call control shows that ordinary
helper calls need not force those complete-run copies. It does not isolate all
ABI, addressing, instruction selection and inlining effects, so the timing gap
cannot be attributed exclusively to copies. No new layout or source primitive
was needed to admit this program.

Retained observations are `priority-measurements.csv` and
`priority-boundary-measurements.csv`. Reproduce checks and generate fresh samples:

```sh
make -C research/experiments/container-representation/families check
make -C research/experiments/container-representation/families measure
```

Inspect optimized Whitefoot storage from this directory:

```sh
clang -O2 -Wno-override-module -x ir -S -emit-llvm .build/priority.ll -o .build/priority.opt.ll
rg -n 'define.*wf_priority_trace|call.*wf_(push|pop)|load <2 x i64>' .build/priority.opt.ll
```

### Consumed input and result destination reuse

The storage planner now considers an ordinary call's whole result for reuse of
one consumed, same-typed aggregate binding. This requires checked source-call
ownership, one eligible input, dead prior content under complete CFG liveness,
and no exposed backing. The current callee ABI snapshots its aggregate inputs
before any body or result write. Source `own` by itself therefore does not grant
aliasing permission; the lowering order and liveness jointly justify this case.
May-suspend callees, overlap/completion schedules, ambiguous inputs and ordered
multi-results retain separate storage.

In the optimized heap trace, `wf_push` now receives the same address for input
and result, removing the caller's 144-byte transfer after each push. `wf_pop`
returns both the new heap and removed element, so its result remains separate
and the caller transfer remains. Callee entry and result snapshots remain too.
This is a bounded reduction of an observed compiler cost, not completion of
aggregate transfer optimization or a new in-place source API.

Both matched controls again pass the independent 320-input oracle. The two
`priority-coalesced*-measurements.csv` files retain another 56 samples under the
same harness, compiler flags and host conditions. A subsequent compiler rebuild
including the Box borrow repair produced identical optimized heap instructions
(only the diagnostic ModuleID path differed). Median nanoseconds per trace:

| Control run | Rounds | Whitefoot median (range) | C median (range) |
| --- | ---: | ---: | ---: |
| Ordinary helpers | 1 | 767.09 (757.08–819.09) | 267.33 (260.99–283.20) |
| Ordinary helpers | 16 | 12,442.14 (12,233.15–12,769.53) | 4,426.27 (4,334.47–13,920.17) |
| Retained C helper calls | 1 | 798.10 (781.25–811.77) | 278.08 (269.29–290.28) |
| Retained C helper calls | 16 | 12,180.66 (12,165.77–12,330.57) | 4,283.45 (4,273.44–4,423.34) |

The 16-round Whitefoot medians are still about 2.8 times the matched C medians.
The ordinary C run contains a large outlier, and neither old nor new sampling
used exclusive host isolation. These timings cannot establish a small causal
speedup over the earlier run. The removed caller copy is directly visible in IR;
the substantial remaining performance gap stays open.

### Multi-result destination experiment

The discriminating control recorded at `8bc22df5` keeps `priority.wf` and the complete `wf_pop`
callee unchanged. Its baseline caller keeps separate storage for a 144-byte
heap and a 152-byte `(heap, value)` result, then transfers the result's first
field back to the heap after every pop. The candidate gives the heap the first
field of one complete, correctly aligned 152-byte result allocation and uses
that allocation for the call result. It must never place a 152-byte result in
a 144-byte allocation. The callee's existing input snapshot must finish before
any result write, and the old heap is consumed at this ordinary synchronous
call. No view, deferred call, or new source acceptance is part of this control.

The experiment changes only the caller's allocation and its two address
definitions in a copy of the emitted LLVM. It retains the original transfer
instruction so ordinary LLVM optimization, rather than deletion by the
experiment, must recognize the identical addresses. Its selection criterion is
that both existing independent 320-input sorting checks still pass, including
the extracted scalar, and retained optimized code loses the caller's 144-byte
post-pop transfer. Callee instructions and all other functions must remain
byte-identical before native optimization. A successful result establishes a
placement opportunity for this consumer, not an implemented compiler feature,
an admissible general alias rule, or a timing improvement. General result-field
placement still needs CFG liveness, address-exposure, input-alias, and deferred
retirement evidence before it can replace separate compiler storage.

The control met that criterion on 2026-09-10 with the rebuilt v0.56 compiler
working tree based on `13cc6ca8` and Apple Clang 21.0.0, `-O2`, arm64 macOS.
Both unmodified controls and both adapted controls passed their 320-input
checks. The adapter verified that reversing exactly the three changed lines
restored the entire input module, including the unchanged `wf_pop` body and
post-pop `memmove`. Both optimized traces retain all 16 `wf_pop` calls per
round. The original trace has four 32-byte load/store pairs and one 16-byte
load/store pair after each of the first 15 calls; LLVM already removes the
final dead-result transfer. The adapted trace reads only the returned scalar
before updating its checksum, so those 15 additional 144-byte post-pop
transfers are absent. The trace's
explicit stack adjustment changes from 416 to 272 bytes. This is static
instruction and frame evidence, not elapsed-time, memory-traffic, or whole-call
stack high-water evidence; the callee's existing transfers remain.

Reproduce the three-line control with the pre-field-placement compiler at
`8bc22df5`, using its ordinary exported `.build/priority.ll` from the targets
above. The current compiler already selects field placement, as described in
the next section. In a separate copy of the baseline, replace only this prefix of
`wf_priority_round`:

```llvm
%wf.frame = alloca { { [16 x i64], i64, i64 }, %wf.t3 }, align 8
%wf.slot.0 = getelementptr inbounds { { [16 x i64], i64, i64 }, %wf.t3 }, ptr %wf.frame, i32 0, i32 0
%wf.slot.1 = getelementptr inbounds { { [16 x i64], i64, i64 }, %wf.t3 }, ptr %wf.frame, i32 0, i32 1
```

with the complete result allocation and its two selected addresses:

```llvm
%wf.frame = alloca %wf.t3, align 8
%wf.slot.0 = getelementptr inbounds %wf.t3, ptr %wf.frame, i32 0, i32 0
%wf.slot.1 = getelementptr inbounds %wf.t3, ptr %wf.frame, i32 0
```

Here `%wf.t3` must still be `{ { [16 x i64], i64, i64 }, i64 }`; do not apply
the transformation if the input frame, type, or snapshot protocol differs.
Build the copy with the same `priority-costs.c` commands as the original,
once normally and once with `-DRETAIN_HELPERS`, and run each executable with
`check`. Emit assembly for both modules with `clang -O2 -Wno-override-module
-x ir -S` and compare the retained `wf_priority_trace` pop sequences. No timing
samples were taken for this control.

### Compiler-selected result fields

The compiler now selects the complete-result placement through its general
storage planner. A synchronous call may place one consumed aggregate input and
its consumed result field in that field of the full result allocation. The
parent has only distinct consuming projections in the call's block. Existing
coalesced groups undergo complete CFG interference checking, with only the
particular call's input read and selected consuming projection admitting the
overlap. Exposed storage and deferred calls remain excluded. Field offsets come
from the actual struct type, and a returned child keeps its normal copy into
the caller-provided result. The choice does not require a new source
rule, an input/result value-equality proof, or a runtime alias test.

On 2026-09-10, an isolated tree matching `e3924d7f` used the ordinary build
targets above to regenerate both priority executables from unchanged
`priority.wf`; both independent 320-input checks
passed. The emitted round owns one 152-byte tuple allocation, with its heap in
field zero. The original 144-byte transfer instruction remains between equal
field addresses. Apple Clang 21.0.0 at `-O2` removes the same 15 post-pop copies
identified by the earlier control and retains all 16 pop calls. The optimized
trace's explicit stack adjustment is 272 bytes, versus the baseline's 416.
The exported LLVM SHA-256 is
`d4456a94e333ee2c9fb258062bcb0f0bc21a865767b10168d8302ba14b05f720`;
the preserved baseline is
`99acd8a67620facad1da6dbd3dcfcebf23b22dcfd7279ba813b73abd9a875211`.
These are compiler-selected placement and static code results, not new timing
samples. The callee's entry snapshot and internal movement remain.
This isolated build carries the published v0.55 source rules. Its LLVM differs
from the earlier developing v0.56 build only in the qualification-version
comment, and the native assembly is byte-identical.

The native owned-place controls also execute a three-result tuple with an
aggregate between a byte and a 16-bit scalar, preserve both siblings, return
the smaller aggregate through another helper, and repeat the call zero and
three times. A separate replacement control extracts an old array, sends it
through a multi-result helper, then verifies it remains independent of writes
to the original containing place. Arrays are affine; this control uses
replacement rather than a nonexistent implicit array copy. Both controls run
with normal and retained helper calls in all three lowering modes. Planner
controls check the full parent extent and preserve unrelated conflicts and
exposed values. Cross-block parent uses, repeated field reads, whole-parent
uses, and nested or conflicting placement components remain unresolved by this
selection and retain their independent backing.

The timing follow-up uses the unchanged two heap controls and their existing
seven-sample protocol against both preserved pre-field-placement LLVM and the
published v0.55 compiler output. Alternate baseline/current executable order
across complete cohorts. Every invocation retains the independent sorting checks.
Compare each Whitefoot trace with its same-run C control, and retain every
sample rather than selecting a stable-looking subset. The native copy difference
is the causal evidence for placement; timings can bound the remaining gap but
cannot assign it wholly to copying or establish small changes under host noise.

The 2026-09-10 follow-up retains all 224 samples in
[`priority-result-field-measurements.csv`](priority-result-field-measurements.csv).
Two complete cohorts reverse baseline/current order; each executable still
alternates its seven Whitefoot/C sample pairs at one and sixteen rounds.
The baseline LLVM is the preserved `99acd8a6...` module above, and current is
the published v0.55 `d4456a94...` module. Both are linked with the unchanged
`priority-costs.c` and the same Clang flags; every invocation passes its sorting
oracle. Fourteen samples contribute to each median and range below.
Nanoseconds per sixteen-round trace:

| Compiler placement | C helper mode | Whitefoot median (range) | C median (range) |
| --- | --- | ---: | ---: |
| Before result fields | Ordinary | 13,582.40 (13,164.06–24,351.32) | 4,487.43 (4,357.91–8,604.74) |
| Complete result fields | Ordinary | 12,961.55 (12,651.86–13,352.54) | 4,545.41 (4,414.06–4,739.75) |
| Before result fields | Retained | 14,529.05 (13,574.95–29,800.29) | 4,669.56 (4,496.83–11,422.61) |
| Complete result fields | Retained | 13,162.60 (12,717.29–22,355.22) | 4,560.55 (4,381.10–6,911.87) |

Current Whitefoot medians remain about 2.85 and 2.89 times their same-run C
medians. The ranges expose substantial host noise, including simultaneous
baseline/C slow samples; retain these observations rather than interpreting a
small before/after difference as a reliable causal speedup. Removing the caller
copies is established by the native code comparison, while the remaining
same-algorithm cost still needs a representation and lowering explanation.

### Borrowed full-array heap

The scalar-heap comparison also permits a source representation
competitor: a full array borrowed by each mutation helper, with its logical
count passed and returned as an ordinary scalar. Keep the random recurrence,
all sift operations, duplicate handling, empty/reuse behavior and final checksum
unchanged. Checked count contracts must admit every partial operation without
impossible-case control flow. Count full-array initialization and all retained
helper costs rather than assuming a borrow is free. This tests a scalar dense
representation and source API; it cannot establish spare-storage construction
or a reusable heap for resource-owning elements.

An additional symmetric call-boundary control marks only the Whitefoot push
and pop helpers `noinline` and uses the existing retained C helper build. Its
sorting oracle and operations remain unchanged. Check whether aggregate
transfers reappear when the helper boundary cannot disappear; this separates
the borrowed storage contract from a win relying entirely on helper inlining.

The ordinary source competitor is
[`priority-borrowed.wf`](priority-borrowed.wf). On 2026-09-10 it executes the
command's duplicate/sorted/reuse checks with default, `--par` and `--no-overlap`
lowering on the published v0.55 compiler. All three matched control executables
pass the same independent 320-input sorting oracle. No compiler or source rule
change is needed. The local invariants after let-bound calls preserve the
returned count relationships before the mutable count binding is overwritten;
they add no executable checks.

The LLVM linkage adapter remains unchanged for the scalar trace. The symmetric
retained target additionally checks and marks exactly the two Whitefoot mutation
helpers `noinline`. Both forms have no whole-array transfer in the heap operation.
Ordinary optimization inlines the Whitefoot helpers; the symmetric control keeps
their calls and still has no aggregate copy. The ordinary trace reserves 176
stack bytes including saved registers, and the symmetric trace reserves 272.
The compiler still emits 128 bytes of array zero-initialization each round and
scalar result stores. Both WF's array constructor and C's partially written
aggregate initializer semantically initialize all array elements; optimization
of that initialization and count/result storage can differ. The WF count is a
scalar passed and returned across calls, whereas the C count is a heap field.
The comparison is of usable source representations, not identical ABIs.

Two cohorts for each control retain 168 timing samples in
[`priority-borrowed-measurements.csv`](priority-borrowed-measurements.csv), using
the same warmup, seed range, seven alternating sample pairs, Clang flags and host
conditions as the previous heap comparison. Each median/range contains fourteen
samples. Nanoseconds per complete trace:

| Helper boundary | Rounds | Whitefoot median (range) | C median (range) |
| --- | ---: | ---: | ---: |
| Ordinary optimization | 1 | 263.67 (258.54–285.64) | 266.97 (261.96–293.46) |
| Ordinary optimization | 16 | 4,216.43 (4,196.53–4,352.78) | 4,285.64 (4,260.50–4,434.08) |
| C helpers retained | 1 | 265.01 (257.08–272.95) | 272.46 (265.14–284.91) |
| C helpers retained | 16 | 4,196.53 (4,182.86–4,321.04) | 4,279.17 (4,267.09–4,415.77) |
| WF and C mutation helpers retained | 1 | 266.85 (262.45–270.51) | 269.17 (265.14–270.51) |
| WF and C mutation helpers retained | 16 | 4,348.27 (4,237.79–4,491.70) | 4,377.44 (4,273.93–4,586.67) |

This scalar, sixteen-element fixed heap has close matched native costs even
with the mutation calls retained. The sample ranges do not establish a small
advantage over C. The earlier roughly threefold gap therefore does not establish
that this operation needs a new storage primitive: the existing array and borrow
model can express an efficient competitor. It also does not excuse unnecessary
transfers in the owning-run API. General element construction, generic comparison
behavior, growing heaps and resource-owning payloads remain untested by this
competitor. The maintained family `check` and `measure` targets include all three
new controls while retaining the original owning-run tests and comparisons.

### Run-helper transfer attribution: comparison criterion

The next comparison uses the current `priority.wf` output and the unchanged
`priority-costs.c` harness, with both mutation helper boundaries retained.
An experimental LLVM adapter will give the callee's working run the consumed
input/result storage directly. It must change only storage placement and
whole-value transfers, preserve sift operations and scalar results, retain
inline stack storage, and pass the existing independent sorting oracle. The
adapter is an experiment, not compiler lowering or source-language admission.
Its fail-closed shape checks must reject a changed input rather than silently
transform a different program.

Before timing, read the remaining transfer byte counts from optimized IR.
Compare baseline and adapted output with the same seeds, rounds, warmups and
alternating samples as the retained C control; reverse executable order in a
second cohort. Retain all samples. Their difference estimates the cost of this
storage/transfer intervention, including its optimizer interactions, rather
than the isolated latency of a memcpy. The adapted-to-C difference is the
remaining cost under this placement contract. The existing retained borrowed
full-array witness is a separate usable representation control. A successful
intervention near C would make a checked placement contract worth proposing;
otherwise the remaining non-transfer instructions need to explain the gap.
Neither outcome establishes a universal floor or selects a language amendment.

#### Measured transfer intervention

On 2026-09-10, compiler `8482c0cb` (unmodified compiler sources), Rust 1.98.1,
Apple Clang 21.0.0 at `-O2`, and arm64 macOS 26.6.2 produced the following
observations. No affinity, frequency lock or exclusive host isolation was used.
[`priority-placement.rs`](priority-placement.rs) checks the complete emitted
push, pop and round bodies against the reviewed witness, including guards which
reject changed tuple extent and changed heap arithmetic. It emits two modules:
the current code with push/pop explicitly `noinline`, and a copy whose timed
round calls two cloned helpers with a guaranteed shared input/result address.
The original helpers and command fixture remain unchanged in the latter.

The clones redirect ten local storage addresses to the result place, retain
every sift instruction and transfer instruction, and remove three tuple clears
already absent from the baseline optimized IR. Each cleared tuple was completely
overwritten by its run field and scalar field before any read. Retaining those
clears after co-location would erase live input. Push saves the entering length
before incrementing it; pop saves the minimum and last value before replacing
their storage. Pop receives the caller's **complete 152-byte result allocation**,
not a 144-byte run masquerading as a larger object. Its run lives in field zero;
the scalar at offset 144 is consumed before the next call. No allocation,
pinning, suspension or new source-language admission is added by this model.
Ordinary LLVM optimization removes the resulting self-transfers; the adapter
does not delete memcpy/memmove instructions or rewrite the heap algorithm.

Optimized-IR transfer counts per executed mutation call:

| Contract | Push | Pop | Caller post-call run transfer |
| --- | ---: | ---: | ---: |
| Current owning run, retained helpers | 4 x 144-byte memcpy + 128-byte vector loads/stores + 16-byte metadata stores = **720 B** | 6 x 144-byte memcpy = **864 B** | 0 B |
| Guaranteed same-place run model | **0 B** | **0 B** | 0 B |
| Existing borrowed full array, retained helpers | **0 B** | **0 B** | 0 B |

The push vector pairs copy all sixteen elements; two scalar stores copy the
length and head into the next local run. The separate element insertion and
sift writes are not transfer bytes. Each round executes sixteen pushes and
sixteen pops, so the baseline accounts for 25,344 transferred bytes per round,
or 405,504 per sixteen-round trace. These count explicit IR transfers, not
cache misses, bus traffic or twice-counted read-plus-write bytes. The native
code retains the copies. Baseline push/pop explicitly reserve 448/1024 stack
bytes; placed helpers reserve none. The timed caller still reserves 272 bytes
in all three retained variants. These are observed frame adjustments, not a
general stack-envelope proof. Construction and ordinary element writes remain.

[`priority-interface-measurements.csv`](priority-interface-measurements.csv)
retains all 224 samples from two reversed-order cohorts. The unchanged harness
warms 256 calls, times 4096 seeds (19 through 4114), and alternates seven WF/C
pairs at one and sixteen rounds. Each executable first passes the independent
320-input sorting oracle. Each table cell below has fourteen samples; units are
nanoseconds per complete sixteen-round trace, and parentheses are min/max,
not confidence intervals.

| Contract | WF median (range) | Same-run C median (range) |
| --- | ---: | ---: |
| Original priority.wf / ordinary C control | 12,840.45 (12,623.05–12,972.66) | 4,509.77 (4,382.08–4,641.85) |
| Current owning run, both helper boundaries retained | 12,813.72 (12,713.87–12,982.91) | 4,484.86 (4,384.52–4,572.51) |
| Guaranteed same-place run model, boundaries retained | 5,504.64 (5,460.94–5,918.46) | 4,473.51 (4,380.37–4,650.63) |
| Existing borrowed full array, boundaries retained | 4,450.32 (4,355.71–4,523.68) | 4,501.83 (4,401.37–4,604.74) |

The retained baseline gap is 8,328.86 ns. Placement reduces WF time by
7,309.08 ns; adjusting for the same-run C medians gives 7,297.73 ns, or
**87.6% of the gap**, leaving **1,031.13 ns** (placed WF is 1.23 times C).
The two cohorts separately attribute 87.7% and 87.5%. This is a measured
storage/transfer intervention: it includes smaller callee frames, one fewer
pointer argument and optimization consequences. It is not an isolated memcpy
latency measurement. Original and explicitly retained WF medians differ by
about 0.2%, below the sample ranges; forcing the boundary is not the large loss.

The 5.50 microsecond result is an attainable zero-transfer cost for this run
contract under the measured lowering, not a proof that its global optimum is
5.50 microseconds. All run descriptors, window addressing, scalar result stores
and construction remain. The placed IR still reloads the head and forms wrapped
indices; the C control uses direct array indices. Those are concrete remaining
differences, but this experiment does not causally divide the residual between
addressing, alias analysis, scalar ABI and instruction selection. The borrowed
array reaches 4.45 microseconds while keeping storage inline and boundaries
retained; it changes the representation contract and cannot isolate transfer
cost or establish a solution for uninitialized owning elements.

Reproduce with `make check-interface` and `make measure-interface` here. The
former belongs to `make check`, whose ordinary WF loop retains all three
lowering modes. The counterfactual and timings use `--no-overlap` output.
Only `measure-interface` writes fresh CSVs, under `.build`; neither target
regenerates the retained observations. `priority-placement --inspect` reports
copy intrinsic and vector-load counts from the generated `.opt.ll` files and
requires the placed helpers to have retained calls, no aggregate transfers,
no tuple clears and no local allocations. The scalar metadata stores above
were read separately from the optimized push body.
The exported baseline/placed LLVM SHA-256 values are respectively
`c6df0cd442b291eee9650b931b477f13f75273effeedd5c6fd6400e732a2cd88` and
`74c1ebd68ebe29b3fc08640d4d3f2e1c5bace6a0231695b9cda66483a1054ad6`.

#### Interface recommendation — proposal for owner decision

**Propose a checked same-place contract for consumed run input/output.** Keep
the source value and proof interface of `set x = f(move x, ...)`: the callee
owns the entering value, and its returned run denotes the new value. Permit an
explicit, statically checked contract to require that this input and one
selected output occupy the same storage throughout the call. At an admitted
call, a whole-run transfer at that boundary or along the declared update chain
must not be a hidden fallback. Calls that cannot meet the guarantee need a
different explicitly permitted interface. This is a proposed guarantee and
checking obligation, not syntax or semantics this branch implements.

The checker/lowering must establish one eligible consumed input, the selected
return correspondence, no live overlapping loan, and no later use of an old
value after its storage is reused. Scalar entry images may be saved before
mutation. For multi-results, the allocation must cover the complete result,
with each live sibling preserved; a 152-byte result cannot inhabit a 144-byte
object. If two independently live versions are required, they cannot share
this place. Entry snapshots, intermediate construction and return lowering
must all respect this obligation; caller destination coalescing alone is
insufficient. The witness is synchronous and does not establish address
stability across a staged call or retained I/O loan.

**BLK-4 and MSR-3 need no change for the measured own/return shape.** Existing
`requires`/`ensures` retain the entering measure datum and publish the returned
run's length. A guaranteed-place feature would need its own placement and call
rules, including how failure is diagnosed; it must not redefine `own` as
universally immovable or universally free to copy. Which return relationships
are expressible and cheaply checkable beyond this witness remains open.

Do not select merely lifting BLK-4's `&uniq` run refusal as the solution.
A conservative kill can prevent stale length/head facts, but MSR-3 still forbids
an `ensures` measure over that exclusive formal. Returning a scalar count alone
does not establish that count equals the run's post-call length. Repeated
push/pop proofs therefore need a post-state contract design or intentional
runtime-domain handling, not impossible error branches added for the checker.
The borrowed full array avoids this problem because its extent is type-fixed;
it is a useful current option for initialized scalar storage, not evidence
that variable initialized windows are unnecessary.

The grounds are the 87.6% attributed gap and the attainable 5.50 microsecond
run cost without heap allocation. Remaining evidence: larger runs and varied
capacities, affine/linear elements and cleanup, alternative/fallible results,
aliasing and independent old-value controls, generic helpers, and other native
targets. No language amendment, general placement implementation or growing
container result follows from these timings. The owner can choose whether this
bounded contract or an explicit mutable post-state design is the next proposal
to develop; this experiment does not make that language decision.

## Explicit byte-run growth and refusal

`growth.wf` allocates and fills an initial byte run, then either keeps it, refuses
additional growth above a caller limit, or allocates a larger run, copies the
prefix and fills the suffix. It hashes every resulting byte and releases the
backing. Initial allocation refusal returns zero; growth allocation refusal
hashes the still-readable original bytes with a distinct status. The limit
applies only to additional growth: initial allocation and the no-growth path
can succeed with `limit < initial`. This is an intended policy, not an impossible
error arm added to satisfy proof. There is no growth or raw-pointer language
primitive hidden in the source.

The failed-growth path establishes preservation and cleanup *inside this trace*.
The scalar result does not establish a reusable library helper returning the old
owner and its capacity facts to a caller for retry. It also does not model
non-copy relocation, a wrapped source, inline/spill transitions, or outstanding
borrows during growth. Sparse migration has different obligations.

`growth-costs.c` compares the same reserve/fill/grow/digest/free contract using
a source loop, `memcpy`, or `realloc`. Each retains a trace call; shared native
code and digest may inline. Variant selection occurs outside the timed batch
loop, which makes direct calls. An independent oracle generates the logical
byte sequence without allocating or copying. Sixteen seeds, four initial sizes
`{1,16,257,4096}`, two requested extents, three limits and three allocation
refusal positions check 4,608 implementation/input combinations. The ledger
also checks request count, no surviving backing and logical live extents.
First- and second-request refusal are injected equally in all implementations.

`growth-abi.rs` is a private experiment adapter. It verifies the emitted signature,
exports the scalar trace, renames the fixture main, and redirects allocator calls
to the native ledger. It changes no acceptance rule or executable instruction
other than those symbol targets. After optimization it requires the Heap
parameter to have `readnone` and `nocapture`/`captures(none)` attributes before
linking a harness that supplies a placeholder. The current trace does not use
that parameter. This is not a public Heap constructor or FFI contract; a compiler
that starts accessing or retaining it must stop this experiment for adaptation.
Independent negative checks removed each required property and the adapter
rejected both. Normal native fixture execution in all three modes uses the
ordinary command Heap and no adapter.

The 84 retained samples in `growth-measurements.csv` were collected on the host
and toolchain above, with unchanged compiler code through revision
`297adb4e2e2ab736f660a69a7a8f64e71538a50b`. Both languages use Clang `-O2`, without
LTO. Seven samples rotate four variants at each size. Each warms itself with
32 complete calls (two at the largest size), then measures 65,536, 1,024 or eight
calls respectively. Seeds vary per call; observable sums are checked against
the independent oracle outside timing. Allocation instrumentation, ledger reset,
generation, copying, digest and cleanup are inside timing. These are complete
operations, not isolated resize latency. There is no exclusive-host or CPU
affinity control; the 4 KiB WF samples include visible drift.

Median nanoseconds per complete trace, with sample minimum/maximum in parentheses:

| Initial → requested bytes | Whitefoot | C source loop | C bulk copy | C realloc |
| --- | ---: | ---: | ---: | ---: |
| 16 → 32 | 93.48 (93.02–94.57) | 79.01 (77.93–81.27) | 78.75 (77.91–81.05) | 96.88 (95.61–97.61) |
| 4,096 → 8,192 | 16,966.80 (16,788.09–21,961.91) | 15,907.23 (15,802.73–16,193.36) | 15,791.02 (15,691.41–16,051.76) | 15,958.98 (15,873.05–16,008.79) |
| 524,288 → 1,048,576 | 2,155,875 (2,150,500–2,234,250) | 2,069,000 (2,015,875–2,118,750) | 2,093,500 (2,025,000–2,107,375) | 2,047,875 (1,995,625–2,150,000) |

The current WF loop needs no initialization of unused capacity. Its optimized
copy retains scalar byte accesses and circular-index select/subtract operations;
the C source loop is vectorized and the bulk control retains `memcpy`. This is
an actionable lowering difference, not an isolated causal explanation for every
timing difference. In this operation, byte generation and digest also execute
for the entire final sequence. The result does not show a general resize
performance ceiling or that bulk copying is irrelevant to a copy-heavy workload.

Allocate/copy has two explicit backing requests and simultaneous old-plus-new
logical extent. `realloc` has one malloc and one resize request. Its returned
address remained unchanged in all 56 timed largest-size calls and in none of
the smaller timed calls on this allocator. The ledger cannot observe storage
temporarily used inside realloc, allocator headers or resident memory; its
smaller recorded extent is **not** a measured physical peak-memory advantage.
Request counts include injected refusals. Realloc was slower at 16 bytes here;
its mere availability does not establish a performance win, while a provider
contract permitting it remains independently useful to test.
