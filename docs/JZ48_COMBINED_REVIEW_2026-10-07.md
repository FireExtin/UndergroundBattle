# JZ48 mechanism and JZ48/JZ50 original card-face review

JZ48 街头劫匪 now gains exactly one permanent influence and one defense while another face-up criminal character in its actual region has the same current controller. Two distinct JZ48 instances with that controller can support each other; losing the supporting character immediately removes the bonus. This candidate preserves JZ49 and adds the original JZ48/JZ50 artwork. JZ50 remains artwork-only.

The reviewed product is **`c98bf422795c48d5842011dd5b65a97e647d3f14`**, based on published/reviewed `2305aa17c53e73dd3683ae9fe1e654531777f16e`. The delivery commit adds only this review record and sanitized evidence. Its Rust, WASM and Sites trees and all existing Web files remain identical to the reviewed product. Site35 is still current; publication and a main merge await the parent's decision.

## Identity and source

| Binding | Exact value |
| --- | --- |
| Rules | `hegemony-pdf-v1` |
| Engine | `rust-v0.2.44-jz48-criminal-condition-candidate` |
| Pool | `limited-v2.41-jz48-criminal-condition-candidate` |
| Isolated mechanism | `02a69dc47142c4b48800f25af38ba3dae2b8e532` |
| Isolated artwork | `65ad3b97ec3bc7d3e6f567ad40dfda05d02cb190` |
| Rust tree | `b4f921ae4b1e1cb45266f5072667c532343cd001` |
| Entire WASM tree | `9cdff1379436567e1bae70d8c73f832f81a88793` |
| WASM source subtree | `188680abc5cd379e03ee2bd8ba85c22662b49335` |
| Sites tree | `54993f3790128a5952d1d38b2fdc7e73e6f766d3` |
| Web tree | `cf67a80cdb3d980ff35fafda1fa4c5fb6c83bffb` |

The complete [JZ48 original](<../resource/ymsj-fun.github.io/cards/JZ48 街头劫匪.jpg>) was actually viewed and matched to [the served original](../web/public/cards/JZ48.jpg): cost1, black loyalty1, human/criminal character, no domain, permanent combat1, no temporary icons, defense1, nonunique. Its SHA256 is `a9a5da75938c22872cca524776356b10f2a0250370d3ba3ed3b31deb332e5b4f` (499,021 bytes, 400×560).

The primary [霸权说明书](<../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf>) is SHA256 `a1e5bca72b8dbb374357feace28b7fbfd31136349ee23455d60f5f62e0a6b886`. The implementation and independent reviewer actually viewed printed P16/P21, corresponding to physical PDF pages17/22: 本方 means the current controller alone; 友方 includes teammates. Source files were compared with the exact Git blobs at the reviewed product, not only by filename. [Complete card specifications](factions/card-specifications.json), records JZ48/JZ50, provide the secondary field audit.

The complete [JZ50 original](<../resource/ymsj-fun.github.io/cards/JZ50 墓穴食尸鬼.jpg>) also matches [its served original](../web/public/cards/JZ50.jpg), SHA256 `403e44bb5f7f0ff8715acb267a8369c3715b2dc752d7fe612de085da29b523e7` (533,019 bytes, 400×560). Its printed reveal/search-to-graveyard ability has no implementation or admission in this candidate. Artwork URLs do not populate the gameplay catalog.

## Bounded implementation and counts

One parameterless, JZ48-specific static modifier supplies both attributes through the same live condition. The entire JZ48 Definition is whitelisted. Self, hidden cards, assets, other regions, teammates and enemies cannot qualify; an exhausted supporting criminal retains its subtype. Several supporters still grant only one bonus. Current controller determines the condition, while existing owner rules determine graveyard destination. Source exhaustion preserves its attributes and existing participation rules. No new generic binding, identity, queue or death mechanism is introduced.

