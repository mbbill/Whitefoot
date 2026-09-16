# Test-system simplification

This investigation records the selected changes and remaining questions for
replacing redundant verification machinery. The [inventory](test-inventory.md)
and its [Chinese translation](test-inventory.zh-CN.md) describe the existing
system; this document describes subsequent work, not implemented behavior.
Keep it current during that work and retire it when the replacement system's
guidance and design decisions cover these choices and no questions remain.

## Selected direction

The discussion on 2026-09-15 selected these changes for later implementation.
First remove redundant assertions, derived copies and unnecessary test/tool
boundaries while retaining useful correctness and performance-regression
checks. Defer timing optimization and new timing campaigns until those
responsibilities are clear. Required correctness validation still applies;
existing execution guards remain in force.

1. **Remove the Rust specification tool.** Delete `whitefoot-spec`, its
   Cargo binary/test target and its gate invocation. Fold useful document
   checks into the existing compiler-independent Python conformance tooling:
   duplicate rule definitions and unresolved rule references, with the
   necessary malformed-input tests. That tooling already reads the active
   Markdown and extracts rule IDs. Remove redundant same-source identity
   comparisons rather than translating them. Keep the build-derived identity
   and specification embedding used by the compiler. Do not automatically
   reproduce `--index`/`--counts`; no automatic caller was found, and a
   retained query needs a concrete consumer.
2. **Remove the specification's Status line.** The active path already names
   the active specification and the title already names its version.
   Delete `Status: ACTIVE vN` and the checks for that duplicate field.
   `build.rs` already reads the version from the title. Update the
   `AGENTS.md`/`CLAUDE.md` wording about changing two version lines and
   affected fixtures/comments. This changes specification bytes, so use the
   normal version/archive procedure against the then-current main; preserve
   existing archives. No WF syntax or semantic change is intended.
3. **Generate parser tables during the build.** Retain the Rust EBNF/table
   derivation logic, invoke it from `build.rs`, write the generated table to
   Cargo's `OUT_DIR`, and include that output in the compiler. Remove the
   independent `whitefoot-grammar-tables` binary/test target, committed
   `compiler/src/syntax/grammar/generated.rs` copy and the test that merely
   regenerates and compares that copy. Preserve tests of actual parsing and
   diagnostics. Track generation inputs and keep generation independent of
   the compiler library it produces data for. Automatic derivation removes
   stale-copy maintenance; it does not prove the generator algorithm correct.

The [proposed build-input decisions](../../../design/amendments/compiler-build-inputs.md)
record the corresponding tree amendment. The live tree has not changed and
the exact tree revision has not received an owner ruling. The separate
verification-cost amendment now also proposes the selected test responsibility
and admission principles; its original four decisions remain unchanged.
The proposed `design/compiler/build-inputs.md` is a new child of the
compiler root and replaces no existing live node. Corpus case retirement
and the replacement integration-target layout are outside that amendment.

## Selected corpus responsibilities

On 2026-09-15 the owner selected the four corpus responsibilities in the
table below: normative requirements in conformance, whole-program behavior
in programs, canonical checks without a dedicated executable, and individual
snapshot migration or retirement. Execution remains deferred while discussion
continues. This does not authorize deleting all 484 snapshot cases or settle
individual retirements and the final target layout.

## Basic test contract

The owner selected the review method: establish this baseline for compiler
`#[test]` cases and the four corpora first, then consider the remaining
checks. The discussion now uses the related batches below. The owner selected
the baseline, including admission by protected property, and added caution
about the cumulative cost of new cases
that compile and run WF. These choices are recorded for later implementation;
execution remains deferred. They introduce no WF language rule, elapsed-time
limit or new measurement campaign. The corresponding additions to the pending
[verification amendment](../../../design/amendments/compiler-verification-cost.md)
propose tree wording; selection of these principles is not a ruling on that
exact wording or a live-tree change.

### Responsibilities

Classify an assertion by the property it protects and the source of its
expected result. `#[test]` is a Rust execution mechanism, not a guarantee
that a case is a small unit test or belongs in the compiler library.

| Group | Intended contents and expectation | Inputs, location and execution |
|---|---|---|
| Compiler implementation tests | A named implementation obligation: token/AST structure, proof-state transition or derivation record, lowering choice, calling convention or compiler diagnostic contract. The assertion must observe something beyond an already-covered public language verdict or program result. | Prefer Rust fixtures and small WF inputs beside their owning implementation under `compiler/src/`. An integration module may exercise a compiler obligation across stages; the final target layout remains open. Native construction/execution needs the justification below and must be reported explicitly. |
| `conformance` | Specification-derived acceptance, rejection and runtime requirements. The active specification selects expected results; current compiler output cannot define them. | WF cases and expectations in `tests/conformance/`, with the current adapter in `compiler/tests/conformance/`. Compile each selected case through the ordinary compiler path; link/run only cases whose requirement needs execution. |
| `programs` | Whole-program functionality, host interaction and parallel behavior. Expectations come from the program's stated function, concrete expected outputs or an independent reference, consistently with the language specification. Admission needs a concrete program scenario and observable behavior; this is not a catch-all for WF wrapped in Rust. | WF programs in `tests/programs/` and orchestration in `compiler/tests/programs/`. Arrange real input/files/peers/configuration; compile, link and run as required by the property, then check the result. Compilation success is normally a prerequisite to the behavior assertion, not a separate acceptance case. |
| Canonical batch | Canonical source bytes, parse/render consistency and idempotence, plus the exact normative example. These do not establish semantic acceptance or runtime behavior. | Visit the authoritative WF files already owned by conformance/programs and the specification example; do not duplicate their source collection. Use compiler parsing/rendering functionality. Keep one batch facility within the source-test organization, without a dedicated executable. |
| Snapshot migration | Historical verdicts are leads to investigate, not another correctness authority. Assess each case against the active specification; move useful unique coverage to the appropriate group and retire demonstrated duplicates or obsolete expectations. | Existing `tests/snapshot/` is migration input. Track each disposition while migrating; do not create a permanent second set of historical acceptance expectations or bulk-delete unexamined cases. |

An intentionally invalid syntax fixture need not pass canonical parsing.
Canonical collection must account explicitly for such cases and must not
weaken their negative conformance expectation to make rendering pass.

### Admission by protected property

Identify the main assertion and the contract selecting its expected result
before choosing its home. Neither the fixture language, `#[test]`, native
execution nor the present directory selects the category.

| Admission question | Primary home | Boundary and example |
|---|---|---|
| Does a focused case verify a language rule's acceptance, rejection or runtime requirement? | `conformance` | Give the normative ground and enough source to exercise the rule through the ordinary compiler path. A rule about executing an operation may need a native run; conformance is not limited to compile-only verdicts. |
| Does it verify a useful program's stated function or interaction across components/configurations? | `programs` | Name the scenario and expected output/effect, such as matching the right lines across input files or preserving an independently checked result across worker counts. Merely accepting a WF fragment, or executing it without a meaningful result assertion, does not meet this criterion. |
| Does it verify an additional compiler implementation obligation that those external results do not establish? | Compiler implementation tests | Name that obligation and observe it directly: retained proof dependencies, an IR lowering choice, deterministic instance symbols, or the compiler's concrete native calling convention. Explain the regression the observation distinguishes. Adding an incidental internal assertion does not justify a duplicate end-to-end case. |

For compiler tests that construct or execute a native program, name the
compiler-specific obligation and why execution at that boundary supplies the
needed evidence. For example, a WF/C argument-and-return round trip can check
the implemented calling convention; IR text alone does not establish that
both compiled sides interoperate. If the actual assertion is only a language
runtime requirement, use conformance; if it is only application behavior,
use programs. The same rule applies to existing cases, not only new ones.
Choose the narrowest test path that adequately exercises the obligation;
do not require a native build merely because the fixture contains WF.

Adding a case that compiles WF or builds/runs its native output needs care:
the repeated construction and execution can accumulate across the suite.
Name the missing coverage before adding another such path. Prefer extending
an existing case or using a focused implementation assertion when it provides
the required evidence; reuse valid immutable construction where appropriate.
Identify any additional compiler pass, native build/run, configuration or
repeat the new case requires, without demanding a new timing experiment or
per-case approval form. Preserve ordinary compiler calls for normative
conformance and native execution when the protected behavior needs it.
This admission discipline does not justify retiring a required check merely
because it is slow; timing optimization remains later work.

These are responsibilities, not a requirement for three separate executable
targets. Existing Rust orchestration may be shared where it fits. A single
case may check related observations, and a whole-program fixture may expose
a useful compiler regression. Keep its primary purpose explicit; move an
independent internal assertion to the implementation group when needed,
reusing the fixture and construction where valid. Do not force a separate
case for every assertion or replace a meaningful regression with an arbitrary
smaller input.

During migration, both directions need inspection. For example,
`compiler/tests/programs/generics.rs` currently checks generated-symbol
determinism as well as execution, and `compiler/tests/programs/parallel.rs`
checks permission-ledger details as well as program results. Their directory
does not settle which assertions belong together; assess the individual
properties before moving or merging them. Likewise, an implementation
`#[test]` that only checks application output should be considered for programs.
An assertion outside these core responsibilities, such as a direct C runtime
probe, belongs in the remaining-check review rather than being silently
assigned to programs.

### Research boundary and extraction

The owner clarified that nothing under `research/` belongs in daily CI or
`make check`. This applies to every research subdirectory, active or historical,
and to indirect dependencies: a Rust test importing a research WF source,
LLVM adapter or C oracle is still dependent on research. Calling the current
compiler or printing timing data does not admit an experiment to the gate.

The extraction audit follows current automatic CI and `make check` callers,
including their transitive inputs. Do not inspect research test cases that
neither entry point enables. Historical targets and manual-only experiments
are outside this cleanup; their presence in the inventory is not work to do.
For enabled research, identify useful observations and their formal receivers,
then remove every research dependency from those daily entry points. An enabled
experiment with no useful missing observation needs no replacement test.

Review that enabled content only for observations worth extracting. Give each
retained case a formal compiler/runtime, programs, conformance or performance
test home, with the minimal inputs, oracle and support it actually needs.
Research may then reuse those formal fixtures. Do not move an entire bundle,
keep a research dependency behind a new wrapper, or duplicate an existing case
merely to retain its old gate entry. Unneeded research stays outside daily
verification; this cleanup does not require repairing or refactoring its models,
reporters or measurement tools. Useful dated evidence is retained in place.

This supersedes the earlier exception for current research witnesses and
independent oracles. A useful oracle remains in daily verification only after
it and its required inputs have formal test ownership. The existing paired
performance-regression obligation remains, but its daily implementation and
inputs must also be extracted; renaming the workflow is insufficient.
Explicitly requested research jobs may continue to use research inputs.

The relevant standing wording has been corrected in both agent entry files,
the research index and engineering practice. Completion-review T4-T6 now check
case admission/home, necessary construction/execution, and direct/indirect
research dependencies. The fourth decision in the pending verification
amendment is revised accordingly; the other five decisions and the build-input
amendment are unchanged. No live-tree revision or spec change is made.

Actual Make/Cargo/CI migration remains part of the deferred implementation.
Current invocation tables describe the pre-migration code, not exceptions to
this boundary. Before retiring each caller, account for useful observations
in the formal suite; experimental self-checks need no replacement merely to
preserve their count. This discussion does not claim the current gate has
already stopped reaching research.

### What earns a test its place

- Be able to name the property, concrete input, expected observation and kind
  of defect the assertion distinguishes. Use an existing test name, fixture,
  manifest or short comment; do not add a mandatory metadata form to every case.
  Before adding it, identify its primary home under the admission rules and
  what useful observation is missing from existing coverage. Extend an existing
  case when that adequately covers the regression; a new WF file or Rust
  wrapper is not evidence of a new requirement.
- Prefer one primary home for a given assertion. The same WF file may be used
  to observe proof state, runtime behavior and canonical rendering when those
  are different properties; a shared filename alone proves no redundancy.
- Compare the property, input, failure mode, configuration and oracle before
  merging checks. A compiler-private regression may coexist with a public
  conformance case when it checks something the latter cannot observe.
- Cover the relevant valid, invalid and boundary behavior. Repetitions and
  worker/policy/platform axes need a stated fault or observation to exercise;
  neither a large matrix nor an arbitrary repeat count is coverage by itself.
  Required isolation or repetition is not removed merely because it repeats work.
- Do not preserve tests whose only job is keeping an unnecessary manual copy
  synchronized with its automatic source. Remove that copy and its policing
  test together. A parser algorithm still needs correctness evidence after
  grammar-table generation becomes automatic.
- Compiler crashes, unsupported capabilities, timeouts, missing resources and
  build/launch failures must remain distinct from normative source rejection.
  A tracked expected failure records the correct expectation and the current
  defect; it does not redefine success or hide a newly fixed/changed outcome.
- Share fixture preparation and immutable construction where the dependency
  and isolation requirements allow it. Do not share a previously obtained
  semantic verdict in place of a required current compiler call.
- Retire or move a case with its technical reason and receiving coverage, if
  any. An unexplained deletion, ignored case or updated golden file is not a
  deduplication result. Preserve applicable conformance-evidence obligations.

### Execution and reporting

Keep construction separate from assertion execution. Cargo constructs the
compiler and selected Rust test executables, including generated parser data.
The compiler-test group then calls implementation functions; source checks
use parsing, semantic compilation and native construction/execution only as
their respective properties require. A compile/link success is not a claim
that a runtime workload passed.

The normal full verification entry point must explicitly select every retained
check. A retained ordinary case must not depend on a developer remembering a
special ignored-test opt-in outside that entry point. The intended native
conformance run remains part of full verification. Platform-only cases have
a named host job and explicit unsupported-host reporting; absence of a host
is not a passing execution.

Each source-case failure should identify the group, case/source, expected
result, actual result and failing phase. Report construction failures,
assertion failures, skips and tracked defects separately. Count Rust cases
and WF cases as different objects. Collection integrity must make omissions
visible without maintaining redundant hand-copied counts.

Logical categories do not each require a new binary, crate, script, directory
or CI job. Only introduce a separate execution boundary for a concrete need
such as private access, a different host/toolchain or process isolation.
The exact target layout will follow the remaining-check review, not precede it.
Current resource guards remain; new timing optimization is deferred.

## Review of the remaining checks

After accepting R07, the owner changed the discussion unit from individual
checks to batches of related checks. Group by the protected property and
shared resources, explain construction and callers once, and give a separate
row for each materially different observation or disposition. A batch is a
discussion unit, not a requirement to merge its executables. Do not use a
whole-directory label to hide unrelated assertions. Read the sources and
current callers before recommending dispositions.

For each item, show the following in the conversation in Chinese:

1. **Identity and inputs:** name, source path, current caller, fixture form and
   resources. Identify whether it uses Rust `#[test]`, WF, C, Python or shell.
2. **Actual work:** what is constructed, what executable/function runs, what
   it observes and asserts, and what real defect a failure would reveal.
3. **Need:** should this check exist? Separate a current correctness or
   performance-regression check from historical reproduction or exploratory
   measurement. Do not retire required coverage merely because it is slow;
   apply the admission discipline to unnecessary construction and execution.
4. **Home and stage:** the proposed owning group/location and execution phase,
   including any host-specific requirement.
5. **Overlap and recommendation:** identify actual receiving checks when
   proposing a merge; explain distinct coverage that must survive. Recommend
   keep, move/merge, retire, or explicit experiment-only use with reasons.
6. **Owner decision:** accept a ruling on the whole batch or specified rows,
   record exceptions and remaining uncertainty, then present the next batch.
   Do not treat grouping as approval of its recommendations or implement
   changes while the owner's execution deferral remains in force.

Use actual automatic CI and canonical-gate reachability as the traversal
boundary. The remaining enabled research models/oracles, benchmark checks,
performance-regression protocol and repository/tooling checks are in scope;
unwired or manual-only research is not. Previously selected runtime/platform
changes remain selected; only a newly discovered conflict or changed premise
reopens one. Audit individual compiler/corpus cases under the baseline as
needed during their migration, rather than assuming their current
classification is correct.

The owner agreed to R01 through R07, B01 and B02, and selected the common C
runner direction described below. B04a's completed current-consumer audit
and retirement recommendation for B02b's orphaned operations are now agreed.
The owner also agreed to B03a-ah: the scheduler C probes, Rust parallel
lowering, loop splitting, resource exhaustion, derived cleanup and all four
adjacent stack-ledger tests. Both parts of B04 (a-r), covering native adapters,
platform wiring and Windows namespace/compiled programs, are also agreed.
Implementation remains deferred. B05a-g have been screened under the owner's
research-extraction boundary: the models/controls stay outside daily checks,
and the useful WF behavior has a proposed formal receiving home. The C
control's unconditional timing was already identified in B05f; leaving that
control in research needs no further audit of its manual measurement protocol.
The inventory uses the stable batch IDs below. B08 is now excluded from the
remaining review, leaving three open scopes: B05 (partly screened), B06 and
B07. These are discussion scopes, not test counts, new targets or a promise
that each fits one conversation. Individual compiler/corpus migration audits
still apply under the accepted baseline.

| Batch | Related checks and existing homes | Status |
|---|---|---|
| B01 | Completion publication, wake and lifetime: selected `completion/harness.c` functions and `ordinary_values_probe.c::concurrent_half_close_probe` | Agreed; implementation deferred |
| B02 | Remaining completion adapter/bridge file, directory, queue, helper-policy and progress checks in `compiler/src/backend/completion/` | Agreed; B04a's consumer audit and orphaned-operation retirement also agreed; implementation deferred |
| B03 | Scheduler startup, deque, worker/parallel and exhaustion checks in `compiler/src/backend/sched/`, the related Rust backend sampling modules and adjacent stack-ledger tests | All five reviewed parts agreed; implementation deferred |
| B04 | Linux/Windows native adapters, host-specific probes/WF callers, sanitizer selection, cross-build and link/syntax guards in `compiler/Makefile` and `io-hosts.yml` | Both reviewed parts agreed; implementation deferred |
| B05 | Extract useful observations from currently enabled research models, compiler witnesses, oracles and indirect formal-test inputs | Authority/foundation screened; eight remaining entry groups below |
| B06 | Enabled IO/compute benchmark construction and correctness, and automatic paired performance regression | Three remaining responsibility groups below; manual-only timing protocols excluded |
| B07 | Repository/specification checks, formatting/lint/docs, test collection, runner/process-guard and design-tool self-tests | Not yet reviewed, except the already selected spec/grammar simplifications |
| B08 | Historical or explicit experiment/instrument runners outside automatic CI and the default gate | Excluded by the owner's scope clarification; no case review or cleanup required |

**Remaining enabled review.** The following is a source-level caller count at
the documentation base `39d2b6b0`, not a test-case or executable count. No build
or test was run to obtain it. Shared dependencies are reviewed once with all
their callers; the groups do not prescribe separate binaries or review turns.

| Scope | Remaining groups and current entry points |
|---|---|
| B05: eight entry groups | (1) `proof-use-cost`; (2)-(5) container `lifecycle`, `dense`, `costs`, `families`; (6) ripgrep runner self-tests; (7) raw-DEFLATE oracle self-tests; (8) research inputs already imported by formal lowering, semantic slice/loop-permission and backend slice tests. Groups 1-7 are reached from root `research-tests`; group 8 is reached through compiler tests. Authority/foundation were already screened in B05a-g and are excluded from this remaining count. |
| B06: three responsibility groups | (1) IO `programs-check` reached from root `bench-programs`; (2) compute construction/verification and support reached from root `bench-programs` and automatic `compute-bench.yml` pushes; (3) the paired `compute-regression.yml` comparison/verdict, including `verdict-test` reached from root `research-tests`. Review only the timing needed by the actual automatic regression check, not full manual scoreboard or IO timing matrices. B04o already covers the enabled Windows component-source receiver. |
| B07: ten check types from inventory section 10 | Rust formatting, Clippy, rustdoc, process-guard self-tests, remaining repository invariants, released-spec archive immutability, specification-prose checks, design-linter/self-tests, conformance structure/runner self-tests, and test collection/partition checks. The spec scanner and grammar generator already have selected simplifications; do not reopen them as additional undecided items. |

`historical-tool-tests` and manual-only `io-bench.yml` are not reached by the
automatic gate. The manual comparison arms of `compute-bench.yml` are likewise
outside this review; its push-triggered build/verify arm remains in B06. Follow
helpers used by an enabled arm even when the same helper also serves a manual
experiment. Do not open unrelated research cases merely because they share a
directory. These are the remaining discussion items, not a claim that the
already selected migrations or the full redesign have been implemented.

**Selected C runner organization.** The owner agreed to consolidate compatible
C cases into one main runtime test executable, with logical case groups rather
than a binary per category. Scope scripted directory/clock/other hooks to the
case that needs them and forward ordinary host calls elsewhere. Preserve the
small number of justified construction variants: the isolated core/read
publication implementation conflicts with the real bridge's strong symbol,
the uninstrumented default-policy probe needs its actual shipped-call build,
and sanitizer instrumentation changes the artifact. Share compatible native
objects with correct dependencies across those variants.

One executable can still need several fresh processes: helper count and native
engine selection are initialized once per process. Select only the cases that
observe each setting rather than rerunning the entire suite for every setting.
Compatible cases run together under one configuration, through a common entry
point. This is not authorization to merge incompatible hooks, reset live
runtime singletons, remove required host observations or start implementation.

| Item | Check | Recommendation | Owner ruling |
|---|---|---|---|
| R01 | Completion core/read probe and its build/run variants | Keep useful C adapter assertions; retire unsupported repeat/TSan claims and redundant boundary guards; details below | Agreed; implementation deferred |
| R02 | Default-policy file reads in the bridge probe | Keep the real-default concurrent C runtime check, reuse deterministic policy cases and tighten route assertions; details below | Agreed; implementation deferred |
| R03 | TCP lifecycle in the default bridge probe | Merge overlapping bridge lifecycle logic, preserve distinct runtime/host configurations and correct the transfer, endpoint and timeout checks; details below | Agreed; implementation deferred |
| R04 | Text values in the ordinary linked-library probe | Keep focused C encoding/value assertions, strengthen existing buffer observations and stop repeating text for helper settings that it does not use; details below | Agreed; implementation deferred |
| R05 | Ordinary file acquisition, reads, close and factory accounting | Keep focused C library integration assertions for exact credits and result construction, tighten refusal/buffer observations and correct overstated overlap claims; details below | Agreed; implementation deferred |
| R06 | Ordinary directory enumeration and independent cursors | Keep the real-directory C integration case; observe known entries and cursor independence, require progress and bounded completion, and check the actual writable window; details below | Agreed; implementation deferred |
| R07 | Crossed ordinary TCP halves and factory accounting | Keep precise native per-resource credit observations, retarget the existing transfer fixture to the surviving halves and share TCP setup/guards without adding a WF case; details below | Agreed; implementation deferred |

### R01 — Completion core/read probe

**Inputs and construction.** `compiler/src/backend/completion/core_read_probe.c`
is a C executable with two test functions and a `CHECK` macro, not a Rust
`#[test]` wrapper or a WF corpus. `completion-core-read-test` in
`compiler/Makefile` invokes the host C compiler with C11, `-O2 -g`, warnings
as errors and pthread support. It constructs
`$(COMPLETION_TMP)/core-read-probe` from these eight C translation units:

- `compiler/src/backend/sched/{core,prim_host,entry}.c`;
- `compiler/src/backend/completion/{runtime,wait_host,file_adapter,file_posix,core_read_probe}.c`.

The probe supplies a local `wf_completion_record_complete` that publishes
directly to the record; it does not link `bridge.c` or exercise its submit/join
calls. `WF_COMPLETION_PREAD=wf_completion_test_pread` interposes a call
counter which forwards nonempty reads to the real host `pread`. It does not
script fake read results. Each test creates a scratch file containing
`abcdef` and removes it afterward. Resources are a POSIX C toolchain, writable
executable scratch storage, ordinary local file operations and `nm` for the
current extra symbol guard. The scheduler units are linked, but this probe
starts no worker or helper thread: both adapter initializations pass capacity
and helper count zero, and progress runs on the calling thread.

**Actual assertions.** Every adapter read checks submission ownership,
completion/progress and the result attached to the submitted record.

