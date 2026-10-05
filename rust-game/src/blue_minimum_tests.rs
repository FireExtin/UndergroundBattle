//! Explicit offline layouts. Commands and restored four-seat views use the
//! production RoomEnvelope protocol; these are not claims of natural public play.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, model::*, rules::*};

fn activate(id: &str, key: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(key.into()),
        ..Action::new("activate")
    }
}
fn selection(g: &Game, ids: Vec<String>) -> Action {
    Action {
        choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
        selected: Some(ids),
        ..Action::new("choose")
    }
}
fn deck(g: &mut Game, actor: usize, defs: &[&str]) -> Vec<String> {
    g.players[actor].deck.clear();
    let mut ids = vec![];
    for def in defs {
        let c = g.make_card(def, actor);
        ids.push(c.id.clone());
        g.players[actor].deck.push(c);
    }
    ids
}
fn fixture(kind: &str, g: &Game) {
    if let Ok(dir) = std::env::var("BLUE_FRONTEND_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        let row = serde_json::json!({"kind":kind,"state":serde_json::to_string(&r).unwrap(),"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()});
        std::fs::write(
            format!("{dir}/{kind}.json"),
            serde_json::to_vec(&row).unwrap(),
        )
        .unwrap();
    }
}
fn elder(g: &mut Game, actor: usize) -> String {
    elder_owned(g, actor, actor)
}
fn elder_owned(g: &mut Game, actor: usize, owner: usize) -> String {
    let source = board(g, "JC032", owner, 2);
    g.board_mut(&source).unwrap().controller = actor;
    fund(g, actor, "JC125", 2);
    apply(g, actor, activate(&source, "top-six-vampire-hidden"));
    source
}

#[test]
fn blue_whole_original_fields_and_closed_source_marker() {
    let elder = catalog::card("JC032");
    let tracker = catalog::card("JZ24");
    assert_eq!(
        (&*elder.name, elder.cost, &*elder.magic, elder.defense),
        ("血族长老", 5, "死亡", Some(3))
    );
    assert_eq!(elder.subtypes, ["吸血鬼"]);
    assert_eq!(elder.loyalty, ["蓝色", "蓝色"]);
    assert!(!elder.unique);
    assert_eq!(
        elder.permanent_icons,
        Icons {
            investigation: 1,
            combat: 1,
            influence: 2
        }
    );
    assert_eq!(elder.temporary_icons, Icons::default());
    assert_eq!(
        (
            &*tracker.name,
            tracker.cost,
            &*tracker.magic,
            tracker.defense
        ),
        ("卡迪纳追迹人", 4, "死亡", Some(1))
    );
    assert_eq!(tracker.subtypes, ["吸血鬼", "罪犯"]);
    assert_eq!(tracker.loyalty, ["蓝色"]);
    assert!(!tracker.unique);
    assert_eq!(
        tracker.permanent_icons,
        Icons {
            investigation: 1,
            combat: 2,
            influence: 0
        }
    );
    assert_eq!(tracker.temporary_icons, Icons::default());
    let a = &definition("JC032").abilities[0];
    assert_eq!(a.per_turn_limit, Some(1));
    assert!(matches!(a.costs.as_slice(), [Cost::Assets(2)]));
    assert!(a.targets.is_empty() && a.event.is_none() && !a.requires_ready_source);
    assert!(matches!(a.ops.as_slice(), [Op::JC032TopSixVampireHidden]));
    let a = &definition("JZ24").abilities[0];
    assert_eq!(a.event, Some(Event::Reveal));
    assert!(a.costs.is_empty() && a.targets.is_empty() && a.per_turn_limit.is_none());
    let mut g = game(0);
    for def in ["JC029", "JC030", "JC032", "JZ24"] {
        let c = g.make_card(def, 0);
        let snapshot = g.source_snapshot(&c, Some(2));
        assert_eq!(
            snapshot.source_region_instance.is_some(),
            matches!(def, "JC032" | "JZ24")
        );
        let value = serde_json::to_value(snapshot).unwrap();
        assert_eq!(
            value.get("source_region_instance").is_some(),
            matches!(def, "JC032" | "JZ24")
        );
    }
}

