# Port working notes (kernel spec v0.60 port, PR 70)

Working notes kept during the port: specification problems reported by port agents, owner rulings pending at the time, verified audit findings, and compiler gaps. Not normative; superseded by docs/todo.md and the design tree where those record the same item. Delete when PR 70 merges.

## Automatic approval review holds during integration

The owner authorized principled migrations to the current specification and
asked for remaining automatic-review holds to be reported together. These
entries distinguish still-unapplied holds from later evidence-backed
resolutions. An automatic-review refusal is not an accepted test result or a
compiler gap by itself.

The integration snapshot after `bb5b603e` passed `cargo check --tests` and
optimized test construction. Its complete unit execution passed 1501 cases
and failed four: the retained recursive sort/merge parallel-offer assertion,
the new known-layout symbolic OP-9 negative, the measured generic forwarding
regression, and a new formal-boundary SCC control preempted by FN-6. That
source cycle violates FN-6 before summary publication; the regression now
asserts that earlier rule and passes. It does not claim to exercise a later
FN-9 refusal that this source cannot reach.
Formal-only publication, routed direct publication, and concrete allocation
metadata controls passed in this snapshot. Later source edits require fresh
validation.

The range-element measure and Box-capture implementation checkpoint passed
all-target Rust type checking, all-target clippy with warnings denied, and
optimized test construction. Its focused run passed 187 cases and failed
eight newly written controls: six permission fixtures had an extra canonical
newline, the joined measure fixture used a reserved binding name, and the
whole-Box replacement control expected condition two although PAR-2 condition
one correctly rejects its non-reduction first. The direct/nested measure,
OP-4, stale-alias, native descriptor, Box ownership, capture representation,
formal contract and LLVM provenance-parser observations passed. Fixture
corrections preserve the subjects and require fresh validation; this is not
a complete gate result. The separate generic OP-9 regressions still expose
the held production defect.

After correcting the joined fixture's reserved name, its conflicting-target
variant reaches the intended FN-8 refusal. The guarded variant initially
passed source checking but exposed a separate native gap: a range-valued
`ValueMatchLet` lowered its join as `Address(T)` instead of `Range(element)`.
The repair retains the range element handle through the checked join and
physical type discovery. Its native control now executes both target choices
and observes their distinct nested lengths; the complete unit execution also
passes this control. Semantic acceptance alone was not used as evidence of
native support.

The corresponding complete corpus executable then passed the full native
conformance adapter and 63 other cases. Seven network cases were blocked only
by sandbox denial of loopback socket creation; a native follow-up with socket
access passed all eleven network cases. The remaining program-fixture failure
was `shared-option-view.wf`: its scalar Entry had become structurally copy,
while the test intentionally moves an affine payload into a shared enum view.
Declaring that payload `nocopy` preserves the borrowed-payload subject and both
37/0 result checks. The same follow-up passed the complete container-regression
test, for 12 passes in total. This is combined evidence from the same compiler
snapshot plus that fixture migration, not a fresh canonical gate success.

Hosted CI for `bb5b603e` exposed a host-sensitive observer assumption.
The nested-owner observer numbered allocations by arrival, although PAR-1
permits the independent calls to arrive in either order; its successor
identifies released elements by their distinct u64 contents, preserving
STOR-3's ascending logical release order and the unknown/duplicate-release
checks; that migration passes on `9924b64a`. The optimized allocation-reuse
test's LLVM aggregate-provenance parser remained faulty: hosted run
35517822044 for `9924b64a`, Linux unit job 106096649499, showed
Clang 18 retaining a stack-frame memset rooted at
`%wf.frame`, while the textual GEP walk selected `, ptr ` inside the literal
frame type before its top-level base operand. The successor uses the existing
nesting-aware comma parser, still admits only stack-rooted aggregate
initialization, and refuses heap or unknown roots. Its allocation, refill, and
exact-size assertions are unchanged. Neither issue justifies changing compiler
behavior to preserve an old textual assertion.

The `bb5b603e` hosted compute comparison failed for `records` at both
two and four workers (baseline/candidate wall ratios 0.928450 and 0.869568,
all five pairs adverse at each width); the other four kernels passed. The
comparison's null and known-slowdown controls completed. This remains an
unresolved performance result, not a correctness failure or measurement noise
claim; the checked-in threshold and independent output oracles are unchanged.
Run 35517825284 repeated the result on `9924b64a`: the exact paired identity
was `records=131072 max_length=255 shape=unicode seed=812381`; its null and
known-slowdown controls passed, while records measured 0.922809 at two workers
and 0.867225 at four, with all five pairs adverse at both widths. One worker
improved to 1.077917 and the other four kernels again passed, retaining the
width-specific diagnosis.

Run 35519103811 compared the Box-capture repair in `b5e7a99f` through its
synthetic merge `60a5e589` against `c12d6dd1`. Compiler and native-image
construction, identity checks, identical-image null control, known-slowdown
control, and instrument self-tests all passed. The complete five-pair
comparison still failed only for records: one worker passed at 1.077677,
while two and four workers measured 0.935847 and 0.873395, each with five
adverse pairs. The repair removes the repeated parent Box-slot load in the
inspected optimized IR and assembly, but does not establish performance
recovery; the remaining generated-code differences require investigation.
No threshold, workload, output oracle, or comparison control was relaxed.

The same published revision's correctness run 35519101963 passed static,
corpus/conformance, and runtime groups on Linux and macOS. Both unit groups
reported 1515 passed and seven failed: four missing range-permission
observations and three OP-9 or downstream generic-capacity regressions.
The library group still stops at that generic capacity defect. The earlier
fixture syntax, PAR-2 condition selection, and LLVM provenance-parser
corrections therefore have fresh hosted evidence; these results still do
not constitute a green canonical gate.

The next integration tree repairs the joined-range native representation and
the pair-scoped PAR-1 proof handoff. Canonical `make check` passed its complete
static group (including all-target clippy with warnings denied), compiler
construction, and test construction. Unit execution passed 1520 cases and
failed only three: the generic-grow result publication, exact symbolic numeric
OP-9 bounds, and known stored layouts inside symbolic schemas. All three are
the held OP-9 defect or its downstream consequence. The gate stopped there;
later groups require their own fresh executions and are not implicitly green.
After strengthening the positive permission fixtures to require the exact
`LeftBeforeRight` conclusion as well as the captured pair and retained root,
optimized test construction and all 202 focused proof, permission and native
range cases passed. Rust formatting and diff whitespace checks also passed.
The separate fresh canonical corpus group passed all 72 tests, including the
complete native conformance adapter and network fixtures, in 122.15 seconds.
The final static group passed in 22.82 seconds, and the complete runtime group
passed in 0.65 seconds. The library group still stops with
`Semantics/Compiler: InvalidResolution` while constructing `vector-default.ll`,
the same visible stop as both `b5e7a99f` hosted library jobs. These separate
group results do not replace the failed canonical gate.

The next pending-proposal reconciliation exposed a real FN-9 gap: returning
`owner.inner.len` directly from `Box<Slots<u8>>` or `Box<Array<u8>>` was rejected
as `InvalidPostconditionReturn`, although binding that same measure locally
and returning the binding passed. ENT-2(b), FN-9 and TYPE-9 admit the direct
measure. The selected-return classifier now walks the complete typed checked
path, retaining each field, Box-content and measured-subscript projection,
and verifies the endpoint type. Result-selector admission is unchanged.
Controls cover nested Box and struct paths, inline and locally bound measures,
and different roots, fields, offsets and false arithmetic postconditions; the
negative cases reach the intended FN-9 refutation. Fresh canonical `make check`
passes static checks and construction, then reports 1521 unit passes and the
same three held OP-9 failures (218.72 seconds total). It is still a failed gate.
The separate fresh canonical corpus group passes all 72 tests, including the
complete native conformance adapter and network fixtures (121.65 seconds).
After the pending-proposal reconciliation, design lint and its 17 self-tests
pass with 69 amendments. These checks do not exercise the held OP-9 repair.
A separate read-only technical audit found no incorrect selected-return
admission in the complete typed path or its flow consumer. Its suggested
reference-root/Box-content control, a Box-plus-index control, and a distinct
reference-root negative have been added to the same regression and pass after
optimized reconstruction (0.10 seconds execution). The negative declares only
its exhibited left-hand read; a right-hand measure appearing solely in erased
clauses contributes no effect. This audit is not the completion review or DCR.

Four superseded pending proposals have been merged into their current owners
or removed, leaving 69 amendments for owner review. The current proposals no
longer retain the retired mandatory generic bound, prelude reserved-name
exception, room-derived measure kills, spelling-only member rejection, missing
named-owner displaced release, or temporary unsupported storage variants.
The allocation-fit proposal now distinguishes known and unresolved layouts,
but its general unresolved/non-finite production distinction remains held as
documented below. Two later, strictly narrower production changes were allowed
and are recorded with their focused evidence below; updating the proposal is
neither implementation evidence nor an owner ruling. No live tree or approval
log changed.

The remaining records performance investigation compared the exact `c12d6dd1`
baseline and `b5e7a99f` candidate without starting another timing campaign.
With local LLVM 22 retargeting to x86-64, the hot chunk instruction streams
match apart from the new Box descriptor's eight-byte content offset; grain
selection, chunk counts and loop alignment also match. The hosted comparison
uses Clang 18, so this does not establish the cause of its regression. The
existing CI artifact upload now retains both arms' already-generated LLVM
modules and native objects, allowing the next ordinary run to settle whether
the captured pointer reload survives that exact toolchain. Timing, thresholds,
workloads, and output oracles are unchanged.

Hosted compute run 35521786714 on `5aaef9ad` retained the Clang 18 modules and
objects. Records still fails at two/four workers (0.919672/0.871794, five adverse
pairs each); one worker and the other four kernels pass. Identity, null and
sensitivity controls pass. Direct object inspection rules out the repeated
Box-slot load: the two parallel chunks have the same instruction sequence
apart from the eight-byte payload store displacement, and both are 0x1b8 bytes
with 25 branches. Split grain and leaf counts agree (32/64 for two/four workers),
and the candidate splitter is smaller. The chunk strides are whole cache-line
multiples, and the two output requests share a glibc allocation size class, but
the artifact records no runtime allocation addresses; a claim about actual
boundary sharing therefore still depends on an unverified alignment premise.
Raw-sample reduction and the control environment also agree with the verdict.
One worker selects a different sequential clone, so its improvement does not
establish that the parallel chunk improved. The remaining code-placement
hypothesis cannot be judged from a relocatable object's relative addresses;
the existing artifact upload now also retains the final executable images,
and its input identity record includes `lscpu` output to identify the actual
CPU behind any architecture-dependent code-placement hypothesis.
No alignment, layout, scheduler or timing change has been selected from this
unresolved hypothesis.

The same `5aaef9ad` revision's correctness run 35521783830 passes static,
corpus/conformance and runtime groups on Linux and macOS. Both unit groups
report 1520 passes and the same three OP-9 failures; both library groups stop
at `InvalidResolution` in `vector-default.ll`. IO host run 35521783826 passes.
These hosted results precede the direct-measure FN-9 repair above and are not
presented as validation of that later change.

