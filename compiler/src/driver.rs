//! One ordinary active-specification compilation pipeline.
//!
//! The driver keeps source failures, unsupported compiler capabilities,
//! resource failures, invariant failures, lowering failures, and backend
//! failures distinct while returning owned LLVM assembly to callers.

use core::fmt;

use crate::{
    ACTIVE_KERNEL_SPEC_HASH, BackendFailure, CanonicalLimits, CanonicalOutcome, FinalizeLimits,
    FinalizeOutcome, LexLimits, LexOutcome, LoweringFailure, ParseLimits, ParseOutcome,
    ResolutionOutcome, SemanticOutcome, SourceBundle, SourceInput, SourceLimits, TerminalLimits,
    TerminalOutcome, audit_canonical, check_semantics, classify_terminals, emit_llvm, finalize,
    lex, lower_checked, parse, resolve_with_inventory,
};

/// Host-compiler optimization arguments for every Whitefoot executable.
///
/// One definition serves the driver executable and every test that links an
/// emitted module, so no path can silently link an unoptimized binary while
/// another links an optimized one. There is no writer-facing switch: the
/// optimization level cannot change which programs are accepted, discharge a
/// static source obligation, or make a potentially failing claim disappear,
/// so no writer decision exists and the default shape is the only shape. The
/// level is provisional and may move once a measurement asks for it.
pub const HOST_OPTIMIZATION_ARGUMENTS: &[&str] = &["-O2"];

/// Explicit implementation ceilings for one compiler invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompilerLimits {
    /// Ordered source-envelope limits.
    pub source: SourceLimits,
    /// Lossless lexical limits.
    pub lexer: LexLimits,
    /// Terminal-classification limits.
    pub terminals: TerminalLimits,
    /// Predictive parser limits.
    pub parser: ParseLimits,
    /// Finalized-tree limits.
    pub finalizer: FinalizeLimits,
    /// Canonical-source audit limits.
    pub canonical: CanonicalLimits,
}

impl Default for CompilerLimits {
    fn default() -> Self {
        Self {
            source: SourceLimits {
                max_sources: 1_024,
                max_logical_path_bytes: 4_096,
                max_source_bytes: 16 * 1_024 * 1_024,
                max_total_source_bytes: 64 * 1_024 * 1_024,
                max_binding_bytes: 128 * 1_024 * 1_024,
            },
            lexer: LexLimits {
                max_sources: 1_024,
                max_source_bytes: 16 * 1_024 * 1_024,
                max_total_source_bytes: 64 * 1_024 * 1_024,
                max_token_bytes: 1_024 * 1_024,
                max_tokens: 8 * 1_024 * 1_024,
                max_lexemes: 16 * 1_024 * 1_024,
            },
            terminals: TerminalLimits {
                max_tokens: 8 * 1_024 * 1_024,
            },
            parser: ParseLimits {
                max_work: 256 * 1_024 * 1_024,
                max_tasks: 8 * 1_024 * 1_024,
                max_frames: 65_536,
                max_elements: 16 * 1_024 * 1_024,
            },
            finalizer: FinalizeLimits {
                max_work: 256 * 1_024 * 1_024,
                max_roots: 8 * 1_024 * 1_024,
                max_shape_tasks: 8 * 1_024 * 1_024,
                max_nodes: 8 * 1_024 * 1_024,
                max_child_edges: 8 * 1_024 * 1_024,
                max_terminals: 8 * 1_024 * 1_024,
                max_sources: 1_024,
            },
            canonical: CanonicalLimits {
                max_work: 256 * 1_024 * 1_024,
                max_source_bytes: 16 * 1_024 * 1_024,
                max_total_source_bytes: 64 * 1_024 * 1_024,
                max_gaps: 8 * 1_024 * 1_024,
                max_path_components: 65_536,
            },
        }
    }
}

/// Compiler stage at which one invocation stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilationStage {
    /// PROG-2 source envelope.
    SourceEnvelope,
    /// Raw lossless lexing.
    Lexing,
    /// Context-free terminal membership.
    TerminalClassification,
    /// Strong-LL(2) grammar derivation.
    Parsing,
    /// Finalized production topology.
    Finalization,
    /// Exact FORM-2 source audit.
    CanonicalSource,
    /// Declaration and lexical-use resolution.
    Resolution,
    /// Target-independent semantic checking.
    Semantics,
    /// Checked-program to target-independent IR lowering.
    Lowering,
    /// Selected-target representability and target-domain discharge.
    TargetLayout,
    /// [QUAL-1] system-interface target qualification.
    TargetQualification,
    /// Conservative textual LLVM emission.
    Backend,
}

/// Category of compiler stop, independent of the stage that reported it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilationFailureKind {
    /// A numbered source-language rule was violated.
    Source,
    /// Valid source requires an unimplemented compiler capability.
    Unsupported,
    /// An explicit implementation ceiling or host storage stopped work.
    Resource,
    /// The caller supplied an invalid compilation envelope or stage identity.
    Invocation,
    /// A trusted compiler invariant failed.
    Compiler,
    /// Checked-program to IR lowering failed internally.
    Lowering,
    /// A statically materialized object is not representable on the selected target.
    TargetLayout,
    /// The selected target has no approved implementation of a system
    /// operation the program uses, or does not supply a guarantee that
    /// operation's record requires [QUAL-1, QUAL-2].
    TargetQualification,
    /// LLVM emission failed internally.
    Backend,
}

/// One compiler stop with its category preserved in the detail text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilationFailure {
    stage: CompilationStage,
    kind: CompilationFailureKind,
    rule_id: Option<&'static str>,
    detail: String,
}

impl CompilationFailure {
    fn new(stage: CompilationStage, kind: CompilationFailureKind, detail: impl fmt::Debug) -> Self {
        Self {
            stage,
            kind,
            rule_id: None,
            detail: format!("{detail:?}"),
        }
    }

    /// One source-language rejection carrying the rule its stage attributed.
    ///
    /// Every stage that can reject source already selects exactly one numbered
    /// rule under DIAG-1; this constructor only publishes that selection, so a
    /// caller comparing cited rules sees the same attribution at every stage.
    fn source(stage: CompilationStage, rule_id: &'static str, detail: impl fmt::Debug) -> Self {
        Self {
            stage,
            kind: CompilationFailureKind::Source,
            rule_id: Some(rule_id),
            detail: format!("{detail:?}"),
        }
    }

    /// Returns the stage that did not produce a complete result.
    #[must_use]
    pub const fn stage(&self) -> CompilationStage {
        self.stage
    }

