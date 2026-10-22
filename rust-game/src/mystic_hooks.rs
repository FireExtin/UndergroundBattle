//! Integration seam for pack B. Empty hooks have no gameplay effect. Card
//! implementations must validate the entire original program and receipts;
//! this module never admits arbitrary numerical bonuses or copied operations.
use crate::{engine::RuleResult, model::Game, mystic_contest::{ContestOutcome, MysticShown, MysticBonus}};
impl Game {
    pub(crate) fn mystic_start_observers(&mut self, _contest_id: &str) -> RuleResult<()> { Ok(()) }
    pub(crate) fn mystic_win_observers(&mut self, _outcome: &ContestOutcome) -> RuleResult<()> { Ok(()) }
    pub(crate) fn mystic_shown_after(&mut self, _outcome: &ContestOutcome, _index: usize) -> RuleResult<()> { Ok(()) }
    pub(crate) fn mystic_extra_shown_bonus(&self, _shown: &MysticShown) -> RuleResult<u32> { Ok(0) }
    pub(crate) fn mystic_validate_bonus(&self, _bonus: &MysticBonus) -> RuleResult<()> { Err("尚无登记的隐秘开始加值收据程序".into()) }
}
