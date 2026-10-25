# Blue paid transaction review follow-up (WIP, 2026-10-25)

Base aedfe09249941b95ebf10c0c77ba4390194ae9e9; all four card admission gates remain false.

Two real Native command/persistence counterexamples show that replacing the deploy/reveal transaction UID by its retired source UID was previously accepted. Reuse the existing `pack_a_transaction_matches` full fresh-card proof: source UID < transaction UID and transaction UID + 1 == frame UID, plus the complete fresh snapshot. Source and transaction deliberately do not share one UID. No unrelated play protocol was changed.

The BQ034 no-vampire and no-blood exits clear only their own `post_frame`. The original `mystic_after_next` queues one `MysticContinue`; the blue helper neither queues nor invokes the continuation. Two directed actual-contest tests put WM061 at a later shown index and verify its one-card mill occurs once, no early advance while the blue target is pending, a single history result, no leftover continuation, persistence rejection of missing/duplicate barriers, and rejection of replayed target choice. The first draft incorrectly expected WM061's played mass-mill; corrected to the printed shown one-card effect. That test correction is not a production fix.

Evidence: `evidence/next20-blue-review-20261025/`. Red: two genuine UID counterexamples, plus two corrected test expectation failures. Green: 4/4 new tests. Blue family: 17/17. Native only; full regression, WASM, Worker, UI and independent review NOT_RUN. No push, CI, deployment, settings/workflow change, or original-table action.

Remote read at 2026-10-25T07:43Z: review/game-serial-abi99-wip-20261024 = e7b3366b16ed7d70783ce00bf96e87df4d220789, unchanged from prior evidence. Dynamic Copilot workflow remains active and its push trigger set unknown, so keep local checkpoints/bundles.
