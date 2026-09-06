//! The [BLK-0] kernel declaration domain's signature records.
//!
//! [BLK-0] states that each operation of the container and store domain is
//! one complete signature record: its type, const and region parameters in
//! declared order, its named value parameters, one declared effect row, one
//! declared result mode and type or one ordered result list, one declared
//! requirement list, and one declared relation list. This module is that
//! record data, and the checker, the publication source and the lowering all
//! read it generically: nothing selects behaviour from a spelling, and the
//! only per-row discriminant is [`crate::KernelRow`], which is the record's
//! own identity.
//!
//! The resolver's table [`crate::resolution::kernel`] carries what resolution
//! needs — spellings, parameter names, result binder spellings. This table
//! carries what checking needs, in a closed shape language that is exactly as
//! wide as the twelve rows of the inventory: [BLK-2]'s formation and
//! reservation rows and [BLK-3]'s four boundary rows.

use crate::KernelRow;

use super::model::CheckedMeasure;

/// The type of one value parameter or one declared result of a kernel row, in
/// the closed shape language the inventory writes.
///
/// A shape names a type only up to the row's own parameters: the element type
/// `T`, the run type parameter `V` [BLK-3], the store region `'s`, and the two
/// or three const parameters. One resolved instance fixes every one of
/// them at a call, so a shape plus one resolved instance is a checked type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KernelShape {
    /// `own u64`, the one primitive this domain writes.
    U64,
    /// The element type `T`.
    Element,
    /// The run type parameter `V` [BLK-3], supplied by the `vector` operand.
    Run,
    /// `FixedVector<T, n>`.
    FixedVector,
    /// `Vector<'s, T>`.
    Vector,
    /// `Option<Vector<'s, T>>`.
    OptionVector,
    /// `Result<Box<'s, T>, T>` S39: the outcome of a cell formation, whose
    /// refusal arm hands the value back because the row consumed it.
    ResultBox,
    /// `Arena<'s, bytes, align>`.
    Extent,
    /// `Heap<'s>`.
    Heap,
    /// [VIEW-2]'s **viewable** operand class: the storage a view may be
    /// formed over.
    ///
    /// It is a class rather than one parameter because the class is wider
    /// than any one type — the two runs [BLK-1] and, until S34 retires
    /// them, `array<T, N>` and `buffer<T>` — and because nothing in the
    /// formation reads what that storage is made of. The row's element type
    /// is the operand's own element, so the class supplies `T`, and the
    /// operand's borrow supplies the view's region.
    Viewable,
    /// `Slice<'r, T>`.
    Slice,
    /// `MutSlice<'r, T>`.
    MutSlice,
}

/// The mode of one value parameter [OWN-2]. The domain writes exactly two:
/// every transformed run and every count is `own`, and every provider is a
/// `&uniq` state operand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KernelMode {
    Own,
    /// A `&'r` operand: the mode the shared view's formation row takes
    /// [VIEW-2]. It reads the operand's state and takes one shared access to
    /// it [OWN-5], and the value the row produces — not this borrow — holds
    /// the loan [PROV-3].
    Shared,
    Unique,
}

/// One declared value parameter of a row [GRAM-11].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KernelParameter {
    pub(crate) name: &'static str,
    pub(crate) mode: KernelMode,
    pub(crate) shape: KernelShape,
}

/// One declared result of a row, in the ordered result list [FN-1, CALL-4].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KernelResult {
    pub(crate) name: &'static str,
    pub(crate) shape: KernelShape,
}

/// One written type or const parameter of a row, and which operand supplies
/// it when an operand does [BLK-0].
///
/// [BLK-0] decides written arguments per argument, not per callee: a type or
/// const argument is written exactly when no operand of that row supplies it.
/// `supplied` records that judgment as record data rather than as a rule the
/// checker re-derives per row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KernelGenericParameter {
    pub(crate) name: &'static str,
    pub(crate) kind: KernelGenericKind,
    /// `true` when an operand of this row determines this parameter, so the
    /// call writes no argument for it.
    pub(crate) supplied: bool,
}

/// Which generic axis one written parameter occupies [GRAM-2].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KernelGenericKind {
    /// The element type `T`.
    Type,
    /// A `u64` const parameter, identified by its position in the row's own
    /// const list.
    Const(KernelConst),
    /// The store region `'s`.
    Region,
}

/// Which const parameter of the inventory one written const argument is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum KernelConst {
    /// `FixedVector<T, n>`'s capacity.
    Capacity,
    /// An `Arena`'s byte extent.
    Bytes,
    /// An `Arena`'s alignment.
    Align,
}

/// The place one operand of a declared requirement or relation names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KernelPlace {
    /// One value parameter, at the denotation [MSR-3]'s table gives its mode:
    /// an `own` operand is that call's call datum and a `&uniq` state operand
    /// is the post-state.
    Parameter(u32),
    /// `<measure>(<parameter> at the call)`: that call's call datum for the
    /// same place, which is the form [BLK-0] admits for a `&uniq` state
    /// operand.
    ParameterAtCall(u32),
    /// One declared result ordinal [CALL-4].
    Result(u32),
    /// The payload binder of a routed clause, which names the result the
    /// route's variant carries.
    Payload,
}

/// One operand of a declared requirement or relation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KernelOperand {
    /// One [MSR-1] measure of a place.
    Measure(CheckedMeasure, KernelPlace),
    /// One value parameter read as its own `u64` datum.
    Value(u32),
    /// One const parameter of this instance.
    Const(KernelConst),
    /// `align_ceiling(T)`, the layout ceiling [OP-9] fixes, which is a
    /// compile-time constant of one concrete instance.
    AlignCeiling,
    /// The zero term, so a bare literal is one operand shape rather than two.
    Zero,
}

/// The written displacement of one operand.
///
/// [BLK-0] makes `advance<T>(count)` a symbolic constant when `count` is
/// closed and an opaque term otherwise, and says that a relation over it is
/// an ordinary difference bound between two terms. That is exactly this
/// shape: an `advance` displacement resolves to a constant at a closed count
/// and to nothing at an open one, and a relation whose displacement does not
/// resolve is simply unavailable, which only under-derives [ENT-1].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KernelOffset {
    Constant(i64),
    /// `advance<T>(count)` over the named value parameter [BLK-0].
    Advance(u32),
    /// `advance<T>(1)`: the bytes one cell occupies S39. A cell row has no
    /// count operand, so its advance names no parameter; the quantity is the
    /// same stride rounded up to the store's own alignment constant.
    AdvanceCell,
}

