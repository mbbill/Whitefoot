use core::fmt;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Dense source ordinal interpreted within an accompanying source context.
///
/// This is a portable coordinate, not a validated or global identity. The same
/// value can name different files in different bundles. Code that needs a
/// bundle-bound source or span must retain the handle returned by that bundle.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceId(u32);

impl SourceId {
    /// Creates an identity from a bundle-order ordinal.
    #[must_use]
    pub const fn from_ordinal(ordinal: u32) -> Self {
        Self(ordinal)
    }

    /// Returns the bundle-order ordinal.
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.0
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// A raw byte offset within one source file.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ByteOffset(u64);

impl ByteOffset {
    /// Creates an offset from its raw byte count.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw byte count.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// A validated half-open byte range bound to one exact bundle source.
///
/// The borrowed source file is the identity boundary: this handle can expose
/// only the bytes against which its offsets were validated. Persisted artifact
/// coordinates will instead require the enclosing source-binding identity.
#[derive(Clone, Copy)]
pub struct SourceSpan<'bundle> {
    source: SourceId,
    start: ByteOffset,
    end: ByteOffset,
    start_index: usize,
    end_index: usize,
    file: &'bundle SourceFile,
}

impl fmt::Debug for SourceSpan<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceSpan")
            .field("source", &self.source)
            .field("logical_path", &self.file.logical_path())
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

impl<'bundle> SourceSpan<'bundle> {
    /// Returns the source containing the complete span.
    #[must_use]
    pub const fn source(self) -> SourceId {
        self.source
    }

    /// Returns the inclusive starting byte offset.
    #[must_use]
    pub const fn start(self) -> ByteOffset {
        self.start
    }

    /// Returns the exclusive ending byte offset.
    #[must_use]
    pub const fn end(self) -> ByteOffset {
        self.end
    }

    /// Returns the exact source file against which this span was validated.
    #[must_use]
    pub const fn file(self) -> &'bundle SourceFile {
        self.file
    }

    /// Returns the exact bytes covered by the validated half-open range.
    #[must_use]
    pub fn bytes(self) -> &'bundle [u8] {
        &self.file.bytes()[self.start_index..self.end_index]
    }
}

/// A portable, bundle-local source name.
///
/// Logical paths use `/` separators and ASCII components containing only
/// letters, digits, `.`, `_`, and `-`. They are never host filesystem paths.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LogicalPath(Box<str>);

impl LogicalPath {
    /// Validates a logical source path without normalizing it.
    pub fn parse(value: impl Into<Box<str>>) -> Result<Self, LogicalPathError> {
        let value = value.into();
        if value.is_empty() {
            return Err(LogicalPathError::Empty);
        }
        if value.starts_with('/') {
            return Err(LogicalPathError::Absolute);
        }
        for (index, byte) in value.bytes().enumerate() {
            let valid = byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/');
            if !valid {
                return Err(LogicalPathError::InvalidByte { index, byte });
            }
        }
        for component in value.split('/') {
            if component.is_empty() {
                return Err(LogicalPathError::EmptyComponent);
            }
            if matches!(component, "." | "..") {
                return Err(LogicalPathError::DotComponent);
            }
        }
        Ok(Self(value))
    }

    /// Returns the exact logical path text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LogicalPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Why a logical source path is not a portable relative path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogicalPathError {
    /// The path has no bytes.
    Empty,
    /// The path begins at a filesystem root.
    Absolute,
    /// A component between separators has no bytes.
    EmptyComponent,
    /// A component is `.` or `..`.
    DotComponent,
    /// A byte is outside the closed portable spelling.
    InvalidByte {
        /// Zero-based byte position.
        index: usize,
        /// Offending byte value.
        byte: u8,
    },
}

impl fmt::Display for LogicalPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("logical path is empty"),
            Self::Absolute => formatter.write_str("logical path is absolute"),
            Self::EmptyComponent => formatter.write_str("logical path has an empty component"),
            Self::DotComponent => formatter.write_str("logical path contains . or .."),
            Self::InvalidByte { index, byte } => {
                write!(
                    formatter,
                    "logical path byte {index} is not portable: 0x{byte:02x}"
                )
            }
        }
    }
}

impl std::error::Error for LogicalPathError {}

/// One caller-supplied source before bundle validation.
#[derive(Clone, Eq, PartialEq)]
pub struct SourceInput {
    logical_path: Box<str>,
    bytes: Vec<u8>,
}

impl fmt::Debug for SourceInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceInput")
            .field("logical_path", &self.logical_path)
            .field("byte_len", &self.bytes.len())
            .finish()
    }
}

impl SourceInput {
    /// Creates an input while preserving its exact path spelling and source bytes.
    #[must_use]
    pub fn new(logical_path: impl Into<Box<str>>, bytes: Vec<u8>) -> Self {
        Self {
            logical_path: logical_path.into(),
            bytes,
        }
    }
}

/// One validated source in a bundle.
#[derive(Clone, Eq, PartialEq)]
pub struct SourceFile {
    logical_path: LogicalPath,
    bytes: Arc<[u8]>,
    byte_len: u64,
}

