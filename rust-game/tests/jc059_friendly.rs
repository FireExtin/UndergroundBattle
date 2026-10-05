use hegemony_server::{
    catalog,
    model::{Card, Game, Icons},
};

fn put(game: &mut Game, definition: &str, controller: usize, region: usize, hidden: bool) -> Card {
    let mut card = game.make_card(definition, controller);
    card.face_down = hidden;
    game.regions[region].cards.push(card.clone());
    card
}

#[test]
fn public_region_totals_include_team_temporary_hidden_ready_and_static_icons_without_identity() {
    let mut game = Game::new(
        "totals".into(),
        "TOTALS".into(),
        "teams".into(),
        "P0".into(),
        "keepers".into(),
        7312,
    )
    .unwrap();
    for seat in 1..4 {
        game.join(format!("P{seat}"), "watchers".into()).unwrap();
    }
    for player in &mut game.players {
        player.ready = true;
    }
    game.apply(0, hegemony_server::model::Action::new("start"))
        .unwrap();
    // Explicit layout fixture: both teammates contribute, white icons only for team 1.
    game.first_team = 1;
    put(&mut game, "LC21", 0, 0, false);
    put(&mut game, "JZ08", 1, 0, false);
    let hidden = put(&mut game, "LC20", 0, 0, true);
    put(&mut game, "LC21", 2, 0, false);
    put(&mut game, "JC059", 2, 0, false);
    put(&mut game, "JZ08", 3, 0, false);
    put(&mut game, "JZ08", 3, 0, false);
    game.regions[0].cards.last_mut().unwrap().exhausted = true;
    put(&mut game, "LC20", 3, 0, true);
    game.regions[0].cards.last_mut().unwrap().exhausted = true;
    // Enemy hidden entities are not characters: the elite's continuous bonus applies.
    put(&mut game, "JC016", 2, 1, false);
    put(&mut game, "LC20", 0, 1, true);
    game.regions[1].skip = true;
    let expected = [
        Icons {
            investigation: 0,
            combat: 1,
            influence: 1,
        },
        Icons {
            investigation: 0,
            combat: 4,
            influence: 1,
        },
    ];
    for seat in 0..4 {
        let view = game.view(seat);
        assert!(!view.regions[0].skip_confrontation);
        assert!(view.regions[1].skip_confrontation);
        assert_eq!(view.regions[0].icons_by_team, expected);
        assert_eq!(
            view.regions[1].icons_by_team,
            [
                Icons {
                    investigation: 0,
                    combat: 0,
                    influence: 1
                },
                Icons {
                    investigation: 1,
                    combat: 2,
                    influence: 1
                },
            ]
        );
        if seat != 0 {
            let projected = view.regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == hidden.id)
                .unwrap();
            assert!(projected.card_id.is_none() && projected.icons.is_none());
            assert_eq!(projected.name, "暗藏者");
        }
    }
}

#[test]
fn friendly_defense_includes_teammate_but_not_enemy_other_region_hidden_or_source() {
    let mut game = Game::new(
        "aura".into(),
        "AURA".into(),
        "teams".into(),
        "P0".into(),
        "keepers".into(),
        7312,
    )
    .unwrap();
    for seat in 1..4 {
        game.join(format!("P{seat}"), "watchers".into()).unwrap();
    }
    for player in &mut game.players {
        player.ready = true;
    }
    game.apply(0, hegemony_server::model::Action::new("start"))
        .unwrap();
    // Initial layout fixture; defense queries check the printed continuous rule.
    let source = put(&mut game, "JC059", 0, 2, false);
    let own = put(&mut game, "LC20", 0, 2, false);
    let friend = put(&mut game, "LC20", 1, 2, false);
    let enemy = put(&mut game, "LC20", 2, 2, false);
    let distant = put(&mut game, "LC20", 1, 3, false);
    let hidden = put(&mut game, "LC20", 0, 2, true);
    let base = catalog::card("LC20").defense.unwrap();
    assert_eq!(game.defense(&friend, 2), base + 1, "友方包括同队另一玩家");
    assert_eq!(game.defense(&own, 2), base + 1);
    assert_eq!(game.defense(&enemy, 2), base);
    assert_eq!(game.defense(&distant, 3), base);
    assert_eq!(game.defense(&hidden, 2), base);
    assert_eq!(
        game.defense(&source, 2),
        catalog::card("JC059").defense.unwrap()
    );
}
