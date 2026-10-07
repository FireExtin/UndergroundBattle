import { lazy, Suspense } from "react";
import { GameApp } from "../game/GameApp";

const LegacyDebuggerEntry = lazy(() => import("../debugger/LegacyDebuggerEntry"));

// The cloud game is the default experience; the Go debugger remains explicitly accessible.
export function AppShell() {
  return window.location.pathname.startsWith('/legacy-debugger')
    ? <Suspense fallback={<p>正在打开调试器…</p>}><LegacyDebuggerEntry /></Suspense>
    : <GameApp />;
}
