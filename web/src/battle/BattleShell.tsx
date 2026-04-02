import { useEffect, useReducer, useRef, useState, type Dispatch } from "react";

import {
  advanceBattleSetup,
  fetchBattleSetupState,
  fetchDebuggerMessages,
  resetDebuggerSession,
  startBattleSetup,
  submitDebuggerAction,
  type SetupAdvanceInput,
  type SetupStartInput,
  type SetupState
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
import { SetupWizard } from "./components/SetupWizard";
import {
  deriveBattleInfoLogs,
  deriveBattleState,
  type BattleInfoLogEntry,
  type BattlePlayerId
} from "./model";

// Purpose: Hosts setup wizard + playable battle table on top of authoritative sandbox endpoints.

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
  const [setupState, setSetupState] = useState<SetupState | null>(null);
  const [setupLoading, setSetupLoading] = useState<boolean>(true);
  const [setupPending, setSetupPending] = useState<boolean>(false);
  const [setupErrorMessage, setSetupErrorMessage] = useState<string>("");
  const nextActionNumber = useRef(1);

  useEffect(() => {
    let cancelled = false;

    async function initializeSetup() {
      setSetupLoading(true);
      setSetupErrorMessage("");

      try {
        const setup = await fetchBattleSetupState();
        if (cancelled) {
          return;
        }
        setSetupState(setup);
        setSetupLoading(false);
        if (setup.active && setup.completed) {
          await reloadLiveMessages(dispatch, fallbackMessageSets);
        }
      } catch {
        if (cancelled) {
          return;
        }
        setSetupLoading(false);
        setSetupErrorMessage("无法连接服务端，已切换离线协议演示。");
        dispatch({
          type: "loadFellBack",
          messages: fallbackMessageSets[0]?.messages ?? [],
          errorMessage: "实时服务不可用，已切换到离线协议数据。"
        });
      }
    }

    void initializeSetup();

    return () => {
      cancelled = true;
    };
  }, [fallbackMessageSets]);

  const battle = deriveBattleState(state.messages, localPlayerId);
  const infoLogs = deriveBattleInfoLogs(state.messages);
  const winnerPlayerId = battle.match.winnerPlayerId ?? battle.score.winnerPlayerId ?? "";
  const disabledReason = deriveDisabledReason(state.mode, state.errorMessage, battle.match.status, winnerPlayerId);

  if (setupLoading) {
    return (
      <main className="battle-shell">
        <section className="panel battle-setup">
          <h1>加载开局设置中...</h1>
        </section>
      </main>
    );
  }

  if (!setupState?.active || !setupState.completed) {
    return (
      <main className="battle-shell">
        <SetupWizard
          setupState={setupState}
          pending={setupPending}
          errorMessage={setupErrorMessage}
          onStartSetup={(input) => startSetupFlow(input)}
          onAdvanceSetup={(input) => advanceSetupFlow(input)}
          onRefreshSetup={() => refreshSetupFlow()}
        />
      </main>
    );
  }

  return (
    <main className="battle-shell">
      <header className="panel battle-shell__banner">
        <p className="muted">数据源：{state.mode === "live" ? "实时沙盒" : "离线回退"}</p>
        {disabledReason ? <p className="muted">{disabledReason}</p> : null}
        <div className="battle-shell__banner-actions">
          <button
            type="button"
            className="action-button action-button--secondary"
            disabled={state.loading || state.submitting || setupPending}
            onClick={() => void reloadLiveMessages(dispatch, fallbackMessageSets)}
          >
            刷新状态
          </button>
          <button
            type="button"
            className="action-button action-button--secondary"
            aria-label="重置并返回开局设置"
            disabled={state.loading || state.submitting || setupPending}
            onClick={() => void resetAndReturnToSetup()}
          >
            重置并返回开局设置
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
              <option value="accepted">已接受</option>
              <option value="rejected">已拒绝</option>
              <option value="system">系统</option>
            </select>
          </label>
        </div>
        <InfoLogList entries={filterInfoLogs(infoLogs, logFilter)} />
      </section>
    </main>
  );

  async function startSetupFlow(input: SetupStartInput) {
    setSetupPending(true);
    setSetupErrorMessage("");
    try {
      const nextSetup = await startBattleSetup(input);
      setSetupState(nextSetup);
    } catch (error) {
      setSetupErrorMessage(error instanceof Error ? error.message : "开局设置启动失败");
    } finally {
      setSetupPending(false);
    }
  }

  async function advanceSetupFlow(input: SetupAdvanceInput) {
    setSetupPending(true);
    setSetupErrorMessage("");
    try {
      const nextSetup = await advanceBattleSetup(input);
      setSetupState(nextSetup);
      if (nextSetup.completed) {
        if (nextSetup.startingPlayerId === "P1" || nextSetup.startingPlayerId === "P2") {
          setLocalPlayerId(nextSetup.startingPlayerId);
        }
        nextActionNumber.current = 1;
        setAutoFillHint(undefined);
        await reloadLiveMessages(dispatch, fallbackMessageSets);
      }
    } catch (error) {
      setSetupErrorMessage(error instanceof Error ? error.message : "执行开局步骤失败");
    } finally {
      setSetupPending(false);
    }
  }

  async function refreshSetupFlow() {
    setSetupPending(true);
    setSetupErrorMessage("");
    try {
      const nextSetup = await fetchBattleSetupState();
      setSetupState(nextSetup);
      if (nextSetup.completed) {
        await reloadLiveMessages(dispatch, fallbackMessageSets);
      }
    } catch (error) {
      setSetupErrorMessage(error instanceof Error ? error.message : "刷新设置状态失败");
    } finally {
      setSetupPending(false);
    }
  }

  async function resetAndReturnToSetup() {
    setSetupPending(true);
    setSetupErrorMessage("");
    dispatch({ type: "loadStarted" });
    try {
      await resetDebuggerSession();
      const nextSetup = await fetchBattleSetupState();
      setSetupState(nextSetup);
      dispatch({ type: "loadSucceeded", messages: [] });
      nextActionNumber.current = 1;
      setAutoFillHint(undefined);
      if (nextSetup.completed) {
        await reloadLiveMessages(dispatch, fallbackMessageSets);
      }
    } catch (error) {
      setSetupErrorMessage(error instanceof Error ? error.message : "重置失败");
      dispatch({
        type: "loadFellBack",
        messages: fallbackMessageSets[0]?.messages ?? [],
        errorMessage: "实时服务不可用，已切换到离线协议数据。"
      });
    } finally {
      setSetupPending(false);
    }
  }
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
      errorMessage: error instanceof Error ? error.message : "动作提交失败。"
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
      errorMessage: "实时服务不可用，已切换到离线协议数据。"
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
    return error || "实时服务不可用，当前为离线演示数据。";
  }

  if (matchStatus === "finished") {
    return winnerPlayerId === "" ? "对局结束。" : `对局结束，胜者：${winnerPlayerId}`;
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
