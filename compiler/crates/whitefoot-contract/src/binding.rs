use core::fmt;
use std::sync::Arc;

use crate::{LogicalPath, LogicalPathError, SourceBundle, SourceLimit, SourceLimits, SpecHash};

const MAGIC: [u8; 8] = *b"WFSOURCE";

/// Current canonical source-binding codec version.
pub const SOURCE_BINDING_CODEC_VERSION: u16 = 1;

/// One source record in an unverified source/spec binding.
///
/// Its [`crate::SourceId`] is implicit from sequence position and is therefore
/// not redundantly encoded.
#[derive(Clone, Eq, PartialEq)]
pub struct BoundSource {
    logical_path: LogicalPath,
    bytes: Arc<[u8]>,
}

impl fmt::Debug for BoundSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoundSource")
            .field("logical_path", &self.logical_path)
            .field("byte_len", &self.bytes.len())
            .finish()
    }
}

impl BoundSource {
    /// Creates an unverified source record from a valid logical path and raw bytes.
    #[must_use]
    pub fn new(logical_path: LogicalPath, bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            logical_path,
            bytes: bytes.into(),
        }
    }

    /// Returns the exact logical source name.
    #[must_use]
    pub const fn logical_path(&self) -> &LogicalPath {
        &self.logical_path
    }

    /// Returns the exact unnormalized source bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Canonical, but not yet verified, binding of ordered source bytes to a spec.
#[derive(Clone, Eq, PartialEq)]
pub struct SourceBinding {
    spec_hash: SpecHash,
    sources: Box<[BoundSource]>,
}

impl fmt::Debug for SourceBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceBinding")
            .field("spec_hash", &self.spec_hash)
            .field("sources", &self.sources)
            .finish()
    }
}

impl SourceBinding {
    /// Creates a candidate binding without claiming that it matches an input bundle.
    #[must_use]
    pub fn new(spec_hash: SpecHash, sources: Vec<BoundSource>) -> Self {
        Self {
            spec_hash,
            sources: sources.into_boxed_slice(),
        }
    }

    /// Constructs the canonical candidate for an existing source bundle.
    #[must_use]
    pub fn from_bundle(spec_hash: SpecHash, bundle: &SourceBundle) -> Self {
        let sources = bundle
            .iter()
            .map(|(_, source)| {
                BoundSource::new(source.logical_path().clone(), source.shared_bytes())
            })
            .collect();
        Self::new(spec_hash, sources)
    }

    /// Returns the exact specification identity carried by the candidate.
    #[must_use]
    pub const fn spec_hash(&self) -> SpecHash {
        self.spec_hash
    }

    /// Returns source records in their binding order.
    #[must_use]
    pub fn sources(&self) -> &[BoundSource] {
        &self.sources
    }

    /// Encodes the binding in its unique versioned byte form under explicit ceilings.
    pub fn encode_canonical(&self, limits: SourceLimits) -> Result<Vec<u8>, EncodeError> {
        let capacity = self.encoded_capacity(limits)?;
        let mut encoded = Vec::with_capacity(capacity);
        encoded.extend_from_slice(&MAGIC);
        encoded.extend_from_slice(&SOURCE_BINDING_CODEC_VERSION.to_be_bytes());
        encoded.extend_from_slice(self.spec_hash.digest().as_bytes());
        write_len(&mut encoded, self.sources.len())?;
        for source in &self.sources {
            write_len(&mut encoded, source.logical_path.as_str().len())?;
            encoded.extend_from_slice(source.logical_path.as_str().as_bytes());
            write_len(&mut encoded, source.bytes.len())?;
            encoded.extend_from_slice(&source.bytes);
        }
        Ok(encoded)
    }

