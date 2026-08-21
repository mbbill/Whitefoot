//! The permission judgment P: whether two sibling call statements may be
//! executed with their evaluations overlapped.
//!
//! P is a compiler-internal legality judgment. It refuses nothing, changes no
//! acceptance, and grants no lowering by itself; it records, per analyzed
//! site, whether overlapping the two statements is permitted and whether the
//! permitted overlap is actualizable.
//!
//! For an ordered pair (s1, s2) of adjacent `let x = f(...);` statements in
//! one block, both binding the result of one named-function call, P(s1, s2)
//! holds exactly when all four conditions hold:
//!
//! 1. **No dataflow.** No argument of s2 mentions a binding s1 defines.
//! 2. **Disjoint footprints.** Projecting each callee's declared `reads` and
//!    `writes` regions onto its argument resolved places — the same [EFF-2]
//!    boundary projection [ENT-5] kills use — gives W(s) and R(s). P requires
//!    W(s1) disjoint from W(s2) and R(s2), and W(s2) disjoint from R(s1),
//!    under [OWN-7]'s overlap relation over resolved places. An actual whose
//!    caller place this analysis cannot resolve fails closed.
//! 3. **Row gate.** Neither callee's row carries `external` or `blocks`.
//!    Rows gate; places prove. No disjointness is ever derived from a row.
//! 4. **No skipping exit.** No exit edge of s1 bypasses s2: s1's only
//!    continuation is s2. A `propagate` right-hand side has an `Err` edge to
//!    the function-return sink [ERR-3], so it is never a first member.
//!
//! Actualization additionally requires eligibility: the transitive call
//! closure of both callees reaches zero `claim` sites. Claims are the only
//! writer-reachable runtime checks, so a claim-free closure has no trap site
//! and therefore no trap-selection question.
//!
//! **Invariant.** P consults typing, declared effect rows, resolved places
//! [OWN-5, OWN-7], the statement graph's exit edges, and the monomorphized
//! call graph — and never the entailment fact state. Nothing here reads a
//! derived fact, an obligation disposition, a claim disposition, or any
//! optimizer fact; the only thing it asks about a claim is that its statement
//! exists. Facts-on and facts-off compilation therefore produce the same
//! permission table by construction, and permission can never turn an
//! accepted program into a rejected one or move a required check.

use std::collections::VecDeque;

use super::entailment::collect_statement_calls;
use super::model::{
    BindingId, CheckedArrayRoot, CheckedExpression, CheckedFunction, CheckedMode,
    CheckedSliceSource, CheckedStatement, CheckedType, FunctionId, expression_children,
};
use super::places::{PlaceMap, PlaceRoot, PlaceTerm, ResolvedPlace};
use crate::{DeclarationId, NodePath};

/// The declared effect row and region parameters of one concrete function, as
/// P reads them. This is the callable boundary only: no body fact enters.
#[derive(Clone, Debug, Default)]
pub(crate) struct PermissionSignature {
    /// Formal region parameters in declaration order.
    pub(crate) region_parameters: Vec<DeclarationId>,
    pub(crate) reads: Vec<DeclarationId>,
    pub(crate) writes: Vec<DeclarationId>,
    pub(crate) allocates_arenas: Vec<DeclarationId>,
    pub(crate) external: bool,
    pub(crate) blocks: bool,
}

/// Which member of an analyzed pair a denial cites.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PairSide {
    First,
    Second,
}

/// One exit edge of a candidate statement that does not reach the statement's
/// ordinary successor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExitKind {
    /// A `propagate` right-hand side's `Err` edge to the function-return
    /// sink [ERR-3].
    PropagateError,
}

/// One footprint element: a resolved caller place, or one arena region whose
/// allocation list the callee appends to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Access {
    /// Storage the callee reaches through one actual.
    Place {
        place: ResolvedPlace,
        /// The actual's source node, for citation.
        argument: NodePath,
    },
    /// The caller region an `allocates(arena 'r)` row appends into. Two
    /// overlapped calls allocating into one region would both mutate that
    /// region's allocation list, so the region is a written footprint element
    /// of its own, with no actual to project onto.
    Arena {
        region: DeclarationId,
        call: NodePath,
    },
}

impl Access {
    fn conflicts(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Place { place: left, .. }, Self::Place { place: right, .. }) => {
                left.overlaps(right)
            }
            (Self::Arena { region: left, .. }, Self::Arena { region: right, .. }) => left == right,
            (Self::Place { .. }, Self::Arena { .. }) | (Self::Arena { .. }, Self::Place { .. }) => {
                false
            }
        }
    }
}

