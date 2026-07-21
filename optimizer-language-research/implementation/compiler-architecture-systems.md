# Production Compiler Architecture — Systems and Evidence

Status: OWNER-APPROVED ARCHITECTURE WITH BLOCKERS. This file is part of the
single [Production Compiler Architecture](compiler-architecture-design.md)
design record. `THE-PLAN.md` is execution authority. This file does not amend a
numbered specification, protected semantic surface, profile, or entrance gate.

This file uniquely owns Decisions 15 through 18. Cross-stage authority, entrance gates,
execution order, owner decisions, and exit status remain in the parent index.

## Decision records

### Decision 15: resource bounds, determinism, and failure atomicity

**Decision:** Every invocation carries a versioned `ResourceProfile` whose
fields are checked before work and allocation. The implementation also has a
reviewed hard maximum; callers may tighten but never loosen it. Numerical
release values must be evidence-selected and committed before the corresponding
stage implementation, not improvised inside algorithms.

**Problem being solved:** Safe Rust prevents memory corruption, not memory/time
exhaustion, host recursion overflow, output floods, nondeterministic ordering,
or partially published artifacts.

**Specification and project constraints:** The compiler must be bounded,
deterministic, and failure-atomic. A resource failure is not a normative
rejection and may not waive a rule.

**Selected design:** Use versioned caller-tightenable profiles under reviewed
hard maxima, per-stage semantic work counters, an audited allocation discipline,
and atomic per-invocation-directory publication. A limit check and a successful
capacity calculation do not by themselves make a later allocation fallible.

**Selected profile fields:**

| Stage | Required explicit ceilings |
|---|---|
| ingress | sources, logical-path bytes, per-source bytes, total bytes, binding bytes |
| lexical | token bytes, tokens, trivia/lexemes, scan work |
| syntax | classified tokens, nodes, depth, parser-stack entries, list members, expected terminals, work, tree bytes |
| resolution/type | scopes, declarations, uses, spelling bytes, type terms/depth, const digits/elements, substitution args, normalized nested-region slots, lexical/instance-slot region actuals, finite region-fact profiles, region-call environments and composition steps, symbolic/concrete rechecked nodes, template/concrete typed-call records, typed-call boundaries, semantic/code-instance keys and mappings, seed/follow steps, work |
| CFG/ownership | template/concrete functions, blocks, edges, operations, place depth, bindings, region-tree nodes/depth, outlives and storage-root/scope queries, owner-tag validations, holders, loans, A-16 claim relations, call-provenance boundaries, state cells, per-value/total origin members, origin-term depth, provenance summaries/equations/edges, substitution expansion, provenance SCC iterations, edge-state and state-origin products, cleanup records, loop checks, provenance work, total work |
| effects/graphs | row entries, template/concrete graph vertices and derived call edges, call-SCC certificate records/edges/iterations, work |
| artifact | records by family, strings, proof steps, reference depth, source/tree bytes, total encoded bytes |
| target/backend | layout depth, aggregate/frame bytes, ABI records; untrusted backend-profile encoded bytes, record/string/path bytes, argument count/bytes, environment entry/name/value bytes, linked-input/output/postprocessor counts, held handles, decode/validation work; observation files/bytes/hash work; plan instructions, retained guards, report-table entries/bytes, symbols, constants, LLVM text/object/link/runtime bytes, audit work |
| external tools | CPU and wall time, address-space/resident memory, stdout bytes, stderr bytes, sandbox work/temp/cache bytes, output-file bytes, open files, child count, process-tree depth |
| publication | expected entries/path bytes, publication-staging files/bytes, combined work-tree bytes, import/copy/hash/sync work, stale directories/bytes, recovery scan/delete work |
| optional facts | facts, proof bytes, dependency edges, verifier work, authorized consequences |

Stage names such as `FactResourceProfile` denote validated, read-only typed
views of this one invocation `ResourceProfile`, not independently supplied or
caller-forgeable limit sets. Each view carries the parent profile identity and
can only tighten the reviewed hard maxima.

`BackendProfileResourceProfile` is available before decoding untrusted
`BackendInvocationProfile` bytes. Framing rejects an encoded-length excess
before allocation; bounded decoding then accounts for every record, string,
logical path, argument/environment byte, linked input, requested output,
postprocessor entry (required to be zero in the baseline), retained handle, and
validation/hash-work step. Artifact byte limits do not substitute for these
pre-compilation limits.

Limits are inclusive: `actual <= maximum` is admitted. All count products and
byte capacities use checked arithmetic before reservation. Authority-bearing
paths pre-count where possible, reserve exact capacity with fallible APIs, and
prefer slices, offsets, arenas, and sorted vectors over node-allocating maps.
After reservation they use only operations proven not to grow. Infallible
`collect`, formatting, implicit clone growth, `Box`/`Arc` conversion, map-node
insertion, bigint growth, and dependency-internal allocation are forbidden on
such a path unless a path-and-version-specific audit proves the allocation is
already covered or the API is replaced with a fallible boundary. Algorithms
use iterative stacks and bounded, deterministically ordered worklists. Every
stage counts semantic work in addition to nominal input size so a small
adversarial graph cannot hide quadratic expansion.

**Input contract:** Valid hard profile, optional tighter request, exact inputs,
and an output publication destination selected by the invocation but operated
only inside the downstream publication component.

**Output contract and established invariants:** Within an audited in-process
authority path, either one complete immutable stage result or one
resource/input/invariant outcome is returned; never a partial capability. At
the process boundary, a supervisor treats abnormal termination, allocator
abort, signal, OS kill, and dependency crash as a tool-process failure. Those
events are not falsely reported as a classified in-process allocation refusal,
but the publication protocol still prevents partial durable authority.

**Explicit non-responsibilities:** Resource profiles do not change language
semantics, make a program unsupported, authorize truncation, or excuse missing
records. `REPRESENTABLE` limits are not production defaults.

**Why this stage owns the work / why adjacent stages do not:** Each stage knows
its true growth dimensions. The supervisor owns process limits and the
downstream publication component owns durable-publication limits. A global byte
cap alone cannot bound semantic graph work.

**Alternatives considered and rejected:** Host allocator termination is not a
controlled compiler result. Recursion depth tied to host stack is unstable.
Timeout alone is nondeterministic. Publishing then validating exposes partial
authority.

**Trusted assumptions and threat model:** The OS, filesystem, allocator,
subprocess limiter, and audited dependency versions remain TCB components.
Hostile input and artifacts maximize depth, width, state products, graph
cycles, hidden allocation, child processes, temporary storage, and output.
Full controlled in-process allocation failure is not claimed until the
path/API/dependency audit is complete.

**Failure modes:** A deterministic semantic limit reports exact
field/maximum/actual. A failure returned by an admitted fallible allocation is
`AllocationFailure`; counter overflow is a compiler/resource outcome. An
allocator abort, OS kill, or uninstrumented dependency allocation remains a
supervised `CompilerProcessFailure`, not `AllocationFailure`. Unsupported host
enforcement of memory, disk, file-descriptor, or process-tree limits rejects
the requested release profile before compilation.