#[test]
fn blue_jc032_positive_lengths_and_stale_choice_are_atomic() {
    for len in [1, 5, 6, 7, 8] {
        let mut g = game(0);
        let mut defs = vec!["JC030"; len];
        defs[0] = "JC029";
        let ids = deck(&mut g, 0, &defs);
        elder(&mut g, 0);
        let mut paid = envelope(&g).game;
        pass_top(&mut g);
        pass_top(&mut paid);
        assert_eq!(
            serde_json::to_value(&g).unwrap(),
            serde_json::to_value(&paid).unwrap()
        );
        assert_eq!(
            g.view(0).pending_choice.unwrap().preview_cards.len(),
            len.min(6)
        );
        let selected = selection(&g, vec![ids[0].clone()]);
        let mut old = selected.clone();
        old.choice_id = Some("stale-choice".into());
        reject(&mut g, 0, old);
        reject(&mut g, 1, selected.clone());
        choose(&mut g, vec![ids[0].clone()]);
        reject(&mut g, 0, selected);
        assert_eq!(
            g.regions[2]
                .cards
                .iter()
                .filter(|c| c.definition == "JC029")
                .count(),
            1
        );
    }
}

#[test]
fn blue_jc032_borrowed_source_and_selection_keep_owner_actor_and_original_region() {
    for mutation in 0..3 {
        let mut g = game(0);
        let ids = deck(
            &mut g,
            0,
            &["JC029", "JC030", "JC125", "JZ24", "JC003", "JC059", "JC029"],
        );
        g.players[0].deck[0].owner = 2;
        g.players[0].deck[0].controller = 2;
        let source = elder_owned(&mut g, 0, 2);
        match mutation {
            0 => {
                let c = g.regions[2].cards.remove(0);
                g.regions[1].cards.push(c);
            }
            1 => g.return_hand(&source),
            _ => g.board_mut(&source).unwrap().controller = 3,
        }
        pass_top(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().seat, 0);
        fixture(&format!("jc032-source-change-{mutation}"), &g);
        choose(&mut g, vec![ids[0].clone()]);
        let result = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "JC029")
            .unwrap();
        assert_eq!((result.owner, result.controller), (2, 0));
        assert!(result.face_down);
        assert_ne!(result.id, ids[0]);
        for viewer in 0..4 {
            let view = g.view(viewer);
            let c = view.regions[2]
                .characters
                .iter()
                .find(|c| c.instance_id == result.id)
                .unwrap();
            assert_eq!(c.card_id.is_some(), viewer == 0);
        }
        if mutation == 2 {
            fixture("jc032-borrowed-hidden-result", &g);
        }
    }
}

#[test]
fn blue_jc032_region_incarnation_and_prefix_guard_preserve_paid_once_rng_and_ids() {
    for pending in [false, true] {
        let mut g = game(0);
        deck(
            &mut g,
            0,
            &["JC029", "JC030", "JC125", "JC003", "JC059", "JZ24", "JC029"],
        );
        let source = elder(&mut g, 0);
        let resources = g.resources(0);
        let usage = serde_json::to_value(&g.turn_ability_usage).unwrap();
        if pending {
            pass_top(&mut g);
        }
        let before_deck = serde_json::to_value(&g.players[0].deck).unwrap();
        let rng = g.random;
        let old = g.regions[2].card.clone();
        g.regions[2].card = g.fresh(old);
        if pending {
            let a = selection(&g, vec![g.players[0].deck[0].id.clone()]);
            reject(&mut g, 0, a);
        } else {
            pass_top(&mut g);
            assert!(g.pending.is_none());
            reject(&mut g, 0, activate(&source, "top-six-vampire-hidden"));
        }
        assert_eq!(g.resources(0), resources);
        assert_eq!(serde_json::to_value(&g.turn_ability_usage).unwrap(), usage);
        assert_eq!(g.random, rng);
        assert_eq!(
            serde_json::to_value(&g.players[0].deck).unwrap(),
            before_deck
        );
        assert_eq!(g.regions[2].cards.len(), 1);
    }
    let mut g = game(0);
    deck(&mut g, 0, &["JC029", "JZ24"]);
    elder(&mut g, 0);
    pass_top(&mut g);
    let a = selection(&g, vec![g.players[0].deck[0].id.clone()]);
    g.players[0].deck.swap(0, 1);
    reject(&mut g, 0, a);
}

