//! One ordinary active-specification compilation pipeline.
//!
//! The driver keeps source failures, unsupported compiler capabilities,
//! resource failures, invariant failures, lowering failures, and backend
//! failures distinct while returning owned LLVM assembly to callers.

use core::fmt;

mod rejection;

/// The probe corpus that pins every diagnostic sentence by its rendered text.
#[cfg(test)]
mod pinned_sentences;

use rejection::Located;

use crate::{
    ACTIVE_KERNEL_SPEC_HASH, BackendFailure, CanonicalLimits, CanonicalOutcome, FinalizeLimits,
    FinalizeOutcome, LexLimits, LexOutcome, LoweringFailure, ParseLimits, ParseOutcome,
    ResolutionOutcome, SemanticLocation, SemanticOutcome, SourceBundle, SourceInput, SourceLimits,
    TerminalLimits, TerminalOutcome, audit_canonical, check_semantics, classify_terminals,
    emit_llvm, finalize, lex, lower_checked, parse, resolve_with_inventory,
};

/// Host-compiler optimization arguments for every Whitefoot executable.
///
/// One definition serves the driver executable and every test that links an
/// emitted module, so no path can silently link an unoptimized binary while
/// another links an optimized one. There is no writer-facing switch: the
/// optimization level cannot change which programs are accepted, discharge a
/// static source obligation, or insert a runtime proof fallback,
/// so no writer decision exists and the default shape is the only shape. The
/// level is provisional and may move once a measurement asks for it.
pub const HOST_OPTIMIZATION_ARGUMENTS: &[&str] = &["-O2"];

/// The host libraries a link of an emitted module names, for the same
/// one-definition reason.
///
/// A Whitefoot module reaches libm without asking for it: the backend lowers a
/// rounding to `roundevenf` and a fused multiply-add to `fma`, and the host
/// optimizer is free to turn ordinary float arithmetic into another of that
/// library's entry points. Darwin serves those from the same library as
/// `write` and needs nothing said; an ELF host keeps them in `libm` and the
/// link fails with an undefined symbol. Naming the library on both hosts is
/// one link path instead of a per-target one, and on Darwin it resolves to a
/// stub that is already linked.
///
/// Every link that builds an executable from an emitted module belongs on this
/// constant rather than on its own library list — the shipped driver's, the
/// backend's linked-executable helper, the program-corpus harness, and the
/// conformance adapter all take it from here. A link that builds only
/// compiler-owned C units, such as the exhaustion floor fixture, reaches no
/// entry point the emitter chose and does not need it. A link that should have
/// named it and did not fails loudly with an undefined symbol on an ELF host;
/// it never changes a verdict or an outcome, which is why the shipped path is
/// the one this constant has to reach.
///
/// Found by running the program corpus on an x86-64 Linux runner in batch
/// 0090: `grayscale_pixels` and `feedback_controller` compile there and did
/// not link, in the shipped driver's own link path.
pub const HOST_LINK_LIBRARIES: &[&str] = &["-lm"];

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
/// This is the shipped completion-only default: eligible direct finite target
/// operations may overlap, while compute-call outlining remains opt-in through
/// [`compile_with_overlap`] and `whitefootc --par`.
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
    compile_reporting(inputs, limits, crate::Inventory::ACTIVE, overlap)
        .map(|reported| reported.module)
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
        .map(|reported| (reported.module, reported.ledger))
}