The durable publication unit is one invocation manifest plus every output it
names: base/composite artifacts, report projection and templates, LLVM text,
runtime metadata, and any requested object or executable. An approved
`PublicationHostProfile` fixes same-filesystem rename/sync semantics, a private
staging root, canonical file/directory metadata, and an enforceable subprocess
filesystem sandbox. Backend validation records only non-operational
`PublicationHostEvidence`: the profile/root identity and successful support
probes, with no writable handle or create/write/rename method. A downstream
`whitefoot-publication` component privately owns the reviewed
directory-handle-relative, no-follow, exclusive-create, metadata, sync, and atomic-no-replace
implementation. Any safe wrapper dependency or platform helper implementing
those OS primitives is a named, pinned TCB component under Decision 17; a host
lacking the required operations is rejected before compilation and all
properties are revalidated under newly opened private handles at commit.

Each external tool runs in a per-invocation private work directory under an
OS-enforced sandbox whose only writable filesystem roots are that directory and
its explicitly declared temp/cache subdirectories. All temp/cache environment
variables point inside it, undeclared path access is denied, stdout/stderr use
bounded pipes, and the disk quota counts the tool work tree, temp/cache data,
and publication staging together. Tools never write the publication directory.
The host supervisor contains, terminates on failure, and reaps the complete
process tree; import starts only after no sandboxed child remains. The stage
owning the supervised invocation then imports only named outputs inside that
same private operation: it opens each
entry relative to the work-directory handle without following links, requires
a stable regular file with the expected single-link status, bounded size, and
unchanged pre/post-copy metadata, then streams bytes into a newly
exclusive-created runner-owned regular file. Symlinks, hardlinks, directories where a
file is expected, sockets, devices, FIFOs, extra entries, path escapes, and
races reject the invocation.

Authority-bearing tool inputs are exact-byte capabilities, not paths or
digests. `ValidatedBackendInvocation` owns driver-controlled immutable
`PinnedLaunchImageSet`/`PinnedLinkInputSet` bytes (plus OS-backed evidence for
the already loaded compiler), so a source pathname, hardlink, or writable file
descriptor cannot change a later launch/link input. The retained-guard text
validator never accepts caller-supplied text. `whitefoot-llvm-emit` alone
borrows `CodegenPlan` and returns `LlvmEmissionOutput`, owning its exact complete
text and plan/compilation/backend binding. The separately implemented guard
validator consumes only that capability and wraps it in
`GuardAuditedLlvmModule`; it cannot relabel arbitrary guard-shaped IR as
plan-derived. Only `whitefoot-llvm-run`'s pinned runner accepts the audited module.
That runner's private launch/wait/bounded-import operation alone constructs
`LlvmRunOutput`, owning the exact imported object candidate and binding it to
the audited module, pinned LLVM image and closure, plan, compilation, and
backend profile. `whitefoot-object-audit` consumes only that capability and can
then wrap it in `GuardPresenceAuditedObject`; it cannot attach arbitrary object
bytes to the expected LLVM invocation.

Only `whitefoot-link`'s pinned runner accepts `GuardPresenceAuditedObject`
together with the exact pinned link inputs. Its private
launch/wait/bounded-import operation alone constructs `LinkRunOutput`, owning the complete exact
linker-output candidate set and binding it to the object, linker image and
closure, every pinned link input, plan, compilation, and backend profile.
`whitefoot-output-audit` consumes only `LinkRunOutput`; after validating its
closed set it wraps that run result in `LinkBoundOutputSet`.
`whitefoot-publication::finalize_and_commit` accepts only that capability plus
the exactly matching opaque `FinalizedCompilation`. It performs final
cross-output validation and internally reprojects static artifacts, reports, and
runtime metadata rather than accepting caller-supplied bytes. Only on success
does it create crate-private `PublicationReadyOutputSet`, owning the closed
requested byte/metadata set and manifest. Each runner materializes inputs from
capability-owned bytes into fresh exclusive driver-controlled files. Raw
emitter/tool drafts are discarded at every boundary. Baseline postprocessor
count is zero. Receipts/digests are diagnostic comparisons only: bytes, a
mutable path, or reconstructed receipt cannot enter any private factory.
Substitution anywhere from input pinning through final output validation is an
invariant failure and prevents publication.

The requested output-set contract, embedded in `BackendInvocationProfile`,
fixes every logical relative path, regular-file/directory kind, executable or
read-only canonical mode, and an empty/closed xattr and ACL policy; its baseline
postprocessor inventory is empty. The publication component sets and revalidates
those modes, ownership, link counts, xattr/ACL absence,
entry set, lengths, and hashes; tool-created metadata is never carried into
publication. Numeric UID/GID and wall-clock timestamps never enter portable
identity: ownership must equal the active publication principal under the host
profile, and timestamps are normalized or explicitly ignored by all validators
and consumers. It also requires the running Whitefoot compiler/build manifest,
lowering dossier, plan schema, output validator, guard/runtime objects, and all
external tools to equal the profile. `whitefoot-output-audit` owns the sole
link-output validation, while final output validation and publication authority
are co-located downstream. `whitefoot-publication::finalize_and_commit`, taking
`LinkBoundOutputSet` and `&FinalizedCompilation`, is the only public publication
operation. It obtains no
filesystem capability from its caller and never returns its private ready set.
After final validation it opens the configured root without following links,
revalidates its identity, same-filesystem placement, host semantics, and policy
against the set's backend profile and `PublicationHostEvidence`, and keeps all
operational handles crate-private. It materializes the set's owned bytes and
metadata into a fresh staging directory, writes its canonical manifest, and
revalidates every staged byte and metadata field. The sole commit point is an
atomic no-replace rename
of the **whole staging directory** to its final `CompilationId`-named path,
followed by parent-directory sync. Consumers use the same no-follow validation
and trust only a committed directory whose closed entry set, manifest, embedded
full-output identity, kinds, modes, link policy, lengths, and hashes validate.

A crash before the directory rename leaves only that invocation's staging
directory; a crash after it exposes the complete validated set or, under the
host profile's durability semantics, no committed set—never a mixture. There
are no pre-commit durable content blobs and therefore no orphan-blob leak.
Concurrent publication uses atomic no-replace: a loser validates an existing
byte-and-metadata-identical `CompilationId` directory as idempotent success,
while any different entry, kind, mode, policy, length, hash, or identity is an
invariant failure and is never overwritten. Startup/recovery traverses by
directory handle without following links and removes only positively identified
stale staging/work directories under explicit count, byte, age, scan-work, and
deletion-work ceilings; a hard combined quota prevents repeated crashes or tool
temp/cache output from growing storage without bound. Same-filesystem atomic
no-replace rename, file/directory sync, sandbox enforcement, and
crash-durability guarantees are stated by the approved host profile.

**Independent evidence required:** Exact and one-over every field,
checked-product overflow, deep/wide hostile shapes, fallible-allocation injection,
audited hidden-allocation scans, dependency-version review, repeat-run
determinism, environment/path perturbation, subprocess memory/disk/FD/process
floods, a crash at every staged-file/manifest/sync/directory-rename step,
symlink/hardlink/device/FIFO/socket/extra-entry mutants, mode/xattr/ACL and
owner/link-count mutations, path-swap races, temp/cache/root escape attempts,
loaded-image/pathname mismatch, source mutation between observation and use,
hardlink/writable-fd tool substitution, dynamic-library closure substitution,
link-output/receipt/staged-byte substitution, nonzero baseline postprocessor,
idempotent republish, conflicting destination, repeated-crash combined quota,
bounded no-follow recovery, and no-partial-authority tests.

