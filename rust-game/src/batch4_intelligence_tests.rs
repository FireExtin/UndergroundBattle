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
    act(g, seat, a); crate::jc029_tests::pass_top(g);
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
    let d = definition("WM044"); assert_eq!(d.abilities.len(),3);
    assert_eq!(d.abilities[0].key,"attach");
    assert_eq!(d.abilities[1].event,Some(Event::MysticContestWon));
    assert_eq!(d.abilities[2].event,Some(Event::Enter));
    assert!(catalog::catalog().societies.iter().any(|s|s.card.id=="MSJC14"));
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

fn wm044_contest(g: &mut Game, winner: Option<usize>) {
    use crate::mystic_contest as m;
    g.begin_window(Window::Action(0));
    for seat in 0..4 {
        g.players[seat].hand.clear();g.players[seat].deck.clear();
        let def=if winner==Some(seat){"JZ49"}else{"JC125"};
        held(g,def,seat);
        for _ in 0..3 {let c=g.make_card("JC125",seat);g.players[seat].deck.push(c);}
    }
    fund(g,0,"WM022",3);let source=field(g,"WM022",0);g.board_mut(&source).unwrap().face_down=true;
    act(g,0,Action{card_id:Some(source),..Action::new("reveal")});
    for _ in 0..32 { if g.pending.is_some(){break;} pass(g); }
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,m::COMMIT_KIND);
    for seat in 0..4 {
        let id=g.players[seat].hand.iter().find(|c|c.definition==if winner==Some(seat){"JZ49"}else{"JC125"}).unwrap().id.clone();
        assert_eq!(g.pending.as_ref().unwrap().seat,seat);choose(g,vec![id]);
    }
    for _ in 0..4 {
        if g.pending.as_ref().is_some_and(|p|p.choice.kind==m::REWARD_KIND){choose(g,vec![]);}else{break;}
    }
}
fn wm044_reach_trigger(g: &mut Game) -> bool {
    for _ in 0..64 {
        if let Some(p)=&g.pending {
            if matches!(&p.resolution,ChoiceResolution::Declare{declaration:d,..} if d.ability.key==crate::batch4_intelligence::INSPECT){return true;}
            if p.choice.kind=="batch3-entry-order" {let id=p.choice.options[0].id.clone();choose(g,vec![id]);}
            else {assert_eq!(p.choice.min,Some(0));choose(g,vec![]);}
        } else if !g.stack.is_empty(){pass(g);}else{return false;}
    }
    panic!("bounded WM044 trigger scheduling")
}
fn wm044_target(g: &mut Game, target: &str) {
    assert!(wm044_reach_trigger(g));
    assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id==target));
    choose(g,vec![target.into()]);
}
fn hidden(g: &mut Game, def: &str, seat: usize) -> String {
    let id=field(g,def,seat);g.board_mut(&id).unwrap().face_down=true;id
}
#[test]
fn wm044_trigger_actual_win_hidden_inspection_only_original_player_sees_front() {
    let mut g=game();center(&mut g,0);let target=hidden(&mut g,"JC031",2);
    wm044_contest(&mut g,Some(0));wm044_target(&mut g,&target);
    crate::jc029_tests::pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"wm044-private-inspection");
    for seat in 0..4 {
        let view=g.view(seat);
        if seat==0 {assert_eq!(view.pending_choice.as_ref().unwrap().preview_cards[0].card_id.as_deref(),Some("JC031"));}
        else {assert!(view.pending_choice.is_none());assert!(!serde_json::to_string(&view.waiting_choice).unwrap().contains("JC031"));}
        let board=view.regions[0].characters.iter().find(|c|c.instance_id==target).unwrap();
        assert!(board.face_down);if seat!=2 {assert!(board.card_id.is_none());}
    }
    choose(&mut g,vec![]);assert!(g.board(&target).unwrap().1.face_down);
    assert_eq!(g.board(&target).unwrap().1.lock_markers,0);restore(&mut g);
}
#[test]
fn wm044_trigger_secret_plan_real_play_private_inspection_and_no_public_reveal() {
    let mut g=game();center(&mut g,0);g.begin_window(Window::Action(1));
    fund(&mut g,2,"WM046",3);g.add_society_markers(2,0,1);
    let id=held(&mut g,"WM046",2);
    act(&mut g,2,Action{card_id:Some(id),target_id:Some("society:p2".into()),option:Some("secret-plan".into()),..Action::new("play")});top(&mut g);
    let target=g.attachments.iter().find(|a|a.card.definition=="WM046").unwrap().card.id.clone();
    wm044_contest(&mut g,Some(0));wm044_target(&mut g,&target);crate::jc029_tests::pass_top(&mut g);
    let view=g.view(0);assert_eq!(view.pending_choice.unwrap().preview_cards[0].card_id.as_deref(),Some("WM046"));
    for seat in [1,3] {
        let v=g.view(seat);let a=v.attachments.iter().find(|a|a.card.instance_id==target).unwrap();
        assert!(a.card.card_id.is_none() && a.card.face_down);assert_eq!(a.card.name,"秘密计划");
        assert!(v.pending_choice.is_none());
    }
    choose(&mut g,vec![]);assert!(g.in_play(&target).unwrap().1.face_down);restore(&mut g);
}
#[test]
fn wm044_trigger_decline_tie_loss_and_missing_target() {
    for winner in [None,Some(2),Some(0)] {
        let mut g=game();center(&mut g,0);let target=hidden(&mut g,"JC125",2);
        wm044_contest(&mut g,winner);
        assert_eq!(wm044_reach_trigger(&mut g),winner==Some(0));
        if winner==Some(0){choose(&mut g,vec![]);assert!(g.pending.is_none());}
        assert!(g.board(&target).unwrap().1.face_down);restore(&mut g);
    }
    let mut g=game();center(&mut g,0);wm044_contest(&mut g,Some(0));assert!(!wm044_reach_trigger(&mut g));
}
#[test]
fn wm044_trigger_source_departure_does_not_cancel_declared_inspection() {
    let mut g=game();let source=center(&mut g,0);let target=hidden(&mut g,"JC125",2);
    wm044_contest(&mut g,Some(0));wm044_target(&mut g,&target);
    g.remove_dead(&source,RemovalCause::Destroy);g.drive().unwrap();restore(&mut g);
    crate::jc029_tests::pass_top(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"wm044-private-inspection");
    choose(&mut g,vec![]);restore(&mut g);
}
#[test]
fn wm044_trigger_target_departure_cancels_without_rebinding() {
    let mut g=game();center(&mut g,0);let target=hidden(&mut g,"JC031",2);
    wm044_contest(&mut g,Some(0));wm044_target(&mut g,&target);
    g.return_hand(&target);g.drive().unwrap();let replacement=hidden(&mut g,"JC031",2);restore(&mut g);
    crate::jc029_tests::pass_top(&mut g);
    assert!(g.pending.is_none());assert!(g.board(&replacement).unwrap().1.face_down);restore(&mut g);
}
#[test]
fn wm044_trigger_current_controller_each_actual_win_not_only_first() {
    let mut g=game();center(&mut g,1);let target=hidden(&mut g,"JC125",2);
    for _ in 0..2 {
        wm044_contest(&mut g,Some(0));assert!(wm044_reach_trigger(&mut g));
        assert_eq!(g.pending.as_ref().unwrap().seat,1);
        choose(&mut g,vec![target.clone()]);crate::jc029_tests::pass_top(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().seat,1);choose(&mut g,vec![]);
        wm044_reach_trigger(&mut g);restore(&mut g);
    }
    assert_eq!(g.mystic_history.len(),2);
}
#[test]
fn wm044_trigger_inspecting_jz26_queues_its_real_hidden_observation() {
    let mut g=game();center(&mut g,0);let target=hidden(&mut g,"JZ26",2);fund(&mut g,2,"JC031",2);
    wm044_contest(&mut g,Some(0));wm044_target(&mut g,&target);crate::jc029_tests::pass_top(&mut g);
    assert!(g.effects.iter().any(|e|matches!(e,Effect::Declare{declaration:d} if d.source.card.definition=="JZ26")));
    choose(&mut g,vec![]);restore(&mut g);
}
#[test]
fn wm044_trigger_sentinel_dependency_is_exact_and_not_substituted_by_gray_society() {
    let mut g=game();g.players[0].society_zone.card=Some(g.make_card("MSJC05",0));
    assert!(!g.wm044_has_sentinel(0));let target=hidden(&mut g,"JC125",2);
    center(&mut g,0);assert_eq!(g.board(&target).unwrap().1.lock_markers,0);assert!(g.pending.is_none());
    assert!(catalog::catalog().societies.iter().any(|s|s.card.id=="MSJC14"));restore(&mut g);
}
#[test]
fn wm044_trigger_two_centers_each_offer_once_for_same_actual_win() {
    let mut g=game();let first=center(&mut g,0);center(&mut g,0);
    assert_eq!(g.attachments.iter().filter(|a|a.card.definition=="WM044").count(),2);
    let target=hidden(&mut g,"JC125",2);
    for _ in 0..2 {
        wm044_contest(&mut g,Some(0));
        assert_eq!(g.pending.as_ref().unwrap().choice.kind,"batch3-entry-order");
        let mut sources=std::collections::BTreeSet::new();
        for _ in 0..2 {
            assert!(wm044_reach_trigger(&mut g));
            let ChoiceResolution::Declare{declaration:d,..}=&g.pending.as_ref().unwrap().resolution else{panic!()};
            assert!(sources.insert(d.source.card.id.clone()));
            choose(&mut g,vec![target.clone()]);
        }
        for _ in 0..2 {
            crate::jc029_tests::pass_top(&mut g);
            assert_eq!(g.pending.as_ref().unwrap().choice.kind,"wm044-private-inspection");
            choose(&mut g,vec![]);
        }
        assert!(sources.contains(&first));assert!(!wm044_reach_trigger(&mut g));restore(&mut g);
    }
}

