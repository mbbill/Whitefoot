# Current Plan: unified-state completion I/O

Status: IMPLEMENTED, VALIDATED, AND ACTIVATED on
`codex/io-first-principles`.

Active language authority: v0.38,
`5a43c7638bd5839d77829836518374f9a169eb953d9c1edbd66b87815aedfb2d`.
`spec/kernel-spec.md` carries those exact ACTIVE bytes; the superseded v0.37 is
archived at `spec/kernel-spec-v0.37.md` and the merge-time record is in
`governance/APPROVALS.md`. Activation is branch content: nothing merges to
`main` until the owner approves the exact revision and canonical `make check`
passes on that revision. The batch record is
`docs/done/0082-unified-state-completion-io.md`.

v0.38 rides on `batch/0091-par3-judgment` and adds one rule, [PAR-3], the
staged loop permission: it cuts the body of any `for_stmt` or `loop_stmt` at
its first `may-suspend` submission and admits executing the remainder of one
iteration against the prologue of a later one. It also amends [SYS-2] in one
sentence, to name the release milestone of the name an open borrows: batch
0089's adapters publish that loan release at submission, and the contract now
says so. That batch lands the judgment and its ledger only — no lowering and no
runtime change — so it grants a verdict a later batch actualizes and moves no
published byte today. Its record is `docs/done/0091-par3-judgment.md`.

The remaining hole in the two-host gate is closed. Batch 0090 put canonical
`make check` on a GitHub Linux runner and ended it red on one thing: the
compiler had no approved [SYS-14] directory-enumeration row for Linux, because
its record model required a per-entry name length and `struct linux_dirent64`
states none. Batch 0094 replaced that model with one that asks the target where
a name's length comes from — a field on Darwin, a scan bounded by `d_reclen` on
Linux — and landed the row. The two directory-walking corpus programs now
compile, run, and publish the same bytes on both hosts for the same tree; the
Linux conformance adapter reports the macOS number, `Pass=509 Skip=1`; and the
host limits 0090 had to declare are removed rather than narrowed. Its record is
`docs/done/0094-linux-directory-row.md`.

## Objective

Whitefoot exposes I/O as ordinary calls over ordinary values:

```whitefoot
fn write_once['o, 's](
  output: &uniq 'o Output,
  source: &'s buffer<u8>,
  start: own u64,
  end: own u64
) -> result: own Result<u64, IoError>
reads(output, source), writes(output);
```

The writer supplies values, `own`, `move`, borrows, exact effects, and typed
outcomes. The compiler keeps independent operations in flight. The target
publishes completion. The scheduler runs writer code only after the required
ordinary ownership returns.

Completion is the only writer-visible I/O model. Direct, inline, readiness,
helper, io_uring, and IOCP paths are target lowerings of that model.

## Settled language boundary

1. `reads` and `writes` name formal parameters or static fields rooted in a
   formal parameter. They no longer accept a lifetime.
2. A lifetime states only loan duration and outlives relations.
3. `own`, `move`, `&`, and `&uniq` provide all authority. Files, sockets,
   Outputs, clocks, factories, permits, and Sources are ordinary opaque affine
   values rather than a capability language category.
4. The language has one state/effect system. It has no `memory` or `world`
   effect tag and no `external` or `blocks` atom.
5. A fine effect path describes checked behavior. It never narrows the loan
   written in the parameter or actual argument.
6. Changing state machines use `own` or `&uniq`. Independent work is exposed
   through distinct ordinary owned values, fields, borrows, or permits.
7. No logical-root registry, family/fragment table, Free/Ordered/Exclusive
   relation, or Output-specific ordering edge participates in permission.
8. A factory or reserve call exhibits its own effect. Later operations on its
   local result frame locally; they are not relabelled through child-to-parent
   ancestry.
9. Existing move semantics transfers identity. The compiler may retain
   internal summaries needed to check calls and release, but the language gains
   no lineage or generative-identity feature.
10. A false claim is impossible in a correctly reviewed program and adds no
    cost, metadata, gate, wake, or serialization to a correct operation path.
11. File opens consume ordinary one-shot `FilePermit` owners produced by a
    total inline `reserve_file(&uniq FileFactory)`. `DirectoryRead` remains a
    shared stable selector, so two permits allow two opens through one
    directory without a long directory or factory loan.
12. The first permit is proof-only and burns on every open outcome. It reserves
    no host quota; `ResourceExhausted` remains a typed open result, and backend
    lowering erases the permit before the native open ABI.

The complete derivation is in
`research/investigations/io-model/FIRST-PRINCIPLES.md`; the concise selected
design is in `research/investigations/io-model/DESIGN.md`.

## Retained implementation substrate

The former completion candidate established useful target/runtime work. The
rebuild retained and requalified:

- finite generation-checked operation records;
- one terminal publisher and monotonic result-ready, per-path loan-release,
  and terminal milestones;
- release/acquire result publication and drain-before-resume;
- announce, recheck, then park with one compute/completion sleeping decision;
- target callbacks and helpers which receive typed operation bundles and never
  execute writer code;
- pure-compute link isolation;
- selective stackless lowering and direct/inline depth-one specialization;
- real Linux io_uring operations, bounded macOS helper fallback, and the
  Windows IOCP/OVERLAPPED foundation; and
- deterministic hostile-race, sanitizer, target, and performance probes.

