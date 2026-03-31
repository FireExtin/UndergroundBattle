import type { Action, DebuggerProtocolEnvelope, ViewerId } from "./protocol";

// Purpose: Provides the minimal HTTP client and preset action builders used by the live sandbox debugger.

export type LiveActionPresetId =
  | "passPriority"
  | "advancePhase"
  | "revealOwnSecret"
  | "inspectOwnSecret"
  | "castReadMinds"
  | "castDreamMaze"
  | "equipAlloyKnuckles";

export type LiveActionPreset = {
  id: LiveActionPresetId;
  label: string;
};

export const liveActionPresets: LiveActionPreset[] = [
  { id: "passPriority", label: "Pass Priority" },
  { id: "advancePhase", label: "Advance Phase" },
  { id: "revealOwnSecret", label: "Reveal Own Secret" },
  { id: "inspectOwnSecret", label: "Inspect Own Secret" },
  { id: "castReadMinds", label: "Cast 读心术 (BQ010)" },
  { id: "castDreamMaze", label: "Cast 多重梦境迷宫 (BQ005)" },
  { id: "equipAlloyKnuckles", label: "Equip 合金指虎 (BQ022)" }
];

export async function fetchDebuggerMessages(): Promise<DebuggerProtocolEnvelope[]> {
  const response = await fetch("/api/debugger/messages", undefined);
  return readJSONResponse<DebuggerProtocolEnvelope[]>(response);
}

export async function submitDebuggerAction(action: Action): Promise<DebuggerProtocolEnvelope[]> {
  const response = await fetch("/api/debugger/actions", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(action)
  });

  return readJSONResponse<DebuggerProtocolEnvelope[]>(response);
}

export function buildActionFromPreset(
  viewerId: Exclude<ViewerId, "spectator">,
  presetId: LiveActionPresetId,
  sequence: number
): Action {
  const action: Action = {
    id: `act-web-${viewerId.toLowerCase()}-${sequence}`,
    actorId: viewerId,
    kind: "pass_priority"
  };

  switch (presetId) {
    case "passPriority":
      return action;
    case "advancePhase":
      return {
        ...action,
        kind: "advance_phase"
      };
    case "revealOwnSecret":
      return {
        ...action,
        kind: "reveal_card",
        cardId: ownSecretCardId(viewerId)
      };
    case "inspectOwnSecret":
      return {
        ...action,
        kind: "inspect_card",
        cardId: ownSecretCardId(viewerId)
      };
    case "castReadMinds":
      return {
        ...action,
        kind: "queue_operation",
        cardId: "BQ010",
        targetPlayerId: opposingPlayerId(viewerId)
      };
    case "castDreamMaze":
      return {
        ...action,
        kind: "queue_operation",
        cardId: "BQ005",
        targetCardId: opposingTableCardId(viewerId)
      };
    case "equipAlloyKnuckles":
      return {
        ...action,
        kind: "queue_operation",
        cardId: "BQ022",
        targetCardId: ownTableCardId(viewerId)
      };
  }
}

async function readJSONResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }

  return (await response.json()) as T;
}

function ownSecretCardId(viewerId: Exclude<ViewerId, "spectator">) {
  return viewerId === "P1" ? "P1-HAND-SECRET" : "P2-HAND-SECRET";
}

function ownTableCardId(viewerId: Exclude<ViewerId, "spectator">) {
  return viewerId === "P1" ? "P1-TABLE-1" : "P2-TABLE-1";
}

function opposingTableCardId(viewerId: Exclude<ViewerId, "spectator">) {
  return viewerId === "P1" ? "P2-TABLE-1" : "P1-TABLE-1";
}

function opposingPlayerId(viewerId: Exclude<ViewerId, "spectator">) {
  return viewerId === "P1" ? "P2" : "P1";
}
