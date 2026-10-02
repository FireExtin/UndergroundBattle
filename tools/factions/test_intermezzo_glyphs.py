"""Pinned original-image glyph observations; no engine acceptance implied."""
import json
import unittest
from build_research_index import ROOT

class IntermezzoGlyphTests(unittest.TestCase):
    def test_permanent_and_temporary_are_not_interchangeable(self):
        cards = json.loads((ROOT / "docs/factions/card-specifications.json").read_text())["cards"]
        observations = {'JZ40': ((1, 0, 1), (0, 0, 0)), 'JZ43': ((0, 0, 1), (0, 1, 0)), 'JZ44': ((0, 1, 1), (0, 1, 0)), 'JZ45': ((1, 1, 1), (0, 0, 0)), 'JZ46': ((0, 0, 1), (0, 0, 0)), 'JZ51': ((0, 2, 0), (1, 0, 0)), 'JZ52': ((0, 2, 2), (3, 0, 0)), 'JZ53': ((0, 0, 1), (1, 0, 1)), 'JZ59': ((0, 1, 1), (0, 0, 0)), 'JZ61': ((0, 1, 1), (0, 0, 0)), 'JZ62': ((1, 0, 0), (0, 0, 0)), 'JZ64': ((1, 1, 1), (0, 1, 0)), 'JZ70': ((0, 0, 1), (0, 1, 0)), 'JZ71': ((1, 0, 1), (0, 1, 0)), 'JZ72': ((0, 0, 1), (0, 0, 0)), 'JZ73': ((1, 0, 1), (0, 0, 0))}
        for card_id, (permanent, temporary) in observations.items():
            with self.subTest(card_id=card_id):
                fields = cards[card_id]["fields"]
                self.assertEqual(tuple(fields["permanentIcons"].values()), permanent)
                self.assertEqual(tuple(fields["temporaryIcons"].values()), temporary)
                self.assertFalse(cards[card_id]["selectionEnabled"])

    def test_street_robber_bonus_is_permanent(self):
        cards = json.loads((ROOT / "docs/factions/card-specifications.json").read_text())["cards"]
        self.assertIn("永久势力1", cards["JZ48"]["fields"]["printedRuleTextLines"][0])
