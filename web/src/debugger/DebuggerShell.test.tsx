import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { DebuggerShell } from "./DebuggerShell";
import { defaultMockMessageSets } from "./mockProtocol";
import type { MockMessageSet } from "./protocol";

// Purpose: Verifies the minimal debugger shell renders mock protocol state and viewer-specific hidden information.
describe("DebuggerShell", () => {
  it("renders the stack with the top item first", () => {
    render(<DebuggerShell messageSets={defaultMockMessageSets} />);

    const stack = screen.getByRole("list", { name: "Stack Items" });
    const items = within(stack).getAllByRole("listitem");
    expect(items.map((item) => item.textContent)).toEqual([
      expect.stringContaining("Quick Barrier"),
      expect.stringContaining("Dream Snare"),
      expect.stringContaining("Opening Move")
    ]);
  });

  it("renders the action log in message order", () => {
    render(<DebuggerShell messageSets={defaultMockMessageSets} />);

    const log = screen.getByRole("list", { name: "Action Log Entries" });
    const items = within(log).getAllByRole("listitem");

    expect(items).toHaveLength(2);
    expect(items[0]).toHaveTextContent("ActionAccepted");
    expect(items[0]).toHaveTextContent("act-queue-1");
    expect(items[1]).toHaveTextContent("ActionRejected");
    expect(items[1]).toHaveTextContent("act-illegal-1");
  });

  it("shows the current revision for the selected viewer", () => {
    render(<DebuggerShell messageSets={defaultMockMessageSets} />);

    expect(screen.getByText("Current Revision")).toBeInTheDocument();
    expect(screen.getByText("7")).toBeInTheDocument();
  });

  it("shows the active player plus public score and winner", () => {
    render(<DebuggerShell messageSets={defaultMockMessageSets} />);

    const panel = screen.getByRole("heading", { name: "Match Status" }).closest("section");
    if (!panel) {
      throw new Error("Match Status panel not found");
    }

    const scoped = within(panel);
    expect(scoped.getByText("Active Player")).toBeInTheDocument();
    expect(scoped.getAllByText("P1")).toHaveLength(2);
    expect(scoped.getByText(/P1: 2/)).toBeInTheDocument();
    expect(scoped.getByText(/P2: 1/)).toBeInTheDocument();
    expect(scoped.getByText("First-Player Privilege")).toBeInTheDocument();
    expect(scoped.getByText("available")).toBeInTheDocument();
    expect(scoped.getByText("Winner")).toBeInTheDocument();
  });

  it("renders structured legality failure details", () => {
    render(<DebuggerShell messageSets={defaultMockMessageSets} />);

    expect(screen.getByText("LEGALITY_FAILED_NOT_YOUR_PRIORITY")).toBeInTheDocument();
    expect(screen.getByText("rules.legality.not_your_priority")).toBeInTheDocument();
    expect(screen.getByText("turn.priority")).toBeInTheDocument();
    expect(screen.getByText(/priorityPlayerId: P2/)).toBeInTheDocument();
  });

  it("switches per-player views and preserves hidden-information differences", () => {
    render(<DebuggerShell messageSets={defaultMockMessageSets} />);

    expect(screen.getByText("Secret Archive")).toBeInTheDocument();
    expect(screen.queryByText("P1-HAND-SECRET")).not.toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "View as P2" }));
    expect(screen.queryByText("Secret Archive")).not.toBeInTheDocument();
    expect(screen.queryByText("P1-HAND-SECRET")).not.toBeInTheDocument();
    const viewerCards = screen.getByRole("list", { name: "Viewer Cards" });
    expect(within(viewerCards).getAllByText("hidden").length).toBeGreaterThan(0);

    fireEvent.click(screen.getByRole("button", { name: "View as spectator" }));
    expect(screen.queryByText("Secret Archive")).not.toBeInTheDocument();
    expect(screen.queryByText("P1-HAND-SECRET")).not.toBeInTheDocument();
  });

  it("shows face-down status and marker summary when patch includes them", () => {
    const faceDownMarkersMessageSet: MockMessageSet = {
      id: "face-down-markers",
      label: "FaceDown + Markers",
      messages: [
        {
          version: "0.1.0",
          kind: "view",
          messageId: "msg-state-patched-face-down-markers",
          name: "StatePatched",
          revision: 1,
          payload: {
            type: "StatePatched",
            audienceKind: "player",
            audienceId: "P1",
            revision: {
              number: 1
            },
            event: {
              id: "evt:1",
              actionId: "act:1",
              operationId: "op:1",
              kind: "face_down_set",
              revisionNumber: 1,
              stackDepth: 0
            },
            playerView: {
              gameId: "game-face-down-markers",
              viewerPlayerId: "P1",
              revision: {
                number: 1
              },
              match: {
                status: "active"
              },
              turn: {
                turnNumber: 1,
                activePlayerId: "P1",
                priorityPlayerId: "P1",
                firstPlayerPrivilegeUsed: true,
                priority: {
                  currentPlayerId: "P1",
                  passCount: 0,
                  windowKind: "action"
                },
                phase: {
                  name: "main",
                  step: "action",
                  allowsStack: true,
                  stepEnded: false
                }
              },
              score: {
                byPlayer: { P1: 0, P2: 0 },
                victoryThreshold: 10
              },
              markers: {
                secret_society: 2
              },
              board: {
                stack: [],
                resolved: [],
                randomResults: [],
                cards: [
                  {
                    cardId: "P1-TABLE-HIDDEN-1",
                    name: "Hidden Operator",
                    ownerId: "P1",
                    zone: "table",
                    visibility: "visible",
                    revealed: false,
                    faceDown: true,
                    exhausted: false,
                    destroyed: false,
                    keywords: [],
                    stats: {
                      combat: 1,
                      defense: 1,
                      influence: 0,
                      investigation: 1
                    },
                    counters: {
                      damage: 0,
                      influence: 0
                    }
                  }
                ]
              }
            }
          }
        }
      ]
    };

    render(<DebuggerShell messageSets={[faceDownMarkersMessageSet]} />);

    expect(screen.getByText(/face-down: yes/i)).toBeInTheDocument();
    expect(screen.getByText(/secret_society: 2/i)).toBeInTheDocument();
    expect(screen.getByText("First-Player Privilege")).toBeInTheDocument();
    expect(screen.getByText("used")).toBeInTheDocument();
  });

  it("shows shield counters and shield_consumed event details", () => {
    const shieldMessageSet: MockMessageSet = {
      id: "shield-consumed",
      label: "Shield Consumed",
      messages: [
        {
          version: "0.1.0",
          kind: "event",
          messageId: "msg-action-accepted-shield",
          name: "ActionAccepted",
          revision: 8,
          payload: {
            type: "ActionAccepted",
            action: {
              id: "act-shield-1",
              actorId: "P1",
              kind: "queue_operation",
              cardId: "BQ005",
              targetCardId: "P2-TABLE-SHIELD"
            },
            operation: {
              id: "op:act-shield-1",
              actionId: "act-shield-1",
              actorId: "P1",
              kind: "card_effect",
              status: "resolved",
              requiresStack: true,
              cardId: "BQ005",
              targetCardId: "P2-TABLE-SHIELD",
              label: "多重梦境迷宫"
            },
            event: {
              id: "evt:act-shield-1",
              actionId: "act-shield-1",
              operationId: "op:act-shield-1",
              kind: "shield_consumed",
              revisionNumber: 8,
              stackDepth: 0,
              markerType: "shield",
              markerAmount: 1,
              sourceCardId: "BQ005",
              targetCardId: "P2-TABLE-SHIELD",
              targetPlayerId: "P2",
              appliedAmount: 1
            },
            revision: {
              number: 8,
              actionId: "act-shield-1",
              operationId: "op:act-shield-1",
              eventId: "evt:act-shield-1"
            }
          }
        },
        {
          version: "0.1.0",
          kind: "view",
          messageId: "msg-state-patched-shield",
          name: "StatePatched",
          revision: 8,
          payload: {
            type: "StatePatched",
            audienceKind: "player",
            audienceId: "P1",
            revision: {
              number: 8
            },
            event: {
              id: "evt:act-shield-1",
              actionId: "act-shield-1",
              operationId: "op:act-shield-1",
              kind: "shield_consumed",
              revisionNumber: 8,
              stackDepth: 0
            },
            playerView: {
              gameId: "game-shield-view",
              viewerPlayerId: "P1",
              revision: {
                number: 8
              },
              match: {
                status: "active"
              },
              turn: {
                turnNumber: 2,
                activePlayerId: "P1",
                priorityPlayerId: "P1",
                priority: {
                  currentPlayerId: "P1",
                  passCount: 0,
                  windowKind: "action"
                },
                phase: {
                  name: "main",
                  step: "action",
                  allowsStack: true,
                  stepEnded: false
                }
              },
              score: {
                byPlayer: { P1: 0, P2: 0 },
                victoryThreshold: 10
              },
              board: {
                stack: [],
                resolved: [],
                randomResults: [],
                cards: [
                  {
                    cardId: "P2-TABLE-SHIELD",
                    name: "Shielded Target",
                    ownerId: "P2",
                    zone: "table",
                    visibility: "visible",
                    revealed: true,
                    faceDown: false,
                    exhausted: false,
                    destroyed: false,
                    keywords: [],
                    stats: {
                      combat: 1,
                      defense: 3,
                      influence: 0,
                      investigation: 1
                    },
                    counters: {
                      damage: 0,
                      influence: 0,
                      shield: 1
                    }
                  }
                ]
              }
            }
          }
        }
      ]
    };

    render(<DebuggerShell messageSets={[shieldMessageSet]} />);

    expect(screen.getByText(/shield_consumed/i)).toBeInTheDocument();
    expect(screen.getByText(/shd 1/i)).toBeInTheDocument();
  });
});
