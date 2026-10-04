//! Printed JC004 and the finite resolution-time state query.
//! Layout changes below are explicit unit fixtures, not natural public play.
use crate::{catalog, deck, model::*, room::RoomEnvelope, rules::*};

fn game() -> Game {
    let mut g = Game::new(
        "jc004-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        4,
    )
    .unwrap();
    for seat in 1..4 {
        g.join(format!("P{seat}"), "watchers".into()).unwrap();
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
    }
    for r in &mut g.regions {
        r.cards.clear();
    }
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    g
}
fn board(g: &mut Game, definition: &str, owner: usize, region: usize) -> String {
    let c = g.make_card(definition, owner);
    let id = c.id.clone();
    g.regions[region].cards.push(c);
    id
}
fn hand(g: &mut Game, definition: &str, owner: usize) -> String {
    let c = g.make_card(definition, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn fund(g: &mut Game, actor: usize) {
    for _ in 0..4 {
        let c = g.make_card("JC003", actor);
        g.players[actor].assets.push(c);
    }
}
fn pass_top(g: &mut Game) {
    let count = g.stack.len();
    assert!(count > 0);
    for _ in 0..32 {
        if g.stack.len() < count || g.pending.is_some() {
            return;
        }
        let seat = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .expect("response pass");
        g.apply(seat, Action::new("pass")).unwrap();
    }
    panic!("stack did not progress");
}
fn choose(g: &mut Game, selected: Vec<String>) {
    let p = g.pending.clone().unwrap();
    g.apply(
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(selected),
            ..Action::new("choose")
        },
    )
    .unwrap();
}
fn enter(g: &mut Game, region: usize) -> String {
    fund(g, 0);
    let source = hand(g, "JC004", 0);
    g.apply(
        0,
        Action {
            card_id: Some(source.clone()),
            region: Some(region),
            ..Action::new("deploy")
        },
    )
    .unwrap();
    assert_eq!(g.resources(0), 0);
    pass_top(g);
    let new = g.regions[region]
        .cards
        .iter()
        .find(|c| c.definition == "JC004" && c.owner == 0)
        .unwrap()
        .id
        .clone();
    assert_ne!(new, source);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
    new
}
fn bound(g: &mut Game, target: &str) -> String {
    let source = enter(g, 0);
    choose(g, vec![target.into()]);
    let frame = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert!(frame.already_paid.is_empty());
    assert_eq!(frame.targets[0].id, target);
    assert!(matches!(frame.steps[0].op, Op::IfTargetExhausted { .. }));
    source
}
fn rejects(g: &mut Game, seat: usize, action: Action) {
    let before = serde_json::to_string(g).unwrap();
    assert!(g.apply(seat, action).is_err());
    assert_eq!(serde_json::to_string(g).unwrap(), before);
}

#[test]
fn jc004_complete_printed_definition_and_candidate_identity_are_isolated() {
    let d = catalog::card("JC004");
    assert_eq!(
        (
            &*d.name,
            &*d.kind,
            d.cost,
            &*d.color,
            &*d.magic,
            &*d.society
        ),
        ("力场法师", "character", 4, "黄", "心灵", "帷幕守望")
    );
    assert_eq!(d.loyalty, ["黄色", "黄色"]);
    assert_eq!(d.subtypes, ["人类", "法师"]);
    assert_eq!(
        d.permanent_icons,
        Icons {
            investigation: 1,
            influence: 1,
            combat: 0
        }
    );
    assert_eq!(
        d.temporary_icons,
        Icons {
            investigation: 0,
            influence: 0,
            combat: 1
        }
    );
    assert_eq!(d.defense, Some(2));
    assert!(!d.unique && d.keywords.is_empty());
    assert!(d.text.contains("本地区的目标角色") && !d.text.contains("暗藏"));
    let spec = &crate::rules::definition("JC004").abilities[0];
    assert_eq!(spec.event, Some(Event::Enter));
    assert_eq!(spec.response_policy, ResponsePolicy::Respondable);
    assert!(spec.costs.is_empty() && spec.modes.is_empty());
    assert_eq!(spec.targets.len(), 1);
    assert_eq!(spec.targets[0].kind, EntityKind::Character);
    assert_eq!(spec.targets[0].range, Range::SourceRegion);
    assert_eq!(spec.targets[0].relation, Relation::Any);
    assert_eq!(
        catalog::ENGINE_VERSION,
        "rust-v0.2.17-protection-batch-candidate"
    );
    assert_eq!(
        catalog::POOL_VERSION,
        "limited-v2.14-protection-batch-candidate"
    );
    let mut draft = deck::preset("watchers").unwrap();
    draft.cards = vec![
        catalog::DeckEntry {
            card_id: "JC004".into(),
            count: 3,
        },
        catalog::DeckEntry {
            card_id: "JC125".into(),
            count: 47,
        },
    ];
    assert!(deck::validate(draft.clone()).is_ok());
    draft.cards[0].count = 4;
    draft.cards[1].count = 46;
    assert!(deck::validate(draft).is_err());
    assert!(catalog::catalog().cards.iter().any(|c| c.id == "JC008"));
    let mut g = game();
    let id = board(&mut g, "JC004", 0, 0);
    assert_eq!(
        g.icons(g.board(&id).unwrap().1, 0),
        Icons {
            investigation: 1,
            combat: 1,
            influence: 1
        }
    );
    g.first_team = 1;
    assert_eq!(
        g.icons(g.board(&id).unwrap().1, 0),
        Icons {
            investigation: 1,
            combat: 0,
            influence: 1
        }
    );
    g.board_mut(&id).unwrap().exhausted = true;
    assert_eq!(g.icons(g.board(&id).unwrap().1, 0), Icons::default());
    let mut old = RoomEnvelope::from_game(game());
    old.versions.engine = "rust-v0.2.9".into();
    old.versions.card_pool = "limited-v2.6".into();
    old.game.versions = old.versions.clone();
    assert!(RoomEnvelope::from_persisted(&serde_json::to_string(&old).unwrap()).is_err());
}

#[test]
fn jc004_ready_only_exhausts_and_exhausted_only_returns_at_resolution_after_restore() {
    for declared in [false, true] {
        for resolved in [false, true] {
            let mut g = game();
            let target = board(&mut g, "JC125", 1, 0);
            g.board_mut(&target).unwrap().exhausted = declared;
            bound(&mut g, &target);
            // Explicit response-state unit fixture, covering both directions even
            // though this pool has no admitted fast ready operation.
            g.board_mut(&target).unwrap().exhausted = resolved;
            g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
            pass_top(&mut g);
            if resolved {
                assert!(g.board(&target).is_none());
                let returned = &g.players[1].hand[0];
                assert_eq!(returned.definition, "JC125");
                assert_ne!(returned.id, target);
            } else {
                assert!(g.board(&target).unwrap().1.exhausted);
                assert!(g.players[1].hand.is_empty());
            }
            assert_eq!(g.resources(0), 0);
            let state = serde_json::to_string(&g).unwrap();
            let mut restored = Game::from_persisted(&state).unwrap();
            restored.drive().unwrap();
            assert_eq!(serde_json::to_string(&restored).unwrap(), state);
        }
    }
}

#[test]
fn jc004_actual_fast_exhaust_response_changes_the_branch_without_extra_cost() {
    let mut g = game();
    let target = board(&mut g, "JC125", 1, 0);
    let guard = board(&mut g, "JC003", 2, 1);
    fund(&mut g, 2);
    bound(&mut g, &target);
    g.apply(0, Action::new("pass")).unwrap();
    g.apply(1, Action::new("pass")).unwrap();
    g.apply(
        2,
        Action {
            card_id: Some(guard.clone()),
            target_id: Some(target.clone()),
            ability_id: Some("exhaust".into()),
            ..Action::new("activate")
        },
    )
    .unwrap();
    assert_eq!(g.resources(2), 2);
    assert!(g.board(&guard).unwrap().1.exhausted);
    pass_top(&mut g);
    assert!(g.board(&target).unwrap().1.exhausted);
    let encoded = serde_json::to_string(&g).unwrap();
    g = Game::from_persisted(&encoded).unwrap();
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    assert_eq!(g.players[1].hand[0].definition, "JC125");
}

#[test]
fn jc004_character_targets_include_self_friends_and_enemies_but_no_hidden_or_remote_or_barrier() {
    let mut g = game();
    let friend = board(&mut g, "JC125", 1, 0);
    let enemy = board(&mut g, "JC125", 2, 0);
    let hidden = board(&mut g, "JC004", 1, 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    g.board_mut(&hidden).unwrap().controller = 3;
    let remote = board(&mut g, "JC125", 1, 1);
    let barrier = board(&mut g, "JZ08", 2, 0);
    let source = enter(&mut g, 0);
    let p = g.pending.clone().unwrap();
    let ids: Vec<_> = p.choice.options.iter().map(|o| o.id.as_str()).collect();
    for id in [&friend, &enemy, &source] {
        assert!(ids.contains(&id.as_str()));
    }
    for id in [&hidden, &remote, &barrier] {
        assert!(!ids.contains(&id.as_str()));
    }
    for seat in 0..4 {
        let v = g.view(seat);
        let c = v.regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == hidden)
            .unwrap();
        assert_eq!(
            c.card_id.as_deref(),
            if seat == 3 { Some("JC004") } else { None }
        );
        if seat != 3 {
            assert_eq!(c.name, "暗藏者");
            assert!(c.text.is_none() && c.icons.is_none());
        }
        assert_eq!(v.pending_choice.is_some(), seat == 0);
    }
    for id in [hidden, remote, barrier] {
        rejects(
            &mut g,
            0,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(vec![id]),
                ..Action::new("choose")
            },
        );
    }
    choose(&mut g, vec![]);
    assert!(g.stack.is_empty());
    assert!(g.board(&source).is_some());
}

