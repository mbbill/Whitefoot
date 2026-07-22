"""Canonical JSON codec and stable public API for source-route receipts."""

from __future__ import annotations

import json

from measurement import measure
from model import RouteError
from receipt_validation import validate_receipt


def _reject_duplicate_pairs(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise RouteError(f"receipt contains duplicate key: {key}")
        result[key] = value
    return result


def encode_receipt(receipt: dict[str, object]) -> bytes:
    """Validate and encode the sole canonical receipt representation."""

    validate_receipt(receipt)
    try:
        return (
            json.dumps(
                receipt,
                ensure_ascii=True,
                allow_nan=False,
                separators=(",", ":"),
                sort_keys=True,
            )
            + "\n"
        ).encode("ascii")
    except (TypeError, ValueError) as error:
        raise RouteError("receipt contains a noncanonical JSON value") from error


def decode_receipt(encoded: bytes) -> dict[str, object]:
    """Strictly decode, validate, and reproduce canonical receipt bytes."""

    try:
        receipt = json.loads(encoded, object_pairs_hook=_reject_duplicate_pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RouteError("source receipt is not canonical JSON") from error
    if not isinstance(receipt, dict):
        raise RouteError("source receipt top level is not an object")
    validate_receipt(receipt)
    if encode_receipt(receipt) != encoded:
        raise RouteError("source receipt JSON representation is not canonical")
    return receipt


__all__ = ("decode_receipt", "encode_receipt", "measure")
