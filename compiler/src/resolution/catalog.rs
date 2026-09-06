use super::{
    DeclarationClass, OperationFamilyId, PreludeDeclarationId, PreludeDeclarationRecord,
    ReservedNameClass, SystemDeclarationId, SystemDeclarationRecord,
};

pub(crate) const PRELUDE_DECLARATIONS: [PreludeDeclarationRecord; 24] = [
    prelude(0, "Bool", Some(DeclarationClass::NominalType)),
    prelude(1, "True", Some(DeclarationClass::EnumVariant)),
    prelude(2, "False", Some(DeclarationClass::EnumVariant)),
    prelude(3, "Option", Some(DeclarationClass::NominalType)),
    prelude(4, "T", None),
    prelude(5, "None", Some(DeclarationClass::EnumVariant)),
    prelude(6, "Some", Some(DeclarationClass::EnumVariant)),
    prelude(7, "value", None),
    prelude(8, "Result", Some(DeclarationClass::NominalType)),
    prelude(9, "T", None),
    prelude(10, "E", None),
    prelude(11, "Ok", Some(DeclarationClass::EnumVariant)),
    prelude(12, "value", None),
    prelude(13, "Err", Some(DeclarationClass::EnumVariant)),
    prelude(14, "error", None),
    prelude(15, "Overflow", Some(DeclarationClass::NominalType)),
    prelude(16, "Overflow", Some(DeclarationClass::EnumVariant)),
    prelude(17, "DivError", Some(DeclarationClass::NominalType)),
    prelude(18, "DivideByZero", Some(DeclarationClass::EnumVariant)),
    prelude(19, "DivOverflow", Some(DeclarationClass::EnumVariant)),
    prelude(20, "NarrowError", Some(DeclarationClass::NominalType)),
    prelude(21, "NarrowError", Some(DeclarationClass::EnumVariant)),
    prelude(22, "Int", Some(DeclarationClass::Contract)),
    prelude(23, "Float", Some(DeclarationClass::Contract)),
];

const fn prelude(
    ordinal: u8,
    spelling: &'static str,
    class: Option<DeclarationClass>,
) -> PreludeDeclarationRecord {
    PreludeDeclarationRecord {
        id: PreludeDeclarationId::new(ordinal),
        spelling,
        class,
    }
}

/// Distinct OP-1 spellings in normative table order, with repeated `cvt`
/// collapsed at its first occurrence as required by OP-1.
pub(crate) const OPERATION_FAMILIES: [&str; 91] = [
    "+wrap",
    "-wrap",
    "*wrap",
    "+",
    "-",
    "*",
    "+defined",
    "-defined",
    "*defined",
    "+checked",
    "-checked",
    "*checked",
    "/",
    "%",
    "/defined",
    "%defined",
    "/checked",
    "%checked",
    "ineg.wrap",
    "ineg",
    "ineg.defined",
    "ineg.checked",
    "==",
    "!=",
    "<",
    "<=",
    ">",
    ">=",
    "eeq",
    "ene",
    "fadd.strict",
    "fsub.strict",
    "fmul.strict",
    "fdiv.strict",
    "feq",
    "flt",
    "fle",
    "fgt",
    "fge",
    "fne",
    "band",
    "bor",
    "bxor",
    "bnot",
    "cvt",
    "len_of",
    "cap_of",
    "room_of",
    "head_of",
    "fits",
    "iand",
    "ior",
    "ixor",
    "inot",
    "ishl.wrap",
    "ishr.wrap",
    "ishl",
    "ishr",
    "ishl.defined",
    "ishr.defined",
    "irotl",
    "irotr",
    "ipopcount",
    "iclz",
    "ictz",
    "ibswap",
    "imulhi",
    "+sat",
    "-sat",
    "*sat",
    "imin",
    "imax",
    "iabs.wrap",
    "iabs",
    "iabs.defined",
    "iabs.checked",
    "reinterpret",
    "fneg",
    "fabs",
    "fcopysign",
    "fmin",
    "fmax",
    "ffloor",
    "fceil",
    "ftrunc",
    "froundeven",
    "frem",
    "fsqrt.strict",
    "ffma.strict",
    "finf",
    "fnan",
];

pub(crate) const MODE_WORDS: [&str; 5] = ["wrap", "defined", "checked", "sat", "strict"];

pub(crate) fn operation_id(spelling: &str) -> Option<OperationFamilyId> {
    OPERATION_FAMILIES
        .iter()
        .position(|candidate| *candidate == spelling)
        .and_then(OperationFamilyId::from_index)
}

pub(crate) fn operation_spelling(id: OperationFamilyId) -> Option<&'static str> {
    OPERATION_FAMILIES.get(usize::from(id.0)).copied()
}

/// Which of [SYS-2]'s three categories one system nominal type belongs to.
///
/// A system-declared struct is the third category, added by v0.50 for
/// `TcpConnection` [SYS-18]. It contributes one nominal-type entry and no
/// constructor entry, so no source expression constructs one; its two field
/// records are owner-local to the nominal, and every rule that already
/// governs a source struct's fields governs them unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemNominalCategory {
    /// An opaque type with no writer-visible field, variant, or literal.
    Opaque,
    /// A system-declared struct with owner-local field records.
    Struct,
    /// An outcome enum whose variants are constructor entries.
    Enum,
}

/// One [SYS-2] system nominal type in normative table order.
#[derive(Clone, Copy, Debug)]
pub struct SystemNominal {
    /// Exact TYPEID spelling.
    pub spelling: &'static str,
    /// The [SYS-2] category this nominal belongs to.
    pub category: SystemNominalCategory,
    /// Declared fields in declared order, for a struct nominal only; every
    /// other category declares none here, because an enum's fields belong to
    /// its constructors and an opaque type has none.
    pub fields: &'static [SystemField],
}

impl SystemNominal {
    /// Whether this is one of [SYS-2]'s opaque types.
    #[must_use]
    pub const fn is_opaque(&self) -> bool {
        matches!(self.category, SystemNominalCategory::Opaque)
    }

    /// Whether this is a system-declared struct [SYS-18].
    #[must_use]
    pub const fn is_struct(&self) -> bool {
        matches!(self.category, SystemNominalCategory::Struct)
    }
}

/// One [SYS-2] enum-variant constructor in normative table order.
#[derive(Clone, Copy, Debug)]
pub struct SystemConstructor {
    /// Exact TYPEID spelling.
    pub spelling: &'static str,
    /// Index of the owning enum nominal in [`SYSTEM_NOMINALS`].
    pub owner: u8,
    /// Declared fields in declared order; owner-local, never in source lookup.
    pub fields: &'static [SystemField],
}

/// One owner-local [SYS-2] constructor field.
#[derive(Clone, Copy, Debug)]
pub struct SystemField {
    /// Declared field name.
    pub name: &'static str,
    /// Declared field type.
    pub ty: SystemTypeRef,
}

/// One [SYS-2] operation signature in normative table order.
///
/// Target-stage result bounds are attached here because they are contracts of
/// the semantic identity, not properties an approved implementation version
/// may add or remove.
#[derive(Clone, Copy, Debug)]
pub struct SystemOperation {
    /// Exact IDENT spelling.
    pub spelling: &'static str,
    /// Region parameters in declared order, complete sigiled spellings.
    pub regions: &'static [&'static str],
    /// Value parameters in declared order; owner-local, never in source lookup.
    pub parameters: &'static [SystemParameter],
    /// Result type; every [SYS-2] result mode is `own`.
    pub result: SystemTypeRef,
    /// Extra value-parameter state observations not implied by borrow mode.
    pub state_reads: &'static [u8],
    /// Extra value-parameter state transitions not implied by borrow mode.
    pub state_writes: &'static [u8],
    /// Fixed selected-target upper bound for a plain integer result, when the
    /// semantic identity supplies one.
    pub integer_result_bound: Option<SystemIntegerResultBound>,
    /// Compiler-owned execution contract for this operation. This is target
    /// metadata, never a source effect or a source-visible ordering token.
    pub target_action: TargetAction,
    /// Whether the operation's result contains fresh opaque state. Fresh
    /// state has no incoming-formal origin and carries no parent relation.
    pub result_state_origin: SystemResultStateOrigin,
}

/// One selected-target upper-bound contract fixed by a system operation's
/// semantic identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SystemIntegerResultBound {
    /// The integer result is no greater than the selected target's
    /// address-index maximum.
    AddressIndexMaximum,
}

/// Whether a system operation's result contains fresh opaque state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SystemResultStateOrigin {
    /// The result type carries no opaque state identity.
    None,
    /// A successful result contains state supplied by no input formal.
    Fresh,
}

/// How one target action returns control to its Whitefoot continuation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TargetDispatch {
    /// The result is available before the native call returns.
    Inline,
    /// The operation may return through the completion backend.
    MaySuspend,
}

/// The milestone that restores the action's transferred ownership bundle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TargetCompletion {
    /// No ownership escapes the native call; completion is its return.
    CallReturn,
    /// One finite operation completes when its ownership bundle is restored.
    OwnershipComplete,
}

/// Separately named completion facts for one finite action.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TargetMilestones {
    /// When the operation result becomes usable.
    pub result_ready: TargetCompletion,
    /// When payload loans return to their owner.
    pub payload_released: TargetCompletion,
    /// When transferred owners and loans return to their owner.
    pub ownership_released: TargetCompletion,
    /// When no later completion fact can arrive.
    pub terminal: TargetCompletion,
}

/// Closed compiler metadata for one target action.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TargetAction {
    /// Whether the action can suspend its Whitefoot continuation.
    pub dispatch: TargetDispatch,
    /// The milestone at which its transferred ownership becomes usable again.
    pub completion: TargetCompletion,
    /// Product-state milestone contract. The first system slice publishes all
    /// four facts together while retaining their distinct meanings.
    pub milestones: TargetMilestones,
}

impl TargetAction {
    /// A target-independent computation that completes in the call itself.
    pub const INLINE: Self = Self {
        dispatch: TargetDispatch::Inline,
        completion: TargetCompletion::CallReturn,
        milestones: TargetMilestones {
            result_ready: TargetCompletion::CallReturn,
            payload_released: TargetCompletion::CallReturn,
            ownership_released: TargetCompletion::CallReturn,
            terminal: TargetCompletion::CallReturn,
        },
    };

    /// A finite one-shot action that may complete asynchronously.
    pub const MAY_SUSPEND: Self = Self {
        dispatch: TargetDispatch::MaySuspend,
        completion: TargetCompletion::OwnershipComplete,
        milestones: TargetMilestones {
            result_ready: TargetCompletion::OwnershipComplete,
            payload_released: TargetCompletion::OwnershipComplete,
            ownership_released: TargetCompletion::OwnershipComplete,
            terminal: TargetCompletion::OwnershipComplete,
        },
    };

    /// Fail-closed metadata for a missing or malformed target record.
    pub const CONSERVATIVE: Self = Self::MAY_SUSPEND;

    /// Reports whether lowering must preserve a resumable continuation.
    #[must_use]
    pub const fn may_suspend(self) -> bool {
        matches!(self.dispatch, TargetDispatch::MaySuspend)
    }

    /// Conservatively combines two reachable target actions.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        if self.may_suspend() || other.may_suspend() {
            Self::MAY_SUSPEND
        } else {
            Self::INLINE
        }
    }
}

/// One owner-local [SYS-2] operation value parameter.
#[derive(Clone, Copy, Debug)]
pub struct SystemParameter {
    /// Declared parameter name; [GRAM-11] named arguments must equal it.
    pub name: &'static str,
    /// Declared parameter mode.
    pub mode: SystemParameterMode,
    /// Declared parameter type.
    pub ty: SystemTypeRef,
}

