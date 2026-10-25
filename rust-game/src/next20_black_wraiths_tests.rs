//! Finite native adapters, explicitly arranged layouts. The death replacement
//! boundary remains a main-engine integration dependency, not a UI pass.
use crate::{catalog, model::*, next20_black_wraiths as black, rules::*, unit_support::*};

fn at(g: &mut Game, id: &str, seat: usize, region: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.regions[region].cards.push(c);
    id
}
fn grave(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.players[seat].graveyard.push(c);
    id
}
fn deck(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.players[seat].deck.push(c);
    id
}
fn search(g: &mut Game, source: &str, victim: &str) -> ResolutionFrame {
    let sources = g.cq25_capture_sources();
    let (r, c) = g.board(victim).unwrap();
    let dead = g.next20_source(c, r);
    g.remove_dead(victim, RemovalCause::Destroy);
    g.cq25_observe_death(&sources, &dead);
    let d = g
        .effects
        .iter()
        .find_map(|e| match e {
            Effect::Declare { declaration }
                if declaration.source.card.id == source
                    && declaration.ability.key == black::SEARCH =>
            {
                Some(declaration.clone())
            }
            _ => None,
        })
        .unwrap();
    ResolutionFrame {
        resource_payments: vec![],
        frame_id: g.id(),
        ability_key: d.ability.key,
        actor: d.actor,
        source: d.source,
        targets: vec![],
        already_paid: vec![],
        guard: GuardState::Accepted,
        cursor: 1,
        steps: d
            .ability
            .ops
            .into_iter()
            .map(|op| Step {
                context: d.actor,
                op,
            })
            .collect(),
        chosen_region: None,
    }
}
fn move_frame(g: &mut Game, ghost: &str, target: &str, sealed: bool) -> ResolutionFrame {
    let (source, host) = if sealed {
        let (s, h) = g.xg25_sealed_source(0, ghost).unwrap();
        (s, Some(Box::new(h)))
    } else {
        let (r, c) = g.board(ghost).unwrap();
        (g.next20_source(c, r), None)
    };
    let mut a = black::definition("XG25").unwrap().abilities.remove(0);
    a.ops = vec![Op::Next20Wraith(black::WraithProgram::GhostMove {
        sealed_host: host,
    })];
    let command = Action {
        target_id: Some(target.into()),
        ..Action::new("activate")
    };
    let targets = g.bind_action(0, &source, &a, &command).unwrap();
    let paid = g.pay_ability_costs(0, &source, &a, &command).unwrap();
    let mut f = g.make_frame(0, source, &a, targets, paid, None);
    f.frame_id = g.id();
    f.cursor = 1;
    f.guard = GuardState::Accepted;
    f
}
#[test]
fn next20_black_printed_fields_spirit_and_fixed_abilities() {
    for (id, cost, subtypes) in [
        ("CQ25", 4, vec!["猫", "诅咒"]),
        ("XG25", 2, vec!["幽灵", "诅咒"]),
    ] {
        let d = catalog::card(id);
        assert_eq!(
            (d.cost, d.defense, d.unique, d.supported),
            (cost, Some(1), false, true)
        );
        assert_eq!(d.subtypes, subtypes);
        assert_eq!(d.loyalty, vec!["黑色"]);
        assert_eq!(d.magic, "死亡");
        assert_eq!(d.temporary_icons.combat, 1);
        assert_eq!(d.permanent_icons.combat, if id == "CQ25" { 2 } else { 0 });
        let definition = black::definition(id).unwrap();
        assert_eq!(definition.traits.spirit, id == "XG25");
        black::validate_definition(id, &definition).unwrap();
        for a in &definition.abilities {
            black::validate_ability(id, a).unwrap();
            assert!(black::validate_ability("JC125", a).is_err());
        }
    }
    let a = black::definition("XG25").unwrap().abilities.remove(0);
    assert_eq!(a.per_turn_limit, Some(1));
    assert!(a.activation_only);
}
#[test]
fn next20_black_cq25_spirit_requires_current_personal_secrecy_and_faceup() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    assert!(!g.cq25_spirit(g.board(&tiger).unwrap().1));
    g.players[1].society_zone.secrecy = 1;
    assert!(!g.cq25_spirit(g.board(&tiger).unwrap().1));
    g.players[0].society_zone.secrecy = 1;
    assert!(g.cq25_spirit(g.board(&tiger).unwrap().1));
    g.board_mut(&tiger).unwrap().controller = 2;
    assert!(!g.cq25_spirit(g.board(&tiger).unwrap().1));
    g.players[2].society_zone.secrecy = 1;
    assert!(g.cq25_spirit(g.board(&tiger).unwrap().1));
    g.board_mut(&tiger).unwrap().face_down = true;
    assert!(!g.cq25_spirit(g.board(&tiger).unwrap().1));
}
#[test]
fn next20_black_cq25_bonus_only_own_displayed_curse_in_its_entry_contest() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    let curse = g.make_card("XG25", 0);
    let plain = g.make_card("JC125", 0);
    let (r, c) = g.board(&tiger).unwrap();
    let source = g.next20_source(c, r);
    assert_eq!(
        g.cq25_contest_bonus(&source, black::CONTEST, 0, 0, &curse),
        2
    );
    for seat in 1..4 {
        assert_eq!(
            g.cq25_contest_bonus(&source, black::CONTEST, 0, seat, &curse),
            0
        );
    }
    assert_eq!(
        g.cq25_contest_bonus(&source, black::SEARCH, 0, 0, &curse),
        0
    );
    assert_eq!(
        g.cq25_contest_bonus(&source, black::CONTEST, 0, 0, &plain),
        0
    );
    let mut other = source;
    other.card.definition = "XG25".into();
    assert_eq!(
        g.cq25_contest_bonus(&other, black::CONTEST, 0, 0, &curse),
        0
    );
}
#[test]
fn next20_black_cq25_grave_search_seals_on_original_instance_without_shuffle() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    let victim = field(&mut g, "JC125", 2);
    let ghost = grave(&mut g, "XG25", 0);
    let f = search(&mut g, &tiger, &victim);
    let rng = g.random;
    g.cq25_search_start(f.clone()).unwrap();
    assert!(g
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .any(|o| o.id == ghost));
    g.cq25_search_finish(&f, 0, &[ghost.clone()]).unwrap();
    assert_eq!(g.random, rng);
    assert!(g
        .sealed_cards
        .iter()
        .any(|s| s.card.definition == "XG25" && s.host_id == tiger && s.card.id != ghost));
}
#[test]
fn next20_black_cq25_deck_search_shuffles_and_resets_identity() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    let victim = field(&mut g, "JC125", 2);
    let ghost = deck(&mut g, "XG25", 0);
    let f = search(&mut g, &tiger, &victim);
    let rng = g.random;
    assert!(g
        .cq25_search_finish(&f, 0, &["search-deck-no-card".into()])
        .is_err());
    assert_eq!(g.random, rng);
    g.cq25_search_finish(&f, 0, &[ghost.clone()]).unwrap();
    assert_ne!(g.random, rng);
    assert!(!g.players[0].deck.iter().any(|c| c.id == ghost));
    assert!(g
        .sealed_cards
        .iter()
        .any(|s| s.host_id == tiger && s.card.id != ghost));
}
#[test]
fn next20_black_cq25_failed_deck_search_still_shuffles_and_requires_confirmation() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    let victim = field(&mut g, "JC125", 2);
    let f = search(&mut g, &tiger, &victim);
    g.cq25_search_start(f.clone()).unwrap();
    let p = g.pending.as_ref().unwrap();
    assert_eq!((p.choice.min, p.choice.max), (Some(1), Some(1)));
    let before = g.random;
    assert!(g.cq25_search_finish(&f, 0, &[]).is_err());
    g.cq25_search_finish(&f, 0, &["search-deck-no-card".into()])
        .unwrap();
    assert_ne!(g.random, before);
}
#[test]
fn next20_black_cq25_source_leaving_never_seals_on_replacement_tiger() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    let victim = field(&mut g, "JC125", 2);
    let ghost = grave(&mut g, "XG25", 0);
    let f = search(&mut g, &tiger, &victim);
    g.remove_dead(&tiger, RemovalCause::Destroy);
    let replacement = field(&mut g, "CQ25", 0);
    g.cq25_search_finish(&f, 0, &[ghost.clone()]).unwrap();
    assert!(!g.sealed_cards.iter().any(|s| s.host_id == replacement));
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XG25" && c.id != ghost));
}
#[test]
fn next20_black_cq25_only_real_enemy_faceup_characters_in_original_region() {
    for (seat, region, hidden, expected) in [
        (0, 0, false, 0),
        (1, 0, false, 0),
        (2, 1, false, 0),
        (2, 0, true, 0),
        (2, 0, false, 1),
    ] {
        let mut g = game();
        field(&mut g, "CQ25", 0);
        let victim = at(&mut g, "JC125", seat, region);
        g.board_mut(&victim).unwrap().face_down = hidden;
        let sources = g.cq25_capture_sources();
        let (r, c) = g.board(&victim).unwrap();
        let dead = g.next20_source(c, r);
        g.cq25_observe_death(&sources, &dead);
        assert_eq!(g.effects.iter().filter(|e| matches!(e,Effect::Declare { declaration } if declaration.ability.key == black::SEARCH)).count(),expected);
    }
}
#[test]
fn next20_black_cq25_foreign_zone_wrong_player_and_corrupt_death_do_not_mutate() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    let victim = field(&mut g, "JC125", 2);
    let foreign = grave(&mut g, "XG25", 2);
    let f = search(&mut g, &tiger, &victim);
    let before = serde_json::to_value(&g).unwrap();
    assert!(g.cq25_search_finish(&f, 0, &[foreign]).is_err());
    assert!(g
        .cq25_search_finish(&f, 1, &["search-deck-no-card".into()])
        .is_err());
    let mut bad = f.clone();
    let Op::Next20Wraith(black::WraithProgram::TigerSearch {
        observation: Some(o),
    }) = &mut bad.steps[0].op
    else {
        panic!()
    };
    o.dead.card.controller = 0;
    assert!(g.cq25_search_start(bad).is_err());
    assert_eq!(serde_json::to_value(&g).unwrap(), before);
}
#[test]
fn next20_black_xg25_moves_adjacent_enemy_with_attachments_and_one_fee() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let victim = at(&mut g, "JC029", 2, 1);
    let card = g.make_card("XQ14", 2);
    let blood = card.id.clone();
    g.attachments.push(Attachment {
        card,
        host_id: victim.clone(),
    });
    fund(&mut g, 0, "JC084", 2);
    let f = move_frame(&mut g, &ghost, &victim, false);
    assert_eq!(g.resources(0), 1);
    g.xg25_move(&f).unwrap();
    assert_eq!(g.board(&victim).unwrap().0, 0);
    assert!(g
        .attachments
        .iter()
        .any(|a| a.card.id == blood && a.host_id == victim));
    assert_eq!(
        g.turn_ability_usage
            .iter()
            .filter(|u| u.source_instance == ghost && u.ability_key == black::MOVE)
            .count(),
        1
    );
    let source = f.source.clone();
    let a = black::definition("XG25").unwrap().abilities.remove(0);
    assert!(g
        .pay_ability_costs(0, &source, &a, &Action::new("activate"))
        .is_err());
    assert_eq!(g.resources(0), 1);
}
#[test]
fn next20_black_xg25_target_predicate_rejects_friendly_same_region_hidden_and_nonadjacent() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let (r, c) = g.board(&ghost).unwrap();
    let source = g.next20_source(c, r);
    let t = black::definition("XG25").unwrap().abilities[0].targets[0].clone();
    for (seat, region, hidden, expected) in [
        (2, 1, false, true),
        (0, 1, false, false),
        (1, 1, false, false),
        (2, 0, false, false),
        (2, 2, false, false),
        (2, 1, true, false),
    ] {
        let victim = at(&mut g, "JC125", seat, region);
        g.board_mut(&victim).unwrap().face_down = hidden;
        assert_eq!(g.valid_binding(0, &source, &t, &victim), expected);
    }
}
#[test]
fn next20_black_xg25_original_region_is_frozen_and_replacement_region_does_not_receive() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let victim = at(&mut g, "JC125", 2, 1);
    fund(&mut g, 0, "JC084", 1);
    let f = move_frame(&mut g, &ghost, &victim, false);
    g.regions[0].card = g.make_card("DQJC115", 0);
    g.xg25_move(&f).unwrap();
    assert_eq!(g.board(&victim).unwrap().0, 1);
    assert_eq!(g.resources(0), 0);
}
#[test]
fn next20_black_xg25_sealed_on_character_can_pay_and_move_without_granting_other_text() {
    let mut g = game();
    let host = field(&mut g, "CQ25", 0);
    let c = g.make_card("XG25", 0);
    g.seal_card_on_valid_host(c, 0, &host);
    let ghost = g.sealed_cards[0].card.id.clone();
    let victim = at(&mut g, "JC125", 2, 1);
    fund(&mut g, 0, "JC084", 1);
    let f = move_frame(&mut g, &ghost, &victim, true);
    g.xg25_move(&f).unwrap();
    assert_eq!(g.board(&victim).unwrap().0, 0);
    assert!(g.board(&ghost).is_none());
    assert!(g.sealed_cards.iter().any(|s| s.card.id == ghost));
    assert!(g.xg25_sealed_source(1, &ghost).is_err());
    let view = g.sealed_card_view(&g.sealed_cards[0].card, 0);
    assert!(view.icons.is_none());
    assert!(view.magic.is_none());
}
#[test]
fn next20_black_xg25_sealed_on_society_has_no_activation_and_host_departure_returns_to_owner() {
    let mut g = game();
    let c = g.make_card("XG25", 0);
    g.seal_card_on_valid_host(c, 0, "society:p0");
    let id = g.sealed_cards[0].card.id.clone();
    assert!(g.xg25_sealed_source(0, &id).is_err());
    let host = field(&mut g, "CQ25", 0);
    let c = g.make_card("XG25", 0);
    g.seal_card_on_valid_host(c, 0, &host);
    let ghost = g.sealed_cards[1].card.id.clone();
    g.remove_dead(&host, RemovalCause::Destroy);
    assert!(g.xg25_sealed_source(0, &ghost).is_err());
    assert!(g.players[0].hand.iter().any(|c| c.definition == "XG25"));
}
#[test]
fn next20_black_xg25_death_replacement_is_conditioned_on_original_controllers_secrecy() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let (r, c) = g.board(&ghost).unwrap();
    let s = g.next20_source(c, r);
    assert!(!g.xg25_replaces_death(&s));
    g.players[1].society_zone.secrecy = 1;
    assert!(!g.xg25_replaces_death(&s));
    g.players[0].society_zone.secrecy = 1;
    assert!(g.xg25_replaces_death(&s));
    let mut bad = s.clone();
    bad.card.face_down = true;
    assert!(!g.xg25_replaces_death(&bad));
    let mut bad = s;
    bad.card.definition = "CQ25".into();
    assert!(!g.xg25_replaces_death(&bad));
}
#[test]
fn next20_black_xg25_death_adapter_seals_before_grave_and_preserves_owner() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let host = field(&mut g, "JC125", 2);
    g.players[0].society_zone.secrecy = 1;
    let (r, c) = g.board(&ghost).unwrap();
    let s = g.next20_source(c, r);
    let (_, departed) = g.leave_board(&ghost).unwrap();
    g.xg25_seal_instead_of_grave(departed, &s, &host).unwrap();
    assert!(g.players[0].graveyard.is_empty());
    assert!(g
        .sealed_cards
        .iter()
        .any(|s| s.host_id == host && s.card.definition == "XG25" && s.card.owner == 0));
}
#[test]
fn next20_black_xg25_invalid_original_host_falls_back_to_owner_grave() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    g.players[0].society_zone.secrecy = 1;
    let (r, c) = g.board(&ghost).unwrap();
    let s = g.next20_source(c, r);
    let (_, departed) = g.leave_board(&ghost).unwrap();
    g.xg25_seal_instead_of_grave(departed, &s, "old-host")
        .unwrap();
    assert_eq!(g.players[0].graveyard.len(), 1);
    assert!(g.sealed_cards.is_empty());
}
#[test]
fn next20_black_xg25_replacement_rejects_duplicate_physical_card() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let host = field(&mut g, "JC125", 2);
    g.players[0].society_zone.secrecy = 1;
    let (r, c) = g.board(&ghost).unwrap();
    let s = g.next20_source(c, r);
    let duplicate = c.clone();
    let before = serde_json::to_value(&g).unwrap();
    assert!(g.xg25_seal_instead_of_grave(duplicate, &s, &host).is_err());
    assert_eq!(serde_json::to_value(&g).unwrap(), before);
}
#[test]
fn next20_black_search_pending_roundtrip_and_corruption_checks() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    let victim = field(&mut g, "JC125", 2);
    grave(&mut g, "XG25", 0);
    let f = search(&mut g, &tiger, &victim);
    g.cq25_search_start(f).unwrap();
    let p = g.pending.as_ref().unwrap().clone();
    g.validate_next20_black_pending(&p).unwrap();
    let restored: Pending = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
    g.validate_next20_black_pending(&restored).unwrap();
    for n in 0..4 {
        let mut bad = p.clone();
        match n {
            0 => bad.seat = 1,
            1 => bad.choice.options.clear(),
            2 => bad.choice.min = Some(0),
            _ => bad.choice.allow_decline = Some(true),
        }
        assert!(g.validate_next20_black_pending(&bad).is_err());
    }
    let mut wrong_continuation = p;
    if let ChoiceResolution::Frame { choice, .. } = &mut wrong_continuation.resolution {
        *choice = FrameChoice::VampireBlood;
    }
    assert!(g
        .validate_next20_black_pending(&wrong_continuation)
        .is_err());
}
#[test]
fn next20_black_xg25_normal_board_action_uses_public_payment_response_and_restore() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let victim = at(&mut g, "JC125", 2, 1);
    fund(&mut g, 0, "JC084", 1);
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.card_id.as_deref() == Some(&ghost)
                && a.action.ability_id.as_deref() == Some(black::MOVE)
                && a.action.target_id.as_deref() == Some(&victim)
        })
        .expect("offered XG25 action")
        .action;
    act(&mut g, 0, a);
    top(&mut g);
    assert_eq!(g.board(&victim).unwrap().0, 0);
    assert_eq!(g.resources(0), 0);
    restore(&mut g);
}
#[test]
fn next20_black_cq25_actual_shared_contest_binds_bonus_to_actor_and_restores_outcome() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    for seat in 0..4 {
        held(&mut g, "XG25", seat);
    }
    let (r, c) = g.board(&tiger).unwrap();
    let source = g.next20_source(c, r);
    let a = black::definition("CQ25").unwrap().abilities.remove(0);
    let mut f = g.make_frame(0, source, &a, vec![], vec![], None);
    f.frame_id = g.id();
    f.guard = GuardState::Accepted;
    f.cursor = 1;
    g.validate_cq25_contest_frame(&f).unwrap();
    g.mystic_contest_start(f, 0).unwrap();
    g.drive().unwrap();
    for _ in 0..60 {
        if g.mystic_contest.is_none() {
            break;
        }
        if let Some(p) = g.pending.clone() {
            let selected = match p.choice.kind.as_str() {
                crate::mystic_contest::COMMIT_KIND => vec![g.players[p.seat]
                    .hand
                    .iter()
                    .find(|c| c.definition == "XG25")
                    .unwrap()
                    .id
                    .clone()],
                crate::mystic_contest::REWARD_KIND => vec![],
                other => panic!("unexpected {other}"),
            };
            choose(&mut g, selected);
        } else if !g.stack.is_empty() {
            pass(&mut g);
        } else {
            g.drive().unwrap();
        }
    }
    assert!(g.mystic_contest.is_none());
    let o = &g.mystic_history[0];
    assert_eq!(o.actor_bonus, 0);
    assert_eq!(
        o.shown.iter().map(|s| s.value).collect::<Vec<_>>(),
        vec![4, 2, 2, 2]
    );
    g.validate_mystic_outcome(o).unwrap();
    restore(&mut g);
}
#[test]
fn next20_black_state_rejects_duplicate_death_and_paid_move_with_wrong_public_target_region() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    let victim = field(&mut g, "JC125", 2);
    let _ = search(&mut g, &tiger, &victim);
    g.validate_next20_black_state().unwrap();
    let duplicate = g.effects.iter().find(|e| matches!(e,Effect::Declare { declaration } if declaration.ability.key == black::SEARCH)).unwrap().clone();
    g.effects.push_back(duplicate);
    assert!(g.validate_next20_black_state().is_err());
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let victim = at(&mut g, "JC125", 2, 1);
    fund(&mut g, 0, "JC084", 1);
    let mut f = move_frame(&mut g, &ghost, &victim, false);
    f.targets[0].public.region = Some(2);
    assert!(g.validate_xg25_move_frame(&f).is_err());
}
