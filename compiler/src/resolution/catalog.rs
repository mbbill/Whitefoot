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
pub(crate) const OPERATION_FAMILIES: [&str; 94] = [
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
    "ieq",
    "ine",
    "ilt",
    "ile",
    "igt",
    "ige",
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
    "len",
    "slice_of",
    "box_new",
    "arena_new",
    "array_new",
    "buffer_new",
    "buffer_vacant",
    "buffer_fits",
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

/// One [SYS-2] system nominal type in normative table order.
#[derive(Clone, Copy, Debug)]
pub struct SystemNominal {
    /// Exact TYPEID spelling.
    pub spelling: &'static str,
    /// `true` for the seven opaque types, `false` for the seven outcome enums.
    pub opaque: bool,
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
    /// Fixed `external` classification from the written [SYS-2] row.
    pub external: bool,
    /// Fixed `blocks` classification from the written [SYS-2] row.
    pub blocks: bool,
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
    /// `u32`.
    U32,
    /// `u64`.
    U64,
    /// `buffer<u8>`.
    BufferU8,
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
const DIRECTORY_LIST: u8 = 14;
const LIST_OUTCOME: u8 = 15;

/// The traversal surface switch [SYS-2, SYS-14], activated as v0.32.
///
/// `false` admits exactly the v0.31 inventory: the traversal rows below are
/// unreachable, every declaration ordinal keeps its v0.31 value, and the
/// resolver, checker, and backend see the same one hundred sixty-seven
/// records they saw before. `true` admits the directory-enumeration
/// surface — `DirectoryList`, `ListOutcome`, `open_directory`, `open_list`,
/// and `list_once` — as the last row of each [SYS-2] table. It is now `true`,
/// because v0.32 is the active specification; `false` stays reachable as the
/// differential against the base tables.
pub const TRAVERSAL_SURFACE: bool = true;

/// The v0.33-candidate file-open-by-name switch [SYS-2, SYS-11].
///
/// `false` admits exactly the active v0.32 inventory: the `open_file` row
/// below is unreachable, every declaration ordinal keeps its v0.32 value, and
/// the resolver, checker, and backend see the same one hundred ninety-two
/// records they see today. `true` admits the candidate operation — the
/// `open_read` sibling that takes a caller-owned single path component
/// instead of a `RelativePath` — as the last row of the [SYS-2] operation
/// table. It is `false` until the owner approves the candidate bytes; the
/// candidate inventory is reachable only by naming it at a compilation, which
/// is what the end-to-end evidence does.
pub const OPEN_BY_NAME: bool = true;

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
    /// The active v0.32 inventory: [`Inventory::Base`] plus the [SYS-14]
    /// traversal surface.
    Traversal,
    /// The v0.33 candidate inventory: [`Inventory::Traversal`] plus the
    /// [SYS-11] `open_file` operation.
    OpenByName,
}

impl Inventory {
    /// The inventory the shipped compilation path selects, fixed by the two
    /// switches above and read once, by `compile` and `resolve`.
    pub const ACTIVE: Self = if OPEN_BY_NAME {
        Self::OpenByName
    } else if TRAVERSAL_SURFACE {
        Self::Traversal
    } else {
        Self::Base
    };

    /// How many [`SYSTEM_NOMINALS`] rows this state admits.
    const fn nominals(self) -> usize {
        match self {
            Self::Base => BASE_NOMINALS,
            Self::Traversal | Self::OpenByName => SYSTEM_NOMINALS.len(),
        }
    }

    /// How many [`SYSTEM_CONSTRUCTORS`] rows this state admits.
    const fn constructors(self) -> usize {
        match self {
            Self::Base => BASE_CONSTRUCTORS,
            Self::Traversal | Self::OpenByName => SYSTEM_CONSTRUCTORS.len(),
        }
    }

    /// How many [`SYSTEM_OPERATIONS`] rows this state admits.
    const fn operations(self) -> usize {
        match self {
            Self::Base => BASE_OPERATIONS,
            Self::Traversal => TRAVERSAL_OPERATIONS,
            Self::OpenByName => SYSTEM_OPERATIONS.len(),
        }
    }
}

/// The v0.31 nominal-type count: the prefix of [`SYSTEM_NOMINALS`] the v0.31
/// specification declared.
const BASE_NOMINALS: usize = 14;
/// The v0.31 constructor count: the prefix of [`SYSTEM_CONSTRUCTORS`] the
/// v0.31 specification declared.
const BASE_CONSTRUCTORS: usize = 39;
/// The v0.31 operation count: the prefix of [`SYSTEM_OPERATIONS`] the v0.31
/// specification declared.
const BASE_OPERATIONS: usize = 11;
/// The v0.32 operation count: the prefix of [`SYSTEM_OPERATIONS`] the active
/// specification declares.
const TRAVERSAL_OPERATIONS: usize = 14;

/// The [SYS-2] nominal types in normative table order.
///
/// The first fourteen are the active specification's; the last two are the
/// traversal-surface candidate's and are admitted only under
/// [`TRAVERSAL_SURFACE`].
pub const SYSTEM_NOMINALS: [SystemNominal; 16] = [
    nominal("Args", true),
    nominal("HostString", true),
    nominal("RelativePath", true),
    nominal("DirectoryRead", true),
    nominal("ReadFile", true),
    nominal("Output", true),
    nominal("ExitStatus", true),
    nominal("ArgError", false),
    nominal("Utf8Error", false),
    nominal("CopyError", false),
    nominal("Utf8CopyError", false),
    nominal("PathError", false),
    nominal("ReadOutcome", false),
    nominal("IoError", false),
    nominal("DirectoryList", true),
    nominal("ListOutcome", false),
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

const fn nominal(spelling: &'static str, opaque: bool) -> SystemNominal {
    SystemNominal { spelling, opaque }
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
pub const SYSTEM_CONSTRUCTORS: [SystemConstructor; 42] = [
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
    io_class("Interrupted"),
    io_class("WouldBlock"),
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
/// file-open-by-name candidate's, admitted only under [`OPEN_BY_NAME`].
///
/// Each row registers the declared region parameters, value parameters, result
/// type, and the fixed `external`/`blocks` classification. System operations
/// cannot exhibit the `traps` effect: their partial domains are static
/// call-site obligations and fallible host results are explicit outcomes. The
/// `reads`/`writes` region entries are not stored: [SYS-2] fixes them as a
/// mechanical derivation from the parameter modes — every borrow of region
/// `'r` contributes `reads('r)`, and every `&uniq 'r` parameter (each one is
/// changed by its operation in this inventory) additionally contributes
/// `writes('r)` — which [`operation_region_effects`] performs.
pub const SYSTEM_OPERATIONS: [SystemOperation; 15] = [
    SystemOperation {
        spelling: "args_count",
        regions: &["'a"],
        parameters: &[parameter(
            "args",
            SystemParameterMode::Borrow(0),
            SystemTypeRef::Nominal(ARGS),
        )],
        result: SystemTypeRef::U64,
        external: false,
        blocks: false,
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
        external: false,
        blocks: false,
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
        external: false,
        blocks: false,
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
                SystemTypeRef::BufferU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: ok_u64(COPY_ERROR),
        external: false,
        blocks: false,
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
        external: false,
        blocks: false,
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
                SystemTypeRef::BufferU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: ok_u64(UTF8_COPY_ERROR),
        external: false,
        blocks: false,
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
        external: false,
        blocks: false,
    },
    SystemOperation {
        spelling: "open_read",
        regions: &["'c", "'p"],
        parameters: &[
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
        result: ok_nominal(READ_FILE, IO_ERROR),
        external: true,
        blocks: true,
    },
    SystemOperation {
        spelling: "read_once",
        regions: &["'f", "'d"],
        parameters: &[
            parameter(
                "file",
                SystemParameterMode::UniqueBorrow(0),
                SystemTypeRef::Nominal(READ_FILE),
            ),
            parameter(
                "destination",
                SystemParameterMode::UniqueBorrow(1),
                SystemTypeRef::BufferU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: SystemTypeRef::Nominal(READ_OUTCOME),
        external: true,
        blocks: true,
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
                SystemTypeRef::BufferU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: ok_u64(IO_ERROR),
        external: true,
        blocks: true,
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
        external: false,
        blocks: false,
    },
    // The three traversal-surface candidate rows [SYS-14].
    SystemOperation {
        spelling: "open_directory",
        regions: &["'c", "'n"],
        parameters: &[
            parameter(
                "root",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(DIRECTORY_READ),
            ),
            parameter(
                "name",
                SystemParameterMode::Borrow(1),
                SystemTypeRef::BufferU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: ok_nominal(DIRECTORY_READ, IO_ERROR),
        external: true,
        blocks: true,
    },
    SystemOperation {
        spelling: "open_list",
        regions: &["'c"],
        parameters: &[parameter(
            "directory",
            SystemParameterMode::Borrow(0),
            SystemTypeRef::Nominal(DIRECTORY_READ),
        )],
        result: ok_nominal(DIRECTORY_LIST, IO_ERROR),
        external: true,
        blocks: true,
    },
    SystemOperation {
        spelling: "list_once",
        regions: &["'l", "'d"],
        parameters: &[
            parameter(
                "list",
                SystemParameterMode::UniqueBorrow(0),
                SystemTypeRef::Nominal(DIRECTORY_LIST),
            ),
            parameter(
                "destination",
                SystemParameterMode::UniqueBorrow(1),
                SystemTypeRef::BufferU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: SystemTypeRef::Nominal(LIST_OUTCOME),
        external: true,
        blocks: true,
    },
    // The file-open-by-name candidate row [SYS-11]: `open_read`'s sibling
    // over a caller-owned single path component, taking exactly the name
    // range `open_directory` takes.
    SystemOperation {
        spelling: "open_file",
        regions: &["'c", "'n"],
        parameters: &[
            parameter(
                "root",
                SystemParameterMode::Borrow(0),
                SystemTypeRef::Nominal(DIRECTORY_READ),
            ),
            parameter(
                "name",
                SystemParameterMode::Borrow(1),
                SystemTypeRef::BufferU8,
            ),
            parameter("start", SystemParameterMode::Own, SystemTypeRef::U64),
            parameter("end", SystemParameterMode::Own, SystemTypeRef::U64),
        ],
        result: ok_nominal(READ_FILE, IO_ERROR),
        external: true,
        blocks: true,
    },
];

/// Derives one operation's `reads`/`writes` region entries from its modes.
///
/// Returns the zero-based declared region-parameter indices carried by
/// `reads` and by `writes`, each in declared region-parameter order — the
/// mechanical [SYS-2] derivation, not a hand-curated table.
pub fn operation_region_effects(operation: &SystemOperation) -> (Vec<u8>, Vec<u8>) {
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    for region in 0..operation.regions.len() {
        let Ok(region) = u8::try_from(region) else {
            continue;
        };
        let borrowed = operation.parameters.iter().any(|parameter| {
            matches!(
                parameter.mode,
                SystemParameterMode::Borrow(index) | SystemParameterMode::UniqueBorrow(index)
                    if index == region
            )
        });
        let written = operation.parameters.iter().any(|parameter| {
            matches!(parameter.mode, SystemParameterMode::UniqueBorrow(index) if index == region)
        });
        if borrowed {
            reads.push(region);
        }
        if written {
            writes.push(region);
        }
    }
    (reads, writes)
}

/// One [SYS-5] release row: the fixed effect row of a type's compiler-derived
/// release action, given as the presence of the two payload-free categories.
///
/// [STOR-3] makes this row the sole input to [EFF-2]'s release contribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemReleaseRow {
    /// The release action may observe or change state outside ordinary memory.
    pub external: bool,
    /// The release action may block its host thread.
    pub blocks: bool,
}

impl SystemReleaseRow {
    /// The empty release row: a logical consume or detach with no host call.
    pub const EMPTY: Self = Self {
        external: false,
        blocks: false,
    };
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
    /// One directory-read capability [PATH-2, SYS-10].
    DirectoryRead,
    /// One stateful open file with one cursor domain [SYS-11].
    ReadFile,
    /// One stateful output sink [SYS-12].
    Output,
    /// One immutable portable command code [SYS-13].
    ExitStatus,
    /// One stateful directory enumeration with one entry cursor [SYS-14].
    DirectoryList,
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
    /// `Output`'s logical source detach: it neither closes nor flushes the
    /// host descriptor [SYS-12], and operating-system process teardown closes
    /// the native descriptor afterwards.
    SourceDetach,
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
/// The seven outcome enums have no release action and take no row in the
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
            SystemResourceType::Output,
            SystemReleaseAction::SourceDetach,
            SystemResourceBacking::Opaque,
        ),
        EXIT_STATUS => (
            SystemResourceType::ExitStatus,
            SystemReleaseAction::LogicalConsume,
            SystemResourceBacking::Opaque,
        ),
        DIRECTORY_LIST => (
            SystemResourceType::DirectoryList,
            SystemReleaseAction::NativeCloseAttempt,
            SystemResourceBacking::Opaque,
        ),
        _ => return None,
    };
    let row = match action {
        // Only a native close attempt reaches the host; a logical consume and
        // a source detach make no target call and perform no external effect.
        SystemReleaseAction::NativeCloseAttempt => SystemReleaseRow {
            external: true,
            blocks: true,
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
/// `DirectoryRead`, `ReadFile`, and the candidate `DirectoryList` release with
/// at most one native close attempt (`external, blocks`); every other system
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

/// Maps one lookup-class [SYS-2] declaration to its nominal-table index.
#[must_use]
pub fn system_nominal_index(id: SystemDeclarationId, inventory: Inventory) -> Option<u8> {
    let ordinal = id.ordinal();
    (usize::from(ordinal) < system_nominals(inventory).len()).then_some(ordinal)
}

/// Maps one lookup-class [SYS-2] declaration to its constructor-table index.
#[must_use]
pub fn system_constructor_index(id: SystemDeclarationId, inventory: Inventory) -> Option<u8> {
    let mut ordinal = usize::from(id.ordinal());
    let nominals = system_nominals(inventory).len();
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
    let mut ordinal = system_nominals(inventory).len();
    for (constructor_index, constructor) in system_constructors(inventory).iter().enumerate() {
        if constructor_index == usize::from(index) {
            return u8::try_from(ordinal).ok().map(SystemDeclarationId::new);
        }
        ordinal += 1 + constructor.fields.len();
    }
    None
}

/// Maps one lookup-class [SYS-2] declaration to its operation-table index.
#[must_use]
pub fn system_operation_index(id: SystemDeclarationId, inventory: Inventory) -> Option<u8> {
    let mut ordinal = usize::from(id.ordinal());
    let nominals = system_nominals(inventory).len();
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
/// The active specification's inventory is one hundred sixty-seven records;
/// the traversal-surface candidate's is one hundred ninety-two.
pub(crate) fn system_declarations(inventory: Inventory) -> Vec<SystemDeclarationRecord> {
    let mut records = Vec::with_capacity(192);
    let push = |spelling: &'static str, class: Option<DeclarationClass>, records: &mut Vec<_>| {
        let Ok(ordinal) = u8::try_from(records.len()) else {
            unreachable!("the closed SYS-2 inventory fits one byte of ordinals");
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
    let nominals = system_nominals(inventory);
    if ordinal < nominals.len() {
        return Some(SystemEntity::Nominal(&nominals[ordinal]));
    }
    ordinal -= nominals.len();
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
        SystemDeclarationId, SystemEntity, SystemParameterMode, SystemResultPayload, SystemTypeRef,
        operation_region_effects, reserved_name, system_constructors, system_declarations,
        system_entity, system_nominals, system_operations,
    };

    #[test]
    fn system_inventory_matches_the_sys2_counted_totals() {
        // [SYS-2]: fourteen nominal types, thirty-nine enum-variant
        // constructors, sixty-four variant fields, eleven operations,
        // fourteen operation region parameters, twenty-five operation value
        // parameters — one hundred sixty-seven records in preorder.
        let nominals = system_nominals(Inventory::Base);
        let constructors = system_constructors(Inventory::Base);
        let operations = system_operations(Inventory::Base);
        assert_eq!(nominals.len(), 14);
        assert_eq!(nominals.iter().filter(|n| n.opaque).count(), 7);
        assert_eq!(constructors.len(), 39);
        assert_eq!(
            constructors
                .iter()
                .map(|constructor| constructor.fields.len())
                .sum::<usize>(),
            64
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
            25
        );

        let records = system_declarations(Inventory::Base);
        assert_eq!(records.len(), 167);
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
            39
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
            103
        );

        // Deterministic preorder spot checks used by diagnostic origins.
        for (ordinal, spelling) in [
            (0, "Args"),
            (6, "ExitStatus"),
            (7, "ArgError"),
            (13, "IoError"),
            (14, "InvalidIndex"),
            (27, "NotFound"),
            (114, "Other"),
            (117, "args_count"),
            (146, "open_read"),
            (165, "exit_status"),
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
            assert!(!SYSTEM_NOMINALS[usize::from(constructor.owner)].opaque);
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
            (14, 39, 11, 103)
        );
        assert!(system_entity(SystemDeclarationId::new(167), Inventory::Base).is_none());
        assert!(system_entity(SystemDeclarationId::new(u8::MAX), Inventory::Base).is_none());
    }

    /// The candidate inventory is the v0.31 inventory plus exactly the
    /// traversal rows, and every preorder ordinal below the first new nominal
    /// keeps its meaning only where the specification's own preorder keeps it:
    /// the two new nominal types shift every constructor and operation
    /// ordinal by two, which is why the switch selects one whole inventory
    /// rather than patching the other.
    #[test]
    fn traversal_candidate_inventory_matches_its_counted_totals() {
        let nominals = system_nominals(Inventory::Traversal);
        let constructors = system_constructors(Inventory::Traversal);
        let operations = system_operations(Inventory::Traversal);
        assert_eq!(nominals.len(), 16);
        assert_eq!(nominals.iter().filter(|n| n.opaque).count(), 8);
        assert_eq!(constructors.len(), 42);
        assert_eq!(
            constructors
                .iter()
                .map(|constructor| constructor.fields.len())
                .sum::<usize>(),
            67
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
            34
        );

        let records = system_declarations(Inventory::Traversal);
        assert_eq!(records.len(), 192);
        assert!(
            records
                .iter()
                .enumerate()
                .all(|(index, record)| usize::from(record.id().ordinal()) == index)
        );
        for (ordinal, spelling) in [
            (14, "DirectoryList"),
            (15, "ListOutcome"),
            (16, "InvalidIndex"),
            (116, "Other"),
            (119, "ListBytes"),
            (122, "ListEnd"),
            (123, "ListFailed"),
            (125, "args_count"),
            (173, "exit_status"),
            (175, "open_directory"),
            (182, "open_list"),
            (185, "list_once"),
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
            (16, 42, 14, 120)
        );
        assert!(system_entity(SystemDeclarationId::new(192), Inventory::Traversal).is_none());
    }

    /// The [SYS-11] file-open-by-name candidate's own counted totals.
    ///
    /// The candidate adds one operation and nothing else, so the delta is
    /// exactly one operation record, its two region-parameter records, and
    /// its four value-parameter records: 192 + 7 = 199. Because the row is
    /// appended, every ordinal the active inventory assigns is unchanged,
    /// which is the property the differential program tests rest on.
    #[test]
    fn open_by_name_candidate_inventory_matches_its_counted_totals() {
        let nominals = system_nominals(Inventory::OpenByName);
        let constructors = system_constructors(Inventory::OpenByName);
        let operations = system_operations(Inventory::OpenByName);
        assert_eq!(nominals.len(), 16);
        assert_eq!(constructors.len(), 42);
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
            38
        );

        let records = system_declarations(Inventory::OpenByName);
        assert_eq!(records.len(), 199);
        // Every active-inventory record keeps its exact ordinal and spelling.
        for (ordinal, record) in system_declarations(Inventory::Traversal).iter().enumerate() {
            assert_eq!(records[ordinal].spelling(), record.spelling());
        }
        for (ordinal, spelling) in [
            (192, "open_file"),
            (193, "'c"),
            (194, "'n"),
            (195, "root"),
            (196, "name"),
            (197, "start"),
            (198, "end"),
        ] {
            assert_eq!(records[ordinal].spelling(), spelling, "ordinal {ordinal}");
        }
        let open_file = SystemDeclarationId::new(192);
        let Some(SystemEntity::Operation(operation)) =
            system_entity(open_file, Inventory::OpenByName)
        else {
            panic!("the candidate ordinal must name the candidate operation");
        };
        assert_eq!(operation.spelling, "open_file");
        assert_eq!((operation.external, operation.blocks), (true, true));
        // Off, the same ordinal is past the inventory and names nothing.
        assert!(system_entity(open_file, Inventory::Traversal).is_none());
        assert!(system_entity(SystemDeclarationId::new(199), Inventory::OpenByName).is_none());
    }

    #[test]
    fn system_inventory_matches_independent_extraction_from_exact() {
        let spec = crate::ACTIVE_KERNEL_SPEC_TEXT;

        // The eight opaque nominal types, from the [SYS-2] prose sentence.
        let opaque_sentence = spec
            .split_once("Eight opaque nominal types: ")
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
                .filter(|nominal| nominal.opaque)
                .map(|nominal| nominal.spelling)
                .collect::<Vec<_>>()
        );

        // The seven enums, their thirty-nine variants, and every field, from
        // the first [SYS-2] code block.
        let sys2 = spec
            .split_once("[SYS-2] The system inventory is exactly:")
            .expect("exact SYS-2 opening")
            .1;
        let enum_block = sys2
            .split_once("```\n")
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
            .filter(|(_, nominal)| !nominal.opaque)
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

        // The fourteen complete operation signatures, from the second [SYS-2]
        // code block, including each written effect row.
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
            SystemTypeRef::U32 => "u32".to_owned(),
            SystemTypeRef::U64 => "u64".to_owned(),
            SystemTypeRef::BufferU8 => "buffer<u8>".to_owned(),
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
        if !operation.regions.is_empty() {
            rendered.push('[');
            rendered.push_str(&operation.regions.join(", "));
            rendered.push(']');
        }
        rendered.push('(');
        let parameters: Vec<_> = operation
            .parameters
            .iter()
            .map(|parameter| {
                let mode = match parameter.mode {
                    SystemParameterMode::Own => "own ".to_owned(),
                    SystemParameterMode::Borrow(region) => {
                        format!("&{} ", operation.regions[usize::from(region)])
                    }
                    SystemParameterMode::UniqueBorrow(region) => {
                        format!("&uniq {} ", operation.regions[usize::from(region)])
                    }
                };
                format!("{}: {mode}{}", parameter.name, render_type(parameter.ty))
            })
            .collect();
        rendered.push_str(&parameters.join(", "));
        rendered.push_str(") -> own ");
        rendered.push_str(&render_type(operation.result));
        let (reads, writes) = operation_region_effects(operation);
        let mut effects = Vec::new();
        if !reads.is_empty() {
            let regions: Vec<_> = reads
                .iter()
                .map(|region| operation.regions[usize::from(*region)])
                .collect();
            effects.push(format!("reads({})", regions.join(" ")));
        }
        if !writes.is_empty() {
            let regions: Vec<_> = writes
                .iter()
                .map(|region| operation.regions[usize::from(*region)])
                .collect();
            effects.push(format!("writes({})", regions.join(" ")));
        }
        if operation.external {
            effects.push("external".to_owned());
        }
        if operation.blocks {
            effects.push("blocks".to_owned());
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
        assert_eq!(OPERATION_FAMILIES.len(), 84);
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
        // OP-1 (iii)'s derived-set consequence after the owner cancelled the
        // infix comparisons: every respelled row is a dotted OPNAME and was
        // never a member, so all six integer comparisons are reserved and
        // DotlessOperationNames has exactly its v0.22 membership. The ordinals
        // are consecutive because the six occupy one contiguous op-column run.
        for (spelling, ordinal) in [
            ("ieq", 16),
            ("ine", 17),
            ("ilt", 18),
            ("ile", 19),
            ("igt", 20),
            ("ige", 21),
        ] {
            assert_eq!(
                reserved_name(spelling),
                Some((ReservedNameClass::DotlessOperation, ordinal)),
                "{spelling} is a dotless operation name"
            );
        }
        assert_eq!(
            reserved_name("cvt"),
            Some((ReservedNameClass::DotlessOperation, 38))
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
