//! The words of the repair a goal's rejection carries [DIAG-1, MSR-4].
//!
//! [DIAG-1] fixes what a repair may direct and leaves its words to the
//! toolchain: carried out as it directs, each alternative lets the rejected
//! judgment succeed in a state that is not contradictory, an alternative that
//! needs a condition the checker has not decided states it, and no alternative
//! writes text a rule rejects wherever it stands. For a goal that choice turns
//! on two things the checker knows. A refuted goal is false where it stands
//! [ENT-4], so establishing it there only makes the point contradictory; its
//! repair changes what reaches the construct, or the operation or clause that
//! poses the goal. An unproved goal is established by a fact source its terms
//! admit: a `requires` on the enclosing function when every term is a
//! parameter no event on a path to the goal writes, since that fact then
//! still holds where the goal is asked; a proof whose premises the checker
//! cannot guess otherwise, which is why those routes state the condition they
//! need, the callee's `ensures` among them only when the goal reads a value a
//! call returned; or a guard where skipping the construct is the program's
//! intent. An operand
//! that is no term [ENT-2], such as an element read, admits none of these
//! until a `let` binds it, so that binding is its repair.
//!
//! The sentences live here, in one place, so that wording can follow evidence
//! from agents without touching the judgments that select them.

use std::collections::BTreeSet;

use super::super::entailment::TermRead;
use super::super::goal::{
    EvaluatedValueOccurrence, GoalDatum, GoalExpression, GoalOperation, GoalProjection,
};
use super::super::model::{
    BindingId, CheckedConversionMode, CheckedExpression, CheckedFunction, CheckedIntegerOperation,
    CheckedStatement, expression_children,
};
use super::super::permission::visit_read_bindings;
use crate::NodePath;

/// Whether the checker derived a goal false or derived neither sign [ENT-4].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Disposition {
    Refuted,
    Unproved,
}

/// What a goal's terms are, which selects the routes that can establish it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GoalTerms {
    /// Every term is a constant or a parameter of the enclosing function that
    /// no event on a path to the goal writes, at least one of them a
    /// parameter: a `requires` naming the goal holds at entry and still holds
    /// where the goal is asked.
    Parameters,
    /// The goal reads the value one argument has inside the call itself,
    /// which no fact can name [FN-8].
    CallArgument(u32),
    /// The goal reads a value only its own occurrence identifies, or a range
    /// formed where it is used: no condition has it as its goal origin and
    /// no fact names it until a `let` binds it [ENT-2, ENT-3].
    Unnamed,
    /// The goal reads a computed or local value, or an element, which a
    /// condition naming the same admitted expression establishes [ENT-3].
    Computed,
}

/// What a goal reads, as its repair needs it.
#[derive(Clone, Copy, Debug)]
pub(super) struct GoalReads {
    pub(super) terms: GoalTerms,
    /// Some read goes through a reference parameter [EFF-2].
    pub(super) referenced: bool,
    /// Some read is of a value a user call returned, which that callee's
    /// `ensures` can bound [FN-9].
    pub(super) called: bool,
}

impl GoalTerms {
    /// Classifies a concrete goal over the enclosing function's bindings,
    /// `written` being every binding some event on a path to the goal writes
    /// or consumes, sorted.
    pub(super) fn of_goal(
        goal: &GoalExpression,
        function: &CheckedFunction,
        written: &[BindingId],
    ) -> GoalReads {
        let mut reads = Reads::new(function, written);
        reads.goal(goal);
        reads.finish()
    }

    /// Classifies the terms one obligation's normalized relations read.
    pub(super) fn of_terms(
        terms: &[TermRead],
        function: &CheckedFunction,
        written: &[BindingId],
    ) -> GoalReads {
        let mut reads = Reads::new(function, written);
        for term in terms {
            match term {
                TermRead::Constant => {}
                TermRead::Binding(binding) => reads.binding(*binding),
                TermRead::Unnamed => reads.unnamed = true,
                TermRead::Computed => reads.computed = true,
            }
        }
        reads.finish()
    }
}

struct Reads<'a> {
    function: &'a CheckedFunction,
    written: &'a [BindingId],
    parameter: bool,
    computed: bool,
    unnamed: bool,
    argument: Option<u32>,
    /// Some read goes through a reference parameter, which executable code
    /// that repeats it must declare in the row [EFF-2].
    referenced: bool,
    /// Every binding a read is rooted at.
    bindings: Vec<BindingId>,
}