#[test]
fn jc004_guard_cancels_reentered_hidden_remote_and_shielded_targets_before_the_query() {
    for change in ["reenter", "hide", "remote", "shield"] {
        let mut g = game();
        let target = board(&mut g, "JC125", 2, 0);
        bound(&mut g, &target);
        let actual = match change {
            "reenter" => {
                let (_, c) = g.leave_board(&target).unwrap();
                let c = g.reset_zone_card(c);
                let id = c.id.clone();
                g.regions[0].cards.push(c);
                id
            }
            "hide" => {
                g.board_mut(&target).unwrap().face_down = true;
                target.clone()
            }
            "remote" => {
                let (_, c) = g.remove_board(&target).unwrap();
                g.regions[1].cards.push(c);
                target.clone()
            }
            "shield" => {
                g.board_mut(&target).unwrap().shield = 1;
                target.clone()
            }
            _ => unreachable!(),
        };
        g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_top(&mut g);
        assert!(!g.board(&actual).unwrap().1.exhausted);
        assert!(g.players[2].hand.is_empty());
        if change == "shield" {
            assert_eq!(g.board(&actual).unwrap().1.shield, 0);
        }
        assert!(g.log.last().is_some());
        assert_eq!(g.resources(0), 0);
    }
    // Ordinary in-play movement preserves identity: out-and-back is legal at
    // the guard, while a target still outside the snapshotted source region is not.
    let mut g = game();
    let target = board(&mut g, "JC125", 1, 0);
    bound(&mut g, &target);
    let (_, c) = g.remove_board(&target).unwrap();
    g.regions[1].cards.push(c);
    let (_, c) = g.remove_board(&target).unwrap();
    g.regions[0].cards.push(c);
    pass_top(&mut g);
    assert!(g.board(&target).unwrap().1.exhausted);
}