| Test function | Cases and observations |
|---|---|
| `test_positioned_read_result_boundaries` | Six unconditional cases: empty read succeeds without a host call even with invalid descriptor/offset; full read returns `abc`; a four-byte request at offset four returns only `ef` and leaves the remaining destination bytes unchanged; EOF returns zero without changing the destination; invalid descriptor returns `EBADF`; negative offset reaches the host once and reports `EINVAL`, leaving the destination unchanged. A seventh case applies only where `off_t` cannot represent `INT64_MAX`: refusal precedes the host call and leaves the destination unchanged. That branch is not exercised on the supported 64-bit POSIX hosts. |
| `test_independent_reads_complete_in_reverse_order` | Submit reads for offsets zero and five before progressing. The current zero-helper path completes the newer request first. Check that it receives `f`, the first request remains pending with its buffer untouched, then the first receives `a`. This observes completion/result association in an explicit reverse order, not a concurrent execution. |

**Current callers and variants.** Root `make check` reaches the ordinary
probe through `compiler` -> compiler `check` -> `completion-test`.
Compiler `static` also calls `completion-test`, so Linux/macOS gate static
jobs and the Linux `io-hosts` completion job run it. The ordinary target also
compares the source list against a separately copied allowed list, explicitly
forbids `bridge.c`, and checks a bridge symbol with `nm` after execution.
`completion-core-read-sanitize`, reached from the Linux `completion-sanitize`
job, rebuilds the same input with `-O1 -fsanitize=address,undefined` and runs
it once. The Linux IO job also builds/runs `completion-core-read-tsan` with
`-fsanitize=thread`. The explicit, non-gate `completion-core-read-stress`
target runs the ordinary target and then repeats its executable 200 times.
The Makefile's statement that those repeats exercise sleep/publication races
does not describe the current two test functions.

**Overlap.** `harness.c::test_single_thread_file_progress` already checks a
normal adapter positioned read in an open/write/read/status/close sequence;
`harness.c::test_bridge_independent_positioned_reads` checks two requests
through real bridge submit/join calls. These overlap in successful reads but
exercise different boundaries. Neither cited test supplies the probe's
empty-read host-call observation, short-read destination checks and forced
intermediate reverse-completion observation. This comparison does not settle
all other runtime probes. The probe's local completion function and the real
bridge define the same strong symbol, so their link requirements differ.

**Selected disposition.** The owner agreed to the following recommendations.
Implementation remains deferred; this ruling does not change the live tree.

- Keep the useful assertions as C runtime adapter unit tests in the existing
  completion source area, selected by the common runtime-test stage of the
  local full gate and POSIX CI. Keep their isolated link configuration;
  there is no reason to compile WF or add a Rust wrapper for these properties.
  Other C unit cases with compatible link/hooks may share a runner later;
  do not merge this file into the real-bridge harness without addressing its
  different completion hook and retaining the useful observations.
- Retain reverse-completion/result association as the protected property.
  The current LIFO progress behavior arranges the case; it is not a new
  language or permanent adapter ordering requirement. If queue policy changes,
  arrange the needed order deliberately and reassess the fixture.
- Remove this probe's 200-repeat stress target and standalone TSan variant:
  the former has no varied input or concurrent schedule, and the latter has
  no concurrent execution to observe. This recommendation concerns only this
  probe, not the real-thread bridge/default-route/deque tests. Retain its
  memory/undefined-behavior checks in the shared C sanitizer phase, whose
  complete organization will be reviewed later.
- Remove the copied allowed-source list, redundant explicit bridge exclusion
  and `nm` bridge-symbol guard. The explicit minimal link inputs and the
  incompatible strong completion definitions already establish the current
  link separation; duplicated lists and symbol policing add no behavior
  observation. Keep any genuinely required link boundary explicit in the
  eventual common runner. Update stale race comments and callers together.

This is source inspection, not a fresh execution or a savings measurement.
The exact shared runtime target layout and the remaining sanitizer/platform
checks are still to be reviewed.

### R02 — Default-policy file reads

**Scope and inputs.** The file-read portion of
`compiler/src/backend/completion/bridge_default_probe.c` exercises the real
completion bridge with `WF_IO_HELPERS` unset. The probe refuses a set value.
It calls the bridge's C submit/join ABI using opaque aligned record storage,
but compiles no WF and contains no Rust `#[test]`. Current WF code calls
ordinary linked functions; their native definitions in `ordinary_values.c`
use this bridge. The probe does not validate that preceding compiler/call
boundary, despite its older comment about standing in for emitted code.
It also currently runs `probe_loopback_round_trip` after the read phase; TCP
assertions and that portion's resource/timeout handling belong to R03, not
this disposition.

`completion-default-route-test` compiles the POSIX executable with C11,
`-O2 -g`, warnings as errors and pthread support. Its eleven C inputs are
`sched/{core,prim_host,entry}.c` and
`completion/{runtime,wait_host,file_adapter,file_posix,bridge,linux_io_uring,native_contract,bridge_default_probe}.c`,
all under `compiler/src/backend/`. The output is
`$(COMPLETION_TMP)/bridge-default-probe`. It uses the real host calls and
clock, without the main harness's macro-interposed host/clock configuration.
The Windows CI build uses the same probe, Windows scheduler/wait/file leaves,
`windows_iocp.c` and `windows_runtime.c` in its eleven-source link, with
Winsock. Neither build constructs the compiler.

**Work and assertions.** The read phase creates a 4,096-byte file whose byte
at offset `n` is `n % 251`. Four real runtime primitive threads each perform
4,000 submit/join rounds, with a changing offset and one byte per positioned
read: 16,000 such requests in a successful invocation. Each lane additionally
submits one non-positioned read every 64 rounds, including round zero, for
252 additional requests. Those requests disturb the queued-versus-inline
precondition; they are not 252 distinct language cases. A separate watchdog
thread fails a stuck read phase after 180 seconds. That is a failure bound,
not a measurement or a reason for the round count.

| Observation | Current assertion and limit |
|---|---|
| Positioned read results | Every join must return one byte, no error and the independently computed byte for its requested offset. A lane failure fails the process. Concurrent publication, result association and progress are exercised through the real bridge. |
| Non-positioned reads | Each response is either nonnegative/no-error or negative/with-error; the invocation must not mix these two categories. This watches completion and route consistency, not full stream-read semantics: it checks neither returned bytes nor a specific refusal code. |
| Native versus adapter | Read-phase ring, adapter, inline and helper counts are captured before TCP. Explicit `WF_IO_NO_NATIVE_RING=1` must leave the ring submission count zero. A normal local invocation may legitimately use an available ring or fall back. |
| Adaptive policy | With no ring submissions, at least one operation must have run inline or at least one helper must have started. This requires an observed policy branch, not both branches or all transitions on every host. |
| Native helper policy | The helper count is reported, but a native-ring run does not currently assert that it stays zero, as `bridge.c::wf_bridge_helper_policy` and the live completion-runtime decision require when helpers are unset. This is a missing observation, not evidence of an implementation failure. |

The non-positioned branch still describes a target that may have no stream
read, although the current Windows leaf implements `WF_FILE_READ`. Its fixture
opens an overlapped Windows handle for positioned reads and also passes it to
the stream-read path, whose `file_windows.c` implementation uses a null
`OVERLAPPED` for synchronous stream handles. The current accept-either-result
check does not establish the right result for that fixture. No fresh Windows
execution or particular observed error is claimed here.

These are repeated native-thread operations exercising current correctness;
there is no throughput/latency regression verdict. The sources inspected do
not derive 4,000 as a minimum useful round count. This review neither declares
the repetitions redundant nor chooses an arbitrary smaller count.

**Callers and resources.** Root `make check`, compiler `static` and the
Linux/macOS static gate reach this through `completion-test`. The ordinary
Linux target runs the executable once under default routing and once with
the ring explicitly disabled; macOS runs it once. Linux IO CI calls that
same target, then rebuilds/runs ASan+UBSan and TSan variants with `-O1`.
Those sanitizer targets currently do not explicitly repeat the forced-adapter
configuration. Real Windows CI compiles the native executable and runs both
IOCP and forced-adapter modes, checking the reported route. Explicit Windows
cross-build/Wine targets are development paths, not substitutes for that host.

The file portion needs writable executable scratch storage, ordinary file
access, native threads and the host C runtime; native-engine coverage needs
the corresponding host facility. The current combined executable additionally
needs loopback sockets because TCP is still attached. Startup settings are
process-wide and initialized once, so different route configurations require
fresh processes, though they can reuse the same executable.

**Selected disposition and overlap.** The owner agreed to the following
recommendations; implementation remains deferred.

- Keep this as a C runtime integration check under the existing completion
  source owner, called from the common runtime verification stage and the
  appropriate native host jobs. It validates the real default selection and
  concurrent bridge path without paying for WF compilation. R01 bypasses
  the bridge and starts no threads, so it cannot replace this check.
- Reuse the main harness's existing deterministic policy cases:
  `test_pool_stays_empty_when_operations_do_not_wait`,
  `test_pool_grows_when_operations_wait` and
  `test_helper_growth_stops_at_the_declared_bound`. They provide controlled
  short/long wait and cap evidence; the real-clock probe should not claim that
  every branch happened or reproduce those cases with another driver. The
  main harness pins the bridge's helper setting, so its ordinary bridge tests
  do not replace the unset-policy observation.
- Add the missing zero-helper assertion to the existing native-ring read
  run. Keep forced-adapter routing verifiable, and require native execution
  when a host job claims native-route coverage rather than accepting fallback
  under that label. Reuse the observed counters; no new WF case is needed.
- Give the non-positioned queue-disturbance requests an input with a clear
  host contract, such as a suitable synchronous stream handle, and check its
  defined result. Do not retain success-or-any-error as an oracle or use the
  same handle for incompatible fixture purposes merely to share setup. Keep
  the queue-disturbance purpose explicit and update the stale capability
  comments; this need not add a WF program or another test executable.
- Retain meaningful concurrent sampling and TSan as well as ASan/UBSan.
  Unlike R01, real threads contend here. Make sanitizer route scope explicit:
  a native-only run cannot stand for default adapter growth, and even a
  forced-adapter run does not guarantee growth under the real clock.
  Deterministic growth cases own that guarantee. Any later reduction in the
  sample needs preserved relevant race coverage and a stated ground; no
  timing or loop-count change is selected now.
- Give file reads and TCP distinct selection/reporting in the common C test
  organization; this need not introduce another executable. Preserve the
  real-default link/clock configuration and fresh-process startup boundary
  when considering a shared runner, instead of putting this behind the
  main harness's unconditional helper pinning. TCP's own disposition is R03.

No fresh construction, execution, timing result or retirement accompanies
this source-based recommendation.

### R03 — TCP lifecycle in the default bridge probe

**Identity, construction and resources.** The function
`probe_loopback_round_trip` in
`compiler/src/backend/completion/bridge_default_probe.c` is C code called by
that probe's `main` after the file-read phase. It shares R02's eleven-source
runtime link and its `bridge-default-probe` output, including the real bridge
and host leaves; it has no separate build, WF source, compiler invocation or
Rust `#[test]`. Windows uses the same function with its native link leaves and
Winsock. It needs a permitted IPv4 loopback listener and two connection ends
in the same process, not an Internet service or another WF program. The test
driver submits and joins each step sequentially; runtime helpers may still
execute work. The combined executable currently also requires R02's file
fixture and threads.

**Actual work.** This is a connection lifecycle with an eight-byte transfer
from client to accepted connection, not an echo or bidirectional payload
exchange. In the ordinary successful path it makes ten bridge submissions:

| Phase | Current action and assertion |
|---|---|
| Listen | Try IPv4 loopback ports starting at 45,231, up to 64 candidates. Any failed listen moves to the next candidate, not just address-in-use. Keep the first nonnegative descriptor. |
| Connect and accept | Connect to that listener, accept the pending connection, and require nonnegative descriptors. Check the accepted peer's address words equal IPv4 loopback and its low port bits are nonzero. Unlike the main harness, this function does not separately check the family flag. |
| Transfer | Send `{3,1,4,1,5,9,2,6}` in one call, require eight bytes, then receive in one call and require all eight bytes with an exact content match. |
| Close | Close each connection's two directions, four operations total, then close the listener. The helper accepts any nonnegative joined value and does not assert a zero error field or the first/last release values. It does not observe handle lifetime. |

The TCP ring-submission delta is printed separately, but is not asserted.
The final `route=native-ring` label and the Windows caller's check of that
label come from the earlier file reads, so they do not establish that a
particular TCP operation used the native engine. An allowed immediate socket
transfer also need not submit to a ring; that fast path is not a failure.
There is no timing or throughput verdict.

**Current invocation and effectiveness.** The function runs whenever R02's
combined executable runs: local `completion-default-route-test` through the
full gate, Linux/macOS gate jobs, Linux IO default/forced-adapter runs and
sanitizer variants, Windows IOCP/forced-adapter runs, and explicit cross/Wine
development paths. It runs once per invocation, not once per file-read round.
The file phase stores `probe_finished = 1` before this function is called.
Consequently its 180-second watchdog no longer covers TCP; outer guarded
commands or CI job limits may eventually stop a hang, but a direct executable
or the direct Make target has no active TCP-specific deadline here. Several
early-return failures also bypass socket cleanup.

**Existing receiving coverage.**

| Existing check | Relationship to this function |
|---|---|
| `completion/harness.c::test_socket_lifecycle_and_the_pair_two_count` | The same bridge listen/connect/accept/transfer/close sequence, under pinned helper configurations. It additionally checks IPv4 family, zero-length receive results, first-close value zero with the descriptor still open, last-close value one with the descriptor closed, both direction orders, and a refused connection after the listener is closed. This is the primary receiving case for the duplicated lifecycle logic. Its single-call transfer assumption also needs correction. |
| `completion/native_adapter_probe.c::probe_loopback_round_trip` | Submits records directly to the native adapter, bypassing the bridge. It protects the engine's request/publication path and cannot be replaced merely by a bridge exchange; its individual disposition remains later work. |
| `ordinary_values_probe.c::tcp_probe` and concurrent half-close probes | Exercise ordinary linked values, crossed receive/send components, factory-credit accounting and close races. Their additional representation/lifetime properties are not supplied by a basic bridge lifecycle. |
| `compiler/tests/programs/network.rs` with `tests/programs/tcp_echo.wf` and related programs | Checks compiled WF programs interacting with Rust peers, including payload echo, end-of-stream and reset outcomes. It includes compiler and ordinary-call integration that this C-only test does not establish. Do not retire it just because both use TCP. |

**Selected disposition.** The owner agreed to the following recommendations;
implementation remains deferred.

- Retain one maintained C bridge lifecycle case by combining the common logic
  with `test_socket_lifecycle_and_the_pair_two_count`. Preserve the stronger
  existing observations and expose the same case to default-policy and pinned
  configurations, native and adapter modes, and the supported host jobs.
  Use small host helpers for endpoint and handle observations. Share code
  without forcing distinct macro/link configurations into one build or
  silently dropping Windows/default-policy coverage. Logical selection and
  reporting need not add a binary, Rust wrapper or WF case.
- Replace fixed-port scanning with a port-zero listener and a host-side
  query of the bound address, retaining the listener until the connection is
  made. The main C harness already does this; `ordinary_values_probe.c` has a
  Windows/POSIX `listener_port` helper. This is test fixture setup, so the
  absence of a WF port-query function is not a restriction and warrants no
  language addition. Adapt/reuse those host operations instead of keeping
  another retry policy that can obscure unrelated listen failures.
- Allow valid short transfers: advance by the actual successful count,
  retain the original buffers and check the complete received payload when
  assembled. The active `PRE-1` send/receive contracts do not promise that
  one call reaches the requested end, and the runtime uses ordinary
  `send`/`recv` or equivalent native operations. Require consistent success
  and error fields and preserve the bridge's distinct first/last close
  values from `bridge.h`; do not substitute a blanket value-zero assertion
  for a last close that legitimately returns one. Unexpected EOF, errors or
  lack of progress must fail visibly rather than spin or be relabeled success.
- Cover the entire selected TCP case with the common C test timeout and
  failure cleanup, keeping the current guard active through it when invoked
  in the combined process. Failure reports should identify the socket phase.
  The self-contained peer sequence does not make blocking or implementation
  bugs impossible. Reuse existing guard machinery, with no new timing campaign
  or arbitrary deadline choice in this review.
- Keep route assertions about the TCP work they name. A file-read route label
  is not TCP engine evidence, and permitted immediate completion is not
  evidence of failed routing. Reuse the existing native-adapter tests for
  direct engine coverage and preserve any distinct bridge-routing observation
  when consolidating callers; do not simply count printed counters as tests.

The related C value, native-adapter and WF program cases retain their own
review scope. This recommendation selects neither their wholesale deletion
nor a new test-target layout. No implementation, execution or new measurement
has accompanied this review.

### R04 — Text values in the ordinary linked-library probe

**Identity and construction.** `text_probe` in
`compiler/src/backend/ordinary_values_probe.c` directly calls the C ordinary
library in `ordinary_values.c`, using native `wf_value` and `wf_view` records.
It uses C `assert`, explicitly enabled even if `NDEBUG` was defined. It is
neither a Rust `#[test]` nor a WF program. View copies enter the private
pointer-parameter C body; they do not validate the compiler's emitted LLVM
calling convention.

The current POSIX `ordinary-values-test` Make target builds one
`$(COMPLETION_TMP)/ordinary-values-probe` with C11, `-O2 -g`, warnings as
errors and pthread support, from eleven C translation units under
`compiler/src/backend/`:

- `sched/{core,prim_host,entry}.c`;
- `completion/{runtime,wait_host,file_adapter,file_posix,bridge,linux_io_uring}.c`;
- `ordinary_values.c` and `ordinary_values_probe.c`.

The whole executable also tests files/directories, TCP values and concurrent
half-closes. Its shutdown-observer macro is needed by that last group, not
by text conversion. The Windows IO job links its ten `WINDOWS_UNITS` plus
the two ordinary-value sources, producing
`windows-ordinary-values-probe.exe` with Winsock and shell32. These are C
builds; neither constructs the WF compiler or a WF program.

**Inputs and actual assertions.** The function constructs four arguments in
memory: `abc`, U+1F600, an invalid encoding and an empty string. POSIX uses
byte strings; its invalid bytes are `ed a0 80`, an encoded surrogate which
UTF-8 must reject. Windows uses UTF-16 code units: `d83d de00` for the valid
pair and a lone `d800` for the invalid input. A sixteen-byte stack array is
the copy destination.

| Operation | Current observation and limit |
|---|---|
| Argument access | Require count four, successful index zero and refusal at index four. The intermediate `arg_get` calls are not separately checked before using their returned values. |
| Too-small UTF-8 copy | Copy `abc` into `[2,3)`. Require an error, required length three and destination byte two still equal to its sentinel. It does not check the error's variant tag or the other fifteen destination bytes. |
| Successful UTF-8 copy | Copy `abc` into `[2,5)`. Require success, returned endpoint five and those three bytes equal to `abc`. This distinguishes an absolute endpoint from length three; bytes outside the copy window are not asserted. |
| Non-ASCII encoding | Require U+1F600 to have UTF-8 length four. It never copies that value, so it does not observe the four encoded bytes or exercise the Windows four-byte encoder stores. |
| Invalid encoding | Require `Utf8CopyInvalid` from copying the invalid argument into `[0,16)`. It does not assert that the destination remains unchanged. |
| Empty relative path | Convert the empty host string to a relative path and require success with length zero, reusing the value/result storage. This is not general path validation or filesystem access. |

**Resources, callers and repeated work.** These particular functions only
inspect or copy memory; the Windows UTF-16 measurement/encoding helpers also
perform no host IO. They do not consult the completion bridge or
`WF_IO_HELPERS`. The current combined `main`, however, requires a scratch
directory and changes into it, initializes the close observer's wait object,
then calls `text_probe` before creating ordinary inputs and running the other
groups. Those startup requirements belong to the combined driver, not to
the text assertions.

The Make target runs the same executable once with `WF_IO_HELPERS=0` and once
with `WF_IO_HELPERS=2`. It is reached by `completion-test`, compiler `check`
and `static`, root `make check`, Linux/macOS static gate jobs and the Linux
IO completion job. Windows IO CI repeats the same two helper settings. For
the text group alone, both runs use identical inputs, implementation paths
and observations; the setting supplies no additional coverage. The host
distinction does supply different encoding implementations. The current
ordinary-values target has no dedicated sanitizer variant; this description
does not count the completion harness's sanitizer runs as text coverage.

**Overlap and ownership.** Existing conformance cases already exercise
ordinary argument access, invalid UTF-8 length, invalid-copy refusal and
too-small-copy refusal through compiled WF. In particular,
`run-syshost-copyutf8-invalid-unchanged` and
`run-syshost-copyutf8-toosmall-unchanged` check the exact error variant and the
entire destination. The former arranges `61 ff 62`, a different validator
failure from this C fixture's encoded surrogate. The case named
`run-syshost-nontext-argv-utf8-invalid` actually calls `host_utf8_len`, despite
its current copy-oriented description. `v033-run-system-nonzero-next` checks
a nonzero endpoint for `host_copy_bytes`, not the UTF-8-copy C body here.
This comparison supports a focused native encoding/representation check,
not another complete copy of the normative corpus.

`backend/tests/system.rs::run_arguments` also reads conformance WF files,
compiles and runs them, and checks success with empty stdout/stderr. Several
argument/text tests use that wrapper without additional internal assertions.
Flag them for the agreed compiler/corpus audit; this R04 ruling does not
retire those separate Rust cases. Other tests in that module compare ABI
plans or emitted calls, and `system_io.rs` runs a larger ordinary IO chain.
Neither additional compiler observation nor real-program composition is
established by this direct C function.

**Selected disposition.** The owner agreed to the following recommendations;
implementation remains deferred.

- Keep a focused ordinary-library C unit group in the existing backend test
  source area, protecting native text representation, encoding branches and
  destination boundaries. Run it in the common runtime unit-test phase of
  the local full gate and supported host CI. Normative WF outcomes remain in
  conformance; explicit compiler ABI assertions retain their separate purpose.
- Select text once per host/build configuration that changes its code or
  observations. Stop repeating it merely for helper counts zero and two,
  and let text selection run without the other groups' directory, ordinary
  inputs or close-observer startup. Reuse the C test construction/runner;
  neither a new executable, a production-library split nor a WF wrapper is
  justified just to obtain this logical selection. The IO/concurrency
  groups' helper requirements are not decided by this item.
- Strengthen the current small fixtures: assert the argument and copy error
  tags, compare the whole destination after each refusal, check bytes outside
  a successful copy window and actually copy U+1F600 to its known four UTF-8
  bytes. Keep the nonzero endpoint observation. These check concrete encoding
  and write-boundary defects without new WF compilation, a large encoding
  matrix or repeated process runs. Keep the related argument/empty-value
  assertions together; no separate test executable per assertion is needed.
- Preserve POSIX byte-input and Windows UTF-16 coverage. Helper variation and
  TSan would not add a concurrency observation to these memory-only calls;
  any shared memory/undefined-behavior sanitizer phase may run them under
  that distinct instrumentation. Its overall organization remains later work.

No source, corpus verdict, caller or build recipe has changed. This is source
inspection with no fresh execution, timing or savings claim. The remaining
groups in `ordinary_values_probe.c` will be considered separately.

### R05 — Ordinary file operations and factory accounting

**Scope and construction.** This item covers the first part of
`compiler/src/backend/ordinary_values_probe.c::file_probe`, through closing
the read file into a different factory, and the purpose of the final credit
transfer back. Directory enumeration between those portions is a separate
item: sharing one C function does not make cursor behavior the same property
as file-result construction or credit accounting.

The code directly calls `ordinary_values.c` using C values and `assert`.
Its private pointer-parameter view bodies do not check emitted LLVM ABI.
It shares R04's executable, eleven-source POSIX or twelve-source Windows C
construction, `-O2 -g` flags and local/CI callers. There is no separate file
test executable, Rust `#[test]`, WF source or WF compiler invocation.

**Fixture and work.** C stdio creates `ordinary-values.data` containing five
bytes, `hello`, in the scratch working directory. `fopen`/`fwrite`/`fclose`
arrange the input; they are not tests of the WF write library. The name view
contains POSIX component bytes or Windows UTF-16 component bytes. File reads
reuse a 4,096-byte buffer needed by the later directory batch, but their
nonempty request is only seven bytes, not 4 KiB. The file sequence makes
four ordinary open attempts, three read calls and two closes, with no stress
loop. One open is rejected by zero factory capacity before submitting work;
the other eight calls submit bridge requests on the intended path. These
counts are library calls/requests, not a count of host system calls.

