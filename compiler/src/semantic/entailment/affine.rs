//! Deterministic affine arithmetic for the normal semantic checker.
//!
//! This module normalizes checked affine expressions, carries exact affine
//! value forms, constructs canonical inequalities, and applies the fixed
//! coefficient-one residual and interval rules used by semantic checking. It
//! performs no heuristic search, every `i128` operation is checked, and every
//! operation is charged against compiler-owned work limits.

/// Dense identity of one term in a semantic-checker-owned affine vocabulary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct AffineTermId(u32);

impl AffineTermId {
    pub(crate) const fn from_index(index: u32) -> Self {
        Self(index)
    }

    pub(crate) const fn index(self) -> u32 {
        self.0
    }
}

/// Minimal resolved expression accepted by the affine normalizer.
///
/// Source typing, name resolution, and invariant-scope checks happen before
/// this value is built. Consequently the arithmetic core needs no binding
/// table or program-point authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AffineExpression {
    Constant(i128),
    Term(AffineTermId),
    Add(Box<AffineExpression>, Box<AffineExpression>),
    Subtract(Box<AffineExpression>, Box<AffineExpression>),
    MultiplyByConstant {
        constant: i128,
        value: Box<AffineExpression>,
    },
}

/// One nonzero coefficient in a canonical affine left-hand side.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct AffineCoefficient {
    term: AffineTermId,
    coefficient: i128,
}

/// One exact mathematical value carried by the normal semantic flow.
///
/// `constant + sum(coefficient * term)` is canonical: term identities are
/// strictly ordered and zero coefficients are absent.  A form records what a
/// source value equals at the current program point; it is not a proposition
/// and grants no proof authority by itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AffineForm {
    terms: Box<[AffineCoefficient]>,
    constant: i128,
}

impl AffineForm {
    pub(crate) fn constant(value: i128) -> Self {
        Self {
            terms: Box::new([]),
            constant: value,
        }
    }

    pub(crate) fn term(term: AffineTermId) -> Self {
        Self {
            terms: Box::new([AffineCoefficient {
                term,
                coefficient: 1,
            }]),
            constant: 0,
        }
    }

    pub(crate) fn terms(&self) -> &[AffineCoefficient] {
        &self.terms
    }

    pub(crate) const fn constant_value(&self) -> i128 {
        self.constant
    }

    /// Returns the canonical nonconstant coefficient vector with a zero
    /// constant. Structural joins use this only after every incoming form has
    /// the same `terms()` slice; the differing constants are represented by
    /// one separately bounded delta atom.
    pub(crate) fn nonconstant_part(&self) -> Self {
        Self {
            terms: self.terms.clone(),
            constant: 0,
        }
    }

    pub(crate) fn unit_term(&self) -> Option<AffineTermId> {
        let [coefficient] = self.terms.as_ref() else {
            return None;
        };
        (self.constant == 0 && coefficient.coefficient == 1).then_some(coefficient.term)
    }

    pub(crate) fn add(
        &self,
        right: &Self,
        check: &mut AffineCheckState,
    ) -> Result<Self, AffineCheckError> {
        Ok(Self {
            terms: merge_scaled(self.terms(), right.terms(), 1, check)?.into_boxed_slice(),
            constant: checked_add(self.constant, right.constant)?,
        })
    }

    pub(crate) fn subtract(
        &self,
        right: &Self,
        check: &mut AffineCheckState,
    ) -> Result<Self, AffineCheckError> {
        Ok(Self {
            terms: merge_scaled(self.terms(), right.terms(), -1, check)?.into_boxed_slice(),
            constant: checked_sub(self.constant, right.constant)?,
        })
    }

    pub(crate) fn scale(
        &self,
        factor: i128,
        check: &mut AffineCheckState,
    ) -> Result<Self, AffineCheckError> {
        if factor == 0 {
            check.charge(1)?;
            return Ok(Self::constant(0));
        }
        Ok(Self {
            terms: merge_scaled(&[], self.terms(), factor, check)?.into_boxed_slice(),
            constant: checked_mul(self.constant, factor)?,
        })
    }
}

impl AffineCoefficient {
    pub(crate) const fn term(self) -> AffineTermId {
        self.term
    }

    pub(crate) const fn coefficient(self) -> i128 {
        self.coefficient
    }
}

/// Canonical `sum(coefficient * term) <= upper`.
///
/// Terms are strictly ordered by identity and zero coefficients are removed.
/// Private fields ensure all values pass the same checked canonicalization.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct AffineInequality {
    terms: Box<[AffineCoefficient]>,
    upper: i128,
}

