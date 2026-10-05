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
fn play_target(g: &mut Game, id: &str, target: &str) {
    act(
        g,
        0,
        Action {
            card_id: Some(id.into()),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    );
}
fn resources(g: &Game, s: usize) -> usize {
    g.players[s].assets.iter().filter(|c| !c.exhausted).count()
}
fn drain(g: &mut Game) {
    for _ in 0..100 {
        if g.stack.is_empty() && g.pending.is_none() {
            return;
        }
        if g.pending.is_some() {
            resolve_choice(g);
        } else {
            pass(g);
        }
    }
    panic!("bounded drain");
}
fn painter(g: &mut Game) -> String {
    let id = field(g, "JC103", 0);
    act(
        g,
        0,
        Action {
            card_id: Some(id.clone()),
            ability_id: Some("reduce-next-purple".into()),
            ..Action::new("activate")
        },
    );
    drain(g);
    id
}
fn purple_funds(g: &mut Game, n: usize) {
    fund(g, 0, "JC104", n);
}
fn collapse(g: &mut Game, target: &str) -> String {
    let id = held(g, "JC107", 0);
    act(
        g,
        0,
        Action {
            card_id: Some(id.clone()),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    );
    id
}
fn asset(g: &mut Game, id: &str, s: usize) -> String {
    fund(g, s, id, 1);
    g.players[s].assets.last().unwrap().id.clone()
}
#[test]
fn purple_tools_printed_fields_and_finite_target_programs() {
    for (id, cost, loyalty, kind) in [
        ("JC103", 3, vec!["紫色"], "character"),
        ("JC107", 3, vec!["紫色", "紫色"], "spell"),
    ] {
        let d = catalog::card(id);
        assert_eq!(
            (
                d.cost,
                d.loyalty.clone(),
                d.kind.as_str(),
                d.magic.as_str(),
                d.unique
            ),
            (
                cost,
                loyalty.iter().map(|s| s.to_string()).collect(),
                kind,
                "",
                false
            )
        );
        for a in &definition(id).abilities {
            validate_ability(id, a).unwrap();
        }
    }
    let p = catalog::card("JC103");
    assert_eq!(
        p.permanent_icons,
        Icons {
            investigation: 1,
            combat: 0,
            influence: 1
        }
    );
    assert_eq!(
        p.temporary_icons,
        Icons {
            investigation: 0,
            combat: 1,
            influence: 0
        }
    );
    assert_eq!(p.defense, Some(1));
    assert_eq!(catalog::card("JC107").subtypes, vec!["事务", "灾难"]);
    let mut bad = definition("JC107").abilities[0].clone();
    bad.targets[0].zone = Zone::Board;
    assert!(validate_ability("JC107", &bad).is_err());
    assert!(validate_ability("JC103", &definition("JC107").abilities[0]).is_err());
}
#[test]
fn resource_actions_real_room_declarations_resolve_without_response_windows() {
    use crate::room::{RoomCommand, RoomEnvelope, SessionAction};
    for (definition, ability) in [
        ("JC103", "reduce-next-purple"),
        ("JC112", "reduce-next-magic"),
    ] {
        let mut g = game();
        purple_funds(&mut g, 4);
        let source = field(&mut g, definition, 0);
        let priority = g.priority_team;
        let window = g.window.clone();
        let room = RoomEnvelope::from_game(g);
        let command = RoomCommand {
            command_id: format!("resource-policy-{definition}"),
            expected_version: room.revision,
            action: SessionAction::Game {
                action: Action {
                    card_id: Some(source.clone()),
                    ability_id: Some(ability.into()),
                    ..Action::new("activate")
                },
            },
        };
        let transition = room.transition(0, Some(command), 1_000).unwrap();
        assert_eq!(transition.outcome, "accepted");
        let restored = RoomEnvelope::from_persisted(&transition.state).unwrap();
        assert!(
            restored.stack.is_empty(),
            "{definition} must resolve immediately"
        );
        assert!(restored.pending.is_none());
        assert!(restored.effects.is_empty());
        assert_eq!(restored.priority_team, priority);
        assert_eq!(restored.game.window, window);
        assert_eq!(restored.modifiers.len(), 1);
        assert_eq!(restored.modifiers[0].uses, 1);
        assert_eq!(resources(&restored, 0), 4);
        if definition == "JC103" {
            assert!(restored.board(&source).unwrap().1.exhausted);
        } else {
            assert!(restored.board(&source).is_none());
            assert_eq!(restored.players[0].graveyard.len(), 1);
            assert_ne!(restored.players[0].graveyard[0].id, source);
        }
        assert_eq!(serde_json::to_string(&restored).unwrap(), transition.state);
        for seat in 0..4 {
            let view = restored.view(seat, 1_000);
            assert!(view.response_window.is_none());
            assert!(view.stack.is_empty());
            let again = RoomEnvelope::from_persisted(&transition.state).unwrap();
            assert_eq!(
                serde_json::to_value(again.view(seat, 1_000)).unwrap(),
                serde_json::to_value(view).unwrap()
            );
        }
    }
}
#[test]
fn purple_tools_painter_exhaust_only_then_discount_collapse_asset_without_character_death() {
    let mut g = game();
    purple_funds(&mut g, 4);
    let p = field(&mut g, "JC103", 0);
    let victim = asset(&mut g, "XQ17", 2);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(p.clone()),
            ..Action::new("activate")
        },
    );
    assert_eq!(resources(&g, 0), 4);
    assert!(g.board(&p).unwrap().1.exhausted);
    assert_eq!(g.modifiers.len(), 1);
    assert!(g.stack.is_empty());
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(p),
            ..Action::new("activate")
        },
    );
    drain(&mut g);
    assert!(g.modifiers[0].paid_reveal);
    let spell = collapse(&mut g, &victim);
    assert_eq!(resources(&g, 0), 2);
    assert_eq!(g.modifiers[0].uses, 0);
    while !g.stack.is_empty() {
        assert!(g.pending.is_none());
        pass(&mut g);
    }
    assert!(g.players[2].assets.is_empty());
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ17" && c.id != victim && !c.exhausted && c.controller == 2));
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC107" && c.id != spell));
    assert!(!g.log.iter().any(|l| l.text.contains("祭品：死亡")));
}
#[test]
fn purple_tools_painter_secret_asset_free_and_paid_reveal_scopes() {
    let mut g = game();
    purple_funds(&mut g, 4);
    painter(&mut g);
    let heldid = held(&mut g, "JC104", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(heldid),
            region: Some(0),
            ..Action::new("conceal")
        },
    );
    assert_eq!(resources(&g, 0), 3);
    assert_eq!(g.modifiers[0].uses, 1);
    let hidden = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC104")
        .unwrap()
        .id
        .clone();
    let assetid = held(&mut g, "JC125", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(assetid),
            ..Action::new("asset")
        },
    );
    assert_eq!(g.modifiers[0].uses, 1);
    let before = resources(&g, 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    assert_eq!(resources(&g, 0), before - 1);
    assert_eq!(g.modifiers[0].uses, 0);
    drain(&mut g);
    let mut g = game();
    purple_funds(&mut g, 2);
    painter(&mut g);
    let id = field(&mut g, "JC104", 0);
    g.board_mut(&id).unwrap().face_down = true;
    let before = resources(&g, 0);
    g.world_reveal(0, &id, true).unwrap();
    restore(&mut g);
    assert_eq!(resources(&g, 0), before);
    assert_eq!(g.modifiers[0].uses, 1);
}
#[test]
fn purple_tools_legacy_reduction_still_excludes_paid_reveal() {
    let mut g = game();
    purple_funds(&mut g, 4);
    let cat = field(&mut g, "JC112", 0);
    let hidden = field(&mut g, "JC104", 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    act(
        &mut g,
        0,
        Action {
            card_id: Some(cat),
            ..Action::new("activate")
        },
    );
    assert!(!g.modifiers[0].paid_reveal);
    assert!(!serde_json::to_string(&g.modifiers[0])
        .unwrap()
        .contains("paid_reveal"));
    act(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    assert_eq!(resources(&g, 0), 2);
    assert_eq!(g.modifiers[0].uses, 1);
    drain(&mut g);
}
#[test]
fn purple_tools_painter_nonpurple_other_actor_loyalty_and_declaration_failure_do_not_consume() {
    let mut g = game();
    purple_funds(&mut g, 4);
    painter(&mut g);
    let nonpurple = held(&mut g, "JC125", 0);
    play_card(&mut g, 0, &nonpurple);
    drain(&mut g);
    assert_eq!(g.modifiers[0].uses, 1);
    let p = held(&mut g, "JC107", 0);
    let target = asset(&mut g, "JC125", 2);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(p.clone()),
            target_id: Some("missing".into()),
            ..Action::new("play")
        },
    );
    assert_eq!(g.modifiers[0].uses, 1);
    for c in &mut g.players[0].assets {
        c.definition = "JC075".into();
    }
    restore(&mut g);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(p),
            target_id: Some(target),
            ..Action::new("play")
        },
    );
    assert_eq!(g.modifiers[0].uses, 1);
    let mut g = game();
    purple_funds(&mut g, 4);
    painter(&mut g);
    fund(&mut g, 2, "JC104", 2);
    let id = held(&mut g, "JC104", 2);
    g.begin_window(Window::Action(g.team(2)));
    let before = resources(&g, 2);
    play_card(&mut g, 2, &id);
    assert_eq!(resources(&g, 2), before - 2);
    assert_eq!(g.modifiers[0].uses, 1);
}
#[test]
fn purple_tools_painter_stacking_caps_zero_and_consumes_matching_next() {
    let mut g = game();
    purple_funds(&mut g, 2);
    painter(&mut g);
    painter(&mut g);
    let id = held(&mut g, "JZ67", 0);
    play_target(&mut g, &id, "p2");
    assert_eq!(resources(&g, 0), 2);
    assert!(g.modifiers.iter().all(|m| m.uses == 0));
    drain(&mut g);
}
#[test]
fn purple_tools_painter_source_return_keeps_original_actor_and_turn_end_clears() {
    let mut g = game();
    purple_funds(&mut g, 4);
    let p = field(&mut g, "JC103", 2);
    g.board_mut(&p).unwrap().controller = 0;
    let rescue = field(&mut g, "JC075", 0);
    fund(&mut g, 0, "JC075", 1);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(p.clone()),
            ..Action::new("activate")
        },
    );
    act(
        &mut g,
        0,
        Action {
            card_id: Some(rescue),
            target_id: Some(p.clone()),
            ..Action::new("activate")
        },
    );
    drain(&mut g);
    assert!(g.board(&p).is_none());
    assert!(g.players[2]
        .hand
        .iter()
        .any(|c| c.definition == "JC103" && c.id != p));
    assert_eq!(g.modifiers[0].actor, 0);
    let turn = g.turn;
    for _ in 0..400 {
        if g.turn > turn {
            break;
        }
        if g.pending.is_some() {
            resolve_choice(&mut g);
        } else {
            pass(&mut g);
        }
    }
    assert!(g.turn > turn);
    assert!(g.modifiers.is_empty());
}
#[test]
fn purple_tools_collapse_assets_all_seats_owner_identity_and_target_summaries() {
    for s in 0..4 {
        let mut g = game();
        purple_funds(&mut g, 3);
        let victim = asset(&mut g, "JC125", s);
        if s == 2 {
            g.players[2].assets.last_mut().unwrap().owner = 3;
            g.players[2].assets.last_mut().unwrap().controller = 2;
        }
        let owner = if s == 2 { 3 } else { s };
        let id = held(&mut g, "JC107", 0);
        let a = g
            .legal_actions(0)
            .into_iter()
            .find(|a| {
                a.action.card_id.as_deref() == Some(&id)
                    && a.action.target_id.as_deref() == Some(&victim)
            })
            .unwrap();
        assert!(a.label.contains("资产"));
        assert!(!a.label.contains("无知路人"));
        act(&mut g, 0, a.action);
        for viewer in 0..4 {
            let view = g.view(viewer);
            let t = &view.stack.last().unwrap().target_summaries[0];
            assert_eq!(t.kind, "asset");
            assert_eq!(t.label, "资产");
            assert_eq!(t.owner.as_deref(), Some(format!("p{owner}").as_str()));
            assert!(t.region.is_none());
        }
        drain(&mut g);
        assert!(g.players[owner]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC125" && c.id != victim && c.controller == owner));
        assert!(!g.players[s].assets.iter().any(|c| c.id == victim));
    }
}
#[test]
fn purple_tools_collapse_rejects_other_zones_atomically_and_pays_loyalty() {
    let mut g = game();
    purple_funds(&mut g, 3);
    let spell = held(&mut g, "JC107", 0);
    let character = field(&mut g, "JC125", 2);
    let hidden = field(&mut g, "JC125", 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    let heldcard = held(&mut g, "JC125", 2);
    let c = g.make_card("JC125", 2);
    let grave = c.id.clone();
    g.players[2].graveyard.push(c);
    for target in [
        character,
        hidden,
        heldcard,
        grave,
        "region:0".into(),
        "p2".into(),
        "missing".into(),
    ] {
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(spell.clone()),
                target_id: Some(target),
                ..Action::new("play")
            },
        );
    }
    let target = asset(&mut g, "JC125", 2);
    g.players[0].assets.pop();
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(spell),
            target_id: Some(target),
            ..Action::new("play")
        },
    );
}
#[test]
fn purple_tools_collapse_control_attachment_restores_host_and_wounded_armor_host_dies() {
    let mut g = game();
    purple_funds(&mut g, 3);
    fund(&mut g, 2, "JC036", 5);
    let host = field(&mut g, "JC125", 0);
    let leash = held(&mut g, "JC036", 2);
    g.begin_window(Window::Action(g.team(2)));
    act(
        &mut g,
        2,
        Action {
            card_id: Some(leash),
            target_id: Some(host.clone()),
            ..Action::new("play")
        },
    );
    drain(&mut g);
    assert_eq!(g.board(&host).unwrap().1.controller, 2);
    let attachment = g.attachments[0].card.id.clone();
    g.begin_window(Window::Action(0));
    collapse(&mut g, &attachment);
    drain(&mut g);
    assert_eq!(g.board(&host).unwrap().1.controller, 0);
    assert!(g.attachments.is_empty());
    assert!(g.control_effects.is_empty());
    let mut g = game();
    purple_funds(&mut g, 3);
    let host = field(&mut g, "XQ17", 2);
    let vest = g.make_card("XQ47", 2);
    let attachment = vest.id.clone();
    g.attachments.push(Attachment {
        card: vest,
        host_id: host.clone(),
    });
    g.board_mut(&host).unwrap().damage = 1;
    restore(&mut g);
    collapse(&mut g, &attachment);
    until_choice(&mut g, "trigger");
    assert!(g.board(&host).is_none());
    let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution
    else {
        panic!()
    };
    assert_eq!(declaration.source.card.id, host);
    assert_eq!(declaration.actor, 2);
    choose(&mut g, vec![]);
    drain(&mut g);
}
#[test]
fn purple_tools_collapse_stale_attachment_real_host_return_keeps_paid_cost() {
    let mut g = game();
    purple_funds(&mut g, 3);
    let host = field(&mut g, "JC125", 2);
    let rescue = field(&mut g, "JC075", 2);
    let vest = g.make_card("XQ47", 2);
    let old = vest.id.clone();
    g.attachments.push(Attachment {
        card: vest,
        host_id: host.clone(),
    });
    fund(&mut g, 2, "JC075", 2);
    collapse(&mut g, &old);
    while !g.can_fast(2) {
        pass(&mut g);
    }
    act(
        &mut g,
        2,
        Action {
            card_id: Some(rescue),
            ability_id: Some("rescue".into()),
            target_id: Some(host.clone()),
            ..Action::new("activate")
        },
    );
    drain(&mut g);
    assert_eq!(resources(&g, 0), 0);
    assert!(g.attachments.is_empty());
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ47" && c.id != old));
    assert!(g.players[2]
        .hand
        .iter()
        .any(|c| c.definition == "JC125" && c.id != host));
}