#[test]
fn wm044_entry_lock_from_legally_selected_sentinel() {
    let mut g=crate::sentinel_tests::sentinel_game();let target=hidden(&mut g,"JC125",1);
    assert!(g.wm044_has_sentinel(0));center(&mut g,0);
    assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id==target));
    choose(&mut g,vec![target.clone()]);top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.lock_markers,1);restore(&mut g);
}

#[test]
fn wm044_sentinel_entry_decline_and_response_target_loss() {
    for decline in [false,true] {
        let mut g=crate::sentinel_tests::sentinel_game();let target=hidden(&mut g,"JC125",1);center(&mut g,0);
        if decline { choose(&mut g,vec![]);assert_eq!(g.board(&target).unwrap().1.lock_markers,0); }
        else { choose(&mut g,vec![target.clone()]);g.remove_dead(&target,RemovalCause::Destroy);top(&mut g);assert!(g.board(&target).is_none()); }
        restore(&mut g);
    }
}

// Independent normal-command response checks; explicit layouts are not full matches.
#[test]
fn independent_wm044_paid_response_destruction_changes_later_hidden_entry() {
    let mut g=game(); let source=center(&mut g,0); g.begin_window(Window::Action(0));
    fund(&mut g,0,"JC084",2); fund(&mut g,2,"JC005",2);
    let dead=g.make_card("JC125",0); let target=dead.id.clone(); g.players[0].graveyard.push(dead);
    let summon=held(&mut g,"JC092",0); let destroy=held(&mut g,"JC005",2);
    let a=g.legal_actions(0).into_iter().find(|a|a.action.kind=="play" && a.action.card_id.as_deref()==Some(&summon)
        && a.action.target_id.as_deref()==Some(&target) && a.action.region==Some(0)).unwrap().action;
    act(&mut g,0,a);
    // A declaration retains priority; pass the acting team before its opponent responds.
    for _ in 0..4 {if g.can_fast(2){break;} pass(&mut g);}
    let a=g.legal_actions(2).into_iter().find(|a|a.action.kind=="play" && a.action.card_id.as_deref()==Some(&destroy)
        && a.action.target_id.as_deref()==Some(&source)).expect("paid fast destruction with mind assets").action;
    act(&mut g,2,a); top(&mut g); assert!(g.in_play(&source).is_none()); assert_eq!(g.resources(2),0);
    top(&mut g); let c=g.regions[0].cards.last().unwrap(); assert_eq!(c.definition,"JC125"); assert!(c.face_down && !c.exhausted);
}

