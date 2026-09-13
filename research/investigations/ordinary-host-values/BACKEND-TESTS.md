# C2 backend evidence and retired assertions

This record explains the backend migrations against the pre-amendment
`cc24e880` tree. It accompanies [CASES.md](CASES.md), which owns normative
conformance changes. It is maintained with the ordinary callable ABI and its
native library witnesses; supersede it when that ABI is replaced.

## One callable ABI

An ordinary declaration without blocks has a `FunctionAbi` and no activation
storage plan. A WF definition with the same parameter/result types has the
same ABI. `system::ordinary_declarations_have_no_frame_and_share_the_call_abi`
guards the previously failing bodyless-declaration path. The paired view
test adds a WF wrapper with the exact `host_copy_bytes` signature and source
contract, checks ABI equality, and executes raw non-UTF-8 bytes through both
calls in the three driver configurations.

`ordinary_values.ll` supplies ten ordinary library definitions whose view
parameters are LLVM `{ptr, i64}` values. Each stores that descriptor locally
and calls a private C body through pointers/scalars. This is necessary because
the platform C aggregate calling conventions differ: the WF ABI is shared
by all definitions and calls and must not change for a linked definition.
No compiler operation selector, native tag, qualification, or alternative
call instruction selects these bodies. Only the two-word descriptor is
stored; the payload is neither copied nor extended in lifetime. The other
native definitions already match the scalar/aggregate-pointer ABI directly.

The native C-only probe first passed while an actual WF read exposed this
descriptor ABI mismatch. It is therefore complementary evidence, not a
replacement for the paired WF/linked execution test.

Directory enumeration returns three ordinary values: a unit/error result,
the numeric endpoint, and the entry count. Its unconditional endpoint bounds
use the existing multi-result contract path. The earlier success-payload form
could not retain a non-measure relation through a subsequent own-place match
under CALL-4; the library interface now avoids that limitation without changing
the checker rule. The native result layout is independently asserted as a
240-byte status followed by two eight-byte integers. The LLVM definition keeps
the same ordinary aggregate-destination ABI. Empty, successful, and end batches
preserve the original behavior; every failure reports the starting endpoint.

## Retired assertions and preserved observations

| Pre-amendment assertion family | C2 rule change and replacement evidence |
| --- | --- |
| `system.rs`: semantic identity lookup; missing target mapping/guarantee; argument/directory/enumeration qualification; ABI-equivalent but wrong system/entry identity mutation | SYS/QUAL and the separate declaration domain are deleted. There is no such checked-IR identity to mutate. Ordinary nominal/signature checking remains; the new bodyless signature and paired view ABI regressions exercise the shared backend path. |
| `system.rs`: command bootstrap/type/label qualification | Entry kind and labels are deleted. The build launcher calls an ordinary accepted signature. No-input status, ordinary Inputs, complete argv, raw bytes, and explicit Heap entry are executable cases; unsupported launcher profiles remain a link/build issue. |
| `system.rs`: exact implicit release action expansion | SYS-5/STOR-3 release rows are deleted and opaque drop is empty. The affine opaque-drop test observes no close call; explicit linear close chains are in the corpus and `ordinary_values_probe.c`. |
| `system.rs`: compiler-emitted component flags, directory record scans, provisional descriptor validation, TCP operation-to-symbol table | Native bodies are ordinary linked implementations. The native probe checks component rejection/open, directory records, range writes and TCP; scripted-library tests preserve provisional validation failure and close-error behavior. Native platform CI compiles the platform bodies. A compiler symbol inventory is not retained as a surrogate semantic domain. |
| `system_io.rs`: Windows compiler-specific UTF-16 bootstrap/read route and native error inventory order | Windows invocation conversion belongs to build initialization (`CommandLineToArgvW`); portable error construction belongs to the C library. Ordinary raw/text argv and range tests retain the observable contracts. The error-arm generator reads the ordinary prelude enum rather than a semantic catalog. |
| `system_io.rs`: static SYS-8 transfer rejection | Unchanged invalid endpoints now fail the ordinary declared FN-8 requirement. Their conformance manifest entries record this rule change. |
| `system_io.rs`: zero-byte-write branch text in compiler-generated LLVM | The prefix/absolute endpoint assertions remain. The test additionally forces native `Accept(0)` through the deterministic linked implementation and observes `Err(WriteZero)` with code/origin zero. It no longer mistakes removed compiler-body text for execution evidence. |
| `system_io.rs`: target release close count and no-retry text | Explicit close is an ordinary call. The deterministic library suite checks exactly one close attempt, including failure; native credit tests check reuse. Empty opaque drop must not close anything. |
| `system_io.rs`: reserve-permit drain, returned permit count and implicit cleanup | The selected direct-factory API has no permit value to drain. Native probe tests finite factory credits, refusal preservation, close into a different factory, and last-half-only TCP credit return. Source close/reopen and failed-open/retry cases preserve the ordinary chains. This does not claim to execute the retired permit API. |
| `system_io.rs`: compiler transfer body has no allocation/copy/dispatch/lock | That body has moved to the linked library. Native operation tests establish behavior; workload and optimized linked-code measurements own cost claims. No inference from absent compiler IR is used as a performance proof. |
| Shared `optimized_main_wrapper` test helper | Its only cost-census caller was retired with the qualification-based census. Repository search found no remaining calls; deleting this dead helper removes no test. The ordinary optimized-entry helper remains. |
| `requires.rs`: command contract rejected before creating a wrapper | FN-7's command restriction is deleted. An ordinary selected function with a true numeric requirement compiles and runs without an executable guard; a contradictory requirement leaves an accepted callable module with no executable launcher. Ordinary callers still must prove the requirement. |
| `resource_enums.rs`: moved payload must retain reads of its former enum owner | EFF-2's value-history roots are deleted. After an owned match the payload is local. The exact-row regression now accepts pure and rejects an added reads(owner); the same active-variant cleanup and native output checks remain. |
| `target.rs`: heap-domain test helper also reduced the address-index domain | Ordinary PRE declarations exposed the helper's conflation: a 16-byte heap test rejected a 32-byte opaque representation first. The helper now changes only allocator limits, as documented. Separate exact address-domain tests and every allocator boundary assertion remain. |

