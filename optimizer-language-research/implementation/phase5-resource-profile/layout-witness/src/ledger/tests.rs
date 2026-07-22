use super::*;

const MAGIC: [u8; 16] = *b"WFRP1PEAKLEDGER\0";

fn sample_ledger() -> Result<PeakLedger, LedgerError> {
    let labels = EvidenceLabels::try_new([
        "host-class",
        "observed-host",
        "toolchain",
        "allocator",
        "supervisor",
    ])?;
    let mut rows = CANONICAL_ROW_ORDER.map(PeakRow::zeroed);
    rows[0].record_capacities[RecordFamily::SourceFile.ordinal()] = 3;
    rows[0].exact_charges[ExactChargeKind::SourceBytes.ordinal()] = 17;
    rows[11].record_capacities[RecordFamily::LookupEntry.ordinal()] = 2;
    rows[11].record_capacities[RecordFamily::OrderingScratch.ordinal()] = 2;
    Ok(PeakLedger {
        identity: EvidenceIdentity {
            digests: EvidenceDigests {
                proposal: APPROVED_PROPOSAL_SHA256,
                candidate_specification: APPROVED_CANDIDATE_SPECIFICATION_SHA256,
                storage_model: STORAGE_MODEL_SHA256,
                witness_executable: [4; 32],
            },
            labels,
            run_ordinal: 7,
        },
        process_baseline_bytes: 11,
        u32_identity_counts: U32IdentityCounts { values: [5; 8] },
        rss: RssObservation::Measured {
            high_water_bytes: 19,
        },
        rows,
    })
}

fn first_row_offset(bytes: &[u8]) -> Result<usize, LedgerError> {
    let mut position = MAGIC.len() + 2 + 128;
    for _ in 0..5 {
        let length_bytes: [u8; 2] = bytes
            .get(position..position + 2)
            .ok_or(LedgerError::Truncated)?
            .try_into()
            .map_err(|_| LedgerError::Truncated)?;
        position = position
            .checked_add(2 + usize::from(u16::from_le_bytes(length_bytes)))
            .ok_or(LedgerError::Truncated)?;
    }
    position
        .checked_add(8 + 8 + U32_IDENTITY_DOMAIN_COUNT * 8 + 1 + 8 + 1)
        .ok_or(LedgerError::Truncated)
}

#[test]
fn canonical_bytes_are_exact_and_round_trip() -> Result<(), LedgerError> {
    let ledger = sample_ledger()?;
    let first = ledger.try_encode()?;
    let second = ledger.try_encode()?;
    assert_eq!(first, second);
    assert_eq!(first.len(), 4_819);
    assert_eq!(first.get(..MAGIC.len()), Some(MAGIC.as_slice()));
    assert_eq!(PeakLedger::decode(&first)?, ledger);
    Ok(())
}

#[test]
fn requested_accounted_and_rss_values_remain_distinct() -> Result<(), LedgerError> {
    let mut ledger = sample_ledger()?;
    ledger.rows[0].exact_charges[ExactChargeKind::VectorHeaders.ordinal()] = 23;
    let totals = ledger.totals(0)?;
    assert_eq!(totals.requested_bytes, 3 * 64 + 17);
    assert_eq!(totals.accounted_bytes, 3 * 64 + 17 + 23);
    assert_eq!(totals.modeled_process_bytes, 11 + 3 * 64 + 17 + 23);
    assert_eq!(
        ledger.rss,
        RssObservation::Measured {
            high_water_bytes: 19
        }
    );
    Ok(())
}

#[test]
fn row_order_mutation_fails_closed() -> Result<(), LedgerError> {
    let ledger = sample_ledger()?;
    let mut bytes = ledger.try_encode()?;
    let first_row = first_row_offset(&bytes)?;
    if let Some(value) = bytes.get_mut(first_row) {
        *value = PeakKind::Parser as u8;
    } else {
        return Err(LedgerError::Truncated);
    }
    assert_eq!(
        PeakLedger::decode(&bytes),
        Err(LedgerError::InvalidRowOrder)
    );
    Ok(())
}