Existing death settlement freezes each simultaneous lethal set before removal and then processes newly lethal sets. Existing cleanup, control expiry, paid commands, source-instance resets and JZ49 behavior are retained. The [mechanism record](JZ48_CRIMINAL_CONDITION_CLOUD_CANDIDATE_2026-10-07.md) and [artwork record](JZ48_JZ50_CARD_FACES_CANDIDATE_2026-10-07.md) retain their isolated-branch verification scope and earlier failures.

The archive has **684 face/component IDs**, excluding TK007's initiative marker. It includes societies, world regions, transformed views and generated tokens. Runtime card counts are listed separately:

| Count | Current Site35 / engine43 | Candidate engine44 |
| --- | ---: | ---: |
| Ordinary catalog entries, including10 regions | 101 | 102 |
| Non-region ordinary entries | 91 | 92 |
| Societies | 8 | 8 |
| Admitted archive IDs | 109 | 110 |
| Archive IDs awaiting admission | 575 | 574 |
| Registered original faces | 109 | 111 |
| Archive faces awaiting registration | 575 | 573 |

Of the573 unregistered faces,572 have recorded source images and **TK011 remains missing**. JZ50 is registered artwork and remains outside admission. Historical research-index design/acceptance labels retain their recorded baseline. [Fresh counts](evidence/jz48-combined-2026-10-07/admission-art-counts.json) use the actual44/43 catalogs and current artwork registry; they do not relabel an old shared-mechanism matrix.

## Tests and independent review

- Final mechanism workspace: **545 native** (513 core +32 integration),0 failed/ignored. The exact Rust tree is reused in the combination. It includes the prior JZ49 tests and19 new JZ48 tests.
- Combined candidate: **557 Web tests in66 files**, **25 Worker/D1 tests**, TypeScript, Vite and Worker dry-run build passed. Artwork independently reran68 focused tests and TypeScript. Old109 artwork records and image bytes remain exact.
- Independent mechanism review reran **19 native**, **1416 four-seat WASM projections** (59 transitions,8 rejections,282 checkpoints,13-step paid response chain), and3 routing tests. All passed. Frozen43's separate JZ49 regression retained3616 projections. The historical199 files and five frozen43 files were independently byte-checked.
- Current-only routing rejects39 historical tuples and5 altered identities without dispatching them to the current interpreter. Production44 rejects historical43 state; frozen historical modules remain for audit, outside the production Worker.

New reviewers `/root/review_jz48_mechanism` and `/root/review_jz48_jz50_card_faces` did not implement either batch. Both fixed branches and the exact combination have **no P0–P3 findings**. Their independent reruns are distinguished from implementation/root full-suite log inspection. See [mechanism report](evidence/jz48-combined-2026-10-07/independent-mechanism-review.md), [artwork report](evidence/jz48-combined-2026-10-07/independent-art-review.md) and [combined findings](evidence/jz48-combined-2026-10-07/independent-summary.json). Verbatim reports retain local execution paths and intermediate-stage observations; final dependency links were removed before the delivery commit.

The native/WASM fixtures cover controller and region matrices, owner≠controller, same-card independent instances,2v2 teammate exclusion, hidden/exhausted/asset cases, support and source movement/control changes, control expiry, cleanup ordering, real paid XQ16 hiding, damage/wounds, simultaneous and cascading deaths, exact influence/score10 thresholds, declaration/stack/save/replay/SQLite, duplicate commands and invalid Definition transplants. These fixture cases retain their explicit offline initial layouts.

## Actual cloud UI evidence

JZ48 focused natural UI used the production combination in cloud Chromium. The user-visible deck builder made JC125×44 / JC091×3 / JZ48×3, followed by ordinary create/join/ready/start and all gameplay commands. The test wrapper fixed only the normal create operation's8-byte seed input to4586; IDs and seat tokens used native entropy. No engine-state or D1 layout was injected.

