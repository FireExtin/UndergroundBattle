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
        if condition==1 {g.regions[1].destroyed=true;} if condition==2 {g.players[0].destruction_markers=1;}
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
