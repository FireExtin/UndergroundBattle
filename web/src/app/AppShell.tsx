import { LiveDebuggerShell } from "../debugger/LiveDebuggerShell";
import { defaultMockMessageSets } from "../debugger/mockProtocol";
import { GameApp } from "../game/GameApp";

// The cloud game is the default experience; the Go debugger remains explicitly accessible.
export function AppShell() {
  return window.location.pathname.startsWith('/legacy-debugger')
    ? <LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />
    : <GameApp />;
}