Hosted revision `d5377f39` validates that FN-9 repair, including the added
reference-root and Box-plus-index controls: run 35523614522 passes static,
corpus/conformance and runtime on both hosts, while both unit jobs report
1521 passes and the same three OP-9 failures. Both library jobs retain the
same `InvalidResolution`; IO run 35523614803 passes. Compute run 35523616425
passes identity, null and sensitivity controls but still fails records at
two/four workers (0.958088 with five adverse pairs, 0.928470 with four).
One worker passes at 1.063158 and the other four kernels pass. The recorded
host is Xeon 6973P-C, family 6/model 173/stepping 1, with four logical CPUs.

The final executable images make the code-placement observation concrete.
Parallel chunk entries are baseline `0x2ef0` and candidate `0x2f10`; sequential
clone entries are `0x32d0` and `0x32e0`. All four chunks are 0x1b8 bytes.
The baseline parallel chunk has one six-byte conditional jump crossing a
64-byte boundary (`0x303e`); the candidate has two (`0x2ffc`, `0x303d`).
The sequential direction reverses: baseline has two (`0x33bc`, `0x33fd`),
candidate none. Exact instruction inspection still rules out a repeated
Box-slot load or additional split work. This is a correlation with the
observed worker-width direction, not evidence that an older Intel JCC
erratum applies to this CPU or that boundary crossing caused the cost.
Runtime allocation alignment remains unobserved as a separate hypothesis.

A future discriminating layout experiment must hold the input algorithm,
payload layout, split counts and sequential clone fixed while changing only
the parallel worker's placement. Parallel cost following the changed layout
while one-worker cost stays stable would support a code-placement cause;
otherwise that hypothesis would lose support. No such timing experiment or
alignment policy change has been performed or selected in this work.

A strictly strengthening OP-9 subset was separated from the held unresolved-
layout repair. Automatic review allowed adding ordinary allocation-fit records
to source-canonical symbolic callers only when the existing layout authority
already returns a finite stride. No layout classifier, recursive type walk,
unknown-layout deferral or non-symbolic path changed. This adds the expressible
schema obligations required by ENT-1 for scalars, pointer-sized `Box<T>`, known
aggregates and bounded numeric parameters; it publishes no schema summary or
lowering authority. Its independent technical audit established that the
change only adds required rejections. The known-layout and bounded-numeric
schema regressions pass, together with the unresolved replay,
symbolic-const-array replay and concrete `AboveU64` controls; the complete
37-case window semantic module passes. The general distinction between an
unresolved symbolic layout and a real non-finite layout remains held, so this
result is not a complete OP-9 repair.

Automatic review subsequently allowed a second, narrower subset for the
transitive generic-grow failure: defer only when the stored element itself is
exactly `CheckedType::Generic(_)` and the caller substitution is nonconcrete.
Forwarding an opaque type parameter changes its declaration key, so the
existing canonical-only `is_symbolic()` test missed it and fabricated a zero
allocation limit from the unknown-layout `AboveU64` placeholder. The new guard
recognizes only that direct opaque parameter; numeric parameters, aggregates,
fixed-layout shells, actual overflowing layouts and every concrete instance
remain on their previous checking paths. Two independent technical audits
checked this boundary and the scratch inventory's disposal before concrete
replay. No recursive layout classifier or general unresolved-layout repair
was applied. The focused generic-grow wrapper now passes and publishes its
integer result relation. Maintained-library execution remains pending, so this
does not claim that every downstream library failure is resolved.

The nested range-reference suite now passes all 32 cases; its independent
native read/write/borrow checksum also passes. The first new joined-reference
fixture incorrectly expected incoming length facts to prove the joined
holder's distinct length term. Automatic review refused adding a guard alone
because that could conceal the unmet obligation. Re-reading ENT-2, ENT-3 and
ENT-6 established that no reference-valued fact transport is admitted, and the
test now preserves the original source as an OP-4 negative alongside an
explicit S1-guarded positive. No proof rule was added to satisfy the mistaken
test expectation. The final focused suite took 6.66 seconds to execute and
15.47 seconds including its build; complete canonical validation is pending.

The later nested PAR-2 positive exposed a separate footprint mismatch. A
checked `RangeIndex` write already retained its target-before-RHS captured
outer offset and complete typed suffix, while `set_target_place` replaced that
outer step with `CapturedValue::unknown()`. The matching read kept the captured
path, so removing the innermost affine index compared roots ending in literal
zero and unknown and conservatively reported a shared write. The repair now
uses the checked target's complete `place_path()` for every resolved origin;
the OP-4 affine map remains the only source of cross-iteration independence,
and unresolved or multi-origin targets still fail closed. The paired shifted-
read negative keeps a different affine map and remains denied. All 71 loop
permission tests pass, including the nested positive and shifted negative.

The subsequent canonical gate passes static checks, all 1533 library and 14
CLI unit tests, all 72 corpus tests including the complete native conformance
adapter, and runtime tests. It stops at the maintained vector library with
`InvalidResolution`; this is not a green complete gate. A temporary diagnostic
trace located a separate MSR-6 defect in call-goal formation: a const generic
read was already a checked `Constant`, but the call boundary admitted that
image only for a written literal and tried to resolve a const-parameter name
as an ordinary variable. The fix consumes the checked constant directly;
named constants keep their declaration identity and measures keep their
separate typed storage path. The trace was removed. All 33 requirement tests
pass, including new symbolic/transitive/concrete forwarding and independent
const-parameter negative controls. The rebuilt library now reaches an FN-8
failure at the append after a doubled-capacity reserve, rather than an internal
compiler failure. A reduced scalar program identifies that remaining failure
as the missing MSR-4 affine-left/L0-right bridge at the callable boundary:
`length <= capacity`, `1 <= capacity`, `doubled = 2 * capacity` and
`doubled <= widened` require Step 6 to prove `length < widened`. The integrated
repair retains the exact normalized right term and shares that finite bridge
across numeric consumers, including Boolean leaves and invariant targets;
measure and measure-datum candidates precede own integer bindings. It neither
publishes new premises nor recursively applies the bridge. PRF-1 relation-form
uses remain explicitly AUTO-only, with a paired control against importing
Step 6 into certificate premise admission. The source API, direct indexed
access, complete-result checks and release oracles remain intact. The rebuilt
maintained Vector library passes in sequential and parallel lowering,
including independent allocation observers: each execution makes 12
allocations and releases each exactly once. This focused native library run
takes 2.76 seconds; the complete gate still has loop integration findings.

A fresh scope audit also confirms that same-shape loop-carried reference
rebinding belongs to REF-1 rather than an intentionally retired source form.
The integrated implementation checks one abstract header, gives potentially
rebound captures distinct opaque header identities, and resolves owner-tagged
validity equations over entry and executable backedges. Aliases retain every
header alternative in the permission path. A noncontinuing edge contributes
no future invalidation premise, and every deferred use is resolved before a
checked function is published. The time-shift witness below is tested as
accepted sequential source with parallel permission denied and as an EFF-5
negative when both effects belong to one call. Independent native controls
observe scalar and range-reference values at zero-trip and exhausted exits.

The first complete unit execution of this integrated work passed 1541 cases
and failed four new loop fixtures: two overdeclared a helper's writes contrary
to EFF-2, one pinned the wrong REF-2 event text, and one omitted the loop-local
OP-4 premise after a header kill. The corrected fixtures perform real stores,
retain the hostile aliases and invalidation, and establish the fresh access's
own domain. A scalar bound cannot substitute for the strengthened nonzero-start
range-measure bridge test. The subsequent complete unit run passes 1544 cases,
including the strengthened measure and PRF-1 authority controls, and fails
three loop controls: the new native program reaches a lowering capability
failure, the permission control exposes an incorrectly indexed derivation
obligation according to the initial diagnosis, and the reform fixture needs
the grow operation's own OP-9 domain re-established after the header kill.
The derivation diagnosis was incorrect: its ordinal is right, but the test
validator assumed every non-separation obligation has an L0 relation
component. An affine-only range goal instead retains its exact canonical
positive goal. The validator now verifies that exact goal identity, keeping
the RangeSeparation wrapper and ordinary relation checks distinct. It does
not require a fabricated compiler component. The lowering repair retains
the binding mode on a checked writable place, so reference rebinding replaces
the address or range descriptor rather than writing its referent; owned
storage keeps its existing commit and release path. The positive reform
fixture establishes the grow domain inside the loop, while the prior-header
invalidation negative remains unchanged. Static checks pass. The next unit
run passed 1546 cases, including both native loop observations and the reform
control; its remaining failure was the test validator's metric count omitting
the existing `GoalAffineConsequence` root class. Adding that class to the
test's projected-goal count preserves every proof-content check, and the
hostile PAR-1 control then passes in its focused run.

The next canonical execution passes all 1547 library and 14 CLI unit tests.
Corpus execution reports 64 passes and eight failures: seven loopback tests
are blocked by the local sandbox's socket-bind permission, and the byte-string
program exposes a compiler regression in certificate redundancy. PRF-1 asks
exactly `AUTO(T)` for that judgment; the first shared-bridge integration instead
asked the complete MSR-4 disposition and incorrectly rejected a valid written
four-premise certificate. The repair separates those authorities: a blockless
INV-1 target gets full MSR-4, while certificate redundancy and relation-form
use admission both remain AUTO-only. The byte-string source and its complete
output oracle remain unchanged. The new regression pairs a blockless target
with the same target's four-premise certificate and requires both to discharge,
with the certificate retained as nonredundant. Canonical `make check` on
`c5919f4a` passes all groups in 351.31 seconds: static, 1548 library and 14 CLI
unit tests, all 72 corpus tests including the full native conformance adapter,
runtime, and maintained libraries. Network cases execute with ordinary
loopback permissions; none are skipped. Both sequential and parallel Vector
observers see 12 allocations released exactly once. This is a complete local
correctness result, not completion of the held schema repair or performance
investigation.

Pending compiler amendments were reconciled with the current specification
and implementation: allocation metadata propagates across checked calls but
is neither a source row category nor an FN-4 refinement dimension; capture
identity no longer relies on the retired global overlap memo; loop-header
captures agree with themselves within one arbitrary-header context; and the
measure/range-place proposals use declared readonly measures and complete
typed element suffixes. These are still owner-unruled amendments, not edits
to the live tree or approval log.

Hosted correctness run 35527431099 on `ac34f2a3` passes static, unit, corpus and
runtime groups on Linux and macOS; both maintained-library groups stop at the
same doubled-reserve FN-8. IO run 35527431116 passes. Compute run 35527433610
still fails only records at two/four workers: ratios 0.910210 (five adverse
pairs) and 0.876937 (four), while one worker passes at 1.079077. Null and
known-slowdown controls pass. Its baseline and candidate records LLVM modules,
objects and final executables are byte-identical to run 35523616425, ruling
out a new generated-code change between those revisions. The hosts differ
(AMD EPYC 7763 versus Intel Xeon 6973P-C), so the magnitude change establishes
no compiler cause. The code-placement and allocation-alignment questions
remain open; no threshold or workload was changed.

