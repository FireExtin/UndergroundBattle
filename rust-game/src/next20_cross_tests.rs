//! Cross-package acceptance paths. Every client command round-trips all four views.
use crate::{catalog, model::*, rules::*, unit_support::*};

fn request(g: &Game, selected: Vec<String>) -> Action {
    Action {
        choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
        selected: Some(selected),
        ..Action::new("choose")
    }
}
#[track_caller]
fn invalid(g: &Game, change: impl FnOnce(&mut Game)) {
    let mut bad = g.clone();
    change(&mut bad);
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut room = crate::jc029_tests::envelope(g);
    room.game = bad;
    assert!(
        crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err()
    );
}
fn cause(g: &mut Game) -> &mut ResolutionFrame {
    let ChoiceResolution::Rumor { batch } = &mut g.pending.as_mut().unwrap().resolution else {
        panic!("rumor expected")
    };
    batch.cause.as_mut().unwrap()
}
fn duplicate_response(g: &mut Game, mut response: StackItem) {
    let id = g.id();
    response.id = id.clone();
    response.frame.as_mut().unwrap().frame_id = id;
    g.stack.push(response);
}
fn grave(g: &mut Game, definition: &str) -> String {
    let c = g.make_card(definition, 0);
    let id = c.id.clone();
    g.players[0].graveyard.push(c);
    id
}

#[test]
fn next20_cross_registry_contains_exact_twenty_complete_prints() {
    let ids = [
        "JC051", "JC055", "JZ32", "JZ34", "JZ41", "BQ077", "BQ085", "CQ26", "XG27", "XQ39",
        "JC039", "JZ23", "JZ28", "XG10", "CQ08", "BQ059", "CQ17", "WM039", "WM040", "WM042",
    ];
    // This test's twenty IDs remain the second batch; the current complete
    // catalogue also contains the independently pinned third batch.
    crate::unit_support::assert_abi94_catalog_pool();
    crate::rules::validate_definitions(crate::rules::definitions()).unwrap();
    assert_eq!(ids.iter().filter(|id| catalog::card(id).unique).count(), 10);
    for id in ids {
        assert!(catalog::card(id).supported);
        assert!(!crate::rules::definition(id).abilities.is_empty());
    }
}

#[test]
fn next20_cross_mixed_preparation_preserves_forced_order_target_and_same_name_limit() {
    let mut g = game();
    let red = field(&mut g, "JC055", 0);
    let black = field(&mut g, "BQ085", 0);
    let blue = field(&mut g, "CQ08", 0);
    field(&mut g, "CQ08", 0);
    field(&mut g, "CQ08", 0);
    let a = grave(&mut g, "JC125");
    let b = grave(&mut g, "JZ50");
    let deck = deck_cards(&mut g, 0, &["JC125", "JC125", "JC125", "JZ28"]);
    g.begin_window(Window::End);
    until_choice(&mut g, "jc035-preparation-order");
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 5);
    let decline = request(&g, vec![]);
    reject(&mut g, 0, decline);
    choose(&mut g, vec![red.clone()]);
    let p = g.pending.as_ref().unwrap();
    assert_eq!(
        (p.choice.kind.as_str(), p.choice.min, p.choice.allow_decline),
        ("jc055-preparation-target", Some(1), Some(false))
    );
    let decline = request(&g, vec![]);
    reject(&mut g, 0, decline);
    choose(&mut g, vec![red.clone()]);
    choose(&mut g, vec![blue]);
    assert_eq!(
        g.pending
            .as_ref()
            .unwrap()
            .choice
            .options
            .iter()
            .map(|o| &o.id)
            .collect::<Vec<_>>(),
        vec![&black]
    );
    choose(&mut g, vec![black.clone()]);
    until_choice(&mut g, "bq085-bottom");
    choose(&mut g, vec![b, a]);
    until_choice(&mut g, "next-blue-gray-search");
    assert!(g.view(2).pending_choice.is_none());
    choose(&mut g, vec![deck[3].clone()]);
    drain(&mut g);
    assert_eq!(g.board(&red).unwrap().1.time_markers, 1);
    assert!(g.board(&black).is_some());
    assert!(g.players[0].graveyard.is_empty());
    assert_eq!(
        g.regions[0]
            .cards
            .iter()
            .filter(|c| c.definition == "JZ28" && !c.face_down)
            .count(),
        1
    );
    let entries = &g.jc035_preparation.as_ref().unwrap().entries;
    assert_eq!(
        entries
            .iter()
            .filter(|e| e.source.card.definition == "CQ08"
                && matches!(
                    e.decision,
                    Some(crate::jc035::PreparationDecision::Resolved { .. })
                ))
            .count(),
        1
    );
    assert_eq!(
        entries
            .iter()
            .filter(|e| e.source.card.definition == "CQ08"
                && matches!(
                    e.decision,
                    Some(crate::jc035::PreparationDecision::Declined)
                ))
            .count(),
        2
    );
}

