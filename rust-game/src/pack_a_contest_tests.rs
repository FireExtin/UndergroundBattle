//! Explicit native layouts, with real commands and persistence between steps.
//! No fixture represents a public-browser playtest or card-pool admission.
use crate::{catalog, model::*, rules::*, unit_support::*, pack_a_contest as a, mystic_contest as m};
use crate::jc029_tests::{checkpoint, pass_top};
fn deck(g: &mut Game, seat: usize, ids: &[&str]) {
    g.players[seat].deck.clear();
    for id in ids { let c = g.make_card(id,seat); g.players[seat].deck.push(c); }
}
fn layout() -> Game {
    let mut g=game();
    for seat in 0..4 { deck(&mut g,seat,&["JC125","JC125","JC125"]); }
    g
}

#[test]
fn independent_ab_ten_wm032_bonus_is_personal_and_or_not_sum() {
    for (destroyed, personal, teammate, expected) in [(false,false,true,3), (true,true,false,6), (true,false,true,6)] {
        let mut g=layout();
        if destroyed { g.regions[1].destroyed=true; }
        for seat in 0..2 {
            if (seat==0 && personal) || (seat==1 && teammate) {
                g.players[seat].destruction_markers=1;
                let won=g.make_card("DQJC115",seat); g.players[seat].score_cards.push(won);
            }
        }
        held(&mut g,"WM032",0);start(&mut g);commits(&mut g,["WM032","JC125","JC125","JC125"]);
        assert_eq!(g.mystic_history[0].shown[0].value,expected);
        rewards(&mut g);decline_triggers(&mut g);checkpoint(&g);
    }
}