#[test]
fn jc004_return_uses_owner_and_clears_identity_controller_markers_and_attachment() {
    let mut g = game();
    let target = board(&mut g, "JC004", 1, 0);
    let c = g.board_mut(&target).unwrap();
    c.controller = 0;
    c.exhausted = true;
    c.wounds = 1;
    c.shield = 2;
    let equipment = g.make_card("BQ022", 3);
    let equipment_id = equipment.id.clone();
    g.attachments.push(Attachment {
        card: equipment,
        host_id: target.clone(),
    });
    bound(&mut g, &target);
    pass_top(&mut g);
    assert!(g.board(&target).is_none() && g.attachments.is_empty());
    let c = g.players[1]
        .hand
        .iter()
        .find(|c| c.definition == "JC004")
        .unwrap();
    assert_ne!(c.id, target);
    assert_eq!((c.owner, c.controller), (1, 1));
    assert!(!c.exhausted && !c.face_down);
    assert_eq!((c.damage, c.wounds, c.shield), (0, 0, 0));
    let attachment = g.players[3]
        .hand
        .iter()
        .find(|c| c.definition == "BQ022")
        .unwrap();
    assert_ne!(attachment.id, equipment_id);
    assert!(!g.players[0].hand.iter().any(|c| c.definition == "JC004"));
}

