# THE PLAN

Status: CANONICAL ROADMAP, updated 2026-07-19.

This file contains Whitefoot's one current execution order. Other files may
define law, specify behavior, explain a design, or preserve evidence. They do
not set current priority or authorize work.

## Authority and scope

- `CONSTITUTION.md` defines project law.
- `optimizer-language-research/notes/user-directives.md` records owner rulings.
- `spec/kernel-spec-v0.8.md` defines the accepted language.
- `PATTERNS.md` defines the closed set of forms writers use.
- This file orders implementation and records its authorization boundary.
- `optimizer-language-research/implementation/decision-gates.md` preserves the
  append-only evidence log. `mcts_mem/` preserves design reasons and rejected
  alternatives.

If these sources disagree, stop at the conflict and reconcile the governing
source and this roadmap in one change. A research result, handover, follow-up
list, design dossier, archived plan, or MCTS alternative cannot authorize work.

On 2026-07-17 the owner directed the project to consolidate its plan files and
complete phases 1 through 7 below. That direction authorizes each phase after
its predecessor passes its exit gate. Phase 1 completed on 2026-07-17; phase 2
is active. The authorized scope ends with the first production `seq` component.
It excludes every later
container, checked-source library, public I/O or FFI surface, compiler
migration, and concurrency feature.

A failed correctness gate returns work to the first phase that owns the defect.
A phase cannot borrow a future artifact to satisfy its gate.

Every reproducible defect or discrepancy gets the smallest practical automated
regression before its fix closes. Language and specification discrepancies use
conformance cases; implementation defects use focused unit or integration
tests; hostile boundary failures also pin failure atomicity and guard behavior.
If automation is not practical, the decision log records the reproducer and why
the gate cannot execute it yet.

## Measured baseline

Baseline date: 2026-07-17, before plan consolidation.

- The wfc unit contains 32 Whitefoot files, 24,695 lines, and 477 functions.
  Concatenation in `sources.txt` order produces 1,095,342 source bytes, 217,254
  tokens, and 109,235 unique-head AST nodes.
- wfc lexes, parses, validates, indexes, resolves declarations and types, and
  builds function scopes for that complete unit. Two parses produce identical
  token and AST tapes.
- Body semantics classify 15 functions as clean, 462 as legal but unsupported,
  and zero as semantic rejects. The first source-order frontier is
  `lexer_scan_op_suffix`.
- LLVM support emits the same 15 clean functions as one deterministic module.
  Clang accepts it and the native tests pass.
- `wfc_frontend_run` is the current external seam. The complete `wfc_compile`
  path, a stage-1 compiler, and a self-hosting fixpoint do not exist yet.
- wfc uses fixed-capacity structure-of-arrays tapes backed by primitive
  buffers. Source size bounds each tape. Bootstrap needs no growable
  collection, pool, generic container, or thread.
- Stage 0 is `prototype/democ`. It compiles wfc with optimizer facts disabled.
  The current v0.6 implementation and conformance suite provide the independent
  behavior oracle while wfc catches up.
- The systems-capability research package has finished. Its 15-rule loan/freeze
  design passed a 97-program machine corpus and all nine mutation tests. Its
  `seq` operation table and dry run provide landing evidence. None of that work
  has changed the production specification or compiler.
- `make check` and `make -C compiler check` passed at the baseline commit.

## Phase 1: establish one roadmap

**Entry:** the 2026-07-17 owner direction above.

Work:

1. Move the live compiler sequence from `compiler/PLAN.md` into this file.
   Preserve architecture in `compiler/README.md` and `mcts_mem/`; remove the
   duplicate plan.
2. Move `HANDOVER.md` and the completed validation-harness plan into `archive/`.
   Fold unresolved production prerequisites from the capability follow-up list
   into the relevant gates below, then remove that list.
3. Replace dated current-focus material in `AGENTS.md` and `CLAUDE.md` with a
   pointer here. Keep those two files byte-identical.
4. Add a root check that rejects another plan-named file, a second canonical
   roadmap marker, a retired active path, or a duplicate current-focus section.
   Historical plans may remain under `archive/` and rejected alternatives may
   remain under `mcts_mem/**.alt/`.
5. Correct stale counts, decisions, and authorization statements in active
   documentation. Preserve completed protocols and archived evidence unchanged.

**Exit gate:** this file is the sole active roadmap. The structural check
rejects competing roadmap markers and retired paths; review confirms that no
other active document defines execution order. Both project gates pass. The
change has one commit and one decision-log entry.

## Phase 2: self-host the compiler subset with facts off

**Entry:** phase 1 exit.

Execute these steps in order:

1. **Complete body semantics.** Grow wfc's type, ownership, loan, and effect
   checks until it classifies every function in the exact `sources.txt` unit.
   Its rejection ABI records one rule ID, canonical node path, primary and
   related nodes, source span, and an applicable mechanical fix code.
   Add one structural capability family per slice. Compare every covered
   conformance case and the whole compiler unit with stage 0. Resolve each
   verdict difference as a wfc defect or a stage-0 defect backed by a new
   conformance case.
2. **Complete whole-unit lowering.** Start after body-semantics coverage reaches
   the full unit. Grow the shared fact-driven LLVM families until
   `wfc_compile(sources)` emits every function. Clang must accept the complete
   module, and native tests must exercise it.
3. **Run compiler-subset conformance through stage 1.** Add a wfc adapter to the
   conformance runner. Require zero verdict differences for the language subset
   wfc's source uses.
