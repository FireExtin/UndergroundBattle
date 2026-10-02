"""Validate source-completion claims separately from engine and UI acceptance."""
from __future__ import annotations

FIELDS = {
    "name", "subtitle", "nameInk", "colorKey", "basicType", "subtypes",
    "printedCollectorCode", "seriesSymbol", "printedCost", "loyalty", "domains",
    "permanentIcons", "temporaryIcons", "defense", "startingHand",
    "influenceThreshold", "points", "printedKeywords", "printedRuleTextLines", "additionalPrintedSymbols",
}
COMPLETE = "completeGameplayFieldsForPinnedImage"


def validate_specifications(specifications, coverage, index, recovery):
    errors = []
    specs = specifications["cards"]
    records = {c["id"]: c for c in index["records"]}
    evidence = coverage["evidence"]
    mechanism_ids = {m["id"] for m in coverage["mechanisms"]}
    completed = set()
    blocked = set()

    def require(ok, message):
        if not ok:
            errors.append(message)

    require(specifications["schemaVersion"] == recovery["schemaVersion"] == 1,
            "Unknown specification or recovery schema")
    require(specifications["auditedCommit"] == coverage["auditedCommit"], "Specification baseline differs")
    require(set(specs) <= set(records), "Full specification has no archive identity")
    for question_id, question in specifications["openQuestions"].items():
        require(set(question.get("evidenceIds", [])) <= set(evidence),
                "Unknown question evidence: " + question_id)
    for i, s in specs.items():
        source = s["sourceVerification"]
        fields = s["fields"]
        status = source["status"]
        require(s["cardId"] == i and FIELDS <= set(fields), "Incomplete field inventory: " + i)
        require(fields["colorKey"] in {None, "黄", "绿", "蓝", "红", "灰", "白", "黑", "紫", "中立"},
                "Unknown or noncanonical source color key: " + i)
        require(status in {COMPLETE, "blocked"}, "Unknown source completion status: " + i)
        require(source["basis"] == "directOriginalImageReview", "Locator cannot complete a source specification: " + i)
        e = evidence.get(source["evidenceId"], {})
        require(e.get("path") == source["imagePath"] and e.get("sha256") == source["imageSha256"]
                and e.get("visuallyReviewed") is True, "Specification image pin differs from reviewed original: " + i)
        require({"path": source["imagePath"], "sha256": source["imageSha256"]} in records[i]["sourceImages"],
                "A different original variant cannot silently replace the indexed image: " + i)
        if status == COMPLETE:
            completed.add(i)
            require(not source["blockingFields"], "Blocked fields cannot be counted as complete: " + i)
            require(all(x["status"] == "confirmed" for x in fields["additionalPrintedSymbols"]),
                    "Unresolved additional printed symbol cannot complete source fields: " + i)
        else:
            blocked.add(i)
            require(bool(source["blockingFields"]), "Blocked specification must identify unknown fields: " + i)
        require(set(source["blockingFields"]) <= FIELDS, "Unknown blocking field: " + i)
        require(bool(fields["printedCollectorCode"])
                and s["edition"]["printedCollectorCode"] == fields["printedCollectorCode"]
                and s["edition"]["releaseYearClaimed"] is False,
                "Print identity must be observed, not guessed from locator or release year: " + i)
        for key in ["printedCost", "defense", "startingHand", "influenceThreshold", "points"]:
            value = fields[key]
            symbolic = value == "X" and any(x.get("field") == key and x.get("symbol") == "X"
                                            and x.get("definitionText") for x in s.get("variablePrintedValues", []))
            require(value is None or type(value) is int and value >= 0 or symbolic,
                    "Numeric field must be verified integer, defined printed X or explicit N/A: " + i)
        for key in ["permanentIcons", "temporaryIcons"]:
            require(set(fields[key]) == {"investigation", "combat", "influence"}
                    and all(type(v) is int and v >= 0 for v in fields[key].values()), "Invalid icon vector: " + i)
        require(bool(fields["printedRuleTextLines"]) and bool(s["abilities"]), "Missing printed rules/ability inventory: " + i)
        require(set(s["confirmedMechanismIds"]) <= mechanism_ids
                and set(s["rulesEvidenceIds"]) <= set(evidence), "Unknown rule or mechanism evidence: " + i)
        require(set(s["openQuestionIds"]) <= set(specifications["openQuestions"]), "Unknown card-level rules question: " + i)
        require(not s["selectionEnabled"] and s["gameAcceptance"] == "notAccepted"
                and s["strategyUiPlaytest"] == "notRun", "Source review cannot approve gameplay or UI: " + i)
        require(all(x["status"] == "notRun" for x in s["acceptanceCases"]), "Do not fabricate game-test results: " + i)
        require(records[i]["fullCardVerification"] == status
                and records[i]["fullSpecRef"] == "docs/factions/card-specifications.json#cards/" + i
                and records[i]["sourceRecordVerification"] == status
                and records[i]["sourceSpecificationKind"] == "printedCard",
                "Index completion claim differs from manual full specification: " + i)

    for i, r in records.items():
        if i not in specs:
            require(r["fullCardVerification"] == "notStarted" and r["fullSpecRef"] is None,
                    "Partial image review or locator must not count as whole-card verification: " + i)
            if i not in specifications.get("nonCardReferences", {}):
                require(r["sourceRecordVerification"] == "notStarted" and r["sourceSpecRef"] is None,
                        "Unreviewed or partial records cannot gain whole-source completion: " + i)
    require(index["totals"]["completeSourceSpecifications"] == len(completed)
            and index["totals"]["blockedSourceSpecifications"] == len(blocked), "Full-source counts differ")
    references = specifications.get("nonCardReferences", {})
    for i, reference in references.items():
        source = reference["sourceVerification"]
        r = records.get(i, {})
        e = evidence.get(source["evidenceId"], {})
        require(i == "TK007" and i not in specs and r.get("role") == "markerReference"
                and reference["artifactKind"] == "nonCardKeyMarker" and reference["noPrintedCardFields"] is True,
                "Non-card reference must not fabricate or override a printed card: " + i)
        require(source["status"] == "completeNonCardReference" and source["basis"] == "originalImageAndOriginalRulebook"
                and source["imagePath"] == e.get("path") and source["imageSha256"] == e.get("sha256")
                and e.get("visuallyReviewed") is True and set(reference["rulesEvidenceIds"]) <= set(evidence),
                "Non-card reference needs its original graphic and reviewed rulebook: " + i)
        require(reference["labelBasis"] == "originalRulebookDefinition" and not reference["selectionEnabled"]
                and r.get("sourceRecordVerification") == source["status"]
                and r.get("sourceSpecRef") == "docs/factions/card-specifications.json#nonCardReferences/" + i,
                "Marker label is a rulebook definition, never a printed card name or playable choice: " + i)
    require(index["totals"]["completeNonCardReferences"] == len(references)
            and index["totals"]["completeSourceRecords"] == len(completed) + len(references),
            "Card and non-card completion totals must remain separate")
    batch_complete = set()
    batch_blocked = set()
    for b in specifications["batches"]:
        require(set(b["completedCardIds"]) | set(b["blockedCardIds"]) == set(b["cardIds"]), "Batch card inventory incomplete")
        require(not b["gameImplementationAccepted"] and b["independentUiPlaytestStatus"] == "notRun",
                "Source batch cannot attest to implementation acceptance")
        batch_complete.update(b["completedCardIds"])
        batch_blocked.update(b["blockedCardIds"])
    require(batch_complete == completed and batch_blocked == blocked, "Batch completion counts differ from source fields")
    missing = {i for i, r in records.items() if not r["sourceImages"]}
    require({x["cardId"] for x in recovery["quarantinedRecords"]} == missing, "Missing-original quarantine differs from actual files")
    require(all(not x["selectionEnabled"] and x["fullSourceVerification"] == "blocked"
                for x in recovery["quarantinedRecords"]), "Missing originals cannot be released or fully verified")
    require(all(not x["archiveBindingConfirmed"] for x in recovery["originalPdfWitnesses"]),
            "A same-name PDF example cannot silently bind a missing archive version")
    require(all(not x["useAsRuleEvidence"] for x in recovery["externalAttempts"]),
            "Failed network recovery cannot become rule evidence")
    for unresolved in recovery.get("unresolvedPrintedSymbols", []):
        i = unresolved["cardId"]
        require(i in specs and specs[i]["sourceVerification"]["status"] == "blocked"
                and "additionalPrintedSymbols" in specs[i]["sourceVerification"]["blockingFields"]
                and unresolved["interpretation"] is None and not unresolved["selectionEnabled"]
                and unresolved["failedSearchIsNotRuleEvidence"],
                "Unresolved printed-symbol recovery must preserve source quarantine: " + i)
    # Specific semantics where normalizing icons/costs commonly changes a card.
    lc = specs["LC23"]["abilities"][0]
    require(lc["costs"] == {} and lc["sourceExhaustIsEffectNotCost"] is True,
            "LC23 source exhaust is an effect, never an added activation cost")
    require(specs["JC049"]["abilities"][0]["effect"]["countsAsDraw"] is False,
            "JC049 bottom-to-hand is not a draw")
    hk = specs["DQJC116"]["abilities"][0]
    require(hk["order"] == ["allPlayersCommitPrivateSearch", "simultaneousReveal",
                            "simultaneousMoveToEachOwnersHand", "shuffleEachPlayersDeck"],
            "Hong Kong cannot reveal early or shuffle before simultaneous hand entry")
    require(specs["JC076"]["abilities"][1]["triggersOnNormalFaceUpPlay"] is False,
            "JC076 reveal trigger is not an ordinary enter trigger")
    if "JC058" in specs:
        ability = specs["JC058"]["abilities"][0]
        require(ability["triggersOnNormalFaceUpPlay"] is False and ability["grantsPeekPermission"] is False,
                "JC058 has reveal destruction, not normal enter or peek permission")
        grave = specs["JC085"]["abilities"][1]
        require(grave["sourceZone"] == "controllersGraveyard" and grave["normalPrintedCostAndLoyalty"]
                and grave["faceUpOnly"] and not grave["firstMoveToHand"],
                "JC085 grave play is direct face-up normal-cost play, not return to hand or secret deploy")
        renown = specs["JC074"]["abilities"][0]
        require(renown["duration"] == "untilEndOfTurn"
                and renown["grants"]["permanentIcons"] == {"investigation": 2, "combat": 0, "influence": 0},
                "JC074 grants two permanent investigation icons for a temporary duration")
        control = specs["JC036"]["abilities"][1]
        require(control["eligibilityUsesPrintedNotCurrentSubtype"] and control["otherSubtypesPreserved"]
                and control["singleControlSourceRemovalRestores"] == "owner",
                "JC036 printed-human eligibility, other types and owner restoration must be preserved")
        peek = specs["JC057"]["abilities"][1]
        require(peek["privateViewer"] == "sourceController" and not peek["entersStack"]
                and not peek["targeted"] and peek["hiddenIdentityNotReplaced"]
                and peek["requires"]["sourceUnexhausted"] and peek["requires"]["sameRegion"],
                "JC057 continuous private peek cannot become public reveal, targeted action or permanent access")
        targets = specs["JC062"]["abilities"][0]["target"]
        require(targets["minimum"] == 0 and targets["maximum"] == 2,
                "JC062 up-to-two targets must retain the zero-target draw route")
        heal = specs["LC19"]["abilities"][1]
        require(heal["costs"] == {"currency": 2, "exhaustSource": True}
                and heal["notOrdinaryTurnDamage"], "LC19 original includes source exhaust and removes wounds, not turn damage")
    if "JC125" in specs:
        for i in ["LC21", "LC22"]:
            guard = specs[i]["abilities"][1]
            require(guard["requiresAssignAtLeastXBeforeOtherCharacters"] and not guard["damageReduction"]
                    and guard["doesNotChangeNonCombatDamage"], "Guard is combat allocation priority, not damage reduction: " + i)
        security = specs["JC059"]
        aura = security["abilities"][1]
        require(not security["fields"]["loyalty"] and aura["scope"]["relation"] == "friendly"
                and aura["scope"]["excludeSource"] and aura["teamsFriendlyIncludesTeammate"],
                "JC059 has zero printed loyalty and affects other friendly characters, including teammates")
        require(specs["DQJC107"]["abilities"][0]["order"] == ["eachPlayerSearchOneOwnDeck",
                    "shuffleEachPlayersRemainingDeck", "chosenCardOnEachOwnersDeckTop"],
                "Sunken ruins must shuffle before putting the chosen card on top")
        for i in ["JC118", "JC091"]:
            fast = specs[i]["abilities"][0]
            require(fast["playRestriction"] == {"phase": "action"} and fast["fastDoesNotWaivePhaseRestriction"],
                    "Fast timing must retain an explicit action-phase restriction: " + i)
        discount = specs["JC042"]["abilities"][0]
        require(discount["doesNotReducePaidReveal"] and discount["eligiblePlay"] == {
                    "faceUpOnly": True, "eitherColor": "红", "orDomain": "blood"},
                "JC042 next matching face-up discount cannot apply to paid reveal")
        nuclear = specs["DQJC113"]["abilities"][0]
        require(nuclear["damage"] == 1 and nuclear["notDestroyAll"] and not nuclear["targeted"],
                "Chernobyl deals one damage; it does not automatically destroy all characters")
    if "MSJC10" in specs:
        for i in ["MSJC02", "MSJC03", "MSJC04", "MSJC05", "MSJC06", "MSJC07"]:
            abilities = specs[i]["abilities"]
            require(abilities[1]["costs"] == {"currency": 3, "exhaustSource": True}
                    and abilities[2]["costs"] == {"currency": 4, "exhaustSource": True}
                    and abilities[2]["search"]["goldNameUnique"],
                    "Base society actions retain source-exhaust costs and unique-card search: " + i)
        tapir = specs["MSJC10"]["abilities"]
        require(tapir[1]["handCardIsEffectNotCost"] and tapir[1]["costs"] == {"currency": 2, "exhaustSource": True}
                and tapir[2]["sealedCardBottomIsCost"] and tapir[3]["sealedCardBottomIsCost"]
                and tapir[2]["costs"]["thisSocietySealedCardsToOwnersDeckBottom"] == 1
                and tapir[3]["costs"]["thisSocietySealedCardsToOwnersDeckBottom"] == 2,
                "MSJC10 hand sealing is an effect; sealed-to-bottom movements are activation costs")
        family = specs["MSJC12"]["abilities"]
        require(family[1]["scope"]["printedCostMaximum"] == 2 and family[1]["otherSubtypesPreserved"]
                and all(a["threeCharacterExhaustIsCost"] and a["costs"]["exhaustDistinctControlledCharacters"]["count"] == 3
                        for a in family[2:]),
                "MSJC12 reads printed cost and requires three distinct characters exhausted as costs")
        for i, color in [("MSJC13", "红"), ("MSJC15", "白")]:
            r = specs[i]["abilities"][0]["restriction"]
            require(r == {"uncappedColors": [color, "中立"], "maximumOtherColorCardsCombined": 12},
                    "Other-color cap is twelve combined, not a strict two-color ban or per-color allowance: " + i)
        sentinel = specs["MSJC14"]["abilities"][2]
        require(len(sentinel["targets"]) == 2 and sentinel["destination"] == "currentRegionOfLockedAnchor",
                "MSJC14 movement has two independently guarded targets")
        funeral = specs["MSJC16"]["abilities"]
        require(funeral[1]["kind"] == "optionalFirstTurnPreparationStartTrigger" and funeral[1]["notGameStartTiming"]
                and funeral[3]["sevenSealsIsCostNotEffect"]
                and funeral[3]["costs"]["sealOwnGraveyardCardsOnThisSociety"] == 7
                and "gravePlay" not in specs["MSJC16"]["confirmedMechanismIds"],
                "MSJC16 first-turn trigger and seven-seal cost are not ordinary grave play")
        dreams = specs["MSJC17"]["abilities"]
        require(dreams[1]["drawOrigin"] == "deckBottom" and dreams[1]["doesNotReplaceSearchToHand"]
                and dreams[1]["doesNotChangeExplicitTopReference"] and dreams[2]["source"] == "controllerDeckTop",
                "MSJC17 bottom draw must retain explicit deck-top and search-to-hand operations")
    if "MSBQ01" in specs:
        forced = specs["MSBQ02"]["abilities"][1]
        require(forced["kind"] == "forcedPlayedOrPaidRevealTrigger" and not forced["optional"]
                and forced["maximumLossControlMarkers"] == 3, "Shoggoth red-play loss-of-control trigger is mandatory and capped at three")
        core = specs["MSBQ02"]["abilities"][0]["restriction"]["selectedCoreCharacter"]
        require(core == {"printedCategory": "characterCard", "exactSubtitle": "修格斯"},
                "Shoggoth core selection is by character subtitle, not name or arbitrary core icon")
        scarab = specs["MSBQ03"]["abilities"]
        require(scarab[1]["costs"] == {"currency": 1, "exhaustSource": True}
                and scarab[1]["corpseIsEffectSelectionNotCost"],
                "Scarab first action costs one and source exhaust; corpse sealing is an effect")
        require(scarab[2]["perCarrierSealedPrintedCharacterCount"] and scarab[2]["sealedNonCharactersDoNotCount"],
                "Scarab influence counts sealed characters per attachment, not every sealed card")
        specter = specs["MSWM01"]["abilities"][2]
        require(specter["scope"]["owner"] == "abilityController" and specter["scope"]["allZones"]
                and specter["duration"] == "thisGame", "Mannu specter modification follows ownership across all zones for this game")
        psc = specs["MSWM02"]["abilities"][2]
        require(psc["discardIsCostNotEffect"] and psc["effect"]["bonusEqualsDiscardedCardsPrintedCost"],
                "PSC media discard is a cost and mystic bonus uses printed cost")
        gang = specs["MSWM03"]["abilities"][2]
        require(gang["costsBindSpecificDiscardedAndExhaustedInstances"] and gang["hideIsEffectNotCost"],
                "Cold mountain hiding refers to the same cost-exhausted instance")
        mother = specs["MSWM04"]["abilities"]
        require(mother[0]["restriction"] == {"allowedColors": ["紫"], "neutralAllowed": False}
                and mother[1]["otherColorAndDomainLoyaltyNotWaived"] and mother[3]["costs"] == {"currency": 0}
                and mother[3]["remainingLoyaltyRequirementsPreserved"],
                "Dream mother allows only purple and waives card currency cost without other loyalty")
    if "LC14" in specs:
        metatron = specs["LC14"]
        hand = metatron["abilities"][1]
        require(metatron["fields"]["domains"] == [{"id": "holy", "count": 4}]
                and not metatron["fields"]["loyalty"] and hand["activationStep"] == "preparation"
                and hand["costs"] == {"publiclyShowThisSourceFromOwnHand": True, "currency": 2}
                and hand["showingHandSourceDoesNotPlayIt"],
                "LC14 hand action shows the hand source for two in preparation; four holy is its separate play loyalty")
        horror = specs["CQ12"]["abilities"]
        require(horror[0]["condition"] == {"controllersAssetColor": "红", "minimum": 2}
                and horror[0]["regionAnchorUnresolved"] and horror[1]["appliesToBothNormalPlayAndPaidReveal"],
                "CQ12 has two-red hand condition, unresolved hand-region anchor and an explicit paid-reveal reduction")
        widow = specs["JZ10"]["abilities"]
        require(widow[0]["printedWoundKeywordHasNoNumber"] and widow[1]["notAnyDeathTrigger"]
                and widow[1]["requiresDeathCausedByThisSourcesAssault"],
                "JZ10 wound keyword has no number and move-hide requires its own assault-caused death")
        shield = specs["BQ007"]["abilities"][0]
        require(shield["shieldEntryMarkerCount"] == 1 and shield["shieldProtectionIsForcedEnemyTargetTrigger"],
                "BQ007 provides shield entry generation and forced enemy-target protection, not just fixture consumption")
        dog = specs["JC069"]["abilities"][2]
        require(dog["costs"] == {"currency": 2} and dog["noSourceExhaustCostPrinted"]
                and dog["order"][-1] == "grantSourceOnePermanentInvestigationUntilEndOfTurn",
                "JC069 costs only two and grants a permanent investigation icon for a temporary duration")
        hourglass = specs["JC048"]["abilities"][1]
        require(hourglass["printedAddMarkerRecipient"] == "thisAttachment"
                and hourglass["printedDamageAndSacrificeCounterRecipient"] == "carrierRegion"
                and not hourglass["recipientMismatchUnresolved"]
                and hourglass["regionTimeCountIncludesMarkersOnAllCharactersAndAttachmentsThere"]
                and hourglass["markersRemainPhysicallyOnOriginalRecipient"]
                and "manual-marker-inheritance" in specs["JC048"]["rulesEvidenceIds"],
                "JC048 region count includes attached markers; printed recipients must not be silently rewritten")
        cancel = specs["XQ04"]["abilities"][0]["target"]
        require(cancel["category"] == "stackCard" and cancel["includesPaidReveal"]
                and not cancel["includesNonCardAbility"], "XQ04 terminates stack cards, not arbitrary abilities")
        lotus = specs["LC15"]["abilities"]
        require(lotus[3]["playedCardSubtypeAny"] == lotus[4]["cardSubtypeAny"] == ["法术", "阴"]
                and lotus[2]["optional"] is False and lotus[4]["countsAsDraw"] is False,
                "LC15 says spells or yin cards, not yin-yang; enter sealing is forced and grave recovery is not draw")
        matron = specs["WM071"]["abilities"]
        require(matron[1]["destination"] == "ownersDeck" and matron[1]["graveEntryEventOrderUnresolved"]
                and matron[2]["search"]["category"] == "characterCard" and matron[2]["explicitPublicRevealNotPrinted"],
                "WM071 preserves owner shuffle, unresolved transition events and private character-only search")
    if "TK001" in specs:
        token_ids = {"TK001", "TK002", "TK003", "TK005", "TK006", "TK008", "TK009"}
        require(all(specs[i]["fields"]["printedCost"] is None
                    and specs[i]["fields"]["loyalty"] == []
                    and specs[i]["fields"]["printedCollectorCode"] == "T"
                    and specs[i]["constructionRole"] == "generatedTokenNotPlayerDeckCard" for i in token_ids),
                "Token originals have no printed cast cost or loyalty; do not turn them into zero-cost player cards")
        require("TK007" not in specs, "A key marker graphic cannot fabricate a complete printed card")
        repress = specs["JC040"]["abilities"][0]["effect"]
        require(repress["markerSelectionPlayer"] == repress["markerOwner"] == "targetPlayer"
                and repress["scope"] == "allInPlay" and repress["targetNotRequiredToBeDiscardingPlayer"],
                "JC040 repress target chooses its own all-board markers; triggering discard does not bind that target")
        require(specs["JC080"]["abilities"][1]["doesNotTriggerForPutIntoHand"]
                and specs["JC080"]["abilities"][1]["event"]["count"] == "actualCardsDrawnInThisEvent",
                "JC080 uses actual draw count, not every hand entry or a fixed one")
        curse = specs["JC101"]["abilities"][1]
        require(curse["blankCarrierPrintedRulesText"] and curse["removeCarrierAllMagicDomainIcons"]
                and curse["carrierAbilityIconsPreserved"] and curse["doesNotBlankAllExternalGrantedEffects"],
                "JC101 blanks printed text and domains, preserving ability icons and external grants")
        require(specs["JC102"]["abilities"][0]["allowedPhase"] == "action",
                "JC102 fast timing cannot bypass the printed action-phase restriction")
        bonus = specs["JC105"]["abilities"][1]
        require(bonus["grantTemporaryIcons"] == {"investigation": 0, "combat": 0, "influence": 1}
                and bonus["scope"]["excludeSourceInstance"]
                and bonus["scope"]["controller"] == "sourceController",
                "JC105 grants other own spirits temporary influence, not itself, permanent icons or teammates")
    return errors
