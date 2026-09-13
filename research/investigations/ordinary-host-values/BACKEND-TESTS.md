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
the linked library with zero/four workers and verifies an actual lane grant.
The runtime's private completion-record layout distinction grants no source
overlap permission.

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

These are intermediate local results, not a claim that C2 or host CI is done.

The traversal integration's compiler-IR component-validation test is retired
with QUAL-1/SYS-14 compiler wrappers: the body is now a linked ordinary
implementation. `ordinary_values_probe.c` and the retained
`sys14-open-directory-empty-name` / `sys14-open-directory-component` cases
exercise the same invalid component outcomes. The traversal still checks
ordinary calls and exact tree output. Its moved-source, byte-to-path and
nonexhaustive-status negatives retain OWN-1, TYPE-5 and ERR-2 respectively.

## Integrated proof-state repair

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

Deleting PAR-3 in v0.58 removes the lowering contract those assertions tested.
This revision therefore deletes the hook, its two private window constants,
the test's two mirrored constants, that single test function and its runner
invocation. It does not change the native submit/join protocol, progress,
directory, TCP, credit-transfer, queue, race, or resource-failure checks.
The adjacent io_uring doorbell test remains: deferring a native submission is
an implementation behavior, independent of the retired source permission.
The orphan state-routing comment above `Checker::constants` is also removed;
the state-routing field and implementation were already absent.