impl fmt::Debug for SourceFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceFile")
            .field("logical_path", &self.logical_path)
            .field("byte_len", &self.byte_len)
            .finish()
    }
}

impl SourceFile {
    /// Returns the portable logical source name.
    #[must_use]
    pub const fn logical_path(&self) -> &LogicalPath {
        &self.logical_path
    }

    /// Returns the exact unnormalized source bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the source length as a portable byte count.
    #[must_use]
    pub const fn byte_len(&self) -> u64 {
        self.byte_len
    }

    pub(crate) fn shared_bytes(&self) -> Arc<[u8]> {
        Arc::clone(&self.bytes)
    }
}

/// Which implementation resource ceiling was exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceLimit {
    /// Bytes in the complete canonical source-binding transport.
    BindingBytes,
    /// Number of logical source files.
    Sources,
    /// Bytes in one logical path.
    LogicalPathBytes,
    /// Bytes in one source file.
    SourceBytes,
    /// Sum of bytes in every source file.
    TotalSourceBytes,
}

/// Explicit toolchain input ceilings, separate from Whitefoot legality.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceLimits {
    /// Maximum source count.
    pub max_sources: u32,
    /// Maximum bytes in one logical path.
    pub max_logical_path_bytes: u64,
    /// Maximum bytes in one source file.
    pub max_source_bytes: u64,
    /// Maximum total bytes across all source files.
    pub max_total_source_bytes: u64,
    /// Maximum bytes in one complete canonical source binding.
    pub max_binding_bytes: u64,
}

impl SourceLimits {
    /// Maximum counts and byte lengths accepted by the current schemas.
    ///
    /// The source count is capped at `u32::MAX`, so the final `u32` ordinal is
    /// deliberately not assigned. Production callers must use tighter limits.
    pub const REPRESENTABLE: Self = Self {
        max_sources: u32::MAX,
        max_logical_path_bytes: u64::MAX,
        max_source_bytes: u64::MAX,
        max_total_source_bytes: u64::MAX,
        max_binding_bytes: u64::MAX,
    };

    /// Maximum source count.
    #[must_use]
    pub const fn max_sources(self) -> u32 {
        self.max_sources
    }

    /// Maximum bytes in one logical path.
    #[must_use]
    pub const fn max_logical_path_bytes(self) -> u64 {
        self.max_logical_path_bytes
    }

    /// Maximum bytes in one source file.
    #[must_use]
    pub const fn max_source_bytes(self) -> u64 {
        self.max_source_bytes
    }

    /// Maximum total source bytes.
    #[must_use]
    pub const fn max_total_source_bytes(self) -> u64 {
        self.max_total_source_bytes
    }

    /// Maximum bytes in one complete canonical source binding.
    #[must_use]
    pub const fn max_binding_bytes(self) -> u64 {
        self.max_binding_bytes
    }
}

/// One closed, explicitly ordered collection of source files.
///
/// File order is caller authority. Paths never sort the bundle. Later parsing
/// combines complete top-level items in file order, then item order within each
/// file; no token or item may cross a file boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceBundle {
    files: Box<[SourceFile]>,
    total_bytes: u64,
}

impl SourceBundle {
    /// Builds a bundle under explicit toolchain resource ceilings.
    pub fn with_limits(
        inputs: impl IntoIterator<Item = SourceInput>,
        limits: SourceLimits,
    ) -> Result<Self, SourceBundleError> {
        let mut files = Vec::new();
        let mut first_positions = BTreeMap::<LogicalPath, u32>::new();
        let mut total_bytes = 0_u64;

        for input in inputs {
            let position =
                u32::try_from(files.len()).map_err(|_| SourceBundleError::LimitExceeded {
                    limit: SourceLimit::Sources,
                    maximum: u64::from(limits.max_sources),
                    actual: u64::MAX,
                })?;
            if position >= limits.max_sources {
                return Err(SourceBundleError::LimitExceeded {
                    limit: SourceLimit::Sources,
                    maximum: u64::from(limits.max_sources),
                    actual: u64::from(position) + 1,
                });
            }

            let path_len = u64::try_from(input.logical_path.len())
                .map_err(|_| SourceBundleError::ArithmeticOverflow)?;
            if path_len > limits.max_logical_path_bytes {
                return Err(SourceBundleError::LimitExceeded {
                    limit: SourceLimit::LogicalPathBytes,
                    maximum: limits.max_logical_path_bytes,
                    actual: path_len,
                });
            }
            let logical_path =
                LogicalPath::parse(input.logical_path).map_err(SourceBundleError::LogicalPath)?;
            if let Some(first_position) = first_positions.get(&logical_path) {
                return Err(SourceBundleError::DuplicateLogicalPath {
                    path: logical_path,
                    first_position: *first_position,
                    duplicate_position: position,
                });
            }

            let source_len = u64::try_from(input.bytes.len())
                .map_err(|_| SourceBundleError::ArithmeticOverflow)?;
            if source_len > limits.max_source_bytes {
                return Err(SourceBundleError::LimitExceeded {
                    limit: SourceLimit::SourceBytes,
                    maximum: limits.max_source_bytes,
                    actual: source_len,
                });
            }
            total_bytes = total_bytes
                .checked_add(source_len)
                .ok_or(SourceBundleError::ArithmeticOverflow)?;
            if total_bytes > limits.max_total_source_bytes {
                return Err(SourceBundleError::LimitExceeded {
                    limit: SourceLimit::TotalSourceBytes,
                    maximum: limits.max_total_source_bytes,
                    actual: total_bytes,
                });
            }

            first_positions.insert(logical_path.clone(), position);
            files.push(SourceFile {
                logical_path,
                bytes: Arc::from(input.bytes),
                byte_len: source_len,
            });
        }

        Ok(Self {
            files: files.into_boxed_slice(),
            total_bytes,
        })
    }

