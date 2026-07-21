use std::io::Cursor;

use whitefoot_contract::{
    ByteOffset, DecodeError, EncodeError, SourceBundleError, SourceId, SourceLimit, SourceLimits,
};
use whitefoot_lexer::{LexCompilerFailure, LexLimit, LexOutcome, LexResourceFailure, LexStorage};

use crate::ACTIVE_KERNEL_SPEC_HASH;
use crate::protocol::{AdapterError, RESPONSE_MAGIC, RESPONSE_VERSION, read_request};

const SOURCE_LIMITS: SourceLimits = SourceLimits {
    max_sources: 8,
    max_logical_path_bytes: 64,
    max_source_bytes: 1_024,
    max_total_source_bytes: 2_048,
    max_binding_bytes: 4_096,
};

fn empty_binding() -> Vec<u8> {
    let mut binding = Vec::new();
    binding.extend_from_slice(b"WFSOURCE");
    binding.extend_from_slice(&1_u16.to_be_bytes());
    binding.extend_from_slice(ACTIVE_KERNEL_SPEC_HASH.digest().as_bytes());
    binding.extend_from_slice(&0_u64.to_be_bytes());
    binding
}

fn request(binding: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"WFLEXREQ");
    bytes.extend_from_slice(&1_u16.to_be_bytes());
    bytes.extend_from_slice(&SOURCE_LIMITS.max_sources.to_be_bytes());
    bytes.extend_from_slice(&SOURCE_LIMITS.max_logical_path_bytes.to_be_bytes());
    bytes.extend_from_slice(&SOURCE_LIMITS.max_source_bytes.to_be_bytes());
    bytes.extend_from_slice(&SOURCE_LIMITS.max_total_source_bytes.to_be_bytes());
    bytes.extend_from_slice(&SOURCE_LIMITS.max_binding_bytes.to_be_bytes());
    bytes.extend_from_slice(&8_u32.to_be_bytes());
    for limit in [1_024_u64, 2_048, 1_024, 1_024, 2_048] {
        bytes.extend_from_slice(&limit.to_be_bytes());
    }
    bytes.extend_from_slice(&u64::try_from(binding.len()).unwrap().to_be_bytes());
    bytes.extend_from_slice(binding);
    bytes
}

#[test]
fn exact_empty_request_decodes_without_ambient_inputs() {
    let binding = empty_binding();
    let encoded = request(&binding);
    assert_eq!(encoded.len(), 98 + binding.len());
    assert_eq!(&encoded[..8], b"WFLEXREQ");
    let decoded = read_request(&mut Cursor::new(encoded)).unwrap();
    assert_eq!(decoded.source_limits, SOURCE_LIMITS);
    assert_eq!(decoded.lex_limits.max_sources, 8);
    assert_eq!(decoded.lex_limits.max_lexemes, 2_048);
    assert_eq!(decoded.binding, binding);
}

#[test]
fn truncation_and_trailing_bytes_are_tool_failures() {
    let encoded = request(&empty_binding());
    for end in [0, 9, 97, encoded.len() - 1] {
        assert_eq!(
            read_request(&mut Cursor::new(&encoded[..end])).err(),
            Some(AdapterError::RequestRead)
        );
    }
    let mut trailing = encoded;
    trailing.push(0);
    assert_eq!(
        read_request(&mut Cursor::new(trailing)).err(),
        Some(AdapterError::TrailingRequestBytes)
    );
}

#[test]
fn owned_source_storage_failures_remain_distinct_from_invalid_input() {
    let binding_failure = DecodeError::StorageUnavailable {
        limit: SourceLimit::SourceBytes,
        requested: 7,
    };
    assert_eq!(
        super::map_binding_decode_error(binding_failure),
        AdapterError::SourceBindingStorageUnavailable
    );
    let bundle_failure = SourceBundleError::StorageUnavailable {
        limit: SourceLimit::Sources,
        requested: 3,
    };
    assert_eq!(
        super::map_source_bundle_error(bundle_failure),
        AdapterError::SourceBundleStorageUnavailable
    );
    let encode_failure = EncodeError::StorageUnavailable {
        limit: SourceLimit::BindingBytes,
        requested: 11,
    };
    assert_eq!(
        super::map_binding_encode_error(encode_failure),
        AdapterError::SourceBindingStorageUnavailable
    );
}

