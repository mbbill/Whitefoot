//! [CALL-6] the consistency of one published relation set — a source
//! contract's, and a compiler-owned [BLK-0] row's.
//!
//! Publication is what makes a declared relation a fact at a caller, and a
//! caller closes every relation it receives together [ENT-4]. An
//! inconsistent set is therefore not one wrong fact: at a contradictory point
//! every relation and both signs of every goal are derivable, so a caller
//! discharges every obligation it submits, including the subscript bounds and
//! the integer domains that keep memory intact. The check belongs at the
//! declaration because that is where the set is fixed — a caller's own state
//! is not consulted and cannot repair it.
//!
//! The judgment is the ordinary difference-bound closure the fragment already
//! uses, run over the declared templates rather than over a program point:
//! each distinct operand datum is one abstract term, a literal folds through
//! the zero term with its value, and the set is contradictory exactly when
//! the closure derives a negative self-bound or forces two terms that one
//! declared disequality separates to be equal.
//!
//! A kernel-domain row's declared list is the same kind of set and is judged
//! by the same closure [BLK-0]. What differs is only how an operand is keyed:
//! a source clause names a formal, a result, a named const or a measure of a
//! place, and a row names one of the closed operand shapes its record
//! notation writes.

#[cfg(test)]
use super::super::kernel::{
    KernelOffset, KernelOperand, KernelPlace, KernelRelation, KernelRoute, KernelSignature,
};
use super::super::postcondition::{
    NormalizedRelation, PostconditionPlaceRoot, RelationDatum, RelationTemplate, RelationTerm,
};
use crate::semantic::model::{CheckedMeasure, CheckedValue, IntegerType};

/// The abstract term one operand denotes in the declaration-domain closure:
/// an offset from a named term, where term `0` is the zero term.
#[derive(Clone, Copy)]
struct Operand {
    term: usize,
    offset: i128,
}

/// One set of difference bounds over abstract terms, where term `0` is the
/// zero term and every other index is one interned operand key.
///
/// This is the whole of [CALL-6]'s consistency judgment; what a term index
/// *means* belongs to whichever declaration domain interned it.
#[derive(Default)]
struct DifferenceSystem {
    /// One past the largest term index any bound or disequality names.
    terms: usize,
    /// `bounds[(left, right)]` is one declared `left - right <= c`.
    bounds: Vec<(usize, usize, i128)>,
    distinct: Vec<(usize, i128, usize, i128)>,
}

impl DifferenceSystem {
    fn observe(&mut self, term: usize) {
        self.terms = self.terms.max(term.saturating_add(1));
    }

    fn bound(&mut self, left: Operand, right: Operand, constant: i128) {
        // `left.term + left.offset - right.term - right.offset <= constant`.
        let Some(shifted) = constant
            .checked_add(right.offset)
            .and_then(|value| value.checked_sub(left.offset))
        else {
            return;
        };
        self.observe(left.term);
        self.observe(right.term);
        self.bounds.push((left.term, right.term, shifted));
    }

    fn distinguish(&mut self, left: Operand, right: Operand) {
        self.observe(left.term);
        self.observe(right.term);
        self.distinct
            .push((left.term, left.offset, right.term, right.offset));
    }

    /// The tightest declared bound on every ordered pair, by the same
    /// transitive composition [ENT-4] rule (1) performs. The matrix is one
    /// flat row-major buffer: `bound(left, right)` lives at
    /// `left * count + right`.
    fn close(&self) -> Closure {
        let count = self.terms.max(1);
        let mut bounds = vec![None; count * count];
        for index in 0..count {
            bounds[index * count + index] = Some(0);
        }
        for (left, right, constant) in &self.bounds {
            let slot = &mut bounds[left * count + right];
            if slot.is_none_or(|existing| *constant < existing) {
                *slot = Some(*constant);
            }
        }
        for middle in 0..count {
            for left in 0..count {
                let Some(first) = bounds[left * count + middle] else {
                    continue;
                };
                for right in 0..count {
                    let Some(second) = bounds[middle * count + right] else {
                        continue;
                    };
                    let Some(composed) = first.checked_add(second) else {
                        continue;
                    };
                    let slot = &mut bounds[left * count + right];
                    if slot.is_none_or(|existing| composed < existing) {
                        *slot = Some(composed);
                    }
                }
            }
        }
        Closure { count, bounds }
    }

    fn is_contradictory(&self) -> bool {
        let closure = self.close();
        if (0..closure.count)
            .any(|index| closure.bound(index, index).is_some_and(|value| value < 0))
        {
            return true;
        }
        // A declared disequality contradicts the bounds exactly when they
        // already force its two operands equal.
        self.distinct
            .iter()
            .any(|(left, left_offset, right, right_offset)| {
                let Some(gap) = right_offset.checked_sub(*left_offset) else {
                    return false;
                };
                closure
                    .bound(*left, *right)
                    .is_some_and(|value| value <= gap)
                    && closure
                        .bound(*right, *left)
                        .is_some_and(|value| value <= -gap)
            })
    }
}

