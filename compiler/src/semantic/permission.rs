//! The permission judgment [PAR-1]: whether two adjacent statements of one
//! block may be executed with their evaluations overlapped, and which runs of
//! adjacent statements may all overlap.
//!
//! P is a compiler-internal legality judgment. It refuses nothing, changes no
//! acceptance, and grants no lowering by itself; it records, per analyzed
//! block, which adjacent pairs may overlap and which runs the checked program
//! hands the backend as overlap groups.
//!
//! # The judgment
//!
//! [PAR-1] admits an overlap of two *adjacent* statements "exactly when the
//! first's write paths are disjoint from the second's read and write paths
//! and the second's write paths are disjoint from the first's, using the same
//! path-overlap and index/range-disjointness judgment as [EFF-5] and
//! [OWN-7]". Read/read overlap is admitted. There is no window of interposed
//! statements: two calls separated by a third statement are not adjacent, and
//! the run rule below is what lets all three overlap together.
//!
//! One statement's footprint has three halves:
//!
//! - **writes** — the places a call's declared `writes` row reaches through
//!   its actuals after [EFF-5] substitution, the place every by-value
//!   consumption empties ("a by-value consumption counts as a write of the
//!   argument's place"), the place a `set` names, and the binding a `let`
//!   defines ("a `let`'s defined binding is a write path"). That last clause
//!   is what makes ordinary dataflow a footprint conflict rather than a
//!   condition of its own: where the second statement reads what the first
//!   defines, the first's write path and the second's read path are one
//!   place.
//! - **reads** — the places a call's declared `reads` row reaches through the
//!   same substitution.
//! - **operand reads** — the storage the statement's own argument
//!   expressions touch on the calling thread. "Evaluating a statement's own
//!   argument expressions is part of that statement, so each statement's
//!   write paths must also be disjoint from the places the other statement's
//!   argument expressions read." Both directions are required, because which
//!   statement's operand evaluation an overlap moves is the implementation's
//!   choice of lane.
//!
//! Allocation and release contribute no path [STOR-8], so an allocating call
//! adds nothing here and never denies an adjacency on that ground.
//!
//! A footprint element whose caller place this analysis does not resolve
//! overlaps every place and denies permission, and a statement form whose
//! footprint this analysis does not compute denies for the same reason: a
//! missing element would *widen* permission, which is the one direction the
//! judgment must never fail in.
//!
//! # Runs
//!
//! "Permission composes: any run of adjacent statements that pairwise may
//! overlap may all overlap, and 'pairwise' means every ordered pair in the
//! run." The runs are computed over a running union footprint, as
//! `compiler/checker-facts` decides: a statement joins the current run when
//! its write paths miss the union's read and write paths and its read paths
//! miss the union's write paths, and otherwise starts a new run. Disjointness
//! from a union is the conjunction of disjointness from each member, so the
//! running test accepts exactly the runs the rule's every-ordered-pair
//! condition accepts, at a cost linear in the statements of the block. A
//! greedy partition can lose an opportunity and can never change a verdict,
//! because permission is never an obligation.
//!
//! A run's members that are not calls carry no hand-out and leave the
//! parallel lowering's clone set untouched, as
//! `compiler/parallel-lowering/two-worlds` decides: only a handed-out call
//! has a lowering that differs between the two worlds, so a permitted
//! adjacency among ordinary statements is scheduling and alias metadata
//! rather than a new hand-out site.
//!
//! # Exit edges
//!
//! [PAR-1] v0.60 states no condition about exit edges, where v0.59 required
//! that "every normal continuation of s1 reaches s2". A statement whose
//! continuation may leave the block cannot be overlapped with the statement
//! written after it, because that statement may not execute at all and the
//! rule still promises that "bindings and every Whitefoot state place equal
//! the source-order result". This judgment therefore refuses an exit-bearing
//! statement, which is the fail-closed reading of a sentence the rule no
//! longer carries.
//!
//! # Proof statements
//!
//! Every source proof statement has already been checked against its
//! control-flow facts before permission metadata is built, and is erased
//! before lowering: it has no runtime evaluation, effect, exit edge, or
//! scheduler-visible event. A failed proof rejects the program instead of
//! creating a runtime fallback. The judgment therefore neither rechecks
//! proofs nor models a proof failure path, and a proof statement carries an
//! empty footprint.
//!
//! **Invariant.** This judgment consults typing, declared effect rows,
//! resolved places [REF-1, OWN-7], and statement-graph exit edges. It cannot
//! turn an accepted program into a rejected one or move a required check.

use super::loop_permission::LoopPermission;
use super::model::{
    BindingId, CheckedArrayRoot, CheckedEffects, CheckedExpression, CheckedFunction, CheckedMode,
    CheckedPlaceStep, CheckedSetTarget, CheckedStatePath, CheckedStatement, FunctionId,
    expression_children,
};
use super::places::{
    CapturedRange, CapturedValue, PlaceMap, PlaceRoot, PlaceStep, ResolvedPlace,
    UnprovedSeparations,
};
use crate::NodePath;

/// The declared effect row of one concrete function, as P reads it. This is
/// the callable boundary only: no body fact enters.
#[derive(Clone, Debug, Default)]
pub(crate) struct PermissionSignature {
    pub(crate) reads: Vec<CheckedStatePath>,
    pub(crate) writes: Vec<CheckedStatePath>,
}

/// Which statement of an analyzed adjacency a denial cites.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PairSide {
    First,
    Second,
}

/// One exit edge of a statement that does not reach the statement's ordinary
/// successor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExitKind {
    /// A `propagate` right-hand side's `Err` edge to the function-return
    /// sink [ERR-3].
    PropagateError,
    /// A `return`, `give`, or `break` edge, which leaves the enclosing block
    /// or function without reaching the next statement.
    BlockExit,
}

