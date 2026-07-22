use core::fmt;

use whitefoot_contract::{
    BoundSource, ByteOffset, LogicalPath, SourceBinding, SourceBundle, SourceId, SpecHash,
};

/// One source view whose candidate binding has been verified in full.
#[derive(Clone, Copy, Debug)]
pub struct VerifiedSource<'binding> {
    source_id: SourceId,
    source: &'binding BoundSource,
}

impl<'binding> VerifiedSource<'binding> {
    /// Returns the source's position in this verified binding.
    #[must_use]
    pub const fn source_id(self) -> SourceId {
        self.source_id
    }

    /// Returns the exact verified logical path.
    #[must_use]
    pub const fn logical_path(self) -> &'binding LogicalPath {
        self.source.logical_path()
    }

    /// Returns the exact verified raw bytes.
    #[must_use]
    pub fn bytes(self) -> &'binding [u8] {
        self.source.bytes()
    }
}

/// A source/spec binding that exactly matches the audit invocation's inputs.
///
/// The constructor is private. Later verification stages can require this type
/// rather than accepting an unverified [`SourceBinding`].
#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedSourceBinding {
    binding: SourceBinding,
}

impl VerifiedSourceBinding {
    /// Returns the verified specification identity.
    #[must_use]
    pub const fn spec_hash(&self) -> SpecHash {
        self.binding.spec_hash()
    }

    /// Returns the number of verified ordered sources.
    #[must_use]
    pub fn source_count(&self) -> usize {
        self.binding.sources().len()
    }

    /// Iterates narrow verified views in candidate transport order.
    pub fn sources(&self) -> impl Iterator<Item = VerifiedSource<'_>> {
        (0_u32..)
            .zip(self.binding.sources())
            .map(|(ordinal, source)| VerifiedSource {
                source_id: SourceId::from_ordinal(ordinal),
                source,
            })
    }
}

/// Verifies exact specification identity, source order, names, and raw bytes.
///
/// This is deliberately not a language judgment. Exceeding a source limit or
/// providing malformed source remains an input/toolchain or later lexer
/// result, never a Whitefoot rule rejection produced here.
pub fn verify_source_binding(
    expected_spec: SpecHash,
    expected_sources: &SourceBundle,
    candidate: SourceBinding,
) -> Result<VerifiedSourceBinding, VerifySourceBindingError> {
    if candidate.spec_hash() != expected_spec {
        return Err(VerifySourceBindingError::SpecificationMismatch {
            expected: expected_spec,
            actual: candidate.spec_hash(),
        });
    }
    if candidate.sources().len() != expected_sources.len() {
        return Err(VerifySourceBindingError::SourceCountMismatch {
            expected: expected_sources.len(),
            actual: candidate.sources().len(),
        });
    }

    for (ordinal, (expected, actual)) in
        (0_u32..).zip(expected_sources.files().iter().zip(candidate.sources()))
    {
        let source_id = SourceId::from_ordinal(ordinal);
        if actual.logical_path() != expected.logical_path() {
            return Err(VerifySourceBindingError::LogicalPathMismatch { source: source_id });
        }
        if actual.bytes() != expected.bytes() {
            let first_difference = actual
                .bytes()
                .iter()
                .zip(expected.bytes())
                .position(|(actual_byte, expected_byte)| actual_byte != expected_byte)
                .unwrap_or_else(|| actual.bytes().len().min(expected.bytes().len()));
            let first_difference = u64::try_from(first_difference)
                .map(ByteOffset::new)
                .map_err(|_| VerifySourceBindingError::HostLengthOverflow { source: source_id })?;
            let actual_len = u64::try_from(actual.bytes().len())
                .map_err(|_| VerifySourceBindingError::HostLengthOverflow { source: source_id })?;
            return Err(VerifySourceBindingError::SourceBytesMismatch {
                source: source_id,
                first_difference,
                expected_len: expected.byte_len(),
                actual_len,
            });
        }
    }

    Ok(VerifiedSourceBinding { binding: candidate })
}

/// Why a candidate is not bound to the audit invocation's source/spec input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerifySourceBindingError {
    /// The candidate names another numbered specification.
    SpecificationMismatch {
        /// Identity required by the invocation.
        expected: SpecHash,
        /// Identity carried by the candidate.
        actual: SpecHash,
    },
    /// The ordered source sequence has another length.
    SourceCountMismatch {
        /// Number of invocation sources.
        expected: usize,
        /// Number of candidate sources.
        actual: usize,
    },
    /// One source position carries another logical name.
    LogicalPathMismatch {
        /// Implicit source identity.
        source: SourceId,
    },
    /// One source position carries different raw bytes.
    SourceBytesMismatch {
        /// Implicit source identity.
        source: SourceId,
        /// First differing byte, or the common-prefix length when lengths differ.
        first_difference: ByteOffset,
        /// Invocation source length.
        expected_len: u64,
        /// Candidate source length.
        actual_len: u64,
    },
    /// A host collection length cannot fit the portable byte-offset domain.
    HostLengthOverflow {
        /// Source whose host length could not be represented.
        source: SourceId,
    },
}

impl fmt::Display for VerifySourceBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SpecificationMismatch { expected, actual } => {
                write!(
                    formatter,
                    "specification mismatch: expected {expected}, got {actual}"
                )
            }
            Self::SourceCountMismatch { expected, actual } => {
                write!(
                    formatter,
                    "source count mismatch: expected {expected}, got {actual}"
                )
            }
            Self::LogicalPathMismatch { source } => {
                write!(formatter, "logical path mismatch at source {source}")
            }
            Self::SourceBytesMismatch {
                source,
                first_difference,
                expected_len,
                actual_len,
            } => write!(
                formatter,
                "source bytes mismatch at source {source}, byte {}: expected length {expected_len}, got {actual_len}",
                first_difference.value()
            ),
            Self::HostLengthOverflow { source } => {
                write!(
                    formatter,
                    "host source length cannot fit u64 at source {source}"
                )
            }
        }
    }
}

