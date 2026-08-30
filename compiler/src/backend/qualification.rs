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
    IrInstruction, IrNominalKind, IrOperation, IrProgram, IrSystemOperation, SystemReleaseAction,
    SystemResourceContract, SystemResourceType,
};

use super::emitter::BackendFailure;

/// The number of [SYS-2] system operations, and therefore of semantic IDs.
const OPERATION_COUNT: usize = crate::SYSTEM_OPERATIONS.len();

/// The specification version this qualification table was last reviewed
/// against.
///
/// A deliberate per-activation review forcer, kept as one hand-written
/// literal on purpose: when an activation changes
/// `spec_identity::SPEC_VERSION`, `command_entry_row` reports the command
/// entry unmapped until someone re-reviews this table against the new
/// specification and bumps this constant inside the same reviewed change.
/// Exactly one such tripwire exists at the command entry because every
/// checked program in this version has exactly one command entry and reaches
/// that row even when it contains no system operation or resource. Copies in
/// `operation_row` or `resource_row` would force no additional review, only
/// additional bumps. Do not generate this constant from the specification
/// (that would delete the review it exists to force), and do not add more
/// copies.
// v0.33 review (2026-08-20): unified static contracts, mandatory symbolic
// result names, exact-integer domains, and claim-only trap ownership erase
// before target qualification and change no system representation, target
// guarantee, facility, or private machine ABI. AllocationFit is discharged
// against a target-independent ceiling before lowering and the selected
// target's actual size, alignment, and stride are checked against that ceiling
// by target-layout validation before emission; it adds no qualification row.
// The former unlabelled entry is gone: every unit uses the existing command
// bootstrap, optional command capabilities, command-lifetime backing
// guarantee, and ExitStatus mapping, with no source-callable or foreign entry.
// System declarations are globally reserved, but that visibility adds no use,
// target row, or runtime dispatch.
//
// The seven SYS-8 range-bearing operations now take proved half-open start/end
// endpoints and return absolute success endpoints. Their wrapper arity, scalar
// widths, aggregate result layouts, host facilities, and resource rows are
// unchanged; the statically discharged obligations leave no runtime trap.
// `directory_next` now normalizes one kind byte plus a little-endian u16 name length,
// and the target record fixes the Darwin-family component limit at 1023 and
// the Linux-family limit at 255 while retaining the reviewed Darwin record
// offsets and fail-closed missing Linux enumeration mapping. The new
// `open_file` semantic ID at ordinal 14 uses the existing directory-relative
// and lossless-byte guarantees, opens no-follow/nonblocking through `openat`,
// validates the provisional descriptor through the target's `fstat` ABI,
// publishes only the existing `ReadFile` representation, and applies the
// existing one-attempt close policy on either post-open error. All operation
// rows remain statically selected implementation version 1 under the v0.33
// semantic-ID key.
//
// v0.34 review (2026-08-21): claim locality and residual claim canonicality
// tightened which claims source may express. They add no system operation,
// resource type, semantic ID, target guarantee, host facility, representation,
// release action, private machine ABI record, or entry contract, and a claim
// erases before target qualification either way, so every row above stands
// exactly as reviewed for v0.33 under the v0.34 semantic-ID key.
//
// v0.35 review (2026-08-23): the candidate's whole delta is the added [PAR-1]
// execution-overlap rule. It likewise adds no system operation, resource type,
// semantic ID, target guarantee, host facility, representation, release action,
// private machine ABI record, or entry contract, so every row above stands as
// reviewed and each remains statically selected implementation version 1, now
// under the v0.35 semantic-ID key.
//
// [PAR-1] may now include a direct system operation. Static qualification
// still selects only the target implementation of that semantic identity;
// permission separately proves ordinary state paths, loans, dataflow, and
// exits. A `may-suspend` record selects completion lowering only
// when the backend has a typed adapter for the exact operation. Otherwise the
// valid source retains its sequential qualified call or reports unsupported
// target support; no row is fabricated and no source rejection changes.
//
// Runtime queues, operation slots, and scheduler frames carry no semantic ID
// of their own. A file helper accepts a closed typed target descriptor, never
// an outlined writer function. Native completion and the bounded helper
// fallback therefore implement rows selected here without becoming alternate
// system declarations or a second qualification path.
// v0.37 review (2026-08-27): source effects now name concrete state
// parameters while target suspension remains compiler-owned. FileFactory and
// FilePermit use a proof-only bit representation; reserve_file returns that
// harmless value inline, and the open wrappers erase it before the native ABI.
// The three renamed operations retain ordinals 8, 12,
// and 13. Ordinal 8 adds one u64 file offset and binds to pread instead of
// cursor-mutating read. DirectorySource keeps the descriptor representation
// and close release of the former traversal source. Interrupted and WouldBlock
// leave the portable IoError set: no-progress interruption and readiness
// refusal remain target progress and are not passed to the portable error
// mapper. Completion dispatch is an emitted execution choice after this
// static target qualification and adds no dynamic operation mapping.
// v0.38 review (2026-08-27): the added rule [PAR-3] is a permission over
// the iterations of a loop and names no target facility. It adds no system
// operation, no opaque resource type, no release action, no outcome
// constructor, and no entry form, and it changes no signature, borrow mode,
// or effect row this table maps. Its target-visible sentences — that the host
// resources a system operation of the loop creates are not execution resources
// an implementation spends on overlapping, and that the remainder's accesses to
// storage rooted outside the body are taken in iteration order — constrain a
// schedule an implementation may take, not a facility this qualification
// approves, and nothing in this version emits that schedule. The same version
// amends [SYS-2] in one sentence to name the release milestone of the name an
// open borrows, which is published before target transfer. That is a milestone
// of the operation record rather than a facility: it adds no operation, no
// resource type, no release action, no outcome constructor, and no entry form,
// it changes no signature, borrow mode, or effect row this table maps, and it
// is the fact both shipped adapters already publish at submission — the POSIX
// file adapter after `wf_file_work_bind_path`, the io_uring adapter after
// target acceptance. Every v0.37 mapping therefore stands unchanged.
// v0.39 review (2026-08-28): the single changed paragraph narrows [CLM-1]'s
// claim-authority control dependence, a front-end source-acceptance judgment
// over one function's own definitions. It names no target facility: no system
// operation, opaque resource type, release action, outcome constructor, or
// entry form is added or removed, no signature, borrow mode, or effect row
// this table maps changes, and no host guarantee is newly required. A claim
// this version newly admits lowers to the same executed check-else-trap every
// admitted claim already lowers to, so the trap and diagnostic surfaces a
// target must supply are the ones already qualified here. Every v0.38 mapping
// therefore stands unchanged.
// v0.40 review (2026-08-30): S5's post-SET-1 value image and ENT-5's
// close-before-lexical-kill order are front-end proof rules. They add no
// system operation, resource type, release action, outcome constructor,
// entry form, target guarantee, signature, borrow mode, effect row, or host
// ABI. Both rules erase before lowering; they can only admit the same raw
// partial-operation instruction after its existing static obligation has a
// derivation. Every v0.39 target mapping therefore stands unchanged.
const REVIEWED_FOR: &str = "v0.40";