/// One footprint element: a resolved caller place and the source node that
/// cites it.
///
/// There is no second shape. An arena region was v0.59's one footprint
/// element with no place of its own; [STOR-8] gives allocation and release no
/// effect entry and [PAR-1] states that they "contribute no path", so the
/// whole allocation half of the footprint is gone rather than retargeted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Access {
    pub(crate) place: ResolvedPlace,
    /// The actual or statement node the element is cited at.
    pub(crate) argument: NodePath,
}

/// One half of a statement's [PAR-1] footprint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FootprintHalf {
    Write,
    Read,
    OperandRead,
}

impl FootprintHalf {
    const fn name(self) -> &'static str {
        match self {
            Self::Write => "write",
            Self::Read => "read",
            Self::OperandRead => "operand read",
        }
    }
}

/// Which two footprint halves a conflict joins, named from the earlier and
/// the later statement, so a denial states the access it actually found.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ConflictKind {
    pub(crate) earlier: FootprintHalf,
    pub(crate) later: FootprintHalf,
}

impl ConflictKind {
    const fn new(earlier: FootprintHalf, later: FootprintHalf) -> Self {
        Self { earlier, later }
    }

    /// The two footprint halves this conflict joins, earlier first, as the
    /// ledger names them.
    pub(crate) const fn halves(self) -> (&'static str, &'static str) {
        (self.earlier.name(), self.later.name())
    }
}

/// Why P does not hold for one analyzed adjacency.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Denial {
    /// Two accesses of the two footprints overlap under [OWN-7].
    Footprint {
        kind: ConflictKind,
        left: Access,
        right: Access,
        /// The two statements the accesses belong to, earlier first.
        sides: (PairSide, PairSide),
    },
    /// Fail-closed: a footprint element whose caller place this analysis does
    /// not resolve overlaps every place [PAR-1].
    UnresolvedFootprint { side: PairSide, argument: NodePath },
    /// Fail-closed: a statement form whose footprint this analysis does not
    /// compute. Such a statement would otherwise contribute nothing and widen
    /// permission.
    UnclassifiedForm {
        side: PairSide,
        /// The form, as the ledger names it to the writer.
        form: &'static str,
    },
    /// A statement whose continuation may leave the block, so the statement
    /// written after it need not execute at all.
    SkippingExit { side: PairSide, kind: ExitKind },
}

impl Denial {
    /// The judgment clause this denial cites. The permission ledger prints it
    /// and the judgment tests assert it; acceptance never reads it.
    pub(crate) const fn condition(&self) -> u8 {
        match self {
            Self::Footprint { .. }
            | Self::UnresolvedFootprint { .. }
            | Self::UnclassifiedForm { .. } => 1,
            Self::SkippingExit { .. } => 2,
        }
    }
}

/// The judgment's outcome for one analyzed adjacency.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PermissionVerdict {
    /// P holds, so the two statements may be overlapped.
    PermittedEligible,
    Denied(Denial),
}

impl PermissionVerdict {
    pub(crate) const fn is_eligible(&self) -> bool {
        matches!(self, Self::PermittedEligible)
    }

    /// The cited clause of a denial, or `None` for a permitted verdict.
    #[allow(dead_code)]
    pub(crate) const fn denied_condition(&self) -> Option<u8> {
        match self {
            Self::Denied(denial) => Some(denial.condition()),
            Self::PermittedEligible => None,
        }
    }
}

/// One analyzed statement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PermissionSite {
    /// The statement node.
    pub(crate) statement: NodePath,
    /// The binding the statement defines, or `None` where it defines none.
    pub(crate) binding: Option<BindingId>,
    /// The named-function call this statement's right-hand side is, where it
    /// is one. Only a call-rooted member adds a hand-out; an ordinary
    /// statement is a member of the overlap group and carries none
    /// [parallel-lowering/two-worlds].
    pub(crate) call: Option<NodePath>,
    /// The callee's name for a call member, or the statement's form for any
    /// other member. The ledger prints it.
    pub(crate) callee_name: String,
}

/// One ordered pair of adjacent statements and its verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PermissionPair {
    pub(crate) first: PermissionSite,
    pub(crate) second: PermissionSite,
    pub(crate) verdict: PermissionVerdict,
}

/// A maximal run of at least two adjacent statements that may all overlap.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PermissionRun {
    pub(crate) sites: Vec<PermissionSite>,
}

/// Every analyzed pair and run of one concrete function, in source order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FunctionPermissions {
    pub(crate) function: String,
    pub(crate) pairs: Vec<PermissionPair>,
    pub(crate) runs: Vec<PermissionRun>,
    /// The [PAR-2] verdict of every counted loop of this function, in source
    /// order.
    pub(crate) loops: Vec<LoopPermission>,
}

/// The whole-program permission table, dense by [`FunctionId`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PermissionMetadata {
    pub(crate) functions: Vec<FunctionPermissions>,
}

impl PermissionMetadata {
    /// The table of one concrete function by its dense identity.
    pub(crate) fn of(&self, function: FunctionId) -> Option<&FunctionPermissions> {
        self.functions.get(function.0 as usize)
    }

    /// The table of one concrete function by its source name. Ledger and test
    /// convenience; the dense index is the identity.
    #[allow(dead_code)]
    pub(crate) fn named(&self, name: &str) -> Option<&FunctionPermissions> {
        self.functions
            .iter()
            .find(|entry| entry.function.as_str() == name)
    }
}

