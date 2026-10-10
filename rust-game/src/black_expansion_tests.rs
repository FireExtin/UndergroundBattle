//! Native explicit layouts; paid Game commands and persisted Room transitions.
//! This is engine regression evidence, not independent desktop play acceptance.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, deck, model::*, room::*, rules::*};

fn initial(actor: usize) -> Game {
    let mut g = game(actor);
    for p in &mut g.players {
        p.deck.clear();
    }
    g
}
fn held(g: &mut Game, def: &str, actor: usize) -> String {
    let c = g.make_card(def, actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    id
}
fn deploy(g: &mut Game, def: &str, actor: usize, region: usize) -> String {
    fund(g, actor, def, 2);
    let id = held(g, def, actor);
    apply(
        g,
        actor,
        Action {
            card_id: Some(id),
            region: Some(region),
            ..Action::new("deploy")
        },
    );
    pass_top(g);
    g.regions[region]
        .cards
        .iter()
        .find(|c| c.definition == def && c.controller == actor)
        .unwrap()
        .id
        .clone()
}
fn cast(g: &mut Game, def: &str, actor: usize, target: &str) {
    let id = held(g, def, actor);
    let a = g
        .legal_actions(actor)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(&id)
                && a.action.target_id.as_deref() == Some(target)
        })
        .unwrap_or_else(|| {
            panic!(
                "missing {def} cast for seat {actor}: {:?}",
                g.legal_actions(actor)
            )
        })
        .action;
    apply(g, actor, a);
    pass_top(g);
}
fn accept(g: &mut Game) {
    choose(g, vec!["accept".into()]);
}
fn enter(g: &mut Game, def: &str, owner: usize, actor: usize, region: usize) -> String {
    // Explicit native entry input. Payment/deploy coverage uses deploy above.
    let id = board(g, def, owner, region);
    g.board_mut(&id).unwrap().controller = actor;
    g.enter_triggers(actor, def, &id, false);
    g.drive().unwrap();
    checkpoint(g);
    id
}
fn corpse(g: &mut Game, holder: usize, owner: usize) -> String {
    let c = g.make_card("XQ46", owner);
    let id = c.id.clone();
    g.players[holder].graveyard.push(c);
    id
}
fn kill(g: &mut Game, id: &str) {
    // Explicit native death input; separate tests destroy with real JC091.
    g.remove_dead(id, RemovalCause::Destroy);
    g.drive().unwrap();
    checkpoint(g);
}
fn fixture(kind: &str, g: &Game) {
    if let Ok(dir) = std::env::var("BLACK_EXPANSION_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let room = envelope(g);
        std::fs::write(
            format!("{dir}/{kind}.json"),
            serde_json::to_vec(&serde_json::json!({
                "kind": kind, "state": serde_json::to_string(&room).unwrap(),
                "views": (0..4).map(|s| room.view(s, 0)).collect::<Vec<_>>()
            }))
            .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn black_expansion_original_fields_traits_and_exact_programs() {
    let c = catalog::card("WM059");
    assert_eq!(
        (&*c.name, c.cost, &*c.color, &*c.magic, c.defense),
        ("雨夜屠夫", 2, "黑", "", Some(1))
    );
    assert!(c.unique);
    assert_eq!(c.loyalty, ["黑色"]);
    assert_eq!(c.subtypes, ["人类", "罪犯"]);
    assert_eq!(
        c.permanent_icons,
        Icons {
            combat: 1,
            ..Default::default()
        }
    );
    assert_eq!(c.temporary_icons, Icons::default());
    assert_eq!(c.keywords, ["杀伤1"]);
    assert_eq!(c.rule_traits.kill, 1);
    assert_eq!(deck::copy_limit(c), Some(3));
    let a = &definition("WM059").abilities[0];
    assert_eq!(a.event, Some(Event::EnterRegion));
    assert_eq!(a.targets.len(), 1);
    assert_eq!(a.targets[0].zone, Zone::Board);
    assert_eq!(a.targets[0].kind, EntityKind::Character);
    assert_eq!(a.targets[0].relation, Relation::EnemyTeam);
    assert_eq!(a.targets[0].range, Range::SourceRegion);
    assert_eq!(a.targets[0].printed_cost_max, Some(1));
    assert!(matches!(
        a.ops.as_slice(),
        [Op::Destroy(EntityRef::Target(0))]
    ));
    let c = catalog::card("BQ078");
    assert_eq!(
        (&*c.name, c.cost, &*c.color, &*c.magic, c.defense),
        ("专业清理员", 2, "黑", "", Some(1))
    );
    assert_eq!(c.loyalty, ["黑色"]);
    assert_eq!(c.subtypes, ["人类", "雇员"]);
    assert!(!c.unique);
    assert_eq!(
        c.permanent_icons,
        Icons {
            influence: 1,
            ..Default::default()
        }
    );
    assert_eq!(
        c.temporary_icons,
        Icons {
            combat: 1,
            ..Default::default()
        }
    );
    assert_eq!(c.keywords, ["公开"]);
    assert!(c.rule_traits.public && !c.rule_traits.spirit);
    assert_eq!(deck::copy_limit(c), Some(3));
    let d = definition("BQ078");
    assert_eq!(d.abilities[0].event, Some(Event::Enter));
    assert_eq!(d.abilities[0].targets[0].zone, Zone::Player);
    assert_eq!(d.abilities[0].targets[0].relation, Relation::Any);
    assert!(matches!(
        d.abilities[0].ops.as_slice(),
        [Op::MoveDeckTopToGraveyard {
            player: PlayerRef::Target(0),
            count: 4
        }]
    ));
    assert_eq!(d.abilities[1].event, Some(Event::Death));
    assert!(d.abilities[1].targets.is_empty());
    assert!(matches!(
        d.abilities[1].ops.as_slice(),
        [Op::BQ078ReturnNamelessCorpse]
    ));
    for a in definition("WM059").abilities.iter().chain(&d.abilities) {
        assert_eq!(a.timing, Timing::Fast);
        assert_eq!(a.response_policy, ResponsePolicy::Respondable);
        assert!(
            a.costs.is_empty()
                && a.modes.is_empty()
                && a.per_turn_limit.is_none()
                && !a.play_only
                && !a.activation_only
                && !a.requires_ready_source
        );
    }
}

#[test]
fn black_expansion_payment_loyalty_and_public_conceal_are_atomic() {
    for def in ["WM059", "BQ078"] {
        for (asset, count) in [("JC059", 2), (def, 0), (def, 1)] {
            let mut g = initial(0);
            fund(&mut g, 0, asset, count);
            let id = held(&mut g, def, 0);
            reject(
                &mut g,
                0,
                Action {
                    card_id: Some(id),
                    region: Some(2),
                    ..Action::new("deploy")
                },
            );
        }
    }
    let mut g = initial(0);
    fund(&mut g, 0, "BQ078", 4);
    let id = held(&mut g, "BQ078", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
}

#[test]
fn black_expansion_wm059_kill_participation_and_bq078_temporary_combat_use_real_contest() {
    for mode in 0..3 {
        let mut g = initial(0);
        let source = board(&mut g, "WM059", 0, 2);
        board(&mut g, "XQ37", 0, 2);
        let target = board(&mut g, "XQ36", 2, 2);
        if mode == 1 {
            g.board_mut(&source).unwrap().exhausted = true;
        }
        if mode == 2 {
            g.board_mut(&source).unwrap().face_down = true;
        }
        g.begin_window(Window::Before(2, 1));
        g.close_window().unwrap();
        g.drive().unwrap();
        let p = g.pending.as_ref().unwrap();
        assert_eq!(p.choice.amount, Some(if mode == 0 { 3 } else { 1 }));
        let a = Action {
            choice_id: Some(p.choice.id.clone()),
            allocations: Some(std::collections::BTreeMap::from([(
                target.clone(),
                if mode == 0 { 3 } else { 1 },
            )])),
            ..Action::new("choose")
        };
        apply(&mut g, 0, a);
        assert!(g.board(&target).is_none());
        checkpoint(&g);
    }
    let mut g = initial(0);
    let id = board(&mut g, "BQ078", 0, 2);
    let c = g.board(&id).unwrap().1;
    assert_eq!(
        g.current_icons(c, 2),
        Icons {
            combat: 1,
            influence: 1,
            ..Default::default()
        }
    );
    g.first_team = 1;
    let c = g.board(&id).unwrap().1;
    assert_eq!(
        g.current_icons(c, 2),
        Icons {
            influence: 1,
            ..Default::default()
        }
    );
    checkpoint(&g);
}

#[test]
fn black_expansion_wm059_paid_deploy_and_reveal_choose_only_enemy_local_cheap_public_character() {
    for hidden in [false, true] {
        let mut g = initial(0);
        let victim = board(&mut g, "XQ36", 2, 2);
        let ally = board(&mut g, "XQ36", 1, 2);
        let dear = board(&mut g, "XQ37", 2, 2);
        g.modifiers.push(CostModifier {
            actor: 2,
            filter: CardFilter::Any,
            amount: 9,
            expires_turn: g.turn,
            uses: 1,
            paid_reveal: false,
        });
        assert_eq!(g.effective_cost(2, g.board(&dear).unwrap().1), 0);
        let remote = board(&mut g, "XQ36", 2, 1);
        let back = board(&mut g, "XQ36", 3, 2);
        g.board_mut(&back).unwrap().face_down = true;
        fund(&mut g, 0, "WM059", 4);
        let hand = held(&mut g, "WM059", 0);
        apply(
            &mut g,
            0,
            Action {
                card_id: Some(hand),
                region: Some(2),
                ..Action::new(if hidden { "conceal" } else { "deploy" })
            },
        );
        if hidden {
            assert!(g.pending.is_none() && g.stack.is_empty());
            let source = g.regions[2]
                .cards
                .iter()
                .find(|c| c.definition == "WM059")
                .unwrap()
                .id
                .clone();
            apply(
                &mut g,
                0,
                Action {
                    card_id: Some(source),
                    ..Action::new("reveal")
                },
            );
        }
        pass_top(&mut g);
        assert_eq!(
            g.pending
                .as_ref()
                .unwrap()
                .choice
                .options
                .iter()
                .map(|o| &o.id)
                .collect::<Vec<_>>(),
            vec![&victim]
        );
        for illegal in [&ally, &dear, &remote, &back] {
            let id = g.pending.as_ref().unwrap().choice.id.clone();
            reject(
                &mut g,
                0,
                Action {
                    choice_id: Some(id),
                    selected: Some(vec![illegal.clone()]),
                    ..Action::new("choose")
                },
            );
        }
        fixture(
            if hidden {
                "wm059-paid-reveal-target"
            } else {
                "wm059-paid-deploy-target"
            },
            &g,
        );
        choose(&mut g, vec![victim.clone()]);
        assert!(g.board(&victim).is_some());
        assert_eq!(g.stack.len(), 1);
        pass_top(&mut g);
        assert!(g.board(&victim).is_none());
        assert!(g.players[2]
            .graveyard
            .iter()
            .any(|c| c.definition == "XQ36"));
        checkpoint(&g);
    }
}

#[test]
fn black_expansion_wm059_decline_and_absent_targets_do_nothing() {
    for eligible in [false, true] {
        let mut g = initial(0);
        let target = board(&mut g, if eligible { "XQ36" } else { "XQ37" }, 2, 2);
        deploy(&mut g, "WM059", 0, 2);
        if eligible {
            choose(&mut g, vec![]);
        }
        assert!(g.pending.is_none() && g.stack.is_empty());
        assert!(g.board(&target).is_some());
    }
}

#[test]
fn black_expansion_wm059_shield_stops_declared_destruction_and_consumes_one_shield() {
    let mut g = initial(0);
    let target = board(&mut g, "XQ36", 2, 2);
    g.board_mut(&target).unwrap().shield = 1;
    deploy(&mut g, "WM059", 0, 2);
    choose(&mut g, vec![target.clone()]);
    pass_top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.shield, 0);
    assert!(g.players[2].graveyard.is_empty());
    checkpoint(&g);
}

#[test]
fn black_expansion_target_declaration_and_stack_restore_replay_same_actions() {
    for def in ["WM059", "BQ078"] {
        let mut g = initial(0);
        let target = if def == "WM059" {
            board(&mut g, "XQ36", 2, 2)
        } else {
            let c = g.make_card("XQ46", 2);
            g.players[2].deck.push(c);
            player_id(2)
        };
        deploy(&mut g, def, 0, 2);
        let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
        choose(&mut g, vec![target.clone()]);
        choose(&mut restored, vec![target]);
        assert_eq!(
            serde_json::to_value(&g).unwrap(),
            serde_json::to_value(&restored).unwrap()
        );
        let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_top(&mut g);
        pass_top(&mut restored);
        assert_eq!(
            serde_json::to_value(&g).unwrap(),
            serde_json::to_value(&restored).unwrap()
        );
        checkpoint(&g);
    }
}

#[test]
fn black_expansion_wm059_frozen_region_actor_recheck_target_and_source_departure() {
    for change in 0..6 {
        let mut g = initial(0);
        let target = board(&mut g, "XQ36", 2, 2);
        let source = enter(&mut g, "WM059", 1, 0, 2);
        choose(&mut g, vec![target.clone()]);
        // Explicit response-state perturbations exercise the real frame guard.
        match change {
            0 => g.return_hand(&source),
            1 => {
                let (_, c) = g.remove_board(&source).unwrap();
                g.regions[1].cards.push(c);
            }
            2 => g.board_mut(&source).unwrap().controller = 2,
            3 => {
                let (_, c) = g.remove_board(&target).unwrap();
                g.regions[1].cards.push(c);
            }
            4 => g.board_mut(&target).unwrap().controller = 1,
            _ => g.return_hand(&target),
        }
        checkpoint(&g);
        pass_top(&mut g);
        let destroyed = g.players[2]
            .graveyard
            .iter()
            .any(|c| c.definition == "XQ36");
        assert_eq!(destroyed, change < 3);
        checkpoint(&g);
    }
}

#[test]
fn black_expansion_wm059_real_return_response_invalidates_original_target_and_keeps_paid_cost() {
    let mut g = initial(0);
    let target = board(&mut g, "XQ36", 2, 2);
    deploy(&mut g, "WM059", 0, 2);
    choose(&mut g, vec![target.clone()]);
    fund(&mut g, 2, "JC075", 3);
    let rescuer = board(&mut g, "JC075", 2, 1);
    let available = g.resources(2);
    // The active team retains first response priority. Its two real passes
    // transfer the existing window to the defender before the paid rescue.
    apply(&mut g, 0, Action::new("pass"));
    apply(&mut g, 1, Action::new("pass"));
    assert_eq!(g.priority_team, g.team(2));
    let rescue = g
        .legal_actions(2)
        .into_iter()
        .find(|a| {
            a.action.kind == "activate"
                && a.action.card_id.as_deref() == Some(&rescuer)
                && a.action.target_id.as_deref() == Some(&target)
        })
        .unwrap()
        .action;
    apply(&mut g, 2, rescue);
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    let after_response = g.resources(2);
    assert!(after_response < available);
    pass_top(&mut g);
    assert_eq!(g.resources(2), after_response);
    assert!(!g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ36"));
}

#[test]
fn black_expansion_region_move_emits_wm059_region_trigger_but_no_bq078_entry_trigger() {
    for def in ["WM059", "BQ078"] {
        let mut g = initial(0);
        let source = board(&mut g, def, 0, 1);
        let victim = board(&mut g, "XQ36", 2, 2);
        let (_, c) = g.remove_board(&source).unwrap();
        let snapshot = g.source_snapshot(&c, Some(2));
        g.regions[2].cards.push(c);
        // Exact production MoveOnBoard event, with an explicit movement layout.
        g.emit_event(0, snapshot, Event::EnterRegion);
        g.drive().unwrap();
        checkpoint(&g);
        if def == "WM059" {
            assert_eq!(g.pending.as_ref().unwrap().choice.options[0].id, victim);
            choose(&mut g, vec![victim.clone()]);
            pass_top(&mut g);
            assert!(g.board(&victim).is_none());
        } else {
            assert!(g.pending.is_none() && g.stack.is_empty());
        }
    }
}

#[test]
fn black_expansion_bq078_paid_entry_mills_single_chosen_living_player_short_and_empty_deck() {
    for actor in 0..4 {
        for target in 0..4 {
            for count in [0, 1, 3, 4, 5] {
                let mut g = initial(actor);
                let mut top = vec![];
                for seat in 0..4 {
                    for _ in 0..count {
                        let c = g.make_card("JZ61", seat);
                        if seat == target {
                            top.push(c.id.clone());
                        }
                        g.players[seat].deck.push(c);
                    }
                }
                deploy(&mut g, "BQ078", actor, 2);
                assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 4);
                choose(&mut g, vec![player_id(target)]);
                pass_top(&mut g);
                for seat in 0..4 {
                    assert_eq!(
                        g.players[seat].graveyard.len(),
                        if seat == target { count.min(4) } else { 0 }
                    );
                    assert_eq!(
                        g.players[seat].deck.len(),
                        if seat == target {
                            count.saturating_sub(4)
                        } else {
                            count
                        }
                    );
                    assert!(!g.players[seat].eliminated && g.players[seat].hand.is_empty());
                }
                for (old, c) in top.iter().zip(&g.players[target].graveyard) {
                    assert_ne!(&c.id, old);
                    assert_eq!((c.owner, c.controller), (target, target));
                }
                assert!(g.pending.is_none() && g.effects.is_empty() && g.stack.is_empty());
                checkpoint(&g);
            }
        }
    }
}

#[test]
fn black_expansion_bq078_entry_decline_eliminated_player_and_foreign_owner() {
    let mut g = initial(0);
    g.players[3].eliminated = true;
    let foreign = g.make_card("XQ46", 1);
    let old = foreign.id.clone();
    g.players[2].deck.push(foreign);
    deploy(&mut g, "BQ078", 0, 2);
    assert!(!g
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .any(|o| o.id == "p3"));
    choose(&mut g, vec![]);
    assert_eq!(g.players[2].deck[0].id, old);
    let source = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "BQ078")
        .unwrap()
        .id
        .clone();
    g.enter_triggers(0, "BQ078", &source, false);
    g.drive().unwrap();
    choose(&mut g, vec!["p2".into()]);
    pass_top(&mut g);
    assert!(g.players[2].deck.is_empty() && g.players[2].graveyard.is_empty());
    assert_eq!(g.players[1].graveyard[0].definition, "XQ46");
    assert_ne!(g.players[1].graveyard[0].id, old);
    assert!(g.pending.is_none()); // Milling XQ46 never creates a board Death event.
    checkpoint(&g);
}

#[test]
fn black_expansion_bq078_entry_survives_real_destroy_response_and_uses_frozen_player() {
    let mut g = initial(0);
    let c = g.make_card("XQ36", 2);
    g.players[2].deck.push(c);
    let source = deploy(&mut g, "BQ078", 0, 2);
    choose(&mut g, vec!["p2".into()]);
    fund(&mut g, 0, "JC091", 4);
    cast(&mut g, "JC091", 0, &source);
    assert!(g.board(&source).is_none());
    // Its genuine death trigger is a distinct optional declaration.
    assert_eq!(g.pending.as_ref().unwrap().choice.options[0].id, "accept");
    choose(&mut g, vec![]);
    pass_top(&mut g);
    assert!(g.players[2].deck.is_empty());
    assert_eq!(g.players[2].graveyard[0].definition, "XQ36");
    assert!(g.pending.is_none() && g.stack.is_empty());
}

#[test]
fn black_expansion_bq078_death_uses_snapshot_controller_graveyard_and_hand_preserves_owner() {
    for actor in 0..4 {
        let mut g = initial(actor);
        let source = board(&mut g, "BQ078", (actor + 1) % 4, 2);
        g.board_mut(&source).unwrap().controller = actor;
        let chosen = corpse(&mut g, actor, (actor + 2) % 4);
        let other = corpse(&mut g, (actor + 1) % 4, (actor + 1) % 4);
        kill(&mut g, &source);
        let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution
        else {
            panic!()
        };
        assert_eq!(declaration.actor, actor);
        assert_eq!(declaration.source.card.controller, actor);
        assert_eq!(declaration.source.card.id, source);
        assert_eq!(declaration.ability.event, Some(Event::Death));
        accept(&mut g);
        pass_top(&mut g);
        let p = g.pending.as_ref().unwrap();
        assert_eq!(
            (p.seat, p.choice.min, p.choice.max, p.choice.allow_decline),
            (actor, Some(1), Some(1), Some(false))
        );
        assert_eq!(
            p.choice.options.iter().map(|o| &o.id).collect::<Vec<_>>(),
            vec![&chosen]
        );
        fixture(&format!("bq078-death-corpse-choice-{actor}"), &g);
        let bad = Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec![other]),
            ..Action::new("choose")
        };
        reject(&mut g, actor, bad);
        choose(&mut g, vec![chosen.clone()]);
        let c = g.players[actor]
            .hand
            .iter()
            .find(|c| c.definition == "XQ46")
            .unwrap();
        assert_ne!(c.id, chosen);
        assert_eq!((c.owner, c.controller), ((actor + 2) % 4, (actor + 2) % 4));
        assert!(!c.face_down && !c.exhausted && c.damage == 0 && c.wounds == 0 && c.shield == 0);
        assert!(g.pending.is_none() && g.stack.is_empty());
        checkpoint(&g);
    }
}