#[test]
fn blue_jc032_next_turn_refreshes_limit_without_repaying_saved_action() {
    let mut g = game(0);
    deck(
        &mut g,
        0,
        &[
            "JC029", "JZ24", "JC030", "JC125", "JC125", "JC125", "JC125", "JC125", "JC125",
        ],
    );
    let source = elder(&mut g, 0);
    pass_top(&mut g);
    let selected = g.players[0].deck[0].id.clone();
    choose(&mut g, vec![selected]);
    fund(&mut g, 0, "JC125", 2);
    reject(&mut g, 0, activate(&source, "top-six-vampire-hidden"));
    let turn = g.turn;
    for _ in 0..250 {
        if g.turn > turn {
            break;
        }
        let seat = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .expect("bounded next-turn passes");
        apply(&mut g, seat, Action::new("pass"));
        while let Some(p) = g.pending.clone() {
            if matches!(p.choice.kind.as_str(), "investigation" | "order") {
                apply(
                    &mut g,
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id),
                        top: Some(p.choice.options.iter().map(|o| o.id.clone()).collect()),
                        bottom: Some(vec![]),
                        ..Action::new("choose")
                    },
                );
            } else {
                choose(
                    &mut g,
                    p.choice
                        .options
                        .iter()
                        .take(p.choice.min.unwrap_or(0))
                        .map(|o| o.id.clone())
                        .collect(),
                );
            }
        }
    }
    assert!(g.turn > turn);
    g.begin_window(Window::Action(0));
    fund(&mut g, 0, "JC125", 2);
    apply(&mut g, 0, activate(&source, "top-six-vampire-hidden"));
    assert!(!g.board(&source).unwrap().1.exhausted);
}

#[test]
fn blue_jz24_snapshot_after_responses_never_adds_newly_eligible_enemy() {
    let mut g = game(2);
    let a = board(&mut g, "JC029", 0, 2);
    let b = board(&mut g, "JC029", 1, 2);
    for seat in [0, 1] {
        for _ in 0..3 {
            let c = g.make_card("JC125", seat);
            g.players[seat].hand.push(c);
        }
    }
    let extra = g.make_card("JC125", 0);
    g.players[0].hand.push(extra);
    tracker(&mut g, 2, true);
    g.players[0].hand.pop();
    let extra = g.make_card("JC125", 1);
    g.players[1].hand.push(extra);
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    g.players[1].hand.pop();
    choose(&mut g, vec![a]);
    assert!(g.pending.is_none());
    assert!(g.board(&b).is_some());
    checkpoint(&g);
}

#[test]
fn blue_jz24_optional_skip_faceup_no_trigger_local_controller_and_actor_order() {
    let mut g = game(2);
    let victim = board(&mut g, "JC029", 0, 2);
    tracker(&mut g, 2, false);
    assert!(g.pending.is_none() && g.stack.is_empty() && g.board(&victim).is_some());
    let mut g = game(2);
    let c = g.make_card("JZ24", 2);
    let id = c.id.clone();
    g.players[2].hand.push(c);
    fund(&mut g, 2, "XQ12", 4);
    apply(
        &mut g,
        2,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    assert!(g.pending.is_none() && g.effects.is_empty());
    for actor in 0..4 {
        let mut g = game(actor);
        let enemies = if actor < 2 { [2, 3] } else { [0, 1] };
        let mut ids = vec![];
        for seat in enemies {
            ids.push(board(&mut g, "JC029", seat, 2));
        }
        let teammate = actor ^ 1;
        board(&mut g, "JC029", teammate, 2);
        tracker(&mut g, actor, true);
        pass_top(&mut g);
        for (seat, id) in enemies.into_iter().zip(ids) {
            assert_eq!(g.pending.as_ref().unwrap().seat, seat);
            choose(&mut g, vec![id]);
        }
        assert!(g.pending.is_none());
    }
}

#[test]
fn blue_jz24_source_change_region_guard_and_death_continuation_priority() {
    for mutation in 0..4 {
        let mut g = game(2);
        let a = board(&mut g, "XQ12", 0, 2);
        let b = board(&mut g, "JC029", 1, 2);
        let source = tracker(&mut g, 2, true);
        match mutation {
            0 => {
                let i = g.regions[2]
                    .cards
                    .iter()
                    .position(|c| c.id == source)
                    .unwrap();
                let c = g.regions[2].cards.remove(i);
                g.regions[1].cards.push(c);
            }
            1 => g.return_hand(&source),
            2 => g.board_mut(&source).unwrap().controller = 1,
            _ => {
                let old = g.regions[2].card.clone();
                g.regions[2].card = g.fresh(old);
            }
        }
        pass_top(&mut g);
        if mutation == 3 {
            assert!(g.pending.is_none());
            assert!(g.board(&a).is_some() && g.board(&b).is_some());
            continue;
        }
        choose(&mut g, vec![a]);
        assert_eq!(g.pending.as_ref().unwrap().seat, 1);
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "jz24_sacrifice");
        let stale = selection(&g, vec![b.clone()]);
        let mut restored = envelope(&g).game;
        choose(&mut g, vec![b.clone()]);
        choose(&mut restored, vec![b]);
        assert_eq!(
            serde_json::to_value(&g).unwrap(),
            serde_json::to_value(&restored).unwrap()
        );
        assert_eq!(g.pending.as_ref().unwrap().seat, 0); // XQ12 Death declaration only after both sacrifices.
        reject(&mut g, 1, stale);
    }
    let mut g = game(2);
    let a = board(&mut g, "JC029", 0, 2);
    tracker(&mut g, 2, true);
    pass_top(&mut g);
    let action = selection(&g, vec![a]);
    let old = g.regions[2].card.clone();
    g.regions[2].card = g.fresh(old);
    reject(&mut g, 0, action);
    let mut g = game(2);
    let b = board(&mut g, "JC029", 1, 2);
    tracker(&mut g, 2, true);
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 1);
    choose(&mut g, vec![b]);
    assert!(g.pending.is_none());
}

