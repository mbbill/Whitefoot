#!/usr/bin/env python3
"""Build the exact trait-implementation-to-topology routing registry."""

from __future__ import annotations

import argparse
import collections
import csv
import hashlib
import io
import re
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CROSSWALK = ROOT / "RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv"
VOCABULARY = ROOT / "G0-FAMILY-GATE-VOCABULARY.md"
OUTPUT = ROOT / "G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv"

FIELDS = [
    "impl_ordinal",
    "topology_route_identity",
    "impl_key",
    "selection_family",
    "owning_contract_ids",
    "implementer",
    "exact_classifier_id",
    "classifier_rationale",
    "primary_refinement_family_or_gate",
    "required_predecessor_family_ids",
    "required_predecessor_gate_stage_ids",
    "implicated_or_reopening_family_ids",
    "implicated_or_reopening_gate_stage_ids",
    "classification_policy",
    "topology_scope_policy",
    "source_artifact",
    "source_row_sha256",
    "source_crosswalk_sha256",
    "vocabulary_sha256",
    "classifier_member_count",
    "classifier_member_sha256",
    "policy_version",
]

POLICY_VERSION = "xlang-g0-trait-impl-topology-routing-v1"
CLASSIFICATION_POLICY = (
    "EXACT_IMPLEMENTER_STRING_MEMBERSHIP_IN_CLOSED_ENUMERATED_CLASS;NO_REGEX;"
    "NO_PREFIX;NO_SUBSTRING;NO_FUZZY_MATCH;NO_DEFAULT;NO_UNMATCHED;"
    "PRIMARY_REFINEMENT_AND_PREDECESSOR_AND_IMPLICATED_REOPENING_ARE_TYPED_"
    "SEPARATELY;PREDECESSOR_NEVER_CONFERS_EVIDENCE_APPLICABILITY;TOPOLOGY_"
    "TARGETS_ARE_REFINEMENT_FAMILIES_OR_TYPED_GATE_STAGES_NOT_CROSSCUT_"
    "DIMENSIONS"
)
TOPOLOGY_SCOPE_POLICY = (
    "CLASSIFY_SELECTED_STORAGE_OR_OWNERSHIP_TOPOLOGY;AN_ALLOCATOR_TYPE_"
    "PARAMETER_ALONE_DOES_NOT_ROUTE_TO_F_ALLOC_BECAUSE_THE_CROSSWALK_SELECTS_"
    "NO_USER_VISIBLE_ALLOCATOR_SERVICE_CONTRACT;ALLOCATOR_POLICY_REMAINS_"
    "SEPARATELY_SCOPED_TO_F_ALLOC"
)


@dataclass(frozen=True)
class ExactClass:
    primary_owner_or_gate: str
    predecessor_family_ids: tuple[str, ...]
    implementers: frozenset[str]


def exact(values: str) -> frozenset[str]:
    return frozenset(line for line in values.splitlines() if line)


