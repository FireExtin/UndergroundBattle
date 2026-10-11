//! Resumable operations used by the ten printed base-world regions.
use crate::{catalog::card, engine::RuleResult, model::*, rules::CardFilter};
use std::collections::BTreeMap;

impl Game {
    pub(crate) fn world_free_reveal_choice(
        &mut self,
        frame: ResolutionFrame,
        seat: usize,
        require_loyalty: bool,
    ) -> bool {
        let options = self
            .regions
            .iter()
            .enumerate()
            .flat_map(|(r, reg)| reg.cards.iter().map(move |c| (r, c)))
            .filter(|(_, c)| {
                c.controller == seat
                    && c.face_down
                    && (!require_loyalty || self.loyalty(seat, &c.definition))
            })
            .map(|(r, c)| self.option(c, seat, Some(r), None))
            .collect::<Vec<_>>();
        if options.is_empty() {
            return false;
        }
        self.choice(
            seat,
            "target",
            "可选择一个自己操控的暗藏者免费翻面".into(),
            options,
            0,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(frame),
                choice: FrameChoice::FreeReveal {
                    seat,
                    require_loyalty,
                },
            },
        );
        true
    }
    pub(crate) fn world_reveal(
        &mut self,
        seat: usize,
        id: &str,
        require_loyalty: bool,
    ) -> RuleResult<()> {
        let (_, c) = self
            .board(id)
            .filter(|(_, c)| c.controller == seat && c.face_down)
            .ok_or("免费翻面角色已失效")?;
        if require_loyalty && !self.loyalty(seat, &c.definition) {
            return Err("忠诚不足".into());
        }
        let (region, c) = self.remove_board(id).ok_or("暗藏者已离场")?;
        let mut c = self.fresh(c);
        c.face_down = false;
        c.damage = 0;
        c.wounds = 0;
        c.shield = 0;
        let id = c.id.clone();
        let definition = c.definition.clone();
        self.regions[region].cards.push(c);
        self.note(format!(
            "{} 通过地区效果免费翻面 {}",
            self.players[seat].name,
            card(&definition).name
        ));
        self.enter_triggers(seat, &definition, &id, true);
        Ok(())
    }
    pub(crate) fn world_sacrifice_choice(&mut self, frame: ResolutionFrame, seat: usize) -> bool {
        let options = self
            .regions
            .iter()
            .enumerate()
            .flat_map(|(r, reg)| reg.cards.iter().map(move |c| (r, c)))
            .filter(|(_, c)| {
                c.controller == seat && !c.face_down && card(&c.definition).kind == "character"
            })
            .map(|(r, c)| self.option(c, seat, Some(r), None))
            .collect::<Vec<_>>();
        if options.is_empty() {
            return false;
        }
        self.choice(
            seat,
            "target",
            "牺牲一个角色，再按其离场前防御力抓牌".into(),
            options,
            1,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(frame),
                choice: FrameChoice::SacrificeDraw { seat },
            },
        );
        true
    }
    pub(crate) fn world_region_choice(&mut self, frame: ResolutionFrame) {
        let options = self
            .regions
            .iter()
            .enumerate()
            .filter(|(_, reg)| !reg.vacant)
            .map(|(r, reg)| ChoiceOption {
                id: format!("region:{r}"),
                label: format!("地区{}：{}", r + 1, card(&reg.card.definition).name),
                card: None,
            })
            .collect::<Vec<_>>();
        if options.is_empty() { return; }
        self.choice(
            frame.actor,
            "target",
            if frame.source.card.definition == "JC050" { "选择要消灭其中角色与暗藏者的地区".into() }
            else { "选择墓地角色进入的地区".into() },
            options,
            1,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(frame),
                choice: FrameChoice::Region,
            },
        );
    }
    pub(crate) fn world_graveyard_choice(
        &mut self,
        frame: ResolutionFrame,
        seat: usize,
        region: usize,
    ) -> bool {
        let options = self.players[seat]
            .graveyard
            .iter()
            .filter(|c| card(&c.definition).kind == "character")
            .map(|c| self.option(c, seat, None, None))
            .collect::<Vec<_>>();
        if options.is_empty() {
            return false;
        }
        self.choice(
            seat,
            "target",
            format!("从自己的墓地选择一个角色进入地区{}", region + 1),
            options,
            1,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(frame),
                choice: FrameChoice::GraveyardEntry { seat, region },
            },
        );
        true
    }
    pub(crate) fn world_graveyard_entry(
        &mut self,
        seat: usize,
        region: usize,
        id: &str,
    ) -> RuleResult<()> {
        if !self.region_live(region) {
            return Err("墓地进场地区已失效".into());
        }
        let index = self.players[seat]
            .graveyard
            .iter()
            .position(|c| c.id == id && card(&c.definition).kind == "character")
            .ok_or("墓地角色已失效")?;
        let c = self.players[seat].graveyard.remove(index);
        let c = self.reset_zone_card(c);
        let id = c.id.clone();
        let definition = c.definition.clone();
        self.regions[region].cards.push(c);
        self.note(format!(
            "{} 的 {} 从墓地进入地区{}",
            self.players[seat].name,
            card(&definition).name,
            region + 1
        ));
        self.enter_triggers(seat, &definition, &id, false);
        Ok(())
    }
    pub(crate) fn world_hide(&mut self, except_subtype: Option<&str>, mix_hidden: bool) {
        let selected = self
            .regions
            .iter()
            .flat_map(|reg| reg.cards.iter())
            .filter(|c| {
                !c.face_down
                    && card(&c.definition).kind == "character"
                    && except_subtype.is_none_or(|subtype| {
                        !self.current_subtypes(c).iter().any(|s| s == subtype)
                    })
            })
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        for id in selected {
            let controller = self.controller_after_reset(&id);
            if let Some((r, c)) = self.leave_board(&id) {
                let mut c = self.fresh(c);
                if let Some(controller) = controller {
                    c.controller = controller;
                }
                c.face_down = true;
                c.damage = 0;
                c.wounds = 0;
                c.shield = 0;
                self.regions[r].cards.push(c);
            }
        }
        if mix_hidden {
            for r in 0..self.regions.len() {
                let mut groups = BTreeMap::<usize, Vec<Card>>::new();
                let all = std::mem::take(&mut self.regions[r].cards);
                for c in all {
                    if c.face_down {
                        groups.entry(c.controller).or_default().push(c);
                    } else {
                        self.regions[r].cards.push(c);
                    }
                }
                for (_, mut cards) in groups {
                    if cards.len() > 1 {
                        self.shuffle(&mut cards);
                        for c in cards {
                            let c = self.fresh(c);
                            self.regions[r].cards.push(c);
                        }
                    } else {
                        self.regions[r].cards.extend(cards);
                    }
                }
            }
        }
    }
    pub(crate) fn world_search_next(
        &mut self,
        frame: ResolutionFrame,
        filter: CardFilter,
        participants: Vec<usize>,
        mut committed: Vec<(usize, Option<String>)>,
    ) -> RuleResult<()> {
        while let Some(&seat) = participants.get(committed.len()) {
            let options = self.players[seat]
                .deck
                .iter()
                .filter(|c| self.filter_card(&filter, c))
                .map(|c| self.option(c, seat, None, None))
                .collect::<Vec<_>>();
            if options.is_empty() {
                committed.push((seat, None));
                continue;
            }
            self.choice(
                seat,
                "search",
                "私密选择检索牌；所有玩家选择后同时展示并入手".into(),
                options,
                0,
                1,
                None,
                ChoiceResolution::Frame {
                    frame: Box::new(frame),
                    choice: FrameChoice::SimultaneousSearch {
                        filter,
                        participants,
                        committed,
                    },
                },
            );
            return Ok(());
        }
        // Every commitment remains private until all players have chosen. No deck
        // is changed or shuffled while another player's choice is pending.
        let mut found = Vec::new();
        for (seat, selected) in &committed {
            if let Some(id) = selected {
                let index = self.players[*seat]
                    .deck
                    .iter()
                    .position(|c| c.id == *id && self.filter_card(&filter, c))
                    .ok_or("同时检索的原牌已失效")?;
                let c = self.players[*seat].deck.remove(index);
                found.push((*seat, c));
            }
        }
        for (seat, c) in &found {
            self.note(format!(
                "{} 同时展示检索的 {}",
                self.players[*seat].name,
                card(&c.definition).name
            ));
        }
        for (seat, c) in found {
            let c = self.fresh(c);
            self.players[seat].hand.push(c);
        }
        for seat in participants {
            self.shuffle_player(seat);
        }
        self.effects.push_front(Effect::Frame {
            frame: Box::new(frame),
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog,
        rules::{self, Op},
    };
    use std::collections::BTreeSet;

    fn fixture(mode: &str, seed: u64) -> Game {
        let mut g = Game::new(
            "world-test".into(),
            "WORLD".into(),
            mode.into(),
            "P0".into(),
            "responders".into(),
            seed,
        )
        .unwrap();
        for seat in 1..g.capacity() {
            g.join(format!("P{seat}"), "responders".into()).unwrap();
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
        g.window = Some(Window::Action(0));
        g.priority_team = 0;
        g.active_team = 0;
        g.passed.clear();
        g.team_passed = [false; 2];
        for p in &mut g.players {
            p.hand.clear();
            p.assets.clear();
            p.graveyard.clear();
        }
        for reg in &mut g.regions {
            reg.cards.clear();
        }
        g
    }
    fn board(g: &mut Game, id: &str, seat: usize, region: usize, hidden: bool) -> String {
        let mut c = g.make_card(id, seat);
        c.face_down = hidden;
        let id = c.id.clone();
        g.regions[region].cards.push(c);
        id
    }
    fn program(g: &mut Game, id: &str, actor: usize, ops: Option<Vec<Op>>) {
        let c = g.make_card(id, actor);
        let source = g.source_snapshot(&c, Some(0));
        let mut spec = rules::definition(id).abilities[0].clone();
        if let Some(ops) = ops {
            spec.ops = ops;
        }
        let frame = g.make_frame(actor, source, &spec, vec![], vec![], None);
        g.resolve_frame(frame).unwrap();
        g.drive().unwrap();
    }
    fn select(g: &mut Game, selected: Vec<String>) {
        let p = g.pending.clone().unwrap();
        g.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(selected),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    fn restored(g: &Game) -> Game {
        serde_json::from_str(&serde_json::to_string(g).unwrap()).unwrap()
    }

    #[test]
    fn base_world_rejects_duplicate_fillers_missing_cards_and_unregistered_regions() {
        let c = catalog::catalog();
        assert!(catalog::validate_base_world(&c.world, &c.cards).is_ok());
        let mut repeated = c.world.clone();
        repeated[2].count = 2;
        assert!(catalog::validate_base_world(&repeated, &c.cards).is_err());
        repeated[2] = repeated[1].clone();
        assert!(catalog::validate_base_world(&repeated, &c.cards).is_err());
        assert!(catalog::validate_base_world(&c.world[..9], &c.cards).is_err());
        repeated[2].card_id = "DQunknown".into();
        assert!(catalog::validate_base_world(&repeated, &c.cards).is_err());
        let mut names = c.cards.clone();
        let second = names.iter().position(|d| d.id == "DQJC108").unwrap();
        names[second].name = catalog::card("DQJC107").name.clone();
        assert!(catalog::validate_base_world(&c.world, &names).is_err());
    }
    #[test]
    fn duel_and_teams_start_with_ten_distinct_physical_regions_across_seeds() {
        for mode in ["duel", "teams"] {
            for seed in [1, 2, 3, 7, 17, 42, 99, u64::MAX] {
                let g = fixture(mode, seed);
                assert_eq!(g.regions.len(), if mode == "duel" { 3 } else { 5 });
                let all = g
                    .regions
                    .iter()
                    .map(|r| &r.card)
                    .chain(g.world.iter())
                    .collect::<Vec<_>>();
                assert_eq!(all.len(), 10);
                assert_eq!(all.iter().map(|c| &c.id).collect::<BTreeSet<_>>().len(), 10);
                assert_eq!(
                    all.iter()
                        .map(|c| &c.definition)
                        .collect::<BTreeSet<_>>()
                        .len(),
                    10
                );
                assert_eq!(
                    all.iter()
                        .map(|c| &catalog::card(&c.definition).name)
                        .collect::<BTreeSet<_>>()
                        .len(),
                    10
                );
            }
        }
    }
    #[test]
    fn world_values_match_the_ten_printed_originals() {
        for (id, name, threshold, points) in [
            ("DQJC107", "沉没的废墟", 4, 4),
            ("DQJC108", "佛罗伦萨", 4, 3),
            ("DQJC109", "京都", 3, 3),
            ("DQJC110", "伦敦", 4, 4),
            ("DQJC111", "莫斯科", 4, 3),
            ("DQJC112", "纽约", 4, 4),
            ("DQJC113", "切尔诺贝利", 3, 2),
            ("DQJC114", "上海", 4, 3),
            ("DQJC115", "死者之城", 3, 2),
            ("DQJC116", "香港", 3, 3),
        ] {
            let d = catalog::card(id);
            assert_eq!(d.name, name);
            assert_eq!(d.threshold, Some(threshold));
            assert_eq!(d.points, Some(points));
            assert_eq!(rules::definition(id).abilities.len(), 1);
        }
    }
    #[test]
    fn printed_zero_cost_needs_no_assets_but_keeps_standard_timing_and_team_distance() {
        let d = catalog::card("JC125");
        assert_eq!(d.cost, 0);
        assert!(d.loyalty.is_empty());
        assert_eq!(d.defense, Some(1));
        assert_eq!(d.permanent_icons, Icons::default());
        assert_eq!(d.temporary_icons, Icons::default());
        let mut g = fixture("teams", 17);
        let c = g.make_card("JC125", 0);
        let id = c.id.clone();
        g.players[0].hand.push(c);
        let action = Action {
            card_id: Some(id.clone()),
            region: Some(0),
            ..Action::new("deploy")
        };
        for region in [3, 4] {
            let before = serde_json::to_string(&g).unwrap();
            assert!(g
                .apply(
                    0,
                    Action {
                        region: Some(region),
                        ..action.clone()
                    }
                )
                .is_err());
            assert_eq!(serde_json::to_string(&g).unwrap(), before);
        }
        g.window = Some(Window::Before(0, 0));
        let before = serde_json::to_string(&g).unwrap();
        assert!(g.apply(0, action.clone()).is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        g.window = Some(Window::Action(0));
        assert!(g
            .legal_actions(0)
            .iter()
            .any(|a| a.action.kind == "deploy" && a.action.card_id.as_deref() == Some(&id)));
        g.apply(0, action).unwrap();
        assert_eq!(g.stack.len(), 1);
        assert_eq!(g.stack[0].card.as_ref().unwrap().definition, "JC125");
        assert!(g.players[0].assets.is_empty());
        assert!(g.players[0].hand.is_empty());
        assert!(g.regions[0].cards.is_empty());
    }
    #[test]
    fn florence_rotates_independent_players_and_free_reveal_ignores_loyalty_under_adopted_faq() {
        let mut g = fixture("teams", 17);
        for seat in 0..4 {
            board(&mut g, "JC003", seat, 2, true);
        }
        assert!((0..4).all(|s| !g.loyalty(s, "JC003")));
        program(&mut g, "DQJC108", 2, None);
        for seat in [2, 3, 0, 1] {
            assert_eq!(g.pending.as_ref().unwrap().seat, seat);
            let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
            assert!(g.view((seat + 1) % 4).pending_choice.is_none());
            g = restored(&g);
            select(&mut g, vec![id.clone()]);
            assert!(g.regions.iter().flat_map(|r| &r.cards).all(|c| c.id != id));
        }
        assert!(g.pending.is_none());
        assert_eq!(
            g.regions
                .iter()
                .flat_map(|r| &r.cards)
                .filter(|c| !c.face_down)
                .count(),
            4
        );
        assert!(g.players.iter().all(|p| p.assets.is_empty()));
    }
    #[test]
    fn florence_can_decline_and_does_not_change_ordinary_reveal_loyalty() {
        let mut g = fixture("duel", 17);
        let id = board(&mut g, "JC003", 0, 0, true);
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(id.clone()),
                    ..Action::new("reveal")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        program(&mut g, "DQJC108", 0, None);
        select(&mut g, vec![]);
        assert_eq!(g.board(&id).unwrap().1.face_down, true);
    }
    #[test]
    fn kyoto_hides_only_nonhuman_face_up_characters_and_preserves_exhaustion() {
        let mut g = fixture("duel", 17);
        let human = board(&mut g, "JC125", 0, 0, false);
        let nonhuman = board(&mut g, "XQ12", 1, 1, false);
        let hidden = board(&mut g, "JC003", 1, 1, true);
        let c = g.board_mut(&nonhuman).unwrap();
        c.exhausted = true;
        c.damage = 1;
        c.wounds = 1;
        c.shield = 1;
        program(&mut g, "DQJC109", 0, None);
        assert!(!g.board(&human).unwrap().1.face_down);
        assert!(g.board(&hidden).unwrap().1.face_down);
        assert!(g.board(&nonhuman).is_none());
        let z = g.regions[1]
            .cards
            .iter()
            .find(|c| c.definition == "XQ12")
            .unwrap();
        assert!(z.face_down);
        assert!(z.exhausted);
        assert_eq!((z.damage, z.wounds, z.shield), (0, 0, 0));
        let opponent = g.view(0).regions[1]
            .characters
            .iter()
            .find(|c| c.instance_id == z.id)
            .unwrap()
            .clone();
        assert_eq!(opponent.name, "暗藏者");
        assert!(opponent.card_id.is_none());
        assert!(opponent.text.is_none());
    }
    #[test]
    fn london_mixes_existing_hidden_with_newly_hidden_per_controller_and_region_deterministically()
    {
        let mut g = fixture("teams", 17);
        let old = (0..4)
            .map(|s| board(&mut g, "JC125", s, 2, false))
            .collect::<Vec<_>>();
        let extra = board(&mut g, "JC003", 0, 2, true);
        let mut replay = restored(&g);
        program(&mut g, "DQJC110", 0, None);
        program(&mut replay, "DQJC110", 0, None);
        assert_eq!(
            serde_json::to_string(&g).unwrap(),
            serde_json::to_string(&replay).unwrap()
        );
        assert_eq!(g.regions[2].cards.len(), 5);
        assert!(g.regions[2]
            .cards
            .iter()
            .all(|c| c.face_down && !old.contains(&c.id) && c.id != extra));
        for seat in 0..4 {
            assert_eq!(
                g.regions[2]
                    .cards
                    .iter()
                    .filter(|c| c.controller == seat)
                    .count(),
                if seat == 0 { 2 } else { 1 }
            );
        }
        assert!(g.view(2).regions[2]
            .characters
            .iter()
            .filter(|c| c.controller != "p2")
            .all(|c| c.card_id.is_none()));
    }
    #[test]
    fn moscow_resumes_seat_order_and_uses_the_pre_sacrifice_defense_snapshot() {
        let mut g = fixture("teams", 17);
        for seat in 0..4 {
            board(&mut g, "JC125", seat, 2, false);
        }
        board(&mut g, "JC059", 2, 2, false);
        let hands = g.players.iter().map(|p| p.hand.len()).collect::<Vec<_>>();
        program(&mut g, "DQJC111", 2, None);
        for seat in [2, 3, 0, 1] {
            assert_eq!(g.pending.as_ref().unwrap().seat, seat);
            let id = g
                .pending
                .as_ref()
                .unwrap()
                .choice
                .options
                .iter()
                .find(|o| o.card.as_ref().unwrap().card_id.as_deref() == Some("JC125"))
                .unwrap()
                .id
                .clone();
            g = restored(&g);
            select(&mut g, vec![id]);
            assert_eq!(
                g.players[seat].hand.len(),
                hands[seat] + if seat >= 2 { 2 } else { 1 }
            );
            assert!(g.players[seat]
                .graveyard
                .iter()
                .any(|c| c.definition == "JC125"));
        }
        assert!(g.pending.is_none());
    }
    #[test]
    fn dead_city_retains_chosen_region_and_each_players_graveyard_choice_after_restore() {
        let mut g = fixture("teams", 17);
        for seat in 0..4 {
            let mut c = g.make_card("JC125", seat);
            c.exhausted = true;
            c.damage = 7;
            g.players[seat].graveyard.push(c);
        }
        program(&mut g, "DQJC115", 1, None);
        assert_eq!(g.pending.as_ref().unwrap().seat, 1);
        select(&mut g, vec!["region:4".into()]);
        for seat in [1, 2, 3, 0] {
            assert_eq!(g.pending.as_ref().unwrap().seat, seat);
            g = restored(&g);
            let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
            select(&mut g, vec![id.clone()]);
            assert!(g.players[seat].graveyard.is_empty());
            let c = g.regions[4].cards.iter().find(|c| c.owner == seat).unwrap();
            assert_ne!(c.id, id);
            assert!(!c.face_down);
            assert!(!c.exhausted);
            assert_eq!(c.damage, 0);
        }
        assert!(g.pending.is_none());
        assert_eq!(g.regions[4].cards.len(), 4);
    }
    #[test]
    fn simultaneous_search_keeps_commitments_private_and_decks_unchanged_until_all_players_choose()
    {
        let mut g = fixture("teams", 17);
        for seat in 0..4 {
            g.players[seat].deck.clear();
            for id in ["JC003", "JC125"] {
                let c = g.make_card(id, seat);
                g.players[seat].deck.push(c);
            }
        }
        let initial_decks = g
            .players
            .iter()
            .map(|p| p.deck.iter().map(|c| c.id.clone()).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let rng = g.random;
        let log_len = g.log.len();
        // Nonempty shared-operation fixture uses registered characters. The exact
        // Hong Kong binding is separately checked to filter attachments.
        program(
            &mut g,
            "DQJC116",
            0,
            Some(vec![Op::SimultaneousSearch {
                filter: CardFilter::Kind("character".into()),
            }]),
        );
        for seat in 0..4 {
            assert_eq!(g.pending.as_ref().unwrap().seat, seat);
            assert_eq!(g.random, rng);
            assert_eq!(g.log.len(), log_len);
            assert!(g.players.iter().all(|p| p.hand.is_empty()));
            assert_eq!(
                g.players
                    .iter()
                    .map(|p| p.deck.iter().map(|c| c.id.clone()).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
                initial_decks
            );
            let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
            g = restored(&g);
            select(&mut g, if seat == 1 { vec![] } else { vec![id] });
        }
        assert!(g.pending.is_none());
        assert_eq!(
            g.players.iter().map(|p| p.hand.len()).collect::<Vec<_>>(),
            vec![1, 0, 1, 1]
        );
        assert_eq!(
            g.log
                .iter()
                .filter(|l| l.text.contains("同时展示检索"))
                .count(),
            3
        );
        assert_eq!(
            g.players.iter().map(|p| p.deck.len()).collect::<Vec<_>>(),
            vec![1, 2, 1, 1]
        );
    }
    #[test]
    fn hong_kong_exact_binding_is_attachment_only_and_empty_search_does_not_draw() {
        assert!(
            matches!(rules::definition("DQJC116").abilities[0].ops.as_slice(), [Op::SimultaneousSearch { filter: CardFilter::Kind(kind) }] if kind == "attachment")
        );
        let mut g = fixture("duel", 17);
        let hands = g.players.iter().map(|p| p.hand.len()).collect::<Vec<_>>();
        let sizes = g.players.iter().map(|p| p.deck.len()).collect::<Vec<_>>();
        program(&mut g, "DQJC116", 0, None);
        assert!(g.pending.is_none());
        assert_eq!(
            g.players.iter().map(|p| p.hand.len()).collect::<Vec<_>>(),
            hands
        );
        assert_eq!(
            g.players.iter().map(|p| p.deck.len()).collect::<Vec<_>>(),
            sizes
        );
    }
}
