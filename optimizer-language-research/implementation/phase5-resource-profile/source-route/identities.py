"""Fail-closed identity and parser-independence checks for source route A."""

from __future__ import annotations

import ast
from hashlib import sha256
from pathlib import Path

from model import RouteError


ROUTE_ROOT = Path(__file__).resolve().parent
REPOSITORY_ROOT = ROUTE_ROOT.parents[3]
PROPOSAL_ROOT = (
    REPOSITORY_ROOT
    / "optimizer-language-research/implementation/phase5-successor-proposal"
)
PROFILE_ROOT = ROUTE_ROOT.parent
PARSER_ROOT = REPOSITORY_ROOT / "grammar-verifier/proposal"

PROPOSAL_SHA256 = "7fc48cc30f94d25be5be1106e3265d92c1b0cdf2bfea5a7a17759a12f3cf092d"
CANDIDATE_SHA256 = "71073e25219455896250e15e13d1ffdbfc443c87a9b28cb9906d73a020dc33e9"
MEANING_DIGESTS = {
    "semantics": "981878811e38716acfd5dc0bbacccf278c68b2db29aa987af98937e65649d754",
    "storage": "6d624da13ddd48d6dd46f3a2feaac38b83b51e4154e0e70e08a73524e9e7505a",
    "work": "2d085436e8d9288a982ef83a13554c2310cead38892e8223d7f2661b60b3c7e7",
}

BOUND_FILES = {
    PROPOSAL_ROOT / "PROPOSAL.md": PROPOSAL_SHA256,
    PROPOSAL_ROOT / "kernel-spec-v0.10-candidate.md": CANDIDATE_SHA256,
    PROFILE_ROOT / "schema.py": "d7c0c2a276fa506ebfd6c25f77d23f5d2ffe07582fb88b17a8752f7eef4f9f0f",
    PROFILE_ROOT / "SCHEMA-SEMANTICS.md": "046660f49d6585c2566db482bebed2f3e11a0e6a3bd92e3f61da539ef0766afa",
    PROFILE_ROOT / "WORK-SCHEDULE.md": "8db571e1dfa3807d79a7c59464ea5397210ce26eb230362ca8305d20e5d85005",
    PROFILE_ROOT / "STORAGE-MODEL.md": "ee6e8cd0dd70d81eaa0ca11db4614e3877afce1241e9413a7dd9863aeb4f3139",
}

PARSER_FILES = {
    "form2_independent_lex.py": "4821a83002ddbc5928ce6289f2657241499bb1f4fd90fa2303ec17af6c606854",
    "form2_independent_parse.py": "431a87b8baada0c2aacb38db02206a7a88f5a9e4d772e9730eb1479dd1547438",
    "form2_independent_parse_core.py": "bf45d6fce5c97df5bd60b7b5a1d8a42152b38ad601b009ec396ef865f6e1dfac",
    "form2_independent_parse_expressions.py": "cb9579135cd05ffbb8ddd129ac82668fd6662375e99f0624a994cba63a5a0054",
    "form2_independent_parse_items.py": "7c283eb4377fb2ae74a66b8d87d0fd0913767ac3ac05edaf99ca7b83eb2840cc",
    "form2_independent_parse_signatures.py": "a358df9717c88b7131bd4084d8cb55100f188a8fb89b98b5e295e936d4d1f186",
    "form2_independent_parse_statements.py": "d17d5d9da9bcef23a5d773095791303b119ecabf9c6b6e19371b7ba81b21ddaf",
    "form2_independent_parse_types.py": "e79cb429a273099528f5c0d49d380db1e4648cc7b19b91013e128a105bc12351",
    "form2_independent_syntax.py": "d49df3ad6036ff2d2244a23763a1217313d3cbda58d4464bbb09305958401a17",
}

FORBIDDEN_PARSER_FRAGMENTS = (
    b"whitefoot_",
    b"diagnostic" + b"_evidence",
    b"archive" + b"/",
    b"route-b",
    b"source-route",
)
ROUTE_FILES = (
    "counts.py",
    "fn8.py",
    "identities.py",
    "inventory_selection.py",
    "lexical_selection.py",
    "manifest.py",
    "measurement.py",
    "model.py",
    "parser_adapter.py",
    "receipt.py",
    "receipt_diagnostic.py",
    "receipt_schema.py",
    "receipt_structure.py",
    "receipt_validation.py",
    "receipt_values.py",
    "resolution.py",
    "roles.py",
    "run.py",
    "topology.py",
)
STANDARD_IMPORT_ROOTS = frozenset(
    {
        "__future__",
        "argparse",
        "ast",
        "dataclasses",
        "hashlib",
        "json",
        "os",
        "pathlib",
        "re",
        "stat",
        "struct",
        "sys",
        "tempfile",
        "typing",
    }
)
LOCAL_IMPORT_ROOTS = frozenset(name.removesuffix(".py") for name in ROUTE_FILES)
AUDITED_PARSER_IMPORT_ROOTS = frozenset(
    {"form2_independent_lex", "form2_independent_parse"}
)
FORBIDDEN_ROUTE_FRAGMENTS = (
    b"analytic" + b"-route",
    b"cross_route" + b"_agreement",
    b"diagnostic" + b"_evidence",
    b"evidence" + b"_manifest",
    b"archive" + b"/",
    b"workloads" + b".py",
)