/// The number of [SYS-2] opaque resource types, including the
/// traversal-surface candidate's `DirectorySource`.
const RESOURCE_COUNT: usize = 10;

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
    /// through, and the facility a qualified wrapper disposes of a
    /// provisional descriptor through.
    const fn close(self) -> &'static str {
        match self {
            Self::Native => "wf__completion_file_close_direct",
            #[cfg(test)]
            Self::DeterministicTest => "wf_test_close",
        }
    }

    /// The directory-relative open facility `open_read` resolves through
    /// [PATH-2].
    const fn file_open(self) -> &'static str {
        match self {
            Self::Native => "wf__completion_file_open_at_direct",
            #[cfg(test)]
            Self::DeterministicTest => "wf_test_openat",
        }
    }

    /// The descriptor-status facility used to validate that `open_file`
    /// produced the regular-file value its semantic row promises.
    const fn file_status(self, _native: &'static str) -> &'static str {
        match self {
            Self::Native => "wf__completion_file_status_direct",
            #[cfg(test)]
            Self::DeterministicTest => "wf_test_fstat",
        }
    }

    const fn uses_typed_completion_file_adapter(self) -> bool {
        match self {
            Self::Native => true,
            #[cfg(test)]
            Self::DeterministicTest => false,
        }
    }

    /// The facility one positioned `read_at` transfer attempt reaches.
    const fn pread(self) -> &'static str {
        match self {
            Self::Native => "wf__completion_file_pread_direct",
            #[cfg(test)]
            Self::DeterministicTest => "wf_test_pread",
        }
    }

    /// The facility one `write_once` transfer attempt reaches.
    const fn write(self) -> &'static str {
        match self {
            Self::Native => "wf__completion_file_write_direct",
            #[cfg(test)]
            Self::DeterministicTest => "wf_test_write",
        }
    }
}

