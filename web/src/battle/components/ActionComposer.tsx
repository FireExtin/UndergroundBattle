import { useEffect, useMemo, useState } from "react";

import type { Action } from "../../debugger/protocol";
import type { BattlePlayerId, BattleState } from "../model";

// Purpose: Builds explicit action payloads from the current battle view model without client-side rule adjudication.

const actionKindOptions = [
  { value: "pass_priority", label: "让过优先权" },
  { value: "advance_phase", label: "推进阶段" },
  { value: "play_card", label: "打出卡牌" },
  { value: "reveal_card", label: "揭示卡牌" },
  { value: "inspect_card", label: "检视卡牌" },
  { value: "declare_attack", label: "发起攻击" },
  { value: "declare_investigation", label: "发起调查" },
  { value: "move_card", label: "移动卡牌" },
  { value: "queue_operation", label: "入栈结算" },
  { value: "set_marker", label: "设置标志" },
  { value: "remove_marker", label: "移除标志" },
  { value: "set_face_down", label: "设为暗置" },
  { value: "use_first_player_privilege", label: "使用先手特权" },
  { value: "set_card_marker", label: "设置卡牌标记" },
  { value: "remove_card_marker", label: "移除卡牌标记" }
] as const;

export type ComposerActionInput = Omit<Action, "id" | "actorId">;

export type ComposerAutoFillHint = {
  token: number;
  sourceCardId?: string;
  targetCardId?: string;
};

type ActionComposerProps = {
  actorId: BattlePlayerId;
  battle: BattleState;
  pending: boolean;
  disabledReason: string;
  autoFillHint?: ComposerAutoFillHint;
  onSubmitAction: (action: ComposerActionInput) => void;
};

