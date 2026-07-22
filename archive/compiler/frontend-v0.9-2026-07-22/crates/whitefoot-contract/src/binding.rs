use core::fmt;

use crate::{
    LogicalPath, LogicalPathError, SourceBundle, SourceFile, SourceInput, SourceLimit,
    SourceLimits, SpecHash,
};

const MAGIC: [u8; 8] = *b"WFSOURCE";

/// Current canonical source-binding codec version.
pub const SOURCE_BINDING_CODEC_VERSION: u16 = 1;

/// One source record in an unverified source/spec binding.
///
/// Its [`crate::SourceId`] is implicit from sequence position and is therefore
/// not redundantly encoded.
#[derive(Eq, PartialEq)]
pub struct BoundSource {
    logical_path: LogicalPath,
    bytes: Vec<u8>,
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
#[derive(Eq, PartialEq)]
pub struct SourceBinding {
    spec_hash: SpecHash,
    sources: Vec<BoundSource>,
}

trait SourceRecordView {
    fn record_path(&self) -> &str;
    fn record_bytes(&self) -> &[u8];
}

impl SourceRecordView for SourceInput<'_> {
    fn record_path(&self) -> &str {
        self.logical_path()
    }

    fn record_bytes(&self) -> &[u8] {
        self.bytes()
    }
}

impl SourceRecordView for SourceFile {
    fn record_path(&self) -> &str {
        self.logical_path().as_str()
    }

    fn record_bytes(&self) -> &[u8] {
        self.bytes()
    }
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
    /// Fallibly copies candidate records under the complete binding ceilings.
    pub fn try_from_sources(
        spec_hash: SpecHash,
        records: &[SourceInput<'_>],
        limits: SourceLimits,
    ) -> Result<Self, EncodeError> {
        Self::try_from_records(spec_hash, records, limits)
    }

    fn try_from_records<Record>(
        spec_hash: SpecHash,
        records: &[Record],
        limits: SourceLimits,
    ) -> Result<Self, EncodeError>
    where
        Record: SourceRecordView,
    {
        let source_capacity = records.len();
        let source_count =
            u64::try_from(source_capacity).map_err(|_| EncodeError::LengthOverflow)?;
        check_encode_limit(
            SourceLimit::Sources,
            u64::from(limits.max_sources()),
            source_count,
        )?;

        let mut total_source_bytes = 0_u64;
        let mut encoded_len = 8_u64 + 2 + 32 + 8;
        check_encode_limit(
            SourceLimit::BindingBytes,
            limits.max_binding_bytes(),
            encoded_len,
        )?;

        for record in records {
            let path = record.record_path();
            let bytes = record.record_bytes();
            let path_len = u64::try_from(path.len()).map_err(|_| EncodeError::LengthOverflow)?;
            check_encode_limit(
                SourceLimit::LogicalPathBytes,
                limits.max_logical_path_bytes(),
                path_len,
            )?;
            let source_len = u64::try_from(bytes.len()).map_err(|_| EncodeError::LengthOverflow)?;
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
            check_encode_limit(
                SourceLimit::BindingBytes,
                limits.max_binding_bytes(),
                encoded_len,
            )?;
        }

        for record in records {
            LogicalPath::validate(record.record_path()).map_err(map_encode_path_error)?;
        }

        let mut sources = Vec::new();
        try_reserve_encode(
            &mut sources,
            source_capacity,
            SourceLimit::Sources,
            source_count,
        )?;
        for record in records {
            let path = record.record_path();
            let bytes = record.record_bytes();
            let logical_path = LogicalPath::parse(path).map_err(map_encode_path_error)?;
            let source_len = u64::try_from(bytes.len()).map_err(|_| EncodeError::LengthOverflow)?;
            let mut owned_bytes = Vec::new();
            try_reserve_encode(
                &mut owned_bytes,
                bytes.len(),
                SourceLimit::SourceBytes,
                source_len,
            )?;
            owned_bytes.extend_from_slice(bytes);
            sources.push(BoundSource {
                logical_path,
                bytes: owned_bytes,
            });
        }

        Ok(Self { spec_hash, sources })
    }

    /// Fallibly constructs the canonical candidate for an existing source bundle.
    pub fn try_from_bundle(
        spec_hash: SpecHash,
        bundle: &SourceBundle,
        limits: SourceLimits,
    ) -> Result<Self, EncodeError> {
        Self::try_from_records(spec_hash, bundle.files(), limits)
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
        let requested = u64::try_from(capacity).map_err(|_| EncodeError::LengthOverflow)?;
        let mut encoded = Vec::new();
        try_reserve_encode(&mut encoded, capacity, SourceLimit::BindingBytes, requested)?;
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

        let source_capacity =
            usize::try_from(source_count).map_err(|_| DecodeError::LengthOverflow)?;
        let mut sources = Vec::new();
        try_reserve_decode(
            &mut sources,
            source_capacity,
            SourceLimit::Sources,
            source_count,
        )?;
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
            let logical_path = LogicalPath::parse(path_text).map_err(map_decode_path_error)?;

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
            let source_bytes = reader.take_u64(source_len)?;
            let mut bytes = Vec::new();
            try_reserve_decode(
                &mut bytes,
                source_bytes.len(),
                SourceLimit::SourceBytes,
                source_len,
            )?;
            bytes.extend_from_slice(source_bytes);
            sources.push(BoundSource {
                logical_path,
                bytes,
            });
        }

        if reader.remaining() != 0 {
            return Err(DecodeError::TrailingBytes(reader.remaining()));
        }
        Ok(Self { spec_hash, sources })
    }
}

