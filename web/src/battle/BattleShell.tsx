import { useEffect, useReducer, useRef, useState, type Dispatch } from "react";

import {
  fetchDebuggerMessages,
  resetDebuggerSession,
  submitDebuggerAction
} from "../debugger/live";
import {
  createInitialLiveDebuggerState,
  liveDebuggerReducer,
  type LiveDebuggerAction
} from "../debugger/liveModel";
import type { MockMessageSet } from "../debugger/protocol";
import {
  ActionComposer,
  type ComposerActionInput,
  type ComposerAutoFillHint
} from "./components/ActionComposer";
import { BattleTable, type BattleCardPick } from "./components/BattleTable";
import {
  deriveBattleInfoLogs,
  deriveBattleState,
  type BattleInfoLogEntry,
  type BattlePlayerId
} from "./model";

// Purpose: Hosts the playable table-facing client shell over the existing authoritative sandbox HTTP endpoints.

type BattleShellProps = {
  fallbackMessageSets: MockMessageSet[];
};

export function BattleShell({ fallbackMessageSets }: BattleShellProps) {
  const [state, dispatch] = useReducer(
    liveDebuggerReducer,
    fallbackMessageSets,
    createInitialLiveDebuggerState
  );
  const [localPlayerId, setLocalPlayerId] = useState<BattlePlayerId>("P1");
  const [autoFillHint, setAutoFillHint] = useState<ComposerAutoFillHint | undefined>(undefined);
  const [logFilter, setLogFilter] = useState<"all" | "accepted" | "rejected" | "system">("all");
  const nextActionNumber = useRef(1);

  useEffect(() => {
    let cancelled = false;

    async function loadMessages() {
      dispatch({ type: "loadStarted" });

      try {
        const messages = await fetchDebuggerMessages();
        if (cancelled) {
          return;
        }
        dispatch({ type: "loadSucceeded", messages });
      } catch {
        if (cancelled) {
          return;
        }

        dispatch({
          type: "loadFellBack",
          messages: fallbackMessageSets[0]?.messages ?? [],
          errorMessage: "Live server unavailable. Showing mock protocol data."
        });
      }
    }

    void loadMessages();

    return () => {
      cancelled = true;
    };
  }, [fallbackMessageSets]);

  const battle = deriveBattleState(state.messages, localPlayerId);
  const infoLogs = deriveBattleInfoLogs(state.messages);
  const winnerPlayerId = battle.match.winnerPlayerId ?? battle.score.winnerPlayerId ?? "";
  const disabledReason = deriveDisabledReason(state.mode, state.errorMessage, battle.match.status, winnerPlayerId);

  return (
    <main className="battle-shell">
      <header className="panel battle-shell__banner">
        <p className="muted">Source: {state.mode === "live" ? "Live Sandbox" : "Mock Fallback"}</p>
        {disabledReason ? <p className="muted">{disabledReason}</p> : null}
        <div className="battle-shell__banner-actions">
          <button
            type="button"
            className="action-button action-button--secondary"
            disabled={state.loading || state.submitting}
            onClick={() => void reloadLiveMessages(dispatch, fallbackMessageSets)}
          >
            刷新状态
          </button>
          <button
            type="button"
            className="action-button action-button--secondary"
            aria-label="重开对局"
            disabled={state.loading || state.submitting}
            onClick={() => void resetLiveSession(dispatch, fallbackMessageSets)}
          >
            重开对局
          </button>
        </div>
      </header>

      <BattleTable
        battle={battle}
        localPlayerId={localPlayerId}
        onLocalPlayerChanged={setLocalPlayerId}
        onCardPicked={(picked) => {
          setAutoFillHint((previous) => nextAutoFillHint(previous, picked));
        }}
      />

      <ActionComposer
        actorId={localPlayerId}
        battle={battle}
        pending={state.loading || state.submitting}
        disabledReason={disabledReason}
        autoFillHint={autoFillHint}
        onSubmitAction={(action) =>
          void submitBattleAction(action, localPlayerId, nextActionNumber.current, dispatch, () => {
            nextActionNumber.current += 1;
          })
        }
      />

      <section className="panel battle-info-logs" aria-label="对局信息日志">
        <div className="battle-info-logs__header">
          <h2>对局信息日志</h2>
          <label className="battle-info-logs__filter">
            <span>过滤</span>
            <select
              aria-label="日志过滤"
              value={logFilter}
              onChange={(event) => {
                setLogFilter(event.target.value as "all" | "accepted" | "rejected" | "system");
              }}
            >
              <option value="all">全部</option>
              <option value="accepted">accepted</option>
              <option value="rejected">rejected</option>
              <option value="system">system</option>
            </select>
          </label>
        </div>
        <InfoLogList entries={filterInfoLogs(infoLogs, logFilter)} />
      </section>
    </main>
  );
}

