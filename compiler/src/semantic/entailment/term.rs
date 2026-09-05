//! [ENT-2] terms: the closed vocabulary the L0 fragment relates.
//!
//! A term is a tracked place, a length term over a place, one of the two
//! private endpoint captures of a written counted range, the private commit
//! value of one `set` occurrence, a constant, a symbolic const-generic
//! parameter, or the distinguished zero term Z. Term identity is
//! declaration-anchored: two places are the same term exactly when
//! their roots resolve to the same declaration event — one [`BindingId`] in
//! one checked function — and their canonical spellings agree, which the
//! structural representation below captures byte-for-byte for canonical
//! source. Identity deliberately under-approximates aliasing; kills use the
//! [OWN-7] overlap relation over resolved places instead, which
//! over-approximates it [ENT-5].

use std::collections::HashMap;

use super::super::model::{CheckedMeasure, IntegerType};
pub(crate) use super::super::places::{PlaceProjection, PlaceRoot, PlaceTerm, ProjectedPlaceTerm};
use crate::DeclarationId;

/// Which once-captured endpoint one private counted-range term denotes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CountedCaptureSide {
    Lower,
    Upper,
}

/// One [ENT-2] term.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum TermKind {
    /// The distinguished zero term Z, carrying constant bounds.
    Zero,
    /// The mathematical value of an integer literal or integer-typed named
    /// const. Interning constants as terms lets disequalities and bounds share
    /// one representation; the implicit equality to Z folds them back.
    Constant(i128),
    /// An in-scope integer-typed const-generic parameter, judged symbolically.
    ConstParameter(DeclarationId),
    /// A tracked place whose final selected type is one fragment type.
    Place(PlaceTerm, IntegerType),
    /// The same [ENT-2] tracked-place class when field selections precede a
    /// deref or more than one deref occurs in the canonical spelling.
    ProjectedPlace(ProjectedPlaceTerm, IntegerType),
    /// One measure term `len_of(P)`, `cap_of(P)`, `room_of(P)` or `head_of(P)` [MSR-1],
    /// of fragment type u64. Its support is P's descriptor storage [MSR-2].
    Measure(CheckedMeasure, PlaceTerm),
    /// A measure term whose place has interleaved field/deref projections.
    ProjectedMeasure(CheckedMeasure, ProjectedPlaceTerm),
    /// One immutable compiler-owned endpoint capture [ENT-2, S11]. The
    /// finalized `for_stmt` path plus the endpoint side is its complete
    /// function-local identity; source can neither name nor mutate it.
    CountedCapture {
        range_path: Vec<u32>,
        side: CountedCaptureSide,
    },
    /// One immutable compiler-owned commit value [ENT-2]: the value the
    /// right-hand side of one `set` statement evaluated to at that
    /// occurrence, before its target kill. The statement's finalized
    /// NodePath plus the value's fragment type is its complete function-local
    /// identity; source can neither name nor mutate it. The flow visits that
    /// statement once, so this one term denotes its value in the single
    /// abstract evaluation the walk performs, as a counted header image does
    /// for an arbitrary iteration.
    CommitValue {
        commit_path: Vec<u32>,
        ty: IntegerType,
    },
    /// One immutable compiler-owned call datum [ENT-3.S13, MSR-3]: the value
    /// one `own` operand of a declared relation had at that call's
    /// pre-transfer point, or one measure of it. The call's finalized
    /// NodePath, the formal ordinal, the operand's ordered projections, and
    /// and which measure of it the datum denotes, if any, are its complete
    /// function-local identity [MSR-1]. No place occurs in it, so no
    /// [ENT-5] event kills it and a relation stated over it survives the
    /// consume the same statement performs.
    CallDatum {
        call_path: Vec<u32>,
        formal: u32,
        projections: Vec<CallDatumProjection>,
        measure: Option<CheckedMeasure>,
        ty: IntegerType,
    },
    /// One immutable compiler-owned entry datum [MSR-3]: the value one
    /// [MSR-1] measure of an `own` or shared-borrow parameter had at body
    /// entry. The formal ordinal, the operand's ordered projections and which
    /// measure of it the datum denotes are its complete function-local
    /// identity. No place occurs in it, so no [ENT-5] event kills it: that is
    /// what makes an `ensures` naming a parameter's measure mean the entry
    /// value even where the body writes that parameter back [LIV-2], and it
    /// is the same datum a caller substitutes as that call's call datum.
    EntryDatum {
        formal: u32,
        projections: Vec<CallDatumProjection>,
        measure: CheckedMeasure,
    },
}

/// Ordered projection identity inside one call datum's operand place.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CallDatumProjection {
    Deref,
    Field(u32),
}

/// Dense identity of one interned term.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub(crate) struct TermId(pub(crate) u32);

/// One implicit [ENT-2] equality a measure term carries at every program
/// point: the `array<T, N>` `len` and `cap` equality to N, with concrete N a
/// constant and const-generic N a symbolic constant term, and [MSR-2]'s
/// standing constant for a table cell whose value is fixed. Implicit facts
/// hold at every program point and never die.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MeasureBound {
    Constant(i128),
    Equal(TermId),
}

