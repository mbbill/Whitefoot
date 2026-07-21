from __future__ import annotations

from dataclasses import dataclass
import hashlib

from form2_independent_syntax import (
    IndependentForest,
    IndependentNode,
    source_item_projection,
)


class IndependentTopologyError(RuntimeError):
    pass


@dataclass(frozen=True)
class IndependentSourceTopology:
    source_ordinal: int
    forest: IndependentForest


@dataclass(frozen=True)
class IndependentGlobalTopology:
    sources: tuple[IndependentSourceTopology, ...]

    def __post_init__(self) -> None:
        if not self.sources:
            raise IndependentTopologyError(
                "a global topology requires a nonempty source bundle"
            )
        if tuple(source.source_ordinal for source in self.sources) != tuple(
            range(len(self.sources))
        ):
            raise IndependentTopologyError(
                "source ordinals are not complete transport order"
            )

    def item_base(self, source_ordinal: int) -> int:
        if source_ordinal < 0 or source_ordinal >= len(self.sources):
            raise IndependentTopologyError(
                "source ordinal is outside the global topology"
            )
        return sum(
            len(source.forest.items) for source in self.sources[:source_ordinal]
        )


@dataclass(frozen=True)
class IndependentSeparatorOwner:
    source_ordinal: int
    boundary_index: int
    boundary_kind: str
    location_kind: str
    owner_path: tuple[int, ...]
    owner_production: str | None


def bundle_topology_projection(topology: IndependentGlobalTopology) -> bytes:
    output = bytearray()
    output.extend((7).to_bytes(4, "big"))
    output.extend(b"program")
    output.extend(len(topology.sources).to_bytes(4, "big"))
    for source in topology.sources:
        for scalar in (
            source.source_ordinal,
            0,
            source.forest.source_length,
        ):
            output.extend(scalar.to_bytes(8, "big"))
        output.extend(len(source.forest.items).to_bytes(4, "big"))
    output.extend(
        sum(len(source.forest.items) for source in topology.sources).to_bytes(
            8, "big"
        )
    )
    for source in topology.sources:
        for item_ordinal, item in enumerate(source.forest.items):
            projection = source_item_projection(item)
            output.extend(source.source_ordinal.to_bytes(8, "big"))
            output.extend(item_ordinal.to_bytes(4, "big"))
            output.extend(len(projection).to_bytes(8, "big"))
            output.extend(projection)
    if output.count(b"program") != 1:
        raise IndependentTopologyError(
            "bundle topology does not contain exactly one program node"
        )
    return bytes(output)


def bundle_topology_projection_sha256(
    topology: IndependentGlobalTopology,
) -> str:
    return hashlib.sha256(bundle_topology_projection(topology)).hexdigest()


def _terminal_owners(
    topology: IndependentGlobalTopology,
    source: IndependentSourceTopology,
) -> tuple[dict[int, tuple[tuple[int, ...], str]], dict[int, int]]:
    owners: dict[int, tuple[tuple[int, ...], str]] = {}
    items: dict[int, int] = {}
    base = topology.item_base(source.source_ordinal)

    def visit(node: IndependentNode, path: tuple[int, ...], item_index: int) -> None:
        if node.kind == "program":
            raise IndependentTopologyError(
                "a source node claims the global program kind"
            )
        for token_index in node.direct_token_indices():
            if token_index in owners:
                raise IndependentTopologyError(
                    "one terminal has two direct production owners"
                )
            owners[token_index] = (path, node.kind)
            items[token_index] = item_index
        for child_index, child in enumerate(node.children):
            visit(child, path + (child_index,), item_index)

    for local_index, item in enumerate(source.forest.items):
        visit(item, (base + local_index,), local_index)
    if set(owners) != set(range(source.forest.terminal_count)):
        raise IndependentTopologyError(
            "not every terminal has one direct production owner"
        )
    return owners, items


def _common_ancestor(left: tuple[int, ...], right: tuple[int, ...]) -> tuple[int, ...]:
    length = 0
    while length < min(len(left), len(right)) and left[length] == right[length]:
        length += 1
    if length == 0:
        raise IndependentTopologyError(
            "an intra-item separator has no production ancestor"
        )
    return left[:length]


def derive_separator_owners(
    topology: IndependentGlobalTopology,
    source_ordinal: int,
) -> tuple[IndependentSeparatorOwner, ...]:
    if source_ordinal < 0 or source_ordinal >= len(topology.sources):
        raise IndependentTopologyError(
            "source ordinal is outside the global topology"
        )
    source = topology.sources[source_ordinal]
    if source.forest.terminal_count == 0:
        return (
            IndependentSeparatorOwner(
                source_ordinal, 0, "zero-item", "SourceBytes", (), None
            ),
        )

    terminal_owners, terminal_items = _terminal_owners(topology, source)
    nodes_by_path: dict[tuple[int, ...], str] = {}
    for local_index, item in enumerate(source.forest.items):
        base_path = (topology.item_base(source_ordinal) + local_index,)
        stack = [(item, base_path)]
        while stack:
            node, path = stack.pop()
            nodes_by_path[path] = node.kind
            for child_index in range(len(node.children) - 1, -1, -1):
                stack.append((node.children[child_index], path + (child_index,)))

    result = [
        IndependentSeparatorOwner(
            source_ordinal, 0, "source-leading", "SourceBytes", (), None
        )
    ]
    for right_index in range(1, source.forest.terminal_count):
        left_index = right_index - 1
        if terminal_items[left_index] != terminal_items[right_index]:
            result.append(
                IndependentSeparatorOwner(
                    source_ordinal,
                    right_index,
                    "inter-item",
                    "SourceBytes",
                    (),
                    None,
                )
            )
            continue
        path = _common_ancestor(
            terminal_owners[left_index][0], terminal_owners[right_index][0]
        )
        production = nodes_by_path.get(path)
        if production is None or production == "program":
            raise IndependentTopologyError(
                "separator owner is absent or global"
            )
        result.append(
            IndependentSeparatorOwner(
                source_ordinal,
                right_index,
                "within-item",
                "SourceNode",
                path,
                production,
            )
        )
    result.append(
        IndependentSeparatorOwner(
            source_ordinal,
            source.forest.terminal_count,
            "source-final",
            "SourceBytes",
            (),
            None,
        )
    )
    return tuple(result)


def separator_owner_projection(
    topology: IndependentGlobalTopology, source_ordinal: int
) -> bytes:
    owners = derive_separator_owners(topology, source_ordinal)
    output = bytearray(len(owners).to_bytes(4, "big"))
    for owner in owners:
        output.extend(owner.source_ordinal.to_bytes(8, "big"))
        output.extend(owner.boundary_index.to_bytes(8, "big"))
        for value in (owner.boundary_kind, owner.location_kind):
            encoded = value.encode("ascii")
            output.extend(len(encoded).to_bytes(4, "big"))
            output.extend(encoded)
        output.extend(len(owner.owner_path).to_bytes(4, "big"))
        for ordinal in owner.owner_path:
            output.extend(ordinal.to_bytes(8, "big"))
        production = (
            b""
            if owner.owner_production is None
            else owner.owner_production.encode("ascii")
        )
        output.extend(len(production).to_bytes(4, "big"))
        output.extend(production)
    return bytes(output)


def separator_owner_projection_sha256(
    topology: IndependentGlobalTopology, source_ordinal: int
) -> str:
    return hashlib.sha256(
        separator_owner_projection(topology, source_ordinal)
    ).hexdigest()