impl AffineInequality {
    pub(crate) fn from_terms(
        terms: &[(AffineTermId, i128)],
        upper: i128,
        check: &mut AffineCheckState,
    ) -> Result<Self, AffineCheckError> {
        if terms.len() > check.limits.max_input_terms {
            return Err(AffineCheckError::LimitExceeded(
                AffineCheckLimit::InputTerms,
            ));
        }
        let mut canonical = Vec::new();
        for &(term, coefficient) in terms {
            check.charge(1)?;
            insert_coefficient(&mut canonical, term, coefficient, check)?;
        }
        Ok(Self {
            terms: canonical.into_boxed_slice(),
            upper,
        })
    }

    pub(crate) fn terms(&self) -> &[AffineCoefficient] {
        &self.terms
    }

    pub(crate) const fn upper(&self) -> i128 {
        self.upper
    }

    /// Forms the proposition `left <= right` from two exact current values.
    pub(crate) fn from_forms(
        left: &AffineForm,
        right: &AffineForm,
        check: &mut AffineCheckState,
    ) -> Result<Self, AffineCheckError> {
        let difference = left.subtract(right, check)?;
        Ok(Self {
            terms: difference.terms,
            upper: checked_neg(difference.constant)?,
        })
    }

    /// Forms the proposition `left - right <= bound` from two exact current
    /// values and one already-established ordinary difference bound.
    pub(crate) fn from_bounded_forms(
        left: &AffineForm,
        right: &AffineForm,
        bound: i128,
        check: &mut AffineCheckState,
    ) -> Result<Self, AffineCheckError> {
        let mut inequality = Self::from_forms(left, right, check)?;
        inequality.upper = checked_add(inequality.upper, bound)?;
        Ok(inequality)
    }

    /// Removes one already-established premise with coefficient one.
    ///
    /// If interval facts prove the returned inequality, adding `premise`
    /// proves `target`. The semantic checker deliberately performs no
    /// coefficient search: if this fixed coefficient-one rule does not close
    /// the target, this route has not proved it.
    pub(crate) fn residual_after(
        target: &Self,
        premise: &Self,
        check: &mut AffineCheckState,
    ) -> Result<Self, AffineCheckError> {
        Self::residual_after_scaled(target, premise, 1, check)
    }

    /// Removes one already-established premise multiplied by the written
    /// mathematical factor selected by the deterministic residual rule.
    pub(crate) fn residual_after_scaled(
        target: &Self,
        premise: &Self,
        factor: i128,
        check: &mut AffineCheckState,
    ) -> Result<Self, AffineCheckError> {
        Ok(Self {
            terms: merge_scaled(target.terms(), premise.terms(), checked_neg(factor)?, check)?
                .into_boxed_slice(),
            upper: checked_sub(target.upper, checked_mul(premise.upper, factor)?)?,
        })
    }
}

/// Selects the greatest positive integer factor that cancels at least one
/// same-sign coefficient without crossing zero in any same-sign overlap.
/// `None` means that this premise cannot make the current residual smaller by
/// the fixed rule. Terms are visited once in canonical identity order.
pub(crate) fn maximum_safe_residual_factor(
    residual: &AffineInequality,
    premise: &AffineInequality,
    check: &mut AffineCheckState,
) -> Result<Option<i128>, AffineCheckError> {
    let mut residual_index = 0;
    let mut premise_index = 0;
    let mut factor = None::<u128>;
    while residual_index < residual.terms().len() && premise_index < premise.terms().len() {
        check.charge(1)?;
        let residual_term = residual.terms()[residual_index];
        let premise_term = premise.terms()[premise_index];
        match residual_term.term().cmp(&premise_term.term()) {
            std::cmp::Ordering::Less => residual_index += 1,
            std::cmp::Ordering::Greater => premise_index += 1,
            std::cmp::Ordering::Equal => {
                if residual_term.coefficient().signum() == premise_term.coefficient().signum() {
                    let candidate = residual_term.coefficient().unsigned_abs()
                        / premise_term.coefficient().unsigned_abs();
                    factor = Some(factor.map_or(candidate, |current| current.min(candidate)));
                }
                residual_index += 1;
                premise_index += 1;
            }
        }
    }
    let Some(factor) = factor.filter(|factor| *factor != 0) else {
        return Ok(None);
    };
    Ok(i128::try_from(factor).ok())
}