    /// Returns the source/unsupported/resource/invocation/internal category.
    #[must_use]
    pub const fn kind(&self) -> CompilationFailureKind {
        self.kind
    }

    /// Returns the structured debug detail retained by that stage.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// Returns the exact numbered source rule this rejection cites.
    ///
    /// Present for every [`CompilationFailureKind::Source`] stop, at whichever
    /// stage selected it, and absent for every stop that is not a
    /// source-language rejection and therefore cites no language rule
    /// [DIAG-1].
    #[must_use]
    pub const fn rule_id(&self) -> Option<&'static str> {
        self.rule_id
    }
}

impl fmt::Display for CompilationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(rule_id) = self.rule_id {
            write!(
                formatter,
                "{:?}/{:?} [{rule_id}]: {}",
                self.stage, self.kind, self.detail
            )
        } else {
            write!(
                formatter,
                "{:?}/{:?}: {}",
                self.stage, self.kind, self.detail
            )
        }
    }
}

impl std::error::Error for CompilationFailure {}

/// Compiles one ordered closed source bundle to conservative textual LLVM.
///
/// This is the shipped default, and it performs no overlap lowering: see
/// [`compile_with_overlap`] for the compilation `whitefootc --par` performs.
pub fn compile(
    inputs: &[SourceInput<'_>],
    limits: CompilerLimits,
) -> Result<String, CompilationFailure> {
    compile_with_inventory(inputs, limits, crate::Inventory::ACTIVE)
}

/// [`compile`] with the [PAR-1 candidate] overlap lowering named explicitly.
///
/// [`crate::OverlapLowering::Off`] emits the module a compiler without this
/// path emits; [`crate::OverlapLowering::On`] hands every eligible group the
/// lowering can carry to a worker lane. The judgment runs either way — it is
/// pure, it changes no accepted program, and its ledger is identical — so this
/// selects an emitted lowering and nothing else.
pub fn compile_with_overlap(
    inputs: &[SourceInput<'_>],
    limits: CompilerLimits,
    overlap: crate::OverlapLowering,
) -> Result<String, CompilationFailure> {
    compile_reporting(inputs, limits, crate::Inventory::ACTIVE, overlap).map(|(module, _)| module)
}

/// [`compile_with_overlap`] plus the non-normative permission ledger for the
/// same compilation.
///
/// The ledger reports, one line per analyzed sibling-call site, whether the
/// permission judgment allows overlapping the two statements and whether a
/// permitted overlap is actualizable. It is developer output on the caller's
/// own channel: it participates in no mandatory record, changes no accepted
/// program, and selects no lowering — the same lines are reported whether or
/// not this compilation actualizes any of them. `whitefootc --par-ledger` is
/// its one caller outside tests.
pub fn compile_with_permission_ledger(
    inputs: &[SourceInput<'_>],
    limits: CompilerLimits,
    overlap: crate::OverlapLowering,
) -> Result<(String, Vec<String>), CompilationFailure> {
    compile_reporting(inputs, limits, crate::Inventory::ACTIVE, overlap)
}

/// [`compile`] against one named [SYS-2] inventory state.
///
/// `inventory` selects which prefix of the [SYS-2] tables the compilation
/// admits. It exists so an end-to-end test can compile and run a real program
/// against an inventory before activation, and so the differential against an
/// earlier inventory stays reachable afterward; the shipped compilation path
/// reads [`crate::Inventory::ACTIVE`] and has exactly one inventory. Historical
/// prefix states remain test-only differentials, never runtime switches.
pub fn compile_with_inventory(
    inputs: &[SourceInput<'_>],
    limits: CompilerLimits,
    inventory: crate::Inventory,
) -> Result<String, CompilationFailure> {
    compile_reporting(inputs, limits, inventory, crate::OverlapLowering::Off)
        .map(|(module, _)| module)
}

/// The one compilation path, returning the module and the developer-channel
/// permission ledger it produced. Every public entry point above is a
/// projection of this function; there is no second pipeline.
fn compile_reporting(
    inputs: &[SourceInput<'_>],
    limits: CompilerLimits,
    inventory: crate::Inventory,
    overlap: crate::OverlapLowering,
) -> Result<(String, Vec<String>), CompilationFailure> {
    let bundle = SourceBundle::with_limits(inputs, limits.source).map_err(|failure| {
        CompilationFailure::new(
            CompilationStage::SourceEnvelope,
            CompilationFailureKind::Invocation,
            failure,
        )
    })?;
    let lexed = match lex(&bundle, limits.lexer) {
        LexOutcome::Complete(complete) => complete,
        LexOutcome::SourceIssue(issue) => {
            return Err(CompilationFailure::source(
                CompilationStage::Lexing,
                issue.kind().rule_id(),
                issue,
            ));
        }
        LexOutcome::ResourceFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Lexing,
                CompilationFailureKind::Resource,
                failure,
            ));
        }
        LexOutcome::CompilerFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Lexing,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let classified = match classify_terminals(&lexed, ACTIVE_KERNEL_SPEC_HASH, limits.terminals) {
        TerminalOutcome::Complete(complete) => complete,
        TerminalOutcome::SourceIssue(issue) => {
            return Err(CompilationFailure::source(
                CompilationStage::TerminalClassification,
                issue.owner().id(),
                issue,
            ));
        }
        TerminalOutcome::ResourceFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::TerminalClassification,
                CompilationFailureKind::Resource,
                failure,
            ));
        }
        TerminalOutcome::InvocationFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::TerminalClassification,
                CompilationFailureKind::Invocation,
                failure,
            ));
        }
        TerminalOutcome::CompilerFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::TerminalClassification,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let parsed = match parse(&classified, limits.parser) {
        ParseOutcome::Complete(complete) => complete,
        ParseOutcome::SourceIssue(issue) => {
            return Err(CompilationFailure::source(
                CompilationStage::Parsing,
                issue.rule().id(),
                issue,
            ));
        }
        ParseOutcome::ResourceFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Parsing,
                CompilationFailureKind::Resource,
                failure,
            ));
        }
        ParseOutcome::InvocationFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Parsing,
                CompilationFailureKind::Invocation,
                failure,
            ));
        }
        ParseOutcome::CompilerFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Parsing,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let finalized = match finalize(parsed, limits.finalizer) {
        FinalizeOutcome::Complete(complete) => complete,
        FinalizeOutcome::ResourceFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Finalization,
                CompilationFailureKind::Resource,
                failure,
            ));
        }
        FinalizeOutcome::CompilerFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Finalization,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let canonical = match audit_canonical(finalized, limits.canonical) {
        CanonicalOutcome::Complete(complete) => complete,
        CanonicalOutcome::SourceIssue(issue) => {
            return Err(CompilationFailure::source(
                CompilationStage::CanonicalSource,
                issue.rule().id(),
                issue,
            ));
        }
        CanonicalOutcome::ResourceFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::CanonicalSource,
                CompilationFailureKind::Resource,
                failure,
            ));
        }
        CanonicalOutcome::CompilerFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::CanonicalSource,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let resolved = match resolve_with_inventory(canonical, inventory) {
        ResolutionOutcome::Complete(complete) => complete,
        ResolutionOutcome::SourceIssue { issue, .. } => {
            return Err(CompilationFailure::source(
                CompilationStage::Resolution,
                issue.rule().id(),
                issue,
            ));
        }
        ResolutionOutcome::CompilerFailure { failure, .. } => {
            return Err(CompilationFailure::new(
                CompilationStage::Resolution,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let checked = match check_semantics(resolved) {
        SemanticOutcome::Complete(complete) => *complete,
        SemanticOutcome::SourceIssue { issue, .. } => {
            return Err(CompilationFailure::source(
                CompilationStage::Semantics,
                issue.rule_id(),
                issue,
            ));
        }
        SemanticOutcome::ResolutionIssue { issue, .. } => {
            return Err(CompilationFailure::source(
                CompilationStage::Resolution,
                issue.rule().id(),
                issue,
            ));
        }
        SemanticOutcome::Unsupported { unsupported, .. } => {
            return Err(CompilationFailure::new(
                CompilationStage::Semantics,
                CompilationFailureKind::Unsupported,
                unsupported,
            ));
        }
        SemanticOutcome::CompilerFailure { failure, .. } => {
            return Err(CompilationFailure::new(
                CompilationStage::Semantics,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let mut permission_ledger = checked.data.permission_ledger.clone();
    let ir = lower_checked(checked, overlap).map_err(|failure: LoweringFailure| {
        CompilationFailure::new(
            CompilationStage::Lowering,
            CompilationFailureKind::Lowering,
            failure,
        )
    })?;
    // What this lowering did with each permission it was given, appended after
    // the judgment's own lines. The judgment reports the same verdicts with or
    // without `--par`; these lines report an actualization, which only a
    // compilation that asked for one has.
    permission_ledger.extend_from_slice(ir.actualization_ledger());
    emit_llvm(&ir)
        .map(|module| (module.into_string(), permission_ledger))
        .map_err(|failure: BackendFailure| {
            let (stage, kind) = match failure {
                BackendFailure::TargetLayout(_) => (
                    CompilationStage::TargetLayout,
                    CompilationFailureKind::TargetLayout,
                ),
                // A qualification stop is a target failure like a layout
                // stop: it is not a source-language rejection and cites no
                // language rule [DIAG-1].
                BackendFailure::TargetQualification(_) => (
                    CompilationStage::TargetQualification,
                    CompilationFailureKind::TargetQualification,
                ),
                _ => (CompilationStage::Backend, CompilationFailureKind::Backend),
            };
            CompilationFailure::new(stage, kind, failure)
        })
}

#[cfg(test)]
mod tests {
    use super::{
        CompilationFailureKind, CompilationStage, CompilerLimits, compile,
        compile_with_permission_ledger,
    };
    use crate::{OverlapLowering, SourceInput};

    /// The permission ledger of one compiled source, in the order the driver
    /// hands it to `whitefootc --par-ledger`.
    ///
    /// The judgment is pure, so the ledger belongs to the source and not to
    /// the lowering: this reads it from the default compilation, the one that
    /// actualizes nothing.
    fn ledger_of(name: &str, source: &[u8]) -> Vec<String> {
        let (_, ledger) = compile_with_permission_ledger(
            &[SourceInput::new(name, source)],
            CompilerLimits::default(),
            OverlapLowering::Off,
        )
        .expect("a permission-ledger fixture must compile");
        ledger
    }

    const TREE_PRELUDE: &str = "enum BoxNode {
  Leaf(w: u64);
  Branch(left: box<BoxNode>, right: box<BoxNode>, w: u64);
}

fn boxed_leaf(w: own u64) -> result: own box<BoxNode> allocates(heap) {
  let leaf = Leaf(w: w);
  return box_new(move leaf);
}

fn boxed_branch(left: own box<BoxNode>, right: own box<BoxNode>) -> result: own box<BoxNode> allocates(heap) {
  let branch = Branch(left: move left, right: move right, w: 0_u64);
  return box_new(move branch);
}

";

    /// The ledger states a permitted and eligible pair, and a permitted pair
    /// that a reachable claim keeps out of reach of actualization, with the
    /// claim count and the function the claim sits in. Both lines are exactly
    /// the developer-channel form of DESIGN section 4.
    #[test]
    fn the_permission_ledger_reports_eligible_and_not_actualizable_pairs() {
        let eligible = format!(
            "{TREE_PRELUDE}fn fold['b](node: &uniq 'b box<BoxNode>) -> result: own u64 reads('b), writes('b) {{
  match deref(deref(node)) {{
    Leaf(w: leaf_w) => {{
      return deref(leaf_w);
    }}
    Branch(left: l, right: r, w: slot) => {{
      let a = fold<'b>(node: move l);
      let b = fold<'b>(node: move r);
      let total = imax(a, b);
      set deref(slot) = total;
      return total;
    }}
  }}
}}

command fn main() -> status: own ExitStatus allocates(heap) {{
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  region 'tree {{
    let total = fold<'tree>(node: &uniq 'tree branch0);
  }}
  return exit_status(code: 0_u8);
}}
"
        );
        let ledger = ledger_of("fold.wf", eligible.as_bytes());
        assert_eq!(
            ledger[0],
            "PAR permitted   fold.wf:22  pair(fold, fold)  eligible"
        );
        // The chain the pair composes into, reported beside it. Pairs alone
        // cannot tell one three-member run from three separate two-member
        // ones, and those are completely different work, so the chain is
        // stated rather than left to be inferred.
        assert_eq!(
            ledger[1],
            "PAR chain       fold.wf:22  run(fold, fold)  2 members through line 23"
        );

        // The same tree fold with one claim in the recursive closure. P still
        // holds; the line reports why the overlap is not actualized.
        let claiming = format!(
            "{TREE_PRELUDE}fn bubble['b](node: &uniq 'b box<BoxNode>) -> result: own u64 reads('b), writes('b), traps {{
  match deref(deref(node)) {{
    Leaf(w: leaf_w) => {{
      return deref(leaf_w);
    }}
    Branch(left: l, right: r, w: slot) => {{
      let a = bubble<'b>(node: move l);
      let b = bubble<'b>(node: move r);
      let sum_defined = a +defined b;
      claim bubble_sum_fits: sum_defined because \"an in-memory tree has representable widths\";
      let total = a + b;
      set deref(slot) = total;
      return total;
    }}
  }}
}}

command fn main() -> status: own ExitStatus allocates(heap), traps {{
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  region 'tree {{
    let total = bubble<'tree>(node: &uniq 'tree branch0);
    claim bubble_total: ieq(total, 7_u64) because \"bubble total\";
  }}
  return exit_status(code: 0_u8);
}}
"
        );
        let ledger = ledger_of("bubble.wf", claiming.as_bytes());
        assert_eq!(
            ledger[0],
            "PAR permitted   bubble.wf:22  pair(bubble, bubble)  not-actualizable: 1 claim site via bubble"
        );

        // `bubble` is not actualizable, so its permitted pair composes into no
        // chain and gets no `run` line — which is exactly the distinction the
        // chain lines exist to make visible.
        //
        // Both programs end with main's own two leaf allocations, which are
        // eligible and do form a chain, and then the branch call that consumes
        // them, which is denied by condition 1. The ledger is in source order,
        // so those lines follow the recursive one and the file is fully
        // reported.
        assert_eq!(
            ledger[1],
            "PAR permitted   bubble.wf:34  pair(boxed_leaf, boxed_leaf)  eligible"
        );
        assert_eq!(
            ledger[2],
            "PAR chain       bubble.wf:34  run(boxed_leaf, boxed_leaf)  2 members through line 35"
        );
        assert_eq!(
            ledger[3],
            "PAR denied      bubble.wf:35  pair(boxed_leaf, boxed_branch)  condition 1: the operands of s2 read what s1 defines"
        );
        assert_eq!(ledger.len(), 4);
    }

    /// One denial line per numbered condition, each citing that condition and
    /// the source text that refused the overlap. A denial arriving under the
    /// wrong condition, or with an empty citation, fails here.
    #[test]
    fn the_permission_ledger_names_the_condition_that_refused_each_pair() {
        // Condition 2: two `&uniq` actuals resolve to one place, so the line
        // has to name both actuals as the writer wrote them.
        let overlapping =
            b"fn bump['r](slot: &uniq 'r u64) -> result: own u64 reads('r), writes('r) {
  let seen = deref(slot);
  set deref(slot) = 7_u64;
  return seen;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  region 'r {
    let lo = bump<'r>(slot: &uniq 'r cell);
    let hi = bump<'r>(slot: &uniq 'r cell);
    let total = imax(lo, hi);
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("bump.wf", overlapping),
            vec![
                "PAR denied      bump.wf:10  pair(bump, bump)  condition 2: the write of s1 overlaps the write of s2 at &uniq 'r cell vs &uniq 'r cell"
                    .to_owned()
            ]
        );

        // Condition 3: the row gate, reported with the exact categories the
        // refused row carries.
        let external =
            b"fn release_read_file(file: own ReadFile) -> result: own unit external, blocks {
  return unit;
}