# These are closed exact-string classes. The builder requires exact set equality
# with all implementer strings in the pinned 334-row crosswalk. Do not replace
# them with a prefix, substring, regular-expression, or catch-all classifier.
EXACT_CLASSES = {
    "CLASS-DENSE": ExactClass(
        "F-DENSE",
        (),
        exact(
            """&'a Vec<T, A>
&'a [T; N]
&'a [T]
&'a mut Vec<T, A>
&'a mut [T; N]
&'a mut [T]
&[T]
&[T]where T: PartialEq<U>,
&mut [T]
&mut [T]where T: PartialEq<U>,
Box<[T; N]>
Box<[T], A>
Box<[T]>
Box<[u8], A>
Vec<T, A1>where T: PartialEq<U>,
Vec<T, A1>where T: PartialOrd, A1: Allocator, A2: Allocator,
Vec<T, A>
Vec<T, A>where T: PartialEq<U>,
Vec<T>
Vec<u8>
[T; 0]
[T; N]
[T; N]where T: Copy,
[T; N]where T: PartialEq<U>,
[T; N]where [T]: Index<I>,
[T; N]where [T]: IndexMut<I>,
[T]
[T]where I: SliceIndex<[T]>,
[T]where T: PartialEq<U>,"""
        )
        | frozenset(f"[T; {index}]where T: Default," for index in range(1, 33)),
    ),
    "CLASS-RECURSIVE-BOX": ExactClass(
        "F-RECURSIVE",
        (),
        exact(
            """Box<T, A>
Box<T>"""
        ),
    ),
    "CLASS-TEXT": ExactClass(
        "F-TEXT",
        ("F-DENSE",),
        exact(
            """&mut str
&str
Box<str>
String
Stringwhere I: SliceIndex<str>,
str
strwhere I: SliceIndex<str>,"""
        ),
    ),
    "CLASS-DEQUE": ExactClass(
        "F-DEQUE",
        (),
        exact(
            """&'a VecDeque<T, A>
&'a mut VecDeque<T, A>
VecDeque<T, A>
VecDeque<T, A>where T: PartialEq<U>,
VecDeque<T>"""
        ),
    ),
    "CLASS-SPARSE": ExactClass(
        "F-SPARSE",
        ("F-DENSE",),
        exact(
            """&'a HashMap<K, V, S, A>
&'a HashSet<T, S, A>
&'a mut HashMap<K, V, S, A>
HashMap<K, V, S, A>
HashMap<K, V, S, A>where K: Clone, V: Clone, S: Clone, A: Allocator + Clone,
HashMap<K, V, S, A>where K: Eq + Hash + Borrow<Q>, Q: Eq + Hash + ?Sized, S: BuildHasher, A: Allocator,
HashMap<K, V, S, A>where K: Eq + Hash + Copy, V: Copy, S: BuildHasher, A: Allocator,
HashMap<K, V, S, A>where K: Eq + Hash, S: BuildHasher, A: Allocator,
HashMap<K, V, S, A>where K: Eq + Hash, V: Eq, S: BuildHasher, A: Allocator,
HashMap<K, V, S, A>where K: Eq + Hash, V: PartialEq, S: BuildHasher, A: Allocator,
HashMap<K, V, S>where K: Eq + Hash, S: BuildHasher + Default,
HashMap<K, V, S>where S: Default,
HashSet<T, S, A>
HashSet<T, S, A>where T: 'a + Eq + Hash + Copy, S: BuildHasher, A: Allocator,
HashSet<T, S, A>where T: Clone, S: Clone, A: Allocator + Clone,
HashSet<T, S, A>where T: Eq + Hash, S: BuildHasher, A: Allocator,
HashSet<T, S>where S: Default,
HashSet<T, S>where T: Eq + Hash, S: BuildHasher + Default,"""
        ),
    ),
    "CLASS-ORDERED": ExactClass(
        "F-ORDERED",
        ("F-DENSE", "F-IDENTITY", "F-RECURSIVE"),
        exact(
            """&'a BTreeMap<K, V, A>
&'a BTreeSet<T, A>
&'a mut BTreeMap<K, V, A>
BTreeMap<K, V, A>
BTreeMap<K, V, A>where K: Borrow<Q> + Ord, Q: Ord + ?Sized,
BTreeMap<K, V>
BTreeSet<T, A>
BTreeSet<T>"""
        ),
    ),
    "CLASS-HEAP": ExactClass(
        "F-HEAP",
        ("F-DENSE",),
        exact(
            """&'a BinaryHeap<T, A>
BinaryHeap<T, A>
BinaryHeap<T>"""
        ),
    ),
    "CLASS-LINKED": ExactClass(
        "GATE-LINKED-COMPOSITION",
        ("F-DENSE", "F-IDENTITY", "F-RECURSIVE"),
        exact(
            """&'a LinkedList<T, A>
&'a mut LinkedList<T, A>
LinkedList<T, A>
LinkedList<T>"""
        ),
    ),
    "CLASS-SHARED": ExactClass(
        "F-SHARED",
        (),
        exact(
            """Rc<T, A>
Rc<T>
Weak<T, A>
Weak<T>"""
        ),
    ),
    "CLASS-SHARED-DENSE": ExactClass(
        "F-SHARED",
        ("F-DENSE",),
        exact(
            """Rc<[T; N], A>
Rc<[T], A>
Rc<[T]>
Rc<[u8]>"""
        ),
    ),
    "CLASS-SHARED-TEXT": ExactClass(
        "F-SHARED",
        ("F-TEXT",),
        exact("""Rc<str>"""),
    ),
    "CLASS-DYNAMIC-BORROW": ExactClass(
        "F-DYNAMIC-BORROW",
        (),
        exact(
            """Ref<'_, T>
RefCell<T>
RefMut<'_, T>"""
        ),
    ),
    "CLASS-ITERATION": ExactClass(
        "F-ITERATION",
        (),
        exact(
            """Char
Ipv4Addr
Ipv6Addr
NonZero<u128>
NonZero<u16>
NonZero<u32>
NonZero<u64>
NonZero<u8>
NonZero<usize>
char
i128
i16
i32
i64
i8
isize
u128
u16
u32
u64
u8
usize"""
        ),
    ),
}

