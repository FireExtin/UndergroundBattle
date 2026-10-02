import copy
import json
import unittest
from build_research_index import ROOT
from validate_proposals import validate_proposals


class ProposalTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        def read(path):
            return json.loads((ROOT / path).read_text())
        cls.proposals = read('docs/factions/deck-proposals.json')
        cls.rulings = read('docs/factions/urgent-playtest-rulings.json')
        cls.specifications = read('docs/factions/card-specifications.json')
        cls.coverage = read('rust-game/data/faction-coverage.json')

    def check(self, proposals=None, rulings=None):
        return validate_proposals(proposals or self.proposals, rulings or self.rulings, self.specifications, self.coverage)

    def test_disabled_full_source_candidates_and_world_inventory_are_consistent(self):
        self.assertEqual(self.check(), [])

    def test_neutral_filler_cannot_be_substituted_for_verified_faction_card(self):
        p = copy.deepcopy(self.proposals)
        p['decks'][0]['entries'][0].update(cardId='JC125', name='无知路人', colorKey='中立')
        self.assertTrue(any('neutral-color' in error for error in self.check(proposals=p)))

    def test_source_complete_deck_cannot_be_unlocked_without_game_and_ui_acceptance(self):
        p = copy.deepcopy(self.proposals)
        p['decks'][0]['selectionEnabled'] = True
        self.assertTrue(any('cannot unlock' in error for error in self.check(proposals=p)))

    def test_two_distinct_entity_ids_do_not_allow_same_definition_twice_in_base_inventory(self):
        r = copy.deepcopy(self.rulings)
        r['officialBaseWorld']['entries'][0] = dict(r['officialBaseWorld']['entries'][6])
        self.assertTrue(any('ten distinct single physical cards' in error for error in self.check(rulings=r)))


if __name__ == '__main__':
    unittest.main()
