from __future__ import annotations

from pathlib import Path
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
PROPOSAL = ROOT / "proposal"
if str(PROPOSAL) not in sys.path:
    sys.path.insert(0, str(PROPOSAL))

from form2_independent_lex import lex_independently  # noqa: E402
from form2_independent_parse import parse_independently  # noqa: E402
from form2_independent_syntax import (  # noqa: E402
    IndependentForest,
    IndependentNode,
    IndependentTreeError,
    source_forest_projection as independent_forest_projection,
)
from form2_independent_topology import (  # noqa: E402
    IndependentGlobalTopology,
    IndependentSourceTopology,
    bundle_topology_projection as independent_bundle_projection,
    derive_separator_owners as derive_independent_owners,
    separator_owner_projection as independent_owner_projection,
)
from form2_topology import (  # noqa: E402
    GlobalProgramTopology,
    SourceForestTopology,
    TopologyError,
    bundle_topology_projection,
    derive_separator_owners,
    separator_owner_projection,
)
from form2_tree import (  # noqa: E402
    ProductionNode,
    load_candidate_parser,
    parse_one,
    source_forest_projection,
)


def independently_derive(raw: bytes):
    tokens = lex_independently(raw)
    return parse_independently(tokens, len(raw))


class PrimaryTopologyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.parser = load_candidate_parser()

    def test_source_projection_omits_the_parser_program_wrapper(self) -> None:
        derivation = parse_one(
            self.parser, b"fn main() -> own unit pure { return unit; }\n"
        ).derivation
        self.assertEqual([item.name for item in derivation.items], ["item"])
        self.assertNotIn(b"program", source_forest_projection(derivation.items))

    def test_global_topology_has_one_program_and_retains_empty_source(self) -> None:
        first = parse_one(
            self.parser, b"fn main() -> own unit pure { return unit; }\n"
        ).derivation
        empty = parse_one(self.parser, b"  \n").derivation
        topology = GlobalProgramTopology(
            (
                SourceForestTopology(
                    0, first.source_length, len(first.terminals), first.items
                ),
                SourceForestTopology(1, empty.source_length, 0, empty.items),
            )
        )
        self.assertEqual(bundle_topology_projection(topology).count(b"program"), 1)
        self.assertEqual(topology.program_node_count, 1)
        self.assertEqual(
            [
                (owner.boundary_kind, owner.location_kind)
                for owner in derive_separator_owners(topology, 1)
            ],
            [("zero-item", "SourceBytes")],
        )

    def test_separator_owners_come_from_global_item_paths(self) -> None:
        derivation = parse_one(
            self.parser,
            b"const a: i32 = 1_i32; const b: i32 = 2_i32;\n",
        ).derivation
        topology = GlobalProgramTopology(
            (
                SourceForestTopology(
                    0,
                    derivation.source_length,
                    len(derivation.terminals),
                    derivation.items,
                ),
            )
        )
        owners = derive_separator_owners(topology, 0)
        self.assertEqual(owners[0].location_kind, "SourceBytes")
        self.assertEqual(owners[-1].location_kind, "SourceBytes")
        inter_item = [
            owner for owner in owners if owner.boundary_kind == "inter-item"
        ]
        self.assertEqual(len(inter_item), 1)
        self.assertEqual(inter_item[0].owner_path, ())
        within = [
            owner for owner in owners if owner.boundary_kind == "within-item"
        ]
        self.assertTrue(within)
        self.assertTrue(
            all(
                owner.location_kind == "SourceNode"
                and owner.owner_path
                and owner.owner_production != "program"
                for owner in within
            )
        )

    def test_source_forest_rejects_a_fabricated_program_node(self) -> None:
        fabricated = ProductionNode("program", 0, 0, (0,), ())
        with self.assertRaisesRegex(TopologyError, "program node"):
            GlobalProgramTopology(
                (SourceForestTopology(0, 1, 1, (fabricated,)),)
            )


