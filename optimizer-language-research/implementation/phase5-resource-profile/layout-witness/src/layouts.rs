//! Exact checks for frontend records exposed by the active compiler crates.

use core::mem::{align_of, size_of};

use whitefoot_contract::SourceFile;
use whitefoot_lexer::Lexeme;
use whitefoot_syntax::{BundleSourceExtent, ClassifiedToken};

use crate::charges::RecordFamily;

const fn fits<T>(family: RecordFamily) -> bool {
    let charge = family.charge();
    let size = size_of::<T>() as u64;
    let alignment = align_of::<T>() as u64;
    size <= charge.stride_bytes
        && alignment <= charge.maximum_alignment
        && charge.stride_bytes.is_multiple_of(alignment)
}

const _: () = assert!(fits::<SourceFile>(RecordFamily::SourceFile));
const _: () = assert!(fits::<usize>(RecordFamily::DuplicatePathOrder));
const _: () = assert!(fits::<Lexeme<'static>>(RecordFamily::Lexeme));
const _: () = assert!(fits::<usize>(RecordFamily::SourceBoundary));
const _: () = assert!(fits::<ClassifiedToken<'static>>(
    RecordFamily::ClassifiedToken
));
const _: () = assert!(fits::<BundleSourceExtent>(RecordFamily::BundleSourceExtent));
const _: () = assert!(fits::<u32>(RecordFamily::NodePathComponent));

/// An exact runtime observation of one compiled production type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObservedLayout {
    /// Exact Rust type or representation role checked by the witness.
    pub production_type: &'static str,
    /// Candidate record family against which it was checked.
    pub family: RecordFamily,
    /// Runtime `size_of` result for this binary.
    pub size_bytes: u64,
    /// Runtime `align_of` result for this binary.
    pub alignment_bytes: u64,
}

impl ObservedLayout {
    fn of<T>(production_type: &'static str, family: RecordFamily) -> Self {
        Self {
            production_type,
            family,
            size_bytes: size_of::<T>() as u64,
            alignment_bytes: align_of::<T>() as u64,
        }
    }

    /// Repeats the compile-time stride and alignment check at runtime.
    #[must_use]
    pub fn fits_charge(self) -> bool {
        let charge = self.family.charge();
        self.size_bytes <= charge.stride_bytes
            && self.alignment_bytes <= charge.maximum_alignment
            && charge.stride_bytes.is_multiple_of(self.alignment_bytes)
    }
}

/// Returns every exact active-frontend layout accessible across crate boundaries.
#[must_use]
pub fn observed_public_layouts() -> [ObservedLayout; 7] {
    [
        ObservedLayout::of::<SourceFile>(
            "whitefoot_contract::SourceFile",
            RecordFamily::SourceFile,
        ),
        ObservedLayout::of::<usize>(
            "usize duplicate-path order entry",
            RecordFamily::DuplicatePathOrder,
        ),
        ObservedLayout::of::<Lexeme<'static>>("whitefoot_lexer::Lexeme", RecordFamily::Lexeme),
        ObservedLayout::of::<usize>("usize source-boundary offset", RecordFamily::SourceBoundary),
        ObservedLayout::of::<ClassifiedToken<'static>>(
            "whitefoot_syntax::ClassifiedToken",
            RecordFamily::ClassifiedToken,
        ),
        ObservedLayout::of::<BundleSourceExtent>(
            "whitefoot_syntax::BundleSourceExtent",
            RecordFamily::BundleSourceExtent,
        ),
        ObservedLayout::of::<u32>("u32 NodePath component", RecordFamily::NodePathComponent),
    ]
}

/// Active frontend records whose Rust visibility prevents this external witness
/// from naming the production type.
pub const PRIVATE_FRONTEND_RECORDS_REQUIRING_IN_CRATE_ASSERTIONS: [(&str, RecordFamily); 8] = [
    ("whitefoot_syntax::parser::Task", RecordFamily::ParserTask),
    ("whitefoot_syntax::parser::Frame", RecordFamily::ParserFrame),
    (
        "whitefoot_syntax::parser::DerivationElement",
        RecordFamily::DerivationElement,
    ),
    (
        "whitefoot_syntax::parser::finalize::Completed",
        RecordFamily::FinalizerRoot,
    ),
    (
        "whitefoot_syntax::parser::finalize::ShapeTask",
        RecordFamily::ShapeTask,
    ),
    (
        "whitefoot_syntax::parser::finalize::NodeRecord",
        RecordFamily::NodeRecord,
    ),
    (
        "whitefoot_syntax::parser::finalize::TerminalRecord",
        RecordFamily::TerminalRecord,
    ),
    (
        "whitefoot_syntax::parser::finalize::GapStyle",
        RecordFamily::CanonicalGap,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_layouts_repeat_compile_time_checks() {
        for layout in observed_public_layouts() {
            assert!(layout.fits_charge(), "{layout:?}");
        }
    }

    #[test]
    fn inaccessible_types_are_not_misreported_as_measured() {
        for (_, family) in PRIVATE_FRONTEND_RECORDS_REQUIRING_IN_CRATE_ASSERTIONS {
            assert!(
                !observed_public_layouts()
                    .iter()
                    .any(|layout| layout.family == family)
            );
        }
    }
}
