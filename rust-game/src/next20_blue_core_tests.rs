//! Tests of production engine commands and persistence, with explicit initial
//! layouts. No patched kernel, fixture registry, or name-only BQ009 is used.
use crate::{catalog, model::*, next20_blue_vampires as blue, unit_support::*};

fn grave(g: &mut Game, definition: &str, owner: usize) -> String {
    let c = g.make_card(definition, owner);
    let id = c.id.clone();
    g.players[owner].graveyard.push(c);
    id
}
fn attachment(g: &mut Game, definition: &str, controller: usize, host: &str) -> String {
    let c = g.make_card(definition, controller);
    let id = c.id.clone();
    g.attachments.push(Attachment {
        card: c,
        host_id: host.into(),
    });
    id
}
fn until_choice(g: &mut Game) {
    for _ in 0..40 {
        if g.pending.is_some() {
            return;
        }
        assert!(!g.stack.is_empty(), "expected a real response frame");
        pass(g);
    }
    panic!("choice did not arrive");
}
fn enemy_card(g: &mut Game, target: &str) {
    g.begin_window(Window::Action(g.team(2)));
    fund(g, 2, "JC102", 2);
    let card = held(g, "JC102", 2);
    let a = g
        .legal_actions(2)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_ref() == Some(&card)
                && a.action.target_id.as_deref() == Some(target)
        })
        .unwrap()
        .action;
    act(g, 2, a);
}

#[test]
fn next20_blue_core_registry_is_real_and_admission_stays_closed() {
    assert_current_serial_catalog();
    for id in blue::CARDS {
        assert!(!catalog::card(id).supported);
        assert!(catalog::runtime_implemented(catalog::card(id)));
    }
    assert_eq!(catalog::card("BQ034").subtitle.as_deref(), Some("美浓之鬼"));
    assert!(!catalog::catalog().cards.iter().any(|c| c.id == "BQ009"));
    assert!(["CQ25", "XG25"]
        .iter()
        .all(|id| !catalog::card(id).supported));
}

#[test]
fn next20_blue_core_deploy_entry_blood_uses_legal_action_and_roundtrips() {
    let mut g = game();
    fund(&mut g, 0, "JC029", 1);
    fund(&mut g, 0, "JC125", 3);
    let hand = held(&mut g, "BQ034", 0);
    let action = Action {
        card_id: Some(hand.clone()),
        region: Some(0),
        ..Action::new("deploy")
    };
    reject(&mut g, 0, action.clone());
    g.players[0].assets.pop();
    fund(&mut g, 0, "JC029", 1);
    let blood = grave(&mut g, "XQ14", 0);
    let legal = g
        .legal_actions(0)
        .into_iter()
        .find(|a| a.action == action)
        .unwrap()
        .action;
    act(&mut g, 0, legal);
    assert_eq!(g.resources(0), 0);
    until_choice(&mut g);
    choose(&mut g, vec!["accept".into()]);
    until_choice(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, blue::ENTRY);
    choose(&mut g, vec![blood]);
    assert!(g.attachments.iter().any(|a| a.card.definition == "XQ14"
        && g.board(&a.host_id)
            .is_some_and(|(_, c)| c.definition == "BQ034")));
    restore(&mut g);
}

#[test]
fn next20_blue_core_paid_reveal_is_real_and_entry_can_be_declined() {
    let mut g = game();
    fund(&mut g, 0, "JC029", 4);
    let hidden = field(&mut g, "BQ034", 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| a.action.kind == "reveal" && a.action.card_id.as_ref() == Some(&hidden))
        .unwrap()
        .action;
    act(&mut g, 0, a);
    until_choice(&mut g);
    choose(&mut g, vec![]);
    assert!(g.pending.is_none());
    assert!(g.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "BQ034" && !c.face_down));
    assert_eq!(g.resources(0), 0);
}

#[test]
fn next20_blue_core_enemy_transaction_counter_sacrifices_and_never_refunds() {
    let mut g = game();
    field(&mut g, "BQ034", 0);
    let target = field(&mut g, "JC029", 0);
    let blood = attachment(&mut g, "XQ14", 0, &target);
    enemy_card(&mut g, &target);
    choose(&mut g, vec!["accept".into()]);
    until_choice(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, blue::COUNTER);
    choose(&mut g, vec![blood.clone()]);
    assert!(g.board(&target).is_some());
    assert!(g.in_play(&blood).is_none());
    assert!(g.stack.is_empty());
    assert_eq!(g.resources(2), 0);
    assert_eq!(
        g.players[2]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JC102")
            .count(),
        1
    );
}

