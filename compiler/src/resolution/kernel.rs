//! The [BLK-0] kernel declaration domain and the [TYPE-2] container and
//! provider nominals it operates over.
//!
//! [BLK-0] states that the container and store operations are one
//! compiler-owned generic declaration domain, admitted to every compilation
//! unit on exactly [SYS-3]'s terms. Like the system domain it is data of the
//! specification rather than a source record: no source construct declares,
//! redeclares, extends, or overrides an entry, and a source declaration whose
//! spelling equals an entry's in the same domain is the ordinary [DIAG-1]
//! collision.
//!
//! Two tables live here because they are two [TYPE-6] domains. The four
//! container and provider nominals are entries of the nominal-type TYPEID
//! domain and are contributed by [TYPE-2]; the nine operations are entries of
//! the lexical IDENT domain and are contributed by [BLK-0]. The
//! `container_declaration_ordinal` a diagnostic origin carries is the second
//! table's own index, which is [BLK-2]'s rows followed by [BLK-3]'s.

use super::DeclarationClass;

/// Dense identity of one [TYPE-2] container or provider nominal.
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

/// Dense identity of one [BLK-0] kernel-domain operation, in the preorder of
/// [BLK-2]'s rows followed by [BLK-3]'s.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KernelOperationId(u8);

impl KernelOperationId {
    pub(crate) const fn new(ordinal: u8) -> Self {
        Self(ordinal)
    }

    /// Returns the zero-based `container_declaration_ordinal` [BLK-0].
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        self.0
    }
}

/// One [TYPE-2] compiler-owned nominal spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContainerNominal {
    /// Exact TYPEID spelling.
    pub spelling: &'static str,
    /// Which of the four this row is.
    pub shape: ContainerShape,
}

/// The four compiler-owned nominal shapes [TYPE-2].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContainerShape {
    /// `Vector<'s, T>`: a store-resident run [BLK-1].
    Vector,
    /// `FixedVector<T, n>`: a frame-resident run [BLK-1].
    FixedVector,
    /// `Heap<'s>`: the general store's provider [PROV-1].
    Heap,
    /// `Arena<'s, bytes, align>`: a bump extent's provider [PROV-1].
    Arena,
}

/// The four container and provider nominals, in [TYPE-2] order.
pub const CONTAINER_NOMINALS: [ContainerNominal; 4] = [
    ContainerNominal {
        spelling: "Vector",
        shape: ContainerShape::Vector,
    },
    ContainerNominal {
        spelling: "FixedVector",
        shape: ContainerShape::FixedVector,
    },
    ContainerNominal {
        spelling: "Heap",
        shape: ContainerShape::Heap,
    },
    ContainerNominal {
        spelling: "Arena",
        shape: ContainerShape::Arena,
    },
];

/// Which row of the inventory one kernel-domain operation is.
///
/// The checker selects behaviour from this discriminant rather than from the
/// spelling, so the record is what the compiler reads and the name is only how
/// a writer reaches it [BLK-0].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KernelRow {
    /// `seq_fixed<T, const n: u64>() -> own FixedVector<T, n>` [BLK-2].
    SeqFixed,
    /// `seq_arena<T, const bytes, const align>['s](arena, count)` [BLK-2].
    SeqArena,
    /// The proved arena take [BLK-2].
    SeqArenaProved,
    /// `seq_heap<T>['s](heap, count)` [BLK-2].
    SeqHeap,
    /// `arena_frame<const bytes, const align>['s]()` [BLK-2].
    ArenaFrame,
    /// `seq_place(vector, value)` [BLK-3].
    SeqPlace,
    /// `seq_place_front(vector, value)` [BLK-3].
    SeqPlaceFront,
    /// `seq_take(vector)` [BLK-3].
    SeqTake,
    /// `seq_take_front(vector)` [BLK-3].
    SeqTakeFront,
}

/// One [BLK-0] operation record's lookup data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelOperation {
    /// Exact IDENT spelling.
    pub spelling: &'static str,
    /// The inventory row this spelling names.
    pub row: KernelRow,
    /// Declared value parameter names in declared order [GRAM-11]. The first
    /// is the value the operation transforms and returns, the provider of one
    /// that transforms nothing, or the value one that neither transforms nor
    /// provides observes [BLK-0].
    pub parameters: &'static [&'static str],
    /// Declared result binder spellings in declared order [FN-1].
    pub results: &'static [&'static str],
}

