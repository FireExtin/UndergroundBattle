"""Whole source completion with explicit authorized source holdbacks."""
import json
import unittest
from build_research_index import ROOT

class SourceCompletionTests(unittest.TestCase):
    def test_all_readable_records_are_complete_and_holds_are_explicit(self):
        index = json.loads((ROOT / "rust-game/data/card-research-index.json").read_text())
        records = {r["id"]: r for r in index["records"]}
        self.assertEqual(index["totals"]["completeSourceRecords"], 679)
        self.assertEqual(index["totals"]["unreviewedWithImage"], 0)
        blocked = {i for i, r in records.items() if r["sourceRecordVerification"] == "blocked"}
        self.assertEqual(blocked, {"MSJC08", "MSJC09", "JC108", "MSWM04", "DQWM001"})
        self.assertEqual({i for i, r in records.items() if r["sourceRecordVerification"] == "notStarted"}, {"TK011"})
        self.assertEqual(records["TK007"]["sourceRecordVerification"], "completeNonCardReference")
        self.assertTrue(all(not r["selectionEnabled"] for r in records.values()))

    def test_unresolved_name_reference_does_not_erase_source_text(self):
        cards = json.loads((ROOT / "docs/factions/card-specifications.json").read_text())["cards"]
        self.assertEqual(cards["MSWM04"]["fields"]["name"], "梦魇主母")
        self.assertIn("梦魔主母", cards["XG32"]["fields"]["printedRuleTextLines"][0])
        self.assertIn("NightmareMatriarchNameVariant", cards["XG32"]["openQuestionIds"])