**Resource and determinism bounds:** The profile plus audited allocation and
publication paths define the bound. Claimed complexities are lexer/parser
linear after grammar repair, resolution/type log-linear plus explicit term
work, semantic CFG proportional to counted state-transition work, artifact
replay proportional to explicit records and checked local/graph work, and
lowering proportional to accepted artifact plus emitted output. CPU/wall limits
may affect only the class of tool failure; every successful semantic result and
published byte remains deterministic.

**Dependencies on unresolved specification questions:** Numeric release values,
join-state representation, artifact schema, target/frame contract, supported
target set, approved host-filesystem profiles, and enforceable subprocess
limits.

**Migration or foundation-audit consequences:** Retain current lexical hard
profile as evidence, not the whole-compiler profile. Audit every authority-path
allocation and dependency call; refactor infallible source, binding-codec,
formatting, shared-owner, map, and bigint growth before making a complete
controlled-failure claim. Add transactional publication before any emitted
artifact is treated as durable authority.

**Approval status:** The architecture is adopted. Each release and target
profile remains separately gated; changing a resource ceiling alone does not
change the numbered language specification.

### Decision 16: independent evidence system

**Decision:** Build every stage with a preregistered independent evidence lane
and hostile mutants. Use the joined evidence matrix below as a minimum, not a
menu.

**Problem being solved:** A second test program that shares the same table,
algorithm, or expected-output producer can reproduce the same bug and falsely
look independent.

**Specification and project constraints:** Conformance expectations are
compiler-independent and protected. Additive tests are allowed; weakening or
regenerating authority is owner-gated. Passing examples never prove generality.

**Selected design:** The evidence matrix and explicit independence rationale
are mandatory entrance criteria for every implementation tranche.

| Stage/claim | Independent oracle and hostile mutants | Why independent | Evidence bound |
|---|---|---|---|
| source envelope and codec | separately implemented decoder; truncation, length, order, path, byte, and domain-hash mutations | different implementation over pinned wire vectors | every field boundary; total bytes under artifact/source profile |
| shape lexer | existing Python byte model; exhaustive byte contexts and source-boundary/resource mutations | different language and algorithm; no Rust token tables | all 256 byte values in authored contexts plus seeded bounded corpus |
| terminal partition and grammar | separately runnable candidate-grammar verifier: mechanical extraction plus nullable/`FIRST₂`/`FOLLOW₂`/strong-LL(2) `SELECT₂` audit, independently extracting Earley/GLR `zero|one|many` oracle, extraction/priority mutants | engines share exact specification/proposal bytes only; oracle owns separate source binding, extraction, token membership, and grammar representation and imports neither audit decisions nor production parser tables | bounded extraction/tokens/chart items/packed alternatives/work; a bound hit is inconclusive; a `SELECT₂` overlap is a predictive collision, not by itself a full ambiguity proof; proposal-transition evidence is separate from the whole-grammar parser gate; complete-derivation claims are limited to the stated oracle domain |
| derivation tree | independent leaf-coverage audit and tree-driven renderer; orphan, reuse, interval, token, punctuation, and cross-source mutations | renderer/auditor do not call parser builders | all production shapes plus bounded generated trees |
| multi-file composition | split/merge/reorder metamorphics, incomplete cross-file item, empty-file, bundle-root mutations | hand-authored transport properties, not parser output | sources/items within source profile |
| lexical resolution | bounded scope-event model; wrong target/order/namespace/shadow/reservation/coverage mutants | separate simple sorted-event algorithm | exhaustive small scope graphs plus seeded larger graphs |
| types and typed labels | hand-authored operation/signature matrices and bounded term evaluator; wrong kind/type/owner/member/order, symbolic-vs-concrete disagreement, and nested/generic storable-type mutants | data and evaluator authored independently of checker dispatch | every closed type/operation/member/storage form, template bound, and depth boundary |
| constants and integers | arbitrary-precision mathematical oracle; sign/range/leading-zero/const-dependency mutations | oracle does not use Rust primitive conversion semantics | every primitive boundary and bounded digit/array cases |
| floats | independent exact decimal/IEEE conversion and shortest-roundtrip oracle; NaN/infinity/zero/exponent mutations | distinct conversion implementation or vetted external reference | exhaustive selected bit regions plus seeded bounded corpus |
| CFG/control flow | bounded structured path enumerator; missing/duplicate/wrong origin/successor/reachability and all-functions-template/concrete-coverage mutants | constructs paths from a separate compact control model | every edge kind, zero/parametric functions, generic-to-zero-to-generic SCCs, generic-to-zero borrow/slice returns, and exhaustive small symbolic/concrete nesting |
| ownership/regions | focused operational model and bounded state exploration; region class/tree/outlives, OWN-10 storage-root/scope, live actual, owner-discriminated holder/loan identity, move/copy/match-binder claim transfer after A-16, overlap, suspension, body-to-return-contract conformance, staged typed/provenance call-boundary substitution, slice authority, join, recursion, and loop mutations | representation/algorithm distinct from production CFG checker | exhaustive region-pair and storage-root/scope classes; copied-shared/moved-unique/match-binder holder cases after A-16; finite roots/places/origin sets/loans; template and concrete swapped-caller/callee, cross-function, raw-cross-owner, premature-place/origin-in-typed-boundary, stale-creation-holder, multi-holder-loan, wrong/under/overbroad contract, slice-view expired-source/copy/move/join mutants; exact/one-over member, substitution-product, equation-edge, SCC-iteration, and provenance-work bounds |
| drops/releases | independent path-to-cleanup multiset/order interpreter; missing/extra/reordered/trap cleanup mutations | derives only from modeled scope exits and owner states | every exit kind and exhaustive small scope trees |
| effects and recursion | independently derive template/concrete call edges from the corresponding typed-call records, run distinct SCCs and syntax/effect folds, and require the A-18-resolved FN-6 gate before instance work; edge/row/substitution/grounding/component/order/witness mutants target production outputs | different graph algorithm and explicit approved rule; production call-edge/SCC tables are never oracle inputs | exhaustive small template/concrete call-record graphs including type-growing, const-only, and mixed type/const-generic cycles; kind/arity/vector mismatches; several violating edges/cycles; canonical shortest-witness tie breaks and insertion-order permutations; exact proof that semantic-instance work remains zero while A-18 is unresolved or the gate rejects; bounded seeded graphs |
| instantiation closure | independent finite graph reachability from every zero-type/const empty-key function seed after all-source-function template coverage; semantic keys contain normalized nested-region slots plus the finite A-17 fact profile, baseline code keys wrap semantic keys one-to-one, and every incoming typed-call environment uses tagged lexical/instance-slot actuals with bounded whole-boundary-graph composition; key/parent/mapping/composition/missing/duplicate/unreachable/FN-6-order/region-recursion mutants | no checker worklist or instance table reuse | exhaustive small graphs, region-only seeded functions, unused zero/parametric function bodies, generic-to-zero-to-generic SCC/summary dependencies, `f<T> -> g<T>` slot forwarding, same-erased-type/different-region calls, second-incoming-edge profile mismatch, caller-owner nesting attempts, stable profile-preserving slot cycles, local-region recursion that reuses finite semantic/code keys, forbidden unverified `EmissionShapeKey` merging, expanding type-cycle rejection before enqueue, and profile boundaries |
| target qualification | independent layout/ABI calculator and foreign ABI fixtures; size/alignment/offset/frame/overflow mutants | separate implementation plus external calling convention observations | every type/layout form and exact frame/object boundaries per target |
| backend invocation validation | self/tool/runtime byte substitution, decoded-ID-only forgery, build/dossier/schema/output/host mismatch, path-race and capability-rebinding mutants | validator owns live observations and immutable copied-byte/loaded-image capabilities; candidate profile/driver cannot author them | every profile field, encoded-byte/record/string/path/arg/env/input/output/handle/work limit at exact and one-over, tool/loader/library/runtime object, target compatibility field, and host/sandbox operation |
| artifact projection/replay | byte-level omission/duplication/order/reference/coverage mutants and exact reprojection | mutations are independent; replay itself is explicitly not semantic independence | every record kind and exact/one-over artifact limits |
| final composite/envelope | base/backend/compiler-build/plan-schema/lowering-dossier/output-set/publication-host/overlay/report substitution, hash-cycle, decoded-view, sealed-report, and authority-forging mutants; independent byte reprojector | mutation/reprojection harness is separate; reused semantic replay/fact judgments are declared shared, not independent | every envelope field/tag/reference plus exact/one-over composite limits |
| semantic kernel | compiler-independent conformance, focused models above, metamorphics, and seeded semantic mutants | expectations/models do not come from the kernel or catalog dispatch | every normative rule/facet lane with stated finite/fuzz bounds |
| diagnostics/failure authority | compact independent stage/order/location model; BundleRoot, multi-defect, wrong-rule, fallback-serialization, and report-hash mutants | model and golden byte writer do not call production outcome selection/formatter | every closed failure family, ordering tie, location form, and serialization bound |
| CodegenPlan | independent accepted-unit reference interpreter and plan mutation; field-ownership audit | interpreter does not consume LLVM plan decisions | every semantic operation/edge/target form and plan limits |
| LLVM/runtime | LLVM verifier, differential execution, ABI fixtures, trap/report checks, retained-guard text structure/dataflow audit, narrower object reference/site-presence audit, exact-byte/plan/profile capability substitution, pass-erasure mutants, cleanup fault injection, poison review | external tool/runtime observations and hand-reviewed hazards; pinned LLVM semantic preservation remains an explicit TCB assumption, not something the object audit proves | every operation mode, retained/omitted check, edge, report, validated-text-to-runner and audited-object-to-linker boundary, and named target/backend profile |
| link/final output chain | audited-object/runtime/startup substitution, forged receipt, linked-byte swap, missing/extra output, metadata/manifest mismatch, mutable staging-path and postprocessor mutants | link-output audit owns `LinkBoundOutputSet`; publication owns final validation and its private ready set; `LinkRunOutput` is provenance only | every linked input/output, requested output/metadata field, exact-byte transition, and baseline-zero postprocessor count |
| optional facts | proposition-specific near misses/verifier mutants, scope/target/backend-context rebinding, exact/one-over resource and invariant-outcome classification, initial fallback versus final-revalidation mutants, empty-overlay differential | verifier distinct from producer; base execution is control | every fact/consumer pair, both authority/audit APIs, and every explicit proof/work limit |
| Rust dependency/capability boundary | resolved Cargo graph/API snapshots, compile-fail forgery clients, forbidden re-export/feature/env/build-script/proc-macro/unsafe-dependency mutants | policy tool and external Rust compiler inspect production artifacts but are not linked into them | every allowed/forbidden direct edge, public authority type/factory, feature, target, and dependency kind |
| determinism/publication/resources | environment/path/insertion-order permutations, exact/one-over limits, allocation injection, crash-point replay | harness perturbs non-semantic inputs and filesystem/process outcomes | every profile field and publication step |

