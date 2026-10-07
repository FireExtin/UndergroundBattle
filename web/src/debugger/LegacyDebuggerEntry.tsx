import { LiveDebuggerShell } from "./LiveDebuggerShell";
import { defaultMockMessageSets } from "./mockProtocol";

// This optional route loads its protocol UI and fallback data on demand.
export default function LegacyDebuggerEntry() {
  return <LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />;
}
