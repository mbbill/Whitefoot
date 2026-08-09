//! The [QUAL-1] target-qualification table and the [QUAL-2] target guarantees.
//!
//! [QUAL-1] gives every system operation exactly one target-independent
//! semantic identity and keeps a separate table mapping each
//! `(specification version, semantic ID, target, program kind)` to exactly one
//! approved implementation version and one private ABI symbol. This module is
//! that table: fixed Rust data and one match, consulted after the exact target
//! is selected and before any use of an operation is emitted. It is
//! compiler-internal data — the language defines no registry, negotiation
//! protocol, dynamic loading, or plugin interface [PROG-1].
//!
//! Every [SYS-2] semantic identity now has one approved implementation on a
//! qualified target, so the only stop left here is qualification itself:
//!
//! * an absent mapping, an implementation incompatible with the selected
//!   target or program kind, and an unmet [QUAL-2] guarantee are
//!   target-qualification failures under [DIAG-1] — like a target-layout
//!   failure they are not source-language rejections and cite no language
//!   rule; and
//! * qualification never narrows a semantic ID to what a target can supply,
//!   and no weaker operation is substituted for an unqualified one.

use crate::{
    ACTIVE_KERNEL_SPEC_VERSION, IrEntry, IrInstruction, IrNominalKind, IrOperation, IrProgram,
    IrSystemOperation, SystemReleaseAction, SystemResourceContract, SystemResourceType,
};

use super::emitter::BackendFailure;

/// The number of [SYS-2] system operations, and therefore of semantic IDs.
const OPERATION_COUNT: usize = crate::SYSTEM_OPERATIONS.len();

/// The number of [SYS-2] opaque resource types.
const RESOURCE_COUNT: usize = 7;

/// The [HOST-1] code-unit family a qualified target's host strings belong to.
///
/// [HOST-1] fixes exactly two lossless families. A target whose native
/// representation belongs to neither qualifies for the host-string and path
/// semantic IDs only under a specification amendment, so this compiler
/// qualifies exactly the Unix family and narrows nothing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CodeUnitFamily {
    /// One 8-bit code unit in `0x01..0xff`: every non-NUL byte sequence is
    /// representable and preserved exactly.
    Unix,
}

/// The set of host facilities one qualified target's approved
/// implementations call.
///
/// This is the target column of the [QUAL-1] row key made concrete. One
/// semantic identity may have more than one approved implementation, one per
/// target, and every implementation of it answers the same specified
/// signature, outcomes, ownership transitions, and effects. An operation whose
/// implementation touches no host object at all — argument counting, the
/// host-string routes, path construction, `exit_status` — resolves to the same
/// approved row on every target, so only the facilities that reach a real
/// operating-system object appear here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HostFacilities {
    /// The target's own native facilities.
    Native,
    /// The deterministic test host: the same operations answered from
    /// scripted in-process state instead of real operating-system objects, so
    /// a contract test can force a condition — a close that fails, a read that
    /// stops short, a write that is only partly accepted — that a real file or
    /// pipe cannot produce on demand. It supplies exactly the arrangements
    /// those contract tests need and is not a simulator of the host.
    ///
    /// It exists only in a test build, so no `whitefootc` compilation of a
    /// real program can select it.
    #[cfg(test)]
    DeterministicTest,
}

impl HostFacilities {
    /// The facility the [QUAL-3] bootstrap opens the initial working
    /// directory through.
    const fn directory_open(self) -> &'static str {
        match self {
            Self::Native => "open",
            #[cfg(test)]
            Self::DeterministicTest => "wf_test_open",
        }
    }

    /// The facility one [SYS-5] consuming release attempts its single close
    /// through.
    const fn close(self) -> &'static str {
        match self {
            Self::Native => "close",
            #[cfg(test)]
            Self::DeterministicTest => "wf_test_close",
        }
    }

    /// The directory-relative open facility `open_read` resolves through
    /// [PATH-2].
    const fn file_open(self) -> &'static str {
        match self {
            Self::Native => "openat",
            #[cfg(test)]
            Self::DeterministicTest => "wf_test_openat",
        }
    }

    /// The facility one `read_once` transfer attempt reaches.
    const fn read(self) -> &'static str {
        match self {
            Self::Native => "read",
            #[cfg(test)]
            Self::DeterministicTest => "wf_test_read",
        }
    }

    /// The facility one `write_once` transfer attempt reaches.
    const fn write(self) -> &'static str {
        match self {
            Self::Native => "write",
            #[cfg(test)]
            Self::DeterministicTest => "wf_test_write",
        }
    }
}

