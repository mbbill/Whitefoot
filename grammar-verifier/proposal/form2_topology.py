from __future__ import annotations

from dataclasses import dataclass
import hashlib
import struct

from form2_tree import ProductionNode, source_forest_projection


class TopologyError(RuntimeError):
    pass


@dataclass(frozen=True)
class SourceForestTopology:
    source_ordinal: int
    source_length: int
    terminal_count: int
    items: tuple[ProductionNode, ...]


@dataclass(frozen=True)
class GlobalProgramTopology:
    sources: tuple[SourceForestTopology, ...]

    def __post_init__(self) -> None:
        if not self.sources:
            raise TopologyError("a global program topology requires a nonempty source bundle")
        if tuple(source.source_ordinal for source in self.sources) != tuple(
            range(len(self.sources))
        ):
            raise TopologyError("source ordinals are not complete transport order")
        for source in self.sources:
            _validate_source_forest(source)

    @property
    def program_node_count(self) -> int:
        return 1

    def item_base(self, source_ordinal: int) -> int:
        if source_ordinal < 0 or source_ordinal >= len(self.sources):
            raise TopologyError("source ordinal is outside the global topology")
        return sum(len(source.items) for source in self.sources[:source_ordinal])


@dataclass(frozen=True)
class SeparatorOwner:
    source_ordinal: int
    boundary_index: int
    boundary_kind: str
    location_kind: str
    owner_path: tuple[int, ...]
    owner_production: str | None


def _validate_source_forest(source: SourceForestTopology) -> None:
    if (
        source.source_ordinal < 0
        or source.source_length < 0
        or source.terminal_count < 0
    ):
        raise TopologyError("source topology contains a negative scalar")
    if any(
        node.name == "program"
        for item in source.items
        for node in item.walk()
    ):
        raise TopologyError("a source-local forest contains a program node")
    if any(item.name != "item" for item in source.items):
        raise TopologyError("a source-local forest contains a non-item root")
    direct_owners: list[int] = []

    def validate_node(node: ProductionNode) -> None:
        if node.first_token < 0 or node.last_token < node.first_token:
            raise TopologyError("a source production has an invalid token extent")
        covered = list(node.direct_tokens)
        for child in node.children:
            validate_node(child)
            covered.extend(range(child.first_token, child.last_token + 1))
        if sorted(covered) != list(range(node.first_token, node.last_token + 1)):
            raise TopologyError(
                "a source production does not own its terminal extent exactly once"
            )
        direct_owners.extend(node.direct_tokens)

    covered: list[int] = []
    for item in source.items:
        validate_node(item)
        covered.extend(range(item.first_token, item.last_token + 1))
    if covered != list(range(source.terminal_count)):
        raise TopologyError("source item roots do not cover every terminal exactly once")
    if source.terminal_count == 0 and source.items:
        raise TopologyError("an empty source forest contains an item node")
    if sorted(direct_owners) != list(range(source.terminal_count)):
        raise TopologyError(
            "source productions do not directly own every terminal exactly once"
        )


def bundle_topology_projection(topology: GlobalProgramTopology) -> bytes:
    output = bytearray()
    root_name = b"program"
    output.extend(struct.pack(">I", len(root_name)))
    output.extend(root_name)
    output.extend(struct.pack(">I", len(topology.sources)))
    for source in topology.sources:
        output.extend(
            struct.pack(
                ">QQQI",
                source.source_ordinal,
                0,
                source.source_length,
                len(source.items),
            )
        )
    output.extend(
        struct.pack(
            ">Q", sum(len(source.items) for source in topology.sources)
        )
    )
    for source in topology.sources:
        for source_item_ordinal, item in enumerate(source.items):
            projection = source_forest_projection((item,))
            output.extend(
                struct.pack(
                    ">QIQ",
                    source.source_ordinal,
                    source_item_ordinal,
                    len(projection),
                )
            )
            output.extend(projection)
    if output.count(b"program") != 1:
        raise TopologyError(
            "bundle topology does not contain exactly one global program node"
        )
    return bytes(output)


def bundle_topology_projection_sha256(topology: GlobalProgramTopology) -> str:
    return hashlib.sha256(bundle_topology_projection(topology)).hexdigest()


