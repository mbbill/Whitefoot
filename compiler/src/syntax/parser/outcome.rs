use crate::syntax::grammar::{
    LookaheadPredicate, Production, RuleOwner, diagnostic_terminal_order,
};
use crate::syntax::terminal::TerminalSet;
use crate::{ByteOffset, SourceId};

use crate::ClassifiedBundle;

use super::DerivationTree;

/// Which explicit parser ceiling was exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseLimit {
    /// Total deterministic parser and diagnostic work units.
    Work,
    /// Simultaneously pending iterative parser tasks.
    Tasks,
    /// Simultaneously open production frames.
    Frames,
    /// Terminal leaves plus production nodes in the private postorder arena.
    Elements,
}

/// Caller-selected implementation ceilings for active-specification parsing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseLimits {
    /// Maximum parser and diagnostic work units.
    pub max_work: u64,
    /// Maximum pending task-stack length.
    pub max_tasks: u64,
    /// Maximum open production-frame depth.
    pub max_frames: u64,
    /// Maximum private postorder element count.
    pub max_elements: u64,
}

/// Which parser allocation could not be extended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseStorage {
    /// Iterative parser or diagnostic tasks.
    Tasks,
    /// Open production frames.
    Frames,
    /// Private postorder derivation elements.
    Elements,
}

/// A toolchain resource failure, never a Whitefoot source verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseResourceFailure {
    /// A configured inclusive ceiling was exceeded.
    LimitExceeded {
        /// Exceeded ceiling category.
        limit: ParseLimit,
        /// Configured inclusive maximum.
        maximum: u64,
        /// Exact requested value beyond that maximum.
        actual: u64,
    },
    /// A host allocation length could not represent the requested count.
    AddressSpaceExceeded {
        /// Affected storage category.
        storage: ParseStorage,
        /// Requested element count.
        requested: u64,
    },
    /// The allocator could not extend already-counted storage.
    StorageUnavailable {
        /// Affected storage category.
        storage: ParseStorage,
        /// Requested element count.
        requested: u64,
    },
}

/// An invocation-envelope failure, never a Whitefoot source verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseInvocationFailure {
    /// PROG-2 requires one nonempty ordered source sequence.
    EmptySourceBundle,
    /// Classified input and syntax data name different numbered specifications.
    SpecificationMismatch,
}

/// An impossible condition in trusted grammar data or parser control flow.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseCompilerFailure {
    /// A generated source-EBNF node identity was absent.
    MissingGrammarNode,
    /// A generated node had a kind or range impossible for its task.
    InvalidGrammarData,
    /// More than one source arm accepted the same complete lookahead.
    PredictiveConflict,
    /// A task completed without its required production frame.
    MissingProductionFrame,
    /// A production completed under a different frame kind.
    ProductionFrameMismatch,
    /// A non-program production derived no terminal extent.
    MissingProductionExtent,
    /// One production attempted to span two source records.
    CrossSourceProduction,
    /// A classified source partition contained a token from another source.
    TokenSourceMismatch,
    /// Diagnostic descent reached a successful arm end after recognition failed.
    DiagnosticReachedSuccessfulEnd,
    /// A checked parser counter overflowed.
    CounterOverflow,
}

/// Exact source-byte coordinate for a pre-tree grammar rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyntaxCoordinate {
    source: SourceId,
    start: ByteOffset,
    end: ByteOffset,
}

impl SyntaxCoordinate {
    pub(crate) const fn new(source: SourceId, start: ByteOffset, end: ByteOffset) -> Self {
        Self { source, start, end }
    }

    /// Returns the bundle-order source identity.
    #[must_use]
    pub const fn source(self) -> SourceId {
        self.source
    }

    /// Returns the inclusive byte start.
    #[must_use]
    pub const fn start(self) -> ByteOffset {
        self.start
    }

    /// Returns the exclusive byte end.
    #[must_use]
    pub const fn end(self) -> ByteOffset {
        self.end
    }
}

/// Numbered language rule selected by exact DIAG-1 grammar attribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyntaxRule {
    /// FORM-1 unknown construct or canonical-form grammar spelling.
    Form1,
    /// FORM-2 exact per-source canonical byte formatting.
    Form2,
    /// FORM-3 name spelling.
    Form3,
    /// FORM-5 numeric-form membership.
    Form5,
    /// GRAM-2 item grammar.
    Gram2,
    /// GRAM-3 type grammar.
    Gram3,
    /// GRAM-4 statement grammar.
    Gram4,
    /// GRAM-5 expression grammar.
    Gram5,
    /// GRAM-9 atom-only flat-computation rule.
    Gram9,
    /// CONST-1 constant-expression grammar.
    Const1,
    /// CONST-2 constant-value grammar.
    Const2,
    /// EFF-1 effect-row grammar.
    Eff1,
}

impl From<RuleOwner> for SyntaxRule {
    fn from(owner: RuleOwner) -> Self {
        match owner {
            RuleOwner::Gram2 => Self::Gram2,
            RuleOwner::Gram3 => Self::Gram3,
            RuleOwner::Gram4 => Self::Gram4,
            RuleOwner::Gram5 => Self::Gram5,
            RuleOwner::Const1 => Self::Const1,
            RuleOwner::Const2 => Self::Const2,
            RuleOwner::Eff1 => Self::Eff1,
        }
    }
}

/// Closed expected-terminal set in approved grammar-first order.
#[derive(Clone, Copy, Debug)]
pub struct ExpectedTerminals {
    terminals: TerminalSet,
    source_end: bool,
}

