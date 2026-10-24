use crate::{catalog, import20_tattoo as t, model::*, rules::*, unit_support::*};
use crate::jc029_tests::pass_top;
fn attach(g:&mut Game,actor:usize,host:&str)->String {
    g.begin_window(Window::Action(g.team(actor))); fund(g,actor,"JC084",6); let hand=held(g,t::ID,actor);
    act(g,actor,Action{card_id:Some(hand),target_id:Some(host.into()),..Action::new("play")});pass_top(g);
    g.attachments.iter().rev().find(|a|a.card.definition==t::ID).unwrap().card.id.clone()
}
fn action(source:&str,cost:&str,target:&str)->Action {Action{card_id:Some(source.into()),ability_id:Some(t::FAST.into()),cost_selected:Some(vec![cost.into()]),target_id:Some(target.into()),..Action::new("activate")}}
fn hidden(g:&mut Game,id:&str,actor:usize)->String {let uid=field(g,id,actor);g.board_mut(&uid).unwrap().face_down=true;uid}
fn priority(g:&mut Game,actor:usize){for _ in 0..8{if g.can_fast(actor){return;}pass(g);}panic!("missing priority");}
#[test]
fn import20_wm064_exact_printed_attachment_and_closed_definition(){
    let c=catalog::card(t::ID);assert_eq!((c.cost,c.color.as_str(),c.kind.as_str()),(6,"黑","attachment"));assert_eq!(c.loyalty,["黑色"]);
    assert!(!c.supported&&!c.unique);assert!(catalog::runtime_implemented(c));let d=definition(t::ID);assert!(d.attachment.is_some());
    assert_eq!(d.attachment.as_ref().unwrap().host_icons,Icons{investigation:1,combat:2,influence:1});assert_eq!(d.attachment.as_ref().unwrap().host_defense_bonus,2);
    let mut all=definitions().clone();all.get_mut(t::ID).unwrap().abilities.pop();assert!(validate_definitions(&all).is_err());
    let mut all=definitions().clone();all.get_mut("JC125").unwrap().abilities.push(d.abilities[1].clone());assert!(validate_definitions(&all).is_err());
    validate_definitions(definitions()).unwrap();
}
#[test]
fn import20_wm064_real_paid_attachment_adds_host_icons_defense_status_and_four_views(){
    let mut g=game();let host=field(&mut g,"JC125",0);let ally=field(&mut g,"JC125",1);
    let base=g.icons(g.board(&host).unwrap().1,0);let defense=g.defense(g.board(&host).unwrap().1,0);let source=attach(&mut g,0,&host);
    let c=g.board(&host).unwrap().1;assert_eq!(g.icons(c,0),base.add(Icons{investigation:1,combat:2,influence:1}));assert_eq!(g.defense(c,0),defense+2);
    assert!(g.current_subtypes(c).iter().any(|s|s=="罪犯"));assert!(g.current_leader(c)&&g.has_renown(c));assert!(!g.current_leader(g.board(&ally).unwrap().1));
    for seat in 0..4 {let c=g.card_view(c,seat,Some(0),None);assert_eq!(c.current_leader,Some(true));assert_eq!(c.current_renown,Some(true));assert!(c.current_subtypes.as_ref().unwrap().iter().any(|s|s=="罪犯"));}
    g.board_mut(&source).unwrap().exhausted=true;assert!(g.current_leader(g.board(&host).unwrap().1));g.first_team=1;
    assert_eq!(g.icons(g.board(&host).unwrap().1,0),base.add(Icons{investigation:1,combat:2,influence:1}));restore(&mut g);
}
#[test]
fn import20_wm064_rejects_ally_enemy_hidden_nonhuman_host_and_loyalty_without_payment(){
    let mut g=game();let own=field(&mut g,"JC125",0);let ally=field(&mut g,"JC125",1);let enemy=field(&mut g,"JC125",2);let other=field(&mut g,"JC029",0);let dark=hidden(&mut g,"JC125",0);
    fund(&mut g,0,"JC125",6);let hand=held(&mut g,t::ID,0);
    reject(&mut g,0,Action{card_id:Some(hand.clone()),target_id:Some(own),..Action::new("play")});fund(&mut g,0,"JC084",1);
    for id in [ally,enemy,other,dark]{reject(&mut g,0,Action{card_id:Some(hand.clone()),target_id:Some(id),..Action::new("play")});}
    assert_eq!(g.resources(0),7);
}
#[test]
fn import20_wm064_granted_criminal_host_is_real_cost_and_departed_source_still_reveals(){
    let mut g=game();let host=field(&mut g,"JC125",0);let source=attach(&mut g,0,&host);let target=hidden(&mut g,"JC084",0);fund(&mut g,0,"JC125",1);
    let a=action(&source,&host,&target);assert!(g.legal_actions(0).iter().any(|x|x.action==a));act(&mut g,0,a);
    assert!(g.board(&host).is_none());assert!(!g.attachments.iter().any(|a|a.card.id==source));assert_eq!(g.resources(0),0);pass_top(&mut g);
    assert!(g.board(&target).is_none());assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC084"&&!c.face_down));
    assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition==t::ID).count(),1);restore(&mut g);
}
#[test]
fn import20_wm064_noncriminal_hidden_front_spends_all_costs_but_stays_hidden(){
    let mut g=game();let host=field(&mut g,"JC125",0);let source=attach(&mut g,0,&host);let paid=field(&mut g,"JC084",0);let target=hidden(&mut g,"JC125",0);fund(&mut g,0,"JC125",1);
    let a=action(&source,&paid,&target);assert!(g.legal_actions(0).iter().any(|x|x.action==a));act(&mut g,0,a);assert!(g.board(&paid).is_none());pass_top(&mut g);
    assert!(g.board(&target).unwrap().1.face_down);assert!(g.board(&source).unwrap().1.exhausted);assert_eq!(g.resources(0),0);assert!(g.current_leader(g.board(&host).unwrap().1));
}
#[test]
fn import20_wm064_no_printed_payment_or_loyalty_is_required_for_forced_reveal(){
    let mut g=game();let host=field(&mut g,"JC125",0);let source=attach(&mut g,0,&host);let paid=field(&mut g,"JC084",0);
    g.players[0].assets.clear(); // explicit legal no-loyalty layout; exhausted assets would still grant loyalty
    let target=hidden(&mut g,"WM058",0);fund(&mut g,0,"JC125",1);
    assert!(!g.loyalty(0,"WM058"));act(&mut g,0,action(&source,&paid,&target));pass_top(&mut g);
    assert!(g.regions[0].cards.iter().any(|c|c.definition=="WM058"&&!c.face_down));assert_eq!(g.resources(0),0);restore(&mut g);
}
#[test]
fn import20_wm064_illegal_costs_foreign_hidden_targets_and_payloads_rollback(){
    let mut g=game();let host=field(&mut g,"JC125",0);let source=attach(&mut g,0,&host);let target=hidden(&mut g,"JC084",0);let good=field(&mut g,"JC084",0);
    let ally=field(&mut g,"JC084",1);let enemy=field(&mut g,"JC084",2);let not_criminal=field(&mut g,"JC125",0);let dark_cost=hidden(&mut g,"JC084",0);let foreign=hidden(&mut g,"JC084",2);fund(&mut g,0,"JC125",1);
    for id in [ally,enemy,not_criminal,dark_cost]{reject(&mut g,0,action(&source,&id,&target));}
    reject(&mut g,0,action(&source,&good,&foreign));
    reject(&mut g,0,Action{region:Some(0),..action(&source,&good,&target)});
    g.board_mut(&source).unwrap().exhausted=true;reject(&mut g,0,action(&source,&good,&target));assert_eq!(g.resources(0),1);
}
#[test]
fn import20_wm064_actual_disintegration_response_leaves_paid_effect_independent(){
    let mut g=game();let host=field(&mut g,"JC125",0);let source=attach(&mut g,0,&host);let paid=field(&mut g,"JC084",0);let target=hidden(&mut g,"JC084",0);fund(&mut g,0,"JC125",1);
    act(&mut g,0,action(&source,&paid,&target));fund(&mut g,2,"JC005",2);let spell=held(&mut g,"JC005",2);priority(&mut g,2);
    act(&mut g,2,Action{card_id:Some(spell),target_id:Some(source.clone()),..Action::new("play")});pass_top(&mut g);assert!(g.board(&source).is_none());pass_top(&mut g);
    assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC084"&&!c.face_down));assert!(!g.current_leader(g.board(&host).unwrap().1));
}
#[test]
fn import20_wm064_additional_leader_uses_existing_mandatory_sacrifice_choice(){
    let mut g=game();let old=field(&mut g,"WM025",0);let host=field(&mut g,"JC125",0);attach(&mut g,0,&host);
    assert!(g.pending.as_ref().is_some_and(|p|matches!(p.resolution,ChoiceResolution::Unique)));
    let ids=g.pending.as_ref().unwrap().choice.options.iter().map(|c|c.id.clone()).collect::<Vec<_>>();assert!(ids.contains(&host)&&ids.contains(&old));
    choose(&mut g,vec![old]);assert!(g.board(&host).is_some());assert!(g.current_leader(g.board(&host).unwrap().1));restore(&mut g);
}
#[test]
fn import20_wm064_real_tattoo_removal_clears_status_and_hidden_views_do_not_leak(){
    let mut g=game();let host=field(&mut g,"JC125",0);let source=attach(&mut g,0,&host);fund(&mut g,2,"JC005",2);let spell=held(&mut g,"JC005",2);priority(&mut g,2);
    act(&mut g,2,Action{card_id:Some(spell),target_id:Some(source),..Action::new("play")});pass_top(&mut g);
    assert!(!g.current_leader(g.board(&host).unwrap().1));assert!(!g.has_renown(g.board(&host).unwrap().1));assert!(!g.current_subtypes(g.board(&host).unwrap().1).iter().any(|s|s=="罪犯"));
    assert!(g.hide_in_place(&host));let dark=g.regions[0].cards.iter().find(|c|c.face_down&&c.controller==0).unwrap().clone();for seat in 0..4{let c=g.card_view(&dark,seat,Some(0),None);assert!(c.current_leader.is_none()&&c.current_renown.is_none());}restore(&mut g);
}
#[test]
fn import20_wm064_control_primitive_human_and_controller_requirements_cleanup_grant(){
    let mut g=game();let host=field(&mut g,"JC125",0);attach(&mut g,0,&host);
    g.add_control(&host,2,ControlLifetime::TurnEnd{turn:g.turn},SubtypeChange::HumanToVampire);g.settle_deaths();
    assert!(g.attachments.is_empty());assert!(!g.current_leader(g.board(&host).unwrap().1));assert!(!g.has_renown(g.board(&host).unwrap().1));restore(&mut g);
}
