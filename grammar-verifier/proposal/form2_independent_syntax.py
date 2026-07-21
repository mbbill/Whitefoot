from __future__ import annotations

from dataclasses import dataclass


PRODUCTION_KINDS = frozenset(
    {
        "arm",
        "atom",
        "atom_list",
        "borrow_expr",
        "break_stmt",
        "call",
        "callee",
        "check_stmt",
        "conform_decl",
        "const",
        "const_decl",
        "construct",
        "contract_decl",
        "cvalue",
        "doc",
        "effect",
        "effects",
        "enum_decl",
        "expr",
        "expr_stmt",
        "field",
        "fieldbind",
        "fieldbind_list",
        "fieldinit",
        "fieldinit_list",
        "fn_bind",
        "fn_decl",
        "fn_sig",
        "generics",
        "give_stmt",
        "gparam",
        "item",
        "law",
        "law_arg",
        "let_stmt",
        "loop_stmt",
        "match_stmt",
        "mode",
        "ordinary_let_rhs",
        "param",
        "param_list",
        "pbase",
        "place",
        "psuffix",
        "region_params",
        "region_stmt",
        "requires_block",
        "requires_entry",
        "return_stmt",
        "rtype",
        "set_stmt",
        "stmt",
        "struct_decl",
        "targ",
        "targs",
        "try_let_rhs",
        "type",
        "value_match",
        "variant",
        "vfield",
        "vfield_list",
    }
)


class IndependentParseError(RuntimeError):
    def __init__(self, token_index: int, byte_offset: int, reason: str):
        super().__init__(f"token {token_index}, byte {byte_offset}: {reason}")
        self.token_index = token_index
        self.byte_offset = byte_offset
        self.reason = reason


class IndependentTreeError(RuntimeError):
    pass


@dataclass(frozen=True)
class IndependentNode:
    kind: str
    start: int
    end: int
    children: tuple[IndependentNode, ...] = ()

    def descendants(self) -> tuple[IndependentNode, ...]:
        output: list[IndependentNode] = []
        stack = [self]
        while stack:
            node = stack.pop()
            output.append(node)
            stack.extend(reversed(node.children))
        return tuple(output)

    def direct_token_indices(self) -> tuple[int, ...]:
        if self.start < 0 or self.end < self.start:
            raise IndependentTreeError(f"invalid extent for {self.kind}")
        cursor = self.start
        direct: list[int] = []
        for child in self.children:
            if child.start < cursor or child.end <= child.start or child.end > self.end:
                raise IndependentTreeError(f"invalid child extent in {self.kind}")
            direct.extend(range(cursor, child.start))
            cursor = child.end
        direct.extend(range(cursor, self.end))
        return tuple(direct)


@dataclass(frozen=True)
class IndependentForest:
    items: tuple[IndependentNode, ...]
    terminal_count: int
    source_length: int

    def __post_init__(self) -> None:
        if self.terminal_count < 0 or self.source_length < 0:
            raise IndependentTreeError("source forest has a negative scalar")
        if any(item.kind != "item" for item in self.items):
            raise IndependentTreeError("source forest contains a non-item root")
        if any(
            node.kind == "program"
            for item in self.items
            for node in item.descendants()
        ):
            raise IndependentTreeError("source forest contains a program node")
        descendants = self.descendants()
        if any(node.kind not in PRODUCTION_KINDS for node in descendants):
            raise IndependentTreeError("source forest contains an unknown production")
        if any(node.start == node.end for node in descendants):
            raise IndependentTreeError(
                "a source production consumes no terminal"
            )
        covered = [
            index
            for item in self.items
            for index in range(item.start, item.end)
        ]
        if covered != list(range(self.terminal_count)):
            raise IndependentTreeError(
                "source item roots do not cover every terminal exactly once"
            )
        direct_owners = [
            index
            for node in descendants
            for index in node.direct_token_indices()
        ]
        if sorted(direct_owners) != list(range(self.terminal_count)):
            raise IndependentTreeError(
                "source productions do not directly own every terminal exactly once"
            )

    def descendants(self) -> tuple[IndependentNode, ...]:
        return tuple(node for item in self.items for node in item.descendants())


def _append_projection(output: bytearray, node: IndependentNode) -> None:
    if node.kind not in PRODUCTION_KINDS:
        raise IndependentTreeError(f"unknown production kind: {node.kind}")
    if node.start == node.end:
        raise IndependentTreeError("a source production consumes no terminal")
    first_token = node.start
    last_token = node.end - 1

    name = node.kind.encode("ascii")
    direct = node.direct_token_indices()
    output.extend(len(name).to_bytes(4, "big"))
    output.extend(name)
    output.extend(first_token.to_bytes(8, "big"))
    output.extend(last_token.to_bytes(8, "big"))
    output.extend(len(direct).to_bytes(4, "big"))
    for token_index in direct:
        output.extend(token_index.to_bytes(8, "big"))
    output.extend(len(node.children).to_bytes(4, "big"))
    for child in node.children:
        _append_projection(output, child)


def source_item_projection(item: IndependentNode) -> bytes:
    projection = bytearray()
    _append_projection(projection, item)
    output = bytearray((1).to_bytes(4, "big"))
    output.extend(len(projection).to_bytes(8, "big"))
    output.extend(projection)
    return bytes(output)


def source_forest_projection(forest: IndependentForest) -> bytes:
    output = bytearray(len(forest.items).to_bytes(4, "big"))
    for item in forest.items:
        projection = bytearray()
        _append_projection(projection, item)
        output.extend(len(projection).to_bytes(8, "big"))
        output.extend(projection)
    if b"program" in output:
        raise IndependentTreeError("source forest projection contains a program node")
    return bytes(output)
