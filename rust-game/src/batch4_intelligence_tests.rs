//! Synthetic layouts, then real legal actions/payments; this is not whole-card acceptance.
use crate::{catalog, model::*, rules::*, unit_support::*};

fn center(g: &mut Game, seat: usize) -> String {
    g.begin_window(Window::Action(g.team(seat)));
    fund(g, seat, "JC063", 2);
    let hand = held(g, "WM044", seat);
    let a = g.legal_actions(seat).into_iter().find(|a| a.action.kind == "play"
        && a.action.card_id.as_deref() == Some(&hand)
        && a.action.target_id.as_deref() == Some(&format!("society:p{seat}"))).expect("legal society attachment").action;
    let before = g.resources(seat);
    act(g, seat, a); top(g);
    assert_eq!(g.resources(seat), before - 1);
    g.attachments.iter().find(|a| a.card.definition == "WM044" && a.card.controller == seat).unwrap().card.id.clone()
}
fn conceal(g: &mut Game, seat: usize, definition: &str, region: usize) -> String {
    g.begin_window(Window::Action(g.team(seat)));
    fund(g, seat, "JC125", 2);
    g.players[seat].conceal_used = false;
    let h = held(g, definition, seat);
    let a = g.legal_actions(seat).into_iter().find(|a| a.action.kind == "conceal"
        && a.action.card_id.as_deref() == Some(&h) && a.action.region == Some(region))
        .or_else(|| g.legal_actions(seat).into_iter().find(|a| a.action.kind == "conceal" && a.action.card_id.as_deref()==Some(&h))).unwrap().action;
    let region=a.region.unwrap();
    act(g, seat, a);
    g.regions[region].cards.last().unwrap().id.clone()
}
#[test]
fn wm044_hidden_slow_all_four_seats_paid_entry_and_privacy() {
    let mut g = game(); center(&mut g, 0);
    for seat in 0..4 {
        let id = conceal(&mut g, seat, "JC125", 0);
        let c = g.board(&id).unwrap().1;
        assert!(c.face_down && c.exhausted);
        for viewer in 0..4 {
            let view = g.card_view(c, viewer, Some(0), None);
            assert!(view.exhausted);
            if viewer != seat { assert_eq!(view.card_id, None); }
        }
    }
    restore(&mut g);
}
#[test]
fn wm044_hidden_slow_not_retroactive_and_removal_stops_future_entries() {
    let mut g = game();
    let old = conceal(&mut g, 0, "JC125", 0);
    assert!(!g.board(&old).unwrap().1.exhausted);
    let source = center(&mut g, 2);
    assert!(!g.board(&old).unwrap().1.exhausted);
    let now = conceal(&mut g, 1, "JC125", 0);
    assert!(g.board(&now).unwrap().1.exhausted);
    g.remove_dead(&source, RemovalCause::Destroy); g.drive().unwrap(); restore(&mut g);
    let later = conceal(&mut g, 3, "JC125", 0);
    assert!(!g.board(&later).unwrap().1.exhausted);
    assert!(g.board(&now).unwrap().1.exhausted);
}
#[test]
fn wm044_hidden_slow_live_source_only_not_asset_or_grave() {
    for zone in ["hand", "asset", "grave"] {
        let mut g = game(); let c = g.make_card("WM044", 0);
        match zone { "hand" => g.players[0].hand.push(c), "asset" => g.players[0].assets.push(c), _ => g.players[0].graveyard.push(c) }
        let id = conceal(&mut g, 2, "JZ49", 0);
        assert!(!g.board(&id).unwrap().1.exhausted, "{zone}");
        assert!(g.pending.is_none()); // Hidden printed slow/entry text never fires.
    }
}
#[test]
fn wm044_hidden_slow_face_up_and_reveal_do_not_gain_it() {
    let mut g = game(); center(&mut g, 0);
    let id = conceal(&mut g, 0, "JC125", 0);
    g.board_mut(&id).unwrap().exhausted = false; // explicit ready layout
    g.world_reveal(0, &id, false).unwrap(); g.drive().unwrap();
    let c = g.regions[0].cards.last().unwrap();
    assert!(!c.face_down && !c.exhausted);
    restore(&mut g);
    let id = held(&mut g, "JC125", 0); g.begin_window(Window::Action(0));
    let a = g.legal_actions(0).into_iter().find(|a| a.action.kind == "deploy" && a.action.card_id.as_deref() == Some(&id) && a.action.region == Some(0)).unwrap().action;
    act(&mut g, 0, a); top(&mut g);
    assert!(!g.regions[0].cards.last().unwrap().exhausted);
}
#[test]
fn wm044_hidden_slow_hidden_grave_entry_uses_resolution_and_no_printed_trigger() {
    let mut g = game(); center(&mut g, 0); g.begin_window(Window::Action(0));
    fund(&mut g, 0, "JC084", 2);
    let c = g.make_card("JZ49", 0); let id = c.id.clone(); g.players[0].graveyard.push(c);
    let spell = held(&mut g, "JC092", 0);
    let a = g.legal_actions(0).into_iter().find(|a| a.action.kind == "play" && a.action.card_id.as_deref()==Some(&spell)
        && a.action.target_id.as_deref()==Some(&id) && a.action.region==Some(0)).unwrap().action;
    act(&mut g, 0, a); top(&mut g);
    let c = g.regions[0].cards.last().unwrap(); assert!(c.face_down && c.exhausted);
    assert!(g.pending.is_none()); restore(&mut g);
}
#[test]
fn wm044_hidden_slow_movement_and_hide_preserve_orientation() {
    let mut g = game(); center(&mut g, 0);
    let id = conceal(&mut g, 0, "JC125", 0);
    g.board_mut(&id).unwrap().exhausted = false;
    g.move_on_board_instance(&id, 1);
    assert!(!g.board(&id).unwrap().1.exhausted);
    let face = field(&mut g, "JC125", 0);
    let hidden = g.hide_in_place_successor(&face).unwrap();
    assert!(!hidden.exhausted && hidden.face_down);
    restore(&mut g);
}
#[test]
fn wm044_hidden_slow_multiple_exhausted_sources_and_new_source_identity() {
    let mut g = game(); let first = center(&mut g, 0); let second = center(&mut g, 1);
    for a in &mut g.attachments { a.card.exhausted = true; }
    g.remove_dead(&first, RemovalCause::Destroy); g.drive().unwrap();
    let id = conceal(&mut g, 3, "JC125", 0); assert!(g.board(&id).unwrap().1.exhausted);
    g.remove_dead(&second, RemovalCause::Destroy); g.drive().unwrap();
    let id = conceal(&mut g, 2, "JC125", 0); assert!(!g.board(&id).unwrap().1.exhausted);
}
#[test]
fn wm044_hidden_slow_partial_definition_does_not_claim_whole_card_or_admission() {
    let c = catalog::card("WM044"); assert_eq!(c.cost,1); assert_eq!(c.loyalty,["灰色","灰色"]);
    assert!(!c.supported); assert!(catalog::runtime_implemented(c));
    let d = definition("WM044"); assert_eq!(d.abilities.len(),1);
    assert_eq!(d.abilities[0].key,"attach");
    assert!(d.abilities.iter().all(|a|a.event.is_none()));
    let draft = crate::deck::DeckDraft { id:"wm044-wip".into(),name:"not admitted".into(),description:String::new(),society_id:None,
        cards:vec![catalog::DeckEntry{card_id:"JC125".into(),count:49},catalog::DeckEntry{card_id:"WM044".into(),count:1}],
        rules_version:catalog::RULES_VERSION.into(),card_pool_version:catalog::POOL_VERSION.into(),engine_version:catalog::ENGINE_VERSION.into(),updated_at:"2026-10-23".into() };
    assert!(crate::deck::validate(draft).is_err());
}
#[test]
fn wm044_hidden_slow_normal_next_turn_readies_without_reentry() {
    let mut g=game(); center(&mut g,0); let id=conceal(&mut g,0,"JC125",0);
    assert!(g.board(&id).unwrap().1.exhausted);
    g.effects.push_back(Effect::NextTurn); g.drive().unwrap();
    assert!(!g.board(&id).unwrap().1.exhausted); restore(&mut g);
}
#[test]
fn wm044_hidden_slow_jc032_paid_search_and_bq104_entry_search() {
    for source in ["JC032","BQ104"] {
        let mut g=game(); center(&mut g,0); g.begin_window(Window::Action(0));
        g.players[0].deck.clear();
        let searched=if source=="JC032" {"JC031"} else {"BQ104"};
        let c=g.make_card(searched,0);let selected=c.id.clone();g.players[0].deck.push(c);
        let id=field(&mut g,source,0);
        if source=="JC032" {
            fund(&mut g,0,"JC125",2);
            act(&mut g,0,Action{card_id:Some(id),ability_id:Some("top-six-vampire-hidden".into()),..Action::new("activate")});
            crate::jc029_tests::pass_top(&mut g);
        } else {
            g.enter_triggers(0,source,&id,false);g.drive().unwrap();
            choose(&mut g,vec!["accept".into()]);crate::jc029_tests::pass_top(&mut g);
        }
        choose(&mut g,vec![selected]);
        let c=g.regions[0].cards.iter().find(|c|c.definition==searched && c.face_down).unwrap();
        assert!(c.face_down && c.exhausted,"{source}");assert!(g.pending.is_none());restore(&mut g);
    }
}
#[test]
fn wm044_hidden_slow_xg27_paid_death_reentry() {
    let mut g=game();center(&mut g,0);g.begin_window(Window::Action(0));
    let spirit=g.make_card("XG27",0);g.players[0].graveyard.push(spirit);
    fund(&mut g,0,"JC084",1);
    let victim=field(&mut g,"JC125",2);
    // Production death primitive from an explicit layout, followed by actual
    // optional declaration, fee, response window and hidden-entry resolution.
    g.remove_dead(&victim,RemovalCause::Destroy);g.drive().unwrap();
    choose(&mut g,vec!["accept".into()]);top(&mut g);
    let c=g.regions[0].cards.iter().find(|c|c.definition=="XG27").unwrap();
    assert!(c.face_down && c.exhausted);restore(&mut g);
}
#[test]
fn wm044_hidden_slow_reads_live_source_after_response_before_grave_entry() {
    let mut g=game();let source=center(&mut g,0);g.begin_window(Window::Action(0));
    fund(&mut g,0,"JC084",2);
    let c=g.make_card("JC125",0);let target=c.id.clone();g.players[0].graveyard.push(c);
    let spell=held(&mut g,"JC092",0);
    let a=g.legal_actions(0).into_iter().find(|a|a.action.kind=="play" && a.action.card_id.as_deref()==Some(&spell)
        && a.action.target_id.as_deref()==Some(&target) && a.action.region==Some(0)).unwrap().action;
    act(&mut g,0,a);
    // Explicit responding-removal primitive; not a claim of a second paid spell.
    g.remove_dead(&source,RemovalCause::Destroy);g.drive().unwrap();restore(&mut g);
    top(&mut g);
    let c=g.regions[0].cards.last().unwrap();assert!(c.face_down && !c.exhausted);
    restore(&mut g);
}