#[test]
fn next20_cross_jz23_rumor_receipt_cannot_replay_paid_reward_or_discard() {
    for sealed in [false, true] {
        let mut g = game();
        field(&mut g, "JZ23", 0);
        let vampire = field(&mut g, "JC029", 0);
        g.board_mut(&vampire).unwrap().face_down = true;
        let host = field(&mut g, "JC125", 2);
        let news = held(&mut g, "WM042", 2);
        fund(&mut g, 0, "JC029", 8);
        act(
            &mut g,
            0,
            Action {
                card_id: Some(vampire),
                ..Action::new("reveal")
            },
        );
        until_choice(&mut g, "trigger");
        choose(&mut g, vec!["p2".into()]);
        let response = g.stack.last().unwrap().clone();
        let paid = resources(&g, 0);
        until_choice(&mut g, "discard");
        let vampire = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "JC029")
            .unwrap()
            .id
            .clone();
        assert_eq!(g.board(&vampire).unwrap().1.permanent_combat_markers, 1);
        choose(&mut g, vec![news]);
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "rumor");
        assert!(g.view(0).pending_choice.is_none());
        invalid(&g, |bad| duplicate_response(bad, response.clone()));
        invalid(&g, |bad| {
            let mut f = cause(bad).clone();
            f.cursor = 0;
            f.guard = GuardState::Unchecked;
            bad.effects.push_back(Effect::Frame { frame: Box::new(f) });
        });
        invalid(&g, |bad| {
            if let Op::JZ23RevealReward { observation } = &mut cause(bad).steps[0].op {
                *observation = None;
            }
        });
        invalid(&g, |bad| {
            if let ChoiceResolution::Rumor { batch } = &mut bad.pending.as_mut().unwrap().resolution
            {
                batch.redraw = true;
            }
        });
        invalid(&g, |bad| {
            cause(bad).source.observed_exposure = Some(1);
        });
        let command = request(&g, if sealed { vec![host] } else { vec![] });
        act(&mut g, 2, command.clone());
        reject(&mut g, 2, command);
        drain(&mut g);
        assert_eq!(resources(&g, 0), paid);
        assert_eq!(g.board(&vampire).unwrap().1.permanent_combat_markers, 1);
        assert_eq!(
            g.sealed_cards
                .iter()
                .filter(|s| s.card.definition == "WM042")
                .count(),
            usize::from(sealed)
        );
        assert_eq!(
            g.players[2]
                .graveyard
                .iter()
                .filter(|c| c.definition == "WM042")
                .count(),
            usize::from(!sealed)
        );
    }
}

#[test]
fn next20_cross_cq26_observes_actual_mill_once_then_rumor_discard_notifies_jz28_once() {
    for sealed in [false, true] {
        let mut g = game();
        let angel = field(&mut g, "CQ26", 0);
        let watcher = field(&mut g, "JZ28", 2);
        let hand_news = held(&mut g, "WM042", 0);
        deck_cards(&mut g, 0, &["JC125", "WM042", "JC125", "JC125", "JC084"]);
        fund(&mut g, 0, "JC084", 1);
        let mill = held(&mut g, "XQ36", 0);
        act(
            &mut g,
            0,
            Action {
                card_id: Some(mill),
                region: Some(0),
                ..Action::new("deploy")
            },
        );
        until_choice(&mut g, "trigger");
        choose(&mut g, vec!["accept".into()]);
        until_choice(&mut g, "rumor");
        assert!(!g.effects.iter().any(|e| matches!(e, Effect::Declare { declaration } if declaration.source.card.definition == "CQ26")));
        choose(&mut g, vec![angel.clone()]);
        until_choice(&mut g, "trigger");
        let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution
        else {
            panic!("CQ26 trigger")
        };
        assert_eq!(declaration.source.card.id, angel);
        let Op::BlackDeckToGraveyard {
            observation: Some(observation),
        } = &declaration.ability.ops[0]
        else {
            panic!("mill receipt")
        };
        assert_eq!(observation.cards.len(), 3);
        assert!(observation.cards.iter().all(|c| c.definition == "JC125"));
        assert_eq!(g.players[0].graveyard.len(), 3);
        choose(&mut g, vec!["accept".into()]);
        let response = g.stack.last().unwrap().clone();
        until_choice(&mut g, "discard");
        choose(&mut g, vec![hand_news]);
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "rumor");
        invalid(&g, |bad| duplicate_response(bad, response.clone()));
        invalid(&g, |bad| {
            if let Op::BlackDeckToGraveyard { observation } = &mut cause(bad).steps[0].op {
                *observation = None;
            }
        });
        invalid(&g, |bad| {
            if let ChoiceResolution::Rumor { batch } = &mut bad.pending.as_mut().unwrap().resolution
            {
                batch.redraw = false;
            }
        });
        invalid(&g, |bad| {
            if let ChoiceResolution::Rumor { batch } = &mut bad.pending.as_mut().unwrap().resolution
            {
                batch.from_deck = true;
            }
        });
        let command = request(&g, if sealed { vec![angel] } else { vec![] });
        act(&mut g, 0, command.clone());
        reject(&mut g, 0, command);
        if sealed {
            assert!(g.pending.is_none());
            assert!(g.players[0].hand.is_empty());
        } else {
            let ChoiceResolution::Declare { declaration, .. } =
                &g.pending.as_ref().unwrap().resolution
            else {
                panic!("one actual discard trigger")
            };
            assert_eq!(declaration.source.card.id, watcher);
            assert!(g.players[0].hand.iter().any(|c| c.definition == "JC084"));
            choose(&mut g, vec![]);
        }
        assert!(g.effects.is_empty());
        assert!(g.pending.is_none());
        assert_eq!(g.players[0].graveyard.len(), if sealed { 3 } else { 4 });
        assert_eq!(g.sealed_cards.len(), if sealed { 2 } else { 1 });
    }
}