#[test]
fn jc004_paid_deploy_requires_four_assets_two_yellow_and_reveal_preserves_orientation() {
    let mut g = game();
    let source = hand(&mut g, "JC004", 0);
    for _ in 0..4 {
        let c = g.make_card("JC125", 0);
        g.players[0].assets.push(c);
    }
    rejects(
        &mut g,
        0,
        Action {
            card_id: Some(source.clone()),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    let mut g = game();
    let hidden = board(&mut g, "JC004", 0, 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    g.board_mut(&hidden).unwrap().exhausted = true;
    fund(&mut g, 0);
    g.apply(
        0,
        Action {
            card_id: Some(hidden.clone()),
            ..Action::new("reveal")
        },
    )
    .unwrap();
    pass_top(&mut g);
    let source = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC004")
        .unwrap();
    assert_ne!(source.id, hidden);
    assert!(source.exhausted && !source.face_down);
    assert_eq!(g.resources(0), 0);
    assert!(g.pending.is_some());
    choose(&mut g, vec![]);
}

#[test]
fn jc004_branch_cursor_survives_a_following_choice_and_never_replays_effect() {
    for exhausted in [false, true] {
        let mut g = game();
        let target = board(&mut g, "JC125", 1, 0);
        g.board_mut(&target).unwrap().exhausted = exhausted;
        bound(&mut g, &target);
        let mut frame = g.stack.pop().unwrap().frame.unwrap();
        // Shared interpreter continuation regression; JC004 itself has no forecast.
        frame.steps.push(Step {
            context: 0,
            op: Op::Forecast {
                player: PlayerRef::Actor,
                count: 2,
            },
        });
        g.resolve_frame(frame).unwrap();
        let p = g.pending.clone().unwrap();
        if let ChoiceResolution::Frame { frame, .. } = &p.resolution {
            assert_eq!(frame.cursor, 3);
            assert!(matches!(frame.guard, GuardState::Accepted));
        } else {
            panic!("expected saved continuation");
        }
        let returned_ids: Vec<_> = g.players[1].hand.iter().map(|c| c.id.clone()).collect();
        g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
        g.apply(
            0,
            Action {
                choice_id: Some(p.choice.id),
                top: Some(p.choice.options.iter().map(|o| o.id.clone()).collect()),
                bottom: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
        assert_eq!(
            g.players[1]
                .hand
                .iter()
                .map(|c| c.id.clone())
                .collect::<Vec<_>>(),
            returned_ids
        );
        if !exhausted {
            assert!(g.board(&target).unwrap().1.exhausted);
        }
    }
}

#[test]
fn jc004_finite_declarations_reject_wrong_slot_source_operations_nesting_and_nonboard_target() {
    let spec = crate::rules::definition("JC004").abilities[0].clone();
    for branch in [
        Op::Exhaust(EntityRef::Source),
        Op::Move(EntityRef::Target(0), Destination::ActorHand),
        Op::Draw {
            player: PlayerRef::Actor,
            count: 1,
            end: DeckEnd::Top,
        },
        spec.ops[0].clone(),
    ] {
        let mut invalid = spec.clone();
        invalid.ops = vec![Op::IfTargetExhausted {
            slot: 0,
            exhausted: Box::new(branch),
            ready: Box::new(Op::Exhaust(EntityRef::Target(0))),
        }];
        assert!(crate::rules::validate_ability("JC004", &invalid).is_err());
    }
    let mut invalid = spec.clone();
    invalid.targets[0].zone = Zone::Graveyard;
    assert!(crate::rules::validate_ability("JC004", &invalid).is_err());
    let mut invalid = spec.clone();
    invalid.targets[0].kind = EntityKind::Hidden;
    assert!(crate::rules::validate_ability("JC004", &invalid).is_err());
    let mut invalid = spec.clone();
    invalid.ops = vec![Op::IfTargetExhausted {
        slot: 1,
        exhausted: Box::new(Op::Move(EntityRef::Target(1), Destination::OwnerHand)),
        ready: Box::new(Op::Exhaust(EntityRef::Target(1))),
    }];
    assert!(crate::rules::validate_ability("JC004", &invalid).is_err());
}