/// Why P does not hold for one analyzed pair. Each variant names exactly one
/// condition of the judgment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Denial {
    /// Condition 1: an argument of s2 mentions a binding s1 defines.
    Dataflow { binding: BindingId },
    /// Condition 2: two accesses of the two footprints conflict.
    Footprint { left: Access, right: Access },
    /// Condition 2, fail-closed: the row projects an access through an actual
    /// whose caller place this analysis cannot resolve.
    UnresolvedFootprint { side: PairSide, argument: NodePath },
    /// Condition 3: a callee row carries `external` or `blocks`.
    Row {
        side: PairSide,
        external: bool,
        blocks: bool,
    },
    /// Condition 4: an exit edge of s1 does not reach s2.
    SkippingExit { kind: ExitKind },
}

impl Denial {
    /// The judgment condition this denial cites. The permission ledger prints
    /// it and the judgment tests assert it; acceptance never reads it.
    #[allow(dead_code)]
    pub(crate) const fn condition(&self) -> u8 {
        match self {
            Self::Dataflow { .. } => 1,
            Self::Footprint { .. } | Self::UnresolvedFootprint { .. } => 2,
            Self::Row { .. } => 3,
            Self::SkippingExit { .. } => 4,
        }
    }
}

/// One reachable `claim` site that keeps a permitted pair from being
/// actualized, with the call path that reaches it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClaimWitness {
    /// Analyzed-callee-to-claim-bearing-callee path, root first.
    pub(crate) path: Vec<FunctionId>,
    /// The claim-bearing function's own name, for citation.
    pub(crate) function: String,
    /// The claim's written name.
    pub(crate) claim: String,
    pub(crate) node_path: NodePath,
}

/// The judgment's outcome for one analyzed pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PermissionVerdict {
    /// P holds and both call closures are claim-free.
    PermittedEligible,
    /// P holds, but a reachable `claim` site keeps the overlap out of reach
    /// of this version's actualization. Each checker improvement that turns a
    /// claim into a proof widens the eligible set [ENT-1].
    PermittedNotActualizable {
        claim_sites: usize,
        witness: ClaimWitness,
    },
    Denied(Denial),
}

impl PermissionVerdict {
    pub(crate) const fn is_eligible(&self) -> bool {
        matches!(self, Self::PermittedEligible)
    }

    /// The cited condition of a denial, or `None` for a permitted verdict.
    #[allow(dead_code)]
    pub(crate) const fn denied_condition(&self) -> Option<u8> {
        match self {
            Self::Denied(denial) => Some(denial.condition()),
            Self::PermittedEligible | Self::PermittedNotActualizable { .. } => None,
        }
    }
}

/// One analyzed call statement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PermissionSite {
    /// The owning `let_stmt` node.
    pub(crate) statement: NodePath,
    /// The binding the statement defines. A join must complete before the
    /// first use of it.
    pub(crate) binding: BindingId,
    /// The call occurrence inside it.
    pub(crate) call: NodePath,
    pub(crate) callee: FunctionId,
    pub(crate) callee_name: String,
}

/// One ordered pair of adjacent call statements and its verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PermissionPair {
    pub(crate) first: PermissionSite,
    pub(crate) second: PermissionSite,
    pub(crate) verdict: PermissionVerdict,
}

/// A maximal chain of at least two adjacent call statements every ordered
/// pair of which is permitted and eligible. A chain is not implied by its
/// adjacent pairs, so every ordered pair inside it is judged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PermissionRun {
    pub(crate) sites: Vec<PermissionSite>,
}

/// Every analyzed pair and eligible chain of one concrete function, in source
/// order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FunctionPermissions {
    pub(crate) function: String,
    pub(crate) pairs: Vec<PermissionPair>,
    pub(crate) runs: Vec<PermissionRun>,
}

/// The whole-program permission table, dense by [`FunctionId`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PermissionMetadata {
    pub(crate) functions: Vec<FunctionPermissions>,
}

