//! Immutable exact-v0.15 grammar and strong-LL(2) production data.
//!
//! This crate contains no parser and grants no syntax or semantic authority.
//! Its committed generated arrays are checked against the exact numbered
//! specification before the production compiler is built.

use crate::syntax::terminal::TerminalPredicateV0_15;
use crate::{KERNEL_SPEC_V0_15_HASH, SpecHash};

mod generated;

pub use generated::ProductionV0_15;

/// Exact specification identity owning all data in this crate.
pub const SYNTAX_DATA_SPEC_V0_15: SpecHash = KERNEL_SPEC_V0_15_HASH;

/// The numbered rule containing a production's unique definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuleOwnerV0_15 {
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
pub enum NamePredicateV0_15 {
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

impl NamePredicateV0_15 {
    /// Returns the corresponding complete terminal predicate.
    #[must_use]
    pub const fn terminal(self) -> TerminalPredicateV0_15 {
        match self {
            Self::Identifier => TerminalPredicateV0_15::Identifier,
            Self::TypeIdentifier => TerminalPredicateV0_15::TypeIdentifier,
            Self::RegionIdentifier => TerminalPredicateV0_15::RegionIdentifier,
            Self::Label => TerminalPredicateV0_15::Label,
            Self::OperationName => TerminalPredicateV0_15::OperationName,
        }
    }
}

/// One position in an exact two-token predictive word.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LookaheadPredicateV0_15 {
    /// One approved predicate over a formed token.
    Terminal(TerminalPredicateV0_15),
    /// The non-token end-of-source sentinel.
    SourceEnd,
}

/// Dense stable identity of one source-EBNF node in exact v0.15.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GrammarNodeIdV0_15(u16);

impl GrammarNodeIdV0_15 {
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
pub enum GrammarNodeKindV0_15 {
    /// One reference to another normative production.
    Production(ProductionV0_15),
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
pub struct GrammarNodeV0_15 {
    kind: GrammarNodeKindV0_15,
    range_start: u16,
    range_len: u8,
    decision: Option<u8>,
    atom_only_reference: bool,
}

impl GrammarNodeV0_15 {
    pub(crate) const fn new(
        kind: GrammarNodeKindV0_15,
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
    pub const fn kind(self) -> GrammarNodeKindV0_15 {
        self.kind
    }

    /// Returns the child-node slice for a structural node.
    #[must_use]
    pub fn children(self) -> &'static [GrammarNodeIdV0_15] {
        if !matches!(
            self.kind,
            GrammarNodeKindV0_15::Sequence
                | GrammarNodeKindV0_15::Choice
                | GrammarNodeKindV0_15::Group
                | GrammarNodeKindV0_15::Optional
                | GrammarNodeKindV0_15::RepeatZero
                | GrammarNodeKindV0_15::RepeatOne
        ) {
            return &[];
        }
        let start = self.range_start as usize;
        &generated::GRAMMAR_CHILDREN[start..start + self.range_len as usize]
    }

    /// Returns the terminal sequence for a terminal-bearing node.
    #[must_use]
    pub fn terminals(self) -> &'static [LookaheadPredicateV0_15] {
        if self.kind != GrammarNodeKindV0_15::TerminalSequence {
            return &[];
        }
        let start = self.range_start as usize;
        &generated::GRAMMAR_TERMINALS[start..start + self.range_len as usize]
    }