The historical measurements in the I/O investigation remain evidence only for
the components they actually measured.

## Replaced or deleted

- REGIONID/capability mixed effect operands;
- separate memory and capability effect sets;
- `CapabilityResultOrigin` and any I/O-specific root propagation;
- family-fragment and Ordered permission;
- shared writer-visible mutation of Output or DirectorySource;
- ordered Output batches, fixed 2-16 or 2-64 group concepts, and whole-group
  waits;
- legacy bridge APIs which expose roots, families, or batch ordering; and
- compiler/runtime branches which treat a direct system call as a different
  source-language call class.

## Completed implementation sequence

1. Rewrite the first-principles, design, current-plan, roadmap, specification,
   and compiler-facing documentation so they state one model without a
   superseded sibling.
2. Replace effect syntax and semantic storage with parameter-rooted static
   paths. Normalize contracts by parameter and field ordinal.
3. Project user and system call effects directly onto actual resolved places,
   including slices and reborrows. Preserve ordinary owner identity across
   moves, results, aggregates, and compiler-derived release only where the
   existing value flow requires it.
4. Remove I/O capability origins, family relations, and Ordered IR edges.
   Permission continues to check data, control, operand reads, effects, actual
   loans, consumed owners, and exits.
5. Change advancing system APIs to `own` or `&uniq`. Keep shared positioned
   reads and shared directory selectors where the mapped value itself has no
   consumed cursor or observation state; carry each file-open occurrence in an
   owned one-shot permit.
6. Replace batch actualization with dependency-driven submission. A successor
   becomes eligible when its exact value and loan requirements return, without
   waiting for unrelated operations.
7. Remove the legacy bridge and group admission surfaces while retaining the
   qualified single-operation completion core and target-private channels.
8. Migrate all compiler tests, conformance cases, maintained programs, and
   documentation to the same specification bytes.
9. Run the complete verification matrix and measure the cleaned runtime again.
10. Archive the outgoing v0.36 bytes, activate v0.37, record the activation and
    conformance boundary, and bring every digest anchor to the new identity.

## Evidence obtained

The activated revision proves:

- two parameters sharing one lifetime remain separate effect subjects;
- an owned resource parameter is directly nameable in an effect;
- a field effect preserves sibling facts without narrowing a whole-object loan;
- two distinct outputs overlap and two unique loans of one output do not;
- a later same-output call starts after the earlier loan returns without
  waiting for unrelated I/O;
- a moved incoming owner is attributed correctly at call and release;
- no local fresh child requires a factory ancestry or hidden authority table;
- two short factory loans can mint two affine permits, and those permits admit
  two opens through one shared `DirectoryRead` without a retained factory loan;
- permit move is single-use, host exhaustion stays a typed open result, and no
  permit argument reaches the native open ABI;
- completion-before-wait, stale generation, duplicate terminal, capacity,
  cancellation, and multi-waiter races preserve every owner and loan;
- pure compute links and executes no completion machinery;
- API success, empty, partial, EOF, failure, untouched-tail, and defined
  release outcomes are correct; finish/recycle APIs remain outside this first
  slice;
- macOS executes its qualified fallback path; the retained Linux io_uring path
  has its existing native evidence but was not re-executed on this Mac; and
  Windows remains honestly fail-closed until native execution evidence exists;
- focused tests, maintained programs, conformance, sanitizers, stress, and
  every independently runnable component behind the specification archive
  gate pass; and
- cleaned fast paths are measured against the best matched native shape.

## Program-level performance, measured 2026-08-27

The evidence above is component evidence. Whole programs were first measured
on branch `batch/0084-io-perf` and re-measured on `batch/0086-open-handout`
with the base commit and the branch interleaved in one plan on a quiet host;
both tables are in `research/investigations/io-model/RESULTS.md` and the
bundle that produces them is `research/experiments/io-completion-bench/`.

On a many-independent-files workload the shipped build is about two times
faster than its own sequential build on macOS and about 2.8 times on Linux, so
the overlap is real. Against the native ceiling it is within 6 percent of a
hand-written io_uring pipeline running at the same queue depth the four-wide
source asks for; at eight-wide the same comparison opens to 26 percent, and
against a thread pool that also folds each file on the worker that read it —
compute parallelism the source cannot express — it is 1.5x on macOS and 3.0x
on Linux.

Batch 0086 moved the Linux open onto the ring, which removes the last blocking
`openat` from a scheduler thread at no measured cost, and established what the
remaining distance is not. It is not the opens: on Linux the entire
open-plus-close budget is 9 percent of the program, and moving it changed the
total by about one percent. It is not the completion protocol's per-operation
cost either: the four-wide program still matches a hand-written ring at the
same depth.

What remains is the width a source can express and the barrier that comes with
it. Overlap groups are runs of consecutive calls in one basic block, so a loop
with one I/O call per iteration overlaps nothing and measures like the
sequential build, and a hand-widened one joins its whole group before starting
the next — paying the maximum of N latencies per round where a native pipeline
keeps N continuously in flight. Deciding whether the language, the lowering, or
neither should widen and pipeline that is the next I/O question, and it is a
design decision rather than a defect.

No test, verdict, check, or failure path was weakened to make this revision
green. Canonical `make check` now runs to completion on the activated identity
rather than stopping at the candidate archive gate. No merge into `main`
occurs without owner approval of the exact revision.
