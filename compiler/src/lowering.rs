//! Target-independent lowering from the semantically checked active Whitefoot specification.
//!
//! The private IR records exact value types, nominal construction/projection,
//! direct calls, erased source proofs, and explicit control-flow edges. It performs
//! no source admission, label lookup, exhaustiveness decision, or ownership
//! judgment.

use crate::semantic::{
    CheckedBooleanOperation, CheckedElement, CheckedEnumType, CheckedFlatElement,
    CheckedFloatOperation, CheckedIntegerOperation, CheckedLayoutCeiling, CheckedLayoutMagnitude,
    CheckedLoopId, CheckedNumericType, CheckedProgram, CheckedRuntimeTargetObligations,
    CheckedTargetDomainObligation, CheckedType,
};
use crate::{SystemRelease, SystemResourceContract};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IrValueId(u32);

impl IrValueId {
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.0
    }

    const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IrBlockId(u32);

impl IrBlockId {
    pub(crate) fn from_index(index: usize) -> Result<Self, LoweringFailure> {
        Ok(Self(
            u32::try_from(index).map_err(|_| LoweringFailure::CounterOverflow)?,
        ))
    }

    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.0
    }

    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IrNominalId(u32);

impl IrNominalId {
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.0
    }

    const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IrConstantId(u32);

impl IrConstantId {
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.0
    }

    const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IrFlatElement {
    Unit,
    Bool,
    Integer {
        width: u8,
        signed: bool,
    },
    Float {
        width: u8,
    },
    TagOnlyNominal(IrNominalId),
    /// One affine aggregate element: a non-copy nominal stored by value.
    /// Only `buffer` element positions carry this variant [TYPE-2].
    Nominal(IrNominalId),
}

impl IrFlatElement {
    pub const fn ty(self) -> IrType {
        match self {
            Self::Unit => IrType::Unit,
            Self::Bool => IrType::Bool,
            Self::Integer { width, signed } => IrType::Integer { width, signed },
            Self::Float { width } => IrType::Float { width },
            Self::TagOnlyNominal(id) | Self::Nominal(id) => IrType::Nominal(id),
        }
    }
}

/// [BLK-1] the type of one slot of a run, with the same one-level lift the
/// checked element domain carries: a flat element, or one run of flat
/// elements whose descriptor lives in the slot.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IrElement {
    Flat(IrFlatElement),
    FixedVector {
        element: IrFlatElement,
        length: u64,
    },
    Vector {
        element: IrFlatElement,
        release: IrReleaseClass,
    },
}

impl IrElement {
    pub const fn ty(self) -> IrType {
        match self {
            Self::Flat(element) => element.ty(),
            Self::FixedVector { element, length } => IrType::FixedVector {
                element: Self::Flat(element),
                length,
            },
            Self::Vector { element, release } => IrType::Vector {
                element: Self::Flat(element),
                release,
            },
        }
    }
}

/// One [BLK-2] take from a store, in the shape its emission reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrStoreTake {
    /// The address of the `&uniq` provider operand: the take reads the
    /// store's state and writes it back through the same borrow.
    pub store: IrValueId,
    pub count: IrValueId,
    /// The stride one slot occupies [OP-9], which is the spacing a run's
    /// window is laid out at [BLK-1].
    pub stride: u64,
    /// The bump extent's own byte extent and alignment. A general store has
    /// neither and asks its host instead.
    pub extent: Option<IrExtentConstants>,
    /// The `Option` the row hands back when the store has nothing to give; a
    /// row whose domain requirement is proved carries none.
    pub refusal: Option<IrRefusal>,
}

/// The two type constants of one bump extent [BLK-2]: its byte extent and
/// its alignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrExtentConstants {
    pub bytes: u64,
    pub align: u64,
}

/// S39 one cell formation: the store's own take, the value the cell takes,
/// and the outcome that carries either.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrStoreBox {
    /// The `&uniq` provider operand's address.
    pub store: IrValueId,
    /// The value the cell takes, consumed by this operation.
    pub value: IrValueId,
    /// The bytes one cell occupies, which is one stride rounded up to the
    /// store's own alignment where it has one [OP-9].
    pub bytes: u64,
    /// `Some` for a bump extent, whose take is a cursor advance inside the
    /// reservation; `None` for the general store, which is asked.
    pub extent: Option<IrExtentConstants>,
    /// The `Result<Box<'s, T>, T>` the row hands back: `made` is the `Ok`
    /// tag and `refused` the `Err` tag.
    pub outcome: IrRefusal,
}

/// The `Option` a refusing [BLK-2] row hands back, by the tags [PRE-1] gives
/// its two variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrRefusal {
    pub nominal: IrNominalId,
    /// The tag of the variant carrying the run.
    pub made: u32,
    /// The tag of the empty variant.
    pub refused: u32,
}

/// The referent of an [`IrType::Address`]: directly stored content that a
/// borrow addresses.
///
/// Descriptor values (`buffer`, `slice`) and opaque handles (`box`, system
/// resources) are already their own borrow and never appear here.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IrAddressed {
    Unit,
    Bool,
    Integer {
        width: u8,
        signed: bool,
    },
    Float {
        width: u8,
    },
    Nominal(IrNominalId),
    /// One `FixedVector<T, n>` [BLK-1]. A frame-resident run is inline
    /// storage in its owner, exactly as a struct is, so a borrow of one is
    /// the address of that storage rather than a copy of the run.
    FixedVector {
        element: IrElement,
        length: u64,
    },
    /// One `Vector<'s, T>` [BLK-1]. Its descriptor is storage in its owner's
    /// frame, and a borrow of the run is the address of that descriptor, so
    /// both runs are borrowed through one path.
    Vector {
        element: IrElement,
        release: IrReleaseClass,
    },
    /// One provider value [PROV-1]. A provider is the one operand a [BLK-0]
    /// acquiring row takes by `&uniq`, and a bump take advances its cursor
    /// through that borrow, so its binding carries a stable address exactly
    /// as a stored scalar's does.
    Provider,
}

impl IrAddressed {
    pub const fn ty(self) -> IrType {
        match self {
            Self::Unit => IrType::Unit,
            Self::Bool => IrType::Bool,
            Self::Integer { width, signed } => IrType::Integer { width, signed },
            Self::Float { width } => IrType::Float { width },
            Self::Nominal(id) => IrType::Nominal(id),
            Self::FixedVector { element, length } => IrType::FixedVector { element, length },
            Self::Vector { element, release } => IrType::Vector { element, release },
            Self::Provider => IrType::Provider,
        }
    }

    const fn of(ty: IrType) -> Option<Self> {
        Some(match ty {
            IrType::Unit => Self::Unit,
            IrType::Bool => Self::Bool,
            IrType::Integer { width, signed } => Self::Integer { width, signed },
            IrType::Float { width } => Self::Float { width },
            IrType::Nominal(id) => Self::Nominal(id),
            IrType::FixedVector { element, length } => Self::FixedVector { element, length },
            IrType::Vector { element, release } => Self::Vector { element, release },
            IrType::Provider => Self::Provider,
            IrType::Address(_)
            | IrType::Array { .. }
            | IrType::Buffer { .. }
            | IrType::Slice { .. } => return None,
        })
    }
}

