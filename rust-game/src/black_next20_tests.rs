//! Explicit layouts, public Game::apply commands and four-seat restored views.
//! Fixtures may arrange zones; every tested trigger is caused by a client action.
use crate::{black_next20, catalog, model::*, rules::*, unit_support::*};

fn resolve(g: &mut Game) {
    let id = g.stack.last().expect("response frame").id.clone();
    for _ in 0..32 {
        if !g.stack.iter().any(|s| s.id == id) {
            return;
        }
        assert!(
            g.pending.is_none(),
            "unhandled choice {:?}",
            g.pending.as_ref().map(|p| &p.choice.kind)
        );
        pass(g);
    }
    panic!("response did not resolve");
}
fn offered(
    g: &Game,
    actor: usize,
    kind: &str,
    source: &str,
    key: Option<&str>,
    target: Option<&str>,
) -> Action {
    g.legal_actions(actor)
        .into_iter()
        .find(|a| {
            a.action.kind == kind
                && a.action.card_id.as_deref() == Some(source)
                && key.is_none_or(|key| a.action.ability_id.as_deref() == Some(key))
                && target.is_none_or(|target| a.action.target_id.as_deref() == Some(target))
        })
        .unwrap_or_else(|| {
            panic!(
                "missing offered {kind} {source} {key:?} {target:?}: {:?}",
                g.legal_actions(actor)
            )
        })
        .action
}
fn deploy(g: &mut Game, actor: usize, source: &str, region: usize) {
    let a = g
        .legal_actions(actor)
        .into_iter()
        .find(|a| {
            a.action.kind == "deploy"
                && a.action.card_id.as_deref() == Some(source)
                && a.action.region == Some(region)
        })
        .expect("offered deploy")
        .action;
    act(g, actor, a);
    resolve(g);
}
fn mill(g: &mut Game, actor: usize) {
    fund(g, actor, "JC084", 1);
    let source = held(g, "XQ36", actor);
    deploy(g, actor, &source, 0);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
    choose(g, vec!["accept".into()]);
    resolve(g);
}
fn grave(g: &mut Game, id: &str, actor: usize) -> String {
    let c = g.make_card(id, actor);
    let uid = c.id.clone();
    g.players[actor].graveyard.push(c);
    uid
}
fn prepare_from_end(g: &mut Game) {
    // Fixture starts at the public end-step window, then client passes advance
    // the real turn and preparation observation, without calling emit_event.
    g.begin_window(Window::End);
    for _ in 0..16 {
        if g.pending.is_some() {
            return;
        }
        pass(g);
    }
    panic!("preparation choice missing");
}
fn butcher(g: &mut Game, sacrifice: &str, target: &str) -> String {
    let source = field(g, "JC094", 0);
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.kind == "activate"
                && a.action.card_id.as_deref() == Some(source.as_str())
                && a.action.target_id.as_deref() == Some(target)
                && a.action
                    .cost_selected
                    .as_ref()
                    .is_some_and(|v| v == &[sacrifice.to_string()])
        })
        .expect("offered sacrifice destroy")
        .action;
    act(g, 0, a);
    source
}
fn mark(g: &mut Game, actor: usize) -> String {
    g.players[actor].society_zone.card = Some(g.make_card("MSJC04", actor));
    fund(g, actor, "JC084", 2);
    let source = held(g, "XQ39", actor);
    let a = offered(
        g,
        actor,
        "play",
        &source,
        None,
        Some(&format!("society:p{actor}")),
    );
    act(g, actor, a);
    resolve(g);
    g.attachments
        .iter()
        .find(|a| a.card.definition == "XQ39" && a.card.controller == actor)
        .unwrap()
        .card
        .id
        .clone()
}
fn ability(g: &mut Game, actor: usize, source: &str, key: &str, target: Option<&str>) {
    let a = offered(g, actor, "activate", source, Some(key), target);
    act(g, actor, a);
    resolve(g);
}
fn no_black_trigger(g: &Game) {
    assert!(
        !g.pending.as_ref().is_some_and(|p| matches!(&p.resolution,
        ChoiceResolution::Declare { declaration: d, .. } if d.ability.key.starts_with("black20-")))
    );
    assert!(!g.effects.iter().any(
        |e| matches!(e, Effect::Declare { declaration: d } if d.ability.key.starts_with("black20-"))
    ));
}

