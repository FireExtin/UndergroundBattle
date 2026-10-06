//! Explicit initial layouts/funding, followed by actual paid card commands,
//! response stacks and windows. Every checkpoint restores exact state/views.
use crate::{catalog, model::*, rules::*};

fn game() -> Game {
    let mut g = Game::new(
        "protection-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        8,
    )
    .unwrap();
    for s in 1..4 {
        g.join(format!("P{s}"), "watchers".into()).unwrap();
    }
    for p in &mut g.players {
        p.ready = true;
    }
    g.apply(0, Action::new("start")).unwrap();
    while g.pending.is_some() {
        choose(&mut g, vec![]);
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
    }
    for r in &mut g.regions {
        r.cards.clear();
        r.influence = [0; 2];
    }
    g.regions[0].card = g.make_card("DQJC115", 0);
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    g
}
fn field(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn held(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn fund(g: &mut Game, seat: usize, def: &str, count: usize) {
    for _ in 0..count {
        let c = g.make_card(def, seat);
        g.players[seat].assets.push(c);
    }
}
fn mirror(g: &mut Game) {
    let state = serde_json::to_string(g).unwrap();
    let views = (0..4)
        .map(|s| serde_json::to_value(g.view(s)).unwrap())
        .collect::<Vec<_>>();
    *g = Game::from_persisted(&state).unwrap();
    assert_eq!(serde_json::to_string(g).unwrap(), state);
    for s in 0..4 {
        assert_eq!(serde_json::to_value(g.view(s)).unwrap(), views[s]);
    }
}
fn choose(g: &mut Game, selected: Vec<String>) {
    mirror(g);
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
    mirror(g);
}
fn pass(g: &mut Game) {
    mirror(g);
    let seat = (0..4)
        .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
        .unwrap();
    g.apply(seat, Action::new("pass")).unwrap();
    mirror(g);
}
fn pass_top(g: &mut Game) {
    let n = g.stack.len();
    assert!(n > 0);
    for _ in 0..32 {
        if g.stack.len() < n {
            return;
        }
        pass(g);
    }
    panic!("bounded protection stack");
}
fn give_priority(g: &mut Game, seat: usize) {
    while g.priority_team != g.team(seat) {
        pass(g);
    }
}
fn cast(g: &mut Game, def: &str, seat: usize, target: &str) {
    let source = held(g, def, seat);
    g.apply(
        seat,
        Action {
            card_id: Some(source),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    )
    .unwrap();
    pass_top(g);
}

fn activate(g: &mut Game, source: &str, target: &str) {
    g.apply(
        0,
        Action {
            card_id: Some(source.into()),
            target_id: Some(target.into()),
            ability_id: Some("protect-local-character".into()),
            ..Action::new("activate")
        },
    )
    .unwrap();
}
fn rejected(g: &mut Game, seat: usize, a: Action) {
    let before = serde_json::to_string(g).unwrap();
    assert!(g.apply(seat, a).is_err());
    assert_eq!(serde_json::to_string(g).unwrap(), before);
    mirror(g);
}
fn attached(g: &mut Game, host: &str) -> String {
    fund(g, 0, "JC075", 3);
    cast(g, "JC073", 0, host);
    g.attachments.last().unwrap().card.id.clone()
}
fn to_next_turn(g: &mut Game) {
    let turn = g.turn;
    for _ in 0..600 {
        if g.turn > turn {
            return;
        }
        if let Some(p) = &g.pending {
            let ids = p
                .choice
                .options
                .iter()
                .take(p.choice.min.unwrap_or(0))
                .map(|o| o.id.clone())
                .collect();
            choose(g, ids);
        } else {
            pass(g);
        }
    }
    panic!("bounded turn advance");
}

#[test]
fn protection_original_fields_and_only_four_admitted_cards() {
    assert_eq!(catalog::catalog().cards.len(), 99);
    let hermit = catalog::card("JC071");
    assert_eq!(
        (hermit.cost, hermit.defense, hermit.magic.as_str()),
        (2, Some(1), "星辰")
    );
    assert_eq!(hermit.loyalty, ["白色"]);
    assert_eq!(hermit.permanent_icons.influence, 1);
    let net = catalog::card("JC073");
    assert_eq!((net.cost, net.defense), (3, None));
    assert_eq!(net.subtypes, ["装备", "护身符"]);
    assert_eq!(net.loyalty, ["白色"]);
    let fire = catalog::card("JC102");
    assert_eq!(fire.cost, 2);
    assert_eq!(fire.loyalty, ["紫色"]);
    assert_eq!(fire.magic_icon, MagicIcon::Blood);
    assert_eq!(fire.subtypes, ["法术", "阴"]);
    let wall = catalog::card("JC132");
    assert_eq!(
        (wall.cost, wall.color.as_str(), wall.magic.as_str()),
        (4, "中立", "心灵")
    );
    assert_eq!(wall.loyalty, ["黄色"]);
    assert_eq!(wall.subtypes, ["法术", "空间"]);
    for id in ["JC071", "JC073", "JC102", "JC132"] {
        for ability in &definition(id).abilities {
            validate_ability(id, ability).unwrap();
        }
    }
}
#[test]
fn protection_hermit_requires_other_same_region_current_controller() {
    let mut g = game();
    let source = field(&mut g, "JC071", 0);
    let target = field(&mut g, "JC125", 2);
    g.board_mut(&target).unwrap().controller = 0; // explicit initial control checkpoint
    let mate = field(&mut g, "JC125", 1);
    let enemy = field(&mut g, "JC125", 2);
    let outside = field(&mut g, "JC125", 0);
    let c = g.remove_board(&outside).unwrap().1;
    g.regions[1].cards.push(c);
    for illegal in [&source, &mate, &enemy, &outside] {
        rejected(
            &mut g,
            0,
            Action {
                card_id: Some(source.clone()),
                target_id: Some(illegal.clone()),
                ability_id: Some("protect-local-character".into()),
                ..Action::new("activate")
            },
        );
    }
    activate(&mut g, &source, &target);
    assert!(g.board(&source).unwrap().1.exhausted);
    pass_top(&mut g);
    assert_eq!(g.defense(g.board(&target).unwrap().1, 0), 2);
    assert_eq!(g.turn_attribute_modifiers[0].target_instance, target);
    for seat in 0..4 {
        let c = g.view(seat).regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == target)
            .unwrap()
            .clone();
        assert_eq!(
            (c.owner.as_str(), c.controller.as_str(), c.defense),
            ("p2", "p0", Some(2))
        );
    }
}
#[test]
fn protection_hermit_exhausted_and_hidden_target_rejections_are_atomic() {
    let mut g = game();
    let source = field(&mut g, "JC071", 0);
    let target = field(&mut g, "JC125", 0);
    g.board_mut(&target).unwrap().face_down = true;
    let a = Action {
        card_id: Some(source.clone()),
        target_id: Some(target.clone()),
        ability_id: Some("protect-local-character".into()),
        ..Action::new("activate")
    };
    rejected(&mut g, 0, a.clone());
    g.board_mut(&target).unwrap().face_down = false;
    g.board_mut(&source).unwrap().exhausted = true;
    rejected(&mut g, 0, a);
}
#[test]
fn protection_declared_hermit_ability_survives_source_return() {
    let mut g = game();
    let source = field(&mut g, "JC071", 0);
    let target = field(&mut g, "JC125", 0);
    fund(&mut g, 2, "JC002", 2);
    activate(&mut g, &source, &target);
    give_priority(&mut g, 2);
    cast(&mut g, "JC006", 2, &source);
    assert!(g.board(&source).is_none());
    pass_top(&mut g);
    assert_eq!(g.defense(g.board(&target).unwrap().1, 0), 2);
}
#[test]
fn protection_hermit_response_return_cancels_target_without_refunding_exhaustion() {
    let mut g = game();
    let source = field(&mut g, "JC071", 0);
    let target = field(&mut g, "JC003", 0);
    fund(&mut g, 2, "JC002", 2);
    activate(&mut g, &source, &target);
    give_priority(&mut g, 2);
    cast(&mut g, "JC006", 2, &target);
    pass_top(&mut g);
    assert!(g.turn_attribute_modifiers.is_empty());
    assert!(g.board(&source).unwrap().1.exhausted);
}
#[test]
fn protection_temporary_defense_and_damage_cleanup_preserve_survivor() {
    let mut g = game();
    let source = field(&mut g, "JC071", 0);
    let target = field(&mut g, "JC125", 0);
    fund(&mut g, 2, "JC104", 2);
    activate(&mut g, &source, &target);
    pass_top(&mut g);
    give_priority(&mut g, 2);
    cast(&mut g, "JC102", 2, &target);
    assert_eq!(g.board(&target).unwrap().1.damage, 1);
    assert_eq!(g.defense(g.board(&target).unwrap().1, 0), 2);
    to_next_turn(&mut g);
    assert!(g.turn_attribute_modifiers.is_empty());
    assert_eq!(g.board(&target).unwrap().1.damage, 0);
    assert_eq!(g.defense(g.board(&target).unwrap().1, 0), 1);
    mirror(&mut g);
}
#[test]
fn protection_dreamcatcher_temporary_icon_follows_host_initiative_not_attachment_controller() {
    let mut g = game();
    let host = field(&mut g, "JC125", 2);
    attached(&mut g, &host);
    let c = g.board(&host).unwrap().1;
    assert_eq!(g.current_icons(c, 0).investigation, 0);
    assert!(g.has_barrier(c));
    assert!(!g.targetable(0, c) && !g.targetable(1, c));
    assert!(g.targetable(2, c) && g.targetable(3, c));
    g.first_team = 1; // explicit initiative checkpoint
    assert_eq!(
        g.current_icons(g.board(&host).unwrap().1, 0).investigation,
        1
    );
    mirror(&mut g);
}
#[test]
fn protection_dreamcatcher_all_seats_public_projection_and_attachment_removal() {
    let mut g = game();
    let host = field(&mut g, "JC125", 0);
    let net = attached(&mut g, &host);
    for seat in 0..4 {
        let c = g.view(seat).regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == host)
            .unwrap()
            .clone();
        assert_eq!(c.current_barrier, Some(true));
        assert_eq!(c.icons.unwrap().investigation, 1);
    }
    fund(&mut g, 0, "JC002", 2);
    cast(&mut g, "JC005", 0, &net);
    assert!(g.attachments.is_empty());
    assert!(!g.has_barrier(g.board(&host).unwrap().1));
    assert_eq!(
        g.current_icons(g.board(&host).unwrap().1, 0).investigation,
        0
    );
}
#[test]
fn protection_dreamcatcher_equipment_rejects_forbidden_host_and_bad_loyalty_atomically() {
    let mut g = game();
    let host = field(&mut g, "JC001", 0);
    fund(&mut g, 0, "JC075", 3);
    let spell = held(&mut g, "JC073", 0);
    rejected(
        &mut g,
        0,
        Action {
            card_id: Some(spell.clone()),
            target_id: Some(host),
            ..Action::new("play")
        },
    );
    let host = field(&mut g, "JC125", 0);
    g.players[0].assets.clear();
    fund(&mut g, 0, "JC125", 3);
    rejected(
        &mut g,
        0,
        Action {
            card_id: Some(spell),
            target_id: Some(host),
            ..Action::new("play")
        },
    );
}
#[test]
fn protection_hide_host_cleans_equipment_and_all_private_flags() {
    let mut g = game();
    let host = field(&mut g, "JC125", 0);
    attached(&mut g, &host);
    fund(&mut g, 0, "JC063", 3);
    let spell = held(&mut g, "JC063", 0);
    g.apply(
        0,
        Action {
            card_id: Some(spell),
            target_id: Some(host.clone()),
            option: Some("hide".into()),
            ..Action::new("play")
        },
    )
    .unwrap();
    pass_top(&mut g);
    assert!(g.attachments.is_empty());
    for seat in 0..4 {
        let c = g.view(seat).regions[0]
            .characters
            .iter()
            .find(|c| c.face_down)
            .unwrap()
            .clone();
        assert_eq!(c.current_barrier, None);
        if seat != 0 {
            assert!(c.card_id.is_none() && c.icons.is_none());
        }
    }
    mirror(&mut g);
}
#[test]
fn protection_faerie_fire_action_phase_restriction_with_purple_loyalty() {
    let mut g = game();
    let target = field(&mut g, "JC125", 2);
    let spell = held(&mut g, "JC102", 0);
    fund(&mut g, 0, "JC104", 2);
    let a = Action {
        card_id: Some(spell),
        target_id: Some(target.clone()),
        ..Action::new("play")
    };
    g.begin_window(Window::Before(0, 0));
    rejected(&mut g, 0, a.clone());
    g.begin_window(Window::Action(0));
    g.apply(0, a).unwrap();
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125"));
}
#[test]
fn protection_faerie_fire_blood_without_purple_rejects_atomically_then_real_asset_enables_payment()
{
    let mut g = game();
    let target = field(&mut g, "JC125", 2);
    let spell = held(&mut g, "JC102", 0);
    let purple_asset = held(&mut g, "JC104", 0);
    fund(&mut g, 0, "JC002", 2);
    assert_eq!(g.resources(0), 2);
    assert!(g.players[0].assets.iter().all(|c| {
        catalog::card(&c.definition).magic_icon == MagicIcon::Blood
            && catalog::card(&c.definition).color != "紫"
    }));
    let action = Action {
        card_id: Some(spell),
        target_id: Some(target.clone()),
        ..Action::new("play")
    };
    let before = serde_json::to_string(&g).unwrap();
    let views = (0..4)
        .map(|s| serde_json::to_value(g.view(s)).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(g.apply(0, action.clone()).unwrap_err(), "忠诚不足");
    assert_eq!(serde_json::to_string(&g).unwrap(), before);
    for s in 0..4 {
        assert_eq!(serde_json::to_value(g.view(s)).unwrap(), views[s]);
    }
    mirror(&mut g);
    g.apply(
        0,
        Action {
            card_id: Some(purple_asset),
            ..Action::new("asset")
        },
    )
    .unwrap();
    assert_eq!(g.resources(0), 3);
    g.apply(0, action).unwrap();
    assert_eq!(g.resources(0), 1);
    assert_eq!(
        g.players[0].assets.iter().filter(|c| c.exhausted).count(),
        2
    );
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    mirror(&mut g);
}
#[test]
fn protection_faerie_fire_shield_cancels_damage_and_keeps_paid_costs() {
    let mut g = game();
    let target = field(&mut g, "JC125", 2);
    g.board_mut(&target).unwrap().shield = 1; // explicit shield starting checkpoint
    fund(&mut g, 0, "JC104", 2);
    cast(&mut g, "JC102", 0, &target);
    let c = g.board(&target).unwrap().1;
    assert_eq!((c.damage, c.shield), (0, 0));
    assert_eq!(
        g.players[0].assets.iter().filter(|c| c.exhausted).count(),
        2
    );
}
#[test]
fn protection_faerie_fire_cancels_when_response_return_changes_instance() {
    let mut g = game();
    let target = field(&mut g, "JC003", 2);
    fund(&mut g, 0, "JC104", 2);
    fund(&mut g, 2, "JC002", 2);
    let spell = held(&mut g, "JC102", 0);
    g.apply(
        0,
        Action {
            card_id: Some(spell),
            target_id: Some(target.clone()),
            ..Action::new("play")
        },
    )
    .unwrap();
    give_priority(&mut g, 2);
    cast(&mut g, "JC006", 2, &target);
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    assert!(!g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC003"));
    assert!(g.players[2]
        .hand
        .iter()
        .any(|c| c.definition == "JC003" && c.id != target));
}
#[test]
fn protection_region_exhaustion_ignores_character_barrier_and_skips_hidden_other_regions() {
    let mut g = game();
    let first = field(&mut g, "JC125", 0);
    let barrier = field(&mut g, "JZ08", 2);
    let hidden = field(&mut g, "JC125", 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    let outside = field(&mut g, "JC125", 2);
    let c = g.remove_board(&outside).unwrap().1;
    g.regions[1].cards.push(c);
    fund(&mut g, 0, "JC002", 4);
    let spell = held(&mut g, "JC132", 0);
    g.apply(
        0,
        Action {
            card_id: Some(spell),
            region: Some(0),
            ..Action::new("play")
        },
    )
    .unwrap();
    pass_top(&mut g);
    assert!(g.board(&first).unwrap().1.exhausted && g.board(&barrier).unwrap().1.exhausted);
    assert!(!g.board(&hidden).unwrap().1.exhausted && !g.board(&outside).unwrap().1.exhausted);
}
#[test]
fn protection_neutral_wall_requires_yellow_loyalty_and_standard_timing() {
    let mut g = game();
    fund(&mut g, 0, "JC125", 4);
    let spell = held(&mut g, "JC132", 0);
    let a = Action {
        card_id: Some(spell),
        region: Some(0),
        ..Action::new("play")
    };
    rejected(&mut g, 0, a.clone());
    let c = g.make_card("JC002", 0);
    g.players[0].assets[0] = c;
    g.begin_window(Window::Before(0, 0));
    rejected(&mut g, 0, a);
}
#[test]
fn protection_damage_program_rejects_unbound_or_hidden_targets() {
    let mut spec = definition("JC102").abilities[0].clone();
    spec.targets.clear();
    assert!(validate_ability("JC102", &spec).is_err());
    spec = definition("JC102").abilities[0].clone();
    spec.targets[0].kind = EntityKind::Hidden;
    assert!(validate_ability("JC102", &spec).is_err());
}

#[test]
fn protection_multiple_equipment_grants_add_temporary_icons_but_keep_one_barrier() {
    let mut g = game();
    let host = field(&mut g, "JC125", 0);
    let first = attached(&mut g, &host);
    attached(&mut g, &host);
    assert_eq!(
        g.current_icons(g.board(&host).unwrap().1, 0).investigation,
        2
    );
    fund(&mut g, 0, "JC002", 2);
    cast(&mut g, "JC005", 0, &first);
    assert_eq!(
        g.current_icons(g.board(&host).unwrap().1, 0).investigation,
        1
    );
    assert!(g.has_barrier(g.board(&host).unwrap().1));
    mirror(&mut g);
}

#[test]
fn protection_faerie_fire_is_a_real_paid_fast_response_during_action_phase() {
    let mut g = game();
    let target = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "JC002", 2);
    fund(&mut g, 2, "JC104", 2);
    let spell = held(&mut g, "JC008", 0);
    g.apply(
        0,
        Action {
            card_id: Some(spell),
            target_id: Some(target.clone()),
            ..Action::new("play")
        },
    )
    .unwrap();
    give_priority(&mut g, 2);
    cast(&mut g, "JC102", 2, &target);
    assert!(g.board(&target).is_none());
    pass_top(&mut g);
    assert!(g.turn_attribute_modifiers.is_empty());
    assert_eq!(
        g.players[2].assets.iter().filter(|c| c.exhausted).count(),
        2
    );
    mirror(&mut g);
}
