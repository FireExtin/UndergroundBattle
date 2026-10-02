"""Regression tests for the mistakes this audit must prevent, not engine fixtures."""
import copy
import itertools
import json
import unittest

from build_research_index import ROOT, build, serialize
from validate_coverage import validate, validate_groups


class CoverageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.coverage = json.loads((ROOT / "rust-game/data/faction-coverage.json").read_text())
        cls.index = json.loads((ROOT / "rust-game/data/card-research-index.json").read_text())
        cls.reviews = json.loads((ROOT / "docs/factions/card-reviews.json").read_text())
        cls.groups = json.loads((ROOT / "docs/factions/acceptance-groups.json").read_text())

    def check(self, c=None, i=None, r=None, files=False):
        return validate(c or self.coverage, i or self.index, r or self.reviews, check_files=files)[0]

    def test_checked_in_data_and_original_hashes_are_consistent(self):
        self.assertEqual(self.check(files=True), [])

    def test_generated_index_is_reproducible(self):
        self.assertEqual(serialize(build()), (ROOT / "rust-game/data/card-research-index.json").read_text())

    def test_all_28_base_pairs_include_white_and_purple_without_enabling_them(self):
        pairing = self.coverage["mixingRules"]["baseQuickPairing"]
        pairs = list(itertools.combinations(pairing["factionIds"], 2))
        self.assertEqual(len(pairs), 28)
        self.assertEqual(sum("white" in p for p in pairs), 7)
        self.assertEqual(sum("purple" in p for p in pairs), 7)
        self.assertFalse(pairing["selectionEnabled"])
        for f in self.coverage["factions"]:
            if f["id"] in {"white", "purple"}:
                self.assertEqual(f["catalogPlayerCardCount"], 0)

    def test_five_preconstructed_decks_cannot_replace_official_navigation(self):
        c = copy.deepcopy(self.coverage)
        c["factions"] = c["factions"][:5]
        self.assertTrue(any("eight" in e for e in self.check(c=c)))

    def test_lobby_theme_colors_cannot_override_official_colors(self):
        c = copy.deepcopy(self.coverage)
        next(f for f in c["factions"] if f["id"] == "green")["colorKey"] = "红"
        self.assertTrue(any("mapping" in e for e in self.check(c=c)))

    def test_unsupported_faction_or_pair_cannot_be_unlocked(self):
        c = copy.deepcopy(self.coverage)
        next(f for f in c["factions"] if f["id"] == "purple")["selectionEnabled"] = True
        c["mixingRules"]["baseQuickPairing"]["selectionEnabled"] = True
        self.assertGreaterEqual(len(self.check(c=c)), 2)

    def test_keyword_presence_and_fixture_state_cannot_promote_mechanisms(self):
        c = copy.deepcopy(self.coverage)
        for m in c["mechanisms"]:
            if m["id"] in {"shield", "seal", "renown"}:
                m["status"] = "implemented"
        self.assertTrue(any("Shield" in e for e in self.check(c=c)))
        self.assertTrue(any("Renown" in e for e in self.check(c=c)))

    def test_unreviewed_locator_text_cannot_be_called_verified(self):
        i = copy.deepcopy(self.index)
        card = next(c for c in i["records"] if c["evidenceState"] == "unreviewed")
        card["verifiedFields"] = {"name": card["locator"]["name"]}
        card["confirmedMechanismIds"] = card["candidateMechanismIds"][:1]
        self.assertTrue(any("promoted" in e for e in self.check(i=i)))

    def test_archive_card_existence_cannot_be_called_implemented(self):
        i = copy.deepcopy(self.index)
        next(c for c in i["records"] if c["id"] == "JC070")["engineStatus"] = "implemented"
        self.assertTrue(any("existence" in e for e in self.check(i=i)))

    def test_current_registration_cannot_claim_full_original_card_equivalence(self):
        i = copy.deepcopy(self.index)
        next(c for c in i["records"] if c["id"] == "JC003")["engineStatus"] = "implemented"
        self.assertTrue(any("full original-card acceptance" in e for e in self.check(i=i)))

    def test_transformed_views_and_noninitial_society_cannot_be_starting_cards(self):
        c = copy.deepcopy(self.coverage)
        next(s for s in c["societies"] if s["cardId"] == "MSJZ03")["canStartByPrintedCard"] = True
        self.assertTrue(any("Non-initial" in e for e in self.check(c=c)))
        records = {x["id"]: x for x in self.index["records"]}
        self.assertEqual(records["ZHMSJZ02"]["role"], "transformedView")
        self.assertEqual(records["ZHMSJZ02"]["formOf"], "MSJZ02")

    def test_same_names_test_labels_and_missing_originals_are_not_silently_deduplicated(self):
        records = {x["id"]: x for x in self.index["records"]}
        self.assertIn("XQ14", records["BQ040"]["sameNameRecordIds"])
        self.assertFalse(records["BQ040"]["sameNameMeansDuplicate"])
        self.assertEqual(records["BQ010"]["editionStatus"], "uncertain")
        self.assertEqual(records["TEST033"]["editionStatus"], "testImageConfirmed")
        self.assertEqual(records["TK011"]["evidenceState"], "missingImage")

    def test_research_queue_cannot_fabricate_ui_acceptance_or_unlock_cards(self):
        i = copy.deepcopy(self.index)
        i["records"][0]["selectionEnabled"] = True
        i["records"][0]["strategyUiPlaytest"] = "passed"
        self.assertTrue(any("unlock" in e for e in self.check(i=i)))
        self.assertTrue(any("fabricate" in e for e in self.check(i=i)))

    def test_group_plan_cannot_claim_release_or_replace_independent_ui_playtest(self):
        groups = copy.deepcopy(self.groups)
        groups["groups"][0]["releaseApproved"] = True
        groups["groups"][1]["strategyUiPlaytest"]["apiOnlyIsSufficient"] = True
        errors = validate_groups(groups, self.coverage, self.index)
        self.assertTrue(any("fabricate" in e for e in errors))
        self.assertTrue(any("independent UI playtest" in e for e in errors))

    def test_group_plan_cannot_hide_dependency_cycle_or_unreviewed_representative(self):
        groups = copy.deepcopy(self.groups)
        groups["groups"][0]["dependsOnGroupIds"] = [groups["groups"][0]["id"]]
        groups["groups"][1]["representativeCardIds"] = ["DQBQ01"]
        errors = validate_groups(groups, self.coverage, self.index)
        self.assertTrue(any("cycle" in e for e in errors))
        self.assertTrue(any("original image review" in e for e in errors))


if __name__ == "__main__":
    unittest.main()