impl PermissionMetadata {
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
///
/// `signatures` is dense by [`FunctionId`] and carries only callable-boundary
/// data. Nothing in this call graph, statement walk, or place resolution reads
/// a derived fact.
pub(crate) fn analyze_permission(
    functions: &[CheckedFunction],
    signatures: &[PermissionSignature],
) -> PermissionMetadata {
    let program = Program {
        functions,
        signatures,
        callees: direct_callees(functions),
        claims: functions
            .iter()
            .map(|function| {
                let mut sites = Vec::new();
                collect_claim_sites(&function.body, &mut sites);
                sites
            })
            .collect(),
    };
    PermissionMetadata {
        functions: functions
            .iter()
            .map(|function| program.analyze_function(function))
            .collect(),
    }
}

/// One `claim` statement, by written name and node.
struct ClaimRecord {
    name: String,
    node_path: NodePath,
}

struct Program<'check> {
    functions: &'check [CheckedFunction],
    signatures: &'check [PermissionSignature],
    /// Direct monomorphized callees per function, ascending and deduplicated.
    callees: Vec<Vec<FunctionId>>,
    /// `claim` statements per function, in source order.
    claims: Vec<Vec<ClaimRecord>>,
}

/// One candidate statement: a `let` whose right-hand side is exactly one
/// named-function call.
struct Candidate<'check> {
    statement: NodePath,
    /// The binding the statement defines.
    binding: BindingId,
    call: NodePath,
    callee: FunctionId,
    arguments: &'check [CheckedExpression],
    argument_nodes: &'check [NodePath],
    regions: &'check [DeclarationId],
    /// An exit edge of this statement that does not reach its successor.
    exit: Option<ExitKind>,
}

impl<'check> Program<'check> {
    fn analyze_function(&self, function: &'check CheckedFunction) -> FunctionPermissions {
        let places = PlaceMap::for_function(function);
        let mut permissions = FunctionPermissions {
            function: function.name.clone(),
            pairs: Vec::new(),
            runs: Vec::new(),
        };
        let mut blocks = vec![function.body.as_slice()];
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
        permissions
    }

    fn analyze_block(
        &self,
        places: &PlaceMap,
        block: &'check [CheckedStatement],
        permissions: &mut FunctionPermissions,
    ) {
        let mut index = 0;
        while index < block.len() {
            let mut group = Vec::new();
            while let Some(candidate) = block.get(index).and_then(candidate_of) {
                group.push(candidate);
                index += 1;
            }
            if group.is_empty() {
                index += 1;
                continue;
            }
            for window in group.windows(2) {
                let verdict = self.judge(places, &window[0], &window[1]);
                permissions.pairs.push(PermissionPair {
                    first: self.site(&window[0]),
                    second: self.site(&window[1]),
                    verdict,
                });
            }
            self.collect_runs(places, &group, permissions);
        }
    }

