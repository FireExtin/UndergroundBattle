//! Complete source, paid commands, frozen regions and explicitly labelled leader probes.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, model::*, rules::*};
const KEY: &str = "jc052-destroy-other-original-region-characters";
fn held(g: &mut Game, actor: usize) -> String {
    let c = g.make_card("JC052", actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    id
}
fn enter(g: &mut Game, actor: usize) -> String {
    fund(g, actor, "JC042", 7);
    let id = held(g, actor);
    apply(
        g,
        actor,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(g);
    g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC052")
        .unwrap()
        .id
        .clone()
}
fn finish_optional(g: &mut Game) {
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
    panic!("declarations did not finish");
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
    if let Ok(dir) = std::env::var("JC052_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/{name}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}
fn snapshot(name: &str, g: &Game, a: &Action) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("JC052_EVIDENCE_DIR") {
        let r = envelope(g);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/{name}.json"),serde_json::to_vec(&serde_json::json!({
            "scope":"explicit Native offline layout; actual paid or optional Room command, not browser play",
            "state":serde_json::to_string(&r).unwrap(),"action":a,"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    }
}
#[test]
fn jc052_full_print_and_definition_are_closed_without_mandatory_entry_or_extra_targets() {
    let c = catalog::card("JC052");
    assert_eq!(
        (&*c.name, c.subtitle.as_deref(), c.cost),
        ("终焉的诗寇蒂", Some("剪断命运者"), 7)
    );
    assert_eq!(c.loyalty, ["红色", "红色", "红色", "红色"]);
    assert_eq!(c.magic_icon, MagicIcon::Other("星辰".into()));
    assert_eq!(c.subtypes, ["人类", "圣者"]);
    assert_eq!(c.defense, Some(1));
    assert!(c.unique);
    assert_eq!(c.keywords, ["领袖"]);
    assert!(crate::jc052::printed_leader("JC052"));
    assert_eq!(
        c.permanent_icons,
        Icons {
            investigation: 1,
            influence: 2,
            combat: 0
        }
    );
    assert_eq!(
        c.temporary_icons,
        Icons {
            investigation: 1,
            influence: 0,
            combat: 0
        }
    );
    let d = crate::jc052::definition();
    assert_eq!(d.abilities.len(), 1);
    let a = &d.abilities[0];
    assert_eq!(a.event, Some(Event::Enter));
    assert_eq!(a.key, KEY);
    assert!(a.costs.is_empty() && a.targets.is_empty());
    assert!(matches!(
        a.ops.as_slice(),
        [Op::JC052DestroyOtherOriginalRegionCharacters]
    ));
    for n in 0..4 {
        let mut bad = d.clone();
        match n {
            0 => bad.abilities[0].event = Some(Event::Reveal),
            1 => bad.abilities[0].ops.clear(),
            2 => bad.abilities[0].costs.push(Cost::Assets(1)),
            _ => bad.abilities[0]
                .ops
                .push(Op::JC052DestroyOtherOriginalRegionCharacters),
        }
        let mut defs = definitions().clone();
        defs.insert("JC052".into(), bad);
        assert!(crate::rules::validate_definitions(&defs).is_err());
    }
    let mut defs = definitions().clone();
    defs.insert("JC125".into(), d);
    assert!(crate::rules::validate_definitions(&defs).is_err());
}
#[test]
fn jc052_actual_entry_all_seats_decline_or_accept_destroys_every_relation_but_original_self() {
    for actor in 0..4 {
        for accept in [false, true] {
            let mut g = game(actor);
            let mut targets = vec![];
            for owner in 0..4 {
                targets.push(board(&mut g, "JC125", owner, 2));
                let h = board(&mut g, "JC125", owner, 2);
                g.board_mut(&h).unwrap().face_down = true;
                targets.push(h);
            }
            let outside = board(&mut g, "JC125", (actor + 2) % 4, 1);
            let id = enter(&mut g, actor);
            assert_eq!(g.resources(actor), 0);
            assert_eq!(g.pending.as_ref().unwrap().seat, actor);
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
            assert!(g.board(&id).is_some() && g.board(&outside).is_some());
            for target in targets {
                assert_eq!(g.board(&target).is_none(), accept);
            }
            finish_optional(&mut g);
            checkpoint(&g);
        }
    }
}
#[test]
fn jc052_secret_deploy_is_not_entry_and_paid_reveal_is_one_optional_entry() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 1);
    let id = held(&mut g, 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
    assert!(g.pending.is_none());
    let hidden = g.regions[2].cards[0].id.clone();
    assert!(!g.leader_options(0).iter().any(|o| o.id == hidden));
    let other = board(&mut g, "JC125", 2, 2);
    fund(&mut g, 0, "JC042", 7);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut g);
    assert!(matches!(
        g.pending.as_ref().unwrap().resolution,
        ChoiceResolution::Declare { .. }
    ));
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    assert!(g.board(&other).is_none());
    assert!(g.pending.is_none() && g.stack.is_empty());
    checkpoint(&g);
}
#[test]
fn jc052_loyalty_cost_and_unprinted_payload_reject_atomically() {
    for n in 0..8 {
        let mut g = game(0);
        fund(
            &mut g,
            0,
            "JC042",
            if n == 0 {
                6
            } else if n == 1 {
                3
            } else {
                7
            },
        );
        if n == 1 {
            fund(&mut g, 0, "JC029", 4);
        }
        let id = held(&mut g, 0);
        let mut a = Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        };
        match n {
            0 | 1 => {}
            2 => a.target_id = Some("region:2".into()),
            3 => a.ability_id = Some(KEY.into()),
            4 => a.selected = Some(vec![]),
            5 => a.cost_selected = Some(vec![]),
            6 => a.option = Some("extra".into()),
            _ => a.choice_id = Some("extra".into()),
        }
        reject(&mut g, 0, a);
    }
}
#[test]
fn jc052_source_movement_or_departure_never_transfers_original_region_wave() {
    for leave in [false, true] {
        let mut g = game(0);
        let old = board(&mut g, "JC125", 2, 2);
        let safe = board(&mut g, "JC125", 2, 3);
        let source = enter(&mut g, 0);
        choose(&mut g, vec!["accept".into()]);
        if leave {
            g.return_hand(&source);
        } else {
            // Explicit movement primitive during the real response interval.
            let (_, c) = g.remove_board(&source).unwrap();
            g.regions[3].cards.push(c);
        }
        checkpoint(&g);
        pass_top(&mut g);
        assert!(g.board(&old).is_none() && g.board(&safe).is_some());
        if !leave {
            assert_eq!(g.board(&source).unwrap().0, 3);
        }
        finish_optional(&mut g);
        checkpoint(&g);
    }
}
#[test]
fn jc052_old_source_instance_is_not_a_same_name_returned_instance() {
    let mut g = game(0);
    let source = enter(&mut g, 0);
    choose(&mut g, vec!["accept".into()]);
    g.return_hand(&source);
    let c = g.players[0].hand.pop().unwrap();
    let new = g.reset_zone_card(c);
    let new_id = new.id.clone();
    g.regions[2].cards.push(new);
    // Explicit primitive re-entry, no fabricated second entry declaration.
    assert_ne!(new_id, source);
    checkpoint(&g);
    pass_top(&mut g);
    assert!(g.board(&new_id).is_none());
    finish_optional(&mut g);
    checkpoint(&g);
}
#[test]
fn jc052_original_region_replacement_or_vacancy_is_noop_without_redirect() {
    for vacant in [false, true] {
        let mut g = game(0);
        let source = enter(&mut g, 0);
        choose(&mut g, vec!["accept".into()]);
        g.return_hand(&source);
        // Explicit physical-region departure/replacement contract; original slot cleared.
        g.regions[2].cards.clear();
        let original = g.regions[2].card.clone();
        g.regions[2].card = g.regions[3].card.clone();
        g.regions[3].card = original;
        g.regions[2].vacant = vacant;
        if vacant {
            g.world.clear();
            g.regions[2].influence = [0, 0];
            g.regions[2].skip = false;
        }
        let sentinel = (!vacant).then(|| board(&mut g, "JC125", 2, 2));
        checkpoint(&g);
        pass_top(&mut g);
        if let Some(id) = sentinel {
            assert!(g.board(&id).is_some());
        }
        assert_eq!(g.resources(0), 0);
        finish_optional(&mut g);
        checkpoint(&g);
    }
}
#[test]
fn jc052_real_response_returns_source_and_entry_still_clears_original_region() {
    let mut g = game(0);
    let victim = board(&mut g, "JC125", 2, 2);
    let source = enter(&mut g, 0);
    choose(&mut g, vec!["accept".into()]);
    let responder = board(&mut g, "JC075", 0, 2);
    fund(&mut g, 0, "JC075", 2);
    while !g.legal_actions(0).iter().any(|l| {
        l.action.kind == "activate"
            && l.action.card_id.as_deref() == Some(&responder)
            && l.action.target_id.as_deref() == Some(&source)
    }) {
        let seat = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        apply(&mut g, seat, Action::new("pass"));
    }
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|l| {
            l.action.kind == "activate"
                && l.action.card_id.as_deref() == Some(&responder)
                && l.action.target_id.as_deref() == Some(&source)
        })
        .unwrap()
        .action;
    apply(&mut g, 0, a);
    pass_top(&mut g);
    assert!(g.board(&source).is_none());
    checkpoint(&g);
    pass_top(&mut g);
    assert!(g.board(&victim).is_none() && g.board(&responder).is_none());
    finish_optional(&mut g);
    checkpoint(&g);
}
#[test]
fn jc052_hidden_deaths_do_not_declare_and_character_observers_freeze_the_whole_wave() {
    let mut g = game(0);
    let observer = board(&mut g, "JC031", 0, 2);
    let character = board(&mut g, "JZ31", 0, 2);
    let hidden = board(&mut g, "JZ31", 0, 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    enter(&mut g, 0);
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    let mut deaths = vec![];
    if let Some(p) = &g.pending {
        if let ChoiceResolution::Declare { declaration: d, .. } = &p.resolution {
            deaths.push(d.clone());
        }
    }
    deaths.extend(g.effects.iter().filter_map(|e| {
        if let Effect::Declare { declaration } = e {
            Some(declaration.clone())
        } else {
            None
        }
    }));
    assert!(deaths.iter().any(|d| d.source.card.id == observer
        && d.source
            .observed_death
            .as_ref()
            .is_some_and(|x| x.card.id == character)));
    assert!(deaths.iter().all(|d| d.source.card.id != hidden
        && d.source
            .observed_death
            .as_ref()
            .is_none_or(|x| x.card.id != hidden)));
    finish_optional(&mut g);
    checkpoint(&g);
}
#[test]
fn jc052_leader_groups_use_controller_and_ignore_names_owners_teammates_and_hidden() {
    let mut g = game(0);
    let a = g.make_card("JC052", 2);
    let mut b = g.make_card("LC23", 3);
    let mut a = a;
    a.controller = 0;
    b.controller = 0;
    let mut team = g.make_card("JC052", 1);
    team.controller = 1;
    let mut hidden = g.make_card("JC052", 0);
    hidden.face_down = true;
    let ordinary = g.make_card("JC125", 0);
    // Printed-metadata grouping contract with a different-name hypothetical leader.
    // LC23's real definition is unchanged and it is not admitted as a Leader.
    let ids = crate::jc052::leader_instances(
        0,
        [
            (&a, true),
            (&b, true),
            (&team, true),
            (&hidden, true),
            (&ordinary, false),
        ],
    );
    assert_eq!(ids, vec![a.id, b.id]);
}
#[test]
fn jc052_duplicate_entry_mandatory_leader_choice_preserves_sacrificed_sources_trigger() {
    let mut g = game(0);
    let old = board(&mut g, "JC052", 0, 1);
    let fresh = enter(&mut g, 0);
    let p = g.pending.as_ref().unwrap();
    assert!(matches!(p.resolution, ChoiceResolution::Unique));
    assert_eq!(
        (p.choice.min, p.choice.max, p.choice.allow_decline),
        (Some(1), Some(1), Some(false))
    );
    choose(&mut g, vec![fresh.clone()]);
    assert!(g.board(&fresh).is_none() && g.board(&old).is_some());
    assert!(matches!(
        g.pending.as_ref().unwrap().resolution,
        ChoiceResolution::Declare { .. }
    ));
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    assert!(g.board(&old).is_some());
    finish_optional(&mut g);
    checkpoint(&g);
}
#[test]
fn jc052_three_leaders_and_control_change_require_sacrifices_until_only_one_remains() {
    let mut g = game(0);
    let one = board(&mut g, "JC052", 0, 1);
    let two = board(&mut g, "JC052", 2, 2);
    let three = board(&mut g, "JC052", 3, 3);
    g.add_control(
        &two,
        0,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    g.add_control(
        &three,
        0,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    g.settle_deaths();
    g.drive().unwrap();
    checkpoint(&g);
    choose(&mut g, vec![two]);
    assert!(g.pending.is_some());
    choose(&mut g, vec![three]);
    assert!(g.pending.is_none());
    assert!(g.board(&one).is_some());
    assert_eq!(g.leader_options(0).len(), 1);
    checkpoint(&g);
}
#[test]
fn jc052_leaders_on_different_controllers_coexist_and_hidden_leader_is_not_a_conflict() {
    let mut g = game(0);
    board(&mut g, "JC052", 0, 2);
    board(&mut g, "JC052", 1, 3);
    let h = board(&mut g, "JC052", 0, 1);
    g.board_mut(&h).unwrap().face_down = true;
    g.settle_deaths();
    g.drive().unwrap();
    assert!(g.pending.is_none());
    assert_eq!(g.leader_options(0).len(), 1);
    checkpoint(&g);
}
#[test]
fn jc052_optional_entry_metadata_and_leader_metadata_tampering_reject_both_loaders() {
    let mut g = game(0);
    enter(&mut g, 0);
    for n in 0..12 {
        let mut b = g.clone();
        let p = b.pending.as_mut().unwrap();
        match n {
            0 => p.choice.min = Some(1),
            1 => p.choice.allow_decline = Some(false),
            2 => p.choice.max = Some(2),
            3 => p.choice.options[0].label.clear(),
            4 => p.choice.description.clear(),
            5 => p.choice.title.clear(),
            6 => p.choice.id.clear(),
            7 => p.seat = 2,
            8 => p.choice.amount = Some(1),
            9 => p.choice.options.clear(),
            10 => p.choice.player_id = "p2".into(),
            _ => {
                if let ChoiceResolution::Declare { stage, .. } = &mut p.resolution {
                    *stage = DeclareChoice::Target
                }
            }
        }
        invalid(&g, b, &format!("entry-metadata-{n}"));
    }
    choose(&mut g, vec![]);
    let second = board(&mut g, "JC052", 0, 3);
    g.enter_triggers(0, "JC052", &second, false);
    g.drive().unwrap();
    for n in 0..9 {
        let mut b = g.clone();
        let p = b.pending.as_mut().unwrap();
        match n {
            0 => p.choice.min = Some(0),
            1 => p.choice.allow_decline = Some(true),
            2 => p.choice.max = Some(2),
            3 => p.choice.options.pop().map(|_| ()).unwrap(),
            4 => p.choice.title.clear(),
            5 => p.choice.description.clear(),
            6 => p.seat = 2,
            7 => p.choice.options[0].label.clear(),
            _ => p.choice.amount = Some(1),
        }
        invalid(&g, b, &format!("leader-metadata-{n}"));
    }
}
#[test]
fn jc052_frames_payment_regions_program_and_replay_tampering_reject_both_loaders() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 7);
    let id = held(&mut g, 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    for n in 0..7 {
        let mut b = g.clone();
        let f = b.stack[0].frame.as_mut().unwrap();
        match n {
            0 => f.source.source_region_instance = None,
            1 => f.already_paid.clear(),
            2 => f.source.play_source = None,
            3 => f.chosen_region = Some(3),
            4 => f.cursor = 1,
            5 => f.steps.push(Step {
                context: 0,
                op: Op::JC052DestroyOtherOriginalRegionCharacters,
            }),
            _ => f.source.card.definition = "JC125".into(),
        }
        invalid(&g, b, &format!("deploy-frame-{n}"));
    }
    let mut b = g.clone();
    let f = b.stack.pop().unwrap().frame.unwrap();
    b.effects.push_back(Effect::Frame { frame: Box::new(f) });
    invalid(&g, b, "direct-deploy-without-stack");
    pass_top(&mut g);
    choose(&mut g, vec!["accept".into()]);
    for n in 0..7 {
        let mut b = g.clone();
        let f = b.stack[0].frame.as_mut().unwrap();
        match n {
            0 => f.source.source_region_instance = None,
            1 => f.actor = 2,
            2 => f.steps.clear(),
            3 => f.steps.push(f.steps[0].clone()),
            4 => f.guard = GuardState::Accepted,
            5 => f.cursor = 1,
            _ => f.source.card.face_down = true,
        }
        invalid(&g, b, &format!("entry-frame-{n}"));
    }
    let mut b = g.clone();
    let f = b.stack.pop().unwrap().frame.unwrap();
    b.effects.push_back(Effect::Frame { frame: Box::new(f) });
    invalid(&g, b, "direct-or-replayed-entry");
}
#[test]
fn jc052_four_ui_inputs_are_actual_deploy_accept_decline_and_leader_commands() {
    let mut g = game(0);
    g.room_id = "f00000000000000000000052".into();
    fund(&mut g, 0, "JC042", 7);
    let id = held(&mut g, 0);
    let a = Action {
        card_id: Some(id),
        region: Some(2),
        ..Action::new("deploy")
    };
    snapshot("deploy", &g, &a);
    apply(&mut g, 0, a);
    pass_top(&mut g);
    let p = g.pending.as_ref().unwrap();
    let a = Action {
        choice_id: Some(p.choice.id.clone()),
        selected: Some(vec!["accept".into()]),
        ..Action::new("choose")
    };
    snapshot("entry-accept", &g, &a);
    let mut accepted = g.clone();
    apply(&mut accepted, 0, a.clone());
    pass_top(&mut accepted);
    finish_optional(&mut accepted);
    let a = Action {
        selected: Some(vec![]),
        ..a
    };
    snapshot("entry-decline", &g, &a);
    choose(&mut g, vec![]);
    let second = board(&mut g, "JC052", 0, 3);
    g.enter_triggers(0, "JC052", &second, false);
    g.drive().unwrap();
    let p = g.pending.as_ref().unwrap();
    let a = Action {
        choice_id: Some(p.choice.id.clone()),
        selected: Some(vec![second]),
        ..Action::new("choose")
    };
    snapshot("leader-sacrifice", &g, &a);
    apply(&mut g, 0, a);
    finish_optional(&mut g);
}

#[test]
fn jc052_nontargeted_wave_ignores_barrier_and_shield_and_cleans_owned_attachments() {
    let mut g = game(0);
    let victim = board(&mut g, "JZ08", 2, 2);
    g.board_mut(&victim).unwrap().shield = 3;
    assert!(g.has_barrier(g.board(&victim).unwrap().1));
    let ink = g.make_card("WM038", 3);
    g.attachments.push(Attachment {
        card: ink,
        host_id: victim.clone(),
    });
    let source = enter(&mut g, 0);
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    assert!(g.board(&victim).is_none() && g.board(&source).is_some());
    assert!(g.attachments.is_empty());
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JZ08"));
    assert!(g.players[3]
        .graveyard
        .iter()
        .any(|c| c.definition == "WM038"));
    finish_optional(&mut g);
    checkpoint(&g);
}

#[test]
fn jc052_restored_optional_and_forced_choices_continue_once_and_cannot_skip_enforcement() {
    use crate::room::{RoomCommand, RoomEnvelope, SessionAction};
    let mut g = game(0);
    enter(&mut g, 0);
    for selected in [vec![], vec!["accept".into()]] {
        let room = envelope(&g);
        let command = RoomCommand {
            command_id: format!("jc052-entry-once-{}", selected.len()),
            expected_version: room.revision,
            action: SessionAction::Game {
                action: Action {
                    choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
                    selected: Some(selected.clone()),
                    ..Action::new("choose")
                },
            },
        };
        let first = room.transition(0, Some(command.clone()), 0).unwrap();
        assert!(first.error_code.is_none());
        let restored = RoomEnvelope::from_persisted(&first.state).unwrap();
        let repeat = restored.transition(0, Some(command), 0).unwrap();
        let repeat = RoomEnvelope::from_persisted(&repeat.state).unwrap();
        assert_eq!(
            serde_json::to_value(&restored.game).unwrap(),
            serde_json::to_value(&repeat.game).unwrap()
        );
        assert_eq!(restored.game.stack.len(), selected.len());
    }
    choose(&mut g, vec![]);
    let fresh = board(&mut g, "JC052", 0, 3);
    g.enter_triggers(0, "JC052", &fresh, false);
    g.drive().unwrap();
    let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    let choice_id = restored.pending.as_ref().unwrap().choice.id.clone();
    reject(
        &mut restored,
        0,
        Action {
            choice_id: Some(choice_id),
            selected: Some(vec![]),
            ..Action::new("choose")
        },
    );
    choose(&mut restored, vec![fresh]);
    finish_optional(&mut restored);
    assert_eq!(restored.leader_options(0).len(), 1);
    checkpoint(&restored);
    let mut bad = g.clone();
    bad.pending = None;
    bad.effects.retain(|e| !matches!(e, Effect::Unique { .. }));
    invalid(&g, bad, "leaders-without-enforcement");
}

#[test]
fn jc052_source_guard_preserves_existing_granted_renown_and_combat_glory_programs() {
    for glory in [false, true] {
        let mut g = game(0);
        let source = board(&mut g, "JC052", 0, 2);
        // Explicit existing-grant layout, not an invented second JC052 ability.
        g.turn_attribute_modifiers.push(TurnAttributeModifier {
            target_instance: source.clone(),
            defense_bonus: 1,
            kill_bonus: 0,
            grants_retreat: false,
            printed_defense_override: None,
            grants_barrier: false, prevents_damage: false,
            ordinary_icons: Icons {
                combat: 1,
                ..Default::default()
            },
            grants_renown: !glory,
            expires_turn: g.turn,
        });
        if glory {
            let curse = g.make_card("JC089", 2);
            g.attachments.push(Attachment {
                card: curse,
                host_id: source,
            });
            let d = g.jc089_combat_glory(0, 2).unwrap();
            g.effects.push_back(Effect::Declare { declaration: d });
        } else {
            g.window = Some(Window::After(2, 2));
            g.complete_influence_contest(2);
        }
        g.drive().unwrap();
        checkpoint(&g);
        let before = g.regions[2].influence[0];
        choose(&mut g, vec!["accept".into()]);
        checkpoint(&g);
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence[0], before + 1);
        checkpoint(&g);
    }
}

#[test]
fn jc052_source_uid_cannot_alias_an_existing_other_entity_or_change_immutable_owner() {
    let mut g = game(0);
    let victim = board(&mut g, "JC125", 0, 1);
    let source = enter(&mut g, 0);
    let asset = g.players[0].assets[0].id.clone();
    let region = g.regions[1].card.id.clone();
    for (name, id) in [("board", victim), ("asset", asset), ("region", region)] {
        let mut bad = g.clone();
        if let ChoiceResolution::Declare { declaration, .. } =
            &mut bad.pending.as_mut().unwrap().resolution
        {
            declaration.source.card.id = id;
        }
        invalid(&g, bad, &format!("source-uid-alias-{name}"));
    }
    let mut bad = g.clone();
    if let ChoiceResolution::Declare { declaration, .. } =
        &mut bad.pending.as_mut().unwrap().resolution
    {
        declaration.source.card.owner = 1;
    }
    invalid(&g, bad, "source-immutable-owner");

    // Explicit existing control primitive: snapshot control is historical, owner is immutable.
    g.add_control(&source, 2, ControlLifetime::TurnEnd { turn: g.turn }, SubtypeChange::None);
    let restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    assert_eq!(restored.board(&source).unwrap().1.controller, 2);
    assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&envelope(&g)).unwrap()).is_ok());
    choose(&mut g, vec!["accept".into()]);
    let other = board(&mut g, "JC125", 2, 2);
    let mut bad = g.clone();
    bad.stack[0].frame.as_mut().unwrap().source.card.id = other.clone();
    invalid(&g, bad, "source-stack-uid-alias");
    pass_top(&mut g);
    assert!(g.board(&source).is_some() && g.board(&other).is_none());
    finish_optional(&mut g);
    checkpoint(&g);
}