#[test]
fn independent_wm044_real_target_reveal_cancels_old_inspection() {
    let mut g=game(); center(&mut g,0); let target=hidden(&mut g,"JC125",2);
    wm044_contest(&mut g,Some(0)); wm044_target(&mut g,&target);
    for _ in 0..4 {if g.can_fast(2){break;} pass(&mut g);}
    let a=g.legal_actions(2).into_iter().find(|a|a.action.kind=="reveal" && a.action.card_id.as_deref()==Some(&target)).unwrap().action;
    act(&mut g,2,a); crate::jc029_tests::pass_top(&mut g);
    assert!(g.regions[0].cards.iter().any(|c|c.controller==2 && c.definition=="JC125" && !c.face_down));
    crate::jc029_tests::pass_top(&mut g); assert!(g.pending.is_none());
    assert!(g.stack.is_empty()); for s in 0..4 {assert!(g.view(s).pending_choice.is_none());}
}

#[test]
fn independent_wm044_mixed_contest_sources_order_and_private_completion() {
    let mut g=game(); center(&mut g,0); center(&mut g,0); field(&mut g,"BQ031",0);
    let target=hidden(&mut g,"JC125",2); wm044_contest(&mut g,Some(0));
    let p=g.pending.as_ref().unwrap(); assert_eq!(p.choice.kind,"batch3-entry-order");
    // The actual WM022 contest starter also observes the win.
    assert_eq!(p.choice.options.len(),4);
    assert_eq!(p.choice.options.iter().filter(|o|o.label.contains("情报分析中心")).count(),2);
    let feed=p.choice.options.iter().find(|o|o.label.contains("雾羽千军")).unwrap().id.clone();
    choose(&mut g,vec![feed]); assert_eq!(g.pending.as_ref().unwrap().choice.min,Some(0)); choose(&mut g,vec![]);
    let mut sources=std::collections::BTreeSet::new();
    for _ in 0..2 {
        assert!(wm044_reach_trigger(&mut g));
        let ChoiceResolution::Declare{declaration,..}=&g.pending.as_ref().unwrap().resolution else{panic!()};
        assert!(sources.insert(declaration.source.card.id.clone())); choose(&mut g,vec![target.clone()]);
    }
    for _ in 0..2 {
        crate::jc029_tests::pass_top(&mut g); assert_eq!(g.pending.as_ref().unwrap().choice.kind,"wm044-private-inspection");
        assert_eq!(g.view(0).pending_choice.unwrap().preview_cards[0].card_id.as_deref(),Some("JC125"));
        for s in [1,2,3] {assert!(g.view(s).pending_choice.is_none());}
        choose(&mut g,vec![]);
    }
    assert!(g.stack.is_empty() && g.pending.is_none() && g.entry_order_groups.is_empty());
    assert!(g.board(&target).unwrap().1.face_down);
}
