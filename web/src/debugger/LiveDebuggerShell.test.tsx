import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { LiveDebuggerShell } from "./LiveDebuggerShell";
import { defaultMockMessageSets } from "./mockProtocol";
import type { DebuggerProtocolEnvelope } from "./protocol";

// Purpose: Verifies the live debugger can bootstrap from HTTP, submit actions, and fall back to mock data when the sandbox server is unavailable.

const initialLiveMessages = defaultMockMessageSets[0].messages.filter(
  (message) => message.name === "StatePatched"
);

const playableLiveMessages: DebuggerProtocolEnvelope[] = initialLiveMessages.map((message) => {
  if (message.name !== "StatePatched") {
    return message;
  }

  if (message.payload.playerView) {
    return {
      ...message,
      payload: {
        ...message.payload,
        playerView: {
          ...message.payload.playerView,
          match: {
            ...message.payload.playerView.match,
            status: "active",
            endReason: undefined,
            winnerPlayerId: undefined,
            finishedAtRevision: undefined
          },
          score: {
            ...message.payload.playerView.score,
            winnerPlayerId: undefined
          }
        }
      }
    };
  }

  if (message.payload.spectatorView) {
    return {
      ...message,
      payload: {
        ...message.payload,
        spectatorView: {
          ...message.payload.spectatorView,
          match: {
            ...message.payload.spectatorView.match,
            status: "active",
            endReason: undefined,
            winnerPlayerId: undefined,
            finishedAtRevision: undefined
          },
          score: {
            ...message.payload.spectatorView.score,
            winnerPlayerId: undefined
          }
        }
      }
    };
  }

  return message;
});

const rejectionBatch: DebuggerProtocolEnvelope[] = [
  {
    version: "0.1.0",
    kind: "event",
    messageId: "msg-live-rejected-1",
    name: "ActionRejected",
    payload: {
      type: "ActionRejected",
      action: {
        id: "act-live-rejected-1",
        actorId: "P1",
        kind: "pass_priority"
      },
      legality: {
        ok: false,
        reasonCode: "LEGALITY_FAILED_NOT_YOUR_PRIORITY",
        messageKey: "rules.legality.not_your_priority",
        hook: "turn.priority",
        context: {
          actorId: "P1",
          priorityPlayerId: "P2"
        }
      }
    }
  }
];

