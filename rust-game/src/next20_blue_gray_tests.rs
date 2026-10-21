use crate::{catalog, model::*, next20_blue_gray as bg, rules::*, unit_support::*};
fn activate(g: &mut Game, seat: usize, source: &str, key: &str, target: Option<&str>) {
    act(
        g,
        seat,
        Action {
            card_id: Some(source.into()),
            ability_id: Some(key.into()),
            target_id: target.map(str::to_owned),
            ..Action::new("activate")
        },
    );
}
fn search(g: &mut Game) {
    until_choice(g, "next-blue-gray-search");
}
fn no_leak(g: &Game, seat: usize, ids: &[String]) {
    for other in 0..4 {
        if other != seat {
            let value = serde_json::to_string(&g.view(other)).unwrap();
            for id in ids {
                assert!(
                    !value.contains(id),
                    "private deck instance leaked to p{other}: {id}"
                );
            }
            assert!(g.view(other).pending_choice.is_none());
        }
    }
}
fn invalid_state(g: &Game, mutate: impl FnOnce(&mut Game)) {
    let mut bad = g.clone();
    mutate(&mut bad);
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut envelope = serde_json::to_value(crate::jc029_tests::envelope(g)).unwrap();
    envelope["game"] = serde_json::to_value(bad).unwrap();
    assert!(
        crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&envelope).unwrap())
            .is_err()
    );
}
#[test]
fn next20_blue_gray_complete_definitions_reject_cost_or_program_transplants() {
    crate::rules::validate_definitions(crate::rules::definitions()).unwrap();
    for id in bg::CARDS {
        assert!(catalog::card(id).supported);
        let d = bg::definition(id).unwrap();
        bg::validate_definition(id, &d).unwrap();
        for a in &d.abilities {
            bg::validate_ability(id, a).unwrap();
        }
        let mut bad = d.clone();
        bad.traits.retreat = !bad.traits.retreat;
        assert!(bg::validate_definition(id, &bad).is_err());
    }
    for id in ["JC039", "JZ28", "CQ17"] {
        assert!(catalog::card(id).unique);
    }
    let mut a = bg::definition("BQ059")
        .unwrap()
        .abilities
        .last()
        .unwrap()
        .clone();
    a.costs[0] = Cost::Assets(6);
    assert!(bg::validate_ability("BQ059", &a).is_err());
    assert!(bg::validate_ability("JC029", &a).is_err());
}
#[test]
fn next20_blue_gray_jc039_public_attach_and_both_discount_channels() {
    for reveal in [false, true] {
        let mut g = game();
        g.players[0].society_zone.card = Some(g.make_card("MSJC04", 0));
        fund(&mut g, 0, "JC029", 10);
        let emblem = attach(&mut g, "JC039", "society:p0");
        activate(&mut g, 0, &emblem, "next-bg-jc039-discount", None);
        top(&mut g);
        invalid_state(&g, |bad| bad.modifiers[0].paid_reveal = false);
        invalid_state(&g, |bad| bad.modifiers[0].amount = 2);
        invalid_state(&g, |bad| bad.modifiers[0].uses = 2);
        let id = if reveal {
            let id = field(&mut g, "JZ23", 0);
            g.board_mut(&id).unwrap().face_down = true;
            id
        } else {
            held(&mut g, "JZ23", 0)
        };
        let before = resources(&g, 0);
        act(
            &mut g,
            0,
            Action {
                card_id: Some(id),
                region: (!reveal).then_some(0),
                ..Action::new(if reveal { "reveal" } else { "deploy" })
            },
        );
        assert_eq!(resources(&g, 0), before - 3);
        assert!(g.modifiers.iter().all(|m| m.uses == 0));
        drain(&mut g);
        assert!(g.regions[0]
            .cards
            .iter()
            .any(|c| c.definition == "JZ23" && !c.face_down));
    }
}
#[test]
fn next20_blue_gray_jc039_hide_is_own_blue_any_region_and_resets_marker() {
    let mut g = game();
    g.players[0].society_zone.card = Some(g.make_card("MSJC04", 0));
    fund(&mut g, 0, "JC029", 8);
    let emblem = attach(&mut g, "JC039", "society:p0");
    let own = field(&mut g, "JC029", 0);
    g.board_mut(&own).unwrap().permanent_combat_markers = 2;
    let c = g.regions[0].cards.pop().unwrap();
    g.regions[1].cards.push(c);
    let team = field(&mut g, "JC029", 1);
    let gray = field(&mut g, "CQ17", 0);
    for id in [&team, &gray] {
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(emblem.clone()),
                ability_id: Some("next-bg-jc039-hide".into()),
                target_id: Some(id.clone()),
                ..Action::new("activate")
            },
        );
    }
    activate(&mut g, 0, &emblem, "next-bg-jc039-hide", Some(&own));
    top(&mut g);
    assert!(g.board(&own).is_none());
    let hidden = g.regions[1]
        .cards
        .iter()
        .find(|c| c.definition == "JC029")
        .unwrap();
    assert!(hidden.face_down);
    assert_eq!(hidden.permanent_combat_markers, 0);
}
#[test]
fn next20_blue_gray_xg10_exact_named_attachment_costs_private_choices_and_control() {
    for embrace in [false, true] {
        let mut g = game();
        fund(&mut g, 0, "JC029", 7);
        let source = field(&mut g, "XG10", 0);
        let host = field(&mut g, if embrace { "JC125" } else { "JC029" }, 1);
        let ids = deck_cards(&mut g, 0, &["XQ14", "BQ040", "JC036", "JC029"]);
        let before = resources(&g, 0);
        activate(
            &mut g,
            0,
            &source,
            if embrace {
                "next-bg-xg10-embrace"
            } else {
                "next-bg-xg10-blood"
            },
            Some(&host),
        );
        assert_eq!(resources(&g, 0), before - if embrace { 5 } else { 1 });
        search(&mut g);
        no_leak(&g, 0, &ids);
        let p = g.pending.clone().unwrap();
        assert_eq!(p.choice.options.len(), if embrace { 1 } else { 2 });
        invalid_state(&g, |bad| bad.pending.as_mut().unwrap().choice.max = Some(2));
        reject(
            &mut g,
            0,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(vec![p.choice.options[0].id.clone()]),
                region: Some(0),
                ..Action::new("choose")
            },
        );
        reject(
            &mut g,
            1,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(vec![p.choice.options[0].id.clone()]),
                ..Action::new("choose")
            },
        );
        let replay = Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![p.choice.options[0].id.clone()]),
            ..Action::new("choose")
        };
        act(&mut g, 0, replay.clone());
        reject(&mut g, 0, replay);
        drain(&mut g);
        assert!(g
            .attachments
            .iter()
            .any(|a| a.host_id == host
                && a.card.definition == if embrace { "JC036" } else { "XQ14" }));
        if embrace {
            let c = g.board(&host).unwrap().1;
            assert_eq!(c.controller, 0);
            assert!(g.current_subtypes(c).iter().any(|s| s == "吸血鬼"));
        }
    }
}
#[test]
fn next20_blue_gray_bq059_shield_is_entry_trigger_and_x_is_exact_bounded() {
    let mut g = game();
    fund(&mut g, 0, "CQ17", 16);
    let id = held(&mut g, "BQ059", 0);
    assert_eq!(g.players[0].hand.last().unwrap().shield, 0);
    play_card(&mut g, 0, &id);
    drain(&mut g);
    let id = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "BQ059")
        .unwrap()
        .id
        .clone();
    assert_eq!(g.board(&id).unwrap().1.shield, 2);
    assert!(g.has_retreat(g.board(&id).unwrap().1));
    assert_eq!(g.effective_kill(g.board(&id).unwrap().1), 4);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id.clone()),
            ability_id: Some("next-bg-bq059-search-6".into()),
            ..Action::new("activate")
        },
    );
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id.clone()),
            ability_id: Some("next-bg-bq059-search-5".into()),
            option: Some("6".into()),
            ..Action::new("activate")
        },
    );
    let ids = deck_cards(&mut g, 0, &["BQ054", "CQ16", "CQ17", "LC22", "BQ059"]);
    let before = resources(&g, 0);
    activate(&mut g, 0, &id, "next-bg-bq059-search-3", None);
    assert_eq!(resources(&g, 0), before - 3);
    search(&mut g);
    assert_eq!(
        g.pending
            .as_ref()
            .unwrap()
            .choice
            .options
            .iter()
            .map(|o| o.id.clone())
            .collect::<Vec<_>>(),
        vec![ids[2].clone(), ids[3].clone()]
    );
    no_leak(&g, 0, &ids);
    choose(&mut g, vec![ids[3].clone()]);
    drain(&mut g);
    assert!(g.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "LC22" && !c.face_down));
    g.board_mut(&id).unwrap().exhausted = false;
    let before = resources(&g, 0);
    activate(&mut g, 0, &id, "next-bg-bq059-search-5", None);
    assert_eq!(resources(&g, 0), before - 5);
    search(&mut g);
    assert!(g.pending.as_ref().unwrap().choice.options.is_empty());
    choose(&mut g, vec![]);
}
#[test]
fn next20_blue_gray_cq17_exact_cost_aura_is_local_other_own_gray() {
    let mut g = game();
    fund(&mut g, 0, "CQ17", 3);
    let source = field(&mut g, "CQ17", 0);
    let own = field(&mut g, "BQ054", 0);
    let teammate = field(&mut g, "BQ054", 1);
    let blue = field(&mut g, "JC029", 0);
    assert_eq!(g.cq17_combat_bonus(g.board(&own).unwrap().1, 0), 1);
    assert!(g.has_retreat(g.board(&own).unwrap().1));
    for id in [&source, &teammate, &blue] {
        assert_eq!(g.cq17_combat_bonus(g.board(id).unwrap().1, 0), 0);
        assert!(!g.has_retreat(g.board(id).unwrap().1));
    }
    let ids = deck_cards(&mut g, 0, &["BQ054", "CQ16", "CQ17", "LC22"]);
    activate(&mut g, 0, &source, "next-bg-cq17-search", None);
    search(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 1);
    choose(&mut g, vec![ids[1].clone()]);
    drain(&mut g);
    g.board_mut(&source).unwrap().face_down = true;
    assert!(!g.has_retreat(g.board(&own).unwrap().1));
}
#[test]
fn next20_blue_gray_cq08_reveal_search_and_shared_preparation_original_region() {
    let mut g = game();
    fund(&mut g, 0, "JC029", 4);
    let id = field(&mut g, "CQ08", 0);
    g.board_mut(&id).unwrap().face_down = true;
    let ids = deck_cards(&mut g, 0, &["CQ08", "JC029"]);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            ..Action::new("reveal")
        },
    );
    until_choice(&mut g, "trigger");
    choose(&mut g, vec!["accept".into()]);
    search(&mut g);
    no_leak(&g, 0, &ids);
    choose(&mut g, vec![ids[0].clone()]);
    assert!(g.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "CQ08" && c.face_down));
    drain(&mut g);
    // Three public sources, spread across regions, create one shared choice of
    // source. The chosen source fixes the destination for its entire frame.
    g.regions.iter_mut().for_each(|r| r.cards.clear());
    let chosen = field(&mut g, "CQ08", 0);
    let second = field(&mut g, "CQ08", 0);
    let third = field(&mut g, "CQ08", 0);
    let c = g.regions[0].cards.pop().unwrap();
    g.regions[1].cards.push(c);
    let ids = deck_cards(&mut g, 0, &["JZ28", "JC029"]);
    g.jc035_preparation = None;
    g.begin_window(Window::Prepare);
    g.drive().unwrap();
    restore(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 3);
    choose(&mut g, vec![chosen]);
    assert!(g.pending.is_none());
    assert_eq!(
        g.stack
            .iter()
            .filter(|s| s
                .frame
                .as_ref()
                .is_some_and(|f| f.ability_key == "next-bg-cq08-prepare"))
            .count(),
        1
    );
    search(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 1);
    invalid_state(&g, |bad| {
        if let ChoiceResolution::Frame { frame, .. } = &mut bad.pending.as_mut().unwrap().resolution
        {
            frame.source.region = Some(1);
        }
    });
    choose(&mut g, vec![ids[0].clone()]);
    drain(&mut g);
    assert!(g.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "JZ28" && !c.face_down));
    assert!(g.board(&second).is_some() && g.board(&third).is_some());
    let n = g
        .jc035_preparation
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .filter(|e| {
            matches!(
                e.decision,
                Some(
                    crate::jc035::PreparationDecision::Accepted { .. }
                        | crate::jc035::PreparationDecision::Resolved { .. }
                )
            )
        })
        .count();
    assert_eq!(n, 1);
}
#[test]
fn next20_blue_gray_jz23_observes_reveal_pays_once_marker_persists_and_discard_is_private() {
    let mut g = game();
    fund(&mut g, 0, "JC029", 8);
    field(&mut g, "JZ23", 0);
    let hidden = field(&mut g, "JC029", 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    let discarded = held(&mut g, "JC125", 1);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    until_choice(&mut g, "trigger");
    assert!(
        matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare {declaration,..} if declaration.source.card.definition=="JZ23")
    );
    let before = resources(&g, 0);
    choose(&mut g, vec!["p1".into()]);
    assert_eq!(resources(&g, 0), before - 1);
    until_choice(&mut g, "discard");
    let c = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC029")
        .unwrap();
    let id = c.id.clone();
    assert_eq!(c.permanent_combat_markers, 1);
    for viewer in 0..4 {
        let view = g.card_view(c, viewer, Some(0), None);
        assert_eq!(view.permanent_combat_markers, Some(1));
    }
    invalid_state(&g, |bad| {
        bad.players[1].hand[0].permanent_combat_markers = 1
    });
    no_leak(&g, 1, &[discarded.clone()]);
    choose(&mut g, vec![discarded]);
    drain(&mut g);
    let before = g.icons(g.board(&id).unwrap().1, 0).combat;
    g.turn += 1;
    g.first_team = 1;
    restore(&mut g);
    assert_eq!(g.board(&id).unwrap().1.permanent_combat_markers, 1);
    assert!(g.icons(g.board(&id).unwrap().1, 0).combat >= 1);
    assert!(before >= 1);
    let index = g.regions[0].cards.iter().position(|c| c.id == id).unwrap();
    let c = g.regions[0].cards.remove(index);
    g.regions[1].cards.push(c);
    g.board_mut(&id).unwrap().controller = 1;
    assert_eq!(g.board(&id).unwrap().1.permanent_combat_markers, 1);
    restore(&mut g);
}
#[test]
fn next20_blue_gray_jz28_actual_discard_top_four_private_not_mill() {
    let mut g = game();
    let source = field(&mut g, "JZ28", 0);
    g.players[0].society_zone.card = Some(g.make_card("MSJC04", 0));
    fund(&mut g, 0, "JC029", 5);
    let discarded = held(&mut g, "JC125", 2);
    let ids = deck_cards(&mut g, 0, &["JC125", "JC029", "JC125", "JC125", "CQ08"]);
    let attachment = held(&mut g, "JZ29", 0);
    play_target(&mut g, &attachment, "society:p0");
    until_choice(&mut g, "trigger");
    choose(&mut g, vec!["p2".into()]);
    until_choice(&mut g, "discard");
    choose(&mut g, vec![discarded]);
    until_choice(&mut g, "trigger");
    assert!(
        matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare {declaration,..} if declaration.source.card.id==source)
    );
    choose(&mut g, vec!["accept".into()]);
    search(&mut g);
    // The fifth card is not offered even though it is another vampire.
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 1);
    let preview = &g.view(0).pending_choice.unwrap().preview_cards;
    assert_eq!(preview.len(), 4);
    no_leak(&g, 0, &ids);
    invalid_state(&g, |bad| {
        if let ChoiceResolution::Frame {
            choice: FrameChoice::NextBlueGraySearch { inspected_ids },
            ..
        } = &mut bad.pending.as_mut().unwrap().resolution
        {
            inspected_ids.reverse();
        }
    });
    let picked = g.pending.as_ref().unwrap().choice.options[0].id.clone();
    choose(&mut g, vec![picked]);
    drain(&mut g);
    assert!(g.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "JC029" && c.face_down));
    g.mill_cards(2, 1);
    assert!(!g.effects.iter().any(
        |e| matches!(e,Effect::Declare {declaration} if declaration.source.card.definition=="JZ28")
    ));
}
#[test]
fn next20_blue_gray_bq059_two_real_shields_cancel_two_enemy_targets() {
    let mut g = game();
    fund(&mut g, 0, "CQ17", 7);
    let id = held(&mut g, "BQ059", 0);
    play_card(&mut g, 0, &id);
    drain(&mut g);
    let id = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "BQ059")
        .unwrap()
        .id
        .clone();
    g.begin_window(Window::Action(1));
    fund(&mut g, 2, "JC091", 9);
    for remaining in [1, 0] {
        cast(&mut g, "JC091", 2, &id);
        assert_eq!(g.board(&id).unwrap().1.shield, remaining);
    }
    cast(&mut g, "JC091", 2, &id);
    assert!(g.board(&id).is_none());
}
#[test]
fn next20_blue_gray_source_region_replacement_keeps_cost_and_never_retargets() {
    let mut g = game();
    let source = field(&mut g, "CQ17", 0);
    fund(&mut g, 0, "CQ17", 2);
    let ids = deck_cards(&mut g, 0, &["CQ16"]);
    activate(&mut g, 0, &source, "next-bg-cq17-search", None);
    g.regions[0].card = g.make_card("DQJC115", 0);
    search(&mut g);
    choose(&mut g, vec![ids[0].clone()]);
    assert_eq!(resources(&g, 0), 0);
    assert_eq!(g.players[0].deck.len(), 1);
    assert!(!g
        .regions
        .iter()
        .flat_map(|r| &r.cards)
        .any(|c| c.definition == "CQ16"));
    let mut deployed = game();
    fund(&mut deployed, 0, "JC029", 4);
    let id = held(&mut deployed, "JZ23", 0);
    play_card(&mut deployed, 0, &id);
    deployed.regions[0].card = deployed.make_card("DQJC115", 0);
    drain(&mut deployed);
    assert!(!deployed.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "JZ23"));
    assert!(deployed.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JZ23"));
}
#[test]
fn next20_blue_gray_jz28_reveal_raid_deals_two_and_preserves_leader() {
    let mut g = game();
    fund(&mut g, 0, "JC029", 6);
    let hidden = field(&mut g, "JZ28", 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    let target = field(&mut g, "BQ059", 2);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    until_choice(&mut g, "trigger");
    choose(&mut g, vec![target.clone()]);
    top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.damage, 2);
    assert!(crate::jc052::printed_leader("JZ28"));
}
#[test]
fn next20_blue_gray_jz23_receipt_payment_and_duplicate_observation_are_bound() {
    let mut g = game();
    fund(&mut g, 0, "JC029", 8);
    field(&mut g, "JZ23", 0);
    let hidden = field(&mut g, "JC029", 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    act(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    until_choice(&mut g, "trigger");
    choose(&mut g, vec!["p2".into()]);
    invalid_state(&g, |bad| {
        bad.stack
            .last_mut()
            .unwrap()
            .frame
            .as_mut()
            .unwrap()
            .already_paid
            .clear()
    });
    invalid_state(&g, |bad| {
        if let Op::JZ23RevealReward {
            observation: Some(o),
        } = &mut bad.stack.last_mut().unwrap().frame.as_mut().unwrap().steps[0].op
        {
            o.vampire = false;
        }
    });
    invalid_state(&g, |bad| {
        let copy = bad.stack.last().unwrap().clone();
        bad.stack.push(copy);
    });
    drain(&mut g);
}
