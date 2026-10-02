#!/usr/bin/env python3
"""Validate research provenance, concept separation and fail-closed availability.

This is not a rules engine, PDF parser, OCR verifier or game acceptance test.
--allow-code-drift reports stale source anchors as warnings; it never enables cards.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
from collections import Counter
from pathlib import Path

from build_research_index import ROOT, build, serialize

STATUSES = {"implemented", "partial", "unimplemented", "uncertain"}
OFFICIAL = {
    "yellow": ("帷幕守望", "黄"), "green": ("猎魔人", "绿"),
    "blue": ("王座会", "蓝"), "red": ("鸣钟教派", "红"),
    "gray": ("国家机构", "灰"), "white": ("圣贤", "白"),
    "black": ("方碑序列", "黑"), "purple": ("梦境行者", "紫"),
}


def validate_groups(groups: dict, coverage: dict, index: dict) -> list[str]:
    """Planning assertions; these do not attest to execution of any acceptance gate."""
    errors = []
    items = {g["id"]: g for g in groups["groups"]}
    mechanisms = {m["id"] for m in coverage["mechanisms"]}
    records = {r["id"]: r for r in index["records"]}
    questions = {q["id"] for q in coverage["openQuestions"]}
    if groups["schemaVersion"] != 1 or groups["auditedCommit"] != coverage["auditedCommit"]:
        errors.append("Acceptance planning schema or source baseline differs")
    if not groups["allGroupsAreUnaccepted"] or not groups["sourceReviewRequiredBeforeImplementation"]:
        errors.append("Planning data cannot declare acceptance or skip primary review")
    if len(items) != len(groups["groups"]):
        errors.append("Duplicate acceptance group IDs")
    if not set(groups["rulesUnknownBlockers"]) <= questions:
        errors.append("Acceptance group refers to unknown rules blockers")
    for g in items.values():
        if g["status"] != "planned" or g["releaseApproved"] or g["representativesAreCompleteSpecs"]:
            errors.append("Planning group cannot fabricate a release or complete card specification: " + g["id"])
        if not set(g["mechanismIds"]) <= mechanisms or not set(g["dependsOnGroupIds"]) <= set(items):
            errors.append("Acceptance group has unknown dependencies: " + g["id"])
        if not g["representativeCardIds"] or any(
            i not in records or records[i]["evidenceState"] != "primaryImageReviewed"
            for i in g["representativeCardIds"]
        ):
            errors.append("Acceptance representatives must have original image review: " + g["id"])
        required_gates = {"completePrimarySourceSpec", "engineAndPersistenceRegression",
                          "legalActionsAndPrivateProjection", "usableUiChoices",
                          "independentStrategyUiPlaytest", "feedbackFixAndRegression",
                          "mainImplementationOwnerReview"}
        if not required_gates <= set(g["gates"]):
            errors.append("Acceptance group omits a required evidence gate: " + g["id"])
        playtest = g["strategyUiPlaytest"]
        if (playtest["tool"] != "dot" or playtest["minimumIndependentTesters"] != 2
                or playtest["maximumIndependentTesters"] != 4
                or playtest["style"] != "humanStyleActualUi"
                or playtest["status"] != "notRun" or playtest["apiOnlyIsSufficient"]
                or not playtest["feedbackFixAndRegressionRequired"]):
            errors.append("Planning must retain the requested independent UI playtest and no fabricated result: " + g["id"])

    def visit(i: str, active: set[str], done: set[str]) -> None:
        if i in active:
            errors.append("Acceptance group dependency cycle: " + i)
            return
        if i in done:
            return
        for dep in items[i]["dependsOnGroupIds"]:
            if dep in items:
                visit(dep, active | {i}, done)
        done.add(i)

    done: set[str] = set()
    for i in items:
        visit(i, set(), done)
    return errors


def validate(coverage: dict, index: dict, reviews: dict, root: Path = ROOT,
             check_files: bool = True, allow_code_drift: bool = False) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []

    def require(condition: bool, message: str) -> None:
        if not condition:
            errors.append(message)

    evidence = coverage["evidence"]
    mechanisms = {m["id"]: m for m in coverage["mechanisms"]}
    factions = {f["id"]: f for f in coverage["factions"]}
    societies = {s["cardId"]: s for s in coverage["societies"]}
    records = {c["id"]: c for c in index["records"]}
    manual = reviews["cards"]
    require(coverage["schemaVersion"] == index["schemaVersion"] == reviews["schemaVersion"] == 1,
            "Unsupported research schema version")
    require(bool(re.fullmatch(r"[a-f0-9]{40}", coverage["auditedCommit"])), "Missing audited commit SHA")
    require(index["auditedCommit"] == coverage["auditedCommit"], "Index and coverage commits differ")
    require(len(factions) == len(coverage["factions"]), "Duplicate faction IDs")
    require(len(societies) == len(coverage["societies"]), "Duplicate society IDs")
    require(len(mechanisms) == len(coverage["mechanisms"]), "Duplicate mechanism IDs")
    require(len(records) == len(index["records"]), "Duplicate research card IDs")
    require({f["id"] for f in factions.values() if f["kind"] == "colorFaction"} == set(OFFICIAL),
            "Official color navigation must cover all eight verified factions")
    for faction_id, (name, color) in OFFICIAL.items():
        f = factions.get(faction_id, {})
        require((f.get("name"), f.get("colorKey")) == (name, color),
                f"Wrong official color/name mapping: {faction_id}")
    require(factions.get("neutral", {}).get("kind") == "neutral", "Neutral must be separate from eight colors")
    require(factions.get("neutral", {}).get("colorName") == "褐色", "Neutral display color is brown")
    require(all(not f["selectionEnabled"] for f in factions.values()), "Official navigation cannot unlock factions")
    require(all(not f["themeIsExclusive"] for f in factions.values()), "Faction themes must not be exclusive rules")
    require(all(not s["selectionEnabled"] and s["status"] == "unimplemented" for s in societies.values()),
            "Society metadata cannot enable unimplemented society cards")
    expected_societies = {f"MSJC{i:02}" for i in range(1, 18)} | {
        f"MSBQ{i:02}" for i in range(1, 5)} | {"MSCQ01", "MSCQ02"} | {
        f"MSJZ{i:02}" for i in range(1, 5)} | {f"MSLC{i:02}" for i in range(1, 7)} | {
        f"MSWM{i:02}" for i in range(1, 5)} | {"ZHMSJZ02"}
    require(set(societies) == expected_societies, "Society inventory omitted an archived original or form")
    for card_id in ["MSJZ03", "ZHMSJZ02"]:
        require(not societies[card_id]["canStartByPrintedCard"] and societies[card_id]["startingHand"] is None,
                f"Non-initial society/form cannot be an initial selection: {card_id}")
    pairing = coverage["mixingRules"]["baseQuickPairing"]
    require(set(pairing["factionIds"]) == set(OFFICIAL) and len(pairing["factionIds"]) == 8,
            "Base pairing must include all eight colors and exclude neutral")
    require((pairing["chooseDistinct"], pairing["cardsPerFaction"], pairing["totalCards"]) == (2, 25, 50),
            "Wrong source-verified 25+25 base pairing rule")
    require(not pairing["selectionEnabled"] and pairing["status"] == "unimplemented",
            "Rule explanation must not offer unimplemented constructed pairs")
    construction = coverage["mixingRules"]["customConstruction"]
    require(construction["allowsAnyNumberOfColorsBeforeSocietyRestriction"]
            and not construction["societyCountsTowardMinimum"]
            and construction["minimumPlayerCards"] == 50 and construction["maxSameName"] == 3,
            "General construction differs from two-half quick pairing; society is extra")
    require({d["deckId"] for d in coverage["curatedDecks"]}.isdisjoint(set(OFFICIAL)),
            "Deck strategy IDs cannot double as official faction IDs")
    require(all(d["kind"] == "curatedLimited" and not d["officialPreconstructed"]
                for d in coverage["curatedDecks"]), "Custom preconstructed decks cannot be called official")
    for collection in [coverage["factions"], coverage["societies"], coverage["modes"], coverage["mechanisms"]]:
        for entry in collection:
            require(entry["status"] in STATUSES, "Invalid implementation status")
            require(bool(entry["evidenceIds"]) and set(entry["evidenceIds"]) <= set(evidence),
                    "An official/implementation claim lacks original evidence")
    for m in mechanisms.values():
        require(set(m["dependencies"]) <= set(mechanisms), f"Unknown shared dependency: {m['id']}")
        require(bool(m["scope"]) and bool(m["codeEvidence"]), f"Missing bounded code audit: {m['id']}")
        if m["status"] == "partial":
            require(bool(m["limitations"]), f"Partial mechanism must explain its limit: {m['id']}")
    require(mechanisms["shield"]["status"] == "partial", "Shield fixture does not imply playable shield cards")
    require(mechanisms["healWounds"]["status"] == "partial", "Healing fixtures do not implement wound generation")
    require(mechanisms["renown"]["status"] == mechanisms["seal"]["status"] == "unimplemented",
            "Renown/seal cannot be enabled by keyword presence")
    # A dependency cycle would make batch planning ambiguous.
    def visit(i: str, active: set[str], done: set[str]) -> None:
        if i in active:
            errors.append("Shared mechanism dependency cycle: " + i)
            return
        if i in done:
            return
        for dep in mechanisms[i]["dependencies"]:
            if dep in mechanisms:
                visit(dep, active | {i}, done)
        done.add(i)
    completed: set[str] = set()
    for i in mechanisms:
        visit(i, set(), completed)
    require(set(manual) <= set(records), "A primary review has no research record")
    require(index["policy"]["locatorFieldsAreVerified"] is False
            and index["policy"]["researchIndexCanUnlockCards"] is False,
            "Research policy cannot treat locators as verified or release cards")
    for card_id, c in records.items():
        require(c["selectionEnabled"] is False, f"Research record must never unlock a card: {card_id}")
        require(c["engineStatus"] in STATUSES, f"Invalid card support status: {card_id}")
        require(set(c["confirmedMechanismIds"] + c["candidateMechanismIds"]) <= set(mechanisms),
                f"Card refers to undefined shared mechanism: {card_id}")
        require(c["candidateBasis"] == "heuristicLocatorSearchOnly", "Heuristic hints cannot become rules")
        require(c["sameNameMeansDuplicate"] is False, "Same names cannot be silently deduplicated")
        require(c["strategyUiPlaytest"] == "notRunInThisAudit", "Do not fabricate independent UI acceptance")
        if c["evidenceState"] == "primaryImageReviewed":
            require(card_id in manual and bool(c["sourceImages"]) and bool(c["verifiedFields"]),
                    f"Claimed review lacks a manual source: {card_id}")
            if card_id in manual:
                r = manual[card_id]
                require(r["evidenceId"] in evidence, f"Review has no original evidence: {card_id}")
                require(c["verifiedFields"] == r["verifiedFields"]
                        and c["confirmedMechanismIds"] == r["confirmedMechanismIds"]
                        and c["abilitySummary"] == r["abilitySummary"],
                        f"Verified values drifted from manual review: {card_id}")
        else:
            require(not c["verifiedFields"] and not c["confirmedMechanismIds"] and c["abilitySummary"] is None,
                    f"Unreviewed locator promoted to verified fields: {card_id}")
        if c["role"] == "transformedView":
            require(c["formOf"] in records and c["acceptanceState"] == "notAccepted",
                    f"Transformation is a view of its source card: {card_id}")
        if c["engineStatus"] == "implemented":
            require(c["inCurrentCatalog"], f"Card existence is not implementation: {card_id}")
        if c["inCurrentCatalog"]:
            require(c["engineStatus"] == "partial", "Finite registration is not full original-card acceptance: " + card_id)
    totals = index["totals"]
    require(totals["archiveRecords"] == len(records), "Archive count is not an engine pool count")
    require(totals["primaryImageReviewed"] == sum(c["evidenceState"] == "primaryImageReviewed" for c in records.values()),
            "Primary review count is stale")
    require(totals["sourceImagesMissing"] == sum(not c["sourceImages"] for c in records.values()),
            "Missing original image count is stale")
    require(totals["locatorRoleCounts"] == dict(Counter(c["role"] for c in records.values())), "Role totals stale")
    if not check_files:
        return errors, warnings

    groups = json.loads((root / "docs/factions/acceptance-groups.json").read_text())
    errors.extend(validate_groups(groups, coverage, index))

    def check_source(p: str, expected: str, code: bool = False) -> None:
        path = root / p
        if path.resolve().is_relative_to(root.resolve()) and path.is_file():
            actual = hashlib.sha256(path.read_bytes()).hexdigest()
            if actual != expected:
                (warnings if code and allow_code_drift else errors).append("Source snapshot changed: " + p)
        else:
            errors.append("Missing or invalid source path: " + p)

    for p, sha in coverage["codeSnapshot"].items():
        check_source(p, sha, code=True)
    for e in evidence.values():
        require(e["visuallyReviewed"] is True, "Evidence manifest contains unviewed original")
        check_source(e["path"], e["sha256"])
        if e["kind"] == "pdfPage":
            require(e["pdfPage"] >= 1 and e["printedPage"] >= 1, "PDF references are 1-based")
    for k in ["locatorSource", "reviewsSource", "coverageSource"]:
        check_source(index[k]["path"], index[k]["sha256"])
    archive = json.loads((root / index["locatorSource"]["path"]).read_text())
    require(set(records) == set(archive), "The all-record queue must not silently omit archive entries")
    catalog = json.loads((root / coverage["currentPool"]["catalogPath"]).read_text())
    player_cards = {c["id"]: c for c in catalog["cards"] if c["kind"] != "region"}
    catalog_ids = {c["id"] for c in catalog["cards"]}
    text = (root / "rust-game/src/rules.rs").read_text()
    bindings = set(re.findall(r'm\.insert\(\s*"([^"]+)"', text))
    require(catalog_ids == bindings, "Catalog registration and finite rule declarations do not match")
    require({i for i, c in records.items() if c["inCurrentCatalog"]} == catalog_ids,
            "Research implementation labels must match the actual finite registry")
    require(totals["currentCatalogDefinitions"] == len(catalog_ids), "Runtime count stale")
    for f in factions.values():
        expected = sorted(i for i, c in player_cards.items() if c["color"] == f["colorKey"])
        require(sorted(f["catalogPlayerCardIds"]) == expected and f["catalogPlayerCardCount"] == len(expected),
                "Faction coverage must count player definitions, not world cards or copies: " + f["id"])
        require(f["status"] == ("partial" if expected else "unimplemented"),
                "Few present cards cannot establish a complete faction: " + f["id"])
    color_id = {f["colorKey"]: f["id"] for f in factions.values()}
    decks = {d["id"]: d for d in catalog["decks"]}
    require(set(decks) == {d["deckId"] for d in coverage["curatedDecks"]}, "Curated deck IDs stale")
    for d in coverage["curatedDecks"]:
        raw_deck = decks[d["deckId"]]
        actual = Counter()
        for item in raw_deck["cards"]:
            require(item["cardId"] in player_cards, "World/unknown card cannot be a player preconstructed entry")
            actual[color_id[player_cards[item["cardId"]]["color"]]] += item["count"]
        require(d["factionCardCounts"] == dict(actual) and d["cardCount"] == sum(actual.values()),
                "Deck color composition must come from actual cards: " + d["deckId"])
    all_code = "\n".join((root / p).read_text() for p in coverage["codeSnapshot"] if p.endswith(".rs"))
    for m in mechanisms.values():
        for e in m["codeEvidence"]:
            if e["anchor"] not in (root / e["path"]).read_text():
                (warnings if allow_code_drift else errors).append("Code anchor changed: " + m["id"])
        for name in m["existingTestNames"]:
            require("fn " + name + "(" in all_code, "Referenced existing test missing: " + name)
    if not allow_code_drift:
        require((root / "rust-game/data/card-research-index.json").read_text() == serialize(build(root)),
                "Research index is not reproducible from reviewed inputs")
    return errors, warnings


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--allow-code-drift", action="store_true")
    args = parser.parse_args()
    inputs = [json.loads((ROOT / p).read_text()) for p in [
        "rust-game/data/faction-coverage.json", "rust-game/data/card-research-index.json",
        "docs/factions/card-reviews.json",
    ]]
    errors, warnings = validate(*inputs, allow_code_drift=args.allow_code_drift)
    for warning in warnings:
        print("WARNING:", warning)
    for error in errors:
        print("ERROR:", error)
    if errors:
        raise SystemExit(1)
    print(f"Validated {len(inputs[1]['records'])} research records, eight colors + neutral, "
          f"{len(inputs[0]['societies'])} society records and current finite catalog")


if __name__ == "__main__":
    main()
