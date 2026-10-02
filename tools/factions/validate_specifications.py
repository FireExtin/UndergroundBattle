"""Validate source-completion claims separately from engine and UI acceptance."""
from __future__ import annotations

FIELDS = {
    "name", "subtitle", "nameInk", "colorKey", "basicType", "subtypes",
    "printedCollectorCode", "seriesSymbol", "printedCost", "loyalty", "domains",
    "permanentIcons", "temporaryIcons", "defense", "startingHand",
    "influenceThreshold", "points", "printedKeywords", "printedRuleTextLines",
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
    for i, s in specs.items():
        source = s["sourceVerification"]
        fields = s["fields"]
        status = source["status"]
        require(s["cardId"] == i and FIELDS <= set(fields), "Incomplete field inventory: " + i)
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
            require(value is None or type(value) is int and value >= 0, "Numeric field must be verified integer or explicit N/A: " + i)
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
                and records[i]["fullSpecRef"] == "docs/factions/card-specifications.json#cards/" + i,
                "Index completion claim differs from manual full specification: " + i)

    for i, r in records.items():
        if i not in specs:
            require(r["fullCardVerification"] == "notStarted" and r["fullSpecRef"] is None,
                    "Partial image review or locator must not count as whole-card verification: " + i)
    require(index["totals"]["completeSourceSpecifications"] == len(completed)
            and index["totals"]["blockedSourceSpecifications"] == len(blocked), "Full-source counts differ")
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
    return errors