4. **Reach the facts-off fixpoint.** Build wfc1 with stage 0; compile the unit to
   IR2 and build wfc2; compile the same unit to IR3. Require byte-identical
   `IR2 == IR3`. Freeze stage 0 at this point. Keep it as a differential oracle,
   but add no language feature to it.

New helper functions change the unit total, so each slice regenerates and pins
the exact count. Facts remain disabled throughout this phase.

**No-reborrow decision — RESOLVED (2026-07-18): bounded statement-scoped reborrow,
landed in v0.7.** Growing body semantics surfaced that wfc's own source reborrows
at ~1,062 verified sites — it borrows a sub-place of an exclusive `&uniq` holder
and passes it onward as a call argument (e.g. `frontend_unit_reset` forms
`&uniq 'frontend_reset_tokens deref(analyzed).tokens`), which v0.6 forbade. The
owner-directed investigation (`optimizer-language-research/implementation/
reborrow-investigation/`) weighed keeping the rule and rewriting the code against
relaxing it, on written pros and cons, a Featherweight-Rust reconciliation, a
1,000,000-program model-check (zero aliasing), and a hostile no-alias fact-channel
review (PASS-WITH-CONDITIONS; performance preserved). The owner ratified the narrow
relaxation, and kernel-spec **v0.7** admits the bounded, non-escaping,
statement-scoped child reborrow (OWN-5/6/9/12 + new STOR-5) while deferring
result-transfer and the harder forms. What remains in Phase 2 is production
implementation of an already-landed spec rule, not a new spec decision: implement
the fragment in the production checker with the recorded conditions, and grow wfc
body semantics and lowering so every reborrowing function classifies clean and
lowers — no function may remain unsupported at the Phase 2 exit.

**Exit gate:** the exact compiler unit has complete body semantics and lowering;
stage-1 wfc matches stage 0 on compiler-subset conformance; the facts-off IR
fixpoint is byte-identical; both project gates pass.

## Phase 3: implement all of v0.8 with facts off

**Entry:** phase 2 exit and a frozen stage 0.

Implement every accepted v0.8 construct that wfc's own source did not require.
The work includes `requires`: AST representation, parsing, entry-check
normalization, body-derived obligation accounting, deterministic diagnostics,
lowering, and report/no-report byte identity. Complete effect checking before
any optimizer fact can affect output.

Run the full v0.8 conformance suite through wfc. Use frozen stage 0 as the
behavior oracle only for its frozen subset. Use specification-derived expected
artifacts or purpose-built reference checkers for constructs stage 0 lacks.
Resolve all fourteen pending source cases so the runner reports zero skips.
Implement executable DIAG-2 elaborated-artifact and DIAG-3 report-schema gates;
annotations do not close those rules. Preserve a conformance case for each
discrepancy. Keep facts disabled and rerun the facts-off self-hosting fixpoint
after the final language slice.

**Exit gate:** wfc accepts and rejects every source-emittable v0.8 case with
zero skips, supports `requires` end to end, emits the DIAG-2 artifact and DIAG-3
reports, completes effect checking, and retains the facts-off byte-identical
fixpoint. Both project gates pass.

## Phase 4: enable fact channels and finish the compiler baseline

**Entry:** phase 3 exit.

For each fact family, freeze its proposition, provenance, producer, consumer,
invalidators, and per-site accounting before enabling it. Add observational
reports first. Require report/no-report output identity. Then enable one family,
run its facts-off control, inspect generated IR and assembly, and obtain hostile
review before shipping that family.

Require wfc to match the reference proof accounting on the bounds corpus and
all codegen-parity pins. Re-run full conformance in both fact modes and
establish a byte-identical facts-on self-hosting fixpoint.

Finish the compiler boundary in this phase: remove intentional compiler-owned
allocation leaks, retain deterministic diagnostics, harden `wfc_compile`, and
provide the trusted launcher shim. The launcher shim does not define public
file-I/O language semantics.

**Exit gate:** wfc passes full v0.8 conformance, codegen parity, per-site proof
accounting, resource cleanup, and byte-identical self-hosting with facts on.
Every fact family has hostile-review evidence. Both project gates pass. wfc is
the production compiler baseline; frozen stage 0 remains an oracle.

## Phase 5: freeze the production landing contract

**Entry:** phase 4 exit. The owner authorization at the top of this file opens
this phase without another scope decision.

Convert the sequential projection of the ratified loan/freeze research and all
24 `seq` rows into one implementation packet. Freeze exact grammar, judgments,
diagnostics, lowering, facts and invalidators, operation rows, trap behavior,
drop order, failure behavior, target mapping, and artifact hashes. Resolve every
open flag that loan/freeze or `seq` consumes.
Honor the selected explicit `copy struct` design; infer no structural copy. Add
that feature only if the frozen `seq` rows require composite Copy in phase 7.
Resolve ALLOC-ERR through a recorded owner ruling before freezing any allocating
row. The recommended rule makes environmental allocation failure a `Result`,
preserves every owned input on the error arm, and keeps capacity overflow as a
programmer-error trap.

Freeze a corpus adapter before production checking. Classify each research AST
case as source-translated, verdict-only abstract semantics, or deferred
concurrency. Define sealed signature stubs with no runtime authority for the
abstract cases, map research rule IDs to production IDs, and exclude trusted
`body=None` entries from source-conformance credit.

Define the five acceptance legs against one frozen artifact and a named Linux
x86-64 host, compiler, allocator, reference-library version, and workload.
Preregister model bounds, differential and fuzz seeds, run budgets, sanitizers,
fault schedules, review severities, sample counts, statistical bands, teardown,
and resource ceilings. Provision that runner before phase 5 exits; Apple M4 dry
runs provide design evidence but do not satisfy the deploy gate.