/// The zero term is always interned first.
pub(crate) const ZERO: TermId = TermId(0);

/// Function-scoped term registry. Only terms written in the function
/// participate [ENT-4]; the registry grows monotonically during the forward
/// walk. A term registered after a query cannot change that query's answer:
/// every implicit fact relates a term to Z (or, for an array length, to its
/// constant length term), so any derivation hopping through a later term's
/// implicit bounds factors through Z, and no other fact mentions an
/// unregistered term.
pub(crate) struct TermTable {
    terms: Vec<TermKind>,
    ids: HashMap<TermKind, TermId>,
    measure_bounds: HashMap<TermId, MeasureBound>,
}

impl TermTable {
    pub(crate) fn new() -> Self {
        let mut table = Self {
            terms: Vec::new(),
            ids: HashMap::new(),
            measure_bounds: HashMap::new(),
        };
        let zero = table.intern(TermKind::Zero);
        debug_assert_eq!(zero, ZERO);
        table
    }

    pub(crate) fn set_measure_bound(&mut self, term: TermId, bound: MeasureBound) {
        self.measure_bounds.insert(term, bound);
    }

    pub(crate) fn measure_bound(&self, term: TermId) -> Option<MeasureBound> {
        self.measure_bounds.get(&term).copied()
    }

    /// Another measure of the same place as `term`, when that measure term is
    /// registered. [MSR-2]'s standing orderings relate two measures of one
    /// place, so the table has to find the sibling by its place.
    pub(crate) fn sibling_measure(&self, term: TermId, measure: CheckedMeasure) -> Option<TermId> {
        let sibling = match self.kind(term) {
            TermKind::Measure(_, place) => TermKind::Measure(measure, place.clone()),
            TermKind::ProjectedMeasure(_, place) => {
                TermKind::ProjectedMeasure(measure, place.clone())
            }
            _ => return None,
        };
        self.interned(&sibling)
    }

    /// Interns one term, canonicalizing the written constant zero to Z.
    ///
    /// Relations are over mathematical values [ENT-2], so a written `0_T`
    /// and the distinguished zero term denote the same value and must be
    /// one term. Kept apart, a disequality reaches Z only by a bound
    /// strengthened through the constant's implicit equality, which exists
    /// only where the fragment already bounds the operand on that side: a
    /// a source relation `d != 0_i32` then could not discharge an obligation stated
    /// against Z at a signed type, and [OP-2]'s own mechanical fix would be
    /// unwritable. Z carries exactly the bounds the constant zero would
    /// have contributed, so the merge loses no fact.
    pub(crate) fn intern(&mut self, kind: TermKind) -> TermId {
        let kind = if matches!(kind, TermKind::Constant(0)) {
            TermKind::Zero
        } else {
            kind
        };
        if let Some(id) = self.ids.get(&kind) {
            return *id;
        }
        let id = TermId(
            u32::try_from(self.terms.len())
                .expect("ENT term inventory exceeds the u32 identity space"),
        );
        self.terms.push(kind.clone());
        self.ids.insert(kind, id);
        id
    }

    /// The identity of one already interned term, without interning it.
    pub(crate) fn interned(&self, kind: &TermKind) -> Option<TermId> {
        self.ids.get(kind).copied()
    }

    pub(crate) fn kind(&self, id: TermId) -> &TermKind {
        &self.terms[id.0 as usize]
    }

    /// Every registered term, for implicit-fact materialization [ENT-4].
    pub(crate) fn ids(&self) -> impl Iterator<Item = TermId> {
        (0..self.terms.len()).map(|index| {
            TermId(u32::try_from(index).expect("ENT term inventory exceeds the u32 identity space"))
        })
    }

    pub(crate) fn into_inventory(self) -> (Vec<TermKind>, Vec<Option<MeasureBound>>) {
        let measure_bounds = (0..self.terms.len())
            .map(|index| {
                let id = TermId(
                    u32::try_from(index)
                        .expect("ENT term inventory exceeds the u32 identity space"),
                );
                self.measure_bounds.get(&id).copied()
            })
            .collect();
        (self.terms, measure_bounds)
    }
}

/// Inclusive value range of one fragment type, as mathematical integers.
pub(crate) const fn type_range(ty: IntegerType) -> (i128, i128) {
    match ty {
        IntegerType::I8 => (i8::MIN as i128, i8::MAX as i128),
        IntegerType::I16 => (i16::MIN as i128, i16::MAX as i128),
        IntegerType::I32 => (i32::MIN as i128, i32::MAX as i128),
        IntegerType::I64 => (i64::MIN as i128, i64::MAX as i128),
        IntegerType::U8 => (0, u8::MAX as i128),
        IntegerType::U16 => (0, u16::MAX as i128),
        IntegerType::U32 => (0, u32::MAX as i128),
        IntegerType::U64 => (0, u64::MAX as i128),
    }
}

/// The mathematical value of one checked integer constant, whose `bits` hold
/// the type-width two's-complement pattern.
pub(crate) const fn integer_value(ty: IntegerType, bits: u64) -> i128 {
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