fn release_pair(first: own ReadFile, second: own ReadFile) -> result: own unit external, blocks {
  let done_first = release_read_file(file: move first);
  let done_second = release_read_file(file: move second);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("row.wf", external),
            vec![
                "PAR denied      row.wf:6  pair(release_read_file, release_read_file)  condition 3: the row of s1 carries external, blocks"
                    .to_owned()
            ]
        );

        // Condition 4: the first statement's `propagate` Err edge leaves the
        // function, so the second statement's write must not run under an
        // overlap the sequential execution skips.
        let propagating = b"fn narrow(v: own u32) -> result: own Result<u8, NarrowError> pure {
  return cvt<u32, u8>(v);
}

fn stamp['o](slot: &uniq 'o u8) -> result: own u64 writes('o) {
  set deref(slot) = 9_u8;
  return 1_u64;
}

fn probe['o](v: own u32, slot: &uniq 'o u8) -> result: own Result<unit, NarrowError> writes('o) {
  let narrowed = propagate narrow(v: v);
  let stamped = stamp<'o>(slot: move slot);
  return Ok<unit, NarrowError>(value: unit);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("propagate.wf", propagating),
            vec![
                "PAR denied      propagate.wf:11  pair(narrow, stamp)  condition 4: the Err edge of s1 skips s2"
                    .to_owned()
            ]
        );
    }

    /// A counted loop that reduces under an exactly-associative integer
    /// operation is permitted, and the line names the operation the
    /// accumulator recombines under.
    ///
    /// This is the shape the pair judgment can never reach: two iterations of
    /// one statement are not a pair, so before the loop rule the compiler
    /// reported the most parallel loop in a program by saying nothing about
    /// it. The callee is a real `pure`, claim-free function with a loop of its
    /// own, so the case is about the writer's loop rather than about a body
    /// small enough to be uninteresting.
    #[test]
    fn a_counted_loop_reducing_under_an_associative_operation_is_permitted() {
        let source = b"fn interesting(index: own u64) -> result: own Bool pure {
  let low = iand(index, 7_u64);
  let seen = 0_u64;
  loop @spin {
    let done = ieq(seen, 4_u64);
    if done {
      break @spin;
    }
    set seen = seen +wrap 1_u64;
  }
  return ieq(low, 3_u64);
}

