//! Explicit boundary layouts/checkpoints; all recorded commands use Room ABI.
//! The separate natural case starts with legal decks and real dealt cards.
use super::*;
use hegemony_server::model::Card;

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
fn fund(g: &mut Game, id: &str, seat: usize, n: usize) {
    for _ in 0..n {
        let c = g.make_card(id, seat);
        g.players[seat].assets.push(c);
    }
}
fn board<'a>(g: &'a Game, id: &str) -> Option<(usize, &'a Card)> {
    g.regions
        .iter()
        .enumerate()
        .find_map(|(r, x)| x.cards.iter().find(|c| c.id == id).map(|c| (r, c)))
}
fn resources(g: &Game, s: usize) -> usize {
    g.players[s].assets.iter().filter(|c| !c.exhausted).count()
}
fn action(id: &str, key: &str, target: Option<&str>, sacrifice: Option<&str>) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(key.into()),
        target_id: target.map(str::to_string),
        cost_selected: sacrifice.map(|id| vec![id.into()]),
        ..Action::new("activate")
    }
}
fn pass_action(g: &Game) -> (usize, Action) {
    (0..4)
        .find_map(|s| {
            g.legal_actions(s)
                .into_iter()
                .find(|a| a.action.kind == "pass")
                .map(|a| (s, a.action))
        })
        .unwrap()
}
fn pass_top(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    let n = r.stack.len();
    assert!(n > 0);
    for _ in 0..80 {
        if r.stack.len() < n && r.pending.is_none() {
            return;
        }
        let (s, a) = if r.pending.is_some() {
            pick_choice(&r.game)
        } else {
            pass_action(&r.game)
        };
        apply_game(r, steps, s, a);
    }
    panic!("bounded equipment stack");
}
fn advance(r: &mut RoomEnvelope, steps: &mut Vec<Value>, done: impl Fn(&RoomEnvelope) -> bool) {
    for _ in 0..700 {
        if done(r) {
            return;
        }
        let (s, a) = if r.pending.is_some() {
            pick_choice(&r.game)
        } else {
            pass_action(&r.game)
        };
        apply_game(r, steps, s, a);
    }
    panic!("bounded equipment advance");
}
fn cast(r: &mut RoomEnvelope, steps: &mut Vec<Value>, s: usize, id: &str, target: &str) -> String {
    let definition = r.players[s]
        .hand
        .iter()
        .find(|c| c.id == id)
        .unwrap()
        .definition
        .clone();
    apply_game(
        r,
        steps,
        s,
        Action {
            card_id: Some(id.into()),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    );
    pass_top(r, steps);
    r.attachments
        .iter()
        .rev()
        .find(|a| a.card.definition == definition)
        .map_or_else(|| id.into(), |a| a.card.id.clone())
}
fn activate(
    r: &mut RoomEnvelope,
    steps: &mut Vec<Value>,
    s: usize,
    id: &str,
    key: &str,
    target: Option<&str>,
    sacrifice: Option<&str>,
) {
    apply_game(r, steps, s, action(id, key, target, sacrifice));
}
fn reject(r: &mut RoomEnvelope, steps: &mut Vec<Value>, s: usize, a: Action) {
    let c = RoomCommand {
        command_id: format!("oracle:rejected:{}", steps.len()),
        expected_version: r.revision,
        action: a.into(),
    };
    let now = r.pacing.last_server_now_ms;
    let t = r.transition(s, Some(c.clone()), now).unwrap();
    assert_eq!(t.outcome, "rejected");
    assert!(!t.changed);
    record_transition(r, steps, "applyRoom", json!([s, c, now.to_string()]), t);
}
fn pre_attach(g: &mut Game, id: &str, host: &str) -> String {
    g.apply(
        0,
        Action {
            card_id: Some(id.into()),
            target_id: Some(host.into()),
            ..Action::new("play")
        },
    )
    .unwrap();
    for _ in 0..80 {
        if g.stack.is_empty() && g.pending.is_none() {
            return g.attachments.last().unwrap().card.id.clone();
        }
        let (s, a) = if g.pending.is_some() {
            pick_choice(g)
        } else {
            pass_action(g)
        };
        g.apply(s, a).unwrap();
    }
    panic!("bounded prepared attachment");
}
// Initial response checkpoints must include the Room pacing state generated
// by the actual declaration, not only a Game stack wrapped in default pacing.
fn declare_checkpoint(g: &mut Game, action: Action) -> RoomEnvelope {
    assert!(g.stack.is_empty() && g.pending.is_none());
    let room = RoomEnvelope::from_game(g.clone());
    let command = RoomCommand {
        command_id: "checkpoint:declare".into(),
        expected_version: room.revision,
        action: action.into(),
    };
    let transition = room.transition(0, Some(command), 0).unwrap();
    assert_eq!(transition.outcome, "accepted");
    let next = RoomEnvelope::from_persisted(&transition.state).unwrap();
    assert_eq!(
        serde_json::to_string(&room.replay_events(&transition.journal).unwrap()).unwrap(),
        transition.state
    );
    *g = next.game.clone();
    next
}
fn move_initial(g: &mut Game, id: &str, region: usize) {
    let old = g
        .regions
        .iter()
        .position(|r| r.cards.iter().any(|c| c.id == id))
        .unwrap();
    let i = g.regions[old]
        .cards
        .iter()
        .position(|c| c.id == id)
        .unwrap();
    let c = g.regions[old].cards.remove(i);
    g.regions[region].cards.push(c);
}
fn icons(g: &Game, id: &str) -> hegemony_server::model::Icons {
    let (r, c) = board(g, id).unwrap();
    g.current_icons(c, r)
}
fn overridden(g: &Game, id: &str) -> bool {
    g.turn_attribute_modifiers
        .iter()
        .any(|m| m.target_instance == id && m.printed_defense_override == Some(1))
}

fn boundary(kind: &str) -> Value {
    let mut g = attachment_initial("teams", "888888888888888888880131");
    g.regions[0].card = g.make_card("DQJC115", 0);
    g.regions[0].influence = [0, 0];
    let host = field(&mut g, "LC01", 0);
    let enemy = field(&mut g, "JC003", 2);
    let hidden = field(&mut g, "JC125", 3);
    g.regions[0].cards.last_mut().unwrap().face_down = true;
    let victim1 = field(&mut g, "JC125", 0);
    let victim2 = field(&mut g, "JC125", 0);
    let victim3 = field(&mut g, "JC125", 0);
    let helper = field(&mut g, "JC071", 0);
    let outside = field(&mut g, "JC003", 2);
    move_initial(&mut g, &outside, 1);
    let mut teammate_host = None;
    let mut control_source = None;
    if kind == "controller-only-hosts" {
        teammate_host = Some(field(&mut g, "JC125", 1));
        control_source = Some(field(&mut g, "JZ27", 1));
        g.regions[0].cards.last_mut().unwrap().face_down = true;
        fund(&mut g, "XQ16", 1, 6);
    }
    let mut gun = held(&mut g, "JC116", 0);
    let mut knife = held(&mut g, "JC020", 0);
    let mut blade = held(&mut g, "JC093", 0);
    let mut water = held(&mut g, "XQ07", 0);
    let vest = held(&mut g, "XQ47", 0);
    let return_host = held(&mut g, "JC006", 2);
    let destroy_source = held(&mut g, "JC005", 2);
    let late = held(&mut g, "JC003", 0);
    fund(&mut g, "JC014", 0, 8);
    fund(&mut g, "JC084", 0, 4);
    fund(&mut g, "JC002", 0, 3);
    fund(&mut g, "JC002", 2, 4);
    let checkpoint = matches!(
        kind,
        "knife-source-moved"
            | "knife-target-moved"
            | "knife-paid-shield"
            | "blade-host-moved"
            | "blade-host-rebound"
            | "blade-control"
            | "water-source-moved"
    );
    let mut checkpoint_room = None;
    if checkpoint {
        if kind.starts_with("knife") {
            knife = pre_attach(&mut g, &knife, &host);
            checkpoint_room = Some(declare_checkpoint(
                &mut g,
                action(
                    &knife,
                    if kind == "knife-paid-shield" {
                        "sacrifice-local-damage"
                    } else {
                        "exhaust-local-hidden"
                    },
                    Some(if kind == "knife-paid-shield" {
                        &enemy
                    } else {
                        &hidden
                    }),
                    None,
                ),
            ));
            if kind == "knife-source-moved" {
                move_initial(&mut g, &host, 1);
            }
            if kind == "knife-target-moved" {
                move_initial(&mut g, &hidden, 1);
            }
            if kind == "knife-paid-shield" {
                g.regions[0]
                    .cards
                    .iter_mut()
                    .find(|c| c.id == enemy)
                    .unwrap()
                    .shield = 1;
            }
        } else if kind.starts_with("blade") {
            blade = pre_attach(&mut g, &blade, &host);
            checkpoint_room = Some(declare_checkpoint(
                &mut g,
                action(
                    &blade,
                    "sacrifice-character-host-icons",
                    None,
                    Some(&victim1),
                ),
            ));
            if kind == "blade-host-moved" {
                move_initial(&mut g, &host, 1);
            }
            if kind == "blade-host-rebound" {
                g.attachments
                    .iter_mut()
                    .find(|a| a.card.id == blade)
                    .unwrap()
                    .host_id = victim2.clone();
            }
            if kind == "blade-control" {
                for _ in 0..80 {
                    if g.stack.is_empty() {
                        break;
                    }
                    let (s, a) = pass_action(&g);
                    g.apply(s, a).unwrap();
                }
                g.attachments
                    .iter_mut()
                    .find(|a| a.card.id == blade)
                    .unwrap()
                    .card
                    .controller = 1;
                g.regions[0]
                    .cards
                    .iter_mut()
                    .find(|c| c.id == victim2)
                    .unwrap()
                    .controller = 1;
                g.regions[0]
                    .cards
                    .iter_mut()
                    .find(|c| c.id == victim3)
                    .unwrap()
                    .controller = 1;
            }
        } else {
            water = pre_attach(&mut g, &water, &host);
            checkpoint_room = Some(declare_checkpoint(
                &mut g,
                action(&water, "sacrifice-local-printed-defense", None, None),
            ));
            move_initial(&mut g, &host, 1);
        }
    }
    if kind == "gun-win" {
        g.regions[0].influence = [9, 0];
        g.players[0]
            .hand
            .iter_mut()
            .find(|c| c.id == gun)
            .unwrap()
            .owner = 3;
        gun = pre_attach(&mut g, &gun, &host);
        prepared_win(&mut g, 0, 0);
    }
    if kind == "water-group" {
        let c = g.make_card("BQ022", 2);
        g.attachments.push(Attachment {
            card: c,
            host_id: enemy.clone(),
        });
        g.regions[0]
            .cards
            .iter_mut()
            .find(|c| c.id == enemy)
            .unwrap()
            .shield = 2;
    }
    let mut r = if g.stack.is_empty() {
        RoomEnvelope::from_game(g)
    } else {
        let mut prepared = checkpoint_room.expect("prepared response pacing");
        prepared.game = g;
        prepared.revision = prepared.game.version;
        prepared
    };
    // Validate the very first opaque state as well as all later transitions.
    RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();
    let mut steps = vec![step(
        &r,
        "initialFixture",
        json!([serde_json::to_string(&r).unwrap()]),
        0,
    )];
    let mut milestones = vec![];
    match kind {
        "controller-only-hosts" => {
            let before = resources(&r.game, 0);
            for source in [&knife, &water] {
                for target in [teammate_host.as_ref().unwrap(), &enemy] {
                    assert!(!r
                        .game
                        .legal_actions(0)
                        .iter()
                        .any(|a| a.action.kind == "play"
                            && a.action.card_id.as_ref() == Some(source)
                            && a.action.target_id.as_ref() == Some(target)));
                    reject(
                        &mut r,
                        &mut steps,
                        0,
                        Action {
                            card_id: Some(source.clone()),
                            target_id: Some(target.clone()),
                            ..Action::new("play")
                        },
                    );
                    assert_eq!(resources(&r.game, 0), before);
                    assert!(r.players[0].hand.iter().any(|c| &c.id == source));
                    assert!(r.attachments.is_empty() && r.players[0].graveyard.is_empty());
                }
            }
            knife = cast(&mut r, &mut steps, 0, &knife, &victim1);
            water = cast(&mut r, &mut steps, 0, &water, &victim1);
            assert_eq!(resources(&r.game, 0), before - 3);
            gun = cast(&mut r, &mut steps, 0, &gun, &victim1);
            blade = cast(&mut r, &mut steps, 0, &blade, &victim1);
            let source = control_source.unwrap();
            apply_game(
                &mut r,
                &mut steps,
                1,
                Action {
                    card_id: Some(source),
                    ..Action::new("reveal")
                },
            );
            for _ in 0..80 {
                if r.pending.is_some() {
                    break;
                }
                let (seat, action) = pass_action(&r.game);
                apply_game(&mut r, &mut steps, seat, action);
            }
            let pending = r.pending.clone().unwrap();
            assert!(pending.choice.options.iter().any(|o| o.id == victim1));
            apply_game(
                &mut r,
                &mut steps,
                pending.seat,
                Action {
                    choice_id: Some(pending.choice.id),
                    selected: Some(vec![victim1.clone()]),
                    ..Action::new("choose")
                },
            );
            pass_top(&mut r, &mut steps);
            assert_eq!(board(&r.game, &victim1).unwrap().1.controller, 1);
            for (definition, old) in [("JC020", knife), ("XQ07", water)] {
                assert!(!r.attachments.iter().any(|a| a.card.id == old));
                let cards = r.players[0]
                    .graveyard
                    .iter()
                    .filter(|c| c.definition == definition)
                    .collect::<Vec<_>>();
                assert_eq!(cards.len(), 1);
                assert_ne!(cards[0].id, old);
                assert_eq!((cards[0].owner, cards[0].controller), (0, 0));
            }
            assert!(r.attachments.iter().any(|a| a.card.id == gun));
            assert!(r.attachments.iter().any(|a| a.card.id == blade));
            milestones.push(json!({"kind":"both-own-paid-teammate-enemy-atomic-rejection-actual-JZ27-control-change-cleanup","step":steps.len()-1}));
        }
        "paid-four" => {
            let before = resources(&r.game, 0);
            gun = cast(&mut r, &mut steps, 0, &gun, &host);
            knife = cast(&mut r, &mut steps, 0, &knife, &host);
            blade = cast(&mut r, &mut steps, 0, &blade, &host);
            water = cast(&mut r, &mut steps, 0, &water, &host);
            assert_eq!(resources(&r.game, 0), before - 5);
            assert_eq!(r.attachments.len(), 4);
            reject(
                &mut r,
                &mut steps,
                0,
                action(&gun, "attach", Some(&victim1), None),
            );
            activate(
                &mut r,
                &mut steps,
                0,
                &knife,
                "exhaust-local-hidden",
                Some(&hidden),
                None,
            );
            pass_top(&mut r, &mut steps);
            assert!(board(&r.game, &hidden).unwrap().1.exhausted);
            assert!(!board(&r.game, &host).unwrap().1.exhausted);
            for v in [&victim1, &victim2] {
                activate(
                    &mut r,
                    &mut steps,
                    0,
                    &blade,
                    "sacrifice-character-host-icons",
                    None,
                    Some(v),
                );
                pass_top(&mut r, &mut steps);
            }
            reject(
                &mut r,
                &mut steps,
                0,
                action(
                    &blade,
                    "sacrifice-character-host-icons",
                    None,
                    Some(&victim3),
                ),
            );
            assert_eq!(r.turn_ability_usage[0].uses, 2);
            assert_eq!(icons(&r.game, &host).combat, 3);
            activate(
                &mut r,
                &mut steps,
                0,
                &water,
                "sacrifice-local-printed-defense",
                None,
                None,
            );
            pass_top(&mut r, &mut steps);
            for s in 0..4 {
                assert_eq!(
                    r.game.view(s).regions[0]
                        .characters
                        .iter()
                        .find(|c| c.instance_id == host)
                        .unwrap()
                        .current_printed_defense,
                    Some(1)
                );
            }
            activate(
                &mut r,
                &mut steps,
                0,
                &knife,
                "sacrifice-local-damage",
                Some(&enemy),
                None,
            );
            pass_top(&mut r, &mut steps);
            assert!(board(&r.game, &enemy).is_none());
            assert!(r.players[0]
                .graveyard
                .iter()
                .any(|c| c.definition == "JC020" && c.id != knife));
            milestones.push(json!({"kind":"actual-five-resource-four-card-play-plus-all-activated-abilities","step":steps.len()-1}));
        }
        "knife-source-moved" | "knife-target-moved" => {
            pass_top(&mut r, &mut steps);
            assert_eq!(
                board(&r.game, &hidden).unwrap().1.exhausted,
                kind == "knife-source-moved"
            );
            assert!(
                r.attachments
                    .iter()
                    .find(|a| a.card.id == knife)
                    .unwrap()
                    .card
                    .exhausted
            );
        }
        "knife-paid-shield" => {
            pass_top(&mut r, &mut steps);
            assert_eq!(board(&r.game, &enemy).unwrap().1.damage, 0);
            assert_eq!(board(&r.game, &enemy).unwrap().1.shield, 0);
            assert!(!r.attachments.iter().any(|a| a.card.id == knife));
            assert_eq!(
                r.players[0]
                    .graveyard
                    .iter()
                    .filter(|c| c.definition == "JC020")
                    .count(),
                1
            );
        }
        "blade-twice" => {
            blade = cast(&mut r, &mut steps, 0, &blade, &host);
            reject(
                &mut r,
                &mut steps,
                0,
                action(&blade, "sacrifice-character-host-icons", None, Some(&enemy)),
            );
            assert!(r.turn_ability_usage.is_empty());
            for v in [&victim1, &victim2] {
                activate(
                    &mut r,
                    &mut steps,
                    0,
                    &blade,
                    "sacrifice-character-host-icons",
                    None,
                    Some(v),
                );
                pass_top(&mut r, &mut steps);
            }
            reject(
                &mut r,
                &mut steps,
                0,
                action(
                    &blade,
                    "sacrifice-character-host-icons",
                    None,
                    Some(&victim3),
                ),
            );
            assert_eq!(r.turn_ability_usage[0].uses, 2);
            let turn = r.turn;
            advance(&mut r, &mut steps, |r| r.turn > turn);
            assert!(r.turn_ability_usage.is_empty());
            assert!(r.turn_attribute_modifiers.is_empty());
        }
        "blade-source-left" | "blade-host-left" => {
            blade = cast(&mut r, &mut steps, 0, &blade, &host);
            activate(
                &mut r,
                &mut steps,
                0,
                &blade,
                "sacrifice-character-host-icons",
                None,
                Some(&victim1),
            );
            advance(&mut r, &mut steps, |r| r.priority_team == 1);
            if kind == "blade-source-left" {
                cast(&mut r, &mut steps, 2, &destroy_source, &blade);
            } else {
                cast(&mut r, &mut steps, 2, &return_host, &host);
            }
            pass_top(&mut r, &mut steps);
            assert_eq!(r.turn_ability_usage[0].uses, 1);
            if kind == "blade-source-left" {
                assert_eq!(icons(&r.game, &host).combat, 1);
            } else {
                assert!(board(&r.game, &host).is_none());
                assert!(r.turn_attribute_modifiers.is_empty());
            }
            assert_eq!(icons(&r.game, &victim2).combat, 0);
        }
        "blade-host-moved" | "blade-host-rebound" => {
            pass_top(&mut r, &mut steps);
            assert_eq!(icons(&r.game, &host).combat, 1);
            assert_eq!(icons(&r.game, &victim2).combat, 0);
            assert_eq!(r.turn_ability_usage[0].uses, 1);
        }
        "blade-control" => {
            activate(
                &mut r,
                &mut steps,
                1,
                &blade,
                "sacrifice-character-host-icons",
                None,
                Some(&victim2),
            );
            pass_top(&mut r, &mut steps);
            reject(
                &mut r,
                &mut steps,
                1,
                action(
                    &blade,
                    "sacrifice-character-host-icons",
                    None,
                    Some(&victim3),
                ),
            );
            assert_eq!(r.turn_ability_usage[0].uses, 2);
        }
        "water-group" => {
            cast(&mut r, &mut steps, 0, &vest, &host);
            water = cast(&mut r, &mut steps, 0, &water, &host);
            activate(
                &mut r,
                &mut steps,
                0,
                &helper,
                "protect-local-character",
                Some(&host),
                None,
            );
            pass_top(&mut r, &mut steps);
            assert_eq!(r.game.defense(board(&r.game, &host).unwrap().1, 0), 6);
            activate(
                &mut r,
                &mut steps,
                0,
                &water,
                "sacrifice-local-printed-defense",
                None,
                None,
            );
            pass_top(&mut r, &mut steps);
            assert_eq!(r.game.defense(board(&r.game, &host).unwrap().1, 0), 3);
            assert_eq!(board(&r.game, &enemy).unwrap().1.shield, 2);
            assert!(overridden(&r.game, &enemy));
            assert!(!overridden(&r.game, &hidden));
            assert!(!overridden(&r.game, &outside));
            assert!(!overridden(&r.game, &victim1));
            apply_game(
                &mut r,
                &mut steps,
                0,
                Action {
                    card_id: Some(late),
                    region: Some(0),
                    ..Action::new("deploy")
                },
            );
            pass_top(&mut r, &mut steps);
            let later = r.regions[0]
                .cards
                .iter()
                .find(|c| c.definition == "JC003" && c.controller == 0)
                .unwrap();
            assert!(!overridden(&r.game, &later.id));
            let turn = r.turn;
            advance(&mut r, &mut steps, |r| r.turn > turn);
            assert!(r.turn_attribute_modifiers.is_empty());
        }
        "water-source-moved" => {
            pass_top(&mut r, &mut steps);
            assert!(!overridden(&r.game, &host));
            assert!(overridden(&r.game, &enemy));
            assert!(!overridden(&r.game, &outside));
            assert!(r.players[0]
                .graveyard
                .iter()
                .any(|c| c.definition == "XQ07" && c.id != water));
        }
        "gun-win" => {
            advance(&mut r, &mut steps, |r| {
                r.players[3].hand.iter().any(|c| c.definition == "JC116")
            });
            assert!(board(&r.game, &host).is_none());
            let fresh = r.players[3]
                .hand
                .iter()
                .find(|c| c.definition == "JC116")
                .unwrap();
            assert_ne!(fresh.id, gun);
            assert_eq!((fresh.owner, fresh.controller), (3, 3));
            assert!(r.players[0].deck.iter().any(|c| c.definition == "LC01"));
        }
        _ => panic!("unknown equipment fixture {kind}"),
    }
    json!({"name":format!("equipment-abilities-{kind}"),"seed":r.seed.to_string(),"syntheticInitialLayout":true,"syntheticInitialFunding":true,"preparedResponseCheckpoint":checkpoint,"preparedWinWindow":kind=="gun-win","postInitialStateInjection":false,"publicNaturalUiAcceptance":false,"milestones":milestones,"steps":steps})
}
fn draft() -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    d.id = "local-equipment-abilities".into();
    d.name = "Local equipment abilities".into();
    d.society_id = None;
    d.cards = [
        "JC116", "JC020", "JC093", "XQ07", "JC014", "JC084", "JC002", "LC01", "JC003", "JC085",
        "JC086", "JC088",
    ]
    .into_iter()
    .map(|id| catalog::DeckEntry {
        card_id: id.into(),
        count: 3,
    })
    .collect();
    d.cards.push(catalog::DeckEntry {
        card_id: "JC125".into(),
        count: 14,
    });
    deck::validate(d).unwrap()
}
fn own<'a>(g: &'a Game, s: usize, a: &Action) -> Option<&'a Card> {
    a.card_id.as_ref().and_then(|id| {
        g.players[s]
            .hand
            .iter()
            .chain(g.regions.iter().flat_map(|r| &r.cards))
            .chain(g.attachments.iter().map(|a| &a.card))
            .find(|c| c.id == *id && c.controller == s)
    })
}
fn natural_actions(seed: u64) -> Vec<(usize, Action)> {
    let d = draft();
    let mut g = Game::new_with_deck(
        "888888888888888888880132".into(),
        "NATURAL".into(),
        "teams".into(),
        "P0".into(),
        d.clone(),
        seed,
    )
    .unwrap();
    for s in 1..4 {
        g.join_with_deck(format!("P{s}"), d.clone()).unwrap();
    }
    let mut actions = vec![];
    for s in 0..4 {
        let a = Action::new("ready");
        g.apply(s, a.clone()).unwrap();
        actions.push((s, a));
    }
    let a = Action::new("start");
    g.apply(0, a.clone()).unwrap();
    actions.push((0, a));
    let mut paid = [false; 4];
    let mut exhausted = false;
    let mut twice = false;
    let mut watered = false;
    let mut damaged = false;
    let mut complete_turn = None;
    for _ in 0..3800 {
        for (i, id) in ["JC116", "JC020", "JC093", "XQ07"].iter().enumerate() {
            paid[i] |= g.attachments.iter().any(|a| a.card.definition == *id);
        }

        twice |= g.turn_ability_usage.iter().any(|u| u.uses == 2);
        if paid.iter().all(|x| *x)
            && exhausted
            && twice
            && watered
            && damaged
            && g.stack.is_empty()
            && g.pending.is_none()
        {
            complete_turn.get_or_insert(g.turn);
        }
        if complete_turn.is_some_and(|t| g.turn > t) {
            assert!(g.turn_attribute_modifiers.is_empty());
            assert!(g.turn_ability_usage.is_empty());
            return actions;
        }
        assert!(g.status=="playing"&&g.turn<=12,"natural equipment stalled seed={seed} turn={} flags={:?} hand={:?} assets={:?} board={:?}",g.turn,(paid,exhausted,twice,watered,damaged),g.players[0].hand.iter().map(|c|&c.definition).collect::<Vec<_>>(),g.players[0].assets.iter().map(|c|&c.definition).collect::<Vec<_>>(),g.regions[2].cards.iter().map(|c|(&c.definition,c.controller,c.face_down)).collect::<Vec<_>>());
        let hosts = g.regions[2]
            .cards
            .iter()
            .filter(|c| c.controller == 0 && !c.face_down && c.definition == "JC125")
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        let first = hosts.first();
        let (s, a) = if let Some(p) = &g.pending {
            if p.choice.kind == "mulligan"
                || matches!(&p.resolution, ChoiceResolution::Declare { .. })
            {
                (
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(vec![]),
                        ..Action::new("choose")
                    },
                )
            } else if p.choice.kind == "discard" {
                let critical = ["JC116", "JC020", "JC093", "XQ07", "LC01", "JC125"];
                let mut counts = BTreeMap::new();
                for c in &g.players[p.seat].hand {
                    *counts.entry(c.definition.clone()).or_insert(0usize) += 1;
                }
                let mut selected = vec![];
                for _ in 0..p.choice.min.unwrap_or(0) {
                    let option = p
                        .choice
                        .options
                        .iter()
                        .filter(|o| !selected.contains(&o.id))
                        .min_by_key(|o| {
                            let c = g.players[p.seat]
                                .hand
                                .iter()
                                .find(|c| c.id == o.id)
                                .unwrap();
                            let color = &catalog::card(&c.definition).color;
                            let available = g.players[p.seat]
                                .assets
                                .iter()
                                .filter(|a| catalog::card(&a.definition).color == *color)
                                .count();
                            let needs_color = p.seat == 0
                                && ((color == "黑" && available < 2)
                                    || (color == "绿" && available < 1));
                            if ["JC116", "JC020", "JC093", "XQ07"].contains(&c.definition.as_str())
                                && counts[&c.definition] <= 1
                            {
                                3
                            } else if needs_color {
                                2
                            } else if critical.contains(&c.definition.as_str())
                                && counts[&c.definition] <= 1
                            {
                                1
                            } else {
                                0
                            }
                        })
                        .unwrap();
                    let c = g.players[p.seat]
                        .hand
                        .iter()
                        .find(|c| c.id == option.id)
                        .unwrap();
                    *counts.get_mut(&c.definition).unwrap() -= 1;
                    selected.push(option.id.clone());
                }
                (
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(selected),
                        ..Action::new("choose")
                    },
                )
            } else {
                pick_choice(&g)
            }
        } else {
            let legal = (0..4).map(|s| g.legal_actions(s)).collect::<Vec<_>>();
            let mut chosen = None;
            if g.stack.is_empty() {
                for s in 0..4 {
                    let count = |color: &str| {
                        g.players[s]
                            .assets
                            .iter()
                            .filter(|c| catalog::card(&c.definition).color == color)
                            .count()
                    };
                    if g.players[s].assets.len() < 8
                        || (s == 0 && (count("黑") < 2 || count("绿") < 1))
                    {
                        let candidate = legal[s]
                            .iter()
                            .filter(|a| a.action.kind == "asset")
                            .filter_map(|a| own(&g, s, &a.action).map(|c| (a, c)))
                            .filter(|(_, c)| {
                                let needed = if s != 0 {
                                    0
                                } else {
                                    match c.definition.as_str() {
                                        "JC116" => usize::from(!paid[0]),
                                        "JC020" => usize::from(!(exhausted && damaged)),
                                        "JC093" => usize::from(!twice),
                                        "XQ07" => usize::from(!watered),
                                        "LC01" => 0,
                                        "JC125" => {
                                            let required = if !twice {
                                                3usize
                                            } else if !damaged {
                                                2
                                            } else {
                                                0
                                            };
                                            required.saturating_sub(hosts.len())
                                        }
                                        _ => 0,
                                    }
                                };
                                g.players[s]
                                    .hand
                                    .iter()
                                    .filter(|h| h.definition == c.definition)
                                    .count()
                                    > needed
                            })
                            .min_by_key(|(_, c)| {
                                let color = &catalog::card(&c.definition).color;
                                if s == 0 && color == "黑" && count("黑") < 2 {
                                    0
                                } else if s == 0 && color == "绿" && count("绿") < 1 {
                                    1
                                } else {
                                    3
                                }
                            });
                        if let Some((a, _)) = candidate {
                            chosen = Some((s, a.action.clone()));
                            break;
                        }
                    }
                    if s == 0 {
                        if let Some(a) = legal[0].iter().find(|a| {
                            let def = own(&g, 0, &a.action).map(|c| c.definition.as_str());
                            let target = a.action.target_id.as_ref();
                            let want = match (a.action.kind.as_str(), def) {
                                ("deploy", Some("JC125")) => {
                                    (hosts.len() < 3 && !twice) || (hosts.len() < 2 && !damaged)
                                }
                                ("play", Some(id @ ("JC116" | "JC020" | "JC093" | "XQ07"))) => {
                                    let required = match id {
                                        "JC116" => !paid[0],
                                        "JC020" => !(exhausted && damaged),
                                        "JC093" => !twice,
                                        _ => !watered,
                                    };
                                    required
                                        && !g.attachments.iter().any(|x| x.card.definition == id)
                                        && first.is_some()
                                        && target == first
                                }
                                ("activate", Some("JC020"))
                                    if a.action.ability_id.as_deref()
                                        == Some("exhaust-local-hidden") =>
                                {
                                    !damaged
                                        && target.is_some_and(|id| {
                                            board(&g, id)
                                                .is_some_and(|(_, c)| c.face_down && !c.exhausted)
                                        })
                                }
                                ("activate", Some("XQ07")) => !watered && paid.iter().all(|x| *x),
                                ("activate", Some("JC093")) => {
                                    watered
                                        && !twice
                                        && (hosts.len() >= 3
                                            || g.turn_ability_usage.iter().any(|u| {
                                                u.turn == g.turn
                                                    && u.source_instance
                                                        == *a.action.card_id.as_ref().unwrap()
                                                    && u.uses == 1
                                            }))
                                        && a.action.cost_selected.as_ref().is_some_and(|ids| {
                                            ids.iter()
                                                .all(|id| hosts.contains(id) && Some(id) != first)
                                        })
                                }
                                ("activate", Some("JC020")) => {
                                    twice
                                        && !damaged
                                        && target.is_some_and(|id| {
                                            hosts.contains(id) && Some(id) != first
                                        })
                                }
                                _ => false,
                            };
                            want && a.action.region.is_none_or(|r| r == 2)
                        }) {
                            chosen = Some((0, a.action.clone()));
                            break;
                        }
                    }
                    if s == 2
                        && !exhausted
                        && g.attachments.iter().any(|a| a.card.definition == "JC020")
                        && !g.regions[2].cards.iter().any(|c| c.face_down)
                    {
                        if let Some(a) = legal[2]
                            .iter()
                            .find(|a| a.action.kind == "conceal" && a.action.region == Some(2))
                        {
                            chosen = Some((2, a.action.clone()));
                            break;
                        }
                    }
                }
            }
            chosen.unwrap_or_else(|| pass_action(&g))
        };
        let def = own(&g, s, &a).map(|c| c.definition.clone());
        if a.kind == "activate" && def.as_deref() == Some("XQ07") {
            watered = true;
        }
        if a.kind == "activate" && def.as_deref() == Some("JC020") {
            if a.ability_id.as_deref() == Some("exhaust-local-hidden") {
                exhausted = true;
            } else {
                damaged = true;
            }
        }
        if s == 0 && matches!(a.kind.as_str(), "asset" | "play" | "deploy" | "activate") {
            eprintln!(
                "natural equipment turn={} kind={} def={:?} assets={}",
                g.turn,
                a.kind,
                def,
                g.players[0].assets.len()
            );
        }
        g.apply(s, a.clone()).unwrap();
        actions.push((s, a));
    }
    panic!("bounded natural equipment");
}
pub fn natural_case() -> Value {
    let seed = 513;
    let actions = natural_actions(seed);
    let d = draft();
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "888888888888888888880132".into(),
            "NATURAL".into(),
            "teams".into(),
            "P0".into(),
            d.clone(),
            seed,
        )
        .unwrap(),
    );
    let mut steps = vec![step(
        &r,
        "newGameWithDeck",
        json!([
            r.room_id,
            r.invite_code,
            "teams",
            "P0",
            serde_json::to_string(&d).unwrap(),
            seed.to_string()
        ]),
        0,
    )];
    for s in 1..4 {
        r.game.join_with_deck(format!("P{s}"), d.clone()).unwrap();
        r.revision = r.game.version;
        RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();
        steps.push(step(
            &r,
            "joinGameWithDeck",
            json!([format!("P{s}"), serde_json::to_string(&d).unwrap()]),
            s,
        ));
    }
    let mut milestones = vec![];
    let mut paid = [false; 4];
    let mut water_seen = false;
    let mut twice_seen = false;
    for (s, a) in actions {
        let def = own(&r.game, s, &a).map(|c| c.definition.clone());
        let before = resources(&r.game, s);
        apply_game(&mut r, &mut steps, s, a.clone());
        if r.regions.len() < 3 {
            continue;
        }
        for (i, id) in ["JC116", "JC020", "JC093", "XQ07"].iter().enumerate() {
            if !paid[i] && r.attachments.iter().any(|x| x.card.definition == *id) {
                paid[i] = true;
                milestones.push(json!({"step":steps.len()-1,"kind":format!("{id}-actual-paid-attachment"),"turn":r.turn}));
            }
        }
        if a.kind == "play"
            && def
                .as_deref()
                .is_some_and(|id| ["JC116", "JC020", "JC093", "XQ07"].contains(&id))
        {
            let cost = usize::from(def.as_deref() == Some("JC020")) + 1;
            assert_eq!(resources(&r.game, s), before - cost);
        }
        if !water_seen && a.kind == "activate" && def.as_deref() == Some("XQ07") {
            water_seen = true;
            assert!(!r
                .attachments
                .iter()
                .any(|x| Some(&x.card.id) == a.card_id.as_ref()));
            assert!(!r.regions[2].cards.iter().any(|c| !c.face_down
                && catalog::card(&c.definition).magic_icon != rules::MagicIcon::None));
            milestones.push(json!({"step":steps.len()-1,"kind":"water-real-sacrifice-original-region-empty-magic-character-set","turn":r.turn}));
        }
        if !twice_seen && r.turn_ability_usage.iter().any(|u| u.uses == 2) {
            twice_seen = true;
            milestones.push(json!({"step":steps.len()-1,"kind":"blade-two-real-paid-declarations-same-source-same-turn","turn":r.turn}));
        }
        if a.kind == "activate" && def.as_deref() == Some("JC020") {
            milestones.push(json!({"step":steps.len()-1,"kind":a.ability_id,"turn":r.turn}));
        }
    }
    assert!(paid.iter().all(|x| *x) && water_seen && twice_seen);
    assert!(r.turn_attribute_modifiers.is_empty());
    assert!(r.turn_ability_usage.is_empty());
    json!({"name":"equipment-abilities-natural-four-seat-paid-four","seed":seed.to_string(),"syntheticInitialLayout":false,"syntheticInitialFunding":false,"postInitialStateInjection":false,"publicNaturalUiAcceptance":false,"legalFiftyCardDeck":true,"naturalWaterMagicCharacterSetEmpty":true,"finalTurn":r.turn,"milestones":milestones,"steps":steps})
}
pub fn find_natural_seed() {
    let d = draft();
    for seed in 2..1000 {
        let mut g = Game::new_with_deck(
            "offline-seed-selection".into(),
            "LOCAL".into(),
            "teams".into(),
            "P0".into(),
            d.clone(),
            seed,
        )
        .unwrap();
        for s in 1..4 {
            g.join_with_deck(format!("P{s}"), d.clone()).unwrap();
        }
        for s in 0..4 {
            g.apply(s, Action::new("ready")).unwrap();
        }
        g.apply(0, Action::new("start")).unwrap();
        if ["JC116", "JC020", "JC093", "XQ07"]
            .iter()
            .filter(|id| g.players[0].hand.iter().any(|c| c.definition == **id))
            .count()
            >= 3
            && ["JC014", "JC125"]
                .iter()
                .all(|id| g.players[0].hand.iter().any(|c| c.definition == *id))
            && g.players[0]
                .hand
                .iter()
                .any(|c| ["JC084", "JC085", "JC086", "JC088"].contains(&c.definition.as_str()))
            && std::panic::catch_unwind(|| natural_actions(seed)).is_ok()
        {
            println!("selected natural equipment seed {seed}");
            return;
        }
    }
    panic!("no bounded natural equipment seed");
}
pub fn checkpoint_cases() -> impl Iterator<Item = Value> {
    [
        "knife-source-moved",
        "knife-target-moved",
        "knife-paid-shield",
        "blade-host-moved",
        "blade-host-rebound",
        "blade-control",
        "water-source-moved",
    ]
    .into_iter()
    .map(boundary)
}
pub fn cases() -> impl Iterator<Item = Value> {
    [
        "paid-four",
        "knife-source-moved",
        "knife-target-moved",
        "knife-paid-shield",
        "blade-twice",
        "blade-source-left",
        "blade-host-left",
        "blade-host-moved",
        "blade-host-rebound",
        "blade-control",
        "water-group",
        "water-source-moved",
        "gun-win",
    ]
    .into_iter()
    .map(boundary)
    .chain(std::iter::once_with(natural_case))
    .chain(std::iter::once_with(|| boundary("controller-only-hosts")))
}