#[test]
fn black_expansion_bq078_death_empty_decline_hidden_and_selected_count() {
    for mode in 0..4 {
        let mut g = initial(0);
        let source = board(&mut g, "BQ078", 0, 2);
        if mode == 2 {
            g.board_mut(&source).unwrap().face_down = true;
        }
        let first = if mode > 0 {
            Some(corpse(&mut g, 0, 0))
        } else {
            None
        };
        let second = if mode == 3 {
            Some(corpse(&mut g, 0, 0))
        } else {
            None
        };
        kill(&mut g, &source);
        if mode == 2 {
            assert!(g.pending.is_none());
            continue;
        }
        if mode == 1 {
            choose(&mut g, vec![]);
            assert!(g.players[0].hand.is_empty());
            continue;
        }
        accept(&mut g);
        pass_top(&mut g);
        if mode == 0 {
            assert!(g.pending.is_none());
            continue;
        }
        let id = g.pending.as_ref().unwrap().choice.id.clone();
        for selected in [
            vec![],
            vec![first.clone().unwrap(), second.unwrap()],
            vec![first.clone().unwrap(), first.clone().unwrap()],
        ] {
            reject(
                &mut g,
                0,
                Action {
                    choice_id: Some(id.clone()),
                    selected: Some(selected),
                    ..Action::new("choose")
                },
            );
        }
        let selection = Action {
            choice_id: Some(id),
            selected: Some(vec![first.unwrap()]),
            ..Action::new("choose")
        };
        reject(&mut g, 2, selection.clone());
        apply(&mut g, 0, selection.clone());
        reject(&mut g, 0, selection);
        assert_eq!(g.players[0].hand.len(), 1);
    }
}

