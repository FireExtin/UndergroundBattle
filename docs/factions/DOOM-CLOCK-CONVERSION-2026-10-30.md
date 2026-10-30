# Closed clock conversion continuation

Base a44e6c2b785e0f29795b2a8b7d87750040dddd3d. This is local engineering WIP, not independent approval or public admission. BQ047/MSJZ03/JZ42 complete-card increments remain zero until the complete natural conversion and applicable validation close.

Original images inspected directly:
- JZ42: `resource/ymsj-fun.github.io/cards/JZ42 末日钟降临.jpg`, SHA256 a1e754694aa8b45153a678f49fbfa7139d3bdaa0be55d88da7216ca88efb90f2. Printed19, five red loyalty, disaster/eternal will, unique, cannot respond. Each destroyed region reduces4; removing X time from own cards reduces X; replace own society with the clock.
- MSJZ03: `resource/ymsj-fun.github.io/cards/MSJZ03 末日钟.jpg`, SHA256 56051dc91558ed994570aee388aefa0d2f34e8b4b8e813ddc96ee923aaf3a5f2. Not an initial society. Disaster-only play/paid reveal; no gained points; three destruction markers win. Two exhaust actions: time placement/destruction, or hand discard/generate priest.
- TK009 original has permanent investigation1/combat2, defense2, human/mage/cultist, death-observed time placement and influence per time. It uses the already verified official token zero-cost/no-loyalty/nonasset rule; it does not inherit TK010's special paid-reveal prohibition.

Project execution choices, not official rulings:
- JZ42 red cost ring is treated as nonasset, consistently with the prior localized red-ring choice; no other printed card is changed.
- MSJZ03 “then that region” means the target's actual physical region at resolution. An attachment without a region receives time but does not invent a region to destroy. Explicit region targets bind the original region UID; ordinary targeting does not enter core zones.
- Conversion replaces the printed society definition on the same physical society object. Existing legal damage/status/attachments and society-zone records remain; it does not clear unrelated state or import an old ABI.
- New region scores while the clock is active retain explicit zero-score receipts. Earlier points remain; newly acquired destruction markers do not retroactively deduct earlier points. Three personal destruction markers use the existing team victory path.

The payment draft, resource/time receipts, conversion, action frames and token provenance reuse existing serialized state and queues. Original compiler failures and test failures remain in `/workspace/bq084-validation/logs/doom-*`. The first passive score failures were incomplete test draining; the later payment failure was a real pending-state timing check corrected to authenticate the original standard window. Illegal simultaneous exposure/secrecy and society time were corrected only in test preparation. An initial action test passed only one response rather than resolving the stack; explicit region UID binding was also missing and has been added.

Current running selection: doom-actions02, eleven targeted tests including a new natural private50-card MSCQ02 opening → two ordinary BQ047 entries → JZ42 conversion → clock actions and BQ047 condition. No final result claimed while running. Cross-platform checks for this increment are pending; earlier a44e6c2b checks are not reused as proof of this Rust change. All candidates remain supported:false; no independent review, upload, push, main merge or deployment.

TK009 original SHA256: 53fab1195f462cad1c3499705a2dc6bc5d2d116d9a1edd8a991958e0cbb6f707. Its printed observer reuses JC045's already existing death event and influence modifier, while retaining distinct token provenance, zero-cost paid reveal and disappearance behavior. No printed JC045 card is converted into a generated object by name.
