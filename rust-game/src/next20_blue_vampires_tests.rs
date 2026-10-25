//! Directed adapter tests. The isolated harness supplies registry/wiring
//! suggestions; these are not WASM, Worker, UI or public-admission evidence.
use crate::{catalog, model::*, next20_blue_vampires as blue, rules::*, unit_support::*};

fn grave(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.players[seat].graveyard.push(c);
    id
}
fn entry(g: &mut Game, source: &str, id: &str, target: Option<&str>) -> ResolutionFrame {
    let (r, c) = g.board(source).unwrap();
    let source = g.next20_source(c, r);
    let a = blue::definition(id).unwrap().abilities.remove(0);
    let targets = target
        .map(|id| {
            let spec = a.targets[0].clone();
            BoundTarget {
                id: id.into(),
                public: g.public_target(source.card.controller, &source, &spec, id),
                spec,
                region_instance: None,
            }
        })
        .into_iter()
        .collect();
    ResolutionFrame {
        resource_payments: vec![],
        frame_id: g.id(),
        ability_key: a.key,
        actor: source.card.controller,
        source,
        targets,
        already_paid: vec![],
        guard: GuardState::Accepted,
        cursor: 1,
        steps: a
            .ops
            .into_iter()
            .map(|op| Step { context: 0, op })
            .collect(),
        chosen_region: None,
    }
}
fn cast_frame(g: &mut Game, target: &str) -> ResolutionFrame {
    // An explicit already-paid enemy card transaction fixture, with a real
    // finite printed spell body and ordinary binding/frame construction.
    let c = g.make_card("JC102", 2);
    let source = g.source_snapshot(&c, None);
    let a = crate::rules::definition("JC102").abilities[0].clone();
    let command = Action {
        target_id: Some(target.into()),
        ..Action::new("play")
    };
    let targets = g.bind_action(2, &source, &a, &command).unwrap();
    let f = g.make_frame(2, source, &a, targets, vec![], None);
    g.push_frame(f, "targeted enemy card fixture".into(), Some(c));
    g.stack.last().unwrap().frame.clone().unwrap()
}
fn observed_counter(g: &mut Game, tx: &ResolutionFrame) -> ResolutionFrame {
    if !g.effects.iter().any(
        |e| matches!(e,Effect::Declare { declaration } if declaration.ability.key == blue::COUNTER),
    ) {
        g.bq034_observe_card(tx);
    }
    let d = g
        .effects
        .iter()
        .find_map(|e| match e {
            Effect::Declare { declaration } if declaration.ability.key == blue::COUNTER => {
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
fn attach(g: &mut Game, id: &str, actor: usize, host: &str) -> String {
    let card = g.make_card(id, actor);
    let id = card.id.clone();
    g.attachments.push(Attachment {
        card,
        host_id: host.into(),
    });
    id
}
#[test]
fn next20_blue_definitions_reject_transplant_missing_trigger_and_extra_program() {
    for id in blue::CARDS {
        let d = blue::definition(id).unwrap();
        blue::validate_definition(id, &d).unwrap();
        for a in &d.abilities {
            blue::validate_ability(id, a).unwrap();
            assert!(blue::validate_ability("JC125", a).is_err());
        }
        let mut bad = d.clone();
        bad.abilities.clear();
        assert!(blue::validate_definition(id, &bad).is_err());
        let mut bad = d.abilities[0].clone();
        bad.event = None;
        assert!(blue::validate_ability(id, &bad).is_err());
        let mut bad = d.abilities[0].clone();
        bad.ops.push(Op::Draw {
            player: PlayerRef::Actor,
            count: 1,
            end: DeckEnd::Top,
        });
        assert!(blue::validate_ability(id, &bad).is_err());
    }
    assert!(blue::definition("BQ033").unwrap().traits.spirit);
    assert_eq!(
        (blue::shown_bonus("BQ034"), blue::shown_bonus("BQ033")),
        (1, 0)
    );
}
#[test]
fn next20_blue_bq033_extra_payment_is_exact_own_society_and_returns_to_real_owner() {
    let mut g = game();
    let c = g.make_card("BQ009", 1);
    g.seal_card_on_valid_host(c, 0, "society:p0");
    let eligible = g.sealed_cards[0].card.id.clone();
    let c = g.make_card("BQ009", 0);
    g.seal_card_on_valid_host(c, 1, "society:p1");
    let c = g.make_card("XQ14", 0);
    g.seal_card_on_valid_host(c, 0, "society:p0");
    let holder = field(&mut g, "JC029", 0);
    let c = g.make_card("BQ009", 0);
    g.seal_card_on_valid_host(c, 0, &holder);
    assert_eq!(g.bq033_anchor_options(0), vec![eligible.clone()]);
    let old0 = g.players[0].deck.len();
    let old1 = g.players[1].deck.len();
    let receipt = g.bq033_pay_anchor(0, &eligible).unwrap();
    g.validate_bq033_payment(&receipt).unwrap();
    assert_eq!(
        (g.players[0].deck.len(), g.players[1].deck.len()),
        (old0, old1 + 1)
    );
    assert!(g.players[1]
        .deck
        .iter()
        .any(|c| c.id == receipt.deck_instance && c.definition == "BQ009" && c.owner == 1));
    assert!(!g.sealed_cards.iter().any(|s| s.card.id == eligible));
    assert!(!g
        .players
        .iter()
        .any(|p| p.hand.iter().any(|c| c.definition == "BQ009")));
    let mut bad = receipt.clone();
    bad.payer = 1;
    assert!(g.validate_bq033_payment(&bad).is_err());
    let mut bad = receipt;
    bad.deck_instance = bad.anchor.id.clone();
    assert!(g.validate_bq033_payment(&bad).is_err());
}
#[test]
fn next20_blue_bq033_invalid_anchor_does_not_mutate_or_spend() {
    let mut g = game();
    let before = serde_json::to_value(&g).unwrap();
    assert!(g.bq033_pay_anchor(0, "not-a-seal").is_err());
    assert_eq!(serde_json::to_value(&g).unwrap(), before);
    assert!(g.bq033_anchor_options(99).is_empty());
}
#[test]
fn next20_blue_bq033_random_discard_is_one_real_hand_card_and_empty_hand_is_noop() {
    for seat in 0..4 {
        let mut g = game();
        let source = field(&mut g, "BQ033", 0);
        for id in ["JC125", "JC029", "XQ14"] {
            held(&mut g, id, seat);
        }
        let before = g.players[seat]
            .hand
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        let f = entry(&mut g, &source, "BQ033", Some(&format!("p{seat}")));
        g.bq033_random_discard(&f).unwrap();
        assert_eq!(g.players[seat].hand.len(), 2);
        assert_eq!(g.players[seat].graveyard.len(), 1);
        assert!(!before.contains(&g.players[seat].graveyard[0].id));
        for other in (0..4).filter(|s| *s != seat) {
            assert!(g.players[other].graveyard.is_empty());
        }
        g.players[seat].hand.clear();
        let before = serde_json::to_value(&g).unwrap();
        g.bq033_random_discard(&f).unwrap();
        assert_eq!(serde_json::to_value(&g).unwrap(), before);
    }
}
#[test]
fn next20_blue_entry_feeds_from_current_controllers_grave_or_society() {
    for sealed in [false, true] {
        let mut g = game();
        let source = field(&mut g, "BQ034", 0);
        let f = entry(&mut g, &source, "BQ034", None);
        let own0 = grave(&mut g, "XQ14", 0);
        let blood = if sealed {
            let c = g.make_card("XQ14", 1);
            g.seal_card_on_valid_host(c, 1, "society:p1");
            g.sealed_cards[0].card.id.clone()
        } else {
            grave(&mut g, "XQ14", 1)
        };
        g.board_mut(&source).unwrap().controller = 1;
        g.bq034_feed_start(f.clone(), source.clone()).unwrap();
        let p = g.pending.as_ref().unwrap();
        assert_eq!((p.seat, p.choice.min, p.choice.max), (1, Some(1), Some(1)));
        assert!(!p.choice.options.iter().any(|o| o.id == own0));
        g.bq034_feed_finish(&f, &source, 1, &[blood]).unwrap();
        assert!(g
            .attachments
            .iter()
            .any(|a| a.host_id == source && a.card.controller == 1 && a.card.definition == "XQ14"));
    }
}
#[test]
fn next20_blue_feed_never_uses_hand_or_wrong_recipient() {
    let mut g = game();
    let source = field(&mut g, "BQ034", 0);
    let f = entry(&mut g, &source, "BQ034", None);
    let hand = held(&mut g, "XQ14", 0);
    let other = field(&mut g, "JC029", 0);
    assert!(g.bq034_feed_start(f.clone(), other).is_err());
    g.bq034_feed_start(f.clone(), source.clone()).unwrap();
    assert!(g.pending.is_none());
    assert!(g.bq034_feed_finish(&f, &source, 0, &[hand]).is_err());
}
#[test]
fn next20_blue_counter_sacrifices_exact_attachment_and_stops_original_card_without_refund() {
    let mut g = game();
    field(&mut g, "BQ034", 0);
    let target = field(&mut g, "JC029", 0);
    let blood = attach(&mut g, "XQ14", 0, &target);
    let tx = cast_frame(&mut g, &target);
    let f = observed_counter(&mut g, &tx);
    let resource = g.resources(2);
    g.bq034_counter_start(f.clone()).unwrap();
    assert_eq!(
        g.pending
            .as_ref()
            .unwrap()
            .choice
            .options
            .iter()
            .map(|o| &o.id)
            .collect::<Vec<_>>(),
        vec![&blood]
    );
    g.bq034_counter_finish(&f, 0, &[blood.clone()]).unwrap();
    assert!(g.board(&target).is_some());
    assert!(g.in_play(&blood).is_none());
    assert!(!g.stack.iter().any(|s| s.id == tx.frame_id));
    assert_eq!(g.resources(2), resource);
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC102"));
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ14"));
}
#[test]
fn next20_blue_counter_decline_does_not_sacrifice_or_stop_card() {
    let mut g = game();
    field(&mut g, "BQ034", 0);
    let target = field(&mut g, "JC029", 0);
    let blood = attach(&mut g, "XQ14", 0, &target);
    let tx = cast_frame(&mut g, &target);
    let f = observed_counter(&mut g, &tx);
    let before = serde_json::to_value(&g).unwrap();
    g.bq034_counter_finish(&f, 0, &[]).unwrap();
    assert_eq!(serde_json::to_value(&g).unwrap(), before);
    assert!(g.in_play(&blood).is_some());
}
#[test]
fn next20_blue_counter_does_not_take_teammates_attachment_or_other_host() {
    let mut g = game();
    field(&mut g, "BQ034", 0);
    let target = field(&mut g, "JC029", 0);
    let other = field(&mut g, "JC029", 0);
    let foreign = attach(&mut g, "XQ14", 1, &target);
    let wrong = attach(&mut g, "XQ14", 0, &other);
    let tx = cast_frame(&mut g, &target);
    let f = observed_counter(&mut g, &tx);
    let before = serde_json::to_value(&g).unwrap();
    assert!(g.bq034_counter_finish(&f, 0, &[foreign]).is_err());
    assert!(g.bq034_counter_finish(&f, 0, &[wrong]).is_err());
    assert_eq!(serde_json::to_value(&g).unwrap(), before);
}
#[test]
fn next20_blue_counter_keeps_declared_observer_after_departure_but_missing_card_costs_nothing() {
    let mut g = game();
    let source = field(&mut g, "BQ034", 0);
    let target = field(&mut g, "JC029", 0);
    let blood = attach(&mut g, "XQ14", 0, &target);
    let tx = cast_frame(&mut g, &target);
    let f = observed_counter(&mut g, &tx);
    g.remove_dead(&source, RemovalCause::Destroy);
    g.bq034_counter_finish(&f, 0, &[blood]).unwrap();
    assert!(!g.stack.iter().any(|s| s.id == tx.frame_id));
    let mut g = game();
    field(&mut g, "BQ034", 0);
    let target = field(&mut g, "JC029", 0);
    let blood = attach(&mut g, "XQ14", 0, &target);
    let tx = cast_frame(&mut g, &target);
    let f = observed_counter(&mut g, &tx);
    g.stack.clear();
    g.bq034_counter_start(f).unwrap();
    assert!(g.pending.is_none());
    assert!(g.in_play(&blood).is_some());
}
#[test]
fn next20_blue_target_observer_is_personal_vampire_and_unique_per_bound_instance() {
    for (id, seat, expected) in [("JC125", 0, 0), ("JC029", 1, 0), ("JC029", 0, 1)] {
        let mut g = game();
        field(&mut g, "BQ034", 0);
        let target = field(&mut g, id, seat);
        let mut tx = cast_frame(&mut g, &target);
        tx.targets.push(tx.targets[0].clone());
        g.effects.retain(|e| !matches!(e,Effect::Declare { declaration } if declaration.ability.key == blue::COUNTER));
        g.bq034_observe_card(&tx);
        assert_eq!(g.effects.iter().filter(|e| matches!(e,Effect::Declare { declaration } if declaration.ability.key == blue::COUNTER)).count(),expected);
    }
}
#[test]
fn next20_blue_counter_corruption_is_rejected_before_sacrifice() {
    let mut g = game();
    field(&mut g, "BQ034", 0);
    let target = field(&mut g, "JC029", 0);
    let blood = attach(&mut g, "XQ14", 0, &target);
    let tx = cast_frame(&mut g, &target);
    let f = observed_counter(&mut g, &tx);
    for mutation in 0..5 {
        let mut f = f.clone();
        match mutation {
            0 => f.source.card.definition = "JC125".into(),
            1 => f.steps.push(f.steps[0].clone()),
            2 => f.steps[0].context = 1,
            3 => f.actor = 1,
            _ => {
                let Op::Next20Blue(blue::BlueProgram::Counter {
                    observation: Some(o),
                }) = &mut f.steps[0].op
                else {
                    panic!()
                };
                o.subtypes.clear();
            }
        }
        let before = serde_json::to_value(&g).unwrap();
        assert!(g.bq034_counter_finish(&f, 0, &[blood.clone()]).is_err());
        assert_eq!(serde_json::to_value(&g).unwrap(), before);
    }
}
#[test]
fn next20_blue_printed_fields_are_image_specific_and_never_publicly_admitted() {
    let dream = catalog::card("BQ033");
    assert_eq!(
        (dream.cost, dream.defense, dream.unique, dream.supported),
        (3, Some(1), false, false)
    );
    let yayoi = catalog::card("BQ034");
    assert_eq!(
        (yayoi.cost, yayoi.defense, yayoi.unique, yayoi.supported),
        (4, Some(1), true, false)
    );
    assert_eq!(yayoi.loyalty, vec!["蓝色", "蓝色"]);
    assert_eq!(yayoi.subtitle.as_deref(), Some("美浓之鬼"));
    assert_eq!(yayoi.temporary_icons.investigation, 1);
}
#[test]
fn next20_blue_pending_roundtrip_preserves_current_blood_options_and_rejects_forgery() {
    let mut g = game();
    let source = field(&mut g, "BQ034", 0);
    let f = entry(&mut g, &source, "BQ034", None);
    grave(&mut g, "XQ14", 0);
    g.bq034_feed_start(f, source).unwrap();
    let p = g.pending.as_ref().unwrap().clone();
    g.validate_next20_blue_pending(&p).unwrap();
    let saved = serde_json::to_string(&p).unwrap();
    let restored: Pending = serde_json::from_str(&saved).unwrap();
    g.validate_next20_blue_pending(&restored).unwrap();
    for n in 0..5 {
        let mut bad = p.clone();
        match n {
            0 => bad.seat = 1,
            1 => bad.choice.options.clear(),
            2 => bad.choice.max = Some(2),
            3 => bad.choice.allow_decline = Some(true),
            _ => {
                let ChoiceResolution::Frame {
                    choice: FrameChoice::Next20Blue(blue::BlueChoice::Blood { recipient }),
                    ..
                } = &mut bad.resolution
                else {
                    panic!()
                };
                *recipient = "i1".into();
            }
        }
        assert!(g.validate_next20_blue_pending(&bad).is_err());
    }
    let mut wrong_continuation = p.clone();
    if let ChoiceResolution::Frame { choice, .. } = &mut wrong_continuation.resolution {
        *choice = FrameChoice::VampireBlood;
    }
    assert!(g.validate_next20_blue_pending(&wrong_continuation).is_err());
    let a = Action {
        amount: Some(99),
        ..Action::new("choose")
    };
    assert!(blue::validate_choice_action(&p, &a).is_err());
}
#[test]
fn next20_blue_actions_require_mandatory_anchor_on_play_and_paid_reveal() {
    let mut deploy = Action {
        card_id: Some("i1".into()),
        region: Some(0),
        ..Action::new("deploy")
    };
    assert!(blue::validate_action("BQ033", &deploy).is_err());
    deploy.cost_selected = Some(vec!["i2".into()]);
    blue::validate_action("BQ033", &deploy).unwrap();
    assert!(blue::validate_action("BQ034", &deploy).is_err());
    let hide = Action {
        card_id: Some("i1".into()),
        region: Some(0),
        ..Action::new("conceal")
    };
    blue::validate_action("BQ033", &hide).unwrap();
    let mut reveal = Action {
        card_id: Some("i1".into()),
        ..Action::new("reveal")
    };
    assert!(blue::validate_action("BQ033", &reveal).is_err());
    reveal.cost_selected = Some(vec!["i2".into()]);
    blue::validate_action("BQ033", &reveal).unwrap();
}
#[test]
fn next20_blue_shown_post_effect_uses_real_contest_and_resumes_after_blood() {
    let mut g = game();
    let vampire = field(&mut g, "JC029", 0);
    let blood = grave(&mut g, "XQ14", 0);
    let source = field(&mut g, "WM022", 0);
    for seat in 0..4 {
        held(&mut g, if seat == 0 { "BQ034" } else { "JC125" }, seat);
    }
    let (r, c) = g.board(&source).unwrap();
    let source = g.source_snapshot(c, Some(r));
    let a = crate::pack_a_contest::definition("WM022")
        .unwrap()
        .abilities
        .remove(0);
    let mut f = g.make_frame(0, source, &a, vec![], vec![], None);
    f.frame_id = g.id();
    f.cursor = 1;
    f.guard = GuardState::Accepted;
    // Directly start the existing protocol from an authentic canonical entry
    // frame; all commitments, rewards and post choices use Game::apply below.
    g.mystic_contest_start(f, 0).unwrap();
    g.drive().unwrap();
    let mut fed = false;
    for _ in 0..80 {
        if g.mystic_contest.is_none() {
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
                crate::mystic_contest::REWARD_KIND => vec![],
                blue::SHOWN => vec![vampire.clone()],
                blue::ENTRY => {
                    fed = true;
                    vec![blood.clone()]
                }
                "trigger" => vec![],
                other => panic!("unexpected choice {other}"),
            };
            choose(&mut g, selected);
        } else if !g.stack.is_empty() {
            pass(&mut g);
        } else {
            g.drive().unwrap();
        }
    }
    assert!(fed);
    assert!(g.mystic_contest.is_none());
    assert_eq!(g.mystic_history[0].shown[0].value, 5);
    assert!(g
        .attachments
        .iter()
        .any(|a| a.host_id == vampire && a.card.definition == "XQ14"));
    restore(&mut g);
}
#[test]
fn next20_blue_state_rejects_duplicate_target_observations_and_response_shell_edits() {
    let mut g = game();
    field(&mut g, "BQ034", 0);
    let target = field(&mut g, "JC029", 0);
    let tx = cast_frame(&mut g, &target);
    let _ = observed_counter(&mut g, &tx);
    g.validate_next20_blue_state().unwrap();
    let duplicate = g.effects.iter().find(|e| matches!(e,Effect::Declare { declaration } if declaration.ability.key == blue::COUNTER)).unwrap().clone();
    let mut bad = g.clone();
    bad.effects.push_back(duplicate);
    assert!(bad.validate_next20_blue_state().is_err());
    let mut bad = g.clone();
    let Effect::Declare { declaration } = bad.effects.iter_mut().find(|e| matches!(e,Effect::Declare { declaration } if declaration.ability.key == blue::COUNTER)).unwrap() else { panic!() };
    declaration.ability.ops.push(Op::Draw {
        player: PlayerRef::Actor,
        count: 9,
        end: DeckEnd::Top,
    });
    assert!(bad.validate_next20_blue_state().is_err());
}
#[test]
fn next20_blue_counter_normal_enemy_play_trigger_response_choice_and_room_restore() {
    let mut g = game();
    field(&mut g, "BQ034", 0);
    let vampire = field(&mut g, "JC029", 0);
    let blood = attach(&mut g, "XQ14", 0, &vampire);
    g.begin_window(Window::Action(g.team(2)));
    fund(&mut g, 2, "JC102", catalog::card("JC102").cost as usize);
    let spell = held(&mut g, "JC102", 2);
    act(
        &mut g,
        2,
        Action {
            card_id: Some(spell),
            target_id: Some(vampire.clone()),
            ..Action::new("play")
        },
    );
    let p = g.pending.as_ref().expect("real card-target trigger");
    assert!(
        matches!(&p.resolution,ChoiceResolution::Declare { declaration,.. } if declaration.ability.key == blue::COUNTER)
    );
    choose(&mut g, vec!["accept".into()]);
    for _ in 0..40 {
        if g.pending.is_some() {
            break;
        }
        pass(&mut g);
    }
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, blue::COUNTER);
    choose(&mut g, vec![blood.clone()]);
    assert!(g.board(&vampire).is_some());
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
    crate::jc029_tests::checkpoint(&g);
    restore(&mut g);
}
#[test]
fn next20_blue_bq034_normal_deploy_requires_two_blue_then_entry_feeds() {
    let mut g = game();
    fund(&mut g, 0, "JC029", 1);
    fund(&mut g, 0, "JC125", 3);
    let hand = held(&mut g, "BQ034", 0);
    let command = Action {
        card_id: Some(hand),
        region: Some(0),
        ..Action::new("deploy")
    };
    reject(&mut g, 0, command.clone());
    g.players[0].assets.pop();
    fund(&mut g, 0, "JC029", 1);
    let blood = grave(&mut g, "XQ14", 0);
    act(&mut g, 0, command);
    assert_eq!(g.resources(0), 0);
    for _ in 0..40 {
        if g.pending.is_some() {
            break;
        }
        pass(&mut g);
    }
    assert!(
        matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare { declaration,.. } if declaration.ability.key == blue::ENTRY)
    );
    choose(&mut g, vec!["accept".into()]);
    for _ in 0..40 {
        if g.pending.is_some() {
            break;
        }
        pass(&mut g);
    }
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, blue::ENTRY);
    choose(&mut g, vec![blood]);
    let source = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "BQ034")
        .unwrap()
        .id
        .clone();
    assert!(g
        .attachments
        .iter()
        .any(|a| a.host_id == source && a.card.definition == "XQ14"));
    restore(&mut g);
}