CLASS_RATIONALES = {
    "CLASS-DENSE": "Contiguous Vec, slice, array, and boxed-slice or boxed-array storage is refined by the dense family.",
    "CLASS-RECURSIVE-BOX": "Generic Box ownership is the uniquely owned recursive substrate; an allocator type parameter alone imports no allocator-service contract.",
    "CLASS-TEXT": "String, str, and boxed str require the text family, with dense storage retained only as a predecessor.",
    "CLASS-DEQUE": "VecDeque implementers retain the distinct ring and rebalance topology.",
    "CLASS-SPARSE": "Hash-map and hash-set implementers require sparse occupancy, with dense storage retained only as a predecessor.",
    "CLASS-ORDERED": "B-tree map and set implementers retain comparison-ordered topology after reviewed dense node arrays and both general recursive-owner and stable-identity node routes; candidates need not combine them or pay unused metadata.",
    "CLASS-HEAP": "BinaryHeap implementers require heap repair semantics, with dense storage retained only as a predecessor.",
    "CLASS-LINKED": "LinkedList implementers pass through the linked-composition gate after dense, identity, and recursive predecessors.",
    "CLASS-SHARED": "Generic Rc and Weak implementers are owned by shared-ownership refinement and do not import dense sequence topology.",
    "CLASS-SHARED-DENSE": "Rc slice and array wrappers are owned by shared refinement while dense payload storage remains a predecessor; dense cannot disposition the wrapper child.",
    "CLASS-SHARED-TEXT": "Rc<str> is owned by shared refinement while sealed text is a predecessor; text cannot disposition the wrapper child.",
    "CLASS-DYNAMIC-BORROW": "RefCell and guard implementers require runtime dynamic-borrow topology and do not import dense sequence topology.",
    "CLASS-ITERATION": "Step implementers are exact scalar or address traversal sources owned by iteration refinement.",
}


def fail(message: str) -> None:
    raise SystemExit(f"G0 trait-impl topology-routing build failed: {message}")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def row_sha256(fields: list[str], row: dict[str, str]) -> str:
    return sha256_bytes(("\t".join(row[field] for field in fields) + "\n").encode())


def read_tsv(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    if not fields or any(None in row for row in rows):
        fail(f"{path.name} is malformed")
    if any(any("\r" in value or "\n" in value for value in row.values()) for row in rows):
        fail(f"{path.name} contains an embedded newline")
    return fields, rows


def markdown_authority_rows(
    text: str, begin_marker: str, end_marker: str, column_count: int
) -> list[list[str]]:
    if text.count(begin_marker) != 1 or text.count(end_marker) != 1:
        fail(f"authority markers are missing or duplicated: {begin_marker}")
    body = text.split(begin_marker, 1)[1].split(end_marker, 1)[0]
    table_lines = [line for line in body.splitlines() if line.startswith("|")]
    if len(table_lines) < 3:
        fail(f"authority table is empty: {begin_marker}")
    rows: list[list[str]] = []
    for line in table_lines[2:]:
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) != column_count:
            fail(f"authority table column count changed: {begin_marker}")
        rows.append(
            [
                cell[1:-1] if cell.startswith("`") and cell.endswith("`") else cell
                for cell in cells
            ]
        )
    return rows


