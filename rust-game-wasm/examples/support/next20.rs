//! Explicit initial layouts; recorded play, choices, responses and recovery use
//! the existing Room oracle. These are not natural browser acceptance games.
use super::*;
use hegemony_server::model::Card;

fn layout(name: &str) -> Game {
    let mut g = attachment_initial("teams", &format!("next20-{name}"));
    g.regions[0].card = g.make_card("DQJC115", 0);
    for r in &mut g.regions {
        r.influence = [0, 0];
    }
    for p in &mut g.players {
        p.graveyard.clear();
    }
    g
}
fn fund(g: &mut Game, seat: usize, definition: &str, n: usize) {
    for _ in 0..n {
        let c = g.make_card(definition, seat);
        g.players[seat].assets.push(c);
    }
}
fn held(g: &mut Game, definition: &str, seat: usize) -> String {
    let c = g.make_card(definition, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn field(g: &mut Game, definition: &str, seat: usize, region: usize) -> String {
    let c = g.make_card(definition, seat);
    let id = c.id.clone();
    g.regions[region].cards.push(c);
    id
}
fn grave(g: &mut Game, definition: &str, seat: usize) -> String {
    let c = g.make_card(definition, seat);
    let id = c.id.clone();
    g.players[seat].graveyard.push(c);
    id
}
fn deck(g: &mut Game, seat: usize, definitions: &[&str]) -> Vec<String> {
    let cards = definitions
        .iter()
        .map(|d| g.make_card(d, seat))
        .collect::<Vec<_>>();
    let ids = cards.iter().map(|c| c.id.clone()).collect();
    g.players[seat].deck = cards;
    ids
}
fn board<'a>(r: &'a RoomEnvelope, id: &str) -> Option<(usize, &'a Card)> {
    r.regions
        .iter()
        .enumerate()
        .find_map(|(i, region)| region.cards.iter().find(|c| c.id == id).map(|c| (i, c)))
}
fn ready(r: &RoomEnvelope, seat: usize) -> usize {
    r.players[seat]
        .assets
        .iter()
        .filter(|c| !c.exhausted)
        .count()
}
fn initial(g: Game) -> (RoomEnvelope, Vec<Value>) {
    let r = RoomEnvelope::from_game(g);
    let state = serde_json::to_string(&r).unwrap();
    let r = RoomEnvelope::from_persisted(&state).unwrap();
    let steps = vec![step(&r, "initialFixture", json!([state]), 0)];
    (r, steps)
}
fn result(name: &str, steps: Vec<Value>, cards: &[&str]) -> Value {
    json!({"name":format!("next20-{name}"),"seed":"9007199254740993","steps":steps,
        "syntheticInitialLayout":true,"syntheticCardRegistry":false,"publicNaturalUiAcceptance":false,
        "postInitialStateInjection":false,"effectCards":cards})
}
fn pass(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    let (seat, a) = (0..r.players.len())
        .find_map(|s| {
            r.game
                .legal_actions(s)
                .into_iter()
                .find(|a| a.action.kind == "pass")
                .map(|a| (s, a.action))
        })
        .expect("legal pass");
    apply_game(r, steps, seat, a);
}
fn until(r: &mut RoomEnvelope, steps: &mut Vec<Value>, kind: &str) {
    for _ in 0..80 {
        if let Some(p) = &r.pending {
            assert_eq!(p.choice.kind, kind);
            return;
        }
        pass(r, steps);
    }
    panic!("missing {kind}");
}
fn choose(r: &mut RoomEnvelope, steps: &mut Vec<Value>, selected: Vec<String>) {
    let p = r.pending.clone().expect("choice");
    apply_game(
        r,
        steps,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(selected),
            ..Action::new("choose")
        },
    );
}
fn settle(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    for _ in 0..80 {
        if let Some(p) = &r.pending {
            if p.choice.kind == "trigger" && p.choice.min == Some(0) {
                choose(r, steps, vec![]);
            } else {
                let (s, a) = pick_choice(&r.game);
                apply_game(r, steps, s, a);
            }
        } else if r.stack.is_empty() {
            return;
        } else {
            attachment_pass_top(r, steps);
        }
    }
    panic!("bounded next20 resolution");
}
fn activate(
    r: &mut RoomEnvelope,
    steps: &mut Vec<Value>,
    seat: usize,
    source: &str,
    key: &str,
    target: Option<&str>,
) {
    let a = r
        .game
        .legal_actions(seat)
        .into_iter()
        .find(|a| {
            a.action.kind == "activate"
                && a.action.card_id.as_deref() == Some(source)
                && a.action.ability_id.as_deref() == Some(key)
                && target.is_none_or(|t| a.action.target_id.as_deref() == Some(t))
        })
        .expect("offered activation")
        .action;
    apply_game(r, steps, seat, a);
}
fn reject(r: &mut RoomEnvelope, steps: &mut Vec<Value>, seat: usize, action: Action) {
    let command = RoomCommand {
        command_id: format!("next20:reject:{}", steps.len()),
        expected_version: r.revision,
        action: action.into(),
    };
    let now = r.pacing.last_server_now_ms;
    let t = r.transition(seat, Some(command.clone()), now).unwrap();
    assert_eq!(t.outcome, "rejected");
    assert!(!t.changed);
    record_transition(
        r,
        steps,
        "applyRoom",
        json!([seat, command, now.to_string()]),
        t,
    );
}
fn repeat_last(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    let entry = steps.last().unwrap();
    let seat = entry["args"][0].as_u64().unwrap() as usize;
    let command: RoomCommand = serde_json::from_value(entry["args"][1].clone()).unwrap();
    let now = r.pacing.last_server_now_ms;
    let t = r.transition(seat, Some(command.clone()), now).unwrap();
    assert_eq!(t.error_code.as_deref(), Some("version_conflict"));
    assert!(!t.changed);
    record_transition(
        r,
        steps,
        "applyRoom",
        json!([seat, command, now.to_string()]),
        t,
    );
}
fn private(r: &RoomEnvelope, seat: usize, ids: &[String]) {
    for viewer in 0..r.players.len() {
        if viewer != seat {
            let v = r.view(viewer, r.pacing.last_server_now_ms);
            assert!(v.game.pending_choice.is_none());
            let raw = serde_json::to_string(&v).unwrap();
            for id in ids {
                assert!(
                    !raw.contains(id),
                    "private instance leaked to seat {viewer}"
                );
            }
        }
    }
}
fn trigger_source(r: &RoomEnvelope) -> &str {
    let ChoiceResolution::Declare { declaration, .. } = &r.pending.as_ref().unwrap().resolution
    else {
        panic!("trigger declaration")
    };
    &declaration.source.card.definition
}