/// The [FN-7] program kind one build produces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProgramKind {
    /// The unlabelled entry: no standard input and no produced status.
    Unlabelled,
    /// A natively compiled `command`.
    Command,
}

/// One portable [SYS-7] class and the native error codes a target maps onto
/// it.
///
/// A target's table is the complete closed thirty-class set in [SYS-2]
/// declared order: a class no native facility of that target produces keeps
/// its row with an empty code list rather than disappearing, so the table
/// states the whole portable vocabulary and narrows nothing by omission. A
/// native error named by no row has no portable distinction in this set and
/// maps to `Other` [SYS-7], which is the closed set's own rule rather than a
/// wildcard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PortableErrorClass {
    /// The [SYS-2] `IoError` variant spelling this row maps onto.
    pub(crate) class: &'static str,
    /// The target's own native error codes, each value-preserving in `u32`.
    pub(crate) codes: &'static [i32],
}

const fn class(class: &'static str, codes: &'static [i32]) -> PortableErrorClass {
    PortableErrorClass { class, codes }
}

/// The Darwin-family mapping of native `errno` values onto [SYS-7] classes.
const DARWIN_ERROR_CLASSES: [PortableErrorClass; 30] = [
    class("NotFound", &[2]),
    class("PermissionDenied", &[1, 13]),
    class("AlreadyExists", &[17]),
    class("NotDirectory", &[20]),
    class("IsDirectory", &[21]),
    class("DirectoryNotEmpty", &[66]),
    class("ReadOnly", &[30]),
    class("ResourceBusy", &[16, 26]),
    class("InvalidInput", &[22]),
    class("InvalidPath", &[62, 63]),
    class("Unsupported", &[45, 78, 102]),
    class("Interrupted", &[4]),
    class("WouldBlock", &[35]),
    class("TimedOut", &[60]),
    class("BrokenPipe", &[32]),
    // No native code produces these two: `WriteZero` is [SYS-8]'s own
    // host-accepted-nothing outcome, and no v0.22 operation reports a
    // truncated required transfer.
    class("WriteZero", &[]),
    class("UnexpectedEnd", &[]),
    class("ConnectionRefused", &[61]),
    class("ConnectionReset", &[54]),
    class("ConnectionAborted", &[53]),
    class("NotConnected", &[57]),
    class("AddressInUse", &[48]),
    class("AddressUnavailable", &[49]),
    class("ResourceExhausted", &[12, 23, 24, 55]),
    class("FileTooLarge", &[27, 84]),
    class("NoSpace", &[28]),
    class("QuotaExceeded", &[69]),
    class("CrossDevice", &[18]),
    class("DeviceFailure", &[5, 6, 19]),
    // Every native error with no portable distinction in this set [SYS-7].
    class("Other", &[]),
];

/// The Linux-family mapping of native `errno` values onto [SYS-7] classes.
///
/// The two families share the first thirty-four codes and diverge above
/// them, so this is a separate table rather than a diff of the first.
const LINUX_ERROR_CLASSES: [PortableErrorClass; 30] = [
    class("NotFound", &[2]),
    class("PermissionDenied", &[1, 13]),
    class("AlreadyExists", &[17]),
    class("NotDirectory", &[20]),
    class("IsDirectory", &[21]),
    class("DirectoryNotEmpty", &[39]),
    class("ReadOnly", &[30]),
    class("ResourceBusy", &[16, 26]),
    class("InvalidInput", &[22]),
    class("InvalidPath", &[36, 40]),
    class("Unsupported", &[38, 95]),
    class("Interrupted", &[4]),
    class("WouldBlock", &[11]),
    class("TimedOut", &[110]),
    class("BrokenPipe", &[32]),
    class("WriteZero", &[]),
    class("UnexpectedEnd", &[]),
    class("ConnectionRefused", &[111]),
    class("ConnectionReset", &[104]),
    class("ConnectionAborted", &[103]),
    class("NotConnected", &[107]),
    class("AddressInUse", &[98]),
    class("AddressUnavailable", &[99]),
    class("ResourceExhausted", &[12, 23, 24, 105]),
    class("FileTooLarge", &[27, 75]),
    class("NoSpace", &[28]),
    class("QuotaExceeded", &[122]),
    class("CrossDevice", &[18]),
    class("DeviceFailure", &[5, 6, 19]),
    class("Other", &[]),
];

