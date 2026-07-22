use crate::lexer::Token;
use crate::syntax::grammar::ProductionV0_10;
use crate::syntax::terminal::TerminalPredicateV0_10;
use crate::{ByteOffset, SourceId};

/// One private typed postorder element.
#[derive(Debug)]
pub(crate) enum DerivationElement<'source> {
    Terminal {
        token: Token<'source>,
        predicate: TerminalPredicateV0_10,
    },
    Production {
        production: ProductionV0_10,
        child_count: u32,
        subtree_elements: u64,
        extent: DerivationExtent,
    },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum DerivationExtent {
    Source {
        source: SourceId,
        start: ByteOffset,
        end: ByteOffset,
    },
    BundleRoot,
}

#[derive(Debug)]
pub(crate) struct DerivationTree<'source> {
    pub(crate) elements: Vec<DerivationElement<'source>>,
    pub(crate) terminal_count: u64,
    pub(crate) production_count: u64,
}

#[derive(Debug)]
pub(crate) struct Frame {
    pub(crate) production: ProductionV0_10,
    pub(crate) element_start: usize,
    pub(crate) child_count: u32,
    pub(crate) extent: Option<(SourceId, ByteOffset, ByteOffset)>,
    pub(crate) atom_only: bool,
}
