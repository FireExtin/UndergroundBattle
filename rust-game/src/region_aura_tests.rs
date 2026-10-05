//! Actual XQ43/MSJC08. Boundary fixtures will be explicit; no new fake cards.
use crate::{catalog, model::*, rules::*};

#[test]
fn xq43_whole_original_and_exact_region_binding_admitted() {
    let c = catalog::catalog()
        .cards
        .iter()
        .find(|c| c.id == "XQ43")
        .expect("actual XQ43 must be admitted with its whole printed program");
    assert_eq!(
        (&*c.name, &*c.color, &*c.kind),
        ("具现化", "紫", "attachment")
    );
    assert!(c.unique);
    assert_eq!(c.cost, 2);
    assert_eq!(c.loyalty, ["紫色"]);
    assert_eq!(c.magic, "星辰");
    assert_eq!(c.subtypes, ["结界"]);
    assert_eq!(c.defense, None);
    assert!(c.keywords.is_empty());
    assert_eq!(c.permanent_icons, crate::model::Icons::default());
    assert_eq!(c.temporary_icons, crate::model::Icons::default());
    let d = definition("XQ43");
    let host = &d
        .attachment
        .as_ref()
        .expect("actual region attachment")
        .host;
    assert_eq!(host.zone, Zone::Region);
    assert_eq!(host.kind, EntityKind::Any);
    assert_eq!(host.relation, Relation::Any);
    assert_eq!(d.abilities.len(), 1);
    assert!(d.abilities[0].play_only);
    assert_eq!(d.abilities[0].timing, Timing::Standard);
    assert!(d.abilities[0].ops.is_empty());
}

#[test]
fn msjc08_whole_original_reuses_paid_private_purple_unique_search() {
    let c = catalog::catalog()
        .societies
        .iter()
        .find(|s| s.card.id == "MSJC08")
        .expect("actual MSJC08 must be admitted after true purple capacity exists");
    assert_eq!(
        (&*c.card.name, &*c.subtitle, &*c.card.color),
        ("梦境行者", "幻梦呓语", "紫")
    );
    assert_eq!(c.card.subtypes, ["群体", "梦境"]);
    assert_eq!(c.starting_hand, 6);
    let search = definition("MSJC08")
        .abilities
        .iter()
        .find(|a| a.key == "search-purple-unique")
        .unwrap();
    assert!(search.once_per_game);
    assert_eq!(search.costs.len(), 2);
    assert!(
        matches!(search.ops.as_slice(), [Op::Search { player: PlayerRef::Actor, filter: CardFilter::PrintedColorAndUnique { color }, to_top: false, optional: false, visibility: SearchVisibility::Reveal }] if color == "紫")
    );
}

