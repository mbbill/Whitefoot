# Audit: lexer and syntax against the design tree

Modules: `compiler/src/lexer/` (~1,125 lines incl. tests) and
`compiler/src/syntax/` (~20,100 lines incl. tests and the generated grammar
tables). Direction: code to tree — finding choices the code embodies that
`design/` does not record, not the reverse.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md`, `design/language.md`,
`design/language/surface-form.md` and its five children, and
`design/language/name-resolution.md`. Also read for grounding:
`spec/kernel-spec.md` §§1-3 (scope, canonical form, grammar) and §12
(diagnostics), since both modules exist to implement those sections.

Two scope notes that shape section 1:

- Several rule IDs that read as grammar rules are actually enforced in
  `compiler/src/semantic/` or `compiler/src/resolution/`, not here, because
  they need type or name information the parser does not have: GRAM-6
  (if/match selection by scrutinee type, empty/nested `else`), GRAM-7's
  delivery completeness, GRAM-8 (declared-order/spelling of named
  construction), GRAM-10 (match-binder freshness), and GRAM-11 (named call
  arguments). `SyntaxRule` in `compiler/src/syntax/parser/outcome.rs` has no
  `Gram6`/`Gram7`/`Gram8`/`Gram10`/`Gram11` variant, which confirms this from
  the code side. So `design/language/surface-form/match-form.md` and
  `construction-form.md` are covered only where noted below; the rest of
  their decisions have no implementing code in these two modules at all.
- `design/language/name-resolution.md` is not implemented in either module
  (it is `compiler/src/resolution/`'s concern); it is omitted from section 1
  rather than force-fit.

Git history note, relevant to "no reason recorded" below: this repository's
visible history has one root commit, `65b3d24` (2026-08-28), a single
3,134-file import of the whole pre-existing project with no parent (recorded
independently in `design/recall-tmp/sources.md`). `git blame` traces nearly
all of both modules' substantive code to that one commit, whose message
("wip: TYPE-5 and OWN-10 publish the two sides they compared") does not
address lexer or parser architecture. Fine-grained commits and PRs exist only
from that date forward (`design/recall-tmp/sources/commits.md` and
`pull-requests.md`); where they contain relevant material it is cited below.

## 1. Covered by the tree

- `design/compiler.md` (decision 1: safe-Rust research compiler, unsafe
  forbidden) — `compiler/src/lib.rs:1` `#![forbid(unsafe_code)]`, crate-wide;
  neither module contains an `unsafe` block.