impl<'a> Reads<'a> {
    const fn new(function: &'a CheckedFunction, written: &'a [BindingId]) -> Self {
        Self {
            function,
            written,
            parameter: false,
            computed: false,
            unnamed: false,
            argument: None,
            referenced: false,
            bindings: Vec::new(),
        }
    }

    fn binding(&mut self, binding: BindingId) {
        self.bindings.push(binding);
        let parameter = self
            .function
            .parameters
            .iter()
            .find(|parameter| parameter.binding == binding);
        self.referenced |= parameter.is_some_and(|parameter| parameter.mode.is_reference());
        if parameter.is_some() && self.written.binary_search(&binding).is_err() {
            self.parameter = true;
        } else {
            self.computed = true;
        }
    }

    fn goal(&mut self, goal: &GoalExpression) {
        match goal {
            // An admitted element read is part of the goal's identity, which
            // a guard establishes; no clause names it as a parameter does.
            GoalExpression::Operation {
                row:
                    GoalOperation::ArrayIndex { .. }
                    | GoalOperation::BufferIndex { .. }
                    | GoalOperation::RunIndex { .. },
                arguments,
                ..
            } => {
                self.computed = true;
                for argument in arguments {
                    self.goal(argument);
                }
            }
            // A measure is a term of the place it measures [MSR-1], which a
            // clause spells when every step of that place is spelled.
            GoalExpression::Operation {
                row:
                    GoalOperation::ContainerMeasure { .. }
                    | GoalOperation::ArrayMeasure { .. }
                    | GoalOperation::BufferMeasure { .. },
                arguments,
                ..
            } => {
                for argument in arguments {
                    self.measured(argument);
                }
            }
            GoalExpression::Operation { arguments, .. } => {
                for argument in arguments {
                    self.goal(argument);
                }
            }
            GoalExpression::Datum(datum) => self.datum(datum),
        }
    }

    fn datum(&mut self, datum: &GoalDatum) {
        match datum {
            GoalDatum::Literal(_) => {}
            GoalDatum::NamedConst { projections, .. } if spelled(projections) => {}
            GoalDatum::Place {
                root, projections, ..
            } if spelled(projections) => self.binding(*root),
            GoalDatum::EvaluatedValue {
                occurrence: EvaluatedValueOccurrence::CallArgument { argument, .. },
                ..
            } => {
                self.argument.get_or_insert(*argument);
            }
            // A value only its occurrence identifies has no goal origin
            // [ENT-3], and a range formed at its use is no measure place
            // [ENT-2].
            GoalDatum::EvaluatedValue { .. } => self.unnamed = true,
            GoalDatum::NamedConst { projections, .. } | GoalDatum::Place { projections, .. }
                if ranged(projections) =>
            {
                self.unnamed = true;
            }
            _ => self.computed = true,
        }
    }

    /// The place a measure reads. A measure of an element is a term
    /// [MSR-1], but its offset is captured where the place is formed and no
    /// clause can name that capture, so it counts as computed.
    fn measured(&mut self, place: &GoalExpression) {
        match place {
            GoalExpression::Datum(GoalDatum::NamedConst { projections, .. })
                if spelled(projections) => {}
            GoalExpression::Datum(GoalDatum::Place {
                root, projections, ..
            }) if spelled(projections) => self.binding(*root),
            GoalExpression::Datum(
                datum @ GoalDatum::EvaluatedValue {
                    occurrence: EvaluatedValueOccurrence::CallArgument { .. },
                    ..
                },
            ) => self.datum(datum),
            GoalExpression::Datum(GoalDatum::Place { projections, .. }) if ranged(projections) => {
                self.unnamed = true;
            }
            _ => self.computed = true,
        }
    }

    fn finish(self) -> GoalReads {
        let terms = match self {
            Self {
                argument: Some(argument),
                ..
            } => GoalTerms::CallArgument(argument),
            Self { unnamed: true, .. } => GoalTerms::Unnamed,
            Self {
                parameter: true,
                computed: false,
                ..
            } => GoalTerms::Parameters,
            _ => GoalTerms::Computed,
        };
        let results = call_results(self.function);
        GoalReads {
            terms,
            referenced: self.referenced,
            called: self
                .bindings
                .iter()
                .any(|binding| results.contains(binding)),
        }
    }
}