fn mixed_preparation() -> Value {
    let mut g = layout("shared-preparation");
    let red = field(&mut g, "JC055", 0, 0);
    let black = field(&mut g, "BQ085", 0, 0);
    let blue = field(&mut g, "CQ08", 0, 0);
    field(&mut g, "CQ08", 0, 0);
    field(&mut g, "CQ08", 0, 1);
    let a = grave(&mut g, "JC125", 0);
    let b = grave(&mut g, "JZ50", 0);
    let ids = deck(&mut g, 0, &["JC125", "JC125", "JC125", "JZ28"]);
    g.window = Some(Window::End);
    // This initial checkpoint has entered End with no JC054 on the board.
    g.jc054_end_step_turn = Some(g.turn);
    let (mut r, mut steps) = initial(g);
    until(&mut r, &mut steps, "jc035-preparation-order");
    assert_eq!(r.pending.as_ref().unwrap().choice.options.len(), 5);
    let choice = r.pending.as_ref().unwrap().choice.id.clone();
    reject(
        &mut r,
        &mut steps,
        0,
        Action {
            choice_id: Some(choice),
            selected: Some(vec![]),
            ..Action::new("choose")
        },
    );
    choose(&mut r, &mut steps, vec![red.clone()]);
    assert_eq!(
        r.pending.as_ref().unwrap().choice.kind,
        "jc055-preparation-target"
    );
    choose(&mut r, &mut steps, vec![red.clone()]);
    choose(&mut r, &mut steps, vec![blue]);
    assert_eq!(r.pending.as_ref().unwrap().choice.options.len(), 1);
    choose(&mut r, &mut steps, vec![black.clone()]);
    until(&mut r, &mut steps, "bq085-bottom");
    choose(&mut r, &mut steps, vec![b, a]);
    repeat_last(&mut r, &mut steps);
    until(&mut r, &mut steps, "next-blue-gray-search");
    private(&r, 0, &ids[3..]);
    choose(&mut r, &mut steps, vec![ids[3].clone()]);
    settle(&mut r, &mut steps);
    assert_eq!(board(&r, &red).unwrap().1.time_markers, 1);
    assert!(board(&r, &black).is_some());
    assert!(r.players[0].graveyard.is_empty());
    assert_eq!(
        r.regions[0]
            .cards
            .iter()
            .filter(|c| c.definition == "JZ28" && !c.face_down)
            .count(),
        1
    );
    let entries = &r.jc035_preparation.as_ref().unwrap().entries;
    assert_eq!(
        entries
            .iter()
            .filter(|e| e.source.card.definition == "CQ08"
                && serde_json::to_value(&e.decision)
                    .unwrap()
                    .get("Resolved")
                    .is_some())
            .count(),
        1
    );
    result(
        "shared-forced-preparation-and-cq08-once",
        steps,
        &["JC055", "BQ085", "CQ08"],
    )
}

fn reveal_rumor(sealed: bool) -> Value {
    let mut g = layout("reveal-rumor");
    field(&mut g, "JZ23", 0, 0);
    let vampire = field(&mut g, "JC029", 0, 0);
    g.regions[0].cards.last_mut().unwrap().face_down = true;
    let host = field(&mut g, "JC125", 2, 0);
    let news = held(&mut g, "WM042", 2);
    fund(&mut g, 0, "JC029", 8);
    let (mut r, mut steps) = initial(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(vampire),
            ..Action::new("reveal")
        },
    );
    until(&mut r, &mut steps, "trigger");
    assert_eq!(trigger_source(&r), "JZ23");
    let before = ready(&r, 0);
    choose(&mut r, &mut steps, vec!["p2".into()]);
    assert_eq!(ready(&r, 0), before - 1);
    // The revealed JC029 also offers its independent optional raid.
    assert_eq!(trigger_source(&r), "JC029");
    choose(&mut r, &mut steps, vec![]);
    until(&mut r, &mut steps, "discard");
    private(&r, 2, &[news.clone()]);
    choose(&mut r, &mut steps, vec![news]);
    assert_eq!(r.pending.as_ref().unwrap().choice.kind, "rumor");
    private(&r, 2, &[]);
    choose(&mut r, &mut steps, if sealed { vec![host] } else { vec![] });
    repeat_last(&mut r, &mut steps);
    settle(&mut r, &mut steps);
    assert_eq!(ready(&r, 0), before - 1);
    let c = r.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC029")
        .unwrap();
    assert_eq!(c.permanent_combat_markers, 1);
    for s in 0..4 {
        assert_eq!(
            r.view(s, r.pacing.last_server_now_ms).game.regions[0]
                .characters
                .iter()
                .find(|v| v.instance_id == c.id)
                .unwrap()
                .permanent_combat_markers,
            Some(1)
        );
    }
    assert_eq!(
        r.sealed_cards
            .iter()
            .filter(|s| s.card.definition == "WM042")
            .count(),
        usize::from(sealed)
    );
    assert_eq!(
        r.players[2]
            .graveyard
            .iter()
            .filter(|c| c.definition == "WM042")
            .count(),
        usize::from(!sealed)
    );
    result(
        if sealed {
            "jz23-paid-marker-rumor-sealed"
        } else {
            "jz23-paid-marker-rumor-declined"
        },
        steps,
        &["JZ23", "WM042"],
    )
}

