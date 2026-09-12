# Audit: the LLVM emitter and backend policy against the design tree

Module: `compiler/src/backend/emitter.rs`, every file under
`compiler/src/backend/emitter/` (`system`, `completion`, `parallel`, `runs`,
`integer`, `cleanup`, `places`, `operations`, `buffer`, `frontier`,
`conversion` and `conversion/float_endpoint`, `floating`, `slice`, `arena`,
`array`, `boxes`, `floor`, `reinterpret`), and
`compiler/src/backend/{backend.rs (the mod.rs), abi.rs, target.rs,
stack_ledger.rs, qualification.rs}` — about 19,150 lines excluding tests.
Direction: code to tree — finding choices the code embodies that `design/`
does not record, not the reverse. `compiler/src/backend/tests.rs` and
`compiler/src/backend/tests/` were read only as evidence of intended
behavior, per the audit's own rule, and nothing here proposes changing them.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md` and every node under
`design/compiler/` (`cleanup-traversal`, `parallel-lowering` with its
`two-worlds` and `parallel-runtime` children, `resource-exhaustion-floor`,
`tag-only-lowering`, `wide-probe-lowering`), `design/language.md`,
`design/language/data-model.md`, `design/language/system-interface.md`
and its `declaration-home`/`directory-enumeration` children, and
`design/language/effects.md`. Also read as the two finished audits' own
format: `design/recall-tmp/audit/resolution.md` and
`design/recall-tmp/audit/lexer-and-syntax.md`.

Two boundary notes:

- `compiler/src/backend/storage.rs` and `compiler/src/backend/graph.rs` sit
  inside `compiler/src/backend/` but are not part of this module family's
  file list; `abi.rs` and `cleanup.rs`/`frontier.rs`/`stack_ledger.rs`
  import from them (`is_stored_aggregate`, `components`), so they are read
  only far enough to understand a call site, never audited for their own
  content. `compiler/src/backend/sched/`, `compiler/src/backend/completion/`
  (the C sources), `windows_runtime.c`, `wf_floor.c`, and `lowering` are
  black boxes per the task's own instruction; a fact from one of them is
  cited only where the emitter's own text depends on it (a byte constant, a
  symbol name).
- Sources: `git log -S`, `git show`, and `git blame` over the full,
  already-unshallowed history (confirmed complete back to the true root
  `7c1d7641`, 2026-07-07, per `design/recall-tmp/sources.md`), plus the
  pre-built `design/recall-tmp/sources/commits.md` survey and `mcts_mem/`.
  `design/log.md` was also read in full: it records that both trees were
  migrated from `mcts_mem/` and reviewed with the owner on 2026-09-11 and
  2026-09-12, essentially concluding as this audit runs. That review
  directly settles one candidate finding below (the backend's silence on
  effect- and ownership-derived LLVM attributes) by an explicit, quoted
  owner ruling that the question needs no tree node at all — see the
  `design/language.md` item in §1. No git command that changes state was
  run.

## 1. Covered by the tree