#[test]
fn independent_ab_ten_wm021_hand_bat_and_manor_history_are_distinct() {
    for with_manor in [false,true] {
        let mut g=layout();field(&mut g,"WM021",0);fund(&mut g,0,"XQ14",2);
        let manor=if with_manor {
            fund(&mut g,0,"XQ15",2);let house=held(&mut g,"XQ15",0);
            act(&mut g,0,Action{card_id:Some(house),target_id:Some("society:p0".into()),..Action::new("play")});pass_top(&mut g);
            Some(g.attachments.iter().find(|a|a.card.definition=="XQ15").unwrap().card.id.clone())
        } else {None};
        let shown=if with_manor {"JC125"} else {"JC030"};held(&mut g,shown,0);
        start(&mut g);commits(&mut g,[shown,"JC125","JC125","JC125"]);
        assert_eq!(g.mystic_history[0].shown[0].value,if with_manor {2}else{3});
        rewards(&mut g);decline_triggers(&mut g);
        if let Some(id)=manor {g.remove_dead(&id,RemovalCause::Destroy);}
        checkpoint(&g);assert_eq!(g.mystic_history[0].shown[0].value,if with_manor {2}else{3});
    }
}
fn start(g: &mut Game) {
    fund(g,0,"WM022",3); let uid=field(g,"WM022",0);g.board_mut(&uid).unwrap().face_down=true;
    act(g,0,Action { card_id:Some(uid),..Action::new("reveal") });
    for _ in 0..24 { if g.pending.is_some() {break;} pass(g); }
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,m::COMMIT_KIND);
    checkpoint(g);
}
fn commits(g: &mut Game, ids: [&str;4]) {
    for (seat,id) in ids.iter().enumerate() {
        assert_eq!(g.pending.as_ref().unwrap().seat,seat);
        let uid=g.players[seat].hand.iter().find(|c|c.definition==*id).unwrap().id.clone();choose(g,vec![uid]);
    }
}
fn rewards(g: &mut Game) {
    for _ in 0..4 { if g.pending.as_ref().is_some_and(|p|p.choice.kind==m::REWARD_KIND) {choose(g,vec![]);} else {break;} }
}
fn decline_triggers(g: &mut Game) {
    for _ in 0..20 {
        if let Some(p)=g.pending.as_ref() {
            if p.choice.kind=="batch3-entry-order" {let id=p.choice.options[0].id.clone();choose(g,vec![id]);}
            else {assert_eq!(p.choice.min,Some(0));choose(g,vec![]);}
        } else if !g.stack.is_empty() {pass(g);} else {break;}
    }
    assert!(g.pending.is_none()&&g.stack.is_empty());checkpoint(g);
}
fn invalid(g: &Game, v: serde_json::Value) {
    let text=serde_json::to_string(&v).unwrap();
    let result=std::panic::catch_unwind(||Game::from_persisted(&text));assert!(result.is_ok()&&result.unwrap().is_err());
    let mut room=serde_json::to_value(crate::jc029_tests::envelope(g)).unwrap();room["game"]=v;
    let result=std::panic::catch_unwind(||crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()));assert!(result.is_ok()&&result.unwrap().is_err());
}
#[test]
fn pack_a_original_data_and_fixed_programs() {
    // Inventory is 22 complete native pilots plus XG21's explicitly closed-admission WIP.
    assert_eq!(crate::batch4::IMPLEMENTING.len(),23);
    assert!(crate::batch4::IMPLEMENTING.contains(&"XG21") && !catalog::card("XG21").supported);
    assert!(a::CARDS.iter().all(|id|!crate::batch4::IMPLEMENTING.contains(id)));
    for (id,cost,def) in [("WM021",1,Some(1)),("WM022",3,Some(1)),("WM032",3,Some(1)),("WM061",2,None),("BQ082",4,Some(1))] {
        let c=catalog::card(id);assert_eq!((c.cost,c.defense,c.supported),(cost,def,false));
        let mut defs=definitions().clone();defs.get_mut(id).unwrap().traits.public=true;assert!(validate_definitions(&defs).is_err());
    }
    assert_eq!(catalog::card("BQ082").subtitle.as_deref(),Some("首席药剂师"));
    assert_eq!(catalog::card("BQ082").loyalty,["黑色","黑色"]);
    assert!(!definition("BQ082").traits.public);
    for id in ["WM022","WM032","WM061","BQ082"] { for ability in &definition(id).abilities {assert!(validate_ability("JC125",ability).is_err());} }
    let mut g=layout();let id=held(&mut g,"WM061",0);reject(&mut g,0,Action{card_id:Some(id),..Action::new("asset")});
}
#[test]
fn pack_a_actual_reveal_paid_once_and_four_private_commitments() {
    let mut g=layout();let secret=held(&mut g,"WM021",0);start(&mut g);assert_eq!(g.resources(0),0);
    assert_eq!(g.players.iter().map(|p|p.hand.len()).collect::<Vec<_>>(),[2,1,1,1]);
    let first=g.pending.clone().unwrap();choose(&mut g,vec![secret.clone()]);
    for viewer in 1..4 {let v=g.view(viewer);assert!(v.revealed_hands.is_empty());assert!(!serde_json::to_string(&v).unwrap().contains(&secret));}
    for viewer in [0,2,3] {assert!(g.view(viewer).pending_choice.is_none());}
    reject(&mut g,0,Action{choice_id:Some(first.choice.id),selected:Some(vec![secret]),..Action::new("choose")});
    for seat in 1..4 {let id=g.players[seat].hand[0].id.clone();choose(&mut g,vec![id]);}
    assert_eq!(g.mystic_history[0].totals,[6,0]);assert_eq!(g.mystic_history[0].first_winners,[0,1]);
    for viewer in 0..4 {assert_eq!(g.view(viewer).revealed_hands.len(),4);}
    assert!(g.players[0].deck.iter().any(|c|c.definition=="WM021"));rewards(&mut g);decline_triggers(&mut g);assert_eq!(g.resources(0),0);
}
#[test]
fn pack_a_team_sum_tie_has_no_winners_or_rewards() {
    let mut g=layout();held(&mut g,"WM021",0);held(&mut g,"WM021",2);start(&mut g);
    commits(&mut g,["WM021","JC125","WM021","JC125"]);
    assert_eq!(g.mystic_history[0].totals,[6,6]);assert_eq!(g.mystic_history[0].winner_team,None);
    assert!(g.mystic_history[0].first_winners.is_empty());assert!(g.pending.is_none());assert!(g.mystic_contest.is_none());checkpoint(&g);
}
#[test]
fn pack_a_personal_vampire_bonus_excludes_teammate_and_hidden_observer() {
    let mut g=layout();field(&mut g,"WM021",0);field(&mut g,"WM021",0);field(&mut g,"WM021",1);
    let hidden=field(&mut g,"WM021",0);g.board_mut(&hidden).unwrap().face_down=true;
    held(&mut g,"WM021",0);held(&mut g,"WM022",1);start(&mut g);
    commits(&mut g,["WM021","WM022","JC125","JC125"]);
    let o=&g.mystic_history[0];assert_eq!(o.shown[0].value,10);assert_eq!(o.shown[1].value,5);assert_eq!(o.totals,[15,0]);rewards(&mut g);decline_triggers(&mut g);
}
#[test]
fn pack_a_marker_rewards_are_personal_opposite_and_cancel() {
    let mut g=layout();g.players[0].society_zone.exposure=1;g.players[3].society_zone.secrecy=1;
    held(&mut g,"WM021",0);start(&mut g);commits(&mut g,["WM021","JC125","JC125","JC125"]);
    assert_eq!(g.pending.as_ref().unwrap().seat,0);choose(&mut g,vec!["secrecy".into()]);assert_eq!(g.players[0].society_zone.exposure,0);
    assert_eq!(g.pending.as_ref().unwrap().seat,1);choose(&mut g,vec!["exposure".into()]);assert_eq!(g.players[3].society_zone.secrecy,0);
    assert_eq!(g.players[2].society_zone.exposure,0);decline_triggers(&mut g);
}
#[test]
fn pack_a_preacher_conditional_printed_bonus() {
    for condition in 0..3 {let mut g=layout();held(&mut g,"WM032",0);
        if condition==1 {g.regions[1].destroyed=true;} if condition==2 {g.players[0].destruction_markers=1;let won=g.make_card("DQJC115",0);g.players[0].score_cards.push(won);}
        start(&mut g);commits(&mut g,["WM032","JC125","JC125","JC125"]);
        assert_eq!(g.mystic_history[0].shown[0].value,if condition==0 {3}else{6});rewards(&mut g);decline_triggers(&mut g);
    }
}
#[test]
fn pack_a_earth_normal_action_mills_each_player_and_short_deck() {
    let mut g=layout();deck(&mut g,0,&["JC125";7]);deck(&mut g,1,&[]);fund(&mut g,0,"WM061",2);let id=held(&mut g,"WM061",0);
    act(&mut g,0,Action{card_id:Some(id),..Action::new("play")});assert_eq!(g.resources(0),0);pass_top(&mut g);
    assert_eq!(g.players.iter().map(|p|p.deck.len()).collect::<Vec<_>>(),[2,0,0,0]);
    assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JC125").count(),5);checkpoint(&g);
}
#[test]
fn pack_a_earth_shown_shuffles_before_post_mill() {
    let mut g=layout();for s in 0..4 {deck(&mut g,s,&[]);held(&mut g,"JC125",s);}held(&mut g,"WM061",0);
    start(&mut g);commits(&mut g,["WM061","JC125","JC125","JC125"]);assert_eq!(g.mystic_history[0].shown[0].value,4);
    rewards(&mut g);assert_eq!(g.players[0].deck.len(),0);assert!(g.players[0].graveyard.iter().any(|c|c.definition=="WM061"));decline_triggers(&mut g);
}
#[test]
fn pack_a_public_requires_personal_exposure() {
    for exposed in [false,true] {let mut g=layout();fund(&mut g,0,"BQ082",4);let id=held(&mut g,"BQ082",0);
        if exposed {g.players[0].society_zone.exposure=1;} else {g.players[1].society_zone.exposure=1;}
        let action=Action{card_id:Some(id),region:Some(0),..Action::new("conceal")};
        if exposed {reject(&mut g,0,action);} else {act(&mut g,0,action);assert!(g.regions[0].cards.iter().any(|c|c.definition=="BQ082"&&c.face_down));}
    }
}
#[test]
fn pack_a_private_pending_and_result_tampering_rejected_by_both_restore_envelopes() {
    let mut g=layout();held(&mut g,"WM021",0);start(&mut g);let base=serde_json::to_value(&g).unwrap();
    for i in 0..8 {let mut v=base.clone();match i {
        0=>v["pending"]["seat"]=1.into(),1=>v["pending"]["choice"]["min"]=0.into(),2=>v["mystic_contest"]["actor_bonus"]=99.into(),
        3=>v["mystic_contest"]["participants"][0]=3.into(),4=>v["pending"]["choice"]["options"][0]["id"]="foreign".into(),
        5=>v["pending"]["resolution"]["Frame"]["choice"]["MysticCommit"]["contest_id"]="i1".into(),
        6=>v["mystic_contest"]["phase"]["Selecting"]["committed"]=serde_json::json!([[2,"i1"]]),_=>v["mystic_contest"]["turn"]=99.into(),};invalid(&g,v);}
    commits(&mut g,["WM021","JC125","JC125","JC125"]);let base=serde_json::to_value(&g).unwrap();
    for i in 0..6 {let mut v=base.clone();match i {0=>v["mystic_history"][0]["totals"][0]=999.into(),1=>v["mystic_history"][0]["winner_team"]=1.into(),2=>v["mystic_history"][0]["first_winners"]=serde_json::json!([]),3=>v["mystic_history"][0]["shown"][0]["value"]=8.into(),4=>v["pending"]["choice"]["max"]=2.into(),_=>v["mystic_contest"]["phase"]["Rewards"]["awarded"]=serde_json::json!([[0,"Secrecy"]])};invalid(&g,v);}
}
fn corpse(g: &mut Game, id: &str, seat: usize) -> String {let c=g.make_card(id,seat);let uid=c.id.clone();g.players[seat].graveyard.push(c);uid}
fn poison(g: &mut Game, target: &str) {
    fund(g,0,"BQ082",4);let id=held(g,"BQ082",0);act(g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});pass_top(g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"trigger");choose(g,vec![target.into()]);
    assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().ability_key,a::POISON);
}
#[test]
fn pack_a_poison_actual_paid_entry_target_before_responses_bonus_and_exhausted_reentry() {
    let mut g=layout();let target=corpse(&mut g,"WM021",0);let ally=corpse(&mut g,"WM021",1);held(&mut g,"WM021",0);
    poison(&mut g,&target);assert_eq!(g.resources(0),0);let f=g.stack.last().unwrap().frame.as_ref().unwrap();assert_eq!(f.targets[0].id,target);assert_ne!(f.targets[0].id,ally);
    pass_top(&mut g);commits(&mut g,["WM021","JC125","JC125","JC125"]);assert_eq!(g.mystic_history[0].shown[0].value,8);assert_eq!(g.mystic_history[0].shown[1].value,0);
    rewards(&mut g);assert!(!g.players[0].graveyard.iter().any(|c|c.id==target));assert!(g.players[1].graveyard.iter().any(|c|c.id==ally));
    assert!(g.regions[0].cards.iter().any(|c|c.definition=="WM021"&&c.exhausted&&c.id!=target));checkpoint(&g);
}
#[test]
fn pack_a_poison_source_departure_keeps_paid_trigger_but_original_region_replacement_fizzles_entry() {
    for replace in [false,true] {let mut g=layout();let target=corpse(&mut g,"WM021",0);held(&mut g,"WM021",0);poison(&mut g,&target);
        let source=g.regions[0].cards.iter().find(|c|c.definition=="BQ082").unwrap().id.clone();g.remove_dead(&source,RemovalCause::Destroy);
        if replace {g.regions[0].card=g.make_card("DQJC115",0);}
        pass_top(&mut g);commits(&mut g,["WM021","JC125","JC125","JC125"]);rewards(&mut g);
        assert_eq!(g.players[0].graveyard.iter().any(|c|c.id==target),replace);assert_eq!(g.regions[0].cards.iter().any(|c|c.definition=="WM021"),!replace);checkpoint(&g);
    }
}
#[test]
fn pack_a_poison_departed_target_cancels_whole_trigger_without_starting_contest() {
    let mut g=layout();let target=corpse(&mut g,"WM021",0);poison(&mut g,&target);
    let c=g.players[0].graveyard.remove(0);let c=g.reset_zone_card(c);g.players[0].hand.push(c);
    pass_top(&mut g);assert!(g.mystic_history.is_empty());assert!(g.mystic_contest.is_none());assert!(g.pending.is_none());assert_eq!(g.resources(0),0);checkpoint(&g);
}
#[test]
fn pack_a_poison_cannot_target_teammate_or_spell_and_obeys_double_black_loyalty() {
    let mut g=layout();let ally=corpse(&mut g,"WM021",1);corpse(&mut g,"WM061",0);fund(&mut g,0,"JC125",3);fund(&mut g,0,"BQ082",1);let id=held(&mut g,"BQ082",0);
    reject(&mut g,0,Action{card_id:Some(id.clone()),region:Some(0),..Action::new("deploy")});fund(&mut g,0,"BQ082",1);
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});pass_top(&mut g);assert!(g.pending.is_none());assert!(g.mystic_history.is_empty());assert!(g.players[1].graveyard.iter().any(|c|c.id==ally));
}
#[test]
fn pack_a_shadow_first_win_personal_once_per_turn_and_private_target_discard() {
    let mut g=layout();held(&mut g,"WM021",0);let discard=held(&mut g,"JC125",2);start(&mut g);commits(&mut g,["WM021","JC125","JC125","JC125"]);rewards(&mut g);
    assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key==a::SHADOW_WIN));
    choose(&mut g,vec!["p2".into()]);pass_top(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"discard");assert_eq!(g.pending.as_ref().unwrap().seat,2);
    for s in [0,1,3] {assert!(g.view(s).pending_choice.is_none());}
    let uid=g.players[2].hand[0].id.clone();choose(&mut g,vec![uid]);assert_eq!(g.players[2].graveyard.len(),1);assert!(!g.players[2].graveyard.iter().any(|c|c.id==discard));
    held(&mut g,"WM021",0);start(&mut g);commits(&mut g,["WM021","JC125","JC125","JC125"]);rewards(&mut g);
    assert!(g.mystic_history[1].first_winners.is_empty());assert!(g.pending.is_none());assert!(g.stack.is_empty());
    g.effect(Effect::NextTurn).unwrap();assert!(g.mystic_history.is_empty());g.drive().unwrap();while g.pending.is_some(){choose(&mut g,vec![]);}g.effects.clear();g.begin_window(Window::Action(0));
    held(&mut g,"WM021",0);start(&mut g);commits(&mut g,["WM021","JC125","JC125","JC125"]);assert_eq!(g.mystic_history[0].first_winners,[0,1]);rewards(&mut g);decline_triggers(&mut g);
}
#[test]
fn pack_a_preacher_win_queues_multiple_sources_and_resolves_current_destroyed_regions() {
    let mut g=layout();let one=field(&mut g,"WM032",0);field(&mut g,"WM032",1);g.regions[0].destroyed=true;g.regions[1].destroyed=true;held(&mut g,"WM021",0);
    start(&mut g);commits(&mut g,["WM021","JC125","JC125","JC125"]);rewards(&mut g);
    // All observations are captured once. Controllers retain the existing ordering UI.
    for _ in 0..12 {
        let Some(p)=g.pending.as_ref() else {break;};
        if p.choice.kind=="batch3-entry-order" {let key=p.choice.options.iter().find(|o|o.id.contains(a::PREACHER_WIN)).unwrap_or(&p.choice.options[0]).id.clone();choose(&mut g,vec![key]);}
        else if matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key==a::PREACHER_WIN) {choose(&mut g,vec!["accept".into()]);}
        else {choose(&mut g,vec![]);}
    }
    assert_eq!(g.stack.len(),2);g.remove_dead(&one,RemovalCause::Destroy);g.regions[1].destroyed=false;
    while !g.stack.is_empty(){pass_top(&mut g);}assert_eq!(g.regions[0].influence[0],2);assert_eq!(g.regions[1].influence[0],0);checkpoint(&g);
}
#[test]
fn pack_a_empty_deck_with_hand_is_not_draw_elimination() {
    let mut g=layout();for s in 0..4 {deck(&mut g,s,&[]);held(&mut g,"JC125",s);}start(&mut g);commits(&mut g,["JC125";4]);
    assert!(g.players.iter().all(|p|!p.eliminated));assert_eq!(g.players.iter().map(|p|p.deck.len()).collect::<Vec<_>>(),[1,1,1,1]);checkpoint(&g);
}
#[test]
fn pack_a_earth_mill_pauses_for_rumor_and_resumes_each_player_once() {
    let mut g=layout();let host=field(&mut g,"JC125",0);deck(&mut g,0,&["WM042","JC125"]);fund(&mut g,0,"WM061",2);let id=held(&mut g,"WM061",0);
    act(&mut g,0,Action{card_id:Some(id),..Action::new("play")});pass_top(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"rumor");checkpoint(&g);
    let base=serde_json::to_value(&g).unwrap();let mut bad=base.clone();bad["pending"]["resolution"]["Rumor"]["batch"]["cause"]["steps"][1]["op"]["MoveDeckTopToGraveyard"]["count"]=9.into();invalid(&g,bad);
    choose(&mut g,vec![host]);assert!(g.sealed_cards.iter().any(|s|s.card.definition=="WM042"));assert_eq!(g.players.iter().map(|p|p.deck.len()).collect::<Vec<_>>(),[0,0,0,0]);assert_eq!(g.players[1].graveyard.len(),3);checkpoint(&g);
}
#[test]
fn pack_a_earth_shown_post_mill_has_own_original_cause_and_rumor_cursor() {
    let mut g=layout();let host=field(&mut g,"JC125",0);held(&mut g,"WM061",0);start(&mut g);commits(&mut g,["WM061","JC125","JC125","JC125"]);
    // Deterministic top layout after the real shuffle, before rewards finish.
    deck(&mut g,0,&["WM042","JC125"]);rewards(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"rumor");checkpoint(&g);
    let ChoiceResolution::Rumor {batch}= &g.pending.as_ref().unwrap().resolution else {unreachable!()};assert_eq!(batch.cause.as_ref().unwrap().source.card.definition,"WM061");
    let mut bad=serde_json::to_value(&g).unwrap();bad["pending"]["resolution"]["Rumor"]["batch"]["cause"]["actor"]=1.into();invalid(&g,bad);
    choose(&mut g,vec![host]);assert_eq!(g.players[0].deck.len(),1);assert_eq!(g.sealed_cards.len(),1);decline_triggers(&mut g);
}
#[test]
fn pack_a_commit_rejects_other_seat_extra_payload_duplicate_and_foreign_card() {
    let mut g=layout();start(&mut g);let p=g.pending.clone().unwrap();let own=g.players[0].hand[0].id.clone();let foreign=g.players[1].hand[0].id.clone();
    for selected in [vec![],vec![foreign],vec![own.clone(),own.clone()]] {reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(selected),..Action::new("choose")});}
    reject(&mut g,1,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![own.clone()]),..Action::new("choose")});
    reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec![own]),region:Some(9),..Action::new("choose")});
}
#[test]
fn pack_a_earth_restored_program_cannot_drop_reorder_duplicate_or_replace_player_slots() {
    let mut g=layout();fund(&mut g,0,"WM061",2);let id=held(&mut g,"WM061",0);act(&mut g,0,Action{card_id:Some(id),..Action::new("play")});
    let base=serde_json::to_value(&g).unwrap();
    for i in 0..5 {let mut v=base.clone();let steps=v["stack"][0]["frame"]["steps"].as_array_mut().unwrap();match i {
        0=>steps.truncate(1),1=>{steps.remove(2);},2=>steps.swap(1,2),3=>steps[1]["context"]=0.into(),_=>steps[3]["context"]=4.into(),};invalid(&g,v);}
}
#[test]
fn pack_a_earth_player_elimination_before_and_after_payment_keeps_fixed_slots_without_milling_dead_seat() {
    for before in [false,true] {let mut g=layout();if before {g.eliminate(1);}fund(&mut g,0,"WM061",2);let id=held(&mut g,"WM061",0);
        act(&mut g,0,Action{card_id:Some(id),..Action::new("play")});assert_eq!(g.stack[0].frame.as_ref().unwrap().steps.len(),4);
        if !before {g.eliminate(1);}pass_top(&mut g);assert!(g.players[1].graveyard.is_empty());assert_eq!(g.players[2].graveyard.len(),3);checkpoint(&g);
    }
}
#[test]
fn pack_a_lower_response_stack_does_not_block_contest_selection() {
    let mut g=layout();fund(&mut g,0,"WM061",2);let earth=held(&mut g,"WM061",0);act(&mut g,0,Action{card_id:Some(earth),..Action::new("play")});
    let lower=g.stack[0].id.clone();while g.priority_team!=0 {pass(&mut g);}
    start(&mut g);assert!(g.stack.iter().any(|s|s.id==lower));assert_eq!(g.mystic_contest.as_ref().unwrap().response_base,[lower.clone()]);
    commits(&mut g,["JC125";4]);assert!(g.mystic_contest.is_none());assert_eq!(g.stack.len(),1);assert_eq!(g.stack[0].id,lower);pass_top(&mut g);assert!(g.players.iter().all(|p|p.deck.is_empty()));checkpoint(&g);
}
#[test]
fn pack_a_start_response_elimination_keeps_receipt_then_excludes_dead_participant() {
    let mut g=layout();fund(&mut g,0,"WM022",3);let source=field(&mut g,"WM022",0);g.board_mut(&source).unwrap().face_down=true;
    act(&mut g,0,Action{card_id:Some(source),..Action::new("reveal")});
    for _ in 0..12 {if g.stack.last().is_some_and(|s|s.frame.as_ref().unwrap().ability_key==a::SHADOW_REVEAL) {break;}pass(&mut g);}
    let item=g.stack.pop().unwrap();assert_eq!(item.frame.as_ref().unwrap().ability_key,a::SHADOW_REVEAL);
    // Stop exactly at the existing interpreter's start-hook boundary.
    g.resolve_stack(item).unwrap();let Effect::Frame {frame}=g.effects.pop_front().unwrap() else {unreachable!()};g.resolve_frame(*frame).unwrap();assert!(matches!(g.mystic_contest.as_ref().unwrap().phase,m::MysticPhase::BeforeSelection));g.eliminate(1);checkpoint(&g);
    g.drive().unwrap();assert_eq!(g.mystic_contest.as_ref().unwrap().participants,[0,2,3]);checkpoint(&g);
    for seat in [0,2,3] {assert_eq!(g.pending.as_ref().unwrap().seat,seat);let id=g.players[seat].hand[0].id.clone();choose(&mut g,vec![id]);}
    assert_eq!(g.mystic_history[0].participants,[0,2,3]);assert!(g.pending.is_none());checkpoint(&g);
}
#[test]
fn pack_a_owner_manor_aura_changes_human_display_and_preserves_witness() {
    let mut g=layout();field(&mut g,"WM021",0);fund(&mut g,0,"XQ15",2);let house=held(&mut g,"XQ15",0);
    act(&mut g,0,Action{card_id:Some(house),target_id:Some("society:p0".into()),..Action::new("play")});pass_top(&mut g);
    start(&mut g);commits(&mut g,["JC125";4]);assert_eq!(g.mystic_history[0].shown[0].value,2);assert!(g.mystic_history[0].shown[0].manor.is_some());
    assert!(g.mystic_history[0].shown[1].manor.is_none());rewards(&mut g);decline_triggers(&mut g);
}
#[test]
fn pack_a_shared_ops_reject_nested_and_mode_transplants() {
    for op in [Op::MysticContest{actor_bonus:0},Op::MysticWin{contest_id:None},Op::MysticDestroyedInfluence] {
        for wrapper in 0..4 {
            let mut ability=definition("WM022").abilities[0].clone();ability.key="forged".into();ability.event=None;
            ability.ops=match wrapper {0=>vec![Op::ForEachLivingPlayer(vec![op.clone()])],1=>vec![Op::ForEachLivingPlayerFromActor(vec![op.clone()])],
                2=>vec![Op::IfTargetExhausted{slot:0,exhausted:Box::new(op.clone()),ready:Box::new(op.clone())}],_=>vec![]};
            if wrapper==3 {ability.modes.push(Mode{key:"foreign".into(),label:"foreign".into(),targets:vec![],ops:vec![op.clone()]});}
            assert!(validate_ability("JC125",&ability).is_err());let mut defs=definitions().clone();defs.get_mut("JC125").unwrap().abilities.push(ability);assert!(validate_definitions(&defs).is_err());
        }
    }
}
#[test]
fn pack_a_poison_conditional_public_is_seen_by_alpha_only_when_personally_exposed() {
    for exposed in [false,true] {let mut g=layout();field(&mut g,"WM041",0);fund(&mut g,0,"BQ082",4);let id=held(&mut g,"BQ082",0);
        if exposed {g.players[0].society_zone.exposure=1;}else{g.players[1].society_zone.exposure=1;}
        act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});
        assert_eq!(g.stack.iter().any(|s|s.frame.as_ref().is_some_and(|f|f.source.card.definition=="WM041")),exposed);
        checkpoint(&g);while !g.stack.is_empty(){pass_top(&mut g);}assert_eq!(g.players[0].society_zone.exposure,if exposed{2}else{0});checkpoint(&g);
    }
}
#[test]
fn pack_a_rng_above_js_safe_integer_and_room_revision_survive_each_choice() {
    let mut g=layout();g.seed=9_007_199_254_740_997;g.random=u64::MAX-117;held(&mut g,"WM021",0);start(&mut g);
    for seat in 0..4 {checkpoint(&g);let id=g.players[seat].hand[0].id.clone();choose(&mut g,vec![id]);checkpoint(&g);}
    rewards(&mut g);decline_triggers(&mut g);assert_eq!(g.seed,9_007_199_254_740_997);
}
#[test]
fn pack_a_finished_history_cannot_omit_still_living_loser_but_retains_later_eliminated_participant() {
    let mut g=layout();held(&mut g,"WM021",0);start(&mut g);commits(&mut g,["WM021","JC125","JC125","JC125"]);rewards(&mut g);decline_triggers(&mut g);assert!(g.mystic_contest.is_none());
    let mut v=serde_json::to_value(&g).unwrap();v["mystic_history"][0]["participants"].as_array_mut().unwrap().pop();v["mystic_history"][0]["shown"].as_array_mut().unwrap().pop();invalid(&g,v);
    g.eliminate(3);checkpoint(&g);assert!(g.mystic_history[0].participants.contains(&3));
}
#[test]
fn pack_a_recursive_foreign_saved_frames_rejected_in_response_and_effect_queue() {
    let mut g=layout();let id=held(&mut g,"JC125",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});let base=serde_json::to_value(&g).unwrap();
    for op in [Op::MysticContest{actor_bonus:0},Op::MysticWin{contest_id:None},Op::MysticDestroyedInfluence] {
        for wrapper in 0..4 {
            let wrapped=match wrapper {0=>op.clone(),1=>Op::ForEachLivingPlayer(vec![op.clone()]),2=>Op::ForEachLivingPlayerFromActor(vec![op.clone()]),_=>Op::IfTargetExhausted{slot:0,exhausted:Box::new(op.clone()),ready:Box::new(op.clone())}};
            let mut v=base.clone();v["stack"][0]["frame"]["steps"]=serde_json::to_value(vec![Step{context:0,op:wrapped}]).unwrap();invalid(&g,v.clone());
            let frame=v["stack"][0]["frame"].clone();v["stack"]=serde_json::json!([]);v["effects"]=serde_json::json!([{"Frame":{"frame":frame}}]);invalid(&g,v);
        }
    }
}
#[test]
fn pack_a_duel_uses_two_individual_values_and_opposite_enemy_reward() {
    let mut g=Game::new("duel".into(),"LOCAL".into(),"duel".into(),"P0".into(),"watchers".into(),17).unwrap();g.join("P1".into(),"watchers".into()).unwrap();for p in &mut g.players {p.ready=true;}g.apply(0,Action::new("start")).unwrap();
    while let Some(p)=g.pending.clone(){g.apply(p.seat,Action{choice_id:Some(p.choice.id),selected:Some(vec![]),..Action::new("choose")}).unwrap();}
    for p in &mut g.players{p.hand.clear();p.assets.clear();}for r in &mut g.regions{r.cards.clear();}g.regions[0].card=g.make_card("DQJC115",0);g.begin_window(Window::Action(0));for s in 0..2{deck(&mut g,s,&["JC125","JC125"]);}held(&mut g,"WM021",0);fund(&mut g,0,"WM022",3);let source=field(&mut g,"WM022",0);g.board_mut(&source).unwrap().face_down=true;
    g.apply(0,Action{card_id:Some(source),..Action::new("reveal")}).unwrap();
    for _ in 0..12{if g.pending.is_some(){break;}let seat=(0..2).find(|s|g.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();g.apply(seat,Action::new("pass")).unwrap();}
    for s in 0..2 {let p=g.pending.clone().unwrap();let id=g.players[s].hand[0].id.clone();g.apply(s,Action{choice_id:Some(p.choice.id),selected:Some(vec![id]),..Action::new("choose")}).unwrap();let state=serde_json::to_string(&g).unwrap();g=Game::from_persisted(&state).unwrap();let room=crate::jc029_tests::envelope(&g);assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_ok());}
    assert_eq!(g.mystic_history[0].totals,[6,0]);assert_eq!(g.mystic_history[0].first_winners,[0]);let p=g.pending.clone().unwrap();g.apply(0,Action{choice_id:Some(p.choice.id),selected:Some(vec!["exposure".into()]),..Action::new("choose")}).unwrap();assert_eq!(g.players[1].society_zone.exposure,1);
}
#[test]
fn pack_a_paid_response_cannot_be_moved_to_effect_queue_or_inject_transaction_attributes() {
    for id in ["WM021","WM022","WM032","BQ082","WM061"] {
        let mut g=layout();fund(&mut g,0,id,catalog::card(id).cost as usize);let uid=held(&mut g,id,0);
        let action=if id=="WM061" {Action{card_id:Some(uid),..Action::new("play")}}else{Action{card_id:Some(uid),region:Some(0),..Action::new("deploy")}};act(&mut g,0,action);
        let base=serde_json::to_value(&g).unwrap();let mut v=base.clone();let frame=v["stack"][0]["frame"].clone();v["stack"]=serde_json::json!([]);v["effects"]=serde_json::json!([{"Frame":{"frame":frame}}]);invalid(&g,v);
        for n in 0..5 {let mut v=base.clone();match n {0=>v["stack"][0]["card"]["time_markers"]=9.into(),1=>v["stack"][0]["card"]["damage"]=9.into(),2=>v["stack"][0]["card"]["id"]=v["stack"][0]["frame"]["source"]["card"]["id"].clone(),3=>v["stack"][0]["card"]["owner"]=1.into(),_=>v["stack"][0]["frame"]=serde_json::Value::Null};invalid(&g,v);}
    }
}