const BLUES: [&str; 9] = [
    "XQ12", "XQ16", "JC036", "JZ27", "XQ14", "JC029", "JC030", "JC032", "JZ24",
];
fn blue_draft(n: usize) -> crate::deck::DeckDraft {
    let mut d = crate::deck::preset("watchers").unwrap();
    d.society_id = Some("MSJC03".into());
    d.cards.clear();
    let mut left = n;
    for id in BLUES {
        let count = left.min(3);
        if count > 0 {
            d.cards.push(catalog::DeckEntry {
                card_id: id.into(),
                count,
            });
        }
        left -= count;
    }
    assert_eq!(left, 0);
    d.cards.push(catalog::DeckEntry {
        card_id: "JC125".into(),
        count: 50 - n,
    });
    d
}
#[test]
fn blue_ms03_minimum_pool_and_printed_unique_card_filter_not_character_only() {
    // Printed unique restricts in-play identities, not the three-copy deck limit.
    for n in [24, 25, 27] {
        assert_eq!(crate::deck::validate(blue_draft(n)).is_ok(), n >= 25);
    }
    let s = crate::society::definition("MSJC03").unwrap();
    assert_eq!(
        (&*s.card.name, &*s.subtitle, s.starting_hand),
        ("王座会", "通往至高王座", 6)
    );
    assert_eq!(s.card.subtypes, ["法师结社", "吸血鬼"]);
    assert!(s.card.unique);
    assert_eq!(
        serde_json::to_value(&s.deck_constraints).unwrap(),
        serde_json::json!([{"kind":"minimumColor","color":"蓝","count":25}])
    );
    let a = &definition("MSJC03").abilities[1];
    let Op::Search {
        filter,
        optional,
        to_top,
        visibility,
        ..
    } = &a.ops[0]
    else {
        panic!("finite blue search")
    };
    assert!(!optional && !to_top && *visibility == SearchVisibility::Reveal && a.once_per_game);
    assert!(filter.matches(catalog::card("JZ27")));
    assert!(!filter.matches(catalog::card("JC029")));
    let mut synthetic = catalog::card("JZ27").clone();
    synthetic.kind = "attachment".into();
    assert!(filter.matches(&synthetic));
    synthetic.color = "绿".into();
    assert!(!filter.matches(&synthetic));
}