Keep `prototype/democ` frozen. Use the 15-rule research checker as the
independent loan/freeze oracle; wfc owns production semantics. Record this
division in the design tree and specification derivation ledger. Hostile review
must attack the packet before implementation begins.

**Exit gate:** the packet contains no open semantic flag, no dependency on a
later component, an owner-ratified ALLOC-ERR policy, a complete corpus adapter,
all 24 `seq` rows, executable pass and fail bands, a provisioned deploy runner,
and a passing hostile landing review. Both project gates pass.

## Phase 6: land sequential loan/freeze without a container

**Entry:** phase 5 exit.

Land rules R1 through R13 and the sequential part of R15, plus confined
borrow-carrying values, across the production specification, wfc parser and AST,
checker, diagnostics, lowering, conformance suite, derivation ledger, and
teaching material. Defer R14 and R15's concurrent-invocation clause with the
concurrency layer. Keep this change free of container implementation.

Run the 88 non-parallel research cases through the frozen adapter and all nine
mutation tests against the independent checker and wfc. Route source-translated
cases through the parser; route abstract cases through the verdict-only semantic
entry. Add production conformance cases for rule boundaries, invalidations,
early exits, effect interactions, and malformed internal state. Pin
unchanged-source IR and diagnostics. Obtain hostile review of the facts the new
judgment makes available to lowering. Preserve the nine parallel cases for the
later R14 landing.

**Exit gate:** wfc and the independent checker agree on all 88 sequential cases;
all applicable mutants fail as intended; full conformance and codegen parity
pass; no safety check has weakened; both project gates pass.

## Phase 7: land `seq` as the first sealed component

**Entry:** phase 6 exit and a phase-5 packet with no unresolved `seq` flag.

Implement `seq<T, N>` with inline capacity `N` as one vertical production
slice. Cover affine elements, uninitialized spare capacity, spill and growth,
take and replace, insert and remove, drain, slices, fact production and
invalidation, capacity overflow, allocation failure, drop order, and teardown.
Land its specification rows, wfc recognition and lowering, sealed
implementation, conformance cases, diagnostics, code-shape pins, and teaching
entry in the same phase.

Run all five acceptance legs across the exact 24-row surface on the same frozen
bytes:

1. differential operation testing against the reference implementation;
2. bounded state and ownership modeling;
3. sanitizer-backed fuzz and fault-injection soak;
4. hostile safety, ABI, drop, and proof-fact review;
5. Linux x86-64 performance and assembly checks, including the preregistered
   wfc-shaped tiny-vector band against SmallVec. A separate layout fixture must
   embed inline `seq` in an ordinary struct and exercise insert, remove, and
   drain. Phase 5 must define the future pool-slot fact-flow check without
   requiring a production pool implementation in this phase.

Do not migrate wfc from its structure-of-arrays tapes during this phase. Those
tapes provide the control for any later migration experiment.

**Exit gate:** all five bounded legs pass on the frozen artifact with zero
observed uninitialized reads, leaks, double drops, stale loans, missed traps, or
failure-atomicity divergences within the preregistered runs. The performance and
code-shape bands pass; both project gates pass. Stop and report completion of
the authorized seven-phase scope.

## Execution cursor

Phase 2 is active. The canonical rejection ABI, explicit call-region retention,
and arbitrary-arity exact call substitution are complete. The current unit has
639 functions: 164 clean, 475 legal but unsupported, and zero rejected. Its
self-parse is deterministic at 1,727,663 source bytes, 348,731 tokens, and
173,085 unique-head AST nodes. The parser census is 4,804 regionful calls: 496
explicit and 4,308 staged omissions. LLVM support remains the same
byte-identical 15-function module.

Kernel v0.8 and its tag-only enum equality implementation are complete.
Stage 0 and the wfc reader recognize only exact nominal tag-only `eeq`/`ene`;
`ieq`/`ine` remain integer-only. Stage 0 lowers both enum operations directly
at the selected tag width, including i1 for `Bool` and two-variant enums. The
complete compiler-source migration has 255 `eeq` sites in 92 functions across
18 files and 22 tag-only types, with zero non-integer `ieq`/`ine` sites. The
approved 16-case additive conformance surface passes, and hostile tests cover
same-width nominal confusion, payload enums, malformed symbol/type topology,
declaration collisions, missing explicit type arguments, all bounded truth
tables, and direct raw/optimized code shape. Five source functions move to
CLEAN through the repaired equality domain: `semantic_body_kind_is`,
`llvm_scalar_node_is`, `llvm_scalar_type_is`, `llvm_scalar_mode_is`, and
`llvm_scalar_operation_is`.

The F1 acyclic read-only tranche covers general signatures and exact effect
rows, general tag-only enum matches and values, shared struct fields, typed
scalar and tag-only-enum buffer `index`, and field/buffer `len`. The first
bounded F2 slice is complete: the reader carries exact fall/return/break
may-flow through sequential blocks and exhaustive matches, admits canonical
loops with function-unique labels and innermost labeled `break`, admits
exact-type mutation of owned `let` bindings, recognizes exact traps-only rows
and callees, and discovers effects recursively through `set` values and loop
bodies. It deliberately keeps parameter mutation, outer-target breaks from a
nested loop, duplicate labels, and a no-break loop without any return witness
unsupported. `semantic_reader_u64_literal_any` is the one pre-existing
function newly classified clean; the other three clean additions are the new
flow-query helpers. A fresh corpus inventory found no additional isolated
loop/local-mutation unlock; admitting owned-parameter mutation also conflicts
with stage-0 lowering and unlocked zero compiler functions, so that experiment
was fully reverted and the bounded F2 compiler-family tranche is complete.