command fn main() -> status: own ExitStatus pure {
  let hits = 0_u64;
  for @scan i in 0_u64..4096_u64 {
    let escaped = interesting(index: i);
    if escaped {
      set hits = hits +wrap 1_u64;
    }
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("counting.wf", source),
            vec![
                "PAR loop        counting.wf:16  loop  permitted   eligible; \
                 one accumulator under +wrap"
                    .to_owned()
            ]
        );
    }

    /// The float denial, and the reason a loop rule can exist at all.
    ///
    /// `fadd.strict` is not associative, so an implementation free to choose
    /// the combination tree would publish different bytes at a different
    /// worker count — the one failure this whole path exists to make
    /// impossible. The admitted set is enumerated and contains no float, so
    /// the loop is refused outright rather than permitted with a hedge, and
    /// the line cites the statement, which names the operation the writer
    /// wrote.
    #[test]
    fn a_counted_loop_reducing_under_a_float_operation_is_denied_by_condition_one() {
        let source = b"command fn main() -> status: own ExitStatus pure {
  let total = 0.0_f64;
  let step = 0.5_f64;
  for @sum i in 0_u64..1024_u64 {
    set total = fadd.strict(total, step);
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("folding.wf", source),
            vec![
                "PAR loop        folding.wf:4  loop  denied      condition 1: the loop writes \
                 storage outliving the iteration that no exactly associative operation reduces, \
                 at set total = fadd.strict(total, step);"
                    .to_owned()
            ]
        );

        // The identical loop over an integer accumulator is permitted, so the
        // refusal above is about the operation and not about the loop.
        let integral = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  let step = 5_u64;
  for @sum i in 0_u64..1024_u64 {
    set total = total +wrap step;
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("folding.wf", integral),
            vec![
                "PAR loop        folding.wf:4  loop  permitted   eligible; \
                 one accumulator under +wrap"
                    .to_owned()
            ]
        );
    }

    /// A counted loop that writes into a buffer is refused by condition 2.
    ///
    /// Two iterations write two elements of one buffer, and a resolved place
    /// carries no index segment [ENT-2], so the judgment reads them as one
    /// place and fails closed. The parallel map is the deferred capability,
    /// and the line says which write costs the overlap rather than leaving the
    /// loop unreported.
    #[test]
    fn a_counted_loop_writing_into_a_buffer_is_denied_by_condition_two() {
        let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 0_u64);
  for @fill i in 0_u64..64_u64 {
    set out[i] = i *wrap i;
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("mapping.wf", source),
            vec![
                "PAR loop        mapping.wf:3  loop  denied      condition 2: the body writes \
                 storage that is neither introduced by the iteration nor the accumulator, \
                 at set out[i] = i *wrap i;"
                    .to_owned()
            ]
        );
    }

    /// A counted loop whose carried state is written by a callee is refused by
    /// condition 2, however associative the accumulator in view is.
    ///
    /// The loop below folds a float total through a `&uniq` parameter, beside
    /// an ordinary `+wrap` counter — and the counter is what a line reading
    /// only the body's `set` statements would name, permitting a loop whose
    /// real carried state is a `fadd.strict` fold one frame away. The row's
    /// projection onto the actual is the fact that refuses it, so the
    /// enumerated combine set governs all of the loop's carried state and not
    /// only the part written in view.
    #[test]
    fn a_counted_loop_whose_callee_writes_carried_state_is_denied_by_condition_two() {
        let source =
            b"fn accum['s](slot: &uniq 's f64, x: own f64) -> result: own u64 reads('s), writes('s) {
  set deref(slot) = fadd.strict(deref(slot), x);
  let bits = reinterpret<f64, u64>(deref(slot));
  return iand(bits, 1_u64);
}

