//! Prepared Native layouts for UI/ABI regression, not natural browser playtests.
use hegemony_server::model::{Action, Game, Window};
use hegemony_server::room::{RoomCommand, RoomEnvelope, SessionAction};
use serde_json::json;

fn apply(g: &mut Game, seat: usize, action: Action) {
    g.apply(seat, action).unwrap();
}
fn choose(g: &mut Game, selected: Vec<String>) {
    let pending = g.pending.as_ref().unwrap();
    let seat = pending.seat;
    let choice_id = pending.choice.id.clone();
    apply(
        g,
        seat,
        Action {
            choice_id: Some(choice_id),
            selected: Some(selected),
            ..Action::new("choose")
        },
    );
}
fn pass_top(g: &mut Game) {
    let count = g.stack.len();
    for _ in 0..32 {
        if g.stack.len() < count || g.pending.is_some() {
            return;
        }
        let seat = (0..4)
            .find(|seat| {
                g.legal_actions(*seat)
                    .iter()
                    .any(|a| a.action.kind == "pass")
            })
            .unwrap();
        apply(g, seat, Action::new("pass"));
    }
    panic!("response did not finish");
}
fn fixture(attachment: bool, discard: bool) -> serde_json::Value {
    let mut g = Game::new(
        "595959595959595959595959".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        9,
    )
    .unwrap();
    for seat in 1..4 {
        g.join(format!("P{seat}"), "watchers".into()).unwrap();
    }
    for p in &mut g.players {
        p.ready = true;
    }
    apply(&mut g, 0, Action::new("start"));
    while g.pending.is_some() {
        choose(&mut g, vec![]);
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
        p.graveyard.clear();
    }
    for r in &mut g.regions {
        r.cards.clear();
        r.influence = [0; 2];
    }
    g.first_team = g.team(0);
    g.active_team = 0;
    g.priority_team = 0;
    g.window = Some(Window::Action(0));
    g.passed.clear();
    g.team_passed = [false; 2];
    for definition in if attachment {
        vec!["JC125", "BQ040"]
    } else {
        vec!["JC125"]
    } {
        let card = g.make_card(definition, 1);
        g.players[1].hand.push(card);
    }
    let source = g.make_card("BQ028", 0);
    let source_id = source.id.clone();
    g.players[0].hand.push(source);
    for _ in 0..3 {
        let card = g.make_card("BQ028", 0);
        g.players[0].assets.push(card);
    }
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(source_id),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
    let hidden = g.regions[2].cards.last().unwrap().id.clone();
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut g);
    choose(&mut g, vec!["p1".into()]);
    pass_top(&mut g);
    assert_eq!(
        g.pending.as_ref().unwrap().choice.kind,
        "bq028-hand-inspect"
    );
    let choice = g.pending.as_ref().unwrap().choice.clone();
    let selected = if discard {
        vec![choice.options[0].id.clone()]
    } else {
        vec![]
    };
    let action = Action {
        choice_id: Some(choice.id),
        selected: Some(selected),
        ..Action::new("choose")
    };
    let room = RoomEnvelope::from_game(g);
    let before = serde_json::to_string(&room).unwrap();
    RoomEnvelope::from_persisted(&before).unwrap();
    let views: Vec<_> = (0..4)
        .map(|seat| room.view(seat, room.pacing.last_server_now_ms))
        .collect();
    let command = RoomCommand {
        command_id: format!("seven-ui-{attachment}-{discard}"),
        expected_version: room.revision,
        action: SessionAction::Game { action },
    };
    let transition = room.transition(0, Some(command.clone()), 1000).unwrap();
    assert_eq!(transition.outcome, "accepted");
    let next = RoomEnvelope::from_persisted(&transition.state).unwrap();
    json!({"attachment":attachment,"discard":discard,"before":before,"views":views,"command":command,"after":transition.state,"afterViews":(0..4).map(|seat|next.view(seat,next.pacing.last_server_now_ms)).collect::<Vec<_>>()})
}
fn main() {
    let path = std::env::args().nth(1).expect("fixture output path");
    let fixtures = vec![
        fixture(false, false),
        fixture(true, false),
        fixture(true, true),
    ];
    std::fs::write(path, serde_json::to_vec_pretty(&fixtures).unwrap()).unwrap();
}