The120 unique successful commands and121 contiguous journal events show v16's single JZ48 at influence0/defense1, v65's two different instances each at1/2, and actual paid JC091《谋杀》exhausting three assets and destroying the second instance. At v121 the original first instance is immediately0/1, with one JZ48 in its owner's graveyard. Two target cancellations leave state/commands/journal unchanged; SQLite reopening and two-seat reloads preserve the result. The reviewer independently matched UI steps, HTTP, persisted receipts and command journal, reran the full WASM journal byte-exactly and reran native44 read-only audit with unchanged DB/WAL hashes.

Desktop and390×844 mobile readers show the surviving current stats and the full400×560 original with `contain`. A subsequent read-only mobile check restored the same seat/table, scrolled to the surviving instance, and made zero POSTs with an unchanged database. The original20 artwork fixture tests and12 fixture screenshots remain presentation/privacy evidence. The actual JZ48 run finishes **playing, turn3** and covers the focused sequence above. Broader2v2/control/cascade cases are bound to the offline tests; a complete engine44 strategy terminal game is outside this evidence. [JZ48 UI summary](evidence/jz48-combined-2026-10-07/jz48-ui-summary.json) · [mobile check](evidence/jz48-combined-2026-10-07/mobile-readonly-summary.json).

Separately, the original engine43 four-seat natural room on reviewed2305 reached **finished at turn7, team0 score13 versus0, winScore10**. It has1063 accepted unique UI/AutoPass commands and1066 continuous journal versions. All1068 recorded successful command HTTP responses match persisted receipts, including5 duplicates;357 recorded409 responses (including the deliberate terminal stale request) have no successful persisted receipt. Native43 read-only replay matches; terminal duplicate/stale checks, SQLite reopening and all four seat reloads passed without additional writes. Desktop/mobile victory evidence was captured. [Terminal summary](evidence/jz48-combined-2026-10-07/engine43-terminal-summary.json).

That four-seat policy used watchers on team0 and responders cooperatively passing on team1. Browser/driver restarts are documented at versions8 and25: opening mulligans first resolved a driver locator timeout; after a renderer crash, the original pending command was reconstructed exactly from four captured HTTP receipts and replayed through the actual UI. The same naturally created room and original entry receipts were retained. All HTTP records were persisted from the original create; zero HTTP records were reconstructed. This qualification covers engine43's cooperative end-to-end flow and does not supply an adversarial balance or engine44 terminal result.

## Build and artifact bindings

The actual current-only Worker has **one WASM**,2,191,175 bytes, SHA256 `6cf6dca6a06844c84e38fbed2a084145bc0e63ae3897de18b864814c314d11f9`. Worker JS is47,273 bytes, SHA256 `0601398bfef9651e7cf815aee1978684b58f68be177f6275c848ff763a5006c4`; total **2,238,448 bytes**. Wrangler dry-run reported2185.98KiB upload /570.04KiB gzip. Build source trees, current pkg/generated/Worker bytes and the executed test logs are bound in [bindings.json](evidence/jz48-combined-2026-10-07/bindings.json), with Git blob IDs and SHA256 for code, tests and primary sources.

The verified local product Git bundle `c98bf42-jz48-combined-product.bundle` advertises exact productc98bf42, requires reviewed2305, and is1,230,119 bytes / SHA256 `bbd1b5ade84957b0ad5241addf025a890d32a639d07d1afc60b2edf8bff74a07`. Isolated mechanism/art bundle identities are in the same binding record. Bundles and raw local runtime evidence were not uploaded as attachments; the normal source branch and this sanitized repository record provide the review delivery.

Only Markdown and sanitized JSON are added by the delivery commit. No seat auth, entry response, raw private database, full room state, unredacted HTTP/view snapshot or actual runtime screenshot pixels are committed. Screenshot and external QA-driver hashes bind local evidence. The local ENOSPC interruption was resolved by clearing completed reproducible compilation/test copies while retaining source, ABI history, audit binaries and evidence. The optional Library helper still returned401; no retry or alternate artifact upload was used. Site35 and main remain unchanged by this delivery.
