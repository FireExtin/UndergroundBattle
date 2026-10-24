# Shared keyword reward dispatch: author WIP

The post-publication Native workspace audit found a normal-play defect in
ABI111: a real JC074 renown grant or JC089 combat-glory grant reaches a shared
reward declaration, but twenty recipients' native-program validators reject
it. This is independent of malformed-save exploration. The original failing
run and every intermediate failed attempt are preserved.

Runtime fix: `79de7154902a952d6dba7e29c1b211c3c9becf2e`.
HTTP fixture identity correction: `e9f3a15b419022ea9bf3712dee0aa8fff27849ef`
(two initial-layout fields only). Final recipient/pending-contract correction:
`b9dc611cdf433652b933dd2d68f5ccd27676e252`. These descend from the ABI111 evidence freeze
`55dc95d3cd4d7dc1fb8405179e782b9f8906be1d`. No GitHub main or deployed source
was changed by this repair.

## Resulting behavior

The shared reward validator runs before recipient-specific native programs.
Only the existing complete, cost-free, target-free one-influence renown or
combat-glory program is admitted. It authenticates the character snapshot,
actor, exact original region, atomic frame, response wrapper and optional
accept/decline contract. Reward abilities cannot be transplanted into static
definitions. A later response removing the character or grant does not cancel
an already observed event; a replacement region does not inherit its reward.

Both reward generators now freeze the original region for every recipient,
using the existing snapshot field. No schema, fee model, generic transaction,
card definition, supported flag or public view shape was changed. The newest
engine identity is `rust-v0.2.112-shared-keyword-rewards`; the card pool remains
`limited-v2.102-import20-rulings-complete`. There is no ABI111 state migration
or compatibility core in the ABI112 candidate.

## Actual author checks

* Existing normal-grant audit: combat glory 145/145 recipients through actual
  paid grant, accept and response commands; renown 145/145 actual paid grants
  through declaration/restore. No recipients skipped. XQ46 receives real
  friendly defense support. Ready XQ22 still forbids JC074; it receives the
  real WM064 human-host tattoo with six actual black assets in a standard
  action window instead. The spell prohibition was not relaxed.
* Final keyword/failure/adjacent Native scope: 95 passed. This includes source
  departure, replacement regions, exact programs, all nine failures below and
  illegal command atomicity. Existing invalid pending/recipient assertions remain.
* Final Native workspace integration scope: 72 passed, zero ignored, including
  attachments, deck construction, service/session/pacing and the two repaired
  grant matrices, plus all 27 BQ030 integration cases. The previously deferred
  pause trace executed and passed. WASM host-unit and workspace doc targets
  completed with zero tests; they are not counted as actual WASM execution.
* Four fresh actual Room witnesses: WM030, XQ19, CQ24 and BQ026. Their complete
  Native/WASM catalog, 20 normal commands, 96 views, opaque states and u64
  decimal seeds match exactly.
* Actual local Workerd/HTTP/D1: all four scenarios passed, including 20
  commands, 20 identical duplicate receipts, 96 views and exact stored
  states/journals. These are isolated synthetic initial layouts and normal
  protocol commands, not browser or public acceptance evidence.
* The full Native library invocation completed: 2494 passed, 9 failed, zero
  ignored. The failures identified the pending-envelope and character-only
  admission contracts, five stale exact engine pins, one old explicit queue
  fixture lacking current frozen-region/frame metadata, and the card-ID error
  context used by the XQ37 test. All nine passed in the final 95-test rerun.
  The queue probe now uses actual printed renown, preserves its departed source,
  original region and a real frame ID; its no-rescore assertion was retained.
  The XQ37 synthetic reward variant retains its canonical prompt metadata and
  all 45 original rejection mutations. No rejection assertion was weakened.
  The full failed invocation and directed rerun overlap and must not be added
  as a unique count or called one clean full-workspace invocation. The 72
  integration cases and zero-test targets were subsequently completed.
* Initial workspace compilation also exposed four stale type initializations
  in three example-only files. Current empty payment/observation fields and the
  damage-pattern field were added. Historical opaque-state adapters were not
  executed or used as evidence.

WASM was built with Rust 1.90.0 and wasm-bindgen 0.2.104 under the explicit
normalized source/Cargo paths. The final sole module is 8,128,116 bytes, SHA-256
`324add323aea19ef6af5c633d6dbbd96d8d36fd49bcd41853626002861726b4f`.
It was rebuilt after the final contract correction. Earlier narrower probes
remain separately identified with their original module/source hashes.
This is author execution, not an independent reproducibility claim. The ignored
local development package now contains the final ABI112 module; the previous
ABI111 package was retained intact in task evidence. No deployed output changed.

The first HTTP attempt returned 404 because the older unit-layout room name
was not a legal HTTP room identifier. Rust regenerated new witnesses with
valid IDs; no opaque JSON string was relabeled. That failed run and the first
WASM-only probe are retained separately. The current112 HTTP fixture was
generated from the final directed Native exporter and checked against the final
actual WASM. Earlier probes were moved intact into task evidence. The loader
retains the complete 329-file source inventory, module and raw-JSON hash checks.
Protocol numbers are checked for safe integer representation; private u64 state
remains opaque. This four-case fixture is reusable, but does not replace the
broader UI/Worker fixture families.

## Remaining scope

All failures described here have author-directed closure evidence; independent
review remains pending. This is an unreviewed local repair WIP. Full Web/Sites ABI111 evidence is
preserved at the previous freeze; it is not represented as ABI112 evidence.
The broader current-only UI/Worker inputs need fresh ABI112 exports before
their complete suites can be reused. No historical opaque state may be
rewritten to satisfy that requirement. The four-scenario platform proof above
is narrower than full-batch platform acceptance.

The live overseas Site still runs v3 ABI111, including its repaired card-face
indexes. Its natural terminal browser match (turn6, A9:B0), both-seat refresh
and twenty image loads remain ABI111 evidence. ABI112 has not been published,
played in a browser or independently reviewed. Native fifty-card and import
twenty-card counts are unchanged by this cross-card repair.

The latest checkpoint instruction forbids triggering GitHub CI. Normal push
events for the active dynamic Copilot workflow cannot be proven absent, so
new commits/bundles remain local. No dispatch, rerun, workflow/security change,
forced push, GitHub main update or public health-endpoint retry occurred.