    /// Grows maximal chains whose every ordered pair is permitted and
    /// eligible.
    fn collect_runs(
        &self,
        places: &PlaceMap,
        group: &[Candidate<'check>],
        permissions: &mut FunctionPermissions,
    ) {
        let mut start = 0;
        while start + 1 < group.len() {
            let mut end = start;
            while end + 1 < group.len()
                && (start..=end).all(|earlier| {
                    self.judge(places, &group[earlier], &group[end + 1])
                        .is_eligible()
                })
            {
                end += 1;
            }
            if end > start {
                permissions.runs.push(PermissionRun {
                    sites: group[start..=end]
                        .iter()
                        .map(|candidate| self.site(candidate))
                        .collect(),
                });
                start = end + 1;
            } else {
                start += 1;
            }
        }
    }

    fn site(&self, candidate: &Candidate<'check>) -> PermissionSite {
        PermissionSite {
            statement: candidate.statement.clone(),
            binding: candidate.binding,
            call: candidate.call.clone(),
            callee: candidate.callee,
            callee_name: self
                .functions
                .get(candidate.callee.0 as usize)
                .map(|function| function.name.clone())
                .unwrap_or_default(),
        }
    }

    /// The four conditions in their numbered order, then eligibility.
    fn judge(
        &self,
        places: &PlaceMap,
        first: &Candidate<'check>,
        second: &Candidate<'check>,
    ) -> PermissionVerdict {
        // Condition 1: ordinary def-use.
        let mut used = Vec::new();
        for argument in second.arguments {
            collect_used_bindings(argument, &mut used);
        }
        if used.contains(&first.binding) {
            return PermissionVerdict::Denied(Denial::Dataflow {
                binding: first.binding,
            });
        }

        // Condition 2: disjoint footprints under OWN-7, fail closed.
        let left = self.footprint(places, first);
        let right = self.footprint(places, second);
        if let Some(argument) = left.unresolved {
            return PermissionVerdict::Denied(Denial::UnresolvedFootprint {
                side: PairSide::First,
                argument,
            });
        }
        if let Some(argument) = right.unresolved {
            return PermissionVerdict::Denied(Denial::UnresolvedFootprint {
                side: PairSide::Second,
                argument,
            });
        }
        for write in &left.writes {
            for access in right.writes.iter().chain(right.reads.iter()) {
                if write.conflicts(access) {
                    return PermissionVerdict::Denied(Denial::Footprint {
                        left: write.clone(),
                        right: access.clone(),
                    });
                }
            }
        }
        for write in &right.writes {
            for read in &left.reads {
                if write.conflicts(read) {
                    return PermissionVerdict::Denied(Denial::Footprint {
                        left: read.clone(),
                        right: write.clone(),
                    });
                }
            }
        }

        // Condition 3: the row gate.
        for (side, candidate) in [(PairSide::First, first), (PairSide::Second, second)] {
            let Some(signature) = self.signatures.get(candidate.callee.0 as usize) else {
                return PermissionVerdict::Denied(Denial::Row {
                    side,
                    external: true,
                    blocks: true,
                });
            };
            if signature.external || signature.blocks {
                return PermissionVerdict::Denied(Denial::Row {
                    side,
                    external: signature.external,
                    blocks: signature.blocks,
                });
            }
        }

        // Condition 4: no exit edge of s1 bypasses s2.
        if let Some(kind) = first.exit {
            return PermissionVerdict::Denied(Denial::SkippingExit { kind });
        }

        // Eligibility: both call closures reach zero claim sites.
        match self.claim_closure(&[first.callee, second.callee]) {
            Some((claim_sites, witness)) => PermissionVerdict::PermittedNotActualizable {
                claim_sites,
                witness,
            },
            None => PermissionVerdict::PermittedEligible,
        }
    }

    /// The written and read footprints of one call, by [EFF-2] boundary
    /// projection onto the actuals' resolved places.
    fn footprint(&self, places: &PlaceMap, candidate: &Candidate<'check>) -> Footprint {
        let mut footprint = Footprint::default();
        let (Some(signature), Some(callee)) = (
            self.signatures.get(candidate.callee.0 as usize),
            self.functions.get(candidate.callee.0 as usize),
        ) else {
            footprint.unresolved = Some(candidate.call.clone());
            return footprint;
        };

        // An `allocates(arena 'r)` row appends to the caller region's
        // allocation list, which is written storage with no actual of its own.
        for formal in &signature.allocates_arenas {
            match signature
                .region_parameters
                .iter()
                .position(|region| region == formal)
                .and_then(|index| candidate.regions.get(index))
            {
                Some(region) => footprint.writes.push(Access::Arena {
                    region: *region,
                    call: candidate.call.clone(),
                }),
                None => footprint.unresolved = Some(candidate.call.clone()),
            }
        }

        for (index, parameter) in callee.parameters.iter().enumerate() {
            let Some(argument) = candidate.arguments.get(index) else {
                footprint.unresolved = Some(candidate.call.clone());
                return footprint;
            };
            let node = candidate
                .argument_nodes
                .get(index)
                .unwrap_or(&candidate.call);
            let mode_region = match parameter.mode {
                CheckedMode::Own => None,
                CheckedMode::Shared(region) | CheckedMode::Unique(region) => Some(region),
            };
            let slice_region = match parameter.ty {
                CheckedType::Slice { region, .. } => Some(region),
                _ => None,
            };
            let carries = |declared: &[DeclarationId]| {
                mode_region.is_some_and(|region| declared.contains(&region))
                    || slice_region.is_some_and(|region| declared.contains(&region))
            };
            let written = carries(&signature.writes);
            let read = carries(&signature.reads);
            if written || read {
                match argument_place(places, argument) {
                    Some(place) => {
                        let access = Access::Place {
                            place,
                            argument: node.clone(),
                        };
                        if written {
                            footprint.writes.push(access.clone());
                        }
                        if read {
                            footprint.reads.push(access);
                        }
                    }
                    None => footprint.unresolved = Some(node.clone()),
                }
                continue;
            }
            // A consumed `own` actual transfers caller storage into the
            // callee. The affine discipline already forbids two consumers of
            // one place; the footprint states it rather than assuming it.
            if parameter.mode == CheckedMode::Own
                && let Some(place) = consumed_place(places, argument)
            {
                footprint.writes.push(Access::Place {
                    place,
                    argument: node.clone(),
                });
            }
        }
        footprint
    }

    /// Total reachable `claim` sites over the transitive call closure of the
    /// given roots, with the first witness in deterministic order, or `None`
    /// when the closure is claim-free. One breadth-first walk covers both
    /// roots, so a function reachable from both is counted once.
    fn claim_closure(&self, roots: &[FunctionId]) -> Option<(usize, ClaimWitness)> {
        let mut total = 0;
        let mut witness = None;
        let mut visited = vec![false; self.functions.len()];
        let mut queue = VecDeque::new();
        let mut parents: Vec<(FunctionId, Option<FunctionId>)> = Vec::new();
        for root in roots {
            let Some(seen) = visited.get_mut(root.0 as usize) else {
                continue;
            };
            if !*seen {
                *seen = true;
                parents.push((*root, None));
                queue.push_back(*root);
            }
        }
        while let Some(current) = queue.pop_front() {
            let claims = self
                .claims
                .get(current.0 as usize)
                .map(Vec::as_slice)
                .unwrap_or_default();
            total += claims.len();
            if witness.is_none()
                && let Some(claim) = claims.first()
            {
                witness = Some(ClaimWitness {
                    path: witness_path(&parents, current),
                    function: self
                        .functions
                        .get(current.0 as usize)
                        .map(|function| function.name.clone())
                        .unwrap_or_default(),
                    claim: claim.name.clone(),
                    node_path: claim.node_path.clone(),
                });
            }
            let callees = self
                .callees
                .get(current.0 as usize)
                .map(Vec::as_slice)
                .unwrap_or_default();
            for callee in callees {
                let Some(seen) = visited.get_mut(callee.0 as usize) else {
                    continue;
                };
                if !*seen {
                    *seen = true;
                    parents.push((*callee, Some(current)));
                    queue.push_back(*callee);
                }
            }
        }
        witness.map(|witness| (total, witness))
    }
}

/// The breadth-first path from a root to `target`, root first.
fn witness_path(
    parents: &[(FunctionId, Option<FunctionId>)],
    target: FunctionId,
) -> Vec<FunctionId> {
    let mut path = vec![target];
    let mut current = target;
    while let Some((_, Some(parent))) = parents.iter().find(|(node, _)| *node == current) {
        path.push(*parent);
        current = *parent;
    }
    path.reverse();
    path
}

#[derive(Debug, Default)]
struct Footprint {
    writes: Vec<Access>,
    reads: Vec<Access>,
    /// Set when the row projects an access this analysis cannot resolve to a
    /// caller place. Every such call is denied.
    unresolved: Option<NodePath>,
}

/// One candidate statement, or `None` for every other statement shape.
fn candidate_of(statement: &CheckedStatement) -> Option<Candidate<'_>> {
    let (node_path, binding, value, exit) = match statement {
        CheckedStatement::Let {
            node_path,
            binding,
            value,
        } => (node_path, binding, value, None),
        CheckedStatement::PropagateLet {
            node_path,
            binding,
            scrutinee,
            ..
        } => (
            node_path,
            binding,
            scrutinee,
            Some(ExitKind::PropagateError),
        ),
        _ => return None,
    };
    let CheckedExpression::UserCall {
        function,
        call,
        argument_nodes,
        arguments,
        goal_regions,
        ..
    } = value
    else {
        return None;
    };
    Some(Candidate {
        statement: node_path.clone(),
        binding: *binding,
        call: call.clone(),
        callee: *function,
        arguments,
        argument_nodes,
        regions: goal_regions,
        exit,
    })
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
        _ => {}
    }
}

