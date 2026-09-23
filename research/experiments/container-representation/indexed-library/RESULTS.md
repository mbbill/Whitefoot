# Indexed composite cost experiment

This explicit experiment implements the discriminator registered in
[X1](../../../investigations/containers-and-resources/X1-LIBRARY.md#indexed-composite-trial).
It bundles the maintained `tests/programs/containers/indexed-store.wf` operation
implementation with a research-only trace caller. The formal caller does not
depend on this directory. The Makefile, controls and samples exist to test the
complete indexed consumer and shared-sift cost; retire them when superseded
and no maintained claim needs their replay.

## Prospective protocol, recorded before timing

The initial compiler is the frozen main `345e2966a` CLI, SHA-256
`cbffd4dd1ae8641ef03790457181188988bf70cc4af1a53c50c1f406307bb7f9`.
The plain queue source at `c3c2a50fd` is the original comparison image. The
candidate uses the shared indexed core and its ordinary no-op specialization.
The indexed comparison uses the same maintained composite and trace with a
bounded standalone indexed sift control. Neither the standalone source nor
the C controls replace the maintained application implementation. Admission
subsequently exposed the compiler's effect-row refinement defect: it rejects a
callback's `reads(env)` actual against a `writes(env)` formal even though EFF-1
makes that formal permission include reads. Two later blockers were an OP-9
layout query treating nested symbolic nominal arguments as concrete, and an
unused formal's symbolic nominal escaping into the executable inventory.
The three current-specification repairs are published at `3f85b1030`; no
language rule changed. Every new comparison image uses its frozen CLI,
SHA-256 `c71aaf16fdf175b9c269e07b18fa45b82b2a54d0a3cd4d4443b1eb185454cb8f`.
This is a source-architecture comparison using one compiler, not a comparison
between baseline and repaired compilers. The reconstructed original queue's
raw LLVM is byte-identical to the preserved `345e2966a` artifact (SHA-256
`dfceb1385f84124146c0458fcebcf4e734c3a09d0f76001cf28042f81661a4f5`);
both optimized modes differ only in their ModuleID path line.

Before any costs are used to choose, require the complete outcome transcript
to match an independent array/ordered-sequence model in every matrix cell.
Each offered owner has a distinct identity; every returned/consumed payload
word, status, handle and observed identity is compared, with per-owner offered
and consumed counts. The maintained corpus separately owns the droppable and
nodrop, wrong-store, refusal/retry, bounded-generation, malformed-position and
parallel-lowering witnesses. Inspect the original/plain no-op optimized code
before timing; an unexplained material change is reported before proceeding.

The indexed matrix is fixed at 24 cells:

| Axis | Values |
| --- | --- |
| Initial live records | 16, 256, 4096 |
| Slab payload size | 8 or 256 bytes; every heap entry has the same layout |
| Membership policy | weak, retained |
| Trace | reserved mixed operations, growth and ordered cleanup |
| Public boundaries | normally optimized, retained public operations and real callbacks |
| Implementations per image | Whitefoot, source-shaped swap C, direct indexed hole C |
| Repetitions | seeds 101 through 107; two reversed-order cohorts |

The Slab capacity is the initial record count, with ceiling 8192 and generation
limit `UINT64_MAX`. The reserved trace starts with map and heap capacity twice
that count; growth starts both indexes at zero. Slab has no growth operation.
The hash callback is ID modulo 8192. Record `i` has ID `2*i+1`, except indices
congruent to one modulo eight use the preceding ID plus 8192. This gives one
deliberate colliding pair per eight records. No collision distribution or
population is selected after observing timings.

Both traces insert, attach and schedule every initial owner, check a refused
extra Slab owner, and consume it explicitly. The growth trace then expires all
entries in deadline/ID/index/generation order and destroys the empty store.
The reserved mixed trace looks up and replaces every payload, reschedules even
records earlier and odd records later, cancels and reschedules every fourth
record, and deletes/reuses every eighth record. Weak deletion leaves exactly
`n/8` stale heap entries at that point; retained deletion first observes Busy
and retires both memberships in alternating orders. Both policies reuse the
same slot and ID with a new generation. Partial expiry consumes the remaining
early records (indices congruent to two modulo four), which are then reinserted
under the same IDs. A second expiry at `9*n-1` processes the weak stale
generations, protects replacement ID associations, and consumes exactly `n/2`
live even records under both policies. Final destruction retires both indexes
and consumes the `n/2` remaining odd Slab owners. Each step records its explicit
success/refusal/expiry
outcome; setup and final cleanup are included in elapsed time.

One timed sample repeats the complete trace `max(1, 2048/n)` times for small
payloads and `max(1, 512/n)` for wide payloads. There is one mixed-operation
round per trace. The trace seed is sample seed plus repetition index. Initial
deadlines are `4*n + ((LCG(seed) >> 32) % (4*n))`, with the existing unsigned
LCG multiplier 6364136223846793005 and increment 1442695040888963407.
Earlier deadlines are the record index; later deadlines are `16*n+i`;
cancelled/reused records use `8*n+i`; the two expiry boundaries are `4*n-1`
and `9*n-1`.
Payload word zero is its unique owner number; remaining words are a fixed
seed/owner/word-index pattern. There is no checksum-only correctness oracle.

The independent model and full log are outside the timed interval. Each timed
implementation retains the same digest work, but uses a zero-capacity outcome
log. Diagnostic operation counts use a separate image: comparison, swap or
assignment, position-report, handle-validation and hash-probe counts are not
silently added to only one timed implementation. Report actual layouts,
payload transfers, allocation/release identity and requested/peak bytes,
including stale entries and old/new growth overlap.
The diagnostic byte columns multiply explicit heap/payload assignment counts
by their payload extents. They describe algorithm-level movement, excluding
parameter/result temporaries, callback spills, inactive-field initialization
and cache traffic; optimized code inspection accounts for those lowering costs.
That separate diagnostic image runs seed 101 once in every cell for both C
controls, yielding 48 rows; its counts are not timing samples or averages over
the seven timing seeds.

For the indexed retained mode, the retained public boundary is the composite's
15 `record_store_*` operations, including its validation/position accessors,
plus actual ordering, hash/equality, borrowed observation, edit, position-report
and owner-consumption callbacks. Retention means a `noinline` boundary, not
dynamic export: both indexed WF images internalize non-bridge definitions to
match static C helper scope, preserving the two C-called trace bridges and
ordinary runtime declarations. This experiment-only instrument permits the
same unused-argument and result optimization in both languages; production
compiler linkage is unchanged. Underlying library helpers remain normally
optimizable in both languages. The separate plain-PQ comparison preserves its
existing public queue and payload-callback policy and exact reference images.
That preserved pair has static C helpers and external WF definitions, so its
WF/native ratio includes different ABI optimization opportunities and is not
an isolation of same-ABI lowering. In particular, an empty
position callback is not forced out of line: the original plain queue has no
corresponding callback boundary.

The independent backing formula uses a 16-byte header, 32-byte ID buckets and
32-byte heap entries. Slab cell strides are 56/304 bytes for 8/256-byte payloads.
The source-shaped C public results retain the backend's separate variant
payload fields rather than overlaying them in unions. Their extents are
40 bytes for handle and record-info lookups, 24 for a position result, 12 for
a unit visit result, 40/288 for insertion and 48/296 for deletion. Membership
booleans occupy one byte. Static C assertions check these extents; the admitted
WF module remains the concrete-layout cross-check before timing.
For the reserved trace, three allocations request and peak at
`48+n*(SlabStride+128)` bytes. For growth, each index allocates capacities
0, 1, 2, ..., n: requests are `3+2*(log2(n)+1)`, requested bytes are
`48+n*SlabStride+2*sum(16+32*c)` over positive growth capacities, and peak bytes
are `64+n*(SlabStride+80)`, including the old and new final heap backings.
These formulas are checked after each trace or timed batch, outside its timed
interval, in addition to the allocation/release identity ledger.

Each image contributes 2,016 rows (24 cells × 3 implementations × 7 seeds ×
2 cohorts × 2 boundary modes). Run one fresh A/A unchanged-image pair and one
A/B standalone/shared pair. Preserve the existing complete plain-PriorityQueue
matrix: 2,520 rows per image, seven seeds, both cohorts/modes, all five paths,
both payload sizes and both C controls. Run fresh original/original and
original/no-op pairs, preserving original sources and actual image identities.
The unchanged C controls remain in every image. Within an image rotate
implementation order by sample and reverse it in cohort one; reverse both
image order and normal/retained mode order between cohorts.

For each cell and cohort, pair equal-seed elapsed times and normalize the WF
A/B ratio by the matching unchanged swap-C ratio. Report raw and normalized
ratios, both C controls, medians, full ranges and individual tails. A material
change must exceed the maximum of 3%, that cell's observed unchanged-image/C
variation, and measured clock quantum relative to its sample time. Agreement
inside that band permits sharing on maintenance grounds; it does not establish
native parity. An indexed benefit must repeat beyond that band in both
cohorts and have algorithmic or emitted-code attribution. Any unexplained
material indexed or plain-no-op regression leaves the shared-core selection
open. Policies and workload cells are not averaged together.

Concretely, compute seven equal-seed ratios in each cohort and take their
median. The cell's variation band is the maximum absolute departure from one
among its two null normalized-WF medians and its unchanged swap-C and hole-C
medians in both null and A/B runs. The clock term is four times the smallest
observed positive monotonic-clock increment divided by the smallest median
sample interval in the cell. These terms and 3% define one band for the cell;
individual tails remain visible but do not replace a seven-sample median.
A benefit or regression repeats only when both cohort medians exceed that band
in the same direction. A one-cohort excursion remains unresolved and can
trigger only the complete fixed repeat below.

At most one complete fixed repeat of both paired matrices is allowed if the
initial result is ambiguous from noise. Keep every initial/repeat sample and
tail; no narrower rerun is selected to remove a regression. If the full repeat
cannot fit the remaining budget, report the ambiguity. The combined budgets
are 120 seconds for artifact construction, 40 seconds for native correctness
execution and 60 seconds for timing, including null runs and that repeat.
Compiler construction and the canonical gate are separate guarded stages.
These rules were frozen before the first timing; the retained
`protocol-before-timing.md` records that prospective state.

The repeat, if used, includes A/A and A/B for both the indexed and plain-queue
matrices, with unchanged seeds, trace sizes and ordering. The initial protocol
therefore retains 18,144 rows, and the sole permitted full repeat would add
another 18,144. No tail, mode or policy is sampled separately to choose a result.

## Admission and correctness progress

The first construction stage took 1.08 seconds. Both native C modes compiled;
the shared WF bundle stopped in the maintained helper at a reserved `entry`
local name, before admission. This is a source parser failure, not a measured
cost or a changed acceptance verdict. The helper owner corrected the local
names; its subsequent formal admission exposed the compiler blockers described
above. The full maintained bundle subsequently passed sequential and parallel
native execution, with both allocation observers reporting exactly 129.

The native execution stage took 2.60 seconds. Each mode passed 832 unique
complete matrix inputs and 1,664 swap/hole executions against every transcript
word and per-owner ledger, with matching backing counts, requested bytes and
peak overlap. Those inputs cover every seed used by every timed repetition;
overlapping seed/repetition combinations are checked once. This initial native
precheck preceded the static-review addition of the second expiry boundary
described above, which ensures stale generations are processed before
destruction. No timing preceded that change. Subsequent edits
to a control or helper require affected checks before timing.

Static fairness review also corrected the native controls' callback argument
shapes, result extents and one-byte booleans, and added the independent backing
formulas above. The initial native precheck does not certify those later edits.
The maintained constructor now states the ordinary OP-9 upper bound for its
32-byte heap element; all registered capacities satisfy that source
precondition, so the C controls need no extra runtime assertion for it.

The repaired-compiler construction attempts include the research caller's
canonical formatting correction and an explicit verified upper bound on its
reserved capacity. These preserve the registered trace. The shared image and
both plain images are built. Their native execution took 5.51 seconds: each
indexed boundary mode passed 832 complete inputs and 2,496 full-transcript and
owner-ledger executions; each of the four plain original/no-op boundary modes
passed the existing 2,160 complete scalar/owning traces and four refusal/retry
chains. The composite retained-operation call checks also passed.

The concrete standalone control required the ordinary Copy spelling:
`swap(first: &queue_slot, second: &other_slot)` is refused under OP-11 for
`RecordDue`; its temporary/read/assignment form preserves the same three
assignments and position reports. FN-2 permits the generic source's explicit
ownership operations to specialize over a Copy type. OWN-1 likewise removes
explicit `move` from concrete Copy operands. Direct comparison reads no empty
environment, so EFF-2 removes that unused read from the concrete rise/push rows.
Its next admission stops after `priority_queue_make_room<RecordDue, ceiling>`:
the following `place_back` does not recover `len < cap`. The 0.21-second
diagnostic isolated this before any timing: an immediate `room: len < cap`
invariant fails with a symbolic ceiling, also fails after adding
`requires ceiling <= 576460752303423487_u64`, and admits with literal 8192.
Thus adding the ordinary OP-9 bound to the caller does not recover this symbolic
call summary. Read-only diagnosis supports a missing callee proof: reserve's
own contract supplies only `total <= ceiling`, while its known 32-byte element
requires `total <= UINT64_MAX/32`. ENT-2 does not import the caller's extra
premise into that callee body. Literal substitution supplies the bound locally.
This is not evidence of a compiler publication defect; no language acceptance
verdict is revised by the experiment. It does not establish that concrete
controls are impossible: a literal ceiling or a suitably bounded callee
contract supplies a different proof. The existing
`transitive_known_layouts_do_not_take_the_direct_opaque_deferral` compiler
regression covers the distinction between a direct opaque layout and a
transitively known layout. Improving this diagnostic is separate from the
sharing comparison.

The unadmitted concrete control is replaced before measurement by seven bounded
generic functions: rise, sink, repair, push, peek-at, remove-at and replace-at.
They preserve the shared candidate's element opacity, ordinary comparison and
placement formals, bodies and contracts, but are bundled beside the frozen
original plain queue. The maintained consumer is byte-identical in both arms;
the four experimental generic-binding substitutions are removed. This is still
one standalone/shared comparison, with no additional measured variant. It
tests sharing against a viable duplicated indexed core without charging only
one arm for the concrete Copy spelling or unavailable symbolic call summary.
The rejected concrete spelling is source-admission evidence, not a timing
result or a new language restriction.

Final construction also internalized indexed helpers, preserved the weak
runtime floor and external runtime entry body, and matched runtime object
identity and link order in the two indexed images. These instrument corrections
preceded the final checks and every timing. The final 6.38-second correctness
stage checked both indexed images in both boundary modes: each passed 832
unique complete inputs and 2,496 Whitefoot/swap/hole transcript and owner-ledger
executions. All 15 retained composite-operation call checks passed in both
images. The separate event image produced all 48 registered diagnostic rows.

The layout review compared actual fields and offsets as well as extents.
Tag-only `RecordStatus` uses an i32 tag; membership booleans occupy one byte.
Small/wide records place ID, payload, membership and position at offsets
0/8/16/24 and 0/8/264/272, with extents 32/280. Handle-lookup and info results
have separate tag, success and error fields at 0/8/32, extent 40; position
results use 0/8/16, extent 24; unit visits use 0/4/8, extent 12. Insert results
use tag/handle/payload at 0/8/32, extent 40/288; delete results use
tag/record/status at 0/8/40 or 0/8/288, extent 48/296. The map lookup and put
products are each 32 bytes, with fields at 0/8/24 and 0/4/8 respectively.
The C controls preserve these products rather than using unions. This removes
a representation confound; it does not assert identical native argument/result
ABIs. For example, WF unit has an i8 representation where a C callback may
return void. Timed C helpers contain the externally required handle, identity
and refusal checks, with proof-only structural assertions left in the oracle.

Final spent budgets, including failed source/instrument attempts, are
**27.46/120 seconds construction, 14.49/40 seconds correctness, and
51.57/60 seconds timing**. The two timing guards took 25.76 and 25.81 seconds.
Earlier attempts remain in these totals; none reset after source corrections.
The outer qualification guard includes the separately logged link/check stages
and is not counted again. Compiler repair builds and maintained corpus tests
are separate from these experiment budgets.

Every construction attempt is accounted for below. A failed stage can include
successful earlier image construction; the full elapsed stage remains charged.
The original attempt logs are retained locally. The archive keeps only the
construction logs needed to identify the final images, final/plain correctness
logs, paired timing logs and final identity check.

| Construction stage | Seconds | Outcome |
| --- | ---: | --- |
| Initial native/shared admission | 1.08 | Native controls built; maintained reserved local name stopped WF parsing |
| Repaired-compiler admission | 1.07 | Research caller canonical formatting refusal |
| Shared caller proof | 0.86 | Reserved capacity upper bound unproved |
| Shared explicit proof attempt | 0.86 | Redundant `use` block refused |
| Shared admission | 1.06 | Passed |
| Standalone first admission | 0.11 | Comment prefix refused |
| Standalone concrete source | 0.44 | OP-11 Copy swap refused |
| Standalone Copy spelling | 0.55 | Extra effect on empty comparison environment refused |
| Initial image construction | 6.44 | Shared and both plain images built; standalone canonical spacing refused |
| Standalone spacing correction | 0.11 | Second canonical spacing refusal |
| Standalone ownership correction | 0.54 | Explicit move of Copy value refused |
| Standalone concrete room proof | 0.87 | Missing `place_back` capacity premise |
| Three ceiling diagnostics | 0.21 | Symbolic/caller-bounded refused; literal admitted |
| Closed-scope instrument | 1.73 | Weak linkage combined incorrectly with internal linkage |
| Closed-scope link | 2.05 | Runtime main body incorrectly internalized |
| Shared closed-scope construction | 3.78 | Shared built; generic standalone final blank line refused |
| Generic standalone construction | 4.11 | Passed |
| Matching runtime-object link order | 1.59 | Passed |
| **Total construction** | **27.46** | **Within 120-second budget** |

Correctness stages are the initial native precheck (2.60 seconds), the initial
shared/full plain checks (5.51 seconds), and both final indexed images plus
event counts (6.38 seconds), totaling 14.49 seconds. Timing consists only of
the complete initial series (25.76 seconds) and complete fixed repeat (25.81
seconds). Queue refusals ran no command and add no execution time.

## Measured result

The shared core is recommended on maintenance grounds under the registered
no-material-regression criterion. Both series classify all 48 indexed
mode/workload cells as equivalent. The plain comparison has 59 equivalent
cells and one ambiguous cell in each series, with different ambiguous cells.
The initial apparent loss resolves in the sole complete repeat; the remaining
repeat ambiguity points toward a possible benefit in one cohort and is not a
proven gain. No cell meets the predeclared material-change rule of both cohort
medians exceeding its band in the same direction. This supports reuse of the
shared implementation without claiming that every plain cell established
equivalence, that sharing improves speed, or that WF reaches native parity.
The result remains bounded to this host, toolchain, source and matrix.

The two independently constructed indexed executables are byte-identical in
each boundary mode after matching runtime object order. Their raw LLVM differs
because the standalone arm bundles the original plain queue plus the seven
duplicated generic indexed operations, while the shared arm bundles the shared
queue. The final identity check verifies the distinct construction inputs and
the identical native images. This is direct code evidence that the measured
indexed sharing choice adds no native instructions on this toolchain.

| Image | SHA-256 |
| --- | --- |
| Indexed standalone/shared, normal | `36e84350eccae405df3ab8fd7e3b7d73ada1ffef5e9ea873b7acd43e8b4e58a2` |
| Indexed standalone/shared, retained | `9e66c9d7471ce95f20314f9ef61f2255166b19b40cb405892b6133807d13eabf` |
| Plain original, normal | `6cf3d4be23cc3fd43c3d50f284d396a91c3c78cd9e81c315b672aa4f426a2481` |
| Plain no-op, normal | `cb9a41f64c223172c4728857b4f42c8809eb211598f6c67495c8eafcb7d31743` |
| Plain original, retained | `5cb4d2fcb5ba631b92a433f0c3416b003d075c426c3b7c025fe8c5b749df4a23` |
| Plain no-op, retained | `488dbd42b9a835ed12fb00bb269ba659ec9d04de399232cfa1d1b1976a150014` |

The environment is Darwin 25.6.0 arm64, Apple Clang 21.0.0
(`clang-2100.3.34.2`), target `arm64-apple-darwin25.6.0`, with `-O2`.
The archive manifest records every source, compiler, executable and shared
runtime-object hash. All 33 manifest hashes were rechecked after measurement.
The unchanged maintained composite is SHA-256
`d9dda2738eb4797653560609756183b183a4be2d2ea2b9010f5070afbe0101b3`.
The original plain PQ is
`8a9f7a529ec61db4b888ce866f4b687b35f8226ccce94de668ee68f35cee08fe`;
the shared PQ is
`977d83e3d3fe77ec19b88b0047a858941cf079e44d6db9d638fe3403fbe3adaf`.

The exact ambiguous cells are below. Each pair gives cohort 0 / cohort 1;
ratios are no-op/original. Normalization divides each equal-seed WF ratio by
its unchanged swap-C ratio before taking the seven-sample median.

| Plain cell | Series | Band | Raw WF median | Normalized WF median | Classification |
| --- | --- | ---: | --- | --- | --- |
| Normal, 8 bytes, grow-pop, 4096 | Initial | 3% | 1.001912 / 1.000000 | 1.035093 / 0.997795 | Ambiguous |
| Same cell | Repeat | 3% | 1.000000 / 0.997980 | 1.003899 / 0.991870 | Equivalent |
| Retained, 8 bytes, pop-push, 16 | Initial | 5.5762% | 1.007692 / 0.992268 | 1.007692 / 1.015584 | Equivalent |
| Same cell | Repeat | 3% | 0.979221 / 1.002564 | 0.969773 / 0.990951 | Ambiguous possible benefit |

For the first ambiguity, initial swap-C cohort medians are
0.998120 / 0.996124 and hole-C medians are 0.998092 / 1.000000.
Its initial normalized individual ranges are 0.988395–1.085699 and
0.985582–1.028283. For the repeat-only ambiguity, both control medians are
1.000000 in both cohorts; normalized individual ranges are
0.964557–0.998688 and 0.962346–1.018568. The preserved plain native comparison
finds equal instruction counts in all 43 common queue/trace functions in each
mode. The normal rise's changed condition, `cmp #2; b.hs` versus
`cmp #1; b.hi`, is equivalent for unsigned integers. There is no surviving
extra empty placement callback. These observations support the absence of a
new plain overhead, without equating entire plain executables.

The minimum positive monotonic-clock increment was 1,000 ns in both series
(2,267 and 2,260 positive transitions in 100,000 reads; maximum steps 20,000
and 5,000 ns). Short samples therefore have bands wider than 3%. The following
ranges retain that limitation and every observed individual normalized tail;
they summarize cells for display and are not an aggregate selection test.

| Study / series / mode | Cohort median range | Cell band range | Null individual range | A/B individual range |
| --- | --- | --- | --- | --- |
| Indexed / initial / normal | 0.9876–1.0222 | 3–7.6555% | 0.8001–1.2522 | 0.8495–1.9267 |
| Indexed / initial / retained | 0.9824–1.0170 | 3–5.0317% | 0.8312–1.2032 | 0.8788–1.8725 |
| Indexed / repeat / normal | 0.9727–1.0165 | 3–9.5618% | 0.4984–1.1684 | 0.8243–1.2006 |
| Indexed / repeat / retained | 0.9740–1.0226 | 3–7.9299% | 0.7896–1.3474 | 0.7047–1.1297 |
| Plain / initial / normal | 0.9688–1.0351 | 3–19.0476% | 0.7326–1.1854 | 0.8148–2.6440 |
| Plain / initial / retained | 0.9741–1.0233 | 3–28.3019% | 0.7641–1.1137 | 0.8734–1.1368 |
| Plain / repeat / normal | 0.9750–1.0323 | 3–19.0476% | 0.7100–1.2748 | 0.7778–1.6326 |
| Plain / repeat / retained | 0.9698–1.0200 | 3–8.8700% | 0.7919–1.7720 | 0.8269–1.2890 |

## Remaining native costs

These are shared-image WF/C ratios, computed within each image by equal-seed
pairing, then taking each cohort's seven-sample median. Ranges span the three
counts, both cohorts and both complete series, without averaging policies or
traces. Values above one mean slower WF. They describe remaining costs;
they are not the standalone/shared selection statistic. `Retained` in the
mode column means retained public/callback boundaries; the policy column
independently selects retained or weak membership.

| Mode | Payload | Policy | Trace | WF / source-swap C | WF / hole C |
| --- | ---: | --- | --- | --- | --- |
| Normal | 8 | Weak | Mixed | 0.973–1.024 | 1.012–1.055 |
| Normal | 8 | Retained | Mixed | 0.954–1.040 | 1.008–1.075 |
| Normal | 8 | Weak | Growth | 1.124–1.197 | 1.137–1.258 |
| Normal | 8 | Retained | Growth | 1.128–1.219 | 1.144–1.264 |
| Normal | 256 | Weak | Mixed | 0.646–0.710 | 0.659–0.738 |
| Normal | 256 | Retained | Mixed | 0.640–0.681 | 0.654–0.707 |
| Normal | 256 | Weak | Growth | 0.917–1.032 | 0.939–1.148 |
| Normal | 256 | Retained | Growth | 0.917–1.025 | 0.927–1.150 |
| Retained | 8 | Weak | Mixed | 1.288–1.319 | 1.302–1.371 |
| Retained | 8 | Retained | Mixed | 1.308–1.334 | 1.317–1.366 |
| Retained | 8 | Weak | Growth | 1.261–1.365 | 1.268–1.407 |
| Retained | 8 | Retained | Growth | 1.261–1.368 | 1.266–1.408 |
| Retained | 256 | Weak | Mixed | 0.847–0.888 | 0.871–0.932 |
| Retained | 256 | Retained | Mixed | 0.846–0.868 | 0.860–0.911 |
| Retained | 256 | Weak | Growth | 1.051–1.106 | 1.085–1.222 |
| Retained | 256 | Retained | Growth | 1.035–1.087 | 1.085–1.211 |

The largest consistent indexed residual is retained boundaries with 8-byte
payloads in growth at 4096 records: WF/swap medians are 1.354–1.368 and
WF/hole medians are 1.392–1.408 across policies, cohorts and both series.
WF sample medians are 0.852–0.862 ms. Retained small mixed operations also have
a consistent gap. Conversely, wide mixed operations favor WF against both C
controls in this matrix. Neither a universal native gap nor parity follows
from these different paths.

The unchanged plain comparison retains the following native costs. Its
static-C/external-WF linkage difference remains part of these ratios, so they
cannot isolate same-ABI lowering or be substituted for indexed costs.

| Mode | Payload | Trace | No-op WF / source-swap C | No-op WF / hole C |
| --- | ---: | --- | --- | --- |
| Normal | 8 | Grow-pop | 0.892–0.983 | 0.891–0.990 |
| Normal | 8 | Heapify-pop | 0.990–1.096 | 0.963–1.087 |
| Normal | 8 | Pop-push | 0.995–1.198 | 0.986–1.121 |
| Normal | 8 | Replace-top | 1.000–1.077 | 0.913–1.081 |
| Normal | 8 | Setup-cleanup | 0.969–1.000 | 0.969–1.032 |
| Normal | 256 | Grow-pop | 0.871–1.015 | 1.100–1.279 |
| Normal | 256 | Heapify-pop | 0.855–0.988 | 1.152–1.311 |
| Normal | 256 | Pop-push | 0.965–1.106 | 1.281–1.556 |
| Normal | 256 | Replace-top | 0.974–1.281 | 1.276–1.424 |
| Normal | 256 | Setup-cleanup | 0.981–1.014 | 0.986–1.007 |
| Retained | 8 | Grow-pop | 0.793–0.995 | 0.795–0.983 |
| Retained | 8 | Heapify-pop | 0.724–0.924 | 0.720–0.909 |
| Retained | 8 | Pop-push | 1.169–1.719 | 1.182–1.732 |
| Retained | 8 | Replace-top | 0.790–0.914 | 0.787–0.860 |
| Retained | 8 | Setup-cleanup | 0.962–1.000 | 0.962–1.000 |
| Retained | 256 | Grow-pop | 0.859–1.023 | 1.062–1.093 |
| Retained | 256 | Heapify-pop | 0.806–0.924 | 1.007–1.055 |
| Retained | 256 | Pop-push | 0.892–1.075 | 1.222–1.742 |
| Retained | 256 | Replace-top | 0.755–0.827 | 0.653–1.026 |
| Retained | 256 | Setup-cleanup | 0.988–1.012 | 0.994–1.012 |

The largest plain swap-control residual is retained, 8-byte pop-push at 256
records: WF/swap medians are 1.679–1.719 and WF/hole medians 1.696–1.732
across both series/cohorts, with WF sample medians 0.799–0.818 ms. The original
and no-op images have the same relevant instruction counts, so this is an
existing cost rather than an observed sharing regression.

Read-only generated-code inspection identifies concrete residual work. In
`build/shared/indexed-library-retained.opt.ll`,
`wf_record_report_position$instance$136` snapshots a 32-byte heap entry and a
separate 16-byte handle before checking index, generation and occupied state
and calling the genuine setter. The native code retains these stack stores
and an 80-byte frame. C `small_swap_report` loads index/generation directly,
uses a 32-byte frame, performs the same three checks and calls its retained
setter. Scheduling reports once after append and twice per executed swap;
thus this snapshot work is exposed by the operation counts. Its dynamic
execution time was not measured separately.

Other indexed retained sites are the successful-info 24-byte payload copy in
`wf_record_store_schedule$instance$66` (native frame 352 bytes versus C's 176),
and `wf_record_store_insert$instance$88` clearing a 40-byte output before
storing active handle fields. C insertion stores the tag and active handle
without that whole-result clear. The three LLVM copy intrinsics for a WF
heap swap do not imply an extra machine transfer: its native schedule uses
two 32-byte loads and two 32-byte stores, eliminating the temporary.

In `build/shared/plain-noop/priority-library-retained.opt.ll`, successful
scalar push clears a 16-byte indirect result; C returns two registers.
The WF caller loads and checks that result tag. WF scalar pop reloads both
words on each swap, while C retains the displaced word in a register and
loads the child. Both preserve real comparator calls; the plain pair's
different helper linkage/ABI opportunities qualify this observation. No
dynamic empty-placement callback call survives. These are surviving code
differences, not measured causal shares of the ratios above. General reporter
snapshot, result lowering and ABI/register-retention improvements are bounded
follow-up opportunities; this trial selects no container-specific workaround
or language change. Any later improvement must preserve the operation and
ownership protocol and earn its own matched-image result.

The independent event image makes the algorithmic control difference explicit.
At 4096 records, seed 101, 8-byte payloads, growth has the same counts under
both membership policies. All 48 complete event rows, including wide payloads,
are retained in the raw archive.

| Trace / policy / C control | Comparisons | Swaps | Heap assignments | Position reports | Slab validations | Hash probes | Allocations |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Mixed / weak / swap | 107737 | 49555 | 186041 | 114470 | 172326 | 34304 | 3 |
| Mixed / weak / hole | 107737 | 0 | 82835 | 64915 | 122771 | 34304 | 3 |
| Mixed / retained / swap | 97243 | 44069 | 169583 | 103498 | 161354 | 36352 | 3 |
| Mixed / retained / hole | 97243 | 0 | 77349 | 59429 | 117285 | 36352 | 3 |
| Growth / either / swap | 87237 | 43574 | 155294 | 95339 | 124011 | 46550 | 29 |
| Growth / either / hole | 87237 | 0 | 68146 | 51765 | 80437 | 46550 | 29 |

Growth's logical heap-assignment bytes fall from 4,969,408 to 2,180,672 in the
hole control. Payload assignments remain 12,290 (98,320 bytes), identity
checks 16,384, handle comparisons 4,096, requested backing bytes 754,064 and
peak backing bytes 557,120. Mixed paths have 27,138 payload assignments,
39,936 store-identity checks and 9,216 handle comparisons under either control;
requested and peak backing bytes are both 753,712. The hole algorithm thus
reduces movement and position-report validation work without reducing the
comparison, hash or ownership protocol. These source counts do not partition
elapsed cost or include lowering temporaries; the favorable wide WF results
also rule out treating a logical assignment count as measured machine traffic.

## Retained data and replay

[Raw samples and evidence](measurements-raw.tar.gz) retain all 36,288 timing
rows in 64 original CSV files: 8,064 indexed and 10,080 plain rows per series.
No sample or tail is dropped. The archive also contains both clock records,
48 separate event rows, the prospective protocol snapshot, all frozen input
identities, nine final-image construction/check/timing/identity logs, and the
three bounded source-admission probes with their diagnostics. Abandoned
construction/debug logs remain local; their elapsed costs are accounted for
above rather than included as a bulk archive.
[The paired summary](measurements-summary.csv) has 432 data rows, one per
study/series/mode/workload/cohort, including raw and normalized medians, both
control ratios, full individual ranges, cell bands and classifications.
Its nine-digit ratios retain more precision than the tables above.

Archive SHA-256:
`899f252f8323b6cdd3eff50928a56b49d4f7f8ecc4b7baebf480c38e18ffaedf`.
Summary SHA-256:
`7f5f8d8ea421bc151fd98849c56e2810d96ab1987bb252c61fada2bcbaa4b626`.
Published identity/log labels replace local paths with `compiler/whitefootc`,
`build/standalone`, `build/shared`, `build/samples`, `toolchain/bin` and the
repository root `.`. Probe diagnostic labels use `probes/room-*.wf`.
This is a documented path substitution: hashes, command options, object order,
raw timing/clock/event CSV bytes and the three probe sources are unchanged.
Local scratch paths are not part of the durable record.

The existing Makefile's explicit `build`, `check-built`, `events`, `plain-build`
and `measure-pair` targets own reconstruction and replay. The archive logs
preserve invocation options and shared runtime-object order with the portable
labels above. Before replay, substitute chosen absolute local roots for those
labels and set `WHITEFOOTC` to the frozen compiler with the recorded SHA-256;
the normalized logs are not literal shell commands for an arbitrary working
directory. `HEAP_STYLE=standalone` extracts the hashed original queue at
`c3c2a50fd`; `HEAP_STYLE=shared` uses the current shared source. Measurements
consume already built images, refuse to overwrite sample files and use the
registered AA/AB/repeat naming. No research target is added to the formal gate,
and no additional timing variant or repeat was used.

**Design suitability.** Reusing the maintained composite keeps the measured
protocol aligned with formal ownership tests. The standalone indexed control
tests sharing against a viable bounded alternative and produces identical
native images on this toolchain. The source-shaped and hole C controls expose
remaining lowering and algorithmic work separately. Retained small-payload
snapshot/result and plain ABI/register costs merit targeted general-backend
follow-up, deferred because this experiment selects source sharing rather than
a new lowering mechanism. Wide paths run differently, so those opportunities
need matched validation before claiming a general benefit. Full transcript
checks and separate counting remain necessary to preserve ownership and
membership equivalence.