| Step | Current assertion and its purpose |
|---|---|
| Failure with one credit | Start a native factory at one. Try to open the regular file as a directory. Require failure other than error tag 21 and credit still one. The following successful file open consumes that credit, and closing restores it. This checks failure/success/close accounting on a deliberately constrained factory. |
| Refusal with zero credits | Temporarily set the input factory's credit word to zero. A file open must fail with tag 21 (`ResourceExhausted`) and leave zero. The test does not check that error's code/origin fields. Restore the saved capacity afterward. |
| Successful acquisition | Open the file from the input factory. Require success and exactly one fewer credit. |
| Short read and absolute endpoint | From file offset zero, read into destination `[2,9)`. Require `Ok(7)`, bytes `[2,7)` equal to `hello`, and bytes one and seven still equal to the sentinel. The returned value is destination endpoint seven, not byte count five. |
| End of file | Read at file offset five into the same nonempty range. Require `ReadEnd` and compare the entire buffer with its saved value. |
| Empty destination range | Read into `[9,9)`. Require `Ok(9)`, distinguishing an empty successful transfer from nonempty EOF even though both bridge results have amount zero. The buffer is not compared again after this call. |
| Close into another factory | Close the acquired file into an initially empty receiving factory. Require the acquiring factory to remain one credit down and the receiving factory to become one. The later directory-source open spends that received credit, and its close into the original factory restores the original total. This is a real owner transfer, not a direct addition to balance the fixture. |

The native counter is `wf_value.words[0]`, not a WF-visible field. The
current [resource-exhaustion-floor decision](../../../design/compiler/resource-exhaustion-floor.md)
places acquisition/close accounting in the ordinary exclusively borrowed
factory. `wf_open` returns a taken credit on acquisition failure; `wf_close`
returns it to its supplied factory. These are concrete implementation
properties that cannot be inferred merely from successful high-capacity
opens. This probe does not examine close-error consumption, all host error
classes, descriptor leakage after every refusal, or arbitrary factory states.

**Weak observations.** The first refusal accepts any class other than
`ResourceExhausted`; an unrelated path-validation failure would satisfy that
assertion without establishing the intended host/kind refusal. Tightening it
must respect the actual host path: the POSIX component operation supplies
`O_DIRECTORY`, whereas the Windows leaf can open and then reject a nonmatching
kind as `WF_WINDOWS_OPEN_OTHER_KIND`, which the ordinary library maps to
`Unsupported`. Do not impose a single guessed host error on both. No fresh
host execution or newly observed runtime failure is claimed here.

The successful read only observes two guard bytes outside its returned range.
Its final empty read does not check that the buffer stayed unchanged. These
are gaps in observation, not proof that the implementation currently corrupts
memory. The driver is sequential; runtime helper threads may execute work.
It needs ordinary local files, an opened directory value and the linked
completion runtime. The full executable additionally needs the later groups'
network and close-race facilities.

**Overlap and actual coverage.** R01 checks low-level adapter amounts,
errors and destination bytes; it neither constructs ordinary `ReadStop`
variants/absolute endpoints nor owns a `HandleFactory`. The completion
harness checks bridge open/status/close results and rejected descriptor
cleanup, likewise without ordinary factory accounting. Keep those distinct
observations; a full copy of their host-operation matrix is not needed here.

Conformance's `run-sysfile-{empty,short,exact,multichunk}` cases and the
`backend/tests/system_io.rs` read tests cover compiled WF behavior. The latter
include nonzero destination bounds, short reads and zero-length reads. Their
normative observations belong in the agreed corpus audit; a C-only call does
not replace the WF call boundary.

Two particularly overstated claims are
`run-sysfile-close-returns-permit` and
`run-sysfile-failed-open-returns-permit`. Their WF source uses the ordinary
startup factory, does not constrain it to one credit and only checks that a
subsequent open/read succeeds. It never compares the read byte despite the
first case's description claiming the original byte. Therefore a missing
single-credit return need not make either case fail. They also have Rust
`#[test]` wrappers in `system_io.rs` which compile the same corpus source and
check successful process exit with equivalent small fixtures; those wrappers
add no exact-credit observation. Record this limitation for their later
case-by-case migration/retirement. This item does not change their verdicts
or retire either the WF cases or their Rust wrappers.

**Selected disposition.** The owner agreed to the following recommendations;
implementation remains deferred.

- Keep a focused C ordinary-library integration group for exact factory
  accounting and bridge-to-library result construction, in the existing
  backend test area and common runtime verification stage of the local full
  gate and supported host CI. Reuse the current C construction and small file
  fixture. Give file operations and directory enumeration distinct selection
  and failure reporting without requiring separate executables or WF cases.
- Retain the zero/one-credit setup, successful decrement, failure restoration
  and cross-factory close/reuse observations. They exercise the actual state
  without exhausting the host's descriptor table. Check the quota refusal's
  library code/origin as well as its class, and replace the wrong-kind
  `tag != 21` oracle with the intended, source-grounded host outcome. Protect
  final accounting rather than requiring a particular temporary decrement
  order inside acquisition.
- Retain short-read endpoint construction and the distinction between EOF
  and an empty successful transfer. Compare all bytes outside the successful
  returned range and the whole buffer after the empty call, reusing the
  existing input and observations. Do not duplicate R01's entire adapter
  matrix or add a WF program for internal counters.
- Unlike R04's text calls, these operations reach the bridge. Written helper
  counts pin adapter policy; zero versus two can change who executes queued
  adapter work. A native engine may instead take the operation in both runs,
  and this probe currently has no route assertions or forced-adapter run.
  Preserve demonstrably distinct runtime execution coverage in the common C
  configuration organization; do not label helper zero/two as native/adapter
  evidence or multiply every assertion across an unexamined matrix. No
  deletion of this group's helper variants is selected by this item; complete
  configuration/sanitizer organization remains a later review.
- Correct the corpus descriptions' unsupported accounting/payload claims
  during the agreed corpus audit. Keep or merge WF cases for the language
  behavior they actually distinguish, and retire redundant Rust wrappers only
  after preserving any distinct assertion or construction mode. Do not add
  WF-visible counter APIs, huge exhaustion loops or extra WF compilations
  merely to turn this C implementation observation into a language test.

The directory-source iteration loop and independent cursors will be reviewed
next. No implementation, test retirement, specification revision, execution
or timing measurement accompanies this record.

### R06 — Directory enumeration and independent cursors

**Scope and construction.** The directory portion of
`compiler/src/backend/ordinary_values_probe.c::file_probe` directly calls
`wf_open_directory_source`, `wf__body_directory_next` and
`wf_close_directory_source`. It uses C `assert`, the same ordinary-values
executable and eleven-source POSIX/twelve-source Windows C build as R04/R05,
and the same local full-gate and host-CI callers. No Rust `#[test]`, WF
compiler construction, WF compilation or additional executable belongs to
this portion. The native pointer-parameter body does not validate the
compiler's emitted three-result/view calling convention.

It enumerates the real scratch working directory while R05's five-byte
`ordinary-values.data` fixture still exists. The Make/Windows callers create
that directory with `mkdir -p`; the probe neither creates a private fresh
directory per item nor verifies all of its contents. It reuses a 4,096-byte
stack buffer and opens two directory sources before reading either. The
final third open/close transfers R05's received credit back; it is not a third
cursor-independence case.

**Actual work and assertions.**

| Step | Current observation |
|---|---|
| Open cursors A and B | Both opens of the same directory succeed. POSIX reopens `.` relative to the supplied directory; Windows reopens it using the empty relative native name. The implementation deliberately opens a new enumeration cursor rather than duplicating one shared cursor. |
| Empty range on A | Call with `[11,11)`. Require success, `next == 11` and zero entries. The buffer is not compared, nor is the exact unconsumed entry set observed. |
| First nonempty batch on A | Call with `[3,4096)`. Require success, `next > 3` and at least one entry. No filename, record bytes or relation between decoded records and the entry count is checked. This first call also lacks the upper-end assertion used in the loop. |
| Drain A | A `do/while` repeatedly calls with `[3,4096)`, checking only that each returned endpoint lies within that range, until status is no longer success. Intermediate successful calls need not report positive bytes/entries or new records to pass the current assertions. |
| End of A | Require `ListEnd`, `next == 3`, zero entries and the entire buffer unchanged from just before the final call. A `ListFailed` result fails rather than being treated as normal end. |
| First batch on B | After A is exhausted, require B's first read to succeed with positive endpoint/entry count. This catches a shared EOF cursor, but B is neither decoded nor drained and the known fixture name is never required. |
| Close | Both directory-source closes succeed. Aggregate credit restoration is observed later by R05's final transfer. |

There are at least four `directory_next` calls: one empty call, A's first
batch, at least one drain/terminal call and B's first batch. Additional drain
calls depend on the directory contents and host batching; this is not a fixed
stress-repeat count. No per-item deadline or iteration bound appears in the
probe. A repeated successful empty result, or repeated positive batch with a
stuck cursor, can keep the loop running. Outer guarded commands/CI limits may
eventually terminate it; this inspection does not attribute the earlier long
machine stall to this function.

**What executes underneath.** Linux uses `getdents64`; macOS uses
`__getdirentries64`; the Windows host leaf obtains a native directory batch
and converts its UTF-16 records to the intermediate representation consumed
by `ordinary_values.c`. The current `wf_linux_io_uring_carries` and
`wf_windows_iocp_carries` do not carry directory-enumeration requests. Nonempty
enumeration therefore reaches the blocking adapter even when a native engine
is present; the empty request completes in the bridge without a host read.
Helper zero/two can change adapter execution, not turn enumeration into an
io_uring or IOCP operation. Open/close requests have their own routing.

The ordinary body compacts native records in place into a kind byte, a
little-endian two-byte name length and the name bytes. POSIX name bytes and
Windows UTF-16 name bytes differ. It returns the compacted endpoint and
entry count. A host read may already have written the larger native batch
past that compacted endpoint within the supplied window. The active `PRE-1`
declaration states endpoint bounds; it does not promise that `[next,end)`
remains untouched. Any success guard must protect outside `[start,end)`,
not introduce that stronger tail guarantee. This representation is a native
library implementation observation, not a new source-language rule.

**Overlap and limits of existing evidence.**

| Existing check | Relationship |
|---|---|
| `completion/harness.c::test_directory_progress_is_internal` | Scripts interruption, readiness refusal and eventual progress, checking three host attempts, one poll and the platform position-cell behavior. It does not enumerate a real directory or compare two cursors. Preserve that separate retry/progress boundary. |
| `backend/tests/enumeration_records.rs` | Five Rust tests feed hand-built native records through a substituted host facility, run a WF publisher and check decoded bytes or process termination for contradictory records. They cover selected name lengths, kind mapping, a full batch, empty input and malformed layout. Real files cannot arrange those malformed native records. Its future construction/ownership review must retain the useful decoder observations; this real-directory item is not their replacement. Its emitted-shim wording is stale because the current decoder lives in the ordinary C library. |
| Conformance `sys14-list-zero-range` | Checks one empty call and immediately closes the cursor. Despite its source/manifest description, it does not make a later nonempty read, so it does not establish that enumeration was preserved. |
| Conformance `sys14-entry-kind-closed` | Checks kind range, name-length structure and announced record count for a returned batch, but does not require the fixture's names; an immediate `ListEnd` with zero records can pass. It cannot supply the missing real-entry observation here. |
| `compiler/tests/programs/traversal.rs` and `tests/programs/dir_walk.wf` | Compare a real recursive tree walk's sorted kind/path output, plus empty-tree and refused-descent behavior. This protects compiled program composition and real record use, not specifically draining two sources opened on the same directory. The module also contains compile-only ownership/type/exhaustiveness rejections and emitted-call assertions; their homes must follow the already selected contract during the corpus audit. |

**Selected disposition.** The owner agreed to the following recommendations;
implementation remains deferred.

- Keep one small C ordinary-library integration case for real enumeration,
  independent cursor state and result/buffer observations. Put it in the
  common runtime verification stage of the local full gate and supported host
  CI, sharing R05's C construction and controlled fixture while reporting
  directory failures separately. No additional WF program, executable or
  recursive tree fixture is needed.
- Use a known, unchanging test-owned directory. After the empty call on A,
  drain both independently opened cursors and require the known fixture name
  in each. Compare the complete entry collections without depending on host
  order or batch boundaries; account for whatever self/parent entries that
  host supplies rather than hardcoding a portable count. Decode enough of
  each returned batch to verify record bounds, exact name bytes and the
  reported count. This supports the cursor assertion without duplicating the
  scripted decoder's full length/kind/malformed-input matrix.
- Require each successful nonempty call to report a valid progressing batch
  and reject duplicate/unexpected entries in the fixed fixture. Bound the
  walk by that fixture's entries and the terminal observation, not an
  arbitrary stress count. Use the common test-process deadline for a blocked
  host/join, with phase-specific failure reporting and fixture cleanup. Do
  not let a success status alone keep a no-progress loop alive indefinitely.
- Check the entire buffer after the empty call and after EOF. Check both
  endpoint bounds on every call and reserve guard bytes outside the supplied
  success window in the existing buffer. Do not assert that its compacted
  tail `[next,end)` is unchanged: the current in-place native-record conversion
  legitimately uses that space.
- Preserve useful adapter/helper and native-host coverage through the common
  C configuration organization. Do not create a separate native-enumeration
  matrix for request kinds neither current native engine carries, or count
  open/close routing as proof of enumeration routing. Full configuration and
  sanitizer organization remains a later item.
- Correct the conformance coverage descriptions and handle the traversal
  module's mixed assertion ownership in the agreed case audit. Keep the
  actual normative calls and real program observations where required;
  this ruling neither retires their cases nor adds new WF runs to duplicate
  the focused C observations.

The TCP-value and concurrent-close groups are recorded separately below.
No implementation, test retirement, specification change, execution or timing
measurement accompanies this record.

### R07 — Crossed ordinary TCP halves and factory accounting

**Identity, construction and resources.**
`compiler/src/backend/ordinary_values_probe.c::tcp_probe` is a C
ordinary-library integration case using `wf_value`, `wf_connection` and C
`assert`. It shares R04-R06's ordinary-values executable, eleven-source POSIX
or twelve-source Windows `-O2 -g` construction, and local full-gate/host-CI
callers. It does not compile WF or check the compiler's emitted struct/view
ABI. The current caller runs the whole executable with helpers zero and two.

The fixture is one IPv4 loopback listener bound to port zero, with its port
queried by the existing Windows/POSIX `listener_port` helper, and two local
connections. Both ends live in the same process: one listener plus four
connection endpoints consume five descriptor credits. No remote service,
large payload or stress loop is involved. The normal sequence makes sixteen
ordinary IO calls: one listen, two connects, two accepts, one send, one
receive, eight directional closes and one listener close. Address construction
and the host port query are separate from those calls.

Each operation submits and joins before the next one. The driver deliberately
overlaps no half-close calls; helpers may execute operations, but the
shutdown observer linked for the following race tests is unarmed here. The
combined executable still requires its scratch-directory/ordinary-input and
wait-object startup, although this function uses no file or directory fixture.

**Actual values and observations.** Call the two accepted server endpoints
S1 and S2. The function constructs these native aggregates:

| Value | Receive member | Send member |
|---|---|---|
| Original server S1 | R1 | S1-send |
| Original server S2 | R2 | S2-send |
| `crossed_a` | R1 | S2-send |
| `crossed_b` | R2 | S1-send |

Let M be the initial input-factory capacity and B the separate initially
empty `other_factory`.

| Phase | Current assertion |
|---|---|
| Acquire the listener and both connection pairs | Each call succeeds; after all five acquisitions the input factory equals M-5. Individual acquisition deltas are not checked. The two returned peer-address values are unused. |
| One-byte transfer before crossing/closing | First client sends `x` to the first server. Require both returned endpoints to be one and the received byte to equal `x`. It does not observe survival after a half-close. |
| Close both members of `crossed_a` | Close R1, then S2-send, both successfully. The input factory must still equal M-5: each underlying server socket has lost only its first direction. The count is checked after the pair, not after each close. |
| Close S1-send from `crossed_b` into B | Require successful close and B == 1. S1's second direction releases that socket and credits the supplied factory. |
| Close R2 from `crossed_b` into the input factory | Require successful close and input capacity M-4. This releases S2's second direction in the opposite direction order. |
| Close client endpoints and listener | First client closes receive then send; second client closes send then receive; close listener. All succeed and the two factories' final capacities sum to M. The intended final distribution is input M-1, B == 1; the test does not restore the transferred credit to its caller's factory. |

This is a concrete counter observation, not host resource exhaustion. It does
not inspect native descriptor liveness after the first crossed pair closes,
exchange data through the surviving crossed pair, validate peer fields,
exercise IPv6/acquisition refusal, or intentionally race releases. It has no
per-case timeout. Blocking connect/accept/receive/join failures rely on outer
guards or CI limits rather than a bound in this function.

**Ground and overlap.** Active `PRE-1` gives `TcpConnection` ordinary public
receive/send fields and explicitly imposes no relation between fields merely
because a struct was constructed. The native implementation's resource
identity therefore remains attached to each endpoint. In
`completion/file_adapter.c`, `wf_file_connection_release` tracks completed
directions by descriptor; the POSIX/Windows host leaf publishes its last-half
result after shutdown. `ordinary_values.c::wf_close` returns a credit to its
supplied factory only for the final directional release. The capacity design
ground is the current resource-exhaustion-floor decision discussed in R05.

| Existing evidence | Relationship |
|---|---|
| R03 / `harness.c::test_socket_lifecycle_and_the_pair_two_count` | Checks raw bridge first/last release values, descriptor lifetime, both direction orders and a basic transfer. It does not construct crossed ordinary values or observe actual factory counters. Share suitable setup/guards, not a claim that the bridge-level case covers the ordinary library's accounting. |
| R05 file-credit case | Covers acquisition/close credit movement for one-owner files. TCP must delay credit until both original directions close, so returning a file credit does not establish this two-direction rule. |
| `programs/network.rs::crossed_ordinary_tcp_halves_keep_the_other_directions_live` | Compiles WF that constructs crossed values, closes one cross, receives through the surviving second-connection half and sends through the first-connection half. Rust peers observe the bytes and both EOFs while a later checkpoint connection keeps the WF process alive; process teardown cannot supply the EOF evidence. It protects real WF construction/call/close behavior but does not inspect native factory counters. |
| Following `concurrent_half_close_probe` | Pauses a real shutdown while the other direction closes, then observes descriptor reuse and credits. That deliberate interleaving is absent from this sequential case and retains its own review. |
| Conformance `systcp-*` cases | Check ordinary types, ownership/effects and accepted call paths. Cases named `*-permit-returned` have accept verdicts, not runtime credit measurements. They do not establish this native counter sequence. |

The existing WF crossed case uses three compiler driver configurations and
two process configurations per built program. Its `native_ring = true` arm
allows the shipped default; it does not independently require a native engine
to have carried the calls. `false` disables the native engine. That matrix
retains its separate review scope; this item does not add another WF case,
change its configurations or equate an allowed default with observed routing.

**Selected disposition.** The owner agreed to the following recommendations.
Implementation remains deferred; this ruling does not change the live tree.

- Keep this focused C ordinary-library integration group in the existing
  backend test area and common runtime verification stage of local and host
  CI checks. Its reason to exist is exact per-resource/receiving-factory
  accounting for crossed halves. Share compatible C construction, port-zero
  setup, selection/reporting and timeout helpers with the other TCP checks;
  preserve distinct macro/link needs and do not add another executable.
- Observe each acquisition and directional release at its existing call
  site, including both factories where credit can move. Require no credit
  on a first release, exactly one on the original socket's final release,
  the correct recipient and the final distribution. Do not rely solely on
  a total that can hide offsetting errors. Keep the existing two original
  direction orders and arrange reusable fixture state explicitly; a local
  receiving factory going out of scope is not a returned caller credit.
- Replace the pre-close smoke transfer with one-byte exchanges through the
  two surviving directions after `crossed_a` closes. Reuse the current
  listener, clients, two server endpoints and byte buffers: the two
  send/receive pairs require two additional native calls over the current
  single pair, no extra connection, source compilation or test executable.
  This ties data observation to the crossed-lifetime property while retaining
  the exact counter checks. A successful positive transfer of a one-byte
  request is already complete; no large payload or retry-count experiment is
  needed. Keep the existing WF case for the compiler/ordinary-call boundary
  and its process-alive EOF observation.
- Apply the shared TCP process deadline, phase reporting and cleanup to the
  complete case. Keep useful host/helper coverage in the common runtime
  configuration organization, without calling this sequential driver a
  close-race test or using its helper settings as proof of native routing.
  The next deliberate concurrency case and the overall sanitizer/configuration
  review decide their own required observations.

No implementation, test retirement, specification revision, construction,
execution or timing measurement accompanies this record.

### B01 — C runtime publication, wake and lifetime

**Owner ruling.** Agreed, including the common runner direction above;
implementation remains deferred. No exact live-tree wording was approved.

**Shared boundary and construction.** These eight existing test functions
check native implementation obligations. They use C assertions and direct
runtime calls, not Rust `#[test]`, WF compilation or compiler-emitted ABI
evidence. Seven live in `compiler/src/backend/completion/harness.c`; the
eighth is `compiler/src/backend/ordinary_values_probe.c::concurrent_half_close_probe`.
They are grouped for review, not proposed as a new executable.

The completion harness is built from eleven C translation units:
`sched/{core,prim_host,entry}.c` and
`completion/{runtime,wait_host,file_adapter,file_posix,bridge,linux_io_uring,native_contract,harness}.c`.
Compiler `completion-test` uses C11, `-O2 -g`, strict warnings and pthreads,
with the directory/poll and harness observation macros in `compiler/Makefile`.
It runs the whole harness with helpers 0, 1 and 4, and again with no-cache
policy and helpers 1. Root `make check` and compiler `static` reach this
target; Linux/macOS gate and Linux IO-host CI call it. Linux can additionally
require the native ring. ASan/UBSan and TSan rebuild this harness with the
same hooks; TSan runs helpers 0, 1 and 4. These are instrumented executions,
not WF tests. A required-ring invocation does not mean each pure state test
uses the ring.

The ordinary-values executable retains R04-R07's eleven-source POSIX or
twelve-source Windows construction, real host callers and helpers 0/2. Its
`WF_COMPLETION_SHUTDOWN` observer differs from the completion harness's
hooks. No current ordinary-values sanitizer caller was found; the other
harness's sanitizer results do not cover this executable.

Both need a C toolchain and executable scratch storage. Additional resources
are listed per row. Counts below describe source-defined work, not newly
measured durations or proof that an observed schedule is exhaustive.

| Row and function in `harness.c` unless qualified | Actual work and failure observation | Selected disposition |
|---|---|---|
| B01a `test_exactly_one_completion_per_submission_under_race` | Create an eight-byte file; start 12 pthread callers together. Each submits/joins 64 one-byte positioned reads through its own stack records: 768 requests per invocation. Check every value/error/byte and the aggregate publication delta of 768; join all callers. Detects observed lost/extra publications, result mixups and hangs, not every possible schedule. | Keep the real concurrent bridge test and useful route/helper/sanitizer variants. Do not replace it with a single-thread state test or retire rounds solely from the number 768. Reassess its worker/round matrix with the common configuration review, with a stated failure mechanism for retained dimensions. |
| B01b `test_a_completion_publishes_results` | Two direct, same-thread record publications with no registered record waiter. The first checks PENDING, then DONE and value 7; the second checks publication of a close record. No file or extra caller thread. | Keep a small record-publication unit check in the common C runner. Do not describe the previously stored value surviving a same-thread call as cross-thread visibility evidence or give it its own helper matrix/executable. |
| B01c `test_unified_wake_epoch` | On a fresh local runtime, notify with no sleeper: epoch advances, no host wake. Parking against the old epoch returns immediately. Then arrange one real sleeping pthread, notify compute and require a wake plus exact callback/statistic deltas. | Keep the distinct no-sleeper, changed-epoch and one-sleeper branches as a wake-protocol unit group. It does not use file helpers or native IO routing. |
| B01d `test_equal_epoch_notification_rearms_before_resleep` | With one real sleeper, deliver a delayed notification's reset/broadcast tail without advancing its captured epoch. A wait-return hook and mutex handshake observe that it really wakes and rearms before sleeping again. A subsequent notification must wake it and leave no parked thread. | Keep this controlled lost-wakeup regression. It observes an interleaving absent from ordinary one-wake success; do not replace the handshake with sleeps or bulk repetition. |
| B01e `test_one_epoch_wakes_every_announced_thread` | Arrange two real sleepers at one epoch. One compute notification must release both, while the host callback and wake-signal counters advance once. | Keep the two-sleeper/broadcast observation alongside the one-sleeper case. One cannot establish the other's branch; two threads are sufficient for this stated distinction. |
| B01f `test_condition_notifications_coalesce_without_suppressing_external_wakes` | Same-thread scripted announcements, with no actual sleeper: three bursts of 1,024 notifications. Check condition-signal coalescing, rearming for a new announcement, cancellation/no-waiter handling, and an external callback for every notification while announced. | Keep all these state transitions as unit cases, with a small explicit sequence containing repeated notifications. The current Boolean rearm/announcement branches give no separate boundary meaning to 1,024; this is not a concurrent stress test. Keep real sleeper delivery in B01c-e. |
| B01g `test_shutdown_refuses_every_later_entry` | Real pipe, scripted clock, one primer read and 20 queued reads grow four helpers; feed/drain the reads before shutdown. After shutdown check zero/unused query results, `EINVAL` for changing the cap and for a second shutdown. It tests post-shutdown guards and teardown of a previously populated pool, not cancellation with pending IO or concurrent new admission. | Preserve all six post-shutdown observations, but append them to `test_pool_grows_when_operations_wait` after its existing shutdown, retaining its four-helper precondition. That case already uses the same pipe, cap, scripted clock and 21-read growth/drain fixture. Remove this duplicate fixture invocation rather than the teardown coverage; the broader helper-policy review remains B02. |
| B01h `ordinary_values_probe.c::concurrent_half_close_probe` | Real port-zero loopback sockets plus one auxiliary native caller. Pause one direction immediately before actual host `shutdown`; complete the other; resume the paused call. Check credit timing/recipient, then require the next connection to reuse the released descriptor slot, transfer one byte each way and close everything with balanced credits. Run both paused directions. | Keep this deliberate lifetime race and same-slot reuse observation in the ordinary-library C integration group; sequential crossed values and raw bridge release counts do not replace it. Add a common bounded process/phase guard and make the same-slot fixture premise explicit and controlled. Details below. |