export function ActionComposer({
  actorId,
  battle,
  pending,
  disabledReason,
  autoFillHint,
  onSubmitAction
}: ActionComposerProps) {
  const [kind, setKind] = useState<string>("pass_priority");
  const [sourceCardId, setSourceCardId] = useState<string>("");
  const [targetCardId, setTargetCardId] = useState<string>("");
  const [targetRegionCardId, setTargetRegionCardId] = useState<string>("");
  const [playMode, setPlayMode] = useState<string>("face_up");
  const [targetPlayerId, setTargetPlayerId] = useState<string>(battle.opponentPlayerId);
  const [markerType, setMarkerType] = useState<string>("secret_society");
  const [markerAmount, setMarkerAmount] = useState<string>("1");
  const [operationLabel, setOperationLabel] = useState<string>("");
  const [error, setError] = useState<string>("");
  const [sourceManualLocked, setSourceManualLocked] = useState<boolean>(false);
  const [targetManualLocked, setTargetManualLocked] = useState<boolean>(false);
  const [targetRegionManualLocked, setTargetRegionManualLocked] = useState<boolean>(false);

  const sourceOptions = useMemo(() => {
    const ids = [
      ...battle.actionCandidates.localHandCardIds,
      ...battle.actionCandidates.localTableCardIds
    ];
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

  const actionsDisabled = pending || disabledReason !== "";

  useEffect(() => {
    setTargetPlayerId(battle.opponentPlayerId);
    setSourceManualLocked(false);
    setTargetManualLocked(false);
    setTargetRegionManualLocked(false);
  }, [actorId, battle.opponentPlayerId]);

  useEffect(() => {
    if (sourceCardId.trim() !== "" && !sourceOptions.includes(sourceCardId)) {
      setSourceCardId("");
      setSourceManualLocked(false);
    }
  }, [sourceCardId, sourceOptions]);

  useEffect(() => {
    if (targetCardId.trim() !== "" && !targetOptions.includes(targetCardId)) {
      setTargetCardId("");
      setTargetManualLocked(false);
    }
  }, [targetCardId, targetOptions]);

  useEffect(() => {
    if (targetRegionCardId.trim() !== "" && !targetRegionOptions.includes(targetRegionCardId)) {
      setTargetRegionCardId("");
      setTargetRegionManualLocked(false);
    }
  }, [targetRegionCardId, targetRegionOptions]);

  useEffect(() => {
    if (!autoFillHint) {
      return;
    }

    const suggestedSource = autoFillHint.sourceCardId?.trim() ?? "";
    if (suggestedSource !== "" && sourceOptions.includes(suggestedSource)) {
      if (!sourceManualLocked || sourceCardId.trim() === "") {
        setSourceCardId(suggestedSource);
      }
    }

    const suggestedTarget = autoFillHint.targetCardId?.trim() ?? "";
    if (suggestedTarget !== "" && targetOptions.includes(suggestedTarget)) {
      if (!targetManualLocked || targetCardId.trim() === "") {
        setTargetCardId(suggestedTarget);
      }
    }
    if (suggestedTarget !== "" && targetRegionOptions.includes(suggestedTarget)) {
      if (!targetRegionManualLocked || targetRegionCardId.trim() === "") {
        setTargetRegionCardId(suggestedTarget);
      }
    }
  }, [
    autoFillHint,
    sourceCardId,
    sourceManualLocked,
    sourceOptions,
    targetCardId,
    targetManualLocked,
    targetOptions,
    targetRegionCardId,
    targetRegionManualLocked,
    targetRegionOptions
  ]);

  return (
    <section className="panel battle-actions" aria-label="动作面板">
      <h2>动作面板</h2>
      <p className="muted">当前操作方：{actorId}</p>
      {disabledReason ? <p className="muted">{disabledReason}</p> : null}
      <details className="battle-actions__guide">
        <summary>动作说明</summary>
        <ul className="simple-list">
          <li>
            <strong>让过优先权</strong>：在当前窗口内放弃行动，双方连续让过会结束步骤或触发栈结算。
          </li>
          <li>
            <strong>推进阶段</strong>：推进 `main/end` 阶段并触发规则书定义的阶段结算。
          </li>
          <li>
            <strong>打出卡牌</strong>：从手牌部署角色/附属/事务。角色可选明置或暗置并指定地区。
          </li>
          <li>
            <strong>攻击/调查/移动</strong>：`来源卡牌` 选本方驻场，`目标卡牌` 选敌方角色或地区。
          </li>
          <li>
            <strong>标记动作</strong>：用于秘社或卡牌标记增减，需填写类型与数量。
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

        <label className="battle-actions__field">
          <span>来源卡牌</span>
          <select
            aria-label="来源卡牌"
            value={sourceCardId}
            onChange={(event) => {
              setSourceCardId(event.target.value);
              setSourceManualLocked(true);
              setError("");
            }}
            disabled={actionsDisabled}
          >
            <option value="">（无）</option>
            {sourceOptions.map((value) => (
              <option key={value} value={value}>
                {value}
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
              setTargetManualLocked(true);
              setError("");
            }}
            disabled={actionsDisabled}
          >
            <option value="">（无）</option>
            {targetOptions.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </label>

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

        <label className="battle-actions__field battle-actions__field--wide">
          <span>操作标签</span>
          <input
            aria-label="操作标签"
            value={operationLabel}
            onChange={(event) => {
              setOperationLabel(event.target.value);
              setError("");
            }}
            disabled={actionsDisabled}
            placeholder="可选"
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

        <label className="battle-actions__field">
          <span>部署地区</span>
          <select
            aria-label="部署地区"
            value={targetRegionCardId}
            onChange={(event) => {
              setTargetRegionCardId(event.target.value);
              setTargetRegionManualLocked(true);
              setError("");
            }}
            disabled={actionsDisabled}
          >
            <option value="">（无）</option>
            {targetRegionOptions.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </label>
      </div>

      <div className="battle-actions__buttons">
        <button
          type="button"
          className="action-button"
          disabled={actionsDisabled}
          onClick={() => {
            submitWithKind("pass_priority");
          }}
        >
          让过优先权
        </button>
        <button
          type="button"
          className="action-button action-button--secondary"
          disabled={actionsDisabled}
          onClick={() => {
            submitWithKind("advance_phase");
          }}
        >
          推进阶段
        </button>
        <button
          type="button"
          className="action-button"
          disabled={actionsDisabled}
          onClick={() => {
            submitWithKind(kind);
          }}
          aria-label="提交动作"
        >
          提交动作
        </button>
      </div>
      {error ? <p className="custom-action-error">{error}</p> : null}
    </section>
  );

  function submitWithKind(actionKind: string) {
    if (actionsDisabled) {
      return;
    }

    const validation = validateBeforeSubmit(
      actionKind,
      sourceCardId,
      targetCardId,
      markerType,
      markerAmount
    );
    if (validation !== "") {
      setError(validation);
      return;
    }

    const action: ComposerActionInput = {
      kind: actionKind
    };

    if (requiresCardID(actionKind)) {
      action.cardId = sourceCardId;
    }

    if (requiresTargetCardID(actionKind)) {
      action.targetCardId = targetCardId;
    }

    if (usesTargetPlayer(actionKind)) {
      action.targetPlayerId = targetPlayerId || actorId;
    }

    if (usesMarker(actionKind)) {
      action.markerType = markerType;
      action.markerAmount = Number(markerAmount);
    }
    if (actionKind === "play_card") {
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

    if (actionKind === "queue_operation" && operationLabel.trim() !== "") {
      action.operationLabel = operationLabel.trim();
    }

    setError("");
    onSubmitAction(action);
  }
}

function requiresCardID(actionKind: string) {
  return (
    actionKind === "reveal_card" ||
    actionKind === "inspect_card" ||
    actionKind === "declare_attack" ||
    actionKind === "declare_investigation" ||
    actionKind === "move_card" ||
    actionKind === "queue_operation" ||
    actionKind === "set_face_down" ||
    actionKind === "play_card"
  );
}

function requiresTargetCardID(actionKind: string) {
  return (
    actionKind === "declare_attack" ||
    actionKind === "declare_investigation" ||
    actionKind === "move_card" ||
    actionKind === "set_card_marker" ||
    actionKind === "remove_card_marker"
  );
}

function usesTargetPlayer(actionKind: string) {
  return actionKind === "set_marker" || actionKind === "remove_marker";
}

function usesMarker(actionKind: string) {
  return (
    actionKind === "set_marker" ||
    actionKind === "remove_marker" ||
    actionKind === "set_card_marker" ||
    actionKind === "remove_card_marker"
  );
}

function validateBeforeSubmit(
  actionKind: string,
  sourceCardId: string,
  targetCardId: string,
  markerType: string,
  markerAmount: string
) {
  if (requiresCardID(actionKind) && sourceCardId.trim() === "") {
    return "需要选择卡牌";
  }

  if (requiresTargetCardID(actionKind) && targetCardId.trim() === "") {
    return "需要选择目标卡牌";
  }

  if (usesMarker(actionKind) && markerType.trim() === "") {
    return "需要输入标记类型";
  }

  if (usesMarker(actionKind)) {
    const parsed = Number(markerAmount);
    if (!Number.isFinite(parsed) || parsed <= 0) {
      return "标记数量必须大于 0";
    }
  }

  return "";
}

function uniq(values: string[]) {
  const unique = Array.from(new Set(values.filter((value) => value.trim() !== "")));
  unique.sort((left, right) => left.localeCompare(right));
  return unique;
}