command fn main() -> status: own ExitStatus pure {
  let total = 0.0_f64;
  let count = 0_u64;
  for @sum i in 0_u64..8_u64 {
    region 'acc {
      let one = accum<'acc>(slot: &uniq 'acc total, x: 0.5_f64);
      set count = count +wrap one;
    }
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("carrying.wf", source),
            vec![
                "PAR loop        carrying.wf:10  loop  denied      condition 2: the body writes \
                 storage that is neither introduced by the iteration nor the accumulator, \
                 at &uniq 'acc total"
                    .to_owned()
            ]
        );

        // The same loop over a callee that writes nothing is permitted, so the
        // refusal above is about the projected row and not about the shape.
        let reading = b"fn weigh(x: own f64) -> result: own u64 pure {
  let bits = reinterpret<f64, u64>(x);
  return iand(bits, 1_u64);
}

command fn main() -> status: own ExitStatus pure {
  let total = 0.0_f64;
  let count = 0_u64;
  for @sum i in 0_u64..8_u64 {
    let one = weigh(x: total);
    set count = count +wrap one;
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("carrying.wf", reading),
            vec![
                "PAR loop        carrying.wf:9  loop  permitted   eligible; \
                 one accumulator under +wrap"
                    .to_owned()
            ]
        );
    }

    /// A counted loop a `give` can leave is refused by condition 4, however
    /// associative its accumulator is.
    ///
    /// `give` is the fourth exit form, and the one that leaves the enclosing
    /// value initializer as well as the loop. A combination tree over the
    /// whole range has no representation for that edge at all: it folds every
    /// iteration where the loop stopped at the first hit. The fixture below
    /// sums 64 ones, meets a 7 at index 10 and gives there, so the loop
    /// contributes 10 where a full-range fold contributes 70.
    #[test]
    fn a_counted_loop_a_give_can_leave_is_denied_by_condition_four() {
        let source = b"fn scan_until['s](src: &'s buffer<u64>, needle: own u64) -> result: own u64 reads('s) {
  let count = len(deref(src));
  let acc = 0_u64;
  let always = True();
  let answer = if always {
    for @scan i in 0_u64..count {
      let v = deref(src)[i];
      set acc = acc +wrap v;
      let hit = ieq(v, needle);
      if hit {
        give i;
      }
    }
    give 4096_u64;
  } else {
    give 4096_u64;
  }
  return answer +wrap acc;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(64_u64, 1_u64);
  set data[10_u64] = 7_u64;
  region 's {
    let t = scan_until<'s>(src: &'s data, needle: 7_u64);
    return exit_status(code: 0_u8);
  }
}
";
        assert_eq!(
            ledger_of("giving.wf", source),
            vec![
                "PAR loop        giving.wf:6  loop  denied      condition 4: a give leaves the loop"
                    .to_owned()
            ]
        );

        // The same loop with the give removed is permitted, so the refusal is
        // about the exit edge and not about the shape.
        let contained = b"fn scan_until['s](src: &'s buffer<u64>, needle: own u64) -> result: own u64 reads('s) {
  let count = len(deref(src));
  let acc = 0_u64;
  let always = True();
  let answer = if always {
    for @scan i in 0_u64..count {
      let v = deref(src)[i];
      set acc = acc +wrap v;
    }
    give 4096_u64;
  } else {
    give 4096_u64;
  }
  return answer +wrap acc;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(64_u64, 1_u64);
  set data[10_u64] = 7_u64;
  region 's {
    let t = scan_until<'s>(src: &'s data, needle: 7_u64);
    return exit_status(code: 0_u8);
  }
}
";
        assert_eq!(
            ledger_of("giving.wf", contained),
            vec![
                "PAR loop        giving.wf:6  loop  permitted   eligible; \
                 one accumulator under +wrap"
                    .to_owned()
            ]
        );
    }

    /// The split advice outlives exactly one refusal, and names each combine
    /// the way a writer spells it.
    ///
    /// Three accumulators is the one shape this version declines while a
    /// hand-written recursion returning an aggregate still reaches it, so the
    /// loop line reports the refusal and a second line reports the rewrite.
    /// The advice is meant to be typed, so an operation named in a spelling
    /// the language does not have is advice that does not compile: the `Bool`
    /// row is `band`, `bor`, `bxor` [OP-1].
    #[test]
    fn a_refused_multi_accumulator_loop_keeps_advice_naming_the_boolean_combines() {
        let source = b"command fn main() -> status: own ExitStatus pure {
  let every = True();
  let any = False();
  let parity = False();
  for @scan i in 0_u64..64_u64 {
    let low = iand(i, 1_u64);
    let bit = ieq(low, 0_u64);
    set every = band(every, bit);
    set any = bor(any, bit);
    set parity = bxor(parity, bit);
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("booleans.wf", source),
            vec![
                "PAR loop        booleans.wf:5  loop  denied      condition 1: the body carries \
                 3 accumulators, and this rule recombines one"
                    .to_owned(),
                "PAR hint        booleans.wf:5  loop  refused by condition 1; a recursive split \
                 over its index range would be eligible, combining under band, bor, bxor"
                    .to_owned(),
            ]
        );
    }

    /// A program with no analyzed pair reports nothing, and the ledger never
    /// reaches the module: the same compilation with and without it emits the
    /// same bytes.
    #[test]
    fn the_permission_ledger_is_output_beside_an_unchanged_module() {
        let source = b"command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
        let (module, ledger) = compile_with_permission_ledger(
            &[SourceInput::new("quiet.wf", source)],
            CompilerLimits::default(),
            OverlapLowering::Off,
        )
        .expect("the fixture must compile");
        assert!(ledger.is_empty(), "no analyzed pair: {ledger:?}");
        let plain = compile(
            &[SourceInput::new("quiet.wf", source)],
            CompilerLimits::default(),
        )
        .expect("the fixture must compile");
        assert_eq!(module, plain);
    }

    /// Actualization is compile-time opt-in, and the judgment is not: the
    /// ledger of a program full of eligible pairs is the same with the option
    /// on and off, while only the `--par` module names the runtime.
    ///
    /// This is what makes the ledger usable on a shipped build. A developer
    /// reading what the compiler decided about a program is reading a property
    /// of the source, not of the compilation they happened to ask for.
    #[test]
    fn the_permission_ledger_does_not_depend_on_whether_the_lowering_is_taken() {
        let source = format!(
            "{TREE_PRELUDE}fn fold['b](node: &uniq 'b box<BoxNode>) -> result: own u64 reads('b), writes('b) {{
  match deref(deref(node)) {{
    Leaf(w: leaf_w) => {{
      return deref(leaf_w);
    }}
    Branch(left: l, right: r, w: slot) => {{
      let a = fold<'b>(node: move l);
      let b = fold<'b>(node: move r);
      let total = imax(a, b);
      set deref(slot) = total;
      return total;
    }}
  }}
}}