    /// Returns the number of ordered source files.
    #[must_use]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns whether the closed input contains no source files.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Returns the checked sum of all source byte lengths.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Returns source files in authoritative caller-supplied order.
    #[must_use]
    pub const fn files(&self) -> &[SourceFile] {
        &self.files
    }

    /// Looks up a source by its bundle-order identity.
    #[must_use]
    pub fn file(&self, source: SourceId) -> Option<&SourceFile> {
        usize::try_from(source.ordinal())
            .ok()
            .and_then(|index| self.files.get(index))
    }

    /// Iterates in authoritative bundle order with derived source identities.
    pub fn iter(&self) -> impl Iterator<Item = (SourceId, &SourceFile)> {
        (0_u32..)
            .zip(self.files.iter())
            .map(|(ordinal, file)| (SourceId::from_ordinal(ordinal), file))
    }

    /// Validates and creates a half-open span within one source.
    pub fn span(
        &self,
        source: SourceId,
        start: ByteOffset,
        end: ByteOffset,
    ) -> Result<SourceSpan<'_>, SpanError> {
        let file = self.file(source).ok_or(SpanError::UnknownSource(source))?;
        if start > end {
            return Err(SpanError::Reversed { start, end });
        }
        let source_len = file.byte_len();
        if end.value() > source_len {
            return Err(SpanError::OutOfBounds { end, source_len });
        }
        let start_index = usize::try_from(start.value())
            .map_err(|_| SpanError::OutOfBounds { end, source_len })?;
        let end_index =
            usize::try_from(end.value()).map_err(|_| SpanError::OutOfBounds { end, source_len })?;
        Ok(SourceSpan {
            source,
            start,
            end,
            start_index,
            end_index,
            file,
        })
    }
}

/// Why source inputs cannot form a bundle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceBundleError {
    /// One logical path is structurally invalid.
    LogicalPath(LogicalPathError),
    /// Two source positions use the same case-sensitive logical path.
    DuplicateLogicalPath {
        /// Repeated logical path.
        path: LogicalPath,
        /// First bundle position.
        first_position: u32,
        /// Repeated bundle position.
        duplicate_position: u32,
    },
    /// An explicit implementation resource ceiling was exceeded.
    LimitExceeded {
        /// Ceiling category.
        limit: SourceLimit,
        /// Configured inclusive maximum.
        maximum: u64,
        /// Attempted value.
        actual: u64,
    },
    /// A byte count cannot be represented without wrapping.
    ArithmeticOverflow,
}

impl fmt::Display for SourceBundleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LogicalPath(error) => write!(formatter, "{error}"),
            Self::DuplicateLogicalPath {
                path,
                first_position,
                duplicate_position,
            } => write!(
                formatter,
                "duplicate logical path {path} at positions {first_position} and {duplicate_position}"
            ),
            Self::LimitExceeded {
                limit,
                maximum,
                actual,
            } => write!(
                formatter,
                "source input limit {limit:?} is {maximum}, attempted {actual}"
            ),
            Self::ArithmeticOverflow => formatter.write_str("source byte count overflow"),
        }
    }
}

impl std::error::Error for SourceBundleError {}

/// Why a requested source span is not inside one bundle source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpanError {
    /// The source ordinal is not present in the bundle.
    UnknownSource(SourceId),
    /// The inclusive start is after the exclusive end.
    Reversed {
        /// Requested start.
        start: ByteOffset,
        /// Requested end.
        end: ByteOffset,
    },
    /// The exclusive end is beyond the source byte length.
    OutOfBounds {
        /// Requested exclusive end.
        end: ByteOffset,
        /// Actual source byte length.
        source_len: u64,
    },
}

impl fmt::Display for SpanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSource(source) => write!(formatter, "unknown source {source}"),
            Self::Reversed { start, end } => write!(
                formatter,
                "source span starts at {} after ending at {}",
                start.value(),
                end.value()
            ),
            Self::OutOfBounds { end, source_len } => write!(
                formatter,
                "source span ends at {} beyond source length {source_len}",
                end.value()
            ),
        }
    }
}

impl std::error::Error for SpanError {}

#[cfg(test)]
mod tests;