/// The target-owned [SYS-7] `origin` discriminator of the directory-relative
/// open facility.
///
/// `origin` selects which native facility produced a `code`, and is zero when
/// the target supplies no value for the field. Both qualified targets are
/// Unix-family and expose the same three facilities, so one set of values
/// serves both.
pub(crate) const ORIGIN_DIRECTORY_OPEN: u8 = 1;
/// The target-owned `origin` discriminator of the read facility.
pub(crate) const ORIGIN_READ: u8 = 2;
/// The target-owned `origin` discriminator of the write facility.
pub(crate) const ORIGIN_WRITE: u8 = 3;
/// The `origin` value used when no native facility produced the code.
pub(crate) const ORIGIN_NONE: u8 = 0;

/// One [QUAL-2] guarantee a semantic ID's record may require of a target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetGuarantee {
    /// Command-lifetime argument backing.
    CommandLifetimeArgumentBacking,
    /// A lossless host-string code-unit family [HOST-1].
    LosslessCodeUnits,
    /// The target's own directory-relative resolution facility [PATH-2],
    /// never a prefix concatenated onto a path.
    DirectoryRelativeResolution,
}

/// The thing a qualification row is about.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Facility {
    /// The [FN-7] `command` entry and its [PROG-3] start.
    CommandEntry,
    /// One [SYS-2] semantic identity, by inventory ordinal.
    Operation(u8),
    /// One [SYS-2] opaque resource type.
    Resource(SystemResourceType),
}

/// One target-qualification failure [QUAL-1].
///
/// Like a target-layout failure this is not a source-language rejection and
/// cites no language rule [DIAG-1].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QualificationFailure {
    /// No table row maps this facility for the selected specification
    /// version, target, and program kind.
    MissingMapping(Facility),
    /// The approved implementation is incompatible with the selected program
    /// kind.
    IncompatibleProgramKind(Facility),
    /// A required target guarantee is unmet.
    UnmetGuarantee {
        /// The facility whose record requires the guarantee.
        facility: Facility,
        /// The guarantee the selected target does not supply.
        guarantee: TargetGuarantee,
    },
    /// The approved release implementation does not match the [SYS-5] release
    /// action the checked program carries for this resource type.
    InconsistentRelease(Facility),
}

/// One approved implementation of one semantic identity [QUAL-1].
///
/// The version identifies the implementation inside one semantic identity: it
/// may be replaced only within that identity, and a change to anything the
/// ID's record binds is a different semantic ID under a new specification
/// version, never a target-code update.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ApprovedImplementation {
    version: u16,
    symbol: &'static str,
}

impl ApprovedImplementation {
    /// The one private ABI symbol the emitted program calls.
    pub(crate) const fn symbol(self) -> &'static str {
        self.symbol
    }

    /// The approved implementation version inside this semantic identity.
    pub(crate) const fn version(self) -> u16 {
        self.version
    }
}

/// The complete target representation of one opaque [SYS-2] resource value.
///
/// [QUAL-1] fixes an opaque type's representation in its qualification record;
/// no source construct observes it and no operation yields it as a source
/// value of a fixed width [HOST-1].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResourceRepresentation {
    /// [HOST-3]'s inline lease: a private code-unit address and length carried
    /// in the value itself, over backing the invocation supplies.
    InlineLease,
    /// The command's argument vector: a base address and a count.
    ArgumentVector,
    /// One native descriptor.
    Descriptor,
    /// One portable command code [SYS-13].
    CommandCode,
}

impl ResourceRepresentation {
    /// The representation's size in bytes on a qualified target.
    pub(crate) const fn size(self) -> u64 {
        match self {
            Self::InlineLease | Self::ArgumentVector => 16,
            Self::Descriptor => 4,
            Self::CommandCode => 1,
        }
    }

    /// The representation's alignment in bytes on a qualified target.
    pub(crate) const fn align(self) -> u64 {
        match self {
            Self::InlineLease | Self::ArgumentVector => 8,
            Self::Descriptor => 4,
            Self::CommandCode => 1,
        }
    }