fn mill_rumor_discard() -> Value {
    let mut g = layout("mill-rumor-discard");
    let angel = field(&mut g, "CQ26", 0, 0);
    field(&mut g, "JZ28", 2, 0);
    let hand_news = held(&mut g, "WM042", 0);
    deck(&mut g, 0, &["JC125", "WM042", "JC125", "JC125", "JC084"]);
    // XQ36 mills every player's top four before this later discard search.
    let inspected = deck(
        &mut g,
        2,
        &[
            "JC125", "JC125", "JC125", "JC125", "JC125", "JC029", "JC125", "JC125", "CQ08",
        ],
    )[4..]
        .to_vec();
    fund(&mut g, 0, "JC084", 1);
    let mill = held(&mut g, "XQ36", 0);
    let (mut r, mut steps) = initial(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(mill),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    until(&mut r, &mut steps, "trigger");
    choose(&mut r, &mut steps, vec!["accept".into()]);
    until(&mut r, &mut steps, "rumor");
    choose(&mut r, &mut steps, vec![angel]);
    until(&mut r, &mut steps, "trigger");
    assert_eq!(trigger_source(&r), "CQ26");
    assert_eq!(r.players[0].graveyard.len(), 3);
    let ChoiceResolution::Declare { declaration, .. } = &r.pending.as_ref().unwrap().resolution
    else {
        unreachable!()
    };
    assert!(
        matches!(&declaration.ability.ops[0],Op::BlackDeckToGraveyard {observation:Some(o)} if o.cards.len()==3&&o.cards.iter().all(|c|c.definition=="JC125"))
    );
    choose(&mut r, &mut steps, vec!["accept".into()]);
    until(&mut r, &mut steps, "discard");
    choose(&mut r, &mut steps, vec![hand_news]);
    assert_eq!(r.pending.as_ref().unwrap().choice.kind, "rumor");
    choose(&mut r, &mut steps, vec![]);
    assert_eq!(trigger_source(&r), "JZ28");
    choose(&mut r, &mut steps, vec!["accept".into()]);
    until(&mut r, &mut steps, "next-blue-gray-search");
    assert_eq!(r.pending.as_ref().unwrap().choice.options.len(), 1);
    private(&r, 2, &inspected);
    assert_eq!(
        r.view(2, r.pacing.last_server_now_ms)
            .game
            .pending_choice
            .unwrap()
            .preview_cards
            .len(),
        4
    );
    choose(&mut r, &mut steps, vec![inspected[1].clone()]);
    repeat_last(&mut r, &mut steps);
    settle(&mut r, &mut steps);
    assert!(r.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "JC029" && c.controller == 2 && c.face_down));
    assert_eq!(r.players[0].graveyard.len(), 4);
    assert_eq!(r.sealed_cards.len(), 1);
    assert!(r.players[0].hand.iter().any(|c| c.definition == "JC084"));
    result(
        "actual-mill-rumor-cq26-discard-jz28-top-four",
        steps,
        &["CQ26", "WM042", "JZ28"],
    )
}

fn fish_mill() -> Value {
    let mut g = layout("fish-mill");
    g.regions[0].card = g.make_card("DQJC112", 0);
    g.regions[1].card = g.make_card("DQJC107", 0);
    let old = deck(&mut g, 0, &["BQ077"])[0].clone();
    fund(&mut g, 0, "JC084", 1);
    let mill = held(&mut g, "XQ36", 0);
    let (mut r, mut steps) = initial(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(mill),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    until(&mut r, &mut steps, "trigger");
    choose(&mut r, &mut steps, vec!["accept".into()]);
    until(&mut r, &mut steps, "trigger");
    assert_eq!(trigger_source(&r), "BQ077");
    assert!(!r
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .any(|o| o.id == "region:0"));
    let choice = r.pending.as_ref().unwrap().choice.id.clone();
    reject(
        &mut r,
        &mut steps,
        0,
        Action {
            choice_id: Some(choice),
            selected: Some(vec!["region:0".into()]),
            ..Action::new("choose")
        },
    );
    choose(&mut r, &mut steps, vec!["region:1".into()]);
    attachment_pass_top(&mut r, &mut steps);
    let c = r.regions[1]
        .cards
        .iter()
        .find(|c| c.definition == "BQ077")
        .unwrap();
    assert_ne!(c.id, old);
    assert!(!c.face_down && !c.exhausted);
    assert!(r.players[0].graveyard.is_empty());
    result("bq077-real-deck-to-grave-water-entry", steps, &["BQ077"])
}

fn sacrifice_death() -> Value {
    let mut g = layout("sacrifice-death");
    field(&mut g, "BQ085", 0, 0);
    let ghoul = grave(&mut g, "JZ50", 0);
    let spirit = grave(&mut g, "XG27", 0);
    let sacrifice = field(&mut g, "JC125", 0, 0);
    let victim = field(&mut g, "JC125", 2, 2);
    let butcher = field(&mut g, "JC094", 0, 0);
    fund(&mut g, 0, "JC084", 1);
    let (mut r, mut steps) = initial(g);
    let a = r
        .game
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.card_id.as_deref() == Some(&butcher)
                && a.action.target_id.as_deref() == Some(&victim)
                && a.action.cost_selected.as_ref() == Some(&vec![sacrifice.clone()])
        })
        .unwrap()
        .action;
    apply_game(&mut r, &mut steps, 0, a);
    assert_eq!(trigger_source(&r), "BQ085");
    choose(&mut r, &mut steps, vec![ghoul.clone()]);
    attachment_pass_top(&mut r, &mut steps);
    assert!(r.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "JZ50" && c.exhausted && c.id != ghoul));
    until(&mut r, &mut steps, "trigger");
    assert_eq!(trigger_source(&r), "XG27");
    choose(&mut r, &mut steps, vec!["accept".into()]);
    assert_eq!(ready(&r, 0), 0);
    attachment_pass_top(&mut r, &mut steps);
    settle(&mut r, &mut steps);
    let c = r.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "XG27")
        .unwrap();
    assert!(c.face_down && c.id != spirit);
    for s in 1..4 {
        assert!(r.view(s, r.pacing.last_server_now_ms).game.regions[2]
            .characters
            .iter()
            .find(|v| v.instance_id == c.id)
            .unwrap()
            .card_id
            .is_none());
    }
    result(
        "bq085-sacrifice-and-xg27-victim-region-recovery",
        steps,
        &["BQ085", "XG27"],
    )
}

