import type { Action, DebuggerProtocolEnvelope, ViewerId } from "./protocol";

// Purpose: Provides the minimal HTTP client and preset action builders used by the live sandbox debugger.

export type SetupStepStatus = {
  step: number;
  title: string;
  completed: boolean;
};

export type SetupRegionView = {
  cardId: string;
  definitionId: string;
  name: string;
  type: string;
  description?: string;
  faq?: string;
  influenceLimit: number;
  score: number;
  regionOrder: number;
  worldDeckIndex: number;
  worldDeckRemain: number;
};

export type SetupState = {
  active: boolean;
  completed: boolean;
  currentStep: number;
  lifecycle?: {
    kind: "reset" | "setup" | "match_active" | "match_finished";
    setupStep?: number;
    finishedRevision?: number;
    reportPath?: string;
  };
  seed: number;
  steps: SetupStepStatus[];
  p1Societies?: string[];
  p2Societies?: string[];
  markerPoolReady: boolean;
  worldDeckCount: number;
  revealedRegions?: SetupRegionView[];
  playerDeckCount: Record<string, number>;
  playerHandCount: Record<string, number>;
  mulliganUsed: Record<string, boolean>;
  startingPlayerId?: string;
  previousLoserPlayer?: string;
  lastStepMessage?: string;
  runtimeIgnoredScopes?: Record<string, string[]>;
  runtimeNotes?: Record<string, string>;
};

export type SetupStartInput = {
  seed?: number;
  p1Societies?: string[];
  p2Societies?: string[];
  previousLoserPlayer?: string;
};

export type SetupAdvanceInput = {
  p1Societies?: string[];
  p2Societies?: string[];
  mulliganBottomCount?: Record<string, number>;
  startingPlayerId?: string;
  usePreviousLoserChoice?: boolean;
};

export type LiveActionPresetId =
  | "passPriority"
  | "advancePhase"
  | "revealOwnSecret"
  | "inspectOwnSecret"
  | "castReadMinds"
  | "castDreamMaze"
  | "equipAlloyKnuckles"
  | "setSecretSocietyMarker"
  | "removeSecretSocietyMarker"
  | "setOwnTableFaceDown"
  | "useFirstPlayerPrivilege";

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
  { id: "equipAlloyKnuckles", label: "Equip 合金指虎 (BQ022)" },
  { id: "setSecretSocietyMarker", label: "Set Secret Marker" },
  { id: "removeSecretSocietyMarker", label: "Remove Secret Marker" },
  { id: "setOwnTableFaceDown", label: "Set Own Table Face-Down" },
  { id: "useFirstPlayerPrivilege", label: "Use First-Player Privilege" }
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

export async function resetDebuggerSession(): Promise<DebuggerProtocolEnvelope[]> {
  const response = await fetch("/api/debugger/reset", {
    method: "POST"
  });

  return readJSONResponse<DebuggerProtocolEnvelope[]>(response);
}

export async function fetchBattleSetupState(): Promise<SetupState> {
  const response = await fetch("/api/battle/setup/state", undefined);
  return readJSONResponse<SetupState>(response);
}

export async function startBattleSetup(input: SetupStartInput): Promise<SetupState> {
  const response = await fetch("/api/battle/setup/start", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(input)
  });
  return readJSONResponse<SetupState>(response);
}

export async function advanceBattleSetup(input: SetupAdvanceInput): Promise<SetupState> {
  const response = await fetch("/api/battle/setup/advance", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(input)
  });
  return readJSONResponse<SetupState>(response);
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
    case "setSecretSocietyMarker":
      return {
        ...action,
        kind: "set_marker",
        targetPlayerId: viewerId,
        markerType: "secret_society",
        markerAmount: 1
      };
    case "removeSecretSocietyMarker":
      return {
        ...action,
        kind: "remove_marker",
        targetPlayerId: viewerId,
        markerType: "secret_society",
        markerAmount: 1
      };
    case "setOwnTableFaceDown":
      return {
        ...action,
        kind: "set_face_down",
        cardId: ownTableCardId(viewerId)
      };
    case "useFirstPlayerPrivilege":
      return {
        ...action,
        kind: "use_first_player_privilege"
      };
  }
}

export function buildActionFromCustomJSON(
  viewerId: Exclude<ViewerId, "spectator">,
  rawJSON: string,
  sequence: number
): Action {
  const parsed = parseCustomActionJSON(rawJSON);
  const kind = readNonEmptyString(parsed.kind);
  if (kind === "") {
    throw new Error("Custom action JSON must include a non-empty kind.");
  }

  const id = readNonEmptyString(parsed.id);
  const actorId = readNonEmptyString(parsed.actorId);

  return {
    ...(parsed as Action),
    id: id === "" ? `act-web-${viewerId.toLowerCase()}-${sequence}` : id,
    actorId: actorId === "" ? viewerId : actorId,
    kind
  };
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

function parseCustomActionJSON(rawJSON: string): Record<string, unknown> {
  let parsed: unknown;
  try {
    parsed = JSON.parse(rawJSON);
  } catch {
    throw new Error("Custom action must be valid JSON.");
  }

  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("Custom action JSON must be an object.");
  }

  return parsed as Record<string, unknown>;
}

function readNonEmptyString(value: unknown) {
  if (typeof value !== "string") {
    return "";
  }

  return value.trim();
}
