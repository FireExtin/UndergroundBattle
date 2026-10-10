//! Explicit initial layouts/funding, followed by actual paid card commands,
//! response stacks and windows. Every checkpoint restores exact state/views.
use crate::{catalog, model::*, rules::*};

fn game() -> Game {
    let mut g = Game::new(
        "renown-unit".into(),
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
fn field(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn held(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn fund(g: &mut Game, seat: usize, def: &str, count: usize) {
    for _ in 0..count {
        let c = g.make_card(def, seat);
        g.players[seat].assets.push(c);
    }
}
fn mirror(g: &mut Game) {
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
fn choose(g: &mut Game, selected: Vec<String>) {
    mirror(g);
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
    mirror(g);
}
fn pass(g: &mut Game) {
    mirror(g);
    let seat = (0..4)
        .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
        .unwrap();
    g.apply(seat, Action::new("pass")).unwrap();
    mirror(g);
}
fn pass_top(g: &mut Game) {
    let n = g.stack.len();
    assert!(n > 0);
    for _ in 0..32 {
        if g.stack.len() < n {
            return;
        }
        pass(g);
    }
    panic!("bounded renown stack");
}
fn give_priority(g: &mut Game, seat: usize) {
    while g.priority_team != g.team(seat) {
        pass(g);
    }
}
fn decline_region_win_trigger(g: &mut Game) {
    if let Some(p) = &g.pending {
        assert!(
            matches!(&p.resolution, ChoiceResolution::Declare { declaration, .. } if declaration.ability.event == Some(Event::RegionWon))
        );
        choose(g, vec![]);
    }
}
fn cast(g: &mut Game, def: &str, seat: usize, target: &str) {
    let source = held(g, def, seat);
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
fn end_contests(g: &mut Game) {
    g.begin_window(Window::Before(0, 2));
    for _ in 0..8 {
        if g.window != Some(Window::Before(0, 2)) {
            return;
        }
        pass(g);
    }
    panic!("influence window");
}
fn accept(g: &mut Game) {
    choose(g, vec!["accept".into()]);
    assert_eq!(
        g.stack.last().unwrap().frame.as_ref().unwrap().ability_key,
        "renown"
    );
}
fn grants(g: &mut Game, def: &str, seat: usize) -> String {
    let target = field(g, def, seat);
    fund(g, 0, "JC075", 4);
    cast(g, "JC074", 0, &target);
    target
}

#[test]
fn renown_original_fields_and_bounded_program_are_complete() {
    let a = catalog::card("JC070");
    let b = catalog::card("JC076");
    let c = catalog::card("JC074");
    assert_eq!((a.cost, a.defense, a.magic.as_str()), (1, Some(1), "神圣"));
    assert_eq!(a.loyalty, ["白色", "白色"]);
    assert_eq!((b.cost, b.defense, b.magic.as_str()), (3, Some(1), "神圣"));
    assert_eq!(b.loyalty, ["白色"]);
    assert_eq!(
        (c.cost, c.kind.as_str(), c.magic.as_str()),
        (2, "spell", "星辰")
    );
    assert_eq!(c.loyalty, ["白色"]);
    assert_eq!(c.subtypes, ["法术", "预言"]);
    for d in [a, b] {
        assert_eq!(d.subtypes, ["人类", "僧侣"]);
        assert_eq!(d.permanent_icons, Icons::default());
        assert!(!d.unique);
    }
    assert_eq!(a.temporary_icons.investigation, 1);
    assert_eq!(b.temporary_icons.investigation, 2);
    assert!(
        a.rule_traits.public
            && a.rule_traits.renown
            && b.rule_traits.renown
            && !b.rule_traits.public
    );
    let draw = &definition("JC076").abilities[0];
    assert_eq!(draw.event, Some(Event::Reveal));
    let grant = &definition("JC074").abilities[0];
    assert!(matches!(grant.timing, Timing::Fast));
    assert_eq!(grant.targets[0].kind, EntityKind::Character);
    assert_eq!(grant.targets[0].relation, Relation::Any);
    assert_eq!(grant.targets[0].range, Range::Anywhere);
    assert!(matches!(
        grant.ops[0],
        Op::ModifyTargetUntilTurnEnd {
            grants_renown: true,
            defense_bonus: 0,
            ordinary_icons: Icons {
                investigation: 2,
                combat: 0,
                influence: 0
            },
            ..
        }
    ));
    assert_eq!(catalog::catalog().cards.len(), 128);
}

#[test]
fn renown_jc070_loyalty_two_and_public_rejections_are_atomic() {
    for kind in ["loyalty", "conceal"] {
        let mut g = game();
        let source = held(&mut g, "JC070", 0);
        fund(&mut g, 0, "JC075", if kind == "loyalty" { 1 } else { 2 });
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(source),
                    region: Some(0),
                    ..Action::new(if kind == "loyalty" {
                        "deploy"
                    } else {
                        "conceal"
                    })
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
    }
    let mut g = game();
    let source = held(&mut g, "JC070", 0);
    fund(&mut g, 0, "JC075", 2);
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
    assert_eq!(g.resources(0), 1);
    assert!(g.pending.is_none());
    assert!(g.has_renown(&g.regions[0].cards[0]));
}

#[test]
fn renown_jc076_paid_conceal_reveal_draw_is_optional_and_independent() {
    let mut g = game();
    fund(&mut g, 0, "JC075", 4);
    fund(&mut g, 2, "JC002", 2);
    let source = held(&mut g, "JC076", 0);
    g.apply(
        0,
        Action {
            card_id: Some(source),
            region: Some(0),
            ..Action::new("conceal")
        },
    )
    .unwrap();
    let hidden = g.regions[0].cards[0].id.clone();
    assert!(!g.has_renown(&g.regions[0].cards[0]));
    for s in 1..4 {
        assert!(g.view(s).regions[0].characters[0].card_id.is_none());
    }
    g.apply(
        0,
        Action {
            card_id: Some(hidden.clone()),
            ..Action::new("reveal")
        },
    )
    .unwrap();
    pass_top(&mut g);
    assert!(g.board(&hidden).is_none());
    assert_eq!(g.resources(0), 0);
    let p = g.pending.clone().unwrap();
    assert_eq!(p.choice.kind, "trigger");
    assert_eq!(p.seat, 0);
    let actor = match p.resolution {
        ChoiceResolution::Declare { declaration, .. } => {
            assert_eq!(declaration.ability.event, Some(Event::Reveal));
            declaration.actor
        }
        _ => panic!(),
    };
    choose(&mut g, vec!["accept".into()]);
    let face_up = g.regions[0].cards[0].id.clone();
    let n = g.players[actor].hand.len();
    let reply = held(&mut g, "JC006", 2);
    give_priority(&mut g, 2);
    g.apply(
        2,
        Action {
            card_id: Some(reply),
            target_id: Some(face_up.clone()),
            ..Action::new("play")
        },
    )
    .unwrap();
    pass_top(&mut g);
    assert!(g.board(&face_up).is_none());
    assert_eq!(g.players[actor].hand.len(), n + 1);
    pass_top(&mut g);
    assert_eq!(g.players[actor].hand.len(), n + 2);
}

#[test]
fn renown_jc076_face_up_deployment_does_not_draw_or_trigger_reveal() {
    let mut g = game();
    fund(&mut g, 0, "JC075", 3);
    let source = held(&mut g, "JC076", 0);
    let deck = g.players[0].deck.len();
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
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.players[0].deck.len(), deck);
    assert!(g.players[0].hand.is_empty());
}

#[test]
fn renown_granted_icons_are_ordinary_stack_but_keyword_is_one_character() {
    let mut g = game();
    let target = grants(&mut g, "JC070", 2);
    field(&mut g, "JC070", 0);
    cast(&mut g, "JC074", 0, &target);
    assert_eq!(g.icons(g.board(&target).unwrap().1, 0).investigation, 4); // Enemy lacks initiative.
    g.first_team = 1;
    assert_eq!(g.icons(g.board(&target).unwrap().1, 0).investigation, 5);
    g.first_team = 0;
    end_contests(&mut g);
    assert!(g.pending.is_none());
    assert_eq!(g.regions[0].influence, [0, 0]);
    for s in 0..4 {
        assert_eq!(
            g.view(s).regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == target)
                .unwrap()
                .current_renown,
            Some(true)
        );
    }
}

#[test]
fn renown_jc074_rejects_unpaid_hidden_attachment_barrier_and_wrong_loyalty_atomically() {
    for kind in ["unpaid", "hidden", "attachment", "barrier", "loyalty"] {
        let mut g = game();
        let target = field(
            &mut g,
            if kind == "barrier" {
                "JZ08"
            } else if kind == "attachment" {
                "XQ03"
            } else {
                "JC125"
            },
            2,
        );
        if kind == "hidden" {
            g.board_mut(&target).unwrap().face_down = true;
        }
        fund(
            &mut g,
            0,
            if kind == "loyalty" { "JC002" } else { "JC075" },
            if kind == "unpaid" { 1 } else { 2 },
        );
        let source = held(&mut g, "JC074", 0);
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(source),
                    target_id: Some(target),
                    ..Action::new("play")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
    }
}

#[test]
fn renown_jc074_response_source_target_leaves_cancels_without_refunding() {
    let mut g = game();
    fund(&mut g, 0, "JC075", 2);
    fund(&mut g, 2, "JC002", 2);
    let target = field(&mut g, "JC070", 2);
    let source = held(&mut g, "JC074", 0);
    g.apply(
        0,
        Action {
            card_id: Some(source),
            target_id: Some(target.clone()),
            ..Action::new("play")
        },
    )
    .unwrap();
    let reply = held(&mut g, "JC006", 2);
    give_priority(&mut g, 2);
    g.apply(
        2,
        Action {
            card_id: Some(reply),
            target_id: Some(target.clone()),
            ..Action::new("play")
        },
    )
    .unwrap();
    pass_top(&mut g);
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    assert!(g.turn_attribute_modifiers.is_empty());
    assert_eq!(g.resources(0), 0);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC074"));
}

#[test]
fn renown_project_team_totals_merge_two_seats_and_decline_is_final() {
    for decline in [false, true] {
        let mut g = game();
        field(&mut g, "JC070", 0);
        field(&mut g, "JC076", 1);
        field(&mut g, "JC070", 2);
        end_contests(&mut g);
        let p = g.pending.clone().unwrap();
        assert_eq!(p.seat, 0);
        assert_eq!(p.choice.kind, "trigger");
        assert!(g.view(1).pending_choice.is_none());
        assert_eq!(g.view(1).waiting_choice.unwrap().player_id, "p0");
        if decline {
            choose(&mut g, vec![]);
        } else {
            accept(&mut g);
            pass_top(&mut g);
        }
        assert!(g.pending.is_none() && g.stack.is_empty());
        assert_eq!(g.regions[0].influence, [u32::from(!decline), 0]);
        for _ in 0..4 {
            pass(&mut g);
        }
        assert_eq!(g.window, Some(Window::Before(1, 0)));
        assert_eq!(g.regions[0].influence, [u32::from(!decline), 0]);
    }
}

#[test]
fn renown_declares_by_actual_controller_seat_and_counts_only_this_region() {
    let mut g = game();
    let source = field(&mut g, "JC070", 2);
    for _ in 0..3 {
        let c = g.make_card("JC076", 3);
        g.regions[2].cards.push(c);
    }
    fund(&mut g, 1, "JC104", 3);
    cast(&mut g, "JC129", 1, &source);
    assert_eq!(g.board(&source).unwrap().1.owner, 2);
    assert_eq!(g.board(&source).unwrap().1.controller, 1);
    end_contests(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 1);
    let p = g.pending.clone().unwrap();
    let before = serde_json::to_string(&g).unwrap();
    assert!(g
        .apply(
            0,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec!["accept".into()]),
                ..Action::new("choose")
            }
        )
        .is_err());
    assert_eq!(serde_json::to_string(&g).unwrap(), before);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[0].influence, [1, 0]);
    assert_eq!(g.regions[2].influence, [0, 0]);
}

#[test]
fn renown_counts_exclude_hidden_exhausted_and_ties_without_participation_history() {
    for kind in ["hidden", "exhausted", "tie", "none", "no-history"] {
        let mut g = game();
        if kind != "none" {
            let id = field(&mut g, "JC070", 2);
            if kind == "hidden" {
                g.board_mut(&id).unwrap().face_down = true;
            }
            if kind == "exhausted" {
                g.board_mut(&id).unwrap().exhausted = true;
            }
            if kind == "tie" {
                field(&mut g, "JC076", 0);
            }
        }
        end_contests(&mut g);
        if kind == "no-history" {
            // Rear team has zero icons in all three contests.
            assert_eq!(g.pending.as_ref().unwrap().seat, 2);
            accept(&mut g);
            pass_top(&mut g);
            assert_eq!(g.regions[0].influence, [0, 1]);
        } else {
            assert!(g.pending.is_none());
            // A hidden character supplies its ordinary generic influence, even
            // though the concealed printed renown never grants a reward.
            assert_eq!(g.regions[0].influence, [0, u32::from(kind == "hidden")]);
        }
    }
}

#[test]
fn renown_shared_placement_cancels_enemy_influence_and_only_then_wins() {
    let mut g = game();
    field(&mut g, "JC070", 0);
    g.regions[0].influence = [0, 2];
    end_contests(&mut g);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[0].influence, [0, 1]);
    assert_eq!(g.window, Some(Window::After(0, 2)));
}