/// One operand displaced by a written constant: every relation of the
/// inventory is an ordinary difference bound between two such terms, which is
/// exactly what [ENT-4]'s closure represents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KernelTerm {
    pub(crate) operand: KernelOperand,
    pub(crate) offset: KernelOffset,
}

impl KernelTerm {
    pub(crate) const fn new(operand: KernelOperand) -> Self {
        Self {
            operand,
            offset: KernelOffset::Constant(0),
        }
    }

    pub(crate) const fn plus(operand: KernelOperand, offset: i64) -> Self {
        Self {
            operand,
            offset: KernelOffset::Constant(offset),
        }
    }

    /// `<operand> + advance<T>(<count>)`.
    pub(crate) const fn advanced(operand: KernelOperand, count: u32) -> Self {
        Self {
            operand,
            offset: KernelOffset::Advance(count),
        }
    }

    pub(crate) const fn constant(value: i64) -> Self {
        Self {
            operand: KernelOperand::Zero,
            offset: KernelOffset::Constant(value),
        }
    }

    /// The bare `advance<T>(count)` term, which is the zero term displaced by
    /// that acquisition's own advance.
    pub(crate) const fn advance(count: u32) -> Self {
        Self {
            operand: KernelOperand::Zero,
            offset: KernelOffset::Advance(count),
        }
    }

    /// `<operand> + advance<T>(1)` S39.
    pub(crate) const fn advanced_by_a_cell(operand: KernelOperand) -> Self {
        Self {
            operand,
            offset: KernelOffset::AdvanceCell,
        }
    }

    /// The bare `advance<T>(1)` term S39.
    pub(crate) const fn advance_cell() -> Self {
        Self {
            operand: KernelOperand::Zero,
            offset: KernelOffset::AdvanceCell,
        }
    }
}

/// The comparison one declared requirement or relation writes [GRAM-5].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KernelComparison {
    Equal,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

/// The route one relation is restricted to [CALL-6], where the row's result
/// is an enum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KernelRoute {
    /// `when made is Some(value: r):`.
    Some,
    /// `when made is None():`.
    None,
    /// `when made is Ok(value: b):` S39.
    Ok,
    /// `when made is Err(error: back):` S39.
    Err,
}

/// One declared requirement or relation of a row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KernelRelation {
    /// `None` for an unrouted clause, which is available on every exit.
    pub(crate) route: Option<KernelRoute>,
    pub(crate) left: KernelTerm,
    pub(crate) comparison: KernelComparison,
    pub(crate) right: KernelTerm,
}

impl KernelRelation {
    const fn plain(left: KernelTerm, comparison: KernelComparison, right: KernelTerm) -> Self {
        Self {
            route: None,
            left,
            comparison,
            right,
        }
    }

    const fn routed(
        route: KernelRoute,
        left: KernelTerm,
        comparison: KernelComparison,
        right: KernelTerm,
    ) -> Self {
        Self {
            route: Some(route),
            left,
            comparison,
            right,
        }
    }

    /// The difference bounds `left - right <= c` this relation states, as
    /// [ENT-4] represents them. An equality states both directions.
    ///
    /// `displacement` resolves each written offset to its mathematical value
    /// at one concrete instance; a displacement it cannot resolve leaves the
    /// relation unavailable rather than approximated.
    pub(crate) fn bounds(
        &self,
        displacement: impl Fn(KernelOffset) -> Option<i128>,
    ) -> Option<Vec<KernelBound>> {
        // `left.operand + left.offset <compare> right.operand + right.offset`
        // is `left.operand - right.operand <compare> right.offset -
        // left.offset`, so the whole displacement folds into the constant.
        let gap = displacement(self.right.offset)?.checked_sub(displacement(self.left.offset)?)?;
        let forward = KernelBound {
            left: self.left.operand,
            right: self.right.operand,
            bound: gap,
        };
        let backward = KernelBound {
            left: self.right.operand,
            right: self.left.operand,
            bound: -gap,
        };
        Some(match self.comparison {
            KernelComparison::Equal => vec![forward, backward],
            KernelComparison::LessOrEqual => vec![forward],
            KernelComparison::Less => vec![KernelBound {
                bound: gap.checked_sub(1)?,
                ..forward
            }],
            KernelComparison::GreaterOrEqual => vec![backward],
            KernelComparison::Greater => vec![KernelBound {
                bound: gap.checked_neg()?.checked_sub(1)?,
                ..backward
            }],
        })
    }
}

/// One difference bound `left - right <= bound` over two operands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KernelBound {
    pub(crate) left: KernelOperand,
    pub(crate) right: KernelOperand,
    pub(crate) bound: i128,
}

/// The declared effect row of one kernel operation [EFF-1], in [EFF-1]'s own
/// canonical order. Each entry names the value parameter whose state the row
/// reads, writes, or allocates from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KernelEffects {
    pub(crate) reads: Option<u32>,
    pub(crate) writes: Option<u32>,
    pub(crate) allocates: Option<u32>,
}

impl KernelEffects {
    const PURE: Self = Self {
        reads: None,
        writes: None,
        allocates: None,
    };

    /// `reads(P)`: a row that observes its operand and writes nothing.
    const fn reading(parameter: u32) -> Self {
        Self {
            reads: Some(parameter),
            writes: None,
            allocates: None,
        }
    }

    const fn over(parameter: u32, allocates: bool) -> Self {
        Self {
            reads: Some(parameter),
            writes: Some(parameter),
            allocates: if allocates { Some(parameter) } else { None },
        }
    }
}

/// One complete [BLK-0] signature record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KernelSignature {
    pub(crate) row: KernelRow,
    /// The row's exact IDENT spelling [BLK-0], carried here because a record
    /// has no source declaration and a diagnostic arising in this domain
    /// names the operation in its payload.
    pub(crate) spelling: &'static str,
    /// Written and supplied generic parameters in the order [GRAM-2] writes
    /// them: type parameters, then const parameters, then region parameters.
    pub(crate) generics: &'static [KernelGenericParameter],
    pub(crate) parameters: &'static [KernelParameter],
    pub(crate) results: &'static [KernelResult],
    pub(crate) effects: KernelEffects,
    /// The declared requirement list, each clause an obligation the caller
    /// submits under [MSR-4].
    pub(crate) requires: &'static [KernelRelation],
    /// The declared relation list, published per [CALL-6] at the caller.
    pub(crate) ensures: &'static [KernelRelation],
    /// Whether this row carries [OP-9]'s allocation-fit obligation over
    /// `(T, count)`, which the record notation spells `fits::<T>(count)` and
    /// which is not a term [BLK-0].
    pub(crate) fits: Option<u32>,
}