    /// The representation's emitted LLVM type.
    pub(crate) const fn llvm(self) -> &'static str {
        match self {
            Self::InlineLease | Self::ArgumentVector => "{ ptr, i64 }",
            Self::Descriptor => "i32",
            Self::CommandCode => "i8",
        }
    }
}

/// The approved implementation of one type's [SYS-5] release action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReleaseImplementation {
    /// A logical consume or a logical source detach: no host call, no target
    /// call, no handle lookup, no byte copy, and no external effect.
    NoCode,
    /// At most one direct native close attempt through this symbol; the close
    /// diagnostic is discarded and an ambiguous close is never retried.
    NativeClose(&'static str),
}

/// One approved implementation of one opaque resource type's qualification
/// record: its target representation and its release action's code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ResourceImplementation {
    version: u16,
    representation: ResourceRepresentation,
    release: ReleaseImplementation,
}

impl ResourceImplementation {
    /// The value's complete target representation.
    pub(crate) const fn representation(self) -> ResourceRepresentation {
        self.representation
    }

    /// The release action's approved implementation.
    pub(crate) const fn release(self) -> ReleaseImplementation {
        self.release
    }

    /// The approved implementation version inside this semantic identity.
    pub(crate) const fn version(self) -> u16 {
        self.version
    }
}

/// One selected target's system-facing identity and its [QUAL-2] guarantees.
///
/// Everything here is a property of the target rather than of any program:
/// [QUAL-2] states the two guarantees precisely because a program has nothing
/// to check for them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SystemTarget {
    family: Option<CodeUnitFamily>,
    /// Whether the target supplies command-lifetime argument backing.
    ///
    /// [QUAL-2] admits two forms — stable native argument backing, or one
    /// complete snapshot taken before any Whitefoot code runs. Every target
    /// qualified here supplies the first, so the bootstrap takes no snapshot
    /// and copies no argument byte; a target supplying neither fails
    /// qualification for the command entry and for argument access rather
    /// than entering under a weaker guarantee.
    argument_backing: bool,
    directory_relative: bool,
    /// Which host facilities this target's approved implementations call.
    host: HostFacilities,
    root_prefix: u8,
    directory_open_flags: i32,
    file_open_flags: i32,
    errno_location: &'static str,
    errno_declaration: &'static str,
    error_classes: &'static [PortableErrorClass; 30],
    broken_pipe_signal: i32,
    ignored_disposition: i64,
    invalid_disposition: i64,
}

impl SystemTarget {
    /// The single code unit sequence a Unix-family target resolves against a
    /// filesystem root, and therefore the complete [PATH-1] target-root prefix
    /// set of this family: one leading separator.
    pub(crate) const fn root_prefix(self) -> u8 {
        self.root_prefix
    }

    /// The flags the bootstrap opens the initial working directory with, so
    /// [PATH-2] resolution uses the target's own directory-relative facility.
    pub(crate) const fn directory_open_flags(self) -> i32 {
        self.directory_open_flags
    }

    /// The flags a directory-relative open of a file for reading uses.
    pub(crate) const fn file_open_flags(self) -> i32 {
        self.file_open_flags
    }