describe("LiveDebuggerShell", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("loads live protocol messages from HTTP on mount", async () => {
    const fetchMock = vi.fn().mockResolvedValue(createJSONResponse(initialLiveMessages));
    vi.stubGlobal("fetch", fetchMock);

    render(<LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />);

    await screen.findByText(/Source:\s*Live Sandbox/);
    expect(screen.getByText("7")).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledWith("/api/debugger/messages", undefined);
  });

  it("submits preset actions and appends returned protocol messages", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(createJSONResponse(playableLiveMessages))
      .mockResolvedValueOnce(createJSONResponse(rejectionBatch));
    vi.stubGlobal("fetch", fetchMock);

    render(<LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />);

    await screen.findByText(/Source:\s*Live Sandbox/);

    fireEvent.click(screen.getByRole("button", { name: "Pass Priority" }));

    await screen.findByText("LEGALITY_FAILED_NOT_YOUR_PRIORITY");

    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(fetchMock.mock.calls[1]?.[0]).toBe("/api/debugger/actions");
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      }
    });

    const body = JSON.parse(String(fetchMock.mock.calls[1]?.[1]?.body));
    expect(body).toMatchObject({
      actorId: "P1",
      kind: "pass_priority"
    });
  });

  it("disables action controls when the live patch already has a winner", async () => {
    const fetchMock = vi.fn().mockResolvedValue(createJSONResponse(initialLiveMessages));
    vi.stubGlobal("fetch", fetchMock);

    render(<LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />);

    await screen.findByText(/Source:\s*Live Sandbox/);
    expect(screen.getByText("Game over. Winner: P1")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Pass Priority" })).toBeDisabled();
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("resets the sandbox and replaces the feed with a fresh bootstrap state", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(createJSONResponse(initialLiveMessages))
      .mockResolvedValueOnce(createJSONResponse(playableLiveMessages));
    vi.stubGlobal("fetch", fetchMock);

    render(<LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />);

    await screen.findByText(/Source:\s*Live Sandbox/);
    fireEvent.click(screen.getByRole("button", { name: "Reset Sandbox" }));

    await waitFor(() => {
      expect(screen.queryByText("Game over. Winner: P1")).not.toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Pass Priority" })).not.toBeDisabled();
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(fetchMock.mock.calls[1]?.[0]).toBe("/api/debugger/reset");
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({
      method: "POST"
    });
  });

  it("allows action submission again after resetting from a game-over patch", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(createJSONResponse(initialLiveMessages))
      .mockResolvedValueOnce(createJSONResponse(playableLiveMessages))
      .mockResolvedValueOnce(createJSONResponse(rejectionBatch));
    vi.stubGlobal("fetch", fetchMock);

    render(<LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />);

    await screen.findByText(/Source:\s*Live Sandbox/);
    expect(screen.getByRole("button", { name: "Pass Priority" })).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: "Reset Sandbox" }));
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Pass Priority" })).not.toBeDisabled();
    });

    fireEvent.click(screen.getByRole("button", { name: "Pass Priority" }));
    await screen.findByText("LEGALITY_FAILED_NOT_YOUR_PRIORITY");

    expect(fetchMock).toHaveBeenCalledTimes(3);
    expect(fetchMock.mock.calls[2]?.[0]).toBe("/api/debugger/actions");
  });

  it("submits marker and face-down presets with expected action payloads", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(createJSONResponse(playableLiveMessages))
      .mockResolvedValueOnce(createJSONResponse(rejectionBatch))
      .mockResolvedValueOnce(createJSONResponse(rejectionBatch))
      .mockResolvedValueOnce(createJSONResponse(rejectionBatch));
    vi.stubGlobal("fetch", fetchMock);

    render(<LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />);

    await screen.findByText(/Source:\s*Live Sandbox/);

    fireEvent.click(screen.getByRole("button", { name: "Set Secret Marker" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(2);
    });

    fireEvent.click(screen.getByRole("button", { name: "Remove Secret Marker" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(3);
    });

    fireEvent.click(screen.getByRole("button", { name: "Set Own Table Face-Down" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(4);
    });

    const setMarkerBody = JSON.parse(String(fetchMock.mock.calls[1]?.[1]?.body));
    expect(setMarkerBody).toMatchObject({
      id: "act-web-p1-1",
      actorId: "P1",
      kind: "set_marker",
      targetPlayerId: "P1",
      markerType: "secret_society",
      markerAmount: 1
    });

    const removeMarkerBody = JSON.parse(String(fetchMock.mock.calls[2]?.[1]?.body));
    expect(removeMarkerBody).toMatchObject({
      id: "act-web-p1-2",
      actorId: "P1",
      kind: "remove_marker",
      targetPlayerId: "P1",
      markerType: "secret_society",
      markerAmount: 1
    });

    const setFaceDownBody = JSON.parse(String(fetchMock.mock.calls[3]?.[1]?.body));
    expect(setFaceDownBody).toMatchObject({
      id: "act-web-p1-3",
      actorId: "P1",
      kind: "set_face_down",
      cardId: "P1-TABLE-1"
    });
  });

  it("falls back to mock protocol data when the live server is unavailable", async () => {
    const fetchMock = vi.fn().mockRejectedValue(new Error("connect ECONNREFUSED"));
    vi.stubGlobal("fetch", fetchMock);

    render(<LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />);

    await screen.findByText("Live server unavailable. Showing mock protocol data.");
    expect(screen.getByText(/Source:\s*Mock Fallback/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Pass Priority" })).toBeDisabled();
  });

  it("shows custom action JSON validation errors without calling action API", async () => {
    const fetchMock = vi.fn().mockResolvedValue(createJSONResponse(playableLiveMessages));
    vi.stubGlobal("fetch", fetchMock);

    render(<LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />);
    await screen.findByText(/Source:\s*Live Sandbox/);

    fireEvent.change(screen.getByLabelText("Custom Action JSON"), {
      target: { value: "{ bad-json" }
    });
    fireEvent.click(screen.getByRole("button", { name: "Submit Custom Action" }));

    expect(await screen.findByText("Custom action must be valid JSON.")).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("submits custom JSON action payloads with generated id/actor defaults", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(createJSONResponse(playableLiveMessages))
      .mockResolvedValueOnce(createJSONResponse(rejectionBatch));
    vi.stubGlobal("fetch", fetchMock);

    render(<LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />);
    await screen.findByText(/Source:\s*Live Sandbox/);

    fireEvent.change(screen.getByLabelText("Custom Action JSON"), {
      target: { value: JSON.stringify({ kind: "pass_priority" }) }
    });
    fireEvent.click(screen.getByRole("button", { name: "Submit Custom Action" }));

    await screen.findByText("LEGALITY_FAILED_NOT_YOUR_PRIORITY");
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(fetchMock.mock.calls[1]?.[0]).toBe("/api/debugger/actions");
    const body = JSON.parse(String(fetchMock.mock.calls[1]?.[1]?.body));
    expect(body).toMatchObject({
      id: "act-web-p1-1",
      actorId: "P1",
      kind: "pass_priority"
    });
  });

  it("disables custom action submit for spectator view", async () => {
    const fetchMock = vi.fn().mockResolvedValue(createJSONResponse(playableLiveMessages));
    vi.stubGlobal("fetch", fetchMock);

    render(<LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />);
    await screen.findByText(/Source:\s*Live Sandbox/);

    fireEvent.click(screen.getByRole("button", { name: "View as spectator" }));
    expect(screen.getByRole("button", { name: "Submit Custom Action" })).toBeDisabled();
    expect(screen.getByText("Spectator cannot submit actions.")).toBeInTheDocument();
  });

  it("disables custom action submit when game is over and re-enables after reset", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(createJSONResponse(initialLiveMessages))
      .mockResolvedValueOnce(createJSONResponse(playableLiveMessages));
    vi.stubGlobal("fetch", fetchMock);

    render(<LiveDebuggerShell fallbackMessageSets={defaultMockMessageSets} />);
    await screen.findByText(/Source:\s*Live Sandbox/);

    const customSubmitButton = screen.getByRole("button", { name: "Submit Custom Action" });
    expect(customSubmitButton).toBeDisabled();
    expect(screen.getByText("Game over. Winner: P1")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Reset Sandbox" }));
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Submit Custom Action" })).not.toBeDisabled();
    });
  });
});

function createJSONResponse(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: {
      "Content-Type": "application/json"
    }
  });
}