/// The [FN-7] program kind one build produces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProgramKind {
    /// A natively compiled `command`.
    Command,
}

/// One portable [SYS-7] class and the native error codes a target maps onto
/// it.
///
/// A target's table is the complete closed twenty-eight-class set in [SYS-2]
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
const DARWIN_ERROR_CLASSES: [PortableErrorClass; 28] = [
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
const LINUX_ERROR_CLASSES: [PortableErrorClass; 28] = [
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
/// The target-owned `origin` discriminator of descriptor-status inspection.
pub(crate) const ORIGIN_DESCRIPTOR_STATUS: u8 = 4;
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
    /// The target's own directory-enumeration facility [SYS-14]: one
    /// progress-producing transfer reports a bounded batch of entry records
    /// against an open directory descriptor and advances that descriptor's
    /// own position. Interruption and readiness refusal remain target
    /// scheduling state and may repeat before that transfer.
    DirectoryEnumeration,
}

/// How one target's native entry record states the byte length of its name.
///
/// The portable record [SYS-14] fixes carries an explicit `name_length`, so
/// the shim needs that number for every entry. Where the number comes from is
/// the one part of the record model that is genuinely different between the
/// two qualified families, so it is asked of the target rather than assumed:
/// Darwin's `struct dirent` states it in a field of its own, and Linux's
/// `struct linux_dirent64` states no length at all and NUL-terminates the
/// name inside the record's own extent. A model with a mandatory length field
/// would have no way to describe the second, which is exactly why this
/// compiler had no Linux enumeration row until the shape below existed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EntryNameLength {
    /// The record states the name's byte length in a `u16` at this offset.
    Field {
        /// Byte offset of that `u16`.
        offset: u64,
    },
    /// The record states no name length. The name begins at the record's name
    /// offset and ends at the first NUL byte strictly inside the extent the
    /// record's own length field reports, so the length is derived by one
    /// bounded scan that never reads past that extent.
    NulTerminated,
}

/// One target's directory-enumeration facility and the exact record layout it
/// fills [SYS-14, QUAL-1].
///
/// Every field is target data read only by the `directory_next` implementation:
/// the emitted shim walks the native records this facility wrote and
/// normalizes them into the portable
/// `[kind][little-endian u16 name length][name bytes]` form
/// [SYS-14] fixes. A target with no such facility supplies no record here and
/// fails qualification for the enumeration semantic IDs rather than emulating
/// them with a directory-reading loop of its own.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DirectoryEnumeration {
    /// The native facility behind one admitted `directory_next` operation.
    symbol: &'static str,
    /// That symbol's declaration.
    declaration: &'static str,
    /// Byte offset of the native record's own length, a `u16`.
    record_length_offset: u64,
    /// Where the entry name's byte length comes from.
    name_length: EntryNameLength,
    /// Byte offset of the native entry-type discriminant, a `u8`.
    entry_type_offset: u64,
    /// Byte offset of the entry name's first byte.
    name_offset: u64,
    /// The native entry-type value naming a regular file.
    native_regular: u64,
    /// The native entry-type value naming a directory.
    native_directory: u64,
    /// The native entry-type value naming a symbolic link.
    native_symlink: u64,
    /// The native entry-type value meaning the target did not classify the
    /// entry.
    native_unknown: u64,
}

