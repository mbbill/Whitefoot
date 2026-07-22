from dataclasses import replace
from pathlib import Path
import sys
import unittest


sys.path.insert(0, str(Path(__file__).parent))

from manifest import FAMILY_CODEC, FAMILY_COMPILER  # noqa: E402
from relation import (  # noqa: E402
    DOMAIN_CONTRACT,
    DOMAIN_ENUM_CONSTRUCTOR,
    DOMAIN_NOMINAL,
    build_relation,
)
from selection import select_diagnostic  # noqa: E402
from test_manifest import fixture  # noqa: E402


class SelectionTests(unittest.TestCase):
    def test_both_families_earn_complete_at_multiple_scales(self) -> None:
        for family in (FAMILY_COMPILER, FAMILY_CODEC):
            for units in (1, 2, 17):
                manifest, _ = fixture(family, units)
                self.assertEqual(
                    select_diagnostic(build_relation(manifest)), ("Complete",)
                )

    def test_prelude_domains_and_codec_variant_owners_are_exact(self) -> None:
        manifest, _ = fixture(FAMILY_CODEC, 1)
        relation = build_relation(manifest)
        prelude = {
            (entry.spelling, entry.domain, entry.constructor_owner)
            for entry in relation.lookup_entries
            if entry.origin_kind == 0
        }
        self.assertIn(("Bool", DOMAIN_NOMINAL, None), prelude)
        self.assertIn(("Int", DOMAIN_CONTRACT, None), prelude)
        self.assertIn(("Ok", DOMAIN_ENUM_CONSTRUCTOR, "Result"), prelude)
        source_variants = {
            entry.spelling: entry.constructor_owner
            for entry in relation.lookup_entries
            if entry.origin_kind == 2
            and entry.domain == DOMAIN_ENUM_CONSTRUCTOR
        }
        self.assertEqual(source_variants["CodecOk000000"], "CodecResult000000")
        self.assertEqual(source_variants["CodecErr000000"], "CodecResult000000")

    def test_fn8_failure_precedes_inventory_and_resolution(self) -> None:
        manifest, _ = fixture(FAMILY_CODEC, 1)
        relation = build_relation(manifest)
        mutated = replace(relation, requires_shapes=(("ordinary-let",),))
        self.assertEqual(
            select_diagnostic(mutated)[:3],
            ("SourceIssue", "FN-8", "missing-final-check"),
        )

    def test_unresolved_use_is_not_assumed_complete(self) -> None:
        manifest, _ = fixture(FAMILY_COMPILER, 1)
        relation = build_relation(manifest)
        roles = list(relation.roles)
        index = next(
            index for index, role in enumerate(roles) if role.category == "lexical"
        )
        roles[index] = replace(roles[index], spelling="missing_generated_name")
        diagnostic = select_diagnostic(replace(relation, roles=tuple(roles)))
        self.assertEqual(diagnostic[:3], ("SourceIssue", "Resolution", "unresolved"))

    def test_inventory_collision_is_selected_before_uses(self) -> None:
        manifest, _ = fixture(FAMILY_COMPILER, 1)
        relation = build_relation(manifest)
        source_entry = next(
            entry for entry in relation.lookup_entries if entry.origin_kind == 2
        )
        diagnostic = select_diagnostic(
            replace(
                relation,
                lookup_entries=relation.lookup_entries + (source_entry,),
            )
        )
        self.assertEqual(diagnostic[:3], ("SourceIssue", "TYPE-6", "collision"))

    def test_wrong_domain_is_distinct_from_unresolved(self) -> None:
        manifest, _ = fixture(FAMILY_CODEC, 1)
        relation = build_relation(manifest)
        entries = list(relation.lookup_entries)
        index = next(
            index
            for index, entry in enumerate(entries)
            if entry.spelling == "CodecOk000000" and entry.origin_kind == 2
        )
        entries[index] = replace(
            entries[index], domain=DOMAIN_NOMINAL, constructor_owner=None
        )
        diagnostic = select_diagnostic(
            replace(relation, lookup_entries=tuple(entries))
        )
        self.assertEqual(diagnostic[:3], ("SourceIssue", "Resolution", "wrong-domain"))


if __name__ == "__main__":
    unittest.main()