def parse_vocabulary() -> tuple[set[str], set[str], dict[str, int], str]:
    if not VOCABULARY.is_file():
        fail(f"missing {VOCABULARY.name}")
    text = VOCABULARY.read_text(encoding="utf-8")
    tokens = re.findall(r"`((?:F|GATE)-[A-Z0-9-]+)`", text)
    families = {token for token in tokens if token.startswith("F-")}
    gates = {token for token in tokens if token.startswith("GATE-")}
    if not families or not gates:
        fail("typed vocabulary contains no family or gate-stage IDs")
    order: dict[str, int] = {}
    for token in tokens:
        if token not in order:
            order[token] = len(order)
    authority_rows = markdown_authority_rows(
        text,
        "<!-- G0_TOPOLOGY_CLASS_AUTHORITY_BEGIN -->",
        "<!-- G0_TOPOLOGY_CLASS_AUTHORITY_END -->",
        7,
    )
    authority: dict[str, tuple[str, ...]] = {}
    for cells in authority_rows:
        class_id = cells[0]
        if class_id in authority:
            fail(f"duplicate topology-class authority row: {class_id}")
        authority[class_id] = tuple(cells[1:])
    if set(authority) != set(EXACT_CLASSES):
        fail("topology-class authority ID set differs from the closed classifier")
    for class_id, exact_class in EXACT_CLASSES.items():
        primary = exact_class.primary_owner_or_gate
        predecessors = ordered(exact_class.predecessor_family_ids, order)
        implicated_families = primary if primary.startswith("F-") else "NONE"
        implicated_gates = primary if primary.startswith("GATE-") else "NONE"
        expected = (
            primary,
            predecessors,
            "NONE",
            implicated_families,
            implicated_gates,
            CLASS_RATIONALES[class_id],
        )
        if authority[class_id] != expected:
            fail(f"topology-class authority differs from code for {class_id}")
    return families, gates, order, sha256_bytes(text.encode())


def closed_classifier() -> dict[str, tuple[str, ExactClass]]:
    if set(CLASS_RATIONALES) != set(EXACT_CLASSES):
        fail("exact classifier rationale set differs from the classifier set")
    classifier: dict[str, tuple[str, ExactClass]] = {}
    for classifier_id, exact_class in EXACT_CLASSES.items():
        if not exact_class.implementers:
            fail(f"{classifier_id} is empty")
        if not exact_class.primary_owner_or_gate:
            fail(f"{classifier_id} has no topology target")
        for implementer in exact_class.implementers:
            if implementer in classifier:
                fail(f"implementer occurs in multiple exact classes: {implementer}")
            classifier[implementer] = (classifier_id, exact_class)
    return classifier


def ordered(values: tuple[str, ...], vocabulary_order: dict[str, int]) -> str:
    if not values:
        return "NONE"
    if len(values) != len(set(values)):
        fail(f"duplicate topology target in {values}")
    try:
        return ",".join(sorted(values, key=vocabulary_order.__getitem__))
    except KeyError as exc:
        fail(f"topology target is absent from typed vocabulary: {exc.args[0]}")