fn rejects_saved_or_live_black_state(mut bad: Game) {
    let state = serde_json::to_string(&bad).unwrap();
    let expected = Game::from_persisted(&state)
        .err()
        .expect("Game restore must reject");
    let room = crate::room::RoomEnvelope::from_game(bad.clone());
    assert_eq!(
        crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).err(),
        Some(expected.clone())
    );
    assert_eq!(bad.apply(0, Action::new("pass")).err(), Some(expected));
    assert_eq!(serde_json::to_string(&bad).unwrap(), state);
}

#[test]
fn black_next20_registry_complete_fields_and_closed_program_admission() {
    crate::rules::validate_definitions(crate::rules::definitions()).unwrap();
    for id in black_next20::CARDS {
        let c = catalog::card(id);
        assert!(c.supported);
        assert_eq!(c.color, "黑");
        assert!(c.loyalty.iter().all(|l| l == "黑色"));
        assert_eq!(c.unique, *id != "BQ077");
        assert_eq!(crate::deck::copy_limit(c), Some(3));
        let mut definition = crate::rules::definition(id).clone();
        definition.abilities[0].label.push('!');
        let mut map = crate::rules::definitions().clone();
        map.insert((*id).into(), definition);
        assert!(crate::rules::validate_definitions(&map).is_err());
    }
    assert_eq!(catalog::card("XQ39").name, "方碑印记");
    assert_eq!(catalog::card("CQ26").permanent_icons.combat, 0);
    assert_eq!(catalog::card("CQ26").temporary_icons.combat, 1);
    assert!(catalog::card("CQ26").keywords.iter().any(|k| k == "非资产"));
    assert!(crate::rules::definition("XG27").traits.spirit);
    let mut discount = crate::rules::definition("XQ39").abilities[1].clone();
    assert!(crate::rules::validate_ability("JC125", &discount).is_err());
    discount.ops = vec![Op::CostReductionOnFaceUpOrPaidReveal {
        filter: CardFilter::Any,
        amount: 1,
    }];
    assert!(crate::rules::validate_ability("XQ39", &discount).is_err());
}

#[test]
fn black_next20_fish_public_deploy_equipment_rejection_and_water_permanent_combat() {
    let mut g = game();
    g.regions[1].card = g.make_card("DQJC107", 0);
    fund(&mut g, 0, "JC084", 3);
    let fish = held(&mut g, "BQ077", 0);
    deploy(&mut g, 0, &fish, 1);
    let (r, fish) = g
        .regions
        .iter()
        .enumerate()
        .find_map(|(r, region)| {
            region
                .cards
                .iter()
                .find(|c| c.definition == "BQ077")
                .map(|c| (r, c))
        })
        .unwrap();
    let id = fish.id.clone();
    assert_eq!(g.current_permanent_combat(fish, r), 2);
    assert_eq!(g.icons(fish, r).combat, 2);
    let equipment = held(&mut g, "BQ022", 0);
    assert!(!g
        .legal_actions(0)
        .iter()
        .any(|a| a.action.card_id.as_deref() == Some(&equipment)
            && a.action.target_id.as_deref() == Some(&id)));
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(equipment),
            target_id: Some(id.clone()),
            ..Action::new("play")
        },
    );
    let fish = g.regions[1].cards.pop().unwrap();
    g.regions[0].cards.push(fish);
    assert_eq!(g.icons(g.board(&id).unwrap().1, 0).combat, 1);
    restore(&mut g);
}

