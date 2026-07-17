#!/usr/bin/env python3
"""Shared fail-closed loader for SHA-locked literal Python registries."""

from __future__ import annotations

import ast
import hashlib
from pathlib import Path
from typing import AbstractSet


def load_literal_assignments(
    path: Path,
    expected_sha256: str,
    required_names: AbstractSet[str],
) -> dict[str, object]:
    """Return an exact declaration set without executing registry code."""
    data = path.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if digest != expected_sha256:
        raise ValueError(
            "literal registry digest mismatch: "
            f"expected {expected_sha256}, got {digest}"
        )
    try:
        tree = ast.parse(data.decode("utf-8"), filename=str(path), mode="exec")
    except (SyntaxError, UnicodeDecodeError) as error:
        raise ValueError("literal registry is not valid Python data") from error
    if (
        not tree.body
        or not isinstance(tree.body[0], ast.Expr)
        or not isinstance(tree.body[0].value, ast.Constant)
        or not isinstance(tree.body[0].value.value, str)
    ):
        raise ValueError("literal registry lacks its module docstring")

    namespace: dict[str, object] = {}
    for node in tree.body[1:]:
        if (
            not isinstance(node, ast.Assign)
            or len(node.targets) != 1
            or not isinstance(node.targets[0], ast.Name)
        ):
            raise ValueError("literal registry must contain literal assignments only")
        name = node.targets[0].id
        if name not in required_names or name in namespace:
            raise ValueError(f"literal registry has an invalid declaration: {name}")
        try:
            namespace[name] = ast.literal_eval(node.value)
        except (ValueError, TypeError) as error:
            raise ValueError(
                f"literal registry declaration is not literal data: {name}"
            ) from error
    if set(namespace) != set(required_names):
        raise ValueError("literal registry declaration set is not exact")
    return namespace