    fn encoded_capacity(&self, limits: SourceLimits) -> Result<usize, EncodeError> {
        let source_count =
            u64::try_from(self.sources.len()).map_err(|_| EncodeError::LengthOverflow)?;
        check_encode_limit(
            SourceLimit::Sources,
            u64::from(limits.max_sources()),
            source_count,
        )?;

        let mut encoded_len = 8_u64 + 2 + 32 + 8;
        let mut total_source_bytes = 0_u64;
        for source in &self.sources {
            let path_len = u64::try_from(source.logical_path.as_str().len())
                .map_err(|_| EncodeError::LengthOverflow)?;
            check_encode_limit(
                SourceLimit::LogicalPathBytes,
                limits.max_logical_path_bytes(),
                path_len,
            )?;
            let source_len =
                u64::try_from(source.bytes.len()).map_err(|_| EncodeError::LengthOverflow)?;
            check_encode_limit(
                SourceLimit::SourceBytes,
                limits.max_source_bytes(),
                source_len,
            )?;
            total_source_bytes = total_source_bytes
                .checked_add(source_len)
                .ok_or(EncodeError::LengthOverflow)?;
            check_encode_limit(
                SourceLimit::TotalSourceBytes,
                limits.max_total_source_bytes(),
                total_source_bytes,
            )?;
            encoded_len = encoded_len
                .checked_add(8)
                .and_then(|length| length.checked_add(path_len))
                .and_then(|length| length.checked_add(8))
                .and_then(|length| length.checked_add(source_len))
                .ok_or(EncodeError::LengthOverflow)?;
        }
        check_encode_limit(
            SourceLimit::BindingBytes,
            limits.max_binding_bytes(),
            encoded_len,
        )?;
        usize::try_from(encoded_len).map_err(|_| EncodeError::LengthOverflow)
    }

    /// Decodes one complete canonical binding under explicit input ceilings.
    pub fn decode_canonical(encoded: &[u8], limits: SourceLimits) -> Result<Self, DecodeError> {
        let encoded_len = u64::try_from(encoded.len()).map_err(|_| DecodeError::LengthOverflow)?;
        check_decode_limit(
            SourceLimit::BindingBytes,
            limits.max_binding_bytes(),
            encoded_len,
        )?;
        let mut reader = Reader::new(encoded);
        if reader.take(MAGIC.len())? != MAGIC {
            return Err(DecodeError::BadMagic);
        }
        let version = reader.read_u16()?;
        if version != SOURCE_BINDING_CODEC_VERSION {
            return Err(DecodeError::UnsupportedVersion(version));
        }
        let mut spec_bytes = [0_u8; 32];
        spec_bytes.copy_from_slice(reader.take(32)?);
        let spec_hash = SpecHash::from_sha256(spec_bytes);

        let source_count = reader.read_u64()?;
        check_decode_limit(
            SourceLimit::Sources,
            u64::from(limits.max_sources()),
            source_count,
        )?;
        let minimum_record_bytes = 17_u64;
        let possible_records = u64::try_from(reader.remaining())
            .map_err(|_| DecodeError::LengthOverflow)?
            / minimum_record_bytes;
        if source_count > possible_records {
            return Err(DecodeError::ImpossibleSourceCount(source_count));
        }

        let mut sources = Vec::new();
        let mut total_source_bytes = 0_u64;
        for _ in 0..source_count {
            let path_len = reader.read_u64()?;
            check_decode_limit(
                SourceLimit::LogicalPathBytes,
                limits.max_logical_path_bytes(),
                path_len,
            )?;
            let path_bytes = reader.take_u64(path_len)?;
            let path_text =
                core::str::from_utf8(path_bytes).map_err(|_| DecodeError::LogicalPathNotUtf8)?;
            let logical_path = LogicalPath::parse(path_text).map_err(DecodeError::LogicalPath)?;

            let source_len = reader.read_u64()?;
            check_decode_limit(
                SourceLimit::SourceBytes,
                limits.max_source_bytes(),
                source_len,
            )?;
            total_source_bytes = total_source_bytes
                .checked_add(source_len)
                .ok_or(DecodeError::LengthOverflow)?;
            check_decode_limit(
                SourceLimit::TotalSourceBytes,
                limits.max_total_source_bytes(),
                total_source_bytes,
            )?;
            let bytes: Arc<[u8]> = Arc::from(reader.take_u64(source_len)?);
            sources.push(BoundSource::new(logical_path, bytes));
        }

        if reader.remaining() != 0 {
            return Err(DecodeError::TrailingBytes(reader.remaining()));
        }
        Ok(Self::new(spec_hash, sources))
    }
}

fn write_len(encoded: &mut Vec<u8>, length: usize) -> Result<(), EncodeError> {
    let length = u64::try_from(length).map_err(|_| EncodeError::LengthOverflow)?;
    encoded.extend_from_slice(&length.to_be_bytes());
    Ok(())
}

fn check_encode_limit(limit: SourceLimit, maximum: u64, actual: u64) -> Result<(), EncodeError> {
    if actual > maximum {
        return Err(EncodeError::LimitExceeded {
            limit,
            maximum,
            actual,
        });
    }
    Ok(())
}

