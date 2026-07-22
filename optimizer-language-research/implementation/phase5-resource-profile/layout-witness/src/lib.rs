#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Non-authoritative ResourceProfile v1 host-layout and peak-ledger witness.
//!
//! This crate checks candidate storage charges and defines evidence bytes. It
//! selects no resource maximum and grants no language or compiler authority.

pub mod charges;
pub mod host;
pub mod layouts;
pub mod ledger;

pub use charges::{
    AllocationLayout, FeasibilityError, RECORD_CHARGES, RECORD_FAMILY_COUNT, RecordCharge,
    RecordFamily, exact_byte_layout, record_layout,
};
pub use host::{
    ALLOCATOR_ASSUMPTION, BuildIdentity, HostClassError, MINIMUM_PHYSICAL_MEMORY_BYTES,
    MODELED_PROCESS_TARGET_BYTES, SUPERVISED_RSS_CEILING_BYTES, SUPERVISOR_OBLIGATIONS,
    SUPPORTED_RUSTC_RELEASE, SUPPORTED_TARGET,
};
pub use layouts::{
    ObservedLayout, PRIVATE_FRONTEND_RECORDS_REQUIRING_IN_CRATE_ASSERTIONS, observed_public_layouts,
};
pub use ledger::{
    APPROVED_CANDIDATE_SPECIFICATION_SHA256, APPROVED_PROPOSAL_SHA256, BindingOperation,
    CANONICAL_ROW_ORDER, EXACT_CHARGE_COUNT, EvidenceDigests, EvidenceIdentity, EvidenceLabels,
    ExactChargeKind, LEDGER_ROW_COUNT, LedgerError, PEAK_KIND_COUNT, PeakKind, PeakLedger, PeakRow,
    PeakTotals, ResolutionSort, RowIdentity, RssObservation, STORAGE_MODEL_SHA256,
    U32_IDENTITY_DOMAIN_COUNT, U32IdentityCounts, U32IdentityDomain,
};
