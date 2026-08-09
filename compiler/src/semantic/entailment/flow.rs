//! [ENT-3] flow of facts over the conservative structural graph [FN-1], with
//! [ENT-5] kills, joins, and the no-induction loop rule, and [ENT-6]
//! obligation discharge with residual rendering.
//!
//! The walker carries the live fact state forward through the checked
//! statement tree, which is the structural graph: statements sequence, match
//! arms fork and join, loops iterate through their break edges, and
//! `return`/`give`/`break`/`propagate` leave scopes on edges. Scope-exit
//! kills always apply on the edge, before any join at the edge's target.
//!
//! The [ENT-3] fact sources themselves — which checked shape establishes
//! which relation — live in [`sources`]; this module owns the graph, the
//! kills, the joins, and the obligation judgment, and calls into the sources
//! at each establishment point.

mod sources;

use std::collections::HashSet;

use super::super::model::{
    BindingId, CheckedArrayRoot, CheckedConst, CheckedExpression, CheckedFunction, CheckedLoopId,
    CheckedMatchArm, CheckedMode, CheckedNominal, CheckedNominalKind, CheckedSetTarget,
    CheckedStatement, CheckedType, CheckedValue, IntegerType,
};
use super::state::{FactState, OutcomeFact, Relation, close, join};
use super::term::{LengthBound, PlaceRoot, PlaceTerm, TermId, TermKind, TermTable, integer_value};
use super::{
    ClaimDisposition, ClaimOutcome, EntailmentContext, FunctionEntailment, ObligationOutcome,
    fragment_type,
};
use crate::{SYSTEM_OPERATIONS, SystemParameterMode};

/// One [OWN-5] resolved place, for the [OWN-7] overlap relation kills use.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ResolvedPlace {
    root: PlaceRoot,
    fields: Vec<u32>,
}

impl ResolvedPlace {
    /// [OWN-7]: places overlap when one's path is a prefix of the other's.
    fn overlaps(&self, other: &Self) -> bool {
        self.root == other.root
            && (self.fields.starts_with(&other.fields) || other.fields.starts_with(&self.fields))
    }
}

/// One [ENT-5] kill event gathered from a statement or expression.
#[derive(Clone, Debug)]
enum KillEvent {
    /// (a) a `set` commit or (b) a boundary-projected callee write. An
    /// element write targets indexed element storage, which never kills a
    /// length fact [ENT-5].
    Write { place: ResolvedPlace, element: bool },
    /// (c) a consuming use of a binding.
    Consume(BindingId),
}

/// What one binding reads through by `deref`, for place resolution.
#[derive(Clone, Debug)]
enum HolderReferent {
    /// A borrow of a known local place.
    Place {
        binding: BindingId,
        fields: Vec<u32>,
    },
    /// A reborrow: reads through another holder.
    Holder(BindingId),
    /// A parameter or match-binder borrow, or an owning box: the referent has
    /// no caller-visible local place, so the binding itself anchors identity.
    Opaque,
}

#[derive(Clone, Debug, Default)]
struct BindingSummary {
    ty: Option<CheckedType>,
    holder: Option<HolderReferent>,
}

/// A `loop` frame collecting break-edge states for the continuation join.
struct LoopFrame {
    id: CheckedLoopId,
    scope_depth: usize,
    breaks: Vec<FactState>,
}

/// The [ENT-3] facts one `match` scrutinee admits at its arms' entries: the
/// S1 comparison relation, taken positively on `True()` and exactly negated
/// on `False()`, and the S7/S10 fact one named arm's value binder gains,
/// carried with that arm's tag. Every other arm establishes nothing.
#[derive(Default)]
struct ArmFacts {
    comparison: Option<Relation>,
    outcome: Option<(u32, OutcomeFact)>,
}

/// A `value_match` frame collecting give-edge states for the continuation.
struct GiveFrame {
    scope_depth: usize,
    gives: Vec<FactState>,
}

/// The [ENT-5] loop rule's structural kill summary of one loop body.
#[derive(Default)]
struct LoopKills {
    events: Vec<KillEvent>,
    /// `set` targets naming a whole binding, which invalidate any comparison
    /// origin held by that binding.
    set_bindings: HashSet<BindingId>,
}

// A continuing scope-exit edge can close only scopes opened inside the target
// loop body. No binding from such a scope can support a fact in the pre-loop
// state this summary filters. An edge that closes a pre-loop binding's scope
// necessarily leaves the target body and is non-continuing, so kill event (d)
// needs no payload in `LoopKills`.

/// Non-local successors visible while asking whether an edge inside one loop
/// body can reach that loop's next iteration head without leaving the body.
/// Targets outside the body are deliberately absent and therefore do not
/// reach the head.
#[derive(Default)]
struct LoopReachability {
    breaks: Vec<(CheckedLoopId, bool)>,
    gives: Vec<bool>,
}

impl LoopReachability {
    fn break_reaches(&self, target: CheckedLoopId) -> bool {
        self.breaks
            .iter()
            .rev()
            .find_map(|(id, reaches)| (*id == target).then_some(*reaches))
            .unwrap_or(false)
    }
}

pub(super) fn analyze(
    function: &CheckedFunction,
    context: &EntailmentContext<'_>,
) -> FunctionEntailment {
    let mut analyzer = Analyzer {
        context,
        function,
        bindings: Vec::new(),
        terms: TermTable::new(),
        obligations: Vec::new(),
        claims: Vec::new(),
        scopes: Vec::new(),
        loops: Vec::new(),
        gives: Vec::new(),
    };
    analyzer.collect_bindings();
    let mut state = FactState::default();
    analyzer
        .scopes
        .push(function.parameters.iter().map(|p| p.binding).collect());
    // [ENT-3] S4: the substituted `requires` relation enters the body's entry
    // fact state, the one fact that crosses into the body [ENT-2, FN-8].
    analyzer.establish_requires_facts(&mut state);
    analyzer.walk_block(&function.body, &mut state);
    analyzer.scopes.pop();
    FunctionEntailment {
        obligations: analyzer.obligations,
        claims: analyzer.claims,
    }
}

