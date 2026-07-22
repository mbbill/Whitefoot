"""Focused tests for the independent source/tree/role/count route."""

from __future__ import annotations

from hashlib import sha256
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest import mock

from counts import AGREEMENT_DERIVED_NAMES, FIELD_NAMES, TRACE_FIELDS
import identities
from identities import REPOSITORY_ROOT, ROUTE_FILES, route_code_digest, verify_identities
from model import LogicalSource, RouteError
from parser_adapter import parse_bundle
from receipt import decode_receipt, encode_receipt, measure
from roles import ALL_ROLES, project_roles
from topology import build_projection_context


MAIN = b"""fn main() -> own unit pure {
  return unit;
}
"""

ROLE_FEATURES = b"""const capacity: u64 = 1_u64;

const alias: u64 = capacity;

enum E {
  V(field: u64);
}

contract C<T: Int> {
  fn member ['s](arg: &'s T) -> own unit reads('s);
  law identity(0_T);
}

fn helper<T: Int, const n: u64> ['r](p: &'r T, data: own slice<'r, u8>, values: own array<u8, n>) -> own unit reads('r) {
  region 'q {
    let borrowed: &'r T = &'r p;
    let chosen: own T = 0_T;
    helper<'r>(p: p, data: data, values: values);
    match p {
      V(field: bound) => {
        let projected: own u64 = p.field;
        return unit;
      }
    }
  }
}
"""


def by_name(receipt: dict[str, object]) -> dict[str, dict[str, object]]:
    return {row["name"]: row for row in receipt["counts"]}