def digest(raw: bytes) -> str:
    """Return lowercase SHA-256 for exact bytes."""

    return sha256(raw).hexdigest()


def _read_regular(path: Path) -> bytes:
    if not path.is_file() or path.is_symlink():
        raise RouteError(f"identity-bound file is absent or not regular: {path}")
    try:
        return path.read_bytes()
    except OSError as error:
        raise RouteError(f"cannot read identity-bound file: {path}") from error


def verify_identities() -> dict[str, str]:
    """Verify every authority and the audited parser input boundary."""

    observed: dict[str, str] = {}
    for path, expected in BOUND_FILES.items():
        actual = digest(_read_regular(path))
        if actual != expected:
            raise RouteError(f"identity mismatch for {path.name}: {actual}")
        observed[path.name] = actual

    parser_hasher = sha256(b"WHITEFOOT-SOURCE-ROUTE-PARSER-AUDIT-V1\0")
    for name, expected in PARSER_FILES.items():
        path = PARSER_ROOT / name
        raw = _read_regular(path)
        actual = digest(raw)
        if actual != expected:
            raise RouteError(f"audited parser identity mismatch for {name}: {actual}")
        for fragment in FORBIDDEN_PARSER_FRAGMENTS:
            if fragment in raw:
                raise RouteError(
                    f"audited parser crosses a forbidden dependency boundary: {name}"
                )
        encoded_name = name.encode("ascii")
        parser_hasher.update(len(encoded_name).to_bytes(2, "big"))
        parser_hasher.update(encoded_name)
        parser_hasher.update(bytes.fromhex(actual))
    observed["audited_parser_set"] = parser_hasher.hexdigest()
    observed["source_route_code"] = route_code_digest()
    return observed


def meaning_file_hashes() -> dict[str, str]:
    """Bind the receipt to the three non-authoritative meaning documents."""

    return {
        name: digest(_read_regular(PROFILE_ROOT / name))
        for name in ("SCHEMA-SEMANTICS.md", "WORK-SCHEDULE.md", "STORAGE-MODEL.md")
    }


def route_code_digest() -> str:
    """Bind a receipt to the exact closed non-test source-route file set."""

    observed = tuple(
        sorted(
            path.name
            for path in ROUTE_ROOT.glob("*.py")
            if not path.name.startswith("test_")
        )
    )
    if observed != tuple(sorted(ROUTE_FILES)):
        raise RouteError("source-route non-test file set is not closed")
    hasher = sha256(b"WHITEFOOT-SOURCE-ROUTE-CODE-V1\0")
    for name in ROUTE_FILES:
        raw = _read_regular(ROUTE_ROOT / name)
        for fragment in FORBIDDEN_ROUTE_FRAGMENTS:
            if fragment in raw:
                raise RouteError(
                    f"source-route code crosses a forbidden dependency boundary: {name}"
                )
        try:
            tree = ast.parse(raw, filename=name)
        except (SyntaxError, ValueError) as error:
            raise RouteError(f"source-route file is not valid Python: {name}") from error
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                imported = tuple(alias.name.split(".", 1)[0] for alias in node.names)
            elif isinstance(node, ast.ImportFrom):
                if node.level or node.module is None:
                    raise RouteError(
                        f"source-route uses an unclosed relative import: {name}"
                    )
                imported = (node.module.split(".", 1)[0],)
            else:
                continue
            for root in imported:
                if root in AUDITED_PARSER_IMPORT_ROOTS:
                    if name != "parser_adapter.py":
                        raise RouteError(
                            f"source-route parser import escapes its adapter: {name}"
                        )
                elif root not in STANDARD_IMPORT_ROOTS | LOCAL_IMPORT_ROOTS:
                    raise RouteError(
                        f"source-route imports an unapproved dependency {root}: {name}"
                    )
        encoded = name.encode("ascii")
        hasher.update(len(encoded).to_bytes(2, "big"))
        hasher.update(encoded)
        hasher.update(bytes.fromhex(digest(raw)))
    return hasher.hexdigest()