**TCP schedule and its limits.** B01h first acquires a listener and one
connected pair (three credits). The extra caller selects receive or send and
requests a 1 MiB thread stack. The shutdown hook announces that it is paused
under a mutex and waits for an explicit resume predicate; the main caller
waits for that announcement before closing the other direction. The actual
host operation can run on a helper or the joining caller. It is the host
shutdown that is paused, not necessarily the auxiliary thread itself.

While it is paused, the opposite close succeeds without returning a credit
to the input factory. After resumption, the original call must succeed and
return exactly one credit to its initially empty, separate factory. That
factory opens the replacement connection and falls to zero. The test asserts
that the replacement's runtime descriptor equals the released descriptor:
an OS descriptor on POSIX, a CRT descriptor resolving to a Winsock handle on
Windows. Both replacement directions transfer `q`; their closes, the old
client's closes and the listener close restore the input factory's initial
capacity. Normal execution uses eighteen ordinary IO calls per direction
case, five acquisitions in total and at most four simultaneously live socket
descriptors. Two direction choices across helpers 0/2 give four controlled
interleavings per ordinary-values target, without a repeated stress loop.

The thread primitive is detached on POSIX and closes its thread handle on
Windows. The `finished` predicate and wait mutex publish the callback's last
use of the stack argument; the lack of a pthread join is not itself evidence
of a lifetime defect. The important gaps are the unbounded pause/finish waits
and blocking socket calls, no direct native-socket liveness observation at
the paused checkpoint, and a same-slot assertion without explicit fixture
control over intervening descriptor allocation. Preserve the equality
observation: dropping it would stop testing reset of that descriptor's
half-close state. Establish a small host-specific reuse fixture or otherwise
justify and control the allocation premise; do not add an unbounded
open-until-reused loop. Observe native socket validity and the existing
factory balances at the paused checkpoint before releasing the hook.

The implementation ground is concrete: `file_posix.c` and `file_windows.c`
perform shutdown before `wf_file_connection_release`; the latter resets its
per-descriptor state before final close. Publishing a direction's release
before its last host use would permit the other direction to close/recycle
the resource too early. Fresh bidirectional IO and balanced final closes
also check that the reused slot starts with fresh half-close state. R03's
sequential bridge lifetime assertions, R07's crossed identity/accounting
and compiled WF's process-alive EOF observation retain their separate value.

**Selected common home and stage.** Keep these
as native runtime implementation tests under the existing backend owners,
executed in the local runtime correctness stage and applicable host CI.
They do not belong in conformance or programs merely because some use real
sockets. Share compatible construction and report logical subcases; add no
WF wrapper or per-row executable. Run local-runtime wake/state cases once
per relevant host/instrumentation configuration, not again for every helper,
ring and no-cache setting they do not inspect. Keep genuine concurrent
bridge/helper/host distinctions and fresh assertions; a changed macro/link
configuration is still a separate construction input.

The completion harness already has a single 300-second process alarm that
names the current test, plus five-second waits in its wake probes. This is
not a reset-on-progress watchdog or a separate bound for every case. Reuse
the common bounded process/phase reporting policy for both harnesses and
preserve deterministic handshakes. Include real threaded cases, including
the ordinary shutdown observer, in the appropriate shared sanitizer scope
when B04 settles its wiring; TSan does not establish absence of logical
lost-wakeup or premature-close errors, so keep the explicit assertions.

The adjacent harness timing loop, remaining helper/join/adapter cases and
platform-specific wake implementations are outside this batch. No tests,
builds, timing runs, specification changes or new DCR/completion review
accompany this discussion record. The owner subsequently accepted the batch
and the common-runner direction; no implementation is claimed.

### B02 — Completion adapter, bridge and progress behavior

**Shared inputs, construction and callers.** All twenty-two functions below
are in `compiler/src/backend/completion/harness.c`, using its existing C
assertion runner, the eleven-source build and hooks described in B01. No Rust
test executable, WF source or compiler invocation is involved. The existing
`completion-test`, compiler `static`, root `make check`, Linux/macOS gate and
Linux IO-host callers currently repeat the whole executable across helper,
no-cache and some required-ring configurations. ASan/UBSan and TSan are
separate instrumented builds. This POSIX harness is not itself the Windows
test runner; B04 owns equivalent actual-host evidence and final variant wiring.

The sources use scratch files/directories/FIFOs, pipes, local sockets and
native threads. A scripted clock chooses measured-wait classifications; these
are deterministic policy inputs, not measurements of this machine's speed.
Local adapters created by a case have their own explicit helper bounds, while
the bridge uses its process-wide initialized configuration. Keep that
distinction when consolidating invocations.

**The owner agreed to all nine rows, with implementation deferred.** B02b's
consumer-dependent retirement remains conditional as stated below. Function
names in the table omit the common `test_` prefix. Counts describe one
invocation of the present source, not newly measured execution costs.

| Row and current functions | Actual operations and observations | Selected disposition |
|---|---|---|
| B02a `linux_independent_operations_use_available_target`; `bridge_independent_positioned_reads` | The Linux case creates an `xy` file, submits two one-byte reads and checks bytes plus exact native/fallback submission deltas; its native arm also expects no initialized target helpers. The second creates a different file, submits two four-byte reads at different offsets and joins in reverse order; it additionally refuses an offset above `INT64_MAX`, requiring `EINVAL`, one publication, no inline execution and an unchanged buffer. Its success-route checks use cumulative `>= 2` counters. | Merge the two-read fixtures into one bridge read group. Retain result/offset association, reverse join, the oversized-offset refusal and exact per-case route observations. Replace cumulative counters with deltas tied to the operations being claimed. Reverse join is not proof of reverse completion; R01 retains its deliberately arranged completion order. Keep native startup/helper expectations at an explicit fresh-process phase rather than making them depend on unrelated tests having initialized the adapter or not. |
| B02b `single_thread_file_progress`; `bridge_open_status_and_close_are_typed_operations`; `checked_open_rejects_and_closes_nonregular_descriptors`; `open_failure_classes_are_typed_outcomes` | Direct zero-helper adapter: queue an open and invalid close together, observe queue/progress counts and `EBADF`, then positioned write/read, status and close. Bridge: open/status/close and a second close returning `EBADF`. Kind/failure fixtures add FIFO-as-regular, directory-as-regular, missing name, regular-as-directory and a successful directory control; wrong-kind results must already have closed their returned native descriptor. Status checks currently establish the returned byte count, not decoded size/kind. | Share a controlled fixture and operation/result helpers; consolidate kind/failure expectations as named rows, keeping exact error/discriminator and descriptor-lifetime assertions and the distinct active direct-adapter/bridge obligations. **Revisit positioned-write and standalone-status coverage:** only probe callers were found, as detailed below. Do not expand those assertions before establishing a current consumer; retire isolated support together if none exists. If a status consumer is established, inspect stable size/kind fields rather than raw `struct stat` padding. No standalone executable or duplicate WF case is needed. |
| B02c `submitted_open_resolves_the_submitters_bytes`; `a_name_no_pool_record_could_hold_takes_the_completion_path`; `open_results_reach_every_independent_owner` | The first observes the caller's path pointer in a delayed direct-adapter request, then two bridge opens resolving separate A/B marker files, plus a directory marker. It also creates a second directory that is never opened. The long-path case builds up to five 240-byte components, submits a 1,211-byte relative file name and compares the result with the same host's ordinary open, including host refusal; its 1,024-byte threshold names a retired buffer. The owner case starts six threads opening the same empty file; half yield 64 times before joining, with no actual observation that completion preceded the join. | Use one marked-name/independent-owner group with distinct file identities and controlled completion observations. Preserve the delayed borrowed-pointer obligation, directory resolution and the long path reaching the engine unchanged; remove the unused second directory and historical-buffer naming as an active contract. Two independent callers with different marker files supply the identity distinction better than six identical opens. Arrange/observe an already-completed record before its join; use B02g for the real waiting path. Do not treat yielding as a scheduling guarantee or mutate loaned path bytes. |
| B02d `more_operations_outstanding_than_the_old_capacity`; `a_submitted_operation_is_kicked_before_it_waits` | Submit 96 positioned reads of an eight-byte file before joining any, then check each result. This exceeds the current 64-entry native submission depth, although operations may finish during submission: it does not establish 96 simultaneously pending kernel operations. The doorbell case opens an eight-byte file, makes one read then a two-read batch, and on the native route checks no submit-time enter, one enter on the first join and no extra enter on the second. | Keep admission/progress beyond the current engine batch boundary and the one-/two-request enter-count observations. Name current queue/engine behavior, not a deleted pool limit; establish the pressure/progress condition instead of inferring it solely from the number 96. Scope doorbell assertions to an observed/required native route, with shared fixtures and fresh counter deltas. Its non-native branch merely repeats file success already covered elsewhere. These operation counts protect a mechanism and are not an exploratory timing benchmark. |
| B02e `uncached_reads_are_target_policy_only` | Four opens on an eight-byte regular file/missing name: checked success, unchecked success, wrong-kind refusal and absent-name failure; one read verifies the bytes. Check relevant open flags and the aggregate number of forwarded no-cache host hints, two when enabled and zero otherwise. | Keep one cache-policy group selected with the setting absent/present in fresh processes; do not repeat the entire harness to exercise it. Share the normal open/refusal assertions from B02b and inspect hint deltas at each existing call so an incorrect first/second distribution cannot hide in the total. Preserve flags, bytes and failure behavior; this is policy correctness, not proof that caching or IO speed changed. |
| B02f `process_wide_target_helper_budget`; `pool_stays_empty_when_operations_do_not_wait`; `pool_grows_when_operations_wait`; `helper_growth_stops_at_the_declared_bound`; `helper_count_above_its_bound_is_refused` | Process-wide count check against the selected environment. Separately, a fresh local adapter performs 32 serialized reads with a scripted short duration and must retain zero helpers, change UNMEASURED to SHORT and allow caller execution. Two blocked-pipe fixtures each perform one primer plus 20 reads: grow to four, or request a cap of eight under a bound of two and stop at two. Initial count two under bound one returns `EINVAL` without starting that pool. | Keep these different policy branches in one group and fold the process-wide count into the relevant configuration observation. The harness pins an omitted helper setting to one, so its default-count branch is not evidence for shipped default policy; R02 supplies that evidence. Keep a queue genuinely deep enough to test growth/capping. The 32-read sequence crosses the current one-in-sixteen sampling interval; relate its count to that purpose rather than calling it generic stress. Check actual read length/byte as well as the existing error/terminal state. Attach B01g's six post-shutdown assertions to the four-helper fixture. Do not rerun these locally configured cases under every unrelated global helper setting. |
| B02g `a_helper_completion_wakes_a_waiting_join`; `an_io_join_waits_on_the_current_stack` | A real empty-pipe read gets a byte from a writer that sleeps 20 ms; check byte, result and one publication, but the fallback count is cumulative. A separate publisher sleeps 20 ms then completes a synthetic record with value 11; the joining thread must see the result, one publication and an increased wait-announcement count. | Keep the real bridge/pipe integration and the direct publication-to-parked-join regression as subcases. Replace sleeps with bounded handshakes observing the needed engine/record/wait state, and use local counter deltas. A zero-helper join may execute the host read on its own thread: do not wait for a helper-only parking event on that route. A delayed thread alone does not establish which execution/park path ran. |
| B02h `readiness_refusal_is_not_a_terminal_outcome`; `directory_progress_is_internal` | A direct adapter reads an empty nonblocking pipe; a writer sleeps 20 ms then supplies `r`; require success and one publication, but no assertion observes an actual `EAGAIN` or poll. The directory case deliberately scripts `EINTR`, then `EAGAIN`, then one byte; a prefilled pipe supplies readiness. It asserts exactly three host attempts, one poll on that descriptor with `POLLIN` and timeout -1, the byte, and Darwin's changed/Linux's unchanged position cell. | Keep shared retry/progress checks with operation-specific observations. Gate the pipe writer on the observed readiness-wait hook so the refused read cannot be skipped by scheduling; retain a real poll and exact descriptor/events. Preserve the directory script's distinct ABI/position behavior. Enable scripts only for their own case and restore forwarding afterward; R06's real directory records/cursors remain separate integration evidence. |
| B02i `a_peer_bound_request_is_left_to_a_helper` | Start with no helpers and an unmeasured local adapter bounded at two. Submit three accepts on a port-zero listener: two helpers block, one accept stays queued; the caller's progress must leave it there. Connect three peers and check all accepts complete. A second zero-cap adapter, with a peer connected first, must execute accept through caller progress. | Keep this controlled starvation/progress regression and share existing TCP setup/deadlines. It tests eager growth for a peer-bound kind, refusal to block the caller when helpers own that work, and caller progress when zero helpers are explicitly selected. It is not a throughput workload. It directly exercises accept; do not claim it separately observes receive/connect/send execution merely because the classifier lists those kinds. |

**Dispositions and scope.** These are native implementation obligations in
the existing completion owner, running in the common runtime correctness
stage locally and on the appropriate real hosts. Apply the selected main
runner organization, with dependency-correct reuse and only the necessary
fresh-process configuration selections. B02b/c/f/g/h/i can share their
respective setup/helpers without merging distinct assertions into a vague
smoke result. The marker, status, per-operation counters and controlled wait
observations strengthen existing fixtures; they add no WF compilations.

**Consumer gap in B02b, carried into B04.** Searches of current compiler,
runtime, program and experiment sources found `WF_FILE_PWRITE` requests only
in this harness and the Linux `native_adapter_probe.c`; the ordinary library
uses stream write instead. The standalone bridge `file_status_submit/join`
pair was called only by this harness. Neither is established as a current WF
execution requirement by those probe calls. Verify the native-adapter
experiment/consumer boundary in B04, then retire unsupported cases and their
orphaned request/API machinery together if no current purpose remains.
This does not remove host `pwrite` used merely to seed a fixture, the required
`fstat` kind check during open, or its existing deterministic compiler tests.
These are different uses. The recommendation is conditional retirement, not
an assertion that a search alone has proved every possible caller absent.

Most functions already share a binary. Reducing function count alone is not
the purpose: remove duplicate filesystem/thread setup and irrelevant repeated
configurations while preserving named failures. A completed operation may
legitimately be observed before the join; a pending one may run on the joining
caller, a helper or a native engine. Each test must establish the state/path
its assertion requires. The three 20 ms sleeps and the 64 yields do not do
that; the existing B01 wait hook and B02 directory script illustrate the
kind of explicit observation to retain. No proposed timeout chooses a WF
acceptance verdict, and no recommendation lowers round counts on speed alone.

The native contract table self-check (`test_native_contract_inventory`) and
its actual consumers are assigned to B04. The adjacent
`benchmark_record_roundtrip` performs 100,000 same-thread publications and
prints nanoseconds per operation with no regression decision; B06 will rule
on its automatic versus explicit measurement home. R03 already owns the
remaining socket lifecycle function, and B01 owns the seven publication,
wake and shutdown functions. Together these assignments account for all
31 test functions and the timing function in the current completion harness.

The owner subsequently accepted B02. No implementation is claimed, and the
consumer question above remains assigned to B04.

### B03 — Scheduler C probes, first part

**Scope and construction.** This part covers five C sources under
`compiler/src/backend/sched/`: `smoke.c`, `deque_probe.c`, `wake_probe.c`,
`cpu_levels_probe.c` and `recursion_budget_probe.c`. Each includes the maintained
`core.c` directly to observe private scheduler state, and is linked with
`entry.c` and the host primitive implementation. On POSIX the build uses C11,
`-O2 -g`, strict warnings and pthreads. These are native runtime tests, not WF
programs: they require a C toolchain, executable scratch storage and host
threads/wait facilities, but no WF parsing, proof checking or lowering.

`compiler/src/backend/tests/sched.rs` puts five `#[test]` wrappers inside the
existing compiler library test executable. Each stages embedded C sources in
a scratch directory, invokes `clang`, launches the resulting C image and
checks its exit status and PASS text; some also check report fields. It adds
no compiler-internal observation. Its C commands are separate from the Rust
library-test build. `test-unit` runs these wrappers; `test-sampling` selects
the other three backend modules and does not run `sched.rs`.

`compiler/Makefile` also builds/runs `smoke.c` and `deque_probe.c` through
`sched-smoke` and `sched-deque-test`, both dependencies of `completion-test`.
Thus root `make check` reaches them both from Rust and directly. The direct
build additionally enables `-Wpedantic` and uses the selected `CC`, whereas
the Rust wrapper spells `clang`; this incidental caller difference supplies
no distinct runtime assertion or declared compiler-compatibility matrix.
Retain the intended compiler/warning policy at the common native caller.

These source-derived counts describe a successful ordinary local invocation,
before sanitizer/Windows variants. They are not fresh timings or the number
of Rust test executables.

| C source | Through Rust `sched.rs`: C builds / child executions | Through direct Make caller: C builds / child executions | Why more than one process or build exists |
|---|---:|---:|---|
| `smoke.c` | 1 / 8 | 1 / 8 | One hook-enabled build; worker count and startup-failure mode change per process. Both callers select the same eight configurations. |
| `deque_probe.c` | 2 / 2 | 2 / 2 | Eight-slot deque, compiled with statistics enabled and disabled. Both callers select the same two configurations. |
| `wake_probe.c` | 1 / 1 | 0 / 0 | One host park/wake and empty-deque measurement. |
| `cpu_levels_probe.c` | 5 / 5 | 0 / 0 | Only three distinct definition sets: default, allow asymmetric CPUs, and zero idle window. The first two are each rebuilt for worker settings 2 and 8. |
| `recursion_budget_probe.c` | 2 / 8 | 0 / 0 | Default budget build runs six worker settings; enlarged leaves-per-lane build runs two settings to reach the clamp. |
| Total | 11 / 24 | 3 / 10 | 14 C compiler invocations and 34 child executions across five logical probes, in addition to building the Rust library test executable. |

Linux/macOS gate callers reach the Rust wrappers and the direct native tests;
the Linux IO-host job also calls the direct tests and rebuilds the deque
variants under TSan. Actual Windows CI builds smoke/deque/CPU probes with
`prim_windows.c` and Windows host facilities. Those host/instrumentation
differences remain meaningful; B04 owns their final collection and wiring.

**The owner agreed to B03a-e, with implementation deferred.** Preserve the
following properties in the common native runtime correctness stage. Remove
the five C-only Rust wrappers once their unique observations and configuration
callers have moved there; do not leave a probe uninvoked during migration.
There is no reason to construct the Rust compiler tests just to exercise these
C mechanisms. The related Rust lowering/ABI tests remain a separate audit.

| Row | Actual work and protected property | Selected disposition |
|---|---|---|
| B03a: startup, nested joins and frame lifetime (`smoke.c`) | Eight fresh-process configurations: workers 1 normally, or workers 4 normally/owner-wait failure/no worker/one surviving helper/two surviving helpers/delayed readiness/delayed partial readiness. Each first checks eight recursive `sum(12) == 4096` computations. Active-pool paths also check oversized/full-slot refusal, reverse joins and payloads, a delayed completion tail across reuse of the same frame, an old notification arriving while the reused frame is pending, and a thief paused across eight complete ring wraps whose stale claim must lose. The registered-wait reuse case requires at least two helpers. | Keep the distinct startup branches and controlled lifetime regressions; eliminate the duplicate Rust/Make invocation. Separate startup assertions from the protocol-case selection, so a configuration runs the latter only for an identified additional observation; retain no-helper, one-helper and multiple-helper distinctions. Do not turn deliberate interleavings into arbitrary repeated smoke or cut the eight rounds merely on speed. Add a bounded common process/phase guard to the currently unbounded waits. |
| B03b: concurrent deque reuse and counters (`deque_probe.c`) | Prepare four lanes directly: owner plus three real stealing threads, with a fourth auxiliary thread observing live counters. Submit 200,000 uniquely identified tasks in batches of eight, requiring a remote completion before the owner may help. Assert each task executes once, a full lane refuses, every slot returns exactly once, the deque empties, the observer ran and steal counts obey the enabled/disabled statistics contract. Each build covers 25,000 batches; normal local wiring currently runs four such builds/executions, or 800,000 task IDs in total. | Keep real concurrency, the eight-slot wrap boundary and both compiled statistics modes, once per required native configuration. Remove the duplicate Rust caller, not one of the two modes. Preserve the TSan and actual-host distinctions. The current round count samples schedules and does not enumerate them; decide any later count change on the protected failure mechanism/evidence, not a blanket reduction. Add bounded progress guards. |
| B03c: park/wake correctness mixed with calibration (`wake_probe.c`) | Prepare four lanes but use one responder thread and the owner. Run 200 warmup plus 2,000 measured signal/wait round trips; require the responder entered the no-post wait path at least once. Then perform 200,000 uncontended empty-deque searches, each required to find no work. Print median round-trip/half-trip and spin-floor costs. No limit, baseline or regression consumer interprets the timing numbers. | Keep bounded, controlled posted/wait/wake and empty-deque correctness cases in the native gate. Move the warmup/sample/median calibration loops to an explicitly invoked measurement mode, with their assertions intact. They serve scheduler research but do not detect performance regression just by printing nanoseconds. Observing some waits does not establish that both sides slept in every round or that half the round trip measures one isolated park/wake cost. |
| B03d: CPU facts and idle-window policy (`cpu_levels_probe.c`) | Query CPU performance-level count twice and require a stable positive result; query allowed CPU count; start the configured pool; inspect the chosen private idle-window duration. Conditional assertions inspect pool-fit/uniformity, the asymmetric override and a compiled zero window. Workers 2 and 8 do not necessarily exercise both fitting and oversubscribed pools on a given host. The expected decision reads the same CPU primitive as the implementation. | Retain one real-host primitive/startup integration observation and deterministic policy cases with controlled CPU count/level inputs for fitting, oversubscribed, mixed/unknown and disabled/override branches. Reuse one build per distinct definition set instead of rebuilding for process arguments. Correct the comment claiming this independently detects a uniform host misreported as asymmetric: that erroneous shared input can make both implementation and expectation agree. Do not invent an independent hardware oracle from the same query. |
| B03e: recursive scheduling budget (`recursion_budget_probe.c`) | Default build: workers 0, 1, 2, 4, 8 and 16 must return budgets 6, 6, 7, 8, 9 and 10. A separate compile with `WF_PAR_RECURSION_LEAVES_PER_LANE=(1ull << 40)` reaches the otherwise inaccessible clamp: workers 1 and 4 must return 24. Each process calls the budget query twice and checks a stable answer. | Keep these small runtime-policy boundary checks in the native group, reusing two builds across their fresh processes. They test the runtime's answer, not the compiler's budget-carrying ABI or recursive lowering; those Rust observations remain distinct. This scheduling budget changes execution strategy for an already admitted program, never WF proof acceptance. |

**Organization and grounds.** Apply the already selected common C runner:
combine compatible cases, reuse native objects with correct dependencies and
select cases per fresh-process configuration. This does not promise that all
five current sources can be linked unchanged into one image: each includes
`core.c` and defines its own main, while `WF_SCHED_TEST` changes hooks/idle
behavior, the deque changes slot count and compiled statistics, and the CPU
and budget probes change policy macros. Preserve a build variant only for
such an actual difference; five logical properties do not require five
permanent executables. Do not reset a live singleton or substitute a scripted
core for the shipped implementation merely to combine processes.

