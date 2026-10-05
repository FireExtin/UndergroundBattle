//! Explicit starting layouts; declarations, costs, responses and restoration use real actions.
use crate::{catalog, model::*, rules::*};

fn game() -> Game {
    let mut g = Game::new(
        "defence-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        8,
    )
    .unwrap();
    for s in 1..4 {
        g.join(format!("P{s}"), "watchers".into()).unwrap();
    }
    for p in &mut g.players {
        p.ready = true;
    }
    g.apply(0, Action::new("start")).unwrap();
    while g.pending.is_some() {
        choose(&mut g, vec![]);
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
        p.graveyard.clear();
    }
    for r in &mut g.regions {
        r.cards.clear();
        r.influence = [0; 2];
    }
    g.regions[0].card = g.make_card("DQJC115", 0);
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    g
}
fn field(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn held(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn fund(g: &mut Game, seat: usize, id: &str, n: usize) {
    for _ in 0..n {
        let c = g.make_card(id, seat);
        g.players[seat].assets.push(c);
    }
}
fn restore(g: &mut Game) {
    let state = serde_json::to_string(g).unwrap();
    let views = (0..4)
        .map(|s| serde_json::to_value(g.view(s)).unwrap())
        .collect::<Vec<_>>();
    *g = Game::from_persisted(&state).unwrap();
    assert_eq!(serde_json::to_string(g).unwrap(), state);
    for s in 0..4 {
        assert_eq!(serde_json::to_value(g.view(s)).unwrap(), views[s]);
    }
}
fn act(g: &mut Game, s: usize, a: Action) {
    restore(g);
    g.apply(s, a).unwrap();
    restore(g);
}
fn reject(g: &mut Game, s: usize, a: Action) {
    let before = serde_json::to_string(g).unwrap();
    assert!(g.apply(s, a).is_err());
    assert_eq!(serde_json::to_string(g).unwrap(), before);
    restore(g);
}
fn choose(g: &mut Game, ids: Vec<String>) {
    let p = g.pending.clone().unwrap();
    act(
        g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(ids),
            ..Action::new("choose")
        },
    );
}
fn pass(g: &mut Game) {
    let s = (0..4)
        .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
        .unwrap();
    act(g, s, Action::new("pass"));
}
fn top(g: &mut Game) {
    let n = g.stack.len();
    assert!(n > 0);
    for _ in 0..64 {
        if g.pending.is_some() {
            resolve_choice(g);
        } else if g.stack.len() < n {
            return;
        } else {
            pass(g);
        }
    }
    panic!("bounded stack");
}
fn resolve_choice(g: &mut Game) {
    let p = g.pending.clone().unwrap();
    if matches!(
        p.resolution,
        ChoiceResolution::Forecast { .. }
            | ChoiceResolution::Frame {
                choice: FrameChoice::Forecast { .. },
                ..
            }
    ) {
        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect();
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                top: Some(ids),
                bottom: Some(vec![]),
                ..Action::new("choose")
            },
        );
    } else if matches!(p.resolution, ChoiceResolution::Bottom { .. }) {
        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect();
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                bottom: Some(ids),
                ..Action::new("choose")
            },
        );
    } else if matches!(p.resolution, ChoiceResolution::Damage { .. }) {
        let mut allocations = std::collections::BTreeMap::new();
        allocations.insert(p.choice.options[0].id.clone(), p.choice.amount.unwrap());
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                allocations: Some(allocations),
                ..Action::new("choose")
            },
        );
    } else {
        let ids = p
            .choice
            .options
            .iter()
            .take(p.choice.min.unwrap_or(0))
            .map(|o| o.id.clone())
            .collect();
        choose(g, ids);
    }
}
fn play_card(g: &mut Game, seat: usize, id: &str) {
    let kind = if g.players[seat]
        .hand
        .iter()
        .find(|c| c.id == id)
        .is_some_and(|c| catalog::card(&c.definition).kind == "character")
    {
        "deploy"
    } else {
        "play"
    };
    act(
        g,
        seat,
        Action {
            card_id: Some(id.into()),
            region: Some(0),
            ..Action::new(kind)
        },
    );
}
fn until_choice(g: &mut Game, kind: &str) {
    for _ in 0..80 {
        if g.pending.as_ref().is_some_and(|p| p.choice.kind == kind) {
            return;
        }
        if g.pending.is_some() {
            resolve_choice(g);
        } else {
            pass(g);
        }
    }
    panic!("missing {kind}");
}
fn dog_action(id: &str, victim: Option<&str>) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some("sacrifice-draw".into()),
        cost_selected: victim.map(|v| vec![v.into()]),
        ..Action::new("activate")
    }
}
fn deck_cards(g: &mut Game, seat: usize, ids: &[&str]) -> Vec<String> {
    g.players[seat].deck.clear();
    let cards = ids
        .iter()
        .map(|id| g.make_card(id, seat))
        .collect::<Vec<_>>();
    let out = cards.iter().map(|c| c.id.clone()).collect();
    g.players[seat].deck = cards;
    out
}
fn funds(g: &mut Game, seat: usize) {
    for id in ["JC075", "JC014", "JC084", "JC042"] {
        fund(g, seat, id, 4);
    }
}
#[test]
fn hand_deck_original_fields_and_finite_programs() {
    for (id, cost, loyalty, color, magic) in [
        ("JC130", 2, vec!["白色"], "中立", ""),
        ("JC131", 3, vec!["绿色", "绿色"], "中立", ""),
        ("JC096", 4, vec!["黑色", "黑色"], "黑", "鲜血"),
        ("XQ17", 1, vec!["红色"], "红", ""),
    ] {
        let d = catalog::card(id);
        assert_eq!(
            (
                d.cost,
                d.loyalty.clone(),
                d.color.as_str(),
                d.magic.as_str()
            ),
            (
                cost,
                loyalty.iter().map(|s| s.to_string()).collect(),
                color,
                magic
            )
        );
        assert_eq!(d.unique, id == "JC096");
        assert!(definition(id)
            .abilities
            .iter()
            .all(|a| a.targets.is_empty()));
    }
    let dog = catalog::card("JC096");
    assert_eq!(dog.subtitle.as_deref(), Some("墨菲斯托的化身"));
    assert_eq!(
        dog.permanent_icons,
        Icons {
            combat: 2,
            influence: 1,
            investigation: 0
        }
    );
    assert_eq!(dog.temporary_icons.investigation, 1);
    assert_eq!(catalog::card("XQ17").temporary_icons.influence, 1);
    assert!(matches!(
        definition("JC131").abilities[0].ops[0],
        Op::Search {
            visibility: SearchVisibility::Private,
            optional: false,
            ..
        }
    ));
    assert_eq!(definition("XQ17").abilities[0].event, Some(Event::Death));
}
#[test]
fn hand_deck_all_four_pay_printed_costs_with_actual_declarations() {
    for (id, cost) in [("JC130", 2), ("JC131", 3), ("JC096", 4), ("XQ17", 1)] {
        let mut g = game();
        funds(&mut g, 0);
        deck_cards(&mut g, 0, &["LC01", "JC125", "JC003"]);
        let id0 = held(&mut g, id, 0);
        let before = g.resources(0);
        assert!(g
            .legal_actions(0)
            .iter()
            .any(|a| (a.action.kind == "play" || a.action.kind == "deploy")
                && a.action.card_id.as_deref() == Some(&id0)));
        play_card(&mut g, 0, &id0);
        assert_eq!(g.resources(0), before - cost);
        top(&mut g);
        let new = if catalog::card(id).kind == "character" {
            g.board(&id0)
                .map(|(_, c)| c)
                .or_else(|| {
                    g.regions
                        .iter()
                        .flat_map(|r| &r.cards)
                        .find(|c| c.definition == id)
                })
                .unwrap()
        } else {
            g.players[0]
                .graveyard
                .iter()
                .find(|c| c.definition == id)
                .unwrap()
        };
        assert_ne!(new.id, id0);
    }
}
#[test]
fn hand_deck_printed_loyalty_rejects_atomically() {
    for (id, assets) in [
        ("JC130", vec!["JC084"; 8]),
        ("JC131", vec!["JC014", "JC075", "JC075", "JC075"]),
        ("JC096", vec!["JC084", "JC042", "JC075", "JC075"]),
        ("XQ17", vec!["JC075"; 8]),
    ] {
        let mut g = game();
        for asset in assets {
            fund(&mut g, 0, asset, 1);
        }
        let source = held(&mut g, id, 0);
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(source),
                region: Some(0),
                ..Action::new(if catalog::card(id).kind == "character" {
                    "deploy"
                } else {
                    "play"
                })
            },
        );
    }
}
#[test]
fn hand_deck_research_draws_two_before_exact_private_discard_and_fresh_ids() {
    let mut g = game();
    funds(&mut g, 0);
    let old = deck_cards(&mut g, 0, &["LC01", "JC003", "JC125"]);
    let s = held(&mut g, "JC130", 0);
    play_card(&mut g, 0, &s);
    until_choice(&mut g, "discard");
    assert_eq!(g.players[0].deck.len(), 1);
    assert_eq!(g.players[0].hand.len(), 2);
    assert!(g.players[0].hand.iter().all(|c| !old.contains(&c.id)));
    let p = g.pending.clone().unwrap();
    assert_eq!((p.seat, p.choice.min, p.choice.max), (0, Some(1), Some(1)));
    for seat in 1..4 {
        assert!(g.view(seat).pending_choice.is_none());
    }
    let id = g.players[0].hand[0].id.clone();
    for selected in [vec![], vec![id.clone(), id.clone()], vec![old[0].clone()]] {
        reject(
            &mut g,
            0,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(selected),
                ..Action::new("choose")
            },
        );
    }
    reject(
        &mut g,
        1,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec![id.clone()]),
            ..Action::new("choose")
        },
    );
    choose(&mut g, vec![id.clone()]);
    assert_eq!(g.players[0].hand.len(), 1);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "LC01" && c.id != id));
}
#[test]
fn hand_deck_airdrop_private_search_all_cards_and_deterministic_shuffle() {
    let mut g = game();
    funds(&mut g, 0);
    let old = deck_cards(&mut g, 0, &["LC01", "JC003", "JC125", "JC130"]);
    let s = held(&mut g, "JC131", 0);
    play_card(&mut g, 0, &s);
    until_choice(&mut g, "search");
    let p = g.pending.clone().unwrap();
    assert_eq!(p.choice.options.len(), 4);
    assert_eq!(p.choice.min, Some(1));
    for seat in 1..4 {
        let v = serde_json::to_string(&g.view(seat)).unwrap();
        assert!(g.view(seat).pending_choice.is_none());
        assert!(!v.contains("LC01") && !v.contains("西比尔"));
    }
    let before = g.random;
    choose(&mut g, vec![old[0].clone()]);
    assert_ne!(g.random, before);
    assert_eq!(g.players[0].deck.len(), 3);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].hand[0].definition, "LC01");
    assert_ne!(g.players[0].hand[0].id, old[0]);
    assert!(!g
        .log
        .iter()
        .any(|e| e.text.contains("西比尔") || e.text.contains("展示检索")));
}
#[test]
fn hand_deck_empty_search_does_not_draw_or_eliminate() {
    let mut g = game();
    funds(&mut g, 0);
    deck_cards(&mut g, 0, &[]);
    let s = held(&mut g, "JC131", 0);
    play_card(&mut g, 0, &s);
    top(&mut g);
    assert!(!g.players[0].eliminated);
    assert!(g.pending.is_none());
    assert!(g.players[0].hand.is_empty());
}
#[test]
fn hand_deck_research_exhausting_library_uses_existing_elimination() {
    let mut g = game();
    funds(&mut g, 0);
    deck_cards(&mut g, 0, &["JC125"]);
    let s = held(&mut g, "JC130", 0);
    play_card(&mut g, 0, &s);
    top(&mut g);
    assert!(g.players[0].eliminated);
    assert!(g.players[0].hand.is_empty());
    assert!(g.pending.is_none());
}
#[test]
fn hand_deck_dog_requires_each_cost_and_only_controlled_faceup_character() {
    for invalid in [
        "missing",
        "teammate",
        "enemy",
        "hidden",
        "asset",
        "duplicate",
        "stale",
        "exhausted",
        "no-assets",
    ] {
        let mut g = game();
        funds(&mut g, 0);
        let dog = field(&mut g, "JC096", 0);
        let own = field(&mut g, "JC125", 0);
        let teammate = field(&mut g, "JC125", 1);
        let enemy = field(&mut g, "JC125", 2);
        let hidden = field(&mut g, "JC125", 0);
        g.board_mut(&hidden).unwrap().face_down = true;
        let mut a = dog_action(&dog, Some(&own));
        match invalid {
            "missing" => a.cost_selected = None,
            "teammate" => a.cost_selected = Some(vec![teammate]),
            "enemy" => a.cost_selected = Some(vec![enemy]),
            "hidden" => a.cost_selected = Some(vec![hidden]),
            "asset" => a.cost_selected = Some(vec![g.players[0].assets[0].id.clone()]),
            "duplicate" => a.cost_selected = Some(vec![own.clone(), own]),
            "stale" => a.cost_selected = Some(vec!["old-instance".into()]),
            "exhausted" => g.board_mut(&dog).unwrap().exhausted = true,
            "no-assets" => g.players[0].assets.clear(),
            _ => unreachable!(),
        }
        reject(&mut g, 0, a);
    }
}
#[test]
fn hand_deck_dog_self_sacrifice_preserves_paid_source_snapshot_and_draws() {
    let mut g = game();
    funds(&mut g, 0);
    let old = deck_cards(&mut g, 0, &["LC01", "JC125"]);
    let dog = field(&mut g, "JC096", 0);
    let before = g.resources(0);
    act(&mut g, 0, dog_action(&dog, Some(&dog)));
    assert_eq!(g.resources(0), before - 1);
    assert!(g.board(&dog).is_none());
    let frame = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert_eq!(frame.source.card.id, dog);
    assert_eq!(frame.actor, 0);
    assert_eq!(frame.source.region, Some(0));
    assert_eq!(frame.already_paid.len(), 3);
    top(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_ne!(g.players[0].hand[0].id, old[0]);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC096" && c.id != dog));
}
#[test]
fn hand_deck_dog_can_sacrifice_nonowned_controlled_character() {
    let mut g = game();
    funds(&mut g, 0);
    deck_cards(&mut g, 0, &["JC125", "JC125"]);
    let dog = field(&mut g, "JC096", 0);
    let victim = field(&mut g, "JC125", 2);
    g.board_mut(&victim).unwrap().controller = 0;
    act(&mut g, 0, dog_action(&dog, Some(&victim)));
    assert!(g.board(&victim).is_none());
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125" && c.id != victim));
    top(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
    assert!(g.board(&dog).unwrap().1.exhausted);
}
#[test]
fn hand_deck_offering_death_uses_last_controller_not_owner_and_optional_trigger() {
    for accept in [false, true] {
        let mut g = game();
        funds(&mut g, 0);
        deck_cards(&mut g, 0, &["LC01", "JC125", "JC125"]);
        let dog = field(&mut g, "JC096", 0);
        let offering = field(&mut g, "XQ17", 2);
        g.board_mut(&offering).unwrap().controller = 0;
        act(&mut g, 0, dog_action(&dog, Some(&offering)));
        until_choice(&mut g, "trigger");
        let p = g.pending.clone().unwrap();
        assert_eq!(p.seat, 0);
        let ChoiceResolution::Declare { declaration, .. } = &p.resolution else {
            panic!("death declaration")
        };
        assert_eq!(declaration.source.card.id, offering);
        assert_eq!(
            (
                declaration.actor,
                declaration.source.card.owner,
                declaration.source.card.controller,
                declaration.source.region
            ),
            (0, 2, 0, Some(0))
        );
        choose(
            &mut g,
            if accept {
                vec!["accept".into()]
            } else {
                vec![]
            },
        );
        for _ in 0..80 {
            if g.stack.is_empty() && g.pending.is_none() {
                break;
            }
            if g.pending.is_some() {
                resolve_choice(&mut g);
            } else {
                pass(&mut g);
            }
        }
        assert_eq!(g.players[0].hand.len(), 1);
        assert_eq!(g.players[0].deck.len(), if accept { 1 } else { 2 });
        assert!(g.players[2]
            .graveyard
            .iter()
            .any(|c| c.definition == "XQ17" && c.id != offering));
    }
}
#[test]
fn hand_deck_offering_hidden_removal_and_return_do_not_trigger_death() {
    for hidden in [false, true] {
        let mut g = game();
        let c = field(&mut g, "XQ17", 0);
        g.board_mut(&c).unwrap().face_down = hidden;
        if hidden {
            g.remove_dead(&c, RemovalCause::Destroy);
        } else {
            g.return_hand(&c);
        }
        assert!(!g.effects.iter().any(|e|matches!(e,Effect::Declare{declaration} if declaration.source.card.definition=="XQ17")));
        restore(&mut g);
    }
}

#[test]
fn hand_deck_dog_printed_domain_is_an_attribute_not_extra_loyalty() {
    let mut g = game();
    fund(&mut g, 0, "JC084", 4);
    let source = held(&mut g, "JC096", 0);
    assert!(g.loyalty(0, "JC096"));
    play_card(&mut g, 0, &source);
    top(&mut g);
    assert!(g
        .regions
        .iter()
        .flat_map(|r| &r.cards)
        .any(|c| c.definition == "JC096"));
}

#[test]
fn hand_deck_dog_returned_in_real_response_draws_for_original_actor_and_instance() {
    let mut g = game();
    funds(&mut g, 0);
    deck_cards(&mut g, 0, &["LC01", "JC125"]);
    let dog = field(&mut g, "JC096", 2);
    g.board_mut(&dog).unwrap().controller = 0;
    let victim = field(&mut g, "JC125", 0);
    let rescuer = field(&mut g, "JC075", 0);
    act(&mut g, 0, dog_action(&dog, Some(&victim)));
    act(
        &mut g,
        0,
        Action {
            card_id: Some(rescuer),
            ability_id: Some("rescue".into()),
            target_id: Some(dog.clone()),
            ..Action::new("activate")
        },
    );
    top(&mut g);
    assert!(g.board(&dog).is_none());
    let returned = g.players[2]
        .hand
        .iter()
        .find(|c| c.definition == "JC096")
        .unwrap();
    assert_ne!(returned.id, dog);
    assert_eq!(returned.controller, 2);
    let f = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert_eq!(
        (f.actor, f.source.card.controller, f.source.card.owner),
        (0, 0, 2)
    );
    assert_eq!(f.source.card.id, dog);
    top(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[2].hand.len(), 1);
}
