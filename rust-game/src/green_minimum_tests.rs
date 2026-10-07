//! Approved finite green minimum. Explicit native layouts; no public UI claim.
use crate::jc029_tests::{apply, board, checkpoint, choose, envelope, fund, pass_top, reject};
use crate::{catalog, deck, model::*, rules::*};
const GREENS: [&str; 9] = [
    "JC014", "JC016", "JZ08", "BQ022", "JC020", "XQ07", "LC30", "JC018", "JC015",
];
fn draft(n: usize) -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    d.society_id = Some("MSJC02".into());
    d.cards.clear();
    let mut left = n;
    for id in GREENS {
        let count = left.min(3);
        if count > 0 {
            d.cards.push(catalog::DeckEntry {
                card_id: id.into(),
                count,
            });
        }
        left -= count;
    }
    assert_eq!(left, 0);
    d.cards.push(catalog::DeckEntry {
        card_id: "JC125".into(),
        count: 50 - n,
    });
    d
}
fn initial(actor: usize) -> Game {
    let mut g = Game::new_with_deck(
        "green-native".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        draft(27),
        19,
    )
    .unwrap();
    for s in 1..4 {
        g.join_with_deck(format!("P{s}"), draft(27)).unwrap();
    }
    for s in 0..4 {
        g.apply(s, Action::new("ready")).unwrap();
    }
    g.apply(0, Action::new("start")).unwrap();
    assert!(g.players.iter().all(|p| p.hand.len() == 6));
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
        p.deck.clear();
    }
    for r in &mut g.regions {
        r.cards.clear();
    }
    for s in 0..4 {
        for _ in 0..8 {
            let c = g.make_card("JC125", s);
            g.players[s].deck.push(c);
        }
    }
    g.first_team = g.team(actor);
    g.begin_window(Window::Action(g.team(actor)));
    g
}
fn attach(g: &mut Game, def: &str, owner: usize, host: &str) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.attachments.push(Attachment {
        card: c,
        host_id: host.into(),
    });
    id
}
fn hand(g: &mut Game, def: &str, owner: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn activate(id: &str, key: &str, target: Option<String>) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(key.into()),
        target_id: target,
        ..Action::new("activate")
    }
}
fn turn_control(g: &mut Game, id: &str, seat: usize, subtype: SubtypeChange) {
    g.add_control(id, seat, ControlLifetime::TurnEnd { turn: g.turn }, subtype);
}
fn stats(g: &Game, id: &str) -> (u32, u32) {
    let (r, c) = g.board(id).unwrap();
    (g.current_permanent_combat(c, r), g.defense(c, r))
}
fn play_action(g: &Game, actor: usize, id: &str, target: &str) -> Action {
    g.legal_actions(actor)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(id)
                && a.action.target_id.as_deref() == Some(target)
        })
        .expect("actual legal spell")
        .action
}
fn source(g: &Game, s: usize) -> String {
    g.players[s].society_zone.card.as_ref().unwrap().id.clone()
}

