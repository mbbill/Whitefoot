//! Canonical charged-capacity and externally measured RSS ledger.

use crate::charges::{
    FeasibilityError, RECORD_FAMILY_COUNT, RecordFamily, exact_byte_layout, record_layout,
};

mod codec;
mod identity;

pub use identity::{
    APPROVED_CANDIDATE_SPECIFICATION_SHA256, APPROVED_PROPOSAL_SHA256, EvidenceDigests,
    EvidenceIdentity, EvidenceLabels, STORAGE_MODEL_SHA256, U32_IDENTITY_DOMAIN_COUNT,
    U32IdentityCounts, U32IdentityDomain,
};
use identity::{copy_identity, validate_digests, validate_identity};

#[cfg(test)]
mod tests;

/// One of the twelve lifetime-peak categories fixed by `STORAGE-MODEL.md`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PeakKind {
    /// Source records, copied paths and bytes, and duplicate-order scratch.
    SourceBundleConstruction = 0,
    /// SourceBinding copy and canonical encode/decode candidates.
    SourceBindingAndCodec = 1,
    /// Lexer count-pass fixed state.
    LexerCountPass = 2,
    /// Source plus growing lexical and boundary arrays.
    LexerEmission = 3,
    /// Source, complete lexical tape, and growing classified arrays.
    Classifier = 4,
    /// Retained classified inputs, stacks, and growing derivation.
    Parser = 5,
    /// Retained derivation, transient finalizer state, and growing topology.
    Finalizer = 6,
    /// Retained frontend state, gaps, and the largest selected node path.
    CanonicalAudit = 7,
    /// Resolution preflight plus every exact reservation.
    ResolutionPreflight = 8,
    /// One of the four mandatory resolution orderings.
    ResolutionSort = 9,
    /// Complete retained resolution state.
    RetainedResolution = 10,
    /// Largest diagnostic materialization with required tables retained.
    DiagnosticMaterialization = 11,
}

/// The four resolution orderings, in the approved construction order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ResolutionSort {
    /// Validation ordering by same-scope declaration key.
    SameScopeKey = 0,
    /// Validation ordering by region-owner key.
    RegionOwnerKey = 1,
    /// Validation ordering by match-arm binder key.
    ArmBinderKey = 2,
    /// Final production query ordering by complete lookup key.
    LookupKey = 3,
}

/// The three separately retained source-binding operations in peak category 2.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum BindingOperation {
    /// Fallible copy from the caller's ordered source records.
    SourceBindingCopy = 0,
    /// Canonical source-binding encoding with its retained input.
    CanonicalEncode = 1,
    /// Canonical source-binding decoding with its retained input bytes.
    CanonicalDecode = 2,
}

/// Stable identity of one canonical row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RowIdentity {
    /// Lifetime-peak category.
    pub peak: PeakKind,
    /// Zero except for source-binding operations and resolution-sort keys.
    /// Those two expanded categories use their respective enum ordinals.
    pub variant: u8,
}

/// Number of lifetime-peak categories.
pub const PEAK_KIND_COUNT: usize = 12;
/// Number of canonical rows after expanding three binding and four sort operations.
pub const LEDGER_ROW_COUNT: usize = 17;

/// Exact canonical row order. The four sort rows collectively implement peak 10.
pub const CANONICAL_ROW_ORDER: [RowIdentity; LEDGER_ROW_COUNT] = [
    row(PeakKind::SourceBundleConstruction, 0),
    row(
        PeakKind::SourceBindingAndCodec,
        BindingOperation::SourceBindingCopy as u8,
    ),
    row(
        PeakKind::SourceBindingAndCodec,
        BindingOperation::CanonicalEncode as u8,
    ),
    row(
        PeakKind::SourceBindingAndCodec,
        BindingOperation::CanonicalDecode as u8,
    ),
    row(PeakKind::LexerCountPass, 0),
    row(PeakKind::LexerEmission, 0),
    row(PeakKind::Classifier, 0),
    row(PeakKind::Parser, 0),
    row(PeakKind::Finalizer, 0),
    row(PeakKind::CanonicalAudit, 0),
    row(PeakKind::ResolutionPreflight, 0),
    row(PeakKind::ResolutionSort, ResolutionSort::SameScopeKey as u8),
    row(
        PeakKind::ResolutionSort,
        ResolutionSort::RegionOwnerKey as u8,
    ),
    row(PeakKind::ResolutionSort, ResolutionSort::ArmBinderKey as u8),
    row(PeakKind::ResolutionSort, ResolutionSort::LookupKey as u8),
    row(PeakKind::RetainedResolution, 0),
    row(PeakKind::DiagnosticMaterialization, 0),
];