/// The bindings of `function` whose value a user call returned, directly or
/// through local computation from such a value: the values a callee's
/// `ensures` can bound [FN-9]. The closure ignores control flow, which only
/// widens it.
fn call_results(function: &CheckedFunction) -> BTreeSet<BindingId> {
    let mut definitions = Vec::new();
    if let Some(body) = &function.body {
        collect_definitions(body, None, &mut definitions);
    }
    let mut results: BTreeSet<BindingId> = definitions
        .iter()
        .filter(|definition| definition.call)
        .map(|definition| definition.binding)
        .collect();
    loop {
        let before = results.len();
        for definition in &definitions {
            if definition.reads.iter().any(|read| results.contains(read)) {
                results.insert(definition.binding);
            }
        }
        if results.len() == before {
            return results;
        }
    }
}

/// One value a statement gives a binding: the bindings it reads, and whether
/// a user call computes it.
struct Definition {
    binding: BindingId,
    reads: Vec<BindingId>,
    call: bool,
}

impl Definition {
    fn of(binding: BindingId, values: &[&CheckedExpression]) -> Self {
        let mut reads = Vec::new();
        for value in values {
            visit_read_bindings(value, &mut |read| reads.push(read));
        }
        Self {
            binding,
            reads,
            call: values.iter().any(|value| calls(value)),
        }
    }
}

/// Whether an expression tree contains a user call.
fn calls(expression: &CheckedExpression) -> bool {
    matches!(expression, CheckedExpression::UserCall { .. })
        || expression_children(expression).into_iter().any(calls)
}

/// Every value the statements give a binding, `give` naming the binding a
/// value initializer's `give` delivers to.
fn collect_definitions(
    statements: &[CheckedStatement],
    give: Option<BindingId>,
    definitions: &mut Vec<Definition>,
) {
    for statement in statements {
        match statement {
            CheckedStatement::Let { binding, value, .. }
            | CheckedStatement::PropagateLet {
                binding,
                scrutinee: value,
                ..
            } => definitions.push(Definition::of(*binding, &[value])),
            CheckedStatement::DestructuringLet {
                bindings, value, ..
            } => {
                for (binding, _, _) in bindings {
                    definitions.push(Definition::of(*binding, &[value]));
                }
            }
            CheckedStatement::Set { target, value, .. } => {
                definitions.push(Definition::of(target.binding(), &[value]));
            }
            CheckedStatement::Give { value, .. } => {
                if let Some(binding) = give {
                    definitions.push(Definition::of(binding, &[value]));
                }
            }
            CheckedStatement::Match {
                scrutinee, arms, ..
            } => {
                for arm in arms {
                    for binder in &arm.binders {
                        definitions.push(Definition::of(binder.binding, &[scrutinee]));
                    }
                    collect_definitions(&arm.body, give, definitions);
                }
            }
            CheckedStatement::ValueMatchLet {
                binding,
                scrutinee,
                arms,
                ..
            } => {
                for arm in arms {
                    for binder in &arm.binders {
                        definitions.push(Definition::of(binder.binding, &[scrutinee]));
                    }
                    collect_definitions(&arm.body, Some(*binding), definitions);
                }
            }
            CheckedStatement::Loop { body, .. } => collect_definitions(body, give, definitions),
            CheckedStatement::CountedRange {
                binder,
                lower,
                upper,
                body,
                ..
            } => {
                definitions.push(Definition::of(*binder, &[lower, upper]));
                collect_definitions(body, give, definitions);
            }
            CheckedStatement::Evaluate { .. }
            | CheckedStatement::DropExpression { .. }
            | CheckedStatement::Proof(_)
            | CheckedStatement::Return { .. }
            | CheckedStatement::Break { .. } => {}
        }
    }
}

/// Whether the value the `return` at `statement` delivers is, or reads, a
/// value a user call returned, which the callee's `ensures` can bound [FN-9].
pub(super) fn returns_call_result(function: &CheckedFunction, statement: &NodePath) -> bool {
    let results = call_results(function);
    function
        .body
        .as_deref()
        .and_then(|body| returned_value(body, statement))
        .is_some_and(|value| {
            let mut read = false;
            visit_read_bindings(value, &mut |binding| read |= results.contains(&binding));
            read || calls(value)
        })
}