The terminal-partition and grammar row is also the required evidence path for
a proposed grammar change. Its standalone entry point compares exact full
current-specification bytes with exact full, non-authoritative proposal bytes;
it never writes the numbered specification surface. The static auditor and the
separately implemented generalized oracle independently extract and source-bind
their inputs, and one canonical report binds both input hashes, both extraction
coverages, and any disagreement. The current registry's `deref(p)` witness and
the separately registered proposal case `deref(x)` must reproduce `many` under
exact v0.8. The latter case binds exact bytes, exact shape tokens, entry
nonterminal `expr`, and the isolated-case end sentinel. The pending
fixed-terminal/`IDENT` proposal must reach `one` through
the `expr/atom/place/pbase` fixed-`deref` alternative, remove the named
conflict, and introduce no unreviewed conflict. Retained unrelated blockers
remain explicit, so this transition result is not a claim that the whole
grammar is ready. A proposal report neither amends nor approves a
specification. The verifier requires the dossier's pinned exact-v0.8 hash and
binds the full proposal hash. After approval, the same entry point must require
byte-for-byte equality with the reviewed proposal bytes and recomputed SHA-256
equality with the bound proposal hash, independently extract those bytes, and
prove a disjoint terminal partition and conflict-free complete strong-LL(2)
static audit. Authored oracle cases must match their preregistered derivation
counts; within the bounded generated domain, static recognition must correspond
to generalized-oracle `one` and rejection to `zero`. The bounded oracle makes
no claim outside those cases. These conditions must pass before parser work
begins. Neither evidence engine is a normal-compilation dependency.

The following table assigns the same keyed rows to production owners and makes
shared premises explicit. A shared premise is part of the TCB/evidence
assumption; it never counts as an independent oracle merely because two
programs deserialize it.