impl DirectoryEnumeration {
    /// The native facility behind one admitted `directory_next` operation.
    pub(crate) const fn symbol(self) -> &'static str {
        self.symbol
    }

    /// That symbol's declaration.
    pub(crate) const fn declaration(self) -> &'static str {
        self.declaration
    }

    /// Byte offset of the native record's own length, a `u16`.
    pub(crate) const fn record_length_offset(self) -> u64 {
        self.record_length_offset
    }

    /// Where the entry name's byte length comes from.
    pub(crate) const fn name_length(self) -> EntryNameLength {
        self.name_length
    }

    /// Byte offset of the native entry-type discriminant, a `u8`.
    pub(crate) const fn entry_type_offset(self) -> u64 {
        self.entry_type_offset
    }

    /// Byte offset of the entry name's first byte.
    pub(crate) const fn name_offset(self) -> u64 {
        self.name_offset
    }

    /// The native entry-type value naming a regular file.
    pub(crate) const fn native_regular(self) -> u64 {
        self.native_regular
    }

    /// The native entry-type value naming a directory.
    pub(crate) const fn native_directory(self) -> u64 {
        self.native_directory
    }

    /// The native entry-type value naming a symbolic link.
    pub(crate) const fn native_symlink(self) -> u64 {
        self.native_symlink
    }

    /// The native entry-type value meaning the target did not classify the
    /// entry.
    pub(crate) const fn native_unknown(self) -> u64 {
        self.native_unknown
    }
}

/// The Darwin-family enumeration facility.
///
/// `__getdirentries64` is the one libSystem entry that reports a batch of
/// 64-bit-inode directory records against an open descriptor and advances it;
/// `getdirentries` is unavailable on a 64-bit-inode target and `opendir`
/// plus `readdir` is two calls with an allocation, which [QUAL-3] excludes.
/// The offsets are `struct dirent`'s measured Darwin layout: `d_ino` 0,
/// `d_seekoff` 8, `d_reclen` 16, `d_namlen` 18, `d_type` 20, `d_name` 21.
const DARWIN_ENUMERATION: DirectoryEnumeration = DirectoryEnumeration {
    symbol: "__getdirentries64",
    declaration: "declare i64 @__getdirentries64(i32, ptr, i64, ptr)",
    record_length_offset: 16,
    name_length: EntryNameLength::Field { offset: 18 },
    entry_type_offset: 20,
    name_offset: 21,
    // `DT_REG`, `DT_DIR`, `DT_LNK`, `DT_UNKNOWN`.
    native_regular: 8,
    native_directory: 4,
    native_symlink: 10,
    native_unknown: 0,
};

/// The Linux-family enumeration facility.
///
/// `getdents64` is the one host call that reports a batch of the entries of an
/// open directory and advances that descriptor's own position, which is
/// exactly the [QUAL-2] guarantee. `readdir` is a library scan built out of
/// other operations with an allocation and a per-entry call, which [QUAL-3]
/// excludes, and the legacy `getdents` reports a record this family's
/// 64-bit inodes and offsets do not fit.
///
/// The offsets are `struct linux_dirent64`'s layout, which is
/// architecture-independent: `d_ino` 0, `d_off` 8, `d_reclen` 16, `d_type` 18,
/// `d_name` 19. The record states no name length; `d_name` is NUL-terminated
/// and `d_reclen` is padded to the record's own alignment, so a name's length
/// is neither `d_reclen - 19` nor anything else derivable without reading the
/// name. That is what `EntryNameLength::NulTerminated` says, and it is the
/// whole reason this row could not be written against a record model with a
/// mandatory length field.
const LINUX_ENUMERATION: DirectoryEnumeration = DirectoryEnumeration {
    symbol: "getdents64",
    declaration: "declare i64 @getdents64(i32, ptr, i64)",
    record_length_offset: 16,
    name_length: EntryNameLength::NulTerminated,
    entry_type_offset: 18,
    name_offset: 19,
    // `DT_REG`, `DT_DIR`, `DT_LNK`, `DT_UNKNOWN`. The two families share these
    // four values; they are still stated per target, because sharing them is
    // an observation about today's rows rather than a promise a third family
    // would keep.
    native_regular: 8,
    native_directory: 4,
    native_symlink: 10,
    native_unknown: 0,
};

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
    /// A proof-only affine value with no target resource representation.
    ProofToken,
}