#[test]
fn pack_a_response_identity_cannot_reuse_retired_or_future_instance() {
    for id in ["WM021", "WM022", "WM032", "BQ082", "WM061"] {
      for retired_after_source in [true, false] {
        let mut g = layout();
        let early_source = retired_after_source.then(|| held(&mut g, id, 0));
        let retired = held(&mut g, "JC125", 0);
        g.discard_hand_card(0, &retired).unwrap();
        assert!(!g.batch3_physical_cards().any(|c| c.id == retired));
        fund(&mut g, 0, id, catalog::card(id).cost as usize);
        let uid = early_source.unwrap_or_else(|| held(&mut g, id, 0));
        let action = if id == "WM061" {
            Action { card_id: Some(uid), ..Action::new("play") }
        } else {
            Action { card_id: Some(uid), region: Some(0), ..Action::new("deploy") }
        };
        act(&mut g, 0, action);
        let future = g.id();
        checkpoint(&g);
        let base = serde_json::to_value(&g).unwrap();
        for (case, wrong) in [("retired shell", retired.clone()), ("future shell", future), ("retired frame", retired)] {
            let mut value = base.clone();
            if case == "retired frame" {
                value["stack"][0]["id"] = wrong.clone().into();
                value["stack"][0]["frame"]["frame_id"] = wrong.into();
            } else {
                value["stack"][0]["card"]["id"] = wrong.into();
            }
            assert!(Game::from_persisted(&serde_json::to_string(&value).unwrap()).is_err(), "{id} restored {case}; retired_after_source={retired_after_source}");
            invalid(&g, value);
        }
      }
    }
}

