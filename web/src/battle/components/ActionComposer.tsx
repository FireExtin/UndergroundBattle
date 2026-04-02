import { useEffect, useMemo, useState } from "react";

import type { Action } from "../../debugger/protocol";
import type { BattlePlayerId, BattleState } from "../model";

// Purpose: Builds explicit action payloads from the current battle view model without client-side rule adjudication.

const actionKindOptions = [
  { value: "pass_priority", label: "Pass Priority" },
  { value: "advance_phase", label: "Advance Phase" },
  { value: "reveal_card", label: "Reveal Card" },
  { value: "inspect_card", label: "Inspect Card" },
  { value: "declare_attack", label: "Declare Attack" },
  { value: "declare_investigation", label: "Declare Investigation" },
  { value: "move_card", label: "Move Card" },
  { value: "queue_operation", label: "Queue Operation" },
  { value: "set_marker", label: "Set Marker" },
  { value: "remove_marker", label: "Remove Marker" },
  { value: "set_face_down", label: "Set Face Down" },
  { value: "use_first_player_privilege", label: "Use First-Player Privilege" },
  { value: "set_card_marker", label: "Set Card Marker" },
  { value: "remove_card_marker", label: "Remove Card Marker" }
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
  const [targetPlayerId, setTargetPlayerId] = useState<string>(battle.opponentPlayerId);
  const [markerType, setMarkerType] = useState<string>("secret_society");
  const [markerAmount, setMarkerAmount] = useState<string>("1");
  const [operationLabel, setOperationLabel] = useState<string>("");
  const [error, setError] = useState<string>("");
  const [sourceManualLocked, setSourceManualLocked] = useState<boolean>(false);
  const [targetManualLocked, setTargetManualLocked] = useState<boolean>(false);

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

  const actionsDisabled = pending || disabledReason !== "";

  useEffect(() => {
    setTargetPlayerId(battle.opponentPlayerId);
    setSourceManualLocked(false);
    setTargetManualLocked(false);
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
  }, [
    autoFillHint,
    sourceCardId,
    sourceManualLocked,
    sourceOptions,
    targetCardId,
    targetManualLocked,
    targetOptions
  ]);

  return (
    <section className="panel battle-actions" aria-label="动作面板">
      <h2>动作面板</h2>
      <p className="muted">Actor: {actorId}</p>
      {disabledReason ? <p className="muted">{disabledReason}</p> : null}
      <details className="battle-actions__guide">
        <summary>动作说明</summary>
        <ul className="simple-list">
          <li>
            <strong>Pass Priority</strong>：在当前窗口内放弃响应，连续双方都 pass 后通常会结束当前步骤或结算栈。
          </li>
          <li>
            <strong>Advance Phase</strong>：推进阶段（如 main→end / end→main），并触发规则书定义的阶段结算。
          </li>
          <li>
            <strong>Declare Attack / Investigation</strong>：`Source Card` 选本方角色牌，`Target Card` 选敌方角色或地区。
          </li>
          <li>
            <strong>Move Card</strong>：把一张驻场牌移动到目标地区，需同时填写 source/target。
          </li>
          <li>
            <strong>Set/Remove Marker</strong>：用于秘社或卡牌标记增减，需填写 marker type/amount。
          </li>
        </ul>
        <a href="/battle-action-guide.md" target="_blank" rel="noreferrer">
          查看完整说明
        </a>
      </details>

      <div className="battle-actions__grid">
        <label className="battle-actions__field">
          <span>Action Kind</span>
          <select
            aria-label="Action Kind"
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
          <span>Source Card</span>
          <select
            aria-label="Source Card"
            value={sourceCardId}
            onChange={(event) => {
              setSourceCardId(event.target.value);
              setSourceManualLocked(true);
              setError("");
            }}
            disabled={actionsDisabled}
          >
            <option value="">(none)</option>
            {sourceOptions.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </label>

        <label className="battle-actions__field">
          <span>Target Card</span>
          <select
            aria-label="Target Card"
            value={targetCardId}
            onChange={(event) => {
              setTargetCardId(event.target.value);
              setTargetManualLocked(true);
              setError("");
            }}
            disabled={actionsDisabled}
          >
            <option value="">(none)</option>
            {targetOptions.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </label>

        <label className="battle-actions__field">
          <span>Target Player</span>
          <select
            aria-label="Target Player"
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
          <span>Marker Type</span>
          <input
            aria-label="Marker Type"
            value={markerType}
            onChange={(event) => {
              setMarkerType(event.target.value);
              setError("");
            }}
            disabled={actionsDisabled}
          />
        </label>

        <label className="battle-actions__field">
          <span>Marker Amount</span>
          <input
            aria-label="Marker Amount"
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
          <span>Operation Label</span>
          <input
            aria-label="Operation Label"
            value={operationLabel}
            onChange={(event) => {
              setOperationLabel(event.target.value);
              setError("");
            }}
            disabled={actionsDisabled}
            placeholder="optional"
          />
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
          Pass Priority
        </button>
        <button
          type="button"
          className="action-button action-button--secondary"
          disabled={actionsDisabled}
          onClick={() => {
            submitWithKind("advance_phase");
          }}
        >
          Advance Phase
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

    const validation = validateBeforeSubmit(actionKind, sourceCardId, targetCardId, markerType, markerAmount);
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
    actionKind === "set_face_down"
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
