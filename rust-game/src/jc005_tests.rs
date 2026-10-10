//! JC005 uses the shared live target guard and Destroy operation.
//! Non-character hosts and post-declaration mutations are explicit primitive
//! fixtures, not additional playable attachment cards or natural gameplay.
use crate::{catalog, deck, model::*, rules::*};

fn game() -> Game {
    let mut g = Game::new(
        "jc005-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        4,
    )
    .unwrap();
    for s in 1..4 {
        g.join(format!("P{s}"), "watchers".into()).unwrap();
    }
    for p in &mut g.players {
        p.ready = true;
    }
    g.apply(0, Action::new("start")).unwrap();
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
    }
    for r in &mut g.regions {
        r.cards.clear();
    }
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    g
}
fn field(g: &mut Game, definition: &str, owner: usize, region: usize) -> String {
    let c = g.make_card(definition, owner);
    let id = c.id.clone();
    g.regions[region].cards.push(c);
    id
}
fn held(g: &mut Game, definition: &str, owner: usize) -> String {
    let c = g.make_card(definition, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn fund(g: &mut Game, actor: usize, definition: &str, count: usize) {
    for _ in 0..count {
        let c = g.make_card(definition, actor);
        g.players[actor].assets.push(c);
    }
}
fn attached(g: &mut Game, host: &str, owner: usize) -> String {
    let c = g.make_card("BQ022", owner);
    let id = c.id.clone();
    g.attachments.push(Attachment {
        card: c,
        host_id: host.into(),
    });
    id
}
fn pass_top(g: &mut Game) {
    let n = g.stack.len();
    assert!(n > 0);
    for _ in 0..32 {
        if g.stack.len() < n {
            return;
        }
        let s = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(s, Action::new("pass")).unwrap();
    }
    panic!("bounded response stack did not settle");
}
fn play(g: &mut Game, source: &str, target: &str) {
    g.apply(
        0,
        Action {
            card_id: Some(source.into()),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    )
    .unwrap();
}
fn source(g: &Game, id: &str) -> SourceSnapshot {
    g.source_snapshot(g.players[0].hand.iter().find(|c| c.id == id).unwrap(), None)
}
fn slot() -> TargetSlotSpec {
    crate::rules::definition("JC005").abilities[0].targets[0].clone()
}

#[test]
fn jc005_printed_metadata_is_fast_two_cost_one_yellow_mind_without_extra_cost_or_equipment_filter()
{
    let d = catalog::card("JC005");
    assert_eq!(
        (d.name.as_str(), d.kind.as_str(), d.cost),
        ("裂解术", "spell", 2)
    );
    assert_eq!(d.loyalty, ["黄色"]);
    assert_eq!(d.magic_icon, MagicIcon::Mind);
    assert_eq!(d.subtypes, ["法术", "空间"]);
    assert_eq!(d.defense, None);
    assert_eq!(d.permanent_icons, Icons::default());
    assert_eq!(d.temporary_icons, Icons::default());
    assert!(!d.unique && d.keywords.is_empty());
    let a = &crate::rules::definition("JC005").abilities[0];
    assert_eq!(a.timing, Timing::Fast);
    assert_eq!(a.response_policy, ResponsePolicy::Respondable);
    assert!(a.costs.is_empty() && a.modes.is_empty() && a.event.is_none());
    assert!(matches!(
        a.ops.as_slice(),
        [Op::Destroy(EntityRef::Target(0))]
    ));
    assert_eq!(a.targets[0].kind, EntityKind::Attachment);
    assert_eq!(a.targets[0].range, Range::Anywhere);
    assert_eq!(a.targets[0].relation, Relation::Any);
    assert!(a.targets[0].subtype.is_none() && a.targets[0].subtypes_any.is_empty());
    assert!(matches!(
        a.targets[0].attachment_host_condition,
        Some(AttachmentHostCondition::CharacterOrActorAssetDomain {
            magic: MagicIcon::Mind
        })
    ));
    let mut d = deck::preset("watchers").unwrap();
    d.cards = vec![
        catalog::DeckEntry {
            card_id: "JC005".into(),
            count: 3,
        },
        catalog::DeckEntry {
            card_id: "JC125".into(),
            count: 47,
        },
    ];
    assert!(deck::validate(d.clone()).is_ok());
    d.cards[0].count = 4;
    d.cards[1].count = 46;
    assert!(deck::validate(d).is_err());
    assert!(catalog::catalog().cards.iter().any(|c| c.id == "JC008"));
}

#[test]
fn jc005_host_predicate_requires_a_board_attachment_and_preserves_old_target_spec_decode() {
    let a = crate::rules::definition("JC005").abilities[0].clone();
    for (zone, kind, magic) in [
        (Zone::Graveyard, EntityKind::Attachment, MagicIcon::Mind),
        (Zone::Board, EntityKind::Character, MagicIcon::Mind),
        (Zone::Board, EntityKind::Attachment, MagicIcon::None),
    ] {
        let mut invalid = a.clone();
        invalid.targets[0].zone = zone;
        invalid.targets[0].kind = kind;
        invalid.targets[0].attachment_host_condition =
            Some(AttachmentHostCondition::CharacterOrActorAssetDomain { magic });
        assert!(crate::rules::validate_ability("fixture", &invalid).is_err());
    }
    let mut old = serde_json::to_value(slot()).unwrap();
    old.as_object_mut()
        .unwrap()
        .remove("attachment_host_condition");
    let decoded: TargetSlotSpec = serde_json::from_value(old).unwrap();
    assert!(decoded.attachment_host_condition.is_none());
}

#[test]
fn jc005_range_uses_only_actor_asset_domains_even_exhausted_and_is_not_a_payment() {
    let mut g = game();
    let src = held(&mut g, "JC005", 0);
    let snap = source(&g, &src);
    let character = field(&mut g, "LC22", 2, 4);
    let natural = attached(&mut g, &character, 3);
    // There is no admitted non-character-host attachment. Isolate the query only.
    let non_character = field(&mut g, "XQ03", 2, 1);
    let synthetic = attached(&mut g, &non_character, 2);
    assert!(g.valid_binding(0, &snap, &slot(), &natural));
    assert!(!g.valid_binding(0, &snap, &slot(), &synthetic));
    fund(&mut g, 1, "JC004", 1);
    fund(&mut g, 0, "JC085", 1);
    assert!(!g.valid_binding(0, &snap, &slot(), &synthetic));
    fund(&mut g, 0, "JC004", 1);
    g.players[0].assets.last_mut().unwrap().exhausted = true;
    assert!(g.valid_binding(0, &snap, &slot(), &synthetic));
    assert_eq!(g.resources(0), 1); // No target query spends or readies an asset.
    assert!(g.players[0].assets.last().unwrap().exhausted);
    let unattached = field(&mut g, "BQ022", 0, 0);
    assert!(!g.valid_binding(0, &snap, &slot(), &unattached));
    assert!(!g.valid_binding(0, &snap, &slot(), &character));
    g.attachments
        .iter_mut()
        .find(|a| a.card.id == synthetic)
        .unwrap()
        .card
        .face_down = true;
    assert!(!g.valid_binding(0, &snap, &slot(), &synthetic));
}

#[test]
fn jc005_destroy_goes_to_owner_graveyard_without_recycling_or_bottom_and_pays_only_two() {
    for mind in [false, true] {
        let mut g = game();
        let src = held(&mut g, "JC005", 0);
        fund(&mut g, 0, "JC002", 2);
        if mind {
            fund(&mut g, 0, "JC004", 1);
            g.players[0].assets.last_mut().unwrap().exhausted = true;
        }
        assert_eq!(g.actor_has_asset_domain(0, &MagicIcon::Mind, 1), mind);
        let host = field(&mut g, "LC22", 2, 4);
        g.board_mut(&host).unwrap().shield = 2;
        let id = attached(&mut g, &host, 3);
        let c = &mut g.attachments[0].card;
        c.controller = 2;
        c.exhausted = true;
        c.damage = 1;
        c.wounds = 1;
        let deck = g.players[3].deck.clone();
        play(&mut g, &src, &id);
        assert_eq!(g.resources(0), 0);
        assert_eq!(g.stack.len(), 1);
        g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_top(&mut g);
        assert!(g.attachments.is_empty());
        assert_eq!(g.board(&host).unwrap().1.shield, 2);
        let dead = g.players[3].graveyard.last().unwrap();
        assert_eq!(dead.definition, "BQ022");
        assert_ne!(dead.id, id);
        assert_eq!(
            (
                dead.owner,
                dead.controller,
                dead.face_down,
                dead.exhausted,
                dead.damage,
                dead.wounds,
                dead.shield
            ),
            (3, 3, false, false, 0, 0, 0)
        );
        assert!(g.players[3].hand.iter().all(|c| c.definition != "BQ022"));
        assert_eq!(
            serde_json::to_string(&g.players[3].deck).unwrap(),
            serde_json::to_string(&deck).unwrap()
        );
        assert!(g.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC005"));
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(src),
                    target_id: Some(id),
                    ..Action::new("play")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
    }
}

#[test]
fn jc005_synthetic_host_move_preserves_the_attachment_instance_and_anywhere_range() {
    let mut g = game();
    let src = held(&mut g, "JC005", 0);
    fund(&mut g, 0, "JC002", 2);
    let host = field(&mut g, "LC22", 2, 0);
    let id = attached(&mut g, &host, 3);
    play(&mut g, &src, &id);
    // No admitted fast action moves a host while JC005 waits. This is a
    // position-change checkpoint fixture, not a fabricated mobility action.
    let c = g.regions[0].cards.remove(0);
    g.regions[1].cards.push(c);
    assert_eq!(g.attachments[0].card.id, id);
    g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    pass_top(&mut g);
    assert!(g.attachments.is_empty() && g.board(&host).is_some());
    assert_eq!(g.board(&host).unwrap().0, 1);
    assert!(g.players[3]
        .graveyard
        .iter()
        .any(|c| c.definition == "BQ022"));
}

#[test]
fn jc005_natural_return_and_hide_responses_recycle_by_host_departure_then_cancel_original_target() {
    for response in ["JC006", "JC063"] {
        let mut g = game();
        let src = held(&mut g, "JC005", 0);
        fund(&mut g, 0, "JC002", 2);
        fund(&mut g, 2, "JC003", 2);
        fund(&mut g, 2, "JC063", 2);
        let host = field(&mut g, "JC004", 2, 0);
        let id = attached(&mut g, &host, 3);
        let reply = held(&mut g, response, 2);
        play(&mut g, &src, &id);
        g.apply(0, Action::new("pass")).unwrap();
        g.apply(1, Action::new("pass")).unwrap();
        g.apply(
            2,
            Action {
                card_id: Some(reply),
                target_id: Some(host.clone()),
                option: (response == "JC063").then(|| "hide".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_top(&mut g);
        assert!(g.attachments.is_empty());
        let returned = g.players[3]
            .hand
            .iter()
            .find(|c| c.definition == "BQ022")
            .unwrap();
        assert_ne!(returned.id, id);
        pass_top(&mut g);
        assert_eq!(g.resources(0), 0);
        assert!(g.players[3]
            .graveyard
            .iter()
            .all(|c| c.definition != "BQ022"));
        assert!(g.log.iter().any(|line| line.text.contains("效果取消")));
    }
}

#[test]
fn jc005_restored_guard_checks_current_instance_host_relation_type_and_attachment_visibility() {
    for change in [
        "host-change",
        "host-type",
        "target-reentered",
        "target-hidden",
    ] {
        let mut g = game();
        let src = held(&mut g, "JC005", 0);
        fund(&mut g, 0, "JC002", 2);
        let host = field(&mut g, "LC22", 2, 0);
        let second = field(&mut g, "LC22", 1, 4);
        let id = attached(&mut g, &host, 3);
        play(&mut g, &src, &id);
        let mut current = id.clone();
        match change {
            "host-change" => g.attachments[0].host_id = second,
            "host-type" => g.board_mut(&host).unwrap().definition = "XQ03".into(),
            "target-reentered" => {
                g.remove_dead(&id, RemovalCause::Destroy);
                current = attached(&mut g, &host, 3);
            }
            "target-hidden" => g.attachments[0].card.face_down = true,
            _ => unreachable!(),
        }
        g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_top(&mut g);
        assert_eq!(g.resources(0), 0);
        if change == "host-change" {
            assert!(g.board(&current).is_none());
        } else {
            assert!(g.log.iter().any(|line| line.text.contains("效果取消")));
        }
        if change == "target-reentered" || change == "target-hidden" {
            assert!(g.board(&current).is_some());
        }
        if change == "host-type" {
            assert!(g.attachments.is_empty());
            assert!(g.players[3].hand.iter().all(|c| c.definition != "BQ022"));
        }
    }
}

#[test]
fn jc005_asset_gain_loss_and_host_change_are_live_guard_predicate_fixtures_not_frozen_choices() {
    let mut g = game();
    let src = held(&mut g, "JC005", 0);
    let host = field(&mut g, "XQ03", 2, 0);
    let id = attached(&mut g, &host, 3);
    fund(&mut g, 0, "JC004", 1);
    let snap = source(&g, &src);
    let target = slot();
    let a = crate::rules::definition("JC005").abilities[0].clone();
    let frame = g.make_frame(
        0,
        snap.clone(),
        &a,
        vec![BoundTarget {
            region_instance: None,
            id: id.clone(),
            spec: target.clone(),
            public: g.public_target(0, &snap, &target, &id),
        }],
        vec![],
        None,
    );
    let mut restored: ResolutionFrame =
        serde_json::from_str(&serde_json::to_string(&frame).unwrap()).unwrap();
    let mind = g.players[0].assets.pop().unwrap();
    assert!(!g.accept_frame_guard(&mut restored));
    g.players[0].assets.push(mind);
    let mut refreshed = frame.clone();
    assert!(g.accept_frame_guard(&mut refreshed));
    g.players[0].assets.clear();
    g.board_mut(&host).unwrap().definition = "LC22".into();
    let mut refreshed = frame;
    assert!(g.accept_frame_guard(&mut refreshed));
    // Query/guard fixtures above deliberately do not drive an illegal BQ022 host.
    // Normal settlement still destroys a BQ022 whose host loses its allowed kind.
    g.board_mut(&host).unwrap().definition = "XQ03".into();
    fund(&mut g, 0, "JC004", 1);
    g.settle_deaths();
    assert!(g.attachments.is_empty());
    assert!(g.players[3]
        .graveyard
        .iter()
        .any(|c| c.definition == "BQ022"));
}