/// [PROV-6, STOR-3] which release action a store-backed run's own reclamation
/// is, carried into the IR because the region that decided it is erased there.
///
/// The checker fixes this from the store region's declaration alone
/// [`crate::semantic::CheckedReleaseClass`]; nothing after that point
/// rediscovers it, and no lowering may infer one action from a type shape.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IrReleaseClass {
    /// A free to the general store the run was taken from.
    General,
    /// Empty: the extent's reclamation is its region's own reset [BLK-2].
    Extent,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IrType {
    Unit,
    Bool,
    Integer {
        width: u8,
        signed: bool,
    },
    Float {
        width: u8,
    },
    Nominal(IrNominalId),
    Address(IrAddressed),
    Array {
        element: IrFlatElement,
        length: u64,
    },
    Buffer {
        element: IrFlatElement,
    },
    Slice {
        element: IrFlatElement,
    },
    /// One `FixedVector<T, n>` [BLK-1]: `n` inline slots followed by the two
    /// descriptor words `len` and `head`. The capacity is the type constant
    /// and is stored nowhere.
    FixedVector {
        element: IrElement,
        length: u64,
    },
    /// One `Vector<'s, T>` [BLK-1]: the descriptor `{ pointer, cap, len,
    /// head }` over a run taken from the store `'s` names. The region is
    /// erased here, and the release action it decided travels in its place.
    Vector {
        element: IrElement,
        release: IrReleaseClass,
    },
    /// One provider value [PROV-1]. It is proof-only: the general store's
    /// provider carries no runtime state at all, and the bump extent's
    /// carries exactly its cursor.
    Provider,
}

pub(crate) const fn lower_release_class(
    value: crate::semantic::CheckedReleaseClass,
) -> IrReleaseClass {
    match value {
        crate::semantic::CheckedReleaseClass::General => IrReleaseClass::General,
        crate::semantic::CheckedReleaseClass::Extent => IrReleaseClass::Extent,
    }
}

/// One nominal's lowered identity, read through the region erasure
/// [S20, PROV-1]: two instances of one declaration that differ only in their
/// region arguments are two checked types and one IR nominal.
fn erased_nominal(erasure: &[IrNominalId], id: crate::NominalId) -> IrNominalId {
    erasure
        .get(id.0 as usize)
        .copied()
        .unwrap_or(IrNominalId(id.0))
}

fn lower_element(
    erasure: &[IrNominalId],
    value: CheckedElement,
) -> Result<IrElement, LoweringFailure> {
    Ok(match value {
        CheckedElement::Flat(element) => IrElement::Flat(lower_flat_element(erasure, element)?),
        CheckedElement::FixedVector { element, length } => IrElement::FixedVector {
            element: lower_flat_element(erasure, element)?,
            length: match length.value() {
                Some(value) => value,
                None => return Err(LoweringFailure::InvalidCheckedProgram),
            },
        },
        CheckedElement::Vector {
            element, release, ..
        } => IrElement::Vector {
            element: lower_flat_element(erasure, element)?,
            release: lower_release_class(release),
        },
    })
}

