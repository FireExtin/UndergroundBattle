//! Foundation tests, not Game/Room admission or end-to-end card evidence.
use crate::{bq095::{Destination as D, Headquarters}, model::Card, unit_support::game};

fn source(g: &mut crate::model::Game, owner: usize, controller: usize) -> Card {
    // BQ095 intentionally has no live catalog/dispatch entry at this stage.
    let mut c = g.make_card("BQ091", owner);
    c.definition = "BQ095".into();
    c.controller = controller;
    c
}

#[test]
fn bq095_first_entry_moves_all_cards_with_existing_zone_reset_and_fresh_identity() {
    let mut g = game();
    let source = source(&mut g, 0, 0);
    g.players[0].graveyard = vec![g.make_card("JC125", 0), g.make_card("JC001", 0)];
    let before = g.players[0].graveyard.clone();
    let mut expected = g.clone();
    let expected_zone = expected.id();
    let expected_cards = before.iter().cloned().map(|c| expected.reset_zone_card(c)).collect::<Vec<_>>();
    let hq = Headquarters::first_entry(&mut g, &source, "society:p0").unwrap();
    assert!(g.players[0].graveyard.is_empty());
    assert_eq!(hq.id, expected_zone);
    assert_eq!(serde_json::to_value(&hq.cards).unwrap(), serde_json::to_value(expected_cards).unwrap());
    assert!(hq.cards.iter().all(|c| !before.iter().any(|old| old.id == c.id)));
    assert_eq!(hq.cards.iter().map(|c| c.definition.as_str()).collect::<Vec<_>>(), ["JC125", "JC001"]);
    hq.validate().unwrap();
}

#[test]
fn bq095_owned_text_replacement_does_not_follow_controller_or_enemy_text() {
    let mut g = game();
    let entry = source(&mut g, 0, 0);
    let hq = Headquarters::first_entry(&mut g, &entry, "society:p0").unwrap();
    let mut ours = g.make_card("CQ09", 0);
    ours.controller = 2;
    assert_eq!(hq.own_printed_reference(&ours), D::Headquarters(hq.id.clone()));
    let mut foreign = g.make_card("CQ09", 2);
    foreign.controller = 0;
    assert_eq!(hq.own_printed_reference(&foreign), D::Graveyard(0));
    assert_eq!(Headquarters::ordinary_grave_reference(0), D::Graveyard(0));
    assert_eq!(Headquarters::ordinary_grave_reference(2), D::Graveyard(2));
}

#[test]
fn bq095_destination_replacement_is_distinct_from_reference_replacement() {
    let mut g = game();
    let entry = source(&mut g, 2, 0);
    let hq = Headquarters::first_entry(&mut g, &entry, "society:p0").unwrap();
    // The entering attachment's owner does not change which society it serves.
    assert_eq!(hq.seat, 0);
    assert_eq!(hq.incoming_destination(0), D::Headquarters(hq.id.clone()));
    assert_eq!(hq.incoming_destination(2), D::Graveyard(2));
}

#[test]
fn bq095_rejects_wrong_source_host_and_duplicate_migration_without_mutation() {
    let mut g = game();
    let entry = source(&mut g, 0, 0);
    let card = g.make_card("JC125", 0);
    for case in 0..5 {
        let mut source = entry.clone();
        let mut grave = vec![card.clone()];
        let mut host = "society:p0";
        match case {
            0 => source.definition = "BQ091".into(),
            1 => source.face_down = true,
            2 => host = "society:p2",
            3 => grave.push(card.clone()),
            _ => grave[0].owner = 2,
        }
        g.players[0].graveyard = grave;
        let before = serde_json::to_value(&g).unwrap();
        assert!(Headquarters::first_entry(&mut g, &source, host).is_err());
        assert_eq!(serde_json::to_value(&g).unwrap(), before);
    }
}

#[test]
fn bq095_zone_round_trip_retains_modifier_after_original_attachment_leaves() {
    let mut g = game();
    let entry = source(&mut g, 0, 0);
    g.players[0].graveyard = vec![g.make_card("JC125", 0)];
    let hq = Headquarters::first_entry(&mut g, &entry, "society:p0").unwrap();
    let bytes = serde_json::to_string(&hq).unwrap();
    let restored: Headquarters = serde_json::from_str(&bytes).unwrap();
    restored.validate().unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(), bytes);
    assert_eq!(restored.incoming_destination(0), D::Headquarters(hq.id));
    let mut bad = restored.clone();
    bad.cards.push(bad.cards[0].clone());
    assert!(bad.validate().is_err());
    let mut extra = serde_json::to_value(restored).unwrap();
    extra["graveyardAlias"] = true.into();
    assert!(serde_json::from_value::<Headquarters>(extra).is_err());
}

#[test]
fn bq095_real_container_receive_and_take_preserve_one_physical_instance() {
    let mut g = game();
    let entry = source(&mut g, 0, 0);
    let mut hq = Headquarters::first_entry(&mut g, &entry, "society:p0").unwrap();
    let card = g.make_card("JC125", 0);
    let before = serde_json::to_value(&card).unwrap();
    hq.receive(card.clone()).unwrap();
    assert!(hq.receive(card.clone()).is_err());
    assert!(hq.receive(g.make_card("JC125", 2)).is_err());
    let foreign_source = g.make_card("CQ09", 2);
    assert!(hq.take_for_own_printed_reference(&foreign_source, &card.id).is_err());
    assert_eq!(hq.cards.len(), 1);
    let own_source = g.make_card("CQ09", 0);
    let taken = hq.take_for_own_printed_reference(&own_source, &card.id).unwrap();
    assert_eq!(serde_json::to_value(taken).unwrap(), before);
    assert!(hq.cards.is_empty());
    assert!(hq.take_for_own_printed_reference(&own_source, &card.id).is_err());
}
