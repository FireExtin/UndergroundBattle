//! Explicit native layouts; actions use actual payment and response paths.
//! Every command round-trips Game; checkpoints also verify Room and four views.
use crate::jc029_tests::{checkpoint, pass_top};
use crate::{
    batch4_vampire_contest as b, batch4_vampires as v, model::*, mystic_contest as m, rules::*,
    unit_support::*,
};
fn layout() -> Game {
    let mut g = game();
    for seat in 0..4 {
        deck_cards(&mut g, seat, &["JC125", "JC125", "JC125"]);
    }
    g
}
fn grave(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.players[seat].graveyard.push(c);
    id
}
fn attached(g: &mut Game, id: &str, seat: usize, host: &str) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.attachments.push(Attachment {
        card: c,
        host_id: host.into(),
    });
    id
}
fn hand_action(id: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(b::HAND.into()),
        ..Action::new("activate")
    }
}
fn sacrifice_action(id: &str, cost: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(b::SACRIFICE.into()),
        cost_selected: Some(vec![cost.into()]),
        ..Action::new("activate")
    }
}
fn start(g: &mut Game) {
    fund(g, 0, "JC125", 2);
    let id = held(g, "BQ032", 0);
    act(g, 0, hand_action(&id));
    checkpoint(g);
    pass_top(g);
    checkpoint(g);
}
fn commit(g: &mut Game, ids: [&str; 4]) {
    for (seat, id) in ids.into_iter().enumerate() {
        assert_eq!(g.pending.as_ref().unwrap().seat, seat);
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, m::COMMIT_KIND);
        let uid = g.players[seat]
            .hand
            .iter()
            .find(|c| c.definition == id)
            .unwrap()
            .id
            .clone();
        choose(g, vec![uid]);
        checkpoint(g);
    }
}
fn rewards(g: &mut Game) {
    for _ in 0..4 {
        if g.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == m::REWARD_KIND)
        {
            choose(g, vec![]);
        } else {
            break;
        }
    }
    checkpoint(g);
}
fn trigger(g: &mut Game, key: &str, accept: bool) {
    for _ in 0..20 {
        let p = g.pending.clone().expect("trigger");
        match &p.resolution {
            ChoiceResolution::Batch3EntryOrder { .. } => {
                let uid = p
                    .choice
                    .options
                    .iter()
                    .find(|o| o.id.ends_with(key))
                    .unwrap()
                    .id
                    .clone();
                choose(g, vec![uid]);
            }
            ChoiceResolution::Declare { declaration, .. } if declaration.ability.key == key => {
                choose(
                    g,
                    if accept {
                        vec!["accept".into()]
                    } else {
                        vec![]
                    },
                );
                return;
            }
            ChoiceResolution::Declare { .. } => choose(g, vec![]),
            _ => panic!("unexpected {}", p.choice.kind),
        }
    }
    panic!("trigger bound");
}
fn drain_optional(g: &mut Game) {
    for _ in 0..40 {
        if let Some(p) = g.pending.clone() {
            match p.resolution {
                ChoiceResolution::Batch3EntryOrder { .. } => {
                    choose(g, vec![p.choice.options[0].id.clone()])
                }
                ChoiceResolution::Declare { .. } => choose(g, vec![]),
                _ => panic!("unexpected {}", p.choice.kind),
            }
        } else if !g.stack.is_empty() {
            pass(g);
        } else {
            return;
        }
    }
    panic!("drain bound");
}
fn invalid(g: &Game, bad: Game) {
    let raw = serde_json::to_string(&bad).unwrap();
    assert!(std::panic::catch_unwind(|| Game::from_persisted(&raw))
        .unwrap()
        .is_err());
    let mut r = serde_json::to_value(crate::jc029_tests::envelope(g)).unwrap();
    r["game"] = serde_json::to_value(bad).unwrap();
    assert!(
        std::panic::catch_unwind(|| crate::room::RoomEnvelope::from_persisted(&r.to_string()))
            .unwrap()
            .is_err()
    );
}