class SourceRouteTests(unittest.TestCase):
    def test_identity_and_closed_field_order(self) -> None:
        identities = verify_identities()
        self.assertEqual(len(identities["audited_parser_set"]), 64)
        self.assertEqual(identities["source_route_code"], route_code_digest())
        receipt = measure((LogicalSource("main.wf", MAIN),))
        self.assertEqual(
            [row["name"] for row in receipt["counts"]], list(FIELD_NAMES)
        )
        states = by_name(receipt)
        self.assertEqual(
            {name for name, row in states.items() if row["state"] == "trace-required"},
            set(TRACE_FIELDS),
        )
        self.assertEqual(receipt["status"], "trace-incomplete")
        self.assertEqual(
            set(receipt["identities"]["meaning_sha256"]),
            {"semantics", "storage", "work"},
        )
        self.assertEqual(
            tuple(receipt["agreement_derived_counts"]),
            AGREEMENT_DERIVED_NAMES,
        )
        self.assertEqual(encode_receipt(receipt), encode_receipt(receipt))

    def test_exact_frontend_and_resolution_receipts(self) -> None:
        receipt = measure((LogicalSource("main.wf", MAIN),))
        fields = by_name(receipt)
        expected = {
            "max_sources": 1,
            "max_total_source_bytes": len(MAIN),
            "max_binding_bytes": 50 + 16 + len(b"main.wf") + len(MAIN),
            "max_tokens": 13,
            "max_classified_tokens": 13,
            "max_production_nodes": 11,
            "max_mixed_elements": 23,
            "max_tree_depth": 6,
            "max_declarations": 25,
            "max_scopes": 3,
            "max_scope_depth": 2,
            "max_declaration_events": 1,
            "max_lookup_entries": 102,
            "max_ancestry_steps": 2,
            "max_coverage_records": 12,
        }
        for name, value in expected.items():
            self.assertEqual((fields[name]["state"], fields[name]["value"]), ("exact", value))
        derived = receipt["derived_counts"]
        self.assertEqual(
            derived["private_derivation_elements"],
            fields["max_production_nodes"]["value"] + fields["max_tokens"]["value"],
        )
        self.assertEqual(
            derived["mixed_elements"],
            fields["max_production_nodes"]["value"] - 1 + fields["max_tokens"]["value"],
        )
        self.assertEqual(receipt["selected_diagnostic"], None)

    def test_prog2_paths_and_source_order_are_closed(self) -> None:
        receipt = measure(
            (
                LogicalSource("empty.wf", b"\n"),
                LogicalSource("src/main-1.wf", MAIN),
            )
        )
        fields = by_name(receipt)
        self.assertEqual(fields["max_sources"]["value"], 2)
        self.assertEqual(fields["max_source_bytes"]["value"], len(MAIN))
        self.assertEqual(receipt["source_bundle"]["sources"][0]["logical_path"], "empty.wf")
        bad = (
            "",
            "/absolute.wf",
            "trailing/",
            "a//b.wf",
            "./a.wf",
            "a/../b.wf",
            "space name.wf",
            "unicode-λ.wf",
            "back\\slash.wf",
        )
        for path in bad:
            with self.subTest(path=path), self.assertRaises(RouteError):
                measure((LogicalSource(path, MAIN),))
        with self.assertRaises(RouteError):
            measure((LogicalSource("a.wf", MAIN), LogicalSource("a.wf", MAIN)))

    def test_noncanonical_or_unparseable_source_fails_closed(self) -> None:
        with self.assertRaises(RouteError):
            measure((LogicalSource("main.wf", MAIN.replace(b"  return", b" return")),))
        with self.assertRaises(RouteError):
            measure((LogicalSource("main.wf", b"fn main("),))

    def test_fn8_precedes_roles_and_leaves_counts_not_derived(self) -> None:
        source = b"""fn f() -> own unit pure requires {
} {
  return unit;
}
"""
        receipt = measure((LogicalSource("fn8.wf", source),))
        issue = receipt["selected_diagnostic"]
        self.assertEqual((issue["stage"], issue["rule"]), ("fn8-admission", "FN-8"))
        self.assertEqual(receipt["projection_summary"]["role_occurrences"], 0)
        fields = by_name(receipt)
        self.assertEqual(fields["max_declarations"]["state"], "not-derived")
        self.assertEqual(fields["max_diagnostic_paths"]["value"], 1)
        self.assertEqual(
            receipt["derived_counts"]["diagnostic_issue_elements"],
            2 + len(issue["node_path"]),
        )

    def test_inventory_and_resolution_diagnostics_are_ordered(self) -> None:
        reserved = b"""fn ieq() -> own unit pure {
  return unit;
}
"""
        duplicate = b"""fn f() -> own unit pure {
  return unit;
}

fn f() -> own unit pure {
  return unit;
}
"""
        unresolved = b"""fn f() -> own unit pure {
  missing();
  return unit;
}
"""
        cases = (
            (reserved, ("inventory", "FORM-3", "reserved-declaration-name")),
            (duplicate, ("inventory", "TYPE-6", "same-scope-collision")),
            (unresolved, ("lexical-resolution", "OP-1", "admissible-target-absent")),
        )
        for source, expected in cases:
            with self.subTest(expected=expected):
                issue = measure((LogicalSource("issue.wf", source),))["selected_diagnostic"]
                self.assertEqual((issue["stage"], issue["rule"], issue["reason"]), expected)

    def test_smoke_sources_cover_the_complete_role_matrix(self) -> None:
        observed: set[str] = set()
        for source in (ROLE_FEATURES,):
            parsed = parse_bundle((LogicalSource("roles.wf", source),))
            context = build_projection_context(parsed)
            observed.update(role.role_id for role in project_roles(parsed, context))
        for family in ("compiler", "codec"):
            source, _ = self._generate(family, 1)
            parsed = parse_bundle((LogicalSource(f"{family}.wf", source),))
            context = build_projection_context(parsed)
            observed.update(role.role_id for role in project_roles(parsed, context))
        self.assertEqual(observed, set(ALL_ROLES))

    def _generate(self, family: str, units: int) -> tuple[bytes, bytes]:
        generator = (
            REPOSITORY_ROOT
            / "optimizer-language-research/implementation/phase5-resource-profile/workloads.py"
        )
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "source.wf"
            manifest = Path(directory) / "manifest.json"
            subprocess.run(
                [
                    "python3",
                    str(generator),
                    "--family",
                    family,
                    "--units",
                    str(units),
                    "--output",
                    str(source),
                    "--manifest-output",
                    str(manifest),
                ],
                cwd=REPOSITORY_ROOT,
                check=True,
                capture_output=True,
            )
            return source.read_bytes(), manifest.read_bytes()

    def test_neutral_smoke_manifests_bind_without_importing_producer(self) -> None:
        goldens = {
            "compiler": {
                "max_source_bytes": 593,
                "max_tokens": 130,
                "max_production_nodes": 117,
                "max_declarations": 38,
                "max_scopes": 11,
                "max_lookup_entries": 115,
                "max_coverage_records": 144,
            },
            "codec": {
                "max_source_bytes": 742,
                "max_tokens": 150,
                "max_production_nodes": 123,
                "max_declarations": 40,
                "max_scopes": 11,
                "max_lookup_entries": 114,
                "max_coverage_records": 157,
            },
        }
        for family, expected in goldens.items():
            source, manifest = self._generate(family, 1)
            logical_path = f"demand/{family}-000001.wf"
            receipt = measure((LogicalSource(logical_path, source),), manifest)
            fields = by_name(receipt)
            self.assertEqual(receipt["workload"]["family"], family)
            self.assertEqual(receipt["workload"]["units"], 1)
            for name, value in expected.items():
                self.assertEqual(fields[name]["value"], value)
            encoded = encode_receipt(receipt)
            self.assertEqual(len(sha256(encoded).hexdigest()), 64)

    def test_manifest_mutation_fails_closed(self) -> None:
        source, raw = self._generate("compiler", 1)
        value = json.loads(raw)
        value["parameters"][0]["value"] = 7
        mutated = (
            json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True)
            + "\n"
        ).encode("ascii")
        with self.assertRaises(RouteError):
            measure(
                (LogicalSource("demand/compiler-000001.wf", source),),
                mutated,
            )
        value = json.loads(raw)
        value["sources"].append(
            {
                "byte_length": len(MAIN),
                "logical_path": "extra.wf",
                "sha256": sha256(MAIN).hexdigest(),
            }
        )
        two_source = (
            json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True)
            + "\n"
        ).encode("ascii")
        with self.assertRaises(RouteError):
            measure(
                (
                    LogicalSource("demand/compiler-000001.wf", source),
                    LogicalSource("extra.wf", MAIN),
                ),
                two_source,
            )

    def test_nested_receipt_mutations_fail_closed(self) -> None:
        duplicate = b"""fn f() -> own unit pure {
  return unit;
}

fn f() -> own unit pure {
  return unit;
}
"""
        baseline = measure((LogicalSource("duplicate.wf", duplicate),))
        self.assertTrue(baseline["selected_diagnostic"]["origins"])
        mutations = []

        meaning = json.loads(encode_receipt(baseline))
        meaning["identities"]["meaning_sha256"]["extra"] = "0" * 64
        mutations.append(meaning)

        source = json.loads(encode_receipt(baseline))
        source["source_bundle"]["sources"][0]["extra"] = 0
        mutations.append(source)

        payload = json.loads(encode_receipt(baseline))
        payload["selected_diagnostic"]["payload"]["extra"] = []
        mutations.append(payload)

        origin = json.loads(encode_receipt(baseline))
        origin["selected_diagnostic"]["origins"][0]["extra"] = False
        mutations.append(origin)

        trace = json.loads(encode_receipt(baseline))
        trace["trace_gaps"][0]["allowed_inputs"] = [{"not": "text"}]
        mutations.append(trace)

        for index, value in enumerate(mutations):
            with self.subTest(index=index):
                forged = (
                    json.dumps(
                        value,
                        ensure_ascii=True,
                        allow_nan=False,
                        separators=(",", ":"),
                        sort_keys=True,
                    )
                    + "\n"
                ).encode("ascii")
                with self.assertRaises(RouteError):
                    decode_receipt(forged)

    def test_route_dependency_and_file_set_are_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            copied = Path(directory)
            for name in ROUTE_FILES:
                shutil.copy2(identities.ROUTE_ROOT / name, copied / name)
            with (copied / "counts.py").open("ab") as stream:
                stream.write(b"\nimport workloads\n")
            with mock.patch.object(identities, "ROUTE_ROOT", copied):
                with self.assertRaisesRegex(RouteError, "dependency"):
                    route_code_digest()

        with tempfile.TemporaryDirectory() as directory:
            copied = Path(directory)
            for name in ROUTE_FILES:
                shutil.copy2(identities.ROUTE_ROOT / name, copied / name)
            (copied / "unreviewed.py").write_text("value = 1\n", encoding="ascii")
            with mock.patch.object(identities, "ROUTE_ROOT", copied):
                with self.assertRaisesRegex(RouteError, "file set is not closed"):
                    route_code_digest()


if __name__ == "__main__":
    unittest.main()
