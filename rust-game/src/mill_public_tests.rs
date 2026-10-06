//! Explicit offline layouts, real commands and restored four-seat projections.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, deck, model::*, rules::*};

fn initial(actor: usize) -> Game {
    let mut g = game(actor);
    for p in &mut g.players {
        p.deck.clear();
    }
    g
}
fn held(g: &mut Game, id: &str, owner: usize) -> String {
    let c = g.make_card(id, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn deploy(g: &mut Game, actor: usize, id: &str) {
    let a = g
        .legal_actions(actor)
        .into_iter()
        .find(|a| {
            a.action.kind == "deploy"
                && a.action.card_id.as_deref() == Some(id)
                && a.action.region == Some(2)
        })
        .unwrap()
        .action;
    apply(g, actor, a);
    pass_top(g);
}
fn play(g: &mut Game, actor: usize, id: &str, target: &str) {
    let a = g.legal_actions(actor).into_iter().find(|a| a.action.kind == "play"
        && a.action.card_id.as_deref() == Some(id) && a.action.target_id.as_deref() == Some(target)).unwrap_or_else(|| panic!("offered play missing: actor={actor} resources={} card={} target={target} legal={:?}", g.resources(actor), g.players[actor].hand.iter().find(|c|c.id==id).unwrap().definition, g.legal_actions(actor))).action;
    apply(g, actor, a);
    pass_top(g);
}
fn accept(g: &mut Game) {
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
    choose(g, vec!["accept".into()]);
}
fn fixture(kind: &str, g: &Game) {
    if let Ok(dir) = std::env::var("MILL_PUBLIC_FRONTEND_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(
            format!("{dir}/{kind}.json"),
            serde_json::to_vec(&serde_json::json!({
                "kind":kind,"state":serde_json::to_string(&r).unwrap(),
                "views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()
            }))
            .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn mill_public_whole_source_fields_and_ordinary_copy_limits() {
    let mill = catalog::card("XQ36");
    let corpse = catalog::card("XQ46");
    assert_eq!(
        (&*mill.name, &*mill.color, mill.cost, mill.defense),
        ("圣甲虫的清理员", "黑", 1, Some(1))
    );
    assert_eq!(mill.loyalty, ["黑色"]);
    assert_eq!(mill.subtypes, ["人类", "雇员"]);
    assert_eq!(
        mill.permanent_icons,
        Icons {
            influence: 1,
            ..Default::default()
        }
    );
    assert_eq!(mill.temporary_icons, Icons::default());
    assert!(mill.magic.is_empty() && mill.keywords.is_empty());
    assert_eq!(mill.abilities.len(), 1);
    assert!(mill.abilities[0].triggered);
    assert_eq!(
        (&*corpse.name, &*corpse.color, corpse.cost, corpse.defense),
        ("无名尸体", "中立", 0, Some(0))
    );
    assert_eq!(corpse.subtypes, ["人类"]);
    assert!(corpse.loyalty.is_empty() && corpse.magic.is_empty());
    assert_eq!(corpse.permanent_icons, Icons::default());
    assert_eq!(corpse.temporary_icons, Icons::default());
    assert_eq!(corpse.keywords, ["公开"]);
    assert!(corpse.rule_traits.public && corpse.abilities.is_empty());
    for c in [mill, corpse] {
        assert!(!c.unique);
        assert_eq!(deck::copy_limit(c), Some(3));
    }
    assert_eq!(catalog::catalog().cards.len(), 100);
    assert_eq!(catalog::catalog().societies.len(), 8);
}

#[test]
fn mill_public_real_entry_mills_all_four_seats_empty_and_short_decks_without_draw_or_death() {
    for actor in 0..4 {
        for count in [0, 1, 3, 4, 5] {
            let mut g = initial(actor);
            fund(&mut g, actor, "JC084", 1);
            let mut before = vec![];
            for seat in 0..4 {
                for n in 0..count {
                    let c = g.make_card(if n == 4 { "JC032" } else { "JZ61" }, seat);
                    g.players[seat].deck.push(c);
                }
                before.push(g.players[seat].deck.clone());
            }
            let hand = held(&mut g, "XQ36", actor);
            deploy(&mut g, actor, &hand);
            assert_eq!(g.pending.as_ref().unwrap().seat, actor);
            assert_eq!(g.pending.as_ref().unwrap().choice.min, Some(0));
            fixture(&format!("entry-choice-{actor}-{count}"), &g);
            accept(&mut g);
            pass_top(&mut g);
            for seat in 0..4 {
                let p = &g.players[seat];
                assert!(!p.eliminated);
                assert_eq!(p.graveyard.len(), count.min(4));
                assert_eq!(
                    serde_json::to_value(&p.deck).unwrap(),
                    serde_json::to_value(&before[seat][count.min(4)..]).unwrap()
                );
                assert!(p.hand.is_empty());
                if count == 5 {
                    for viewer in 0..4 {
                        assert!(!serde_json::to_string(&g.view(viewer))
                            .unwrap()
                            .contains(&before[seat][4].id));
                    }
                }
                for (old, new) in before[seat].iter().zip(&p.graveyard) {
                    assert_ne!(old.id, new.id);
                    assert_eq!(old.definition, new.definition);
                    assert_eq!((new.owner, new.controller), (seat, seat));
                    assert!(!new.face_down && !new.exhausted);
                    assert_eq!((new.damage, new.wounds, new.shield), (0, 0, 0));
                }
            }
            assert!(g.pending.is_none() && g.stack.is_empty() && g.effects.is_empty());
            assert_eq!(g.status, "playing"); // Empty library is not failed Draw; JZ61 did not die on board.
            fixture(&format!("entry-result-{actor}-{count}"), &g);
            checkpoint(&g);
        }
    }
}

#[test]
fn mill_public_optional_decline_and_paid_reveal_trigger() {
    let mut g = initial(0);
    fund(&mut g, 0, "JC084", 3);
    let card = held(&mut g, "XQ36", 0);
    let top = g.make_card("JC032", 0);
    let top_id = top.id.clone();
    g.players[0].deck.push(top);
    deploy(&mut g, 0, &card);
    choose(&mut g, vec![]);
    assert_eq!(g.players[0].deck[0].id, top_id);
    assert!(g.players[0].graveyard.is_empty());
    let hidden = held(&mut g, "XQ36", 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
    assert!(g.pending.is_none());
    assert_eq!(g.players[0].deck[0].id, top_id);
    let back = g.regions[2]
        .cards
        .iter()
        .find(|c| c.face_down && c.definition == "XQ36")
        .unwrap()
        .id
        .clone();
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(back.clone()),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut g);
    assert!(g.board(&back).is_none());
    accept(&mut g);
    pass_top(&mut g);
    assert!(g.players[0].deck.is_empty());
    assert_eq!(g.players[0].graveyard[0].definition, "JC032");
    checkpoint(&g);
}

#[test]
fn mill_public_source_departure_does_not_cancel_declared_mill_and_foreign_owner_is_preserved() {
    let mut g = initial(0);
    fund(&mut g, 0, "JC084", 4);
    let top = g.make_card("JC032", 2);
    let old = top.id.clone();
    g.players[0].deck.push(top);
    let card = held(&mut g, "XQ36", 0);
    deploy(&mut g, 0, &card);
    accept(&mut g);
    let live = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "XQ36")
        .unwrap()
        .id
        .clone();
    let destroy = held(&mut g, "JC091", 0);
    play(&mut g, 0, &destroy, &live);
    assert!(g.board(&live).is_none());
    assert_eq!(g.stack.len(), 1);
    pass_top(&mut g);
    assert!(g.players[0].deck.is_empty());
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC032" && c.id != old && c.owner == 2 && c.controller == 2));
    assert!(!g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC032"));
    checkpoint(&g);
}

#[test]
fn mill_public_make_frame_freezes_living_seats_at_declaration_not_each_step() {
    let mut g = initial(0);
    g.players[1].eliminated = true;
    let c = g.make_card("XQ36", 0);
    let source = g.source_snapshot(&c, Some(2));
    let frame = g.make_frame(
        0,
        source,
        &definition("XQ36").abilities[0],
        vec![],
        vec![],
        None,
    );
    assert_eq!(
        frame.steps.iter().map(|s| s.context).collect::<Vec<_>>(),
        [0, 2, 3]
    );
    g.players[1].eliminated = false;
    g.players[2].eliminated = true;
    assert_eq!(
        frame.steps.iter().map(|s| s.context).collect::<Vec<_>>(),
        [0, 2, 3]
    );
}

#[test]
fn mill_public_corpse_zero_defense_dies_before_another_action_public_reject_is_atomic() {
    let mut g = initial(0);
    let id = held(&mut g, "XQ46", 0);
    assert!(!g
        .legal_actions(0)
        .iter()
        .any(|a| a.action.kind == "conceal" && a.action.card_id.as_deref() == Some(&id)));
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id.clone()),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
    deploy(&mut g, 0, &id);
    assert!(!g
        .regions
        .iter()
        .flat_map(|r| &r.cards)
        .any(|c| c.definition == "XQ46"));
    let dead = g.players[0]
        .graveyard
        .iter()
        .find(|c| c.definition == "XQ46")
        .unwrap();
    assert_ne!(dead.id, id);
    assert!(g.pending.is_none());
    fixture("corpse-zero-result", &g);
    checkpoint(&g);
}

#[test]
fn mill_public_corpse_live_aura_same_name_enemy_region_and_real_departure_chain() {
    for actor in 0..4 {
        let mut g = initial(actor);
        let friend = if actor % 2 == 0 { actor + 1 } else { actor - 1 };
        let enemy = (actor + 2) % 4;
        let aura = board(&mut g, "JC059", friend, 2);
        board(&mut g, "JC059", enemy, 2);
        board(&mut g, "JC059", actor, 1);
        let hand = held(&mut g, "XQ46", actor);
        deploy(&mut g, actor, &hand);
        let live = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "XQ46")
            .unwrap()
            .id
            .clone();
        assert_eq!(g.defense(g.board(&live).unwrap().1, 2), 1);
        fixture(&format!("corpse-aura-{actor}"), &g);
        fund(&mut g, actor, "JC084", 3);
        let destroy = held(&mut g, "JC091", actor);
        play(&mut g, actor, &destroy, &aura);
        assert!(g.board(&aura).is_none() && g.board(&live).is_none());
        assert!(g.players[actor]
            .graveyard
            .iter()
            .any(|c| c.definition == "XQ46" && c.id != live));
        assert!(g.players[friend]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC059"));
        checkpoint(&g);
    }
}

#[test]
fn mill_public_corpse_effect_hide_and_asset_are_allowed_by_public() {
    let mut g = initial(0);
    board(&mut g, "JC059", 0, 2);
    fund(&mut g, 0, "JC056", 2);
    let hand = held(&mut g, "XQ46", 0);
    deploy(&mut g, 0, &hand);
    let live = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "XQ46")
        .unwrap()
        .id
        .clone();
    let hide = held(&mut g, "JC063", 0);
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(&hide)
                && a.action.option.as_deref() == Some("hide")
                && a.action.target_id.as_deref() == Some(&live)
        })
        .unwrap()
        .action;
    apply(&mut g, 0, a);
    pass_top(&mut g);
    let back = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "XQ46")
        .unwrap();
    assert!(back.face_down);
    assert_ne!(back.id, live);
    fixture("corpse-forced-hidden", &g);
    let asset = held(&mut g, "XQ46", 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(asset.clone()),
            ..Action::new("asset")
        },
    );
    assert!(g.players[0]
        .assets
        .iter()
        .any(|c| c.definition == "XQ46" && c.id != asset));
    checkpoint(&g);
}