const TYPE_WRITTEN: KernelGenericParameter = KernelGenericParameter {
    name: "T",
    kind: KernelGenericKind::Type,
    supplied: false,
};

const TYPE_SUPPLIED: KernelGenericParameter = KernelGenericParameter {
    name: "T",
    kind: KernelGenericKind::Type,
    supplied: true,
};

const CAPACITY_WRITTEN: KernelGenericParameter = KernelGenericParameter {
    name: "n",
    kind: KernelGenericKind::Const(KernelConst::Capacity),
    supplied: false,
};

const BYTES_WRITTEN: KernelGenericParameter = KernelGenericParameter {
    name: "bytes",
    kind: KernelGenericKind::Const(KernelConst::Bytes),
    supplied: false,
};

const BYTES_SUPPLIED: KernelGenericParameter = KernelGenericParameter {
    name: "bytes",
    kind: KernelGenericKind::Const(KernelConst::Bytes),
    supplied: true,
};

const ALIGN_WRITTEN: KernelGenericParameter = KernelGenericParameter {
    name: "align",
    kind: KernelGenericKind::Const(KernelConst::Align),
    supplied: false,
};

const ALIGN_SUPPLIED: KernelGenericParameter = KernelGenericParameter {
    name: "align",
    kind: KernelGenericKind::Const(KernelConst::Align),
    supplied: true,
};

const REGION_WRITTEN: KernelGenericParameter = KernelGenericParameter {
    name: "'s",
    kind: KernelGenericKind::Region,
    supplied: false,
};

const REGION_SUPPLIED: KernelGenericParameter = KernelGenericParameter {
    name: "'s",
    kind: KernelGenericKind::Region,
    supplied: true,
};

/// `fixed_vector<T, const n: u64>() -> result: own FixedVector<T, n> pure`.
const SEQ_FIXED: KernelSignature = KernelSignature {
    row: KernelRow::FixedVector,
    spelling: "fixed_vector",
    generics: &[TYPE_WRITTEN, CAPACITY_WRITTEN],
    parameters: &[],
    results: &[KernelResult {
        name: "result",
        shape: KernelShape::FixedVector,
    }],
    effects: KernelEffects::PURE,
    requires: &[],
    ensures: &[
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::constant(0),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Const(KernelConst::Capacity)),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Room,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Const(KernelConst::Capacity)),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Head,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::constant(0),
        ),
    ],
    fits: None,
};

/// The four measures a formation row publishes over its own new run, where
/// that run is the routed payload rather than the whole result.
const SEQ_ARENA_PAYLOAD: [KernelRelation; 4] = [
    KernelRelation::routed(
        KernelRoute::Some,
        KernelTerm::new(KernelOperand::Measure(
            CheckedMeasure::Length,
            KernelPlace::Payload,
        )),
        KernelComparison::Equal,
        KernelTerm::constant(0),
    ),
    KernelRelation::routed(
        KernelRoute::Some,
        KernelTerm::new(KernelOperand::Measure(
            CheckedMeasure::Capacity,
            KernelPlace::Payload,
        )),
        KernelComparison::Equal,
        KernelTerm::new(KernelOperand::Value(1)),
    ),
    KernelRelation::routed(
        KernelRoute::Some,
        KernelTerm::new(KernelOperand::Measure(
            CheckedMeasure::Room,
            KernelPlace::Payload,
        )),
        KernelComparison::Equal,
        KernelTerm::new(KernelOperand::Value(1)),
    ),
    KernelRelation::routed(
        KernelRoute::Some,
        KernelTerm::new(KernelOperand::Measure(
            CheckedMeasure::Head,
            KernelPlace::Payload,
        )),
        KernelComparison::Equal,
        KernelTerm::constant(0),
    ),
];

/// `arena_vector<T, const bytes, const align>['s](store, count) -> made: own
/// Option<Vector<'s, T>>`.
const SEQ_ARENA: KernelSignature = KernelSignature {
    row: KernelRow::ArenaVector,
    spelling: "arena_vector",
    generics: &[
        TYPE_WRITTEN,
        BYTES_SUPPLIED,
        ALIGN_SUPPLIED,
        REGION_SUPPLIED,
    ],
    parameters: &[
        KernelParameter {
            name: "store",
            mode: KernelMode::Unique,
            shape: KernelShape::Extent,
        },
        KernelParameter {
            name: "count",
            mode: KernelMode::Own,
            shape: KernelShape::U64,
        },
    ],
    results: &[KernelResult {
        name: "made",
        shape: KernelShape::OptionVector,
    }],
    effects: KernelEffects::over(0, true),
    requires: &[KernelRelation::plain(
        KernelTerm::new(KernelOperand::Const(KernelConst::Align)),
        KernelComparison::GreaterOrEqual,
        KernelTerm::new(KernelOperand::AlignCeiling),
    )],
    ensures: &[
        SEQ_ARENA_PAYLOAD[0],
        SEQ_ARENA_PAYLOAD[1],
        SEQ_ARENA_PAYLOAD[2],
        SEQ_ARENA_PAYLOAD[3],
        // `len_of(arena) == len_of(arena at the call) + advance<T>(count)`.
        KernelRelation::routed(
            KernelRoute::Some,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Parameter(0),
            )),
            KernelComparison::Equal,
            KernelTerm::advanced(
                KernelOperand::Measure(CheckedMeasure::Length, KernelPlace::ParameterAtCall(0)),
                1,
            ),
        ),
        KernelRelation::routed(
            KernelRoute::None,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Parameter(0),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::ParameterAtCall(0),
            )),
        ),
        KernelRelation::routed(
            KernelRoute::None,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Room,
                KernelPlace::Parameter(0),
            )),
            KernelComparison::Less,
            KernelTerm::advance(1),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::Parameter(0),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::ParameterAtCall(0),
            )),
        ),
    ],
    fits: Some(1),
};

