"""Primitive validators for canonical source-route receipt values."""

from __future__ import annotations

from model import RouteError
from receipt_schema import U64_MAX


def exact_keys(
    value: object, expected: frozenset[str] | set[str], label: str
) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != set(expected):
        raise RouteError(f"receipt {label} is open or incomplete")
    return value


def u64(value: object, label: str) -> int:
    if type(value) is not int or value < 0 or value > U64_MAX:
        raise RouteError(f"receipt {label} is not u64")
    return value


def digest(value: object, label: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise RouteError(f"receipt {label} is not lowercase SHA-256")
    return value


def text(value: object, label: str) -> str:
    if (
        not isinstance(value, str)
        or not value
        or not value.isascii()
        or any(ord(character) < 0x20 or ord(character) > 0x7E for character in value)
    ):
        raise RouteError(f"receipt {label} is not nonempty graphic ASCII text")
    return value


def hex_bytes(value: object, label: str) -> str:
    result = text(value, label)
    if len(result) % 2 or any(
        character not in "0123456789abcdef" for character in result
    ):
        raise RouteError(f"receipt {label} is not lowercase whole-byte hexadecimal")
    return result


def text_list(
    value: object,
    label: str,
    *,
    allow_empty: bool = True,
    require_sorted: bool = True,
) -> list[str]:
    if not isinstance(value, list) or (not allow_empty and not value):
        raise RouteError(f"receipt {label} is not a closed text list")
    result = [text(member, f"{label} member") for member in value]
    if len(result) != len(set(result)) or (
        require_sorted and result != sorted(result)
    ):
        raise RouteError(f"receipt {label} is not in its closed unique order")
    return result


def node_path(value: object, label: str) -> list[int]:
    if not isinstance(value, list):
        raise RouteError(f"receipt {label} is not a node path")
    return [u64(component, f"{label} component") for component in value]