#[test]
fn green_whole_printed_fields_and_closed_three_shapes() {
    let d = catalog::card("LC30");
    assert_eq!((&*d.name, &*d.color, d.cost), ("J·罗伯茨，“海雕”", "绿", 4));
    assert_eq!(d.subtitle.as_deref(), Some("“猛禽”战术小组"));
    assert_eq!(d.loyalty, ["绿色"]);
    assert_eq!(d.subtypes, ["人类", "猎手"]);
    assert!(d.unique && d.magic.is_empty());
    assert_eq!(d.defense, Some(2));
    assert_eq!(d.keywords, ["护卫2"]);
    assert_eq!(
        d.permanent_icons,
        Icons {
            combat: 1,
            influence: 1,
            ..Default::default()
        }
    );
    assert_eq!(d.temporary_icons, Icons::default());
    assert_eq!(d.rule_traits.guard, 2);
    let d = catalog::card("JC018");
    assert_eq!(d.cost, 4);
    assert_eq!(d.loyalty, ["绿色", "绿色"]);
    assert_eq!(d.magic_icon, MagicIcon::Mind);
    assert_eq!(d.subtypes, ["人类", "猎手", "超能力者"]);
    assert_eq!(d.defense, Some(2));
    assert!(!d.rule_traits.renown && !d.unique); // Printed 威名 is combat-timed, not 声望.
    assert_eq!(
        d.permanent_icons,
        Icons {
            combat: 1,
            ..Default::default()
        }
    );
    assert_eq!(
        d.temporary_icons,
        Icons {
            combat: 1,
            influence: 1,
            ..Default::default()
        }
    );
    let d = catalog::card("JC015");
    assert_eq!(d.cost, 3);
    assert_eq!(d.loyalty, ["绿色"]);
    assert_eq!(d.magic, "神圣");
    assert_eq!(d.subtypes, ["人类", "僧侣"]);
    assert_eq!(d.defense, Some(1));
    assert_eq!(
        d.permanent_icons,
        Icons {
            influence: 1,
            ..Default::default()
        }
    );
    assert_eq!(
        d.temporary_icons,
        Icons {
            investigation: 1,
            combat: 1,
            ..Default::default()
        }
    );
    assert!(!d.unique && d.keywords.is_empty());
    let a = &definition("JC015").abilities[0];
    assert_eq!(a.timing, Timing::Standard);
    assert!(matches!(
        a.costs.as_slice(),
        [Cost::ExhaustSource, Cost::SacrificeSource]
    ));
    assert!(matches!(
        a.ops.as_slice(),
        [Op::Destroy(EntityRef::Target(0))]
    ));
    assert_eq!(
        a.targets[0].predicate,
        Some(TargetPredicate::JC015NonHumanPrintedCostAtLeastThree)
    );
    assert_eq!(catalog::catalog().cards.len(), 103);
    assert_eq!(catalog::catalog().societies.len(), 8);
    assert!(crate::society::definition("MSJC03").is_ok());
    assert!(!catalog::catalog().cards.iter().any(|c| c.id == "XQ11"));
}
#[test]
fn green_real_twentyfive_deck_constraint_and_six_cards_for_four_seats() {
    for n in [24, 25, 27] {
        assert_eq!(deck::validate(draft(n)).is_ok(), n >= 25);
    }
    let mut d = draft(27);
    d.cards[0].count = 4;
    assert!(deck::validate(d).is_err());
    let mut d = draft(25);
    d.cards.last_mut().unwrap().count -= 1;
    assert!(deck::validate(d).is_err());
    let g = initial(0);
    let s = crate::society::definition("MSJC02").unwrap();
    assert_eq!((&*s.card.name, &*s.subtitle), ("猎魔人", "猎杀异种战团"));
    assert_eq!(s.card.subtypes, ["群体"]);
    assert_eq!(s.starting_hand, 6);
    assert!(
        s.card.unique
            && s.printed_cost.is_none()
            && s.card.defense.is_none()
            && s.card.loyalty.is_empty()
    );
    assert_eq!(
        serde_json::to_value(&s.deck_constraints).unwrap(),
        serde_json::json!([{"kind":"minimumColor","color":"绿","count":25}])
    );
    checkpoint(&g);
}
#[test]
fn green_lc30_only_live_self_weapons_and_exhaustion_not_asset_or_other_host() {
    let mut g = initial(0);
    let id = board(&mut g, "LC30", 0, 0);
    let other = board(&mut g, "JC059", 1, 1);
    fund(&mut g, 0, "JC116", 1);
    attach(&mut g, "JC116", 1, &other);
    attach(&mut g, "XQ47", 3, &id);
    assert_eq!(
        g.green_source_attribute_bonus(g.board(&id).unwrap().1, 0),
        (0, 0)
    );
    assert_eq!(stats(&g, &id), (1, 3));
    let first = attach(&mut g, "BQ022", 2, &id);
    assert_eq!(stats(&g, &id), (3, 4));
    let second = attach(&mut g, "JC116", 1, &id);
    assert_eq!(stats(&g, &id), (5, 5));
    for a in &mut g.attachments {
        a.card.exhausted = true;
    }
    assert_eq!(stats(&g, &id), (5, 5));
    g.attachments
        .iter_mut()
        .find(|a| a.card.id == second)
        .unwrap()
        .card
        .face_down = true;
    // Existing host_icons path still contributes JC116's printed grant;
    // only LC30's new source-bound weapon count excludes this synthetic back.
    assert_eq!(stats(&g, &id), (4, 4));
    g.attachments
        .iter_mut()
        .find(|a| a.card.id == second)
        .unwrap()
        .card
        .face_down = false;
    let c = g.remove_board(&id).unwrap().1;
    g.regions[1].cards.push(c);
    // JC059 is now a same-region teammate and its old defense aura adds one.
    assert_eq!(stats(&g, &id), (5, 6));
    assert_eq!(
        g.green_source_attribute_bonus(g.board(&id).unwrap().1, 1),
        (2, 2)
    );
    checkpoint(&g);
    g.board_mut(&id).unwrap().face_down = true;
    assert_eq!(
        g.green_source_attribute_bonus(g.board(&id).unwrap().1, 1),
        (0, 0)
    );
    g.board_mut(&id).unwrap().face_down = false;
    g.remove_board(&id);
    let new = board(&mut g, "LC30", 0, 1);
    assert_ne!(new, id);
    assert_eq!(stats(&g, &new), (1, 3));
    g.settle_attachments();
    // This deliberately bypassed leave_board to retain a stale host identity.
    // Invalid-attachment cleanup sends it to grave; real host departure and
    // owner-hand recycling are exercised by the paid unique-sacrifice test.
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "BQ022" && c.id != first));
    checkpoint(&g);
    assert_eq!(catalog::card("LC30").permanent_icons.combat, 1);
    assert_eq!(catalog::card("LC30").defense, Some(2));
}
#[test]
fn green_lc30_actual_combat_window_emits_guard_two_allocation() {
    let mut g = initial(0);
    board(&mut g, "JC016", 0, 0);
    board(&mut g, "JC016", 0, 0);
    let guard = board(&mut g, "LC30", 1, 0);
    turn_control(&mut g, &guard, 2, SubtypeChange::None);
    attach(&mut g, "BQ022", 3, &guard);
    let other = board(&mut g, "JC059", 2, 0);
    g.begin_window(Window::Before(0, 1));
    g.close_window().unwrap();
    g.drive().unwrap();
    let p = g.pending.clone().expect("actual combat damage choice");
    assert_eq!(p.choice.kind, "damage");
    // Combat 4 - 3, plus two existing JC016 kill values, gives three.
    assert_eq!(p.choice.amount, Some(3));
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id.clone()),
            allocations: Some(std::collections::BTreeMap::from([(other.clone(), 3)])),
            ..Action::new("choose")
        },
    );
    apply(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id),
            allocations: Some(std::collections::BTreeMap::from([
                (guard.clone(), 2),
                (other.clone(), 1),
            ])),
            ..Action::new("choose")
        },
    );
    assert_eq!(g.board(&guard).unwrap().1.damage, 2);
    assert_eq!(g.board(&other).unwrap().1.damage, 1);
    checkpoint(&g);
}
#[test]
fn green_lc30_guard_two_requires_two_before_other_enemy_even_when_exhausted_and_borrowed() {
    for amount in [2, 3] {
        let mut g = initial(0);
        let guard = board(&mut g, "LC30", 0, 0);
        turn_control(&mut g, &guard, 2, SubtypeChange::None);
        attach(&mut g, "BQ022", 2, &guard);
        g.board_mut(&guard).unwrap().exhausted = true;
        let other = board(&mut g, "JC059", 2, 0);
        g.effect(Effect::Damage {
            seat: 0,
            region: 0,
            amount,
        })
        .unwrap();
        let p = g.pending.as_ref().unwrap().clone();
        reject(
            &mut g,
            0,
            Action {
                choice_id: Some(p.choice.id.clone()),
                allocations: Some(std::collections::BTreeMap::from([
                    (guard.clone(), 1),
                    (other.clone(), amount - 1),
                ])),
                ..Action::new("choose")
            },
        );
        let mut allocations = std::collections::BTreeMap::from([(guard.clone(), 2)]);
        if amount > 2 {
            allocations.insert(other.clone(), amount - 2);
        }
        apply(
            &mut g,
            0,
            Action {
                choice_id: Some(p.choice.id),
                allocations: Some(allocations),
                ..Action::new("choose")
            },
        );
        assert_eq!(g.board(&guard).unwrap().1.damage, 2);
        assert_eq!(g.board(&other).unwrap().1.damage, amount - 2);
        checkpoint(&g);
    }
}
#[test]
fn green_lc30_lost_weapon_uses_existing_lethal_stable_point_and_vest_counterexample() {
    for vest in [false, true] {
        let mut g = initial(0);
        let id = board(&mut g, "LC30", 0, 0);
        let weapon = attach(&mut g, "JC116", 2, &id);
        if vest {
            attach(&mut g, "XQ47", 3, &id);
        }
        g.board_mut(&id).unwrap().damage = 2;
        assert!(stats(&g, &id).1 > 2);
        fund(&mut g, 0, "JC003", 2);
        let spell = hand(&mut g, "JC005", 0);
        let a = play_action(&g, 0, &spell, &weapon);
        apply(&mut g, 0, a);
        pass_top(&mut g);
        assert_eq!(g.board(&id).is_some(), vest);
        assert!(g.players[2]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC116" && c.id != weapon));
        if vest {
            assert_eq!(stats(&g, &id).1, 3);
        } else {
            assert!(g.players[0]
                .graveyard
                .iter()
                .any(|c| c.definition == "LC30" && c.id != id));
        }
        checkpoint(&g);
    }
}
#[test]
fn green_lc30_duplicate_sacrifice_and_borrowed_return_keep_original_owners() {
    let mut g = initial(0);
    let borrowed = board(&mut g, "LC30", 1, 0);
    turn_control(&mut g, &borrowed, 0, SubtypeChange::None);
    let knife = attach(&mut g, "BQ022", 2, &borrowed);
    fund(&mut g, 0, "JC014", 4);
    let original = hand(&mut g, "LC30", 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(original),
            region: Some(1),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    assert!(matches!(
        g.pending.as_ref().unwrap().resolution,
        ChoiceResolution::Unique
    ));
    choose(&mut g, vec![borrowed.clone()]);
    assert!(g.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "LC30" && c.id != borrowed && c.controller == 1));
    assert!(g.players[0].graveyard.is_empty());
    assert!(g.players[2]
        .hand
        .iter()
        .any(|c| c.definition == "BQ022" && c.id != knife));
    checkpoint(&g);
    let mut g = initial(0);
    let borrowed = board(&mut g, "LC30", 1, 0);
    turn_control(&mut g, &borrowed, 0, SubtypeChange::None);
    g.return_hand(&borrowed);
    // Published return_hand uses fresh(), not reset_zone_card(): owner zone
    // and identity reset, while the departing controller is retained. The
    // finite green admission must not silently change this old path.
    assert!(g.players[1]
        .hand
        .iter()
        .any(|c| c.definition == "LC30" && c.id != borrowed && c.controller == 0));
    assert!(g.players[0].hand.is_empty());
    checkpoint(&g);
}
#[test]
fn green_jc018_counts_controller_mind_assets_including_exhausted_and_not_teammate() {
    let mut g = initial(0);
    let id = board(&mut g, "JC018", 0, 0);
    fund(&mut g, 2, "JC003", 2);
    fund(&mut g, 1, "JC003", 3);
    hand(&mut g, "JC004", 0);
    board(&mut g, "JC004", 0, 1);
    assert_eq!(stats(&g, &id).0, 1);
    fund(&mut g, 0, "JC003", 1);
    assert_eq!(stats(&g, &id).0, 2);
    fund(&mut g, 0, "JC004", 1);
    assert_eq!(stats(&g, &id).0, 3);
    for c in &mut g.players[0].assets {
        c.exhausted = true;
    }
    assert_eq!(stats(&g, &id).0, 3);
    g.first_team = 1;
    assert_eq!(stats(&g, &id).0, 3);
    turn_control(&mut g, &id, 1, SubtypeChange::None);
    assert_eq!(stats(&g, &id).0, 4);
    assert_eq!(catalog::card("JC018").permanent_icons.combat, 1);
    let c = g.remove_board(&id).unwrap().1;
    g.regions[1].cards.push(c);
    assert_eq!(stats(&g, &id).0, 4);
    checkpoint(&g);
    g.board_mut(&id).unwrap().face_down = true;
    assert_eq!(g.current_permanent_combat(g.board(&id).unwrap().1, 1), 0);
    checkpoint(&g);
}
#[test]
fn green_jc015_only_current_nonhuman_printed_cost_three_enemy_faceup_any_region() {
    for (def, owner, hidden, converted, expected) in [
        ("JC096", 2, false, false, true),
        ("JC029", 2, false, false, false),
        ("JC059", 2, false, false, false),
        ("JC059", 2, false, true, true),
        ("JC096", 1, false, false, false),
        ("JC096", 2, true, false, false),
    ] {
        let mut g = initial(0);
        let src = board(&mut g, "JC015", 0, 0);
        let target = board(&mut g, def, owner, 1);
        g.board_mut(&target).unwrap().face_down = hidden;
        if converted {
            turn_control(&mut g, &target, owner, SubtypeChange::HumanToVampire);
        }
        let a = activate(&src, "exorcise", Some(target.clone()));
        let before = serde_json::to_string(&g).unwrap();
        if expected {
            apply(&mut g, 0, a);
            assert!(g.board(&src).is_none());
            assert_eq!(
                g.stack
                    .last()
                    .unwrap()
                    .frame
                    .as_ref()
                    .unwrap()
                    .already_paid
                    .len(),
                2
            );
            pass_top(&mut g);
            assert!(g.board(&target).is_none());
        } else {
            reject(&mut g, 0, a);
            assert_eq!(serde_json::to_string(&g).unwrap(), before);
        }
        checkpoint(&g);
    }
}
#[test]
fn green_jc015_four_actor_borrowed_source_sacrifice_recovery_and_independent_frame() {
    for actor in 0..4 {
        let mut g = initial(actor);
        let owner = (actor + 1) % 4;
        let src = board(&mut g, "JC015", owner, 0);
        turn_control(&mut g, &src, actor, SubtypeChange::None);
        // Adjacent seats share a team; +2 is always the opposing team.
        let target = board(&mut g, "JC096", (actor + 2) % 4, 1);
        let attachment_owner = (actor + 2) % 4;
        let knife = attach(&mut g, "BQ022", attachment_owner, &src);
        apply(
            &mut g,
            actor,
            activate(&src, "exorcise", Some(target.clone())),
        );
        assert!(g.players[owner]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC015" && c.id != src && c.controller == owner));
        assert!(g.players[attachment_owner]
            .hand
            .iter()
            .any(|c| c.definition == "BQ022" && c.id != knife));
        let mut restored = envelope(&g).game;
        pass_top(&mut g);
        pass_top(&mut restored);
        assert_eq!(
            serde_json::to_value(&g).unwrap(),
            serde_json::to_value(&restored).unwrap()
        );
        assert!(g.board(&target).is_none());
        checkpoint(&g);
    }
}
#[test]
fn green_jc015_target_hides_replaced_or_becomes_friendly_after_payment() {
    for variant in 0..3 {
        let mut g = initial(0);
        let src = board(&mut g, "JC015", 0, 0);
        let target = board(&mut g, "JC096", 2, 1);
        apply(&mut g, 0, activate(&src, "exorcise", Some(target.clone())));
        match variant {
            0 => g.board_mut(&target).unwrap().face_down = true,
            1 => {
                g.remove_board(&target);
                board(&mut g, "JC096", 2, 1);
            }
            _ => turn_control(&mut g, &target, 0, SubtypeChange::None),
        }
        pass_top(&mut g);
        assert!(g.regions[1].cards.iter().any(|c| c.definition == "JC096"));
        assert_eq!(
            g.players[0]
                .graveyard
                .iter()
                .filter(|c| c.definition == "JC015")
                .count(),
            1
        );
        checkpoint(&g);
    }
}
#[test]
fn green_jc015_real_response_destroys_embrace_restores_human_and_invalidates_target() {
    let mut g = initial(0);
    let src = board(&mut g, "JC015", 0, 0);
    let target = board(&mut g, "JC059", 2, 1);
    let embrace = attach(&mut g, "JC036", 2, &target);
    g.add_control(
        &target,
        2,
        ControlLifetime::Attached {
            source_instance: embrace.clone(),
        },
        SubtypeChange::HumanToVampire,
    );
    apply(&mut g, 0, activate(&src, "exorcise", Some(target.clone())));
    for _ in 0..8 {
        if g.priority_team == g.team(2) {
            break;
        }
        let s = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        apply(&mut g, s, Action::new("pass"));
    }
    fund(&mut g, 2, "JC003", 2);
    let spell = hand(&mut g, "JC005", 2);
    let a = play_action(&g, 2, &spell, &embrace);
    apply(&mut g, 2, a);
    pass_top(&mut g);
    assert!(g
        .current_subtypes(g.board(&target).unwrap().1)
        .iter()
        .any(|s| s == "人类"));
    pass_top(&mut g);
    assert!(g.board(&target).is_some());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC015"));
    checkpoint(&g);
}
#[test]
fn green_ms02_paid_unique_search_private_choices_public_reveal_shuffle_and_once() {
    for actor in 0..4 {
        let mut g = initial(actor);
        fund(&mut g, actor, "JC125", 4);
        let gold = g.make_card("LC30", actor);
        let gold_id = gold.id.clone();
        g.players[actor].deck.push(gold);
        let nongold = g.make_card("JC018", actor);
        g.players[actor].deck.push(nongold);
        let id = source(&g, actor);
        apply(&mut g, actor, activate(&id, "search-green-unique", None));
        assert!(g.players[actor]
            .society_zone
            .used_once_per_game
            .contains("search-green-unique"));
        let rng = g.random;
        pass_top(&mut g);
        let p = g.pending.as_ref().unwrap();
        assert_eq!(p.seat, actor);
        assert_eq!(p.choice.options.len(), 1);
        assert_eq!(p.choice.options[0].id, gold_id);
        for s in 0..4 {
            assert_eq!(g.view(s).pending_choice.is_some(), s == actor);
        }
        let mut restored = envelope(&g).game;
        choose(&mut g, vec![gold_id.clone()]);
        choose(&mut restored, vec![gold_id.clone()]);
        assert_eq!(
            serde_json::to_value(&g).unwrap(),
            serde_json::to_value(restored).unwrap()
        );
        assert!(g.players[actor]
            .hand
            .iter()
            .any(|c| c.definition == "LC30" && c.id != gold_id));
        assert_ne!(g.random, rng);
        g.players[actor]
            .society_zone
            .card
            .as_mut()
            .unwrap()
            .exhausted = false;
        fund(&mut g, actor, "JC125", 4);
        reject(&mut g, actor, activate(&id, "search-green-unique", None));
        for viewer in 0..4 {
            assert!(g
                .view(viewer)
                .log
                .iter()
                .any(|e| e.text.contains("展示检索的")));
        }
        checkpoint(&g);
    }
}
#[test]
fn green_ms02_draw_checks_initiative_at_resolution_and_own_three_payment() {
    for first in [0, 1] {
        let mut g = initial(0);
        fund(&mut g, 0, "JC125", 3);
        let id = source(&g, 0);
        apply(&mut g, 0, activate(&id, "drawWithInitiative", None));
        g.first_team = first;
        pass_top(&mut g);
        assert_eq!(g.players[0].hand.len(), usize::from(first == 0));
        assert_eq!(g.resources(0), 0);
        assert!(g.players[0].society_zone.card.as_ref().unwrap().exhausted);
        assert!(g.players[0].society_zone.used_once_per_game.is_empty());
        checkpoint(&g);
    }
}
#[test]
fn green_ms02_terminated_or_empty_search_still_consumes_once_and_does_not_eliminate() {
    for variant in 0..3 {
        let mut g = initial(0);
        fund(&mut g, 0, "JC125", 4);
        let id = source(&g, 0);
        apply(&mut g, 0, activate(&id, "search-green-unique", None));
        if variant == 0 {
            let top = g.stack.last_mut().unwrap();
            top.frame.as_mut().unwrap().guard = GuardState::Cancelled;
        }
        if variant == 2 {
            g.players[0].deck.clear();
        }
        pass_top(&mut g);
        assert!(g.pending.is_none() && !g.players[0].eliminated);
        assert!(g.players[0].hand.is_empty());
        assert!(g.players[0]
            .society_zone
            .used_once_per_game
            .contains("search-green-unique"));
        g.players[0].society_zone.card.as_mut().unwrap().exhausted = false;
        fund(&mut g, 0, "JC125", 4);
        reject(&mut g, 0, activate(&id, "search-green-unique", None));
        checkpoint(&g);
    }
}

#[test]
fn green_ms02_natural_turn_ready_and_source_identity_keep_quota_restart_clears() {
    let mut g = initial(0);
    fund(&mut g, 0, "JC125", 8);
    let id = source(&g, 0);
    apply(&mut g, 0, activate(&id, "search-green-unique", None));
    pass_top(&mut g);
    let turn = g.turn;
    for _ in 0..250 {
        if g.turn > turn {
            break;
        }
        let seat = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .expect("bounded ordinary turn passes");
        apply(&mut g, seat, Action::new("pass"));
    }
    assert!(g.turn > turn);
    assert!(!g.players[0].society_zone.card.as_ref().unwrap().exhausted);
    g.begin_window(Window::Action(0));
    let id = source(&g, 0);
    reject(&mut g, 0, activate(&id, "search-green-unique", None));
    let c = g.players[0].society_zone.card.take().unwrap();
    g.players[0].society_zone.card = Some(g.fresh(c));
    let id = source(&g, 0);
    checkpoint(&g);
    reject(&mut g, 0, activate(&id, "search-green-unique", None));
    g.status = "finished".into();
    apply(&mut g, 0, Action::new("restart"));
    assert!(g
        .players
        .iter()
        .all(|p| p.society_zone.used_once_per_game.is_empty() && p.hand.len() == 6));
    checkpoint(&g);
}

#[test]
fn green_three_cards_borrowed_hidden_controller_privacy_and_full_refresh_views() {
    let mut fixtures = vec![];
    for controller in 0..4 {
        let mut g = initial(controller);
        g.room_id = format!("green-hidden-controller-{controller}");
        let owner = (controller + 1) % 4;
        for def in ["LC30", "JC018", "JC015"] {
            let id = board(&mut g, def, owner, 0);
            // Explicit concealed borrowed layout, not a public room mutation.
            let c = g.board_mut(&id).unwrap();
            c.controller = controller;
            c.face_down = true;
            hand(&mut g, def, owner);
        }
        let room = envelope(&g);
        let state = serde_json::to_string(&room).unwrap();
        let restored = crate::room::RoomEnvelope::from_persisted(&state).unwrap();
        let views = (0..4)
            .map(|seat| {
                let view = room.view(seat, 0);
                assert_eq!(
                    serde_json::to_value(&view).unwrap(),
                    serde_json::to_value(restored.view(seat, 0)).unwrap()
                );
                for c in &view.regions[0].characters {
                    assert_eq!(c.card_id.is_some(), seat == controller);
                    if seat != controller {
                        assert_eq!(c.name, "暗藏者");
                        assert!(c.cost.is_none() && c.text.is_none() && c.defense.is_none());
                    }
                }
                assert_eq!(view.hand.len(), if seat == owner { 3 } else { 0 });
                serde_json::to_value(view).unwrap()
            })
            .collect::<Vec<_>>();
        fixtures.push(
            serde_json::json!({"controller":controller,"owner":owner,"state":state,"views":views}),
        );
        checkpoint(&g);
    }
    let mut g = initial(0);
    g.room_id = "green-dynamic-stat-row".into();
    g.first_team = 1;
    let lc30 = board(&mut g, "LC30", 0, 0);
    attach(&mut g, "BQ022", 2, &lc30);
    attach(&mut g, "JC116", 3, &lc30);
    attach(&mut g, "XQ47", 1, &lc30);
    board(&mut g, "JC018", 0, 0);
    fund(&mut g, 0, "JC003", 1);
    fund(&mut g, 0, "JC004", 1);
    for c in &mut g.players[0].assets {
        c.exhausted = true;
    }
    let room = envelope(&g);
    fixtures.push(serde_json::json!({"kind":"dynamic","state":serde_json::to_string(&room).unwrap(),"views":(0..4).map(|s|serde_json::to_value(room.view(s,0)).unwrap()).collect::<Vec<_>>()}));
    checkpoint(&g);
    if let Ok(path) = std::env::var("GREEN_FRONTEND_FIXTURE_FILE") {
        std::fs::write(path, serde_json::to_vec(&serde_json::json!({"scope":"explicit native hidden borrowed layouts; complete state and four RoomViews, not public gameplay","fixtures":fixtures})).unwrap()).unwrap();
    }
}
