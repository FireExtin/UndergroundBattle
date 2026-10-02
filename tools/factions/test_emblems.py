"""Keep observed society-emblem variants bound to official colors and primary images."""
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]


def errors_for(variants, coverage, specifications):
    errors = []
    societies = {x['cardId']: x for x in coverage['societies']}
    if variants.get('selectionEnabled') is not False or variants.get('notAdditionalColors') is not True:
        errors.append('Emblem evidence cannot unlock colors or gameplay')
    for variant in variants['variants']:
        if variant['selectionEnabled'] is not False:
            errors.append('Variant cannot unlock gameplay: ' + variant['id'])
        for card_id in variant['societyCardIds']:
            society = societies.get(card_id, {})
            if society.get('colorKey') != variant['colorKey']:
                errors.append('Society color differs: ' + card_id)
            spec = specifications['cards'].get(card_id, {})
            if spec.get('fields', {}).get('colorKey') != variant['colorKey']:
                errors.append('Observed color differs: ' + card_id)
            if spec.get('sourceVerification', {}).get('status') != 'completeGameplayFieldsForPinnedImage':
                errors.append('Society source is not complete: ' + card_id)
        for evidence_id in variant['evidenceIds']:
            if coverage['evidence'].get(evidence_id, {}).get('visuallyReviewed') is not True:
                errors.append('Unreviewed emblem evidence: ' + evidence_id)
    return errors


class EmblemTests(unittest.TestCase):
    def setUp(self):
        self.variants = json.loads((ROOT / 'docs/factions/emblem-variants.json').read_text())
        self.coverage = json.loads((ROOT / 'rust-game/data/faction-coverage.json').read_text())
        self.specifications = json.loads((ROOT / 'docs/factions/card-specifications.json').read_text())

    def test_primary_societies_anchor_existing_color_variants(self):
        self.assertEqual(errors_for(self.variants, self.coverage, self.specifications), [])

    def test_emblem_cannot_change_color_or_unlock_selection(self):
        self.variants['variants'][0]['colorKey'] = '白'
        self.variants['variants'][1]['selectionEnabled'] = True
        errors = errors_for(self.variants, self.coverage, self.specifications)
        self.assertTrue(any('color differs' in e for e in errors))
        self.assertTrue(any('unlock gameplay' in e for e in errors))
