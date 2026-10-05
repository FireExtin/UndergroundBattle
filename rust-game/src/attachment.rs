//! In-play attachments share target guards and zone identities with other cards.
//! Host departure is a semantic operation: moving between regions is not departure.
use crate::{
    catalog::card,
    model::*,
    rules::{self, EntityKind, HostLeaveDestination, Relation},
};

impl Game {
    pub(crate) fn reattach_source(&mut self, source_id: &str, target_id: &str) {
        let Some(index) = self.attachments.iter().position(|a| a.card.id == source_id) else {
            self.note("转移结附的来源已离场，效果不再改变场上对象".into());
            return;
        };
        self.attachments[index].host_id = target_id.into();
        self.note(format!(
            "{} 转移结附",
            card(&self.attachments[index].card.definition).name
        ));
        self.settle_deaths();
    }
    pub(crate) fn prepare_region_return(&mut self, region: usize) -> Result<(), String> {
        if self.region_return.is_some() || region >= self.regions.len() {
            return Err("赢区回底批次无效".into());
        }
        let mut batch = RegionReturnBatch {
            region,
            hand: vec![],
            bottom: vec![vec![]; self.players.len()],
            orders: vec![None; self.players.len()],
        };
        // BQ PDF10: every in-play character, hidden card and attachment in
        // this region has a destination before any host is removed.
        for (r, c) in self.in_play_cards() {
            if r != region {
                continue;
            }
            if !c.face_down && rules::definition(&c.definition).traits.retreat {
                batch.hand.push(c.id.clone());
            } else {
                batch.bottom[c.owner].push(c.id.clone());
            }
        }
        self.region_return = Some(batch);
        Ok(())
    }
    pub(crate) fn commit_region_return(&mut self, region: usize) -> Result<(), String> {
        let batch = self.region_return.take().ok_or("缺少赢区回底批次")?;
        if batch.region != region || batch.orders.iter().any(Option::is_none) {
            return Err("赢区回底排序未完成".into());
        }
        let ids = batch
            .hand
            .iter()
            .chain(batch.bottom.iter().flatten())
            .cloned()
            .collect::<Vec<_>>();
        if ids
            .iter()
            .any(|id| self.board(id).is_none_or(|(r, _)| r != region))
        {
            return Err("赢区回底对象已失效".into());
        }
        // Remove attachments before their hosts while retaining every object in
        // this explicit batch. No host-departure cleanup can change destinations.
        let mut removing = ids.clone();
        removing.sort_by_key(|id| !self.attachments.iter().any(|a| a.card.id == *id));
        let mut cards = std::collections::BTreeMap::new();
        for id in removing {
            let (_, c) = self.remove_board(&id).ok_or("赢区回底对象已失效")?;
            cards.insert(id, c);
        }
        for id in batch.hand {
            let c = self.reset_zone_card(cards.remove(&id).ok_or("撤回对象已失效")?);
            self.players[c.owner].hand.push(c);
        }
        for (owner, order) in batch.orders.into_iter().enumerate() {
            for id in order.unwrap() {
                let c = self.reset_zone_card(cards.remove(&id).ok_or("回底对象重复或已失效")?);
                self.players[owner].deck.push(c);
            }
        }
        Ok(())
    }
    pub(crate) fn attachment_host_valid(&self, attachment: &Attachment) -> bool {
        let Some(spec) = &rules::definition(&attachment.card.definition).attachment else {
            return false;
        };
        let Some((_, host)) = self.board(&attachment.host_id) else {
            return false;
        };
        let d = card(&host.definition);
        // Remaining attached is a continuous condition, not a new targeting event:
        // barriers and shields never remove an already attached card.
        let kind = match spec.host.kind {
            EntityKind::Character => !host.face_down && d.kind == "character",
            _ => false,
        };
        let relation = match spec.host.relation {
            Relation::Any => true,
            Relation::ControlledByActor => host.controller == attachment.card.controller,
            Relation::OwnedByActor => host.owner == attachment.card.controller,
            Relation::FriendlyTeam => !self.is_enemy(attachment.card.controller, host),
            Relation::EnemyTeam => self.is_enemy(attachment.card.controller, host),
        };
        kind && relation
            && (!card(&attachment.card.definition)
                .subtypes
                .iter()
                .any(|s| s == "装备")
                || !rules::definition(&host.definition)
                    .traits
                    .cannot_be_equipped)
            && spec
                .host
                .subtype
                .as_ref()
                .is_none_or(|s| self.target_subtypes(host, &spec.host).contains(s))
            && (spec.host.subtypes_any.is_empty()
                || spec
                    .host
                    .subtypes_any
                    .iter()
                    .any(|s| self.target_subtypes(host, &spec.host).contains(s)))
    }
    pub(crate) fn host_leaves(&mut self, host_id: &str) {
        let all = std::mem::take(&mut self.attachments);
        let (departing, staying): (Vec<_>, Vec<_>) = all
            .into_iter()
            .partition(|attachment| attachment.host_id == host_id);
        self.attachments = staying;
        for attachment in departing {
            let owner = attachment.card.owner;
            let name = card(&attachment.card.definition).name.clone();
            let destination = rules::definition(&attachment.card.definition)
                .attachment
                .as_ref()
                .map_or(HostLeaveDestination::OwnerGraveyard, |s| s.host_leaves);
            let c = self.reset_zone_card(attachment.card);
            if !self.players[owner].eliminated {
                match destination {
                    HostLeaveDestination::OwnerHand => {
                        self.players[owner].hand.push(c);
                        self.note(format!("{name}：宿主离场，回收至拥有者手牌"));
                    }
                    HostLeaveDestination::OwnerGraveyard => {
                        self.players[owner].graveyard.push(c);
                        self.note(format!("{name}：宿主离场，附属进入拥有者墓地"));
                    }
                }
            }
        }
    }
    pub(crate) fn leave_board(&mut self, id: &str) -> Option<(usize, Card)> {
        if self.board(id).is_some() {
            self.host_leaves(id);
        }
        let removed = self.remove_board(id);
        // Removed snapshots retain their last controller for death triggers.
        // The old target/source identity cannot carry its effects into a new one.
        self.settle_controls();
        removed
    }
    pub(crate) fn settle_attachments(&mut self) {
        let invalid = self
            .attachments
            .iter()
            .filter(|a| !self.attachment_host_valid(a))
            .map(|a| a.card.id.clone())
            .collect::<Vec<_>>();
        for id in invalid {
            if let Some(index) = self.attachments.iter().position(|a| a.card.id == id) {
                // A host losing its required type is not departure. Neither this
                // cleanup nor destroying an attachment itself invokes recycling.
                let attachment = self.attachments.remove(index);
                let owner = attachment.card.owner;
                self.note(format!(
                    "{}：结附条件不再满足，附属进入拥有者墓地",
                    card(&attachment.card.definition).name
                ));
                let c = self.reset_zone_card(attachment.card);
                if !self.players[owner].eliminated {
                    self.players[owner].graveyard.push(c);
                }
            }
        }
    }
    pub(crate) fn in_play_cards(&self) -> Vec<(usize, &Card)> {
        let mut result = self
            .regions
            .iter()
            .enumerate()
            .flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .collect::<Vec<_>>();
        for attachment in &self.attachments {
            if let Some((r, _)) = self.board(&attachment.host_id) {
                result.push((r, &attachment.card));
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{AbilitySpec, EntityRef, Op, RegionRef, ResponsePolicy, Timing};

    // Explicit initial layouts isolate rules. After setup, ordinary card scenarios
    // below use real Actions, never administration commands or live room edits.
    fn fixture(mode: &str) -> Game {
        let mut g = Game::new(
            "attachment-test".into(),
            "TEST".into(),
            mode.into(),
            "P0".into(),
            "hunters".into(),
            8726,
        )
        .unwrap();
        for s in 1..g.capacity() {
            g.join(format!("P{s}"), "hunters".into()).unwrap();
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
        }
        for r in &mut g.regions {
            r.cards.clear();
        }
        g.first_team = 0;
        g.begin_window(Window::Action(0));
        g
    }
    fn field(g: &mut Game, definition: &str, owner: usize, r: usize, hidden: bool) -> String {
        let mut c = g.make_card(definition, owner);
        c.face_down = hidden;
        let id = c.id.clone();
        g.regions[r].cards.push(c);
        id
    }
    fn hand(g: &mut Game, definition: &str, owner: usize) -> String {
        let c = g.make_card(definition, owner);
        let id = c.id.clone();
        g.players[owner].hand.push(c);
        id
    }
    fn assets(g: &mut Game, owner: usize, definition: &str, n: usize) {
        for _ in 0..n {
            let c = g.make_card(definition, owner);
            g.players[owner].assets.push(c);
        }
    }
    fn attach(g: &mut Game, actor: usize, host: &str) {
        assets(g, actor, "JC125", 1);
        let id = hand(g, "BQ022", actor);
        g.apply(
            actor,
            Action {
                card_id: Some(id),
                target_id: Some(host.into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        resolve_top(g);
    }
    fn resolve_top(g: &mut Game) {
        let n = g.stack.len();
        assert!(n > 0);
        while g.stack.len() >= n && g.pending.is_none() {
            let seat = (0..g.players.len())
                .find(|s| g.team(*s) == g.priority_team && !g.passed.contains(s))
                .unwrap();
            g.apply(seat, Action::new("pass")).unwrap();
        }
    }
    fn primitive(g: &mut Game, source: &str, ops: Vec<Op>) {
        // A primitive contract fixture, not a claim that BQ022 grants these actions.
        let (r, c) = g.board(source).unwrap();
        let snapshot = g.source_snapshot(c, Some(r));
        let spec = AbilitySpec {
            play_only: false,
            activation_only: false,
            key: "lifecycle-fixture".into(),
            label: "primitive fixture".into(),
            timing: Timing::Fast,
            response_policy: ResponsePolicy::Respondable,
            costs: vec![],
            targets: vec![],
            ops,
            event: None,
            modes: vec![],
            requires_ready_source: false,
            once_per_game: false,
            per_turn_limit: None,
        };
        let frame = g.make_frame(c.controller, snapshot, &spec, vec![], vec![], None);
        g.resolve_frame(frame).unwrap();
        g.settle_deaths();
    }
    #[test]
    fn attachment_targets_public_humans_or_vampires_any_team_any_region_and_pays_atomically() {
        let mut g = fixture("teams");
        let hosts = [
            field(&mut g, "LC22", 0, 0, false),
            field(&mut g, "LC22", 1, 4, false),
            field(&mut g, "LC22", 2, 3, false),
            field(&mut g, "XQ12", 3, 4, false),
        ];
        let hidden = field(&mut g, "LC22", 2, 1, true);
        let barrier = field(&mut g, "JZ08", 2, 1, false);
        let spell = field(&mut g, "XQ03", 2, 1, false); // malformed board fixture, still never a character target
        let equipment = hand(&mut g, "BQ022", 0);
        assets(&mut g, 0, "JC125", 1);
        let candidates = g
            .legal_actions(0)
            .into_iter()
            .filter(|a| a.action.kind == "play" && a.action.card_id.as_deref() == Some(&equipment))
            .collect::<Vec<_>>();
        assert_eq!(candidates.len(), 4);
        for host in &hosts {
            assert!(candidates
                .iter()
                .any(|a| a.action.target_id.as_ref() == Some(host)));
        }
        for invalid in [hidden, barrier, spell] {
            let before = serde_json::to_string(&g).unwrap();
            assert!(g
                .apply(
                    0,
                    Action {
                        card_id: Some(equipment.clone()),
                        target_id: Some(invalid),
                        ..Action::new("play")
                    }
                )
                .is_err());
            assert_eq!(serde_json::to_string(&g).unwrap(), before);
        }
        g.apply(
            0,
            Action {
                card_id: Some(equipment.clone()),
                target_id: Some(hosts[1].clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        assert_eq!(g.resources(0), 0);
        assert!(g.attachments.is_empty());
        assert!(
            g.stack
                .last()
                .unwrap()
                .frame
                .as_ref()
                .unwrap()
                .already_paid
                .len()
                == 1
        );
        resolve_top(&mut g);
        assert_eq!(g.attachments.len(), 1);
        let a = &g.attachments[0];
        assert_eq!(a.host_id, hosts[1]);
        assert_eq!(a.card.owner, 0);
        assert_eq!(a.card.controller, 0);
        assert_ne!(a.card.id, equipment);
        assert!(g.players[0]
            .graveyard
            .iter()
            .all(|c| c.definition != "BQ022"));
        assert!(g
            .regions
            .iter()
            .flat_map(|r| r.cards.iter())
            .all(|c| c.definition != "BQ022"));
        for viewer in 0..4 {
            let v = g.view(viewer);
            let a = &v.attachments[0];
            assert_eq!(a.card.kind, "attachment");
            assert!(!a.card.face_down);
            assert_eq!(a.card.region, Some(4));
            assert_eq!(a.card.card_id.as_deref(), Some("BQ022"));
            assert_eq!(a.card.owner, "p0");
            assert_eq!(
                v.regions[4].icons_by_team,
                g.view(0).regions[4].icons_by_team
            );
            assert!(
                v.regions[1]
                    .characters
                    .iter()
                    .find(|c| c.face_down)
                    .unwrap()
                    .card_id
                    .is_none()
                    || viewer == 2
            );
        }
    }
    #[test]
    fn multiple_equipment_stacks_only_on_host_and_exhausted_host_contributes_zero() {
        let mut g = fixture("teams");
        let host = field(&mut g, "LC22", 2, 0, false);
        attach(&mut g, 0, &host);
        attach(&mut g, 0, &host);
        assert_eq!(g.icons(g.board(&host).unwrap().1, 0).combat, 3);
        assert_eq!(g.contest_counts(0, 1), [0, 3]);
        assert_eq!(g.contributors(1, 0, 1), vec![2]);
        assert_eq!(g.view(0).regions[0].characters.len(), 1);
        assert_eq!(g.view(0).attachments.len(), 2);
        g.board_mut(&host).unwrap().exhausted = true;
        assert_eq!(g.contest_counts(0, 1), [0, 0]);
    }
    #[test]
    fn attachment_missing_host_after_murder_response_buries_single_card_without_refund() {
        let mut g = fixture("duel");
        let host = field(&mut g, "LC22", 1, 0, false);
        let equipment = hand(&mut g, "BQ022", 0);
        let murder = hand(&mut g, "JC091", 1);
        assets(&mut g, 0, "JC125", 1);
        assets(&mut g, 1, "JC091", 3);
        g.apply(
            0,
            Action {
                card_id: Some(equipment),
                target_id: Some(host.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        g.apply(0, Action::new("pass")).unwrap();
        g.apply(
            1,
            Action {
                card_id: Some(murder),
                target_id: Some(host.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        resolve_top(&mut g);
        assert!(g.board(&host).is_none());
        assert_eq!(g.stack.len(), 1);
        let mut g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
        resolve_top(&mut g);
        assert!(g.attachments.is_empty());
        assert_eq!(g.resources(0), 0);
        assert_eq!(
            g.players[0]
                .graveyard
                .iter()
                .filter(|c| c.definition == "BQ022")
                .count(),
            1
        );
        assert!(g
            .log
            .iter()
            .any(|l| l.text.contains("合金指虎") && l.text.contains("取消")));
    }
    #[test]
    fn actual_hide_recycles_to_owner_new_private_identity_without_response_or_hidden_attachment() {
        let mut g = fixture("duel");
        let host = field(&mut g, "LC22", 1, 0, false);
        attach(&mut g, 0, &host);
        let old = g.attachments[0].card.id.clone();
        let chase = hand(&mut g, "JC063", 0);
        assets(&mut g, 0, "JC063", 2);
        g.apply(
            0,
            Action {
                card_id: Some(chase),
                target_id: Some(host.clone()),
                option: Some("hide".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        resolve_top(&mut g);
        assert!(g.attachments.is_empty());
        assert!(g.stack.is_empty());
        assert!(g.board(&host).is_none());
        let returned = g.players[0]
            .hand
            .iter()
            .find(|c| c.definition == "BQ022")
            .unwrap();
        assert_ne!(returned.id, old);
        assert_eq!(returned.controller, returned.owner);
        assert!(g.players[1].hand.iter().all(|c| c.definition != "BQ022"));
        assert!(g
            .view(1)
            .hand
            .iter()
            .all(|c| c.card_id.as_deref() != Some("BQ022")));
        assert!(g
            .regions
            .iter()
            .flat_map(|r| r.cards.iter())
            .any(|c| c.face_down));
    }
    #[test]
    fn ordinary_move_preserves_equipment_but_host_departure_and_attachment_destruction_differ() {
        let mut g = fixture("duel");
        let host = field(&mut g, "JC059", 0, 0, false);
        attach(&mut g, 0, &host);
        let attachment_id = g.attachments[0].card.id.clone();
        g.board_mut(&host).unwrap().damage = 1;
        g.board_mut(&host).unwrap().exhausted = true;
        primitive(
            &mut g,
            &host,
            vec![Op::MoveOnBoard {
                entity: EntityRef::Source,
                region: RegionRef::Chosen,
            }],
        );
        // Above has no chosen region and is a no-op; explicit next frame tests actual movement.
        let source = g.source_snapshot(g.board(&host).unwrap().1, Some(0));
        let mut frame = ResolutionFrame {
            frame_id: "move".into(),
            ability_key: "move".into(),
            actor: 0,
            source,
            targets: vec![],
            already_paid: vec![],
            guard: GuardState::Unchecked,
            cursor: 0,
            steps: vec![Step {
                context: 0,
                op: Op::MoveOnBoard {
                    entity: EntityRef::Source,
                    region: RegionRef::Chosen,
                },
            }],
            chosen_region: Some(1),
        };
        g.resolve_frame(frame.clone()).unwrap();
        assert_eq!(g.board(&host).unwrap().0, 1);
        assert_eq!(g.board(&host).unwrap().1.damage, 1);
        assert!(g.board(&host).unwrap().1.exhausted);
        assert_eq!(g.attachments[0].card.id, attachment_id);
        assert_eq!(g.view(1).attachments[0].card.region, Some(1));
        frame.steps = vec![Step {
            context: 0,
            op: Op::Destroy(EntityRef::Source),
        }];
        frame.source = g.source_snapshot(g.board(&attachment_id).unwrap().1, Some(1));
        frame.cursor = 0;
        g.resolve_frame(frame).unwrap();
        assert!(g.board(&host).is_some());
        assert!(g.attachments.is_empty());
        assert!(g.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "BQ022"));
        assert!(g.players[0].hand.iter().all(|c| c.definition != "BQ022"));
        g.board_mut(&host).unwrap().exhausted = false;
        attach(&mut g, 0, &host);
        g.return_hand(&host);
        assert!(g.attachments.is_empty());
        assert_eq!(
            g.players[0]
                .hand
                .iter()
                .filter(|c| c.definition == "BQ022")
                .count(),
            1
        );
    }
    #[test]
    fn invalid_host_type_destroys_instead_of_recycling_and_world_hide_uses_shared_cleanup() {
        let mut g = fixture("duel");
        let host = field(&mut g, "LC22", 0, 0, false);
        attach(&mut g, 0, &host);
        // No released card changes type. This is explicitly a continuous-condition primitive fixture.
        g.board_mut(&host).unwrap().definition = "XQ03".into();
        g.settle_attachments();
        assert!(g.attachments.is_empty());
        assert!(g.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "BQ022"));
        let second = field(&mut g, "LC22", 1, 0, false);
        attach(&mut g, 0, &second);
        g.world_hide(None, false);
        assert!(g.attachments.is_empty());
        assert!(g.players[0].hand.iter().any(|c| c.definition == "BQ022"));
    }
    #[test]
    fn won_region_batches_cross_owner_equipment_and_preserves_aura_until_every_order_is_saved() {
        let mut g = fixture("teams");
        let host0 = field(&mut g, "LC22", 0, 0, false);
        let host1 = field(&mut g, "LC22", 1, 0, false);
        let aura = field(&mut g, "JC059", 0, 0, false);
        let patient = field(&mut g, "JC125", 1, 0, false);
        attach(&mut g, 1, &host0);
        attach(&mut g, 0, &host1);
        let hidden = field(&mut g, "LC21", 2, 0, true);
        field(&mut g, "LC20", 2, 0, false);
        field(&mut g, "JC125", 3, 0, true);
        field(&mut g, "LC23", 3, 0, false);
        g.board_mut(&hidden).unwrap().damage = 7;
        g.board_mut(&patient).unwrap().damage = 1;
        g.board_mut(&host0).unwrap().exhausted = true;
        g.board_mut(&host0).unwrap().shield = 2;
        g.board_mut(&host1).unwrap().controller = 0;
        let old_ids = g
            .in_play_cards()
            .iter()
            .map(|(_, c)| c.id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let old_prefixes = g
            .players
            .iter()
            .map(|p| serde_json::to_string(&p.deck).unwrap())
            .collect::<Vec<_>>();
        let mut expected_bottoms = vec![Vec::<String>::new(); 4];
        let old_decks = g.players.iter().map(|p| p.deck.len()).collect::<Vec<_>>();
        let old_equipment = g
            .players
            .iter()
            .map(|p| p.deck.iter().filter(|c| c.definition == "BQ022").count())
            .collect::<Vec<_>>();
        g.begin_window(Window::Win(0, 0));
        for seat in 0..4 {
            g.apply(seat, Action::new("pass")).unwrap();
        }
        for seat in 0..4 {
            let pending = g.pending.clone().unwrap();
            assert_eq!(pending.seat, seat);
            assert_eq!(pending.choice.options.len(), if seat < 2 { 3 } else { 2 });
            let ids = pending
                .choice
                .options
                .iter()
                .rev()
                .map(|o| o.id.clone())
                .collect::<Vec<_>>();
            expected_bottoms[seat] = ids
                .iter()
                .map(|id| g.board(id).unwrap().1.definition.clone())
                .collect();
            g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
            g.apply(
                seat,
                Action {
                    choice_id: Some(pending.choice.id),
                    bottom: Some(ids),
                    ..Action::new("choose")
                },
            )
            .unwrap();
            if seat < 3 {
                assert!(g.board(&aura).is_some());
                assert!(g.board(&patient).is_some());
                assert_eq!(g.attachments.len(), 2);
                assert_eq!(
                    g.players.iter().map(|p| p.deck.len()).collect::<Vec<_>>(),
                    old_decks
                );
            }
        }
        assert!(g.region_return.is_none());
        assert!(g.attachments.is_empty());
        let mut new_ids = std::collections::BTreeSet::new();
        for seat in 0..4 {
            assert_eq!(
                g.players[seat].deck.len(),
                old_decks[seat] + expected_bottoms[seat].len()
            );
            assert_eq!(
                serde_json::to_string(&g.players[seat].deck[..old_decks[seat]]).unwrap(),
                old_prefixes[seat]
            );
            let returned = &g.players[seat].deck[old_decks[seat]..];
            assert_eq!(
                returned
                    .iter()
                    .map(|c| c.definition.clone())
                    .collect::<Vec<_>>(),
                expected_bottoms[seat]
            );
            for c in returned {
                assert!(!old_ids.contains(&c.id) && new_ids.insert(c.id.clone()));
                assert_eq!((c.owner, c.controller), (seat, seat));
                assert!(!c.exhausted && !c.face_down);
                assert_eq!((c.damage, c.wounds, c.shield), (0, 0, 0));
            }
            assert!(g.players[seat].hand.iter().all(|c| c.definition != "BQ022"));
            assert!(g.players[seat].graveyard.is_empty());
            assert_eq!(
                g.players[seat]
                    .deck
                    .iter()
                    .filter(|c| c.definition == "BQ022")
                    .count(),
                old_equipment[seat] + usize::from(seat < 2)
            );
        }
        assert!(g.log.iter().all(|l| !l.text.contains("死亡")));
    }
}
