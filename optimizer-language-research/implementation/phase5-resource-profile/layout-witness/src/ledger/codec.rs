//! Exact binary codec for one validated peak ledger.

use crate::charges::RECORD_FAMILY_COUNT;

use super::{
    CANONICAL_ROW_ORDER, EXACT_CHARGE_COUNT, EvidenceDigests, EvidenceIdentity, EvidenceLabels,
    LEDGER_ROW_COUNT, LedgerError, PeakKind, PeakLedger, PeakRow, RssObservation,
    U32_IDENTITY_DOMAIN_COUNT, U32IdentityCounts, copy_identity, row, validate_identity,
};

const MAGIC: [u8; 16] = *b"WFRP1PEAKLEDGER\0";
const VERSION: u16 = 1;
const MAX_ENCODED_LENGTH: usize = 8_192;

impl PeakLedger {
    /// Encodes the exact canonical binary representation.
    pub fn try_encode(&self) -> Result<Vec<u8>, LedgerError> {
        self.validate()?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(MAX_ENCODED_LENGTH)
            .map_err(|_| LedgerError::StorageUnavailable)?;
        bytes.extend_from_slice(&MAGIC);
        put_u16(&mut bytes, VERSION);
        bytes.extend_from_slice(&self.identity.digests.proposal);
        bytes.extend_from_slice(&self.identity.digests.candidate_specification);
        bytes.extend_from_slice(&self.identity.digests.storage_model);
        bytes.extend_from_slice(&self.identity.digests.witness_executable);
        for value in self.identity.labels.values() {
            put_identity(&mut bytes, value)?;
        }
        put_u64(&mut bytes, self.identity.run_ordinal);
        put_u64(&mut bytes, self.process_baseline_bytes);
        for value in self.u32_identity_counts.values {
            put_u64(&mut bytes, value);
        }
        match self.rss {
            RssObservation::Unmeasured => {
                bytes.push(0);
                put_u64(&mut bytes, 0);
            }
            RssObservation::Measured { high_water_bytes } => {
                bytes.push(1);
                put_u64(&mut bytes, high_water_bytes);
            }
        }
        bytes.push(LEDGER_ROW_COUNT as u8);
        for row in &self.rows {
            bytes.push(row.identity.peak as u8);
            bytes.push(row.identity.variant);
            for count in row.record_capacities {
                put_u64(&mut bytes, count);
            }
            for charge in row.exact_charges {
                put_u64(&mut bytes, charge);
            }
        }
        if bytes.len() > MAX_ENCODED_LENGTH {
            return Err(LedgerError::EncodedLengthExceeded);
        }
        Ok(bytes)
    }

    /// Decodes only the exact canonical binary representation.
    pub fn decode(bytes: &[u8]) -> Result<Self, LedgerError> {
        if bytes.len() > MAX_ENCODED_LENGTH {
            return Err(LedgerError::EncodedLengthExceeded);
        }
        let mut cursor = Cursor::new(bytes);
        if cursor.take(MAGIC.len())? != MAGIC {
            return Err(LedgerError::InvalidMagic);
        }
        if cursor.u16()? != VERSION {
            return Err(LedgerError::InvalidVersion);
        }
        let digests = EvidenceDigests {
            proposal: cursor.array32()?,
            candidate_specification: cursor.array32()?,
            storage_model: cursor.array32()?,
            witness_executable: cursor.array32()?,
        };
        let labels = EvidenceLabels {
            host_class: cursor.identity()?,
            observed_host: cursor.identity()?,
            toolchain: cursor.identity()?,
            allocator: cursor.identity()?,
            supervisor: cursor.identity()?,
        };
        let run_ordinal = cursor.u64()?;
        let process_baseline_bytes = cursor.u64()?;
        let mut u32_identity_values = [0_u64; U32_IDENTITY_DOMAIN_COUNT];
        for value in &mut u32_identity_values {
            *value = cursor.u64()?;
        }
        let rss = decode_rss(&mut cursor)?;
        if usize::from(cursor.u8()?) != LEDGER_ROW_COUNT {
            return Err(LedgerError::InvalidRowCount);
        }

        let mut decoded_rows = Vec::new();
        decoded_rows
            .try_reserve_exact(LEDGER_ROW_COUNT)
            .map_err(|_| LedgerError::StorageUnavailable)?;
        for expected in CANONICAL_ROW_ORDER {
            let peak = decode_peak(cursor.u8()?)?;
            let variant = cursor.u8()?;
            if row(peak, variant) != expected {
                return Err(LedgerError::InvalidRowOrder);
            }
            let mut record_capacities = [0_u64; RECORD_FAMILY_COUNT];
            for value in &mut record_capacities {
                *value = cursor.u64()?;
            }
            let mut exact_charges = [0_u64; EXACT_CHARGE_COUNT];
            for value in &mut exact_charges {
                *value = cursor.u64()?;
            }
            decoded_rows.push(PeakRow {
                identity: expected,
                record_capacities,
                exact_charges,
            });
        }
        if !cursor.is_complete() {
            return Err(LedgerError::TrailingBytes);
        }
        let rows: [PeakRow; LEDGER_ROW_COUNT] = decoded_rows
            .try_into()
            .map_err(|_| LedgerError::InvalidRowCount)?;
        let ledger = Self {
            identity: EvidenceIdentity {
                digests,
                labels,
                run_ordinal,
            },
            process_baseline_bytes,
            u32_identity_counts: U32IdentityCounts {
                values: u32_identity_values,
            },
            rss,
            rows,
        };
        ledger.validate()?;
        if ledger.try_encode()? != bytes {
            return Err(LedgerError::NonCanonicalRepresentation);
        }
        Ok(ledger)
    }
}

