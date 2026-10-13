//! Explicit offline layouts; real paid commands, private hand choice and entry.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, model::*, rules::*};
const KEY: &str = "wm023-unique-hand-entry";
fn held(g: &mut Game, owner: usize, actor: usize, def: &str) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    id
}
fn action(id: &str, r: usize) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(KEY.into()),
        region: Some(r),
        ..Action::new("activate")
    }
}
fn setup(actor: usize) -> (Game, String) {
    let mut g = game(actor);
    fund(&mut g, actor, "JC029", 5);
    let id = held(&mut g, actor, actor, "WM023");
    apply(
        &mut g,
        actor,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    let id = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "WM023")
        .unwrap()
        .id
        .clone();
    assert!(g.board(&id).unwrap().1.exhausted);
    // Prepared ready source; the printed slow entry above used the actual production path.
    g.board_mut(&id).unwrap().exhausted = false;
    (g, id)
}
fn pick(g: &Game, id: &str) -> Action {
    Action {
        choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
        selected: Some(vec![id.into()]),
        ..Action::new("choose")
    }
}
fn snap(name: &str, g: &Game, a: &Action) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("WM023_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{name}.json"),serde_json::to_vec(&serde_json::json!({"scope":"offline-wm023-native","state":serde_json::to_string(&r).unwrap(),"action":a,"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()})).unwrap()).unwrap();
    }
}
fn invalid(g: &Game, bad: Game, name: &str) {
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut r = envelope(g);
    r.game = bad;
    let state = serde_json::to_string(&r).unwrap();
    assert!(crate::room::RoomEnvelope::from_persisted(&state).is_err());
    if let Ok(dir) = std::env::var("WM023_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/{name}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn wm023_whole_original_public_slow_unique_and_closed_hand_entry_program() {
    let c = catalog::card("WM023");
    assert_eq!(
        (&*c.name, c.subtitle.as_deref(), c.cost, c.defense),
        ("尽职的珍珠", Some("庄园仆役"), 4, Some(1))
    );
    assert_eq!(c.loyalty, vec!["蓝色"]);
    assert_eq!(c.subtypes, vec!["人类", "奴仆"]);
    assert!(c.unique && c.magic.is_empty());
    assert_eq!(c.keywords, vec!["公开", "迟缓"]);
    assert_eq!(
        c.permanent_icons,
        Icons {
            investigation: 0,
            combat: 0,
            influence: 1
        }
    );
    assert_eq!(c.temporary_icons, Icons::default());
    let d = definition("WM023");
    assert!(d.traits.public && d.traits.slow);
    assert_eq!(d.abilities[0].timing, Timing::Standard);
    for n in 0..10 {
        let mut bad = d.clone();
        match n {
            0 => bad.traits.slow = false,
            1 => bad.traits.public = false,
            2 => bad.abilities[0].costs = vec![Cost::Assets(1)],
            3 => bad.abilities[0].timing = Timing::Fast,
            4 => bad.abilities[0].targets[0].range = Range::SourceRegion,
            5 => bad.abilities[0].per_turn_limit = Some(1),
            6 => bad.abilities[0].activation_only = false,
            7 => bad.abilities[0].response_policy = ResponsePolicy::Immediate,
            8 => bad.abilities[0].event = Some(Event::Enter),
            _ => bad.abilities[0].targets.clear(),
        }
        let mut ds = definitions().clone();
        ds.insert("WM023".into(), bad);
        assert!(crate::rules::validate_definitions(&ds).is_err());
    }
    let mut ds = definitions().clone();
    ds.insert("JC125".into(), d.clone());
    assert!(crate::rules::validate_definitions(&ds).is_err());
    crate::rules::validate_definitions(definitions()).unwrap();
}
#[test]
fn wm023_union_is_unique_character_and_mage_or_vampire_not_domain_or_attachment() {
    let mut d = catalog::card("JZ27").clone();
    for subtype in ["法师", "吸血鬼"] {
        d.subtypes = vec![subtype.into()];
        assert!(crate::wm023::eligible(&d));
    }
    d.subtypes = vec!["人类".into()];
    assert!(!crate::wm023::eligible(&d));
    d.subtypes = vec!["法师".into()];
    d.unique = false;
    assert!(!crate::wm023::eligible(&d));
    d.unique = true;
    d.kind = "attachment".into();
    assert!(!crate::wm023::eligible(&d));
}
#[test]
fn wm023_actual_public_slow_deploy_cost_loyalty_no_secret_and_next_turn_ready() {
    let mut g = game(0);
    fund(&mut g, 0, "JC093", 4);
    let id = held(&mut g, 0, 0, "WM023");
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id.clone()),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id.clone()),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
    let (mut g, id) = setup(0);
    g.board_mut(&id).unwrap().exhausted = true;
    g.effects.push_back(Effect::NextTurn);
    g.drive().unwrap();
    assert!(!g.board(&id).unwrap().1.exhausted);
    checkpoint(&g);
}
#[test]
fn wm023_all_seats_paid_target_then_actual_mage_or_vampire_hand_entry_without_printed_loyalty_or_reveal(
) {
    for actor in 0..4 {
        for def in ["JC094", "JZ27"] {
            let (mut g, id) = setup(actor);
            let card = held(&mut g, actor, actor, def);
            if actor == 0 && def == "JC094" {
                snap("activation", &g, &action(&id, 4));
            }
            apply(&mut g, actor, action(&id, 4));
            assert_eq!(g.resources(actor), 0);
            assert!(g.board(&id).unwrap().1.exhausted);
            pass_top(&mut g);
            assert_eq!(g.pending.as_ref().unwrap().seat, actor);
            assert_eq!(g.pending.as_ref().unwrap().choice.options[0].id, card);
            if actor == 0 {
                snap(
                    if def == "JC094" {
                        "hand-mage"
                    } else {
                        "hand-vampire"
                    },
                    &g,
                    &pick(&g, &card),
                );
            }
            choose(&mut g, vec![card.clone()]);
            assert!(!g.players[actor].hand.iter().any(|c| c.id == card));
            let c = g.regions[4]
                .cards
                .iter()
                .find(|c| c.definition == def)
                .unwrap();
            assert_ne!(c.id, card);
            assert_eq!(c.controller, actor);
            assert!(!c.face_down);
            assert!(g.pending.is_none()); // JZ27 reveal control must not fire on put-into-play.
            checkpoint(&g);
        }
    }
}
#[test]
fn wm023_private_current_hand_filter_choice_counts_teammate_and_noneligible_rejected() {
    let (mut g, id) = setup(0);
    let good = held(&mut g, 0, 0, "JC094");
    let wrong = held(&mut g, 0, 0, "JC096");
    held(&mut g, 0, 0, "JC045");
    held(&mut g, 1, 1, "JZ27");
    apply(&mut g, 0, action(&id, 1));
    pass_top(&mut g);
    let p = g.pending.as_ref().unwrap();
    assert_eq!(p.choice.options.len(), 1);
    assert_eq!(p.choice.options[0].id, good);
    let r = envelope(&g);
    for seat in 1..4 {
        assert!(r.view(seat, 0).pending_choice.is_none());
    }
    let a = pick(&g, &good);
    reject(&mut g, 1, a);
    let a = pick(&g, &wrong);
    reject(&mut g, 0, a);
    let mut a = pick(&g, &good);
    a.selected = Some(vec![]);
    reject(&mut g, 0, a);
    let mut a = pick(&g, &good);
    a.selected = Some(vec![good.clone(), good.clone()]);
    reject(&mut g, 0, a);
    choose(&mut g, vec![good]);
    checkpoint(&g);
}
#[test]
fn wm023_no_eligible_hand_keeps_payment_exhaustion_and_does_not_invent_activation_gate() {
    let (mut g, id) = setup(0);
    held(&mut g, 0, 0, "JC096");
    apply(&mut g, 0, action(&id, 2));
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert!(g.board(&id).unwrap().1.exhausted);
    assert_eq!(g.resources(0), 0);
    checkpoint(&g);
}
#[test]
fn wm023_exhaustion_assets_wrong_actor_timing_and_extra_payload_reject_atomically() {
    let (mut g, id) = setup(0);
    g.board_mut(&id).unwrap().exhausted = true;
    reject(&mut g, 0, action(&id, 4));
    g.board_mut(&id).unwrap().exhausted = false;
    g.players[0].assets.clear();
    reject(&mut g, 0, action(&id, 4));
    fund(&mut g, 0, "JC029", 1);
    reject(&mut g, 1, action(&id, 4));
    g.begin_window(Window::Before(2, 0));
    reject(&mut g, 0, action(&id, 4));
    g.begin_window(Window::Action(0));
    let mut a = action(&id, 4);
    a.target_id = Some("hand-card".into());
    reject(&mut g, 0, a);
    assert!(!g.board(&id).unwrap().1.exhausted);
    assert_eq!(g.resources(0), 1);
    checkpoint(&g);
}
#[test]
fn wm023_real_source_hide_response_preserves_actor_hand_and_original_target_region() {
    let (mut g, id) = setup(0);
    let card = held(&mut g, 0, 0, "JC094");
    let responder = board(&mut g, "JC063", 0, 2);
    apply(&mut g, 0, action(&id, 4));
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(responder),
            ability_id: Some("chase".into()),
            option: Some("hide".into()),
            target_id: Some(id.clone()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    assert!(g.board(&id).is_none());
    pass_top(&mut g);
    snap("hand-after-source-hide", &g, &pick(&g, &card));
    choose(&mut g, vec![card]);
    assert!(g.regions[4].cards.iter().any(|c| c.definition == "JC094"));
    checkpoint(&g);
}
#[test]
fn wm023_source_control_departure_or_movement_does_not_switch_hand_owner_or_destination() {
    for n in 0..3 {
        let (mut g, id) = setup(0);
        let card = held(&mut g, 0, 0, "JC094");
        held(&mut g, 2, 2, "JZ27");
        apply(&mut g, 0, action(&id, 4));
        match n {
            0 => g.board_mut(&id).unwrap().controller = 2,
            1 => {
                let (_, c) = g.remove_board(&id).unwrap();
                g.players[0].hand.push(c);
            }
            _ => {
                let (_, c) = g.remove_board(&id).unwrap();
                g.regions[0].cards.push(c);
            }
        }
        pass_top(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().seat, 0);
        choose(&mut g, vec![card]);
        assert_eq!(
            g.regions[4]
                .cards
                .iter()
                .find(|c| c.definition == "JC094")
                .unwrap()
                .controller,
            0
        );
        checkpoint(&g);
    }
}
#[test]
fn wm023_target_region_replacement_fizzles_without_consuming_hand_or_refunding_costs() {
    let (mut g, id) = setup(0);
    let card = held(&mut g, 0, 0, "JC094");
    apply(&mut g, 0, action(&id, 4));
    g.regions[4].card.id = "replacement-region".into();
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == card));
    assert_eq!(g.resources(0), 0);
    assert!(g.board(&id).unwrap().1.exhausted);
    checkpoint(&g);
}
#[test]
fn wm023_resolution_uses_current_hand_and_retains_foreign_physical_owner() {
    let (mut g, id) = setup(0);
    let card = held(&mut g, 0, 0, "JC094");
    apply(&mut g, 0, action(&id, 4));
    let c = g.players[0].hand.pop().unwrap();
    g.players[0].graveyard.push(c);
    let replacement = held(&mut g, 2, 0, "JZ27");
    pass_top(&mut g);
    assert_eq!(
        g.pending.as_ref().unwrap().choice.options[0].id,
        replacement
    );
    choose(&mut g, vec![replacement]);
    let c = g.regions[4]
        .cards
        .iter()
        .find(|c| c.definition == "JZ27")
        .unwrap();
    assert_eq!((c.owner, c.controller), (2, 0));
    assert!(!g.players[0].hand.iter().any(|c| c.id == card));
    checkpoint(&g);
}
#[test]
fn wm023_put_entry_runs_real_same_name_unique_sacrifice_and_normal_entry_trigger() {
    let (mut g, id) = setup(0);
    let old = board(&mut g, "JC094", 0, 1);
    let card = held(&mut g, 0, 0, "JC094");
    apply(&mut g, 0, action(&id, 4));
    pass_top(&mut g);
    choose(&mut g, vec![card]);
    assert!(matches!(
        g.pending.as_ref().unwrap().resolution,
        ChoiceResolution::Unique
    ));
    choose(&mut g, vec![old]);
    assert!(g.regions[4].cards.iter().any(|c| c.definition == "JC094"));
    checkpoint(&g);
    let (mut g, id) = setup(0);
    let card = held(&mut g, 0, 0, "JZ02");
    apply(&mut g, 0, action(&id, 4));
    pass_top(&mut g);
    choose(&mut g, vec![card]);
    let p = g.pending.as_ref().unwrap();
    let ChoiceResolution::Declare { declaration, .. } = &p.resolution else {
        panic!("entry missing")
    };
    assert_eq!(declaration.source.card.definition, "JZ02");
    assert_eq!(declaration.ability.event, Some(Event::Enter));
    choose(&mut g, vec![]);
    checkpoint(&g);
}
#[test]
fn wm023_invalid_frame_costs_targets_actor_program_cursor_and_response_bypass_are_rejected() {
    let (mut g, id) = setup(0);
    held(&mut g, 0, 0, "JC094");
    apply(&mut g, 0, action(&id, 4));
    for n in 0..9 {
        let mut bad = g.clone();
        let f = bad.stack.last_mut().unwrap().frame.as_mut().unwrap();
        match n {
            0 => f.already_paid.clear(),
            1 => f.already_paid.pop().map(|_| ()).unwrap(),
            2 => f.actor = 1,
            3 => f.cursor = 1,
            4 => f.source.card.exhausted = true,
            5 => f.targets[0].region_instance = None,
            6 => f.targets[0].spec.range = Range::SourceRegion,
            7 => f.steps.push(f.steps[0].clone()),
            _ => f.targets[0].public.region = Some(3),
        }
        invalid(&g, bad, &format!("frame-{n}"));
    }
    let mut bad = g.clone();
    let f = bad.stack.pop().unwrap().frame.unwrap();
    bad.effects.push_back(Effect::Frame { frame: Box::new(f) });
    invalid(&g, bad, "response-bypass");
    let mut bad = g.clone();
    let source = g
        .stack
        .last()
        .unwrap()
        .frame
        .as_ref()
        .unwrap()
        .source
        .clone();
    let mut a = definition("WM023").abilities[0].clone();
    a.key = "fake-trigger".into();
    a.event = Some(Event::Enter);
    a.targets.clear();
    a.costs.clear();
    a.ops = vec![Op::Draw {
        player: PlayerRef::Actor,
        count: 1,
        end: DeckEnd::Top,
    }];
    bad.effects.push_back(Effect::Declare {
        declaration: Declaration {
            actor: 0,
            source,
            ability: a,
        },
    });
    invalid(&g, bad, "fake-trigger");
}
#[test]
fn wm023_pending_private_options_projection_limits_region_and_foreign_frame_are_closed() {
    let (mut g, id) = setup(0);
    held(&mut g, 0, 0, "JC094");
    apply(&mut g, 0, action(&id, 4));
    pass_top(&mut g);
    for n in 0..10 {
        let mut bad = g.clone();
        let p = bad.pending.as_mut().unwrap();
        match n {
            0 => p.seat = 1,
            1 => p.choice.player_id = "p1".into(),
            2 => p.choice.allow_decline = Some(true),
            3 => p.choice.min = Some(0),
            4 => p.choice.amount = Some(2),
            5 => p.choice.title = "fake".into(),
            6 => p.choice.options[0].card.as_mut().unwrap().name = "fake".into(),
            7 => bad.regions[4].card.id = "replaced".into(),
            8 => {
                let ChoiceResolution::Frame { frame, .. } = &mut p.resolution else {
                    panic!()
                };
                frame.cursor = 0;
            }
            _ => {
                let ChoiceResolution::Frame { frame, .. } = &mut p.resolution else {
                    panic!()
                };
                frame.source.card.definition = "JC125".into();
                frame.ability_key = "foreign".into();
                frame.actor = 999;
                frame.steps.clear();
            }
        }
        invalid(&g, bad, &format!("pending-{n}"));
    }
}

