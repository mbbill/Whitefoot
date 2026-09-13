use crate::{BuiltinPreludeId, DeclarationId, NodePath};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct FunctionId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BindingId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedMode {
    Own,
    Shared(DeclarationId),
    Unique(DeclarationId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CheckedLoopId(pub(crate) u32);

/// One proof-only mathematical integer expression. Each leaf retains
/// its exact source integer type while denoting its value in the mathematical
/// integers; this metadata does not request a runtime conversion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedAffineExpression {
    pub(crate) node_path: NodePath,
    pub(crate) kind: CheckedAffineExpressionKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedAffineExpressionKind {
    Constant {
        value: i128,
        ty: IntegerType,
    },
    Local {
        binding: BindingId,
        ty: IntegerType,
    },
    /// [INV-1, MSR-6] one in-scope const generic written as an affine atom,
    /// at the one source-canonical symbolic instance.
    ///
    /// A concrete instance reads the mathematical constant [FN-2] fixed for
    /// it and forms [`Self::Constant`]; the symbolic instance keeps the
    /// declaration-anchored constant term [ENT-2] clause (c) already gives
    /// it, which nothing kills, so the affine image is one immutable atom for
    /// the whole walk.
    ConstGeneric {
        declaration: DeclarationId,
        ty: IntegerType,
        /// The writer's own spelling, so a residual renders `n` and never an
        /// internal identity.
        name: String,
    },
    /// [INV-1, MSR-1] one measure former written as an affine factor. The
    /// inner expression is exactly the measure read the [OP-1] reader row
    /// checks, so the affine domain reaches the same [ENT-2] term the
    /// automatic derivation already carries an atom for; the invariant
    /// evaluates nothing and reads no storage [INV-1].
    Measure(Box<CheckedExpression>),
    Add(Box<CheckedAffineExpression>, Box<CheckedAffineExpression>),
    Subtract(Box<CheckedAffineExpression>, Box<CheckedAffineExpression>),
    MultiplyByConstant {
        constant: i128,
        constant_ty: IntegerType,
        value: Box<CheckedAffineExpression>,
    },
}

/// One normalized source-written affine ordered relation. `left - right <=
/// bound` has `bound == 0` for non-strict order and `bound == -1` for strict
/// integer order. The checker has already admitted the expression vocabulary,
/// but this record alone grants no fact: INV-1 or PRF-1 must still prove its
/// owning judgment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedAffineRelation {
    pub(crate) node_path: NodePath,
    pub(crate) left: CheckedAffineExpression,
    pub(crate) right: CheckedAffineExpression,
    pub(crate) bound: i128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedLoopInvariant {
    pub(crate) loop_id: CheckedLoopId,
    pub(crate) declaration: DeclarationId,
    pub(crate) name: String,
    pub(crate) relation: CheckedAffineRelation,
}

/// One source-written `use` in a local invariant certificate.
///
/// `multiplicity` is how many times the premise is added into the certificate
/// sum. The omitted source spelling is represented as the literal one. This
/// record deliberately contains no accumulating state: every use is checked
/// against the invariant statement's same entering proof context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedProofUse {
    pub(crate) node_path: NodePath,
    pub(crate) multiplicity: CheckedProofMultiplicity,
    pub(crate) source: CheckedProofUseSource,
}

/// How many times one written `use` adds its premise into the certificate sum.
///
/// A bare decimal is a proof-domain integer known where the certificate is
/// checked. A named value is read at the entering program point instead;
/// [PRF-1] admits only an unsigned integer there, so the nonnegativity the
/// scaling step needs is a property of the written type rather than an
/// obligation the certificate must separately discharge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedProofMultiplicity {
    Literal(i128),
    Value { binding: BindingId, ty: IntegerType },
}

/// The source selected by one written `use`.
///
/// A named source is the immutable theorem image published by the resolved
/// invariant declaration. A relation source is independently proved by AUTO
/// in the local invariant's entering context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedProofUseSource {
    Named(DeclarationId),
    Relation(CheckedAffineRelation),
}

/// One erased source-written local invariant. Every `use` and the target are
/// written in the `.wf` source; later analysis proves each use independently,
/// follows the written multipliers, and publishes only the checked target.
///
/// The historical type name remains internal while the parser surface moves
/// from `prove` to `invariant`; it does not grant a separate proof language or
/// runtime operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedSourceProof {
    pub(crate) node_path: NodePath,
    pub(crate) declaration: DeclarationId,
    pub(crate) name: String,
    pub(crate) target: CheckedAffineRelation,
    pub(crate) uses: Vec<CheckedProofUse>,
}

/// The checked source production that owns a value initializer. These forms
/// share GIVE-1 typing and lowering, but only `value_if` is an ENT-5 relation
/// carrier.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ValueInitializerKind {
    ValueIf,
    ValueMatch,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct NominalId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CheckedConstantId(pub(crate) u32);

/// One checker-interned symbolic const operation [`DerivedConst`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct DerivedConstId(pub(crate) u32);

/// The five bare const-expression operations of the CONST-1 candidate
/// grammar. Const evaluation happens at monomorphization in the unsigned
/// 64-bit domain under the const-eval overflow policy: a result outside that
/// domain or a zero divisor is a compile-time rejection citing CONST-1, never
/// a runtime trap, so this family is disjoint from the runtime arithmetic
/// modes and excluded from EFF-2's state/allocation effect relation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ConstOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

impl ConstOperation {
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Remainder => "%",
        }
    }
}

/// One interned symbolic const-expression node: exactly one operation over
/// two operands, mirroring the one-operation source grammar. At least one
/// operand is symbolic — a fully concrete operation is evaluated eagerly and
/// never interned — so a value of this shape exists only while a generic
/// template or symbolic validation instance is being checked, and every
/// concrete instantiation evaluates it away.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct DerivedConst {
    pub(crate) operation: ConstOperation,
    pub(crate) left: CheckedConst,
    pub(crate) right: CheckedConst,
}