The current grounds are `design/compiler/parallel-lowering.md` and its
`parallel-runtime.md`/`two-worlds.md` children: current-stack joins help real
work, full lanes refuse, startup may retain a partial pool, and recursive
budgeting limits parallel strategy without changing accepted behavior. The
runtime decision explicitly treats the native deque/TSan probe as observed
concurrency evidence, not an exhaustive scheduler proof; startup, completion
and stack-floor obligations are separate. These recommendations preserve
those distinctions while removing duplicate orchestration and unjudged
measurement from automatic correctness checks. B06 will review the wider
performance-regression/explicit-measurement protocol.

**Following parts.** `compiler/src/backend/tests/{parallel,loop_split,exhaustion}.rs`
contain 34, 17 and 23 `#[test]` cases respectively, all in the same compiler
library test executable. Their current `test-sampling` filter does not
establish that each case samples a schedule. The following sections audit
parallel lowering and loop splitting; exhaustion remains the next group.
`sched/grant_observer.c` is linked into some generated-program tests to observe
real grants/threads; it is not a sixth standalone probe with its own main.

The owner subsequently accepted B03a-e. No implementation is claimed.

### B03 — Rust parallel lowering, second part

**Scope and construction.** This part audits all 34 `#[test]` cases in
`compiler/src/backend/tests/parallel.rs`, grouped into eight rows below.
They belong to the existing compiler library test executable, not 34 Rust
executables. `compiler/Makefile::test-sampling` selects the whole module
alongside `loop_split` and `exhaustion`; root `make check` and the
Linux/macOS sampling CI jobs reach that selector.

The resource boundary matters more than the module name:

| Current kind | Cases | What is constructed and executed |
|---|---:|---|
| Compiler IR/layout observations | 11 | Most call the compiler library on embedded WF source and inspect checked/lowered state or emitted LLVM. The slot-constant check reads embedded runtime source. No host compiler or emitted-program execution is required. |
| Result-comparison helper check | 1 | Ordinary Rust vectors/strings exercise `identical`; no WF compilation or native child. |
| Machine stack-frame observations | 2 | Compile WF to LLVM, then invoke Clang with `-S -fstack-usage`; inspect assembly/stack-usage through the stack ledger. No executable is linked or run. |
| Native compiler-output observations | 20 | Compile embedded/generated WF through the Rust library, invoke Clang to build native images, sometimes add a C observer or replace selected emitted calls, then execute fresh child processes. One Rust case may construct many images and run a matrix. |

The native cases need Clang, executable scratch storage and the shipped
ordinary library, scheduler, completion engine and stack floor. Particular
cases additionally need real worker threads, stdout, the ordinary directory/
file library, or controlled allocation/lane observers. They do not run
`cargo test` once per WF source or require a separate compiler CLI invocation:
the library test process calls the compiler API. An IR assertion can still
pay WF checking/lowering cost even when it has no native build.

Current `build_linked_executable` shares immutable runtime objects for
ordinary no-macro builds and adds a fresh WF module/observer. In contrast,
`CountedProgram::link` uses `link_counting_grants`, a separate Clang command
that stages and recompiles the runtime sources. Unify that construction
through the established helper with the same inputs/options and an explicit
observer, retaining fresh executions/assertions. Do not describe existing
runtime-object reuse as caching WF results or eliminating every native build.

**The owner agreed to B03f-m, with implementation deferred.** These are
property/fixture groups, not eight proposed executables. Where only external
program results remain, the receiving owner is the existing
`compiler/tests/programs/parallel.rs` group; direct implementation
observations remain compiler tests. Reuse a fixture and its valid construction
rather than duplicating it merely to split assertions across categories.

| Row and current count | Actual operations and observations | Selected disposition and home |
|---|---|---|
| B03f: IR, layout and offer selection (11) | Inspect the outlined thunk, stores only after acquisition, offer/inline/join/value order, sequential clone set and bodies, one bootstrap world selection, no extra caller stack slot, denied/borrowed-call handling and scalar-leaf suppression. Layout tests distinguish exactly fitting/too-wide frames, budget-field overhead and an unrepresentable target domain; a source check compares Rust/C slot constants. The Windows case inspects external declarations and absence of weak fallbacks, without performing a Windows link. | Keep focused compiler implementation tests. Share repeated emission of the same `OVERLAPPING_FOLD`/lowering configuration within the fixture group while retaining named assertions. These are not schedule-sampling cases and need no native execution just because their input is WF. Keep the cross-language layout agreement until its duplicate constants are actually removed; generation was not selected here. Name the Windows assertion as emitted linkage obligations, leaving actual host-link evidence to B04. |
| B03g: generated-code boundaries and distinct call shapes (7) | Host builds exercise source/runtime symbol coexistence, exact/too-wide lane frames, a call in an if condition, three siblings feeding loop phis, linked/source ordinary calls, and mixed small/large scalar offers. Checks include concrete exit values/bytes and LLVM offer/join counts/order. Several use workers 0/1/4, but successful runs alone do not prove every granted/refused edge ran. One case claims a module links without runtime while using the always-linked common helper. | Keep the six useful compiler-output/ABI/control-flow cases; share construction for unchanged modules and keep meaningful granted/refused observations where the claim needs them. Retire the misleading no-runtime native smoke: merge its no-offer predicate into the existing denied/default IR group and remove the obsolete link claim. Do not add a new no-runtime build path to perpetuate it: the current driver and design link the ordinary runtime for every program. Mere absence of compute offers still is a useful emission property. |
| B03h: stack-frame cost (2) | Both use `DEEP_RECURSION` at depth 1,000. One obtains ordinary and parallel stack ledgers, requiring the parallel module's sequential clone to have the ordinary frame size. The other rebuilds the same parallel module and requires its budget variant's frame to exceed the clone by at most 48 bytes. Despite their names, neither runs a deep recursion. | Merge fixture construction into one stack-layout group: one ordinary and one parallel machine-code report, with both assertions. Keep in compiler native-output tests. This removes the third assembly compilation without replacing a machine-frame observation with LLVM text. The 48-byte constraint is an implementation resource-regression check, not elapsed-time benchmarking. |
| B03i: program results across lowering/worker choices (5) | The ordinary tree fold is compared under workers 1/2/4/8 by two cases: one runs five repetitions per setting plus a separately compiled sequential reference, the other runs once per setting plus that same reference. A pinned-budget-2 form checks tree borrows/owned construction with two worker settings. A 4,000-deep spine and a fold with a builtin between its sibling calls each run four settings three times plus a sequential reference. | Consolidate the demonstrably overlapping ordinary-tree comparisons and share their source/lowerings with the budget variant. Put external tree/spine/window behavior in the programs parallel group, retaining focused internal assertions in their compiler owner without creating duplicate full runs for incidental IR checks. Preserve the deep, interposed-statement and owned/borrowed distinctions unless receiving coverage actually establishes them. Give the program family a justified expected result, preferably a small independent reference, alongside the sequential/parallel differential; two wrong lowerings agreeing is not an independent semantic oracle. Reassess repetition/configuration axes by observed paths, not a blanket reduction. |
| B03j: real runtime use, configuration and worker IO (3) | Two counted tree cases inspect actual steal counts and pool startup: opt-out 0/1, absent setting, invalid abc/-1/65 before program output, and a four-worker execution that must steal where a host-core heuristic enables the assertion. Up to 32 additional attempts may search for a steal. A separate published WF helper calls the real linked IO library and prints X; its observer prevents the owner from joining until another worker enters the actual queued callback, checking exact publish/enter/complete/other-thread counts at workers 0/4. | Keep runtime/driver integration and the compiler-generated worker-to-library ABI observation; a pure C scheduler test does not replace the latter. Merge the counted-tree setup/configuration runs and use a bounded controlled real-worker observation rather than relying on a tiny task being stolen by chance. Reuse the worker-IO case's synchronization pattern without replacing the real queue or callback. The current retry helper checks exit success but not each retry's bytes: every retained run supplying positive steal evidence must also check the expected result. Do not count a host-skipped assertion as positive concurrency evidence or claim fewer cores mathematically prevent a steal. Preserve the worker-IO case and bound its spin wait. |
| B03k: ownership and failure cleanup (2) | A two-field owned result checks independent caller destinations after lane release. Its observed build runs forced refusal, ordinary real acquisition, and publication deferred until join; exact attempt/grant/release/pending counts accompany the result. A Heap loop creates two cells on each of four iterations, swaps borrowed owners through a helper that calls the linked file library, and checks the exact allocation/free trace and final values. Off/On lowering each tests success and refusal of allocations 2 through 9: currently 18 native images/runs because the C observer hardcodes the refusal position. | Keep both as compiler ABI/ownership tests, sharing existing allocation/lane helpers rather than adding WF smoke cases. The Heap case tests ordered execution and cleanup, not an actually split loop. Pass its refusal position as controlled per-process input to one observer: two lowering builds can retain all 18 fresh executions and exact traces. Do not drop the distinct failure positions merely because they look repetitive. Remove the stale comment referring to an absent Arena case below it. |
| B03l: recursive policy/ABI matrix (2) | The large case covers self/mutual recursion, scalar/destination results, nine policy spellings, runtime budget 5/0 and all-refused/one-grant schedules: 36 native images and 144 executions. A deterministic C observer stores one callback and runs it at join; this tests generated control/ABI, not real concurrency. The other case emits three policies for a nonrecursive leaf, asserts each entire module equals the ordinary parallel module, then builds/runs all three equal modules. | Preserve recursive graph shape, result ABI, budget cut and refusal-path observations. Keep default-versus-explicit-default emission equality but share the native image after equality is established; only vary runtime budget where emitted code queries it. Keep fixed budgets 1/3/6 and budget-off differences. For the leaf case, retain all policy equality checks and build/run the common module once. These remain compiler implementation tests, independent of B03e's native runtime arithmetic. The matrix calculation below identifies exact redundant dimensions, not a speed-based pruning rule. |
| B03m: comparison and missing-join negative controls (2) | One small Rust test feeds equal/different/short/empty bytes to `identical`. The other replaces all emitted join calls with no-ops in the tree fixture, builds the damaged module and tries up to twelve real four-worker runs until one exits abnormally or differs from the intact reference. It observes sensitivity to one injected defect, with schedule-dependent detection and possible crashes; it does not test twelve language behaviors. | Keep the inexpensive comparison-helper check with shared test support, requiring no WF/native build. Replace the probabilistic crash-oriented negative control with one bounded controlled missing-join check using the existing delayed-publication/join-before-release observation pattern on an appropriate existing scalar/owned-result fixture. Establish rejection of the injected missing-join path before retiring the old control; keep positive real-scheduler coverage in B03j. Do not move it to conformance or call damaged generated code a rejected WF program. |

**Exact recursive-matrix overlap.** `lowering/builder.rs` normalizes the two
implicit-default forms to the same budget/refusal/leaf-limit settings as
their two explicit `RuntimeDerived` twins. Retain all nine entry-form
emissions and compare each twin's entire emitted module before sharing the
native build. That leaves seven distinct native policy builds per source
shape rather than nine: 28 instead of 36 over four shapes. Of those seven,
only two query the runtime budget. The other five (Off, pinned 1/3/6, and
pinned 3 with sequential refusal) cannot observe `WF_TEST_BUDGET`, so their
two budget-value executions are duplicates for each grant setting. Keeping
both runtime answers where used and both grant selections gives
`4 * (2 * 2 * 2 + 5 * 2) = 72` executions instead of 144. This is a
source-derived proposed matrix, conditional on the emission-equality
assertions; no performance result or executed equivalence is claimed.

**Limits and receiving coverage.** The current public helper
`module_requires_parallel_runtime` detects emitted offer definitions or
declarations; its repository callers are tests, not the present driver's
link selection. Its old comments do not override the current shared-library
design. B03g retains the useful emission assertions without constructing a
retired optional-runtime test path. The same claim also appears in the
programs/loop tests and must be corrected during their corresponding audits.

The programs group already includes `par_layout.wf`, adaptive quadrature,
whole-corpus overlap builds and its own internal ledger/IR checks. These
are relevant receiving/overlap candidates, not evidence that each tree,
interposed-call, owned-result or deep-spine observation is already covered.
Before retiring any of those fixtures, compare its actual input, path and
oracle with a named receiving case. Moving a source must not add a new copy
of its native run under the previous owner. No source-language acceptance or
rejection verdict changes in this proposal.

Reasons come from the accepted property-based admission baseline and the
current `design/compiler.md` ordinary ABI/runtime decision,
`parallel-lowering.md` policy decisions and `two-worlds.md` clone,
budget-family and lane-owned-frame decisions. Required compiler-output
checks survive; a pure Rust sampling-module label, an old test title or an
incidental assertion does not justify an extra end-to-end construction.
The following section reviews loop splitting (17 cases); exhaustion (23)
remains next.

**Complete function mapping.** Names below are all the current tests in
`compiler/src/backend/tests/parallel.rs`; each appears exactly once. A row
can split construction/observations during migration without claiming the
current function is already in its proposed home.

**B03f (11 current tests)**

- `selected_target_proves_the_complete_ordinary_lane_frame`
- `ordinary_lane_frame_limits_match_the_runtime_slot`
- `a_permitted_pair_is_outlined_offered_and_joined`
- `handing_a_call_out_adds_no_stack_slot`
- `the_sequential_clone_is_the_sequential_lowering`
- `the_bootstrap_selects_one_world_once`
- `windows_parallel_modules_fail_closed_at_the_link_boundary`
- `a_denied_pair_emits_exactly_the_sequential_calls`
- `a_permitted_pair_whose_first_member_is_borrowed_is_not_handed_out`
- `the_default_compilation_hands_nothing_out`
- `scalar_leaf_control_drops_all_small_offers_without_a_clone_or_runtime`

**B03g (7 current tests)**

- `a_program_named_like_the_runtime_still_compiles_and_links`
- `ordinary_overlap_uses_only_target_proved_lane_frames`
- `a_call_written_as_an_if_condition_joins_a_compute_overlap_group`
- `a_group_joins_its_compute_members_newest_first_and_continues_at_the_oldest`
- `a_module_that_hands_nothing_out_needs_no_runtime`
- `a_linked_body_and_source_bodies_use_one_ordinary_call_protocol`
- `scalar_leaf_control_keeps_mixed_chain_results_and_join_boundary`

**B03h (2 current tests)**

- `handing_calls_out_keeps_the_sequential_recursion_depth`
- `the_shipped_default_keeps_a_deep_recursion`

**B03i (5 current tests)**

- `a_recursion_deeper_than_the_offer_bound_still_publishes_the_sequential_bytes`
- `an_overlapped_program_reports_one_byte_sequence_at_every_worker_count`
- `a_budget_family_preserves_exclusive_tree_borrows_and_owned_constructors`
- `the_overlapped_lowering_agrees_with_the_lowering_that_hands_nothing_out`
- `a_fold_whose_calls_are_separated_by_a_builtin_hands_out_and_agrees`

**B03j (3 current tests)**

- `the_runtime_replaces_the_modules_weak_refusal`
- `an_absent_worker_setting_starts_the_pool_and_an_explicit_opt_out_does_not`
- `an_ordinary_worker_helper_can_call_the_linked_io_library`

**B03k (2 current tests)**

- `owned_pair_results_survive_ordinary_join_and_forced_refusal`
- `heap_box_loop_keeps_provider_order_and_updates_borrowed_owners`

**B03l (2 current tests)**

- `recursive_controls_preserve_scalar_and_destination_results`
- `recursive_controls_keep_leaf_calls_unchanged`

**B03m (2 current tests)**

- `the_repeat_comparison_reports_an_injected_difference`
- `the_repeat_reports_a_lowering_whose_joins_were_removed`

The owner subsequently accepted B03f-m. No implementation is claimed.

### B03 — Loop splitting, third part

**Scope and construction.** All 17 `#[test]` cases in
`compiler/src/backend/tests/loop_split.rs` belong to the same Rust compiler
library test executable as the preceding module. Five inspect compiler
IR/ledger or embedded slot constants without native construction; twelve
build and execute compiler output. They currently all run under the
module-wide `test-sampling` selector, locally through `make check` and on
Linux/macOS sampling CI. This is not 17 independently built Rust targets.

Fixtures are embedded WF strings or small generators in that Rust source:
`PERMITTED_FOLD`, `EDGE_RANGES`, `WIDE_FRAME`, `CAPTURED_XOR_FOLD`,
`INDEPENDENT_MAP`, two map-source edits and `admitted_combine_source`.
The Rust process calls the compiler library to obtain LLVM/permission
evidence. Native cases then use the existing Clang builder, shipped ordinary
library/runtime/floor and sometimes `CountedProgram` or another C observer.
Their extra resources are executable scratch storage, worker threads and
captured stdout/stderr; the alleged stack-limit case also starts a shell.
No corpus, test source, specification or caller is changed in this audit.

The basic fold and map use 400,000 source iterations, with a 24-round integer
mix per element; the captured fold uses three values (salt, round count and
stride) asymmetrically. These workloads were made substantial enough for the
runtime's work threshold to allow splitting. This is a source-level
description, not an instruction count or a new duration measurement: native
optimization can remove or simplify work. The combine table already packs
its rows into one WF program; it is not seventeen WF executables.

**The owner agreed to B03n-u, with implementation deferred.** Apply the
already agreed B03 shared construction and controlled-worker observation. In
particular, a retained positive grant execution must also check its output;
the current shared retry helper does not do that. Preserve real default
runtime integration separately from controlled compiler-path coverage.

| Row and current count | Actual operations and observations | Selected disposition and home |
|---|---|---|
| B03n: emitted shape, capture frame and defaults (5) | Check that a split emits a chunk which remains a loop, plus a splitter using ordinary acquire/publish/join/release; budget is queried once at entry. Check ordinary compilation and the sequential clone avoid splitting, while chunk/clone bodies agree. A wide-scope source must remain permitted but decline actualization with a frame-size diagnostic. A separate Rust/C frame-limit check repeats the same header-number assertion in B03f. | Keep focused compiler tests and share basic-fold emission. Merge the duplicate slot-limit check into B03f's stronger capacity/alignment check. Make the wide-frame fixture depend on captures the loop actually uses: it currently declares 32 locals but reads only `a0`, and relies on the present whole-scope capture implementation. Do not require retaining unused captures just to keep this test declining after a legitimate compiler improvement. Keep permission, target fit and optional lowering refusal distinct. |
| B03o: ordinary fold and sequential reference (2) | The same 400,000-element `+wrap` fold is built/run first at workers 0/1/2/3/4/5/8/10/16 and unset, then separately at 1/2/4/8 against a separately emitted unsplit build. An additional counted image searches for a steal at 4/8 (up to four attempts when the host heuristic enables it), plus an opt-out run. | Consolidate one program family with one ordinary reference and one parallel module, retaining required observed-build variants rather than regenerating the same inputs per assertion. External output behavior belongs in the existing programs parallel group; internal split/clone evidence stays with the compiler. Retain default and disabled behavior and meaningful division/scheduling boundaries, with a justified result oracle. Select worker cases by what they distinguish: for a work-sufficient range, 2/3 workers can yield the same budget, as can 4/5 and 8/10. More worker settings do not automatically mean different split trees, although worker scheduling can still differ. |
| B03p: split-work policy and ordinary reporting (2) | A C exit observer directly asks `wf__par_split_budget` for spans 0, 4,096, 65,536 and maximum unsigned, using weight 219 or maximum. Seven absent/empty/valid `WF_SPLIT_WORK` settings check exact budget tuples; four malformed/out-of-range settings must stop before WF output. Separately, the ordinary unobserved image runs workers 1/4 with report modes 0/1/2, checking silence versus one compute report, worker-start counts and unchanged bytes; mode 3 is refused. | Move the budget arithmetic/environment-parser matrix to the common C runtime policy group. It does not need to execute a 400,000-element WF fold to read four C return values. Queries can run before pool creation; even the maximum configured-width arithmetic boundary need not launch that many workers. Retain generated-program integration for disabled/enabled splitting and refusal before the body, sharing B03o's images. Preserve the ordinary unhooked report path and its output/diagnostic assertions; an observer that prints counters itself is not a substitute for shipped reporting. This is configuration correctness, not a timing benchmark. |
| B03q: captures and nonzero reduction seed (1) | Inspect the chunk's six i64 parameters: incoming seed, two endpoints and three captures. Compare captured XOR fold bytes with unsplit code at workers 1/2/4/8; optionally build a counted image for actual worker evidence. Capture values are used asymmetrically, and the source accumulator starts at a nonzero value. | Keep as a compiler capture/parameter-passing and seed-preservation regression, sharing observers/building helpers with the other cases. Observe the actual split/callback path and its result. Correct the comments claiming XOR and wrapping addition have different identity elements: both are zero. The distinct combine operation, incoming nonzero seed and capture order are the real reasons for this fixture; the 17-row table does not establish all of those ABI observations. |
| B03r: owned map, borrowed read-modify-write and map plus reduction (3) | The owned map writes a 400,000-byte buffer; IR checks Unit-returning chunks, descriptor capture, joins and exactly one outer release on each return path. The borrowed variant updates the same-index element through `&uniq`, but initializes every old element to zero. The mixed variant returns a real u64 reduction through the splitter and captures the output descriptor, but exposes only the checksum's low byte by overwriting element zero. All compare native output with separately emitted ordinary code. | Keep the three compiler mapping/capture/ownership distinctions, with common fixture generation and construction, not three new executables merely for categorization. Use nonzero initial data for the read-modify-write case so omitting the old-value read changes the result. Publish/check the full reduction value alongside all map bytes instead of checking eight bits and discarding the original first mapped byte. Preserve the owned-map release/join assertions and observe the intended path for each claimed variant. These changes improve the existing oracles without adding a new WF compilation family. |
| B03s: all admitted combines, one table case (1) | Ten operations are represented by 17 rows: seven integer combines at u64/u8 and three Bool combines. Each row folds 200,000 source elements; the program emits 136 bytes. Check a split-ledger entry naming each row/operator, run ordinary and parallel builds at eight named worker settings plus unset, compare per-row bytes, then optionally look for a positive aggregate steal count at eight workers. Calls are deliberately dependent so counted compute work comes from range splitting, not sibling-call overlap. | Keep the combined table as compiler native-lowering evidence, with row-specific diagnostics. Establish an executed split/join and correct result for each row using a small controlled budget/observer configuration; one aggregate positive steal proves at most that some row supplied work. Separate this from a representative run with the real default policy. The large spans currently compensate for the work threshold, so use smaller discriminating data once controlled path coverage is established, preserving meaningful seed, width, overflow and uneven-boundary cases. Do not create seventeen binaries or blindly multiply the native table across all widths. Keep the existing independent Rust identity table: it covers signed/unsigned 8/16/32/64-bit cases that this native table does not. |
| B03t: empty, inverted and singleton ranges (1) | A single WF source starts its accumulator at 7; empty and inverted ranges must leave it at 7, and a one-element range must add that element exactly once. The source is compiled with splitting enabled and run at workers 0/1/2/4/8. The check observes exact program results and presence of a splitter, not that a short range received a positive runtime allowance. | Keep these focused compiler-lowering boundaries and integrate their construction with the controlled split cases. Observe that empty/inverted ranges contribute no work and do not send a wrapped span into budget/recursion; distinguish budget-zero leaf behavior from a positive-budget range too thin to divide. A controlled allowance can exercise the latter without a large workload. Do not claim five worker values establish five boundary paths or substitute a permission-only conformance case for the generated splitter checks. |
| B03u: misleading optional-runtime and stack claims (2) | The first checks a weak split-budget definition, then runs the fully linked basic fold at workers 1/8; it never performs a runtime-free link. The second runs that same program at workers 8 after `ulimit -s 512`, claiming bounded split stack use. The ordinary floor instead creates the WF entry stack with its own 1 GiB reservation, and workers use that runtime size; the case does not observe which stack was used or its consumption. | Retire both redundant native wrappers with their obsolete claims. Merge the useful weak-definition assertion into B03n and the 1/8 output observations into B03o. Replace the alleged stack proof with focused generated-split control observations (empty/thin/zero-budget termination and decreasing recursive budget) plus the runtime budget cap in B03p. Those establish the logical split bound, not measured whole-thread stack bytes; any machine-byte claim belongs to an actual stack-ledger/floor observation. Do not keep a 512 KiB claim whose execution is normally on a different stack. |

