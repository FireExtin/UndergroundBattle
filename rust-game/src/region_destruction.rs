//! A destroyed region stays in play. Its single marker moves to the winner's score area.
use crate::{catalog, engine::RuleResult, model::*};

impl Game {
    pub(crate) fn destroy_all_regions(&mut self) {
        for region in &mut self.regions {
            if !region.vacant { region.destroyed = true; }
        }
    }

    pub(crate) fn destroyed_region_count(&self) -> u32 {
        self.regions.iter().filter(|r| !r.vacant && r.destroyed).count() as u32
    }

    pub(crate) fn player_score(&self, seat: usize) -> u32 {
        let p = &self.players[seat];
        p.score_cards.iter().map(|c| catalog::card(&c.definition).points.unwrap_or(0))
            .sum::<u32>().saturating_sub(p.destruction_markers)
    }

    pub(crate) fn validate_destruction_state(&self) -> RuleResult<()> {
        if self.regions.iter().any(|r| r.vacant && r.destroyed)
            || self.players.iter().any(|p| p.destruction_markers as usize > p.score_cards.len()) {
            return Err("毁灭标志不能留在空位，也不能多于本方已赢得的地区".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit_support::{game, restore};

    #[test]
    fn destruction_stays_in_play_is_idempotent_and_round_trips() {
        let mut g = game();
        let ids = g.regions.iter().map(|r| r.card.id.clone()).collect::<Vec<_>>();
        g.destroy_all_regions();
        g.destroy_all_regions();
        assert_eq!(g.destroyed_region_count(), g.regions.len() as u32);
        assert!(g.regions.iter().all(|r| !r.vacant));
        assert_eq!(g.regions.iter().map(|r| r.card.id.clone()).collect::<Vec<_>>(), ids);
        assert!(g.view(0).regions.iter().all(|r| r.destroyed));
        restore(&mut g);
    }

    #[test]
    fn capture_transfers_one_marker_and_the_replacement_is_not_destroyed() {
        let mut g = game();
        let printed = catalog::card(&g.regions[0].card.definition).points.unwrap();
        g.regions[0].destroyed = true;
        g.effect(Effect::Score { seat: 0, region: 0 }).unwrap();
        assert_eq!(g.players[0].destruction_markers, 1);
        assert_eq!(g.view(0).players[0].score, printed - 1);
        assert_eq!(g.view(0).players[0].destruction_markers, 1);
        assert!(!g.regions[0].destroyed);
        restore(&mut g);
    }

    #[test]
    fn exhausted_world_clears_region_marker_and_restore_rejects_extra_markers() {
        let mut g = game();
        g.world.clear();
        g.regions[0].destroyed = true;
        g.effect(Effect::Score { seat: 0, region: 0 }).unwrap();
        assert!(g.regions[0].vacant);
        assert!(!g.regions[0].destroyed);
        restore(&mut g);
        g.players[0].destruction_markers = 2;
        assert!(Game::from_persisted(&serde_json::to_string(&g).unwrap()).is_err());
    }
}
