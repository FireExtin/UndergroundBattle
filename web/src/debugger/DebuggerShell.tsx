import { useMemo, useReducer, type ReactNode } from "react";

import { ActionLogPanel } from "./components/ActionLogPanel";
import { LegalityFailurePanel } from "./components/LegalityFailurePanel";
import { MatchStatusPanel } from "./components/MatchStatusPanel";
import { PlayerViewPanel } from "./components/PlayerViewPanel";
import { StackPanel } from "./components/StackPanel";
import { ViewerSwitcher } from "./components/ViewerSwitcher";
import {
  createInitialDebuggerState,
  debuggerReducer,
  selectActionLog,
  selectActiveMessageSet,
  selectCurrentCards,
  selectCurrentPatch,
  selectCurrentStack,
  selectLatestRejected
} from "./model";
import type { MockMessageSet, ViewerId } from "./protocol";

// Purpose: Hosts the minimal match debugger using only React useReducer and mock protocol envelopes.
type DebuggerShellProps = {
  messageSets: MockMessageSet[];
  lede?: string;
  renderControls?: (context: { selectedViewer: ViewerId; activeSet?: MockMessageSet }) => ReactNode;
};

export function DebuggerShell({ messageSets, lede, renderControls }: DebuggerShellProps) {
  const [state, dispatch] = useReducer(debuggerReducer, messageSets, createInitialDebuggerState);
  const activeSet = useMemo(
    () => selectActiveMessageSet(messageSets, state.selectedSetId),
    [messageSets, state.selectedSetId]
  );

  const messages = activeSet?.messages ?? [];
  const currentPatch = selectCurrentPatch(messages, state.selectedViewer);
  const currentCards = selectCurrentCards(currentPatch);
  const currentStack = selectCurrentStack(currentPatch);
  const actionLog = selectActionLog(messages);
  const latestRejected = selectLatestRejected(messages);

  return (
    <main className="app-shell">
      <header className="app-header">
        <p className="eyebrow">隐秘世界 Web 调试器</p>
        <h1>最小对局骨架</h1>
        <p className="lede">
          {lede ?? "当前只消费 mock protocol envelopes，负责展示、调试和视图切换，不承担裁判逻辑。"}
        </p>
        <p className="muted">Source: {activeSet?.label ?? "No data"}</p>
      </header>

      <MatchStatusPanel patch={currentPatch} />
      <ViewerSwitcher
        selectedViewer={state.selectedViewer}
        onViewerSelected={(viewerId) => dispatch({ type: "viewerSelected", viewerId })}
      />
      {renderControls
        ? renderControls({
            selectedViewer: state.selectedViewer,
            activeSet
          })
        : null}

      <div className="debugger-layout">
        <section className="debugger-main">
          <PlayerViewPanel viewerLabel={state.selectedViewer} cards={currentCards} />
        </section>

        <aside className="debugger-sidebar">
          <StackPanel stack={currentStack} />
          <ActionLogPanel entries={actionLog} />
          <LegalityFailurePanel rejection={latestRejected} />
        </aside>
      </div>
    </main>
  );
}