The first nine bounded F3 writer slices are complete. The writes-only profile admits one or more exclusive
borrows of structs in exactly one declared region, an exact writes-only row for
that region, one or more flat direct scalar/tag-only-enum field assignments from
own parameters, canonical `u8`/`u64` literals, or exact nullary `Bool`/tag-only-
enum constructors, or a prior direct top-level `u64` constant initialized by a
canonical `u64` literal, and a final `return unit`. Constructor resolution is
confined to this field-writer path: a bounded whole-unit scan proves one globally
unique direct nullary variant, a tag-only owning enum, and exact nominal field
equality. Constant resolution is likewise writer-local: it requires the exact
value-symbol binding, declaration-before-use, exact const/name/type/value topology,
source-anchored head tokens, and exact `u64` type and initializer. It admits no
mutable global state (which the language forbids), forward reference, alias,
array, non-`u64` constant, or function symbol. Shared or cross-region exclusive roots,
general `unit` readers, borrowed RHS values, payload or ambiguous constructors,
general constructor expressions, missing/spurious or wrong-region writes, nested
control, writer calls, and non-unit returns remain unsupported. The first slice moved exactly
four pre-existing functions to CLEAN:
`symbol_report`, `semantic_body_set_report`,
`semantic_type_resolve_set_report`, and `llvm_supported_fail`. Hostile review
caught and closed an initial widening that also admitted three read-only
exclusive-borrow helpers; both exclusive borrows and `unit` are now fenced to
the exact writer profile, and the body independently verifies the exclusive
target and flat RHS shape. The second slice moves exactly seven more pre-existing
functions to CLEAN: `byte_tape_reset`, `semantic_all_types_fail`,
`frontend_token_tape_reset`, `llvm_scalar_fail`, `llvm_linear_fail`,
`llvm_buffer_fail`, and `llvm_scanner_fail`. Its writer-specific implementation
and hostile tests live in focused files rather than enlarging the general reader
and unit-test modules. The third slice moves exactly six more pre-existing
functions to CLEAN: `symbol_tape_reset`, `semantic_type_tape_reset`,
`semantic_node_facts_reset`, `frontend_ast_tape_reset`,
`frontend_validation_reset`, and `frontend_report_reset`. Its hostile review also
fixed canonical `u64` validation at the exact 20-digit maximum and pinned
out-of-range rejection, stale symbol redirection, forged AST heads, malformed
const topology, forward declarations, and non-const value symbols. The focused
writer implementation remains 861 lines. The fourth slice admits the one measured
mixed-effect boundary: exactly two distinct regions, exactly one shared root for
an exact reads row, exactly one exclusive root for an exact writes row, `traps`,
and the same flat direct-field writer body, with user calls allowed only as
right-hand-side values under that mixed profile. Ordinary call analysis proves
callee signature and result type; exact call-region substitution proves shared
argument provenance; and effect reconciliation independently proves every declared
read, rejects undeclared reads or traps, and rejects write-effect callees. The source
function `parser_fail` now declares only its exhibited token-tape read and spells
both call-region arguments explicitly, moving exactly that one pre-existing function
to CLEAN. Hostile review pinned omitted and wrong call regions, extra roots and
regions, missing/spurious/wrong rows, wrong call results, hidden write-effect callees,
malformed region nodes, call topology, and duplicate effect nodes. Mixed-profile
logic lives in a focused 237-line module. The fifth slice extends the established
writes-only signature proof from exactly one exclusive root to one or more roots,
while requiring every root, including unused roots, to belong to the singleton
declared write region. The existing body proof already checks every assignment
target independently against an exclusive parameter and that region. This moves
exactly `semantic_body_mark_failure`, whose three exclusive output tapes share one
region, to CLEAN; shared roots, zero roots, and an unused cross-region root remain
Unsupported. Its focused hostile test is 571 lines. The sixth slice admits exact
flat indexed targets only under `writes(region), traps` with no read row: the
target must be `index<T>(deref(exclusive_struct).buffer_field,
own_u64_parameter)`, the root must belong to the singleton write region, the
element and right-hand-side types must match exactly, and the bounds-checked
target supplies `traps` without fabricating a read of the exclusive root.
Immutable-global subscripts, direct buffer roots, shared or cross-region roots,
mixed read/write rows, and malformed target topology remain unsupported.
Hostile review additionally source-anchors every semantics-bearing target head
and leaf, including same-spelling token redirects. Exactly
`semantic_body_set_value_fact`, `semantic_buffer_write_result_fact`, and
`semantic_buffer_write_place_fact` move to CLEAN. Target logic lives in a
focused module and a separate hostile test; extracting the
prior field-target proof reduces the writer module to 814 lines and the general
reader. The seventh slice extends only the indexed-target subscript boundary to
canonical `u64` numeric literals. The shared index analyzer receives an explicit
capability flag that is false for ordinary indexed reads and true only for writer
targets; independent target validation rechecks the literal kind, canonical value,
and exact source anchoring. Leading-zero, wrong-width, and out-of-range literals,
same-spelling token redirects, immutable-global subscripts, and literal subscripts
on ordinary indexed reads remain Unsupported. Exactly
`semantic_body_initialize_types` moves to CLEAN. The target module is 388 lines,
its hostile test is 356 lines, the writer module remains 814 lines, and the general
reader is 6,623 lines. The eighth slice admits only prior direct immutable `u64`
constants as indexed-target subscripts. It reuses the writer-local constant proof:
an exact value-symbol binding must name a source-anchored three-child `const`
declaration before the use, with exact `u64` type and a canonical direct `u64`
literal initializer. Ordinary indexed reads keep the capability disabled.
Forward, wrong-width, noncanonical, out-of-range, and function-symbol cases stay
Unsupported, as do same-named non-own parameters: parameter lookup distinguishes
an absent name from a present but invalid binding before global fallback. Exactly
`semantic_buffer_initialize_types` moves to CLEAN. Target logic is 408 lines, its
focused hostile test is 418 lines, the writer module remains 814 lines, and the
general reader is 6,634 lines. The ninth slice admits the one remaining
loop-based indexed initializer, `semantic_body_initialize_facts`. Its exact
control protocol requires an owned `u64` cursor initialized by canonical
`0_u64`; one loop whose source-anchored `ige<u64>(cursor, own_u64_parameter)`
guard has a `True` arm containing only a break to the same source-anchored label
and an empty `False` arm; one or more indexed assignments rooted in the exact
write region and subscripted only by that cursor; the exact trapping
`cursor = iadd.trap<u64>(cursor, 1_u64)` update; one or more direct field tail
writes; and a final `return unit`. The ordinary reader, flat writer, and indexed
read profiles are unchanged. Hostile review caught and closed swapped arm tags,
non-cursor subscripts, and same-spelling type/name/label token redirection before
the slice landed. Exactly `semantic_body_initialize_facts` moves to CLEAN; no
other pre-existing function moves and no prior CLEAN function is lost. Control
logic is isolated in a 674-line module with a 325-line focused hostile test;
the general reader is 6,644 lines. The post-slice inventory left
`frontend_unit_reset` as the only Unsupported function with writes but neither
reads nor allocation; it is a statement-scoped reborrow/call-composition shape,
so the bounded F3 compiler-family tranche is complete.

