//! Face-up character play permissions share normal cost, loyalty and frames.
use crate::{
    catalog::card,
    engine::RuleResult,
    model::{Card, Game, PlaySource},
    rules,
};

impl Game {
    pub(crate) fn character_play_source(
        &self,
        actor: usize,
        id: &str,
        concealed: bool,
    ) -> RuleResult<(Card, PlaySource)> {
        if let Some(c) = self.players[actor].hand.iter().find(|c| c.id == id) {
            return Ok((c.clone(), PlaySource::Hand));
        }
        let c = self.players[actor]
            .graveyard
            .iter()
            .find(|c| c.id == id)
            .ok_or("出牌来源引用已失效")?;
        if concealed
            || card(&c.definition).kind != "character"
            || !rules::definition(&c.definition).graveyard_face_up
        {
            return Err("此墓地牌没有当前正面出牌许可，不能秘密派遣或建立资产".into());
        }
        Ok((c.clone(), PlaySource::Graveyard))
    }
    pub(crate) fn take_character_play_source(
        &mut self,
        actor: usize,
        id: &str,
        source: PlaySource,
    ) -> RuleResult<Card> {
        match source {
            PlaySource::Hand => self.remove_hand(actor, id),
            PlaySource::Graveyard => {
                let index = self.players[actor]
                    .graveyard
                    .iter()
                    .position(|c| c.id == id)
                    .ok_or("墓地出牌引用已失效")?;
                Ok(self.players[actor].graveyard.remove(index))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Action, Icons, Window};

    fn fixture_mode(mode: &str) -> Game {
        let mut g = Game::new(
            "grave-play-tests".into(),
            "QA".into(),
            mode.into(),
            "甲".into(),
            "reclaimers".into(),
            1,
        )
        .unwrap();
        g.join("乙".into(), "reclaimers".into()).unwrap();
        if mode == "teams" {
            g.join("丙".into(), "reclaimers".into()).unwrap();
            g.join("丁".into(), "reclaimers".into()).unwrap();
        }
        for p in &mut g.players {
            p.ready = true;
        }
        g.apply(0, Action::new("start")).unwrap();
        while let Some(p) = g.pending.clone() {
            g.apply(
                p.seat,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(vec![]),
                    ..Action::new("choose")
                },
            )
            .unwrap();
        }
        for p in &mut g.players {
            p.hand.clear();
            p.assets.clear();
            p.graveyard.clear();
        }
        for r in &mut g.regions {
            r.cards.clear();
        }
        g.first_team = 0;
        g.active_team = 0;
        g.priority_team = 0;
        g.window = Some(Window::Action(0));
        g.passed.clear();
        g.team_passed = [false; 2];
        g
    }
    fn fixture() -> Game {
        fixture_mode("duel")
    }
    fn assets(g: &mut Game, actor: usize, ids: &[&str]) {
        for id in ids {
            let c = g.make_card(id, actor);
            g.players[actor].assets.push(c);
        }
    }
    #[test]
    fn grave_play_pays_normal_cost_once_and_persists_its_origin_until_face_up_entry() {
        let mut g = fixture();
        assets(&mut g, 0, &["JC085", "JC084"]);
        let c = g.make_card("JC085", 0);
        let old = c.id.clone();
        g.players[0].graveyard.push(c);
        let action = Action {
            card_id: Some(old.clone()),
            region: Some(0),
            ..Action::new("deploy")
        };
        assert!(g.view(0).legal_actions.iter().any(|a| a.action == action));
        g.apply(0, action).unwrap();
        assert!(g.players[0].hand.is_empty() && g.players[0].graveyard.is_empty());
        assert_eq!(
            g.players[0].assets.iter().filter(|c| c.exhausted).count(),
            2
        );
        let frame = g.stack[0].frame.as_ref().unwrap();
        assert_eq!(frame.source.card.id, old);
        assert_eq!(frame.source.play_source, Some(PlaySource::Graveyard));
        let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
        for seat in [0, 1] {
            restored.apply(seat, Action::new("pass")).unwrap();
        }
        let entered = &restored.regions[0].cards[0];
        assert_eq!(entered.definition, "JC085");
        assert_ne!(entered.id, old);
        assert!(!entered.face_down);
        assert_eq!(restored.icons(entered, 0).influence, 1);
        assert_eq!(
            restored.players[0]
                .assets
                .iter()
                .filter(|c| c.exhausted)
                .count(),
            2
        );
    }
    #[test]
    fn late_confrontation_grave_entry_waits_for_a_standard_action_window() {
        // Public four-seat evidence: Chernobyl killed JC085 during the final
        // round's influence-after window. The game ended before another action
        // phase. This is an explicit local fixture, not a modified UI room.
        let mut g = fixture_mode("teams");
        assets(&mut g, 0, &["JC085", "JC084"]);
        let c = g.make_card("JC085", 0);
        let old = c.id.clone();
        g.players[0].graveyard.push(c);
        g.turn = 7;
        g.window = Some(Window::After(1, 2));
        let action = Action {
            card_id: Some(old.clone()),
            region: Some(2),
            ..Action::new("deploy")
        };
        assert!(!g.view(0).legal_actions.iter().any(|a| a.action == action));
        let before = serde_json::to_string(&g).unwrap();
        assert!(g.apply(0, action.clone()).is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);

        // Contrast the same permission, card and available payment in a normal
        // action window. No claim that round eight happened in the real game.
        let mut standard = g.clone();
        standard.turn = 8;
        standard.first_team = 1;
        standard.window = Some(Window::Action(0));
        assert!(standard
            .view(0)
            .legal_actions
            .iter()
            .any(|a| a.action == action));
        standard.apply(0, action.clone()).unwrap();
        assert!(standard.players[0].hand.is_empty());
        assert!(standard.players[0].graveyard.is_empty());
        assert_eq!(
            standard.stack[0].frame.as_ref().unwrap().source.play_source,
            Some(PlaySource::Graveyard)
        );
        assert_eq!(
            standard.players[0]
                .assets
                .iter()
                .filter(|c| c.exhausted)
                .count(),
            2
        );

        g.status = "finished".into();
        assert!(!g.view(0).legal_actions.iter().any(|a| a.action == action));
        let before = serde_json::to_string(&g).unwrap();
        assert!(g.apply(0, action).is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
    }
    #[test]
    fn grave_sources_reject_conceal_asset_unpermitted_foreign_loyalty_cost_and_phase_atomically() {
        let mut g = fixture();
        assets(&mut g, 0, &["JC085", "JC084"]);
        let c = g.make_card("JC085", 0);
        let id = c.id.clone();
        g.players[0].graveyard.push(c);
        for kind in ["conceal", "asset"] {
            let before = serde_json::to_string(&g).unwrap();
            assert!(g
                .apply(
                    0,
                    Action {
                        card_id: Some(id.clone()),
                        region: Some(0),
                        ..Action::new(kind)
                    }
                )
                .is_err());
            assert_eq!(serde_json::to_string(&g).unwrap(), before);
        }
        for altered in 0..5 {
            let mut bad = g.clone();
            let mut target = id.clone();
            match altered {
                0 => bad.players[0].assets[0].exhausted = true,
                1 => {
                    bad.players[0].assets.clear();
                    assets(&mut bad, 0, &["JC125", "JC125"]);
                }
                2 => {
                    let c = bad.make_card("JC084", 0);
                    target = c.id.clone();
                    bad.players[0].graveyard.push(c);
                }
                3 => {
                    let c = bad.make_card("JC085", 1);
                    target = c.id.clone();
                    bad.players[1].graveyard.push(c);
                }
                _ => bad.window = Some(Window::Prepare),
            }
            let before = serde_json::to_string(&bad).unwrap();
            assert!(bad
                .apply(
                    0,
                    Action {
                        card_id: Some(target),
                        region: Some(0),
                        ..Action::new("deploy")
                    }
                )
                .is_err());
            assert_eq!(serde_json::to_string(&bad).unwrap(), before);
        }
    }
    #[test]
    fn conditional_icons_use_controllers_assets_and_team_markers_with_first_team_and_posture() {
        let mut g = fixture();
        let street = g.make_card("JC084", 0);
        let street_id = street.id.clone();
        g.regions[0].cards.push(street);
        g.regions[0].influence = [1, 1];
        assert_eq!(
            g.icons(&g.regions[0].cards[0], 0),
            Icons {
                investigation: 1,
                combat: 1,
                influence: 1
            }
        );
        g.first_team = 1;
        assert_eq!(
            g.icons(&g.regions[0].cards[0], 0),
            Icons {
                investigation: 0,
                combat: 1,
                influence: 0
            }
        );
        g.first_team = 0;
        g.regions[0].influence = [0, 0];
        assert_eq!(g.icons(&g.regions[0].cards[0], 0).influence, 0);
        let dead = g.make_card("JC085", 0);
        g.regions[1].cards.push(dead);
        assets(&mut g, 1, &["JC085"]);
        assert_eq!(g.icons(&g.regions[1].cards[0], 1).influence, 0);
        assets(&mut g, 0, &["JC085"]);
        g.players[0].assets[0].exhausted = true;
        assert_eq!(g.icons(&g.regions[1].cards[0], 1).influence, 1);
        g.regions[1].cards[0].exhausted = true;
        assert_eq!(g.icons(&g.regions[1].cards[0], 1), Icons::default());
        g.board_mut(&street_id).unwrap().face_down = true;
        assert_eq!(
            g.icons(&g.regions[0].cards[0], 0),
            Icons {
                influence: 1,
                ..Icons::default()
            }
        );
    }
    #[test]
    fn teams_do_not_share_personal_domains_and_moved_characters_query_their_current_region() {
        let mut g = fixture_mode("teams");
        let dead = g.make_card("JC085", 0);
        g.regions[0].cards.push(dead);
        assets(&mut g, 1, &["JC085"]);
        assert_eq!(g.icons(&g.regions[0].cards[0], 0).influence, 0);
        g.regions[0].cards[0].controller = 1;
        assert_eq!(g.icons(&g.regions[0].cards[0], 0).influence, 1);
        g.regions[0].cards[0].controller = 0;
        assets(&mut g, 0, &["JC085"]);
        g.players[0].assets[0].exhausted = true;
        assert_eq!(g.icons(&g.regions[0].cards[0], 0).influence, 1);
        let street = g.make_card("JC084", 1);
        g.regions[0].cards.push(street);
        g.regions[0].influence = [1, 1];
        assert_eq!(g.icons(&g.regions[0].cards[1], 0).investigation, 1);
        let moved = g.regions[0].cards.pop().unwrap();
        g.regions[1].cards.push(moved);
        assert_eq!(g.icons(&g.regions[1].cards[0], 1).influence, 0);
        assert_eq!(g.icons(&g.regions[1].cards[0], 1).investigation, 0);
        g.regions[1].influence = [1, 1];
        assert_eq!(g.icons(&g.regions[1].cards[0], 1).influence, 1);
        g.first_team = 1;
        assert_eq!(g.icons(&g.regions[1].cards[0], 1).influence, 0);
    }
}