async function submitBattleAction(
  action: ComposerActionInput,
  actorId: BattlePlayerId,
  actionNumber: number,
  dispatch: Dispatch<LiveDebuggerAction>,
  afterSuccess: () => void
) {
  dispatch({ type: "submitStarted" });

  try {
    const messages = await submitDebuggerAction({
      id: `act-battle-${actorId.toLowerCase()}-${actionNumber}`,
      actorId,
      ...action
    });
    afterSuccess();
    dispatch({ type: "submitSucceeded", messages });
  } catch (error) {
    dispatch({
      type: "submitFailed",
      errorMessage: error instanceof Error ? error.message : "Action submission failed."
    });
  }
}

async function reloadLiveMessages(
  dispatch: Dispatch<LiveDebuggerAction>,
  fallbackMessageSets: MockMessageSet[]
) {
  dispatch({ type: "loadStarted" });

  try {
    const messages = await fetchDebuggerMessages();
    dispatch({ type: "loadSucceeded", messages });
  } catch {
    dispatch({
      type: "loadFellBack",
      messages: fallbackMessageSets[0]?.messages ?? [],
      errorMessage: "Live server unavailable. Showing mock protocol data."
    });
  }
}

async function resetLiveSession(
  dispatch: Dispatch<LiveDebuggerAction>,
  fallbackMessageSets: MockMessageSet[]
) {
  dispatch({ type: "loadStarted" });

  try {
    const messages = await resetDebuggerSession();
    dispatch({ type: "loadSucceeded", messages });
  } catch {
    dispatch({
      type: "loadFellBack",
      messages: fallbackMessageSets[0]?.messages ?? [],
      errorMessage: "Live server unavailable. Showing mock protocol data."
    });
  }
}

function deriveDisabledReason(
  mode: "loading" | "live" | "fallback",
  error: string,
  matchStatus: "active" | "finished",
  winnerPlayerId: string
) {
  if (mode !== "live") {
    return error || "Live server unavailable. Showing mock protocol data.";
  }

  if (matchStatus === "finished") {
    return winnerPlayerId === "" ? "Game over." : `Game over. Winner: ${winnerPlayerId}`;
  }

  return "";
}

function nextAutoFillHint(
  previous: ComposerAutoFillHint | undefined,
  picked: BattleCardPick
): ComposerAutoFillHint {
  const token = (previous?.token ?? 0) + 1;
  if (picked.intent === "source") {
    return {
      token,
      sourceCardId: picked.cardId
    };
  }

  return {
    token,
    targetCardId: picked.cardId
  };
}

function filterInfoLogs(
  entries: BattleInfoLogEntry[],
  filter: "all" | "accepted" | "rejected" | "system"
) {
  if (filter === "all") {
    return entries;
  }

  return entries.filter((entry) => entry.kind === filter);
}

function InfoLogList({ entries }: { entries: BattleInfoLogEntry[] }) {
  if (entries.length === 0) {
    return <p className="muted">(暂无日志)</p>;
  }

  return (
    <ol className="battle-info-logs__list">
      {entries.map((entry) => (
        <li key={entry.id} className={`battle-info-logs__item battle-info-logs__item--${entry.kind}`}>
          <p>
            <strong>{entry.summary}</strong>
          </p>
          <p className="muted">{entry.detail}</p>
        </li>
      ))}
    </ol>
  );
}
