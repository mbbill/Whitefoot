use core::fmt;

use whitefoot_contract::{ByteOffset, SourceId, SourceSpan};

/// A validated in-memory identity for one token occurrence.
///
/// The handle is tied to the exact borrowed source file that the lexer
/// inspected. Its source and offsets are portable coordinates only when they
/// are accompanied by a separately verified source binding; they are not a
/// global identifier, digest, or authentication token.
#[derive(Clone, Copy)]
pub struct TokenId<'source> {
    span: SourceSpan<'source>,
}

impl fmt::Debug for TokenId<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TokenId")
            .field("source", &self.source())
            .field("start", &self.start())
            .field("end", &self.end())
            .finish()
    }
}

impl<'source> TokenId<'source> {
    pub(crate) const fn new(span: SourceSpan<'source>) -> Self {
        Self { span }
    }

    /// Returns the bundle-order source coordinate.
    #[must_use]
    pub const fn source(self) -> SourceId {
        self.span.source()
    }

    /// Returns the inclusive starting byte coordinate.
    #[must_use]
    pub const fn start(self) -> ByteOffset {
        self.span.start()
    }

    /// Returns the exclusive ending byte coordinate.
    #[must_use]
    pub const fn end(self) -> ByteOffset {
        self.span.end()
    }

    /// Returns the exact source-bound span that validates this handle.
    #[must_use]
    pub const fn span(self) -> SourceSpan<'source> {
        self.span
    }
}

/// Shape-only lexical category for one Whitefoot token.
///
/// Lowercase words include keywords and dotless operation names. Numeric forms
/// retain their spelling without deciding type, range, or canonical float
/// representation. Those distinctions belong to later stages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenKind {
    /// A FORM-3 lowercase word shape.
    LowerWordForm,
    /// A FORM-3 uppercase word shape.
    UpperWordForm,
    /// An apostrophe-prefixed region-name shape.
    RegionForm,
    /// An at-sign-prefixed label shape.
    LabelForm,
    /// A dotted operation-name shape with one closed mode suffix.
    OperationNameForm,
    /// A decimal numeric candidate retained for later literal checking.
    NumberForm,
    /// A string with a lexically valid raw body and escape structure.
    StringForm,
    /// `(`.
    LeftParen,
    /// `)`.
    RightParen,
    /// `{`.
    LeftBrace,
    /// `}`.
    RightBrace,
    /// `[`.
    LeftBracket,
    /// `]`.
    RightBracket,
    /// `<`.
    LeftAngle,
    /// `>`.
    RightAngle,
    /// `,`.
    Comma,
    /// `:`.
    Colon,
    /// `;`.
    Semicolon,
    /// `.` when it is not part of an operation name.
    Dot,
    /// `=` when it is not part of a fat arrow.
    Equal,
    /// `->`.
    ThinArrow,
    /// `=>`.
    FatArrow,
    /// `&`.
    Ampersand,
}

/// One validated token bound to the exact source bytes it covers.
#[derive(Clone, Copy, Debug)]
pub struct Token<'source> {
    span: SourceSpan<'source>,
    kind: TokenKind,
}

impl<'source> Token<'source> {
    pub(crate) const fn new(span: SourceSpan<'source>, kind: TokenKind) -> Self {
        Self { span, kind }
    }

    /// Returns the source-bound token identity.
    #[must_use]
    pub const fn id(self) -> TokenId<'source> {
        TokenId::new(self.span)
    }

    /// Returns the shape-only lexical category.
    #[must_use]
    pub const fn kind(self) -> TokenKind {
        self.kind
    }

    /// Returns the exact source span and bytes for this token.
    #[must_use]
    pub const fn span(self) -> SourceSpan<'source> {
        self.span
    }
}

/// Kind of byte-preserving trivia between tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TriviaKind {
    /// One maximal nonempty run of ASCII space bytes.
    Spaces,
    /// One LF byte.
    LineFeed,
}

/// One trivia piece bound to the exact source bytes it covers.
#[derive(Clone, Copy, Debug)]
pub struct Trivia<'source> {
    span: SourceSpan<'source>,
    kind: TriviaKind,
}

impl<'source> Trivia<'source> {
    pub(crate) const fn new(span: SourceSpan<'source>, kind: TriviaKind) -> Self {
        Self { span, kind }
    }

    /// Returns the trivia category.
    #[must_use]
    pub const fn kind(self) -> TriviaKind {
        self.kind
    }

    /// Returns the exact source span and bytes for this trivia.
    #[must_use]
    pub const fn span(self) -> SourceSpan<'source> {
        self.span
    }
}

/// One member of the exact lexical partition of a source file.
#[derive(Clone, Copy, Debug)]
pub enum Lexeme<'source> {
    /// A token-shaped byte range.
    Token(Token<'source>),
    /// A retained space or line-feed byte range.
    Trivia(Trivia<'source>),
}

impl<'source> Lexeme<'source> {
    /// Returns the exact source span covered by this partition member.
    #[must_use]
    pub const fn span(self) -> SourceSpan<'source> {
        match self {
            Self::Token(token) => token.span(),
            Self::Trivia(trivia) => trivia.span(),
        }
    }
}
