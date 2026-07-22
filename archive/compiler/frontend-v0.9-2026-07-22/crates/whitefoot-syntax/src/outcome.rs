use whitefoot_contract::{ByteOffset, SourceBundle, SourceId, SpecHash};
use whitefoot_language_data::TerminalSetV0_9;
use whitefoot_lexer::{LexedBundle, Token, TokenId};

/// Which explicit terminal-classification ceiling was exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalLimit {
    /// Total formed tokens across the ordered source bundle.
    Tokens,
}

/// Caller-selected implementation ceilings for terminal classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalLimits {
    /// Maximum total formed-token count.
    pub max_tokens: u64,
}

/// Which exact output allocation could not be represented or reserved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalStorage {
    /// Classified token records.
    Tokens,
    /// Per-source token-boundary indices.
    SourceBoundaries,
}

/// A toolchain resource failure, never a Whitefoot source verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalResourceFailure {
    /// A configured inclusive ceiling was exceeded.
    LimitExceeded {
        /// Exceeded ceiling category.
        limit: TerminalLimit,
        /// Configured inclusive maximum.
        maximum: u64,
        /// Exact requested value beyond that maximum.
        actual: u64,
    },
    /// An exact output count does not fit the host address space.
    AddressSpaceExceeded {
        /// Affected output allocation.
        storage: TerminalStorage,
        /// Requested element count.
        requested: u64,
    },
    /// The allocator could not reserve the already-counted output.
    StorageUnavailable {
        /// Affected output allocation.
        storage: TerminalStorage,
        /// Requested element count.
        requested: u64,
    },
}

/// An invocation/input failure, never a Whitefoot source verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalInvocationFailure {
    /// The caller selected a different numbered specification.
    SpecificationMismatch {
        /// Exact specification required by this entry point.
        expected: SpecHash,
        /// Exact specification supplied by the caller.
        actual: SpecHash,
    },
}

/// An impossible condition in the trusted terminal-classification boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalCompilerFailure {
    /// A shape token did not satisfy the corresponding approved predicate.
    InvalidFormedToken {
        /// Source containing the impossible token.
        source: SourceId,
        /// Inclusive token start.
        start: ByteOffset,
        /// Exclusive token end.
        end: ByteOffset,
    },
    /// A complete lexical result omitted one source partition.
    MissingSourcePartition {
        /// Source whose partition was absent.
        source: SourceId,
    },
    /// The lexer-reported count disagreed with the token stream.
    TokenCountDisagreement {
        /// Count reported by the complete lexical result.
        expected: u64,
        /// Count observed while classifying.
        actual: u64,
    },
    /// A checked counter overflowed despite the lexical result's bounds.
    CounterOverflow,
}

/// The numbered rule owning a terminal-membership rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalIssueOwner {
    /// FORM-5 literal spelling membership.
    Form5,
}

/// The first formed token that matched no approved terminal predicate.
#[derive(Clone, Copy, Debug)]
pub struct TerminalIssue<'source> {
    pub(crate) token: TokenId<'source>,
    pub(crate) owner: TerminalIssueOwner,
}

impl<'source> TerminalIssue<'source> {
    /// Returns the exact source-bound offending token.
    #[must_use]
    pub const fn token(self) -> TokenId<'source> {
        self.token
    }

    /// Returns the numbered rule owning the rejected spelling.
    #[must_use]
    pub const fn owner(self) -> TerminalIssueOwner {
        self.owner
    }
}

/// One formed token and every approved v0.9 predicate it satisfies.
#[derive(Clone, Copy, Debug)]
pub struct ClassifiedToken<'source> {
    pub(crate) token: Token<'source>,
    pub(crate) terminals: TerminalSetV0_9,
}

impl<'source> ClassifiedToken<'source> {
    /// Returns the original source-bound shape token.
    #[must_use]
    pub const fn token(self) -> Token<'source> {
        self.token
    }

    /// Returns every matching terminal predicate without priority selection.
    #[must_use]
    pub const fn terminals(self) -> TerminalSetV0_9 {
        self.terminals
    }
}

/// A complete terminal-membership projection over one lossless lexical result.
///
/// The original lexical tape remains borrowed so later tree/source audit work
/// can retain trivia and exact byte ownership. This type is not a parse tree,
/// a portable token identity, or an acceptance capability.
#[derive(Debug)]
pub struct ClassifiedBundle<'lexed, 'source> {
    pub(crate) spec: SpecHash,
    pub(crate) lexed: &'lexed LexedBundle<'source>,
    pub(crate) tokens: Vec<ClassifiedToken<'source>>,
    pub(crate) source_offsets: Vec<usize>,
}

impl<'lexed, 'source> ClassifiedBundle<'lexed, 'source> {
    /// Returns the exact numbered specification owning every predicate.
    #[must_use]
    pub const fn spec_hash(&self) -> SpecHash {
        self.spec
    }

    /// Returns the complete lossless lexical input, including trivia.
    #[must_use]
    pub const fn lexed_bundle(&self) -> &'lexed LexedBundle<'source> {
        self.lexed
    }

    /// Returns all classified tokens in source order, then byte order.
    #[must_use]
    pub fn tokens(&self) -> &[ClassifiedToken<'source>] {
        &self.tokens
    }

    /// Returns one source's classified token sequence, including an empty one.
    #[must_use]
    pub fn source_tokens(&self, source: SourceId) -> Option<&[ClassifiedToken<'source>]> {
        let index = usize::try_from(source.ordinal()).ok()?;
        let start = *self.source_offsets.get(index)?;
        let end = *self.source_offsets.get(index.checked_add(1)?)?;
        self.tokens.get(start..end)
    }

    /// Returns the exact source bundle underlying every token handle.
    #[must_use]
    pub const fn source_bundle(&self) -> &'source SourceBundle {
        self.lexed.source_bundle()
    }
}

/// Failure-atomic result of complete v0.9 terminal classification.
#[derive(Debug)]
pub enum TerminalOutcome<'lexed, 'source> {
    /// Every formed token retained at least one approved predicate.
    Complete(ClassifiedBundle<'lexed, 'source>),
    /// One token matched no approved terminal predicate.
    SourceIssue(TerminalIssue<'source>),
    /// Explicit ceilings or host storage prevented completion.
    ResourceFailure(TerminalResourceFailure),
    /// The invocation does not select the exact contract required here.
    InvocationFailure(TerminalInvocationFailure),
    /// A trusted compiler invariant failed.
    CompilerFailure(TerminalCompilerFailure),
}