/// `arena_vector_proved<T, const bytes, const align>['s](store, count) ->
/// result: own Vector<'s, T>`.
const SEQ_ARENA_PROVED: KernelSignature = KernelSignature {
    row: KernelRow::ArenaVectorProved,
    spelling: "arena_vector_proved",
    generics: &[
        TYPE_WRITTEN,
        BYTES_SUPPLIED,
        ALIGN_SUPPLIED,
        REGION_SUPPLIED,
    ],
    parameters: SEQ_ARENA.parameters,
    results: &[KernelResult {
        name: "result",
        shape: KernelShape::Vector,
    }],
    effects: KernelEffects::over(0, true),
    requires: &[
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Const(KernelConst::Align)),
            KernelComparison::GreaterOrEqual,
            KernelTerm::new(KernelOperand::AlignCeiling),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Room,
                KernelPlace::Parameter(0),
            )),
            KernelComparison::GreaterOrEqual,
            KernelTerm::advance(1),
        ),
    ],
    ensures: &[
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::constant(0),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Value(1)),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Room,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Value(1)),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Head,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::constant(0),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Parameter(0),
            )),
            KernelComparison::Equal,
            KernelTerm::advanced(
                KernelOperand::Measure(CheckedMeasure::Length, KernelPlace::ParameterAtCall(0)),
                1,
            ),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::Parameter(0),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::ParameterAtCall(0),
            )),
        ),
    ],
    fits: Some(1),
};

/// `heap_vector<T>['s](store, count) -> made: own Option<Vector<'s, T>>`.
///
/// A `Heap<'s>` operand reaches a call only as a written parameter of the
/// enclosing declaration [PROV-6]: [FN-7]'s standard-input row, which is the
/// one route by which a program *obtains* the general store's provider, is
/// still that rule's DEFERRED entry, so this row is reachable at a checked
/// declaration and by no executable path.
const SEQ_HEAP: KernelSignature = KernelSignature {
    row: KernelRow::HeapVector,
    spelling: "heap_vector",
    generics: &[TYPE_WRITTEN, REGION_SUPPLIED],
    parameters: &[
        KernelParameter {
            name: "store",
            mode: KernelMode::Unique,
            shape: KernelShape::Heap,
        },
        KernelParameter {
            name: "count",
            mode: KernelMode::Own,
            shape: KernelShape::U64,
        },
    ],
    results: &[KernelResult {
        name: "made",
        shape: KernelShape::OptionVector,
    }],
    effects: KernelEffects::over(0, true),
    requires: &[],
    ensures: &SEQ_ARENA_PAYLOAD,
    fits: Some(1),
};

/// `arena_box<T, const bytes, const align>['s](store: &uniq Arena<'s, bytes,
/// align>, value: own T) -> made: own Result<Box<'s, T>, T>` S39.
///
/// The refusal hands `value` back, because unlike every run formation this
/// row **consumes** an affine input: a take of a run has nothing to return on
/// its refusal arm and an `Option` is enough there, while a cell formation
/// that dropped the value it was given would destroy it [L3]. That is why the
/// outcome is a `Result` and not an `Option`.
const ARENA_BOX: KernelSignature = KernelSignature {
    row: KernelRow::ArenaBox,
    spelling: "arena_box",
    generics: &[
        TYPE_SUPPLIED,
        BYTES_SUPPLIED,
        ALIGN_SUPPLIED,
        REGION_SUPPLIED,
    ],
    parameters: &BOX_PARAMETERS_ARENA,
    results: &[KernelResult {
        name: "made",
        shape: KernelShape::ResultBox,
    }],
    effects: KernelEffects::over(0, true),
    requires: &[KernelRelation::plain(
        KernelTerm::new(KernelOperand::Const(KernelConst::Align)),
        KernelComparison::GreaterOrEqual,
        KernelTerm::new(KernelOperand::AlignCeiling),
    )],
    ensures: &[
        // `len_of(store) == len_of(store at the call) + advance<T>(1)`.
        KernelRelation::routed(
            KernelRoute::Ok,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Parameter(0),
            )),
            KernelComparison::Equal,
            KernelTerm::advanced_by_a_cell(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::ParameterAtCall(0),
            )),
        ),
        KernelRelation::routed(
            KernelRoute::Err,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Parameter(0),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::ParameterAtCall(0),
            )),
        ),
        KernelRelation::routed(
            KernelRoute::Err,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Room,
                KernelPlace::Parameter(0),
            )),
            KernelComparison::Less,
            KernelTerm::advance_cell(),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::Parameter(0),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::ParameterAtCall(0),
            )),
        ),
    ],
    fits: None,
};

/// `heap_box<T>['s](store: &uniq Heap<'s>, value: own T) -> made: own
/// Result<Box<'s, T>, T>` S39.
///
/// `Heap<'s>` carries no measure, so this row publishes nothing: what it
/// hands back is decided by its own outcome and not by a store's state.
const HEAP_BOX: KernelSignature = KernelSignature {
    row: KernelRow::HeapBox,
    spelling: "heap_box",
    generics: &[TYPE_SUPPLIED, REGION_SUPPLIED],
    parameters: &BOX_PARAMETERS_HEAP,
    results: &[KernelResult {
        name: "made",
        shape: KernelShape::ResultBox,
    }],
    effects: KernelEffects::over(0, true),
    requires: &[],
    ensures: &[],
    fits: None,
};

/// The two value parameters each cell formation writes: the store's provider
/// and the value the cell takes [S39, BLK-0].
const BOX_PARAMETERS_ARENA: [KernelParameter; 2] = [
    KernelParameter {
        name: "store",
        mode: KernelMode::Unique,
        shape: KernelShape::Extent,
    },
    KernelParameter {
        name: "value",
        mode: KernelMode::Own,
        shape: KernelShape::Element,
    },
];

const BOX_PARAMETERS_HEAP: [KernelParameter; 2] = [
    KernelParameter {
        name: "store",
        mode: KernelMode::Unique,
        shape: KernelShape::Heap,
    },
    KernelParameter {
        name: "value",
        mode: KernelMode::Own,
        shape: KernelShape::Element,
    },
];

/// `arena_frame<const bytes, const align>['s]() -> result: own Arena<'s,
/// bytes, align> pure`.
const ARENA_FRAME: KernelSignature = KernelSignature {
    row: KernelRow::ArenaFrame,
    spelling: "arena_frame",
    generics: &[BYTES_WRITTEN, ALIGN_WRITTEN, REGION_WRITTEN],
    parameters: &[],
    results: &[KernelResult {
        name: "result",
        shape: KernelShape::Extent,
    }],
    effects: KernelEffects::PURE,
    requires: &[],
    ensures: &[
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::constant(0),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Const(KernelConst::Bytes)),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Room,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Const(KernelConst::Bytes)),
        ),
    ],
    fits: None,
};

/// The two value parameters every [BLK-3] placement row writes.
const PLACE_PARAMETERS: [KernelParameter; 2] = [
    KernelParameter {
        name: "vector",
        mode: KernelMode::Own,
        shape: KernelShape::Run,
    },
    KernelParameter {
        name: "value",
        mode: KernelMode::Own,
        shape: KernelShape::Element,
    },
];

