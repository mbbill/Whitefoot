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
the exact tree revision has not received an owner ruling. The earlier
verification-cost amendment remains separate and unchanged.
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

## Basic test contract — discussion draft

The owner selected the review method: establish this baseline for compiler
`#[test]` cases and the four corpora first, then consider every remaining
check one at a time. The owner agrees with the other baseline provisions;
the remaining discussion is the admission boundary between compiler tests,
conformance and programs. The admission rules below refine that proposal.
This is not an implemented gate or a replacement of current guidance while
execution remains deferred. It introduces no WF language rule, elapsed-time
limit or new measurement campaign.

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
   measurement. Do not decide from elapsed time.
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

The initial cursor is the first direct C runtime check:
`compiler/src/backend/completion/core_read_probe.c`. It has not yet been
presented under this process. The table below records this review as it
proceeds; the next item is selected only after the current ruling.

| Item | Check | Recommendation | Owner ruling |
|---|---|---|---|
| R01 | Completion core/read probe | Not yet presented | Pending |

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
