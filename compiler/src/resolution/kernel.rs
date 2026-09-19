//! The three compiler-owned storage nominals [TYPE-9], the cell identity they
//! share a reader with, and their [TYPE-6] lookup classes.
//!
//! [TYPE-9] states that `Array`, `Slots` and `Ring` each contribute one
//! nominal-type entry and one constructor entry of the same spelling. They are
//! specification data rather than source records: no source construct
//! declares, redeclares, extends, or overrides one, and a source declaration
//! whose spelling equals an entry's in the same domain is the ordinary
//! [DIAG-1] collision.
//!
//! The constructor entry exists to be refused. [TYPE-9] makes a constructor
//! `call` naming one of the three a hard error with the restructuring `build it
//! with a construction function [OP-13]`, and the entry is what makes that
//! refusal a judgment over a resolved declaration rather than a name
//! comparison in the checker.
//!
//! The cell `Box` is not one of them any more. [TYPE-2] and [PRE-1] make it
//! the prelude's own `opaque struct Box<T> { inner: T; }`, declared and parsed
//! like every other prelude record, so it contributes its nominal, its refused
//! constructor and its field through the ordinary declaration path. What
//! survives here is its *identity*: a written `Box` type still names one
//! compiler-owned shape rather than a source struct, so resolution maps that
//! one prelude declaration onto [`CELL_NOMINAL_ID`] and every later stage
//! reads the cell exactly where it read it before.
//!
//! [BLK-0]'s kernel declaration domain is gone in v0.60: the construction
//! functions [OP-13] and the window operations [OP-10] are ordinary [PRE-1]
//! records parsed by the same grammar as source declarations, so this module
//! carries no operation table any more.

use super::DeclarationClass;

/// Dense identity of one compiler-owned container: a [TYPE-9] storage nominal
/// or the [TYPE-2] cell.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContainerNominalId(u8);

impl ContainerNominalId {
    pub(crate) const fn new(ordinal: u8) -> Self {
        Self(ordinal)
    }

    /// Returns the zero-based index into [`CONTAINER_NOMINALS`], or
    /// [`CELL_NOMINAL_ID`]'s ordinal, which is one past its end.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        self.0
    }
}

/// One compiler-owned nominal spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContainerNominal {
    /// Exact TYPEID spelling.
    pub spelling: &'static str,
    /// Which of the four this row is.
    pub shape: ContainerShape,
}

/// The three [TYPE-9] storage shapes and the [TYPE-2] cell.
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
    /// measure at all, a cell being never empty [TYPE-9]. Its declaration is
    /// the prelude's opaque struct [TYPE-2, PRE-1], not a row below.
    Box,
}

/// The three storage nominals, in [TYPE-9] order. The cell is not one of them:
/// it is declared by [PRE-1] and read through [`CELL_NOMINAL`].
pub const CONTAINER_NOMINALS: [ContainerNominal; 3] = [
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
];

/// The cell's compiler-owned identity [TYPE-2, TYPE-9].
///
/// `Box` is declared by [PRE-1] rather than by a row of [`CONTAINER_NOMINALS`],
/// so nothing iterating that table offers it a nominal-type or constructor
/// entry of its own; what this record supplies is the identity every stage
/// after resolution reads a written `Box` through.
pub const CELL_NOMINAL: ContainerNominal = ContainerNominal {
    spelling: "Box",
    shape: ContainerShape::Box,
};

/// The identity resolution gives a use that names [PRE-1]'s cell declaration.
pub const CELL_NOMINAL_ID: ContainerNominalId =
    ContainerNominalId::new(CONTAINER_NOMINALS.len() as u8);

/// The nominal record one resolved container target names: a [TYPE-9] storage
/// shape, or the [TYPE-2] cell at the ordinal one past them.
#[must_use]
pub fn container_nominal(id: ContainerNominalId) -> Option<&'static ContainerNominal> {
    if id.ordinal() == CELL_NOMINAL_ID.ordinal() {
        return Some(&CELL_NOMINAL);
    }
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
    use super::{CELL_NOMINAL, CELL_NOMINAL_ID, CONTAINER_NOMINALS, container_nominal};

    /// [FORM-3]: every storage nominal is TYPEID-shaped, so the three rows
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

    /// [TYPE-9]: the three nominals are exactly the shapes the rule names,
    /// read out of its own text rather than pinned by hand. The sentence that
    /// names them also states the count, so a shape entering or leaving the
    /// table is read here and not counted by hand.
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
        assert_eq!(CONTAINER_NOMINALS.len(), 3);
    }

    /// [TYPE-2, PRE-1]: the cell is declared by the prelude, so it takes no
    /// row of the storage table, and its identity is readable at the ordinal
    /// one past that table's end.
    #[test]
    fn the_cell_is_outside_the_storage_table_and_still_readable() {
        assert!(
            !CONTAINER_NOMINALS
                .iter()
                .any(|nominal| nominal.spelling == CELL_NOMINAL.spelling)
        );
        assert_eq!(container_nominal(CELL_NOMINAL_ID), Some(&CELL_NOMINAL));
        assert!(crate::ACTIVE_KERNEL_SPEC_TEXT.contains(
            "`Box<T>` is the prelude's opaque struct `opaque struct Box<T> { inner: T; }`"
        ));
    }
}