def _terminal_owners(
    topology: GlobalProgramTopology,
    source: SourceForestTopology,
) -> tuple[dict[int, tuple[tuple[int, ...], str]], dict[int, int]]:
    owner_by_terminal: dict[int, tuple[tuple[int, ...], str]] = {}
    item_by_terminal: dict[int, int] = {}
    base = topology.item_base(source.source_ordinal)

    def visit(node: ProductionNode, path: tuple[int, ...], item_index: int) -> None:
        if node.name == "program":
            raise TopologyError("a source node claims the global program kind")
        for token_index in node.direct_tokens:
            if token_index in owner_by_terminal:
                raise TopologyError("one terminal has two direct production owners")
            owner_by_terminal[token_index] = (path, node.name)
            item_by_terminal[token_index] = item_index
        for child_index, child in enumerate(node.children):
            visit(child, path + (child_index,), item_index)

    for local_index, item in enumerate(source.items):
        visit(item, (base + local_index,), local_index)
    if set(owner_by_terminal) != set(range(source.terminal_count)):
        raise TopologyError("not every terminal has one direct production owner")
    return owner_by_terminal, item_by_terminal


def _common_ancestor(
    left: tuple[int, ...], right: tuple[int, ...]
) -> tuple[int, ...]:
    length = 0
    while length < min(len(left), len(right)) and left[length] == right[length]:
        length += 1
    if length == 0:
        raise TopologyError("an intra-item separator has no production ancestor")
    return left[:length]


def derive_separator_owners(
    topology: GlobalProgramTopology,
    source_ordinal: int,
) -> tuple[SeparatorOwner, ...]:
    if source_ordinal < 0 or source_ordinal >= len(topology.sources):
        raise TopologyError("source ordinal is outside the global topology")
    source = topology.sources[source_ordinal]
    if source.terminal_count == 0:
        return (
            SeparatorOwner(
                source_ordinal,
                0,
                "zero-item",
                "SourceBytes",
                (),
                None,
            ),
        )
    owners, item_by_terminal = _terminal_owners(topology, source)
    nodes_by_path = {path: name for path, name in owners.values()}
    for local_index, item in enumerate(source.items):
        base_path = (topology.item_base(source_ordinal) + local_index,)
        stack = [(item, base_path)]
        while stack:
            node, path = stack.pop()
            nodes_by_path[path] = node.name
            for index in range(len(node.children) - 1, -1, -1):
                stack.append((node.children[index], path + (index,)))

    result = [
        SeparatorOwner(source_ordinal, 0, "source-leading", "SourceBytes", (), None)
    ]
    for right_index in range(1, source.terminal_count):
        left_index = right_index - 1
        if item_by_terminal[left_index] != item_by_terminal[right_index]:
            result.append(
                SeparatorOwner(
                    source_ordinal,
                    right_index,
                    "inter-item",
                    "SourceBytes",
                    (),
                    None,
                )
            )
            continue
        path = _common_ancestor(owners[left_index][0], owners[right_index][0])
        production = nodes_by_path.get(path)
        if production is None or production == "program":
            raise TopologyError("separator owner is absent or global")
        result.append(
            SeparatorOwner(
                source_ordinal,
                right_index,
                "within-item",
                "SourceNode",
                path,
                production,
            )
        )
    result.append(
        SeparatorOwner(
            source_ordinal,
            source.terminal_count,
            "source-final",
            "SourceBytes",
            (),
            None,
        )
    )
    return tuple(result)


def separator_owner_projection(
    topology: GlobalProgramTopology, source_ordinal: int
) -> bytes:
    owners = derive_separator_owners(topology, source_ordinal)
    output = bytearray(struct.pack(">I", len(owners)))
    for owner in owners:
        boundary = owner.boundary_kind.encode("ascii")
        location = owner.location_kind.encode("ascii")
        production = (
            b""
            if owner.owner_production is None
            else owner.owner_production.encode("ascii")
        )
        output.extend(struct.pack(">QQ", owner.source_ordinal, owner.boundary_index))
        for value in (boundary, location):
            output.extend(struct.pack(">I", len(value)))
            output.extend(value)
        output.extend(struct.pack(">I", len(owner.owner_path)))
        for ordinal in owner.owner_path:
            output.extend(struct.pack(">Q", ordinal))
        output.extend(struct.pack(">I", len(production)))
        output.extend(production)
    return bytes(output)


def separator_owner_projection_sha256(
    topology: GlobalProgramTopology, source_ordinal: int
) -> str:
    return hashlib.sha256(
        separator_owner_projection(topology, source_ordinal)
    ).hexdigest()