    /// The symbol yielding the address of the calling thread's native error
    /// slot, read immediately after a failing facility call.
    pub(crate) const fn errno_location(self) -> &'static str {
        self.errno_location
    }

    /// That symbol's declaration.
    pub(crate) const fn errno_declaration(self) -> &'static str {
        self.errno_declaration
    }

    /// The target's complete [SYS-7] class mapping, in [SYS-2] declared order.
    pub(crate) const fn error_classes(self) -> &'static [PortableErrorClass; 30] {
        self.error_classes
    }

    /// The host facility the [QUAL-3] bootstrap opens the initial working
    /// directory through on this target.
    pub(crate) const fn directory_open_symbol(self) -> &'static str {
        self.host.directory_open()
    }

    /// The host facility `open_read` resolves a relative path through
    /// [PATH-2].
    pub(crate) const fn file_open_symbol(self) -> &'static str {
        self.host.file_open()
    }

    /// The host facility one `read_once` transfer attempt reaches [SYS-8].
    pub(crate) const fn read_symbol(self) -> &'static str {
        self.host.read()
    }

    /// The host facility one `write_once` transfer attempt reaches [SYS-8].
    ///
    /// This is the `write_once` row's facility only. The mandatory [DIAG-3]
    /// trap record writes through the native `write` on every target: a
    /// scripted host must never be able to truncate a trap record.
    pub(crate) const fn write_symbol(self) -> &'static str {
        self.host.write()
    }

    /// The write-to-closed-pipe signal number [QUAL-3] normalizes once.
    pub(crate) const fn broken_pipe_signal(self) -> i32 {
        self.broken_pipe_signal
    }

    /// The disposition value meaning "ignored".
    pub(crate) const fn ignored_disposition(self) -> i64 {
        self.ignored_disposition
    }

    /// The disposition value the target returns when it installed none.
    pub(crate) const fn invalid_disposition(self) -> i64 {
        self.invalid_disposition
    }

    fn supplies(self, guarantee: TargetGuarantee) -> bool {
        match guarantee {
            TargetGuarantee::CommandLifetimeArgumentBacking => self.argument_backing,
            TargetGuarantee::LosslessCodeUnits => self.family.is_some(),
            TargetGuarantee::DirectoryRelativeResolution => self.directory_relative,
        }
    }

    /// Returns the system-facing record of one selected target triple.
    ///
    /// The macOS and Linux command targets differ in the directory-open flag
    /// value, the native error-slot symbol, and the native error codes their
    /// facilities produce; both supply stable native argument backing that
    /// outlives the invocation, the Unix code-unit family, and `openat`-style
    /// directory-relative resolution.
    pub(crate) fn for_triple(triple: &str) -> Option<Self> {
        let (directory_open_flags, errno_location, errno_declaration, error_classes) = match triple
        {
            // `O_RDONLY | O_DIRECTORY` on the Darwin ABI.
            "aarch64-apple-darwin" | "x86_64-apple-darwin" => (
                0x0010_0000,
                "__error",
                "declare ptr @__error()",
                &DARWIN_ERROR_CLASSES,
            ),
            // `O_RDONLY | O_DIRECTORY` on the Linux asm-generic ABI, which
            // both supported architectures use.
            "aarch64-unknown-linux-gnu" | "x86_64-unknown-linux-gnu" => (
                0o200_000,
                "__errno_location",
                "declare ptr @__errno_location()",
                &LINUX_ERROR_CLASSES,
            ),
            _ => return None,
        };
        Some(Self {
            family: Some(CodeUnitFamily::Unix),
            argument_backing: true,
            directory_relative: true,
            host: HostFacilities::Native,
            root_prefix: b'/',
            directory_open_flags,
            // `O_RDONLY` is zero on both families. `open_read` opens for
            // reading only and adds no creation, truncation, or mode flag:
            // [SYS-11] creates one live readable file and nothing else.
            file_open_flags: 0,
            errno_location,
            errno_declaration,
            error_classes,
            // `SIGPIPE` is 13 on every supported target.
            broken_pipe_signal: 13,
            // `SIG_IGN` and `SIG_ERR`.
            ignored_disposition: 1,
            invalid_disposition: -1,
        })
    }

    /// The deterministic test target: the host triple's own layout, ABI, and
    /// [QUAL-2] guarantees, with the file and descriptor facilities answered
    /// by a scripted in-process host instead of the operating system.
    ///
    /// It is a second column of the [QUAL-1] table rather than a relaxed one.
    /// Every semantic identity keeps its specified signature, outcomes,
    /// ownership transitions, and effect row; a program compiled for it runs
    /// the same emitted lowering, so a forced condition is observed through
    /// real compiled code. It qualifies exactly the guarantees the native
    /// target qualifies, because a fake host that withheld one would be
    /// answering a different specification.
    #[cfg(test)]
    pub(crate) fn deterministic_test() -> Self {
        let triple = super::target::TargetLayout::host()
            .expect("the deterministic test target runs on a qualified host")
            .triple();
        let mut target =
            Self::for_triple(triple).expect("a supported host triple is a qualified target");
        target.host = HostFacilities::DeterministicTest;
        target
    }

    /// Builds one target record directly, for tests that need a target which
    /// deliberately withholds a [QUAL-2] guarantee.
    #[cfg(test)]
    pub(crate) fn probe(
        family: Option<CodeUnitFamily>,
        argument_backing: bool,
        directory_relative: bool,
    ) -> Self {
        let mut target = Self::for_triple("aarch64-apple-darwin")
            .expect("the probe base triple is a qualified target");
        target.family = family;
        target.argument_backing = argument_backing;
        target.directory_relative = directory_relative;
        target
    }
}

