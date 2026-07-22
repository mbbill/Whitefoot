#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeSet;

use super::*;
use crate::{KERNEL_SPEC_V0_8_HASH, SourceInput};

const MAXIMUM: SourceLimits = SourceLimits::REPRESENTABLE;

fn bundle(inputs: &[SourceInput<'_>]) -> SourceBundle {
    SourceBundle::with_limits(inputs, MAXIMUM).unwrap()
}

fn sample_bundle() -> SourceBundle {
    bundle(&[
        SourceInput::new("z.wf", &[0xff, 0x00]),
        SourceInput::new("a.wf", b"fn bytes"),
    ])
}

fn candidate(spec_hash: SpecHash, records: &[SourceInput<'_>]) -> SourceBinding {
    SourceBinding::try_from_sources(spec_hash, records, MAXIMUM).unwrap()
}

fn binding_for_bundle(bundle: &SourceBundle) -> SourceBinding {
    SourceBinding::try_from_bundle(KERNEL_SPEC_V0_8_HASH, bundle, MAXIMUM).unwrap()
}

fn encode(binding: &SourceBinding) -> Vec<u8> {
    binding.encode_canonical(MAXIMUM).unwrap()
}

fn header(source_count: u64) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(b"WFSOURCE");
    encoded.extend_from_slice(&SOURCE_BINDING_CODEC_VERSION.to_be_bytes());
    encoded.extend_from_slice(KERNEL_SPEC_V0_8_HASH.digest().as_bytes());
    encoded.extend_from_slice(&source_count.to_be_bytes());
    encoded
}

fn push_record(encoded: &mut Vec<u8>, path: &[u8], bytes: &[u8]) {
    encoded.extend_from_slice(&(path.len() as u64).to_be_bytes());
    encoded.extend_from_slice(path);
    encoded.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    encoded.extend_from_slice(bytes);
}

#[test]
fn canonical_round_trip_preserves_order_paths_and_raw_bytes() {
    let binding = binding_for_bundle(&sample_bundle());
    let encoded = encode(&binding);
    let decoded = SourceBinding::decode_canonical(&encoded, MAXIMUM).unwrap();
    assert_eq!(decoded, binding);
    assert_eq!(encode(&decoded), encoded);
}

#[test]
fn canonical_encoding_has_one_pinned_field_order() {
    let bundle = bundle(&[SourceInput::new("a", b"bc")]);
    let encoded = encode(&binding_for_bundle(&bundle));
    let mut expected = header(1);
    push_record(&mut expected, b"a", b"bc");
    assert_eq!(encoded, expected);
}

#[test]
fn zero_source_binding_round_trips() {
    let binding = candidate(KERNEL_SPEC_V0_8_HASH, &[]);
    let encoded = encode(&binding);
    assert_eq!(
        SourceBinding::decode_canonical(&encoded, MAXIMUM),
        Ok(binding)
    );
}

#[test]
fn source_binding_v1_empty_vector_remains_source_and_spec_only() {
    assert_eq!(SOURCE_BINDING_CODEC_VERSION, 1);
    let encoded = encode(&candidate(KERNEL_SPEC_V0_8_HASH, &[]));
    let mut expected = b"WFSOURCE".to_vec();
    expected.extend_from_slice(&1_u16.to_be_bytes());
    expected.extend_from_slice(KERNEL_SPEC_V0_8_HASH.digest().as_bytes());
    expected.extend_from_slice(&0_u64.to_be_bytes());
    assert_eq!(encoded.len(), 50);
    assert_eq!(encoded, expected);
}

#[test]
fn framing_distinguishes_concatenations_splits_and_order() {
    let candidates = [
        bundle(&[SourceInput::new("a", b"bc")]),
        bundle(&[SourceInput::new("ab", b"c")]),
        bundle(&[SourceInput::new("a", b"b"), SourceInput::new("c", b"c")]),
        bundle(&[SourceInput::new("c", b"c"), SourceInput::new("a", b"b")]),
    ];
    let encodings: BTreeSet<_> = candidates
        .iter()
        .map(|bundle| encode(&binding_for_bundle(bundle)))
        .collect();
    assert_eq!(encodings.len(), candidates.len());
}

#[test]
fn every_truncated_valid_prefix_is_rejected_without_panicking() {
    let encoded = encode(&binding_for_bundle(&sample_bundle()));
    for end in 0..encoded.len() {
        let result = SourceBinding::decode_canonical(&encoded[..end], MAXIMUM);
        assert!(result.is_err(), "accepted prefix ending at {end}");
        if let Err(DecodeError::Truncated { offset }) = result {
            assert_eq!(offset, end);
        }
    }
}

#[test]
fn malformed_headers_and_trailing_bytes_are_rejected() {
    let mut bad_magic = header(0);
    bad_magic[0] ^= 1;
    assert!(matches!(
        SourceBinding::decode_canonical(&bad_magic, MAXIMUM),
        Err(DecodeError::BadMagic)
    ));

    let mut wrong_version = header(0);
    wrong_version[MAGIC.len()..MAGIC.len() + 2].copy_from_slice(&2_u16.to_be_bytes());
    assert!(matches!(
        SourceBinding::decode_canonical(&wrong_version, MAXIMUM),
        Err(DecodeError::UnsupportedVersion(2))
    ));

    let mut trailing = header(0);
    trailing.push(0);
    assert!(matches!(
        SourceBinding::decode_canonical(&trailing, MAXIMUM),
        Err(DecodeError::TrailingBytes(1))
    ));
}

#[test]
fn truncated_offset_is_the_first_unavailable_byte() {
    assert_eq!(
        SourceBinding::decode_canonical(b"WFSOU", MAXIMUM),
        Err(DecodeError::Truncated { offset: 5 })
    );
}

#[test]
fn maximum_and_impossible_framed_lengths_are_rejected() {
    assert!(matches!(
        SourceBinding::decode_canonical(&header(u64::MAX), MAXIMUM),
        Err(DecodeError::LimitExceeded {
            limit: SourceLimit::Sources,
            ..
        })
    ));
    assert_eq!(
        SourceBinding::decode_canonical(&header(1), MAXIMUM),
        Err(DecodeError::ImpossibleSourceCount(1))
    );

    let mut path_length = header(1);
    path_length.extend_from_slice(&u64::MAX.to_be_bytes());
    path_length.extend_from_slice(&[0; 9]);
    assert!(matches!(
        SourceBinding::decode_canonical(&path_length, MAXIMUM),
        Err(DecodeError::Truncated { .. })
    ));

    let mut source_length = header(1);
    source_length.extend_from_slice(&1_u64.to_be_bytes());
    source_length.push(b'a');
    source_length.extend_from_slice(&u64::MAX.to_be_bytes());
    assert!(matches!(
        SourceBinding::decode_canonical(&source_length, MAXIMUM),
        Err(DecodeError::Truncated { .. })
    ));
}

#[test]
fn invalid_path_transport_and_spelling_are_distinct() {
    let mut invalid_utf8 = header(1);
    push_record(&mut invalid_utf8, &[0xff], b"");
    assert_eq!(
        SourceBinding::decode_canonical(&invalid_utf8, MAXIMUM),
        Err(DecodeError::LogicalPathNotUtf8)
    );

    let mut invalid_path = header(1);
    push_record(&mut invalid_path, b"a\\b", b"");
    assert!(matches!(
        SourceBinding::decode_canonical(&invalid_path, MAXIMUM),
        Err(DecodeError::LogicalPath(_))
    ));
}

#[test]
fn decoder_enforces_every_resource_limit() {
    let encoded = encode(&binding_for_bundle(&bundle(&[
        SourceInput::new("aa", b"12"),
        SourceInput::new("bb", b"34"),
    ])));
    let cases = [
        (
            SourceLimits {
                max_sources: 1,
                max_logical_path_bytes: 8,
                max_source_bytes: 8,
                max_total_source_bytes: 8,
                max_binding_bytes: 1024,
            },
            SourceLimit::Sources,
        ),
        (
            SourceLimits {
                max_sources: 2,
                max_logical_path_bytes: 1,
                max_source_bytes: 8,
                max_total_source_bytes: 8,
                max_binding_bytes: 1024,
            },
            SourceLimit::LogicalPathBytes,
        ),
        (
            SourceLimits {
                max_sources: 2,
                max_logical_path_bytes: 8,
                max_source_bytes: 1,
                max_total_source_bytes: 8,
                max_binding_bytes: 1024,
            },
            SourceLimit::SourceBytes,
        ),
        (
            SourceLimits {
                max_sources: 2,
                max_logical_path_bytes: 8,
                max_source_bytes: 8,
                max_total_source_bytes: 3,
                max_binding_bytes: 1024,
            },
            SourceLimit::TotalSourceBytes,
        ),
        (
            SourceLimits {
                max_sources: 2,
                max_logical_path_bytes: 8,
                max_source_bytes: 8,
                max_total_source_bytes: 8,
                max_binding_bytes: encoded.len() as u64 - 1,
            },
            SourceLimit::BindingBytes,
        ),
    ];
    for (limits, expected_limit) in cases {
        assert!(matches!(
            SourceBinding::decode_canonical(&encoded, limits),
            Err(DecodeError::LimitExceeded { limit, .. }) if limit == expected_limit
        ));
    }
}

#[test]
fn encoder_enforces_aggregate_and_output_limits_before_allocation() {
    let binding = binding_for_bundle(&sample_bundle());
    assert!(matches!(
        binding.encode_canonical(SourceLimits {
            max_sources: 2,
            max_logical_path_bytes: 64,
            max_source_bytes: 64,
            max_total_source_bytes: 1,
            max_binding_bytes: 1024,
        }),
        Err(EncodeError::LimitExceeded {
            limit: SourceLimit::TotalSourceBytes,
            ..
        })
    ));
    assert!(matches!(
        binding.encode_canonical(SourceLimits {
            max_sources: 2,
            max_logical_path_bytes: 64,
            max_source_bytes: 64,
            max_total_source_bytes: 64,
            max_binding_bytes: 1,
        }),
        Err(EncodeError::LimitExceeded {
            limit: SourceLimit::BindingBytes,
            ..
        })
    ));
}

#[test]
fn candidate_construction_checks_limits_before_owned_copy() {
    let limits = SourceLimits {
        max_sources: 1,
        max_logical_path_bytes: 1,
        max_source_bytes: 1,
        max_total_source_bytes: 1,
        max_binding_bytes: 128,
    };
    assert!(matches!(
        SourceBinding::try_from_sources(
            KERNEL_SPEC_V0_8_HASH,
            &[SourceInput::new("long", b"")],
            limits,
        ),
        Err(EncodeError::LimitExceeded {
            limit: SourceLimit::LogicalPathBytes,
            maximum: 1,
            actual: 4,
        })
    ));
    assert!(matches!(
        SourceBinding::try_from_sources(
            KERNEL_SPEC_V0_8_HASH,
            &[SourceInput::new("a\\b", b"")],
            MAXIMUM,
        ),
        Err(EncodeError::LogicalPath(
            LogicalPathError::InvalidByte { .. }
        ))
    ));
}

#[test]
fn impossible_binding_reservations_are_explicit_failures() {
    let mut encode_values = Vec::<u8>::new();
    assert_eq!(
        try_reserve_encode(
            &mut encode_values,
            usize::MAX,
            SourceLimit::BindingBytes,
            u64::MAX,
        ),
        Err(EncodeError::StorageUnavailable {
            limit: SourceLimit::BindingBytes,
            requested: u64::MAX,
        })
    );

    let mut decode_values = Vec::<u8>::new();
    assert_eq!(
        try_reserve_decode(
            &mut decode_values,
            usize::MAX,
            SourceLimit::SourceBytes,
            u64::MAX,
        ),
        Err(DecodeError::StorageUnavailable {
            limit: SourceLimit::SourceBytes,
            requested: u64::MAX,
        })
    );
}

#[test]
fn inserted_and_deleted_source_bytes_do_not_decode_as_the_same_binding() {
    let encoded = encode(&binding_for_bundle(&sample_bundle()));
    let mut inserted = encoded.clone();
    inserted.push(0);
    assert!(matches!(
        SourceBinding::decode_canonical(&inserted, MAXIMUM),
        Err(DecodeError::TrailingBytes(1))
    ));
    assert!(matches!(
        SourceBinding::decode_canonical(&encoded[..encoded.len() - 1], MAXIMUM),
        Err(DecodeError::Truncated { .. })
    ));
}

#[test]
fn duplicate_paths_remain_unverified_candidate_data() {
    let mut encoded = header(2);
    push_record(&mut encoded, b"same", b"first");
    push_record(&mut encoded, b"same", b"second");
    let decoded = SourceBinding::decode_canonical(&encoded, MAXIMUM).unwrap();
    assert_eq!(decoded.sources().len(), 2);
    assert_eq!(
        decoded.sources()[0].logical_path(),
        decoded.sources()[1].logical_path()
    );
}

#[test]
fn deterministic_arbitrary_bytes_never_panic() {
    let mut state = 0x6a09_e667_f3bc_c909_u64;
    for length in 0..256 {
        let mut bytes = Vec::with_capacity(length);
        for _ in 0..length {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            bytes.push((state >> 56) as u8);
        }
        let _ = SourceBinding::decode_canonical(&bytes, MAXIMUM);
    }
}

#[test]
fn small_domain_encodings_are_injective_and_canonical() {
    let byte_values: &[&[u8]] = &[b"", &[0], &[1], &[0, 1]];
    let mut bindings = vec![candidate(KERNEL_SPEC_V0_8_HASH, &[])];
    for path in ["a", "b"] {
        for bytes in byte_values {
            bindings.push(candidate(
                KERNEL_SPEC_V0_8_HASH,
                &[SourceInput::new(path, bytes)],
            ));
        }
    }
    for first in byte_values {
        for second in byte_values {
            bindings.push(candidate(
                KERNEL_SPEC_V0_8_HASH,
                &[SourceInput::new("a", first), SourceInput::new("b", second)],
            ));
        }
    }

    let mut observed = BTreeSet::new();
    for binding in bindings {
        let encoded = encode(&binding);
        assert!(observed.insert(encoded.clone()));
        let decoded = SourceBinding::decode_canonical(&encoded, MAXIMUM).unwrap();
        assert_eq!(decoded, binding);
        assert_eq!(encode(&decoded), encoded);
    }
}