#[test]
fn renown_normal_influence_win_skips_old_and_replacement_renown() {
    let mut g = game();
    for _ in 0..3 {
        field(&mut g, "JC075", 0);
    }
    field(&mut g, "JC070", 2);
    let old = g.regions[0].card.id.clone();
    end_contests(&mut g);
    assert!(matches!(g.window, Some(Window::Win(0, 0))));
    assert!(g.pending.is_none() && g.stack.is_empty());
    for _ in 0..80 {
        if g.regions[0].card.id != old {
            break;
        }
        if let Some(p) = g.pending.clone() {
            choose(
                &mut g,
                p.choice.options.iter().map(|o| o.id.clone()).collect(),
            );
        } else {
            pass(&mut g);
        }
    }
    assert_ne!(g.regions[0].card.id, old);
    assert_eq!(g.players[0].score_cards.len(), 1);
    decline_region_win_trigger(&mut g);
    let replacement = field(&mut g, "JC070", 2); // Explicit sentinel after replacement.
    for _ in 0..4 {
        pass(&mut g);
    }
    assert_eq!(g.window, Some(Window::Before(1, 0)));
    assert_eq!(g.regions[0].influence, [0, 0]);
    assert!(g.board(&replacement).is_some());
}

#[test]
fn renown_award_uses_real_win_return_score_and_does_not_repeat() {
    let mut g = game();
    let source = field(&mut g, "JC070", 0);
    g.regions[0].influence = [2, 0];
    let old = g.regions[0].card.id.clone();
    end_contests(&mut g);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.window, Some(Window::Win(0, 0)));
    assert_eq!(g.regions[0].influence, [3, 0]);
    for _ in 0..4 {
        pass(&mut g);
    }
    assert_ne!(g.regions[0].card.id, old);
    assert!(g.board(&source).is_none());
    assert_eq!(g.players[0].score_cards.len(), 1);
    assert_eq!(g.players[0].deck.last().unwrap().definition, "JC070");
    assert_eq!(g.window, Some(Window::After(0, 2)));
    decline_region_win_trigger(&mut g);
    assert!(g.pending.is_none() && g.stack.is_empty());
}