**Why the combine table and identity unit are distinct.**
`lowering/builder/split.rs::the_identity_of_every_admitted_combine_is_two_sided`
uses the production identity/operation selection with an independent Rust
evaluator over both signs and four widths, plus Bool. It cannot replace
compiled chunks, capture transport and result recombination; conversely,
the native table's u8/u64/Bool representatives do not cover every signed or
intermediate-width table entry. This existing unit test is an adjacent
dependency, not an eighteenth case in the current loop module.

The native-table comment about a wrong XOR identity always cancelling over
an even number of chunks is also inconsistent with the current builder:
the incoming accumulator goes left and only the right child gets the
identity. Already at one bisection there is one new identity seed, not two.
Correct the explanation and check actual seed placement in the controlled
table; do not discard a useful row on that cancellation claim. This is
separate from the captured-XOR comments' simpler error that addition's
identity differs from XOR's.

**Meaning of the stack and policy observations.** Normal POSIX entry setup
in `backend/wf_floor.c::wf__floor_run` requests
`WF_FLOOR_STACK_BYTES`, and `sched/prim_host.c` sets explicit worker stack
sizes. If entry thread setup fails, the floor can fall back to the original
thread; the alleged bound test neither requires that fallback nor measures
its stack. An inherited shell limit therefore does not establish a fixed
bound for this test's intended execution. Keep the logical recurrence
observation and real machine-resource evidence correctly named, rather than
launching a large recursion merely to replace this wrapper.

`wf__par_split_budget` combines a work-derived chunk allowance with the
per-lane cap and returns its base-two logarithm. Direct C tests should retain
zero/disabled, threshold, saturation and cap behavior; generated-code tests
should establish that the right values reach the query and that subdivision
preserves results. A controlled test budget does not replace the real
policy's test, and neither chooses WF source acceptance.

**Authority and affected evidence.** Active `PAR-2` enumerates the ten
combines and requires unchanged values/iteration membership, while allowing
an implementation to overlap nothing. Therefore seeing source acceptance or
a ledger entry does not prove a generated parallel path executed. The
current compiler grounds are the loop-leaf decision in
`design/compiler/parallel-lowering.md`, clone/budget/frame decisions in
`two-worlds.md`, and runtime-owned-stack and work-cap decisions in
`parallel-runtime.md`/`resource-exhaustion-floor.md`. Keep normative
conformance and implementation actualization observations distinct.

The `PAR-2` coverage declaration in `tests/conformance/manifest.jsonl`
currently describes ten worker settings and the compiler's identity/output
checks. Bring that declaration into agreement with the eventual receiving
checks/matrix in the implementation change; do not leave obsolete counts or
claim per-row execution from one global counter. No manifest/verdict or
specification changes accompany this discussion. Exhaustion's 23 cases are
the next B03 review group.

**Complete function mapping.** All 17 current tests in
`compiler/src/backend/tests/loop_split.rs` appear once below.

**B03n (5 current tests)**

- `a_permitted_loop_is_outlined_split_and_joined`
- `the_default_compilation_of_a_permitted_loop_splits_nothing`
- `the_sequential_world_of_a_split_loop_is_the_loop`
- `a_loop_whose_frame_is_too_wide_declines_and_says_so`
- `the_compile_time_frame_bound_is_the_runtimes`

**B03o (2 current tests)**

- `a_split_loop_publishes_one_byte_sequence_at_every_worker_count`
- `a_split_loop_agrees_with_the_lowering_that_splits_nothing`

**B03p (2 current tests)**

- `split_work_setting_changes_budget_without_changing_the_fold`
- `ordinary_shared_runtime_can_report_without_an_observer`

**B03q (1 current tests)**

- `a_split_loop_carries_its_captures_and_a_second_combine`

**B03r (3 current tests)**

- `an_independent_map_joins_and_preserves_its_outer_buffer`
- `a_borrowed_read_modify_map_preserves_the_sequential_bytes`
- `a_map_and_reduction_preserves_both_results`

**B03s (1 current tests)**

- `every_admitted_combine_splits_and_publishes_the_unsplit_bytes`

**B03t (1 current tests)**

- `a_degenerate_range_folds_to_the_accumulator_it_arrived_with`

**B03u (2 current tests)**

- `a_module_with_a_split_loop_and_no_runtime_still_runs`
- `a_split_loop_costs_a_bounded_stack`

The owner subsequently accepted B03n-u. No implementation is claimed.

### B03 — Resource exhaustion and derived cleanup, fourth part

**Scope and construction.** All 23 `#[test]` functions in
`compiler/src/backend/tests/exhaustion.rs` share the Rust compiler library
test executable. Seven inspect emitted LLVM without a native build; one
builds a WF-derived executable only to run `nm` over it; five build and run
C-only floor fixtures; ten build and execute WF-derived native code. The
large-frame case builds both an intact and an ablated image. The current
module-wide `test-sampling` selector runs every one locally through
`make check` and on Linux/macOS sampling CI, including the seven IR-only
checks. A module name is not evidence that each case samples a schedule.

WF sources are embedded constants/generators in that file:
`MIXED_DEFINITIONS`, `spine_source`, `HEAP_RECORD_LANE`,
`REFUSED_ALLOCATION`, `ALL_HEAP_FORMS`, `LARGE_FRAME_SPINE`,
`boxed_spine_source`, `buffer_chain_source`, `SHALLOW_OWNERSHIP`,
`BUFFER_CYCLE` and `WIDE_BUFFER_CYCLE`. The ordinary builder uses the
compiler library's LLVM output, Clang `-O2`, and the shipped runtime/floor.
The C-only `build_floor_fixture` helper instead stages a fresh
`floor_body.c` and copy of `wf_floor.c`, compiling both with Clang
`-pthread -O2` on each call. Its setup-refusal variant prefixes only the
floor translation unit with host-function substitutions; the body still
calls real host facilities. None of these five C cases calls the WF compiler.

Resources include child processes, real threads and signals, protected or
unmapped virtual-memory regions, alternate signal stacks, captured stdout/
stderr, and, in the deep cases, actual recursive stack use or many allocated
nodes. The shipped POSIX floor reserves a 1 GiB entry stack and exports that
size for compute workers. Reservation alone is not resident usage, but
descending until exhaustion is not merely reserving unused address space.
These are source-derived facts, not new timings or memory measurements.

**The owner agreed to B03v-ad, with implementation deferred.** The nine rows
are property groups, not nine proposed executables. Keep the accepted common C
runner and compiler/program/conformance ownership rules; combine construction
only when its inputs and observation requirements agree.

| Row and current count | Actual operations and observations | Selected disposition and home |
|---|---|---|
| B03v: stack-probe completeness, ordinary cost and large-frame containment (3) | Compile a mixed ownership/recursion fixture with and without parallel lowering and require every emitted definition to carry the probe attribute. Separately link the ordinary fixture, run `nm`, and require no `chkstk` string, without checking `nm` success. The large-frame case exposes a generated function with a 7,168-u64 array, calls its base case once on a controlled 32 KiB stack above 16 MiB of protected reservation, and compares probed versus exactly-one-definition-ablated images: resource record/abort versus the host protection signal. | Keep compiler emission and real machine-code containment evidence; preserve the bounded positive/negative large-frame fixture. Share the ordinary fixture's emission. Attribute completeness is only over definitions actually reached: verify relevant thunk/clone/chunk/drop families using already retained fixtures, not a claim that two emissions necessarily produce every kind. Replace the whole-executable symbol-name assertion with a checked, target-appropriate observation of the small WF function's machine prologue, sharing existing assembly construction where possible. The Linux target uses inline probing, so absence of `chkstk` alone proves no absence of probing cost. Do not substitute IR text for the actual containment check. |
| B03w: entry and compute-worker stack provisioning (2) | Run a two-million-level sequential recursion under a 1 MiB shell stack limit. Separately run the same depth with parallel lowering at workers 0/1/2/4/8/16 and unset, three times each: 21 executions. Only returned status is checked; the fixture does not establish which recursive segment ran on a worker or measure that thread's stack. | Keep the real provisioning regression, with direct host stack-bound observations on the actual floor entry and an observed compute-worker callback in the native runtime group. Verify the shipped reservation independently from a controlled small-stack WF recursion success/failure fixture; retain a generated-code integration observation rather than replacing all WF execution with C arithmetic. Replace the two-million-depth/repeat matrix only after these receiving checks detect the original undersized-worker defect. Select startup settings by their distinct paths, sharing B03 startup coverage. Treat the documented protected original-thread fallback separately from successful entry-thread creation; neither stack reservation nor these samples proves schedule-independent remaining depth under nested helping. |
| B03x: genuine entry and worker exhaustion (2) | Build sequential and parallel versions of a 100-million-level recursion. Run the former once and the latter three times with inherited/default worker configuration. Each must end by a signal and write exactly the stack record. The worker case explicitly relies on a historical probability of a steal; it never establishes that the exhausted stack was a worker's. | Keep actual overflow reporting on both thread classes in compiler/runtime integration. Reuse a generated recursion with a controlled small stack and an observed real scheduler handoff, so the worker path and guard hit are prerequisites, not guesses from three runs. Keep the default-size provisioning checks in B03w and exact resource record plus expected abort disposition here. Retire the giant/repeated probes only when equivalent positive-path evidence is demonstrated. The existing single-large-frame ablation checks a different containment failure and is not by itself the complete replacement. |
| B03y: protection setup refusal (1) | One instrumented C image supplies 13 fresh-process scenarios: eight entry/process modes and five attached-thread modes. Inject alternate-stack mapping/installation failure, either signal-install failure, page-size query failure, or either host stack-bound query failure; successful controls must print `ran` and return 73. Failure modes must abort with the setup diagnostic before the body marker. | Move the useful matrix to the common native floor group and remove the Rust wrapper after it has a caller there. Retain all distinct failure branches, the entry/attached-thread distinction and both positive controls. Preserve floor-only substitution with real forwarding on successful calls. These setup failures are different from classified exhaustion and must not be accepted merely because some resource record appeared. One build already serves its process arguments; no per-row rebuild is needed. |
| B03z: foreign faults, classification band and external signals (3) | Three C images: a wild write outside the stack; a fixture-owned stack over a deliberately unmapped pad, faulted at page/2, one page, four pages, 64 KiB and 16 MiB below it; and a process-delivered SIGBUS followed by a marker and an irrelevant 1,000-level C recursion. The first two check exact signals/record presence. The external-signal case requires any signal, no record and no completed marker, but not SIGBUS specifically. | Move these real host/floor observations into the same native group, with fresh child processes for fatal cases. Keep genuine page faults and foreign-signal disposition; a pure classification helper cannot replace signal delivery/alternate-stack integration. Add aligned near-boundary observations for the actual probe-stride/red-zone policy, rather than inferring its exact edge from sparse distances; deduplicate numerically equal offsets on hosts where four pages is 64 KiB. Make external delivery to the intended test thread explicit and require the original signal, so later failure is not a substitute. Remove the trailing recursion as evidence for that signal property. |
| B03aa: shared resource-record latch (2) | A C body writes 1 into the shared latch, schedules `alarm(2)`, and requests a 400-million-level recursion. Rust only requires empty stderr and excludes SIGABRT: normal exit also passes. It never proves the handler reached the occupied-latch branch. A separate IR-only parallel module checks that the heap abort writer requests the floor's latch and that the fixture emits a thunk and abort edge. | Keep the compiler's shared-latch reference assertion; move the C mechanism check into the native floor group. Use a controlled in-band fault and an observed occupied-latch handler path, with bounded parent/child coordination to terminate the expected waiting loser. Do not count ordinary return, an unrelated signal or a timer firing before the handler as success. Preserve a real winning writer/one-record check and confirm the emitted heap writer and floor use the same linked latch. The timer can bound a failed test, but elapsed time or silence alone cannot be its success oracle. |
| B03ab: allocation refusal and target qualification (3) | Two IR cases separately compile the same four-form source: filled buffer, vacant buffer, box and legacy arena allocation. Check no dynamic target-domain guard for proved layouts and that every null-allocation branch calls the resource abort. A native fixture asks for 4,000,000,000,000,000,000 bytes through an argument-dependent read, then requires a heap record and signal termination. The request is intended to be refused, not to fill that amount of memory. | Keep compiler-specific target/abort-edge evidence and share the four-form emission. Preserve all currently implemented allocation forms until their language/implementation retirement, not merely because a comment calls them retiring. Replace the enormous-request dependency with a scoped allocator refusal on the generated allocation path, using the existing native-observer pattern and a small observable source. Establish the actual allocation/refusal was reached, exact heap record and abort disposition; do not accidentally refuse runtime startup allocations. A C-only abort call would not cover the compiler's null-result branch. |
| B03ac: generated recursive and acyclic cleanup structure (3) | Emit box and buffer cycles at depth four and find a back edge in the drop-call graph, while rejecting old worklist symbols and release-path abort/realloc. The acyclic case checks only that no helper calls itself directly, missing mutual recursion. The buffer-order case re-emits the depth-four buffer source, checks an ascending index and that an element load appears textually before the block free. | Keep compiler cleanup observations and share the small buffer emission. Reuse the actual graph walk to rule out all cycles in the acyclic fixture, not only direct self-calls. Check the element release action and loop-to-block-release control relationship, not only textual load order. State no hidden allocation/worklist as the protected property; a broad `wf.drop.run` name ban must not reject the current legitimate store-run helper if a fixture changes. Preserve cyclic release as permitted by PROV-6 and keep machine-stack reporting as separate ledger evidence. |
| B03ad: cleanup acceptance and executed reclamation (4) | Run a 10,000-level boxed spine, a minimal buffer-cycle program, a one-million-level buffer chain, and a four-child buffer ownership tree. The first three check exit zero and empty stderr, not which allocations were released. The fourth asks the host allocator for `MallocScribble`/`MallocPreScribble` and checks only exit zero, without establishing that poisoning was enabled. The million-level case still claims to prove stack-independent cleanup after recursive release replaced the worklist. | Put the minimal normative cycle-admission requirement in the conformance family if its precise property is missing; reuse existing coverage when established, without automatically adding another native run. Keep compiler-observed box/buffer cleanup with distinct pointers and exact release/ordering/lifetime assertions, using a shared allocation/release observer and small branching/nested values. Require no missing or duplicate release and backing survival through element cleanup; normal exit alone cannot establish these. Replace the million-depth worklist-era claim and unsupported host-poisoning oracle after the stronger checks detect their target defects. Retain deep/resource behavior only for a named regression with demonstrated discriminatory depth, not as an unbounded-cleanup guarantee the current language does not make. |

**Native consolidation and fatal-case isolation.** The five C-only wrappers
have no private compiler observation and belong with the previously selected
runtime C runner. They can share one entry point and valid immutable
construction, with a separate substituted-floor build only where required.
They cannot simply run all bodies successively in one process: some cases
deliberately terminate it, others install process-wide dispositions or take a
one-shot latch. The runner must select cases in fresh child processes and
interpret exact status, signal and channels. Preserve host-specific
protection signals and real alternate-stack attachment. Bound hangs at the
process/phase level without treating a timeout as an expected WF verdict.

The common `assert_resource_record` exact-byte comparison already implies
its subsequent line-count/absent-field checks. One shared exact-record oracle
plus the appropriate process disposition suffices; this simplification must
not turn an unexpected signal into a successful exhaustion observation.
Tests may fix the shipped record bytes as an implementation regression,
but SCOPE-3 does not fix them as a source-language outcome. Likewise, the
comment that signal safety or PAR-1 alone forces these bytes is not a
normative ground: current resource outcomes are explicitly outside the source
outcome model. The record contract's owner is the runtime design decision.

**Adjacent evidence is not silently retired.**
`compiler/src/backend/tests/stack_ledger.rs` is outside the sampling
selector and has four tests. One actually emits machine stack-usage reports
at two frame widths, then builds/runs depths just inside/outside the reported
ceiling; it also imports `spine_source` and `assert_resource_record` from
the current module. Its report-to-machine comparison is a different property
from a bare exhaustion report. Changing the shared helpers or floor-size
test configuration must preserve that dependency. This four-test group is
the next B03 supplement before B04; its inclusion in `test-unit` must not
hide its native construction and real exhaustion. The compiler-side
`emitter/floor.rs` constant-agreement unit is a further referenced boundary,
not a twenty-fourth exhaustion case.

**Authority and implementation boundary.** Read current SCOPE-3, PROV-6 and
STOR-3 together with `design/compiler/resource-exhaustion-floor.md` and
`cleanup-traversal.md`. PROV-6 permits value-depth recursive release and
STOR-3 fixes element-before-backing order; resource shortage is not source
rejection. The legacy allocation spellings used here remain in the active
specification. The module's earlier retirement prose grants no permission
to delete their refusal coverage or to substitute a fallible provider
operation with a different outcome.

Actual guard faults, runtime setup failures, generated probe containment,
shared-latch integration and observed reclamation protect distinct
correctness properties. Keep them in the automatic correctness gate under
their receiving owners. The proposal replaces redundant construction and
unsupported success criteria, not all failing-child tests with source-text
inspection. Fatal-case core-dump handling remains the existing local/CI
runner's responsibility; this audit neither changes host settings nor
produces a core. No new timing measurement selects any recommendation.

**Complete function mapping.** All 23 current tests in
`compiler/src/backend/tests/exhaustion.rs` appear once below.

**B03v (3 current tests)**

- `every_generated_definition_carries_the_stack_probe`
- `an_ordinary_frame_emits_no_probe_call`
- `a_frame_larger_than_the_guard_region_is_still_reported`

**B03w (2 current tests)**

- `the_entry_runs_on_a_stack_the_compiler_sized`
- `a_deep_recursion_completes_at_every_worker_count`

**B03x (2 current tests)**

- `an_exhausted_entry_writes_one_resource_record`
- `an_exhausted_lane_writes_the_same_resource_record`

**B03y (1 current tests)**

- `floor_setup_refusal_stops_before_unprotected_execution`

**B03z (3 current tests)**

- `a_fault_that_is_not_exhaustion_keeps_its_own_disposition`
- `only_a_fault_within_the_probe_stride_is_read_as_an_exhausted_stack`
- `an_externally_delivered_signal_does_not_disarm_the_floor`

**B03aa (2 current tests)**

- `the_floor_and_the_module_share_one_record_latch`
- `a_module_that_writes_a_resource_record_and_hands_a_call_out_is_latched`

**B03ab (3 current tests)**

- `an_allocation_the_host_refuses_writes_one_resource_record`
- `target_qualified_buffers_keep_only_the_heap_refusal_path`
- `every_allocation_refusal_edge_reaches_the_resource_abort`

**B03ac (3 current tests)**

- `a_cyclic_release_graph_lowers_to_one_release_action_that_enters_itself`
- `an_ownership_chain_keeps_its_straight_line_drop`
- `a_buffer_in_a_cleanup_cycle_is_walked_in_the_order_the_rule_fixes`

**B03ad (4 current tests)**

- `a_deep_boxed_spine_is_reclaimed_without_a_record`
- `a_cleanup_cycle_through_a_buffer_is_accepted_and_runs`
- `a_deep_cleanup_cycle_through_a_buffer_is_reclaimed_without_a_record`
- `a_buffer_block_outlives_the_elements_the_traversal_takes_from_it`

The owner subsequently accepted B03v-ad. No implementation is claimed.

### B03 — Stack-ledger evidence, fifth part

**Scope and construction.** The four `#[test]` functions in
`compiler/src/backend/tests/stack_ledger.rs` share the same compiler library
test executable as the previous modules. Two run only Rust over supplied
strings/constants; one compiles a WF ownership fixture to assembly and reads
a machine stack report without executing a program; one obtains reports for
two recursion shapes and builds/runs their predicted depth boundaries.
They all currently run in `test-unit`, locally through `make check` and in
Linux/macOS CI's unit job, because this module is outside
`SAMPLING_MODULES`. That stage name does not mean there is no Clang
construction or child-process execution.

The boundary case imports the narrow `exhaustion::spine_source` and defines
`wide_frame_source`, whose nonuniform `FixedVector<u64, 256>` stays live
across recursion and whose recursive result selects a later element. Those
features matter: an optimized-away wide frame would remove the intended
second geometry. The cleanup fixture `RECURSIVE_VALUE` uses ordinary
`Heap<'s>` and `Box<'s, Tree<'s>>`; it is not the legacy lowercase
`box<T>` fixture in the preceding group.

`ledger_lines` writes LLVM, invokes Clang
`-S -fstack-usage -O2`, and reads the resulting `.s` and `.su` from that
same invocation. The Rust `stack_ledger` function parses the frames and
post-optimization call graph, adds the target's return-address correction,
then renders frame, cycle and bounded-chain rows using a supplied stack
capacity. The report is developer output, not a source proof or acceptance
condition.

**Current construction counts, not timings.**

| Current case kind | WF library compilations | Clang assembly/report constructions | Native executable constructions | Executions of those programs |
|---|---:|---:|---:|---:|
| Two Rust-only checks | 0 | 0 | 0 | 0 |
| Depth boundary at two frame widths | 6 | 2 | 4 | 4 |
| Derived-cleanup report | 1 | 1 | 0 | 0 |
| Total for these four tests | 7 | 3 | 4 | 4 |

Counts describe a normal passing traversal of the source. They do not count
the already-built Rust test binary, the first construction of shared runtime
objects, or work in other test modules. The boundary case obtains its report
at source depth 1,000, then emits two new source modules per shape with the
predicted inside/outside depths embedded as different constants. Its header's
four-program description counts executable builds, not all WF compilations.

**The owner agreed to all four rows, with implementation deferred.**

| Row and current count | Actual operations and observations | Recommendation and home |
|---|---|---|
| B03ae: per-activation arithmetic (1) | Supply one synthetic 64-byte static frame and a self-call assembly string to the real ledger. At a 4,096-byte stack, require 72 bytes/activation and 56 levels for x86-64, versus 64 bytes and 64 levels for Arm64. Check both the frame row and cycle calculation. No WF, Clang or program process is involved. | Keep this direct regression for the return address omitted by x86-64's stack-usage figure. Place it with the existing six Rust-only ledger tests in `backend/stack_ledger.rs::tests`, preserving both architectures on every test host. Their world labels, graph parsing and chain boundaries do not fully replace this two-architecture arithmetic check. It needs no new executable or native run. |
| B03af: selected architecture (1) | Compare `Architecture::HOST` with a match on `std::env::consts::ARCH`. Both are facts about the Rust compiler's build target; this does not query the physical CPU or independently inspect the emitted machine ABI. | Retain as a small mapping assertion in that same Rust group, and correct the independent-machine-oracle wording. It can catch a wrong enum mapping without being an independent hardware measurement. Actual host codegen/ABI evidence remains with the native tests and B04. Moving it is organization within the existing library executable, not a saved native build. |
| B03ag: reported depth versus real exhaustion (1) | For narrow and wide frames, obtain a report at source depth 1,000 and calculate levels from the full shipped 1 GiB capacity. Recompile new sources at 99.9% and just over 100.1% of that reported level count, link four executables and run them. Inside must return zero; outside must emit the exact stack record. The helper does not require the expected abort signal, and success does not inspect the record channel. The reported machine code is not the code executed: changed depth literals create separate optimizer inputs without checking that frame size stayed the same. | Keep the quantitative report-to-machine regression in compiler native checks. Make depth a runtime input and use the same compiled WF machine code for report and both boundary executions, retaining the two genuinely different frame geometries. Obtain `.s`/`.su` together and link the measured code rather than silently reoptimizing a different module. Use a controlled smaller stack with independently established usable bounds and fixed entry overhead; preserve the shipped reservation check in B03w. Define the error allowance from that geometry before choosing outcomes, sufficiently discriminating to catch wrong per-level cost; do not copy the 1 GiB test's 0.1% tolerance blindly or widen it until a run passes. Require normal success with no resource record on the inside and the exact stack record/abort on the outside, plus a bounded process guard. Share the receiving entry-exhaustion fixture with B03w/x where equivalent, while retaining their distinct real-worker observation. |
| B03ah: compiler-derived cleanup appears in the machine report (1) | Compile a small store-backed recursive tree, obtain one assembly/stack-usage pair, and require some `wf.drop.` frame row and some `wf.drop.` cycle row. The program is never linked or executed. Its source has no written recursion; the recursion belongs to generated release. | Keep this compiler machine-report integration. Tie the expected cycle/frame to the relevant generated cleanup functions rather than accepting any unrelated matching substring. Preserve the store-backed type/cleanup path and one assembly construction. B03ac's LLVM graph assertion does not establish that the post-optimization machine report still contains that cycle, and B03ad's release trace proves a different property. Share construction only when the complete module and host options agree, not because two fixtures both contain a function named `wf_spine` or a recursive tree. |