#[test]
fn black_next20_fish_actual_mill_targets_only_water_and_uses_fresh_identity() {
    let mut g = game();
    g.regions[0].card = g.make_card("DQJC112", 0);
    g.regions[1].card = g.make_card("DQJC107", 0);
    g.regions[2].card = g.make_card("DQJC114", 0);
    let old = deck_cards(&mut g, 0, &["BQ077"])[0].clone();
    mill(&mut g, 0);
    let p = g.pending.clone().unwrap();
    assert!(p.choice.options.iter().any(|o| o.id == "region:1"));
    assert!(p.choice.options.iter().any(|o| o.id == "region:2"));
    assert!(p.choice.options.iter().all(|o| o.id != "region:0"));
    let grave_id = g.players[0]
        .graveyard
        .iter()
        .find(|c| c.definition == "BQ077")
        .unwrap()
        .id
        .clone();
    assert_ne!(old, grave_id);
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec!["region:0".into()]),
            ..Action::new("choose")
        },
    );
    choose(&mut g, vec!["region:1".into()]);
    assert_eq!(
        g.stack.last().unwrap().frame.as_ref().unwrap().targets[0]
            .region_instance
            .as_deref(),
        Some(g.regions[1].card.id.as_str())
    );
    resolve(&mut g);
    let entered = g.regions[1]
        .cards
        .iter()
        .find(|c| c.definition == "BQ077")
        .unwrap();
    assert!(!entered.face_down && !entered.exhausted);
    assert_ne!(entered.id, grave_id);
    assert!(g.players[0].graveyard.is_empty());
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec!["region:1".into()]),
            ..Action::new("choose")
        },
    );
}

#[test]
fn black_next20_fish_other_players_deck_owner_grave_and_region_replacement() {
    let mut g = game();
    g.regions[1].card = g.make_card("DQJC107", 0);
    g.players[0].deck = vec![g.make_card("BQ077", 2)];
    mill(&mut g, 0);
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    choose(&mut g, vec!["region:1".into()]);
    // Response fixture replaces the physical region; the old binding must fizzle.
    g.regions[1].card = g.make_card("DQJC107", 0);
    restore(&mut g);
    resolve(&mut g);
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "BQ077"));
    assert!(!g.regions[1].cards.iter().any(|c| c.definition == "BQ077"));
}

#[test]
fn black_next20_cq26_once_per_multi_character_batch_private_optional_discard_and_dynamic_icons() {
    let mut g = game();
    let angel = field(&mut g, "CQ26", 0);
    deck_cards(&mut g, 0, &["JC125", "JC125", "JC125", "JC125", "JC084"]);
    for _ in 0..3 {
        grave(&mut g, "JC125", 0);
    }
    let hidden_hand = held(&mut g, "BQ077", 0);
    let secret = held(&mut g, "JZ50", 0);
    mill(&mut g, 0);
    assert_eq!(g.players[0].graveyard.len(), 7);
    assert_eq!(
        g.icons(g.board(&angel).unwrap().1, 0),
        Icons {
            investigation: 1,
            combat: 2,
            influence: 1
        }
    );
    assert_eq!(g.current_permanent_combat(g.board(&angel).unwrap().1, 0), 1);
    assert_eq!(g.effects.iter().filter(|e| matches!(e, Effect::Declare { declaration:d } if d.source.card.definition == "CQ26")).count(), 0);
    choose(&mut g, vec!["accept".into()]);
    resolve(&mut g);
    let p = g.pending.clone().unwrap();
    assert_eq!(
        (p.choice.kind.as_str(), p.choice.min, p.choice.max),
        ("discard", Some(0), Some(1))
    );
    for viewer in [1, 2, 3] {
        let view = serde_json::to_string(&g.view(viewer)).unwrap();
        assert!(!view.contains(&hidden_hand) && !view.contains(&secret));
    }
    choose(&mut g, vec![hidden_hand]);
    assert_eq!(g.players[0].hand.len(), 2);
    assert!(g.players[0].hand.iter().any(|c| c.definition == "JC084"));
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "BQ077"));
    no_black_trigger(&g); // Discarding the fish is not a deck-to-grave event.
    g.first_team = 1;
    assert_eq!(
        g.icons(g.board(&angel).unwrap().1, 0),
        Icons {
            combat: 1,
            influence: 1,
            ..Default::default()
        }
    );
    g.players[0].graveyard.truncate(6);
    assert_eq!(
        g.icons(g.board(&angel).unwrap().1, 0),
        Icons {
            influence: 1,
            ..Default::default()
        }
    );
    restore(&mut g);
}

