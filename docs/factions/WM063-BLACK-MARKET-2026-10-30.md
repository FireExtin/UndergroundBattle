# WM063 black market — closed implementation work

Baseline 2ac753148398722a09e68099dd631d8a5e9db0c0. No independent approval or admission; complete-card increment remains zero until the applicable natural/cross-end chain is verified. MSWM03 is not implemented by this change.

Source: tracked `resource/ymsj-fun.github.io/cards/WM063 黑市.jpg`, SHA256 `4f99db76b9f7da15c1a0a7daa242ddb8a94040b9410dbe53048e83675d066228`. Black attachment, cost2/one black loyalty, facility/criminal. The eye in the lower-right is the expansion symbol, not a magic-domain grant. Three ordinary actions: pay2/discard a spell/draw1; pay1/discard a character/repress2; pay1/discard an attachment/place one permanent combat icon marker on target character. No exhaust cost is printed.

The card omits an attachment-host line. The manual attachment section requires an in-play card or region host and does not supply this missing default. **Project ruling, not official text:** WM063 alone attaches to its controller's own society. Other attachments gain no default host. The personal secrecy interpretation follows the existing WM057 project boundary: an enemy may borrow only while that enemy's own society has secrecy; a teammate's/enemy partner's secrecy does not authorize everyone. Costs and benefits belong to the actual actor; payment snapshots preserve authorization through later responses.

The manual's Repress X entry (printed page3) says the actor chooses a player, and that player chooses/removes X of their influence, or all if fewer; zero markers has no effect. The black-market implementation must follow this two-player choice sequence, including selection of oneself, without copying narrower existing card implementations.

WIP checks cover paid attachment, typed discard, borrower ownership, marker placement and serialized repression choices. Test/compilation failures are retained under `/workspace/bq084-validation/logs/wm063-*`. Final coverage and source identity will be recorded after execution; this document does not claim completion, full-library validation, or browser play.

WIP checkpoint: Native targeted7/7 passed (run wm063-native06), including real Room natural50-card attachment→draw and three prepared enemy-borrow actions. Compilation/host-registration/source-binding/fixture-seat failures from runs01–05 remain preserved. WASM/UI/Worker and related regression are pending at this checkpoint, so complete-card increment remains0. The manual PDF SHA256 is `2a771874efbdb3ad8a339ea01ff62888766090d075fe2e013632c79de1aeba35`.

## Closure of this card

The earlier WIP0 count is superseded: WM063 is one complete **closed** implementation, independent approvals0, supported:false. MSWM03 remains unimplemented; this does not establish a new reviewed20-card batch or browser play.

Tested Rust source: `b4f3c10a8668fb66f77e4b0d616a7dc102967c10`,444 source/lock inputs. Final changes after it only correct a UI test's required target-selection interaction and add this report; production Rust, WASM input and lockfile bytes remain unchanged.

- Native7 targeted +43 disjoint relevant borrowed-action/marker/attachment/JC008 checks:50passed,0failed,0ignored. No full-library rerun on this source.
- Default WASM build and Native parity:4cases,332commands,1328views,332actual duplicate outputs. Raw Native SHA256 `5ebc682c02f0f2dda7c3ebf3db6b9e2da6bdc6f41b445138df3bb6b70e436612`; source manifest `c364bef81f03a759603a868c642114d55ce19aa60af82f566de02d970aa0e87e`; WASM `b03438e93162d4baac0da61d3f8d9ea5a32fee1ca37a35d9a44ef5cb0bea8f98`.
- UI related whole selection:9files/198passed. First run197passed/1failed is retained; that test omitted the existing ability-group→target→confirm UI sequence. No production UI or assertion was weakened.
- Local Worker HTTP/D1:4passed/0failed/0skipped,332commands and cold reopens,1328views,332duplicate receipt/database invariance checks, original receipt rechecked after each complete scenario. Full natural315commands were replayed, including162before the first black-market play and153from attachment through following draw. Remaining17commands are three explicitly prepared borrow scenarios. The harness diagnostic `nativePreparationCommands:0` denotes no omitted Native-only prefix here, not zero preparation.
- Web and local Worker builds passed. Rust1.90.0/cargo1.90.0, Node24.19.0, wasm-bindgen0.2.104; default native/WASM, one build job/test thread. Cargo.lock SHA256 `1745cc1af931f500e763222c39100da3ac0521f4a1f96fc54e82c8589e2edb54` unchanged.

Evidence stays under `/workspace/wm063-validation/` and original command logs under `/workspace/bq084-validation/logs/wm063-*`. Local files/bundle are not a claim of Library availability; existing Library401, independent-review and push restrictions were not retried. No merge, deployment or public admission. Remaining focused follow-up is MSWM03's own original search/action, not a second count for WM063; paused neutral task regions and unrelated clock/omen groups remain outside this slice.