The unchanged observations include short reads, end only after an observed
zero read, empty transfer, exact written prefix, absolute endpoints, ordered
output, non-UTF-8 preservation, full UTF-8 validation before writing, short
destination preservation, relative path behavior, recoverable broken pipe,
and source-visible error classes. The deterministic linked-library suite
also preserves interrupted/would-block retries and provisional cleanup.

## Refusal paths and ordinary overlap

The 34 host cases with an old reserve failure branch retain that real branch
under direct open's `ResourceExhausted` with origin zero. Native open failures
retain their old open-failure branch. The distinction is implemented by an
ordinary borrowed enum match, not compiler classification. Every new early
return closes already acquired linear owners. The other cases have no reserve
branch, or explicitly re-derive the retired permit assertion in CASES.md.

The generic scheduler publishes ordinary callees using one `wf__par_publish`
protocol. Its callback may call a linked body and park; no suspension class
selects publication. The ordinary worker-helper regression writes `X` through
the linked library with zero/four workers. It identifies the published thunk
that calls `write_byte`, then records exactly one publication, entry on another
thread, and completion. The zero-worker control records none, with the same
`X` output and successful exit. The runtime's private completion-record layout
distinction grants no source overlap permission.

The previous detector incorrectly treated `wf__par_grants` (a count of steals)
as a count of publications. A submitting thread can legally execute its own
published task during join, so repeated fresh processes did not establish the
claimed observation. The fixture now intercepts only this main-body publish
and join pair: it calls the real publication, waits for callback entry on a
different thread, then calls the real join. Entry is recorded before invoking
the original thunk, allowing the caller to join while linked I/O parks and
resumes. Actual worker startup is checked before waiting; acquisition refusal
skips the observer and fails the positive ledger rather than waiting. The real
frame, acquire/release, thunk, native body and scheduler completion protocol
are unchanged. This deterministically exercises one legal schedule without
changing the runtime or claiming that every publication must be stolen.

Sequential clones now plan storage in the sequential world, including callee
reuse, instead of retaining deferred-operand interference from the parallel
body. The existing byte-for-byte sequential-clone comparison and one-time
launcher world-selection tests guard that fix. They retain the ordinary
destination ABI for aggregate results.

