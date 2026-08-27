# Batch 0082 — unified-state completion I/O: one ownership system for memory and the outside world

Branch: `codex/io-first-principles`, from main at `eab81a33`.
Deliverables: the unified-state rebuild across eight commits, `4ef85df6`
through `ea700f7f`; spec v0.37 activating it as rule text; this record.

## What was built and why

v0.36 answered "what does this call touch?" twice. Memory was answered by
ordinary ownership — `own`, `move`, `&`, `&uniq`, place overlap. The outside
world was answered by a second, parallel vocabulary: a `world` region an
effect row could name, an `external` effect atom, a `blocks` effect atom, a
system-capability class that made resource types their own permission
category, and an `Ordered` relation that serialized outside actions against
each other. Two systems described one question, so they could disagree, and
where they disagreed the second one won by being coarser.

v0.37 deletes the second system. There is now one ownership system and one
`reads`/`writes` row, and the row's operands are formal-rooted static state
paths rather than lifetimes:

```whitefoot
fn write_once['o, 's](
  output: &uniq 'o Output,
  source: &'s buffer<u8>,
  start: own u64,
  end: own u64
) -> result: own Result<u64, IoError>
reads(output, source), writes(output)
```

The separation this buys is the whole point of the change. A lifetime says how
long a loan lives; a state path says which state the body touches. Those are
different questions, and REGIONID could not answer the second one: two
parameters may share one lifetime, and an owned state resource has no borrow
lifetime at all. Naming the formal directly makes `output` and `source`
distinct effect subjects even under one region, and makes an `own` parameter
nameable in a row for the first time.

Deleted outright, with no replacement: the `world` region, the `external`
effect, the `blocks` effect, the system-capability class, the logical-root
registry, the family/fragment relation, and the `Ordered` relation.
`external` and `blocks` are now ordinary identifiers — the grammar's fixed
lowercase atom set shrank by two. A direct system call is judged under
[PAR-1] by exactly the effect, loan, dataflow, and exit permission a user call
gets; nothing in the rule asks whether the callee is a system operation.

Underneath, the runtime is completion-only. Completion is the sole
source-level I/O model; direct and inline depth-one execution are lowering
specializations of it, not a second API. Operation records are finite and
generation-checked, one publisher owns each terminal, and result publication
is release/acquire with drain-before-resume. macOS uses a bounded typed helper
fallback for regular files; Linux submits real `IORING_OP_READ` and
`IORING_OP_WRITE` entries and waits on one epoll set holding the ring fd;
Windows has an IOCP/OVERLAPPED foundation that strict-cross-links as an
x86-64 PE but has never been executed. Target helpers receive typed operation
bundles and never a writer function pointer, so a helper thread cannot become
a second executor of Whitefoot code — the concrete failure the discarded
experimental branch had.

File opening is where the model had to prove it needed no new mechanism.
An open is an observation occurrence, so it consumes an ordinary affine
one-shot `FilePermit` minted by a total inline `reserve_file(&uniq
FileFactory)`. `DirectoryRead` stays a shared selector. Two short factory
loans therefore mint two permits, and two opens proceed through one shared
directory with no retained factory or directory loan — using nothing but
affinity and loan scoping. The permit is proof-only: it reserves no host
quota, `ResourceExhausted` remains a typed open result, and backend lowering
erases the permit before the native open ABI.

## The semantic decision worth stating plainly

[EFF-5] no longer orders distinct values. Under v0.36 every outside action was
sequenced against every other. Under v0.37, coexistence is decided by owned
places, exact effect paths, and loans, so two calls on two *distinct* `Output`
values may overlap — and if the host has redirected stdout and stderr to one
sink, their bytes may interleave there.

This is a real weakening of an observable, and it is deliberate. The language
never promised anything about a sink two values happen to share; v0.36's
blanket order promised it accidentally, and paid for it by serializing every
independent outside action. The measured price of that accident was roughly 24
to 30 percent in the rejected experimental revision's own comparisons. What is
still guaranteed is what a writer can actually name: two calls taking
successive unique loans of *one* `Output` run in that order. The runnable
conformance case `run-sysout-redirect-same-sink-order` was rewritten to
observe exactly that — the byte order of two successive unique loans of one
`Output` — rather than the cross-value order that no longer holds.

