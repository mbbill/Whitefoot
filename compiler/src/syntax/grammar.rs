//! Immutable active-specification grammar and strong-LL(2) production data.
//!
//! This crate contains no parser and grants no syntax or semantic authority.
//! Its committed generated arrays are checked against the exact numbered
//! specification before the production compiler is built.

use crate::syntax::terminal::TerminalPredicate;
use crate::{ACTIVE_KERNEL_SPEC_HASH, SpecHash};

mod generated;

pub use generated::Production;

/// Exact specification identity owning all data in this crate.
pub const SYNTAX_DATA_SPEC_HASH: SpecHash = ACTIVE_KERNEL_SPEC_HASH;

/// The numbered rule containing a production's unique definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuleOwner {
    /// GRAM-2 item grammar.
    Gram2,
    /// GRAM-3 type grammar.
    Gram3,
    /// GRAM-4 statement grammar.
    Gram4,
    /// GRAM-5 expression grammar.
    Gram5,
    /// CONST-1 constant-expression grammar.
    Const1,
    /// CONST-2 constant-value grammar.
    Const2,
    /// EFF-1 effect-row grammar.
    Eff1,
}

/// One of the five name predicates used by DIAG-1 name-slot attribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamePredicate {
    /// IDENT.
    Identifier,
    /// TYPEID.
    TypeIdentifier,
    /// REGIONID.
    RegionIdentifier,
    /// LABEL.
    Label,
    /// OPNAME.
    OperationName,
}

impl NamePredicate {
    /// Returns the corresponding complete terminal predicate.
    #[must_use]
    pub const fn terminal(self) -> TerminalPredicate {
        match self {
            Self::Identifier => TerminalPredicate::Identifier,
            Self::TypeIdentifier => TerminalPredicate::TypeIdentifier,
            Self::RegionIdentifier => TerminalPredicate::RegionIdentifier,
            Self::Label => TerminalPredicate::Label,
            Self::OperationName => TerminalPredicate::OperationName,
        }
    }
}

/// One position in an exact two-token predictive word.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LookaheadPredicate {
    /// One approved predicate over a formed token.
    Terminal(TerminalPredicate),
    /// The non-token end-of-source sentinel.
    SourceEnd,
}

/// Dense stable identity of one source-EBNF node in active specification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GrammarNodeId(u16);

impl GrammarNodeId {
    pub(crate) const fn new(index: u16) -> Self {
        Self(index)
    }

    /// Returns the dense contract-local index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// The closed runtime form of one source-EBNF node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GrammarNodeKind {
    /// One reference to another normative production.
    Production(Production),
    /// One source terminal occurrence, possibly expanding to two raw tokens.
    TerminalSequence,
    /// An ordered sequence.
    Sequence,
    /// A left-to-right source choice without priority semantics.
    Choice,
    /// A source grouping node.
    Group,
    /// An absent-or-once source node.
    Optional,
    /// A zero-or-more source repetition.
    RepeatZero,
    /// A one-or-more source repetition.
    RepeatOne,
}

/// One immutable source-EBNF node record.
#[derive(Clone, Copy, Debug)]
pub struct GrammarNode {
    kind: GrammarNodeKind,
    range_start: u16,
    range_len: u8,
    decision: Option<u8>,
    atom_only_reference: bool,
}

impl GrammarNode {
    pub(crate) const fn new(
        kind: GrammarNodeKind,
        range_start: u16,
        range_len: u8,
        decision: Option<u8>,
        atom_only_reference: bool,
    ) -> Self {
        Self {
            kind,
            range_start,
            range_len,
            decision,
            atom_only_reference,
        }
    }

    /// Returns this node's closed kind.
    #[must_use]
    pub const fn kind(self) -> GrammarNodeKind {
        self.kind
    }

    /// Returns the child-node slice for a structural node.
    #[must_use]
    pub fn children(self) -> &'static [GrammarNodeId] {
        if !matches!(
            self.kind,
            GrammarNodeKind::Sequence
                | GrammarNodeKind::Choice
                | GrammarNodeKind::Group
                | GrammarNodeKind::Optional
                | GrammarNodeKind::RepeatZero
                | GrammarNodeKind::RepeatOne
        ) {
            return &[];
        }
        let start = self.range_start as usize;
        &generated::GRAMMAR_CHILDREN[start..start + self.range_len as usize]
    }

    /// Returns the terminal sequence for a terminal-bearing node.
    #[must_use]
    pub fn terminals(self) -> &'static [LookaheadPredicate] {
        if self.kind != GrammarNodeKind::TerminalSequence {
            return &[];
        }
        let start = self.range_start as usize;
        &generated::GRAMMAR_TERMINALS[start..start + self.range_len as usize]
    }

    /// Returns this node's predictive decision, if it owns one.
    #[must_use]
    pub fn decision(self) -> Option<&'static Decision> {
        self.decision
            .map(|index| &generated::DECISIONS[index as usize])
    }

    /// Whether this reference enters an atom-only GRAM-9 position.
    #[must_use]
    pub const fn is_atom_only_reference(self) -> bool {
        self.atom_only_reference
    }
}

/// One source-EBNF decision kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionKind {
    /// A written `|` choice.
    Choice,
    /// A written `?` decision.
    Optional,
    /// A written `*` continuation decision.
    Repeat0,
    /// A written `+` continuation decision.
    Repeat1,
}