#[test]
fn ab_card_closure_all_ten_paid_runtime_entries_views_and_deck_gate() {
    use crate::deck::{DeckDraft,validate};
    use crate::catalog::DeckEntry;
    let control=DeckDraft{id:"ab-closure-control".into(),name:"AB closure".into(),description:String::new(),
        society_id:None,cards:vec![DeckEntry{card_id:"JC125".into(),count:50}],
        rules_version:catalog::RULES_VERSION.into(),card_pool_version:catalog::POOL_VERSION.into(),
        engine_version:catalog::ENGINE_VERSION.into(),updated_at:"2026-10-22".into()};
    assert!(validate(control.clone()).is_ok());
    for id in ["WM021","WM022","WM032","WM061","BQ082","BQ027","BQ031","BQ032","BQ036","BQ038"] {
        let d=catalog::card(id);assert!(catalog::runtime_implemented(d)&&!d.supported);
        assert!(definitions().contains_key(id));
        let mut blocked=control.clone();blocked.cards=vec![DeckEntry{card_id:"JC125".into(),count:49},DeckEntry{card_id:id.into(),count:1}];
        assert!(validate(blocked).is_err(),"native entry must not open construction for {id}");
        let mut g=layout();fund(&mut g,0,id,d.cost as usize);let hand=held(&mut g,id,0);
        let action=match d.kind.as_str(){
            "character"=>Action{card_id:Some(hand),region:Some(0),..Action::new("deploy")},
            "attachment"=>Action{card_id:Some(hand),target_id:Some("society:p0".into()),..Action::new("play")},
            "spell"=>Action{card_id:Some(hand),..Action::new("play")},_=>panic!("unexpected card kind")};
        act(&mut g,0,action);assert_eq!(g.resources(0),0,"printed payment for {id}");checkpoint(&g);
        pass_top(&mut g);checkpoint(&g);
        assert!(g.stack.is_empty()&&g.effects.is_empty()&&g.pending.is_none(),"entry lifecycle for {id}");
        let live=match d.kind.as_str(){
            "character"=>g.regions[0].cards.iter().find(|c|c.definition==id).unwrap(),
            "attachment"=>&g.attachments.iter().find(|a|a.card.definition==id).unwrap().card,
            _=>g.players[0].graveyard.iter().find(|c|c.definition==id).unwrap()};
        for viewer in 0..4 {
            let v=g.card_view(live,viewer,(d.kind=="character").then_some(0),None);
            assert_eq!(v.card_id.as_deref(),Some(id));assert_eq!(v.name,d.name);
        }
        println!("AB closure {id}: paid={}, actual {} entry, both restores, four views, construction rejected",d.cost,d.kind);
    }
}

#[test]
fn ab_card_closure_bq031_bq032_day_nonasset_never_grants_public() {
    for id in ["BQ031","BQ032"] {for exposed in [false,true] {
        let mut g=layout();fund(&mut g,0,id,4);let uid=held(&mut g,id,0);
        if exposed{g.players[0].society_zone.exposure=1;}else{g.players[2].society_zone.exposure=1;}
        assert!(!definition(id).traits.public);
        act(&mut g,0,Action{card_id:Some(uid),region:Some(0),..Action::new("conceal")});checkpoint(&g);
        let hidden=g.regions[0].cards.iter().find(|c|c.definition==id).unwrap();assert!(hidden.face_down);
        for viewer in 0..4 {
            let v=g.card_view(hidden,viewer,Some(0),None);
            assert_eq!(v.card_id.is_some(),viewer==0,"{id}, exposure={exposed}");
        }
    }}
}