/// Runs P over every concrete function of one checked program.
pub(crate) fn analyze_permission(
    functions: &[CheckedFunction],
    signatures: &[PermissionSignature],
) -> PermissionMetadata {
    let program = Program {
        functions,
        signatures,
    };
    PermissionMetadata {
        functions: functions
            .iter()
            .map(|function| program.analyze_function(function))
            .collect(),
    }
}

pub(super) struct Program<'check> {
    functions: &'check [CheckedFunction],
    signatures: &'check [PermissionSignature],
}

/// One call occurrence, reduced to what the [EFF-5] substitution reads.
pub(super) struct CallProjection<'check> {
    pub(super) formal_effects: Option<&'check CheckedEffects>,
    pub(super) call: &'check NodePath,
    pub(super) target: FunctionId,
    pub(super) arguments: &'check [CheckedExpression],
    pub(super) argument_nodes: &'check [NodePath],
}

/// The call one expression is, or `None` for every other expression form.
pub(super) fn call_projection(value: &CheckedExpression) -> Option<CallProjection<'_>> {
    match value {
        CheckedExpression::UserCall {
            function,
            call,
            argument_nodes,
            arguments,
            formal_effects,
            ..
        } => Some(CallProjection {
            formal_effects: formal_effects.as_deref(),
            call,
            target: *function,
            arguments,
            argument_nodes,
        }),
        _ => None,
    }
}

/// One statement of a block, classified once for every adjacency it takes
/// part in.
///
/// `site` is absent for a statement the checked model gives no node of its
/// own — an expression statement, a `match`, a `loop`, a `break`. Every such
/// form is refused below, so an absent site is never a run member and never a
/// reported pair member; it still ends the run it interrupts.
struct Classified {
    site: Option<PermissionSite>,
    footprint: Result<Footprint, Refusal>,
}

/// Why one statement cannot take part in an overlap as written.
#[derive(Clone, Copy)]
enum Refusal {
    /// It carries an exit edge.
    Exit(ExitKind),
    /// Its footprint is not computed for this form.
    Form(&'static str),
}

impl<'check> Program<'check> {
    fn analyze_function(&self, function: &'check CheckedFunction) -> FunctionPermissions {
        let places = PlaceMap::for_function(function);
        let mut permissions = FunctionPermissions {
            function: function.name.clone(),
            pairs: Vec::new(),
            runs: Vec::new(),
            loops: Vec::new(),
        };
        let mut blocks = vec![function.body.as_deref().unwrap_or_default()];
        while let Some(block) = blocks.pop() {
            self.analyze_block(&places, block, &mut permissions);
            for statement in block {
                push_nested_blocks(statement, &mut blocks);
            }
        }
        permissions.pairs.sort_by(|left, right| {
            left.first
                .statement
                .components()
                .cmp(right.first.statement.components())
        });
        permissions.runs.sort_by(|left, right| {
            left.sites[0]
                .statement
                .components()
                .cmp(right.sites[0].statement.components())
        });
        // The loop judgment runs last and reads the finished verdicts, so a
        // loop that already holds an eligible adjacency is never additionally
        // told to become one. Its own verdict does not read them: [PAR-2] is
        // a judgment of the loop, not of what a writer could put inside it.
        let eligible = permissions
            .pairs
            .iter()
            .filter(|pair| pair.verdict.is_eligible())
            .map(|pair| pair.first.statement.clone())
            .collect::<Vec<_>>();
        permissions.loops =
            super::loop_permission::judge_loops(self, &places, function, &eligible);
        permissions
    }

    /// Judges every adjacent pair of one block and collects its runs.
    ///
    /// Every statement is classified and every footprint projected a single
    /// time per block, because a block of n statements is read n-1 times as
    /// an adjacency and once more by the run walk.
    fn analyze_block(
        &self,
        places: &PlaceMap,
        block: &'check [CheckedStatement],
        permissions: &mut FunctionPermissions,
    ) {
        if block.len() < 2 {
            return;
        }
        let classified = block
            .iter()
            .map(|statement| self.classify(places, statement))
            .collect::<Vec<_>>();
        for window in classified.windows(2) {
            let [first, second] = window else {
                continue;
            };
            // The ledger reports adjacencies a writer can act on. A pair of
            // two statements neither of which holds a call has no hand-out to
            // gain and would bury the lines that do, so it is judged for the
            // run walk below and not reported.
            let (Some(first_site), Some(second_site)) = (&first.site, &second.site) else {
                continue;
            };
            if first_site.call.is_none() && second_site.call.is_none() {
                continue;
            }
            permissions.pairs.push(PermissionPair {
                first: first_site.clone(),
                second: second_site.clone(),
                verdict: self.judge(places, first, second),
            });
        }
        self.collect_runs(places, &classified, permissions);
    }

    /// The verdict of one ordered adjacency.
    fn judge(&self, places: &PlaceMap, first: &Classified, second: &Classified) -> PermissionVerdict {
        for (side, classified) in [(PairSide::First, first), (PairSide::Second, second)] {
            match &classified.footprint {
                Err(Refusal::Exit(kind)) => {
                    return PermissionVerdict::Denied(Denial::SkippingExit {
                        side,
                        kind: *kind,
                    });
                }
                Err(Refusal::Form(form)) => {
                    return PermissionVerdict::Denied(Denial::UnclassifiedForm { side, form });
                }
                Ok(footprint) => {
                    if let Some(argument) = &footprint.unresolved {
                        return PermissionVerdict::Denied(Denial::UnresolvedFootprint {
                            side,
                            argument: argument.clone(),
                        });
                    }
                }
            }
        }
        let (left, right) = match (&first.footprint, &second.footprint) {
            (Ok(left), Ok(right)) => (left, right),
            // Every refusal already returned a denial above. This arm keeps
            // the match total without a panic in a judgment that is allowed
            // to deny and never allowed to fail.
            _ => {
                return PermissionVerdict::Denied(Denial::UnclassifiedForm {
                    side: PairSide::First,
                    form: "a statement form this judgment does not compute",
                });
            }
        };
        match footprint_conflict(places, left, right) {
            Some(denial) => PermissionVerdict::Denied(denial),
            None => PermissionVerdict::PermittedEligible,
        }
    }

