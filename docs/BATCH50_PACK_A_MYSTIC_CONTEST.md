# Pack A: hidden contest native handoff (WIP, not counted complete)

Lifecycle follow-up from integrated baseline `47256d59…` is documented in
[PACK_A_LIFECYCLE_2026-10-22.md](factions/PACK_A_LIFECYCLE_2026-10-22.md).
It supersedes this historical handoff's nesting/response-order/terminal-state
limitations only. B hooks, costs and end-to-end admission remain incomplete.

2026-10-22. Independent ordinary clone `UndergroundBattle-packA`, branch
`wip/batch50-pack-a-hidden-contest-20261022`, base
`adf0a1c080e21b75e971bd650a61f22e47f42e91`. No author worktree writes,
remote pushes, CI, deployment, or admission. Author-owned XQ20,
`batch4_markers.rs`, its tests and `batch4_plans.rs` are untouched.

Inputs are the existing original images WM021, WM022, WM032, WM061, BQ082,
`resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf` physical pages 20
(printed19), 22 (printed21) and 23 (printed22), and
`resource/ymsj-fun.github.io/public/docs/帷幕之后说明书.jpg`.
Original images were visually read, not inferred from third-party research JSON.
The five card image hashes remain in the appended runtime records. No third-party
implementation code or images were copied. BQ082 has the printed sun condition
before public: its public keyword applies only with its controller's exposure.
This correction is recorded here and in the runtime text; the historical source
specification JSON has not been silently rewritten.

## Contract for pack B

Files `mystic_contest.rs` and `mystic_hooks.rs` own one protocol. Existing frames,
response stack, private Pending choices, RNG and clone-before-apply remain in use.

```rust
Game::mystic_contest_start(frame: ResolutionFrame, actor_bonus: u32) -> RuleResult<()>
Game::mystic_add_bonus(frame: ResolutionFrame, amount: u32) -> RuleResult<()>
Game::mystic_start_observers(&mut self, contest_id: &str) -> RuleResult<()>
Game::mystic_win_observers(&mut self, outcome: &ContestOutcome) -> RuleResult<()>
Game::mystic_shown_after(&mut self, outcome: &ContestOutcome, index: usize) -> RuleResult<()>
Game::mystic_extra_shown_bonus(&self, shown: &MysticShown) -> RuleResult<u32>
Game::mystic_validate_bonus(&self, bonus: &MysticBonus) -> RuleResult<()>
```

The five hook implementations are currently empty/zero, and bonus validation
rejects all unregistered receipts. B implements these hooks by delegating to its
own finite card validators; there is no generic user-provided numerical bonus.
`MysticBonus` stores `contest_id`, the entire resolved frame, and `amount`.
The BQ038 operation must authenticate its own exact event, original source,
current/pre-captured counted-character evidence, once-only frame, and amount.
It calls `mystic_add_bonus` only during `BeforeSelection`; restored receipts call
`mystic_validate_bonus` too. If the receipt needs B-specific proof, place the
proof in its B-owned operation/frame type, not arbitrary JSON.

`ContestOutcome` retains `contest_id`, `turn`, `frame_id`, `actor`, `source`,
`ability_key`, `actor_bonus`, `bonuses`, `participants`, `shown`, `totals`,
`winner_team`, `first_winners`. `MysticShown` retains `seat`, original `card`,
`manor` (owner vampire aura witness), `vampire_observers` (personal WM021),
`destroyed_regions`, `destruction_markers`, `value`.

Start hooks enqueue ordinary declarations before one `Effect::MysticContinue`.
The protocol waits only for response frames added above its captured lower stack.
Each living seat puts one deck top into hand (not Draw), commits privately in seat
order, and all reveal only when every commitment is present. Printed cost,
printed purple/hidden bonus, controller's WM021 bonus, actor bonus and admitted
start receipts contribute to individual values; teams sum individual values.
Shown cards reset identity and shuffle into their owners' decks once. The outcome
is captured before optional winner marker choices, and win hooks enqueue normal
trigger declarations. Each winning person chooses own secrecy or the directly
opposite enemy's exposure (0↔2, 1↔3; duel0↔1), or declines.

Shown-card post hooks run in participant order after rewards, after the shown-card
shuffle. The continuation cursor advances before calling a hook, and a private
pending choice may pause it. Hooks must finish their own entire effect before the
next `MysticContinue`; they must neither reshuffle the contest nor select a new
hand card. WM061's shown post-effect uses its own original shown-card frame as the
cause of the existing rumor replacement, not the contest initiator's source.

B must extend the finite start admission/dispatch boundaries for its actual
starter cards (currently only WM022 and BQ082 are admitted), and its own definition,
frame, declaration, pending, and restore validators. Pack A's `related` guard
currently owns the shared Mystic ops; integration must route B's admitted
programs to B validation before that guard. No B cards are implemented here.
BQ031 uses `outcome`; first wins are personal for the turn, not reset per observer.
BQ027 uses the exact shown-card snapshot/index. BQ038's printed purple symbol is
an individual hidden-contest bonus, not regional influence.

## Verified checkpoint

The five bounded programs are implemented; the shared start hooks remain an
integration seam for B. **Completed pilot count stays 22/50.** These five are
separately recorded as `nativeWipDirectedPassedPendingClosure`, with zero added
to completion, admission, WASM, browser acceptance or publication counts. The original `batch4::IMPLEMENTING` remains the same 22 IDs; the five WIP
programs have a separate finite `pack_a_contest::CARDS` registry. All
five appended runtime records remain `supported:false`. The original 213-card
canonical source fingerprint is unchanged. Historical source specs are unchanged.

