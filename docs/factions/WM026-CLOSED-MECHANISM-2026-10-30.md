# WM026 closed implementation

WM026 (blue) is now one complete closed implementation; ZHWM026 is its reverse, not another completed card. Independent review count remains zero; both forms stay unsupported in the public catalog, whose admission count remains236. No browser play, push, merge, release or refused-review retry occurred.

Original images and hashes are pinned in `WM026-SOURCE-CHECK-2026-10-29.md`. Corrected transcription: enemy hand at most three; preparation text does not say mandatory. Death interception and response use the existing effect frames and serializable pending source; same-instance transformation preserves damage/markers and attachments. Lethal retained damage settles again without repeating the replacement. Hidden death, source departure, actor elimination, restart and corrupt Game/Room restoration are covered. Preparation return uses the existing optional preparation ordering and frozen enemy-hand observation.

Project execution choices, not official rulings: death replacement mandatory/respondable; preparation optional under existing machinery; mist has absent printed-cost display with zero numeric comparison, is not independently constructible/playable, and restores physical printed identity on departure. No other token/card rules were generalized from this choice.

## Exact validation scope

Frozen engine, Native tests and Worker test source: `4f6aba43f6ee19ae6f2cb20a6f0a74139bdddba2`. Final code checkpoint `1aeee68247c788de7ce4aaa5cb7d5482e724e8df` differs only by the UI directory expectation316→318. This closure commit adds documentation only.

- Native final13:11 discovered/pass, zero fail/ignored. Related13:57 discovered/pass, zero fail/ignored across JC035, WM036, death continuation and BQ044.68 distinct tests; not a full library run.
- Default WASM build02 and web production build passed. WASM parity:898 actual commands,3592 views,898 duplicates.
- UI02:8 files,165 passed, zero failed/skipped. UI01's164 pass/1 failure is preserved: stale directory size316 versus318. Public236, negative admission checks and old artwork digests remain unchanged.
- Worker01:2 passed,0 failed/cancelled/skipped;898 commands,3592 views,898 duplicates,898 cold reopens,2 original receipt replays after completion. Runtime1882.088seconds; child max RSS1618752KiB; sampled cgroup range12718022656–15259406336bytes. This includes environment baseline, not just test memory.

Natural case starts from actual validated private-candidate50-card construction and executes all888 commands through Native/WASM and Worker. The setup is a Native initializer, not public lobby admission. Original raw audit confirms seat0 plays actual WM036 at command index733; board instance i324 transforms at745 and returns at887. A separate prepared sacrifice/response case executes10 commands; it is not natural setup. Neither case is browser play.

Source manifest SHA256 `28a64add980b18970f6af8db85114bc3044a3587ccef2cdbe47a3dadc007b758`; WASM SHA256 `3e79f64b42017e71844977b60d7746b92b9b4922cc9cf3c54c757d1cabd6015c`; Cargo.lock SHA256 `1745cc1af931f500e763222c39100da3ac0521f4a1f96fc54e82c8589e2edb54`. Rust/Cargo1.90.0, Node24.19.0, wasm-bindgen0.2.104; jobs1/test threads1, locked offline Native, default features. Exact commands, logs and resource measurements are in local `results.json`.

Evidence: `/workspace/wm026-validation/delivery/`; final recovery bundle and archive recorded in the delivery summary. Original red/failed WIP tests, natural08–11 failures, UI01 failure and superseded builds are retained. A supplemental audit script initially traversed historical snapshots as physical cards; scoping it to board cards correctly verified the actual two transformations without changing or rerunning the game.

No full-library sweep or independent review is claimed. Library401 and push/review restrictions remain; evidence is local only, no invented Library ID. WM026 supersedes its earlier incomplete inventory row; BQ049's neutral task-region dependency is unchanged. This is not a qualified public20-card batch.