fn check_decode_limit(limit: SourceLimit, maximum: u64, actual: u64) -> Result<(), DecodeError> {
    if actual > maximum {
        return Err(DecodeError::LimitExceeded {
            limit,
            maximum,
            actual,
        });
    }
    Ok(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    const fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], DecodeError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(DecodeError::LengthOverflow)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(DecodeError::Truncated {
                offset: self.bytes.len(),
            })?;
        self.offset = end;
        Ok(value)
    }

    fn take_u64(&mut self, length: u64) -> Result<&'a [u8], DecodeError> {
        let remaining = u64::try_from(self.remaining()).map_err(|_| DecodeError::LengthOverflow)?;
        if length > remaining {
            return Err(DecodeError::Truncated {
                offset: self.bytes.len(),
            });
        }
        let length = usize::try_from(length).map_err(|_| DecodeError::LengthOverflow)?;
        self.take(length)
    }

    fn read_u16(&mut self) -> Result<u16, DecodeError> {
        let mut bytes = [0_u8; 2];
        bytes.copy_from_slice(self.take(2)?);
        Ok(u16::from_be_bytes(bytes))
    }

    fn read_u64(&mut self) -> Result<u64, DecodeError> {
        let mut bytes = [0_u8; 8];
        bytes.copy_from_slice(self.take(8)?);
        Ok(u64::from_be_bytes(bytes))
    }
}

/// Why canonical source-binding encoding cannot complete.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// A host collection length does not fit the portable `u64` framing.
    LengthOverflow,
    /// An explicit artifact-output resource ceiling was exceeded.
    LimitExceeded {
        /// Ceiling category.
        limit: SourceLimit,
        /// Configured inclusive maximum.
        maximum: u64,
        /// Value that would be encoded.
        actual: u64,
    },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow => formatter.write_str("source-binding length exceeds u64"),
            Self::LimitExceeded {
                limit,
                maximum,
                actual,
            } => write!(
                formatter,
                "source-binding output limit {limit:?} is {maximum}, attempted {actual}"
            ),
        }
    }
}

impl std::error::Error for EncodeError {}

/// Why bytes are not one complete canonical source binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// The domain-separation prefix is not the source-binding prefix.
    BadMagic,
    /// The codec version is not understood.
    UnsupportedVersion(u16),
    /// The input ends before a declared field does.
    Truncated {
        /// First unavailable byte position.
        offset: usize,
    },
    /// A framed length cannot be represented or accumulated.
    LengthOverflow,
    /// The declared source count cannot fit in the remaining input.
    ImpossibleSourceCount(u64),
    /// A logical path is not UTF-8 transport metadata.
    LogicalPathNotUtf8,
    /// A logical path violates the portable path contract.
    LogicalPath(LogicalPathError),
    /// An explicit artifact-input resource ceiling was exceeded.
    LimitExceeded {
        /// Ceiling category.
        limit: SourceLimit,
        /// Configured inclusive maximum.
        maximum: u64,
        /// Encoded value.
        actual: u64,
    },
    /// Complete decoding left extra, unauthenticated bytes.
    TrailingBytes(usize),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMagic => formatter.write_str("invalid source-binding magic"),
            Self::UnsupportedVersion(version) => {
                write!(
                    formatter,
                    "unsupported source-binding codec version {version}"
                )
            }
            Self::Truncated { offset } => {
                write!(formatter, "source binding is truncated at byte {offset}")
            }
            Self::LengthOverflow => formatter.write_str("source-binding length overflow"),
            Self::ImpossibleSourceCount(count) => {
                write!(
                    formatter,
                    "source count {count} cannot fit in the remaining bytes"
                )
            }
            Self::LogicalPathNotUtf8 => {
                formatter.write_str("source-binding logical path is not UTF-8")
            }
            Self::LogicalPath(error) => write!(formatter, "{error}"),
            Self::LimitExceeded {
                limit,
                maximum,
                actual,
            } => write!(
                formatter,
                "source-binding limit {limit:?} is {maximum}, encoded {actual}"
            ),
            Self::TrailingBytes(count) => {
                write!(formatter, "source binding has {count} trailing bytes")
            }
        }
    }
}

impl std::error::Error for DecodeError {}

#[cfg(test)]
mod tests;