#[test]
fn next20_cross_red_and_black_capture_one_shared_death_wave_before_removal() {
    let mut g = game();
    field(&mut g, "JZ34", 0);
    field(&mut g, "BQ085", 0);
    field(&mut g, "JC125", 0);
    field(&mut g, "WM039", 2);
    grave(&mut g, "XG27");
    fund(&mut g, 0, "JC047", 10);
    let disaster = held(&mut g, "JZ37", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(disaster),
            ..Action::new("play")
        },
    );
    until_choice(&mut g, "trigger");
    let mut declarations = g
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
    if let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution {
        declarations.push(declaration);
    }
    for (definition, count) in [("JZ34", 3), ("BQ085", 2), ("XG27", 1)] {
        assert_eq!(
            declarations
                .iter()
                .filter(|d| d.source.card.definition == definition)
                .count(),
            count,
            "{definition}"
        );
    }
    assert!(g.regions.iter().all(|r| r.cards.is_empty()));
    invalid(&g, |bad| {
        if let ChoiceResolution::Declare { declaration, .. } =
            &bad.pending.as_ref().unwrap().resolution
        {
            bad.effects.push_back(Effect::Declare {
                declaration: declaration.clone(),
            });
        }
    });
}

fn granted_reward(definition: &str, glory: bool, replace_region: bool) {
    let mut g = game();
    let host = field(&mut g, definition, 0);
    if glory {
        let protector = field(&mut g, "LC12", 0);
        act(
            &mut g,
            0,
            Action {
                card_id: Some(protector),
                ability_id: Some("protect-local-character".into()),
                target_id: Some(host.clone()),
                ..Action::new("activate")
            },
        );
        top(&mut g);
        fund(&mut g, 0, "JC084", 2);
    } else {
        fund(&mut g, 0, "JC075", 2);
    }
    let grant = held(&mut g, if glory { "JC089" } else { "JC074" }, 0);
    play_target(&mut g, &grant, &host);
    top(&mut g);
    g.begin_window(Window::Before(0, if glory { 1 } else { 2 }));
    until_choice(&mut g, "trigger");
    let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution
    else {
        panic!("reward")
    };
    assert_eq!(declaration.source.card.id, host);
    assert_eq!(
        declaration.ability.key,
        if glory {
            "jc089-combat-glory"
        } else {
            "renown"
        }
    );
    invalid(&g, |bad| {
        if let ChoiceResolution::Declare { declaration, .. } =
            &mut bad.pending.as_mut().unwrap().resolution
        {
            declaration.source.source_region_instance = None;
        }
    });
    invalid(&g, |bad| {
        if let ChoiceResolution::Declare { declaration, .. } =
            &mut bad.pending.as_mut().unwrap().resolution
        {
            let Op::PlaceInfluence { amount, .. } = &mut declaration.ability.ops[0] else {
                panic!("reward program")
            };
            *amount = 2;
        }
    });
    choose(&mut g, vec!["accept".into()]);
    invalid(&g, |bad| {
        if let Op::PlaceInfluence { amount, .. } =
            &mut bad.stack.last_mut().unwrap().frame.as_mut().unwrap().steps[0].op
        {
            *amount = 2;
        }
    });
    let before = g.regions[0].influence[0];
    if replace_region {
        g.regions[0].card = g.make_card("DQJC115", 0);
    }
    top(&mut g);
    assert_eq!(
        g.regions[0].influence[0],
        before + u32::from(!replace_region),
        "{definition}, glory={glory}, replaced={replace_region}"
    );
}
#[test]
fn next20_cross_all_groups_accept_real_renown_and_reject_replacement_region_rewards() {
    for definition in ["JC055", "CQ26", "JZ23", "WM039"] {
        for replaced in [false, true] {
            granted_reward(definition, false, replaced);
        }
    }
}
#[test]
fn next20_cross_all_groups_accept_real_glory_and_reject_replacement_region_rewards() {
    for definition in ["JC055", "CQ26", "JZ23", "WM039"] {
        for replaced in [false, true] {
            granted_reward(definition, true, replaced);
        }
    }
}
