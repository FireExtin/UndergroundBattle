"""Regression tests for the mistakes this audit must prevent, not engine fixtures."""
import copy
import itertools
import json
import unittest

from build_research_index import ROOT, build, serialize
from validate_coverage import validate, validate_groups
from validate_specifications import validate_specifications


class CoverageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.coverage = json.loads((ROOT / "rust-game/data/faction-coverage.json").read_text())
        cls.index = json.loads((ROOT / "rust-game/data/card-research-index.json").read_text())
        cls.reviews = json.loads((ROOT / "docs/factions/card-reviews.json").read_text())
        cls.groups = json.loads((ROOT / "docs/factions/acceptance-groups.json").read_text())
        cls.specifications = json.loads((ROOT / "docs/factions/card-specifications.json").read_text())
        cls.recovery = json.loads((ROOT / "docs/factions/source-recovery.json").read_text())

    def check(self, c=None, i=None, r=None, files=False):
        return validate(c or self.coverage, i or self.index, r or self.reviews, check_files=files)[0]

    def test_checked_in_data_and_original_hashes_are_consistent(self):
        self.assertEqual(self.check(files=True), [])

    def test_generated_index_is_reproducible(self):
        self.assertEqual(serialize(build()), (ROOT / "rust-game/data/card-research-index.json").read_text())

    def test_rules_questions_cannot_cite_missing_evidence(self):
        specifications = copy.deepcopy(self.specifications)
        next(iter(specifications["openQuestions"].values()))["evidenceIds"] = ["missing-original-page"]
        errors = validate_specifications(specifications, self.coverage, self.index, self.recovery)
        self.assertTrue(any("Unknown question evidence" in error for error in errors))

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
        groups["groups"][1]["representativeCardIds"] = ["TK011"]
        errors = validate_groups(groups, self.coverage, self.index)
        self.assertTrue(any("cycle" in e for e in errors))
        self.assertTrue(any("original image review" in e for e in errors))

    def source_errors(self, specs=None, index=None, recovery=None):
        return validate_specifications(specs or self.specifications, self.coverage,
                                       index or self.index, recovery or self.recovery)

    def test_partial_image_review_cannot_be_counted_as_whole_card_verification(self):
        index = copy.deepcopy(self.index)
        c = next(c for c in index["records"] if c["evidenceState"] == "primaryImageReviewed"
                 and c["fullCardVerification"] == "notStarted")
        c["fullCardVerification"] = "completeGameplayFieldsForPinnedImage"
        c["fullSpecRef"] = "docs/factions/card-specifications.json#cards/" + c["id"]
        self.assertTrue(any("whole-card verification" in e for e in self.source_errors(index=index)))

    def test_unreadable_field_cannot_be_counted_as_complete_source(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSJC08"]["sourceVerification"]["status"] = "completeGameplayFieldsForPinnedImage"
        self.assertTrue(any("Blocked fields" in e for e in self.source_errors(specs=specs)))

    def test_same_name_pdf_witness_or_failed_fetch_cannot_complete_missing_version(self):
        recovery = copy.deepcopy(self.recovery)
        recovery["originalPdfWitnesses"][0]["archiveBindingConfirmed"] = True
        recovery["externalAttempts"][0]["useAsRuleEvidence"] = True
        errors = self.source_errors(recovery=recovery)
        self.assertTrue(any("same-name" in e for e in errors))
        self.assertTrue(any("Failed network" in e for e in errors))

    def test_recovered_original_cannot_use_mutable_branch_or_wrong_identity(self):
        recovery = copy.deepcopy(self.recovery)
        recovery["recoveredOriginals"][0]["sourceRepositoryCommit"] = "master"
        self.assertTrue(any("immutable public source" in e for e in self.source_errors(recovery=recovery)))
        recovery = copy.deepcopy(self.recovery)
        recovery["recoveredOriginals"][0]["cardId"] = "DQBQ02"
        self.assertTrue(any("exact ID" in e for e in self.source_errors(recovery=recovery)))

    def test_recovered_image_hash_must_match_observed_source_binding(self):
        recovery = copy.deepcopy(self.recovery)
        recovery["recoveredOriginals"][0]["sha256"] = "0" * 64
        self.assertTrue(any("bound separately" in e for e in self.source_errors(recovery=recovery)))

    def test_recovered_image_cannot_unlock_gameplay(self):
        recovery = copy.deepcopy(self.recovery)
        recovery["recoveredOriginals"][0]["selectionEnabled"] = True
        self.assertTrue(any("gameplay acceptance" in e for e in self.source_errors(recovery=recovery)))

    def test_normalizing_icons_cannot_move_source_exhaust_from_effect_to_cost(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["LC23"]["abilities"][0]["costs"] = {"exhaustSource": True}
        self.assertTrue(any("activation cost" in e for e in self.source_errors(specs=specs)))

    def test_hong_kong_and_reveal_trigger_cannot_be_simplified_into_other_events(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["DQJC116"]["abilities"][0]["order"] = ["reveal", "shuffle", "toHand"]
        specs["cards"]["JC076"]["abilities"][1]["triggersOnNormalFaceUpPlay"] = True
        errors = self.source_errors(specs=specs)
        self.assertTrue(any("Hong Kong" in e for e in errors))
        self.assertTrue(any("ordinary enter" in e for e in errors))

    def test_jc058_cannot_inherit_peek_or_normal_enter_from_other_cards(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC058"]["abilities"][0]["grantsPeekPermission"] = True
        specs["cards"]["JC058"]["abilities"][0]["triggersOnNormalFaceUpPlay"] = True
        self.assertTrue(any("reveal destruction" in e for e in self.source_errors(specs=specs)))

    def test_jc085_cannot_avoid_printed_payment_by_returning_to_hand(self):
        specs = copy.deepcopy(self.specifications)
        a = specs["cards"]["JC085"]["abilities"][1]
        a["firstMoveToHand"] = True
        a["normalPrintedCostAndLoyalty"] = False
        self.assertTrue(any("direct face-up" in e for e in self.source_errors(specs=specs)))

    def test_temporary_duration_cannot_turn_jc074_icons_into_white_icons(self):
        specs = copy.deepcopy(self.specifications)
        a = specs["cards"]["JC074"]["abilities"][0]
        a["grants"]["permanentIcons"]["investigation"] = 0
        a["duration"] = "permanent"
        self.assertTrue(any("temporary duration" in e for e in self.source_errors(specs=specs)))

    def test_jc036_cannot_recheck_current_type_and_self_destroy(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC036"]["abilities"][1]["eligibilityUsesPrintedNotCurrentSubtype"] = False
        self.assertTrue(any("printed-human" in e for e in self.source_errors(specs=specs)))

    def test_private_peek_cannot_be_public_or_lose_zero_target_route(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC057"]["abilities"][1]["privateViewer"] = "allPlayers"
        specs["cards"]["JC062"]["abilities"][0]["target"]["minimum"] = 1
        errors = self.source_errors(specs=specs)
        self.assertTrue(any("continuous private peek" in e for e in errors))
        self.assertTrue(any("zero-target" in e for e in errors))

    def test_lc19_locator_omission_cannot_remove_printed_exhaust_cost(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["LC19"]["abilities"][1]["costs"].pop("exhaustSource")
        self.assertTrue(any("source exhaust" in e for e in self.source_errors(specs=specs)))

    def test_guard_is_damage_allocation_priority_not_damage_reduction(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["LC21"]["abilities"][1]["damageReduction"] = True
        self.assertTrue(any("allocation priority" in e for e in self.source_errors(specs=specs)))

    def test_security_aura_cannot_gain_loyalty_or_exclude_friendly_teammates(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC059"]["fields"]["loyalty"] = [{"kind": "color", "value": "灰", "count": 1}]
        specs["cards"]["JC059"]["abilities"][1]["scope"]["relation"] = "ownController"
        self.assertTrue(any("friendly" in e for e in self.source_errors(specs=specs)))

    def test_world_search_must_shuffle_before_placing_chosen_card_on_top(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["DQJC107"]["abilities"][0]["order"] = ["search", "top", "shuffle"]
        self.assertTrue(any("shuffle before" in e for e in self.source_errors(specs=specs)))

    def test_fast_cards_cannot_drop_explicit_action_phase_restriction(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC091"]["abilities"][0]["fastDoesNotWaivePhaseRestriction"] = False
        self.assertTrue(any("action-phase" in e for e in self.source_errors(specs=specs)))

    def test_next_faceup_discount_cannot_apply_to_paid_reveal(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC042"]["abilities"][0]["doesNotReducePaidReveal"] = False
        self.assertTrue(any("paid reveal" in e for e in self.source_errors(specs=specs)))

    def test_global_one_damage_cannot_become_automatic_board_destruction(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["DQJC113"]["abilities"][0]["notDestroyAll"] = False
        self.assertTrue(any("one damage" in e for e in self.source_errors(specs=specs)))

    def test_tapir_sealing_and_sealed_to_bottom_have_different_cost_positions(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSJC10"]["abilities"][1]["handCardIsEffectNotCost"] = False
        specs["cards"]["MSJC10"]["abilities"][2]["sealedCardBottomIsCost"] = False
        self.assertTrue(any("activation costs" in e for e in self.source_errors(specs=specs)))

    def test_family_cannot_move_three_character_exhaustions_into_effects(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSJC12"]["abilities"][2]["threeCharacterExhaustIsCost"] = False
        self.assertTrue(any("distinct characters" in e for e in self.source_errors(specs=specs)))

    def test_society_color_cap_cannot_become_exactly_two_colors(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSJC13"]["abilities"][0]["restriction"]["maximumOtherColorCardsCombined"] = 0
        self.assertTrue(any("twelve combined" in e for e in self.source_errors(specs=specs)))

    def test_sentinel_move_cannot_drop_its_locked_anchor_target(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSJC14"]["abilities"][2]["targets"].pop()
        self.assertTrue(any("two independently" in e for e in self.source_errors(specs=specs)))

    def test_funeral_society_trigger_and_effect_cannot_be_ordinary_grave_play(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSJC16"]["abilities"][1]["notGameStartTiming"] = False
        specs["cards"]["MSJC16"]["confirmedMechanismIds"].append("gravePlay")
        self.assertTrue(any("not ordinary grave play" in e for e in self.source_errors(specs=specs)))

    def test_bottom_draw_cannot_invert_every_deck_top_reference(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSJC17"]["abilities"][1]["doesNotChangeExplicitTopReference"] = False
        self.assertTrue(any("explicit deck-top" in e for e in self.source_errors(specs=specs)))

    def test_unknown_printed_symbol_cannot_be_hidden_to_complete_a_source(self):
        specs = copy.deepcopy(self.specifications)
        source = specs["cards"]["MSWM04"]["sourceVerification"]
        source["status"] = "completeGameplayFieldsForPinnedImage"
        source["blockingFields"] = []
        self.assertTrue(any("Unresolved additional" in e for e in self.source_errors(specs=specs)))

    def test_shoggoth_trigger_cannot_be_made_optional(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSBQ02"]["abilities"][1]["optional"] = True
        self.assertTrue(any("mandatory" in e for e in self.source_errors(specs=specs)))

    def test_shoggoth_core_cannot_select_by_name_instead_of_subtitle(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSBQ02"]["abilities"][0]["restriction"]["selectedCoreCharacter"]["exactSubtitle"] = "任意"
        self.assertTrue(any("character subtitle" in e for e in self.source_errors(specs=specs)))

    def test_scarab_locator_cannot_override_original_cost_or_sealed_card_count(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSBQ03"]["abilities"][1]["costs"]["currency"] = 2
        specs["cards"]["MSBQ03"]["abilities"][2]["sealedNonCharactersDoNotCount"] = False
        errors = self.source_errors(specs=specs)
        self.assertTrue(any("costs one" in e for e in errors))
        self.assertTrue(any("per attachment" in e for e in errors))

    def test_ownership_modifier_cannot_be_limited_to_current_controlled_board(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSWM01"]["abilities"][2]["scope"]["allZones"] = False
        self.assertTrue(any("ownership across" in e for e in self.source_errors(specs=specs)))

    def test_media_discard_must_remain_cost_and_use_printed_value(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSWM02"]["abilities"][2]["effect"]["bonusEqualsDiscardedCardsPrintedCost"] = False
        self.assertTrue(any("PSC media" in e for e in self.source_errors(specs=specs)))

    def test_gang_hiding_cannot_select_a_new_character_after_paying(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSWM03"]["abilities"][2]["costsBindSpecificDiscardedAndExhaustedInstances"] = False
        self.assertTrue(any("same cost-exhausted" in e for e in self.source_errors(specs=specs)))

    def test_dream_mother_cannot_allow_neutral_or_waive_every_loyalty(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["MSWM04"]["abilities"][0]["restriction"]["neutralAllowed"] = True
        specs["cards"]["MSWM04"]["abilities"][1]["otherColorAndDomainLoyaltyNotWaived"] = False
        self.assertTrue(any("allows only purple" in e for e in self.source_errors(specs=specs)))

    def test_hand_action_does_not_cast_source_or_inherit_printed_play_loyalty(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["LC14"]["abilities"][1]["showingHandSourceDoesNotPlayIt"] = False
        self.assertTrue(any("separate play loyalty" in e for e in self.source_errors(specs=specs)))

    def test_horror_herald_condition_cannot_gain_mind_domain_or_guess_region(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["CQ12"]["abilities"][0]["condition"] = {"color": "红", "domain": "mind"}
        specs["cards"]["CQ12"]["abilities"][0]["regionAnchorUnresolved"] = False
        self.assertTrue(any("two-red" in e for e in self.source_errors(specs=specs)))

    def test_shrike_cannot_gain_an_unprinted_wound_number_or_any_death_trigger(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JZ10"]["abilities"][0]["printedWoundKeywordHasNoNumber"] = False
        specs["cards"]["JZ10"]["abilities"][1]["notAnyDeathTrigger"] = False
        self.assertTrue(any("assault-caused" in e for e in self.source_errors(specs=specs)))

    def test_shield_entry_generation_cannot_be_erased_by_fixture_only_implementation(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["BQ007"]["abilities"][0]["shieldEntryMarkerCount"] = 0
        self.assertTrue(any("shield entry generation" in e for e in self.source_errors(specs=specs)))

    def test_police_dog_cannot_gain_exhaust_cost_or_temporary_icon(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC069"]["abilities"][2]["costs"]["exhaustSource"] = True
        self.assertTrue(any("costs only two" in e for e in self.source_errors(specs=specs)))

    def test_hourglass_cannot_silently_replace_region_wording_with_attachment(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC048"]["abilities"][1]["printedDamageAndSacrificeCounterRecipient"] = "thisAttachment"
        self.assertTrue(any("silently rewritten" in e for e in self.source_errors(specs=specs)))

    def test_erasure_cannot_target_non_card_abilities(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["XQ04"]["abilities"][0]["target"]["includesNonCardAbility"] = True
        self.assertTrue(any("arbitrary abilities" in e for e in self.source_errors(specs=specs)))

    def test_lotus_cannot_widen_yin_to_yin_and_yang_cards(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["LC15"]["abilities"][3]["playedCardSubtypeAny"] = ["法术", "阴", "阳"]
        self.assertTrue(any("not yin-yang" in e for e in self.source_errors(specs=specs)))

    def test_matron_cannot_cast_private_search_as_public_or_ignore_owner(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["WM071"]["abilities"][1]["destination"] = "controllersDeck"
        self.assertTrue(any("owner shuffle" in e for e in self.source_errors(specs=specs)))

    def test_token_originals_cannot_gain_zero_cast_cost_or_player_deck_permission(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["TK001"]["fields"]["printedCost"] = 0
        self.assertTrue(any("zero-cost player cards" in e for e in self.source_errors(specs=specs)))

    def test_hourglass_must_count_other_cards_markers_in_its_region(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC048"]["abilities"][1]["regionTimeCountIncludesMarkersOnAllCharactersAndAttachmentsThere"] = False
        self.assertTrue(any("region count includes" in e for e in self.source_errors(specs=specs)))

    def test_repress_marker_choice_belongs_to_target_player(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC040"]["abilities"][0]["effect"]["markerSelectionPlayer"] = "abilityController"
        self.assertTrue(any("target chooses" in e for e in self.source_errors(specs=specs)))

    def test_kayla_cannot_treat_search_to_hand_as_draw(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC080"]["abilities"][1]["doesNotTriggerForPutIntoHand"] = False
        self.assertTrue(any("actual draw count" in e for e in self.source_errors(specs=specs)))

    def test_abyss_text_blanking_does_not_remove_ability_icons(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC101"]["abilities"][1]["carrierAbilityIconsPreserved"] = False
        self.assertTrue(any("preserving ability icons" in e for e in self.source_errors(specs=specs)))

    def test_fast_faerie_fire_cannot_skip_phase_restriction(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC102"]["abilities"][0]["allowedPhase"] = "any"
        self.assertTrue(any("action-phase restriction" in e for e in self.source_errors(specs=specs)))

    def test_nightmare_lord_cannot_gain_own_or_permanent_aura(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["JC105"]["abilities"][1]["scope"]["excludeSourceInstance"] = False
        self.assertTrue(any("other own spirits temporary" in e for e in self.source_errors(specs=specs)))

    def test_non_card_key_marker_cannot_fabricate_printed_card_fields(self):
        specs = copy.deepcopy(self.specifications)
        specs["nonCardReferences"]["TK007"]["noPrintedCardFields"] = False
        self.assertTrue(any("fabricate or override" in e for e in self.source_errors(specs=specs)))

    def test_printed_variable_defense_needs_its_source_definition(self):
        specs = copy.deepcopy(self.specifications)
        specs["cards"]["ZHJZ65"]["variablePrintedValues"] = []
        self.assertTrue(any("defined printed X" in e for e in self.source_errors(specs=specs)))


if __name__ == "__main__":
    unittest.main()