fn badge_discount(definition: &str, reveal: bool) -> Value {
    let mut g = layout("badge-discount");
    let blue = definition == "JC039";
    fund(&mut g, 0, if blue { "JC029" } else { "JC084" }, 10);
    let badge = held(&mut g, definition, 0);
    let discounted = if blue { "JC029" } else { "BQ077" };
    let target = if reveal {
        let id = field(&mut g, discounted, 0, 0);
        g.regions[0].cards.last_mut().unwrap().face_down = true;
        id
    } else {
        held(&mut g, discounted, 0)
    };
    let (mut r, mut steps) = initial(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(badge),
            target_id: Some("society:p0".into()),
            ..Action::new("play")
        },
    );
    attachment_pass_top(&mut r, &mut steps);
    let live = r.attachments[0].card.id.clone();
    activate(
        &mut r,
        &mut steps,
        0,
        &live,
        if blue {
            "next-bg-jc039-discount"
        } else {
            "black20-xq39-discount"
        },
        None,
    );
    attachment_pass_top(&mut r, &mut steps);
    let before = ready(&r, 0);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(target),
            region: (!reveal).then_some(0),
            ..Action::new(if reveal { "reveal" } else { "deploy" })
        },
    );
    assert_eq!(
        ready(&r, 0),
        before - catalog::card(discounted).cost.saturating_sub(1) as usize
    );
    repeat_last(&mut r, &mut steps);
    settle(&mut r, &mut steps);
    assert!(r.modifiers.iter().all(|m| m.uses == 0));
    assert!(r.players[0].society_zone.card.is_none());
    assert!(r.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == discounted && !c.face_down));
    result(
        &format!(
            "{definition}-empty-zone-discount-{}",
            if reveal { "paid-reveal" } else { "play" }
        ),
        steps,
        &[definition],
    )
}

fn badge_recover() -> Value {
    let mut g = layout("badge-recover");
    fund(&mut g, 0, "JC084", 4);
    let badge = held(&mut g, "XQ39", 0);
    let target = grave(&mut g, "JZ50", 0);
    let foreign = grave(&mut g, "JZ50", 2);
    let (mut r, mut steps) = initial(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(badge),
            target_id: Some("society:p0".into()),
            ..Action::new("play")
        },
    );
    attachment_pass_top(&mut r, &mut steps);
    let id = r.attachments[0].card.id.clone();
    let before = r.players[0].deck.clone();
    let rng = r.random;
    reject(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(id.clone()),
            ability_id: Some("black20-xq39-recover".into()),
            target_id: Some(foreign),
            ..Action::new("activate")
        },
    );
    activate(
        &mut r,
        &mut steps,
        0,
        &id,
        "black20-xq39-recover",
        Some(&target),
    );
    attachment_pass_top(&mut r, &mut steps);
    assert_eq!(ready(&r, 0), 0);
    assert_eq!(r.players[0].deck[0].definition, "JZ50");
    assert_ne!(r.players[0].deck[0].id, target);
    assert_eq!(&r.players[0].deck[1..], before.as_slice());
    assert_eq!(r.random, rng);
    result("xq39-own-grave-to-top-no-shuffle", steps, &["XQ39"])
}

fn blue_hide() -> Value {
    let mut g = layout("blue-hide");
    fund(&mut g, 0, "JC029", 8);
    let badge = held(&mut g, "JC039", 0);
    let own = field(&mut g, "JC029", 0, 1);
    g.regions[1]
        .cards
        .last_mut()
        .unwrap()
        .permanent_combat_markers = 2;
    let teammate = field(&mut g, "JC029", 1, 1);
    let (mut r, mut steps) = initial(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(badge),
            target_id: Some("society:p0".into()),
            ..Action::new("play")
        },
    );
    attachment_pass_top(&mut r, &mut steps);
    let source = r.attachments[0].card.id.clone();
    reject(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(source.clone()),
            ability_id: Some("next-bg-jc039-hide".into()),
            target_id: Some(teammate),
            ..Action::new("activate")
        },
    );
    activate(
        &mut r,
        &mut steps,
        0,
        &source,
        "next-bg-jc039-hide",
        Some(&own),
    );
    attachment_pass_top(&mut r, &mut steps);
    assert!(board(&r, &own).is_none());
    let c = r.regions[1]
        .cards
        .iter()
        .find(|c| c.controller == 0)
        .unwrap();
    assert!(c.face_down);
    assert_eq!(c.permanent_combat_markers, 0);
    result(
        "jc039-own-blue-hide-fresh-instance-clears-marker",
        steps,
        &["JC039"],
    )
}

