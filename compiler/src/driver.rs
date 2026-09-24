//! One ordinary active-specification compilation pipeline.
//!
//! The driver keeps source failures, unsupported compiler capabilities,
//! resource failures, invariant failures, lowering failures, and backend
//! failures distinct while returning owned LLVM assembly to callers.

use core::fmt;

mod rejection;

pub(crate) mod launcher;
/// The probe corpus that pins every diagnostic sentence by its rendered text.
#[cfg(test)]
mod pinned_sentences;

use rejection::Located;

use crate::backend::{emitter::emit_llvm_with_layout, target::TargetLayout};
use crate::{
    ACTIVE_KERNEL_SPEC_HASH, BackendFailure, CanonicalLimits, CanonicalOutcome,
    CanonicalSyntaxUnit, CheckedProgram, FinalizeLimits, FinalizeOutcome, LexLimits, LexOutcome,
    LoweringFailure, ParseLimits, ParseOutcome, ResolutionOutcome, SemanticLocation,
    SemanticOutcome, SourceBundle, SourceInput, SourceLimits, TerminalLimits, TerminalOutcome,
    audit_canonical, check_semantics, classify_terminals, finalize, lex, parse, parse_graph,
    resolve,
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
    /// Module graph formation and entry selection [MOD-1, MOD-9].
    ModuleGraph,
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
    /// Checked-program to typed IR lowering, including optional loop shapes.
    Lowering,
    /// Selected-target representability and target-domain discharge.
    TargetLayout,
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
    fn source_envelope(failure: crate::SourceBundleError) -> Self {
        use crate::{LogicalPathError, SourceBundleError};

        let kind = match &failure {
            SourceBundleError::LimitExceeded { .. }
            | SourceBundleError::StorageUnavailable { .. }
            | SourceBundleError::ArithmeticOverflow
            | SourceBundleError::LogicalPath(
                LogicalPathError::LengthOverflow | LogicalPathError::StorageUnavailable { .. },
            ) => CompilationFailureKind::Resource,
            SourceBundleError::EmptySourceSequence
            | SourceBundleError::LogicalPath(
                LogicalPathError::Empty
                | LogicalPathError::Absolute
                | LogicalPathError::EmptyComponent
                | LogicalPathError::DotComponent
                | LogicalPathError::InvalidByte { .. },
            )
            | SourceBundleError::DuplicateLogicalPath { .. }
            | SourceBundleError::UnknownModule => CompilationFailureKind::Invocation,
        };
        Self::new(CompilationStage::SourceEnvelope, kind, failure)
    }

    fn new(stage: CompilationStage, kind: CompilationFailureKind, detail: impl fmt::Debug) -> Self {
        Self {
            stage,
            kind,
            rule_id: None,
            detail: format!("{detail:?}"),
        }
    }

    fn lowering(failure: LoweringFailure) -> Self {
        let (stage, kind) = match failure {
            LoweringFailure::TargetLayout(_) => (
                CompilationStage::TargetLayout,
                CompilationFailureKind::TargetLayout,
            ),
            // An unavailable compiler-owned body is a capability stop after
            // semantic acceptance, never a source verdict.
            LoweringFailure::UnimplementedPreludeRow(_) => (
                CompilationStage::Lowering,
                CompilationFailureKind::Unsupported,
            ),
            LoweringFailure::InvalidCheckedProgram | LoweringFailure::CounterOverflow => {
                (CompilationStage::Lowering, CompilationFailureKind::Lowering)
            }
        };
        Self::new(stage, kind, failure)
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

    /// A malformed compiler-supplied declaration is a pipeline defect, not a
    /// rejection of the writer's source bundle.
    fn at_source(
        stage: CompilationStage,
        rule: &'static str,
        detail: impl fmt::Debug,
        bundle: &SourceBundle,
        source: crate::SourceId,
    ) -> Self {
        if bundle
            .file(source)
            .is_some_and(|file| file.prelude().is_some())
        {
            Self::new(stage, CompilationFailureKind::Compiler, detail)
        } else {
            Self::source(stage, rule, detail)
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
/// Calls run sequentially by default; ordinary call outlining is opt-in through
/// [`compile_with_overlap`] and `whitefootc --par`.
pub fn compile(
    inputs: &[SourceInput<'_>],
    limits: CompilerLimits,
) -> Result<String, CompilationFailure> {
    compile_with_overlap(inputs, limits, crate::OverlapLowering::Off)
}

/// Checks one ordered closed source bundle through complete target-independent
/// semantic acceptance and monomorphization.
///
/// This is the source-verdict projection of the same front-end path [`compile`]
/// uses. It stops before lowering, selected-target layout qualification, and
/// executable-caller construction, so a later target or backend failure cannot
/// become a source rejection or erase successful source acceptance [STOR-6].
pub fn check(inputs: &[SourceInput<'_>], limits: CompilerLimits) -> Result<(), CompilationFailure> {
    with_checked_program(inputs, None, limits, |_, _| Ok(()))
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
    compile_reporting(inputs, limits, overlap).map(|reported| reported.module)
}

/// [`compile_with_overlap`] plus the non-normative permission ledger for the
/// same compilation.
///
/// The ledger reports, one line per analyzed sibling-call site, whether the
/// permission judgment allows overlapping the two statements and whether a
/// permitted overlap is actualizable. It is developer output on the caller's
/// own channel: it participates in no mandatory record, changes no accepted
/// program, and selects no lowering. Permission verdicts are independent of
/// the actualization policy; additional actualization lines describe that
/// policy's choices, including omitted scalar-leaf offers. The compiler's
/// `--par-ledger` switch is its caller outside tests.
pub fn compile_with_permission_ledger(
    inputs: &[SourceInput<'_>],
    limits: CompilerLimits,
    overlap: crate::OverlapLowering,
) -> Result<(String, Vec<String>), CompilationFailure> {
    compile_reporting(inputs, limits, overlap).map(|reported| (reported.module, reported.ledger))
}

/// What one module program invocation runs [MOD-9, PROG-3].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleEntry<'a> {
    /// A named entry of the graph, with its requirements.
    Named(&'a str),
    /// Any ordinary function of a registered module, run as an unnamed entry
    /// with no requirement: `module` is `pkg` or `pkg::a::b`.
    Function {
        /// The module's qualified name.
        module: &'a str,
        /// The function's name.
        function: &'a str,
    },
}

/// Forms the module graph one `modules.wfg` record writes [MOD-1].
///
/// The record passes the ordinary syntax stages under the `graph_file` start
/// and then graph formation; any rejection cites its rule at the record.
pub fn form_module_graph(
    graph: SourceInput<'_>,
    limits: CompilerLimits,
) -> Result<crate::ModuleGraph, CompilationFailure> {
    let bundle = SourceBundle::with_limits(&[graph], limits.source)
        .map_err(CompilationFailure::source_envelope)?;
    with_canonical_syntax(
        &bundle,
        limits,
        true,
        |canonical| match crate::graph::form_graph(&canonical) {
            Ok(Ok(graph)) => Ok(graph),
            Ok(Err(issue)) => {
                let coordinate = issue.coordinate();
                Err(CompilationFailure::at_source(
                    CompilationStage::ModuleGraph,
                    issue.rule_id(),
                    Located::new(issue, &bundle, coordinate),
                    &bundle,
                    coordinate.source(),
                ))
            }
            Err(failure) => Err(CompilationFailure::new(
                CompilationStage::ModuleGraph,
                CompilationFailureKind::Compiler,
                failure,
            )),
        },
    )
}

/// Checks every registered module of a module program through complete
/// source acceptance, without selecting an entry [MOD-8].
///
/// `inputs` are the modules' interface and implementation records, each
/// placed with [`SourceInput::in_module`].
pub fn check_module_program(
    graph: &crate::ModuleGraph,
    inputs: &[SourceInput<'_>],
    limits: CompilerLimits,
) -> Result<(), CompilationFailure> {
    with_checked_program(inputs, Some(graph.modules()), limits, |_, _| Ok(()))
}

/// Checks one module against the interfaces it may name [MOD-8]: the
/// module's own interface and implementation records, and the interface
/// records of every module in its dependency closure. No other module's
/// implementation record is read, so the verdict holds while a dependency's
/// implementation is absent, incomplete or failing, and an edit to another
/// module's implementation cannot change it. With `interface_only`, the
/// module's own implementation records are left out as well, which checks
/// the interface an architect writes before any body exists.
pub fn check_module(
    graph: &crate::ModuleGraph,
    inputs: &[SourceInput<'_>],
    module: &str,
    interface_only: bool,
    limits: CompilerLimits,
) -> Result<(), CompilationFailure> {
    let target = graph.module_named(module).ok_or_else(|| {
        CompilationFailure::new(
            CompilationStage::ModuleGraph,
            CompilationFailureKind::Invocation,
            format!("the graph registers no module `{module}`"),
        )
    })?;
    let closure = graph.dependency_closure(target);
    let selected: Vec<_> = inputs
        .iter()
        .copied()
        .filter(|input| {
            if input.module() == target {
                !interface_only || input.role() == crate::SourceRole::Interface
            } else {
                input.role() == crate::SourceRole::Interface && closure.contains(&input.module())
            }
        })
        .collect();
    with_checked_program(&selected, Some(graph.modules()), limits, |_, _| Ok(()))
}

/// Checks a module program through complete source acceptance and admits
/// one of its entries [MOD-9, STOR-8], stopping before lowering.
pub fn check_module_entry(
    graph: &crate::ModuleGraph,
    inputs: &[SourceInput<'_>],
    entry: ModuleEntry<'_>,
    limits: CompilerLimits,
) -> Result<(), CompilationFailure> {
    let selection = entry_selection(graph, entry)?;
    with_checked_program(inputs, Some(graph.modules()), limits, |checked, bundle| {
        admit_entry(&checked, bundle, &selection)
    })
}

/// The function a module program entry names [MOD-9].
fn entry_selection<'graph>(
    graph: &'graph crate::ModuleGraph,
    entry: ModuleEntry<'graph>,
) -> Result<Selection<'graph>, CompilationFailure> {
    let invocation = |detail: String| {
        CompilationFailure::new(
            CompilationStage::ModuleGraph,
            CompilationFailureKind::Invocation,
            detail,
        )
    };
    Ok(match entry {
        ModuleEntry::Named(name) => {
            let entry = graph
                .entry(name)
                .ok_or_else(|| invocation(format!("the graph has no entry named `{name}`")))?;
            Selection {
                module: entry.module(),
                name: entry.function(),
                no_heap: entry.no_heap(),
                public: true,
            }
        }
        ModuleEntry::Function { module, function } => Selection {
            module: graph
                .module_named(module)
                .ok_or_else(|| invocation(format!("the graph registers no module `{module}`")))?,
            name: function,
            no_heap: false,
            public: false,
        },
    })
}

/// Compiles a module program for one named or unnamed entry to textual LLVM
/// [MOD-9, PROG-3].
pub fn compile_module_program(
    graph: &crate::ModuleGraph,
    inputs: &[SourceInput<'_>],
    entry: ModuleEntry<'_>,
    limits: CompilerLimits,
    overlap: crate::OverlapLowering,
) -> Result<String, CompilationFailure> {
    let selection = entry_selection(graph, entry)?;
    compile_selected(inputs, Some(graph.modules()), limits, overlap, &selection)
        .map(|reported| reported.module)
}

/// One compilation's module and the developer-channel text it produced.
struct Reported {
    module: String,
    ledger: Vec<String>,
}

/// The one compilation path, returning the module and the developer-channel
/// permission ledger it produced. Every public entry point above is a
/// projection of this function; there is no second pipeline.
fn compile_reporting(
    inputs: &[SourceInput<'_>],
    limits: CompilerLimits,
    overlap: crate::OverlapLowering,
) -> Result<Reported, CompilationFailure> {
    compile_selected(
        inputs,
        None,
        limits,
        overlap,
        &Selection {
            module: crate::ModuleId::BUNDLE_ROOT,
            name: "main",
            no_heap: false,
            public: false,
        },
    )
}

/// The function one invocation runs [PROG-3]: a source bundle's `main`, or a
/// module program's named or unnamed entry [MOD-9].
struct Selection<'a> {
    module: crate::ModuleId,
    name: &'a str,
    /// The entry states the no-heap requirement [STOR-8].
    no_heap: bool,
    /// A named entry, which must select a public function [MOD-9].
    public: bool,
}

fn compile_selected(
    inputs: &[SourceInput<'_>],
    modules: Option<&[crate::ModuleRecord]>,
    limits: CompilerLimits,
    overlap: crate::OverlapLowering,
    selection: &Selection<'_>,
) -> Result<Reported, CompilationFailure> {
    with_checked_program(inputs, modules, limits, |checked, bundle| {
        if modules.is_some() {
            admit_entry(&checked, bundle, selection)?;
        }
        lower_selected(inputs, modules, limits, overlap, selection, bundle, checked)
    })
}

/// [MOD-9, STOR-8] a module program's entry selects one ordinary function of
/// its module, public for a named entry; a no-heap entry's execution closure
/// introduces no heap requirement, and a closure that does is rejected at
/// the function that introduces it.
fn admit_entry(
    checked: &CheckedProgram<'_, '_, '_>,
    bundle: &SourceBundle,
    selection: &Selection<'_>,
) -> Result<(), CompilationFailure> {
    // [MOD-8] composition needs every declared function's definition; a
    // pending interface declaration blocks it at the declaration.
    if let Some(pending) = checked
        ._resolved
        .interface_functions()
        .iter()
        .find(|function| function.definition().is_none())
        .and_then(|function| checked._resolved.declaration(function.declaration()))
    {
        let coordinate = pending.origin().coordinate();
        let detail = format!(
            "the interface declares `{}`, and no implementation record of its module defines it yet; a module with a pending declaration checks, but no entry composes until the definition exists",
            pending.spelling()
        );
        return Err(CompilationFailure::at_source(
            CompilationStage::Semantics,
            "MOD-8",
            Located::new(detail, bundle, coordinate),
            bundle,
            coordinate.source(),
        ));
    }
    let entry_failure =
        |detail: String| CompilationFailure::source(CompilationStage::Semantics, "MOD-9", detail);
    let Some(function) = checked.data.functions.iter().find(|function| {
        !function.formal_hypothesis
            && function.module == selection.module
            && function.name == selection.name
    }) else {
        return Err(entry_failure(format!(
            "the entry names `{}`, which is no ordinary nongeneric function of its module",
            selection.name
        )));
    };
    let declaration = checked._resolved.declaration(function.declaration);
    if selection.public && !declaration.is_some_and(crate::DeclarationRecord::is_public) {
        return Err(entry_failure(format!(
            "a named entry runs a public function, and `{}` is private to its module",
            selection.name
        )));
    }
    if selection.no_heap
        && let Some(path) = checked.heap_introducer(function.id, |function| {
            crate::lowering::holds_heap_storage(&checked.data, function)
        })
    {
        // The path names each function by its module's path and its source
        // name; an instance keeps its template's name, since the walk passes
        // through it with its supplied actuals.
        let names = path
            .iter()
            .filter_map(|id| checked.data.functions.get(id.0 as usize))
            .map(|function| match bundle.module(function.module) {
                Some(module) => format!("{}::{}", module.qualified_name(), function.name),
                None => function.name.clone(),
            })
            .collect::<Vec<_>>()
            .join(" -> ");
        let introducer = path
            .last()
            .and_then(|id| checked.data.functions.get(id.0 as usize))
            .and_then(|function| checked._resolved.declaration(function.declaration));
        let detail = format!(
            "the entry states no_heap, and its execution closure requires the heap along {names}; the last function on that path introduces the requirement by allocating or by holding a Box or runtime-capacity value"
        );
        return Err(match introducer {
            Some(declaration) => {
                let coordinate = declaration.origin().coordinate();
                CompilationFailure::at_source(
                    CompilationStage::Semantics,
                    "STOR-8",
                    Located::new(detail, bundle, coordinate),
                    bundle,
                    coordinate.source(),
                )
            }
            None => CompilationFailure::source(CompilationStage::Semantics, "STOR-8", detail),
        });
    }
    Ok(())
}

/// Runs the syntax stages over one bundle, from raw lexical formation through
/// the canonical [FORM-2] audit, and lends the canonical unit to one
/// continuation while every borrowed stage input remains alive. A module
/// graph file takes the `graph_file` start and every source bundle the
/// `program` start [GRAM-2, MOD-1]; the stages are otherwise one path.
fn with_canonical_syntax<'bundle, T, F>(
    bundle: &'bundle SourceBundle,
    limits: CompilerLimits,
    graph: bool,
    continuation: F,
) -> Result<T, CompilationFailure>
where
    F: for<'classified, 'lexed> FnOnce(
        CanonicalSyntaxUnit<'classified, 'lexed, 'bundle>,
    ) -> Result<T, CompilationFailure>,
{
    let lexed = match lex(bundle, limits.lexer) {
        LexOutcome::Complete(complete) => complete,
        LexOutcome::SourceIssue(issue) => {
            return Err(CompilationFailure::at_source(
                CompilationStage::Lexing,
                issue.kind().rule_id(),
                issue,
                bundle,
                issue.span().source(),
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
            return Err(CompilationFailure::at_source(
                CompilationStage::TerminalClassification,
                issue.owner().id(),
                issue,
                bundle,
                issue.token().source(),
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
    let parsed = match if graph {
        parse_graph(&classified, limits.parser)
    } else {
        parse(&classified, limits.parser)
    } {
        ParseOutcome::Complete(complete) => complete,
        ParseOutcome::SourceIssue(issue) => {
            let coordinate = issue.coordinate();
            return Err(CompilationFailure::at_source(
                CompilationStage::Parsing,
                issue.rule().id(),
                Located::new(issue, classified.source_bundle(), coordinate),
                bundle,
                coordinate.source(),
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
            return Err(CompilationFailure::at_source(
                CompilationStage::CanonicalSource,
                issue.rule().id(),
                Located::in_gap(issue, classified.source_bundle(), coordinate),
                bundle,
                coordinate.source(),
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
    continuation(canonical)
}

/// Runs the one source front end and lends its checked program to one
/// projection while every borrowed stage input remains alive. Both `check`
/// and `compile` enter here; neither reconstructs a source verdict.
fn with_checked_program<T, F>(
    inputs: &[SourceInput<'_>],
    modules: Option<&[crate::ModuleRecord]>,
    limits: CompilerLimits,
    continuation: F,
) -> Result<T, CompilationFailure>
where
    F: for<'classified, 'lexed, 'source> FnOnce(
        CheckedProgram<'classified, 'lexed, 'source>,
        &SourceBundle,
    ) -> Result<T, CompilationFailure>,
{
    let bundle = match modules {
        Some(modules) => {
            SourceBundle::with_prelude_and_modules(inputs, modules.to_vec(), limits.source)
        }
        None => SourceBundle::with_prelude(inputs, limits.source),
    }
    .map_err(CompilationFailure::source_envelope)?;
    with_canonical_syntax(&bundle, limits, false, |canonical| {
        let classified = canonical.classified_bundle();
        let resolved = match resolve(canonical) {
            ResolutionOutcome::Complete(complete) => complete,
            ResolutionOutcome::SourceIssue { issue, .. } => {
                let coordinate = issue.origin().coordinate();
                return Err(CompilationFailure::at_source(
                    CompilationStage::Resolution,
                    issue.rule().id(),
                    Located::new(issue, classified.source_bundle(), coordinate),
                    &bundle,
                    coordinate.source(),
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
                let SemanticLocation::SourceNode(_, coordinate) = issue.location();
                let coordinate = *coordinate;
                let request = issue.request();
                return Err(CompilationFailure::at_source(
                    CompilationStage::Semantics,
                    rule_id,
                    Located::new(issue, classified.source_bundle(), coordinate)
                        .requested_at(classified.source_bundle(), request),
                    &bundle,
                    coordinate.source(),
                ));
            }
            SemanticOutcome::ResolutionIssue { issue, .. } => {
                let coordinate = issue.origin().coordinate();
                return Err(CompilationFailure::at_source(
                    CompilationStage::Resolution,
                    issue.rule().id(),
                    Located::new(issue, classified.source_bundle(), coordinate),
                    &bundle,
                    coordinate.source(),
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
        continuation(checked, &bundle)
    })
}

fn lower_selected(
    inputs: &[SourceInput<'_>],
    modules: Option<&[crate::ModuleRecord]>,
    limits: CompilerLimits,
    overlap: crate::OverlapLowering,
    selection: &Selection<'_>,
    bundle: &SourceBundle,
    checked: CheckedProgram<'_, '_, '_>,
) -> Result<Reported, CompilationFailure> {
    let entry = checked.data.functions.iter().find(|function| {
        !function.formal_hypothesis
            && function.module == selection.module
            && function.name == selection.name
    });
    let selected = entry
        .map_or(selection.name, |function| function.symbol.as_str())
        .to_owned();
    let selected = selected.as_str();
    let launcher_contract_ready = entry.is_some_and(|main| main.requirements.is_empty());
    // This is a build caller, checked by the same pipeline as user-written
    // callers. No precondition fact is manufactured from native initialization.
    let mut caller_failure = None;
    if !launcher_contract_ready
        && let Some(function) = entry
        && let Some((name, source)) = launcher::caller_source(&checked, function)
    {
        let bundle_name = (0_u64..)
            .map(|index| format!("executable-caller-{index}.wf"))
            .find(|candidate| {
                !bundle
                    .files()
                    .iter()
                    .any(|file| file.logical_path().as_str() == candidate)
            })
            .expect("a finite bundle leaves a caller source name");
        let mut with_caller = inputs.to_vec();
        with_caller.push(
            SourceInput::new(&bundle_name, source.as_bytes())
                .in_module(selection.module, crate::SourceRole::Implementation),
        );
        let caller_selection = Selection {
            module: selection.module,
            name: &name,
            no_heap: selection.no_heap,
            public: false,
        };
        match compile_selected(&with_caller, modules, limits, overlap, &caller_selection) {
            Ok(reported) => return Ok(reported),
            Err(failure) if failure.kind() == CompilationFailureKind::Source => {
                caller_failure = Some(failure.to_string());
            }
            Err(failure) => return Err(failure),
        }
    }
    let permission_ledger = checked.data.permission_ledger.clone();
    let target =
        TargetLayout::host().map_err(|failure| CompilationFailure::lowering(failure.into()))?;
    // [MOD-9] a module program entry's build emits what that entry's run
    // reaches; the other entries' code is checked but is not this
    // executable's. A source bundle keeps every definition.
    let roots = modules.and(entry).map(|function| [function.id]);
    let ir = crate::lower_checked_from(
        checked,
        overlap,
        target,
        roots.as_ref().map(|roots| roots.as_slice()),
    )
    .map_err(CompilationFailure::lowering)?;
    // What this lowering did with each permission it was given, appended after
    // the judgment's own lines. The judgment reports the same verdicts with or
    // without `--par`; these lines report an actualization, which only a
    // compilation that asked for one has.
    let mut ledger: Vec<String> = permission_ledger
        .into_iter()
        .map(|line| line.text)
        .collect();
    ledger.extend_from_slice(ir.actualization_ledger());
    emit_llvm_with_layout(&ir, target)
        .and_then(|module| {
            // Emission decides which recursive components carry a runtime
            // budget after their clone families are known.
            let mut ledger = ledger;
            ledger.extend_from_slice(module.actualization_ledger());
            let launch = if launcher_contract_ready {
                launcher::render(&ir, selected)?
            } else {
                String::new()
            };
            Ok(Reported {
                module: module.into_string()
                    + &launch
                    + &caller_failure.map_or_else(String::new, |failure| {
                        format!(
                            "\n; Executable caller was not admitted: {}\n",
                            failure.replace(['\n', '\r'], " ")
                        )
                    }),
                ledger,
            })
        })
        .map_err(|failure: BackendFailure| {
            let (stage, kind) = match failure {
                BackendFailure::TargetLayout(_) => (
                    CompilationStage::TargetLayout,
                    CompilationFailureKind::TargetLayout,
                ),
                _ => (CompilationStage::Backend, CompilationFailureKind::Backend),
            };
            CompilationFailure::new(stage, kind, failure)
        })
}

#[cfg(test)]
mod tests {
    use super::{
        CompilationFailureKind, CompilationStage, CompilerLimits, check, compile,
        compile_with_permission_ledger,
    };
    use crate::{OverlapLowering, RecursionBudget, SourceInput};

    /// Places each record in the module its directory names, in the
    /// interface role when it is that directory's `module.wfm` [MOD-2].
    fn module_inputs<'a>(
        graph: &crate::ModuleGraph,
        records: &'a [(&'a str, &'a [u8])],
    ) -> Vec<SourceInput<'a>> {
        records
            .iter()
            .map(|(path, bytes)| {
                let (directory, file) = path.rsplit_once('/').unwrap_or(("", path));
                let components: Vec<&str> = if directory.is_empty() {
                    Vec::new()
                } else {
                    directory.split('/').collect()
                };
                let module = graph
                    .modules()
                    .iter()
                    .position(|module| module.path() == components.as_slice())
                    .and_then(crate::ModuleId::from_index)
                    .expect("every record lies in a registered module");
                let role = if file == "module.wfm" {
                    crate::SourceRole::Interface
                } else {
                    crate::SourceRole::Implementation
                };
                SourceInput::new(path, bytes).in_module(module, role)
            })
            .collect()
    }

    /// [FN-2, MOD-8] a rejection raised while checking a concrete instance
    /// stays at the template's source, in the module that owns it, and names
    /// the call in another module that requested the instance. Here the
    /// `Bool` instance's `ensures` has no fragment result datum.
    #[test]
    fn an_instance_failure_names_the_template_and_its_requesting_call() {
        let graph = crate::form_module_graph(
            SourceInput::new(
                "modules.wfg",
                b"pkg::lib: [];\npkg: [pkg::lib];\n\nentry app = pkg::main;\n",
            ),
            CompilerLimits::default(),
        )
        .expect("the graph forms");
        let records: [(&str, &[u8]); 4] = [
            (
                "lib/module.wfm",
                b"public fn same<T: copy>(value: T) -> result: T pure contract {\n  ensures result == value;\n} doc \"Returns its argument.\";\n",
            ),
            (
                "lib/same.wf",
                b"fn same<T: copy>(value: T) -> result: T pure contract {\n  ensures result == value;\n} {\n  return value;\n}\n",
            ),
            ("module.wfm", b"public fn main() -> status: ExitStatus pure;\n"),
            (
                "main.wf",
                b"fn main() -> status: ExitStatus pure {\n  let small = pkg::lib::same::<u8>(value: 3_u8);\n  let yes = True();\n  let flag = pkg::lib::same::<Bool>(value: yes);\n  return exit_status(code: small);\n}\n",
            ),
        ];
        let inputs = module_inputs(&graph, &records);
        let failure = crate::check_module_program(&graph, &inputs, CompilerLimits::default())
            .expect_err("the Bool instance's ensures has no fragment result datum");
        assert_eq!(failure.rule_id(), Some("FN-9"));
        let detail = failure.to_string();
        assert!(detail.contains(" at lib/same.wf:1:"), "{detail}");
        assert!(
            detail.contains("in the instance requested at main.wf:4:14"),
            "{detail}"
        );
    }

    /// [STOR-8, MOD-9] a no-heap entry's build emits its execution closure
    /// alone, so neither the other entry's allocating module nor an unused
    /// allocating helper of its own module leaves an allocator reference in
    /// its output, while the heap-using entry of the same graph keeps one.
    #[test]
    fn a_no_heap_entry_build_names_no_allocator_that_its_sibling_entry_uses() {
        let graph = crate::form_module_graph(
            SourceInput::new(
                "modules.wfg",
                b"pkg: [];\npkg::tools: [];\n\nentry kernel = pkg::start {\n  no_heap;\n}\n\nentry tool = pkg::tools::run;\n",
            ),
            CompilerLimits::default(),
        )
        .expect("the graph forms");
        let records: [(&str, &[u8]); 4] = [
            ("module.wfm", b"public fn start() -> status: ExitStatus pure;\n"),
            (
                "start.wf",
                b"fn spare() -> result: u8 pure {\n  let cell = box_new::<u8>(value: 1_u8);\n  return 0_u8;\n}\n\nfn start() -> status: ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
            ),
            ("tools/module.wfm", b"public fn run() -> status: ExitStatus pure;\n"),
            (
                "tools/run.wf",
                b"fn run() -> status: ExitStatus pure {\n  let cell = box_new::<u8>(value: 7_u8);\n  return exit_status(code: 0_u8);\n}\n",
            ),
        ];
        let inputs = module_inputs(&graph, &records);
        let build = |entry| {
            super::compile_module_program(
                &graph,
                &inputs,
                super::ModuleEntry::Named(entry),
                CompilerLimits::default(),
                OverlapLowering::Off,
            )
            .expect("the entry builds")
        };
        let kernel = build("kernel");
        assert!(!kernel.contains("@malloc"), "{kernel}");
        assert!(!kernel.contains("@free"), "{kernel}");
        assert!(!kernel.contains("spare"), "{kernel}");
        assert!(!kernel.contains("wf_tools.run"), "{kernel}");
        let tool = build("tool");
        assert!(tool.contains("call ptr @malloc"), "{tool}");
        assert!(tool.contains("@wf_tools.run("), "{tool}");
    }

    #[test]
    fn public_compilation_requires_a_source_record_before_adding_the_prelude() {
        for failure in [
            check(&[], CompilerLimits::default()).expect_err("no source record was supplied"),
            compile(&[], CompilerLimits::default()).expect_err("no source record was supplied"),
        ] {
            assert_eq!(failure.stage(), CompilationStage::SourceEnvelope);
            assert_eq!(failure.kind(), CompilationFailureKind::Invocation);
            assert_eq!(failure.rule_id(), None);
            assert_eq!(failure.detail(), "EmptySourceSequence");
        }

        let empty = check(
            &[SourceInput::new("empty.wf", b"")],
            CompilerLimits::default(),
        )
        .expect_err("a present record still needs its canonical final newline");
        assert_eq!(empty.stage(), CompilationStage::CanonicalSource);
        assert_eq!(empty.kind(), CompilationFailureKind::Source);
        assert_eq!(empty.rule_id(), Some("FORM-2"));

        check(
            &[SourceInput::new("empty.wf", b"\n")],
            CompilerLimits::default(),
        )
        .expect("a canonical source record may contain no declarations");
    }

    #[test]
    fn source_envelope_limits_are_resource_failures_in_both_public_projections() {
        let inputs = [SourceInput::new("main.wf", b"@")];
        let mut count_limits = CompilerLimits::default();
        count_limits.source.max_sources = 0;
        let mut byte_limits = CompilerLimits::default();
        byte_limits.source.max_source_bytes = 0;

        for (limits, detail) in [
            (
                count_limits,
                format!(
                    "LimitExceeded {{ limit: Sources, maximum: 0, actual: {} }}",
                    inputs.len() + crate::prelude::DECLARATIONS.len()
                ),
            ),
            (
                byte_limits,
                "LimitExceeded { limit: SourceBytes, maximum: 0, actual: 1 }".to_owned(),
            ),
        ] {
            for failure in [
                check(&inputs, limits).expect_err("the source envelope exceeds its ceiling"),
                compile(&inputs, limits).expect_err("the source envelope exceeds its ceiling"),
            ] {
                assert_eq!(failure.stage(), CompilationStage::SourceEnvelope);
                assert_eq!(failure.kind(), CompilationFailureKind::Resource);
                assert_eq!(failure.rule_id(), None);
                assert_eq!(failure.detail(), detail);
            }
        }
    }

    #[test]
    fn source_envelope_invalid_paths_remain_invocation_failures() {
        let invalid = [SourceInput::new("/main.wf", b"@")];
        let duplicate = [
            SourceInput::new("main.wf", b"@"),
            SourceInput::new("main.wf", b"@"),
        ];
        for (inputs, detail) in [
            (invalid.as_slice(), "LogicalPath(Absolute)"),
            (
                duplicate.as_slice(),
                "DuplicateLogicalPath { path: LogicalPath(\"main.wf\"), first_position: 0, duplicate_position: 1 }",
            ),
        ] {
            for failure in [
                check(inputs, CompilerLimits::default())
                    .expect_err("the source envelope is invalid"),
                compile(inputs, CompilerLimits::default())
                    .expect_err("the source envelope is invalid"),
            ] {
                assert_eq!(failure.stage(), CompilationStage::SourceEnvelope);
                assert_eq!(failure.kind(), CompilationFailureKind::Invocation);
                assert_eq!(failure.rule_id(), None);
                assert_eq!(failure.detail(), detail);
            }
        }
    }

    #[test]
    fn source_envelope_storage_and_representation_failures_preserve_their_details() {
        use crate::{LogicalPathError, SourceBundleError, SourceLimit};

        // Exercise allocator failure classification without exhausting the host.
        for error in [
            SourceBundleError::StorageUnavailable {
                limit: SourceLimit::Sources,
                requested: 1,
            },
            SourceBundleError::ArithmeticOverflow,
            SourceBundleError::LogicalPath(LogicalPathError::LengthOverflow),
            SourceBundleError::LogicalPath(LogicalPathError::StorageUnavailable { requested: 7 }),
        ] {
            let detail = format!("{error:?}");
            let failure = super::CompilationFailure::source_envelope(error);
            assert_eq!(failure.stage(), CompilationStage::SourceEnvelope);
            assert_eq!(failure.kind(), CompilationFailureKind::Resource);
            assert_eq!(failure.rule_id(), None);
            assert_eq!(failure.detail(), detail);
        }
    }

    #[test]
    fn the_executable_caller_proves_the_selected_functions_contract() {
        let source = b"fn main() -> result: unit pure contract {\n  requires 0_u64 <= 1_u64;\n} {\n  return unit;\n}\n";
        let llvm = compile(
            &[SourceInput::new("entry-contract.wf", source)],
            CompilerLimits::default(),
        )
        .expect("the ordinary generated caller proves a constant requirement");
        assert!(llvm.contains("@wf_executable_caller_0("));
        assert!(llvm.contains("define i32 @main("));
        assert!(!llvm.contains("Executable caller was not admitted"));
    }

    #[test]
    fn an_uninhabited_function_is_a_library_without_an_unproved_executable_call() {
        let source = b"fn main() -> result: unit pure contract {\n  requires 1_u64 <= 0_u64;\n} {\n  return unit;\n}\n";
        let llvm = compile(
            &[SourceInput::new("uninhabited-entry.wf", source)],
            CompilerLimits::default(),
        )
        .expect("an uninhabited ordinary declaration is accepted as a library");
        assert!(llvm.contains("@wf_main("));
        assert!(!llvm.contains("define i32 @main("));
        assert!(llvm.contains("Executable caller was not admitted"));
    }

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
        let source = br#"fn main() -> status: ExitStatus pure {
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
            detail.contains(r#"expected: [";", "{", ")", ",", "<", ">", "["#),
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
                // v0.60's [INV-1] admits `==` in an invariant target and
                // refuses `!=` in either position, which is the reverse of
                // the v0.59 row this fixture carried.
                b"fn main() -> status: ExitStatus pure {\n  invariant bad: 0_u64 != 0_u64;\n  return exit_status(code: 0_u8);\n}\n"
                    .as_slice(),
                CompilationStage::Semantics,
                "INV-1",
            ),
            (
                "local-target-unproved.wf",
                b"fn main() -> status: ExitStatus pure {\n  invariant bad: 1_u64 <= 0_u64;\n  return exit_status(code: 0_u8);\n}\n",
                CompilationStage::Semantics,
                "INV-1",
            ),
            (
                "use-relation-formation.wf",
                b"fn check(value: u64, limit: u64) -> result: unit pure {\n  invariant scaled: 2_u64 * value <= 2_u64 * limit {\n    use (value == limit);\n  }\n  return unit;\n}\n\nfn main() -> status: ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
                CompilationStage::Semantics,
                "PRF-1",
            ),
            (
                "use-relation-name.wf",
                b"fn check(value: u64, limit: u64) -> result: unit pure {\n  invariant scaled: 2_u64 * value <= 2_u64 * limit {\n    use (value <= missing);\n  }\n  return unit;\n}\n\nfn main() -> status: ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
                CompilationStage::Resolution,
                "PRF-1",
            ),
            (
                "named-use-scope.wf",
                b"fn check(value: u64, limit: u64) -> result: unit pure {\n  invariant scaled: 2_u64 * value <= 2_u64 * limit {\n    use missing;\n  }\n  return unit;\n}\n\nfn main() -> status: ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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
        let source = b"fn main() -> status: ExitStatus pure {\n  doc \"One double space where canonical form admits one space.\";\n  return exit_status(code:  0_u8);\n}\n";
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

    /// An ordinary reference parameter keeps its later call requirement.
    ///
    /// Retired subject: the child reborrow inside a `region { .. }` block,
    /// which ended a loan before the following ordinary statement. v0.60 has
    /// no regions and no reborrows [REF-1, REF-3]; the successor kept here is
    /// the second half this case always carried — a missing `open_file` range
    /// proof is still reported at the call and is still repaired by writing
    /// the requirement, not by adding a scope.
    #[test]
    fn a_reference_parameter_keeps_its_later_call_requirement() {
        let source = br#"fn walk(factory: &HandleFactory, root: &DirectoryRead, name: &[u8]) -> result: u8 reads(root), reads(name), writes(factory) {
  match open_file(factory: factory, root: root, name: name, start: 0_u64, end: 1_u64) {
    Ok(value: handle) => {
      close_read(factory: factory, file: move handle);
    }
    Err(error: problem) => {
    }
  }
  let later = 0_u8;
  return 0_u8;
}
"#;
        let failure = compile(
            &[SourceInput::new("walk.wf", source)],
            CompilerLimits::default(),
        )
        .expect_err("the unchanged source still lacks the file-name range proof");
        assert_eq!(failure.rule_id(), Some("FN-8"));
        assert!(
            failure.detail().contains("1_u64 <= deref(name).len"),
            "{}",
            failure.detail()
        );
        let bounded = std::str::from_utf8(source).unwrap().replace(
            "writes(factory) {",
            "writes(factory) contract {\n  requires 1_u64 <= deref(name).len;\n} {",
        );
        compile(
            &[SourceInput::new("bounded_walk.wf", bounded.as_bytes())],
            CompilerLimits::default(),
        )
        .expect("the required range, not an extra scope helper, completes the valid program");
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
        let affine = br#"nocopy struct Counts {
  lines: u64;
}

fn main() -> status: ExitStatus pure {
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
        let collision = br#"fn main() -> status: ExitStatus pure {
  let permit = 1_u64;
  if permit == 1_u64 {
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
        let source = "fn main() -> status: ExitStatus pure {\n  let x = \u{a3};\n  return exit_status(code: 0_u8);\n}\n";
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
        let source = br#"fn main() -> status: ExitStatus pure {
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    set total = total +wrap index;
  }
  return exit_status(code: 0_u8);
}
"#;
        let host = "/absolute/path/counted.wf";
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
  Branch(left: Box<BoxNode>, right: Box<BoxNode>, w: u64);
}

fn boxed_leaf(w: u64) -> result: Box<BoxNode> pure {
  let leaf = BoxNode::Leaf(w: w);
  return box_new::<BoxNode>(value: move leaf);
}

fn boxed_branch(left: Box<BoxNode>, right: Box<BoxNode>) -> result: Box<BoxNode> pure {
  let branch = BoxNode::Branch(left: move left, right: move right, w: 0_u64);
  return box_new::<BoxNode>(value: move branch);
}

";

    /// The ledger states an eligible pair and the chain it composes into, then
    /// says the same of a pair whose recursive closure carries an erased
    /// source proof. Proof syntax changes no runtime footprint and therefore
    /// produces no separate `not-actualizable` class.
    #[test]
    fn the_permission_ledger_reports_eligible_pairs_and_their_chains() {
        let eligible = format!(
            "{TREE_PRELUDE}fn fold(node: &Box<BoxNode>) -> result: u64 writes(node) {{
  match deref(node).inner {{
    Leaf(w: leaf_w) => {{
      return deref(leaf_w);
    }}
    Branch(left: l, right: r, w: slot) => {{
      let a = fold(node: l);
      let b = fold(node: r);
      let total = imax(a, b);
      set deref(slot) = total;
      return total;
    }}
  }}
}}

fn main() -> status: ExitStatus pure {{
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  let total = fold(node: &branch0);
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
            "{TREE_PRELUDE}fn scaled(values: Array<u64, 8>, index: u64) -> result: u64 pure {{
  let size = values.len;
  let bounded = iand(index, 7_u64);
  invariant index_in_range: bounded <= 7_u64;
  return values[bounded];
}}

fn bubble(node: &Box<BoxNode>) -> result: u64 writes(node) {{
  match deref(node).inner {{
    Leaf(w: leaf_w) => {{
      let w = deref(leaf_w);
      let values = array_filled::<u64, 8>(value: 1_u64);
      let touched = scaled(values: values, index: w);
      return w;
    }}
    Branch(left: l, right: r, w: slot) => {{
      let a = bubble(node: l);
      let b = bubble(node: r);
      let total = a +wrap b;
      set deref(slot) = total;
      return total;
    }}
  }}
}}

fn main() -> status: ExitStatus pure {{
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  let total = bubble(node: &branch0);
  if total == 7_u64 {{
  }} else {{
    return exit_status(code: 1_u8);
  }}
  return exit_status(code: 0_u8);
}}
"
        );
        let ledger = ledger_of("bubble.wf", proved.as_bytes());
        // `scaled`'s own body is reported first, in source order: its proof
        // statement joins the chain the two preceding `let`s form, and its
        // array construction and consuming call are the pair that follows.
        assert_eq!(
            ledger[0],
            "PAR chain       bubble.wf:17  run(a let statement, a let statement, \
             a proof statement)  3 members through line 19"
        );
        assert_eq!(
            ledger[1],
            "PAR permitted   bubble.wf:26  pair(a let statement, array_filled)  eligible"
        );
        assert_eq!(
            ledger[2],
            "PAR chain       bubble.wf:26  run(a let statement, array_filled)  \
             2 members through line 27"
        );
        assert_eq!(
            ledger[3],
            "PAR denied      bubble.wf:27  pair(array_filled, scaled)  condition 1: \
             the write of s1 overlaps the operand read of s2 at \
             let values = array_filled::<u64, 8>(value: 1_u64); vs values"
        );
        assert_eq!(
            ledger[4],
            "PAR denied      bubble.wf:28  pair(scaled, a return statement)  condition 2: \
             the exit edge of s2 may skip the statement written after it"
        );
        assert_eq!(
            ledger[5],
            "PAR permitted   bubble.wf:32  pair(bubble, bubble)  eligible"
        );
        assert_eq!(
            ledger[6],
            "PAR chain       bubble.wf:32  run(bubble, bubble)  2 members through line 33"
        );
        assert_eq!(
            ledger[7],
            "PAR denied      bubble.wf:33  pair(bubble, a let statement)  condition 1: \
             the write of s1 overlaps the operand read of s2 at \
             let b = bubble(node: r); vs let total = a +wrap b;"
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
            ledger[8],
            "PAR permitted   bubble.wf:42  pair(boxed_leaf, boxed_leaf)  eligible"
        );
        assert_eq!(
            ledger[9],
            "PAR chain       bubble.wf:42  run(boxed_leaf, boxed_leaf)  2 members through line 43"
        );
        assert_eq!(
            ledger[10],
            "PAR denied      bubble.wf:43  pair(boxed_leaf, boxed_branch)  condition 1: \
             the write of s1 overlaps the write of s2 at \
             let leaf1 = boxed_leaf(w: 4_u64); vs move leaf1"
        );
        assert_eq!(
            ledger[11],
            "PAR denied      bubble.wf:44  pair(boxed_branch, bubble)  condition 1: \
             the write of s1 overlaps the write of s2 at \
             let branch0 = boxed_branch(left: move leaf0, right: move leaf1); vs &branch0"
        );
        assert_eq!(ledger.len(), 12);
    }

    /// One denial line per numbered condition, each citing that condition and
    /// the source text that refused the overlap. A denial arriving under the
    /// wrong condition, or with an empty citation, fails here.
    #[test]
    fn the_permission_ledger_names_the_condition_that_refused_each_pair() {
        // Condition 1: two reference actuals resolve to one place, so the line
        // has to name both actuals as the writer wrote them. v0.60 has no
        // permission marker, so what the pair rule sees is two writes of one
        // storage rather than two exclusive loans, and the overlap is
        // reported under condition 1 [REF-1, EFF-1, PAR-1].
        let overlapping = b"fn bump(slot: &u64) -> result: u64 writes(slot) {
  let seen = deref(slot);
  set deref(slot) = 7_u64;
  return seen;
}

fn main() -> status: ExitStatus pure {
  let cell = 1_u64;
  let lo = bump(slot: &cell);
  let hi = bump(slot: &cell);
  let total = imax(lo, hi);
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("bump.wf", overlapping),
            vec![
                "PAR denied      bump.wf:8  pair(a let statement, bump)  condition 1: \
                 the write of s1 overlaps the write of s2 at let cell = 1_u64; vs &cell"
                    .to_owned(),
                "PAR denied      bump.wf:9  pair(bump, bump)  condition 1: \
                 the write of s1 overlaps the write of s2 at &cell vs &cell"
                    .to_owned(),
                "PAR denied      bump.wf:10  pair(bump, a let statement)  condition 1: \
                 the write of s1 overlaps the operand read of s2 at \
                 let hi = bump(slot: &cell); vs let total = imax(lo, hi);"
                    .to_owned(),
            ]
        );

        // Affine opaque values have empty release under PRE-1 and STOR-3.
        let capability_releases = b"fn release_read_file(file: OutputStream) -> result: unit pure {
  return unit;
}

fn release_pair(first: OutputStream, second: OutputStream) -> result: unit pure {
  let done_first = release_read_file(file: move first);
  let done_second = release_read_file(file: move second);
  return unit;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("row.wf", capability_releases),
            vec![
                "PAR permitted   row.wf:6  pair(release_read_file, release_read_file)  eligible".to_owned(),
                "PAR chain       row.wf:6  run(release_read_file, release_read_file)  2 members through line 7".to_owned(),
                "PAR denied      row.wf:7  pair(release_read_file, a return statement)  \
                 condition 2: the exit edge of s2 may skip the statement written after it"
                    .to_owned(),
            ]
        );

        // The `propagate` edge: a `propagate` is never a window member itself
        // [PAR-1], and the ledger now reports its Err edge under condition 2
        // — the edge condition — once for each adjacent pair it stands in
        // rather than once for the two ordinary calls it separates.
        let propagating = b"fn peek(slot: &u8) -> result: u64 reads(slot) {
  return cvt::<u8, u64>(deref(slot));
}

fn stamp(slot: &u8) -> result: u64 writes(slot) {
  set deref(slot) = 9_u8;
  return 1_u64;
}

fn probe(outcome: Result<u8, NarrowError>, a: &u8, b: &u8) -> result: Result<unit, NarrowError> reads(b), writes(a) {
  let seen = peek(slot: b);
  let narrowed = propagate outcome;
  let stamped = stamp(slot: a);
  return Ok<unit, NarrowError>(value: unit);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("propagate.wf", propagating),
            vec![
                "PAR denied      propagate.wf:11  pair(peek, a propagate statement)  \
                 condition 2: the Err edge of s2 may skip the statement written after it"
                    .to_owned(),
                "PAR denied      propagate.wf:12  pair(a propagate statement, stamp)  \
                 condition 2: the Err edge of s1 may skip the statement written after it"
                    .to_owned(),
                "PAR denied      propagate.wf:13  pair(stamp, a return statement)  \
                 condition 2: the exit edge of s2 may skip the statement written after it"
                    .to_owned(),
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
        let source = b"fn interesting(index: u64) -> result: Bool pure {
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

fn main() -> status: ExitStatus pure {
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
                "PAR chain       counting.wf:2  run(a let statement, a let statement)  \
                 2 members through line 3"
                    .to_owned(),
                "PAR loop        counting.wf:16  loop  permitted   eligible; \
                 one accumulator under +wrap"
                    .to_owned(),
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
        let source = b"fn main() -> status: ExitStatus pure {
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
                "PAR chain       folding.wf:2  run(a let statement, a let statement)  \
                 2 members through line 3"
                    .to_owned(),
                "PAR loop        folding.wf:4  loop  denied      condition 1: the loop writes \
                 storage outliving the iteration that no exactly associative operation reduces, \
                 at set total = fadd.strict(total, step);"
                    .to_owned()
            ]
        );

        // The identical loop over an integer accumulator is permitted, so the
        // refusal above is about the operation and not about the loop.
        let integral = b"fn main() -> status: ExitStatus pure {
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
                "PAR chain       folding.wf:2  run(a let statement, a let statement)  \
                 2 members through line 3"
                    .to_owned(),
                "PAR loop        folding.wf:4  loop  permitted   eligible; \
                 one accumulator under +wrap"
                    .to_owned(),
            ]
        );
    }

    /// A counted loop whose proved index is exactly its binder is reported as
    /// an eligible map with no accumulator.
    #[test]
    fn a_proven_counted_binder_buffer_map_is_permitted() {
        let source = b"fn main() -> status: ExitStatus pure {
  let values = array_filled::<u64, 64>(value: 0_u64);
  let out = slots_from_array::<u64, 64>(values: values);
  for @fill (i in 0_u64..64_u64) {
    set out[i] = i *wrap i;
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("mapping.wf", source),
            vec![
                "PAR denied      mapping.wf:2  pair(array_filled, slots_from_array)  \
                 condition 1: the write of s1 overlaps the operand read of s2 at \
                 let values = array_filled::<u64, 64>(value: 0_u64); vs values"
                    .to_owned(),
                "PAR denied      mapping.wf:3  pair(slots_from_array, a for loop)  \
                 condition 1: s2 is a for loop"
                    .to_owned(),
                "PAR loop        mapping.wf:4  loop  permitted   eligible; no accumulator"
                    .to_owned(),
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
        let source = b"fn accum(slot: &f64, x: f64) -> result: u64 writes(slot) {
  set deref(slot) = fadd.strict(deref(slot), x);
  let bits = reinterpret::<f64, u64>(deref(slot));
  return iand(bits, 1_u64);
}

fn main() -> status: ExitStatus pure {
  let total = 0.0_f64;
  let count = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    let one = accum(slot: &total, x: 0.5_f64);
    set count = count +wrap one;
  }
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("carrying.wf", source),
            vec![
                "PAR chain       carrying.wf:8  run(a let statement, a let statement)  \
                 2 members through line 9"
                    .to_owned(),
                "PAR loop        carrying.wf:10  loop  denied      condition 2: the body \
                 writes storage that is neither introduced by the iteration nor the \
                 accumulator, at &total"
                    .to_owned(),
                "PAR denied      carrying.wf:11  pair(accum, a set statement)  condition 1: \
                 the write of s1 overlaps the operand read of s2 at \
                 let one = accum(slot: &total, x: 0.5_f64); vs set count = count +wrap one;"
                    .to_owned(),
            ]
        );

        // The same loop over a callee that writes nothing is permitted, so the
        // refusal above is about the projected row and not about the shape.
        let reading = b"fn weigh(x: f64) -> result: u64 pure {
  let bits = reinterpret::<f64, u64>(x);
  return iand(bits, 1_u64);
}

fn main() -> status: ExitStatus pure {
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
                "PAR chain       carrying.wf:7  run(a let statement, a let statement)  \
                 2 members through line 8"
                    .to_owned(),
                "PAR loop        carrying.wf:9  loop  permitted   eligible; \
                 one accumulator under +wrap"
                    .to_owned(),
                "PAR denied      carrying.wf:10  pair(weigh, a set statement)  condition 1: \
                 the write of s1 overlaps the operand read of s2 at \
                 let one = weigh(x: total); vs set count = count +wrap one;"
                    .to_owned(),
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
        let source = b"fn scan_until(src: &Slots<u64, 64>, needle: u64) -> result: u64 reads(src) {
  let count = deref(src).len;
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

fn main() -> status: ExitStatus pure {
  let values = array_filled::<u64, 64>(value: 1_u64);
  let data = slots_from_array::<u64, 64>(values: values);
  set data[10_u64] = 7_u64;
  let t = scan_until(src: &data, needle: 7_u64);
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("giving.wf", source),
            vec![
                "PAR chain       giving.wf:2  run(a let statement, a let statement, \
                 a let statement)  3 members through line 4"
                    .to_owned(),
                "PAR loop        giving.wf:6  loop  denied      condition 4: a give leaves the loop"
                    .to_owned(),
                "PAR chain       giving.wf:8  run(a set statement, a let statement)  \
                 2 members through line 9"
                    .to_owned(),
                "PAR denied      giving.wf:22  pair(array_filled, slots_from_array)  \
                 condition 1: the write of s1 overlaps the operand read of s2 at \
                 let values = array_filled::<u64, 64>(value: 1_u64); vs values"
                    .to_owned(),
                "PAR denied      giving.wf:23  pair(slots_from_array, a set statement)  \
                 condition 1: the write of s1 overlaps the write of s2 at \
                 let data = slots_from_array::<u64, 64>(values: values); vs \
                 set data[10_u64] = 7_u64;"
                    .to_owned(),
                "PAR denied      giving.wf:24  pair(a set statement, scan_until)  \
                 condition 1: the write of s1 overlaps the read of s2 at \
                 set data[10_u64] = 7_u64; vs &data"
                    .to_owned(),
                "PAR denied      giving.wf:25  pair(scan_until, a return statement)  \
                 condition 2: the exit edge of s2 may skip the statement written after it"
                    .to_owned(),
            ]
        );

        // The same loop with the give removed is permitted, so the refusal is
        // about the exit edge and not about the shape.
        let contained =
            b"fn scan_until(src: &Slots<u64, 64>, needle: u64) -> result: u64 reads(src) {
  let count = deref(src).len;
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

fn main() -> status: ExitStatus pure {
  let values = array_filled::<u64, 64>(value: 1_u64);
  let data = slots_from_array::<u64, 64>(values: values);
  set data[10_u64] = 7_u64;
  let t = scan_until(src: &data, needle: 7_u64);
  return exit_status(code: 0_u8);
}
";
        assert_eq!(
            ledger_of("giving.wf", contained),
            vec![
                "PAR chain       giving.wf:2  run(a let statement, a let statement, \
                 a let statement)  3 members through line 4"
                    .to_owned(),
                "PAR loop        giving.wf:6  loop  permitted   eligible; \
                 one accumulator under +wrap"
                    .to_owned(),
                "PAR denied      giving.wf:18  pair(array_filled, slots_from_array)  \
                 condition 1: the write of s1 overlaps the operand read of s2 at \
                 let values = array_filled::<u64, 64>(value: 1_u64); vs values"
                    .to_owned(),
                "PAR denied      giving.wf:19  pair(slots_from_array, a set statement)  \
                 condition 1: the write of s1 overlaps the write of s2 at \
                 let data = slots_from_array::<u64, 64>(values: values); vs \
                 set data[10_u64] = 7_u64;"
                    .to_owned(),
                "PAR denied      giving.wf:20  pair(a set statement, scan_until)  \
                 condition 1: the write of s1 overlaps the read of s2 at \
                 set data[10_u64] = 7_u64; vs &data"
                    .to_owned(),
                "PAR denied      giving.wf:21  pair(scan_until, a return statement)  \
                 condition 2: the exit edge of s2 may skip the statement written after it"
                    .to_owned(),
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
        let source = b"fn main() -> status: ExitStatus pure {
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
                "PAR chain       booleans.wf:2  run(a let statement, a let statement, \
                 a let statement)  3 members through line 4"
                    .to_owned(),
                "PAR loop        booleans.wf:5  loop  denied      condition 1: the body carries \
                 3 accumulators, and this rule recombines one"
                    .to_owned(),
                "PAR hint        booleans.wf:5  loop  refused by condition 1; a recursive split \
                 over its index range would be eligible, combining under band, bor, bxor"
                    .to_owned(),
                "PAR chain       booleans.wf:8  run(a set statement, a set statement, \
                 a set statement)  3 members through line 10"
                    .to_owned(),
            ]
        );
    }

    /// Only ordinary counted-loop permission remains after C2 deletes PAR-3.
    #[test]
    fn a_counted_loop_reports_only_its_ordinary_permission() {
        let source = b"fn main() -> status: ExitStatus pure {
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
        let source =
            b"fn main() -> status: ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
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
    /// judgment lines of a program full of eligible pairs are the same with
    /// the option on and off, while only the `--par` module names the runtime.
    ///
    /// What `--par` adds to the report is what it actualized — which offers it
    /// kept, and what it did with each recursive component it found — and
    /// those lines say so in their own first word. A compilation that
    /// actualizes nothing has none of them.
    ///
    /// This is what makes the ledger usable on a shipped build. A developer
    /// reading what the compiler decided about a program is reading a property
    /// of the source, not of the compilation they happened to ask for.
    /// Every cyclic component a `--par` build finds is named in the ledger,
    /// with what was done to it or the member that stopped it.
    ///
    /// Plain `--par` is the first case below, because the budget it reports is
    /// the shipped default: the runtime's own answer. The three cases after it
    /// are the control writing that default out, pinning a starting value
    /// instead, and withholding the family altogether.
    ///
    /// The recursion budget is an actualization choice, so a reader has to be
    /// able to see which recursions got a family and which kept the ordinary
    /// path — an unspecialized recursion that said nothing would be
    /// indistinguishable from one the compiler never noticed. The exclusions
    /// are load-bearing in the same way: a splitter is cyclic, and it is the
    /// reason the three map kernels of the compute scoreboard cannot be
    /// touched by this at all.
    #[test]
    fn the_ledger_names_every_cyclic_component_and_what_the_budget_did_with_it() {
        let recursive = format!(
            "{TREE_PRELUDE}fn fold(node: &Box<BoxNode>) -> result: u64 writes(node) {{
  match deref(node).inner {{
    Leaf(w: leaf_w) => {{
      return deref(leaf_w);
    }}
    Branch(left: l, right: r, w: slot) => {{
      let a = fold(node: l);
      let b = fold(node: r);
      let total = imax(a, b);
      set deref(slot) = total;
      return total;
    }}
  }}
}}

fn main() -> status: ExitStatus pure {{
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  let total = fold(node: &branch0);
  return exit_status(code: 0_u8);
}}
"
        );
        let budgeted = |budget| OverlapLowering::OnWithRecursionBudget {
            budget,
            maximum_scalar_leaf_operations: None,
            sequential_refusal: false,
        };
        let lines = |name: &str, source: &[u8], overlap| {
            compile_with_permission_ledger(
                &[SourceInput::new(name, source)],
                CompilerLimits::default(),
                overlap,
            )
            .expect("the fixture must compile")
            .1
            .into_iter()
            .filter(|line| line.starts_with("PAR frontier"))
            .collect::<Vec<_>>()
        };
        for (overlap, summary, component) in [
            // Plain `--par` first: what the shipped default reports. The three
            // controls after it are the same mechanism written out.
            (
                OverlapLowering::On,
                "recursion budget runtime-derived  family emitted for 1 of 1 cyclic components",
                "component(fold)  budget-carrying clone family, entered with recursion budget \
                 runtime-derived",
            ),
            (
                budgeted(RecursionBudget::RuntimeDerived),
                "recursion budget runtime-derived  family emitted for 1 of 1 cyclic components",
                "component(fold)  budget-carrying clone family, entered with recursion budget \
                 runtime-derived",
            ),
            (
                budgeted(RecursionBudget::Pinned(
                    std::num::NonZeroU8::new(8).unwrap(),
                )),
                "recursion budget pinned 8  family emitted for 1 of 1 cyclic components",
                "component(fold)  budget-carrying clone family, entered with recursion budget \
                 pinned 8",
            ),
            (
                budgeted(RecursionBudget::Off),
                "recursion budget off  family emitted for 0 of 1 cyclic components",
                "component(fold)  no family: recursion budget off",
            ),
        ] {
            assert_eq!(
                lines("fold.wf", recursive.as_bytes(), overlap),
                vec![
                    format!("PAR frontier    {summary}"),
                    format!("PAR frontier    {component}"),
                ]
            );
        }
        // A build that actualizes no compute at all reports none of this.
        assert!(
            lines("fold.wf", recursive.as_bytes(), OverlapLowering::Off).is_empty(),
            "a default build has no actualization to report"
        );

        // A splitter calls itself to halve its range, so it is a cyclic
        // component of its own — and a synthesized one, which is why a kernel
        // that reaches the runtime through a split cannot get a family.
        let counted = b"fn interesting(index: u64) -> result: Bool pure {
  let low = iand(index, 7_u64);
  return low == 3_u64;
}

fn main() -> status: ExitStatus pure {
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
        let split = lines(
            "counted.wf",
            counted,
            budgeted(RecursionBudget::RuntimeDerived),
        );
        assert_eq!(
            split.len(),
            2,
            "the split fixture must have exactly one cyclic component: {split:?}"
        );
        assert!(
            split[0].contains("family emitted for 0 of 1 cyclic components"),
            "{split:?}"
        );
        assert!(
            split[1].contains("is a synthesized loop function"),
            "{split:?}"
        );
    }

    #[test]
    fn the_permission_ledger_does_not_depend_on_whether_the_lowering_is_taken() {
        let source = format!(
            "{TREE_PRELUDE}fn fold(node: &Box<BoxNode>) -> result: u64 writes(node) {{
  match deref(node).inner {{
    Leaf(w: leaf_w) => {{
      return deref(leaf_w);
    }}
    Branch(left: l, right: r, w: slot) => {{
      let a = fold(node: l);
      let b = fold(node: r);
      let total = imax(a, b);
      set deref(slot) = total;
      return total;
    }}
  }}
}}

fn main() -> status: ExitStatus pure {{
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  let total = fold(node: &branch0);
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

        let (judgment, actualization) = loud_ledger.split_at(quiet_ledger.len());
        assert_eq!(quiet_ledger, judgment);
        assert!(
            actualization
                .iter()
                .all(|line| line.starts_with("PAR actualization")
                    || line.starts_with("PAR frontier")),
            "only actualization lines may differ between the two lowerings: {actualization:?}"
        );
        assert!(
            actualization
                .iter()
                .any(|line| line.contains("component(fold)")),
            "the fixture's recursive fold must be named with what it got: {actualization:?}"
        );
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
    fn driver_erases_empty_formal_and_actual_groups_before_lowering() {
        let source = b"interface Empty {\n}\n\nbinding Selected : Empty {\n}\n\nfn main() -> status: ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
        let llvm = compile(
            &[SourceInput::new("value.wf", source)],
            CompilerLimits::default(),
        )
        .expect("group expansion must use the ordinary lowering path");
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
                b"// nope\nfn probe() -> result: unit pure {\n  return unit;\n}\n".as_slice(),
                CompilationStage::Lexing,
                "FORM-4",
            ),
            (
                "tab.wf",
                b"fn probe() -> result: unit pure {\n\treturn unit;\n}\n",
                CompilationStage::Lexing,
                "FORM-2",
            ),
            // The v0.59 row here lexed `'Bad` as a malformed REGIONID. v0.60
            // has no REGIONID at all, so `'` is an ordinary non-source byte
            // and that row's subject left the language with regions. The
            // successor is the other lexical name shape FORM-3 owns: a LABEL
            // whose `@` is not followed by the IDENT shape.
            (
                "sigil.wf",
                b"fn probe() -> result: unit pure {\n  loop @Bad {\n    break @Bad;\n  }\n  return unit;\n}\n",
                CompilationStage::Lexing,
                "FORM-3",
            ),
            (
                "dollar.wf",
                b"$\nfn probe() -> result: unit pure {\n  return unit;\n}\n",
                CompilationStage::Lexing,
                "FORM-1",
            ),
            (
                "string.wf",
                b"fn probe() -> result: unit pure {\n  let text: str = \"bad\\t\";\n  return unit;\n}\n",
                CompilationStage::Lexing,
                "FORM-5",
            ),
            (
                "numeric.wf",
                b"fn probe() -> result: unit pure {\n  let value: i32 = 1e+;\n  return unit;\n}\n",
                CompilationStage::TerminalClassification,
                "FORM-5",
            ),
            (
                "construct.wf",
                b"nope value;\n\nfn probe() -> result: unit pure {\n  return unit;\n}\n",
                CompilationStage::Parsing,
                "FORM-1",
            ),
            (
                "spacing.wf",
                b"fn  main() -> result: unit pure {\n  return unit;\n}\n",
                CompilationStage::CanonicalSource,
                "FORM-2",
            ),
            (
                // The v0.22 row wrote an undeclared region on a borrow and
                // reached [OWN-3] in the resolver. v0.60 has no region
                // spelling and no [OWN-3], so the reference's own unresolved
                // root is what this stage still owns: a reference expression
                // whose place base names nothing is a resolver rejection.
                "reference-root.wf",
                b"fn probe() -> result: unit pure {\n  let value = 0_i32;\n  let borrowed = &gone;\n  return unit;\n}\n",
                CompilationStage::Resolution,
                "TYPE-5",
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
        let source = b"fn main() -> status: ExitStatus pure {\n  let values = array_filled::<u8, 18446744073709551615>(value: 0_u8);\n  return exit_status(code: 0_u8);\n}\n";
        check(
            &[SourceInput::new("value.wf", source)],
            CompilerLimits::default(),
        )
        .expect("the array is source-valid before selected-target layout");
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
    fn a_loop_frame_outside_the_selected_address_domain_stays_a_target_failure() {
        use crate::backend::target::{TargetLayout, TargetLayoutFailure, TargetObject};

        let source = br#"fn folded(values: Array<u8, 216>) -> result: u64 pure {
  let total = 0_u64;
  for (i in 0_u64..2_u64) {
    let copied = values;
    let byte = copied[0_u64];
    let word = cvt::<u8, u64>(byte);
    set total = total +wrap word;
  }
  return total;
}

fn main() -> status: ExitStatus pure {
  let values = array_filled::<u8, 216>(value: 17_u8);
  let total = folded(values: values);
  return exit_status(code: 0_u8);
}
"#;
        let target = TargetLayout::host()
            .expect("supported test target")
            .with_address_index_max_for_test(255);
        let failure = super::with_checked_program(
            &[SourceInput::new("frame.wf", source)],
            None,
            CompilerLimits::default(),
            |checked, _| {
                crate::lower_checked_with_layout(checked, crate::OverlapLowering::On, target)
                    .map(|_| ())
                    .map_err(super::CompilationFailure::lowering)
            },
        )
        .expect_err("the complete 256-byte frame exceeds the selected address domain");
        assert_eq!(failure.stage(), CompilationStage::TargetLayout);
        assert_eq!(failure.kind(), CompilationFailureKind::TargetLayout);
        assert_eq!(failure.rule_id(), None);
        assert!(failure.detail().contains("ParallelLaneFrame"));
        assert_eq!(
            crate::LoweringFailure::from(TargetLayoutFailure::Unrepresentable(
                TargetObject::ParallelLaneFrame
            )),
            crate::LoweringFailure::TargetLayout(TargetLayoutFailure::Unrepresentable(
                TargetObject::ParallelLaneFrame
            )),
        );
        assert_eq!(
            crate::LoweringFailure::from(TargetLayoutFailure::InvalidIr),
            crate::LoweringFailure::InvalidCheckedProgram,
            "malformed compiler data must not be published as a target-domain failure",
        );
    }

    #[test]
    fn u16_buffer_whose_proved_count_exceeds_the_target_byte_domain_is_a_target_failure() {
        let source = br#"fn bounded_count(n: u64) -> result: u64 pure contract {
  ensures result <= 5000000000000000000_u64;
} {
  if n <= 5000000000000000000_u64 {
    return n;
  } else {
    return 5000000000000000000_u64;
  }
}

fn make(n: u64) -> result: Box<Array<u16>> pure {
  let bounded = bounded_count(n: n);
  return box_array_filled::<u16>(count: bounded, value: 0_u16);
}

fn main() -> status: ExitStatus pure {
  let values = make(n: 4_u64);
  return exit_status(code: 0_u8);
}
"#;
        check(
            &[SourceInput::new("value.wf", source)],
            CompilerLimits::default(),
        )
        .expect("the OP-9 proof is accepted before selected-target qualification");
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
        let source = b"fn main() -> status: ExitStatus pure {\n  let left = array_filled::<u8, 4611686018427387904>(value: 0_u8);\n  let right = array_filled::<u8, 4611686018427387904>(value: 0_u8);\n  return exit_status(code: 0_u8);\n}\n";
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

    // C2 replaces the SYS/QUAL and FN-7 entry assertions with PRE-1 ordinary
    // declarations and an optional build caller; unit results are ordinary.
    #[test]
    fn prelude_functions_and_unit_results_use_the_normal_call_path() {
        for source in [
            b"fn main() -> status: ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n"
                .as_slice(),
            b"fn main() -> result: unit pure {\n  return unit;\n}\n",
        ] {
            let llvm = compile(
                &[SourceInput::new("entry.wf", source)],
                CompilerLimits::default(),
            )
            .expect("an ordinary callable signature can be selected by the build caller");
            assert!(llvm.contains("define i32 @main(i32 %argc, ptr %argv)"));
        }
        // Which rule owns each of these two swapped in v0.60: [EFF-1] now
        // admits a row entry only over a reference parameter, so a row naming
        // an `own` parameter is that rule's own rejection, and the declared
        // write through a reference the body never writes is [EFF-2]'s
        // exactness. Both are still rejections of the same two sources.
        for (source, rule) in [
            (
                b"fn probe(args: Args) -> result: unit reads(args) {\n  return unit;\n}\n"
                    .as_slice(),
                "EFF-1",
            ),
            (
                b"fn probe(file: &ReadFile) -> result: unit writes(file) {\n  return unit;\n}\n",
                "EFF-2",
            ),
        ] {
            let failure = compile(
                &[SourceInput::new("rejected.wf", source)],
                CompilerLimits::default(),
            )
            .expect_err("ordinary parameter rows retain their exactness and mode checks");
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
            br#"fn double(value: u64) -> out: u64 pure {
  return value +wrap value;
}

fn helper(value: u64) -> out: u64 pure {
  let a = double(value: double(value: value));
  return a;
}

fn main() -> status: ExitStatus pure {
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
            br#"fn count(data: &[u8], start: u64, end: u64) -> lines: u64 reads(data) contract {
  requires imax(start, imin(start, end)) <= end;
} {
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
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
                br#"fn count(data: &[u8], start: u64, end: u64) -> lines: u64 pure contract {
  define spare = deref(data).len;
  requires end <= spare;
} {
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
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
    /// Retired subject: the v0.59 defect this case showed was `writes(cwd),
    /// writes(out)` — two occurrences of one category, which that version's
    /// row forbade. v0.60 admits a category more than once in one row, so
    /// that source is no longer a defect and the sentence it published no
    /// longer exists. The successor pinned here is the condition the rule
    /// does still state: a row lists each path at most once per category.
    #[test]
    fn an_effect_row_defect_names_its_condition_and_the_row_that_repairs_it() {
        let detail = rejection(
            "row.wf",
            br#"fn probe(cwd: &u64, out: &u64) -> status: ExitStatus writes(cwd), writes(cwd), writes(out) {
  set deref(cwd) = 1_u64;
  set deref(out) = 2_u64;
  return exit_status(code: 0_u8);
}
"#,
        );
        assert!(detail.contains("[EFF-1]"), "{detail}");
        assert!(
            detail.contains(
                r#"reason: "a row lists each path at most once per category, and this entry repeats one""#
            ),
            "{detail}"
        );
        assert!(
            detail.contains(
                r#"mechanical_fix: "delete the repeated entry; `writes(p)` already subsumes `reads(p)`, so the pair is never written for one path""#
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
            br#"fn count(data: &[u8]) -> lines: u64 reads(data) {
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
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
            br#"fn main() -> status: ExitStatus pure {
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
            br#"fn helper(value: u8) -> out: Result<u8, unit> pure {
  return Ok(value: value);
}

fn main() -> status: ExitStatus pure {
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
                br#"fn helper(value: u8) -> out: Result<u8, unit> pure {
  return Ok<u8, unit>(value: value);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
            )],
            CompilerLimits::default(),
        )
        .expect("the constructor spelling TYPE-5 names must be accepted");
    }

    /// A reference is never a result, and the grammar says so at the result
    /// type.
    ///
    /// Retired subject: [OWN-10]'s `InvalidBorrowLifetime` sentence, which
    /// named a region, the binder whose storage it outlived, and the
    /// `region 'r { .. }` repair. v0.60 has no regions, no region parameters
    /// and no lifetimes, so that rule, that sentence and that repair all left
    /// the language. The successor pinned here is [REF-3]'s own refusal,
    /// which the grammar reaches first: `type` has no reference production
    /// [GRAM-3], so a written reference result stops at the result type with
    /// the spelling the position does admit, including a qualified type's
    /// leading module alias or `pkg` [GRAM-3, MOD-3].
    #[test]
    fn a_reference_result_is_refused_at_the_result_type() {
        let detail = rejection(
            "reference-result.wf",
            br#"fn caller(anchor: &u64) -> out: &u64 pure {
  return anchor;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        );
        assert!(detail.contains("[GRAM-3]"), "{detail}");
        assert!(
            detail.contains(
                r#"expected: ["IDENT", "TYPEID", "pkg", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "unit"]"#
            ),
            "{detail}"
        );
        assert!(
            detail.contains(
                r#"at reference-result.wf:1:33 in line "fn caller(anchor: &u64) -> out: &u64 pure {""#
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
            b"fn helper(value: u64) -> out: u64 pure {\n  let a = value +wrap 1_u64;\n    let b = a +wrap 2_u64;\n  return b;\n}\n\nfn main() -> status: ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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
            b"fn helper(value: u64) -> out: u64 pure {\n  let a = value +wrap 1_u64;\n  let b = a  +wrap 2_u64;\n  return b;\n}\n\nfn main() -> status: ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        );
        assert!(
            inline.contains(r#"at spacing.wf:3:12 in line "  let b = a  +wrap 2_u64;""#),
            "{inline}"
        );
    }
}