impl ResourceRepresentation {
    /// The representation's size in bytes on a qualified target.
    pub(crate) const fn size(self) -> u64 {
        match self {
            Self::InlineLease | Self::ArgumentVector => 16,
            Self::Descriptor => 4,
            Self::CommandCode | Self::ProofToken => 1,
        }
    }

    /// The representation's alignment in bytes on a qualified target.
    pub(crate) const fn align(self) -> u64 {
        match self {
            Self::InlineLease | Self::ArgumentVector => 8,
            Self::Descriptor => 4,
            Self::CommandCode | Self::ProofToken => 1,
        }
    }

    /// The representation's emitted LLVM type.
    pub(crate) const fn llvm(self) -> &'static str {
        match self {
            Self::InlineLease | Self::ArgumentVector => "{ ptr, i64 }",
            Self::Descriptor => "i32",
            Self::CommandCode => "i8",
            Self::ProofToken => "i1",
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
    /// Whether the target family has a directory-enumeration facility at
    /// all, independently of whether this compiler has an approved ABI
    /// record mapping for it.
    directory_enumeration_facility: bool,
    /// The target's own directory-enumeration facility, when it has one
    /// [SYS-14].
    directory_enumeration: Option<DirectoryEnumeration>,
    /// Which host facilities this target's approved implementations call.
    host: HostFacilities,
    root_prefix: u8,
    /// The greatest byte length of one directory component admitted by this
    /// target's filesystem-facing ABI.
    component_limit: u64,
    /// Bootstrap and self-component directory opens.
    directory_open_flags: i32,
    /// `open_read`'s namespace-following relative-path open.
    file_open_flags: i32,
    /// Single-component `open_directory`, which must not follow its terminal
    /// symbolic link.
    component_directory_open_flags: i32,
    /// Single-component `open_file`, which must not follow its terminal
    /// symbolic link or block while a non-regular object is rejected.
    component_file_open_flags: i32,
    /// The target `struct stat` size and `st_mode` byte offset.
    file_status_size: u64,
    file_status_mode_offset: u64,
    native_file_status_symbol: &'static str,
    errno_location: &'static str,
    errno_declaration: &'static str,
    error_classes: &'static [PortableErrorClass; 28],
    broken_pipe_signal: i32,
    ignored_disposition: i64,
    invalid_disposition: i64,
}

impl SystemTarget {
    /// Whether this target's qualified read/write facilities are the POSIX
    /// descriptor operations implemented by the first typed completion
    /// adapter.  Scripted deterministic targets keep their own direct
    /// implementation and therefore never enter the native adapter.
    pub(crate) const fn supports_posix_file_completion(self) -> bool {
        matches!(self.host, HostFacilities::Native)
    }

    /// The single code unit sequence a Unix-family target resolves against a
    /// filesystem root, and therefore the complete [PATH-1] target-root prefix
    /// set of this family: one leading separator.
    pub(crate) const fn root_prefix(self) -> u8 {
        self.root_prefix
    }