| Stage/claim | Production owner | Shared premises that do not count as independence |
|---|---|---|
| source envelope and codec | `contract`/ingress | approved wire schema, domain tags, resource profile |
| shape lexer | `whitefoot-lexer` | exact source bytes and lexical-shape specification |
| terminal partition and grammar | syntax classifier/parser | approved grammar text and terminal inventory |
| derivation tree | syntax finalizer | approved production/node mapping and source binding |
| multi-file composition | ingress + syntax finalizer | owner-approved numbered-spec compilation-unit and FORM-2 rules |
| lexical resolution | semantic resolver | approved namespace/visibility rules and prelude inventory |
| types and typed labels | symbolic/concrete semantic type checker | approved type/operation/member declarations and kind/contract environments |
| constants and integers | semantic constant checker | numeric types, rounding/range rules, resource limits |
| floats | semantic constant checker | approved float spelling and IEEE target format |
| CFG/control flow | template/concrete semantic CFG builder | approved grammar origins and control/evaluation rules |
| ownership/regions | semantic region/provenance/ownership checker | approved region/outlives, place-overlap, provenance, transition, and join rules |
| drops/releases | semantic cleanup checker | approved storage/normal-edge cleanup rules |
| effects and recursion | semantic effect/graph checker | approved effect/recursion rules and template/concrete typed-call schemas; production call edges/SCCs are outputs, not shared premises |
| instantiation closure | semantic instance checker | every zero-type/const empty-key function seed including region-only functions, all-source-function template semantic coverage, A-17 normalized semantic keys/fact profiles, one-to-one baseline code wrappers, validated typed-call environments, concrete calls, and resource profile |
| target qualification | semantic target qualifier | approved target profile and ABI/runtime contract |
| backend invocation validation | `whitefoot-backend-invocation` | candidate profile bytes, opaque validated target borrow, non-operational host evidence, and compiled-in build/dossier/schema manifests |
| artifact projection/replay | semantic projector/replay + artifact schema | approved canonical schema and checked judgments; replay shares the semantic kernel |
| final composite/envelope | `whitefoot-compilation` | artifact schema, semantic replay/audit context, fact verifier, backend profile, and report schema |
| semantic kernel | `semantics` | numbered specification and approved successor decisions only |
| diagnostics/failure authority | stage-local outcome producers + driver serializer | approved DIAG rules, location schema, and closed ordering contract |
| CodegenPlan | compilation plan builder | accepted views, target/backend profiles, selected overlay, sealed report contract |
| LLVM emission provenance | `whitefoot-llvm-emit` | CodegenPlan and backend/lowering contract; returns exact `LlvmEmissionOutput`, never raw public text |
| LLVM text audit | `whitefoot-llvm-audit` | `LlvmEmissionOutput`, retained-guard text contract, and exact originating plan/backend bindings |
| LLVM run provenance | `whitefoot-llvm-run` | `GuardAuditedLlvmModule`, pinned LLVM launch image/closure, supervised process result, and bounded named-output import |
| object audit | `whitefoot-object-audit` | `LlvmRunOutput`, retained-guard object contract, and exact originating plan/backend bindings |
| link run provenance | `whitefoot-link` | `GuardPresenceAuditedObject`, exact pinned link inputs/linker image, supervised process result, and bounded closed-output import |
| link-output audit | `whitefoot-output-audit` | `LinkRunOutput` and closed linker-output contract; returns only `LinkBoundOutputSet` |
| final output and atomic publication | `whitefoot-publication` | `LinkBoundOutputSet` plus exactly matching opaque `FinalizedCompilation`; static-output reprojection, private `PublicationReadyOutputSet`, root opening, handles, staging, revalidation, and rename stay inside one call |
| optional facts | fact-family producer and independent verifier | originating or audit-only opaque context, accepted/audited base identity, validated/decoded target/backend identities, closed proposition and resource contracts |
| Rust dependency/capability boundary | each production crate + Cargo policy gate | approved direct-edge graph, public capability matrix, lock/dependency policy |
| determinism/resources/orchestration | stage owners + driver/supervisor | approved resource, subprocess, and host-filesystem profiles; the driver composes but does not interpret outputs or access publication primitives |

Every matrix row preregisters at least one positive, negative, exact-boundary,
one-over-boundary, malformed, and targeted mutant case; it also preregisters a
differential or metamorphic case wherever an executable observation exists.
If a class is genuinely inapplicable, the registration says why rather than
silently omitting it. Each receipt records exact specification/schema/source
identities, oracle and producer revisions, dependency/tool versions, target and
resource profiles, exhaustive domain or recorded seeds, mutant inventory,
outputs, and whether any protected file or frozen digest changed. A protected
change requires its separately logged approval; an additive case is labeled as
such.

A tranche exits only when every responsibility it adds maps to a matrix row,
all preregistered cases run within their declared limits, every targeted mutant
is killed by the intended independent lane, receipts are durable, no protected
authority was weakened, and hostile review accepts the stated independence.
Shared declarative tables may be correct TCB inputs, but two consumers of one
table are one premise, not two independent confirmations.

No row may count same-kernel artifact replay, the static catalog, a producer
report, or another program importing the same soundness predicate as independent
semantic evidence.

**Input contract:** Exact specification/source identities, authored expected
properties, fixed generation seeds or exhaustive finite domains, bounded
profiles, and declared independence rationale.

**Output contract and established invariants:** Every production responsibility
maps to at least one positive, negative, boundary, malformed, mutant, and where
applicable differential/runtime observation. Receipts bind exact inputs, tool
versions, profiles, and outputs.

**Explicit non-responsibilities:** Evidence assets do not dispatch compiler
semantics, close facets by their presence, edit expectations, or become
production dependencies.

**Why this stage owns the work / why adjacent stages do not:** Tests are designed
with the failure boundary they attack. Adding them after implementation biases
the oracle toward implementation shape.

**Alternatives considered and rejected:** Snapshot-only testing misses
generality. Compiler-generated expected verdicts are circular. Shared parser or
semantic tables invalidate an oracle's independence. Dogfood success is not a
soundness proof.

**Trusted assumptions and threat model:** Hand-authored or external oracles can
also be wrong; use diverse evidence and exact provenance. Mutation tests ask
whether evidence notices the specific false fact, not merely whether coverage
ran.

**Failure modes:** Evidence disagreement stops the affected tranche as an
investigation. It never triggers expectation regeneration or a silent fallback.

**Independent evidence required:** The full matrix in this document,
conformance, independent models, fuzzing, malformed artifacts, runtime/ABI
differentials, LLVM review, resource tests, deterministic rebuilds, and
preregistered dogfood profiles.

**Resource and determinism bounds:** Every generator/oracle has input, work,
memory, output, and time bounds; randomness uses recorded seeds and never
defines normative truth; exhaustive claims state the exact finite domain.

**Dependencies on unresolved specification questions:** Blocked rules receive
observation cases but no invented expected verdict. Protected changes require
their own logged approval.

**Migration or foundation-audit consequences:** Keep catalog, discrepancy,
overlay, lexical-model, and conformance tooling outside production semantic
dispatch. Add new evidence alongside each implementation tranche.

**Approval status:** The evidence architecture is adopted; protected-surface
changes remain separately gated.

### Decision 17: Rust responsibility and dependency boundaries

**Decision:** Organize Rust by invariant-bearing responsibility with one-way
dependencies and minimal public APIs. Exact crate count is secondary; the
following logical boundaries and forbidden edges are mandatory.