## Evidence

Canonical `make check` at the repository root, run on the activated tree,
ends `== WHITEFOOT ALL TESTS GREEN ==`. Its own output:

```text
default Rust library tests       1321 passed, 0 failed
gate all-target Rust tests       1412 passed, 0 failed, 1 costly adapter ignored
maintained programs              54 passed, 0 failed
conformance structure             29 of 29
conformance coverage             137 of 137 rules, 0 uncovered
separate conformance adapter     502 Pass, 1 Skip
```

Sanitizers, the hostile stress loop, and the native helper policies are not in
the canonical entry point; they are separate `compiler/Makefile` targets
(`completion-sanitize`, `completion-core-read-stress`,
`completion-windows-cross`) invoked independently, and
`research/investigations/io-model/RESULTS.md` records their results on this
tree: ASan and UBSan pass, the core/read hostile stress passes 200 of 200, and
the macOS helper policies pass at 0, 1, and 4 helpers. Those are that
document's measurements, not this gate run's.

The measurement that selected the lowering boundary
(`research/investigations/io-model/RESULTS.md`, Mac16,12 / Apple M4 /
macOS 26.5.2, cached reads, three complete runs): the generation-checked
accepted-terminal round trip is stable at 35.594 to 36.299 ns/op; direct
cached read is 410.6 to 423.9 ns/op against 468.0 to 494.9 ns/op for
zero-helper completion progress, an added 57.4 to 71.1 ns or 14.0 to 17.0
percent. Positioned reads and 64-byte writes had run-to-run dispersion large
enough to swamp the delta, and no comparative number is claimed for them.
That is what selects the specialization rule: a call with no independent work
should not pay the core, and a call with real independent work may pay ~60 ns
to expose overlap.

The load-bearing correctness witness is not a shape assertion on emitted IR —
it is `independent_io_reaches_the_second_operation_before_the_first_unblocks`
in `compiler/src/backend/tests/completion.rs:562`. It builds and *runs* the
emitted executable with the bulk stdout pipe deliberately blocked, and
requires the independent marker write to arrive on stderr within three
seconds, repeating under `WF_IO_HELPERS` of 1, 0, and 4. If overlap were only
a claim in the rule text, that test would time out. A sibling test,
`a_reused_unique_output_waits_only_for_its_own_prior_operation`, pins the
other direction at the IR level.

The Windows evidence is compile-and-link only: the probe's import table
contains `CreateIoCompletionPort`, `GetQueuedCompletionStatus`,
`PostQueuedCompletionStatus`, `ReadFile`, and `WriteFile`, reproducible with
`make -C compiler completion-windows-cross`. No Windows runner executed it.
Production Windows qualification stays fail-closed.

## The design record

- `research/investigations/io-model/FIRST-PRINCIPLES.md` is the derivation and
  supersedes the earlier mixed memory/world design in place. `DESIGN.md` was
  rewritten to the selected concrete API and lowering surface and defers the
  derivation to it.
- `HANDOFF.md` was deleted: it described a handoff that has happened.
- `RESULTS.md` and `IMPLEMENTATION-AUDIT.md` are retained as historical
  evidence. The audit's runtime measurements and its finding about writer-code
  helpers still stand; its positive references to capability roots, family
  fragments, and ordered attribution are superseded and are not requirements.

## Spec v0.37 (activation in this branch, approval at merge)

Rule count stays 137. The grammar gains one production, `effect_path`, and
loses two fixed lowercase atoms. System operations go 199 to 203: positioned
read adds `file_offset`, four open operations each add one `permit`, file
reservation adds one operation, one region, and one parameter, and the
backend-only `Interrupted` and `WouldBlock` constructors with their four
fields are removed — no-progress interruption and readiness refusal are target
progress, not portable errors, which is why the portable `IoError` set drops
from thirty classes to twenty-eight. Writer spellings: `read_at`,
`open_directory_source`, and `directory_next` replace the cursor-shaped
spellings; `reserve_file` is added; `FileFactory` and `FilePermit` are added;
`DirectoryList` becomes `DirectorySource`. [EFF-5] is rewritten as described
above, and effect, system, release, trap, and overlap rules are amended.

