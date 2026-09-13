//! Target-independent lowering from the semantically checked active Whitefoot specification.
//!
//! The private IR records exact value types, nominal construction/projection,
//! direct calls, erased source proofs, and explicit control-flow edges. It performs
//! no source admission, label lookup, exhaustiveness decision, or ownership
//! judgment.

use crate::semantic::{
    CheckedBooleanOperation, CheckedElement, CheckedEnumType, CheckedFlatElement,
    CheckedFloatOperation, CheckedIntegerOperation, CheckedLayoutCeiling, CheckedLayoutMagnitude,
    CheckedNumericType, CheckedProgram, CheckedRuntimeTargetObligations,
    CheckedTargetDomainObligation, CheckedType,
};

mod physical_types;
mod specialize;

/// A closed executable instance's type interpretation. Source region identity
/// has already been checked; only its storage reclamation remains in the IR.
#[derive(Clone, Copy)]
struct TypeLowering<'a> {
    nominals: &'a [IrNominalId],
    elements: &'a [Option<IrElement>],
    releases: &'a [(crate::DeclarationId, crate::semantic::CheckedReleaseClass)],
}

impl TypeLowering<'_> {
    const EMPTY: Self = Self {
        nominals: &[],
        elements: &[],
        releases: &[],
    };

    fn release(
        self,
        region: crate::DeclarationId,
        fallback: crate::semantic::CheckedReleaseClass,
    ) -> crate::semantic::CheckedReleaseClass {
        self.releases
            .iter()
            .find_map(|(candidate, class)| (*candidate == region).then_some(*class))
            .unwrap_or(fallback)
    }
}

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

/// The complete type of an array or run element, interned in its program's type table.
/// Structural nesting uses handles; only nominal edges can close a type graph.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IrElement(u32);

