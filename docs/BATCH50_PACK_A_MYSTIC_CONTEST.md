# Pack A: hidden contest integration checkpoint (WIP)

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

## Checkpoint state

Source registration, five finite programs and shared protocol are present, but
this is WIP. First compile with no default features succeeded before directed
cases and rumor post-frame integration were added. The first ten native directed cases compiled and ran: 9 passed, 1 failed
because the destruction-marker fixture omitted its required won-region score
card. That fixture is being corrected; no all-pass result is claimed. Source roster remains
22/50 until actual passes; the five newly appended runtime records are still
`supported:false`. Browser/WASM/acceptance checks were not run.

Validation uses an independent on-disk Cargo cache/target outside Git, offline,
1 Cargo job, 1 test thread, two CPU affinity slots, nice15, and the repository's
storage guard. The original toolchain is read-only. No credentials, database,
cache, compiler output or temporary rendered rule pages are staged.

Remaining work: finish and harden the five native paths; shared Game/Room mutation
rejection cases; costs/response and BQ082 original-target/region cases; targeted
adjacent regressions and original source checks; update only honest pilot counts.
If a participant has neither hand nor deck, the original rules read here do not
settle a shown-card value. The implementation explicitly rejects that contest
without inventing printed cost0; this remains an admission blocker. Nested
contests while another contest is active are likewise not admitted yet.
The integration owner must review these boundaries and merge both the module and
minimal wiring commits; no public/full completion claim is made by this checkpoint.