/// The one value parameter every [BLK-3] removal row writes.
const TAKE_PARAMETERS: [KernelParameter; 1] = [KernelParameter {
    name: "vector",
    mode: KernelMode::Own,
    shape: KernelShape::Run,
}];

/// The ordered result list of the two removal rows.
const TAKE_RESULTS: [KernelResult; 2] = [
    KernelResult {
        name: "rest",
        shape: KernelShape::Run,
    },
    KernelResult {
        name: "value",
        shape: KernelShape::Element,
    },
];

/// `requires room_of(vector) > 0_u64;`
const ROOM_AVAILABLE: KernelRelation = KernelRelation::plain(
    KernelTerm::new(KernelOperand::Measure(
        CheckedMeasure::Room,
        KernelPlace::Parameter(0),
    )),
    KernelComparison::Greater,
    KernelTerm::constant(0),
);

/// `requires len_of(vector) > 0_u64;`
const LENGTH_AVAILABLE: KernelRelation = KernelRelation::plain(
    KernelTerm::new(KernelOperand::Measure(
        CheckedMeasure::Length,
        KernelPlace::Parameter(0),
    )),
    KernelComparison::Greater,
    KernelTerm::constant(0),
);

/// The three relations a placement row publishes over its own result before
/// the `head` cell, which the back and front rows write differently.
const fn placement_relations(result: u32) -> [KernelRelation; 3] {
    [
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Result(result),
            )),
            KernelComparison::Equal,
            KernelTerm::plus(
                KernelOperand::Measure(CheckedMeasure::Length, KernelPlace::Parameter(0)),
                1,
            ),
        ),
        KernelRelation::plain(
            KernelTerm::plus(
                KernelOperand::Measure(CheckedMeasure::Room, KernelPlace::Result(result)),
                1,
            ),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Room,
                KernelPlace::Parameter(0),
            )),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::Result(result),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::Parameter(0),
            )),
        ),
    ]
}

/// The three relations a removal row publishes before the `head` cell.
const fn removal_relations() -> [KernelRelation; 3] {
    [
        KernelRelation::plain(
            KernelTerm::plus(
                KernelOperand::Measure(CheckedMeasure::Length, KernelPlace::Result(0)),
                1,
            ),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Parameter(0),
            )),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Room,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::plus(
                KernelOperand::Measure(CheckedMeasure::Room, KernelPlace::Parameter(0)),
                1,
            ),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::Result(0),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::Parameter(0),
            )),
        ),
    ]
}

/// `ensures head_of(result) == head_of(vector);` — the back rows leave the window
/// origin where it was.
const fn head_retained(result: u32) -> KernelRelation {
    KernelRelation::plain(
        KernelTerm::new(KernelOperand::Measure(
            CheckedMeasure::Head,
            KernelPlace::Result(result),
        )),
        KernelComparison::Equal,
        KernelTerm::new(KernelOperand::Measure(
            CheckedMeasure::Head,
            KernelPlace::Parameter(0),
        )),
    )
}

/// The two-sided `head` publication of a front row [MSR-1]: the one bounded
/// cell of the table, which no row re-establishes exactly.
const fn head_bounded(result: u32) -> [KernelRelation; 2] {
    [
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Head,
                KernelPlace::Result(result),
            )),
            KernelComparison::GreaterOrEqual,
            KernelTerm::constant(0),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Head,
                KernelPlace::Result(result),
            )),
            KernelComparison::LessOrEqual,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::Result(result),
            )),
        ),
    ]
}

const PLACE_BACK: [KernelRelation; 3] = placement_relations(0);
const PLACE_FRONT_HEAD: [KernelRelation; 2] = head_bounded(0);
const TAKE_BACK: [KernelRelation; 3] = removal_relations();

/// `place_back(vector: own V, value: own T) -> result: own V`.
const SEQ_PLACE: KernelSignature = KernelSignature {
    row: KernelRow::PlaceBack,
    spelling: "place_back",
    generics: &[TYPE_SUPPLIED],
    parameters: &PLACE_PARAMETERS,
    results: &[KernelResult {
        name: "result",
        shape: KernelShape::Run,
    }],
    effects: KernelEffects::over(0, false),
    requires: &[ROOM_AVAILABLE],
    ensures: &[
        PLACE_BACK[0],
        PLACE_BACK[1],
        PLACE_BACK[2],
        head_retained(0),
    ],
    fits: None,
};

/// `place_front(vector: own V, value: own T) -> result: own V`.
const SEQ_PLACE_FRONT: KernelSignature = KernelSignature {
    row: KernelRow::PlaceFront,
    spelling: "place_front",
    generics: &[TYPE_SUPPLIED],
    parameters: &PLACE_PARAMETERS,
    results: &[KernelResult {
        name: "result",
        shape: KernelShape::Run,
    }],
    effects: KernelEffects::over(0, false),
    requires: &[ROOM_AVAILABLE],
    ensures: &[
        PLACE_BACK[0],
        PLACE_BACK[1],
        PLACE_BACK[2],
        PLACE_FRONT_HEAD[0],
        PLACE_FRONT_HEAD[1],
    ],
    fits: None,
};

/// `take_back(vector: own V) -> (rest: own V, value: own T)`.
const SEQ_TAKE: KernelSignature = KernelSignature {
    row: KernelRow::TakeBack,
    spelling: "take_back",
    generics: &[TYPE_SUPPLIED],
    parameters: &TAKE_PARAMETERS,
    results: &TAKE_RESULTS,
    effects: KernelEffects::over(0, false),
    requires: &[LENGTH_AVAILABLE],
    ensures: &[TAKE_BACK[0], TAKE_BACK[1], TAKE_BACK[2], head_retained(0)],
    fits: None,
};

/// `take_front(vector: own V) -> (rest: own V, value: own T)`.
const SEQ_TAKE_FRONT: KernelSignature = KernelSignature {
    row: KernelRow::TakeFront,
    spelling: "take_front",
    generics: &[TYPE_SUPPLIED],
    parameters: &TAKE_PARAMETERS,
    results: &TAKE_RESULTS,
    effects: KernelEffects::over(0, false),
    requires: &[LENGTH_AVAILABLE],
    ensures: &[
        TAKE_BACK[0],
        TAKE_BACK[1],
        TAKE_BACK[2],
        PLACE_FRONT_HEAD[0],
        PLACE_FRONT_HEAD[1],
    ],
    fits: None,
};