/// One structural key by which two operand datums are the same term. Two
/// clauses naming the same result, formal, projection chain, or named
/// constant name one term; a literal is folded onto the zero term instead.
#[derive(Eq, PartialEq)]
enum OperandKey {
    Result,
    Parameter(u32, ProjectionKey),
    NamedConst(crate::DeclarationId, ProjectionKey),
    /// One measure of one formal place [MSR-1]: two clauses name one term
    /// only when they name the same measure of the same place.
    Measure(CheckedMeasure, u32, ProjectionKey),
    /// One measure of one declared result place [CALL-4].
    ResultMeasure(CheckedMeasure, u32, ProjectionKey),
}

/// The exact projection path of one operand's place, as this table keys it.
///
/// It is the written path itself rather than a lossy digest of it: a
/// subscript's offset [MSR-1] is part of the place's identity, so two clauses
/// naming two elements of one run name two terms.
type ProjectionKey = Vec<super::super::goal::GoalProjection>;

/// The declared relations of one contract, as difference bounds over
/// abstract terms.
#[derive(Default)]
struct DeclaredSystem {
    keys: Vec<OperandKey>,
    system: DifferenceSystem,
}

impl DeclaredSystem {
    /// The abstract term index of one operand key, interning it on first use.
    /// Index `0` is reserved for the zero term.
    fn term(&mut self, key: OperandKey) -> usize {
        if let Some(position) = self.keys.iter().position(|existing| *existing == key) {
            return position + 1;
        }
        self.keys.push(key);
        self.keys.len()
    }

    /// One relation term: its datum's abstract term, displaced by the
    /// constant the clause side writes [FN-9].
    fn relation_term(&mut self, term: &RelationTerm) -> Option<Operand> {
        let operand = self.operand(&term.datum)?;
        Some(Operand {
            term: operand.term,
            offset: operand.offset.checked_add(term.displacement)?,
        })
    }

    fn operand(&mut self, datum: &RelationDatum) -> Option<Operand> {
        let key = match datum {
            RelationDatum::Result { .. } => OperandKey::Result,
            RelationDatum::Parameter {
                ordinal,
                projections,
                ..
            } => OperandKey::Parameter(*ordinal, projection_key(projections)),
            RelationDatum::NamedConst {
                declaration,
                projections,
                ..
            } => OperandKey::NamedConst(*declaration, projection_key(projections)),
            RelationDatum::Measure(measure, place) => match place.root {
                PostconditionPlaceRoot::Parameter { ordinal } => {
                    OperandKey::Measure(*measure, ordinal, projection_key(&place.projections))
                }
                PostconditionPlaceRoot::Result { ordinal } => {
                    OperandKey::ResultMeasure(*measure, ordinal, projection_key(&place.projections))
                }
            },
            RelationDatum::Literal { value, .. } => {
                // A literal is the zero term displaced by its own
                // mathematical value, exactly as [ENT-2] folds a constant
                // operand through Z.
                let CheckedValue::Integer { ty, bits } = value else {
                    return None;
                };
                return Some(Operand {
                    term: 0,
                    offset: integer_value(*ty, *bits),
                });
            }
        };
        Some(Operand {
            term: self.term(key),
            offset: 0,
        })
    }

    fn add(&mut self, template: &RelationTemplate) -> Option<()> {
        let left = self.relation_term(&template.operands[0])?;
        let right = self.relation_term(&template.operands[1])?;
        match template.normalized {
            NormalizedRelation::Equal => {
                self.system.bound(left, right, 0);
                self.system.bound(right, left, 0);
            }
            NormalizedRelation::NotEqual => self.system.distinguish(left, right),
            NormalizedRelation::UpperBound {
                left: first,
                right: second,
                strict,
            } => {
                let operands = [left, right];
                let lower = *operands.get(usize::from(first))?;
                let upper = *operands.get(usize::from(second))?;
                self.system.bound(lower, upper, if strict { -1 } else { 0 });
            }
        }
        Some(())
    }
}

/// The transitive closure of one declared relation set, as a flat row-major
/// matrix of tightest bounds.
struct Closure {
    count: usize,
    bounds: Vec<Option<i128>>,
}

impl Closure {
    fn bound(&self, left: usize, right: usize) -> Option<i128> {
        self.bounds[left * self.count + right]
    }
}

fn projection_key(projections: &[super::super::goal::GoalProjection]) -> ProjectionKey {
    projections.to_vec()
}

/// Whether the relations one contract publishes on one route are
/// contradictory at their establishment point [CALL-6].
///
/// A template whose operand shape this closure cannot represent is skipped
/// rather than assumed consistent: skipping only removes premises, so the
/// answer stays a genuine contradiction whenever it is `true`.
pub(super) fn relations_are_contradictory(templates: &[&RelationTemplate]) -> bool {
    let mut declared = DeclaredSystem::default();
    for template in templates {
        let _ = declared.add(template);
    }
    declared.system.is_contradictory()
}