#[test]
fn renown_declared_reward_is_independent_of_source_leaving() {
    let mut g = game();
    let source = field(&mut g, "JC070", 0);
    fund(&mut g, 2, "JC002", 2);
    end_contests(&mut g);
    accept(&mut g);
    let reply = held(&mut g, "JC006", 2);
    give_priority(&mut g, 2);
    g.apply(
        2,
        Action {
            card_id: Some(reply),
            target_id: Some(source.clone()),
            ..Action::new("play")
        },
    )
    .unwrap();
    pass_top(&mut g);
    assert!(g.board(&source).is_none());
    pass_top(&mut g);
    assert_eq!(g.regions[0].influence, [1, 0]);
}

#[test]
fn renown_grant_follows_current_controller_and_exact_instance_until_cleanup() {
    let mut g = game();
    let target = grants(&mut g, "JC125", 2);
    fund(&mut g, 0, "JC104", 3);
    cast(&mut g, "JC129", 0, &target);
    assert_eq!(g.board(&target).unwrap().1.controller, 0);
    end_contests(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[0].influence, [1, 0]);
    let printed = field(&mut g, "JC070", 0);
    g.begin_window(Window::End);
    let turn = g.turn;
    for _ in 0..16 {
        if g.turn > turn {
            break;
        }
        pass(&mut g);
    }
    assert_eq!(g.turn, turn + 1);
    assert!(g.turn_attribute_modifiers.is_empty());
    assert_eq!(g.board(&target).unwrap().1.controller, 2);
    assert!(!g.has_renown(g.board(&target).unwrap().1));
    assert!(g.has_renown(g.board(&printed).unwrap().1));
}