v0.36 is archived byte-exact as `spec/kernel-spec-v0.36.md`; the chain,
generated identity, qualification review note, and every digest anchor name
v0.37 at
`ee9f12ec9356267c13b536e962288ebbffa0b3507cfac0a5345f99e8dce53619`. Canonical
`make check` is green end to end.

## Open items, stated honestly

Language and API surface not yet built:

- **No clock or `now` API.** Reading time is the obvious next state value, and
  it raises a question this model has not answered: timing code wants an
  *ordering fence* against work that has no data dependence on the clock, and
  v0.37 deleted the only mechanism that used to provide one incidentally.
  Ordering that a writer can name must come back as something nameable, not as
  a restored blanket order.
- **Directory entries are not keyed places.** The natural spelling is
  `cwd[name]`, which would make two opens through one directory disjoint by
  key rather than by permit. Until that exists, `DirectorySource` is a fresh
  owner with no whole-directory loan behind it.
- **No create or namespace-mutation API** — no create, rename, unlink, or
  permission change.
- **No quota or recycle.** The permit reserves nothing; exhaustion stays a
  typed result.
- **No network, timer, deadline, cancellation, or finish-required output
  APIs.** A finish operation with a meaningful result is designed for but not
  built.

Implementation limits:

- **Windows is not executed or qualified.** Cross-link only, and the current
  IOCP wake packet is neither coalesced nor persistent for every
  already-announced waiter. Both must close before `implemented` changes.
- **Stackless suspension covers only single-instruction tail chains.**
- **Hand-out is limited to fd-backed read and write.**
- **No program-level performance measurement.** Every number above is a
  component or microbenchmark on one cached-read host. Nothing here predicts
  cold storage, durable writes, io_uring throughput under load, IOCP, or
  scheduler contention inside a whole Whitefoot program.

Compiler performance:

- **`tests/programs/wfgrep.wf` costs about 22 seconds to compile.** Measured
  on this host with the release `whitefootc`: 22.2 s wall, 18.5 s user, for
  `--emit-llvm` over the 53 KB source. A 8-second `sample` of that run
  attributes 4,739 of 5,861 stack samples (81 percent) to one path:
  `semantic::check::Checker::claim_residuality_outcome` ->
  `analyze_function_inventory_with_mask` ->
  `entailment::flow::analyze_candidate_with_mask`. The driver is therefore the
  claim-residuality search re-running the *whole* entailment flow analysis once
  per candidate mask, not any single leaf. Inside that loop the largest
  contributors are `close_with_excluded_term` (586 frames), `join_at_once`
  (527), `DerivationLedger::intern_for` (503), and `insert_closed_candidate`
  (409). One structural aggravator is visible in the source: the closure memo
  at `compiler/src/semantic/entailment/state.rs:3063` is guarded by
  `excluded.is_none()`, so every `close_excluding_term` call — one per
  `value_if` edge — bypasses the cache and recloses over the function's whole
  term universe. Sharing a base closure across masks is the obvious first
  experiment, but the mask loop above it is where the factor lives. This is
  the first place where a real program's compile time, not its runtime, is
  what hurts.

## Approval classes for the merge

- **Specification bytes change** (v0.37 activation): the merge-time record is
  in `governance/APPROVALS.md` and becomes effective with the owner's merge
  approval of this exact revision.
- **Conformance content changes**: relative to the v0.36 activation boundary
  at `main` tip `eab81a33`, two case files are added
  (`accept-sysfile-two-permits-shared-directory.wf`,
  `reject-sysfile-permit-used-twice.wf`), ninety-five case files and
  `manifest.jsonl` are modified, and nothing is deleted or renamed. In the
  manifest, two records are added and twenty-one are modified in place —
  three rule annotations ([CAP-1], [SYS-7], [EFF-5]) and eighteen case records
  carrying rule-list and doc-text updates for the renamed operations, the
  `DirectorySource` spelling, the explicit `Args` state path, the `FilePermit`
  parameter, and the five-row command input table. No pre-existing `expect`
  verdict changes. The exact boundary is recorded in
  `governance/APPROVALS.md`.
- **No new root entries.**