/// The one value parameter both [VIEW-2] formation rows write: the viewable
/// storage the view is formed over, borrowed at the row's own strength.
const VIEW_PARAMETER: [KernelParameter; 2] = [
    KernelParameter {
        name: "vector",
        mode: KernelMode::Shared,
        shape: KernelShape::Viewable,
    },
    KernelParameter {
        name: "vector",
        mode: KernelMode::Unique,
        shape: KernelShape::Viewable,
    },
];

/// `requires head_of(vector) <= room_of(vector);` — [VIEW-2]'s non-wrap
/// premise.
///
/// A view is one contiguous range and a wrapped window is two, so the row
/// admits exactly the operands whose window does not wrap. The rule states
/// the premise as `head_of + len_of <= cap_of`, which has three measure
/// operands; under [MSR-2]'s standing identity `len_of + room_of = cap_of` it
/// is exactly this difference bound between two terms, which is the shape
/// [ENT-4]'s closure carries and the record notation admits. An empty run
/// satisfies it from the standing `head_of <= cap_of` alone, and the two
/// retiring operand types satisfy it from their own table row, whose `head`
/// and `room` cells are both the constant zero [MSR-1].
const NON_WRAPPED: KernelRelation = KernelRelation::plain(
    KernelTerm::new(KernelOperand::Measure(
        CheckedMeasure::Head,
        KernelPlace::Parameter(0),
    )),
    KernelComparison::LessOrEqual,
    KernelTerm::new(KernelOperand::Measure(
        CheckedMeasure::Room,
        KernelPlace::Parameter(0),
    )),
);

/// The four relations a formation row publishes over the view it forms
/// [MSR-1]: the viewed extent is the view's own length and its capacity, and
/// a view is never a window.
const fn view_relations(result: u32) -> [KernelRelation; 4] {
    [
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Result(result),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Parameter(0),
            )),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Capacity,
                KernelPlace::Result(result),
            )),
            KernelComparison::Equal,
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Length,
                KernelPlace::Parameter(0),
            )),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Room,
                KernelPlace::Result(result),
            )),
            KernelComparison::Equal,
            KernelTerm::constant(0),
        ),
        KernelRelation::plain(
            KernelTerm::new(KernelOperand::Measure(
                CheckedMeasure::Head,
                KernelPlace::Result(result),
            )),
            KernelComparison::Equal,
            KernelTerm::constant(0),
        ),
    ]
}

const VIEW_RELATIONS: [KernelRelation; 4] = view_relations(0);

/// `slice_of['r, T](vector: &'r V) -> result: own Slice<'r, T> reads(vector)`.
const SLICE_OF: KernelSignature = KernelSignature {
    row: KernelRow::SliceOf,
    spelling: "slice_of",
    generics: &[TYPE_SUPPLIED, REGION_SUPPLIED],
    parameters: &[VIEW_PARAMETER[0]],
    results: &[KernelResult {
        name: "result",
        shape: KernelShape::Slice,
    }],
    effects: KernelEffects::reading(0),
    requires: &[NON_WRAPPED],
    ensures: &VIEW_RELATIONS,
    fits: None,
};

/// `mut_slice_of['r, T](vector: &uniq 'r V) -> result: own MutSlice<'r, T>
/// reads(vector)`.
const MUT_SLICE_OF: KernelSignature = KernelSignature {
    row: KernelRow::MutSliceOf,
    spelling: "mut_slice_of",
    generics: &[TYPE_SUPPLIED, REGION_SUPPLIED],
    parameters: &[VIEW_PARAMETER[1]],
    results: &[KernelResult {
        name: "result",
        shape: KernelShape::MutSlice,
    }],
    // The formation reads the operand's state and writes none of it: an
    // element write through the formed view is that view's own access to its
    // origin [PROV-3 use 1] and happens at the write, not here.
    effects: KernelEffects::reading(0),
    requires: &[NON_WRAPPED],
    ensures: &VIEW_RELATIONS,
    fits: None,
};

/// Every [BLK-0] signature record, in the `container_declaration_ordinal`
/// preorder [BLK-2]'s rows followed by [BLK-3]'s.
///
/// The last two are [VIEW-2]'s formation rows. Their record data is this
/// domain's — the operand class, the borrow mode, the non-wrap requirement
/// and the four published relations — while their *spelling* is still the
/// [OP-1] family entry every existing program writes, because the transitional
/// operand domain includes `array<T, N>` and `buffer<T>` and those two types
/// retire with S34. Two domains may not claim one spelling [TYPE-6], so the
/// spelling passes to the kernel IDENT domain in the same change that retires
/// them, and until then these two rows carry no resolver entry.
pub(crate) const KERNEL_SIGNATURES: [KernelSignature; 13] = [
    SEQ_FIXED,
    SEQ_ARENA,
    SEQ_ARENA_PROVED,
    SEQ_HEAP,
    ARENA_BOX,
    HEAP_BOX,
    ARENA_FRAME,
    SEQ_PLACE,
    SEQ_PLACE_FRONT,
    SEQ_TAKE,
    SEQ_TAKE_FRONT,
    SLICE_OF,
    MUT_SLICE_OF,
];

/// The `container_declaration_ordinal` of one row [BLK-0], which is its index
/// in the inventory above.
pub(crate) fn kernel_ordinal(row: KernelRow) -> u8 {
    u8::try_from(
        KERNEL_SIGNATURES
            .iter()
            .position(|signature| signature.row == row)
            .expect("every row is an inventory member"),
    )
    .expect("the inventory is far below u8")
}

/// The record at one `container_declaration_ordinal`.
pub(crate) fn kernel_signature_at(ordinal: u8) -> Option<&'static KernelSignature> {
    KERNEL_SIGNATURES.get(usize::from(ordinal))
}

/// `advance<T>(count)`, the bump domain's acquire quantity [BLK-0].
///
/// It is `round_up(stride_ceiling(T) * count, align)`: a run's slots are
/// stride-spaced, so `count` of them occupy `stride * count` bytes, and the
/// store's own alignment constant rounds that up so the cursor stays a
/// multiple of it at every program point [MSR-1].
///
/// The term is a symbolic constant when `count` is a closed expression and an
/// opaque term otherwise; this reader answers for the first case and hands
/// back `None` for the second, where a relation over it is simply unavailable
/// and a requirement over it cannot be built.
/// `advance<T>(1)` S39: the bytes one cell occupies, which is the stride
/// rounded up to the store's own alignment constant exactly as a take of one
/// slot would be.
pub(in crate::semantic) fn kernel_cell_advance(
    instance: &super::model::CheckedKernelInstance,
) -> Option<i128> {
    let super::model::CheckedLayoutMagnitude::Finite(stride) = instance.element_ceiling.stride
    else {
        return None;
    };
    let align = match instance.align {
        Some(super::model::CheckedConst::Value(align)) => align,
        // The general store has no alignment constant of its own; a cell it
        // hands out is one stride.
        None => return Some(i128::from(stride)),
        Some(_) => return None,
    };
    let rounded = stride.checked_add(align.checked_sub(1)?)? / align * align;
    Some(i128::from(rounded))
}