#[test]
fn mill_public_real_control_and_sacrifice_use_controller_but_owner_graveyard() {
    let mut g = initial(0);
    board(&mut g, "JC059", 0, 2);
    board(&mut g, "JC059", 2, 2);
    let corpse = board(&mut g, "XQ46", 2, 2);
    let other = board(&mut g, "XQ46", 2, 2);
    assert_ne!(corpse, other);
    fund(&mut g, 0, "JC104", 5);
    fund(&mut g, 0, "JC042", 2);
    let spell = held(&mut g, "JC049", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(spell.clone()),
            cost_selected: Some(vec![corpse.clone()]),
            ..Action::new("play")
        },
    );
    let steal = held(&mut g, "JC129", 0);
    play(&mut g, 0, &steal, &corpse);
    assert_eq!(
        (
            g.board(&corpse).unwrap().1.owner,
            g.board(&corpse).unwrap().1.controller
        ),
        (2, 0)
    );
    assert_eq!(g.board(&other).unwrap().1.controller, 2);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(spell.clone()),
            cost_selected: Some(vec![other.clone()]),
            ..Action::new("play")
        },
    );
    let mut a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| a.action.kind == "play" && a.action.card_id.as_deref() == Some(&spell))
        .unwrap()
        .action;
    a.cost_selected = Some(vec![corpse.clone()]);
    apply(&mut g, 0, a);
    assert!(g.board(&corpse).is_none());
    let dead = g.players[2]
        .graveyard
        .iter()
        .find(|c| c.definition == "XQ46")
        .unwrap();
    assert_ne!(dead.id, corpse);
    assert_eq!(dead.controller, 2);
    assert!(!g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ46"));
    pass_top(&mut g);
    checkpoint(&g);
}
