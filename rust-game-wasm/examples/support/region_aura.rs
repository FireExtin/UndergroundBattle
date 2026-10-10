//! XQ43 exact region attachment and finite all-spirit conversion.
//! All initial layouts and primitive frame boundaries are disclosed.
use super::*;
fn fund(g: &mut Game, s: usize, definition: &str, n: usize) {
    for _ in 0..n {
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
fn board<'a>(g: &'a Game, id: &str) -> Option<&'a hegemony_server::model::Card> {
    g.regions.iter().flat_map(|r| &r.cards).find(|c| c.id == id)
}
fn initial(name: &str) -> Game {
    let mut g = attachment_initial("teams", &format!("xq43-{name}"));
    for s in 0..4 {
        g.players[s].deck.clear();
        g.players[s].assets.clear();
        for _ in 0..12 {
            let c = g.make_card("JC125", s);
            g.players[s].deck.push(c);
        }
    }
    g.regions[0].card = g.make_card("DQJC107", 0);
    fund(&mut g, 0, "JC091", 6);
    fund(&mut g, 0, "JC104", 9);
    fund(&mut g, 0, "JC073", 6);
    g
}
fn checkpoint(r: RoomEnvelope) -> (RoomEnvelope, Vec<Value>) {
    let raw = serde_json::to_string(&r).unwrap();
    assert_eq!(
        serde_json::to_string(&RoomEnvelope::from_persisted(&raw).unwrap()).unwrap(),
        raw
    );
    let steps = vec![step(&r, "initialFixture", json!([raw]), 0)];
    (r, steps)
}
fn pass(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    let (s, a) = (0..4)
        .find_map(|s| {
            r.game
                .legal_actions(s)
                .into_iter()
                .find(|a| a.action.kind == "pass")
                .map(|a| (s, a.action))
        })
        .unwrap();
    apply_game(r, steps, s, a);
}
fn advance(r: &mut RoomEnvelope, steps: &mut Vec<Value>, done: impl Fn(&RoomEnvelope) -> bool) {
    for _ in 0..150 {
        if done(r) {
            return;
        }
        assert!(r.pending.is_none(), "unexpected choice");
        pass(r, steps);
    }
    panic!("bounded XQ43 progress");
}
fn choose(r: &mut RoomEnvelope, steps: &mut Vec<Value>, ids: Vec<String>) {
    let p = r.pending.clone().unwrap();
    apply_game(
        r,
        steps,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(ids),
            ..Action::new("choose")
        },
    );
}
fn drain(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    for _ in 0..150 {
        if r.stack.is_empty() && r.pending.is_none() {
            return;
        }
        if let Some(p) = r.pending.clone() {
            let ids = p
                .choice
                .options
                .iter()
                .take(p.choice.min.unwrap_or(0))
                .map(|o| o.id.clone())
                .collect();
            choose(r, steps, ids);
        } else {
            pass(r, steps);
        }
    }
    panic!("bounded region aura stack");
}
fn play(id: &str, target: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        target_id: Some(target.into()),
        ..Action::new("play")
    }
}
fn activate(id: &str, key: &str, target: Option<&str>) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(key.into()),
        target_id: target.map(str::to_string),
        ..Action::new("activate")
    }
}
fn reject(r: &mut RoomEnvelope, steps: &mut Vec<Value>, s: usize, a: Action) {
    let c = RoomCommand {
        command_id: format!("xq43-reject:{}", steps.len()),
        expected_version: r.revision,
        action: a.into(),
    };
    let now = r.pacing.last_server_now_ms;
    let t = r.transition(s, Some(c.clone()), now).unwrap();
    assert_eq!(t.outcome, "rejected");
    assert!(!t.changed);
    record_transition(r, steps, "applyRoom", json!([s, c, now.to_string()]), t);
}
fn region_play(id: &str, region: usize) -> Action {
    Action {
        card_id: Some(id.into()),
        region: Some(region),
        ..Action::new("play")
    }
}
fn icon(g: &Game, id: &str) -> hegemony_server::model::Icons {
    let (r, c) = g
        .regions
        .iter()
        .enumerate()
        .find_map(|(r, x)| x.cards.iter().find(|c| c.id == id).map(|c| (r, c)))
        .unwrap();
    g.current_icons(c, r)
}
fn boundary(kind: &str) -> Value {
    use hegemony_server::model::{
        Effect, GuardState, Icons, ResolutionFrame, SourceSnapshot, Step,
    };
    use hegemony_server::rules::{EntityRef, Op, RegionRef};
    let mut g = initial(kind);
    let enemy = field(&mut g, "JZ58", 3, 2);
    let friend = field(&mut g, "JZ58", 0, 0);
    let human = field(&mut g, "JC049", 2, 2);
    if matches!(kind, "rear" | "ordinary-turn-bonus" | "movement-primitive") {
        g.first_team = 1;
    }
    if kind == "hidden-initial" {
        g.regions[0].cards[0].face_down = true;
    }
    if kind == "exhausted" {
        g.regions[0].cards[0].exhausted = true;
    }
    if kind == "outside-region" {
        let c = g.regions[0].cards.remove(0);
        g.regions[1].cards.push(c);
    }
    let xq = held(&mut g, "XQ43", 0);
    let net = matches!(
        kind,
        "granted-temporary" | "ordinary-turn-bonus" | "movement-primitive"
    )
    .then(|| held(&mut g, "JC073", 0));
    let blade = (kind == "ordinary-turn-bonus").then(|| held(&mut g, "JC093", 0));
    let sacrifice = (kind == "ordinary-turn-bonus").then(|| field(&mut g, "JC125", 0, 0));
    let destroy =
        matches!(kind, "destroy-source" | "ordinary-turn-bonus").then(|| held(&mut g, "JC107", 0));
    let disintegrate = kind.starts_with("jc005-").then(|| held(&mut g, "JC005", 0));
    if kind == "jc005-mind-qualified" {
        fund(&mut g, 0, "JC003", 1);
    }
    let second = if kind == "multiple-controllers" {
        fund(&mut g, 2, "JC104", 4);
        Some(held(&mut g, "XQ43", 2))
    } else if kind == "same-controller-unique" {
        Some(held(&mut g, "XQ43", 0))
    } else {
        None
    };
    let entry = if kind == "hidden-real-trigger" {
        fund(&mut g, 0, "XQ16", 4);
        Some(held(&mut g, "XQ16", 0))
    } else {
        None
    };
    if kind == "win-four-owner-return" {
        for s in 0..4 {
            field(&mut g, "JC125", s, s);
            field(&mut g, "JC049", s, s);
        }
    }
    if kind == "reject-cost" {
        g.players[0].assets.clear();
        fund(&mut g, 0, "JC104", 1);
    }
    if kind == "reject-loyalty" {
        g.players[0].assets.clear();
        fund(&mut g, 0, "JC125", 3);
    }
    let old_region = g.regions[0].card.id.clone();
    let mut preparation = vec![];
    let mut pre = RoomEnvelope::from_game(g);
    let paid_checkpoint = matches!(
        kind,
        "paid-region-replaced"
            | "dangling-exact-host"
            | "movement-primitive"
            | "win-four-owner-return"
    );
    if paid_checkpoint {
        apply_game(&mut pre, &mut preparation, 0, region_play(&xq, 0));
        if kind != "paid-region-replaced" {
            drain(&mut pre, &mut preparation);
        }
        if matches!(kind, "paid-region-replaced" | "dangling-exact-host") {
            pre.game.regions[0].card = pre.game.make_card("DQJC107", 0);
        }
        if kind == "movement-primitive" {
            apply_game(
                &mut pre,
                &mut preparation,
                0,
                play(net.as_ref().unwrap(), &friend),
            );
            drain(&mut pre, &mut preparation);
            let frame = ResolutionFrame {
                frame_id: "disclosed-movement-primitive".into(),
                ability_key: "movement-primitive".into(),
                actor: 0,
                source: SourceSnapshot {
                    observed_death: None,
                    source_region_instance: None,
                    card: board(&pre, &friend).unwrap().clone(),
                    region: Some(0),
                    attachment_host_instance: None,
                    play_source: None,
                },
                targets: vec![],
                already_paid: vec![],
                guard: GuardState::Unchecked,
                cursor: 0,
                steps: vec![Step {
                    context: 0,
                    op: Op::MoveOnBoard {
                        entity: EntityRef::Source,
                        region: RegionRef::Chosen,
                    },
                }],
                chosen_region: Some(1),
            };
            pre.game.effects.push_back(Effect::Frame {
                frame: Box::new(frame),
            });
        }
        if kind == "win-four-owner-return" {
            pre.game.window = Some(Window::Win(0, 0));
            pre.game.priority_team = 0;
            pre.game.active_team = 0;
            pre.game.passed.clear();
            pre.game.team_passed = [false; 2];
        }
    }
    let (mut r, mut steps) = checkpoint(pre);
    if kind.starts_with("reject-") {
        let a = match kind {
            "reject-region" => region_play(&xq, 99),
            "reject-host-zone" => play(&xq, &human),
            _ => region_play(&xq, 0),
        };
        reject(&mut r, &mut steps, 0, a);
    } else if kind == "paid-region-replaced" {
        for s in 0..4 {
            assert_eq!(
                r.view(s, r.pacing.last_server_now_ms)
                    .game
                    .stack
                    .last()
                    .unwrap()
                    .target_summaries[0]
                    .status,
                "missing"
            );
        }
        drain(&mut r, &mut steps);
        assert!(r.attachments.is_empty());
        assert!(r.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "XQ43"));
    } else if kind == "dangling-exact-host" {
        pass(&mut r, &mut steps);
        assert!(r.attachments.is_empty());
        assert!(r.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "XQ43"));
    } else if kind == "movement-primitive" {
        assert_eq!(icon(&r, &friend).investigation, 1);
        pass(&mut r, &mut steps);
        assert_eq!(icon(&r, &friend), Icons::default());
        assert!(r.regions[1].cards.iter().any(|c| c.id == friend));
        assert_eq!(
            r.view(0, r.pacing.last_server_now_ms)
                .game
                .attachments
                .iter()
                .find(|a| a.card.card_id.as_deref() == Some("JC073"))
                .unwrap()
                .card
                .region,
            Some(1)
        );
    } else if kind == "win-four-owner-return" {
        let aura = r.attachments[0].card.id.clone();
        let old = r.players.iter().map(|p| p.deck.len()).collect::<Vec<_>>();
        for _ in 0..4 {
            pass(&mut r, &mut steps);
        }
        let mut orders = vec![vec![]; 4];
        while r.game.region_return.is_some() {
            let p = r.pending.clone().unwrap();
            let ids = p
                .choice
                .options
                .iter()
                .rev()
                .map(|o| o.id.clone())
                .collect::<Vec<_>>();
            if p.seat == 0 {
                assert!(ids.contains(&aura));
            }
            orders[p.seat] = ids
                .iter()
                .map(|id| {
                    r.regions[0]
                        .cards
                        .iter()
                        .chain(r.attachments.iter().map(|a| &a.card))
                        .find(|c| c.id == *id)
                        .unwrap()
                        .definition
                        .clone()
                })
                .collect::<Vec<_>>();
            apply_game(
                &mut r,
                &mut steps,
                p.seat,
                Action {
                    choice_id: Some(p.choice.id),
                    bottom: Some(ids),
                    ..Action::new("choose")
                },
            );
            if r.game.region_return.is_some() {
                assert!(r.attachments.iter().any(|a| a.card.id == aura));
            }
        }
        assert!(r.attachments.is_empty());
        assert_ne!(r.regions[0].card.id, old_region);
        for s in 0..4 {
            assert_eq!(
                r.players[s].deck[old[s]..]
                    .iter()
                    .map(|c| c.definition.clone())
                    .collect::<Vec<_>>(),
                orders[s]
            );
        }
        assert!(!r.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "XQ43"));
    } else {
        let resources = r.players[0].assets.iter().filter(|c| !c.exhausted).count();
        apply_game(&mut r, &mut steps, 0, region_play(&xq, 0));
        drain(&mut r, &mut steps);
        let aura = r
            .attachments
            .iter()
            .find(|a| a.card.definition == "XQ43")
            .unwrap()
            .card
            .id
            .clone();
        assert_ne!(aura, xq);
        assert_eq!(r.attachments[0].host_id, old_region);
        assert_eq!(
            r.players[0].assets.iter().filter(|c| !c.exhausted).count(),
            resources - 2
        );
        if kind == "hidden-initial" {
            for s in 0..4 {
                let v = r.view(s, r.pacing.last_server_now_ms).game;
                assert!(v.regions[0].characters[0]
                    .converted_temporary_icons
                    .is_none());
                if s != 2 {
                    assert!(v.regions[0].characters[0].card_id.is_none());
                }
            }
        } else {
            assert_eq!(
                icon(&r, &enemy).influence,
                if kind == "outside-region" { 0 } else { 1 }
            );
            assert_eq!(icon(&r, &friend).influence, 1);
            if kind == "exhausted" {
                assert_eq!(
                    r.game.icons(board(&r, &enemy).unwrap(), 0),
                    Icons::default()
                );
            }
        }
        if matches!(kind, "granted-temporary" | "ordinary-turn-bonus") {
            apply_game(&mut r, &mut steps, 0, play(net.as_ref().unwrap(), &friend));
            drain(&mut r, &mut steps);
            assert_eq!(
                icon(&r, &friend),
                Icons {
                    investigation: 1,
                    combat: 0,
                    influence: 1
                }
            );
            if kind == "ordinary-turn-bonus" {
                apply_game(
                    &mut r,
                    &mut steps,
                    0,
                    play(blade.as_ref().unwrap(), &friend),
                );
                drain(&mut r, &mut steps);
                let blade = r
                    .attachments
                    .iter()
                    .find(|a| a.card.definition == "JC093")
                    .unwrap()
                    .card
                    .id
                    .clone();
                let mut a = activate(&blade, "sacrifice-character-host-icons", None);
                a.cost_selected = Some(vec![sacrifice.unwrap()]);
                apply_game(&mut r, &mut steps, 0, a);
                drain(&mut r, &mut steps);
                assert_eq!(
                    icon(&r, &friend),
                    Icons {
                        investigation: 1,
                        combat: 1,
                        influence: 2
                    }
                );
                apply_game(
                    &mut r,
                    &mut steps,
                    0,
                    play(destroy.as_ref().unwrap(), &aura),
                );
                drain(&mut r, &mut steps);
                assert_eq!(
                    icon(&r, &friend),
                    Icons {
                        investigation: 0,
                        combat: 1,
                        influence: 1
                    }
                );
            }
        }
        if matches!(kind, "destroy-source" | "jc005-mind-qualified") {
            let id = destroy.as_ref().or(disintegrate.as_ref()).unwrap();
            apply_game(&mut r, &mut steps, 0, play(id, &aura));
            drain(&mut r, &mut steps);
            assert!(r.attachments.is_empty());
            assert_eq!(icon(&r, &enemy), Icons::default());
        }
        if kind == "jc005-no-mind" {
            reject(
                &mut r,
                &mut steps,
                0,
                play(disintegrate.as_ref().unwrap(), &aura),
            );
        }
        if kind == "multiple-controllers" {
            advance(&mut r, &mut steps, |r| {
                r.window == Some(Window::Action(1)) && r.stack.is_empty()
            });
            apply_game(
                &mut r,
                &mut steps,
                2,
                region_play(second.as_ref().unwrap(), 0),
            );
            drain(&mut r, &mut steps);
            assert_eq!(r.attachments.len(), 2);
            assert_eq!(icon(&r, &enemy).influence, 1);
        }
        if kind == "same-controller-unique" {
            apply_game(
                &mut r,
                &mut steps,
                0,
                region_play(second.as_ref().unwrap(), 0),
            );
            drain(&mut r, &mut steps);
            assert_eq!(r.attachments.len(), 1);
            assert_eq!(icon(&r, &enemy).influence, 1);
        }
        if kind == "hidden-real-trigger" {
            apply_game(
                &mut r,
                &mut steps,
                0,
                Action {
                    card_id: Some(entry.unwrap()),
                    region: Some(0),
                    ..Action::new("deploy")
                },
            );
            advance(&mut r, &mut steps, |r| r.pending.is_some());
            choose(&mut r, &mut steps, vec![friend.clone()]);
            drain(&mut r, &mut steps);
            assert!(board(&r, &friend).is_none());
            let hidden = r.regions[0]
                .cards
                .iter()
                .find(|c| c.definition == "JZ58" && c.controller == 0)
                .unwrap();
            assert!(hidden.face_down);
            assert_ne!(hidden.id, friend);
            for s in 0..4 {
                let v = r.view(s, r.pacing.last_server_now_ms).game;
                let c = v.regions[0]
                    .characters
                    .iter()
                    .find(|c| c.instance_id == hidden.id)
                    .unwrap();
                assert!(c.converted_temporary_icons.is_none());
                if s != 0 {
                    assert!(c.card_id.is_none() && c.icons.is_none());
                }
            }
        }
    }
    json!({"name":format!("XQ43-{kind}"),"seed":"9007199254740993","steps":steps,
        "syntheticInitialLayout":true,"syntheticPaidCheckpoint":paid_checkpoint,
        "syntheticRegionReplacement":matches!(kind,"paid-region-replaced"|"dangling-exact-host"),
        "syntheticMovementPrimitiveFrame":kind=="movement-primitive","syntheticWinWindow":kind=="win-four-owner-return",
        "syntheticCardRegistry":false,"publicNaturalUiAcceptance":false,"ordinaryTurnBonusesReclassified":false})
}
fn natural_deck() -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    d.id = "purple-region-natural".into();
    d.name = "梦境行者具现化有限正常链".into();
    d.society_id = Some("MSJC08".into());
    d.cards = [
        "JC104", "JC102", "JZ67", "JC103", "JC107", "JZ59", "JZ58", "JZ61", "XQ43", "JC073",
    ]
    .into_iter()
    .map(|id| catalog::DeckEntry {
        card_id: id.into(),
        count: 3,
    })
    .collect();
    d.cards.push(catalog::DeckEntry {
        card_id: "JC125".into(),
        count: 20,
    });
    deck::validate(d).unwrap()
}
pub(super) fn find_natural_seed() {
    let d = natural_deck();
    for seed in 9007199254744000..9007199254746000 {
        let mut g = Game::new_with_deck(
            "seed-check".into(),
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
        for p in &mut g.players {
            p.ready = true;
        }
        g.apply(0, Action::new("start")).unwrap();
        let early = |s: usize| {
            g.players[s]
                .hand
                .iter()
                .chain(g.players[s].deck.iter().take(4))
                .map(|c| c.definition.as_str())
                .collect::<Vec<_>>()
        };
        let p0 = early(0);
        let p2 = early(2);
        if p0.iter().filter(|id| **id == "JC073").count() >= 2
            && p0.contains(&"JZ58")
            && p2.contains(&"JZ58")
        {
            println!(
                "{}",
                json!({"selectedSeed":seed.to_string(),"firstTenPrintedCardsSeat0":p0,"firstTenPrintedCardsSeat2":p2,"selection":"read-only search over normal legal deck deal, no post-start edits"})
            );
            return;
        }
    }
    panic!("bounded legal-deal seed search");
}
pub(super) fn natural() -> Value {
    let seed = "9007199254744004";
    let d = natural_deck();
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "purple-region-natural".into(),
            "LOCAL".into(),
            "teams".into(),
            "P0".into(),
            d.clone(),
            seed.parse().unwrap(),
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
            seed
        ]),
        0,
    )];
    for s in 1..4 {
        r.game.join_with_deck(format!("P{s}"), d.clone()).unwrap();
        r.revision = r.game.version;
        steps.push(step(
            &r,
            "joinGameWithDeck",
            json!([format!("P{s}"), serde_json::to_string(&d).unwrap()]),
            s,
        ));
    }
    for s in 0..4 {
        apply_game(&mut r, &mut steps, s, Action::new("ready"));
    }
    apply_game(&mut r, &mut steps, 0, Action::new("start"));
    assert!(r.players.iter().all(|p| p.hand.len() == 6));
    let mut searches = vec![];
    let mut paid = vec![];
    let mut roles = [false; 2];
    let mut concealed_id = None;
    let mut revealed_id = None;
    let mut privacy_seen = false;
    let mut complete = false;
    for _ in 0..2600 {
        roles[r.first_team] = true;
        if let Some(p) = r.pending.clone() {
            if p.choice.kind == "search" {
                for s in 0..4 {
                    assert_eq!(
                        r.view(s, r.pacing.last_server_now_ms)
                            .pending_choice
                            .is_some(),
                        s == p.seat
                    );
                }
                let id = p.choice.options.first().unwrap().id.clone();
                let c = r.players[p.seat].deck.iter().find(|c| c.id == id).unwrap();
                assert_eq!(c.definition, "XQ43");
                let old = id.clone();
                choose(&mut r, &mut steps, vec![id]);
                let fresh = r.players[p.seat].hand.last().unwrap();
                assert_ne!(fresh.id, old);
                searches.push(json!({"seat":p.seat,"oldInstance":old,"newInstance":fresh.id,"cardId":"XQ43","fourSeatPrivateChoice":true}));
            } else if p.choice.kind == "discard" {
                let mut seen = std::collections::BTreeSet::new();
                let mut options = p
                    .choice
                    .options
                    .iter()
                    .map(|o| {
                        let c = r.players[p.seat]
                            .hand
                            .iter()
                            .find(|c| c.id == o.id)
                            .unwrap();
                        let essential = ["XQ43", "JZ58", "JC073"].contains(&c.definition.as_str());
                        let first = seen.insert(c.definition.clone());
                        (if essential && first { 10 } else { 0 }, o.id.clone())
                    })
                    .collect::<Vec<_>>();
                options.sort_by_key(|x| x.0);
                choose(
                    &mut r,
                    &mut steps,
                    options
                        .into_iter()
                        .take(p.choice.min.unwrap())
                        .map(|x| x.1)
                        .collect(),
                );
            } else {
                let (s, a) = pick_choice(&r.game);
                apply_game(&mut r, &mut steps, s, a);
            }
            continue;
        }
        if !r.stack.is_empty() {
            pass(&mut r, &mut steps);
            continue;
        }
        let auras = r
            .attachments
            .iter()
            .filter(|a| a.card.definition == "XQ43" && a.host_id == r.regions[2].card.id)
            .collect::<Vec<_>>();
        let own = r.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "JZ58" && c.controller == 0);
        let other = r.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "JZ58" && c.controller == 2 && !c.face_down);
        if let Some(c) = own.filter(|c| c.face_down) {
            privacy_seen = true;
            concealed_id = Some(c.id.clone());
            for s in 0..4 {
                let v = r.view(s, r.pacing.last_server_now_ms).game;
                let projected = v.regions[2]
                    .characters
                    .iter()
                    .find(|x| x.instance_id == c.id)
                    .unwrap();
                assert!(projected.converted_temporary_icons.is_none());
                assert_eq!(projected.card_id.is_some(), s == 0);
            }
        }
        if let (Some(c), Some(other)) = (own.filter(|c| !c.face_down), other) {
            revealed_id = Some(c.id.clone());
            let net = r
                .attachments
                .iter()
                .any(|a| a.card.definition == "JC073" && a.host_id == c.id);
            if auras.len() == 2
                && net
                && searches.len() == 4
                && roles == [true, true]
                && r.first_team == 0
            {
                for s in 0..4 {
                    let v = r.view(s, r.pacing.last_server_now_ms).game;
                    for id in [&c.id, &other.id] {
                        let x = v.regions[2]
                            .characters
                            .iter()
                            .find(|x| x.instance_id == *id)
                            .unwrap();
                        assert_eq!(x.converted_temporary_icons.unwrap().influence, 1);
                        assert_eq!(x.icons.unwrap().influence, 1);
                    }
                    assert_eq!(
                        v.regions[2]
                            .characters
                            .iter()
                            .find(|x| x.instance_id == c.id)
                            .unwrap()
                            .converted_temporary_icons
                            .unwrap()
                            .investigation,
                        1
                    );
                }
                assert!(privacy_seen);
                assert_ne!(concealed_id, revealed_id);
                complete = true;
                break;
            }
        }
        let mut acted = false;
        for s in 0..4 {
            let legal = r.game.legal_actions(s);
            let assets = &r.players[s].assets;
            let purple = assets
                .iter()
                .any(|c| catalog::card(&c.definition).color == "紫");
            let white = assets
                .iter()
                .any(|c| catalog::card(&c.definition).color == "白");
            if assets.len() < 4 || s == 0 && !white {
                let candidate = legal
                    .iter()
                    .filter(|a| a.action.kind == "asset")
                    .filter_map(|a| {
                        r.players[s]
                            .hand
                            .iter()
                            .find(|c| Some(&c.id) == a.action.card_id.as_ref())
                            .map(|c| (a, c))
                    })
                    .filter(|(_, c)| !["XQ43", "JZ58"].contains(&c.definition.as_str()))
                    .min_by_key(|(_, c)| {
                        if !purple && catalog::card(&c.definition).color == "紫" {
                            0
                        } else if s == 0 && !white && c.definition == "JC073" {
                            1
                        } else if c.definition == "JC125" {
                            2
                        } else {
                            3
                        }
                    });
                if let Some((a, _)) = candidate {
                    apply_game(&mut r, &mut steps, s, a.action.clone());
                    acted = true;
                    break;
                }
            }
            if let Some(a) = legal
                .iter()
                .find(|a| a.action.ability_id.as_deref() == Some("search-purple-unique"))
            {
                apply_game(&mut r, &mut steps, s, a.action.clone());
                acted = true;
                break;
            }
            if s == 0 || s == 2 {
                let aura_present = r
                    .attachments
                    .iter()
                    .any(|a| a.card.definition == "XQ43" && a.card.controller == s);
                if !aura_present {
                    if let Some(a) = legal.iter().find(|a| {
                        a.action.kind == "play"
                            && a.action.region == Some(2)
                            && r.players[s].hand.iter().any(|c| {
                                c.definition == "XQ43" && Some(&c.id) == a.action.card_id.as_ref()
                            })
                    }) {
                        let old = a.action.card_id.clone().unwrap();
                        let resources = r.players[s].assets.iter().filter(|c| !c.exhausted).count();
                        apply_game(&mut r, &mut steps, s, a.action.clone());
                        assert_eq!(
                            r.players[s].assets.iter().filter(|c| !c.exhausted).count(),
                            resources - 2
                        );
                        paid.push(json!({"seat":s,"sourceHandInstance":old,"cost":2,"regionInstance":r.regions[2].card.id}));
                        acted = true;
                        break;
                    }
                }
                let spirit = r.regions[2]
                    .cards
                    .iter()
                    .find(|c| c.definition == "JZ58" && c.controller == s);
                let prepared = r
                    .attachments
                    .iter()
                    .filter(|a| a.card.definition == "XQ43")
                    .count()
                    == 2
                    && [0, 2].into_iter().all(|actor| {
                        r.players[actor].hand.iter().any(|c| c.definition == "JZ58")
                            || r.regions[2]
                                .cards
                                .iter()
                                .any(|c| c.definition == "JZ58" && c.controller == actor)
                    })
                    && (r.players[0].hand.iter().any(|c| c.definition == "JC073")
                        || r.attachments.iter().any(|a| a.card.definition == "JC073"))
                    && r.players[0]
                        .assets
                        .iter()
                        .any(|c| catalog::card(&c.definition).color == "白");
                if spirit.is_none() && prepared {
                    if let Some(a) = legal.iter().find(|a| {
                        a.action.kind == if s == 0 { "conceal" } else { "deploy" }
                            && a.action.region == Some(2)
                            && r.players[s].hand.iter().any(|c| {
                                c.definition == "JZ58" && Some(&c.id) == a.action.card_id.as_ref()
                            })
                    }) {
                        apply_game(&mut r, &mut steps, s, a.action.clone());
                        acted = true;
                        break;
                    }
                } else if s == 0 && spirit.is_some() {
                    let spirit = spirit.unwrap();
                    if spirit.face_down
                        && r.attachments
                            .iter()
                            .filter(|a| a.card.definition == "XQ43")
                            .count()
                            == 2
                    {
                        if let Some(a) = legal.iter().find(|a| {
                            a.action.kind == "reveal"
                                && a.action.card_id.as_deref() == Some(&spirit.id)
                        }) {
                            apply_game(&mut r, &mut steps, s, a.action.clone());
                            acted = true;
                            break;
                        }
                    } else if !spirit.face_down
                        && !r
                            .attachments
                            .iter()
                            .any(|a| a.card.definition == "JC073" && a.host_id == spirit.id)
                    {
                        if let Some(a) = legal.iter().find(|a| {
                            a.action.kind == "play"
                                && a.action.target_id.as_deref() == Some(&spirit.id)
                                && r.players[s].hand.iter().any(|c| {
                                    c.definition == "JC073"
                                        && Some(&c.id) == a.action.card_id.as_ref()
                                })
                        }) {
                            apply_game(&mut r, &mut steps, s, a.action.clone());
                            acted = true;
                            break;
                        }
                    }
                }
                if r.team(s) == r.first_team {
                    if let Some(a) = legal
                        .iter()
                        .find(|a| a.action.ability_id.as_deref() == Some("drawWithInitiative"))
                    {
                        apply_game(&mut r, &mut steps, s, a.action.clone());
                        acted = true;
                        break;
                    }
                }
            }
        }
        if !acted {
            pass(&mut r, &mut steps);
        }
    }
    assert!(
        complete,
        "bounded normal XQ43 chain failed at turn {}, searches {}, paid {:?}",
        r.turn,
        searches.len(),
        paid
    );
    assert_eq!(paid.len(), 2);
    json!({"name":"printed-MSJC08-XQ43-natural-four-seat-paid-region-conceal-reveal-granted-icons-restore",
        "seed":seed,"steps":steps,"searches":searches,"paidRegionPlays":paid,"concealedInstance":concealed_id,"revealedInstance":revealed_id,
        "ordinaryDeckSize":50,"purpleDeckCount":27,"startingHand":6,"syntheticInitialLayout":false,"syntheticCardRegistry":false,
        "normalRoomCommandsOnly":true,"seedSelection":"read-only normal legal-deal search; no post-start state edits","fourSeatConcealedPrivacy":privacy_seen,"bothInitiativeTeamsObserved":roles,
        "publicNaturalUiAcceptance":false})
}
pub(super) fn cases() -> impl Iterator<Item = Value> {
    [
        "first",
        "rear",
        "hidden-initial",
        "exhausted",
        "granted-temporary",
        "ordinary-turn-bonus",
        "reject-cost",
        "reject-loyalty",
        "reject-region",
        "reject-host-zone",
        "paid-region-replaced",
        "dangling-exact-host",
        "movement-primitive",
        "destroy-source",
        "jc005-no-mind",
        "jc005-mind-qualified",
        "multiple-controllers",
        "same-controller-unique",
        "hidden-real-trigger",
        "win-four-owner-return",
        "outside-region",
    ]
    .into_iter()
    .map(boundary)
    .chain(std::iter::once_with(natural))
}