/// The value of the `return` at `path`, wherever it is nested.
fn returned_value<'a>(
    statements: &'a [CheckedStatement],
    path: &NodePath,
) -> Option<&'a CheckedExpression> {
    statements.iter().find_map(|statement| match statement {
        CheckedStatement::Return {
            node_path, value, ..
        } => (node_path == path).then_some(value),
        CheckedStatement::Match { arms, .. } | CheckedStatement::ValueMatchLet { arms, .. } => {
            arms.iter().find_map(|arm| returned_value(&arm.body, path))
        }
        CheckedStatement::Loop { body, .. } | CheckedStatement::CountedRange { body, .. } => {
            returned_value(body, path)
        }
        _ => None,
    })
}

/// Field selections and `deref` are the steps a clause spells as written.
fn spelled(projections: &[GoalProjection]) -> bool {
    projections
        .iter()
        .all(|projection| matches!(projection, GoalProjection::Deref | GoalProjection::Field(_)))
}

/// A range step, which a goal carries only for a range formed at its use.
fn ranged(projections: &[GoalProjection]) -> bool {
    projections
        .iter()
        .any(|projection| matches!(projection, GoalProjection::Range(_)))
}

/// Whether a goal is one relation over atoms, so that its rendering is the
/// source text of a condition or a `requires` clause [GRAM-9].
pub(super) fn is_source_relation(goal: &GoalExpression) -> bool {
    let GoalExpression::Operation { row, arguments, .. } = goal else {
        return false;
    };
    let relation = match row {
        GoalOperation::Integer { operation, .. } => matches!(
            operation,
            CheckedIntegerOperation::Equal
                | CheckedIntegerOperation::NotEqual
                | CheckedIntegerOperation::Less
                | CheckedIntegerOperation::LessEqual
                | CheckedIntegerOperation::Greater
                | CheckedIntegerOperation::GreaterEqual
                | CheckedIntegerOperation::AddDefined
                | CheckedIntegerOperation::SubtractDefined
                | CheckedIntegerOperation::MultiplyDefined
                | CheckedIntegerOperation::DivideDefined
                | CheckedIntegerOperation::RemainderDefined
                | CheckedIntegerOperation::NegateDefined
                | CheckedIntegerOperation::AbsoluteDefined
                | CheckedIntegerOperation::ShiftLeftDefined
                | CheckedIntegerOperation::ShiftRightDefined
        ),
        GoalOperation::NumericConversion { mode, .. } => *mode == CheckedConversionMode::Defined,
        _ => false,
    };
    relation && arguments.iter().all(is_source_atom)
}

fn is_source_atom(expression: &GoalExpression) -> bool {
    match expression {
        GoalExpression::Datum(GoalDatum::Literal(_)) => true,
        GoalExpression::Datum(
            GoalDatum::NamedConst { projections, .. } | GoalDatum::Place { projections, .. },
        ) => spelled(projections),
        // A measure, and an admitted element read, render as the place a
        // source atom writes.
        GoalExpression::Operation {
            row:
                GoalOperation::ContainerMeasure { .. }
                | GoalOperation::ArrayMeasure { .. }
                | GoalOperation::BufferMeasure { .. }
                | GoalOperation::ArrayIndex { .. }
                | GoalOperation::BufferIndex { .. }
                | GoalOperation::RunIndex { .. },
            arguments,
            ..
        } => arguments.iter().all(is_source_atom),
        _ => false,
    }
}

/// The total spellings [OP-2]'s mode table gives an exact operation whose
/// canonical goal is this `.defined` query.
pub(super) fn total_forms(goal: &GoalExpression) -> Option<&'static str> {
    let GoalExpression::Operation {
        row: GoalOperation::Integer { operation, .. },
        ..
    } = goal
    else {
        return None;
    };
    Some(match operation {
        CheckedIntegerOperation::AddDefined => "`+wrap`, `+checked` or `+sat`",
        CheckedIntegerOperation::SubtractDefined => "`-wrap`, `-checked` or `-sat`",
        CheckedIntegerOperation::MultiplyDefined => "`*wrap`, `*checked` or `*sat`",
        CheckedIntegerOperation::DivideDefined => "`/checked`",
        CheckedIntegerOperation::RemainderDefined => "`%checked`",
        CheckedIntegerOperation::NegateDefined => "`ineg.wrap` or `ineg.checked`",
        CheckedIntegerOperation::AbsoluteDefined => "`iabs.wrap` or `iabs.checked`",
        CheckedIntegerOperation::ShiftLeftDefined => "`ishl.wrap`",
        CheckedIntegerOperation::ShiftRightDefined => "`ishr.wrap`",
        _ => return None,
    })
}