fn collect_claim_sites(statements: &[CheckedStatement], out: &mut Vec<ClaimRecord>) {
    for statement in statements {
        match statement {
            CheckedStatement::Claim { name, site, .. } => out.push(ClaimRecord {
                name: name.clone(),
                node_path: site.node_path.clone(),
            }),
            CheckedStatement::Match { arms, .. } | CheckedStatement::ValueMatchLet { arms, .. } => {
                for arm in arms {
                    collect_claim_sites(&arm.body, out);
                }
            }
            CheckedStatement::Loop { body, .. }
            | CheckedStatement::Region { body, .. }
            | CheckedStatement::CountedRange { body, .. } => collect_claim_sites(body, out),
            _ => {}
        }
    }
}

/// The direct monomorphized callees of every function, from the shared
/// concrete call-occurrence collector. This is call-graph shape only.
fn direct_callees(functions: &[CheckedFunction]) -> Vec<Vec<FunctionId>> {
    functions
        .iter()
        .map(|function| {
            let mut calls = Vec::new();
            collect_statement_calls(function.id, &function.body, &mut calls);
            let mut callees = calls
                .into_iter()
                .map(|call| call.callee)
                .collect::<Vec<_>>();
            callees.sort_unstable_by_key(|callee| callee.0);
            callees.dedup();
            callees
        })
        .collect()
}