fn throne_game(actor: usize) -> Game {
    let mut g = Game::new_with_deck(
        "blue-throne".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        blue_draft(25),
        19,
    )
    .unwrap();
    for s in 1..4 {
        g.join_with_deck(format!("P{s}"), blue_draft(25)).unwrap();
    }
    for s in 0..4 {
        g.apply(s, Action::new("ready")).unwrap();
    }
    g.apply(0, Action::new("start")).unwrap();
    assert!(g.players.iter().all(|p| p.hand.len() == 6));
    while let Some(p) = g.pending.clone() {
        g.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
        p.deck.clear();
        p.graveyard.clear();
    }
    for r in &mut g.regions {
        r.cards.clear();
    }
    g.first_team = g.team(actor);
    g.begin_window(Window::Action(g.team(actor)));
    g
}
#[test]
fn blue_ms03_paid_search_restore_private_public_reveal_and_once_across_all_actors() {
    for actor in 0..4 {
        let mut g = throne_game(actor);
        let ids = deck(
            &mut g,
            actor,
            &["JC029", "JZ27", "JC125", "JC036", "LC30", "JZ27"],
        );
        fund(&mut g, actor, "JC125", 4);
        let source = g.players[actor]
            .society_zone
            .card
            .as_ref()
            .unwrap()
            .id
            .clone();
        apply(&mut g, actor, activate(&source, "search-blue-unique"));
        assert_eq!(g.resources(actor), 0);
        assert!(
            g.players[actor]
                .society_zone
                .card
                .as_ref()
                .unwrap()
                .exhausted
        );
        pass_top(&mut g);
        assert_eq!(
            g.pending
                .as_ref()
                .unwrap()
                .choice
                .options
                .iter()
                .map(|o| o.id.clone())
                .collect::<Vec<_>>(),
            vec![ids[1].clone(), ids[5].clone()]
        );
        for viewer in 0..4 {
            assert_eq!(g.view(viewer).pending_choice.is_some(), viewer == actor);
        }
        let mut restored = envelope(&g).game;
        let mut expected = g.clone();
        expected.players[actor].deck.remove(1);
        expected.shuffle_player(actor);
        choose(&mut g, vec![ids[1].clone()]);
        choose(&mut restored, vec![ids[1].clone()]);
        assert_eq!(
            serde_json::to_value(&g).unwrap(),
            serde_json::to_value(&restored).unwrap()
        );
        assert_eq!(g.random, expected.random);
        assert!(g.players[actor]
            .hand
            .iter()
            .any(|c| c.definition == "JZ27" && c.id != ids[1]));
        for viewer in 0..4 {
            assert!(g
                .view(viewer)
                .log
                .iter()
                .any(|e| e.text.contains("展示检索的")));
        }
        g.players[actor]
            .society_zone
            .card
            .as_mut()
            .unwrap()
            .exhausted = false;
        fund(&mut g, actor, "JC125", 4);
        reject(&mut g, actor, activate(&source, "search-blue-unique"));
        if actor == 0 {
            fixture("msjc03-search-result", &g);
        }
    }
}

#[test]
fn blue_ms03_draw_checks_initiative_at_execution_and_empty_search_consumes_once() {
    for first in [0, 1] {
        let mut g = throne_game(0);
        deck(&mut g, 0, &["JC125", "JC125"]);
        fund(&mut g, 0, "JC125", 3);
        let source = g.players[0].society_zone.card.as_ref().unwrap().id.clone();
        apply(&mut g, 0, activate(&source, "drawWithInitiative"));
        g.first_team = first;
        pass_top(&mut g);
        assert_eq!(g.players[0].hand.len(), usize::from(first == 0));
        assert_eq!(g.resources(0), 0);
    }
    let mut g = throne_game(0);
    deck(&mut g, 0, &["JC029", "JC030", "JC036"]);
    fund(&mut g, 0, "JC125", 4);
    let source = g.players[0].society_zone.card.as_ref().unwrap().id.clone();
    apply(&mut g, 0, activate(&source, "search-blue-unique"));
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert!(g.players[0]
        .society_zone
        .used_once_per_game
        .contains("search-blue-unique"));
    g.players[0].society_zone.card.as_mut().unwrap().exhausted = false;
    fund(&mut g, 0, "JC125", 4);
    reject(&mut g, 0, activate(&source, "search-blue-unique"));
}