/// One rejected goal: its disposition, what its terms are, its source text
/// when that text is a condition, and the enclosing function a `requires`
/// would be added to.
pub(super) struct GoalCase<'a> {
    pub(super) disposition: Disposition,
    pub(super) terms: GoalTerms,
    /// The goal reads through a reference parameter [EFF-2].
    pub(super) referenced: bool,
    /// The goal reads a value a user call returned [FN-9].
    pub(super) called: bool,
    /// The goal as the payload renders it.
    pub(super) text: &'a str,
    /// Whether `text` is the source of one condition over atoms.
    pub(super) condition: bool,
    pub(super) function: &'a str,
}

impl GoalCase<'_> {
    /// `requires` on the enclosing function, and the guard; the two routes
    /// every unproved goal over unwritten parameters has.
    fn parameter_routes(&self, construct: &str, intent: &str) -> String {
        let guard = self.guard(construct, intent);
        if self.condition {
            format!(
                "add `requires {};` to the `contract` of `{}`, which each caller then establishes; or {guard}",
                self.text, self.function,
            )
        } else {
            format!(
                "state the relation over the parameters of `{}` as a `requires` in its `contract`, which each caller then establishes; or {guard}",
                self.function,
            )
        }
    }

    /// The guard alternative, for a program whose intent is to skip the
    /// construct when the goal fails. Unlike a clause or an invariant, a
    /// condition is executable code, so a read it makes through a reference
    /// parameter is one the row must declare [EFF-2].
    fn guard(&self, construct: &str, intent: &str) -> String {
        let condition = if self.condition {
            format!("`if {}`", self.text)
        } else {
            String::from("an `if` whose condition establishes it")
        };
        let row = if self.referenced {
            ", adding to the effect row any read that condition makes which the row does not yet declare"
        } else {
            ""
        };
        format!("guard the {construct} with {condition} {intent}{row}")
    }

    /// The route of an unproved goal that reads a value no fact names: once
    /// a `let` binds that value the goal is over a term, and the
    /// rejection that follows names the routes for it.
    fn unnamed_route(&self, construct: &str) -> String {
        format!(
            "`{}` reads a value no fact can name until a `let` binds it: bind that value with one preceding `let`, use the binding in the {construct}, and establish the relation over the binding",
            self.text
        )
    }

    /// The proof routes of an unproved goal over computed values. Each
    /// states the condition it needs, which the checker cannot decide: a
    /// written certificate or a callee relation the program does not have yet.
    fn computed_routes(&self, construct: &str, intent: &str) -> String {
        let guard = self.guard(construct, intent);
        // A callee's `ensures` bounds only a value that callee returned.
        let callee = if self.called {
            "; when the callee whose result it reads can prove the bound, state it in that callee's `ensures`"
        } else {
            ""
        };
        format!(
            "when facts that reach the {construct} imply it, prove it with an `invariant` whose `use` steps name them (a loop's header `invariant` for a value the loop computes){callee}; or {guard}"
        )
    }
}

const SKIP: &str = "where skipping it is the intended behavior";

/// [FN-8] an ordinary call's requirement.
pub(super) fn call_requirement(case: &GoalCase<'_>) -> String {
    match (case.disposition, case.terms) {
        (Disposition::Refuted, _) => format!(
            "`{}` is false for the values that reach this call, so no fact can establish it here: pass arguments that satisfy it, or change the statements or requirements that fix those values",
            case.text
        ),
        (Disposition::Unproved, GoalTerms::CallArgument(argument)) => format!(
            "argument #{argument} is evaluated inside the call, where no fact names its value: bind it with one preceding `let`, establish the requirement over that binding, and pass the binding, borrowing it when the parameter is a reference"
        ),
        (Disposition::Unproved, GoalTerms::Unnamed) => case.unnamed_route("call"),
        (Disposition::Unproved, GoalTerms::Parameters) => case.parameter_routes("call", SKIP),
        (Disposition::Unproved, GoalTerms::Computed) => format!(
            "`{}` is not proved before this call: {}",
            case.text,
            case.computed_routes("call", SKIP)
        ),
    }
}