#[test]
fn b_contest_hand_action_pays_two_discards_self_once_and_private_restore() {
    let mut g = layout();
    fund(&mut g, 0, "JC125", 2);
    let id = held(&mut g, "BQ032", 0);
    let other = held(&mut g, "BQ032", 0);
    assert!(g
        .legal_actions(0)
        .iter()
        .any(|a| a.action == hand_action(&id)));
    act(&mut g, 0, hand_action(&id));
    assert_eq!(g.resources(0), 0);
    assert!(g.players[0].hand.iter().any(|c| c.id == other));
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "BQ032" && c.id != id));
    let f = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert!(
        matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids),PaidCost::Discarded{old_instance,..}] if ids.len()==2&&old_instance==&id)
    );
    checkpoint(&g);
    pass_top(&mut g);
    assert!(g.mystic_contest.is_some());
    for viewer in 1..4 {
        assert!(g.view(viewer).pending_choice.is_none());
    }
    commit(&mut g, ["JC125"; 4]);
    assert_eq!(g.mystic_history[0].actor_bonus, 0);
    assert!(g.mystic_contest.is_none());
    assert_eq!(g.resources(0), 0);
    reject(&mut g, 0, hand_action(&id));
}
#[test]
fn b_contest_hand_invalid_cost_extra_payload_board_and_enemy_are_atomic() {
    let mut g = layout();
    let id = held(&mut g, "BQ032", 0);
    fund(&mut g, 0, "JC125", 1);
    reject(&mut g, 0, hand_action(&id));
    fund(&mut g, 0, "JC125", 1);
    let other = held(&mut g, "JC125", 0);
    let mut a = hand_action(&id);
    a.cost_selected = Some(vec![other]);
    reject(&mut g, 0, a);
    let mut a = hand_action(&id);
    a.region = Some(2);
    reject(&mut g, 0, a);
    let board = field(&mut g, "BQ032", 0);
    reject(&mut g, 0, hand_action(&board));
    reject(&mut g, 1, hand_action(&id));
    let before = serde_json::to_string(&g).unwrap();
    let mut no_cards = g.clone();
    no_cards.players[2].hand.clear();
    no_cards.players[2].deck.clear();
    act(&mut no_cards, 0, hand_action(&id));
    let mut steps = 0;
    while no_cards.mystic_contest.is_none() && steps < 4 {
        let seat = (0..4).find(|s| {
            no_cards
                .legal_actions(*s)
                .iter()
                .any(|a| a.action.kind == "pass")
        });
        if let Some(s) = seat {
            if no_cards.apply(s, Action::new("pass")).is_err() {
                break;
            }
        } else {
            break;
        }
        steps += 1;
    }
    assert!(no_cards.mystic_contest.is_none());
    assert_eq!(no_cards.resources(0), 0);
    assert_eq!(serde_json::to_string(&g).unwrap(), before);
}
#[test]
fn b_contest_bq036_payment_accepts_personal_attachment_on_personal_character_and_not_printed_cost()
{
    let mut g = layout();
    let source = field(&mut g, "BQ036", 0);
    let host = field(&mut g, "JC029", 0);
    let cost = attached(&mut g, "XQ14", 0, &host);
    fund(&mut g, 0, "JC125", 1);
    let a = sacrifice_action(&source, &cost);
    assert!(g.legal_actions(0).iter().any(|legal| legal.action == a));
    act(&mut g, 0, a);
    assert_eq!(g.resources(0), 0);
    assert!(!g.attachments.iter().any(|a| a.card.id == cost));
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ14" && c.id != cost));
    checkpoint(&g);
    g.remove_dead(&source, RemovalCause::Destroy);
    pass_top(&mut g);
    commit(&mut g, ["JC125"; 4]);
    assert_eq!(g.mystic_history[0].shown[0].value, 2);
    assert_eq!(g.mystic_history[0].shown[1].value, 0);
    rewards(&mut g);
    assert!(g.mystic_contest.is_none());
}
#[test]
fn b_contest_bq036_rejects_foreign_cost_wrong_host_and_double_payment_restore() {
    let mut g = layout();
    let source = field(&mut g, "BQ036", 0);
    let host = field(&mut g, "JC029", 0);
    let ally = field(&mut g, "JC029", 1);
    fund(&mut g, 0, "JC125", 1);
    let foreign = attached(&mut g, "XQ14", 1, &host);
    let wrong = attached(&mut g, "XQ14", 0, &ally);
    let society = attached(&mut g, "BQ038", 0, "society:p0");
    for id in [foreign, wrong, society, host.clone()] {
        reject(&mut g, 0, sacrifice_action(&source, &id));
    }
    let cost = attached(&mut g, "XQ14", 0, &host);
    act(&mut g, 0, sacrifice_action(&source, &cost));
    let mut bad = g.clone();
    bad.stack
        .last_mut()
        .unwrap()
        .frame
        .as_mut()
        .unwrap()
        .already_paid
        .pop();
    invalid(&g, bad);
    let mut bad = g.clone();
    let f = bad.stack.last_mut().unwrap().frame.as_mut().unwrap();
    if let PaidCost::BQ036Attachment(p) = &mut f.already_paid[1] {
        p.attachment.card.controller = 1;
    }
    invalid(&g, bad);
}
#[test]
fn b_contest_shown_bq027_adds_two_after_shuffle_targets_only_personal_vampire_and_feeds() {
    let mut g = layout();
    let target = field(&mut g, "JC029", 0);
    let ally = field(&mut g, "JC029", 1);
    let human = field(&mut g, "JC125", 0);
    let blood = grave(&mut g, "XQ14", 0);
    let shown = held(&mut g, "BQ027", 0);
    start(&mut g);
    commit(&mut g, ["BQ027", "JC125", "JC125", "JC125"]);
    assert_eq!(g.mystic_history[0].shown[0].value, 3);
    assert!(g.players[0]
        .deck
        .iter()
        .any(|c| c.definition == "BQ027" && c.id != shown));
    rewards(&mut g);
    let p = g.pending.as_ref().unwrap();
    assert_eq!(p.choice.kind, b::TARGET);
    assert_eq!(p.choice.min, Some(1));
    let options = p
        .choice
        .options
        .iter()
        .map(|o| o.id.clone())
        .collect::<Vec<_>>();
    assert_eq!(options, vec![target.clone()]);
    assert!(!options.contains(&ally) && !options.contains(&human));
    let random = g.random;
    choose(&mut g, vec![target.clone()]);
    assert_eq!(
        g.pending.as_ref().unwrap().choice.kind,
        "batch4-vampire-blood"
    );
    checkpoint(&g);
    choose(&mut g, vec![blood]);
    assert_eq!(g.random, random);
    assert!(g.mystic_contest.is_none());
    assert!(g.attachments.iter().any(|a| a.host_id == target));
    checkpoint(&g);
}
#[test]
fn b_contest_shown_bq027_no_target_or_no_blood_finishes_and_two_shown_keep_cursor() {
    for blood_available in [false, true] {
        let mut g = layout();
        if blood_available {
            grave(&mut g, "XQ14", 0);
        } else {
            field(&mut g, "JC029", 0);
        }
        held(&mut g, "BQ027", 0);
        start(&mut g);
        commit(&mut g, ["BQ027", "JC125", "JC125", "JC125"]);
        rewards(&mut g);
        if g.pending.is_some() {
            let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
            choose(&mut g, vec![id]);
        }
        assert!(g.pending.is_none() && g.mystic_contest.is_none());
        checkpoint(&g);
    }
    let mut g = layout();
    let zero = field(&mut g, "JC029", 0);
    let two = field(&mut g, "JC029", 2);
    grave(&mut g, "XQ14", 0);
    grave(&mut g, "BQ040", 2);
    held(&mut g, "BQ027", 0);
    held(&mut g, "BQ027", 2);
    start(&mut g);
    commit(&mut g, ["BQ027", "JC125", "BQ027", "JC125"]);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    choose(&mut g, vec![zero]);
    let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
    choose(&mut g, vec![id]);
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    choose(&mut g, vec![two]);
    let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
    choose(&mut g, vec![id]);
    assert!(g.mystic_contest.is_none());
    checkpoint(&g);
}
#[test]
fn b_contest_shown_bq027_rejects_post_frame_target_and_cursor_forgery() {
    let mut g = layout();
    let target = field(&mut g, "JC029", 0);
    grave(&mut g, "XQ14", 0);
    held(&mut g, "BQ027", 0);
    start(&mut g);
    commit(&mut g, ["BQ027", "JC125", "JC125", "JC125"]);
    rewards(&mut g);
    let mut bad = g.clone();
    if let m::MysticPhase::After { next_shown, .. } =
        &mut bad.mystic_contest.as_mut().unwrap().phase
    {
        *next_shown += 1;
    }
    invalid(&g, bad);
    let mut bad = g.clone();
    if let ChoiceResolution::Frame { frame, .. } = &mut bad.pending.as_mut().unwrap().resolution {
        frame.ability_key = v::FEED_TARGET.into();
    }
    invalid(&g, bad);
    choose(&mut g, vec![target]);
    let mut bad = g.clone();
    if let ChoiceResolution::Frame { frame, .. } = &mut bad.pending.as_mut().unwrap().resolution {
        frame.targets.clear();
    }
    invalid(&g, bad);
    checkpoint(&g);
}
#[test]
fn b_contest_bq031_win_is_personal_each_contest_and_shares_victory_order_with_a() {
    let mut g = layout();
    let own = field(&mut g, "BQ031", 0);
    let ally = field(&mut g, "BQ031", 1);
    field(&mut g, "WM022", 0);
    field(&mut g, "WM032", 0);
    grave(&mut g, "XQ14", 0);
    grave(&mut g, "XQ14", 0);
    grave(&mut g, "BQ040", 1);
    for n in 0..2 {
        g.begin_window(Window::Action(0));
        held(&mut g, "WM021", 0);
        start(&mut g);
        commit(&mut g, ["WM021", "JC125", "JC125", "JC125"]);
        rewards(&mut g);
        trigger(&mut g, b::WIN, true);
        drain_until_blood(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().seat, 0);
        let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
        choose(&mut g, vec![id]);
        drain_optional(&mut g);
        assert_eq!(
            g.attachments.iter().filter(|a| a.host_id == own).count(),
            n + 1
        );
    }
    assert!(!g.attachments.iter().any(|a| a.host_id == ally));
    assert_eq!(g.mystic_history.len(), 2);
    assert!(g.mystic_history[1].first_winners.is_empty());
}
fn drain_until_blood(g: &mut Game) {
    for _ in 0..30 {
        if g.pending.as_ref().is_some_and(|p| {
            matches!(
                p.resolution,
                ChoiceResolution::Frame {
                    choice: FrameChoice::VampireBlood,
                    ..
                }
            )
        }) {
            return;
        }
        if let Some(p) = g.pending.clone() {
            match p.resolution {
                ChoiceResolution::Batch3EntryOrder { .. } => {
                    choose(g, vec![p.choice.options[0].id.clone()])
                }
                ChoiceResolution::Declare { .. } => choose(g, vec![]),
                _ => panic!("unexpected {}", p.choice.kind),
            }
        } else if !g.stack.is_empty() {
            pass(g);
        } else {
            panic!("no blood");
        }
    }
    panic!("blood bound");
}
#[test]
fn b_contest_bq031_departure_does_not_feed_replacement_and_win_receipt_is_required() {
    let mut g = layout();
    let source = field(&mut g, "BQ031", 0);
    grave(&mut g, "XQ14", 0);
    held(&mut g, "WM021", 0);
    start(&mut g);
    commit(&mut g, ["WM021", "JC125", "JC125", "JC125"]);
    rewards(&mut g);
    trigger(&mut g, b::WIN, true);
    let mut bad = g.clone();
    let f = bad.stack.last_mut().unwrap().frame.as_mut().unwrap();
    f.steps[0].op = Op::BQ031WinFeed {
        contest_id: Some("i1".into()),
    };
    invalid(&g, bad);
    g.remove_dead(&source, RemovalCause::Destroy);
    field(&mut g, "BQ031", 0);
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert!(g.attachments.is_empty());
    checkpoint(&g);
}
#[test]
fn b_contest_bq038_start_is_respondable_counts_each_personal_host_once_and_freezes() {
    let mut g = layout();
    let source = attached(&mut g, "BQ038", 0, "society:p0");
    let first = field(&mut g, "JC029", 0);
    let second = field(&mut g, "JC125", 0);
    let ally = field(&mut g, "JC029", 1);
    attached(&mut g, "XQ14", 0, &first);
    attached(&mut g, "BQ040", 2, &first);
    attached(&mut g, "BQ022", 2, &second);
    attached(&mut g, "XQ14", 0, &ally);
    start(&mut g);
    assert!(matches!(
        g.mystic_contest.as_ref().unwrap().phase,
        m::MysticPhase::BeforeSelection
    ));
    assert!(g.players.iter().all(|p| p.hand.is_empty()));
    trigger(&mut g, b::START, true);
    assert!(g.pending.is_none());
    checkpoint(&g);
    g.remove_dead(&source, RemovalCause::Destroy);
    pass_top(&mut g);
    assert_eq!(g.mystic_contest.as_ref().unwrap().bonuses[0].amount, 2);
    g.remove_dead(&first, RemovalCause::Destroy);
    g.remove_dead(&second, RemovalCause::Destroy);
    checkpoint(&g);
    commit(&mut g, ["JC125"; 4]);
    assert_eq!(g.mystic_history[0].shown[0].value, 2);
    assert_eq!(g.mystic_history[0].shown[1].value, 0);
    rewards(&mut g);
    checkpoint(&g);
}
#[test]
fn b_contest_bq038_decline_and_zero_are_distinct_and_bonus_tampering_is_rejected() {
    for accept in [false, true] {
        let mut g = layout();
        attached(&mut g, "BQ038", 0, "society:p0");
        start(&mut g);
        trigger(&mut g, b::START, accept);
        if accept {
            pass_top(&mut g);
        }
        let state = g.mystic_contest.as_ref().unwrap();
        assert_eq!(state.bonuses.len(), usize::from(accept));
        if accept {
            assert_eq!(state.bonuses[0].amount, 0);
            let mut bad = g.clone();
            bad.mystic_contest.as_mut().unwrap().bonuses[0].amount = 1;
            invalid(&g, bad);
            let mut bad = g.clone();
            let mut extra = bad.mystic_contest.as_ref().unwrap().bonuses[0].clone();
            extra.frame.frame_id = bad.id();
            bad.mystic_contest.as_mut().unwrap().bonuses.push(extra);
            invalid(&g, bad);
        }
        commit(&mut g, ["JC125"; 4]);
        assert!(g.mystic_contest.is_none());
        checkpoint(&g);
    }
}
#[test]
fn b_contest_programs_cannot_be_transplanted_or_expanded() {
    for id in ["BQ031", "BQ032", "BQ036", "BQ038"] {
        let a = b::ability_for(id).unwrap();
        assert!(validate_ability("JC125", &a).is_err());
        let mut bad = a.clone();
        bad.ops.push(Op::MysticContest { actor_bonus: 99 });
        assert!(validate_ability(id, &bad).is_err());
        let mut bad = a;
        bad.timing = Timing::ActionFast;
        assert!(validate_ability(id, &bad).is_err());
    }
    let mut g = layout();
    fund(&mut g, 0, "JC125", 2);
    let source = held(&mut g, "BQ032", 0);
    act(&mut g, 0, hand_action(&source));
    for change in 0..4 {
        let mut bad = g.clone();
        let f = bad.stack.last_mut().unwrap().frame.as_mut().unwrap();
        match change {
            0 => f.source.card.definition = "JC125".into(),
            1 => f.steps[0].op = Op::MysticContest { actor_bonus: 2 },
            2 => f.already_paid.clear(),
            _ => f.steps.push(f.steps[0].clone()),
        }
        invalid(&g, bad);
    }
}

