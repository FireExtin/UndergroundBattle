# Historical Web bindings, current36

Code candidate: **5cbea3dd15162587e0abdbfa0bf591b6c97bc173**, parent **baed2594400dc38834943ab70ee2c42afcda7b72**. A following documentation commit adds this record only. The code commit changes 15 historical test bindings and adds three byte-exact historical scan-registry snapshots. It changes no UI production, Rust rules, current kernel bytes, public scan registry or existing golden. This is an independent test branch; no main integration or publication occurred. Parent independent review is pending.

The former20 failures came from historical expectations/fixtures using current36. Pinning each existing historical catalog/scan test to its own frozen kernel allows its existing assertions to execute against the recorded version. No assertion was deleted, loosened or rewritten. Reversing only the kernel/registry paths and removing the explanatory header restores every modified test file byte-for-byte to baed.

| Frozen catalog kernel | Test files |
|---|---|
| 32 | JC005Scan, JC008Scan, MSJC01Scan, MSJC06Scan, MSJC07Scan, MSJC08Scan, MSJC11Scan, SocietyScan, YellowSearchBatchScan |
| 34 | GreenMinimumScan, JC029Scan |
| 35 | BlueMinimumScan, JC030Scan, JZ31DeathInfluence, MillPublicBatch |

Existing explicit fixture readers remain intact: GreenMinimum fixtures still use frozen31; BlueMinimum fixtures still use frozen33; JC030 fixtures still use frozen32. JZ31 and MillPublicV035 fixtures now reach their matching frozen35 reader. All pre-existing fixture JSON and all193 frozen-kernel files retain original bytes. Current36 still uses the original current package in unchanged `JZ55UniqueDestroy.test.jsx`: eight tests, including21 complete native fixtures/84 seat projections, catalog100/eight societies/five presets, current108 scans, real response/death cascade, hidden targets and draft limits.

The new registry JSON files are complete historical Git blobs, not a filtered current registry:

| Fixture | Exact source commit | Entries | SHA256 |
|---|---|---|---|
| cardScansV032.fixture.json | 713be787f47d4188640922a5d373662bd04ffe38 | 101 | 9a46bf09c8cf4d6b6b1c13d928739e171b036c2762fd0b0f0788975faa5f20a6 |
| cardScansV034.fixture.json | 3ee9f70a707714d776cf5a19e106963a56bfef05 | 106 | 2a38e937782e7ca0c80f4b0fdc79bd3c382e8eb57952b535644721084d99a9ce |
| cardScansV035.fixture.json | b533ef25318585bd5d2b1c2a9225ea891108b339 | 107 | 8c06e08c8d0abbb81026e92c3af4a65305498d529129731bb6b37c97568f5f8f |

Each source path is `web/public/card-scans.json`. The production registry remains108 entries and is independently asserted by the unchanged current36 suite. The actual reader still uses the current production card-art map; historical URL/SHA assertions continue checking the actual original image bytes.

Validation on the code candidate used the previously formally built current36 package, SHA256 `0f3a637d854074fe0689927c51e7969e8a32551ddcb48e1e1f3f1b6d022ea036`:

- Full Web on baed plus this test-only commit: **60 files,473 passed,0 failed,0 skipped**, exit0. All473 identities match the original baseline; the former20 failed identities now pass.
- Local combination with Lantern UI evidence237a6bea: **61 files,476 passed,0 failed,0 skipped**, exit0. Combination HEAD `664c643aa199e6bb67f1063180f827e5857641bf`; all four UI production files equal fixed5862ad6 bytes. The extra three same-name instance tests remain green. This validation branch was not pushed or deployed.

Actual commands from each worktree's `web/` directory:

```bash
npm test -- --reporter=default --reporter=json --outputFile=/workspace/lantern-historical-bindings-evidence/full-web-results.json
npm test -- --reporter=default --reporter=json --outputFile=/workspace/lantern-historical-bindings-evidence/ui-combined-web-results.json
```

The only additional files in the subsequent evidence commit are this REVIEW and `summary.json`, which records counts, provenance, source-preservation checks and log hashes. It contains no session credentials, raw browser storage, database dump, private room views, screenshots or full HTTP log. The separate Lantern natural-UI evidence still belongs to fixed5862ad6; these new results are full-Web combination regression only, not a new natural-UI or publication claim.