    /// Grows maximal runs over a running union footprint [checker-facts].
    ///
    /// A statement joins the current run when its write paths miss the
    /// union's read and write paths and its read paths miss the union's write
    /// paths; otherwise it starts a new run. Disjointness from a union is the
    /// conjunction of disjointness from each member, so this accepts exactly
    /// the runs [PAR-1]'s every-ordered-pair condition accepts.
    fn collect_runs(
        &self,
        places: &PlaceMap,
        classified: &[Classified],
        permissions: &mut FunctionPermissions,
    ) {
        let mut run: Vec<usize> = Vec::new();
        let mut union = Footprint::default();
        let mut flush = |run: &mut Vec<usize>, union: &mut Footprint| {
            let sites = run
                .iter()
                .filter_map(|index| classified[*index].site.clone())
                .collect::<Vec<_>>();
            // Every form that is given a footprint carries a node of its own,
            // so the filter above removes nothing; the length is read after
            // it so that a form which later loses its node cannot shorten a
            // run behind the judgment's back.
            if sites.len() >= 2 && sites.len() == run.len() {
                permissions.runs.push(PermissionRun { sites });
            }
            run.clear();
            *union = Footprint::default();
        };
        for (index, statement) in classified.iter().enumerate() {
            let Ok(footprint) = &statement.footprint else {
                // An exit-bearing or unclassified statement joins no run and
                // ends the one before it: the statements after it are not
                // reached from the same straight-line edge.
                flush(&mut run, &mut union);
                continue;
            };
            if footprint.unresolved.is_some() {
                flush(&mut run, &mut union);
                continue;
            }
            if run.is_empty() {
                run.push(index);
                union.absorb(footprint);
                continue;
            }
            if footprint_conflict(places, &union, footprint).is_some() {
                flush(&mut run, &mut union);
                run.push(index);
                union.absorb(footprint);
            } else {
                run.push(index);
                union.absorb(footprint);
            }
        }
        flush(&mut run, &mut union);
    }

    /// One statement, reduced to what [PAR-1] judges, or the reason it cannot
    /// be.
    ///
    /// The match is exhaustive on purpose: a statement form this analysis did
    /// not classify would contribute an empty footprint and *widen*
    /// permission. Every form is either given a footprint here or refused
    /// here.
    fn classify(&self, places: &PlaceMap, statement: &'check CheckedStatement) -> Classified {
        let (node, binding, call, label, footprint) = match statement {
            // A source proof is checked before permission and erased before
            // lowering: no runtime evaluation, effect, exit edge, or
            // scheduler-visible event, so its footprint is empty and it joins
            // a run without changing it.
            CheckedStatement::Proof(proof) => (
                Some(&proof.node_path),
                None,
                None,
                "a proof statement",
                Ok(Footprint::default()),
            ),
            CheckedStatement::Let {
                node_path,
                binding,
                value,
            } => {
                let mut footprint = self.value_footprint(places, value, node_path);
                // "a `let`'s defined binding is a write path" [PAR-1]. This
                // is what makes reading what an earlier statement defines a
                // footprint conflict rather than a rule of its own.
                footprint.writes.push(Access {
                    place: ResolvedPlace::binding(*binding),
                    argument: node_path.clone(),
                });
                let projection = call_projection(value);
                let label = projection
                    .as_ref()
                    .map_or("a let statement", |_| "a call statement");
                let call = projection.map(|projection| projection.call.clone());
                (
                    Some(node_path),
                    Some(*binding),
                    call,
                    label,
                    Ok(footprint),
                )
            }
            CheckedStatement::Set {
                node_path,
                target,
                value,
            } => {
                let mut footprint = self.value_footprint(places, value, node_path);
                set_target_place(places, target, node_path, &mut footprint);
                (Some(node_path), None, None, "a set statement", Ok(footprint))
            }
            // Exit-bearing forms.
            CheckedStatement::PropagateLet { node_path, .. } => (
                Some(node_path),
                None,
                None,
                "a propagate statement",
                Err(Refusal::Exit(ExitKind::PropagateError)),
            ),
            CheckedStatement::Return { node_path, .. } => (
                Some(node_path),
                None,
                None,
                "a return statement",
                Err(Refusal::Exit(ExitKind::BlockExit)),
            ),
            CheckedStatement::Give { node_path, .. } => (
                Some(node_path),
                None,
                None,
                "a give statement",
                Err(Refusal::Exit(ExitKind::BlockExit)),
            ),
            CheckedStatement::Break { .. } => (
                None,
                None,
                None,
                "a break statement",
                Err(Refusal::Exit(ExitKind::BlockExit)),
            ),
            // Forms carrying their own control flow and their own drops. A
            // `match` statement's arms are statements this walk does not
            // fold into the statement's own footprint, so the statement is
            // refused rather than judged on its scrutinee alone. v0.59's
            // [PAR-1] carried a sentence putting a scrutinee call's arms
            // outside the judged statement; v0.60's does not, and the
            // fail-closed reading of its absence is this refusal.
            CheckedStatement::Match { .. } => {
                (None, None, None, "a match statement", Err(Refusal::Form("a match statement")))
            }
            CheckedStatement::ValueMatchLet { node_path, .. } => (
                Some(node_path),
                None,
                None,
                "a value match or value if",
                Err(Refusal::Form("a value match or value if")),
            ),
            CheckedStatement::Loop { .. } => {
                (None, None, None, "a loop", Err(Refusal::Form("a loop")))
            }
            CheckedStatement::CountedRange { node_path, .. } => (
                Some(node_path),
                None,
                None,
                "a for loop",
                Err(Refusal::Form("a for loop")),
            ),
            // [CALL-4] a binder list defines more than one place in one
            // statement, and this judgment describes one definition per
            // statement.
            CheckedStatement::DestructuringLet { node_path, .. } => (
                Some(node_path),
                None,
                None,
                "a statement that binds an ordered result list",
                Err(Refusal::Form("a statement that binds an ordered result list")),
            ),
            // An expression statement's reach is projected by no row, and a
            // discarded result carries its own [STOR-3] release walk.
            CheckedStatement::Evaluate(_) => (
                None,
                None,
                None,
                "an expression statement",
                Err(Refusal::Form("an expression statement")),
            ),
            CheckedStatement::DropExpression { .. } => (
                None,
                None,
                None,
                "a discarded expression statement",
                Err(Refusal::Form("a discarded expression statement")),
            ),
            CheckedStatement::Dispose { node_path, .. } => (
                Some(node_path),
                None,
                None,
                "a dispose statement",
                Err(Refusal::Form("a dispose statement")),
            ),
            // Forms whose source production v0.60 no longer has, and which
            // the checker no longer builds: `replace` [SET-2], the multi-
            // target commit [LIV-2], and the region block [STOR-2]. They are
            // refused rather than given a footprint, because the arm cannot
            // be reached and a footprint written for an unreachable shape
            // would be unverifiable. Their variants leave `CheckedStatement`
            // with the lowering that still matches on them.
            CheckedStatement::Replace { .. }
            | CheckedStatement::SetList { .. }
            | CheckedStatement::Region { .. } => (
                None,
                None,
                None,
                "a statement form this version no longer writes",
                Err(Refusal::Form("a statement form this version no longer writes")),
            ),
        };
        let callee_name = statement_value(statement)
            .and_then(call_projection)
            .and_then(|projection| {
                self.functions
                    .get(projection.target.0 as usize)
                    .map(|function| function.name.clone())
            })
            .unwrap_or_else(|| label.to_owned());
        Classified {
            site: node.cloned().map(|statement| PermissionSite {
                statement,
                binding,
                call,
                callee_name,
            }),
            footprint,
        }
    }