    /// The selected target's maximum component length in bytes.
    pub(crate) const fn component_limit(self) -> u64 {
        self.component_limit
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

    /// Flags for a no-follow single-component directory open.
    pub(crate) const fn component_directory_open_flags(self) -> i32 {
        self.component_directory_open_flags
    }

    /// Flags for a no-follow, nonblocking single-component file open.
    pub(crate) const fn component_file_open_flags(self) -> i32 {
        self.component_file_open_flags
    }

    /// The target descriptor-status facility and record coordinates used to
    /// validate `open_file` before the resource reaches source.
    pub(crate) const fn file_status_symbol(self) -> &'static str {
        self.host.file_status(self.native_file_status_symbol)
    }

    pub(crate) const fn file_status_size(self) -> u64 {
        self.file_status_size
    }

    pub(crate) const fn file_status_mode_offset(self) -> u64 {
        self.file_status_mode_offset
    }

    /// Whether the selected target reaches file open/status/close through the
    /// typed completion adapter ABI rather than the deterministic test shim.
    pub(crate) const fn uses_typed_completion_file_adapter(self) -> bool {
        self.host.uses_typed_completion_file_adapter()
    }

    /// The same target close facility resource release uses. An `open_file`
    /// classification failure consumes the provisional descriptor here.
    pub(crate) const fn close_symbol(self) -> &'static str {
        self.host.close()
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
    pub(crate) const fn error_classes(self) -> &'static [PortableErrorClass; 28] {
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

    /// The host facility one positioned `read_at` attempt reaches [SYS-8].
    pub(crate) const fn pread_symbol(self) -> &'static str {
        self.host.pread()
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

    /// The target's own directory-enumeration facility [SYS-14].
    ///
    /// Emission may read it directly only after qualification accepted the
    /// program, which is exactly when the enumeration guarantee held.
    pub(crate) const fn directory_enumeration(self) -> Option<DirectoryEnumeration> {
        self.directory_enumeration
    }

