# WM057 normal-play private inspection acknowledgement

Overseas v4 / ABI112 actual dual-seat table `97EC1A99E58C` reached the required `wm057-inspection-confirm` 0/0 node on turn 4. The actor could read the correct private card, but its confirmation button was disabled and no skip existed. The other seat did not see the private preview. The original table is paused and saved at this node; this is a failed normal-play acceptance result.

Native `import20_broker.rs` requires the original actor to acknowledge with an empty selected list. The UI exact-kind zero-confirmation allowlist omitted this choice. The change adds only that kind while preserving the no-decline, min=0, max=0, empty-options guards, and labels this preview as a private inspected card rather than a deck-top card. No Rust, ABI, state or schema changes. Unknown/nonzero/inconsistent contracts, busy and missing-action submission remain closed.

New regression first reproduced the disabled button (1 failed, 6 passed). After the two-line UI correction, 4 files / 46 directed and adjacent tests passed, including actual-current-WASM Table controls. These are author tests, not independent review or completed browser acceptance. Full browser terminal, CQ28-specific mill and shared-keyword rewards remain outstanding. New packaging/identity checks are required before any UI fix publication.