The first bounded F4 slice implements that exact compiler shape. It admits one
exclusive struct parameter in one caller region and a nonempty sequence of
distinct local region statements. Each local region contains exactly one
expression-statement call with exactly one named argument: an unbound unique
child borrow of one direct field of the parent. The local region spelling must
match the child, cannot shadow the caller region or another local region, and
ends with that statement; the call returns own `unit`; the parent is
structurally suspended because no sibling or other argument exists. The callee
has exactly one unique parameter, an exact writes-only row for its formal
region, and an independently validated flat writer body, so a dishonest or
unsupported callee cannot supply the effect fact. This implements the exact
ancestor effect exemption without widening the ordinary field reader. Shared
children, whole-parent or deeper suffixes, explicit call-region arguments,
bound results, multiple arguments, siblings, let-bound parents, read/write
callees, and borrow returns remain Unsupported. Hostile review pins local-region
confinement and uniqueness, parent suspension, non-escape, dishonest callees,
disjoint and overlapping deferred siblings, exact nominal field types,
semantics-bearing source heads, and cyclic topology. Exactly
`frontend_unit_reset` moves to CLEAN and no prior CLEAN function is lost. F4
logic is isolated in 590- and 389-line modules with a 366-line focused test; the
general reader is 6,692 lines.

The second bounded F4 slice admits the exact same-region `ByteTape` push
protocol and its smallest call-composition boundary. The independently proven
callee has one exclusive struct root and one owned `u8` parameter, exact
singleton `reads(root)` and `writes(root)` rows plus `traps`, and the exact
six-statement push body: read the count and byte-buffer capacity, compare them,
write either the indexed byte or a matching nullary status constructor, perform
the trapping count increment, and return unit. Field declarations, nominal
types, parameter and local bindings, effects, constructor ownership, source
heads, and body topology are all revalidated without recognizing project
names. The caller has one exclusive parent and one fresh one-statement local
region; it passes exactly a whole-parent unique reborrow and one canonical
`u8` literal to that proven callee, then returns unit. The exact call shape
makes suspension and non-escape structural. Any same-region mixed signature
outside these two bodies stops before generic writer reconciliation. Shared or
field/deeper borrows, explicit call-region arguments, reordered or additional
arguments, noncanonical literals, bound results, multiple calls or regions,
dishonest callees, and sibling children remain Unsupported. Hostile review
pinned wrong or missing rows, field and parameter types, local uniqueness,
callee-body substitutions, renamed positive controls, same-spelling source-head
redirection in both directions, and cyclic sibling topology. Exactly
`byte_tape_push` and twelve one-byte `llvm_text_emit_*` wrappers move to CLEAN;
no prior CLEAN function is lost and all twenty new helpers remain Unsupported.
The focused implementation is split across 133-, 231-, 325-, 201-, and
296-line modules with a 349-line hostile test; the general reader is 6,730
lines. A fresh post-slice inventory finds 67 simple unsupported region-call
wrappers. The next bounded boundary is the exact `byte_tape_emit_chunk` body;
it has eight guarded whole-parent calls to the proven push using owned `u8`
parameters, and it is the prerequisite for 37 simple fixed ten-argument chunk
wrappers (30 one-region, six two-region, and one four-region). General sibling
overlap and recursive numeric emission remain later F4 work.