struct Analyzer<'check, 'unit> {
    context: &'check EntailmentContext<'unit>,
    function: &'check CheckedFunction,
    /// Dense per-binding summaries indexed by [`BindingId`].
    bindings: Vec<BindingSummary>,
    terms: TermTable,
    obligations: Vec<ObligationOutcome>,
    claims: Vec<ClaimOutcome>,
    /// Lexical scope stack: the bindings declared in each open block.
    scopes: Vec<Vec<BindingId>>,
    loops: Vec<LoopFrame>,
    gives: Vec<GiveFrame>,
}

impl Analyzer<'_, '_> {
    // ------------------------------------------------------------------
    // Binding prepass
    // ------------------------------------------------------------------

    fn summary_mut(&mut self, binding: BindingId) -> &mut BindingSummary {
        let index = binding.0 as usize;
        if self.bindings.len() <= index {
            self.bindings.resize(index + 1, BindingSummary::default());
        }
        &mut self.bindings[index]
    }

    fn summary(&self, binding: BindingId) -> Option<&BindingSummary> {
        self.bindings.get(binding.0 as usize)
    }

    fn collect_bindings(&mut self) {
        let function = self.function;
        for parameter in &function.parameters {
            let holder = match parameter.mode {
                CheckedMode::Own => None,
                CheckedMode::Shared(_) | CheckedMode::Unique(_) => Some(HolderReferent::Opaque),
            };
            let summary = self.summary_mut(parameter.binding);
            summary.ty = Some(parameter.ty);
            summary.holder = holder;
        }
        self.collect_block_bindings(&function.body);
    }