fn initial() -> Game {
    let mut g = Game::new(
        "region-aura-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        9,
    )
    .unwrap();
    for s in 1..4 {
        g.join(format!("P{s}"), "watchers".into()).unwrap();
    }
    for s in 0..4 {
        g.apply(s, Action::new("ready")).unwrap();
    }
    g.apply(0, Action::new("start")).unwrap();
    while g.pending.is_some() {
        choose(&mut g, vec![]);
    }
    for s in 0..4 {
        g.players[s].hand.clear();
        g.players[s].assets.clear();
        g.players[s].deck.clear();
        for _ in 0..12 {
            let c = g.make_card("JC125", s);
            g.players[s].deck.push(c);
        }
    }
    for r in &mut g.regions {
        r.cards.clear();
        r.influence = [0, 0];
    }
    g.regions[0].card = g.make_card("DQJC115", 0);
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    fund(&mut g, 0, "JC091", 6);
    fund(&mut g, 0, "JC104", 6);
    fund(&mut g, 0, "JC073", 6);
    g
}
fn fund(g: &mut Game, s: usize, definition: &str, count: usize) {
    for _ in 0..count {
        let c = g.make_card(definition, s);
        g.players[s].assets.push(c);
    }
}
fn field(g: &mut Game, definition: &str, owner: usize, controller: usize) -> String {
    let mut c = g.make_card(definition, owner);
    c.controller = controller;
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn held(g: &mut Game, definition: &str, s: usize) -> String {
    let c = g.make_card(definition, s);
    let id = c.id.clone();
    g.players[s].hand.push(c);
    id
}
fn restore(g: &mut Game) {
    let raw = serde_json::to_string(g).unwrap();
    let views = (0..4)
        .map(|s| serde_json::to_value(g.view(s)).unwrap())
        .collect::<Vec<_>>();
    *g = Game::from_persisted(&raw).unwrap();
    assert_eq!(serde_json::to_string(g).unwrap(), raw);
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
    let raw = serde_json::to_string(g).unwrap();
    assert!(g.apply(s, a).is_err());
    assert_eq!(serde_json::to_string(g).unwrap(), raw);
    restore(g);
}
fn choose(g: &mut Game, selected: Vec<String>) {
    let p = g.pending.clone().unwrap();
    act(
        g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(selected),
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

fn drain(g: &mut Game) {
    for _ in 0..120 {
        if g.stack.is_empty() && g.pending.is_none() {
            return;
        }
        if let Some(p) = g.pending.clone() {
            let ids = p
                .choice
                .options
                .iter()
                .take(p.choice.min.unwrap_or(0))
                .map(|o| o.id.clone())
                .collect();
            choose(g, ids);
        } else {
            pass(g);
        }
    }
    panic!("bounded aura resolution");
}
fn play_region(g: &mut Game, seat: usize, region: usize) -> (String, String) {
    let old = held(g, "XQ43", seat);
    act(
        g,
        seat,
        Action {
            card_id: Some(old.clone()),
            region: Some(region),
            ..Action::new("play")
        },
    );
    drain(g);
    let fresh = g
        .attachments
        .iter()
        .find(|a| a.card.controller == seat && a.card.definition == "XQ43")
        .unwrap()
        .card
        .id
        .clone();
    (old, fresh)
}
fn play_target(g: &mut Game, definition: &str, target: &str) {
    let id = held(g, definition, 0);
    act(
        g,
        0,
        Action {
            card_id: Some(id),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    );
    drain(g);
}
fn icon(g: &Game, id: &str) -> Icons {
    let (r, c) = g.board(id).unwrap();
    g.current_icons(c, r)
}
#[test]
fn xq43_paid_region_play_is_exact_fresh_public_and_all_spirits_ignore_initiative() {
    let mut g = initial();
    let enemy = field(&mut g, "JZ58", 2, 2);
    let friend = field(&mut g, "JZ58", 1, 1);
    let human = field(&mut g, "JC049", 2, 2);
    assert_eq!(icon(&g, &enemy), Icons::default());
    let human_before = icon(&g, &human);
    let resources = g.resources(0);
    let region_id = g.regions[0].card.id.clone();
    let (old, fresh) = play_region(&mut g, 0, 0);
    assert_ne!(old, fresh);
    assert_eq!(g.resources(0), resources - 2);
    assert_eq!(g.attachments[0].host_id, region_id);
    assert!(g.board(&region_id).is_none());
    assert_eq!(icon(&g, &enemy).influence, 1);
    assert_eq!(icon(&g, &friend).influence, 1);
    assert_eq!(icon(&g, &human), human_before);
    for first in [0, 1] {
        g.first_team = first;
        restore(&mut g);
        assert_eq!(icon(&g, &enemy).influence, 1);
        assert_eq!(icon(&g, &friend).influence, 1);
        for viewer in 0..4 {
            let v = g.view(viewer);
            assert_eq!(v.attachments[0].host_id, region_id);
            assert_eq!(v.attachments[0].card.card_id.as_deref(), Some("XQ43"));
            let c = v.regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == enemy)
                .unwrap();
            assert_eq!(c.converted_temporary_icons.unwrap().influence, 1);
        }
    }
}
#[test]
fn xq43_rejects_missing_wrong_zone_out_of_range_insufficient_cost_and_loyalty_atomically() {
    let mut g = initial();
    let id = held(&mut g, "XQ43", 0);
    let target = g.regions[0].card.id.clone();
    for a in [
        Action {
            card_id: Some(id.clone()),
            ..Action::new("play")
        },
        Action {
            card_id: Some(id.clone()),
            target_id: Some(target),
            ..Action::new("play")
        },
        Action {
            card_id: Some(id.clone()),
            region: Some(99),
            ..Action::new("play")
        },
    ] {
        reject(&mut g, 0, a);
    }
    g.players[0].assets.clear();
    fund(&mut g, 0, "JC125", 3);
    let action = Action {
        card_id: Some(id.clone()),
        region: Some(0),
        ..Action::new("play")
    };
    reject(&mut g, 0, action.clone());
    g.players[0].assets.clear();
    fund(&mut g, 0, "JC104", 1);
    reject(&mut g, 0, action);
}
#[test]
fn xq43_bound_region_replacement_cancels_without_refund_and_reports_missing() {
    let mut g = initial();
    let id = held(&mut g, "XQ43", 0);
    let resources = g.resources(0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(0),
            ..Action::new("play")
        },
    );
    let frame = g.stack.last().unwrap().frame.clone().unwrap();
    assert_eq!(
        frame.targets[0].region_instance.as_ref(),
        Some(&g.regions[0].card.id)
    );
    g.regions[0].card = g.make_card("DQJC115", 0);
    restore(&mut g);
    assert!(!g.valid_bound_target(0, &frame.source, &frame.targets[0]));
    assert_eq!(
        g.target_summary(&frame, &frame.targets[0]).status,
        "missing"
    );
    drain(&mut g);
    assert!(g.attachments.is_empty());
    assert_eq!(g.resources(0), resources - 2);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ43"));
}
#[test]
fn xq43_real_attachment_temporary_grants_and_ordinary_turn_bonus_are_distinct() {
    let mut g = initial();
    let spirit = field(&mut g, "JZ58", 0, 0);
    play_target(&mut g, "JC073", &spirit);
    play_target(&mut g, "JC093", &spirit);
    let blade = g
        .attachments
        .iter()
        .find(|a| a.card.definition == "JC093")
        .unwrap()
        .card
        .id
        .clone();
    let sacrifice = field(&mut g, "JC125", 0, 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(blade),
            ability_id: Some("sacrifice-character-host-icons".into()),
            cost_selected: Some(vec![sacrifice]),
            ..Action::new("activate")
        },
    );
    drain(&mut g);
    g.first_team = 1;
    assert_eq!(
        icon(&g, &spirit),
        Icons {
            investigation: 0,
            combat: 1,
            influence: 1
        }
    );
    play_region(&mut g, 0, 0);
    assert_eq!(
        icon(&g, &spirit),
        Icons {
            investigation: 1,
            combat: 1,
            influence: 2
        }
    );
    assert_eq!(
        g.converted_temporary_icons(g.board(&spirit).unwrap().1, 0),
        Some(Icons {
            investigation: 1,
            combat: 0,
            influence: 1
        })
    );
    let aura = g
        .attachments
        .iter()
        .find(|a| a.card.definition == "XQ43")
        .unwrap()
        .card
        .id
        .clone();
    play_target(&mut g, "JC107", &aura);
    assert_eq!(
        icon(&g, &spirit),
        Icons {
            investigation: 0,
            combat: 1,
            influence: 1
        }
    );
}
#[test]
fn xq43_pure_vector_all_three_convert_once_ordinary_component_unchanged() {
    // Synthetic query vectors only: no fake definition or natural combat-grant claim.
    let p = Icons {
        investigation: 2,
        combat: 3,
        influence: 4,
    };
    let t = Icons {
        investigation: 5,
        combat: 6,
        influence: 7,
    };
    let ordinary = Icons {
        investigation: 8,
        combat: 9,
        influence: 10,
    };
    for initiative in [true, false] {
        assert_eq!(
            Game::combine_character_icons(p, t, ordinary, initiative, true),
            p.add(t).add(ordinary)
        );
        assert_eq!(
            Game::combine_character_icons(p.add(t), Icons::default(), ordinary, initiative, true),
            p.add(t).add(ordinary)
        );
    }
    assert_eq!(
        Game::combine_character_icons(p, t, ordinary, false, false),
        p.add(ordinary)
    );
}
#[test]
fn xq43_moving_character_enters_and_leaves_exact_region_with_equipment() {
    let mut g = initial();
    g.first_team = 1;
    let spirit = field(&mut g, "JZ58", 0, 0);
    play_target(&mut g, "JC073", &spirit);
    play_region(&mut g, 0, 0);
    assert_eq!(icon(&g, &spirit).investigation, 1);
    // No admitted spirit has mobility. Exercise the existing movement primitive
    // with an explicit synthetic frame, without changing any printed definition.
    move_primitive(&mut g, &spirit, 1);
    assert_eq!(g.board(&spirit).unwrap().0, 1);
    assert_eq!(icon(&g, &spirit), Icons::default());
    assert_eq!(
        g.view(0)
            .attachments
            .iter()
            .find(|a| a.card.card_id.as_deref() == Some("JC073"))
            .unwrap()
            .card
            .region,
        Some(1)
    );
    move_primitive(&mut g, &spirit, 0);
    assert_eq!(
        icon(&g, &spirit),
        Icons {
            investigation: 1,
            combat: 0,
            influence: 1
        }
    );
}
#[test]
fn xq43_multiple_controller_auras_do_not_stack_and_one_departure_keeps_conversion() {
    let mut g = initial();
    let spirit = field(&mut g, "JZ58", 2, 2);
    let (_, first) = play_region(&mut g, 0, 0);
    fund(&mut g, 2, "JC104", 4);
    g.begin_window(Window::Action(1));
    play_region(&mut g, 2, 0);
    assert_eq!(g.attachments.len(), 2);
    assert_eq!(icon(&g, &spirit).influence, 1);
    g.begin_window(Window::Action(0));
    play_target(&mut g, "JC107", &first);
    assert_eq!(g.attachments.len(), 1);
    assert_eq!(icon(&g, &spirit).influence, 1);
}
#[test]
fn xq43_same_controller_uniqueness_uses_public_in_play_attachment_cards() {
    let mut g = initial();
    let spirit = field(&mut g, "JZ58", 2, 2);
    let (_, first) = play_region(&mut g, 0, 0);
    let second = held(&mut g, "XQ43", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(second),
            region: Some(0),
            ..Action::new("play")
        },
    );
    for _ in 0..40 {
        if g.pending.is_some() {
            break;
        }
        pass(&mut g);
    }
    let p = g.pending.as_ref().unwrap();
    assert!(matches!(p.resolution, ChoiceResolution::Unique));
    assert_eq!(p.choice.options.len(), 2);
    assert!(p.choice.options.iter().any(|o| o.id == first));
    choose(&mut g, vec![first]);
    drain(&mut g);
    assert_eq!(g.attachments.len(), 1);
    assert_eq!(icon(&g, &spirit).influence, 1);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ43"));
}
#[test]
fn xq43_as_asset_retains_domain_but_has_no_region_conversion_or_unique_conflict() {
    let mut g = initial();
    let spirit = field(&mut g, "JZ58", 2, 2);
    let id = held(&mut g, "XQ43", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            ..Action::new("asset")
        },
    );
    assert_eq!(icon(&g, &spirit), Icons::default());
    assert!(g.attachments.is_empty());
    let asset = g.view(0).assets.last().unwrap().clone();
    assert_eq!(
        (asset.color.as_deref(), asset.magic.as_deref()),
        (Some("紫"), Some("星辰"))
    );
    assert!(asset.card_id.is_none() && asset.converted_temporary_icons.is_none());
    play_region(&mut g, 0, 0);
    assert!(g.pending.is_none());
    assert_eq!(g.attachments.len(), 1);
}
#[test]
fn xq43_region_host_qualifies_jc005_only_with_real_mind_asset_domain() {
    let mut g = initial();
    let (_, aura) = play_region(&mut g, 0, 0);
    let id = held(&mut g, "JC005", 0);
    let action = Action {
        card_id: Some(id),
        target_id: Some(aura.clone()),
        ..Action::new("play")
    };
    reject(&mut g, 0, action.clone());
    fund(&mut g, 0, "JC003", 1);
    act(&mut g, 0, action);
    drain(&mut g);
    assert!(g.board(&aura).is_none());
}
#[test]
fn xq43_actual_win_batch_returns_region_attachment_to_owner_before_replacing_region() {
    let mut g = initial();
    let (_, aura) = play_region(&mut g, 0, 0);
    let old_region = g.regions[0].card.id.clone();
    for s in 0..4 {
        field(&mut g, "JC125", s, s);
        field(&mut g, "JC049", s, s);
    }
    let old_decks = g.players.iter().map(|p| p.deck.len()).collect::<Vec<_>>();
    g.begin_window(Window::Win(0, 0));
    for _ in 0..4 {
        pass(&mut g);
    }
    let mut expected = vec![vec![]; 4];
    while g.region_return.is_some() {
        let p = g.pending.clone().unwrap();
        assert!(matches!(p.resolution, ChoiceResolution::Bottom { .. }));
        let ids = p
            .choice
            .options
            .iter()
            .rev()
            .map(|o| o.id.clone())
            .collect::<Vec<_>>();
        expected[p.seat] = ids
            .iter()
            .map(|id| g.board(id).unwrap().1.definition.clone())
            .collect::<Vec<_>>();
        if p.seat == 0 {
            assert!(ids.contains(&aura));
        }
        act(
            &mut g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                bottom: Some(ids),
                ..Action::new("choose")
            },
        );
    }
    assert_ne!(g.regions[0].card.id, old_region);
    assert!(g.attachments.is_empty());
    for s in 0..4 {
        assert_eq!(
            g.players[s].deck[old_decks[s]..]
                .iter()
                .map(|c| c.definition.clone())
                .collect::<Vec<_>>(),
            expected[s]
        );
    }
    assert!(!g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ43"));
}

