//! Native layouts followed by public Game commands; every command round-trips
//! Game/Room state and all four private projections through the shared helper.
use crate::{
    catalog,
    jc029_tests::{apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject},
    model::*,
    rules::*,
};
fn held(g: &mut Game, def: &str, owner: usize, holder: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.players[holder].hand.push(c);
    id
}
fn activate(g: &mut Game, actor: usize, id: &str, key: &str) {
    apply(
        g,
        actor,
        Action {
            card_id: Some(id.into()),
            ability_id: Some(key.into()),
            ..Action::new("activate")
        },
    );
}
fn news(g: &mut Game, actor: usize, mode: &str) {
    let id = held(g, "WM042", actor, actor);
    fund(g, actor, "JC059", 1);
    apply(
        g,
        actor,
        Action {
            card_id: Some(id),
            option: Some(mode.into()),
            ..Action::new("play")
        },
    );
    pass_top(g);
}
fn bad(g: &Game, changed: Game) {
    assert!(Game::from_persisted(&serde_json::to_string(&changed).unwrap()).is_err());
    let mut r = envelope(g);
    r.game = changed;
    assert!(
        crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).is_err()
    );
}
fn mill_source(g: &mut Game, holder: usize) {
    let id = held(g, "JZ49", 0, 0);
    fund(g, 0, "JZ46", 4);
    apply(
        g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(g);
    choose(g, vec![player_id(holder)]);
    pass_top(g);
}
#[test]
fn exposure_complete_printed_programs_and_finite_rejection() {
    for id in ["WM039", "WM040", "WM042"] {
        let c = catalog::card(id);
        assert_eq!(c.color, "灰");
        assert!(!c.unique);
        assert_eq!(c.loyalty, ["灰色"]);
        let d = crate::exposure::definition(id).unwrap();
        crate::exposure::validate_definition(id, &d).unwrap();
        for a in &d.abilities {
            crate::rules::validate_ability(id, a).unwrap();
            if a.key == "mobility" {
                continue;
            }
            let mut changed = a.clone();
            changed.costs.push(Cost::Assets(1));
            assert!(crate::rules::validate_ability(id, &changed).is_err());
            assert!(crate::rules::validate_ability("JC125", a).is_err());
        }
    }
    let c = catalog::card("WM039");
    assert_eq!(
        (c.cost, c.defense, c.permanent_icons.influence),
        (2, Some(1), 1)
    );
    let c = catalog::card("WM040");
    assert_eq!(
        (
            c.cost,
            c.defense,
            c.permanent_icons.investigation,
            c.permanent_icons.influence
        ),
        (4, Some(2), 1, 2)
    );
    assert!(!c.rule_traits.public);
    assert_eq!(catalog::card("WM042").kind, "spell");
}
#[test]
fn exposure_immediate_cancellation_dynamic_hand_limit_and_public_views() {
    let mut g = game(0);
    g.add_society_markers(0, 0, 2);
    g.add_society_markers(1, 3, 0);
    g.add_society_markers(2, 0, 1);
    news(&mut g, 0, "expose");
    assert_eq!(
        (
            g.players[0].society_zone.secrecy,
            g.society_exposure(0),
            g.hand_limit(0)
        ),
        (1, 0, 8)
    );
    assert_eq!((g.society_exposure(1), g.hand_limit(1)), (4, 3));
    assert_eq!(
        (g.society_exposure(2), g.players[2].society_zone.secrecy),
        (0, 0)
    );
    g.begin_window(Window::Action(0));
    news(&mut g, 0, "cover");
    assert_eq!((g.society_exposure(1), g.hand_limit(1)), (3, 4));
    for viewer in 0..4 {
        let v = g.view(viewer);
        assert_eq!(
            (v.society_zones[0].secrecy, v.society_zones[1].hand_limit),
            (1, 4)
        );
    }
    let mut changed = g.clone();
    changed.players[1].society_zone.secrecy = 1;
    bad(&g, changed);
    g.add_society_markers(3, 10, 0);
    assert_eq!(g.hand_limit(3), 0);
}
#[test]
fn exposure_analyst_removal_is_effect_once_per_turn_and_bonus_expires() {
    let mut g = game(0);
    let id = board(&mut g, "WM039", 0, 2);
    fund(&mut g, 0, "JC059", 3);
    assert!(g.has_mobility(g.board(&id).unwrap().1));
    activate(&mut g, 0, &id, "wm039-analysis");
    pass_top(&mut g);
    assert_eq!(g.icons(g.board(&id).unwrap().1, 2).investigation, 1);
    assert_eq!(g.resources(0), 2);
    g.begin_window(Window::Action(0));
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id.clone()),
            ability_id: Some("wm039-analysis".into()),
            ..Action::new("activate")
        },
    );
    g.effect(Effect::FinishCleanup).unwrap();
    g.drive().unwrap();
    assert_eq!(g.icons(g.board(&id).unwrap().1, 2).investigation, 0);
    g.begin_window(Window::Action(0));
    g.add_society_markers(0, 2, 0);
    activate(&mut g, 0, &id, "wm039-analysis");
    pass_top(&mut g);
    assert_eq!(g.society_exposure(0), 1);
}
#[test]
fn exposure_commissioner_conditions_follow_controller_and_change_immediately() {
    let mut g = game(0);
    let id = board(&mut g, "WM040", 1, 2);
    g.board_mut(&id).unwrap().controller = 0;
    g.add_society_markers(1, 1, 0);
    assert!(!g.has_mobility(g.board(&id).unwrap().1));
    assert!(!g.current_public(g.board(&id).unwrap().1, 0));
    news(&mut g, 0, "expose");
    assert!(g.has_mobility(g.board(&id).unwrap().1));
    assert!(g.view(2).regions[2]
        .characters
        .iter()
        .find(|c| c.instance_id == id)
        .unwrap()
        .current_public
        .unwrap());
    g.begin_window(Window::Action(0));
    news(&mut g, 0, "cover");
    assert!(!g.has_mobility(g.board(&id).unwrap().1));
    g.begin_window(Window::Action(0));
    let hand = held(&mut g, "WM040", 0, 0);
    fund(&mut g, 0, "JC059", 5);
    g.add_society_markers(0, 1, 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(hand.clone()),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
    news(&mut g, 0, "cover");
    g.begin_window(Window::Action(0));
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(hand),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
}
#[test]
fn exposure_commissioner_actual_removal_repress_pending_restore_and_heal_wounds() {
    let mut g = game(0);
    g.add_society_markers(0, 2, 0);
    g.add_society_markers(1, 1, 0);
    g.add_society_markers(3, 0, 2);
    g.regions[0].influence[1] = 2;
    g.regions[2].influence[1] = 2;
    let id = held(&mut g, "WM040", 0, 0);
    fund(&mut g, 0, "JC059", 7);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    choose(&mut g, vec!["p1".into()]);
    pass_top(&mut g);
    let p = g.pending.clone().unwrap();
    assert_eq!((p.seat, p.choice.min), (1, Some(3)));
    assert!(g.players.iter().all(|p| p.society_zone.exposure == 0));
    assert_eq!(g.players[3].society_zone.secrecy, 2);
    let mut changed = g.clone();
    if let ChoiceResolution::Frame {
        choice: FrameChoice::WM040Repress { removed, .. },
        ..
    } = &mut changed.pending.as_mut().unwrap().resolution
    {
        removed[0].1 = 0;
    }
    bad(&g, changed);
    let ids = p
        .choice
        .options
        .iter()
        .take(3)
        .map(|o| o.id.clone())
        .collect();
    choose(&mut g, ids);
    assert_eq!(g.regions.iter().map(|r| r.influence[1]).sum::<u32>(), 1);
    let own = board(&mut g, "JC125", 0, 2);
    let teammate = board(&mut g, "JC125", 2, 2);
    let away = board(&mut g, "JC125", 0, 1);
    g.board_mut(&own).unwrap().wounds = 1;
    g.board_mut(&teammate).unwrap().wounds = 1;
    g.board_mut(&away).unwrap().wounds = 1;
    let source = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "WM040")
        .unwrap()
        .id
        .clone();
    g.begin_window(Window::Action(0));
    activate(&mut g, 0, &source, "wm040-heal");
    pass_top(&mut g);
    assert_eq!(g.board(&own).unwrap().1.wounds, 0);
    assert_eq!(g.board(&teammate).unwrap().1.wounds, 1);
    assert_eq!(g.board(&away).unwrap().1.wounds, 1);
    assert_eq!(g.society_exposure(0), 1);
    assert!(g.board(&source).unwrap().1.exhausted);
}
#[test]
fn exposure_rumor_mill_owner_choice_before_grave_return_and_tamper_rejection() {
    let mut g = game(0);
    let host = board(&mut g, "JC125", 0, 2);
    let rumor = g.make_card("WM042", 2);
    let old = rumor.id.clone();
    let next = g.make_card("JC125", 1);
    g.players[1].deck = vec![rumor, next];
    mill_source(&mut g, 1);
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    assert_eq!(g.players[1].deck[0].id, old);
    assert!(g.players[2].graveyard.is_empty());
    for s in [0, 1, 3] {
        assert!(g.view(s).pending_choice.is_none());
    }
    let stale = Action {
        choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
        selected: Some(vec![host.clone()]),
        ..Action::new("choose")
    };
    reject(&mut g, 1, stale.clone());
    let mut changed = g.clone();
    changed.pending.as_mut().unwrap().seat = 1;
    bad(&g, changed);
    let mut changed = g.clone();
    if let ChoiceResolution::Rumor { batch } = &mut changed.pending.as_mut().unwrap().resolution {
        batch.cursor = 1;
    }
    bad(&g, changed);
    choose(&mut g, vec![host.clone()]);
    assert!(g.players[2].graveyard.is_empty());
    assert_eq!(g.players[1].graveyard.len(), 1);
    assert_eq!(g.sealed_cards.len(), 1);
    assert_ne!(g.sealed_cards[0].card.id, old);
    reject(&mut g, 2, stale);
    g.return_hand(&host);
    assert!(g.sealed_cards.is_empty());
    assert!(g.players[2].hand.iter().any(|c| c.definition == "WM042"));
    checkpoint(&g);
}
#[test]
fn exposure_rumor_decline_two_rumors_resume_without_duplicate_and_no_host() {
    let mut g = game(0);
    let host = board(&mut g, "JC125", 0, 2);
    let a = g.make_card("WM042", 1);
    let b = g.make_card("WM042", 2);
    g.players[1].deck = vec![a, b];
    mill_source(&mut g, 1);
    choose(&mut g, vec![]);
    assert_eq!(g.players[1].graveyard.len(), 1);
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    checkpoint(&g);
    choose(&mut g, vec![host]);
    assert_eq!(g.players[1].graveyard.len(), 1);
    assert_eq!(g.players[2].graveyard.len(), 0);
    assert_eq!(g.sealed_cards.len(), 1);
    let mut g = game(0);
    let rumor = g.make_card("WM042", 1);
    g.players[1].deck = vec![rumor];
    // No host exists in this direct primitive fixture; a real mill source would itself be a host.
    g.mill_cards(1, 1);
    assert!(g.pending.is_none());
    assert_eq!(g.players[1].graveyard.len(), 1);
}
#[test]
fn exposure_rumor_effect_discard_batches_count_only_actual_discard_and_continue_draw() {
    let mut g = game(0);
    let host = board(&mut g, "JC125", 0, 2);
    let observer = board(&mut g, "JC040", 0, 1);
    let source = board(&mut g, "JC038", 0, 2);
    let victim = board(&mut g, "JC027", 0, 2);
    let rumor = held(&mut g, "WM042", 2, 1);
    let before = g.players[0].hand.len();
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            ability_id: Some("batch20-jc038-discard-draw".into()),
            target_id: Some("p1".into()),
            cost_selected: Some(vec![victim]),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    choose(&mut g, vec![rumor]);
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    assert_eq!(g.players[0].hand.len(), before);
    choose(&mut g, vec![host]);
    assert_eq!(g.players[0].hand.len(), before + 1);
    assert!(!g
        .effects
        .iter()
        .any(|e| matches!(e,Effect::Declare{declaration} if declaration.source.card.id==observer)));
    assert!(g.players[2].graveyard.is_empty());
}
#[test]
fn exposure_normal_spell_burial_cleanup_and_cost_discard_do_not_trigger_rumor() {
    let mut g = game(0);
    board(&mut g, "JC125", 0, 2);
    news(&mut g, 0, "expose");
    assert!(g.pending.is_none());
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "WM042")
            .count(),
        1
    );
    g.begin_window(Window::Action(0));
    let rumor = held(&mut g, "WM042", 0, 0);
    for _ in 0..7 {
        held(&mut g, "JC125", 0, 0);
    }
    g.effect(Effect::Cleanup).unwrap();
    g.drive().unwrap();
    let p = g.pending.clone().unwrap();
    assert_eq!(p.choice.min, Some(2));
    let second = p
        .choice
        .options
        .iter()
        .find(|o| o.id != rumor)
        .unwrap()
        .id
        .clone();
    choose(&mut g, vec![rumor, second]);
    assert!(g.pending.as_ref().is_none_or(|p| p.choice.kind != "rumor"));
    let mut g = game(0);
    board(&mut g, "JC125", 0, 2);
    let id = held(&mut g, "WM042", 0, 0);
    g.discard_hand_card(0, &id).unwrap();
    assert!(g.pending.is_none());
    assert_eq!(g.players[0].graveyard.len(), 1);
}