fn named_attachment(embrace: bool) -> Value {
    let mut g = layout("named-attachment");
    fund(&mut g, 0, "JC029", 7);
    let source = field(&mut g, "XG10", 0, 0);
    let host = field(&mut g, if embrace { "JC125" } else { "JC029" }, 1, 0);
    let ids = deck(&mut g, 0, &["XQ14", "BQ040", "JC036", "JC029"]);
    let (mut r, mut steps) = initial(g);
    let before = ready(&r, 0);
    activate(
        &mut r,
        &mut steps,
        0,
        &source,
        if embrace {
            "next-bg-xg10-embrace"
        } else {
            "next-bg-xg10-blood"
        },
        Some(&host),
    );
    assert_eq!(ready(&r, 0), before - if embrace { 5 } else { 1 });
    until(&mut r, &mut steps, "next-blue-gray-search");
    private(&r, 0, &ids);
    assert_eq!(
        r.pending.as_ref().unwrap().choice.options.len(),
        if embrace { 1 } else { 2 }
    );
    let p = r.pending.clone().unwrap();
    let selected = p.choice.options[0].id.clone();
    reject(
        &mut r,
        &mut steps,
        1,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![selected.clone()]),
            ..Action::new("choose")
        },
    );
    choose(&mut r, &mut steps, vec![selected]);
    repeat_last(&mut r, &mut steps);
    settle(&mut r, &mut steps);
    assert!(r
        .attachments
        .iter()
        .any(|a| a.host_id == host && a.card.definition == if embrace { "JC036" } else { "XQ14" }));
    if embrace {
        assert_eq!(board(&r, &host).unwrap().1.controller, 0);
    }
    result(
        if embrace {
            "xg10-five-cost-embrace-control"
        } else {
            "xg10-one-cost-blood-search"
        },
        steps,
        &["XG10"],
    )
}

fn military_search(battlegroup: bool, replaced: bool) -> Value {
    let mut g = layout("military-search");
    fund(&mut g, 0, "CQ17", 16);
    let definition = if battlegroup { "BQ059" } else { "CQ17" };
    let source = if battlegroup {
        held(&mut g, definition, 0)
    } else {
        field(&mut g, definition, 0, 0)
    };
    let ally = field(&mut g, "BQ054", 0, 0);
    let teammate = field(&mut g, "BQ054", 1, 0);
    let ids = deck(&mut g, 0, &["BQ054", "CQ16", "CQ17", "LC22", "BQ059"]);
    let (mut r, mut steps) = initial(g);
    let source = if battlegroup {
        apply_game(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(source),
                region: Some(0),
                ..Action::new("deploy")
            },
        );
        settle(&mut r, &mut steps);
        let c = r.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == definition)
            .unwrap();
        assert_eq!(c.shield, 2);
        c.id.clone()
    } else {
        assert_eq!(
            r.game.current_icons(board(&r, &ally).unwrap().1, 0).combat,
            r.game
                .current_icons(board(&r, &teammate).unwrap().1, 0)
                .combat
                + 1
        );
        source
    };
    let before = ready(&r, 0);
    if battlegroup {
        reject(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(source.clone()),
                ability_id: Some("next-bg-bq059-search-6".into()),
                ..Action::new("activate")
            },
        );
    }
    activate(
        &mut r,
        &mut steps,
        0,
        &source,
        if battlegroup {
            "next-bg-bq059-search-3"
        } else {
            "next-bg-cq17-search"
        },
        None,
    );
    assert_eq!(ready(&r, 0), before - if battlegroup { 3 } else { 2 });
    if replaced {
        // Boundary case starts at a real paid command's saved frame; only its
        // initial layout substitutes the region. No rule program is constructed.
        r.game.regions[0].card = r.game.make_card("DQJC115", 0);
        let raw = serde_json::to_string(&r).unwrap();
        r = RoomEnvelope::from_persisted(&raw).unwrap();
        steps = vec![step(&r, "initialFixture", json!([raw]), 0)];
    }
    until(&mut r, &mut steps, "next-blue-gray-search");
    private(&r, 0, &ids);
    let selected = if battlegroup {
        ids[3].clone()
    } else {
        ids[1].clone()
    };
    choose(&mut r, &mut steps, vec![selected]);
    settle(&mut r, &mut steps);
    let wanted = if battlegroup { "LC22" } else { "CQ16" };
    assert_eq!(
        r.regions[0].cards.iter().any(|c| c.definition == wanted),
        !replaced
    );
    let mut out = result(
        &format!(
            "{definition}-exact-search-{}",
            if replaced {
                "replacement-region-fizzles"
            } else {
                "original-region"
            }
        ),
        steps,
        &[definition],
    );
    out["syntheticPaidFrameCheckpoint"] = json!(replaced);
    out
}