const fn row(peak: PeakKind, variant: u8) -> RowIdentity {
    RowIdentity { peak, variant }
}

/// Exact byte families that are not record capacities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ExactChargeKind {
    /// Complete copied logical-path byte capacity.
    LogicalPathBytes = 0,
    /// Complete copied source byte capacity.
    SourceBytes = 1,
    /// Complete canonical binding byte capacity.
    BindingBytes = 2,
    /// Complete retained spelling byte capacity.
    SpellingBytes = 3,
    /// Inline vector headers live at this peak.
    VectorHeaders = 4,
    /// Explicit alignment padding not already included in a record stride.
    AlignmentPadding = 5,
    /// Fixed generated grammar tables resident at this peak.
    FixedGrammarTables = 6,
    /// Profile, capability, and other fixed control records.
    ProfileCapabilityRecords = 7,
}

impl ExactChargeKind {
    /// Every exact byte family in canonical order.
    pub const ALL: [Self; 8] = [
        Self::LogicalPathBytes,
        Self::SourceBytes,
        Self::BindingBytes,
        Self::SpellingBytes,
        Self::VectorHeaders,
        Self::AlignmentPadding,
        Self::FixedGrammarTables,
        Self::ProfileCapabilityRecords,
    ];

    /// Returns the canonical ordinal.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        self as usize
    }

    const fn is_allocation_request(self) -> bool {
        matches!(
            self,
            Self::LogicalPathBytes | Self::SourceBytes | Self::BindingBytes | Self::SpellingBytes
        )
    }
}

/// Number of exact byte families in each canonical row.
pub const EXACT_CHARGE_COUNT: usize = ExactChargeKind::ALL.len();

/// One canonical lifetime-peak row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeakRow {
    /// Required peak and subpeak identity.
    pub identity: RowIdentity,
    /// Live capacity count for every record family in canonical order.
    pub record_capacities: [u64; RECORD_FAMILY_COUNT],
    /// Live bytes for every exact family in canonical order.
    pub exact_charges: [u64; EXACT_CHARGE_COUNT],
}

impl PeakRow {
    /// Returns one all-zero accounting row with the given identity.
    ///
    /// Zero is a valid measured absence, not an omitted or approved value.
    #[must_use]
    pub const fn zeroed(identity: RowIdentity) -> Self {
        Self {
            identity,
            record_capacities: [0; RECORD_FAMILY_COUNT],
            exact_charges: [0; EXACT_CHARGE_COUNT],
        }
    }

    /// Returns the candidate record-capacity request plus exact byte-array requests.
    pub fn requested_bytes(&self) -> Result<u64, LedgerError> {
        let mut total = 0_u64;
        for family in RecordFamily::ALL {
            let count = self.record_capacities[family.ordinal()];
            let layout = record_layout(family, count).map_err(LedgerError::InfeasibleLayout)?;
            total = total
                .checked_add(layout.charged_bytes)
                .ok_or(LedgerError::ArithmeticOverflow)?;
        }
        for kind in ExactChargeKind::ALL {
            if kind.is_allocation_request() {
                let bytes = self.exact_charges[kind.ordinal()];
                exact_byte_layout(bytes).map_err(LedgerError::InfeasibleLayout)?;
                total = total
                    .checked_add(bytes)
                    .ok_or(LedgerError::ArithmeticOverflow)?;
            }
        }
        Ok(total)
    }

    /// Returns all ledger charges except the separately recorded process baseline.
    pub fn accounted_bytes(&self) -> Result<u64, LedgerError> {
        let mut total = self.requested_bytes()?;
        for kind in ExactChargeKind::ALL {
            if !kind.is_allocation_request() {
                total = total
                    .checked_add(self.exact_charges[kind.ordinal()])
                    .ok_or(LedgerError::ArithmeticOverflow)?;
            }
        }
        Ok(total)
    }
}

/// External process-level RSS evidence for this run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RssObservation {
    /// No supervised RSS value accompanies the charged-byte ledger.
    Unmeasured,
    /// Externally observed compiler-process high-water RSS.
    Measured {
        /// Maximum resident bytes reported by the named supervisor.
        high_water_bytes: u64,
    },
}