impl ExpectedTerminals {
    /// Returns whether the set contains one terminal or the end sentinel.
    #[must_use]
    pub const fn contains(self, predicate: LookaheadPredicate) -> bool {
        match predicate {
            LookaheadPredicate::Terminal(terminal) => self.terminals.contains(terminal),
            LookaheadPredicate::SourceEnd => self.source_end,
        }
    }

    /// Returns the distinct expected predicates in exact DIAG-1 order.
    pub fn iter(self) -> impl Iterator<Item = LookaheadPredicate> {
        diagnostic_terminal_order()
            .iter()
            .copied()
            .filter(move |predicate| self.contains(*predicate))
            .chain(self.source_end.then_some(LookaheadPredicate::SourceEnd))
    }

    /// Returns the number of distinct expected predicates.
    #[must_use]
    pub fn len(self) -> usize {
        self.iter().count()
    }

    /// Returns whether no terminal or end sentinel is present.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.terminals.is_empty() && !self.source_end
    }
}

pub(crate) struct ExpectedBuilder {
    terminals: TerminalSet,
    source_end: bool,
}

impl ExpectedBuilder {
    pub(crate) const fn empty() -> Self {
        Self {
            terminals: TerminalSet::empty(),
            source_end: false,
        }
    }

    pub(crate) const fn only_end() -> Self {
        Self {
            terminals: TerminalSet::empty(),
            source_end: true,
        }
    }

    pub(crate) fn insert(&mut self, predicate: LookaheadPredicate) {
        match predicate {
            LookaheadPredicate::Terminal(terminal) => self.terminals.insert(terminal),
            LookaheadPredicate::SourceEnd => self.source_end = true,
        }
    }

    pub(crate) const fn finish(self) -> ExpectedTerminals {
        ExpectedTerminals {
            terminals: self.terminals,
            source_end: self.source_end,
        }
    }
}

/// The first exact grammar-derivation rejection in stage order.
#[derive(Clone, Copy, Debug)]
pub struct SyntaxIssue {
    pub(crate) rule: SyntaxRule,
    pub(crate) coordinate: SyntaxCoordinate,
    pub(crate) expected: ExpectedTerminals,
}

impl SyntaxIssue {
    /// Returns the one numbered language rule selected by DIAG-1.
    #[must_use]
    pub const fn rule(self) -> SyntaxRule {
        self.rule
    }

    /// Returns the exact source-byte failure coordinate.
    #[must_use]
    pub const fn coordinate(self) -> SyntaxCoordinate {
        self.coordinate
    }

    /// Returns the complete expected-terminal set.
    #[must_use]
    pub const fn expected(self) -> ExpectedTerminals {
        self.expected
    }
}

/// Complete private grammar derivation over one classified bundle.
///
/// This value is not a finalized tree, `CanonicalSyntaxUnit`, portable syntax
/// identity, semantic verdict, or compilation acceptance capability.
#[derive(Debug)]
pub struct ParsedBundle<'classified, 'lexed, 'source> {
    pub(crate) classified: &'classified ClassifiedBundle<'lexed, 'source>,
    pub(crate) tree: DerivationTree<'source>,
}

impl<'classified, 'lexed, 'source> ParsedBundle<'classified, 'lexed, 'source> {
    /// Returns the exact classified source input retained by the derivation.
    #[must_use]
    pub const fn classified_bundle(&self) -> &'classified ClassifiedBundle<'lexed, 'source> {
        self.classified
    }

    /// Returns the complete matched terminal-leaf count.
    #[must_use]
    pub const fn terminal_count(&self) -> u64 {
        self.tree.terminal_count
    }

    /// Returns the complete production-node count, including one program root.
    #[must_use]
    pub const fn production_count(&self) -> u64 {
        self.tree.production_count
    }

    /// Returns the private postorder element count.
    #[must_use]
    pub fn element_count(&self) -> usize {
        self.tree.elements.len()
    }

    /// Returns the root's number of flattened top-level item children.
    #[must_use]
    pub fn top_level_item_count(&self) -> Option<u32> {
        match self.tree.elements.last()? {
            super::DerivationElement::Production {
                production: Production::Program,
                child_count,
                ..
            } => Some(*child_count),
            _ => None,
        }
    }
}

/// Failure-atomic result of complete active-specification grammar derivation.
#[derive(Debug)]
pub enum ParseOutcome<'classified, 'lexed, 'source> {
    /// Every source derived completely into one private postorder program tree.
    Complete(ParsedBundle<'classified, 'lexed, 'source>),
    /// The first grammar defect under exact DIAG-1 stage order.
    SourceIssue(SyntaxIssue),
    /// Explicit ceilings or host storage prevented completion or diagnosis.
    ResourceFailure(ParseResourceFailure),
    /// The bound invocation envelope does not select this parser contract.
    InvocationFailure(ParseInvocationFailure),
    /// A trusted grammar-data or parser invariant failed.
    CompilerFailure(ParseCompilerFailure),
}

pub(crate) struct Work {
    used: u64,
    maximum: u64,
}

impl Work {
    pub(crate) const fn new(maximum: u64) -> Self {
        Self { used: 0, maximum }
    }

    pub(crate) fn spend(&mut self, amount: u64) -> Result<(), ParseResourceFailure> {
        let actual = self
            .used
            .checked_add(amount)
            .ok_or(ParseResourceFailure::LimitExceeded {
                limit: ParseLimit::Work,
                maximum: self.maximum,
                actual: u64::MAX,
            })?;
        if actual > self.maximum {
            return Err(ParseResourceFailure::LimitExceeded {
                limit: ParseLimit::Work,
                maximum: self.maximum,
                actual,
            });
        }
        self.used = actual;
        Ok(())
    }
}
