//! Candidate record charges and host-address feasibility checks.

use std::alloc::Layout;

/// A record family named by the version-1 storage model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum RecordFamily {
    /// One retained source record.
    SourceFile = 0,
    /// One entry in duplicate-path ordering scratch.
    DuplicatePathOrder = 1,
    /// One lossless lexical partition member.
    Lexeme = 2,
    /// One source-local lexical boundary offset.
    SourceBoundary = 3,
    /// One token with its complete terminal membership.
    ClassifiedToken = 4,
    /// One pending parser task.
    ParserTask = 5,
    /// One active parser production frame.
    ParserFrame = 6,
    /// One private postorder derivation element.
    DerivationElement = 7,
    /// One finalizer root record.
    FinalizerRoot = 8,
    /// One finalizer shape-validation task.
    ShapeTask = 9,
    /// One finalized production node.
    NodeRecord = 10,
    /// One C-01 ordered child-or-terminal element.
    MixedElement = 11,
    /// One finalized terminal record.
    TerminalRecord = 12,
    /// One source extent in the flattened program root.
    BundleSourceExtent = 13,
    /// One canonical-source gap record.
    CanonicalGap = 14,
    /// One child ordinal in a diagnostic node path.
    NodePathComponent = 15,
    /// One future resolver declaration record.
    Declaration = 16,
    /// One future resolver scope record.
    Scope = 17,
    /// One future resolver declaration event.
    DeclarationEvent = 18,
    /// One future immediately resolved lexical use.
    LexicalUse = 19,
    /// One future deferred use.
    DeferredUse = 20,
    /// One future retained lookup entry.
    LookupEntry = 21,
    /// One future four-sort scratch entry.
    OrderingScratch = 22,
    /// One future resolution coverage record.
    CoverageRecord = 23,
    /// One future diagnostic issue element.
    DiagnosticIssueElement = 24,
}

impl RecordFamily {
    /// Every family in the canonical ledger order.
    pub const ALL: [Self; 25] = [
        Self::SourceFile,
        Self::DuplicatePathOrder,
        Self::Lexeme,
        Self::SourceBoundary,
        Self::ClassifiedToken,
        Self::ParserTask,
        Self::ParserFrame,
        Self::DerivationElement,
        Self::FinalizerRoot,
        Self::ShapeTask,
        Self::NodeRecord,
        Self::MixedElement,
        Self::TerminalRecord,
        Self::BundleSourceExtent,
        Self::CanonicalGap,
        Self::NodePathComponent,
        Self::Declaration,
        Self::Scope,
        Self::DeclarationEvent,
        Self::LexicalUse,
        Self::DeferredUse,
        Self::LookupEntry,
        Self::OrderingScratch,
        Self::CoverageRecord,
        Self::DiagnosticIssueElement,
    ];

    /// Returns the stable ledger ordinal.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        self as usize
    }

    /// Returns the storage-model record name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        RECORD_CHARGES[self.ordinal()].name
    }

    /// Returns the candidate charge for this family.
    #[must_use]
    pub const fn charge(self) -> RecordCharge {
        RECORD_CHARGES[self.ordinal()]
    }
}

/// One conservative capacity charge from `STORAGE-MODEL.md`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordCharge {
    /// Stable human-readable family name.
    pub name: &'static str,
    /// Bytes charged for each capacity element.
    pub stride_bytes: u64,
    /// Maximum permitted alignment for the eventual production record.
    pub maximum_alignment: u64,
    /// Whether the production record is not implemented yet.
    pub future_record: bool,
}

/// Number of record families in each canonical peak row.
pub const RECORD_FAMILY_COUNT: usize = RecordFamily::ALL.len();

/// The exact candidate charge table from `STORAGE-MODEL.md`.
pub const RECORD_CHARGES: [RecordCharge; RECORD_FAMILY_COUNT] = [
    charge("SourceFile record", 64, 16, false),
    charge("duplicate-path order entry", 8, 8, false),
    charge("Lexeme", 64, 16, false),
    charge("source-boundary offset", 8, 8, false),
    charge("ClassifiedToken", 96, 16, false),
    charge("parser task", 32, 16, false),
    charge("parser frame", 64, 16, false),
    charge("DerivationElement", 64, 16, false),
    charge("finalizer root", 96, 16, false),
    charge("shape task", 16, 8, false),
    charge("NodeRecord", 128, 16, false),
    charge("MixedElement", 16, 8, true),
    charge("TerminalRecord", 32, 8, false),
    charge("BundleSourceExtent", 24, 8, false),
    charge("canonical gap", 32, 8, false),
    charge("NodePath component", 4, 4, false),
    charge("Declaration", 128, 16, true),
    charge("Scope", 64, 8, true),
    charge("DeclarationEvent", 64, 8, true),
    charge("LexicalUse", 64, 8, true),
    charge("DeferredUse", 64, 8, true),
    charge("LookupEntry", 128, 16, true),
    charge("OrderingScratch entry", 128, 16, true),
    charge("CoverageRecord", 32, 8, true),
    charge("DiagnosticIssueElement", 128, 16, true),
];

const fn charge(
    name: &'static str,
    stride_bytes: u64,
    maximum_alignment: u64,
    future_record: bool,
) -> RecordCharge {
    RecordCharge {
        name,
        stride_bytes,
        maximum_alignment,
        future_record,
    }
}

const _: () = {
    let mut index = 0;
    while index < RECORD_CHARGES.len() {
        let item = RECORD_CHARGES[index];
        assert!(item.stride_bytes > 0);
        assert!(item.maximum_alignment.is_power_of_two());
        assert!(item.stride_bytes.is_multiple_of(item.maximum_alignment));
        index += 1;
    }
};