    /// Returns this node's predictive decision, if it owns one.
    #[must_use]
    pub fn decision(self) -> Option<&'static DecisionV0_15> {
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
pub enum DecisionKindV0_15 {
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
pub enum DecisionContextV0_15 {
    /// No construct-entry or program-leftover override.
    Ordinary,
    /// An `item`, `stmt`, or `requires_entry` entry frontier.
    ConstructEntry,
    /// The per-source `program` item repetition.
    ProgramItems,
}

/// One exact strong-LL(2) decision and its contiguous row range.
#[derive(Clone, Copy, Debug)]
pub struct DecisionV0_15 {
    node: GrammarNodeIdV0_15,
    production: ProductionV0_15,
    kind: DecisionKindV0_15,
    context: DecisionContextV0_15,
    arm_count: u8,
    row_start: u16,
    row_len: u16,
}

impl DecisionV0_15 {
    pub(crate) const fn new(
        node: GrammarNodeIdV0_15,
        production: ProductionV0_15,
        kind: DecisionKindV0_15,
        context: DecisionContextV0_15,
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
    pub const fn node(self) -> GrammarNodeIdV0_15 {
        self.node
    }

    /// Returns the containing production.
    #[must_use]
    pub const fn production(self) -> ProductionV0_15 {
        self.production
    }

    /// Returns the source decision kind.
    #[must_use]
    pub const fn kind(self) -> DecisionKindV0_15 {
        self.kind
    }

    /// Returns closed diagnostic-entry metadata.
    #[must_use]
    pub const fn context(self) -> DecisionContextV0_15 {
        self.context
    }

    /// Returns the number of source arms.
    #[must_use]
    pub const fn arm_count(self) -> u8 {
        self.arm_count
    }

    /// Returns every provenance-retaining SELECT2 row in source-arm order.
    #[must_use]
    pub fn rows(self) -> &'static [SelectRowV0_15] {
        let start = self.row_start as usize;
        &generated::SELECT_ROWS[start..start + self.row_len as usize]
    }
}

/// One SELECT-row position with source-EBNF diagnostic provenance.
#[derive(Clone, Copy, Debug)]
pub struct SelectAtomV0_15 {
    predicate: LookaheadPredicateV0_15,
    provenance: Option<GrammarNodeIdV0_15>,
    inside_arm: bool,
    transparent_name: Option<NamePredicateV0_15>,
    atom_only: bool,
}

impl SelectAtomV0_15 {
    pub(crate) const fn new(
        predicate: LookaheadPredicateV0_15,
        provenance: Option<GrammarNodeIdV0_15>,
        inside_arm: bool,
        transparent_name: Option<NamePredicateV0_15>,
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
    pub const fn predicate(self) -> LookaheadPredicateV0_15 {
        self.predicate
    }

    /// Returns the unique source-EBNF terminal occurrence, when one exists.
    #[must_use]
    pub const fn provenance(self) -> Option<GrammarNodeIdV0_15> {
        self.provenance
    }

    /// Whether this position came from inside the selected source arm.
    #[must_use]
    pub const fn is_inside_arm(self) -> bool {
        self.inside_arm
    }

    /// Returns a transparent mandatory-name endpoint, when one exists.
    #[must_use]
    pub const fn transparent_name(self) -> Option<NamePredicateV0_15> {
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
pub struct SelectRowV0_15 {
    arm: u8,
    first: u16,
    second: u16,
}

impl SelectRowV0_15 {
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
    pub fn position(self, index: usize) -> Option<SelectAtomV0_15> {
        let atom = match index {
            0 => self.first,
            1 => self.second,
            _ => return None,
        };
        generated::SELECT_ATOMS.get(atom as usize).copied()
    }
}

impl ProductionV0_15 {
    /// Returns the root source-EBNF node for this production.
    #[must_use]
    pub fn root(self) -> GrammarNodeIdV0_15 {
        generated::PRODUCTION_ROOTS[self.index()]
    }

    /// Returns the numbered rule containing this production.
    #[must_use]
    pub fn owner(self) -> RuleOwnerV0_15 {
        generated::PRODUCTION_OWNERS[self.index()]
    }
}

/// Returns every production in specification-definition order.
#[must_use]
pub const fn productions_v0_15() -> &'static [ProductionV0_15] {
    &generated::PRODUCTIONS
}

/// Returns one checked grammar node.
#[must_use]
pub fn grammar_node_v0_15(node: GrammarNodeIdV0_15) -> Option<GrammarNodeV0_15> {
    generated::GRAMMAR_NODES.get(node.index()).copied()
}

/// Returns all terminal predicates in first source-grammar occurrence order.
#[must_use]
pub const fn diagnostic_terminal_order_v0_15() -> &'static [LookaheadPredicateV0_15] {
    &generated::DIAGNOSTIC_ORDER
}

#[cfg(test)]
mod tests;
