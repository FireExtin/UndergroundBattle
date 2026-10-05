//! Explicit initial layouts; all control grants use real printed card declarations.
//! Natural joined-game cases live separately in the default WASM oracle.
use crate::{catalog, model::*, rules::*};

// Reviewer counterexample: printed eligibility and primitive lifetime contract
// are kept distinct. XQ12 remains non-Human among the admitted Death cards.
#[test]
fn control_simultaneous_nonhuman_death_card_still_invalid_for_jz27() {
    let death_cards = catalog::catalog()
        .cards
        .iter()
        .filter(|card| {
            definition(&card.id)
                .abilities
                .iter()
                .any(|a| a.event == Some(Event::Death))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        death_cards
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        vec!["XQ12", "XQ17", "JZ59"]
    );
    assert_eq!(death_cards[0].subtypes, vec!["吸血鬼", "奴仆"]);
    assert_eq!(death_cards[1].subtypes, vec!["人类"]);
    assert_eq!(death_cards[2].subtypes, vec!["人类", "宿主"]);
    let mut g = game();
    let target = field(&mut g, "XQ12", 2);
    fund(&mut g, 0, "XQ16", 6);
    let source = field(&mut g, "JZ27", 0);
    g.board_mut(&source).unwrap().face_down = true;
    g.apply(
        0,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    )
    .unwrap();
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert!(g.control_effects.is_empty());
    assert_eq!(controller(&g, &target), 2);
}

fn death_actor_and_snapshot(g: &Game, target: &str) -> (usize, usize) {
    let declarations = g
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::Declare { declaration }
                if declaration.ability.event == Some(Event::Death)
                    && declaration.source.card.id == target =>
            {
                Some(declaration)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 1);
    (
        declarations[0].actor,
        declarations[0].source.card.controller,
    )
}
fn ordered_death_pair(g: &mut Game, source: &str, target: &str, source_first: bool) {
    g.regions[0].cards.sort_by_key(|c| {
        if c.id == source {
            !source_first
        } else {
            source_first
        }
    });
    let expected = if source_first {
        vec![source, target]
    } else {
        vec![target, source]
    };
    assert_eq!(
        g.regions[0]
            .cards
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn control_simultaneous_legal_jc129_death_actor_stays_fixed_in_both_orders() {
    for source_first in [true, false] {
        let mut g = game();
        let target = field(&mut g, "XQ12", 2);
        let source = field(&mut g, "JZ27", 0);
        fund(&mut g, 0, "JC104", 3);
        play(&mut g, 0, "JC129", &target);
        ordered_death_pair(&mut g, &source, &target, source_first);
        mirror(&mut g);
        g.damage(std::collections::BTreeMap::from([
            (source.clone(), 3),
            (target.clone(), 1),
        ]))
        .unwrap();
        assert_eq!(death_actor_and_snapshot(&g, &target), (0, 0));
        assert!(g.board(&source).is_none() && g.board(&target).is_none());
    }
}

#[test]
fn control_simultaneous_single_source_death_restores_live_legal_human_immediately() {
    let mut g = game();
    let target = field(&mut g, "JC125", 2);
    let source = reveal(&mut g, 0, &target);
    assert_eq!(controller(&g, &target), 0);
    g.damage(std::collections::BTreeMap::from([(source.clone(), 3)]))
        .unwrap();
    assert!(g.board(&source).is_none());
    assert_eq!(controller(&g, &target), 2);
    assert!(g.control_effects.is_empty());
    assert_eq!(
        g.current_subtypes(g.board(&target).unwrap().1),
        vec!["人类"]
    );
}

#[test]
fn control_simultaneous_real_jz27_human_pair_has_no_death_trigger_in_either_order() {
    for source_first in [true, false] {
        let mut g = game();
        let target = field(&mut g, "JC125", 2);
        let source = reveal(&mut g, 0, &target);
        assert_eq!(controller(&g, &target), 0);
        ordered_death_pair(&mut g, &source, &target, source_first);
        mirror(&mut g);
        g.damage(std::collections::BTreeMap::from([
            (source.clone(), 3),
            (target.clone(), 1),
        ]))
        .unwrap();
        assert!(g.board(&source).is_none() && g.board(&target).is_none());
        assert!(g.control_effects.is_empty() && g.control_baselines.is_empty());
        assert!(g.effects.iter().all(|e| !matches!(e, Effect::Declare { declaration } if declaration.ability.event == Some(Event::Death))));
        assert!(g.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "JZ27" && c.owner == 0 && c.controller == 0));
        assert!(g.players[2]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC125" && c.owner == 2 && c.controller == 2));
    }
}

fn game() -> Game {
    let mut g = Game::new(
        "control-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        7,
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
fn field(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn hand(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn fund(g: &mut Game, seat: usize, def: &str, n: usize) {
    for _ in 0..n {
        let c = g.make_card(def, seat);
        g.players[seat].assets.push(c);
    }
}
fn pass_top(g: &mut Game) {
    let n = g.stack.len();
    assert!(n > 0);
    for _ in 0..32 {
        if g.stack.len() < n {
            return;
        }
        let seat = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(seat, Action::new("pass")).unwrap_or_else(|e| {
            panic!(
                "pass actor={seat} error={e} priority={} window={:?} stack={} pending={:?}",
                g.priority_team,
                g.window,
                g.stack.len(),
                g.pending.as_ref().map(|p| &p.choice.kind)
            )
        });
    }
    panic!("bounded response stack");
}
fn play(g: &mut Game, seat: usize, def: &str, target: &str) {
    let source = hand(g, def, seat);
    g.apply(
        seat,
        Action {
            card_id: Some(source),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    )
    .unwrap();
    pass_top(g);
}
fn mirror(g: &mut Game) {
    let bytes = serde_json::to_string(g).unwrap();
    *g = Game::from_persisted(&bytes).unwrap();
    assert_eq!(serde_json::to_string(g).unwrap(), bytes);
}
fn controller(g: &Game, id: &str) -> usize {
    g.board(id).unwrap().1.controller
}
fn top(g: &Game, seat: usize) {
    for s in 0..4 {
        assert_eq!(
            g.view(s).private_deck_top.as_ref().map(|c| &c.instance_id),
            if s == seat {
                g.players[s].deck.first().map(|c| &c.id)
            } else {
                None
            }
        );
    }
}
fn embrace(g: &mut Game, seat: usize, target: &str) -> String {
    fund(g, seat, "XQ16", 5);
    g.begin_window(Window::Action(g.team(seat)));
    play(g, seat, "JC036", target);
    g.attachments.last().unwrap().card.id.clone()
}
fn hide(g: &mut Game, seat: usize, target: &str) {
    fund(g, seat, "XQ16", 2);
    g.begin_window(Window::Action(g.team(seat)));
    let lawyer = hand(g, "XQ16", seat);
    g.apply(
        seat,
        Action {
            card_id: Some(lawyer),
            region: Some(0),
            ..Action::new("deploy")
        },
    )
    .unwrap();
    pass_top(g);
    let p = g.pending.as_ref().unwrap();
    g.apply(
        p.seat,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec![target.into()]),
            ..Action::new("choose")
        },
    )
    .unwrap();
    pass_top(g);
}
fn reveal(g: &mut Game, seat: usize, target: &str) -> String {
    fund(g, seat, "XQ16", 6);
    let source = field(g, "JZ27", seat);
    g.board_mut(&source).unwrap().face_down = true;
    g.begin_window(Window::Action(g.team(seat)));
    g.apply(
        seat,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    )
    .unwrap();
    pass_top(g);
    let id = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JZ27" && c.controller == seat)
        .unwrap()
        .id
        .clone();
    let p = g.pending.as_ref().unwrap();
    g.apply(
        p.seat,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec![target.into()]),
            ..Action::new("choose")
        },
    )
    .unwrap();
    pass_top(g);
    id
}
#[test]
fn control_temporary_real_spell_preserves_bonus_identity_and_four_seat_top_permission() {
    let mut g = game();
    let t = field(&mut g, "LC01", 2);
    fund(&mut g, 0, "JC002", 2);
    play(&mut g, 0, "JC008", &t);
    let old = g.board(&t).unwrap().1.clone();
    let defense = g.defense(&old, 0);
    let icons = g.current_icons(&old, 0);
    fund(&mut g, 0, "JC104", 3);
    play(&mut g, 0, "JC129", &t);
    let now = g.board(&t).unwrap().1;
    assert_eq!(
        (now.owner, now.controller, now.id.as_str()),
        (2, 0, t.as_str())
    );
    assert_eq!(
        (g.defense(now, 0), g.current_icons(now, 0).combat),
        (defense, icons.combat)
    );
    top(&g, 0);
    // Damage, attribute expiry and controller restoration form one cleanup batch.
    g.board_mut(&t).unwrap().damage = 4;
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC129"));
    mirror(&mut g);
    g.effect(Effect::FinishCleanup).unwrap();
    assert_eq!(controller(&g, &t), 2);
    assert!(g.control_effects.is_empty());
    assert!(g.control_baselines.is_empty());
    top(&g, 2);
    assert_eq!(g.board(&t).unwrap().1.damage, 0);
}
#[test]
fn control_spell_rejects_friendly_hidden_without_loyalty_and_paid_response_changed_controller() {
    let mut g = game();
    let friend = field(&mut g, "JC125", 1);
    let hidden = field(&mut g, "LC01", 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    let s = hand(&mut g, "JC129", 0);
    fund(&mut g, 0, "JC125", 3);
    for target in [&friend, &hidden] {
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(s.clone()),
                    target_id: Some(target.clone()),
                    ..Action::new("play")
                }
            )
            .is_err());
    }
    let t = field(&mut g, "JC125", 2);
    assert!(g
        .apply(
            0,
            Action {
                card_id: Some(s.clone()),
                target_id: Some(t.clone()),
                ..Action::new("play")
            }
        )
        .is_err());
    fund(&mut g, 0, "JC104", 1);
    g.apply(
        0,
        Action {
            card_id: Some(s),
            target_id: Some(t.clone()),
            ..Action::new("play")
        },
    )
    .unwrap();
    // Explicit response checkpoint, not another implemented card family.
    g.board_mut(&t).unwrap().controller = 1;
    pass_top(&mut g);
    assert_eq!(controller(&g, &t), 1);
    assert!(g.control_effects.is_empty());
}
#[test]
fn control_embrace_printed_eligibility_current_types_and_attachment_ownership() {
    let mut g = game();
    let t = field(&mut g, "LC01", 2);
    let a = embrace(&mut g, 0, &t);
    assert_eq!(controller(&g, &t), 0);
    assert!(g.attachment_host_valid(&g.attachments[0]));
    let types = g.current_subtypes(g.board(&t).unwrap().1);
    assert_eq!(types, vec!["法师", "吸血鬼"]);
    assert_eq!(catalog::card("LC01").subtypes, vec!["人类", "法师"]);
    let view = g.view(2);
    assert_eq!(
        view.regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == t)
            .unwrap()
            .current_subtypes
            .as_ref(),
        Some(&types)
    );
    mirror(&mut g);
    assert_eq!(
        (
            g.attachments[0].card.owner,
            g.attachments[0].card.controller
        ),
        (0, 0)
    );
    g.attachments[0].card.controller = 1;
    g.settle_deaths();
    assert_eq!(controller(&g, &t), 1);
    assert_eq!(g.attachments[0].card.owner, 0);
    g.remove_dead(&a, RemovalCause::Destroy);
    g.settle_deaths();
    assert_eq!(controller(&g, &t), 2);
    assert_eq!(
        g.current_subtypes(g.board(&t).unwrap().1),
        vec!["人类", "法师"]
    );
}
#[test]
fn control_last_active_layer_restores_lower_then_original_baseline() {
    let mut g = game();
    let t = field(&mut g, "LC01", 2);
    let a = embrace(&mut g, 0, &t);
    fund(&mut g, 2, "JC104", 3);
    g.begin_window(Window::Action(1));
    play(&mut g, 2, "JC129", &t);
    assert_eq!(controller(&g, &t), 2);
    assert!(g
        .current_subtypes(g.board(&t).unwrap().1)
        .contains(&"吸血鬼".into()));
    mirror(&mut g);
    g.effect(Effect::FinishCleanup).unwrap();
    assert_eq!(controller(&g, &t), 0);
    top(&g, 0);
    g.remove_dead(&a, RemovalCause::Destroy);
    g.settle_deaths();
    assert_eq!(controller(&g, &t), 2);
    top(&g, 2);
}
#[test]
fn control_lower_source_removed_under_upper_layer_never_resurrects() {
    let mut g = game();
    let t = field(&mut g, "LC01", 2);
    let a = embrace(&mut g, 0, &t);
    fund(&mut g, 2, "JC104", 3);
    g.begin_window(Window::Action(1));
    play(&mut g, 2, "JC129", &t);
    g.remove_dead(&a, RemovalCause::Destroy);
    g.settle_deaths();
    assert_eq!(controller(&g, &t), 2);
    assert!(!g
        .current_subtypes(g.board(&t).unwrap().1)
        .contains(&"吸血鬼".into()));
    mirror(&mut g);
    g.effect(Effect::FinishCleanup).unwrap();
    assert_eq!(controller(&g, &t), 2);
    assert!(g.control_effects.is_empty());
}
#[test]
fn control_reveal_current_human_local_target_fixed_recipient_source_departure() {
    let mut g = game();
    let t = field(&mut g, "LC01", 2);
    let source = reveal(&mut g, 0, &t);
    assert_eq!(controller(&g, &t), 0);
    assert_eq!(
        g.current_subtypes(g.board(&t).unwrap().1),
        vec!["人类", "法师", "奴仆"]
    );
    top(&g, 0);
    // Real opponent spell transfers the source, never the historical beneficiary.
    fund(&mut g, 2, "JC104", 3);
    g.begin_window(Window::Action(1));
    play(&mut g, 2, "JC129", &source);
    assert_eq!(controller(&g, &source), 2);
    assert_eq!(controller(&g, &t), 0);
    mirror(&mut g);
    g.return_hand(&source);
    g.settle_deaths();
    assert_eq!(controller(&g, &t), 2);
    assert_eq!(
        g.current_subtypes(g.board(&t).unwrap().1),
        vec!["人类", "法师"]
    );
    top(&g, 2);
}
#[test]
fn control_target_hide_ends_all_old_target_effects_and_no_reveal_inheritance() {
    let mut g = game();
    let t = field(&mut g, "LC01", 2);
    let a = embrace(&mut g, 0, &t);
    let lawyer = hand(&mut g, "XQ16", 0);
    fund(&mut g, 0, "XQ16", 2);
    g.begin_window(Window::Action(0));
    g.apply(
        0,
        Action {
            card_id: Some(lawyer),
            region: Some(0),
            ..Action::new("deploy")
        },
    )
    .unwrap();
    pass_top(&mut g);
    let p = g.pending.as_ref().unwrap();
    g.apply(
        p.seat,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec![t.clone()]),
            ..Action::new("choose")
        },
    )
    .unwrap();
    pass_top(&mut g);
    assert!(g.board(&t).is_none());
    assert!(g.attachments.iter().all(|x| x.card.id != a));
    assert!(g.control_effects.is_empty());
    let c = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "LC01")
        .unwrap();
    assert_eq!((c.owner, c.controller, c.face_down), (2, 2, true));
    let hidden = c.id.clone();
    for s in 0..4 {
        let v = g.view(s);
        let c = v.regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == hidden)
            .unwrap();
        assert_eq!(c.card_id.is_some(), s == 2);
        assert!(c.current_subtypes.is_none());
    }
    fund(&mut g, 2, "JC002", 5);
    g.begin_window(Window::Action(1));
    g.apply(
        2,
        Action {
            card_id: Some(hidden.clone()),
            ..Action::new("reveal")
        },
    )
    .unwrap();
    pass_top(&mut g);
    let c = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "LC01")
        .unwrap();
    assert_ne!(c.id, hidden);
    assert_eq!(c.controller, 2);
    assert_eq!(g.current_subtypes(c), vec!["人类", "法师"]);
}
#[test]
fn control_baseline_can_differ_from_owner_without_rewriting_that_baseline() {
    let mut g = game();
    let t = field(&mut g, "LC01", 3);
    g.board_mut(&t).unwrap().controller = 2; // explicit lawful baseline primitive
    fund(&mut g, 0, "JC104", 3);
    play(&mut g, 0, "JC129", &t);
    assert_eq!(controller(&g, &t), 0);
    g.effect(Effect::FinishCleanup).unwrap();
    assert_eq!(controller(&g, &t), 2);
    assert_eq!(g.board(&t).unwrap().1.owner, 3);
}
#[test]
fn control_bound_program_validation_rejects_noncharacter_or_unbound_target() {
    let mut spec = definition("JC129").abilities[0].clone();
    spec.targets.clear();
    assert!(validate_ability("invalid", &spec).is_err());
    let mut spec = definition("JC129").abilities[0].clone();
    spec.targets[0].kind = EntityKind::Hidden;
    assert!(validate_ability("invalid", &spec).is_err());
}
#[test]
fn control_three_real_layers_and_distinct_expiry_restore_each_valid_lower_layer() {
    let mut g = game();
    let t = field(&mut g, "LC01", 2);
    let source = reveal(&mut g, 0, &t);
    let a = embrace(&mut g, 2, &t);
    assert_eq!(controller(&g, &t), 2);
    assert_eq!(
        g.current_subtypes(g.board(&t).unwrap().1),
        vec!["法师", "奴仆", "吸血鬼"]
    );
    fund(&mut g, 0, "JC104", 3);
    g.begin_window(Window::Action(0));
    play(&mut g, 0, "JC129", &t);
    assert_eq!(controller(&g, &t), 0);
    mirror(&mut g);
    g.effect(Effect::FinishCleanup).unwrap();
    assert_eq!(controller(&g, &t), 2);
    g.remove_dead(&a, RemovalCause::Destroy);
    g.settle_deaths();
    assert_eq!(controller(&g, &t), 0);
    assert!(g
        .current_subtypes(g.board(&t).unwrap().1)
        .contains(&"奴仆".into()));
    g.return_hand(&source);
    g.settle_deaths();
    assert_eq!(controller(&g, &t), 2);
    assert_eq!(
        g.current_subtypes(g.board(&t).unwrap().1),
        vec!["人类", "法师"]
    );
}
#[test]
fn control_source_hide_ends_historical_effect_and_new_instance_is_clean() {
    let mut g = game();
    let t = field(&mut g, "LC01", 2);
    let source = reveal(&mut g, 0, &t);
    hide(&mut g, 0, &source);
    assert!(g.board(&source).is_none());
    assert_eq!(controller(&g, &t), 2);
    assert_eq!(
        g.current_subtypes(g.board(&t).unwrap().1),
        vec!["人类", "法师"]
    );
    top(&g, 2);
    mirror(&mut g);
    let hidden = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JZ27")
        .unwrap();
    assert!(hidden.face_down);
    assert_ne!(hidden.id, source);
    assert!(g.control_effects.is_empty());
}
#[test]
fn control_temporary_target_hide_restores_baseline_before_new_instance() {
    let mut g = game();
    let t = field(&mut g, "LC01", 2);
    fund(&mut g, 0, "JC104", 3);
    play(&mut g, 0, "JC129", &t);
    hide(&mut g, 0, &t);
    let c = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "LC01")
        .unwrap();
    assert_eq!((c.owner, c.controller, c.face_down), (2, 2, true));
    assert_ne!(c.id, t);
    assert!(g.control_effects.is_empty());
    assert!(g.control_baselines.is_empty());
}
#[test]
fn control_jz27_faceup_deployment_does_not_trigger_and_reveal_uses_current_human() {
    let mut g = game();
    let t = field(&mut g, "LC01", 2);
    fund(&mut g, 0, "XQ16", 6);
    let source = hand(&mut g, "JZ27", 0);
    g.apply(
        0,
        Action {
            card_id: Some(source),
            region: Some(0),
            ..Action::new("deploy")
        },
    )
    .unwrap();
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert!(g.control_effects.is_empty());
    assert_eq!(controller(&g, &t), 2);
    let mut g = game();
    let t = field(&mut g, "LC01", 2);
    embrace(&mut g, 0, &t);
    let human = field(&mut g, "JC125", 1);
    fund(&mut g, 0, "XQ16", 6);
    let source = field(&mut g, "JZ27", 0);
    g.board_mut(&source).unwrap().face_down = true;
    g.apply(
        0,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    )
    .unwrap();
    pass_top(&mut g);
    let p = g.pending.as_ref().unwrap();
    assert!(!p.choice.options.iter().any(|o| o.id == t));
    assert!(p.choice.options.iter().any(|o| o.id == human));
}
#[test]
fn control_source_return_in_real_fast_response_does_not_leave_historical_grant() {
    let mut g = game();
    let t = field(&mut g, "LC01", 2);
    fund(&mut g, 0, "XQ16", 6);
    let source = field(&mut g, "JZ27", 0);
    g.board_mut(&source).unwrap().face_down = true;
    g.apply(
        0,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    )
    .unwrap();
    pass_top(&mut g);
    let source = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JZ27")
        .unwrap()
        .id
        .clone();
    let p = g.pending.as_ref().unwrap();
    g.apply(
        p.seat,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec![t.clone()]),
            ..Action::new("choose")
        },
    )
    .unwrap();
    for seat in [0, 1] {
        if g.legal_actions(seat)
            .iter()
            .any(|a| a.action.kind == "pass")
        {
            g.apply(seat, Action::new("pass")).unwrap();
        }
    }
    fund(&mut g, 2, "JC002", 2);
    play(&mut g, 2, "JC006", &source);
    pass_top(&mut g);
    assert_eq!(controller(&g, &t), 2);
    assert!(g.control_effects.is_empty());
    assert_eq!(
        g.current_subtypes(g.board(&t).unwrap().1),
        vec!["人类", "法师"]
    );
}
#[test]
fn control_change_queues_existing_unique_conflict_for_new_controller() {
    let mut g = game();
    let existing = field(&mut g, "LC01", 0);
    let taken = field(&mut g, "LC01", 2);
    fund(&mut g, 0, "JC104", 3);
    play(&mut g, 0, "JC129", &taken);
    let p = g.pending.as_ref().unwrap();
    assert_eq!(p.seat, 0);
    assert!(matches!(p.resolution, ChoiceResolution::Unique));
    assert!(p.choice.options.iter().any(|o| o.id == existing));
    assert!(p.choice.options.iter().any(|o| o.id == taken));
    g.apply(
        0,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec![existing.clone()]),
            ..Action::new("choose")
        },
    )
    .unwrap();
    assert!(g.board(&existing).is_none());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "LC01"));
    assert_eq!(controller(&g, &taken), 0);
    g.effect(Effect::FinishCleanup).unwrap();
    assert_eq!(controller(&g, &taken), 2);
}
#[test]
fn control_real_mobility_keeps_instance_and_attribute_modifier() {
    let mut g = game();
    let target = field(&mut g, "JC014", 2);
    fund(&mut g, 0, "JC002", 2);
    play(&mut g, 0, "JC008", &target);
    fund(&mut g, 0, "JC104", 3);
    play(&mut g, 0, "JC129", &target);
    let card = g.board(&target).unwrap().1.clone();
    let icons = g.current_icons(&card, 0);
    let defense = g.defense(&card, 0);
    g.begin_window(Window::Mobility);
    let source = g.source_snapshot(&card, Some(0));
    g.emit_event(0, source, Event::ConfrontationStart);
    g.drive().unwrap();
    let p = g.pending.as_ref().unwrap();
    assert_eq!(p.seat, 0);
    g.apply(
        0,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec!["region:1".into()]),
            ..Action::new("choose")
        },
    )
    .unwrap();
    pass_top(&mut g);
    let (region, card) = g.board(&target).unwrap();
    assert_eq!(region, 1);
    assert_eq!((card.owner, card.controller), (2, 0));
    assert_eq!(g.current_icons(card, 1).combat, icons.combat);
    assert_eq!(g.defense(card, 1), defense);
    assert!(g
        .turn_attribute_modifiers
        .iter()
        .any(|m| m.target_instance == target));
    assert!(g
        .control_effects
        .iter()
        .any(|e| e.target_instance == target));
    mirror(&mut g);
}

#[test]
fn hand_deck_real_jz27_offering_simultaneous_death_keeps_last_controller_both_orders() {
    for source_first in [true, false] {
        let mut g = game();
        let target = field(&mut g, "XQ17", 2);
        let source = reveal(&mut g, 0, &target);
        assert_eq!(controller(&g, &target), 0);
        ordered_death_pair(&mut g, &source, &target, source_first);
        mirror(&mut g);
        g.damage(std::collections::BTreeMap::from([
            (source.clone(), 3),
            (target.clone(), 1),
        ]))
        .unwrap();
        assert_eq!(death_actor_and_snapshot(&g, &target), (0, 0));
        assert!(g.board(&source).is_none() && g.board(&target).is_none());
        assert!(g.control_effects.is_empty() && g.control_baselines.is_empty());
        assert!(g.players[2]
            .graveyard
            .iter()
            .any(|c| c.definition == "XQ17" && c.id != target && c.controller == 2));
        mirror(&mut g);
    }
}