#[test]
fn next20_blue_core_counter_decline_keeps_attachment_and_original_transaction() {
    let mut g = game();
    field(&mut g, "BQ034", 0);
    let target = field(&mut g, "JC029", 0);
    let blood = attachment(&mut g, "XQ14", 0, &target);
    enemy_card(&mut g, &target);
    choose(&mut g, vec!["accept".into()]);
    until_choice(&mut g);
    choose(&mut g, vec![]);
    assert!(g.in_play(&blood).is_some());
    assert_eq!(g.stack.len(), 1);
    top(&mut g);
    assert_eq!(
        g.players[2]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JC102")
            .count(),
        1
    );
}

#[test]
fn next20_blue_core_targeted_activated_ability_is_not_an_enemy_card() {
    let mut g = game();
    field(&mut g, "BQ034", 0);
    let target = field(&mut g, "JC029", 0);
    let source = field(&mut g, "JC003", 2);
    fund(&mut g, 2, "JC003", 2);
    g.begin_window(Window::Action(g.team(2)));
    let a = g
        .legal_actions(2)
        .into_iter()
        .find(|a| {
            a.action.kind == "activate"
                && a.action.card_id.as_ref() == Some(&source)
                && a.action.target_id.as_ref() == Some(&target)
        })
        .unwrap()
        .action;
    act(&mut g, 2, a);
    assert!(g.pending.is_none());
    assert!(!g.effects.iter().any(
        |e| matches!(e, Effect::Declare { declaration } if declaration.ability.key == blue::COUNTER)
    ));
    top(&mut g);
    assert!(g.board(&target).unwrap().1.exhausted);
}

#[test]
fn next20_blue_core_no_anchor_rejects_deploy_and_paid_reveal_without_mutation() {
    let mut g = game();
    fund(&mut g, 0, "JC029", 6);
    let heat = held(&mut g, "BQ033", 0);
    assert!(!g
        .legal_actions(0)
        .iter()
        .any(|a| a.action.kind == "deploy" && a.action.card_id.as_ref() == Some(&heat)));
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(heat.clone()),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(heat.clone()),
            region: Some(0),
            cost_selected: Some(vec!["not-an-anchor".into()]),
            ..Action::new("deploy")
        },
    );
    let conceal = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.kind == "conceal"
                && a.action.card_id.as_ref() == Some(&heat)
                && a.action.region == Some(0)
        })
        .unwrap()
        .action;
    act(&mut g, 0, conceal);
    let hidden = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "BQ033")
        .unwrap()
        .id
        .clone();
    assert!(!g
        .legal_actions(0)
        .iter()
        .any(|a| a.action.kind == "reveal" && a.action.card_id.as_ref() == Some(&hidden)));
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            cost_selected: Some(vec!["not-an-anchor".into()]),
            ..Action::new("reveal")
        },
    );
}

#[test]
fn next20_blue_core_effect_entry_has_no_play_fee_and_runs_random_discard() {
    let mut g = game();
    let source = held(&mut g, "BQ089", 0);
    fund(&mut g, 0, "BQ089", 2);
    let heat = grave(&mut g, "BQ033", 0);
    held(&mut g, "JC125", 2);
    held(&mut g, "JC125", 2);
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_ref() == Some(&source)
                && serde_json::to_string(&a.action).unwrap().contains(&heat)
        })
        .unwrap()
        .action;
    act(&mut g, 0, a);
    until_choice(&mut g);
    assert!(
        matches!(&g.pending.as_ref().unwrap().resolution, ChoiceResolution::Declare { declaration, .. } if declaration.ability.key == blue::DISCARD)
    );
    choose(&mut g, vec!["p2".into()]);
    top(&mut g);
    assert_eq!(g.players[2].hand.len(), 1);
    assert!(g
        .regions
        .iter()
        .any(|r| r.cards.iter().any(|c| c.definition == "BQ033")));
}

