//! Exact evidence identities and narrow integer-representation receipts.

use super::LedgerError;

const MAX_IDENTITY_LENGTH: usize = 255;

/// Exact owner-approved Phase-5 successor proposal SHA-256.
pub const APPROVED_PROPOSAL_SHA256: [u8; 32] = [
    0x7f, 0xc4, 0x8c, 0xc3, 0x0f, 0x94, 0xd2, 0x5b, 0xe5, 0xbe, 0x11, 0x06, 0xe3, 0x26, 0x5d, 0x92,
    0xc1, 0xb0, 0xcd, 0xf2, 0xbf, 0xea, 0x5a, 0x7a, 0x17, 0x75, 0x9a, 0x12, 0xf3, 0xcf, 0x09, 0x2d,
];

/// Exact generated successor candidate SHA-256 approved for evidence work.
pub const APPROVED_CANDIDATE_SPECIFICATION_SHA256: [u8; 32] = [
    0x71, 0x07, 0x3e, 0x25, 0x21, 0x94, 0x55, 0x89, 0x62, 0x50, 0xe1, 0x5e, 0x13, 0xd1, 0xff, 0xdb,
    0xfc, 0x44, 0x3c, 0x87, 0xa9, 0xb2, 0x8c, 0xb9, 0x90, 0x6d, 0x73, 0xa0, 0x20, 0xdc, 0x33, 0xe9,
];

/// Exact SHA-256 of the storage-model bytes implemented by this witness.
pub const STORAGE_MODEL_SHA256: [u8; 32] = [
    0xee, 0x6e, 0x8c, 0xd0, 0xdd, 0x70, 0xd8, 0x1e, 0xaa, 0x0c, 0xa1, 0x1d, 0xb4, 0x61, 0x4e, 0x38,
    0x77, 0xaf, 0xce, 0x12, 0x41, 0xe9, 0x41, 0x3a, 0x7d, 0xd9, 0x86, 0x3a, 0xeb, 0x4f, 0x31, 0x39,
];

/// Cryptographic identities supplied by the evidence supervisor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvidenceDigests {
    /// Exact owner-approved successor proposal SHA-256.
    pub proposal: [u8; 32],
    /// Exact generated successor candidate SHA-256.
    pub candidate_specification: [u8; 32],
    /// Exact `STORAGE-MODEL.md` SHA-256 used for this run.
    pub storage_model: [u8; 32],
    /// Exact witness executable SHA-256 measured before this run.
    pub witness_executable: [u8; 32],
}

/// Bounded graphic-ASCII identities supplied by the evidence supervisor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceLabels {
    /// Selected compiler-host class identifier.
    pub host_class: String,
    /// Exact observed OS and machine identity.
    pub observed_host: String,
    /// Exact pinned toolchain identity.
    pub toolchain: String,
    /// Exact allocator identity and configuration.
    pub allocator: String,
    /// External RSS supervisor identity.
    pub supervisor: String,
}

impl EvidenceLabels {
    /// Validates and fallibly copies all five closed identity strings.
    pub fn try_new(values: [&str; 5]) -> Result<Self, LedgerError> {
        Ok(Self {
            host_class: copy_identity(values[0])?,
            observed_host: copy_identity(values[1])?,
            toolchain: copy_identity(values[2])?,
            allocator: copy_identity(values[3])?,
            supervisor: copy_identity(values[4])?,
        })
    }

    pub(super) fn values(&self) -> [&str; 5] {
        [
            &self.host_class,
            &self.observed_host,
            &self.toolchain,
            &self.allocator,
            &self.supervisor,
        ]
    }
}

/// Complete identity of one peak-ledger run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceIdentity {
    /// Exact byte identities.
    pub digests: EvidenceDigests,
    /// Closed host, toolchain, allocator, and supervisor identities.
    pub labels: EvidenceLabels,
    /// Monotonic run ordinal within the surrounding evidence manifest.
    pub run_ordinal: u64,
}

/// An actual-count domain whose production representation is fixed to `u32`.
///
/// Future resolver IDs are deliberately absent because no resolver record
/// representation exists yet. Selecting another `u32` dense ID requires a new
/// ledger codec version and an added domain here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum U32IdentityDomain {
    /// Actual logical source count underlying every `SourceId`.
    Sources = 0,
    /// Actual production-node count underlying every `NodeId`.
    ProductionNodes = 1,
    /// Largest actual direct production-child count in one node.
    MaximumProductionChildren = 2,
    /// Actual complete mixed-element count addressing mixed ranges.
    MixedElements = 3,
    /// Largest actual mixed-range start stored in a node record.
    MaximumMixedStart = 4,
    /// Largest actual mixed-range count stored in a node record.
    MaximumMixedCount = 5,
    /// Largest actual production-parent depth stored in a node record.
    MaximumTreeDepth = 6,
    /// Largest actual FORM-2 format depth stored in a node record.
    MaximumFormatDepth = 7,
}

impl U32IdentityDomain {
    /// Every selected `u32` domain in canonical codec order.
    pub const ALL: [Self; 8] = [
        Self::Sources,
        Self::ProductionNodes,
        Self::MaximumProductionChildren,
        Self::MixedElements,
        Self::MaximumMixedStart,
        Self::MaximumMixedCount,
        Self::MaximumTreeDepth,
        Self::MaximumFormatDepth,
    ];

    /// Returns the canonical ordinal.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        self as usize
    }
}

/// Number of actual-count domains fixed to `u32` in ledger codec version 1.
pub const U32_IDENTITY_DOMAIN_COUNT: usize = U32IdentityDomain::ALL.len();

/// Exact actual maxima for every representation selected as `u32`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct U32IdentityCounts {
    /// Values in `U32IdentityDomain::ALL` order.
    pub values: [u64; U32_IDENTITY_DOMAIN_COUNT],
}

impl U32IdentityCounts {
    /// Checks every actual value against the selected representation.
    pub fn validate(self) -> Result<(), LedgerError> {
        if self.values.iter().any(|value| *value > u64::from(u32::MAX)) {
            return Err(LedgerError::U32IdentityExceeded);
        }
        Ok(())
    }

    /// Returns one actual value by its closed identity domain.
    #[must_use]
    pub const fn get(self, domain: U32IdentityDomain) -> u64 {
        self.values[domain.ordinal()]
    }
}

pub(super) fn validate_digests(digests: EvidenceDigests) -> Result<(), LedgerError> {
    if digests.proposal != APPROVED_PROPOSAL_SHA256 {
        return Err(LedgerError::WrongProposalDigest);
    }
    if digests.candidate_specification != APPROVED_CANDIDATE_SPECIFICATION_SHA256 {
        return Err(LedgerError::WrongCandidateSpecificationDigest);
    }
    if digests.storage_model != STORAGE_MODEL_SHA256 {
        return Err(LedgerError::WrongStorageModelDigest);
    }
    if digests.witness_executable.iter().all(|byte| *byte == 0) {
        return Err(LedgerError::MissingWitnessExecutableDigest);
    }
    Ok(())
}

pub(super) fn validate_identity(value: &str) -> Result<(), LedgerError> {
    if value.is_empty()
        || value.len() > MAX_IDENTITY_LENGTH
        || !value.bytes().all(|byte| (0x21..=0x7e).contains(&byte))
    {
        return Err(LedgerError::InvalidIdentity);
    }
    Ok(())
}

pub(super) fn copy_identity(value: &str) -> Result<String, LedgerError> {
    validate_identity(value)?;
    let mut copied = String::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|_| LedgerError::StorageUnavailable)?;
    copied.push_str(value);
    Ok(copied)
}
