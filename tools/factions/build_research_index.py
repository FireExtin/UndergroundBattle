#!/usr/bin/env python3
"""Build a non-executable research index; raw archive fields remain locators.

Only card-reviews.json can supply verified values. This script cannot release cards.
Run from any directory. --check detects stale checked-in output without writing it.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ARCHIVE = Path("resource/ymsj-fun.github.io/cards/cards.json")
REVIEWS = Path("docs/factions/card-reviews.json")
COVERAGE = Path("rust-game/data/faction-coverage.json")
SPECIFICATIONS = Path("docs/factions/card-specifications.json")
RECOVERY = Path("docs/factions/source-recovery.json")
OUTPUT = Path("rust-game/data/card-research-index.json")

# Search hints only. Full text, alternate spelling and semantics still need review.
# These names are shared mechanism IDs in faction-coverage.json, never engine ops.
HINTS = {
    "hidden": ["暗藏者", "潜伏", "现身"],
    "public": ["公开"], "exhaust": ["横置", "重置"],
    "mobility": ["机动", "移动"],
    "combatTraits": ["杀伤", "护卫", "撤回"],
    "barrier": ["屏障"], "shield": ["护盾"],
    "graveRecovery": ["墓地"], "gravePlay": ["墓地行动"],
    "sacrifice": ["牺牲", "栖牲"],
    "costReduction": ["费用减少", "费用减", "所需费用减"],
    "deckChoice": ["抓", "预测", "洗牌", "牌库", "弃"],
    "continuous": ["持续", "获得", "失去", "视作空白"],
    "healWounds": ["创伤"], "attachment": ["结附", "附属", "回收"],
    "seal": ["封印"], "renown": ["声望"], "spirit": ["灵体"],
    "lock": ["锁定"], "privatePeek": ["检视"], "timeDestroy": ["时间标志", "毁灭"],
    "damagePrevention": ["防止", "伤害改", "不受伤害"],
    "control": ["操控"], "transform": ["转变"], "blink": ["闪烁"],
    "nonAsset": ["非资产"], "slow": ["迟缓"], "leader": ["领袖", "唯一"],
    "token": ["指示物", "生成"],
    "mysticContest": ["隐秘", "暴露", "神秘学博弈"],
    "plan": ["秘密计划", "可推进", "推进标志"],
    "rumor": ["传闻"], "blood": ["饮血", "名为鲜血", "名称为鲜血"],
    "task": ["任务"], "core": ["核心", "名誉点", "休整", "失控", "命令"],
    "encounter": ["利用", "恐怖", "遭遇角色"],
    "combatKeywords": ["袭击", "创伤", "遏制", "威名"],
    "hiddenOnly": ["隐匿"], "cancel": ["终止"],
    "handAbility": ["手牌行动", "墓地行动", "手牌能力"],
    "crisis": ["危机", "剧情", "威胁", "支援牌库"],
    "chaos": ["混乱"], "herald": ["先兆"],
    "reincarnation": ["转生"], "eternal": ["永恒"],
}


def read(root: Path, path: Path) -> dict:
    return json.loads((root / path).read_text(encoding="utf-8"))


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tentative_role(card: dict) -> str:
    # Transformed views have priority over token/deckcard flags: ZHMSJZ02 is not
    # a new token or a separately constructible card despite archive istoken=true.
    if card["id"].startswith("ZH"):
        return "transformedView"
    if card["basic-type"] == "秘社":
        return "society"
    if card["basic-type"] == "核心区":
        return "coreAreaReference"
    if card["basic-type"] == "指示物":
        return "markerReference"
    if card["istoken"]:
        return "generatedToken"
    if card["basic-type"] == "遭遇角色":
        return "worldEncounter"
    if card["basic-type"] == "地区":
        return "worldRegion"
    if card["basic-type"] == "任务/地区":
        return "playerTaskRegion"
    if card["basic-type"] == "角色/附属":
        return "playerDualType"
    return "playerCardCandidate"


def build(root: Path = ROOT) -> dict:
    archive = read(root, ARCHIVE)
    reviews = read(root, REVIEWS)["cards"]
    coverage = read(root, COVERAGE)
    all_specifications = read(root, SPECIFICATIONS)
    recovery = read(root, RECOVERY)
    recovered = defaultdict(list)
    for original in recovery.get("recoveredOriginals", []):
        if not original["archiveBindingConfirmed"]:
            continue
        path = Path(original["path"])
        if (not path.is_relative_to("docs/factions/recovered-originals")
                or not path.name.startswith(original["cardId"] + " ")
                or digest(root / path) != original["sha256"]):
            raise ValueError("Recovered original identity/hash differs: " + original["cardId"])
        recovered[original["cardId"]].append(str(path))
    specifications = all_specifications["cards"]
    non_card_references = all_specifications.get("nonCardReferences", {})
    catalog_path = Path(coverage["currentPool"]["catalogPath"])
    if digest(root / catalog_path) != coverage["codeSnapshot"][str(catalog_path)]:
        raise ValueError("Catalog changed: re-audit coverage before rebuilding support labels")
    catalog = read(root, catalog_path)
    active = {c["id"] for c in catalog["cards"]}
    mechanisms = {m["id"]: m for m in coverage["mechanisms"]}
    societies = {s["cardId"]: s for s in coverage["societies"]}
    cards_dir = root / "resource/ymsj-fun.github.io/cards"
    by_name = defaultdict(list)
    for card in archive.values():
        by_name[card["name"]].append(card["id"])
    records = []
    for card_id, raw in sorted(archive.items()):
        image_paths = sorted({str(p.relative_to(root)) for p in cards_dir.glob(card_id + " *")
                              if p.suffix.lower() in {".jpg", ".jpeg", ".png"}} | set(recovered[card_id]))
        review = reviews.get(card_id)
        spec = specifications.get(card_id)
        reference = non_card_references.get(card_id)
        confirmed = review["confirmedMechanismIds"] if review else []
        searchable = "\n".join([raw["text"], raw["type"], *raw["keywords"]])
        candidates = sorted(i for i, terms in HINTS.items() if any(t in searchable for t in terms))
        if raw["basic-type"] == "秘社":
            candidates.append("society")
        edition = review.get("editionStatus", "unreviewed") if review else "unreviewed"
        if raw["set"] == "测试" and edition == "unreviewed":
            edition = "uncertain"
        # A registered finite definition can run in the current curated pool,
        # but partial image review is not full printed-rule equivalence.
        support = "partial" if card_id in active else "unimplemented"
        if not image_paths or edition in {"uncertain", "testImageConfirmed"}:
            support = "uncertain" if card_id not in active else support
        role = tentative_role(raw)
        interactions = sorted({u for i in confirmed for u in mechanisms[i]["interactionIds"]})
        records.append({
            "id": card_id,
            "role": role,
            "roleBasis": "primaryImage" if review else "locatorOnly",
            "sourceImages": [{"path": p, "sha256": digest(root / p)} for p in image_paths],
            "evidenceState": "primaryImageReviewed" if review else "missingImage" if not image_paths else "unreviewed",
            "reviewRef": "docs/factions/card-reviews.json#cards/" + card_id if review else None,
            "fullCardVerification": spec["sourceVerification"]["status"] if spec else "notStarted",
            "fullSpecRef": str(SPECIFICATIONS) + "#cards/" + card_id if spec else None,
            "sourceRecordVerification": spec["sourceVerification"]["status"] if spec else reference["sourceVerification"]["status"] if reference else "notStarted",
            "sourceSpecificationKind": "printedCard" if spec else "nonCardReference" if reference else None,
            "sourceSpecRef": str(SPECIFICATIONS) + "#cards/" + card_id if spec else str(SPECIFICATIONS) + "#nonCardReferences/" + card_id if reference else None,
            "implementationDesignStatus": spec["implementationDesignStatus"] if spec else "notStarted",
            "editionStatus": edition,
            "locator": {k: raw[k] for k in [
                "name", "set", "set-id", "type", "basic-type", "istoken", "deckcard",
                "color", "cost", "lyl", "magic", "abl", "dfc", "hands", "sc", "req",
                "core", "society", "text", "keywords", "related",
            ] if k in raw},
            "verifiedFields": review["verifiedFields"] if review else {},
            "abilitySummary": review["abilitySummary"] if review else None,
            "confirmedMechanismIds": confirmed,
            "candidateMechanismIds": sorted(set(candidates)),
            "candidateBasis": "heuristicLocatorSearchOnly",
            "interactionIds": interactions,
            "mechanismCoverage": {i: mechanisms[i]["status"] for i in confirmed},
            "engineStatus": support,
            "engineStatusScope": "Full original-card equivalence is not accepted; current catalog IDs have finite handwritten implementations only.",
            "inCurrentCatalog": card_id in active,
            "selectionEnabled": False,
            "acceptanceState": "existingLimitedPool" if card_id in active else "notAccepted",
            "strategyUiPlaytest": "notRunInThisAudit",
            "sameNameRecordIds": [i for i in by_name[raw["name"]] if i != card_id],
            "sameNameMeansDuplicate": False,
            "formOf": card_id[2:] if card_id.startswith("ZH") else None,
            "initialSocietyAllowed": societies[card_id]["canStartByPrintedCard"] if card_id in societies else None,
        })
    totals = {
        "archiveRecords": len(records),
        "sourceImagesMissing": sum(not c["sourceImages"] for c in records),
        "primaryImageReviewed": sum(c["evidenceState"] == "primaryImageReviewed" for c in records),
        "unreviewedWithImage": sum(c["evidenceState"] == "unreviewed" for c in records),
        "currentCatalogDefinitions": len(active),
        "completeSourceSpecifications": sum(c["fullCardVerification"] == "completeGameplayFieldsForPinnedImage" for c in records),
        "blockedSourceSpecifications": sum(c["fullCardVerification"] == "blocked" for c in records),
        "completeNonCardReferences": sum(c["sourceRecordVerification"] == "completeNonCardReference" for c in records),
        "completeSourceRecords": sum(c["sourceRecordVerification"] in {"completeGameplayFieldsForPinnedImage", "completeNonCardReference"} for c in records),
        "locatorSetCounts": dict(sorted(Counter(c["locator"]["set"] for c in records).items())),
        "locatorRoleCounts": dict(sorted(Counter(c["role"] for c in records).items())),
        "editionStatusCounts": dict(sorted(Counter(c["editionStatus"] for c in records).items())),
    }
    return {
        "schemaVersion": 1,
        "purpose": "Non-executable, all-record research queue; not a playable pool or a release declaration.",
        "auditedCommit": coverage["auditedCommit"],
        "locatorSource": {"path": str(ARCHIVE), "sha256": digest(root / ARCHIVE), "authoritativeRules": False},
        "reviewsSource": {"path": str(REVIEWS), "sha256": digest(root / REVIEWS)},
        "coverageSource": {"path": str(COVERAGE), "sha256": digest(root / COVERAGE)},
        "specificationsSource": {"path": str(SPECIFICATIONS), "sha256": digest(root / SPECIFICATIONS)},
        "recoverySource": {"path": str(RECOVERY), "sha256": digest(root / RECOVERY)},
        "policy": {
            "locatorFieldsAreVerified": False,
            "unreviewedCardsCanBeReleased": False,
            "imageReviewImpliesCompleteRules": False,
            "sameNameImpliesSameVersion": False,
            "transformedViewsAreExtraDeckCards": False,
            "researchIndexCanUnlockCards": False,
        },
        "totals": totals,
        "records": records,
    }


def serialize(data: dict) -> str:
    return json.dumps(data, ensure_ascii=False, indent=2) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = serialize(build())
    if args.check:
        if (ROOT / OUTPUT).read_text(encoding="utf-8") != generated:
            raise SystemExit("Research index is stale; review changes, then rebuild")
        print("Research index is reproducible")
    else:
        (ROOT / OUTPUT).write_text(generated, encoding="utf-8")
        print("Wrote research index (no playable registry changes)")


if __name__ == "__main__":
    main()