// The [BLK-0] row form of the same judgment. A row's declared set is fixed by
// this specification rather than by a program, so what judges it is this
// repository's own evidence: the unit test over the record data, and not a
// compilation.

/// One abstract term of a [BLK-0] row's own operand language.
///
/// A row names one of five closed operand shapes and nothing else, so the
/// key is that shape itself: two clauses name one term exactly when they
/// write the same measure of the same place, the same value parameter, the
/// same const parameter, or the same layout ceiling.
#[cfg(test)]
#[derive(Eq, PartialEq)]
enum KernelOperandKey {
    Measure(CheckedMeasure, KernelPlaceKey),
    Value(u32),
    Const(super::super::kernel::KernelConst),
    AlignCeiling,
}

/// The place one row operand names, keyed so that a `&uniq` state operand's
/// post-state and that call's call datum for the same place are two terms
/// [BLK-0, MSR-3].
#[cfg(test)]
#[derive(Eq, PartialEq)]
enum KernelPlaceKey {
    Parameter(u32),
    ParameterAtCall(u32),
    Result(u32),
    Payload,
}

#[cfg(test)]
const fn kernel_place_key(place: KernelPlace) -> KernelPlaceKey {
    match place {
        KernelPlace::Parameter(ordinal) => KernelPlaceKey::Parameter(ordinal),
        KernelPlace::ParameterAtCall(ordinal) => KernelPlaceKey::ParameterAtCall(ordinal),
        KernelPlace::Result(ordinal) => KernelPlaceKey::Result(ordinal),
        KernelPlace::Payload => KernelPlaceKey::Payload,
    }
}

/// Whether one [BLK-0] row's declared requirement and relation lists are
/// contradictory on one of its declared exits, at one resolution of
/// `advance<T>(count)` [CALL-6].
///
/// The set judged is every requirement — which a caller has discharged
/// before a relation of the row is established — together with every
/// relation the exit carries: an unrouted clause is a member of every exit's
/// set and a routed one of its own [CALL-6].
///
/// `advance` is the one operand of the record notation that is not a
/// constant of the declaration: it is an opaque `u64` term whose value the
/// call's own instance fixes [BLK-0], and a difference-bound closure carries
/// a constant displacement and not a symbolic one. The judgment is therefore
/// made at a resolution of it, and the caller supplies the resolutions it
/// wants covered.
#[cfg(test)]
pub(crate) fn kernel_row_is_contradictory(
    signature: &KernelSignature,
    exit: Option<KernelRoute>,
    advance: i128,
) -> bool {
    let mut keys: Vec<KernelOperandKey> = Vec::new();
    let mut system = DifferenceSystem::default();
    let term = |keys: &mut Vec<KernelOperandKey>, operand: KernelOperand| -> Operand {
        let key = match operand {
            // The zero term carries every constant displacement [ENT-2].
            KernelOperand::Zero => {
                return Operand { term: 0, offset: 0 };
            }
            KernelOperand::Measure(measure, place) => {
                KernelOperandKey::Measure(measure, kernel_place_key(place))
            }
            KernelOperand::Value(ordinal) => KernelOperandKey::Value(ordinal),
            KernelOperand::Const(which) => KernelOperandKey::Const(which),
            KernelOperand::AlignCeiling => KernelOperandKey::AlignCeiling,
        };
        let index = keys
            .iter()
            .position(|existing| *existing == key)
            .map_or_else(
                || {
                    keys.push(key);
                    keys.len()
                },
                |position| position + 1,
            );
        Operand {
            term: index,
            offset: 0,
        }
    };
    let displacement = |offset: KernelOffset| match offset {
        KernelOffset::Constant(value) => Some(i128::from(value)),
        KernelOffset::Advance(_) => Some(advance),
    };
    let carried = |relation: &&KernelRelation| relation.route.is_none() || relation.route == exit;
    for relation in signature
        .requires
        .iter()
        .chain(signature.ensures)
        .filter(carried)
    {
        let Some(bounds) = relation.bounds(displacement) else {
            continue;
        };
        for bound in bounds {
            let left = term(&mut keys, bound.left);
            let right = term(&mut keys, bound.right);
            system.bound(left, right, bound.bound);
        }
    }
    system.is_contradictory()
}

/// The mathematical value of one checked integer constant, whose `bits` hold
/// the type-width two's-complement pattern [ENT-2].
const fn integer_value(ty: IntegerType, bits: u64) -> i128 {
    let value = bits as i128;
    if ty.signed() {
        let width = ty.width() as u32;
        let sign_bit = 1_u64 << (width - 1);
        if bits & sign_bit != 0 {
            return value - (1_i128 << width);
        }
    }
    value
}
