//! Disclosed boundary layouts; new declarations and responses use real actions.
use crate::{catalog, model::*, rules::*};

fn game() -> Game {
    let mut g = Game::new(
        "dream-unit".into(),
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

fn kill_source(g: &mut Game, source: &str) {
    fund(g, 0, "JC091", 3);
    let c = held(g, "JC091", 0);
    play_target(g, &c, source);
    until_choice(g, "trigger");
}
#[test]
fn dream_original_complete_fields_named_filter_and_finite_bindings() {
    let d = catalog::card("JZ58");
    assert_eq!(d.cost, 0);
    assert_eq!(d.loyalty, vec!["紫色"]);
    assert_eq!(d.magic, "星辰");
    assert_eq!(d.subtypes, vec!["梦魔"]);
    assert_eq!(d.defense, Some(1));
    assert_eq!(d.permanent_icons, Icons::default());
    assert_eq!(d.temporary_icons.influence, 1);
    assert!(!d.unique);
    assert!(definition("JZ58").traits.spirit);
    let v = catalog::card("JZ61");
    assert_eq!(v.cost, 2);
    assert_eq!(v.magic, "");
    assert_eq!(v.subtypes, vec!["人类", "宿主"]);
    assert_eq!(v.permanent_icons.combat, 1);
    assert_eq!(v.permanent_icons.influence, 1);
    for id in ["JZ58", "JZ61"] {
        validate_ability(id, &definition(id).abilities[0]).unwrap();
    }
    let mut bad = definition("JZ58").abilities[0].clone();
    bad.targets[0].relation = Relation::Any;
    assert!(validate_ability("JZ58", &bad).is_err());
    bad = definition("JZ58").abilities[0].clone();
    bad.ops.push(bad.ops[0].clone());
    assert!(validate_ability("JZ58", &bad).is_err());
    assert!(validate_ability("JZ61", &definition("JZ58").abilities[0]).is_err());
    let f = CardFilter::NamedCharacter("噩梦残像".into());
    assert!(f.matches(catalog::card("JZ58")));
    assert!(!f.matches(v));
    assert!(!f.matches(catalog::card("JZ67")));
}
#[test]
fn dream_spirit_strict_domain_count_repeats_exhaustion_controller_and_hidden_boundary() {
    let mut g = game();
    let s = field(&mut g, "JZ58", 0);
    assert!(g.spirit_protected(g.board(&s).unwrap().1));
    let enemy = field(&mut g, "LC12", 2);
    assert!(!g.spirit_protected(g.board(&s).unwrap().1));
    let ally = field(&mut g, "LC12", 1);
    g.board_mut(&ally).unwrap().exhausted = true;
    assert!(g.spirit_protected(g.board(&s).unwrap().1));
    g.board_mut(&ally).unwrap().face_down = true;
    assert!(!g.spirit_protected(g.board(&s).unwrap().1));
    g.board_mut(&ally).unwrap().face_down = false;
    g.board_mut(&enemy).unwrap().controller = 0;
    assert!(g.spirit_protected(g.board(&s).unwrap().1));
    g.board_mut(&s).unwrap().controller = 2;
    assert!(!g.spirit_protected(g.board(&s).unwrap().1));
    g.board_mut(&s).unwrap().face_down = true;
    assert!(!g.spirit_protected(g.board(&s).unwrap().1));
    restore(&mut g);
    for viewer in 0..4 {
        let v = g.view(viewer);
        assert!(v.regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == s)
            .unwrap()
            .current_spirit_protection
            .is_none());
    }
}
#[test]
fn dream_spirit_excludes_assets_other_regions_and_dead_zones_but_counts_attached_domains() {
    let mut g = game();
    let s = field(&mut g, "JZ58", 0);
    let e = field(&mut g, "LC12", 2);
    assert!(!g.spirit_protected(g.board(&s).unwrap().1));
    fund(&mut g, 0, "LC12", 8);
    let other = field(&mut g, "LC12", 0);
    let c = g.regions[0].cards.pop().unwrap();
    g.regions[1].cards.push(c);
    assert!(!g.spirit_protected(g.board(&s).unwrap().1));
    let host = field(&mut g, "JC125", 0);
    let a = g.make_card("JC073", 0);
    g.attachments.push(Attachment {
        card: a,
        host_id: host,
    });
    assert!(g.spirit_protected(g.board(&s).unwrap().1));
    assert!(g.board(&e).is_some() && g.board(&other).is_some());
    restore(&mut g);
}
fn blast(g: &mut Game, target: &str) {
    fund(g, 0, "JC102", 3);
    let b = held(g, "JC102", 0);
    play_target(g, &b, target);
    drain(g);
}
#[test]
fn dream_real_spell_damage_is_prevented_at_advantage_but_tie_and_deficit_kill() {
    for enemy_domains in 0..3 {
        let mut g = game();
        let s = field(&mut g, "JZ58", 2);
        for _ in 0..enemy_domains {
            field(&mut g, "LC12", 0);
        }
        blast(&mut g, &s);
        if enemy_domains == 0 {
            let c = g.board(&s).unwrap().1;
            assert_eq!(c.damage, 0);
            assert!(g.spirit_protected(c));
        } else {
            assert!(g.board(&s).is_none());
            assert!(g.players[2]
                .graveyard
                .iter()
                .any(|c| c.definition == "JZ58" && c.id != s));
        }
    }
}
#[test]
fn dream_spirit_domain_loss_from_real_simultaneous_spell_does_not_retroactively_damage() {
    let mut g = game();
    let s = field(&mut g, "JZ58", 2);
    let friendly = field(&mut g, "LC12", 2);
    let enemy = field(&mut g, "LC12", 0);
    let vest = g.make_card("XQ47", 0);
    g.attachments.push(Attachment {
        card: vest,
        host_id: enemy.clone(),
    });
    assert!(g.spirit_protected(g.board(&s).unwrap().1));
    fund(&mut g, 0, "JC047", 4);
    let c = held(&mut g, "JC047", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(c),
            region: Some(0),
            ..Action::new("play")
        },
    );
    drain(&mut g);
    assert!(g.board(&friendly).is_none());
    assert_eq!(g.board(&s).unwrap().1.damage, 0);
    assert!(g.board(&enemy).is_some());
    assert!(!g.spirit_protected(g.board(&s).unwrap().1));
    for viewer in 0..4 {
        assert_eq!(
            g.view(viewer).regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == s)
                .unwrap()
                .current_spirit_protection,
            Some(false)
        );
    }
}
#[test]
fn dream_spirit_does_not_prevent_real_death_wound_or_target_destruction() {
    let mut g = game();
    let s = field(&mut g, "JZ58", 2);
    let source = field(&mut g, "JZ59", 0);
    assert!(g.spirit_protected(g.board(&s).unwrap().1));
    kill_source(&mut g, &source);
    choose(&mut g, vec![s.clone()]);
    drain(&mut g);
    assert!(g.board(&s).is_none());
    let mut g = game();
    let s = field(&mut g, "JZ58", 2);
    field(&mut g, "LC12", 2);
    fund(&mut g, 0, "JC088", 3);
    let assassin = field(&mut g, "JC088", 0);
    g.board_mut(&assassin).unwrap().face_down = true;
    act(
        &mut g,
        0,
        Action {
            card_id: Some(assassin),
            ..Action::new("reveal")
        },
    );
    until_choice(&mut g, "trigger");
    assert!(g.spirit_protected(g.board(&s).unwrap().1));
    choose(&mut g, vec![s.clone()]);
    drain(&mut g);
    assert!(g.board(&s).is_none());
}
fn reveal_trigger(g: &mut Game) -> String {
    let s = field(g, "JZ58", 0);
    g.board_mut(&s).unwrap().face_down = true;
    fund(g, 0, "JC103", 1);
    act(
        g,
        0,
        Action {
            card_id: Some(s.clone()),
            ..Action::new("reveal")
        },
    );
    until_choice(g, "trigger");
    s
}
#[test]
fn dream_paid_zero_reveal_targets_only_opponents_and_target_player_removes_one_any_region() {
    let mut g = game();
    g.regions[2].influence = [1, 2];
    g.regions[4].influence = [0, 1];
    let old = reveal_trigger(&mut g);
    let p = g.pending.clone().unwrap();
    assert_eq!(p.seat, 0);
    assert_eq!(
        p.choice
            .options
            .iter()
            .map(|o| o.id.as_str())
            .collect::<Vec<_>>(),
        vec!["p2", "p3"]
    );
    for id in ["p0", "p1"] {
        reject(
            &mut g,
            0,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(vec![id.into()]),
                ..Action::new("choose")
            },
        );
    }
    choose(&mut g, vec!["p2".into()]);
    until_choice(&mut g, "target");
    let p = g.pending.clone().unwrap();
    assert_eq!(p.seat, 2);
    assert_eq!(
        p.choice
            .options
            .iter()
            .map(|o| o.id.as_str())
            .collect::<Vec<_>>(),
        vec!["region:2", "region:4"]
    );
    for viewer in 0..4 {
        assert_eq!(g.view(viewer).pending_choice.is_some(), viewer == 2);
    }
    reject(
        &mut g,
        3,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec!["region:4".into()]),
            ..Action::new("choose")
        },
    );
    reject(
        &mut g,
        2,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec!["region:0".into()]),
            ..Action::new("choose")
        },
    );
    choose(&mut g, vec!["region:4".into()]);
    drain(&mut g);
    assert_eq!(g.regions[2].influence, [1, 2]);
    assert_eq!(g.regions[4].influence, [0, 0]);
    assert!(g.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "JZ58" && c.id != old && !c.face_down));
    assert_eq!(resources(&g, 0), 1);
}
#[test]
fn dream_reveal_decline_and_no_influence_have_no_extra_cost() {
    for decline in [false, true] {
        let mut g = game();
        let _ = reveal_trigger(&mut g);
        choose(&mut g, if decline { vec![] } else { vec!["p2".into()] });
        drain(&mut g);
        assert!(g.pending.is_none());
        assert_eq!(resources(&g, 0), 1);
        assert!(g.regions.iter().all(|r| r.influence == [0, 0]));
    }
}
#[test]
fn dream_faceup_deploy_has_no_reveal_trigger_and_zero_cost_still_needs_purple_loyalty() {
    let mut g = game();
    let c = held(&mut g, "JZ58", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(c.clone()),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    fund(&mut g, 0, "JC103", 1);
    play_card(&mut g, 0, &c);
    drain(&mut g);
    assert_eq!(resources(&g, 0), 1);
    assert!(g.pending.is_none());
    assert!(g.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "JZ58" && !c.face_down));
}
#[test]
fn dream_death_search_uses_last_controller_own_deck_reveal_fresh_instance_and_shuffle() {
    let mut g = game();
    let source = field(&mut g, "JZ61", 3);
    g.board_mut(&source).unwrap().controller = 0;
    let n = g.make_card("JZ58", 0);
    let old = n.id.clone();
    g.players[0].deck.push(n);
    let n = g.make_card("JZ58", 3);
    let other = n.id.clone();
    g.players[3].deck.push(n);
    kill_source(&mut g, &source);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
    choose(&mut g, vec![id]);
    until_choice(&mut g, "search");
    let p = g.pending.clone().unwrap();
    assert_eq!(p.seat, 0);
    assert_eq!(p.choice.options.len(), 1);
    assert_eq!(p.choice.options[0].id, old);
    let seed = g.random;
    choose(&mut g, vec![old.clone()]);
    drain(&mut g);
    assert!(g.players[0]
        .hand
        .iter()
        .any(|c| c.definition == "JZ58" && c.id != old));
    assert!(g.players[3].deck.iter().any(|c| c.id == other));
    assert!(g.players[3]
        .graveyard
        .iter()
        .any(|c| c.definition == "JZ61" && c.id != source));
    assert_ne!(g.random, seed);
    assert!(g
        .view(2)
        .log
        .iter()
        .any(|l| l.text.contains("展示检索的 噩梦残像")));
}
#[test]
fn dream_death_search_empty_and_optional_decline_still_shuffle_without_exposing_other_decks() {
    for present in [false, true] {
        let mut g = game();
        let s = field(&mut g, "JZ61", 0);
        if present {
            let c = g.make_card("JZ58", 0);
            g.players[0].deck.push(c);
        }
        kill_source(&mut g, &s);
        let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
        choose(&mut g, vec![id]);
        if present {
            until_choice(&mut g, "search");
            let seed = g.random;
            choose(&mut g, vec![]);
            drain(&mut g);
            assert_ne!(g.random, seed);
        } else {
            drain(&mut g);
        }
        assert!(g.players[0].hand.iter().all(|c| c.definition != "JZ58"));
    }
}
#[test]
fn dream_real_amnesia_offers_shuffle_only_for_actual_current_faceup_controller_and_both_choices_restore(
) {
    for seat in [0, 1, 2] {
        for hidden in [false, true] {
            for accept in [false, true] {
                let mut g = game();
                let s = field(&mut g, "JZ58", seat);
                g.board_mut(&s).unwrap().face_down = hidden;
                fund(&mut g, 0, "JZ67", 2);
                let c = held(&mut g, "JZ67", 0);
                held(&mut g, "JC125", 2);
                play_target(&mut g, &c, "p2");
                for _ in 0..100 {
                    if g.pending
                        .as_ref()
                        .is_some_and(|p| p.choice.kind == "optional-shuffle")
                        || g.pending.is_none() && g.stack.is_empty()
                    {
                        break;
                    }
                    if g.pending.is_some() {
                        resolve_choice(&mut g);
                    } else {
                        pass(&mut g);
                    }
                }
                let eligible = seat == 0 && !hidden;
                assert_eq!(
                    g.pending
                        .as_ref()
                        .is_some_and(|p| p.choice.kind == "optional-shuffle"),
                    eligible
                );
                if eligible {
                    let deck = g.players[2]
                        .deck
                        .iter()
                        .map(|c| c.id.clone())
                        .collect::<Vec<_>>();
                    let seed = g.random;
                    choose(
                        &mut g,
                        if accept {
                            vec!["shuffle".into()]
                        } else {
                            vec![]
                        },
                    );
                    drain(&mut g);
                    if accept {
                        assert_ne!(g.random, seed);
                    } else {
                        assert_eq!(g.random, seed);
                        assert_eq!(
                            g.players[2]
                                .deck
                                .iter()
                                .map(|c| c.id.clone())
                                .collect::<Vec<_>>(),
                            deck
                        );
                    }
                }
                restore(&mut g);
            }
        }
    }
}
#[test]
fn dream_spirit_with_real_granted_combat_icon_still_participates_under_hegemony() {
    let mut g = game();
    let s = field(&mut g, "JZ58", 0);
    fund(&mut g, 0, "JC008", 2);
    let c = held(&mut g, "JC008", 0);
    play_target(&mut g, &c, &s);
    drain(&mut g);
    assert_eq!(g.icons(g.board(&s).unwrap().1, 0).combat, 1);
    assert!(g.spirit_protected(g.board(&s).unwrap().1));
}