impl std::error::Error for VerifySourceBindingError {}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use whitefoot_contract::{KERNEL_SPEC_V0_8_HASH, SourceInput};

    fn expected_bundle() -> SourceBundle {
        SourceBundle::with_limits(
            &[
                SourceInput::new("z.wf", b"first"),
                SourceInput::new("a.wf", &[0xff, 0x00, b'\n']),
            ],
            whitefoot_contract::SourceLimits::REPRESENTABLE,
        )
        .unwrap()
    }

    fn candidate(sources: &[SourceInput<'_>]) -> SourceBinding {
        SourceBinding::try_from_sources(
            KERNEL_SPEC_V0_8_HASH,
            sources,
            whitefoot_contract::SourceLimits::REPRESENTABLE,
        )
        .unwrap()
    }

    #[test]
    fn exact_candidate_yields_unforgeable_verified_state() {
        let expected = expected_bundle();
        let candidate = SourceBinding::try_from_bundle(
            KERNEL_SPEC_V0_8_HASH,
            &expected,
            whitefoot_contract::SourceLimits::REPRESENTABLE,
        )
        .unwrap();
        let verified = verify_source_binding(KERNEL_SPEC_V0_8_HASH, &expected, candidate).unwrap();
        assert_eq!(verified.spec_hash(), KERNEL_SPEC_V0_8_HASH);
        assert_eq!(verified.source_count(), 2);
        let sources: Vec<_> = verified.sources().collect();
        assert_eq!(sources[1].source_id(), SourceId::from_ordinal(1));
        assert_eq!(sources[1].logical_path().as_str(), "a.wf");
        assert_eq!(sources[1].bytes(), [0xff, 0x00, b'\n']);
    }

    #[test]
    fn specification_mutation_fails_first() {
        let expected = expected_bundle();
        let wrong = SpecHash::from_sha256([0x55; 32]);
        let candidate = SourceBinding::try_from_sources(
            wrong,
            &[],
            whitefoot_contract::SourceLimits::REPRESENTABLE,
        )
        .unwrap();
        assert!(matches!(
            verify_source_binding(KERNEL_SPEC_V0_8_HASH, &expected, candidate),
            Err(VerifySourceBindingError::SpecificationMismatch { .. })
        ));
    }

    #[test]
    fn source_order_and_path_mutations_are_distinct() {
        let expected = expected_bundle();
        let swapped = candidate(&[
            SourceInput::new("a.wf", &[0xff, 0x00, b'\n']),
            SourceInput::new("z.wf", b"first"),
        ]);
        assert!(matches!(
            verify_source_binding(KERNEL_SPEC_V0_8_HASH, &expected, swapped),
            Err(VerifySourceBindingError::LogicalPathMismatch {
                source,
                ..
            }) if source == SourceId::from_ordinal(0)
        ));

        let renamed = candidate(&[
            SourceInput::new("other.wf", b"first"),
            SourceInput::new("a.wf", &[0xff, 0x00, b'\n']),
        ]);
        assert!(matches!(
            verify_source_binding(KERNEL_SPEC_V0_8_HASH, &expected, renamed),
            Err(VerifySourceBindingError::LogicalPathMismatch {
                source,
                ..
            }) if source == SourceId::from_ordinal(0)
        ));
    }

    #[test]
    fn byte_mutation_reports_the_first_difference() {
        let expected = expected_bundle();
        let changed = candidate(&[
            SourceInput::new("z.wf", b"firzt"),
            SourceInput::new("a.wf", &[0xff, 0x00, b'\n']),
        ]);
        assert!(matches!(
            verify_source_binding(KERNEL_SPEC_V0_8_HASH, &expected, changed),
            Err(VerifySourceBindingError::SourceBytesMismatch {
                source,
                first_difference,
                ..
            }) if source == SourceId::from_ordinal(0) && first_difference == ByteOffset::new(3)
        ));
    }

    #[test]
    fn duplicate_candidate_paths_fail_at_the_first_positional_mismatch() {
        let expected = expected_bundle();
        let duplicate = candidate(&[
            SourceInput::new("z.wf", b"first"),
            SourceInput::new("z.wf", &[0xff, 0x00, b'\n']),
        ]);
        assert!(matches!(
            verify_source_binding(KERNEL_SPEC_V0_8_HASH, &expected, duplicate),
            Err(VerifySourceBindingError::LogicalPathMismatch { source, .. })
                if source == SourceId::from_ordinal(1)
        ));
    }

    #[test]
    fn canonical_decode_does_not_bypass_verification() {
        let expected = expected_bundle();
        let wrong = candidate(&[
            SourceInput::new("z.wf", b"wrong"),
            SourceInput::new("a.wf", &[0xff, 0x00, b'\n']),
        ]);
        let encoded = wrong
            .encode_canonical(whitefoot_contract::SourceLimits::REPRESENTABLE)
            .unwrap();
        let decoded = SourceBinding::decode_canonical(
            &encoded,
            whitefoot_contract::SourceLimits::REPRESENTABLE,
        )
        .unwrap();
        assert!(matches!(
            verify_source_binding(KERNEL_SPEC_V0_8_HASH, &expected, decoded),
            Err(VerifySourceBindingError::SourceBytesMismatch { .. })
        ));
    }
}