pub(in crate::semantic) fn kernel_advance(
    instance: &super::model::CheckedKernelInstance,
    goal_arguments: &[super::goal::GoalExpression],
    ordinal: u32,
) -> Option<i128> {
    let super::goal::GoalExpression::Datum(super::goal::GoalDatum::Literal(
        super::model::CheckedValue::Integer { bits, .. },
    )) = goal_arguments.get(ordinal as usize)?
    else {
        return None;
    };
    let super::model::CheckedConst::Value(align) = instance.align? else {
        return None;
    };
    let super::model::CheckedLayoutMagnitude::Finite(stride) = instance.element_ceiling.stride
    else {
        return None;
    };
    let bytes = stride.checked_mul(*bits)?;
    let rounded = bytes.checked_add(align.checked_sub(1)?)? / align * align;
    Some(i128::from(rounded))
}

/// The signature record of one resolved kernel row.
pub(crate) fn kernel_signature(row: KernelRow) -> &'static KernelSignature {
    KERNEL_SIGNATURES
        .iter()
        .find(|signature| signature.row == row)
        .expect("every inventory row has one signature record")
}

#[cfg(test)]
mod tests {
    use super::{
        KERNEL_SIGNATURES, KernelGenericKind, KernelOperand, KernelPlace, KernelShape,
        kernel_signature,
    };
    use crate::KERNEL_OPERATIONS;
    use crate::semantic::model::{CheckedMeasure, MeasuredKind};

    /// [BLK-0]: the two tables are one inventory. The resolver's row and the
    /// checker's record agree on every parameter name, every result binder
    /// spelling, the operation spelling, and the order of both.
    ///
    /// The record table is the longer of the two by exactly [VIEW-2]'s two
    /// formation rows, whose spelling is still an [OP-1] family entry while
    /// the transitional operand domain includes `array<T, N>` and
    /// `buffer<T>`; every row the *resolver* carries has a record, and the
    /// resolver's own index is the record's index, which is what
    /// `container_declaration_ordinal` names.
    #[test]
    fn every_resolved_row_has_the_record_it_resolves_to() {
        assert!(KERNEL_OPERATIONS.len() <= KERNEL_SIGNATURES.len());
        for (ordinal, operation) in KERNEL_OPERATIONS.iter().enumerate() {
            assert_eq!(
                KERNEL_SIGNATURES[ordinal].row, operation.row,
                "{}",
                operation.spelling
            );
            assert_eq!(
                kernel_signature(operation.row).spelling,
                operation.spelling,
                "{}",
                operation.spelling
            );
        }
        for operation in KERNEL_OPERATIONS {
            let signature = kernel_signature(operation.row);
            assert_eq!(
                operation.parameters.len(),
                signature.parameters.len(),
                "{}",
                operation.spelling
            );
            for (name, parameter) in operation.parameters.iter().zip(signature.parameters) {
                assert_eq!(*name, parameter.name, "{}", operation.spelling);
            }
            assert_eq!(
                operation.results.len(),
                signature.results.len(),
                "{}",
                operation.spelling
            );
            for (name, result) in operation.results.iter().zip(signature.results) {
                assert_eq!(*name, result.name, "{}", operation.spelling);
            }
        }
    }

    /// [BLK-0]: every row is complete over every measure it writes, on every
    /// exit. A row whose result is a run publishes all four of its measures
    /// on every exit its result reaches, and a row whose effect row writes a
    /// measured state operand publishes every measure that operand's table
    /// row has.
    #[test]
    fn every_row_is_complete_over_every_measure_it_writes() {
        for signature in KERNEL_SIGNATURES {
            // The exits a routed relation set partitions into: an unrouted
            // relation is a member of every route's set [CALL-6].
            let routes: Vec<_> = {
                let mut routes: Vec<_> = signature
                    .ensures
                    .iter()
                    .filter_map(|relation| relation.route)
                    .collect();
                routes.dedup();
                if routes.is_empty() { vec![] } else { routes }
            };
            let published = |place: KernelPlace, route: Option<super::KernelRoute>| {
                let mut measures: Vec<CheckedMeasure> = signature
                    .ensures
                    .iter()
                    .filter(|relation| relation.route.is_none() || relation.route == route)
                    .flat_map(|relation| [relation.left.operand, relation.right.operand])
                    .filter_map(|operand| match operand {
                        KernelOperand::Measure(measure, operand_place)
                            if operand_place == place =>
                        {
                            Some(measure)
                        }
                        _ => None,
                    })
                    .collect();
                measures.sort_unstable();
                measures.dedup();
                measures
            };
            // [BLK-0]'s completeness sentence, read with [MSR-2]: a row
            // publishes every cell the table gives the measured type, except
            // that `room` is the complement the standing identity `len + room
            // = cap` already determines with empty support, so a row that
            // publishes both of the other two has published it.
            let expected = |measured: MeasuredKind, published: &[CheckedMeasure]| {
                let determined = published.contains(&CheckedMeasure::Length)
                    && published.contains(&CheckedMeasure::Capacity);
                let mut measures: Vec<CheckedMeasure> = [
                    CheckedMeasure::Length,
                    CheckedMeasure::Capacity,
                    CheckedMeasure::Room,
                    CheckedMeasure::Head,
                ]
                .into_iter()
                .filter(|measure| {
                    let absent = matches!(
                        measure.cell(measured),
                        super::super::model::MeasureCell::Absent
                    );
                    let derived = determined
                        && *measure == CheckedMeasure::Room
                        && !published.contains(&CheckedMeasure::Room);
                    !absent && !derived
                })
                .collect();
                measures.sort_unstable();
                measures
            };
            // A `FixedVector`'s `cap` is its type constant and a run's four
            // measures are the complete row; a `Vector`'s `cap` is a runtime
            // quantity and is published the same way.
            for (index, result) in signature.results.iter().enumerate() {
                let measured = match result.shape {
                    KernelShape::FixedVector => MeasuredKind::FixedVector,
                    KernelShape::Vector | KernelShape::OptionVector => MeasuredKind::Vector,
                    KernelShape::Extent => MeasuredKind::Extent,
                    // `V` is either run and the two rows agree on every cell.
                    KernelShape::Run => MeasuredKind::FixedVector,
                    // The measure table gives both views one row [MSR-1].
                    KernelShape::Slice | KernelShape::MutSlice => MeasuredKind::Slice,
                    // S39 a cell carries no measure at all, so neither it
                    // nor the outcome that carries one has a row here.
                    KernelShape::U64
                    | KernelShape::Element
                    | KernelShape::Heap
                    | KernelShape::ResultBox
                    | KernelShape::Viewable => continue,
                };
                let place = if matches!(result.shape, KernelShape::OptionVector) {
                    KernelPlace::Payload
                } else {
                    KernelPlace::Result(
                        u32::try_from(index).expect("a result list is far below u32"),
                    )
                };
                let route = (!routes.is_empty()).then_some(super::KernelRoute::Some);
                // A run only exists on the `Some` arm, so completeness over a
                // routed result is stated over the arm the result reaches.
                let actual = published(place, route);
                assert_eq!(
                    actual,
                    expected(measured, &actual),
                    "{:?} is incomplete over its result",
                    signature.row
                );
            }
            // The state operand: a row whose effect row writes it publishes
            // every measure of its table row, on every declared exit.
            let Some(written) = signature.effects.writes else {
                continue;
            };
            let Some(parameter) = signature.parameters.get(written as usize) else {
                panic!("{:?} writes a parameter it does not declare", signature.row);
            };
            let measured = match parameter.shape {
                KernelShape::Extent => MeasuredKind::Extent,
                // A viewable operand is read and never written, so no row
                // reaches this arm through it.
                KernelShape::Viewable => continue,
                // The run a boundary row transforms is `own`, so its
                // post-state is its result rather than the operand.
                KernelShape::Run | KernelShape::Heap => continue,
                other => panic!("{other:?} is written by {:?}", signature.row),
            };
            let place = KernelPlace::Parameter(written);
            let exits: Vec<Option<super::KernelRoute>> = if routes.is_empty() {
                vec![None]
            } else {
                routes.iter().copied().map(Some).collect()
            };
            for route in exits {
                let actual = published(place, route);
                assert_eq!(
                    actual,
                    expected(measured, &actual),
                    "{:?} is incomplete over its state operand on {route:?}",
                    signature.row
                );
            }
        }
    }