The exact final source was tested with Rust1.90.0, `cargo test --offline --lib
--no-default-features pack_a -- --test-threads=1`: **32 passed**. Adjacent tests
using filters `batch4_manor batch3_alpha batch3_entry_order rumor
recovery_validation_tests rules::tests`: **62 passed**. These are native rules
and persistence tests, not native-service endpoint or WASM/browser tests.
The source/roster test ran **2 assertions successfully** via direct Node execution (file-only subprocess runner output was not used as assertion evidence). It verifies original image hashes, icons, costs,
the frozen213 fingerprint and disabled public admission. The five unfinished
cards are explicitly counted apart from passed pilots in that check.

Coverage includes real paid reveal/deploy/play commands; private commitments and
four-seat projections; duel/team sum/tie; first win per person per turn;
controller-only WM021 and owner XQ15 aura; WM032 conditions and surviving queued
triggers; personal rewards/opposite enemies; BQ082 double-black loyalty,
conditional public/WM041, original grave target/region and exhausted reentry;
WM061 all-player fixed slots, post-shuffle milling and rumor pauses; u64 RNG;
Game and Room recovery parity; atomic rejected/replayed choices.

Integrator review cases were fixed and retained:

- WM061 truncation/reorder/duplicate/replaced seat contexts reject in Game and
  Room. Its expanded frame has every original seat; eliminated seats use the
  existing mill no-op. Elimination before and after payment both pass.
- Deleting a still-living loser from a completed outcome was reproduced as
  accepted before the fix. Both recovery envelopes now reject it; history may
  retain players who participated then were eliminated later.
- Shared ops cannot be transplanted through modes, either player loop, or
  conditional branches. Recursive ownership is checked in static definitions,
  response frames, queued frames, declarations and frame choices. A genuine
  foreign JC125 deploy frame supplies the mutation baseline.
- Response-card outer fields must match the existing fresh-card transformation; forged marks, old IDs and missing frames reject. A queued effect frame is accepted only for the exact existing rumor continuation, so it cannot bypass the response stack.
- Beginning-response elimination keeps the original start receipt but removes
  eliminated seats before private selection. A lower ordinary response stack
  does not prevent a newly started contest from selecting cards.

The A and B winning observers must enqueue their declarations before the core's
**single combined** `batch3_register_entry_order` call. B's win hook must not
separately register a competing order group for the same event. B must extend
that module's finite introduced-card list for its own observers.

Validation used an independent on-disk Cargo cache/target outside Git, offline,
1 Cargo job, 1 test thread, two CPU affinity slots, nice15, and the repository's
storage guard. Original toolchain paths were read-only. No credentials, database,
cache, compiler output or temporary rendered rule pages are staged.

## Remaining boundaries

If a participant has neither hand nor deck, the original rules read here do not
settle a shown-card value. The implementation explicitly rejects that contest
without inventing printed cost0. A participant with an empty deck but an existing
hand is supported and is not eliminated as though drawing.

Nested contests while another contest is active are not admitted. In particular,
B's responsive start hooks must cover response-trigger ordering and nesting before
those hooks are enabled; empty hooks are not evidence of that behavior. Later
elimination caused by a B shown-card post effect and game-end cancellation also
need B integration tests. These are documented limitations, not completed B work.

Integration must extend the finite starter whitelist/Op dispatch and B receipt
validation, then rerun A/B cross-card ordering and persistence cases. WASM, UI,
public pool admission and independent integration review remain pending. The
source-check roster adjustment preserves the author's 22 completed pilots while
accounting for the five present WIP definitions. No merge or push was performed.

Local checkpoint ancestry starts with module commit
`b7b50eaec0ad52ac2256e575cbf4ab8cc749b5a7` and minimal wiring commit
`619a411715e49203ebde1795bef380f7b3341e11`. Subsequent hardening and its minimal
WM061 frame-expansion wiring are separate commits. Use the final handoff SHA and
cherry-pick the full sequence; the initial WIP commits alone are not the final fix.

## Read-only reconciliation with pack B

Read B delivery `a5db0f5ebc28cdb5e76299f34ad4f7c7e61a2379` and its source
handoff. No B files were edited or independent blood/search abilities duplicated.
Integration mapping for the five outstanding clauses:

| B clause | Shared seam and binding |
| --- | --- |
| BQ027 purple +2 | `mystic_extra_shown_bonus` from the exact shown definition |
| BQ027 post-contest own vampire feeding | `mystic_shown_after(outcome,index)` with that original shown card as effect source; use B's existing private blood program |
| BQ031 wins | `mystic_win_observers` captures winning participant/controller from the same outcome and queues a B-owned validated trigger |
| BQ032 hand action | After the existing pay2/discard-self transaction, resolve the admitted B-owned frame's `Op::MysticContest { actor_bonus:0 }` |
| BQ036 action | After pay1/qualified-attachment sacrifice, resolve its admitted B-owned frame's `Op::MysticContest { actor_bonus:2 }` |
| BQ038 start trigger | `mystic_start_observers` emits a normal respondable B declaration carrying contest ID. Its resolved count proof is retained in `MysticBonus.frame`; call `mystic_add_bonus` only after B validation. |

B's partial-definition validator must be extended with its own complete programs;
A does not supply fake free activations, infer its target/cost signatures, or
claim B completion. The explicit finite admission points are `admitted_start`
and `validate_pack_a_frame`/`rules::validate_ability` routing as described above.
Merging source guards requires preserving B's five historical-spec corrections
and A's separate WIP accounting; neither baseline fingerprint may be weakened.