    /// The footprint of one statement's right-hand side: its own operand
    /// reads, the places its by-value consumptions empty, and, where the
    /// value is a call, that call's substituted row.
    fn value_footprint(
        &self,
        places: &PlaceMap,
        value: &CheckedExpression,
        node: &NodePath,
    ) -> Footprint {
        let mut footprint = Footprint::default();
        if let Some(projection) = call_projection(value) {
            self.call_footprint(places, &projection, &mut footprint);
            return footprint;
        }
        collect_consumed_places(places, value, node, &mut footprint);
        collect_operand_reads(places, value, node, &mut footprint);
        footprint
    }

    /// The written and read footprints of one call, by [EFF-5] substitution
    /// of the callee's declared row onto the actuals' resolved places.
    pub(super) fn footprint(&self, places: &PlaceMap, call: &CallProjection<'_>) -> Footprint {
        let mut footprint = Footprint::default();
        self.call_footprint(places, call, &mut footprint);
        footprint
    }

    fn call_footprint(
        &self,
        places: &PlaceMap,
        call: &CallProjection<'_>,
        footprint: &mut Footprint,
    ) {
        let (Some(signature), Some(callee)) = (
            self.signatures.get(call.target.0 as usize),
            self.functions.get(call.target.0 as usize),
        ) else {
            footprint.unresolved = Some(call.call.clone());
            return;
        };

        // "A by-value consumption counts as a write of the argument's place"
        // [PAR-1]. The affine discipline already forbids two consumers of one
        // place; the footprint states it rather than assuming it.
        for (index, parameter) in callee.parameters.iter().enumerate() {
            if !matches!(parameter.mode, CheckedMode::Own) {
                continue;
            }
            let Some(argument) = call.arguments.get(index) else {
                footprint.unresolved = Some(call.call.clone());
                return;
            };
            let node = call.argument_nodes.get(index).unwrap_or(call.call);
            if !consumes_root(argument) {
                continue;
            }
            match argument_places(places, argument) {
                Some(places) => footprint.writes.extend(places.into_iter().map(|place| Access {
                    place,
                    argument: node.clone(),
                })),
                None => footprint.unresolved = Some(node.clone()),
            }
        }

        let effects = call.formal_effects;
        let reads = effects.map_or(&signature.reads, |effects| &effects.reads);
        let writes = effects.map_or(&signature.writes, |effects| &effects.writes);
        for (written, declared) in [(false, reads), (true, writes)] {
            for path in declared {
                let Some(index) = callee
                    .parameters
                    .iter()
                    .position(|parameter| parameter.declaration == path.root)
                else {
                    footprint.unresolved = Some(call.call.clone());
                    continue;
                };
                let (Some(argument), Some(node)) = (
                    call.arguments.get(index),
                    call.argument_nodes.get(index),
                ) else {
                    footprint.unresolved = Some(call.call.clone());
                    continue;
                };
                let Some(roots) = argument_places(places, argument) else {
                    footprint.unresolved = Some(node.clone());
                    continue;
                };
                for mut place in roots {
                    place.path.extend(substituted_steps(path));
                    let access = Access {
                        place,
                        argument: node.clone(),
                    };
                    if written {
                        footprint.writes.push(access);
                    } else {
                        footprint.reads.push(access);
                    }
                }
            }
        }

        // The caller-side half: what this statement's own operand evaluation
        // touches before the call.
        for (index, argument) in call.arguments.iter().enumerate() {
            let node = call.argument_nodes.get(index).unwrap_or(call.call);
            collect_operand_reads(places, argument, node, footprint);
        }
    }
}

/// The statement's right-hand side, for the one classification that needs to
/// read the callee's name back out of it.
fn statement_value(statement: &CheckedStatement) -> Option<&CheckedExpression> {
    match statement {
        CheckedStatement::Let { value, .. } => Some(value),
        _ => None,
    }
}

/// The resolved steps of one declared [EFF-1] path below its substituted root.
///
/// An index or range position of a signature names a value parameter of the
/// same callable, and [EFF-5] replaces it with the value that parameter's own
/// argument supplies. This analysis does not hold the call's argument values,
/// so such a position becomes an unknown captured value, which no admitted
/// family separates [checker-facts]. That is conservative in the direction
/// permission must fail in: an unknown index overlaps every index.
fn substituted_steps(path: &CheckedStatePath) -> Vec<PlaceStep> {
    path.steps
        .iter()
        .map(|step| match step {
            super::model::CheckedEffectStep::Field(field) => PlaceStep::Field(*field),
            super::model::CheckedEffectStep::Deref => PlaceStep::Deref,
            super::model::CheckedEffectStep::Payload { variant, field } => PlaceStep::Payload {
                variant: *variant,
                field: *field,
            },
            super::model::CheckedEffectStep::Index(_) => {
                PlaceStep::Index(CapturedValue::unknown())
            }
            super::model::CheckedEffectStep::Range { .. } => PlaceStep::Range(CapturedRange {
                start: CapturedValue::unknown(),
                end: CapturedValue::unknown(),
            }),
            super::model::CheckedEffectStep::Part(part) => PlaceStep::Part(*part),
            super::model::CheckedEffectStep::Measure(measure) => PlaceStep::Measure(*measure),
        })
        .collect()
}

#[derive(Clone, Debug, Default)]
pub(super) struct Footprint {
    pub(super) writes: Vec<Access>,
    pub(super) reads: Vec<Access>,
    /// Storage this statement's own operand expressions read on the calling
    /// thread, before the call. [PAR-1] judges this half in both directions,
    /// because which statement's operand evaluation an overlap moves is the
    /// implementation's choice of lane.
    pub(super) operand_reads: Vec<Access>,
    /// Set where a footprint element's caller place is not resolved. Every
    /// such statement denies [PAR-1].
    pub(super) unresolved: Option<NodePath>,
}

impl Footprint {
    /// Adds one statement's halves to a running union [checker-facts].
    fn absorb(&mut self, other: &Self) {
        self.writes.extend(other.writes.iter().cloned());
        self.reads.extend(other.reads.iter().cloned());
        self.operand_reads
            .extend(other.operand_reads.iter().cloned());
    }

