import { Component, lazy, Suspense, type ReactNode } from "react";
import { GameApp } from "../game/GameApp";

const LegacyDebuggerEntry = lazy(() => import("../debugger/LegacyDebuggerEntry"));

class LegacyDebuggerBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  render() {
    if (this.state.failed) {
      // A full document navigation resets both React.lazy's rejection and the
      // browser module map; rerendering the same lazy component cannot retry it.
      return <main role="alert">
        <h1>调试器加载失败</h1>
        <p>请检查网络连接，然后重新加载调试器。</p>
        <a href="/legacy-debugger">重新加载调试器</a>{" "}<a href="/">返回游戏</a>
      </main>;
    }
    return this.props.children;
  }
}

// The cloud game is the default experience; the Go debugger remains explicitly accessible.
export function AppShell() {
  return window.location.pathname.startsWith('/legacy-debugger')
    ? <LegacyDebuggerBoundary><Suspense fallback={<p role="status">正在打开调试器…</p>}><LegacyDebuggerEntry /></Suspense></LegacyDebuggerBoundary>
    : <GameApp />;
}
