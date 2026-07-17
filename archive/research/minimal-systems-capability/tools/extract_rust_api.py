#!/usr/bin/env python3
"""Extract a deterministic public-API inventory from an exact rustdoc tree.

The extractor intentionally reads rendered rustdoc rather than compiler-private
metadata. It inventories public module entries, item declarations, inherent
members, and trait declarations. Trait implementations on concrete types are
not expanded because rustdoc repeats them on every implementer; their protocol
contracts are represented once on the defining trait.

Very large target-intrinsic subtrees may be collapsed by module prefix. A
collapsed subtree still emits one row per reachable module with exact direct
item counts and an entry digest, so compression cannot silently erase a module
or change its contents.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import html
import json
import pathlib
import re
import subprocess
import sys
from dataclasses import dataclass, asdict
from typing import Iterable
from urllib.parse import urldefrag


SCRIPT_SCHEMA = "rustdoc-public-api-v3"
ITEM_CLASSES = {
    "attr",
    "constant",
    "derive",
    "enum",
    "fn",
    "keyword",
    "macro",
    "mod",
    "primitive",
    "static",
    "struct",
    "trait",
    "traitalias",
    "type",
    "union",
}

TAG_RE = re.compile(r"<[^>]+>")
SPACE_RE = re.compile(r"\s+")
ITEM_TABLE_RE = re.compile(r'<dl class="item-table">(.*?)</dl>', re.S)
ITEM_DT_RE = re.compile(r"<dt(?:\s[^>]*)?>(?P<body>.*?)</dt>", re.S)
ITEM_DD_SUFFIX_RE = re.compile(
    r"\s*(?:<dd(?:\s[^>]*)?>(?P<body>.*?)</dd>)?\s*",
    re.S,
)
ANCHOR_RE = re.compile(
    r'<a class="([^"]+)" href="([^"]+)"(?: title="([^"]+)")?>(.*?)</a>',
    re.S,
)
ITEM_DECL_RE = re.compile(r'<pre class="rust item-decl"><code>(.*?)</code></pre>', re.S)
SECTION_RE = re.compile(
    r'<section id="((?:method|tymethod|associatedtype|associatedconstant)\.[^"]+)" '
    r'class="([^"]+)">(.*?)</section>',
    re.S,
)
HEADER_RE = re.compile(r'<h4 class="code-header">(.*?)</h4>', re.S)
SOURCE_RE = re.compile(r'<a class="src(?: rightside)?" href="([^"]+)">Source</a>')
SINCE_RE = re.compile(r'title="Stable since Rust version ([^",]+)')


def plain(fragment: str) -> str:
    text = SPACE_RE.sub(" ", html.unescape(TAG_RE.sub("", fragment))).strip()
    return text.replace(" ⓘ", "").replace("ⓘ", "")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: pathlib.Path) -> str:
    return sha256_bytes(path.read_bytes())


def run_text(argv: list[str]) -> str:
    result = subprocess.run(argv, check=True, text=True, stdout=subprocess.PIPE)
    return result.stdout.strip()


def within(path: pathlib.Path, parent: pathlib.Path) -> bool:
    try:
        path.resolve().relative_to(parent.resolve())
        return True
    except ValueError:
        return False


def stability(fragment: str, fallback: str = "stable") -> str:
    if 'class="stab unstable"' in fragment:
        return "unstable"
    return fallback


def since(fragment: str) -> str:
    match = SINCE_RE.search(fragment)
    return match.group(1) if match else ""


def source_path(page: pathlib.Path, fragment: str, doc_root: pathlib.Path) -> str:
    match = SOURCE_RE.search(fragment)
    if not match:
        return ""
    href = html.unescape(match.group(1))
    path_part, anchor = urldefrag(href)
    line_anchor = re.fullmatch(r"(\d+)(?:-\d+)?", anchor)
    if line_anchor:
        anchor = line_anchor.group(1)
    if path_part.startswith(("http://", "https://")):
        return href
    resolved = (page.parent / path_part).resolve()
    try:
        relative = resolved.relative_to(doc_root.resolve()).as_posix()
    except ValueError:
        relative = resolved.as_posix()
    return f"{relative}#{anchor}" if anchor else relative


def docs_path(page: pathlib.Path, doc_root: pathlib.Path, anchor: str = "") -> str:
    relative = page.resolve().relative_to(doc_root.resolve()).as_posix()
    return f"{relative}#{anchor}" if anchor else relative


def item_path_from_title(kind: str, title: str, module_path: str, name: str) -> str:
    prefix = f"{kind} "
    if title.startswith(prefix):
        return title[len(prefix) :]
    return f"{module_path}::{name}"


def first_item_anchor(dt: str) -> tuple[str, str, str, str]:
    recognized: list[tuple[str, str, str, str]] = []
    for match in ANCHOR_RE.finditer(dt):
        classes = set(match.group(1).split())
        kinds = classes & ITEM_CLASSES
        if not kinds:
            continue
        if len(kinds) != 1:
            raise ValueError(f"ambiguous rustdoc item classes {sorted(kinds)} in {plain(dt)!r}")
        kind = next(iter(kinds))
        recognized.append(
            (
                kind,
                html.unescape(match.group(2)),
                html.unescape(match.group(3) or ""),
                plain(match.group(4)),
            )
        )
    if len(recognized) != 1:
        raise ValueError(
            "rustdoc item-table entry must contain exactly one recognized item anchor: "
            f"{plain(dt)!r}"
        )
    return recognized[0]


@dataclass
class ModuleEntry:
    kind: str
    href: str
    title: str
    name: str
    summary: str
    status: str
    deprecated: bool
    has_description: bool
    raw_digest: str


def module_entries(text: str) -> tuple[list[ModuleEntry], int]:
    entries: list[ModuleEntry] = []
    without_description = 0
    for table in ITEM_TABLE_RE.findall(text):
        item_matches = list(ITEM_DT_RE.finditer(table))
        prefix = table[: item_matches[0].start()] if item_matches else table
        if prefix.strip():
            raise ValueError(f"unparsed rustdoc item-table prefix: {plain(prefix)!r}")
        for index, match in enumerate(item_matches):
            dt = match.group("body")
            next_start = (
                item_matches[index + 1].start()
                if index + 1 < len(item_matches)
                else len(table)
            )
            suffix = table[match.end() : next_start]
            dd_match = ITEM_DD_SUFFIX_RE.fullmatch(suffix)
            if dd_match is None:
                raise ValueError(
                    "rustdoc item-table entry has unparsed or repeated description markup: "
                    f"{plain(suffix)!r}"
                )
            dd_body = dd_match.group("body")
            has_description = dd_body is not None
            dd = dd_body or ""
            if not has_description:
                without_description += 1
            kind, href, title, name = first_item_anchor(dt)
            raw = f"{dt}<dd>{dd}</dd>" if has_description else f"{dt}<no-dd/>"
            entries.append(
                ModuleEntry(
                    kind=kind,
                    href=href,
                    title=title,
                    name=name,
                    summary=plain(dd),
                    status=stability(dt),
                    deprecated='class="stab deprecated"' in dt,
                    has_description=has_description,
                    raw_digest=sha256_bytes(raw.encode("utf-8")),
                )
            )
    entries.sort(key=lambda row: (row.kind, row.title, row.href, row.name))
    return entries, without_description


def member_name(anchor: str, signature: str) -> str:
    prefix, encoded = anchor.split(".", 1)
    class_name = {
        "method": "fn",
        "tymethod": "fn",
        "associatedtype": "associatedtype",
        # Trait constants use `associatedconstant`; inherent constants use
        # `constant` in Rust 1.97 rustdoc headers.
        "associatedconstant": "(?:associatedconstant|constant)",
    }[prefix]
    pattern = re.compile(rf'<a [^>]*class="{class_name}"[^>]*>(.*?)</a>', re.S)
    match = pattern.search(signature)
    return plain(match.group(1)) if match else encoded


def member_kind(anchor: str) -> str:
    prefix = anchor.split(".", 1)[0]
    return {
        "method": "provided_or_inherent_method",
        "tymethod": "required_method",
        "associatedtype": "associated_type",
        "associatedconstant": "associated_constant",
    }[prefix]


def member_regions(text: str, kind: str) -> list[str]:
    if kind == "trait":
        end = text.find('id="implementors-list"')
        return [text[: end if end >= 0 else len(text)]]

    start = text.find('id="implementations"')
    if start < 0:
        return []
    candidates = []
    for marker in (
        'id="trait-implementations"',
        'id="synthetic-implementations"',
        'id="blanket-implementations"',
        'id="deref-methods',
        'id="implementors"',
    ):
        position = text.find(marker, start + 1)
        if position >= 0:
            candidates.append(position)
    end = min(candidates) if candidates else len(text)
    return [text[start:end]]


def page_metadata(page: pathlib.Path, status_hint: str, doc_root: pathlib.Path) -> tuple[str, str, bool, str, str]:
    text = page.read_text(encoding="utf-8")
    declaration = ITEM_DECL_RE.search(text)
    signature = plain(declaration.group(1)) if declaration else ""
    prefix_end = declaration.start() if declaration else min(len(text), 30000)
    prefix = text[:prefix_end]
    item_status = stability(prefix, status_hint) if declaration else status_hint
    return item_status, since(prefix), 'class="stab deprecated"' in prefix, signature, source_path(page, prefix, doc_root)


def page_members(page: pathlib.Path, kind: str, doc_root: pathlib.Path) -> list[dict[str, object]]:
    text = page.read_text(encoding="utf-8")
    members: list[dict[str, object]] = []
    for region in member_regions(text, kind):
        for match in SECTION_RE.finditer(region):
            anchor = match.group(1)
            section_classes = set(match.group(2).split())
            if "trait-impl" in section_classes:
                continue
            anchor_kind = anchor.split(".", 1)[0]
            defining_classes = {
                "method": {"method"},
                "tymethod": {"method"},
                "associatedtype": {"method", "associatedtype"},
                "associatedconstant": {"method", "associatedconstant"},
            }[anchor_kind]
            if section_classes.isdisjoint(defining_classes):
                continue
            body = match.group(3)
            header = HEADER_RE.search(body)
            if not header:
                continue
            signature_html = header.group(1)
            signature = plain(signature_html)
            tail = region[match.end() :]
            local_ends = [
                position
                for marker in (
                    "</summary>",
                    '<div class="docblock">',
                    "<h2 ",
                    '<section id="',
                    '<details class="',
                )
                if (position := tail.find(marker)) >= 0
            ]
            local_end = min(local_ends) if local_ends else len(tail)
            surrounding = region[match.start() : match.end() + local_end]
            members.append(
                {
                    "anchor": anchor,
                    "member_kind": member_kind(anchor),
                    "member_name": member_name(anchor, signature_html),
                    "stability": stability(surrounding),
                    "since": since(body),
                    "deprecated": 'class="stab deprecated"' in surrounding,
                    "caller_safety": "unsafe" if "unsafe fn" in signature else "safe",
                    "signature": signature,
                    "source_path": source_path(page, body, doc_root),
                    "docs_path": docs_path(page, doc_root, anchor),
                }
            )
    members.sort(key=lambda row: (str(row["member_kind"]), str(row["member_name"]), str(row["anchor"])))
    return members


def is_collapsed(module_path: str, prefixes: Iterable[str]) -> bool:
    return any(module_path == prefix or module_path.startswith(prefix + "::") for prefix in prefixes)


def canonical_key(row: dict[str, object]) -> str:
    source = str(row["source_path"])
    member = str(row["member_name"])
    kind = str(row["member_kind"])
    if source:
        # Rustdoc renders macro-generated declarations with `Self` in one crate
        # and the concrete primitive in another. Source anchor + declaration
        # kind + name is stable across those reexports and still separates
        # overloads because rustdoc points distinct declarations at distinct
        # source anchors.
        return f"{source}|{kind}|{member}"
    return f"{row['item_path']}|{kind}|{member}|{row['signature']}"


def write_tsv(path: pathlib.Path, fieldnames: list[str], rows: list[dict[str, object]]) -> None:
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames, delimiter="\t", lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def extract(args: argparse.Namespace) -> None:
    doc_root = pathlib.Path(args.doc_root).resolve()
    source_root = pathlib.Path(args.source_root).resolve()
    output_dir = pathlib.Path(args.output_dir).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    rustc_verbose = run_text([args.rustc, f"+{args.toolchain}", "--version", "--verbose"])
    release_match = re.search(r"^release: (.+)$", rustc_verbose, re.M)
    commit_match = re.search(r"^commit-hash: ([0-9a-f]+)$", rustc_verbose, re.M)
    if not release_match or release_match.group(1) != args.expected_version:
        raise SystemExit(f"rustc release mismatch: expected {args.expected_version}\n{rustc_verbose}")
    if not commit_match or commit_match.group(1) != args.expected_commit:
        raise SystemExit(f"rustc commit mismatch: expected {args.expected_commit}\n{rustc_verbose}")
    source_commit = run_text(["git", "-C", str(source_root), "rev-parse", "HEAD"])
    if source_commit != args.expected_commit:
        raise SystemExit(f"source commit mismatch: expected {args.expected_commit}, got {source_commit}")

    inventory: list[dict[str, object]] = []
    module_rows: list[dict[str, object]] = []
    missing_pages: list[str] = []
    external_modules: list[str] = []
    entries_without_descriptions = 0

    queue: list[tuple[str, str, pathlib.Path]] = [
        (crate, crate, doc_root / crate / "index.html") for crate in ("core", "alloc", "std")
    ]
    seen_modules: set[tuple[str, str]] = set()

    while queue:
        queue.sort(key=lambda item: (item[0], item[1], item[2].as_posix()))
        crate, module_path, page = queue.pop(0)
        identity = (crate, module_path)
        if identity in seen_modules:
            continue
        seen_modules.add(identity)
        if not page.is_file():
            missing_pages.append(f"{module_path}\t{page}")
            continue

        text = page.read_text(encoding="utf-8")
        entries, module_without_descriptions = module_entries(text)
        entries_without_descriptions += module_without_descriptions
        collapsed = is_collapsed(module_path, args.collapse_prefix)
        stable_count = sum(entry.status == "stable" for entry in entries)
        unstable_count = sum(entry.status == "unstable" for entry in entries)
        digest_material = "\n".join(
            f"{entry.kind}\t{entry.title}\t{entry.href}\t{entry.status}\t{entry.raw_digest}"
            for entry in entries
        ).encode("utf-8")
        module_rows.append(
            {
                "crate": crate,
                "module_path": module_path,
                "mode": "collapsed" if collapsed else "detailed",
                "direct_modules": sum(entry.kind == "mod" for entry in entries),
                "direct_items": len(entries),
                "direct_stable_items": stable_count,
                "direct_unstable_items": unstable_count,
                "entry_digest": sha256_bytes(digest_material),
                "docs_path": docs_path(page, doc_root),
            }
        )

        for entry in entries:
            item_path = item_path_from_title(entry.kind, entry.title, module_path, entry.name)
            href_path, _ = urldefrag(entry.href)
            item_page = (page.parent / href_path).resolve() if not href_path.startswith(("http://", "https://")) else None

            if entry.kind == "mod":
                if item_page is not None and within(item_page, doc_root / crate):
                    queue.append((crate, item_path, item_page))
                else:
                    external_modules.append(f"{module_path}\t{item_path}\t{entry.href}")

            if collapsed:
                continue

            item_status = entry.status
            item_since = ""
            item_deprecated = entry.deprecated
            signature = ""
            src = ""
            local_page = item_page if item_page is not None and item_page.is_file() else None
            if local_page is not None and entry.kind != "mod":
                item_status, item_since, page_deprecated, signature, src = page_metadata(
                    local_page, entry.status, doc_root
                )
                item_deprecated = item_deprecated or page_deprecated

            item_row: dict[str, object] = {
                "surface_crate": crate,
                "module_path": module_path,
                "item_path": item_path,
                "item_kind": entry.kind,
                "member_kind": "item",
                "member_name": entry.name,
                "stability": item_status,
                "since": item_since,
                "deprecated": "yes" if item_deprecated else "no",
                "caller_safety": "unsafe" if "unsafe fn" in signature or signature.startswith("unsafe ") else "safe",
                "signature": signature,
                "summary": entry.summary,
                "source_path": src,
                "docs_path": docs_path(local_page, doc_root) if local_page is not None else entry.href,
                "canonical_key": "",
                "duplicate_of": "",
            }
            item_row["canonical_key"] = canonical_key(item_row)
            inventory.append(item_row)

            if local_page is None or entry.kind not in {"struct", "enum", "union", "primitive", "trait"}:
                continue
            for member in page_members(local_page, entry.kind, doc_root):
                row = {
                    "surface_crate": crate,
                    "module_path": module_path,
                    "item_path": item_path,
                    "item_kind": entry.kind,
                    "member_kind": member["member_kind"],
                    "member_name": member["member_name"],
                    "stability": member["stability"],
                    "since": member["since"],
                    "deprecated": "yes" if member["deprecated"] else "no",
                    "caller_safety": member["caller_safety"],
                    "signature": member["signature"],
                    "summary": "",
                    "source_path": member["source_path"],
                    "docs_path": member["docs_path"],
                    "canonical_key": "",
                    "duplicate_of": "",
                }
                row["canonical_key"] = canonical_key(row)
                inventory.append(row)

    inventory.sort(
        key=lambda row: (
            str(row["surface_crate"]),
            str(row["item_path"]),
            str(row["member_kind"]),
            str(row["member_name"]),
            str(row["docs_path"]),
        )
    )
    first_by_key: dict[str, str] = {}
    for row in inventory:
        surface_key = (
            str(row["item_path"])
            if row["member_kind"] == "item"
            else f"{row['item_path']}#{row['member_kind']}:{row['member_name']}"
        )
        key = str(row["canonical_key"])
        if key in first_by_key:
            row["duplicate_of"] = first_by_key[key]
        else:
            first_by_key[key] = surface_key

    module_rows.sort(key=lambda row: (str(row["crate"]), str(row["module_path"])))
    inventory_path = output_dir / "RUST-1.97.0-API-INVENTORY.tsv"
    modules_path = output_dir / "RUST-1.97.0-MODULE-ACCOUNTING.tsv"
    manifest_path = output_dir / "RUST-1.97.0-CENSUS-MANIFEST.json"
    inventory_fields = [
        "surface_crate",
        "module_path",
        "item_path",
        "item_kind",
        "member_kind",
        "member_name",
        "stability",
        "since",
        "deprecated",
        "caller_safety",
        "signature",
        "summary",
        "source_path",
        "docs_path",
        "canonical_key",
        "duplicate_of",
    ]
    module_fields = [
        "crate",
        "module_path",
        "mode",
        "direct_modules",
        "direct_items",
        "direct_stable_items",
        "direct_unstable_items",
        "entry_digest",
        "docs_path",
    ]
    write_tsv(inventory_path, inventory_fields, inventory)
    write_tsv(modules_path, module_fields, module_rows)

    canonical_stable_safe = {
        str(row["canonical_key"])
        for row in inventory
        if row["stability"] == "stable" and row["caller_safety"] == "safe"
    }
    canonical_stable_unsafe = {
        str(row["canonical_key"])
        for row in inventory
        if row["stability"] == "stable" and row["caller_safety"] == "unsafe"
    }
    manifest = {
        "schema": SCRIPT_SCHEMA,
        "rust": {
            "version": args.expected_version,
            "commit": args.expected_commit,
            "rustc_verbose": rustc_verbose.splitlines(),
            "source_commit": source_commit,
        },
        "scope": {
            "crates": ["core", "alloc", "std"],
            "collapse_prefixes": sorted(args.collapse_prefix),
            "trait_impl_policy": "count defining trait declarations; do not expand repeated concrete impl methods",
            "member_section_policy": "count defining methods, deprecated methods, associated types, and associated constants; reject rustdoc trait-impl repeats",
            "stability_policy": "stable caller contracts are the anchor; unstable and unsafe rows remain evidence",
            "item_class_policy": "fail closed on every unrecognized or ambiguous rustdoc item-table entry",
            "item_table_policy": "consume every dt and require zero or one adjacent dd; reject all orphan markup",
            "recognized_item_classes": sorted(ITEM_CLASSES),
        },
        "counts": {
            "inventory_rows": len(inventory),
            "stable_safe_rows": sum(
                row["stability"] == "stable" and row["caller_safety"] == "safe" for row in inventory
            ),
            "stable_unsafe_rows": sum(
                row["stability"] == "stable" and row["caller_safety"] == "unsafe" for row in inventory
            ),
            "unstable_rows": sum(row["stability"] == "unstable" for row in inventory),
            "canonical_stable_safe_declarations": len(canonical_stable_safe),
            "canonical_stable_unsafe_declarations": len(canonical_stable_unsafe),
            "module_rows": len(module_rows),
            "collapsed_module_rows": sum(row["mode"] == "collapsed" for row in module_rows),
            "missing_pages": len(missing_pages),
            "external_module_links": len(external_modules),
            "item_table_entries_without_descriptions": entries_without_descriptions,
        },
        "outputs": {
            inventory_path.name: sha256_file(inventory_path),
            modules_path.name: sha256_file(modules_path),
        },
        "missing_pages": sorted(missing_pages),
        "external_module_links": sorted(external_modules),
    }
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--doc-root", required=True)
    parser.add_argument("--source-root", required=True)
    parser.add_argument("--output-dir", required=True)
    parser.add_argument("--toolchain", default="1.97.0")
    parser.add_argument("--rustc", default="rustc")
    parser.add_argument("--expected-version", default="1.97.0")
    parser.add_argument(
        "--expected-commit",
        default="2d8144b7880597b6e6d3dfd63a9a9efae3f533d3",
    )
    parser.add_argument(
        "--collapse-prefix",
        action="append",
        default=["core::arch", "core::intrinsics"],
        help="Module prefix to retain as counted/digested module rows without item expansion",
    )
    return parser.parse_args(argv)


if __name__ == "__main__":
    extract(parse_args(sys.argv[1:]))