**Why the boundary change is needed.** The current comparison assumes that
changing a depth literal does not change native optimization, although the
test's own rationale says unrelated source changes can alter frame width.
A failure can therefore mix a wrong ledger with a different emitted frame;
a passing pair does not establish that the measured frame belongs to either
executed image. One runtime-parameterized module per frame shape removes
that uncontrolled difference and allows one native image to serve both
executions. The intended reduction is in repeated construction; both sides
of the boundary and both frame geometries remain necessary observations.

A smaller controlled stack also changes how important fixed caller frames,
thread setup, alignment and guard pages are. The replacement must account
for them explicitly and establish that its recursive machine frame survives
optimization with the stated static cost. It must retain sensitivity to
the actual model defect, including a missing return-address contribution,
rather than declaring a smaller workload equivalent solely because it ends
with the same signal. No concrete stack size, tolerance or measured speedup
is selected by this source-only discussion. The current report excludes
runtime C frames; the narrow/wide self-recursions do not prove an arbitrary
call graph's whole-thread stack bound.

**Existing adjacent evidence and shared helpers.** The six Rust-only cases
in `compiler/src/backend/stack_ledger.rs::tests` cover separate sequential/
parallel cycles, budget variants, absence of cycles on acyclic functions,
bounded-chain termination at recursion, frame-row coverage and ELF PLT call
resolution. They are compatible with the two pure checks above and require
no WF compilation. Their embedded strings are data, not executed assembly;
keep their distinct observations without adding native copies.

B03h already selected one ordinary and one parallel stack report for clone
equality and budget-frame overhead. Those callers use this module's
`ledger_lines`/`reported_frame_bytes`; put shared construction under the
existing compiler test support and preserve their exact module/option
identity. The current `emitter/floor.rs` Rust/C stack-constant agreement
check and the B03w actual-thread observations retain the separate default
capacity boundary. None of these makes the measured-depth comparison
redundant.

The CLI's `whitefootc.rs::print_stack_ledger` likewise collects `.s` and
`.su` together, but its ordinary native link is a separate operation on the
same LLVM text. The current library tests exercise the ledger function,
not CLI option parsing, stdout/channel selection or its error paths. Those
existing CLI/tool checks retain their B07 review home; this audit creates
neither a new command-line option nor another CLI integration executable.

**Placement and authority.** Keep pure Rust ledger checks in the compiler's
Rust unit group. Report generation and actual depth probes remain compiler
native checks with construction and execution explicit in the eventual
caller map. Retain them in the automatic correctness gate: this is checking
developer-report accuracy and a resource regression, not printing an
exploratory timing score. All can remain in the existing library executable;
no test-target split follows from these four property groups.

The grounds are `design/compiler/resource-exhaustion-floor.md`'s completed
host-codegen ledger decision, `cleanup-traversal.md`'s derived release and
`parallel-lowering/two-worlds.md`'s separate frame costs. SCOPE-3 and STOR-6
keep available stack outside acceptance and distinguish it from mandatory
target representability. No language rule or runtime default is changed.
B04's actual-platform adapters, host callers and instrumentation are next.

**Complete function mapping.** All four current tests in
`compiler/src/backend/tests/stack_ledger.rs` appear once below.

**B03ae (1 current test)**

- `a_row_is_what_one_activation_costs`

**B03af (1 current test)**

- `the_hosts_architecture_is_the_one_the_machine_reports`

**B03ag (1 current test)**

- `the_reported_ceiling_is_the_measured_one`

**B03ah (1 current test)**

- `the_compilers_own_drop_glue_has_rows_and_reports_its_cycle`

The owner subsequently accepted B03ae-ah. No implementation is claimed.

### B04 — Native adapters and platform check wiring, first part

**Scope and construction.** These are direct C runtime checks and their
Make/CI callers, not additional Rust `#[test]` executables or WF cases.
This part audits all platform branches of
`compiler/src/backend/completion/native_adapter_probe.c`,
`native_contract.{c,h}`, the Windows bridge initialization failure probe,
and the corresponding native/sanitizer/cross/Wine recipes.
`windows_namespace_probe.c` and the later Windows WF program steps in
`.github/workflows/io-hosts.yml` remain the next part of B04. Listing their
builds here does not claim to have reviewed their assertions.

| Current artifact | Construction and resources | Current execution/caller |
|---|---|---|
| Linux `linux-native-probe` | Host C11 `-O2 -g`, strict warnings and pthreads; `sched/{core,prim_host,entry}.c`, `completion/{runtime,wait_host,linux_io_uring,native_adapter_probe}.c`. Requires a real usable io_uring instance, scratch files/FIFO, loopback sockets, eventfd/epoll and native threads. | `completion-test`, reached through local `make check`, builds/runs once on Linux; startup refusal 77 is allowed locally. Linux `io-hosts` separately builds/runs the same probe as required evidence, then invokes `completion-test`, which builds/runs it again. |
| Windows `windows-adapter-probe.exe` | C11 `-O2 -g` with strict warnings; scheduler core/Windows leaf/entry, completion runtime/Windows wait/native-contract/IOCP, `windows_runtime.c` and the same probe source, with Winsock linkage. Requires a real Windows IOCP, files/overlapped handles and a thread. | Windows `io-hosts` compiles and executes the Windows branch. Explicit cross/Wine targets separately build/run that branch for development. This is one platform-selected `main`, not all branches in one execution. |
| Windows `windows-bridge-init-fail-stop-probe.exe` | First compile `bridge.c` with its two engine initializer calls and final abort renamed by the probe prelude; then link the other nine Windows runtime units and probe with that object. | One fresh process in real Windows CI; check exit 86, empty stdout, a whole-line match for the shipped initialization-failure diagnostic and one newline in normalized stderr. It must not return from submit, which would yield 87. |
| ASan/UBSan variants | `completion-sanitize` builds core/read, default-route and harness images using `-O1 -g -fsanitize=address,undefined`; each retains its own ordinary/scripted-hook source set. | Three image constructions and three executions in the Linux CI recipe. Default policy is unset for its probe; harness defaults to one helper. All three use `ASAN_OPTIONS=detect_leaks=0`. |
| TSan variants | Core/read, default-route and harness at `-O1`; deque at ordinary `-O2`, separately with stats on/off; all add `-fsanitize=thread`. | Five image constructions and seven executions: one core/read, one default-route, two deque and three harness helper settings 0/1/4. These are Linux CI/explicit targets, not current local `make check` prerequisites. |

Counts above describe recipe construction/execution, not elapsed times or
new measurements. A failure can stop a recipe before its later commands.
ASan/UBSan and TSan change the generated native artifact; they cannot reuse
uninstrumented objects as though flags did not matter. No Cargo profile or
WF optimization option is selected by these C compiler flags.

**The owner agreed to all nine rows, with implementation deferred.**

| Row | Actual checks and gaps | Recommendation, home and stage |
|---|---|---|
| B04a: self-declared capability table and B02b's missing consumers | `native_contract.c` returns fixed platform flags; `harness.c::test_native_contract_inventory` and the Windows adapter probe assert those same flags. No production caller was found. A renewed current-source search found completion `WF_FILE_PWRITE` requests only in the harness and Linux adapter probe, and standalone `WF_FILE_STATUS` requests/bridge calls only in the harness; current programs and active experiment sources supply no other caller. | Retire the table, its two self-check sites and otherwise-unused build entries. Actual native operations and required-host callers below provide platform evidence. Resolve B02b's conditional retirement in favor of removing the orphaned positioned-write/status request/API machinery and their isolated assertions together, subject to the usual implementation-time affected-consumer check. Preserve ordinary stream writes, host `pwrite` used to seed fixtures, and the required open-time `fstat` kind check. Do not retain unused runtime operations merely to give their tests something to test. |
| B04b: real native transfer and typed acquisition | Linux submits two positioned reads of an eight-byte file at offsets 0/4, checks both result values/errors and exact bytes; its TCP fixture submits connect/accept then send/receive, checking a real loopback peer and `ring!` payload. Eight open/close attempts cover regular success/close, second-close `EBADF`, missing name, directory/FIFO refused as regular with their returned descriptors already closed, and directory success/close. Windows writes `iocp` with a separate synchronous handle, associates an overlapped read handle with IOCP, and requires its read result/bytes and zero final in-flight count. One cached read does not establish both immediate and deferred host completion paths. | Keep the distinct native-engine ownership/result/disposal observations in the common C runtime organization, selected on the actual Linux/Windows host. Share fixtures and expectation rows with B02 and the default-route cases where their complete obligations agree; do not replace direct-engine assertions with fallback file success or add a WF copy. Fold the separate Linux borrowed-path open into the existing open fixture: its current pointer assertion only reads the request field, not the SQE address it claims to inspect. Observe the actual native submission if retaining that stronger claim. A second close without intervening descriptor reuse proves `EBADF`, not protection of a reused descriptor; the latter retains its existing lifecycle home. |
| B04c: submission pressure and completion overflow | A ring created with SQ depth 8/CQ depth 16 receives 12 one-byte reads before terminal draining; each must be accepted and return its selected byte. A separate 24-read batch is explicitly flushed, then nonblocking progress must reap all 24 and increase the overflow-flush counter. It addresses completions retained by the kernel beyond the mapped queue, not merely a large request count. | Keep both engine boundary regressions in the Linux native group. The 12 submissions do not establish 12 simultaneously outstanding kernel operations, so name the actual admission/progress property. Preserve evidence that the overflow path was exercised, exact result association and complete draining. Share setup with other native reads and B02d's bridge-pressure observation where compatible; the bridge's 96 requests alone do not replace the 24-read kernel-overflow assertion. No throughput benchmark or arbitrary repetition is needed. |
| B04d: actual native wait/wake integration | Linux checks an epoch change before park without an eventfd write; a real announced sleeper woken by compute notification, followed by eventfd drain; a 100 ms io_uring timeout completion waking through the ring descriptor without an eventfd write; and four real waiters receiving distinct synthetically published record values without consuming another waiter's broadcast. Windows starts with an empty IOCP so a file completion cannot hide a lost helper wake, observes one real wait/post and the exact published value 37 before its file test. | Keep these host wait integrations; B01's generic publication/wake checks do not replace eventfd/epoll/IOCP behavior. Reuse the logical waiter/result helpers, retain fresh empty-port and re-park premises, and explicitly fail if a required announcement was not observed. Bound both phase progress and the whole child process. The Linux 100 ms completion has a concrete kernel-wake purpose; do not delete it as a gratuitous sleep. A controlled pending native operation may replace it only while retaining an observed native-CQ wake and the no-eventfd assertion. |
| B04e: engine error and initialization failure | Linux stores `EIO` directly in `adapter.progress_error`, then checks progress and park return it, with zero publications. This checks sticky-error propagation; it does not induce a kernel failure or execute the bridge's post-acceptance fail-stop. Windows makes both IOCP and helper-adapter initialization fail and observes the production diagnostic plus the renamed terminal abort. Failure of the ring alone is already a successful fallback case elsewhere. | Keep the small Linux component assertion and the Windows no-engine failure case with honest scope. Retain the Windows bridge-only object variant and fresh child process, reusing unchanged compatible runtime objects; do not rename every object's initializer or test the probe stub in place of the production failure branch. Pin the startup conditions needed to reach both injected calls. Preserve exit 86/empty stdout, compare complete normalized stderr rather than only a matching line/newline count, and bound execution. Neither case proves the other failure boundary. |
| B04f: native Make/CI callers | Linux CI duplicates the native-probe build/run. On a ring-capable host, `completion-test` also runs the whole harness with required io_uring/helpers 1, and CI repeats that setting among helpers 0/1/4. Other whole-harness repetitions cover no-cache or startup helper settings even for unrelated cases. Windows has real native and forced-fallback default-route runs. | Have local and CI callers share construction and case selection. Build the ordinary artifact once per actual input/flag set and run each required configuration's relevant groups once. Preserve an explicit local-unavailable result and make unavailable/refused native capability a failure in the Linux qualification job; never relabel fallback as native success. Keep both real Windows routes and earlier selected helper/default-policy distinctions. The supported host's ordinary runtime checks belong in the automatic correctness path; only observations that need another kernel move to that host's CI job. Final repository-wide entry-point wiring remains B07. |
| B04g: sanitizer selection and verdicts | ASan/UBSan instrument memory/undefined operations; TSan observes real threaded executions. The current sanitizer recipe repeats already distinct C images, disables leak detection, and supplies no `-fno-sanitize-recover=undefined` or UBSan stop-on-error setting. Most UBSan checks otherwise report and continue. Core/read is single-threaded despite its separate TSan image; deque stats on/off and bridge/helper races are genuinely different configurations. | Keep ASan/UBSan and TSan as separate required supported-Linux CI configurations of the receiving C test organization, available explicitly locally. Run memory/UB assertions on the relevant cases and TSan on the actual concurrency groups; apply R01's already-selected retirement of single-thread core/read TSan and unsupported stress repetition. Retain needed default/native/helper and deque macro distinctions without multiplying every case by every setting. Explicitly make a sanitizer finding fail the job, for example with `-fno-sanitize-recover=undefined`; do not silently discard diagnostics or claim leak coverage while it is disabled. Instrumented native-route evidence must state which route actually ran. |
| B04h: cross-link, syntax and obsolete symbol guards | `completion-windows-cross` constructs three PE executables (default route, native adapter, namespace), floor/Windows-runtime objects and nine more objects for an `nm` dependency comparison. It requires imports including `CreateFiberEx`, `SwitchToFiber`, `ConvertThreadToFiberEx`, although current runtime sources have no such calls. Its floor-only dependency premise conflicts with `whitefootc::runtime_units`, which always links the complete ordinary runtime. `pure_compute_probe.c` is a standalone C arithmetic function deliberately linked to no runtime, not a WF compiler link. On non-Linux hosts, the extra io_uring syntax command sees a guarded header and one typedef. Windows CI also strict-checks 12 C runtime units after already strict-building many of them. | Retain a shared explicit Windows cross-build aid for current compile/link errors, with no claim of Windows execution. Retire the stale import whitelist and floor-only `nm` construction/guard, plus the unrelated pure-C zero-runtime probe. Remove the non-Linux empty io_uring syntax pass; ordinary construction already compiles that guarded unit. Fold strict diagnostics into shared real Windows object construction; remove the extra pass only after every unit/configuration is covered, especially `wf_floor_windows.c`, which the first default-route/ordinary probes do not build. Real-host execution and actual complete-runtime linkage remain required evidence. |
| B04i: Wine development caller | `completion-windows-wine` first builds every cross target, then checks whether Wine exists. It runs the adapter once and the default-route image twice, requiring route labels/loopback port and zero TCP ring count on forced fallback. Native-route TCP ring count may be zero and is explicitly not real Windows evidence. Namespace is built but not executed. The two route processes feed `tee` pipelines without an explicit check of their own exit statuses. | Keep Wine as an optional development aid, outside required Windows correctness qualification. Check tool availability before unnecessary construction, request only the artifacts actually run, preserve both useful route runs and propagate each probe's own exit status as well as checking its outputs. Correct stale fiber descriptions. Real Windows CI remains the owner of Windows kernel/runtime evidence; a Wine pass or skip cannot satisfy it. |

**Why these findings differ from blanket deduplication.** Native-adapter
probes supply their own strong `wf_completion_record_complete`, publishing
through a probe-owned runtime. Linking them unchanged with `bridge.c` would
conflict with the bridge's strong definition; their native record fixture
does not become a bridge integration merely by sharing a `main`. Use the
already-selected scoped-hook organization where compatible, or retain a
justified isolated-publication variant with shared case support and immutable
objects. The Windows bridge-failure build is another genuine object variant.
Sanitizer variants and host leaves also remain separate constructions.
None needs a new Rust integration executable or a duplicated WF fixture.

Conversely, a hard-coded capability flag is not a measurement of native
completion, and importing a particular old API is not the current runtime's
behavior contract. The source audit finds no active fiber calls; current
`parallel-runtime.md` explicitly selects persistent native worker threads
and ordinary stacks. The CLI always links the complete runtime. Those are
the grounds for retiring the specific historical guards, while keeping
real native waits, strict builds and actual host execution.

**Error policy and process bounds.** The Linux probe currently has blocking
progress loops and `UINT32_MAX` parks followed by unbounded pthread joins;
several helpers instead count 100,000 progress attempts or 1,000,000 yields.
Those loop counts do not establish a scheduling premise and are not a
portable elapsed-time bound. Windows bounds its empty-port waiter but not
the later file-progress loop. Apply the already-selected observed-state and
bounded-process method to these cases as well, reporting the failed phase
and terminating/reaping its owned children. No guard selects a language
acceptance verdict or converts an incomplete probe to a pass.

