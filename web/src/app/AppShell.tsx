import { LiveDebuggerShell } from "../debugger/LiveDebuggerShell";
import { defaultMockMessageSets } from "../debugger/mockProtocol";

// Purpose: Boots the live sandbox debugger first, while retaining mock protocol data as an offline fallback.
export function AppShell() {
  return <LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />;
}
