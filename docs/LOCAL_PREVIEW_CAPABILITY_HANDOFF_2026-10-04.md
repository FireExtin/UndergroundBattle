# Local preview capability handoff — 2026-10-04

The production candidate remains `c4d20b599be77a3e09a4bd8dc3e5e0bd272e0620`. The subsequent `5bd2178cc5105e112a4032381875f6262d5c0472` adds only the LC01 original-icon recheck test and its evidence document: two files, 39 added lines. Its standalone binary diff is 3,816 bytes. No complete regression was repeated for this handoff.

## Available files and configuration

- Worktree: `/workspace/UndergroundBattle-jc008-candidate`.
- Existing built entry: `web/dist/index.html`; the Rust router serves `WEB_DIST`, defaulting to `web/dist`.
- Existing Vite development configuration proxies `/api` to `http://127.0.0.1:8090` and `/api/debugger` to `http://127.0.0.1:8080`. Installed Vite defaults to development port 5173. These are configured local addresses, not running or cloud-reachable preview URLs.
- `rust-game/src/main.rs` defaults `PORT` to 8090 but binds to `0.0.0.0:{port}`. It was not launched because this task forbids adding public ports or network permissions.

## Two separate seat contexts

The existing `web/src/game/GameApp.tsx` exposes “开始新的独立玩家会话”. `web/src/game/playerStorage.ts` uses a random sessionStorage namespace for independent sessions and localStorage for the ordinary session. Two separate browser profiles can therefore hold separate seat credentials. Two tabs must each explicitly start a new independent session before entering different seats; simply duplicating a tab can inherit the same sessionStorage context. Refresh preserves that tab's independent session.

This is a source-level access-path handoff. No browser profiles, seats, rooms, or gameplay were created or operated in this task; two live browser contexts have not been exercised here.

## Preview blocker

The current tool inventory exposes no browser, preview, port-forwarding, or tunnel action. `SITES_MANAGED_LINUX_CONTAINER` is unset and no `.sites-runtime/execution-profile.json` exists in this candidate or the existing Site checkout, so no managed Sites preview path is established. The Sites portable-preview guidance requires a supported access/forwarding path rather than assuming a remote browser can reach loopback.

Accordingly, there is no supported cloud-browser preview URL to hand off in this environment. No server was started, no extra permissions requested, and no GitHub push, Sites save, or Sites deployment attempted. A parent-selected environment with a supported preview/browser access path is required for the independent two-seat experience.
