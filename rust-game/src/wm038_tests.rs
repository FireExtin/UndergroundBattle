//! Explicit offline fixtures; real paid/optional/response commands and restore projections.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, model::*, rules::*};
const PLAY: &str = "wm038-attach-human";
const FAST: &str = "wm038-sacrifice-cultist-blood-discount";
fn held(g: &mut Game, actor: usize, def: &str) -> String {
    let c = g.make_card(def, actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    id
}
fn play(id: &str, host: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        target_id: Some(host.into()),
        ability_id: Some(PLAY.into()),
        ..Action::new("play")
    }
}
fn fast(source: &str, cost: &str) -> Action {
    Action {
        card_id: Some(source.into()),
        ability_id: Some(FAST.into()),
        cost_selected: Some(vec![cost.into()]),
        ..Action::new("activate")
    }
}
fn snap(name: &str, g: &Game, a: &Action) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("WM038_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{name}.json"),serde_json::to_vec(&serde_json::json!({"scope":"offline-wm038-native: explicit layout, actual accepted Room command","state":serde_json::to_string(&r).unwrap(),"action":a,"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()})).unwrap()).unwrap();
    }
}
fn invalid(g: &Game, bad: Game, name: &str) {
    assert!(
        Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),
        "Game accepted {name}"
    );
    let mut r = envelope(g);
    r.game = bad;
    let state = serde_json::to_string(&r).unwrap();
    assert!(
        crate::room::RoomEnvelope::from_persisted(&state).is_err(),
        "Room accepted {name}"
    );
    if let Ok(dir) = std::env::var("WM038_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/{name}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}
fn setup(actor: usize, owner: usize, accept: bool) -> (Game, String, String) {
    let mut g = game(actor);
    let host = board(&mut g, "JC125", owner, 2);
    fund(&mut g, actor, "JC042", 1);
    let source = held(&mut g, actor, "WM038");
    apply(&mut g, actor, play(&source, &host));
    pass_top(&mut g);
    assert!(g
        .current_subtypes(g.board(&host).unwrap().1)
        .contains(&"邪教徒".into()));
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
    let source = g
        .attachments
        .iter()
        .find(|a| a.host_id == host)
        .unwrap()
        .card
        .id
        .clone();
    (g, source, host)
}
fn discount(g: &mut Game, actor: usize, source: &str, cost: &str) {
    apply(g, actor, fast(source, cost));
    pass_top(g);
    assert!(g
        .modifiers
        .iter()
        .any(|m| matches!(m.filter, CardFilter::WM038BloodCard)
            && m.actor == actor
            && m.uses == 1));
}
#[test]
fn wm038_original_fields_complete_and_programs_are_closed() {
    let c = catalog::card("WM038");
    assert_eq!(
        (&*c.name, &*c.kind, &*c.color, c.cost),
        ("信徒烙印", "attachment", "红", 1)
    );
    assert_eq!(c.loyalty, ["红色"]);
    assert_eq!(c.magic_icon, MagicIcon::Blood);
    assert_eq!(c.subtypes, ["纹身"]);
    assert!(!c.unique && c.defense.is_none() && c.keywords.is_empty());
    assert_eq!(c.permanent_icons, Icons::default());
    assert_eq!(c.temporary_icons, Icons::default());
    let d = definition("WM038");
    assert_eq!(d.abilities.len(), 3);
    assert_eq!(d.abilities[0].targets[0].subtype.as_deref(), Some("人类"));
    assert_eq!(d.abilities[1].event, Some(Event::Enter));
    assert_eq!(d.abilities[2].timing, Timing::Fast);
    for n in 0..12 {
        let mut b = d.clone();
        match n {
            0 => b.abilities[0].targets[0].relation = Relation::FriendlyTeam,
            1 => b.abilities[0].targets[0].subtype = None,
            2 => b.abilities[1].event = Some(Event::Reveal),
            3 => b.abilities[1].ops.clear(),
            4 => b.abilities[2].costs.clear(),
            5 => b.abilities[2].costs = vec![Cost::ExhaustSource],
            6 => b.abilities[2].per_turn_limit = Some(1),
            7 => b.abilities[2].response_policy = ResponsePolicy::Immediate,
            8 => {
                b.abilities[2].ops = vec![Op::CostReduction {
                    filter: CardFilter::HasPrintedMagic,
                    amount: 2,
                }]
            }
            9 => b.modifiers.clear(),
            10 => b.attachment.as_mut().unwrap().controls_host = true,
            _ => b.attachment.as_mut().unwrap().host_leaves = HostLeaveDestination::OwnerHand,
        }
        let mut ds = definitions().clone();
        ds.insert("WM038".into(), b);
        assert!(crate::rules::validate_definitions(&ds).is_err());
    }
    let mut ds = definitions().clone();
    ds.insert("JC125".into(), d.clone());
    assert!(crate::rules::validate_definitions(&ds).is_err());
    crate::rules::validate_definitions(definitions()).unwrap();
}
#[test]
fn wm038_paid_human_attachment_all_seats_all_relations_and_optional_original_host_time() {
    for actor in 0..4 {
        for owner in 0..4 {
            for accept in [false, true] {
                let (g, source, host) = setup(actor, owner, accept);
                assert_eq!(g.resources(actor), 0);
                assert_eq!(g.board(&host).unwrap().1.time_markers, u32::from(accept));
                assert_eq!(g.board(&host).unwrap().1.controller, owner);
                assert!(g
                    .current_subtypes(g.board(&host).unwrap().1)
                    .contains(&"邪教徒".into()));
                assert!(!g.board(&source).unwrap().1.exhausted);
                checkpoint(&g);
            }
        }
    }
}
#[test]
fn wm038_cost_loyalty_and_nonhuman_hidden_asset_targets_reject_atomically() {
    for n in 0..5 {
        let mut g = game(0);
        let h = board(&mut g, if n == 2 { "JC029" } else { "JC125" }, 0, 2);
        if n == 3 {
            g.board_mut(&h).unwrap().face_down = true;
        }
        fund(
            &mut g,
            0,
            if n == 0 { "JC029" } else { "JC042" },
            if n == 1 { 0 } else { 1 },
        );
        let id = held(&mut g, 0, "WM038");
        let mut a = play(&id, &h);
        if n == 4 {
            a.target_id = Some(g.players[0].assets[0].id.clone());
        }
        reject(&mut g, 0, a);
    }
}
#[test]
fn wm038_frontend_four_snapshots_are_actual_play_accept_fast_and_discounted_blood_deploy() {
    let mut g = game(0);
    let host = board(&mut g, "JC125", 0, 2);
    fund(&mut g, 0, "JC042", 1);
    let source = held(&mut g, 0, "WM038");
    let a = play(&source, &host);
    snap("attach", &g, &a);
    apply(&mut g, 0, a);
    pass_top(&mut g);
    let p = g.pending.as_ref().unwrap();
    let a = Action {
        choice_id: Some(p.choice.id.clone()),
        selected: Some(vec!["accept".into()]),
        ..Action::new("choose")
    };
    snap("host-time-accept", &g, &a);
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    let source = g.attachments[0].card.id.clone();
    let cost = board(&mut g, "JC042", 0, 1);
    let a = fast(&source, &cost);
    snap("blood-discount", &g, &a);
    discount(&mut g, 0, &source, &cost);
    fund(&mut g, 0, "JC029", 2);
    let blood = held(&mut g, 0, "JC029");
    let a = Action {
        card_id: Some(blood),
        region: Some(2),
        ..Action::new("deploy")
    };
    snap("discounted-blood-deploy", &g, &a);
    apply(&mut g, 0, a);
    pass_top(&mut g);
    assert_eq!(g.resources(0), 2);
    assert!(g.modifiers.iter().all(|m| m.uses == 0));
}
#[test]
fn wm038_actual_activation_all_seats_sacrifices_controlled_current_cultist_anywhere() {
    for actor in 0..4 {
        let (mut g, source, _) = setup(actor, (actor + 2) % 4, false);
        let cost = board(&mut g, "JC042", actor, 4);
        discount(&mut g, actor, &source, &cost);
        assert!(g.board(&cost).is_none());
        assert!(g.board(&source).unwrap().1.exhausted);
        assert!(g.players[actor]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC042"));
        checkpoint(&g);
    }
}
#[test]
fn wm038_granted_cultist_host_is_valid_sacrifice_source_leaves_and_paid_frame_survives() {
    let (mut g, source, host) = setup(0, 0, true);
    discount(&mut g, 0, &source, &host);
    assert!(g.board(&host).is_none() && g.board(&source).is_none());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "WM038"));
    let blood = held(&mut g, 0, "JC029");
    fund(&mut g, 0, "JC029", 2);
    assert_eq!(
        g.effective_cost(
            0,
            &g.players[0].hand.iter().find(|c| c.id == blood).unwrap()
        ),
        0
    );
    checkpoint(&g);
}
#[test]
fn wm038_non_cultist_teammate_enemy_hidden_wrong_count_wrong_actor_extra_payload_and_exhaustion_reject(
) {
    let (mut g, source, _) = setup(0, 0, false);
    let wrong = board(&mut g, "JC125", 0, 1);
    let ally = board(&mut g, "JC042", 1, 2);
    let enemy = board(&mut g, "JC042", 2, 2);
    let hidden = board(&mut g, "JC042", 0, 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    for id in [wrong, ally, enemy, hidden] {
        reject(&mut g, 0, fast(&source, &id));
    }
    let good = board(&mut g, "JC042", 0, 1);
    reject(&mut g, 1, fast(&source, &good));
    for n in 0..3 {
        let mut a = fast(&source, &good);
        match n {
            0 => a.cost_selected = None,
            1 => a.cost_selected = Some(vec![good.clone(), good.clone()]),
            _ => a.target_id = Some(good.clone()),
        }
        reject(&mut g, 0, a);
    }
    g.board_mut(&source).unwrap().exhausted = true;
    reject(&mut g, 0, fast(&source, &good));
}
#[test]
fn wm038_legal_actions_offer_only_current_controlled_cultists_and_no_unprinted_turn_limit() {
    let (mut g, source, host) = setup(0, 0, false);
    let good = board(&mut g, "JC042", 0, 4);
    let wrong = board(&mut g, "JC125", 0, 1);
    let actions = g
        .legal_actions(0)
        .into_iter()
        .filter(|a| a.action.card_id.as_deref() == Some(&source) && a.action.kind == "activate")
        .collect::<Vec<_>>();
    assert_eq!(actions.len(), 2);
    assert!(actions
        .iter()
        .all(
            |a| a.action.cost_selected.as_ref().unwrap() == &vec![good.clone()]
                || a.action.cost_selected.as_ref().unwrap() == &vec![host.clone()]
        ));
    assert!(!actions
        .iter()
        .any(|a| a.action.cost_selected.as_ref().unwrap().contains(&wrong)));
    discount(&mut g, 0, &source, &good);
    let another = board(&mut g, "JC042", 0, 1);
    reject(&mut g, 0, fast(&source, &another));
    g.effects.push_back(Effect::NextTurn);
    g.drive().unwrap();
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    discount(&mut g, 0, &source, &another);
}
#[test]
fn wm038_discount_matches_printed_blood_only_and_not_death_mind_or_color() {
    let (mut g, source, _) = setup(0, 0, false);
    let cost = board(&mut g, "JC042", 0, 2);
    discount(&mut g, 0, &source, &cost);
    for id in ["JC125", "WM023", "XQ18", "JC001"] {
        let c = g.make_card(id, 0);
        assert!(!CardFilter::WM038BloodCard.matches(catalog::card(id)));
        assert_eq!(g.effective_cost(0, &c), catalog::card(id).cost);
    }
    for id in ["JC029", "BQ040", "WM038", "JC045"] {
        let c = g.make_card(id, 0);
        assert!(CardFilter::WM038BloodCard.matches(catalog::card(id)));
        assert_eq!(
            g.effective_cost(0, &c),
            catalog::card(id).cost.saturating_sub(2)
        );
    }
}
#[test]
fn wm038_nonblood_play_does_not_consume_discount_next_blood_play_does_not_cross_players() {
    let (mut g, source, _) = setup(0, 0, false);
    let cost = board(&mut g, "JC042", 0, 2);
    discount(&mut g, 0, &source, &cost);
    let plain = held(&mut g, 0, "JC125");
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(plain),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    assert_eq!(g.modifiers[0].uses, 1);
    let other = g.make_card("JC029", 1);
    assert_eq!(g.effective_cost(1, &other), 2);
    let blood = held(&mut g, 0, "JC029");
    fund(&mut g, 0, "JC029", 2);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(blood),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    assert!(g.modifiers.iter().all(|m| m.uses == 0));
    let next = g.make_card("JC029", 0);
    assert_eq!(g.effective_cost(0, &next), 2);
}
#[test]
fn wm038_secret_deploy_and_paid_reveal_do_not_consume_play_only_discount() {
    let (mut g, source, _) = setup(0, 0, false);
    let cost = board(&mut g, "JC042", 0, 2);
    discount(&mut g, 0, &source, &cost);
    fund(&mut g, 0, "JC029", 3);
    let id = held(&mut g, 0, "JC029");
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
    assert_eq!(g.resources(0), 2);
    assert_eq!(g.modifiers[0].uses, 1);
    let id = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC029")
        .unwrap()
        .id
        .clone();
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut g);
    assert_eq!(g.resources(0), 0);
    assert_eq!(g.modifiers[0].uses, 1);
}
#[test]
fn wm038_multiple_discount_instances_stack_clamp_to_zero_and_all_consume_on_one_blood_play() {
    let (mut g, first, host) = setup(0, 0, false);
    fund(&mut g, 0, "JC042", 1);
    let id = held(&mut g, 0, "WM038");
    apply(&mut g, 0, play(&id, &host));
    pass_top(&mut g);
    choose(&mut g, vec![]);
    let second = g.attachments.last().unwrap().card.id.clone();
    let one = board(&mut g, "JC042", 0, 0);
    let two = board(&mut g, "JC042", 0, 4);
    discount(&mut g, 0, &first, &one);
    discount(&mut g, 0, &second, &two);
    let id = held(&mut g, 0, "JC029");
    fund(&mut g, 0, "JC029", 2);
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
    assert_eq!(g.resources(0), 2);
    assert!(g.modifiers.iter().all(|m| m.uses == 0));
}
#[test]
fn wm038_expiration_and_original_host_departure_do_not_transfer_time_or_subtype() {
    let (mut g, source, host) = setup(0, 0, true);
    let cost = board(&mut g, "JC042", 0, 2);
    discount(&mut g, 0, &source, &cost);
    g.return_hand(&host);
    assert!(g.board(&source).is_none());
    let new = g.players[0]
        .hand
        .iter()
        .find(|c| c.definition == "JC125")
        .unwrap();
    assert_ne!(new.id, host);
    assert_eq!(new.time_markers, 0);
    assert!(!g.current_subtypes(new).contains(&"邪教徒".into()));
    g.effects.push_back(Effect::Cleanup);
    g.drive().unwrap();
    assert!(g.modifiers.is_empty());
    checkpoint(&g);
}
#[test]
fn wm038_declared_entry_host_move_remains_original_instance_and_departure_is_noop() {
    for leaves in [false, true] {
        let (mut g, _, host) = setup(0, 0, false);
        fund(&mut g, 0, "JC042", 1);
        let id = held(&mut g, 0, "WM038");
        apply(&mut g, 0, play(&id, &host));
        pass_top(&mut g);
        choose(&mut g, vec!["accept".into()]);
        if leaves {
            g.return_hand(&host);
        } else {
            let (_, c) = g.remove_board(&host).unwrap();
            g.regions[4].cards.push(c);
        }
        checkpoint(&g);
        pass_top(&mut g);
        if leaves {
            assert!(g.players[0]
                .hand
                .iter()
                .filter(|c| c.definition == "JC125")
                .all(|c| c.time_markers == 0));
        } else {
            assert_eq!(g.board(&host).unwrap().0, 4);
            assert_eq!(g.board(&host).unwrap().1.time_markers, 1);
        }
        checkpoint(&g);
    }
}
#[test]
fn wm038_real_response_returns_target_before_attachment_no_entry_and_cost_not_refunded() {
    let mut g = game(0);
    let host = board(&mut g, "JC125", 2, 2);
    fund(&mut g, 0, "JC042", 1);
    let id = held(&mut g, 0, "WM038");
    apply(&mut g, 0, play(&id, &host));
    fund(&mut g, 2, "JC075", 2);
    let response = board(&mut g, "JC075", 2, 1);
    while !g
        .legal_actions(2)
        .iter()
        .any(|a| a.action.kind == "activate" && a.action.card_id.as_deref() == Some(&response))
    {
        let seat = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        apply(&mut g, seat, Action::new("pass"));
    }
    apply(
        &mut g,
        2,
        Action {
            card_id: Some(response),
            target_id: Some(host.clone()),
            ability_id: Some(definition("JC075").abilities[0].key.clone()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    pass_top(&mut g);
    assert!(g.attachments.is_empty() && g.pending.is_none());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "WM038"));
    assert_eq!(g.resources(0), 0);
}
#[test]
fn wm038_bq030_observer_and_own_entry_are_separate_optional_triggers() {
    let mut g = game(0);
    board(&mut g, "BQ030", 0, 2);
    let host = board(&mut g, "JC125", 0, 2);
    fund(&mut g, 0, "JC042", 1);
    let id = held(&mut g, 0, "WM038");
    apply(&mut g, 0, play(&id, &host));
    pass_top(&mut g);
    assert!(g.pending.is_some());
    for _ in 0..4 {
        if g.pending.is_none() {
            break;
        }
        choose(&mut g, vec![]);
    }
    assert!(g.pending.is_none());
    assert_eq!(g.board(&host).unwrap().1.time_markers, 0);
    checkpoint(&g);
}
#[test]
fn wm038_invalid_frames_targets_shells_payment_and_modifiers_reject_game_and_room() {
    let (mut g, source, _) = setup(0, 0, false);
    let cost = board(&mut g, "JC042", 0, 2);
    apply(&mut g, 0, fast(&source, &cost));
    for n in 0..9 {
        let mut b = g.clone();
        let s = b.stack.last_mut().unwrap();
        let f = s.frame.as_mut().unwrap();
        match n {
            0 => f.actor = 99,
            1 => f.source.attachment_host_instance = None,
            2 => f.source.card.exhausted = true,
            3 => f.cursor = 1,
            4 => f.already_paid.clear(),
            5 => {
                f.steps[0].op = Op::CostReduction {
                    filter: CardFilter::HasPrintedMagic,
                    amount: 2,
                }
            }
            6 => s.card = Some(f.source.card.clone()),
            7 => f.guard = GuardState::Accepted,
            _ => s.reveal = true,
        }
        invalid(&g, b, &format!("activation-{n}"));
    }
    pass_top(&mut g);
    for n in 0..5 {
        let mut b = g.clone();
        match n {
            0 => b.modifiers[0].amount = 3,
            1 => b.modifiers[0].paid_reveal = true,
            2 => b.modifiers[0].actor = 99,
            3 => b.modifiers[0].uses = 2,
            _ => b.modifiers[0].expires_turn = g.turn + 1,
        }
        invalid(&g, b, &format!("modifier-{n}"));
    }
}
#[test]
fn wm038_invalid_in_play_human_host_identity_visibility_and_declarations_reject() {
    let (g, _, _) = setup(0, 0, false);
    for n in 0..6 {
        let mut b = g.clone();
        match n {
            0 => b.attachments[0].host_id = "missing".into(),
            1 => b.attachments[0].card.face_down = true,
            2 => b.attachments[0].card.controller = 99,
            3 => b.attachments[0].card.owner = 99,
            4 => b.attachments[0].card.id.clear(),
            _ => {
                let a = b.attachments[0].clone();
                b.attachments.push(a)
            }
        }
        invalid(&g, b, &format!("in-play-{n}"));
    }
}
#[test]
fn wm038_continuous_subtype_is_not_exhaustion_or_controller_gated_and_invalid_human_host_cleans() {
    let (mut g, source, host) = setup(0, 2, true);
    g.board_mut(&source).unwrap().exhausted = true;
    assert!(g
        .current_subtypes(g.board(&host).unwrap().1)
        .contains(&"邪教徒".into()));
    // Prepared control/type conversion primitive; granted subtype does not recurse into its own host guard.
    g.add_control(
        &host,
        1,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::HumanToVampire,
    );
    g.settle_deaths();
    assert!(g.board(&source).is_none());
    let c = g.board(&host).unwrap().1;
    assert!(!g.current_subtypes(c).contains(&"邪教徒".into()));
    assert!(g.current_subtypes(c).contains(&"吸血鬼".into()));
    assert_eq!(c.time_markers, 1);
    checkpoint(&g);
}
#[test]
fn wm038_pending_entry_and_play_frame_tampering_reject_both_loaders() {
    let mut g = game(0);
    let host = board(&mut g, "JC125", 0, 2);
    fund(&mut g, 0, "JC042", 1);
    let id = held(&mut g, 0, "WM038");
    apply(&mut g, 0, play(&id, &host));
    for n in 0..7 {
        let mut b = g.clone();
        let s = b.stack.last_mut().unwrap();
        let f = s.frame.as_mut().unwrap();
        match n {
            0 => f.targets[0].spec.subtype = None,
            1 => f.source.region = Some(2),
            2 => f.already_paid.clear(),
            3 => f.targets[0].region_instance = Some("wrong".into()),
            4 => f.targets[0].public.kind = "attachment".into(),
            5 => s.card = None,
            _ => s.deploy_region = Some(2),
        }
        invalid(&g, b, &format!("play-{n}"));
    }
    pass_top(&mut g);
    for n in 0..4 {
        let mut b = g.clone();
        let p = b.pending.as_mut().unwrap();
        let ChoiceResolution::Declare {
            declaration: d,
            stage,
        } = &mut p.resolution
        else {
            panic!()
        };
        match n {
            0 => p.seat = 1,
            1 => d.source.attachment_host_instance = None,
            2 => d.ability.ops.clear(),
            _ => *stage = DeclareChoice::Target,
        }
        invalid(&g, b, &format!("entry-choice-{n}"));
    }
    choose(&mut g, vec!["accept".into()]);
    let mut b = g.clone();
    let frame = b.stack.pop().unwrap().frame.unwrap();
    b.effects.push_back(Effect::Frame {
        frame: Box::new(frame),
    });
    invalid(&g, b, "direct-entry-effect");
    pass_top(&mut g);
}