    fn collect_block_bindings(&mut self, statements: &[CheckedStatement]) {
        for statement in statements {
            match statement {
                CheckedStatement::Let { binding, value } => {
                    let holder = holder_from_value(value);
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(value.ty());
                    summary.holder = holder;
                }
                CheckedStatement::PropagateLet {
                    binding, ok_type, ..
                } => {
                    self.summary_mut(*binding).ty = Some(*ok_type);
                }
                CheckedStatement::ValueMatchLet {
                    binding,
                    result_type,
                    arms,
                    ..
                } => {
                    self.summary_mut(*binding).ty = Some(*result_type);
                    for arm in arms {
                        self.collect_arm_bindings(arm);
                    }
                }
                CheckedStatement::Match { arms, .. } => {
                    for arm in arms {
                        self.collect_arm_bindings(arm);
                    }
                }
                CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                    self.collect_block_bindings(body);
                }
                _ => {}
            }
        }
    }

    fn collect_arm_bindings(&mut self, arm: &CheckedMatchArm) {
        for binder in &arm.binders {
            let holder = match binder.mode {
                CheckedMode::Own => None,
                CheckedMode::Shared(_) | CheckedMode::Unique(_) => Some(HolderReferent::Opaque),
            };
            let summary = self.summary_mut(binder.binding);
            summary.ty = Some(binder.ty);
            summary.holder = holder;
        }
        self.collect_block_bindings(&arm.body);
    }

    fn is_holder(&self, binding: BindingId) -> bool {
        self.summary(binding)
            .is_some_and(|summary| summary.holder.is_some())
    }

    // ------------------------------------------------------------------
    // Place resolution and support
    // ------------------------------------------------------------------

    /// Resolves a spelled place to its [OWN-5] resolved place, reading
    /// through let-bound borrows; opaque holders anchor at themselves.
    fn resolve(&self, place: &PlaceTerm) -> ResolvedPlace {
        match place.root {
            PlaceRoot::Constant(id) => ResolvedPlace {
                root: PlaceRoot::Constant(id),
                fields: place.fields.clone(),
            },
            PlaceRoot::Binding(binding) => {
                let mut resolved = if place.deref {
                    self.resolve_deref(binding, 0)
                } else {
                    ResolvedPlace {
                        root: PlaceRoot::Binding(binding),
                        fields: Vec::new(),
                    }
                };
                resolved.fields.extend_from_slice(&place.fields);
                resolved
            }
        }
    }

    fn resolve_deref(&self, holder: BindingId, depth: usize) -> ResolvedPlace {
        let anchored = ResolvedPlace {
            root: PlaceRoot::Binding(holder),
            fields: Vec::new(),
        };
        if depth > 32 {
            return anchored;
        }
        match self
            .summary(holder)
            .and_then(|summary| summary.holder.as_ref())
        {
            Some(HolderReferent::Place { binding, fields }) => {
                let mut resolved = if self.is_holder(*binding) {
                    self.resolve_deref(*binding, depth + 1)
                } else {
                    ResolvedPlace {
                        root: PlaceRoot::Binding(*binding),
                        fields: Vec::new(),
                    }
                };
                resolved.fields.extend_from_slice(fields);
                resolved
            }
            Some(HolderReferent::Holder(next)) => self.resolve_deref(*next, depth + 1),
            Some(HolderReferent::Opaque) | None => anchored,
        }
    }

    /// Whether a kill event kills a fact supported by `term` [ENT-5].
    fn event_kills_term(&self, term: TermId, event: &KillEvent) -> bool {
        match self.terms.kind(term).clone() {
            TermKind::Zero | TermKind::Constant(_) | TermKind::ConstParameter(_) => false,
            TermKind::Place(place, _) => match event {
                KillEvent::Write {
                    place: written,
                    element: _,
                } => self.resolve(&place).overlaps(written),
                KillEvent::Consume(root) => place.root == PlaceRoot::Binding(*root),
            },
            TermKind::Length(place) => match event {
                // An element write never kills a length fact: the length is
                // fixed at allocation or by the type [ENT-5].
                KillEvent::Write { element: true, .. } => false,
                KillEvent::Write {
                    place: written,
                    element: false,
                } => {
                    let root = PlaceTerm {
                        root: place.root,
                        deref: place.deref,
                        fields: Vec::new(),
                    };
                    self.resolve(&root).overlaps(written)
                }
                KillEvent::Consume(root) => place.root == PlaceRoot::Binding(*root),
            },
        }
    }

    /// Whether leaving the scopes of `exited` kills a fact supported by
    /// `term`: the support contains every tracked place's root binding and
    /// every holder read through, which is the spelling root here.
    fn scope_kills_term(&self, term: TermId, exited: &HashSet<BindingId>) -> bool {
        match self.terms.kind(term) {
            TermKind::Zero | TermKind::Constant(_) | TermKind::ConstParameter(_) => false,
            TermKind::Place(place, _) | TermKind::Length(place) => match place.root {
                PlaceRoot::Binding(binding) => exited.contains(&binding),
                PlaceRoot::Constant(_) => false,
            },
        }
    }

    fn apply_kills(&self, state: &mut FactState, events: &[KillEvent]) {
        if events.is_empty() {
            return;
        }
        state.kill(|term| {
            events
                .iter()
                .any(|event| self.event_kills_term(term, event))
        });
    }

    /// Applies the scope-exit kills for every scope deeper than `depth`,
    /// as the edge event ordered before any join [ENT-5].
    fn exit_scopes_to(&self, state: &mut FactState, depth: usize) {
        let exited: HashSet<BindingId> =
            self.scopes.iter().skip(depth).flatten().copied().collect();
        if exited.is_empty() {
            return;
        }
        state.kill(|term| self.scope_kills_term(term, &exited));
        state.origins.retain(|binding, _| !exited.contains(binding));
        state
            .outcomes
            .retain(|binding, _| !exited.contains(binding));
    }

    // ------------------------------------------------------------------
    // Terms and relations from checked expressions
    // ------------------------------------------------------------------

    /// Reads an expression as a term or constant [ENT-2]; anything else is
    /// no operand and establishes or derives nothing.
    fn read_operand(&mut self, expression: &CheckedExpression) -> Option<TermId> {
        match expression {
            CheckedExpression::Constant(CheckedValue::Integer { ty, bits }) => Some(
                self.terms
                    .intern(TermKind::Constant(integer_value(*ty, *bits))),
            ),
            CheckedExpression::Binding { binding, ty, .. } => {
                let fragment = fragment_type(*ty)?;
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(*binding),
                    deref: false,
                    fields: Vec::new(),
                };
                Some(self.terms.intern(TermKind::Place(place, fragment)))
            }
            CheckedExpression::Project {
                binding,
                fields,
                ty,
                consume_root: false,
                ..
            } => {
                let fragment = fragment_type(*ty)?;
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(*binding),
                    deref: false,
                    fields: fields.clone(),
                };
                Some(self.terms.intern(TermKind::Place(place, fragment)))
            }
            CheckedExpression::DerefAddressed { binding, ty } => {
                let fragment = fragment_type(*ty)?;
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(*binding),
                    deref: true,
                    fields: Vec::new(),
                };
                Some(self.terms.intern(TermKind::Place(place, fragment)))
            }
            CheckedExpression::BoxDeref {
                referent, value, ..
            } => {
                let fragment = fragment_type(*referent)?;
                let CheckedExpression::Binding { binding, .. } = value.as_ref() else {
                    return None;
                };
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(*binding),
                    deref: true,
                    fields: Vec::new(),
                };
                Some(self.terms.intern(TermKind::Place(place, fragment)))
            }
            CheckedExpression::ProjectValue {
                value, field, ty, ..
            } => {
                let fragment = fragment_type(*ty)?;
                let mut fields = vec![*field];
                let mut cursor = value.as_ref();
                loop {
                    match cursor {
                        CheckedExpression::ProjectValue { value, field, .. } => {
                            fields.insert(0, *field);
                            cursor = value.as_ref();
                        }
                        CheckedExpression::DerefAddressed { binding, .. } => {
                            let place = PlaceTerm {
                                root: PlaceRoot::Binding(*binding),
                                deref: true,
                                fields,
                            };
                            return Some(self.terms.intern(TermKind::Place(place, fragment)));
                        }
                        CheckedExpression::BoxDeref { value, .. } => {
                            let CheckedExpression::Binding { binding, .. } = value.as_ref() else {
                                return None;
                            };
                            let place = PlaceTerm {
                                root: PlaceRoot::Binding(*binding),
                                deref: true,
                                fields,
                            };
                            return Some(self.terms.intern(TermKind::Place(place, fragment)));
                        }
                        _ => return None,
                    }
                }
            }
            _ => None,
        }
    }

    /// [ENT-3] comparison-origin shape (a): a direct comparison call whose
    /// operands are each a term or constant.
    fn direct_comparison(&mut self, expression: &CheckedExpression) -> Option<Relation> {
        let CheckedExpression::IntegerOperation {
            operation,
            operand_type,
            arguments,
            ..
        } = expression
        else {
            return None;
        };
        fragment_type(*operand_type)?;
        let [left_expression, right_expression] = arguments.as_slice() else {
            return None;
        };
        let left = self.read_operand(left_expression)?;
        let right = self.read_operand(right_expression)?;
        sources::comparison_relation(*operation, left, right)
    }

    /// [ENT-3] comparison origin of a match scrutinee: shape (a) directly, or
    /// shape (b), a bare `own Bool` binding whose initializer comparison is
    /// still valid on every path to this use.
    fn scrutinee_relation(
        &mut self,
        expression: &CheckedExpression,
        state: &FactState,
    ) -> Option<Relation> {
        if let Some(relation) = self.direct_comparison(expression) {
            return Some(relation);
        }
        if let CheckedExpression::Binding { binding, ty, .. } = expression
            && *ty == CheckedType::Bool
        {
            return state.origins.get(binding).cloned();
        }
        None
    }

    // ------------------------------------------------------------------
    // Kill collection from expressions
    // ------------------------------------------------------------------

    fn is_copy(&self, ty: CheckedType) -> bool {
        match ty {
            CheckedType::Unit
            | CheckedType::Bool
            | CheckedType::Integer(_)
            | CheckedType::Float(_) => true,
            CheckedType::Nominal(id) => self
                .context
                .nominals
                .get(id.0 as usize)
                .is_some_and(CheckedNominal::is_copy),
            _ => false,
        }
    }

    /// The resolved place a borrow-shaped call argument reads through, and
    /// whether a callee write through it is an element write.
    fn argument_referent(&self, argument: &CheckedExpression) -> Option<(ResolvedPlace, bool)> {
        match argument {
            CheckedExpression::BorrowBuffer { root } => {
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                };
                Some((self.resolve(&place), true))
            }
            CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. } => {
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(*binding),
                    deref: self.is_holder(*binding),
                    fields: Vec::new(),
                };
                Some((self.resolve(&place), false))
            }
            CheckedExpression::ReborrowAddressed { binding, .. } => {
                Some((self.resolve_deref(*binding, 0), false))
            }
            _ => None,
        }
    }

    /// Collects [ENT-5] kill events (b) and (c) from one expression tree.
    fn collect_expression_kills(
        &self,
        expression: &CheckedExpression,
        events: &mut Vec<KillEvent>,
    ) {
        match expression {
            CheckedExpression::Binding { binding, ty, .. } => {
                if !self.is_copy(*ty) {
                    events.push(KillEvent::Consume(*binding));
                }
            }
            CheckedExpression::Project {
                binding,
                consume_root,
                ..
            } => {
                if *consume_root {
                    events.push(KillEvent::Consume(*binding));
                }
            }
            CheckedExpression::UserCall {
                function,
                arguments,
                ..
            } => {
                let callee = self.context.callee(*function);
                for (index, argument) in arguments.iter().enumerate() {
                    let written = callee.is_some_and(|callee| {
                        callee.parameter_writes.get(index).copied().unwrap_or(false)
                    });
                    if written && let Some((place, element)) = self.argument_referent(argument) {
                        events.push(KillEvent::Write { place, element });
                    }
                    self.collect_expression_kills(argument, events);
                }
            }
            CheckedExpression::SystemCall {
                operation,
                arguments,
                ..
            } => {
                let parameters = SYSTEM_OPERATIONS
                    .get(usize::from(*operation))
                    .map(|operation| operation.parameters)
                    .unwrap_or_default();
                for (index, argument) in arguments.iter().enumerate() {
                    let written = parameters.get(index).is_some_and(|parameter| {
                        matches!(parameter.mode, SystemParameterMode::UniqueBorrow(_))
                    });
                    if written && let Some((place, element)) = self.argument_referent(argument) {
                        events.push(KillEvent::Write { place, element });
                    }
                    self.collect_expression_kills(argument, events);
                }
            }
            _ => {
                for child in expression_children(expression) {
                    self.collect_expression_kills(child, events);
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Obligations [ENT-6]
    // ------------------------------------------------------------------

    /// Judges every bounds obligation inside one expression against the
    /// state at this point, inner offsets before the sites they feed.
    fn judge_expression(&mut self, expression: &CheckedExpression, state: &FactState) {
        match expression {
            CheckedExpression::ArrayIndex {
                root,
                length,
                offset,
                trap,
                ..
            } => {
                self.judge_expression(offset, state);
                let base = self.array_root_place(root);
                let node_path = trap.node_path.clone();
                self.judge_obligation(base, Some(*length), offset, node_path, state);
            }
            CheckedExpression::BufferIndex {
                root, offset, trap, ..
            } => {
                self.judge_expression(offset, state);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                };
                let node_path = trap.node_path.clone();
                self.judge_obligation(base, None, offset, node_path, state);
            }
            CheckedExpression::SliceIndex {
                root, offset, trap, ..
            } => {
                self.judge_expression(offset, state);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: Vec::new(),
                };
                let node_path = trap.node_path.clone();
                self.judge_obligation(base, None, offset, node_path, state);
            }
            _ => {
                for child in expression_children(expression) {
                    self.judge_expression(child, state);
                }
            }
        }
    }

    fn array_root_place(&self, root: &CheckedArrayRoot) -> PlaceTerm {
        match root {
            CheckedArrayRoot::Binding { binding, fields } => PlaceTerm {
                root: PlaceRoot::Binding(*binding),
                deref: self.is_holder(*binding),
                fields: fields.clone(),
            },
            CheckedArrayRoot::Constant(id) => PlaceTerm {
                root: PlaceRoot::Constant(*id),
                deref: false,
                fields: Vec::new(),
            },
        }
    }

    /// Interns the length term `len(P)` and, for an `array<T, N>` place,
    /// registers the [ENT-2] implicit length equality `len(P) = N` that holds
    /// at every program point. Every length term is created here, so the
    /// implicit equality is never missed at a site that only reads a length.
    fn length_term(&mut self, base: PlaceTerm, array_length: Option<CheckedConst>) -> TermId {
        let length_term = self.terms.intern(TermKind::Length(base));
        if let Some(length) = array_length {
            let bound = match length {
                CheckedConst::Value(value) => LengthBound::Constant(i128::from(value)),
                CheckedConst::Parameter(declaration) => {
                    LengthBound::Equal(self.terms.intern(TermKind::ConstParameter(declaration)))
                }
            };
            self.terms.set_length_bound(length_term, bound);
        }
        length_term
    }

    /// [ENT-6]: the bounds obligation `i < len(P)`, normalized
    /// `i - len(P) <= -1`, discharged exactly when the closed fact state at
    /// the node derives it.
    fn judge_obligation(
        &mut self,
        base: PlaceTerm,
        array_length: Option<CheckedConst>,
        offset: &CheckedExpression,
        node_path: crate::NodePath,
        state: &FactState,
    ) {
        let length_term = self.length_term(base.clone(), array_length);
        let offset_term = self.read_operand(offset);
        let closed = close(state, &self.terms);
        let discharged = match offset_term {
            Some(offset_term) => closed.derives_bound(offset_term, length_term, -1),
            // An operand that is not a term or constant leaves the relation
            // underivable, never ill-formed [ENT-6] — unless the state is
            // contradictory, where every obligation is discharged [ENT-4].
            None => closed.contradictory(),
        };
        let residual = if discharged {
            None
        } else {
            Some(format!(
                "{} < len({})",
                self.render_expression(offset),
                self.render_place(&base)
            ))
        };
        self.obligations.push(ObligationOutcome {
            node_path,
            discharged,
            contradictory: closed.contradictory(),
            residual,
        });
    }

    fn judge_set_target(&mut self, target: &CheckedSetTarget, state: &FactState) {
        match target {
            CheckedSetTarget::Place(_) => {}
            CheckedSetTarget::ArrayIndex(target) => {
                self.judge_expression(&target.offset, state);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(target.binding),
                    deref: self.is_holder(target.binding),
                    fields: target.fields.clone(),
                };
                let node_path = target.trap.node_path.clone();
                self.judge_obligation(base, Some(target.length), &target.offset, node_path, state);
            }
            CheckedSetTarget::BufferIndex(target) => {
                self.judge_expression(&target.offset, state);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(target.root.binding),
                    deref: self.is_holder(target.root.binding),
                    fields: target.root.fields.clone(),
                };
                let node_path = target.trap.node_path.clone();
                self.judge_obligation(base, None, &target.offset, node_path, state);
            }
        }
    }

    // ------------------------------------------------------------------
    // Statement walk
    // ------------------------------------------------------------------

    /// Walks one block in its own lexical scope. Returns the fall-through:
    /// `true` when control continues past the block with `state` holding the
    /// post-scope-exit facts.
    fn walk_block(&mut self, statements: &[CheckedStatement], state: &mut FactState) -> bool {
        self.scopes.push(Vec::new());
        let mut continues = true;
        for statement in statements {
            if !continues {
                break;
            }
            continues = self.walk_statement(statement, state);
        }
        if continues {
            let depth = self.scopes.len() - 1;
            self.exit_scopes_to(state, depth);
        }
        self.scopes.pop();
        continues
    }

    fn declare(&mut self, binding: BindingId) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(binding);
        }
    }

    fn expression_effects(&mut self, expression: &CheckedExpression, state: &mut FactState) {
        self.judge_expression(expression, state);
        let mut events = Vec::new();
        self.collect_expression_kills(expression, &mut events);
        self.apply_kills(state, &events);
    }

    fn walk_statement(&mut self, statement: &CheckedStatement, state: &mut FactState) -> bool {
        match statement {
            CheckedStatement::Let { binding, value } => {
                self.expression_effects(value, state);
                self.declare(*binding);
                if value.ty() == CheckedType::Bool
                    && let Some(relation) = self.direct_comparison(value)
                {
                    state.origins.insert(*binding, relation);
                }
                // Sources S5, S6, S7, and S9 establish at the binding, after
                // the initializer's own kills [ENT-3, ENT-5].
                self.establish_binding_facts(*binding, value, state);
                true
            }
            CheckedStatement::PropagateLet {
                binding, scrutinee, ..
            } => {
                // The Err edge leaves the function; the normal continuation
                // keeps the preceding state subject to the initializer
                // call's own kill events, and the binder gains no fact
                // [ENT-5].
                self.expression_effects(scrutinee, state);
                self.declare(*binding);
                true
            }
            CheckedStatement::Set { target, value } => {
                // [SET-1]: the target's base and offset are evaluated before
                // the right-hand side; both are judged at this point, then
                // the commit kill applies.
                self.judge_set_target(target, state);
                self.expression_effects(value, state);
                let mut events = Vec::new();
                match target {
                    CheckedSetTarget::Place(place) => {
                        let spelled = PlaceTerm {
                            root: PlaceRoot::Binding(place.binding),
                            deref: self.is_holder(place.binding),
                            fields: place.fields.clone(),
                        };
                        events.push(KillEvent::Write {
                            place: self.resolve(&spelled),
                            element: false,
                        });
                        if place.fields.is_empty() {
                            state.origins.remove(&place.binding);
                            state.outcomes.remove(&place.binding);
                        }
                    }
                    CheckedSetTarget::ArrayIndex(target) => {
                        let spelled = PlaceTerm {
                            root: PlaceRoot::Binding(target.binding),
                            deref: self.is_holder(target.binding),
                            fields: target.fields.clone(),
                        };
                        events.push(KillEvent::Write {
                            place: self.resolve(&spelled),
                            element: true,
                        });
                    }
                    CheckedSetTarget::BufferIndex(target) => {
                        let spelled = PlaceTerm {
                            root: PlaceRoot::Binding(target.root.binding),
                            deref: self.is_holder(target.root.binding),
                            fields: target.root.fields.clone(),
                        };
                        events.push(KillEvent::Write {
                            place: self.resolve(&spelled),
                            element: true,
                        });
                    }
                }
                self.apply_kills(state, &events);
                true
            }
            CheckedStatement::Evaluate(value) | CheckedStatement::DropExpression { value, .. } => {
                self.expression_effects(value, state);
                true
            }
            CheckedStatement::Check { condition, .. } => {
                self.expression_effects(condition, state);
                self.establish_passed_condition(condition, state);
                true
            }
            CheckedStatement::Claim {
                name,
                condition,
                trap,
                ..
            } => {
                self.expression_effects(condition, state);
                // [CLM-2] is judged at the claim with the state before the
                // claim's own passed fact: redundancy when the closed state
                // derives the predicate (a contradictory state derives
                // everything and never refutes), refutation when the
                // non-contradictory state derives the exact negation.
                let disposition = match self.scrutinee_relation(condition, state) {
                    None => ClaimDisposition::Retained,
                    Some(relation) => {
                        let closed = close(state, &self.terms);
                        if closed.derives(&relation) {
                            ClaimDisposition::Redundant
                        } else if closed.derives(&relation.negated()) {
                            ClaimDisposition::Refuted {
                                predicate: self.render_relation(&relation),
                                negation: self.render_relation(&relation.negated()),
                            }
                        } else {
                            ClaimDisposition::Retained
                        }
                    }
                };
                self.claims.push(ClaimOutcome {
                    node_path: trap.node_path.clone(),
                    name: name.clone(),
                    disposition,
                });
                // [ENT-3] S3: the passed predicate holds on the normal
                // continuation, exactly as S2 establishes a check's.
                self.establish_passed_condition(condition, state);
                true
            }
            CheckedStatement::Return { value, .. } => {
                self.expression_effects(value, state);
                false
            }
            CheckedStatement::Give { value, .. } => {
                self.expression_effects(value, state);
                if let Some(depth) = self.gives.last().map(|frame| frame.scope_depth) {
                    let mut exit = state.clone();
                    self.exit_scopes_to(&mut exit, depth);
                    if let Some(frame) = self.gives.last_mut() {
                        frame.gives.push(exit);
                    }
                }
                false
            }
            CheckedStatement::Break { target, .. } => {
                if let Some(position) = self.loops.iter().rposition(|frame| frame.id == *target) {
                    let depth = self.loops[position].scope_depth;
                    let mut exit = state.clone();
                    self.exit_scopes_to(&mut exit, depth);
                    self.loops[position].breaks.push(exit);
                }
                false
            }
            CheckedStatement::Match {
                scrutinee,
                enum_type,
                arms,
                ..
            } => {
                let facts = self.arm_facts(scrutinee, *enum_type, state);
                let mut exits = Vec::new();
                for arm in arms {
                    if let Some(exit) = self.walk_arm(arm, state, &facts) {
                        exits.push(exit);
                    }
                }
                if exits.is_empty() {
                    false
                } else {
                    *state = join(&exits, &self.terms);
                    true
                }
            }
            CheckedStatement::ValueMatchLet {
                binding,
                scrutinee,
                enum_type,
                arms,
                ..
            } => {
                let facts = self.arm_facts(scrutinee, *enum_type, state);
                self.gives.push(GiveFrame {
                    scope_depth: self.scopes.len(),
                    gives: Vec::new(),
                });
                for arm in arms {
                    // Every delivering path leaves by `give`; an arm's
                    // fall-through state contributes nothing [GIVE-1].
                    let _ = self.walk_arm(arm, state, &facts);
                }
                let frame = self.gives.pop();
                let gives = frame.map(|frame| frame.gives).unwrap_or_default();
                self.declare(*binding);
                if gives.is_empty() {
                    false
                } else {
                    *state = join(&gives, &self.terms);
                    true
                }
            }
            CheckedStatement::Loop { id, body, .. } => {
                // [ENT-5] no-induction loop rule: the head state is the state
                // before the loop minus every fact a continuing kill event in
                // the body may kill. The body's normal exit is this loop's
                // backedge; exits from the body are not.
                let mut kills = LoopKills::default();
                self.collect_continuing_loop_kills(
                    body,
                    true,
                    &mut LoopReachability::default(),
                    &mut kills,
                );
                self.apply_loop_kills(state, &kills);
                self.loops.push(LoopFrame {
                    id: *id,
                    scope_depth: self.scopes.len(),
                    breaks: Vec::new(),
                });
                let mut head = state.clone();
                let _ = self.walk_block(body, &mut head);
                let frame = self.loops.pop();
                let breaks = frame.map(|frame| frame.breaks).unwrap_or_default();
                // The continuation is the join over the break edges; with no
                // break it is the contradictory all-derivable state, matching
                // an unreachable-in-truth continuation the conservative graph
                // keeps reachable [ENT-5].
                *state = join(&breaks, &self.terms);
                true
            }
            CheckedStatement::Region { body, .. } => self.walk_block(body, state),
        }
    }

    /// Walks one match arm from `entry`; establishes the arm-entry facts the
    /// scrutinee admits, applies the arm's scope-exit kills on fall-through,
    /// and returns the arm-exit state when the arm reaches the continuation.
    fn walk_arm(
        &mut self,
        arm: &CheckedMatchArm,
        entry: &FactState,
        facts: &ArmFacts,
    ) -> Option<FactState> {
        let mut state = entry.clone();
        if let Some(relation) = &facts.comparison {
            // Bool arms: tag 1 is `True()`, tag 0 is `False()`; the False
            // arm takes the exact negation [ENT-3].
            if arm.tag == 1 {
                state.establish(relation);
            } else if arm.tag == 0 {
                state.establish(&relation.negated());
            }
        }
        if let Some((tag, outcome)) = &facts.outcome
            && arm.tag == *tag
        {
            self.establish_binder_fact(arm, outcome, &mut state);
        }
        self.scopes
            .push(arm.binders.iter().map(|b| b.binding).collect());
        let mut continues = true;
        for statement in &arm.body {
            if !continues {
                break;
            }
            continues = self.walk_statement(statement, &mut state);
        }
        if continues {
            let depth = self.scopes.len() - 1;
            self.exit_scopes_to(&mut state, depth);
        }
        self.scopes.pop();
        continues.then_some(state)
    }

    // ------------------------------------------------------------------
    // Loop kill summary
    // ------------------------------------------------------------------

    /// Returns whether a block entry can reach the loop head whose summary is
    /// being built. `normal_reaches` describes the containing block's normal
    /// exit. This is structural reachability over [FN-1], not an executable
    /// constant-folding judgment.
    fn loop_block_reaches(
        &self,
        statements: &[CheckedStatement],
        normal_reaches: bool,
        reachability: &mut LoopReachability,
    ) -> bool {
        let mut reaches = normal_reaches;
        for statement in statements.iter().rev() {
            reaches = self.loop_statement_reaches(statement, reaches, reachability);
        }
        reaches
    }

    fn loop_statement_reaches(
        &self,
        statement: &CheckedStatement,
        normal_reaches: bool,
        reachability: &mut LoopReachability,
    ) -> bool {
        match statement {
            CheckedStatement::Let { .. }
            | CheckedStatement::PropagateLet { .. }
            | CheckedStatement::Set { .. }
            | CheckedStatement::Evaluate(_)
            | CheckedStatement::DropExpression { .. }
            | CheckedStatement::Check { .. }
            | CheckedStatement::Claim { .. } => normal_reaches,
            CheckedStatement::Return { .. } => false,
            CheckedStatement::Give { .. } => reachability.gives.last().copied().unwrap_or(false),
            CheckedStatement::Break { target, .. } => reachability.break_reaches(*target),
            CheckedStatement::Match { arms, .. } => {
                let mut reaches = false;
                for arm in arms {
                    reaches |= self.loop_block_reaches(&arm.body, normal_reaches, reachability);
                }
                reaches
            }
            CheckedStatement::ValueMatchLet { arms, .. } => {
                // Arm fallthrough never reaches a value initializer's
                // continuation. Its `give` edges do, and nested value
                // initializers shadow this target while they are inspected.
                reachability.gives.push(normal_reaches);
                let mut reaches = false;
                for arm in arms {
                    reaches |= self.loop_block_reaches(&arm.body, false, reachability);
                }
                reachability.gives.pop();
                reaches
            }
            CheckedStatement::Loop { id, body, .. } => {
                // A nested loop body reaches its successor through its own
                // break edges, or can escape through another visible target.
                // A backedge alone cannot create reachability, so evaluating
                // the body with a false normal exit computes the least fixed
                // point. Once the body entry reaches the target, its normal
                // exit can take another iteration and eventually use that
                // same route.
                reachability.breaks.push((*id, normal_reaches));
                let body_reaches = self.loop_block_reaches(body, false, reachability);
                reachability.breaks.pop();
                // [FN-1] also keeps a conservative direct edge from the
                // nested loop statement to its normal successor. That edge
                // carries no event from inside the body.
                normal_reaches || body_reaches
            }
            CheckedStatement::Region { body, .. } => {
                self.loop_block_reaches(body, normal_reaches, reachability)
            }
        }
    }

    /// Collects exactly the kill events whose carrying edge can reach this
    /// loop's next head. The return value is the same structural entry
    /// reachability computed by [`Self::loop_block_reaches`].
    fn collect_continuing_loop_kills(
        &self,
        statements: &[CheckedStatement],
        normal_reaches: bool,
        reachability: &mut LoopReachability,
        kills: &mut LoopKills,
    ) -> bool {
        let mut reaches = normal_reaches;
        for statement in statements.iter().rev() {
            reaches =
                self.collect_continuing_statement_kills(statement, reaches, reachability, kills);
        }
        reaches
    }

    fn collect_continuing_statement_kills(
        &self,
        statement: &CheckedStatement,
        normal_reaches: bool,
        reachability: &mut LoopReachability,
        kills: &mut LoopKills,
    ) -> bool {
        match statement {
            CheckedStatement::Let { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::DropExpression { value, .. }
            | CheckedStatement::Check {
                condition: value, ..
            }
            | CheckedStatement::Claim {
                condition: value, ..
            }
            | CheckedStatement::PropagateLet {
                scrutinee: value, ..
            } => {
                if normal_reaches {
                    self.collect_expression_kills(value, &mut kills.events);
                }
                normal_reaches
            }
            CheckedStatement::Set { target, value } => {
                if normal_reaches {
                    self.collect_set_kills(target, value, kills);
                }
                normal_reaches
            }
            CheckedStatement::Return { .. } => false,
            CheckedStatement::Give { value, .. } => {
                let reaches = reachability.gives.last().copied().unwrap_or(false);
                if reaches {
                    self.collect_expression_kills(value, &mut kills.events);
                }
                reaches
            }
            CheckedStatement::Break { target, .. } => reachability.break_reaches(*target),
            CheckedStatement::Match {
                scrutinee, arms, ..
            } => {
                let mut reaches = false;
                for arm in arms {
                    reaches |= self.collect_continuing_loop_kills(
                        &arm.body,
                        normal_reaches,
                        reachability,
                        kills,
                    );
                }
                if reaches {
                    self.collect_expression_kills(scrutinee, &mut kills.events);
                }
                reaches
            }
            CheckedStatement::ValueMatchLet {
                scrutinee, arms, ..
            } => {
                reachability.gives.push(normal_reaches);
                let mut reaches = false;
                for arm in arms {
                    reaches |=
                        self.collect_continuing_loop_kills(&arm.body, false, reachability, kills);
                }
                reachability.gives.pop();
                if reaches {
                    self.collect_expression_kills(scrutinee, &mut kills.events);
                }
                reaches
            }
            CheckedStatement::Loop { id, body, .. } => {
                reachability.breaks.push((*id, normal_reaches));
                let body_reaches = self.loop_block_reaches(body, false, reachability);
                self.collect_continuing_loop_kills(body, body_reaches, reachability, kills);
                reachability.breaks.pop();
                normal_reaches || body_reaches
            }
            CheckedStatement::Region { body, .. } => {
                self.collect_continuing_loop_kills(body, normal_reaches, reachability, kills)
            }
        }
    }

    fn collect_set_kills(
        &self,
        target: &CheckedSetTarget,
        value: &CheckedExpression,
        kills: &mut LoopKills,
    ) {
        self.collect_expression_kills(value, &mut kills.events);
        match target {
            CheckedSetTarget::Place(place) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(place.binding),
                    deref: self.is_holder(place.binding),
                    fields: place.fields.clone(),
                };
                kills.events.push(KillEvent::Write {
                    place: self.resolve(&spelled),
                    element: false,
                });
                if place.fields.is_empty() {
                    kills.set_bindings.insert(place.binding);
                }
            }
            CheckedSetTarget::ArrayIndex(target) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(target.binding),
                    deref: self.is_holder(target.binding),
                    fields: target.fields.clone(),
                };
                kills.events.push(KillEvent::Write {
                    place: self.resolve(&spelled),
                    element: true,
                });
            }
            CheckedSetTarget::BufferIndex(target) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(target.root.binding),
                    deref: self.is_holder(target.root.binding),
                    fields: target.root.fields.clone(),
                };
                kills.events.push(KillEvent::Write {
                    place: self.resolve(&spelled),
                    element: true,
                });
            }
        }
    }

    fn apply_loop_kills(&self, state: &mut FactState, kills: &LoopKills) {
        state.kill(|term| {
            kills
                .events
                .iter()
                .any(|event| self.event_kills_term(term, event))
        });
        state
            .origins
            .retain(|binding, _| !kills.set_bindings.contains(binding));
        state
            .outcomes
            .retain(|binding, _| !kills.set_bindings.contains(binding));
    }

    // ------------------------------------------------------------------
    // Canonical rendering [ENT-6]
    // ------------------------------------------------------------------

    fn binding_name(&self, binding: BindingId) -> String {
        self.context
            .binding_names
            .get(binding.0 as usize)
            .cloned()
            .unwrap_or_else(|| "?".to_owned())
    }

    fn render_place(&self, place: &PlaceTerm) -> String {
        let (mut rendered, mut ty) = match place.root {
            PlaceRoot::Binding(binding) => {
                let base = if place.deref {
                    format!("deref({})", self.binding_name(binding))
                } else {
                    self.binding_name(binding)
                };
                (base, self.summary(binding).and_then(|summary| summary.ty))
            }
            PlaceRoot::Constant(id) => (
                self.context
                    .constants
                    .get(id.0 as usize)
                    .map(|constant| constant.name.clone())
                    .unwrap_or_else(|| "?".to_owned()),
                self.context
                    .constants
                    .get(id.0 as usize)
                    .map(|constant| constant.ty),
            ),
        };
        for field in &place.fields {
            let name = ty
                .and_then(|current| self.field_name(current, *field))
                .unwrap_or(None);
            match name {
                Some((field_name, field_ty)) => {
                    rendered.push('.');
                    rendered.push_str(&field_name);
                    ty = Some(field_ty);
                }
                None => {
                    rendered.push_str(".?");
                    ty = None;
                }
            }
        }
        rendered
    }

    #[allow(clippy::type_complexity)]
    fn field_name(&self, ty: CheckedType, field: u32) -> Option<Option<(String, CheckedType)>> {
        let CheckedType::Nominal(id) = ty else {
            return Some(None);
        };
        let nominal = self.context.nominals.get(id.0 as usize)?;
        let CheckedNominalKind::Struct { fields } = &nominal.kind else {
            return Some(None);
        };
        let field = fields.get(field as usize)?;
        Some(Some((field.name.clone(), field.ty)))
    }

    /// Renders one normalized relation for the [CLM-2] refutation diagnostic.
    fn render_relation(&self, relation: &Relation) -> String {
        match relation {
            Relation::Bound { left, right, bound } => format!(
                "{} - {} <= {bound}",
                self.render_term(*left),
                self.render_term(*right)
            ),
            Relation::Equal { left, right } => {
                format!("{} = {}", self.render_term(*left), self.render_term(*right))
            }
            Relation::Distinct { left, right } => {
                format!(
                    "{} != {}",
                    self.render_term(*left),
                    self.render_term(*right)
                )
            }
        }
    }

    fn render_term(&self, term: TermId) -> String {
        match self.terms.kind(term) {
            TermKind::Zero => "0".to_owned(),
            TermKind::Constant(value) => value.to_string(),
            TermKind::ConstParameter(_) => "<const parameter>".to_owned(),
            TermKind::Place(place, _) => self.render_place(place),
            TermKind::Length(place) => format!("len({})", self.render_place(place)),
        }
    }

    fn render_expression(&self, expression: &CheckedExpression) -> String {
        match expression {
            CheckedExpression::Constant(CheckedValue::Integer { ty, bits }) => {
                format!("{}_{}", integer_value(*ty, *bits), integer_type_name(*ty))
            }
            CheckedExpression::Binding { binding, .. } => self.binding_name(*binding),
            CheckedExpression::Project {
                binding, fields, ..
            } => self.render_place(&PlaceTerm {
                root: PlaceRoot::Binding(*binding),
                deref: false,
                fields: fields.clone(),
            }),
            CheckedExpression::DerefAddressed { binding, .. } => {
                format!("deref({})", self.binding_name(*binding))
            }
            CheckedExpression::BoxDeref { value, .. } => {
                format!("deref({})", self.render_expression(value))
            }
            CheckedExpression::ProjectValue {
                value,
                nominal,
                field,
                ..
            } => {
                let field_name = self
                    .context
                    .nominals
                    .get(nominal.0 as usize)
                    .and_then(|nominal| match &nominal.kind {
                        CheckedNominalKind::Struct { fields } => {
                            fields.get(*field as usize).map(|field| field.name.clone())
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| "?".to_owned());
                format!("{}.{field_name}", self.render_expression(value))
            }
            CheckedExpression::ArrayIndex { root, offset, .. } => {
                let base = self.array_root_place(root);
                format!(
                    "{}[{}]",
                    self.render_place(&base),
                    self.render_expression(offset)
                )
            }
            CheckedExpression::BufferIndex { root, offset, .. } => {
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                };
                format!(
                    "{}[{}]",
                    self.render_place(&base),
                    self.render_expression(offset)
                )
            }
            CheckedExpression::SliceIndex { root, offset, .. } => {
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: Vec::new(),
                };
                format!(
                    "{}[{}]",
                    self.render_place(&base),
                    self.render_expression(offset)
                )
            }
            _ => "?".to_owned(),
        }
    }
}