The third bounded F4 slice admits the exact guarded eight-byte chunk callee.
Its signature has one exclusive struct root, one owned `u64` count, eight
distinct owned `u8` parameters, exact singleton `reads(root)` and
`writes(root)` rows plus `traps`, and an own-unit result. The body first proves
the count is at most canonical `8_u64`; its exact invalid arm writes a nullary
constructor belonging to an enum field of the same root and returns. Eight
ordered `ige<u64>(count, N_u64)` matches then either return early or enter one
statement-scoped region and call the independently proven push callee with a
whole-parent unique reborrow and the corresponding `u8` parameter. The eighth
false arm is empty and the body ends with `return unit`. Every operation,
threshold, arm tag and statement count, parameter and named-argument order,
nominal type, source head, and topology is revalidated without recognizing
project names. The ordinary body analyzer's new fallback accepts only an owned
`u8` parameter as the second argument of this already-bounded region-call
shape; the same-region effect gate still requires the complete exact chunk
body before returning CLEAN, so the flow support cannot lend authority to any
other mixed read/write function. Hostile review pins wrong or missing rows,
parameter modes and types, count guards, threshold order, early-return shape,
status-field and constructor types, local-region confinement, shared or deeper
borrows, explicit call regions, argument order and binding, dishonest callees,
same-spelling source-head redirection in both directions, and cyclic topology.
Exactly `byte_tape_emit_chunk` moves to CLEAN; no prior CLEAN function is lost,
and all fifteen new helpers remain Unsupported. The unit is 619 total / 122
CLEAN / 497 Unsupported / 0 rejected; the exact 15-function LLVM module is
unchanged. The parser census is 4,525 regionful calls = 496 explicit + 4,029
staged omissions; self-parse is deterministic at 1,663,946 bytes / 335,070
tokens / 166,278 nodes. A shared 149-line region-call shape module reduces the
prior region module from 201 to 64 lines; chunk logic is split across 187-,
131-, 195-, and 85-line modules with a 382-line hostile test, and the general
reader is 6,739 lines. A fresh inventory finds 41 Unsupported callers of the
now-proven chunk: 30 are exact one-region, two-statement fixed-literal wrappers;
six contain two chunk regions, one contains four, and four compose one chunk
with recursive numeric emission. The exact 30-wrapper one-region family is the
next bounded F4 boundary; multi-region and numeric composition remain later.

The fourth bounded F4 slice admits that exact one-region fixed-literal wrapper
family. Each caller has one exclusive struct root, exact singleton reads and
writes rows plus traps, own unit, one fresh one-statement local region, and a
final unit return. The call has exactly ten named arguments in formal order: a
whole-parent unique reborrow confined to the local region, one canonical `u64`
count literal, and eight canonical `u8` literals. The count is not treated as a
proof fact; the independently re-proven chunk callee retains its exact
`count <= 8` guard before any write. The caller proof also requires exact
nominal root type, argument/formal name correspondence, source anchoring, and
topology. The ordinary body analyzer gained only a fallback for this exact call
shape with the same independent callee proof; the focused caller signature and
body gate remains the only source of classifier authority. Shared, deeper, or
wrong-region parents, explicit call regions, bound calls, wrong or reordered
arguments, noncanonical or out-of-range literals, dishonest chunk or nested
push callees, multi-statement regions, and multiple regions remain Unsupported.
Hostile review additionally caught and corrected a test mutation that first
targeted the nested callee instead of the wrapper, then pinned the intended
multi-region boundary plus same-spelling head redirection and cyclic topology.
Exactly 30 pre-existing functions move to CLEAN; no prior CLEAN function is
lost and all seven new helpers remain Unsupported. The unit is 626 total / 152
CLEAN / 474 Unsupported / 0 rejected; the exact 15-function LLVM module is
unchanged. The parser census is 4,568 regionful calls = 496 explicit + 4,072
staged omissions; self-parse is deterministic at 1,675,543 bytes / 337,523
tokens / 167,503 nodes. Shared call syntax is 166 lines, the new focused module
is 216 lines, its hostile test is 280 lines, and the general reader is 6,748
lines. A fresh inventory leaves exactly 11 Unsupported chunk callers: six
two-region wrappers, one four-region wrapper, and four chunk-plus-recursive-
number compositions. The exact six-wrapper two-region family is next;
four-region and recursive composition remain later.

The fifth bounded F4 slice admits the exact six-wrapper two-region family.
The established one-parent signature and exact fixed-literal chunk call proof
are unchanged. The caller body has exactly two sequential one-statement local
regions followed by a unit return; both locals are fresh relative to the outer
region and distinct from each other. Each call independently re-proves its
whole-parent unique child, ten named actuals in formal order, canonical `u64`
and `u8` literals, nominal parent type, guarded chunk callee, and nested byte-
push callee. Because each unbound own-unit call and its local region end before
the next statement begins, the parent is suspended for one statement at a time
and resumes between them; no sibling children coexist. One-region callers keep
their prior path. Shared, deeper, explicit, bound, wrong-parent, wrong-literal,
wrong-callee, duplicate-local, outer-shadowing, mixed-call, extra-statement,
three-or-more-region, and cyclic shapes remain Unsupported. Hostile review
also converts the prior appended-second-region negative into the intended
positive boundary and adds a third-region negative, rather than weakening the
fence. Exactly six pre-existing functions move to CLEAN:
`llvm_text_emit_buffer_type`, `llvm_text_emit_overflow_pair_type`,
`llvm_text_emit_llvm_trap`, `llvm_text_emit_extractvalue`,
`llvm_text_emit_getelementptr`, and `llvm_text_emit_unreachable`. No prior CLEAN
function is lost and the one new helper remains Unsupported. The unit is 627
total / 158 CLEAN / 469 Unsupported / 0 rejected; the exact 15-function LLVM
module is unchanged. The parser census is 4,580 regionful calls = 496 explicit
+ 4,084 staged omissions; self-parse is deterministic at 1,678,161 bytes /
338,051 tokens / 167,770 nodes. The focused proof is 247 lines, its hostile
test is 381 lines, the shared caller gate is 330 lines, and the general reader
remains 6,748 lines. Fresh inventory leaves exactly five Unsupported chunk
callers: one exact four-region wrapper and four chunk-plus-recursive-number
compositions. The four-region wrapper is next; recursive composition remains
later.