#[test]
fn magic_version_and_hard_profile_fail_before_payload_allocation() {
    let encoded = request(&empty_binding());
    let mut magic = encoded.clone();
    magic[0] ^= 1;
    assert_eq!(
        read_request(&mut Cursor::new(magic)).err(),
        Some(AdapterError::RequestMagic)
    );
    let mut version = encoded.clone();
    version[9] = 2;
    assert_eq!(
        read_request(&mut Cursor::new(version)).err(),
        Some(AdapterError::RequestVersion)
    );
    let mut source_count = encoded;
    source_count[10..14].copy_from_slice(&4_097_u32.to_be_bytes());
    assert_eq!(
        read_request(&mut Cursor::new(source_count)).err(),
        Some(AdapterError::SourceLimitsOutsideProfile)
    );

    for (offset, value) in [
        (14, 4_097_u64),
        (22, 1_048_577),
        (30, 1_048_577),
        (38, 2_097_153),
    ] {
        let mut outside = request(&empty_binding());
        outside[offset..offset + 8].copy_from_slice(&value.to_be_bytes());
        assert_eq!(
            read_request(&mut Cursor::new(outside)).err(),
            Some(AdapterError::SourceLimitsOutsideProfile)
        );
    }

    let mut binding_length = request(&empty_binding());
    binding_length[90..98].copy_from_slice(&4_097_u64.to_be_bytes());
    assert_eq!(
        read_request(&mut Cursor::new(binding_length)).err(),
        Some(AdapterError::BindingLengthOutsideProfile)
    );
}

#[test]
fn empty_complete_response_has_one_exact_neutral_shape() {
    let bundle = whitefoot_contract::SourceBundle::with_limits(&[], SOURCE_LIMITS).unwrap();
    let outcome = whitefoot_lexer::lex_v0_9(
        &bundle,
        whitefoot_lexer::LexLimits {
            max_sources: 8,
            max_source_bytes: 1_024,
            max_total_source_bytes: 2_048,
            max_token_bytes: 1_024,
            max_tokens: 1_024,
            max_lexemes: 2_048,
        },
    );
    let actual = crate::projection::encode_observation(&bundle, outcome).unwrap();
    let mut expected = Vec::new();
    expected.extend_from_slice(&RESPONSE_MAGIC);
    expected.extend_from_slice(&RESPONSE_VERSION.to_be_bytes());
    expected.extend_from_slice(ACTIVE_KERNEL_SPEC_HASH.digest().as_bytes());
    expected.push(0);
    expected.extend_from_slice(&0_u64.to_be_bytes());
    expected.extend_from_slice(&0_u32.to_be_bytes());
    assert_eq!(actual, expected);
}

fn empty_bundle() -> whitefoot_contract::SourceBundle {
    whitefoot_contract::SourceBundle::with_limits(&[], SOURCE_LIMITS).unwrap()
}

fn payload(outcome: LexOutcome<'_>, bundle: &whitefoot_contract::SourceBundle) -> Vec<u8> {
    let response = crate::projection::encode_observation(bundle, outcome).unwrap();
    assert_eq!(&response[..8], b"WFLEXRSP");
    assert_eq!(
        &response[10..42],
        ACTIVE_KERNEL_SPEC_HASH.digest().as_bytes()
    );
    response[42..].to_vec()
}