## Recorded local evidence

- Native probe: text, ranges, factory refusal, files, directories, TCP, crossed
  halves and cross-factory credit transfer pass with zero/two I/O helpers.
- Full completion target passed after ordinary native implementation extraction;
  later whole-language integration and the final exact-revision gate remain
  separate requirements.
- All 33 run cases in the assigned host subset executed successfully in
  default, explicit no-overlap and parallel configurations: 99 executions.
  The complete 99-execution run was repeated after restoring the explicit
  reserve-refusal branches, with the same fixture outcomes.
- After restoring those refusal branches, all 59 assigned host cases passed
  their exact current manifest verdict and rule checks. No verdict was edited.
- The ordinary system/system_io subset passed 21 tests, including the forced
  zero-write replacement. The deterministic linked-library suite passed 16.
- After the three-result directory change, all five native record-decoder tests
  passed. The native value probe passed with zero/two helpers, including empty
  range, exact endpoint observations, and enumeration through the end outcome.
- The exact view-signature WF/linked regression passed all three driver
  configurations after the ordinary wrapper reconstructed its result inside
  the call match, where CALL-4 makes the selected payload relation available.

Windows links using the ordinary launcher omit the old Unicode-entry flag:
the launcher defines `main`, while the library converts the native command
line itself. The three observed/control/sanitized WF links in native CI include
both ordinary library units and the shell import library, matching the driver.
Native probes with their own entry remain independent of that launcher choice.

These are local results; the PR's canonical gate and native host CI record
whether its exact revision satisfies the complete C2 acceptance criteria.

The main integration exposed a native Windows lost wake. The completion core
now tests `wake_needed` before notifying an announced sleeper, but the merged
IOCP park path only incremented the sleeper count. A helper completion could
therefore publish DONE without posting an IOCP wake; unlike overlapped file
I/O, that helper has no kernel packet to wake the caller independently. IOCP
now uses the same `wf_completion_announce_park_locked` protocol as the other
wait paths, with its epoch recheck and notification cohorts unchanged.

The Windows arm of `native_adapter_probe.c` exercises a helper publication on
a fresh empty port, before submitting any file I/O. A real native waiter is
observed entering the kernel-wait path, then a completion is published through
the real runtime notifier. The test requires exactly one host wake, the DONE
record and result 37, and no remaining parked waiter. Finite probe waits make
the missing notification a failed check instead of an indefinitely hung job.
The pre-fix source skips that wake because `wake_needed` remains zero; Windows
execution, rather than cross-compilation alone, owns confirmation of the fix.

The traversal integration's compiler-IR component-validation test is retired
with QUAL-1/SYS-14 compiler wrappers: the body is now a linked ordinary
implementation. `ordinary_values_probe.c` and the retained
`sys14-open-directory-empty-name` / `sys14-open-directory-component` cases
exercise the same invalid component outcomes. The traversal still checks
ordinary calls and exact tree output. Its moved-source, byte-to-path and
nonexhaustive-status negatives retain OWN-1, TYPE-5 and ERR-2 respectively.

## Integrated proof-state repair

The search integration's former permit-inventory/QUAL assertion is replaced by
five exact ordinary direct-call checks. Merely finding each symbol now also
finds an always-supplied PRE declaration, so that would no longer establish
the operation chain. All recursive-search execution oracles remain unchanged;
the old catalog-ordinal explanation is retired with SYS/QUAL.

The private engine's positioned-read regression retains every caller-selection,
publication and refusal assertion. Its source slice now ends at the statistics
queries: the earlier delimiter was the unused PAR-3 window section deleted by
C2. This repairs the inspection boundary without narrowing the submit-family
assertion or restoring the retired hook.

Ordinary contracts exposed a branch-state implementation defect: checking a
call in one branch changed the analyzer's shared measure image, so the other
branch could lose an entering length fact before the branches even joined.
The image now belongs to each cloned affine flow state. Atom allocation stays
global, but writes kill only that edge's current image. A join keeps an image
only when both entering images agree; this does not strengthen ENT-6's
conservative common-fact rule.

