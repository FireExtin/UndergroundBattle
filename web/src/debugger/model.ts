import type {
  ActionAcceptedEnvelope,
  ActionRejectedEnvelope,
  CardView,
  DebuggerProtocolEnvelope,
  MockMessageSet,
  StatePatchedEnvelope,
  ViewerId
} from "./protocol";

// Purpose: Derives viewer-specific debugger state from protocol envelopes while keeping React state minimal.

export type DebuggerState = {
  selectedViewer: ViewerId;
  selectedSetId: string;
};

export type DebuggerAction =
  | { type: "viewerSelected"; viewerId: ViewerId }
  | { type: "messageSetSelected"; setId: string };

export function createInitialDebuggerState(messageSets: MockMessageSet[]): DebuggerState {
  return {
    selectedViewer: "P1",
    selectedSetId: messageSets[0]?.id ?? ""
  };
}

export function debuggerReducer(state: DebuggerState, action: DebuggerAction): DebuggerState {
  switch (action.type) {
    case "viewerSelected":
      return {
        ...state,
        selectedViewer: action.viewerId
      };
    case "messageSetSelected":
      return {
        ...state,
        selectedSetId: action.setId
      };
    default:
      return state;
  }
}

export function selectActiveMessageSet(
  messageSets: MockMessageSet[],
  selectedSetId: string
): MockMessageSet | undefined {
  return messageSets.find((messageSet) => messageSet.id === selectedSetId) ?? messageSets[0];
}

export function selectActionLog(messages: DebuggerProtocolEnvelope[]) {
  return messages.filter(
    (message): message is ActionAcceptedEnvelope | ActionRejectedEnvelope =>
      message.name === "ActionAccepted" || message.name === "ActionRejected"
  );
}

export function selectLatestRejected(
  messages: DebuggerProtocolEnvelope[]
): ActionRejectedEnvelope | undefined {
  const rejected = messages.filter(
    (message): message is ActionRejectedEnvelope => message.name === "ActionRejected"
  );
  return rejected[rejected.length - 1];
}

export function selectCurrentPatch(
  messages: DebuggerProtocolEnvelope[],
  viewerId: ViewerId
): StatePatchedEnvelope | undefined {
  const patches = messages.filter(
    (message): message is StatePatchedEnvelope => message.name === "StatePatched"
  );

  const matchingPatches = patches.filter((patch) => {
    if (viewerId === "spectator") {
      return patch.payload.audienceKind === "spectator";
    }

    return patch.payload.audienceKind === "player" && patch.payload.audienceId === viewerId;
  });

  return matchingPatches[matchingPatches.length - 1];
}

export function selectCurrentCards(patch: StatePatchedEnvelope | undefined): CardView[] {
  if (!patch) {
    return [];
  }

  if (patch.payload.playerView) {
    return patch.payload.playerView.board.cards;
  }

  if (patch.payload.spectatorView) {
    return patch.payload.spectatorView.board.cards;
  }

  return [];
}

export function selectCurrentStack(patch: StatePatchedEnvelope | undefined) {
  if (!patch) {
    return [];
  }

  const stack = patch.payload.playerView?.board.stack ?? patch.payload.spectatorView?.board.stack ?? [];
  return [...stack].reverse();
}

export function selectCurrentTurn(patch: StatePatchedEnvelope | undefined) {
  return patch?.payload.playerView?.turn ?? patch?.payload.spectatorView?.turn;
}
