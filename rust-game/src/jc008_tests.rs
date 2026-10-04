//! Initial layouts, controller/movement checkpoints and wounds are explicit
//! fixtures. Plays, responses, passes, choices and cleanup use existing rules.
use crate::{catalog, model::*, rules::*};

fn game() -> Game {
    let mut g = Game::new(
        "jc008-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        4,
    )
    .unwrap();
    for s in 1..4 {
        g.join(format!("P{s}"), "watchers".into()).unwrap();
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
fn field(g: &mut Game, def: &str, owner: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn hand(g: &mut Game, def: &str, owner: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn fund(g: &mut Game, actor: usize, def: &str, count: usize) {
    for _ in 0..count {
        let c = g.make_card(def, actor);
        g.players[actor].assets.push(c);
    }
}
fn action(source: &str, target: &str) -> Action {
    Action {
        card_id: Some(source.into()),
        target_id: Some(target.into()),
        ..Action::new("play")
    }
}
fn pass_top(g: &mut Game) {
    let n = g.stack.len();
    assert!(n > 0);
    for _ in 0..32 {
        if g.stack.len() < n {
            return;
        }
        let s = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(s, Action::new("pass")).unwrap();
    }
    panic!("bounded stack did not settle");
}
fn grant(g: &mut Game, target: &str) {
    fund(g, 0, "JC002", 2);
    let source = hand(g, "JC008", 0);
    g.apply(0, action(&source, target)).unwrap();
    pass_top(g);
}
fn roundtrip(g: &mut Game) {
    let state = serde_json::to_string(g).unwrap();
    *g = Game::from_persisted(&state).unwrap();
    assert_eq!(serde_json::to_string(g).unwrap(), state);
}
fn pass_until(g: &mut Game, test: impl Fn(&Game) -> bool) {
    for _ in 0..40 {
        if test(g) {
            return;
        }
        let s = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(s, Action::new("pass")).unwrap();
    }
    panic!("bounded window did not progress");
}
fn finish_turn(g: &mut Game) {
    let turn = g.turn;
    g.begin_window(Window::End);
    pass_until(g, |g| g.turn > turn || g.pending.is_some());
}

#[test]
fn jc008_printed_admission_and_finite_modifier_program() {
    let c = catalog::card("JC008");
    assert_eq!(
        (c.name.as_str(), c.kind.as_str(), c.cost, c.defense),
        ("灵能激发", "spell", 2, None)
    );
    assert_eq!(c.loyalty, ["黄色"]);
    assert_eq!(c.magic_icon, MagicIcon::Mind);
    assert_eq!(c.subtypes, ["法术", "心灵"]);
    assert_eq!(
        (c.permanent_icons, c.temporary_icons),
        (Icons::default(), Icons::default())
    );
    let a = crate::rules::definition("JC008").abilities[0].clone();
    assert!(matches!(a.timing, Timing::Fast));
    assert!(a.costs.is_empty() && a.modes.is_empty());
    assert_eq!(a.targets[0].kind, EntityKind::Character);
    assert_eq!(a.targets[0].relation, Relation::Any);
    assert!(matches!(
        a.ops[0],
        Op::ModifyTargetUntilTurnEnd {
            slot: 0,
            defense_bonus: 1,
            ordinary_icons: Icons {
                combat: 1,
                investigation: 0,
                influence: 0
            }
        }
    ));
    assert!(crate::rules::validate_ability("fixture", &a).is_ok());
    for kind in [
        EntityKind::Any,
        EntityKind::Attachment,
        EntityKind::CharacterOrHidden,
    ] {
        let mut wrong = a.clone();
        wrong.targets[0].kind = kind;
        assert!(crate::rules::validate_ability("fixture", &wrong).is_err());
    }
    assert_eq!(
        catalog::catalog()
            .societies
            .iter()
            .map(|s| s.card.id.as_str())
            .collect::<Vec<_>>(),
        ["MSJC09", "MSJC01"]
    );
    assert!(catalog::catalog().cards.iter().all(|c| c.id != "MSJC01"));
}

#[test]
fn jc008_rejects_hidden_noncharacter_barrier_and_unpaid_declarations_without_changes() {
    for bad in ["hidden", "attachment", "barrier", "unpaid", "loyalty"] {
        let mut g = game();
        let source = hand(&mut g, "JC008", 0);
        fund(
            &mut g,
            0,
            if bad == "loyalty" { "JC063" } else { "JC002" },
            if bad == "unpaid" { 1 } else { 2 },
        );
        let target = field(
            &mut g,
            if bad == "barrier" {
                "JZ08"
            } else if bad == "attachment" {
                "XQ03"
            } else {
                "JC125"
            },
            2,
        );
        if bad == "hidden" {
            g.board_mut(&target).unwrap().face_down = true;
        }
        let before = serde_json::to_string(&g).unwrap();
        assert!(g.apply(0, action(&source, &target)).is_err(), "{bad}");
        assert_eq!(serde_json::to_string(&g).unwrap(), before, "{bad}");
    }
}

#[test]
fn jc008_ordinary_icons_apply_to_both_teams_and_remain_current_when_exhausted() {
    let mut g = game();
    let first = field(&mut g, "JC004", 0);
    let rear = field(&mut g, "JC004", 2);
    grant(&mut g, &first);
    grant(&mut g, &rear);
    assert_eq!(g.icons(g.board(&first).unwrap().1, 0).combat, 2); // printed first-only one + ordinary one
    assert_eq!(g.icons(g.board(&rear).unwrap().1, 0).combat, 1); // rear still has granted ordinary one
    assert_eq!(catalog::card("JC004").permanent_icons.combat, 0);
    assert_eq!(catalog::card("JC004").temporary_icons.combat, 1);
    g.board_mut(&rear).unwrap().exhausted = true;
    assert_eq!(g.current_icons(g.board(&rear).unwrap().1, 0).combat, 1);
    assert_eq!(g.icons(g.board(&rear).unwrap().1, 0), Icons::default());
    assert_eq!(g.defense(g.board(&rear).unwrap().1, 0), 3);
    assert_eq!(
        g.view(2).regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == rear)
            .unwrap()
            .icons,
        Some(Icons::default())
    );
}

#[test]
fn jc008_stacks_each_paid_resolution_once_and_is_not_tethered_to_spell_source() {
    let mut g = game();
    let target = field(&mut g, "JC125", 2);
    fund(&mut g, 0, "JC002", 4);
    let one = hand(&mut g, "JC008", 0);
    let two = hand(&mut g, "JC008", 0);
    g.apply(0, action(&one, &target)).unwrap();
    g.apply(0, action(&two, &target)).unwrap();
    roundtrip(&mut g);
    assert!(g.turn_attribute_modifiers.is_empty());
    pass_top(&mut g);
    assert_eq!(g.turn_attribute_modifiers.len(), 1);
    roundtrip(&mut g);
    pass_top(&mut g);
    roundtrip(&mut g);
    assert_eq!(g.turn_attribute_modifiers.len(), 2);
    assert_eq!(g.defense(g.board(&target).unwrap().1, 0), 3);
    assert_eq!(g.icons(g.board(&target).unwrap().1, 0).combat, 2);
    assert!(g
        .turn_attribute_modifiers
        .iter()
        .all(|m| m.target_instance == target && m.expires_turn == g.turn));
    assert_eq!(
        g.players[0].assets.iter().filter(|c| c.exhausted).count(),
        4
    );
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JC008")
            .count(),
        2
    );
    assert_eq!(catalog::card("JC125").defense, Some(1));
}

#[test]
fn jc008_shared_shield_guard_cancels_enemy_bonus_and_preserves_friendly_shield() {
    for owner in [0, 2] {
        let mut g = game();
        let target = field(&mut g, "JC125", owner);
        g.board_mut(&target).unwrap().shield = 1;
        grant(&mut g, &target);
        assert_eq!(g.turn_attribute_modifiers.len(), usize::from(owner == 0));
        assert_eq!(g.board(&target).unwrap().1.shield, u32::from(owner == 0));
        assert_eq!(
            g.players[0].assets.iter().filter(|c| c.exhausted).count(),
            2
        );
    }
}

#[test]
fn jc008_normal_return_and_hide_responses_cancel_original_target_after_payment() {
    for hide in [false, true] {
        let mut g = game();
        let target = field(&mut g, "JC003", 2);
        fund(&mut g, 0, "JC002", 2);
        fund(&mut g, 2, "JC003", 2);
        fund(&mut g, 2, "JC063", 2);
        let source = hand(&mut g, "JC008", 0);
        let reply = hand(&mut g, if hide { "JC063" } else { "JC006" }, 2);
        g.apply(0, action(&source, &target)).unwrap();
        roundtrip(&mut g);
        g.apply(0, Action::new("pass")).unwrap();
        g.apply(1, Action::new("pass")).unwrap();
        let mut response = action(&reply, &target);
        if hide {
            response.option = Some("hide".into());
        }
        g.apply(2, response).unwrap();
        pass_top(&mut g);
        roundtrip(&mut g);
        pass_top(&mut g);
        assert!(g.turn_attribute_modifiers.is_empty());
        assert!(g.board(&target).is_none());
        assert_eq!(
            g.players[0].assets.iter().filter(|c| c.exhausted).count(),
            2
        );
        assert!(g
            .log
            .iter()
            .any(|l| l.text.contains("原目标") && l.text.contains("取消")));
    }
}

#[test]
fn jc008_granted_instance_bonus_does_not_survive_departure_or_face_changes() {
    for hide in [false, true] {
        let mut g = game();
        let old = field(&mut g, "JC003", 2);
        grant(&mut g, &old);
        if hide {
            let (_, c) = g.leave_board(&old).unwrap();
            let mut c = g.fresh(c);
            c.face_down = true;
            g.regions[0].cards.push(c);
            g.settle_deaths();
            let hidden = g.regions[0].cards[0].id.clone();
            assert_ne!(hidden, old);
            assert!(g.turn_attribute_modifiers.is_empty());
            let (_, c) = g.leave_board(&hidden).unwrap();
            let mut c = g.fresh(c);
            c.face_down = false;
            g.regions[0].cards.push(c);
        } else {
            g.return_hand(&old);
            g.settle_deaths();
            assert!(g.turn_attribute_modifiers.is_empty());
            let c = g.players[2].hand.pop().unwrap();
            let c = g.fresh(c);
            g.regions[0].cards.push(c);
        }
        let c = &g.regions[0].cards[0];
        assert_ne!(c.id, old);
        assert_eq!(g.defense(c, 0), 1);
        assert_eq!(g.current_icons(c, 0).combat, 0);
    }
}

#[test]
fn jc008_instance_bonus_follows_movement_and_controller_with_current_team_counts() {
    let mut g = game();
    let id = field(&mut g, "JC004", 2);
    grant(&mut g, &id);
    let c = g.regions[0].cards.remove(0);
    g.regions[1].cards.push(c);
    g.settle_deaths();
    assert_eq!(g.defense(g.board(&id).unwrap().1, 1), 3);
    g.board_mut(&id).unwrap().controller = 1;
    g.settle_deaths();
    roundtrip(&mut g);
    assert_eq!(g.turn_attribute_modifiers.len(), 1);
    assert_eq!(g.board(&id).unwrap().1.owner, 2);
    assert_eq!(g.current_icons(g.board(&id).unwrap().1, 1).combat, 2);
    assert_eq!(g.contest_counts(1, 1), [2, 0]);
    let v = g.view(3).regions[1]
        .characters
        .iter()
        .find(|c| c.instance_id == id)
        .unwrap()
        .clone();
    assert_eq!(
        (v.owner.as_str(), v.controller.as_str(), v.defense),
        ("p2", "p1", Some(3))
    );
}

#[test]
fn jc008_cleanup_simultaneously_clears_damage_and_expires_bonus_without_transient_death() {
    let mut g = game();
    let id = field(&mut g, "JC125", 2);
    grant(&mut g, &id);
    g.board_mut(&id).unwrap().damage = 1;
    g.settle_deaths();
    roundtrip(&mut g);
    assert_eq!(g.defense(g.board(&id).unwrap().1, 0), 2);
    finish_turn(&mut g);
    let c = g
        .board(&id)
        .expect("base1 + bonus1 + damage1 must survive atomic cleanup")
        .1;
    assert_eq!((c.damage, g.defense(c, 0)), (0, 1));
    assert!(g.turn_attribute_modifiers.is_empty());
    assert!(g.players[2]
        .graveyard
        .iter()
        .all(|c| c.definition != "JC125"));
}

#[test]
fn jc008_cleanup_preserves_wounds_shields_and_settles_true_zero_defense() {
    let mut g = game();
    let survivor = field(&mut g, "JC004", 2);
    let dead = field(&mut g, "JC125", 2);
    grant(&mut g, &survivor);
    grant(&mut g, &dead);
    let c = g.board_mut(&survivor).unwrap();
    c.wounds = 1;
    c.damage = 1;
    c.shield = 2;
    g.board_mut(&dead).unwrap().wounds = 1;
    g.settle_deaths();
    finish_turn(&mut g);
    let c = g.board(&survivor).unwrap().1;
    assert_eq!(
        (c.wounds, c.shield, c.damage, g.defense(c, 0)),
        (1, 2, 0, 1)
    );
    assert!(g.board(&dead).is_none());
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125"));
}

#[test]
fn jc008_hand_limit_pause_preserves_damage_and_expiry_until_final_atomic_cleanup() {
    let mut g = game();
    let id = field(&mut g, "JC125", 2);
    grant(&mut g, &id);
    g.board_mut(&id).unwrap().damage = 1;
    for s in [0, 2] {
        for _ in 0..8 {
            hand(&mut g, "JC125", s);
        }
    }
    finish_turn(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    for expected in [0, 2] {
        let p = g.pending.clone().unwrap();
        assert_eq!(p.seat, expected);
        assert_eq!(g.board(&id).unwrap().1.damage, 1);
        assert_eq!(g.turn_attribute_modifiers.len(), 1);
        assert!(g.effects.iter().any(|e| matches!(e, Effect::FinishCleanup)));
        roundtrip(&mut g);
        g.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![p.choice.options[0].id.clone()]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    roundtrip(&mut g);
    assert_eq!(g.board(&id).unwrap().1.damage, 0);
    assert!(g.turn_attribute_modifiers.is_empty());
    assert_eq!((g.players[0].hand.len(), g.players[2].hand.len()), (7, 7));
}

#[test]
fn jc008_cleanup_death_trigger_resolves_before_initiative_changes_and_repeats_cleanup() {
    let mut g = game();
    let id = field(&mut g, "XQ12", 2);
    grant(&mut g, &id);
    g.board_mut(&id).unwrap().wounds = catalog::card("XQ12").defense.unwrap();
    let old_turn = g.turn;
    finish_turn(&mut g);
    let p = g
        .pending
        .clone()
        .expect("expiry death triggers existing XQ12 ability");
    assert_eq!(p.choice.kind, "trigger");
    assert_eq!(g.turn, old_turn);
    assert_eq!(g.first_team, 0);
    roundtrip(&mut g);
    g.apply(
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![]),
            ..Action::new("choose")
        },
    )
    .unwrap();
    assert_eq!(g.turn, old_turn);
    assert!(g.effects.iter().all(|e| !matches!(e, Effect::NextTurn)));
    pass_until(&mut g, |g| g.turn > old_turn);
    assert_eq!(g.first_team, 1);
}

#[test]
fn jc008_rear_actor_bonus_and_reverse_initiative_hand_limit_order() {
    let mut g = game();
    g.first_team = 1;
    g.begin_window(Window::Action(0));
    let id = field(&mut g, "JC125", 2);
    grant(&mut g, &id);
    assert_eq!(g.icons(g.board(&id).unwrap().1, 0).combat, 1);
    g.board_mut(&id).unwrap().damage = 1;
    for s in [0, 2] {
        for _ in 0..8 {
            hand(&mut g, "JC125", s);
        }
    }
    finish_turn(&mut g);
    for expected in [2, 0] {
        let p = g.pending.clone().unwrap();
        assert_eq!(p.seat, expected);
        assert_eq!(g.turn_attribute_modifiers.len(), 1);
        assert_eq!(g.board(&id).unwrap().1.damage, 1);
        roundtrip(&mut g);
        g.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![p.choice.options[0].id.clone()]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    assert_eq!(g.first_team, 0);
    assert_eq!(g.board(&id).unwrap().1.damage, 0);
    assert!(g.turn_attribute_modifiers.is_empty());
}
