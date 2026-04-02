import { useEffect, useMemo, useState } from "react";

import type { Action, CardView } from "../../debugger/protocol";
import { validateActionInput } from "../actionPolicy";
import type { BattlePlayerId, BattleState } from "../model";
import type { BattleCardPick } from "./BattleTable";

// Purpose: Builds battle actions from click-first card selection while keeping server-side adjudication authoritative.

const actionKindOptions = [
  { value: "play_card", label: "打出卡牌" },
  { value: "build_asset", label: "建立资产" },
  { value: "declare_attack", label: "发起攻击" },
  { value: "declare_investigation", label: "发起调查" },
  { value: "move_card", label: "移动卡牌" },
  { value: "set_marker", label: "设置玩家标志" },
  { value: "remove_marker", label: "移除玩家标志" },
  { value: "set_card_marker", label: "设置卡牌标记" },
  { value: "remove_card_marker", label: "移除卡牌标记" }
] as const;

export type ComposerActionInput = Omit<Action, "id" | "actorId">;

export type ComposerCardPickEvent = {
  token: number;
  picked: BattleCardPick;
};

type ActionComposerProps = {
  actorId: BattlePlayerId;
  battle: BattleState;
  pending: boolean;
  endPending: boolean;
  disabledReason: string;
  pickedCardEvent?: ComposerCardPickEvent;
  onSubmitAction: (action: ComposerActionInput) => void;
  onEndAction: () => void;
};

