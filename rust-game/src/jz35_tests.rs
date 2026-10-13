//! Offline prepared layouts, actual paid commands and explicitly labelled primitive probes.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, model::*, rules::*};
const KEY: &str = "jz35-sacrifice-return-controlled-hosts";
fn held(g: &mut Game, actor: usize) -> String {
    let c = g.make_card("JZ35", actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    id
}
fn play(id: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        ..Action::new("play")
    }
}
fn snapshot(name: &str, g: &Game, action: &Action) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("JZ35_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{name}.json"), serde_json::to_vec(&serde_json::json!({
            "scope":"offline prepared Native JZ35 layout; actual paid Room action, not browser play",
            "state":serde_json::to_string(&r).unwrap(),"action":action,
            "views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    }
}
fn cast(g: &mut Game, actor: usize, name: Option<&str>) -> String {
    fund(g, actor, "JC042", 2);
    let id = held(g, actor);
    let a = play(&id);
    if let Some(name) = name {
        snapshot(name, g, &a);
    }
    apply(g, actor, a);
    pass_top(g);
    id
}
fn skip_death_triggers(g: &mut Game) {
    for _ in 0..128 {
        if let Some(p) = &g.pending {
            assert!(matches!(p.resolution, ChoiceResolution::Declare { .. }));
            assert_eq!(p.choice.min, Some(0));
            choose(g, vec![]);
        } else if !g.stack.is_empty() {
            pass_top(g);
        } else {
            return;
        }
    }
    panic!("optional death declarations did not finish");
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
    if let Ok(dir) = std::env::var("JZ35_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/{name}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn jz35_full_original_and_closed_standard_spell() {
    let c = catalog::card("JZ35");
    assert_eq!(
        (&*c.name, &*c.kind, &*c.color, c.cost),
        ("思维湮灭", "spell", "红", 2)
    );
    assert_eq!(c.subtypes, ["法术", "心灵"]);
    assert_eq!(c.loyalty, ["红色"]);
    assert_eq!(c.magic_icon, MagicIcon::Mind);
    assert!(!c.unique && c.defense.is_none() && c.keywords.is_empty());
    assert_eq!(c.permanent_icons, Icons::default());
    assert_eq!(c.temporary_icons, Icons::default());
    let d = crate::jz35::definition();
    let a = &d.abilities[0];
    assert_eq!(d.abilities.len(), 1);
    assert_eq!(a.key, KEY);
    assert_eq!(a.timing, Timing::Standard);
    assert!(a.play_only && a.targets.is_empty() && a.costs.is_empty() && a.event.is_none());
    assert!(matches!(
        a.ops.as_slice(),
        [Op::JZ35SacrificeReturnControlledHosts]
    ));
    for n in 0..5 {
        let mut bad = d.clone();
        match n {
            0 => bad.abilities[0].timing = Timing::Fast,
            1 => bad.abilities[0]
                .ops
                .push(Op::JZ35SacrificeReturnControlledHosts),
            2 => bad.abilities[0].play_only = false,
            3 => bad.abilities[0]
                .costs
                .push(Cost::SacrificeSelectedControlledCharacter),
            _ => bad.abilities[0].event = Some(Event::Enter),
        }
        let mut defs = definitions().clone();
        defs.insert("JZ35".into(), bad);
        assert!(crate::rules::validate_definitions(&defs).is_err());
    }
    let mut defs = definitions().clone();
    defs.insert("JC125".into(), d);
    assert!(crate::rules::validate_definitions(&defs).is_err());
    crate::rules::validate_definitions(definitions()).unwrap();
}
#[test]
fn jz35_paid_same_name_hosts_all_seats_and_owners_return_as_new_owner_controlled_instances() {
    for actor in 0..4 {
        for owner in 0..4 {
            let mut g = game(actor);
            let one = board(&mut g, "JZ59", owner, 1);
            let two = board(&mut g, "JZ59", owner, 3);
            // Explicit valid temporary-control fixture; no fabricated public action.
            for id in [&one, &two] {
                g.add_control(
                    id,
                    actor,
                    ControlLifetime::TurnEnd { turn: g.turn },
                    SubtypeChange::None,
                );
                let c = g.board_mut(id).unwrap();
                c.time_markers = 3;
                c.lock_markers = 2;
                c.exhausted = true;
                c.shield = 1;
            }
            let old_grave = g.make_card("JZ59", owner);
            let old_grave_id = old_grave.id.clone();
            g.players[owner].graveyard.push(old_grave);
            cast(
                &mut g,
                actor,
                (actor == 0 && owner == 0).then_some("same-name-hosts-play"),
            );
            assert_eq!(g.resources(actor), 0);
            assert!(g.board(&one).is_none() && g.board(&two).is_none());
            for region in [1, 3] {
                let cs = g.regions[region]
                    .cards
                    .iter()
                    .filter(|c| c.definition == "JZ59")
                    .collect::<Vec<_>>();
                assert_eq!(cs.len(), 1);
                let c = cs[0];
                assert_eq!((c.owner, c.controller), (owner, owner));
                assert!(!c.face_down && !c.exhausted);
                assert_eq!(
                    (c.time_markers, c.lock_markers, c.damage, c.wounds, c.shield),
                    (0, 0, 0, 0, 0)
                );
            }
            assert!(g.players[owner]
                .graveyard
                .iter()
                .any(|c| c.id == old_grave_id));
            assert!(g
                .control_effects
                .iter()
                .all(|e| e.target_instance != one && e.target_instance != two));
            skip_death_triggers(&mut g);
            checkpoint(&g);
        }
    }
}
#[test]
fn jz35_current_controller_only_excludes_teammates_enemies_hidden_nonhosts_and_assets() {
    let mut g = game(0);
    let selected = board(&mut g, "JZ59", 2, 2);
    g.add_control(
        &selected,
        0,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    let teammate = board(&mut g, "JZ59", 1, 2);
    let enemy = board(&mut g, "JZ59", 2, 2);
    let hidden = board(&mut g, "JZ59", 0, 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    let ordinary = board(&mut g, "JC125", 0, 2);
    let asset = g.make_card("JZ59", 0);
    let asset_id = asset.id.clone();
    g.players[0].assets.push(asset);
    cast(&mut g, 0, Some("enemy-owned-host-play"));
    assert!(g.board(&selected).is_none());
    for id in [&teammate, &enemy, &hidden, &ordinary] {
        assert!(g.board(id).is_some());
    }
    // The host asset may pay the printed cost; it is never resurrected as a host.
    assert!(g
        .regions
        .iter()
        .all(|r| r.cards.iter().all(|c| c.id != asset_id)));
    let returned = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JZ59" && c.owner == 2 && c.id != enemy)
        .unwrap();
    assert_eq!(returned.controller, 2);
    skip_death_triggers(&mut g);
    checkpoint(&g);
}
#[test]
fn jz35_no_hosts_is_a_legal_paid_noop_and_spell_is_buried() {
    let mut g = game(0);
    let unrelated = board(&mut g, "JC125", 0, 2);
    cast(&mut g, 0, Some("no-hosts-play"));
    assert!(g.board(&unrelated).is_some());
    assert_eq!(g.resources(0), 0);
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JZ35")
            .count(),
        1
    );
    checkpoint(&g);
}
#[test]
fn jz35_resolves_current_hosts_after_responses_instead_of_freezing_declaration_targets() {
    let mut g = game(0);
    let old = board(&mut g, "JZ59", 0, 2);
    fund(&mut g, 0, "JC042", 2);
    let spell = held(&mut g, 0);
    snapshot("response-interval-play", &g, &play(&spell));
    apply(&mut g, 0, play(&spell));
    // Explicit control change/new entry during the response interval contract.
    g.add_control(
        &old,
        2,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    let new = board(&mut g, "JZ59", 0, 4);
    checkpoint(&g);
    pass_top(&mut g);
    assert!(g.board(&old).is_some());
    assert!(g.board(&new).is_none());
    assert_eq!(
        g.regions[4]
            .cards
            .iter()
            .filter(|c| c.definition == "JZ59" && c.owner == 0)
            .count(),
        1
    );
    skip_death_triggers(&mut g);
    checkpoint(&g);
}
#[test]
fn jz35_actual_departure_receipts_distinguish_two_same_name_same_region_cards_and_reset_entry() {
    let mut g = game(0);
    let one = board(&mut g, "JZ59", 0, 2);
    let two = board(&mut g, "JZ59", 0, 2);
    // Primitive single-effect probe: no command is allowed to settle this prepared lethal damage first.
    let c = g.board_mut(&one).unwrap();
    c.damage = 50;
    c.wounds = 50;
    let returns = g.jz35_sacrifice_hosts(0);
    assert_eq!(returns.len(), 2);
    let ids = returns
        .iter()
        .map(|r| r.death.grave_instance.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 2);
    for ret in &returns {
        assert_ne!(ret.death.old_instance, ret.death.grave_instance);
        assert!(ret.death.old_instance == one || ret.death.old_instance == two);
        assert!(g.players[0]
            .graveyard
            .iter()
            .any(|c| c.id == ret.death.grave_instance));
    }
    g.jz35_return_hosts(&returns).unwrap();
    assert_eq!(g.regions[2].cards.len(), 2);
    assert!(g.regions[2].cards.iter().all(|c| c.id != one
        && c.id != two
        && !ids.contains(&c.id)
        && c.damage == 0
        && c.wounds == 0));
    let before = serde_json::to_value(&g).unwrap();
    g.jz35_return_hosts(&returns).unwrap();
    assert_eq!(before, serde_json::to_value(&g).unwrap());
    g.drive().unwrap();
    skip_death_triggers(&mut g);
    checkpoint(&g);
}
#[test]
fn jz35_missing_actual_grave_card_never_substitutes_old_same_name_or_other_owners_copy() {
    let mut g = game(0);
    board(&mut g, "JZ59", 0, 2);
    let returns = g.jz35_sacrifice_hosts(0);
    let ret = &returns[0];
    let index = g.players[0]
        .graveyard
        .iter()
        .position(|c| c.id == ret.death.grave_instance)
        .unwrap();
    let actual = g.players[0].graveyard.remove(index);
    let actual = g.reset_zone_card(actual);
    g.players[0].hand.push(actual);
    // Explicit post-removal relocation probe, not an admitted death replacement card.
    for owner in [0, 2] {
        let c = g.make_card("JZ59", owner);
        g.players[owner].graveyard.push(c);
    }
    let before = serde_json::to_value(&g).unwrap();
    g.jz35_return_hosts(&returns).unwrap();
    assert_eq!(before, serde_json::to_value(&g).unwrap());
    assert!(g.regions[2].cards.is_empty());
}
#[test]
fn jz35_departure_that_did_not_happen_has_no_receipt_or_death_declaration() {
    let mut g = game(0);
    let id = board(&mut g, "JZ59", 0, 2);
    let source = g.source_snapshot(g.board(&id).unwrap().1, Some(2));
    // Explicit pre-removal diversion probe: the frozen original is now in hand.
    let (_, c) = g.leave_board(&id).unwrap();
    let c = g.reset_zone_card(c);
    g.players[0].hand.push(c);
    let before = serde_json::to_value(&g).unwrap();
    let receipts = g.remove_death_batch_with_receipts(vec![(id, RemovalCause::Sacrifice, source)]);
    assert!(receipts.is_empty());
    assert_eq!(before, serde_json::to_value(&g).unwrap());
}
#[test]
fn jz35_original_region_vacant_missing_or_replaced_leaves_actual_card_in_grave() {
    for n in 0..3 {
        let mut g = game(0);
        board(&mut g, "JZ59", 0, 4);
        let returns = g.jz35_sacrifice_hosts(0);
        // Explicit region identity loss between the two helper halves; no public mutation API.
        match n {
            0 => g.regions[4].vacant = true,
            1 => {
                g.regions.truncate(4);
            }
            _ => g.regions[4].card.id = "different-physical-region".into(),
        }
        g.jz35_return_hosts(&returns).unwrap();
        assert!(g.players[0]
            .graveyard
            .iter()
            .any(|c| c.id == returns[0].death.grave_instance));
        assert!(g.regions.iter().all(|r| r.cards.is_empty()));
    }
}
#[test]
fn jz35_host_attachments_leave_and_cannot_return_or_grant_subtype_to_new_instance() {
    let mut g = game(0);
    let host = board(&mut g, "JZ59", 0, 2);
    let ink = g.make_card("WM038", 2);
    g.attachments.push(Attachment {
        card: ink,
        host_id: host.clone(),
    });
    checkpoint(&g);
    assert!(g
        .current_subtypes(g.board(&host).unwrap().1)
        .contains(&"邪教徒".into()));
    cast(&mut g, 0, None);
    assert!(g.attachments.is_empty());
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "WM038"));
    let returned = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JZ59")
        .unwrap();
    assert!(!g.current_subtypes(returned).contains(&"邪教徒".into()));
    skip_death_triggers(&mut g);
    checkpoint(&g);
}
#[test]
fn jz35_death_triggers_keep_old_controller_source_and_original_region_after_return() {
    let mut g = game(0);
    let host = board(&mut g, "JZ31", 2, 2);
    g.add_control(
        &host,
        0,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    cast(&mut g, 0, None);
    let p = g.pending.as_ref().unwrap();
    assert_eq!(p.seat, 0);
    let ChoiceResolution::Declare { declaration: d, .. } = &p.resolution else {
        panic!()
    };
    assert_eq!(d.source.card.id, host);
    assert_eq!(d.source.card.controller, 0);
    let returned = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JZ31")
        .unwrap();
    assert_ne!(returned.id, host);
    assert_eq!(returned.controller, 2);
    let before = g.regions[2].influence[g.team(0)];
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence[g.team(0)], before + 1);
    checkpoint(&g);
}
#[test]
fn jz35_ordinary_grave_entry_enqueues_enter_but_not_reveal_triggers_for_the_owner() {
    let mut g = game(0);
    let c = g.make_card("JZ30", 2);
    let id = c.id.clone();
    g.players[2].graveyard.push(c);
    // Shared entry contract probe using an admitted Enter + Reveal card; JZ30 is not a Host.
    g.world_graveyard_entry(2, 2, &id).unwrap();
    let declarations = g
        .effects
        .iter()
        .filter_map(|e| {
            if let Effect::Declare { declaration } = e {
                Some(declaration)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert!(!declarations.is_empty());
    assert!(declarations
        .iter()
        .all(|d| d.actor == 2 && d.ability.event == Some(Event::Enter)));
    g.drive().unwrap();
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    let c = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JZ30")
        .unwrap();
    assert_eq!(c.time_markers, 1);
    checkpoint(&g);
}
#[test]
fn jz35_simultaneous_observers_keep_each_old_dead_identity_after_all_hosts_return() {
    let mut g = game(0);
    let observer = board(&mut g, "JC045", 0, 2);
    let one = board(&mut g, "JZ31", 0, 1);
    let two = board(&mut g, "JZ59", 0, 3);
    // Explicit atomic-helper probe; no declaration is driven between its halves.
    let returns = g.jz35_sacrifice_hosts(0);
    g.jz35_return_hosts(&returns).unwrap();
    let seen = g
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::Declare { declaration: d } if d.source.card.id == observer => {
                Some(d.source.observed_death.as_ref().unwrap().card.id.clone())
            }
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(seen, [one, two].into_iter().collect());
    g.drive().unwrap();
    skip_death_triggers(&mut g);
    checkpoint(&g);
}
#[test]
fn jz35_insufficient_cost_or_loyalty_and_extra_payload_are_atomic_rejections() {
    for n in 0..10 {
        let mut g = game(0);
        fund(
            &mut g,
            0,
            if n == 0 { "JC029" } else { "JC042" },
            if n == 1 { 1 } else { 2 },
        );
        let id = held(&mut g, 0);
        let mut a = play(&id);
        match n {
            0 | 1 => {}
            2 => a.region = Some(2),
            3 => a.target_id = Some("region:2".into()),
            4 => a.cost_selected = Some(vec![]),
            5 => a.selected = Some(vec![]),
            6 => a.option = Some("extra".into()),
            7 => a.choice_id = Some("extra".into()),
            8 => a.ability_id = Some("extra".into()),
            _ => a.top = Some(vec![]),
        }
        reject(&mut g, 0, a);
    }
}
#[test]
fn jz35_cannot_be_played_as_a_response_and_has_no_unprinted_once_per_turn_limit() {
    let mut g = game(0);
    board(&mut g, "JZ59", 0, 2);
    fund(&mut g, 0, "JC042", 4);
    let one = held(&mut g, 0);
    let two = held(&mut g, 0);
    apply(&mut g, 0, play(&one));
    reject(&mut g, 0, play(&two));
    pass_top(&mut g);
    skip_death_triggers(&mut g);
    let first_entry = g.regions[2].cards[0].id.clone();
    apply(&mut g, 0, play(&two));
    pass_top(&mut g);
    skip_death_triggers(&mut g);
    assert_ne!(g.regions[2].cards[0].id, first_entry);
    assert_eq!(g.resources(0), 0);
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JZ35")
            .count(),
        2
    );
    checkpoint(&g);
}
#[test]
fn jz35_duplicate_room_command_and_consumed_hand_action_do_not_pay_or_remove_twice() {
    use crate::room::{RoomCommand, RoomEnvelope, SessionAction};
    let mut g = game(0);
    board(&mut g, "JZ59", 0, 2);
    fund(&mut g, 0, "JC042", 2);
    let id = held(&mut g, 0);
    let room = envelope(&g);
    let command = RoomCommand {
        command_id: "jz35-once".into(),
        expected_version: room.revision,
        action: SessionAction::Game { action: play(&id) },
    };
    let first = room.transition(0, Some(command.clone()), 0).unwrap();
    assert!(first.error_code.is_none());
    let room = RoomEnvelope::from_persisted(&first.state).unwrap();
    let repeat = room.transition(0, Some(command), 0).unwrap();
    let repeated = RoomEnvelope::from_persisted(&repeat.state).unwrap();
    assert_eq!(
        serde_json::to_value(&room.game).unwrap(),
        serde_json::to_value(&repeated.game).unwrap()
    );
    apply(&mut g, 0, play(&id));
    reject(&mut g, 0, play(&id));
    pass_top(&mut g);
    skip_death_triggers(&mut g);
    checkpoint(&g);
}
#[test]
fn jz35_modified_frames_and_stack_shells_reject_both_restore_loaders() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 2);
    let id = held(&mut g, 0);
    apply(&mut g, 0, play(&id));
    for n in 0..25 {
        let mut bad = g.clone();
        let s = bad.stack.last_mut().unwrap();
        let f = s.frame.as_mut().unwrap();
        match n {
            0 => f.ability_key = "other".into(),
            1 => f.actor = 2,
            2 => f.source.card.definition = "JC125".into(),
            3 => f.cursor = 1,
            4 => f.guard = GuardState::Accepted,
            5 => f.chosen_region = Some(2),
            6 => f.source.region = Some(2),
            7 => f.steps[0].context = 2,
            8 => f.steps.push(f.steps[0].clone()),
            9 => f.steps.clear(),
            10 => f.already_paid.clear(),
            11 => f.already_paid = vec![PaidCost::Assets(vec!["same".into(), "same".into()])],
            12 => s.card = None,
            13 => s.target = Some("region:2".into()),
            14 => s.reveal = true,
            15 => s.deploy_region = Some(2),
            16 => f.source.card.id.clear(),
            17 => f.source.card.controller = 2,
            18 => f.source.card.owner = 99,
            19 => f.source.source_region_instance = Some("extra".into()),
            20 => f.source.attachment_host_instance = Some("extra".into()),
            21 => {
                f.steps[0].op =
                    Op::ForEachLivingPlayer(vec![Op::JZ35SacrificeReturnControlledHosts])
            }
            22 => f.already_paid = vec![PaidCost::Assets(vec!["a".into(), "b".into(), "c".into()])],
            23 => s.card.as_mut().unwrap().definition = "JC125".into(),
            _ => {
                f.source.observed_death = Some(ObservedCharacterDeath {
                    card: f.source.card.clone(),
                    vampire: false,
                })
            }
        }
        invalid(&g, bad, &format!("frame-{n}"));
    }
    let mut bad = g.clone();
    let s = bad.stack.pop().unwrap();
    bad.effects.push_back(Effect::Frame {
        frame: Box::new(s.frame.unwrap()),
    });
    invalid(&g, bad, "direct-or-replayed-effect");
    let mut bad = g.clone();
    bad.stack[0].frame = None;
    invalid(&g, bad, "no-program");
    let mut bad = g.clone();
    let f = bad.stack.pop().unwrap().frame.unwrap();
    bad.choice(
        0,
        "region",
        "forged JZ35 continuation".into(),
        vec![],
        0,
        0,
        None,
        ChoiceResolution::Frame {
            frame: Box::new(f),
            choice: FrameChoice::Region,
        },
    );
    invalid(&g, bad, "forged-pending-continuation");
    checkpoint(&g);
}