#[test]
fn truncation_and_trailing_mutations_fail_closed() -> Result<(), LedgerError> {
    let ledger = sample_ledger()?;
    let mut bytes = ledger.try_encode()?;
    let truncated = bytes.get(..bytes.len() - 1).ok_or(LedgerError::Truncated)?;
    assert_eq!(PeakLedger::decode(truncated), Err(LedgerError::Truncated));
    bytes.push(0);
    assert_eq!(PeakLedger::decode(&bytes), Err(LedgerError::TrailingBytes));
    Ok(())
}

#[test]
fn rss_tag_mutation_fails_closed() -> Result<(), LedgerError> {
    let ledger = sample_ledger()?;
    let mut bytes = ledger.try_encode()?;
    let first_row = first_row_offset(&bytes)?;
    let rss_tag = first_row.checked_sub(10).ok_or(LedgerError::Truncated)?;
    if let Some(value) = bytes.get_mut(rss_tag) {
        *value = 7;
    } else {
        return Err(LedgerError::Truncated);
    }
    assert_eq!(
        PeakLedger::decode(&bytes),
        Err(LedgerError::InvalidRssObservation)
    );
    Ok(())
}

#[test]
fn digest_and_non_graphic_identity_mutations_are_rejected() -> Result<(), LedgerError> {
    let mut ledger = sample_ledger()?;
    ledger.identity.digests.proposal[0] ^= 1;
    assert_eq!(ledger.validate(), Err(LedgerError::WrongProposalDigest));
    ledger = sample_ledger()?;
    ledger.identity.digests.candidate_specification[0] ^= 1;
    assert_eq!(
        ledger.validate(),
        Err(LedgerError::WrongCandidateSpecificationDigest)
    );
    ledger = sample_ledger()?;
    ledger.identity.digests.storage_model[0] ^= 1;
    assert_eq!(ledger.validate(), Err(LedgerError::WrongStorageModelDigest));
    ledger = sample_ledger()?;
    ledger.identity.digests.witness_executable = [0; 32];
    assert_eq!(
        ledger.validate(),
        Err(LedgerError::MissingWitnessExecutableDigest)
    );
    assert_eq!(
        EvidenceLabels::try_new(["host class", "host", "tool", "alloc", "super"]),
        Err(LedgerError::InvalidIdentity)
    );
    Ok(())
}

#[test]
fn encoded_repository_identity_mutations_fail_closed() -> Result<(), LedgerError> {
    let canonical = sample_ledger()?.try_encode()?;
    for (offset, expected) in [
        (MAGIC.len() + 2, LedgerError::WrongProposalDigest),
        (
            MAGIC.len() + 2 + 32,
            LedgerError::WrongCandidateSpecificationDigest,
        ),
        (MAGIC.len() + 2 + 64, LedgerError::WrongStorageModelDigest),
    ] {
        let mut mutated = canonical.clone();
        if let Some(byte) = mutated.get_mut(offset) {
            *byte ^= 1;
        } else {
            return Err(LedgerError::Truncated);
        }
        assert_eq!(PeakLedger::decode(&mutated), Err(expected));
    }
    Ok(())
}

#[test]
fn every_u32_identity_domain_accepts_exact_maximum_and_rejects_one_over() -> Result<(), LedgerError>
{
    let mut boundary = sample_ledger()?;
    boundary.u32_identity_counts.values = [u64::from(u32::MAX); U32_IDENTITY_DOMAIN_COUNT];
    assert_eq!(boundary.validate(), Ok(()));

    for domain in U32IdentityDomain::ALL {
        let mut one_over = sample_ledger()?;
        one_over.u32_identity_counts.values[domain.ordinal()] = u64::from(u32::MAX) + 1;
        assert_eq!(
            one_over.validate(),
            Err(LedgerError::U32IdentityExceeded),
            "{domain:?}"
        );
    }
    Ok(())
}