/// One canonical evidence ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeakLedger {
    /// Run and evidence identity.
    pub identity: EvidenceIdentity,
    /// Empty-process baseline from the pinned build and supervisor protocol.
    pub process_baseline_bytes: u64,
    /// Actual maxima for every representation selected as `u32`.
    pub u32_identity_counts: U32IdentityCounts,
    /// External RSS evidence, deliberately separate from modeled bytes.
    pub rss: RssObservation,
    /// Seventeen ordered rows covering all twelve lifetime-peak categories.
    pub rows: [PeakRow; LEDGER_ROW_COUNT],
}

impl PeakLedger {
    /// Checks identity, row order, arithmetic, and every host layout request.
    pub fn validate(&self) -> Result<(), LedgerError> {
        validate_digests(self.identity.digests)?;
        self.u32_identity_counts.validate()?;
        for value in self.identity.labels.values() {
            validate_identity(value)?;
        }
        if matches!(
            self.rss,
            RssObservation::Measured {
                high_water_bytes: 0
            }
        ) {
            return Err(LedgerError::InvalidRssObservation);
        }
        for (index, row) in self.rows.iter().enumerate() {
            if row.identity != CANONICAL_ROW_ORDER[index] {
                return Err(LedgerError::InvalidRowOrder);
            }
            let accounted = row.accounted_bytes()?;
            self.process_baseline_bytes
                .checked_add(accounted)
                .ok_or(LedgerError::ArithmeticOverflow)?;
        }
        Ok(())
    }

    /// Returns derived byte totals for one canonical row.
    pub fn totals(&self, index: usize) -> Result<PeakTotals, LedgerError> {
        let row = self.rows.get(index).ok_or(LedgerError::InvalidRowOrder)?;
        let requested_bytes = row.requested_bytes()?;
        let accounted_bytes = row.accounted_bytes()?;
        let modeled_process_bytes = self
            .process_baseline_bytes
            .checked_add(accounted_bytes)
            .ok_or(LedgerError::ArithmeticOverflow)?;
        Ok(PeakTotals {
            requested_bytes,
            accounted_bytes,
            modeled_process_bytes,
        })
    }
}

/// Derived totals that are never encoded as a second source of truth.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PeakTotals {
    /// Candidate capacity requests plus exact dynamic byte-array requests.
    pub requested_bytes: u64,
    /// Requested bytes plus fixed accounting charges, excluding baseline.
    pub accounted_bytes: u64,
    /// Accounted bytes plus the separately measured empty-process baseline.
    pub modeled_process_bytes: u64,
}

/// A closed ledger or codec failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LedgerError {
    /// The proposal digest was not the exact owner-approved proposal.
    WrongProposalDigest,
    /// The candidate-specification digest was not the exact approved candidate.
    WrongCandidateSpecificationDigest,
    /// The storage-model digest was not the exact model implemented here.
    WrongStorageModelDigest,
    /// The witness executable digest was an all-zero placeholder.
    MissingWitnessExecutableDigest,
    /// One identity was empty, too long, or outside graphic ASCII.
    InvalidIdentity,
    /// A row did not occupy its exact canonical position.
    InvalidRowOrder,
    /// The fixed row count was absent or changed.
    InvalidRowCount,
    /// Checked total arithmetic overflowed.
    ArithmeticOverflow,
    /// One actual identity-domain count exceeded the selected `u32` representation.
    U32IdentityExceeded,
    /// A record or byte capacity is impossible on this host.
    InfeasibleLayout(FeasibilityError),
    /// A measured RSS value was zero or its canonical tag was malformed.
    InvalidRssObservation,
    /// The exact ledger representation exceeded its closed byte ceiling.
    EncodedLengthExceeded,
    /// The small canonical output or decoded identity could not be allocated.
    StorageUnavailable,
    /// The magic domain separator was wrong.
    InvalidMagic,
    /// The codec version was wrong.
    InvalidVersion,
    /// Input ended before the next complete field.
    Truncated,
    /// A peak ordinal was outside the closed set.
    InvalidPeak,
    /// Bytes followed the final canonical row.
    TrailingBytes,
    /// Decoded fields did not reproduce the exact input bytes.
    NonCanonicalRepresentation,
}