/// The complete qualification of one program against one selected target.
///
/// Every facility the program uses is resolved here, before layout and before
/// emission, so no emission site consults the table again and no runtime
/// operation-ID switch, target tag, per-call dispatch table, or handle lookup
/// can select among implementations [QUAL-3].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Qualification {
    target: SystemTarget,
    kind: ProgramKind,
    operations: [Option<ApprovedImplementation>; OPERATION_COUNT],
    resources: [Option<ResourceImplementation>; RESOURCE_COUNT],
}

impl Qualification {
    /// The selected target this program was qualified against.
    pub(crate) const fn target(&self) -> SystemTarget {
        self.target
    }

    /// The [FN-7] program kind the qualification selected for.
    pub(crate) const fn kind(&self) -> ProgramKind {
        self.kind
    }

    /// The approved implementation of one used semantic identity.
    pub(crate) fn operation(
        &self,
        operation: IrSystemOperation,
    ) -> Result<ApprovedImplementation, BackendFailure> {
        self.operations
            .get(usize::from(operation.ordinal()))
            .copied()
            .flatten()
            .ok_or(BackendFailure::InvalidIr)
    }

    /// The approved implementation of one used opaque resource type.
    pub(crate) fn resource(
        &self,
        resource: SystemResourceType,
    ) -> Result<ResourceImplementation, BackendFailure> {
        self.resources
            .get(resource_index(resource))
            .copied()
            .flatten()
            .ok_or(BackendFailure::InvalidIr)
    }

    /// Every used semantic identity with its approved implementation, in
    /// inventory order, so emission can define each private wrapper once.
    pub(crate) fn used_operations(
        &self,
    ) -> impl Iterator<Item = (u8, ApprovedImplementation)> + '_ {
        self.operations
            .iter()
            .enumerate()
            .filter_map(|(ordinal, row)| {
                let implementation = (*row)?;
                let ordinal = u8::try_from(ordinal).ok()?;
                Some((ordinal, implementation))
            })
    }
}

/// The target representation one opaque [SYS-2] resource type has on every
/// target this compiler qualifies.
///
/// This is the single definition the table rows are built from. It takes no
/// target because exactly one code-unit family and one descriptor ABI qualify
/// today [HOST-1]; a second qualified family would give this function a target
/// parameter rather than a second copy of the data. Emission may read it
/// directly only because it runs after `qualify_program` accepted the program,
/// so every resource it sees already resolved to this row.
pub(crate) const fn qualified_representation(
    resource: SystemResourceType,
) -> ResourceRepresentation {
    match resource {
        SystemResourceType::Args => ResourceRepresentation::ArgumentVector,
        // [HOST-3]: a private code-unit address and length carried in the
        // value itself, which [PATH-1] construction retypes unchanged.
        SystemResourceType::HostString | SystemResourceType::RelativePath => {
            ResourceRepresentation::InlineLease
        }
        SystemResourceType::DirectoryRead
        | SystemResourceType::ReadFile
        | SystemResourceType::Output => ResourceRepresentation::Descriptor,
        SystemResourceType::ExitStatus => ResourceRepresentation::CommandCode,
    }
}

const fn resource_index(resource: SystemResourceType) -> usize {
    match resource {
        SystemResourceType::Args => 0,
        SystemResourceType::HostString => 1,
        SystemResourceType::RelativePath => 2,
        SystemResourceType::DirectoryRead => 3,
        SystemResourceType::ReadFile => 4,
        SystemResourceType::Output => 5,
        SystemResourceType::ExitStatus => 6,
    }
}