- `design/compiler/cleanup-traversal.md` (one release action per node type,
  called again where its own release graph closes, so depth is the value's
  rather than the type's; no explicit worklist) —
  `compiler/src/backend/emitter/cleanup.rs::emit_cleanup_jobs`'s
  `IrNominalKind::Enum` arm (the self-call at the point the graph closes) and
  `emit_resource_drop_helpers` (line 24), whose own doc comment names "the
  owner's ruling of 2026-09-04" verbatim; `stack_ledger.rs`'s `STACK cycle`
  rows are exactly this decision's other stated consequence.
- `design/compiler/resource-exhaustion-floor.md` (decision 1: exhausting the
  stack or the heap ends the process through one defined abort naming only
  the exhausted resource class; a refused compute offer is never an
  exhausted resource) — every heap allocation site's OOM arm calls
  `wf_resource_abort()` and nothing else:
  `compiler/src/backend/emitter/boxes.rs::emit_box_new` (line 6),
  `arena.rs::emit_arena_new` (line 55), `buffer.rs::emit_buffer_fill`/
  `emit_buffer_vacant` (line 6, 74); `floor.rs`'s `FLOOR_RUNTIME_SOURCE`,
  `FLOOR_STACK_BYTES` (line 28), and `FLOOR_RUNTIME_FALLBACK` carry the same
  floor into every module unconditionally. The module keeps a second,
  disjoint abort path — a bare `call void @abort()` with no record, used
  only for a compiler-internal defect the checker should have made
  unreachable (an enum tag outside its declared range in
  `cleanup.rs::emit_enum_cleanup_body`, line 571; a `tcb.defect` arm in
  several of `system.rs`'s completion mappers) — which never claims to be a
  resource-exhaustion record, matching the decision's own text that "the
  absence of those fields is what distinguishes the record from a language
  trap record."
- `design/compiler/resource-exhaustion-floor.md` (decision 2: every
  generated function carries the target's stack-probing attribute instead of
  a per-prologue pointer check) —
  `compiler/src/backend/target.rs::TargetLayout::stack_probe` (line 34, 142;
  `__chkstk_darwin`, `inline-asm`, or `__chkstk` per target) and
  `compiler/src/backend/emitter.rs::attach_stack_probe` (line 679), which
  attaches the attribute group to every `define` in the finished module text
  rather than at each emission site, so a function introduced later is
  covered without anyone having to remember it.
- `design/compiler/tag-only-lowering.md` (Bool/≤2-variant tag-only enums to
  one bit, larger ones to 32 bits, consistently across values, calls,
  equality, and match) — `compiler/src/backend/emitter.rs::llvm_type`'s
  `IrType::Nominal` arm (line 2737: `i1` for ≤2 variants, `i32` otherwise);
  `emitter/operations.rs::emit_enum` (line 285, `or i1/i32 0, {variant}`) and
  `emit_enum_equality`; `buffer.rs::buffer_element_size`'s tag-only case
  mirrors the same 1-byte/4-byte split at the aggregate-storage layer.
- `design/compiler/wide-probe-lowering.md` (a header fast path built on one
  probe reporting provably effect-free upcoming iterations, falling back to
  the ordinary lowering on any mismatch) —
  `compiler/src/backend/emitter/buffer.rs::emit_buffer_probe_skip` (line
  281), whose own doc comment restates the decision's "reports how many
  upcoming iterations are provably effect-free" almost verbatim.
- `design/compiler/parallel-lowering.md` (decision 1: a range split descends
  to a subrange leaf, never a one-iteration leaf) and
  `.../two-worlds.md` (the clone set treats a synthesized chunk and splitter
  asymmetrically) — `emitter/parallel.rs::emit_loop_split`/`LoopSplitSite`
  (line 563): the sequential world calls the chunk, described in the
  module's own comment as "the loop itself, seeded with the accumulator's
  incoming value."
- `design/compiler/parallel-lowering.md` (decision 2: native queues, lanes,
  and completion state are target-private protocol state, never Whitefoot
  shared storage) — the whole `wf__par_*`/`wf__completion_*` symbol
  vocabulary in `parallel.rs` and `completion.rs` is compiler-and-runtime
  internal ABI, never a type a source program can name or hold.
- `design/compiler/parallel-lowering.md` (decision 3: Windows requires the
  compiler-owned runtime at link time; every other target keeps a working
  fallback; a host that starts fewer workers than asked is not a broken
  configuration) — `parallel.rs::PARALLEL_RUNTIME_DECLARATIONS` (Windows,
  hard `declare`, line ~93) against `PARALLEL_RUNTIME_FALLBACK` (line 122,
  `define weak`, every acquisition refused, "exactly today's schedule").
- `design/compiler/parallel-lowering/two-worlds.md` (decisions 1-4: two
  byte-identical-in-the-sequential-half lowerings selected once at bootstrap;
  the clone set is exactly the functions reachable from the entry that can
  reach a hand-out; `--par-sequential-refusal` reuses the existing refused
  edge; a component gets one budget-carrying variant, entered with the
  runtime's own answer) — `parallel.rs::sequential_clone_set` (line 238,
  forward-reachable-from-entry ∩ reaches-a-hand-out, with the chunk/splitter
  carve-out); the `refusal_clones`-gated arm of `emit_handed_out_call` (line
  426); `frontier.rs::RecursiveFrontiers` (line 46: excludes a member that
  may-suspend, carries a completion pipeline, is synthesized, or has no
  clone, naming the excluding member in `--par-ledger`); `emitter.rs`'s
  `RecursionBudget::{Off, Pinned, RuntimeDerived}` handling in
  `emit_recursion_budget_entry` (line 587).
- `design/compiler/parallel-lowering/parallel-runtime.md` (a full deque
  refuses and runs inline; a worker's stack is the entry's own fixed size)
  — `parallel.rs::emit_handed_out_call`/`emit_overlap_joins`'s
  acquire/publish/join/release call sequence (lines 426, 690) is exactly
  that protocol's caller side; `floor.rs::FLOOR_STACK_BYTES` is the one
  constant both the floor and every worker lane are sized from.
- `design/language/data-model.md` (decision 4: full fixed arrays, initialized
  prefixes, and circular windows are distinct states, with no mandatory
  handle or store indirection) — `emitter/runs.rs::RunShape::{Inline,
  Descriptor}` (line 20) and the whole run/window emission built on it:
  `wrap_offset` (line 777) and `boundary_slot` (line 733) each do the
  wraparound in one conditional subtract rather than a modulus, which is
  [BLK-1]'s own stated arithmetic once the domain bound `sum < 2·cap` holds.
- `design/language/data-model.md` (decision 5: a signature's checked
  ownership and access modes are independent of how a value is represented,
  and representation grants no permission) —
  `compiler/src/backend/abi.rs::FunctionAbi::build` (line 53) selects
  `ParameterAbi::Value` vs. `ContentPointer` purely from
  `is_stored_aggregate`, a representation fact, never from a parameter's
  source-level mode.
- `design/language/system-interface.md` (decision 1: system access is
  ordinary owned resources under ordinary ownership, with no separate
  capability category or bespoke runtime mechanism) and
  `.../declaration-home.md` (system types and operations resolve from one
  distinct compiler-owned domain) —
  `compiler/src/backend/qualification.rs`'s whole `[QUAL-1]`/`[SYS-2]` table:
  every `SystemResourceType` gets one fixed, ordinary-value representation
  (`Descriptor` = `i32`, `ProofToken` = `i1`, `InlineLease`/`ArgumentVector`
  = `{ ptr, i64 }`, `InternetAddress` = `{ i64, i64, i32 }`,
  `ResourceRepresentation` line 979), reached only through the operation
  table and never through a bespoke capability type.
- `design/language/system-interface.md` (decision 3: arguments and paths
  preserve the target host's own bytes with explicit conversion) —
  `qualification.rs::CodeUnitFamily::{Unix, Windows}` (line 402) and the
  `InlineLease` representation `[HOST-3]` fixes for `HostString`/
  `RelativePath`.
- `design/language/system-interface.md` (decision 4: exactly one
  uncallable, uncontracted `command fn main`) —
  `emitter/system.rs::emit_entry` (line 3848), `ENTRY_BODY_SYMBOL`, and
  `START_FAILURE_STATUS` implement `[PROG-3]`'s bootstrap exactly as
  described: one process-owning bootstrap, one invocation of the entry, one
  `ExitStatus` mapped onto the host process status.
- `design/language/system-interface/directory-enumeration.md` (decision 1:
  one-attempt bounded-batch transfer into the caller's own buffer, no
  handle-owned window; decision 2: self/parent entries unfiltered, no
  promised order) — `qualification.rs::DirectoryEnumeration` (line 756) and
  its three target instances, `DARWIN_ENUMERATION`/`LINUX_ENUMERATION`/
  `WINDOWS_ENUMERATION` (lines 841, 872, 894), each with the measured native
  record layout and an explicit note that `readdir`/`opendir` are excluded
  because "[QUAL-3] excludes" (line ~838) the extra allocation and call they
  would cost.
- `design/language/effects.md` (decision 2: host scheduling belongs to
  target lowering, invisible to source effects; decision 4: `pure` promises
  nothing about termination) together with `design/language.md` (decision 3:
  "whatever the compiler hands its backend, such as aliasing attributes or
  inlining, is an implementation detail the language never sees") — the
  entire completion/parallel runtime is unreachable from a source effect row,
  and separately the emitter adds no effect- or ownership-derived optimizer
  fact at all: no `willreturn`/`memory(none)` and no `noalias`/alias-scope
  metadata anywhere in the module (confirmed by search). This is not an
  oversight the tree is silent on:
  `compiler/src/backend/tests/effect_attributes.rs` is a standing tripwire
  that fails "on the day anyone emits `willreturn`," citing `[EFF-3]` by
  name, and `design/log.md`'s
  2026-09-12 compiler-root entry records the owner striking the tree's own
  prior optimizer-fact nodes for exactly this reason, quoted there: "what the
  compiler hands LLVM, such as aliasing attributes or inlining, is an
  implementation detail the language never sees and needs no rule." The two
  historical experiments that measured a real win from these attributes on
  the retired prototype (`research/experiments/effect-attrs-channel/`,
  `scoped-alias-channel/`) are correctly kept as `docs/roadmap.md` candidates
  (`outline:PROOF-2`/`PROOF-3`), not as compiler-tree decisions, since
  `docs/roadmap.md` is explicitly not part of this loop.

## 2. No decision needed

- No `unwrap`, `expect`, `panic!`, or `unreachable!` appears in this
  module's production code (confirmed by search); every occurrence is inside
  `#[cfg(test)]` (`qualification.rs`'s `deterministic_test`/`probe*`
  builders, `stack_ledger.rs`'s and `completion.rs`'s own `mod tests`).
  Every internal invariant instead threads through `BackendFailure`,
  `TargetLayoutFailure`, or `QualificationFailure` — ordinary engineering
  quality, not the "hardening and re-verification" pattern
  `design/compiler.md`'s own Rejected list declines to require of this
  compiler.
- Every `emit_*` function re-checks that the IR's own recorded operand and
  result types agree with what the operation expects before emitting a
  single line, failing `InvalidIr` on any mismatch — ordinary defensive
  consumption of a typed IR, not a re-derivation of a judgment an earlier
  stage already made.
- `reinterpret.rs::emit_reinterpret`'s same-width, differently-signed case
  and `conversion.rs::emit_integer_cast`'s same-width case both render `or
  i{width} x, 0` rather than a `bitcast`, because LLVM has no separate
  signed/unsigned integer type to bitcast between and every `IrValueId`
  still needs a defining instruction; `operations.rs::emit_constant` and
  `emit_insert_sequence`'s empty-aggregate arm use the same trick
  (`select i1 true, C, C`) to bind a plain constant to a fresh name.
- `conversion/float_endpoint.rs` implements `[OP-6]`'s EXACT `cvt` semantics
  for float↔int by round-tripping a saturating cast and comparing for
  equality, with two explicitly documented power-of-two/precision collision
  corrections, rather than a direct range comparison — a non-obvious but
  ordinary numerical technique for testing exact representability with the
  primitives LLVM actually offers.
- The storage plan's two-tier value representation — a plain SSA register or
  an addressed stack slot reached by `getelementptr`/`load`/`store`, chosen
  per value by `places.rs` — is ordinary compiler technique, not a language
  or backend policy question.
- `target.rs::checked_add`/`checked_mul`/`align_up`'s target-layout
  arithmetic is a direct, checked transcription of `[OP-9]`'s own
  layout-ceiling algorithm ("unbounded mathematical integers... round each
  current offset up"); the closed set of exactly five named target triples
  in `TargetLayout::for_triple` and
  `qualification.rs::SystemTarget::for_triple`, refusing anything else
  rather than approximating a nearby ABI, is the same discipline `[QUAL-2]`
  requires per target.
- `qualification.rs`'s 28-row `[SYS-7]` errno/Win32 error-class tables
  (`DARWIN_ERROR_CLASSES`/`LINUX_ERROR_CLASSES`/`WINDOWS_ERROR_CLASSES`) and
  the three directory-record field-offset tables are mechanical, measured
  target facts, each independently justified in its own doc comment; no
  competent implementation targeting these hosts would transcribe them
  differently.
- `system.rs::catalog_ir_type`'s explicit rejection of ABI-equivalent-but-
  distinct source types (signed vs. unsigned of the same width, a buffer
  element vs. an opaque resource of the same LLVM shape) before use is
  ordinary `[QUAL-1]`-mandated exact-identity validation, not a policy of
  its own.
- No custom LLVM calling convention (`fastcc`, `tailcc`, and the like)
  appears anywhere in the module; every emitted function uses the ordinary
  default convention, consistent with the ABI decisions already covered
  above and adding nothing beyond them.
- The stack-probe attribute is attached by one textual pass over the
  finished module rather than at each function's own emission site
  (`attach_stack_probe`); this is an implementation-robustness technique in
  service of the covered decision above, not a second policy.

## 3. Choices without a node

**1. The stack ledger derives every number from a completed host
compilation of the already-emitted module, with a fixed per-architecture
correction for what `-fstack-usage` does not report.**
`stack_ledger()` never estimates a frame from anything the emitter itself
tracked; it runs the host C compiler on the finished LLVM text with
`-fstack-usage`, reads the call graph back out of that same compilation's
assembly, and (since 2026-08-27) adds a fixed `call_return_address_bytes`
constant per architecture — 8 for x86-64, 0 for arm64 — before treating the
two numbers as comparable "cost of one activation" figures.
Alternative: estimate or accumulate a frame size during target
qualification (before codegen exists at all), or use `-fstack-usage`'s raw,
uncorrected numbers.
Where: `compiler/src/backend/stack_ledger.rs::stack_ledger` (line 79),
`parse_frames` (line 207), `Architecture::call_return_address_bytes` (line
194, with its own long doc comment).
Reason (quoted, commit `87afd8db`, 2026-08-23, "backend: report what a level
of stack costs and how many the program gets"): "It is measured after
codegen because nowhere earlier has the numbers. The compiler does
accumulate a frame size during target qualification, and on the deepest
recursion in the corpus that accumulator reports zero bytes for the function
that ends the program: its real cost is the ABI frame record a non-leaf is
forced to keep plus the spills the register allocator chose, and neither
exists before LLVM runs." The per-architecture correction has its own
later, equally explicit reason (commit `45ef81f0`, 2026-08-27, "compiler:
charge every frame the return address x86-64 does not report"): "The Linux
gate found the stack ledger over-promising recursion depth by a factor of
two: it reported 134,217,728 levels of the tight spine on the one-gigabyte
runtime stack and the program died at 134,083,510... Over-promising depth is
the dangerous direction for this artifact: a writer who believes it writes a
recursion the machine cannot run." This reads as a deliberate, carefully
measured architecture choice — not a defect — but it is a real, non-obvious
compiler policy (measure post-codegen from host-compiler text output, and
trust a fixed correction constant rather than a per-build measurement) that
no `design/compiler` node states, unlike the sibling facts about the same
subsystem (`resource-exhaustion-floor.md`'s abort mechanism,
`cleanup-traversal.md`'s cycle-row reporting) that already have one.
Effect: tooling accuracy only, on a channel `design/compiler.md` itself
calls non-normative developer output; it changes no acceptance and no
lowering.

**2. A compute overlap group's members are joined newest-published-first,
never in publish order, as one emitter-wide policy with no counterpart in
the runtime's own design node.**
`compute_join_order` reverses only the compute members of a group's publish
queue (leaving completion members exactly where they were published) before
`emit_overlap_joins`, `overlap_join_tail`, and `block_exit_label` consume it,
because the runtime's own deque is Chase-Lev: the owner's own end always
holds either the newest entry or nothing (already stolen), never an older
entry buried under a newer one.
Alternative: join in publish order (the naive choice, and the one every
group with exactly one member cannot distinguish from this one), or teach
the runtime a notion of "the group's own join order" instead of fixing it in
the emitter.
Where: `compiler/src/backend/emitter/parallel.rs::compute_join_order` (line
788, with its own paragraph-length justification) and its three call sites
named in its own doc comment.
Reason (quoted, commit `bccf1813`, 2026-09-05, "Join a group's compute
members newest first through one compute_join_order"): "Design §4: the
deque is Chase-Lev, so joining newest first finds the target at the owner's
end or already stolen, never buried." This is a deliberate, well-reasoned
choice grounded in the runtime's own concurrency structure — but
`design/compiler/parallel-lowering/parallel-runtime.md` documents the deque
mechanism itself (offer, refusal, waiting, idle window, worker stack) and
never states this specific consequence for a multi-member hand-out group,
which is squarely the emitter's own concern.
Effect: correctness-adjacent but not acceptance-affecting — [PAR-1] fixes
every value to its source-order result regardless of join order, so this is
purely which order the runtime is asked to wait in, chosen once so every
join finds its target cheaply instead of occasionally parking.

**3. A handed-out call's frame belongs to the acquired lane, and the lane
must be claimed before the frame is built — reversing an earlier shape that
put the frame in the calling function's own stack.**
`emit_handed_out_call` calls `wf__par_acquire_lane` first and only builds
the argument/result frame (the `getelementptr`/`store` sequence) inside the
branch where a lane was actually granted; a refused lane reserves nothing
beyond the null pointer the acquisition itself returned. The module's own
doc comment states this as a load-bearing ordering fact ("Lane acquisition
comes before the frame"), not an incidental one.
Alternative: build the frame unconditionally in the calling function's own
entry block (a stack slot present on every activation whether or not a lane
is ever granted), which is what an earlier version of this emitter did.
Where: `compiler/src/backend/emitter/parallel.rs::emit_handed_out_call`
(line 426) and the module doc comment at the top of the file (lines 19-28).
Reason (quoted, commit `b6f496b4`, 2026-08-21, "compiler: claim a lane
before building the hand-out frame"): "A handed-out call used to store its
arguments into a stack slot of the *calling* function, so every activation
of an eligible recursive function carried that slot and its spills whether
or not a lane was ever granted. The measured price was four times the stack
per frame on a small activation and a bare SIGSEGV where the sequential
build ran fine — the same schedule under a different resource envelope,
with the pool off."
Effect: both. Performance: a refused (or never-attempted) hand-out now costs
nothing extra on the calling stack. Safety: this is the fix for a real,
measured stack-exhaustion regression under `--par` with the pool off — a
program that ran to completion sequentially could previously crash once
compiled with `--par`, purely from the added per-activation slot, which is
exactly the kind of fact `design/compiler/parallel-lowering/two-worlds.md`
and `parallel-runtime.md` were written to police (their own text discusses
frame layout and worker-stack sizing) yet neither mentions where a
handed-out call's own frame lives relative to its caller's frame.

**4. The finite-completion I/O protocol (file, directory, and TCP
operations) is a hard link requirement on every target, with no working
fallback anywhere, unlike the sibling compute/parallel runtime.**
A module that submits any file, directory-enumeration, or socket operation
names the completion runtime's seven submit entries as plain external
`declare`s on every target (`COMPLETION_RUNTIME_DECLARATIONS`, non-Windows
included) — never as `define weak` bodies the way the parallel runtime's
four lane-protocol entries are on non-Windows targets. A missing completion
runtime is therefore an unresolved-symbol link failure everywhere, not a
silently-sequential fallback anywhere.
Alternative: keep a weak "execute the request directly, inline" fallback
for a link with no completion runtime, mirroring exactly what the parallel
runtime still does on non-Windows targets today.
Where:
`compiler/src/backend/emitter/completion.rs::COMPLETION_RUNTIME_DECLARATIONS`
(line 259) and
`module_requires_completion_runtime` (line 311, "the direct family has no
spelling left to look for"); `compiler/src/backend/emitter.rs`'s
Windows/non-Windows declaration-selection site (lines 486-520), whose own
comment states the policy directly: "a submit answers nothing, so there is
no weak body that could stand in for the runtime, and a link that omits it
is an unresolved symbol rather than a program that silently runs a second
arm."
Reason (quoted, commit `60e6a2ae`, 2026-09-05, "The completion path has one
arm: submit, then join"): "The seven submits return nothing; an emitted
module that submits requires the runtime, so the weak fallbacks that
answered 'run it directly' go, with the inline arm, the offered phi, the
eligibility branches and the Windows verdict fork." The reason is complete
and consistent (a submit-only, no-verdict protocol has nothing left for a
weak body to *do*), but it is recorded only in this commit and in
`research/investigations/io-model/PARK-ON-MISS.md` §8 — `design/compiler`
has no node for the completion/I/O runtime at all, so nothing states that
its link policy is deliberately *stricter* than the parallel runtime's
sibling policy that `parallel-lowering.md` decision 3 does cover.
Effect: linking only, on every target — a program with no I/O at all is
unaffected; a program that touches a file, a directory, or a socket now
depends on this runtime unconditionally, which is a real behavioral
difference from how the compute/parallel side of the same emitter behaves
by design.

**5. `qualification.rs`'s target-mapping table stays honest against
specification drift through exactly one hand-checked version-string tripwire
plus a per-version review comment, both maintained by hand and never
generated from the specification.**
`REVIEWED_FOR` is one string constant, compared against the live
`spec_identity::SPEC_VERSION` inside `command_entry_row` alone (every other
row's check would "force no additional review, only additional bumps," per
its own comment); above it sits a chronological, per-activation English
review note running from v0.33 through v0.53, each paragraph arguing why
that version's amendment leaves every operation, resource, and release row
unchanged (or, where it does not, exactly which row moved).
Alternative: generate `REVIEWED_FOR` (and the review requirement) from the
specification automatically; or review each table row's own currency
independently rather than gating the whole table on one command-entry
check; or drop the running commentary and rely on the version-mismatch
failure alone.
Where: `compiler/src/backend/qualification.rs::REVIEWED_FOR` (line 386, with
its own justification at lines 33-47) and `command_entry_row` (line 1800);
the version-by-version comment block above it (lines 48-386).
Reason: no commit body was found beyond the code's own text (`736b06ca`,
2026-08-20, "spec: complete the v0.33 target qualification candidate", is
bodiless); the comment itself is the fullest reason on record: "Do not
generate this constant from the specification (that would delete the review
it exists to force), and do not add more copies." This is a deliberate,
carefully maintained engineering discipline — twenty-one version reviews
deep and still current at v0.53 with no gap — but it is a real compiler
policy with genuine alternatives (automation, per-row checks, or no running
commentary at all), and no `design/compiler` node records that this is how
the project keeps a hand-authored target-mapping table from silently going
stale under `spec/kernel-spec.md`'s own append-only version discipline.
Effect: none on acceptance or on any emitted program; a lapsed review would
surface as every program failing target qualification at the command entry,
never as a silent staleness.

## Summary

- Covered by the tree: 17
- No decision needed: 10
- Choices without a node: 5 (5 architecture/measurement choices, 0 likely
  defects found in this pass)
