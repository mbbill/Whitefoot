use core::fmt;

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
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LogicalPath(String);

impl LogicalPath {
    /// Checks a logical source path without allocating or normalizing it.
    pub(crate) fn validate(value: &str) -> Result<(), LogicalPathError> {
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
        Ok(())
    }

    /// Validates and fallibly owns a logical source path without normalizing it.
    pub fn parse(value: &str) -> Result<Self, LogicalPathError> {
        Self::validate(value)?;
        let requested = u64::try_from(value.len()).map_err(|_| LogicalPathError::LengthOverflow)?;
        let mut owned = String::new();
        owned
            .try_reserve_exact(value.len())
            .map_err(|_| LogicalPathError::StorageUnavailable { requested })?;
        owned.push_str(value);
        Ok(Self(owned))
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    /// The host path length does not fit the portable length domain.
    LengthOverflow,
    /// The allocator could not reserve the exact validated path spelling.
    StorageUnavailable {
        /// Requested path bytes.
        requested: u64,
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
            Self::LengthOverflow => formatter.write_str("logical path length exceeds u64"),
            Self::StorageUnavailable { requested } => {
                write!(
                    formatter,
                    "storage unavailable for logical path of {requested} bytes"
                )
            }
        }
    }
}

impl std::error::Error for LogicalPathError {}

/// One caller-supplied source before bundle validation.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SourceInput<'input> {
    logical_path: &'input str,
    bytes: &'input [u8],
}

impl fmt::Debug for SourceInput<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceInput")
            .field("logical_path", &self.logical_path)
            .field("byte_len", &self.bytes.len())
            .finish()
    }
}

impl<'input> SourceInput<'input> {
    /// Creates a borrowed input view without allocating or normalizing it.
    #[must_use]
    pub const fn new(logical_path: &'input str, bytes: &'input [u8]) -> Self {
        Self {
            logical_path,
            bytes,
        }
    }

    pub(crate) const fn logical_path(&self) -> &'input str {
        self.logical_path
    }

    pub(crate) const fn bytes(&self) -> &'input [u8] {
        self.bytes
    }
}