fn map_encode_path_error(error: LogicalPathError) -> EncodeError {
    match error {
        LogicalPathError::LengthOverflow => EncodeError::LengthOverflow,
        LogicalPathError::StorageUnavailable { requested } => EncodeError::StorageUnavailable {
            limit: SourceLimit::LogicalPathBytes,
            requested,
        },
        error => EncodeError::LogicalPath(error),
    }
}

fn map_decode_path_error(error: LogicalPathError) -> DecodeError {
    match error {
        LogicalPathError::LengthOverflow => DecodeError::LengthOverflow,
        LogicalPathError::StorageUnavailable { requested } => DecodeError::StorageUnavailable {
            limit: SourceLimit::LogicalPathBytes,
            requested,
        },
        error => DecodeError::LogicalPath(error),
    }
}

fn try_reserve_encode<T>(
    values: &mut Vec<T>,
    additional: usize,
    limit: SourceLimit,
    requested: u64,
) -> Result<(), EncodeError> {
    values
        .try_reserve_exact(additional)
        .map_err(|_| EncodeError::StorageUnavailable { limit, requested })
}

fn try_reserve_decode<T>(
    values: &mut Vec<T>,
    additional: usize,
    limit: SourceLimit,
    requested: u64,
) -> Result<(), DecodeError> {
    values
        .try_reserve_exact(additional)
        .map_err(|_| DecodeError::StorageUnavailable { limit, requested })
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

/// Why candidate source-binding construction or canonical encoding cannot complete.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// A host collection length does not fit the portable `u64` framing.
    LengthOverflow,
    /// A candidate logical path violates the portable path contract.
    LogicalPath(LogicalPathError),
    /// An explicit candidate or source-binding wire ceiling was exceeded.
    LimitExceeded {
        /// Ceiling category.
        limit: SourceLimit,
        /// Configured inclusive maximum.
        maximum: u64,
        /// Value that would be encoded.
        actual: u64,
    },
    /// The allocator could not reserve validated binding storage.
    StorageUnavailable {
        /// Storage category that could not be reserved.
        limit: SourceLimit,
        /// Exact requested count or byte length.
        requested: u64,
    },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow => formatter.write_str("source-binding length exceeds u64"),
            Self::LogicalPath(error) => write!(formatter, "{error}"),
            Self::LimitExceeded {
                limit,
                maximum,
                actual,
            } => write!(
                formatter,
                "source-binding limit {limit:?} is {maximum}, attempted {actual}"
            ),
            Self::StorageUnavailable { limit, requested } => write!(
                formatter,
                "storage unavailable for source binding {limit:?}, requested {requested}"
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
    /// An explicit source-binding input ceiling was exceeded.
    LimitExceeded {
        /// Ceiling category.
        limit: SourceLimit,
        /// Configured inclusive maximum.
        maximum: u64,
        /// Encoded value.
        actual: u64,
    },
    /// The allocator could not reserve validated decoded storage.
    StorageUnavailable {
        /// Storage category that could not be reserved.
        limit: SourceLimit,
        /// Exact requested count or byte length.
        requested: u64,
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
            Self::StorageUnavailable { limit, requested } => write!(
                formatter,
                "storage unavailable while decoding source binding {limit:?}, requested {requested}"
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
