import { BattleShell } from "../battle/BattleShell";
import { defaultMockMessageSets } from "../debugger/mockProtocol";

// Purpose: Boots the playable battle table first, while retaining mock protocol data as an offline fallback.
export function AppShell() {
  return <BattleShell fallbackMessageSets={defaultMockMessageSets} />;
}