- `design/compiler.md` (decision 2: admits/lowers by the specification's
  rules alone, never by recognizing a name, signature, or shape) —
  `compiler/src/syntax/grammar.rs` + `grammar/generated.rs` (tables
  mechanically derived from `spec/kernel-spec.md`'s EBNF) and
  `compiler/src/syntax/parser/engine.rs::parse`/`execute_node` (derivation is
  table-driven over token predicates; no identifier or program is special-cased).
- `design/compiler.md` (decision 3: an unimplemented-but-valid source stops as
  an explicit unsupported capability, never reported as invalid; decision 4:
  no acceptance path depends on a timeout, fuel, or work budget) — every
  stage keeps `CompilerFailure`/`ResourceFailure` structurally distinct from
  `SourceIssue`: `compiler/src/lexer/outcome.rs::LexOutcome`,
  `compiler/src/syntax/outcome.rs::TerminalOutcome`,
  `compiler/src/syntax/parser/outcome.rs::ParseOutcome` (+ `Work::spend`,
  which turns an exhausted deterministic work counter into a
  `ResourceFailure`, never a rejection), `compiler/src/syntax/parser/finalize/outcome.rs::FinalizeOutcome`/`CanonicalOutcome`.
- `design/compiler.md` (decision 4: a rejection names the numbered rule and
  the location) — `compiler/src/syntax/parser/outcome.rs::SyntaxIssue`/`SyntaxRule::id`;
  `compiler/src/syntax/parser/diagnostic.rs::diagnose_decision`/`frontier`/`override_issue`
  (the DIAG-1 attribution machinery).
- `design/language/surface-form.md` (decision 1: one spelling and one
  byte-level formatting; non-canonical input rejected, never reformatted) —
  `compiler/src/syntax/parser/finalize/canonical.rs::audit_canonical`;
  `canonical/format.rs::build_gap_styles`/`canonical_gap`.
- `design/language/surface-form.md` (decision 2: no comments; documentation
  lives in `doc`) — `compiler/src/lexer/scanner.rs::Scanner::next`
  (`CommentPrefix` arm rejects `//`/`/*`); the `doc` production (GRAM-2) in
  `grammar/generated.rs`.
- `design/language/surface-form.md` (decision 3: flat three-address form, one
  operation per expression) — `compiler/src/syntax/parser/diagnostic.rs::forbidden_atom_override`
  (GRAM-9); the `atom` production admits no nested `call`/`construct`.
- `design/language/surface-form.md` (decision 4: a body binder's mode and
  type are derived, never written) — `compiler/src/syntax/grammar/generated.rs`:
  `Production::OrdinaryLetRhs`/`PropagateLetRhs`/`ReplaceLetRhs` carry no
  mode/type slot, unlike `Production::Param`/`ResultBinding`, which do.
- `design/language/surface-form.md` (decision 5: two iteration forms —
  loop+break, ascending half-open counted `for`) — `grammar/generated.rs`:
  `Production::LoopStmt`/`ForStmt`/`ForBinding`/`HeaderInvariant`; header
  formatting in `compiler/src/syntax/parser/finalize/canonical/format.rs::build_gap_styles`
  (the `ForStmt`/`LoopStmt` branch).
- `design/language/surface-form/borrow-lexicon.md` (decision 2: the exclusive
  mode is `&uniq`, not `&mut`) — `compiler/src/syntax/terminal.rs::FixedTerminal::Uniq`
  (there is no `Mut` terminal in the inventory at all); `mode` production
  (GRAM-3).
- `design/language/surface-form/operation-spelling.md` (decision 2: explicit
  `wrap`/`defined`/`checked`/`sat`/`strict` modes, no bare trapping operator)
  — `compiler/src/lexer/scanner.rs::operation_name_end` (the closed
  five-suffix OPNAME set); `compiler/src/syntax/terminal.rs::is_operation_name`/`FixedTerminal::is_operator_form`.
- `design/language/surface-form/operation-spelling.md` (decision 3: the
  call-site `::` delimiter dissolves the `<` collision between comparison and
  type application) — the grammar's `call` vs. `infix_tail` SELECT_2 rows are
  disjoint by construction (`grammar/generated.rs`), exercised by
  `compiler/src/syntax/parser/tests.rs::bare_angle_after_a_name_is_a_comparison_and_type_application_needs_its_delimiter`.
- `design/language/surface-form/operation-spelling.md` (decision 4: `use N
  times (relation)`, a relation premise always parenthesized with a stated
  space) — `compiler/src/syntax/parser/finalize/canonical/format.rs::build_gap_styles`
  (the `Production::UsePremise` branch).

## 2. No decision needed

Lexer:
- Hand-written, single-pass, byte-at-a-time character-class scanner (no
  regex/lexer-generator) — ordinary technique for a small fixed alphabet.
- Two-stage token model: a permissive raw-shape scan (`scanner.rs`) feeding a
  stricter, separate terminal-membership check (`classifier.rs`) — GRAM-1
  fixes this split, not the lexer.
- Spaces and line feeds are retained as first-class trivia lexemes for
  lossless, byte-exact reconstruction — required by FORM-2.
- Numeric raw scanning is deliberately over-broad (letters/digits/`_`/`.`/
  exponent sign) and defers exact well-formedness to later stages —
  spec-mandated (GRAM-1 names `1e+`/`1.00_f64` as intended raw candidates).
- Multi-byte UTF-8 scalars are decoded by hand at each cursor rather than via
  a whole-file `str::from_utf8`, so one bad sequence is pinpointed to its own
  span — required by DIAG-1's byte-exact defect spans.
- Resource ceilings surface as a distinct `ResourceFailure` outcome, never a
  panic or a language rejection.
- `TokenKind`/`TriviaKind` are closed enums, one variant per exact
  punctuation shape, rather than a generic `Punct(byte)` — ordinary
  type-safety practice.

Syntax:
- Table-driven strong-LL(2) predictive parsing, fixed two-token lookahead, no
  backtracking, no precedence climbing — GRAM-1 fixes this as the grammar's
  own discipline.
- `grammar/generated.rs` is a mechanical transcription of the specification's
  EBNF, produced and checked by a separate offline generator
  (`compiler/src/bin/grammar_tables/`, outside this audit); its content is
  not an independent compiler decision.
- No error recovery: derivation stops at the first defect per stage and
  reports it — DIAG-1 mandates exactly this, not panic-mode synchronization
  or multi-error batching.
- Diagnostic attribution (which token/rule/expected set to report) follows a
  fixed scoring and attribution procedure transcribed near-verbatim from
  DIAG-1, not an independent compiler heuristic.
- Terminal membership is a bitset of every matching predicate (`TerminalSet`,
  a `u128`), never a single priority-chosen kind — GRAM-1 requires retaining
  every match.
- Every stage (classify/parse/finalize/canonical-audit) is a function
  returning one closed outcome enum with `SourceIssue`/`ResourceFailure`/
  `CompilerFailure` — mirrors DIAG-1's stage order and failure categories.
- Canonical checking streams a comparison against the finalized tree's
  computed gap bytes rather than fully rendering then diffing; a from-scratch
  renderer (`render_canonical`) shares the same layout logic but has no
  production caller, only test fixtures.
- `NodePath` (DIAG-1's child-ordinal path from the root) is a parent-pointer
  walk to the root — the direct encoding of DIAG-1's own definition.
- Checked/saturating arithmetic throughout instead of raw arithmetic —
  ordinary overflow-safety practice, unrelated to language semantics.
- Extensive fixture-driven tests, including grammar shared-prefix/angle
  disambiguation cases and a whole-corpus tree-shape check
  (`finalize/tests/corpus_shape.rs`) — ordinary test engineering.

## 3. Choices without a node

**1. Two-pass, count-then-allocate lexer.**
`lex_shapes` scans every source file twice: once to count lexemes, tokens,
and bytes against the caller's ceilings without allocating, once more to
build the final `Vec<Lexeme>` at exactly that capacity, treating any
disagreement between the two passes as an internal `CompilerFailure`
(`PassDisagreement`/`PassCountDisagreement`) rather than trusting the first
count.
Alternative: one pass appending to a growable `Vec` (the ordinary approach),
or a size estimate with reallocation as needed.
Where: `compiler/src/lexer/scanner.rs::lex_shapes` (first pass ~L325-410,
second pass ~L410-495); `compiler/src/lexer/outcome.rs::LexCompilerFailure`.
Reason: none recorded. Git blame attributes the function to the single root
import commit `65b3d24`, whose message does not discuss the lexer.
Effect: performance only (every source is scanned twice); does not change
which programs are accepted.

**2. Every resource ceiling is mandatory, with no built-in default.**
`LexLimits`, and by the same pattern `TerminalLimits`, `ParseLimits`,
`FinalizeLimits`, and `CanonicalLimits`, have no `Default` impl; every
embedder (including every test in both modules) must name every ceiling
explicitly.
Alternative: ship a built-in default profile, or leave limits unbounded
unless overridden.
Where: `compiler/src/lexer/outcome.rs::LexLimits` (doc comment, L22-25);
mirrored with no stated reason by `TerminalLimits`
(`compiler/src/syntax/outcome.rs`), `ParseLimits`
(`compiler/src/syntax/parser/outcome.rs`), `FinalizeLimits`/`CanonicalLimits`
(`compiler/src/syntax/parser/finalize/outcome.rs`).
Reason (quoted, `compiler/src/lexer/outcome.rs:24-25`): "There is deliberately
no default or unbounded production profile. The caller must select every
ceiling explicitly for its deployment."
Effect: structural/API only inside these two modules; a program's actual
acceptance still depends on whichever concrete numbers a caller supplies
(the driver's numbers live in `compiler/src/driver.rs`, outside this audit).

**3. Grammar derivation, its diagnostic re-walk, and shape re-verification
are hand-rolled iterative task-stack machines, never native recursion.**
The parser (`engine.rs`), the DIAG-1 diagnostic probe (`diagnostic.rs::probe`),
and the finalizer's independent shape check (`finalize/shape.rs::verify`)
each keep an explicit `Vec`-based task stack and loop over push/pop, instead
of recursing through the grammar via ordinary function calls; nesting depth
in the source is bounded only by caller-chosen ceilings
(`ParseLimits::max_tasks`/`max_frames`, `FinalizeLimits::max_shape_tasks`),
never by the native call stack.
Alternative: ordinary recursive descent, which the specification explicitly
allows as an equal alternative to a table-driven engine
(`spec/kernel-spec.md:2250`: "A recursive-descent or table-driven
implementation must report the result of this same source-EBNF diagnostic
machine").
Where: `compiler/src/syntax/parser/engine.rs` (`struct Parser`,
`execute_node`, `parse_source`); `parser/diagnostic.rs::probe`;
`parser/finalize/shape.rs::verify`.
Reason: none recorded; blame again resolves to `65b3d24`.
Effect: both. Performance: an explicit stack costs more per node than a
direct call, in exchange for depth bounded by policy rather than guesswork.
Safety/acceptance: a deeply nested but otherwise valid construct fails
cleanly with a `ResourceFailure` instead of an unguarded stack overflow — and
that exact hazard is live one layer downstream and still open: `docs/todo.md`
("Unguarded affine expression nesting depth") records the driver aborting
(exit 134, no diagnostic) on an `affine_expr` nested about 1400 parentheses
deep, reachable from a `use` premise or an `invariant` target. The commit
that recorded it (`b50d21f`, 2026-09-05, "Record the affine-nesting crash
where gaps are recorded") says the fix "wants a bisect between the parser and
the semantic former" — as of that commit, still unresolved which side owns
it. `affine_expr` is parsed by the very engine described here, and neither it
nor its finalizer contains native recursion; the `AffineCheckError` type the
todo item's proposed repair would extend lives entirely in
`compiler/src/semantic/entailment/`, outside these two modules — so this
audit did not find the crash's cause inside `lexer`/`syntax`, but the
project's own record does not yet rule either side out.

**4. The frontend never panics, and each stage re-derives what the previous
stage should already guarantee instead of trusting it.**
No `unwrap`, `expect`, `panic!`, `unreachable!`, or `todo!` appears in either
module's production code (confirmed by search; every occurrence is inside
`#[cfg(test)]`); every internal invariant is threaded through a typed
`*CompilerFailure` variant instead. On top of that, later stages
independently re-verify facts earlier stages already established:
`finalize/shape.rs::verify` re-runs the grammar's own LL(2) arm selection to
re-derive each production's expected child shape from scratch and rejects
(as a `CompilerFailure`, not a source rejection) if it disagrees with what
the parser produced; `finalize/engine.rs` and `finalize/canonical.rs`
similarly re-check terminal-predicate membership, token identity, and
source-byte coverage the parser and classifier already checked.
Alternative: ordinary panics/`assert!`/`unreachable!()` for "should never
happen" states, caught (if at all) at one outer boundary; trust each stage's
output at the next stage's entry.
Where: pervasive; clearest single illustration is
`compiler/src/syntax/parser/finalize/shape.rs::verify_production_shape`,
which the test suite names for exactly this property —
`compiler/src/syntax/parser/tests.rs:672`:
`panic!("the all-production derivation must pass the independent shape
finalizer")`.
Reason: none stated; consistent with but not derived from the crate's
`#![forbid(unsafe_code)]` (`compiler/src/lib.rs:1`), which is about memory
safety, not panic-freedom or cross-stage re-verification.
Effect: safety/structure, not acceptance or speed for a valid program: a
compiler-side bug becomes a reported `CompilerFailure` instead of an unwind
or crash, at the cost of a large amount of duplicated verification code and,
in the finalizer's case, a second full LL(2) derivation of every production.

**5. A curated, non-uniform subset of syntax diagnostics carries a
hand-written "mechanical fix"; the rest carry only the expected-terminal
list.**
`SyntaxIssue::mechanical_fix` is populated with bespoke prose for exactly
three situations — a `call`/`construct` written where only an `atom` is
allowed (two versions, body vs. `contract_block`), a name written in the
wrong lexical class (four versions, one per IDENT/TYPEID/REGIONID/LABEL
slot), and a `contract_block` clause out of section order — and left `None`
for every other syntax rule (FORM-1, FORM-2, FORM-5, most of GRAM-3/4/5,
CONST-1, CONST-2, EFF-1).
Alternative: give every `SyntaxIssue` a fix, or none at all, relying solely
on the expected-terminal list as most rules already do.
Where: `compiler/src/syntax/parser/diagnostic.rs`, constants
`GRAM9_BODY_FIX`/`GRAM9_CONTRACT_FIX`/`FORM3_IDENT_FIX`/`FORM3_TYPEID_FIX`/
`FORM3_REGIONID_FIX`/`FORM3_LABEL_FIX`/`GRAM2_CONTRACT_ORDER_FIX`, and
functions `name_class_fix`, `production_fix`.
Reason (partial, quoted, `diagnostic.rs:60-61`, motivating the name-class
case only): "The expectation list names the class and never says what the
class is, so a writer who spelled a const `Limit` read only `expected:
[\"IDENT\"]`." Nothing explains why this specific set of rules, and not
others that leave the same gap, was chosen; the specification does not
require fix text for any of these grammar-stage rules (`spec/kernel-spec.md`
mandates fix wording only for later semantic rules, e.g. FORM-8, GIVE-1,
SET-2). Commit `39da854` (2026-08-28) only corrects a doc comment's claim
that `mechanical_fix` "had no caller"; it does not address the selection.
Effect: diagnostics quality/consistency only; no effect on acceptance or
performance.

**6. The parse tree is a flat postorder array, finalized into an
index-addressed topology by a wholly separate second pass, rather than a
directly linked tree built during parsing.**
`engine.rs` appends terminals and finished productions to one flat
`Vec<DerivationElement>` in postorder (a production entry records only its
child count and subtree size, no child pointers); a separate `finalize` stage
re-walks that array once to build `FinalizedTopology` — parallel arrays of
`NodeRecord` (parent id, child-index range, tree/format depth) and a flat
`children: Vec<NodeId>` edge list — which canonical rendering/audit,
`NodePath`, and `entry_form`'s scan all read instead of the parser's own
output.
Alternative: a directly linked tree built in one pass (e.g. `enum Node {
Terminal(Token), Production(Production, Vec<Node>) }`), avoiding a second
full pass and its own resource ceilings
(`FinalizeLimits::max_roots`/`max_nodes`/`max_child_edges`/`max_shape_tasks`).
Where: `compiler/src/syntax/parser/tree.rs` (`DerivationElement`,
`DerivationTree`); `parser/finalize/topology.rs` (`NodeRecord`,
`FinalizedTopology`); `parser/finalize/engine.rs` (`Finalizer::run`,
`production`, `assign_depths`).
Reason: none stated for the split itself. `canonical/render.rs:3-8` explains
only why the renderer and auditor share logic once the topology exists
("They share the layout rules rather than restating them ... so a change to
either rule moves the auditor and the renderer together"), not why the
topology is a second pass rather than built during parsing.
Effect: performance/structure, not acceptance: a flat, index-addressed tree
is cache-friendly and makes `NodePath` a direct parent-pointer walk
(`canonical.rs::node_path`), at the cost of a second full linear pass over
every parsed node plus its own duplicated resource-ceiling machinery.

## Summary

- Covered by the tree: 13
- No decision needed: 17 (7 lexer, 10 syntax)
- Choices without a node: 6
