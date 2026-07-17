#!/usr/bin/env python3
"""Build the exact Rust 1.97 concrete-trait implementation crosswalk.

The public API inventory intentionally excludes concrete trait implementations.
This targeted extractor closes that evidence boundary for the cross-cutting
TRAIT-* rows and for the sealed-at-the-stable-surface Step set used by the new
range iterators.  It reads only the pinned local Rust 1.97 rustdoc and rust-src
trees, rejects unknown or duplicate identities, and records a digest of the
exact source lines behind every selected implementation.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import html
import pathlib
import re
import subprocess
import sys
from dataclasses import dataclass
from urllib.parse import urldefrag


ROOT = pathlib.Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv"
TOOLCHAIN: pathlib.Path | None = None
DOC_ROOT: pathlib.Path
SOURCE_ROOT: pathlib.Path
RUSTC: pathlib.Path

EXPECTED_RUSTC = (
    "rustc 1.97.0 (2d8144b78 2026-07-07)\n"
    "binary: rustc\n"
    "commit-hash: 2d8144b7880597b6e6d3dfd63a9a9efae3f533d3\n"
    "commit-date: 2026-07-07\n"
    "host: aarch64-apple-darwin\n"
    "release: 1.97.0\n"
    "LLVM version: 22.1.6"
)

FIELDS = [
    "impl_key",
    "selection_family",
    "owning_contract_ids",
    "trait_path",
    "trait_application",
    "implementer",
    "impl_signature",
    "associated_bindings",
    "required_method_shapes",
    "stability",
    "stable_since",
    "stable_surface_reachable",
    "stable_surface_note",
    "ownership_shape",
    "rustdoc_identity",
    "rustdoc_aliases",
    "source_identity",
    "source_snippet_sha256",
]

TYPE_PAGES = (
    "core/primitive.array.html",
    "core/primitive.slice.html",
    "core/primitive.str.html",
    "alloc/boxed/struct.Box.html",
    "alloc/vec/struct.Vec.html",
    "alloc/collections/vec_deque/struct.VecDeque.html",
    "alloc/collections/linked_list/struct.LinkedList.html",
    "alloc/collections/binary_heap/struct.BinaryHeap.html",
    "alloc/collections/btree_map/struct.BTreeMap.html",
    "alloc/collections/btree_set/struct.BTreeSet.html",
    "std/collections/hash_map/struct.HashMap.html",
    "std/collections/hash_set/struct.HashSet.html",
    "alloc/string/struct.String.html",
    "alloc/rc/struct.Rc.html",
    "alloc/rc/struct.Weak.html",
    "core/cell/struct.RefCell.html",
    "core/cell/struct.Ref.html",
    "core/cell/struct.RefMut.html",
)

FAMILY_BY_TRAIT = {
    "IntoIterator": ("INTO_ITERATOR", "TRAIT-INTOITER-01"),
    "Extend": ("EXTEND", "TRAIT-EXTEND-01"),
    "FromIterator": ("FROM_ITERATOR", "TRAIT-COLLECT-01"),
    "Index": ("INDEX", "TRAIT-INDEX-01"),
    "IndexMut": ("INDEX", "TRAIT-INDEX-01"),
    "Deref": ("DEREF", "TRAIT-DEREF-01"),
    "DerefMut": ("DEREF", "TRAIT-DEREF-01"),
    "AsRef": ("BORROW_PROJECTION", "TRAIT-BORROW-01"),
    "AsMut": ("BORROW_PROJECTION", "TRAIT-BORROW-01"),
    "Borrow": ("BORROW_PROJECTION", "TRAIT-BORROW-01"),
    "BorrowMut": ("BORROW_PROJECTION", "TRAIT-BORROW-01"),
    "From": ("CONVERSION", "TRAIT-CONVERT-01"),
    "TryFrom": ("CONVERSION", "TRAIT-CONVERT-01"),
    "Clone": ("CLONE", "TRAIT-CLONE-01"),
    "Default": ("DEFAULT", "TRAIT-DEFAULT-01"),
    "PartialEq": ("COMPARISON_HASH", "TRAIT-CMP-01"),
    "Eq": ("COMPARISON_HASH", "TRAIT-CMP-01"),
    "PartialOrd": ("COMPARISON_HASH", "TRAIT-CMP-01"),
    "Ord": ("COMPARISON_HASH", "TRAIT-CMP-01"),
    "Hash": ("COMPARISON_HASH", "TRAIT-CMP-01"),
    "Drop": ("DROP", "TRAIT-DROP-01"),
}

METHOD_NAMES = {
    "IntoIterator": {"into_iter"},
    "Extend": {"extend"},
    "FromIterator": {"from_iter"},
    "Index": {"index"},
    "IndexMut": {"index_mut"},
    "Deref": {"deref"},
    "DerefMut": {"deref_mut"},
    "AsRef": {"as_ref"},
    "AsMut": {"as_mut"},
    "Borrow": {"borrow"},
    "BorrowMut": {"borrow_mut"},
    "From": {"from"},
    "TryFrom": {"try_from"},
    "Clone": {"clone", "clone_from"},
    "Default": {"default"},
    "PartialEq": {"eq"},
    "Eq": set(),
    "PartialOrd": {"partial_cmp"},
    "Ord": {"cmp"},
    "Hash": {"hash"},
    "Drop": {"drop"},
}

ASSOCIATED_NAMES = {
    "IntoIterator": {"Item", "IntoIter"},
    "Index": {"Output"},
    "IndexMut": set(),
    "Deref": {"Target"},
    "DerefMut": set(),
}

TAG_RE = re.compile(r"<[^>]+>")
SPACE_RE = re.compile(r"\s+")
IMPL_SECTION_RE = re.compile(
    r'<section id="(?P<anchor>impl-[^"]+)" class="impl">(?P<body>.*?)</section>',
    re.S,
)
HEADER_RE = re.compile(r'<h3 class="code-header">(?P<body>.*?)</h3>', re.S)
TRAIT_ANCHOR_RE = re.compile(
    r'<a class="trait"[^>]*title="trait (?P<path>[^"]+)"[^>]*>.*?</a>',
    re.S,
)
SOURCE_RE = re.compile(r'<a class="src(?: rightside)?" href="([^"]+)">Source</a>')
SINCE_RE = re.compile(r'title="Stable since Rust version ([^",]+)')
DETAIL_TOKEN_RE = re.compile(r"<details\b|</details>")
ASSOCIATED_SECTION_RE = re.compile(
    r'<section id="associatedtype\.[^"]+" class="[^"]*trait-impl[^"]*">'
    r'.*?<h4 class="code-header">(?P<header>.*?)</h4>.*?</section>',
    re.S,
)
METHOD_SECTION_RE = re.compile(
    r'<section id="method\.(?P<name>[^"]+)" class="[^"]*trait-impl[^"]*">'
    r'.*?<h4 class="code-header">(?P<header>.*?)</h4>.*?</section>',
    re.S,
)


def plain(fragment: str) -> str:
    return SPACE_RE.sub(" ", html.unescape(TAG_RE.sub("", fragment))).strip()


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def rustc_identity() -> str:
    result = subprocess.run(
        [str(RUSTC), "-vV"],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return result.stdout.strip()


def configure_toolchain(explicit_sysroot: pathlib.Path | None = None) -> None:
    global TOOLCHAIN, DOC_ROOT, SOURCE_ROOT, RUSTC
    if explicit_sysroot is None:
        result = subprocess.run(
            ["rustc", "+1.97.0", "--print", "sysroot"],
            check=True,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        sysroot = pathlib.Path(result.stdout.strip())
    else:
        sysroot = explicit_sysroot
    TOOLCHAIN = sysroot.resolve()
    DOC_ROOT = TOOLCHAIN / "share/doc/rust/html"
    SOURCE_ROOT = TOOLCHAIN / "lib/rustlib/src/rust/library"
    RUSTC = TOOLCHAIN / "bin/rustc"
    for required in (RUSTC, DOC_ROOT, SOURCE_ROOT):
        if not required.exists():
            raise ValueError(f"pinned Rust 1.97 component is missing under sysroot: {required}")


def details_region(text: str, section_start: int, lower_bound: int) -> str:
    start = text.rfind('<details class="toggle implementors-toggle"', lower_bound, section_start)
    if start < 0:
        raise ValueError("trait implementation section is not inside an implementor detail")
    depth = 0
    for token in DETAIL_TOKEN_RE.finditer(text, start):
        if token.group(0).startswith("<details"):
            depth += 1
        else:
            depth -= 1
            if depth == 0:
                end = token.end()
                if not start <= section_start < end:
                    break
                return text[start:end]
    raise ValueError("unbalanced rustdoc implementation details")


def source_identity(page: pathlib.Path, section: str) -> tuple[str, str]:
    match = SOURCE_RE.search(section)
    if match is None:
        raise ValueError("implementation has no rustdoc source link")
    href, anchor = urldefrag(html.unescape(match.group(1)))
    resolved = (page.parent / href).resolve()
    relative = resolved.relative_to(DOC_ROOT.resolve()).as_posix()
    line_match = re.fullmatch(r"(?P<start>\d+)(?:-(?P<end>\d+))?", anchor)
    if line_match is None:
        raise ValueError(f"implementation source anchor is not an exact line range: {anchor!r}")
    start = int(line_match.group("start"))
    end = int(line_match.group("end") or start)
    source_match = re.fullmatch(r"src/([^/]+)/(.+)\.html", relative)
    if source_match is None:
        raise ValueError(f"implementation source is outside pinned rust-src: {relative!r}")
    source_file = SOURCE_ROOT / source_match.group(1) / "src" / source_match.group(2)
    if not source_file.is_file():
        raise ValueError(f"missing pinned rust-src file {source_file}")
    lines = source_file.read_text(encoding="utf-8").splitlines(keepends=True)
    if start < 1 or end < start or end > len(lines):
        raise ValueError(f"invalid source line range {start}-{end} for {source_file}")
    snippet = "".join(lines[start - 1 : end])
    identity = f"library/{source_match.group(1)}/src/{source_match.group(2)}:{start}-{end}"
    return identity, sha256_text(snippet)


def type_root(type_shape: str) -> str:
    shape = type_shape.split("where", 1)[0].strip().rstrip(",")
    shape = re.sub(r"^&(?:'[A-Za-z_][A-Za-z0-9_]*\s+)?", "", shape)
    shape = re.sub(r"^mut\s+", "", shape)
    if shape.startswith("["):
        close = shape.find("]")
        if close < 0:
            return "UNKNOWN"
        return "array" if ";" in shape[: close + 1] else "slice"
    if shape.startswith("str"):
        return "str"
    match = re.match(r"([A-Za-z_][A-Za-z0-9_]*)", shape)
    return match.group(1) if match else "UNKNOWN"


def trait_parameter_root(trait_name: str, application: str) -> str:
    suffix = application[len(trait_name) :].strip()
    if not suffix:
        return "SELF"
    if not suffix.startswith("<") or not suffix.endswith(">"):
        return "UNKNOWN"
    return type_root(suffix[1:-1])


def supported_wrapper_shape(implementer: str) -> bool:
    return not any(
        forbidden in implementer
        for forbidden in ("ByteStr", "CStr", "dyn Error", "RawWaker", "LocalWaker")
    )


def selected(trait_name: str, application: str, implementer: str) -> bool:
    root = type_root(implementer)
    if trait_name == "IntoIterator":
        return root in {
            "array", "slice", "Vec", "VecDeque", "LinkedList", "BinaryHeap",
            "BTreeMap", "BTreeSet", "HashMap", "HashSet",
        }
    if trait_name == "Extend":
        parameter = trait_parameter_root(trait_name, application)
        return root in {
            "Vec", "VecDeque", "LinkedList", "BinaryHeap", "BTreeMap",
            "BTreeSet", "HashMap", "HashSet", "String",
        } and parameter in {"T", "UNKNOWN", "char", "str", "Box", "Cow", "String"}
    if trait_name == "FromIterator":
        parameter = trait_parameter_root(trait_name, application)
        return root in {
            "Box", "Vec", "VecDeque", "LinkedList", "BinaryHeap", "BTreeMap",
            "BTreeSet", "HashMap", "HashSet", "String", "Rc",
        } and parameter in {"T", "UNKNOWN", "char", "str", "Box", "Cow", "String"} \
            and supported_wrapper_shape(implementer)
    if trait_name in {"Index", "IndexMut"}:
        roots = {"array", "slice", "Vec", "VecDeque", "String", "str"}
        if trait_name == "Index":
            roots |= {"BTreeMap", "HashMap"}
        return root in roots
    if trait_name in {"Deref", "DerefMut"}:
        return root in {"Box", "Vec", "String", "Rc", "Ref", "RefMut"}
    if trait_name in {"AsRef", "AsMut", "Borrow", "BorrowMut"}:
        target = trait_parameter_root(trait_name, application)
        return (
            root in {"array", "slice", "str", "Box", "Vec", "String", "Rc"}
            and target in {"T", "array", "slice", "str"}
            and supported_wrapper_shape(implementer)
        )
    if trait_name in {"From", "TryFrom"}:
        allowed = {"array", "slice", "str", "Box", "Vec", "String", "Rc"}
        source = trait_parameter_root(trait_name, application)
        return (
            root in allowed
            and source in allowed | {"T"}
            and supported_wrapper_shape(implementer)
            and supported_wrapper_shape(application)
        )
    if trait_name == "Clone":
        return root in {
            "array", "Box", "Vec", "VecDeque", "LinkedList", "BinaryHeap",
            "BTreeMap", "BTreeSet", "HashMap", "HashSet", "String", "Rc",
            "Weak", "RefCell",
        } and supported_wrapper_shape(implementer)
    if trait_name == "Default":
        return root in {
            "array", "slice", "str", "Box", "Vec", "VecDeque", "LinkedList",
            "BinaryHeap", "BTreeMap", "BTreeSet", "HashMap", "HashSet", "String",
            "Rc", "Weak", "RefCell",
        } and supported_wrapper_shape(implementer)
    if trait_name in {"PartialEq", "Eq", "PartialOrd", "Ord", "Hash"}:
        allowed = {
            "array", "slice", "str", "Vec", "VecDeque", "LinkedList",
            "BinaryHeap", "BTreeMap", "BTreeSet", "HashMap", "HashSet", "String",
            "Rc", "Weak",
        }
        rhs = trait_parameter_root(trait_name, application)
        return root in allowed and (rhs == "SELF" or rhs in allowed)
    if trait_name == "Drop":
        return root in {
            "Box", "Vec", "VecDeque", "LinkedList", "BinaryHeap", "BTreeMap",
            "BTreeSet", "HashMap", "HashSet", "String", "Rc", "Weak", "RefCell",
        }
    return False


@dataclass
class Candidate:
    selection_family: str
    owning_contract_ids: str
    trait_path: str
    trait_application: str
    implementer: str
    impl_signature: str
    associated_bindings: str
    required_method_shapes: str
    stability: str
    stable_since: str
    stable_surface_reachable: str
    stable_surface_note: str
    ownership_shape: str
    rustdoc_identity: str
    source_identity: str
    source_snippet_sha256: str


def parse_type_page(relative: str) -> list[Candidate]:
    page = DOC_ROOT / relative
    text = page.read_text(encoding="utf-8")
    region_start = text.find('id="trait-implementations"')
    if region_start < 0:
        raise ValueError(f"{relative} has no direct trait implementation region")
    ends = [
        position
        for marker in (
            'id="synthetic-implementations"',
            'id="auto-trait-implementations"',
            'id="blanket-implementations"',
        )
        if (position := text.find(marker, region_start + 1)) >= 0
    ]
    region_end = min(ends) if ends else len(text)
    region = text[region_start:region_end]
    output: list[Candidate] = []
    for section_match in IMPL_SECTION_RE.finditer(region):
        section = section_match.group(0)
        header_match = HEADER_RE.search(section)
        if header_match is None:
            raise ValueError(f"{relative}#{section_match.group('anchor')} has no impl header")
        raw_header = header_match.group("body")
        for_position = raw_header.rfind(" for ")
        if for_position < 0:
            continue
        trait_anchors = [
            match for match in TRAIT_ANCHOR_RE.finditer(raw_header)
            if match.end() <= for_position
        ]
        if not trait_anchors:
            continue
        trait_anchor = trait_anchors[-1]
        trait_path = trait_anchor.group("path")
        trait_name = trait_path.rsplit("::", 1)[-1]
        if trait_name not in FAMILY_BY_TRAIT:
            continue
        application = plain(raw_header[trait_anchor.start() : for_position])
        implementer = plain(raw_header[for_position + len(" for ") :])
        if not selected(trait_name, application, implementer):
            continue
        absolute_start = region_start + section_match.start()
        if not METHOD_NAMES[trait_name] and not ASSOCIATED_NAMES.get(trait_name, set()):
            outer = section
        else:
            try:
                outer = details_region(text, absolute_start, region_start)
            except ValueError as error:
                raise ValueError(
                    f"{relative}#{section_match.group('anchor')}: {error}"
                ) from error
        header_end = outer.find('<div class="impl-items">')
        implementation_header = outer[: header_end if header_end >= 0 else len(outer)]
        if 'class="stab unstable"' in implementation_header:
            stability = "unstable"
            stable_since = "NONE"
        else:
            since_match = SINCE_RE.search(implementation_header)
            if since_match is None:
                raise ValueError(
                    f"selected impl has unknown stability: {relative}#"
                    f"{section_match.group('anchor')}"
                )
            stability = "stable"
            stable_since = since_match.group(1)
        if stability != "stable":
            continue
        associated = []
        expected_associated = ASSOCIATED_NAMES.get(trait_name, set())
        for match in ASSOCIATED_SECTION_RE.finditer(outer):
            binding = plain(match.group("header"))
            name_match = re.match(r"type\s+([A-Za-z_][A-Za-z0-9_]*)\s*=", binding)
            if name_match and name_match.group(1) in expected_associated:
                associated.append(binding)
        methods = []
        for match in METHOD_SECTION_RE.finditer(outer):
            method_name = re.sub(r"-\d+$", "", match.group("name"))
            if method_name in METHOD_NAMES[trait_name]:
                methods.append(plain(match.group("header")))
        if expected_associated and len(associated) != len(expected_associated):
            raise ValueError(
                f"{relative}#{section_match.group('anchor')} has incomplete associated bindings: "
                f"expected {sorted(expected_associated)}, got {associated}"
            )
        if METHOD_NAMES[trait_name] and not methods:
            raise ValueError(
                f"{relative}#{section_match.group('anchor')} has no selected required method shape"
            )
        source, source_digest = source_identity(page, section)
        family, contract = FAMILY_BY_TRAIT[trait_name]
        output.append(
            Candidate(
                selection_family=family,
                owning_contract_ids=contract,
                trait_path=trait_path,
                trait_application=application,
                implementer=implementer,
                impl_signature=plain(raw_header),
                associated_bindings=" | ".join(sorted(associated)) or "NONE",
                required_method_shapes=" | ".join(sorted(set(methods))) or "NONE",
                stability=stability,
                stable_since=stable_since,
                stable_surface_reachable="YES",
                stable_surface_note="stable direct implementation on a stable selected surface",
                ownership_shape="NOT_CLASSIFIED_BY_THIS_CROSSWALK",
                rustdoc_identity=f"{relative}#{section_match.group('anchor')}",
                source_identity=source,
                source_snippet_sha256=source_digest,
            )
        )
    return output


def parse_step_implementors() -> list[Candidate]:
    relative = "core/iter/trait.Step.html"
    page = DOC_ROOT / relative
    text = page.read_text(encoding="utf-8")
    start = text.find('id="implementors-list"')
    if start < 0:
        raise ValueError("Step rustdoc page has no implementor list")
    output: list[Candidate] = []
    for match in IMPL_SECTION_RE.finditer(text[start:]):
        section = match.group(0)
        header_match = HEADER_RE.search(section)
        if header_match is None:
            raise ValueError(f"Step implementor {match.group('anchor')} has no header")
        raw_header = header_match.group("body")
        trait_match = TRAIT_ANCHOR_RE.search(raw_header)
        if trait_match is None or trait_match.group("path") != "core::iter::Step":
            raise ValueError(f"unexpected Step implementor header {plain(raw_header)!r}")
        for_position = raw_header.rfind(" for ")
        application = plain(raw_header[trait_match.start() : for_position])
        implementer = plain(raw_header[for_position + len(" for ") :])
        source, source_digest = source_identity(page, section)
        output.append(
            Candidate(
                selection_family="RANGE_STEP",
                owning_contract_ids=(
                    "RANGE-ITER-HALFOPEN-01,RANGE-ITER-FROM-01,"
                    "RANGE-ITER-INCLUSIVE-01"
                ),
                trait_path="core::iter::Step",
                trait_application=application,
                implementer=implementer,
                impl_signature=plain(raw_header),
                associated_bindings="NONE",
                required_method_shapes=(
                    "steps_between(&Self, &Self) -> (usize, Option<usize>) | "
                    "forward_checked(Self, usize) -> Option<Self> | "
                    "backward_checked(Self, usize) -> Option<Self>"
                ),
                stability="unstable-sealed-stable-surface",
                stable_since="NONE",
                stable_surface_reachable="NO" if implementer == "Char" else "YES",
                stable_surface_note=(
                    "core::ascii::Char is unstable in Rust 1.97.0"
                    if implementer == "Char"
                    else "stable implementer reaches the private unstable Step implementation through the stable Range iterator surface"
                ),
                ownership_shape="COPY_BORROW_FREE",
                rustdoc_identity=f"{relative}#{match.group('anchor')}",
                source_identity=source,
                source_snippet_sha256=source_digest,
            )
        )
    return output


def build_rows(explicit_sysroot: pathlib.Path | None = None) -> list[dict[str, str]]:
    configure_toolchain(explicit_sysroot)
    if rustc_identity() != EXPECTED_RUSTC:
        raise ValueError("local Rust compiler identity is not the pinned 1.97.0 toolchain")
    candidates = [candidate for page in TYPE_PAGES for candidate in parse_type_page(page)]
    candidates.extend(parse_step_implementors())
    grouped: dict[tuple[str, str, str, str], list[Candidate]] = {}
    for candidate in candidates:
        key = (
            candidate.selection_family,
            candidate.trait_path,
            candidate.impl_signature,
            candidate.source_identity,
        )
        grouped.setdefault(key, []).append(candidate)
    rows: list[dict[str, str]] = []
    for key, duplicates in grouped.items():
        first = duplicates[0]
        comparable = {
            (
                row.owning_contract_ids,
                row.trait_application,
                row.implementer,
                row.associated_bindings,
                row.required_method_shapes,
                row.stability,
                row.stable_since,
                row.stable_surface_reachable,
                row.stable_surface_note,
                row.ownership_shape,
                row.source_snippet_sha256,
            )
            for row in duplicates
        }
        if len(comparable) != 1:
            raise ValueError(f"duplicate rustdoc renderings disagree for {key}")
        identities = sorted({row.rustdoc_identity for row in duplicates})
        impl_key = sha256_text("\0".join(key))
        rows.append(
            {
                "impl_key": impl_key,
                "selection_family": first.selection_family,
                "owning_contract_ids": first.owning_contract_ids,
                "trait_path": first.trait_path,
                "trait_application": first.trait_application,
                "implementer": first.implementer,
                "impl_signature": first.impl_signature,
                "associated_bindings": first.associated_bindings,
                "required_method_shapes": first.required_method_shapes,
                "stability": first.stability,
                "stable_since": first.stable_since,
                "stable_surface_reachable": first.stable_surface_reachable,
                "stable_surface_note": first.stable_surface_note,
                "ownership_shape": first.ownership_shape,
                "rustdoc_identity": identities[0],
                "rustdoc_aliases": ",".join(identities[1:]) or "NONE",
                "source_identity": first.source_identity,
                "source_snippet_sha256": first.source_snippet_sha256,
            }
        )
    rows.sort(
        key=lambda row: (
            row["selection_family"], row["trait_path"], row["impl_signature"],
            row["source_identity"],
        )
    )
    keys = [row["impl_key"] for row in rows]
    if len(keys) != len(set(keys)):
        raise ValueError("duplicate implementation key after canonicalization")
    return rows


def render(rows: list[dict[str, str]]) -> str:
    import io

    buffer = io.StringIO(newline="")
    writer = csv.DictWriter(buffer, fieldnames=FIELDS, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return buffer.getvalue()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--sysroot", type=pathlib.Path)
    args = parser.parse_args()
    try:
        content = render(build_rows(args.sysroot))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"trait implementation crosswalk build failed: {error}") from error
    if args.check:
        if not OUTPUT.is_file() or OUTPUT.read_text(encoding="utf-8") != content:
            raise SystemExit("trait implementation crosswalk build failed: checked-in TSV is stale")
        print("trait implementation crosswalk build: PASS")
        return
    OUTPUT.write_text(content, encoding="utf-8")
    print(f"wrote {OUTPUT} with {content.count(chr(10)) - 1} implementation rows")


if __name__ == "__main__":
    main()