#[test]
fn wm023_deploy_shell_identity_payment_and_physical_card_cannot_be_forged() {
    let mut g = game(0);
    fund(&mut g, 0, "JC029", 4);
    let id = held(&mut g, 0, 0, "WM023");
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    for n in 0..6 {
        let mut bad = g.clone();
        let s = bad.stack.last_mut().unwrap();
        match n {
            0 => s.id = "fake-stack".into(),
            1 => s.card.as_mut().unwrap().definition = "JC125".into(),
            2 => s.card.as_mut().unwrap().controller = 1,
            3 => s.deploy_region = Some(3),
            4 => s.frame.as_mut().unwrap().source.card.definition = "JC125".into(),
            _ => s.frame.as_mut().unwrap().already_paid.clear(),
        }
        invalid(&g, bad, &format!("deploy-shell-{n}"));
    }
}

#[test]
fn wm023_canonical_acquired_renown_is_optional_even_on_slow_exhausted_source() {
    let (mut g, id) = setup(0);
    g.board_mut(&id).unwrap().exhausted = true;
    for accept in [false, true] {
        let source = g.source_snapshot(g.board(&id).unwrap().1, Some(2));
        let ability = crate::renown::renown_ability(&g.regions[2].card.id);
        g.effects.push_back(Effect::Declare {
            declaration: Declaration {
                actor: 0,
                source,
                ability,
            },
        });
        g.drive().unwrap();
        checkpoint(&g);
        choose(
            &mut g,
            if accept {
                vec!["accept".into()]
            } else {
                vec![]
            },
        );
        if accept {
            pass_top(&mut g);
        }
        assert_eq!(g.regions[2].influence[0], if accept { 1 } else { 0 });
        checkpoint(&g);
    }
}