/// A [SYS-2] parameter mode; borrow modes index the operation's region list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemParameterMode {
    /// `own`.
    Own,
    /// `&'r` with the zero-based declared region-parameter index.
    Borrow(u8),
    /// `&uniq 'r` with the zero-based declared region-parameter index.
    UniqueBorrow(u8),
}

/// The closed set of types written in the [SYS-2] table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemTypeRef {
    /// `u8`.
    U8,
    /// `u16`.
    U16,
    /// `u32`.
    U32,
    /// `u64`.
    U64,
    /// [SYS-8] the **destination** operand class of a range-bearing
    /// operation [SYS-8]: the storage that operation writes.
    ///
    /// It is a class rather than one type for the reason [VIEW-2]'s viewable
    /// class is one — the class is wider than any one type. Its member is the
    /// exclusive view `MutSlice<'r, u8>`, and, until S34 retires the old
    /// container surface, `buffer<u8>` as well. Nothing in a row reads what
    /// the storage is made of: the operation writes element storage through
    /// the descriptor it is handed, and its two range obligations are stated
    /// over `len_of` of whichever member the call supplied.
    DestinationU8,
    /// [SYS-8] the **source** operand class of a range-bearing operation
    /// [SYS-8]: the storage that operation reads. Its member is the shared
    /// view `Slice<'r, u8>` and, transitionally, `buffer<u8>`.
    SourceU8,
    /// One system nominal type, by index into [`SYSTEM_NOMINALS`].
    Nominal(u8),
    /// One [PRE-1] `Result<T, E>` instantiation over table types.
    Result {
        /// The `Ok` payload type.
        ok: SystemResultPayload,
        /// The `Err` payload, an enum index into [`SYSTEM_NOMINALS`].
        err: u8,
    },
}

/// The closed set of `Ok` payloads in [SYS-2] `Result` instantiations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemResultPayload {
    /// `u64`.
    U64,
    /// One system nominal type, by index into [`SYSTEM_NOMINALS`].
    Nominal(u8),
}

/// One resolved [SYS-2] lookup entry, for consumers of a resolved unit.
#[derive(Clone, Copy, Debug)]
pub enum SystemEntity {
    /// A system nominal type.
    Nominal(&'static SystemNominal),
    /// A system enum-variant constructor.
    Constructor(&'static SystemConstructor),
    /// A system operation.
    Operation(&'static SystemOperation),
}

const ARGS: u8 = 0;
const HOST_STRING: u8 = 1;
const RELATIVE_PATH: u8 = 2;
const DIRECTORY_READ: u8 = 3;
const READ_FILE: u8 = 4;
const OUTPUT: u8 = 5;
const EXIT_STATUS: u8 = 6;
const ARG_ERROR: u8 = 7;
const UTF8_ERROR: u8 = 8;
const COPY_ERROR: u8 = 9;
const UTF8_COPY_ERROR: u8 = 10;
const PATH_ERROR: u8 = 11;
const READ_OUTCOME: u8 = 12;
const IO_ERROR: u8 = 13;
const DIRECTORY_SOURCE: u8 = 14;
const LIST_OUTCOME: u8 = 15;
const HANDLE_FACTORY: u8 = 16;
const HANDLE_PERMIT: u8 = 17;
const FILE_OPEN_OUTCOME: u8 = 18;
const DIRECTORY_OPEN_OUTCOME: u8 = 19;
const SOURCE_OPEN_OUTCOME: u8 = 20;
const INPUT_STREAM: u8 = 21;
const SOCKET_ADDRESS: u8 = 22;
const TCP_LISTENER: u8 = 23;
const TCP_RECEIVE: u8 = 24;
const TCP_SEND: u8 = 25;
const TCP_CONNECTION: u8 = 26;
const LISTEN_OUTCOME: u8 = 27;
const ACCEPT_OUTCOME: u8 = 28;
const CONNECT_OUTCOME: u8 = 29;

/// The traversal surface switch [SYS-2, SYS-14], activated as v0.32.
///
/// `false` admits exactly the v0.31 inventory: the traversal rows below are
/// unreachable, every declaration ordinal keeps its v0.31 value, and the
/// resolver, checker, and backend see the same one hundred sixty-seven
/// records they saw before. `true` admits the directory-enumeration
/// surface — `DirectorySource`, `ListOutcome`, `open_directory`,
/// `open_directory_source`, and `directory_next` — as the last row of each
/// [SYS-2] table. It is now `true`,
/// because v0.32 activated that surface; `false` stays reachable as the exact
/// differential against the v0.31 base tables.
pub const TRAVERSAL_SURFACE: bool = true;

/// The active v0.33 file-open-by-name switch [SYS-2, SYS-11].
///
/// `false` admits exactly the superseded v0.32 inventory: the `open_file` row
/// below is unreachable, every declaration ordinal keeps its v0.32 value, and
/// the resolver, checker, and backend see the same one hundred ninety-two
/// records in that archive. `true` admits the active operation — the
/// `open_read` sibling that takes a caller-owned single path component
/// instead of a `RelativePath` — as the last row of the [SYS-2] operation
/// table. The compiler selects `true` so the complete v0.33 surface follows
/// the ordinary path; `false` remains the exact superseded-v0.32 differential.
pub const OPEN_BY_NAME: bool = true;

/// The active v0.50 streams-and-TCP switch [SYS-2, SYS-15, SYS-16, SYS-17,
/// SYS-18].
///
/// `false` admits exactly the superseded v0.49 inventory: the stream, address,
/// listener and connection rows below are unreachable, every declaration
/// ordinal keeps its v0.49 value, and the resolver, checker, and backend see
/// the same two hundred twenty-seven records that archive declares. `true`
/// admits the active surface — `InputStream`, `SocketAddress`, `TcpListener`,
/// `TcpReceive`, `TcpSend`, the system-declared struct `TcpConnection`, the
/// three connection outcome enums, and the ten operations of §4 — as the last
/// rows of each [SYS-2] table. The compiler selects `true`; `false` remains
/// the exact superseded-v0.49 differential.
pub const STREAMS_AND_TCP: bool = true;

/// One selected [SYS-2] inventory state.
///
/// The three states are strictly nested prefixes of the tables below, taken
/// in normative order, so a state is a length rather than a set of
/// independent features: every declaration ordinal an earlier state assigns
/// keeps exactly that value in a later one. That is what lets one differential
/// test show that switching a candidate off leaves every earlier program's
/// emitted module byte-identical.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Inventory {
    /// The v0.31 inventory: the tables with no candidate row at all.
    Base,
    /// The superseded v0.32 inventory: [`Inventory::Base`] plus the [SYS-14]
    /// traversal surface.
    Traversal,
    /// The active v0.33 inventory: [`Inventory::Traversal`] plus the
    /// [SYS-11] `open_file` operation.
    OpenByName,
    /// The superseded v0.49 unified-state file-open surface:
    /// [`Inventory::OpenByName`] plus the explicit handle factory, one-shot
    /// permit, and reservation row.
    FilePermits,
    /// The active v0.50 surface: [`Inventory::FilePermits`] plus the readable
    /// stream, the socket address, the listener, the connection struct with
    /// its two direction resources, the three connection outcome enums, and
    /// their ten operations.
    StreamsAndTcp,
}

impl Inventory {
    /// The inventory the shipped compilation path selects, fixed by the two
    /// switches above and read once, by `compile` and `resolve`.
    pub const ACTIVE: Self = if STREAMS_AND_TCP {
        Self::StreamsAndTcp
    } else if OPEN_BY_NAME {
        Self::FilePermits
    } else if TRAVERSAL_SURFACE {
        Self::Traversal
    } else {
        Self::Base
    };

    /// How many [`SYSTEM_NOMINALS`] rows this state admits.
    const fn nominals(self) -> usize {
        match self {
            Self::Base => BASE_NOMINALS,
            Self::Traversal | Self::OpenByName => OPEN_BY_NAME_NOMINALS,
            Self::FilePermits => FILE_PERMIT_NOMINALS,
            Self::StreamsAndTcp => SYSTEM_NOMINALS.len(),
        }
    }

    /// How many [`SYSTEM_CONSTRUCTORS`] rows this state admits.
    const fn constructors(self) -> usize {
        match self {
            Self::Base => BASE_CONSTRUCTORS,
            Self::Traversal | Self::OpenByName => OPEN_BY_NAME_CONSTRUCTORS,
            Self::FilePermits => FILE_PERMIT_CONSTRUCTORS,
            Self::StreamsAndTcp => SYSTEM_CONSTRUCTORS.len(),
        }
    }

    /// How many [`SYSTEM_OPERATIONS`] rows this state admits.
    const fn operations(self) -> usize {
        match self {
            Self::Base => BASE_OPERATIONS,
            Self::Traversal => TRAVERSAL_OPERATIONS,
            Self::OpenByName => OPEN_BY_NAME_OPERATIONS,
            Self::FilePermits => FILE_PERMIT_OPERATIONS,
            Self::StreamsAndTcp => SYSTEM_OPERATIONS.len(),
        }
    }
}

/// The v0.31 nominal-type count: the prefix of [`SYSTEM_NOMINALS`] the v0.31
/// specification declared.
const BASE_NOMINALS: usize = 14;
/// The v0.31 constructor count: the prefix of [`SYSTEM_CONSTRUCTORS`] the
/// v0.31 specification declared.
const BASE_CONSTRUCTORS: usize = 37;
/// The v0.31 operation count: the prefix of [`SYSTEM_OPERATIONS`] the v0.31
/// specification declared.
const BASE_OPERATIONS: usize = 11;
/// The v0.32 operation count: the prefix of [`SYSTEM_OPERATIONS`] that
/// specification declared.
const TRAVERSAL_OPERATIONS: usize = 14;
/// The v0.33-v0.36 operation count before explicit file-open authority.
const OPEN_BY_NAME_OPERATIONS: usize = 15;
/// The v0.33-v0.36 nominal count before explicit file-open authority.
const OPEN_BY_NAME_NOMINALS: usize = 16;
/// The v0.32-v0.41 constructor count before the open outcome enums.
const OPEN_BY_NAME_CONSTRUCTORS: usize = 40;
/// The v0.49 nominal count before the streams-and-TCP surface.
const FILE_PERMIT_NOMINALS: usize = 21;
/// The v0.49 constructor count before the streams-and-TCP surface.
const FILE_PERMIT_CONSTRUCTORS: usize = 46;
/// The v0.49 operation count before the streams-and-TCP surface.
const FILE_PERMIT_OPERATIONS: usize = 19;

/// The [SYS-2] nominal types in normative table order.
///
/// The first fourteen are v0.31's; the last two are v0.32's traversal-surface
/// additions and are admitted only under
/// [`TRAVERSAL_SURFACE`].
pub const SYSTEM_NOMINALS: [SystemNominal; 30] = [
    opaque("Args"),
    opaque("HostString"),
    opaque("RelativePath"),
    opaque("DirectoryRead"),
    opaque("ReadFile"),
    opaque("OutputStream"),
    opaque("ExitStatus"),
    enumeration("ArgError"),
    enumeration("Utf8Error"),
    enumeration("CopyError"),
    enumeration("Utf8CopyError"),
    enumeration("PathError"),
    enumeration("ReadOutcome"),
    enumeration("IoError"),
    opaque("DirectorySource"),
    enumeration("ListOutcome"),
    opaque("HandleFactory"),
    opaque("HandlePermit"),
    enumeration("FileOpenOutcome"),
    enumeration("DirectoryOpenOutcome"),
    enumeration("SourceOpenOutcome"),
    opaque("InputStream"),
    opaque("SocketAddress"),
    opaque("TcpListener"),
    opaque("TcpReceive"),
    opaque("TcpSend"),
    // The one system-declared struct [SYS-18]. It carries its two field
    // records here, contributes no constructor entry, and is therefore
    // constructed by no source expression.
    system_struct("TcpConnection", &CONNECTION_DIRECTIONS),
    enumeration("ListenOutcome"),
    enumeration("AcceptOutcome"),
    enumeration("ConnectOutcome"),
];

/// The two owner-local field records of [SYS-18]'s `TcpConnection`.
const CONNECTION_DIRECTIONS: [SystemField; 2] = [
    field("receive", SystemTypeRef::Nominal(TCP_RECEIVE)),
    field("send", SystemTypeRef::Nominal(TCP_SEND)),
];

/// The [SYS-2] nominal types one inventory state admits.
#[must_use]
pub fn system_nominals(inventory: Inventory) -> &'static [SystemNominal] {
    &SYSTEM_NOMINALS[..inventory.nominals()]
}

/// The [SYS-2] enum-variant constructors one inventory state admits.
#[must_use]
pub fn system_constructors(inventory: Inventory) -> &'static [SystemConstructor] {
    &SYSTEM_CONSTRUCTORS[..inventory.constructors()]
}