command fn main() -> status: own ExitStatus allocates(heap) {{
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  region 'tree {{
    let total = fold<'tree>(node: &uniq 'tree branch0);
  }}
  return exit_status(code: 0_u8);
}}
"
        );
        let inputs = [SourceInput::new("fold.wf", source.as_bytes())];
        let (default, quiet_ledger) = compile_with_permission_ledger(
            &inputs,
            CompilerLimits::default(),
            OverlapLowering::Off,
        )
        .expect("the fixture must compile");
        let (requested, loud_ledger) =
            compile_with_permission_ledger(&inputs, CompilerLimits::default(), OverlapLowering::On)
                .expect("the fixture must compile");

        assert_eq!(quiet_ledger, loud_ledger);
        assert!(
            quiet_ledger.iter().any(|line| line.contains("eligible")),
            "the fixture must report an eligible pair: {quiet_ledger:?}"
        );
        assert!(
            !default.contains("wf__par_"),
            "the default module must name no runtime symbol"
        );
        assert!(
            requested.contains("wf__par_claim"),
            "the requested module must offer a lane"
        );
    }

    #[test]
    fn driver_lowers_static_contract_metadata_without_executable_artifacts() {
        let source = b"contract Empty {\n}\n\nconform i32: Empty {\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
        let llvm = compile(
            &[SourceInput::new("value.wf", source)],
            CompilerLimits::default(),
        )
        .expect("static contract metadata must use the ordinary lowering path");
        assert!(llvm.contains("define i32 @main(i32 %argc, ptr %argv)"));
        assert!(!llvm.contains("Empty"));
    }

    #[test]
    fn every_pre_semantic_rejection_publishes_the_rule_its_stage_attributed() {
        // One source per pre-semantic stage that can reject source, each
        // reaching its stage's own DIAG-1 attribution. A stop that publishes
        // no rule cannot be told from a stop that cites none, so the whole
        // frontend is checked here rather than only the semantic stage.
        for (name, source, stage, rule) in [
            (
                "comment.wf",
                b"// nope\nfn probe() -> result: own unit pure {\n  return unit;\n}\n".as_slice(),
                CompilationStage::Lexing,
                "FORM-4",
            ),
            (
                "tab.wf",
                b"fn probe() -> result: own unit pure {\n\treturn unit;\n}\n",
                CompilationStage::Lexing,
                "FORM-2",
            ),
            (
                "sigil.wf",
                b"fn probe() -> result: own unit pure {\n  let value: own i32 = 'Bad;\n  return unit;\n}\n",
                CompilationStage::Lexing,
                "FORM-3",
            ),
            (
                "dollar.wf",
                b"$\nfn probe() -> result: own unit pure {\n  return unit;\n}\n",
                CompilationStage::Lexing,
                "FORM-1",
            ),
            (
                "string.wf",
                b"fn probe() -> result: own unit pure {\n  let text: own str = \"bad\\t\";\n  return unit;\n}\n",
                CompilationStage::Lexing,
                "FORM-5",
            ),
            (
                "numeric.wf",
                b"fn probe() -> result: own unit pure {\n  let value: own i32 = 1e+;\n  return unit;\n}\n",
                CompilationStage::TerminalClassification,
                "FORM-5",
            ),
            (
                "construct.wf",
                b"nope value;\n\nfn probe() -> result: own unit pure {\n  return unit;\n}\n",
                CompilationStage::Parsing,
                "FORM-1",
            ),
            (
                "spacing.wf",
                b"fn  main() -> result: own unit pure {\n  return unit;\n}\n",
                CompilationStage::CanonicalSource,
                "FORM-2",
            ),
            (
                // The v0.22 case put the undeclared region in a `let`
                // annotation, which A3 deletes along with the violation. A
                // borrow keeps writing its region, so the same undeclared
                // spelling reaches the same OWN-3 at the same stage.
                "region.wf",
                b"fn probe() -> result: own unit pure {\n  let value = 0_i32;\n  let borrowed = &'gone value;\n  return unit;\n}\n",
                CompilationStage::Resolution,
                "OWN-3",
            ),
        ] {
            let failure = compile(&[SourceInput::new(name, source)], CompilerLimits::default())
                .expect_err("the case must be rejected");
            assert_eq!(failure.stage(), stage, "{name}: {failure}");
            assert_eq!(
                failure.kind(),
                CompilationFailureKind::Source,
                "{name}: {failure}"
            );
            assert_eq!(failure.rule_id(), Some(rule), "{name}: {failure}");
            assert!(
                failure.to_string().contains(rule),
                "{name}: published diagnostic omitted {rule}: {failure}"
            );
        }
    }

    #[test]
    fn unrepresentable_array_is_a_target_failure_without_a_source_rule() {
        let source = b"command fn main() -> status: own ExitStatus pure {\n  let values = array_new<u8, 18446744073709551615>(0_u8);\n  return exit_status(code: 0_u8);\n}\n";
        let failure = compile(
            &[SourceInput::new("value.wf", source)],
            CompilerLimits::default(),
        )
        .expect_err("the selected target cannot represent the array object");
        assert_eq!(failure.stage(), CompilationStage::TargetLayout);
        assert_eq!(failure.kind(), CompilationFailureKind::TargetLayout);
        assert_eq!(failure.rule_id(), None);
        assert!(failure.detail().contains("Unrepresentable"));
    }

    #[test]
    fn complete_frame_is_checked_after_each_slot_layout_succeeds() {
        let source = b"command fn main() -> status: own ExitStatus pure {\n  let left = array_new<u8, 4611686018427387904>(0_u8);\n  let right = array_new<u8, 4611686018427387904>(0_u8);\n  return exit_status(code: 0_u8);\n}\n";
        let failure = compile(
            &[SourceInput::new("value.wf", source)],
            CompilerLimits::default(),
        )
        .expect_err("two individually representable slots cannot form one target frame");
        assert_eq!(failure.stage(), CompilationStage::TargetLayout);
        assert_eq!(failure.kind(), CompilationFailureKind::TargetLayout);
        assert_eq!(failure.rule_id(), None);
        assert!(failure.detail().contains("StackFrame"));
    }

    #[test]
    fn system_interface_constructs_compile_through_the_normal_path() {
        // The canonical FN-7 command-entry header with a conforming body
        // and exact row: the entry admits, its system calls type against
        // the [SYS-2] catalog, and [EFF-2] attribution accepts the row —
        // `external, blocks` from the DirectoryRead input's compiler-derived
        // close attempt on the return edge. [QUAL-1] qualification now maps
        // each identity to an approved implementation and the [QUAL-3]
        // bootstrap supplies the standard inputs, so the program emits.
        let kind_entry = b"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus external, blocks {\n  return exit_status(code: 0_u8);\n}\n";
        let llvm = compile(
            &[SourceInput::new("entry.wf", kind_entry)],
            CompilerLimits::default(),
        )
        .expect("a qualified command program must emit");
        assert!(llvm.contains("define i32 @main(i32 %argc, ptr %argv)"));

        // A valid command unit whose entry declares no standard input emits
        // the same bootstrap shape: qualification is over the IR's own system
        // facts, not over the entry's parameter list.
        let no_inputs =
            b"command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
        let llvm = compile(
            &[SourceInput::new("entry.wf", no_inputs)],
            CompilerLimits::default(),
        )
        .expect("a command entry selecting no input must emit");
        assert!(llvm.contains("define i32 @main(i32 %argc, ptr %argv)"));

        // `open_read`, `read_once`, and `write_once` complete the qualified
        // interface: every [SYS-2] semantic identity now has an approved
        // implementation on this target, so no unsupported stop remains
        // between an accepted system program and its emitted module.
        let writing =b"command fn main(command.stdout as out: own Output) -> status: own ExitStatus allocates(heap), external, blocks {\n  let bytes = buffer_new(1_u64, 65_u8);\n  region 'o {\n    region 's {\n      match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 0_u64, end: 1_u64) {\n        Ok(value: written) => {\n          return exit_status(code: 0_u8);\n        }\n        Err(error: problem) => {\n          return exit_status(code: 1_u8);\n        }\n      }\n    }\n  }\n}\n";
        let llvm = compile(
            &[SourceInput::new("entry.wf", writing)],
            CompilerLimits::default(),
        )
        .expect("a qualified writing command must emit");
        assert!(llvm.contains("; QUAL-1 semantic id 9 -> @wf.sys.write_once.v1"));

        // A `command` entry whose written result is not `own ExitStatus` is a
        // source rejection now, not an unsupported stop: the FN-7 entry-form
        // judgment is implemented and runs before the remaining capability
        // stops.
        let wrong_result = b"command fn main() -> result: own unit pure {\n  return unit;\n}\n";
        let failure = compile(
            &[SourceInput::new("entry.wf", wrong_result)],
            CompilerLimits::default(),
        )
        .expect_err("a command entry returning own unit must be rejected");
        assert_eq!(failure.stage(), CompilationStage::Semantics);
        assert_eq!(failure.kind(), CompilationFailureKind::Source);
        assert_eq!(failure.rule_id(), Some("FN-7"));

        // The `external` and `blocks` categories are checked, not stopped.
        // These two internal bodies exhibit neither, so declaring either is
        // an ordinary EFF-2 declared-but-unexhibited rejection.
        for source in [
            b"fn probe() -> result: own unit external {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n".as_slice(),
            b"fn probe() -> result: own unit blocks {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        ] {
            let failure = compile(
                &[SourceInput::new("rejected.wf", source)],
                CompilerLimits::default(),
            )
            .expect_err("an undeclarable category must reject citing EFF-2");
            assert_eq!(failure.stage(), CompilationStage::Semantics);
            assert_eq!(failure.kind(), CompilationFailureKind::Source);
            assert_eq!(failure.rule_id(), Some("EFF-2"));
        }
    }

    #[test]
    fn compiler_independent_negative_cases_keep_their_semantic_rule() {
        for (name, source, rule) in [
            (
                "gram11-neg-misspelled.wf",
                include_bytes!("../../tests/conformance/cases/gram11-neg-misspelled.wf").as_slice(),
                "GRAM-11",
            ),
            (
                "eff2-neg-declared-unexhibited.wf",
                include_bytes!("../../tests/conformance/cases/eff2-neg-declared-unexhibited.wf")
                    .as_slice(),
                "EFF-2",
            ),
            // `fn2-neg-implicit-instantiation.wf` sat here until 2026-08-08,
            // when the case was retired: A1 respelled its violation out of
            // existence, so it compiled at exit 0 and this row could never
            // hold again. Its FN-2 content lives at
            // `fn2-neg-eeq-implicit-type`, repurposed onto a user-generic
            // call. The entry goes with the case it names rather than being
            // an assertion dropped on its own.
            (
                "form7-neg-out-of-range.wf",
                include_bytes!("../../tests/conformance/cases/form7-neg-out-of-range.wf")
                    .as_slice(),
                "FORM-7",
            ),
            (
                "type5-neg-arg-mismatch.wf",
                include_bytes!("../../tests/conformance/cases/type5-neg-arg-mismatch.wf")
                    .as_slice(),
                "TYPE-5",
            ),
            (
                "x-struct-neg-field-order.wf",
                include_bytes!("../../tests/conformance/cases/x-struct-neg-field-order.wf")
                    .as_slice(),
                "GRAM-8",
            ),
            (
                "x-match-gram10-out-of-order-fields.wf",
                include_bytes!(
                    "../../tests/conformance/cases/x-match-gram10-out-of-order-fields.wf"
                )
                .as_slice(),
                "GRAM-10",
            ),
            (
                "err2-neg-missing-variant.wf",
                include_bytes!("../../tests/conformance/cases/err2-neg-missing-variant.wf")
                    .as_slice(),
                "ERR-2",
            ),
            (
                "x-ownmove-partial-move-kills-binding.wf",
                include_bytes!(
                    "../../tests/conformance/cases/x-ownmove-partial-move-kills-binding.wf"
                )
                .as_slice(),
                "OWN-1",
            ),
            (
                "x-ownmove-payload-binder-consumed-twice.wf",
                include_bytes!(
                    "../../tests/conformance/cases/x-ownmove-payload-binder-consumed-twice.wf"
                )
                .as_slice(),
                "OWN-1",
            ),
            (
                "x-gram-construct-repeated-field.wf",
                include_bytes!("../../tests/conformance/cases/x-gram-construct-repeated-field.wf")
                    .as_slice(),
                "GRAM-8",
            ),
            (
                "x-gram-construct-missing-field.wf",
                include_bytes!("../../tests/conformance/cases/x-gram-construct-missing-field.wf")
                    .as_slice(),
                "GRAM-8",
            ),
            (
                "x-typ-match-foreign-variant.wf",
                include_bytes!("../../tests/conformance/cases/x-typ-match-foreign-variant.wf")
                    .as_slice(),
                "TYPE-6",
            ),
            (
                "x-match-give1-wrong-type.wf",
                include_bytes!("../../tests/conformance/cases/x-match-give1-wrong-type.wf")
                    .as_slice(),
                // Moved TYPE-5 -> GIVE-1 by the 2026-08-08 M3b dispositions
                // ruling (d), source unchanged. The manifest row was updated
                // then and this second witness was not, which is exactly the
                // desync it exists to catch — so it is updated by hand against
                // the ruling, never derived from the manifest.
                "GIVE-1",
            ),
            (
                "x-integ-give-in-statement-match-rejected.wf",
                include_bytes!(
                    "../../tests/conformance/cases/x-integ-give-in-statement-match-rejected.wf"
                )
                .as_slice(),
                "GIVE-1",
            ),
            (
                "reject-sysentry-label-unknown.wf",
                include_bytes!("../../tests/conformance/cases/reject-sysentry-label-unknown.wf")
                    .as_slice(),
                "FN-7",
            ),
            (
                "reject-sysentry-label-repeated.wf",
                include_bytes!("../../tests/conformance/cases/reject-sysentry-label-repeated.wf")
                    .as_slice(),
                "FN-7",
            ),
            (
                "reject-sysentry-label-out-of-order.wf",
                include_bytes!(
                    "../../tests/conformance/cases/reject-sysentry-label-out-of-order.wf"
                )
                .as_slice(),
                "FN-7",
            ),
            (
                "reject-sysentry-label-outside-entry.wf",
                include_bytes!(
                    "../../tests/conformance/cases/reject-sysentry-label-outside-entry.wf"
                )
                .as_slice(),
                "FN-7",
            ),
            (
                "reject-sysentry-input-type-mismatch.wf",
                include_bytes!(
                    "../../tests/conformance/cases/reject-sysentry-input-type-mismatch.wf"
                )
                .as_slice(),
                "FN-7",
            ),
            (
                "reject-sysentry-call-to-kind-entry.wf",
                include_bytes!(
                    "../../tests/conformance/cases/reject-sysentry-call-to-kind-entry.wf"
                )
                .as_slice(),
                "FN-7",
            ),
        ] {
            let failure = compile(&[SourceInput::new(name, source)], CompilerLimits::default())
                .expect_err("negative conformance case must reject");
            assert_eq!(
                failure.stage(),
                CompilationStage::Semantics,
                "{name}: {failure}"
            );
            assert_eq!(
                failure.kind(),
                CompilationFailureKind::Source,
                "{name}: {failure}"
            );
            assert_eq!(failure.rule_id(), Some(rule), "{name}: {failure}");
            assert!(
                failure.to_string().contains(rule),
                "{name}: published diagnostic omitted {rule}: {failure}"
            );
        }
    }

    #[test]
    fn retired_program_kind_spellings_reject_in_parsing_with_form1() {
        for (name, source) in [
            (
                "reject-form1-service-leading-construct.wf",
                include_bytes!(
                    "../../tests/conformance/cases/reject-form1-service-leading-construct.wf"
                )
                .as_slice(),
            ),
            (
                "reject-form1-embedded-leading-construct.wf",
                include_bytes!(
                    "../../tests/conformance/cases/reject-form1-embedded-leading-construct.wf"
                )
                .as_slice(),
            ),
            (
                "reject-form1-daemon-leading-construct.wf",
                include_bytes!(
                    "../../tests/conformance/cases/reject-form1-daemon-leading-construct.wf"
                )
                .as_slice(),
            ),
        ] {
            let failure = compile(&[SourceInput::new(name, source)], CompilerLimits::default())
                .expect_err("a retired program-kind spelling must reject");
            assert_eq!(
                failure.stage(),
                CompilationStage::Parsing,
                "{name}: {failure}"
            );
            assert_eq!(
                failure.kind(),
                CompilationFailureKind::Source,
                "{name}: {failure}"
            );
            assert_eq!(failure.rule_id(), Some("FORM-1"), "{name}: {failure}");
            assert!(
                failure.to_string().contains("FORM-1"),
                "{name}: published diagnostic omitted FORM-1: {failure}"
            );
        }
    }
}
