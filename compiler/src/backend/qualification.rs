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
//! Three stops are distinct and never conflated:
//!
//! * an absent mapping, an implementation incompatible with the selected
//!   target or program kind, and an unmet [QUAL-2] guarantee are
//!   target-qualification failures under [DIAG-1] — like a target-layout
//!   failure they are not source-language rejections and cite no language
//!   rule;
//! * a semantic identity this compiler has not implemented yet is an explicit
//!   unsupported compiler capability, never a qualification verdict; and
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

/// The [FN-7] program kind one build produces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProgramKind {
    /// The unlabelled entry: no standard input and no produced status.
    Unlabelled,
    /// A natively compiled `command`.
    Command,
}

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
    root_prefix: u8,
    directory_open_flags: i32,
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
    /// The macOS and Linux command targets differ only in the directory-open
    /// flag value; both supply stable native argument backing that outlives
    /// the invocation, the Unix code-unit family, and `openat`-style
    /// directory-relative resolution.
    pub(crate) fn for_triple(triple: &str) -> Option<Self> {
        let directory_open_flags = match triple {
            // `O_RDONLY | O_DIRECTORY` on the Darwin ABI.
            "aarch64-apple-darwin" | "x86_64-apple-darwin" => 0x0010_0000,
            // `O_RDONLY | O_DIRECTORY` on the Linux asm-generic ABI, which
            // both supported architectures use.
            "aarch64-unknown-linux-gnu" | "x86_64-unknown-linux-gnu" => 0o200_000,
            _ => return None,
        };
        Some(Self {
            family: Some(CodeUnitFamily::Unix),
            argument_backing: true,
            directory_relative: true,
            root_prefix: b'/',
            directory_open_flags,
            // `SIGPIPE` is 13 on every supported target.
            broken_pipe_signal: 13,
            // `SIG_IGN` and `SIG_ERR`.
            ignored_disposition: 1,
            invalid_disposition: -1,
        })
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

/// One table row's implementation state for a facility this compiler knows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OperationRow {
    /// One approved implementation.
    Approved(ApprovedImplementation),
    /// The semantic identity is specified but this compiler emits no
    /// implementation for it yet. That is an explicit unsupported compiler
    /// capability, not a qualification verdict.
    NotImplemented,
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
) -> Result<OperationRow, QualificationFailure> {
    let facility = Facility::Operation(operation);
    if ACTIVE_KERNEL_SPEC_VERSION != "v0.19" || usize::from(operation) >= OPERATION_COUNT {
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
        10 => "wf.sys.exit_status.v1",
        // `open_read`, `read_once`, and `write_once` are qualified operations
        // whose native emission this compiler does not implement yet.
        _ => return Ok(OperationRow::NotImplemented),
    };
    Ok(OperationRow::Approved(ApprovedImplementation {
        version: 1,
        symbol,
    }))
}

/// The `(specification version, resource type, target, program kind)` row.
fn resource_row(
    contract: SystemResourceContract,
    target: SystemTarget,
    kind: ProgramKind,
) -> Result<ResourceImplementation, QualificationFailure> {
    let facility = Facility::Resource(contract.resource);
    if ACTIVE_KERNEL_SPEC_VERSION != "v0.19" {
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
            ReleaseImplementation::NativeClose("close")
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
    if ACTIVE_KERNEL_SPEC_VERSION != "v0.19" {
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
                match operation_row(ordinal, target, kind)
                    .map_err(BackendFailure::TargetQualification)?
                {
                    OperationRow::Approved(implementation) => {
                        qualification.operations[usize::from(ordinal)] = Some(implementation);
                    }
                    // Reported as an unsupported compiler capability, never as
                    // a qualification verdict and never as a source rejection.
                    OperationRow::NotImplemented => {
                        return Err(BackendFailure::UnsupportedSystemInterface);
                    }
                }
            }
        }
    }
    Ok(qualification)
}