/// [FN-9] a normal-result relation at one selected return; `called` says
/// whether the returned value reads a value a user call returned.
pub(super) fn postcondition(disposition: Disposition, called: bool) -> &'static str {
    match (disposition, called) {
        (Disposition::Refuted, _) => {
            "the value this `return` delivers makes the postcondition false: return a value that satisfies it, state a postcondition this return satisfies, or change the requirements that fix the returned value"
        }
        (Disposition::Unproved, true) => {
            "the postcondition is not proved where this `return` delivers its value: add a `requires` over the parameters the value is computed from, prove the bound before the return with an `invariant` whose `use` steps name the facts it follows from, state it in the `ensures` of the callee whose result the value reads when that callee can prove it, or state a postcondition the body proves"
        }
        (Disposition::Unproved, false) => {
            "the postcondition is not proved where this `return` delivers its value: add a `requires` over the parameters the value is computed from, prove the bound before the return with an `invariant` whose `use` steps name the facts it follows from, or state a postcondition the body proves"
        }
    }
}

/// [FN-9] a clause whose route no normal return selects.
pub(super) const NO_SELECTED_EXIT: &str = "no `return` of this function delivers a value this clause's route selects: return such a value on some path, or delete the clause";

/// [OP-2] an exact integer operation's `.defined` domain.
pub(super) fn integer_domain(case: &GoalCase<'_>, forms: Option<&str>) -> String {
    let total = forms.map_or_else(String::new, |forms| format!("; or write the {forms} form"));
    match (case.disposition, case.terms) {
        (Disposition::Refuted, _) => {
            let total = forms.map_or_else(String::new, |forms| {
                format!(", or write the {forms} form for the result the program intends")
            });
            format!(
                "the operands that reach this operation make `{}` false, so the exact operation cannot execute here: change the operands or their type{total}",
                case.text
            )
        }
        (Disposition::Unproved, GoalTerms::Parameters) => {
            format!("{}{total}", case.parameter_routes("operation", SKIP))
        }
        (Disposition::Unproved, GoalTerms::Unnamed | GoalTerms::CallArgument(_)) => {
            format!("{}{total}", case.unnamed_route("operation"))
        }
        (Disposition::Unproved, GoalTerms::Computed) => format!(
            "`{}` is not proved here: {}{total}",
            case.text,
            case.computed_routes("operation", SKIP)
        ),
    }
}

/// [OP-6] a bare conversion's domain. `checked` spells the total form and
/// `integer_source` whether an affine invariant can bound the operand.
pub(super) fn conversion_domain(
    case: &GoalCase<'_>,
    checked: &str,
    destination: &str,
    integer_source: bool,
) -> String {
    let fallible = format!("use `{checked}` and handle its `Err`");
    match (case.disposition, case.terms) {
        (Disposition::Refuted, _) => format!(
            "the value that reaches this conversion is outside `{destination}`: convert a value `{destination}` holds, choose a destination type that holds this one, or {fallible}"
        ),
        (Disposition::Unproved, GoalTerms::Parameters) => {
            format!(
                "{}; or {fallible}",
                case.parameter_routes("conversion", SKIP)
            )
        }
        (Disposition::Unproved, GoalTerms::Unnamed | GoalTerms::CallArgument(_)) => {
            format!("{}; or {fallible}", case.unnamed_route("conversion"))
        }
        (Disposition::Unproved, _) if integer_source => format!(
            "`{}` is not proved here: {}; or {fallible}",
            case.text,
            case.computed_routes("conversion", SKIP)
        ),
        (Disposition::Unproved, _) => format!(
            "`{}` is not proved here, and no fact bounds a float operand: {}, or {fallible}",
            case.text,
            case.guard("conversion", SKIP)
        ),
    }
}

/// [OP-4] a subscript's bound.
pub(super) fn bounds(case: &GoalCase<'_>) -> String {
    match (case.disposition, case.terms) {
        (Disposition::Refuted, _) => format!(
            "`{}` is false where this access executes: index within the storage, or give the storage a length that holds this index",
            case.text
        ),
        (Disposition::Unproved, GoalTerms::Parameters) => case.parameter_routes("access", SKIP),
        (Disposition::Unproved, GoalTerms::Unnamed | GoalTerms::CallArgument(_)) => {
            case.unnamed_route("access")
        }
        (Disposition::Unproved, GoalTerms::Computed) => format!(
            "`{}` is not proved here: {}",
            case.text,
            case.computed_routes("access", SKIP)
        ),
    }
}