/// The well-founded residual measure is the lexicographic vector of absolute
/// coefficients over the fixed affine term universe. This checks whether one
/// candidate is strictly smaller without materializing the dense vector.
pub(crate) fn residual_measure_decreases(
    before: &AffineInequality,
    after: &AffineInequality,
    check: &mut AffineCheckState,
) -> Result<bool, AffineCheckError> {
    let mut before_index = 0;
    let mut after_index = 0;
    while before_index < before.terms().len() || after_index < after.terms().len() {
        check.charge(1)?;
        let before_term = before.terms().get(before_index).copied();
        let after_term = after.terms().get(after_index).copied();
        let next = match (before_term, after_term) {
            (Some(before), Some(after)) => before.term().min(after.term()),
            (Some(before), None) => before.term(),
            (None, Some(after)) => after.term(),
            (None, None) => break,
        };
        let before_magnitude = before_term
            .filter(|term| term.term() == next)
            .map_or(0, |term| term.coefficient().unsigned_abs());
        let after_magnitude = after_term
            .filter(|term| term.term() == next)
            .map_or(0, |term| term.coefficient().unsigned_abs());
        if before_term.is_some_and(|term| term.term() == next) {
            before_index += 1;
        }
        if after_term.is_some_and(|term| term.term() == next) {
            after_index += 1;
        }
        if before_magnitude != after_magnitude {
            return Ok(after_magnitude < before_magnitude);
        }
    }
    Ok(false)
}

/// Verifies one author-selected affine certificate by summing its premises in
/// exactly the supplied order and comparing the canonical result with
/// `target`.
///
/// The caller is responsible for resolving the written premise names and for
/// preserving their source order. This core neither selects nor reorders a
/// premise, guesses a coefficient, searches for a subset, nor weakens the
/// target. Every premise has the fixed coefficient one. Success means that
/// both the accumulated coefficient vector and accumulated upper bound equal
/// the target exactly.
pub(crate) fn verify_explicit_inequality_sum(
    premises: &[AffineInequality],
    target: &AffineInequality,
    check: &mut AffineCheckState,
) -> Result<(), AffineCheckError> {
    if premises.len() > check.limits.max_certificate_premises {
        return Err(AffineCheckError::LimitExceeded(
            AffineCheckLimit::CertificatePremises,
        ));
    }

    let mut terms = Vec::new();
    let mut upper = 0_i128;
    for premise in premises {
        terms = merge_scaled(&terms, premise.terms(), 1, check)?;
        upper = checked_add(upper, premise.upper())?;
    }

    check.charge(1)?;
    if upper != target.upper() {
        return Err(AffineCheckError::CoefficientMismatch);
    }
    check.charge(1)?;
    if terms.len() != target.terms().len() {
        return Err(AffineCheckError::CoefficientMismatch);
    }
    for (formed, expected) in terms.iter().zip(target.terms()) {
        check.charge(1)?;
        if formed != expected {
            return Err(AffineCheckError::CoefficientMismatch);
        }
    }
    Ok(())
}

/// Uses one independently known inclusive interval per affine term.  This is
/// the fixed interval rule: positive coefficients select the upper endpoint,
/// negative coefficients select the lower endpoint.  `false` means only that
/// this rule did not prove the proposition; it never means the proposition is
/// false.
pub(crate) fn interval_proves(
    inequality: &AffineInequality,
    interval: impl FnMut(AffineTermId) -> Option<(i128, i128)>,
    check: &mut AffineCheckState,
) -> Result<bool, AffineCheckError> {
    let Some(maximum) = interval_maximum(inequality.terms(), interval, check)? else {
        return Ok(false);
    };
    check.charge(1)?;
    Ok(maximum <= inequality.upper())
}

/// Computes the greatest value of one affine term sum from independently
/// known inclusive intervals. This is the numeric half of
/// [`interval_proves`], exposed so a finite domain rule can derive an exact
/// operand endpoint before checking a fixed set of nonlinear endpoint
/// combinations. It performs no optimization search: each coefficient is
/// visited once and selects exactly one endpoint by its sign.
pub(crate) fn interval_maximum(
    coefficients: &[AffineCoefficient],
    mut interval: impl FnMut(AffineTermId) -> Option<(i128, i128)>,
    check: &mut AffineCheckState,
) -> Result<Option<i128>, AffineCheckError> {
    let mut maximum = 0_i128;
    for coefficient in coefficients {
        check.charge(1)?;
        let Some((minimum, upper)) = interval(coefficient.term()) else {
            return Ok(None);
        };
        let endpoint = if coefficient.coefficient() > 0 {
            upper
        } else {
            minimum
        };
        maximum = checked_add(maximum, checked_mul(coefficient.coefficient(), endpoint)?)?;
    }
    Ok(Some(maximum))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AffineCheckLimits {
    max_expression_nodes: usize,
    max_input_terms: usize,
    max_terms: usize,
    max_certificate_premises: usize,
    max_work: u64,
}

/// Compiler-owned deterministic ceilings. They are ordinary checker
/// capacities, not values selected by source text or an external caller.
const AFFINE_CHECK_LIMITS: AffineCheckLimits = AffineCheckLimits {
    max_expression_nodes: 4_096,
    max_input_terms: 4_096,
    max_terms: 4_096,
    max_certificate_premises: 4_096,
    max_work: 10_000_000,
};

/// One finite capacity of an affine semantic-check operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AffineCheckLimit {
    ExpressionNodes,
    InputTerms,
    ResultTerms,
    CertificatePremises,
    Work,
}