fn cq08_reveal() -> Value {
    let mut g = layout("cq08-reveal");
    fund(&mut g, 0, "JC029", 4);
    let source = field(&mut g, "CQ08", 0, 0);
    g.regions[0].cards.last_mut().unwrap().face_down = true;
    let ids = deck(&mut g, 0, &["CQ08", "JC029"]);
    let (mut r, mut steps) = initial(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    );
    until(&mut r, &mut steps, "trigger");
    choose(&mut r, &mut steps, vec!["accept".into()]);
    until(&mut r, &mut steps, "next-blue-gray-search");
    private(&r, 0, &ids);
    choose(&mut r, &mut steps, vec![ids[0].clone()]);
    settle(&mut r, &mut steps);
    assert_eq!(
        r.regions[0]
            .cards
            .iter()
            .filter(|c| c.definition == "CQ08" && c.face_down)
            .count(),
        1
    );
    result(
        "cq08-reveal-private-same-name-hidden-entry",
        steps,
        &["CQ08"],
    )
}

fn red_badge(definition: &str) -> Value {
    let mut g = layout("red-badge");
    fund(&mut g, 0, "JC047", 6);
    let source = field(&mut g, "JC051", 0, 0);
    g.regions[0].cards.last_mut().unwrap().face_down = true;
    let ids = deck(&mut g, 0, &[definition, "JC125", "JC125"]);
    let (mut r, mut steps) = initial(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    );
    until(&mut r, &mut steps, "trigger");
    choose(&mut r, &mut steps, vec!["accept".into()]);
    until(&mut r, &mut steps, "search");
    private(&r, 0, &ids);
    choose(&mut r, &mut steps, vec![ids[0].clone()]);
    if r.pending.is_some() {
        assert_eq!(r.pending.as_ref().unwrap().choice.kind, "target");
        choose(&mut r, &mut steps, vec!["society:p0".into()]);
    }
    settle(&mut r, &mut steps);
    assert!(r.players[0].society_zone.card.is_none());
    assert!(r.attachments.iter().any(|a| a.card.definition == definition
        && a.host_id == "society:p0"
        && a.card.id != ids[0]));
    result(
        &format!("jc051-private-badge-{definition}-empty-zone"),
        steps,
        &["JC051"],
    )
}

fn red_hourglass(replaced: bool) -> Value {
    let mut g = layout("red-hourglass");
    fund(&mut g, 0, "JC047", 1);
    let source = field(&mut g, "JZ32", 0, 0);
    let hourglass = grave(&mut g, "JC048", 0);
    let (mut r, mut steps) = initial(g);
    activate(
        &mut r,
        &mut steps,
        0,
        &source,
        "next20-red-jz32-hourglass",
        None,
    );
    assert_eq!(ready(&r, 0), 0);
    assert!(board(&r, &source).is_none());
    if replaced {
        r.game.regions[0].card = r.game.make_card("DQJC115", 0);
        let raw = serde_json::to_string(&r).unwrap();
        r = RoomEnvelope::from_persisted(&raw).unwrap();
        steps = vec![step(&r, "initialFixture", json!([raw]), 0)];
        attachment_pass_top(&mut r, &mut steps);
        assert!(r.attachments.is_empty());
        assert!(r.players[0].graveyard.iter().any(|c| c.id == hourglass));
    } else {
        until(&mut r, &mut steps, "graveyard");
        choose(&mut r, &mut steps, vec![hourglass.clone()]);
        repeat_last(&mut r, &mut steps);
        settle(&mut r, &mut steps);
        assert!(r.attachments.iter().any(|a| a.card.definition == "JC048"
            && a.card.id != hourglass
            && a.host_id == r.regions[0].card.id));
    }
    let mut out = result(
        if replaced {
            "jz32-paid-replacement-region-fizzle"
        } else {
            "jz32-real-sacrifice-hourglass-return"
        },
        steps,
        &["JZ32"],
    );
    out["syntheticPaidFrameCheckpoint"] = json!(replaced);
    out
}