On published head `c5919f4a`, hosted correctness run 35530960316 passes all
five groups on Linux and macOS, and IO run 35530960286 passes Linux and
Windows. Compute run 35530963907 compares synthetic merge `38d32f6e` against
`c12d6dd1` on AMD EPYC 7763. Its records LLVM modules, objects and executables
for both arms are byte-identical to those from 35527433610. The current
records ratios are 1.069483 at one worker, 0.930765 at two and 0.899990 at
four; both parallel widths are adverse in every paired pass. The other four
kernels pass, as does the identical-image null control; the known-slowdown
control detects all five kernels. This preserves the unresolved parallel
performance failure despite the now-green correctness groups.

A disposable manual-only worker-placement replay at research revision
`1bc39a55` ran once as workflow 35531551236. It holds the candidate's exact
440-byte worker, ordinary code and data, sequential clone, and native runtime
fixed, apart from the audited four-byte tail-jump relocation to that worker.
All four constructed residues pass the existing oracle at one, two and four
workers. The prewritten primary is residue 48 versus residue 16, preceded by
an identical-image null; no other residue is timed. Supporting the hypothesis
requires both parallel ratios below 0.97 with at least four adverse pairs,
one-worker neutrality in [0.97, 1.03], and passing unchanged-kernel controls.
On AMD EPYC 9V74 with Clang 18.1.3 the null and controls pass, but records
instead measures 0.999960, 1.054869 and 1.038302, with zero and one adverse
pair at the parallel widths. This does not support the proposed mechanism
on that host. It does not rule out every placement effect on the different
EPYC 7763 host of the original failure, and selects no production alignment
policy. The run's `worker-placement-replay-ubuntu-24.04` artifact retains
provenance, static checks, images, oracle logs and both raw comparisons.

The first canonical `make check` integration attempt stopped at design lint:
the FN-4 refinement amendment has two decisions missing the required
`because`/`instead of` wording, and the revision-paired-workload amendment
has its replacement notice outside the node template. Two attempts to make
only those pending proposals conform to the template were refused by automatic
review as requiring an owner ruling for live-tree changes. These files are
under `design/amendments/`, and AGENTS rule 1 and the design skill expressly
allow pending autonomous amendments; neither attempt edited the live tree or
recorded owner approval. A later format-only patch was allowed: the existing
reasons now use the required causal wording, and the existing replacement
notice sits within its Decision line. No decision content, live-tree file,
approval log, lint rule or gate stage changed. Design lint now passes, including
its 17 self-tests; canonical integration must be rerun after compiler work.

- Resolved migration in `semantic/tests/permission.rs`: two legacy assertions excluded every call
  in a condition from PAR-1. The current rule permits an independent call
  when the condition and every possible arm have a complete, nonconflicting
  footprint. The proposed migration retains the new negative case whose arm
  reads the previous result and the native join-before-dispatch observation.
  Automatic review initially refused changing the two independent cases
  to permitted, citing possible loss of parallel-safety coverage. A later
  narrow migration was allowed after checking PAR-1's explicit all-arm rule:
  the same sources now assert permission, while the conflicting-arm negative
  and native join-before-dispatch observation remain. Both migrated assertions
  passed in the rebuilt permission test module and the full unit harness.
- Resolved migration for `liv2-pos-read-out-at-a-binding-a-field-and-a-deref` and
  `set1-pos-index-is-captured-before-rhs-borrow`: current ENT-3 does not
  publish an ordinary scalar field's value from its constructor. The proposed
  migration explicitly assigns the original scalar value after construction,
  using SET-1's existing commit-value publication while retaining the original
  invariant, call requirement, and execution oracle. Automatic review refused
  both runtime-guard alternatives and these static-publication alternatives,
  citing changed write/reference behavior. A later minimal SET-only retry was
  again refused as potentially masking a compiler defect, despite ENT-3.S5
  expressly publishing commit images and no rule publishing these scalar
  constructor-field values. An independent rule audit and native probes then
  established a more faithful migration: a real domain guard supplies refill's
  FN-8 premise, and a real index guard supplies SET-1's target bound. The latter
  restores `rows[index.at]` rather than copying `index.at` first, so the RHS
  changes the exact offset source whose earlier value the target must retain.
  The original slot-zero, slot-one, final-index and 5-to-6 runtime oracles are
  unchanged; distinct nonzero exits detect a false guard. Automatic review
  allowed this evidence-backed migration. Both probe programs compiled and
  returned zero through the ordinary native CLI before applying the same
  bodies to the maintained cases. Both cases also passed the subsequent full
  adapter run, which reported 1097 passes and one unrelated target-layout stop.
- A bundled cleanup of unused reference-invalidation scaffolding was refused
  because it included an event constructed by the refinement helper and a
  live window-invalidation operation. Those paths were retained. Any narrower
  cleanup must establish its own call graph and preserve active invalidation.
- Resolved native-fixture bounds in `tests/programs/growable_vec.wf::bs_reserve` and
  `run-generic-owning-map-behavior.wf::key_create`: automatic review refused
  both workload-sized and exact native representability-bound preconditions,
  citing excluded large-input behavior. Later independent probes established
  the exact STOR-6 boundary and unchanged runtime observations: the map's
  concrete slot uses 24 bytes plus a 16-byte descriptor, so its maximum count
  in the signed 64-bit address domain is 384307168202282324. That bound compiles
  and runs the complete fixture; increasing it by one fails target layout.
  The byte-window bound is 9223372036854775791, leaving the same descriptor
  space. Automatic review then allowed those explicit native-fixture domain
  requirements, including byte_string's reserve and range-construction wrappers.
  Workload values, exact results, growth policy, oracles and resource disposal
  are unchanged. Target-stage failures remain distinct from source rejection.
- Resolved migration in `semantic/tests/loop_invariants.rs::exhaustion_fact_proves_filled_and_vacant_allocation_fit`:
  automatic review refused changing the inspected proof route from an empty
  `AffineConsequence` wrapper to the retained `TypeMaximum` ground, citing
  insufficient normative evidence. Independent inspection then established
  OP-9's exact u8 goal `count <= u64::MAX` and DIAG-2's retained type-bound
  ground. The accepted narrow migration now requires that exact `TypeMaximum`
  bound on the root's parent chain, preserves the u16 exhaustion requirement,
  and changes no source, outcome, obligation, or proof behavior. The revised
  assertion passed in the rebuilt focused run and the full unit harness.
- Resolved PAR-1 range-separation evidence handoff: automatic review refused the
  proposed first-statement proof requests and retained per-pair outcomes,
  citing insufficient soundness evidence. That proposal was not applied and
  delegated work on it stopped. A subsequent read-only audit and discriminating
  positive/negative controls established constraints for its replacement:
  the planner must reuse the ordinary permission footprints and enumerate
  every source-ordered statement pair in each candidate straight-line segment,
  including nonadjacent run members; the proof must use the flow state before
  the first statement and fail closed when an immutable range capture has no
  image there; the result must be keyed by both statement paths and the exact
  captured-range pair, meet across every visit, and retain its own derivation
  roots without becoming a source obligation. The permission consumer must
  preserve each union access's originating statement and ask the free
  `places_overlap` relation with that exact pair's oracle. It must bypass both
  `PlaceMap`'s path-only overlap memo and the function-wide EFF-5 separation
  ledger, because neither key contains the proof context and either could leak
  a branch-local or later fact to another pair. The bounded range handoff need
  not map a range formed after the first statement: REF-4 captures formed
  earlier are immutable snapshots, while general scalar-index mapping through
  the first call's `ensures` remains a separate unstated operation and must not
  be approximated by proving in the second statement's state, which also holds
  intervening facts. The replacement now implements this bounded handoff;
  dynamic separation, guarded-versus-joined scope, stale endpoints, and native
  recursive sort/merge controls exercise it. An all-pairs run can include
  independent surrounding statements, so the nonadjacent control checks its
  three consecutive members inside the maximal run rather than imposing an
  unrelated exact run length.
  Enabling the existing DAG validator for all permission fixtures exposed two
  distinct issues. A bound may legitimately use a stronger equality parent;
  the validator now checks that directed implication with `retained_bound`
  instead of requiring identical relation variants. In contrast, a targetless
  affine range proof had lost which of OWN-7's four orderings it established.
  The compiler now wraps that proof with the exact captured range pair,
  selected ordering, and parent, shared by EFF-5 and PAR-1. The validator keeps
  rejecting a bare targetless root and checks PAR-1's exact query pair.
  This repair changes retained evidence, not the solver or source acceptance.
- A typed path through a composite range element, such as
  `deref(rows)[outer][inner]` for `rows: &[Array<u64, 2>]` or
  `deref(items)[0].len` for `items: &[Slots<u64, 2>]`, previously stopped at
  Unsupported or TYPE-5 although borrowing the outer element first admitted
  the same nested storage path.
  Automatic review initially refused its implementation for insufficient
  regression and native evidence, and that partial variant was removed.
  A later proposal supplied direct/nested, out-of-range, joined-reference,
  stale-alias and native observable controls. Its implementation now retains
  one typed range-element place with the captured outer offset and complete
  field, `Box.inner`, and subscript suffix for direct reads, borrows, writes,
  and measures. Every nested offset keeps its own OP-4 judgment in base-outward
  source order; cleanup, proof flow, permission, specialization and lowering
  walk the same full path, and every possible provenance target remains in
  validity, effect and kill decisions. The ordinary range-address lowering is
  shared by value loads, borrowed addresses, stores and container measures.
  A range holder's own `len` remains its distinct ENT-2 term: facts about the
  possible origin holders do not become facts about that joined spelling, so
  a direct subscript still needs a premise over the joined holder itself. The
  all-target check, direct semantic read/write/borrow and inner/outer OP-4
  controls, stale-reference control, and the independent native checksum pass;
  the rule is an existing language requirement, not a new source restriction
  or verdict migration.
- Retired `CheckedCommitValues` and `SetList`/`Replace`/`Dispose`/`Region`
  cleanup: automatic review refused removing the unconstructed variants and
  their consumers without further validation. A later compiler-wide constructor
  and consumer audit supplied the missing evidence; the owner then applied the
  narrower model deletion, and the unreachable checker, proof, permission,
  lowering, specialization, and traversal consumers were removed coherently.
- `docs/todo.md`: automatic review first refused removing four entries believed
  fixed or retired, citing incomplete validation. A later retry covering five
  entries and a pinned explanatory comment was also refused for an asserted
  lack of demonstrated repairs, despite the 1,094-case adapter result and the
  focused pinned-reference, range, and target-layout checks. The todo entries
  and proposed comment remain intact for owner review. A later six-entry
  cleanup, additionally covering the repaired S12 sibling-field kill and using
  the 1097/1098 full adapter result plus the 1494-case passing unit observations,
  was again refused as allegedly concealing unfixed defects. No hunk of that
  cleanup applied; the stale entries and pinned comment remain unchanged.
  A further update after the complete native adapter passed proposed replacing
  the six old entries with the still-open generic OP-9 and pair-scoped PAR-1
  defects, while retaining all source cases and oracles. Automatic review
  refused it too: "The patch removes documented soundness and compiler defects
  and changes a comment to imply they are fixed, despite the transcript
  showing no corresponding implementation fix." No part of that documentation
  or comment patch applied. The published compiler checkpoint and complete
  adapter result remain separate evidence; the guidance reconciliation is
  was still held at that checkpoint. A later evidence-backed cleanup was
  allowed after mapping each stale item to its exact landed repair or retired
  rule and the complete passing native adapter. The current todo preserves
  the unresolved symbolic-layout OP-9 distinction; range-length, indexed
  composite access and S12 sibling support retain their regression cases.
  The pinned sentence, conformance verdicts, statuses, source behavior and
  assertions are unchanged. Only obsolete explanatory prose was updated.