#[test]
fn black_next20_cq26_only_own_deck_and_actual_character_with_decline_and_nonasset() {
    let mut g = game();
    field(&mut g, "CQ26", 0);
    field(&mut g, "CQ26", 2);
    g.players[0].deck = vec![g.make_card("JC125", 1)]; // foreign owner, own deck still qualifies
    g.players[2].deck = vec![g.make_card("XQ34", 2)]; // transaction alone does not
    let hand = held(&mut g, "JC125", 0);
    let angel_hand = held(&mut g, "CQ26", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(angel_hand),
            ..Action::new("asset")
        },
    );
    mill(&mut g, 0);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    assert!(g.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125"));
    choose(&mut g, vec!["accept".into()]);
    resolve(&mut g);
    choose(&mut g, vec![]);
    assert!(g.players[0].hand.iter().any(|c| c.id == hand));
    no_black_trigger(&g);
}

#[test]
fn black_next20_bq085_forced_prepare_orders_two_graves_and_cannot_skip_or_repeat() {
    let mut g = game();
    let pile = field(&mut g, "BQ085", 0);
    let a = grave(&mut g, "JC125", 0);
    let b = grave(&mut g, "JZ50", 0);
    let spare = grave(&mut g, "XQ34", 0);
    let old_deck = g.players[0].deck.clone();
    prepare_from_end(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.min, Some(1));
    let first = g.pending.clone().unwrap();
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(first.choice.id),
            selected: Some(vec![]),
            ..Action::new("choose")
        },
    );
    choose(&mut g, vec![pile.clone()]);
    resolve(&mut g);
    let p = g.pending.clone().unwrap();
    assert_eq!(
        (p.choice.kind.as_str(), p.choice.min, p.choice.max),
        ("bq085-bottom", Some(2), Some(2))
    );
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec![a.clone()]),
            ..Action::new("choose")
        },
    );
    choose(&mut g, vec![b.clone(), a.clone()]);
    assert_eq!(g.players[0].deck.len(), old_deck.len() + 2);
    assert_eq!(g.players[0].deck[old_deck.len()].definition, "JZ50");
    assert_eq!(g.players[0].deck[old_deck.len() + 1].definition, "JC125");
    assert_ne!(g.players[0].deck[old_deck.len()].id, b);
    assert_eq!(g.players[0].graveyard[0].id, spare);
    assert!(g.board(&pile).is_some());
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![a, b]),
            ..Action::new("choose")
        },
    );
}

#[test]
fn black_next20_bq085_prepare_insufficient_grave_sacrifices_real_source_without_partial_payment() {
    let mut g = game();
    let pile = field(&mut g, "BQ085", 0);
    let single = grave(&mut g, "JC125", 0);
    prepare_from_end(&mut g);
    choose(&mut g, vec![pile.clone()]);
    resolve(&mut g);
    assert!(g.board(&pile).is_none());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "BQ085" && c.id != pile));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == single));
    no_black_trigger(&g);
}

#[test]
fn black_next20_bq085_real_sacrifice_freezes_observer_region_and_returns_only_own_ghoul_exhausted()
{
    let mut g = game();
    let pile = field(&mut g, "BQ085", 0);
    let own = grave(&mut g, "JZ50", 0);
    let foreign = grave(&mut g, "JZ50", 2);
    let nonghoul = grave(&mut g, "JC125", 0);
    let sacrifice = field(&mut g, "JC125", 0);
    let victim = field(&mut g, "JC125", 2);
    butcher(&mut g, &sacrifice, &victim);
    let p = g.pending.clone().unwrap();
    assert!(p.choice.options.iter().any(|o| o.id == own));
    assert!(!p
        .choice
        .options
        .iter()
        .any(|o| o.id == foreign || o.id == nonghoul));
    choose(&mut g, vec![own.clone()]);
    // Moving the observer after declaration does not move the original event region.
    let i = g.regions[0]
        .cards
        .iter()
        .position(|c| c.id == pile)
        .unwrap();
    let c = g.regions[0].cards.remove(i);
    g.regions[2].cards.push(c);
    restore(&mut g);
    resolve(&mut g);
    let entered = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JZ50")
        .unwrap();
    assert!(entered.exhausted && !entered.face_down && entered.id != own);
    assert!(g.regions[2].cards.iter().all(|c| c.definition != "JZ50"));
    assert!(g.pending.is_none()); // placement is Enter, never the JZ50 Reveal search
}

