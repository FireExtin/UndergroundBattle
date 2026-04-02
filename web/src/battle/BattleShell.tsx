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
import { ActionComposer, type ComposerActionInput } from "./components/ActionComposer";
import { BattleTable } from "./components/BattleTable";
import { deriveBattleState, type BattlePlayerId } from "./model";

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
      />

      <ActionComposer
        actorId={localPlayerId}
        battle={battle}
        pending={state.loading || state.submitting}
        disabledReason={disabledReason}
        onSubmitAction={(action) =>
          void submitBattleAction(action, localPlayerId, nextActionNumber.current, dispatch, () => {
            nextActionNumber.current += 1;
          })
        }
      />
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
