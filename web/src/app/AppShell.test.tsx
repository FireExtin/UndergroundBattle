import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// Exercise the lazy import itself: API fallback cannot handle a chunk rejection.
describe("AppShell optional debugger loading", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.doMock("../game/GameApp", () => ({ GameApp: () => <h1>游戏入口</h1> }));
  });

  afterEach(() => {
    window.history.replaceState(null, "", "/");
    vi.doUnmock("../game/GameApp");
    vi.doUnmock("../debugger/LegacyDebuggerEntry");
    vi.restoreAllMocks();
  });

  it("renders the normal game without invoking the debugger import", async () => {
    const importDebugger = vi.fn(() => { throw new Error("Must stay unloaded"); });
    vi.doMock("../debugger/LegacyDebuggerEntry", importDebugger);
    window.history.replaceState(null, "", "/");
    const { AppShell } = await import("./AppShell");
    const view = render(<AppShell />);
    view.rerender(<AppShell />);
    expect(screen.getByRole("heading", { name: "游戏入口" })).toBeVisible();
    expect(importDebugger).not.toHaveBeenCalled();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("shows loading state, then the successfully imported debugger", async () => {
    let resolveImport!: (module: { default: () => ReactNode }) => void;
    const module = new Promise<{ default: () => ReactNode }>(resolve => { resolveImport = resolve; });
    vi.doMock("../debugger/LegacyDebuggerEntry", () => module);
    window.history.replaceState(null, "", "/legacy-debugger");
    const { AppShell } = await import("./AppShell");
    render(<AppShell />);
    expect(screen.getByRole("status")).toHaveTextContent("正在打开调试器");
    resolveImport({ default: () => <h1>已加载调试器</h1> });
    expect(await screen.findByRole("heading", { name: "已加载调试器" })).toBeVisible();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("contains an import rejection and offers a full-document reload instead of a cached rerender", async () => {
    // React reports caught boundary errors to the console in this test mode.
    vi.spyOn(console, "error").mockImplementation(() => {});
    const importDebugger = vi.fn(() => { throw new Error("Chunk request failed"); });
    vi.doMock("../debugger/LegacyDebuggerEntry", importDebugger);
    window.history.replaceState(null, "", "/legacy-debugger");
    const { AppShell } = await import("./AppShell");
    const view = render(<AppShell />);
    expect(await screen.findByRole("alert")).toHaveTextContent("调试器加载失败");
    expect(screen.getByText("请检查网络连接，然后重新加载调试器。")).toBeVisible();
    expect(screen.getByRole("link", { name: "重新加载调试器" })).toHaveAttribute("href", "/legacy-debugger");
    expect(screen.getByRole("link", { name: "返回游戏" })).toHaveAttribute("href", "/");
    view.rerender(<AppShell />);
    expect(importDebugger).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("alert")).toBeVisible();
  });
});