/// The nine operations of the inventory, in [BLK-2] then [BLK-3] order.
pub const KERNEL_OPERATIONS: [KernelOperation; 9] = [
    KernelOperation {
        spelling: "seq_fixed",
        row: KernelRow::SeqFixed,
        parameters: &[],
        results: &["result"],
    },
    KernelOperation {
        spelling: "seq_arena",
        row: KernelRow::SeqArena,
        parameters: &["arena", "count"],
        results: &["made"],
    },
    KernelOperation {
        spelling: "seq_arena_proved",
        row: KernelRow::SeqArenaProved,
        parameters: &["arena", "count"],
        results: &["result"],
    },
    KernelOperation {
        spelling: "seq_heap",
        row: KernelRow::SeqHeap,
        parameters: &["heap", "count"],
        results: &["made"],
    },
    KernelOperation {
        spelling: "arena_frame",
        row: KernelRow::ArenaFrame,
        parameters: &[],
        results: &["result"],
    },
    KernelOperation {
        spelling: "seq_place",
        row: KernelRow::SeqPlace,
        parameters: &["vector", "value"],
        results: &["result"],
    },
    KernelOperation {
        spelling: "seq_place_front",
        row: KernelRow::SeqPlaceFront,
        parameters: &["vector", "value"],
        results: &["result"],
    },
    KernelOperation {
        spelling: "seq_take",
        row: KernelRow::SeqTake,
        parameters: &["vector"],
        results: &["rest", "value"],
    },
    KernelOperation {
        spelling: "seq_take_front",
        row: KernelRow::SeqTakeFront,
        parameters: &["vector"],
        results: &["rest", "value"],
    },
];

/// The nominal record one resolved container target names.
#[must_use]
pub fn container_nominal(id: ContainerNominalId) -> Option<&'static ContainerNominal> {
    CONTAINER_NOMINALS.get(usize::from(id.ordinal()))
}

/// The operation record one resolved kernel target names.
#[must_use]
pub fn kernel_operation(id: KernelOperationId) -> Option<&'static KernelOperation> {
    KERNEL_OPERATIONS.get(usize::from(id.ordinal()))
}

/// The lookup classes of every container nominal: one entry of the
/// nominal-type TYPEID domain and one of the constructor TYPEID domain
/// [TYPE-6], exactly as a source `struct_decl` contributes both.
///
/// The constructor entry exists to be refused: [BLK-1] states that no
/// `construct` produces a run, a provider, or a store, and the entry is what
/// makes that refusal a judgment over a resolved declaration rather than a
/// name comparison in the checker.
pub const CONTAINER_NOMINAL_CLASSES: [DeclarationClass; 2] = [
    DeclarationClass::NominalType,
    DeclarationClass::StructConstructor,
];

/// The nominal-type class of a container nominal, which every `type` position
/// admits [TYPE-6].
pub const CONTAINER_NOMINAL_CLASS: DeclarationClass = DeclarationClass::NominalType;

/// The lookup class of every kernel-domain operation: one entry of the
/// lexical IDENT domain, taking the function class [BLK-0, TYPE-6].
pub const KERNEL_OPERATION_CLASS: DeclarationClass = DeclarationClass::Function;

#[cfg(test)]
mod tests {
    use super::{CONTAINER_NOMINALS, KERNEL_OPERATIONS};
    use crate::resolution::catalog::{MODE_WORDS, OPERATION_FAMILIES, SYSTEM_NOMINALS};

    /// [BLK-0]: a kernel-domain operation spelling is IDENT-shaped, contains
    /// no dot, and is no member of `ReservedLowerNames` [OP-1], so adding the
    /// inventory takes no spelling away from a writer's declarations.
    #[test]
    fn no_kernel_operation_spelling_is_reserved() {
        for operation in KERNEL_OPERATIONS {
            assert!(!operation.spelling.contains('.'), "{}", operation.spelling);
            assert!(
                operation
                    .spelling
                    .starts_with(|byte: char| byte.is_ascii_lowercase()),
                "{}",
                operation.spelling
            );
            assert!(
                !OPERATION_FAMILIES.contains(&operation.spelling),
                "{} is an OP-1 family spelling",
                operation.spelling
            );
            assert!(
                !MODE_WORDS.contains(&operation.spelling),
                "{} is a mode word",
                operation.spelling
            );
        }
    }

    /// [TYPE-6]: spellings are unique within each domain and disjoint from
    /// the system inventory's spellings of the same domain.
    #[test]
    fn kernel_spellings_are_unique_and_disjoint() {
        let mut nominals: Vec<_> = CONTAINER_NOMINALS
            .iter()
            .map(|nominal| nominal.spelling)
            .collect();
        nominals.sort_unstable();
        let count = nominals.len();
        nominals.dedup();
        assert_eq!(nominals.len(), count);
        for nominal in CONTAINER_NOMINALS {
            assert!(
                !SYSTEM_NOMINALS
                    .iter()
                    .any(|system| system.spelling == nominal.spelling),
                "{} collides with a system nominal",
                nominal.spelling
            );
        }
        let mut operations: Vec<_> = KERNEL_OPERATIONS
            .iter()
            .map(|operation| operation.spelling)
            .collect();
        operations.sort_unstable();
        let count = operations.len();
        operations.dedup();
        assert_eq!(operations.len(), count);
    }

    /// [BLK-0]'s first-parameter ordering, over the inventory this version
    /// carries: the transforming rows name `vector` first, the acquiring rows
    /// name their provider first, and the two rows that neither transform nor
    /// provide take no value parameter at all.
    #[test]
    fn every_row_orders_its_first_parameter() {
        for operation in KERNEL_OPERATIONS {
            let Some(first) = operation.parameters.first() else {
                continue;
            };
            assert!(
                matches!(*first, "vector" | "arena" | "heap"),
                "{} names {first} first",
                operation.spelling
            );
        }
    }
}