/// Every binding one expression tree mentions, for the ordinary def-use test.
fn collect_used_bindings(expression: &CheckedExpression, out: &mut Vec<BindingId>) {
    let mut note = |binding: BindingId| {
        if !out.contains(&binding) {
            out.push(binding);
        }
    };
    match expression {
        CheckedExpression::Binding { binding, .. }
        | CheckedExpression::Project { binding, .. }
        | CheckedExpression::BorrowAddressed { binding, .. }
        | CheckedExpression::BorrowBox { binding, .. }
        | CheckedExpression::BorrowSystemResource { binding, .. }
        | CheckedExpression::ReborrowAddressed { binding, .. }
        | CheckedExpression::DerefAddressed { binding, .. } => note(*binding),
        CheckedExpression::BorrowBuffer { root, .. }
        | CheckedExpression::BufferLength { root }
        | CheckedExpression::BufferIndex { root, .. } => note(root.binding),
        CheckedExpression::SliceLength { root } | CheckedExpression::SliceIndex { root, .. } => {
            note(root.binding)
        }
        CheckedExpression::ArrayLength { root, .. }
        | CheckedExpression::ArrayIndex { root, .. } => {
            if let CheckedArrayRoot::Binding { binding, .. } = root {
                note(*binding);
            }
        }
        CheckedExpression::SliceOf { source, .. } => match source {
            CheckedSliceSource::Array { root, .. } => {
                if let CheckedArrayRoot::Binding { binding, .. } = root {
                    note(*binding);
                }
            }
            CheckedSliceSource::Buffer(root) => note(root.binding),
            CheckedSliceSource::ArenaContent { binding, .. } => note(*binding),
        },
        _ => {}
    }
    for child in expression_children(expression) {
        collect_used_bindings(child, out);
    }
}

/// The caller place one actual reaches, for a parameter whose row projects an
/// access through it.
fn argument_place(places: &PlaceMap, argument: &CheckedExpression) -> Option<ResolvedPlace> {
    if let Some((place, _element, _entry_image)) = places.argument_referent(argument) {
        return Some(place);
    }
    match argument {
        CheckedExpression::SliceOf { source, .. } => Some(slice_source_place(places, source)),
        _ => None,
    }
}

/// The storage a direct slice value views.
fn slice_source_place(places: &PlaceMap, source: &CheckedSliceSource) -> ResolvedPlace {
    match source {
        CheckedSliceSource::Array { root, .. } => match root {
            CheckedArrayRoot::Binding { binding, fields } => rooted_place(places, *binding, fields),
            CheckedArrayRoot::Constant(id) => ResolvedPlace {
                root: PlaceRoot::Constant(*id),
                fields: Vec::new(),
            },
        },
        CheckedSliceSource::Buffer(root) => rooted_place(places, root.binding, &root.fields),
        CheckedSliceSource::ArenaContent {
            binding, fields, ..
        } => rooted_place(places, *binding, fields),
    }
}

fn rooted_place(places: &PlaceMap, binding: BindingId, fields: &[u32]) -> ResolvedPlace {
    places.resolve(&PlaceTerm {
        root: PlaceRoot::Binding(binding),
        deref: places.is_holder(binding),
        fields: fields.to_vec(),
    })
}

/// The caller place a consuming `own` actual transfers away, when the actual
/// names one.
fn consumed_place(places: &PlaceMap, argument: &CheckedExpression) -> Option<ResolvedPlace> {
    match argument {
        CheckedExpression::Binding {
            binding,
            consume_root: true,
            ..
        } => Some(rooted_place(places, *binding, &[])),
        CheckedExpression::Project {
            binding,
            consume_root: true,
            fields,
            ..
        } => Some(rooted_place(places, *binding, fields)),
        _ => None,
    }
}
