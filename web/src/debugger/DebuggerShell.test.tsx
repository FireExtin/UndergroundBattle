import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { DebuggerShell } from "./DebuggerShell";
import { defaultMockMessageSets } from "./mockProtocol";

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
});
