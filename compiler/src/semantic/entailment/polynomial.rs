//! The transient degree-two accumulator a written certificate passes through.
//!
//! [PRF-1] scales each premise by its written multiplicity and adds the
//! results. A bare-decimal multiplicity keeps that sum affine, but a term
//! multiplicity does not: scaling `p - k + 1 <= 0` by `n` produces `n*p`,
//! which no affine inequality can hold. Such a monomial exists only while the
//! certificate is being checked — every one of them must fold back to the
//! value image of an admitted exact product before the residual is formed —
//! so it lives here rather than in [`super::affine::AffineForm`], the fact
//! state, or a published conclusion.
//!
//! Degree two is the whole domain: a premise is scaled by one multiplicity and
//! an admitted product has two operands, so nothing in [PRF-1] can reach
//! degree three. A multiplication that would is an error rather than a
//! silently dropped term.

use std::collections::BTreeMap;

use super::affine::{AffineCheckState, AffineForm, AffineInequality, AffineTermId};

/// One monomial of the accumulator.
///
/// The empty product is the constant, one atom is the affine case, and an
/// ordered pair of atoms is the only nonlinear shape [PRF-1] admits. The pair
/// is stored with its smaller atom first so `n*p` and `p*n` are one key.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Monomial {
    Constant,
    Linear(AffineTermId),
    Quadratic(AffineTermId, AffineTermId),
}

impl Monomial {
    fn quadratic(left: AffineTermId, right: AffineTermId) -> Self {
        if left.index() <= right.index() {
            Self::Quadratic(left, right)
        } else {
            Self::Quadratic(right, left)
        }
    }

    /// The product of two monomials, or `None` when it would exceed degree two.
    fn multiply(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Self::Constant, other) | (other, Self::Constant) => Some(other),
            (Self::Linear(left), Self::Linear(right)) => Some(Self::quadratic(left, right)),
            _ => None,
        }
    }
}

/// What stopped a certificate accumulation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PolynomialError {
    /// A coefficient or constant left the checked `i128` proof domain.
    ArithmeticOverflow,
    /// A product would have exceeded degree two.
    DegreeExceeded,
    /// The accumulator holds more monomials than one certificate may form.
    LimitExceeded,
}

/// The largest number of distinct monomials one certificate may accumulate.
///
/// A certificate over `t` atoms with one term multiplicity spans at most
/// `t*(t+1)/2 + t + 1` monomials, so this is a structural ceiling on written
/// source rather than a work budget: nothing in [PRF-1] iterates over it.
const MAX_MONOMIALS: usize = 4096;

/// `sum(coefficient * monomial)`, canonical: zero coefficients are absent and
/// the monomial order is total, so equal polynomials compare equal.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CertificatePolynomial {
    terms: BTreeMap<Monomial, i128>,
}

impl CertificatePolynomial {
    pub(crate) fn zero() -> Self {
        Self::default()
    }

    /// The polynomial `p` of an inequality written as `p <= 0`.
    ///
    /// [`AffineInequality`] segregates its constant as an upper bound, so the
    /// bound moves to the left with its sign flipped and the two shapes then
    /// add without a special case.
    pub(crate) fn from_inequality(inequality: &AffineInequality) -> Result<Self, PolynomialError> {
        let mut polynomial = Self::zero();
        for coefficient in inequality.terms() {
            polynomial.add_monomial(
                Monomial::Linear(coefficient.term()),
                coefficient.coefficient(),
            )?;
        }
        polynomial.add_monomial(
            Monomial::Constant,
            inequality
                .upper()
                .checked_neg()
                .ok_or(PolynomialError::ArithmeticOverflow)?,
        )?;
        Ok(polynomial)
    }

    /// The polynomial of one value image.
    pub(crate) fn from_form(form: &AffineForm) -> Result<Self, PolynomialError> {
        let mut polynomial = Self::zero();
        for coefficient in form.terms() {
            polynomial.add_monomial(
                Monomial::Linear(coefficient.term()),
                coefficient.coefficient(),
            )?;
        }
        polynomial.add_monomial(Monomial::Constant, form.constant_value())?;
        Ok(polynomial)
    }