#[repr(C, align(4))]
struct Slot4([u8; 4]);
#[repr(C, align(8))]
struct Slot8([u8; 8]);
#[repr(C, align(8))]
struct Slot16A8([u8; 16]);
#[repr(C, align(16))]
struct Slot32A16([u8; 32]);
#[repr(C, align(8))]
struct Slot24A8([u8; 24]);
#[repr(C, align(8))]
struct Slot32A8([u8; 32]);
#[repr(C, align(8))]
struct Slot64A8([u8; 64]);
#[repr(C, align(16))]
struct Slot64A16([u8; 64]);
#[repr(C, align(16))]
struct Slot96A16([u8; 96]);
#[repr(C, align(16))]
struct Slot128A16([u8; 128]);

/// A checked host allocation layout for one charged capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AllocationLayout {
    /// Requested element count.
    pub count: u64,
    /// Candidate charged bytes, before allocator overhead.
    pub charged_bytes: u64,
    /// Host layout size.
    pub host_size: usize,
    /// Host layout alignment.
    pub host_alignment: usize,
    /// Whether the storage order permits an allocator call.
    pub allocation_required: bool,
}

/// Why a candidate capacity cannot be represented by the supported host class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeasibilityError {
    /// `count * stride` exceeded the portable `u64` accounting domain.
    ChargeOverflow,
    /// The count or charged byte length does not fit the host `usize` domain.
    UsizeExceeded,
    /// The charged byte length exceeds Rust's `isize::MAX` allocation bound.
    IsizeExceeded,
    /// `Layout::array` rejected the count and aligned charged slot.
    LayoutRejected,
    /// The static charge table and its aligned witness slot disagree.
    ChargeDefinitionMismatch,
}

/// Checks one candidate record capacity without allocating it.
pub fn record_layout(
    family: RecordFamily,
    count: u64,
) -> Result<AllocationLayout, FeasibilityError> {
    let charge = family.charge();
    let charged_bytes = count
        .checked_mul(charge.stride_bytes)
        .ok_or(FeasibilityError::ChargeOverflow)?;
    let host_count = usize::try_from(count).map_err(|_| FeasibilityError::UsizeExceeded)?;
    let _host_bytes =
        usize::try_from(charged_bytes).map_err(|_| FeasibilityError::UsizeExceeded)?;
    if charged_bytes > isize::MAX as u64 {
        return Err(FeasibilityError::IsizeExceeded);
    }

    let layout = layout_for_charge(charge, host_count)?;
    if u64::try_from(layout.size()) != Ok(charged_bytes)
        || u64::try_from(layout.align()).ok() > Some(charge.maximum_alignment)
    {
        return Err(FeasibilityError::ChargeDefinitionMismatch);
    }
    Ok(AllocationLayout {
        count,
        charged_bytes,
        host_size: layout.size(),
        host_alignment: layout.align(),
        allocation_required: count != 0,
    })
}

/// Checks an exact byte-array request without allocating it.
pub fn exact_byte_layout(byte_count: u64) -> Result<AllocationLayout, FeasibilityError> {
    let host_count = usize::try_from(byte_count).map_err(|_| FeasibilityError::UsizeExceeded)?;
    if byte_count > isize::MAX as u64 {
        return Err(FeasibilityError::IsizeExceeded);
    }
    let layout = Layout::array::<u8>(host_count).map_err(|_| FeasibilityError::LayoutRejected)?;
    if u64::try_from(layout.size()) != Ok(byte_count) {
        return Err(FeasibilityError::ChargeDefinitionMismatch);
    }
    Ok(AllocationLayout {
        count: byte_count,
        charged_bytes: byte_count,
        host_size: layout.size(),
        host_alignment: layout.align(),
        allocation_required: byte_count != 0,
    })
}

fn layout_for_charge(charge: RecordCharge, count: usize) -> Result<Layout, FeasibilityError> {
    let layout = match (charge.stride_bytes, charge.maximum_alignment) {
        (4, 4) => Layout::array::<Slot4>(count),
        (8, 8) => Layout::array::<Slot8>(count),
        (16, 8) => Layout::array::<Slot16A8>(count),
        (24, 8) => Layout::array::<Slot24A8>(count),
        (32, 8) => Layout::array::<Slot32A8>(count),
        (32, 16) => Layout::array::<Slot32A16>(count),
        (64, 8) => Layout::array::<Slot64A8>(count),
        (64, 16) => Layout::array::<Slot64A16>(count),
        (96, 16) => Layout::array::<Slot96A16>(count),
        (128, 16) => Layout::array::<Slot128A16>(count),
        _ => return Err(FeasibilityError::ChargeDefinitionMismatch),
    };
    layout.map_err(|_| FeasibilityError::LayoutRejected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_record_charge_has_an_exact_layout_witness() {
        for family in RecordFamily::ALL {
            let layout = record_layout(family, 3);
            assert!(layout.is_ok(), "{}: {layout:?}", family.name());
        }
    }

    #[test]
    fn zero_capacity_never_requires_allocation() {
        for family in RecordFamily::ALL {
            let layout = record_layout(family, 0);
            assert_eq!(layout.map(|value| value.allocation_required), Ok(false));
        }
        assert_eq!(
            exact_byte_layout(0).map(|value| value.allocation_required),
            Ok(false)
        );
    }

    #[test]
    fn isize_boundary_fails_before_layout_creation() {
        let charge = RecordFamily::SourceFile.charge().stride_bytes;
        let first_excess = (isize::MAX as u64 / charge) + 1;
        assert_eq!(
            record_layout(RecordFamily::SourceFile, first_excess),
            Err(FeasibilityError::IsizeExceeded)
        );
        assert_eq!(
            exact_byte_layout(isize::MAX as u64 + 1),
            Err(FeasibilityError::IsizeExceeded)
        );
    }
}
