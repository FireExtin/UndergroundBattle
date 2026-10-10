# Card-face deck builder UI candidate

Base: `454e46f67f4a55814321b66817b704d74fbad2bc`, on `codex/jz48-combined-review-20261007`, in a normal cloud clone. This is the UI task's candidate for independent review. Main and Site37 have not been published or merged by this task.

The lobby's deck builder now occupies the full row. The desktop catalog displays a scrollable original-card grid (five columns at 1440px), with the editable deck and sticky count/save controls on the right. The expandable workspace retains edits and filters when closed. A card or selected-deck thumbnail opens a large, uncropped original beside public catalog metadata and rule text. Missing registrations and failed image loads show an explicit placeholder. The original images and their registry are reused unchanged.

Search, faction, card kind and implementation filters remain; printed cost, only-selected and reset controls are added. Grid, large preview and selected-deck counters use the same draft and functional relative updates, including repeated/batched clicks. Existing `validateDeckDraft`, catalog admission and copy-limit metadata supply all legality feedback. Invalid drafts can still be saved for correction; selection remains blocked until the existing validator passes. Same-name exceptions and limits, minimum/capacity, version provenance, society constraints, local storage failures and selected public snapshots keep their existing behavior. No new rule or playable card is introduced.

The grid is two columns at 390px. The preview traps focus, supports Escape/button/backdrop close, and restores its opener. If removing the last selected copy filters out the opener, focus returns to search. Nested preview close returns to the expanded workspace; closing the workspace restores body scrolling and focuses its opening control.

## Actual validation

- TypeScript app/node checks and production Vite build passed.
- Five focused files: **40/40 tests**, including eight new grid/preview regression tests.
- Whole Web suite: **613/613 tests in 69 files**, using the freshly compiled current WASM in the ignored `pkg` directory. No generated artifacts are part of this UI change.
- No standalone lint script exists. `git diff --check` and Python compilation passed.
- Chromium: **9 checkpoints**, **6 candidate screenshots**, no JavaScript errors or room/API writes. Coverage includes multiple rows, repeated increment/decrement, preview and selected-deck linkage, combined filters and zero cost, invalid save/reload, valid selection, workspace close/reopen, 390px layout, removed-opener focus, and a deliberately simulated image 404.

The actual existing Site37 was visited in the cloud browser: its deck library had zero images; three repeated additions and local save/reload passed. Its public engine45/pool42 catalog recording has 103 ordinary entries, including ten regions, and eight separate societies. All seven public kernel catalog fields match a fresh current-source WASM catalog; the hosted service adds `transport` and `entryIdempotency`. Candidate browser runs use the real local Vite application with only `/api/catalog` fulfilled from that public recording. They establish frontend behavior and local persistence, not live server admission or a deployed candidate. The baseline browser accepted the environment proxy certificate via `ignore_https_errors`; no owner credential was used.

An execution transport interruption and a stopped preview server occurred during setup; final recorded runs completed successfully. Screenshot inspection found and fixed missing color variables for the portal's primary action before the final runs.

## Review evidence and reproduction

[Review packet](evidence/deck-builder-ui-2026-10-07/review-packet.json) binds the seven product/test file hashes, commands/results and all evidence hashes. [Browser results](evidence/deck-builder-ui-2026-10-07/browser-results.json) bind all six screenshot hashes and checkpoints. [Live baseline](evidence/deck-builder-ui-2026-10-07/site37-baseline-results.json) records the separate deployed-site scope.

Visual evidence: [desktop grid](evidence/deck-builder-ui-2026-10-07/desktop-grid.png), [large original](evidence/deck-builder-ui-2026-10-07/desktop-original-preview.png), [copy-limit feedback](evidence/deck-builder-ui-2026-10-07/desktop-copy-limit-feedback.png), [mobile grid](evidence/deck-builder-ui-2026-10-07/mobile-grid.png), [mobile original](evidence/deck-builder-ui-2026-10-07/mobile-original-preview.png), and [simulated missing image](evidence/deck-builder-ui-2026-10-07/desktop-image-failure-placeholder.png).

From the repository, start `npm run dev -- --host 0.0.0.0` in `web`, then run:

```sh
python tools/cloud-playtest/deck_builder.py \
  --catalog docs/evidence/deck-builder-ui-2026-10-07/site37-catalog.json \
  --output /tmp/deck-builder-ui-review
```

The driver requires the environment's existing Python Playwright and Chromium. For the complete Web suite, first build the current ignored WASM with the repository's existing `rust-game-wasm/build.sh`. Publication and independent acceptance remain with the parent task.