    fn add_monomial(
        &mut self,
        monomial: Monomial,
        coefficient: i128,
    ) -> Result<(), PolynomialError> {
        if coefficient == 0 {
            return Ok(());
        }
        let entry = self.terms.entry(monomial).or_insert(0);
        *entry = entry
            .checked_add(coefficient)
            .ok_or(PolynomialError::ArithmeticOverflow)?;
        if *entry == 0 {
            self.terms.remove(&monomial);
        } else if self.terms.len() > MAX_MONOMIALS {
            return Err(PolynomialError::LimitExceeded);
        }
        Ok(())
    }

    pub(crate) fn add(&self, other: &Self) -> Result<Self, PolynomialError> {
        let mut sum = self.clone();
        for (monomial, coefficient) in &other.terms {
            sum.add_monomial(*monomial, *coefficient)?;
        }
        Ok(sum)
    }

    pub(crate) fn multiply(&self, other: &Self) -> Result<Self, PolynomialError> {
        let mut product = Self::zero();
        for (left, left_coefficient) in &self.terms {
            for (right, right_coefficient) in &other.terms {
                let monomial = left
                    .multiply(*right)
                    .ok_or(PolynomialError::DegreeExceeded)?;
                product.add_monomial(
                    monomial,
                    left_coefficient
                        .checked_mul(*right_coefficient)
                        .ok_or(PolynomialError::ArithmeticOverflow)?,
                )?;
            }
        }
        Ok(product)
    }

    /// Folds each nonlinear monomial back to the one atom that already equals
    /// it: the value image of an admitted exact product of the same two
    /// operands.
    ///
    /// The direction matters, and not only because it is the safer of two
    /// symmetric choices. Folding is bounded: the sum holds finitely many
    /// monomials and each fold removes one, so it terminates in the degree-two
    /// domain it started in. Expanding the target's product atoms is not
    /// bounded, because an operand may itself be a product — `let a = n * p;
    /// let b = a * q;` expands `b` to `a*q` and then `a` to `n*p`, which is
    /// degree three and outside anything [PRF-1] can hold. Expansion also
    /// rewrites a proposition that was already affine, so it can turn a
    /// provable residual into an unprovable one. What folding leaves is an
    /// ordinary affine sum, so the residual, its integer tightenings, and the
    /// L0 route that discharges it are the same ones a bare-decimal
    /// certificate uses.
    pub(crate) fn fold_products(
        &self,
        products: &BTreeMap<(AffineTermId, AffineTermId), AffineTermId>,
    ) -> Result<Self, PolynomialError> {
        let mut folded = Self::zero();
        for (monomial, coefficient) in &self.terms {
            let product = match monomial {
                Monomial::Quadratic(left, right) => products.get(&(*left, *right)).copied(),
                Monomial::Constant | Monomial::Linear(_) => None,
            };
            match product {
                Some(atom) => folded.add_monomial(Monomial::Linear(atom), *coefficient)?,
                None => folded.add_monomial(*monomial, *coefficient)?,
            }
        }
        Ok(folded)
    }

    pub(crate) fn scale(&self, factor: i128) -> Result<Self, PolynomialError> {
        let mut scaled = Self::zero();
        for (monomial, coefficient) in &self.terms {
            scaled.add_monomial(
                *monomial,
                coefficient
                    .checked_mul(factor)
                    .ok_or(PolynomialError::ArithmeticOverflow)?,
            )?;
        }
        Ok(scaled)
    }

    /// Whether every monomial is the constant or a single atom.
    pub(crate) fn is_affine(&self) -> bool {
        !self
            .terms
            .keys()
            .any(|monomial| matches!(monomial, Monomial::Quadratic(_, _)))
    }

    /// This polynomial read back as the inequality `p <= 0`, or `None` when a
    /// nonlinear monomial survives.
    pub(crate) fn into_inequality(
        self,
        check: &mut AffineCheckState,
    ) -> Option<Result<AffineInequality, PolynomialError>> {
        if !self.is_affine() {
            return None;
        }
        let mut terms = Vec::new();
        let mut constant = 0_i128;
        for (monomial, coefficient) in self.terms {
            match monomial {
                Monomial::Constant => constant = coefficient,
                Monomial::Linear(atom) => terms.push((atom, coefficient)),
                Monomial::Quadratic(_, _) => unreachable!("checked affine above"),
            }
        }
        let Some(upper) = constant.checked_neg() else {
            return Some(Err(PolynomialError::ArithmeticOverflow));
        };
        Some(
            AffineInequality::from_terms(&terms, upper, check)
                .map_err(|_| PolynomialError::LimitExceeded),
        )
    }
}