/// The [QUAL-2] guarantees one semantic identity's record requires.
fn operation_guarantees(operation: u8) -> &'static [TargetGuarantee] {
    use TargetGuarantee::{
        CommandLifetimeArgumentBacking, DirectoryRelativeResolution, LosslessCodeUnits,
    };
    const ARGUMENTS: &[TargetGuarantee] = &[CommandLifetimeArgumentBacking];
    const ARGUMENT_STRING: &[TargetGuarantee] =
        &[CommandLifetimeArgumentBacking, LosslessCodeUnits];
    const STRINGS: &[TargetGuarantee] = &[LosslessCodeUnits];
    const DIRECTORY: &[TargetGuarantee] = &[LosslessCodeUnits, DirectoryRelativeResolution];
    match operation {
        // `args_count` reads the command-lifetime argument backing.
        0 => ARGUMENTS,
        // `arg_get` leases code units out of that backing [HOST-3].
        1 => ARGUMENT_STRING,
        // The two host-string routes and relative-path construction are the
        // host-string and path semantic IDs [HOST-1, PATH-1].
        2..=6 => STRINGS,
        // `open_read` resolves a relative path through the target's own
        // directory-relative facility [PATH-2].
        7 => DIRECTORY,
        // `read_once`, `write_once`, and `exit_status` require neither.
        _ => &[],
    }
}

/// The [QUAL-2] guarantees one opaque resource type's record requires.
fn resource_guarantees(resource: SystemResourceType) -> &'static [TargetGuarantee] {
    use TargetGuarantee::{
        CommandLifetimeArgumentBacking, DirectoryRelativeResolution, LosslessCodeUnits,
    };
    const ARGUMENTS: &[TargetGuarantee] = &[CommandLifetimeArgumentBacking];
    const LEASE: &[TargetGuarantee] = &[CommandLifetimeArgumentBacking, LosslessCodeUnits];
    const DIRECTORY: &[TargetGuarantee] = &[DirectoryRelativeResolution];
    match resource {
        SystemResourceType::Args => ARGUMENTS,
        // A lease denotes valid code units however it is used exactly because
        // its command-lifetime backing strictly outlives it [HOST-3].
        SystemResourceType::HostString | SystemResourceType::RelativePath => LEASE,
        SystemResourceType::DirectoryRead => DIRECTORY,
        SystemResourceType::ReadFile
        | SystemResourceType::Output
        | SystemResourceType::ExitStatus => &[],
    }
}

/// The `(specification version, semantic ID, target, program kind)` row.
fn operation_row(
    operation: u8,
    target: SystemTarget,
    kind: ProgramKind,
) -> Result<ApprovedImplementation, QualificationFailure> {
    let facility = Facility::Operation(operation);
    if ACTIVE_KERNEL_SPEC_VERSION != "v0.24" || usize::from(operation) >= OPERATION_COUNT {
        return Err(QualificationFailure::MissingMapping(facility));
    }
    // Every [SYS-2] operation exists only in a system-admitted unit, which is
    // exactly a kind-declaring one [SYS-3], and the only kind this version
    // defines a form for is `command` [FN-7].
    if kind != ProgramKind::Command {
        return Err(QualificationFailure::IncompatibleProgramKind(facility));
    }
    for guarantee in operation_guarantees(operation) {
        if !target.supplies(*guarantee) {
            return Err(QualificationFailure::UnmetGuarantee {
                facility,
                guarantee: *guarantee,
            });
        }
    }
    let symbol = match operation {
        0 => "wf.sys.args_count.v1",
        1 => "wf.sys.arg_get.v1",
        2 => "wf.sys.host_bytes_len.v1",
        3 => "wf.sys.host_copy_bytes.v1",
        4 => "wf.sys.host_utf8_len.v1",
        5 => "wf.sys.host_copy_utf8.v1",
        6 => "wf.sys.relative_path.v1",
        7 => "wf.sys.open_read.v1",
        8 => "wf.sys.read_once.v1",
        9 => "wf.sys.write_once.v1",
        10 => "wf.sys.exit_status.v1",
        // The ordinal bound above admits no other value.
        _ => return Err(QualificationFailure::MissingMapping(facility)),
    };
    Ok(ApprovedImplementation { version: 1, symbol })
}