/// [`compile_with_overlap`] plus the permission-ledger lines an ordinary
/// compile reports on the default developer channel.
///
/// Exactly the denied verdicts of the program's I/O loops: a `[PAR-3]` staged
/// denial, and the `[PAR-2]` counted denial of a loop the staged judgment also
/// reached. Those are the two ways a loop a writer wrote to do I/O loses its
/// pipeline, and they are missed optimizations on the source in front of the
/// writer rather than a reading of the judgment, so they do not wait for a
/// flag. A granted verdict is silent; `compile_with_permission_ledger` remains
/// the full report.
///
/// Nothing here is a rejection. The compilation succeeded and the module is
/// returned beside the notices.
pub fn compile_with_io_notices(
    inputs: &[SourceInput<'_>],
    limits: CompilerLimits,
    overlap: crate::OverlapLowering,
) -> Result<(String, Vec<String>), CompilationFailure> {
    compile_reporting(inputs, limits, crate::Inventory::ACTIVE, overlap)
        .map(|reported| (reported.module, reported.notices))
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
    compile_reporting(
        inputs,
        limits,
        inventory,
        crate::OverlapLowering::Completion,
    )
    .map(|reported| reported.module)
}

/// One compilation's module and the developer-channel text it produced.
///
/// Two channels, one rendering. `ledger` is the complete report a caller asks
/// for by flag; `notices` is the subset every compile reports without one.
/// They are projections of the same rendered lines, so a notice can never say
/// something the full report does not.
struct Reported {
    module: String,
    ledger: Vec<String>,
    notices: Vec<String>,
}

/// The one compilation path, returning the module and the developer-channel
/// permission ledger it produced. Every public entry point above is a
/// projection of this function; there is no second pipeline.
fn compile_reporting(
    inputs: &[SourceInput<'_>],
    limits: CompilerLimits,
    inventory: crate::Inventory,
    overlap: crate::OverlapLowering,
) -> Result<Reported, CompilationFailure> {
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
            let coordinate = issue.coordinate();
            return Err(CompilationFailure::source(
                CompilationStage::Parsing,
                issue.rule().id(),
                Located::new(issue, classified.source_bundle(), coordinate),
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
            // FORM-2's coordinate is the trivia gap between two terminals, not
            // a written construct, so the reader is anchored inside the gap
            // rather than at its first byte.
            let coordinate = issue.location().coordinate();
            return Err(CompilationFailure::source(
                CompilationStage::CanonicalSource,
                issue.rule().id(),
                Located::in_gap(issue, classified.source_bundle(), coordinate),
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
            let coordinate = issue.origin().coordinate();
            return Err(CompilationFailure::source(
                CompilationStage::Resolution,
                issue.rule().id(),
                Located::new(issue, classified.source_bundle(), coordinate),
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
            // A semantic rejection carries the richest payload in the
            // toolchain and, until now, the poorest location: `SourceId(0)`
            // and a byte offset. The coordinate the rule already selected
            // names a line of the file the caller named, so it is printed the
            // same way a syntax rejection's is.
            let rule_id = issue.rule_id();
            let coordinate = match issue.location() {
                SemanticLocation::SourceNode(_, coordinate) => Some(*coordinate),
                SemanticLocation::BundleRoot(_) => None,
            };
            return Err(match coordinate {
                Some(coordinate) => CompilationFailure::source(
                    CompilationStage::Semantics,
                    rule_id,
                    Located::new(issue, classified.source_bundle(), coordinate),
                ),
                None => CompilationFailure::source(CompilationStage::Semantics, rule_id, issue),
            });
        }
        SemanticOutcome::ResolutionIssue { issue, .. } => {
            let coordinate = issue.origin().coordinate();
            return Err(CompilationFailure::source(
                CompilationStage::Resolution,
                issue.rule().id(),
                Located::new(issue, classified.source_bundle(), coordinate),
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
    let permission_ledger = checked.data.permission_ledger.clone();
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
    let notices = permission_ledger
        .iter()
        .filter(|line| line.notice)
        .map(|line| line.text.clone())
        .collect();
    let mut ledger: Vec<String> = permission_ledger
        .into_iter()
        .map(|line| line.text)
        .collect();
    ledger.extend_from_slice(ir.actualization_ledger());
    emit_llvm(&ir)
        .map(|module| Reported {
            module: module.into_string(),
            ledger,
            notices,
        })
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
        CompilationFailureKind, CompilationStage, CompilerLimits, compile, compile_with_io_notices,
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

    /// The ledger lines the same compilation reports without any flag.
    fn notices_of(name: &str, source: &[u8]) -> Vec<String> {
        let (_, notices) = compile_with_io_notices(
            &[SourceInput::new(name, source)],
            CompilerLimits::default(),
            OverlapLowering::Off,
        )
        .expect("a permission-ledger fixture must compile");
        notices
    }

    /// The scratch buffer hoisted above the loop, which denies the staged
    /// verdict at `&uniq data`.
    const DENIED_IO_LOOP: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let name = buffer_new(16_u64, 97_u8);
  let data = buffer_new(64_u64, 0_u8);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let permit = reserve_file(factory: &uniq files);
    region {
      match open_file(permit: move permit, root: &cwd, name: &name, start: 0_u64, end: 4_u64) {
        Ok(value: handle) => {
          region 'h {
            region {
              match read_at(file: &'h handle, destination: &uniq data, file_offset: 0_u64, start: 0_u64, end: 64_u64) {
                ReadBytes(next: produced) => {
                  set total = total +wrap produced;
                }
                ReadEnd() => {
                }
                ReadFailed(error: problem) => {
                }
              }
            }
          }
        }
        Err(error: problem) => {
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

    /// The same loop with its scratch inside the body, which the staged
    /// judgment grants.
    const GRANTED_IO_LOOP: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set total = total +wrap 1_u64;
          }
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

    /// A denied I/O loop is reported on an ordinary compile; a granted one is
    /// silent.
    ///
    /// The judgment was landed, correct, and unreachable: a writer compiled
    /// five ordinary utilities, every I/O loop in them was denied, and nothing
    /// said so, because the report was behind a flag they had no reason to
    /// run. A loop that lost its pipeline is a missed optimization on the
    /// program in front of them, so it does not wait to be asked for.
    ///
    /// The granted case is the other half and the harder one. Its counted
    /// [PAR-2] verdict *is* denied — the counted rule refuses the short factory
    /// loan the staged rule exists to admit — so a notice channel that reported
    /// every denial would tell this writer their granted loop was denied.
    #[test]
    fn a_denied_io_loop_is_reported_without_a_flag_and_a_granted_one_is_silent() {
        let notices = notices_of("hoisted.wf", DENIED_IO_LOOP);
        let ledger = ledger_of("hoisted.wf", DENIED_IO_LOOP);
        // Both verdicts of the one loop, and every place that denied it. The
        // counted rule and the staged rule refuse it for different reasons, so
        // both are losses the writer can act on; and the `stage` line names
        // only the condition the judgment stopped at, so the denied rows of
        // its table come with it. A repaired first cause otherwise uncovers a
        // second denial the writer was never told about, which is what the
        // verification of 2026-08-28 met on its first loop.
        assert_eq!(notices.len(), 3, "{notices:?}");
        assert!(
            notices[0].starts_with("PAR loop        hoisted.wf:5") && notices[0].contains("denied"),
            "{notices:?}"
        );
        assert!(
            notices[1].starts_with("PAR stage       hoisted.wf:5")
                && notices[1].contains("condition 3")
                && notices[1].ends_with("&uniq data"),
            "{notices:?}"
        );
        assert!(
            notices[2].starts_with("PAR place       hoisted.wf:5")
                && notices[2].contains("denied")
                && notices[2].contains("&uniq data"),
            "{notices:?}"
        );
        // A row that is not denied stays inside the full report: the notice
        // channel states what cost the loop its pipeline, not the whole table.
        assert!(
            notices.iter().all(|notice| !notice.contains("read-only")),
            "{notices:?}"
        );
        // Every notice is a line of the full report, verbatim: one rendering,
        // two channels.
        assert!(
            notices.iter().all(|notice| ledger.contains(notice)),
            "notices are a subset of the report: {ledger:?}"
        );

        assert!(
            notices_of("staged.wf", GRANTED_IO_LOOP).is_empty(),
            "a granted staged verdict says nothing without a flag"
        );
        // And the report still carries that loop's counted denial, which the
        // notice channel deliberately withholds.
        assert!(
            ledger_of("staged.wf", GRANTED_IO_LOOP)
                .iter()
                .any(|line| line.starts_with("PAR loop") && line.contains("denied")),
            "the counted denial stays in the full report"
        );
    }

    /// A syntax rejection prints the spellings it expected and the line it
    /// stopped in.
    ///
    /// Flat three-address form is the largest departure from every other
    /// systems language, so this is the rule an unguided writer hits first.
    /// They hit it as `TerminalSet(38424498140022966840644862354)` and a byte
    /// offset, and ran `head -c` on their own program to find out what it
    /// meant. The compiler holds the expected set and the source bytes; both
    /// are printed here.
    #[test]
    fn a_syntax_rejection_prints_the_expected_spellings_and_the_offending_line() {
        let source = br#"command fn main() -> status: own ExitStatus pure {
  doc "Writes a nested call where the grammar admits an atom.";
  let dotted = 1_u8;
  let addressable = 2_u8;
  let skip = bor(dotted, bnot(addressable));
  return exit_status(code: skip);
}
"#;
        let failure = compile(
            &[SourceInput::from_host_path(
                "input0.wf",
                "/absolute/path/wc.wf",
                source,
            )],
            CompilerLimits::default(),
        )
        .expect_err("a nested call is not an atom");
        assert_eq!(failure.rule_id(), Some("GRAM-9"));
        let detail = failure.detail();
        // The set as spellings, in the grammar's own order.
        assert!(
            detail.contains(r#"expected: ["{", ";", ")", ",", "<", ">", "["#),
            "{detail}"
        );
        // The line the writer wrote, and where in it the parser stopped.
        assert!(
            detail.contains(r#"at /absolute/path/wc.wf:5:26 in line "  let skip = bor(dotted, bnot(addressable));""#),
            "{detail}"
        );
    }

    #[test]
    fn invariant_targets_and_certificate_steps_keep_distinct_rule_owners() {
        for (name, source, stage, rule) in [
            (
                "local-target-formation.wf",
                b"command fn main() -> status: own ExitStatus pure {\n  invariant bad: 0_u64 == 0_u64;\n  return exit_status(code: 0_u8);\n}\n"
                    .as_slice(),
                CompilationStage::Semantics,
                "INV-1",
            ),
            (
                "local-target-unproved.wf",
                b"command fn main() -> status: own ExitStatus pure {\n  invariant bad: 1_u64 <= 0_u64;\n  return exit_status(code: 0_u8);\n}\n",
                CompilationStage::Semantics,
                "INV-1",
            ),
            (
                "use-relation-formation.wf",
                b"fn check(value: own u64, limit: own u64) -> result: own unit pure {\n  invariant scaled: 2_u64 * value <= 2_u64 * limit {\n    use value == limit;\n  }\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
                CompilationStage::Semantics,
                "PRF-1",
            ),
            (
                "use-relation-name.wf",
                b"fn check(value: own u64, limit: own u64) -> result: own unit pure {\n  invariant scaled: 2_u64 * value <= 2_u64 * limit {\n    use value <= missing;\n  }\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
                CompilationStage::Resolution,
                "PRF-1",
            ),
            (
                "named-use-scope.wf",
                b"fn check(value: own u64, limit: own u64) -> result: own unit pure {\n  invariant scaled: 2_u64 * value <= 2_u64 * limit {\n    use missing;\n  }\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
                CompilationStage::Resolution,
                "INV-1",
            ),
        ] {
            let failure = compile(
                &[SourceInput::new(name, source)],
                CompilerLimits::default(),
            )
            .expect_err("the focused invalid proof form must reject");
            assert_eq!(failure.stage(), stage, "{name}: {failure}");
            assert_eq!(failure.kind(), CompilationFailureKind::Source);
            assert_eq!(failure.rule_id(), Some(rule), "{name}: {failure}");
        }
    }

    /// A canonical-form rejection prints the bytes it wanted and the bytes it
    /// found.
    ///
    /// FORM-2 is machine-decided, so the auditor knows both at the point it
    /// stops. It used to print neither, and one double space in an effect row
    /// cost a writer a compile round spent bisecting a byte offset.
    #[test]
    fn a_canonical_rejection_prints_the_expected_bytes_beside_the_found_bytes() {
        let source = b"command fn main() -> status: own ExitStatus pure {\n  doc \"One double space where canonical form admits one space.\";\n  return exit_status(code:  0_u8);\n}\n";
        let failure = compile(
            &[SourceInput::from_host_path(
                "input0.wf",
                "/absolute/path/report.wf",
                source,
            )],
            CompilerLimits::default(),
        )
        .expect_err("a double space is not canonical form");
        assert_eq!(failure.rule_id(), Some("FORM-2"));
        let detail = failure.detail();
        assert!(detail.contains(r#"expected: " ", found: "  ""#), "{detail}");
        assert!(detail.contains("/absolute/path/report.wf:3:"), "{detail}");
    }

    /// The one rule the blind writer could not apply from the specification
    /// text now says what it means and states the whole working idiom.
    ///
    /// This is the writer's own shape: reserve a permit from a borrowed
    /// factory and open through it, which is what every recursive directory
    /// walker wants and what a one-statement region cannot hold. Batch 0099
    /// gave the rejection two routes and the verification writer took neither
    /// to a working walker — `replace` cannot commit where the call consumed
    /// the target's root, which `move permit` does, and the helper alone is
    /// one third of the idiom `tests/programs/dir_walk.wf` uses.
    #[test]
    fn a_child_reborrow_rejection_states_the_scope_rule_and_the_whole_idiom() {
        let source = br#"fn walk['c](factory: &uniq FileFactory, root: &'c DirectoryRead, name: &'c buffer<u8>) -> result: own u8 reads(factory, root, name), writes(factory) {
  region {
    let permit = reserve_file(factory: &uniq deref(factory));
    match open_file(permit: move permit, root: root, name: name, start: 0_u64, end: 1_u64) {
      Ok(value: handle) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return 0_u8;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let name = buffer_new(16_u64, 0_u8);
  let code = 0_u8;
  region {
    set code = walk(factory: &uniq files, root: &cwd, name: &name);
  }
  return exit_status(code: code);
}
"#;
        let failure = compile(
            &[SourceInput::new("walk.wf", source)],
            CompilerLimits::default(),
        )
        .expect_err("a two-statement region cannot carry a child reborrow");
        assert_eq!(failure.rule_id(), Some("OWN-6"));
        let detail = failure.detail();
        // What the rule means, in the two facts a writer meets at once.
        assert!(
            detail.contains(
                "a child reborrow's region admits exactly one statement, and a value that \
                 statement binds dies at the region's end"
            ),
            "{detail}"
        );
        // All three parts of the idiom, in the vocabulary `docs/patterns.md`
        // uses, and the exact limit of the `replace` route.
        assert!(
            detail.contains(
                "move the reserve and the open into one helper that takes the holder as \
                 `&uniq 'f` and returns the opened value"
            ),
            "{detail}"
        );
        assert!(
            detail.contains(
                "make the single statement of the region the `match` on that helper's call"
            ),
            "{detail}"
        );
        assert!(
            detail.contains(
                "write every statement that uses the opened value inside that `match` arm"
            ),
            "{detail}"
        );
        assert!(
            detail.contains("P4 linear threading, P15 recursive walker"),
            "{detail}"
        );
        assert!(
            detail.contains(
                "applies only where the call leaves the target's root alive: a call that \
                 consumes the target root — one taking `move permit` — rejects OWN-1 instead"
            ),
            "{detail}"
        );
    }

    /// A post-syntax rejection names the file it is talking about and quotes
    /// the line, in both stages that reject source after parsing.
    ///
    /// The blind-writer trial's six rejections all printed `SourceId(0)` and a
    /// byte offset, and the writer ran `head -c` on their own program to find
    /// out what the offset meant. Semantics and resolution both already hold
    /// the coordinate the rule selected; this is that coordinate resolved.
    #[test]
    fn a_post_syntax_rejection_names_its_file_and_quotes_its_line() {
        let host = "/absolute/path/counts.wf";

        // [OWN-1], reached in the semantic checker.
        let affine = br#"struct Counts {
  lines: u64;
}

command fn main() -> status: own ExitStatus pure {
  let running = Counts(lines: 0_u64);
  let totals = running;
  return exit_status(code: 0_u8);
}
"#;
        let failure = compile(
            &[SourceInput::from_host_path("input0.wf", host, affine)],
            CompilerLimits::default(),
        )
        .expect_err("a bare affine use is rejected");
        assert_eq!(failure.rule_id(), Some("OWN-1"));
        let detail = failure.detail();
        assert!(detail.contains(&format!("{host}:7:16")), "{detail}");
        assert!(detail.contains("let totals = running;"), "{detail}");
        assert!(!detail.contains("input0.wf"), "{detail}");

        // [TYPE-6], reached in the resolver.
        let collision = br#"command fn main() -> status: own ExitStatus pure {
  let permit = 1_u64;
  region {
    let permit = 2_u64;
  }
  return exit_status(code: 0_u8);
}
"#;
        let failure = compile(
            &[SourceInput::from_host_path("input0.wf", host, collision)],
            CompilerLimits::default(),
        )
        .expect_err("a redeclared binder is rejected");
        assert_eq!(failure.rule_id(), Some("TYPE-6"));
        let detail = failure.detail();
        assert!(detail.contains(&format!("{host}:4:9")), "{detail}");
        assert!(detail.contains("let permit = 2_u64;"), "{detail}");
    }

    /// A lexical rejection names the host path too.
    ///
    /// It is the one stage that already printed a path of its own, from the
    /// span rather than from a wrapper, and the path it printed was the
    /// bundle's positional key — so the first rejection a writer can possibly
    /// receive was also the one that cited a file that does not exist.
    #[test]
    fn a_lexical_rejection_names_the_host_path() {
        let host = "/absolute/path/pound.wf";
        let source = "command fn main() -> status: own ExitStatus pure {\n  let x = \u{a3};\n  return exit_status(code: 0_u8);\n}\n";
        let failure = compile(
            &[SourceInput::from_host_path(
                "input0.wf",
                host,
                source.as_bytes(),
            )],
            CompilerLimits::default(),
        )
        .expect_err("a non-source byte is rejected");
        assert_eq!(failure.rule_id(), Some("FORM-1"));
        let detail = failure.detail();
        assert!(detail.contains(host), "{detail}");
        assert!(!detail.contains("input0.wf"), "{detail}");
    }

    /// A source read from a host path the closed logical spelling cannot hold
    /// is still named by that host path everywhere a reader looks.
    ///
    /// An absolute path is how a script, a Makefile, and an agent all invoke
    /// this compiler. Renaming it to a positional `input0.wf` made every
    /// ledger line and every byte offset refer to a file that exists nowhere
    /// on disk, so the output was not usable as emitted.
    #[test]
    fn a_ledger_names_the_host_path_the_source_was_read_from() {
        let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set total = total +wrap 1_u64;
          }
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
        let host = "/absolute/path/staged.wf";
        let (_, ledger) = compile_with_permission_ledger(
            &[SourceInput::from_host_path("input0.wf", host, source)],
            CompilerLimits::default(),
            OverlapLowering::Off,
        )
        .expect("the fixture compiles");
        assert!(
            ledger.iter().all(|line| line.contains(host)),
            "every ledger line names the host path: {ledger:?}"
        );
        assert!(
            ledger.iter().all(|line| !line.contains("input0.wf")),
            "the bundle's own key is not reader-facing text: {ledger:?}"
        );
    }

    const TREE_PRELUDE: &str = "enum BoxNode {
  Leaf(w: u64);
  Branch(left: box<BoxNode>, right: box<BoxNode>, w: u64);
}

fn boxed_leaf(w: own u64) -> result: own box<BoxNode> pure {
  let leaf = Leaf(w: w);
  return box_new(move leaf);
}

fn boxed_branch(left: own box<BoxNode>, right: own box<BoxNode>) -> result: own box<BoxNode> pure {
  let branch = Branch(left: move left, right: move right, w: 0_u64);
  return box_new(move branch);
}

";

    /// The ledger states an eligible pair and the chain it composes into, then
    /// says the same of a pair whose recursive closure carries an erased
    /// source proof. Proof syntax changes no runtime footprint and therefore
    /// produces no separate `not-actualizable` class.
    #[test]
    fn the_permission_ledger_reports_eligible_pairs_and_their_chains() {
        let eligible = format!(
            "{TREE_PRELUDE}fn fold(node: &uniq box<BoxNode>) -> result: own u64 reads(node), writes(node) {{
  match deref(deref(node)) {{
    Leaf(w: leaf_w) => {{
      return deref(leaf_w);
    }}
    Branch(left: l, right: r, w: slot) => {{
      let a = fold(node: move l);
      let b = fold(node: move r);
      let total = imax(a, b);
      set deref(slot) = total;
      return total;
    }}
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  region {{
    let total = fold(node: &uniq branch0);
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

        // The same tree fold with one checked proof in the recursive closure.
        // `scaled` makes the fact explicit, the semantic checker verifies it,
        // and lowering erases it before the permission table is consumed.
        let proved = format!(
            "{TREE_PRELUDE}fn scaled(values: own array<u64, 8>, index: own u64) -> result: own u64 pure {{
  let size = len_of(values);
  let bounded = iand(index, 7_u64);
  invariant index_in_range: bounded <= 7_u64;
  return values[bounded];
}}

fn bubble(node: &uniq box<BoxNode>) -> result: own u64 reads(node), writes(node) {{
  match deref(deref(node)) {{
    Leaf(w: leaf_w) => {{
      let w = deref(leaf_w);
      let values = array_new::<u64, 8>(1_u64);
      let touched = scaled(values: move values, index: w);
      return w;
    }}
    Branch(left: l, right: r, w: slot) => {{
      let a = bubble(node: move l);
      let b = bubble(node: move r);
      let total = a +wrap b;
      set deref(slot) = total;
      return total;
    }}
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  region {{
    let total = bubble(node: &uniq branch0);
    if total == 7_u64 {{
    }} else {{
      return exit_status(code: 1_u8);
    }}
  }}
  return exit_status(code: 0_u8);
}}
"
        );
        let ledger = ledger_of("bubble.wf", proved.as_bytes());
        assert_eq!(
            ledger[0],
            "PAR permitted   bubble.wf:32  pair(bubble, bubble)  eligible"
        );
        assert_eq!(
            ledger[1],
            "PAR chain       bubble.wf:32  run(bubble, bubble)  2 members through line 33"
        );
        assert!(
            !ledger.iter().any(|line| line.contains("not-actualizable")),
            "the not-actualizable verdict class no longer exists:\n{}",
            ledger.join("\n")
        );

        // Both programs end with main's own two leaf allocations, which are
        // eligible and do form a chain, and then the branch call that consumes
        // them, which is denied by condition 1. The ledger is in source order,
        // so those lines follow the recursive ones and the file is fully
        // reported.
        assert_eq!(
            ledger[2],
            "PAR permitted   bubble.wf:42  pair(boxed_leaf, boxed_leaf)  eligible"
        );
        assert_eq!(
            ledger[3],
            "PAR chain       bubble.wf:42  run(boxed_leaf, boxed_leaf)  2 members through line 43"
        );
        assert_eq!(
            ledger[4],
            "PAR denied      bubble.wf:43  pair(boxed_leaf, boxed_branch)  condition 1: the operands of s2 read what s1 defines"
        );
        assert_eq!(ledger.len(), 5);
    }

    /// One denial line per numbered condition, each citing that condition and
    /// the source text that refused the overlap. A denial arriving under the
    /// wrong condition, or with an empty citation, fails here.
    #[test]
    fn the_permission_ledger_names_the_condition_that_refused_each_pair() {
        // Condition 2: two `&uniq` actuals resolve to one place, so the line
        // has to name both actuals as the writer wrote them.
        let overlapping = b"fn bump(slot: &uniq u64) -> result: own u64 reads(slot), writes(slot) {
  let seen = deref(slot);
  set deref(slot) = 7_u64;
  return seen;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  region {
    let lo = bump(slot: &uniq cell);
    let hi = bump(slot: &uniq cell);
    let total = imax(lo, hi);
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("bump.wf", overlapping),
            vec![
                "PAR denied      bump.wf:10  pair(bump, bump)  condition 2: the exclusive loan of s1 overlaps the exclusive loan of s2 at &uniq cell vs &uniq cell"
                    .to_owned()
            ]
        );

        // Completion and outside authority no longer form a global row gate.
        // Distinct release capabilities are therefore reported as one
        // eligible pair and chain rather than a synthetic condition-3 denial.
        let capability_releases =
            b"fn release_read_file(file: own ReadFile) -> result: own unit writes(file) {
  return unit;
}

fn release_pair(first: own ReadFile, second: own ReadFile) -> result: own unit writes(first, second) {
  let done_first = release_read_file(file: move first);
  let done_second = release_read_file(file: move second);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("row.wf", capability_releases),
            vec![
                "PAR permitted   row.wf:6  pair(release_read_file, release_read_file)  eligible".to_owned(),
                "PAR chain       row.wf:6  run(release_read_file, release_read_file)  2 members through line 7".to_owned(),
            ]
        );

        // Condition 4: the first statement's `propagate` Err edge leaves the
        // function, so the second statement's write must not run under an
        // overlap the sequential execution skips.
        let propagating = b"fn narrow(v: own u32) -> result: own Result<u8, NarrowError> pure {
  return cvt::<u32, u8>(v);
}

fn stamp(slot: &uniq u8) -> result: own u64 writes(slot) {
  set deref(slot) = 9_u8;
  return 1_u64;
}

fn probe(v: own u32, slot: &uniq u8) -> result: own Result<unit, NarrowError> writes(slot) {
  let narrowed = propagate narrow(v: v);
  let stamped = stamp(slot: move slot);
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
    /// it. The callee is a real `pure` function with a loop of its
    /// own, so the case is about the writer's loop rather than about a body
    /// small enough to be uninteresting.
    #[test]
    fn a_counted_loop_reducing_under_an_associative_operation_is_permitted() {
        let source = b"fn interesting(index: own u64) -> result: own Bool pure {
  let low = iand(index, 7_u64);
  let seen = 0_u64;
  loop @spin {
    let done = seen == 4_u64;
    if done {
      break @spin;
    }
    set seen = seen +wrap 1_u64;
  }
  return low == 3_u64;
}

command fn main() -> status: own ExitStatus pure {
  let hits = 0_u64;
  for @scan (i in 0_u64..4096_u64) {
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
  for @sum (i in 0_u64..1024_u64) {
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
  for @sum (i in 0_u64..1024_u64) {
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

    /// A counted loop whose proved index is exactly its binder is reported as
    /// an eligible map with no accumulator.
    #[test]
    fn a_proven_counted_binder_buffer_map_is_permitted() {
        let source = b"command fn main() -> status: own ExitStatus pure {
  let out = buffer_new(64_u64, 0_u64);
  for @fill (i in 0_u64..64_u64) {
    set out[i] = i *wrap i;
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("mapping.wf", source),
            vec![
                "PAR loop        mapping.wf:3  loop  permitted   eligible; no accumulator"
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
            b"fn accum(slot: &uniq f64, x: own f64) -> result: own u64 reads(slot), writes(slot) {
  set deref(slot) = fadd.strict(deref(slot), x);
  let bits = reinterpret::<f64, u64>(deref(slot));
  return iand(bits, 1_u64);
}

command fn main() -> status: own ExitStatus pure {
  let total = 0.0_f64;
  let count = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    let one = accum(slot: &uniq total, x: 0.5_f64);
    set count = count +wrap one;
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("carrying.wf", source),
            vec![
                "PAR loop        carrying.wf:10  loop  denied      condition 2: an iteration \
                 holds an exclusive loan on storage the iteration does not introduce, \
                 at &uniq total"
                    .to_owned()
            ]
        );

        // The same loop over a callee that writes nothing is permitted, so the
        // refusal above is about the projected row and not about the shape.
        let reading = b"fn weigh(x: own f64) -> result: own u64 pure {
  let bits = reinterpret::<f64, u64>(x);
  return iand(bits, 1_u64);
}

command fn main() -> status: own ExitStatus pure {
  let total = 0.0_f64;
  let count = 0_u64;
  for @sum (i in 0_u64..8_u64) {
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
        let source =
            b"fn scan_until(src: &buffer<u64>, needle: own u64) -> result: own u64 reads(src) {
  let count = len_of(deref(src));
  let acc = 0_u64;
  let always = True();
  let answer = if always {
    for @scan (i in 0_u64..count) {
      let v = deref(src)[i];
      set acc = acc +wrap v;
      let hit = v == needle;
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

command fn main() -> status: own ExitStatus pure {
  let data = buffer_new(64_u64, 1_u64);
  set data[10_u64] = 7_u64;
  region {
    let t = scan_until(src: &data, needle: 7_u64);
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
        let contained =
            b"fn scan_until(src: &buffer<u64>, needle: own u64) -> result: own u64 reads(src) {
  let count = len_of(deref(src));
  let acc = 0_u64;
  let always = True();
  let answer = if always {
    for @scan (i in 0_u64..count) {
      let v = deref(src)[i];
      set acc = acc +wrap v;
    }
    give 4096_u64;
  } else {
    give 4096_u64;
  }
  return answer +wrap acc;
}

command fn main() -> status: own ExitStatus pure {
  let data = buffer_new(64_u64, 1_u64);
  set data[10_u64] = 7_u64;
  region {
    let t = scan_until(src: &data, needle: 7_u64);
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
  for @scan (i in 0_u64..64_u64) {
    let low = iand(i, 1_u64);
    let bit = low == 0_u64;
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

    /// The staged verdict of a loop that performs I/O, and the disposition
    /// table underneath it.
    ///
    /// The table is the teaching channel the ledger exists for: a reader sees
    /// what every place the body touches cost, not only that a loop was
    /// granted. Asserting the whole block rather than the verdict line is
    /// deliberate — a table that silently lost a row would still report a
    /// grant, and a grant whose table is wrong is exactly the failure a
    /// permission rule cannot afford.
    #[test]
    fn the_permission_ledger_reports_a_granted_stage_and_its_disposition_table() {
        let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set total = total +wrap 1_u64;
          }
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
        assert_eq!(
            ledger_of("staged.wf", source),
            vec![
                // The counted rule refuses the same loop, and says why: the
                // short unique factory loan is an exclusive loan on storage the
                // iteration does not introduce. The staged rule admits exactly
                // that loan, because prologues run in index order and never
                // overlap. Both lines are printed, and neither judgment reads
                // the other's verdict. Both anchor on the loop head, so a
                // reader matching the two lines up does not have to know that
                // one judgment cites the loop and the other its submission.
                "PAR loop        staged.wf:3  loop  denied      condition 2: an iteration holds \
                 an exclusive loan on storage the iteration does not introduce, at \
                 &uniq files"
                    .to_owned(),
                "PAR stage       staged.wf:3  for   permitted   staged at \
                 open_file(permit: move permit, root: &'f cwd, name: &name, \
                 start: 0_u64, end: 4_u64); 4 places classified"
                    .to_owned(),
                "PAR place       staged.wf:3  serialized-P  &uniq files  every footprint \
                 element and loan touching it belongs to the prologue, and prologues run in \
                 index order without overlapping"
                    .to_owned(),
                "PAR place       staged.wf:3  read-only     &'f cwd  no footprint of the body \
                 writes it or any place overlapping it, and every loan on it is shared"
                    .to_owned(),
                "PAR place       staged.wf:3  serialized-E  set total = total +wrap 1_u64;  \
                 every footprint element and loan touching it belongs to the remainder, whose \
                 accesses to storage rooted outside the loop are taken in index order"
                    .to_owned(),
                "PAR place       staged.wf:3  replicated    let name = buffer_new(16_u64, \
                 97_u8);  iteration-own storage with copy elements, which an implementation may \
                 give each in-flight iteration its own of"
                    .to_owned(),
            ]
        );
    }

    /// The disposition table prints one row per classified place even when two
    /// rows come out byte-identical.
    ///
    /// Every operand read of one statement is cited at that statement, so the
    /// two enclosing buffers this body reads in one `let` carry the same
    /// citation, the same disposition, and the same reason. Collapsing lines by
    /// their text alone dropped one of them and printed a five-row table under
    /// a `stage` line counting six places — a table that is evidence of nothing
    /// if the reader cannot tell a missing place from a repeated one. The
    /// collapse that keeps two instances of one generic to one reported site
    /// still holds, because those two rows agree on their position in the table
    /// as well as on their text.
    #[test]
    fn a_disposition_table_keeps_one_row_per_place_when_two_rows_read_alike() {
        let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let name = buffer_new(16_u64, 97_u8);
  let left = buffer_new(8_u64, 1_u8);
  let right = buffer_new(8_u64, 2_u8);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let permit = reserve_file(factory: &uniq files);
    region {
      match open_file(permit: move permit, root: &cwd, name: &name, start: 0_u64, end: 4_u64) {
        Ok(value: handle) => {
          let sum = left[0_u64] +wrap right[0_u64];
          let wide = cvt::<u8, u64>(sum);
          set total = total +wrap wide;
        }
        Err(error: problem) => {
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
        let ledger = ledger_of("alike.wf", source);
        let stage = ledger
            .iter()
            .find(|line| line.starts_with("PAR stage"))
            .expect("the loop performs I/O and carries a stage line");
        assert!(
            stage.ends_with("; 6 places classified"),
            "the two buffers are two places: {stage}"
        );
        let places: Vec<&String> = ledger
            .iter()
            .filter(|line| line.starts_with("PAR place"))
            .collect();
        assert_eq!(
            places.len(),
            6,
            "the table has a row for every place the stage line counts: {ledger:?}"
        );
        let shared = "PAR place       alike.wf:6  read-only     let sum = left[0_u64] +wrap \
                      right[0_u64];  no footprint of the body writes it or any place \
                      overlapping it, and every loan on it is shared";
        assert_eq!(
            places.iter().filter(|line| line.as_str() == shared).count(),
            2,
            "both buffers are read-only and both are printed: {ledger:?}"
        );
    }

    /// A denial names the numbered condition, the place, and one admitted
    /// writer form.
    ///
    /// The writer form is what makes the line worth printing: "this loop got no
    /// pipeline" teaches nothing, while "allocate the scratch storage inside the
    /// loop body" is a change the writer can make. It comes from the judgment
    /// itself, so it cannot drift from the condition that produced it.
    #[test]
    fn the_permission_ledger_names_the_condition_the_place_and_the_admitted_form() {
        let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let name = buffer_new(16_u64, 97_u8);
  let data = buffer_new(64_u64, 0_u8);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let permit = reserve_file(factory: &uniq files);
    region {
      match open_file(permit: move permit, root: &cwd, name: &name, start: 0_u64, end: 4_u64) {
        Ok(value: handle) => {
          region 'h {
            region {
              match read_at(file: &'h handle, destination: &uniq data, file_offset: 0_u64, start: 0_u64, end: 64_u64) {
                ReadBytes(next: produced) => {
                  set total = total +wrap produced;
                }
                ReadEnd() => {
                }
                ReadFailed(error: problem) => {
                }
              }
            }
          }
        }
        Err(error: problem) => {
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
        let ledger = ledger_of("hoisted.wf", source);
        assert_eq!(
            ledger[1],
            "PAR stage       hoisted.wf:5  for   denied      condition 3: a may-suspend call \
             retains a borrow past its own submission on storage the body writes and the \
             iteration does not introduce; instead, allocate the scratch storage inside the \
             loop body, so each iteration owns the buffer it reads and writes, at \
             &uniq data"
        );
        // The table survives the denial, and it names the second place the
        // writer must move as well as the one the verdict cites.
        assert!(
            ledger.iter().any(|line| line.contains(
                "denied        &uniq data  the body writes it and a may-suspend call retains \
                 a borrow of it past its own submission"
            )),
            "the denied place is in the table: {ledger:?}"
        );
    }

    /// The write a condition-3 denial names is a write, and a place is never
    /// reported as overlapping itself.
    ///
    /// A row keeps the first node that *cites* its place, and a read cites one
    /// as readily as a write does. The denied place of
    /// `accept-par3-staged-denied-read-before-write.wf` is cited by the fold
    /// that reads the destination before the transfer fills it, so a denial
    /// that fell back to the row's citation printed the read as the write, told
    /// the writer to stop rewriting a record that is not there, and asserted an
    /// [OWN-7] overlap between one place and itself. Both bodies below are the
    /// same hazard as the hoisted destination above — one buffer the body
    /// writes and a `may-suspend` call retains a borrow of — so both carry that
    /// denial's own advice, and the one whose write is a node of its own names
    /// that node under a phrase that does not assert self-overlap.
    #[test]
    fn a_retained_borrow_denial_names_a_write_and_never_an_overlap_with_itself() {
        let read_first = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let name = buffer_new(16_u64, 97_u8);
  let data = buffer_new(64_u64, 0_u8);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let byte = data[0_u64];
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            region 'h {
              region {
                match read_at(file: &'h handle, destination: &uniq data, file_offset: 0_u64, start: 0_u64, end: 64_u64) {
                  ReadBytes(next: produced) => {
                    set total = total +wrap produced;
                  }
                  ReadEnd() => {
                  }
                  ReadFailed(error: problem) => {
                  }
                }
              }
            }
          }
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
        let ledger = ledger_of("read_first.wf", read_first);
        assert_eq!(
            ledger[1],
            "PAR stage       read_first.wf:5  for   denied      condition 3: a may-suspend call \
             retains a borrow past its own submission on storage the body writes and the \
             iteration does not introduce; instead, allocate the scratch storage inside the \
             loop body, so each iteration owns the buffer it reads and writes, at \
             &uniq data"
        );
        assert_eq!(
            ledger[2],
            "PAR place       read_first.wf:5  denied        let byte = data[0_u64];  the body \
             writes it and a may-suspend call retains a borrow of it past its own submission"
        );

        // The same body with a write of the destination in front of the
        // transfer instead of a read. The write is now a node of its own, and
        // the denial names it as the write it is.
        let write_first = String::from_utf8(read_first.to_vec())
            .expect("the fixture is text")
            .replace("let byte = data[0_u64];", "set data[0_u64] = 7_u8;");
        let ledger = ledger_of("write_first.wf", write_first.as_bytes());
        assert_eq!(
            ledger[1],
            "PAR stage       write_first.wf:5  for   denied      condition 3: a may-suspend call \
             retains a borrow past its own submission on storage the body writes and the \
             iteration does not introduce; instead, allocate the scratch storage inside the \
             loop body, so each iteration owns the buffer it reads and writes, at \
             &uniq data, and the body writes it at set data[0_u64] = 7_u8;"
        );
    }

    /// Two nested loops whose only submission is the inner one's print at
    /// their own heads, not both at the shared cut.
    ///
    /// The inner loop holds the body's first `may-suspend` call, so that call
    /// is the outer loop's first submission too and both judgments cite it.
    /// Anchoring the line on the cut printed two verdicts at one source
    /// position and a reader could not tell which loop either belonged to.
    #[test]
    fn nested_loops_sharing_one_cut_print_at_their_own_heads() {
        let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  for @outer (step in 0_u64..2_u64) {
    let shared = buffer_new(16_u64, 97_u8);
    for @scan (index in 0_u64..4_u64) {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &cwd, name: &shared, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
          }
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
        let ledger = ledger_of("nested.wf", source);
        let stages: Vec<&String> = ledger
            .iter()
            .filter(|line| line.starts_with("PAR stage"))
            .collect();
        assert_eq!(stages.len(), 2, "one line per loop: {ledger:?}");
        assert!(
            stages[0].starts_with("PAR stage       nested.wf:2  for   denied      condition 1"),
            "the outer loop is anchored on its own head: {stages:?}"
        );
        assert!(
            stages[1].starts_with("PAR stage       nested.wf:4  for   permitted"),
            "the inner loop is anchored on its own head: {stages:?}"
        );
    }

    /// A loop whose body performs no I/O has no cut, so it gets no `stage`
    /// line at all. The staged judgment adds ledger volume exactly where it has
    /// something to say, and every counted loop that had one `loop` line still
    /// has exactly that.
    #[test]
    fn a_loop_without_io_gets_a_counted_line_and_no_staged_line() {
        let source = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("counting.wf", source),
            vec![
                "PAR loop        counting.wf:3  loop  permitted   eligible; one accumulator \
                 under +wrap"
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
            "{TREE_PRELUDE}fn fold(node: &uniq box<BoxNode>) -> result: own u64 reads(node), writes(node) {{
  match deref(deref(node)) {{
    Leaf(w: leaf_w) => {{
      return deref(leaf_w);
    }}
    Branch(left: l, right: r, w: slot) => {{
      let a = fold(node: move l);
      let b = fold(node: move r);
      let total = imax(a, b);
      set deref(slot) = total;
      return total;
    }}
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  region {{
    let total = fold(node: &uniq branch0);
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
            requested.contains("wf__par_acquire_lane"),
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
        let source = b"command fn main() -> status: own ExitStatus pure {\n  let values = array_new::<u8, 18446744073709551615>(0_u8);\n  return exit_status(code: 0_u8);\n}\n";
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
    fn u16_buffer_whose_proved_count_exceeds_the_target_byte_domain_is_a_target_failure() {
        let source = br#"fn bounded_count(n: own u64) -> result: own u64 pure contract {
  ensures result <= 5000000000000000000_u64;
} {
  if n <= 5000000000000000000_u64 {
    return n;
  } else {
    return 5000000000000000000_u64;
  }
}

fn make(n: own u64) -> result: own buffer<u16> pure {
  let bounded = bounded_count(n: n);
  return buffer_new(bounded, 0_u16);
}

command fn main() -> status: own ExitStatus pure {
  let values = make(n: 4_u64);
  return exit_status(code: 0_u8);
}
"#;
        let failure = compile(
            &[SourceInput::new("value.wf", source)],
            CompilerLimits::default(),
        )
        .expect_err("the proved u16 byte ceiling exceeds the selected target domain");
        assert_eq!(failure.stage(), CompilationStage::TargetLayout);
        assert_eq!(failure.kind(), CompilationFailureKind::TargetLayout);
        assert_eq!(failure.rule_id(), None);
        assert!(failure.detail().contains("RuntimeSizedAllocation"));
    }

    #[test]
    fn complete_frame_is_checked_after_each_slot_layout_succeeds() {
        let source = b"command fn main() -> status: own ExitStatus pure {\n  let left = array_new::<u8, 4611686018427387904>(0_u8);\n  let right = array_new::<u8, 4611686018427387904>(0_u8);\n  return exit_status(code: 0_u8);\n}\n";
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
        // `writes(cwd)` from the DirectoryRead input's compiler-derived close
        // attempt on the return edge. [QUAL-1] qualification now maps
        // each identity to an approved implementation and the [QUAL-3]
        // bootstrap supplies the standard inputs, so the program emits.
        let kind_entry = b"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output, command.files as files: own FileFactory) -> status: own ExitStatus writes(cwd) {\n  return exit_status(code: 0_u8);\n}\n";
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

        // `open_read`, `read_at`, and `write_once` complete the qualified
        // interface: every [SYS-2] semantic identity now has an approved
        // implementation on this target, so no unsupported stop remains
        // between an accepted system program and its emitted module.
        let writing =b"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out) {\n  let bytes = buffer_new(1_u64, 65_u8);\n  region 'o {\n    region {\n      match write_once(output: &uniq 'o out, source: &bytes, start: 0_u64, end: 1_u64) {\n        Ok(value: written) => {\n          return exit_status(code: 0_u8);\n        }\n        Err(error: problem) => {\n          return exit_status(code: 1_u8);\n        }\n      }\n    }\n  }\n}\n";
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

        // State rows are checked in definition order. The valid but unused
        // `reads(args)` path reaches EFF-2's exact-row check. `writes(file)`
        // names a shared parameter, so EFF-1 rejects that malformed row before
        // a body can be compared with it.
        for (source, rule) in [
            (
                b"fn probe(args: own Args) -> result: own unit reads(args) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n".as_slice(),
                "EFF-2",
            ),
            (
                b"fn probe(file: &ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
                "EFF-1",
            ),
        ] {
            let failure = compile(
                &[SourceInput::new("rejected.wf", source)],
                CompilerLimits::default(),
            )
            .expect_err("the invalid state row must reject at its earliest rule");
            assert_eq!(failure.stage(), CompilationStage::Semantics);
            assert_eq!(failure.kind(), CompilationFailureKind::Source);
            assert_eq!(failure.rule_id(), Some(rule));
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

    // -----------------------------------------------------------------------
    // Batch 0100: the payloads the verification re-writer of 2026-08-28 asked
    // for, each pinned by the exact rendered text a writer reads. A change to
    // any of these sentences is a change to what the compiler teaches, and has
    // to be made here on purpose.
    // -----------------------------------------------------------------------

    /// The numbered rule and the rendered detail of one compilation that must
    /// fail, in the shape `whitefootc` prints them.
    fn rejection(name: &str, source: &[u8]) -> String {
        let failure = compile(&[SourceInput::new(name, source)], CompilerLimits::default())
            .expect_err("this fixture exists to be rejected");
        format!(
            "[{}] {}",
            failure.rule_id().unwrap_or("no rule"),
            failure.detail()
        )
    }

    /// [GRAM-9] names the binding form its grammar position admits.
    ///
    /// The rule's own repair is "bind the computed value with a preceding
    /// `let`", and inside a `contract_block` that repair is wrong: the block
    /// has no `let_stmt` and its binding form is `define IDENT = expr;`. The
    /// position is read from the open production frames, never from the text.
    #[test]
    fn a_forbidden_atom_names_the_binding_form_its_grammar_position_admits() {
        let body = rejection(
            "body.wf",
            br#"fn double(value: own u64) -> out: own u64 pure {
  return value +wrap value;
}

fn helper(value: own u64) -> out: own u64 pure {
  let a = double(value: double(value: value));
  return a;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        );
        assert!(body.contains("[GRAM-9]"), "{body}");
        assert!(
            body.contains(
                r#"mechanical_fix: "a `call` or `construct` in an atom position does not derive [GRAM-9]: bind the inner call with its own preceding `let` in this body and write that binder in the atom position — `let inner = f(x: 0_u64); let outer = g(y: inner);`""#
            ),
            "{body}"
        );

        let contract = rejection(
            "contract.wf",
            br#"fn count(data: &buffer<u8>, start: own u64, end: own u64) -> lines: own u64 reads(data) contract {
  requires buffer_fits::<u8>(len_of(deref(data)));
} {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        );
        assert!(contract.contains("[GRAM-9]"), "{contract}");
        assert!(
            contract.contains(
                r#"mechanical_fix: "a `call` or `construct` in an atom position does not derive [GRAM-9]: a `contract_block` has no `let`, so bind the inner call with a preceding `define` in this same block and write that binder in the atom position — `define inner = f(x: 0_u64); requires g(y: inner);`""#
            ),
            "{contract}"
        );
    }

    /// The `define` route the contract-block repair names is accepted.
    ///
    /// A repair the compiler refuses is worse than no repair, so the two are
    /// pinned together: the rejection above and the program below differ only
    /// by taking it.
    #[test]
    fn the_contract_block_repair_gram9_names_is_accepted() {
        compile(
            &[SourceInput::new(
                "repaired.wf",
                br#"fn count(data: &buffer<u8>, start: own u64, end: own u64) -> lines: own u64 pure contract {
  define spare = len_of(deref(data));
  requires end <= spare;
} {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
            )],
            CompilerLimits::default(),
        )
        .expect("the repair GRAM-9 names must be accepted");
    }

    /// [EFF-1] states the condition the row failed and the row that repairs it.
    ///
    /// `writes(cwd), writes(out)` is two occurrences of one category, which the
    /// rule forbids in one sentence the diagnostic did not carry.
    #[test]
    fn an_effect_row_defect_names_its_condition_and_the_row_that_repairs_it() {
        let detail = rejection(
            "row.wf",
            br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output) -> status: own ExitStatus reads(cwd, out), writes(cwd), writes(out) {
  return exit_status(code: 0_u8);
}
"#,
        );
        assert!(detail.contains("[EFF-1]"), "{detail}");
        assert!(
            detail.contains(
                r#"reason: "a category appears at most once in one row, and the row is written in the canonical order reads, writes, allocates""#
            ),
            "{detail}"
        );
        assert!(
            detail.contains(
                r#"mechanical_fix: "merge the repeated category's paths into one occurrence — `writes(cwd), writes(out)` is `writes(cwd, out)` — and order the categories reads, writes, allocates""#
            ),
            "{detail}"
        );
    }

    /// [EFF-2] publishes both rows and the exact difference between them.
    ///
    /// Four blind-writer rounds met a bare `EffectMismatch`: the writer was
    /// told two rows differ and had to derive both sides by hand.
    #[test]
    fn an_effect_mismatch_publishes_both_rows_and_the_exact_difference() {
        let detail = rejection(
            "effects.wf",
            br#"fn count(data: &buffer<u8>) -> lines: own u64 reads(data) {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        );
        assert!(detail.contains("[EFF-2]"), "{detail}");
        assert!(detail.contains(r#"expected_row: "pure""#), "{detail}");
        assert!(detail.contains(r#"found_row: "reads(data)""#), "{detail}");
        assert!(detail.contains("missing: []"), "{detail}");
        assert!(detail.contains(r#"extra: ["reads(data)"]"#), "{detail}");
        assert!(
            detail.contains(
                r#"mechanical_fix: "declare exactly the row the body exhibits: add every missing category and path and remove every extra one; EFF-2 admits no wider and no narrower declaration than the union of the body-syntactic and release contributions""#
            ),
            "{detail}"
        );
    }

    /// [TYPE-5] publishes the two sides it compared.
    #[test]
    fn a_type_mismatch_publishes_the_type_required_and_the_type_written() {
        let detail = rejection(
            "types.wf",
            br#"command fn main() -> status: own ExitStatus pure {
  let a = 1_u64;
  let b = 2_u32;
  let c = a <= b;
  return exit_status(code: 0_u8);
}
"#,
        );
        assert!(detail.contains("[TYPE-5]"), "{detail}");
        assert!(
            detail.contains(r#"TypeMismatch { expected: "own u64", found: "own u32" }"#),
            "{detail}"
        );
    }

    /// A generic form written with no type arguments names both spellings that
    /// carry them.
    ///
    /// A writer meeting this at `Ok(value: v)` sees a constructor name and no
    /// type anywhere, so naming the type spelling alone would not locate the
    /// repair.
    #[test]
    fn a_generic_form_without_type_arguments_names_both_spellings() {
        let detail = rejection(
            "result.wf",
            br#"fn helper(value: own u8) -> out: own Result<u8, unit> pure {
  return Ok(value: value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        );
        assert!(detail.contains("[TYPE-5]"), "{detail}");
        assert!(
            detail.contains(
                r#"expected: "Result with both type arguments written: as a type `Result<u64, IoError>`, and as a variant constructor `Ok<u64, IoError>(value: v)`", found: "Result with no written type-argument list""#
            ),
            "{detail}"
        );
        // And the spelling it names is accepted.
        compile(
            &[SourceInput::new(
                "result-repaired.wf",
                br#"fn helper(value: own u8) -> out: own Result<u8, unit> pure {
  return Ok<u8, unit>(value: value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
            )],
            CompilerLimits::default(),
        )
        .expect("the constructor spelling TYPE-5 names must be accepted");
    }

    /// [OWN-10] names the region, the binder, and where a region it admits has
    /// to be introduced.
    #[test]
    fn a_borrow_lifetime_rejection_names_the_region_the_binder_and_the_repair() {
        let detail = rejection(
            "lifetime.wf",
            br#"fn sum(data: &buffer<u8>) -> out: own u64 reads(data) {
  return len_of(deref(data));
}

fn caller['r](anchor: &'r buffer<u8>) -> out: &'r buffer<u8> pure {
  let local = buffer_new(4_u64, 0_u8);
  let counted = sum(data: &'r local);
  return anchor;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        );
        assert!(detail.contains("[OWN-10]"), "{detail}");
        assert!(
            detail.contains(
                r#"InvalidBorrowLifetime { region: "'r", binder: "local", mechanical_fix: "a borrow of local storage names a region introduced inside that binding's own scope: write `region 'r { ... }` after the binding and take the borrow inside it. A caller-supplied region parameter is never admitted here, because it outlives the storage." }"#
            ),
            "{detail}"
        );
    }

    /// [FORM-2] quotes the line its offending bytes are in.
    ///
    /// The coordinate is the trivia gap between two terminals, and a gap that
    /// carries a line break starts at the end of the line *before* the one the
    /// writer must edit: the verification writer was shown the enclosing item's
    /// header with a byte offset two lines further down.
    #[test]
    fn a_canonical_gap_quotes_the_line_its_offending_bytes_are_in() {
        let detail = rejection(
            "indent.wf",
            b"fn helper(value: own u64) -> out: own u64 pure {\n  let a = value +wrap 1_u64;\n    let b = a +wrap 2_u64;\n  return b;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        );
        assert!(detail.contains("[FORM-2]"), "{detail}");
        assert!(
            detail.contains(r#"at indent.wf:3:1 in line "    let b = a +wrap 2_u64;""#),
            "{detail}"
        );

        // A gap that stays inside one line is unchanged: the reader is sent to
        // the first byte of the gap, which is where the wrong bytes begin.
        let inline = rejection(
            "spacing.wf",
            b"fn helper(value: own u64) -> out: own u64 pure {\n  let a = value +wrap 1_u64;\n  let b = a  +wrap 2_u64;\n  return b;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        );
        assert!(
            inline.contains(r#"at spacing.wf:3:12 in line "  let b = a  +wrap 2_u64;""#),
            "{inline}"
        );
    }

    /// One output stream written per iteration is offered the remedy that
    /// works, and the program that takes it is granted.
    ///
    /// Replication is not advice a writer can take for stdout, and "leave this
    /// loop sequential" was the only other half of the sentence. The remedy
    /// that works is to take the write out of the loop: the two programs below
    /// differ in that and in nothing else.
    #[test]
    fn a_one_position_resource_is_offered_the_hoist_that_works() {
        const EMIT: &str = r#"fn emit(out: &uniq Output, value: own u8) -> written: own u64 reads(out), writes(out) {
  let one = buffer_new(1_u64, value);
  let sent = 0_u64;
  region {
    match write_once(output: &uniq deref(out), source: &one, start: 0_u64, end: 1_u64) {
      Ok(value: n) => {
        set sent = n;
      }
      Err(error: e) => {
      }
    }
  }
  return sent;
}
"#;
        let per_iteration = format!(
            "{EMIT}
command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out) {{
  for @scan (index in 0_u64..4_u64) {{
    let wrote = emit(out: &uniq out, value: 65_u8);
  }}
  return exit_status(code: 0_u8);
}}
"
        );
        let notices = notices_of("stream.wf", per_iteration.as_bytes());
        let staged = notices
            .iter()
            .find(|notice| notice.starts_with("PAR stage"))
            .unwrap_or_else(|| panic!("the per-iteration write must deny the stage: {notices:?}"));
        assert!(
            staged.contains(
                // Written on one line: a sentence a test pins has to be
                // greppable as the bytes a writer reads.
                "instead, give each iteration its own resource; or, where the body only publishes to that storage — an output stream is the pointed case — hoist the per-iteration write out of the loop, folding a total in the body and writing it once after the loop; or leave this loop sequential, because storage that carries one position cannot be held by two iterations at once"
            ),
            "{staged}"
        );

        // The same program with the write hoisted out: the loop is granted and
        // the default channel says nothing about it.
        let hoisted = format!(
            "{EMIT}
command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, out, files), writes(cwd, out, files) {{
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {{
    let name = buffer_new(16_u64, 97_u8);
    region 'f {{
      let permit = reserve_file(factory: &uniq files);
      region {{
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {{
          Ok(value: handle) => {{
            set total = total +wrap 1_u64;
          }}
          Err(error: problem) => {{
          }}
        }}
      }}
    }}
  }}
  region {{
    let wrote = emit(out: &uniq out, value: 65_u8);
  }}
  return exit_status(code: 0_u8);
}}
"
        );
        assert!(
            notices_of("hoisted-stream.wf", hoisted.as_bytes()).is_empty(),
            "the hoisted form is what the remedy names, so it must be granted"
        );
    }
}
