"""Independent canonical topology, lexical, and scope projections."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Iterator

from model import NodeSite, ParsedSource, RouteError, Scope


@dataclass(frozen=True)
class ProjectionContext:
    """Topology and scope annotations consumed by the role projector."""

    sites: dict[int, NodeSite]
    scopes: tuple[Scope, ...]
    node_scope: dict[int, int]
    node_partitions: dict[int, tuple[str, ...]]
    node_functions: dict[int, str | None]
    node_arms: dict[int, str | None]
    production_nodes: int
    terminals: int
    tree_depth: int


def _extent(source: ParsedSource, node: object) -> tuple[int, int]:
    if node.start < 0 or node.end <= node.start or node.end > len(source.tokens):
        raise RouteError(f"invalid independent production extent: {node.kind}")
    return source.tokens[node.start].start, source.tokens[node.end - 1].end


def _sites(parsed: tuple[ParsedSource, ...]) -> tuple[dict[int, NodeSite], int, int]:
    sites: dict[int, NodeSite] = {}
    item_ordinal = 0
    maximum_depth = 0
    for source in parsed:
        for item in source.forest.items:
            stack = [(item, (item_ordinal,), ())]
            item_ordinal += 1
            while stack:
                node, path, parent_path = stack.pop()
                if id(node) in sites:
                    raise RouteError("independent parser reused a production object")
                start, end = _extent(source, node)
                sites[id(node)] = NodeSite(
                    source.ordinal,
                    path,
                    node,
                    node.kind,
                    start,
                    end,
                    parent_path,
                )
                maximum_depth = max(maximum_depth, len(path))
                for ordinal, child in reversed(tuple(enumerate(node.children))):
                    stack.append((child, path + (ordinal,), path))
    return sites, 1 + len(sites), maximum_depth


def mixed_traversal(
    parsed: tuple[ParsedSource, ...], sites: dict[int, NodeSite]
) -> Iterator[tuple[str, ParsedSource | None, object | None, int | None]]:
    """Yield Program, then every production/direct terminal in mixed DFS order."""

    yield "production", None, None, None

    def walk(source: ParsedSource, node: object):
        yield "production", source, node, None
        children = {child.start: child for child in node.children}
        cursor = node.start
        while cursor < node.end:
            child = children.get(cursor)
            if child is not None:
                yield from walk(source, child)
                cursor = child.end
            else:
                yield "terminal", source, node, cursor
                cursor += 1

    for source in parsed:
        for item in source.forest.items:
            if id(item) not in sites:
                raise RouteError("topology traversal reached an unbound item")
            yield from walk(source, item)


class _ScopeBuilder:
    def __init__(self, sites: dict[int, NodeSite]):
        self.sites = sites
        self.scopes: list[Scope] = [
            Scope(0, "compilation-unit", (), None, 0, None, None, None, (), None, None)
        ]
        self.node_scope: dict[int, int] = {}
        self.node_partitions: dict[int, tuple[str, ...]] = {}
        self.node_functions: dict[int, str | None] = {}
        self.node_arms: dict[int, str | None] = {}

    def add_scope(
        self,
        kind: str,
        node: object,
        parent: int,
        partitions: tuple[str, ...],
        function: str | None,
        arm: str | None,
    ) -> int:
        site = self.sites[id(node)]
        scope_id = len(self.scopes)
        parent_scope = self.scopes[parent]
        self.scopes.append(
            Scope(
                scope_id,
                kind,
                site.path,
                parent,
                parent_scope.depth + 1,
                site.source_ordinal,
                site.byte_start,
                site.byte_end,
                partitions,
                function,
                arm,
            )
        )
        return scope_id

    def mark(
        self,
        node: object,
        scope: int,
        partitions: tuple[str, ...],
        function: str | None,
        arm: str | None,
    ) -> None:
        self.node_scope[id(node)] = scope
        self.node_partitions[id(node)] = partitions
        self.node_functions[id(node)] = function
        self.node_arms[id(node)] = arm

    def generic_child(self, node: object) -> object | None:
        return next((child for child in node.children if child.kind == "generics"), None)

    def walk(
        self,
        node: object,
        scope: int,
        partitions: tuple[str, ...],
        function: str | None,
        arm: str | None,
        top_partition: str,
    ) -> None:
        self.mark(node, scope, partitions, function, arm)
        kind = node.kind

        if kind in {"fn_decl", "struct_decl", "enum_decl", "contract_decl"}:
            generic = self.generic_child(node)
            declaration_partitions = (top_partition,)
            generic_scope = scope
            if generic is not None:
                generic_scope = self.add_scope(
                    "declaration-generic",
                    generic,
                    scope,
                    declaration_partitions,
                    function,
                    arm,
                )
                self.walk(
                    generic,
                    generic_scope,
                    declaration_partitions,
                    function,
                    arm,
                    top_partition,
                )
            if kind == "fn_decl":
                function_key = f"function:{self.sites[id(node)].source_ordinal}:{self.sites[id(node)].path}"
                signature = self.add_scope(
                    "function-region-signature",
                    node,
                    generic_scope,
                    declaration_partitions,
                    function_key,
                    None,
                )
                body = self.add_scope(
                    "function-body-lexical-block",
                    node,
                    signature,
                    declaration_partitions,
                    function_key,
                    None,
                )
                for child in node.children:
                    if child is generic:
                        continue
                    if child.kind == "requires_block":
                        requires = self.add_scope(
                            "requires-lexical-block",
                            child,
                            signature,
                            declaration_partitions,
                            function_key,
                            None,
                        )
                        self.walk(
                            child,
                            requires,
                            declaration_partitions,
                            function_key,
                            None,
                            top_partition,
                        )
                    elif child.kind == "stmt":
                        self.walk(
                            child,
                            body,
                            declaration_partitions,
                            function_key,
                            None,
                            top_partition,
                        )
                    else:
                        self.walk(
                            child,
                            signature,
                            declaration_partitions,
                            function_key,
                            None,
                            top_partition,
                        )
                return
            for child in node.children:
                if child is not generic:
                    self.walk(
                        child,
                        generic_scope,
                        declaration_partitions,
                        function,
                        arm,
                        top_partition,
                    )
            return

        if kind == "fn_sig":
            site = self.sites[id(node)]
            signature_key = f"signature:{site.source_ordinal}:{site.path}"
            signature_partitions = partitions + (signature_key,)
            signature = self.add_scope(
                "contract-signature",
                node,
                scope,
                signature_partitions,
                signature_key,
                None,
            )
            self.mark(
                node,
                signature,
                signature_partitions,
                signature_key,
                None,
            )
            for child in node.children:
                self.walk(
                    child,
                    signature,
                    signature_partitions,
                    signature_key,
                    None,
                    top_partition,
                )
            return

        if kind in {"arm", "loop_stmt", "region_stmt"}:
            site = self.sites[id(node)]
            intermediate_kind = {
                "arm": "arm",
                "loop_stmt": "loop-label",
                "region_stmt": "local-region",
            }[kind]
            next_arm = (
                f"arm:{site.source_ordinal}:{site.path}" if kind == "arm" else arm
            )
            intermediate = self.add_scope(
                intermediate_kind,
                node,
                scope,
                partitions,
                function,
                next_arm,
            )
            body = self.add_scope(
                "nested-lexical-block",
                node,
                intermediate,
                partitions,
                function,
                next_arm,
            )
            self.mark(node, intermediate, partitions, function, next_arm)
            for child in node.children:
                child_scope = body if child.kind == "stmt" else intermediate
                self.walk(
                    child,
                    child_scope,
                    partitions,
                    function,
                    next_arm,
                    top_partition,
                )
            return

        for child in node.children:
            self.walk(child, scope, partitions, function, arm, top_partition)


def build_projection_context(parsed: tuple[ParsedSource, ...]) -> ProjectionContext:
    """Build independent node paths, scopes, owners, and tree counts."""

    sites, production_nodes, tree_depth = _sites(parsed)
    builder = _ScopeBuilder(sites)
    for source in parsed:
        for item in source.forest.items:
            site = sites[id(item)]
            top_partition = f"declaration:{site.source_ordinal}:{site.path}"
            builder.walk(item, 0, (), None, None, top_partition)
    if set(builder.node_scope) != set(sites):
        raise RouteError("scope construction did not annotate every production")
    terminals = sum(len(source.tokens) for source in parsed)
    traversed = tuple(mixed_traversal(parsed, sites))
    if sum(kind == "production" for kind, *_ in traversed) != production_nodes:
        raise RouteError("mixed traversal production count disagrees with topology")
    if sum(kind == "terminal" for kind, *_ in traversed) != terminals:
        raise RouteError("mixed traversal terminal count disagrees with lexer")
    return ProjectionContext(
        sites,
        tuple(builder.scopes),
        builder.node_scope,
        builder.node_partitions,
        builder.node_functions,
        builder.node_arms,
        production_nodes,
        terminals,
        tree_depth,
    )


def lexical_partition_counts(source: bytes, tokens: tuple[object, ...]) -> tuple[int, int]:
    """Return exact token count and maximal token/trivia lexeme count."""

    lexemes = 0
    cursor = 0
    for token in tokens:
        gap = source[cursor : token.start]
        index = 0
        while index < len(gap):
            if gap[index] == 0x20:
                while index < len(gap) and gap[index] == 0x20:
                    index += 1
                lexemes += 1
            elif gap[index] == 0x0A:
                index += 1
                lexemes += 1
            else:
                raise RouteError("canonical source gap is not space/LF trivia")
        lexemes += 1
        cursor = token.end
    gap = source[cursor:]
    index = 0
    while index < len(gap):
        if gap[index] == 0x20:
            while index < len(gap) and gap[index] == 0x20:
                index += 1
            lexemes += 1
        elif gap[index] == 0x0A:
            index += 1
            lexemes += 1
        else:
            raise RouteError("canonical source suffix is not space/LF trivia")
    return len(tokens), lexemes