```text
whitefoot-language-data -> whitefoot-contract
whitefoot-target        -> whitefoot-contract
whitefoot-source-audit  -> whitefoot-contract

whitefoot-syntax-data -> whitefoot-contract, whitefoot-language-data
whitefoot-lexer       -> whitefoot-contract
whitefoot-syntax      -> whitefoot-contract, whitefoot-language-data,
                         whitefoot-syntax-data, whitefoot-lexer

whitefoot-artifact-schema -> whitefoot-contract, whitefoot-language-data,
                             whitefoot-syntax-data, whitefoot-target

whitefoot-semantics -> whitefoot-contract, whitefoot-language-data,
                       whitefoot-syntax-data, whitefoot-syntax,
                       whitefoot-source-audit, whitefoot-artifact-schema,
                       whitefoot-target
  owns: checker, target qualification, projection, same-kernel replay,
        ArtifactDecision, private AcceptedCompilation factory

whitefoot-backend-invocation -> whitefoot-contract,
                                whitefoot-language-data, whitefoot-target
  owns: BackendInvocationProfile codec/validator, PublicationHostProfile,
        sealed self/tool/runtime/host observations, non-operational
        PublicationHostEvidence, pinned executable/object capabilities,
        BackendProfileAuditView,
        private ValidatedBackendInvocation factory

whitefoot-fact-contract -> whitefoot-contract, whitefoot-language-data,
                           whitefoot-target, whitefoot-artifact-schema,
                           whitefoot-backend-invocation,
                           whitefoot-semantics (narrow capability-borrow types)
whitefoot-fact-verifier -> whitefoot-contract, whitefoot-language-data,
                           whitefoot-target, whitefoot-fact-contract,
                           whitefoot-backend-invocation,
                           whitefoot-artifact-schema,
                           whitefoot-semantics (narrow read-only fact context)

whitefoot-compilation -> whitefoot-contract, whitefoot-language-data,
                         whitefoot-syntax-data, whitefoot-target,
                         whitefoot-semantics, whitefoot-artifact-schema,
                         whitefoot-backend-invocation,
                         whitefoot-fact-contract, whitefoot-fact-verifier
  owns: composite projection, final-envelope validation,
        CompilationArtifactDecision,
        private FinalizedCompilation factory

whitefoot-codegen-plan -> whitefoot-contract, whitefoot-language-data,
                          whitefoot-syntax-data, whitefoot-target,
                          whitefoot-artifact-schema,
                          whitefoot-backend-invocation,
                          whitefoot-fact-contract,
                          whitefoot-compilation
whitefoot-llvm-emit    -> whitefoot-contract, whitefoot-target,
                          whitefoot-backend-invocation,
                          whitefoot-codegen-plan
  owns: LLVM emitter, LlvmEmissionOutput and its private factory
whitefoot-llvm-audit   -> whitefoot-contract, whitefoot-target,
                          whitefoot-backend-invocation,
                          whitefoot-codegen-plan, whitefoot-llvm-emit
  owns: GuardAuditedLlvmModule, exact text validator and private factory
whitefoot-llvm-run     -> whitefoot-contract, whitefoot-target,
                          whitefoot-backend-invocation,
                          whitefoot-codegen-plan, whitefoot-llvm-audit
  owns: supervised pinned-LLVM launch/wait/import, LlvmRunOutput and its
        private factory
whitefoot-object-audit -> whitefoot-contract, whitefoot-target,
                          whitefoot-backend-invocation,
                          whitefoot-codegen-plan, whitefoot-llvm-audit,
                          whitefoot-llvm-run
  owns: GuardPresenceAuditedObject, exact object validator and private factory
whitefoot-link         -> whitefoot-contract, whitefoot-target,
                          whitefoot-backend-invocation,
                          whitefoot-codegen-plan, whitefoot-object-audit
  owns: supervised pinned-linker launch/wait/import, LinkRunOutput and its
        private factory
whitefoot-output-audit -> whitefoot-contract, whitefoot-target,
                          whitefoot-artifact-schema,
                          whitefoot-backend-invocation,
                          whitefoot-compilation, whitefoot-codegen-plan,
                          whitefoot-object-audit, whitefoot-link
  owns: LinkBoundOutputSet, exact closed-link-output validator and private
        factory
whitefoot-publication  -> whitefoot-contract, whitefoot-target,
                          whitefoot-artifact-schema,
                          whitefoot-backend-invocation,
                          whitefoot-compilation, whitefoot-codegen-plan,
                          whitefoot-output-audit
  owns: sole public finalize_and_commit(LinkBoundOutputSet,
        &FinalizedCompilation), final cross-output validator/reprojection,
        crate-private PublicationReadyOutputSet, all operational filesystem
        handles/primitives, and CommittedOutputSet

whitefoot-driver -> every stage it composes; no production stage depends on
                    whitefoot-driver.

Evidence, catalog, and conformance tools remain outside every production
dependency.
```

The capability flow is representable without a dependency cycle. In schematic
Rust, with fields and factories private to the named crate, the only candidate
interfaces are:

```text
whitefoot_llvm_emit::emit(
    plan: &'plan CodegenPlan
) -> Result<LlvmEmissionOutput<'plan>, BackendFailure>

whitefoot_llvm_audit::validate(
    emission: LlvmEmissionOutput<'plan>
) -> Result<GuardAuditedLlvmModule<'plan>, LlvmTextAuditFailure>

whitefoot_llvm_run::run(
    module: GuardAuditedLlvmModule<'plan>
) -> Result<LlvmRunOutput<'plan>, BackendFailure>

whitefoot_object_audit::validate(
    run: LlvmRunOutput<'plan>
) -> Result<GuardPresenceAuditedObject<'plan>, ObjectAuditFailure>

whitefoot_link::run(
    object: GuardPresenceAuditedObject<'plan>
) -> Result<LinkRunOutput<'plan>, BackendFailure>

whitefoot_output_audit::validate_link(
    run: LinkRunOutput<'plan>
) -> Result<LinkBoundOutputSet<'plan>, OutputAuditFailure>

whitefoot_publication::finalize_and_commit(
    linked: LinkBoundOutputSet<'plan>, compilation: &'plan FinalizedCompilation
) -> Result<CommittedOutputSet, PublicationFailure>
```

Each stage result owns its exact predecessor capability plus any new
emitted/imported bytes or audit evidence; the pinned executable/input borrows are
reached only through that predecessor's validated backend binding. `emit` is
the only operation that can construct complete LLVM text with plan provenance.
The `run` operations include launch, complete-tree wait, and bounded no-follow
import before their private result factories fire. The validators therefore
cannot accept arbitrary candidate bytes and attach an expected digest afterward.
`finalize_and_commit` consumes the linked-output capability, validates the
matching finalized compilation, keeps its ready set private, and rejects a
filesystem root or host observation that differs from the bound profile. All
operational filesystem values are created and destroyed inside that call; none
appears in a public signature or crosses to the driver.

In the diagram, `A -> B` means A may depend on B. Artifact schema depends on
judgment-free syntax data, never lexer/parser implementation. The semantics
crate may expose public artifact audit returning `ArtifactDecision`; its
accepted path may lend `ArtifactAuditContext` for nested read-only audit, while
its private originating-invocation path alone returns `AcceptedCompilation`.
`whitefoot-backend-invocation` owns the profile codec and the only
`ValidatedBackendInvocation` constructor. Its originating API receives candidate
profile bytes, an opaque validated target-profile borrow, OS-backed observation
providers, and approved tool-directory handles—not caller-authored observation
structs or publication handles. Internally
it requires OS-backed launch-time evidence for the actually mapped current
compiler image and loader/library closure; a pathname reopen is never evidence.
It copies every later tool launch closure and runtime/startup/link object through
stable no-follow source handles into exclusive driver-owned bounded byte blobs,
hashes the copied bytes, closes all writable destinations, and seals immutable
`PinnedLaunchImageSet`/`PinnedLinkInputSet` capabilities. Later runners
materialize only those owned bytes in private sandbox inputs. It also validates
the embedded compiler/build, plan-schema, lowering-dossier, validator,
output-set, host, and sandbox manifests; probes the required host operations;
and retains the pinned byte capabilities plus non-operational
`PublicationHostEvidence` that later stages may borrow.
Only this private sealed
`InvocationObservationSet` can satisfy the constructor. A successful decode
without live observations returns at most `BackendProfileAuditView`; neither the
driver nor target/compilation crates can forge or upgrade it.
The fact verifier's compile-authority API sees only a lifetime-scoped
`CompilationFactContext` pairing `AcceptedCompilation`'s narrow borrowed fact
view with the exact `ValidatedBackendInvocation`; its separate audit API sees
only `FactEnvelopeAuditContext` and returns `FactAuditOutcome`. Neither can be
called by semantic acceptance, and resource/invariant outcomes remain distinct
from fact rejection.
`whitefoot-fact-contract` owns both context layouts and their only validating
constructors. Those constructors require the opaque semantic audit/acceptance
borrow plus the corresponding backend-invocation-owned validation/audit borrow and
compare exact base/target/backend identities; there is no field constructor or
decoded-ID-only route. This later dependency on semantics does not point back:
`whitefoot-semantics` has no dependency on fact contract or verifier.
`whitefoot-compilation` is the only component that can
combine `AcceptedCompilation` with an empty or verified overlay, but it returns
`FinalizedCompilation` only after reconstructing and validating the exact
composite bytes as Decision 11 requires. A dependency-policy test resolves the
actual Cargo graph and rejects every edge not listed here;
dev-dependencies for evidence are reviewed separately and cannot be linked into
production artifacts. Every crate directly names the owner of each type or
table it imports; a convenience re-export cannot hide an omitted direct edge.
In particular, `whitefoot-codegen-plan` imports judgment-free operation/type,
node/path, artifact-view, and verified-consequence record types directly from
their owning data-contract crates. Those dependencies grant no candidate-byte
authority: its only value-level input remains read-only views borrowed through
`FinalizedCompilation`, and only `whitefoot-compilation` can construct that
capability.