impl IrElement {
    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

/// One [BLK-2] take from a store, in the shape its emission reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrStoreTake {
    /// The address of the `&uniq` provider operand: the take reads the
    /// store's state and writes it back through the same borrow.
    pub store: IrValueId,
    pub count: IrValueId,
    /// The slot's own type, which the target stage lays out to check its
    /// actual size, alignment and stride against the ceilings below [STOR-6].
    pub element: IrType,
    /// [OP-9]'s language ceilings for that element type.
    pub layout_ceiling: IrLayoutCeiling,
    /// The upper bound [OP-9]'s accepted judgment retained for `count`, which
    /// target layout scales by the actual stride [STOR-6].
    pub count_upper_bound: u64,
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
    /// The cell's referent type, laid out by the target stage to check its
    /// actual size and alignment against the ceilings below and against the
    /// store's own [STOR-6].
    pub element: IrType,
    /// [OP-9]'s language ceilings for that referent type.
    pub layout_ceiling: IrLayoutCeiling,
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

/// The content of an [`IrType::Address`]. A typed place may hold inline
/// content, a descriptor, or a handle. Source borrows of descriptors and
/// handles still use their value ABI; a place containing one is distinct
/// from the storage or resource that descriptor or handle denotes.
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
    Buffer {
        element: IrFlatElement,
    },
    Slice {
        element: IrFlatElement,
    },
    /// One `FixedVector<T, n>` [BLK-1]. A frame-resident run is inline
    /// storage in its owner, exactly as a struct is, so a borrow of one is
    /// the address of that storage rather than a copy of the run.
    FixedVector {
        element: IrElement,
        length: u64,
    },
    /// Dense inline array storage reached through a checked borrow or target.
    Array {
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
            Self::Buffer { element } => IrType::Buffer { element },
            Self::Slice { element } => IrType::Slice { element },
            Self::FixedVector { element, length } => IrType::FixedVector { element, length },
            Self::Array { element, length } => IrType::Array { element, length },
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
            IrType::Buffer { element } => Self::Buffer { element },
            IrType::Slice { element } => Self::Slice { element },
            IrType::FixedVector { element, length } => Self::FixedVector { element, length },
            IrType::Array { element, length } => Self::Array { element, length },
            IrType::Vector { element, release } => Self::Vector { element, release },
            IrType::Provider => Self::Provider,
            IrType::Address(_) => return None,
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
        element: IrElement,
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
/// [S20, PROV-1]: instances of one declaration with different regions share
/// an IR nominal when their complete reclamation graphs also agree.
fn erased_nominal(erasure: TypeLowering<'_>, id: crate::NominalId) -> IrNominalId {
    erasure
        .nominals
        .get(id.0 as usize)
        .copied()
        .unwrap_or(IrNominalId(id.0))
}

fn lower_element(
    erasure: TypeLowering<'_>,
    value: CheckedElement,
) -> Result<IrElement, LoweringFailure> {
    erasure
        .elements
        .get(value.index())
        .copied()
        .flatten()
        .ok_or(LoweringFailure::InvalidCheckedProgram)
}

fn lower_flat_element(
    erasure: TypeLowering<'_>,
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

/// Whether a value of this type derives any release work at all [STOR-3].
///
/// This is the single reading of "does dropping this value do something": the
/// target stage asks it to decide whether a drop emits a cleanup. A `None`
/// answer is a malformed nominal
/// reference, which each caller reports in its own vocabulary; no caller may
/// read it as "no release", because unknown must never be silently inert.
pub(crate) fn type_derives_release(
    nominals: &[IrNominal],
    elements: &[IrType],
    ty: IrType,
) -> Option<bool> {
    let nominal_kind = |id: IrNominalId| nominals.get(id.index()).map(IrNominal::kind);
    let mut pending = vec![ty];
    let mut visited = std::collections::HashSet::new();
    while let Some(current) = pending.pop() {
        if !visited.insert(current) {
            continue;
        }
        match current {
            IrType::Buffer { .. } => return Some(true),
            // A run's own backing action is its release class [PROV-6]: a
            // general store's run spends that store's provider capability, and
            // a bump extent's run is reclaimed by its own region reset, which
            // is no action at all. A frame-resident run reclaims none of its
            // own either. Every run still needs a walk when its window holds
            // values that derive one, and [PROV-6] visits those elements
            // before the backing is released [STOR-3, BLK-1].
            IrType::Vector {
                release: IrReleaseClass::General,
                ..
            } => return Some(true),
            IrType::Array { element, .. }
            | IrType::Vector { element, .. }
            | IrType::FixedVector { element, .. } => {
                pending.push(*elements.get(element.index())?);
            }
            IrType::Provider => {}
            // S39 a cell needs a release exactly when its own storage or
            // its referent does: a bump extent's cell whose referent derives
            // nothing needs no walk at all.
            IrType::Nominal(id)
                if matches!(
                    nominal_kind(id),
                    Some(IrNominalKind::Box {
                        release: IrReleaseClass::General,
                        ..
                    })
                ) =>
            {
                return Some(true);
            }
            IrType::Nominal(id)
                if matches!(nominal_kind(id), Some(IrNominalKind::Box { .. })) =>
            {
                let Some(IrNominalKind::Box { referent, .. }) = nominal_kind(id) else {
                    return None;
                };
                pending.push(*referent);
            }
            IrType::Nominal(id) => match nominal_kind(id)? {
                IrNominalKind::Struct { fields } => {
                    pending.extend(fields.iter().map(IrField::ty));
                }
                IrNominalKind::Enum { variants } => {
                    pending.extend(
                        variants
                            .iter()
                            .flat_map(IrVariant::fields)
                            .map(IrField::ty),
                    );
                }
                IrNominalKind::Box { .. }
                // The allocation-list drop is the region's storage
                // release [STOR-3]: walk and free.
                | IrNominalKind::ArenaStorage => {
                    return Some(true);
                }
                // An arena value's storage is released with its region,
                // never by an owner-scope cleanup [STOR-3, STOR-4].
                IrNominalKind::Arena { .. } => {}
                // Ordinary opaque values have empty release [PRE-1].
                IrNominalKind::Opaque => {}
            },
            IrType::Unit
            | IrType::Bool
            | IrType::Integer { .. }
            | IrType::Float { .. }
            // [VIEW-1, PROV-3] a view is loan-bearing: it owns no storage and
            // no element, so nothing of it is ever released.
            | IrType::Slice { .. }
            | IrType::Address(_) => {}
        }
    }
    Some(false)
}

fn lower_type(erasure: TypeLowering<'_>, value: CheckedType) -> Result<IrType, LoweringFailure> {
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
            element: lower_element(erasure, element)?,
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
            element,
            release,
            region,
        } => IrType::Vector {
            release: lower_release_class(erasure.release(region, release)),
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
    /// An ordinary opaque nominal supplied by PRE-1.
    Opaque,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrNominal {
    name: String,
    id: IrNominalId,
    kind: IrNominalKind,
}

impl IrNominal {
    /// The ordinary declaration name retained for debug and link descriptions.
    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn id(&self) -> IrNominalId {
        self.id
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

fn lower_enum_type(erasure: TypeLowering<'_>, value: CheckedEnumType) -> IrEnumType {
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
    Array(Vec<IrGlobalValue>),
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
pub enum IrPlaceProjection {
    /// A field of directly stored nominal content.
    Field { nominal: IrNominalId, field: u32 },
    /// The allocation payload reached through a stored Box owner slot.
    BoxReferent { nominal: IrNominalId },
    /// One payload field of an enum in directly addressed storage.
    EnumVariant {
        nominal: IrNominalId,
        variant: u32,
        field: u32,
    },
    /// An initialized run element selected by its checked logical offset.
    RunElement {
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
    },
    /// An initialized legacy array element, without run descriptor words.
    ArrayElement {
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrOperation {
    Constant(IrConstant),
    Call {
        function: u32,
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
    /// BLK-3's consuming conversion between a full fixed run and its dense
    /// array. Source and result have identical element type and extent.
    FullArrayConversion {
        value: IrValueId,
    },
    /// One discharged source subscript read [OP-4]: the checker has already
    /// derived the bounds obligation, so no runtime branch is emitted in any
    /// build mode; the offset is consumed directly.
    ArrayIndex {
        root: IrArrayRoot,
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
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
    /// [VIEW-2] one view over typed owner storage: a run's initialized window
    /// [BLK-1] or a complete array with its type's length and zero head.
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
    /// Address of immutable program-lifetime storage. It never owns a frame
    /// slot or participates in local destination reuse.
    ConstantAddress {
        constant: IrConstantId,
    },
    /// A typed child place of an already stable owner or borrow. This keeps
    /// the same backing and lifetime; it does not read or copy its content.
    ProjectAddress {
        address: IrValueId,
        projection: IrPlaceProjection,
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
    /// Capture every cleanup subject before running this checked release
    /// sequence. A release must not change a later subject's saved value.
    Drops(Vec<IrDrop>),
}

/// A release consumes either an already captured value or the initialized
/// content at an existing typed place. Naming a cleanup place does not read
/// its entire content into another owned aggregate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrDropSubject {
    Value(IrValueId),
    Place(IrValueId),
}

/// One compiler-derived release, explicit on the normal control-flow edge
/// that carries it [STOR-3].
///
/// Every drop and every release is represented before lowering. The IR places
/// these records on `Jump` and `Return` terminators and as `Drops` instructions
/// in straight-line position. Their order inside one edge is the checked
/// program's reverse declaration order. Their position relative to calls
/// preserves the ordinary scope-exit release sequence [STOR-3].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrDrop {
    subject: IrDropSubject,
    ty: IrType,
}

impl IrDrop {
    pub const fn subject(self) -> IrDropSubject {
        self.subject
    }

    pub const fn operand(self) -> IrValueId {
        match self.subject {
            IrDropSubject::Value(value) | IrDropSubject::Place(value) => value,
        }
    }

    pub const fn ty(self) -> IrType {
        self.ty
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

/// Whether lowering actualizes ordinary permission-derived overlap.
///
/// Every mode runs the same permission judgment and preserves source acceptance.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverlapLowering {
    /// Emit sequential ordinary calls.
    #[default]
    Off,
    /// Outline eligible ordinary calls and counted-loop groups.
    On,
    /// Control over the recursion budget every other `On` form derives from
    /// the runtime: pin its starting value, or emit no budget family at all.
    OnWithRecursionBudget {
        /// Where one call into a recursive component starts counting.
        budget: RecursionBudget,
        /// Optional scalar-leaf offer suppression.
        maximum_scalar_leaf_operations: Option<u32>,
        /// Also select sequential clones on refused compute offers.
        sequential_refusal: bool,
    },
    /// Optional control: an ungranted ordinary call may enter
    /// its existing ordinary-ABI sequential clone at the original join.
    OnWithSequentialRefusal {
        /// Optional suppression of small scalar leaf offers, as in the leaf control.
        maximum_scalar_leaf_operations: Option<u32>,
    },
    /// Retain `On` except for offers of straight-line scalar
    /// leaves with at most this many nonconstant IR operations. This is an
    /// actualization heuristic, not an acceptance bound or machine-cost claim.
    OnWithoutSmallScalarLeaves {
        /// Maximum nonconstant operations in a scalar leaf whose offer is omitted.
        maximum_operations: u32,
    },
}

/// Where one call into an ordinary cyclic call-graph component starts
/// counting the levels that may still hand work out.
///
/// Every member of such a component gets one synthesized variant carrying this
/// count as a hidden trailing parameter; inside the family a call spends one
/// level, and a call made with nothing left enters the component's existing
/// same-ABI sequential clone, the world with no scheduler test in it. The
/// count therefore selects how much of an admitted program is actualized in
/// parallel. It is read by the emitter and by no acceptance path: it is not a
/// timeout, a fuel bound, a proof-work budget or an early failure, it cannot
/// reject a program, and compiling the same source with any of these three
/// forms accepts exactly the same programs and computes exactly the same
/// values.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RecursionBudget {
    /// Emit no budget family: every node of every recursive component offers.
    ///
    /// The control, and what every `--par` build did before the family
    /// existed. On the compute scoreboard's quadrature kernel this is 35 to 40
    /// percent behind the default at every measured width.
    Off,
    /// Ask the runtime once, at the component's ordinary entry. The answer
    /// follows the pool width, which a compile-time constant cannot — and the
    /// measured best fixed value is not the same value at two lanes and at
    /// four, which is why the default is a query rather than a number.
    #[default]
    RuntimeDerived,
    /// Start from this compile-time value instead of asking. The control the
    /// measured depth sweep uses.
    Pinned(std::num::NonZeroU8),
}

/// One group of pure sibling calls whose evaluations may be overlapped
/// [PAR-1 candidate].
///
/// The members are the values those calls define, in source order, all in one
/// block of one function. The compute scheduler may hand out every member but
/// the last, runs that source-last member on the calling lane, and joins the
/// handed-out calls before any value use or block exit.
///
/// The group is a permission the target stage may take, never an obligation:
/// a target that hands nothing out emits exactly the sequential code, because
/// the handed-out call and the default inline fallback call the same
/// monomorphized function on the same arguments. Optional refusal/frontier
/// controls may choose a same-ABI sequential clone that declines descendant
/// offers; the original operations, arguments and join boundary remain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrOverlap {
    members: Vec<IrValueId>,
}

impl IrOverlap {
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

/// How large a lane frame a handed-out call is granted, in bytes.
///
/// This restates `WF_SCHED_FRAME_BYTES` in `backend/sched/core.h`, because the
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

/// A checked source signature's mode, independent of its lowered value type.
///
/// Descriptor and opaque-handle types can have the same representation in all
/// three modes. This record does not carry a loan origin or its lifetime, and
/// cannot by itself authorize aliasing an input and a result destination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrSourceMode {
    /// The source signature passes an owned value.
    Own,
    /// The source signature passes shared access to an existing value.
    Shared,
    /// The source signature passes exclusive access to an existing value.
    Unique,
}

/// Checked source roles retained independently of representation and erased regions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrSourceSignature {
    parameters: Vec<IrSourceMode>,
    result: IrSourceMode,
}

impl IrSourceSignature {
    pub(crate) fn parameters(&self) -> &[IrSourceMode] {
        &self.parameters
    }

    pub(crate) const fn result(&self) -> IrSourceMode {
        self.result
    }
}

/// One source argument's checked use, distinct from its formal passing mode.
///
/// Consuming a unique holder transfers that holder, not ownership of its
/// referent. Combine this record with the callee's source signature; neither
/// the representation type nor a consume flag alone supplies that distinction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrSourceArgument {
    /// A direct binding occurrence; the flag is the owning checker's verdict.
    Binding {
        /// Whether this occurrence consumes the source binding.
        consume_root: bool,
    },
    /// A checked field projection, including its enclosing-root consumption.
    Projection {
        /// Whether this projection consumes its enclosing source binding.
        consume_root: bool,
    },
    /// A borrow or reborrow formed over existing storage.
    Borrow,
    /// Content selected through a place or dereference. This does not claim
    /// that the enclosing owner is consumed or that affine content is copyable.
    PlaceRead,
    /// A literal or another computed value, with no binding-transfer claim.
    Value,
}

/// Source-call use and direct borrow-result relations tied to one IR call.
///
/// The actual arguments and their typed address/projection operations remain
/// on the call. A result's origin is the complete candidate argument; it need
/// not be the exact subplace selected inside the callee. No source offset is
/// reevaluated to produce this metadata, and it adds no executable read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrSourceCall {
    result: IrValueId,
    arguments: Vec<IrSourceArgument>,
    /// A direct borrow result's checked candidate. Absence says nothing about
    /// loans carried inside owned view results or other aggregates.
    returned_borrow_argument: Option<usize>,
}

impl IrSourceCall {
    pub(crate) const fn result(&self) -> IrValueId {
        self.result
    }

    pub(crate) fn arguments(&self) -> &[IrSourceArgument] {
        &self.arguments
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrFunction {
    name: String,
    parameters: Vec<(IrValueId, IrType)>,
    /// Checked source modes, or `None` for a compiler-synthesized function.
    /// Internal transfer contracts must not be invented from representation.
    source_signature: Option<IrSourceSignature>,
    /// Only calls lowered from checked source; synthesized calls do not
    /// acquire invented source use or provenance records.
    source_calls: Vec<IrSourceCall>,
    result: IrType,
    values: Vec<IrType>,
    blocks: Vec<IrBlock>,
    overlaps: Vec<IrOverlap>,
    synthesis: Option<IrSynthesis>,
}

impl IrFunction {
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Why this function exists, or `None` for a source function.
    pub const fn synthesis(&self) -> Option<IrSynthesis> {
        self.synthesis
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

    pub(crate) const fn source_signature(&self) -> Option<&IrSourceSignature> {
        self.source_signature.as_ref()
    }

    pub(crate) fn source_calls(&self) -> &[IrSourceCall] {
        &self.source_calls
    }

    /// The permission-derived overlap groups of this function's body, in
    /// source order and pairwise disjoint in their members.
    pub fn overlaps(&self) -> &[IrOverlap] {
        &self.overlaps
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

#[derive(Debug)]
pub struct IrProgram<'classified, 'lexed, 'source> {
    _checked: CheckedProgram<'classified, 'lexed, 'source>,
    nominals: Vec<IrNominal>,
    elements: Vec<IrType>,
    constants: Vec<IrGlobalConstant>,
    functions: Vec<IrFunction>,
    actualization: Vec<String>,
    sequential_compute_refusal: bool,
    recursion_budget: Option<RecursionBudget>,
}

impl IrProgram<'_, '_, '_> {
    /// How a compute-actualizing lowering fixes the recursion budget, or
    /// `None` where this lowering actualizes no compute at all. Selects
    /// emitted machine code; never an acceptance bound.
    pub(crate) const fn recursion_budget(&self) -> Option<RecursionBudget> {
        self.recursion_budget
    }

    /// Opt-in machine-code selection after a refused compute acquisition.
    pub(crate) const fn sequential_compute_refusal(&self) -> bool {
        self.sequential_compute_refusal
    }

    pub fn nominals(&self) -> &[IrNominal] {
        &self.nominals
    }

    pub fn elements(&self) -> &[IrType] {
        &self.elements
    }

    pub fn element(&self, element: IrElement) -> Option<IrType> {
        self.elements.get(element.index()).copied()
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
