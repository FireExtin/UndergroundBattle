import { useEffect, useReducer, useRef, useState, type Dispatch, type MutableRefObject } from "react";

import { DebuggerShell } from "./DebuggerShell";
import { ActionControlsPanel } from "./components/ActionControlsPanel";
import {
  buildActionFromPreset,
  buildActionFromCustomJSON,
  fetchDebuggerMessages,
  liveActionPresets,
  resetDebuggerSession,
  submitDebuggerAction,
  type LiveActionPresetId
} from "./live";
import {
  type LiveDebuggerAction,
  createInitialLiveDebuggerState,
  liveDebuggerReducer
} from "./liveModel";
import type { MockMessageSet, StatePatchedEnvelope, ViewerId } from "./protocol";

// Purpose: Switches the debugger from static mock-only mode to a minimal live sandbox fed by the Go HTTP server.
type LiveDebuggerShellProps = {
  fallbackMessageSets: MockMessageSet[];
};

export function LiveDebuggerShell({ fallbackMessageSets }: LiveDebuggerShellProps) {
  const [state, dispatch] = useReducer(
    liveDebuggerReducer,
    fallbackMessageSets,
    createInitialLiveDebuggerState
  );
  const [customActionDraft, setCustomActionDraft] = useState('{"kind":"pass_priority"}');
  const [customActionError, setCustomActionError] = useState("");
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

  const messageSets: MockMessageSet[] = [
    {
      id: "live-sandbox",
      label: state.mode === "live" ? "Live Sandbox" : "Mock Fallback",
      messages: state.messages
    }
  ];

  return (
    <DebuggerShell
      messageSets={messageSets}
      lede={
        state.mode === "live"
          ? "当前连接 Go 规则核的最小 HTTP sandbox，可以提交预置动作并观察真实 revision、stack、priority 与投影变化。"
          : "当前未连接到 Go sandbox，已退回 mock protocol 数据。规则真相源仍然只存在于 Go。"
      }
      renderControls={({ selectedViewer, currentPatch }) => (
        <ActionControlsPanel
          selectedViewer={selectedViewer}
          pending={state.loading || state.submitting}
          presets={liveActionPresets}
          disabledReason={disabledReason(
            state.mode,
            selectedViewer,
            state.errorMessage,
            currentWinnerPlayerId(currentPatch)
          )}
          customActionDraft={customActionDraft}
          customActionError={customActionError}
          onActionSelected={(presetId) =>
            void handleActionSelection(
              presetId,
              selectedViewer,
              nextActionNumber,
              dispatch,
              setCustomActionError
            )
          }
          onCustomActionDraftChanged={(draft) => {
            setCustomActionDraft(draft);
            setCustomActionError("");
          }}
          onCustomActionSubmit={() =>
            void handleCustomActionSelection(
              customActionDraft,
              selectedViewer,
              nextActionNumber,
              dispatch,
              setCustomActionError
            )
          }
          onReload={() => void reloadLiveMessages(dispatch, fallbackMessageSets)}
          onReset={() => void resetLiveSession(dispatch, fallbackMessageSets)}
        />
      )}
    />
  );
}

async function reloadLiveMessages(
  dispatch: Dispatch<LiveDebuggerAction>,
  fallbackMessageSets: MockMessageSet[]
) {
  dispatch(loadStartedAction());

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
  dispatch(loadStartedAction());

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

async function handleActionSelection(
  presetId: LiveActionPresetId,
  selectedViewer: ViewerId,
  nextActionNumber: MutableRefObject<number>,
  dispatch: Dispatch<LiveDebuggerAction>,
  setCustomActionError: Dispatch<string>
) {
  if (selectedViewer === "spectator") {
    return;
  }

  dispatch({ type: "submitStarted" });
  setCustomActionError("");

  try {
    const action = buildActionFromPreset(
      selectedViewer,
      presetId,
      nextActionNumber.current
    );
    nextActionNumber.current += 1;
    const messages = await submitDebuggerAction(action);
    dispatch({ type: "submitSucceeded", messages });
  } catch (error) {
    dispatch({
      type: "submitFailed",
      errorMessage: error instanceof Error ? error.message : "Action submission failed."
    });
  }
}

async function handleCustomActionSelection(
  customActionDraft: string,
  selectedViewer: ViewerId,
  nextActionNumber: MutableRefObject<number>,
  dispatch: Dispatch<LiveDebuggerAction>,
  setCustomActionError: Dispatch<string>
) {
  if (selectedViewer === "spectator") {
    return;
  }

  let action;
  try {
    action = buildActionFromCustomJSON(
      selectedViewer,
      customActionDraft,
      nextActionNumber.current
    );
  } catch (error) {
    setCustomActionError(error instanceof Error ? error.message : "Custom action is invalid.");
    return;
  }

  dispatch({ type: "submitStarted" });
  setCustomActionError("");

  try {
    nextActionNumber.current += 1;
    const messages = await submitDebuggerAction(action);
    dispatch({ type: "submitSucceeded", messages });
  } catch (error) {
    dispatch({
      type: "submitFailed",
      errorMessage: error instanceof Error ? error.message : "Action submission failed."
    });
  }
}

function disabledReason(
  mode: "loading" | "live" | "fallback",
  selectedViewer: ViewerId,
  error: string,
  winnerPlayerId: string
) {
  if (mode !== "live") {
    return error || "Live server unavailable. Showing mock protocol data."
  }

  if (winnerPlayerId !== "") {
    return `Game over. Winner: ${winnerPlayerId}`
  }

  if (selectedViewer === "spectator") {
    return "Spectator cannot submit actions."
  }

  return "";
}

function loadStartedAction() {
  return { type: "loadStarted" } as const;
}

function currentWinnerPlayerId(
  patch: StatePatchedEnvelope | undefined
) {
  return patch?.payload.playerView?.match.winnerPlayerId ?? patch?.payload.spectatorView?.match.winnerPlayerId ?? "";
}