    /// Every place this footprint reads, in either half.
    fn read_halves(&self) -> impl Iterator<Item = (FootprintHalf, &Access)> {
        self.reads
            .iter()
            .map(|access| (FootprintHalf::Read, access))
            .chain(
                self.operand_reads
                    .iter()
                    .map(|access| (FootprintHalf::OperandRead, access)),
            )
    }
}

/// [PAR-1]'s disjointness clause over one ordered pair of footprints.
///
/// The earlier statement's writes are judged against every half of the later
/// one, and the later one's writes against every half of the earlier one.
/// Read/read overlap is admitted. The relation is [OWN-7]'s, asked of the
/// resolved paths through the function's overlap memo.
fn footprint_conflict(places: &PlaceMap, earlier: &Footprint, later: &Footprint) -> Option<Denial> {
    let oracle = UnprovedSeparations;
    for write in &earlier.writes {
        for (half, access) in later
            .writes
            .iter()
            .map(|access| (FootprintHalf::Write, access))
            .chain(later.read_halves())
        {
            if places.overlaps(&oracle, &write.place, &access.place) {
                return Some(Denial::Footprint {
                    kind: ConflictKind::new(FootprintHalf::Write, half),
                    left: write.clone(),
                    right: access.clone(),
                    sides: (PairSide::First, PairSide::Second),
                });
            }
        }
    }
    for write in &later.writes {
        for (half, access) in earlier.read_halves() {
            if places.overlaps(&oracle, &write.place, &access.place) {
                return Some(Denial::Footprint {
                    kind: ConflictKind::new(half, FootprintHalf::Write),
                    left: access.clone(),
                    right: write.clone(),
                    sides: (PairSide::First, PairSide::Second),
                });
            }
        }
    }
    None
}

/// Records the storage a `set` names, and the operands its subscripts read.
pub(super) fn set_target_place(
    places: &PlaceMap,
    target: &CheckedSetTarget,
    node: &NodePath,
    footprint: &mut Footprint,
) {
    let resolved = match target {
        CheckedSetTarget::Place(target) => {
            places.resolve(PlaceRoot::Binding(target.binding), &field_steps(&target.fields))
        }
        CheckedSetTarget::ArrayIndex(target) => {
            collect_operand_reads(places, &target.offset, node, footprint);
            let mut steps = field_steps(&target.fields);
            steps.push(PlaceStep::Index(CapturedValue::unknown()));
            places.resolve(PlaceRoot::Binding(target.binding), &steps)
        }
        CheckedSetTarget::BufferIndex(target) => {
            collect_operand_reads(places, &target.offset, node, footprint);
            let mut steps = target.root.place_path();
            steps.push(PlaceStep::Index(CapturedValue::unknown()));
            places.resolve(PlaceRoot::Binding(target.root.binding), &steps)
        }
        // [REF-4] a range reference names one path, so the element a
        // subscript through it writes is that path extended by the index.
        CheckedSetTarget::RangeIndex(target) => {
            collect_operand_reads(places, &target.offset, node, footprint);
            places.resolve(
                PlaceRoot::Binding(target.root.binding),
                &[PlaceStep::Index(CapturedValue::unknown())],
            )
        }
        CheckedSetTarget::Storage(target) => {
            for offset in target.offsets() {
                collect_operand_reads(places, offset, node, footprint);
            }
            places.resolve(target.root, &container_steps(target))
        }
    };
    footprint.writes.extend(resolved.into_iter().map(|place| Access {
        place,
        argument: node.clone(),
    }));
}

pub(super) fn field_steps(fields: &[u32]) -> Vec<PlaceStep> {
    fields.iter().copied().map(PlaceStep::Field).collect()
}

/// The resolved steps of one checked storage path.
pub(super) fn container_steps(root: &super::model::CheckedContainerRoot) -> Vec<PlaceStep> {
    root.path.iter().map(CheckedPlaceStep::place_step).collect()
}

/// Every caller place one expression transfers away by consuming an `own`
/// value, recorded as a write of the storage it empties [PAR-1].
pub(super) fn collect_consumed_places(
    places: &PlaceMap,
    expression: &CheckedExpression,
    node: &NodePath,
    footprint: &mut Footprint,
) {
    if consumes_root(expression)
        && let Some(resolved) = argument_places(places, expression)
    {
        footprint.writes.extend(resolved.into_iter().map(|place| Access {
            place,
            argument: node.clone(),
        }));
    }
    for child in expression_children(expression) {
        collect_consumed_places(places, child, node, footprint);
    }
}

/// Whether this expression consumes the place it names [OWN-1].
fn consumes_root(expression: &CheckedExpression) -> bool {
    matches!(
        expression,
        CheckedExpression::Binding {
            consume_root: true,
            ..
        } | CheckedExpression::Project {
            consume_root: true,
            ..
        }
    )
}

/// Every block nested inside one statement, for the whole-body walk.
fn push_nested_blocks<'check>(
    statement: &'check CheckedStatement,
    blocks: &mut Vec<&'check [CheckedStatement]>,
) {
    match statement {
        CheckedStatement::Match { arms, .. } | CheckedStatement::ValueMatchLet { arms, .. } => {
            for arm in arms {
                blocks.push(arm.body.as_slice());
            }
        }
        CheckedStatement::Loop { body, .. }
        | CheckedStatement::Region { body, .. }
        | CheckedStatement::CountedRange { body, .. } => blocks.push(body.as_slice()),
        CheckedStatement::Let { .. }
        | CheckedStatement::DestructuringLet { .. }
        | CheckedStatement::PropagateLet { .. }
        | CheckedStatement::Set { .. }
        | CheckedStatement::SetList { .. }
        | CheckedStatement::Replace { .. }
        | CheckedStatement::Proof(_)
        | CheckedStatement::Dispose { .. }
        | CheckedStatement::DropExpression { .. }
        | CheckedStatement::Evaluate(_)
        | CheckedStatement::Return { .. }
        | CheckedStatement::Give { .. }
        | CheckedStatement::Break { .. } => {}
    }
}