fn lower_flat_element(
    erasure: &[IrNominalId],
    value: CheckedFlatElement,
) -> Result<IrFlatElement, LoweringFailure> {
    Ok(match value {
        CheckedFlatElement::Unit => IrFlatElement::Unit,
        CheckedFlatElement::Bool => IrFlatElement::Bool,
        CheckedFlatElement::Integer(integer) => IrFlatElement::Integer {
            width: integer.width(),
            signed: integer.signed(),
        },
        CheckedFlatElement::Float(float) => IrFlatElement::Float {
            width: float.width(),
        },
        // [FN-2] a symbolic element belongs to the pre-IR pass alone: every
        // lowered instance is concrete.
        CheckedFlatElement::GenericInt(_) | CheckedFlatElement::Generic(_) => {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        CheckedFlatElement::GenericFloat(_) => {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        CheckedFlatElement::TagOnlyNominal(id) => {
            IrFlatElement::TagOnlyNominal(erased_nominal(erasure, id))
        }
        CheckedFlatElement::Nominal(id) => IrFlatElement::Nominal(erased_nominal(erasure, id)),
    })
}

fn lower_type(erasure: &[IrNominalId], value: CheckedType) -> Result<IrType, LoweringFailure> {
    Ok(match value {
        CheckedType::Unit => IrType::Unit,
        CheckedType::Bool => IrType::Bool,
        CheckedType::Integer(integer) => IrType::Integer {
            width: integer.width(),
            signed: integer.signed(),
        },
        CheckedType::Float(float) => IrType::Float {
            width: float.width(),
        },
        CheckedType::Generic(_) | CheckedType::GenericInt(_) | CheckedType::GenericFloat(_) => {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        CheckedType::Nominal(id) => IrType::Nominal(erased_nominal(erasure, id)),
        CheckedType::Array { element, length } => IrType::Array {
            element: lower_flat_element(erasure, element)?,
            length: length
                .value()
                .ok_or(LoweringFailure::InvalidCheckedProgram)?,
        },
        CheckedType::Buffer { element } => IrType::Buffer {
            element: lower_flat_element(erasure, element)?,
        },
        CheckedType::Slice { element, .. } => IrType::Slice {
            element: lower_flat_element(erasure, element)?,
        },
        CheckedType::FixedVector { element, length } => IrType::FixedVector {
            element: lower_element(erasure, element)?,
            length: length
                .value()
                .ok_or(LoweringFailure::InvalidCheckedProgram)?,
        },
        CheckedType::Vector {
            element, release, ..
        } => IrType::Vector {
            release: lower_release_class(release),
            element: lower_element(erasure, element)?,
        },
        CheckedType::Heap { .. } | CheckedType::Extent { .. } => IrType::Provider,
    })
}

const fn lower_numeric_type(value: CheckedNumericType) -> IrType {
    match value {
        CheckedNumericType::Integer(integer) => IrType::Integer {
            width: integer.width(),
            signed: integer.signed(),
        },
        CheckedNumericType::Float(float) => IrType::Float {
            width: float.width(),
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrField {
    ty: IrType,
}

impl IrField {
    pub const fn ty(&self) -> IrType {
        self.ty
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrVariant {
    tag: u32,
    fields: Vec<IrField>,
}

impl IrVariant {
    pub const fn tag(&self) -> u32 {
        self.tag
    }

    pub fn fields(&self) -> &[IrField] {
        &self.fields
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrNominalKind {
    Struct {
        fields: Vec<IrField>,
    },
    Enum {
        variants: Vec<IrVariant>,
    },
    Box {
        referent: IrType,
        /// [PROV-6, S39] which release action this cell's own reclamation is.
        /// The ambient-heap `box<T>` [STOR-2] and a `Box<'s, T>` at a general
        /// store both free their cell; a `Box<'s, T>` at a bump extent is
        /// reclaimed by its region's own reset and has no action of its own.
        release: IrReleaseClass,
    },
    /// One `arena<'r, T>` instance: a pointer-shaped handle to region-owned
    /// heap content, released with its region rather than with an owner
    /// scope [STOR-3, STOR-4].
    Arena {
        content: IrType,
    },
    /// One region block's compiler-owned arena allocation-list cell; its
    /// drop walks and frees every registered allocation [STOR-3].
    ArenaStorage,
    /// One [SYS-2] opaque system resource type. It has no field, variant, or
    /// source-visible content: its identity is the target-independent
    /// semantic identity [QUAL-1] the contract carries, together with the
    /// [SYS-5] release action, that action's row, and the [HOST-3] backing
    /// class. Every use of a value of this type — a move, a `match` binder, a
    /// struct or enum field, a return, or a call argument — keeps that
    /// identity, because the type is what fixes the release action.
    SystemResource(SystemResourceContract),
}

/// The declaration family that gave one nominal its identity before lowering.
///
/// LLVM shape is deliberately not type identity: a source enum, a prelude
/// `Result`, and a system outcome can have identical fields while remaining
/// non-interchangeable. The backend retains this compact origin so its
/// target-facing signature checks fail closed on malformed IR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrNominalIdentity {
    /// A source nominal or a prelude nominal other than `Result`.
    Ordinary,
    /// One concrete prelude `Result<T, E>` instance.
    PreludeResult,
    /// One exact [SYS-2] nominal-table row.
    System(u8),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrNominal {
    id: IrNominalId,
    identity: IrNominalIdentity,
    kind: IrNominalKind,
}

impl IrNominal {
    pub const fn id(&self) -> IrNominalId {
        self.id
    }

    /// The declaration family retained independently of representation.
    pub const fn identity(&self) -> IrNominalIdentity {
        self.identity
    }

    pub const fn kind(&self) -> &IrNominalKind {
        &self.kind
    }

    pub fn is_tag_only_enum(&self) -> bool {
        matches!(
            &self.kind,
            IrNominalKind::Enum { variants }
                if variants.iter().all(|variant| variant.fields.is_empty())
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrEnumType {
    Bool,
    Nominal(IrNominalId),
}

fn lower_enum_type(erasure: &[IrNominalId], value: CheckedEnumType) -> IrEnumType {
    match value {
        CheckedEnumType::Bool => IrEnumType::Bool,
        CheckedEnumType::Nominal(id) => IrEnumType::Nominal(erased_nominal(erasure, id)),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrIntegerOperation {
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

impl From<CheckedIntegerOperation> for IrIntegerOperation {
    fn from(value: CheckedIntegerOperation) -> Self {
        match value {
            CheckedIntegerOperation::AddWrap => Self::AddWrap,
            CheckedIntegerOperation::SubtractWrap => Self::SubtractWrap,
            CheckedIntegerOperation::MultiplyWrap => Self::MultiplyWrap,
            CheckedIntegerOperation::AddExact => Self::AddExact,
            CheckedIntegerOperation::SubtractExact => Self::SubtractExact,
            CheckedIntegerOperation::MultiplyExact => Self::MultiplyExact,
            CheckedIntegerOperation::AddDefined => Self::AddDefined,
            CheckedIntegerOperation::SubtractDefined => Self::SubtractDefined,
            CheckedIntegerOperation::MultiplyDefined => Self::MultiplyDefined,
            CheckedIntegerOperation::AddChecked => Self::AddChecked,
            CheckedIntegerOperation::SubtractChecked => Self::SubtractChecked,
            CheckedIntegerOperation::MultiplyChecked => Self::MultiplyChecked,
            CheckedIntegerOperation::DivideChecked => Self::DivideChecked,
            CheckedIntegerOperation::RemainderChecked => Self::RemainderChecked,
            CheckedIntegerOperation::DivideExact => Self::DivideExact,
            CheckedIntegerOperation::RemainderExact => Self::RemainderExact,
            CheckedIntegerOperation::DivideDefined => Self::DivideDefined,
            CheckedIntegerOperation::RemainderDefined => Self::RemainderDefined,
            CheckedIntegerOperation::AbsoluteWrap => Self::AbsoluteWrap,
            CheckedIntegerOperation::AbsoluteExact => Self::AbsoluteExact,
            CheckedIntegerOperation::AbsoluteDefined => Self::AbsoluteDefined,
            CheckedIntegerOperation::AbsoluteChecked => Self::AbsoluteChecked,
            CheckedIntegerOperation::NegateWrap => Self::NegateWrap,
            CheckedIntegerOperation::NegateExact => Self::NegateExact,
            CheckedIntegerOperation::NegateDefined => Self::NegateDefined,
            CheckedIntegerOperation::NegateChecked => Self::NegateChecked,
            CheckedIntegerOperation::BitAnd => Self::BitAnd,
            CheckedIntegerOperation::BitOr => Self::BitOr,
            CheckedIntegerOperation::BitXor => Self::BitXor,
            CheckedIntegerOperation::BitNot => Self::BitNot,
            CheckedIntegerOperation::ShiftLeftWrap => Self::ShiftLeftWrap,
            CheckedIntegerOperation::ShiftRightWrap => Self::ShiftRightWrap,
            CheckedIntegerOperation::ShiftLeftExact => Self::ShiftLeftExact,
            CheckedIntegerOperation::ShiftRightExact => Self::ShiftRightExact,
            CheckedIntegerOperation::ShiftLeftDefined => Self::ShiftLeftDefined,
            CheckedIntegerOperation::ShiftRightDefined => Self::ShiftRightDefined,
            CheckedIntegerOperation::RotateLeft => Self::RotateLeft,
            CheckedIntegerOperation::RotateRight => Self::RotateRight,
            CheckedIntegerOperation::PopulationCount => Self::PopulationCount,
            CheckedIntegerOperation::LeadingZeros => Self::LeadingZeros,
            CheckedIntegerOperation::TrailingZeros => Self::TrailingZeros,
            CheckedIntegerOperation::ByteSwap => Self::ByteSwap,
            CheckedIntegerOperation::MultiplyHigh => Self::MultiplyHigh,
            CheckedIntegerOperation::AddSaturating => Self::AddSaturating,
            CheckedIntegerOperation::SubtractSaturating => Self::SubtractSaturating,
            CheckedIntegerOperation::MultiplySaturating => Self::MultiplySaturating,
            CheckedIntegerOperation::Minimum => Self::Minimum,
            CheckedIntegerOperation::Maximum => Self::Maximum,
            CheckedIntegerOperation::Equal => Self::Equal,
            CheckedIntegerOperation::NotEqual => Self::NotEqual,
            CheckedIntegerOperation::Less => Self::Less,
            CheckedIntegerOperation::LessEqual => Self::LessEqual,
            CheckedIntegerOperation::Greater => Self::Greater,
            CheckedIntegerOperation::GreaterEqual => Self::GreaterEqual,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrBooleanOperation {
    And,
    Or,
    ExclusiveOr,
    Not,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrFloatOperation {
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

impl From<CheckedFloatOperation> for IrFloatOperation {
    fn from(value: CheckedFloatOperation) -> Self {
        match value {
            CheckedFloatOperation::AddStrict => Self::AddStrict,
            CheckedFloatOperation::SubtractStrict => Self::SubtractStrict,
            CheckedFloatOperation::MultiplyStrict => Self::MultiplyStrict,
            CheckedFloatOperation::DivideStrict => Self::DivideStrict,
            CheckedFloatOperation::Equal => Self::Equal,
            CheckedFloatOperation::Less => Self::Less,
            CheckedFloatOperation::LessEqual => Self::LessEqual,
            CheckedFloatOperation::Greater => Self::Greater,
            CheckedFloatOperation::GreaterEqual => Self::GreaterEqual,
            CheckedFloatOperation::NotEqual => Self::NotEqual,
            CheckedFloatOperation::Negate => Self::Negate,
            CheckedFloatOperation::Absolute => Self::Absolute,
            CheckedFloatOperation::CopySign => Self::CopySign,
            CheckedFloatOperation::Minimum => Self::Minimum,
            CheckedFloatOperation::Maximum => Self::Maximum,
            CheckedFloatOperation::Floor => Self::Floor,
            CheckedFloatOperation::Ceil => Self::Ceil,
            CheckedFloatOperation::Truncate => Self::Truncate,
            CheckedFloatOperation::RoundEven => Self::RoundEven,
            CheckedFloatOperation::Remainder => Self::Remainder,
            CheckedFloatOperation::SquareRootStrict => Self::SquareRootStrict,
            CheckedFloatOperation::FusedMultiplyAddStrict => Self::FusedMultiplyAddStrict,
            CheckedFloatOperation::Infinity => Self::Infinity,
            CheckedFloatOperation::Nan => Self::Nan,
        }
    }
}

impl From<CheckedBooleanOperation> for IrBooleanOperation {
    fn from(value: CheckedBooleanOperation) -> Self {
        match value {
            CheckedBooleanOperation::And => Self::And,
            CheckedBooleanOperation::Or => Self::Or,
            CheckedBooleanOperation::ExclusiveOr => Self::ExclusiveOr,
            CheckedBooleanOperation::Not => Self::Not,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrConstant {
    Unit,
    Bool(bool),
    Integer { ty: IrType, bits: u64 },
    Float { ty: IrType, bits: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrGlobalValue {
    Scalar(IrConstant),
    Array(Vec<IrConstant>),
    /// One struct-typed rodata constant [CONST-2 candidate]: complete field
    /// values in declared order.
    Struct(Vec<IrGlobalValue>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrGlobalConstant {
    id: IrConstantId,
    name: String,
    ty: IrType,
    value: IrGlobalValue,
}

impl IrGlobalConstant {
    pub const fn id(&self) -> IrConstantId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn ty(&self) -> IrType {
        self.ty
    }

    pub const fn value(&self) -> &IrGlobalValue {
        &self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrArrayRoot {
    Value(IrValueId),
    Constant(IrConstantId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrTargetDomainObligation {
    RuntimeSizedAllocation,
    ElementAddress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrRuntimeTargetObligations {
    allocation: IrTargetDomainObligation,
    element_address: IrTargetDomainObligation,
    source_length_upper_bound: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrLayoutMagnitude {
    Finite(u64),
    AboveU64,
}

impl IrLayoutMagnitude {
    pub(crate) const fn permits(self, actual: u64) -> bool {
        match self {
            Self::Finite(ceiling) => actual <= ceiling,
            Self::AboveU64 => true,
        }
    }
}

impl From<CheckedLayoutMagnitude> for IrLayoutMagnitude {
    fn from(value: CheckedLayoutMagnitude) -> Self {
        match value {
            CheckedLayoutMagnitude::Finite(value) => Self::Finite(value),
            CheckedLayoutMagnitude::AboveU64 => Self::AboveU64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrLayoutCeiling {
    pub size: IrLayoutMagnitude,
    pub align: u64,
    pub stride: IrLayoutMagnitude,
}

impl From<CheckedLayoutCeiling> for IrLayoutCeiling {
    fn from(value: CheckedLayoutCeiling) -> Self {
        Self {
            size: value.size.into(),
            align: value.align,
            stride: value.stride.into(),
        }
    }
}

impl TryFrom<CheckedRuntimeTargetObligations> for IrRuntimeTargetObligations {
    type Error = LoweringFailure;

    fn try_from(value: CheckedRuntimeTargetObligations) -> Result<Self, Self::Error> {
        Ok(Self {
            allocation: value.allocation().into(),
            element_address: value.element_address().into(),
            source_length_upper_bound: value
                .source_length_upper_bound()
                .ok_or(LoweringFailure::InvalidCheckedProgram)?,
        })
    }
}

impl IrRuntimeTargetObligations {
    pub(crate) const fn is_complete(self) -> bool {
        matches!(
            (self.allocation, self.element_address),
            (
                IrTargetDomainObligation::RuntimeSizedAllocation,
                IrTargetDomainObligation::ElementAddress
            )
        )
    }

    pub(crate) const fn source_length_upper_bound(self) -> u64 {
        self.source_length_upper_bound
    }
}

impl From<CheckedTargetDomainObligation> for IrTargetDomainObligation {
    fn from(value: CheckedTargetDomainObligation) -> Self {
        match value {
            CheckedTargetDomainObligation::RuntimeSizedAllocation => Self::RuntimeSizedAllocation,
            CheckedTargetDomainObligation::ElementAddress => Self::ElementAddress,
        }
    }
}

/// The target-independent semantic identity of one [SYS-2] system operation
/// [QUAL-1].
///
/// It is the operation's index in the specification's own inventory table, so
/// no source function name or spelling, logical path, project, corpus, test,
/// or signature lookalike can select one. A target stage maps this identity
/// to one approved implementation and one private ABI symbol; the identity
/// itself names no target.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IrSystemOperation(u8);

impl IrSystemOperation {
    // Read by the target stage that maps this identity through the [QUAL-1]
    // qualification table; nothing before that stage may dispatch on it.
    #[allow(dead_code)]
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        self.0
    }
}

/// The [MSR-1] measure one reader row loads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrMeasure {
    Length,
    Capacity,
    Room,
    Head,
}

/// Which of [BLK-3]'s four boundary operations one run operation is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrBoundary {
    PlaceBack,
    PlaceFront,
    TakeBack,
    TakeFront,
}

impl IrBoundary {
    /// Whether this row moves the front boundary, which is the one that can
    /// leave `head` nonzero [MSR-1].
    #[must_use]
    pub const fn front(self) -> bool {
        matches!(self, Self::PlaceFront | Self::TakeFront)
    }

    /// Whether this row places a value rather than removing one.
    #[must_use]
    pub const fn places(self) -> bool {
        matches!(self, Self::PlaceBack | Self::PlaceFront)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrOperation {
    Constant(IrConstant),
    Call {
        function: u32,
        arguments: Vec<IrValueId>,
    },
    /// One call to a [SYS-2] system operation, by semantic identity, with its
    /// value arguments in declared parameter order.
    SystemCall {
        operation: IrSystemOperation,
        /// Compiler-owned execution and completion contract.
        target_action: crate::TargetAction,
        arguments: Vec<IrValueId>,
    },
    Integer {
        operation: IrIntegerOperation,
        operand_type: IrType,
        arguments: Vec<IrValueId>,
    },
    Float {
        operation: IrFloatOperation,
        operand_type: IrType,
        arguments: Vec<IrValueId>,
    },
    NumericConversion {
        source_type: IrType,
        destination_type: IrType,
        value: IrValueId,
    },
    Reinterpret {
        source_type: IrType,
        destination_type: IrType,
        value: IrValueId,
    },
    Boolean {
        operation: IrBooleanOperation,
        arguments: Vec<IrValueId>,
    },
    EnumEquality {
        equal: bool,
        operand_type: IrType,
        arguments: [IrValueId; 2],
    },
    ArrayFill {
        value: IrValueId,
        target_domain: IrTargetDomainObligation,
    },
    /// One discharged source subscript read [OP-4]: the checker has already
    /// derived the bounds obligation, so no runtime branch is emitted in any
    /// build mode; the offset is consumed directly.
    ArrayIndex {
        root: IrArrayRoot,
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
    },
    InsertArray {
        aggregate: IrValueId,
        index: IrValueId,
        value: IrValueId,
    },
    BufferFill {
        length: IrValueId,
        value: IrValueId,
        layout_ceiling: IrLayoutCeiling,
        target_domains: IrRuntimeTargetObligations,
    },
    /// One `buffer_vacant::<T>(n)` allocation [OP-1, OP-9]: the defined value's
    /// buffer type names the `Option<T>` element instance, and every element
    /// is initialized to the compiler-minted `None()` of that instance.
    BufferVacant {
        length: IrValueId,
        layout_ceiling: IrLayoutCeiling,
        target_domains: IrRuntimeTargetObligations,
    },
    BufferFits {
        length: IrValueId,
        maximum_length: u64,
    },
    BufferMeasure {
        buffer: IrValueId,
    },
    /// [BLK-2] `fixed_vector`: one frame-resident run of the defined type's own
    /// capacity, whose window is empty. Every slot is raw and the two
    /// descriptor words are zero.
    FixedVector,
    /// [BLK-2] `arena_frame`: one bump extent reserved in the reserving
    /// activation's own frame. The provider value is that reservation's base
    /// address and its cursor, and the reservation establishes the extent's
    /// initial state — the cursor at zero — at every activation of the region
    /// block naming its store region.
    ArenaFrame {
        bytes: u64,
        align: u64,
    },
    /// [BLK-2] one take from a store: the run of `count` slots the store
    /// hands out, and the store's own advanced state.
    ///
    /// `store` is the address of the `&uniq` provider operand, so the take
    /// reads the store's state and writes it back through the same borrow.
    /// A `refusal` names the `Option` the row hands back when the store has
    /// nothing to give; a row whose domain requirement is proved carries
    /// none and always succeeds.
    StoreTake(IrStoreTake),
    /// S39 one cell formation over a store.
    StoreBox(IrStoreBox),
    /// [MSR-1] one measure of a run or a bump extent, read as its [OP-1]
    /// reader row loads it. A cell the measure table fixes as a constant
    /// never reaches here.
    ContainerMeasure {
        measure: IrMeasure,
        container: IrValueId,
    },
    /// One discharged source subscript read of a run [OP-4, BLK-1]: the
    /// offset is a logical one and the storage read is slot
    /// `(head + i) mod cap`. See [`Self::ArrayIndex`] for the discharge.
    RunIndex {
        run: IrValueId,
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
    },
    /// One discharged element-position store into a run [SET-1, SET-2,
    /// BLK-1]: the offset is a logical one and the storage written is slot
    /// `(head + i) mod cap`. The value is the run with that slot replaced;
    /// the two descriptor words are untouched.
    RunStore {
        run: IrValueId,
        offset: IrValueId,
        value: IrValueId,
        target_domain: IrTargetDomainObligation,
    },
    /// [BLK-3] the run one boundary operation hands back: one store at the
    /// boundary slot for a placement, and one boundary arithmetic for both.
    RunBoundary {
        row: IrBoundary,
        run: IrValueId,
        /// The placed element; a removal row has none.
        value: Option<IrValueId>,
    },
    /// [BLK-3] the element a removal row hands back, read from the boundary
    /// slot before the boundary moves.
    RunTaken {
        row: IrBoundary,
        run: IrValueId,
    },
    /// One discharged source subscript read [OP-4]; see [`Self::ArrayIndex`].
    BufferIndex {
        buffer: IrValueId,
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
    },
    /// One semantics-preserving wide-probe step over a `u8` buffer.
    ///
    /// Computes how many upcoming iterations of a recognized byte-walk loop
    /// are provably no-ops: the count of leading bytes at `index ..` that
    /// match no needle, but only when `index + 16 <= min(limit, length)`
    /// bounds both the walk's exit guard and every skipped read; otherwise 0.
    /// Every byte at which anything observable can happen — a needle hit or
    /// the exit bound — therefore reaches the unchanged scalar body. The probe
    /// itself reads only bytes its internal guard proves in bounds.
    BufferProbeSkip {
        buffer: IrValueId,
        index: IrValueId,
        limit: IrValueId,
        needles: Vec<IrValueId>,
    },
    SliceFromArray {
        array: IrArrayRoot,
    },
    SliceFromBuffer {
        buffer: IrValueId,
    },
    /// [VIEW-2] one view over a run's initialized window [BLK-1].
    ///
    /// The window is `len` slots beginning at `head`, and the row's own
    /// requirement `head_of(vector) <= room_of(vector)` was discharged before
    /// this operation exists, so the window is one contiguous range and the
    /// descriptor is the slot at `head` together with `len`.
    SliceFromRun {
        run: IrValueId,
    },
    SliceMeasure {
        slice: IrValueId,
    },
    /// One discharged source subscript read [OP-4]; see [`Self::ArrayIndex`].
    SliceIndex {
        slice: IrValueId,
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
    },
    BoxNew {
        nominal: IrNominalId,
        value: IrValueId,
    },
    /// S39 the destructuring consume of a cell: its referent is loaded out
    /// and its own storage is released, which is a free on a general store
    /// and nothing on a bump extent.
    BoxTake {
        nominal: IrNominalId,
        value: IrValueId,
    },
    BoxDeref {
        nominal: IrNominalId,
        value: IrValueId,
    },
    /// One region block's arena allocation-list cell, materialized at region
    /// entry: a stack cell reset to empty, whose address is the operation's
    /// value [STOR-2, STOR-3].
    ArenaListNew,
    /// One `arena_new` allocation: heap storage for the content, registered
    /// on the owning region's allocation list so the region's exit release
    /// frees it [STOR-2, STOR-3, STOR-4]. The value is the content address.
    ArenaNew {
        nominal: IrNominalId,
        list: IrValueId,
        value: IrValueId,
    },
    /// Arena content read through explicit `deref` [STOR-2].
    ArenaDeref {
        nominal: IrNominalId,
        value: IrValueId,
    },
    ConstructStruct {
        nominal: IrNominalId,
        fields: Vec<IrValueId>,
    },
    ConstructEnum {
        nominal: IrNominalId,
        variant: u32,
        fields: Vec<IrValueId>,
    },
    ProjectStruct {
        aggregate: IrValueId,
        nominal: IrNominalId,
        field: u32,
        consume_root: bool,
    },
    InsertStruct {
        aggregate: IrValueId,
        nominal: IrNominalId,
        field: u32,
        value: IrValueId,
    },
    ProjectVariant {
        aggregate: IrValueId,
        nominal: IrNominalId,
        variant: u32,
        field: u32,
    },
    AddressOf {
        value: IrValueId,
        referent: IrAddressed,
    },
    Load {
        address: IrValueId,
        referent: IrAddressed,
    },
    /// One permitted counted loop [PAR-2 candidate], actualized as a recursive
    /// split of its index range.
    ///
    /// The whole loop is one instruction here because it has exactly two
    /// renderings and the choice between them is the world, not the source. The
    /// overlapped world asks the runtime what a split of this span may afford
    /// and calls `splitter`; the sequential world calls `chunk`, which *is* the
    /// loop, so that world runs the code the loop always had. A reduction uses
    /// the first argument and result for its accumulator. An independent map
    /// uses `Unit` in both positions as a synchronization token; its observable
    /// result is the disjoint stores completed before the call returns. The
    /// site therefore has nothing to recombine in either form.
    ///
    /// `splitter` and `chunk` are ordinary synthesized [`IrFunction`]s: the
    /// splitter's two recursive calls are one ordinary overlap group, so the
    /// hand-out, the thunk, the deque, and the join are the ones every other
    /// permitted pair uses.
    LoopSplit {
        splitter: u32,
        chunk: u32,
        /// A reduction's accumulator on entry, or the `Unit` synchronization
        /// token of an independent map. A reduction seed folds into the
        /// leftmost chunk, which keeps its leaf order the source's own.
        seed: IrValueId,
        lower: IrValueId,
        upper: IrValueId,
        /// The values the body reads from the enclosing scope, in the order the
        /// two synthesized functions declare them.
        captures: Vec<IrValueId>,
        /// The static cost estimate of one iteration, which the runtime
        /// allowance multiplies by the span. A cost over the emitted IR, never
        /// a name, a signature, or a source shape.
        weight: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrInstruction {
    Define {
        result: IrValueId,
        ty: IrType,
        operation: IrOperation,
    },
    StoreBuffer {
        buffer: IrValueId,
        index: IrValueId,
        value: IrValueId,
    },
    /// One element-position store through an exclusive view [SET-1,
    /// VIEW-1]. The descriptor is unchanged; the storage written is the
    /// origin's, reached through the view's own data pointer.
    StoreSlice {
        slice: IrValueId,
        index: IrValueId,
        value: IrValueId,
    },
    Store {
        address: IrValueId,
        value: IrValueId,
        referent: IrAddressed,
    },
    Drop(IrDrop),
}

/// One compiler-derived release, explicit on the normal control-flow edge
/// that carries it [STOR-3].
///
/// Every drop and every release is represented before lowering. The IR places
/// these records on `Jump` and `Return` terminators and as `Drop` instructions
/// in straight-line position. Their order inside one edge is the checked
/// program's reverse declaration order, and their position relative to
/// surrounding calls is the order [EFF-5] requires of every conforming
/// lowering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrDrop {
    value: IrValueId,
    ty: IrType,
    release: SystemRelease,
}

impl IrDrop {
    pub const fn value(self) -> IrValueId {
        self.value
    }

    pub const fn ty(self) -> IrType {
        self.ty
    }

    /// The exact [SYS-5] release this drop performs: the released value's own
    /// action when it is one system resource, together with the union of the
    /// rows of every system release it may run over owned content.
    pub const fn release(self) -> SystemRelease {
        self.release
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrMatchTarget {
    tag: u32,
    block: IrBlockId,
}

impl IrMatchTarget {
    pub const fn tag(self) -> u32 {
        self.tag
    }

    pub const fn block(self) -> IrBlockId {
        self.block
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrTerminator {
    /// A checker-proved uninhabited function body. The function keeps its
    /// ordinary ABI but has no source-derived executable path.
    Unreachable,
    Jump {
        target: IrBlockId,
        arguments: Vec<IrValueId>,
        drops: Vec<IrDrop>,
    },
    Match {
        scrutinee: IrValueId,
        enum_type: IrEnumType,
        targets: Vec<IrMatchTarget>,
    },
    Return {
        value: IrValueId,
        drops: Vec<IrDrop>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrBlock {
    parameters: Vec<(IrValueId, IrType)>,
    instructions: Vec<IrInstruction>,
    terminator: IrTerminator,
}

impl IrBlock {
    pub fn parameters(&self) -> &[(IrValueId, IrType)] {
        &self.parameters
    }

    pub fn instructions(&self) -> &[IrInstruction] {
        &self.instructions
    }

    pub const fn terminator(&self) -> &IrTerminator {
        &self.terminator
    }
}

/// Whether lowering actualizes permission-derived overlap schedules.
///
/// The judgment itself is pure and always runs: `--par-ledger` reports the same
/// verdicts either way, and no accepted program changes. This selects only
/// whether a permitted compute group or direct completion schedule reaches the
/// IR, and therefore whether the backend submits or outlines work.
///
/// `Completion` is the shipped default: it actualizes only compiler-owned
/// finite target operations and leaves pure compute output byte-identical to
/// `Off`. Compute outlining remains opt-in because it is not free. The compute
/// audit measured that lowering alone, with no runtime linked and
/// `WF_WORKERS` unset,
/// at about 1.2x on the layout demo and 2.1x on `fib(38)`: an outlined call
/// passes its arguments through a memory frame, is reached through a function
/// pointer, and so cannot be inlined. `Off` remains only the exact sequential
/// reference; `On` adds eligible compute groups to the default completion set.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverlapLowering {
    /// Consult no permission group. Retained for exact sequential-reference
    /// tests; this is not the shipped default.
    Off,
    /// Actualize only direct finite target operations. Pure compute output is
    /// therefore byte-identical to `Off`, while completion I/O needs no flag.
    #[default]
    Completion,
    /// Actualize completion operations and eligible compute groups.
    On,
}

/// One group of pure sibling calls whose evaluations may be overlapped
/// [PAR-1 candidate].
///
/// The members are the values those calls define, in source order, all in one
/// block of one function. The compute scheduler may hand out every member but
/// the last, runs that source-last member on the calling lane, and joins the
/// handed-out calls before any value use or block exit. Direct target
/// operations use [`IrCompletionStep`] instead of this group representation.
///
/// The group is a permission the target stage may take, never an obligation:
/// a target that hands nothing out emits exactly the sequential code, because
/// the handed-out call and the inline fallback call the same monomorphized
/// function on the same arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrOverlap {
    members: Vec<IrValueId>,
}

impl IrOverlap {
    /// Every source member in order, including the source-last join site.
    pub fn members(&self) -> &[IrValueId] {
        &self.members
    }

    /// The value whose definition is the group's join site: the last member,
    /// which runs on the calling thread.
    pub fn join_site(&self) -> Option<IrValueId> {
        self.members.last().copied()
    }

    /// The members that may be handed to a worker lane, in source order.
    pub fn handed_out(&self) -> &[IrValueId] {
        self.members
            .split_last()
            .map_or(&[][..], |(_, earlier)| earlier)
    }
}

/// One source-ordered call step in a direct completion schedule.
///
/// `wait_for` contains only earlier operations whose ordinary result or loan
/// must be returned before this call is reached.  A submitted operation has
/// at least one later statement which the permission judgment proved can run
/// while it is in flight.  This is target-independent scheduling metadata: it
/// names SSA values and carries no resource family or I/O-specific identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrCompletionStep {
    call: IrValueId,
    wait_for: Vec<IrValueId>,
    submit: bool,
    finish: bool,
}

impl IrCompletionStep {
    pub(crate) fn new(
        call: IrValueId,
        wait_for: Vec<IrValueId>,
        submit: bool,
        finish: bool,
    ) -> Self {
        Self {
            call,
            wait_for,
            submit,
            finish,
        }
    }

    pub(crate) const fn call(&self) -> IrValueId {
        self.call
    }

    pub(crate) fn wait_for(&self) -> &[IrValueId] {
        &self.wait_for
    }

    pub(crate) const fn submit(&self) -> bool {
        self.submit
    }

    pub(crate) const fn finish(&self) -> bool {
        self.finish
    }
}

/// The arguments one loop's window query is asked with.
///
/// `wf__completion_window` answers from the runtime's own capacity and from
/// these three, each of which is a bound and none of which is a request. Zero
/// means "this one places no bound": a loop whose trip count is not statically
/// known passes zero for `span`, a loop with no privatized storage passes zero
/// for `slot_bytes`, and a loop the compiler puts no static cap on passes zero
/// for `ceiling`. The writer never spells any of them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrCompletionWindow {
    span: u64,
    slot_bytes: u64,
    ceiling: u64,
}

impl IrCompletionWindow {
    /// Builds the target-independent bounds carried by a staged-loop
    /// descriptor. The lowering that owns the loop supplies these values;
    /// the backend does not infer them.
    pub(crate) const fn new(span: u64, slot_bytes: u64, ceiling: u64) -> Self {
        Self {
            span,
            slot_bytes,
            ceiling,
        }
    }

    pub(crate) const fn span(&self) -> u64 {
        self.span
    }

    pub(crate) const fn slot_bytes(&self) -> u64 {
        self.slot_bytes
    }

    pub(crate) const fn ceiling(&self) -> u64 {
        self.ceiling
    }
}

/// One function's staged loop pipeline, or nothing where the loop judgment
/// grants no such schedule.
///
/// Three things about emission change when a function carries this. The window
/// is asked once at the loop's entry block, never per iteration, exactly as
/// `wf__par_split_budget` is asked. A block named by `carrying` may end with
/// the loop's target operation still outstanding, and the complete driver's
/// exact drain retires it before result use. This distinction is CFG-owned:
/// unrelated blocks that happen to sit between feeder and drain in the linear
/// block vector own neither transition. Each call site's completion storage
/// is a ring of [`Self::slots`] operation records rather than one, addressed
/// through the slot index whichever block reaches it addresses its ring
/// through.
///
/// The ring is what makes a back edge with work in flight *correct* rather
/// than merely admitted. A carrying block is emitted once and reached many
/// times, so the straight-line walk sees one hand-out at a site while the
/// running program has one per iteration in flight: with one storage element
/// the second iteration's submission would hand the target a token, a result
/// slot and a staged path the first iteration's operation is still being read
/// from and written to. The count is static because the storage is an
/// entry-block reservation; which element an operation owns is a run-time
/// choice, and the runtime's window never exceeds the count.
///
/// Production lowering constructs two complete forms. A one-slot feeder has a
/// mandatory drain successor. A fixed two-slot bounded batch issues up to the
/// selected window, drains every issued slot in order, and only then reuses
/// slot zero. In both forms the generated CFG, not backend inference, owns the
/// result edge and every reuse boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrCompletionPipeline {
    /// Checked-tree identity that selected this descriptor.
    source_loop: CheckedLoopId,
    /// The only three states a source-derived descriptor can occupy. A
    /// pending permission is invisible to emission; either driven state owns
    /// the exact feeder, drain, and result cut points it needs.
    driver: IrCompletionDriver,
    entry: IrBlockId,
    carrying: Vec<IrBlockId>,
    window: IrCompletionWindow,
    slots: u64,
    slot_index: Vec<(IrBlockId, IrValueId)>,
    /// The SSA value defined by the entry's compiler-owned window query when
    /// the generated CFG consumes that answer. The depth-one form asks only
    /// for scheduling evidence and therefore leaves this absent.
    window_value: Option<IrValueId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IrCompletionDriver {
    Pending,
    OneSlot(IrCompletionOneSlotDriver),
    BoundedBatch(IrCompletionBatchDriver),
}

impl IrCompletionPipeline {
    /// Records one permitted source loop in ordinary lowering without
    /// actualizing an incomplete schedule.
    ///
    /// This is the identity bridge from the checker's [`CheckedLoopId`] to
    /// target-independent IR. Its empty carrying set is intentional: until
    /// lowering has produced the slot driver and delayed-result SSA, the
    /// backend must emit exactly the ordinary schedule.
    pub(crate) fn pending(
        source_loop: CheckedLoopId,
        entry: IrBlockId,
        window: IrCompletionWindow,
    ) -> Self {
        Self {
            source_loop,
            driver: IrCompletionDriver::Pending,
            entry,
            carrying: Vec::new(),
            window,
            slots: 1,
            slot_index: Vec::new(),
            window_value: None,
        }
    }

    /// Records and activates the already-materialized depth-one feeder/drain
    /// topology.
    pub(crate) fn plan_one_slot(&mut self, feeder: IrBlockId, drain: IrBlockId, result: IrValueId) {
        self.carrying = vec![feeder];
        self.driver = IrCompletionDriver::OneSlot(IrCompletionOneSlotDriver {
            feeder,
            drain,
            result,
        });
        // A depth-one driver has no reuse race: the feeder's only successor
        // is the drain, and the drain joins the operation before dispatching
        // on its result.  There is therefore no outstanding operation at the
        // next submission and no slot index to carry.  Mark this exact shape
        // ready here; wider drivers must still supply explicit slot cycling,
        // per-slot reuse waits, and exit retirement before they may do so.
    }

    /// The one-slot topology lowering materialized before activation.
    pub(crate) const fn planned_driver(&self) -> Option<IrCompletionOneSlotDriver> {
        match self.driver {
            IrCompletionDriver::OneSlot(driver) => Some(driver),
            IrCompletionDriver::Pending | IrCompletionDriver::BoundedBatch(_) => None,
        }
    }

    /// Activates a complete bounded-batch driver whose generated CFG proves
    /// both slot indices are in `0..slots` and drains the whole batch before
    /// slot zero can be reused.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn plan_bounded_batch(
        &mut self,
        carrying: Vec<IrBlockId>,
        slots: u64,
        slot_index: Vec<(IrBlockId, IrValueId)>,
        window_value: IrValueId,
        feeder: IrBlockId,
        drain: IrBlockId,
        result: IrValueId,
    ) {
        self.carrying = carrying;
        self.slots = slots;
        self.slot_index = slot_index;
        self.window_value = Some(window_value);
        self.driver = IrCompletionDriver::BoundedBatch(IrCompletionBatchDriver {
            feeder,
            drain,
            result,
        });
    }

    pub(crate) const fn planned_batch_driver(&self) -> Option<IrCompletionBatchDriver> {
        match self.driver {
            IrCompletionDriver::BoundedBatch(driver) => Some(driver),
            IrCompletionDriver::Pending | IrCompletionDriver::OneSlot(_) => None,
        }
    }

    /// How many operations of one call site the region may have in flight.
    ///
    /// The completion storage of each site — its token, its result slot, its
    /// raw value and error, an open's outcome, a directory cursor's position,
    /// an open's staged component — is this many elements rather than one, and
    /// the element an operation owns is chosen at run time by
    /// [`Self::slot_index`]. It is a static count because the storage is an
    /// entry-block reservation; the runtime's window is what decides how many
    /// of them are ever occupied, and it never exceeds this.
    pub(crate) const fn slots(&self) -> u64 {
        self.slots
    }

    /// The checked loop whose staged permission selected this descriptor.
    pub(crate) const fn source_loop(&self) -> CheckedLoopId {
        self.source_loop
    }

    /// Whether lowering supplied every run-time driver obligation required
    /// before the backend may leave an operation live across an edge.
    pub(crate) const fn driver_ready(&self) -> bool {
        !matches!(self.driver, IrCompletionDriver::Pending)
    }

    /// The value naming the slot the completion storage addressed in this
    /// block belongs to.
    ///
    /// It is a `u64` the driver threads into the region along its edges — in
    /// the ordinary shape the loop-carried parameter of the header, which
    /// dominates every block of the body — so the element pointer each block
    /// materializes from it dominates every use of that pointer. A block with
    /// no entry addresses element zero, which is every block of a one-slot
    /// region and every block outside a ring.
    pub(crate) fn slot_index(&self, block: IrBlockId) -> Option<IrValueId> {
        self.slot_index
            .iter()
            .find(|(named, _)| *named == block)
            .map(|(_, value)| *value)
    }

    /// The block the window is asked in, once per loop entry.
    pub(crate) const fn entry(&self) -> IrBlockId {
        self.entry
    }

    /// Whether this block's terminator may leave operations outstanding.
    pub(crate) fn carries(&self, block: IrBlockId) -> bool {
        self.carrying.contains(&block)
    }

    /// Whether this exact compiler-generated block retires the pipeline's
    /// outstanding operation before consuming its result.
    pub(crate) fn drains(&self, block: IrBlockId) -> bool {
        match self.driver {
            IrCompletionDriver::OneSlot(driver) => driver.drain == block,
            IrCompletionDriver::BoundedBatch(driver) => driver.drain == block,
            IrCompletionDriver::Pending => false,
        }
    }

    /// The delayed SSA result owned by a complete driver.
    pub(crate) const fn driven_result(&self) -> Option<IrValueId> {
        match self.driver {
            IrCompletionDriver::OneSlot(driver) => Some(driver.result),
            IrCompletionDriver::BoundedBatch(driver) => Some(driver.result),
            IrCompletionDriver::Pending => None,
        }
    }

    pub(crate) const fn window(&self) -> IrCompletionWindow {
        self.window
    }

    pub(crate) const fn window_value(&self) -> Option<IrValueId> {
        self.window_value
    }
}

/// One materialized single-slot driver edge.
///
/// `feeder` ends immediately after the submitted operation, `drain` is its
/// only successor and dispatches on `result`, and no next loop submission is
/// reachable without passing through that drain. These identities are IR
/// topology, not source coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct IrCompletionOneSlotDriver {
    feeder: IrBlockId,
    drain: IrBlockId,
    result: IrValueId,
}

impl IrCompletionOneSlotDriver {
    #[cfg(test)]
    pub(crate) const fn feeder(self) -> IrBlockId {
        self.feeder
    }

    #[cfg(test)]
    pub(crate) const fn drain(self) -> IrBlockId {
        self.drain
    }

    pub(crate) const fn result(self) -> IrValueId {
        self.result
    }
}

/// The two semantic cut points of one compiler-generated bounded batch.
///
/// `feeder` submits one operation using its proved issue slot. `drain` joins
/// one operation using its proved retirement slot and defines `result` before
/// dispatching the source match. The generated CFG, rather than the backend,
/// owns iteration order, slot reuse, and the complete-drain boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct IrCompletionBatchDriver {
    feeder: IrBlockId,
    drain: IrBlockId,
    result: IrValueId,
}

impl IrCompletionBatchDriver {
    #[cfg(test)]
    pub(crate) const fn feeder(self) -> IrBlockId {
        self.feeder
    }

    #[cfg(test)]
    pub(crate) const fn drain(self) -> IrBlockId {
        self.drain
    }

    pub(crate) const fn result(self) -> IrValueId {
        self.result
    }
}

/// How large a lane frame a handed-out call is granted, in bytes.
///
/// This restates `WF_PAR_FRAME_BYTES` in `backend/par_runtime.c`, because the
/// decision to emit a [`IrOperation::LoopSplit`] at all has to be made long
/// before a runtime exists — and a split whose frame is over the bound would be
/// refused every lane at run time and sequentialize with no report. The two
/// numbers live in two languages and are pinned to each other by
/// `the_compile_time_frame_bound_is_the_runtimes`.
pub const LANE_FRAME_BYTES: u64 = 256;

/// Why a function exists, for the one consumer that has to tell the two worlds
/// apart: a source function is emitted into both, while the two halves of a
/// [`IrOperation::LoopSplit`] each belong to exactly one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrSynthesis {
    /// The recursive range splitter. It exists only in the overlapped world —
    /// the sequential world calls the chunk directly — so cloning it would emit
    /// a second, unreachable caller of the chunk's clone and cost that clone
    /// the single-call-site inlining the sequential world depends on.
    Splitter,
    /// The loop over a subrange, seeded by its first parameter. Both worlds
    /// reach it, so it is cloned like a source function and each world's copy
    /// has exactly one caller.
    Chunk,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrFunction {
    name: String,
    parameters: Vec<(IrValueId, IrType)>,
    result: IrType,
    values: Vec<IrType>,
    blocks: Vec<IrBlock>,
    overlaps: Vec<IrOverlap>,
    completion_steps: Vec<IrCompletionStep>,
    completion_pipeline: Option<IrCompletionPipeline>,
    synthesis: Option<IrSynthesis>,
    target_action: crate::TargetAction,
}

impl IrFunction {
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Why this function exists, or `None` for a source function.
    pub const fn synthesis(&self) -> Option<IrSynthesis> {
        self.synthesis
    }

    /// Conservative compiler-owned suspension summary.
    pub const fn target_action(&self) -> crate::TargetAction {
        self.target_action
    }

    pub const fn result(&self) -> IrType {
        self.result
    }

    pub fn parameters(&self) -> &[(IrValueId, IrType)] {
        &self.parameters
    }

    pub fn blocks(&self) -> &[IrBlock] {
        &self.blocks
    }

    /// The permission-derived overlap groups of this function's body, in
    /// source order and pairwise disjoint in their members.
    pub fn overlaps(&self) -> &[IrOverlap] {
        &self.overlaps
    }

    /// Direct calls whose ordinary dependencies admit completion submission.
    pub(crate) fn completion_steps(&self) -> &[IrCompletionStep] {
        &self.completion_steps
    }

    /// The staged loop pipeline this function's loop judgment granted, or
    /// `None` where none was.
    pub(crate) const fn completion_pipeline(&self) -> Option<&IrCompletionPipeline> {
        self.completion_pipeline.as_ref()
    }

    /// The staged descriptor the backend may actualize. A permission whose IR
    /// shape could not be driven remains retained but invisible to emission.
    pub(crate) fn driven_completion_pipeline(&self) -> Option<&IrCompletionPipeline> {
        self.completion_pipeline()
            .filter(|pipeline| pipeline.driver_ready())
    }

    pub(crate) fn contains_buffer(&self) -> bool {
        self.values
            .iter()
            .any(|ty| matches!(ty, IrType::Buffer { .. }))
    }

    /// Every defined value's type, for whole-program type enumeration.
    pub(crate) fn value_types(&self) -> &[IrType] {
        &self.values
    }

    pub(crate) fn value_type(&self, value: IrValueId) -> Option<IrType> {
        self.values.get(value.index()).copied()
    }
}

/// The [FN-7] entry form the program starts with [PROG-3].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrEntry {
    /// A `command` entry: program start supplies exactly the standard inputs
    /// these table ordinals select, in this order, and maps the returned
    /// `ExitStatus` to the host process status.
    Command { inputs: Vec<u8> },
}

#[derive(Debug)]
pub struct IrProgram<'classified, 'lexed, 'source> {
    _checked: CheckedProgram<'classified, 'lexed, 'source>,
    nominals: Vec<IrNominal>,
    constants: Vec<IrGlobalConstant>,
    functions: Vec<IrFunction>,
    main: u32,
    entry: IrEntry,
    actualization: Vec<String>,
}

impl IrProgram<'_, '_, '_> {
    pub fn nominals(&self) -> &[IrNominal] {
        &self.nominals
    }

    /// The entry form program start must implement.
    pub const fn entry(&self) -> &IrEntry {
        &self.entry
    }

    pub fn nominal(&self, id: IrNominalId) -> Option<&IrNominal> {
        self.nominals.get(id.index())
    }

    pub fn constants(&self) -> &[IrGlobalConstant] {
        &self.constants
    }

    pub fn constant(&self, id: IrConstantId) -> Option<&IrGlobalConstant> {
        self.constants.get(id.index())
    }

    pub fn functions(&self) -> &[IrFunction] {
        &self.functions
    }

    pub const fn main_ordinal(&self) -> u32 {
        self.main
    }

    /// The non-normative ledger lines this lowering added: one per permitted
    /// counted loop it either actualized or declined to.
    ///
    /// The judgment's own ledger states what [PAR-2] permits and is the same
    /// with or without `--par`. These state what *this* lowering did with a
    /// permission, which is a different fact and exists only where actualization
    /// was asked for. Both are developer output on the caller's channel; neither
    /// participates in acceptance or in any mandatory [DIAG-3] record.
    pub fn actualization_ledger(&self) -> &[String] {
        &self.actualization
    }

    /// Test-only malformed-IR probe: retypes one command parameter while
    /// keeping the function's local value table internally consistent.
    #[cfg(test)]
    pub(crate) fn retype_main_parameter_for_test(&mut self, parameter: usize, ty: IrType) -> bool {
        let Some(main) = self.functions.get_mut(self.main as usize) else {
            return false;
        };
        let Some((value, declared)) = main.parameters.get_mut(parameter) else {
            return false;
        };
        *declared = ty;
        let Some(stored) = main.values.get_mut(value.index()) else {
            return false;
        };
        *stored = ty;
        true
    }

    /// Test-only malformed-IR probe: retypes the command result while leaving
    /// its semantic entry identity unchanged.
    #[cfg(test)]
    pub(crate) fn retype_main_result_for_test(&mut self, ty: IrType) -> bool {
        let Some(main) = self.functions.get_mut(self.main as usize) else {
            return false;
        };
        main.result = ty;
        true
    }

    /// Test-only malformed-IR probe: retypes one argument of the first
    /// system call without changing its semantic operation identity.
    #[cfg(test)]
    pub(crate) fn retype_first_system_argument_for_test(
        &mut self,
        argument: usize,
        ty: IrType,
    ) -> bool {
        for function in &mut self.functions {
            let selected = function.blocks.iter().find_map(|block| {
                block.instructions.iter().find_map(|instruction| {
                    let IrInstruction::Define {
                        operation: IrOperation::SystemCall { arguments, .. },
                        ..
                    } = instruction
                    else {
                        return None;
                    };
                    arguments.get(argument).copied()
                })
            });
            let Some(value) = selected else {
                continue;
            };
            let Some(stored) = function.values.get_mut(value.index()) else {
                return false;
            };
            *stored = ty;
            return true;
        }
        false
    }

    /// Test-only malformed-IR probe: retypes the first system call's result
    /// while preserving the operation identity and local SSA agreement.
    #[cfg(test)]
    pub(crate) fn retype_first_system_result_for_test(&mut self, ty: IrType) -> bool {
        for function in &mut self.functions {
            let selected = function
                .blocks
                .iter()
                .enumerate()
                .find_map(|(block, data)| {
                    data.instructions
                        .iter()
                        .enumerate()
                        .find_map(|(instruction, value)| {
                            let IrInstruction::Define {
                                result,
                                operation: IrOperation::SystemCall { .. },
                                ..
                            } = value
                            else {
                                return None;
                            };
                            Some((block, instruction, *result))
                        })
                });
            let Some((block, instruction, result)) = selected else {
                continue;
            };
            let IrInstruction::Define { ty: declared, .. } =
                &mut function.blocks[block].instructions[instruction]
            else {
                return false;
            };
            *declared = ty;
            let Some(stored) = function.values.get_mut(result.index()) else {
                return false;
            };
            *stored = ty;
            return true;
        }
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoweringFailure {
    InvalidCheckedProgram,
    CounterOverflow,
}

mod builder;

#[cfg(test)]
mod tests;

pub use builder::lower_checked;