#[cfg(feature = "native")]
#[tokio::test]
async fn blue_real_host_receipts_keep_original_transition_after_restart_for_both_choices() {
    use crate::room::{RoomCommand, SessionAction};
    use crate::service::{CreateRoom, JoinRoom, Store};
    for elder_choice in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("blue-receipts.sqlite");
        let store = Store::open(&path).unwrap();
        let host = store
            .create(CreateRoom {
                name: "P0".into(),
                mode: "teams".into(),
                deck_id: "watchers".into(),
                deck_draft: None,
            })
            .await
            .unwrap();
        let mut sessions = vec![host];
        for seat in 1..4 {
            sessions.push(
                store
                    .join(JoinRoom {
                        invite_code: sessions[0].invite_code.clone(),
                        name: format!("P{seat}"),
                        deck_id: "watchers".into(),
                        deck_draft: None,
                    })
                    .await
                    .unwrap(),
            );
        }
        drop(store);
        let mut g = if elder_choice { game(0) } else { game(2) };
        if elder_choice {
            deck(
                &mut g,
                0,
                &["JC029", "JC030", "JC125", "JZ24", "JC003", "JC059", "JC029"],
            );
            elder(&mut g, 0);
            pass_top(&mut g);
        } else {
            for seat in [0, 1] {
                for _ in 0..3 {
                    let c = g.make_card("JC125", seat);
                    g.players[seat].hand.push(c);
                }
            }
            let host = board(&mut g, "JC016", 1, 2);
            g.board_mut(&host).unwrap().controller = 0;
            let blade = g.make_card("BQ022", 1);
            g.attachments.push(Attachment {
                card: blade,
                host_id: host,
            });
            board(&mut g, "JC029", 1, 2);
            tracker(&mut g, 2, true);
            pass_top(&mut g);
        }
        g.room_id = sessions[0].room_id.clone();
        g.invite_code = sessions[0].invite_code.clone();
        let room = envelope(&g);
        let initial = serde_json::to_string(&room).unwrap();
        let db = rusqlite::Connection::open(&path).unwrap();
        db.execute(
            "UPDATE rooms SET state=?2,initial_state=?2,revision=?3 WHERE id=?1",
            rusqlite::params![g.room_id, initial, room.revision],
        )
        .unwrap();
        db.execute("DELETE FROM journal WHERE room_id=?1", [&g.room_id])
            .unwrap();
        drop(db);
        let seat = g.pending.as_ref().unwrap().seat;
        let selected = g.pending.as_ref().unwrap().choice.options[0].id.clone();
        let command = RoomCommand {
            command_id: "blue-once".into(),
            expected_version: room.revision,
            action: SessionAction::Game {
                action: selection(&g, vec![selected.clone()]),
            },
        };
        let expected = room.transition(seat, Some(command.clone()), 0).unwrap();
        assert!(expected.error_code.is_none());
        let store = Store::open(&path).unwrap();
        let accepted = store
            .command_at_now(&g.room_id, &sessions[seat].token, command.clone(), 0)
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(&accepted).unwrap(),
            serde_json::to_value(
                crate::room::RoomEnvelope::from_persisted(&expected.state)
                    .unwrap()
                    .view(seat, 0)
            )
            .unwrap()
        );
        drop(store);
        let db = rusqlite::Connection::open(&path).unwrap();
        let before: String = db
            .query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |r| {
                r.get(0)
            })
            .unwrap();
        drop(db);
        let store = Store::open(&path).unwrap();
        let duplicate = store
            .command_at_now(&g.room_id, &sessions[seat].token, command.clone(), 90_000)
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(&accepted).unwrap(),
            serde_json::to_value(&duplicate).unwrap()
        );
        let mut conflict = command.clone();
        conflict.expected_version += 1;
        assert_eq!(
            store
                .command_at_now(&g.room_id, &sessions[seat].token, conflict, 90_000)
                .await
                .unwrap_err()
                .error,
            "command_id_conflict"
        );
        let db = rusqlite::Connection::open(&path).unwrap();
        let after: String = db
            .query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(before, after);
        let count: i64 = db
            .query_row(
                "SELECT count(*) FROM commands WHERE room_id=?1",
                [&g.room_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        if !elder_choice {
            let saved = crate::room::RoomEnvelope::from_persisted(&after).unwrap();
            assert_eq!(saved.pending.as_ref().unwrap().seat, 1);
            assert_eq!(saved.players[1].hand.len(), 4);
        }
        if let Ok(dir) = std::env::var("BLUE_FRONTEND_DIR") {
            std::fs::write(format!("{dir}/host-receipt-{}.json",if elder_choice {"jc032"} else {"jz24"}),serde_json::to_vec(&serde_json::json!({"kind":"host-receipt","stateBefore":initial,"stateAfter":after,"command":command,"expectedTransition":expected,"originalView":accepted,"duplicateView":duplicate,"savedReceiptCount":count})).unwrap()).unwrap();
        }
    }
}
fn tracker(g: &mut Game, actor: usize, trigger: bool) -> String {
    let c = g.make_card("JZ24", actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    fund(g, actor, "XQ12", 5);
    apply(
        g,
        actor,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
    let source = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JZ24")
        .unwrap()
        .id
        .clone();
    apply(
        g,
        actor,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    );
    pass_top(g);
    let p = g.pending.as_ref().unwrap();
    assert_eq!(p.seat, actor);
    assert_eq!(p.choice.allow_decline, Some(true));
    let new_source = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JZ24")
        .unwrap()
        .id
        .clone();
    let ids = if trigger {
        vec![p.choice.options[0].id.clone()]
    } else {
        vec![]
    };
    choose(g, ids);
    new_source
}

#[test]
fn blue_jz24_two_enemies_bq022_return_four_cards_still_sacrifices_and_restores() {
    let mut g = game(2);
    for s in [0, 1] {
        for _ in 0..3 {
            let c = g.make_card("JC125", s);
            g.players[s].hand.push(c);
        }
    }
    let host = board(&mut g, "JC016", 1, 2);
    g.board_mut(&host).unwrap().controller = 0;
    let blade = g.make_card("BQ022", 1);
    let blade_id = blade.id.clone();
    g.attachments.push(Attachment {
        card: blade,
        host_id: host.clone(),
    });
    let victim = board(&mut g, "JC029", 1, 2);
    let far = board(&mut g, "JC029", 0, 1);
    let hidden = board(&mut g, "JC029", 0, 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    let source = tracker(&mut g, 2, true);
    pass_top(&mut g);
    let p = g.pending.as_ref().unwrap();
    assert_eq!(p.seat, 0);
    assert_eq!(p.choice.kind, "jz24_sacrifice");
    assert_eq!(
        p.choice
            .options
            .iter()
            .map(|o| o.id.clone())
            .collect::<Vec<_>>(),
        vec![host.clone()]
    );
    for bad in [
        vec![],
        vec![host.clone(), host.clone()],
        vec![victim.clone()],
        vec![far],
        vec![hidden],
        vec![source],
        vec![blade_id.clone()],
    ] {
        let a = selection(&g, bad);
        reject(&mut g, 0, a);
    }
    let wrong = selection(&g, vec![host.clone()]);
    reject(&mut g, 1, wrong);
    fixture("jz24-first-enemy", &g);
    choose(&mut g, vec![host.clone()]);
    assert!(g.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC016" && c.id != host));
    assert_eq!(g.players[1].hand.len(), 4);
    assert!(g.players[1]
        .hand
        .iter()
        .any(|c| c.definition == "BQ022" && c.id != blade_id));
    assert!(!g.attachments.iter().any(|a| a.card.id == blade_id));
    let p = g.pending.as_ref().unwrap();
    assert_eq!(p.seat, 1);
    assert_eq!(p.choice.options[0].id, victim);
    for s in 0..4 {
        assert_eq!(g.view(s).pending_choice.is_some(), s == 1);
    }
    fixture("jz24-bq022-second-enemy", &g);
    let mut restored = envelope(&g).game;
    choose(&mut g, vec![victim.clone()]);
    choose(&mut restored, vec![victim.clone()]);
    assert_eq!(
        serde_json::to_value(&g).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    assert!(g.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC029" && c.id != victim));
    assert!(g.pending.is_none());
    checkpoint(&g);
    fixture("jz24-complete", &g);
}

#[test]
fn blue_jc032_top_six_printed_filter_private_restore_fresh_hidden_and_whole_deck_shuffle() {
    let mut g = game(0);
    let ids = deck(
        &mut g,
        0,
        &[
            "JC030", "JC029", "XQ16", "JZ24", "JC003", "JC030", "JC029", "JC059",
        ],
    );
    // JC030's in-play subtype must not become a printed deck Vampire.
    fund(&mut g, 0, "XQ12", 2);
    let source = elder(&mut g, 0);
    assert!(!g.board(&source).unwrap().1.exhausted);
    assert_eq!(g.turn_ability_usage.len(), 1);
    pass_top(&mut g);
    let p = g.pending.as_ref().unwrap();
    assert_eq!(p.choice.min, Some(1));
    assert_eq!(p.choice.max, Some(1));
    assert_eq!(
        p.choice
            .options
            .iter()
            .map(|o| o.id.clone())
            .collect::<Vec<_>>(),
        vec![ids[1].clone(), ids[3].clone()]
    );
    let before = serde_json::to_value(&g).unwrap();
    for s in 0..4 {
        let view = g.view(s);
        assert_eq!(view.pending_choice.is_some(), s == 0);
        if s == 0 {
            assert_eq!(
                view.pending_choice
                    .unwrap()
                    .preview_cards
                    .iter()
                    .map(|c| c.instance_id.clone())
                    .collect::<Vec<_>>(),
                ids[..6]
            );
        }
    }
    assert_eq!(before, serde_json::to_value(&g).unwrap());
    fixture("jc032-positive", &g);
    for bad in [
        vec![],
        vec![ids[6].clone()],
        vec![ids[0].clone()],
        vec![ids[2].clone()],
        vec![ids[1].clone(), ids[3].clone()],
        vec![ids[1].clone(), ids[1].clone()],
    ] {
        let a = selection(&g, bad);
        reject(&mut g, 0, a);
    }
    let mut restored = envelope(&g).game;
    let mut expected = g.clone();
    expected.players[0].deck.remove(1);
    expected.shuffle_player(0);
    choose(&mut g, vec![ids[1].clone()]);
    choose(&mut restored, vec![ids[1].clone()]);
    assert_eq!(
        serde_json::to_value(&g).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    assert_eq!(g.random, expected.random);
    assert_eq!(
        serde_json::to_value(&g.players[0].deck).unwrap(),
        serde_json::to_value(&expected.players[0].deck).unwrap()
    );
    assert!(g.pending.is_none() && g.stack.is_empty() && g.effects.is_empty());
    let hidden = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC029")
        .unwrap();
    assert_ne!(hidden.id, ids[1]);
    assert!(hidden.face_down);
    assert_eq!((hidden.owner, hidden.controller), (0, 0));
    for s in 0..4 {
        let v = g.view(s);
        let c = v.regions[2]
            .characters
            .iter()
            .find(|c| c.instance_id == hidden.id)
            .unwrap();
        assert_eq!(c.card_id.is_some(), s == 0);
        assert!(v.log.iter().any(|e| e.text.contains("展示检索的 新生血族")));
    }
    fixture("jc032-hidden-result", &g);
    reject(&mut g, 0, activate(&source, "top-six-vampire-hidden"));
    checkpoint(&g);
}

#[test]
fn blue_jc032_zero_hit_private_confirmation_and_empty_deck_no_rng() {
    for len in [0, 1, 5, 6, 8] {
        let mut g = game(0);
        let defs = vec!["JC030"; len];
        let ids = deck(&mut g, 0, &defs);
        elder(&mut g, 0);
        let rng = g.random;
        pass_top(&mut g);
        if len == 0 {
            assert!(g.pending.is_none());
            assert_eq!(g.random, rng);
            continue;
        }
        let p = g.pending.as_ref().unwrap();
        assert_eq!(p.choice.min, Some(0));
        assert_eq!(p.choice.max, Some(0));
        assert_eq!(p.choice.allow_decline, Some(false));
        assert_eq!(
            g.view(0).pending_choice.unwrap().preview_cards.len(),
            len.min(6)
        );
        let a = selection(&g, vec![ids[0].clone()]);
        reject(&mut g, 0, a);
        if len == 6 {
            fixture("jc032-zero-hit", &g);
        }
        let mut expected = g.clone();
        expected.shuffle_player(0);
        choose(&mut g, vec![]);
        assert_eq!(g.random, expected.random);
        assert_eq!(
            serde_json::to_value(&g.players[0].deck).unwrap(),
            serde_json::to_value(&expected.players[0].deck).unwrap()
        );
        assert!(g.regions[2].cards.iter().all(|c| c.definition != "JC030"));
        checkpoint(&g);
    }
}
