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
`#[test]` cases and the four corpora first, then consider every remaining
check one at a time. The owner selected the baseline, including admission by
protected property, and added caution about the cumulative cost of new cases
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

Proceed after discussing the basic contract. Present one independent check
at a time, or one inseparable group of build variants whose differences are
explained. Do not use a whole-directory label to hide unrelated assertions.
Read its sources and current callers before recommending a disposition.

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
6. **Owner decision:** record the ruling and any remaining uncertainty, then
   present the next item. Do not advance through multiple unresolved items or
   implement changes while the owner's execution deferral remains in force.

Use the inventory's remaining runtime probes, platform/sanitizer checks,
research models/oracles, benchmark construction/correctness checks,
repository/tooling checks, performance protocols and historical/explicit
experiment runners as the traversal scope. Previously selected changes
remain selected; only a newly discovered conflict or changed premise reopens
one. Audit individual compiler/corpus cases under the baseline as needed
during their migration, rather than assuming their current classification is correct.

The owner agreed to R01, R02 and R03's dispositions; implementation remains
deferred. The current item is `text_probe` in
`compiler/src/backend/ordinary_values_probe.c`, considered separately from
that file's file/directory, TCP-value and concurrent-close responsibilities.
The next item is presented only after the current owner ruling; none of these
recommendations has been implemented.

| Item | Check | Recommendation | Owner ruling |
|---|---|---|---|
| R01 | Completion core/read probe and its build/run variants | Keep useful C adapter assertions; retire unsupported repeat/TSan claims and redundant boundary guards; details below | Agreed; implementation deferred |
| R02 | Default-policy file reads in the bridge probe | Keep the real-default concurrent C runtime check, reuse deterministic policy cases and tighten route assertions; details below | Agreed; implementation deferred |
| R03 | TCP lifecycle in the default bridge probe | Merge overlapping bridge lifecycle logic, preserve distinct runtime/host configurations and correct the transfer, endpoint and timeout checks; details below | Agreed; implementation deferred |
| R04 | Text values in the ordinary linked-library probe | Keep focused C encoding/value assertions, strengthen existing buffer observations and stop repeating text for helper settings that it does not use; details below | Pending |

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

**Recommendation, pending owner ruling.**

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