fn red_destroy() -> Value {
    let mut g = layout("red-destroy");
    fund(&mut g, 0, "JC047", 6);
    let own = field(&mut g, "JC125", 0, 0);
    let team = field(&mut g, "JC125", 1, 0);
    let enemy = field(&mut g, "JC125", 2, 0);
    let source = held(&mut g, "JZ34", 0);
    let (mut r, mut steps) = initial(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(source),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    until(&mut r, &mut steps, "trigger");
    choose(&mut r, &mut steps, vec!["accept".into()]);
    until(&mut r, &mut steps, "sacrifice");
    assert_eq!(
        r.pending
            .as_ref()
            .unwrap()
            .choice
            .options
            .iter()
            .map(|o| o.id.clone())
            .collect::<Vec<_>>(),
        vec![own.clone()]
    );
    choose(&mut r, &mut steps, vec![own.clone()]);
    assert!(r.regions[0].destroyed);
    assert!(board(&r, &own).is_none());
    assert!(board(&r, &team).is_some() && board(&r, &enemy).is_some());
    assert_eq!(trigger_source(&r), "JZ34");
    choose(&mut r, &mut steps, vec!["region:0".into()]);
    assert_eq!(
        r.stack.last().unwrap().frame.as_ref().unwrap().targets[0].region_instance,
        Some(r.regions[0].card.id.clone())
    );
    attachment_pass_top(&mut r, &mut steps);
    assert_eq!(r.regions[0].influence, [1, 0]);
    result(
        "jz34-entry-personal-sacrifice-destruction-death-influence",
        steps,
        &["JZ34"],
    )
}

fn ring_and_raid() -> Value {
    let mut g = layout("ring-raid");
    fund(&mut g, 0, "JC047", 1);
    fund(&mut g, 0, "JC029", 6);
    let host = field(&mut g, "BQ059", 2, 0);
    let raider = field(&mut g, "JZ28", 0, 0);
    g.regions[0].cards.last_mut().unwrap().face_down = true;
    let ring = held(&mut g, "JZ41", 0);
    let ids = deck(&mut g, 0, &["JC125", "JC029", "JC125"]);
    let (mut r, mut steps) = initial(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(ring),
            target_id: Some(host.clone()),
            ..Action::new("play")
        },
    );
    until(&mut r, &mut steps, "trigger");
    choose(&mut r, &mut steps, vec!["accept".into()]);
    until(&mut r, &mut steps, "investigation");
    private(&r, 0, &ids);
    let p = r.pending.clone().unwrap();
    assert!(matches!(
        p.resolution,
        ChoiceResolution::Frame {
            choice: FrameChoice::Forecast { .. },
            ..
        }
    ));
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            choice_id: Some(p.choice.id),
            top: Some(vec![ids[1].clone()]),
            bottom: Some(vec![ids[0].clone()]),
            ..Action::new("choose")
        },
    );
    assert_eq!(r.players[0].deck.first().unwrap().id, ids[1]);
    assert_eq!(r.players[0].deck.last().unwrap().id, ids[0]);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(raider),
            ..Action::new("reveal")
        },
    );
    until(&mut r, &mut steps, "trigger");
    assert_eq!(trigger_source(&r), "JZ28");
    choose(&mut r, &mut steps, vec![host.clone()]);
    until(&mut r, &mut steps, "trigger");
    assert_eq!(trigger_source(&r), "JZ41");
    choose(&mut r, &mut steps, vec!["accept".into()]);
    attachment_pass_top(&mut r, &mut steps);
    assert_eq!(board(&r, &host).unwrap().1.damage, 2);
    assert_eq!(board(&r, &host).unwrap().1.wounds, 1);
    result(
        "jz41-private-forecast-jz28-raid-adds-real-wound",
        steps,
        &["JZ41", "JZ28"],
    )
}

fn analysis_and_news() -> Value {
    let mut g = layout("analysis-news");
    fund(&mut g, 0, "JC059", 3);
    let analyst = field(&mut g, "WM039", 0, 2);
    g.players[0].society_zone.exposure = 1;
    g.players[1].society_zone.secrecy = 1;
    let expose = held(&mut g, "WM042", 0);
    let cover = held(&mut g, "WM042", 0);
    let (mut r, mut steps) = initial(g);
    activate(&mut r, &mut steps, 0, &analyst, "wm039-analysis", None);
    attachment_pass_top(&mut r, &mut steps);
    assert_eq!(r.players[0].society_zone.exposure, 0);
    assert_eq!(
        r.game
            .current_icons(board(&r, &analyst).unwrap().1, 2)
            .investigation,
        1
    );
    reject(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(analyst),
            ability_id: Some("wm039-analysis".into()),
            ..Action::new("activate")
        },
    );
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(expose),
            option: Some("expose".into()),
            ..Action::new("play")
        },
    );
    attachment_pass_top(&mut r, &mut steps);
    assert_eq!(r.players[1].society_zone.secrecy, 0);
    assert_eq!(r.players[1].society_zone.exposure, 0);
    assert_eq!(r.players[0].society_zone.exposure, 1);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(cover),
            option: Some("cover".into()),
            ..Action::new("play")
        },
    );
    attachment_pass_top(&mut r, &mut steps);
    assert!(r.players.iter().all(|p| p.society_zone.exposure == 0));
    assert_eq!(ready(&r, 0), 0);
    result(
        "wm039-once-analysis-wm042-both-modes-cancellation",
        steps,
        &["WM039", "WM042"],
    )
}

fn commissioner() -> Value {
    let mut g = layout("commissioner");
    fund(&mut g, 0, "JC059", 7);
    g.players[0].society_zone.exposure = 2;
    g.players[1].society_zone.exposure = 1;
    g.players[3].society_zone.secrecy = 2;
    g.regions[0].influence[1] = 2;
    g.regions[2].influence[1] = 2;
    let own = field(&mut g, "JC059", 0, 2);
    g.regions[2].cards.last_mut().unwrap().wounds = 1;
    let team = field(&mut g, "JC059", 1, 2);
    g.regions[2].cards.last_mut().unwrap().wounds = 1;
    let away = field(&mut g, "JC059", 0, 1);
    g.regions[1].cards.last_mut().unwrap().wounds = 1;
    let source = held(&mut g, "WM040", 0);
    let (mut r, mut steps) = initial(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(source),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    until(&mut r, &mut steps, "trigger");
    choose(&mut r, &mut steps, vec!["p2".into()]);
    until(&mut r, &mut steps, "influence");
    assert_eq!(r.pending.as_ref().unwrap().choice.min, Some(3));
    assert!(r.players.iter().all(|p| p.society_zone.exposure == 0));
    let options = r
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .take(3)
        .map(|o| o.id.clone())
        .collect();
    choose(&mut r, &mut steps, options);
    assert_eq!(r.regions.iter().map(|r| r.influence[1]).sum::<u32>(), 1);
    let id = r.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "WM040")
        .unwrap()
        .id
        .clone();
    activate(&mut r, &mut steps, 0, &id, "wm040-heal", None);
    attachment_pass_top(&mut r, &mut steps);
    assert_eq!(board(&r, &own).unwrap().1.wounds, 0);
    assert_eq!(board(&r, &team).unwrap().1.wounds, 1);
    assert_eq!(board(&r, &away).unwrap().1.wounds, 1);
    assert_eq!(r.players[0].society_zone.exposure, 1);
    assert!(board(&r, &id).unwrap().1.exhausted);
    result(
        "wm040-real-exposure-removal-repress-and-local-heal",
        steps,
        &["WM040"],
    )
}

