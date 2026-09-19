//! The four compiler-owned storage nominals [TYPE-9] and their [TYPE-6]
//! lookup classes.
//!
//! [TYPE-9] states that `Array`, `Slots`, `Ring` and `Box` each contribute one
//! nominal-type entry and one constructor entry of the same spelling. They are
//! specification data rather than source records: no source construct
//! declares, redeclares, extends, or overrides one, and a source declaration
//! whose spelling equals an entry's in the same domain is the ordinary
//! [DIAG-1] collision.
//!
//! The constructor entry exists to be refused. [TYPE-9] makes a constructor
//! `call` naming one of the four a hard error with the restructuring `build it
//! with a construction function [OP-13]`, and the entry is what makes that
//! refusal a judgment over a resolved declaration rather than a name
//! comparison in the checker.
//!
//! [BLK-0]'s kernel declaration domain is gone in v0.60: the construction
//! functions [OP-13] and the window operations [OP-10] are ordinary [PRE-1]
//! records parsed by the same grammar as source declarations, so this module
//! carries no operation table any more.

use super::DeclarationClass;

/// Dense identity of one [TYPE-9] compiler-owned storage nominal.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContainerNominalId(u8);

impl ContainerNominalId {
    pub(crate) const fn new(ordinal: u8) -> Self {
        Self(ordinal)
    }

    /// Returns the zero-based index into [`CONTAINER_NOMINALS`].
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        self.0
    }
}

/// One [TYPE-9] compiler-owned nominal spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContainerNominal {
    /// Exact TYPEID spelling.
    pub spelling: &'static str,
    /// Which of the four this row is.
    pub shape: ContainerShape,
}

/// The four compiler-owned nominal shapes [TYPE-9].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContainerShape {
    /// `Array<T, N>` and `Array<T>`: every slot always holds a value and
    /// `a.len == a.cap` [WIN-1].
    Array,
    /// `Slots<T, N>` and `Slots<T>`: a window whose filled prefix is `r.len`
    /// [WIN-1].
    Slots,
    /// `Ring<T, N>` and `Ring<T>`: a window whose logical origin is `r.head`
    /// [WIN-1, MSR-1].
    Ring,
    /// `Box<T>`: one heap object of any nameable T, carrying no brand and no
    /// measure at all, a cell being never empty [TYPE-9].
    Box,
}

/// The four storage nominals, in [TYPE-9] order.
pub const CONTAINER_NOMINALS: [ContainerNominal; 4] = [
    ContainerNominal {
        spelling: "Array",
        shape: ContainerShape::Array,
    },
    ContainerNominal {
        spelling: "Slots",
        shape: ContainerShape::Slots,
    },
    ContainerNominal {
        spelling: "Ring",
        shape: ContainerShape::Ring,
    },
    ContainerNominal {
        spelling: "Box",
        shape: ContainerShape::Box,
    },
];

/// The nominal record one resolved storage target names.
#[must_use]
pub fn container_nominal(id: ContainerNominalId) -> Option<&'static ContainerNominal> {
    CONTAINER_NOMINALS.get(usize::from(id.ordinal()))
}

/// The lookup classes of every storage nominal: one entry of the nominal-type
/// TYPEID domain and one of the constructor TYPEID domain [TYPE-6, TYPE-9],
/// exactly as a source `struct_decl` contributes both.
pub const CONTAINER_NOMINAL_CLASSES: [DeclarationClass; 2] = [
    DeclarationClass::NominalType,
    DeclarationClass::StructConstructor,
];

/// The nominal-type class of a storage nominal, which every `type` position
/// admits [TYPE-6].
pub const CONTAINER_NOMINAL_CLASS: DeclarationClass = DeclarationClass::NominalType;

#[cfg(test)]
mod tests {
    use super::CONTAINER_NOMINALS;

    /// [FORM-3]: every storage nominal is TYPEID-shaped, so the four rows
    /// occupy the nominal-type and constructor domains and take no lexical
    /// IDENT spelling away from a writer's declarations.
    #[test]
    fn every_storage_nominal_is_a_typeid() {
        for nominal in CONTAINER_NOMINALS {
            assert!(
                crate::syntax::terminal::is_type_identifier(nominal.spelling.as_bytes()),
                "{} is not a TYPEID spelling",
                nominal.spelling
            );
        }
    }

    /// [TYPE-6]: spellings are unique within each domain.
    #[test]
    fn storage_nominal_spellings_are_unique() {
        let mut nominals: Vec<_> = CONTAINER_NOMINALS
            .iter()
            .map(|nominal| nominal.spelling)
            .collect();
        nominals.sort_unstable();
        let count = nominals.len();
        nominals.dedup();
        assert_eq!(nominals.len(), count);
    }

    /// [TYPE-9]: the four nominals are exactly the shapes the rule names, read
    /// out of its own text rather than pinned by hand.
    #[test]
    fn storage_nominals_match_the_active_specification() {
        let body = crate::ACTIVE_KERNEL_SPEC_TEXT
            .split_once("[TYPE-9] Three storage shapes, two placements each, and one cell.")
            .expect("exact TYPE-9 opening")
            .1
            .split_once("\n\n")
            .expect("exact TYPE-9 body")
            .0;
        for nominal in CONTAINER_NOMINALS {
            assert!(
                body.contains(&format!("`{}<", nominal.spelling)),
                "TYPE-9 does not name {}",
                nominal.spelling
            );
        }
    }
}