#[test]
fn black_expansion_bq078_corpse_candidates_are_evaluated_after_responses_and_not_targeted() {
    let mut g = initial(0);
    let source = board(&mut g, "BQ078", 0, 2);
    kill(&mut g, &source);
    accept(&mut g);
    let chosen = corpse(&mut g, 0, 0); // Explicit response-state addition.
    g.players[0]
        .graveyard
        .iter_mut()
        .find(|c| c.id == chosen)
        .unwrap()
        .shield = 9;
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options[0].id, chosen);
    choose(&mut g, vec![chosen]);
    assert_eq!(g.players[0].hand[0].definition, "XQ46");
    assert_eq!(g.players[0].hand[0].shield, 0);
    checkpoint(&g);
}

#[test]
fn black_expansion_bq078_real_destroy_then_recovery_and_game_restore_match() {
    let mut g = initial(0);
    let source = board(&mut g, "BQ078", 0, 2);
    corpse(&mut g, 0, 0);
    fund(&mut g, 0, "JC091", 4);
    cast(&mut g, "JC091", 0, &source);
    accept(&mut g);
    let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    pass_top(&mut g);
    pass_top(&mut restored);
    assert_eq!(
        serde_json::to_value(&g).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    let chosen = g.pending.as_ref().unwrap().choice.options[0].id.clone();
    choose(&mut g, vec![chosen.clone()]);
    choose(&mut restored, vec![chosen]);
    assert_eq!(
        serde_json::to_value(&g).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    checkpoint(&g);
}

#[test]
fn black_expansion_room_recovery_command_replay_and_duplicate_receipt() {
    let mut g = initial(0);
    let source = board(&mut g, "BQ078", 0, 2);
    corpse(&mut g, 0, 0);
    kill(&mut g, &source);
    accept(&mut g);
    pass_top(&mut g);
    let room = envelope(&g);
    let p = room.game.pending.as_ref().unwrap();
    let command = RoomCommand {
        command_id: "black-corpse-return".into(),
        expected_version: room.revision,
        action: SessionAction::Game {
            action: Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(vec![p.choice.options[0].id.clone()]),
                ..Action::new("choose")
            },
        },
    };
    let before = serde_json::to_string(&room).unwrap();
    let e = room.transition(0, Some(command.clone()), 0).unwrap();
    assert!(e.error_code.is_none(), "{:?}", e.error_message);
    let replay = RoomEnvelope::from_persisted(&before)
        .unwrap()
        .transition(0, Some(command.clone()), 0)
        .unwrap();
    assert_eq!(e.state, replay.state);
    let after = RoomEnvelope::from_persisted(&e.state).unwrap();
    let duplicate = after.transition(0, Some(command.clone()), 600000).unwrap();
    assert_eq!(duplicate.state, e.state);
    assert_eq!(after.game.players[0].hand.len(), 1);
    assert!(after.game.pending.is_none());
    for s in 0..4 {
        assert_eq!(
            serde_json::to_value(after.view(s, 0)).unwrap(),
            serde_json::to_value(RoomEnvelope::from_persisted(&e.state).unwrap().view(s, 0))
                .unwrap()
        );
    }
    if let Ok(dir) = std::env::var("BLACK_EXPANSION_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/room-corpse-return.json"),
            serde_json::to_vec(&serde_json::json!({
                "state": before, "seat": 0, "command": command, "expected": e,
                "views": (0..4).map(|s| after.view(s, 0)).collect::<Vec<_>>()
            }))
            .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn black_expansion_exact_definitions_reject_rekeys_transplants_nested_ops_and_traits() {
    for def in ["WM059", "BQ078"] {
        for target in [def, "JC002"] {
            for mutation in 0..7 {
                let mut all = definitions().clone();
                let mut d = definition(def).clone();
                match mutation {
                    0 => {}
                    1 => d.abilities[0].key = "other".into(),
                    2 => d.abilities[0].event = Some(Event::Death),
                    3 => {
                        d.abilities[0].ops =
                            vec![Op::ForEachLivingPlayer(d.abilities[0].ops.clone())]
                    }
                    4 => d.abilities[0].costs = vec![Cost::Assets(1)],
                    5 => d.traits.spirit = true,
                    _ => d.abilities.push(d.abilities[0].clone()),
                }
                all.insert(target.into(), d);
                assert_eq!(
                    validate_definitions(&all).is_ok(),
                    target == def && mutation == 0
                );
            }
        }
    }
}

#[test]
fn black_expansion_persisted_frame_and_corpse_choice_tampering_rejected() {
    let mut g = initial(0);
    let target = board(&mut g, "XQ36", 2, 2);
    enter(&mut g, "WM059", 0, 0, 2);
    choose(&mut g, vec![target]);
    let original = envelope(&g);
    for mutation in 0..14 {
        let mut room = original.clone();
        let s = room.game.stack.last_mut().unwrap();
        let f = s.frame.as_mut().unwrap();
        match mutation {
            0 => f.cursor = 1,
            1 => f.guard = GuardState::Accepted,
            2 => f.source.card.definition = "JC002".into(),
            3 => f.source.card.controller = 2,
            4 => f.steps[0].context = 2,
            5 => f.steps.push(f.steps[0].clone()),
            6 => f.ability_key = "other".into(),
            7 => f.targets[0].spec.printed_cost_max = Some(9),
            8 => f.targets[0].spec.relation = Relation::Any,
            9 => f.source.source_region_instance = None,
            10 => f.already_paid.push(PaidCost::Assets(vec![])),
            11 => s.target = Some("p0".into()),
            12 => s.id.clear(),
            _ => f.targets[0].public.instance_id = "fake".into(),
        }
        invalid(mutation, &room);
    }
    let mut g = initial(0);
    let source = board(&mut g, "BQ078", 0, 2);
    corpse(&mut g, 0, 0);
    kill(&mut g, &source);
    accept(&mut g);
    pass_top(&mut g);
    let original = envelope(&g);
    for mutation in 0..17 {
        let mut room = original.clone();
        let p = room.game.pending.as_mut().unwrap();
        let ChoiceResolution::Frame { frame: f, choice } = &mut p.resolution else {
            panic!()
        };
        match mutation {
            0 => p.seat = 2,
            1 => p.choice.player_id = player_id(2),
            2 => p.choice.min = Some(0),
            3 => p.choice.max = Some(2),
            4 => p.choice.allow_decline = Some(true),
            5 => p.choice.options.clear(),
            6 => p.choice.options[0].id = "fake".into(),
            7 => p.choice.options[0].card = None,
            8 => p.choice.options.push(p.choice.options[0].clone()),
            9 => p.choice.kind = "search".into(),
            10 => f.source.card.definition = "WM059".into(),
            11 => f.cursor = 0,
            12 => f.guard = GuardState::Unchecked,
            13 => f.actor = 2,
            14 => f.steps[0].op = Op::ForEachLivingPlayer(vec![Op::BQ078ReturnNamelessCorpse]),
            15 => *choice = FrameChoice::JZ50DeathSearch,
            _ => p.choice.id.clear(),
        }
        invalid(14 + mutation, &room);
    }
}
fn invalid(index: usize, r: &RoomEnvelope) {
    let state = serde_json::to_string(r).unwrap();
    assert!(RoomEnvelope::from_persisted(&state).is_err());
    assert!(Game::from_persisted(&serde_json::to_string(&r.game).unwrap()).is_err());
    if let Ok(dir) = std::env::var("BLACK_EXPANSION_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/invalid-{index:02}.json"),
            serde_json::to_vec(&serde_json::json!({
                "state": state, "expectedError": true
            }))
            .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn black_expansion_unknown_corpse_and_non_candidate_definitions_return_errors_without_panics() {
    let mut g = initial(0);
    let source = board(&mut g, "BQ078", 0, 2);
    corpse(&mut g, 0, 0);
    let unrelated = g.make_card("JZ61", 0);
    g.players[0].graveyard.push(unrelated);
    kill(&mut g, &source);
    accept(&mut g);
    pass_top(&mut g);
    let original = envelope(&g);
    let p = original.game.pending.as_ref().unwrap();
    let selected = vec![p.choice.options[0].id.clone()];
    let ChoiceResolution::Frame { frame, .. } = &p.resolution else {
        panic!()
    };
    let action = Action {
        choice_id: Some(p.choice.id.clone()),
        selected: Some(selected.clone()),
        ..Action::new("choose")
    };
    for (index, def) in ["XQ46", "JZ61", "BQ078"].into_iter().enumerate() {
        let mut room = original.clone();
        room.game.players[0]
            .graveyard
            .iter_mut()
            .find(|c| c.definition == def)
            .unwrap()
            .definition = "unknown".into();
        let state = serde_json::to_string(&room.game).unwrap();
        let load = std::panic::catch_unwind(|| Game::from_persisted(&state));
        assert!(load.is_ok(), "Game loading {def} must not panic");
        let error = load.unwrap().unwrap_err();
        assert!(error.contains("准入") && error.contains("定义"), "{error}");
        let state = serde_json::to_string(&room).unwrap();
        let load = std::panic::catch_unwind(|| RoomEnvelope::from_persisted(&state));
        assert!(load.is_ok(), "Room loading {def} must not panic");
        let error = load.unwrap().unwrap_err();
        assert!(error.contains("准入") && error.contains("定义"), "{error}");
        let mut bad = room.game.clone();
        let before = serde_json::to_value(&bad).unwrap();
        let checked = std::panic::catch_unwind(|| bad.validate_black_expansion_state());
        assert!(
            checked.is_ok(),
            "Black state guard with {def} must not panic"
        );
        assert!(checked.unwrap().unwrap_err().contains("BQ078"));
        let applied = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            bad.apply(0, action.clone())
        }));
        assert!(applied.is_ok(), "Choosing with {def} must not panic");
        let error = applied.unwrap().unwrap_err();
        assert!(error.contains("准入") && error.contains("定义"), "{error}");
        assert_eq!(serde_json::to_value(&bad).unwrap(), before);
        let started = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            bad.bq078_return_start((**frame).clone())
        }));
        assert!(started.is_ok(), "Starting with {def} must not panic");
        assert!(started.unwrap().unwrap_err().contains("BQ078"));
        assert_eq!(serde_json::to_value(&bad).unwrap(), before);
        let completed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            bad.bq078_return_complete(frame, 0, &selected)
        }));
        assert!(completed.is_ok(), "Completing with {def} must not panic");
        assert!(completed.unwrap().unwrap_err().contains("BQ078"));
        assert_eq!(serde_json::to_value(&bad).unwrap(), before);
        invalid(100 + index, &room);
    }
}