fn shown_case(with_target: bool, with_blood: bool) {
    let mut g = game();
    let vampire = with_target.then(|| field(&mut g, "JC029", 0));
    let blood = with_blood.then(|| grave(&mut g, "XQ14", 0));
    let source = field(&mut g, "WM057", 0);
    g.players[0].society_zone.secrecy = 1;
    fund(&mut g, 0, "JC029", 2);
    for s in 0..4 {
        held(&mut g, if s == 0 { "BQ034" } else { "JC125" }, s);
    }
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            ability_id: Some(crate::import20_broker::ACT.into()),
            ..Action::new("activate")
        },
    );
    let mut fed = false;
    for _ in 0..100 {
        if g.stack.is_empty() && g.pending.is_none() && g.mystic_contest.is_none() {
            break;
        }
        if let Some(p) = g.pending.clone() {
            let selected = match p.choice.kind.as_str() {
                crate::mystic_contest::COMMIT_KIND => vec![g.players[p.seat]
                    .hand
                    .iter()
                    .find(|c| c.definition == if p.seat == 0 { "BQ034" } else { "JC125" })
                    .unwrap()
                    .id
                    .clone()],
                crate::mystic_contest::REWARD_KIND | "trigger" => vec![],
                blue::SHOWN => vec![vampire.clone().unwrap()],
                blue::ENTRY => {
                    fed = true;
                    vec![blood.clone().unwrap()]
                }
                other => panic!("unexpected {other}"),
            };
            choose(&mut g, selected);
        } else {
            pass(&mut g);
        }
    }
    assert!(g.mystic_contest.is_none());
    assert_eq!(fed, with_target && with_blood);
    assert_eq!(g.mystic_history[0].shown[0].value, 5);
    restore(&mut g);
}
#[test]
fn next20_blue_core_real_reveal_contest_shown_target_and_blood_resume() {
    shown_case(true, true);
}
#[test]
fn next20_blue_core_real_contest_without_vampire_has_no_stale_post_frame() {
    shown_case(false, true);
}
#[test]
fn next20_blue_core_real_contest_without_blood_finishes_after_target() {
    shown_case(true, false);
}

#[test]
fn next20_blue_core_restored_counter_rejects_changed_choice_and_response_shell() {
    let mut g = game();
    field(&mut g, "BQ034", 0);
    let target = field(&mut g, "JC029", 0);
    attachment(&mut g, "XQ14", 0, &target);
    enemy_card(&mut g, &target);
    let original = serde_json::to_value(&g).unwrap();
    let mut bad = original.clone();
    bad["pending"]["choice"]["max"] = 2.into();
    assert!(Game::from_persisted(&bad.to_string()).is_err());
    choose(&mut g, vec!["accept".into()]);
    let mut bad = serde_json::to_value(&g).unwrap();
    let response = bad["stack"].as_array_mut().unwrap().last_mut().unwrap();
    response["controller"] = 2.into();
    assert!(Game::from_persisted(&bad.to_string()).is_err());
    restore(&mut g);
}