#[test]
fn renown_public_can_hide_by_lawyer_and_grants_do_not_survive_new_identity() {
    let mut g = game();
    let target = grants(&mut g, "JC070", 0);
    fund(&mut g, 0, "XQ16", 2);
    let lawyer = held(&mut g, "XQ16", 0);
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
    choose(&mut g, vec![target.clone()]);
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    assert!(g.turn_attribute_modifiers.is_empty());
    let hidden = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC070")
        .unwrap();
    assert!(hidden.face_down && !g.has_renown(hidden));
    for seat in 0..4 {
        let view = g.view(seat);
        let c = view.regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == hidden.id)
            .unwrap();
        assert_eq!(c.current_renown, None);
        if seat != 0 {
            assert!(c.card_id.is_none() && c.text.is_none());
        }
    }
    let hidden = hidden.id.clone();
    fund(&mut g, 0, "JC075", 1);
    g.apply(
        0,
        Action {
            card_id: Some(hidden.clone()),
            ..Action::new("reveal")
        },
    )
    .unwrap();
    pass_top(&mut g);
    let new = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC070")
        .unwrap();
    assert_ne!(new.id, hidden);
    assert!(g.has_renown(new));
    assert_eq!(g.icons(new, 0).investigation, 1);
}

#[test]
fn renown_skipped_region_and_stale_region_identity_cannot_get_a_reward() {
    let mut g = game();
    field(&mut g, "JC070", 0);
    g.regions[0].skip = true;
    end_contests(&mut g);
    assert!(g.pending.is_none());
    g.regions[0].skip = false;
    g.begin_window(Window::After(0, 2));
    let old = g.regions[0].card.id.clone();
    g.declare_region_renown(0, &old);
    g.drive().unwrap();
    accept(&mut g);
    g.regions[0].card = g.make_card("DQJC109", 0); // Explicit guard primitive, not a natural action.
    mirror(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[0].influence, [0, 0]);
}
