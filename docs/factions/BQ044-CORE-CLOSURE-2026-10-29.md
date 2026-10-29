# BQ044 core and natural entry — verification checkpoint

Base `64c3745c8a4c9c05754f01d3b0063282dfa4125e`; ordinary-trigger WIP history remains in `BQ044-WIP-2026-10-29.md`.

Source image and gameplay fields remain pinned to SHA256 `27f37580799716e19a0de27b529040a7a524c024eceabe17bb4ee15a5b60c791`. The printed core text is “牺牲一个角色，抓一张牌，遏制1。” Both field and core-zone invocation use the original paid BQ048/BQ063 frame; there is no free direct core action. Personal controlled, face-up field characters are eligible sacrifices, including actual generated characters. Existing generated disappearance rules apply without changing token costs or skills.

The state carries the original core, sacrificed character and command cursor. XG25/attached-blood pregrave destinations reuse DeathCaller; ordinary/core-return destinations use a typed continuation in the existing effect queue. Original BQ063 invokes twice then performs its own final repression. Restore rejects wrong core, parent, options and duplicate frame carriers. Empty-deck elimination stops the continuation, and elimination discards its queued resume.

Project choices (not claimed as official rulings): retain the earlier single enemy player choice without a printed-target contract; a core invocation without a legal sacrifice ends without granting the subsequent draw/repression. No broader change to other cards or core-region targeting is made.

Catalog keeps the image-confirmed 修格斯 subtitle and current internal implementation identity. Public `supported:false` and the 236-card admission set remain unchanged. Client selection now recognizes BQ044/BQ045/BQ046 but still requires supported admission. Original JPG, public copy and both scan indexes are aligned; prior image/index digests remain asserted.

Native `bq044-core-10`: 18 discovered, 18 passed, 0 failed/ignored. This includes the earlier 9 ordinary-trigger tests; do not add them again. Fresh export: natural validated 50+1 input and 470 actual Room commands; three explicit prepared core scenarios contain 12/13/16 commands. The separate ordinary JZ54 trigger exports 13 commands. Each step records actual four-seat views, state recovery and duplicate transitions. Native construction uses a private candidate registry, not public admission or a browser.

Failure history remains outside source under `/workspace/bq084-validation/logs/bq044-core-*.log`: enum insertion compile error; fixture ordering/loyalty/secrecy assumptions; SocietyDefinition access compile error; missing retained subtitle; natural asset preparation incorrectly counted non-asset BQ050; diagnostic move error; and natural forecast omitted top/bottom allocation. The natural input now includes existing JC049 for a genuine red asset, preserves 12 commands and exactly 50+1 cards, and assigns every forecast card to a legal top/bottom position. No production constraint was loosened. Full failed 2200-command traces are retained locally; they are not successful coverage.

At this checkpoint, related Native regression and default WASM build are running. UI/Worker verification is pending. Whole-card validation remains pending until those results are recorded; independent review remains blocked and public admission closed. No push, upload retry, main merge or deployment.

## Final applicable closure

Rust and default WASM source: `27d05f6ff8d3fdb277a3fb06b2a9df1cf3064250`; subsequent changes are UI/Worker test instrumentation and this result record. Native BQ044 18/18 plus disjoint affected regression 72/72 = 90 unique tests, zero failed/ignored; not a whole-library run. The prior ordinary 9 and adjacent 48 are not added again.

Fresh Native exports match WASM for 5 cases / 524 commands / 2096 views / 524 duplicate transitions. Actual local HTTP Worker/D1 tests passed 5/5, executing all 524 commands, 524 cold reopens, 2096 player views and duplicate persistence checks, plus each scenario's first receipt after completion. The natural private-registry 50+1 case includes all 470 preparation/execution commands; the other four cases use explicit Native layouts (54 commands), not natural preparation. No tail-only replay.

UI affected selection passed 8 files / 164 tests after retaining the initial 162-pass/1-fail selector log. Direct no-destination actions use the actual direct-action button; destination actions retain their group/region selection. Production UI constraints were not relaxed. Ordinary trigger/enemy/victim refreshed selections were also verified. Default WASM and production web builds passed.

BQ044 now counts as one complete closed implementation, independent review zero and public admission still closed. No whole-library claim, real browser play, independent approval, push or release. Local evidence is `/workspace/bq044-validation/core-delivery`; original failed logs and full failed natural traces remain preserved. Library HTTP 401 remains blocked and was not retried. Native/WASM source manifest SHA256: `cff717a31a34a1918b0f2f4327e9f957c1cd4dfae80cec2bbc184c39aeae95ed`.