/// Every binding one expression tree mentions, for the counted judgment's
/// accumulator count.
pub(super) fn visit_read_bindings(
    expression: &CheckedExpression,
    note: &mut impl FnMut(BindingId),
) {
    match expression {
        CheckedExpression::Binding { binding, .. }
        | CheckedExpression::Project { binding, .. }
        | CheckedExpression::BorrowBox { binding, .. }
        | CheckedExpression::ReborrowAddressed { binding, .. }
        | CheckedExpression::DerefAddressed { binding, .. } => note(*binding),
        CheckedExpression::BorrowAddressed { root, .. }
        | CheckedExpression::ContainerMeasure { root, .. }
        | CheckedExpression::ReadStorage { root, .. } => {
            if let Some(binding) = root.binding() {
                note(binding);
            }
        }
        CheckedExpression::BorrowBuffer { root, .. }
        | CheckedExpression::BufferMeasure { root, .. }
        | CheckedExpression::BufferIndex { root, .. } => note(root.binding),
        CheckedExpression::RangeMeasure { root, .. }
        | CheckedExpression::RangeIndex { root, .. } => note(root.binding),
        CheckedExpression::ArrayMeasure { root, .. }
        | CheckedExpression::ArrayIndex { root, .. } => {
            if let CheckedArrayRoot::Binding { binding, .. } = root {
                note(*binding);
            }
        }
        _ => {}
    }
    for child in expression_children(expression) {
        visit_read_bindings(child, note);
    }
}