/// Deterministic failure from the affine arithmetic checker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AffineCheckError {
    ArithmeticOverflow,
    LimitExceeded(AffineCheckLimit),
    CoefficientMismatch,
}

/// Fixed-work state shared by affine operations in one semantic checking unit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AffineCheckState {
    limits: AffineCheckLimits,
    used: u64,
}

impl AffineCheckState {
    pub(crate) const fn new() -> Self {
        Self {
            limits: AFFINE_CHECK_LIMITS,
            used: 0,
        }
    }

    pub(crate) const fn used(&self) -> u64 {
        self.used
    }

    #[cfg(test)]
    const fn with_limits(limits: AffineCheckLimits) -> Self {
        Self { limits, used: 0 }
    }

    pub(crate) fn charge(&mut self, amount: u64) -> Result<(), AffineCheckError> {
        let used = self
            .used
            .checked_add(amount)
            .ok_or(AffineCheckError::LimitExceeded(AffineCheckLimit::Work))?;
        if used > self.limits.max_work {
            return Err(AffineCheckError::LimitExceeded(AffineCheckLimit::Work));
        }
        self.used = used;
        Ok(())
    }
}

impl Default for AffineCheckState {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalizes one explicit `left <= right` expression into
/// `left - right <= 0`, moving constants to the right-hand side.
pub(crate) fn normalize_less_equal(
    left: &AffineExpression,
    right: &AffineExpression,
    check: &mut AffineCheckState,
) -> Result<AffineInequality, AffineCheckError> {
    let mut scheduled_nodes = 0_usize;
    // Form every child before applying its parent operation. In particular,
    // an outer multiplication by zero cannot erase an overflowing inner
    // coefficient formation: the inner source expression must be valid on
    // its own before the zero is applied.
    let left = normalize_expression(left, check, &mut scheduled_nodes)?;
    let right = normalize_expression(right, check, &mut scheduled_nodes)?;
    AffineInequality::from_forms(&left, &right, check)
}

#[derive(Clone, Copy)]
enum NormalizeExpression<'expression> {
    Visit(&'expression AffineExpression),
    Add,
    Subtract,
    MultiplyByConstant(i128),
}

fn normalize_expression(
    expression: &AffineExpression,
    check: &mut AffineCheckState,
    scheduled_nodes: &mut usize,
) -> Result<AffineForm, AffineCheckError> {
    let mut pending = vec![NormalizeExpression::Visit(expression)];
    let mut values = Vec::new();
    while let Some(next) = pending.pop() {
        match next {
            NormalizeExpression::Visit(expression) => {
                if *scheduled_nodes >= check.limits.max_expression_nodes {
                    return Err(AffineCheckError::LimitExceeded(
                        AffineCheckLimit::ExpressionNodes,
                    ));
                }
                *scheduled_nodes += 1;
                check.charge(1)?;
                match expression {
                    AffineExpression::Constant(value) => {
                        values.push(AffineForm::constant(*value));
                    }
                    AffineExpression::Term(term) => values.push(AffineForm::term(*term)),
                    AffineExpression::Add(left, right) => {
                        pending.push(NormalizeExpression::Add);
                        pending.push(NormalizeExpression::Visit(right));
                        pending.push(NormalizeExpression::Visit(left));
                    }
                    AffineExpression::Subtract(left, right) => {
                        pending.push(NormalizeExpression::Subtract);
                        pending.push(NormalizeExpression::Visit(right));
                        pending.push(NormalizeExpression::Visit(left));
                    }
                    AffineExpression::MultiplyByConstant { constant, value } => {
                        pending.push(NormalizeExpression::MultiplyByConstant(*constant));
                        pending.push(NormalizeExpression::Visit(value));
                    }
                }
            }
            NormalizeExpression::Add => {
                let right = values.pop().ok_or(AffineCheckError::CoefficientMismatch)?;
                let left = values.pop().ok_or(AffineCheckError::CoefficientMismatch)?;
                values.push(left.add(&right, check)?);
            }
            NormalizeExpression::Subtract => {
                let right = values.pop().ok_or(AffineCheckError::CoefficientMismatch)?;
                let left = values.pop().ok_or(AffineCheckError::CoefficientMismatch)?;
                values.push(left.subtract(&right, check)?);
            }
            NormalizeExpression::MultiplyByConstant(constant) => {
                let value = values.pop().ok_or(AffineCheckError::CoefficientMismatch)?;
                values.push(value.scale(constant, check)?);
            }
        }
    }
    let result = values.pop().ok_or(AffineCheckError::CoefficientMismatch)?;
    if values.is_empty() {
        Ok(result)
    } else {
        Err(AffineCheckError::CoefficientMismatch)
    }
}

fn insert_coefficient(
    terms: &mut Vec<AffineCoefficient>,
    term: AffineTermId,
    coefficient: i128,
    check: &mut AffineCheckState,
) -> Result<(), AffineCheckError> {
    if coefficient == 0 {
        return Ok(());
    }

    let mut lower = 0_usize;
    let mut upper = terms.len();
    while lower < upper {
        check.charge(1)?;
        let middle = lower + (upper - lower) / 2;
        if terms[middle].term < term {
            lower = middle + 1;
        } else {
            upper = middle;
        }
    }

    if let Some(existing) = terms.get(lower)
        && existing.term == term
    {
        let combined = checked_add(existing.coefficient, coefficient)?;
        if combined == 0 {
            check.charge(
                u64::try_from(terms.len() - lower)
                    .map_err(|_| AffineCheckError::LimitExceeded(AffineCheckLimit::Work))?,
            )?;
            terms.remove(lower);
        } else {
            check.charge(1)?;
            terms[lower].coefficient = combined;
        }
        return Ok(());
    }

    if terms.len() >= check.limits.max_terms {
        return Err(AffineCheckError::LimitExceeded(
            AffineCheckLimit::ResultTerms,
        ));
    }
    check.charge(
        u64::try_from(terms.len() - lower + 1)
            .map_err(|_| AffineCheckError::LimitExceeded(AffineCheckLimit::Work))?,
    )?;
    terms.insert(lower, AffineCoefficient { term, coefficient });
    Ok(())
}

fn merge_scaled(
    current: &[AffineCoefficient],
    parent: &[AffineCoefficient],
    factor: i128,
    check: &mut AffineCheckState,
) -> Result<Vec<AffineCoefficient>, AffineCheckError> {
    let mut merged = Vec::with_capacity(
        current
            .len()
            .checked_add(parent.len())
            .ok_or(AffineCheckError::LimitExceeded(
                AffineCheckLimit::ResultTerms,
            ))?
            .min(check.limits.max_terms),
    );
    let mut current_index = 0_usize;
    let mut parent_index = 0_usize;
    while current_index < current.len() || parent_index < parent.len() {
        check.charge(1)?;
        let coefficient = match (current.get(current_index), parent.get(parent_index)) {
            (Some(current_term), Some(parent_term)) if current_term.term < parent_term.term => {
                current_index += 1;
                *current_term
            }
            (Some(current_term), Some(parent_term)) if parent_term.term < current_term.term => {
                parent_index += 1;
                AffineCoefficient {
                    term: parent_term.term,
                    coefficient: checked_mul(parent_term.coefficient, factor)?,
                }
            }
            (Some(current_term), Some(parent_term)) => {
                current_index += 1;
                parent_index += 1;
                AffineCoefficient {
                    term: current_term.term,
                    coefficient: checked_add(
                        current_term.coefficient,
                        checked_mul(parent_term.coefficient, factor)?,
                    )?,
                }
            }
            (Some(current_term), None) => {
                current_index += 1;
                *current_term
            }
            (None, Some(parent_term)) => {
                parent_index += 1;
                AffineCoefficient {
                    term: parent_term.term,
                    coefficient: checked_mul(parent_term.coefficient, factor)?,
                }
            }
            (None, None) => break,
        };
        if coefficient.coefficient != 0 {
            if merged.len() >= check.limits.max_terms {
                return Err(AffineCheckError::LimitExceeded(
                    AffineCheckLimit::ResultTerms,
                ));
            }
            merged.push(coefficient);
        }
    }
    Ok(merged)
}

fn checked_add(left: i128, right: i128) -> Result<i128, AffineCheckError> {
    left.checked_add(right)
        .ok_or(AffineCheckError::ArithmeticOverflow)
}

fn checked_sub(left: i128, right: i128) -> Result<i128, AffineCheckError> {
    left.checked_sub(right)
        .ok_or(AffineCheckError::ArithmeticOverflow)
}

fn checked_mul(left: i128, right: i128) -> Result<i128, AffineCheckError> {
    left.checked_mul(right)
        .ok_or(AffineCheckError::ArithmeticOverflow)
}

fn checked_neg(value: i128) -> Result<i128, AffineCheckError> {
    value
        .checked_neg()
        .ok_or(AffineCheckError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn term(index: u32) -> AffineExpression {
        AffineExpression::Term(AffineTermId::from_index(index))
    }

    fn constant(value: i128) -> AffineExpression {
        AffineExpression::Constant(value)
    }

    fn add(left: AffineExpression, right: AffineExpression) -> AffineExpression {
        AffineExpression::Add(Box::new(left), Box::new(right))
    }

    fn subtract(left: AffineExpression, right: AffineExpression) -> AffineExpression {
        AffineExpression::Subtract(Box::new(left), Box::new(right))
    }

    fn multiply(constant: i128, value: AffineExpression) -> AffineExpression {
        AffineExpression::MultiplyByConstant {
            constant,
            value: Box::new(value),
        }
    }

    fn coefficients(inequality: &AffineInequality) -> Vec<(u32, i128)> {
        inequality
            .terms()
            .iter()
            .map(|coefficient| (coefficient.term().index(), coefficient.coefficient()))
            .collect()
    }

    fn inequality(terms: &[(u32, i128)], upper: i128) -> AffineInequality {
        let terms = terms
            .iter()
            .map(|&(term, coefficient)| (AffineTermId::from_index(term), coefficient))
            .collect::<Vec<_>>();
        AffineInequality::from_terms(&terms, upper, &mut AffineCheckState::new())
            .expect("test inequality")
    }

    #[test]
    fn normalization_moves_constants_orders_terms_and_removes_cancellation() {
        // sum + 5 <= 3*i + 9 becomes sum - 3*i <= 4.
        let left = add(term(7), constant(5));
        let right = add(multiply(3, term(2)), constant(9));
        let mut check = AffineCheckState::new();
        let normalized = normalize_less_equal(&left, &right, &mut check).expect("normalization");
        assert_eq!(coefficients(&normalized), vec![(2, -3), (7, 1)]);
        assert_eq!(normalized.upper(), 4);
        assert!(check.used() > 0);

        let cancelled = subtract(term(4), term(4));
        let normalized =
            normalize_less_equal(&cancelled, &constant(0), &mut AffineCheckState::new())
                .expect("cancellation");
        assert!(normalized.terms().is_empty());
    }

    #[test]
    fn canonical_inequalities_merge_and_remove_zero_coefficients() {
        let canonical = AffineInequality::from_terms(
            &[
                (AffineTermId::from_index(7), 0),
                (AffineTermId::from_index(4), 3),
                (AffineTermId::from_index(2), 1),
                (AffineTermId::from_index(4), -1),
                (AffineTermId::from_index(2), -1),
                (AffineTermId::from_index(1), 5),
            ],
            9,
            &mut AffineCheckState::new(),
        )
        .expect("canonical inequality");
        assert_eq!(coefficients(&canonical), vec![(1, 5), (4, 2)]);
        assert_eq!(canonical.upper(), 9);
    }

    #[test]
    fn exact_value_forms_follow_assignment_arithmetic_without_creating_a_fact() {
        let mut check = AffineCheckState::new();
        let sum = AffineForm::term(AffineTermId::from_index(0));
        let wide = AffineForm::term(AffineTermId::from_index(2));
        let updated = sum.add(&wide, &mut check).expect("sum + wide form");
        assert_eq!(
            updated
                .terms()
                .iter()
                .map(|term| (term.term().index(), term.coefficient()))
                .collect::<Vec<_>>(),
            vec![(0, 1), (2, 1)]
        );
        assert_eq!(updated.constant_value(), 0);

        let restored = updated
            .subtract(&wide, &mut check)
            .expect("subtract copied value");
        assert_eq!(restored, sum);
        assert_eq!(
            AffineForm::constant(7)
                .scale(-3, &mut check)
                .expect("constant scale")
                .constant_value(),
            -21
        );
    }

    #[test]
    fn one_header_invariant_leaves_the_weigh_step_as_a_type_interval() {
        // Header: sum_k <= 255*i_k.
        let header = inequality(&[(0, 1), (1, -255)], 0);
        // After the body and hidden update:
        // sum_k + wide <= 255*(i_k + 1).
        let next = inequality(&[(0, 1), (1, -255), (2, 1)], 255);
        let mut check = AffineCheckState::new();
        let residual = AffineInequality::residual_after(&next, &header, &mut check)
            .expect("fixed coefficient-one residual");
        assert_eq!(coefficients(&residual), vec![(2, 1)]);
        assert_eq!(residual.upper(), 255);
        assert!(
            interval_proves(
                &residual,
                |term| (term.index() == 2).then_some((0, 255)),
                &mut check,
            )
            .expect("u8 interval rule")
        );
    }

    #[test]
    fn explicit_sum_certificate_composes_the_weigh_limit_exactly() {
        // sum - 255*count <= 0
        let per_byte = inequality(&[(0, 1), (1, -255)], 0);
        // 255*count <= 255000
        let count_limit = inequality(&[(1, 255)], 255_000);
        // sum <= 255000
        let target = inequality(&[(0, 1)], 255_000);

        assert_eq!(
            verify_explicit_inequality_sum(
                &[per_byte, count_limit],
                &target,
                &mut AffineCheckState::new(),
            ),
            Ok(())
        );
    }

    #[test]
    fn explicit_sum_uses_the_supplied_order_without_changing_the_mathematical_sum() {
        let per_byte = inequality(&[(0, 1), (1, -255)], 0);
        let count_limit = inequality(&[(1, 255)], 255_000);
        let target = inequality(&[(0, 1)], 255_000);

        assert_eq!(
            verify_explicit_inequality_sum(
                &[per_byte.clone(), count_limit.clone()],
                &target,
                &mut AffineCheckState::new(),
            ),
            Ok(())
        );
        assert_eq!(
            verify_explicit_inequality_sum(
                &[count_limit, per_byte],
                &target,
                &mut AffineCheckState::new(),
            ),
            Ok(())
        );

        // These two lists have the same mathematical sum. The safe source
        // order succeeds, while the other reaches an i128 overflow at its
        // second written premise. The checker does not reassociate the list to
        // avoid that intermediate result.
        let maximum = inequality(&[], i128::MAX);
        let plus_one = inequality(&[], 1);
        let minus_one = inequality(&[], -1);
        assert_eq!(
            verify_explicit_inequality_sum(
                &[maximum.clone(), minus_one.clone(), plus_one.clone()],
                &maximum,
                &mut AffineCheckState::new(),
            ),
            Ok(())
        );
        assert_eq!(
            verify_explicit_inequality_sum(
                &[maximum.clone(), plus_one, minus_one],
                &maximum,
                &mut AffineCheckState::new(),
            ),
            Err(AffineCheckError::ArithmeticOverflow)
        );
    }

    #[test]
    fn explicit_sum_rejects_a_missing_premise_as_an_exact_mismatch() {
        let per_byte = inequality(&[(0, 1), (1, -255)], 0);
        let target = inequality(&[(0, 1)], 255_000);
        assert_eq!(
            verify_explicit_inequality_sum(&[per_byte], &target, &mut AffineCheckState::new(),),
            Err(AffineCheckError::CoefficientMismatch)
        );
    }

    #[test]
    fn explicit_sum_has_a_fixed_premise_capacity() {
        let premises = [inequality(&[(0, 1)], 0), inequality(&[(1, 1)], 0)];
        let target = inequality(&[(0, 1), (1, 1)], 0);
        let mut check = AffineCheckState::with_limits(AffineCheckLimits {
            max_certificate_premises: 1,
            ..AFFINE_CHECK_LIMITS
        });
        assert_eq!(
            verify_explicit_inequality_sum(&premises, &target, &mut check),
            Err(AffineCheckError::LimitExceeded(
                AffineCheckLimit::CertificatePremises
            ))
        );
    }

    #[test]
    fn interval_rule_uses_the_endpoint_selected_by_each_coefficient_sign() {
        let target = inequality(&[(0, 3), (1, -2)], 32);
        let mut check = AffineCheckState::new();
        assert!(
            interval_proves(
                &target,
                |term| match term.index() {
                    0 => Some((0, 10)),
                    1 => Some((-1, 8)),
                    _ => None,
                },
                &mut check,
            )
            .expect("fixed interval rule")
        );
        // 3*10 - 2*(-2) = 34, so the same rule cannot prove <= 32.
        assert_eq!(
            interval_proves(
                &target,
                |term| match term.index() {
                    0 => Some((0, 10)),
                    1 => Some((-2, 8)),
                    _ => None,
                },
                &mut check,
            ),
            Ok(false)
        );
    }

    #[test]
    fn residual_factor_eliminates_repeated_coefficient_one_uses_in_one_step() {
        let residual = inequality(&[(0, 6), (1, -6)], 0);
        let premise = inequality(&[(0, 2), (1, -2)], 0);
        let mut check = AffineCheckState::new();
        let factor = maximum_safe_residual_factor(&residual, &premise, &mut check)
            .expect("fixed factor calculation")
            .expect("same-sign overlap");
        assert_eq!(factor, 3);
        let reduced =
            AffineInequality::residual_after_scaled(&residual, &premise, factor, &mut check)
                .expect("scaled residual");
        assert!(reduced.terms().is_empty());
        assert_eq!(reduced.upper(), 0);
        assert!(
            residual_measure_decreases(&residual, &reduced, &mut check)
                .expect("well-founded measure")
        );
    }

    #[test]
    fn residual_measure_rejects_an_earlier_term_regression() {
        let before = inequality(&[(1, 1)], 0);
        let after = inequality(&[(0, -1)], 0);
        assert_eq!(
            residual_measure_decreases(&before, &after, &mut AffineCheckState::new()),
            Ok(false),
            "introducing an earlier canonical term is not progress"
        );
    }

    #[test]
    fn opposite_sign_overlap_does_not_supply_a_safe_factor() {
        let residual = inequality(&[(0, 1)], 0);
        let premise = inequality(&[(0, -1)], 0);
        assert_eq!(
            maximum_safe_residual_factor(&residual, &premise, &mut AffineCheckState::new()),
            Ok(None)
        );
    }

    #[test]
    fn fixed_residual_order_can_stop_on_an_irrelevant_earlier_fact() {
        // B: x <= 1; P: 2x <= 2y; Q: 2y <= x; target: x <= 0.
        // P + Q proves the target, but the fixed automatic order B,P,Q first
        // takes B because it removes x. The residual is then `0 <= -1` and no
        // remaining fact overlaps it. This deliberate incompleteness is why
        // explicit prove/use remains available for author-selected witnesses.
        let target = inequality(&[(0, 1)], 0);
        let lure = inequality(&[(0, 1)], 1);
        let first = inequality(&[(0, 2), (1, -2)], 0);
        let second = inequality(&[(0, -1), (1, 2)], 0);
        let mut check = AffineCheckState::new();
        let factor = maximum_safe_residual_factor(&target, &lure, &mut check)
            .expect("fixed factor")
            .expect("the lure overlaps x");
        assert_eq!(factor, 1);
        let stuck = AffineInequality::residual_after_scaled(&target, &lure, factor, &mut check)
            .expect("lure residual");
        assert!(residual_measure_decreases(&target, &stuck, &mut check).unwrap());
        assert_eq!(interval_proves(&stuck, |_| None, &mut check), Ok(false));
        assert_eq!(
            maximum_safe_residual_factor(&stuck, &first, &mut check),
            Ok(None)
        );
        assert_eq!(
            maximum_safe_residual_factor(&stuck, &second, &mut check),
            Ok(None)
        );
        assert_eq!(
            verify_explicit_inequality_sum(&[first, second], &target, &mut check),
            Ok(())
        );
    }

    #[test]
    fn every_i128_operation_is_checked() {
        let nested_scale = multiply(i128::MAX, multiply(2, term(1)));
        assert_eq!(
            normalize_less_equal(&nested_scale, &constant(0), &mut AffineCheckState::new(),),
            Err(AffineCheckError::ArithmeticOverflow)
        );

        let coefficient_add = add(multiply(i128::MAX, term(1)), term(1));
        assert_eq!(
            normalize_less_equal(&coefficient_add, &constant(0), &mut AffineCheckState::new(),),
            Err(AffineCheckError::ArithmeticOverflow)
        );

        assert_eq!(
            normalize_less_equal(
                &constant(i128::MIN),
                &constant(0),
                &mut AffineCheckState::new(),
            ),
            Err(AffineCheckError::ArithmeticOverflow)
        );

        let negated_multiplier = multiply(i128::MIN, subtract(term(1), term(2)));
        assert_eq!(
            normalize_less_equal(
                &negated_multiplier,
                &constant(0),
                &mut AffineCheckState::new(),
            ),
            Err(AffineCheckError::ArithmeticOverflow)
        );
    }

    #[test]
    fn outer_zero_does_not_hide_an_invalid_inner_formation() {
        let overflowing_inner = multiply(i128::MAX, multiply(2, term(1)));
        let zero_times_inner = multiply(0, overflowing_inner);
        assert_eq!(
            normalize_less_equal(
                &zero_times_inner,
                &constant(0),
                &mut AffineCheckState::new(),
            ),
            Err(AffineCheckError::ArithmeticOverflow)
        );
    }

    #[test]
    fn fixed_work_accounting_stops_expression_and_work_limits() {
        let limits = AffineCheckLimits {
            max_expression_nodes: 2,
            max_input_terms: 8,
            max_terms: 8,
            max_certificate_premises: 8,
            max_work: 100,
        };
        let mut check = AffineCheckState::with_limits(limits);
        assert_eq!(
            normalize_less_equal(&add(term(0), term(1)), &constant(0), &mut check),
            Err(AffineCheckError::LimitExceeded(
                AffineCheckLimit::ExpressionNodes
            ))
        );

        let mut check = AffineCheckState::with_limits(AffineCheckLimits {
            max_work: 1,
            ..AFFINE_CHECK_LIMITS
        });
        assert_eq!(
            AffineInequality::from_terms(&[(AffineTermId::from_index(0), 1)], 0, &mut check),
            Err(AffineCheckError::LimitExceeded(AffineCheckLimit::Work))
        );
    }
}
