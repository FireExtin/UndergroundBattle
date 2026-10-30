# WM063 black market — closed implementation work

Baseline 2ac753148398722a09e68099dd631d8a5e9db0c0. No independent approval or admission; complete-card increment remains zero until the applicable natural/cross-end chain is verified. MSWM03 is not implemented by this change.

Source: tracked `resource/ymsj-fun.github.io/cards/WM063 黑市.jpg`, SHA256 `4f99db76b9f7da15c1a0a7daa242ddb8a94040b9410dbe53048e83675d066228`. Black attachment, cost2/one black loyalty, facility/criminal. The eye in the lower-right is the expansion symbol, not a magic-domain grant. Three ordinary actions: pay2/discard a spell/draw1; pay1/discard a character/repress2; pay1/discard an attachment/place one permanent combat icon marker on target character. No exhaust cost is printed.

The card omits an attachment-host line. The manual attachment section requires an in-play card or region host and does not supply this missing default. **Project ruling, not official text:** WM063 alone attaches to its controller's own society. Other attachments gain no default host. The personal secrecy interpretation follows the existing WM057 project boundary: an enemy may borrow only while that enemy's own society has secrecy; a teammate's/enemy partner's secrecy does not authorize everyone. Costs and benefits belong to the actual actor; payment snapshots preserve authorization through later responses.

The manual's Repress X entry (printed page3) says the actor chooses a player, and that player chooses/removes X of their influence, or all if fewer; zero markers has no effect. The black-market implementation must follow this two-player choice sequence, including selection of oneself, without copying narrower existing card implementations.

WIP checks cover paid attachment, typed discard, borrower ownership, marker placement and serialized repression choices. Test/compilation failures are retained under `/workspace/bq084-validation/logs/wm063-*`. Final coverage and source identity will be recorded after execution; this document does not claim completion, full-library validation, or browser play.

WIP checkpoint: Native targeted7/7 passed (run wm063-native06), including real Room natural50-card attachment→draw and three prepared enemy-borrow actions. Compilation/host-registration/source-binding/fixture-seat failures from runs01–05 remain preserved. WASM/UI/Worker and related regression are pending at this checkpoint, so complete-card increment remains0. The manual PDF SHA256 is `2a771874efbdb3ad8a339ea01ff62888766090d075fe2e013632c79de1aeba35`.