/// [OP-9] an allocation's size.
pub(super) fn allocation_fit(case: &GoalCase<'_>) -> String {
    const REFUSE: &str = "where refusing a larger count is the intended behavior";
    match (case.disposition, case.terms) {
        (Disposition::Refuted, _) => format!(
            "`{}` is false, so this allocation cannot be formed: request a count within that bound",
            case.text
        ),
        (Disposition::Unproved, GoalTerms::Parameters) => {
            case.parameter_routes("allocation", REFUSE)
        }
        (Disposition::Unproved, GoalTerms::Unnamed | GoalTerms::CallArgument(_)) => {
            case.unnamed_route("allocation")
        }
        (Disposition::Unproved, GoalTerms::Computed) => format!(
            "`{}` is not proved here: {}",
            case.text,
            case.computed_routes("allocation", REFUSE)
        ),
    }
}

/// [REF-4] one range-formation conjunct.
pub(super) fn range_formation(case: &GoalCase<'_>) -> String {
    match (case.disposition, case.terms) {
        (Disposition::Refuted, _) => format!(
            "`{}` is false where this range is formed: choose endpoints that satisfy it",
            case.text
        ),
        (Disposition::Unproved, GoalTerms::Parameters) => case.parameter_routes("range", SKIP),
        (Disposition::Unproved, GoalTerms::Unnamed | GoalTerms::CallArgument(_)) => {
            case.unnamed_route("range")
        }
        (Disposition::Unproved, GoalTerms::Computed) => format!(
            "`{}` is not proved here: {}",
            case.text,
            case.computed_routes("range", SKIP)
        ),
    }
}

/// [OP-14] `free_empty`'s requirement that the window is empty. Skipping the
/// release is no repair: a window left alive is released at its scope exit,
/// which owes the same empty proof [PROV-6].
pub(super) fn empty_run_release(case: &GoalCase<'_>) -> String {
    const EMPTY: &str = "take every element out and consume it before this call, so that its zero length is established here";
    match (case.disposition, case.terms) {
        (Disposition::Refuted, _) => String::from(
            "the window still holds elements here: take every element out and consume it before `free_empty`",
        ),
        (Disposition::Unproved, GoalTerms::Parameters) if case.condition => format!(
            "add `requires {};` to the `contract` of `{}`, which each caller then establishes, or {EMPTY}",
            case.text, case.function
        ),
        (Disposition::Unproved, GoalTerms::Parameters) => format!(
            "state the window's zero length over the parameters of `{}` as a `requires` in its `contract`, which each caller then establishes, or {EMPTY}",
            case.function
        ),
        (Disposition::Unproved, _) => format!("`{}` is not proved here: {EMPTY}", case.text),
    }
}

/// [INV-1] a blockless local invariant's target.
pub(super) fn local_invariant(disposition: Disposition, name: &str) -> String {
    match disposition {
        Disposition::Refuted => format!(
            "`{name}` is false where it is stated: correct the relation, or state one that the facts reaching it imply"
        ),
        Disposition::Unproved => format!(
            "`{name}` is not proved from the facts that reach it: establish them before it, add `use` steps naming the facts it follows from, or weaken it"
        ),
    }
}

/// [INV-1] a loop invariant's base judgment on entry to the loop.
pub(super) fn loop_invariant_base(disposition: Disposition, name: &str) -> String {
    match disposition {
        Disposition::Refuted => format!(
            "`{name}` is false on entry to the loop: correct it, or change the values the loop starts from"
        ),
        Disposition::Unproved => format!(
            "`{name}` is not proved on entry to the loop: establish before the loop the facts it follows from, or weaken or correct it"
        ),
    }
}

/// [INV-1] a loop invariant's backedge judgment at the next header.
pub(super) fn loop_invariant_backedge(disposition: Disposition, name: &str) -> String {
    match disposition {
        Disposition::Refuted => format!(
            "an iteration makes `{name}` false at the next loop header: correct it, or change the body so that every iteration preserves it"
        ),
        Disposition::Unproved => format!(
            "`{name}` is not proved preserved at the next loop header: strengthen the invariant prefix, weaken or correct it, or establish in the body the facts from which every reachable fallthrough preserves it"
        ),
    }
}