fn move_primitive(g: &mut Game, id: &str, to: usize) {
    let (r, c) = g.board(id).unwrap();
    let source = g.source_snapshot(c, Some(r));
    let mut spec = definition("JC014").abilities[0].clone();
    spec.event = None;
    spec.targets.clear();
    spec.requires_ready_source = false;
    spec.ops = vec![Op::MoveOnBoard {
        entity: EntityRef::Source,
        region: RegionRef::Chosen,
    }];
    let mut frame = g.make_frame(source.card.controller, source, &spec, vec![], vec![], None);
    frame.chosen_region = Some(to);
    g.resolve_frame(frame).unwrap();
    restore(g);
}
#[test]
fn xq43_hiding_real_spirit_drops_conversion_and_old_equipment_identity() {
    let mut g = initial();
    let spirit = field(&mut g, "JZ58", 0, 0);
    play_target(&mut g, "JC073", &spirit);
    play_region(&mut g, 0, 0);
    fund(&mut g, 0, "XQ16", 4);
    let entry = held(&mut g, "XQ16", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(entry),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    for _ in 0..40 {
        if g.pending.is_some() {
            break;
        }
        pass(&mut g);
    }
    assert!(g
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .any(|o| o.id == spirit));
    choose(&mut g, vec![spirit.clone()]);
    drain(&mut g);
    assert!(g.board(&spirit).is_none());
    let hidden = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JZ58")
        .unwrap();
    assert!(hidden.face_down);
    assert_ne!(hidden.id, spirit);
    for s in 0..4 {
        let v = g.view(s);
        let c = v.regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == hidden.id)
            .unwrap();
        assert!(c.converted_temporary_icons.is_none());
        if s != hidden.controller {
            assert!(c.card_id.is_none() && c.icons.is_none());
        }
    }
    assert!(g.attachments.iter().all(|a| a.card.definition != "JC073"));
    assert!(g.attachments.iter().any(|a| a.card.definition == "XQ43"));
}