/// Closed DIAG-1 behavior attached to a predictive frontier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionContext {
    /// No construct-entry or program-leftover override.
    Ordinary,
    /// An `item`, `stmt`, or `requires_entry` entry frontier.
    ConstructEntry,
    /// The per-source `program` item repetition.
    ProgramItems,
}

/// One exact strong-LL(2) decision and its contiguous row range.
#[derive(Clone, Copy, Debug)]
pub struct Decision {
    node: GrammarNodeId,
    production: Production,
    kind: DecisionKind,
    context: DecisionContext,
    arm_count: u8,
    row_start: u16,
    row_len: u16,
}

impl Decision {
    pub(crate) const fn new(
        node: GrammarNodeId,
        production: Production,
        kind: DecisionKind,
        context: DecisionContext,
        arm_count: u8,
        row_start: u16,
        row_len: u16,
    ) -> Self {
        Self {
            node,
            production,
            kind,
            context,
            arm_count,
            row_start,
            row_len,
        }
    }

    /// Returns the stable source-EBNF node identity.
    #[must_use]
    pub const fn node(self) -> GrammarNodeId {
        self.node
    }

    /// Returns the containing production.
    #[must_use]
    pub const fn production(self) -> Production {
        self.production
    }

    /// Returns the source decision kind.
    #[must_use]
    pub const fn kind(self) -> DecisionKind {
        self.kind
    }

    /// Returns closed diagnostic-entry metadata.
    #[must_use]
    pub const fn context(self) -> DecisionContext {
        self.context
    }

    /// Returns the number of source arms.
    #[must_use]
    pub const fn arm_count(self) -> u8 {
        self.arm_count
    }

    /// Returns every provenance-retaining SELECT2 row in source-arm order.
    #[must_use]
    pub fn rows(self) -> &'static [SelectRow] {
        let start = self.row_start as usize;
        &generated::SELECT_ROWS[start..start + self.row_len as usize]
    }
}

/// One SELECT-row position with source-EBNF diagnostic provenance.
#[derive(Clone, Copy, Debug)]
pub struct SelectAtom {
    predicate: LookaheadPredicate,
    provenance: Option<GrammarNodeId>,
    inside_arm: bool,
    transparent_name: Option<NamePredicate>,
    atom_only: bool,
}

impl SelectAtom {
    pub(crate) const fn new(
        predicate: LookaheadPredicate,
        provenance: Option<GrammarNodeId>,
        inside_arm: bool,
        transparent_name: Option<NamePredicate>,
        atom_only: bool,
    ) -> Self {
        Self {
            predicate,
            provenance,
            inside_arm,
            transparent_name,
            atom_only,
        }
    }

    /// Returns the terminal predicate or source-end sentinel.
    #[must_use]
    pub const fn predicate(self) -> LookaheadPredicate {
        self.predicate
    }

    /// Returns the unique source-EBNF terminal occurrence, when one exists.
    #[must_use]
    pub const fn provenance(self) -> Option<GrammarNodeId> {
        self.provenance
    }

    /// Whether this position came from inside the selected source arm.
    #[must_use]
    pub const fn is_inside_arm(self) -> bool {
        self.inside_arm
    }

    /// Returns a transparent mandatory-name endpoint, when one exists.
    #[must_use]
    pub const fn transparent_name(self) -> Option<NamePredicate> {
        self.transparent_name
    }

    /// Whether this predicate is reached through a GRAM-9 atom-only occurrence.
    #[must_use]
    pub const fn is_atom_only(self) -> bool {
        self.atom_only
    }
}

/// One provenance-retaining row for one source arm.
#[derive(Clone, Copy, Debug)]
pub struct SelectRow {
    arm: u8,
    first: u16,
    second: u16,
}

impl SelectRow {
    pub(crate) const fn new(arm: u8, first: u16, second: u16) -> Self {
        Self { arm, first, second }
    }

    /// Returns the source arm index.
    #[must_use]
    pub const fn arm(self) -> u8 {
        self.arm
    }

    /// Returns the requested position, zero or one.
    #[must_use]
    pub fn position(self, index: usize) -> Option<SelectAtom> {
        let atom = match index {
            0 => self.first,
            1 => self.second,
            _ => return None,
        };
        generated::SELECT_ATOMS.get(atom as usize).copied()
    }
}

impl Production {
    /// Returns the root source-EBNF node for this production.
    #[must_use]
    pub fn root(self) -> GrammarNodeId {
        generated::PRODUCTION_ROOTS[self.index()]
    }

    /// Returns the numbered rule containing this production.
    #[must_use]
    pub fn owner(self) -> RuleOwner {
        generated::PRODUCTION_OWNERS[self.index()]
    }
}

/// Returns every production in specification-definition order.
#[must_use]
pub const fn productions() -> &'static [Production] {
    &generated::PRODUCTIONS
}

/// Returns one checked grammar node.
#[must_use]
pub fn grammar_node(node: GrammarNodeId) -> Option<GrammarNode> {
    generated::GRAMMAR_NODES.get(node.index()).copied()
}

/// Returns all terminal predicates in first source-grammar occurrence order.
#[must_use]
pub const fn diagnostic_terminal_order() -> &'static [LookaheadPredicate] {
    &generated::DIAGNOSTIC_ORDER
}

#[cfg(test)]
mod tests;