The existing locked/offline dependency policy remains the default-deny
boundary: exact lock/checksum/source and feature sets are allowlisted;
unreviewed external packages, build scripts, procedural macros, git/path escape,
default features, and compile-time environment channels are rejected. Any
future dependency with transitive `unsafe` code, executable build behavior, or
native code is an explicitly named TCB addition requiring owner-reviewed
source/version/features, determinism/resource evidence, and policy allowlist;
it is never admitted transitively by convenience. The selected LLVM boundary
is textual IR plus pinned subprocesses under `BackendInvocationProfile`, not an
LLVM FFI crate.

Production source/API policy separately default-denies ambient filesystem
mutation. `whitefoot-publication` is the only component allowed to import the
publication-root safe wrapper or invoke create/write/metadata/sync/rename on
that root. The driver, semantic/backend stages, and their transitive
dependencies may not call `std::fs` mutation/open-for-write APIs, raw mutable
descriptor/platform filesystem APIs, FFI, or a generic writable-path wrapper.
Separately allowlisted source/tool input ingress is read-only and no-follow.
The two supervised runner components are the only other filesystem-mutation
exceptions, and only through an audited
root-relative `ToolWorkDir` interface created for their private sandbox; that
interface neither accepts an absolute/arbitrary path nor can address or reveal
the publication root. A source/dependency/API scan rejects new call paths,
re-exports, wrapper aliases, and feature-selected bypasses. Thus safe Rust's
ambient standard-library availability is not mistaken for a capability
guarantee: the build policy makes the restriction enforceable and the named
OS-boundary components remain explicit TCB.

**Problem being solved:** A nominal crate split can conceal shared semantic
algorithms, dependency cycles, broad utility drawers, and accidental public
constructors.

**Specification and project constraints:** Safe Rust only; no archived-code
dependency; lowering accepts no raw tree, draft, or decoded artifact; optional
fact verification cannot affect acceptance; catalog metadata never drives
semantics.

**Selected design:** Enforce the logical dependency graph and capability rules
above, creating physical crates only when their invariant and API are real.

**Input contract:** Each crate receives only its predecessor's narrow immutable
views and explicit profile. Public constructors validate the complete claim or
return a closed outcome.

**Output contract and established invariants:** Each opaque authority type has
one public claim-establishing operation at its owning boundary, but no public
unchecked or field constructor. The semantic component alone can return
`AcceptedCompilation` from the private originating-invocation replay path; the
fact verifier's compile-authority API alone can return
`VerifiedOptimizationOverlay` inside `FactVerificationOutcome::Verified`; its
audit API returns only `FactAuditOutcome`;
the compilation component alone can return `FinalizedCompilation`.
The backend-invocation component alone can return
`ValidatedBackendInvocation`; its audit API returns only
`BackendProfileAuditView`.
The LLVM emitter alone can borrow `CodegenPlan` and construct
`LlvmEmissionOutput`; the LLVM-audit component consumes only that value and
alone constructs `GuardAuditedLlvmModule`. `whitefoot-llvm-run` alone can
consume the audited module in a supervised invocation and construct
`LlvmRunOutput`; `whitefoot-object-audit` alone can consume that run result and
construct `GuardPresenceAuditedObject`. `whitefoot-link` alone can consume the
audited object with its already-bound pinned inputs and construct
`LinkRunOutput`. None of those operations accepts alternate raw-byte/path
inputs. The emitter exposes no raw-text return or separable constructor; neither
runner exposes a constructor separable from launch/wait/import.
The output-audit component alone accepts `LinkRunOutput` and constructs
`LinkBoundOutputSet`. The downstream publication component alone consumes that
capability together with the exactly matching `FinalizedCompilation`; it owns
final cross-output validation, crate-private `PublicationReadyOutputSet`, and
commit inside one public operation. The ready set has no public accessor or
return path, and every filesystem handle and primitive stays inside the same
private implementation. The terminal path cannot accept a receipt, digest,
byte slice, staged path, or caller-held filesystem capability instead.
`ArtifactAuditContext` and `FactEnvelopeAuditContext` are not accepted by any
authority factory or by CodegenPlan. There is no semantic trait implementable
by external types,
unchecked generic map of semantic records, or production feature/configuration
switch selecting alternate semantics.

**Explicit non-responsibilities:** `contract` owns transport and nominal IDs,
not all data structures. `language-data` contains reviewed declarative tables,
not algorithms. `lexer` and `syntax` do not issue semantic verdicts.
`artifact-schema` does not verify. `driver` does not interpret semantics.

**Why this stage owns the work / why adjacent stages do not:** Dependency
direction mechanically prevents later authority from leaking backward. A
private capability constructor makes accidental lowering of candidate input a
Rust type error.

**Alternatives considered and rejected:** One compiler crate permits accidental
cross-stage calls. One crate per noun creates forwarding layers. A broad
"semantic utilities" crate hides ownership of judgments. A common lowerable
trait could be implemented by unaccepted data.

**Trusted assumptions and threat model:** Shared judgment-free codecs and exact
declarative inventories are legitimate. The semantic kernel and replay walker
are deliberately shared trusted authority; tests may not mislabel that sharing
as independent evidence. Fact verifiers share no fact-producer proof algorithm.

**Failure modes:** Dependency-policy CI rejects forbidden edges, features,
macros, source splicing, environment channels, public authority constructors,
and filesystem access outside the named OS boundaries. Runtime failures remain
stage-qualified.

**Independent evidence required:** Cargo graph audit, public-API snapshots,
compile-fail attempts to forge capabilities or lower candidates, including raw
guard-shaped LLVM, object, or link bytes passed around the emitter/supervised
runner results and direct filesystem publication outside
`whitefoot-publication`; source-policy scan, feature/configuration permutations,
an import-owner/direct-edge audit, and proof that semantics has no dependency on
fact-verifier or lowering crates.

**Resource and determinism bounds:** APIs accept explicit profiles; collection
ownership and fallible allocation are visible in the responsible crate; no
ambient global configuration or thread schedule controls semantics.