The sixth bounded F4 slice admits that exact four-region wrapper,
`llvm_text_emit_uadd_overflow_i64`. Its established one-parent signature and
fixed-literal chunk-call proof are unchanged. The caller contains exactly four
sequential one-statement local regions followed by a unit return. Every local
region differs from the outer region and from all prior locals, and each call
independently re-proves its confined whole-parent unique child, ten named
actuals in formal order, canonical `u64` and `u8` literals, nominal parent
type, guarded chunk callee, and nested byte-push callee. The bounded sequence
helper accepts only two or four regions; one-region callers retain their prior
path, while three-, five-, and composite-region shapes remain Unsupported.
Each unbound own-unit call and local region ends before the next statement, so
the parent resumes between statements and no unique siblings coexist. Hostile
review pins caller modes and effects, exact literals and argument order,
shared, deeper, explicit, and bound calls, missing, mixed, or dishonest nested
callees, pairwise local freshness including nonadjacent duplicates, outer
shadowing, extra statements, closed three- and five-region shapes, source-head
redirection, and cyclic topology. It also caught and corrected two test
mutations that initially targeted a nested callee rather than the wrapper.
Exactly `llvm_text_emit_uadd_overflow_i64` moves to CLEAN; no prior CLEAN
function is lost and the refactored sequence helper remains Unsupported. The
unit is 627 total / 159 CLEAN / 468 Unsupported / 0 rejected; the exact
15-function LLVM module is unchanged. The parser census is 4,581 regionful
calls = 496 explicit + 4,085 staged omissions; self-parse is deterministic at
1,679,443 bytes / 338,276 tokens / 167,874 nodes. The focused proof is 283
lines, its new hostile suite is 224 lines, the shared caller gate is 339 lines,
and the general reader remains 6,748 lines. Fresh inventory leaves four
Unsupported chunk-plus-recursive-number callers. Their next common prerequisite
is the exact recursive decimal emitter `byte_tape_emit_u64`, selected as the
next bounded F4 boundary before admitting any composite caller.

The seventh bounded F4 slice admits that exact terminating recursive decimal
emitter. Its same-region signature has one exclusive struct output, one owned
`u64` value, exact singleton reads and writes rows plus traps, and own unit.
The body compares the value with canonical `10_u64`; only the true arm divides
by the same nonzero constant and recursively calls itself on the quotient in a
fresh one-statement region. For unsigned values at least ten, that quotient is
strictly smaller, so recursion terminates in at most twenty decimal digits.
The tail takes the remainder modulo ten, then indexes an independently resolved
immutable `array<u8, 10>` containing exactly the canonical bytes 48 through 57,
and passes the resulting owned `u8` local through a second fresh region to the
independently re-proven byte-push callee. Both local regions differ from the
outer region, from every binding, and from each other; the first child ends
before the tail resumes the parent. No quotient, remainder, global value, or
bounds result is exported as an optimizer fact, and this slice adds no lowering
authority. Ordinary flow support is limited to exact trapping divide or
remainder by ten, the exact immutable digit table, owned local `u8` arguments
to the proven push, and calls to a fully re-proven emitter. The same-region
effect gate still returns the exact emitter-body verdict directly. Hostile
review pins the signature and effects, comparison, divisor and operation
modes, strict recursive decrease, self-call identity, local confinement,
shared, deeper, explicit, and bound calls, exact table type, length, order, and
values, remainder subscript, push-callee honesty, binding and region
uniqueness, source anchoring, and cyclic topology. It also caught and corrected
three initially mis-targeted source mutations before shipping. Exactly
`byte_tape_emit_u64` moves to CLEAN; no prior CLEAN function is lost and all ten
new helpers remain Unsupported. The unit is 637 total / 160 CLEAN / 477
Unsupported / 0 rejected; the exact 15-function LLVM module is unchanged. The
parser census is 4,772 regionful calls = 496 explicit + 4,276 staged omissions;
self-parse is deterministic at 1,721,479 bytes / 347,440 tokens / 172,434
nodes. The proof is split across 283-, 214-, and 199-line modules with a
332-line hostile suite; the shared signature and caller gate remain 231 and
347 lines, and the general reader is 6,779 lines. Fresh inventory leaves the
four exact chunk-plus-number wrappers `llvm_text_emit_value`,
`llvm_text_emit_place`, `llvm_text_emit_block`, and
`llvm_text_emit_block_ref` Unsupported; that closed four-function family is
the next bounded F4 boundary.

