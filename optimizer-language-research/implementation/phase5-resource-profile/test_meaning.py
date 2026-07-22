from hashlib import sha256
from pathlib import Path
import unittest

from meaning import (
    PROPOSAL_SHA256,
    construct_meaning_digests,
    meaning_digests,
)
from schema import schema_descriptor


DIRECTORY = Path(__file__).resolve().parent
REPOSITORY = DIRECTORY.parents[2]


class MeaningIdentityTests(unittest.TestCase):
    def test_approved_proposal_and_meaning_digests_are_pinned(self) -> None:
        proposal = (
            REPOSITORY
            / "optimizer-language-research"
            / "implementation"
            / "phase5-successor-proposal"
            / "PROPOSAL.md"
        ).read_bytes()
        self.assertEqual(sha256(proposal).hexdigest(), PROPOSAL_SHA256)
        self.assertEqual(
            tuple(value.hex() for value in meaning_digests(DIRECTORY)),
            (
                "981878811e38716acfd5dc0bbacccf278c68b2db29aa987af98937e65649d754",
                "2d085436e8d9288a982ef83a13554c2310cead38892e8223d7f2661b60b3c7e7",
                "6d624da13ddd48d6dd46f3a2feaac38b83b51e4154e0e70e08a73524e9e7505a",
            ),
        )

    def test_every_meaning_input_changes_an_identity(self) -> None:
        inputs = [
            schema_descriptor(),
            (DIRECTORY / "SCHEMA-SEMANTICS.md").read_bytes(),
            (DIRECTORY / "WORK-SCHEDULE.md").read_bytes(),
            (DIRECTORY / "STORAGE-MODEL.md").read_bytes(),
            bytes.fromhex(PROPOSAL_SHA256),
        ]
        baseline = construct_meaning_digests(*inputs)
        for index, value in enumerate(inputs):
            changed = list(inputs)
            mutation = bytearray(value)
            mutation[-1] ^= 1
            changed[index] = bytes(mutation)
            self.assertNotEqual(construct_meaning_digests(*changed), baseline)


if __name__ == "__main__":
    unittest.main()