fn holder_from_value(value: &CheckedExpression) -> Option<HolderReferent> {
    match value {
        CheckedExpression::BorrowAddressed { binding, .. }
        | CheckedExpression::BorrowBox { binding, .. }
        | CheckedExpression::BorrowSystemResource { binding, .. } => Some(HolderReferent::Place {
            binding: *binding,
            fields: Vec::new(),
        }),
        CheckedExpression::BorrowBuffer { root } => Some(HolderReferent::Place {
            binding: root.binding,
            fields: root.fields.clone(),
        }),
        CheckedExpression::ReborrowAddressed { binding, .. } => {
            Some(HolderReferent::Holder(*binding))
        }
        CheckedExpression::BoxNew { .. } => Some(HolderReferent::Opaque),
        _ => None,
    }
}

/// Every direct subexpression, for uniform recursion.
fn expression_children(expression: &CheckedExpression) -> Vec<&CheckedExpression> {
    match expression {
        CheckedExpression::Constant(_)
        | CheckedExpression::Binding { .. }
        | CheckedExpression::ArrayLength { .. }
        | CheckedExpression::BufferLength { .. }
        | CheckedExpression::SliceLength { .. }
        | CheckedExpression::SliceOf { .. }
        | CheckedExpression::BorrowBuffer { .. }
        | CheckedExpression::BorrowAddressed { .. }
        | CheckedExpression::BorrowBox { .. }
        | CheckedExpression::BorrowSystemResource { .. }
        | CheckedExpression::ReborrowAddressed { .. }
        | CheckedExpression::DerefAddressed { .. }
        | CheckedExpression::Project { .. } => Vec::new(),
        CheckedExpression::UserCall { arguments, .. }
        | CheckedExpression::SystemCall { arguments, .. }
        | CheckedExpression::IntegerOperation { arguments, .. }
        | CheckedExpression::FloatOperation { arguments, .. }
        | CheckedExpression::BooleanOperation { arguments, .. }
        | CheckedExpression::EnumEquality { arguments, .. } => arguments.iter().collect(),
        CheckedExpression::NumericConversion { value, .. }
        | CheckedExpression::Reinterpret { value, .. }
        | CheckedExpression::ArrayFill { value, .. }
        | CheckedExpression::BoxNew { value, .. }
        | CheckedExpression::BoxDeref { value, .. }
        | CheckedExpression::ProjectValue { value, .. } => vec![value.as_ref()],
        CheckedExpression::ArrayIndex { offset, .. } => vec![offset.as_ref()],
        CheckedExpression::BufferFill { length, value, .. } => {
            vec![length.as_ref(), value.as_ref()]
        }
        CheckedExpression::BufferIndex { offset, .. }
        | CheckedExpression::SliceIndex { offset, .. } => vec![offset.as_ref()],
        CheckedExpression::ConstructStruct { fields, .. }
        | CheckedExpression::ConstructEnum { fields, .. } => fields.iter().collect(),
    }
}

const fn integer_type_name(ty: IntegerType) -> &'static str {
    match ty {
        IntegerType::I8 => "i8",
        IntegerType::I16 => "i16",
        IntegerType::I32 => "i32",
        IntegerType::I64 => "i64",
        IntegerType::U8 => "u8",
        IntegerType::U16 => "u16",
        IntegerType::U32 => "u32",
        IntegerType::U64 => "u64",
    }
}