**Dependencies on unresolved specification questions:** Final syntax/artifact
types, target contract, replay records, and fact families.

**Migration or foundation-audit consequences:** Do not create all pictured
crates up front. Split only when the corresponding invariant and API are real.
Files are divided when one review can no longer hold their cohesive invariant,
not at an arbitrary line count; broad `util`, forwarding-only modules, and
duplicate generic walkers are prohibited.

**Approval status:** Adopted, including the D22 redecision to the one-kernel
route. Exact crate boundaries are created only with real invariant-bearing
implementations.

### Decision 18: current Rust foundation

**Decision:** Use the audit to recommend preservation of
architecture-independent foundations, narrowing of overstated names, repair of
fallible allocation before stronger boundedness claims, and deferral of all
disputed semantic schemas. It may recommend deletion, but never merely from
distrust; it may recommend preservation, but never merely from sunk cost.

**Problem being solved:** Existing tested code can still freeze an unsupported
authority boundary; wholesale deletion would also discard useful exact source,
lexer, and reproducibility work.

**Specification and project constraints:** Existing user work and dirty-tree
changes must be preserved; archived code is inert; production code is safe
Rust, bounded and deterministic; protected/spec authority is unchanged.

**Selected design:** First produce a read-only audit receipt bound to an exact
manifest of the current workspace bytes, including the dirty checkpoint. The
keep/narrow/refactor/defer table below preregisters recommendations for that
audit; it does not authorize their application. No current semantic component
is preserved or removed by sunk cost alone.

**Input contract:** A content-addressed manifest of every current safe-Rust
workspace input, including tracked modifications and untracked files, its tests
and policies, a path/API/dependency/allocation inventory, and this architecture
after the owner approves the one-kernel redecision.

**Output contract and established invariants:** The table below is the
architecture-level migration disposition, not a completed code audit. Before
implementation resumes, a separately reviewed receipt must enumerate every
current workspace crate/module, public authority-bearing API, production
dependency and feature, and reachable allocation/publication path; it must map
each item to keep, narrow, refactor, delete, or defer. No retained component may
claim more authority than its checks establish.

| Component | Classification | Proposed consequence requiring separate approval |
|---|---|---|
| `compiler/Cargo.toml`, toolchain lock, Cargo/source policy, reproducibility gates | keep | Recommend retaining them as common infrastructure and, if separately approved, extending their dependency/API/allocation audits. |
| `whitefoot-contract/src/digest.rs` nominal spec/catalog hashes | keep | Recommend preserving nominal separation; a digest match remains identity only. |
| `whitefoot-contract/src/source.rs` logical paths, spans, and ordered `SourceBundle` model | keep then refactor | Block the model disposition pending separate A-10 ratification. If A-10 and migration are separately approved, recommend replacing allocating `Into<Box<str>>`, `Vec`, `BTreeMap`, and `Arc` construction on `LogicalPath::parse`, `SourceInput::new`, and `SourceBundle::with_limits` paths with explicit fallible builders or pre-reserved audited storage before whole-compiler boundedness is claimed. |
| `whitefoot-contract/src/binding.rs` source-binding model and codec | keep then refactor | Recommend preserving the exact wire contract and, if migration is separately approved, making `BoundSource::new`, `SourceBinding::{new,from_bundle,encode_canonical,decode_canonical}` and their `Arc`/`Vec` conversions audibly fallible under exact limits. |
| `whitefoot-frontend` scanner, token tape, outcome model, and `lex_v0_8` | keep then rename | If separately approved, recommend renaming the crate to `whitefoot-lexer` and retaining only its shape authority. Recommend that a later `whitefoot-syntax` own classifier/parser/finalizer only after grammar approval, without growing the current broad crate by accumulation. |
| `whitefoot-lexical-observer` and Python model | keep as evidence only | Recommend retaining them only as bounded evidence tools, never as a production dependency or capability receipt. |
| `whitefoot-verifier` source-binding equality audit | keep then rename/narrow | If separately approved, recommend renaming the crate to `whitefoot-source-audit`, correcting its package/docs claim from checked-artifact verifier to exact invocation/source binding, and keeping `verify_source_binding` scoped to that complete claim. Recommend that artifact replay live only in the later semantic component; this crate must not grow into a semantic verifier. |
| static facet catalog and decomposition | keep outside production dispatch | Recommend retaining them as obligation indexes that never select handlers, acceptance, or lowering. |
| capability overlay/discrepancy tools | keep outside compiler workspace | Recommend retaining the empty/non-authorizing overlay and fail-closed discrepancies outside compiler dispatch until evidence replay exists. |
| parser/tree/node-path schema | defer | Recommend no schema until terminal, node-kind, A-10, and pre-tree diagnostic decisions are separately approved. |
| semantic checker/CFG/artifact/proof schema | defer | Recommend no schema until applicable semantic blockers and the one-kernel/artifact contract are separately approved. |
| target/layout/ABI and lowering | defer | Recommend deferral until target-profile approval and the accepted-unit boundary. |
| archived democ/wfc | keep inert in archive | Recommend keeping them as inert evidence only, with no source import, build, semantic, or release dependency. |
| deletion set | none currently proposed | The audit may recommend deletion only if it finds an unrepairable false authority boundary and records the exact path and replacement; the finding never authorizes deletion. |

**Explicit non-responsibilities:** This audit does not approve the current
uncommitted checkpoint, commit implementation, revise roadmap authority, or
resolve spec/test discrepancies.

**Why this stage owns the work / why adjacent stages do not:** Architecture
determines whether a component's responsibility is real. Tests establish its
current behavior, not its future authority.

**Alternatives considered and rejected:** Keep-all preserves misleading names
and infallible boundaries. Delete-all discards correct architecture-independent
work. Creating empty placeholder crates gives false progress and premature APIs.

**Trusted assumptions and threat model:** Existing code remains untrusted at
new boundaries until re-audited. Dirty-worktree changes belong to the current
checkpoint and are not silently rewritten by this design task.

**Failure modes:** Any component that cannot be narrowed without changing a
protected/spec surface stops for owner review. A red policy or spec guard is a
stop, not a baseline-regeneration instruction.

**Independent evidence required:** Current focused tests, a machine-readable
Cargo/API/path inventory, compile-fail capability probes, dependency and
hidden-allocation audit, fallible-allocation boundary tests, reproducible builds, and
the stage evidence defined above after each component acquires a real
responsibility.

**Resource and determinism bounds:** Existing source/lexer limits remain local.
New code cannot inherit `REPRESENTABLE` as production policy or use infallible
growth hidden behind retained APIs.

**Dependencies on unresolved specification questions:** A-10 multi-file
composition, all parser/semantic blockers, target profile, artifact schema, and
owner approval of the D22 redecision.

**Migration or foundation-audit consequences:** The table is a preregistered
disposition. The path-level audit receipt is an entrance gate, and actual
renames/refactors happen in separately reviewed, gate-green changes after owner
approval. The audit must preserve user checkpoint work and report any overlap
before editing it.

**Owner rulings (2026-07-21):** D23 authorized the exact-workspace read-only
audit. After reviewing its disposition, D25 authorized the handoff migration:
truthful lexer/source-audit names and claims plus explicit fallible owned source
and source-binding boundaries. Deletion, a new production crate, wire change,
or an entrance-gated schema remains unapproved.
