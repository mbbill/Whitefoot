#!/usr/bin/env python3
"""Assemble the hash-locked UTF-8 parser base prompt without normalization."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys


HERE = Path(__file__).resolve().parent
TASK = HERE / "task.md"
TEACHING = HERE / "teaching-pack.md"
SEPARATOR = b"\n===== BEGIN COMPLETE XLANG WRITER'S PACK =====\n\n"
EXPECTED = {
    "task.md": "9f301b9a0776b855439fb23d403e990ebc5ce8b2add9730c4040de99071732d9",
    "teaching-pack.md": "88917635d551c9352fd788a0c339369e65ad54459ae16157b566fb0e05782672",
}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def assemble() -> tuple[bytes, dict[str, object]]:
    task = TASK.read_bytes()
    teaching = TEACHING.read_bytes()
    observed = {"task.md": sha256(task), "teaching-pack.md": sha256(teaching)}
    if observed != EXPECTED:
        raise RuntimeError(f"prompt component hash mismatch: {observed}")
    prompt = task + SEPARATOR + teaching
    manifest: dict[str, object] = {
        "components": observed,
        "separator_utf8": SEPARATOR.decode("utf-8"),
        "base_prompt_sha256": sha256(prompt),
        "base_prompt_bytes": len(prompt),
    }
    return prompt, manifest


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        prompt, manifest = assemble()
        with args.output.open("xb") as stream:
            stream.write(prompt)
    except (OSError, RuntimeError) as error:
        print(f"prompt assembly failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps(manifest, sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