#[test]
fn b_contest_bq038_real_fast_response_changes_count_only_at_resolution() {
    let mut g = layout();
    attached(&mut g, "BQ038", 0, "society:p0");
    let target = field(&mut g, "JC029", 0);
    let sacrifice = field(&mut g, "BQ027", 0);
    let blood = grave(&mut g, "XQ14", 0);
    start(&mut g);
    trigger(&mut g, b::START, true);
    let bonus_frame = g.stack.last().unwrap().id.clone();
    // Move the opposing priority window back to this team with real passes.
    for _ in 0..4 {
        if g.priority_team == g.team(0) {
            break;
        }
        pass(&mut g);
    }
    act(
        &mut g,
        0,
        Action {
            card_id: Some(sacrifice),
            target_id: Some(target.clone()),
            ability_id: Some(v::FEED_TARGET.into()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    assert_eq!(
        g.pending.as_ref().unwrap().choice.kind,
        "batch4-vampire-blood"
    );
    assert!(g.mystic_contest.as_ref().unwrap().bonuses.is_empty());
    choose(&mut g, vec![blood]);
    assert_eq!(g.stack.last().unwrap().id, bonus_frame);
    assert!(g.players.iter().all(|p| p.hand.is_empty()));
    pass_top(&mut g);
    assert_eq!(g.mystic_contest.as_ref().unwrap().bonuses[0].amount, 1);
    commit(&mut g, ["JC125"; 4]);
    rewards(&mut g);
    checkpoint(&g);
}
#[test]
fn b_contest_shown_bq027_blood_entry_observer_and_other_shown_keep_recovery() {
    let mut g = layout();
    let target = field(&mut g, "JC029", 0);
    field(&mut g, "BQ031", 0);
    let blood = grave(&mut g, "XQ14", 0);
    held(&mut g, "BQ027", 0);
    held(&mut g, "WM061", 2);
    start(&mut g);
    commit(&mut g, ["BQ027", "JC125", "WM061", "JC125"]);
    rewards(&mut g);
    choose(&mut g, vec![target]);
    choose(&mut g, vec![blood]);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
    drain_optional(&mut g);
    assert_eq!(g.players[2].graveyard.len(), 1);
    checkpoint(&g);
}

#[test]
fn b_contest_paid_sources_preserve_foreign_owners_and_hand_action_cannot_respond() {
    let mut g = layout();
    fund(&mut g, 0, "JC125", 2);
    let mut c = g.make_card("BQ032", 1);
    c.controller = 0;
    let id = c.id.clone();
    g.players[0].hand.push(c);
    act(&mut g, 0, hand_action(&id));
    assert!(g.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "BQ032" && c.controller == 1));
    assert!(g.players[0].graveyard.is_empty());
    fund(&mut g, 2, "JC125", 2);
    let response = held(&mut g, "BQ032", 2);
    reject(&mut g, 2, hand_action(&response));
    checkpoint(&g);

    let mut g = layout();
    let source = field(&mut g, "BQ036", 0);
    let host = field(&mut g, "JC029", 0);
    let cost = attached(&mut g, "XQ14", 1, &host);
    g.attachments.last_mut().unwrap().card.controller = 0;
    fund(&mut g, 0, "JC125", 1);
    act(&mut g, 0, sacrifice_action(&source, &cost));
    assert!(g.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ14" && c.controller == 1));
    checkpoint(&g);
}

#[test]
fn b_contest_bq038_witnesses_and_capture_identity_cannot_be_rewritten() {
    let mut g = layout();
    attached(&mut g, "BQ038", 0, "society:p0");
    let host = field(&mut g, "JC029", 0);
    let blood = attached(&mut g, "XQ14", 2, &host);
    start(&mut g);
    trigger(&mut g, b::START, true);
    pass_top(&mut g);
    for mutation in 0..5 {
        let mut bad = g.clone();
        let bonus = &mut bad.mystic_contest.as_mut().unwrap().bonuses[0];
        let Op::BQ038ContestBonus {
            observation: Some(o),
        } = &mut bonus.frame.steps[0].op
        else {
            unreachable!()
        };
        match mutation {
            0 => {
                let w = o.witnesses.as_ref().unwrap()[0].clone();
                o.witnesses.as_mut().unwrap().push(w);
                bonus.amount = 2;
            }
            1 => o.witnesses.as_mut().unwrap()[0].character.card.controller = 1,
            2 => o.witnesses.as_mut().unwrap()[0].attachment.host_id = "i1".into(),
            3 => o.resolved_id = Some(blood.clone()),
            _ => o.resolved_id = None,
        }
        invalid(&g, bad);
    }
    commit(&mut g, ["JC125"; 4]);
    rewards(&mut g);
    let mut bad = g.clone();
    let extra = bad.mystic_history[0].bonuses[0].clone();
    bad.mystic_history[0].bonuses.push(extra);
    invalid(&g, bad);
    checkpoint(&g);
}

// This is a real B start + BQ038 start-trigger + paid WM022 response, not a
// seeded public-hook fixture. A owns the protocol change. Keep the expected
// behavior as an explicit ignored regression until that contract is delivered.
#[test]
#[ignore = "A-owned start response/nesting protocol is unresolved on baseline 47256d59"]
fn b_contest_protocol_response_reveal_must_dispatch_before_outer_commit() {
    let mut g = layout();
    attached(&mut g, "BQ038", 0, "society:p0");
    fund(&mut g, 2, "WM022", 3);
    let reveal = field(&mut g, "WM022", 2);
    g.board_mut(&reveal).unwrap().face_down = true;
    start(&mut g);
    trigger(&mut g, b::START, true);
    for _ in 0..4 {
        if g.priority_team == g.team(2) {
            break;
        }
        pass(&mut g);
    }
    assert_eq!(g.priority_team, g.team(2));
    act(
        &mut g,
        2,
        Action {
            card_id: Some(reveal),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut g);
    assert!(g
        .pending
        .as_ref()
        .is_none_or(|p| p.choice.kind != m::COMMIT_KIND));
    assert!(g.stack.last().is_some_and(|s| s.frame.as_ref().is_some_and(|f| f.ability_key == crate::pack_a_contest::SHADOW_REVEAL)),
        "The response-generated forced reveal declaration must reach the stack before the outer start resumes; stack={:?}; effects={:?}", g.stack.iter().map(|s|s.frame.as_ref().map(|f|f.ability_key.as_str())).collect::<Vec<_>>(), g.effects.iter().map(|e|match e { Effect::Declare{declaration}=>declaration.ability.key.as_str(), Effect::MysticContinue{..}=>"MysticContinue", _=>"other" }).collect::<Vec<_>>());
    checkpoint(&g);
}
