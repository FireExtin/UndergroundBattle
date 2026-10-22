//! Current, playable Society attachments, including the stable empty zone.
//! The original FAQ explicitly includes Society attachments when strengthened.
use crate::{catalog,model::*,unit_support::*};
fn layout(with_society:bool)->Game{
    let mut g=if with_society{crate::msjc01_tests::game()}else{game()};
    for p in &mut g.players{p.hand.clear();p.assets.clear();p.graveyard.clear();}
    for r in &mut g.regions{r.cards.clear();r.influence=[0,0];}
    g.first_team=0;g.begin_window(Window::Action(0));g
}
fn society_attachment(g:&mut Game)->String{
    fund(g,0,"BQ040",3);let id=held(g,"JC034",0);
    act(g,0,Action{card_id:Some(id),target_id:Some("society:p0".into()),..Action::new("play")});top(g);
    g.attachments.iter().find(|a|a.card.definition=="JC034").unwrap().card.id.clone()
}
fn yellow_without_mind()->String{catalog::catalog().cards.iter().find(|d|d.color=="黄"&&d.magic.is_empty()&&d.supported).unwrap().id.clone()}

#[test]
fn jc005_society_real_paid_play_targets_live_society_attachment_with_or_without_society_card(){
    for with_society in [false,true]{
        let mut g=layout(with_society);let source=society_attachment(&mut g);let before_society=serde_json::to_value(&g.players[0].society_zone).unwrap();
        fund(&mut g,0,&yellow_without_mind(),2);let spell=held(&mut g,"JC005",0);
        let other_mind=g.make_card("JC005",1);g.players[1].assets.push(other_mind);
        let action=Action{card_id:Some(spell.clone()),target_id:Some(source.clone()),..Action::new("play")};
        reject(&mut g,0,action.clone());assert!(!g.legal_actions(0).iter().any(|a|a.action.card_id.as_ref()==Some(&spell)&&a.action.target_id.as_ref()==Some(&source)));
        let mut mind=g.make_card("JC005",0);mind.exhausted=true;g.players[0].assets.push(mind);restore(&mut g);
        assert!(g.legal_actions(0).iter().any(|a|a.action.kind=="play"&&a.action.card_id.as_ref()==Some(&spell)&&a.action.target_id.as_ref()==Some(&source)),"strengthened JC005 must enumerate an actual Society attachment");
        let resources=g.resources(0);act(&mut g,0,action);assert_eq!(g.resources(0),resources-2);top(&mut g);
        assert!(!g.attachments.iter().any(|a|a.card.id==source));assert!(g.players[0].graveyard.iter().any(|c|c.definition=="JC034"));
        assert_eq!(serde_json::to_value(&g.players[0].society_zone).unwrap(),before_society);assert_eq!(g.society_host_seat("society:p0"),Some(0));restore(&mut g);
    }
}

#[test]
fn jc005_society_strengthened_guard_accepts_other_players_live_zone_but_rejects_invalid_host_fixtures(){
    let mut g=layout(false);let source=society_attachment(&mut g);fund(&mut g,0,&yellow_without_mind(),2);let spell=held(&mut g,"JC005",0);
    let c=g.make_card("JC005",0);g.players[0].assets.push(c);
    let s=g.source_snapshot(g.players[0].hand.iter().find(|c|c.id==spell).unwrap(),None);let slot=crate::rules::definition("JC005").abilities[0].targets[0].clone();
    assert!(g.valid_binding(0,&s,&slot,&source));
    let i=g.attachments.iter().position(|a|a.card.id==source).unwrap();g.attachments[i].card.controller=1;g.attachments[i].host_id="society:p1".into();
    assert!(g.valid_binding(0,&s,&slot,&source));restore(&mut g);
    // Pure invalid-host predicate fixtures; never described as a saved legal game.
    for host in ["society:p0","society:p99","i999999999"]{g.attachments[i].host_id=host.into();assert!(!g.valid_binding(0,&s,&slot,&source));}
}