/// One validated source in a bundle.
#[derive(Eq, PartialEq)]
pub struct SourceFile {
    logical_path: LogicalPath,
    bytes: Vec<u8>,
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

/// One closed, explicitly ordered transport collection of source files.
///
/// File order is caller-supplied source identity, not normative compilation-
/// unit or declaration order. The current lexer never crosses a file boundary.
/// Language meaning for multi-file composition remains separately gated.
#[derive(Debug, Eq, PartialEq)]
pub struct SourceBundle {
    files: Vec<SourceFile>,
    total_bytes: u64,
}

fn try_reserve_exact<T>(
    values: &mut Vec<T>,
    additional: usize,
    limit: SourceLimit,
    requested: u64,
) -> Result<(), SourceBundleError> {
    values
        .try_reserve_exact(additional)
        .map_err(|_| SourceBundleError::StorageUnavailable { limit, requested })
}

fn find_duplicate_paths(
    inputs: &[SourceInput<'_>],
) -> Result<Option<(usize, usize)>, SourceBundleError> {
    let requested =
        u64::try_from(inputs.len()).map_err(|_| SourceBundleError::ArithmeticOverflow)?;
    let mut order = Vec::new();
    try_reserve_exact(&mut order, inputs.len(), SourceLimit::Sources, requested)?;
    order.extend(0..inputs.len());
    order.sort_unstable_by(|left, right| {
        inputs[*left]
            .logical_path
            .cmp(inputs[*right].logical_path)
            .then_with(|| left.cmp(right))
    });

    let mut duplicate = None;
    for pair in order.windows(2) {
        let first = pair[0];
        let next = pair[1];
        if inputs[first].logical_path == inputs[next].logical_path
            && duplicate.is_none_or(|(_, previous)| next < previous)
        {
            duplicate = Some((first, next));
        }
    }
    Ok(duplicate)
}

impl SourceBundle {
    /// Builds a bundle under explicit toolchain resource ceilings.
    pub fn with_limits(
        inputs: &[SourceInput<'_>],
        limits: SourceLimits,
    ) -> Result<Self, SourceBundleError> {
        let source_count = inputs.len();
        let source_count_u64 =
            u64::try_from(source_count).map_err(|_| SourceBundleError::ArithmeticOverflow)?;
        if source_count_u64 > u64::from(limits.max_sources) {
            return Err(SourceBundleError::LimitExceeded {
                limit: SourceLimit::Sources,
                maximum: u64::from(limits.max_sources),
                actual: source_count_u64,
            });
        }

        let mut total_bytes = 0_u64;
        for input in inputs {
            let path_len = u64::try_from(input.logical_path.len())
                .map_err(|_| SourceBundleError::ArithmeticOverflow)?;
            if path_len > limits.max_logical_path_bytes {
                return Err(SourceBundleError::LimitExceeded {
                    limit: SourceLimit::LogicalPathBytes,
                    maximum: limits.max_logical_path_bytes,
                    actual: path_len,
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
            let next_total_bytes = total_bytes
                .checked_add(source_len)
                .ok_or(SourceBundleError::ArithmeticOverflow)?;
            if next_total_bytes > limits.max_total_source_bytes {
                return Err(SourceBundleError::LimitExceeded {
                    limit: SourceLimit::TotalSourceBytes,
                    maximum: limits.max_total_source_bytes,
                    actual: next_total_bytes,
                });
            }
            LogicalPath::validate(input.logical_path).map_err(SourceBundleError::LogicalPath)?;
            total_bytes = next_total_bytes;
        }

        if let Some((first, duplicate)) = find_duplicate_paths(inputs)? {
            let path =
                LogicalPath::parse(inputs[duplicate].logical_path).map_err(
                    |error| match error {
                        LogicalPathError::LengthOverflow => SourceBundleError::ArithmeticOverflow,
                        LogicalPathError::StorageUnavailable { requested } => {
                            SourceBundleError::StorageUnavailable {
                                limit: SourceLimit::LogicalPathBytes,
                                requested,
                            }
                        }
                        error => SourceBundleError::LogicalPath(error),
                    },
                )?;
            let first_position =
                u32::try_from(first).map_err(|_| SourceBundleError::ArithmeticOverflow)?;
            let duplicate_position =
                u32::try_from(duplicate).map_err(|_| SourceBundleError::ArithmeticOverflow)?;
            return Err(SourceBundleError::DuplicateLogicalPath {
                path,
                first_position,
                duplicate_position,
            });
        }

        let mut files = Vec::new();
        try_reserve_exact(
            &mut files,
            source_count,
            SourceLimit::Sources,
            source_count_u64,
        )?;
        for input in inputs {
            let logical_path =
                LogicalPath::parse(input.logical_path).map_err(|error| match error {
                    LogicalPathError::LengthOverflow => SourceBundleError::ArithmeticOverflow,
                    LogicalPathError::StorageUnavailable { requested } => {
                        SourceBundleError::StorageUnavailable {
                            limit: SourceLimit::LogicalPathBytes,
                            requested,
                        }
                    }
                    error => SourceBundleError::LogicalPath(error),
                })?;
            let source_len = u64::try_from(input.bytes.len())
                .map_err(|_| SourceBundleError::ArithmeticOverflow)?;
            let mut bytes = Vec::new();
            try_reserve_exact(
                &mut bytes,
                input.bytes.len(),
                SourceLimit::SourceBytes,
                source_len,
            )?;
            bytes.extend_from_slice(input.bytes);
            files.push(SourceFile {
                logical_path,
                bytes,
                byte_len: source_len,
            });
        }

        Ok(Self { files, total_bytes })
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

    /// Returns source files in caller-supplied transport order.
    #[must_use]
    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    /// Looks up a source by its bundle-order identity.
    #[must_use]
    pub fn file(&self, source: SourceId) -> Option<&SourceFile> {
        usize::try_from(source.ordinal())
            .ok()
            .and_then(|index| self.files.get(index))
    }

    /// Iterates in transport order with derived source identities.
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
#[derive(Debug, Eq, PartialEq)]
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
    /// The allocator could not reserve validated input storage.
    StorageUnavailable {
        /// Storage category that could not be reserved.
        limit: SourceLimit,
        /// Exact requested count or byte length.
        requested: u64,
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
            Self::StorageUnavailable { limit, requested } => write!(
                formatter,
                "storage unavailable for source input {limit:?}, requested {requested}"
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