`a_conditional_unique_call_keeps_the_other_branch_measure_image` checks both
branch orders. It refreshes storage after the conditional rather than asking
the join to derive a stronger theorem. The complementary exclusive-replacement
negative retains INV-1 for a stale length. Element-only mutation in maintained
replay sources uses ordinary `MutSlice` parameters to avoid declaring a write
to the owning descriptor.

The frozen real-source proof census retains all fourteen `read_bits` call
sites and their exact selected-result masks. PRE-1's ordinary `read_at` and
`write_once` signatures each add two CALL-6 direct-match endpoint roots, so
the total is eighteen; the test also checks those two roots at each exact call.
Its four raw-DEFLATE initialization loops are counted in `exercise`, where
the ordinary Inputs wrapper moved that operation chain, and main has zero.
Bodyless signatures contribute no source call sites, while their derivation
records remain part of the complete checked-program traversal.

## Retired PAR-3 runtime window hook

The final C1 D/M audit found `wf__completion_window` still declared in
`backend/completion/bridge.h` and defined in `bridge.c`. A repository search
over `compiler/src` and maintained `research/experiments` found no production
caller: its only calls were the boundary assertions in
`test_completion_window_answers_at_the_boundaries` in `completion/harness.c`.
The old compiler's staged loop lowering had been its consumer; ordinary linked
functions and the native engines do not query it.

Deleting PAR-3 in the combined v0.55 publication removes the lowering contract those assertions tested.
This revision therefore deletes the hook, its two private window constants,
the test's two mirrored constants, that single test function and its runner
invocation. It does not change the native submit/join protocol, progress,
directory, TCP, credit-transfer, queue, race, or resource-failure checks.
The adjacent io_uring doorbell test remains: deferring a native submission is
an implementation behavior, independent of the retired source permission.
The orphan state-routing comment above `Checker::constants` is also removed;
the state-routing field and implementation were already absent.

## Native Windows directory self-open