#[test]
fn next20_blue_core_deploy_recovery_rejects_missing_fee_and_wrong_region() {
    let mut g = game();
    fund(&mut g, 0, "JC029", 4);
    let source = held(&mut g, "BQ034", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    let original = serde_json::to_value(&g).unwrap();
    let mut bad = original.clone();
    bad["stack"][0]["frame"]["already_paid"] = serde_json::json!([]);
    assert!(Game::from_persisted(&bad.to_string()).is_err());
    let mut bad = original;
    bad["stack"][0]["deploy_region"] = 1.into();
    assert!(Game::from_persisted(&bad.to_string()).is_err());
    restore(&mut g);
}

fn changed_transaction_instance_is_rejected(reveal: bool) {
    let mut g = game();
    fund(&mut g, 0, "JC029", 4);
    let source = if reveal {
        let id = field(&mut g, "BQ034", 0);
        g.board_mut(&id).unwrap().face_down = true;
        id
    } else { held(&mut g, "BQ034", 0) };
    act(&mut g, 0, Action { card_id: Some(source.clone()),
        region: (!reveal).then_some(0), ..Action::new(if reveal { "reveal" } else { "deploy" }) });
    let item = &g.stack[0];
    let frame = item.frame.as_ref().unwrap();
    let transaction = item.card.as_ref().unwrap();
    // The engine deliberately freshens the card between source and stack. It
    // must retain the *particular* next-before-frame ID, not literal equality.
    assert_ne!(transaction.id, frame.source.card.id);
    assert_eq!(g.mystic_instance_number(&transaction.id).unwrap() + 1,
        g.mystic_instance_number(&frame.frame_id).unwrap());
    let original = serde_json::to_value(&g).unwrap();
    let mut bad = original.clone();
    bad["stack"][0]["card"]["id"] = source.into();
    assert!(Game::from_persisted(&bad.to_string()).is_err(), "retired source ID must not replace the actual paid transaction");
    let mut bad = original;
    bad["stack"][0]["card"]["damage"] = 1.into();
    assert!(Game::from_persisted(&bad.to_string()).is_err(), "transaction must preserve the complete fresh-card snapshot");
    restore(&mut g);
}
#[test]
fn next20_blue_core_review_deploy_rejects_transaction_instance_swap() {
    changed_transaction_instance_is_rejected(false);
}
#[test]
fn next20_blue_core_review_reveal_rejects_transaction_instance_swap() {
    changed_transaction_instance_is_rejected(true);
}

fn shown_continuation_once(with_target: bool) {
    use crate::mystic_contest::{MysticPhase, COMMIT_KIND, REWARD_KIND};
    let mut g = game();
    let vampire = with_target.then(|| field(&mut g, "JC029", 0));
    // No blood in this case. WM061 is a *later* shown program with a measurable
    // effect: its top card to its owner grave, once and after BQ034 ends.
    let source = field(&mut g, "WM057", 0);
    g.players[0].society_zone.secrecy = 1;
    fund(&mut g, 0, "JC029", 2);
    let printed = ["BQ034", "JC125", "WM061", "JC125"];
    for (seat, id) in printed.iter().enumerate() { held(&mut g, id, seat); }
    let grave_before: Vec<_> = g.players.iter().map(|p| p.graveyard.len()).collect();
    act(&mut g, 0, Action { card_id: Some(source),
        ability_id: Some(crate::import20_broker::ACT.into()), ..Action::new("activate") });
    let mut saw_target = false;
    for _ in 0..100 {
        if g.stack.is_empty() && g.pending.is_none() && g.mystic_contest.is_none() { break; }
        if let Some(p) = g.pending.clone() {
            let selected = match p.choice.kind.as_str() {
                COMMIT_KIND => vec![g.players[p.seat].hand.iter()
                    .find(|c| c.definition == printed[p.seat]).unwrap().id.clone()],
                REWARD_KIND | "trigger" => vec![],
                blue::SHOWN => {
                    saw_target = true;
                    let state = g.mystic_contest.as_ref().unwrap();
                    assert!(matches!(&state.phase, MysticPhase::After { next_shown: 1, post_frame: Some(_), .. }));
                    assert_eq!(g.effects.iter().filter(|e| matches!(e, Effect::MysticContinue { contest_id } if *contest_id == state.contest_id)).count(), 1);
                    assert_eq!(g.players.iter().map(|p|p.graveyard.len()).collect::<Vec<_>>(), grave_before);
                    let original = serde_json::to_value(&g).unwrap();
                    let effects = original["effects"].as_array().unwrap();
                    let index = effects.iter().position(|v| v["kind"] == "MysticContinue" || v.to_string().contains("MysticContinue")).unwrap();
                    let mut duplicate = original.clone();
                    duplicate["effects"].as_array_mut().unwrap().push(effects[index].clone());
                    assert!(Game::from_persisted(&duplicate.to_string()).is_err());
                    let mut missing = original;
                    missing["effects"].as_array_mut().unwrap().remove(index);
                    assert!(Game::from_persisted(&missing.to_string()).is_err());
                    vec![vampire.clone().unwrap()]
                },
                other => panic!("unexpected {other}"),
            };
            let old_choice = Action { choice_id: Some(p.choice.id.clone()), selected: Some(selected.clone()), ..Action::new("choose") };
            choose(&mut g, selected);
            if p.choice.kind == blue::SHOWN { reject(&mut g, p.seat, old_choice); }
        } else { pass(&mut g); }
    }
    assert_eq!(saw_target, with_target);
    assert!(g.mystic_contest.is_none());
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert!(!g.effects.iter().any(|e| matches!(e, Effect::MysticContinue { .. } | Effect::MysticFinish { .. })));
    assert_eq!(g.mystic_history.len(), 1);
    for (seat,(p,before)) in g.players.iter().zip(grave_before).enumerate() {
        assert_eq!(p.graveyard.len(), before + usize::from(seat == 2));
    }
    restore(&mut g);
}
#[test]
fn next20_blue_core_review_no_target_continues_later_shown_exactly_once() {
    shown_continuation_once(false);
}
#[test]
fn next20_blue_core_review_no_blood_continues_later_shown_exactly_once() {
    shown_continuation_once(true);
}
