//! [ENT-2] terms: the closed vocabulary the L0 fragment relates.
//!
//! A term is a tracked place, a length term over a place, one of the two
//! private endpoint captures of a written counted range, a constant, a
//! symbolic const-generic parameter, or the distinguished zero term Z. Term
//! identity is declaration-anchored: two places are the same term exactly when
//! their roots resolve to the same declaration event — one [`BindingId`] in
//! one checked function — and their canonical spellings agree, which the
//! structural representation below captures byte-for-byte for canonical
//! source. Identity deliberately under-approximates aliasing; kills use the
//! [OWN-7] overlap relation over resolved places instead, which
//! over-approximates it [ENT-5].

use std::collections::HashMap;

use super::super::model::{BindingId, CheckedConstantId, IntegerType};
use crate::DeclarationId;

/// Which once-captured endpoint one private counted-range term denotes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CountedCaptureSide {
    Lower,
    Upper,
}

/// Root of a tracked place: a function-local binding (parameters, `let`
/// bindings of every right-hand form, and match binders share the dense
/// [`BindingId`] space) or a named const [CONST-2].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PlaceRoot {
    Binding(BindingId),
    Constant(CheckedConstantId),
}

/// One tracked place [ENT-2](a): a root, an optional `deref` reading through
/// a borrow or box holder, and field selections — never an index segment.
///
/// This compact form represents no deref or one leading deref followed by
/// fields. Interleaved or repeated derefs use [`ProjectedPlaceTerm`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PlaceTerm {
    pub(crate) root: PlaceRoot,
    pub(crate) deref: bool,
    pub(crate) fields: Vec<u32>,
}

/// One source-order projection in a tracked place whose spelling cannot be
/// represented by [`PlaceTerm`]'s legacy "one leading deref, then fields"
/// shape. Keeping the order makes `deref(h.value)` distinct from
/// `deref(h).value`, as [ENT-2] requires.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PlaceProjection {
    Field(u32),
    Deref,
}

/// The exact source-order path of a tracked place with interleaved field and
/// deref projections. The root remains declaration-anchored; projections are
/// finite because the checked expression tree is finite.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ProjectedPlaceTerm {
    pub(crate) root: PlaceRoot,
    pub(crate) projections: Vec<PlaceProjection>,
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
    /// The length term `len(P)`, of fragment type u64.
    Length(PlaceTerm),
    /// A length term whose place has interleaved field/deref projections.
    ProjectedLength(ProjectedPlaceTerm),
    /// One immutable compiler-owned endpoint capture [ENT-2, S11]. The
    /// finalized `for_stmt` path plus the endpoint side is its complete
    /// function-local identity; source can neither name nor mutate it.
    CountedCapture {
        range_path: Vec<u32>,
        side: CountedCaptureSide,
    },
}

/// Dense identity of one interned term.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub(crate) struct TermId(pub(crate) u32);

/// The implicit [ENT-2] length equality of one length term whose place has
/// type `array<T, N>`: concrete N is a constant, const-generic N a symbolic
/// constant term. Implicit facts hold at every program point and never die.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LengthBound {
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
    length_bounds: HashMap<TermId, LengthBound>,
}

impl TermTable {
    pub(crate) fn new() -> Self {
        let mut table = Self {
            terms: Vec::new(),
            ids: HashMap::new(),
            length_bounds: HashMap::new(),
        };
        let zero = table.intern(TermKind::Zero);
        debug_assert_eq!(zero, ZERO);
        table
    }

    pub(crate) fn set_length_bound(&mut self, term: TermId, bound: LengthBound) {
        self.length_bounds.insert(term, bound);
    }

    pub(crate) fn length_bound(&self, term: TermId) -> Option<LengthBound> {
        self.length_bounds.get(&term).copied()
    }

    pub(crate) fn intern(&mut self, kind: TermKind) -> TermId {
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

    pub(crate) fn kind(&self, id: TermId) -> &TermKind {
        &self.terms[id.0 as usize]
    }

    /// Every registered term, for implicit-fact materialization [ENT-4].
    pub(crate) fn ids(&self) -> impl Iterator<Item = TermId> {
        (0..self.terms.len()).map(|index| {
            TermId(u32::try_from(index).expect("ENT term inventory exceeds the u32 identity space"))
        })
    }

    pub(crate) fn into_inventory(self) -> (Vec<TermKind>, Vec<Option<LengthBound>>) {
        let length_bounds = (0..self.terms.len())
            .map(|index| {
                let id = TermId(
                    u32::try_from(index)
                        .expect("ENT term inventory exceeds the u32 identity space"),
                );
                self.length_bounds.get(&id).copied()
            })
            .collect();
        (self.terms, length_bounds)
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