def build_rows() -> list[dict[str, str]]:
    crosswalk_fields, crosswalk_rows = read_tsv(CROSSWALK)
    for required in (
        "impl_key",
        "selection_family",
        "owning_contract_ids",
        "implementer",
        "source_snippet_sha256",
    ):
        if required not in crosswalk_fields:
            fail(f"crosswalk lacks {required}")
    if len(crosswalk_rows) != 334:
        fail(f"crosswalk has {len(crosswalk_rows)} rows, expected 334")
    impl_keys = [row["impl_key"] for row in crosswalk_rows]
    if len(impl_keys) != len(set(impl_keys)):
        fail("crosswalk impl_key values are not unique")
    if any(not re.fullmatch(r"[0-9a-f]{64}", key) for key in impl_keys):
        fail("crosswalk contains an invalid impl_key")

    classifier = closed_classifier()
    source_implementers = {row["implementer"] for row in crosswalk_rows}
    if set(classifier) != source_implementers:
        missing = sorted(source_implementers - set(classifier))
        extra = sorted(set(classifier) - source_implementers)
        fail(f"closed implementer set mismatch; missing={missing}; extra={extra}")

    families, gates, vocabulary_order, vocabulary_sha256 = parse_vocabulary()
    for classifier_id, exact_class in EXACT_CLASSES.items():
        primary = exact_class.primary_owner_or_gate
        if primary not in families | gates:
            fail(f"{classifier_id} contains an unknown primary owner or gate")
        if not set(exact_class.predecessor_family_ids) <= families:
            fail(f"{classifier_id} contains unknown predecessor families")
        if primary in exact_class.predecessor_family_ids:
            fail(f"{classifier_id} makes its primary family its own predecessor")
        if any(
            target.startswith("DIM-")
            for target in (primary,) + exact_class.predecessor_family_ids
        ):
            fail(f"{classifier_id} routes a topology child to a crosscut dimension")

    crosswalk_sha256 = sha256_bytes(CROSSWALK.read_bytes())
    rows: list[dict[str, str]] = []
    for ordinal, source_row in enumerate(crosswalk_rows, start=1):
        classifier_id, exact_class = classifier[source_row["implementer"]]
        predecessor_text = ordered(exact_class.predecessor_family_ids, vocabulary_order)
        implicated_family_text = (
            exact_class.primary_owner_or_gate
            if exact_class.primary_owner_or_gate.startswith("F-")
            else "NONE"
        )
        implicated_gate_text = (
            exact_class.primary_owner_or_gate
            if exact_class.primary_owner_or_gate.startswith("GATE-")
            else "NONE"
        )
        member_keys = [
            row["impl_key"]
            for row in crosswalk_rows
            if row["implementer"] in exact_class.implementers
        ]
        member_sha256 = sha256_bytes("".join(f"{key}\n" for key in member_keys).encode())
        base = {
            "impl_ordinal": str(ordinal),
            "impl_key": source_row["impl_key"],
            "selection_family": source_row["selection_family"],
            "owning_contract_ids": source_row["owning_contract_ids"],
            "implementer": source_row["implementer"],
            "exact_classifier_id": classifier_id,
            "classifier_rationale": CLASS_RATIONALES[classifier_id],
            "primary_refinement_family_or_gate": exact_class.primary_owner_or_gate,
            "required_predecessor_family_ids": predecessor_text,
            "required_predecessor_gate_stage_ids": "NONE",
            "implicated_or_reopening_family_ids": implicated_family_text,
            "implicated_or_reopening_gate_stage_ids": implicated_gate_text,
            "classification_policy": CLASSIFICATION_POLICY,
            "topology_scope_policy": TOPOLOGY_SCOPE_POLICY,
            "source_artifact": CROSSWALK.name,
            "source_row_sha256": row_sha256(crosswalk_fields, source_row),
            "source_crosswalk_sha256": crosswalk_sha256,
            "vocabulary_sha256": vocabulary_sha256,
            "classifier_member_count": str(len(member_keys)),
            "classifier_member_sha256": member_sha256,
            "policy_version": POLICY_VERSION,
        }
        identity_fields = [base[field] for field in FIELDS if field != "topology_route_identity"]
        base["topology_route_identity"] = sha256_bytes(
            ("\n".join(identity_fields) + "\n").encode()
        )
        rows.append(base)
    return rows


def render(rows: list[dict[str, str]]) -> str:
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=FIELDS, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return output.getvalue()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    arguments = parser.parse_args()
    expected = render(build_rows())
    if arguments.check:
        if not OUTPUT.is_file() or OUTPUT.read_text(encoding="utf-8") != expected:
            fail(f"{OUTPUT.name} is missing or stale")
    else:
        OUTPUT.write_text(expected, encoding="utf-8")
    class_counts = collections.Counter(row["exact_classifier_id"] for row in build_rows())
    print(
        "G0 trait-impl topology routing: PASS — 334 exact impl_key rows; "
        f"{len(EXACT_CLASSES)} closed exact implementer classes; counts={dict(sorted(class_counts.items()))}"
    )


if __name__ == "__main__":
    main()