/// Evaluates one const operation in the u64 const-eval domain.
///
/// `None` is the const-eval overflow policy's rejection premise: the
/// mathematical result is outside the domain, or the divisor is zero.
pub(crate) const fn evaluate_const_operation(
    operation: ConstOperation,
    left: u64,
    right: u64,
) -> Option<u64> {
    match operation {
        ConstOperation::Add => left.checked_add(right),
        ConstOperation::Subtract => left.checked_sub(right),
        ConstOperation::Multiply => left.checked_mul(right),
        ConstOperation::Divide => left.checked_div(right),
        ConstOperation::Remainder => left.checked_rem(right),
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedConst {
    Value(u64),
    Parameter(DeclarationId),
    /// One symbolic const operation, by checker-interned identity. Structural
    /// identity is id identity because interning is hash-consed.
    Derived(DerivedConstId),
}

impl CheckedConst {
    pub(crate) const fn value(self) -> Option<u64> {
        match self {
            Self::Value(value) => Some(value),
            Self::Parameter(_) | Self::Derived(_) => None,
        }
    }

    pub(crate) const fn is_concrete(self) -> bool {
        matches!(self, Self::Value(_))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum IntegerType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

impl IntegerType {
    pub(crate) const fn width(self) -> u8 {
        match self {
            Self::I8 | Self::U8 => 8,
            Self::I16 | Self::U16 => 16,
            Self::I32 | Self::U32 => 32,
            Self::I64 | Self::U64 => 64,
        }
    }

    pub(crate) const fn signed(self) -> bool {
        matches!(self, Self::I8 | Self::I16 | Self::I32 | Self::I64)
    }

    pub(crate) const fn converts_totally_to(self, destination: Self) -> bool {
        self.width() < destination.width()
            && (self.signed() == destination.signed() || (!self.signed() && destination.signed()))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum FloatType {
    F32,
    F64,
}

impl FloatType {
    pub(crate) const fn width(self) -> u8 {
        match self {
            Self::F32 => 32,
            Self::F64 => 64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedNumericType {
    Integer(IntegerType),
    Float(FloatType),
}

impl CheckedNumericType {
    pub(crate) const fn ty(self) -> CheckedType {
        match self {
            Self::Integer(ty) => CheckedType::Integer(ty),
            Self::Float(ty) => CheckedType::Float(ty),
        }
    }

    pub(crate) const fn converts_totally_to(self, destination: Self) -> bool {
        match (self, destination) {
            (Self::Integer(source), Self::Integer(destination)) => {
                source.converts_totally_to(destination)
            }
            (Self::Integer(source), Self::Float(FloatType::F32)) => source.width() <= 16,
            (Self::Integer(source), Self::Float(FloatType::F64)) => source.width() <= 32,
            (Self::Float(FloatType::F32), Self::Float(FloatType::F64)) => true,
            _ => false,
        }
    }

    pub(crate) const fn reinterprets_to(self, destination: Self) -> bool {
        match (self, destination) {
            (Self::Integer(source), Self::Integer(destination)) => {
                source.width() == destination.width() && source.signed() != destination.signed()
            }
            (Self::Integer(source), Self::Float(destination))
            | (Self::Float(destination), Self::Integer(source)) => {
                source.width() == destination.width()
            }
            (Self::Float(_), Self::Float(_)) => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedFlatElement {
    Unit,
    Bool,
    Integer(IntegerType),
    Float(FloatType),
    GenericInt(DeclarationId),
    GenericFloat(DeclarationId),
    TagOnlyNominal(NominalId),
    /// One affine aggregate element type: a region-free non-copy nominal
    /// stored by value. [TYPE-2] admits this element domain for `buffer`
    /// formation only; slices keep the flat copy domain, so
    /// their element constructors never produce this variant.
    Nominal(NominalId),
    /// One unbounded type parameter in a run's element position [BLK-1].
    ///
    /// [FN-2] makes generics monomorphization-only, so this variant belongs
    /// to the symbolic pass alone: every concrete instance re-parses the
    /// element position with its own substitution and produces a concrete
    /// element. Only a run's element position forms it — a `buffer`, an
    /// `array` and a `slice` keep the element domains [TYPE-2] gives them —
    /// and it reaches no layout, no lowering, and no backend.
    Generic(DeclarationId),
}

impl CheckedFlatElement {
    pub(crate) const fn ty(self) -> CheckedType {
        match self {
            Self::Unit => CheckedType::Unit,
            Self::Bool => CheckedType::Bool,
            Self::Integer(ty) => CheckedType::Integer(ty),
            Self::Float(ty) => CheckedType::Float(ty),
            Self::GenericInt(declaration) => CheckedType::GenericInt(declaration),
            Self::GenericFloat(declaration) => CheckedType::GenericFloat(declaration),
            Self::TagOnlyNominal(id) | Self::Nominal(id) => CheckedType::Nominal(id),
            Self::Generic(declaration) => CheckedType::Generic(declaration),
        }
    }
}

/// [PROV-6, STOR-1, STOR-3] which release action a store-backed run's own
/// reclamation is, decided from its store region's declaration alone.
///
/// A general store's run is released by spending that store's provider
/// capability; a bump extent's is reclaimed by its region's own reset and has
/// no action of its own [BLK-2]. Nothing else decides it: the class is read
/// off the region declaration and travels in the type, which is what lets a
/// region-erased lowering still select the right action.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedReleaseClass {
    /// An unbounded region parameter or a `linear`-bounded one: the release is a free to that store, and an unbounded parameter is
    /// this class fail-closed [PROV-6].
    General,
    /// An `affine`-bounded region parameter or a `region_stmt` region: the
    /// extent's reclamation is its own region reset, so the run's release
    /// action is empty [BLK-2, STOR-3].
    Extent,
}

/// [TYPE-2, BLK-1] the complete type of one array or run element, interned in
/// the checked program.
/// Structural children precede parents; recursive ownership graphs pass through
/// nominal identities. The handle keeps every checked type compact and Copy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CheckedElement(pub(crate) u32);

impl CheckedElement {
    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

/// The strength of the loan one view value holds on its origin ranges
/// [VIEW-1, PROV-3].
///
/// [PROV-3] use 1 judges every access through a view at this strength: a
/// shared-strength view is one shared access to the range of every resolved
/// origin, an exclusive-strength one is one exclusive access to the same.
/// [SET-1] admits a target path through a view exactly at `Exclusive`, and
/// [OWN-1] classifies `Shared` copy and `Exclusive` affine [S27].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum LoanStrength {
    /// `Slice<'r, T>`: reads only, copy, and [OWN-5] admits any number.
    Shared,
    /// `MutSlice<'r, T>`: element writes, affine, and [OWN-5] refuses a
    /// second one on one range.
    Exclusive,
}

impl LoanStrength {
    /// The nominal spelling of the view holding a loan of this strength
    /// [S35], for a diagnostic that renders the type.
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::Shared => "Slice",
            Self::Exclusive => "MutSlice",
        }
    }

    /// The formation row that hands back a view of this strength [VIEW-2,
    /// S38], for a diagnostic that names the operation the writer wrote.
    pub(crate) const fn former(self) -> &'static str {
        match self {
            Self::Shared => "slice_of",
            Self::Exclusive => "mut_slice_of",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedType {
    Unit,
    Bool,
    Integer(IntegerType),
    Float(FloatType),
    Generic(DeclarationId),
    GenericInt(DeclarationId),
    GenericFloat(DeclarationId),
    Nominal(NominalId),
    Array {
        element: CheckedElement,
        length: CheckedConst,
    },
    /// One view [VIEW-1]: `Slice<'r, T>` at shared strength and
    /// `MutSlice<'r, T>` at exclusive strength.
    ///
    /// The strength is a component of the type's identity, so the two views
    /// are two types under the exact identity [TYPE-5] performs, and every
    /// rule that reads the view as a view — its loan-bearing predicate
    /// [PROV-3], its measure row [MSR-1], the storage positions it may not
    /// occupy [STOR-5] — reads one variant.
    Slice {
        region: DeclarationId,
        element: CheckedFlatElement,
        strength: LoanStrength,
    },
    Buffer {
        element: CheckedFlatElement,
    },
    /// One `FixedVector<T, n>` [BLK-1]: a frame-resident run of `n` slots
    /// whose initialized storage is the window `len` slots wide beginning at
    /// `head`. `n` is the type constant; `len` and `head` are descriptor
    /// words [OP-9].
    FixedVector {
        element: CheckedElement,
        length: CheckedConst,
    },
    /// One `Vector<'s, T>` [BLK-1]: a store-resident run taken from the store
    /// the region `'s` names [PROV-1]. The region is part of the type's
    /// identity, so two stores give two types.
    Vector {
        region: DeclarationId,
        element: CheckedElement,
        /// [PROV-6, STOR-3] which release action this run's own reclamation
        /// is. It is a function of `region` alone, so it never separates two
        /// types one region names, and it is carried because lowering erases
        /// the region and must still select between the two actions.
        release: CheckedReleaseClass,
    },
    /// One `Heap<'s>` [PROV-1]: the proof-only provider value of the general
    /// store `'s` names. Ordinary calls transport this provider through
    /// their declared store region.
    Heap {
        region: DeclarationId,
    },
    /// One `Arena<'s, bytes, align>` [PROV-1]: the proof-only provider value
    /// of the bump extent `'s` names, reserved by `arena_frame` [BLK-2].
    Extent {
        region: DeclarationId,
        bytes: CheckedConst,
        align: CheckedConst,
    },
}

impl CheckedType {
    pub(crate) fn is_concrete(self, elements: &[CheckedType]) -> bool {
        match self {
            Self::Generic(_) | Self::GenericInt(_) | Self::GenericFloat(_) => false,
            Self::Array { element, length } | Self::FixedVector { element, length } => {
                elements
                    .get(element.0 as usize)
                    .is_some_and(|ty| ty.is_concrete(elements))
                    && length.is_concrete()
            }
            Self::Vector { element, .. } => elements
                .get(element.0 as usize)
                .is_some_and(|ty| ty.is_concrete(elements)),
            Self::Slice { element, .. } | Self::Buffer { element } => {
                element.ty().is_concrete(elements)
            }
            Self::Extent { bytes, align, .. } => bytes.is_concrete() && align.is_concrete(),
            Self::Unit
            | Self::Bool
            | Self::Integer(_)
            | Self::Float(_)
            | Self::Nominal(_)
            | Self::Heap { .. } => true,
        }
    }
}

/// One of [MSR-1]'s four measures of a measured value.
///
/// The spelling and the [ENT-2] term are the same quantity read two ways, so
/// one enum keys both the [OP-1] reader row and the measure term.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum CheckedMeasure {
    Length,
    Capacity,
    Room,
    Head,
}

/// One cell of [MSR-1]'s measure table.
///
/// This version's table gives every cell of every measured type an exact
/// value, so `Bounded` has no row yet; the enum states the three cell classes
/// the rule requires so a later row cannot smuggle in a fourth.
// [MSR-1] requires every cell of the table to be one of exact, bounded or
// absent. No row of this version.s table selects bounded or absent, and the
// two classes stay named here because the rule is what fixes the closed set:
// a later row that needs one adds the row, not a fourth class.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MeasureCell {
    /// The measure is exactly the measured value's own extent, which the
    /// `len` reader already loads.
    ExactExtent,
    /// The measure is exactly this compile-time constant.
    ExactConstant(u64),
    /// The measure is exactly the type's own written constant: an `array`'s
    /// or a `FixedVector`'s capacity, or an `Arena`'s byte extent.
    ExactTypeConstant,
    /// The measure is exact and is an independent runtime quantity of the
    /// value's own descriptor: a run's `len`, a `Vector`'s `cap`, and the
    /// `room` [MSR-2]'s identity relates to the other two [BLK-1].
    ExactRuntime,
    /// The measure is exact but only two-sidedly published by some writing
    /// operation. A run's `head` is the one cell of this class [BLK-3].
    Bounded,
    /// The type has no such measure.
    Absent,
}

impl MeasureCell {
    /// The [MSR-1] classification word this cell writes in the specification's
    /// own table: *exact*, *bounded*, or *absent*, and nothing else.
    #[cfg(test)]
    pub(crate) const fn classification(self) -> &'static str {
        match self {
            Self::ExactExtent
            | Self::ExactConstant(_)
            | Self::ExactTypeConstant
            | Self::ExactRuntime => "exact",
            Self::Bounded => "bounded",
            Self::Absent => "absent",
        }
    }
}

impl CheckedMeasure {
    /// The [OP-1] spelling of this measure, which is also its [MSR-5] former.
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::Length => "len_of",
            Self::Capacity => "cap_of",
            Self::Room => "room_of",
            Self::Head => "head_of",
        }
    }

    /// [MSR-1]'s measure table, for the three measured types this version
    /// has. Each is completely initialized over its whole capacity at its
    /// formation, so it has no spare room and no window origin but zero.
    ///
    /// The table is data, not a rule: a later version adds a row per measured
    /// type it adds, and only such a row can introduce a bounded or absent
    /// cell.
    pub(crate) const fn cell(self, measured: MeasuredKind) -> MeasureCell {
        match (measured, self) {
            (MeasuredKind::Array | MeasuredKind::Buffer | MeasuredKind::Slice, Self::Length)
            | (MeasuredKind::Array | MeasuredKind::Buffer | MeasuredKind::Slice, Self::Capacity) => {
                MeasureCell::ExactExtent
            }
            (
                MeasuredKind::Array | MeasuredKind::Buffer | MeasuredKind::Slice,
                Self::Room | Self::Head,
            ) => MeasureCell::ExactConstant(0),
            // The two runs and the bump extent [BLK-1, PROV-1]. A run's
            // window is `len` slots beginning at `head` modulo `cap`, so
            // `len` and `head` are descriptor words, `room` is the
            // complement [MSR-2] already relates, and only a `Vector`'s
            // capacity is a runtime quantity rather than a type constant.
            (MeasuredKind::FixedVector | MeasuredKind::Vector, Self::Length)
            | (MeasuredKind::FixedVector | MeasuredKind::Vector, Self::Room)
            | (MeasuredKind::Vector, Self::Capacity)
            | (MeasuredKind::Extent, Self::Length | Self::Room) => MeasureCell::ExactRuntime,
            (MeasuredKind::FixedVector | MeasuredKind::Extent, Self::Capacity) => {
                MeasureCell::ExactTypeConstant
            }
            (MeasuredKind::FixedVector | MeasuredKind::Vector, Self::Head) => MeasureCell::Bounded,
            // A bump extent has no window; a general store has no row at
            // all, and both are absences of table data rather than an
            // exception clause (L6).
            (MeasuredKind::Extent, Self::Head) => MeasureCell::Absent,
        }
    }
}

/// One measured type [MSR-1]: exactly a type the measure table gives a row.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum MeasuredKind {
    Array,
    Buffer,
    Slice,
    FixedVector,
    Vector,
    Extent,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedValue {
    Unit,
    Bool(bool),
    Integer {
        ty: IntegerType,
        bits: u64,
    },
    Float {
        ty: FloatType,
        bits: u64,
    },
    /// One in-scope const generic read as a value [MSR-6]. It is a
    /// monomorphization-time constant, so a concrete [FN-2] instance folds
    /// it to `Integer` at substitution and only the one source-canonical
    /// symbolic instance retains this form, where [ENT-2] clause (c)
    /// already makes it the symbolic constant term.
    ConstGeneric {
        declaration: DeclarationId,
        ty: IntegerType,
    },
    NumericIdentity {
        ty: CheckedType,
        one: bool,
    },
    Array {
        ty: CheckedType,
        elements: Vec<CheckedValue>,
    },
    /// One struct-typed constant value [CONST-2 candidate]: the nominal
    /// instance plus its complete field values in declared order.
    Struct {
        ty: CheckedType,
        fields: Vec<CheckedValue>,
    },
}

impl CheckedValue {
    pub(crate) const fn ty(&self) -> CheckedType {
        match self {
            Self::Unit => CheckedType::Unit,
            Self::Bool(_) => CheckedType::Bool,
            Self::Integer { ty, .. } => CheckedType::Integer(*ty),
            Self::Float { ty, .. } => CheckedType::Float(*ty),
            Self::ConstGeneric { ty, .. } => CheckedType::Integer(*ty),
            Self::NumericIdentity { ty, .. } => *ty,
            Self::Array { ty, .. } => *ty,
            Self::Struct { ty, .. } => *ty,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedConstant {
    pub(crate) id: CheckedConstantId,
    /// Resolved declaration identity retained for named-const goal leaves.
    pub(crate) declaration: DeclarationId,
    pub(crate) name: String,
    /// Source value type, before a fixed-run constant's descriptor-free
    /// dense storage normalization. Borrowing cannot change this identity.
    pub(crate) declared_type: CheckedType,
    pub(crate) ty: CheckedType,
    pub(crate) value: CheckedValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedField {
    pub(crate) name: String,
    pub(crate) ty: CheckedType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedVariant {
    pub(crate) name: String,
    pub(crate) constructor: CheckedConstructor,
    pub(crate) tag: u32,
    pub(crate) fields: Vec<CheckedField>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedConstructor {
    Source(DeclarationId),
    Prelude(BuiltinPreludeId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedNominalKind {
    Struct {
        fields: Vec<CheckedField>,
    },
    Enum {
        variants: Vec<CheckedVariant>,
    },
    /// One boxed cell. `region` is `None` for the ambient-heap `box<T>`
    /// [STOR-2] and `Some(store)` for the store-branded `Box<'s, T>` S39,
    /// whose region is a component of its type exactly as a run's is
    /// [PROV-1] and whose release class that region decides [PROV-6].
    Box {
        referent: CheckedType,
        region: Option<DeclarationId>,
        /// [PROV-6] which release action this cell's own reclamation is, read
        /// off `region` at the moment the nominal is interned. The ambient
        /// heap's `box<T>` and a general store's cell both free; a bump
        /// extent's cell is reclaimed by its region's own reset.
        release: CheckedReleaseClass,
    },
    /// One `arena<'r, T>` instance [STOR-1, STOR-2]. The region is part of
    /// the type's identity, so `arena<'r, T>` and `arena<'s, T>` are two
    /// nominals. Its storage is released with its region rather than with an
    /// owner scope [STOR-3, STOR-4], so the value itself derives no drop.
    Arena {
        region: DeclarationId,
        content: CheckedType,
    },
    /// The compiler-owned allocation list one region block carries when it
    /// has arena allocations: a pointer-shaped cell whose compiler-derived
    /// drop walks and frees every registered allocation, which is exactly
    /// the region's [STOR-3] storage release.
    ArenaStorage,
    /// An ordinary opaque nominal has no fields or constructor.
    Opaque,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedNominal {
    pub(crate) id: NominalId,
    pub(crate) name: String,
    pub(crate) kind: CheckedNominalKind,
    /// [PROV-6] whether this nominal's source declaration carries the
    /// `linear` modifier. Only a source `struct_decl` or `enum_decl` can, so
    /// every compiler-owned nominal is false.
    pub(crate) linear: bool,
}

impl CheckedNominal {
    pub(crate) fn is_copy(&self) -> bool {
        matches!(
            &self.kind,
            CheckedNominalKind::Enum { variants }
                if variants.iter().all(|variant| variant.fields.is_empty())
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedIntegerOperation {
    AddWrap,
    SubtractWrap,
    MultiplyWrap,
    AddExact,
    SubtractExact,
    MultiplyExact,
    AddDefined,
    SubtractDefined,
    MultiplyDefined,
    AddChecked,
    SubtractChecked,
    MultiplyChecked,
    DivideChecked,
    RemainderChecked,
    DivideExact,
    RemainderExact,
    DivideDefined,
    RemainderDefined,
    AbsoluteWrap,
    AbsoluteExact,
    AbsoluteDefined,
    AbsoluteChecked,
    NegateWrap,
    NegateExact,
    NegateDefined,
    NegateChecked,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    ShiftLeftWrap,
    ShiftRightWrap,
    ShiftLeftExact,
    ShiftRightExact,
    ShiftLeftDefined,
    ShiftRightDefined,
    RotateLeft,
    RotateRight,
    PopulationCount,
    LeadingZeros,
    TrailingZeros,
    ByteSwap,
    MultiplyHigh,
    AddSaturating,
    SubtractSaturating,
    MultiplySaturating,
    Minimum,
    Maximum,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedBooleanOperation {
    And,
    Or,
    ExclusiveOr,
    Not,
}

impl CheckedBooleanOperation {
    /// The [OP-1] spelling of each Bool row, exhaustive by construction.
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::And => "band",
            Self::Or => "bor",
            Self::ExclusiveOr => "bxor",
            Self::Not => "bnot",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedFloatOperation {
    AddStrict,
    SubtractStrict,
    MultiplyStrict,
    DivideStrict,
    Equal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    NotEqual,
    Negate,
    Absolute,
    CopySign,
    Minimum,
    Maximum,
    Floor,
    Ceil,
    Truncate,
    RoundEven,
    Remainder,
    SquareRootStrict,
    FusedMultiplyAddStrict,
    Infinity,
    Nan,
}

impl CheckedFloatOperation {
    /// The [OP-1] spelling of each float operation the compiler models,
    /// exhaustive by construction and locked against the specification table
    /// by `semantic::tests::operation_table`.
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::AddStrict => "fadd.strict",
            Self::SubtractStrict => "fsub.strict",
            Self::MultiplyStrict => "fmul.strict",
            Self::DivideStrict => "fdiv.strict",
            Self::Equal => "feq",
            Self::Less => "flt",
            Self::LessEqual => "fle",
            Self::Greater => "fgt",
            Self::GreaterEqual => "fge",
            Self::NotEqual => "fne",
            Self::Negate => "fneg",
            Self::Absolute => "fabs",
            Self::CopySign => "fcopysign",
            Self::Minimum => "fmin",
            Self::Maximum => "fmax",
            Self::Floor => "ffloor",
            Self::Ceil => "fceil",
            Self::Truncate => "ftrunc",
            Self::RoundEven => "froundeven",
            Self::Remainder => "frem",
            Self::SquareRootStrict => "fsqrt.strict",
            Self::FusedMultiplyAddStrict => "ffma.strict",
            Self::Infinity => "finf",
            Self::Nan => "fnan",
        }
    }

    pub(crate) const fn operand_count(self) -> usize {
        match self {
            Self::Infinity | Self::Nan => 0,
            Self::Negate
            | Self::Absolute
            | Self::Floor
            | Self::Ceil
            | Self::Truncate
            | Self::RoundEven
            | Self::SquareRootStrict => 1,
            Self::FusedMultiplyAddStrict => 3,
            _ => 2,
        }
    }

    pub(crate) const fn result_type(self, operand: CheckedType) -> CheckedType {
        match self {
            Self::Equal
            | Self::Less
            | Self::LessEqual
            | Self::Greater
            | Self::GreaterEqual
            | Self::NotEqual => CheckedType::Bool,
            _ => operand,
        }
    }
}

/// The [PRE-1] error type a checked [OP-1] integer row reports.
///
/// The row's `signature` cell names it, so the choice is table data rather
/// than a property of the operation's semantics; this enum exists so that one
/// place decides it and an extraction lock can compare that decision against
/// the specification's own cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedIntegerErrorClass {
    Overflow,
    DivError,
}

impl CheckedIntegerErrorClass {
    /// The exact PRE-1 spelling the `wf-ops` `signature` cell writes.
    ///
    /// Read only by the extraction lock: nothing in the compiler's own path
    /// needs the name, because the class already selects the prelude type.
    #[cfg(test)]
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::Overflow => "Overflow",
            Self::DivError => "DivError",
        }
    }
}

impl CheckedIntegerOperation {
    /// The [OP-1] spelling of each integer operation the compiler models.
    ///
    /// Exhaustive by construction: a new variant is a compile error here,
    /// which is the point — the row it belongs to must be named before it can
    /// be checked. `semantic::tests::operation_table` locks this map against
    /// the specification's own `wf-ops` table in both directions, so every
    /// diagnostic that renders an operation in source terms renders the
    /// spelling the specification fixes.
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::AddWrap => "+wrap",
            Self::SubtractWrap => "-wrap",
            Self::MultiplyWrap => "*wrap",
            Self::AddExact => "+",
            Self::SubtractExact => "-",
            Self::MultiplyExact => "*",
            Self::AddDefined => "+defined",
            Self::SubtractDefined => "-defined",
            Self::MultiplyDefined => "*defined",
            Self::AddChecked => "+checked",
            Self::SubtractChecked => "-checked",
            Self::MultiplyChecked => "*checked",
            Self::DivideExact => "/",
            Self::RemainderExact => "%",
            Self::DivideDefined => "/defined",
            Self::RemainderDefined => "%defined",
            Self::DivideChecked => "/checked",
            Self::RemainderChecked => "%checked",
            Self::AbsoluteWrap => "iabs.wrap",
            Self::AbsoluteExact => "iabs",
            Self::AbsoluteDefined => "iabs.defined",
            Self::AbsoluteChecked => "iabs.checked",
            Self::NegateWrap => "ineg.wrap",
            Self::NegateExact => "ineg",
            Self::NegateDefined => "ineg.defined",
            Self::NegateChecked => "ineg.checked",
            Self::BitAnd => "iand",
            Self::BitOr => "ior",
            Self::BitXor => "ixor",
            Self::BitNot => "inot",
            Self::ShiftLeftWrap => "ishl.wrap",
            Self::ShiftRightWrap => "ishr.wrap",
            Self::ShiftLeftExact => "ishl",
            Self::ShiftRightExact => "ishr",
            Self::ShiftLeftDefined => "ishl.defined",
            Self::ShiftRightDefined => "ishr.defined",
            Self::RotateLeft => "irotl",
            Self::RotateRight => "irotr",
            Self::PopulationCount => "ipopcount",
            Self::LeadingZeros => "iclz",
            Self::TrailingZeros => "ictz",
            Self::ByteSwap => "ibswap",
            Self::MultiplyHigh => "imulhi",
            Self::AddSaturating => "+sat",
            Self::SubtractSaturating => "-sat",
            Self::MultiplySaturating => "*sat",
            Self::Minimum => "imin",
            Self::Maximum => "imax",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
        }
    }

    pub(crate) const fn is_exact(self) -> bool {
        matches!(
            self,
            Self::AddExact
                | Self::SubtractExact
                | Self::MultiplyExact
                | Self::DivideExact
                | Self::RemainderExact
                | Self::AbsoluteExact
                | Self::NegateExact
                | Self::ShiftLeftExact
                | Self::ShiftRightExact
        )
    }

    pub(crate) const fn defined_query(self) -> Option<Self> {
        Some(match self {
            Self::AddExact => Self::AddDefined,
            Self::SubtractExact => Self::SubtractDefined,
            Self::MultiplyExact => Self::MultiplyDefined,
            Self::DivideExact => Self::DivideDefined,
            Self::RemainderExact => Self::RemainderDefined,
            Self::AbsoluteExact => Self::AbsoluteDefined,
            Self::NegateExact => Self::NegateDefined,
            Self::ShiftLeftExact => Self::ShiftLeftDefined,
            Self::ShiftRightExact => Self::ShiftRightDefined,
            _ => return None,
        })
    }

    pub(crate) const fn is_defined_query(self) -> bool {
        matches!(
            self,
            Self::AddDefined
                | Self::SubtractDefined
                | Self::MultiplyDefined
                | Self::DivideDefined
                | Self::RemainderDefined
                | Self::AbsoluteDefined
                | Self::NegateDefined
                | Self::ShiftLeftDefined
                | Self::ShiftRightDefined
        )
    }

    pub(crate) const fn operand_count(self) -> usize {
        match self {
            Self::AbsoluteWrap
            | Self::AbsoluteExact
            | Self::AbsoluteDefined
            | Self::AbsoluteChecked
            | Self::NegateWrap
            | Self::NegateExact
            | Self::NegateDefined
            | Self::NegateChecked
            | Self::BitNot
            | Self::PopulationCount
            | Self::LeadingZeros
            | Self::TrailingZeros
            | Self::ByteSwap => 1,
            _ => 2,
        }
    }

    pub(crate) const fn accepts_operand_type(self, operand: CheckedType) -> bool {
        match (self, operand) {
            (
                Self::AbsoluteWrap
                | Self::AbsoluteExact
                | Self::AbsoluteDefined
                | Self::AbsoluteChecked
                | Self::NegateWrap
                | Self::NegateExact
                | Self::NegateDefined
                | Self::NegateChecked,
                CheckedType::Integer(operand),
            ) => operand.signed(),
            (Self::ByteSwap, CheckedType::Integer(operand)) => operand.width() >= 16,
            (
                Self::AbsoluteWrap
                | Self::AbsoluteExact
                | Self::AbsoluteDefined
                | Self::AbsoluteChecked
                | Self::NegateWrap
                | Self::NegateExact
                | Self::NegateDefined
                | Self::NegateChecked
                | Self::ByteSwap,
                CheckedType::GenericInt(_),
            ) => false,
            (_, CheckedType::Integer(_) | CheckedType::GenericInt(_)) => true,
            _ => false,
        }
    }

    pub(crate) const fn argument_type(
        self,
        operand: CheckedType,
        index: usize,
    ) -> Option<CheckedType> {
        if index >= self.operand_count() {
            return None;
        }
        if index == 1
            && matches!(
                self,
                Self::ShiftLeftWrap
                    | Self::ShiftRightWrap
                    | Self::ShiftLeftExact
                    | Self::ShiftRightExact
                    | Self::ShiftLeftDefined
                    | Self::ShiftRightDefined
                    | Self::RotateLeft
                    | Self::RotateRight
            )
        {
            Some(CheckedType::Integer(IntegerType::U32))
        } else {
            Some(operand)
        }
    }

    /// The error type of a checked row, or `None` for a row whose result is a
    /// scalar. Exactly the complement of [`Self::scalar_result_type`]'s `None`:
    /// a row either produces a scalar or produces `Result<T, E>`.
    pub(crate) const fn checked_error(self) -> Option<CheckedIntegerErrorClass> {
        match self {
            Self::AddChecked
            | Self::SubtractChecked
            | Self::MultiplyChecked
            | Self::AbsoluteChecked
            | Self::NegateChecked => Some(CheckedIntegerErrorClass::Overflow),
            Self::DivideChecked | Self::RemainderChecked => {
                Some(CheckedIntegerErrorClass::DivError)
            }
            _ => None,
        }
    }

    pub(crate) const fn scalar_result_type(self, operand: CheckedType) -> Option<CheckedType> {
        match self {
            Self::AddChecked
            | Self::SubtractChecked
            | Self::MultiplyChecked
            | Self::DivideChecked
            | Self::RemainderChecked
            | Self::AbsoluteChecked
            | Self::NegateChecked => None,
            Self::PopulationCount | Self::LeadingZeros | Self::TrailingZeros => {
                Some(CheckedType::Integer(IntegerType::U32))
            }
            Self::Equal
            | Self::NotEqual
            | Self::Less
            | Self::LessEqual
            | Self::Greater
            | Self::GreaterEqual
            | Self::AddDefined
            | Self::SubtractDefined
            | Self::MultiplyDefined
            | Self::DivideDefined
            | Self::RemainderDefined
            | Self::AbsoluteDefined
            | Self::NegateDefined
            | Self::ShiftLeftDefined
            | Self::ShiftRightDefined => Some(CheckedType::Bool),
            _ => Some(operand),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedTargetDomainObligation {
    RuntimeSizedAllocation,
    ElementAddress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CheckedRuntimeTargetObligations {
    allocation: CheckedTargetDomainObligation,
    element_address: CheckedTargetDomainObligation,
    /// Tightest target-independent length ceiling retained at this source
    /// allocation site. Entailment installs it after proving OP-9; lowering
    /// must not proceed while it is absent.
    source_length_upper_bound: Option<u64>,
}

/// Target-independent upper bounds for one stored value's representation.
/// The backend must qualify its concrete layout against all three cells
/// before it may use the source-level `buffer_fits<T>` proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedLayoutMagnitude {
    Finite(u64),
    AboveU64,
}

impl CheckedLayoutMagnitude {
    /// The exact largest element count admitted by OP-9 for this stride.
    /// Every stride represented by `AboveU64` is greater than U64_MAX, so
    /// only the zero-length allocation can fit its u64 byte-count domain.
    pub(crate) const fn allocation_limit(self) -> u64 {
        match self {
            Self::Finite(stride) => {
                assert!(stride >= 1, "a layout stride ceiling is always positive");
                u64::MAX / stride
            }
            Self::AboveU64 => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CheckedLayoutCeiling {
    pub(crate) size: CheckedLayoutMagnitude,
    pub(crate) align: u64,
    pub(crate) stride: CheckedLayoutMagnitude,
}

impl CheckedRuntimeTargetObligations {
    pub(crate) const fn new() -> Self {
        Self {
            allocation: CheckedTargetDomainObligation::RuntimeSizedAllocation,
            element_address: CheckedTargetDomainObligation::ElementAddress,
            source_length_upper_bound: None,
        }
    }

    pub(crate) const fn allocation(self) -> CheckedTargetDomainObligation {
        self.allocation
    }

    pub(crate) const fn element_address(self) -> CheckedTargetDomainObligation {
        self.element_address
    }

    pub(crate) const fn source_length_upper_bound(self) -> Option<u64> {
        self.source_length_upper_bound
    }

    /// Installs the conclusion of the source allocation proof on the checked
    /// allocation node. This copies an already-derived fact; it performs no
    /// second proof or replay.
    pub(crate) fn install_source_length_upper_bound(&mut self, upper_bound: u64) {
        self.source_length_upper_bound = Some(upper_bound);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedArrayRoot {
    Binding {
        binding: BindingId,
        fields: Vec<u32>,
    },
    Constant(CheckedConstantId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedBufferRoot {
    pub(crate) binding: BindingId,
    pub(crate) fields: Vec<u32>,
    pub(crate) element: CheckedFlatElement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedSliceRoot {
    pub(crate) binding: BindingId,
    pub(crate) element: CheckedFlatElement,
    /// [VIEW-1] the strength of the loan the viewed value holds, which is
    /// what [SET-1] reads to admit or refuse a target path through it.
    pub(crate) strength: LoanStrength,
}

/// A typed storage place used by a borrow or a compiler-owned measure.
///
/// [MSR-2] makes a measure's support the resolved place of the measured value
/// itself, so the root retains every field and subscript that reaches it.
/// The type selects the addressed referent and, when measured, the table's
/// row. A `FixedVector` also carries its capacity constant here because that
/// constant is stored nowhere at run time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedContainerRoot {
    pub(crate) root: super::places::PlaceRoot,
    /// The path below the root: field selections and subscripts, in written
    /// order [MSR-1]. `len_of(table[i])` is a term, so a measured place is
    /// not a field path.
    pub(crate) path: Vec<CheckedPlaceStep>,
    /// The type selected by the complete path.
    pub(crate) ty: CheckedType,
}

/// One step below a storage place's root [MSR-1]: a field selection, or one
/// [OP-4] subscript together with the obligation that subscript owes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedPlaceStep {
    Field(u32),
    /// One explicit dereference of an owning Box. Borrow-holder dereferences
    /// begin from an already-addressed binding and therefore need no step in
    /// this lowering path.
    BoxReferent(NominalId),
    Subscript(Box<CheckedPlaceSubscript>),
}

/// One subscript occurring inside a storage place [MSR-1, OP-4].
///
/// The offset is a logical one and its obligation is against `len_of` of the
/// base it indexes; the storage it selects is slot `(head_of + i) mod cap_of`,
/// which the lowering computes and no source rule mentions [BLK-1].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedPlaceSubscript {
    /// The base this subscript indexes. Its measure-table row is what the
    /// [OP-4] obligation `i < len_of(base)` is stated against [MSR-1].
    pub(crate) base_type: CheckedType,
    /// The [BLK-1] element type the subscript selects.
    pub(crate) element_type: CheckedType,
    pub(crate) offset: CheckedExpression,
    pub(crate) obligation: crate::NodePath,
    pub(crate) target_domain: CheckedTargetDomainObligation,
    /// [OWN-7, LIV-2, ENT-5] the offset as the place relations read it: two
    /// literals decide distinctness, a binding contributes its own support,
    /// and everything else is opaque.
    pub(crate) place_offset: super::places::PlaceOffset,
}

impl CheckedContainerRoot {
    pub(crate) const fn binding(&self) -> Option<BindingId> {
        match self.root {
            super::places::PlaceRoot::Binding(binding) => Some(binding),
            super::places::PlaceRoot::Constant(_) => None,
        }
    }

    /// The exact storage projection consumed by [OWN-7] and [ENT-5].
    pub(crate) fn place_path(&self) -> Vec<super::places::PlaceStep> {
        self.path
            .iter()
            .filter_map(CheckedPlaceStep::place_step)
            .collect()
    }

    /// Offset evaluations are children of the place, including when its
    /// terminal operation only takes an address or reads a measure.
    pub(crate) fn offsets(&self) -> impl Iterator<Item = &CheckedExpression> {
        self.path.iter().filter_map(|step| match step {
            CheckedPlaceStep::Field(_) | CheckedPlaceStep::BoxReferent(_) => None,
            CheckedPlaceStep::Subscript(index) => Some(&index.offset),
        })
    }

    pub(crate) fn offsets_mut(&mut self) -> impl Iterator<Item = &mut CheckedExpression> {
        self.path.iter_mut().filter_map(|step| match step {
            CheckedPlaceStep::Field(_) | CheckedPlaceStep::BoxReferent(_) => None,
            CheckedPlaceStep::Subscript(index) => Some(&mut index.offset),
        })
    }

    /// The same path as [ENT-2] goal projections.
    pub(crate) fn goal_projections(&self) -> Vec<super::goal::GoalProjection> {
        self.path
            .iter()
            .map(|step| match step {
                CheckedPlaceStep::Field(field) => super::goal::GoalProjection::Field(*field),
                CheckedPlaceStep::BoxReferent(_) => super::goal::GoalProjection::Deref,
                CheckedPlaceStep::Subscript(subscript) => {
                    super::goal::GoalProjection::Subscript(subscript.place_offset)
                }
            })
            .collect()
    }

    /// The measure-table row this place selects [MSR-1].
    pub(crate) const fn measured(&self) -> Option<MeasuredKind> {
        match self.ty {
            CheckedType::Array { .. } => Some(MeasuredKind::Array),
            CheckedType::FixedVector { .. } => Some(MeasuredKind::FixedVector),
            CheckedType::Vector { .. } => Some(MeasuredKind::Vector),
            CheckedType::Extent { .. } => Some(MeasuredKind::Extent),
            _ => None,
        }
    }

    /// The element type of a full array or run; a bump extent has none.
    pub(crate) const fn element(&self) -> Option<CheckedElement> {
        match self.ty {
            CheckedType::Array { element, .. }
            | CheckedType::FixedVector { element, .. }
            | CheckedType::Vector { element, .. } => Some(element),
            _ => None,
        }
    }

    /// The written constant a `FixedVector`'s capacity and an `Arena`'s byte
    /// extent are [MSR-2]; a `Vector`'s capacity is a descriptor word and has
    /// none.
    pub(crate) const fn type_constant(&self) -> Option<CheckedConst> {
        match self.ty {
            CheckedType::Array { length, .. } | CheckedType::FixedVector { length, .. } => {
                Some(length)
            }
            CheckedType::Extent { bytes, .. } => Some(bytes),
            _ => None,
        }
    }
}

impl CheckedPlaceStep {
    pub(crate) fn place_step(&self) -> Option<super::places::PlaceStep> {
        match self {
            Self::Field(field) => Some(super::places::PlaceStep::Field(*field)),
            // The allocated referent remains within the owning Box root for
            // overlap. Source-term identity retains this as Deref above.
            Self::BoxReferent(_) => None,
            Self::Subscript(index) => Some(super::places::PlaceStep::Subscript(index.place_offset)),
        }
    }
}

/// The resolved type, const and region arguments of one [BLK-0] kernel-domain
/// call.
///
/// A row's own parameters are owner-local and never enter source lookup
/// [BLK-0], so this is what one call fixes for them: the element type, the run
/// type the `vector` operand supplies [BLK-3], the const arguments the call
/// wrote or an operand supplied, and the store region.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedKernelInstance {
    /// The element type `T`.
    pub(crate) element: CheckedType,
    /// The run type `V` [BLK-3], where the row declares one.
    pub(crate) run: Option<CheckedType>,
    /// `n`, the `FixedVector` capacity.
    pub(crate) capacity: Option<CheckedConst>,
    /// `bytes` and `align`, the two `Arena` constants.
    pub(crate) bytes: Option<CheckedConst>,
    pub(crate) align: Option<CheckedConst>,
    /// The store region `'s` [PROV-1].
    pub(crate) region: Option<DeclarationId>,
    /// The layout ceiling of `T` [OP-9]: `size_ceiling(T)` and
    /// `align_ceiling(T)` are the two compile-time constants the bump rows
    /// name.
    pub(crate) element_ceiling: CheckedLayoutCeiling,
    /// The numeric upper bound [OP-9]'s accepted allocation-fit judgment
    /// retained for this call's `count`, which [STOR-6] multiplies by the
    /// actual target stride before the take is lowered.
    ///
    /// `None` on a row carrying no `fits::<T>(count)` obligation — the two
    /// cell rows, the reservation and the boundary rows — and on an
    /// acceptance-dark test hook whose obligation was deliberately left
    /// undischarged; neither reaches lowering with a count to scale.
    pub(crate) count_upper_bound: Option<u64>,
}

/// One member of the finite static origin set carried by a direct slice value.
///
/// The set is a conservative summary: one runtime slice descriptor points at
/// exactly one source, which must be represented by one member after complete
/// call substitution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedSliceOrigin {
    SourcePlace {
        root: DeclarationId,
        path: Vec<super::places::PlaceStep>,
        origin_region: Option<DeclarationId>,
    },
    ImmutableConst,
    FormalSlice {
        parameter: DeclarationId,
        region: DeclarationId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedSliceSource {
    Array {
        root: CheckedArrayRoot,
        length: CheckedConst,
    },
    Buffer(CheckedBufferRoot),
    /// An array reached in `arena<'r, T>` content through `deref` [OWN-5,
    /// OWN-10]. Semantic checking admits it; the arena runtime lowering is
    /// not implemented yet, and the temporary arena-parameter implementation
    /// stop keeps it from reaching lowering.
    ArenaContent {
        binding: BindingId,
        fields: Vec<u32>,
        length: CheckedConst,
    },
    /// The shared child reborrow of the view a **view holder** reaches
    /// [VIEW-2, OWN-6]: the parent's own descriptor, read once more under the
    /// narrower loan. There is no second storage and no second range, which
    /// is why the source carries only the holder binding.
    ViewHolder {
        binding: BindingId,
        element: CheckedFlatElement,
    },
    /// Typed owner storage: a run [BLK-1] or a complete array [TYPE-2].
    ///
    /// The window is `len_of` slots beginning at `head_of`, and the row's own
    /// requirement is what makes that one contiguous range: `head_of(vector)
    /// <= room_of(vector)` [VIEW-2], so the view is the slots from `head_of`
    /// onward and never wraps. A complete array has its type's length and a
    /// zero head, and the view addresses its original storage in both modes.
    Run(CheckedContainerRoot),
}

/// Source category retained only for integer-operation operands whose exact
/// written constant class affects an ENT-3 source. This is checked metadata,
/// not a second expression tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedIntegerArgumentSource {
    TypedLiteral,
    GenericNumericIdentity,
    NamedConstant { declaration: DeclarationId },
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedIntegerArgument {
    pub(crate) node_path: NodePath,
    pub(crate) source: CheckedIntegerArgumentSource,
}

/// The conservative caller storage claim of a bound borrow-mode result:
/// the complete single provenance-candidate actual [OWN-6, ENT-5]. The
/// delivered referent may be a narrower suffix or unrelated immutable
/// constant. This record supports exclusion and kills, never value identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedResultBorrow {
    /// The sole provenance-candidate argument, in declared parameter order.
    /// Lowering retains this relation to the actual argument value rather
    /// than reconstructing an address from the resolved source path.
    pub(crate) argument: usize,
    pub(crate) root: super::places::PlaceRoot,
    pub(crate) path: Vec<super::places::PlaceStep>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedExpression {
    Constant(CheckedValue),
    /// A named const read retains declaration identity for exact goal-origin
    /// equality while lowering the same immutable value as before.
    NamedConstant {
        declaration: DeclarationId,
        value: CheckedValue,
    },
    Binding {
        carrier: NodePath,
        binding: BindingId,
        ty: CheckedType,
        slice_origins: Vec<CheckedSliceOrigin>,
        /// The owning checker admitted this occurrence as an affine consume.
        /// ENT keeps it beside the checked binding because holder mode is not
        /// recoverable from the value type and must not be re-read from syntax.
        consume_root: bool,
    },
    UserCall {
        function: FunctionId,
        /// The authoritative function-formal row [FN-4, EFF-2], rebased to
        /// the selected concrete callee's parameter declarations. It is
        /// proof-only: lowering still calls `function` directly.
        formal_effects: Option<Box<CheckedEffects>>,
        /// Exact source call occurrence and declared-order argument atoms.
        call: NodePath,
        argument_nodes: Vec<NodePath>,
        arguments: Vec<CheckedExpression>,
        /// Pre-transfer caller images retained for exact GoalTemplate
        /// substitution after the complete concrete function inventory exists.
        goal_arguments: Vec<super::goal::GoalExpression>,
        /// Concrete caller regions supplied for the callee's formal region
        /// parameters, in declaration order.
        goal_regions: Vec<DeclarationId>,
        /// Filled from the complete phase-A inventory before entailment runs,
        /// in callee `requires_clause` source order.
        requirements: Vec<super::goal::CheckedCallRequirement>,
        result: CheckedType,
        slice_origins: Vec<CheckedSliceOrigin>,
        /// For a borrow-mode result admitted under the reborrow extension:
        /// the caller-side storage the result borrow is conservatively rooted
        /// at — the resolved place of the callee signature's single
        /// provenance-candidate actual. Entailment reads it so a write
        /// through the bound result kills exactly the facts on that storage
        /// [ENT-5]; it is `None` for every own-mode result and whenever the
        /// extension is off.
        result_borrow: Option<CheckedResultBorrow>,
    },
    /// One call to a [BLK-0] kernel-domain row.
    ///
    /// The row is a compiler-owned declaration record and not a source
    /// `fn_decl`, so its identity here is the `container_declaration_ordinal`
    /// [BLK-0] rather than a [`FunctionId`], and the diagnostics that arise
    /// in this domain name the row instead of citing a source node.
    KernelCall {
        /// The row's zero-based `container_declaration_ordinal` [BLK-0].
        operation: u8,
        row: crate::KernelRow,
        /// Exact source call occurrence and declared-order argument atoms.
        call: NodePath,
        /// This call's resolution of the row's own type, const and region
        /// parameters.
        instance: Box<CheckedKernelInstance>,
        argument_nodes: Vec<NodePath>,
        arguments: Vec<CheckedExpression>,
        /// Pre-transfer caller images, exactly as an ordinary call retains
        /// them, so [ENT-3.S13] can mint this call's call datums.
        goal_arguments: Vec<super::goal::GoalExpression>,
        /// The row's declared requirement list instantiated at this call, in
        /// declared order [BLK-0]. Each is submitted under [MSR-4].
        requirements: Vec<super::goal::ConcreteGoal>,
        /// The declared result type: one value, or the compiler-owned
        /// result-list nominal that carries an ordered result list [CALL-4].
        result: CheckedType,
    },
    IntegerOperation {
        carrier: NodePath,
        operation: CheckedIntegerOperation,
        operand_type: CheckedType,
        argument_metadata: Vec<CheckedIntegerArgument>,
        arguments: Vec<CheckedExpression>,
        result: CheckedType,
    },
    FloatOperation {
        carrier: NodePath,
        operation: CheckedFloatOperation,
        operand_type: CheckedType,
        arguments: Vec<CheckedExpression>,
    },
    NumericConversion {
        carrier: NodePath,
        source: CheckedNumericType,
        destination: CheckedNumericType,
        value: Box<CheckedExpression>,
        result: CheckedType,
    },
    Reinterpret {
        carrier: NodePath,
        source: CheckedNumericType,
        destination: CheckedNumericType,
        value: Box<CheckedExpression>,
    },
    BooleanOperation {
        carrier: NodePath,
        operation: CheckedBooleanOperation,
        arguments: Vec<CheckedExpression>,
    },
    EnumEquality {
        carrier: NodePath,
        equal: bool,
        operand_type: CheckedType,
        arguments: Vec<CheckedExpression>,
    },
    ArrayFill {
        carrier: NodePath,
        ty: CheckedType,
        value: Box<CheckedExpression>,
        target_domain: CheckedTargetDomainObligation,
    },
    ArrayMeasure {
        measure: CheckedMeasure,
        root: CheckedArrayRoot,
        length: CheckedConst,
    },
    ArrayIndex {
        carrier: NodePath,
        root: CheckedArrayRoot,
        element_type: CheckedType,
        length: CheckedConst,
        offset: Box<CheckedExpression>,
        obligation: NodePath,
        target_domain: CheckedTargetDomainObligation,
    },
    BufferFill {
        carrier: NodePath,
        element: CheckedFlatElement,
        length: Box<CheckedExpression>,
        value: Box<CheckedExpression>,
        layout_ceiling: CheckedLayoutCeiling,
        target_domains: CheckedRuntimeTargetObligations,
    },
    /// One `buffer_vacant::<T>(n)` allocation [OP-1, OP-9]: a flat buffer of
    /// the u64 length whose every element is the compiler-minted `None()`
    /// of the named `Option<T>` instance; no source value is duplicated.
    BufferVacant {
        carrier: NodePath,
        /// The interned `Option<T>` element instance.
        element: NominalId,
        length: Box<CheckedExpression>,
        layout_ceiling: CheckedLayoutCeiling,
        target_domains: CheckedRuntimeTargetObligations,
    },
    /// The canonical total OP-9 allocation-domain predicate. Its Boolean
    /// value is `n <= floor(u64::MAX / stride_ceiling(T))`; it never
    /// allocates and has no partial runtime outcome.
    BufferFits {
        carrier: NodePath,
        element: CheckedType,
        layout_ceiling: CheckedLayoutCeiling,
        length: Box<CheckedExpression>,
    },
    BufferMeasure {
        measure: CheckedMeasure,
        root: CheckedBufferRoot,
    },
    /// One [MSR-1] measure of one declared result place [CALL-4].
    ///
    /// A result binder is the clause's own datum and not a place, so a
    /// measure over it is read here rather than through the ordinary indexed
    /// place. It exists only inside an [FN-9] clause, is discarded with the
    /// clause's typing, and never reaches lowering.
    PostconditionResultMeasure {
        measure: CheckedMeasure,
        ordinal: u32,
        ty: CheckedType,
    },
    /// One [MSR-1] measure of a run [BLK-1] or a bump extent [PROV-1], read
    /// as its [OP-1] reader row. One quantity, one name, term and reader
    /// alike.
    ContainerMeasure {
        measure: CheckedMeasure,
        root: CheckedContainerRoot,
    },
    /// One discharged source subscript read of a run [OP-4, BLK-1].
    ///
    /// The offset is a logical one and its obligation is against `len`; the
    /// storage it selects is slot `(head + i) mod cap`, which the lowering
    /// computes and no source rule mentions.
    ReadStorage {
        carrier: NodePath,
        root: CheckedContainerRoot,
    },
    BufferIndex {
        carrier: NodePath,
        root: CheckedBufferRoot,
        offset: Box<CheckedExpression>,
        obligation: NodePath,
        target_domain: CheckedTargetDomainObligation,
    },
    SliceOf {
        carrier: NodePath,
        source: CheckedSliceSource,
        region: DeclarationId,
        element: CheckedFlatElement,
        /// [VIEW-2] which of the two formation rows this is: `slice_of`
        /// hands back a shared loan and `mut_slice_of` an exclusive one.
        strength: LoanStrength,
        origins: Vec<CheckedSliceOrigin>,
    },
    SliceMeasure {
        measure: CheckedMeasure,
        root: CheckedSliceRoot,
    },
    SliceIndex {
        carrier: NodePath,
        root: CheckedSliceRoot,
        offset: Box<CheckedExpression>,
        obligation: NodePath,
        target_domain: CheckedTargetDomainObligation,
    },
    BoxNew {
        carrier: NodePath,
        nominal: NominalId,
        value: Box<CheckedExpression>,
    },
    BoxDeref {
        carrier: NodePath,
        nominal: NominalId,
        referent: CheckedType,
        value: Box<CheckedExpression>,
    },
    /// One `arena_new::<'r, T>(v)` allocation [STOR-2]: the content moves into
    /// region-owned storage registered on the region's allocation list, and
    /// the whole list is released with the region [STOR-3, STOR-4].
    ArenaNew {
        carrier: NodePath,
        nominal: NominalId,
        /// The owning region's hidden allocation-list binding.
        list: BindingId,
        value: Box<CheckedExpression>,
    },
    /// Arena content read through explicit `deref` [STOR-2, TYPE-7].
    ArenaDeref {
        carrier: NodePath,
        nominal: NominalId,
        content: CheckedType,
        value: Box<CheckedExpression>,
    },
    BorrowBuffer {
        carrier: NodePath,
        root: CheckedBufferRoot,
    },
    /// A borrow of directly stored content, addressed by its complete typed
    /// field/subscript path [OWN-2, OWN-5, OP-4].
    BorrowAddressed {
        carrier: NodePath,
        root: CheckedContainerRoot,
    },
    BorrowBox {
        carrier: NodePath,
        binding: BindingId,
        nominal: NominalId,
    },
    /// The same address, taken from a binding that already holds one: a borrow
    /// whose place is rooted at another borrow holder [OWN-6, OWN-10].
    ReborrowAddressed {
        carrier: NodePath,
        binding: BindingId,
        ty: CheckedType,
    },
    /// The referent value read through such a holder [TYPE-7]. The holder
    /// itself stays a distinct expression, so lowering never has to guess
    /// whether a borrow binding is being passed on or read through.
    DerefAddressed {
        carrier: NodePath,
        binding: BindingId,
        ty: CheckedType,
    },
    ConstructStruct {
        carrier: NodePath,
        nominal: NominalId,
        fields: Vec<CheckedExpression>,
    },
    ConstructEnum {
        carrier: NodePath,
        nominal: NominalId,
        variant: u32,
        fields: Vec<CheckedExpression>,
    },
    Project {
        carrier: NodePath,
        binding: BindingId,
        fields: Vec<u32>,
        ty: CheckedType,
        consume_root: bool,
        residual_drops: Vec<CheckedProjectedDrop>,
    },
    ProjectValue {
        carrier: NodePath,
        value: Box<CheckedExpression>,
        nominal: NominalId,
        field: u32,
        ty: CheckedType,
    },
}

impl CheckedExpression {
    /// Exact PRV-1 carrier node for a positive explicit-dataflow edge.
    pub(crate) const fn carrier(&self) -> Option<&NodePath> {
        match self {
            Self::Constant(_)
            | Self::NamedConstant { .. }
            | Self::ArrayMeasure { .. }
            | Self::BufferMeasure { .. }
            | Self::ContainerMeasure { .. }
            | Self::PostconditionResultMeasure { .. }
            | Self::SliceMeasure { .. } => None,
            Self::UserCall { call, .. } | Self::KernelCall { call, .. } => Some(call),
            Self::Binding { carrier, .. }
            | Self::IntegerOperation { carrier, .. }
            | Self::FloatOperation { carrier, .. }
            | Self::NumericConversion { carrier, .. }
            | Self::Reinterpret { carrier, .. }
            | Self::BooleanOperation { carrier, .. }
            | Self::EnumEquality { carrier, .. }
            | Self::ArrayFill { carrier, .. }
            | Self::ArrayIndex { carrier, .. }
            | Self::BufferFill { carrier, .. }
            | Self::BufferVacant { carrier, .. }
            | Self::BufferFits { carrier, .. }
            | Self::BufferIndex { carrier, .. }
            | Self::ReadStorage { carrier, .. }
            | Self::SliceOf { carrier, .. }
            | Self::SliceIndex { carrier, .. }
            | Self::BoxNew { carrier, .. }
            | Self::BoxDeref { carrier, .. }
            | Self::ArenaNew { carrier, .. }
            | Self::ArenaDeref { carrier, .. }
            | Self::BorrowBuffer { carrier, .. }
            | Self::BorrowAddressed { carrier, .. }
            | Self::BorrowBox { carrier, .. }
            | Self::ReborrowAddressed { carrier, .. }
            | Self::DerefAddressed { carrier, .. }
            | Self::ConstructStruct { carrier, .. }
            | Self::ConstructEnum { carrier, .. }
            | Self::Project { carrier, .. }
            | Self::ProjectValue { carrier, .. } => Some(carrier),
        }
    }

    pub(crate) const fn ty(&self) -> CheckedType {
        match self {
            Self::Constant(value) => value.ty(),
            Self::NamedConstant { value, .. } => value.ty(),
            Self::Binding { ty, .. }
            | Self::UserCall { result: ty, .. }
            | Self::KernelCall { result: ty, .. } => *ty,
            Self::IntegerOperation { result, .. } | Self::NumericConversion { result, .. } => {
                *result
            }
            Self::Reinterpret { destination, .. } => destination.ty(),
            Self::FloatOperation {
                operation,
                operand_type,
                ..
            } => operation.result_type(*operand_type),
            Self::BooleanOperation { .. } | Self::EnumEquality { .. } => CheckedType::Bool,
            Self::ArrayFill { ty, .. } => *ty,
            Self::ArrayMeasure { .. } => CheckedType::Integer(IntegerType::U64),
            Self::ArrayIndex { element_type, .. } => *element_type,
            Self::BufferFill { element, .. } => CheckedType::Buffer { element: *element },
            Self::BufferVacant { element, .. } => CheckedType::Buffer {
                element: CheckedFlatElement::Nominal(*element),
            },
            Self::BufferFits { .. } => CheckedType::Bool,
            Self::BufferMeasure { .. }
            | Self::ContainerMeasure { .. }
            | Self::PostconditionResultMeasure { .. } => CheckedType::Integer(IntegerType::U64),
            Self::BufferIndex { root, .. } => root.element.ty(),
            Self::ReadStorage { root, .. } => root.ty,
            Self::SliceOf {
                region,
                element,
                strength,
                ..
            } => CheckedType::Slice {
                region: *region,
                element: *element,
                strength: *strength,
            },
            Self::SliceMeasure { .. } => CheckedType::Integer(IntegerType::U64),
            Self::SliceIndex { root, .. } => root.element.ty(),
            Self::BoxNew { nominal, .. } | Self::ArenaNew { nominal, .. } => {
                CheckedType::Nominal(*nominal)
            }
            Self::BoxDeref { referent, .. } => *referent,
            Self::ArenaDeref { content, .. } => *content,
            Self::BorrowBuffer { root, .. } => CheckedType::Buffer {
                element: root.element,
            },
            Self::BorrowAddressed { root, .. } => root.ty,
            Self::ReborrowAddressed { ty, .. } | Self::DerefAddressed { ty, .. } => *ty,
            Self::BorrowBox { nominal, .. } => CheckedType::Nominal(*nominal),
            Self::ConstructStruct { nominal, .. } | Self::ConstructEnum { nominal, .. } => {
                CheckedType::Nominal(*nominal)
            }
            Self::Project { ty, .. } | Self::ProjectValue { ty, .. } => *ty,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedEnumType {
    Bool,
    Nominal(NominalId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedMatchBinder {
    pub(crate) node_path: NodePath,
    pub(crate) binding: BindingId,
    pub(crate) field: u32,
    pub(crate) mode: CheckedMode,
    pub(crate) ty: CheckedType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedMatchArm {
    pub(crate) tag: u32,
    pub(crate) binders: Vec<CheckedMatchBinder>,
    pub(crate) body: Vec<CheckedStatement>,
    pub(crate) fallthrough_drops: Vec<CheckedDrop>,
}

/// One compiler-derived release on a normal control-flow edge [STOR-3].
///
/// The record is explicit in the checked program [DIAG-2] rather than being
/// rederived from the type by every consumer. Lowering carries the record
/// into typed IR to emit the ordinary storage release [STOR-3]. Opaque
/// nominals have the empty release.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedDrop {
    /// The existing source edge whose normal exit performs this release.
    pub(crate) source_edge: NodePath,
    pub(crate) binding: BindingId,
    pub(crate) fields: Vec<u32>,
    pub(crate) ty: CheckedType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedProjectedDrop {
    pub(crate) fields: Vec<u32>,
    pub(crate) ty: CheckedType,
}

/// A SET-1 target whose root, path, copy type, and post-RHS writability have
/// all been established by semantic checking.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedWritablePlace {
    pub(crate) binding: BindingId,
    pub(crate) fields: Vec<u32>,
    pub(crate) ty: CheckedType,
    /// [LIV-2] this commit declares the binding it writes, exactly as a `let`
    /// does: the target identifier resolved to none, so the statement is the
    /// binding's own initialization and nothing before it holds its storage.
    pub(crate) declares: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedArraySetTarget {
    pub(crate) binding: BindingId,
    pub(crate) fields: Vec<u32>,
    pub(crate) array_type: CheckedType,
    pub(crate) element_type: CheckedType,
    pub(crate) length: CheckedConst,
    pub(crate) offset: CheckedExpression,
    pub(crate) obligation: NodePath,
    pub(crate) target_domain: CheckedTargetDomainObligation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedBufferSetTarget {
    pub(crate) root: CheckedBufferRoot,
    pub(crate) offset: CheckedExpression,
    pub(crate) obligation: NodePath,
    pub(crate) target_domain: CheckedTargetDomainObligation,
}

/// One element-position store through a view [SET-1, VIEW-1].
///
/// The root's own loan strength is what admitted this target: [SET-1] admits
/// a target path through a view exactly when that view's loan on its origin
/// set is exclusive, so a `MutSlice` root reaches here and a `Slice` root is
/// refused where the place is formed. The storage written is the origin's,
/// which is why the statement's effect row and its [MSR-2] kill are stated
/// over the view's origins and not over the descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedSliceSetTarget {
    pub(crate) root: CheckedSliceRoot,
    pub(crate) offset: CheckedExpression,
    pub(crate) obligation: NodePath,
    pub(crate) target_domain: CheckedTargetDomainObligation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedSetTarget {
    Place(CheckedWritablePlace),
    ArrayIndex(Box<CheckedArraySetTarget>),
    BufferIndex(Box<CheckedBufferSetTarget>),
    /// A typed storage path including all subscripts and terminal fields.
    Storage(CheckedContainerRoot),
    /// [SET-1, VIEW-1] one element-position store through an exclusive view.
    SliceIndex(Box<CheckedSliceSetTarget>),
}

impl CheckedSetTarget {
    pub(crate) fn binding(&self) -> BindingId {
        match self {
            Self::Place(target) => target.binding,
            Self::ArrayIndex(target) => target.binding,
            Self::BufferIndex(target) => target.root.binding,
            Self::Storage(target) => target
                .binding()
                .expect("checked mutation targets have local roots"),
            Self::SliceIndex(target) => target.root.binding,
        }
    }

    pub(crate) fn ty(&self) -> CheckedType {
        match self {
            Self::Place(target) => target.ty,
            Self::ArrayIndex(target) => target.element_type,
            Self::BufferIndex(target) => target.root.element.ty(),
            Self::Storage(target) => target.ty,
            Self::SliceIndex(target) => target.root.element.ty(),
        }
    }
}

/// [LIV-2] the ordinal values one `set` target list commits.
///
/// The two shapes are the two right-hand sides the rule admits, and nothing
/// below this point asks which spelling produced them: a result list projects
/// ordinal i out of one call's value, and a written value list holds ordinal i
/// as its own expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedCommitValues {
    /// One call whose callee declares an ordered result list [CALL-4]; target
    /// i takes result ordinal i, which is field i of that value.
    ///
    /// The call is boxed because a checked expression is the largest value in
    /// this tree and the other shape holds its own in a `Vec`.
    ResultList {
        /// The callee's result-list nominal [CALL-4].
        nominal: NominalId,
        value: Box<CheckedExpression>,
    },
    /// A written value list: expression i is ordinal i, evaluated left to
    /// right and committed after the last one is evaluated.
    Written(Vec<CheckedExpression>),
}

impl CheckedCommitValues {
    /// Every ordinal value, in written order. A result list holds its one
    /// call value; a value list holds one expression per target.
    pub(crate) fn expressions(&self) -> &[CheckedExpression] {
        match self {
            Self::ResultList { value, .. } => std::slice::from_ref(value.as_ref()),
            Self::Written(values) => values,
        }
    }

    /// Every ordinal value, mutably, for the passes that rewrite expressions
    /// in place.
    pub(crate) fn expressions_mut(&mut self) -> &mut [CheckedExpression] {
        match self {
            Self::ResultList { value, .. } => std::slice::from_mut(value.as_mut()),
            Self::Written(values) => values,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PropagationContext {
    pub(crate) function: String,
    pub(crate) node_path: NodePath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedStatement {
    Let {
        node_path: NodePath,
        binding: BindingId,
        value: CheckedExpression,
    },
    /// [GRAM-4, CALL-4] `let (a, b) = f(...);`. One evaluation of a call whose
    /// callee declares an ordered result list, or the source consumes a
    /// struct or cell. Binders follow field order for products; a cell's
    /// sole binder receives its referent.
    DestructuringLet {
        node_path: NodePath,
        /// Each binder and its selected value type, in written order.
        bindings: Vec<(BindingId, CheckedType)>,
        /// The consumed product or cell nominal [CALL-4, TYPE-6, S39].
        nominal: NominalId,
        value: CheckedExpression,
    },
    /// [GRAM-4, CALL-4, LIV-2] `set (x, y) = rhs;`. The right-hand side is
    /// evaluated once and completely, then ordinal i is committed to target i
    /// at one commit, in written order.
    SetList {
        node_path: NodePath,
        targets: Vec<CheckedSetTarget>,
        values: CheckedCommitValues,
    },
    PropagateLet {
        /// Complete owning `let_stmt`, shared by Ok delivery and Err return.
        node_path: NodePath,
        binding: BindingId,
        scrutinee: CheckedExpression,
        result_nominal: NominalId,
        return_nominal: NominalId,
        ok_type: CheckedType,
        error_type: CheckedType,
        error_drops: Vec<CheckedDrop>,
        context: PropagationContext,
    },
    Set {
        node_path: NodePath,
        target: CheckedSetTarget,
        value: CheckedExpression,
    },
    /// A [SET-2] affine-place replacement: one read of the previous value
    /// into the fresh binding and one write of the replacement into the
    /// target, with no writer-observable point between them. The target
    /// root stays live; the commit is not a consuming use.
    Replace {
        node_path: NodePath,
        binding: BindingId,
        target: CheckedSetTarget,
        value: CheckedExpression,
    },
    Evaluate(CheckedExpression),
    /// The discarded result of an expression statement, with the
    /// compiler-derived release it runs [STOR-3].
    DropExpression {
        value: CheckedExpression,
    },
    /// A finite source-written local invariant checked before it is published
    /// and erased before lowering. It has no runtime expression, effect,
    /// branch, or trap.
    Proof(CheckedSourceProof),
    Return {
        node_path: NodePath,
        value: CheckedExpression,
        drops: Vec<CheckedDrop>,
    },
    Match {
        scrutinee: CheckedExpression,
        enum_type: CheckedEnumType,
        arms: Vec<CheckedMatchArm>,
        continues: bool,
    },
    ValueMatchLet {
        node_path: NodePath,
        kind: ValueInitializerKind,
        binding: BindingId,
        result_type: CheckedType,
        scrutinee: CheckedExpression,
        enum_type: CheckedEnumType,
        arms: Vec<CheckedMatchArm>,
        continues: bool,
    },
    Give {
        node_path: NodePath,
        value: CheckedExpression,
        drops: Vec<CheckedDrop>,
    },
    Loop {
        id: CheckedLoopId,
        /// Formed source invariants awaiting the normal semantic proof
        /// checker. Their presence alone grants no authority.
        invariants: Vec<CheckedLoopInvariant>,
        body: Vec<CheckedStatement>,
        backedge_drops: Vec<CheckedDrop>,
    },
    CountedRange {
        id: CheckedLoopId,
        node_path: NodePath,
        binder: BindingId,
        lower: CheckedExpression,
        upper: CheckedExpression,
        /// Formed source invariants awaiting the normal semantic proof
        /// checker. Their presence alone grants no authority.
        invariants: Vec<CheckedLoopInvariant>,
        body: Vec<CheckedStatement>,
        backedge_drops: Vec<CheckedDrop>,
    },
    Break {
        target: CheckedLoopId,
        drops: Vec<CheckedDrop>,
    },
    /// [PROV-6] `dispose p;`. The consumed operand's release graph is walked
    /// here instead of at the scope exit; the drop list is exactly the list
    /// that exit would have carried for this value.
    Dispose {
        node_path: NodePath,
        value: CheckedExpression,
        drops: Vec<CheckedProjectedDrop>,
    },
    Region {
        /// The region's hidden arena allocation-list binding, present exactly
        /// when the block allocates into this region [STOR-2]. Lowering
        /// materializes it at region entry; its compiler-derived drop on
        /// every normal exit edge is the region's storage release
        /// [STOR-3, STOR-4].
        arena_list: Option<BindingId>,
        body: Vec<CheckedStatement>,
        fallthrough_drops: Vec<CheckedDrop>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedParameter {
    pub(crate) name: String,
    pub(crate) declaration: DeclarationId,
    /// The complete source `param` node used by checked diagnostics.
    pub(crate) node_path: NodePath,
    pub(crate) binding: BindingId,
    pub(crate) mode: CheckedMode,
    pub(crate) ty: CheckedType,
    pub(crate) slice_origins: Vec<CheckedSliceOrigin>,
}

/// One callable-boundary state identity.
///
/// `root` is a formal value-parameter declaration and `fields` are source
/// struct ordinals.  Lifetimes deliberately do not participate in identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct CheckedStatePath {
    pub(crate) root: DeclarationId,
    pub(crate) fields: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedFunction {
    /// A function-kind hypothesis exists only during symbolic template
    /// checking. The concrete inventory and lowering contain none.
    pub(crate) formal_hypothesis: bool,
    pub(crate) id: FunctionId,
    pub(crate) declaration: DeclarationId,
    pub(crate) name: String,
    pub(crate) symbol: String,
    /// Formal regions in the same declaration order `UserCall::goal_regions`
    /// uses. Retained for post-acceptance physical release specialization;
    /// semantic identity remains the canonical [`FunctionId`].
    pub(crate) region_parameters: Vec<DeclarationId>,
    pub(crate) parameters: Vec<CheckedParameter>,
    pub(crate) result_mode: CheckedMode,
    pub(crate) result: CheckedType,
    /// Ordinary loan-origin ceiling of this function's view result.
    pub(crate) slice_return_ceiling: Vec<CheckedSliceOrigin>,
    /// Formal state paths named by `writes(...)`.
    pub(crate) declared_state_writes: Vec<CheckedStatePath>,
    /// Callable-boundary predicates in `requires_clause` source order.
    pub(crate) requirements: Vec<super::goal::CheckedRequirement>,
    /// Verified-relation surfaces in `ensures_clause` source order. H1
    /// constructs this metadata; the shared entailment flow proves every
    /// clause at every selected exit.
    pub(crate) postconditions: Vec<super::postcondition::CheckedPostcondition>,
    pub(crate) body: Option<Vec<CheckedStatement>>,
    /// Whether the independently established body-entry requirements close to
    /// a contradiction. The contradiction is retained proof metadata.
    pub(crate) body_disposition: CheckedBodyDisposition,
    /// Retained [ENT] analysis summary [DIAG-2]. Semantic acceptance and
    /// diagnostics read it; lowering deliberately does not.
    #[allow(dead_code)]
    pub(crate) entailment: super::entailment::FunctionEntailment,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum CheckedBodyDisposition {
    #[default]
    Inhabited,
    Uninhabited {
        contradiction: super::entailment::DerivationId,
    },
}

/// The one source-canonical symbolic requirement retained for a generic
/// function template.
///
/// This is acceptance metadata only. It deliberately has no [`FunctionId`]
/// and ordinary lowering must not treat it as an executable instance.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedGenericRequirement {
    pub(crate) declaration: DeclarationId,
    pub(crate) requirement: super::goal::CheckedRequirement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedEffects {
    pub(crate) reads: Vec<CheckedStatePath>,
    pub(crate) writes: Vec<CheckedStatePath>,
    /// [S23] the declared `allocates` paths.
    pub(crate) allocates: Vec<CheckedStatePath>,
    /// Transitional ordinary region allocation [STOR-2, EFF-1].
    pub(crate) allocates_arenas: Vec<DeclarationId>,
}

#[derive(Debug)]
pub(crate) struct CheckedProgramData {
    pub(crate) nominals: Vec<CheckedNominal>,
    /// [BLK-4, DIAG-2] retained confinement for each complete nominal,
    /// including phantom brands. Structural types carry their region and
    /// element handles directly, so together these retain every value's set.
    #[allow(dead_code)]
    pub(crate) nominal_confinement: Vec<Vec<DeclarationId>>,
    /// Append-only structural elements, including unreachable replay history.
    /// Only handles reachable from executable types belong to lowering.
    pub(crate) elements: Vec<CheckedType>,
    // Nominal instances discovered by the ordinary function path form this
    // prefix. Later instances exist only to type-check static metadata.
    pub(crate) executable_nominal_count: usize,
    /// For each nominal, the instance it lowers as: itself, or the first
    /// instance of the same region-erased source family whose complete
    /// reclamation graph agrees [S20, PROV-1].
    ///
    /// A region is a proof-time identity. Two instances at two regions are two
    /// checked types — that is what makes a run of one store unusable at
    /// another. They share one runtime representation only when their store
    /// release classes agree at every owning position. Physical lowering may
    /// specialize the declaration-level classes in this default alias table.
    pub(crate) nominal_lowering_alias: Vec<NominalId>,
    /// Region-erased nominal families before a store-backed Box or Vector's
    /// release class is selected. Post-acceptance physical specialization
    /// combines this identity with its closed release environment; it never
    /// participates in source type equality or acceptance.
    pub(crate) nominal_physical_alias: Vec<NominalId>,
    /// Declaration-selected storage release classes for post-acceptance
    /// physical function specialization. Loan regions do not become
    /// specialization axes merely by occurring in this table.
    pub(crate) region_release_defaults: Vec<(DeclarationId, CheckedReleaseClass)>,
    pub(crate) constants: Vec<CheckedConstant>,
    /// Immutable structural table for every symbolic const expression named
    /// by retained schema metadata. `DerivedConstId` is meaningful only
    /// relative to this checked-program-owned table.
    #[allow(dead_code)]
    pub(crate) derived_consts: Vec<DerivedConst>,
    pub(crate) functions: Vec<CheckedFunction>,
    /// Concrete ordinary-call SCCs in deterministic callee-before-caller
    /// order, with component-atomic verified FN-9 summary publication.
    #[allow(dead_code)]
    pub(crate) postcondition_schedule: super::entailment::PostconditionSchedule,
    /// One symbolic requirement per source generic that declares one. These
    /// entries survive symbolic validation without entering the concrete
    /// function inventory or executable lowering path.
    #[allow(dead_code)]
    pub(crate) generic_requirements: Vec<CheckedGenericRequirement>,
    /// Read-only [PAR-1 candidate] permission table: which sibling call pairs
    /// may be overlapped, and which of those are actualizable. Acceptance
    /// never reads it, and it is identical facts-on and facts-off. The
    /// permission ledger and the overlap lowering are its consumers.
    pub(crate) permission: super::permission::PermissionMetadata,
    /// The rendered non-normative permission ledger, one line per analyzed
    /// source site in source order, each marked with whether an ordinary
    /// compile reports it. This is developer output only: the driver hands the
    /// whole report to `whitefootc --par-ledger` and the marked subset to every
    /// compile, and no mandatory record, no normative output, and no lowering
    /// decision reads it.
    pub(crate) permission_ledger: Vec<super::permission_ledger::LedgerLine>,
}

/// Every direct subexpression, for uniform recursion.
pub(crate) fn expression_children(expression: &CheckedExpression) -> Vec<&CheckedExpression> {
    match expression {
        CheckedExpression::Constant(_)
        | CheckedExpression::NamedConstant { .. }
        | CheckedExpression::Binding { .. }
        | CheckedExpression::ArrayMeasure { .. }
        | CheckedExpression::BufferMeasure { .. }
        | CheckedExpression::PostconditionResultMeasure { .. }
        | CheckedExpression::SliceMeasure { .. }
        | CheckedExpression::BorrowBuffer { .. }
        | CheckedExpression::BorrowBox { .. }
        | CheckedExpression::ReborrowAddressed { .. }
        | CheckedExpression::DerefAddressed { .. }
        | CheckedExpression::Project { .. } => Vec::new(),
        CheckedExpression::BorrowAddressed { root, .. }
        | CheckedExpression::ContainerMeasure { root, .. }
        | CheckedExpression::ReadStorage { root, .. }
        | CheckedExpression::SliceOf {
            source: CheckedSliceSource::Run(root),
            ..
        } => root.offsets().collect(),
        CheckedExpression::SliceOf { .. } => Vec::new(),
        CheckedExpression::UserCall { arguments, .. }
        | CheckedExpression::KernelCall { arguments, .. }
        | CheckedExpression::IntegerOperation { arguments, .. }
        | CheckedExpression::FloatOperation { arguments, .. }
        | CheckedExpression::BooleanOperation { arguments, .. }
        | CheckedExpression::EnumEquality { arguments, .. } => arguments.iter().collect(),
        CheckedExpression::NumericConversion { value, .. }
        | CheckedExpression::Reinterpret { value, .. }
        | CheckedExpression::ArrayFill { value, .. }
        | CheckedExpression::BoxNew { value, .. }
        | CheckedExpression::BoxDeref { value, .. }
        | CheckedExpression::ArenaNew { value, .. }
        | CheckedExpression::ArenaDeref { value, .. }
        | CheckedExpression::ProjectValue { value, .. } => vec![value.as_ref()],
        CheckedExpression::ArrayIndex { offset, .. } => vec![offset.as_ref()],
        CheckedExpression::BufferFill { length, value, .. } => {
            vec![length.as_ref(), value.as_ref()]
        }
        CheckedExpression::BufferVacant { length, .. }
        | CheckedExpression::BufferFits { length, .. } => vec![length.as_ref()],
        CheckedExpression::BufferIndex { offset, .. }
        | CheckedExpression::SliceIndex { offset, .. } => vec![offset.as_ref()],
        CheckedExpression::ConstructStruct { fields, .. }
        | CheckedExpression::ConstructEnum { fields, .. } => fields.iter().collect(),
    }
}