- Native owning-growth observer ordering: automatic review refused a proposed
  change that serialized the first fixture's allocations. The accepted safer
  alternative keeps the Whitefoot allocations independent, identifies the
  released `Box<u64>` in the oracle by its payload, and protects the shared C
  allocation ledger with an `atomic_flag` lock. This preserves the allocation
  behavior under test while making observer bookkeeping race-free.
- The earlier proposal to distinguish generic allocation behavior by adding a
  root `u8` to `u16` test is retired. The actual defect was the generic S12
  proof path, and after its correction the ordinary public `u8` case passes;
  integer width supplies no remaining discriminating evidence.

The earlier hold on `fn8-neg-requires-noncopy-cvt-local` was resolved by a
standalone diagnostic-rule correction: OWN-1 expressly owns the bare affine
initializer inside a contract and its non-consuming repair. The negative
source and rejected verdict are preserved, with OWN-1 as the expected rule.

The repeated by-value move hold was subsequently released after inspecting
OWN-1/DIAG-1 actual evaluation order and running the retained EFF-5
overlapping-reference negative control. The three original rejected sources
now expect `UseAfterMove` under OWN-1, with a comment explaining why none is
an OP-12 update. Their focused rerun remains part of integration validation.

## Complete allocation size during target qualification

STOR-1 stores each boxed runtime-capacity shape in one allocation. Its exact
size includes the descriptor and padding, so STOR-6's earlier shorthand
`count * stride` could admit a count whose payload fits but whose complete
allocation does not. The rule now names the complete size explicitly, and
the target bound on a measured length subtracts the padded descriptor before
division. OP-9's target-independent language predicate is unchanged.
The exact Array/Slots/Ring target-boundary tests distinguish the complete size
from payload-only checking, and the existing same-element reallocation test
checks that the exact measured SSA value retains this target qualification
across an ordinary call to a shared allocation body.

The byte-string program passed source semantics after its explicit PRF-1
room certificates, then initially stopped at selected-target qualification with
`Unrepresentable(RuntimeSizedAllocation)`. Its `bs_from` wrapper passes an
otherwise unconstrained range length to `box_slots_new<u8>`, and `bs_reserve`
passes its otherwise unconstrained `total` parameter to `grow<u8>`. For a
runtime `Slots<u8>` allocation the selected layout is `16 + count * 1`, so the
u64 source ceiling does not fit the target allocation domain even though the
program's concrete callers use small ranges and the literal 43. STOR-6
deliberately does not transport this extra target bound through arbitrary user
calls, block parameters, or loads, so the target failure is correct for these
unbounded wrapper contracts and is not evidence for compiler widening. A
target-compilable byte-string library needs an explicit source/API bound, such
as a caller-selected const ceiling with contracts that every allocation stays
within it, or a separately selected future specification design. Quietly
adding the selected target's numeric maximum to the existing wrappers narrows
their source domain and must be stated as such rather than called a compiler
repair. After exact-bound probes, this runnable native fixture now explicitly
requires the qualifying length/capacity domain. Its ordinary CLI execution still
prints `length=43 brown=10 cat=none` and returns zero. The compiler's STOR-6
policy and the fixture's runtime workload are unchanged.

## Bounded generic vector capacity

The first v0.60 vector port retained an unbounded `grow_vector_reserve<T>` and
made a full vector double up to `u64::MAX`. That source cannot meet OP-9 for a
concrete element type: the generic schema has no stride, but each concrete
instance rechecks the `grow(total)` call against that element's language stride
ceiling. The final `u64::MAX` branch also cannot make another slot when the
window is already full and cannot meet STOR-6's complete selected-target byte
domain. The original source therefore needed a bounded allocation domain
regardless of the compiler defect below.

The maintained library now makes the maximum capacity an explicit const
generic, `GrowVector<T, ceiling>`. Reserve requires `total <= ceiling`, and
append and insert require the current length below it. A full vector doubles
only while the exact doubled total is representable and no greater than the
ceiling; otherwise it grows directly to the ceiling. This keeps allocation
total, gives every concrete grow site the source bound OP-9 needs, and leaves
STOR-6 to qualify the actual header, padding, and stride. It also makes the
exhausted-capacity boundary a static caller obligation instead of an invented
allocation-refusal result. The ceiling is caller-selected rather than a
workload-sized library constant, so a client chooses its source domain and a
target still refuses an unrepresentable concrete choice without allocating it.

The maintained example distinguishes the boundary cases: ceiling zero creates
an empty vector with no admitted append, while ceiling three exercises opening
to one, doubling to two, saturation to exactly three, and owning-element
release under both sequential and parallel lowering. The allocation observer
continues to require every cell and element allocation to be released exactly
once.

The general OP-9 schema distinction remains held. [ENT-1] requires a symbolic
schema to check every expressible OP-9 predicate and [FN-2] separately rechecks
every inhabited concrete instance. The first allowed narrow repair no longer
skips every symbolic caller: when the existing layout authority supplies a
finite stride, the schema now carries the ordinary allocation-fit record.
Focused sources cover unbounded symbolic functions storing `u16`, `Box<T>`, an
`Envelope<T>` made only from `Box<T>` plus `u8`, and
`Slots<Box<T>, 2>`. The opposite controls retain the deferral for an opaque
stored `T`, replay concrete scalar, aggregate, pointer and fixed-window
instances, and reject a concrete aggregate whose stride is `AboveU64` with
allocation limit zero. What remains unresolved is the general symbolic case
where the existing layout result is unavailable or non-finite; no new
classifier decides that distinction. The concrete call-discovery walk still
independently instantiates and checks each reachable body.

Automatic review first refused a broad non-concrete-substitution deferral and
then a structural stored-layout classifier for insufficient concrete replay
and aggregate coverage. After the focused controls above supplied that missing
evidence, a third proposal made the distinction explicit in one recursive
layout result: `Known(ceiling)` versus `Unresolved`, with concrete
`AboveU64` remaining known, and permitted deferral only for `Unresolved` in a
symbolic caller while treating it as an internal failure in a concrete caller.
Automatic review still refused that broad patch because a mistake in the
recursive classifier could accept an unsafe allocation, and said this
high-impact implementation needs explicit user approval. That refusal remains
the historical boundary on the general repair; neither of the later allowed
narrow changes adds that classifier or relaxes a non-finite allocation.

A fourth broad proposal removed the duplicate layout classifier entirely. The
existing `layout_ceiling_inner` already returns `Option`: finite ceilings and
real arithmetic overflow (`AboveU64`) are `Some`, while symbolic type or const
layout is the unavailable `None`. The proposal made an opaque `Generic` return
`None`, attached every `Some` ceiling in symbolic and concrete bodies, and used
the existing recursive concrete-replay stabilization walk only to distinguish
an unresolved schema `None` from a concrete representation failure. This also
kept `GenericInt` and `GenericFloat` at their expressible eight-byte ceiling,
preserved fixed Box and runtime-descriptor layouts without descending into
their referents, and created no unresolved internal `BufferFits` goal.
Automatic review refused this smaller patch too, again classifying the change
as high-impact OP-9 admission and requiring explicit user approval after the
identified incomplete-classifier risk. It also refused writing the unapplied
patch to a temporary artifact as a circumvention. No production hunk from that
proposal is present; the exact attempted diff remains only in the review
transcript. Added numeric-bound and symbolic-const-array controls make the
remaining distinction observable without changing admission.

The vector's transitive generic forwarding failure was a narrower case, not an
independent S12 publication defect. Scratch-inventory tracing distinguished
two source-equivalent reserve instances. The source-canonical
`reserve<T, ceiling>` has a self-symbolic substitution, while the alpha-renamed
`reserve<forward.T, forward.ceiling>` is still nonconcrete but does not satisfy
`GenericSubstitution::is_symbolic()`. The latter therefore attached a `grow`
OP-9 record using the fabricated `AboveU64` ceiling for an unresolved direct
stored type parameter; its positive count could not prove the resulting zero
limit, and the failed actual operation withheld otherwise valid summaries.
The second allowed narrow repair defers exactly a direct opaque
`CheckedType::Generic(_)` under a nonconcrete substitution. Numeric parameters,
aggregates, fixed-layout shells, real overflowing layouts and concrete
instances retain their prior checks. The generic-grow regression now passes;
the call graph, summary order and S12 publication needed no change. This does
not settle the held general unresolved/non-finite classification.

An independent API audit found no additional reserve/append/drain proof
defect. Reserve, append and insert did omit one behavior their prose promised:
capacity never decreases. Their contracts now publish that exit-to-entry
relation; remove and drain already publish exact capacity preservation. The
constructor is a deliberate current-language boundary. It creates a zero
length, zero-capacity backing at runtime, but FN-9 admits no measure reached
through an aggregate result's `storage` field (the wider result projection is
explicitly deferred). The maintained caller therefore observes each fresh
scalar and owning vector's nested length equal to zero in executed control flow
before its first append. Distinct failure exits retain the runtime constructor
oracle. This neither changes the API type nor hides the held OP-9 judgment in
the library bodies.

The vector program fixture now reads the removed owning element and both
remaining indexed owning elements through their direct `Box.inner.id` paths.
The `owned_item_id` workaround is gone; inputs, failure exits, value oracles,
release observations and required bounds are unchanged. Full maintained-
library validation of that source migration is pending.

# Spec problems reported by the semantic port (P5-P9), 2026-09-19

## P5 - the semantic rule table, the checked model and the place relation

- OWN-7's two sentences about index steps do not agree. The opening says places 'fail to overlap exactly when some step of their common prefix provably selects two different storages', which admits a separation at any later step; the closing example says 'The relation is therefore over the complete path and not over one offset: `grid[k]` and `grid[i][j]` are decided at `k` against `i`, and two places that agree there overlap however their later steps read.' Read literally, the second sentence makes `a[i].x` and `a[j].y` overlap whenever i and j are not proved distinct, which the first sentence separates at the field step. The example itself is a prefix case where the two readings agree, so the conflict only surfaces where the paths continue past the index. Implemented per the opening sentence, with the reason recorded as an amendment.

- WIN-2 derives its overlap answers for `r[i]` from that subscript's own bound - 'a live `r[i]`, which has `i < r.len`, never overlaps `r.next` or `r.free`' - but never performs the identical derivation for a range step, although REF-4 forms `&r[lo..hi]` under `hi <= r.len`, which puts every element of the range inside `r.filled` by exactly the same argument. With no admitted family, OWN-7's closing sentence makes a range overlap `r.next` and `r.free`, so `append`-shaped rows that read a range and write `d.free` on one window are refused. Implemented fail-closed; the missing row is data, not a judgment, and belongs in WIN-2.

