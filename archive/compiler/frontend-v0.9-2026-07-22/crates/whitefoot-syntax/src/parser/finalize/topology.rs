use whitefoot_contract::{ByteOffset, SourceId};
use whitefoot_syntax_data::ProductionV0_9;

use super::outcome::BundleSourceExtent;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NodeId(u32);

impl NodeId {
    pub(crate) fn from_index(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }

    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FinalizedExtent {
    Source {
        source: SourceId,
        start: ByteOffset,
        end: ByteOffset,
    },
    BundleRoot,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct NodeRecord {
    pub(crate) production: ProductionV0_9,
    pub(crate) parent: Option<NodeId>,
    pub(crate) child_ordinal: u32,
    pub(crate) child_start: u32,
    pub(crate) child_count: u32,
    pub(crate) tree_depth: u32,
    pub(crate) format_depth: u32,
    pub(crate) top_item: Option<NodeId>,
    pub(crate) first_terminal: u64,
    pub(crate) terminal_count: u64,
    pub(crate) extent: FinalizedExtent,
    pub(crate) body_open: Option<u64>,
    pub(crate) body_close: Option<u64>,
}

impl NodeRecord {
    pub(crate) fn last_terminal(self) -> Option<u64> {
        self.first_terminal
            .checked_add(self.terminal_count.checked_sub(1)?)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TerminalRecord {
    pub(crate) element_index: usize,
    pub(crate) source: SourceId,
    pub(crate) local_ordinal: u64,
    pub(crate) owner: Option<NodeId>,
}

#[derive(Debug)]
pub(crate) struct FinalizedTopology {
    pub(crate) nodes: Vec<NodeRecord>,
    pub(crate) children: Vec<NodeId>,
    pub(crate) terminals: Vec<TerminalRecord>,
    pub(crate) source_extents: Vec<BundleSourceExtent>,
    pub(crate) root: NodeId,
}

impl FinalizedTopology {
    pub(crate) fn node(&self, id: NodeId) -> Option<&NodeRecord> {
        self.nodes.get(id.index())
    }

    pub(crate) fn node_children(&self, id: NodeId) -> Option<&[NodeId]> {
        let node = self.node(id)?;
        let start = usize::try_from(node.child_start).ok()?;
        let count = usize::try_from(node.child_count).ok()?;
        self.children.get(start..start.checked_add(count)?)
    }
}
