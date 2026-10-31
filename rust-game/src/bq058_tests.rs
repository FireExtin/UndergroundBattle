use crate::{catalog, model::*, unit_support::*};
#[test]
fn bq058_partial_catalog_keeps_task_search_and_public_selection_closed() {
    let c = catalog::catalog().cards.iter().find(|c| c.id == "BQ058");
    assert!(c.is_some(), "original BQ058 non-task slice is missing");
    assert!(!c.unwrap().supported);
    assert_current_serial_catalog();
}
use crate::jc029_tests::{envelope, pass_top};
use crate::{bq058 as b, rules::*};
fn setup() -> (Game, String, String) {
    let mut g = game();
    let observer = field(&mut g, b::ID, 0);
    let core = field(&mut g, "BQ046", 0);
    g.core_zones.push(crate::core_zone::CoreZone {
        owner: 0,
        definition: "BQ046".into(),
        rest: 0,
        card: None,
    });
    restore(&mut g);
    (g, observer, core)
}
fn win(g: &mut Game, seat: usize) {
    g.begin_window(Window::Win(0, seat));
    restore(g);
    for _ in 0..4 {
        let seat = g
            .players
            .iter()
            .find(|p| {
                g.legal_actions(p.seat)
                    .iter()
                    .any(|a| a.action.kind == "pass")
            })
            .unwrap()
            .seat;
        act(g, seat, Action::new("pass"));
    }
}
fn settle_choices(g: &mut Game) {
    for _ in 0..24 {
        if let Some(p) = &g.pending {
            let ids = if p.choice.kind == "core-return" {
                vec!["destination".into()]
            } else {
                p.choice
                    .options
                    .iter()
                    .take(p.choice.min.unwrap_or(0))
                    .map(|o| o.id.clone())
                    .collect()
            };
            choose(g, ids);
        } else if !g.stack.is_empty() {
            pass_top(g);
        } else {
            g.drive().unwrap();
            if g.pending.is_none() {
                restore(g);
                return;
            }
        }
    }
    panic!("bounded region return did not settle");
}
fn reject_state(reference: &Game, bad: &Game) {
    assert!(Game::from_persisted(&serde_json::to_string(bad).unwrap()).is_err());
    let mut room = envelope(reference);
    room.game = bad.clone();
    assert!(
        crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err()
    );
}
#[test]
fn bq058_permanent_core_bonus_stacks_ignores_exhaustion_and_not_teammate() {
    let (mut g, observer, core) = setup();
    g.first_team = 1;
    let before = g.current_icons(g.board(&core).unwrap().1, 0);
    assert_eq!(g.bq058_bonus(g.board(&core).unwrap().1, 0), 1);
    assert!(g.has_mobility(g.board(&core).unwrap().1));
    field(&mut g, b::ID, 0);
    let after = g.current_icons(g.board(&core).unwrap().1, 0);
    assert_eq!(
        (after.investigation, after.combat),
        (before.investigation + 1, before.combat + 1)
    );
    g.board_mut(&observer).unwrap().exhausted = true;
    assert_eq!(g.bq058_bonus(g.board(&core).unwrap().1, 0), 2);
    g.board_mut(&observer).unwrap().face_down = true;
    assert_eq!(g.bq058_bonus(g.board(&core).unwrap().1, 0), 1);
    g.board_mut(&core).unwrap().controller = 1;
    assert_eq!(g.bq058_bonus(g.board(&core).unwrap().1, 0), 0);
    assert!(!g.has_mobility(g.board(&core).unwrap().1));
    g.board_mut(&core).unwrap().controller = 0;
    g.board_mut(&core).unwrap().face_down = true;
    assert!(!g.has_mobility(g.board(&core).unwrap().1));
}
#[test]
fn bq058_bonus_requires_actual_selected_core_and_live_source() {
    let (mut g, observer, core) = setup();
    let ordinary = field(&mut g, "JC125", 0);
    assert_eq!(g.bq058_bonus(g.board(&ordinary).unwrap().1, 0), 0);
    assert_eq!(g.bq058_bonus(g.board(&core).unwrap().1, 1), 0);
    g.board_mut(&observer).unwrap().controller = 1;
    assert!(!g.has_mobility(g.board(&core).unwrap().1));
    g.board_mut(&observer).unwrap().controller = 0;
    g.core_zones.clear();
    assert!(!g.has_mobility(g.board(&core).unwrap().1));
}
#[test]
fn bq058_granted_mobility_restores_and_survives_observer_departure() {
    for leave in [false, true] {
        let (mut g, observer, core) = setup();
        g.begin_window(Window::Mobility);
        let (r, c) = g.board(&core).unwrap();
        let source = g.mobility_source(c, r);
        g.emit_event(0, source, Event::ConfrontationStart);
        g.drive().unwrap();
        restore(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 1);
        choose(&mut g, vec!["region:1".into()]);
        if leave {
            g.remove_dead(&observer, RemovalCause::Destroy);
            restore(&mut g);
        }
        pass_top(&mut g);
        assert_eq!(g.board(&core).unwrap().0, 1);
        restore(&mut g);
    }
}
#[test]
fn bq058_mobility_rejects_transplant_and_exhausted_recipient() {
    let (mut g, _, core) = setup();
    g.begin_window(Window::Mobility);
    let (r, c) = g.board(&core).unwrap();
    let source = g.mobility_source(c, r);
    g.emit_event(0, source.clone(), Event::ConfrontationStart);
    g.drive().unwrap();
    restore(&mut g);
    let mut bad = g.clone();
    if let ChoiceResolution::Declare { declaration, .. } =
        &mut bad.pending.as_mut().unwrap().resolution
    {
        if let Op::BQ058Move { proof } = &mut declaration.ability.ops[0] {
            proof.recipient.owner = 1;
        }
    }
    reject_state(&g, &bad);
    choose(&mut g, vec!["region:1".into()]);
    g.hide_in_place(&core);
    pass_top(&mut g);
    assert!(g.regions[1].cards.is_empty());
    restore(&mut g);
    let (mut g, _, core) = setup();
    g.board_mut(&core).unwrap().exhausted = true;
    g.begin_window(Window::Mobility);
    let (r, c) = g.board(&core).unwrap();
    let source = g.mobility_source(c, r);
    g.emit_event(0, source, Event::ConfrontationStart);
    g.drive().unwrap();
    assert!(g.pending.is_none());
}
#[test]
fn bq058_real_win_moves_core_before_return_preserves_identity_attachment_and_score() {
    let (mut g, _, core) = setup();
    let attachment = g.make_card("JC073", 0);
    let aid = attachment.id.clone();
    g.attachments.push(Attachment {
        card: attachment,
        host_id: core.clone(),
    });
    g.board_mut(&core).unwrap().shield = 1;
    let original = g.board(&core).unwrap().1.clone();
    let region = g.regions[0].card.definition.clone();
    win(&mut g, 0);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "bq058-destination");
    restore(&mut g);
    choose(&mut g, vec!["region:1".into()]);
    settle_choices(&mut g);
    assert_eq!(g.board(&core).unwrap().0, 1);
    assert_eq!(g.board(&core).unwrap().1, &original);
    assert!(g
        .attachments
        .iter()
        .any(|a| a.card.id == aid && a.host_id == core));
    assert!(g.players[0]
        .score_cards
        .iter()
        .any(|c| c.definition == region));
    assert!(!g.players[0].deck.iter().any(|c| c.definition == "BQ046"));
    restore(&mut g);
}
#[test]
fn bq058_decline_at_trigger_or_destination_keeps_normal_return() {
    for accept in [false, true] {
        let (mut g, _, core) = setup();
        win(&mut g, 0);
        if accept {
            choose(&mut g, vec!["accept".into()]);
            pass_top(&mut g);
        }
        choose(&mut g, vec![]);
        settle_choices(&mut g);
        assert!(g.board(&core).is_none());
        assert!(g.players[0].deck.iter().any(|c| c.definition == "BQ046"));
        restore(&mut g);
    }
}
#[test]
fn bq058_win_is_personal_and_source_may_leave_after_trigger() {
    let (mut g, _, core) = setup();
    win(&mut g, 1);
    assert!(!g
        .pending
        .as_ref()
        .is_some_and(|p| p.choice.kind == "trigger"));
    settle_choices(&mut g);
    assert!(g.board(&core).is_none());
    let (mut g, observer, core) = setup();
    win(&mut g, 0);
    choose(&mut g, vec!["accept".into()]);
    g.remove_dead(&observer, RemovalCause::Destroy);
    restore(&mut g);
    pass_top(&mut g);
    choose(&mut g, vec!["region:1".into()]);
    settle_choices(&mut g);
    assert_eq!(g.board(&core).unwrap().0, 1);
}
#[test]
fn bq058_core_moved_in_response_is_not_moved_again_or_returned() {
    let (mut g, _, core) = setup();
    win(&mut g, 0);
    choose(&mut g, vec!["accept".into()]);
    g.move_on_board_instance(&core, 1);
    restore(&mut g);
    pass_top(&mut g);
    settle_choices(&mut g);
    assert_eq!(g.board(&core).unwrap().0, 1);
}
#[test]
fn bq058_win_pending_rejects_bad_context_duplicate_and_destination() {
    let (mut g, _, _) = setup();
    win(&mut g, 0);
    let mut bad = g.clone();
    if let ChoiceResolution::Declare { declaration, .. } =
        &mut bad.pending.as_mut().unwrap().resolution
    {
        if let Op::BQ058Evacuate { win: Some(w) } = &mut declaration.ability.ops[0] {
            w.region = 1;
        }
    }
    reject_state(&g, &bad);
    let mut bad = g.clone();
    if let ChoiceResolution::Declare { declaration, .. } = &bad.pending.as_ref().unwrap().resolution
    {
        bad.effects.push_front(Effect::Declare {
            declaration: declaration.clone(),
        });
    }
    reject_state(&g, &bad);
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    let mut bad = g.clone();
    bad.pending.as_mut().unwrap().choice.options[0].id = "region:2".into();
    reject_state(&g, &bad);
    let mut bad = g.clone();
    bad.pending.as_mut().unwrap().choice.min = Some(1);
    reject_state(&g, &bad);
    let choice_id = g.pending.as_ref().map(|p| p.choice.id.clone());
    reject(
        &mut g,
        0,
        Action {
            choice_id,
            selected: Some(vec!["region:2".into()]),
            ..Action::new("choose")
        },
    );
}