#[test]
fn black_next20_bq085_ghoul_and_hidden_death_do_not_qualify() {
    for (id, hidden) in [("JZ50", false), ("JC125", true)] {
        let mut g = game();
        field(&mut g, "BQ085", 0);
        grave(&mut g, "JZ50", 0);
        let sacrifice = field(&mut g, id, 0);
        g.board_mut(&sacrifice).unwrap().face_down = hidden;
        let victim = field(&mut g, "JC125", 2);
        if hidden {
            fund(&mut g, 0, "XQ27", 4);
            let spell = held(&mut g, "XQ27", 0);
            let a = offered(&g, 0, "play", &spell, None, None);
            act(&mut g, 0, a);
            resolve(&mut g);
            assert!(g.board(&sacrifice).is_none());
        } else {
            butcher(&mut g, &sacrifice, &victim);
        }
        no_black_trigger(&g);
    }
}

#[test]
fn black_next20_xg27_enemy_death_pays_once_hidden_entry_at_victims_region_not_observers() {
    let mut g = game();
    let spirit = grave(&mut g, "XG27", 0);
    fund(&mut g, 0, "JC084", 1);
    let sacrifice = field(&mut g, "JC125", 0);
    let victim_card = g.make_card("JC125", 2);
    let victim = victim_card.id.clone();
    g.regions[2].cards.push(victim_card);
    butcher(&mut g, &sacrifice, &victim);
    assert!(g.pending.is_none());
    resolve(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    let pending_id = g.pending.as_ref().unwrap().choice.id.clone();
    choose(&mut g, vec!["accept".into()]);
    assert_eq!(g.resources(0), 0);
    assert_eq!(
        g.stack
            .last()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .source
            .region,
        None
    );
    resolve(&mut g);
    let entered = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "XG27")
        .unwrap();
    assert!(entered.face_down && entered.id != spirit && !entered.exhausted);
    assert!(!g.regions[0].cards.iter().any(|c| c.definition == "XG27"));
    for viewer in [1, 2, 3] {
        let view = g.view(viewer);
        let hidden = view.regions[2]
            .characters
            .iter()
            .find(|c| c.instance_id == entered.id)
            .unwrap();
        assert!(hidden.card_id.is_none() && hidden.face_down && hidden.current_domains.is_none());
    }
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(pending_id),
            selected: Some(vec!["accept".into()]),
            ..Action::new("choose")
        },
    );
}

#[test]
fn black_next20_xg27_requires_prior_grave_enemy_team_and_available_one_asset() {
    for (enemy, money, in_grave) in [(1, 1, true), (2, 0, true), (2, 1, false)] {
        let mut g = game();
        if in_grave {
            grave(&mut g, "XG27", 0);
        } else {
            held(&mut g, "XG27", 0);
        }
        fund(&mut g, 0, "JC084", money);
        let sacrifice = field(&mut g, "JC125", 0);
        let victim = field(&mut g, "JC125", enemy);
        butcher(&mut g, &sacrifice, &victim);
        resolve(&mut g);
        no_black_trigger(&g);
    }
}

