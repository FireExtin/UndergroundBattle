# Attachment UI handoff — 2026-10-02

This is frontend implementation and component-test evidence, not browser/rules acceptance. No room/profile was operated; no build, commit, publishing, API/core/catalog/data/scan edits by this agent. Root owns final review/build; Rust agent owns attachment rules. Formal new core/pool versions are rust-v0.2.6 / limited-v2.4. UI does not hardcode those versions or deck counts.

## Scope

Changed: web/src/game/types.ts, CardTile.tsx, Table.tsx, ReadModal.tsx, CardTile.test.tsx. Added: attachments.css and Attachments.test.tsx. CSS contains only attachment entries/count and current-stat rows, with no global table geometry, breakpoints or choice changes.

Contract: optional View.attachments, each entry is the flat CardView plus hostId. Card.region is the host's present region. Separate owner/controller. New core projects mounted cards face-up and never places them in regions[].characters. Old views without the field render normally. The UI neither checks host subtypes nor decides side/region eligibility. It continues to forward the original server legalAction with play/cardId/targetId and all existing fields.

Host small card shows “附属 N” with data-attachment-count. Host inspector contains “此角色的附属” entries with own data-attachment-instance/data-attachment-host, public owner/controller/host/current-region labels, and a read button. Attachment read reuses the existing modal. No independent region piece/icon contribution is constructed for an attachment.

For face-up characters in a region, full reading shows server Card.icons as “当前有效” and catalog icons as “印刷图标”; current defense and print defense are distinguished. Compact pieces retain the server's icons. Zero values are preserved. Without a frozen definition, effective icons are not falsely labeled print. Regions and concealed identities retain their previous reading semantics. There is no client “combat+1” calculation or card-ID branch.

Reader state now stores only room/viewer/instance references, resolving the latest authorized allCards, active choice or public region projection each render. Other-owner recovery removes the attachment from this view and immediately closes its reader. Same-instance own recovery uses the new hand projection and removes old host context. Movement updates the visible host region while preserving the attachment identity. Viewer changes or ended private choices close reading. No stale card face payload is retained as a fallback.

## Validation

Command: npm test -- src/game/Attachments.test.tsx src/game/CardTile.test.tsx src/game/Table.test.tsx src/game/ChoicePanel.test.tsx src/game/ResponseWindow.test.tsx src/game/AutoPass.test.tsx — six files /64 tests passed. npm run typecheck passed. One additional private-choice lifecycle test was then added; npm test -- src/game/Attachments.test.tsx passed all8 attachment/reader tests. This is65 unique targeted tests across unchanged suites, not a full frontend suite. git diff --check passed for modified tracked frontend files.

New attachment/reader test names:

- uses only server-authored hosts, including an opponent in another region, and forwards the original play action
- counts and reads mounted cards without treating them as region characters or recomputing the host bonus
- updates an open attachment reader from the latest moved host and keeps the same attachment identity
- closes an open public reader when the attachment returns to another owner’s hidden hand
- uses the newest own-hand projection after recovery and drops the old host context
- clears the reading selection on a viewer change rather than reusing a face from the previous seat
- reads a chooser-only search card from the current choice and closes it when that private projection ends
- keeps an old room readable without an attachments field

New CardContent protections cover authoritative current/print icons and defense including zeros; no false print label without catalog; attachment type with no independent fighting icons; unchanged world reading values. Existing owner-hidden, action-targeting, choice, response and optional-pass tests also passed in the targeted command.

## Remaining acceptance

Root's planned new normal four-seat persistent QA will validate actual core wire, normal BQ022 play/payment/response, accumulated effects and owner recovery. No browser gameplay is claimed from these component tests. No scans are served. Existing saved public C26, native3 and native5 contexts/profiles remain untouched.