#[test]
fn bq058_multiple_observers_do_not_move_core_twice() {
    let (mut g, _, core) = setup();
    field(&mut g, b::ID, 0);
    win(&mut g, 0);
    choose(&mut g, vec!["accept".into()]);
    // The ordinary trigger ordering can ask for the other observer before
    // resolving the first stack item; both observations retain the same core.
    if g.pending.is_some() {
        choose(&mut g, vec!["accept".into()]);
    }
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "bq058-destination");
    choose(&mut g, vec!["region:1".into()]);
    settle_choices(&mut g);
    assert_eq!(g.board(&core).unwrap().0, 1);
    restore(&mut g);
}
#[test]
fn bq058_paid_deployment_does_not_claim_task_search() {
    let mut g = game();
    fund(&mut g, 0, "JC056", 4);
    let id = held(&mut g, b::ID, 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert!(g.regions[0].cards.iter().any(|c| c.definition == b::ID));
    assert!(crate::rules::definition(b::ID)
        .abilities
        .iter()
        .all(|a| a.event != Some(Event::Enter)));
    assert!(!catalog::card(b::ID).supported);
    restore(&mut g);
}
#[test]
fn bq058_native_bounded_room_export() {
    use crate::abi112_review_play_tests::Trace;
    let mut cases = vec![];
    for accept in [false, true] {
        let (mut g, _, core) = setup();
        g.begin_window(Window::Win(0, 0));
        let mut t = Trace::new_preserving_rng(
            g,
            format!("bq058-win-accept-{accept}"),
            580 + usize::from(accept),
        );
        t.until_pending();
        t.choose(if accept {
            vec!["accept".into()]
        } else {
            vec![]
        });
        if accept {
            t.until_pending();
            assert_eq!(
                t.room.game.pending.as_ref().unwrap().choice.kind,
                "bq058-destination"
            );
            t.choose(vec!["region:1".into()]);
        }
        for _ in 0..24 {
            if let Some(p) = &t.room.game.pending {
                let ids = if p.choice.kind == "core-return" {
                    vec!["destination".into()]
                } else {
                    p.choice
                        .options
                        .iter()
                        .take(p.choice.min.unwrap_or(0))
                        .map(|o| o.id.clone())
                        .collect()
                };
                t.choose(ids);
            } else if !t.room.game.stack.is_empty() {
                t.pass();
            } else {
                break;
            }
        }
        assert!(
            t.room.game.pending.is_none()
                && t.room.game.stack.is_empty()
                && t.room.game.effects.is_empty()
        );
        assert_eq!(
            t.room.game.board(&core).map(|(r, _)| r),
            accept.then_some(1)
        );
        cases.push(t.value());
    }
    let (mut g, _, core) = setup();
    g.begin_window(Window::Mobility);
    let (r, c) = g.board(&core).unwrap();
    let source = g.mobility_source(c, r);
    g.emit_event(0, source, Event::ConfrontationStart);
    g.drive().unwrap();
    let mut t = Trace::new_preserving_rng(g, "bq058-granted-mobility".into(), 582);
    t.choose(vec!["region:1".into()]);
    for _ in 0..8 {
        if t.room.game.stack.is_empty() {
            break;
        }
        t.pass();
    }
    assert_eq!(t.room.game.board(&core).unwrap().0, 1);
    cases.push(t.value());
    if let Ok(path) = std::env::var("BQ058_NATIVE_OUT") {
        std::fs::write(path,serde_json::to_vec(&serde_json::json!({"catalog":catalog::catalog(),"scope":"three prepared layouts, normal Room commands; not natural/browser play; task search absent","cases":cases})).unwrap()).unwrap();
    }
}