fn decode_rss(cursor: &mut Cursor<'_>) -> Result<RssObservation, LedgerError> {
    match (cursor.u8()?, cursor.u64()?) {
        (0, 0) => Ok(RssObservation::Unmeasured),
        (1, value) if value != 0 => Ok(RssObservation::Measured {
            high_water_bytes: value,
        }),
        _ => Err(LedgerError::InvalidRssObservation),
    }
}

fn put_identity(bytes: &mut Vec<u8>, value: &str) -> Result<(), LedgerError> {
    validate_identity(value)?;
    let length = u16::try_from(value.len()).map_err(|_| LedgerError::InvalidIdentity)?;
    put_u16(bytes, length);
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn put_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn decode_peak(value: u8) -> Result<PeakKind, LedgerError> {
    match value {
        0 => Ok(PeakKind::SourceBundleConstruction),
        1 => Ok(PeakKind::SourceBindingAndCodec),
        2 => Ok(PeakKind::LexerCountPass),
        3 => Ok(PeakKind::LexerEmission),
        4 => Ok(PeakKind::Classifier),
        5 => Ok(PeakKind::Parser),
        6 => Ok(PeakKind::Finalizer),
        7 => Ok(PeakKind::CanonicalAudit),
        8 => Ok(PeakKind::ResolutionPreflight),
        9 => Ok(PeakKind::ResolutionSort),
        10 => Ok(PeakKind::RetainedResolution),
        11 => Ok(PeakKind::DiagnosticMaterialization),
        _ => Err(LedgerError::InvalidPeak),
    }
}

struct Cursor<'input> {
    bytes: &'input [u8],
    position: usize,
}

impl<'input> Cursor<'input> {
    const fn new(bytes: &'input [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'input [u8], LedgerError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(LedgerError::Truncated)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(LedgerError::Truncated)?;
        self.position = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, LedgerError> {
        self.take(1)?.first().copied().ok_or(LedgerError::Truncated)
    }

    fn u16(&mut self) -> Result<u16, LedgerError> {
        let bytes: [u8; 2] = self
            .take(2)?
            .try_into()
            .map_err(|_| LedgerError::Truncated)?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn u64(&mut self) -> Result<u64, LedgerError> {
        let bytes: [u8; 8] = self
            .take(8)?
            .try_into()
            .map_err(|_| LedgerError::Truncated)?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn array32(&mut self) -> Result<[u8; 32], LedgerError> {
        self.take(32)?
            .try_into()
            .map_err(|_| LedgerError::Truncated)
    }

    fn identity(&mut self) -> Result<String, LedgerError> {
        let length = usize::from(self.u16()?);
        let bytes = self.take(length)?;
        let value = std::str::from_utf8(bytes).map_err(|_| LedgerError::InvalidIdentity)?;
        copy_identity(value)
    }

    fn is_complete(&self) -> bool {
        self.position == self.bytes.len()
    }
}