#[test]
fn black_next20_xg27_frozen_victim_region_replacement_cannot_receive_hidden_recovery() {
    let mut g = game();
    grave(&mut g, "XG27", 0);
    fund(&mut g, 0, "JC084", 1);
    let sacrifice = field(&mut g, "JC125", 0);
    let victim = field(&mut g, "JC125", 2);
    butcher(&mut g, &sacrifice, &victim);
    resolve(&mut g);
    choose(&mut g, vec!["accept".into()]);
    g.regions[0].card = g.make_card("DQJC112", 0);
    restore(&mut g);
    resolve(&mut g);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XG27"));
    assert!(!g.regions[0].cards.iter().any(|c| c.definition == "XG27"));
    assert_eq!(g.resources(0), 0);
}

#[test]
fn black_next20_xg27_curse_discount_play_only_and_spirit_protection() {
    let mut g = game();
    field(&mut g, "XG27", 0);
    let victim = field(&mut g, "JC094", 2);
    fund(&mut g, 0, "JC084", 1);
    let curse = held(&mut g, "JC089", 0);
    let a = offered(&g, 0, "play", &curse, None, Some(&victim));
    assert_eq!(
        g.view(0)
            .hand
            .iter()
            .find(|c| c.instance_id == curse)
            .unwrap()
            .effective_cost,
        Some(1)
    );
    act(&mut g, 0, a);
    assert_eq!(g.resources(0), 0);
    resolve(&mut g);
    let hidden = field(&mut g, "BQ083", 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    fund(&mut g, 0, "JC084", 4);
    let reveal = offered(&g, 0, "reveal", &hidden, None, None);
    act(&mut g, 0, reveal);
    assert_eq!(g.resources(0), 0); // Curse play costs 3; paid reveal still costs all 4.
    assert!(
        matches!(g.stack.last().unwrap().frame.as_ref().unwrap().already_paid.as_slice(),
        [PaidCost::Assets(ids)] if ids.len() == 4)
    );
    resolve(&mut g);
    choose(&mut g, vec![]); // Decline the skull's ordinary entry destruction.
    let mut g = game();
    let spirit = field(&mut g, "XG27", 0);
    assert!(g.spirit_protected(g.board(&spirit).unwrap().1));
    field(&mut g, "JZ50", 2);
    assert!(!g.spirit_protected(g.board(&spirit).unwrap().1));
    restore(&mut g);
}

#[test]
fn black_next20_xq39_society_host_black_discount_consumption_and_paid_reveal() {
    let mut g = game();
    let seal = mark(&mut g, 0);
    ability(&mut g, 0, &seal, "black20-xq39-discount", None);
    assert_eq!(g.modifiers.len(), 1);
    assert!(g.modifiers[0].paid_reveal);
    let nonblack = held(&mut g, "JC125", 0);
    deploy(&mut g, 0, &nonblack, 0);
    assert_eq!(g.modifiers[0].uses, 1);
    let black = held(&mut g, "BQ077", 0);
    deploy(&mut g, 0, &black, 0);
    assert_eq!(g.modifiers[0].uses, 0);
    g.board_mut(&seal).unwrap().exhausted = false;
    ability(&mut g, 0, &seal, "black20-xq39-discount", None);
    let hidden = field(&mut g, "BQ077", 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    let a = offered(&g, 0, "reveal", &hidden, None, None);
    assert_eq!(g.resources(0), 0);
    act(&mut g, 0, a);
    assert!(
        matches!(g.stack.last().unwrap().frame.as_ref().unwrap().already_paid.as_slice(),
        [PaidCost::Assets(ids)] if ids.is_empty())
    );
    resolve(&mut g);
    assert!(g.modifiers.iter().all(|m| m.uses == 0));
    assert!(g.in_play(&seal).unwrap().1.exhausted);
}

#[test]
fn black_next20_xq39_two_assets_target_personal_character_to_top_no_shuffle_and_old_target_fizzles()
{
    let mut g = game();
    let seal = mark(&mut g, 0);
    let own = grave(&mut g, "JZ50", 0);
    let foreign = grave(&mut g, "JC125", 2);
    let noncharacter = grave(&mut g, "XQ34", 0);
    fund(&mut g, 0, "JC084", 2);
    let before = g.players[0].deck.clone();
    let actions = g.legal_actions(0);
    assert!(actions
        .iter()
        .any(|a| a.action.card_id.as_deref() == Some(&seal)
            && a.action.target_id.as_deref() == Some(&own)));
    assert!(!actions
        .iter()
        .any(|a| a.action.card_id.as_deref() == Some(&seal)
            && [Some(foreign.as_str()), Some(noncharacter.as_str())]
                .contains(&a.action.target_id.as_deref())));
    ability(&mut g, 0, &seal, "black20-xq39-recover", Some(&own));
    assert_eq!(g.players[0].deck[0].definition, "JZ50");
    assert_ne!(g.players[0].deck[0].id, own);
    assert_eq!(
        serde_json::to_value(&g.players[0].deck[1..]).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    assert_eq!(g.resources(0), 0);
    g.board_mut(&seal).unwrap().exhausted = false;
    fund(&mut g, 0, "JC084", 2);
    let old = grave(&mut g, "JZ50", 0);
    let a = offered(
        &g,
        0,
        "activate",
        &seal,
        Some("black20-xq39-recover"),
        Some(&old),
    );
    act(&mut g, 0, a);
    let i = g.players[0]
        .graveyard
        .iter()
        .position(|c| c.id == old)
        .unwrap();
    let c = g.players[0].graveyard.remove(i);
    let c = g.reset_zone_card(c);
    let new = c.id.clone();
    g.players[0].graveyard.push(c);
    restore(&mut g);
    resolve(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == new));
}

#[test]
fn black_next20_restore_rejects_foreign_batch_duplicate_trigger_and_death_region_forgery() {
    let mut g = game();
    field(&mut g, "CQ26", 0);
    held(&mut g, "JC125", 0);
    deck_cards(&mut g, 0, &["JC125", "JC125", "JC125", "JC125", "JC125"]);
    mill(&mut g, 0);
    let valid_room = crate::room::RoomEnvelope::from_game(g.clone());
    assert!(crate::room::RoomEnvelope::from_persisted(
        &serde_json::to_string(&valid_room).unwrap()
    )
    .is_ok());
    let mut bad = g.clone();
    if let ChoiceResolution::Declare { declaration: d, .. } =
        &mut bad.pending.as_mut().unwrap().resolution
    {
        if let Op::BlackDeckToGraveyard {
            observation: Some(o),
        } = &mut d.ability.ops[0]
        {
            o.deck_seat = 1;
        }
    }
    rejects_saved_or_live_black_state(bad);
    let mut bad = g.clone();
    if let ChoiceResolution::Declare { declaration, .. } = &bad.pending.as_ref().unwrap().resolution
    {
        bad.effects.push_back(Effect::Declare {
            declaration: declaration.clone(),
        });
    }
    rejects_saved_or_live_black_state(bad);
    let mut g = game();
    grave(&mut g, "XG27", 0);
    fund(&mut g, 0, "JC084", 1);
    let sacrifice = field(&mut g, "JC125", 0);
    let victim = field(&mut g, "JC125", 2);
    butcher(&mut g, &sacrifice, &victim);
    resolve(&mut g);
    let mut bad = g.clone();
    if let ChoiceResolution::Declare { declaration: d, .. } =
        &mut bad.pending.as_mut().unwrap().resolution
    {
        if let Op::BlackDeathRecovery {
            observation: Some(o),
        } = &mut d.ability.ops[0]
        {
            o.region = 2;
        }
    }
    rejects_saved_or_live_black_state(bad);
}

#[test]
fn black_next20_all_four_characters_keep_granted_glory_original_region_and_restore() {
    for id in ["BQ077", "BQ085", "CQ26", "XG27"] {
        let mut g = game();
        let source = field(&mut g, id, 0);
        let protector = field(&mut g, "LC12", 0);
        ability(
            &mut g,
            0,
            &protector,
            "protect-local-character",
            Some(&source),
        );
        fund(&mut g, 0, "JC084", 2);
        let curse = held(&mut g, "JC089", 0);
        let a = offered(&g, 0, "play", &curse, None, Some(&source));
        act(&mut g, 0, a);
        resolve(&mut g);
        let region_id = g.regions[0].card.id.clone();
        g.begin_window(Window::Before(0, 1));
        for _ in 0..8 {
            if g.pending.is_some() {
                break;
            }
            pass(&mut g);
        }
        let p = g.pending.as_ref().unwrap();
        let ChoiceResolution::Declare { declaration, .. } = &p.resolution else {
            panic!("glory trigger");
        };
        assert_eq!(declaration.ability.key, "jc089-combat-glory");
        assert_eq!(
            declaration.source.source_region_instance.as_deref(),
            Some(region_id.as_str())
        );
        choose(&mut g, vec!["accept".into()]);
        resolve(&mut g);
        assert_eq!(g.regions[0].influence[0], 1, "{id}");
    }
}

#[test]
fn black_next20_simultaneous_deaths_keep_pile_snapshot_but_do_not_retroactively_arm_spirit() {
    let mut g = game();
    let pile = field(&mut g, "BQ085", 0);
    let spirit = field(&mut g, "XG27", 0);
    let victim = field(&mut g, "JC125", 2);
    let ghoul = grave(&mut g, "JZ50", 0);
    let urd = field(&mut g, "WM034", 0);
    g.board_mut(&urd).unwrap().time_markers = 1;
    fund(&mut g, 0, "JC084", 1);
    let a = offered(&g, 0, "activate", &urd, Some("batch20-wm034-destroy"), None);
    act(&mut g, 0, a);
    resolve(&mut g);
    assert!(g.board(&pile).is_none() && g.board(&spirit).is_none() && g.board(&victim).is_none());
    let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution
    else {
        panic!("death observer");
    };
    assert_eq!(declaration.source.card.id, pile);
    assert_eq!(declaration.ability.key, "black20-bq085-return");
    choose(&mut g, vec![ghoul]);
    resolve(&mut g);
    assert!(g.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "JZ50" && c.exhausted));
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XG27"));
    no_black_trigger(&g);
}

#[test]
fn black_next20_cq26_jz50_public_reveal_search_also_observes_one_actual_batch() {
    let mut g = game();
    field(&mut g, "CQ26", 0);
    let ghoul = field(&mut g, "JZ50", 0);
    g.board_mut(&ghoul).unwrap().face_down = true;
    fund(&mut g, 0, "JC084", 2);
    let in_deck = deck_cards(&mut g, 0, &["XG27", "JC125"])[0].clone();
    held(&mut g, "JC125", 0);
    let a = offered(&g, 0, "reveal", &ghoul, None, None);
    act(&mut g, 0, a);
    resolve(&mut g);
    choose(&mut g, vec!["accept".into()]);
    resolve(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "jz50_death_search");
    choose(&mut g, vec![in_deck]);
    let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution
    else {
        panic!("angel observer");
    };
    assert_eq!(declaration.ability.key, "black20-cq26-loot");
    choose(&mut g, vec![]);
    no_black_trigger(&g);
}

#[test]
fn black_next20_bq085_uses_last_controller_for_death_and_actor_personal_grave_not_owners() {
    let mut g = game();
    let pile = field(&mut g, "BQ085", 2);
    g.board_mut(&pile).unwrap().controller = 0;
    let sacrifice = field(&mut g, "JC125", 2);
    g.board_mut(&sacrifice).unwrap().controller = 0;
    let target = field(&mut g, "JC125", 2);
    let ghoul = grave(&mut g, "JZ50", 0);
    grave(&mut g, "JZ50", 2);
    butcher(&mut g, &sacrifice, &target);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125"));
    let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution
    else {
        panic!("pile trigger");
    };
    assert_eq!(
        (
            declaration.source.card.owner,
            declaration.source.card.controller
        ),
        (2, 0)
    );
    choose(&mut g, vec![ghoul]);
    resolve(&mut g);
    let restored = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JZ50")
        .unwrap();
    assert_eq!(
        (restored.owner, restored.controller, restored.exhausted),
        (0, 0, true)
    );
}