fn conditional_mobility() -> Value {
    let mut g = layout("conditional-mobility");
    let source = field(&mut g, "WM040", 0, 2);
    g.players[0].society_zone.exposure = 1;
    let mut covers = vec![];
    for seat in 0..4 {
        fund(&mut g, seat, "JC059", 1);
        covers.push(held(&mut g, "WM042", seat));
    }
    g.window = Some(Window::Action(1));
    g.active_team = 1;
    g.priority_team = 1;
    let (mut r, mut steps) = initial(g);
    until(&mut r, &mut steps, "trigger");
    let ChoiceResolution::Declare { declaration, .. } = &r.pending.as_ref().unwrap().resolution
    else {
        unreachable!()
    };
    assert_eq!(declaration.ability.key, "mobility");
    assert_eq!(declaration.source.observed_exposure, Some(1));
    choose(&mut r, &mut steps, vec!["region:1".into()]);
    let actor = (0..4).find(|s| r.team(*s) == r.priority_team).unwrap();
    apply_game(
        &mut r,
        &mut steps,
        actor,
        Action {
            card_id: Some(covers[actor].clone()),
            option: Some("cover".into()),
            ..Action::new("play")
        },
    );
    attachment_pass_top(&mut r, &mut steps);
    assert_eq!(r.players[0].society_zone.exposure, 0);
    assert_ne!(
        r.view(0, r.pacing.last_server_now_ms).game.regions[2]
            .characters
            .iter()
            .find(|v| v.instance_id == source)
            .unwrap()
            .current_mobility,
        Some(true)
    );
    attachment_pass_top(&mut r, &mut steps);
    assert_eq!(board(&r, &source).unwrap().0, 1);
    result(
        "wm040-frozen-conditional-mobility-survives-cover-response",
        steps,
        &["WM040", "WM042"],
    )
}

fn granted_reward(definition: &str, replaced: bool) -> Value {
    let mut g = layout("granted-reward");
    g.regions[2].card = g.make_card("DQJC115", 0);
    g.regions[2].influence = [0, 0];
    let source = field(&mut g, definition, 0, 2);
    fund(&mut g, 0, "JC075", 2);
    let grant = held(&mut g, "JC074", 0);
    g.window = Some(Window::Before(2, 2));
    let original = g.regions[2].card.id.clone();
    let (mut r, mut steps) = initial(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(grant),
            target_id: Some(source.clone()),
            ..Action::new("play")
        },
    );
    attachment_pass_top(&mut r, &mut steps);
    until(&mut r, &mut steps, "trigger");
    let ChoiceResolution::Declare { declaration, .. } = &r.pending.as_ref().unwrap().resolution
    else {
        unreachable!()
    };
    assert_eq!(declaration.ability.key, "renown");
    assert_eq!(declaration.source.card.id, source);
    assert_eq!(
        declaration.source.source_region_instance.as_deref(),
        Some(original.as_str())
    );
    choose(&mut r, &mut steps, vec!["accept".into()]);
    if replaced {
        // Save a synthetic paid checkpoint after actual granting and reward
        // commands. Replacing this initial region must not redirect that frame.
        r.game.regions[2].card = r.game.make_card("DQJC115", 0);
        let raw = serde_json::to_string(&r).unwrap();
        r = RoomEnvelope::from_persisted(&raw).unwrap();
        steps = vec![step(&r, "initialFixture", json!([raw]), 0)];
    }
    let before = r.regions[2].influence[0];
    attachment_pass_top(&mut r, &mut steps);
    assert_eq!(r.regions[2].influence[0], before + u32::from(!replaced));
    let mut out = result(
        &format!(
            "{definition}-granted-renown-{}",
            if replaced {
                "replacement-region-fizzle"
            } else {
                "original-region"
            }
        ),
        steps,
        &[definition],
    );
    out["syntheticPaidFrameCheckpoint"] = json!(replaced);
    out
}

pub fn cases() -> impl Iterator<Item = Value> {
    let factories: Vec<Box<dyn Fn() -> Value>> = vec![
        Box::new(mixed_preparation),
        Box::new(|| reveal_rumor(false)),
        Box::new(|| reveal_rumor(true)),
        Box::new(mill_rumor_discard),
        Box::new(fish_mill),
        Box::new(sacrifice_death),
        Box::new(|| badge_discount("JC039", false)),
        Box::new(|| badge_discount("JC039", true)),
        Box::new(|| badge_discount("XQ39", false)),
        Box::new(|| badge_discount("XQ39", true)),
        Box::new(badge_recover),
        Box::new(blue_hide),
        Box::new(|| named_attachment(false)),
        Box::new(|| named_attachment(true)),
        Box::new(|| military_search(true, false)),
        Box::new(|| military_search(false, false)),
        Box::new(|| military_search(false, true)),
        Box::new(cq08_reveal),
        Box::new(|| red_badge("JC039")),
        Box::new(|| red_badge("XQ39")),
        Box::new(|| red_hourglass(false)),
        Box::new(|| red_hourglass(true)),
        Box::new(red_destroy),
        Box::new(ring_and_raid),
        Box::new(analysis_and_news),
        Box::new(commissioner),
        Box::new(conditional_mobility),
    ];
    factories.into_iter().map(|f| f()).chain(
        ["JC051", "CQ26", "JZ23", "WM040"]
            .into_iter()
            .flat_map(|id| {
                [false, true]
                    .into_iter()
                    .map(move |replaced| granted_reward(id, replaced))
            }),
    )
}