#[test]
fn every_resource_failure_has_an_explicit_stable_tag() {
    let bundle = empty_bundle();
    for (limit, tag) in [
        (LexLimit::Sources, 0),
        (LexLimit::SourceBytes, 1),
        (LexLimit::TotalSourceBytes, 2),
        (LexLimit::TokenBytes, 3),
        (LexLimit::Tokens, 4),
        (LexLimit::Lexemes, 5),
    ] {
        let outcome = LexOutcome::ResourceFailure(LexResourceFailure::LimitExceeded {
            limit,
            maximum: 7,
            actual: 8,
        });
        let mut expected = vec![2, 0, tag];
        expected.extend_from_slice(&7_u64.to_be_bytes());
        expected.extend_from_slice(&8_u64.to_be_bytes());
        assert_eq!(payload(outcome, &bundle), expected);
    }

    for (failure, subtype, storage) in [
        (
            LexResourceFailure::AddressSpaceExceeded {
                storage: LexStorage::Lexemes,
                requested: 9,
            },
            1,
            0,
        ),
        (
            LexResourceFailure::StorageUnavailable {
                storage: LexStorage::SourceBoundaries,
                requested: 9,
            },
            2,
            1,
        ),
    ] {
        let mut expected = vec![2, subtype, storage];
        expected.extend_from_slice(&9_u64.to_be_bytes());
        assert_eq!(
            payload(LexOutcome::ResourceFailure(failure), &bundle),
            expected
        );
    }
}

#[test]
fn every_compiler_failure_has_an_explicit_stable_tag() {
    let bundle = empty_bundle();
    let source = SourceId::from_ordinal(6);
    let start = ByteOffset::new(9);
    let end = ByteOffset::new(3);
    let invalid = payload(
        LexOutcome::CompilerFailure(LexCompilerFailure::InvalidProducedSpan { source, start, end }),
        &bundle,
    );
    let mut expected = vec![3, 0];
    expected.extend_from_slice(&6_u32.to_be_bytes());
    expected.extend_from_slice(&9_u64.to_be_bytes());
    expected.extend_from_slice(&3_u64.to_be_bytes());
    assert_eq!(invalid, expected);

    let disagreement = payload(
        LexOutcome::CompilerFailure(LexCompilerFailure::PassDisagreement { source }),
        &bundle,
    );
    assert_eq!(disagreement, [3, 1, 0, 0, 0, 6]);

    let counts = payload(
        LexOutcome::CompilerFailure(LexCompilerFailure::PassCountDisagreement {
            expected_lexemes: 1,
            actual_lexemes: 2,
            expected_tokens: 3,
            actual_tokens: 4,
        }),
        &bundle,
    );
    let mut expected = vec![3, 2];
    for value in [1_u64, 2, 3, 4] {
        expected.extend_from_slice(&value.to_be_bytes());
    }
    assert_eq!(counts, expected);
    assert_eq!(
        payload(
            LexOutcome::CompilerFailure(LexCompilerFailure::CounterOverflow),
            &bundle
        ),
        [3, 3]
    );
}

#[test]
fn maximum_alternating_response_is_reserved_and_encoded_once() {
    const SOURCE_BYTES: usize = 1_048_576;
    let exact: Vec<u8> = (0..SOURCE_BYTES)
        .map(|index| if index % 2 == 0 { b'a' } else { b' ' })
        .collect();
    let limits = SourceLimits {
        max_sources: 1,
        max_logical_path_bytes: 64,
        max_source_bytes: SOURCE_BYTES as u64,
        max_total_source_bytes: SOURCE_BYTES as u64,
        max_binding_bytes: 2_097_152,
    };
    let bundle = whitefoot_contract::SourceBundle::with_limits(
        &[whitefoot_contract::SourceInput::new("maximum.wf", &exact)],
        limits,
    )
    .unwrap();
    let outcome = whitefoot_lexer::lex_v0_9(
        &bundle,
        whitefoot_lexer::LexLimits {
            max_sources: 1,
            max_source_bytes: SOURCE_BYTES as u64,
            max_total_source_bytes: SOURCE_BYTES as u64,
            max_token_bytes: 1,
            max_tokens: (SOURCE_BYTES / 2) as u64,
            max_lexemes: SOURCE_BYTES as u64,
        },
    );
    let response = crate::projection::encode_observation(&bundle, outcome).unwrap();
    assert_eq!(response.len(), 63 + (17 * SOURCE_BYTES));
}