/// The [SYS-2] operations one inventory state admits.
#[must_use]
pub fn system_operations(inventory: Inventory) -> &'static [SystemOperation] {
    &SYSTEM_OPERATIONS[..inventory.operations()]
}

const fn opaque(spelling: &'static str) -> SystemNominal {
    SystemNominal {
        spelling,
        category: SystemNominalCategory::Opaque,
        fields: &[],
    }
}

const fn enumeration(spelling: &'static str) -> SystemNominal {
    SystemNominal {
        spelling,
        category: SystemNominalCategory::Enum,
        fields: &[],
    }
}

const fn system_struct(spelling: &'static str, fields: &'static [SystemField]) -> SystemNominal {
    SystemNominal {
        spelling,
        category: SystemNominalCategory::Struct,
        fields,
    }
}

/// The exact inline detail carried by every [SYS-2] `IoError` class.
const IO_ERROR_DETAIL: [SystemField; 2] = [
    field("code", SystemTypeRef::U32),
    field("origin", SystemTypeRef::U8),
];

const REQUIRED_U64: [SystemField; 1] = [field("required", SystemTypeRef::U64)];
const NEXT_U64: [SystemField; 1] = [field("next", SystemTypeRef::U64)];
const ERROR_IO: [SystemField; 1] = [field("error", SystemTypeRef::Nominal(IO_ERROR))];
/// One `ListBytes` payload: the absolute end of the portable prefix and the
/// exact number of entry records that prefix holds.
const NEXT_AND_ENTRIES_U64: [SystemField; 2] = [
    field("next", SystemTypeRef::U64),
    field("entries", SystemTypeRef::U64),
];
/// The one payload of a successful open: the fresh owner [SYS-10].
const OPENED_FILE: [SystemField; 1] = [field("value", SystemTypeRef::Nominal(READ_FILE))];
const OPENED_DIRECTORY: [SystemField; 1] = [field("value", SystemTypeRef::Nominal(DIRECTORY_READ))];
const OPENED_SOURCE: [SystemField; 1] = [field("value", SystemTypeRef::Nominal(DIRECTORY_SOURCE))];
/// The payload of a refused open: the host's error and the permit the open
/// took, handed back because no descriptor was taken [SYS-10].
const ERROR_AND_PERMIT: [SystemField; 2] = [
    field("error", SystemTypeRef::Nominal(IO_ERROR)),
    field("permit", SystemTypeRef::Nominal(HANDLE_PERMIT)),
];

/// The one payload of a successful `tcp_listen`: the fresh listener [SYS-17].
const LISTENING: [SystemField; 1] = [field("listener", SystemTypeRef::Nominal(TCP_LISTENER))];
/// The payload of a successful `tcp_accept`: the fresh connection pair and the
/// address the target reported for its peer [SYS-17, SYS-18].
const ACCEPTED: [SystemField; 2] = [
    field("connection", SystemTypeRef::Nominal(TCP_CONNECTION)),
    field("peer", SystemTypeRef::Nominal(SOCKET_ADDRESS)),
];
/// The one payload of a successful `tcp_connect`: the fresh connection pair.
const CONNECTED: [SystemField; 1] = [field("connection", SystemTypeRef::Nominal(TCP_CONNECTION))];

const fn field(name: &'static str, ty: SystemTypeRef) -> SystemField {
    SystemField { name, ty }
}

const fn io_class(spelling: &'static str) -> SystemConstructor {
    SystemConstructor {
        spelling,
        owner: IO_ERROR,
        fields: &IO_ERROR_DETAIL,
    }
}

const fn constructor(
    spelling: &'static str,
    owner: u8,
    fields: &'static [SystemField],
) -> SystemConstructor {
    SystemConstructor {
        spelling,
        owner,
        fields,
    }
}

/// The [SYS-2] enum-variant constructors in normative table order: each enum
/// in table order, and within one enum each variant in declared order.
///
/// The first thirty-nine are the active specification's; the last three are
/// the traversal-surface candidate's `ListOutcome` variants, admitted only
/// under [`TRAVERSAL_SURFACE`].
pub const SYSTEM_CONSTRUCTORS: [SystemConstructor; 52] = [
    constructor("InvalidIndex", ARG_ERROR, &[]),
    constructor("Utf8Invalid", UTF8_ERROR, &[]),
    constructor("CopyTooSmall", COPY_ERROR, &REQUIRED_U64),
    constructor("Utf8CopyTooSmall", UTF8_COPY_ERROR, &REQUIRED_U64),
    constructor("Utf8CopyInvalid", UTF8_COPY_ERROR, &[]),
    constructor("PathInvalid", PATH_ERROR, &[]),
    constructor("ReadBytes", READ_OUTCOME, &NEXT_U64),
    constructor("ReadEnd", READ_OUTCOME, &[]),
    constructor("ReadFailed", READ_OUTCOME, &ERROR_IO),
    io_class("NotFound"),
    io_class("PermissionDenied"),
    io_class("AlreadyExists"),
    io_class("NotDirectory"),
    io_class("IsDirectory"),
    io_class("DirectoryNotEmpty"),
    io_class("ReadOnly"),
    io_class("ResourceBusy"),
    io_class("InvalidInput"),
    io_class("InvalidPath"),
    io_class("Unsupported"),
    io_class("TimedOut"),
    io_class("BrokenPipe"),
    io_class("WriteZero"),
    io_class("UnexpectedEnd"),
    io_class("ConnectionRefused"),
    io_class("ConnectionReset"),
    io_class("ConnectionAborted"),
    io_class("NotConnected"),
    io_class("AddressInUse"),
    io_class("AddressUnavailable"),
    io_class("ResourceExhausted"),
    io_class("FileTooLarge"),
    io_class("NoSpace"),
    io_class("QuotaExceeded"),
    io_class("CrossDevice"),
    io_class("DeviceFailure"),
    io_class("Other"),
    constructor("ListBytes", LIST_OUTCOME, &NEXT_AND_ENTRIES_U64),
    constructor("ListEnd", LIST_OUTCOME, &[]),
    constructor("ListFailed", LIST_OUTCOME, &ERROR_IO),
    constructor("FileOpened", FILE_OPEN_OUTCOME, &OPENED_FILE),
    constructor("FileOpenFailed", FILE_OPEN_OUTCOME, &ERROR_AND_PERMIT),
    constructor("DirectoryOpened", DIRECTORY_OPEN_OUTCOME, &OPENED_DIRECTORY),
    constructor(
        "DirectoryOpenFailed",
        DIRECTORY_OPEN_OUTCOME,
        &ERROR_AND_PERMIT,
    ),
    constructor("SourceOpened", SOURCE_OPEN_OUTCOME, &OPENED_SOURCE),
    constructor("SourceOpenFailed", SOURCE_OPEN_OUTCOME, &ERROR_AND_PERMIT),
    constructor("Listening", LISTEN_OUTCOME, &LISTENING),
    constructor("ListenFailed", LISTEN_OUTCOME, &ERROR_AND_PERMIT),
    constructor("Accepted", ACCEPT_OUTCOME, &ACCEPTED),
    constructor("AcceptFailed", ACCEPT_OUTCOME, &ERROR_AND_PERMIT),
    constructor("Connected", CONNECT_OUTCOME, &CONNECTED),
    constructor("ConnectFailed", CONNECT_OUTCOME, &ERROR_AND_PERMIT),
];

const fn parameter(
    name: &'static str,
    mode: SystemParameterMode,
    ty: SystemTypeRef,
) -> SystemParameter {
    SystemParameter { name, mode, ty }
}

const fn ok_nominal(ok: u8, err: u8) -> SystemTypeRef {
    SystemTypeRef::Result {
        ok: SystemResultPayload::Nominal(ok),
        err,
    }
}

const fn ok_u64(err: u8) -> SystemTypeRef {
    SystemTypeRef::Result {
        ok: SystemResultPayload::U64,
        err,
    }
}