The eighth bounded F4 slice admits that exact four-function family. Each caller
retains the exact two-parameter same-region signature: one exclusive struct
output, one owned `u64` identifier, singleton reads and writes rows plus traps,
and own unit. Its body has exactly one fixed-literal chunk region, one recursive
decimal-emitter region that consumes the identifier, and a unit return, in that
order. Both statement-local regions differ from the caller region and from each
other. The chunk call independently re-proves the whole-parent unique child,
canonical literals, exact formal order, guarded chunk callee, and nested byte-
push callee. The numeric call independently re-proves the whole-parent child,
owned `u64` argument, exact formal order, terminating emitter body, immutable
digit table, and nested byte-push callee. Each child ends before the parent
resumes, so no unique siblings coexist. No identifier, digit, global, bounds,
or call result is exported as an optimizer fact, and no lowering authority is
added. Hostile review pins the exact signature and effects, ordered body,
canonical chunk literals, owned identifier use, shared, deeper, explicit, and
bound calls, region freshness, dishonest or missing callees, source anchoring,
and cyclic topology. It also corrected one invalid test assumption: region and
value names are separate syntactic namespaces, while outer and sibling region
freshness remain enforced. Exactly `llvm_text_emit_value`,
`llvm_text_emit_place`, `llvm_text_emit_block`, and
`llvm_text_emit_block_ref` move to CLEAN; no prior CLEAN function is lost and
the two new helpers remain Unsupported. The unit is 639 total / 164 CLEAN /
475 Unsupported / 0 rejected; the exact 15-function LLVM module is unchanged.
The parser census is 4,804 regionful calls = 496 explicit + 4,308 staged
omissions; self-parse is deterministic at 1,727,663 bytes / 348,731 tokens /
173,085 nodes. The focused proof is 90 lines, its hostile suite is 340 lines,
the shared caller gate is 355 lines, and the general reader remains 6,779
lines. Fresh dependency inventory selects `byte_tape_emit_probe` as the next
bounded F4 boundary: its reset, fixed-prefix, and recursive-number callees are
all CLEAN, while the adjacent span probe remains blocked by the Unsupported
`byte_tape_emit_span`.

`lexer_scan_string` remains the source-order
frontier, blocked by aggregate return and other deferred forms. Remaining F4
bounded statement-scoped reborrow, F5 aggregate construction/return, and F6
`allocates`/`move` follow in that order. Whole-unit LLVM lowering, including production emission of general
`eeq`/`ene` calls after revalidating their domain, remains the separate Phase-2
step-2 track and may not treat CLEAN classification as emission authority.

## Work outside the seven-phase scope

- `table`, pool, arena, other sealed components, checked-source libraries,
  catalog cards, and public I/O or FFI wait for a later owner-directed roadmap.
- Concurrency waits for a later roadmap. Its memory model, synchronization
  effect, sharing rules, target lowering, per-form models, and acceptance rows
  must land before any thread, queue, scheduler, or reclamation component.
  Retain the fixed-fan-out v1 choice. Reconsider the dynamic loop-spawn capture
  carve-out after the concurrency layer exists or when a named scenario needs
  runtime-count workers with shared outer borrows; require its own hostile
  review. Fold AMD-7 and AMD-8 into the ratified rules at that landing. Place
  MM-1 through MM-6 and MM-10 in the kernel specification, and keep the writer
  manual limited to MM-0 and MM-7.
- Fold AMD-6 into the ratified rules when a production phase first consumes
  branded endpoints. That obligation does not wait for concurrency if an
  earlier branded component needs it.
- A wfc migration to `seq` remains a measured experiment after phase 7. The
  current structure-of-arrays compiler stays as its control.
- The two completed shipped-library protocols stay frozen. Do not tune them or
  add a third target to rescue or enlarge their claims.
- Surface ergonomics and deferred proof-accounting promotion work require their
  recorded triggers and a later plan amendment.

## Process gates

- Run `make check` and `make -C compiler check` before and after each completed
  slice. Do not weaken a check to restore green.
- Give each completed step one commit and one appended decision-log line.
- Keep semantics, lowering, conformance, and measurement artifacts tied to the
  same frozen source hashes. Treat an unexplained difference as a defect.
- Require hostile review before shipping any new fact channel or production
  safety judgment. A green automated gate does not substitute for that review.
- Keep all repository artifacts in English. Keep `AGENTS.md` and `CLAUDE.md`
  byte-identical.
- Update the relevant MCTS node and its `.alt/` in the same change as any design
  redecision. Preserve rejected designs with paired reasons.
- Stop a phase on a red gate, an unresolved semantic flag, an unsound model, a
  failed mutation, or a missed performance band. Repair or reject the design
  before expanding scope.

## Evidence pointers

- Compiler state and architecture: `compiler/README.md`, `compiler/sources.txt`,
  `compiler/test_self_parse.py`, `compiler/test_semantic_unit.py`, and
  `compiler/test_llvm_supported.py`.
- Toolchain decision: `mcts_mem/whitefoot/toolchain.md` and
  its rejected alternatives under `mcts_mem/whitefoot/toolchain.alt/`.
- Proof contracts and reviews:
  `optimizer-language-research/implementation/requires-check-accounting-REVIEW.md`
  and `mcts_mem/whitefoot/fact-channels.md`.
- Capability landing evidence:
  `optimizer-language-research/implementation/systems-performance-coverage/DESIGN-DOSSIER.md`,
  `m1-loan-judgment/RULES-RATIFIED.md`, `m1-loan-judgment/programs.json`,
  `m2-spec-mass/KERNEL-DELTAS-DRAFT.md`, `m2-spec-mass/optables.md`, and
  `m3a-kernel-dryrun/RESULTS.md` under that directory.
- Completed shipped-library evidence: `experiments/default-floor/RESULTS.md`.

Evidence supports a gate. It does not advance the execution cursor.
