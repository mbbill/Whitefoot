"""Independent FN-8 structural-admission selector."""

from __future__ import annotations

from model import ParsedSource, SelectedDiagnostic
from topology import ProjectionContext


def _only_child(node: object) -> object | None:
    return node.children[0] if len(node.children) == 1 else None


def _entry_shape(entry: object) -> str:
    selected = _only_child(entry)
    if selected is None:
        return "malformed-entry"
    if selected.kind == "doc":
        return "doc"
    if selected.kind != "stmt":
        return "other"
    statement = _only_child(selected)
    if statement is None:
        return "other-statement"
    if statement.kind == "check_stmt":
        return "check"
    if statement.kind != "let_stmt":
        return "other-statement"
    rhs = next(
        (
            child.kind
            for child in statement.children
            if child.kind in {"ordinary_let_rhs", "try_let_rhs", "value_match"}
        ),
        None,
    )
    return "ordinary-let" if rhs == "ordinary_let_rhs" else "nonordinary-let"


def select_fn8(
    parsed: tuple[ParsedSource, ...], context: ProjectionContext
) -> SelectedDiagnostic | None:
    """Select the exact minimum FN-8 issue while scanning every block."""

    candidates: list[SelectedDiagnostic] = []
    for source in parsed:
        for block in (
            node
            for node in source.forest.descendants()
            if node.kind == "requires_block"
        ):
            entries = tuple(
                child for child in block.children if child.kind == "requires_entry"
            )
            selected = None
            reason = ""
            if not entries:
                selected = block
                reason = "missing-final-check-empty"
            else:
                for entry in entries[:-1]:
                    if _entry_shape(entry) != "ordinary-let":
                        selected = entry
                        reason = f"invalid-nonfinal-{_entry_shape(entry)}"
                        break
                if selected is None:
                    final_shape = _entry_shape(entries[-1])
                    if final_shape == "check":
                        continue
                    if final_shape == "ordinary-let":
                        selected = block
                        reason = "missing-final-check-all-let"
                    else:
                        selected = entries[-1]
                        reason = f"invalid-final-{final_shape}"
            site = context.sites[id(selected)]
            candidates.append(
                SelectedDiagnostic(
                    "fn8-admission",
                    "FN-8",
                    reason,
                    source.ordinal,
                    site.path,
                    site.byte_start,
                    site.byte_end,
                    {"shape_kind": reason},
                )
            )
    if not candidates:
        return None
    return min(
        candidates,
        key=lambda issue: (
            issue.source_ordinal,
            issue.byte_start,
            issue.byte_end,
            issue.node_path,
        ),
    )