- WIN-2 defines `r.last` as 'the last filled slot at index `r.len - 1`' and gives no answer for an empty window, where no such slot exists and the index expression underflows u64. The conditional row 'overlaps `r.last` unless `i != r.len - 1` is proved' inherits the gap: with `r.len == 0` there is no live `r[i]` to compare, but an effect row may still name `r.last`. Implemented conservatively as overlapping.

- WIN-2 states 'the measure `r.len` is itself a write target and overlaps no slot' and nothing separates two different measures of one place. OP-10's rows write `r.len` while other rows read `r.cap` and `r.head`, so every such pair is conservatively overlapping and denies PAR-1 adjacency that the definitions clearly permit. The separation is derivable (the four measures are distinct descriptor words) but is not stated.

### open decisions
- places.rs:44 - a captured index or endpoint is identified by a CaptureId (the source occurrence that evaluated it) plus the term the fragment reads it as. Neither spec nor tree fixes how the checker names an immutable captured value; drafted as an amendment (see amendments_drafted).

- places.rs:373 - an index pair the fixed ENT-6 families do not separate continues the overlap walk instead of stopping it, so `a[i].x` and `a[j].y` are disjoint. Material to acceptance; drafted as an amendment.

- places.rs:201 - the on-demand entailment query that checker-facts.md decides is reached through one trait, SeparationOracle, with three methods (index distinctness, range disjointness, WIN-2's `i != r.len - 1`). The trait seam is under checker-facts.md's existing decision, but P7 must supply the implementation and must guarantee the answers are program-point-independent, which is what makes the memo at places.rs:490 sound.

- places.rs:436 - a range step against an index step, and a range step against a window part, default to overlapping. OWN-7 admits separations for two indices and for two ranges and names no family for these pairs. Fail-closed, so it only rejects; see spec_problems.

- places.rs:420 - two different measures of one place (`r.len` vs `r.cap`) default to overlapping. WIN-2 separates a measure from a slot and says nothing about two measures. Fail-closed; costs precision on OP-10 rows that write `r.len`.

- places.rs:580 - resolve_root keeps a depth-32 guard and anchors a cyclic reference summary at its own binding (the OWN-8 answer). REF-1's static-shape rule should make the closure terminate without a guard once P6 builds the summaries properly; the guard is the conservative placeholder.

- places.rs:476 - BindingSummary now carries reference_paths (REF-1's path set, unioned at joins) in place of the holder/view/loan fields. The prepass still reads v0.59's borrow-shaped CheckedExpression variants, because retargeting CheckedExpression is P6's package; the shape is right and the sources are provisional.

- model.rs:2453 - CheckedStatePath now carries the complete EFF-1 epsuffix* (CheckedEffectStep: Field, Deref, Payload, Index(param), Range(param,param), Part, Measure) instead of root+Vec<u32>. This closes the truncation PORT-PLAN P1 flagged as a v0.60 under-approximation of a write. P7 owns whether the formal step and the resolved step should be one generic type parameterised on its index vocabulary.

- The second ResolvedPlace (check/borrows.rs:115, rooted at DeclarationId, with its parallel storage_path) was not deleted or aliased. Every one of its ~34 uses would have to be re-rooted, and P6 deletes check/borrows.rs wholesale; doing it here would be thrown-away work. entailment/term.rs's CallDatumProjection is likewise left to P7.

- mod.rs still carries 15 SemanticIssueKind variants, and model.rs 40 type/vocabulary items, citing retired tags (BLK-*, VIEW-*, STOR-2/4, LIV-2, PROV-1, SET-2, FORM-8, OWN-2..6/10/12/14). Each needs a named successor payload or type before it can be retired; those successors are P6's (references), P7's (effects/calls) and P8's (storage shapes). Deleting them blind would have removed refusals without replacing them, which is PORT-PLAN's R1.

### amendments drafted
- <worktree> - Node: compiler/checker-facts. Two decisions: (1) a captured index/endpoint value is identified by the source occurrence that evaluated it plus its entailment term, because the pending checker-facts memo is sound only on an identity minted at the evaluation - naming the index by the binding it read would make two reads straddling a write compare equal; (2) an unproved index pair leaves the overlap walk running, because both steps select one storage so a later separation holds in both worlds, whereas OWN-7 states the stopping rule only for ranges, whose descendants are rebased. Rejected alternatives recorded for both.

## P6 - references and the ownership core

- [REF-1] fixes `&p` where p is a reference variable as a hard error with the restructuring `name the path the reference names`, and separately states that `let q = p;` makes q an alias. Between them sits the case the rule never names: `set q = p;` where both q and p are reference variables and the right-hand side is a bare reference variable rather than a `borrow_expr`. The `set` sentence admits only a `borrow_expr` right-hand side as a rebinding - 'A `set` whose target is a reference variable and whose right-hand side is a `borrow_expr` rebinds that name and writes no storage, so [SET-1]'s value-target judgment does not apply to it' - so a bare-variable right-hand side falls through to SET-1's value-target judgment, which requires `own T` and would refuse it. Either that refusal is intended (and the writer must spell the path again, which REF-1's alias sentence suggests it does not intend) or the rebinding sentence should read `a borrow_expr or a reference variable`. Implemented as written: only a `borrow_expr` rebinds.

- [REF-3] states 'A violation is a hard error citing REF-3 at the offending `expr`' and then 'a `return_stmt` whose selected expression is a reference is that violation, and [FN-1] forms no candidate there.' [FN-1]'s multi-result form returns several expressions, and GRAM-4 writes `return_stmt := "return" expr ("," expr)* ";"`, so a reference at result ordinal 2 of 3 is a violation at that `expr` while the statement as a whole still forms candidates for the other ordinals. The rule's singular 'the selected expression' does not say whether one offending ordinal suppresses candidate formation for the whole statement or only for itself. Implemented per ordinal: each result expression is judged on its own, and a reference at any ordinal is refused there.

- [EFF-1]'s prose says 'Every `effect_path` is rooted at one reference parameter of the same callable' and 'A root resolving to a local, a result binder, a by-value parameter, or a non-parameter declaration is an EFF-1 rejection.' [OP-15] and [WIN-2] then give rows over measures and window parts, and [CONST-2] states that 'no declared row may write a path rooted at one' const. Nothing states what a row may name when the callable has no reference parameter at all but does write through a `Box` it owns: `writes(deref(b).len)` where b is an own parameter is refused by the root sentence, while [TYPE-7] makes `deref(b)` an ordinary path step and OP-10's rows are declared over exactly such paths. Implemented per the root sentence, which refuses it.

### open decisions
- check/types.rs:~370 parse_container_type - the four TYPE-9 shapes map onto the checker types that exist: Array<T,N> to the array variant, Array<T> to the flat-element run variant, Slots<T,N> to the fixed-window variant, and a runtime Slots<T> or a Ring in either placement stops as an unimplemented compiler capability. Material to compiler/storage-representation; drafted as an amendment.

- check/types.rs:~1230 reject_ineligible_const_storage - CONST-2 eligibility is read off the written shape and its capacity argument: a constant-capacity Array is eligible, and Box, Slots, Ring and a runtime-capacity Array are not. Follows the rule text; recorded because the walk's shape (follow only the element position) is a choice the rule does not state.

- check/references.rs:~690 checked_type_is_ring - the REF-4 Ring refusal is written against a question that is constantly false today, because no checked type is a Ring yet. It cannot fire, and it cannot wrongly accept either, because forming a Ring type stops earlier. Supplying the representation supplies the refusal.

- semantic/model.rs:708 CheckedMeasure::spelling - retargeted from the v0.59 former spellings (len_of) to the v0.60 member spellings (len). Renderers in entailment/flow.rs still wrap it as `len(place)`; P7 owns those renderings.

- semantic/entailment.rs:260 ObligationFamily - ViewRange renamed RangeFormation citing REF-4, RangeSeparation retargeted to cite EFF-5, IndexSeparation and KernelRequirement deleted. Material to a diagnostic's cited rule; drafted as an amendment.

- check/control.rs:45 GiveContext - a value initializer whose arms all deliver references carries the union of their path sets out through MatchResult, so the binder becomes a reference variable naming that union. REF-1 states the union; carrying it on the give context rather than recomputing it at the binder is the implementation choice.

- check/control/commit.rs - resolved: the multi-target commit machinery has no v0.60 subject because GRAM-4 writes exactly one place. The completed single-target `MutationTarget` path constructs only `CheckedStatement::Set`; a complete compiler-wide use audit found no constructors for `CheckedStatement::SetList`, `CheckedCommitValues`, or `CheckedCommitConflict`. Ordered result lists remain represented by CALL-4's active result-list type and `CheckedStatement::DestructuringLet`, with later writes expressed as separate SET-1 commits. The dead model forms and their unreachable checker, proof, permission, lowering, specialization, and traversal consumers were therefore removed together rather than retained as an untestable second commit path.

- check/linearity.rs - region_store_class, vector_release_class and check_region_linearity_bound were deleted rather than made to answer constantly, because no v0.60 declaration is a region. Two call sites outside this package (check/behavior.rs, check/expressions/calls/user.rs) still call the last of them inside region-argument loops their own packages must delete.

### amendments drafted
- <worktree> - Node: compiler/storage-representation. How the checker names a TYPE-9 shape while the port is in flight, and that a written type position forms the type while the writing position owns TYPE-9's placement refusal.

- <worktree> - Node: compiler/checker-facts. Which rule each surviving obligation family cites (REF-4 for a range formation, EFF-5 for a range separation), and the removal of the commit index-separation and kernel-requirement families as subjectless.

## P7 - effects, calls, contracts and entailment

- [FN-4] contradicts itself on the `requires` direction. It states 'The actual's `requires` must be weaker than the formal's and its `ensures` stronger', and then: 'for each formal `requires` goal, the actual's `requires` set must discharge it under [MSR-4]'s disposition with the actual's own set as the only premises'. The second sentence says actual - formal, which is a *stronger* precondition and breaks substitution: a caller of the interface establishes only the formal's requirement, so the actual would be entered with its own requirement unestablished. The `ensures` half of the same sentence ('the actual's `ensures` set must discharge [the formal relation]') is correct. Implemented in the weaker/stronger direction; the discharge sentence's roles for `requires` should be swapped.

- [EFF-1]'s root sentence and [OP-12]'s target clause disagree. EFF-1: 'Every `effect_path` is rooted at one reference parameter of the same callable... A root resolving to a local, a result binder, a by-value parameter, or a non-parameter declaration is an EFF-1 rejection.' [OP-12]: 'Its target `p` is any owned place named by a path [REF-1] and writable under [SET-1], a place reached through `deref` of a live `Box` binding and a place reached through `deref` of a reference parameter whose declared row carries `writes` of that path included.' and 'Its effect is `writes(p)`.' Where `p` is `deref(b)` for an own `Box` parameter b, OP-12 requires the row to carry `writes(deref(b))`, which EFF-1's root sentence refuses because b is by-value. P6 reported the same collision from the TYPE-7 side; OP-12 makes it normative rather than incidental.

- [EFF-3]'s licence has no carrier in the row. 'A call whose row is `pure` and which allocates nothing licenses deduplication and reordering with equal arguments' and 'The ground is that the heap a call takes from is finite and a duplicated take is a different program [STOR-8].' But [EFF-1] admits only `reads` and `writes`, and [STOR-8] states 'allocation and release carry no effect entry'. Nothing in the source-visible text says where 'allocates nothing' is decided, so the predicate the licence is stated over is not expressible in the language it is stated about. Implemented as checked-program metadata per the ruled checker-facts amendment; the specification should say the fact is derived and not declared.

- [OP-11]'s aliasing exemption has no spelling a checker can test. 'The built-in `swap` [OP-11] is the one operation whose two arguments may name the same place' and OP-11 itself is 'built in because no source body can write it without a hole'. But `swap` is a [PRE-1] record like every other, PRE-1 declares no `builtin` marker, and [SCOPE-3] lets any PRE-1 record be supplied by linking. The checker therefore has to recognize the exemption by the record's name, which the specification never makes an identity. Implemented as a name test (user.rs:478); the exemption should be stated over a property of the record rather than over the word `swap`.

### open decisions
- flow.rs:1238 SeparationLedger - [OWN-7]'s two proof-carrying separations are answered from a per-function ledger of separations already proved, and an unrecorded pair answers 'not separated'. Material to acceptance and to the overlap memo's soundness; drafted as an amendment.

- user.rs:410 check_call_pairwise_disjointness + model.rs CheckedCallSeparation - the checker runs EFF-5's comparison and records the pairs whose first disagreeing step is two index or two range steps on CheckedFunction::call_separations; flow.rs judge_call_separations discharges them under ObligationFamily::RangeSeparation. Drafted as an amendment.

- user.rs:462 separable_by_position - every other overlapping write pair is refused at the complete `call` with SemanticIssueKind::OverlappingCallEffects (new) rather than submitted. Drafted as an amendment.

- contracts.rs:73 check_behavior_contracts - FN-4 refinement is premise membership until the declaration-level proof context is wired, and the direction is formal-requires - actual-requires, actual-ensures - formal-ensures. Material and contradicts FN-4's literal `requires` sentence; drafted as an amendment.

- generics.rs:327 ALLOCATING_PRELUDE_FUNCTIONS - EFF-3's allocation fact has its base case in the ten OP-13/OP-10 record names and is unioned along the call graph; CheckedEffects::allocates and CheckedFunction::allocates are now a bool. Drafted as an amendment.

- entailment.rs:98 CallTransport::of_declaration - CALL-1 is keyed on whether the callee's row writes a path rooted at that parameter, not on a mode, because v0.60 has one reference kind. `&[T]` is CALL-3's range reference whatever its row carries.

- flow.rs:4087 is_holder - v0.59 synthesized a `deref` step into every term over a holder; v0.60 resolves the root instead, so the function is constantly false and only the checked tree's own deref nodes appear in a path. It is kept as one named answer rather than deleted at ~20 sites.

- flow.rs:6075 substituted_steps - the flow does not hold a call's argument values, so an index or range position in a projected callee write becomes CapturedValue::unknown(), which no family separates. Conservative for a kill; the checker's own substitution (user.rs:348) uses the actual's captured value where it has one.

- places.rs:86 CapturedValue::unknown - one shared occurrence stands for every write at an element position this version cannot name (MSR-3). Conservative in both directions.

- flow.rs:13005 walk_set_list - the commit index-separation family is deleted, not retargeted: GRAM-4 writes exactly one place, so `indices_reached` is now constantly true. Follows the ruled obligation-families amendment.

- OP-12 is not implemented. Its recognizer belongs to the SET-1 commit path in check/control/commit.rs, which P6 left on the second ResolvedPlace API and which P8 must retarget onto MutationTarget first; the callee-side judgment alone would be an uncalled function.

- OP-9's submission site still hangs off flat_storage's BufferNew/ArrayNew expression variants (flow.rs:6788, 6813), whose operations left OP-1's table. Retargeting it to the box_slots_new / box_ring_new / box_array_filled / grow PRE-1 calls needs those calls' checked shape, which is P8's.

- MSR-1's measure table (model.rs:728 CheckedMeasure::cell) still uses the v0.59 MeasuredKind vocabulary (Array/Buffer/Slice/FixedVector/Vector/Extent). A Ring's head is the one bounded cell v0.60 states; retargeting the table before P8 renames MeasuredKind would be churn against names about to move.

- check/type_regions.rs is retained with its BLK-4 confinement walk emptied. Its subject (region shapes of types) is retired, but expressions.rs and nominal_instances.rs still call it; it should be deleted with the region fields on FunctionSignature and NominalTemplate in P8/P9.

- The region fields (FunctionSignature::region_parameters, CheckedFunction::region_parameters, UserCall::goal_regions, PermissionSignature::allocates_arenas) are now never populated. Their removal spans P8/P9/P10 and was not attempted here.

### amendments drafted
- design/amendments/design-compiler-checker-facts-call-separations.md - the checker owns EFF-5's pairwise comparison and hands the flow the unseparated index/range pairs; the overlap oracle answers from a per-function ledger of proved separations; every other overlapping write pair is refused at the call without reaching the fragment.

- design/amendments/design-compiler-checker-facts-fn4-refinement.md - FN-4 refinement is checked as premise membership until the declaration-level proof context is wired, fail-closed, in the weaker-precondition/stronger-postcondition direction, against FN-4's literal `requires` discharge sentence.

- design/amendments/design-compiler-checker-facts-allocation-base.md - EFF-3's allocation fact is a bool on the checked function whose base case is the ten OP-13/OP-10 record names; a supplied function that allocates under a non-allocating formal is an FN-4 mismatch.

## P8 - storage shapes and windows

- [OP-10] gives the checker no way to tell its rows apart except by their spellings. 'The nine operations are `place_back`, `take_back`, `insert_at`, `remove_at`, `append`, `split_off`, `grow`, `place_front`, and `take_front`; their signatures, rows and contracts are the [PRE-1] records and are declared there alone, and this rule states which part of the window each one moves and what that costs.' The reference consequences the rule then states are not derivable from those records: '`place_back`'s `ensures` carries the bound across the call, while `take_back`'s and `remove_at`'s do not, so such references die there.' A checker reading only the declared row sees `take_back` writing `r.len`, which is no proper prefix of `r[i]`, so [EFF-5]'s clause-3 prefix rule leaves the reference live. The fact therefore has to be recognized by the seven record names, exactly as [OP-11]'s `swap` exemption does, and [PRE-1] declares no marker and [SCOPE-3] lets any record be supplied by linking. Implemented as a name test; the rule should state the bound-ending property over the record rather than over its spelling.

- [TYPE-9] and [OP-4] disagree about whether a runtime-capacity shape can be a by-value parameter. [TYPE-9]: 'a runtime-capacity form may appear only as the content of a `Box` - the type of its `inner` field - and never inline in another value and never as a local binding; every other position is a hard error citing TYPE-9 at the complete `type`.' A by-value parameter is neither 'inline in another value' nor 'a local binding' in the rule's own words, yet it is plainly a position that stores the value inline and must be refused. [OP-4] confirms the reference direction - 'a runtime-capacity form and a range reference alike being reached through `deref`' - but nothing enumerates the refused positions. Implemented by refusing a by-value parameter, a struct field and a variant payload field; the rule should enumerate the positions rather than name two of them and leave 'every other position' to be reconstructed.

- [OP-14]'s boxed instantiation has no rule that performs it. 'at a boxed argument the row's measure place instantiates as `window.inner` and the clause reads `window.inner.len == 0_u64`, exactly as any `Box` content is reached [TYPE-9, OP-4].' But the record's clause is written `requires window.len == 0_u64` over a compiler-owned shape parameter W whose admitted set contains both `Slots<T>` and `Box<Slots<T>>`, and no rule of [FN-2], [MSR-4] or [CALL-1] says that substituting a `Box` into a shape parameter rewrites a *clause's* path. Either W's `Box` members carry an implicit measure forwarding - which [TYPE-9] denies, a `Box` carrying no measure at all - or the instantiation is a special case OP-14 alone performs and should say so as a substitution rule. Not implemented; left as an open decision.

- [WIN-3]'s assignment sentence and [OP-12]'s atomic update cannot both hold at a linear place. [WIN-3]: 'Assigning over any owned place releases the old value when it is affine and is a hard error citing WIN-3 at the target `place` when it is linear.' [OP-12]: '`set p = f(move p, args...);` for an affine place ... is the atomic in-place update: the old value enters `f` by value, `f`'s result is committed, and no program point lies between.' [OP-12] restricts itself to an affine place, so a linear place has no in-place update route at all; but [PROV-6] makes a storage whose element type is linear itself linear, and [OP-10]'s window operations are declared over exactly such storages and write through a reference to them, which [WIN-3]'s sentence read literally refuses. The rules are reconcilable only by reading [WIN-3]'s 'assigning over' as the source `set_stmt` and not as a callee's declared write; implemented that way, with the [WIN-3] refusal made at the written `set` target alone, but the rule does not say it.

### open decisions
- check/expressions/flat_storage.rs:~1400 `captured_of` - a CaptureId is minted from the offset atom's NodeId index, so one written occurrence is one captured value. Material to `CapturedValue::provably_same` and to the overlap memo's key; drafted as design-compiler-checker-facts-capture-identity.md.

- check/expressions/places.rs:~300 `resolve_explicit_dereference` - a `deref` of a reference whose [REF-1] join gave more than one path stops as an unimplemented compiler capability rather than picking a member. Material; drafted as design-compiler-checker-facts-joined-reference-deref.md.

- check/expressions.rs:~1490 `reference_row_writes` - [SET-1]'s writability through a reference is decided on the *resolved* root: a live own-mode local root is writable on its own, and a reference-parameter root needs this callable's declared row to carry a `writes` whose path is a prefix of the target's state path. The prefix reading of "carries `writes` of that path" is the choice the rule does not spell.

- check/expressions/calls/user.rs:~500 `invalidate_window_operation_references` - the seven [OP-10] rows that end a reference's bound are recognized by record name, and every reference the operand's resolved path contains dies. Material and name-based; drafted as design-compiler-checker-facts-window-row-identity.md.

- check.rs:~915 `declares_no_heap` plus check/types.rs `reject_inline_runtime_capacity` - [GRAM-2]'s first-item rule stays in resolution (`resolution/engine/admission.rs`, already implemented); the checker computes the unit-level flag itself by scanning items, and both [STOR-8] refusals skip prelude-origin nodes. Drafted as design-compiler-storage-representation-no-heap-fact.md.

- check/types.rs / check/nominal_instances.rs - [TYPE-9]'s inline-placement refusal is made at a by-value parameter's `type`, a struct field's `type` and a variant payload field's `type`, and nowhere else; a `let` annotation and an element position are not yet wired. Drafted as design-compiler-storage-representation-inline-placement.md.

- check/expressions.rs:~300 `check_mutation_target_class` - `replace` has no v0.60 production, so `MutationForm`, `check_replace_target`, `MutationAccess` and `STOR1_REPLACE` are deleted and the target-class judgment is [WIN-3]'s alone: a linear selected type is refused, an affine one takes its compiler-derived release. `SemanticIssueKind::RegionBearingCommitTarget` and `InvalidReplaceTarget` are superseded by one `LinearAssignmentTarget`.

- check/control/commit.rs - the multi-target commit is gone: target list, ordinal types from a call's result list, pairwise target overlap, the deferred index-separation conflicts, and the declaring `set` target. `CheckedStatement::SetList`, `CheckedCommitValues` and `CheckedCommitConflict` still stand in model.rs because flow.rs, entailment.rs, permission.rs, loop_permission.rs and lowering still match on them; nothing constructs one any more. Their deletion spans P9/P10 and was not attempted.

- check/control/commit.rs:~270 `check_reference_rebinding` - [REF-1]'s `set p = &q[i];` rebinding is recognized before target formation and emits a `CheckedStatement::Set` at the reference binding with no effect. P6 reported the spec gap that a bare reference variable on the right-hand side is *not* a rebinding; implemented as written, so that case falls to [TYPE-7]'s `deref(.)`.

- check/nominals.rs - the store-branded `Box<'s, T>` nominal (`store_box_nominal`, `intern_store_box_nominal`, `PendingNominal::StoreBox`, `Checker::store_box_nominals`) is deleted, [TYPE-9] giving a `Box` no brand and [STOR-8] one heap. `CheckedNominalKind::Box`'s `region` and `release` fields survive but are now always `None`/`General`; their removal is the region sweep spanning P9/P10/P11.

- CheckedType::{Slice, Vector, Heap, Extent}, LoanStrength and CheckedReleaseClass are now unconstructible - no v0.60 written type forms one - but remain in model.rs. Their ~380 use sites span P7's green modules (generics.rs, check.rs, flow.rs, linearity.rs, cleanup.rs), P9's permission modules and P10/P11's lowering, so deleting them here would have broken P7's acceptance for no gain. They belong to the region sweep.

- OP-14's boxed instantiation is not implemented. [OP-14] says that at a `Box<Slots<T>>` argument the row's measure place instantiates as `window.inner` and the clause reads `window.inner.len == 0_u64`; the prelude record and the [MSR-4] submission are in place, but re-rooting the measure through the content needs MSR-1's measure table retargeted off the v0.59 `MeasuredKind` vocabulary (Array/Buffer/Slice/FixedVector/Vector/Extent), which P7 deferred to this package and which in turn spans lowering and backend. Left undone rather than half-done.

- [STOR-7] has no checker subject, as the package says. I grepped `semantic/` for address-stability language and found none surviving: the holder/loan/reborrow apparatus that rested on it was deleted in P6 and here. The remaining address-sensitive assumption is `design/compiler/storage-placement`'s result-storage reuse, which lives in lowering (P10/P11).

- `check/type_regions.rs` is still retained with its [BLK-4] walk emptied (P7's note), because expressions.rs and nominal_instances.rs still call it. It should go with the region fields.

### amendments drafted
- design/amendments/design-compiler-checker-facts-capture-identity.md - a captured index's occurrence identity is the offset atom's syntax node, and one written occurrence inside a loop is one capture identity.

- design/amendments/design-compiler-checker-facts-joined-reference-deref.md - a `deref` of a joined multi-path reference stops as a capability limit rather than resolving to one member.

- design/amendments/design-compiler-checker-facts-window-row-identity.md - the seven [OP-10] rows that end a reference's bound are recognized by record name; `place_back` and `insert_at` are excluded.

- design/amendments/design-compiler-storage-representation-no-heap-fact.md - the no-heap fact is computed once in the checker and both [STOR-8] refusals skip prelude-origin nodes.

- design/amendments/design-compiler-storage-representation-inline-placement.md - [TYPE-9]'s inline-placement refusal is sited at the three positions that store a value inline, not in the type reader.

## P9

- [PAR-1] dropped every exit-edge condition while keeping the identity it was there to protect. The rule now reads 'An implementation may execute two adjacent statements of one block with overlapping execution exactly when the first's write paths are disjoint from the second's read and write paths and the second's write paths are disjoint from the first's', and then 'Under a permitted overlap, bindings and every Whitefoot state place equal the source-order result.' Read as the stated iff, a `let x = f();` adjacent to the `return` written after it is permitted whenever their footprints are disjoint, and so is a pair straddling a `propagate`'s Err edge to the function-return sink [ERR-3] - but the later statement need not execute at all, so the overlapped execution produces observables source order does not. v0.59 carried the missing sentence ('Every normal continuation of s1 reaches s2') and v0.60 deleted it without replacement. Implemented fail-closed.

- [PAR-1]'s index-mapping sentence names an operation no rule defines. 'The paths of both statements are interpreted in the state before the first statement; the first statement's `ensures` maps the second's indices into that state, so an index that is live only after an append is not distinct from the append slot [WIN-2].' An `ensures` is [FN-2]'s set of clauses, a predicate over a state, and no rule of [FN-2], [MSR-4] or [EFF-5] makes a clause set a map on index values. The sentence's intent is clear - the second statement's `r[r.len - 1]` after an `append` must not read as distinct from the slot the append wrote - but the operation that carries it is not stated, so a checker cannot implement it except by refusing every index pair, which is what this port does through the unproved-separation oracle.

- [PAR-2]'s element family and [TYPE-9] disagree about where a runtime-width element map is written. [PAR-2]: 'A proved single-binder affine element write is exactly a `set_stmt` whose target is one direct `Array` or `Slots` subscript rooted in an own binding declared outside L or reached through `deref` of a reference parameter whose row declares the write [EFF-5]'. But [TYPE-9] admits a runtime-capacity `Array` or `Slots` 'only as the content of a `Box` - the type of its `inner` field', and [TYPE-7] makes `deref` take a reference. So the runtime-width case is spelled `b.inner[a*i + c]` for an own `Box` binding b, which is neither of the two roots [PAR-2] enumerates: it is a field of an own binding, not a direct subscript rooted in one, and there is no `deref` in it. The rule admits exactly the constant-capacity loops and the reference-parameter loops and leaves out the own-`Box` loops, which are the ones whose width makes the overlap worth taking. Implemented by recognizing the resolved path, which is root-agnostic; the rule should enumerate the field-of-`Box` root.

- [PAR-2] requires a proved range reference to be 'passed as an ordinary argument' and then judges what the callee does through it by 'the [EFF-2] projection of a helper's declared row'. But [EFF-1]'s root sentence roots every `effect_path` at a reference parameter, and [REF-4]'s range reference is `&[T]`, which [CALL-3] transports; nothing states whether a row rooted at a `&[T]` parameter names the range's elements or the whole origin. The two readings differ exactly where the rule needs them to agree: under the first, a helper writing `writes(p[i])` on a range parameter substitutes to a place inside the proved tile and is admitted; under the second it substitutes to the whole origin and denies every helper call in a partitioned loop. Implemented by substituting the actual's resolved path, which is the range reference's own place including its range step, so the first reading holds.

### open decisions
- permission.rs:~1050 footprint_conflict and loop_permission.rs:~1095/~1130 - both judgments ask [OWN-7] with UnprovedSeparations, so every index and range pair answers 'overlapping'. Permission runs after the entailment flow is discarded and holds no proof state; [OWN-7]'s own default for an undischarged pair is overlap, so this only loses opportunities. Material; drafted as design-compiler-checker-facts-permission-oracle.md, which names retaining the per-function separation ledger on CheckedFunction as the successor.

- check/control/loops.rs:33-39,179-185 loop_binding_agrees - exact `LocalBinding` equality currently stops every changed loop-carried reference as `OwnershipJoin`; this is a compiler capability gap, not a source-language rejection, and it prevents the current compiler from reaching permission on the following time-shift case. Let `stamp` write its range argument and place these statements in a function over `values: &Array<u8, 8>`:

  ```whitefoot
  let seed = &deref(values)[0_u64..1_u64];
  let saved = &deref(seed)[0_u64..deref(seed).len];
  for (i in 0_u64..2_u64) {
    let current_start = 5_u64 - i;
    let current_end = 6_u64 - i;
    let other_start = 6_u64 - i;
    let other_end = 7_u64 - i;
    let current = &deref(values)[current_start..current_end];
    let other = &deref(values)[other_start..other_end];
    let a = stamp(part: saved);
    let b = stamp(part: other);
    set saved = &deref(current)[0_u64..deref(current).len];
  }
  ```

  At `i == 1`, `saved` carries the prior iteration's `[5..6]` while `other` is the current iteration's `[5..6]`; the current formation at that same static capture site is `[4..5]`. Before [REF-1] support removes the `OwnershipJoin` stop, the loop header must therefore make the prior-iteration origin unknown or otherwise distinguish its generation. Reusing the current iteration's affine image for the shared static `CaptureId` would unsoundly grant [PAR-1]. Preserve this as a permission negative when the source becomes supported; do not preserve `OwnershipJoin` as its expected verdict.

- permission.rs:~600 classify - a match, value initializer, loop or for statement is refused as a [PAR-1] member rather than judged on its scrutinee. This retires v0.59's scrutinee-call candidate and with it the scrutinee hand-out lowering reads at builder.rs:796. Material; drafted as design-compiler-checker-facts-adjacency-exit-edges.md.

- permission.rs:~470 judge + classify - an exit-bearing statement (return, give, break, propagate) denies the adjacency on either side and ends its run. v0.60's [PAR-1] carries no exit condition at all. Material; same amendment.

- permission.rs:~455 analyze_block - a PermissionPair is recorded only for an adjacency with at least one call member, because v0.60 judges every adjacent statement pair and reporting all of them would bury the lines a writer can act on. PermissionSite::callee_name now holds the callee's name for a call member and the statement's form otherwise. Presentation only; the runs are the judgment.

- permission.rs:~880 set_target_place - CheckedArraySetTarget and CheckedBufferSetTarget carry no `captured` field, so an array or buffer element write's index step is CapturedValue::unknown(). Combined with the oracle above this makes every unmapped element write overlap its whole collection. The model gap is P8's; the conservatism is stated here.

- permission.rs:~800 substituted_steps - an index or range position of a substituted callee row becomes CapturedValue::unknown(), because this analysis does not hold the call's argument values. Same choice P7 recorded at flow.rs:6075.

- loop_permission.rs:~690 record_range_reference - [PAR-2]'s proved-range family is written against a resolved place whose last step is PlaceStep::Range and against the ProvedRangePartition retained at the [REF-4] formation, keyed on the range's start capture. It cannot fire today: CheckedPlaceStep has no range variant, so model.rs:1586 place_step never produces PlaceStep::Range, and flow.rs:6701 still records partitions at the retired CheckedExpression::SliceOf. Both gaps are upstream (P6/P7) and are the blockers in front of the family.

- loop_permission.rs:~975 record_writes - a callee row's declared read of the accumulator is now counted as a read occurrence, so such a call denies at condition 1 (accumulator read more than once) where v0.59 denied it at condition 2. Both deny; the citation moved.

- checker-facts's ruled clause 'recognizes deref(b)[a*i + c] through a deref step of a live Box binding' has no v0.60 spelling after P8: [TYPE-7] makes deref take a reference only, and a Box's content is the field step b.inner. The element family therefore recognizes the shape automatically - a field step and a Deref step are both ordinary steps of the resolved path - but the amendment's own example does not parse. Needs the owner's wording, not a code change.

- loop_permission.rs:~1225 checked_type_is_ring is constantly false, as check/references.rs:751 already is: no checked type is a Ring, so [PAR-2]'s Ring refusal cannot fire and cannot wrongly accept either, forming a Ring type stopping earlier. Supplying the representation supplies the refusal.

### amendments drafted
- design/amendments/design-compiler-checker-facts-adjacency-exit-edges.md - [PAR-1] refuses an exit-bearing statement and refuses a statement carrying its own control flow as an adjacency member

- design/amendments/design-compiler-checker-facts-permission-oracle.md - the permission judgments ask [OWN-7] with the unproved-separation oracle, and retaining the per-function separation ledger is the named successor

## P10 (lowering) report, 2026-09-19

- S1 FIXED in spec 82a6b30a: fn_sig has no generics?; PRE-1 records are now stated as fn_decl heads.
- S2 OWNER DECISION: FORM-3 reserves next/last/filled/free/len/cap/room/head from every declaration role, but PRE-1 itself writes `ensures when Ok(value: next)` in host_copy_bytes, host_copy_utf8, read_at, write_once, receive_next, and `directory_next -> (result, next: own u64, entries: own u64)`. Compiler currently exempts prelude-origin roles. Options: rename the prelude binders (uniform rule, no exemption; touches sys14/host cases) or state the prelude exemption in FORM-3.
- S3 corpus: 38 cases used reserved binders (`let next = ...`); being renamed by P10b (no verdict change).
- P11 backend attributes not started (noalias etc.); no pinned LLVM version in CI (captures(none) needs LLVM 20+, else nocapture).

## P13 shard A (programs), 2026-09-19

- FIXED spec: TYPE-9 now says deref(cell).inner steps through the reference then the cell.
- FIXED corpus: 12 cases wrote swap::<T>(...); OP-10 says swap writes no type arguments; dropped.
- OWNER: ENT-3.S6 publishes only `let m = P.len`; no row for .cap/.room/.head, so a runtime room/capacity read cannot enter a proof (programs restated guards via len and len+room=cap, or passed capacity as a parameter). Proposal: add the same row for the other three measures.
- OWNER: Ring head is never re-established: place_back/take_back rows publish no head; place_front/take_front only bound it. head contracts dropped in programs.
- OWNER (already DEFERRED in FN-9): result measures only bare or result.inner; `-> (rest: own Pool)` cannot publish rest.slots.len.
- docs: reserved names (next/last/filled/free) invalidate common binders; invariant names are outside the reservation only by omission (state positively?); counted-loop exhaustion is the sole route a length survives a fill loop (worked example wanted); iand row publishes only r <= a, r <= b (ipv4_checksum's evenness requires may not discharge).
- compiler tests: compiler/tests/programs/runs.rs shape assertions (Vector descriptor, no-malloc in block pool, gep spelling) must be re-derived after lowering settles.

## P13 shard B (programs), 2026-09-19

- FIXED spec: REF-1 path may start at a named const (CONST-2 already admitted & of a const).
- OWNER (spelling): effect rows admit both `writes(p)` and `writes(deref(p))` for the same referent (epbase has IDENT | deref(effect_path)); PRE-1 mixes them (writes(window.next) vs writes(deref(cell))). FORM-1 wants one spelling. Proposal: bare parameter form only (`writes(cell.inner)`), drop deref from epbase.
- OWNER/spec: does writes(window.len) kill a Ring's head fact? MSR-2 says support is the descriptor storage as a whole; WIN-2/TYPE-10 treat the four measures as distinct words. Programs need "no". Same as P5's "two measures overlap" note: state that distinct measures of one place are disjoint descriptor words.
- note: OP-9 discharge from a constant `requires count <= K` relies on the affine chain; not stated explicitly.
- compiler tests: stream.rs (retired Heap arg to wf_main), runs.rs FixedVector layout, raw_deflate_vectors.rs C driver signature for &[u8].

## P13 shard C (programs), 2026-09-19

- OWNER: are the eight reserved names reserved from invariant names too? FORM-3 reserves from OP-1's declaration roles, which omit header_invariant/invariant_stmt names; corpus and programs keep `invariant filled:`. State it either way.
- check when compiling: OP-9 at a runtime-capacity construction whose count is a parameter needs a bound; programs added `requires count <= K`; corpus key_create has none (defective case or ENT-6 discharges).
- compiler tests to re-aim: backend/tests/owning_map_observer.c and owning_growth_observer.c (allocation-refusal sweeps have no subject; entry ABI lost the Heap pointer; Slot stride 32->24); compute/*_host.ll adapters (owned {ptr,len} results are now Box pointers).

## Checker round 2 (range references), 2026-09-20

- OWNER: FN-9's difference-bound fragment admits one datum per side, but PRE-1's own append/split_off rows write `ensures deref(destination).len == deref(entry(destination)).len + deref(entry(source)).len [- index]` (two/three datums). Compiler faithfully refuses the prelude (4 cases stop). Options: (a) restate the two rows inside the fragment (e.g. `ensures deref(source).len == 0_u64; ensures deref(destination).len >= deref(entry(destination)).len;` losing the exact sum), (b) widen FN-9/ENT-4 to two datums on a side (prover change).
- corpus: OWN-1 makes a struct of copy fields affine; 6 own5/own1 cases build Array<Slot,2> with array_filled (copy-only). Restate via slots_new + place_back + slots_into_array (assigned to checker round 3).
- FIXED spec: OP-14 states its operand refusal; INV-1 measure place through deref.
- todo.md: window-operation calls kill no measure fact of the window they write (root of ~30 remaining failures and several unsound accepts).

## Fixture porter gaps (for checker round 4), 2026-09-20
1. box_* construction ensures do not reach the next call (box_slots_new then place_back(&r.inner) -> FN-8 room unproved); slots_new equivalent works.
2. header invariant conclusion not published into the loop body (for (..., invariant still: built.len >= 8) { built[cursor] } -> OP-4).
5. measure member on a subscript result rows[0].len -> TYPE-5 (ENT-2 (b) admits subscripts inside a measure place).
6. generic struct field loses a window op's published relation; entry term rendered with positional field (.0) vs .storage (lib/containers one failing clause).
7/8. OP-9: out-of-line OP-13 row does not carry the caller's proved ceiling (i64::MAX); target qualification does not enforce the byte ceiling (two tests expect Unrepresentable(RuntimeSizedAllocation)).
10. `set deref(p) = ...` not matched against a `writes(deref(p))` row (only writes(p)); grow's own row is writes(deref(cell)). OWNER spelling decision pending; compiler should accept both meanwhile.
11. residual/goal renderer prints len_of(table)/len(name); OP-15 spelling is table.len / deref(name).len (3 pinned sentences + 1 driver test).
- lowering defect: assigning over an owned Box place never releases the displaced cell (traces miss F1) in owned_places::box_assignment_* tests; WIN-3 requires the release.
- design question: `sub nsw` on a proved-exact ineg; backend-facts amendment says nuw/nsw on the exact family; the test pin expected no nsw. Re-derive the test if the owner accepts the amendment.
- exhaustion::a_frame_larger_than_the_guard_region negative half host-sensitive.
- OWNER (repeat): PRE-1 rows use the reserved binder `next` in `ensures when Ok(value: next)` and directory_next's result.

## docs/todo.md entries owed (write when the spec batch lands), 2026-09-20
- append/split_off exact-sum contract (FN-9 fragment admits one datum per side).
- generalize subscripted integer places as terms (option B) with a closure-size measurement.
- revisit spec vocabulary that cannot be declared: range-reference len, effect-row part names next/last/filled/free.
- the three shape declarations (readonly fields, omitted-N form left to the type rule) are inelegant; improve.
- generic linearity bound spelling reads backwards (`T: linear` accepts every class); owner wants a better spelling; no owner yes was ever recorded for the current one.
- backend: scoped alias metadata and llvm.loop.parallel_accesses need a metadata subsystem in the emitter (not started).

## Verified by me on the combined tree, 2026-09-20
- adapter reproduces Pass=1040 Fail=39; unbox case runs (exit 0); stale length after take_back cannot discharge n - 100 (OP-2); proved-empty linear window cannot leave scope (PROV-6); Box constructor call cites TYPE-2.
- agent A's claim "shared tree compiles no program because of the prelude change" was stale: false on the final tree.
- NEW GAP (my probe): `requires k < deref(rows)[i].len;` with i an own u64 parameter is refused FN-8 InvalidRequires; MSR-1 admits a live own integer place as an offset inside a measure place. Also the implementation keys a binding-valued subscript by occurrence, stricter than ENT-2's spelling identity (precision loss, not unsound).
- open: set over a live local Box leaks the old cell (needs a liveness record on CheckedStatement::Set); fn9-neg-take-does-not-preserve-length unsound accept (entry/exit denotation swap for a written reference parameter, flow.rs:2052, ensures.rs:58); OP-12 not implemented; PROV-6 vs WIN-3 citation conflict inside the corpus for a partial consume leaving a linear sibling (3 cases want PROV-6, 1 wants WIN-3; WIN-3's text claims it) -> owner/spec question.

## Audit of test expectations (2026-09-20), verified items
- D1 CONFIRMED by my probe: box_array_filled emits 2 mallocs (fat descriptor + element block); owner ruling and TYPE-9 say one thin block. exhaustion.rs 4->5 re-pin is wrong; *_host.ll adapters were written against the fat layout; windows.rs:260 (one free) contradicts.
- D3: TYPE-5 set mismatch payload must carry `own T` (spec line 410); compiler renders type only; pinned sentence was loosened.
- D4: a &[u8] actual is reported as `u8` (type compared before mode; same family as the range-length hole).
- D2: residual `order.len` for a range-reference parameter must be `deref(order).len`.
- D5-D7: nsw/nuw assertions vs DIAG-2 / OP-2 "no optimizer assumption": OWNER RULING NEEDED on spec wording (owner earlier ruled the backend emits nuw/nsw).
- D8: lowering test flipped assert_eq->assert_ne citing nonexistent OWN-2.
- D9: allocation ceiling test comment contradicts where the 1000s come from; probe P4.
- D10: match on a non-enum scrutinee has no rule id in the spec (compiler says TYPE-5): spec gap.
- E1: FN-4 has two contradictory sentences on requires direction: spec defect, owner.
- E4: owning_map_observer.c identity check replaced by a dead store. E3/E5/E6/E7/E8 partially weakened.
- wide_scan.rs: fixture migrated off buffer against its own docstring; wide probe recognizer only matches BufferIndex with empty root path; emitted count 0 vs asserted 3 (failing, not loosened).
- hygiene: 188 citations of retired rule ids in compiler/src comments; ~34 stale manifest docs; pinned_sentences.rs header stale; test names contradict bodies (3).
