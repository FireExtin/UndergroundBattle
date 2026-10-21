//! Third-batch black attributes and the source-bound attachment barrier action.
use crate::{engine::RuleResult, model::*, rules};
use std::collections::BTreeSet;

impl Game {
    pub(crate) fn current_guard(&self, c: &Card) -> u32 {
        if c.face_down {
            return 0;
        }
        if c.definition == "XG26" {
            return u32::from(
                self.players
                    .get(c.controller)
                    .is_some_and(|p| p.society_zone.secrecy > 0),
            );
        }
        rules::definition(&c.definition).traits.guard
    }
    pub(crate) fn xg26_local_influence(&self, c: &Card, region: usize) -> bool {
        c.definition == "XG26"
            && !c.face_down
            && self.regions.get(region).is_some_and(|r| {
                !r.vacant
                    && r.cards.iter().any(|live| live == c)
                    && r.influence[self.team(c.controller)] > 0
            })
    }
    pub(crate) fn xg26_grant_barrier(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_batch3_frame(f)?;
        if f.source.card.definition != "XG26"
            || f.ability_key != "batch3-xg26-barrier"
            || f.cursor != 1
        {
            return Err("看场打手的屏障程序或执行阶段不符".into());
        }
        let target = &f.targets[0].id;
        if !self.in_play(target).is_some_and(|(_,c)|c.controller==f.actor && self.live_attachment_role(c)) {
            return Ok(());
        }
        if !self.turn_attribute_modifiers.iter().any(|m| {
            m.target_instance == *target && m.expires_turn == self.turn && m.grants_barrier
        }) {
            self.turn_attribute_modifiers.push(TurnAttributeModifier {
                target_instance: target.clone(),
                defense_bonus: 0,
                kill_bonus: 0,
                grants_retreat: false,
                printed_defense_override: None,
                ordinary_icons: Icons::default(),
                grants_renown: false,
                grants_barrier: true,
                prevents_damage: false,
                expires_turn: self.turn,
            });
        }
        Ok(())
    }
    /// Reuse the existing exact-UID, turn-duration scalar mechanism; these
    /// grants cannot carry character attributes or survive target departure.
    pub(crate) fn validate_attachment_barrier_modifiers(&self) -> RuleResult<()> {
        let mut targets = BTreeSet::new();
        for m in self
            .turn_attribute_modifiers
            .iter()
            .filter(|m| m.grants_barrier)
        {
            if m.target_instance.is_empty()
                || !targets.insert(&m.target_instance)
                || m.expires_turn != self.turn
                || m.defense_bonus != 0
                || m.kill_bonus != 0
                || m.grants_retreat
                || m.printed_defense_override.is_some()
                || m.ordinary_icons != Icons::default()
                || m.grants_renown
                || m.prevents_damage
                || !self.regions.iter().flat_map(|r| &r.cards).chain(self.attachments.iter().map(|a| &a.card)).find(|c|c.id==m.target_instance)
                    .is_some_and(|c|c.owner<self.players.len() && c.controller<self.players.len() && self.live_attachment_role(c))
            {
                return Err("附属屏障的实体、期限或单一效果不符".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit_support::*;
    use std::collections::BTreeMap;

    fn attachment(g: &mut Game, owner: usize) -> String {
        let c = g.make_card("JC048", owner);
        let id = c.id.clone();
        g.attachments.push(Attachment {
            card: c,
            host_id: g.regions[0].card.id.clone(),
        });
        id
    }
    fn activate(source: &str, target: &str) -> Action {
        Action {
            card_id: Some(source.into()),
            target_id: Some(target.into()),
            ..Action::new("activate")
        }
    }

    #[test]
    fn batch3_xg26_four_seat_views_show_paid_attachment_barrier_and_expiry() {
        for society in [false, true] {
            let mut g = game();
            g.players[0].society_zone.card = None;
            let source = field(&mut g, "XG26", 0);
            let target = g.make_card(if society { "JC068" } else { "JC048" }, 0);
            let id = target.id.clone();
            g.attachments.push(Attachment {
                card: target,
                host_id: if society { "society:p0".into() } else { g.regions[0].card.id.clone() },
            });
            fund(&mut g, 0, "JC047", 1);
            act(&mut g, 0, activate(&source, &id));
            top(&mut g);
            for active in [true, false] {
                if !active {
                    g.turn += 1;
                    g.prune_turn_attribute_modifiers();
                }
                for seat in 0..4 {
                    let view = g.view(seat);
                    let c = &view.attachments.iter().find(|a| a.card.instance_id == id).unwrap().card;
                    assert_eq!(c.current_barrier, active.then_some(true));
                    assert_eq!(g.targetable(2, g.in_play(&id).unwrap().1), !active);
                }
                if active {
                    // Check privacy while the paid modifier still exists.
                    let mut hidden = g.in_play(&id).unwrap().1.clone();
                    hidden.face_down = true;
                    assert!(g.turn_attribute_modifiers.iter().any(|m| m.target_instance == id && m.grants_barrier));
                    for seat in 0..4 {
                        assert_eq!(g.card_view(&hidden, seat, Some(0), Some("attachment")).current_barrier, None);
                        assert_eq!(g.card_view(&hidden, seat, None, Some("asset")).current_barrier, None);
                    }
                }
                restore(&mut g);
            }
            // A concealed print or face-down asset never exports keyword fields.
            let mut hidden = g.in_play(&id).unwrap().1.clone();
            hidden.face_down = true;
            for seat in 0..4 {
                let view = g.card_view(&hidden, seat, Some(0), Some("attachment"));
                assert_eq!(view.current_barrier, None);
                if seat != 0 { assert_eq!(view.card_id, None); }
                assert_eq!(g.card_view(&hidden, seat, None, Some("asset")).current_barrier, None);
            }
        }
    }

    #[test]
    fn batch3_xg26_targets_own_society_attachment_without_a_society_card() {
        let mut g = game();
        g.players[0].society_zone.card = None;
        let source = field(&mut g, "XG26", 0);
        let target = g.make_card("JC068", 0);
        let id = target.id.clone();
        g.attachments.push(Attachment {
            card: target,
            host_id: "society:p0".into(),
        });
        fund(&mut g, 0, "JC047", 1);
        act(&mut g, 0, activate(&source, &id));
        assert_eq!(
            g.stack.last().unwrap().frame.as_ref().unwrap().targets[0]
                .public
                .kind,
            "attachment"
        );
        top(&mut g);
        assert!(!g.targetable(2, g.in_play(&id).unwrap().1));
        restore(&mut g);
    }

    #[test]
    fn batch3_xg26_real_paid_tap_grants_attachment_barrier_until_turn_end() {
        let mut g = game();
        let source = field(&mut g, "XG26", 0);
        let target = attachment(&mut g, 0);
        fund(&mut g, 0, "JC047", 1);
        act(&mut g, 0, activate(&source, &target));
        assert!(g.board(&source).unwrap().1.exhausted);
        assert!(g.players[0].assets[0].exhausted);
        let mut forged = g.clone();
        forged
            .stack
            .last_mut()
            .unwrap()
            .frame
            .as_mut()
            .unwrap()
            .already_paid[0] = PaidCost::Assets(vec![]);
        assert!(Game::from_persisted(&serde_json::to_string(&forged).unwrap()).is_err());
        top(&mut g);
        let card = g.in_play(&target).unwrap().1;
        assert!(!g.targetable(2, card));
        assert!(g.targetable(0, card));
        assert!(g.targetable(1, card));
        let enemy = field(&mut g, "JC043", 2);
        let (r, c) = g.in_play(&enemy).unwrap();
        let snapshot = g.source_snapshot(c, r);
        let slot = &rules::definition("JC043").abilities[0].targets[0];
        assert!(!g.valid_binding(2, &snapshot, slot, &target));
        let unprotected = attachment(&mut g, 0);
        assert!(g.valid_binding(2, &snapshot, slot, &unprotected));
        let mut forged = g.clone();
        forged.turn_attribute_modifiers[0].ordinary_icons.influence = 1;
        assert!(Game::from_persisted(&serde_json::to_string(&forged).unwrap()).is_err());
        let mut forged = g.clone();
        forged
            .attachments
            .iter_mut()
            .find(|a| a.card.id == target)
            .unwrap()
            .card
            .definition = "UNREGISTERED".into();
        assert!(forged.validate_attachment_barrier_modifiers().is_err());
        restore(&mut g);
        g.turn += 1;
        g.prune_turn_attribute_modifiers();
        assert!(g.targetable(2, g.in_play(&target).unwrap().1));
        restore(&mut g);
    }

    #[test]
    fn batch3_xg26_personal_target_guard_and_source_departure_keep_paid_costs() {
        let mut g = game();
        let source = field(&mut g, "XG26", 0);
        let own = attachment(&mut g, 0);
        let ally = attachment(&mut g, 1);
        fund(&mut g, 0, "JC047", 1);
        reject(&mut g, 0, activate(&source, &ally));
        assert!(!g.board(&source).unwrap().1.exhausted);
        assert!(!g.players[0].assets[0].exhausted);
        act(&mut g, 0, activate(&source, &own));
        g.attachments
            .iter_mut()
            .find(|a| a.card.id == own)
            .unwrap()
            .card
            .controller = 2;
        top(&mut g);
        assert!(g.targetable(2, g.in_play(&own).unwrap().1));
        assert!(g.turn_attribute_modifiers.is_empty());
        assert!(g.board(&source).unwrap().1.exhausted); // tap, not sacrifice

        let mut g = game();
        let source = field(&mut g, "XG26", 0);
        let own = attachment(&mut g, 0);
        fund(&mut g, 0, "JC047", 1);
        act(&mut g, 0, activate(&source, &own));
        g.remove_dead(&source, RemovalCause::Destroy);
        g.drive().unwrap();
        top(&mut g);
        assert!(g.board(&source).is_none());
        assert!(!g.targetable(2, g.in_play(&own).unwrap().1));
        restore(&mut g);
        g.remove_dead(&own, RemovalCause::Destroy);
        g.drive().unwrap();
        g.settle_deaths();
        let replacement = attachment(&mut g, 0);
        assert_ne!(replacement, own);
        assert!(g.targetable(2, g.in_play(&replacement).unwrap().1));
        assert!(g.turn_attribute_modifiers.is_empty());
        restore(&mut g);
    }

    #[test]
    fn batch3_xg26_personal_moon_guard_controls_real_combat_even_when_exhausted() {
        for personal_moon in [false, true] {
            let mut g = game();
            field(&mut g, "JC016", 0);
            let guard = field(&mut g, "XG26", 2);
            let other = field(&mut g, "LC24", 2);
            g.board_mut(&guard).unwrap().exhausted = true;
            // An allied moon cannot substitute for this card's controller.
            g.add_society_markers(if personal_moon { 2 } else { 3 }, 0, 1);
            g.begin_window(Window::Before(0, 1));
            g.close_window().unwrap();
            g.drive().unwrap();
            let p = g.pending.clone().unwrap();
            let amount = p.choice.amount.unwrap();
            assert!(amount > 0);
            let ignore_guard = Action {
                choice_id: Some(p.choice.id.clone()),
                allocations: Some(BTreeMap::from([(other.clone(), amount)])),
                ..Action::new("choose")
            };
            if personal_moon {
                reject(&mut g, p.seat, ignore_guard);
                act(
                    &mut g,
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id),
                        allocations: Some(BTreeMap::from([
                            (guard.clone(), 1),
                            (other, amount - 1),
                        ])),
                        ..Action::new("choose")
                    },
                );
                assert!(g.board(&guard).is_none());
            } else {
                act(&mut g, p.seat, ignore_guard);
                assert!(g.board(&guard).is_some());
            }
        }
    }

    #[test]
    fn batch3_xg26_local_influence_is_temporary_and_tap_does_not_sacrifice() {
        let mut g = game();
        let source = field(&mut g, "XG26", 0);
        g.regions[0].influence = [1, 0];
        assert_eq!(g.current_icons(g.board(&source).unwrap().1, 0).influence, 1);
        g.first_team = 1;
        assert_eq!(g.current_icons(g.board(&source).unwrap().1, 0).influence, 0);
        g.regions[0].influence = [0, 1];
        g.first_team = 0;
        assert_eq!(g.current_icons(g.board(&source).unwrap().1, 0).influence, 0);
        let a = crate::batch3::definition("XG26").unwrap().abilities[0].clone();
        assert!(matches!(
            a.costs.as_slice(),
            [rules::Cost::Assets(1), rules::Cost::ExhaustSource]
        ));
        let mut changed = a;
        changed.costs[1] = rules::Cost::SacrificeSource;
        assert!(rules::validate_ability("XG26", &changed).is_err());
    }
}
