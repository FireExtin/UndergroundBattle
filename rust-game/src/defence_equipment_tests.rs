//! Explicit starting layouts; declarations, costs, responses and restoration use real actions.
use crate::{catalog, model::*, rules::*};

fn game() -> Game {
    let mut g = Game::new(
        "defence-unit".into(),
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
        p.graveyard.clear();
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
fn field(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn held(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn fund(g: &mut Game, seat: usize, id: &str, n: usize) {
    for _ in 0..n {
        let c = g.make_card(id, seat);
        g.players[seat].assets.push(c);
    }
}
fn restore(g: &mut Game) {
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
fn act(g: &mut Game, s: usize, a: Action) {
    restore(g);
    g.apply(s, a).unwrap();
    restore(g);
}
fn reject(g: &mut Game, s: usize, a: Action) {
    let before = serde_json::to_string(g).unwrap();
    assert!(g.apply(s, a).is_err());
    assert_eq!(serde_json::to_string(g).unwrap(), before);
    restore(g);
}
fn choose(g: &mut Game, ids: Vec<String>) {
    let p = g.pending.clone().unwrap();
    act(
        g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(ids),
            ..Action::new("choose")
        },
    );
}
fn pass(g: &mut Game) {
    let s = (0..4)
        .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
        .unwrap();
    act(g, s, Action::new("pass"));
}
fn top(g: &mut Game) {
    let n = g.stack.len();
    assert!(n > 0);
    for _ in 0..64 {
        if g.pending.is_some() {
            resolve_choice(g);
        } else if g.stack.len() < n {
            return;
        } else {
            pass(g);
        }
    }
    panic!("bounded stack");
}
fn resolve_choice(g: &mut Game) {
    let p = g.pending.clone().unwrap();
    if matches!(
        p.resolution,
        ChoiceResolution::Forecast { .. }
            | ChoiceResolution::Frame {
                choice: FrameChoice::Forecast { .. },
                ..
            }
    ) {
        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect();
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                top: Some(ids),
                bottom: Some(vec![]),
                ..Action::new("choose")
            },
        );
    } else if matches!(p.resolution, ChoiceResolution::Bottom { .. }) {
        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect();
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                bottom: Some(ids),
                ..Action::new("choose")
            },
        );
    } else if matches!(p.resolution, ChoiceResolution::Damage { .. }) {
        let mut allocations = std::collections::BTreeMap::new();
        allocations.insert(p.choice.options[0].id.clone(), p.choice.amount.unwrap());
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                allocations: Some(allocations),
                ..Action::new("choose")
            },
        );
    } else {
        let ids = p
            .choice
            .options
            .iter()
            .take(p.choice.min.unwrap_or(0))
            .map(|o| o.id.clone())
            .collect();
        choose(g, ids);
    }
}
fn priority(g: &mut Game, s: usize) {
    while g.priority_team != g.team(s) {
        pass(g);
    }
}
fn advance(g: &mut Game, done: impl Fn(&Game) -> bool) {
    for _ in 0..650 {
        if done(g) {
            return;
        }
        if g.pending.is_some() {
            resolve_choice(g);
        } else {
            pass(g);
        }
    }
    panic!("bounded turn");
}
fn play(g: &mut Game, source: &str, s: usize, target: &str) {
    act(
        g,
        s,
        Action {
            card_id: Some(source.into()),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    );
    top(g);
}
fn cast(g: &mut Game, id: &str, s: usize, target: &str) {
    let source = held(g, id, s);
    play(g, &source, s, target);
}
fn attach(g: &mut Game, id: &str, target: &str) -> String {
    cast(g, id, 0, target);
    g.attachments.last().unwrap().card.id.clone()
}
fn transfer(source: &str, target: &str) -> Action {
    Action {
        card_id: Some(source.into()),
        target_id: Some(target.into()),
        ability_id: Some("reattach".into()),
        ..Action::new("activate")
    }
}
fn cat(g: &mut Game) -> String {
    let source = held(g, "JC112", 0);
    act(
        g,
        0,
        Action {
            card_id: Some(source),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    top(g);
    g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC112")
        .unwrap()
        .id
        .clone()
}
fn reduce(g: &mut Game, source: &str) {
    act(
        g,
        0,
        Action {
            card_id: Some(source.into()),
            ability_id: Some("reduce-next-magic".into()),
            ..Action::new("activate")
        },
    );
    assert!(g.stack.is_empty());
}

#[test]
fn defence_original_fields_and_finite_bindings() {
    assert_eq!(catalog::catalog().cards.len(), 99);
    let shield = catalog::card("JC078");
    assert_eq!(shield.cost, 1);
    assert_eq!(shield.loyalty, ["白色"]);
    assert_eq!(shield.magic, "神圣");
    let blood = catalog::card("XQ14");
    assert_eq!(blood.cost, 1);
    assert_eq!(blood.loyalty, ["蓝色"]);
    assert_eq!(blood.magic_icon, MagicIcon::Blood);
    assert_eq!(blood.subtypes, ["状态"]);
    let vest = catalog::card("XQ47");
    assert_eq!(vest.cost, 1);
    assert!(vest.loyalty.is_empty());
    assert_eq!(vest.subtypes, ["装备", "防具"]);
    let cat = catalog::card("JC112");
    assert_eq!((cat.cost, cat.defense), (1, Some(1)));
    assert_eq!(cat.permanent_icons.influence, 1);
    assert!(cat.rule_traits.public && cat.rule_traits.cannot_be_equipped);
    for id in ["JC078", "XQ14", "XQ47", "JC112"] {
        for a in &definition(id).abilities {
            validate_ability(id, a).unwrap();
        }
    }
}
#[test]
fn defence_shield_prevents_repeated_paid_damage_and_does_not_heal_old_damage() {
    let mut g = game();
    let target = field(&mut g, "JZ27", 0);
    g.board_mut(&target).unwrap().damage = 1;
    g.board_mut(&target).unwrap().wounds = 1;
    fund(&mut g, 0, "JC075", 1);
    fund(&mut g, 0, "JC104", 4);
    cast(&mut g, "JC078", 0, &target);
    for _ in 0..2 {
        cast(&mut g, "JC102", 0, &target);
        assert_eq!(g.board(&target).unwrap().1.damage, 1);
    }
    assert_eq!(g.resources(0), 0);
    for s in 0..4 {
        assert_eq!(
            g.view(s).regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == target)
                .unwrap()
                .current_damage_prevention,
            Some(true)
        );
    }
}
#[test]
fn defence_shield_prevents_nontarget_region_damage_only_for_protected_character() {
    let mut g = game();
    let target = field(&mut g, "JC003", 0);
    let other = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "JC075", 1);
    fund(&mut g, 0, "JC042", 3);
    cast(&mut g, "JC078", 0, &target);
    let source = held(&mut g, "JC047", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            region: Some(0),
            ..Action::new("play")
        },
    );
    top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.damage, 0);
    assert!(g.board(&other).is_none());
}
#[test]
fn defence_shield_expires_at_cleanup_and_does_not_stop_destroy() {
    let mut g = game();
    let target = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "JC075", 1);
    fund(&mut g, 2, "JC084", 6);
    cast(&mut g, "JC078", 0, &target);
    priority(&mut g, 2);
    cast(&mut g, "JC091", 2, &target);
    assert!(g.board(&target).is_none());
    assert!(g.turn_attribute_modifiers.is_empty());
    let mut g = game();
    let target = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "JC075", 1);
    cast(&mut g, "JC078", 0, &target);
    let turn = g.turn;
    advance(&mut g, |g| g.turn > turn);
    assert!(!g.damage_prevented(g.board(&target).unwrap().1));
    assert!(g.turn_attribute_modifiers.is_empty());
}
#[test]
fn defence_shield_response_return_cancels_old_instance_without_refund() {
    let mut g = game();
    let target = field(&mut g, "JC003", 2);
    fund(&mut g, 0, "JC075", 1);
    fund(&mut g, 2, "JC002", 2);
    let source = held(&mut g, "JC078", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            target_id: Some(target.clone()),
            ..Action::new("play")
        },
    );
    priority(&mut g, 2);
    cast(&mut g, "JC006", 2, &target);
    top(&mut g);
    assert!(g.turn_attribute_modifiers.is_empty());
    assert_eq!(g.resources(0), 0);
    assert!(g.players[2]
        .hand
        .iter()
        .any(|c| c.definition == "JC003" && c.id != target));
}
#[test]
fn defence_shield_hidden_target_and_stale_projection_remain_private() {
    let mut g = game();
    let target = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "JC075", 1);
    fund(&mut g, 0, "JC063", 3);
    cast(&mut g, "JC078", 0, &target);
    let source = held(&mut g, "JC063", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            target_id: Some(target),
            option: Some("hide".into()),
            ..Action::new("play")
        },
    );
    top(&mut g);
    assert!(g.turn_attribute_modifiers.is_empty());
    for s in 0..4 {
        let c = g.view(s).regions[0].characters[0].clone();
        assert_eq!(c.current_damage_prevention, None);
        if s != 0 {
            assert!(c.card_id.is_none());
        }
    }
}
#[test]
fn defence_blood_requires_current_vampire_and_grants_stacking_public_stats() {
    let mut g = game();
    let vampire = field(&mut g, "JZ27", 2);
    let human = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "XQ16", 3);
    let source = held(&mut g, "XQ14", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(source.clone()),
            target_id: Some(human),
            ..Action::new("play")
        },
    );
    play(&mut g, &source, 0, &vampire);
    attach(&mut g, "XQ14", &vampire);
    for s in 0..4 {
        let v = g.view(s);
        let c = v.regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == vampire)
            .unwrap();
        assert_eq!(c.defense, Some(5));
        assert_eq!(c.icons.unwrap().combat, 4);
    }
}
#[test]
fn defence_blood_accepts_gained_vampire_type_and_leaves_when_that_type_is_lost() {
    let mut g = game();
    let target = field(&mut g, "JC125", 2);
    fund(&mut g, 0, "XQ16", 7);
    fund(&mut g, 0, "JC002", 2);
    let control = attach(&mut g, "JC036", &target);
    attach(&mut g, "XQ14", &target);
    assert_eq!(g.defense(g.board(&target).unwrap().1, 0), 2);
    cast(&mut g, "JC005", 0, &control);
    assert!(g.attachments.is_empty());
    assert_eq!(g.defense(g.board(&target).unwrap().1, 0), 1);
    assert_eq!(g.board(&target).unwrap().1.controller, 2);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ14"));
}
#[test]
fn defence_vest_transfer_pays_two_keeps_instance_and_settles_lost_defense_death() {
    let mut g = game();
    let first = field(&mut g, "JC125", 2);
    let next = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "JC104", 5);
    let vest = attach(&mut g, "XQ47", &first);
    cast(&mut g, "JC102", 0, &first);
    assert_eq!(g.board(&first).unwrap().1.damage, 1);
    act(&mut g, 0, transfer(&vest, &next));
    top(&mut g);
    assert!(g.board(&first).is_none());
    assert_eq!(g.attachments[0].card.id, vest);
    assert_eq!(g.attachments[0].host_id, next);
    assert_eq!(g.defense(g.board(&next).unwrap().1, 0), 2);
    assert_eq!(g.resources(0), 0);
}
#[test]
fn defence_vest_rejects_current_host_hidden_cat_enemy_barrier_and_response_timing() {
    let mut g = game();
    let first = field(&mut g, "JC125", 0);
    let hidden = field(&mut g, "JC125", 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    let cat = field(&mut g, "JC112", 0);
    let barrier = field(&mut g, "JZ08", 2);
    fund(&mut g, 0, "JC125", 4);
    let vest = attach(&mut g, "XQ47", &first);
    for target in [&first, &hidden, &cat, &barrier, &vest] {
        reject(&mut g, 0, transfer(&vest, target));
    }
    let next = field(&mut g, "JC125", 0);
    g.begin_window(Window::Before(0, 0));
    reject(&mut g, 0, transfer(&vest, &next));
}
#[test]
fn defence_vest_destination_return_response_cancels_and_keeps_original_host() {
    let mut g = game();
    let first = field(&mut g, "JC125", 0);
    let next = field(&mut g, "JC003", 2);
    fund(&mut g, 0, "JC125", 3);
    fund(&mut g, 2, "JC002", 2);
    let vest = attach(&mut g, "XQ47", &first);
    act(&mut g, 0, transfer(&vest, &next));
    priority(&mut g, 2);
    cast(&mut g, "JC006", 2, &next);
    top(&mut g);
    assert_eq!(g.attachments[0].host_id, first);
    assert_eq!(g.resources(0), 0);
    assert!(g.board(&next).is_none());
}
#[test]
fn defence_vest_source_destroy_response_does_not_create_another_equipment() {
    let mut g = game();
    let first = field(&mut g, "JC125", 0);
    let next = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "JC125", 3);
    fund(&mut g, 2, "JC002", 2);
    let vest = attach(&mut g, "XQ47", &first);
    act(&mut g, 0, transfer(&vest, &next));
    priority(&mut g, 2);
    cast(&mut g, "JC005", 2, &vest);
    top(&mut g);
    assert!(g.attachments.is_empty());
    assert_eq!(g.resources(0), 0);
    assert_eq!(g.defense(g.board(&next).unwrap().1, 0), 1);
}
#[test]
fn defence_vest_cross_region_transfer_moves_public_attachment_view_only() {
    let mut g = game();
    let first = field(&mut g, "JC125", 2);
    let next = field(&mut g, "JC125", 0);
    let c = g.remove_board(&next).unwrap().1;
    g.regions[1].cards.push(c);
    fund(&mut g, 0, "JC125", 3);
    let vest = attach(&mut g, "XQ47", &first);
    act(&mut g, 0, transfer(&vest, &next));
    top(&mut g);
    for s in 0..4 {
        let v = g.view(s);
        assert_eq!(v.attachments[0].card.region, Some(1));
        assert_eq!(v.attachments[0].card.instance_id, vest);
        assert_eq!(v.attachments[0].card.owner, "p0");
    }
}
#[test]
fn defence_cat_paid_deployment_and_immediate_reduction_survive_restore() {
    let mut g = game();
    fund(&mut g, 0, "JC075", 2);
    let source = cat(&mut g);
    assert_eq!(g.resources(0), 1);
    reduce(&mut g, &source);
    assert!(g.board(&source).is_none());
    assert_eq!(g.modifiers.len(), 1);
    assert_eq!(g.modifiers[0].uses, 1);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC112" && c.id != source));
    restore(&mut g);
    assert_eq!(g.modifiers.len(), 1);
}
#[test]
fn defence_cat_reduction_skips_plain_asset_conceal_and_paid_reveal_then_applies_once() {
    let mut g = game();
    fund(&mut g, 0, "JC104", 8);
    fund(&mut g, 0, "JC075", 1);
    let target = field(&mut g, "JC003", 0);
    let source = cat(&mut g);
    reduce(&mut g, &source);
    let plain = held(&mut g, "JC125", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(plain),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    top(&mut g);
    assert_eq!(g.modifiers[0].uses, 1);
    let asset = held(&mut g, "JC104", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(asset),
            ..Action::new("asset")
        },
    );
    assert_eq!(g.modifiers[0].uses, 1);
    let hidden = held(&mut g, "JC104", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            region: Some(0),
            ..Action::new("conceal")
        },
    );
    let hidden = g.regions[0]
        .cards
        .iter()
        .find(|c| c.face_down)
        .unwrap()
        .id
        .clone();
    act(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    top(&mut g);
    assert_eq!(g.modifiers[0].uses, 1);
    let before = g.resources(0);
    cast(&mut g, "JC102", 0, &target);
    assert_eq!(g.resources(0), before - 1);
    assert_eq!(g.modifiers[0].uses, 0);
}
#[test]
fn defence_cat_reduction_does_not_supply_loyalty_and_failed_payment_is_atomic() {
    let mut g = game();
    fund(&mut g, 0, "JC075", 2);
    let target = field(&mut g, "JC003", 0);
    let purple = held(&mut g, "JC104", 0);
    let fire = held(&mut g, "JC102", 0);
    let source = cat(&mut g);
    reduce(&mut g, &source);
    let a = Action {
        card_id: Some(fire),
        target_id: Some(target),
        ..Action::new("play")
    };
    reject(&mut g, 0, a.clone());
    assert_eq!(g.modifiers[0].uses, 1);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(purple),
            ..Action::new("asset")
        },
    );
    let before = g.resources(0);
    act(&mut g, 0, a);
    top(&mut g);
    assert_eq!(g.resources(0), before - 1);
    assert_eq!(g.modifiers[0].uses, 0);
}
#[test]
fn defence_cat_reduction_is_actor_scoped_and_expires_at_turn_end() {
    let mut g = game();
    fund(&mut g, 0, "JC075", 1);
    fund(&mut g, 1, "JC075", 1);
    let target = field(&mut g, "JC003", 0);
    let source = cat(&mut g);
    reduce(&mut g, &source);
    cast(&mut g, "JC078", 1, &target);
    assert_eq!(g.resources(1), 0);
    assert_eq!(g.modifiers[0].uses, 1);
    let turn = g.turn;
    advance(&mut g, |g| g.turn > turn);
    assert!(g.modifiers.is_empty());
}
#[test]
fn defence_vest_play_and_activation_roles_cannot_be_interchanged() {
    let mut g = game();
    let host = field(&mut g, "JC125", 0);
    let next = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "JC125", 4);
    let source = held(&mut g, "XQ47", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(source.clone()),
            target_id: Some(host.clone()),
            ability_id: Some("reattach".into()),
            ..Action::new("play")
        },
    );
    assert!(g
        .legal_actions(0)
        .iter()
        .filter(|a| a.action.card_id.as_ref() == Some(&source))
        .all(|a| a.action.ability_id.as_deref() != Some("reattach")));
    play(&mut g, &source, 0, &host);
    let vest = g.attachments[0].card.id.clone();
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(vest.clone()),
            target_id: Some(next.clone()),
            ability_id: Some("attach".into()),
            ..Action::new("activate")
        },
    );
    let actions = g
        .legal_actions(0)
        .into_iter()
        .filter(|a| a.action.card_id.as_ref() == Some(&vest) && a.action.kind == "activate")
        .collect::<Vec<_>>();
    assert!(!actions.is_empty());
    assert!(actions
        .iter()
        .all(|a| a.action.ability_id.as_deref() == Some("reattach")));
    act(&mut g, 0, transfer(&vest, &next));
    top(&mut g);
    assert_eq!(g.attachments[0].card.id, vest);
}
#[test]
fn defence_new_programs_reject_unbound_hidden_and_nested_targets() {
    for id in ["JC078", "XQ47"] {
        let original = definition(id).abilities.last().unwrap().clone();
        let mut a = original.clone();
        a.targets.clear();
        assert!(validate_ability(id, &a).is_err());
        let mut a = original;
        a.targets[0].kind = EntityKind::Hidden;
        assert!(validate_ability(id, &a).is_err());
    }
    let mut a = definition("JC078").abilities[0].clone();
    a.ops = vec![Op::ForEachLivingPlayer(a.ops.clone())];
    assert!(validate_ability("JC078", &a).is_err());
}