The [Clang UBSan usage documentation](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html#usage)
specifies report-and-continue as the default for most checks and describes
`-fno-sanitize-recover` for reporting then exiting. The missing verdict
setting is therefore a source-configuration gap, not a claim that a new
sanitizer failure was observed. Leak detection is currently explicitly
disabled; existing exact per-resource credit/lifetime tests retain their
own narrower assertions. This proposal does not introduce a speculative
leak-testing project or change sanitizer findings into allowed failures.

**Audited source-to-row map.** This map records logical observations, not
nine new executables or nine Rust test functions.

- `harness.c::test_native_contract_inventory`,
  `native_contract.c::wf_completion_target_contract_for`, Windows adapter
  metadata assertions, and the B02b/Linux positioned-write/status callers:
  B04a.
- Linux `main`'s positioned-read results and
  `probe_open_and_close_cases`, `probe_open_names_the_submitters_bytes`,
  `probe_loopback_round_trip`; Windows `main`'s overlapped transfer and
  final drain/destruction: B04b.
- `probe_more_in_flight_than_the_ring_is_deep` and
  `probe_completions_past_the_queue_are_flushed_from_the_kernel`: B04c.
- Linux `main`'s four wake blocks and their park/announcement/timeout/
  record-wait helpers; Windows `probe_wait_for_helper` and the empty-port
  block: B04d. The four Linux records are manually completed; they are not
  four additional kernel file reads.
- Linux `main`'s sticky-error block and
  `windows_bridge_init_fail_stop_probe.c`'s prelude, injected initializers,
  terminal abort and `main`: B04e.
- Make `completion-test` and the corresponding Linux native/harness and
  Windows default/native build/run steps: B04f, applying earlier R/B rulings.
- Make `completion-*-sanitize`, `completion-sanitize`,
  `completion-*-tsan`, `completion-tsan`, `sched-deque-tsan` and the
  already-reviewed core/read stress target: B04g.
- Make `completion-windows-cross`, `pure_compute_probe.c`, non-Linux
  io_uring syntax command and the Windows CI strict-runtime step: B04h.
- Make `completion-windows-wine`: B04i.

Record initialization, terminal driving, open/close construction and the
probe's two platform-specific publication definitions are shared support for
the mapped assertions. The unsupported-platform `main` only returns 77;
it is not an additional platform test. Native namespace assertions and the
Windows WF/source-compiler/ABI/floor callers are still unreviewed in this
part. Their earlier accepted component obligations remain selected while
B04's next part audits their additional observations and repeated callers.

The owner subsequently accepted B04a-i. No implementation is claimed.

### B04 — Windows namespace and compiled programs, second part

**Scope.** This part covers the complete
`compiler/src/backend/windows_namespace_probe.c` and the six WF execution
steps at the end of `.github/workflows/io-hosts.yml`'s Windows job. The earlier
deque, CPU-leaf, startup, ordinary-values, default-route, direct-adapter and
bridge-failure steps retain their B03/R/B04a-i rulings; their Windows callers
have been accounted for, not selected for a second independent suite.

**Current construction and execution, not timings.** The workflow first
builds the native Windows compiler with
`cargo build --manifest-path compiler/Cargo.toml --profile gate --locked
--bin whitefootc`. That constructs `compiler/target/gate/whitefootc.exe`;
it is not a Rust unit-test build. The six steps then use that compiler to
check/lower WF and normally invoke Clang at the project's fixed `-O2` to
link the emitted module with the ordinary library and Windows runtime.
`--par` selects compute-overlap lowering, not an optimization level.

| Current input and location | WF compilations | Native images constructed | Executions on a normal passing path | Oracle/resources |
|---|---:|---:|---:|---|
| `compiler/src/backend/windows_namespace_probe.c` | 0 | 1 C-only image | 1 | Strict C11 `-O2 -g`; dynamically resolved NT calls, temporary directories/files, UTF-16 names, changed process cwd and a created symbolic link. Links no Whitefoot runtime. Real Windows CI requires the symlink fixture; exit 77 is failure there. |
| `tests/programs/completion_read_boundary.wf` | 1 | 1 | 1 | Two one-byte files A/B, argument paths, exact stdout `AB`, successful status and empty diagnostics; IOCP-required environment setting. |
| `tests/programs/tcp_echo.wf`, `tests/programs/tcp_refused.wf` | 3 | 3 | 5 | Echo default/parallel and refusal default; echo runs on normal IOCP, parallel IOCP and forced fallback; refusal runs on both routes. Controlled .NET peer, 10,000 known bytes, half-close/EOF, exact echo, expected refusal statuses and empty stderr. |
| `tests/programs/host_string_bytes.wf` | 1 | 1 | 1 | Actual process argument U+4241; the program requires two raw bytes 0x41/0x42, byte length/copy endpoint two, status zero and no output. |
| `research/experiments/io-completion-bench/programs/windows_component_open.wf` | 2 | 2 | 2 | Default and `--no-overlap` builds; argument names U+4241/U+4242 name files containing W/X, while ASCII AB/BB decoys contain D/E. Both outputs must be WX; only the second run sets the IOCP requirement. |
| `tests/programs/par_layout.wf` | 3, one emits LLVM only | 3 | 4 | Driver-built default and `--par` images, plus manually linked `--par` LLVM with `sched/grant_observer.c`. Compare outputs; reject malformed workers before program output; require observed `grants>0`. |
| `runaway.wf` written inside the workflow | 1 | 1 | 10 | One `--par` recursion with source depth 100,000,000; five runs each with default workers and workers 4. Nonzero status, empty stdout and one matching stack-resource record. |
| Six WF steps together, excluding the C-only row | 11 | 11 | 23 | Excludes building the Rust compiler, preceding C probes and extra diagnosis after failure. |

The parallel failure branch can additionally build/run an unobserved manual
control, build/run an ASan image and rerun under `cdb` if available. Those
commands diagnose a failed run; they cannot make it pass and are outside the
passing-path counts. No command in this inventory was executed for this audit.

**Owner decision: agreed.** B04j-r are selected for later implementation.
Migration, retirement, gate rewiring and timing work remain deferred.

| Row | Actual evidence and gap | Recommendation, home and stage |
|---|---|---|
| B04j: namespace probe's private policy and bookkeeping | The standalone probe implements its own component validator, open wrapper, directory decoder and six-handle close tracker. Eight rejection rows exercise that private validator: empty, dot, dot-dot, slash, backslash, drive prefix, UNC prefix and embedded NUL. It does not call `ordinary_values.c` or `windows_runtime.c`. Its policy/limits also differ from the shipped component and relative-path entry points; for example it rejects dot/dot-dot itself, while the production relative-path validator allows them. Its close counters count its own wrapper's `CloseHandle` calls. | Retire these parallel implementations/self-checks when the useful host observations in B04k have a receiving home. Test current component byte validation and result construction through the ordinary C library, sharing R04/R05's group. Do not copy the eight verdicts wholesale into conformance or the runtime: component names and `RelativePath` are distinct inputs, and the active rule/library obligation must select each expectation. The old probe's validator is no authority for a new refusal. Production resource accounting belongs with R05's real per-resource checks, not this private close counter. |
| B04k: real directory-root and reparse observations | The probe takes a directory handle, changes cwd to an empty sibling, and then opens/enumerates non-ASCII names through the saved handle. It checks regular/directory attributes, missing-name error conversion, opening a symlink itself with no-follow and following it to a regular target. Its decoder uses `FileNamesInformation`; production uses `FileDirectoryInformation` and converts to the ordinary directory record. Merely finding the two expected names in the probe does not validate that production conversion. | Keep the root-versus-ambient-cwd, native-name, known-entry, missing-name and follow/no-follow distinctions in the shared Windows C runtime group, extending R05/R06's actual ordinary-library fixture. Exercise the shipped open/enumeration/close path and exact relevant results. In the no-follow component path, preserve rejection and disposal of a terminal reparse object; the probe's successful raw HANDLE to a link is not the ordinary API's expected result. Use real link creation, with required Windows CI failing if that fixture cannot be established. Isolate cwd mutation in a selected child or otherwise exclusive fixture. Retire the standalone 1,009-line probe and its dedicated executable/cross-build caller only after those observations run against production code. |
| B04l: argument paths through the shipped compiler | `completion_read_boundary.wf` destructures ordinary Inputs, constructs two `RelativePath` values, opens with `open_read`, reads one byte each and writes them in argument order. The default lowering is Off; two source reads do not establish simultaneous outstanding IO. This differs from the component-byte/`open_file` API below. | Keep one complete CLI-build-and-run program scenario in the programs organization, selected on Windows with exact AB/status/stderr checks and effective native-route evidence. Reuse existing sources/construction and common host orchestration; C-only IOCP success cannot establish emitted ABI, startup and driver link correctness. Describe it as ordinary file/argument/output integration, not proof of source IO overlap. Do not merge away its distinct `RelativePath` entry merely because another test also reads two files. |
| B04m: WF TCP on real Windows | The echo transfers 10,000 nonuniform bytes through a 4,096-byte program buffer, stopping the sender and requiring the peer to receive the complete echo. The refusal program attempts two connects using one factory and succeeds only on two `ConnectionRefused` outcomes. This proves continued usability, not exact credit restoration: the normal factory has ample remaining credits even if two were lost. Comparable program/oracle logic already exists in `compiler/tests/programs/network.rs`. | Keep real Windows program behavior on both engines and the `--par` construction combination, sharing program sources, payload/oracle logic and host process support with programs. Preserve compile/run differences only while the emitted paths or required observations differ. Do not claim the `--par` echo observed a stolen task merely from its flag. Keep exact factory-credit obligations in the constrained C tests already selected. Bound socket connect/write/read and child lifetime, clean up on every failure, and distinguish a released-port allocation race from the intended refusal. |
| B04n: HostString's complete native bytes | A real UTF-16 process argument crosses ordinary input initialization and the emitted callable/view ABI before `host_bytes_len` and `host_copy_bytes` are checked. R04's C representation tests do not by themselves exercise that complete path. The component-opening program already consumes those same actual process arguments and copies their bytes, but does not currently make all the standalone length/endpoint/byte assertions. | Merge the exact native length, copy endpoint and byte assertions into the receiving component-opening program/fixture in B04o, preserving the actual argument and relevant copy-window observations. Then retire the separate WF image/run and source only after its observations are accounted for, updating corpus collection too. Keep R04's distinct direct C encoding/buffer checks. This saves a duplicate complete construction without replacing a compiler/host boundary by a self-built C value. |
| B04o: copied component names and obsolete two-build control | The Unicode names and AB/BB decoys discriminate native code-unit bytes from accidental narrow-name interpretation. However, `Options::overlap` selects Off for both default and `--no-overlap`; both link the ordinary completion runtime. Neither build flag forces the helper route. The first run does not require IOCP but may use it, so the old direct-versus-completion explanation no longer describes two implementations. The source is also collected automatically by the IO bench Makefile's `programs/*.wf` wildcard. | Keep this useful host-interaction scenario under programs, with B04n's additional byte assertions. Use one construction and one effective required-IOCP execution for the existing observations, preserving the WX output and decoys. Do not retain duplicate builds as a claimed engine comparison. Update the Windows caller, corpus/canonical collection and the benchmark wildcard ownership together when moving the source; no duplicate file or benchmark-only correctness copy. A future different configuration needs a named missing observation. |
| B04p: layout output, actual workers and startup | The workflow builds a sequential program, a production parallel program and an observed parallel image; output equality and one aggregate steal count check different properties. Each ordinary invocation builds a depth-six tree (127 nodes), uses an 8,192-element table and repeats each of its two folds 800 times, with full-width and 4,096-element scans. Only the last value of each fold is published. Correctness CI measures no performance regression here. The current aggregate observer does not identify which fold was stolen. | Keep program-result equivalence and the Windows shipped-driver path in programs. Give correctness a bounded representative invocation of the same source; reserve benchmark repetition for its explicit performance consumer, reviewed in B06. Keep actual non-owner execution in the controlled compiler/runtime boundary selected by B03, on real Windows, with successful output as well as worker identity/path evidence; do not compensate for an uncontrolled handoff by rerunning the large workload. Reuse compatible emission/objects and the existing observer, retaining production-link evidence. Move malformed-worker startup checks into the shared startup group with an explicit before-entry observation. Compare complete output, not only equal/truncated streams or any matching grant line. |
| B04q: real Windows exhaustion | The inline WF source prevents trivial tail-recursion elimination and requests a depth well beyond the runtime's fixed stacks. The ten runs inspect no thread identity; default and workers 4 do not guarantee separate entry and worker overflows. A nonzero process status plus a matching line is weaker than observing the intended classified termination, and there is no per-run bound. | Apply B03w/x's shared entry/worker exhaustion design to the actual Windows floor: controlled smaller stacks with real Windows probing/guard and emergency-stack requirements, explicit entry versus non-owner handoff, exact resource output and verified termination, in isolated bounded children. Preserve the separate default stack-reservation observation. Replace the inline source and ten probabilistic large runs only after the receiver detects the protected floor defects. Do not substitute Linux signal expectations or a Wine pass for Windows exception behavior. Keep in compiler native checks; resource-record bytes and thread provisioning are implementation obligations, not a new conformance verdict. |
| B04r: host runner, required-route verdict and failure diagnosis | The Windows steps duplicate shell/PowerShell build/run/output code. Existing Rust program support uses `/usr/bin/clang` and POSIX runtime objects, so merely moving a test name there would not create Windows coverage. Several processes have no local guard; the TCP receive loop runs before its `WaitForExit(60000)`, so that wait does not bound a blocked stream read. Crucially, `wf_bridge_ring_start` registers the `WF_REQUIRE_WINDOWS_IOCP` exit verifier only after successful initialization. Initialization/refusal fallback can therefore omit that verifier entirely. When registered, it checks only nonzero aggregate submissions and matching completion count, not every operation's routing. | Centralize host-aware construction, per-child environment, byte-preserving output, deadlines and cleanup in the existing test support/caller organization; keep real Windows CI execution and at least the selected ordinary CLI boundaries. Make the required-native verdict fail on unavailable/uninitialized/unexercised IOCP as well as incomplete submissions, including a controlled refusal negative control reusing an existing program image. Preserve ordinary fallback without that requirement. Attribute per-operation route claims only to observations that actually distinguish them. Keep useful failure logs/artifacts automatically; move the extra control/ASan/debugger reproduction out of default failure handling into explicit diagnosis. It adds no passing verdict and its unbounded reruns can prolong the failure. |

**Namespace authority and migration.** The raw probe is useful evidence that
a host facility can be exercised, but its comment about a future target row
does not describe the current ordinary library. Its eight private validation
outcomes are not eight source-language rejections. PRE-1 provides ordinary
declarations/contracts; it does not adopt this C enum or select all of its
host policies. In particular, the existing relative-path tests preserve dot
components, while the standalone probe refuses them before reaching Windows.
Keep these API distinctions explicit and consult the current owner before
selecting any new behavior. This discussion changes no accepted WF program,
native library behavior, conformance expectation or specification text.

The receiving native fixture must actually call production code. Retaining
only the old probe's private decoder or close tracker in a shared executable
would preserve the defect in ownership of the assertion. Share the useful
Unicode/root/link setup with ordinary opens and directory enumeration, check
known names and cursor behavior through the real output format, and verify
the ordinary outcome/handle credit on failures. The old missing-file check
accepts any mapped error other than success or an unmappable status; it must
not be described as an exact `NotFound` assertion.

**The layout reduction has a concrete equivalence criterion.** Both folds
write each node's output field without reading its preceding value; widths,
heights and the word table are unchanged across repetitions. The caller
overwrites the two result variables and prints only the final pair. Each
800-iteration batch starts the seed at 16 and increments by exactly
representable 0.0625, so the final pass uses 65.9375. A short correctness
invocation can exercise that final seed and both full and caller-bounded
folds without traversing all preceding benchmark repetitions. Before
retiring those repetitions from correctness callers, verify the receiving
case's complete result against the established program oracle and preserve
the distinct permission/frame/actual-worker observations in their proper
groups. Repetition remains a benchmark parameter rather than a claim of
800 independent correctness cases. The benchmark script contains fixed
expected result bytes; this Windows CI step currently checks only equality
between compiled runs, so their provenance and use must remain explicit.
No reduced case or equivalence experiment has been executed here.

The namespace C test can join the selected common native executable; tests
that change cwd or terminate still need process isolation. WF scenarios can
share the existing programs orchestration while producing the native images
their distinct sources/options require. This does not imply that all WF
programs can become a single native `main`, require a new Rust test
executable, or preserve a separate shell implementation of each oracle.

B04h's strict-Windows-source obligation has a concrete existing receiver:
the later observed layout link strict-compiles all twelve runtime C units,
including `wf_floor_windows.c`, plus the observer. Preserve that full-unit
coverage in shared construction even if the observed program changes.
Likewise, the retired namespace executable's cross-build collection and the
component source's automatic benchmark collection are affected callers,
not files to leave dangling.

**Complete current caller map.**

- Namespace build/run steps: B04j/k; its explicit cross construction was
  identified in B04h and follows the selected replacement fixture.
- `Compile and run a real Whitefoot program through IOCP`: B04l/r.
- `Compile and run the TCP routes on both engines`: B04m/r.
- `Run the Windows HostString raw-code-unit boundary`: B04n.
- `Open target-native Windows components from compiler-emitted buffers`:
  B04o/n/r; also the IO bench `PROGRAMS`/`CHECKED` wildcard.
- `Require native workers for a real --par program`: B04p/r, including its
  invalid-setting run and failure-only control/ASan/debugger branch.
- `Require the floor to classify an overflow on an ordinary compute thread`:
  B04q/r, including its inline source and two-by-five execution matrix.

B04's selected inventory is now source-reviewed and agreed across its two
parts. B05's maintained research models and independent oracles follow below.
Corpus-wide case migration still follows the baseline;
this platform audit does not certify every program case or the remaining
research, benchmark and repository-tooling scopes.

### B05 — Container models and native controls, first part

**Scope and current callers.** This part covers the complete `authority/`
and `foundation/` subdirectories of
`research/experiments/container-representation/`, including their Makefiles,
all five Rust sources, `layout.c` and `large-result.wf`. Root
`make check -> research-tests -> container-representation check` reaches
both subdirectories. The `research` matrix job in
`.github/workflows/gate.yml` runs that same root target on Ubuntu and macOS.
The parent refreshes gate `whitefootc` before its six subdirectory calls;
this is compiler construction, not a Rust test invocation.

The following are recipe-derived counts for missing products on one host,
excluding that parent compiler refresh and the four unreviewed subdirectories.
Unchanged Make prerequisites can reuse products; their assertions still run.
No build, program, enumeration or measurement was run for this audit.

| Input relative to the experiment root | Construction and products | Current execution under check |
|---|---|---|
| `authority/model.rs` | Two `rustc --edition=2024 -D warnings -C opt-level=2` invocations: ordinary `.build/model` and `--test` `.build/tests` | Seven Rust tests, then the model's normal main |
| `authority/membership.rs` | Same two build modes: `.build/membership` and `.build/membership-tests` | Two Rust tests, then normal main; rustfmt also checks this file |
| `foundation/model.rs` | Same two build modes: `.build/model` and `.build/tests` | Five Rust tests, then normal main |
| `foundation/construction.rs` | Same two build modes: `.build/construction` and `.build/construction-tests` | One Rust test, then normal main |
| `foundation/rust-baseline.rs` | Same two build modes: `.build/rust-baseline` and `.build/rust-baseline-tests` | Two Rust tests, then normal main |
| `foundation/layout.c` | `cc -std=c11 -O2 -Wall -Wextra -Werror` produces `.build/layout` | One C process performs correctness checks, prints layout/count data, then warms and times copies |
| `foundation/large-result.wf` | Two current-compiler calls: `--emit-llvm` produces `.build/large-result.ll`; ordinary compilation separately produces the native `.build/large-result` | One WF native process; check does not inspect the separately retained LLVM |

These rows construct **ten standalone Rust executables, one C executable and
one WF executable**, plus the standalone LLVM output: twelve executable
products and twelve process invocations, with **17 Rust test functions in five
test executables**. The five normal Rust mains are additional runs, not more
test functions. None of the five Rust models imports the current Whitefoot
implementation or consumes compiler-produced output; their toolchain,
standard-library data structures and bounded in-memory fixtures suffice.
Only the last row needs `whitefootc`, its ordinary runtime and native linking.
Foundation also runs rustfmt over its three Rust sources. These standalone
Rust builds do not use Cargo's gate profile.

**Owner direction: research is outside daily verification.** The six model
and native-control rows below stay in research; the actual WF behavior is an
extraction candidate. No refactoring of their research-only runners is required
by this cleanup. Test migration and gate rewiring remain deferred.

| Row | Actual evidence and gap | Recommendation, home and stage |
|---|---|---|
| B05a: finite range authority (`authority/model.rs`) | A private range-token checker is compared with a separately maintained per-slot occupancy/conservation oracle. It constructs all 510 live sets for capacities 1–8; exactly 184 fit one circular window. It checks stale/split/join/raw/live/borrow errors, sparse middle reuse, two wrapped loans and 560 prefix/failure traces for capacities 1–32. Main repeats the 510-set, sparse and wrapped checks and six prefix reports at capacities 4/16/256. The two capacity-256 prefix traces are not in the test loop. This is finite candidate-protocol evidence, not production acceptance or runtime coverage. | Leave this finite candidate checker and its reports in research, outside daily CI and the canonical gate. No current compiler observation needs extraction from this row. The duplicate main and two larger prefix traces matter only if the model is revisited; do not spend this cleanup consolidating its research self-tests. A future production comparison needs a formal test owner and extracted inputs/oracle, not an exception allowing research back into CI. |
| B05b: identity and membership (`authority/membership.rs`) | The private one-slot store compares weak versus retained membership through two indexes, checks wrong-store access-ticket return, stale handles after reuse, access retirement and finite generations 0/1/2. One fixed trace has 31 observations and a separate ledger requires each of nine payloads to drop exactly once; main and the first test repeat it. A second test separately checks generation non-repetition/exhaustion. The access ticket is only a logical deletion guard; it supplies neither backing lifetime nor a real store-ID uniqueness authority. There are no real threads or WF handles. | Leave the model and its two self-tests for explicit research use. It is not evidence about current WF/host handles and no distinct formal regression case has been identified here. Preserve its dated contract/drop observations and limitations without requiring duplicate-run cleanup. Current production handle and concurrent lifetime checks remain with their existing owners. |
| B05c: construction/retirement state machine (`foundation/model.rs`) | A private world enumerates 165 construction/failure/helper-split traces, 42 allocation-first relocations and nine loan-retirement traces. Per-slot IDs and observers detect lost/duplicated values and backing changes. Main executes all 216 traces, and the first of five tests executes them again; the other four tests deliberately weaken full initialization, accounting, identity or retirement to check the oracle. Published/Done/Joined/Consumed are simulated enum states, not actual completion-runtime transitions or worker interleavings. | Leave the five private-model tests and reports in research. The simulated state transitions do not replace B01–B04 production lifetime/progress checks. Removing this research invocation requires no replacement of its 216 trajectories in the compiler/runtime suite and no modernization of the model. |
| B05d: whole-result versus result-tree model (`foundation/construction.rs`) | Seven finite producer/consumer cases run through two private lowering strategies. A separate map/set evaluator checks values, effects, release order and destination; exact modeled whole-slot/transfer counts, six mutated event streams, five invalid-source controls, name independence and direct-placement eligibility are also checked. Both main and the sole Rust test call the same `run_suite`, differing only in printing. Neither strategy invokes the production backend. Source validation is shared by the two model paths, so their agreement is not an independent validation of that shared function or the language rules. The 4,096-byte size is a model label, not measured machine traffic. | Leave the model, event-cost table and self-tests in research. They do not test current backend output or establish a performance-regression verdict. Keep actual production storage-placement/ABI obligations with their compiler owner; do not migrate the private lowerers or copy their candidate direct-placement policy into conformance merely to retain a test count. |
| B05e: safe Rust reusable backing (`foundation/rust-baseline.rs`) | Real Rust Vec/Box conversions at extents 0 and 8 exercise every construction stop point, clear/reuse the backing, and perform three complete checkouts per extent. Per-value IDs/order/drop ledgers check exactly-once destruction; the nonzero trace creates/drops 60 values. Main repeats the same two extent traces that the two tests run. Pointer observations concern the Rust implementation and allocator; the zero extent has no allocated payload and this probe neither injects allocator refusal nor counts allocator calls. | Leave this Rust comparison in research. It currently compares no WF result, so neither its two test cases nor its repeated main need a formal test home. Preserve useful dated pointer/drop observations; do not refactor or maintain this comparison as part of routine compiler verification. |
| B05f: C layout/copy control (`foundation/layout.c`) | Private C full/prefix/wrapped-ring and bitmap/tagged nullable layouts check complete payload copies, zero/empty/partial/wrapped cases and three nullable occupancy patterns. They never call WF code or the shipped C runtime. Main then unconditionally benchmarks nine cohorts: 4,096 warmup copies per cohort and nine samples of 1,200 copies each, yielding 36,864 warmup plus 97,200 measured copy calls, beyond correctness setup. Timings and sizes are printed without a regression verdict. Foundation `measure` currently runs layout through its `check` prerequisite and then runs layout again. | Leave the C control and its timing modes in research, outside daily checks. It compares private C representations, not the shipped runtime or WF output. The duplicate timing reached through measure/check is relevant when this experiment is next requested, but repairing it is not part of extracting formal regression tests. Do not merge the tool into the production-runtime C runner. |
| B05g: actual WF wide-result behavior (`foundation/large-result.wf`) | The current compiler builds a producer returning either Err or a record with 4,096 bytes, then a helper inserts it into `FixedVector<Record, 2>`. Main requires length one, checks every successful byte equals nine, and requires the error path to return an empty run. This can catch actual compilation/output regressions. The nearby compiler `wide_result_returns_preserve_success_refusal_and_owned_children` instead uses 512 u64 words plus owning children, a heap-backed one-slot run, retained calls and an allocation observer, and inspects only the first/last words. It is related, not demonstrated to subsume this byte-array/fixed-run/all-elements case. | Keep the complete external behavior in the existing programs run/container group (`compiler/tests/programs/runs.rs`), with the WF fixture owned under `tests/programs/` and one shared source for the explicit research inspection. Reuse the existing Rust test executable and native construction support; do not add an integration executable for this category. Preserve all-byte success and empty-refusal observations until a concrete receiving case supplies them; do not delete the compiler's distinct owned-child/ABI observation. Remove the uninspected extra LLVM construction from routine check. Generate retained/optimized inspection products only when the experiment requests them, using the same fixture. |

**Extraction, not experiment maintenance.** The six model/control rows
supply no current production-data comparison or required regression verdict.
They stay available for their stated research questions, without new gate
obligations and without refactoring their duplicate executables in this work.
The actual WF witness has useful external behavior to retain in programs.
Independent oracle self-tests also need a concrete formal consumer before
promotion; being independent alone does not earn automatic CI admission.
There is no blanket permission to retain a research suite or its inputs behind
a formal-test wrapper. The boundary above governs all remaining B05/B06 items.

The current `large-result.wf` executable supplies behavior evidence, not a
machine-code shape or copy-cost verdict. `retained-result.ll` and
`optimized-result.ll` are constructed by the explicit measurement recipe;
its awk assertion checks expected function boundaries, not a numeric
performance threshold. When the WF fixture moves, update both research
recipes and current explanatory links, let canonical rendering visit its new
programs home, and remove the old automatic invocation rather than maintaining
two copies/runs. Keep dated measurement facts identifiable as measurements of
their original revisions. The existing owned-place compiler case's two actual
lowering settings and allocation/lifetime observations are not replaced by
this program check or by the result-tree model. No language rule or nominal
test verdict is selected from these research models.

**Source coverage map.** The seven rows cover every executable prerequisite
and command in the two subdirectory `check` targets, including the duplicate
normal/test forms and the hidden C timing call. Membership's 31 observations
are trace entries, not 31 Rust test functions; foundation's 216 trajectories
are bounded model executions, not scheduler schedules. Model counts and
copy-loop counts above describe source work, not elapsed-time measurements or
implemented savings. This audit does not implement any retirement.

The remaining B05 review selects individual useful observations only from
currently enabled proof-use-cost, lifecycle/dense/families, native map controls,
external oracles and indirect formal-test imports. B06's enabled construction
and performance-regression work follows the same extraction boundary; B07
retains its repository-tooling scope. B08's unwired/manual-only instruments
are excluded. Selection of a formal owner does not require a new executable
or an entire experimental bundle's migration.

**Additional current dependency map.** These source-level findings define
extraction work, not completed migrations or an audit of every assertion:

| Current path into research | Observation and next disposition |
|---|---|
| Root `research-tests`: proof-use-cost and container checks | The proof-use runner invokes the compiler for seven accepted fixtures and two PRF-1 negative controls; its check has no native WF execution or performance verdict. Formal PRF-1 conformance and source-proof cases already exist, so compare properties before extracting any additional ceiling/context case. Container lifecycle currently compiles/runs seven accepted programs and checks eight rejected inputs. Normative expectations require the active specification, not the recipe's current-outcome labels; extract missing conformance/program/implementation observations, not both directories wholesale. |
| Root `research-tests`: ripgrep and raw-DEFLATE Python self-tests | The 22 ripgrep tests check a research runner, output normalization, frozen workload selection and statistics. The 19 DEFLATE tests check the research decoder oracle. Existing formal `programs/wfgrep.rs` uses its own Rust reference/system grep, and `programs/raw_deflate.rs` contains wire-byte fixtures and expected payloads; a research citation in a comment is not a runtime import. Do not promote these complete research self-test suites without a demonstrated missing formal consumer. Any useful additional wire vector or output property must first be compared with formal coverage. |
| `compiler/src/lowering/tests.rs`, `semantic/tests/slices.rs`, `semantic/tests/loop_permission.rs`, `backend/tests/slices.rs` | Formal Rust tests already import research WF sources, LLVM host adapters, C oracles and invoke `compute-bench/host-adapter.awk`. Their observations include runtime-extent work estimates, range rejection/loan use, loop-permission metadata, complete compute outputs, wrong-output/missing-observation controls and native path evidence. Retain the useful observations while extracting the necessary inputs and helper logic to formal test ownership. Review redundant builds, benchmark-only code and probabilistic repeats under the previously selected rules rather than copying the entire compute bundle. |
| Root `bench-programs`, Windows IO-host component step | Root checks reach both IO and compute research Makefiles. The Windows step reads `io-completion-bench/programs/windows_component_open.wf`. B04o already selects a programs receiver for its real argument/component observations. Apply the same property-based extraction to other useful inputs and remove experimental construction/measurement machinery from daily callers. |
| Automatic `compute-bench.yml` pushes and `compute-regression.yml` PRs | The first constructs/verifies the research scoreboard; the second decides paired performance regressions using research programs, harnesses and verdict machinery. Its rule self-test is also called from root research-tests. Preserve required performance detection in a formal performance-test owner after extracting the minimal same-workload comparison and its verdict tests; leave exploratory comparators/scoreboards and explicit measurement workflows in research. B06 still owns the detailed comparison/coverage review. |

The dependency map includes indirect compiler imports, not just root Make or
workflow names. Source/document citations to dated evidence remain legitimate;
the prohibited dependency is on executable research machinery and test inputs.
A complete migration audit must follow remaining generated and transitive
inputs too. No broad textual ban or new gate script is added by this discussion.

The research-boundary update corrected standing guidance and added
completion-checklist T4-T6. It updated the fourth pending verification decision;
the enabled-only scope clarification changes no further amendment or guidance.
The build-input amendment and live tree remain unchanged. Existing test code,
Make/CI callers, specification and conformance evidence are unchanged. No build,
executable test, timing campaign or new completion/DCR result is claimed.

## Affected material and evidence

- Specification identity: `compiler/build.rs`, `compiler/src/spec.rs`,
  `compiler/src/bin/spec.rs`, and the title/header of
  `spec/kernel-spec.md`. Current hash generation and recomputation share
  the same SHA-256 implementation; comments claiming independent recorded
  `shasum` evidence are stale.
- Parser construction: `compiler/src/bin/grammar_tables/`,
  `compiler/src/syntax/grammar.rs`, `compiler/src/syntax/grammar/generated.rs`
  and its parser/finalizer/diagnostic consumers.
- Invocation and coverage: Cargo/Make/CI callers,
  `tests/conformance/runner.py`, its tests, and the manifest's
  `whitefoot-spec` coverage annotations. Reassign honest coverage claims;
  do not leave a deleted checker named as evidence or invent a check of
  semantic properties/PR content the replacement does not perform.
- Update current guidance and both inventory languages to the resulting
  system; retain dated measurements with their original revision boundaries.

No implementation, test retirement, specification edit or new performance
measurement accompanies this record.