    fn supplies(self, guarantee: TargetGuarantee) -> bool {
        match guarantee {
            TargetGuarantee::CommandLifetimeArgumentBacking => self.argument_backing,
            TargetGuarantee::LosslessCodeUnits => self.family.is_some(),
            TargetGuarantee::DirectoryRelativeResolution => self.directory_relative,
            TargetGuarantee::DirectoryEnumeration => self.directory_enumeration_facility,
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
        let (
            component_limit,
            directory_open_flags,
            component_directory_open_flags,
            component_file_open_flags,
            file_status_size,
            file_status_mode_offset,
            native_file_status_symbol,
            errno_location,
            errno_declaration,
            error_classes,
            directory_enumeration_facility,
            directory_enumeration,
        ) = match triple {
            // Darwin: O_DIRECTORY, O_NOFOLLOW, and O_NONBLOCK.
            "aarch64-apple-darwin" => (
                1023,
                0x0010_0000,
                0x0010_0100,
                0x0000_0104,
                144,
                4,
                "fstat",
                "__error",
                "declare ptr @__error()",
                &DARWIN_ERROR_CLASSES,
                true,
                Some(DARWIN_ENUMERATION),
            ),
            // x86_64 Darwin uses the inode64-decorated ABI symbol for the
            // modern 144-byte `struct stat`; arm64 exports that ABI as fstat.
            "x86_64-apple-darwin" => (
                1023,
                0x0010_0000,
                0x0010_0100,
                0x0000_0104,
                144,
                4,
                "fstat$INODE64",
                "__error",
                "declare ptr @__error()",
                &DARWIN_ERROR_CLASSES,
                true,
                Some(DARWIN_ENUMERATION),
            ),
            // Linux aarch64 uses the asm-generic O_DIRECTORY/O_NOFOLLOW
            // values; they differ from x86_64 and therefore retain their own
            // target row. The enumeration record is the same on both, because
            // `struct linux_dirent64` is architecture-independent.
            "aarch64-unknown-linux-gnu" => (
                255,
                0x0000_4000,
                0x0000_c000,
                0x0000_8800,
                128,
                16,
                "fstat",
                "__errno_location",
                "declare ptr @__errno_location()",
                &LINUX_ERROR_CLASSES,
                true,
                Some(LINUX_ENUMERATION),
            ),
            // Linux x86_64: O_DIRECTORY, O_NOFOLLOW, and O_NONBLOCK.
            "x86_64-unknown-linux-gnu" => (
                255,
                0x0001_0000,
                0x0003_0000,
                0x0002_0800,
                144,
                24,
                "fstat",
                "__errno_location",
                "declare ptr @__errno_location()",
                &LINUX_ERROR_CLASSES,
                true,
                Some(LINUX_ENUMERATION),
            ),
            _ => return None,
        };
        Some(Self {
            family: Some(CodeUnitFamily::Unix),
            argument_backing: true,
            directory_relative: true,
            directory_enumeration_facility,
            directory_enumeration,
            host: HostFacilities::Native,
            root_prefix: b'/',
            component_limit,
            directory_open_flags,
            // `O_RDONLY` is zero on both families. `open_read` opens for
            // reading only and adds no creation, truncation, or mode flag:
            // [SYS-11] creates one live readable file and nothing else.
            file_open_flags: 0,
            component_directory_open_flags,
            component_file_open_flags,
            file_status_size,
            file_status_mode_offset,
            native_file_status_symbol,
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

    /// A probe target with no directory-enumeration facility at all.
    ///
    /// This is [QUAL-2]'s fourth guarantee withheld: such a target fails
    /// qualification for the enumeration semantic IDs rather than having a
    /// scan built for it out of other operations. It is a separate builder
    /// because it also drops the record, and a target that reported a record
    /// while denying the facility would be describing something that does not
    /// exist.
    #[cfg(test)]
    pub(crate) fn probe_without_enumeration() -> Self {
        let mut target = Self::for_triple("aarch64-apple-darwin")
            .expect("the probe base triple is a qualified target");
        target.directory_enumeration_facility = false;
        target.directory_enumeration = None;
        target
    }

    /// A probe target whose family has a directory-enumeration facility but
    /// for which this compiler holds no approved ABI record.
    ///
    /// The guarantee is met, so the refusal is the missing mapping rather than
    /// an unmet guarantee: `operation_row` fails the enumeration semantic IDs
    /// with `MissingMapping` instead of building a scan out of other
    /// operations [QUAL-1]. This is the state the superseded Linux case
    /// exercised while no Linux row existed.
    #[cfg(test)]
    pub(crate) fn probe_without_enumeration_record() -> Self {
        let mut target = Self::for_triple("aarch64-apple-darwin")
            .expect("the probe base triple is a qualified target");
        target.directory_enumeration = None;
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
        // A directory enumeration is one native descriptor whose whole cursor
        // state is the descriptor's own file position, so it needs no value
        // component of its own [SYS-14].
        SystemResourceType::DirectoryRead
        | SystemResourceType::ReadFile
        | SystemResourceType::Output
        | SystemResourceType::DirectorySource => ResourceRepresentation::Descriptor,
        SystemResourceType::ExitStatus => ResourceRepresentation::CommandCode,
        SystemResourceType::FileFactory | SystemResourceType::FilePermit => {
            ResourceRepresentation::ProofToken
        }
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
        SystemResourceType::DirectorySource => 7,
        SystemResourceType::FileFactory => 8,
        SystemResourceType::FilePermit => 9,
    }
}

/// The [QUAL-2] guarantees one semantic identity's record requires.
fn operation_guarantees(operation: u8) -> &'static [TargetGuarantee] {
    use TargetGuarantee::{
        CommandLifetimeArgumentBacking, DirectoryEnumeration, DirectoryRelativeResolution,
        LosslessCodeUnits,
    };
    const ARGUMENTS: &[TargetGuarantee] = &[CommandLifetimeArgumentBacking];
    const ARGUMENT_STRING: &[TargetGuarantee] =
        &[CommandLifetimeArgumentBacking, LosslessCodeUnits];
    const STRINGS: &[TargetGuarantee] = &[LosslessCodeUnits];
    const DIRECTORY: &[TargetGuarantee] = &[LosslessCodeUnits, DirectoryRelativeResolution];
    const ENUMERATION: &[TargetGuarantee] = &[DirectoryRelativeResolution, DirectoryEnumeration];
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
        // `open_directory` resolves one component name through the same
        // facility [SYS-14], and `open_file` resolves one
        // component name for a file through it [SYS-11].
        11 | 14 => DIRECTORY,
        // `open_directory_source` and `directory_next` additionally require the target's own
        // enumeration facility [SYS-14].
        12 | 13 => ENUMERATION,
        // `read_at`, `write_once`, and `exit_status` require neither.
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
        // An enumeration handle names one directory object it was opened
        // against, so it inherits the same directory-relative guarantee
        // [PATH-2, SYS-14].
        SystemResourceType::DirectorySource => DIRECTORY,
        SystemResourceType::ReadFile
        | SystemResourceType::Output
        | SystemResourceType::ExitStatus
        | SystemResourceType::FileFactory
        | SystemResourceType::FilePermit => &[],
    }
}

/// The `(specification version, semantic ID, target, program kind)` row.
fn operation_row(
    operation: u8,
    target: SystemTarget,
    kind: ProgramKind,
) -> Result<ApprovedImplementation, QualificationFailure> {
    let facility = Facility::Operation(operation);
    if usize::from(operation) >= OPERATION_COUNT {
        return Err(QualificationFailure::MissingMapping(facility));
    }
    // [SYS-3] makes every operation name available in every unit, while
    // executable qualification follows the one entry kind this version
    // defines: `command` [FN-7].
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
    if matches!(operation, 12 | 13) && target.directory_enumeration().is_none() {
        return Err(QualificationFailure::MissingMapping(facility));
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
        8 => "wf.sys.read_at.v1",
        9 => "wf.sys.write_once.v1",
        10 => "wf.sys.exit_status.v1",
        11 => "wf.sys.open_directory.v1",
        12 => "wf.sys.open_directory_source.v1",
        13 => "wf.sys.directory_next.v1",
        14 => "wf.sys.open_file.v1",
        15 => "wf.sys.reserve_file.v1",
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
    // The single per-activation review tripwire lives in `command_entry_row`
    // (`REVIEWED_FOR`); a duplicate version guard here forced no extra review.
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
        // source value without closing or flushing the descriptor
        // [SYS-12], and every other type releases with a logical consume.
        SystemResourceType::DirectoryRead
        | SystemResourceType::ReadFile
        | SystemResourceType::DirectorySource => {
            ReleaseImplementation::NativeClose(target.host.close())
        }
        SystemResourceType::Args
        | SystemResourceType::HostString
        | SystemResourceType::RelativePath
        | SystemResourceType::Output
        | SystemResourceType::ExitStatus
        | SystemResourceType::FileFactory
        | SystemResourceType::FilePermit => ReleaseImplementation::NoCode,
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
fn command_entry_row(
    target: SystemTarget,
    specification_version: &str,
) -> Result<(), QualificationFailure> {
    if specification_version != REVIEWED_FOR {
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
    let kind = ProgramKind::Command;
    let mut qualification = Qualification {
        target,
        kind,
        operations: [None; OPERATION_COUNT],
        resources: [None; RESOURCE_COUNT],
    };
    command_entry_row(target, crate::spec_identity::SPEC_VERSION)
        .map_err(BackendFailure::TargetQualification)?;
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

#[cfg(test)]
mod tests {
    use super::{Facility, QualificationFailure, SystemTarget, command_entry_row};

    #[test]
    fn command_entry_is_the_single_specification_review_tripwire() {
        let target = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("the probe target is qualified");
        assert_eq!(
            command_entry_row(target, "v0.34"),
            Err(QualificationFailure::MissingMapping(Facility::CommandEntry))
        );
        command_entry_row(target, crate::spec_identity::SPEC_VERSION)
            .expect("the reviewed active version qualifies its command entry");
    }
}