class IndependentTopologyTests(unittest.TestCase):
    def test_empty_source_forest_projection_is_exact(self) -> None:
        forest = independently_derive(b"\n")
        projection = independent_forest_projection(forest)
        self.assertEqual(projection, (0).to_bytes(4, "big"))
        self.assertNotIn(b"program", projection)

    def test_global_topology_has_one_program_and_retains_empty_source(self) -> None:
        first = independently_derive(
            b"fn main() -> own unit pure { return unit; }\n"
        )
        empty = independently_derive(b"  \n")
        topology = IndependentGlobalTopology(
            (
                IndependentSourceTopology(0, first),
                IndependentSourceTopology(1, empty),
            )
        )
        self.assertEqual(independent_bundle_projection(topology).count(b"program"), 1)
        self.assertEqual(
            [
                (owner.boundary_kind, owner.location_kind)
                for owner in derive_independent_owners(topology, 1)
            ],
            [("zero-item", "SourceBytes")],
        )

    def test_source_forest_cannot_fabricate_a_program_root(self) -> None:
        fabricated = IndependentNode("program", 0, 1)
        with self.assertRaisesRegex(IndependentTreeError, "non-item root"):
            IndependentForest((fabricated,), 1, 1)

    def test_separator_owner_paths_come_from_adjacent_leaves(self) -> None:
        forest = independently_derive(
            b"const a: i32 = 1_i32; const b: i32 = 2_i32;\n"
        )
        topology = IndependentGlobalTopology(
            (IndependentSourceTopology(0, forest),)
        )
        owners = derive_independent_owners(topology, 0)
        self.assertEqual(owners[0].location_kind, "SourceBytes")
        self.assertEqual(owners[-1].location_kind, "SourceBytes")
        inter_item = [
            owner for owner in owners if owner.boundary_kind == "inter-item"
        ]
        self.assertEqual(len(inter_item), 1)
        self.assertEqual(inter_item[0].owner_path, ())
        within = [
            owner for owner in owners if owner.boundary_kind == "within-item"
        ]
        self.assertTrue(within)
        self.assertTrue(
            all(
                owner.location_kind == "SourceNode"
                and owner.owner_path
                and owner.owner_production != "program"
                for owner in within
            )
        )


class CrossTopologyTests(unittest.TestCase):
    def test_independent_wire_projections_match_primary_multi_source_model(self) -> None:
        parser = load_candidate_parser()
        raw_sources = (
            b"const a: i32 = 1_i32; const b: i32 = 2_i32;\n",
            b"  \n",
            b"fn main() -> own unit pure { return unit; }\n",
        )
        primary_derivations = tuple(
            parse_one(parser, raw).derivation for raw in raw_sources
        )
        independent_forests = tuple(
            independently_derive(raw) for raw in raw_sources
        )
        primary = GlobalProgramTopology(
            tuple(
                SourceForestTopology(
                    ordinal,
                    derivation.source_length,
                    len(derivation.terminals),
                    derivation.items,
                )
                for ordinal, derivation in enumerate(primary_derivations)
            )
        )
        independent = IndependentGlobalTopology(
            tuple(
                IndependentSourceTopology(ordinal, forest)
                for ordinal, forest in enumerate(independent_forests)
            )
        )
        self.assertEqual(
            bundle_topology_projection(primary),
            independent_bundle_projection(independent),
        )
        for ordinal, (derivation, forest) in enumerate(
            zip(primary_derivations, independent_forests)
        ):
            self.assertEqual(
                source_forest_projection(derivation.items),
                independent_forest_projection(forest),
            )
            self.assertEqual(
                separator_owner_projection(primary, ordinal),
                independent_owner_projection(independent, ordinal),
            )
        third_source_nodes = [
            owner
            for owner in derive_separator_owners(primary, 2)
            if owner.location_kind == "SourceNode"
        ]
        self.assertTrue(third_source_nodes)
        self.assertTrue(
            all(owner.owner_path[0] == 2 for owner in third_source_nodes)
        )


if __name__ == "__main__":
    unittest.main()
