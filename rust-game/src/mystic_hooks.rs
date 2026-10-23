//! Finite B adapters; shared protocol scheduling remains in mystic_contest.
use crate::{engine::RuleResult, model::Game, mystic_contest::{ContestOutcome, MysticShown, MysticBonus}};
impl Game {
    pub(crate) fn mystic_start_observers(&mut self, contest_id: &str) -> RuleResult<()> { self.b_mystic_start(contest_id) }
    pub(crate) fn mystic_win_observers(&mut self, outcome: &ContestOutcome) -> RuleResult<()> { self.wm044_observe_win(outcome); self.b_mystic_win(outcome) }
    pub(crate) fn mystic_shown_after(&mut self, outcome: &ContestOutcome, index: usize) -> RuleResult<()> { self.b_mystic_shown(outcome, index)?; self.wm037_shown(outcome,index) }
    pub(crate) fn mystic_extra_shown_bonus(&self, shown: &MysticShown) -> RuleResult<u32> { Ok(self.b_mystic_extra(shown)) }
    pub(crate) fn mystic_validate_bonus(&self, bonus: &MysticBonus) -> RuleResult<()> { self.b_mystic_validate_bonus(bonus) }
}
