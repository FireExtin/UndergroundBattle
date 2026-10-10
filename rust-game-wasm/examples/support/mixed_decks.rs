//! Real four-seat red/black versus red/blue decks. No fixture layout or state edits.
use super::*;

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

fn draft(seat: usize) -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    let other = if seat < 2 { "黑" } else { "蓝" };
    d.id = format!("mixed-red-{other}");
    d.name = format!("红{other}混搭（合法中立补足）");
    let black = ["JC086", "JC092", "JC091", "JC084"];
    // Keep the original regression decks finite as the public card pool grows.
    let red = ["JC042", "JC049", "JC047", "XQ17"];
    let blue = ["XQ12", "XQ16", "JC036", "JZ27", "XQ14"];
    d.cards = catalog::catalog()
        .cards
        .iter()
        .filter(|c| {
            c.color == "红" && red.contains(&c.id.as_str())
                || c.color == other
                    && (if other == "蓝" { &blue[..] } else { &black[..] })
                        .contains(&c.id.as_str())
        })
        .map(|c| catalog::DeckEntry {
            card_id: c.id.clone(),
            count: 3,
        })
        .collect();
    let n: usize = d.cards.iter().map(|c| c.count).sum();
    d.cards.push(catalog::DeckEntry {
        card_id: "JC125".into(),
        count: 50 - n,
    });
    deck::validate(d).unwrap()
}
fn plan(seed: u64) -> Option<Vec<(usize, Action)>> {
    let mut g = Game::new_with_deck(
        "natural-mixed-red".into(),
        "QA".into(),
        "teams".into(),
        "P0".into(),
        draft(0),
        seed,
    )
    .unwrap();
    for s in 1..4 {
        g.join_with_deck(format!("P{s}"), draft(s)).unwrap();
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
    while g.pending.is_some() {
        let (s, a) = pick_choice(&g);
        g.apply(s, a.clone()).unwrap();
        actions.push((s, a));
    }
    let mut sources = vec![];
    let mut red = vec![];
    let mut other = vec![];
    for s in 0..4 {
        let c = g.players[s].hand.iter().find(|c| c.definition == "XQ17")?;
        sources.push(c.id.clone());
        red.push(
            g.players[s]
                .hand
                .iter()
                .find(|a| a.id != c.id && catalog::card(&a.definition).color == "红")?
                .id
                .clone(),
        );
        other.push(
            g.players[s]
                .hand
                .iter()
                .find(|a| catalog::card(&a.definition).color == if s < 2 { "黑" } else { "蓝" })?
                .id
                .clone(),
        );
    }
    let (mut built_other, mut built_red, mut deployed) = ([false; 4], [false; 4], [false; 4]);
    for _ in 0..700 {
        if deployed.iter().all(|x| *x) && g.stack.is_empty() && g.pending.is_none() {
            return Some(actions);
        }
        let next = if g.pending.is_some() {
            Some(pick_choice(&g))
        } else {
            (0..4).find_map(|s| {
                let (kind, id) = if !built_other[s] {
                    ("asset", &other[s])
                } else if !built_red[s] && built_other.iter().all(|x| *x) {
                    ("asset", &red[s])
                } else if !deployed[s] && built_red[s] {
                    ("deploy", &sources[s])
                } else {
                    return None;
                };
                g.legal_actions(s)
                    .into_iter()
                    .find(|a| {
                        a.action.kind == kind
                            && a.action.card_id.as_ref() == Some(id)
                            && (kind != "deploy" || a.action.region == Some(2))
                    })
                    .map(|a| {
                        if kind == "deploy" {
                            deployed[s] = true;
                        } else if !built_other[s] {
                            built_other[s] = true;
                        } else {
                            built_red[s] = true;
                        }
                        (s, a.action)
                    })
            })
        };
        let (s, a) = next.unwrap_or_else(|| pass_action(&g));
        g.apply(s, a.clone()).unwrap();
        actions.push((s, a));
    }
    None
}
fn reject(r: &mut RoomEnvelope, steps: &mut Vec<Value>, seat: usize, action: Action) {
    let c = RoomCommand {
        command_id: format!("mixed-reject:{}", steps.len()),
        expected_version: r.revision,
        action: action.into(),
    };
    let now = r.pacing.last_server_now_ms;
    let t = r.transition(seat, Some(c.clone()), now).unwrap();
    assert_eq!(t.outcome, "rejected");
    assert!(!t.changed);
    assert!(t
        .error_message
        .as_deref()
        .unwrap_or("")
        .contains("忠诚不足"));
    record_transition(r, steps, "applyRoom", json!([seat, c, now.to_string()]), t);
}
fn natural() -> Value {
    let (seed, actions) = (1..10000)
        .find_map(|s| plan(s).map(|a| (s, a)))
        .expect("naturally dealt four shared red characters plus own second-color and red assets");
    let d = draft(0);
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "natural-mixed-red".into(),
            "QA".into(),
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
        let d = draft(s);
        r.game.join_with_deck(format!("P{s}"), d.clone()).unwrap();
        r.revision = r.game.version;
        steps.push(step(
            &r,
            "joinGameWithDeck",
            json!([format!("P{s}"), serde_json::to_string(&d).unwrap()]),
            s,
        ));
    }
    let mut checked = [false; 2];
    let mut checked_other = false;
    for (s, a) in actions {
        apply_game(&mut r, &mut steps, s, a.clone());
        if !checked_other && r.players.iter().all(|p| p.assets.len() == 1) {
            checked_other = true;
            for p in &r.players {
                assert_ne!(catalog::card(&p.assets[0].definition).color, "红");
                assert!(r
                    .game
                    .legal_actions(p.seat)
                    .iter()
                    .all(|l| l.action.kind != "deploy"
                        || !p
                            .hand
                            .iter()
                            .any(|c| c.definition == "XQ17"
                                && l.action.card_id.as_ref() == Some(&c.id))));
            }
        }
        if a.kind == "asset" && r.players[s].assets.len() == 2 && !checked[s / 2] {
            let mate = s ^ 1;
            if r.players[mate].assets.len() == 1 {
                checked[s / 2] = true;
                let source = r.players[mate]
                    .hand
                    .iter()
                    .find(|c| c.definition == "XQ17")
                    .unwrap()
                    .id
                    .clone();
                reject(
                    &mut r,
                    &mut steps,
                    mate,
                    Action {
                        card_id: Some(source),
                        region: Some(2),
                        ..Action::new("deploy")
                    },
                );
            }
        }
    }
    assert!(checked_other && checked == [true, true]);
    let cards: Vec<_> = r.regions[2]
        .cards
        .iter()
        .filter(|c| c.definition == "XQ17")
        .collect();
    assert_eq!(cards.len(), 4);
    assert_eq!(
        cards
            .iter()
            .map(|c| &c.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    for s in 0..4 {
        let c = cards.iter().find(|c| c.owner == s).unwrap();
        assert_eq!(c.controller, s);
        assert_eq!(r.players[s].assets.len(), 2);
        assert_eq!(
            catalog::card(&r.players[s].assets[0].definition).color,
            if s < 2 { "黑" } else { "蓝" }
        );
        assert!(r.players[s].assets[0].exhausted);
        assert!(!r.players[s].assets[1].exhausted);
        assert_eq!(
            catalog::card(&r.players[s].assets[1].definition).color,
            "红"
        );
        let view = r.view(s, r.pacing.last_server_now_ms);
        let projected: Vec<_> = view.regions[2]
            .characters
            .iter()
            .filter(|v| v.card_id.as_deref() == Some("XQ17"))
            .collect();
        assert_eq!(projected.len(), 4);
        for source in &cards {
            let v = projected
                .iter()
                .find(|v| v.instance_id == source.id)
                .unwrap();
            assert_eq!(v.owner, format!("p{}", source.owner));
            assert_eq!(v.controller, v.owner);
        }
    }
    json!({"name":"mixed-decks-natural-red-black-versus-red-blue-four-shared-red-personal-loyalty","seed":seed.to_string(),"syntheticInitialLayout":false,"newNaturalUiCoverage":false,"actualJoinedDealtGameNoStateInjection":true,"steps":steps})
}
pub fn cases() -> impl Iterator<Item = Value> {
    std::iter::once(natural())
}
