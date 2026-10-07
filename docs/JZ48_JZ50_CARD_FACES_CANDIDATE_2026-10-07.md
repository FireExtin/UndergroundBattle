# JZ48 / JZ50 original card-face candidate

This isolated artwork branch starts at published/reviewed `2305aa17c53e73dd3683ae9fe1e654531777f16e`. It adds two pinned originals to the existing reader and compact-art mapping. The previous 109 manifest entries and original image files remain byte-for-byte identical to that commit; the candidate contains 111 originals. JZ49 remains present.

Only the two JPEG files, scan manifest, scan URL mapping, focused tests and this review record change. The JZ55 reader test's expected scan count changes from 109 to 111. Rust, WASM source, card specifications, gameplay catalog, identity, reader authorization and game admission are unchanged. The actual unchanged engine43/pool40 catalog still has 101 ordinary cards and eight societies, including JZ49 and excluding JZ48/JZ50. Artwork availability does not select or admit a card. JZ48's separate mechanism candidate and JZ50's unimplemented mechanism are outside this branch.

## Canonical original review

Both complete originals were actually viewed in the UndergroundBattle cloud workspace and compared field-by-field with `docs/factions/card-specifications.json`; there were no gameplay-field discrepancies. They are ordinary characters with white name ink, no subtitle, no printed keyword, collector codes 48/76 and 50/76, and the observed Intermezzo series symbol. No release year is inferred.

| Card | Printed fields and text | Original identity |
| --- | --- | --- |
| JZ48 街头劫匪 | Cost 1; black loyalty 1; human / criminal; no domain; permanent investigation 0 / combat 1 / influence 0; no temporary icons; defense 1. 持续：若本地区有其他本方罪犯角色，则街头劫匪获得永久势力1和防御力+1。 | `resource/ymsj-fun.github.io/cards/JZ48 街头劫匪.jpg`; 400 × 560; 499,021 bytes; SHA256 `a9a5da75938c22872cca524776356b10f2a0250370d3ba3ed3b31deb332e5b4f` |
| JZ50 墓穴食尸鬼 | Cost 2; black loyalty 1; undead / ghoul; death domain 1; permanent investigation 0 / combat 1 / influence 0; no temporary icons; defense 1. 现身触发：从你的牌库中寻找一张死亡领域的角色牌，将该牌置于你的墓地，然后洗牌。 | `resource/ymsj-fun.github.io/cards/JZ50 墓穴食尸鬼.jpg`; 400 × 560; 533,019 bytes; SHA256 `403e44bb5f7f0ff8715acb267a8369c3715b2dc752d7fe612de085da29b523e7` |

The served JPEG bytes are identical to these canonical sources, without cropping, recompression or editing. Original URLs are public artwork resources, not authenticated card-state resources.

## Verification

- Six focused UI files passed: **68 tests**, including 20 new tests for the two originals, hidden compact presentation for all four seats, controller-authorized hidden reading when owner differs, printed fields, asset presentation, exact source hashes and catalog-driven selection. Existing JZ49 artwork, JZ49 slow-entry, JZ55, ArchivePresentation and ReadModal tests passed.
- Production TypeScript checks and Vite build passed. An initial build invocation from the repository root had no package.json; the corrected `web` invocation and the final verification passed. Its log is retained as a setup error, not a product failure.
- Read-only actual ABI inspection confirmed engine `rust-v0.2.43-jz49-slow-mill-candidate`, pool `limited-v2.40-jz49-slow-mill-candidate`. The unchanged current WASM is 2,186,939 bytes, SHA256 `3b0bee7b1c6788bc0a299338322f1c07d6108161a75e4999ac066971659520bb`.
- Before browser QA, the current Sites skill and its portable local-preview reference were read. The managed-container flag was unset. The owned Vite local preview returned HTTP 200 and was stopped after QA.
- Actual cloud `/usr/bin/chromium` used **one browser, one context and one page**, sequentially changing desktop 1440 × 1000 and mobile 390 × 844 viewports. Production Table and ReadModal rendered a local fixture containing the pinned printed fields. The fixture augments display definitions only, makes no API calls and throws on game actions. It is not a natural match or a mechanism qualification.
- Both cards' compact view, text reader and full-original reader were captured on both viewports: 12 screenshots. Desktop compact art is visible; mobile compact art is hidden by the existing layout. Full originals decode as 400 × 560, use `object-fit: contain`, and fit within both viewports. All screenshots were actually inspected.
- Additional browser checks cover both hidden cards for all four viewer seats with owner p1 and controller p0. Compact identity/art stays hidden for all seats; only p0 may open the hidden printed reader/original. Owner p1 and noncontrollers p2/p3 cannot. This tests client presentation with supplied fixture fields; it does not replace server projection tests. Zero API requests and zero page errors were observed.

## Local review evidence

Evidence is retained in `/tmp/jz48-jz50-card-faces-evidence`: `source-and-preservation.json` lists all 109 prior SHA256 values and byte-preservation results; `current-catalog-summary.json` records the unchanged actual ABI; `focused-web-final.log` and `build-final.log` record final checks; `browser-results.json`, `card-faces-browser.mjs` and the `card-faces-qa.*` fixture files record this round's browser scope. Screenshots are `{desktop,mobile}-{JZ48,JZ50}-{hand,text,original}.png`.

The temporary fixture entrypoints and dependency link are removed from the candidate worktree. Review evidence contains no live room database, credentials, seat session or runtime state. No GitHub push, main merge, Site publication, Library retry or alternate artifact upload is performed by this branch.