    /// [BLK-0, CALL-6]: no row's own declared set is contradictory.
    ///
    /// A caller closes every relation it receives together [ENT-4], so a
    /// contradictory published set does not state one wrong fact — it
    /// discharges *every* obligation the caller submits after it, the
    /// subscript bounds among them. [CALL-6] refuses such a set at a source
    /// declaration; a row's set is fixed by this specification instead, so
    /// this test is where the same judgment is made over it, and a row that
    /// fails it is a defect in this repository's own data rather than in a
    /// program.
    ///
    /// The judgment is made once per declared exit — an unrouted clause is a
    /// member of every exit's set and a routed one of its own — over the
    /// requirements, which a caller has discharged before any relation of the
    /// row is established, together with the relations that exit carries.
    ///
    /// `advance<T>(count)` is the one operand of the record notation that is
    /// not a constant of the declaration [BLK-0], and a difference-bound
    /// closure carries a constant displacement rather than a symbolic one, so
    /// the set is judged at each of three resolutions of it: zero, one, and a
    /// take far larger than any written extent.
    #[test]
    fn no_row_publishes_a_contradictory_relation_set() {
        for signature in KERNEL_SIGNATURES {
            let mut exits: Vec<Option<super::KernelRoute>> = signature
                .ensures
                .iter()
                .filter_map(|relation| relation.route)
                .map(Some)
                .collect();
            exits.dedup();
            if exits.is_empty() {
                exits.push(None);
            }
            for exit in exits {
                for advance in [0, 1, 1 << 20] {
                    assert!(
                        !crate::semantic::check::publication::kernel_row_is_contradictory(
                            &signature, exit, advance
                        ),
                        "{:?} publishes a contradictory set on {exit:?} at advance {advance}",
                        signature.row
                    );
                }
            }
        }
    }

    /// [BLK-0]: every displaced requirement side is the zero term.
    ///
    /// A requirement is one obligation the caller discharges, so its written
    /// displacement has to reach the caller's goal. Where the displacement
    /// sits on the zero term it *is* the whole operand and the goal carries
    /// one literal; where it sat on a measure or a value it would need an
    /// arithmetic node the goal language does not admit here. The inventory
    /// writes only the first shape — `room_of(store) >= advance<T>(count)`
    /// and `room_of(vector) > 0_u64` — and this test is what keeps a later
    /// row from writing the second and having its displacement silently
    /// dropped.
    #[test]
    fn every_displaced_requirement_side_is_the_zero_term() {
        for signature in KERNEL_SIGNATURES {
            for clause in signature.requires {
                for term in [clause.left, clause.right] {
                    if matches!(term.operand, KernelOperand::Zero) {
                        continue;
                    }
                    assert_eq!(
                        term.offset,
                        super::KernelOffset::Constant(0),
                        "{:?} displaces a requirement operand that is not the zero term",
                        signature.row
                    );
                }
            }
        }
    }

    /// [BLK-0]: written arguments are decided per argument. A parameter no
    /// operand determines is written, and one an operand supplies is not.
    #[test]
    fn every_generic_parameter_records_whether_an_operand_supplies_it() {
        for signature in KERNEL_SIGNATURES {
            for generic in signature.generics {
                let supplied = signature.parameters.iter().any(|parameter| {
                    matches!(
                        (generic.kind, parameter.shape),
                        (
                            KernelGenericKind::Type,
                            KernelShape::Element | KernelShape::Run
                        ) | (KernelGenericKind::Type, KernelShape::Viewable)
                            | (
                                KernelGenericKind::Region,
                                KernelShape::Extent | KernelShape::Heap | KernelShape::Vector,
                            )
                            | (
                                // [VIEW-2] the operand's own borrow takes the
                                // view's region [FORM-8], so no argument is
                                // written for it.
                                KernelGenericKind::Region,
                                KernelShape::Viewable
                            )
                            | (KernelGenericKind::Const(_), KernelShape::Extent)
                    )
                });
                assert_eq!(
                    generic.supplied, supplied,
                    "{:?} disagrees about {}",
                    signature.row, generic.name
                );
            }
        }
    }
}