/// Every caller place one operand expression reads on the calling thread,
/// with an unresolved read failing closed.
///
/// This is the storage the *caller* touches while building an actual: a value
/// read out of a binding, a field, a `deref` [TYPE-7], a subscript. Forming a
/// reference names a path and reads no content beyond its own index and
/// endpoint atoms [REF-1, REF-4], so it contributes nothing here — the
/// callee's declared row already covers whatever it reaches through that
/// reference.
///
/// The match is exhaustive on purpose. A future expression form that reads
/// caller storage must be classified here rather than silently contributing
/// nothing, because a missing operand read widens permission.
fn collect_operand_reads(
    places: &PlaceMap,
    expression: &CheckedExpression,
    node: &NodePath,
    footprint: &mut Footprint,
) {
    let read = |footprint: &mut Footprint, resolved: Vec<ResolvedPlace>| {
        footprint
            .operand_reads
            .extend(resolved.into_iter().map(|place| Access {
                place,
                argument: node.clone(),
            }));
    };
    match expression {
        // Reads no caller storage of its own.
        CheckedExpression::Constant(_)
        | CheckedExpression::NamedConstant { .. }
        | CheckedExpression::IntegerOperation { .. }
        | CheckedExpression::FloatOperation { .. }
        | CheckedExpression::NumericConversion { .. }
        | CheckedExpression::Reinterpret { .. }
        | CheckedExpression::BooleanOperation { .. }
        | CheckedExpression::EnumEquality { .. }
        | CheckedExpression::ArrayFill { .. }
        | CheckedExpression::ConstructStruct { .. }
        | CheckedExpression::ConstructEnum { .. }
        | CheckedExpression::ProjectValue { .. } => {}
        // Naming a path reads no content: a reference formation evaluates its
        // index and endpoint atoms, which are this expression's own children
        // and are walked below [REF-1, REF-4].
        CheckedExpression::BorrowAddressed { .. } => {}
        CheckedExpression::Binding { binding, .. } => {
            read(footprint, places.resolve(PlaceRoot::Binding(*binding), &[]));
        }
        CheckedExpression::Project {
            binding, fields, ..
        } => read(
            footprint,
            places.resolve(PlaceRoot::Binding(*binding), &field_steps(fields)),
        ),
        // `deref(p)` is the path `p` names [TYPE-7, REF-1], which is what
        // resolving its root through the reference summary produces.
        CheckedExpression::DerefAddressed { binding, .. } => {
            read(footprint, places.resolve(PlaceRoot::Binding(*binding), &[]));
        }
        CheckedExpression::ContainerMeasure { root, .. }
        | CheckedExpression::ReadStorage { root, .. } => {
            read(footprint, places.resolve(root.root, &container_steps(root)));
        }
        // [REF-4, MSR-2] a measure or element read through a range reference
        // reads the path the reference names; the subscript's own offset is
        // this expression's child and is walked below.
        CheckedExpression::RangeMeasure { root, .. }
        | CheckedExpression::RangeIndex { root, .. } => {
            read(
                footprint,
                places.resolve(PlaceRoot::Binding(root.binding), &[]),
            );
        }
        // Forming a range names a path and reads no content [REF-1, REF-4].
        CheckedExpression::RangeOf { .. } => {}
        CheckedExpression::ArrayMeasure { root, .. }
        | CheckedExpression::ArrayIndex { root, .. } => match root {
            CheckedArrayRoot::Binding { binding, fields } => read(
                footprint,
                places.resolve(PlaceRoot::Binding(*binding), &field_steps(fields)),
            ),
            CheckedArrayRoot::Constant(id) => read(
                footprint,
                vec![ResolvedPlace {
                    root: PlaceRoot::Constant(*id),
                    path: Vec::new(),
                }],
            ),
        },
        // [GRAM-9] forbids a call in argument position; if one ever reaches
        // here its whole footprint is unaccounted for.
        CheckedExpression::UserCall { .. } => {
            footprint.unresolved = Some(node.clone());
        }
        // One clause-only datum; no executable statement carries one.
        CheckedExpression::PostconditionResultMeasure { .. } => {}
        // Expression forms whose v0.60 operation left [OP-1]'s table and
        // which the checker no longer builds: the buffer and arena formers
        // [BLK-1, STOR-2], the box former [OP-13 builds one through a call],
        // and the two reborrow shapes [OWN-6]. An occurrence would be
        // storage this walk cannot account for, so it fails closed rather
        // than contributing nothing.
        CheckedExpression::BufferFill { .. }
        | CheckedExpression::BufferVacant { .. }
        | CheckedExpression::BufferFits { .. }
        | CheckedExpression::BufferMeasure { .. }
        | CheckedExpression::BufferIndex { .. }
        | CheckedExpression::BorrowBuffer { .. }
        | CheckedExpression::BorrowBox { .. }
        | CheckedExpression::ReborrowAddressed { .. }
        | CheckedExpression::BoxNew { .. }
        | CheckedExpression::BoxDeref { .. }
        | CheckedExpression::BoxTake { .. }
        | CheckedExpression::ArenaNew { .. }
        | CheckedExpression::ArenaDeref { .. } => {
            footprint.unresolved = Some(node.clone());
        }
    }
    for child in expression_children(expression) {
        collect_operand_reads(places, child, node, footprint);
    }
}

/// The caller places one actual names [REF-1].
///
/// A reference argument names a path, so its actual resolves to the path the
/// reference names rather than to any storage of its own; at a join a
/// reference names a set, and every check on it must hold for every member.
fn argument_places(
    places: &PlaceMap,
    argument: &CheckedExpression,
) -> Option<Vec<ResolvedPlace>> {
    match argument {
        CheckedExpression::Binding { binding, .. }
        | CheckedExpression::DerefAddressed { binding, .. } => {
            Some(places.resolve(PlaceRoot::Binding(*binding), &[]))
        }
        CheckedExpression::Project {
            binding, fields, ..
        } => Some(places.resolve(PlaceRoot::Binding(*binding), &field_steps(fields))),
        CheckedExpression::BorrowAddressed { root, .. } => {
            Some(places.resolve(root.root, &container_steps(root)))
        }
        _ => None,
    }
}