The Windows ordinary-value probe on `4945e3df` reached the directory-source
open and failed its success assertion. The old probe did not print the error
payload, so that run establishes refusal, not a particular native error code.
The library passed UTF-16 `"."` directly to `NtCreateFile`, which does not
perform Win32 dot-directory normalization. The native implementation now uses
an empty relative object name with the same root handle, retaining a fresh
open and the existing kind validation, descriptor registration and cleanup.
[Zig 0.14.0's NtCreateFile caller](https://github.com/ziglang/zig/blob/0.14.0/lib/std/os/windows.zig#L1009-L1012)
provides independent implementation evidence for this representation; the
native Windows job is the execution check.

The probe now opens two directory sources before consuming either, exhausts
one, and verifies that the other still yields entries. Both are explicitly
closed and the exact factory-credit balance remains asserted. It also prints
the ordinary error payload before the original open assertion on a refusal.
This is a linked-library path correction under unchanged PRE-1 interfaces;
no source acceptance rule or existing assertion is retired.

## Standard-stream compiler assertions

`programs/stream.rs` no longer requires a compiler-emitted
`wf__completion_file_read_submit` / `wf__completion_file_join` pair or a
label-derived `wf_main(i32 1, i32 0, ...)` entry ABI. C2 deletes the SYS-15
compiler operation path and FN-7 input labels: PRE-1 supplies ordinary
`read_next` and `write_once` signatures, and build initialization supplies an
ordinary Inputs owner and Heap value. The replacement assertion counts exactly
one direct call and declaration for each signature, excludes a positioned read
and compiler completion calls, and checks the Inputs/result-destination launcher
ABI. Pipe, redirected-file and EOF executions on both native implementation
routes retain their complete status and byte assertions; native submit/join
behavior remains covered by the completion implementation tests.

## Ordinary public calls in the stack ledger

C2 exposes WF definitions under the same ordinary public ABI as linked
definitions. On Linux, Clang spells direct calls to those definitions with
the ELF `@PLT` suffix. The stack ledger resolved only undecorated names, so it
dropped those edges and reported recursive functions as isolated acyclic
frames. The Linux gate on `812c0465` exposed this in the measured-ceiling test
and both parallel deep-recursion tests; their physical depth assertions are
unchanged.

The graph reader now strips that linkage suffix before resolving against the
same module's measured frame index. A compiler-independent x86 assembly case
checks a bounded call chain and a recursive cycle with exact byte and level
counts, and equality with the undecorated graph. A current WF spine compiled
to x86-64 Linux assembly independently confirms the `callq wf_spine@PLT`
trigger. This repairs non-normative build output; no source rule, signature,
lowering, frame size, or runtime stack budget changes.

## Reusing identical native objects in integration tests

The Linux corpus and conformance jobs on `f7261e05` reached their eight-minute
ceiling. The corpus printed 70/70 passing program tests before cancellation;
the native conformance adapter had not completed. Every executable was
recompiling the same eleven C units and ordinary LLVM library unit. No source
failure, test exclusion or budget increase follows from these timeouts.

One freshly emitted `exit_status(0)` module was linked five times by each
recipe, alternating their order, on arm64 macOS with Apple Clang 21.0.0.
The source and object recipes use the same twelve inputs in the same order,
`-pthread`, `-O2`, both include directories and `-lm`. Default C and explicit
C11 remain separate recipes. The table records seconds and all five pairs:

| C dialect | Full source link samples | Reused object link samples | Median full / reused | Initial object build |
|---|---|---|---|---|
| Host default | 1.5164, 1.5756, 1.9027, 1.9883, 1.7717 | 0.1347, 0.1333, 0.2141, 0.2214, 0.2141 | 1.7717 / 0.2141 | 2.5278 |
| C11 | 1.8159, 1.7754, 1.8298, 1.9745, 1.6992 | 0.1413, 0.1610, 0.1386, 0.1750, 0.1532 | 1.8159 / 0.1532 | 2.3462 |

Both sets occupy 86,320 object bytes. All ten paired executions had identical
stdout, stderr and successful status; the linked `__text` bytes were identical
between source and object recipes in both dialects. The repeated link cost
fell by 87.9% and 91.6%, respectively. These are local build-cost measurements,
not Linux deadline results or program runtime speedups.

`compiler/tests/support/mod.rs`, used by the existing programs and conformance
targets, now holds one in-process object-byte cache per dialect. Every native
unit remains an explicit link input; no archive selection or program-content
classification omits a body. Each emitted module and observer is still
compiled afresh, with the observer in its original position after the floor.
Temporary cache-build files are removed after their bytes are read. Cases
retain their original source/header staging and cleanup policies, and the new
object files are removed before execution so directory fixtures cannot see
them. Backend tests with facility-substitution defines keep their existing
uncached path. No acceptance, runtime, assertion, case collection or verdict
changes accompany this build repair.

## Combined publication over the current-stack runtime

The v0.55 integration adopts main's persistent native worker stacks and
64-slot deques, asymmetric-core idle-window rule, recursive offer budgets,
scalar-leaf and sequential-refusal controls, and native IOCP/io_uring repairs.
Earlier C2 measurements remain measurements of their named pre-merge
revisions. They do not validate performance of this combined runtime; its
maintained controls require fresh samples.

C2 removes the semantic suspension exclusion from recursive offer selection.
An ordinary cycle uses the same eligibility rules whether a member calls a WF
body or an ordinary linked body. The former
`a_suspending_cycle_with_compute_offers_gets_no_budget_family` assertion and
the staged-only half of the recursive-controls test are retired for that
specific removed classification. The pure recursive and scalar-leaf checks
remain, as does execution of a published ordinary call reaching linked I/O.
No runtime budget selects source acceptance or required proof work.

Managed-stack and staged-pipeline assertions describe deleted runtime paths.
Main's smoke, native deque, wake and boundary probes exercise the adopted
runtime; they do not claim the exhaustive schedule coverage of the retired
managed-stack enumerator. Removing the old enumerator follows removal of the
state machine it modeled, rather than a narrower sample of that state machine.
The retained Windows native-control and worker comparisons report samples and
ratios; performance/stability enforcement and retry-to-select-a-pass are not
part of C2's correctness gate.

The added adaptive-quadrature regression initially required the budget-family
definition to use LLVM `internal` linkage. C2 emits that same family as an
ordinary strong definition, so the assertion now checks the family definition
and the budget call separately. The targeted test still executes the original
numerical/output equality oracle at one, two and four workers. This is a
linkage-assertion migration; no source verdict, numerical result or worker
configuration was removed. Its focused pass during the uncommitted main
integration does not establish a complete gate or cross-version speedup.