/// The [SYS-2] operation signatures in normative table order.
///
/// The first eleven are v0.31's; the next three are the [SYS-14] traversal
/// surface's, admitted under [`TRAVERSAL_SURFACE`]; the last is the
/// active v0.33 file-open-by-name addition, admitted under [`OPEN_BY_NAME`].
///
/// Each row registers its region and value parameters, result type, unified
/// state subjects and target action. System
/// operations cannot exhibit `traps`: partial domains are static call-site
/// obligations and host failures are typed outcomes. State entries are the
/// exact stored declaration row and are rendered as formal-parameter paths by
/// [`operation_state_effects`].
pub const SYSTEM_OPERATIONS: [SystemOperation; 29] = [
    SystemOperation {
        spelling: "args_count",
        regions: &["'a"],
        parameters: &[parameter(
            "args",
            SystemParameterMode::Borrow(0),
            SystemTypeRef::Nominal(ARGS),
        )],
        result: SystemTypeRef::U64,
        state_reads: &[0],
        state_writes: &[],
        integer_result_bound: None,
        target_action: TargetAction::INLINE,
        result_state_origin: SystemResultStateOrigin::None,
    },
    SystemOperation {
        spelling: "arg_get",
        regions: &["'a"],
        parameters: &[
            parameter(
                "args",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(ARGS),
            ),
            parameter("position", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: ok_nominal(HOST_STRING, ARG_ERROR),
        state_reads: &[0],
        state_writes: &[],
        integer_result_bound: None,
        target_action: TargetAction::INLINE,
        result_state_origin: SystemResultStateOrigin::None,
    },
    SystemOperation {
        spelling: "host_bytes_len",
        regions: &["'v"],
        parameters: &[parameter(
            "value",
            SystemParameterMode::Borrow(0),
            SystemTypeRef::Nominal(HOST_STRING),
        )],
        result: SystemTypeRef::U64,
        state_reads: &[0],
        state_writes: &[],
        // Every qualified HostString producer installs an inline-lease byte
        // length representable by the selected target's address-index domain.
        integer_result_bound: Some(SystemIntegerResultBound::AddressIndexMaximum),
        target_action: TargetAction::INLINE,
        result_state_origin: SystemResultStateOrigin::None,
    },
    SystemOperation {
        spelling: "host_copy_bytes",
        regions: &["'v", "'d"],
        parameters: &[
            parameter(
                "value",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(HOST_STRING),
            ),
            parameter(
                "destination",
                SystemParameterMode::UniqueBorrow(1),
                SystemTypeRef::DestinationU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: ok_u64(COPY_ERROR),
        state_reads: &[0, 1],
        state_writes: &[1],
        integer_result_bound: None,
        target_action: TargetAction::INLINE,
        result_state_origin: SystemResultStateOrigin::None,
    },
    SystemOperation {
        spelling: "host_utf8_len",
        regions: &["'v"],
        parameters: &[parameter(
            "value",
            SystemParameterMode::Borrow(0),
            SystemTypeRef::Nominal(HOST_STRING),
        )],
        result: ok_u64(UTF8_ERROR),
        state_reads: &[0],
        state_writes: &[],
        integer_result_bound: None,
        target_action: TargetAction::INLINE,
        result_state_origin: SystemResultStateOrigin::None,
    },
    SystemOperation {
        spelling: "host_copy_utf8",
        regions: &["'v", "'d"],
        parameters: &[
            parameter(
                "value",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(HOST_STRING),
            ),
            parameter(
                "destination",
                SystemParameterMode::UniqueBorrow(1),
                SystemTypeRef::DestinationU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: ok_u64(UTF8_COPY_ERROR),
        state_reads: &[0, 1],
        state_writes: &[1],
        integer_result_bound: None,
        target_action: TargetAction::INLINE,
        result_state_origin: SystemResultStateOrigin::None,
    },
    SystemOperation {
        spelling: "relative_path",
        regions: &[],
        parameters: &[parameter(
            "value",
            SystemParameterMode::Own,
            SystemTypeRef::Nominal(HOST_STRING),
        )],
        result: ok_nominal(RELATIVE_PATH, PATH_ERROR),
        state_reads: &[],
        state_writes: &[],
        integer_result_bound: None,
        target_action: TargetAction::INLINE,
        result_state_origin: SystemResultStateOrigin::None,
    },
    SystemOperation {
        spelling: "open_read",
        regions: &["'c", "'p"],
        parameters: &[
            parameter(
                "permit",
                SystemParameterMode::Own,
                SystemTypeRef::Nominal(HANDLE_PERMIT),
            ),
            parameter(
                "root",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(DIRECTORY_READ),
            ),
            parameter(
                "path",
                SystemParameterMode::Borrow(1),
                SystemTypeRef::Nominal(RELATIVE_PATH),
            ),
        ],
        result: SystemTypeRef::Nominal(FILE_OPEN_OUTCOME),
        state_reads: &[0, 1, 2],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
    SystemOperation {
        spelling: "read_at",
        regions: &["'f", "'d"],
        parameters: &[
            parameter(
                "file",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(READ_FILE),
            ),
            parameter(
                "destination",
                SystemParameterMode::UniqueBorrow(1),
                SystemTypeRef::DestinationU8,
            ),
            parameter("file_offset", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: SystemTypeRef::Nominal(READ_OUTCOME),
        state_reads: &[0, 1],
        state_writes: &[1],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::None,
    },
    SystemOperation {
        spelling: "write_once",
        regions: &["'o", "'s"],
        parameters: &[
            parameter(
                "output",
                SystemParameterMode::UniqueBorrow(0),
                SystemTypeRef::Nominal(OUTPUT),
            ),
            parameter(
                "source",
                SystemParameterMode::Borrow(1),
                SystemTypeRef::SourceU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: ok_u64(IO_ERROR),
        state_reads: &[0, 1],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::None,
    },
    SystemOperation {
        spelling: "exit_status",
        regions: &[],
        parameters: &[parameter(
            "code",
            SystemParameterMode::Own,
            SystemTypeRef::U8,
        )],
        result: SystemTypeRef::Nominal(EXIT_STATUS),
        state_reads: &[],
        state_writes: &[],
        integer_result_bound: None,
        target_action: TargetAction::INLINE,
        result_state_origin: SystemResultStateOrigin::None,
    },
    // The three traversal-surface candidate rows [SYS-14].
    SystemOperation {
        spelling: "open_directory",
        regions: &["'c", "'n"],
        parameters: &[
            parameter(
                "permit",
                SystemParameterMode::Own,
                SystemTypeRef::Nominal(HANDLE_PERMIT),
            ),
            parameter(
                "root",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(DIRECTORY_READ),
            ),
            parameter(
                "name",
                SystemParameterMode::Borrow(1),
                SystemTypeRef::SourceU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: SystemTypeRef::Nominal(DIRECTORY_OPEN_OUTCOME),
        state_reads: &[0, 1, 2],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
    SystemOperation {
        spelling: "open_directory_source",
        regions: &["'c"],
        parameters: &[
            parameter(
                "permit",
                SystemParameterMode::Own,
                SystemTypeRef::Nominal(HANDLE_PERMIT),
            ),
            parameter(
                "directory",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(DIRECTORY_READ),
            ),
        ],
        result: SystemTypeRef::Nominal(SOURCE_OPEN_OUTCOME),
        state_reads: &[0, 1],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
    SystemOperation {
        spelling: "directory_next",
        regions: &["'l", "'d"],
        parameters: &[
            parameter(
                "source",
                SystemParameterMode::UniqueBorrow(0),
                SystemTypeRef::Nominal(DIRECTORY_SOURCE),
            ),
            parameter(
                "destination",
                SystemParameterMode::UniqueBorrow(1),
                SystemTypeRef::DestinationU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: SystemTypeRef::Nominal(LIST_OUTCOME),
        state_reads: &[0, 1],
        state_writes: &[0, 1],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::None,
    },
    // The active file-open-by-name row [SYS-11]: `open_read`'s sibling
    // over a caller-owned single path component, taking exactly the name
    // range `open_directory` takes.
    SystemOperation {
        spelling: "open_file",
        regions: &["'c", "'n"],
        parameters: &[
            parameter(
                "permit",
                SystemParameterMode::Own,
                SystemTypeRef::Nominal(HANDLE_PERMIT),
            ),
            parameter(
                "root",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(DIRECTORY_READ),
            ),
            parameter(
                "name",
                SystemParameterMode::Borrow(1),
                SystemTypeRef::SourceU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: SystemTypeRef::Nominal(FILE_OPEN_OUTCOME),
        state_reads: &[0, 1, 2],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
    SystemOperation {
        spelling: "reserve_handle",
        regions: &["'f"],
        parameters: &[parameter(
            "factory",
            SystemParameterMode::UniqueBorrow(0),
            SystemTypeRef::Nominal(HANDLE_FACTORY),
        )],
        result: ok_nominal(HANDLE_PERMIT, IO_ERROR),
        state_reads: &[0],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::INLINE,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
    // The three explicit closes [SYS-10]: each consumes its owner, performs
    // the one native close attempt derived release would perform, and returns
    // the credit the open spent as one fresh permit on every outcome.
    SystemOperation {
        spelling: "close_read",
        regions: &[],
        parameters: &[parameter(
            "file",
            SystemParameterMode::Own,
            SystemTypeRef::Nominal(READ_FILE),
        )],
        result: SystemTypeRef::Nominal(HANDLE_PERMIT),
        state_reads: &[0],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
    SystemOperation {
        spelling: "close_directory",
        regions: &[],
        parameters: &[parameter(
            "directory",
            SystemParameterMode::Own,
            SystemTypeRef::Nominal(DIRECTORY_READ),
        )],
        result: SystemTypeRef::Nominal(HANDLE_PERMIT),
        state_reads: &[0],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
    SystemOperation {
        spelling: "close_directory_source",
        regions: &[],
        parameters: &[parameter(
            "source",
            SystemParameterMode::Own,
            SystemTypeRef::Nominal(DIRECTORY_SOURCE),
        )],
        result: SystemTypeRef::Nominal(HANDLE_PERMIT),
        state_reads: &[0],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
    // The v0.50 stream row [SYS-15]: the unpositioned read of the entry's
    // standard input. It writes the stream because the position it advances is
    // the whole of that value's state, which is exactly what `read_at` does
    // not do.
    SystemOperation {
        spelling: "read_next",
        regions: &["'i", "'d"],
        parameters: &[
            parameter(
                "input",
                SystemParameterMode::UniqueBorrow(0),
                SystemTypeRef::Nominal(INPUT_STREAM),
            ),
            parameter(
                "destination",
                SystemParameterMode::UniqueBorrow(1),
                SystemTypeRef::DestinationU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: SystemTypeRef::Nominal(READ_OUTCOME),
        state_reads: &[0, 1],
        state_writes: &[0, 1],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::None,
    },
    // The two address constructors [SYS-16]: total, pure, no host call.
    SystemOperation {
        spelling: "socket_address_v4",
        regions: &[],
        parameters: &[
            parameter("a", SystemParameterMode::Own, SystemTypeRef::U8),
            parameter("b", SystemParameterMode::Own, SystemTypeRef::U8),
            parameter("c", SystemParameterMode::Own, SystemTypeRef::U8),
            parameter("d", SystemParameterMode::Own, SystemTypeRef::U8),
            parameter("port", SystemParameterMode::Own, SystemTypeRef::U16),
        ],
        result: SystemTypeRef::Nominal(SOCKET_ADDRESS),
        state_reads: &[],
        state_writes: &[],
        integer_result_bound: None,
        target_action: TargetAction::INLINE,
        result_state_origin: SystemResultStateOrigin::None,
    },
    SystemOperation {
        spelling: "socket_address_v6",
        regions: &[],
        parameters: &[
            parameter("a", SystemParameterMode::Own, SystemTypeRef::U16),
            parameter("b", SystemParameterMode::Own, SystemTypeRef::U16),
            parameter("c", SystemParameterMode::Own, SystemTypeRef::U16),
            parameter("d", SystemParameterMode::Own, SystemTypeRef::U16),
            parameter("e", SystemParameterMode::Own, SystemTypeRef::U16),
            parameter("f", SystemParameterMode::Own, SystemTypeRef::U16),
            parameter("g", SystemParameterMode::Own, SystemTypeRef::U16),
            parameter("h", SystemParameterMode::Own, SystemTypeRef::U16),
            parameter("port", SystemParameterMode::Own, SystemTypeRef::U16),
        ],
        result: SystemTypeRef::Nominal(SOCKET_ADDRESS),
        state_reads: &[],
        state_writes: &[],
        integer_result_bound: None,
        target_action: TargetAction::INLINE,
        result_state_origin: SystemResultStateOrigin::None,
    },
    // The three permit-consuming TCP rows [SYS-17]. Each takes the permit by
    // `own` and hands it back inside its failed variant, exactly as an open
    // does, because the handle a listener or a connection costs is one credit
    // of the same factory.
    SystemOperation {
        spelling: "tcp_listen",
        regions: &["'a"],
        parameters: &[
            parameter(
                "permit",
                SystemParameterMode::Own,
                SystemTypeRef::Nominal(HANDLE_PERMIT),
            ),
            parameter(
                "address",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(SOCKET_ADDRESS),
            ),
        ],
        result: SystemTypeRef::Nominal(LISTEN_OUTCOME),
        state_reads: &[0, 1],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
    SystemOperation {
        spelling: "tcp_accept",
        regions: &["'l"],
        parameters: &[
            parameter(
                "permit",
                SystemParameterMode::Own,
                SystemTypeRef::Nominal(HANDLE_PERMIT),
            ),
            parameter(
                "listener",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(TCP_LISTENER),
            ),
        ],
        result: SystemTypeRef::Nominal(ACCEPT_OUTCOME),
        state_reads: &[0, 1],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
    SystemOperation {
        spelling: "tcp_connect",
        regions: &["'a"],
        parameters: &[
            parameter(
                "permit",
                SystemParameterMode::Own,
                SystemTypeRef::Nominal(HANDLE_PERMIT),
            ),
            parameter(
                "address",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(SOCKET_ADDRESS),
            ),
        ],
        result: SystemTypeRef::Nominal(CONNECT_OUTCOME),
        state_reads: &[0, 1],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
    // The two direction transfers [SYS-18]. Each borrows one direction of one
    // connection `&uniq`, so the two directions of one connection are two
    // places that overlap under [PAR-1] with nothing added.
    SystemOperation {
        spelling: "receive_next",
        regions: &["'r", "'d"],
        parameters: &[
            parameter(
                "receive",
                SystemParameterMode::UniqueBorrow(0),
                SystemTypeRef::Nominal(TCP_RECEIVE),
            ),
            parameter(
                "destination",
                SystemParameterMode::UniqueBorrow(1),
                SystemTypeRef::DestinationU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: SystemTypeRef::Nominal(READ_OUTCOME),
        state_reads: &[0, 1],
        state_writes: &[0, 1],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::None,
    },
    SystemOperation {
        spelling: "send_once",
        regions: &["'s", "'b"],
        parameters: &[
            parameter(
                "send",
                SystemParameterMode::UniqueBorrow(0),
                SystemTypeRef::Nominal(TCP_SEND),
            ),
            parameter(
                "source",
                SystemParameterMode::Borrow(1),
                SystemTypeRef::SourceU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: ok_u64(IO_ERROR),
        state_reads: &[0, 1],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::None,
    },
    // The two explicit closes that return the credit [SYS-17, SYS-18].
    // `close_connection` takes the whole pair, because one connection is one
    // credit and the close must name the thing that holds it.
    SystemOperation {
        spelling: "close_connection",
        regions: &[],
        parameters: &[parameter(
            "connection",
            SystemParameterMode::Own,
            SystemTypeRef::Nominal(TCP_CONNECTION),
        )],
        result: SystemTypeRef::Nominal(HANDLE_PERMIT),
        state_reads: &[0],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
    SystemOperation {
        spelling: "close_listener",
        regions: &[],
        parameters: &[parameter(
            "listener",
            SystemParameterMode::Own,
            SystemTypeRef::Nominal(TCP_LISTENER),
        )],
        result: SystemTypeRef::Nominal(HANDLE_PERMIT),
        state_reads: &[0],
        state_writes: &[0],
        integer_result_bound: None,
        target_action: TargetAction::MAY_SUSPEND,
        result_state_origin: SystemResultStateOrigin::Fresh,
    },
];

/// Returns one operation's exact unified state effects as value-parameter
/// ordinals.
pub fn operation_state_effects(operation: &SystemOperation) -> (Vec<u8>, Vec<u8>) {
    (
        operation.state_reads.to_vec(),
        operation.state_writes.to_vec(),
    )
}

/// Compiler-owned execution metadata for a type's derived release action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemReleaseRow {
    /// Whether releasing this value may suspend its continuation.
    pub target_action: TargetAction,
    /// Whether release changes the released value's state.
    pub state_write: bool,
}

impl SystemReleaseRow {
    /// The empty release row: a logical consume or detach with no host call.
    pub const EMPTY: Self = Self {
        target_action: TargetAction::INLINE,
        state_write: false,
    };

    /// Conservatively combines release metadata from owned components.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self {
            target_action: self.target_action.union(other.target_action),
            state_write: self.state_write || other.state_write,
        }
    }
}

/// One [SYS-2] opaque system resource type, by its target-independent
/// semantic identity rather than by any source spelling [QUAL-1].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SystemResourceType {
    /// The command's immutable argument value [SYS-9].
    Args,
    /// One lossless host string [HOST-1, SYS-9].
    HostString,
    /// One relative path admitted by construction from a host string [PATH-1].
    RelativePath,
    /// One directory-read state object [PATH-2, SYS-10].
    DirectoryRead,
    /// One shareable open file for positioned reads [SYS-11].
    ReadFile,
    /// One stateful output sink [SYS-12].
    OutputStream,
    /// One immutable portable command code [SYS-13].
    ExitStatus,
    /// One ordered directory-entry source [SYS-14].
    DirectorySource,
    /// One explicit source of one-shot handle permits.
    HandleFactory,
    /// One affine, one-shot authorization for a single handle-taking attempt.
    HandlePermit,
    /// One readable byte stream with an implicit position [SYS-15].
    InputStream,
    /// One immutable internet address and port [SYS-16].
    SocketAddress,
    /// One bound, listening TCP endpoint [SYS-17].
    TcpListener,
    /// The receiving direction of one TCP connection [SYS-18].
    TcpReceive,
    /// The sending direction of one TCP connection [SYS-18].
    TcpSend,
}

/// The [SYS-5] consuming release action of one system resource type.
///
/// The release table fixes exactly one action per type, and one type carries
/// exactly one release action [HOST-3].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SystemReleaseAction {
    /// A logical consume: no host call, no target call, no handle lookup, no
    /// byte copy, and no external effect.
    LogicalConsume,
    /// At most one native close attempt. It discards only the close
    /// diagnostic and never retries an ambiguous close, because the native
    /// descriptor may already be closed and reusable.
    NativeCloseAttempt,
    /// `OutputStream`'s and `InputStream`'s logical source detach: it neither
    /// closes, flushes, nor drains the host descriptor [SYS-12, SYS-15], and
    /// operating-system process teardown closes the native descriptor
    /// afterwards.
    SourceDetach,
    /// At most one native direction-close attempt: the half-close of one
    /// direction of one connection, with the same discarded diagnostic and the
    /// same no-retry rule as a close. The target releases the underlying
    /// object once both directions have been released, and that second release
    /// is the one that spends the credit [SYS-18].
    NativeDirectionCloseAttempt,
}

/// How one system resource value is backed.
///
/// This is retained for auditing and lowering [DIAG-2]; it is no
/// source-acceptance judgment and refuses no program [HOST-3].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SystemResourceBacking {
    /// An opaque compiler-owned resource with no source-visible backing.
    Opaque,
    /// [HOST-3]: an inline lease — a private code-unit address and length
    /// carried in the value itself — over the command-lifetime argument
    /// snapshot [QUAL-2]. A lease owns no code-unit storage and several live
    /// leases may denote the same backing code units, so lease identity is
    /// retained rather than inferred from value separateness.
    CommandLifetimeLease,
}

/// The complete [SYS-5]/[HOST-3] contract of one system resource type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemResourceContract {
    /// The type's target-independent semantic identity.
    pub resource: SystemResourceType,
    /// Its one consuming release action.
    pub action: SystemReleaseAction,
    /// That action's fixed effect row.
    pub row: SystemReleaseRow,
    /// How its values are backed.
    pub backing: SystemResourceBacking,
}

/// One compiler-derived release, as [STOR-3] and [SYS-5] fix it.
///
/// `action` is the release action of the released value itself when that
/// value is one system resource, and `row` is the union of the [SYS-5] rows
/// of every system release the value may run over owned content: release of
/// an outcome value is release of its components, and a `buffer`, `box`,
/// arena, or `const` release carries the empty row and no system action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemRelease {
    /// The released value's own [SYS-5] action, when it is a system resource.
    pub action: Option<SystemReleaseAction>,
    /// The union of every [SYS-5] row this release may run.
    pub row: SystemReleaseRow,
}

impl SystemRelease {
    /// A release that runs no system release action at all.
    pub const NONE: Self = Self {
        action: None,
        row: SystemReleaseRow::EMPTY,
    };
}

/// Returns one system nominal's complete [SYS-5]/[HOST-3] resource contract.
///
/// The eight outcome enums have no release action and take no row in the
/// [SYS-5] table, so they carry no contract here; their release is the
/// release of their components.
#[must_use]
pub fn system_resource_contract(nominal: u8) -> Option<SystemResourceContract> {
    let (resource, action, backing) = match nominal {
        ARGS => (
            SystemResourceType::Args,
            SystemReleaseAction::LogicalConsume,
            SystemResourceBacking::Opaque,
        ),
        HOST_STRING => (
            SystemResourceType::HostString,
            SystemReleaseAction::LogicalConsume,
            SystemResourceBacking::CommandLifetimeLease,
        ),
        RELATIVE_PATH => (
            SystemResourceType::RelativePath,
            SystemReleaseAction::LogicalConsume,
            SystemResourceBacking::CommandLifetimeLease,
        ),
        DIRECTORY_READ => (
            SystemResourceType::DirectoryRead,
            SystemReleaseAction::NativeCloseAttempt,
            SystemResourceBacking::Opaque,
        ),
        READ_FILE => (
            SystemResourceType::ReadFile,
            SystemReleaseAction::NativeCloseAttempt,
            SystemResourceBacking::Opaque,
        ),
        OUTPUT => (
            SystemResourceType::OutputStream,
            SystemReleaseAction::SourceDetach,
            SystemResourceBacking::Opaque,
        ),
        EXIT_STATUS => (
            SystemResourceType::ExitStatus,
            SystemReleaseAction::LogicalConsume,
            SystemResourceBacking::Opaque,
        ),
        DIRECTORY_SOURCE => (
            SystemResourceType::DirectorySource,
            SystemReleaseAction::NativeCloseAttempt,
            SystemResourceBacking::Opaque,
        ),
        HANDLE_FACTORY => (
            SystemResourceType::HandleFactory,
            SystemReleaseAction::LogicalConsume,
            SystemResourceBacking::Opaque,
        ),
        HANDLE_PERMIT => (
            SystemResourceType::HandlePermit,
            SystemReleaseAction::LogicalConsume,
            SystemResourceBacking::Opaque,
        ),
        INPUT_STREAM => (
            SystemResourceType::InputStream,
            SystemReleaseAction::SourceDetach,
            SystemResourceBacking::Opaque,
        ),
        SOCKET_ADDRESS => (
            SystemResourceType::SocketAddress,
            SystemReleaseAction::LogicalConsume,
            SystemResourceBacking::Opaque,
        ),
        TCP_LISTENER => (
            SystemResourceType::TcpListener,
            SystemReleaseAction::NativeCloseAttempt,
            SystemResourceBacking::Opaque,
        ),
        TCP_RECEIVE => (
            SystemResourceType::TcpReceive,
            SystemReleaseAction::NativeDirectionCloseAttempt,
            SystemResourceBacking::Opaque,
        ),
        TCP_SEND => (
            SystemResourceType::TcpSend,
            SystemReleaseAction::NativeDirectionCloseAttempt,
            SystemResourceBacking::Opaque,
        ),
        // `TcpConnection` is a system struct, not a resource type: it takes no
        // row in the [SYS-5] release table, and releasing one is releasing its
        // two fields [SYS-18].
        _ => return None,
    };
    let row = match action {
        // Only a native close attempt reaches the host; a logical consume and
        // a source detach make no target call and perform no external effect.
        SystemReleaseAction::NativeCloseAttempt
        | SystemReleaseAction::NativeDirectionCloseAttempt => SystemReleaseRow {
            target_action: TargetAction::MAY_SUSPEND,
            state_write: true,
        },
        SystemReleaseAction::LogicalConsume | SystemReleaseAction::SourceDetach => {
            SystemReleaseRow::EMPTY
        }
    };
    Some(SystemResourceContract {
        resource,
        action,
        row,
        backing,
    })
}

/// Returns one system nominal's exact [SYS-5] release row.
///
/// `DirectoryRead`, `ReadFile`, and `DirectorySource` release with
/// at most one native close attempt with `may-suspend` target metadata and an
/// exclusive whole-root state write; every other system
/// type — the remaining
/// opaque types' logical consume or detach and every outcome enum, which has
/// no release action and takes no row in the [SYS-5] table — carries the
/// empty row.
#[must_use]
pub fn system_release_row(nominal: u8) -> SystemReleaseRow {
    match system_resource_contract(nominal) {
        Some(contract) => contract.row,
        None => SystemReleaseRow::EMPTY,
    }
}

/// How many declaration records one inventory's nominal-type block occupies.
///
/// A nominal-type row occupies one record, and a struct nominal's own field
/// records follow it immediately [SYS-2], so this is no longer the count of
/// nominal types.
fn nominal_record_count(inventory: Inventory) -> usize {
    system_nominals(inventory)
        .iter()
        .map(|nominal| 1 + nominal.fields.len())
        .sum()
}

/// Maps one lookup-class [SYS-2] declaration to its nominal-table index.
///
/// Returns `None` for a struct nominal's owner-local field ordinal, which
/// never enters source lookup.
#[must_use]
pub fn system_nominal_index(id: SystemDeclarationId, inventory: Inventory) -> Option<u8> {
    let mut ordinal = usize::from(id.ordinal());
    for (index, nominal) in system_nominals(inventory).iter().enumerate() {
        if ordinal == 0 {
            return u8::try_from(index).ok();
        }
        ordinal -= 1;
        if ordinal < nominal.fields.len() {
            return None;
        }
        ordinal -= nominal.fields.len();
    }
    None
}

/// Maps one lookup-class [SYS-2] declaration to its constructor-table index.
#[must_use]
pub fn system_constructor_index(id: SystemDeclarationId, inventory: Inventory) -> Option<u8> {
    let mut ordinal = usize::from(id.ordinal());
    let nominals = nominal_record_count(inventory);
    if ordinal < nominals {
        return None;
    }
    ordinal -= nominals;
    for (index, constructor) in system_constructors(inventory).iter().enumerate() {
        if ordinal == 0 {
            return u8::try_from(index).ok();
        }
        ordinal -= 1;
        if ordinal < constructor.fields.len() {
            return None;
        }
        ordinal -= constructor.fields.len();
    }
    None
}

/// Maps one constructor-table index to its [SYS-2] declaration identity.
#[must_use]
pub fn system_constructor_declaration(
    index: u8,
    inventory: Inventory,
) -> Option<SystemDeclarationId> {
    let mut ordinal = nominal_record_count(inventory);
    for (constructor_index, constructor) in system_constructors(inventory).iter().enumerate() {
        if constructor_index == usize::from(index) {
            return u16::try_from(ordinal).ok().map(SystemDeclarationId::new);
        }
        ordinal += 1 + constructor.fields.len();
    }
    None
}

/// Maps one lookup-class [SYS-2] declaration to its operation-table index.
#[must_use]
pub fn system_operation_index(id: SystemDeclarationId, inventory: Inventory) -> Option<u8> {
    let mut ordinal = usize::from(id.ordinal());
    let nominals = nominal_record_count(inventory);
    if ordinal < nominals {
        return None;
    }
    ordinal -= nominals;
    for constructor in system_constructors(inventory) {
        ordinal = ordinal.checked_sub(1 + constructor.fields.len())?;
    }
    for (index, operation) in system_operations(inventory).iter().enumerate() {
        if ordinal == 0 {
            return u8::try_from(index).ok();
        }
        ordinal -= 1;
        let locals = operation.regions.len() + operation.parameters.len();
        if ordinal < locals {
            return None;
        }
        ordinal -= locals;
    }
    None
}

/// Builds the [SYS-2] declaration records in normative preorder: each nominal
/// type in table order; then each constructor and its fields in declared
/// order; then each operation, its region parameters, and its value
/// parameters in declared order. Exactly the nominal types, constructors, and
/// operations carry a lookup class; fields and parameters are owner-local
/// records with none.
///
/// The active specification's complete inventory is three hundred and seven
/// records; the retained prefix states are smaller exact table prefixes.
pub(crate) fn system_declarations(inventory: Inventory) -> Vec<SystemDeclarationRecord> {
    let mut records = Vec::with_capacity(307);
    let push = |spelling: &'static str, class: Option<DeclarationClass>, records: &mut Vec<_>| {
        let Ok(ordinal) = u16::try_from(records.len()) else {
            unreachable!("the closed SYS-2 inventory fits two bytes of ordinals");
        };
        records.push(SystemDeclarationRecord {
            id: SystemDeclarationId::new(ordinal),
            spelling,
            class,
        });
    };
    for nominal in system_nominals(inventory) {
        push(
            nominal.spelling,
            Some(DeclarationClass::NominalType),
            &mut records,
        );
        // A system struct's field records follow its own row immediately and
        // carry no lookup class: they are owner-local to the nominal, exactly
        // as a constructor's fields are owner-local to the constructor
        // [SYS-2, SYS-18].
        for field in nominal.fields {
            push(field.name, None, &mut records);
        }
    }
    for constructor in system_constructors(inventory) {
        push(
            constructor.spelling,
            Some(DeclarationClass::EnumVariant),
            &mut records,
        );
        for field in constructor.fields {
            push(field.name, None, &mut records);
        }
    }
    for operation in system_operations(inventory) {
        push(
            operation.spelling,
            Some(DeclarationClass::Function),
            &mut records,
        );
        for region in operation.regions {
            push(region, None, &mut records);
        }
        for value_parameter in operation.parameters {
            push(value_parameter.name, None, &mut records);
        }
    }
    records
}

/// Maps one lookup-class [SYS-2] ordinal to its normative entity.
///
/// Returns `None` for an owner-local field, region-parameter, or
/// value-parameter ordinal, which never enters source lookup.
pub fn system_entity(id: SystemDeclarationId, inventory: Inventory) -> Option<SystemEntity> {
    let mut ordinal = usize::from(id.ordinal());
    for nominal in system_nominals(inventory) {
        if ordinal == 0 {
            return Some(SystemEntity::Nominal(nominal));
        }
        ordinal -= 1;
        if ordinal < nominal.fields.len() {
            return None;
        }
        ordinal -= nominal.fields.len();
    }
    for constructor in system_constructors(inventory) {
        if ordinal == 0 {
            return Some(SystemEntity::Constructor(constructor));
        }
        ordinal -= 1;
        if ordinal < constructor.fields.len() {
            return None;
        }
        ordinal -= constructor.fields.len();
    }
    for operation in system_operations(inventory) {
        if ordinal == 0 {
            return Some(SystemEntity::Operation(operation));
        }
        ordinal -= 1;
        let locals = operation.regions.len() + operation.parameters.len();
        if ordinal < locals {
            return None;
        }
        ordinal -= locals;
    }
    None
}

pub(crate) fn reserved_name(spelling: &str) -> Option<(ReservedNameClass, u16)> {
    if !spelling.contains('.')
        && let Some(index) = OPERATION_FAMILIES
            .iter()
            .position(|candidate| *candidate == spelling)
    {
        return u16::try_from(index)
            .ok()
            .map(|ordinal| (ReservedNameClass::DotlessOperation, ordinal));
    }
    MODE_WORDS
        .iter()
        .position(|candidate| *candidate == spelling)
        .and_then(|index| u16::try_from(index).ok())
        .map(|ordinal| (ReservedNameClass::ModeWord, ordinal))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        DeclarationClass, Inventory, MODE_WORDS, OPERATION_FAMILIES, PRELUDE_DECLARATIONS,
        ReservedNameClass, SYSTEM_CONSTRUCTORS, SYSTEM_NOMINALS, SYSTEM_OPERATIONS,
        SystemDeclarationId, SystemEntity, SystemNominalCategory, SystemParameterMode,
        SystemResultPayload, SystemTypeRef, operation_state_effects, reserved_name,
        system_constructors, system_declarations, system_entity, system_nominals,
        system_operations,
    };

    #[test]
    fn system_inventory_matches_the_sys2_counted_totals() {
        // [SYS-2]: fourteen nominal types, thirty-nine enum-variant
        // constructors, sixty variant fields, eleven operations, fourteen
        // operation region parameters, and twenty-six operation value
        // parameters — the active open signature adds its one permit record
        // while this legacy membership probe keeps the earlier operation set.
        let nominals = system_nominals(Inventory::Base);
        let constructors = system_constructors(Inventory::Base);
        let operations = system_operations(Inventory::Base);
        assert_eq!(nominals.len(), 14);
        assert_eq!(nominals.iter().filter(|n| n.is_opaque()).count(), 7);
        assert_eq!(constructors.len(), 37);
        assert_eq!(
            constructors
                .iter()
                .map(|constructor| constructor.fields.len())
                .sum::<usize>(),
            60
        );
        assert_eq!(operations.len(), 11);
        assert_eq!(
            operations
                .iter()
                .map(|operation| operation.regions.len())
                .sum::<usize>(),
            14
        );
        assert_eq!(
            operations
                .iter()
                .map(|operation| operation.parameters.len())
                .sum::<usize>(),
            27
        );

        let records = system_declarations(Inventory::Base);
        assert_eq!(records.len(), 163);
        assert!(
            records
                .iter()
                .enumerate()
                .all(|(index, record)| usize::from(record.id().ordinal()) == index)
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.lookup_class() == Some(DeclarationClass::NominalType))
                .count(),
            14
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.lookup_class() == Some(DeclarationClass::EnumVariant))
                .count(),
            37
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.lookup_class() == Some(DeclarationClass::Function))
                .count(),
            11
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.lookup_class().is_none())
                .count(),
            101
        );

        // Deterministic preorder spot checks used by diagnostic origins.
        for (ordinal, spelling) in [
            (0, "Args"),
            (6, "ExitStatus"),
            (7, "ArgError"),
            (13, "IoError"),
            (14, "InvalidIndex"),
            (27, "NotFound"),
            (108, "Other"),
            (111, "args_count"),
            (140, "open_read"),
            (161, "exit_status"),
        ] {
            assert_eq!(records[ordinal].spelling(), spelling, "ordinal {ordinal}");
        }
    }

    #[test]
    fn system_inventory_satisfies_the_sys2_data_properties() {
        // Spellings are unique within each contributed domain and disjoint
        // from the PRE-1 spellings of the same domain; no operation spelling
        // is a member of `ReservedLowerNames`; nominal and constructor
        // spellings satisfy TYPEID and operation spellings satisfy dotless
        // IDENT; field and parameter names are unique within their owner.
        let nominal_spellings: HashSet<_> = SYSTEM_NOMINALS
            .iter()
            .map(|nominal| nominal.spelling)
            .collect();
        assert_eq!(nominal_spellings.len(), SYSTEM_NOMINALS.len());
        let constructor_spellings: HashSet<_> = SYSTEM_CONSTRUCTORS
            .iter()
            .map(|constructor| constructor.spelling)
            .collect();
        assert_eq!(constructor_spellings.len(), SYSTEM_CONSTRUCTORS.len());
        let operation_spellings: HashSet<_> = SYSTEM_OPERATIONS
            .iter()
            .map(|operation| operation.spelling)
            .collect();
        assert_eq!(operation_spellings.len(), SYSTEM_OPERATIONS.len());

        for prelude in PRELUDE_DECLARATIONS {
            match prelude.class {
                Some(DeclarationClass::NominalType) => {
                    assert!(!nominal_spellings.contains(prelude.spelling));
                }
                Some(DeclarationClass::EnumVariant) => {
                    assert!(!constructor_spellings.contains(prelude.spelling));
                }
                _ => {}
            }
        }
        for operation in &SYSTEM_OPERATIONS {
            assert_eq!(reserved_name(operation.spelling), None);
            assert!(!operation.spelling.contains('.'));
            if operation.integer_result_bound.is_some() {
                assert_eq!(operation.result, SystemTypeRef::U64);
            }
            assert!(
                operation
                    .spelling
                    .bytes()
                    .next()
                    .is_some_and(|byte| byte.is_ascii_lowercase())
            );
            let parameter_names: HashSet<_> = operation
                .parameters
                .iter()
                .map(|parameter| parameter.name)
                .collect();
            assert_eq!(parameter_names.len(), operation.parameters.len());
            let region_names: HashSet<_> = operation.regions.iter().copied().collect();
            assert_eq!(region_names.len(), operation.regions.len());
        }
        for spelling in nominal_spellings.iter().chain(&constructor_spellings) {
            assert!(
                spelling
                    .bytes()
                    .next()
                    .is_some_and(|byte| byte.is_ascii_uppercase())
            );
        }
        for constructor in &SYSTEM_CONSTRUCTORS {
            assert!(!SYSTEM_NOMINALS[usize::from(constructor.owner)].is_opaque());
            let field_names: HashSet<_> =
                constructor.fields.iter().map(|field| field.name).collect();
            assert_eq!(field_names.len(), constructor.fields.len());
        }
    }

    #[test]
    fn system_entities_are_recovered_from_preorder_ordinals() {
        let mut nominals = 0;
        let mut constructors = 0;
        let mut operations = 0;
        let mut owner_local = 0;
        for record in system_declarations(Inventory::Base) {
            match (
                record.lookup_class(),
                system_entity(record.id(), Inventory::Base),
            ) {
                (Some(DeclarationClass::NominalType), Some(SystemEntity::Nominal(nominal))) => {
                    assert_eq!(nominal.spelling, record.spelling());
                    nominals += 1;
                }
                (
                    Some(DeclarationClass::EnumVariant),
                    Some(SystemEntity::Constructor(constructor)),
                ) => {
                    assert_eq!(constructor.spelling, record.spelling());
                    constructors += 1;
                }
                (Some(DeclarationClass::Function), Some(SystemEntity::Operation(operation))) => {
                    assert_eq!(operation.spelling, record.spelling());
                    operations += 1;
                }
                (None, None) => owner_local += 1,
                (class, entity) => {
                    panic!("inconsistent record {record:?}: {class:?} vs {entity:?}")
                }
            }
        }
        assert_eq!(
            (nominals, constructors, operations, owner_local),
            (14, 37, 11, 101)
        );
        assert!(system_entity(SystemDeclarationId::new(163), Inventory::Base).is_none());
        assert!(system_entity(SystemDeclarationId::new(u16::MAX), Inventory::Base).is_none());
    }

    /// The v0.32 traversal inventory is the v0.31 inventory plus exactly the
    /// traversal rows, and every preorder ordinal below the first new nominal
    /// keeps its meaning only where the specification's own preorder keeps it:
    /// the two new nominal types shift every constructor and operation
    /// ordinal by two, which is why the switch selects one whole inventory
    /// rather than patching the other.
    #[test]
    fn traversal_inventory_matches_its_counted_totals() {
        let nominals = system_nominals(Inventory::Traversal);
        let constructors = system_constructors(Inventory::Traversal);
        let operations = system_operations(Inventory::Traversal);
        assert_eq!(nominals.len(), 16);
        assert_eq!(nominals.iter().filter(|n| n.is_opaque()).count(), 8);
        assert_eq!(constructors.len(), 40);
        assert_eq!(
            constructors
                .iter()
                .map(|constructor| constructor.fields.len())
                .sum::<usize>(),
            63
        );
        assert_eq!(operations.len(), 14);
        assert_eq!(
            operations
                .iter()
                .map(|operation| operation.regions.len())
                .sum::<usize>(),
            19
        );
        assert_eq!(
            operations
                .iter()
                .map(|operation| operation.parameters.len())
                .sum::<usize>(),
            38
        );

        let records = system_declarations(Inventory::Traversal);
        assert_eq!(records.len(), 190);
        assert!(
            records
                .iter()
                .enumerate()
                .all(|(index, record)| usize::from(record.id().ordinal()) == index)
        );
        for (ordinal, spelling) in [
            (14, "DirectorySource"),
            (15, "ListOutcome"),
            (16, "InvalidIndex"),
            (110, "Other"),
            (113, "ListBytes"),
            (116, "ListEnd"),
            (117, "ListFailed"),
            (119, "args_count"),
            (169, "exit_status"),
            (171, "open_directory"),
            (179, "open_directory_source"),
            (183, "directory_next"),
        ] {
            assert_eq!(records[ordinal].spelling(), spelling, "ordinal {ordinal}");
        }
        let mut nominals = 0;
        let mut constructors = 0;
        let mut operations = 0;
        let mut owner_local = 0;
        for record in system_declarations(Inventory::Traversal) {
            match (
                record.lookup_class(),
                system_entity(record.id(), Inventory::Traversal),
            ) {
                (Some(DeclarationClass::NominalType), Some(SystemEntity::Nominal(nominal))) => {
                    assert_eq!(nominal.spelling, record.spelling());
                    nominals += 1;
                }
                (
                    Some(DeclarationClass::EnumVariant),
                    Some(SystemEntity::Constructor(constructor)),
                ) => {
                    assert_eq!(constructor.spelling, record.spelling());
                    constructors += 1;
                }
                (Some(DeclarationClass::Function), Some(SystemEntity::Operation(operation))) => {
                    assert_eq!(operation.spelling, record.spelling());
                    operations += 1;
                }
                (None, None) => owner_local += 1,
                (class, entity) => {
                    panic!("inconsistent record {record:?}: {class:?} vs {entity:?}")
                }
            }
        }
        assert_eq!(
            (nominals, constructors, operations, owner_local),
            (16, 40, 14, 120)
        );
        assert!(system_entity(SystemDeclarationId::new(190), Inventory::Traversal).is_none());
    }

    /// The active [SYS-11] file-open-by-name row's own counted totals.
    ///
    /// This retained membership probe stops before the active factory and
    /// permit rows. Its open operations nevertheless use the active signature,
    /// so their one permit parameter is included in these counted totals.
    #[test]
    fn open_by_name_candidate_inventory_matches_its_counted_totals() {
        let nominals = system_nominals(Inventory::OpenByName);
        let constructors = system_constructors(Inventory::OpenByName);
        let operations = system_operations(Inventory::OpenByName);
        assert_eq!(nominals.len(), 16);
        assert_eq!(constructors.len(), 40);
        assert_eq!(operations.len(), 15);
        assert_eq!(
            operations
                .iter()
                .map(|operation| operation.regions.len())
                .sum::<usize>(),
            21
        );
        assert_eq!(
            operations
                .iter()
                .map(|operation| operation.parameters.len())
                .sum::<usize>(),
            43
        );

        let records = system_declarations(Inventory::OpenByName);
        assert_eq!(records.len(), 198);
        // Every active-inventory record keeps its exact ordinal and spelling.
        for (ordinal, record) in system_declarations(Inventory::Traversal).iter().enumerate() {
            assert_eq!(records[ordinal].spelling(), record.spelling());
        }
        for (ordinal, spelling) in [
            (190, "open_file"),
            (191, "'c"),
            (192, "'n"),
            (193, "permit"),
            (194, "root"),
            (195, "name"),
            (196, "start"),
            (197, "end"),
        ] {
            assert_eq!(records[ordinal].spelling(), spelling, "ordinal {ordinal}");
        }
        let open_file = SystemDeclarationId::new(190);
        let Some(SystemEntity::Operation(operation)) =
            system_entity(open_file, Inventory::OpenByName)
        else {
            panic!("the active ordinal must name the active operation");
        };
        assert_eq!(operation.spelling, "open_file");
        assert_eq!(operation.target_action, super::TargetAction::MAY_SUSPEND);
        // Off, the same ordinal is past the inventory and names nothing.
        assert!(system_entity(open_file, Inventory::Traversal).is_none());
        assert!(system_entity(SystemDeclarationId::new(198), Inventory::OpenByName).is_none());
    }

    #[test]
    fn file_permit_inventory_matches_the_active_counted_totals() {
        let nominals = system_nominals(Inventory::FilePermits);
        let constructors = system_constructors(Inventory::FilePermits);
        let operations = system_operations(Inventory::FilePermits);
        assert_eq!(nominals.len(), 21);
        assert_eq!(
            nominals
                .iter()
                .filter(|nominal| nominal.is_opaque())
                .count(),
            10
        );
        assert_eq!(constructors.len(), 46);
        assert_eq!(operations.len(), 19);
        assert_eq!(
            operations
                .iter()
                .map(|operation| operation.regions.len())
                .sum::<usize>(),
            22
        );
        assert_eq!(
            operations
                .iter()
                .map(|operation| operation.parameters.len())
                .sum::<usize>(),
            47
        );
        let records = system_declarations(Inventory::FilePermits);
        assert_eq!(records.len(), 227);
        let reserve = SystemDeclarationId::new(218);
        let Some(SystemEntity::Operation(operation)) =
            system_entity(reserve, Inventory::FilePermits)
        else {
            panic!("the final active ordinal must name reserve_handle");
        };
        assert_eq!(operation.spelling, "reserve_handle");
        assert_eq!(operation.target_action, super::TargetAction::INLINE);
        assert!(system_entity(SystemDeclarationId::new(209), Inventory::FilePermits).is_none());
    }

    #[test]
    fn system_inventory_matches_independent_extraction_from_exact() {
        let spec = crate::ACTIVE_KERNEL_SPEC_TEXT;

        // The fifteen opaque nominal types, from the [SYS-2] prose sentence.
        let opaque_sentence = spec
            .split_once("Fifteen opaque nominal types: ")
            .expect("exact SYS-2 opaque sentence")
            .1
            .split_once('.')
            .expect("SYS-2 opaque sentence end")
            .0;
        let opaque: Vec<_> = opaque_sentence
            .split('`')
            .enumerate()
            .filter_map(|(index, part)| (index % 2 == 1).then_some(part))
            .collect();
        assert_eq!(
            opaque,
            system_nominals(Inventory::ACTIVE)
                .iter()
                .filter(|nominal| nominal.is_opaque())
                .map(|nominal| nominal.spelling)
                .collect::<Vec<_>>()
        );

        let sys2 = spec
            .split_once("[SYS-2] The system inventory is exactly:")
            .expect("exact SYS-2 opening")
            .1;

        // The one system-declared struct and its two field records, from the
        // first [SYS-2] code block [SYS-18].
        let struct_block = sys2
            .split_once("One struct nominal type with two owner-local field records:\n\n```\n")
            .expect("SYS-2 struct block opening")
            .1
            .split_once("\n```\n")
            .expect("SYS-2 struct block closing")
            .0;
        let mut extracted_structs: Vec<(String, Vec<(String, String)>)> = Vec::new();
        for line in struct_block.lines() {
            let trimmed = line.trim();
            if let Some(header) = trimmed.strip_prefix("struct ") {
                let name = header.strip_suffix(" {").expect("SYS-2 struct header");
                extracted_structs.push((name.to_owned(), Vec::new()));
            } else if let Some(field) = trimmed.strip_suffix(';') {
                let (name, ty) = field.split_once(": ").expect("SYS-2 struct field");
                extracted_structs
                    .last_mut()
                    .expect("SYS-2 field outside struct")
                    .1
                    .push((name.to_owned(), ty.to_owned()));
            }
        }
        let catalog_structs: Vec<(String, Vec<(String, String)>)> =
            system_nominals(Inventory::ACTIVE)
                .iter()
                .filter(|nominal| nominal.is_struct())
                .map(|nominal| {
                    let fields = nominal
                        .fields
                        .iter()
                        .map(|field| (field.name.to_owned(), render_type(field.ty)))
                        .collect();
                    (nominal.spelling.to_owned(), fields)
                })
                .collect();
        assert_eq!(extracted_structs, catalog_structs);
        // A struct nominal contributes no constructor entry, so no source
        // expression constructs one [SYS-2, SYS-18].
        for (index, nominal) in system_nominals(Inventory::ACTIVE).iter().enumerate() {
            if !nominal.is_struct() {
                continue;
            }
            assert!(
                system_constructors(Inventory::ACTIVE)
                    .iter()
                    .all(|constructor| usize::from(constructor.owner) != index),
                "{} contributes a constructor entry",
                nominal.spelling
            );
        }

        // The fourteen enums, their fifty-two variants, and every field, from
        // the second [SYS-2] code block.
        let enum_block = sys2
            .split_once("Fourteen enum nominal types with fifty-two variant constructors:\n\n```\n")
            .expect("SYS-2 enum block opening")
            .1
            .split_once("\n```\n")
            .expect("SYS-2 enum block closing")
            .0;
        let mut extracted_enums: Vec<ExtractedEnum> = Vec::new();
        for line in enum_block.lines() {
            let trimmed = line.trim();
            if let Some(header) = trimmed.strip_prefix("enum ") {
                let name = header.strip_suffix(" {").expect("SYS-2 enum header");
                extracted_enums.push((name.to_owned(), Vec::new()));
            } else if trimmed.ends_with(");") {
                let (variant, rest) = trimmed.split_once('(').expect("SYS-2 variant");
                let fields = rest.strip_suffix(");").expect("SYS-2 variant ending");
                let fields: Vec<_> = if fields.is_empty() {
                    Vec::new()
                } else {
                    fields
                        .split(", ")
                        .map(|field| {
                            let (name, ty) = field.split_once(": ").expect("SYS-2 field");
                            (name.to_owned(), ty.to_owned())
                        })
                        .collect()
                };
                extracted_enums
                    .last_mut()
                    .expect("SYS-2 variant outside enum")
                    .1
                    .push((variant.to_owned(), fields));
            }
        }
        let catalog_enums: Vec<ExtractedEnum> = system_nominals(Inventory::ACTIVE)
            .iter()
            .enumerate()
            .filter(|(_, nominal)| matches!(nominal.category, SystemNominalCategory::Enum))
            .map(|(owner, nominal)| {
                let variants = system_constructors(Inventory::ACTIVE)
                    .iter()
                    .filter(|constructor| usize::from(constructor.owner) == owner)
                    .map(|constructor| {
                        let fields = constructor
                            .fields
                            .iter()
                            .map(|field| (field.name.to_owned(), render_type(field.ty)))
                            .collect();
                        (constructor.spelling.to_owned(), fields)
                    })
                    .collect();
                (nominal.spelling.to_owned(), variants)
            })
            .collect();
        assert_eq!(extracted_enums, catalog_enums);

        // The twenty-nine complete operation signatures, from the third
        // [SYS-2] code block, including each written effect row.
        let operation_block = sys2
            .split_once("`fn_sig` shape:\n\n```\n")
            .expect("SYS-2 operation block opening")
            .1
            .split_once("\n```\n")
            .expect("SYS-2 operation block closing")
            .0;
        let extracted_operations: Vec<_> = operation_block
            .lines()
            .map(|line| line.strip_prefix("fn ").expect("SYS-2 operation line"))
            .map(str::to_owned)
            .collect();
        let catalog_operations: Vec<_> = system_operations(Inventory::ACTIVE)
            .iter()
            .map(render_operation)
            .collect();
        assert_eq!(extracted_operations, catalog_operations);
    }

    /// One extracted enum: its name, then each variant with its named
    /// and typed fields, exactly as the [SYS-2] block writes them.
    type ExtractedEnum = (String, Vec<(String, Vec<(String, String)>)>);

    /// Renders one catalog type exactly as [SYS-2] writes it.
    fn render_type(ty: SystemTypeRef) -> String {
        match ty {
            SystemTypeRef::U8 => "u8".to_owned(),
            SystemTypeRef::U16 => "u16".to_owned(),
            SystemTypeRef::U32 => "u32".to_owned(),
            SystemTypeRef::U64 => "u64".to_owned(),
            SystemTypeRef::DestinationU8 => "MutSlice<u8>".to_owned(),
            SystemTypeRef::SourceU8 => "Slice<u8>".to_owned(),
            SystemTypeRef::Nominal(index) => {
                SYSTEM_NOMINALS[usize::from(index)].spelling.to_owned()
            }
            SystemTypeRef::Result { ok, err } => {
                let ok = match ok {
                    SystemResultPayload::U64 => "u64".to_owned(),
                    SystemResultPayload::Nominal(index) => {
                        SYSTEM_NOMINALS[usize::from(index)].spelling.to_owned()
                    }
                };
                format!(
                    "Result<{ok}, {}>",
                    SYSTEM_NOMINALS[usize::from(err)].spelling
                )
            }
        }
    }

    /// Renders one catalog signature exactly as [SYS-2] writes it, with the
    /// `reads`/`writes` entries produced by the mechanical mode derivation
    /// rather than a stored row.
    fn render_operation(operation: &super::SystemOperation) -> String {
        let mut rendered = operation.spelling.to_owned();
        // [FORM-8] every system operation's region occupies exactly one
        // parameter position and no output position, so the declaration
        // writes no `region_params` and no parameter writes a region name.
        rendered.push('(');
        let parameters: Vec<_> = operation
            .parameters
            .iter()
            .map(|parameter| {
                let mode = match parameter.mode {
                    SystemParameterMode::Own => "own ".to_owned(),
                    SystemParameterMode::Borrow(_) => "&".to_owned(),
                    SystemParameterMode::UniqueBorrow(_) => "&uniq ".to_owned(),
                };
                format!("{}: {mode}{}", parameter.name, render_type(parameter.ty))
            })
            .collect();
        rendered.push_str(&parameters.join(", "));
        rendered.push_str(") -> result: own ");
        rendered.push_str(&render_type(operation.result));
        let (reads, writes) = operation_state_effects(operation);
        let mut effects = Vec::new();
        if !reads.is_empty() {
            let subjects: Vec<_> = reads
                .iter()
                .map(|ordinal| operation.parameters[usize::from(*ordinal)].name)
                .collect();
            effects.push(format!("reads({})", subjects.join(", ")));
        }
        if !writes.is_empty() {
            let subjects: Vec<_> = writes
                .iter()
                .map(|ordinal| operation.parameters[usize::from(*ordinal)].name)
                .collect();
            effects.push(format!("writes({})", subjects.join(", ")));
        }
        if effects.is_empty() {
            effects.push("pure".to_owned());
        }
        rendered.push(' ');
        rendered.push_str(&effects.join(", "));
        rendered.push(';');
        rendered
    }

    #[test]
    fn exact_catalogs_are_closed_and_unique_where_required() {
        assert_eq!(PRELUDE_DECLARATIONS.len(), 24);
        assert_eq!(OPERATION_FAMILIES.len(), 98);
        assert_eq!(
            OPERATION_FAMILIES
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .len(),
            OPERATION_FAMILIES.len()
        );
        assert!(
            MODE_WORDS
                .iter()
                .all(|word| !OPERATION_FAMILIES.contains(word))
        );
        // OP-1's derived-set consequence of the v0.41 comparison symbols:
        // the six integer comparisons are operator spellings, so they occupy
        // their contiguous family ordinals but are no longer dotless names,
        // and the retired names are free identifiers.
        for (spelling, ordinal) in [
            ("==", 22),
            ("!=", 23),
            ("<", 24),
            ("<=", 25),
            (">", 26),
            (">=", 27),
        ] {
            assert_eq!(
                OPERATION_FAMILIES[ordinal], spelling,
                "{spelling} occupies family ordinal {ordinal}"
            );
        }
        for retired in ["ieq", "ine", "ilt", "ile", "igt", "ige"] {
            assert_eq!(
                reserved_name(retired),
                None,
                "{retired} is a free identifier"
            );
        }
        assert_eq!(
            reserved_name("cvt"),
            Some((ReservedNameClass::DotlessOperation, 44))
        );
        assert_eq!(
            reserved_name("wrap"),
            Some((ReservedNameClass::ModeWord, 0))
        );
    }

    #[test]
    fn catalogs_match_independent_extraction_from_exact() {
        let extracted_prelude = extract_prelude_records(crate::ACTIVE_KERNEL_SPEC_TEXT);
        let catalog_prelude: Vec<_> = PRELUDE_DECLARATIONS
            .iter()
            .map(|record| (record.spelling.to_owned(), record.class))
            .collect();
        assert_eq!(catalog_prelude, extracted_prelude);

        assert_eq!(
            OPERATION_FAMILIES.as_slice(),
            extract_operation_families(crate::ACTIVE_KERNEL_SPEC_TEXT)
        );
    }

    fn extract_prelude_records(spec: &str) -> Vec<(String, Option<DeclarationClass>)> {
        let block = spec
            .split_once("[PRE-1] The prelude is exactly:\n\n```\n")
            .expect("exact PRE-1 opening")
            .1
            .split_once("\n```\n")
            .expect("exact PRE-1 closing")
            .0;
        let mut records = Vec::new();
        let mut in_enum = false;
        for line in block.lines() {
            let trimmed = line.trim();
            if let Some(header) = trimmed.strip_prefix("enum ") {
                in_enum = true;
                let name_end = header
                    .find(['<', ' '])
                    .expect("PRE-1 enum header terminator");
                records.push((
                    header[..name_end].to_owned(),
                    Some(DeclarationClass::NominalType),
                ));
                if let Some(generics) = header
                    .split_once('<')
                    .and_then(|(_, rest)| rest.split_once('>'))
                    .map(|(generics, _)| generics)
                {
                    records.extend(
                        generics
                            .split(',')
                            .map(|generic| (generic.trim().to_owned(), None)),
                    );
                }
            } else if in_enum && trimmed == "}" {
                in_enum = false;
            } else if in_enum && trimmed.ends_with(");") {
                let (variant, rest) = trimmed.split_once('(').expect("PRE-1 variant declaration");
                records.push((variant.to_owned(), Some(DeclarationClass::EnumVariant)));
                let fields = rest
                    .strip_suffix(");")
                    .expect("PRE-1 variant declaration ending");
                if !fields.is_empty() {
                    records.extend(fields.split(',').map(|field| {
                        let (name, _) = field
                            .split_once(':')
                            .expect("PRE-1 variant field declaration");
                        (name.trim().to_owned(), None)
                    }));
                }
            } else if let Some(contract) = trimmed.strip_prefix("contract ") {
                records.push((
                    contract
                        .strip_suffix(" {")
                        .expect("PRE-1 contract header")
                        .to_owned(),
                    Some(DeclarationClass::Contract),
                ));
            }
        }
        records
    }

    fn extract_operation_families(spec: &str) -> Vec<&str> {
        let operation_section = spec.split_once("[OP-1]").expect("exact OP-1 opening").1;
        let mut rows = operation_section
            .lines()
            .skip_while(|line| !line.starts_with("| op |"))
            .skip(2);
        let mut seen = HashSet::new();
        let mut operations = Vec::new();
        for row in rows.by_ref().take_while(|line| line.starts_with("| `")) {
            let op_cell = row
                .strip_prefix('|')
                .and_then(|rest| rest.split_once('|'))
                .map(|(cell, _)| cell)
                .expect("OP-1 operation cell");
            for (index, part) in op_cell.split('`').enumerate() {
                if index % 2 == 1 && seen.insert(part) {
                    operations.push(part);
                }
            }
        }
        operations
    }
}