/// The `(specification version, resource type, target, program kind)` row.
fn resource_row(
    contract: SystemResourceContract,
    target: SystemTarget,
    kind: ProgramKind,
) -> Result<ResourceImplementation, QualificationFailure> {
    let facility = Facility::Resource(contract.resource);
    if ACTIVE_KERNEL_SPEC_VERSION != "v0.24" {
        return Err(QualificationFailure::MissingMapping(facility));
    }
    if kind != ProgramKind::Command {
        return Err(QualificationFailure::IncompatibleProgramKind(facility));
    }
    for guarantee in resource_guarantees(contract.resource) {
        if !target.supplies(*guarantee) {
            return Err(QualificationFailure::UnmetGuarantee {
                facility,
                guarantee: *guarantee,
            });
        }
    }
    let representation = qualified_representation(contract.resource);
    let release = match contract.resource {
        // At most one direct native close attempt; `Output` detaches the
        // source capability without closing or flushing the descriptor
        // [SYS-12], and every other type releases with a logical consume.
        SystemResourceType::DirectoryRead | SystemResourceType::ReadFile => {
            ReleaseImplementation::NativeClose(target.host.close())
        }
        SystemResourceType::Args
        | SystemResourceType::HostString
        | SystemResourceType::RelativePath
        | SystemResourceType::Output
        | SystemResourceType::ExitStatus => ReleaseImplementation::NoCode,
    };
    // The approved release code and the [SYS-5] action the checked program
    // carries must be the same action. Emission reads the checked program's
    // record, so a table row that disagreed with it would silently emit a
    // different release.
    let consistent = matches!(
        (contract.action, release),
        (
            SystemReleaseAction::LogicalConsume | SystemReleaseAction::SourceDetach,
            ReleaseImplementation::NoCode,
        ) | (
            SystemReleaseAction::NativeCloseAttempt,
            ReleaseImplementation::NativeClose(_)
        )
    );
    if !consistent {
        return Err(QualificationFailure::InconsistentRelease(facility));
    }
    Ok(ResourceImplementation {
        version: 1,
        representation,
        release,
    })
}

/// The `(specification version, command entry, target, program kind)` row.
///
/// [QUAL-2]: a target qualified for the command entry and for argument access
/// supplies command-lifetime argument backing; a target that can supply
/// neither fails qualification for both IDs.
fn command_entry_row(target: SystemTarget) -> Result<(), QualificationFailure> {
    if ACTIVE_KERNEL_SPEC_VERSION != "v0.24" {
        return Err(QualificationFailure::MissingMapping(Facility::CommandEntry));
    }
    if !target.supplies(TargetGuarantee::CommandLifetimeArgumentBacking) {
        return Err(QualificationFailure::UnmetGuarantee {
            facility: Facility::CommandEntry,
            guarantee: TargetGuarantee::CommandLifetimeArgumentBacking,
        });
    }
    Ok(())
}

/// Qualifies one program against one selected target before layout and
/// emission.
///
/// The scan is over the IR's own facts — the [FN-7] entry form, the opaque
/// resource nominals, and the semantic IDs of the system calls — never over a
/// source name, signature, project, corpus, or test identity [QUAL-1].
pub(crate) fn qualify_program(
    target: SystemTarget,
    program: &IrProgram<'_, '_, '_>,
) -> Result<Qualification, BackendFailure> {
    let kind = match program.entry() {
        IrEntry::Unlabelled => ProgramKind::Unlabelled,
        IrEntry::Command { .. } => ProgramKind::Command,
    };
    let mut qualification = Qualification {
        target,
        kind,
        operations: [None; OPERATION_COUNT],
        resources: [None; RESOURCE_COUNT],
    };
    if kind == ProgramKind::Command {
        command_entry_row(target).map_err(BackendFailure::TargetQualification)?;
    }
    for nominal in program.nominals() {
        let IrNominalKind::SystemResource(contract) = nominal.kind() else {
            continue;
        };
        let implementation =
            resource_row(*contract, target, kind).map_err(BackendFailure::TargetQualification)?;
        qualification.resources[resource_index(contract.resource)] = Some(implementation);
    }
    for function in program.functions() {
        for block in function.blocks() {
            for instruction in block.instructions() {
                let IrInstruction::Define {
                    operation: IrOperation::SystemCall { operation, .. },
                    ..
                } = instruction
                else {
                    continue;
                };
                let ordinal = operation.ordinal();
                let implementation = operation_row(ordinal, target, kind)
                    .map_err(BackendFailure::TargetQualification)?;
                qualification.operations[usize::from(ordinal)] = Some(implementation);
            }
        }
    }
    Ok(qualification)
}
