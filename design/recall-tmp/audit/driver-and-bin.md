# Audit: the driver, the binaries, and the crate's edges against the design tree

Module: `compiler/src/driver.rs` + `compiler/src/driver/{pinned_sentences,rejection}.rs`,
`compiler/src/bin/{whitefootc,spec,grammar}.rs` +
`compiler/src/bin/grammar_tables/{ebnf,main,model}.rs`, `compiler/src/lib.rs`,
`compiler/src/source.rs`, `compiler/src/spec.rs` + `compiler/src/spec/sha256.rs`,
`compiler/build.rs`, `compiler/Cargo.toml`, `compiler/Makefile`, and the gate
wiring in the root `Makefile` and `.github/workflows/gate.yml` (about 10,000
lines of Rust across the named files, plus the three build files). Direction:
code to tree — finding choices the code embodies that `design/` does not
record, not the reverse.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md` (all four decisions
and the rejected list), `design/language.md`,
`design/compiler/parallel-lowering.md` and its children
`parallel-runtime.md`/`two-worlds.md`,
`design/compiler/resource-exhaustion-floor.md`, and, checked for relevance and
found not to bear on this family, `design/compiler/{cleanup-traversal,
tag-only-lowering,wide-probe-lowering}.md`. Also read for grounding, since the
driver's own comments and the permission ledger it prints name them directly:
`design/language/parallelism.md` and
`design/language/parallelism/permission-judgment.md`. `CLAUDE.md`'s
specification-identity and branch-and-main sections, `spec/kernel-spec.md`
[DIAG-1], [DIAG-2], [META-1]–[META-5], and [QUAL-1]/[QUAL-2], and
`docs/todo.md` (for known, already-tracked defects this family's code
independently corroborates). Read as evidence, not audited:
`compiler/tests/{canonical_corpus,conformance,snapshot}.rs` and
`compiler/tests/programs/support.rs`.

Git history note: the clone is complete, 2,475 commits back to the true root
`7c1d7641` (2026-07-07, not shallow — confirmed with
`git rev-parse --is-shallow-repository`), so every "Reason" line below is
checked against the full history with `git log -S` and `git blame` rather than
a truncated view. "No reason recorded" means the introducing commit and its
successors say nothing, checked against that complete history.

## 1. Covered by the tree

- `design/compiler.md` (decision 1: one safe-Rust crate, no dependencies,
  unsafe forbidden, no stable artifact format) — `compiler/src/lib.rs:1`
  `#![forbid(unsafe_code)]` crate-wide plus `#![deny(missing_docs)]`;
  `compiler/Cargo.toml`'s `[lints.rust] unsafe_code = "forbid"`,
  `[lints.clippy] all = "deny"`, and no `[dependencies]` table at all; every
  one of the four `[[bin]]` targets repeats `#![forbid(unsafe_code)]`
  (`whitefootc.rs:1`, `spec.rs:1`, `grammar.rs:1`,
  `grammar_tables/main.rs`); `compiler/src/spec/sha256.rs` is hand-rolled
  specifically because "the crate has no dependencies and keeps none" (its own
  doc comment); `lib.rs`'s own header restates the decision almost verbatim
  ("These stages remain evolvable implementation APIs, not stable
  protocols").
- `design/compiler.md` (decision 2: admits and lowers by the specification's
  rules alone, through one semantic path and one lowering path) —
  `driver.rs::compile_reporting` (lines 347-606) is the one pipeline; every
  public entry point (`compile`, `compile_with_overlap`,
  `compile_with_permission_ledger`, `compile_with_io_notices`, lines 269-330)
  is a thin projection of it, stated in its own doc comment ("there is no
  second pipeline"); `compiler/src/bin/grammar_tables/` mechanically
  regenerates `syntax/grammar/generated.rs` from the specification's own
  seven `wf-ebnf`-fenced blocks (`grammar_tables/ebnf.rs::Owner`) and is
  checked byte-for-byte against the committed file
  (`grammar_tables/main.rs::generate`/`COMMITTED_TABLES`, test
  `committed_tables_are_derived_from_the_active_grammar`).
- `design/compiler.md` (decision 3: valid-but-unimplemented source stops as
  an explicit unsupported capability, never reported as invalid) —
  `driver.rs::CompilationFailureKind::Unsupported` is kept structurally
  distinct from `::Source` throughout (`SemanticOutcome::Unsupported` arm,
  lines 539-545); the driver's own tests exercise exactly this boundary
  (`driver_lowers_static_contract_metadata_without_executable_artifacts`,
  `every_pre_semantic_rejection_publishes_the_rule_its_stage_attributed`).
- `design/compiler.md` (decision 4) + spec [DIAG-1] (a rejection names the
  numbered rule and location; one compiler executable rejects the same
  source the same way; no acceptance path depends on a timeout, budget, or
  order) — `compile_reporting`'s stage sequence follows [DIAG-1]'s stated
  order exactly (lexical, terminal membership, grammar derivation, then
  canonical [FORM-2] rendering) and stops at the first stage that is not
  `Complete`; `Finalization` (lines 446-462) is structurally unable to carry
  a `SourceIssue` (only `Complete`/`ResourceFailure`/`CompilerFailure`),
  matching [DIAG-1]'s stage list not naming finalization as a fifth,
  separately source-rejecting stage; every rejection threads `rule_id()`
  from the stage's own selection (`CompilationFailure::source`, lines
  205-212); Resolution and Semantics rejections carry
  `SemanticLocation::SourceNode`/`::BundleRoot` (lines 518-521) — the exact
  two named variants of [DIAG-1]'s closed location sum; `whitefootc.rs`'s
  dedicated `COMPILER_DRIVER_STACK_BYTES` driver thread (line 42, spawned at
  lines 200-211) exists precisely "so a source program's acceptance does not
  depend on the host executable format" (its own comment) — otherwise
  Windows' 1 MiB default main-thread stack could reject a source that every
  other host accepts.
- spec [QUAL-1]/[QUAL-2] correctly kept outside [DIAG-1]'s rule-citing
  rejections — `driver.rs:589-605` maps `BackendFailure::TargetLayout` and
  `::TargetQualification` to their own `CompilationFailureKind` variants
  with `rule_id: None`, with the code comment stating why: "it is not a
  source-language rejection and cites no language rule [DIAG-1]".
- `design/language.md` (decision 3: one observable behavior; no build mode,
  flag, or optimization choice changes what an accepted program does) —
  `HOST_OPTIMIZATION_ARGUMENTS = ["-O2"]` (`driver.rs:34`) is the one
  link-optimization constant used identically by the shipped driver, the
  backend's own linked-executable helper, `tests/programs/support.rs` (whose
  own comment states the reason: "no path can silently link an unoptimized
  binary while another links an optimized one"), and the conformance
  adapter; the permission-judgment tests
  (`the_permission_ledger_does_not_depend_on_whether_the_lowering_is_taken`,
  `a_no_overlap_build_reports_no_denied_io_loop`) assert byte-identical
  modules and identical verdicts across the `Completion`/`--par`/
  `--no-overlap` lowerings.
- `design/language/parallelism.md` (decision 2: permission and actualization
  are separate judgments) — `compile_with_permission_ledger`'s doc comment
  states it directly ("Permission verdicts are independent of the
  actualization policy"); `whitefootc.rs`'s `--par-ledger` prints the same
  judgment lines whether or not `--par` is given, pinned by
  `overlap_lowering_is_off_unless_the_invocation_asks_for_it` ("reading the
  ledger must not enable lowering").
- `design/compiler/parallel-lowering.md` (decision 4: a 16-nonconstant-
  operation scalar-leaf default, overridable per compilation; decision 5:
  sequential refusal off unless asked, no universal grain policy) —
  `whitefootc.rs::Options::parse`/`overlap()` (lines 610-772): `--par` alone
  defaults `scalar_leaf_limit` to `Some(16)`, `--par-scalar-leaf-limit N|off`
  overrides it, and `--par-sequential-refusal` is off unless named and
  requires `--par`.
- `design/compiler/parallel-lowering/two-worlds.md` (the
  `--par-recursive-frontier auto|N|off` decision: runtime-derived default,
  `N` in 1..32 pinned at compile time, `off` emits no budget-carrying
  family) — `whitefootc.rs::Options::parse`'s `--par-recursive-frontier` arm
  and its `RecursionBudget::{RuntimeDerived,Pinned,Off}` construction match
  the node's exact three-way spelling.
- `design/compiler/parallel-lowering.md` (decision 3: a module with compute
  offers requires the compiler-owned runtime at link time) —
  `whitefootc.rs::compile_executable`/`runtime_units` (lines 382-457) always
  stages the floor group and conditionally stages the scheduler-core and
  completion groups by `module_requires_parallel_runtime`/
  `module_requires_completion_runtime`, so a module that needs the runtime
  fails to link without it.
- `design/compiler/parallel-lowering/parallel-runtime.md` (the deque stress
  probe, under `sched-deque-test` and repeated under ThreadSanitizer, is the
  runtime's concurrency check) — `compiler/Makefile:322-340`
  (`sched-deque-test`, `sched-deque-tsan`), wired into `COMPILER_STAGES`
  through `completion-test`.
- `design/compiler/resource-exhaustion-floor.md` (decision 2: the entry runs
  on a stack the runtime sizes) — `whitefootc.rs::print_stack_ledger`
  (lines 337-374) reports a compiled function's frame cost measured directly
  against `FLOOR_STACK_BYTES`, the one surface where a writer sees that
  exact runtime-owned size.
- CLAUDE.md's specification-identity rules + spec [META-1] ("no rule ID is
  defined twice and every cross-reference resolves") —
  `bin/spec.rs::{is_rule_id, rule_definitions, rule_references,
  validate_spec_integrity}` (lines 10-57, 288-311) implement exactly that;
  `run_gate` (336-360) additionally checks the embedded hash against
  `computed_active_spec_hash()`; `compiler/Makefile`'s `spec:` stage (line
  238, `cargo run --bin whitefoot-spec`) is wired into `COMPILER_STAGES`.
- CLAUDE.md's META-5/no-ledger rule ("the pull request states what changed
  and its selection ground ... There is no separate ledger") +
  `design/language.md`'s matching decision — `bin/spec.rs:302-304`'s own
  comment retires the former header-phrase checks for exactly this reason
  ("META-5 places the delta and selection ground in the change's pull
  request, not the normative bytes"); `compiler/build.rs` +
  `compiler/src/spec.rs`'s hash-derivation pair (`ACTIVE_KERNEL_SPEC_HASH`/
  `computed_active_spec_hash`) is what CLAUDE.md means by "`compiler/build.rs`
  derives it" and "the specification's bytes are its own identity."
- CLAUDE.md's branch-and-main rule 3 ("All repository tests is the root
  `make check` target: ... the specification checks, conformance structure
  and coverage, and the full native conformance adapter including the case
  ordinary Cargo runs mark ignored") — root `Makefile`'s `CHECK_STAGES`
  (lines 24-25) and `.github/workflows/gate.yml`'s per-stage matrix jobs run
  exactly `make static`, `make -C compiler test-partition test-unit`,
  `test-sampling`, `test-corpus`, `make conformance && make conformance-run
  && make snapshot-run`, `make research-tests`, `make bench-programs`;
  `tests/snapshot.rs`/`tests/conformance.rs` are `#[ignore]`d for cost and
  invoked with `--ignored` by name, matching precisely.
- CLAUDE.md's archive-immutability rule ("Released flat
  `spec/kernel-spec-vN.md` archives are immutable ... `make check` checks
  identity consistency and archive immutability") — root
  `Makefile::spec-append-only`/`spec-append-only-staged` (lines 94-113) diff
  released archives against `main` and fail on any modification, removal, or
  rename.

## 2. No decision needed

- Hand-rolled linear argv parser in `whitefootc::Options::parse` — no
  external args crate, matching "the crate has no dependencies."
- Two-name source identity (`logical_path`, the closed portable bundle key,
  vs. `display_path`, the caller's literal argument) in both
  `source.rs::SourceInput` and mirrored in `whitefootc.rs::source_names`/
  `logical_path` — a mechanical consequence of `LogicalPath`'s closed
  portable spelling, fully reasoned in the type's own doc comments.
- `SourceBundle::with_limits`'s explicit `try_reserve_exact` and
  checked-arithmetic-throughout (`checked_add`/`try_from` rather than raw
  arithmetic) — ordinary overflow-safety practice matching the crate-wide
  `[profile.gate]` overflow-checks.
- `driver/rejection.rs::Located`'s one-based byte-column counting — justified
  purely by [FORM-3]'s ASCII-only source ("a byte column is a column," the
  module's own comment).
- `Located::in_gap`'s anchor-at-the-gap's-last-line heuristic for [FORM-2]
  canonical-source rejections — a presentation-only refinement, reasoned in
  its own doc comment, that changes which line is quoted and never a
  verdict.
- `driver/pinned_sentences.rs`'s whole probe corpus and its
  `every_diagnostic_sentence_is_pinned_by_a_probe` test — ordinary
  regression-test engineering; its own header states the reason in full (a
  verification pass found fifty-four rendered sentences no test compared).
- `whitefootc.rs`'s runtime-staging-to-a-temp-directory-then-clang approach,
  preserving `backend/`'s relative include topology
  (`FLOOR_SHARED_UNITS`/`CORE_...`/`COMPLETION_...`, lines 91-184) — an
  implementation necessity of embedding the runtime as Rust string constants
  portably, tested by `runtime_staging_closes_every_quoted_include`.
- `HOST_LINK_LIBRARIES` ("-lm") and Windows' `-lws2_32` — a mechanical
  consequence of which system calls the language surface lowers to,
  discovered empirically (batch 0090, named in the constant's own doc
  comment) and reasoned in place, not a choice with a real alternative.
- `print_stack_ledger`'s second, separate clang invocation rather than a flag
  on the ordinary link — dictated by `-fstack-usage`'s own output-placement
  behavior and the need for the post-inline call graph, both stated in its
  own doc comment (lines 324-336).
- The two-thread process shape in `whitefootc::main` (spawn a named,
  8 MiB-stack `whitefootc-driver` thread, join it, and preserve Rust's
  ordinary panic exit status rather than printing a second, less useful
  panic) — ordinary engineering once the stack size itself is covered above.
- `whitefoot-spec`'s `--index`/`--counts` query flags and their hand-rolled
  JSON writer plus a hand-rolled JSON reader in tests — developer tooling
  for spec review; the crate's no-dependencies rule is why the test reads
  its own emitted JSON by hand rather than pulling in a parser.
- `grammar_tables/main.rs`'s `ENUM_ORDER` and decision-slot-order hand-carried
  tables — explicitly justified in place as historical, non-derivable dense-
  index inputs ("Retired productions leave this inventory instead of
  surviving as dormant parser concepts").
- `whitefoot-grammar`'s `PARSER_PROBES` (five minimal programs) and
  `verify_compiler_grammar`'s SELECT-row coverage/disjointness self-check —
  an ordinary internal-consistency smoke test of the compiler's own derived
  grammar data, matching the file's stated purpose.
- `ebnf.rs`'s extraction keyed on the `wf-ebnf` fence marker — the
  specification itself defines exactly these seven fenced blocks
  (`grep -n 'wf-ebnf' spec/kernel-spec.md`: GRAM-2..5, CONST-1, CONST-2,
  EFF-1), so extraction is spec-derived, not an independent compiler
  convention.
- `Cargo.toml`'s `[profile.gate]` (release codegen plus retained debug
  assertions and overflow checks) — a measured test-speed choice (~136s vs.
  ~8s, stated in the file) that changes no accepted program, only test
  wall-time.
- The root and compiler Makefiles' and `gate.yml`'s staged/parallel-job
  structure, core-dump suppression, and per-OS split — ordinary CI
  engineering, each choice reasoned in its own comment, none of it changing
  what the compiler accepts.
- `whitefootc`'s default of producing a linked executable unless
  `--emit-llvm` is given — the same default shape as any ordinary compiler
  driver (gcc, clang, rustc).
- CLI hygiene: mutually exclusive flags (`--par` and `--no-overlap` together
  is refused), stdout-sharing refusals (`--par-ledger`/`--stack-ledger` with
  `--emit-llvm` and no `-o`), and "written only once" checks on every valued
  flag — ordinary fail-loud argument validation.

## 3. Choices without a node

**1. `CompilerLimits::default()`'s fixed numbers are the one production
resource-ceiling profile every caller uses, with no override and no recorded
reason — reads as an unexamined placeholder, not a deliberate choice.**
`CompilerLimits` (`driver.rs:64-125`) bundles the five stage-local ceiling
structs (`SourceLimits`, `LexLimits`, `TerminalLimits`, `ParseLimits`,
`FinalizeLimits`, `CanonicalLimits`) and, unlike every one of them, supplies a
`Default` impl with concrete numbers: 1,024 sources; 16 MiB one source / 64
MiB total / 128 MiB one binding; 1 MiB one token / 8M tokens / 16M lexemes;
256 MiB parser work / 8M tasks / 65,536 frames / 16M elements; matching
finalizer and canonical numbers. `whitefootc`, the shipped binary, calls
`CompilerLimits::default()` unconditionally at both of its call sites
(`whitefootc.rs:252,267`) and its `Options` (lines 529-608) has no
limit-related field at all — no flag, no environment variable, nothing raises,
lowers, or otherwise selects any individual ceiling. Crate-wide,
`CompilerLimits { .. }` is constructed nowhere except inside this one
`Default` impl: every one of its other 54 call sites (every test, and both
CLI entry points) reads `::default()` verbatim.
Alternative: give `CompilerLimits` no `Default` at all, forcing every caller —
starting with `whitefootc` — to state its own ceilings explicitly, which is
exactly the policy `LexLimits`'s own doc comment states for its type ("There
is deliberately no default or unbounded production profile. The caller must
select every ceiling explicitly for its deployment,"
`compiler/src/lexer/outcome.rs:22-25`, quoted by the lexer-and-syntax audit's
choice 2); or keep a default but expose it on the CLI the way
`--par-scalar-leaf-limit` exposes its own numeric ceiling.
Where: `compiler/src/driver.rs:64-125` (`CompilerLimits`, `impl Default`);
`compiler/src/bin/whitefootc.rs:252,267` (the shipped binary's only two call
sites, both unconditional, lines 529-608 confirming `Options` carries no
limit override).
Reason: none recorded. `git log -S "impl Default for CompilerLimits" --
compiler/src/driver.rs` finds exactly one commit, `28e33780` (2026-07-22,
"Implement first executable scalar compiler slice"), bodiless beyond its
title; every number above is byte-identical between that commit's `driver.rs`
and the current file — 92 commits and about seven weeks of subsequent history
never touched them. No design node, commit, or PR record in
`design/recall-tmp/sources/` mentions `CompilerLimits` by name.
Effect: acceptance, not only structure. These numbers are what an ordinary,
non-adversarial program actually meets: a source file over 16 MiB, or a
parse/finalize pass over 8 million tasks or nodes, or 256 MiB of counted work,
stops with `CompilationFailureKind::Resource` rather than compiling, and no
`whitefootc` invocation can ask for more. Given `CLAUDE.md`'s stated purpose
("capable of compiling nontrivial programs" and "real programs that expose
language and compiler weaknesses"), an un-configurable, seven-week-old,
never-revisited placeholder profile is a plausible source of a defect with the
same shape as one `docs/todo.md` already tracks one layer up in the language's
own [PRF-1] ceiling rather than this implementation ceiling (its "large
`proof_use` block is impractical well below its ceiling" item) — a real
program tripping a number nobody chose for a reason, with no way to move it
short of recompiling the compiler.

**2. `whitefoot-grammar`'s "frontend contract" — the closed set of
specification sections whose bytes must match for a change to count as
grammar-preserving — is a compiler-policy definition with no design-tree
record, and it has already widened once with nothing saying so: reads as a
likely defect (an undecided boundary drifting), not a settled choice.**
`verify_candidate` (`grammar.rs:131-146`) classifies a candidate specification
against a baseline as `GrammarPreserving` when three named sections'
extracted bytes are unchanged ([FORM-1] through `## 4. Types`, [CONST-1]
through `## 5. Ownership`, [EFF-1] through [EFF-2] — `FRONTEND_SECTIONS`,
lines 22-26) and otherwise `StructuralGrammar`, but only if the candidate's
own contract exactly equals `frontend_contract(ACTIVE_KERNEL_SPEC_BYTES)` —
the compiler's own currently-embedded bytes — or the whole verification fails
closed (`VerifyError::ChangedFrontendContract`). Which sections belong in
that set is a real judgment call with a real alternative space, and it
decides which specification edits a human or agent must treat as "the
grammar tables need regenerating and the compiler needs rebuilding before
this can be classified" versus "prose-only, no frontend consequence." It has
also already moved once, silently: the section introduced the array with
*five* disjoint ranges — `("[FORM-3]","[FORM-4]")`, `("[FORM-5]","[FORM-6]")`,
`("[GRAM-1]","## 4. Types")`, plus today's [CONST-1]/[EFF-1] pair
(`894cd5ee`, 2026-07-22) — which left [FORM-1], [FORM-2] (the canonical-
rendering rule itself), and the gaps between FORM-4/FORM-5 and FORM-6/GRAM-1
*outside* the byte-compared contract. `c95bda9b` (2026-07-22, "Rename v0.11
Result propagation form" — an unrelated-sounding title) collapsed those three
ranges into today's single `("[FORM-1]","## 4. Types")`, which, checked
against `spec/kernel-spec-v0.11.md`'s line numbers ([FORM-1] at line 29,
[FORM-2] at 31, [GRAM-1] at 59, `## 4. Types` at 175), is not a same-content
simplification: it strictly widens the contract to include [FORM-1], [FORM-2],
and everything in the two previously-uncovered gaps. A prose edit to [FORM-2]
alone would not have tripped `ChangedFrontendContract` before that commit and
would after it, with nothing in the commit explaining the change either way.
Alternative: a different (larger or smaller) closed section set, chosen and
recorded once rather than able to widen or narrow inside an unrelated commit;
or classify by re-running the byte-for-byte generator comparison
(`grammar_tables::generate`) against both baseline and candidate instead of a
separate marker-derived "contract" extraction; or drop the two-tier
classification and always require the stronger structural check.
Where: `compiler/src/bin/grammar.rs:22-26` (`FRONTEND_SECTIONS`, today's
three-tuple form), `131-169` (`verify_candidate`, `frontend_contract`);
`compiler/src/bin/grammar_tables/ebnf.rs:1-21` (the seven `wf-ebnf`-fence
`Owner`s the sections are built from, a distinct and separately-maintained
list from `FRONTEND_SECTIONS`).
Reason: partially recovered for the comparison's existence, not recorded at
all for its scope. `d5ca6d7f` (2026-08-07, "compiler: give the grammar
verifier both specifications as arguments") explains why a *comparison* was
needed at all ("The workflow's mandatory pre-activation check runs it on the
candidate that is about to become the active specification, so on the run
that matters most the comparison was a file against itself and passed
whatever it was given"), but neither it nor `894cd5ee`/`c95bda9b` (both
bodiless beyond their titles) says why these particular sections, or why the
set was later widened. No design node discusses the classification or its
verification protocol; only `design/recall-tmp/sources/commits.md` shows the
tool in actual use across spec amendments (e.g. `685aec51`, 2026-08-17,
printing "structural grammar candidate verified by the active compiler" for
the v0.31 activation).
Effect: workflow, not acceptance — this binary is a human/agent-run
amendment-time tool, never wired into any Makefile target (only its own
`#[cfg(test)]` self-consistency checks against the currently-committed
specification run inside the gate). But it is the one place that decides,
for a given specification edit, whether the grammar tables must be
regenerated and the compiler rebuilt before the edit can be classified at
all — exactly the kind of "material choice" `design/skill/SKILL.md` asks to
be read and possibly renegotiated deliberately rather than left as an
artifact of whichever set someone typed first, and the one demonstrated
silent widening shows the boundary can and does move without anyone deciding
it should.

**3. A denied I/O loop is reported to the writer without a flag by default,
while every other permission verdict stays behind `--par-ledger`; only
`--no-overlap` silences it — a reasoned diagnostics-channel policy with no
tree record.**
`compile_with_io_notices` (`driver.rs:311-330`) and `io_notice_report`
(`whitefootc.rs:294-322`) together promote exactly two lines of the full
permission report — a `[PAR-3]` staged denial and the `[PAR-2]` counted
denial the staged judgment also reached — to stderr on every ordinary
compile and under `--par`, and print nothing under `--no-overlap`; every
other verdict (a granted loop, a read-only row, anything not a denial) stays
silent unless `--par-ledger` is named. The channel selection is deliberate
and reasoned at length in the driver's own test comments (`driver.rs:
712-724`: "The judgment was landed, correct, and unreachable: a writer
compiled five ordinary utilities, every I/O loop in them was denied, and
nothing said so, because the report was behind a flag they had no reason to
run"), but the specific policy — which two of the several possible verdict
rows are promoted, and that `--no-overlap` (a build-shape flag) rather than a
dedicated quiet flag is what turns the channel off — is not in `design/`
anywhere.
Alternative: print every denial regardless of flag; print nothing without
`--par-ledger`, matching every other verdict; or gate the notices on their
own flag instead of overloading `--no-overlap`'s meaning.
Where: `compiler/src/driver.rs:311-330` (`compile_with_io_notices`, its
doc comment, and the `permission_ledger.iter().filter(|line| line.notice)`
split from the full ledger); `compiler/src/bin/whitefootc.rs:259-272,
294-322` (`io_notice_report`, and its use in `run`).
Reason (quoted): the feature's introducing commit, `9acb3633` (2026-08-28,
"whitefootc: report a denied I/O loop without a flag"), is bodiless, but the
follow-up that silences it under `--no-overlap`, `8a18f654` (2026-08-28,
"io: quiet the denied-loop notes under --no-overlap, and teach the buffer
form"), states the reason directly: "That flag has already said this build
takes no overlap lowering, so a loop without a pipeline is the build it
asked for rather than news about the program. The decision is by the flag,
not by the text of a line." Both commits belong to the batch record
`archive/done/0100-writer-defaults-2.md` ("Batch 0100 — the defaults the
verification writer met, answered"), whose charter names the general
practice this instance follows ("change the compiler, or emit a warning or
diagnostic that teaches the fix; one of the two is mandatory ... a hidden
trick is never allowed") — the general principle is close to
`docs/practice.md:121-123`'s "When improving a diagnostic, expose the
operation, required fact, relevant location, and missing or invalidated
evidence," but that page states a technique, not this specific channel
policy, and neither it nor any design node decides which verdicts are
writer-visible by default.
Effect: diagnostics only — the judgment is pure and every lowering reaches
the same verdicts and the same emitted module either way (pinned by
`a_no_overlap_build_reports_no_denied_io_loop`); what moves is only which
lines a writer sees without asking. A future change to the two-verdict
selection, or to what silences the channel, is currently a driver/CLI code
change with no tree entry to update alongside it.

## Summary

- Covered by the tree: 16
- No decision needed: 18
- Choices without a node: 3 (2 read as likely defects — one an unreasoned
  placeholder, one a boundary shown to have already drifted silently once —
  and 1 a reasoned-in-commits diagnostics policy the tree never received)
