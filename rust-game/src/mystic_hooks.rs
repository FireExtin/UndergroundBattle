//! Finite B adapters; shared protocol scheduling remains in mystic_contest.
use crate::{engine::RuleResult, model::Game, mystic_contest::{ContestOutcome, MysticShown, MysticBonus}};
impl Game {
    pub(crate) fn mystic_start_observers(&mut self, contest_id: &str) -> RuleResult<()> { self.b_mystic_start(contest_id) }
    pub(crate) fn mystic_win_observers(&mut self, outcome: &ContestOutcome) -> RuleResult<()> { self.wm057_observe_win(outcome); self.wm025_observe_win(outcome); self.wm044_observe_win(outcome); self.b_mystic_win(outcome) }
    pub(crate) fn mystic_shown_after(&mut self, outcome: &ContestOutcome, index: usize) -> RuleResult<()> { self.wm043_shown_after(outcome, index)?; self.b_mystic_shown(outcome, index)?; self.bq034_shown(outcome,index)?; self.wm037_shown(outcome,index) }
    pub(crate) fn mystic_extra_shown_bonus(&self, shown: &MysticShown) -> RuleResult<u32> { Ok(self.b_mystic_extra(shown) + crate::next20_blue_vampires::shown_bonus(&shown.card.definition)) }
    pub(crate) fn mystic_validate_bonus(&self, bonus: &MysticBonus) -> RuleResult<()> { if bonus.frame.source.card.definition==crate::mswm02::ID||bonus.frame.ability_key==crate::mswm02::CONTEST {self.mswm02_bonus(bonus)} else {self.b_mystic_validate_bonus(bonus)} }
}