export function ActionComposer({
  actorId,
  battle,
  pending,
  endPending,
  disabledReason,
  pickedCardEvent,
  onSubmitAction,
  onEndAction
}: ActionComposerProps) {
  const [kind, setKind] = useState<string>("play_card");
  const [sourceCardId, setSourceCardId] = useState<string>("");
  const [targetCardId, setTargetCardId] = useState<string>("");
  const [targetRegionCardId, setTargetRegionCardId] = useState<string>("");
  const [playMode, setPlayMode] = useState<string>("face_up");
  const [targetPlayerId, setTargetPlayerId] = useState<string>(battle.opponentPlayerId);
  const [markerType, setMarkerType] = useState<string>("secret_society");
  const [markerAmount, setMarkerAmount] = useState<string>("1");
  const [error, setError] = useState<string>("");

  const cardLookup = useMemo(() => buildCardLookup(battle), [battle]);
  const sourceCardKind = cardLookup.get(sourceCardId)?.kind ?? "";

  const sourceOptions = useMemo(() => {
    const ids = [...battle.actionCandidates.localHandCardIds, ...battle.actionCandidates.localTableCardIds];
    return uniq(ids);
  }, [battle.actionCandidates.localHandCardIds, battle.actionCandidates.localTableCardIds]);

  const targetOptions = useMemo(() => {
    const ids = [
      ...battle.actionCandidates.regionCardIds,
      ...battle.actionCandidates.opponentTableCardIds,
      ...battle.actionCandidates.visibleCardIds
    ];
    return uniq(ids);
  }, [
    battle.actionCandidates.opponentTableCardIds,
    battle.actionCandidates.regionCardIds,
    battle.actionCandidates.visibleCardIds
  ]);

  const targetRegionOptions = useMemo(() => {
    return uniq(battle.actionCandidates.regionCardIds);
  }, [battle.actionCandidates.regionCardIds]);

  const actionsDisabled = pending || endPending || disabledReason !== "";

  useEffect(() => {
    setTargetPlayerId(battle.opponentPlayerId);
  }, [actorId, battle.opponentPlayerId]);

  useEffect(() => {
    if (sourceCardId.trim() !== "" && !sourceOptions.includes(sourceCardId)) {
      setSourceCardId("");
    }
  }, [sourceCardId, sourceOptions]);

  useEffect(() => {
    if (targetCardId.trim() !== "" && !targetOptions.includes(targetCardId)) {
      setTargetCardId("");
    }
  }, [targetCardId, targetOptions]);

  useEffect(() => {
    if (targetRegionCardId.trim() !== "" && !targetRegionOptions.includes(targetRegionCardId)) {
      setTargetRegionCardId("");
    }
  }, [targetRegionCardId, targetRegionOptions]);

  useEffect(() => {
    if (!pickedCardEvent) {
      return;
    }

    const pick = pickedCardEvent.picked;
    if (!pick.cardId) {
      return;
    }
    const isOwnCard = pick.ownerId === actorId;

    if (kind === "set_marker" || kind === "remove_marker") {
      return;
    }

    if (kind === "build_asset") {
      if (isOwnCard && pick.zone === "hand") {
        setSourceCardId(pick.cardId);
        setTargetCardId("");
        setTargetRegionCardId("");
        setError("");
      }
      return;
    }

    const isRegion = pick.kind === "region";

    if (kind === "set_card_marker" || kind === "remove_card_marker") {
      if (!isRegion) {
        setTargetCardId(pick.cardId);
        setError("");
      }
      return;
    }

    if (isRegion) {
      if (isPlayCardActionKind(kind)) {
        setTargetRegionCardId(pick.cardId);
      } else {
        setTargetCardId(pick.cardId);
      }
      setError("");
      return;
    }

    if (isOwnCard) {
      if (pick.zone === "hand") {
        setSourceCardId(pick.cardId);
        setError("");
        return;
      }

      if (isPlayCardActionKind(kind) && sourceCardKind === "asset") {
        setTargetCardId(pick.cardId);
        setError("");
        return;
      }

      setSourceCardId(pick.cardId);
      setError("");
      return;
    }

    setTargetCardId(pick.cardId);
    setError("");
  }, [actorId, kind, pickedCardEvent, sourceCardKind]);

  return (
    <section className="panel battle-actions" aria-label="动作面板">
      <h2>动作面板</h2>
      <p className="muted">当前操作方：{actorId}</p>
      {disabledReason ? <p className="muted">{disabledReason}</p> : null}

      <details className="battle-actions__guide">
        <summary>动作说明</summary>
        <ul className="simple-list">
          <li>
            <strong>点选优先</strong>：先在牌桌上点牌，再在此提交动作。手工下拉仅作补充。
          </li>
          <li>
            <strong>结束行动</strong>：自动提交让过优先权；若步骤结束会自动推进阶段。
          </li>
          <li>
            <strong>打出卡牌</strong>：角色通常需要选择部署地区，附属需要选择宿主。
          </li>
          <li>
            <strong>攻击/调查/移动</strong>：来源选本方驻场，目标选敌方驻场或地区。
          </li>
        </ul>
        <a href="/battle-action-guide.md" target="_blank" rel="noreferrer">
          查看完整说明
        </a>
      </details>

      <div className="battle-actions__grid">
        <label className="battle-actions__field">
          <span>动作类型</span>
          <select
            aria-label="动作类型"
            value={kind}
            onChange={(event) => {
              setKind(event.target.value);
              setError("");
            }}
            disabled={actionsDisabled}
          >
            {actionKindOptions.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </label>

        <div className="battle-actions__field battle-actions__field--wide">
          <span>当前选择</span>
          <div className="battle-actions__selection">
            <p>来源：{cardNameOrPlaceholder(cardLookup, sourceCardId, "点击本方手牌或本方驻场")}</p>
            <p>目标：{cardNameOrPlaceholder(cardLookup, targetCardId, "点击敌方驻场或地区")}</p>
            <p>地区：{cardNameOrPlaceholder(cardLookup, targetRegionCardId, "点击地区牌")}</p>
          </div>
        </div>

        <label className="battle-actions__field">
          <span>目标玩家</span>
          <select
            aria-label="目标玩家"
            value={targetPlayerId}
            onChange={(event) => {
              setTargetPlayerId(event.target.value);
              setError("");
            }}
            disabled={actionsDisabled}
          >
            <option value={actorId}>{actorId}</option>
            <option value={battle.opponentPlayerId}>{battle.opponentPlayerId}</option>
          </select>
        </label>

        <label className="battle-actions__field">
          <span>标记类型</span>
          <input
            aria-label="标记类型"
            value={markerType}
            onChange={(event) => {
              setMarkerType(event.target.value);
              setError("");
            }}
            disabled={actionsDisabled}
          />
        </label>

        <label className="battle-actions__field">
          <span>标记数量</span>
          <input
            aria-label="标记数量"
            type="number"
            min={1}
            step={1}
            value={markerAmount}
            onChange={(event) => {
              setMarkerAmount(event.target.value);
              setError("");
            }}
            disabled={actionsDisabled}
          />
        </label>

        <label className="battle-actions__field">
          <span>部署方式</span>
          <select
            aria-label="部署方式"
            value={playMode}
            onChange={(event) => {
              setPlayMode(event.target.value);
              setError("");
            }}
            disabled={actionsDisabled}
          >
            <option value="face_up">明置</option>
            <option value="face_down">暗置</option>
          </select>
        </label>
      </div>

      <details className="battle-actions__guide">
        <summary>手动调整（可选）</summary>
        <div className="battle-actions__grid">
          <label className="battle-actions__field">
            <span>来源卡牌</span>
            <select
              aria-label="来源卡牌"
              value={sourceCardId}
              onChange={(event) => {
                setSourceCardId(event.target.value);
                setError("");
              }}
              disabled={actionsDisabled}
            >
              <option value="">（无）</option>
              {sourceOptions.map((value) => (
                <option key={value} value={value}>
                  {cardLabel(cardLookup, value)}
                </option>
              ))}
            </select>
          </label>

          <label className="battle-actions__field">
            <span>目标卡牌</span>
            <select
              aria-label="目标卡牌"
              value={targetCardId}
              onChange={(event) => {
                setTargetCardId(event.target.value);
                setError("");
              }}
              disabled={actionsDisabled}
            >
              <option value="">（无）</option>
              {targetOptions.map((value) => (
                <option key={value} value={value}>
                  {cardLabel(cardLookup, value)}
                </option>
              ))}
            </select>
          </label>

          <label className="battle-actions__field">
            <span>部署地区</span>
            <select
              aria-label="部署地区"
              value={targetRegionCardId}
              onChange={(event) => {
                setTargetRegionCardId(event.target.value);
                setError("");
              }}
              disabled={actionsDisabled}
            >
              <option value="">（无）</option>
              {targetRegionOptions.map((value) => (
                <option key={value} value={value}>
                  {cardLabel(cardLookup, value)}
                </option>
              ))}
            </select>
          </label>
        </div>
      </details>

      <div className="battle-actions__buttons">
        <button
          type="button"
          className="action-button action-button--secondary"
          disabled={actionsDisabled}
          onClick={() => {
            onEndAction();
          }}
        >
          结束行动
        </button>
        <button
          type="button"
          className="action-button"
          disabled={actionsDisabled}
          onClick={() => {
            submitCurrentAction();
          }}
          aria-label="提交动作"
        >
          提交动作
        </button>
      </div>
      {error ? <p className="custom-action-error">{error}</p> : null}
    </section>
  );

  function submitCurrentAction() {
    if (actionsDisabled) {
      return;
    }

    const validation = validateBeforeSubmit({
      rulesMetadata: battle.rulesMetadata,
      actionKind: kind,
      sourceCardId,
      targetCardId,
      targetRegionCardId,
      targetPlayerId,
      markerType,
      markerAmount,
      playMode,
      sourceCardKind
    });
    if (validation !== "") {
      setError(validation);
      return;
    }

    const action: ComposerActionInput = {
      kind
    };

    if (isPlayCardActionKind(kind) || kind === "build_asset" || kind === "declare_attack" || kind === "declare_investigation" || kind === "move_card") {
      action.cardId = sourceCardId;
    }

    if (
      kind === "declare_attack" ||
      kind === "declare_investigation" ||
      kind === "move_card" ||
      kind === "set_card_marker" ||
      kind === "remove_card_marker"
    ) {
      action.targetCardId = targetCardId;
    }

    if (kind === "set_marker" || kind === "remove_marker") {
      action.targetPlayerId = targetPlayerId || actorId;
      action.markerType = markerType;
      action.markerAmount = Number(markerAmount);
    }

    if (kind === "set_card_marker" || kind === "remove_card_marker") {
      action.targetCardId = targetCardId;
      action.markerType = markerType;
      action.markerAmount = Number(markerAmount);
    }

    if (isPlayCardActionKind(kind)) {
      action.playMode = playMode;
      if (targetRegionCardId.trim() !== "") {
        action.targetRegionCardId = targetRegionCardId.trim();
      }
      if (targetCardId.trim() !== "") {
        action.targetCardId = targetCardId.trim();
      }
      if (targetPlayerId.trim() !== "") {
        action.targetPlayerId = targetPlayerId.trim();
      }
    }

    setError("");
    onSubmitAction(action);
  }
}

function validateBeforeSubmit(input: {
  rulesMetadata: BattleState["rulesMetadata"];
  actionKind: string;
  sourceCardId: string;
  targetCardId: string;
  targetRegionCardId: string;
  targetPlayerId: string;
  markerType: string;
  markerAmount: string;
  playMode: string;
  sourceCardKind: string;
}) {
  return validateActionInput(input.rulesMetadata, input);
}

function buildCardLookup(battle: BattleState) {
  const lookup = new Map<string, CardView>();
  const push = (card: CardView | undefined) => {
    const cardId = card?.cardId?.trim();
    if (!cardId) {
      return;
    }
    if (!card) {
      return;
    }
    if (!lookup.has(cardId)) {
      lookup.set(cardId, card);
    }
  };

  for (const card of battle.local.hand) {
    push(card);
  }
  for (const card of battle.local.table) {
    push(card);
  }
  for (const card of battle.local.discard) {
    push(card);
  }
  for (const card of battle.local.score) {
    push(card);
  }
  for (const card of battle.opponent.handPreview) {
    push(card);
  }
  for (const card of battle.opponent.table) {
    push(card);
  }
  for (const region of battle.contest.regions) {
    push(region.regionCard);
    for (const card of region.localCards) {
      push(card);
    }
    for (const card of region.opponentCards) {
      push(card);
    }
  }
  for (const card of battle.contest.unassignedTableCards) {
    push(card);
  }

  return lookup;
}

function cardLabel(lookup: Map<string, CardView>, cardId: string) {
  const card = lookup.get(cardId);
  if (!card) {
    return cardId;
  }
  return `${card.name ?? cardId}（${cardId}）`;
}

function cardNameOrPlaceholder(
  lookup: Map<string, CardView>,
  cardId: string,
  placeholder: string
) {
  const trimmed = cardId.trim();
  if (trimmed === "") {
    return placeholder;
  }
  const card = lookup.get(trimmed);
  if (!card) {
    return trimmed;
  }
  if (card.color) {
    return `${card.name ?? trimmed} · ${card.color}`;
  }
  return card.name ?? trimmed;
}

function uniq(values: string[]) {
  const unique = Array.from(new Set(values.filter((value) => value.trim() !== "")));
  unique.sort((left, right) => left.localeCompare(right));
  return unique;
}

function isPlayCardActionKind(kind: string) {
  return kind === "play_card";
}
